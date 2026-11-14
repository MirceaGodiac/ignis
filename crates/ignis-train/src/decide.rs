//! Observation-value ranking (V3.1 chapter 11): one-step preposterior analysis on recorded data,
//! and equal-budget replay (§11.5, §24.6).
//!
//! An action reveals an already-recorded but not-yet-admitted measurement at a time <= cutoff. The
//! ranking ([`decide`]) sees only the `PrefixBundle` and the candidates' metadata
//! (`AcquisitionAction`); candidate values arrive through a separate `reveal` callable only after a
//! choice, and the continuation (t > cutoff) is never an input. Scoring against the continuation is
//! a separate evaluator step after every forecast is sealed.
//!
//! Target Z (fixed before scoring):
//! * "event_probs": the model-conditional vector (A, F_H, F_R, U) at horizon h, quadratic loss,
//!   R(D) = sum_k Var[Z_k | D];
//! * "diameter": latent D(h) conditioned on active & in support at h, squared loss (m^2).
//!
//! U(j) = R(D) - E_{Y_j | D}[R(D ∪ {Y_j})] - λ C(j). Hypothetical Y_j are drawn from the exact
//! conditional Student-t of the candidate given the prefix (§7.5); the updated posterior reweights
//! every particle by p(Y_j | particle, prefix). Common random numbers are shared by all actions.

use std::collections::BTreeMap;

use anyhow::{bail, Context};
use clap::Subcommand;
use rand::{RngExt, SeedableRng};
use serde_json::{json, Value};

use ignis_core::analysis::phase_established_at;
use ignis_core::contracts::DecisionTrace;
use ignis_core::contracts::{
    AcquisitionAction, AnalysisArtifact, CandidateScore, DecisionPolicy, ExperimentRecord,
    Observation, Partition, PrefixBundle, SealedPrediction, SplitPartition,
};
use ignis_core::expert::{find, Expert, Refusal};
use ignis_core::hashing::content_hash;
use ignis_core::prefix::{build_prefix, LeakError, PrefixInputs, ProspectivePolicy};
use ignis_core::registry::{default_root, load, LoadedRelease};
use ignis_core::views::{FlexTrajectory, RunArchive, View, ViewSource};
use ignis_science::predictive::{systematic_resample, CandidatePredictive, VoiParticles};
use ignis_science::studentt::{sample_t, t_logpdf_1d};

use crate::{find_fixture, Fixture};

// ------------------------------------------------------------------------------ VOI core

/// Deterministic removal of inadmissible actions before scoring (§11.2).
pub fn admissible(
    prefix: &PrefixBundle,
    actions: &[AcquisitionAction],
) -> (Vec<AcquisitionAction>, Vec<Value>) {
    let inside: std::collections::BTreeSet<&str> =
        prefix.observations.iter().map(|o| o.id.as_str()).collect();
    let mut sorted = actions.to_vec();
    sorted.sort_by(|a, b| a.time_s.total_cmp(&b.time_s).then_with(|| a.id.cmp(&b.id)));
    let (mut keep, mut dropped) = (vec![], vec![]);
    for a in sorted {
        let why = if a.run_id != prefix.run_id {
            Some("other run")
        } else if a.time_s > prefix.cutoff_s {
            Some("after cutoff (future is sealed)")
        } else if inside.contains(a.observation_id.as_str()) {
            Some("already admitted")
        } else if a.time_s < 0.0 {
            Some("before phase entry")
        } else {
            None
        };
        match why {
            Some(w) => dropped.push(json!({"action_id": a.id, "reason": w})),
            None => keep.push(a),
        }
    }
    (keep, dropped)
}

/// Weighted variance summed over target components; colw multiplies the weights.
fn risk(z: &[Vec<f64>], w: &[f64], colw: Option<&[f64]>) -> f64 {
    let ww: Vec<f64> = w
        .iter()
        .enumerate()
        .map(|(i, x)| x * colw.map_or(1.0, |c| c[i]))
        .collect();
    let tot: f64 = ww.iter().sum();
    if !(tot > 0.0) {
        return 0.0;
    }
    let k = z.first().map_or(0, |r| r.len());
    (0..k)
        .map(|c| {
            let m: f64 = ww.iter().zip(z).map(|(a, r)| a / tot * r[c]).sum();
            let m2: f64 = ww.iter().zip(z).map(|(a, r)| a / tot * r[c] * r[c]).sum();
            (m2 - m * m).max(0.0)
        })
        .sum()
}

fn target(vp: &VoiParticles, t: &str) -> anyhow::Result<(Vec<Vec<f64>>, Option<Vec<f64>>)> {
    match t {
        "event_probs" => Ok((
            vp.z_event
                .clone()
                .context("this expert has no event model; use target=diameter")?,
            None,
        )),
        "diameter" => Ok((
            vp.z_diameter.iter().map(|d| vec![*d]).collect(),
            Some(vp.z_diameter_weight.clone()),
        )),
        o => bail!("unknown VOI target {o:?}"),
    }
}

/// Returns (R0, expected risk (J), MC standard error (J), utility (J)).
pub fn preposterior(
    vp: &VoiParticles,
    tgt: &str,
    cost: &[f64],
    lam: f64,
    n_outcomes: usize,
    seed: u64,
) -> anyhow::Result<(f64, Vec<f64>, Vec<f64>, Vec<f64>)> {
    let w = &vp.w;
    let (z, colw) = target(vp, tgt)?;
    let r0 = risk(&z, w, colw.as_deref());
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let pidx = systematic_resample(w, n_outcomes, &mut rng);
    let zt: Vec<f64> = pidx
        .iter()
        .map(|&p| sample_t(&mut rng, vp.dof[p]))
        .collect();
    let jn = vp.pred_loc.first().map_or(0, |r| r.len());
    let (mut er, mut se, mut ut) = (vec![0.0; jn], vec![0.0; jn], vec![0.0; jn]);
    for j in 0..jn {
        let rm: Vec<f64> = pidx
            .iter()
            .zip(&zt)
            .map(|(&p, &zz)| {
                let y = vp.pred_loc[p][j] + vp.pred_scale2[p][j].sqrt() * zz;
                let ll: Vec<f64> = (0..w.len())
                    .map(|q| t_logpdf_1d(y, vp.pred_loc[q][j], vp.pred_scale2[q][j], vp.dof[q]))
                    .collect();
                let mx = ll.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let wm: Vec<f64> = w.iter().zip(&ll).map(|(a, l)| a * (l - mx).exp()).collect();
                risk(&z, &wm, colw.as_deref())
            })
            .collect();
        let m = rm.iter().sum::<f64>() / rm.len() as f64;
        er[j] = m;
        se[j] = if rm.len() > 1 {
            (rm.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (rm.len() - 1) as f64).sqrt()
                / (rm.len() as f64).sqrt()
        } else {
            f64::NAN
        };
        ut[j] = r0 - er[j] - lam * cost[j];
    }
    Ok((r0, er, se, ut))
}

pub fn predictive_variance(vp: &VoiParticles) -> Vec<f64> {
    let jn = vp.pred_loc.first().map_or(0, |r| r.len());
    (0..jn)
        .map(|j| {
            let m: f64 = vp.w.iter().zip(&vp.pred_loc).map(|(w, l)| w * l[j]).sum();
            let s: f64 = (0..vp.w.len())
                .map(|p| {
                    vp.w[p]
                        * (vp.pred_scale2[p][j] * vp.dof[p] / (vp.dof[p] - 2.0)
                            + vp.pred_loc[p][j].powi(2))
                })
                .sum();
            s - m * m
        })
        .collect()
}

fn fin(x: f64) -> Value {
    if x.is_finite() {
        json!(x)
    } else {
        Value::Null
    }
}

/// Rank candidate observations and choose one under `policy`. Never sees candidate values.
pub fn decide(
    model: &dyn CandidatePredictive,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    actions: &[AcquisitionAction],
    policy: DecisionPolicy,
    s: &Value,
) -> anyhow::Result<DecisionTrace> {
    let tgt = s
        .get("target")
        .and_then(|v| v.as_str())
        .unwrap_or(if model.has_event_model() {
            "event_probs"
        } else {
            "diameter"
        })
        .to_string();
    let lam = s.get("cost_weight").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let mo = s.get("n_outcomes").and_then(|v| v.as_u64()).unwrap_or(16) as usize;
    let seed = s.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);
    let (keep, dropped) = admissible(prefix, actions);
    let mut mc = json!({"n_outcomes": mo, "seed": seed, "dropped_actions": dropped, "target_horizon_s": s.get("target_horizon_s").and_then(|v| v.as_f64()).unwrap_or(2.0), "n_candidates": keep.len()});
    let mut cands = vec![];
    let mut chosen = None;
    if !keep.is_empty() {
        match policy {
            DecisionPolicy::Random => {
                chosen = Some(
                    keep[rand::rngs::StdRng::seed_from_u64(seed).random_range(0..keep.len())]
                        .id
                        .clone(),
                )
            }
            DecisionPolicy::Uniform => {
                let have: Vec<f64> = if prefix.observations.is_empty() {
                    vec![0.0]
                } else {
                    prefix.observations.iter().map(|o| o.time_s).collect()
                };
                let gap: Vec<f64> = keep
                    .iter()
                    .map(|a| {
                        have.iter()
                            .map(|h| (h - a.time_s).abs())
                            .fold(f64::INFINITY, f64::min)
                    })
                    .collect();
                let gmax = gap.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let best = (0..keep.len())
                    .rev()
                    .find(|&j| gap[j] >= gmax - 1e-12)
                    .unwrap();
                chosen = Some(keep[best].id.clone()); // ties -> latest
                mc["uniform_gap_s"] = json!(gap);
            }
            DecisionPolicy::MaxVariance | DecisionPolicy::Voi => {
                let times: Vec<f64> = keep.iter().map(|a| a.time_s).collect();
                let vp = model.voi_particles(release, prefix, &times, s)?;
                if vp.w.is_empty() {
                    mc["refused"] = json!({"gate": vp.support.gate, "reasons": vp.support.reasons});
                } else {
                    mc["particles"] = vp.particles.clone();
                    if policy == DecisionPolicy::MaxVariance {
                        let v = predictive_variance(&vp);
                        let j = (0..v.len()).max_by(|&a, &b| v[a].total_cmp(&v[b])).unwrap();
                        chosen = Some(keep[j].id.clone());
                        mc["predictive_variance_m2"] = json!(v);
                    } else {
                        let cost: Vec<f64> = keep.iter().map(|a| a.cost).collect();
                        let (r0, er, se, u) = preposterior(&vp, &tgt, &cost, lam, mo, seed)?;
                        mc["current_risk"] = json!(r0);
                        cands = keep
                            .iter()
                            .enumerate()
                            .map(|(j, a)| CandidateScore {
                                action_id: a.id.clone(),
                                expected_risk: er[j],
                                utility: u[j],
                                mc_standard_error: se[j].is_finite().then_some(se[j]),
                            })
                            .collect();
                        let jmax = (0..u.len()).max_by(|&a, &b| u[a].total_cmp(&u[b])).unwrap();
                        let tol = 2.0 * if se[jmax].is_finite() { se[jmax] } else { 0.0 };
                        let ties: Vec<usize> =
                            (0..keep.len()).filter(|&j| u[j] >= u[jmax] - tol).collect();
                        let jc = *ties
                            .iter()
                            .min_by(|&&a, &&b| {
                                keep[a]
                                    .cost
                                    .total_cmp(&keep[b].cost)
                                    .then(u[b].total_cmp(&u[a]))
                                    .then(keep[a].id.cmp(&keep[b].id))
                            })
                            .unwrap();
                        chosen = Some(keep[jc].id.clone());
                        mc["practical_tie_set"] =
                            json!(ties.iter().map(|&j| keep[j].id.clone()).collect::<Vec<_>>());
                        mc["tie_rule"] =
                            json!("within 2 MC standard errors of the best -> cheapest");
                    }
                }
            }
        }
    }
    let mut tr = DecisionTrace {
        id: String::new(),
        prefix_hash: prefix.input_hash.clone(),
        model_release_id: release.release.id.clone(),
        target: tgt.clone(),
        loss: if tgt == "event_probs" {
            "quadratic".into()
        } else {
            "squared_error_m2".into()
        },
        cost_weight: lam,
        mc_settings: match clean(mc) {
            Value::Object(m) => m,
            _ => Default::default(),
        },
        candidates: cands,
        chosen_action_id: chosen,
        policy,
        updated_artifact_id: None,
        realized_score: None,
    };
    tr.id = content_hash(&tr, &["id"]);
    Ok(tr)
}

fn clean(v: Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(m.into_iter().map(|(k, v)| (k, clean(v))).collect()),
        Value::Array(a) => Value::Array(a.into_iter().map(clean).collect()),
        Value::Number(n) => fin(n.as_f64().unwrap_or(f64::NAN)),
        o => o,
    }
}

// ------------------------------------------------------------------------------ archive episodes

/// Initial sparse prefix + candidate pool (metadata only) + the vault holding the candidate values.
pub fn build_episode(
    archive: &RunArchive,
    cutoff: f64,
    stride: usize,
    n_candidates: usize,
    policy: &ProspectivePolicy,
) -> anyhow::Result<(
    PrefixBundle,
    Vec<AcquisitionAction>,
    BTreeMap<String, Observation>,
)> {
    let past: Vec<&Observation> = archive
        .observations
        .iter()
        .filter(|o| o.time_s <= cutoff && o.quality_flags.is_empty())
        .collect();
    let initial: Vec<Observation> = past
        .iter()
        .step_by(stride.max(1))
        .map(|o| (*o).clone())
        .collect();
    let ids: std::collections::BTreeSet<&str> = initial.iter().map(|o| o.id.as_str()).collect();
    let mut rest: Vec<&Observation> = past
        .iter()
        .copied()
        .filter(|o| !ids.contains(o.id.as_str()))
        .collect();
    if rest.len() > n_candidates {
        let n = rest.len();
        rest = (0..n_candidates)
            .map(|i| {
                rest[((i as f64) * (n - 1) as f64 / (n_candidates - 1).max(1) as f64).round()
                    as usize]
            })
            .collect();
    }
    let pool = rest
        .iter()
        .map(|o| AcquisitionAction {
            id: format!("act-{}", o.id),
            run_id: o.run_id.clone(),
            observation_id: o.id.clone(),
            time_s: o.time_s,
            cost: 1.0,
            quality: None,
        })
        .collect();
    let vault = rest.iter().map(|o| (o.id.clone(), (*o).clone())).collect();
    let phase = phase_established_at(archive, cutoff);
    let support_evidence = if phase.is_some() {
        vec!["activity confirmed at the cutoff by the run record".to_string()]
    } else {
        vec![]
    };
    let prefix = build_prefix(
        &archive.record,
        &initial,
        cutoff,
        policy,
        &archive.time_origin,
        PrefixInputs {
            phase_established_at_s: phase,
            support_evidence,
            ..Default::default()
        },
    )?;
    Ok((prefix, pool, vault))
}

// ------------------------------------------------------------------------------ replay

pub struct ReplayResult {
    pub traces: Vec<DecisionTrace>,
    pub artifacts: Vec<Option<AnalysisArtifact>>,
    pub refusals: Vec<Option<String>>,
    pub seals: Vec<SealedPrediction>,
    pub final_prefix: PrefixBundle,
}

/// Equal-budget replay: after each choice the archived value comes only through `reveal`, the
/// prefix is rebuilt at the *same* cutoff and a new forecast is produced and sealed.
#[allow(clippy::too_many_arguments)]
pub fn replay(
    expert: &dyn Expert,
    model: &dyn CandidatePredictive,
    release: &LoadedRelease,
    record: &ExperimentRecord,
    initial: &PrefixBundle,
    pool: &[AcquisitionAction],
    reveal: &mut dyn FnMut(&str) -> Observation,
    budget: usize,
    policy: DecisionPolicy,
    settings: &Value,
    forecast_settings: &Value,
) -> anyhow::Result<ReplayResult> {
    let mut prefix = initial.clone();
    let mut admitted = initial.observations.clone();
    let mut remaining = pool.to_vec();
    let mut out = ReplayResult {
        traces: vec![],
        artifacts: vec![],
        refusals: vec![],
        seals: vec![],
        final_prefix: initial.clone(),
    };
    for step in 0..budget {
        let mut s = settings.clone();
        s["seed"] = json!(settings.get("seed").and_then(|v| v.as_u64()).unwrap_or(0) + step as u64);
        let mut tr = decide(model, release, &prefix, &remaining, policy, &s)?;
        let Some(cid) = tr.chosen_action_id.clone() else {
            out.traces.push(tr);
            break;
        };
        let act = remaining.iter().find(|a| a.id == cid).unwrap().clone();
        let o = reveal(&act.observation_id);
        if o.id != act.observation_id
            || o.time_s > prefix.cutoff_s
            || o.available_at_s > prefix.cutoff_s
        {
            return Err(LeakError(format!(
                "revealed {} is not the chosen pre-cutoff observation",
                o.id
            ))
            .into());
        }
        admitted.push(o);
        remaining.retain(|a| a.id != act.id);
        prefix = build_prefix(
            record,
            &admitted,
            prefix.cutoff_s,
            expert.policy(),
            &initial.time_origin,
            PrefixInputs {
                phase_established_at_s: initial.phase_established_at_s,
                support_evidence: initial.support_evidence.clone(),
                transform_versions: initial.transform_versions.clone(),
                ..Default::default()
            },
        )?;
        match expert.forecast(release, &prefix, forecast_settings) {
            Ok(art) => {
                out.seals.push(SealedPrediction {
                    artifact_id: art.id.clone(),
                    artifact_hash: content_hash(&art, &["created_at"]),
                    sealed_at: chrono::Utc::now().to_rfc3339(),
                    split_partition: SplitPartition::Validation,
                    run_id: Some(prefix.run_id.clone()),
                    split_id: Some(release.release.split_id.clone()),
                    prefix_hash: Some(prefix.input_hash.clone()),
                });
                tr.updated_artifact_id = Some(art.id.clone());
                out.artifacts.push(Some(art));
                out.refusals.push(None);
            }
            Err(e) => match Refusal::from_error(&e) {
                Some(r) => {
                    out.artifacts.push(None);
                    out.refusals.push(Some(r.to_string()));
                }
                None => return Err(e),
            },
        }
        out.traces.push(tr);
    }
    out.final_prefix = prefix;
    Ok(out)
}

/// Evaluator step (after sealing): Brier score of the event table at `horizon_s`.
pub fn score_events(a: &AnalysisArtifact, horizon_s: f64, outcome: &str) -> Option<f64> {
    if a.event_cdf.is_empty() || !a.event_categories.iter().any(|c| c == outcome) {
        return None;
    }
    let j = (0..a.times_s.len()).min_by(|&x, &y| {
        (a.times_s[x] - horizon_s)
            .abs()
            .total_cmp(&(a.times_s[y] - horizon_s).abs())
    })?;
    if (a.times_s[j] - horizon_s).abs() > 1e-9 {
        return None;
    }
    Some(
        a.event_categories
            .iter()
            .zip(&a.event_cdf)
            .map(|(c, row)| (row[j] - if c == outcome { 1.0 } else { 0.0 }).powi(2))
            .sum(),
    )
}

/// Evaluator-only: the category realised at h from the view's endpoint (reads the continuation).
pub fn outcome_at(v: &FlexTrajectory, h: f64) -> Option<String> {
    match (
        v.censoring.as_str(),
        v.t_first_terminal_s,
        v.allowed_causes.first(),
    ) {
        ("event" | "interval", Some(t), Some(c)) if t <= h => Some(c.clone()),
        _ if v.t_last_active_s.is_some_and(|t| t >= h) || v.t_s.last().is_some_and(|t| *t >= h) => {
            Some("active_supported".into())
        }
        _ => None,
    }
}

// ------------------------------------------------------------------------------ CLI

#[derive(Debug, Subcommand)]
pub enum DecideCmd {
    /// Rank the not-yet-admitted pre-cutoff observations of one run.
    Rank {
        #[arg(long)]
        release: String,
        #[arg(long)]
        run: Option<String>,
        #[arg(long, default_value_t = 3.0)]
        cutoff: f64,
        #[arg(long, default_value = "voi")]
        policy: String,
        #[arg(long, default_value_t = 10)]
        stride: usize,
        #[arg(long, default_value_t = 12)]
        n_candidates: usize,
        #[arg(long, default_value_t = 16)]
        n_outcomes: usize,
        #[arg(long, default_value_t = 2.0)]
        horizon: f64,
        #[arg(long, default_value_t = 0.0)]
        cost_weight: f64,
        #[arg(long, default_value = "validation")]
        partition: String,
    },
    /// Equal-budget replay of several policies on archived runs.
    Replay {
        #[arg(long)]
        release: String,
        #[arg(long, default_value_t = 3)]
        runs: usize,
        #[arg(long, default_value_t = 3.0)]
        cutoff: f64,
        #[arg(long, default_value_t = 1)]
        budget: usize,
        #[arg(long, default_value = "voi,random,uniform,max_variance")]
        policies: String,
        #[arg(long, default_value_t = 10)]
        stride: usize,
        #[arg(long, default_value_t = 12)]
        n_candidates: usize,
        #[arg(long, default_value_t = 16)]
        n_outcomes: usize,
        #[arg(long, default_value_t = 2.0)]
        horizon: f64,
        #[arg(long, default_value = "validation")]
        partition: String,
    },
}

pub fn parse_policy(s: &str) -> anyhow::Result<DecisionPolicy> {
    Ok(serde_json::from_value(json!(s)).with_context(|| format!("unknown policy {s:?}"))?)
}

struct Setup {
    release: LoadedRelease,
    views: Vec<FlexTrajectory>,
    archives: BTreeMap<String, RunArchive>,
    split: ignis_core::contracts::Split,
}

fn setup(release_id: &str, data: &dyn ViewSource, fixtures: &[Fixture]) -> anyhow::Result<Setup> {
    let rel = load(&default_root(), release_id)?;
    let fam = rel.release.family;
    let fx = rel.release.config.get("fixture").cloned();
    let fixture_data;
    let source: &dyn ViewSource = match fx
        .as_ref()
        .and_then(|f| f.get("kind"))
        .and_then(|k| k.as_str())
    {
        Some(kind) => {
            let f = find_fixture(fixtures, kind, fam)?;
            let fxv = fx.as_ref().unwrap();
            fixture_data = (f.build)(
                fxv["n_runs"].as_u64().unwrap_or(24) as usize,
                fxv["seed"].as_u64().unwrap_or(0),
            )?;
            fixture_data.source.as_ref()
        }
        None => data,
    };
    let split = source.split(&rel.release.split_id)?;
    let views = source
        .views(fam, &rel.release.cohort_id)?
        .into_iter()
        .filter_map(|v| if let View::Flex(f) = v { Some(f) } else { None })
        .collect();
    let archives = source
        .run_archives(fam)?
        .into_iter()
        .map(|a| (a.record.id.clone(), a))
        .collect();
    Ok(Setup {
        release: rel,
        views,
        archives,
        split,
    })
}

fn partition_of(p: &str) -> anyhow::Result<Partition> {
    match p {
        "validation" => Ok(Partition::Validation),
        "train" => Ok(Partition::Train),
        _ => bail!("partition {p:?} refused: decisions are developed on train/validation only (test is off-limits)"),
    }
}

/// The acquisition scoring used by `POST /acquisition`: VOI ranking of `actions` with the release's
/// expert. Sees only the stored prefix and the actions' metadata, never candidate values or the
/// continuation.
pub fn score_candidates(
    experts: &[Box<dyn Expert>],
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    actions: &[AcquisitionAction],
    settings: &Value,
) -> anyhow::Result<DecisionTrace> {
    let name = &release.release.model_name;
    let e = find(experts, name).ok_or_else(|| {
        Refusal::unsupported("no_expert", format!("expert {name} not in this build"))
    })?;
    let m = e.candidate_predictive().ok_or_else(|| {
        Refusal::unsupported(
            "no_candidate_model",
            format!("{name} defines no candidate-measurement model"),
        )
    })?;
    decide(m, release, prefix, actions, DecisionPolicy::Voi, settings)
}

pub fn run(
    cmd: DecideCmd,
    experts: &[Box<dyn Expert>],
    data: &dyn ViewSource,
    fixtures: &[Fixture],
) -> anyhow::Result<()> {
    let pick = |s: &Setup| -> anyhow::Result<(&dyn Expert, &dyn CandidatePredictive)> {
        let name = &s.release.release.model_name;
        let e = find(experts, name).with_context(|| format!("expert {name} not in this build"))?;
        let m = e
            .candidate_predictive()
            .with_context(|| format!("{name} defines no candidate-measurement model (§11.2)"))?;
        Ok((e, m))
    };
    match cmd {
        DecideCmd::Rank {
            release,
            run,
            cutoff,
            policy,
            stride,
            n_candidates,
            n_outcomes,
            horizon,
            cost_weight,
            partition,
        } => {
            let s = setup(&release, data, fixtures)?;
            let part = partition_of(&partition)?;
            let (e, m) = pick(&s)?;
            let v = match run {
                Some(r) => s
                    .views
                    .iter()
                    .find(|v| v.run_id == r)
                    .context("run not in the release cohort")?,
                None => s
                    .views
                    .iter()
                    .find(|v| s.split.partition_of(&v.run_id) == Some(part))
                    .context("no run in partition")?,
            };
            anyhow::ensure!(
                s.split.partition_of(&v.run_id) != Some(Partition::Test),
                "test runs are off-limits"
            );
            let archive = s
                .archives
                .get(&v.run_id)
                .context("run has no stored records")?;
            let (prefix, pool, _vault) =
                build_episode(archive, cutoff, stride, n_candidates, e.policy())?;
            let tr = decide(
                m,
                &s.release,
                &prefix,
                &pool,
                parse_policy(&policy)?,
                &json!({"n_outcomes": n_outcomes, "target_horizon_s": horizon, "cost_weight": cost_weight}),
            )?;
            println!("{}", serde_json::to_string_pretty(&tr)?);
        }
        DecideCmd::Replay {
            release,
            runs,
            cutoff,
            budget,
            policies,
            stride,
            n_candidates,
            n_outcomes,
            horizon,
            partition,
        } => {
            let s = setup(&release, data, fixtures)?;
            let part = partition_of(&partition)?;
            let (e, m) = pick(&s)?;
            let pols: Vec<DecisionPolicy> = policies
                .split(',')
                .map(parse_policy)
                .collect::<anyhow::Result<_>>()?;
            let h = cutoff + horizon;
            let settings = json!({"n_outcomes": n_outcomes, "target_horizon_s": horizon});
            let fset = json!({"times_s": [cutoff, h]});
            let mut report = json!({"release": s.release.release.id, "budget": budget, "cutoff_s": cutoff, "horizon_s": h, "episodes": []});
            for v in s
                .views
                .iter()
                .filter(|v| s.split.partition_of(&v.run_id) == Some(part))
                .take(runs)
            {
                if !v.t_s.iter().any(|t| *t > cutoff) {
                    report["episodes"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"run": v.run_id, "skipped": "run ends before the cutoff"}));
                    continue;
                }
                let archive = s
                    .archives
                    .get(&v.run_id)
                    .context("run has no stored records")?;
                let rec = &archive.record;
                let (prefix, pool, vault) =
                    build_episode(archive, cutoff, stride, n_candidates, e.policy())?;
                let mut results = vec![];
                for &p in &pols {
                    let mut reveal = |id: &str| vault[id].clone();
                    results.push((
                        p,
                        replay(
                            e,
                            m,
                            &s.release,
                            rec,
                            &prefix,
                            &pool,
                            &mut reveal,
                            budget,
                            p,
                            &settings,
                            &fset,
                        )?,
                    ));
                }
                // evaluator step: only now is the continuation opened
                let outcome = outcome_at(v, h);
                let mut ep = json!({"run": v.run_id, "initial_observations": prefix.observations.len(), "pool": pool.len(), "policies": {}, "outcome_at_horizon": outcome});
                for (p, r) in results {
                    let sc: Vec<Value> = r
                        .artifacts
                        .iter()
                        .map(|a| match (a, &outcome) {
                            (Some(a), Some(o)) => {
                                score_events(a, h, o).map_or(Value::Null, |x| json!(x))
                            }
                            _ => Value::Null,
                        })
                        .collect();
                    ep["policies"][serde_json::to_value(p)?.as_str().unwrap()] = json!({
                        "chosen": r.traces.iter().map(|t| t.chosen_action_id.clone()).collect::<Vec<_>>(), "brier_after_each": sc,
                        "refusals": r.refusals, "trace_ids": r.traces.iter().map(|t| t.id.clone()).collect::<Vec<_>>(),
                        "seals": r.seals.iter().map(|x| x.artifact_hash.clone()).collect::<Vec<_>>()});
                }
                report["episodes"].as_array_mut().unwrap().push(ep);
            }
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }
    Ok(())
}

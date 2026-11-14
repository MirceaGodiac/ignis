//! FLEX experts implementing `ignis_core::expert::Expert`.
//!
//! * `flex.v3`            physics-neural model (θ curvature + b0, b1 + bounded residual R_φ + hazards), NUTS
//! * `flex.v1`            same likelihood without the residual (baseline)
//! * `flex.d2_population` constant-K D² law, K = K_ref exp(z(c)'β) from the population, D0 from the prefix
//! * `flex.d2_prefix`     constant-K D² law fitted on the prefix alone (broad fixed prior on log K)
//!
//! Every expert also exposes `objective` / `activations` (lab) and the candidate-measurement
//! predictive used by VOI ranking.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{bail, Context};
use rand::SeedableRng;
use rayon::prelude::*;
use serde_json::{json, Map, Value};

use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, EvidenceClass, Family, Lineage, ModelRelease, PrefixBundle,
    QualificationState, Quantity, Support, SupportGate, TrainedOn,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::hashing::content_hash;
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::{save_release, LoadedRelease, Tensor, TensorMap};
use ignis_core::resources::{configs_dir, Profile};
use ignis_core::views::{FlexTrajectory, View, FLEX_CAUSES};
use ignis_science::ad::{self, Real, Var};
use ignis_science::diagnostics::{summarize, Draws};
use ignis_science::events::{self, ParticleEvents};
use ignis_science::nuts::{neg_hessian_fd, run_nuts, NutsConfig, Precond};
use ignis_science::optim::{lbfgs, Adam, AdamConfig, AdamState, LbfgsConfig};
use ignis_science::predictive::{self as spred, CandidatePredictive, VoiParticles};
use ignis_science::studentt::{conditional_t, dense_scale_matrix, exp_kernel, sample_t};
use ignis_science::transforms::Layout;

use crate::data::{admin_censor, pack_views, prefix_arrays, PrefixData};
use crate::dynamics::{
    forecast_particle, natural_globals, run_state, trajectory_d, Model, Numerics, Particle,
};
use crate::inference::{self as inf, Joint};
use crate::structure::*;
use crate::support::{fit_condition_support, gate_prefix};

pub const QUANTILES: [f64; 5] = [0.05, 0.25, 0.5, 0.75, 0.95];

pub struct FlexExpert {
    st: Structure,
    policy: ProspectivePolicy,
}

fn policy(keys: &BTreeMap<String, String>) -> ProspectivePolicy {
    let mut allowed: Vec<&str> = keys.values().map(|s| s.as_str()).collect();
    allowed.push("fuel");
    ProspectivePolicy::new(Family::Flex, &allowed, &[])
}

impl FlexExpert {
    pub fn new(name: &str) -> FlexExpert {
        FlexExpert {
            st: Structure::get(name).expect("known FLEX model"),
            policy: policy(&default_condition_keys()),
        }
    }
    pub fn structure(&self) -> &Structure {
        &self.st
    }
}

// ------------------------------------------------------------------------------ config helpers

pub fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (k, v) in y {
                match x.get_mut(k) {
                    Some(slot) if slot.is_object() && v.is_object() => merge(slot, v),
                    _ => {
                        x.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

fn f(v: &Value, path: &str, d: f64) -> f64 {
    v.pointer(path).and_then(|x| x.as_f64()).unwrap_or(d)
}
fn u(v: &Value, path: &str, d: u64) -> u64 {
    v.pointer(path)
        .and_then(|x| x.as_u64().or_else(|| x.as_f64().map(|f| f as u64)))
        .unwrap_or(d)
}
fn s<'a>(v: &'a Value, path: &str, d: &'a str) -> &'a str {
    v.pointer(path).and_then(|x| x.as_str()).unwrap_or(d)
}

fn keys_from(cfg: &Value) -> BTreeMap<String, String> {
    let mut k = default_condition_keys();
    if let Some(Value::Object(m)) = cfg.get("condition_keys") {
        for (a, b) in m {
            if let Some(b) = b.as_str() {
                k.insert(a.clone(), b.to_string());
            }
        }
    }
    k
}

fn numerics_from(v: Option<&Value>) -> Numerics {
    v.and_then(|x| serde_json::from_value(x.clone()).ok())
        .unwrap_or_default()
}

/// The `[estimate]` planning model (V3.1 §15.5: budgeted time is not proven time).
/// `c_eval_s_cpu` is single-thread seconds per work unit (measured, see the FLEX configs); runs are
/// evaluated in parallel, so the per-gradient time divides by `parallel_efficiency * min(threads, runs)`.
/// FLEX fits are CPU-bound on every profile (no GPU path in Rust), so the CPU constant applies and
/// the profile matters only through its thread count.
pub fn estimate(cfg: &Value, n_runs: usize, n_obs: usize, threads: usize) -> Value {
    let (newton, nodes, panels) = (
        u(cfg, "/numerics/newton_steps", 8) as f64,
        u(cfg, "/numerics/n_nodes", 8) as f64,
        u(cfg, "/numerics/n_panels", 16) as f64,
    );
    let ev = u(cfg, "/numerics/event_nodes", 16) as f64;
    let work =
        n_obs as f64 * (newton * nodes + 4.0) + n_runs as f64 * (panels * nodes + 2.0 * ev * nodes);
    let c_eval = f(cfg, "/estimate/c_eval_s_cpu", 6e-8);
    let par =
        (f(cfg, "/estimate/parallel_efficiency", 0.6) * threads.min(n_runs).max(1) as f64).max(1.0);
    let grad_s = c_eval * work * f(cfg, "/estimate/residual_factor", 1.0) / par;
    let mut grads =
        u(cfg, "/fit/map/epochs", 20) as f64 * u(cfg, "/fit/map/steps_per_epoch", 50) as f64;
    if s(cfg, "/fit/method", "map") == "nuts" {
        grads += u(cfg, "/fit/nuts/chains", 4) as f64
            * (u(cfg, "/fit/nuts/warmup", 1000) + u(cfg, "/fit/nuts/samples", 1500)) as f64
            * f(cfg, "/estimate/mean_leapfrog", 64.0);
    }
    let secs = grads * grad_s + f(cfg, "/estimate/compile_s", 0.0);
    json!({"estimated_hours": secs / 3600.0, "gradient_evaluations": grads, "seconds_per_gradient": grad_s,
           "n_runs": n_runs, "n_obs": n_obs, "threads": threads, "basis": "declared planning constants (configs/training/defaults.toml); calibrate from the job ledger"})
}

fn jsonable(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.iter()
                .filter(|(k, _)| !k.starts_with('_'))
                .map(|(k, v)| (k.clone(), jsonable(v)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(jsonable).collect()),
        Value::Number(n) if n.as_f64().is_some_and(|x| !x.is_finite()) => Value::Null,
        o => o.clone(),
    }
}

fn obj(v: Value) -> Map<String, Value> {
    match v {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

fn finite(x: f64) -> Value {
    if x.is_finite() {
        json!(x)
    } else {
        Value::Null
    }
}

// ------------------------------------------------------------------------------ bound release

/// A release bound to its model description.
pub struct Bound {
    pub m: Model,
    pub pr: Priors,
    pub keys: BTreeMap<String, String>,
    pub local_ids: Vec<String>,
}

impl FlexExpert {
    fn model_for(&self, fs: FeatureSpec, num: Numerics, pr: &Priors) -> Model {
        Model::new(
            self.st.clone(),
            fs.clone(),
            num,
            Layout::new(global_specs(&self.st, &fs, pr)),
        )
    }

    pub fn bind(&self, rel: &ModelRelease) -> anyhow::Result<Bound> {
        let arch = &rel.architecture;
        let name = arch
            .get("structure")
            .and_then(|s| s.get("model_name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if name != self.st.model_name {
            bail!(
                "release {} is {}, not {}",
                rel.id,
                rel.model_name,
                self.st.model_name
            );
        }
        let fs: FeatureSpec =
            serde_json::from_value(arch.get("feature_spec").cloned().context("feature_spec")?)?;
        if fs.feature_order() != rel.feature_order {
            bail!(
                "feature order mismatch: {:?} != {:?}",
                rel.feature_order,
                fs.feature_order()
            );
        }
        let num = numerics_from(Some(&Value::Object(rel.numerics.clone())));
        let pr = merged_priors(rel.priors.get("hyper"));
        let local_ids = arch
            .get("local_run_ids")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        let keys = fs.condition_keys.clone();
        Ok(Bound {
            m: self.model_for(fs, num, &pr),
            pr,
            keys,
            local_ids,
        })
    }
}

// ------------------------------------------------------------------------------ MAP

struct MapOut {
    x: Vec<f64>,
    diag: Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MapCheckpoint {
    config_hash: String,
    epoch: usize,
    x: Vec<f64>,
    adam: AdamState,
    best: f64,
    best_x: Vec<f64>,
    history: Vec<Value>,
}

fn run_map(
    j: &Joint<'_>,
    init: Vec<f64>,
    cfg: &Value,
    ckpt: Option<&Path>,
    config_hash: &str,
) -> anyhow::Result<MapOut> {
    let t0 = Instant::now();
    let lr = f(cfg, "/learning_rate", 1e-2);
    let acfg = AdamConfig {
        learning_rate: lr,
        clip_norm: f(cfg, "/clip_norm", 10.0),
        weight_decay: f(cfg, "/weight_decay", 0.0),
        ..Default::default()
    };
    let epochs = u(cfg, "/epochs", 20) as usize;
    let spe = u(cfg, "/steps_per_epoch", 50) as usize;
    let patience = u(cfg, "/patience", 0) as usize;
    let mut opt = Adam::new(acfg, init.len());
    let mut x = init;
    let (mut start, mut best, mut best_x, mut history) =
        (0usize, f64::INFINITY, x.clone(), Vec::<Value>::new());
    let mut resumed = false;
    if let Some(p) = ckpt {
        if let Ok(text) = std::fs::read_to_string(p) {
            if let Ok(c) = serde_json::from_str::<MapCheckpoint>(&text) {
                if c.config_hash == config_hash && c.x.len() == x.len() {
                    start = c.epoch;
                    x = c.x;
                    opt.state = c.adam;
                    best = c.best;
                    best_x = c.best_x;
                    history = c.history;
                    resumed = true;
                }
            }
        }
    }
    let mut g = vec![0.0; x.len()];
    let mut since_best = 0;
    let mut grads = 0u64;
    for ep in start..epochs {
        for _ in 0..spe {
            let _ = j.value_grad(&x, &mut g);
            g.iter_mut().for_each(|v| *v = -*v);
            opt.step(&mut x, &mut g);
            grads += 1;
        }
        let v = -j.logp(&x);
        history.push(json!({"epoch": ep + 1, "train_neg_logp": finite(v), "selection": finite(v)}));
        if v.is_finite() && v < best {
            best = v;
            best_x = x.clone();
            since_best = 0;
        } else {
            since_best += 1;
        }
        if let Some(p) = ckpt {
            std::fs::create_dir_all(p.parent().unwrap())?;
            let c = MapCheckpoint {
                config_hash: config_hash.into(),
                epoch: ep + 1,
                x: x.clone(),
                adam: opt.state.clone(),
                best,
                best_x: best_x.clone(),
                history: history.clone(),
            };
            let tmp = p.with_extension("tmp");
            std::fs::write(&tmp, serde_json::to_vec(&c)?)?;
            std::fs::rename(&tmp, p)?;
        }
        if patience > 0 && since_best >= patience {
            break;
        }
    }
    let mut polish = Value::Null;
    if s(cfg, "/optimizer", "adam") == "adam+lbfgs" || s(cfg, "/optimizer", "adam") == "lbfgs" {
        let lc = LbfgsConfig {
            max_iter: u(cfg, "/lbfgs_iter", 200) as usize,
            ..Default::default()
        };
        let r = lbfgs(
            |y| {
                let mut gg = vec![0.0; y.len()];
                let v = j.value_grad(y, &mut gg);
                (-v, gg.into_iter().map(|a| -a).collect())
            },
            &best_x,
            &lc,
        );
        grads += r.evaluations as u64;
        if r.f.is_finite() && r.f < best {
            best = r.f;
            best_x = r.x.clone();
        }
        polish = json!({"lbfgs_iterations": r.iterations, "lbfgs_evaluations": r.evaluations, "converged": r.converged, "message": r.message});
    }
    let diag = json!({"method": "map", "optimizer": s(cfg, "/optimizer", "adam"), "learning_rate": lr, "epochs_run": history.len(),
        "start_epoch": start, "resumed_from_checkpoint": resumed, "history": history.iter().rev().take(10).rev().collect::<Vec<_>>(),
        "best_selection_value": finite(best), "final_neg_logp": finite(best), "gradient_evaluations": grads,
        "wall_s": t0.elapsed().as_secs_f64(), "lbfgs": polish});
    Ok(MapOut { x: best_x, diag })
}

// ------------------------------------------------------------------------------ Expert

fn flex_views<'a>(vs: &[&'a View]) -> Vec<&'a FlexTrajectory> {
    vs.iter()
        .filter_map(|v| if let View::Flex(f) = v { Some(f) } else { None })
        .collect()
}

impl Expert for FlexExpert {
    fn family(&self) -> Family {
        Family::Flex
    }
    fn model_name(&self) -> &str {
        &self.st.model_name
    }
    fn is_baseline(&self) -> bool {
        self.st.is_baseline
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }

    fn default_config(&self) -> ExpertResult<Value> {
        let mut cfg = json!({
            "time_origin": "phase_entry",
            "condition_keys": default_condition_keys(),
            "numerics": Numerics::default(),
            "fit": {"method": "map", "seed": 0, "posterior_draws": 400,
                    "map": {"epochs": 40, "steps_per_epoch": 50, "learning_rate": 0.01, "clip_norm": 10.0, "optimizer": "adam"},
                    "nuts": NutsConfig::default()},
            "eval": {"n_global": 16, "n_local": 128},
            "forecast": {"min_prefix_observations": 2},
            "support": {"k": 3, "min_groups": 2, "radius_percentile": 95.0},
            "priors": {},
        });
        let short = self.st.model_name.trim_start_matches("flex.");
        let p = configs_dir()
            .join("models")
            .join("flex")
            .join(format!("{short}.toml"));
        if let Ok(text) = std::fs::read_to_string(&p) {
            let t: toml::Value =
                toml::from_str(&text).with_context(|| format!("{}", p.display()))?;
            merge(&mut cfg, &serde_json::to_value(t)?);
            cfg["_config_path"] = json!(p.display().to_string());
        }
        Ok(cfg)
    }

    fn estimate_hours(
        &self,
        config: &Value,
        n_runs: usize,
        n_obs: usize,
        profile: &Profile,
    ) -> f64 {
        estimate(config, n_runs, n_obs, profile.threads())["estimated_hours"]
            .as_f64()
            .unwrap_or(0.0)
    }

    fn fit(
        &self,
        data: &FitData<'_>,
        config: &Value,
        ctx: &mut FitContext,
    ) -> ExpertResult<ModelRelease> {
        let ctx: &FitContext = ctx;
        ctx.pool
            .install(|| self.fit_inner(data, config, ctx, &ctx.pool))
    }

    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact> {
        let b = self.bind(&release.release)?;
        let (pd, sup, parts) = self.particles(&b, release, prefix, settings)?;
        if !sup.admitted {
            return Err(Refusal::unsupported(gate_code(sup.gate), sup.reasons.join("; ")).into());
        }
        let parts = parts.unwrap();
        let t_valid = b.m.num.t_valid_s;
        let levels: Vec<f64> = settings
            .get("quantile_levels")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(QUANTILES.to_vec());
        let times: Vec<f64> = match settings
            .get("times_s")
            .and_then(|v| serde_json::from_value::<Vec<f64>>(v.clone()).ok())
        {
            Some(mut t) => {
                t.sort_by(f64::total_cmp);
                t
            }
            None => {
                let h = f(settings, "/horizon_s", 10.0);
                let n = u(settings, "/n_times", 41) as usize;
                let hi = (pd.tau() + h).min(t_valid);
                (0..n)
                    .map(|i| pd.tau() + (hi - pd.tau()) * i as f64 / (n - 1).max(1) as f64)
                    .collect()
            }
        };
        let wd = spred::weight_diagnostics(&parts.w);
        let mut diag = json!({"particles": wd, "log_evidence": finite(parts.log_evidence), "laplace_ok_fraction": parts.laplace_ok,
            "n_global_draws": parts.g, "n_local_per_draw": parts.m,
            "global_uncertainty": if parts.g > 1 { "posterior draws" } else { "MAP point (no global uncertainty)" }});
        let min_ess = f(settings, "/min_ess", 30.0);
        let e = spred::ess(&parts.w);
        if e < min_ess {
            return Err(Refusal::insufficient(
                "numerical",
                format!("importance ESS {e:.1} < {min_ess}"),
            )
            .into());
        }
        let bad: Vec<f64> = times
            .iter()
            .copied()
            .filter(|t| *t < pd.tau() || *t > t_valid)
            .collect();
        if !bad.is_empty() {
            return Err(Refusal::unsupported(
                "horizon",
                format!("requested times {bad:?} outside [cutoff, T_valid={t_valid}]"),
            )
            .into());
        }
        let (res, w) = self.per_particle(&b, &parts, &pd, &times);
        let pe: Vec<ParticleEvents> = res
            .iter()
            .map(|p| ParticleEvents {
                before_b: p.before_b.clone(),
                s_rel: p.s_rel.clone(),
                s_b_rel: p.s_b_rel,
                f_rel: p.f_rel.clone(),
            })
            .collect();
        let (mut cats, mut table) = (vec![], vec![]);
        if self.st.hazards {
            table = events::support_partition(&w, &pe);
            cats = events::categories(&FLEX_CAUSES);
            diag["partition_max_error"] = json!(events::partition_error(&table));
        } else {
            let um: Vec<f64> = (0..times.len())
                .map(|j| {
                    res.iter()
                        .zip(&w)
                        .filter(|(p, _)| !p.before_b[j])
                        .map(|(_, w)| w)
                        .sum()
                })
                .collect();
            diag["unresolved_mass"] = json!(um);
            diag["event_model"] = json!("none (trajectory-only baseline)");
        }
        let (cw, mass) = events::conditioned_trajectory_weights(&w, &pe);
        let d: Vec<Vec<f64>> = res.iter().map(|p| p.d.clone()).collect();
        let lat_q = spred::weighted_quantiles(&d, &cw, &levels);
        let lat_mean: Vec<f64> = (0..times.len())
            .map(|j| {
                if mass[j] > 0.0 {
                    (0..res.len()).map(|p| cw[p][j] * d[p][j]).sum()
                } else {
                    0.0
                }
            })
            .collect();
        let mut rng = rand::rngs::StdRng::seed_from_u64(u(settings, "/seed", 0) + 1);
        let y_pred: Vec<Vec<f64>> = res
            .iter()
            .map(|p| {
                (0..times.len())
                    .map(|j| p.d[j] + p.e_loc[j] + p.e_scale2[j].sqrt() * sample_t(&mut rng, p.dof))
                    .collect()
            })
            .collect();
        let meas_q = spred::weighted_quantiles(&y_pred, &cw, &levels);
        let band = s(settings, "/band", "latent").to_string();
        let (q, other) = if band == "latent" {
            (lat_q, meas_q)
        } else {
            (meas_q, lat_q)
        };
        let bq =
            spred::weighted_quantiles_1d(&res.iter().map(|p| p.b).collect::<Vec<_>>(), &w, &levels);
        diag["band"] = json!(band);
        diag["other_band"] = json!(if band == "latent" {
            "measurement"
        } else {
            "latent"
        });
        diag["other_band_quantiles"] = json!(rows(&other));
        diag["conditioned_mass"] = json!(mass);
        diag["band_meaning"] =
            json!("D(t) conditioned on active phase and support at t (weights 1[t<B] S(t)/S(tau))");
        diag["B_quantiles_s"] = json!(bq.iter().map(|x| finite(*x)).collect::<Vec<_>>());
        diag["times_without_mass"] = json!(times
            .iter()
            .zip(&mass)
            .filter(|(_, m)| **m <= 0.0)
            .map(|(t, _)| *t)
            .collect::<Vec<_>>());
        Ok(artifact(
            release,
            prefix,
            settings,
            release.release.state,
            sup,
            times,
            levels,
            rows(&q),
            lat_mean,
            cats,
            rows(&table),
            diag,
        ))
    }

    fn log_likelihood(&self, release: &LoadedRelease, view: &View) -> ExpertResult<f64> {
        let View::Flex(v) = view else {
            bail!("FLEX experts read FLEX views only")
        };
        let b = self.bind(&release.release)?;
        let gd = inf::select_global_draws(&b.m.layout, &release.params, &release.posterior, 32, 0)?;
        Ok(self.marginal(&b, &gd, v, 128, 0))
    }

    fn objective(
        &self,
        release: &LoadedRelease,
        params: &TensorMap,
        views: &[&View],
    ) -> Option<ExpertResult<(f64, TensorMap)>> {
        Some(self.objective_inner(release, params, views))
    }

    fn activations(
        &self,
        release: &LoadedRelease,
        params: &TensorMap,
        view: &View,
    ) -> Option<ExpertResult<TensorMap>> {
        Some(self.activations_inner(release, params, view))
    }

    fn candidate_predictive(&self) -> Option<&dyn CandidatePredictive> {
        Some(self)
    }
}

fn gate_code(g: SupportGate) -> &'static str {
    match g {
        SupportGate::Semantic => "semantic",
        SupportGate::Measurement => "measurement",
        SupportGate::Condition => "condition",
        SupportGate::Numerical => "numerical",
        SupportGate::Horizon => "horizon",
        SupportGate::Passed => "passed",
    }
}

fn rows(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
    a.iter()
        .map(|r| {
            r.iter()
                .map(|x| if x.is_finite() { *x } else { 0.0 })
                .collect()
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn artifact(
    rel: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
    status: QualificationState,
    support: Support,
    times: Vec<f64>,
    levels: Vec<f64>,
    quantiles: Vec<Vec<f64>>,
    mean: Vec<f64>,
    cats: Vec<String>,
    cdf: Vec<Vec<f64>>,
    diag: Value,
) -> AnalysisArtifact {
    let mut a = AnalysisArtifact {
        id: String::new(),
        kind: ArtifactKind::PrefixForecast,
        model_release_id: rel.release.id.clone(),
        run_id: Some(prefix.run_id.clone()),
        time_origin: Some(prefix.time_origin.clone()),
        prefix_hash: prefix.input_hash.clone(),
        settings_hash: content_hash(&jsonable(settings), &[]),
        status,
        quantity: Some(Quantity::LiquidDiameter),
        unit: Some("m".into()),
        times_s: times,
        quantile_levels: if quantiles.is_empty() { vec![] } else { levels },
        quantiles,
        mean,
        space_s_m: vec![],
        field_quantiles: vec![],
        field_mean: vec![],
        event_categories: cats,
        event_cdf: cdf,
        support,
        diagnostics: obj(jsonable(&diag)),
        evidence_class: EvidenceClass::ModelOutput,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
    };
    a.id = content_hash(&a, &["id", "created_at"]);
    a
}

pub struct Particles {
    pub globals: Vec<Vec<f64>>,
    pub w: Vec<f64>,
    pub g_idx: Vec<usize>,
    pub v: Vec<Vec<f64>>,
    pub log_evidence: f64,
    pub laplace_ok: f64,
    pub g: usize,
    pub m: usize,
}

impl FlexExpert {
    fn fit_inner(
        &self,
        data: &FitData<'_>,
        config: &Value,
        ctx: &FitContext,
        pool: &rayon::ThreadPool,
    ) -> ExpertResult<ModelRelease> {
        let mut cfg = config.clone();
        let num = numerics_from(cfg.get("numerics"));
        let keys = keys_from(&cfg);
        let pr = merged_priors(cfg.get("priors"));
        let train = flex_views(&data.partition("train"));
        let val = flex_views(&data.partition("validation"));
        if train.is_empty() {
            bail!("no train runs in the split");
        }
        // time origin of the views (one per cohort) wins over the config default
        let origins: std::collections::BTreeSet<&str> =
            train.iter().map(|v| v.time_origin.as_str()).collect();
        if origins.len() == 1 {
            cfg["time_origin"] = json!(origins.iter().next().unwrap());
        }
        let mut packed = pack_views(&train, &keys, num.t_valid_s);
        admin_censor(&mut packed, num.t_valid_s);
        let bad: Vec<&String> = packed
            .runs
            .iter()
            .zip(&packed.run_ids)
            .filter(|(r, _)| r.ck.iter().any(|x| !x.is_finite()))
            .map(|(_, id)| id)
            .collect();
        if !bad.is_empty() {
            bail!("train runs with missing/invalid prospective conditions (exclude in the cohort): {bad:?}");
        }
        let fs = fit_feature_spec(
            &packed.runs.iter().map(|r| r.ck).collect::<Vec<_>>(),
            &packed.logd0_proxy(),
            &keys,
            1e-9,
        );
        let m = self.model_for(fs.clone(), num.clone(), &pr);
        let j = Joint {
            m: &m,
            runs: &packed.runs,
            pr: &pr,
        };
        let seed = u(&cfg, "/fit/seed", ctx.seed);
        let mut x = inf::init_params(&m, &packed.runs, &pr, seed);
        let (mut parent_id, mut lineage, mut note) = (None, Lineage::InitialFit, None);
        if let Some(rel) = &ctx.resume {
            if rel.release.model_name != self.st.model_name
                || rel.release.feature_order != fs.feature_order()
            {
                bail!("resume release has a different model or feature order");
            }
            for (s, &o) in m.layout.specs.iter().zip(&m.layout.offsets) {
                if let Some(t) = rel.params.get(&s.name) {
                    if t.shape == s.shape {
                        x[o..o + s.size()].copy_from_slice(&t.data);
                    }
                }
            }
            parent_id = Some(rel.release.id.clone());
            lineage = Lineage::FineTune;
            note = Some("warm start from parent release (MAP)".to_string());
        }
        let repaired = inf::repair_support(&j, &mut x, 20);
        let method = s(&cfg, "/fit/method", "map").to_string();
        let config_hash = content_hash(
            &json!({"cfg": jsonable(&cfg), "runs": packed.run_ids, "model": self.st.model_name}),
            &[],
        );
        let ckpt = ctx.checkpoint_dir.join("map.json");
        let map = run_map(
            &j,
            x,
            cfg.pointer("/fit/map").unwrap_or(&Value::Null),
            Some(&ckpt),
            &config_hash,
        )?;
        let point = map.x.clone();
        let mut posterior = TensorMap::new();
        let mut post_names = vec![];
        let mut state = QualificationState::FitOnly;
        let mut inf_diag = map.diag.clone();
        inf_diag["support_repairs"] = json!(repaired);
        let local_layout = Layout::new(local_specs(&self.st, packed.n(), &pr));
        if method == "nuts" {
            let ncfg: NutsConfig = cfg
                .pointer("/fit/nuts")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            // locals: "centered" (b = L_b eps sampled directly; see inference::Centered) or "noncentered"
            let locals = s(&cfg, "/fit/nuts/locals", "centered").to_string();
            let cen = inf::Centered { j: &j };
            let centered = locals == "centered" && m.n_b() > 0;
            let dens: &dyn ignis_science::nuts::LogDensity = if centered { &cen } else { &j };
            let start = if centered {
                cen.from_eps(&point)
            } else {
                point.clone()
            };
            let t_h = Instant::now();
            let pre = if ncfg.precondition == "laplace" {
                let grad = |y: &[f64]| {
                    let mut g = vec![0.0; y.len()];
                    dens.logp_grad(y, &mut g);
                    g
                };
                let h = neg_hessian_fd(
                    &start,
                    &grad,
                    &j.hessian_probes(),
                    &|i, p| j.attribute(i, p),
                    pool,
                );
                Precond::from_neg_hessian(&start, &h)
            } else {
                Precond::identity(&start)
            };
            let hess_s = t_h.elapsed().as_secs_f64();
            let mut r = run_nuts(dens, &start, &pre, &ncfg, seed, pool)?;
            if centered {
                for c in r.draws.iter_mut() {
                    for d in c.iter_mut() {
                        *d = cen.to_eps(d);
                    }
                }
            }
            let n_keep = u(&cfg, "/fit/posterior_draws", 400) as usize;
            let flat: Vec<&Vec<f64>> = r.draws.iter().flatten().collect();
            let dtot = flat.len();
            let keep: Vec<Vec<f64>> = (0..n_keep.min(dtot))
                .map(|i| {
                    let k = if n_keep.min(dtot) > 1 {
                        (i as f64 * (dtot - 1) as f64 / (n_keep.min(dtot) - 1) as f64).round()
                            as usize
                    } else {
                        0
                    };
                    flat[k].clone()
                })
                .collect();
            let ng = m.n_g();
            stacked_split(&m.layout, &local_layout, ng, &keep, &mut posterior);
            post_names = posterior.keys().cloned().collect();
            // diagnostics per block
            let mut blocks: Vec<(String, Vec<Draws>)> = vec![];
            for (sp, &o) in m
                .layout
                .specs
                .iter()
                .zip(&m.layout.offsets)
                .chain(local_layout.specs.iter().zip(local_layout.offsets.iter()))
            {
                let base = if sp.name.starts_with("local.") { ng } else { 0 };
                let scal: Vec<Draws> = (0..sp.size())
                    .map(|k| {
                        r.draws
                            .iter()
                            .map(|c| c.iter().map(|d| d[base + o + k]).collect())
                            .collect()
                    })
                    .collect();
                blocks.push((sp.name.clone(), scal));
            }
            let mut nd = summarize(&blocks, &r.stats, ncfg.rhat_max, ncfg.min_ess);
            nd["wall_s"] = json!(r.wall_s);
            nd["hessian_s"] = json!(hess_s);
            nd["gradient_evaluations"] = json!(r.gradient_evaluations);
            nd["chains"] = json!(ncfg.chains);
            nd["warmup"] = json!(ncfg.warmup);
            nd["samples"] = json!(ncfg.samples);
            nd["target_accept"] = json!(ncfg.target_accept);
            nd["max_tree_depth"] = json!(ncfg.max_tree_depth);
            nd["mass_matrix"] = json!(ncfg.mass_matrix);
            nd["precondition"] = json!(ncfg.precondition);
            nd["precondition_floored_directions"] = json!(pre.ridge);
            nd["locals_parameterization"] = json!(if centered {
                "centered (b = L_b eps sampled); draws stored as eps"
            } else {
                "noncentered"
            });
            nd["sampler"] = json!("nuts-rs 0.19 (parallel chains, bounded pool)");
            nd["precision"] = json!("float64");
            nd["seed"] = json!(seed);
            nd["method"] = json!("nuts");
            nd["map_init"] = json!({"epochs_run": map.diag["epochs_run"], "final_neg_logp": map.diag["final_neg_logp"]});
            if nd["pass"] != Value::Bool(true) {
                state = QualificationState::InsufficientInference;
            }
            inf_diag = nd;
        } else if method != "map" {
            bail!("unknown fit method {method:?}");
        }
        // validation: held-out log predictive density (selection only; test is never touched here)
        let b = Bound {
            m: m.clone(),
            pr: pr.clone(),
            keys: keys.clone(),
            local_ids: packed.run_ids.clone(),
        };
        let mut ptens = TensorMap::new();
        inf::tensors_from_flat(&m.layout, &point[..m.n_g()], &mut ptens);
        inf::tensors_from_flat(&local_layout, &point[m.n_g()..], &mut ptens);
        let evaluation = self.validate(&b, &ptens, &posterior, &val, &cfg)?;
        // condition support policy (train only)
        let proxy = packed.logd0_proxy();
        let cidx = fs.c_idx();
        let xs: Vec<Vec<f64>> = packed
            .runs
            .iter()
            .zip(&proxy)
            .map(|(r, p)| {
                let c = [r.ck[0], r.ck[1], r.ck[2], r.ck[3], *p];
                cidx.iter().map(|&i| c[i]).collect()
            })
            .collect();
        let sup = fit_condition_support(
            &xs,
            &packed.group_ids,
            u(&cfg, "/support/k", 3) as usize,
            u(&cfg, "/support/min_groups", 2) as usize,
            f(&cfg, "/support/radius_percentile", 95.0),
        );
        let synthetic = data.synthetic || train.iter().any(|v| v.evidence_class == "synthetic");
        let trained_on = if synthetic {
            TrainedOn::Synthetic
        } else {
            TrainedOn::Nasa
        };
        let specs_g = &m.layout.specs;
        let inventory: Map<String, Value> = specs_g
            .iter()
            .map(|s| (s.name.clone(), json!(s.shape)))
            .collect();
        let mut blocks = Map::new();
        for sp in specs_g.iter().chain(local_layout.specs.iter()) {
            blocks.insert(sp.name.clone(), sp.describe());
        }
        let units: BTreeMap<String, String> =
            [("t", "s"), ("d", "m"), ("q", "m^2"), ("K_ref", "m^2/s")]
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .chain([
                    (keys["pressure"].clone(), "Pa".to_string()),
                    (keys["o2"].clone(), "1".into()),
                    (keys["co2"].clone(), "1".into()),
                    (keys["he"].clone(), "1".into()),
                ])
                .collect();
        let mut numerics = obj(serde_json::to_value(&num)?);
        numerics.insert("x64".into(), json!(true));
        numerics.insert(
            "solver".into(),
            json!("Gauss-Legendre panel table + bracketed Newton, implicit gradient"),
        );
        numerics.insert(
            "autodiff".into(),
            json!("reverse-mode tape (ignis-science::ad), fused analytic integrand nodes"),
        );
        numerics.insert("inference".into(), jsonable(&inf_diag));
        let release = ModelRelease {
            id: String::new(),
            family: Family::Flex,
            model_name: self.st.model_name.clone(),
            is_baseline: self.st.is_baseline,
            architecture: obj(json!({
                "structure": structure_json(&self.st), "feature_spec": fs, "parameter_inventory": inventory,
                "n_population_parameters": m.n_g(), "n_local_per_run": self.st.n_b() + 1, "local_run_ids": packed.run_ids,
                "posterior_kind": if posterior.is_empty() { "map_point" } else { "nuts_draws" },
                "residual": if self.st.residual { json!(format!("{}->{}->1 tanh, R=0.5 w [h(c,w)-h(0,w)]", cidx.len() + 1, self.st.hidden)) } else { Value::Null },
                "params_point": "map (unconstrained coordinates)", "implementation": "rust ignis-flex",
            })),
            config: obj(jsonable(&cfg)),
            feature_order: fs.feature_order(),
            units,
            normalization: normalization(),
            priors: obj(json!({"blocks": blocks, "hyper": pr,
                "convention": "priors on natural coordinates; Jacobian added for transformed blocks; log-scale priors (log D0, log sigma, log ell) are on the sampled coordinate (no Jacobian)"})),
            observation_model: obj(json!({
                "operator": "y_j = sqrt(q(t_j)) + e_j (s_g=1, delta_g=0, Delta t_g=0: published calibration applied upstream)",
                "likelihood": "multivariate Student-t process, nu fixed", "nu": num.nu,
                "kernel": "sigma_c^2 exp(-|dt|/ell) + sigma_n^2 1[j=k]", "evaluation": "OU/Kalman O(n), equal to dense",
                "masking": "t<0, t>T_valid, invalid flags, exact duplicate timestamps masked (counted)",
                "events": if self.st.hazards { json!("competing hazards hot_phase_end (w-dependent) and disruption (constant)") } else { Value::Null },
                "interval_events": "1[B>L] int_L^min(R,B) S lambda_k; assumes review established event before support exit",
                "right_censoring": "S(min(C, B)); events after T_valid are administratively right-censored at T_valid",
                "support": "B = min(t(r_min), T_valid), r_min = (d_min/D0)^2; draws with admitted observations beyond t(r_min) have zero likelihood",
                "packing_notes": packed.notes(),
            })),
            numerics,
            tensors: vec![],
            posterior: vec![],
            posterior_parameters: post_names,
            cohort_id: data.cohort_id.to_string(),
            split_id: data.split.id.clone(),
            source_manifest_hash: data.source_manifest_hash.to_string(),
            code_version: String::new(),
            applicability: obj(
                json!({"fuel": "methanol", "population": "free-floating methanol droplet, established hot phase",
                "time_origin": cfg["time_origin"], "condition_support": sup, "promotable": !synthetic,
                "trained_on": trained_on, "min_prefix_observations": u(&cfg, "/forecast/min_prefix_observations", 2)}),
            ),
            evaluation: obj(jsonable(&evaluation)),
            state,
            parent_id,
            lineage,
            lineage_note: note,
            posterior_valid_for_parameters: true,
            trained_on,
            resource_record: ctx.job_id.clone(),
            created_at: String::new(),
            // Conventionally fitted (legacy expert path): no WSP construction manifest.
            construction: None,
        };
        save_release(&ctx.registry_root, release, &ptens, &posterior, &[])
    }

    fn validate(
        &self,
        b: &Bound,
        params: &TensorMap,
        posterior: &TensorMap,
        val: &[&FlexTrajectory],
        cfg: &Value,
    ) -> anyhow::Result<Value> {
        if val.is_empty() {
            return Ok(json!({"validation": "no validation runs"}));
        }
        let gd = inf::select_global_draws(
            &b.m.layout,
            params,
            posterior,
            u(cfg, "/eval/n_global", 16) as usize,
            0,
        )?;
        let nl = u(cfg, "/eval/n_local", 128) as usize;
        let lls: Vec<f64> = val
            .iter()
            .map(|v| self.marginal(b, &gd, v, nl, 0))
            .collect();
        let fin: Vec<f64> = lls.iter().copied().filter(|x| x.is_finite()).collect();
        let n_obs: usize = val
            .iter()
            .map(|v| v.valid.iter().filter(|x| **x).count())
            .sum();
        Ok(json!({"validation_runs": val.len(),
            "validation_log_predictive_density_per_run": if fin.is_empty() { Value::Null } else { json!(fin.iter().sum::<f64>() / fin.len() as f64) },
            "validation_log_predictive_density_total": if fin.is_empty() { Value::Null } else { json!(fin.iter().sum::<f64>()) },
            "validation_runs_zero_density": lls.len() - fin.len(), "validation_observations": n_obs,
            "estimator": "importance sampling over locals (defensive Laplace mixture), global draws equal weight"}))
    }

    /// log p(view | release) by importance sampling over locals.
    pub fn marginal(
        &self,
        b: &Bound,
        gd: &[Vec<f64>],
        view: &FlexTrajectory,
        n_local: usize,
        seed: u64,
    ) -> f64 {
        let mut pk = pack_views(&[view], &b.keys, b.m.num.t_valid_s);
        admin_censor(&mut pk, b.m.num.t_valid_s);
        let run = &pk.runs[0];
        if run.ck.iter().any(|x| !x.is_finite()) {
            return f64::NEG_INFINITY;
        }
        let out = inf::importance_locals(&b.m, &b.pr, gd, run, n_local, seed, 0.2, 1.5, 20);
        spred::normalize_log_weights(&out.logw).1
    }

    pub fn particles(
        &self,
        b: &Bound,
        rel: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> anyhow::Result<(PrefixData, Support, Option<Particles>)> {
        let pd = prefix_arrays(prefix, &b.keys, b.m.num.t_valid_s);
        let c = [
            pd.run.ck[0],
            pd.run.ck[1],
            pd.run.ck[2],
            pd.run.ck[3],
            if pd.y_first.is_finite() {
                (pd.y_first / D_REF_M).ln()
            } else {
                f64::NAN
            },
        ];
        let x_cond: Vec<f64> = b.m.c_idx.iter().map(|&i| c[i]).collect();
        let mut merged = Value::Object(rel.release.applicability.clone());
        merge(&mut merged, settings);
        let sup = gate_prefix(&rel.release, prefix, &pd, &x_cond, &merged);
        if !sup.admitted {
            return Ok((pd, sup, None));
        }
        let seed = u(settings, "/seed", 0);
        let gd = inf::select_global_draws(
            &b.m.layout,
            &rel.params,
            &rel.posterior,
            u(settings, "/n_global", 64) as usize,
            seed,
        )?;
        let g = gd.len();
        let m = u(settings, "/n_local", (2048 / g).max(16) as u64) as usize;
        let out = inf::importance_locals(
            &b.m,
            &b.pr,
            &gd,
            &pd.run,
            m,
            seed,
            f(settings, "/alpha", 0.2),
            f(settings, "/inflate", 1.5),
            20,
        );
        let (w, le) = spred::normalize_log_weights(&out.logw);
        let laplace_ok = out.ok.iter().filter(|x| **x).count() as f64 / out.ok.len().max(1) as f64;
        Ok((
            pd,
            sup,
            Some(Particles {
                g_idx: (0..g * m).map(|i| i / m).collect(),
                globals: gd,
                w,
                v: out.v,
                log_evidence: le,
                laplace_ok,
                g,
                m,
            }),
        ))
    }

    pub fn per_particle(
        &self,
        b: &Bound,
        parts: &Particles,
        pd: &PrefixData,
        times: &[f64],
    ) -> (Vec<Particle>, Vec<f64>) {
        let nb = b.m.n_b();
        let idx: Vec<usize> = (0..parts.w.len()).filter(|&i| parts.w[i] > 0.0).collect();
        let res: Vec<Particle> = idx
            .par_iter()
            .map(|&i| {
                let v = &parts.v[i];
                forecast_particle(
                    &b.m,
                    &parts.globals[parts.g_idx[i]],
                    &pd.run,
                    &v[..nb],
                    v[nb],
                    times,
                )
            })
            .collect();
        (res, idx.iter().map(|&i| parts.w[i]).collect())
    }

    fn objective_inner(
        &self,
        rel: &LoadedRelease,
        params: &TensorMap,
        views: &[&View],
    ) -> anyhow::Result<(f64, TensorMap)> {
        let b = self.bind(&rel.release)?;
        let vs = flex_views(views);
        let mut pk = pack_views(&vs, &b.keys, b.m.num.t_valid_s);
        admin_censor(&mut pk, b.m.num.t_valid_s);
        let xg = inf::flat_from_tensors(&b.m.layout, params)?;
        let ng = b.m.n_g();
        let nb = b.m.n_b();
        let mut grads = TensorMap::new();
        let have_locals = params.contains_key("local.log_d0")
            && pk.run_ids.iter().all(|r| b.local_ids.contains(r));
        if have_locals {
            // joint density, locals taken from params in the bound local order
            let order: Vec<usize> = pk
                .run_ids
                .iter()
                .map(|r| b.local_ids.iter().position(|x| x == r).unwrap())
                .collect();
            let ld = &params["local.log_d0"].data;
            let eps = params
                .get("local.eps_b")
                .map(|t| t.data.clone())
                .unwrap_or_default();
            let mut x = xg.clone();
            for &o in &order {
                x.extend_from_slice(&eps[o * nb..(o + 1) * nb]);
            }
            for &o in &order {
                x.push(ld[o]);
            }
            let j = Joint {
                m: &b.m,
                runs: &pk.runs,
                pr: &b.pr,
            };
            let mut g = vec![0.0; x.len()];
            let v = j.value_grad(&x, &mut g);
            let neg: Vec<f64> = g.iter().map(|a| -a).collect();
            inf::tensors_from_flat(&b.m.layout, &neg[..ng], &mut grads);
            let nl = b.local_ids.len();
            let (mut ge, mut gl) = (vec![0.0; nl * nb], vec![0.0; nl]);
            for (i, &o) in order.iter().enumerate() {
                for k in 0..nb {
                    ge[o * nb + k] = neg[j.local_index(i, k)];
                }
                gl[o] = neg[j.local_index(i, nb)];
            }
            if nb > 0 {
                grads.insert("local.eps_b".into(), Tensor::new(vec![nl, nb], ge));
            }
            grads.insert("local.log_d0".into(), Tensor::vector(gl));
            return Ok((-v, grads));
        }
        // profile: locals at their conditional mode (held fixed, envelope theorem) + global prior
        let modes: Vec<Vec<f64>> = pk
            .runs
            .par_iter()
            .map(|r| {
                let (k, q0) = inf::prefix_d2_estimate(r);
                let v0 = inf::local_init(&b.m, &xg, &r.ck, k, q0.unwrap_or(9e-6));
                inf::newton_mode(&b.m, &b.pr, &xg, r, &v0, 20).0
            })
            .collect();
        let (lp, gp) = ad::value_and_grad(&xg, |p: &[Var]| b.m.layout.log_prior(p));
        let per: Vec<(f64, Vec<f64>)> = pk
            .runs
            .par_iter()
            .zip(&modes)
            .map(|(r, mode)| {
                let mv: Vec<Var> = mode.iter().map(|&x| Var::cst(x)).collect();
                ad::value_and_grad(&xg, |p: &[Var]| inf::local_target(&b.m, &b.pr, p, &mv, r))
            })
            .collect();
        let mut total = -lp;
        let mut g: Vec<f64> = gp.iter().map(|a| -a).collect();
        for (v, gr) in per {
            total -= v;
            for k in 0..ng {
                g[k] -= gr[k];
            }
        }
        inf::tensors_from_flat(&b.m.layout, &g, &mut grads);
        Ok((total, grads))
    }

    fn activations_inner(
        &self,
        rel: &LoadedRelease,
        params: &TensorMap,
        view: &View,
    ) -> anyhow::Result<TensorMap> {
        let View::Flex(v) = view else {
            bail!("FLEX view expected")
        };
        let b = self.bind(&rel.release)?;
        let pk = pack_views(&[v], &b.keys, b.m.num.t_valid_s);
        let run = &pk.runs[0];
        let p = inf::flat_from_tensors(&b.m.layout, params)?;
        let nb = b.m.n_b();
        let (locals, src) = match b.local_ids.iter().position(|r| *r == v.run_id) {
            Some(i) if params.contains_key("local.log_d0") => {
                let mut l: Vec<f64> = params
                    .get("local.eps_b")
                    .map(|t| t.data[i * nb..(i + 1) * nb].to_vec())
                    .unwrap_or_default();
                l.push(params["local.log_d0"].data[i]);
                (l, 0.0)
            }
            _ => {
                let (k, q0) = inf::prefix_d2_estimate(run);
                let v0 = inf::local_init(&b.m, &p, &run.ck, k, q0.unwrap_or(9e-6));
                (inf::newton_mode(&b.m, &b.pr, &p, run, &v0, 20).0, 1.0)
            }
        };
        let rs = run_state(&b.m, &p, &run.ck, &locals[..nb], locals[nb]);
        let ts: Vec<f64> = (0..run.t.len())
            .filter(|&i| run.mask[i])
            .map(|i| run.t[i])
            .collect();
        let ys: Vec<f64> = (0..run.t.len())
            .filter(|&i| run.mask[i])
            .map(|i| run.y[i])
            .collect();
        let r: Vec<f64> = ts
            .iter()
            .map(|&t| rs.r_of_t(&b.m.num, t.clamp(0.0, rs.t_rmin)))
            .collect();
        let w: Vec<f64> = r.iter().map(|x| 1.0 - x).collect();
        let shape: Vec<f64> = w.iter().map(|&x| rs.integ.log_shape(x)).collect();
        let mut out = TensorMap::new();
        let vt = |x: Vec<f64>| Tensor::vector(x);
        out.insert("t_s".into(), vt(ts.clone()));
        out.insert("y_m".into(), vt(ys));
        out.insert("r".into(), vt(r.clone()));
        out.insert("w".into(), vt(w.clone()));
        out.insert(
            "d_pred_m".into(),
            vt(r.iter().map(|x| rs.d0 * x.sqrt()).collect()),
        );
        out.insert(
            "log_rate_ratio".into(),
            vt(shape.iter().map(|s| s + rs.b0).collect()),
        );
        out.insert("log_shape".into(), vt(shape));
        out.insert("t_rmin_s".into(), Tensor::scalar(rs.t_rmin));
        out.insert("B_s".into(), Tensor::scalar(rs.b));
        out.insert("b0".into(), Tensor::scalar(rs.b0));
        out.insert("b1".into(), Tensor::scalar(rs.b1));
        out.insert("c".into(), vt(rs.c.clone()));
        out.insert("locals_from_conditional_mode".into(), Tensor::scalar(src));
        if self.st.hazards {
            out.insert(
                "log_lambda_h".into(),
                vt(w.iter().map(|&x| rs.integ.log_lam_h(x)).collect()),
            );
            out.insert(
                "log_survival".into(),
                vt(r.iter().map(|&x| rs.log_survival_at_u(x).0).collect()),
            );
        }
        Ok(out)
    }
}

fn stacked_split(lg: &Layout, ll: &Layout, ng: usize, draws: &[Vec<f64>], out: &mut TensorMap) {
    let g: Vec<Vec<f64>> = draws.iter().map(|d| d[..ng].to_vec()).collect();
    let l: Vec<Vec<f64>> = draws.iter().map(|d| d[ng..].to_vec()).collect();
    inf::stacked_tensors(lg, &g, out);
    inf::stacked_tensors(ll, &l, out);
}

// ------------------------------------------------------------------------------ VOI

impl CandidatePredictive for FlexExpert {
    fn model_name(&self) -> &str {
        &self.st.model_name
    }
    fn has_event_model(&self) -> bool {
        self.st.hazards
    }

    /// Particles + candidate measurement predictive for one-step preposterior analysis. Uses only the
    /// PrefixBundle and candidate *times*; candidate values are never an input.
    fn voi_particles(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        ct: &[f64],
        settings: &Value,
    ) -> anyhow::Result<VoiParticles> {
        let b = self.bind(&release.release)?;
        let (pd, sup, parts) = self.particles(&b, release, prefix, settings)?;
        let Some(parts) = parts else {
            return Ok(VoiParticles {
                support: sup,
                w: vec![],
                pred_loc: vec![],
                pred_scale2: vec![],
                dof: vec![],
                horizon_s: 0.0,
                particles: Value::Null,
                z_event: None,
                event_categories: vec![],
                z_diameter: vec![],
                z_diameter_weight: vec![],
            });
        };
        let h = (f(settings, "/target_horizon_s", 2.0) + pd.tau()).min(b.m.num.t_valid_s);
        let (res, w) = self.per_particle(&b, &parts, &pd, &[pd.tau(), h]);
        let nb = b.m.n_b();
        let idx: Vec<usize> = (0..parts.w.len()).filter(|&i| parts.w[i] > 0.0).collect();
        let run = &pd.run;
        let ta: Vec<f64> = run.t.clone();
        let preds: Vec<(Vec<f64>, Vec<f64>, f64)> = idx
            .par_iter()
            .map(|&i| {
                let p = &parts.globals[parts.g_idx[i]];
                let v = &parts.v[i];
                let nat = natural_globals(&b.m, p);
                let rs = run_state(&b.m, p, &run.ck, &v[..nb], v[nb]);
                let mut all = ta.clone();
                all.extend_from_slice(ct);
                let d_all = trajectory_d(&rs, &b.m.num, &all);
                let (da, dc) = d_all.split_at(ta.len());
                let ea: Vec<f64> = (0..ta.len())
                    .map(|k| if run.mask[k] { run.y[k] - da[k] } else { 0.0 })
                    .collect();
                let saa =
                    dense_scale_matrix(&ta, nat.sigma_c, nat.sigma_n, nat.ell, Some(&run.mask));
                let sba = exp_kernel(ct, &ta, nat.sigma_c, nat.ell);
                let mut sbb = exp_kernel(ct, ct, nat.sigma_c, nat.ell);
                for k in 0..ct.len() {
                    sbb.set(k, k, sbb.at(k, k) + nat.sigma_n * nat.sigma_n);
                }
                let c = conditional_t(&ea, &saa, &sba, &sbb, b.m.num.nu, Some(&run.mask));
                (
                    (0..ct.len()).map(|k| dc[k] + c.loc[k]).collect(),
                    (0..ct.len()).map(|k| c.scale.at(k, k)).collect(),
                    c.dof,
                )
            })
            .collect();
        let pe: Vec<ParticleEvents> = res
            .iter()
            .map(|p| ParticleEvents {
                before_b: p.before_b.clone(),
                s_rel: p.s_rel.clone(),
                s_b_rel: p.s_b_rel,
                f_rel: p.f_rel.clone(),
            })
            .collect();
        let z_event = self.st.hazards.then(|| {
            pe.iter()
                .map(|p| {
                    events::per_particle_partition(p)
                        .iter()
                        .map(|row| row[1])
                        .collect()
                })
                .collect()
        });
        Ok(VoiParticles {
            support: sup,
            particles: spred::weight_diagnostics(&w),
            horizon_s: h,
            pred_loc: preds.iter().map(|p| p.0.clone()).collect(),
            pred_scale2: preds.iter().map(|p| p.1.clone()).collect(),
            dof: preds.iter().map(|p| p.2).collect(),
            w,
            z_event,
            event_categories: if self.st.hazards {
                events::categories(&FLEX_CAUSES)
            } else {
                vec![]
            },
            z_diameter: res.iter().map(|p| p.d[1]).collect(),
            z_diameter_weight: res
                .iter()
                .map(|p| if p.before_b[1] { p.s_rel[1] } else { 0.0 })
                .collect(),
        })
    }
}

pub fn registry_root_of(settings: &Value) -> Option<PathBuf> {
    settings
        .get("registry_root")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
}

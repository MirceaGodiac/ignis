//! Explicitly synthetic FLEX dataset drawn from the model's own generative process.
//!
//! Used for development (`ignis fit run ... --fixture synthetic`) and checks. Every view is
//! `evidence_class = synthetic`; releases fitted on it carry `trained_on = synthetic`, are never
//! promotable, and the support gate refuses them on non-synthetic prefixes.

use std::collections::BTreeMap;

use rand::{RngExt, SeedableRng};
use rand_distr::{Distribution, Exp1, StandardNormal};

use ignis_core::contracts::{
    Censoring, Cohort, Derivation, EndpointRecord, EvidenceClass, ExperimentRecord, Family,
    Measured, Method, Observation, Quantity, RawValue, Resolution, SourceLocator, Split,
};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{Tensor, TensorMap};
use ignis_core::views::{FlexTrajectory, RunArchive, View, ViewSource};
use ignis_science::studentt::sample_student_t_process;
use ignis_science::transforms::Layout;

use crate::dynamics::{run_state, Model, Numerics};
use crate::structure::{
    default_condition_keys, global_specs, known_features, merged_priors, FeatureSpec, Structure,
};

pub const TIME_ORIGIN: &str = "synthetic_phase_entry";
pub const MAX_OBS: usize = 512;

/// Generating parameters (flex.v3 layout with every feature active).
pub fn truth_params(seed: u64) -> TensorMap {
    let st = Structure::get("flex.v3").unwrap();
    let fs = FeatureSpec::full(default_condition_keys());
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut p = TensorMap::new();
    let v = |x: &[f64]| Tensor::vector(x.to_vec());
    p.insert(
        "beta".into(),
        v(&[0.5f64.ln(), 0.15, -0.10, 0.05, -0.08, 0.10, 0.02, 0.0, 0.0]),
    );
    p.insert("theta".into(), v(&[0.10, -0.05, 0.02]));
    p.insert("b.log_sigma".into(), v(&[0.12f64.ln(), 0.08f64.ln()]));
    p.insert("b.atanh_rho".into(), v(&[0.3]));
    p.insert("hazard.gamma0".into(), Tensor::scalar(0.02f64.ln()));
    p.insert("hazard.gamma".into(), v(&[0.2, -0.1, 0.0, 0.1, 0.0]));
    p.insert("hazard.a".into(), v(&[2.0, 2.0]));
    p.insert("hazard.link".into(), v(&[0.3, 0.0]));
    p.insert("hazard.zeta".into(), Tensor::scalar(0.004f64.ln()));
    p.insert("obs.log_sigma_c".into(), Tensor::scalar(2.0e-5f64.ln()));
    p.insert("obs.log_sigma_n".into(), Tensor::scalar(1.5e-5f64.ln()));
    p.insert("obs.log_ell".into(), Tensor::scalar(0.3f64.ln()));
    for s in global_specs(&st, &fs, &merged_priors(None)) {
        if s.name.starts_with("residual.") {
            let data = (0..s.size())
                .map(|i| {
                    let z: f64 = StandardNormal.sample(&mut rng);
                    z * s.prior.scale(i)
                })
                .collect();
            p.insert(s.name.clone(), Tensor::new(s.shape.clone(), data));
        }
    }
    p
}

/// Per-run truth (evaluator-only continuation facts).
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunTruth {
    pub eps_b: [f64; 2],
    pub log_d0: f64,
    pub event_time: Option<f64>,
    pub cause: Option<String>,
    pub b: f64,
}

pub struct SyntheticFlex {
    pub views: Vec<FlexTrajectory>,
    pub split: Split,
    pub cohort: Cohort,
    pub truth: BTreeMap<String, RunTruth>,
}

fn interp(x: f64, xp: &[f64], fp: &[f64]) -> f64 {
    ignis_science::predictive::interp(x, xp, fp)
}

pub fn make_dataset(
    n_runs: usize,
    seed: u64,
    runs_per_group: usize,
    cadence_s: f64,
) -> anyhow::Result<SyntheticFlex> {
    let st = Structure::get("flex.v3").unwrap();
    let keys = default_condition_keys();
    let fs = FeatureSpec::full(keys.clone());
    let num = Numerics::default();
    let layout = Layout::new(global_specs(&st, &fs, &merged_priors(None)));
    let m = Model::new(st, fs, num.clone(), layout);
    let truth_t = truth_params(seed + 7);
    let p = crate::inference::flat_from_tensors(&m.layout, &truth_t)?;
    let nat = crate::dynamics::natural_globals(&m, &p);
    let (sc, sn, ell) = (nat.sigma_c, nat.sigma_n, nat.ell);
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let grid: Vec<f64> = (0..1500).map(|i| i as f64 / 1499.0).collect();
    let mut views = Vec::new();
    let mut truth = BTreeMap::new();
    for i in 0..n_runs {
        let mut conds = BTreeMap::new();
        conds.insert(keys["o2"].clone(), rng.random_range(0.17..0.27));
        conds.insert(keys["co2"].clone(), rng.random_range(0.0..0.15));
        conds.insert(keys["he"].clone(), rng.random_range(0.0..0.25));
        conds.insert(keys["pressure"].clone(), rng.random_range(0.6e5..1.0e5));
        let ck = known_features(&conds, &keys);
        let eps = [
            StandardNormal.sample(&mut rng),
            StandardNormal.sample(&mut rng),
        ];
        let log_d0 = rng.random_range(2.4e-3f64..3.6e-3).ln();
        let rs = run_state(&m, &p, &ck, &eps, log_d0);
        let b = rs.b;
        // event time by inversion of the cumulative all-cause hazard on a fine grid
        let tg: Vec<f64> = grid.iter().map(|g| b * g).collect();
        let ug: Vec<f64> = tg.iter().map(|&t| rs.r_of_t(&num, t)).collect();
        let lam: Vec<f64> = ug.iter().map(|&u| -rs.log_survival_at_u(u).0).collect();
        let e: f64 = Exp1.sample(&mut rng);
        let camera_end = rng.random_range(0.6..1.2) * b;
        let (t_ev, cause) = if e < *lam.last().unwrap() {
            let t_ev = interp(e, &lam, &tg);
            let ut = interp(t_ev, &tg, &ug);
            let lam_h = rs.integ.log_lam_h(1.0 - ut).exp();
            let c = if rng.random::<f64>() < lam_h / (lam_h + rs.lam_r) {
                "hot_phase_end"
            } else {
                "disruption"
            };
            (Some(t_ev), Some(c.to_string()))
        } else {
            (None, None)
        };
        let end = [t_ev, Some(b), Some(camera_end)]
            .into_iter()
            .flatten()
            .fold(f64::INFINITY, f64::min);
        let n = (end / cadence_s) as usize;
        let mut t: Vec<f64> = (0..n)
            .map(|k| {
                ((k as f64 * cadence_s + rng.random_range(-0.02..0.02)).max(0.0) * 1e4).round()
                    / 1e4
            })
            .collect();
        t.sort_by(f64::total_cmp);
        t.dedup();
        t.retain(|&x| x < end);
        t.truncate(MAX_OBS);
        let d: Vec<f64> = t
            .iter()
            .map(|&tt| rs.d0 * rs.r_of_t(&num, tt).sqrt())
            .collect();
        let tj: Vec<f64> = t
            .iter()
            .enumerate()
            .map(|(k, x)| x + 1e-9 * k as f64)
            .collect();
        let noise = sample_student_t_process(&mut rng, &tj, sc, sn, ell, num.nu);
        let y: Vec<f64> = d.iter().zip(&noise).map(|(a, b)| a + b).collect();
        let valid: Vec<bool> = t.iter().map(|_| rng.random::<f64>() >= 0.05).collect();
        let t_last = t.last().copied().unwrap_or(0.0);
        let (censoring, l, r, causes) = match (t_ev, &cause) {
            (Some(te), Some(c)) if te <= b.min(camera_end) => (
                "interval",
                t_last,
                Some((t_last + cadence_s).max(te + 1e-6)),
                vec![c.clone()],
            ),
            _ => ("right", t_last, None, vec![]),
        };
        let rid = format!("syn-flex-{seed}-{i:03}");
        let gid = format!("syn-date-{seed}-{:02}", i / runs_per_group);
        views.push(FlexTrajectory {
            run_id: rid.clone(),
            group_id: gid,
            available_at_s: t.clone(),
            t_s: t,
            d_m: y,
            valid,
            conditions: conds,
            censoring: censoring.into(),
            t_last_active_s: Some(l),
            t_first_terminal_s: r,
            allowed_causes: causes,
            event_before_support_exit: None,
            time_origin: TIME_ORIGIN.into(),
            evidence_class: "synthetic".into(),
        });
        truth.insert(
            rid,
            RunTruth {
                eps_b: eps,
                log_d0,
                event_time: t_ev,
                cause,
                b,
            },
        );
    }
    let mut groups: Vec<String> = views
        .iter()
        .map(|v| v.group_id.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for i in (1..groups.len()).rev() {
        let j = rng.random_range(0..=i);
        groups.swap(i, j);
    }
    let ng = groups.len();
    let nv = (ng / 5).max(1);
    let nt = (ng / 5).max(1);
    let mut test: Vec<String> = groups[..nt].to_vec();
    let mut val: Vec<String> = groups[nt..(nt + nv).min(ng)].to_vec();
    let mut train: Vec<String> = groups[(nt + nv).min(ng)..].to_vec();
    test.sort();
    val.sort();
    train.sort();
    let run_ids: Vec<String> = views.iter().map(|v| v.run_id.clone()).collect();
    let cohort_id = content_hash(
        &serde_json::json!({"synthetic_flex": seed, "n_runs": n_runs, "runs": run_ids}),
        &[],
    );
    let mut split = Split {
        id: String::new(),
        cohort_id: cohort_id.clone(),
        group_key: "synthetic_date".into(),
        rule: "synthetic shuffled groups 60/20/20".into(),
        seed: seed as i64,
        run_to_group: views
            .iter()
            .map(|v| (v.run_id.clone(), v.group_id.clone()))
            .collect(),
        train,
        validation: val,
        test,
        frozen_at: "2026-11-07T00:00:00+00:00".into(),
    };
    split.id = content_hash(&split, &["id"]);
    let cohort = Cohort {
        id: cohort_id,
        family: Family::Flex,
        version: "synthetic-1".into(),
        criteria: [(
            "kind".to_string(),
            "synthetic FLEX fixture (never NASA)".to_string(),
        )]
        .into_iter()
        .collect(),
        run_ids,
        excluded: vec![],
        source_manifest_hash: "synthetic".into(),
    };
    Ok(SyntheticFlex {
        views,
        split,
        cohort,
        truth,
    })
}

/// The stored records of one generated run. These are the fixture's source records (synthetic by
/// construction), not a derived view of a NASA run.
pub fn archive_of(v: &FlexTrajectory) -> RunArchive {
    let der = Derivation {
        method: Method::Numerical,
        method_id: "flex.synthetic".into(),
        method_version: "1".into(),
        inputs: vec![],
        agent: None,
        notes: None,
    };
    let conditions = v
        .conditions
        .iter()
        .map(|(k, x)| {
            (
                k.clone(),
                Measured {
                    raw_value: Some(RawValue::Number(*x)),
                    raw_unit: None,
                    value_si: Some(*x),
                    unit_si: Some(if k == "pressure" {
                        "Pa".into()
                    } else {
                        "1".into()
                    }),
                    uncertainty_si: None,
                    uncertainty_reason: None,
                    evidence_class: EvidenceClass::Synthetic,
                    locator: None,
                    derivation: None,
                    resolution: Default::default(),
                    flags: vec![],
                },
            )
        })
        .collect();
    let record = ExperimentRecord {
        id: v.run_id.clone(),
        investigation_id: "SYNTHETIC".into(),
        family: Family::Flex,
        session_id: Some(v.group_id.clone()),
        specimen_id: None,
        aliases: vec![],
        fuel_or_material: Some("methanol".into()),
        configuration: None,
        gravity: None,
        conditions,
        identity_resolution: Default::default(),
        identity_evidence: vec![],
        quality_flags: vec![],
        notes: None,
    };
    let loc = SourceLocator {
        asset_sha256: "0".repeat(64),
        path: Some(format!("synthetic://{}", v.run_id)),
        row: None,
        column: None,
        page: None,
        frame: None,
        pts_s: None,
        char_start: None,
        char_end: None,
        ..Default::default()
    };
    let observations = (0..v.t_s.len())
        .map(|j| Observation {
            id: format!("{}-o{j:04}", v.run_id),
            run_id: v.run_id.clone(),
            quantity: Quantity::LiquidDiameter,
            channel: None,
            time_s: v.t_s[j],
            available_at_s: v.available_at_s.get(j).copied().unwrap_or(v.t_s[j]),
            raw_value: v.d_m[j].is_finite().then_some(RawValue::Number(v.d_m[j])),
            raw_unit: Some("m".into()),
            value_si: v.d_m[j].is_finite().then_some(v.d_m[j]),
            unit_si: "m".into(),
            uncertainty_si: None,
            calibration_id: None,
            quality_flags: if v.valid[j] {
                vec![]
            } else {
                vec!["flagged".into()]
            },
            evidence_class: EvidenceClass::Synthetic,
            locator: loc.clone(),
            derivation: der.clone(),
        })
        .collect();
    let censoring = match v.censoring.as_str() {
        "event" => Censoring::Event,
        "interval" => Censoring::Interval,
        "right" => Censoring::Right,
        _ => Censoring::Unknown,
    };
    let endpoint = EndpointRecord {
        run_id: v.run_id.clone(),
        original_label: None,
        original_comment: None,
        allowed_causes: v.allowed_causes.clone(),
        censoring,
        last_confirmed_active_s: v.t_last_active_s,
        first_terminal_s: v.t_first_terminal_s,
        intervention_s: None,
        intervention_kind: None,
        observation_state: None,
        resolution: Resolution::Resolved,
        evidence: vec![],
        derivation: der,
    };
    RunArchive {
        record,
        observations,
        contours: vec![],
        control: None,
        endpoint: Some(endpoint),
        time_origin: v.time_origin.clone(),
    }
}

impl ViewSource for SyntheticFlex {
    fn cohort(&self, cohort_id: &str) -> anyhow::Result<Cohort> {
        anyhow::ensure!(
            cohort_id == self.cohort.id,
            "unknown synthetic cohort {cohort_id}"
        );
        Ok(self.cohort.clone())
    }
    fn split(&self, split_id: &str) -> anyhow::Result<Split> {
        anyhow::ensure!(
            split_id == self.split.id,
            "unknown synthetic split {split_id}"
        );
        Ok(self.split.clone())
    }
    fn views(&self, family: Family, cohort_id: &str) -> anyhow::Result<Vec<View>> {
        anyhow::ensure!(
            family == Family::Flex && cohort_id == self.cohort.id,
            "synthetic FLEX fixture only"
        );
        Ok(self.views.iter().cloned().map(View::Flex).collect())
    }
    fn run_archives(&self, family: Family) -> anyhow::Result<Vec<RunArchive>> {
        anyhow::ensure!(family == Family::Flex, "synthetic FLEX fixture only");
        Ok(self.views.iter().map(archive_of).collect())
    }
}

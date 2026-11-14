//! Essential BASS checks (V3.1 §26): cosine normalisation, parameter inventory, bootstrap multiplicity
//! = duplicated runs, and one synthetic fit → save → load → forecast smoke (with refusals).

use ignis_bass::expert::{BassBaselineExpert, BassOperatorExpert};
use ignis_bass::model::{self as m, GlobalLayout};
use ignis_bass::operator::{self as op, Grid};
use ignis_core::contracts::{
    ContourObservation, ContourStatus, Derivation, EvidenceClass, ExperimentRecord, Family,
    Measured, Method, Resolution, SourceLocator, Split,
};
use ignis_core::expert::{Expert, FitContext, FitData, Refusal};
use ignis_core::prefix::{build_prefix, PrefixInputs};
use ignis_core::registry::load;
use ignis_core::views::{BassSequence, View};
use serde_json::json;

fn ctx(root: &std::path::Path) -> FitContext {
    FitContext {
        profile: serde_json::from_value(json!({"name": "test", "device": "cpu"})).unwrap(),
        pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
        seed: 0,
        registry_root: root.to_path_buf(),
        checkpoint_dir: root.join("ck"),
        resume: None,
        job_id: None,
    }
}

fn measured(v: f64) -> Measured {
    Measured {
        raw_value: None,
        raw_unit: None,
        value_si: Some(v),
        unit_si: None,
        uncertainty_si: None,
        uncertainty_reason: None,
        evidence_class: EvidenceClass::Synthetic,
        locator: None,
        derivation: None,
        resolution: Resolution::Resolved,
        flags: vec![],
    }
}

#[test]
fn cosine_normalisation_and_inventory() {
    let k = op::N_NODES;
    let xi = op::nodes(k);
    let phi = op::cosine_basis(&xi, 8);
    let q = op::trapezoid_weights(k);
    assert!((q.iter().sum::<f64>() - 1.0).abs() < 1e-15 && q[0] == 1.0 / (2 * (k - 1)) as f64);
    let mut worst: f64 = 0.0;
    for a in 0..8 {
        for b in 0..8 {
            let g: f64 = (0..k).map(|j| q[j] * phi[j * 8 + a] * phi[j * 8 + b]).sum();
            worst = worst.max((g - if a == b { 1.0 } else { 0.0 }).abs());
        }
    }
    assert!(worst < 1e-14, "weighted orthogonality error {worst}");
    let grid = Grid::new(k);
    let c = op::coefficients(&grid, &vec![1.0; k], 4);
    assert!(
        (c[0] - 1.0).abs() < 1e-14 && c[1..].iter().all(|v| v.abs() < 1e-14),
        "{c:?}"
    );
    let mode2: Vec<f64> = (0..k).map(|j| phi[j * 8 + 2]).collect();
    let c2 = op::coefficients(&grid, &mode2, 4);
    assert!(
        (c2[2] - 1.0).abs() < 1e-14
            && c2[0].abs() < 1e-14
            && c2[1].abs() < 1e-14
            && c2[3].abs() < 1e-14
    );
    assert_eq!(
        op::n_weights(),
        144 + 3 * (4 * 16 * 16 + 16 * 16 + 16) + 17 + 1776
    );
    assert_eq!(op::n_weights(), 5825);
    assert_eq!(m::n_global(), 5838);
    assert_eq!(GlobalLayout::new().len, 5838);
    assert_eq!(m::N_LOCAL, 12);
    assert_eq!(ignis_bass::experts().len(), 3);
}

#[test]
fn bootstrap_multiplicity_equals_duplicated_runs() {
    std::env::set_var(
        "IGNIS_CONFIGS",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../configs"),
    );
    let s =
        ignis_bass::synthetic::synthetic_sequence("r", "g", 3, 4, 1.0, 40, 0.04, 0.1, 5e-5, None);
    let cfg = ignis_bass::expert::load_config().unwrap();
    let feats = ignis_bass::expert::features_of(&cfg).unwrap();
    let (runs, _) = ignis_bass::data::prepare(&[&s], &feats, &cfg, 32);
    let norm = ignis_bass::data::fit_normalization(&feats, &runs);
    let na = ignis_bass::data::norm_arrays(&feats, &norm);
    let lay = GlobalLayout::new();
    let mut p = m::init_network(&lay, 0);
    p[lay.phys.beta_v] = 1.0;
    let pr = m::Priors::from_json(&cfg["priors"]);
    for (i, off) in [lay.lsc, lay.lsn, lay.lsr].into_iter().enumerate() {
        p[off] = pr.obs[i].0;
    }
    let x: Vec<f64> = m::init_locals(&runs[0]).iter().map(|v| v + 0.05).collect();
    let grid = Grid::new(32);
    let num = m::numerics_from(&cfg, false);
    let (mut g1, mut x1) = (vec![0.0; lay.len], vec![0.0; 12]);
    let twice = m::member_objective(
        &p,
        &x,
        &runs,
        &[2.0],
        &lay,
        &grid,
        num,
        &na,
        &pr,
        &mut g1,
        &mut x1,
    )
    .value;
    let dup = vec![runs[0].clone(), runs[0].clone()];
    let (mut g2, mut x2) = (vec![0.0; lay.len], vec![0.0; 24]);
    let copies = m::member_objective(
        &p,
        &[x.clone(), x].concat(),
        &dup,
        &[1.0, 1.0],
        &lay,
        &grid,
        num,
        &na,
        &pr,
        &mut g2,
        &mut x2,
    )
    .value;
    assert!(
        (twice - copies).abs() <= 1e-12 * twice.abs(),
        "{twice} vs {copies}"
    );
}

fn prefix(
    seq: &BassSequence,
    cutoff: f64,
    expert: &dyn Expert,
    override_status: Option<&str>,
) -> ignis_core::contracts::PrefixBundle {
    let ns = seq.s_m.len();
    let loc = SourceLocator {
        asset_sha256: "0".repeat(64),
        path: None,
        row: None,
        column: None,
        page: None,
        frame: None,
        pts_s: None,
        char_start: None,
        char_end: None,
        ..Default::default()
    };
    let der = Derivation {
        method: Method::Numerical,
        method_id: "synthetic".into(),
        method_version: "0".into(),
        inputs: vec![],
        agent: None,
        notes: None,
    };
    let cons: Vec<ContourObservation> = seq
        .t_s
        .iter()
        .enumerate()
        .filter(|(_, t)| **t <= cutoff)
        .map(|(i, &t)| {
            let st = if override_status.is_some() && i == 2 {
                override_status.unwrap().to_string()
            } else {
                seq.status[i].clone()
            };
            ContourObservation {
                id: format!("{}-{i}", seq.run_id),
                run_id: seq.run_id.clone(),
                time_s: t,
                available_at_s: t,
                s_m: seq.s_m.clone(),
                f_m: seq.f_m[i * ns..(i + 1) * ns]
                    .iter()
                    .map(|v| v.is_finite().then_some(*v))
                    .collect(),
                status: serde_json::from_value(json!(st)).unwrap(),
                calibration_id: "cal".into(),
                scale_uncertainty_rel: Some(0.005),
                mask_dependency_frames: vec![i as i64],
                frame: i as i64,
                evidence_class: EvidenceClass::IgnisDerived,
                locator: loc.clone(),
                derivation: der.clone(),
            }
        })
        .collect();
    let mut cond: Vec<(String, f64)> = seq
        .conditions
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    cond.extend(seq.geometry.iter().map(|(k, v)| (k.clone(), *v)));
    let rec = ExperimentRecord {
        id: seq.run_id.clone(),
        investigation_id: "BASS".into(),
        family: Family::Bass,
        session_id: None,
        specimen_id: None,
        aliases: vec![],
        fuel_or_material: None,
        configuration: None,
        gravity: None,
        conditions: cond.into_iter().map(|(k, v)| (k, measured(v))).collect(),
        identity_resolution: Resolution::Resolved,
        identity_evidence: vec![],
        quality_flags: vec![],
        notes: None,
    };
    let _ = ContourStatus::SingleValued;
    build_prefix(
        &rec,
        &[],
        cutoff,
        expert.policy(),
        "ignition",
        PrefixInputs {
            contours: &cons,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn synthetic_fit_save_load_forecast_smoke() {
    std::env::set_var(
        "IGNIS_CONFIGS",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../configs"),
    );
    let seqs = ignis_bass::synthetic::synthetic_sequences(4, 2, 0);
    let mut groups: Vec<String> = seqs.iter().map(|s| s.group_id.clone()).collect();
    groups.sort();
    groups.dedup();
    let split = Split {
        id: "split".into(),
        cohort_id: "cohort".into(),
        group_key: "specimen".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: seqs
            .iter()
            .map(|s| (s.run_id.clone(), s.group_id.clone()))
            .collect(),
        train: groups[..3].to_vec(),
        validation: groups[3..].to_vec(),
        test: vec![],
        frozen_at: "2026-11-07T00:00:00Z".into(),
    };
    let views: Vec<View> = seqs.iter().cloned().map(View::Bass).collect();
    let data = FitData {
        views: &views,
        split: &split,
        cohort_id: "cohort",
        source_manifest_hash: "synthetic",
        synthetic: true,
    };
    let dir = std::env::temp_dir().join(format!("ignis-bass-smoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let expert = BassOperatorExpert::new().unwrap();
    let steps = 200;
    let t = std::time::Instant::now();
    let rel = expert.fit(&data, &json!({"train": {"steps": steps, "n_members": 5}, "forecast": {"validation_particles": 8}}), &mut ctx(&dir)).unwrap();
    eprintln!("BASS fit: 5 members x {steps} AdamW steps on 6 runs in {:.2} s ({:.1} member-steps/s); validation {:?}",
        t.elapsed().as_secs_f64(), rel.numerics["member_steps_per_s"].as_f64().unwrap(), rel.evaluation.get("validation_mean_log_lik"));
    assert_eq!(rel.architecture["n_members"], 5);
    assert_eq!(rel.architecture["n_network_weights"], 5825);
    let ens = rel.architecture["ensemble"].as_array().unwrap();
    let mults: std::collections::BTreeSet<String> = ens
        .iter()
        .map(|e| e["group_multiplicity"].to_string())
        .collect();
    assert!(mults.len() > 1, "bootstrap members must differ");
    for mm in rel.numerics["members"].as_array().unwrap() {
        assert_eq!(mm["unreached_s"].as_f64().unwrap(), 0.0);
        assert!(mm["max_cfl"].as_f64().unwrap() <= 0.4 + 1e-12);
        assert!(
            mm["final_objective"].as_f64().unwrap() < mm["initial_objective"].as_f64().unwrap()
        );
    }
    let loaded = load(&dir, &rel.id).unwrap();
    let n0: usize = loaded
        .params
        .iter()
        .filter(|(k, _)| k.starts_with("m0.") && !k.starts_with("m0.local"))
        .map(|(_, t)| t.len())
        .sum();
    assert_eq!(n0, 5838);
    let fresh = BassOperatorExpert::new().unwrap();
    let seq = &seqs[0];
    let t = std::time::Instant::now();
    let art = fresh
        .forecast(
            &loaded,
            &prefix(seq, 3.0, &fresh, None),
            &json!({"times_s": [5.0, 6.0], "forecast": {"n_particles": 16}}),
        )
        .unwrap();
    eprintln!(
        "BASS forecast (5 members x 16 particles): {:.3} s",
        t.elapsed().as_secs_f64()
    );
    art.validate().unwrap();
    assert_eq!(art.quantiles.len(), 3);
    for j in 0..art.times_s.len() {
        assert!(
            art.quantiles[0][j] <= art.quantiles[1][j]
                && art.quantiles[1][j] <= art.quantiles[2][j]
        );
    }
    assert!(art.diagnostics["importance"]["ess"].as_f64().unwrap() > 1.0);
    assert_eq!(
        art.diagnostics["member_weights"].as_array().unwrap().len(),
        5
    );
    let refused = fresh
        .forecast(
            &loaded,
            &prefix(seq, 3.0, &fresh, Some("multivalued")),
            &json!({}),
        )
        .unwrap_err();
    assert_eq!(Refusal::from_error(&refused).unwrap().code, "topology");
    let future = fresh
        .forecast(
            &loaded,
            &prefix(seq, 3.0, &fresh, None),
            &json!({"control_changes_s": [4.5]}),
        )
        .unwrap_err();
    assert_eq!(Refusal::from_error(&future).unwrap().code, "future_control");
    let ll = fresh.log_likelihood(&loaded, &views[7]).unwrap();
    assert!(ll.is_finite());
    for kind in ["persistence", "constant_velocity"] {
        let base = BassBaselineExpert::new(kind).unwrap();
        let brel = base.fit(&data, &json!({}), &mut ctx(&dir)).unwrap();
        let bl = load(&dir, &brel.id).unwrap();
        let bart = base
            .forecast(
                &bl,
                &prefix(seq, 3.0, &base, None),
                &json!({"times_s": [5.0]}),
            )
            .unwrap();
        bart.validate().unwrap();
        assert_eq!(bart.mean.len(), 1);
        assert!(base.log_likelihood(&bl, &views[7]).unwrap().is_finite());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

//! Essential ACME checks (V3.1 §26): parameter inventory and one synthetic fit → save → load →
//! forecast smoke with the policy refusals. Timing of the value+gradient path is printed.

use std::collections::BTreeMap;

use ignis_acme::expert::AcmeExpert;
use ignis_acme::model::{self as m, AcmeSpec};
use ignis_core::contracts::{
    ControlStage, EvidenceClass, ExperimentRecord, Family, Measured, Resolution, Split,
};
use ignis_core::expert::{Expert, FitContext, FitData, Refusal};
use ignis_core::prefix::{build_prefix, PrefixInputs};
use ignis_core::registry::load;
use ignis_core::resources::Profile;
use ignis_core::views::View;
use serde_json::json;

pub fn profile() -> Profile {
    serde_json::from_value(json!({"name": "test", "device": "cpu"})).unwrap()
}

pub fn ctx(root: &std::path::Path) -> FitContext {
    FitContext {
        profile: profile(),
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

fn configs() {
    std::env::set_var(
        "IGNIS_CONFIGS",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../configs"),
    );
}

#[test]
fn parameter_inventory() {
    let both = AcmeSpec {
        causes: vec!["extinction".into(), "operator_termination".into()],
        bin_lower_edges_s: vec![0.0, 5.0, 10.0, 20.0, 40.0, 80.0],
        t_ref_s: 10.0,
        n_features: 8,
        linear: true,
        neural: true,
        frailty: true,
        hidden: 8,
        cells_per_bin: 8,
    };
    assert_eq!(m::n_params(&both), 129);
    assert_eq!(m::n_params(&both) - 12 - 16 - 3, 9 * 8 + 8 + 8 * 2 + 2);
    let only_e = AcmeSpec {
        causes: vec!["extinction".into()],
        ..both.clone()
    };
    assert_eq!(m::n_params(&only_e), 6 + 8 + (9 * 8 + 8 + 8 + 1) + 1);
    assert_eq!(ignis_acme::experts().len(), 8);
}

#[test]
fn synthetic_fit_save_load_forecast_smoke() {
    configs();
    let rows = ignis_acme::synthetic::default_rows("normal", 0);
    let mut groups: Vec<String> = rows.iter().map(|r| r.group_id.clone()).collect();
    groups.sort();
    groups.dedup();
    let split = Split {
        id: "split".into(),
        cohort_id: "cohort".into(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: rows
            .iter()
            .map(|r| (r.run_id.clone(), r.group_id.clone()))
            .collect(),
        train: groups[..6].to_vec(),
        validation: groups[6..7].to_vec(),
        test: groups[7..].to_vec(),
        frozen_at: "2026-11-07T00:00:00Z".into(),
    };
    let views: Vec<View> = rows.iter().cloned().map(View::Acme).collect();
    let data = FitData {
        views: &views,
        split: &split,
        cohort_id: "cohort",
        source_manifest_hash: "synthetic",
        synthetic: true,
    };
    let dir = std::env::temp_dir().join(format!("ignis-acme-smoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let expert = AcmeExpert::new("normal", "hazard").unwrap();
    let t = std::time::Instant::now();
    let rel = expert
        .fit(
            &data,
            &json!({"inference": "nuts_if_qualified", "map": {"maxiter": 300},
        "nuts": {"num_warmup": 300, "num_samples": 300, "num_chains": 4}}),
            &mut ctx(&dir),
        )
        .unwrap();
    eprintln!(
        "fit (MAP + NUTS 4x600) {:.2} s; numerics {}",
        t.elapsed().as_secs_f64(),
        serde_json::to_string(&rel.numerics["inference"]).unwrap()
    );
    assert_eq!(rel.architecture["n_population_parameters"], 129);
    let loaded = load(&dir, &rel.id).unwrap();
    assert_eq!(loaded.params["residual.W1"].shape, vec![8, 9]);

    // prefix of a held-out (test-session) run
    let r = rows.iter().find(|r| r.group_id == groups[7]).unwrap();
    let rec = ExperimentRecord {
        id: r.run_id.clone(),
        investigation_id: "PSI-159".into(),
        family: Family::AcmeNormal,
        session_id: Some(r.group_id.clone()),
        specimen_id: None,
        aliases: vec![],
        fuel_or_material: None,
        configuration: Some("normal".into()),
        gravity: None,
        conditions: r
            .features
            .iter()
            .map(|(k, v)| (k.clone(), measured(*v)))
            .collect::<BTreeMap<_, _>>(),
        identity_resolution: Resolution::Resolved,
        identity_evidence: vec![],
        quality_flags: vec![],
        notes: None,
    };
    let stop = ControlStage {
        t_start_s: 60.0,
        t_end_s: None,
        control: "MFCs off".into(),
        value: None,
        operator_action: true,
    };
    let prefix = build_prefix(
        &rec,
        &[],
        10.0,
        expert.policy(),
        "ignition",
        PrefixInputs {
            known_controls: std::slice::from_ref(&stop),
            protocol_scheduled: &["MFCs off"],
            phase_established_at_s: Some(2.0),
            ..Default::default()
        },
    )
    .unwrap();
    let fresh = AcmeExpert::new("normal", "hazard").unwrap();
    let t = std::time::Instant::now();
    let art = fresh
        .forecast(
            &loaded,
            &prefix,
            &json!({"override_condition_support": true}),
        )
        .unwrap();
    eprintln!(
        "forecast (24^2 nodes x {} draws) {:.3} s",
        loaded
            .posterior
            .values()
            .next()
            .map(|t| t.shape[0])
            .unwrap_or(1),
        t.elapsed().as_secs_f64()
    );
    art.validate().unwrap();
    let n = art.times_s.len();
    for j in 0..n {
        let s: f64 = art.event_cdf.iter().map(|r| r[j]).sum();
        assert!((s - 1.0).abs() < 1e-12);
    }
    let i_stop = art.times_s.iter().position(|t| *t == 60.0).unwrap();
    assert!(
        art.event_cdf[0][i_stop..].iter().all(|v| *v == 0.0),
        "no fed-phase survival after the stop"
    );
    assert!(art.diagnostics["interpretation"]
        .as_str()
        .unwrap()
        .contains("observed experimental policy"));

    let cf = fresh
        .forecast(
            &loaded,
            &prefix,
            &json!({"estimand": "counterfactual_natural"}),
        )
        .unwrap_err();
    assert_eq!(
        Refusal::from_error(&cf).unwrap().code,
        "counterfactual_disabled"
    );

    // a normal model never serves an inverse prefix
    let inv = AcmeExpert::new("inverse", "hazard").unwrap();
    let rec_inv = ExperimentRecord {
        family: Family::AcmeInverse,
        ..rec.clone()
    };
    let p_inv = build_prefix(
        &rec_inv,
        &[],
        10.0,
        inv.policy(),
        "ignition",
        PrefixInputs {
            phase_established_at_s: Some(2.0),
            ..Default::default()
        },
    )
    .unwrap();
    let wrong = fresh.forecast(&loaded, &p_inv, &json!({})).unwrap_err();
    assert_eq!(Refusal::from_error(&wrong).unwrap().code, "family_mismatch");

    // known session: a training run is refused, a new run of a training session is served
    let tr = rows.iter().find(|r| r.group_id == groups[0]).unwrap();
    let rec_tr = ExperimentRecord {
        id: tr.run_id.clone(),
        session_id: Some(tr.group_id.clone()),
        ..rec.clone()
    };
    let p_tr = build_prefix(
        &rec_tr,
        &[],
        10.0,
        expert.policy(),
        "ignition",
        PrefixInputs {
            phase_established_at_s: Some(2.0),
            ..Default::default()
        },
    )
    .unwrap();
    let e = fresh
        .forecast(
            &loaded,
            &p_tr,
            &json!({"generalization": "known_session", "override_condition_support": true}),
        )
        .unwrap_err();
    assert_eq!(
        Refusal::from_error(&e).unwrap().code,
        "training_run_known_session"
    );
    let rec_new = ExperimentRecord {
        id: "new-run".into(),
        ..rec_tr
    };
    let p_new = build_prefix(
        &rec_new,
        &[],
        10.0,
        expert.policy(),
        "ignition",
        PrefixInputs {
            phase_established_at_s: Some(2.0),
            ..Default::default()
        },
    )
    .unwrap();
    let ks = fresh
        .forecast(
            &loaded,
            &p_new,
            &json!({"generalization": "known_session", "override_condition_support": true}),
        )
        .unwrap();
    ks.validate().unwrap();

    // baselines fit and forecast on the same data
    for v in ["constant_baseline", "piecewise_baseline", "linear_baseline"] {
        let b = AcmeExpert::new("normal", v).unwrap();
        let brel = b.fit(&data, &json!({}), &mut ctx(&dir)).unwrap();
        let bl = load(&dir, &brel.id).unwrap();
        let ba = b
            .forecast(&bl, &prefix, &json!({"override_condition_support": true}))
            .unwrap();
        ba.validate().unwrap();
        let ll = b.log_likelihood(&bl, &views[0]).unwrap();
        assert!(ll.is_finite());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

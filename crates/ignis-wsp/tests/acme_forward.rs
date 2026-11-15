//! The ACME strict forward on synthetic prefixes with test-only weights: exact competing risks, the
//! role-aware feature columns, the separation of the two configurations and of the two causes.

mod common;

use ignis_core::contracts::{
    ControlStage, Family, HandwrittenWeightPatch, ModelSpec, PrefixBundle, SupportGate,
};
use ignis_core::expert::Expert;
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::acme::{self, Forecast, CATEGORIES};
use ignis_wsp::{acme_tools, assemble, text};
use serde_json::{json, Value};

/// Constant hazards: extinction 0.05 /s, operator 0.02 /s. Not a candidate.
const TOY: &str = "
ext.b0 = -2.995732273553991
op.b0 = -3.912023005428146
";

fn release(family: Family, extra: &str) -> LoadedRelease {
    let spec: ModelSpec = acme::spec(family);
    let mut src = String::from(TOY);
    for skip in ["ext.skip", "op.skip"] {
        if !extra.contains(skip) {
            src.push_str(&format!("{skip}[0:9] = 0.0\n"));
        }
    }
    src.push_str(extra);
    let ops = text::compile(&src, &spec).unwrap_or_else(|e| panic!("{e}\n{src}"));
    let patch: HandwrittenWeightPatch = assemble::make_patch(&spec, common::meta("test-acme"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = ignis_wsp::build_release(family, &patch, &tensors, None, &common::binding())
        .expect("release");
    LoadedRelease {
        release,
        params: tensors,
        posterior: TensorMap::new(),
    }
}

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

fn run(rel: &LoadedRelease, p: &PrefixBundle, settings: Value) -> Forecast {
    acme::forecast(&acme::spec(rel.release.family), rel, p, &settings)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

#[test]
fn constant_hazards_give_the_closed_form_competing_risks() {
    let rel = release(Family::AcmeNormal, "");
    let p = acme_tools::default_fixture(Family::AcmeNormal);
    let f = run(&rel, &p, json!({"times_s": [3.0, 5.0, 13.0, 40.0, 200.0]}));
    let (le, lo) = (0.05, 0.02);
    for (j, t) in f.times_s.iter().enumerate() {
        let lost = 1.0 - (-(le + lo) * (t - 3.0)).exp();
        assert!(
            (f.table[0][j] - (1.0 - lost)).abs() < 1e-12,
            "active at {t}"
        );
        assert!(
            (f.table[1][j] - lost * le / (le + lo)).abs() < 1e-12,
            "extinction at {t}"
        );
        assert!(
            (f.table[2][j] - lost * lo / (le + lo)).abs() < 1e-12,
            "operator at {t}"
        );
        assert_eq!(f.table[3][j], 0.0);
        let total: f64 = f.table.iter().map(|r| r[j]).sum();
        assert!((total - 1.0).abs() < 1e-12);
    }
    // conditioning on the cutoff: the same release at a later cutoff restarts from "active = 1"
    let later = acme_tools::fixture_prefix(
        Family::AcmeNormal,
        20.0,
        101_325.0,
        1.0e-6,
        0.30,
        0.30,
        2300.0,
        0.25,
        "propane",
    );
    assert_eq!(
        run(&rel, &later, json!({"times_s": [20.0]})).table[0][0],
        1.0
    );
    // beyond the model horizon nothing is invented: what is left is unresolved, not a cause
    let far = run(&rel, &p, json!({"times_s": [599.0, 600.0, 700.0]}));
    assert!(far.table[0][0] > 0.0 && far.table[3][0] == 0.0);
    assert!(far.table[0][2] == 0.0 && far.table[3][2] > 0.0);
    assert!((far.table[1][1] - far.table[1][2]).abs() < 1e-15);
}

#[test]
fn features_keep_their_columns_and_each_configuration_its_own_roles() {
    // one weight at a time on the extinction skip: the hazard must scale by exp(w * feature)
    let expected_normal = [
        (0, (2.0_f64).ln()),            // c_logp at 2 atm
        (1, (2.5_f64).ln()),            // c_logm: fuel flow 2.5 mg/s over 1 mg/s
        (2, (0.40 - 0.30) / 0.10),      // c_o2: ambient oxygen
        (3, (0.60_f64 / 0.30).ln()),    // c_xf: injected fuel fraction
        (4, (2500.0 - 2200.0) / 300.0), // c_tad
        (5, (0.45 - 0.25) / 0.20),      // c_zst
        (6, 0.0),                       // f_ethane
        (7, 1.0),                       // f_butane
    ];
    let p = acme_tools::fixture_prefix(
        Family::AcmeNormal,
        3.0,
        2.0 * 101_325.0,
        2.5e-6,
        0.40,
        0.60,
        2500.0,
        0.45,
        "butane",
    );
    for (col, x) in expected_normal {
        let mut skip = [0.0; 9];
        skip[col] = 0.7;
        let rel = release(Family::AcmeNormal, &format!("ext.skip[0:9] = {skip:?}"));
        let f = run(&rel, &p, json!({"times_s": [3.0, 4.0]}));
        assert!(
            (f.features[col] - x).abs() < 1e-12,
            "feature {col}: {} != {x}",
            f.features[col]
        );
        assert!(
            (f.hazard_ext[0] - 0.05 * (0.7 * x).exp()).abs() < 1e-12,
            "column {col}"
        );
        assert!(
            (f.hazard_op[0] - 0.02).abs() < 1e-15,
            "the operator hazard moved with column {col}"
        );
    }
    // inverse: other reference scales for the same keys (flow 5 mg/s, oxygen 0.50 +- 0.25, fuel 0.10)
    let q = acme_tools::fixture_prefix(
        Family::AcmeInverse,
        3.0,
        101_325.0,
        1.0e-5,
        0.85,
        0.20,
        2500.0,
        0.45,
        "ethane",
    );
    let inv = run(&release(Family::AcmeInverse, ""), &q, Value::Null);
    let want = [0.0, (2.0_f64).ln(), 1.4, (2.0_f64).ln(), 1.0, 0.0, 1.0, 0.0];
    for (i, w) in want.iter().enumerate() {
        assert!(
            (inv.features[i] - w).abs() < 1e-12,
            "inverse feature {i}: {} != {w}",
            inv.features[i]
        );
    }
    // elapsed time enters as ln(1 + t / 10 s): a weight of 2 gives a hazard growing as (1 + t/10)^2
    let mut skip = [0.0; 9];
    skip[8] = 2.0;
    let rel = release(Family::AcmeNormal, &format!("ext.skip[0:9] = {skip:?}"));
    let f = run(
        &rel,
        &acme_tools::default_fixture(Family::AcmeNormal),
        json!({"times_s": [10.0, 30.0]}),
    );
    assert!(
        (f.hazard_ext[0] - 0.05 * 4.0).abs() < 1e-12
            && (f.hazard_ext[1] - 0.05 * 16.0).abs() < 1e-12
    );
}

#[test]
fn configurations_are_never_pooled_and_a_stop_is_never_an_extinction() {
    let normal = release(Family::AcmeNormal, "");
    let inverse = release(Family::AcmeInverse, "");
    let pn = acme_tools::default_fixture(Family::AcmeNormal);
    let pi = acme_tools::default_fixture(Family::AcmeInverse);
    // a release of one configuration refuses a prefix of the other, in both directions
    for (rel, p) in [(&normal, &pi), (&inverse, &pn)] {
        let s = acme::forecast(&acme::spec(rel.release.family), rel, p, &Value::Null)
            .expect("runnable")
            .expect_err("pooled");
        assert_eq!(s.gate, SupportGate::Semantic);
    }
    // and a release cannot be run through the other configuration's spec
    assert!(acme::forecast(&acme::spec(Family::AcmeInverse), &normal, &pi, &Value::Null).is_err());
    // the two specs differ where the roles differ
    let (sn, si) = (
        acme::spec(Family::AcmeNormal),
        acme::spec(Family::AcmeInverse),
    );
    assert_ne!(sn.spec_hash(), si.spec_hash());
    assert_ne!(sn.features[1].transform, si.features[1].transform);

    // a protocol stop known at the cutoff: the mass still active then goes to the operator row
    let stage = |t: f64| ControlStage {
        t_start_s: t,
        t_end_s: None,
        control: "scheduled_stop".into(),
        value: None,
        operator_action: true,
    };
    let mut p = pn.clone();
    p.known_controls = vec![stage(10.0)];
    let p = rehash(p);
    let f = run(&normal, &p, json!({"times_s": [9.0, 10.0, 30.0]}));
    let free = run(&normal, &pn, json!({"times_s": [9.0, 10.0, 30.0]}));
    assert_eq!(f.table[1][0], free.table[1][0]);
    assert!(f.table[0][1] == 0.0 && f.table[0][2] == 0.0);
    assert!((f.table[2][1] - (free.table[2][1] + free.table[0][1])).abs() < 1e-12);
    assert_eq!(
        f.table[1][2], f.table[1][1],
        "extinction mass grew after the stop"
    );
    assert!((f.table.iter().map(|r| r[2]).sum::<f64>() - 1.0).abs() < 1e-12);
    // a stop at or before the cutoff: the fed phase is over, no forecast
    let mut p = pn.clone();
    p.known_controls = vec![stage(2.0)];
    let s = acme::forecast(&sn, &normal, &rehash(p), &Value::Null)
        .unwrap()
        .expect_err("stopped");
    assert_eq!(s.gate, SupportGate::Semantic);

    // missing or non-positive conditions are refusals, never defaults
    let mut p = pn.clone();
    p.known_conditions.remove("tad");
    assert_eq!(
        acme::forecast(&sn, &normal, &rehash(p), &Value::Null)
            .unwrap()
            .expect_err("tad")
            .gate,
        SupportGate::Condition
    );
    let mut p = pn.clone();
    p.known_conditions
        .get_mut("injected_mass_flow")
        .unwrap()
        .value_si = Some(0.0);
    assert_eq!(
        acme::forecast(&sn, &normal, &rehash(p), &Value::Null)
            .unwrap()
            .expect_err("flow")
            .gate,
        SupportGate::Condition
    );
    let mut p = pn.clone();
    p.known_conditions.remove("fuel_species");
    assert_eq!(
        acme::forecast(&sn, &normal, &rehash(p), &Value::Null)
            .unwrap()
            .expect_err("fuel")
            .gate,
        SupportGate::Condition
    );

    // the artifact is an event partition with the fitted experts' row names
    let expert = acme::AcmeWsp::new(Family::AcmeNormal);
    let a = expert.forecast(&normal, &pn, &Value::Null).unwrap();
    a.validate().unwrap();
    assert_eq!(a.event_categories, CATEGORIES);
    assert!(a.quantity.is_none() && a.quantiles.is_empty());
    assert!(expert.objective(&normal, &normal.params, &[]).is_none());
}

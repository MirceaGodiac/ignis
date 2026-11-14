//! Essential evaluator checks: seal-before-reveal, final-test gate, logged attempts, comparable releases only.

mod common;

use common::*;
use ignis_core::registry::{Tensor, TensorMap};
use ignis_lab::eval::{self, compare, vault};
use serde_json::{json, Value};

fn continuation(r: &str, session: &str) -> Value {
    let obs: Vec<Value> =
        [1.5f64, 2.5].iter().map(|t| json!({"time_s": t, "value_si": 1e-3 * (2.0 - 0.1 * t), "quantity": "liquid_diameter"})).collect();
    json!({"session_id": session, "observations": obs, "run_id": r})
}

fn deposit_all(sp: &ignis_core::contracts::Split) {
    for (r, g) in [("r_va1", "g2"), ("r_va2", "g3"), ("r_te", "g4")] {
        vault::deposit_continuation(sp, r, continuation(r, g)).unwrap();
    }
    assert!(is_refusal(
        &vault::deposit_continuation(sp, "r_tr", continuation("r_tr", "g1")).unwrap_err(),
        "train run"
    ));
}

fn params(seed: f64) -> TensorMap {
    TensorMap::from([("w".to_string(), Tensor::vector(vec![seed; 3]))])
}

#[test]
fn reveal_requires_prior_seal_and_final_test_flag_and_logs_every_attempt() {
    let env = env("seal");
    let sp = split();
    deposit_all(&sp);
    let rel = release(
        &env,
        &sp.id,
        &["a", "b"],
        json!({"seed": 0}),
        &params(0.0),
        &TensorMap::new(),
    );
    let (va, te) = (
        artifact(&rel.id, "r_va1", 0.0),
        artifact(&rel.id, "r_te", 0.0),
    );
    assert!(is_refusal(
        &eval::reveal(&va.id, false).err().unwrap(),
        "no SealedPrediction"
    ));
    vault::seal(&va, "r_va1", &sp, &env.artifacts()).unwrap();
    vault::seal(&te, "r_te", &sp, &env.artifacts()).unwrap();
    assert!(is_refusal(
        &vault::seal(&va, "r_va1", &sp, &env.artifacts()).unwrap_err(),
        "already sealed"
    ));
    assert_eq!(
        eval::reveal(&va.id, false).unwrap().continuation["run_id"],
        "r_va1"
    );
    assert!(is_refusal(
        &eval::reveal(&te.id, false).err().unwrap(),
        "--final-test"
    ));
    assert_eq!(
        eval::reveal(&te.id, true).unwrap().continuation["partition"],
        "test"
    );
    let log = vault::reveal_log();
    let granted: Vec<bool> = log
        .iter()
        .map(|e| e["granted"].as_bool().unwrap())
        .collect();
    assert_eq!(granted, [false, true, false, true]);
    assert_eq!(log[3]["final_test"], true);
    // a test run whose continuation was opened cannot take new predictions
    let late = artifact(&rel.id, "r_te", 1e-5);
    assert!(is_refusal(
        &vault::seal(&late, "r_te", &sp, &env.artifacts()).unwrap_err(),
        "already revealed"
    ));
    // a tampered seal record is refused (and logged)
    let p = env
        .root
        .join("runs/evaluation/seals")
        .join(format!("{}.json", ignis_core::hashing::hex_part(&va.id)));
    let mut rec: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    rec["run_id"] = json!("r_va2");
    std::fs::write(&p, serde_json::to_string_pretty(&rec).unwrap()).unwrap();
    assert!(is_refusal(
        &eval::reveal(&va.id, false).err().unwrap(),
        "modified"
    ));
    assert_eq!(vault::reveal_log().len(), 5);
}

#[test]
fn compare_refuses_mismatched_feature_order_and_scores_identical_cases() {
    let env = env("compare");
    let sp = split();
    deposit_all(&sp);
    let a = release(
        &env,
        &sp.id,
        &["a", "b"],
        json!({"seed": 1}),
        &params(1.0),
        &TensorMap::new(),
    );
    let b = release(
        &env,
        &sp.id,
        &["b", "a"],
        json!({"seed": 2}),
        &params(2.0),
        &TensorMap::new(),
    );
    for (rel, shift) in [(&a, 0.0), (&b, 5e-5)] {
        for run in ["r_va1", "r_va2"] {
            vault::seal(&artifact(&rel.id, run, shift), run, &sp, &env.artifacts()).unwrap();
        }
    }
    let ids = vec![a.id.clone(), b.id.clone()];
    let mut o = compare::CompareOpts {
        partition: "validation",
        final_test: false,
        allow: compare::Allow::default(),
        intersect: false,
        horizons: None,
        n_boot: 200,
        seed: 0,
    };
    assert!(is_refusal(
        &eval::compare_releases(&ids, &o).unwrap_err(),
        "feature order"
    ));
    o.allow.feature_mismatch = true;
    o.partition = "test";
    assert!(is_refusal(
        &eval::compare_releases(&ids, &o).unwrap_err(),
        "final-test"
    ));
    o.partition = "validation";
    let res = eval::compare_releases(&ids, &o).unwrap();
    assert_eq!(res["n_cases"], 2);
    assert!(!res["allowed_mismatches"].as_array().unwrap().is_empty());
    assert_eq!(res["per_release"][&a.id]["metrics"]["crps"]["n_cases"], 2);
    assert!(
        res["paired_vs_reference"][&b.id]["differences"]["crps"]["estimate"]
            .as_f64()
            .unwrap()
            > 0.0
    );
    assert!(ignis_lab::eval::report::render(&res).contains("Paired differences"));
}

#[test]
fn cohort_deposit_never_holds_prefix_data() {
    use ignis_core::analysis::{at_risk, build_run_prefix};
    use ignis_core::contracts::{Family, Partition};
    use ignis_core::views::ViewSource;
    use ignis_lab::eval::deposit::{continuation_payload, deposit_cohort};

    let env = env("deposit");
    let ds = ignis_flex::synthetic::make_dataset(12, 1, 3, 0.1).unwrap();
    let experts = ignis_flex::experts();
    let expert = ignis_core::expert::find(&experts, "flex.v1").unwrap();
    let out = deposit_cohort(
        &ds,
        &ds.cohort.id,
        &ds.split.id,
        Some(vec![1.0, 3.0]),
        "all",
    )
    .unwrap();
    let deposited = out["deposited_cases"].as_u64().unwrap();
    assert!(deposited > 0);

    let root = env
        .root
        .join("holdout")
        .join(ds.split.id.trim_start_matches("sha256:"));
    let mut files = 0;
    for a in ds.run_archives(Family::Flex).unwrap() {
        let part = ds.split.partition_of(&a.record.id).unwrap();
        for cutoff in [1.0, 3.0] {
            if part == Partition::Train || !at_risk(&a, cutoff) {
                continue;
            }
            let payload = continuation_payload(&a, cutoff);
            let prefix = build_run_prefix(&a, cutoff, expert.policy()).unwrap();
            let inside: Vec<&str> = prefix.observations.iter().map(|o| o.id.as_str()).collect();
            for o in payload["observations"].as_array().unwrap() {
                assert!(o["time_s"].as_f64().unwrap() > cutoff, "{o}");
                assert!(!inside.contains(&o["id"].as_str().unwrap()), "{o}");
            }
            // The stored file is exactly that payload (plus the vault's own bookkeeping fields).
            let dir = root.join(if part == Partition::Test {
                "test"
            } else {
                "validation"
            });
            let name = format!(".cutoff-{cutoff}.json");
            let found: Vec<_> = std::fs::read_dir(&dir)
                .unwrap()
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().ends_with(&name))
                .collect();
            let stored: Vec<Value> = found
                .iter()
                .map(|e| serde_json::from_str(&std::fs::read_to_string(e.path()).unwrap()).unwrap())
                .collect();
            let v = stored
                .iter()
                .find(|v| v["run_id"] == a.record.id.as_str())
                .unwrap();
            assert_eq!(v["observations"], payload["observations"]);
            files += 1;
        }
    }
    assert_eq!(files as u64, deposited);

    // The vault itself refuses a payload that holds anything at or before the cutoff.
    let val_run = ds
        .views
        .iter()
        .map(|v| &v.run_id)
        .find(|r| ds.split.partition_of(r) == Some(Partition::Validation))
        .unwrap();
    let leaky = json!({"observations": [{"id": "x", "time_s": 0.5, "value_si": 1e-3, "quantity": "liquid_diameter"}]});
    assert!(is_refusal(
        &vault::deposit_landmark(&ds.split, val_run, 3.0, leaky).unwrap_err(),
        "not after the cutoff"
    ));
}

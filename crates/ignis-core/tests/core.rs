//! Essential core checks: schema drift, information isolation, checkpoint portability.

use std::collections::BTreeMap;

use ignis_core::contracts::*;
use ignis_core::prefix::{assert_isolated, build_prefix, PrefixInputs, ProspectivePolicy};
use ignis_core::registry::{load_release, save_release, Tensor, TensorMap};

#[test]
fn committed_schemas_match_the_contracts() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    for (name, schema) in ignis_core::schema::all() {
        let committed: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(
            committed,
            ignis_core::hashing::sorted(&schema),
            "{name}: run `ignis schemas export`"
        );
    }
}

fn loc() -> SourceLocator {
    serde_json::from_value(
        serde_json::json!({"asset_sha256": "0".repeat(64), "path": "fixture.csv"}),
    )
    .unwrap()
}
fn der() -> Derivation {
    serde_json::from_value(serde_json::json!({"method": "deterministic", "method_id": "fixture", "method_version": "0"})).unwrap()
}
fn obs(i: usize, t: f64, avail: f64) -> Observation {
    serde_json::from_value(serde_json::json!({
        "id": format!("o{i}"), "run_id": "r1", "quantity": "liquid_diameter", "time_s": t, "available_at_s": avail,
        "raw_value": 2.0, "raw_unit": "mm", "value_si": 2e-3, "unit_si": "m", "evidence_class": "synthetic",
        "locator": loc(), "derivation": der()})).unwrap()
}
fn record(conds: serde_json::Value) -> ExperimentRecord {
    serde_json::from_value(serde_json::json!({"id": "r1", "investigation_id": "PSI-69", "family": "flex", "conditions": conds})).unwrap()
}
fn policy() -> ProspectivePolicy {
    ProspectivePolicy::new(Family::Flex, &["pressure"], &[])
}

#[test]
fn prefix_admits_only_the_available_past_and_detects_tampering() {
    let o = vec![
        obs(0, 0.5, 0.5),
        obs(1, 1.0, 1.0),
        obs(2, 1.5, 2.5),
        obs(3, 3.0, 3.0),
    ];
    let rec = record(
        serde_json::json!({"pressure": {"raw_value": 1.0, "raw_unit": "atm", "value_si": 101325.0, "unit_si": "Pa"}}),
    );
    let p = build_prefix(
        &rec,
        &o,
        2.0,
        &policy(),
        "igniter_retracted",
        PrefixInputs::default(),
    )
    .unwrap();
    assert_eq!(
        p.observations
            .iter()
            .map(|x| x.id.as_str())
            .collect::<Vec<_>>(),
        ["o0", "o1"]
    );
    assert!(assert_isolated(&p, &o[2..]).is_ok());
    assert!(assert_isolated(&p, &o[..1]).is_err());
    let mut forged = p.clone();
    forged.cutoff_s = 9.0;
    assert!(assert_isolated(&forged, &[]).is_err());
    let bad = record(serde_json::json!({"k_full": {"raw_value": 0.5}}));
    assert!(build_prefix(&bad, &[], 1.0, &policy(), "t0", PrefixInputs::default()).is_err());
}

#[test]
fn release_roundtrip_verifies_tensors_and_feature_order() {
    let root = std::env::temp_dir().join(format!("ignis-reg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let rel: ModelRelease = serde_json::from_value(serde_json::json!({
        "id": "", "family": "flex", "model_name": "fixture", "is_baseline": true, "architecture": {}, "config": {},
        "feature_order": ["a", "b"], "units": {"a": "Pa"}, "normalization": {}, "cohort_id": "c", "split_id": "s",
        "source_manifest_hash": "m", "code_version": "test", "trained_on": "synthetic", "created_at": ""})).unwrap();
    let mut params = TensorMap::new();
    params.insert(
        "w".into(),
        Tensor::new(vec![2, 3], (0..6).map(|x| x as f64).collect()),
    );
    let saved = save_release(&root, rel, &params, &BTreeMap::new(), &[]).unwrap();
    let (_, p, _) = load_release(&root, &saved.id, Some(&["a".into(), "b".into()])).unwrap();
    assert_eq!(p["w"].shape, vec![2, 3]);
    assert!(load_release(&root, &saved.id, Some(&["b".into(), "a".into()])).is_err());
    let mut forged = TensorMap::new();
    forged.insert("w".into(), Tensor::new(vec![2, 3], vec![0.0; 6]));
    let npy = forged
        .iter()
        .map(|(k, t)| {
            (
                k.clone(),
                ignis_core::npz::NpyArray::from_f64(t.shape.clone(), &t.data),
            )
        })
        .collect();
    ignis_core::npz::write_npz(
        &ignis_core::registry::release_dir(&root, &saved.id).join("params.npz"),
        &npy,
    )
    .unwrap();
    assert!(load_release(&root, &saved.id, None).is_err());
    let _ = std::fs::remove_dir_all(&root);
}

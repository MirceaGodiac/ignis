use ignis_core::contracts::{HandwrittenWeightPatch, LiteralOp, ModelSpec};

#[test]
fn actual_spec_and_literal_envelope_validate_against_shared_contracts() {
    let spec: ModelSpec = serde_json::from_str(include_str!("spec.json")).unwrap();
    spec.validate().unwrap();
    let ops: Vec<LiteralOp> = serde_json::from_str(include_str!("ops.json")).unwrap();
    let mut patch = serde_json::from_str::<serde_json::Value>(include_str!("meta.json")).unwrap();
    patch["schema_version"] = serde_json::json!("wsp.v1");
    patch["spec_hash"] = serde_json::to_value(spec.spec_hash()).unwrap();
    patch["ops"] = serde_json::to_value(ops).unwrap();
    let patch: HandwrittenWeightPatch = serde_json::from_value(patch).unwrap();
    patch.validate_against(&spec).unwrap();
    assert_eq!(patch.author.model_id, "gpt-6-astra");
    assert!(patch.exposure.datasets.is_empty());
    assert!(patch.exposure.development_feedback.is_empty());
    assert!(patch.exposure.prior_candidates.is_empty());
    assert!(spec
        .tensors
        .iter()
        .all(|t| t.bounds.as_ref().is_none_or(|b| b.min < b.max)));
    assert_eq!(
        spec.target.quantity,
        ignis_core::contracts::Quantity::Temperature
    );
}

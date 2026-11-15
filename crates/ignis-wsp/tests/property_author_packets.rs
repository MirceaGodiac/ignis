//! Validate the actual new author packets before any release assembly.
//! These are construction tests, not experimental qualification or proof of blindness.

use ignis_core::contracts::{LiteralOp, ModelSpec};
use ignis_wsp::assemble::{self, PatchMeta};
use std::path::Path;

fn packet(base: &str, spec: &str, meta: &str, ops: &str) -> (ModelSpec, PatchMeta, Vec<LiteralOp>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/models")
        .join(base);
    (
        serde_json::from_slice(&std::fs::read(root.join(spec)).unwrap()).unwrap(),
        serde_json::from_slice(&std::fs::read(root.join(meta)).unwrap()).unwrap(),
        serde_json::from_slice(&std::fs::read(root.join(ops)).unwrap()).unwrap(),
    )
}

#[test]
fn six_scalar_specifications_materialize_their_actual_astra_literal_packets() {
    let cases = [
        (
            "droplets/property-w0/cleanroom-astra",
            "spec.json",
            "meta.json",
            "ops.json",
        ),
        (
            "spherical/property-w0/cleanroom-astra",
            "spec-normal.json",
            "patch-meta.json",
            "literal-ops.json",
        ),
        (
            "spherical/property-w0/cleanroom-astra",
            "spec-inverse.json",
            "patch-meta.json",
            "literal-ops.json",
        ),
        (
            "bre/property-w0/cleanroom-astra",
            "model_spec.json",
            "patch_meta.json",
            "literal_ops.json",
        ),
        (
            "jets/property-w0/cleanroom-astra",
            "model_spec.json",
            "patch_meta.json",
            "literal_ops.json",
        ),
        (
            "solids/property-w0/cleanroom-astra",
            "spec.json",
            "meta.json",
            "ops.json",
        ),
    ];
    let mut identities = std::collections::BTreeSet::new();
    for (base, spec_file, meta_file, ops_file) in cases {
        let (spec, meta, ops) = packet(base, spec_file, meta_file, ops_file);
        spec.validate().unwrap_or_else(|e| panic!("{base}: {e}"));
        assert!(
            identities.insert(spec.spec_id.to_string()),
            "duplicate spec"
        );
        assert_eq!(meta.author.model_id, "gpt-6-astra");
        assert!(meta.exposure.datasets.is_empty());
        assert!(meta.exposure.development_feedback.is_empty());
        assert!(meta.exposure.prior_candidates.is_empty());
        let patch = assemble::make_patch(&spec, meta, ops);
        patch.validate_against(&spec).unwrap();
        let tensors = assemble::materialize(&spec, &patch, None).unwrap();
        assert_eq!(tensors.len(), spec.tensors.len());
        for descriptor in &spec.tensors {
            let tensor = &tensors[&descriptor.name];
            assert_eq!(tensor.data.len(), descriptor.numel());
            assert!(tensor.data.iter().all(|value| value.is_finite()));
        }
        println!(
            "{} spec={} patch={} authored={}",
            spec.model_name,
            spec.spec_hash(),
            patch.patch_hash(),
            assemble::counts(&spec, &tensors).authored_total
        );
    }
}

#[test]
fn original_invalid_droplet_packet_is_still_preserved_and_refused() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/models/properties/droplet-w0-astra/spec.json");
    let spec: ModelSpec = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        spec.spec_hash().to_string(),
        "sha256:15bd54f7610888b9c5b243ac99322630e58c2e4a498789ef9137bc1c205921b8"
    );
    assert!(spec
        .validate()
        .unwrap_err()
        .to_string()
        .contains("empty bounds"));
}

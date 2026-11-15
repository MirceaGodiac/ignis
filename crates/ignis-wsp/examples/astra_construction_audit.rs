//! Outcome-free counterexamples for the literal-release construction boundary.
//! This creates no release on disk and reads only the named frozen author packet.

use anyhow::{ensure, Context};
use ignis_core::contracts::{Digest, LiteralOp, ModelSpec, UncertaintyKind};
use ignis_wsp::{assemble, audit};
use serde_json::{json, Map, Value};

fn main() -> anyhow::Result<()> {
    ensure!(
        std::env::args().any(|arg| arg == "--execute"),
        "pass --execute to run the synthetic construction audit"
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let packet = root.join("configs/models/properties/droplet-w0-astra");
    let spec: ModelSpec = serde_json::from_slice(&std::fs::read(packet.join("spec.json"))?)?;
    let ops: Vec<LiteralOp> =
        serde_json::from_slice(&std::fs::read(packet.join("literal-ops.json"))?)?;
    let mut envelope: Value =
        serde_json::from_slice(&std::fs::read(packet.join("author-envelope.json"))?)?;
    envelope
        .as_object_mut()
        .context("author envelope object")?
        .remove("schema_version");
    let meta: assemble::PatchMeta = serde_json::from_value(envelope)?;
    let patch = assemble::make_patch(&spec, meta.clone(), ops.clone());
    let spec_error = spec.validate().err().map(|error| error.to_string());
    let materialize_error = assemble::materialize(&spec, &patch, None)
        .err()
        .map(|error| error.to_string());

    // A separately identified in-memory control: same explicit literal operations,
    // with the invalid bound removed. It is never saved or claimed as a release.
    let mut control = spec.clone();
    control.tensors[0].bounds = None;
    control.validate()?;
    let control_patch = assemble::make_patch(&control, meta, ops);
    let tensors = assemble::materialize(&control, &control_patch, None)?;
    let context = assemble::ReleaseContext {
        cohort_id: "synthetic-construction-audit".into(),
        split_id: "synthetic-construction-audit".into(),
        source_manifest_hash: "synthetic-construction-audit".into(),
        applicability: Map::new(),
        observation_model: Map::new(),
        numerics: Map::new(),
        uncertainty_kind: UncertaintyKind::AssumedNoise,
        forward_hash: Digest::of_json(&"deliberately unregistered synthetic audit route"),
        checks: vec![],
    };
    // Deliberately exercise the lower-level helper without its materialize
    // precondition. The correct path above already refused these frozen inputs.
    let direct = assemble::build_release(&spec, &patch, &tensors, None, context);
    let direct_checks = direct
        .as_ref()
        .ok()
        .and_then(|release| release.construction.as_ref())
        .map(|manifest| &manifest.checks);
    let report = json!({
        "origin": "synthetic_fixture",
        "purpose": "construction-boundary counterexample, not scientific qualification",
        "frozen_spec_hash": spec.spec_hash(),
        "frozen_spec_validation_error": spec_error,
        "frozen_materialize_error": materialize_error,
        "hypothetical_control_spec_hash": control.spec_hash(),
        "unknown_route_check": audit::strict_route_check_of("astra.unregistered.synthetic.audit"),
        "direct_build_accepted_invalid_spec": direct.is_ok(),
        "direct_build_error": direct.as_ref().err().map(|error| error.to_string()),
        "direct_build_checks": direct_checks,
        "registry_write_performed": false,
        "forecast_fixture_run": false,
        "dataset_access": false
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

//! Dataset bundle invariants (R2), on synthetic records: a builder bundle holds only its partitions
//! and never a test run's values; columns keep SI units, time roles and the original value next to
//! the SI one (missing is null + invalid, never zero); the service refuses what the scope does not
//! grant and never opens the evaluator root for another role; the content digest is stable across
//! rebuilds and changes with a value.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;

use arrow_array::cast::AsArray;
use arrow_array::types::Float64Type;
use arrow_array::{Array, RecordBatch};
use ignis_core::contracts::{
    BundleAudience, Capability, Cohort, ColumnRole, DatasetBundle, DatasetId, EndpointRecord,
    ErrorCode, ExecutionScope, ExperimentRecord, Family, Observation, Partition, PrincipalId,
    Quantity, ScopeId, ScopeRole, Split, WorkspaceId,
};
use ignis_core::services::DatasetService;
use ignis_data::common::Bundle;
use ignis_data::dataset::{self, BuiltBundle, BundleSpec, DatasetStore};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde_json::json;

/// The test run's droplet diameter: must never appear in a builder bundle.
const SECRET_M: f64 = 0.00987654321;
const RUNS: [(&str, &str); 3] = [
    ("PSI-69:R1", "g1"),
    ("PSI-69:R2", "g2"),
    ("PSI-69:R3", "g3"),
];

fn loc(row: i64) -> serde_json::Value {
    json!({"asset_sha256": "ab".repeat(32), "path": "PSI-69_Raw Data_FLEX-001_DropletData.txt",
           "row": row, "column": "DropletDiameter_mm", "page": null, "frame": null, "pts_s": null,
           "char_start": null, "char_end": null})
}

fn det(id: &str) -> serde_json::Value {
    json!({"method": "deterministic", "method_id": id, "method_version": "1", "inputs": [], "agent": null, "notes": null})
}

fn record(id: &str, pressure_mmhg: f64) -> ExperimentRecord {
    serde_json::from_value(json!({
        "id": id, "investigation_id": "PSI-69", "family": "flex", "session_id": null, "specimen_id": null,
        "aliases": [], "fuel_or_material": "Heptane", "configuration": "droplet", "gravity": null,
        "conditions": {
            "pressure": {"raw_value": pressure_mmhg, "raw_unit": "mmHg", "value_si": pressure_mmhg * 133.322387415,
                "unit_si": "Pa", "uncertainty_si": null, "uncertainty_reason": "not reported by source",
                "evidence_class": "original_observation", "locator": loc(2), "derivation": det("flex.cond.pressure"),
                "resolution": "resolved", "flags": []},
            "fuel": {"raw_value": "heptane", "raw_unit": null, "value_si": null, "unit_si": null, "uncertainty_si": null,
                "uncertainty_reason": null, "evidence_class": "original_observation", "locator": loc(2),
                "derivation": det("flex.cond.fuel"), "resolution": "resolved", "flags": []}
        },
        "identity_resolution": "resolved", "identity_evidence": [], "quality_flags": [], "notes": null
    }))
    .unwrap()
}

fn obs(run: &str, i: usize, t: f64, mm: Option<f64>) -> Observation {
    serde_json::from_value(json!({
        "id": format!("{run}/DropletData/DropletDiameter_mm/{i}"), "run_id": run, "quantity": "liquid_diameter",
        "channel": "DropletData", "time_s": t, "available_at_s": t,
        "raw_value": mm, "raw_unit": "mm", "value_si": mm.map(|v| v / 1000.0), "unit_si": "m",
        "uncertainty_si": null, "calibration_id": null,
        "quality_flags": if mm.is_none() { json!(["missing"]) } else { json!([]) },
        "evidence_class": "source_derived", "locator": loc(i as i64 + 2), "derivation": det("flex.obs.DropletDiameter_mm")
    }))
    .unwrap()
}

fn endpoint(run: &str, t: f64) -> EndpointRecord {
    serde_json::from_value(json!({
        "run_id": run, "original_label": "Extinction", "original_comment": null, "allowed_causes": ["extinction"],
        "censoring": "interval", "last_confirmed_active_s": t, "first_terminal_s": t + 0.1, "intervention_s": null,
        "intervention_kind": null, "observation_state": null, "resolution": "resolved", "evidence": [loc(2)],
        "derivation": det("flex.endpoint.draft")
    }))
    .unwrap()
}

fn fixture(secret_m: f64) -> (Bundle, Cohort, Split) {
    let mut b = Bundle::new("flex");
    for (k, (r, _)) in RUNS.iter().enumerate() {
        b.records.push(record(r, 760.0 + k as f64));
        b.endpoints.push(endpoint(r, 2.0 + k as f64));
        b.observations.push(obs(r, 0, -0.5, Some(2.5)));
        b.observations.push(obs(r, 1, 0.5, None)); // missing: null + invalid, never zero
        let v = if *r == "PSI-69:R3" {
            secret_m * 1000.0
        } else {
            1.5
        };
        b.observations.push(obs(r, 2, 1.0, Some(v)));
    }
    let cohort = Cohort {
        id: "sha256:cohort".into(),
        family: Family::Flex,
        version: "1".into(),
        criteria: BTreeMap::from([("fuel".to_string(), "heptane".to_string())]),
        run_ids: RUNS.iter().map(|(r, _)| r.to_string()).collect(),
        excluded: vec![],
        source_manifest_hash: "sha256:manifests".into(),
    };
    let split = Split {
        id: "sha256:0123456789abcdef0123".into(),
        cohort_id: cohort.id.clone(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: RUNS
            .iter()
            .map(|(r, g)| (r.to_string(), g.to_string()))
            .collect(),
        train: vec!["g1".into()],
        validation: vec!["g2".into()],
        test: vec!["g3".into()],
        frozen_at: "2026-11-07T00:00:00Z".into(),
    };
    (b, cohort, split)
}

fn build(spec: &BundleSpec, secret_m: f64) -> BuiltBundle {
    let (b, c, s) = fixture(secret_m);
    dataset::build(&b, &c, &s, spec, &|_| None).unwrap()
}

fn batch(built: &BuiltBundle, view: &str) -> RecordBatch {
    let v = built.bundle.views.iter().find(|v| v.name == view).unwrap();
    let bytes = &built
        .files
        .iter()
        .find(|(a, _)| a.sha256 == v.artifact.sha256)
        .unwrap()
        .1;
    // unique per artifact: tests run in parallel
    let dir = std::env::temp_dir().join(format!(
        "ignis-dataset-{}-{}.parquet",
        std::process::id(),
        v.artifact.artifact_id
    ));
    std::fs::write(&dir, bytes).unwrap();
    let mut r = ParquetRecordBatchReaderBuilder::try_new(File::open(&dir).unwrap())
        .unwrap()
        .build()
        .unwrap();
    let b = r.next().unwrap().unwrap();
    let _ = std::fs::remove_file(dir);
    b
}

fn strings(b: &RecordBatch, col: &str) -> Vec<Option<String>> {
    b.column_by_name(col)
        .unwrap()
        .as_string::<i32>()
        .iter()
        .map(|x| x.map(String::from))
        .collect()
}

fn floats(b: &RecordBatch, col: &str) -> Vec<Option<f64>> {
    let a = b.column_by_name(col).unwrap().as_primitive::<Float64Type>();
    (0..a.len())
        .map(|i| a.is_valid(i).then(|| a.value(i)))
        .collect()
}

#[test]
fn a_builder_bundle_holds_only_its_partitions() {
    let built = build(
        &BundleSpec::builder(&[Partition::Train, Partition::Validation]),
        SECRET_M,
    );
    built.bundle.validate().unwrap();
    assert_eq!(built.bundle.audience, BundleAudience::Builder);
    assert_eq!(
        built.bundle.dataset_id.as_str(),
        "flex-0123456789abcdef-train-validation"
    );
    for view in ["runs", "conditions", "endpoints", "obs_liquid_diameter"] {
        let runs: BTreeSet<_> = strings(&batch(&built, view), "run_id")
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(
            runs,
            BTreeSet::from(["PSI-69:R1".to_string(), "PSI-69:R2".to_string()]),
            "{view}"
        );
    }
    let d = floats(&batch(&built, "obs_liquid_diameter"), "liquid_diameter_m");
    assert!(
        d.iter().flatten().all(|v| (v - SECRET_M).abs() > 1e-12),
        "a test value leaked"
    );

    // test is refused for builders, and the evaluator bundle is the one that holds it
    let (b, c, s) = fixture(SECRET_M);
    let spec = BundleSpec {
        audience: BundleAudience::Builder,
        partitions: [Partition::Train, Partition::Test].into(),
    };
    assert!(dataset::build(&b, &c, &s, &spec, &|_| None).is_err());
    let ev = build(&BundleSpec::evaluator(), SECRET_M);
    assert_eq!(ev.bundle.audience, BundleAudience::Evaluator);
    let d = floats(&batch(&ev, "obs_liquid_diameter"), "liquid_diameter_m");
    assert!(d.iter().flatten().any(|v| (v - SECRET_M).abs() < 1e-12));
    // and a bundle document claiming test for a builder fails validation
    let mut forged: DatasetBundle = built.bundle.clone();
    forged.partitions.insert(Partition::Test);
    assert!(forged.validate().is_err());
}

#[test]
fn columns_keep_units_time_and_the_original_value() {
    let built = build(&BundleSpec::builder(&[Partition::Train]), SECRET_M);
    let v = built
        .bundle
        .views
        .iter()
        .find(|v| v.name == "obs_liquid_diameter")
        .unwrap();
    let col = |n: &str| v.columns.iter().find(|c| c.name == n).unwrap();
    assert_eq!(col("time_s").role, ColumnRole::Time);
    assert_eq!(col("time_s").unit.as_deref(), Some("s"));
    assert_eq!(col("available_at_s").role, ColumnRole::AvailableAt);
    assert_eq!(
        col("liquid_diameter_m").quantity,
        Some(Quantity::LiquidDiameter)
    );
    assert_eq!(col("liquid_diameter_m").unit.as_deref(), Some("m"));
    assert!(col("liquid_diameter_m").nullable);
    assert_eq!(
        built.bundle.time_origin,
        ignis_core::views::FLEX_TIME_ORIGIN
    );

    let b = batch(&built, "obs_liquid_diameter");
    assert_eq!(strings(&b, "raw_value")[0].as_deref(), Some("2.5"));
    assert_eq!(strings(&b, "raw_unit")[0].as_deref(), Some("mm"));
    assert!((floats(&b, "liquid_diameter_m")[0].unwrap() - 0.0025).abs() < 1e-15);
    assert!(strings(&b, "locator")[0]
        .as_deref()
        .unwrap()
        .contains("DropletDiameter_mm"));
    // the missing sample is null and invalid, not zero
    assert_eq!(floats(&b, "liquid_diameter_m")[1], None);
    let valid = b.column_by_name("valid").unwrap().as_boolean().clone();
    assert!(valid.value(0) && !valid.value(1));

    // conditions: one SI column per numeric key, and the raw value next to the SI one
    let runs = built
        .bundle
        .views
        .iter()
        .find(|v| v.name == "runs")
        .unwrap();
    let p = runs
        .columns
        .iter()
        .find(|c| c.name == "pressure_si")
        .unwrap();
    assert_eq!(p.unit.as_deref(), Some("Pa"));
    let c = batch(&built, "conditions");
    let keys = strings(&c, "key");
    let i = keys
        .iter()
        .position(|k| k.as_deref() == Some("pressure"))
        .unwrap();
    assert_eq!(strings(&c, "raw_unit")[i].as_deref(), Some("mmHg"));
    assert_eq!(strings(&c, "raw_value")[i].as_deref(), Some("760"));
    assert!((floats(&c, "value_si")[i].unwrap() - 760.0 * 133.322387415).abs() < 1e-6);
    assert!(built.bundle.quality_report.is_some());
}

fn scope(role: ScopeRole, partitions: &[Partition], caps: &[Capability]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("s").unwrap(),
        workspace_id: WorkspaceId::new("w").unwrap(),
        principal: PrincipalId::new("p").unwrap(),
        role,
        partitions: partitions.iter().copied().collect(),
        capabilities: caps.iter().copied().collect(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

#[test]
fn the_service_refuses_what_the_scope_does_not_grant() {
    let root = std::env::temp_dir().join(format!("ignis-dataset-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let store = DatasetStore {
        builder_root: root.join("work/datasets"),
        evaluator_root: root.join("holdout/datasets"),
    };
    let train = build(&BundleSpec::builder(&[Partition::Train]), SECRET_M);
    let dev = build(
        &BundleSpec::builder(&[Partition::Train, Partition::Validation]),
        SECRET_M,
    );
    let ev = build(&BundleSpec::evaluator(), SECRET_M);
    for b in [&train, &dev, &ev] {
        store.save(b).unwrap();
    }
    assert!(root
        .join("holdout/datasets")
        .join(format!("{}.json", ev.bundle.dataset_id))
        .exists());
    assert!(!root
        .join("work/datasets")
        .join(format!("{}.json", ev.bundle.dataset_id))
        .exists());

    let rd = [Capability::ReadData];
    let author = scope(ScopeRole::Authoring, &[Partition::Train], &rd);
    let research = scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
        &rd,
    );
    let evaluator = scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
        &rd,
    );
    let id = |b: &BuiltBundle| b.bundle.dataset_id.clone();

    assert_eq!(store.dataset(&author, &id(&train)).unwrap(), train.bundle);
    assert_eq!(
        store.dataset(&author, &id(&dev)).unwrap_err().code,
        ErrorCode::ScopeDenied
    );
    assert_eq!(store.dataset(&research, &id(&dev)).unwrap(), dev.bundle);
    // the evaluator root is not even opened for a research scope
    assert_eq!(
        store.dataset(&research, &id(&ev)).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert_eq!(store.dataset(&evaluator, &id(&ev)).unwrap(), ev.bundle);
    // without the capability, or with a malformed scope claiming test, nothing is served
    let no_cap = scope(
        ScopeRole::Research,
        &[Partition::Train],
        &[Capability::ReadSources],
    );
    assert_eq!(
        store.dataset(&no_cap, &id(&train)).unwrap_err().code,
        ErrorCode::ScopeDenied
    );
    let forged = scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Test],
        &rd,
    );
    assert_eq!(
        store.dataset(&forged, &id(&train)).unwrap_err().code,
        ErrorCode::ScopeDenied
    );
    // view bytes go through the same checks and are verified against their digest
    assert!(store
        .view_path(&author, &id(&train), "obs_liquid_diameter")
        .is_ok());
    assert_eq!(
        store
            .view_path(&author, &id(&dev), "obs_liquid_diameter")
            .unwrap_err()
            .code,
        ErrorCode::ScopeDenied
    );
    assert_eq!(
        store
            .dataset(&research, &DatasetId::new("flex-unknown").unwrap())
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn the_content_digest_is_stable_and_tracks_values() {
    let spec = BundleSpec::builder(&[Partition::Train, Partition::Validation]);
    let (b, c, s) = fixture(SECRET_M);
    let first = dataset::build(&b, &c, &s, &spec, &|_| None).unwrap();
    // a rebuild with the stored artifacts keeps every identity (created_at included)
    let again = dataset::build(&b, &c, &s, &spec, &|d| {
        first
            .files
            .iter()
            .find(|(a, _)| a.sha256 == *d)
            .map(|(a, _)| a.clone())
    })
    .unwrap();
    assert_eq!(first.bundle.content_digest(), again.bundle.content_digest());
    // one train value changes the digest
    let (mut b2, c2, s2) = fixture(SECRET_M);
    b2.observations[0] = obs("PSI-69:R1", 0, -0.5, Some(2.6));
    let changed = dataset::build(&b2, &c2, &s2, &spec, &|_| None).unwrap();
    assert_ne!(
        first.bundle.content_digest(),
        changed.bundle.content_digest()
    );
    // a test value is not part of a builder bundle, so it cannot change its digest
    let other_secret = dataset::build(&fixture(0.0011).0, &c, &s, &spec, &|d| {
        first
            .files
            .iter()
            .find(|(a, _)| a.sha256 == *d)
            .map(|(a, _)| a.clone())
    })
    .unwrap();
    assert_eq!(
        first.bundle.content_digest(),
        other_secret.bundle.content_digest()
    );
}

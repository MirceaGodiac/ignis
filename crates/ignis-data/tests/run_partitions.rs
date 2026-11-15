//! `DatasetStore::run_partitions` (M9): the partition of a run comes from the frozen split files
//! under `IGNIS_DATA/splits`, the same ones that decide which runs `conditions` withholds. The host
//! refuses lineage through a run with an empty answer, so a run no split assigns must come back
//! empty, never as some default partition. (Own test binary: it sets `IGNIS_DATA`.)

use std::collections::{BTreeMap, BTreeSet};

use ignis_core::contracts::{
    Capability, Cohort, ErrorCode, ExecutionScope, Family, Partition, PrincipalId, RunId, ScopeId,
    ScopeRole, Split, WorkspaceId,
};
use ignis_core::hashing::content_hash;
use ignis_core::services::DatasetService;
use ignis_data::dataset::DatasetStore;
use serde_json::json;

fn scope(caps: &[Capability]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("s").unwrap(),
        workspace_id: WorkspaceId::new("w").unwrap(),
        principal: PrincipalId::new("p").unwrap(),
        role: ScopeRole::Research,
        partitions: [Partition::Train, Partition::Validation].into(),
        capabilities: caps.iter().copied().collect(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

#[test]
fn a_run_s_partitions_come_from_the_frozen_splits_and_an_unassigned_run_has_none() {
    let root = std::env::temp_dir().join(format!("ignis-run-partitions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let runs = [
        ("PSI-69:R1", "g1"),
        ("PSI-69:R2", "g2"),
        ("PSI-69:R3", "g3"),
        ("PSI-69:R9", "gx"),
    ];
    let cohort = Cohort {
        id: "sha256:cohort".into(),
        family: Family::Flex,
        version: "1".into(),
        criteria: BTreeMap::new(),
        run_ids: runs.iter().map(|(r, _)| r.to_string()).collect(),
        excluded: vec![],
        source_manifest_hash: "sha256:manifests".into(),
    };
    let split = Split {
        id: "sha256:0123456789abcdef0123".into(),
        cohort_id: cohort.id.clone(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        // R9's group is in no partition: it is not assigned one.
        run_to_group: runs
            .iter()
            .map(|(r, g)| (r.to_string(), g.to_string()))
            .collect(),
        train: vec!["g1".into()],
        validation: vec!["g2".into()],
        test: vec!["g3".into()],
        frozen_at: "2026-11-07T00:00:00Z".into(),
    };
    let ch = content_hash(&json!({"split": split, "cohort": cohort}), &[]);
    let dir = root.as_path().join("splits");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("flex-0123456789abcdef.json"),
        serde_json::to_string(&json!({"split": split, "cohort": cohort, "content_hash": ch}))
            .unwrap(),
    )
    .unwrap();
    std::env::set_var("IGNIS_DATA", root.as_path());

    let store = DatasetStore {
        builder_root: root.as_path().join("builder"),
        evaluator_root: root.as_path().join("evaluator"),
    };
    let sc = scope(&[Capability::ReadData]);
    let parts = |r: &str| store.run_partitions(&sc, &RunId::new(r).unwrap()).unwrap();
    assert_eq!(parts("PSI-69:R1"), BTreeSet::from([Partition::Train]));
    assert_eq!(parts("PSI-69:R3"), BTreeSet::from([Partition::Test]));
    assert!(parts("PSI-69:R9").is_empty(), "a group in no partition");
    assert!(parts("PSI-99:R1").is_empty(), "a run in no split");

    // The lookup itself needs read_data.
    let e = store
        .run_partitions(&scope(&[]), &RunId::new("PSI-69:R1").unwrap())
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::ScopeDenied);
    let _ = std::fs::remove_dir_all(&root);
}

//! An R2b bundle built without a split (`split_id = unsplit`, partition `train`) is withheld as soon
//! as a frozen split assigns any run of its investigation: the split's assignment wins over the
//! unsplit label. One test per binary: it sets `IGNIS_DATA` (where frozen splits are found).

use ignis_core::contracts::{
    Capability, DatasetId, ErrorCode, ExecutionScope, Partition, PrincipalId, ScopeId, ScopeRole,
    WorkspaceId,
};
use ignis_core::services::DatasetService;
use ignis_data::dataset::{unsplit_superseded, DatasetStore};

const MDCA: &str = "\"FCU TimeStamp\",\"FCU_MP_PT16(PSIA)\"
\"2016-04-18 15:00:17.500000\",14.70
";

#[test]
fn a_frozen_split_over_the_investigation_wins_over_unsplit() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    std::fs::create_dir_all(data.join("splits")).unwrap();
    std::env::set_var("IGNIS_DATA", &data);
    let raw = tmp.path().join("raw/PSI-68");
    std::fs::create_dir_all(&raw).unwrap();
    std::fs::write(raw.join("A001D01_Chamber_Readings.txt"), MDCA).unwrap();
    let store = DatasetStore {
        builder_root: tmp.path().join("datasets"),
        evaluator_root: tmp.path().join("holdout"),
    };
    let b = ignis_data::sensors::build_investigation("PSI-68", &raw, &store).unwrap();
    assert_eq!(b.bundle.split_id, "unsplit");
    let sc = ExecutionScope {
        scope_id: ScopeId::new("t").unwrap(),
        workspace_id: WorkspaceId::new("w").unwrap(),
        principal: PrincipalId::new("p").unwrap(),
        role: ScopeRole::Research,
        partitions: [Partition::Train, Partition::Validation].into(),
        capabilities: [Capability::ReadData].into(),
        corpus_snapshot: None,
        parent_scope_id: None,
    };
    let id = DatasetId::new("sensors-psi-68").unwrap();
    store.dataset(&sc, &id).unwrap();

    // A split of another investigation changes nothing.
    let other =
        serde_json::json!({"split": {"id": "flex-0000", "run_to_group": {"PSI-69:FLEX-001": "g"}}});
    std::fs::write(data.join("splits/flex-0000.json"), other.to_string()).unwrap();
    store.dataset(&sc, &id).unwrap();
    assert_eq!(store.datasets(&sc).unwrap().len(), 1);

    // A split that assigns a PSI-68 run (wherever it puts it) supersedes the unsplit bundle.
    let t10 = serde_json::json!({"split": {"id": "t10-flex2", "run_to_group": {"PSI-68:A001D01": "g1"},
                                           "train": [], "validation": [], "test": ["g1"]}});
    std::fs::write(data.join("splits/t10-flex2.json"), t10.to_string()).unwrap();
    let e = store.dataset(&sc, &id).unwrap_err();
    assert_eq!(e.code, ErrorCode::ScopeDenied);
    assert!(
        e.message.contains("t10-flex2") && e.message.contains("split's partitions win"),
        "{}",
        e.message
    );
    assert!(store.datasets(&sc).unwrap().is_empty(), "no longer listed");
    assert_eq!(
        unsplit_superseded(&b.bundle, &ignis_data::cohort::split_files()).as_deref(),
        Some("t10-flex2")
    );

    // An unreadable split file is taken as covering (fail closed).
    std::fs::remove_file(data.join("splits/t10-flex2.json")).unwrap();
    std::fs::write(data.join("splits/broken.json"), "{not json").unwrap();
    assert!(store.dataset(&sc, &id).is_err());
}

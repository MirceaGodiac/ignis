//! R6: the lockbox holds for a document newly added from a held-out investigation.
//!
//! A real FLEX (PSI-69) document from the stored corpus, NASA's FLEX Test Plan (CC0; fixture under
//! tests/fixtures/PSI-69/, as the server stores it under raw/samples/PSI-69/), is parsed by the real
//! parser *without* `--investigation`, indexed, and searched with the real lockbox built from the
//! committed frozen splits. A scope that may not read `test` must receive no passage that is
//! outcome-bearing and not attributed to readable runs; the evaluation scope still finds them.
#![cfg(feature = "pdf")]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Capability, EvidenceSchema, ExecutionScope, Partition, PrincipalId, ScopeId, ScopeRole,
    SearchFilters, SearchMode, SearchRequest, WorkspaceId,
};
use ignis_core::services::KnowledgeService;
use ignis_evidence::service::{EvidenceService, Lockbox};
use ignis_evidence::{catalog, dense, pdf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn scope(role: ScopeRole, partitions: &[Partition]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("lockbox-test").unwrap(),
        workspace_id: WorkspaceId::new("ws").unwrap(),
        principal: PrincipalId::new("tester").unwrap(),
        role,
        partitions: partitions.iter().copied().collect(),
        capabilities: [Capability::ReadSources].into_iter().collect(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn search(svc: &EvidenceService, s: &ExecutionScope, q: &str) -> BTreeSet<String> {
    let page = svc
        .search(
            s,
            &SearchRequest {
                schema_version: EvidenceSchema::V2,
                query: q.into(),
                mode: SearchMode::Lexical,
                filters: SearchFilters::default(),
                corpus_snapshot: None,
                page_size: Some(200),
                cursor: None,
                response_format: None,
            },
        )
        .unwrap();
    page.hits
        .iter()
        .map(|h| h.evidence.evidence_id.as_str().to_string())
        .collect()
}

#[test]
fn a_newly_added_flex_document_never_reaches_a_scope_without_test() {
    // The real lockbox reads the committed frozen splits under data/.
    std::env::set_var("IGNIS_DATA", root().join("data"));
    let fixture = root().join(
        "crates/ignis-evidence/tests/fixtures/PSI-69/PSI-69_Science Documents_FLEX_Test_Plan.pdf",
    );
    let rp = pdf::RunPatterns::load(&root().join("configs/evidence/run_patterns.toml")).unwrap();
    // No --investigation: the scope must come from where the file is stored.
    let parsed = pdf::parse_pdf(&fixture, &pdf::ParseOptions::default(), &rp).unwrap();
    assert_eq!(
        parsed.publication.investigation_ids,
        vec!["PSI-69".to_string()]
    );
    assert!(!parsed.chunks.is_empty());

    let dir = std::env::temp_dir().join(format!("ignis-lockbox-new-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    catalog::upsert_parse_view(
        &mut con,
        &parsed.publication,
        &parsed.chunks,
        &parsed.pages,
        None,
    )
    .unwrap();
    drop(con);
    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let mut svc = EvidenceService::open(&db, cfg, Lockbox::load().unwrap())
        .unwrap()
        .offline();
    svc.ledger = false;

    let by_id: BTreeMap<String, &ignis_evidence::records::Chunk> =
        parsed.chunks.iter().map(|c| (c.id.clone(), c)).collect();
    let research = scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    );
    let evaluation = scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    );
    let mut seen_eval = 0;
    for q in ["droplet", "test", "FLEX", "fuel", "extinction", "oxygen"] {
        let r = search(&svc, &research, q);
        for id in &r {
            let c = by_id[id];
            assert!(
                !(c.outcome_bearing && c.describes_runs.is_empty()),
                "{id}: an unattributed outcome passage of a FLEX document reached a scope without test"
            );
        }
        let e = search(&svc, &evaluation, q);
        assert!(r.is_subset(&e), "a research hit the evaluator cannot see");
        seen_eval += e.len();
    }
    assert!(seen_eval > 0, "the document is searchable at all");
    let _ = std::fs::remove_dir_all(dir);
}

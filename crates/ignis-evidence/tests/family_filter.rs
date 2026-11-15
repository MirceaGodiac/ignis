//! A family filter never hides silently: documents left out only because they carry no family
//! label are counted on the lexical backend's note (`excluded_unlabelled: N`), and the page is not
//! reported degraded. What the filter matches is unchanged.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    BackendStatus, Capability, EvidenceSchema, ExecutionScope, Family, OffsetUnit, Partition,
    PrincipalId, RetrievalBackend, ScopeId, ScopeRole, SearchFilters, SearchMode, SearchPage,
    SearchRequest, SourceLocator, Split, WorkspaceId,
};
use ignis_core::services::KnowledgeService;
use ignis_evidence::service::{EvidenceService, Lockbox};
use ignis_evidence::{catalog, dense, records};
use serde_json::json;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Publication `n` with `families` and one passage.
fn add(con: &mut rusqlite::Connection, n: u8, families: &[&str], text: &str) {
    add_run(con, n, families, text, None);
}

/// [`add`] with an outcome-bearing passage about `run` in investigation PSI-69, when given.
fn add_run(
    con: &mut rusqlite::Connection,
    n: u8,
    families: &[&str],
    text: &str,
    run: Option<&str>,
) {
    let sha = format!("{n:02x}").repeat(32);
    let publ = records::Publication {
        id: format!("pub-{}", &sha[..16]),
        source_sha256: sha.clone(),
        title: Some(format!("Synthetic document {n}")),
        authors: vec![],
        year: None,
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: if run.is_some() {
            vec!["PSI-69".into()]
        } else {
            vec![]
        },
        families: families.iter().map(|f| f.to_string()).collect(),
        file_name: Some(format!("doc{n}.pdf")),
        n_pages: 1,
        rights_status: "verified".into(),
        license: None,
        describes_runs: vec![],
        outcome_bearing: false,
        ingested_at: "2026-11-15T00:00:00Z".into(),
        parser_version: "test".into(),
    };
    let view = records::parse_view_id(&publ.id, &publ.parser_version);
    let chunk = records::Chunk {
        id: format!("{view}:p1:00000"),
        publication_id: publ.id.clone(),
        ordinal: 0,
        kind: "text".into(),
        page: 1,
        char_start: Some(0),
        char_end: Some(text.chars().count() as i64),
        section: None,
        text: text.into(),
        context: json!({}),
        locator: SourceLocator {
            asset_sha256: sha,
            page: Some(1),
            char_start: Some(0),
            char_end: Some(text.chars().count() as i64),
            parse_view_id: Some(view),
            offset_unit: Some(OffsetUnit::UnicodeScalar),
            ..Default::default()
        },
        describes_runs: run.iter().map(|r| r.to_string()).collect(),
        continues_runs: vec![],
        outcome_bearing: run.is_some(),
        evidence_class: "source_derived".into(),
    };
    catalog::upsert_parse_view(con, &publ, &[chunk], &[text.to_string()], None).unwrap();
}

/// FLEX-001 is a train run, FLEX-002 a test run.
fn lockbox() -> Lockbox {
    let split = Split {
        id: "split".into(),
        cohort_id: "cohort".into(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: BTreeMap::from([
            ("PSI-69:R1".to_string(), "g1".to_string()),
            ("PSI-69:R2".to_string(), "g2".to_string()),
        ]),
        train: vec!["g1".into()],
        validation: vec![],
        test: vec!["g2".into()],
        frozen_at: "2026-11-07T00:00:00Z".into(),
    };
    let aliases = HashMap::from([
        ("alias:flex:FLEX-001".to_string(), "PSI-69:R1".to_string()),
        ("alias:flex:FLEX-002".to_string(), "PSI-69:R2".to_string()),
    ]);
    Lockbox::new(&[(split, Family::Flex)], aliases)
}

fn search(s: &EvidenceService, families: &[Family]) -> SearchPage {
    search_as(
        s,
        families,
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    )
}

fn search_as(
    s: &EvidenceService,
    families: &[Family],
    role: ScopeRole,
    partitions: &[Partition],
) -> SearchPage {
    let sc = ExecutionScope {
        scope_id: ScopeId::new("s").unwrap(),
        workspace_id: WorkspaceId::new("w").unwrap(),
        principal: PrincipalId::new("p").unwrap(),
        role,
        partitions: partitions.iter().copied().collect(),
        capabilities: BTreeSet::from([Capability::ReadSources]),
        corpus_snapshot: None,
        parent_scope_id: None,
    };
    let req = SearchRequest {
        response_format: None,
        expected_terms: Vec::new(),
        schema_version: EvidenceSchema::V1,
        query: "opposed flow flame spread".into(),
        mode: SearchMode::Hybrid,
        filters: SearchFilters {
            families: families.to_vec(),
            ..Default::default()
        },
        corpus_snapshot: None,
        page_size: Some(10),
        cursor: None,
    };
    let p = s.search(&sc, &req).unwrap();
    p.validate().unwrap();
    p
}

fn note(p: &SearchPage) -> Option<String> {
    p.backends
        .iter()
        .find(|b| b.backend == RetrievalBackend::Lexical)
        .and_then(|b| b.detail.clone())
}

#[test]
fn a_family_filter_counts_the_unlabelled_documents_it_leaves_out() {
    let dir = std::env::temp_dir().join(format!("ignis-family-filter-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    add(
        &mut con,
        1,
        &["bass"],
        "Opposed flow flame spread over a cotton fabric in BASS.",
    );
    add(
        &mut con,
        2,
        &[],
        "Opposed flow flame spread over PMMA in SAFFIRE.",
    );
    add(
        &mut con,
        3,
        &[],
        "Opposed flow flame spread at low flow speed.",
    );
    add(&mut con, 4, &[], "Droplet combustion of heptane.");
    add(&mut con, 5, &["flex"], "Droplet combustion in FLEX.");
    // An unlabelled document whose matching passage is about a held-out (test) run.
    add_run(
        &mut con,
        6,
        &[],
        "Opposed flow flame spread result for FLEX-002.",
        Some("alias:flex:FLEX-002"),
    );
    drop(con);
    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let mut s = EvidenceService::open(&db, cfg, lockbox())
        .unwrap()
        .offline();
    s.ledger = false;

    // With the filter: the same hits as before (labelled bass documents only) and a note that two
    // unlabelled documents matching the query were left out; the held-out one (FLEX-002, test) is
    // not counted for a research scope. Not a degradation.
    let p = search(&s, &[Family::Bass]);
    assert_eq!(p.hits.len(), 1);
    assert!(!p.degraded());
    let n = note(&p).expect("a note on the lexical backend");
    assert!(n.contains("excluded_unlabelled: 2"), "{n}");
    assert!(n.contains("omit filters.families"), "{n}");
    assert_eq!(
        p.backends
            .iter()
            .find(|b| b.backend == RetrievalBackend::Lexical)
            .unwrap()
            .status,
        BackendStatus::Used
    );

    // Without the filter every matching document is searched and nothing is noted.
    let all = search(&s, &[]);
    assert_eq!(all.hits.len(), 3);
    assert!(note(&all).is_none_or(|n| !n.contains("excluded_unlabelled")));

    // Only documents that match the query are candidates (the unlabelled droplet one is not); the
    // held-out document is a candidate, but the lockbox keeps it out of the count above.
    let candidates: BTreeSet<String> = catalog::unlabelled_family_matches(
        &catalog::connect(&db, true).unwrap(),
        "opposed flow flame spread",
        &catalog::Filters {
            families: vec!["flex".into()],
            ..Default::default()
        },
    )
    .unwrap()
    .into_iter()
    .map(|m| m.publication_id)
    .collect();
    assert_eq!(candidates.len(), 3);

    // An evaluation scope may see the held-out passage, so it counts there.
    let eval = search_as(
        &s,
        &[Family::Bass],
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    );
    assert!(
        note(&eval).unwrap().contains("excluded_unlabelled: 3"),
        "{:?}",
        note(&eval)
    );
}

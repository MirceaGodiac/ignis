//! R7b: `sources_outline` through `KnowledgeService::outline` on fixture parse views (synthetic
//! text in the shape of PSI reports): headings from a PDFium text layer (numbering, capitals,
//! running headers dropped) and from Docling heading items; page spans that nest per the contract;
//! stable ids; depth and subtree selection; the lockbox hides a heading naming a held-out run; and
//! summaries are labelled generated text, attached only where nothing of the document is withheld.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Capability, ErrorCode, EvidenceSchema, ExecutionScope, Family, OffsetUnit, OutlineNode,
    OutlineNodeId, OutlineRequest, Partition, PrincipalId, ScopeId, ScopeRole, SourceId,
    SourceLocator, Split, WorkspaceId,
};
use ignis_core::services::KnowledgeService;
use ignis_evidence::service::{EvidenceService, Lockbox, SUMMARIES_FILE};
use ignis_evidence::{catalog, dense, llm_pass, outline, records};
use serde_json::json;

const PDFIUM_SHA: &str = "abababababababababababababababababababababababababababababababab";
const DOCLING_SHA: &str = "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn scope(role: ScopeRole, partitions: &[Partition]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("test-scope").unwrap(),
        workspace_id: WorkspaceId::new("ws").unwrap(),
        principal: PrincipalId::new("tester").unwrap(),
        role,
        partitions: partitions.iter().copied().collect(),
        capabilities: BTreeSet::from([Capability::ReadSources]),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

/// (page, chunk kind, text, line height, run it describes)
type Spec = (i64, &'static str, &'static str, f64, Option<&'static str>);

fn add_publication(con: &mut rusqlite::Connection, sha: &str, pages: usize, specs: &[Spec]) {
    let publ = records::Publication {
        id: format!("pub-{}", &sha[..16]),
        source_sha256: sha.into(),
        title: Some("Synthetic flight report".into()),
        authors: vec![],
        year: Some(2015),
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: vec!["PSI-69".into()],
        families: vec!["flex".into()],
        file_name: Some(format!("{}.pdf", &sha[..6])),
        n_pages: pages as i64,
        rights_status: "verified".into(),
        license: None,
        describes_runs: vec![],
        outcome_bearing: true,
        ingested_at: "2026-11-15T00:00:00Z".into(),
        parser_version: "test".into(),
    };
    let view = records::parse_view_id(&publ.id, &publ.parser_version);
    let mut page_text: Vec<String> = vec![String::new(); pages];
    let mut chunks = Vec::new();
    for (i, (page, kind, text, height, run)) in specs.iter().enumerate() {
        let t = &mut page_text[*page as usize - 1];
        let a = t.chars().count();
        t.push_str(text);
        let b = t.chars().count();
        t.push('\n');
        chunks.push(records::Chunk {
            id: format!("{view}:p{page}:{i:05}"),
            publication_id: publ.id.clone(),
            ordinal: i as i64,
            kind: (*kind).into(),
            page: *page,
            char_start: Some(a as i64),
            char_end: Some(b as i64),
            section: None,
            text: (*text).into(),
            context: json!({"lines": [{"char_start": a, "char_end": b, "bbox": [0.1, 0.1, 0.9, 0.1 + height]}]}),
            locator: SourceLocator {
                asset_sha256: sha.into(),
                path: Some("synthetic.pdf".into()),
                page: Some(*page),
                char_start: Some(a as i64),
                char_end: Some(b as i64),
                parse_view_id: Some(view.clone()),
                offset_unit: Some(OffsetUnit::UnicodeScalar),
                ..Default::default()
            },
            describes_runs: run.iter().map(|r| r.to_string()).collect(),
            continues_runs: vec![],
            // Only passages about a run bear outcomes (the attribution marks the rest otherwise).
            outcome_bearing: run.is_some(),
            evidence_class: "source_derived".into(),
        });
    }
    catalog::upsert_parse_view(con, &publ, &chunks, &page_text, None).unwrap();
}

const H: &str = "NASA/TM-2015-0001"; // a running header on every page
const BODY: f64 = 0.012;

fn pdfium() -> Vec<Spec> {
    vec![
        (1, "text", H, BODY, None),
        (1, "text", "ABSTRACT", BODY, None),
        (
            1,
            "text",
            "This report describes droplet combustion tests flown in 2015.",
            BODY,
            None,
        ),
        (1, "text", "1 Introduction", BODY, None),
        (
            1,
            "text",
            "2016 flight campaign began with eight tests.",
            BODY,
            None,
        ),
        (2, "text", H, BODY, None),
        (2, "text", "2 Experimental Setup", BODY, None),
        (2, "text", "2.1 Hardware", BODY, None),
        (
            2,
            "text",
            "The chamber held 100 L of gas at 1 atm.",
            BODY,
            None,
        ),
        (3, "text", H, BODY, None),
        (3, "text", "2.2 Procedure", BODY, None),
        (
            3,
            "text",
            "1.5 mm droplets were deployed on fibres.",
            BODY,
            None,
        ),
        (4, "text", H, BODY, None),
        (4, "text", "3 Results", BODY, None),
        (
            4,
            "text",
            "3.1 FLEX-002 Extinction",
            BODY,
            Some("alias:flex:FLEX-002"),
        ),
        (
            4,
            "text",
            "The droplet extinguished at 3 mm.",
            BODY,
            Some("alias:flex:FLEX-002"),
        ),
        (5, "text", H, BODY, None),
        (5, "text", "Large Print Summary Heading", BODY * 1.6, None),
        (5, "text", "References follow.", BODY, None),
        (3, "text", "NASA/TP-2015-0001 3", BODY, None), // a footer with its page number
        (4, "text", "NASA/TP-2015-0001 4", BODY, None),
        (5, "text", "NASA/TP-2015-0001 5", BODY, None),
    ]
}

fn docling() -> Vec<Spec> {
    vec![
        (1, "heading", "Overview", BODY, None),
        (1, "text", "1 Not a heading in a Docling view", BODY, None),
        (2, "heading", "4.1 Diagnostics", BODY, None),
        (3, "heading", "Overview", BODY, None),
    ]
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
        frozen_at: "2026-11-14T00:00:00Z".into(),
    };
    let aliases = HashMap::from([
        ("alias:flex:FLEX-001".to_string(), "PSI-69:R1".to_string()),
        ("alias:flex:FLEX-002".to_string(), "PSI-69:R2".to_string()),
    ]);
    Lockbox::new(&[(split, Family::Flex)], aliases)
}

fn setup(name: &str) -> (PathBuf, EvidenceService) {
    let dir = std::env::temp_dir().join(format!("ignis-outline-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    add_publication(&mut con, PDFIUM_SHA, 5, &pdfium());
    add_publication(&mut con, DOCLING_SHA, 3, &docling());
    drop(con);
    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let mut s = EvidenceService::open(&db, cfg, lockbox())
        .unwrap()
        .offline();
    s.ledger = false;
    (db, s)
}

fn req(sha: &str) -> OutlineRequest {
    OutlineRequest {
        schema_version: EvidenceSchema::V2,
        source_id: SourceId::new(format!("sha256:{sha}")).unwrap(),
        parse_view_id: None,
        node_id: None,
        depth: None,
    }
}

fn shape(nodes: &[OutlineNode]) -> Vec<(String, u32, u32, Vec<(String, u32, u32)>)> {
    nodes
        .iter()
        .map(|n| {
            (
                n.title.clone(),
                n.page_start,
                n.page_end,
                n.children
                    .iter()
                    .map(|c| (c.title.clone(), c.page_start, c.page_end))
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn text_layer_headings_nest_by_numbering_with_page_spans() {
    let (_, s) = setup("text");
    let eval = scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    );
    let o = s.outline(&eval, &req(PDFIUM_SHA)).unwrap();
    o.validate().unwrap();
    let t = |x: &str| x.to_string();
    assert_eq!(
        shape(&o.nodes),
        vec![
            (t("ABSTRACT"), 1, 1, vec![]),
            (t("1 Introduction"), 1, 2, vec![]),
            (
                t("2 Experimental Setup"),
                2,
                4,
                vec![(t("2.1 Hardware"), 2, 3), (t("2.2 Procedure"), 3, 4)]
            ),
            (
                t("3 Results"),
                4,
                5,
                vec![(t("3.1 FLEX-002 Extinction"), 4, 5)]
            ),
            (t("Large Print Summary Heading"), 5, 5, vec![]),
        ],
        "running header, sentences, a year and a measurement are not headings"
    );
    // Ids are the page and a hash of the title: stable across rebuilds.
    assert!(o.nodes[1].node_id.as_str().starts_with("s1-"));
    let again = s.outline(&eval, &req(PDFIUM_SHA)).unwrap();
    assert_eq!(again, o);

    // Depth 1: top-level sections, cut children flagged; a subtree by node id.
    let mut r = req(PDFIUM_SHA);
    r.depth = Some(1);
    let top = s.outline(&eval, &r).unwrap();
    assert_eq!(top.nodes.len(), 5);
    assert!(top.nodes[2].children.is_empty() && top.nodes[2].children_omitted);
    assert!(
        !top.nodes[0].children_omitted,
        "no children, nothing omitted"
    );
    let mut r = req(PDFIUM_SHA);
    r.node_id = Some(o.nodes[2].node_id.clone());
    let sub = s.outline(&eval, &r).unwrap();
    assert_eq!(sub.nodes.len(), 1);
    assert_eq!(sub.nodes[0].children.len(), 2);
    r.node_id = Some(OutlineNodeId::new("s9-00000000").unwrap());
    assert_eq!(s.outline(&eval, &r).unwrap_err().code, ErrorCode::NotFound);
}

#[test]
fn the_lockbox_hides_a_heading_that_names_a_held_out_run() {
    let (_, s) = setup("lockbox");
    let research = scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    );
    let o = s.outline(&research, &req(PDFIUM_SHA)).unwrap();
    let titles: Vec<String> = o
        .nodes
        .iter()
        .flat_map(|n| {
            std::iter::once(n.title.clone()).chain(n.children.iter().map(|c| c.title.clone()))
        })
        .collect();
    assert!(titles.contains(&"3 Results".to_string()));
    assert!(
        !titles.iter().any(|t| t.contains("FLEX-002")),
        "a held-out run's heading is withheld: {titles:?}"
    );
}

#[test]
fn docling_headings_are_used_as_marked_and_repeated_titles_get_distinct_ids() {
    let (_, s) = setup("docling");
    let research = scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    );
    let o = s.outline(&research, &req(DOCLING_SHA)).unwrap();
    o.validate().unwrap();
    let titles: Vec<&str> = o.nodes.iter().map(|n| n.title.as_str()).collect();
    assert_eq!(
        titles,
        vec!["Overview", "Overview"],
        "a numbered text item is not a Docling heading"
    );
    assert_eq!(o.nodes[0].children[0].title, "4.1 Diagnostics");
    assert_ne!(o.nodes[0].node_id, o.nodes[1].node_id);

    // A document without headings is one section over every page.
    let (heads, method) = outline::headings(&[]);
    assert!(heads.is_empty());
    assert_eq!(method.as_str(), "whole_document");
    let whole = outline::build(&heads, 7, "Report").unwrap();
    assert_eq!((whole[0].page_start, whole[0].page_end), (1, 7));
}

#[test]
fn summaries_are_generated_text_and_follow_the_lockbox() {
    let (db, s) = setup("summaries");
    let eval = scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    );
    let research = scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    );
    // The requests the pass would send (leaves only), with an estimate; nothing is called.
    let con = catalog::connect(&db, true).unwrap();
    let pubs = catalog::publications(&con).unwrap();
    let reqs = outline::summary_requests(&con, &pubs, true).unwrap();
    assert!(reqs.iter().all(|r| r.system == outline::SUMMARY_SYSTEM));
    let est = llm_pass::estimate(&reqs, 1.0, 5.0);
    assert_eq!(est.requests, reqs.len());
    assert!(
        est.usd > 0.0
            && est.output_tokens == reqs.len() as u64 * outline::SUMMARY_MAX_TOKENS as u64
    );

    // Answers of a fake model, as `--execute` would append them.
    struct Fake;
    impl llm_pass::Model for Fake {
        fn id(&self) -> &str {
            "fake-model"
        }
        fn call(&self, _: &str, user: &str, _: u32) -> anyhow::Result<String> {
            Ok(format!(
                "Covers {}.",
                user.lines().nth(1).unwrap_or_default()
            ))
        }
    }
    let out = db.with_file_name(SUMMARIES_FILE);
    let r = llm_pass::run(&reqs, &Fake, &out, 3).unwrap();
    assert_eq!(r.answered, reqs.len());
    let r2 = llm_pass::run(&reqs, &Fake, &out, 3).unwrap();
    assert_eq!(
        (r2.answered, r2.skipped_done),
        (0, reqs.len()),
        "a rerun resumes, paying nothing twice"
    );

    // A new service reads the answers: the Docling document (nothing withheld) shows summaries
    // labelled with their model; the PDFium one, with a held-out passage, shows none to research.
    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let mut s2 = EvidenceService::open(&db, cfg, lockbox())
        .unwrap()
        .offline();
    s2.ledger = false;
    let d = s2.outline(&research, &req(DOCLING_SHA)).unwrap();
    let leaf = &d.nodes[0].children[0];
    let sum = leaf.summary.as_ref().expect("leaf summary attached");
    assert_eq!(sum.model_id, "fake-model");
    assert!(sum.prompt_hash.is_some());
    let p = s2.outline(&research, &req(PDFIUM_SHA)).unwrap();
    assert!(p
        .nodes
        .iter()
        .all(|n| n.summary.is_none() && n.children.iter().all(|c| c.summary.is_none())));
    let p = s2.outline(&eval, &req(PDFIUM_SHA)).unwrap();
    assert!(
        p.nodes[0].summary.is_some(),
        "an evaluation scope withholds nothing"
    );
    drop(s);
}

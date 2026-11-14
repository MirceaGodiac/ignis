//! R7a: Docling parse views. The fixture is the real sidecar output (docling 2.150.1) for NASA's
//! FLEX Test Plan (PSI-69, CC0), next to the PDF it was made from.
//!
//! Guarded: locators are citable (spans index the stored page text, boxes normalized top-left on the
//! physical page), table rows carry their table and cells, a view is refused for other bytes or with
//! failed pages, the lockbox holds on the new view, and the MuPDF view of the same PDF stays
//! resolvable after the Docling view becomes active.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Capability, EvidenceSchema, ExecutionScope, Partition, PrincipalId, ScopeId, ScopeRole,
    SearchFilters, SearchMode, SearchRequest, WorkspaceId,
};
use ignis_core::services::KnowledgeService;
use ignis_evidence::attribution::RunPatterns;
use ignis_evidence::docling_view::{import, DoclingView};
use ignis_evidence::service::{EvidenceService, Lockbox};
use ignis_evidence::{catalog, dense};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn fixture(name: &str) -> PathBuf {
    root()
        .join("crates/ignis-evidence/tests/fixtures/PSI-69")
        .join(name)
}

const PDF: &str = "PSI-69_Science Documents_FLEX_Test_Plan.pdf";
const VIEW: &str = "PSI-69_Science Documents_FLEX_Test_Plan.docling.json";

fn view() -> DoclingView {
    serde_json::from_str(&std::fs::read_to_string(fixture(VIEW)).unwrap()).unwrap()
}

fn rp() -> RunPatterns {
    RunPatterns::load(&root().join("configs/evidence/run_patterns.toml")).unwrap()
}

fn scope(role: ScopeRole, partitions: &[Partition]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("docling-view-test").unwrap(),
        workspace_id: WorkspaceId::new("ws").unwrap(),
        principal: PrincipalId::new("tester").unwrap(),
        role,
        partitions: partitions.iter().copied().collect(),
        capabilities: [Capability::ReadSources].into_iter().collect(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

#[test]
fn a_docling_view_imports_as_citable_chunks() {
    let bytes = std::fs::read(fixture(PDF)).unwrap();
    let iv = import(&view(), &fixture(PDF), &bytes, &rp(), &[]).unwrap();
    assert_eq!(iv.publication.investigation_ids, vec!["PSI-69".to_string()]);
    assert!(iv.publication.parser_version.starts_with("docling-"));
    assert_eq!(iv.pages.len() as i64, iv.publication.n_pages);
    let mut kinds = BTreeSet::new();
    for c in &iv.chunks {
        c.locator.validate().unwrap();
        let [x0, y0, x1, y1] = c.locator.bbox.expect("every chunk has a box");
        assert!((0.0..=1.0).contains(&x0) && x0 <= x1 && x1 <= 1.0 && y0 <= y1 && y1 <= 1.0);
        // the span indexes the stored page text exactly
        let span: String = iv.pages[c.page as usize - 1]
            .chars()
            .skip(c.char_start.unwrap() as usize)
            .take((c.char_end.unwrap() - c.char_start.unwrap()) as usize)
            .collect();
        assert_eq!(span, c.text);
        kinds.insert(c.kind.clone());
        if c.kind == "table_row" {
            assert!(c
                .locator
                .table_id
                .as_deref()
                .is_some_and(|t| t.starts_with('t')));
            let cells = c.context["cells"].as_array().unwrap();
            assert!(!cells.is_empty());
            for cell in cells {
                let b: Vec<f64> = serde_json::from_value(cell["bbox"].clone()).unwrap();
                assert!(b.iter().all(|v| (0.0..=1.0).contains(v)));
            }
        }
    }
    // The FLEX test matrix is tables: rows are first-class, citable chunks.
    assert!(kinds.contains("table_row") && kinds.contains("text") && kinds.contains("heading"));
    assert!(iv.chunks.iter().filter(|c| c.kind == "table_row").count() > 100);
}

#[test]
fn a_view_is_refused_for_other_bytes_or_with_failed_pages() {
    let bytes = std::fs::read(fixture(PDF)).unwrap();
    let mut other = bytes.clone();
    *other.last_mut().unwrap() ^= 1;
    assert!(import(&view(), &fixture(PDF), &other, &rp(), &[]).is_err());
    let mut v = view();
    v.failed_pages = vec![3];
    let e = import(&v, &fixture(PDF), &bytes, &rp(), &[]).err().unwrap();
    assert!(e.to_string().contains("keeps its current view"), "{e}");
    let mut v = view();
    v.items[0].bbox = [0.2, 0.1, 1.4, 0.3];
    assert!(import(&v, &fixture(PDF), &bytes, &rp(), &[]).is_err());
}

fn search(svc: &EvidenceService, s: &ExecutionScope, q: &str) -> Vec<String> {
    svc.search(
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
    .unwrap()
    .hits
    .iter()
    .map(|h| h.evidence.evidence_id.as_str().to_string())
    .collect()
}

#[test]
fn the_docling_view_becomes_active_and_keeps_the_lockbox() {
    std::env::set_var("IGNIS_DATA", root().join("data"));
    let dir = std::env::temp_dir().join(format!("ignis-docling-view-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("evidence.sqlite");
    let bytes = std::fs::read(fixture(PDF)).unwrap();
    let iv = import(&view(), &fixture(PDF), &bytes, &rp(), &[]).unwrap();
    {
        let mut con = catalog::connect(&db, false).unwrap();
        catalog::upsert_parse_view(&mut con, &iv.publication, &iv.chunks, &iv.pages, None).unwrap();
    }
    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let mut svc = EvidenceService::open(&db, cfg, Lockbox::load().unwrap())
        .unwrap()
        .offline();
    svc.ledger = false;
    let research = scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    );
    let evaluation = scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    );
    let by_id: std::collections::BTreeMap<&str, &ignis_evidence::records::Chunk> =
        iv.chunks.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut seen = 0;
    for q in ["droplet", "test", "FLEX", "fuel", "oxygen", "pressure"] {
        let r = search(&svc, &research, q);
        for id in &r {
            let c = by_id[id.as_str()];
            assert!(
                !(c.outcome_bearing && c.describes_runs.is_empty()),
                "{id}: unattributed outcome passage reached a scope without test"
            );
        }
        seen += search(&svc, &evaluation, q).len();
    }
    assert!(seen > 0);
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(feature = "pdf")]
#[test]
fn the_mupdf_view_stays_resolvable_after_the_docling_view_is_active() {
    use ignis_evidence::pdf;
    let dir = std::env::temp_dir().join(format!("ignis-docling-both-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("evidence.sqlite");
    let mu = pdf::parse_pdf(&fixture(PDF), &pdf::ParseOptions::default(), &rp()).unwrap();
    let bytes = std::fs::read(fixture(PDF)).unwrap();
    let dv = import(&view(), &fixture(PDF), &bytes, &rp(), &[]).unwrap();
    assert_eq!(
        mu.publication.id, dv.publication.id,
        "same PDF, same publication"
    );
    {
        let mut con = catalog::connect(&db, false).unwrap();
        catalog::upsert_parse_view(&mut con, &mu.publication, &mu.chunks, &mu.pages, None).unwrap();
        catalog::upsert_parse_view(&mut con, &dv.publication, &dv.chunks, &dv.pages, None).unwrap();
        assert_eq!(catalog::views(&con, &dv.publication.id).unwrap().len(), 2);
    }
    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let mut svc = EvidenceService::open(&db, cfg, Lockbox::default())
        .unwrap()
        .offline();
    svc.ledger = false;
    let s = scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    );
    // search sees the Docling view only
    let hits = search(&svc, &s, "droplet");
    assert!(!hits.is_empty());
    let dv_view = format!(
        "{}:",
        ignis_evidence::records::parse_view_id(&dv.publication.id, &dv.publication.parser_version)
    );
    assert!(hits.iter().all(|h| h.starts_with(&dv_view)), "{hits:?}");
    // every MuPDF citation still resolves to its own text
    for c in mu.chunks.iter().take(20) {
        let e = svc
            .resolve_evidence(
                &s,
                &ignis_core::contracts::EvidenceId::new(c.id.clone()).unwrap(),
            )
            .unwrap();
        assert_eq!(e.quote.as_deref(), Some(c.text.as_str()));
    }
    let _ = std::fs::remove_dir_all(dir);
}

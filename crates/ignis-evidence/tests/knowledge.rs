//! KnowledgeService invariants (R1/R3): a scope without `test` never receives test outcomes (search,
//! read, resolve); pages partition one deterministic order with continuing ranks and an opaque
//! cursor bound to its search; a missing backend is reported, never silent; reads say how to
//! continue; and (with `pdf`) a parse view re-derives exactly from the original bytes.
//! No network: services run offline, so dense and rerank are declared unavailable.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    BackendStatus, Capability, CorpusSnapshotId, ErrorCode, EvidenceId, EvidenceSchema,
    ExecutionScope, Family, OffsetUnit, PageCursor, Partition, PrincipalId, ReadRange, ReadRequest,
    RetrievalBackend, ScopeId, ScopeRole, SearchFilters, SearchMode, SearchRequest, SourceId,
    SourceLocator, Split, WorkspaceId,
};
use ignis_core::services::KnowledgeService;
use ignis_evidence::service::{EvidenceService, Lockbox};
use ignis_evidence::{catalog, dense, records};
use serde_json::json;

const SECRET: &str = "SECRET-TEST-OUTCOME";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ignis-knowledge-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn cfg() -> dense::RetrievalCfg {
    dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap()
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

fn research() -> ExecutionScope {
    scope(
        ScopeRole::Research,
        &[Partition::Train, Partition::Validation],
    )
}

fn evaluation() -> ExecutionScope {
    scope(
        ScopeRole::Evaluation,
        &[Partition::Train, Partition::Validation, Partition::Test],
    )
}

struct Spec<'a> {
    page: i64,
    text: &'a str,
    outcome: bool,
    runs: &'a [&'a str],
    continues: &'a [&'a str],
}

const SHA: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";

/// A catalog with one FLEX publication whose chunks are laid out on `pages` exactly as the parser
/// would: each chunk is `page_text[start..end]` of its page in parse view `pv-<sha16>@test`.
fn catalog_with(dir: &Path, specs: &[Spec], pages: usize) -> PathBuf {
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    let publ = records::Publication {
        id: format!("pub-{}", &SHA[..16]),
        source_sha256: SHA.into(),
        title: Some("Synthetic FLEX report".into()),
        authors: vec![],
        year: Some(2015),
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: vec!["PSI-69".into()],
        families: vec!["flex".into()],
        file_name: Some("synthetic.pdf".into()),
        n_pages: pages as i64,
        rights_status: "verified".into(),
        license: None,
        describes_runs: vec![],
        outcome_bearing: true,
        ingested_at: "2026-11-13T00:00:00Z".into(),
        parser_version: "test".into(),
    };
    let view = records::parse_view_id(&publ.id, &publ.parser_version);
    let mut page_text: Vec<String> = vec![String::new(); pages];
    let mut chunks = Vec::new();
    for (i, s) in specs.iter().enumerate() {
        let t = &mut page_text[s.page as usize - 1];
        let a = t.chars().count();
        t.push_str(s.text);
        let b = t.chars().count();
        t.push('\n');
        chunks.push(records::Chunk {
            id: format!("{view}:p{}:{i:05}", s.page),
            publication_id: publ.id.clone(),
            ordinal: i as i64,
            kind: "text".into(),
            page: s.page,
            char_start: Some(a as i64),
            char_end: Some(b as i64),
            section: None,
            text: s.text.into(),
            context: json!({}),
            locator: SourceLocator {
                asset_sha256: SHA.into(),
                path: Some("synthetic.pdf".into()),
                page: Some(s.page),
                char_start: Some(a as i64),
                char_end: Some(b as i64),
                parse_view_id: Some(view.clone()),
                offset_unit: Some(OffsetUnit::UnicodeScalar),
                bbox: Some([0.1, 0.1, 0.9, 0.2]),
                ..Default::default()
            },
            describes_runs: s.runs.iter().map(|x| x.to_string()).collect(),
            continues_runs: s.continues.iter().map(|x| x.to_string()).collect(),
            outcome_bearing: s.outcome,
            evidence_class: "source_derived".into(),
        });
    }
    catalog::upsert_parse_view(&mut con, &publ, &chunks, &page_text, None).unwrap();
    db
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

fn service(db: &Path, cfg: dense::RetrievalCfg) -> EvidenceService {
    let mut s = EvidenceService::open(db, cfg, lockbox()).unwrap().offline();
    s.ledger = false;
    s
}

fn req(query: &str, mode: SearchMode, page_size: u32, cursor: Option<PageCursor>) -> SearchRequest {
    SearchRequest {
        response_format: None,
        expected_terms: Vec::new(),
        schema_version: EvidenceSchema::V1,
        query: query.into(),
        mode,
        filters: SearchFilters::default(),
        corpus_snapshot: None,
        page_size: Some(page_size),
        cursor,
    }
}

fn lockbox_specs() -> Vec<Spec<'static>> {
    vec![
        Spec {
            page: 1,
            text: "Test FLEX-001 extinction at 2.1 mm.",
            outcome: true,
            runs: &["alias:flex:FLEX-001"],
            continues: &[],
        },
        Spec {
            page: 1,
            text: "Test FLEX-002 extinction SECRET-TEST-OUTCOME.",
            outcome: true,
            runs: &["alias:flex:FLEX-002"],
            continues: &[],
        },
        Spec {
            page: 2,
            text: "Across tests the extinction SECRET-TEST-OUTCOME trend.",
            outcome: true,
            runs: &[],
            continues: &[],
        },
        Spec {
            page: 2,
            text: "FLEX-001 row continues extinction SECRET-TEST-OUTCOME.",
            outcome: true,
            runs: &["alias:flex:FLEX-001"],
            continues: &["alias:flex:FLEX-002"],
        },
        Spec {
            page: 3,
            text: "Apparatus for FLEX-002 extinction tests.",
            outcome: false,
            runs: &["alias:flex:FLEX-002"],
            continues: &[],
        },
    ]
}

#[test]
fn a_scope_without_test_never_receives_test_outcomes() {
    let dir = tmp("lockbox");
    let db = catalog_with(&dir, &lockbox_specs(), 3);
    let svc = service(&db, cfg());
    let ids = |page: &ignis_core::contracts::SearchPage| -> BTreeSet<String> {
        page.hits
            .iter()
            .map(|h| h.evidence.evidence_id.as_str().to_string())
            .collect()
    };
    let suffix = |i: usize, page: i64| format!("pv-{}@test:p{page}:{i:05}", &SHA[..16]);

    // research scope: the train-run passage and the apparatus passage only
    let page = svc
        .search(
            &research(),
            &req("extinction", SearchMode::Lexical, 50, None),
        )
        .unwrap();
    page.validate().unwrap();
    assert_eq!(ids(&page), BTreeSet::from([suffix(0, 1), suffix(4, 3)]));
    assert!(!serde_json::to_string(&page).unwrap().contains(SECRET));
    // the scope filter is declared, without counting what it withheld
    let lex = page
        .backends
        .iter()
        .find(|b| b.backend == RetrievalBackend::Lexical)
        .unwrap();
    assert!(lex.detail.as_deref().unwrap().contains("lockbox"));

    // reading and resolving a withheld passage is refused
    let source = SourceId::new(format!("sha256:{SHA}")).unwrap();
    for (i, p) in [(1, 1), (2, 2), (3, 2)] {
        let e = EvidenceId::new(suffix(i, p)).unwrap();
        assert_eq!(
            svc.resolve_evidence(&research(), &e).unwrap_err().code,
            ErrorCode::ScopeDenied
        );
        let r = ReadRequest {
            response_format: None,
            schema_version: EvidenceSchema::V1,
            source_id: source.clone(),
            parse_view_id: None,
            range: ReadRange::Passage { evidence_id: e },
        };
        assert_eq!(
            svc.read(&research(), &r).unwrap_err().code,
            ErrorCode::ScopeDenied
        );
    }
    // a page with withheld passages comes back passage by passage, without them
    let pages = ReadRequest {
        response_format: None,
        schema_version: EvidenceSchema::V1,
        source_id: source.clone(),
        parse_view_id: None,
        range: ReadRange::Pages { first: 1, last: 3 },
    };
    let read = svc.read(&research(), &pages).unwrap();
    read.validate().unwrap();
    assert!(read.complete);
    assert!(!serde_json::to_string(&read).unwrap().contains(SECRET));
    assert!(read
        .segments
        .iter()
        .any(|s| s.text.contains("FLEX-001 extinction")));
    // a neighbour of a readable passage is not leaked through the passage context either
    let ctx = svc
        .read(
            &research(),
            &ReadRequest {
                response_format: None,
                range: ReadRange::Passage {
                    evidence_id: EvidenceId::new(suffix(0, 1)).unwrap(),
                },
                ..pages.clone()
            },
        )
        .unwrap();
    assert!(!serde_json::to_string(&ctx).unwrap().contains(SECRET));

    // an evaluation scope that grants the test partition reads everything
    let all = svc
        .search(
            &evaluation(),
            &req("extinction", SearchMode::Lexical, 50, None),
        )
        .unwrap();
    assert_eq!(all.hits.len(), 5);
    let quote = svc
        .resolve_evidence(&evaluation(), &EvidenceId::new(suffix(1, 1)).unwrap())
        .unwrap();
    assert!(quote.quote.unwrap().contains(SECRET));

    // a research scope that claims the test partition is malformed and gets nothing
    let forged = scope(ScopeRole::Research, &[Partition::Train, Partition::Test]);
    assert_eq!(
        svc.search(&forged, &req("extinction", SearchMode::Lexical, 5, None))
            .unwrap_err()
            .code,
        ErrorCode::ScopeDenied
    );
    let _ = std::fs::remove_dir_all(dir);
}

fn many_specs(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| {
            format!(
                "Apparatus note {i}: droplet deployment needle {}",
                "droplet ".repeat(i % 7)
            )
        })
        .collect()
}

fn walk(
    svc: &EvidenceService,
    mode: SearchMode,
    page_size: u32,
) -> (
    Vec<u32>,
    Vec<String>,
    Vec<ignis_core::contracts::SearchPage>,
) {
    let mut ranks = Vec::new();
    let mut ids = Vec::new();
    let mut pages = Vec::new();
    let mut cursor = None;
    loop {
        let p = svc
            .search(&research(), &req("droplet", mode, page_size, cursor.take()))
            .unwrap();
        p.validate().unwrap();
        ranks.extend(p.hits.iter().map(|h| h.rank));
        ids.extend(
            p.hits
                .iter()
                .map(|h| h.evidence.evidence_id.as_str().to_string()),
        );
        cursor = p.next_cursor.clone();
        pages.push(p);
        if cursor.is_none() {
            break;
        }
    }
    (ranks, ids, pages)
}

#[test]
fn pages_partition_one_order_without_a_result_cap() {
    let texts = many_specs(23);
    let specs: Vec<Spec> = texts
        .iter()
        .map(|t| Spec {
            page: 1,
            text: t,
            outcome: false,
            runs: &[],
            continues: &[],
        })
        .collect();
    let dir = tmp("paging");
    let db = catalog_with(&dir, &specs, 1);

    // every match fits the fused depth: lexical mode counts exhaustively
    let svc = service(&db, cfg());
    let (ranks, ids, pages) = walk(&svc, SearchMode::Lexical, 4);
    assert_eq!(ranks, (1..=23).collect::<Vec<u32>>());
    assert_eq!(ids.iter().collect::<BTreeSet<_>>().len(), 23);
    assert_eq!(pages.len(), 6);
    assert!(pages.iter().all(|p| p.total_count == Some(23)));

    // a fused depth of 5 does not cap the results: the lexical tail continues the same order
    let mut small = cfg();
    small.fusion.lexical_depth = 5;
    let svc5 = service(&db, small);
    let (ranks5, ids5, pages5) = walk(&svc5, SearchMode::Lexical, 4);
    assert_eq!(ranks5, (1..=23).collect::<Vec<u32>>());
    assert_eq!(ids5, ids, "the tail continues the BM25 order");
    assert!(
        pages5.iter().all(|p| p.total_count.is_none()),
        "not exhaustive past the fused depth"
    );

    // the cursor is bound to its search and to the corpus snapshot
    let c = pages[0].next_cursor.clone().unwrap();
    let other = svc
        .search(
            &research(),
            &req("needle", SearchMode::Lexical, 4, Some(c.clone())),
        )
        .unwrap_err();
    assert_eq!(other.code, ErrorCode::InvalidRequest);
    let bogus = svc
        .search(
            &research(),
            &req(
                "droplet",
                SearchMode::Lexical,
                4,
                Some(PageCursor::new("c1.zz").unwrap()),
            ),
        )
        .unwrap_err();
    assert_eq!(bogus.code, ErrorCode::InvalidRequest);
    let mut pinned = req("droplet", SearchMode::Lexical, 4, None);
    pinned.corpus_snapshot = Some(CorpusSnapshotId::new("corpus-0000000000000000").unwrap());
    assert_eq!(
        svc.search(&research(), &pinned).unwrap_err().code,
        ErrorCode::Conflict
    );
    pinned.corpus_snapshot = Some(pages[0].corpus_snapshot.clone());
    assert!(svc.search(&research(), &pinned).is_ok());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_missing_backend_is_reported_never_silent() {
    let texts = many_specs(3);
    let specs: Vec<Spec> = texts
        .iter()
        .map(|t| Spec {
            page: 1,
            text: t,
            outcome: false,
            runs: &[],
            continues: &[],
        })
        .collect();
    let dir = tmp("degraded");
    let db = catalog_with(&dir, &specs, 1);
    let svc = service(&db, cfg());
    let p = svc
        .search(&research(), &req("droplet", SearchMode::Hybrid, 10, None))
        .unwrap();
    p.validate().unwrap();
    assert!(p.degraded());
    let status = |b| p.backends.iter().find(|x| x.backend == b).unwrap();
    assert_eq!(
        status(RetrievalBackend::Lexical).status,
        BackendStatus::Used
    );
    assert_eq!(
        status(RetrievalBackend::Dense).status,
        BackendStatus::Unavailable
    );
    assert_eq!(
        status(RetrievalBackend::Rerank).status,
        BackendStatus::Unavailable
    );
    assert!(
        p.total_count.is_none(),
        "hybrid ranking is never an exhaustive count"
    );
    assert_eq!(p.hits.len(), 3, "lexical still answers");
    // a lexical request does not report the other backends as degraded
    let l = svc
        .search(&research(), &req("droplet", SearchMode::Lexical, 10, None))
        .unwrap();
    assert!(!l.degraded());
    assert_eq!(
        l.backends
            .iter()
            .find(|x| x.backend == RetrievalBackend::Dense)
            .unwrap()
            .status,
        BackendStatus::NotRequested
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn reads_return_exact_page_text_and_say_how_to_continue() {
    let big: Vec<String> = (0..3)
        .map(|i| format!("page{i} ") + &"x".repeat(25_000))
        .collect();
    let specs: Vec<Spec> = big
        .iter()
        .enumerate()
        .map(|(i, t)| Spec {
            page: i as i64 + 1,
            text: t,
            outcome: false,
            runs: &[],
            continues: &[],
        })
        .collect();
    let dir = tmp("read");
    let db = catalog_with(&dir, &specs, 3);
    let svc = service(&db, cfg());
    let source = SourceId::new(format!("sha256:{SHA}")).unwrap();
    let r = svc
        .read(
            &research(),
            &ReadRequest {
                response_format: None,
                schema_version: EvidenceSchema::V1,
                source_id: source.clone(),
                parse_view_id: None,
                range: ReadRange::Pages { first: 1, last: 3 },
            },
        )
        .unwrap();
    r.validate().unwrap();
    assert!(!r.complete);
    assert_eq!(r.next, Some(ReadRange::Pages { first: 3, last: 3 }));
    assert_eq!(r.segments.len(), 2);
    for s in &r.segments {
        s.locator.validate().unwrap();
        assert_eq!(s.locator.char_start, Some(0));
        assert_eq!(s.locator.char_end, Some(s.text.chars().count() as i64));
        assert_eq!(
            s.locator.parse_view_id.as_deref(),
            Some(r.parse_view_id.as_str())
        );
    }
    // another parse view of the same bytes is not silently substituted
    let stale = svc.read(
        &research(),
        &ReadRequest {
            response_format: None,
            schema_version: EvidenceSchema::V1,
            source_id: source,
            parse_view_id: Some(ignis_core::contracts::ParseViewId::new("pv-x@0.1").unwrap()),
            range: ReadRange::Pages { first: 1, last: 1 },
        },
    );
    assert_eq!(stale.unwrap_err().code, ErrorCode::NotFound);
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(feature = "pdf")]
mod pdf_views {
    use super::*;
    use ignis_evidence::{pdf, verify};

    /// A minimal valid PDF (Helvetica, one text line per page) with a correct xref table.
    fn tiny_pdf(pages: &[&str]) -> Vec<u8> {
        let n = pages.len();
        let mut objs: Vec<String> = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".into(),
            format!(
                "<< /Type /Pages /Kids [{}] /Count {n} >>",
                (0..n)
                    .map(|i| format!("{} 0 R", 4 + 2 * i))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
        ];
        for (i, t) in pages.iter().enumerate() {
            objs.push(format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
                5 + 2 * i
            ));
            let s = format!("BT /F1 12 Tf 72 700 Td ({t}) Tj ET");
            objs.push(format!("<< /Length {} >>\nstream\n{s}\nendstream", s.len()));
        }
        let mut out = b"%PDF-1.4\n".to_vec();
        let mut offs = Vec::new();
        for (i, o) in objs.iter().enumerate() {
            offs.push(out.len());
            out.extend(format!("{} 0 obj\n{o}\nendobj\n", i + 1).bytes());
        }
        let xref = out.len();
        out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).bytes());
        for o in offs {
            out.extend(format!("{o:010} 00000 n \n").bytes());
        }
        out.extend(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                objs.len() + 1
            )
            .bytes(),
        );
        out
    }

    #[test]
    fn a_parse_view_re_derives_from_the_original_bytes() {
        let dir = tmp("pdfview");
        let path = dir.join("tiny.pdf");
        let bytes = tiny_pdf(&[
            "Methods: droplet deployment.",
            "Results: Test FLEX-002 extinction at 1.2 mm.",
        ]);
        std::fs::write(&path, &bytes).unwrap();
        let rp =
            pdf::RunPatterns::load(&root().join("configs/evidence/run_patterns.toml")).unwrap();
        let parsed = pdf::parse_pdf(&path, &pdf::ParseOptions::default(), &rp).unwrap();
        assert_eq!(parsed.pages.len(), 2);
        assert!(!parsed.chunks.is_empty());
        for c in &parsed.chunks {
            c.locator.validate().unwrap();
            let [x0, y0, x1, y1] = c.locator.bbox.expect("every text chunk has a bbox");
            assert!(
                x0 < x1 && y0 < y1 && y0 < 0.2,
                "text near the top of the page: {:?}",
                c.locator.bbox
            );
            let span: String = parsed.pages[c.page as usize - 1]
                .chars()
                .skip(c.char_start.unwrap() as usize)
                .take((c.char_end.unwrap() - c.char_start.unwrap()) as usize)
                .collect();
            assert_eq!(span, c.text);
        }
        assert!(parsed
            .chunks
            .iter()
            .any(|c| c.describes_runs == vec!["alias:flex:FLEX-002".to_string()]));

        let db = dir.join("evidence.sqlite");
        let mut con = catalog::connect(&db, false).unwrap();
        let p = &parsed.publication;
        catalog::upsert_parse_view(&mut con, p, &parsed.chunks, &parsed.pages, None).unwrap();
        let ok = verify::verify_view(&con, p, &bytes).unwrap();
        assert!(ok.passed(), "{ok:?}");
        assert_eq!(ok.chunks_with_bbox, parsed.chunks.len());

        // other bytes, or an edited stored text, fail verification
        let mut other = bytes.clone();
        let at = other.len() - 10;
        other[at] ^= 1;
        assert!(!verify::verify_view(&con, p, &other).unwrap().bytes_match);
        con.execute("UPDATE pages SET text = text || 'x' WHERE page = 2", [])
            .unwrap();
        con.execute("UPDATE chunks SET text = 'edited' WHERE page = 2", [])
            .unwrap();
        let bad = verify::verify_view(&con, p, &bytes).unwrap();
        assert_eq!(bad.page_mismatches, vec![2]);
        assert!(!bad.chunk_mismatches.is_empty() && !bad.passed());
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn the_snapshot_id_changes_with_anything_that_changes_an_answer() {
    let dir = tmp("snapshot");
    let db = catalog_with(&dir, &lockbox_specs(), 3);
    let con = catalog::connect(&db, false).unwrap();
    let a = ignis_evidence::corpus::snapshot(&con).unwrap();
    assert_eq!(
        a,
        ignis_evidence::corpus::snapshot(&con).unwrap(),
        "deterministic"
    );
    // re-attributing a passage changes what a scope may read, hence the snapshot
    con.execute(
        "UPDATE chunks SET describes_runs_json = '[]' WHERE ordinal = 0",
        [],
    )
    .unwrap();
    let b = ignis_evidence::corpus::snapshot(&con).unwrap();
    assert_ne!(a.snapshot_id, b.snapshot_id);
    let _ = std::fs::remove_dir_all(dir);
}

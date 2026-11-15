//! A reference list dense with the query's words ranks after the passage that reports the finding
//! (it stays on later pages; nothing is excluded), unless the query asks for references. Catalogs
//! written before `content_class` stay searchable read-only and are backfilled by `reclassify`.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    Capability, EvidenceSchema, ExecutionScope, OffsetUnit, Partition, PrincipalId, ScopeId,
    ScopeRole, SearchFilters, SearchMode, SearchPage, SearchRequest, SourceLocator, WorkspaceId,
};
use ignis_core::services::KnowledgeService;
use ignis_evidence::service::{EvidenceService, Lockbox};
use ignis_evidence::{catalog, dense, records};
use serde_json::json;

const FINDING: &str = "The limiting oxygen concentration for flame spread over the thin cellulose \
sample was 17 % at an opposed flow of 5 cm/s; below this oxygen concentration the flame shrank to a \
dim leading edge and extinguished within 30 s, which sets the flammability limit of the material.";

const REFERENCES: &str = "1. Ohtani, H., Hirano, T., Akita, K., Oxygen concentration flammability limit of thin solids, Proc. Combust. Inst. 19 (1982) pp. 611-618.\n\
2. Olson, S. L., Ferkul, P. V., Oxygen concentration and the flammability limit of PMMA in microgravity, Combust. Flame 159 (2012) 3583-3588.\n\
3. T'ien, J. S., Flammability limit and limiting oxygen concentration of thin fuels, Combust. Sci. Technol. 50 (1986) 185-203.\n\
4. Fujita, O., Flame spread, oxygen concentration and flammability limits in space, Proc. Combust. Inst. 35 (2015) 2487-2502.";

const QUERY: &str = "oxygen concentration flammability limit";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Publication `n` with one passage under `section`.
fn add(con: &mut rusqlite::Connection, n: u8, section: Option<&str>, text: &str) {
    add_page(con, n, &[(section, "text", text)]);
}

/// Publication `n` whose page 1 holds `parts` (section, kind, text) in reading order.
fn add_page(con: &mut rusqlite::Connection, n: u8, parts: &[(Option<&str>, &str, &str)]) {
    let sha = format!("{n:02x}").repeat(32);
    let publ = records::Publication {
        id: format!("pub-{}", &sha[..16]),
        source_sha256: sha.clone(),
        title: Some(format!("Synthetic report {n}")),
        authors: vec![],
        year: None,
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: vec![],
        families: vec![],
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
    let mut page = String::new();
    let mut chunks = Vec::new();
    for (i, (section, kind, text)) in parts.iter().enumerate() {
        let start = page.chars().count() as i64;
        page.push_str(text);
        let end = page.chars().count() as i64;
        page.push('\n');
        chunks.push(records::Chunk {
            id: format!("{view}:p1:{i:05}"),
            publication_id: publ.id.clone(),
            ordinal: i as i64,
            kind: kind.to_string(),
            page: 1,
            char_start: Some(start),
            char_end: Some(end),
            section: section.map(String::from),
            text: text.to_string(),
            context: json!({}),
            locator: SourceLocator {
                asset_sha256: sha.clone(),
                page: Some(1),
                char_start: Some(start),
                char_end: Some(end),
                parse_view_id: Some(view.clone()),
                offset_unit: Some(OffsetUnit::UnicodeScalar),
                ..Default::default()
            },
            describes_runs: vec![],
            continues_runs: vec![],
            outcome_bearing: false,
            evidence_class: "source_derived".into(),
        });
    }
    catalog::upsert_parse_view(con, &publ, &chunks, &[page], None).unwrap();
}

fn source(n: u8) -> String {
    format!("sha256:{}", format!("{n:02x}").repeat(32))
}

fn service(db: &Path) -> EvidenceService {
    let cfg = dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap();
    let mut s = EvidenceService::open(db, cfg, Lockbox::new(&[], HashMap::new()))
        .unwrap()
        .offline();
    s.ledger = false;
    s
}

fn search(s: &EvidenceService, query: &str) -> Vec<String> {
    hits(s, query)
        .iter()
        .map(|h| h.evidence.source_id.as_str().to_string())
        .collect()
}

fn hits(s: &EvidenceService, query: &str) -> Vec<ignis_core::contracts::SearchHit> {
    let sc = ExecutionScope {
        scope_id: ScopeId::new("s").unwrap(),
        workspace_id: WorkspaceId::new("w").unwrap(),
        principal: PrincipalId::new("p").unwrap(),
        role: ScopeRole::Research,
        partitions: [Partition::Train, Partition::Validation]
            .into_iter()
            .collect(),
        capabilities: BTreeSet::from([Capability::ReadSources]),
        corpus_snapshot: None,
        parent_scope_id: None,
    };
    let req = SearchRequest {
        response_format: None,
        expected_terms: Vec::new(),
        schema_version: EvidenceSchema::V1,
        query: query.into(),
        mode: SearchMode::Hybrid,
        filters: SearchFilters::default(),
        corpus_snapshot: None,
        page_size: Some(10),
        cursor: None,
    };
    let p: SearchPage = s.search(&sc, &req).unwrap();
    p.validate().unwrap();
    p.hits
}

/// Publications of the lexical order, with every class ranked with the body (`plain`: BM25 alone).
fn lexical_sources(db: &Path, plain: bool) -> Vec<String> {
    let con = catalog::connect(db, true).unwrap();
    let f = catalog::Filters {
        ranked_with_body: if plain {
            ["references", "contents", "glossary", "fragment"]
                .map(String::from)
                .to_vec()
        } else {
            vec![]
        },
        ..Default::default()
    };
    catalog::lexical(&con, QUERY, &f, 10)
        .unwrap()
        .into_iter()
        .map(|h| h.chunk_id[3..19].to_string())
        .collect()
}

fn fresh(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ignis-refs-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("evidence.sqlite")
}

#[test]
fn a_reference_list_ranks_after_the_finding_unless_the_query_asks_for_references() {
    let db = fresh("rank");
    let mut con = catalog::connect(&db, false).unwrap();
    add(&mut con, 1, Some("3. Results"), FINDING);
    // no heading: classified from the text (a PDFium block)
    add(&mut con, 2, None, REFERENCES);
    // a list item under a Docling "References" heading
    add(
        &mut con,
        3,
        Some("References"),
        "Oxygen concentration, flammability limit, flame spread: a review of limiting oxygen concentration data.",
    );
    drop(con);

    // plain BM25 ranks the reference list first: the failure this guards against
    let raw = lexical_sources(&db, true);
    assert_eq!(raw.len(), 3);
    assert_ne!(
        raw[0], "0101010101010101",
        "fixture no longer reproduces the problem"
    );

    let s = service(&db);
    let hits = search(&s, QUERY);
    assert_eq!(hits.len(), 3, "nothing is excluded");
    assert_eq!(hits[0], source(1), "the finding leads: {hits:?}");

    // a query about references ranks them by BM25 alone again
    let hits = search(&s, &format!("references on {QUERY}"));
    assert_eq!(hits.len(), 3);
    assert_ne!(hits[0], source(1), "{hits:?}");
}

#[test]
fn a_catalog_without_the_class_reads_and_is_backfilled_by_reclassify() {
    let db = fresh("legacy");
    let mut con = catalog::connect(&db, false).unwrap();
    add(&mut con, 1, Some("3. Results"), FINDING);
    add(&mut con, 2, None, REFERENCES);
    // a catalog written before `content_class`
    con.execute_batch("ALTER TABLE chunks DROP COLUMN content_class")
        .unwrap();
    drop(con);

    // read-only (the server): searchable, BM25 order, no demotion
    assert_eq!(lexical_sources(&db, false), lexical_sources(&db, true));
    let s = service(&db);
    assert_eq!(search(&s, QUERY).len(), 2);
    drop(s);

    // opened for writing the column is added (everything `body`); reclassify derives the classes
    let mut con = catalog::connect(&db, false).unwrap();
    let r = catalog::reclassify(&mut con).unwrap();
    assert_eq!(r["changed"], 1, "{r}");
    assert_eq!(r["classes"]["references"], 1, "{r}");
    let again = catalog::reclassify(&mut con).unwrap();
    assert_eq!(again["changed"], 0, "idempotent: {again}");
    drop(con);
    assert_eq!(lexical_sources(&db, false)[0], "0101010101010101");
    assert_eq!(search(&service(&db), QUERY)[0], source(1));
}

#[test]
fn acronym_rows_and_a_crowded_page_do_not_fill_the_first_results() {
    let db = fresh("glossary");
    let mut con = catalog::connect(&db, false).unwrap();
    add(&mut con, 1, Some("3. Results"), FINDING);
    // one page of a safety data package: two acronym rows (short, holding the query words) and
    // four passages of growing length about the same limits
    let gl = Some("1.2 ACRONYMS AND ABBREVIATIONS");
    let body = |n: usize| {
        format!(
            "Flammability limit {n}: the oxygen concentration of the cabin atmosphere stays below \
             the limit of the stored materials during the operations of this payload.{}",
            " The hazard report lists the controls.".repeat(n)
        )
    };
    let (b1, b2, b3, b4) = (body(1), body(2), body(3), body(4));
    add_page(
        &mut con,
        4,
        &[
            (gl, "table_row", "LFL | Lower Flammability Limit"),
            (gl, "table_row", "UFL | Upper Flammability Limit"),
            (None, "text", &b1),
            (None, "text", &b2),
            (None, "text", &b3),
            (None, "text", &b4),
        ],
    );
    drop(con);
    let s = service(&db);
    let h = hits(&s, "lower flammability limit oxygen concentration");
    assert_eq!(h.len(), 7, "nothing is excluded");
    let of =
        |x: &ignis_core::contracts::SearchHit, n: u8| x.evidence.source_id.as_str() == source(n);
    assert!(
        h.iter().take(3).filter(|x| of(x, 4)).count() <= 2,
        "at most two hits of one page before the others"
    );
    assert!(
        h.iter().take(3).any(|x| of(x, 1)),
        "the finding of the other document is in the first three"
    );
    // the page's longest passages take its places (b4, b3); its acronym rows come last
    let page4: Vec<&str> = h
        .iter()
        .filter(|x| of(x, 4))
        .map(|x| x.snippet.as_str())
        .collect();
    assert!(
        page4[0].starts_with("Flammability limit 4")
            || page4[0].starts_with("Flammability limit 3"),
        "{page4:?}"
    );
    assert!(
        page4[1].starts_with("Flammability limit 4")
            || page4[1].starts_with("Flammability limit 3"),
        "{page4:?}"
    );
    assert!(
        h[5..].iter().all(|x| x.snippet.contains(" | ")),
        "{page4:?}"
    );

    // asked what an acronym stands for, the rows rank by BM25 with the body again
    let h = hits(&s, "what does LFL stand for lower flammability limit");
    assert!(
        h.iter().take(2).any(|x| x.snippet.contains("LFL |")),
        "{:?}",
        h.iter().map(|x| &x.snippet).collect::<Vec<_>>()
    );
}

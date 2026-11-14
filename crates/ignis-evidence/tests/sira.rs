//! SIRA expansion (R12b) and the query finalizer, on a synthetic catalog:
//! - a chunk matched only through its document expansion is found, and ranks below a chunk whose
//!   own text matches;
//! - a page the pack protects raises outcome_bearing, and a pack never lowers it;
//! - words held by more than tau of the chunks are dropped from the expansion;
//! - the finalizer keeps only expressions the corpus holds, rarest first, IDF-weighted;
//! - an unenriched catalog searches exactly as before.

use std::path::{Path, PathBuf};

use ignis_core::contracts::{OffsetUnit, SourceLocator};
use ignis_evidence::{catalog, records, sira};
use serde_json::json;

const SHA: &str = "abababababababababababababababababababababababababababababababab";

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ignis-sira-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// One publication, one chunk per page, with the given texts and outcome flags.
fn catalog(dir: &Path, pages: &[(&str, bool)]) -> rusqlite::Connection {
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    let publ = records::Publication {
        id: format!("pub-{}", &SHA[..16]),
        source_sha256: SHA.into(),
        title: Some("Synthetic NASA report".into()),
        authors: vec![],
        year: Some(2020),
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: vec!["PSI-22".into()],
        families: vec![],
        file_name: Some("synthetic.pdf".into()),
        n_pages: pages.len() as i64,
        rights_status: "verified".into(),
        license: None,
        describes_runs: vec![],
        outcome_bearing: false,
        ingested_at: "2026-11-14T00:00:00Z".into(),
        parser_version: "test".into(),
    };
    let view = records::parse_view_id(&publ.id, &publ.parser_version);
    let texts: Vec<String> = pages.iter().map(|(t, _)| t.to_string()).collect();
    let chunks: Vec<records::Chunk> = pages
        .iter()
        .enumerate()
        .map(|(i, (t, outcome))| {
            let n = t.chars().count() as i64;
            records::Chunk {
                id: format!("{view}:p{}:00000", i + 1),
                publication_id: publ.id.clone(),
                ordinal: i as i64,
                kind: "text".into(),
                page: i as i64 + 1,
                char_start: Some(0),
                char_end: Some(n),
                section: None,
                text: t.to_string(),
                context: json!({}),
                locator: SourceLocator {
                    asset_sha256: SHA.into(),
                    page: Some(i as i64 + 1),
                    char_start: Some(0),
                    char_end: Some(n),
                    parse_view_id: Some(view.clone()),
                    offset_unit: Some(OffsetUnit::UnicodeScalar),
                    ..Default::default()
                },
                describes_runs: vec![],
                continues_runs: vec![],
                outcome_bearing: *outcome,
                evidence_class: "source_derived".into(),
            }
        })
        .collect();
    catalog::upsert_parse_view(&mut con, &publ, &chunks, &texts, None).unwrap();
    con
}

fn pack(dir: &Path, rows: &[(i64, &[&str], bool)]) -> PathBuf {
    let p = dir.join("pack");
    std::fs::create_dir_all(&p).unwrap();
    let lines: Vec<String> = rows
        .iter()
        .map(|(page, terms, prot)| {
            json!({"schema": "ignis.docs.export.page/1", "source_sha256": [SHA], "page": page,
                   "expansion": terms, "outcome_bearing": prot})
            .to_string()
        })
        .collect();
    std::fs::write(p.join("pages.jsonl"), lines.join("\n")).unwrap();
    std::fs::write(p.join("MANIFEST.json"), "{}").unwrap();
    p
}

fn page_of(id: &str) -> i64 {
    id.split(":p")
        .nth(1)
        .unwrap()
        .split(':')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn expansion_finds_vocabulary_the_text_lacks_and_ranks_it_below_text_matches() {
    let d = tmp("expansion");
    let mut con = catalog(
        &d,
        &[
            (
                "The photomultiplier signal fell to zero after ignition.",
                false,
            ),
            ("A PMT trace is shown for the second burn.", false),
            ("Coflow nozzle and chamber hardware are described.", false),
            ("The chamber was filled with oxygen and nitrogen.", false),
        ],
    );
    let none = catalog::Filters::default();
    // before enrichment: page 2 is not found by the long name
    let before = catalog::lexical(&con, "photomultiplier", &none, 10).unwrap();
    assert_eq!(
        before
            .iter()
            .map(|h| page_of(&h.chunk_id))
            .collect::<Vec<_>>(),
        vec![1]
    );

    let p = pack(
        &d,
        &[
            (2, &["photomultiplier tube", "PMT = photomultiplier"], false),
            (3, &["chamber"], true),
        ],
    );
    // tau 0.3: "chamber" (2 of 4 chunks) is dropped, "photomultiplier" (1 of 4) kept
    let info = sira::enrich(&mut con, &p, 0.3).unwrap();
    assert_eq!(info["outcome_bearing_raised"], 1);
    assert!(info["words_dropped_by_df"].as_u64().unwrap() >= 1);

    let after = catalog::lexical(&con, "photomultiplier", &none, 10).unwrap();
    let pages: Vec<i64> = after.iter().map(|h| page_of(&h.chunk_id)).collect();
    assert_eq!(
        pages,
        vec![1, 2],
        "the text match first, the expansion match after it"
    );

    // a protected page raises outcome_bearing; a later pack that does not protect it never lowers it
    let ob = |con: &rusqlite::Connection| -> i64 {
        con.query_row(
            "SELECT outcome_bearing FROM chunks WHERE page = 3",
            [],
            |r| r.get(0),
        )
        .unwrap()
    };
    assert_eq!(ob(&con), 1);
    let p2 = pack(&d, &[(3, &["nozzle"], false)]);
    sira::enrich(&mut con, &p2, 0.3).unwrap();
    assert_eq!(ob(&con), 1);
}

#[test]
fn finalizer_keeps_held_expressions_rarest_first() {
    let d = tmp("finalizer");
    let con = catalog(
        &d,
        &[
            ("soot volume fraction was measured", false),
            ("soot and smoke point", false),
            ("smoke point of ethylene", false),
            ("flame length", false),
        ],
    );
    let proposals = vec![
        ("smoke point".to_string(), 1.0),
        ("volume fraction".to_string(), 1.0),
        ("cool flame".to_string(), 1.0),
    ];
    let out = sira::select_expressions(&con, &proposals).unwrap();
    let get = |e: &str| out.iter().find(|c| c.expression == e).unwrap().clone();
    assert_eq!(get("cool flame").chunks, 0);
    assert!(get("cool flame").weight.is_none());
    assert_eq!(get("volume fraction").chunks, 1);
    assert_eq!(get("smoke point").chunks, 2);
    assert!(
        get("volume fraction").weight.unwrap() > get("smoke point").weight.unwrap(),
        "rarer weighs more"
    );

    // query-side expansion brings the phrase's chunk into a search that did not name it
    let none = catalog::Filters::default();
    let plain = catalog::lexical_expanded(&con, "flame", &[], &none, 0, 10).unwrap();
    let exp = catalog::lexical_expanded(
        &con,
        "flame",
        &[("volume fraction".into(), 1.0)],
        &none,
        0,
        10,
    )
    .unwrap();
    assert!(plain.iter().all(|h| page_of(&h.chunk_id) != 1));
    assert!(exp.iter().any(|h| page_of(&h.chunk_id) == 1));
}

#[test]
fn term_stats_count_chunks_and_publications_and_go_stale_with_the_catalog() {
    let d = tmp("terms");
    let mut con = catalog(
        &d,
        &[("flame flame spread", false), ("flame extinction", false)],
    );
    let info = sira::build_term_stats(&mut con).unwrap();
    assert_eq!(info["chunks"], 2);
    let (chunks, pubs, inst): (i64, i64, i64) = con
        .query_row(
            "SELECT chunks, publications, instances FROM term_stats WHERE term = 'flame'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!((chunks, pubs, inst), (2, 1, 3));
    assert!(sira::TermStats::load(&con).unwrap().is_some());
    con.execute("DELETE FROM chunks WHERE page = 2", [])
        .unwrap();
    assert!(
        sira::TermStats::load(&con).unwrap().is_none(),
        "a table of another generation is not read"
    );
}

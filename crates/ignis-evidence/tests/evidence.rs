//! Essential evidence checks: FTS + RRF returns a known chunk with its page locator; the firewall
//! passes on this workspace and catches leaks; sealed runs strip outcome-bearing evidence.
//! No network here: the API key is cleared on the retriever, so dense and rerank degrade declaratively.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(feature = "pdf")]
use ignis_evidence::pdf;
use ignis_evidence::{catalog, dense, firewall, records, search};
use serde_json::json;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ignis-evidence-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn rrf_uses_ranks_only() {
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let (lex, den) = (s(&["a", "b", "c"]), s(&["c", "a"]));
    let fused: HashMap<String, f64> = search::rrf(&[&lex, &den], 60.0).into_iter().collect();
    assert!((fused["a"] - (1.0 / 61.0 + 1.0 / 62.0)).abs() < 1e-12);
    assert!((fused["c"] - (1.0 / 63.0 + 1.0 / 61.0)).abs() < 1e-12);
    assert!((fused["b"] - 1.0 / 62.0).abs() < 1e-12); // absent from dense contributes zero
    assert_eq!(search::rrf(&[&lex, &den], 60.0)[0].0, "a");
}

#[cfg(feature = "pdf")]
#[test]
fn fts_rrf_returns_known_chunk_with_page_locator() {
    let pdf_path = root().join("data/raw/evidence/ntrs/NTRS-20130014061.pdf"); // Nayagam, Dietrich et al. 2013
    if !pdf_path.exists() {
        eprintln!("skipped: NTRS sample PDF not fetched");
        return;
    }
    let rp = pdf::RunPatterns::load(&root().join("configs/evidence/run_patterns.toml")).unwrap();
    let (publ, chunks, _) = pdf::parse_pdf(&pdf_path, &pdf::ParseOptions::default(), &rp).unwrap();
    assert_eq!(chunks.len(), 46); // same as the PyMuPDF reference
    let dir = tmp("fts");
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    catalog::upsert_publication(&mut con, &publ, &chunks).unwrap();
    drop(con);
    let cfg =
        ignis_evidence::dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml"))
            .unwrap();
    let mut r = search::Retriever::open(&db, cfg).unwrap();
    let opts = search::SearchOpts {
        limit: 10,
        use_dense: false,
        use_rerank: false,
        ..Default::default()
    };
    // known passage: §3.3 "Flame Extinction", page 8 — the two extinction mechanisms
    let q = "two mechanisms by which the flame extinguishes: diffusive extinction from insufficient residence time";
    let res = r.search(q, &catalog::Filters::default(), &opts).unwrap();
    assert!(res["stages"]["lexical"].as_str().unwrap().starts_with("ok"));
    assert_eq!(res["stages"]["dense"], "disabled");
    assert_eq!(res["rrf_k"], 60.0);
    let top = &res["results"][0];
    assert_eq!(top["page"], 8);
    assert_eq!(top["section"], "3.3 Flame Extinction");
    assert_eq!(top["kind"], "text");
    assert_eq!(top["lexical_rank"], 1);
    assert!((top["rrf"].as_f64().unwrap() - 1.0 / 61.0).abs() < 1e-12);
    let text = top["text"].as_str().unwrap();
    assert!(text
        .replace(['\u{201c}', '\u{201d}'], "")
        .replace('\n', " ")
        .contains("diffusive extinction"));
    // the locator points to the exact characters of the original page
    let loc = &top["locator"];
    assert_eq!(loc["page"], 8);
    assert_eq!(loc["asset_sha256"], json!(publ.source_sha256));
    let (doc, _) = pdf::open_pdf(&pdf_path).unwrap();
    let span = |page: i64, a: i64, b: i64| -> String {
        pdf::page_text(&doc, page as usize)
            .unwrap()
            .chars()
            .skip(a as usize)
            .take((b - a) as usize)
            .collect()
    };
    assert_eq!(
        span(
            8,
            loc["char_start"].as_i64().unwrap(),
            loc["char_end"].as_i64().unwrap()
        ),
        text
    );
    for c in &chunks {
        if let (Some(a), Some(b)) = (c.char_start, c.char_end) {
            assert_eq!(span(c.page, a, b), c.text, "{}", c.id);
        }
    }
    // metadata filters are applied inside the lexical query
    let f = catalog::Filters {
        family: Some("bass".into()),
        ..Default::default()
    };
    assert!(r.search("methanol", &f, &opts).unwrap()["results"]
        .as_array()
        .unwrap()
        .is_empty());
    // no vector file and no rerank model needed: dense skipped, rerank falls back declaratively
    let res = r
        .search(
            q,
            &catalog::Filters::default(),
            &search::SearchOpts {
                limit: 3,
                use_rerank: false,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(res["stages"]["dense"]
        .as_str()
        .unwrap()
        .starts_with("skipped: no vector file"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn firewall_passes_on_current_workspace() {
    let protected =
        firewall::protected_from_config(&root().join("configs/evidence/firewall.toml")).unwrap();
    let r = firewall::scan(&root().join("Cargo.toml"), &protected).unwrap();
    assert!(r.passed, "{:?}", r.violations);
    assert!(r.present.contains(&"ignis-science".to_string()));
    assert!(r.members_scanned >= 4);
}

#[test]
fn firewall_detects_leaks() {
    let ws = tmp("fw");
    std::fs::write(
        ws.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();
    let krate = |name: &str, deps: &str| {
        let d = ws.join("crates").join(name);
        std::fs::create_dir_all(d.join("src")).unwrap();
        std::fs::write(d.join("src/lib.rs"), "").unwrap();
        std::fs::write(
            d.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n{deps}"
            ),
        )
        .unwrap();
    };
    krate("ignis-evidence", "");
    krate(
        "ignis-util",
        "[dependencies]\nignis-evidence = { path = \"../ignis-evidence\" }\n",
    );
    krate(
        "ignis-science",
        "[dependencies]\nignis-util = { path = \"../ignis-util\" }\n",
    ); // transitive
    krate(
        "ignis-flex",
        "[dev-dependencies]\nev = { package = \"ignis-evidence\", path = \"../ignis-evidence\" }\n",
    ); // renamed, dev
    krate("ignis-bass", "");
    let protected: Vec<String> = firewall::DEFAULT_PROTECTED
        .iter()
        .map(|s| s.to_string())
        .collect();
    let r = firewall::scan(&ws.join("Cargo.toml"), &protected).unwrap();
    assert!(!r.passed);
    let v: HashMap<&str, &Vec<String>> = r
        .violations
        .iter()
        .map(|v| (v.krate.as_str(), &v.chain))
        .collect();
    assert_eq!(v.len(), 2, "{:?}", r.violations);
    assert_eq!(v["ignis-science"][0], "ignis-science");
    assert!(v["ignis-science"]
        .last()
        .unwrap()
        .starts_with("ignis-evidence"));
    assert_eq!(v["ignis-science"].len(), 3);
    assert!(v["ignis-flex"][1].contains("[dev]"));
    let _ = std::fs::remove_dir_all(ws);
}

#[test]
fn sealed_runs_strip_outcome_bearing_evidence() {
    let items = vec![
        json!({"id": "a", "outcome_bearing": true, "describes_runs": ["alias:bass2:testT12"]}),
        json!({"id": "b", "outcome_bearing": true, "describes_runs": ["run-other"]}),
        json!({"id": "c", "outcome_bearing": false, "describes_runs": ["alias:bass2:testT12"]}),
        json!({"id": "d", "outcome_bearing": true, "describes_runs": []}),
    ];
    let amap = HashMap::from([("alias:bass2:testT12".to_string(), "run-T12".to_string())]);
    let (kept, stripped) = firewall::strip_for_sealed(items, &["run-T12".to_string()], &amap, true);
    assert_eq!(stripped, ["a", "d"]);
    assert_eq!(
        kept.iter()
            .map(|x| x["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["b", "c"]
    );
}

fn cfg() -> dense::RetrievalCfg {
    dense::RetrievalCfg::load(&root().join("configs/evidence/retrieval.toml")).unwrap()
}

/// A two-chunk catalog without MuPDF.
fn tiny_catalog(dir: &Path) -> PathBuf {
    let db = dir.join("evidence.sqlite");
    let mut con = catalog::connect(&db, false).unwrap();
    let sha = "ab".repeat(32);
    let publ = records::Publication {
        id: "pub-test".into(),
        source_sha256: sha.clone(),
        title: Some("Droplet tests".into()),
        authors: vec![],
        year: Some(2013),
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: vec![],
        families: vec!["flex".into()],
        file_name: Some("t.pdf".into()),
        n_pages: 1,
        rights_status: "verified".into(),
        license: None,
        describes_runs: vec![],
        outcome_bearing: false,
        ingested_at: "2026-11-14T00:00:00Z".into(),
        parser_version: "test".into(),
    };
    let chunk = |i: i64, text: &str| records::Chunk {
        id: format!("chk-{i}"),
        publication_id: publ.id.clone(),
        ordinal: i,
        kind: "text".into(),
        page: 1,
        char_start: Some(0),
        char_end: Some(text.chars().count() as i64),
        section: None,
        text: text.into(),
        context: json!({}),
        locator: ignis_core::contracts::SourceLocator {
            asset_sha256: sha.clone(),
            path: None,
            row: None,
            column: None,
            page: Some(1),
            frame: None,
            pts_s: None,
            char_start: Some(0),
            char_end: Some(text.chars().count() as i64),
            ..Default::default()
        },
        describes_runs: vec![],
        outcome_bearing: false,
        evidence_class: "source_derived".into(),
    };
    let chunks = vec![
        chunk(0, "methanol droplet flame extinction diameter"),
        chunk(1, "opposed flow flame spread over a solid"),
    ];
    catalog::upsert_publication(&mut con, &publ, &chunks).unwrap();
    db
}

fn unit(i: usize, dim: usize) -> Vec<f32> {
    (0..dim).map(|j| if j == i { 1.0 } else { 0.0 }).collect()
}

#[test]
fn lexical_answers_without_api_key() {
    let dir = tmp("nokey");
    let db = tiny_catalog(&dir);
    let c = cfg();
    let id = c.embedding.identity();
    let dim = c.embedding.dimensions.unwrap_or(8);
    dense::VectorStore::write(
        &dir,
        &["chk-0".into(), "chk-1".into()],
        &[unit(0, dim), unit(1, dim)],
        &id,
        "api:test",
    )
    .unwrap();
    let mut r = search::Retriever::open(&db, c).unwrap();
    r.api_key = None; // offline demo: no key, no network
    let res = r
        .search(
            "methanol extinction",
            &catalog::Filters::default(),
            &search::SearchOpts::default(),
        )
        .unwrap();
    assert!(res["stages"]["lexical"].as_str().unwrap().starts_with("ok"));
    assert_eq!(
        res["stages"]["dense"],
        "skipped: no API key (DEEPINFRA_API_KEY unset)"
    );
    assert_eq!(
        res["stages"]["rerank"],
        "fallback_rrf: no API key (DEEPINFRA_API_KEY unset)"
    );
    assert_eq!(res["results"][0]["chunk_id"], "chk-0");
    assert!(res["results"][0]["rerank"].is_null());
    assert!(res["usage"].as_object().unwrap().is_empty()); // nothing billed
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn mixed_models_are_refused() {
    let dir = tmp("mixed");
    let db = tiny_catalog(&dir);
    let c = cfg();
    let want = c.embedding.identity();
    let mut other = want.clone();
    other.model = "Qwen/Qwen3-Embedding-0.6B".into();
    other.dim = Some(4);
    dense::VectorStore::write(
        &dir,
        &["chk-0".into(), "chk-1".into()],
        &[unit(0, 4), unit(1, 4)],
        &other,
        "api:test",
    )
    .unwrap();
    let meta = dense::VectorStore::read_meta(&dir).unwrap().unwrap();
    assert!(meta.get("ids").is_none());
    // search: a set from another model is never compared with this model's queries
    let e = dense::check_compatible(&meta, &want).unwrap_err();
    assert!(
        e.contains("model Qwen/Qwen3-Embedding-0.6B != Qwen/Qwen3-Embedding-8B")
            && e.contains("dim 4"),
        "{e}"
    );
    let mut r = search::Retriever::open(&db, c.clone()).unwrap();
    r.api_key = None;
    let res = r
        .search(
            "methanol",
            &catalog::Filters::default(),
            &search::SearchOpts {
                use_rerank: false,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        res["stages"]["dense"]
            .as_str()
            .unwrap()
            .contains("not comparable"),
        "{}",
        res["stages"]
    );
    // embed: replacing another model's set, or re-embedding the same model, needs --force
    assert!(dense::embed_permission(Some(&meta), &want, false)
        .unwrap_err()
        .contains("never mixed"));
    assert!(dense::embed_permission(Some(&meta), &want, true).is_ok());
    let mut same = meta.clone();
    same["model"] = json!(want.model);
    same["dim"] = json!(want.dim);
    same["provider"] = json!(want.provider);
    assert!(dense::embed_permission(Some(&same), &want, false)
        .unwrap_err()
        .contains("re-embedding is refused"));
    assert!(dense::check_compatible(&same, &want).is_ok());
    // a legacy local (candle) file has no provider and never matches
    same.as_object_mut().unwrap().remove("provider");
    assert!(dense::check_compatible(&same, &want)
        .unwrap_err()
        .contains("provider local != deepinfra"));
    assert!(dense::embed_permission(None, &want, false).is_ok());
    let _ = std::fs::remove_dir_all(dir);
}

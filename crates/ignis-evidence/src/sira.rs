//! SIRA retrieval for the evidence catalog (LANES_V2 R12a/R12b; arXiv 2605.06647), ported from the
//! Corpus app's national legislation search (`nationalterms`, `national_term_stats.go`,
//! `national_sira_select.go`), from Vespa to SQLite FTS5. No embeddings, no reranker.
//!
//! - **Document side** ([`enrich`]): a pack of page cards (`ignis.docs.export.page/1`, keyed by the
//!   NASA file's sha256 and the 1-based page) gives each chunk the vocabulary a reader would type;
//!   terms present in more than `tau` of the chunks are dropped (they add no discrimination); the
//!   rest go into `expansion_fts`, scored as its own column ([`DOC_EXPANSION_WEIGHT`]). A page the
//!   pack marks outcome-bearing raises its chunks' `outcome_bearing`; nothing is ever lowered.
//! - **Corpus statistics** ([`build_term_stats`], [`TermStats`]): per indexed term the chunks,
//!   publications and instances holding it, plus totals and the catalog generation they were built
//!   on; phrases are counted exactly with one FTS5 MATCH ([`expression_chunks`]).
//! - **Query side** ([`select_expressions`]): the agent's expected terms (with importance) are kept
//!   when the corpus holds them, rarest first, at most [`MAX_EXPRESSIONS`], each weighted by its IDF
//!   relative to the others' mean and bounded to [0.1, 4] (Corpus's "global" arm, the one it serves:
//!   R@12 +0.018 [+0.004, +0.034] on its held-out legislation questions; untuned here).

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{bail, Context, Result};
use ignis_core::hashing::sha256_hex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};

/// Weight of a document-expansion match relative to a match in the chunk text. R12b says to tune
/// it on the T10 development questions; until then it is the value below, chosen so that a passage
/// whose own text matches still outranks one matched by generated vocabulary alone.
pub const DOC_EXPANSION_WEIGHT: f64 = 0.4;
/// SIRA's DF ceiling for document expansion: a term held by more than this share of chunks is dropped.
pub const DEFAULT_TAU: f64 = 0.05;
/// The query-side finalizer keeps at most this many expressions (Corpus: 12).
pub const MAX_EXPRESSIONS: usize = 12;

#[derive(Debug, Deserialize)]
struct PackPage {
    source_sha256: Vec<String>,
    page: i64,
    #[serde(default)]
    expansion: Vec<String>,
    #[serde(default)]
    outcome_bearing: bool,
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 2)
        .map(|w| w.to_lowercase())
        .collect()
}

/// A phrase as an FTS5 MATCH expression (its words as one quoted phrase).
pub fn phrase_query(expr: &str) -> Result<String> {
    let w = words(expr);
    if w.is_empty() {
        bail!("empty expression");
    }
    Ok(format!("\"{}\"", w.join(" ")))
}

/// Chunks whose text holds the expression (an exact count, same tokenizer as the index).
pub fn expression_chunks(con: &Connection, expr: &str) -> Result<i64> {
    let q = match phrase_query(expr) {
        Ok(q) => q,
        Err(_) => return Ok(0),
    };
    Ok(con.query_row(
        "SELECT count(*) FROM chunks_fts WHERE chunks_fts MATCH ?1",
        [q],
        |r| r.get(0),
    )?)
}

fn total_chunks(con: &Connection) -> Result<i64> {
    Ok(con.query_row("SELECT count(*) FROM chunks", [], |r| r.get(0))?)
}

/// What the catalog's text content is: changes whenever chunks are added, removed or re-parsed.
pub fn generation(con: &Connection) -> Result<String> {
    let (n, max, len): (i64, i64, i64) = con.query_row(
        "SELECT count(*), coalesce(max(rowid), 0), coalesce(sum(length(text_norm)), 0) FROM chunks",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok(format!("chunks:{n}:{max}:{len}"))
}

/// Load a pack into `expansion_fts` (replacing any earlier expansion) and raise `outcome_bearing`.
pub fn enrich(con: &mut Connection, pack: &Path, tau: f64) -> Result<Value> {
    let pages_path = pack.join("pages.jsonl");
    let manifest = std::fs::read(pack.join("MANIFEST.json"))
        .with_context(|| format!("read {}/MANIFEST.json", pack.display()))?;
    let mut by_key: HashMap<(String, i64), (Vec<String>, bool)> = HashMap::new();
    let f = std::fs::File::open(&pages_path)
        .with_context(|| format!("open {}", pages_path.display()))?;
    for line in BufReader::new(f).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let p: PackPage = serde_json::from_str(&line).context("pack page row")?;
        for s in &p.source_sha256 {
            by_key.insert(
                (s.clone(), p.page),
                (p.expansion.clone(), p.outcome_bearing),
            );
        }
    }
    let n = total_chunks(con)?;
    let ceiling = (tau * n as f64).max(1.0);
    let mut df: HashMap<String, i64> = HashMap::new();
    let chunks: Vec<(i64, String, i64, bool)> = {
        let mut st = con.prepare(
            "SELECT c.rowid, p.source_sha256, c.page, c.outcome_bearing FROM chunks c JOIN publications p ON p.id = c.publication_id",
        )?;
        let rows = st.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, i64>(3)? != 0))
        })?;
        rows.collect::<rusqlite::Result<_>>()?
    };
    let (mut enriched, mut raised, mut kept_words, mut dropped_words, mut unmatched) =
        (0, 0, 0usize, 0usize, 0);
    let tx = con.transaction()?;
    tx.execute("DELETE FROM expansion_fts", [])?;
    for (rowid, sha, page, outcome) in chunks {
        let Some((terms, protected)) = by_key.get(&(sha, page)) else {
            unmatched += 1;
            continue;
        };
        if *protected && !outcome {
            tx.execute(
                "UPDATE chunks SET outcome_bearing = 1 WHERE rowid = ?1",
                [rowid],
            )?;
            raised += 1;
        }
        let mut kept: Vec<String> = Vec::new();
        let mut seen = HashSet::new();
        for w in terms.iter().flat_map(|t| words(t)) {
            if !seen.insert(w.clone()) {
                continue;
            }
            let d = match df.get(&w) {
                Some(d) => *d,
                None => {
                    let d = expression_chunks(&tx, &w)?;
                    df.insert(w.clone(), d);
                    d
                }
            };
            if (d as f64) > ceiling {
                dropped_words += 1;
            } else {
                kept_words += 1;
                kept.push(w);
            }
        }
        if !kept.is_empty() {
            tx.execute(
                "INSERT INTO expansion_fts(rowid, terms) VALUES (?1, ?2)",
                params![rowid, kept.join(" ")],
            )?;
            enriched += 1;
        }
    }
    let info = json!({"pack_manifest_sha256": sha256_hex(&manifest), "tau": tau, "weight": DOC_EXPANSION_WEIGHT,
        "generation": generation(&tx)?, "chunks": n, "chunks_enriched": enriched, "chunks_without_card": unmatched,
        "words_kept": kept_words, "words_dropped_by_df": dropped_words, "outcome_bearing_raised": raised});
    tx.execute(
        "INSERT OR REPLACE INTO meta(key, value) VALUES ('expansion', ?1)",
        [info.to_string()],
    )?;
    tx.commit()?;
    Ok(info)
}

/// Rebuild `term_stats` from the FTS5 index of chunk text.
pub fn build_term_stats(con: &mut Connection) -> Result<Value> {
    let tx = con.transaction()?;
    tx.execute_batch(
        "DROP TABLE IF EXISTS temp.chunk_vocab;
         CREATE VIRTUAL TABLE temp.chunk_vocab USING fts5vocab(main, 'chunks_fts', 'instance');
         DELETE FROM term_stats;
         INSERT INTO term_stats(term, chunks, publications, instances)
           SELECT v.term, count(DISTINCT v.doc), count(DISTINCT c.publication_id), count(*)
           FROM temp.chunk_vocab v JOIN chunks c ON c.rowid = v.doc
           WHERE v.col = 'text' GROUP BY v.term;
         DROP TABLE temp.chunk_vocab;",
    )?;
    let terms: i64 = tx.query_row("SELECT count(*) FROM term_stats", [], |r| r.get(0))?;
    let pubs: i64 = tx.query_row("SELECT count(*) FROM publications", [], |r| r.get(0))?;
    let info = json!({"generation": generation(&tx)?, "chunks": total_chunks(&tx)?, "publications": pubs, "terms": terms});
    tx.execute(
        "INSERT OR REPLACE INTO meta(key, value) VALUES ('term_stats', ?1)",
        [info.to_string()],
    )?;
    tx.commit()?;
    Ok(info)
}

/// Corpus statistics as the finalizer reads them. `None` when the table was built on another
/// generation of the catalog (a stale table is never read; phrases are then counted exactly).
pub struct TermStats {
    pub chunks: i64,
    pub publications: i64,
}

impl TermStats {
    pub fn load(con: &Connection) -> Result<Option<TermStats>> {
        let Some(raw): Option<String> = con
            .query_row("SELECT value FROM meta WHERE key = 'term_stats'", [], |r| {
                r.get(0)
            })
            .optional()?
        else {
            return Ok(None);
        };
        let v: Value = serde_json::from_str(&raw)?;
        if v["generation"].as_str() != Some(generation(con)?.as_str()) {
            return Ok(None);
        }
        Ok(Some(TermStats {
            chunks: v["chunks"].as_i64().unwrap_or(0),
            publications: v["publications"].as_i64().unwrap_or(0),
        }))
    }
}

/// BM25's IDF of a count `df` in a population `n` (Corpus's `textIDF`).
pub fn idf(df: i64, n: i64) -> f64 {
    (1.0 + (n as f64 - df as f64 + 0.5) / (df as f64 + 0.5)).ln()
}

/// One proposed expression and what the finalizer measured and decided.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Choice {
    pub expression: String,
    pub importance: f64,
    pub chunks: i64,
    /// `None` when set aside, with the reason in `aside`.
    pub weight: Option<f64>,
    pub aside: Option<String>,
}

/// The query-side finalizer: keep each proposed expression the corpus holds (DF > 0), rarest
/// first, at most [`MAX_EXPRESSIONS`]; weight = clamp(idf * k / sum(idf), 0.1, 4) * importance,
/// so the mean kept expression weighs as one query word.
pub fn select_expressions(con: &Connection, proposals: &[(String, f64)]) -> Result<Vec<Choice>> {
    let n = total_chunks(con)?;
    let mut seen = HashSet::new();
    let mut all: Vec<Choice> = Vec::new();
    for (e, imp) in proposals {
        let key = words(e).join(" ");
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        let d = expression_chunks(con, e)?;
        all.push(Choice {
            expression: e.clone(),
            importance: *imp,
            chunks: d,
            weight: None,
            aside: (d == 0).then(|| "no chunk text holds it".to_string()),
        });
    }
    let mut usable: Vec<usize> = (0..all.len()).filter(|&i| all[i].aside.is_none()).collect();
    usable.sort_by_key(|&i| (all[i].chunks, i));
    for &i in usable.iter().skip(MAX_EXPRESSIONS) {
        all[i].aside = Some(format!("beyond the {MAX_EXPRESSIONS} rarest"));
    }
    usable.truncate(MAX_EXPRESSIONS);
    let sum: f64 = usable.iter().map(|&i| idf(all[i].chunks, n)).sum();
    for &i in &usable {
        let rel = if sum > 0.0 {
            idf(all[i].chunks, n) * usable.len() as f64 / sum
        } else {
            1.0
        };
        all[i].weight = Some(rel.clamp(0.1, 4.0) * all[i].importance.clamp(0.0, 1.0));
    }
    Ok(all)
}

/// Importance as the agent states it (Corpus: high 1, medium 0.6, low 0.3).
pub fn importance(label: Option<&str>) -> f64 {
    match label.map(|s| s.to_ascii_lowercase()) {
        Some(s) if s == "high" => 1.0,
        Some(s) if s == "low" => 0.3,
        _ => 0.6,
    }
}

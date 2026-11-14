//! SQLite evidence catalog with FTS5 (V3.1 §14.1, §14.6, §18.2).
//!
//! Path: `artifacts/evidence/evidence.sqlite` (`IGNIS_ARTIFACTS` overrides `artifacts`). The release
//! catalog is read-only for its readers (the server opens it with `SQLITE_OPEN_READONLY`); it is
//! written by one job, never as a multi-host WAL database. The DDL is `schemas/evidence_catalog.sql`
//! (compiled in; the file is the single copy).
//!
//! BM25 convention: FTS5's `bm25()` returns *more negative = more relevant*; `ORDER BY bm25(...)`
//! ascending is best-first. `lexical_rank` = 1 + position in that order. Scores of different
//! retrievers are never added; only ranks enter RRF.
//!
//! Dense vectors live outside SQLite: `vectors.f32` (row-major little-endian float32, L2-normalized,
//! N x dim) + `vectors.ids.json` (row -> chunk id, provider, model id, revision, dimension, query
//! instruction, creation date, sha256 of the f32 file). The `embedding_sets` table records the set
//! that belongs to this catalog (one row; same fields).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use regex::Regex;
use rusqlite::types::Value as SqlValue;
use rusqlite::{params, params_from_iter, Connection, OpenFlags};
use serde_json::{json, Value};
use unicode_normalization::UnicodeNormalization;

use crate::records::{locator_json, Chunk, Finding, Publication};

pub const CATALOG_VERSION: i64 = 1;
pub const DDL: &str = include_str!("../../../schemas/evidence_catalog.sql");

pub fn root() -> PathBuf {
    std::env::var_os("IGNIS_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("artifacts"))
        .join("evidence")
}

pub fn default_db() -> PathBuf {
    root().join("evidence.sqlite")
}

pub fn connect(path: &Path, readonly: bool) -> Result<Connection> {
    let con = if readonly {
        Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("open {} read-only", path.display()))?
    } else {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        let con = Connection::open(path)?;
        con.execute_batch(DDL)?;
        con.execute(
            "INSERT OR IGNORE INTO meta(key, value) VALUES ('catalog_version', ?1)",
            [CATALOG_VERSION.to_string()],
        )?;
        // catalogs created before `embedding_sets.provider` existed (additive; readers never use it)
        let has_provider: bool = con.query_row(
            "SELECT count(*) FROM pragma_table_info('embedding_sets') WHERE name = 'provider'",
            [],
            |r| r.get::<_, i64>(0),
        )? > 0;
        if !has_provider {
            con.execute_batch(
                "ALTER TABLE embedding_sets ADD COLUMN provider TEXT NOT NULL DEFAULT 'local'",
            )?;
        }
        con
    };
    con.execute_batch("PRAGMA foreign_keys = ON")?;
    Ok(con)
}

/// Index/query normalization: NFKC (splits ligatures, folds compatibility forms).
pub fn normalize(text: &str) -> String {
    text.nfkc().collect()
}

fn js<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).expect("json")
}

/// Replace a publication and its chunks (idempotent on the PDF sha256).
pub fn upsert_publication(
    con: &mut Connection,
    p: &Publication,
    chunks: &[Chunk],
) -> Result<usize> {
    let tx = con.transaction()?;
    tx.execute("DELETE FROM chunks WHERE publication_id = ?1", [&p.id])?;
    tx.execute("DELETE FROM publications WHERE id = ?1", [&p.id])?;
    tx.execute(
        "INSERT INTO publications VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
        params![
            p.id, p.source_sha256, p.title, js(&p.authors), p.year, p.ntrs_id, p.doi, p.canonical_url,
            js(&p.investigation_ids), js(&p.families), p.file_name, p.n_pages, p.rights_status, p.license,
            js(&p.describes_runs), p.outcome_bearing as i64, p.ingested_at, p.parser_version
        ],
    )?;
    {
        let mut st = tx.prepare(
            "INSERT INTO chunks(id, publication_id, ordinal, kind, page, char_start, char_end, section, text, text_norm, \
             context_json, locator_json, describes_runs_json, outcome_bearing, evidence_class) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        )?;
        for c in chunks {
            st.execute(params![
                c.id,
                c.publication_id,
                c.ordinal,
                c.kind,
                c.page,
                c.char_start,
                c.char_end,
                c.section,
                c.text,
                normalize(&c.text),
                c.context.to_string(),
                locator_json(&c.locator),
                js(&c.describes_runs),
                c.outcome_bearing as i64,
                c.evidence_class
            ])?;
        }
    }
    tx.commit()?;
    Ok(chunks.len())
}

pub fn add_findings(con: &mut Connection, findings: &[Finding]) -> Result<usize> {
    let tx = con.transaction()?;
    for f in findings {
        if !crate::records::EVIDENCE_TYPES.contains(&f.evidence_type.as_str()) {
            bail!(
                "finding {}: unknown evidence_type {:?}",
                f.id,
                f.evidence_type
            );
        }
        tx.execute(
            "INSERT OR REPLACE INTO findings VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                f.id,
                f.publication_id,
                f.chunk_id,
                f.claim,
                f.population,
                js(&f.conditions),
                f.result,
                f.evidence_type,
                locator_json(&f.locator),
                js(&f.describes_runs),
                f.outcome_bearing as i64,
                js(&f.derivation)
            ],
        )?;
    }
    tx.commit()?;
    Ok(findings.len())
}

fn token_rx() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[\w][\w\-\.]*").unwrap())
}

/// Free text -> FTS5 MATCH expression: quoted tokens joined by OR (BM25 does the ranking).
/// Quoting neutralizes FTS5 operators in user text (AND/OR/NEAR/column filters/quotes).
pub fn fts_query(text: &str) -> Result<String> {
    let norm = normalize(text);
    let toks: Vec<String> = token_rx()
        .find_iter(&norm)
        .map(|m| {
            m.as_str()
                .trim_matches(|c| c == '.' || c == '-')
                .to_string()
        })
        .filter(|t| !t.is_empty())
        .collect();
    if toks.is_empty() {
        bail!("empty query");
    }
    Ok(toks
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" OR "))
}

/// Metadata filters applied inside the SQL (same names as the Python catalog).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Filters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year_min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year_max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub investigation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_bearing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rights_status: Option<String>,
}

impl Filters {
    pub fn is_empty(&self) -> bool {
        self.clauses().0.is_empty()
    }

    fn clauses(&self) -> (Vec<&'static str>, Vec<SqlValue>) {
        let mut c = Vec::new();
        let mut a = Vec::new();
        let mut s = |sql: &'static str, v: Option<SqlValue>| {
            if let Some(v) = v {
                c.push(sql);
                a.push(v);
            }
        };
        s(
            "c.publication_id = ?",
            self.publication_id.clone().map(SqlValue::Text),
        );
        s("c.kind = ?", self.kind.clone().map(SqlValue::Text));
        s("c.page >= ?", self.page_min.map(SqlValue::Integer));
        s("c.page <= ?", self.page_max.map(SqlValue::Integer));
        s("p.year >= ?", self.year_min.map(SqlValue::Integer));
        s("p.year <= ?", self.year_max.map(SqlValue::Integer));
        s(
            "EXISTS (SELECT 1 FROM json_each(p.families_json) WHERE value = ?)",
            self.family.clone().map(SqlValue::Text),
        );
        s(
            "EXISTS (SELECT 1 FROM json_each(p.investigation_ids_json) WHERE value = ?)",
            self.investigation.clone().map(SqlValue::Text),
        );
        s(
            "c.outcome_bearing = ?",
            self.outcome_bearing.map(|b| SqlValue::Integer(b as i64)),
        );
        s(
            "p.rights_status = ?",
            self.rights_status.clone().map(SqlValue::Text),
        );
        (c, a)
    }

    fn sql(&self) -> (String, Vec<SqlValue>) {
        let (c, a) = self.clauses();
        (c.iter().map(|x| format!(" AND {x}")).collect(), a)
    }
}

#[derive(Debug, Clone)]
pub struct LexHit {
    pub chunk_id: String,
    pub bm25: f64,
    pub lexical_rank: usize,
}

/// FTS5 BM25, best first. `bm25` is FTS5's raw value (negative); `lexical_rank` starts at 1.
pub fn lexical(
    con: &Connection,
    query: &str,
    filters: &Filters,
    limit: usize,
) -> Result<Vec<LexHit>> {
    let (where_, mut args) = filters.sql();
    let sql = format!(
        "SELECT c.id, bm25(chunks_fts) FROM chunks_fts JOIN chunks c ON c.rowid = chunks_fts.rowid \
         JOIN publications p ON p.id = c.publication_id WHERE chunks_fts MATCH ?{where_} \
         ORDER BY bm25(chunks_fts) ASC, c.id ASC LIMIT ?"
    );
    args.insert(0, SqlValue::Text(fts_query(query)?));
    args.push(SqlValue::Integer(limit as i64));
    let mut st = con.prepare(&sql)?;
    let rows = st.query_map(params_from_iter(args), |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
    })?;
    let mut out = Vec::new();
    for (i, r) in rows.enumerate() {
        let (chunk_id, bm25) = r?;
        out.push(LexHit {
            chunk_id,
            bm25,
            lexical_rank: i + 1,
        });
    }
    Ok(out)
}

/// Chunk ids passing `filters` (None = no filter).
pub fn allowed_ids(con: &Connection, filters: &Filters) -> Result<Option<HashSet<String>>> {
    if filters.is_empty() {
        return Ok(None);
    }
    let (where_, args) = filters.sql();
    let mut st = con.prepare(&format!("SELECT c.id FROM chunks c JOIN publications p ON p.id = c.publication_id WHERE 1=1{where_}"))?;
    let ids = st
        .query_map(params_from_iter(args), |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<HashSet<_>>>()?;
    Ok(Some(ids))
}

fn parse_json(s: Option<String>) -> Value {
    s.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null)
}

/// Chunk rows (with publication title/year/ntrs_id/canonical_url) keyed by chunk id.
pub fn get_chunks(con: &Connection, ids: &[String]) -> Result<HashMap<String, Value>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let q = vec!["?"; ids.len()].join(",");
    let mut st = con.prepare(&format!(
        "SELECT c.id, c.publication_id, c.ordinal, c.kind, c.page, c.char_start, c.char_end, c.section, c.text, \
         c.context_json, c.locator_json, c.describes_runs_json, c.outcome_bearing, c.evidence_class, \
         p.title, p.year, p.ntrs_id, p.canonical_url \
         FROM chunks c JOIN publications p ON p.id = c.publication_id WHERE c.id IN ({q})"
    ))?;
    let rows = st.query_map(params_from_iter(ids.iter()), |r| {
        Ok(json!({
            "id": r.get::<_, String>(0)?, "publication_id": r.get::<_, String>(1)?, "ordinal": r.get::<_, i64>(2)?,
            "kind": r.get::<_, String>(3)?, "page": r.get::<_, i64>(4)?, "char_start": r.get::<_, Option<i64>>(5)?,
            "char_end": r.get::<_, Option<i64>>(6)?, "section": r.get::<_, Option<String>>(7)?, "text": r.get::<_, String>(8)?,
            "context": parse_json(r.get(9)?), "locator": parse_json(r.get(10)?), "describes_runs": parse_json(r.get(11)?),
            "outcome_bearing": r.get::<_, i64>(12)? != 0, "evidence_class": r.get::<_, String>(13)?,
            "title": r.get::<_, Option<String>>(14)?, "year": r.get::<_, Option<i64>>(15)?,
            "ntrs_id": r.get::<_, Option<String>>(16)?, "canonical_url": r.get::<_, Option<String>>(17)?,
        }))
    })?;
    for r in rows {
        let v = r?;
        out.insert(v["id"].as_str().unwrap_or_default().to_string(), v);
    }
    Ok(out)
}

/// `(chunk_id, text)` for every chunk, in catalog order.
pub fn all_chunk_texts(con: &Connection) -> Result<Vec<(String, String)>> {
    let mut st = con.prepare("SELECT c.id, c.text FROM chunks c JOIN publications p ON p.id = c.publication_id ORDER BY c.rowid")?;
    let rows = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

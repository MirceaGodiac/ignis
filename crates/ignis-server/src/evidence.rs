//! Lexical evidence search over `artifacts/evidence/evidence.sqlite` (FTS5, read-only).
//!
//! Built against `schemas/evidence_catalog.sql` (owned by the evidence lane; layout v1:
//! `publications`, `chunks(rowid, id, publication_id, page, char_start, char_end, section, text,
//! locator_json, describes_runs_json, outcome_bearing, evidence_class, ...)` and the external-content
//! FTS5 table `chunks_fts`). Table and column names are discovered from `sqlite_master` /
//! `pragma_table_info`, so the runtime does not keep a second copy of the DDL and also accepts the
//! simpler `chunks(id, source_id, ..., describes_runs)` + FTS5 layout.

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::types::ValueRef;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use serde_json::Value;

use crate::refusal::Refusal;

pub const HIT_OPEN: &str = "\u{2}";
pub const HIT_CLOSE: &str = "\u{3}";

struct Open {
    conn: Connection,
    fts: String,
    sql: String,
}

pub struct Evidence {
    pub path: PathBuf,
    conn: Mutex<Option<Open>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub chunk_id: Value,
    /// `publication_id` (catalog v1) or `source_id`.
    pub source_id: Value,
    pub title: Value,
    pub canonical_url: Value,
    pub page: Value,
    pub char_start: Value,
    pub char_end: Value,
    pub section: Value,
    /// `SourceLocator` JSON when the catalog stores one.
    pub locator: Value,
    /// FTS5 snippet; matched terms are wrapped in U+0002 … U+0003.
    pub snippet: String,
    pub outcome_bearing: Value,
    pub describes_runs: Value,
    pub evidence_class: Value,
    /// bm25 (lower is better), as FTS5 reports it.
    pub bm25: f64,
}

#[derive(Debug, Serialize)]
pub struct CatalogStatus {
    pub path: String,
    pub available: bool,
    pub detail: String,
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn to_json(v: ValueRef<'_>) -> Value {
    match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::from(i),
        ValueRef::Real(f) => Value::from(f),
        ValueRef::Text(t) => {
            let s = String::from_utf8_lossy(t).to_string();
            serde_json::from_str::<Value>(&s)
                .ok()
                .filter(|v| v.is_array() || v.is_object())
                .unwrap_or(Value::String(s))
        }
        ValueRef::Blob(_) => Value::Null,
    }
}

/// Free text -> FTS5 query of quoted terms joined by OR (bm25 ranks); no FTS syntax passes through.
pub fn fts_query(text: &str) -> Option<String> {
    let terms: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.chars().count() >= 2)
        .take(24)
        .map(|t| format!("\"{}\"", t.to_lowercase()))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" OR "))
}

fn columns(conn: &Connection, table: &str) -> Vec<String> {
    conn.prepare("SELECT name FROM pragma_table_info(?1)")
        .and_then(|mut s| s.query_map([table], |r| r.get(0))?.collect())
        .unwrap_or_default()
}

impl Evidence {
    pub fn new(path: PathBuf) -> Self {
        Evidence {
            path,
            conn: Mutex::new(None),
        }
    }

    fn open(&self) -> Result<Open, String> {
        if !self.path.exists() {
            return Err(format!(
                "{} does not exist (built by `ignis evidence`)",
                self.path.display()
            ));
        }
        let conn = Connection::open_with_flags(
            &self.path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| e.to_string())?;
        let tables: Vec<(String, String)> = conn
            .prepare("SELECT name, sql FROM sqlite_master WHERE type = 'table' AND sql LIKE '%USING fts5%'")
            .and_then(|mut s| s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect())
            .map_err(|e| e.to_string())?;
        let chunk_cols = columns(&conn, "chunks");
        if chunk_cols.is_empty() {
            return Err("catalog has no `chunks` table".into());
        }
        let (fts, fts_sql) = tables
            .iter()
            .find(|(n, _)| n == "chunks_fts")
            .or_else(|| tables.iter().find(|(_, sql)| sql.contains("chunks")))
            .cloned()
            .ok_or("catalog has no FTS5 table over chunks")?;
        let fts_cols = columns(&conn, &fts);
        let f = quote_ident(&fts);
        let join = if fts_cols.iter().any(|c| c == "chunk_id") {
            format!("c.id = {f}.chunk_id")
        } else if fts_cols.iter().any(|c| c == "id") {
            format!("c.id = {f}.id")
        } else if fts_sql.contains("content") {
            format!("c.rowid = {f}.rowid")
        } else {
            return Err(format!("cannot determine how FTS table {fts} joins chunks"));
        };
        let has = |c: &str| chunk_cols.iter().any(|x| x == c);
        let col = |names: &[&str]| {
            names
                .iter()
                .find(|n| has(n))
                .map(|n| format!("c.{n}"))
                .unwrap_or_else(|| "NULL".into())
        };
        let pubs = has("publication_id") && !columns(&conn, "publications").is_empty();
        let (pub_join, title, url) = if pubs {
            (
                " LEFT JOIN publications p ON p.id = c.publication_id",
                "p.title",
                "p.canonical_url",
            )
        } else {
            ("", "NULL", "NULL")
        };
        let sql = format!(
            "SELECT {id}, {src}, {title}, {url}, {page}, {cs}, {ce}, {sec}, {loc}, \
             snippet({f}, 0, char(2), char(3), '…', 24), {ob}, {dr}, {ec}, bm25({f}) AS s \
             FROM {f} JOIN chunks c ON {join}{pub_join} WHERE {f} MATCH ?1 {{filter}} ORDER BY s LIMIT ?2",
            id = col(&["id"]),
            src = col(&["publication_id", "source_id"]),
            page = col(&["page"]),
            cs = col(&["char_start"]),
            ce = col(&["char_end"]),
            sec = col(&["section"]),
            loc = col(&["locator_json", "locator"]),
            ob = col(&["outcome_bearing"]),
            dr = col(&["describes_runs_json", "describes_runs"]),
            ec = col(&["evidence_class"]),
        );
        if !has("outcome_bearing") {
            return Err(
                "chunks has no outcome_bearing column; the runtime will not guess it".into(),
            );
        }
        Ok(Open { conn, fts, sql })
    }

    pub fn status(&self) -> CatalogStatus {
        let mut g = self.conn.lock().unwrap();
        let path = self.path.display().to_string();
        if let Some(o) = g.as_ref() {
            return CatalogStatus {
                path,
                available: true,
                detail: format!("fts table {}", o.fts),
            };
        }
        match self.open() {
            Ok(o) => {
                let detail = format!("fts table {}", o.fts);
                *g = Some(o);
                CatalogStatus {
                    path,
                    available: true,
                    detail,
                }
            }
            Err(e) => CatalogStatus {
                path,
                available: false,
                detail: e,
            },
        }
    }

    /// Blocking; call from `spawn_blocking`.
    pub fn search(
        &self,
        text: &str,
        limit: usize,
        outcome_bearing: Option<bool>,
    ) -> Result<Vec<Hit>, Refusal> {
        let q = fts_query(text)
            .ok_or_else(|| Refusal::invalid("empty_query", "query has no searchable terms"))?;
        let mut g = self.conn.lock().unwrap();
        if g.is_none() {
            *g = Some(
                self.open()
                    .map_err(|e| Refusal::unavailable("evidence_catalog_unavailable", e))?,
            );
        }
        let o = g.as_ref().unwrap();
        let filter = match outcome_bearing {
            Some(true) => "AND c.outcome_bearing = 1",
            Some(false) => "AND (c.outcome_bearing = 0 OR c.outcome_bearing IS NULL)",
            None => "",
        };
        let sql = o.sql.replace("{filter}", filter);
        let mut stmt = o
            .conn
            .prepare(&sql)
            .map_err(|e| Refusal::failed("evidence_query", e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![q, limit as i64], |r| {
                Ok(Hit {
                    chunk_id: to_json(r.get_ref(0)?),
                    source_id: to_json(r.get_ref(1)?),
                    title: to_json(r.get_ref(2)?),
                    canonical_url: to_json(r.get_ref(3)?),
                    page: to_json(r.get_ref(4)?),
                    char_start: to_json(r.get_ref(5)?),
                    char_end: to_json(r.get_ref(6)?),
                    section: to_json(r.get_ref(7)?),
                    locator: to_json(r.get_ref(8)?),
                    snippet: r.get::<_, Option<String>>(9)?.unwrap_or_default(),
                    outcome_bearing: to_json(r.get_ref(10)?),
                    describes_runs: to_json(r.get_ref(11)?),
                    evidence_class: to_json(r.get_ref(12)?),
                    bm25: r.get(13)?,
                })
            })
            .map_err(|e| Refusal::failed("evidence_query", e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| Refusal::failed("evidence_query", e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Catalog layout v1 (subset of `schemas/evidence_catalog.sql`: same tables, view and FTS5 setup).
    /// When the evidence lane's DDL is present in the checkout it is used instead.
    const V1_SUBSET: &str = "
        CREATE TABLE publications (id TEXT PRIMARY KEY, title TEXT, canonical_url TEXT);
        CREATE TABLE chunks (rowid INTEGER PRIMARY KEY, id TEXT NOT NULL UNIQUE, publication_id TEXT NOT NULL,
          page INTEGER NOT NULL, char_start INTEGER, char_end INTEGER, section TEXT, text TEXT NOT NULL,
          text_norm TEXT NOT NULL, locator_json TEXT NOT NULL, describes_runs_json TEXT NOT NULL DEFAULT '[]',
          outcome_bearing INTEGER NOT NULL DEFAULT 1, evidence_class TEXT NOT NULL);
        CREATE VIRTUAL TABLE chunks_fts USING fts5(text, section, title UNINDEXED, content='chunks_fts_src',
          content_rowid='rowid', tokenize='porter unicode61 remove_diacritics 2');
        CREATE VIEW chunks_fts_src AS SELECT c.rowid AS rowid, c.text_norm AS text, c.section AS section,
          p.title AS title FROM chunks c JOIN publications p ON p.id = c.publication_id;
        CREATE TRIGGER chunks_ai AFTER INSERT ON chunks BEGIN
          INSERT INTO chunks_fts(rowid, text, section, title) VALUES (new.rowid, new.text_norm, new.section,
          (SELECT title FROM publications WHERE id = new.publication_id)); END;";

    fn ddl() -> String {
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../schemas/evidence_catalog.sql");
        std::fs::read_to_string(repo).unwrap_or_else(|_| V1_SUBSET.to_string())
    }

    #[test]
    fn search_against_catalog_layout() {
        let dir = std::env::temp_dir().join(format!("ignis-ev-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("evidence.sqlite");
        let _ = std::fs::remove_file(&path);
        {
            let c = Connection::open(&path).unwrap();
            c.execute_batch(&ddl()).unwrap();
            let cols = columns(&c, "publications");
            // Insert only the columns this test knows; NOT NULL columns of the full DDL get values too.
            let mut names = vec!["id", "title", "canonical_url"];
            let mut vals = vec!["'pub-1'", "'Droplet flames'", "'https://ntrs.nasa.gov/x'"];
            for (n, v) in [
                ("source_sha256", "'aa'"),
                ("n_pages", "3"),
                ("rights_status", "'unverified'"),
                ("ingested_at", "'2026-11-14'"),
                ("parser_version", "'1'"),
            ] {
                if cols.iter().any(|c| c == n) {
                    names.push(n);
                    vals.push(v);
                }
            }
            c.execute(
                &format!(
                    "INSERT INTO publications({}) VALUES({})",
                    names.join(","),
                    vals.join(",")
                ),
                [],
            )
            .unwrap();
            let ccols = columns(&c, "chunks");
            let extra = |n: &str| ccols.iter().any(|c| c == n);
            for (id, page, text, ob, runs) in [
                ("k1", 3, "oxygen controls flame persistence", 1, "[\"r1\"]"),
                ("k2", 4, "igniter retraction timing", 0, "[]"),
            ] {
                let (mut n, mut v) = (
                    vec![
                        "id",
                        "publication_id",
                        "page",
                        "char_start",
                        "char_end",
                        "section",
                        "text",
                        "text_norm",
                        "locator_json",
                        "describes_runs_json",
                        "outcome_bearing",
                        "evidence_class",
                    ],
                    vec![
                        format!("'{id}'"),
                        "'pub-1'".into(),
                        page.to_string(),
                        "0".into(),
                        "30".into(),
                        "'Results'".into(),
                        format!("'{text}'"),
                        format!("'{text}'"),
                        "'{\"asset_sha256\":\"aa\",\"page\":3}'".into(),
                        format!("'{runs}'"),
                        ob.to_string(),
                        "'source_derived'".into(),
                    ],
                );
                for (k, val) in [("ordinal", "0"), ("kind", "'text'")] {
                    if extra(k) {
                        n.push(k);
                        v.push(val.into());
                    }
                }
                c.execute(
                    &format!(
                        "INSERT INTO chunks({}) VALUES({})",
                        n.join(","),
                        v.join(",")
                    ),
                    [],
                )
                .unwrap();
            }
        }
        let ev = Evidence::new(path);
        assert!(ev.status().available, "{}", ev.status().detail);
        let hits = ev.search("What does oxygen do?", 5, None).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk_id, Value::from("k1"));
        assert_eq!(hits[0].source_id, Value::from("pub-1"));
        assert_eq!(hits[0].title, Value::from("Droplet flames"));
        assert_eq!(hits[0].describes_runs, serde_json::json!(["r1"]));
        assert!(hits[0].snippet.contains(HIT_OPEN));
        assert!(ev.search("oxygen", 5, Some(false)).unwrap().is_empty());
        assert!(Evidence::new(dir.join("missing.sqlite"))
            .search("x y", 1, None)
            .is_err());
    }
}

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

use crate::content_class;
use crate::records::{locator_json, Chunk, Finding, Publication};

/// 2: parse-view page texts (`pages`) and `asset_locations`. 3: several parse views per
/// publication (`pages.parse_view_id`); search covers the active one.
pub const CATALOG_VERSION: i64 = 3;

/// SQL condition: chunk `c` belongs to the active view of publication `p` (chunk ids start with
/// their view id, `pv-<sha16>@<parser_version>:`).
/// Ids without the `pv-` prefix are legacy single-view chunks and count as active.
pub const ACTIVE_VIEW: &str = "(substr(c.id, 1, 3) <> 'pv-' OR \
     substr(c.id, 1, length('pv-' || substr(p.id, 5) || '@' || p.parser_version) + 1) \
     = 'pv-' || substr(p.id, 5) || '@' || p.parser_version || ':')";
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
        migrate_pages_v3(&con)?;
        con.execute_batch(DDL)?;
        con.execute(
            "INSERT OR REPLACE INTO meta(key, value) VALUES ('catalog_version', ?1)",
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
        // catalogs created before `chunks.continues_runs_json` (v1): additive
        let has_cont: bool = con.query_row(
            "SELECT count(*) FROM pragma_table_info('chunks') WHERE name = 'continues_runs_json'",
            [],
            |r| r.get::<_, i64>(0),
        )? > 0;
        if !has_cont {
            con.execute_batch(
                "ALTER TABLE chunks ADD COLUMN continues_runs_json TEXT NOT NULL DEFAULT '[]'",
            )?;
        }
        // catalogs created before `chunks.content_class`: additive, every chunk `body` until
        // `ignis evidence reclassify` derives the class from the stored section and text
        if !has_content_class(&con)? {
            con.execute_batch(
                "ALTER TABLE chunks ADD COLUMN content_class TEXT NOT NULL DEFAULT 'body'",
            )?;
        }
        con
    };
    con.execute_batch("PRAGMA foreign_keys = ON")?;
    Ok(con)
}

/// Catalogs before v3 kept one page text per (publication, page): move them under the view that
/// was active (`publications.parser_version`), so their citations keep resolving.
fn migrate_pages_v3(con: &Connection) -> Result<()> {
    let has_pages: bool = con.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'pages'",
        [],
        |r| r.get::<_, i64>(0),
    )? > 0;
    if !has_pages {
        return Ok(());
    }
    let has_view: bool = con.query_row(
        "SELECT count(*) FROM pragma_table_info('pages') WHERE name = 'parse_view_id'",
        [],
        |r| r.get::<_, i64>(0),
    )? > 0;
    if has_view {
        return Ok(());
    }
    con.execute_batch(
        "BEGIN; \
         CREATE TABLE pages_v3 (publication_id TEXT NOT NULL REFERENCES publications(id) ON DELETE CASCADE, \
           parse_view_id TEXT NOT NULL, page INTEGER NOT NULL, text TEXT NOT NULL, \
           PRIMARY KEY (publication_id, parse_view_id, page)); \
         INSERT INTO pages_v3 SELECT g.publication_id, 'pv-' || substr(g.publication_id, 5) || '@' || p.parser_version, \
           g.page, g.text FROM pages g JOIN publications p ON p.id = g.publication_id; \
         DROP TABLE pages; ALTER TABLE pages_v3 RENAME TO pages; COMMIT;",
    )?;
    Ok(())
}

/// Whether the catalog stores `chunks.content_class`. A catalog written before it and opened
/// read-only is searched without the demotion until `ignis evidence reclassify` runs on it.
pub fn has_content_class(con: &Connection) -> Result<bool> {
    Ok(con.query_row(
        "SELECT count(*) FROM pragma_table_info('chunks') WHERE name = 'content_class'",
        [],
        |r| r.get::<_, i64>(0),
    )? > 0)
}

/// Recompute `chunks.content_class` of every stored chunk (every view) from its section, kind and
/// text ([`content_class::classify`]); no PDF is parsed. Neither the FTS index nor the corpus
/// snapshot depends on the class, so both stay as they are. Returns the chunks per class and how
/// many changed.
pub fn reclassify(con: &mut Connection) -> Result<Value> {
    let tx = con.transaction()?;
    let rows: Vec<(i64, Option<String>, String, String, String, String)> = {
        let mut st = tx.prepare(
            "SELECT c.rowid, c.section, c.kind, c.text, c.content_class, p.parser_version              FROM chunks c JOIN publications p ON p.id = c.publication_id ORDER BY c.rowid",
        )?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };
    let mut per: std::collections::BTreeMap<&str, usize> = Default::default();
    let mut changed = 0usize;
    {
        let mut up = tx.prepare("UPDATE chunks SET content_class = ?1 WHERE rowid = ?2")?;
        for (rowid, section, kind, text, old, parser) in &rows {
            let c = class_of(parser, section.as_deref(), kind, text);
            *per.entry(c).or_default() += 1;
            if c != old {
                up.execute(params![c, rowid])?;
                changed += 1;
            }
        }
    }
    tx.commit()?;
    Ok(json!({"chunks": rows.len(), "classes": per, "changed": changed}))
}

/// A chunk's content class. A PSI investigation record is written one fact per line on purpose
/// (DOI, size, hardware): every line is a finding, never a fragment to rank down.
fn class_of(parser_version: &str, section: Option<&str>, kind: &str, text: &str) -> &'static str {
    if parser_version.starts_with("psi-record-") {
        return content_class::BODY;
    }
    content_class::classify(section, kind, text)
}

/// What ranking needs of a chunk besides its scores.
#[derive(Debug, Clone)]
pub struct ChunkShape {
    pub publication_id: String,
    pub page: i64,
    /// length of the text in characters
    pub chars: i64,
    /// `body` for a catalog without the column
    pub content_class: String,
}

/// [`ChunkShape`] of the given chunk ids (absent ids are left out).
pub fn chunk_shapes(con: &Connection, ids: &[String]) -> Result<HashMap<String, ChunkShape>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let class = if has_content_class(con)? {
        "content_class"
    } else {
        "'body'"
    };
    for part in ids.chunks(500) {
        let q = vec!["?"; part.len()].join(",");
        let mut st = con.prepare(&format!(
            "SELECT id, publication_id, page, length(text), {class} FROM chunks WHERE id IN ({q})"
        ))?;
        let rows = st.query_map(params_from_iter(part.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                ChunkShape {
                    publication_id: r.get(1)?,
                    page: r.get(2)?,
                    chars: r.get(3)?,
                    content_class: r.get(4)?,
                },
            ))
        })?;
        for r in rows {
            let (id, s) = r?;
            out.insert(id, s);
        }
    }
    Ok(out)
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
    // Only this view's chunks are replaced; other views of the publication stay (their citations
    // resolve). The row is updated in place: deleting it would cascade to every view.
    let prefix = format!(
        "{}:",
        crate::records::parse_view_id(&p.id, &p.parser_version)
    );
    tx.execute(
        "DELETE FROM chunks WHERE publication_id = ?1 AND substr(id, 1, length(?2)) = ?2",
        params![p.id, prefix],
    )?;
    tx.execute(
        "INSERT INTO publications VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18) \
         ON CONFLICT(id) DO UPDATE SET source_sha256=excluded.source_sha256, title=excluded.title, \
         authors_json=excluded.authors_json, year=excluded.year, ntrs_id=excluded.ntrs_id, doi=excluded.doi, \
         canonical_url=excluded.canonical_url, investigation_ids_json=excluded.investigation_ids_json, \
         families_json=excluded.families_json, file_name=excluded.file_name, n_pages=excluded.n_pages, \
         rights_status=excluded.rights_status, license=excluded.license, \
         describes_runs_json=excluded.describes_runs_json, outcome_bearing=excluded.outcome_bearing, \
         ingested_at=excluded.ingested_at, parser_version=excluded.parser_version",
        params![
            p.id, p.source_sha256, p.title, js(&p.authors), p.year, p.ntrs_id, p.doi, p.canonical_url,
            js(&p.investigation_ids), js(&p.families), p.file_name, p.n_pages, p.rights_status, p.license,
            js(&p.describes_runs), p.outcome_bearing as i64, p.ingested_at, p.parser_version
        ],
    )?;
    {
        let mut st = tx.prepare(
            "INSERT INTO chunks(id, publication_id, ordinal, kind, page, char_start, char_end, section, text, text_norm, \
             context_json, locator_json, describes_runs_json, outcome_bearing, evidence_class, continues_runs_json, \
             content_class) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
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
                c.evidence_class,
                js(&c.continues_runs),
                class_of(&p.parser_version, c.section.as_deref(), &c.kind, &c.text)
            ])?;
        }
    }
    tx.commit()?;
    Ok(chunks.len())
}

/// Replace a publication with its full parse view: chunks, the exact page texts, and where this host
/// keeps the bytes (`rel_path`, a cache hint; readers verify the hash).
pub fn upsert_parse_view(
    con: &mut Connection,
    p: &Publication,
    chunks: &[Chunk],
    pages: &[String],
    rel_path: Option<&str>,
) -> Result<usize> {
    let n = upsert_publication(con, p, chunks)?;
    let tx = con.transaction()?;
    let view = crate::records::parse_view_id(&p.id, &p.parser_version);
    tx.execute(
        "DELETE FROM pages WHERE publication_id = ?1 AND parse_view_id = ?2",
        params![p.id, view],
    )?;
    {
        let mut st = tx.prepare(
            "INSERT INTO pages(publication_id, parse_view_id, page, text) VALUES (?1, ?2, ?3, ?4)",
        )?;
        for (i, t) in pages.iter().enumerate() {
            st.execute(params![p.id, view, (i + 1) as i64, t])?;
        }
    }
    if let Some(r) = rel_path {
        tx.execute(
            "INSERT OR REPLACE INTO asset_locations(sha256, rel_path) VALUES (?1, ?2)",
            params![p.source_sha256, r],
        )?;
    }
    tx.commit()?;
    Ok(n)
}

/// Every publication record, ordered by id.
pub fn publications(con: &Connection) -> Result<Vec<Publication>> {
    let mut st = con.prepare(
        "SELECT id, source_sha256, title, authors_json, year, ntrs_id, doi, canonical_url, investigation_ids_json, \
         families_json, file_name, n_pages, rights_status, license, describes_runs_json, outcome_bearing, ingested_at, \
         parser_version FROM publications ORDER BY id",
    )?;
    let arr = |s: String| serde_json::from_str::<Vec<String>>(&s).unwrap_or_default();
    let rows = st.query_map([], |r| {
        Ok(Publication {
            id: r.get(0)?,
            source_sha256: r.get(1)?,
            title: r.get(2)?,
            authors: arr(r.get(3)?),
            year: r.get(4)?,
            ntrs_id: r.get(5)?,
            doi: r.get(6)?,
            canonical_url: r.get(7)?,
            investigation_ids: arr(r.get(8)?),
            families: arr(r.get(9)?),
            file_name: r.get(10)?,
            n_pages: r.get(11)?,
            rights_status: r.get(12)?,
            license: r.get(13)?,
            describes_runs: arr(r.get(14)?),
            outcome_bearing: r.get::<_, i64>(15)? != 0,
            ingested_at: r.get(16)?,
            parser_version: r.get(17)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Stored page texts `[first, last]` (1-based) of a publication's parse view.
pub fn page_texts(
    con: &Connection,
    publication_id: &str,
    view: &str,
    first: i64,
    last: i64,
) -> Result<Vec<(i64, String)>> {
    let mut st = con.prepare(
        "SELECT page, text FROM pages WHERE publication_id = ?1 AND parse_view_id = ?2 \
         AND page BETWEEN ?3 AND ?4 ORDER BY page",
    )?;
    let rows = st
        .query_map(params![publication_id, view, first, last], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Ids of the chunks of `publication_id` on `page`, in reading order.
pub fn chunk_ids_on_page(
    con: &Connection,
    publication_id: &str,
    view: &str,
    page: i64,
) -> Result<Vec<String>> {
    let mut st = con.prepare(
        "SELECT id FROM chunks WHERE publication_id = ?1 AND page = ?2 \
         AND substr(id, 1, length(?3)) = ?3 ORDER BY ordinal",
    )?;
    let rows = st
        .query_map(params![publication_id, page, format!("{view}:")], |r| {
            r.get(0)
        })?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(rows)
}

/// The parse views of a publication that are stored (any of them can be read).
/// The parser version of the publication's active view, if the publication is stored.
pub fn active_parser(con: &Connection, publication_id: &str) -> Result<Option<String>> {
    use rusqlite::OptionalExtension;
    Ok(con
        .query_row(
            "SELECT parser_version FROM publications WHERE id = ?1",
            [publication_id],
            |r| r.get(0),
        )
        .optional()?)
}

/// Store a parse view **without** making it active: its chunks and page texts are written (they
/// read and resolve like any stored view), the publication row and its active view stay as they are.
pub fn add_inactive_view(
    con: &mut Connection,
    p: &Publication,
    chunks: &[Chunk],
    pages: &[String],
) -> Result<usize> {
    anyhow::ensure!(
        active_parser(con, &p.id)?.is_some(),
        "{}: no stored publication to add an inactive view to",
        p.id
    );
    let view = crate::records::parse_view_id(&p.id, &p.parser_version);
    let prefix = format!("{view}:");
    let tx = con.transaction()?;
    tx.execute(
        "DELETE FROM chunks WHERE publication_id = ?1 AND substr(id, 1, length(?2)) = ?2",
        params![p.id, prefix],
    )?;
    tx.execute(
        "DELETE FROM pages WHERE publication_id = ?1 AND parse_view_id = ?2",
        params![p.id, view],
    )?;
    {
        let mut st = tx.prepare(
            "INSERT INTO chunks(id, publication_id, ordinal, kind, page, char_start, char_end, section, text, text_norm, \
             context_json, locator_json, describes_runs_json, outcome_bearing, evidence_class, continues_runs_json, \
             content_class) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
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
                c.evidence_class,
                js(&c.continues_runs),
                class_of(&p.parser_version, c.section.as_deref(), &c.kind, &c.text)
            ])?;
        }
        let mut st = tx.prepare(
            "INSERT INTO pages(publication_id, parse_view_id, page, text) VALUES (?1, ?2, ?3, ?4)",
        )?;
        for (i, t) in pages.iter().enumerate() {
            st.execute(params![p.id, view, (i + 1) as i64, t])?;
        }
    }
    tx.commit()?;
    Ok(chunks.len())
}

pub fn views(con: &Connection, publication_id: &str) -> Result<Vec<String>> {
    let mut st = con.prepare(
        "SELECT DISTINCT parse_view_id FROM pages WHERE publication_id = ?1 ORDER BY parse_view_id",
    )?;
    let rows = st
        .query_map([publication_id], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(rows)
}

/// Where this host keeps the bytes of `sha256`, if recorded at ingest.
/// Where the bytes of `sha256` are kept on this host (a cache hint; readers verify the hash).
pub fn record_location(con: &Connection, sha256: &str, rel_path: &str) -> Result<()> {
    con.execute(
        "INSERT OR REPLACE INTO asset_locations(sha256, rel_path) VALUES (?1, ?2)",
        params![sha256, rel_path],
    )?;
    Ok(())
}

pub fn asset_location(con: &Connection, sha256: &str) -> Result<Option<String>> {
    use rusqlite::OptionalExtension;
    Ok(con
        .query_row(
            "SELECT rel_path FROM asset_locations WHERE sha256 = ?1",
            [sha256],
            |r| r.get(0),
        )
        .optional()?)
}

/// The file a recorded location names on this host. A location is written by whichever process
/// ingested the bytes, with that process's own mount of the data directory (the server sees it at
/// `/opt/ignis/data`, the data-build container at `/data`); when the path as recorded does not
/// exist, the part from its `work/` or `raw/` component on is looked up under this host's
/// `IGNIS_DATA`. Callers still check the bytes against the hash.
pub fn resolve_location(rel_path: &str) -> PathBuf {
    resolve_location_in(&ignis_data::paths::root(), rel_path)
}

fn resolve_location_in(root: &Path, rel_path: &str) -> PathBuf {
    let recorded = PathBuf::from(rel_path);
    if recorded.exists() {
        return recorded;
    }
    let norm = rel_path.replace('\\', "/");
    let parts: Vec<&str> = norm.split('/').collect();
    for (i, part) in parts.iter().enumerate().rev() {
        if *part == "work" || *part == "raw" {
            let mut c = root.to_path_buf();
            for x in &parts[i..] {
                c.push(x);
            }
            if c.exists() {
                return c;
            }
        }
    }
    recorded
}

/// The embedding set recorded for this catalog (sha256 of its vector file), if any.
pub fn embedding_set_id(con: &Connection) -> Result<Option<String>> {
    use rusqlite::OptionalExtension;
    Ok(con
        .query_row(
            "SELECT id FROM embedding_sets ORDER BY id LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?)
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
    /// any of these evidence families (`flex`, `acme`, `bass`, ...)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub families: Vec<String>,
    /// any of these investigations (`PSI-25`, ...)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub investigations: Vec<String>,
    /// any of these assets (bare sha256 of the PDF bytes)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_sha256: Vec<String>,
    /// content classes ranked with the body ([`content_class::tier`]; by default glossary rows and
    /// fragments follow the body, reference lists and contents pages follow those). Not a filter:
    /// nothing is excluded either way.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ranked_with_body: Vec<String>,
}

impl Filters {
    pub fn is_empty(&self) -> bool {
        self.clauses().0.is_empty()
    }

    fn clauses(&self) -> (Vec<String>, Vec<SqlValue>) {
        let mut c: Vec<String> = Vec::new();
        let mut a = Vec::new();
        // a list filter is one clause: OR within the list, AND with the other filters
        let mut any = |sql: &str, vals: &[String]| {
            if !vals.is_empty() {
                c.push(format!("({})", vec![sql; vals.len()].join(" OR ")));
                a.extend(vals.iter().cloned().map(SqlValue::Text));
            }
        };
        any(
            "EXISTS (SELECT 1 FROM json_each(p.families_json) WHERE value = ?)",
            &self.families,
        );
        any(
            "EXISTS (SELECT 1 FROM json_each(p.investigation_ids_json) WHERE value = ?)",
            &self.investigations,
        );
        any("p.source_sha256 = ?", &self.source_sha256);
        let mut s = |sql: &'static str, v: Option<SqlValue>| {
            if let Some(v) = v {
                c.push(sql.to_string());
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

/// A passage of a document a family filter excludes only because the document carries no family
/// label, with what the lockbox needs to decide whether a scope may see it.
#[derive(Debug, Clone)]
pub struct UnlabelledMatch {
    pub publication_id: String,
    pub outcome_bearing: bool,
    pub describes_runs: Vec<String>,
    pub continues_runs: Vec<String>,
    pub investigation_ids: Vec<String>,
}

/// Passages whose text matches `query` and every filter except the family ones, in publications
/// with an empty `families_json`: what a family filter leaves out only for the missing label. Empty
/// when no family filter is set. The caller applies the scope's lockbox before counting.
pub fn unlabelled_family_matches(
    con: &Connection,
    query: &str,
    filters: &Filters,
) -> Result<Vec<UnlabelledMatch>> {
    if filters.families.is_empty() && filters.family.is_none() {
        return Ok(vec![]);
    }
    let mut f = filters.clone();
    f.families.clear();
    f.family = None;
    let (where_, fargs) = f.sql();
    let sql = format!(
        "SELECT c.publication_id, c.outcome_bearing, c.describes_runs_json, c.continues_runs_json,          p.investigation_ids_json FROM chunks_fts JOIN chunks c ON c.rowid = chunks_fts.rowid          JOIN publications p ON p.id = c.publication_id          WHERE chunks_fts MATCH ? AND coalesce(json_array_length(p.families_json), 0) = 0{where_}"
    );
    let mut args = vec![SqlValue::Text(fts_query(query)?)];
    args.extend(fargs);
    let arr = |s: String| serde_json::from_str::<Vec<String>>(&s).unwrap_or_default();
    let mut st = con.prepare(&sql)?;
    let rows = st.query_map(rusqlite::params_from_iter(args), |r| {
        Ok(UnlabelledMatch {
            publication_id: r.get(0)?,
            outcome_bearing: r.get::<_, i64>(1)? != 0,
            describes_runs: arr(r.get(2)?),
            continues_runs: arr(r.get(3)?),
            investigation_ids: arr(r.get(4)?),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
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
    lexical_from(con, query, filters, 0, limit)
}

/// [`lexical`] from position `offset` (0-based) of the BM25 order; ranks continue (`offset + 1`, ...).
/// Ties are broken by chunk id, so consecutive windows partition the full order.
pub fn lexical_from(
    con: &Connection,
    query: &str,
    filters: &Filters,
    offset: usize,
    limit: usize,
) -> Result<Vec<LexHit>> {
    lexical_expanded(con, query, &[], filters, offset, limit)
}

/// [`lexical_from`] with SIRA expansion (crates/ignis-evidence/src/sira.rs). The score of a chunk is
///
/// `bm25(text; query) + w_doc * bm25(expansion; query) + sum_e w_e * bm25(text; e)`
///
/// - the document-side expansion (`expansion_fts`, filled by `ignis evidence enrich`) is matched by
///   the query words with weight [`crate::sira::DOC_EXPANSION_WEIGHT`]; an empty table adds nothing;
/// - each query-side expression `e` (the finalizer's choice from the agent's expected terms) is
///   matched as a phrase in the chunk text with its weight `w_e` (1.0 = a query word).
///
/// FTS5's bm25 is negative (more negative = more relevant), so a chunk matched by several parts
/// sums them; ties break by chunk id, so pages still partition one order.
pub fn lexical_expanded(
    con: &Connection,
    query: &str,
    expressions: &[(String, f64)],
    filters: &Filters,
    offset: usize,
    limit: usize,
) -> Result<Vec<LexHit>> {
    lexical_weighted(
        con,
        query,
        expressions,
        crate::sira::DOC_EXPANSION_WEIGHT,
        filters,
        offset,
        limit,
    )
}

/// [`lexical_expanded`] with the document-expansion weight `doc_weight` as an argument (the τ/w
/// sweep of `ignis evidence retrieval-eval`). `doc_weight = 0` leaves `expansion_fts` out of the
/// query altogether, so a chunk matched only by its expansion is not returned: plain BM25.
pub fn lexical_weighted(
    con: &Connection,
    query: &str,
    expressions: &[(String, f64)],
    doc_weight: f64,
    filters: &Filters,
    offset: usize,
    limit: usize,
) -> Result<Vec<LexHit>> {
    let (where_, fargs) = filters.sql();
    let q = fts_query(query)?;
    let mut parts = vec![
        "SELECT rowid AS rid, bm25(chunks_fts) AS s FROM chunks_fts WHERE chunks_fts MATCH ?"
            .to_string(),
    ];
    let mut args = vec![SqlValue::Text(q.clone())];
    if doc_weight != 0.0 {
        parts.push("SELECT rowid AS rid, ? * bm25(expansion_fts) AS s FROM expansion_fts WHERE expansion_fts MATCH ?".to_string());
        args.push(SqlValue::Real(doc_weight));
        args.push(SqlValue::Text(q));
    } else {
        // keep the union compound: SQLite would flatten a single SELECT into the outer SUM(),
        // where bm25() may not be called
        parts.push("SELECT rowid AS rid, 0.0 AS s FROM chunks_fts WHERE 0".to_string());
    }
    for (e, w) in expressions {
        let Ok(phrase) = crate::sira::phrase_query(e) else {
            continue;
        };
        parts.push("SELECT rowid AS rid, ? * bm25(chunks_fts) AS s FROM chunks_fts WHERE chunks_fts MATCH ?".into());
        args.push(SqlValue::Real(*w));
        args.push(SqlValue::Text(phrase));
    }
    // Body matches first, then glossary rows and fragments, then reference lists and contents
    // pages (a page of references is dense with topic words and would outrank the finding), in
    // BM25 order within each tier. One total order, so consecutive windows still partition it.
    let demote = if has_content_class(con)? {
        format!(
            "{} ASC, ",
            content_class::tier_sql("c.content_class", &filters.ranked_with_body)
        )
    } else {
        String::new()
    };
    let sql = format!(
        "SELECT c.id, SUM(u.s) AS score FROM ({}) u JOIN chunks c ON c.rowid = u.rid          JOIN publications p ON p.id = c.publication_id WHERE {ACTIVE_VIEW}{where_}          GROUP BY c.id ORDER BY {demote}score ASC, c.id ASC LIMIT ? OFFSET ?",
        parts.join(" UNION ALL ")
    );
    args.extend(fargs);
    args.push(SqlValue::Integer(limit as i64));
    args.push(SqlValue::Integer(offset as i64));
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
            lexical_rank: offset + i + 1,
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
    let mut st = con.prepare(&format!("SELECT c.id FROM chunks c JOIN publications p ON p.id = c.publication_id WHERE {ACTIVE_VIEW}{where_}"))?;
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

/// `(chunk_id, text)` for every chunk of an active view, in catalog order.
pub fn all_chunk_texts(con: &Connection) -> Result<Vec<(String, String)>> {
    let mut st = con.prepare(&format!(
        "SELECT c.id, c.text FROM chunks c JOIN publications p ON p.id = c.publication_id \
         WHERE {ACTIVE_VIEW} ORDER BY c.rowid"
    ))?;
    let rows = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

#[cfg(test)]
mod location_tests {
    use super::resolve_location_in;

    #[test]
    fn a_location_recorded_under_another_mount_resolves_under_this_data_root() {
        let root = std::env::temp_dir().join(format!("ignis-loc-{}", std::process::id()));
        let pdf = root.join("work/raw/samples/PSI-21/18249 velocity traces.pdf");
        std::fs::create_dir_all(pdf.parent().unwrap()).unwrap();
        std::fs::write(&pdf, b"%PDF").unwrap();
        let ntrs = root.join("raw/evidence/ntrs/NTRS-1.pdf");
        std::fs::create_dir_all(ntrs.parent().unwrap()).unwrap();
        std::fs::write(&ntrs, b"%PDF").unwrap();
        let server = "/opt/ignis/data/work/raw/samples/PSI-21/18249 velocity traces.pdf";
        assert_eq!(resolve_location_in(&root, server), pdf);
        assert_eq!(
            resolve_location_in(&root, "/elsewhere/raw/evidence/ntrs/NTRS-1.pdf"),
            ntrs
        );
        // nothing on this host: the recorded path comes back, so the error names it
        assert_eq!(
            resolve_location_in(&root, "/opt/x/work/missing.pdf"),
            std::path::PathBuf::from("/opt/x/work/missing.pdf")
        );
        std::fs::remove_dir_all(&root).ok();
    }
}

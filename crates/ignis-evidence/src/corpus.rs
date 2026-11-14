//! Corpus manifest and snapshots (IGNIS_MASTER_V2 §10.3–10.4, §11.1).
//!
//! The corpus is a selective list of works (`configs/evidence/corpus.toml`), not a mirror of the PSI
//! archives. Each seed says why it is included (or why it was rejected). [`status`] derives the state
//! of every seed from facts — local bytes and their hash, the NTRS citation record, the PSI manifest,
//! the catalog — and never from a hand-typed field:
//!
//! | status | meaning |
//! |---|---|
//! | `discovered` | listed, nothing fetched (or local bytes contradict the manifest) |
//! | `metadata_only` | citation record only: no downloadable bytes, or not fetched yet |
//! | `downloaded` | bytes present and hashed, no parse view in the catalog |
//! | `parsed` | in the catalog under an older parser version (re-ingest to refresh) |
//! | `indexed` | parse view of the current parser version in the catalog (FTS-indexed) |
//! | `rejected` | excluded, with the reason |
//! | `restricted` | rights or access forbid ingesting the full text |
//!
//! A [`CorpusSnapshot`] lists what the catalog contains (source, parse view, metadata used by the
//! filters, chunk count, embedding set). Its id is the hash of that list, so the same catalog always
//! has the same id and any change that can change a search answer changes it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use ignis_core::hashing::{canonical_json, file_sha256, sha256_hex};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::catalog;
use crate::records::parse_view_id;
use crate::PARSER_VERSION;

pub const NTRS_API: &str = "https://ntrs.nasa.gov";
/// Generic client name: no personal or project contact details are sent to external services.
pub const USER_AGENT: &str = "ignis-evidence/0.3 (read-only research client)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Collection {
    ScienceResults,
    ExperimentalMethods,
    Measurements,
    IgnisMethods,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusStatus {
    Discovered,
    MetadataOnly,
    Downloaded,
    Parsed,
    Indexed,
    Rejected,
    Restricted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seed {
    pub id: String,
    #[serde(default)]
    pub ntrs_id: Option<String>,
    #[serde(default)]
    pub psi_file: Option<String>,
    pub collection: Collection,
    #[serde(default)]
    pub families: Vec<String>,
    #[serde(default)]
    pub investigations: Vec<String>,
    /// why the work is included (a rejected seed gives its reason in `rejected`)
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub rejected: Option<String>,
    /// reviewed declaration, with its evidence, that no passage can reveal a run outcome
    /// (e.g. written before the flights); ingest then marks every chunk `outcome_bearing = false`
    #[serde(default)]
    pub outcome_free: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedFile {
    seed: Vec<Seed>,
}

pub fn default_seeds_path() -> PathBuf {
    ignis_core::resources::configs_dir()
        .join("evidence")
        .join("corpus.toml")
}

/// Committed manifest of the corpus (`data/corpus/manifest.jsonl`) and its snapshots.
pub fn corpus_dir() -> PathBuf {
    ignis_data::paths::root().join("corpus")
}

pub fn load_seeds(path: &Path) -> Result<Vec<Seed>> {
    let f: SeedFile = toml::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))?;
    let mut ids = BTreeSet::new();
    for s in &f.seed {
        if !ids.insert(&s.id) {
            bail!("corpus seed {} listed twice", s.id);
        }
        if s.ntrs_id.is_some() == s.psi_file.is_some() {
            bail!("corpus seed {}: exactly one of ntrs_id / psi_file", s.id);
        }
        if s.psi_file.is_some() && s.investigations.is_empty() {
            bail!("corpus seed {}: a PSI file needs its investigation", s.id);
        }
        let why = s.rejected.as_deref().unwrap_or(&s.reason);
        if why.trim().is_empty() {
            bail!(
                "corpus seed {}: inclusion (reason) or exclusion (rejected) needs a reason",
                s.id
            );
        }
    }
    Ok(f.seed)
}

pub fn ntrs_dir() -> PathBuf {
    ignis_data::paths::root()
        .join("raw")
        .join("evidence")
        .join("ntrs")
}

pub fn ntrs_pdf(id: &str) -> PathBuf {
    ntrs_dir().join(format!("NTRS-{id}.pdf"))
}

/// The NTRS citation record, saved next to the PDF (the parser reads it as its sidecar).
pub fn ntrs_meta(id: &str) -> PathBuf {
    ntrs_dir().join(format!("NTRS-{id}.json"))
}

/// Where the seed's bytes live on this host (whether or not they are present).
pub fn local_path(seed: &Seed) -> PathBuf {
    match (&seed.ntrs_id, &seed.psi_file) {
        (Some(n), _) => ntrs_pdf(n),
        (_, Some(f)) => {
            ignis_data::paths::raw(seed.investigations.first().map(String::as_str)).join(f)
        }
        _ => unreachable!("load_seeds checks one of ntrs_id / psi_file"),
    }
}

/// One row of `data/corpus/manifest.jsonl`. Absent fields are unknown, never empty defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEntry {
    pub entry_id: String,
    pub status: CorpusStatus,
    pub collection: Collection,
    pub families: Vec<String>,
    pub investigations: Vec<String>,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ntrs_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub psi_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// `sha256:<hex>` of the bytes (the `Source.id`), when they were obtained and hashed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rights_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_view_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_pages: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_chunks: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// What the PSI manifest (bytes) and listing (inventory) say about a PSI file.
fn psi_record(inv: &str, file: &str) -> Result<(Option<Value>, Option<Value>)> {
    let find = |p: PathBuf| -> Result<Option<Value>> {
        if !p.exists() {
            return Ok(None);
        }
        for l in std::fs::read_to_string(&p)?.lines() {
            let v: Value = serde_json::from_str(l).with_context(|| format!("{}", p.display()))?;
            if v["file_name"].as_str() == Some(file) {
                return Ok(Some(v));
            }
        }
        Ok(None)
    };
    let m = ignis_data::paths::manifests();
    Ok((
        find(m.join(format!("{inv}.jsonl")))?,
        find(m.join(format!("{inv}.listing.jsonl")))?,
    ))
}

fn ntrs_record(id: &str) -> Result<Option<Value>> {
    let p = ntrs_meta(id);
    if !p.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&std::fs::read_to_string(&p)?)?))
}

/// The downloadable PDF link of an NTRS citation record, if any.
pub fn ntrs_pdf_link(rec: &Value) -> Option<String> {
    if rec["downloadsAvailable"].as_bool() == Some(false) {
        return None;
    }
    rec["downloads"].as_array()?.iter().find_map(|d| {
        let pdf = d["mimetype"].as_str() == Some("application/pdf");
        let link = d["links"]["pdf"]
            .as_str()
            .or(pdf.then(|| d["links"]["original"].as_str()).flatten())?;
        Some(if link.starts_with("http") {
            link.to_string()
        } else {
            format!("{NTRS_API}{link}")
        })
    })
}

/// Derive the manifest row of every seed. `con` is the catalog (None: nothing is indexed yet).
pub fn status(seeds: &[Seed], con: Option<&Connection>) -> Result<Vec<CorpusEntry>> {
    let pubs = match con {
        Some(c) => catalog::publications(c)?,
        None => Vec::new(),
    };
    let mut out = Vec::new();
    for s in seeds {
        let mut e = CorpusEntry {
            entry_id: s.id.clone(),
            status: CorpusStatus::Discovered,
            collection: s.collection,
            families: s.families.clone(),
            investigations: s.investigations.clone(),
            reason: s.rejected.clone().unwrap_or_else(|| s.reason.clone()),
            ntrs_id: s.ntrs_id.clone(),
            psi_file: s.psi_file.clone(),
            title: None,
            source_id: None,
            size_bytes: None,
            rights_status: None,
            license: None,
            canonical_url: None,
            parse_view_id: None,
            n_pages: None,
            n_chunks: None,
            note: None,
        };
        let mut restricted = false;
        let mut expected_sha: Option<String> = None;
        if let Some(n) = &s.ntrs_id {
            e.canonical_url = Some(format!("{NTRS_API}/citations/{n}"));
            if let Some(rec) = ntrs_record(n)? {
                e.status = CorpusStatus::MetadataOnly;
                e.title = rec["title"].as_str().map(String::from);
                let det = rec["copyright"]["determinationType"].as_str();
                e.license = det.map(String::from);
                e.rights_status = Some(
                    if det.is_some_and(|d| ignis_data::sources::ntrs::PUBLIC_USE.contains(&d)) {
                        "verified"
                    } else {
                        "unverified"
                    }
                    .into(),
                );
                if ntrs_pdf_link(&rec).is_none() {
                    e.note = Some("NTRS lists no downloadable PDF".into());
                }
            }
        }
        if let (Some(f), Some(inv)) = (&s.psi_file, s.investigations.first()) {
            let (manifest, listing) = psi_record(inv, f)?;
            if let Some(m) = &manifest {
                expected_sha = m["id"]
                    .as_str()
                    .map(|x| x.trim_start_matches("sha256:").to_string());
                e.title = m["title"].as_str().map(String::from);
                e.canonical_url = m["canonical_url"].as_str().map(String::from);
                e.rights_status = m["rights"]["status"].as_str().map(String::from);
                e.license = m["rights"]["license"].as_str().map(String::from);
            }
            if let Some(l) = &listing {
                restricted = l["restricted"].as_bool() == Some(true);
                e.canonical_url = e
                    .canonical_url
                    .or_else(|| l["stable_url"].as_str().map(String::from));
                if manifest.is_none() {
                    e.status = CorpusStatus::MetadataOnly;
                }
            }
        }
        let path = local_path(s);
        if path.exists() {
            let sha = file_sha256(&path)?;
            match &expected_sha {
                Some(x) if *x != sha => {
                    e.status = CorpusStatus::Discovered;
                    e.note = Some(format!(
                        "local bytes {sha} differ from the PSI manifest id {x}; not used"
                    ));
                }
                _ => {
                    e.status = CorpusStatus::Downloaded;
                    e.size_bytes = Some(std::fs::metadata(&path)?.len());
                    e.source_id = Some(format!("sha256:{sha}"));
                    if let Some(p) = pubs.iter().find(|p| p.source_sha256 == sha) {
                        e.title = p.title.clone().or(e.title);
                        e.n_pages = Some(p.n_pages);
                        e.parse_view_id = Some(parse_view_id(&p.id, &p.parser_version));
                        e.n_chunks = Some(chunk_count(con.unwrap(), &p.id)?);
                        if p.parser_version == PARSER_VERSION {
                            e.status = CorpusStatus::Indexed;
                        } else {
                            e.status = CorpusStatus::Parsed;
                            e.note = Some(format!(
                                "parse view of parser {} (current {PARSER_VERSION}); re-ingest",
                                p.parser_version
                            ));
                        }
                    }
                }
            }
        }
        if restricted {
            e.status = CorpusStatus::Restricted;
        }
        if s.rejected.is_some() {
            e.status = CorpusStatus::Rejected;
        }
        out.push(e);
    }
    Ok(out)
}

fn chunk_count(con: &Connection, publication_id: &str) -> Result<i64> {
    Ok(con.query_row(
        "SELECT count(*) FROM chunks WHERE publication_id = ?1",
        [publication_id],
        |r| r.get(0),
    )?)
}

/// Write the manifest rows as JSONL (stable order and fields: diffs show real changes only).
pub fn write_manifest(path: &Path, rows: &[CorpusEntry]) -> Result<()> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut s = String::new();
    for r in rows {
        s.push_str(&serde_json::to_string(r)?);
        s.push('\n');
    }
    std::fs::write(path, s)?;
    Ok(())
}

// ------------------------------------------------------------------ snapshots

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotMember {
    pub source_id: String,
    pub publication_id: String,
    pub parse_view_id: String,
    pub families: Vec<String>,
    pub investigations: Vec<String>,
    pub rights_status: String,
    pub n_pages: i64,
    pub n_chunks: i64,
    /// sha256 of every chunk's span, text, section, attribution and outcome flag, in order: run
    /// patterns or a seed declaration change what a scope may read without a new parser version
    pub chunks_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// The catalog content a search runs against. `snapshot_id` = `corpus-<sha16>` of everything else.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusSnapshot {
    pub snapshot_id: String,
    pub catalog_version: i64,
    pub members: Vec<SnapshotMember>,
    /// sha256 of the vector file whose rows embed these chunks (absent: lexical only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding_set: Option<String>,
}

fn chunks_digest(con: &Connection, publication_id: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    let mut st = con.prepare(
        "SELECT id, kind, page, char_start, char_end, section, text, locator_json, describes_runs_json, \
         continues_runs_json, outcome_bearing, evidence_class FROM chunks WHERE publication_id = ?1 ORDER BY ordinal",
    )?;
    let mut rows = st.query([publication_id])?;
    while let Some(r) = rows.next()? {
        let row = json!([
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, Option<i64>>(3)?,
            r.get::<_, Option<i64>>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
            r.get::<_, String>(8)?,
            r.get::<_, String>(9)?,
            r.get::<_, i64>(10)?,
            r.get::<_, String>(11)?
        ]);
        h.update(row.to_string().as_bytes());
        h.update(b"\n");
    }
    Ok(hex::encode(h.finalize()))
}

pub fn snapshot(con: &Connection) -> Result<CorpusSnapshot> {
    let mut members = Vec::new();
    for p in catalog::publications(con)? {
        members.push(SnapshotMember {
            source_id: format!("sha256:{}", p.source_sha256),
            parse_view_id: parse_view_id(&p.id, &p.parser_version),
            families: p.families.clone(),
            investigations: p.investigation_ids.clone(),
            rights_status: p.rights_status.clone(),
            n_pages: p.n_pages,
            n_chunks: chunk_count(con, &p.id)?,
            chunks_digest: chunks_digest(con, &p.id)?,
            title: p.title.clone(),
            publication_id: p.id,
        });
    }
    let embedding_set = catalog::embedding_set_id(con)?;
    let body = json!({"catalog_version": catalog::CATALOG_VERSION, "members": members, "embedding_set": embedding_set});
    let snapshot_id = format!(
        "corpus-{}",
        &sha256_hex(canonical_json(&body).as_bytes())[..16]
    );
    Ok(CorpusSnapshot {
        snapshot_id,
        catalog_version: catalog::CATALOG_VERSION,
        members,
        embedding_set,
    })
}

// ------------------------------------------------------------------ fetch (NTRS)

/// Fetch the citation record and, when NTRS offers one, the PDF of `ntrs_id` (network; callers
/// gate it behind `--execute`). Existing files are kept; the PDF must start with `%PDF`.
pub fn fetch_ntrs(http: &ignis_data::sources::http::Http, ntrs_id: &str) -> Result<Value> {
    let meta = ntrs_meta(ntrs_id);
    let rec: Value = if meta.exists() {
        serde_json::from_str(&std::fs::read_to_string(&meta)?)?
    } else {
        let url = format!("{NTRS_API}/api/citations/{ntrs_id}");
        let got = http.get(&url, &[])?;
        let v: Value = serde_json::from_slice(&got.bytes)
            .with_context(|| format!("{url}: not a JSON citation record"))?;
        std::fs::create_dir_all(ntrs_dir())?;
        std::fs::write(&meta, serde_json::to_string_pretty(&v)? + "\n")?;
        v
    };
    let pdf = ntrs_pdf(ntrs_id);
    let Some(link) = ntrs_pdf_link(&rec) else {
        return Ok(
            json!({"ntrs_id": ntrs_id, "metadata": true, "pdf": null, "note": "no downloadable PDF"}),
        );
    };
    if !pdf.exists() {
        let st = http.stream_to(&link, &pdf, None)?;
        let mut head = [0u8; 4];
        use std::io::Read;
        std::fs::File::open(&pdf)?.read_exact(&mut head).ok();
        if &head != b"%PDF" {
            std::fs::remove_file(&pdf)?;
            bail!("{link}: response is not a PDF ({:?})", st.content_type);
        }
    }
    Ok(
        json!({"ntrs_id": ntrs_id, "metadata": true, "pdf": pdf.display().to_string(), "url": link,
        "sha256": file_sha256(&pdf)?, "bytes": std::fs::metadata(&pdf)?.len()}),
    )
}

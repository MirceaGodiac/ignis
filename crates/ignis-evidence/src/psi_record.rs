//! PSI investigation records as citable sources.
//!
//! NASA's Physical Sciences Informatics repository describes every investigation in a metadata
//! record (title, acronym, DOI, version, licence, ISS hardware, investigators, dates, objectives)
//! and a file listing. `ignis sources inspect` saves both (`data/manifests/<inv>.investigation.json`,
//! `data/manifests/<inv>.listing.jsonl`, and the raw response `data/raw/samples/_meta/<inv>.json`).
//! [`render`] turns them into a plain-text record, one fact per line, in a few sections (one page
//! each); [`ingest`] stores each record as a publication of the evidence catalog, so
//! `sources_search` finds "the DOI of FLEX" or "the size of the FLEX-2 dataset" and the answer can
//! cite the record like any report passage.
//!
//! * The record's bytes are its UTF-8 text, pages separated by U+000C; the source id is their
//!   sha256 and the parse view is `pv-<sha16>@psi-record-<n>`. Rendering reads no clock, so the same
//!   inputs always give the same ids; re-ingest replaces the previous record of the investigation.
//! * Chunks are whole pages (`page_text[0..len]`), `evidence_class = source_derived`,
//!   `outcome_bearing = false`: the record states what NASA published about the investigation, never
//!   how a run ended, so the lockbox never withholds it. Free-text fields that can summarise results
//!   (`researchImpacts`) are not rendered; contact e-mails and phone numbers are never rendered.
//! * The raw metadata response is used only when its sha256 is the one the investigation manifest
//!   cites (`metadata_url_sha256`); otherwise the record keeps the manifest's fields alone.
//! * `records.jsonl` next to the bytes maps each record's source id to the PSI landing page (the
//!   server's source catalog opens it as a link-out).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignis_core::contracts::{OffsetUnit, SourceLocator};
use ignis_core::hashing::sha256_hex;
use ignis_data::sources::psi::{self, RemoteFile};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

use crate::catalog;
use crate::records::{parse_view_id, Chunk, Publication};

/// Parser version of record views (bump when the rendering changes).
pub const RECORD_PARSER: &str = "psi-record-1";
/// Page separator of the record bytes.
pub const PAGE_BREAK: char = '\u{c}';
/// File name of the source-id → landing-page map next to the record bytes.
pub const RECORDS_MANIFEST: &str = "records.jsonl";

/// Where record bytes and `records.jsonl` are written (`data/work/evidence/psi_records`).
pub fn default_dir() -> PathBuf {
    ignis_data::paths::work(&["evidence", "psi_records"])
}

/// The public landing page of an investigation.
pub fn landing_url(investigation: &str) -> String {
    format!(
        "https://psi.nasa.gov/physci/repo/data/investigations/{}",
        psi::quote(investigation)
    )
}

pub fn is_record(p: &Publication) -> bool {
    p.parser_version.starts_with("psi-record-")
}

/// A rendered record: title and (section, page text) pairs.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub investigation: String,
    pub title: String,
    pub doi: Option<String>,
    pub license: Option<String>,
    pub year: Option<i64>,
    pub pages: Vec<(String, String)>,
}

impl Record {
    pub fn bytes(&self) -> Vec<u8> {
        let mut s = String::new();
        for (i, (_, t)) in self.pages.iter().enumerate() {
            if i > 0 {
                s.push(PAGE_BREAK);
            }
            s.push_str(t);
        }
        s.into_bytes()
    }
}

fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|x| !x.is_empty())
}

/// `1234567` → `1,234,567`.
fn thousands(n: u64) -> String {
    let d = n.to_string();
    let mut out = String::new();
    for (i, c) in d.chars().enumerate() {
        if i > 0 && (d.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// `1224752113090` → `1,224.75 GB (1.22 TB; 1,224,752,113,090 bytes)` (decimal units, as PSI).
fn size_text(bytes: u64) -> String {
    let gb = bytes as f64 / 1e9;
    let whole = gb.trunc() as u64;
    let frac = ((gb - gb.trunc()) * 100.0).round() as u64;
    let (whole, frac) = if frac >= 100 {
        (whole + 1, 0)
    } else {
        (whole, frac)
    };
    let gb_s = format!("{}.{frac:02} GB", thousands(whole));
    let other = if bytes >= 1_000_000_000_000 {
        Some(format!("{:.2} TB", bytes as f64 / 1e12))
    } else if bytes < 1_000_000_000 && bytes >= 1_000_000 {
        Some(format!("{:.2} MB", bytes as f64 / 1e6))
    } else if bytes < 1_000_000 {
        Some(format!("{:.1} kB", bytes as f64 / 1e3))
    } else {
        None
    };
    match other {
        Some(o) => format!("{gb_s} ({o}; {} bytes)", thousands(bytes)),
        None => format!("{gb_s} ({} bytes)", thousands(bytes)),
    }
}

/// PSI dates: `08-01-2007` (MM-DD-YYYY) or `31-Jul-2016`; ISO 8601 with the original when parseable.
fn date_text(raw: &str) -> String {
    let iso = chrono::NaiveDate::parse_from_str(raw, "%m-%d-%Y")
        .or_else(|_| chrono::NaiveDate::parse_from_str(raw, "%d-%b-%Y"))
        .ok();
    match iso {
        Some(d) => format!("{} (PSI: {raw})", d.format("%Y-%m-%d")),
        None => raw.to_string(),
    }
}

fn year_of(raw: &str) -> Option<i64> {
    use chrono::Datelike;
    chrono::NaiveDate::parse_from_str(raw, "%m-%d-%Y")
        .or_else(|_| chrono::NaiveDate::parse_from_str(raw, "%d-%b-%Y"))
        .ok()
        .map(|d| d.year() as i64)
}

fn one_line(t: &str) -> String {
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Render the record of `investigation` from its investigation manifest (`info`, required), the raw
/// PSI metadata response (`raw`, optional) and the file listing (`listing`, optional).
pub fn render(
    investigation: &str,
    info: &Value,
    raw: Option<&Value>,
    listing: Option<&[RemoteFile]>,
) -> Record {
    let acronym = s(info, "acronym")
        .or_else(|| raw.and_then(|r| s(r, "investigationAcronym")))
        .unwrap_or(investigation)
        .to_string();
    let name = s(info, "title")
        .or_else(|| raw.and_then(|r| s(r, "title")))
        .unwrap_or("")
        .to_string();
    let label = if name.is_empty() {
        format!("{investigation} {acronym}")
    } else {
        format!("{investigation} {acronym} ({name})")
    };
    let title = format!("NASA PSI investigation record: {label}");
    let header = |section: &str| {
        format!("NASA Physical Sciences Informatics (PSI) investigation record, {section}\nInvestigation: {label}\n")
    };
    let raw_s = |k: &str| raw.and_then(|r| s(r, k));
    let mut pages: Vec<(String, String)> = Vec::new();
    let mut push = |section: &str, lines: Vec<String>| {
        if lines.is_empty() {
            return;
        }
        let mut t = header(section);
        for l in lines {
            t.push_str(&l);
            t.push('\n');
        }
        pages.push((format!("PSI record: {section}"), t));
    };

    // identity and citation (the summary page)
    let doi = s(info, "doi").map(String::from);
    let license = s(info, "license").map(String::from);
    let mut id = vec![
        format!("PSI investigation number (accession): {investigation}"),
        format!("Investigation acronym: {acronym}"),
    ];
    if !name.is_empty() {
        id.push(format!("Investigation title: {name}"));
    }
    if let Some(d) = &doi {
        id.push(format!("Dataset DOI: {d}"));
        id.push(format!("DOI link: https://doi.org/{d}"));
    }
    if let Some(v) = s(info, "investigation_version") {
        let why = s(info, "version_reason")
            .map(|r| format!(" ({r})"))
            .unwrap_or_default();
        id.push(format!("Dataset (investigation) version at PSI: {v}{why}"));
    }
    if let Some(m) = s(info, "modified") {
        id.push(format!("Last modified at PSI: {}", date_text(m)));
    }
    if let Some(l) = &license {
        let lname = raw_s("licenseName")
            .map(|n| format!(" ({n})"))
            .unwrap_or_default();
        let link = s(info, "license_link")
            .map(|u| format!(", {u}"))
            .unwrap_or_default();
        id.push(format!("Licence: {l}{lname}{link}"));
    } else {
        id.push("Licence: not stated in the PSI metadata".into());
    }
    id.push(format!("PSI landing page: {}", landing_url(investigation)));
    id.push(format!(
        "PSI metadata record: {}",
        psi::metadata_url(investigation)
    ));
    for (k, label) in [
        ("proposalTitle", "Proposal title"),
        ("funding", "Funding (grant)"),
        ("projectTitle", "Project title"),
    ] {
        if let Some(v) = raw_s(k) {
            id.push(format!("{label}: {}", one_line(v)));
        }
    }

    // hardware and programme
    let mut hw = Vec::new();
    if let Some(h) = s(info, "hardware").or_else(|| raw_s("experimentHardware")) {
        hw.push(format!("ISS hardware / apparatus: {}", one_line(h)));
    }
    for (k, label) in [
        ("experimentHardwareLink", "Hardware links"),
        ("flightProgram", "Flight program (platform)"),
        ("spaceProgram", "Space program"),
        ("managingNasaCenter", "Managing NASA center"),
        ("researchArea", "Research area"),
        ("subResearchArea", "Research sub-area"),
        ("projectType", "Project type"),
        ("missionName", "Mission"),
    ] {
        if let Some(v) = raw_s(k) {
            hw.push(format!("{label}: {}", one_line(v)));
        }
    }

    // people and dates
    let mut people = Vec::new();
    if let Some(cs) = raw
        .and_then(|r| r.get("contacts"))
        .and_then(Value::as_array)
    {
        // principal investigators first, then the order PSI lists
        let mut rows: Vec<(u8, String)> = Vec::new();
        for c in cs {
            let nm = ["firstName", "midInitials", "lastName"]
                .iter()
                .filter_map(|k| s(c, k))
                .collect::<Vec<_>>()
                .join(" ");
            if nm.is_empty() {
                continue;
            }
            let roles: Vec<&str> = c
                .get("roles")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|r| s(r, "annotationValue"))
                .collect();
            let role = if roles.is_empty() {
                "Contact".to_string()
            } else {
                roles.join(", ")
            };
            let aff = s(c, "affiliation")
                .map(|a| format!(", {a}"))
                .unwrap_or_default();
            let pi = roles.iter().any(|r| r.contains("Principal Investigator"));
            rows.push((u8::from(!pi), format!("{role}: {nm}{aff}")));
        }
        rows.sort_by_key(|r| r.0);
        people.extend(rows.into_iter().map(|r| r.1));
    }
    for (k, label) in [
        ("investigationStartDate", "Investigation start date"),
        ("investigationEndDate", "Investigation end date"),
        ("missionStart", "Mission start"),
        ("missionEnd", "Mission end"),
        ("submissionDate", "Data submission date"),
        ("releaseDate", "Data release date"),
    ] {
        if let Some(v) = raw_s(k) {
            people.push(format!("{label}: {}", date_text(v)));
        }
    }

    // objectives (stated before or independent of the flights; results summaries are not rendered)
    let mut obj = Vec::new();
    for (k, label) in [
        ("objective", "Objective"),
        ("hypothesis", "Hypothesis"),
        ("approach", "Approach"),
        ("description", "Description"),
    ] {
        if let Some(v) = raw_s(k) {
            obj.push(format!("{label}: {}", one_line(v)));
        }
    }

    // files and size, computed from the listing
    let mut files = Vec::new();
    if let Some(l) = listing {
        let total: u64 = l
            .iter()
            .filter_map(|f| f.size_bytes)
            .map(|b| b.max(0) as u64)
            .sum();
        files.push(format!(
            "Total dataset size (sum of the PSI file listing): {} in {} files",
            size_text(total),
            thousands(l.len() as u64)
        ));
        let unknown = l.iter().filter(|f| f.size_bytes.is_none()).count();
        if unknown > 0 {
            files.push(format!(
                "Files without a stated size: {}",
                thousands(unknown as u64)
            ));
        }
        let restricted = l.iter().filter(|f| f.restricted).count();
        files.push(format!(
            "Restricted (not public) files: {}",
            thousands(restricted as u64)
        ));
        let mut by: BTreeMap<&str, (u64, u64)> = BTreeMap::new();
        for f in l {
            let e = by
                .entry(if f.category.is_empty() {
                    "(no category)"
                } else {
                    f.category.as_str()
                })
                .or_default();
            e.0 += 1;
            e.1 += f.size_bytes.unwrap_or(0).max(0) as u64;
        }
        for (cat, (n, b)) in by {
            files.push(format!(
                "Category {cat}: {} files, {}",
                thousands(n),
                size_text(b)
            ));
        }
    }
    if let Some(t) = info.get("total_file_size_bytes").and_then(Value::as_f64) {
        files.push(format!(
            "Total file size stated by PSI metadata: {}",
            size_text(t.max(0.0) as u64)
        ));
    }

    // publications PSI lists for the investigation
    let mut publs = Vec::new();
    if let Some(ps) = raw
        .and_then(|r| r.get("publications"))
        .and_then(Value::as_array)
        .filter(|p| !p.is_empty())
    {
        publs.push(format!("Publications listed by PSI: {}", ps.len()));
        for p in ps {
            let Some(t) = s(p, "title") else { continue };
            let au = s(p, "authorList")
                .map(|a| format!("{}. ", one_line(a)))
                .unwrap_or_default();
            let doi = s(p, "doi")
                .map(|d| format!(" DOI {d}."))
                .unwrap_or_default();
            publs.push(format!("Publication: {au}{}.{doi}", one_line(t)));
        }
    }

    // The first page summarises the facts most asked for (identity, DOI, hardware, PI, dates, size),
    // so the record's best-ranked passage usually answers on its own; the other pages give detail.
    let pick = |lines: &[String], prefixes: &[&str]| -> Vec<String> {
        lines
            .iter()
            .filter(|l| prefixes.iter().any(|p| l.starts_with(p)))
            .cloned()
            .collect()
    };
    let mut summary = id;
    summary.extend(pick(
        &hw,
        &["ISS hardware / apparatus:", "Managing NASA center:"],
    ));
    summary.extend(pick(
        &people,
        &[
            "Principal Investigator",
            "Investigation start date:",
            "Investigation end date:",
        ],
    ));
    summary.extend(pick(
        &files,
        &[
            "Total dataset size (sum of the PSI file listing):",
            "Total file size stated by PSI metadata:",
        ],
    ));
    push("summary and citation", summary);
    push("hardware and programme", hw);
    push("investigators and dates", people);
    push("objectives and approach", obj);
    push("files and dataset size", files);
    push("publications listed", publs);

    let year = raw_s("releaseDate").and_then(year_of);
    Record {
        investigation: investigation.to_string(),
        title,
        doi,
        license,
        year,
        pages,
    }
}

/// Publication and chunks of a rendered record (`rel_path` = where its bytes are written).
pub fn to_catalog(rec: &Record, ingested_at: &str) -> (Publication, Vec<Chunk>, Vec<String>) {
    let bytes = rec.bytes();
    let sha = sha256_hex(&bytes);
    let pub_id = format!("pub-{}", &sha[..16]);
    let view = parse_view_id(&pub_id, RECORD_PARSER);
    let file_name = format!("{}.psi-record.txt", rec.investigation);
    let rights_status = match rec.license.as_deref() {
        Some("CC0-1.0") | Some("CC-BY-4.0") => "verified",
        None => "unverified",
        Some(_) => "restricted",
    };
    let publ = Publication {
        id: pub_id.clone(),
        source_sha256: sha.clone(),
        title: Some(rec.title.clone()),
        authors: vec!["NASA Physical Sciences Informatics (PSI)".into()],
        year: rec.year,
        ntrs_id: None,
        doi: rec.doi.clone(),
        canonical_url: Some(landing_url(&rec.investigation)),
        investigation_ids: vec![rec.investigation.clone()],
        families: vec![],
        file_name: Some(file_name.clone()),
        n_pages: rec.pages.len() as i64,
        rights_status: rights_status.into(),
        license: rec.license.clone(),
        describes_runs: vec![],
        outcome_bearing: false,
        ingested_at: ingested_at.into(),
        parser_version: RECORD_PARSER.into(),
    };
    let mut chunks = Vec::new();
    let mut texts = Vec::new();
    for (i, (section, text)) in rec.pages.iter().enumerate() {
        let page = i as i64 + 1;
        let end = text.chars().count() as i64;
        chunks.push(Chunk {
            id: format!("{view}:p{page}:{i:05}"),
            publication_id: pub_id.clone(),
            ordinal: i as i64,
            kind: "text".into(),
            page,
            char_start: Some(0),
            char_end: Some(end),
            section: Some(section.clone()),
            text: text.clone(),
            context: json!({}),
            locator: SourceLocator {
                asset_sha256: sha.clone(),
                path: Some(file_name.clone()),
                page: Some(page),
                char_start: Some(0),
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
        texts.push(text.clone());
    }
    (publ, chunks, texts)
}

/// The inputs of one investigation's record, read from `manifests` and `meta_dir`.
pub fn load(investigation: &str, manifests: &Path, meta_dir: &Path) -> Result<(Record, Value)> {
    let ip = manifests.join(format!("{investigation}.investigation.json"));
    let info: Value = serde_json::from_str(&std::fs::read_to_string(&ip).with_context(|| {
        format!(
            "{}: run `ignis sources inspect {investigation}`",
            ip.display()
        )
    })?)
    .with_context(|| format!("parse {}", ip.display()))?;
    let mut report = json!({"investigation": investigation});
    let rp = meta_dir.join(format!("{investigation}.json"));
    let raw = match std::fs::read(&rp) {
        Ok(b) => {
            let want = info["metadata_url_sha256"].as_str().unwrap_or_default();
            if sha256_hex(&b) == want {
                Some(
                    serde_json::from_slice::<Value>(&b)
                        .with_context(|| format!("parse {}", rp.display()))?,
                )
            } else {
                report["raw_metadata"] =
                    json!("not used: its sha256 differs from the investigation manifest");
                None
            }
        }
        Err(_) => {
            report["raw_metadata"] = json!("absent (run `ignis sources inspect`)");
            None
        }
    };
    if raw.is_some() {
        report["raw_metadata"] = json!("used");
    }
    let lp = manifests.join(format!("{investigation}.listing.jsonl"));
    let listing = if lp.exists() {
        Some(psi::read_listing(&lp).with_context(|| format!("read {}", lp.display()))?)
    } else {
        report["listing"] = json!("absent");
        None
    };
    Ok((
        render(investigation, &info, raw.as_ref(), listing.as_deref()),
        report,
    ))
}

/// Remove earlier records of `investigation` other than `keep` (a changed record has a new id).
fn remove_stale(con: &mut Connection, investigation: &str, keep: &str) -> Result<usize> {
    let tx = con.transaction()?;
    let inv = serde_json::to_string(&[investigation])?;
    let ids: Vec<String> = {
        let mut st = tx.prepare(
            "SELECT id FROM publications WHERE parser_version LIKE 'psi-record-%' \
             AND investigation_ids_json = ?1 AND id <> ?2",
        )?;
        let rows = st.query_map(params![inv, keep], |r| r.get(0))?;
        rows.collect::<rusqlite::Result<_>>()?
    };
    for id in &ids {
        tx.execute("DELETE FROM chunks WHERE publication_id = ?1", [id])?;
        tx.execute("DELETE FROM pages WHERE publication_id = ?1", [id])?;
        tx.execute("DELETE FROM findings WHERE publication_id = ?1", [id])?;
        tx.execute("DELETE FROM publications WHERE id = ?1", [id])?;
    }
    tx.commit()?;
    Ok(ids.len())
}

/// Render, write and index the record of every investigation in `investigations`. Writes the bytes
/// and `records.jsonl` under `out_dir`. Idempotent: an unchanged record keeps its ids.
pub fn ingest(
    con: &mut Connection,
    investigations: &[&str],
    manifests: &Path,
    meta_dir: &Path,
    out_dir: &Path,
) -> Result<Vec<Value>> {
    std::fs::create_dir_all(out_dir)?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false);
    let mut out = Vec::new();
    let mut manifest = String::new();
    for inv in investigations {
        let (rec, mut report) = match load(inv, manifests, meta_dir) {
            Ok(x) => x,
            Err(e) => {
                out.push(json!({"investigation": inv, "skipped": format!("{e:#}")}));
                continue;
            }
        };
        let (publ, chunks, pages) = to_catalog(&rec, &now);
        let path = out_dir.join(publ.file_name.as_deref().unwrap_or_default());
        std::fs::write(&path, rec.bytes())?;
        catalog::upsert_parse_view(
            con,
            &publ,
            &chunks,
            &pages,
            Some(&path.display().to_string()),
        )?;
        let removed = remove_stale(con, inv, &publ.id)?;
        let source_id = format!("sha256:{}", publ.source_sha256);
        manifest.push_str(
            &json!({"source_id": source_id, "investigation": inv, "landing_url": landing_url(inv),
                "file_name": publ.file_name, "title": publ.title})
            .to_string(),
        );
        manifest.push('\n');
        report["source_id"] = json!(source_id);
        report["publication_id"] = json!(publ.id);
        report["pages"] = json!(publ.n_pages);
        report["replaced"] = json!(removed);
        out.push(report);
    }
    std::fs::write(out_dir.join(RECORDS_MANIFEST), manifest)?;
    Ok(out)
}

/// Re-derive a stored record view from its bytes: hash, page texts and whole-page chunk spans.
pub fn verify(con: &Connection, p: &Publication, bytes: &[u8]) -> Result<Value> {
    let view = parse_view_id(&p.id, &p.parser_version);
    let bytes_match = sha256_hex(bytes) == p.source_sha256;
    let text = String::from_utf8_lossy(bytes);
    let now: Vec<&str> = text.split(PAGE_BREAK).collect();
    let stored = catalog::page_texts(con, &p.id, &view, 1, i64::MAX)?;
    let page_mismatches: Vec<i64> = stored
        .iter()
        .filter(|(n, t)| now.get(*n as usize - 1) != Some(&t.as_str()))
        .map(|(n, _)| *n)
        .collect();
    let mut st = con.prepare(
        "SELECT id, page, char_start, char_end, text FROM chunks WHERE publication_id = ?1 ORDER BY ordinal",
    )?;
    let rows = st.query_map([&p.id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, Option<i64>>(2)?,
            r.get::<_, Option<i64>>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;
    let mut chunk_mismatches = Vec::new();
    let mut n = 0;
    for row in rows {
        let (id, page, a, b, t) = row?;
        n += 1;
        let span: Option<String> = now.get(page as usize - 1).and_then(|pt| {
            let (a, b) = (a? as usize, b? as usize);
            let c: Vec<char> = pt.chars().collect();
            c.get(a..b).map(|x| x.iter().collect())
        });
        if span.as_deref() != Some(t.as_str()) {
            chunk_mismatches.push(id);
        }
    }
    let passed = bytes_match
        && stored.len() == now.len()
        && page_mismatches.is_empty()
        && chunk_mismatches.is_empty();
    Ok(
        json!({"publication_id": p.id, "source_id": format!("sha256:{}", p.source_sha256),
        "parse_view_id": view, "bytes_match": bytes_match, "pages_checked": stored.len(),
        "page_mismatches": page_mismatches, "chunks_checked": n, "chunk_mismatches": chunk_mismatches,
        "check": "PSI record: bytes, page texts and chunk spans", "passed": passed}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_dates() {
        assert_eq!(
            size_text(1_224_752_113_090),
            "1,224.75 GB (1.22 TB; 1,224,752,113,090 bytes)"
        );
        assert_eq!(
            size_text(163_551_372_412),
            "163.55 GB (163,551,372,412 bytes)"
        );
        assert_eq!(
            size_text(41_765_333),
            "0.04 GB (41.77 MB; 41,765,333 bytes)"
        );
        assert_eq!(date_text("08-01-2007"), "2007-08-01 (PSI: 08-01-2007)");
        assert_eq!(date_text("31-Jul-2016"), "2016-07-31 (PSI: 31-Jul-2016)");
        assert_eq!(date_text("soon"), "soon");
    }
}

//! NASA Technical Reports Server (NTRS) client, read-only (R6).
//!
//! Documented public API (`https://ntrs.nasa.gov/api`, checked 2026-11-14):
//!
//! * search  `GET /api/citations/search?q=<text>&page.size=<n>&page.from=<offset>` answers
//!   `{"stats": {"total", "estimate", ...}, "results": [record, ...], "aggregations": ...}`.
//! * record  `GET /api/citations/<id>`.
//! * bytes   `GET /api/citations/<id>/downloads/<file>` (`links.pdf`, and `links.fulltext` for the
//!   provider's own `.txt` text when present). Range works; there is no CORS and framing is refused,
//!   so viewers go through the pass-through proxy ([`AccessMode::Proxy`]).
//!
//! Limits are the provider's: 500 requests per 15 minutes (reported in `x-ratelimit-*` headers, which
//! the gate obeys) and at most 10,000 records per query. A query whose total exceeds that cap is
//! refused rather than silently truncated: narrow it.
//!
//! An NTRS record is a *metadata* source (`IdBasis::MetadataRecord`) until its bytes are fetched.
//! Finding a record with an investigation's name is not identity: entries keep the query that found
//! them, and `investigation_id` stays empty.

use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use ignis_core::contracts::{
    AccessMode, Digest, IdBasis, Rights, RightsStatus, Source, SourceKind,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::http::{Got, Http};
use super::psi::UA;

pub const API: &str = "https://ntrs.nasa.gov";
/// Records per query the provider serves at most.
pub const QUERY_CAP: u64 = 10_000;
/// 500 requests per 15 minutes, spread evenly.
pub const MIN_INTERVAL: Duration = Duration::from_millis(1_800);
/// Rights determinations that permit public use (both spellings occur in NTRS records).
pub const PUBLIC_USE: [&str; 2] = ["PUBLIC_USE_PERMITTED", "GOV_PUBLIC_USE_PERMITTED"];

/// How long to wait before the next request: until the provider's window resets when it reports no
/// remaining requests, otherwise until `MIN_INTERVAL` has passed since the last one.
pub fn wait_before_next(
    since_last: Option<Duration>,
    remaining: Option<u64>,
    reset_epoch_s: Option<u64>,
    now_epoch_s: u64,
) -> Duration {
    if remaining == Some(0) {
        if let Some(r) = reset_epoch_s {
            return Duration::from_secs(r.saturating_sub(now_epoch_s) + 1);
        }
    }
    since_last.map_or(Duration::ZERO, |d| MIN_INTERVAL.saturating_sub(d))
}

/// The parts of a citation record IGNIS uses. The full record is kept alongside (`record`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NtrsRecord {
    pub ntrs_id: String,
    pub title: Option<String>,
    pub sti_type: Option<String>,
    pub distribution: Option<String>,
    pub determination: Option<String>,
    pub downloads_available: bool,
    /// Absolute URL of the PDF, when NTRS offers one.
    pub pdf_url: Option<String>,
    /// Absolute URL of NTRS's own full-text `.txt`, when present.
    pub fulltext_url: Option<String>,
    pub pdf_name: Option<String>,
    pub center: Option<String>,
    pub publication_date: Option<String>,
    /// NTRS's last modification of the record (its provider version).
    pub modified: Option<String>,
}

fn abs(link: &str) -> String {
    if link.starts_with("http") {
        link.to_string()
    } else {
        format!("{API}{link}")
    }
}

fn s(v: &Value) -> Option<String> {
    v.as_str().map(String::from)
}

pub fn parse_record(v: &Value) -> anyhow::Result<NtrsRecord> {
    let ntrs_id = match &v["id"] {
        Value::Number(n) => n.to_string(),
        Value::String(x) if !x.is_empty() => x.clone(),
        other => anyhow::bail!("NTRS record without an id: {other}"),
    };
    let available = v["downloadsAvailable"].as_bool().unwrap_or(false);
    let pdf = v["downloads"].as_array().and_then(|ds| {
        ds.iter().find(|d| {
            d["links"]["pdf"].is_string() || d["mimetype"].as_str() == Some("application/pdf")
        })
    });
    let link = |d: &Value, k: &str| d["links"][k].as_str().map(abs);
    let (pdf_url, fulltext_url, pdf_name) = match pdf.filter(|_| available) {
        Some(d) => (
            link(d, "pdf").or_else(|| link(d, "original")),
            link(d, "fulltext"),
            s(&d["name"]),
        ),
        None => (None, None, None),
    };
    Ok(NtrsRecord {
        ntrs_id,
        title: s(&v["title"]),
        sti_type: s(&v["stiType"]),
        distribution: s(&v["distribution"]),
        determination: s(&v["copyright"]["determinationType"]),
        downloads_available: available,
        pdf_url,
        fulltext_url,
        pdf_name,
        center: s(&v["center"]["code"]),
        publication_date: v["publications"]
            .as_array()
            .and_then(|p| p.iter().find_map(|x| s(&x["publicationDate"]))),
        modified: s(&v["modified"]),
    })
}

/// Rights from NTRS's own determination. Public distribution with a public-use determination is
/// `verified`; non-public distribution is `restricted`; anything else needs a human look.
pub fn rights(r: &NtrsRecord) -> Rights {
    let public = r.distribution.as_deref() == Some("PUBLIC");
    let status = match (public, r.determination.as_deref()) {
        (false, _) => RightsStatus::Restricted,
        (true, Some(d)) if PUBLIC_USE.contains(&d) => RightsStatus::Verified,
        _ => RightsStatus::Unverified,
    };
    Rights {
        license: r.determination.clone(),
        status,
        statement_locator: None,
    }
}

/// Identity of a record version: the canonical record without the per-query fields.
pub fn record_id(v: &Value) -> String {
    Digest::of_json_excluding(v, &["_meta", "index"]).to_string()
}

pub fn landing_url(ntrs_id: &str) -> String {
    format!("{API}/citations/{ntrs_id}")
}

/// The metadata-only `Source` of a record (bytes not fetched yet).
pub fn source_record(v: &Value, r: &NtrsRecord, retrieved_at: &str) -> Source {
    Source {
        id: record_id(v),
        id_basis: IdBasis::MetadataRecord,
        kind: SourceKind::Publication,
        investigation_id: None,
        title: r.title.clone(),
        canonical_url: Some(landing_url(&r.ntrs_id)),
        file_name: r.pdf_name.clone(),
        file_version: None,
        investigation_version: None,
        provider_version: r.modified.clone(),
        doi: None,
        ntrs_id: Some(r.ntrs_id.clone()),
        size_bytes: None,
        media_type: r.pdf_url.as_ref().map(|_| "application/pdf".to_string()),
        retrieved_at: Some(retrieved_at.to_string()),
        rights: rights(r),
    }
}

/// One line of an NTRS discovery manifest: what was found, by which query, and how to open it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NtrsEntry {
    pub query: String,
    /// 1-based rank in the provider's result order for `query`.
    pub rank: u64,
    pub record: NtrsRecord,
    pub source: Source,
    pub access_mode: AccessMode,
}

/// An NTRS id written into a PSI file name (`PSI-69_Presentations_20130010979.pdf`). A candidate
/// duplicate only: exact identity is the same sha256 of the fetched bytes.
pub fn ntrs_id_in(file_name: &str) -> Option<String> {
    let b = file_name.as_bytes();
    (0..b.len().saturating_sub(10)).find_map(|i| {
        let w = &b[i..i + 11];
        let edge = |j: Option<&u8>| j.is_none_or(|c| !c.is_ascii_digit());
        (w.iter().all(u8::is_ascii_digit)
            && (w.starts_with(b"19") || w.starts_with(b"20"))
            && edge(i.checked_sub(1).and_then(|j| b.get(j)))
            && edge(b.get(i + 11)))
        .then(|| String::from_utf8_lossy(w).into_owned())
    })
}

pub struct NtrsClient {
    http: Http,
    last: Option<Instant>,
    remaining: Option<u64>,
    reset: Option<u64>,
}

fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl NtrsClient {
    pub fn new(timeout_s: u64) -> NtrsClient {
        NtrsClient {
            http: Http::new(UA, timeout_s),
            last: None,
            remaining: None,
            reset: None,
        }
    }

    fn get(&mut self, url: &str, query: &[(&str, &str)]) -> anyhow::Result<Got> {
        sleep(wait_before_next(
            self.last.map(|t| t.elapsed()),
            self.remaining,
            self.reset,
            epoch_now(),
        ));
        let got = self.http.get(url, query);
        self.last = Some(Instant::now());
        let got = got?;
        let num = |h: &str| got.header(h).and_then(|v| v.parse().ok());
        self.remaining = num("x-ratelimit-remaining");
        self.reset = num("x-ratelimit-reset");
        Ok(got)
    }

    pub fn citation(&mut self, ntrs_id: &str) -> anyhow::Result<Value> {
        let url = format!("{API}/api/citations/{ntrs_id}");
        let got = self.get(&url, &[])?;
        serde_json::from_slice(&got.bytes).with_context(|| format!("{url}: not JSON"))
    }

    /// Every record of `query`, in the provider's order, paged by `page_size`. Refuses a query whose
    /// total is above `QUERY_CAP` or is only an estimate beyond what can be paged.
    pub fn search_all(&mut self, query: &str, page_size: u32) -> anyhow::Result<Vec<Value>> {
        anyhow::ensure!(page_size > 0, "page_size must be positive");
        let url = format!("{API}/api/citations/search");
        let mut out: Vec<Value> = Vec::new();
        loop {
            let (size, from) = (page_size.to_string(), out.len().to_string());
            let got = self.get(
                &url,
                &[("q", query), ("page.size", &size), ("page.from", &from)],
            )?;
            let v: Value =
                serde_json::from_slice(&got.bytes).with_context(|| format!("{url}: not JSON"))?;
            let total = page_total(&v).with_context(|| format!("{query:?}"))?;
            let page = v["results"].as_array().cloned().unwrap_or_default();
            let n = page.len();
            out.extend(page);
            if n == 0 || out.len() as u64 >= total {
                return Ok(out);
            }
        }
    }
}

/// The exact record total of one search page. A page without `stats.total`, with an estimated
/// total, or with more records than NTRS lets a query page through is refused: paging on a guessed
/// or absent total would silently truncate the listing.
pub fn page_total(page: &Value) -> anyhow::Result<u64> {
    let stats = &page["stats"];
    let total = stats["total"]
        .as_u64()
        .context("search page has no stats.total")?;
    anyhow::ensure!(
        stats["estimate"].as_bool() != Some(true),
        "stats.total {total} is only an estimate; narrow the query"
    );
    anyhow::ensure!(
        total <= QUERY_CAP,
        "{total} records exceed NTRS's {QUERY_CAP}-record cap; narrow the query"
    );
    Ok(total)
}

/// Entries for every record of a query (pure: the network part is [`NtrsClient::search_all`]).
pub fn entries(
    query: &str,
    records: &[Value],
    retrieved_at: &str,
) -> anyhow::Result<Vec<NtrsEntry>> {
    records
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let record = parse_record(v)?;
            Ok(NtrsEntry {
                query: query.to_string(),
                rank: i as u64 + 1,
                source: source_record(v, &record, retrieved_at),
                record,
                access_mode: super::storage::ntrs_access_mode(),
            })
        })
        .collect()
}

//! NASA Physical Sciences Informatics (PSI) repository client, read-only.
//!
//! Request shapes were taken from the public PSI web client (`/physci/repo` Angular bundle) and
//! verified against the live service on 2026-11-07:
//!
//! * metadata `GET  https://psi.nasa.gov/geode-py/ws/repo/investigations/<ACC>`: JSON with `version`
//!   (investigation version), `licenseIdentifier`, `doi`, `experimentalTable`.
//! * listing  `POST https://psi.nasa.gov/geode-py/ws/files/v2` with JSON
//!   `{"folder": "", "fileType": "study", "studyId": "<ACC>", "version": <investigation version>}`.
//!   Returns `{"files": [category node, ...]}`; category nodes carry `displayName` and `files`; leaves
//!   have `directory: false`, `key` (S3 key), `bytes_size`, `version` (file version), `id`.
//! * download `GET  https://psi.nasa.gov/geode-py/ws/studies/<ACC>/download?file=<name>&version=<v>`
//!   answers 302 to a *signed, expiring* S3 URL. The stable identity is the geode URL (investigation +
//!   file name + file version), never the redirect.
//!
//! These are the endpoints the public site uses, not a documented API, so every response we rely on
//! is saved under `data/raw/samples/_meta` with its hash.

use std::path::{Path, PathBuf};

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::http::{sha256_bytes, Http};

pub const BASE: &str = "https://psi.nasa.gov/geode-py/ws";
pub const UA: &str = "ignis-data/0.1 (NASA Space Apps 2026; read-only research client)";

/// Python `urllib.parse.quote(s, safe="")`: everything but `A-Za-z0-9_.-~` is escaped.
const QUOTE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'_')
    .remove(b'.')
    .remove(b'-')
    .remove(b'~');
const QUOTE_PATH: &AsciiSet = &QUOTE.remove(b'/');

pub fn quote(s: &str) -> String {
    utf8_percent_encode(s, QUOTE).to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteFile {
    pub investigation: String,
    /// Top-level PSI category ("Raw Data", "Experimental table", ...).
    pub category: String,
    /// S3 key "psi-data/PSI-69/Raw Data/PSI-69_Raw Data_FLEX-007_DropletData.txt".
    pub key: String,
    pub file_name: String,
    pub size_bytes: Option<i64>,
    pub file_version: Option<i64>,
    pub psi_file_id: Option<String>,
    pub updated: Option<String>,
    pub restricted: bool,
}

impl RemoteFile {
    pub fn stable_url(&self) -> String {
        stable_url(&self.investigation, &self.file_name, self.file_version)
    }
    pub fn to_json(&self) -> Value {
        let mut v = serde_json::to_value(self).unwrap();
        v["stable_url"] = json!(self.stable_url());
        v
    }
}

pub fn stable_url(investigation: &str, file_name: &str, file_version: Option<i64>) -> String {
    let mut q = format!("file={}", quote(file_name));
    if let Some(v) = file_version {
        q.push_str(&format!("&version={v}"));
    }
    format!(
        "{BASE}/studies/{}/download?{q}",
        utf8_percent_encode(investigation, QUOTE_PATH)
    )
}

pub fn metadata_url(investigation: &str) -> String {
    format!(
        "{BASE}/repo/investigations/{}",
        utf8_percent_encode(investigation, QUOTE_PATH)
    )
}

fn as_int(v: &Value) -> Option<i64> {
    v.as_i64()
        .or_else(|| v.as_f64().map(|x| x as i64))
        .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

pub fn flatten_listing(investigation: &str, payload: &Value) -> Vec<RemoteFile> {
    fn walk<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
        for f in node
            .get("files")
            .and_then(|f| f.as_array())
            .into_iter()
            .flatten()
        {
            if f.get("directory") == Some(&Value::Bool(false)) {
                out.push(f);
            } else {
                walk(f, out);
            }
        }
    }
    let mut out = Vec::new();
    for top in payload
        .get("files")
        .and_then(|f| f.as_array())
        .into_iter()
        .flatten()
    {
        let cat = top
            .get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut leaves = Vec::new();
        walk(top, &mut leaves);
        for leaf in leaves {
            let key = leaf["key"].as_str().unwrap_or_default().to_string();
            let updated = leaf
                .get("updated")
                .filter(|v| truthy(v))
                .and_then(|v| v.as_str())
                .map(String::from);
            out.push(RemoteFile {
                investigation: investigation.into(),
                category: cat.clone(),
                file_name: key.rsplit('/').next().unwrap_or("").to_string(),
                key,
                size_bytes: leaf.get("bytes_size").and_then(as_int),
                file_version: leaf.get("version").and_then(as_int),
                psi_file_id: leaf.get("id").and_then(|v| v.as_str()).map(String::from),
                updated,
                restricted: leaf.get("restricted").is_some_and(truthy)
                    || leaf.get("restrictedAccess").is_some_and(truthy),
            });
        }
    }
    out.sort_by(|a, b| a.key.cmp(&b.key));
    out
}

pub struct Fetched {
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: i64,
    pub media_type: Option<String>,
    pub retrieved_at: String,
}

pub struct PsiClient {
    http: Http,
}

impl PsiClient {
    pub fn new(timeout_s: u64) -> PsiClient {
        PsiClient {
            http: Http::new(UA, timeout_s),
        }
    }

    pub fn metadata(&self, investigation: &str) -> anyhow::Result<(Value, Vec<u8>)> {
        let g = self.http.get(&metadata_url(investigation), &[])?;
        Ok((serde_json::from_slice(&g.bytes)?, g.bytes))
    }

    pub fn listing(
        &self,
        investigation: &str,
        version: i64,
    ) -> anyhow::Result<(Vec<RemoteFile>, Vec<u8>)> {
        let body = json!({"folder": "", "fileType": "study", "studyId": investigation, "version": version});
        let g = self.http.post_json(&format!("{BASE}/files/v2"), &body)?;
        let payload: Value = serde_json::from_slice(&g.bytes)?;
        Ok((flatten_listing(investigation, &payload), g.bytes))
    }

    /// Stream one file via its stable URL (following the signed redirect, which is not kept).
    pub fn fetch(
        &self,
        rf: &RemoteFile,
        dest_dir: &Path,
        max_bytes: Option<i64>,
    ) -> anyhow::Result<Fetched> {
        anyhow::ensure!(
            !rf.restricted,
            "{} is restricted; not fetched",
            rf.file_name
        );
        if let (Some(cap), Some(sz)) = (max_bytes, rf.size_bytes) {
            anyhow::ensure!(
                sz <= cap,
                "{} is {sz} B, above the per-file cap {cap} B",
                rf.file_name
            );
        }
        let path = dest_dir.join(&rf.file_name);
        let s = self
            .http
            .stream_to(&rf.stable_url(), &path, rf.size_bytes.map(|x| x as u64))?;
        Ok(Fetched {
            path,
            sha256: s.sha256,
            size_bytes: s.size as i64,
            media_type: s.content_type,
            retrieved_at: now_micros(),
        })
    }
}

/// `datetime.now(UTC)` as pydantic serializes it (`...Z`, microseconds).
pub fn now_micros() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

pub fn save_meta(meta_dir: &Path, name: &str, content: &[u8]) -> anyhow::Result<(PathBuf, String)> {
    std::fs::create_dir_all(meta_dir)?;
    let p = meta_dir.join(name);
    std::fs::write(&p, content)?;
    Ok((p, sha256_bytes(content)))
}

/// The listing manifest: one sorted-key JSON object per line, Python `json.dumps` layout (the
/// file's sha256 is cited by BASS review items, so its bytes must be stable).
pub fn dump_listing(files: &[RemoteFile]) -> String {
    files
        .iter()
        .map(|f| crate::text::py_dumps(&ignis_core::hashing::sorted(&f.to_json())) + "\n")
        .collect()
}

pub fn read_listing(path: &Path) -> anyhow::Result<Vec<RemoteFile>> {
    let rows = crate::common::read_jsonl_values(path)?;
    rows.into_iter()
        .map(|mut r| {
            if let Value::Object(m) = &mut r {
                m.remove("stable_url");
            }
            Ok(serde_json::from_value(r)?)
        })
        .collect()
}

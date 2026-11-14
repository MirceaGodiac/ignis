//! Shared plumbing for the international (non-NASA) source adapters.
//!
//! Manifest lines are `{"record": <type>, "id_basis": ..., "data": <contract dump>, "extra": {...}}`.
//!
//! `id_basis` says what a `Source.id` hashes, because the contract defines it as the hash of the
//! asset bytes and most international assets are deliberately *not* downloaded:
//!
//! * `bytes` — sha256 of bytes we fetched (or a publisher sha256 we re-verified on fetch)
//! * `listing` — sha256 of the canonical listing identity (url, size, publisher checksum); a
//!   placeholder until the bytes are fetched, never a content hash
//! * `metadata_record` — sha256 of the canonical JSON of a public metadata record (access-gated data)

use std::path::{Path, PathBuf};

use ignis_core::contracts::{IdBasis, Rights, Source, SourceKind};
use ignis_core::hashing::{canonical_json, sha256_hex};
use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::paths;
use crate::sources::http::Http;

pub const USER_AGENT: &str =
    "IGNIS-sources-intl/0.1 (NASA Space Apps 2026 research; metadata only)";

pub fn manifest_dir() -> PathBuf {
    paths::manifests().join("intl")
}

pub fn raw_dir() -> PathBuf {
    paths::root().join("raw").join("intl")
}

/// UTC now, whole seconds (`datetime.now(UTC).replace(microsecond=0)`) as pydantic / isoformat print it.
pub fn now() -> chrono::DateTime<chrono::Utc> {
    let t = chrono::Utc::now();
    chrono::DateTime::from_timestamp(t.timestamp(), 0).unwrap()
}
pub fn z(t: &chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
pub fn iso(t: &chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%dT%H:%M:%S+00:00").to_string()
}

pub fn client() -> Http {
    Http::new(USER_AGENT, 120)
}

/// Placeholder Source id for an asset we have listed but not downloaded (id_basis='listing').
pub fn listing_id(identity: Value) -> String {
    format!(
        "sha256:{}",
        sha256_hex(canonical_json(&json!({ "listing_identity": identity })).as_bytes())
    )
}

pub fn record_id(obj: &Value) -> String {
    format!("sha256:{}", sha256_hex(canonical_json(obj).as_bytes()))
}

pub fn entry(
    record: &str,
    model: impl Serialize,
    id_basis: Option<IdBasis>,
    extra: Vec<(&str, Value)>,
) -> Value {
    let mut out = Map::new();
    out.insert("record".into(), json!(record));
    out.insert("data".into(), serde_json::to_value(model).unwrap());
    if let Some(b) = id_basis {
        out.insert("id_basis".into(), serde_json::to_value(b).unwrap());
    }
    if !extra.is_empty() {
        out.insert(
            "extra".into(),
            Value::Object(extra.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
        );
    }
    Value::Object(out)
}

pub fn manifest_path(investigation_id: &str, suffix: &str) -> PathBuf {
    manifest_dir().join(format!("{investigation_id}{suffix}.jsonl"))
}

/// Sorted-key JSON lines in the Python `json.dumps` layout.
pub fn write_jsonl(path: &Path, rows: &[Value]) -> anyhow::Result<usize> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    let body: String = rows
        .iter()
        .map(|r| crate::text::py_dumps(&ignis_core::hashing::sorted(r)) + "\n")
        .collect();
    std::fs::write(path, body)?;
    Ok(rows.len())
}

pub fn read_jsonl(path: &Path) -> anyhow::Result<Vec<Value>> {
    crate::common::read_jsonl_values(path)
}

pub fn raw_path(investigation_id: &str, name: &str) -> PathBuf {
    raw_dir().join(investigation_id).join(name)
}

pub fn local_sha256(path: &Path) -> Option<String> {
    path.exists()
        .then(|| ignis_core::hashing::file_sha256(path).ok())
        .flatten()
}

/// Stream one file to `dest` and verify size / md5 / sha256; returns the sha256 hex of the bytes.
pub fn download(
    url: &str,
    dest: &Path,
    expected_size: Option<u64>,
    md5: Option<&str>,
    sha256: Option<&str>,
) -> anyhow::Result<String> {
    let s = client().stream_to(url, dest, expected_size)?;
    let bad = md5.is_some_and(|m| m != s.md5) || sha256.is_some_and(|h| h != s.sha256);
    if bad {
        let _ = std::fs::remove_file(dest);
        anyhow::bail!("{url}: checksum mismatch");
    }
    Ok(s.sha256)
}

/// A `Source` with every optional field empty.
pub fn source(
    id: String,
    id_basis: IdBasis,
    kind: SourceKind,
    investigation_id: &str,
    rights: Rights,
) -> Source {
    Source {
        id,
        id_basis,
        kind,
        investigation_id: Some(investigation_id.into()),
        title: None,
        canonical_url: None,
        file_name: None,
        file_version: None,
        investigation_version: None,
        provider_version: None,
        doi: None,
        ntrs_id: None,
        size_bytes: None,
        media_type: None,
        retrieved_at: None,
        rights,
    }
}

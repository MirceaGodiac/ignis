//! Source manifests: one [`Source`] per fetched asset, keyed by the sha256 of its bytes.
//!
//! * `data/manifests/<ACC>.jsonl` — fetched assets (Source records, merged by file name + version)
//! * `data/manifests/<ACC>.listing.jsonl` — the remote inventory at a given investigation version
//! * `data/manifests/<ACC>.investigation.json` — investigation version, license, DOI, metadata hash
//!
//! A change of investigation or file version yields new records; existing ids never change meaning.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use ignis_core::contracts::{Rights, RightsStatus, Source, SourceKind, SourceLocator};
use serde_json::{json, Value};

use super::psi::{Fetched, RemoteFile};
use crate::paths;

fn suffix(name: &str) -> String {
    Path::new(name)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
        .unwrap_or_default()
}

pub fn kind_of(file_name: &str, category: &str) -> SourceKind {
    let sfx = suffix(file_name);
    if [".mp4", ".ts", ".avi", ".mov"].contains(&sfx.as_str()) {
        return SourceKind::Video;
    }
    if category.contains("Calibration") {
        return SourceKind::Calibration;
    }
    if [
        "Reports",
        "Presentations",
        "Science Documents",
        "Engineering Documents",
    ]
    .contains(&category)
    {
        return if category.contains("Engineering") || category.contains("Science") {
            SourceKind::Protocol
        } else {
            SourceKind::Publication
        };
    }
    if sfx == ".zip" && category != "Raw Data" {
        return SourceKind::Archive;
    }
    SourceKind::DatasetFile
}

/// Fallback media types (Python `mimetypes.guess_type`) for octet-stream responses.
fn guess_type(name: &str) -> Option<&'static str> {
    Some(match suffix(name).as_str() {
        ".csv" => "text/csv",
        ".txt" => "text/plain",
        ".json" => "application/json",
        ".zip" => "application/zip",
        ".pdf" => "application/pdf",
        ".xls" => "application/vnd.ms-excel",
        ".xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ".docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ".mp4" => "video/mp4",
        ".mov" => "video/quicktime",
        ".avi" => "video/x-msvideo",
        ".png" => "image/png",
        ".jpg" | ".jpeg" => "image/jpeg",
        _ => return None,
    })
}

pub fn rights_from_license(license: Option<&str>, meta_sha256: &str) -> Rights {
    // The license statement is the investigation metadata record itself (cited by hash).
    let status = match license {
        Some("CC0-1.0") | Some("CC-BY-4.0") => RightsStatus::Verified,
        None => RightsStatus::Unverified,
        Some(_) => RightsStatus::Restricted,
    };
    Rights {
        license: license.map(String::from),
        status,
        statement_locator: Some(SourceLocator {
            asset_sha256: meta_sha256.into(),
            path: Some("licenseIdentifier".into()),
            row: None,
            column: None,
            page: None,
            frame: None,
            pts_s: None,
            char_start: None,
            char_end: None,
            ..Default::default()
        }),
    }
}

pub fn source_record(
    rf: &RemoteFile,
    got: &Fetched,
    investigation_version: &str,
    rights: &Rights,
    doi: Option<&str>,
) -> Source {
    let mut media = got.media_type.clone();
    if media
        .as_deref()
        .is_none_or(|m| m == "binary/octet-stream" || m == "application/octet-stream")
    {
        media = guess_type(&rf.file_name).map(String::from).or(media);
    }
    Source {
        id: format!("sha256:{}", got.sha256),
        id_basis: ignis_core::contracts::IdBasis::Bytes,
        kind: kind_of(&rf.file_name, &rf.category),
        investigation_id: Some(rf.investigation.clone()),
        title: Some(rf.key.clone()),
        canonical_url: Some(rf.stable_url()),
        file_name: Some(rf.file_name.clone()),
        file_version: rf.file_version.map(|v| v.to_string()),
        investigation_version: Some(investigation_version.into()),
        provider_version: None,
        doi: doi.map(String::from),
        ntrs_id: None,
        size_bytes: Some(got.size_bytes),
        media_type: media,
        retrieved_at: Some(got.retrieved_at.clone()),
        rights: rights.clone(),
    }
}

pub fn manifest_path(investigation: &str) -> PathBuf {
    paths::manifests().join(format!("{investigation}.jsonl"))
}

pub fn read_manifest(investigation: &str) -> anyhow::Result<Vec<Source>> {
    crate::common::read_jsonl(&manifest_path(investigation))
}

/// Merge by (file_name, file_version): a re-fetch of identical bytes keeps the first retrieved_at.
pub fn write_manifest(investigation: &str, records: &[Source]) -> anyhow::Result<PathBuf> {
    let key = |s: &Source| {
        (
            s.file_name.clone().unwrap_or_default(),
            s.file_version.clone().unwrap_or_default(),
        )
    };
    let mut existing: BTreeMap<(String, String), Source> = BTreeMap::new();
    for s in read_manifest(investigation)? {
        existing.insert(key(&s), s);
    }
    for r in records {
        match existing.get(&key(r)) {
            Some(old) if old.id != r.id => anyhow::bail!(
                "{} v{}: bytes changed ({} -> {}) without a version change",
                r.file_name.as_deref().unwrap_or(""),
                r.file_version.as_deref().unwrap_or("None"),
                old.id,
                r.id
            ),
            Some(_) => {}
            None => {
                existing.insert(key(r), r.clone());
            }
        }
    }
    let p = manifest_path(investigation);
    std::fs::create_dir_all(p.parent().unwrap())?;
    let mut body = String::new();
    for s in existing.values() {
        body.push_str(&serde_json::to_string(s)?);
        body.push('\n');
    }
    std::fs::write(&p, body)?;
    Ok(p)
}

/// Rewrite the manifest with exactly `records` (sorted like `write_manifest`). Only for removing
/// records proven redundant; additions go through the merging `write_manifest`.
pub fn replace_manifest(investigation: &str, records: &[Source]) -> anyhow::Result<PathBuf> {
    let key = |s: &Source| {
        (
            s.file_name.clone().unwrap_or_default(),
            s.file_version.clone().unwrap_or_default(),
        )
    };
    let mut sorted: Vec<&Source> = records.iter().collect();
    sorted.sort_by_key(|s| key(s));
    let p = manifest_path(investigation);
    let mut body = String::new();
    for s in sorted {
        body.push_str(&serde_json::to_string(s)?);
        body.push('\n');
    }
    std::fs::write(&p, body)?;
    Ok(p)
}

/// Identity of the source state a cohort was built from (ids, versions, rights).
pub fn manifest_hash(investigations: &[&str]) -> anyhow::Result<String> {
    let mut invs: Vec<&str> = investigations.to_vec();
    invs.sort();
    let mut rows: Vec<[Option<String>; 6]> = Vec::new();
    for inv in invs {
        for s in read_manifest(inv)? {
            rows.push([
                Some(s.id),
                s.investigation_id,
                s.file_name,
                s.file_version,
                s.investigation_version,
                s.rights.license,
            ]);
        }
    }
    rows.sort();
    Ok(ignis_core::hashing::content_hash(
        &json!({ "sources": rows }),
        &["id"],
    ))
}

pub fn by_name(investigation: &str) -> anyhow::Result<BTreeMap<String, Source>> {
    Ok(read_manifest(investigation)?
        .into_iter()
        .filter_map(|s| {
            s.file_name
                .clone()
                .filter(|n| !n.is_empty())
                .map(|n| (n, s))
        })
        .collect())
}

pub fn local_path(src: &Source) -> PathBuf {
    paths::raw(src.investigation_id.as_deref()).join(src.file_name.as_deref().unwrap_or(""))
}

/// Hex part of a `sha256:<hex>` source id.
pub fn sha_of(src: &Source) -> String {
    ignis_core::hashing::hex_part(&src.id).to_string()
}

pub fn write_investigation(
    investigation: &str,
    meta: &Value,
    meta_sha256: &str,
    retrieved_at: &str,
) -> anyhow::Result<PathBuf> {
    let p = paths::manifests().join(format!("{investigation}.investigation.json"));
    std::fs::create_dir_all(p.parent().unwrap())?;
    let g = |k: &str| meta.get(k).cloned().unwrap_or(Value::Null);
    let version = match meta.get("version") {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => "None".into(),
    };
    let body = crate::text::py_dumps_ordered_indent(&[
        ("investigation", json!(investigation)),
        ("title", g("title")),
        ("acronym", g("investigationAcronym")),
        ("investigation_version", json!(version)),
        ("version_reason", g("versionReason")),
        ("modified", g("modifiedDate")),
        ("doi", g("doi")),
        ("license", g("licenseIdentifier")),
        ("license_link", g("licenseLink")),
        ("hardware", g("experimentHardware")),
        ("total_file_size_bytes", g("totalFileSize")),
        ("metadata_url_sha256", json!(meta_sha256)),
        ("retrieved_at", json!(retrieved_at)),
    ]);
    std::fs::write(&p, body + "\n")?;
    Ok(p)
}

pub fn read_investigation(investigation: &str) -> anyhow::Result<Value> {
    let p = paths::manifests().join(format!("{investigation}.investigation.json"));
    let t = std::fs::read_to_string(&p).with_context(|| {
        format!(
            "{}: run `ignis sources inspect {investigation}`",
            p.display()
        )
    })?;
    Ok(serde_json::from_str(&t)?)
}

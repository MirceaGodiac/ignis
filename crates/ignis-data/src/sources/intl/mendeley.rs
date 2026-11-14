//! Mendeley Data 2gw4pjhkxz v1 — Rojas-Alva & Jomaas, "Classification of results from combustion
//! experiments with solid materials in microgravity" (DOI 10.17632/2gw4pjhkxz.1, CC BY-NC 3.0).
//!
//! A bibliographic evidence index, not measurements. The record holds one file only — a 54-page PDF
//! appendix (2,949,073 bytes, publisher sha256 4260abf6...01d1). There is no CSV/XLSX.
//!
//! The candidate-study extraction from that PDF (`MENDELEY-2GW4PJHKXZ.candidates.jsonl`, 211 rows,
//! produced once with PyMuPDF text extraction) is kept as committed evidence; the Rust manifest
//! builder does not re-derive it (PDF text layout differs between extractors, which would silently
//! change the heuristic fields). It is referenced by the investigation entry when present.

use ignis_core::contracts::{
    Family, IdBasis, Investigation, Rights, RightsStatus, SourceKind, SourceLocator,
};
use serde_json::{json, Value};

use super::common::{self, entry, z};
use crate::sources::http::sha256_bytes;

pub const INVESTIGATION_ID: &str = "MENDELEY-2GW4PJHKXZ";
pub const DATASET: &str = "2gw4pjhkxz";

pub fn api() -> String {
    format!("https://data.mendeley.com/public-api/datasets/{DATASET}")
}
pub fn page() -> String {
    format!("https://data.mendeley.com/datasets/{DATASET}/1")
}

pub fn build_manifest() -> anyhow::Result<Vec<Value>> {
    let retrieved = common::now();
    let got = common::client().get(&api(), &[])?; // the "?version=1" variant omits the file list
    let meta: Value = serde_json::from_slice(&got.bytes)?;
    anyhow::ensure!(
        meta["version"] == json!(1),
        "Mendeley dataset is now version {}; review before re-registering",
        meta["version"]
    );
    let meta_sha = sha256_bytes(&got.bytes);
    let lic = &meta["data_licence"];
    let short = lic["short_name"].as_str();
    let rights = Rights {
        license: short.map(String::from),
        status: if short.unwrap_or("").contains("NC") {
            RightsStatus::Noncommercial
        } else {
            RightsStatus::Unverified
        },
        statement_locator: Some(SourceLocator {
            asset_sha256: meta_sha.clone(),
            path: Some(format!("{}#data_licence", api())),
            row: None,
            column: None,
            page: None,
            frame: None,
            pts_s: None,
            char_start: None,
            char_end: None,
            ..Default::default()
        }),
    };
    let mut s0 = common::source(
        format!("sha256:{meta_sha}"),
        IdBasis::Bytes,
        SourceKind::Webpage,
        INVESTIGATION_ID,
        rights.clone(),
    );
    s0.title = Some("Mendeley Data public API record".into());
    s0.canonical_url = Some(api());
    s0.size_bytes = Some(got.bytes.len() as i64);
    s0.media_type = Some("application/json".into());
    s0.retrieved_at = Some(z(&retrieved));
    let mut sids = vec![s0.id.clone()];
    let mut rows = vec![entry("source", &s0, Some(IdBasis::Bytes), vec![])];
    for f in meta["files"].as_array().into_iter().flatten() {
        let cd = &f["content_details"];
        let fname = f["filename"].as_str().unwrap_or_default();
        let local = common::raw_path(INVESTIGATION_ID, fname);
        let sha = common::local_sha256(&local);
        let publisher = cd["sha256_hash"].as_str().unwrap_or_default();
        if let Some(s) = &sha {
            anyhow::ensure!(
                s == publisher,
                "{}: sha256 differs from the publisher's",
                local.display()
            );
        }
        let mut s = common::source(
            format!("sha256:{publisher}"),
            IdBasis::Bytes,
            SourceKind::Publication,
            INVESTIGATION_ID,
            rights.clone(),
        );
        s.title = Some(format!(
            "{} (appendix)",
            meta["name"].as_str().unwrap_or("")
        ));
        s.canonical_url = Some(format!(
            "https://data.mendeley.com/public-files/datasets/{DATASET}/files/{}/file_downloaded",
            f["id"].as_str().unwrap_or_default()
        ));
        s.file_name = Some(fname.into());
        s.file_version = Some("1".into());
        s.doi = meta["doi"]["id"].as_str().map(String::from);
        s.size_bytes = cd["size"].as_i64();
        s.media_type = cd["content_type"].as_str().map(String::from);
        s.retrieved_at = sha.as_ref().map(|_| z(&retrieved));
        sids.push(s.id.clone());
        rows.push(entry(
            "source",
            &s,
            Some(IdBasis::Bytes),
            vec![
                (
                    "sha256_from",
                    json!(format!(
                        "publisher{}",
                        if sha.is_some() { "+verified" } else { "" }
                    )),
                ),
                ("folder_id", f["folder_id"].clone()),
            ],
        ));
    }
    let inv = Investigation {
        id: INVESTIGATION_ID.into(),
        name: meta["name"].as_str().unwrap_or_default().into(),
        platform: None,
        families: vec![Family::Other],
        source_ids: sids,
    };
    let authors: Vec<String> = meta["contributors"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| {
            format!(
                "{} {}",
                c["first_name"].as_str().unwrap_or("").trim(),
                c["last_name"].as_str().unwrap_or("").trim()
            )
        })
        .collect();
    let cands = common::manifest_path(INVESTIGATION_ID, ".candidates");
    rows.insert(
        0,
        entry(
            "investigation",
            &inv,
            None,
            vec![
                ("role", json!("evidence_index")),
                ("doi", meta["doi"]["id"].clone()),
                ("page", json!(page())),
                ("authors", json!(authors)),
                ("published", meta["publish_date"].clone()),
                ("license", lic["full_name"].clone()),
                ("license_url", lic["url"].clone()),
                ("description", meta["description"].clone()),
                (
                    "platform_reason",
                    json!("bibliographic index spanning many platforms"),
                ),
                ("machine_readable_table", json!("none in record (PDF only)")),
                (
                    "candidates_file",
                    if cands.exists() {
                        json!(format!("{INVESTIGATION_ID}.candidates.jsonl"))
                    } else {
                        Value::Null
                    },
                ),
            ],
        ),
    );
    Ok(rows)
}

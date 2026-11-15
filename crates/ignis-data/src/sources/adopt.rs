//! R6 close-out: recording files that are already on disk, and the coverage of the stored corpus.
//!
//! `adopt` turns bytes that arrived by another route (a teammate's copy uploaded to the server) into
//! manifest records, under the same rules as a fetch: a file is recorded only when its size equals
//! NASA's listing, its bytes are hashed while reading, and the record is the same `Source` a fetch
//! writes. It never downloads. A file whose size differs is refused and listed (a damaged copy is not
//! recorded as NASA's), as is a file without a listed size (nothing to check it against).
//!
//! `coverage` says, per flight investigation, how much of the stored corpus is present and recorded,
//! what is missing, and what was refused; `manifest_hash` over the 19 manifests is the reproducible
//! identity of the stored corpus.

use std::collections::{BTreeSet, HashMap};

use serde::Serialize;

use super::manifest as mf;
use super::psi::{flatten_listing, read_listing, Fetched, RemoteFile};
use super::storage::{storage_class, Flight, StorageClass, FLIGHTS};
use crate::paths;

/// The listing of `inv`: the raw PSI response saved by `sources inspect` when present, else the
/// committed `data/manifests/<inv>.listing.jsonl` (same rows).
pub fn listing(inv: &str) -> anyhow::Result<Vec<RemoteFile>> {
    let raw = paths::meta().join(format!("files_{inv}.json"));
    if raw.exists() {
        return Ok(flatten_listing(
            inv,
            &serde_json::from_str(&std::fs::read_to_string(raw)?)?,
        ));
    }
    let committed = paths::manifests().join(format!("{inv}.listing.jsonl"));
    anyhow::ensure!(
        committed.exists(),
        "no listing for {inv}; run `ignis sources inspect {inv}` first"
    );
    read_listing(&committed)
}

/// A file whose NASA listing states the wrong size, with the size and hash NASA actually serves
/// (`data/manifests/listing_errata.jsonl`, committed). A size mismatch is accepted only through an
/// erratum, and only when the bytes hash to its `sha256`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Erratum {
    pub investigation: String,
    pub file_name: String,
    pub file_version: Option<i64>,
    pub listed_size: i64,
    pub served_size: u64,
    pub sha256: String,
    pub checked_at: String,
    pub note: String,
}

pub fn errata(inv: &str) -> anyhow::Result<HashMap<(String, Option<i64>), Erratum>> {
    let p = paths::manifests().join("listing_errata.jsonl");
    if !p.exists() {
        return Ok(HashMap::new());
    }
    let rows: Vec<Erratum> = crate::common::read_jsonl(&p)?;
    Ok(rows
        .into_iter()
        .filter(|e| e.investigation == inv)
        .map(|e| ((e.file_name.clone(), e.file_version), e))
        .collect())
}

/// The files the server stores for `inv` by the atlas rule: per file name the **latest listed
/// version** (one path holds one version; NASA's listing keeps older versions, e.g. 53 ACME BRE
/// files in v1 and v2), restricted files excluded. Older versions stay listing entries.
pub fn stored_files(inv: &str) -> anyhow::Result<Vec<RemoteFile>> {
    let mut latest: std::collections::BTreeMap<String, RemoteFile> = Default::default();
    for f in listing(inv)? {
        if f.restricted || storage_class(&f.category, &f.file_name) == StorageClass::ManifestOnly {
            continue;
        }
        match latest.get(&f.file_name) {
            Some(cur) if cur.file_version >= f.file_version => {}
            _ => {
                latest.insert(f.file_name.clone(), f);
            }
        }
    }
    Ok(latest.into_values().collect())
}

/// Remove records of an older version that claim exactly the bytes recorded for the latest version
/// of the same file (a duplicate claim, e.g. from counting both listed versions against one path).
/// A record of an older version with other bytes is history and is kept. Returns how many went.
pub fn prune_duplicate_versions(inv: &str) -> anyhow::Result<usize> {
    let recs = mf::read_manifest(inv)?;
    let mut newest: HashMap<String, (Option<String>, String)> = HashMap::new();
    for s in &recs {
        let name = s.file_name.clone().unwrap_or_default();
        let v = s.file_version.clone();
        let num = |v: &Option<String>| v.as_deref().and_then(|x| x.parse::<i64>().ok());
        match newest.get(&name) {
            Some((cur, _)) if num(cur) >= num(&v) => {}
            _ => {
                newest.insert(name, (v, s.id.clone()));
            }
        }
    }
    let keep: Vec<_> = recs
        .iter()
        .filter(|s| {
            let name = s.file_name.clone().unwrap_or_default();
            let (v, id) = &newest[&name];
            s.file_version == *v || s.id != *id
        })
        .cloned()
        .collect();
    let gone = recs.len() - keep.len();
    if gone > 0 {
        mf::replace_manifest(inv, &keep)?;
    }
    Ok(gone)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SizeMismatch {
    pub file_name: String,
    pub listed: i64,
    pub on_disk: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AdoptReport {
    pub investigation: String,
    pub stored_files: usize,
    pub already_recorded: usize,
    /// Recorded by this run (or that would be, without `execute`).
    pub adopted: usize,
    pub missing: usize,
    pub size_mismatch: Vec<SizeMismatch>,
    pub unknown_size: Vec<String>,
    /// Records of an older version claiming the same bytes as the latest, removed.
    pub pruned_duplicate_versions: usize,
}

/// Files recorded between manifest writes of a long adoption.
const CHECKPOINT_EVERY: usize = 500;

fn mtime_utc(md: &std::fs::Metadata) -> String {
    let t: chrono::DateTime<chrono::Utc> = md
        .modified()
        .map(Into::into)
        .unwrap_or_else(|_| chrono::Utc::now());
    t.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

/// Record the present, unrecorded stored files of `inv`. Without `execute` it only counts.
/// `retrieved_at` of an adopted file is when its copy arrived on this host (file mtime): the
/// time the teammate fetched it from NASA is not known here.
pub fn adopt(inv: &str, execute: bool) -> anyhow::Result<AdoptReport> {
    let files = stored_files(inv)?;
    let info = mf::read_investigation(inv)?;
    let rights = mf::rights_from_license(
        info["license"].as_str(),
        info["metadata_url_sha256"].as_str().unwrap_or_default(),
    );
    let iv = info["investigation_version"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let have = mf::by_name(inv)?;
    let fixes = errata(inv)?;
    let mut r = AdoptReport {
        investigation: inv.into(),
        stored_files: files.len(),
        ..Default::default()
    };
    let mut records = Vec::new();
    for rf in &files {
        if have
            .get(&rf.file_name)
            .is_some_and(|s| s.file_version == rf.file_version.map(|v| v.to_string()))
        {
            r.already_recorded += 1;
            continue;
        }
        let local = paths::raw(Some(inv)).join(&rf.file_name);
        let Ok(md) = std::fs::metadata(&local) else {
            r.missing += 1;
            continue;
        };
        match rf.size_bytes {
            None => {
                r.unknown_size.push(rf.file_name.clone());
                continue;
            }
            Some(s) if s < 0 || s as u64 != md.len() => {
                // only a recorded listing erratum with this exact served size may explain it
                let e = fixes.get(&(rf.file_name.clone(), rf.file_version));
                if !e.is_some_and(|e| e.served_size == md.len()) {
                    r.size_mismatch.push(SizeMismatch {
                        file_name: rf.file_name.clone(),
                        listed: s,
                        on_disk: md.len(),
                    });
                    continue;
                }
            }
            Some(_) => {}
        }
        let sha = if execute {
            Some(ignis_core::hashing::file_sha256(&local)?)
        } else {
            None
        };
        if let (Some(e), Some(h)) = (fixes.get(&(rf.file_name.clone(), rf.file_version)), &sha) {
            if rf.size_bytes != Some(md.len() as i64) && *h != e.sha256 {
                r.size_mismatch.push(SizeMismatch {
                    file_name: rf.file_name.clone(),
                    listed: rf.size_bytes.unwrap_or(-1),
                    on_disk: md.len(),
                });
                continue;
            }
        }
        r.adopted += 1;
        if !execute {
            continue;
        }
        let got = Fetched {
            path: local.clone(),
            sha256: sha.expect("hashed when executing"),
            size_bytes: md.len() as i64,
            media_type: None,
            retrieved_at: mtime_utc(&md),
        };
        records.push(mf::source_record(
            rf,
            &got,
            &iv,
            &rights,
            info["doi"].as_str(),
        ));
        if records.len() % CHECKPOINT_EVERY == 0 {
            mf::write_manifest(inv, &records)?;
        }
    }
    if execute && !records.is_empty() {
        mf::write_manifest(inv, &records)?;
    }
    if execute {
        r.pruned_duplicate_versions = prune_duplicate_versions(inv)?;
    }
    Ok(r)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Coverage {
    pub investigation: String,
    pub acronym: String,
    /// NASA's licence identifier; absent = blank in NASA's metadata (to verify before use).
    pub licence: Option<String>,
    pub lockbox: bool,
    /// Stored files by the atlas rule (the target).
    pub stored_files: usize,
    /// Present and recorded with a hash.
    pub recorded: usize,
    pub recorded_bytes: u64,
    /// Present on disk but not recorded (run `sources adopt`), or refused by it.
    pub unrecorded_present: usize,
    /// Not on disk.
    pub missing: Vec<String>,
    /// Listed files that stay at NASA (media, archives): manifest entries only.
    pub manifest_only: usize,
}

pub fn coverage_of(f: &Flight) -> anyhow::Result<Coverage> {
    let inv = f.accession;
    let all = listing(inv)?;
    let stored = stored_files(inv)?;
    let have = mf::by_name(inv)?;
    let mut c = Coverage {
        investigation: inv.into(),
        acronym: f.acronym.into(),
        licence: f.licence.map(String::from),
        lockbox: f.lockbox,
        stored_files: stored.len(),
        recorded: 0,
        recorded_bytes: 0,
        unrecorded_present: 0,
        missing: vec![],
        manifest_only: {
            let mut seen = BTreeSet::new();
            all.iter()
                .filter(|x| {
                    storage_class(&x.category, &x.file_name) == StorageClass::ManifestOnly
                        && seen.insert((x.file_name.clone(), x.file_version))
                })
                .count()
        },
    };
    for rf in &stored {
        let local = paths::raw(Some(inv)).join(&rf.file_name);
        let recorded = have
            .get(&rf.file_name)
            .filter(|s| s.file_version == rf.file_version.map(|v| v.to_string()));
        match (recorded, local.exists()) {
            (Some(s), true) => {
                c.recorded += 1;
                c.recorded_bytes += s.size_bytes.unwrap_or(0).max(0) as u64;
            }
            (_, true) => c.unrecorded_present += 1,
            (_, false) => c.missing.push(rf.file_name.clone()),
        }
    }
    Ok(c)
}

/// Coverage of the 19 flight investigations and the identity of their manifests.
pub fn coverage() -> anyhow::Result<(Vec<Coverage>, String)> {
    let rows = FLIGHTS
        .iter()
        .map(coverage_of)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let invs: Vec<&str> = FLIGHTS.iter().map(|f| f.accession).collect();
    Ok((rows, mf::manifest_hash(&invs)?))
}

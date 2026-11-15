//! Where each source version lives at its provider, from the R6 manifests (`data/manifests`).
//!
//! * `<ACC>.jsonl`: fetched PSI assets ([`Source`] records, id = sha256 of the bytes);
//! * `<ACC>.listing.jsonl`: NASA's listing of every PSI file, fetched or not. A file kept only as a
//!   manifest entry (images, video, archives) has no byte hash, so its id is the listing id of its
//!   stable geode URL and size (`IdBasis::Listing`, the convention of `ignis_data::sources::intl`);
//! * `ntrs/*.jsonl`: NTRS discovery entries (metadata-record id, PDF address).
//!
//! Only metadata is held (a few tens of MB for the 19 investigations); no NASA bytes. An id that is
//! not in the manifests is unknown and refused: an address is never guessed from an id.

use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

use ignis_core::contracts::{Source, SourceId};
use ignis_data::sources::{intl::common::listing_id, ntrs, psi};
use serde::Deserialize;
use serde_json::{json, Value};

/// One PSI file version, the key of a signed URL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PsiFile {
    pub investigation: String,
    pub file_name: String,
    pub file_version: Option<i64>,
}

impl PsiFile {
    pub fn stable_url(&self) -> String {
        psi::stable_url(&self.investigation, &self.file_name, self.file_version)
    }
    pub fn is_ts(&self) -> bool {
        self.file_name.to_ascii_lowercase().ends_with(".ts")
    }
}

/// How the bytes of one source version are reached.
#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    /// Signed S3 URL resolved through geode at view time.
    Psi {
        file: PsiFile,
        size_bytes: Option<u64>,
    },
    /// NTRS PDF (no CORS): through the Worker proxy. `None` when NTRS offers no download.
    Ntrs { pdf_url: Option<String> },
    /// A permanent, unsigned URL on a host whose Range/CORS behaviour was measured.
    Permanent {
        url: String,
        range: bool,
        cors: bool,
    },
    /// Only the provider's page.
    LinkOut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub address: Address,
    /// "Open at NASA".
    pub landing_url: String,
    pub investigation: Option<String>,
}

/// Hosts of permanent URLs measured 2026-11-14 (LANES_V2 §10): `(host, range, cors)`.
const PERMANENT_HOSTS: [(&str, bool, bool); 1] = [("images-assets.nasa.gov", true, true)];

fn host_of(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    Some(&rest[..end])
}

/// The PSI investigation page (the form cited in the project's documents).
pub fn psi_landing(investigation: &str) -> String {
    format!(
        "https://psi.nasa.gov/physci/repo/data/investigations/{}",
        psi::quote(investigation)
    )
}

#[derive(Debug, Default)]
pub struct Catalog {
    entries: HashMap<SourceId, Entry>,
    /// NTRS discovery entries by NTRS record id (for evidence-corpus aliases).
    ntrs_by_record: HashMap<String, Entry>,
    /// Lines that could not be read, by file (reported at load, never silently dropped).
    pub rejected: Vec<(String, usize)>,
}

#[derive(Deserialize)]
struct ListingRow {
    investigation: String,
    file_name: String,
    #[serde(default)]
    file_version: Option<i64>,
    #[serde(default)]
    size_bytes: Option<i64>,
    #[serde(default)]
    restricted: bool,
}

impl Catalog {
    pub fn get(&self, id: &SourceId) -> Option<&Entry> {
        self.entries.get(id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The listing id of a PSI manifest-only file.
    pub fn psi_listing_id(file: &PsiFile, size_bytes: Option<i64>) -> String {
        listing_id(json!({"url": file.stable_url(), "size": size_bytes}))
    }

    /// Read every manifest under `dir` (missing directory = empty catalog).
    pub fn load(dir: &Path) -> anyhow::Result<Catalog> {
        let mut cat = Catalog::default();
        let Ok(rd) = std::fs::read_dir(dir) else {
            return Ok(cat);
        };
        let mut files: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        files.sort();
        for p in &files {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name == "listing_errata.jsonl" {
                // R6's record of listings whose stated size NASA's bytes contradict (read by
                // `ignis sources`); not a source manifest.
                continue;
            } else if name.ends_with(".listing.jsonl") {
                cat.read_lines(p, Catalog::listing_row)?;
            } else if name.ends_with(".jsonl") {
                cat.read_lines(p, Catalog::source_row)?;
            }
        }
        if let Ok(rd) = std::fs::read_dir(dir.join("ntrs")) {
            let mut ntrs: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            ntrs.sort();
            for p in ntrs
                .iter()
                .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
            {
                cat.read_lines(p, Catalog::ntrs_row)?;
            }
        }
        // The evidence corpus (`data/corpus/manifest.jsonl`, next to the manifests directory)
        // cites its documents by byte hash; a document fetched from NTRS is reached through the
        // NTRS discovery record with the same record id (its PDF address), else it links out to
        // its NTRS landing page. An address is still never built from an id.
        if let Some(corpus) = dir
            .parent()
            .map(|d| d.join("corpus").join("manifest.jsonl"))
        {
            if corpus.is_file() {
                cat.read_lines(&corpus, Catalog::corpus_row)?;
            }
        }
        Ok(cat)
    }

    fn corpus_row(&mut self, line: &str) -> Option<()> {
        let v: Value = serde_json::from_str(line).ok()?;
        let Some(id) = v.get("source_id").and_then(Value::as_str) else {
            // A seed without bytes (discovered, metadata only) has nothing to open.
            return Some(());
        };
        let Some(record) = v.get("ntrs_id").and_then(Value::as_str) else {
            // Not an NTRS document: the PSI manifests address it (or nothing does).
            return Some(());
        };
        let entry = match self.ntrs_by_record.get(record) {
            Some(e) => e.clone(),
            None => {
                let landing = v
                    .get("canonical_url")
                    .and_then(Value::as_str)
                    .filter(|u| u.starts_with("https://ntrs.nasa.gov/"))?;
                Entry {
                    address: Address::LinkOut,
                    landing_url: landing.to_string(),
                    investigation: None,
                }
            }
        };
        self.insert(id, entry)
    }

    fn read_lines(
        &mut self,
        path: &Path,
        row: fn(&mut Catalog, &str) -> Option<()>,
    ) -> anyhow::Result<()> {
        let f = std::io::BufReader::new(std::fs::File::open(path)?);
        let mut bad = 0;
        for line in f.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if row(self, &line).is_none() {
                bad += 1;
            }
        }
        if bad > 0 {
            self.rejected.push((path.display().to_string(), bad));
        }
        Ok(())
    }

    fn insert(&mut self, id: &str, e: Entry) -> Option<()> {
        let id = SourceId::new(id).ok()?;
        self.entries.entry(id).or_insert(e);
        Some(())
    }

    fn listing_row(&mut self, line: &str) -> Option<()> {
        let r: ListingRow = serde_json::from_str(line).ok()?;
        if r.restricted {
            // Not public at NASA: nothing to open (counted as present, not as an error).
            return Some(());
        }
        let file = PsiFile {
            investigation: r.investigation.clone(),
            file_name: r.file_name,
            file_version: r.file_version,
        };
        let id = Catalog::psi_listing_id(&file, r.size_bytes);
        self.insert(
            &id,
            Entry {
                address: Address::Psi {
                    file,
                    size_bytes: r.size_bytes.and_then(|s| u64::try_from(s).ok()),
                },
                landing_url: psi_landing(&r.investigation),
                investigation: Some(r.investigation),
            },
        )
    }

    fn source_row(&mut self, line: &str) -> Option<()> {
        let s: Source = serde_json::from_str(line).ok()?;
        let entry = Catalog::entry_of(&s, None)?;
        self.insert(&s.id, entry)
    }

    fn ntrs_row(&mut self, line: &str) -> Option<()> {
        let v: Value = serde_json::from_str(line).ok()?;
        let s: Source = serde_json::from_value(v.get("source")?.clone()).ok()?;
        let pdf = v
            .pointer("/record/pdf_url")
            .and_then(Value::as_str)
            .map(String::from);
        let entry = Catalog::entry_of(&s, Some(pdf))?;
        if let Some(record) = v.pointer("/record/ntrs_id").and_then(Value::as_str) {
            self.ntrs_by_record
                .entry(record.to_string())
                .or_insert_with(|| entry.clone());
        }
        self.insert(&s.id, entry)
    }

    /// The address of a `Source` record; `None` when it names no reachable location.
    fn entry_of(s: &Source, ntrs_pdf: Option<Option<String>>) -> Option<Entry> {
        let canonical = s
            .canonical_url
            .clone()
            .filter(|u| u.starts_with("https://"));
        if let (Some(inv), Some(name)) = (&s.investigation_id, &s.file_name) {
            if canonical
                .as_deref()
                .is_some_and(|u| u.starts_with(psi::BASE))
            {
                let file = PsiFile {
                    investigation: inv.clone(),
                    file_name: name.clone(),
                    file_version: s.file_version.as_deref().and_then(|v| v.parse().ok()),
                };
                // The record's identity URL must be the one geode resolves; otherwise refuse.
                if canonical.as_deref() != Some(file.stable_url().as_str()) {
                    return None;
                }
                return Some(Entry {
                    address: Address::Psi {
                        file,
                        size_bytes: s.size_bytes.and_then(|v| u64::try_from(v).ok()),
                    },
                    landing_url: psi_landing(inv),
                    investigation: Some(inv.clone()),
                });
            }
        }
        if let Some(n) = &s.ntrs_id {
            return Some(Entry {
                address: Address::Ntrs {
                    pdf_url: ntrs_pdf.flatten(),
                },
                landing_url: canonical.unwrap_or_else(|| ntrs::landing_url(n)),
                investigation: s.investigation_id.clone(),
            });
        }
        let url = canonical?;
        let address =
            match host_of(&url).and_then(|h| PERMANENT_HOSTS.iter().find(|(p, ..)| *p == h)) {
                Some(&(_, range, cors)) => Address::Permanent {
                    url: url.clone(),
                    range,
                    cors,
                },
                None => Address::LinkOut,
            };
        Some(Entry {
            address,
            landing_url: url,
            investigation: s.investigation_id.clone(),
        })
    }
}

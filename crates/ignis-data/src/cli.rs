//! `ignis sources ...` and `ignis data ...` command groups.

use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::Subcommand;
use ignis_core::contracts::Source;
use ignis_core::resources::{guard, load_profile, Guard, Job};
use regex::Regex;
use serde_json::{json, Value};

use crate::families::{self, family_of, NAMES};
use crate::sources::intl::{self, common as ic, darts, hreda, mendeley, zenodo};
use crate::sources::psi::{dump_listing, flatten_listing, save_meta, PsiClient, RemoteFile};
use crate::sources::{manifest as mf, samples as smp, storage, INVESTIGATIONS};
use crate::text::py_dumps_ordered_indent;
use crate::{auto, cohort, paths, review, store, views};

#[derive(Subcommand, Debug)]
pub enum SourcesCmd {
    /// Fetch investigation metadata + file listing (read-only) and write the listing manifest.
    Inspect { investigations: Vec<String> },
    /// Fetch the bounded representative sample set (< ~150 MB total) and update manifests.
    Samples { investigations: Vec<String> },
    /// Full / bulk fetch. Dry-run by default (resource guard); prints bytes and estimated time.
    Fetch {
        investigation: String,
        /// Regex on file name.
        #[arg(long, default_value = ".*")]
        pattern: String,
        /// PSI category, e.g. 'Raw Data'.
        #[arg(long)]
        category: Option<String>,
        /// Only the files the server stores by the atlas rule (documents and sensor text; media and
        /// archives stay at NASA). See `sources::storage`.
        #[arg(long)]
        stored: bool,
        #[arg(long, default_value = "cpu-local")]
        profile: String,
        /// Assumed throughput for the estimate.
        #[arg(long, default_value_t = 20.0)]
        mb_per_s: f64,
        /// Actually download (default: dry-run plan).
        #[arg(long)]
        execute: bool,
    },
    /// Print the Source manifest; with --verify, check local bytes against the recorded sha256.
    Manifest {
        investigation: String,
        #[arg(long)]
        verify: bool,
    },
    /// NTRS discovery (read-only, rate-limited by the provider's headers): every record of a query
    /// into `data/manifests/ntrs/<name>.jsonl`, raw records next to the corpus sidecars. Records are
    /// metadata sources until fetched; a query match is not an investigation identity.
    Ntrs {
        query: String,
        /// Manifest file stem; default: the query, slugged.
        #[arg(long)]
        name: Option<String>,
        #[arg(long, default_value_t = 100)]
        page_size: u32,
    },
    /// International microgravity-combustion sources (separate investigations, never NASA cohorts).
    #[command(subcommand)]
    Intl(IntlCmd),
}

#[derive(Subcommand, Debug)]
pub enum IntlCmd {
    /// Registered international investigations and their manifest status.
    List,
    /// Summarise a manifest; for JAXA-GCEM also parse a fetched telemetry ZIP.
    Inspect {
        inv: String,
        /// JAXA-GCEM: telemetry ZIP under data/raw/intl to parse.
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Rebuild manifest(s) from the live public metadata (listings/APIs only; no bulk download).
    Manifest {
        /// Investigation id, or 'all'.
        inv: String,
        /// JAXA: skip HEAD size requests (sizes become null).
        #[arg(long)]
        no_head: bool,
    },
    /// Download selected small files into data/raw/intl/<id>/ (guarded, verified against listed size/checksum).
    Fetch {
        inv: String,
        /// Glob over manifest file_name (default: probe set); repeatable.
        #[arg(long)]
        select: Vec<String>,
        /// Byte cap (default: per-investigation probe cap).
        #[arg(long)]
        max_bytes: Option<u64>,
        #[arg(long)]
        execute: bool,
        #[arg(long, default_value = "cpu-local")]
        profile: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum DataCmd {
    /// Parse fetched samples into typed records (data/work/records/<family>/); applies adjudications.
    Ingest {
        #[arg(default_value = "all")]
        family: String,
    },
    /// Print the FieldRule table (method per produced field).
    Rules { family: String },
    /// Cohorts from declared criteria.
    #[command(subcommand)]
    Cohort(CohortCmd),
    /// Grouped, frozen splits.
    #[command(subcommand)]
    Split(SplitCmd),
    /// Build a DatasetBundle (Parquet views + units + time origin + quality report) from the
    /// frozen split of a family. Builder bundles never contain `test`; `--evaluator` writes the
    /// separate evaluator bundle under the lockbox root (run by the evaluator, not by builders).
    Bundle {
        /// flex | acme_normal | acme_inverse
        family: String,
        /// partitions of a builder bundle, comma-separated
        #[arg(long, default_value = "train,validation")]
        partitions: String,
        #[arg(long)]
        evaluator: bool,
    },
    /// List the builder bundles in the store (ids, partitions, content digest).
    Bundles,
    /// Write the array views for a cohort as Parquet (physical SI scaling only).
    Views {
        cohort_id: String,
        /// Also time a full reload through the ViewSource.
        #[arg(long)]
        time_load: bool,
    },
    /// Semantic review items and adjudications.
    #[command(subcommand)]
    Review(ReviewCmd),
}

#[derive(Subcommand, Debug)]
pub enum CohortCmd {
    /// Apply the family's declared criteria; prints the inclusion/exclusion flow table.
    Build { family: String },
}

#[derive(Subcommand, Debug)]
pub enum SplitCmd {
    /// Freeze a 60/20/20 grouped split of a cohort (refuses to refreeze an existing id).
    Freeze {
        cohort_id: String,
        /// Recorded in the split.
        #[arg(long)]
        seed: i64,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReviewCmd {
    /// Write data/review/<family>/items.jsonl from the last ingest.
    Export { family: String },
    /// Validate and append an adjudication JSONL (agent/reviewer required); re-run ingest to apply.
    Import { family: String, path: PathBuf },
    /// Adjudicate the queue with `claude -p` (two independent passes, cited evidence). Dry-run by default.
    Auto {
        family: String,
        /// This shard of a stable partition, e.g. 2/4.
        #[arg(long, default_value = "1/1")]
        shard: String,
        #[arg(long, default_value = "opus")]
        model: String,
        #[arg(long, default_value_t = 2)]
        passes: usize,
        #[arg(long, default_value_t = 2)]
        concurrency: usize,
        /// Stop after K items of this shard.
        #[arg(long)]
        limit: Option<usize>,
        /// Only items whose id matches this regex.
        #[arg(long)]
        only: Option<String>,
        /// Write each rendered dossier into this directory.
        #[arg(long)]
        dump_dossiers: Option<PathBuf>,
        /// Actually call the model (default: dry-run plan).
        #[arg(long)]
        execute: bool,
        /// After the run, merge all shards and re-run ingest and cohort.
        #[arg(long)]
        import: bool,
    },
    /// Merge every drafts/auto-*.jsonl shard, record the adjudications, re-run ingest and cohort.
    AutoImport { family: String },
}

fn auto_import(family: &str) -> anyhow::Result<()> {
    let (merged, duplicates) = auto::merge_shards(family)?;
    if !duplicates.is_empty() {
        eprintln!("warning: {} items appear in more than one shard file; the first file in sorted order wins", duplicates.len());
    }
    let existing =
        crate::common::read_jsonl_values(&paths::review(family).join("adjudications.jsonl"))?;
    let new = auto::new_adjudications(merged, &existing);
    let n = new.len();
    if n > 0 {
        let tmp =
            std::env::temp_dir().join(format!("ignis-auto-import-{}.jsonl", std::process::id()));
        crate::common::write_jsonl(&tmp, &new)?;
        let imported = review::import_file(family, &tmp);
        let _ = std::fs::remove_file(&tmp);
        imported?;
    }
    println!("{family}: {n} new adjudications recorded");
    ingest_family(family)?;
    build_cohort(family)
}

fn ingest_family(fam: &str) -> anyhow::Result<()> {
    let spec = families::get(family_of(fam)?)?;
    let mut b = (spec.ingest)(spec.family)?;
    let applied = review::apply(&mut b)?;
    let n = store::save(&b)?;
    println!(
        "{fam}: {} identity={} adjudications_applied={applied}",
        store::counts_repr(&n),
        resolution_counts(&b)
    );
    Ok(())
}

fn build_cohort(fam: &str) -> anyhow::Result<()> {
    let (coh, p) = cohort::build(fam)?;
    println!(
        "{fam}: cohort {}  included={} excluded={}  -> {}",
        coh.id,
        coh.run_ids.len(),
        coh.excluded.len(),
        p.display()
    );
    let mut counts: indexmap::IndexMap<String, usize> = indexmap::IndexMap::new();
    for e in &coh.excluded {
        *counts
            .entry(e.reason.split(" (").next().unwrap().to_string())
            .or_default() += 1;
    }
    let mut v: Vec<(String, usize)> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1)); // stable: ties keep first-seen order (Counter.most_common)
    for (why, n) in v {
        println!("   excluded {n:>4}  {why}");
    }
    Ok(())
}

fn families_of(f: &str) -> Vec<String> {
    if f == "all" {
        NAMES.iter().map(|s| s.to_string()).collect()
    } else {
        vec![f.to_string()]
    }
}

/// `datetime.now(UTC).isoformat()`.
fn slug(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn run_ntrs(query: &str, name: Option<&str>, page_size: u32) -> anyhow::Result<()> {
    use crate::sources::ntrs;
    let stem = name.map(String::from).unwrap_or_else(|| slug(query));
    anyhow::ensure!(!stem.is_empty(), "empty manifest name");
    let mut c = ntrs::NtrsClient::new(120);
    let records = c.search_all(query, page_size)?;
    let entries = ntrs::entries(query, &records, &now_iso_offset())?;
    let sidecars = paths::root().join("raw").join("evidence").join("ntrs");
    std::fs::create_dir_all(&sidecars)?;
    for (v, e) in records.iter().zip(&entries) {
        let p = sidecars.join(format!("NTRS-{}.json", e.record.ntrs_id));
        if !p.exists() {
            std::fs::write(&p, serde_json::to_string_pretty(v)? + "\n")?;
        }
    }
    let out = paths::manifests()
        .join("ntrs")
        .join(format!("{stem}.jsonl"));
    std::fs::create_dir_all(out.parent().unwrap())?;
    let mut body = String::new();
    for e in &entries {
        body.push_str(&serde_json::to_string(e)?);
        body.push('\n');
    }
    std::fs::write(&out, body)?;
    let count = |f: &dyn Fn(&ntrs::NtrsEntry) -> bool| entries.iter().filter(|e| f(e)).count();
    println!(
        "{}",
        json!({
            "query": query, "manifest": out.display().to_string(), "records": entries.len(),
            "rights_verified": count(&|e| e.source.rights.status == ignis_core::contracts::RightsStatus::Verified),
            "with_pdf": count(&|e| e.record.pdf_url.is_some()),
            "with_fulltext": count(&|e| e.record.fulltext_url.is_some()),
        })
    );
    Ok(())
}

fn now_iso_offset() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.6f+00:00")
        .to_string()
}

// ------------------------------------------------------------------------------------------ sources

/// The listing of `inv`: the raw PSI response saved by `sources inspect` when present, else the
/// committed `data/manifests/<inv>.listing.jsonl` (same rows), so a fresh checkout can fetch
/// without calling the metadata endpoint again.
fn listing_local(inv: &str) -> anyhow::Result<Vec<RemoteFile>> {
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
    crate::sources::psi::read_listing(&committed)
}

fn inspect_one(c: &PsiClient, inv: &str) -> anyhow::Result<Vec<RemoteFile>> {
    let (meta, raw) = c.metadata(inv)?;
    let now = now_iso_offset();
    let (_, meta_sha) = save_meta(&paths::meta(), &format!("{inv}.json"), &raw)?;
    let version = meta["version"]
        .as_i64()
        .or_else(|| meta["version"].as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0);
    let (files, raw_list) = c.listing(inv, version)?;
    save_meta(&paths::meta(), &format!("files_{inv}.json"), &raw_list)?;
    mf::write_investigation(inv, &meta, &meta_sha, &now)?;
    let out = paths::manifests().join(format!("{inv}.listing.jsonl"));
    std::fs::write(out, dump_listing(&files))?;
    Ok(files)
}

/// Records written to the manifest between files of a bulk fetch.
const CHECKPOINT_EVERY: usize = 100;

fn fetch_all(
    c: &PsiClient,
    inv: &str,
    wanted: &[RemoteFile],
    cap: Option<i64>,
) -> anyhow::Result<Vec<Source>> {
    let info = mf::read_investigation(inv)?;
    let rights = mf::rights_from_license(
        info["license"].as_str(),
        info["metadata_url_sha256"].as_str().unwrap_or_default(),
    );
    let have = mf::by_name(inv)?;
    let iv = info["investigation_version"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let mut records = vec![];
    for rf in wanted {
        let local = paths::raw(Some(inv)).join(&rf.file_name);
        if let Some(src) = have.get(&rf.file_name) {
            if local.exists() && src.file_version == rf.file_version.map(|v| v.to_string()) {
                records.push(src.clone());
                continue;
            }
        }
        let got = c.fetch(rf, &paths::raw(Some(inv)), cap)?;
        records.push(mf::source_record(
            rf,
            &got,
            &iv,
            &rights,
            info["doi"].as_str(),
        ));
        // Checkpoint: an interrupted bulk fetch keeps what it hashed (write_manifest merges).
        if records.len() % CHECKPOINT_EVERY == 0 {
            mf::write_manifest(inv, &records)?;
        }
    }
    mf::write_manifest(inv, &records)?;
    Ok(records)
}

fn flex_stage2(files: &[RemoteFile]) -> anyhow::Result<Vec<RemoteFile>> {
    let mut sheets = vec![];
    for f in files {
        let p = paths::raw(Some("PSI-69")).join(&f.file_name);
        if f.file_name.ends_with("_TestSheet.txt") && p.exists() {
            let ts = families::flex::parse_testsheet(&std::fs::read(&p)?);
            if crate::text::lower(ts.get("fuel").unwrap_or("")).starts_with("methanol") {
                sheets.push((f.file_name.clone(), ts));
            }
        }
    }
    sheets.sort_by(|a, b| a.0.cmp(&b.0));
    // Deterministic spread: first two methanol runs of every 6th test date (sorted), up to 14 runs.
    let mut by_date: BTreeMap<String, Vec<&(String, families::flex::Sheet)>> = BTreeMap::new();
    for s in &sheets {
        by_date
            .entry(
                s.1.get("test_date")
                    .filter(|d| !d.is_empty())
                    .unwrap_or("?")
                    .to_string(),
            )
            .or_default()
            .push(s);
    }
    let mut dates: Vec<&String> = by_date.keys().collect();
    dates.sort_by(|a, b| by_date[*a][0].0.cmp(&by_date[*b][0].0));
    let num_re = Regex::new(r"FLEX-(\d+)_").unwrap();
    let mut chosen: BTreeMap<String, String> = BTreeMap::new();
    for d in dates.iter().step_by(6) {
        for (name, ts) in by_date[*d].iter().take(2).map(|x| (&x.0, &x.1)) {
            let num = num_re.captures(name).unwrap()[1].to_string();
            chosen.insert(num, ts.get("cir_test_number").unwrap_or("").to_string());
        }
    }
    let nums: Vec<String> = chosen.keys().take(14).cloned().collect();
    Ok(smp::flex_stage2(files, &nums, &chosen))
}

pub fn run_sources(cmd: SourcesCmd) -> anyhow::Result<()> {
    let invs = |v: Vec<String>| {
        if v.is_empty() {
            INVESTIGATIONS.iter().map(|s| s.to_string()).collect()
        } else {
            v
        }
    };
    match cmd {
        SourcesCmd::Inspect { investigations } => {
            let c = PsiClient::new(120);
            for inv in invs(investigations) {
                let files = inspect_one(&c, &inv)?;
                let info = mf::read_investigation(&inv)?;
                let total: i64 = files.iter().filter_map(|f| f.size_bytes).sum();
                println!(
                    "{inv} v{} {} doi:{} files={} total={:.1} GB",
                    info["investigation_version"].as_str().unwrap_or(""),
                    info["license"].as_str().unwrap_or("None"),
                    info["doi"].as_str().unwrap_or("None"),
                    files.len(),
                    total as f64 / 1e9
                );
                let mut cats: Vec<(String, usize, i64)> = vec![];
                for f in &files {
                    match cats.iter_mut().find(|c| c.0 == f.category) {
                        Some(c) => {
                            c.1 += 1;
                            c.2 += f.size_bytes.unwrap_or(0);
                        }
                        None => cats.push((f.category.clone(), 1, f.size_bytes.unwrap_or(0))),
                    }
                }
                cats.sort_by(|a, b| b.1.cmp(&a.1));
                for (cat, n, size) in cats {
                    println!("   {cat:<32} {n:>6}  {:>12.1} MB", size as f64 / 1e6);
                }
            }
        }
        SourcesCmd::Samples { investigations } => {
            let c = PsiClient::new(120);
            for inv in invs(investigations) {
                let files = listing_local(&inv)?;
                let wanted = match inv.as_str() {
                    "PSI-69" => {
                        let (ok, _) = smp::capped(smp::flex_stage1(&files));
                        fetch_all(&c, &inv, &ok, Some(smp::MAX_FILE_BYTES))?;
                        flex_stage2(&files)?
                    }
                    "PSI-159" => smp::acme(&files),
                    _ => smp::bass(&files),
                };
                let (ok, skipped) = smp::capped(wanted);
                let recs = fetch_all(&c, &inv, &ok, Some(smp::MAX_FILE_BYTES))?;
                let mb: i64 = recs.iter().filter_map(|r| r.size_bytes).sum();
                let skip = if skipped.is_empty() {
                    String::new()
                } else {
                    format!(
                        "; skipped (cap/restricted): {:?}",
                        skipped.iter().map(|f| &f.file_name).collect::<Vec<_>>()
                    )
                };
                println!(
                    "{inv}: {} sample files, {:.1} MB{skip}",
                    recs.len(),
                    mb as f64 / 1e6
                );
            }
        }
        SourcesCmd::Fetch {
            investigation,
            pattern,
            category,
            stored,
            profile,
            mb_per_s,
            execute,
        } => {
            let rx = Regex::new(&pattern)?;
            // NASA's listings repeat some rows: each (file name, version) is fetched once.
            let mut seen = std::collections::BTreeSet::new();
            let files: Vec<RemoteFile> = listing_local(&investigation)?
                .into_iter()
                .filter(|f| {
                    rx.is_match(&f.file_name)
                        && category.as_ref().is_none_or(|c| *c == f.category)
                        && !f.restricted
                        && (!stored
                            || storage::storage_class(&f.category, &f.file_name)
                                != storage::StorageClass::ManifestOnly)
                        && seen.insert((f.file_name.clone(), f.file_version))
                })
                .collect();
            let total: i64 = files.iter().filter_map(|f| f.size_bytes).sum();
            let prof = load_profile(&profile)?;
            let g = guard(
                &format!("sources.fetch {investigation}"),
                &prof,
                total as f64 / (mb_per_s * 1e6) / 3600.0,
                execute,
                json!({"files": files.len(), "bytes": total, "dest": paths::raw(Some(&investigation)).display().to_string()}),
            )?;
            if g == Guard::DryRun {
                return Ok(());
            }
            let job = Job::start(
                "sources.fetch",
                &prof,
                json!({"investigation": investigation, "pattern": pattern, "files": files.len()}),
            )?;
            let recs = fetch_all(&PsiClient::new(600), &investigation, &files, None)?;
            job.finish(json!({"records": recs.len()}));
        }
        SourcesCmd::Manifest {
            investigation,
            verify,
        } => {
            let mut bad = 0;
            for s in mf::read_manifest(&investigation)? {
                let mut status = String::new();
                if verify {
                    let p = mf::local_path(&s);
                    status = if !p.exists() {
                        "missing".into()
                    } else if format!("sha256:{}", ignis_core::hashing::file_sha256(&p)?) == s.id {
                        "ok".into()
                    } else {
                        "MISMATCH".into()
                    };
                    bad += (status != "ok") as usize;
                }
                println!(
                    "{}  v{}  {:>10}  {}  {} {status}",
                    &s.id[..19.min(s.id.len())],
                    s.file_version.as_deref().unwrap_or("None"),
                    s.size_bytes
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| "None".into()),
                    s.rights.license.as_deref().unwrap_or("None"),
                    s.file_name.as_deref().unwrap_or("")
                );
            }
            if verify && bad > 0 {
                anyhow::bail!("{bad} manifest entries missing or mismatched");
            }
        }
        SourcesCmd::Ntrs {
            query,
            name,
            page_size,
        } => run_ntrs(&query, name.as_deref(), page_size)?,
        SourcesCmd::Intl(c) => run_intl(c)?,
    }
    Ok(())
}

// --------------------------------------------------------------------------------------------- intl

fn check_inv(inv: &str) -> anyhow::Result<()> {
    let all = intl::investigations();
    anyhow::ensure!(
        all.iter().any(|(k, _)| k == inv),
        "unknown investigation {inv:?}; one of: {}",
        all.iter()
            .map(|(k, _)| k.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

fn load_intl(inv: &str) -> anyhow::Result<Vec<Value>> {
    let p = ic::manifest_path(inv, "");
    anyhow::ensure!(
        p.exists(),
        "no manifest at {}; run `ignis sources intl manifest {inv}`",
        p.display()
    );
    ic::read_jsonl(&p)
}

/// insertion-ordered counter as a JSON object.
fn counter<'a>(it: impl Iterator<Item = String> + 'a) -> Value {
    let mut m: indexmap::IndexMap<String, usize> = indexmap::IndexMap::new();
    for k in it {
        *m.entry(k).or_default() += 1;
    }
    json!(m)
}

fn run_intl(cmd: IntlCmd) -> anyhow::Result<()> {
    match cmd {
        IntlCmd::List => {
            for (inv, desc) in intl::investigations() {
                let p = ic::manifest_path(&inv, "");
                let n = if p.exists() {
                    ic::read_jsonl(&p)?.len()
                } else {
                    0
                };
                let st = if n > 0 {
                    format!("{n} lines")
                } else {
                    "no manifest".into()
                };
                println!("{inv:42} {st:>12}  {desc}");
            }
        }
        IntlCmd::Inspect { inv, file } => {
            check_inv(&inv)?;
            let rows = load_intl(&inv)?;
            let srcs: Vec<&Value> = rows.iter().filter(|r| r["record"] == "source").collect();
            let invrec = rows.iter().find(|r| r["record"] == "investigation");
            let mut inv_summary = Value::Null;
            let mut inv_extra = json!({});
            if let Some(ir) = invrec {
                let mut d = ir["data"].clone();
                d["source_ids"] = json!(ir["data"]["source_ids"]
                    .as_array()
                    .map(|a| a.len())
                    .unwrap_or(0));
                inv_summary = d;
                if let Some(e) = ir["extra"].as_object() {
                    inv_extra = Value::Object(
                        e.iter()
                            .filter(|(k, _)| *k != "creators")
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect(),
                    );
                }
            }
            let mut rights: Vec<String> = srcs
                .iter()
                .map(|r| {
                    let rr = &r["data"]["rights"];
                    format!(
                        "{{\"license\": {}, \"status\": {}}}",
                        crate::text::py_dumps(&rr["license"]),
                        crate::text::py_dumps(&rr["status"])
                    )
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            rights.sort();
            let listed: i64 = srcs
                .iter()
                .filter_map(|r| r["data"]["size_bytes"].as_i64())
                .sum();
            let s = |v: &Value| match v {
                Value::String(s) => s.clone(),
                Value::Null => "null".into(),
                v => v.to_string(),
            };
            let summary = py_dumps_ordered_indent(&[
                ("investigation", inv_summary),
                ("investigation_extra", inv_extra),
                ("records", counter(rows.iter().map(|r| s(&r["record"])))),
                (
                    "sources_by_kind",
                    counter(
                        srcs.iter()
                            .map(|r| s(r["extra"].get("kind").unwrap_or(&r["data"]["kind"]))),
                    ),
                ),
                (
                    "sources_by_id_basis",
                    counter(srcs.iter().map(|r| s(&r["id_basis"]))),
                ),
                ("listed_bytes", json!(listed)),
                ("rights", json!(rights)),
                (
                    "quality_flags",
                    counter(
                        rows.iter()
                            .filter(|r| r["record"] == "experiment")
                            .flat_map(|r| {
                                r["data"]["quality_flags"]
                                    .as_array()
                                    .cloned()
                                    .unwrap_or_default()
                            })
                            .map(|f| s(&f)),
                    ),
                ),
            ]);
            println!("{summary}");
            if inv == darts::INVESTIGATION_ID {
                let path =
                    file.unwrap_or_else(|| ic::raw_path(&inv, intl::probe(&inv).unwrap().0[4]));
                if path.exists() {
                    let sha = ic::local_sha256(&path).unwrap_or_default();
                    let (name, body) = darts::read_telemetry_zip(&std::fs::read(&path)?)?;
                    let stem = std::path::Path::new(&name)
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let fname = path.file_name().unwrap().to_string_lossy().to_string();
                    let tp = darts::parse_telemetry_csv(
                        &body,
                        &sha,
                        &format!("{fname}/{name}"),
                        &darts::run_id(&stem),
                        None,
                    )?;
                    let mut by: indexmap::IndexMap<String, (f64, f64, usize)> =
                        indexmap::IndexMap::new();
                    for o in &tp.observations {
                        let v = o.value_si.unwrap_or(f64::NAN);
                        let e = by
                            .entry(o.channel.clone().unwrap_or_default())
                            .or_insert((v, v, 0));
                        e.0 = e.0.min(v);
                        e.1 = e.1.max(v);
                        e.2 += 1;
                    }
                    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
                    let ch: serde_json::Map<String, Value> = by
                        .iter()
                        .map(|(k, v)| (k.clone(), json!([r3(v.0), r3(v.1), v.2])))
                        .collect();
                    let t0 = tp.t0.format("%Y-%m-%dT%H:%M:%S%.6f+00:00").to_string();
                    println!(
                        "{}",
                        py_dumps_ordered_indent(&[
                            ("telemetry_file", json!(path.display().to_string())),
                            ("zip_sha256", json!(sha)),
                            ("csv", json!(name)),
                            ("run_id", json!(tp.run_id)),
                            ("t0_gmt", json!(t0)),
                            ("rows", json!(tp.n_rows)),
                            ("observations", json!(tp.observations.len())),
                            ("span_s", json!(tp.observations.last().map(|o| o.time_s))),
                            ("channels_si_min_max_n", Value::Object(ch)),
                            ("igniter_stages", json!(tp.controls)),
                        ])
                    );
                } else {
                    println!(
                        "(no local telemetry at {}; `ignis sources intl fetch {inv} --execute`)",
                        path.display()
                    );
                }
            }
        }
        IntlCmd::Manifest { inv, no_head } => {
            let all: Vec<String> = intl::investigations().into_iter().map(|(k, _)| k).collect();
            let targets: Vec<String> = if inv == "all" {
                all.clone()
            } else {
                vec![inv.clone()]
            };
            for t in &targets {
                check_inv(t)?;
            }
            let mut built: indexmap::IndexMap<String, Vec<Value>> = indexmap::IndexMap::new();
            let has = |k: &str| targets.iter().any(|t| t == k);
            if has(darts::INVESTIGATION_ID) {
                built.insert(
                    darts::INVESTIGATION_ID.into(),
                    darts::build_manifest(!no_head)?,
                );
            }
            if has(zenodo::INVESTIGATION_ID) {
                built.insert(zenodo::INVESTIGATION_ID.into(), zenodo::build_manifest()?);
            }
            if hreda::TARGETS.iter().any(|t| has(t.0)) {
                for (k, v) in hreda::build_all()? {
                    if has(&k) {
                        built.insert(k, v);
                    }
                }
            }
            if has(mendeley::INVESTIGATION_ID) {
                built.insert(
                    mendeley::INVESTIGATION_ID.into(),
                    mendeley::build_manifest()?,
                );
            }
            for (k, rows) in built {
                let p = ic::manifest_path(&k, "");
                let n = ic::write_jsonl(&p, &rows)?;
                println!("{k}: wrote {n} lines to {}", p.display());
            }
        }
        IntlCmd::Fetch {
            inv,
            select,
            max_bytes,
            execute,
            profile,
        } => {
            check_inv(&inv)?;
            let Some((pats, cap0)) = intl::probe(&inv) else {
                anyhow::bail!("{inv}: access is login-gated (restricted); nothing can be fetched")
            };
            let pats: Vec<String> = if select.is_empty() {
                pats.iter().map(|s| s.to_string()).collect()
            } else {
                select
            };
            let cap = max_bytes.unwrap_or(cap0);
            let rows: Vec<Value> = load_intl(&inv)?
                .into_iter()
                .filter(|r| r["record"] == "source" && r["data"]["file_name"].is_string())
                .collect();
            let chosen: Vec<&Value> = rows
                .iter()
                .filter(|r| {
                    pats.iter()
                        .any(|p| intl::fnmatch(r["data"]["file_name"].as_str().unwrap(), p))
                })
                .collect();
            let unknown: Vec<&str> = chosen
                .iter()
                .filter(|r| r["data"]["size_bytes"].is_null())
                .map(|r| r["data"]["file_name"].as_str().unwrap())
                .collect();
            let total: u64 = chosen
                .iter()
                .filter_map(|r| r["data"]["size_bytes"].as_u64())
                .sum();
            let plan = json!({"files": chosen.len(), "total_bytes": total, "cap_bytes": cap, "unknown_size": unknown,
                              "dest": ic::raw_dir().join(&inv).display().to_string()});
            println!("total bytes: {total} in {} files (cap {cap})", chosen.len());
            if total > cap || !unknown.is_empty() {
                let mut p = plan.clone();
                p["refused"] = json!("exceeds byte cap or sizes unknown");
                println!("{}", serde_json::to_string_pretty(&p)?);
                anyhow::bail!("refused: exceeds byte cap or sizes unknown");
            }
            if guard(
                &format!("intl.fetch:{inv}"),
                &load_profile(&profile)?,
                0.01,
                execute,
                plan,
            )? == Guard::DryRun
            {
                return Ok(());
            }
            for r in chosen {
                let (d, x) = (&r["data"], &r["extra"]);
                let name = d["file_name"].as_str().unwrap();
                let dest = ic::raw_path(&inv, name);
                let size = d["size_bytes"].as_u64();
                if dest.exists() && std::fs::metadata(&dest)?.len() == size.unwrap_or(u64::MAX) {
                    println!("have  {name}");
                    continue;
                }
                let sha = if r["id_basis"] == "bytes" {
                    d["id"]
                        .as_str()
                        .and_then(|s| s.split_once(':'))
                        .map(|p| p.1.to_string())
                } else {
                    None
                };
                let got = ic::download(
                    d["canonical_url"].as_str().unwrap_or_default(),
                    &dest,
                    size,
                    x["md5"].as_str(),
                    sha.as_deref(),
                )?;
                println!("got   {name}  sha256:{got}");
            }
            println!(
                "rebuild the manifest to record byte hashes: ignis sources intl manifest {inv}"
            );
        }
    }
    Ok(())
}

// --------------------------------------------------------------------------------------------- data

fn resolution_counts(b: &crate::common::Bundle) -> String {
    let mut m: indexmap::IndexMap<String, usize> = indexmap::IndexMap::new();
    for r in &b.records {
        *m.entry(crate::common::enum_str(r.identity_resolution))
            .or_default() += 1;
    }
    store::counts_repr(&m)
}

pub fn run_data(cmd: DataCmd) -> anyhow::Result<()> {
    match cmd {
        DataCmd::Ingest { family } => {
            for fam in families_of(&family) {
                ingest_family(&fam)?;
            }
        }
        DataCmd::Rules { family } => {
            let fam = family_of(&family)?;
            let rows = (families::get(fam)?.rules)(fam);
            let w = rows.iter().map(|r| r.field.len()).max().unwrap_or(5);
            for r in rows {
                println!(
                    "{:<w$}  {:<13} {} v{}\n{:<w$}  {}",
                    r.field,
                    crate::common::enum_str(r.method),
                    r.method_id,
                    r.version,
                    "",
                    r.rationale
                );
            }
        }
        DataCmd::Cohort(CohortCmd::Build { family }) => {
            for fam in families_of(&family) {
                build_cohort(&fam)?;
            }
        }
        DataCmd::Split(SplitCmd::Freeze { cohort_id, seed }) => {
            let (s, p) = cohort::freeze(&cohort_id, seed)?;
            println!(
                "split {} groups train/val/test = {}/{}/{} -> {}",
                s.id,
                s.train.len(),
                s.validation.len(),
                s.test.len(),
                p.display()
            );
        }
        DataCmd::Bundle {
            family,
            partitions,
            evaluator,
        } => {
            use crate::dataset::{self, BundleSpec, DatasetStore};
            let fam: ignis_core::contracts::Family = serde_json::from_value(json!(family))?;
            let spec = if evaluator {
                BundleSpec::evaluator()
            } else {
                let ps = partitions
                    .split(',')
                    .map(|p| serde_json::from_value(json!(p.trim())))
                    .collect::<Result<Vec<ignis_core::contracts::Partition>, _>>()?;
                BundleSpec::builder(&ps)
            };
            let store = DatasetStore::default_roots();
            let (built, path) = dataset::build_and_save(fam, &spec, &store)?;
            let b = &built.bundle;
            let mut out = json!({"dataset_id": b.dataset_id, "audience": b.audience, "partitions": b.partitions,
                "content_digest": b.content_digest(), "bundle": path.display().to_string()});
            if !evaluator {
                // values of an evaluator bundle are never printed; builder summaries are
                let (copy, changed) = dataset::publish_builder_copy(b)?;
                out["committed_copy"] = json!(copy.display().to_string());
                out["copy_changed"] = json!(changed);
                out["time_origin"] = json!(b.time_origin);
                out["views"] = json!(b
                    .views
                    .iter()
                    .map(|v| json!({"name": v.name, "rows": v.row_count,
                    "columns": v.columns.len(), "artifact": v.artifact.artifact_id}))
                    .collect::<Vec<_>>());
                out["quality"] = built.quality.clone();
            }
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        DataCmd::Bundles => {
            use ignis_core::services::DatasetService;
            let store = crate::dataset::DatasetStore::default_roots();
            let scope = ignis_core::contracts::ExecutionScope {
                scope_id: ignis_core::contracts::ScopeId::new("local-cli")?,
                workspace_id: ignis_core::contracts::WorkspaceId::new("local")?,
                principal: ignis_core::contracts::PrincipalId::new("local-researcher")?,
                role: ignis_core::contracts::ScopeRole::Research,
                partitions: [
                    ignis_core::contracts::Partition::Train,
                    ignis_core::contracts::Partition::Validation,
                ]
                .into(),
                capabilities: [ignis_core::contracts::Capability::ReadData].into(),
                corpus_snapshot: None,
                parent_scope_id: None,
            };
            let mut rows = Vec::new();
            for id in store.list_builder()? {
                let b = store.dataset(&scope, &ignis_core::contracts::DatasetId::new(id)?);
                rows.push(match b {
                    Ok(b) => json!({"dataset_id": b.dataset_id, "family": b.family, "partitions": b.partitions,
                        "views": b.views.len(), "content_digest": b.content_digest()}),
                    Err(e) => json!({"error": e}),
                });
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"bundles": rows}))?
            );
        }
        DataCmd::Views {
            cohort_id,
            time_load,
        } => {
            let p = views::build(&cohort_id)?;
            println!("{}", p.display());
            if time_load {
                use ignis_core::views::ViewSource;
                let ds = views::DataStore::new();
                let coh = ds.cohort(&cohort_id)?;
                let t = std::time::Instant::now();
                let v = ds.views(coh.family, &cohort_id)?;
                let n: usize = v.iter().map(|x| x.n_observations()).sum();
                println!(
                    "loaded {} views ({n} valid observations) in {:.2} ms",
                    v.len(),
                    t.elapsed().as_secs_f64() * 1e3
                );
            }
        }
        DataCmd::Review(ReviewCmd::Export { family }) => {
            for fam in families_of(&family) {
                let p = review::export(&store::load(&fam)?)?;
                let n = std::fs::read_to_string(&p)?.lines().count();
                println!("{fam}: {n} items -> {}", p.display());
            }
        }
        DataCmd::Review(ReviewCmd::Import { family, path }) => {
            println!(
                "{} adjudications recorded for {family}",
                review::import_file(&family, &path)?
            );
        }
        DataCmd::Review(ReviewCmd::Auto {
            family,
            shard,
            model,
            passes,
            concurrency,
            limit,
            only,
            dump_dossiers,
            execute,
            import,
        }) => {
            let cfg = auto::AutoConfig {
                family: family.clone(),
                shard: auto::parse_shard(&shard)?,
                model,
                passes,
                concurrency,
                limit,
                only: only.as_deref().map(Regex::new).transpose()?,
                execute,
                dump_dossiers,
            };
            auto::run(&cfg)?;
            if import && execute {
                auto_import(&family)?;
            }
        }
        DataCmd::Review(ReviewCmd::AutoImport { family }) => auto_import(&family)?,
    }
    Ok(())
}

impl SourcesCmd {
    pub fn run(self) -> anyhow::Result<()> {
        run_sources(self)
    }
}

impl DataCmd {
    pub fn run(self) -> anyhow::Result<()> {
        run_data(self)
    }
}

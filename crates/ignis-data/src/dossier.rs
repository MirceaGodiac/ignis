//! Self-contained evidence dossiers for automatic adjudication ([`crate::auto`]).
//!
//! A dossier holds everything the model may use for one review item: the question, the candidates,
//! the cited source text and, per family, the table row, test sheet, series digest, script timeline
//! or file listing. Every line carries an id (`L0001`) and a source locator; the model must cite ids,
//! and code checks the citations. Only NASA source files, the committed manifests and the ingested
//! records are read here, never model outputs (`artifacts/`, `holdout/`, `runs/`).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Context;
use ignis_core::contracts::{EndpointRecord, ExperimentRecord, Quantity, Source, SourceLocator};
use regex::Regex;

use crate::common::{loc, read_jsonl, read_jsonl_values, ReviewItem};
use crate::families::{bass, flex};
use crate::paths;
use crate::sources::manifest as mf;
use crate::sources::psi::{read_listing, RemoteFile};
use crate::text::{
    decode_utf8_or_cp1252, decode_utf8_sig_lossy, normalize_newlines, strip, table_rows, Row,
};

/// Upper bound for the small files fetched only to complete dossiers.
pub const MAX_FETCH_BYTES: i64 = 200_000_000;

const FORBIDDEN_PATH_PARTS: [&str; 4] = [
    "artifacts/",
    "holdout/",
    "runs/evaluation",
    "runs\\evaluation",
];

#[derive(Debug, Clone)]
pub struct Line {
    pub id: String,
    pub section: usize,
    pub text: String,
    pub locator: Option<SourceLocator>,
}

#[derive(Debug, Clone, Default)]
pub struct Dossier {
    pub item_id: String,
    pub family: String,
    pub run_id: String,
    pub field: String,
    pub question: String,
    /// Concrete allowed values (BASS: real run ids instead of the `<run_id>` placeholder).
    pub candidates: Vec<String>,
    /// Definitions and routing notes; they cannot be cited.
    pub notes: Vec<String>,
    pub sections: Vec<String>,
    pub lines: Vec<Line>,
    /// Earliest and latest time of any series of the run (FLEX), for the validator.
    pub observed_range: Option<(f64, f64)>,
}

impl Dossier {
    pub fn new(item: &ReviewItem) -> Dossier {
        Dossier {
            item_id: item.id.clone(),
            family: item.family.clone(),
            run_id: item.run_id.clone(),
            field: item.field.clone(),
            question: item.question.clone(),
            candidates: item
                .candidates
                .iter()
                .filter(|c| *c != "<run_id>")
                .cloned()
                .collect(),
            ..Default::default()
        }
    }

    pub fn section(&mut self, title: impl Into<String>) {
        self.sections.push(title.into());
    }

    pub fn note(&mut self, text: impl Into<String>) {
        self.notes.push(text.into());
    }

    /// Adds a citable line (its locator must never point at model outputs).
    pub fn add(&mut self, text: impl Into<String>, locator: Option<SourceLocator>) {
        if let Some(l) = &locator {
            let path = l.path.as_deref().unwrap_or("");
            assert!(
                !FORBIDDEN_PATH_PARTS.iter().any(|p| path.contains(p)),
                "dossier refuses model outputs: {path}"
            );
        }
        if self.sections.is_empty() {
            self.sections.push("Evidence".into());
        }
        let id = format!("L{:04}", self.lines.len() + 1);
        self.lines.push(Line {
            id,
            section: self.sections.len() - 1,
            text: text.into(),
            locator,
        });
    }

    pub fn line(&self, id: &str) -> Option<&Line> {
        self.lines.iter().find(|l| l.id == id)
    }

    pub fn render(&self) -> String {
        let mut s = format!(
            "ITEM: {}\nFIELD: {}\nQUESTION: {}\nCANDIDATES: {}\n",
            self.item_id,
            self.field,
            self.question,
            self.candidates.join(" | ")
        );
        if !self.notes.is_empty() {
            s.push_str("NOTES (definitions, not evidence):\n");
            for n in &self.notes {
                s.push_str(&format!("- {n}\n"));
            }
        }
        let mut current = usize::MAX;
        for l in &self.lines {
            if l.section != current {
                current = l.section;
                s.push_str(&format!("\n=== {} ===\n", self.sections[current]));
            }
            s.push_str(&format!("[{}] {}\n", l.id, l.text));
        }
        s
    }
}

fn cache_dir(inv: &str) -> PathBuf {
    paths::root().join("cache").join("review").join(inv)
}

/// Reads NASA files from the manifest, the raw directory or the dossier cache, and knows the listing.
pub struct Assets {
    manifests: HashMap<String, BTreeMap<String, Source>>,
    listings: HashMap<String, Vec<RemoteFile>>,
    listing_sha: HashMap<String, String>,
}

impl Assets {
    pub fn load(invs: &[&str]) -> anyhow::Result<Assets> {
        let (mut manifests, mut listings, mut listing_sha) =
            (HashMap::new(), HashMap::new(), HashMap::new());
        for inv in invs {
            manifests.insert(inv.to_string(), mf::by_name(inv)?);
            let p = paths::manifests().join(format!("{inv}.listing.jsonl"));
            listings.insert(inv.to_string(), read_listing(&p)?);
            listing_sha.insert(
                inv.to_string(),
                ignis_core::hashing::file_sha256(&p).with_context(|| p.display().to_string())?,
            );
        }
        Ok(Assets {
            manifests,
            listings,
            listing_sha,
        })
    }

    /// Bytes and sha256 of a NASA file available locally.
    pub fn read(&self, inv: &str, name: &str) -> anyhow::Result<Option<(Vec<u8>, String)>> {
        if let Some(src) = self.manifests.get(inv).and_then(|m| m.get(name)) {
            let p = mf::local_path(src);
            if p.exists() {
                return Ok(Some((std::fs::read(&p)?, mf::sha_of(src))));
            }
        }
        for p in [paths::raw(Some(inv)).join(name), cache_dir(inv).join(name)] {
            if p.exists() {
                return Ok(Some((
                    std::fs::read(&p)?,
                    ignis_core::hashing::file_sha256(&p)?,
                )));
            }
        }
        Ok(None)
    }

    fn present(&self, inv: &str, name: &str) -> bool {
        let in_manifest = self
            .manifests
            .get(inv)
            .and_then(|m| m.get(name))
            .is_some_and(|s| mf::local_path(s).exists());
        in_manifest
            || paths::raw(Some(inv)).join(name).exists()
            || cache_dir(inv).join(name).exists()
    }

    fn listed(&self, inv: &str, name: &str) -> Option<(i64, &RemoteFile)> {
        self.listings
            .get(inv)?
            .iter()
            .enumerate()
            .find(|(_, f)| f.file_name == name)
            .map(|(i, f)| (i as i64 + 1, f))
    }

    fn listing_locator(&self, inv: &str, row: i64, name: &str) -> SourceLocator {
        loc(
            self.listing_sha.get(inv).map(|s| s.as_str()).unwrap_or(""),
            &format!("{inv}.listing.jsonl"),
            Some(row),
            Some(name),
        )
    }

    pub fn missing(&self, wanted: &[(String, String)]) -> Vec<RemoteFile> {
        let mut seen = BTreeSet::new();
        let mut out = vec![];
        for (inv, name) in wanted {
            if seen.insert((inv.clone(), name.clone())) && !self.present(inv, name) {
                if let Some((_, f)) = self.listed(inv, name).filter(|(_, f)| !f.restricted) {
                    out.push(f.clone());
                }
            }
        }
        out
    }
}

/// Fetches the small files the dossiers need into `data/cache/review/` (gitignored; manifests are
/// untouched so concurrent shards do not conflict and cohort ids do not depend on who ran what).
pub fn fetch_missing(files: &[RemoteFile]) -> anyhow::Result<usize> {
    if files.is_empty() {
        return Ok(0);
    }
    let client = crate::sources::psi::PsiClient::new(120);
    for f in files {
        let dest = cache_dir(&f.investigation);
        std::fs::create_dir_all(&dest)?;
        client.fetch(f, &dest, Some(MAX_FETCH_BYTES))?;
    }
    Ok(files.len())
}

pub fn investigations(family: &str) -> &'static [&'static str] {
    match family {
        "flex" => &["PSI-69"],
        "acme_normal" | "acme_inverse" => &["PSI-159"],
        _ => &["PSI-25", "PSI-26"],
    }
}

/// What the builder reads about the runs of a family (no observations: series come from raw files).
pub struct Ctx {
    pub assets: Assets,
    records: HashMap<String, ExperimentRecord>,
    endpoints: HashMap<String, EndpointRecord>,
}

impl Ctx {
    pub fn load(family: &str) -> anyhow::Result<Ctx> {
        let d = paths::work(&["records", family]);
        anyhow::ensure!(
            d.join("experiments.jsonl").exists(),
            "no ingested records for {family}; run `ignis data ingest {family}`"
        );
        let records: Vec<ExperimentRecord> = read_jsonl(&d.join("experiments.jsonl"))?;
        let endpoints: Vec<EndpointRecord> = read_jsonl(&d.join("endpoints.jsonl"))?;
        Ok(Ctx {
            assets: Assets::load(investigations(family))?,
            records: records.into_iter().map(|r| (r.id.clone(), r)).collect(),
            endpoints: endpoints
                .into_iter()
                .map(|e| (e.run_id.clone(), e))
                .collect(),
        })
    }

    fn alias(&self, run_id: &str, scheme: &str) -> Option<String> {
        self.records
            .get(run_id)?
            .aliases
            .iter()
            .find(|a| a.scheme == scheme)
            .map(|a| a.name.clone())
    }
}

pub fn read_items(family: &str) -> anyhow::Result<Vec<ReviewItem>> {
    read_jsonl_values(&paths::review(family).join("items.jsonl"))?
        .iter()
        .map(ReviewItem::from_json)
        .collect()
}

/// Small files worth having for these items (only those listed by NASA are ever requested).
pub fn wanted_files(ctx: &Ctx, items: &[&ReviewItem]) -> Vec<(String, String)> {
    let mut out = vec![];
    for it in items {
        match it.family.as_str() {
            "flex" => {
                let (Some(num), Some(cir)) = (
                    ctx.alias(&it.run_id, "file_flex_number"),
                    ctx.alias(&it.run_id, "cir_test_point"),
                ) else {
                    continue;
                };
                let mut names = vec![
                    format!("PSI-69_Raw Data_{num}_TestSheet.txt"),
                    format!("PSI-69_Raw Data_{cir}_MDCA_Testpoint_Settings.txt"),
                    format!("PSI-69_Raw Data_{cir}_Comments.txt"),
                ];
                if it.field == "flex_endpoint" {
                    names.push(format!("PSI-69_Raw Data_{cir}.txt"));
                    for k in ["DropletData", "ColorCamData", "LLUVData"] {
                        names.push(format!("PSI-69_Raw Data_{num}_{k}.txt"));
                    }
                }
                out.extend(names.into_iter().map(|n| ("PSI-69".to_string(), n)));
            }
            "acme_normal" | "acme_inverse" => {
                let tid = acme_test_id(&it.run_id);
                for k in ["TP", "Comments", "ImagesComments", "ImageComments"] {
                    out.push((
                        "PSI-159".to_string(),
                        format!("PSI-159_Raw Data_{tid}_{k}.txt"),
                    ));
                }
            }
            _ => {}
        }
    }
    out
}

fn acme_test_id(run_id: &str) -> String {
    run_id
        .rsplit(':')
        .next()
        .unwrap_or("")
        .split('#')
        .next()
        .unwrap_or("")
        .to_string()
}

fn text_lines(d: &mut Dossier, raw: &[u8], sha: &str, name: &str, cap: usize) {
    let text = normalize_newlines(&decode_utf8_or_cp1252(raw));
    let mut n = 0;
    for (i, l) in text.split('\n').enumerate() {
        let t = strip(l);
        if t.is_empty() {
            continue;
        }
        if n == cap {
            d.note(format!(
                "{name}: only the first {cap} non-empty lines are shown"
            ));
            break;
        }
        d.add(
            t.to_string(),
            Some(loc(sha, name, Some(i as i64 + 1), None)),
        );
        n += 1;
    }
}

fn row_lines(d: &mut Dossier, row: &Row, sha: &str, name: &str) {
    for (col, val) in &row.cells {
        let v = strip(val);
        let shown = if v.is_empty() { "(empty)" } else { v };
        d.add(
            format!("{} = {shown}", strip(col)),
            Some(loc(sha, name, Some(row.row), Some(col))),
        );
    }
}

/// Single line "Header=value; ..." for one table row (BASS rows, many at once).
fn compact_row(row: &Row) -> String {
    row.cells
        .iter()
        .filter(|(_, v)| !strip(v).is_empty())
        .map(|(k, v)| format!("{}={}", strip(k), strip(v)))
        .collect::<Vec<_>>()
        .join("; ")
}

fn add_file(
    d: &mut Dossier,
    assets: &Assets,
    inv: &str,
    name: &str,
    title: &str,
    cap: usize,
) -> anyhow::Result<bool> {
    let Some((raw, sha)) = assets.read(inv, name)? else {
        return Ok(false);
    };
    d.section(title.to_string());
    text_lines(d, &raw, &sha, name, cap);
    Ok(true)
}

fn fmt(v: f64) -> String {
    format!("{v:.4}")
}

#[allow(clippy::too_many_arguments)]
fn series_digest(
    d: &mut Dossier,
    assets: &Assets,
    run_id: &str,
    name: &str,
    kind: &str,
    tcol: &str,
    vcol: &str,
    q: Quantity,
    tail: usize,
) -> anyhow::Result<Option<(f64, f64)>> {
    let Some((raw, sha)) = assets.read("PSI-69", name)? else {
        d.section(format!("{kind} channel"));
        match assets.listed("PSI-69", name) {
            Some((row, f)) => d.add(
                format!(
                    "{kind}: listed at NASA ({} bytes) but not available locally for this dossier",
                    f.size_bytes.unwrap_or(0)
                ),
                Some(assets.listing_locator("PSI-69", row, name)),
            ),
            None => d.add(
                format!("{kind}: no such file in the NASA listing for this test"),
                None,
            ),
        }
        return Ok(None);
    };
    let (header, rows) = flex::parse_series(&raw);
    if !header.iter().any(|h| h == tcol) || !header.iter().any(|h| h == vcol) {
        d.section(format!("{kind} channel"));
        d.add(
            format!("{kind}: unexpected header {header:?}"),
            Some(loc(&sha, name, Some(1), None)),
        );
        return Ok(None);
    }
    let obs = flex::series_observations(
        run_id, &sha, name, &header, &rows, tcol, vcol, q, "mm", kind,
    );
    d.section(format!(
        "{kind} digest ({vcol}, time {tcol}, values rounded to 4 decimals)"
    ));
    if obs.is_empty() {
        d.add(
            format!("{kind}: file present but has no timed rows"),
            Some(loc(&sha, name, None, None)),
        );
        return Ok(None);
    }
    let (mut tmin, mut tmax) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut vmin, mut vmax) = (f64::INFINITY, f64::NEG_INFINITY);
    let mut flags: BTreeMap<&str, usize> = BTreeMap::new();
    let mut gaps = vec![];
    let mut prev: Option<&ignis_core::contracts::Observation> = None;
    for o in &obs {
        tmin = tmin.min(o.time_s);
        tmax = tmax.max(o.time_s);
        if let Some(v) = o.value_si {
            vmin = vmin.min(v * 1e3);
            vmax = vmax.max(v * 1e3);
        }
        for f in &o.quality_flags {
            *flags.entry(f.as_str()).or_default() += 1;
        }
        if let Some(p) = prev {
            if o.quality_flags.iter().any(|f| f == "gap_gt_1s") {
                gaps.push(format!(
                    "gap of {:.2} s between t={} (line {}) and t={} (line {})",
                    o.time_s - p.time_s,
                    fmt(p.time_s),
                    p.locator.row.unwrap_or(0),
                    fmt(o.time_s),
                    o.locator.row.unwrap_or(0)
                ));
            }
        }
        prev = Some(o);
    }
    let flag_text = if flags.is_empty() {
        "none".to_string()
    } else {
        flags
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let range = if vmin.is_finite() {
        format!("{}..{} mm", fmt(vmin), fmt(vmax))
    } else {
        "no finite value".to_string()
    };
    d.add(
        format!(
            "{kind}: {} timed rows; time {}..{} s; value {range}; flags: {flag_text}",
            obs.len(),
            fmt(tmin),
            fmt(tmax)
        ),
        Some(loc(&sha, name, None, None)),
    );
    for g in gaps.iter().take(8) {
        d.add(format!("{kind}: {g}"), Some(loc(&sha, name, None, None)));
    }
    let n = obs.len();
    let head = 5usize.min(n);
    let tail_start = n.saturating_sub(tail).max(head);
    let step = ((tail_start.saturating_sub(head)) / 18).max(1);
    let mut chosen: BTreeSet<usize> = (0..head).chain(tail_start..n).collect();
    chosen.extend((head..tail_start).step_by(step));
    for i in chosen {
        let o = &obs[i];
        let value = o
            .value_si
            .map(|v| format!("{} mm", fmt(v * 1e3)))
            .unwrap_or_else(|| "(empty)".to_string());
        let fl = if o.quality_flags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", o.quality_flags.join(","))
        };
        d.add(
            format!(
                "{kind} file line {}: t={} s, {vcol}={value}{fl}",
                o.locator.row.unwrap_or(0),
                fmt(o.time_s)
            ),
            Some(o.locator.clone()),
        );
    }
    Ok(Some((tmin, tmax)))
}

fn flex_endpoint_notes(d: &mut Dossier) {
    for n in [
        "cause: hot_phase_end = the visible flame ended (source labels extinction or completion; it does not claim every reaction stopped); disruption = the droplet broke up, shattered or micro-exploded so the single-droplet model no longer holds (choose it when the disruption ended the burn, also when coincident with extinction); unknown = the sources do not establish the cause.",
        "A droplet leaving the camera field of view, a recording stopping or a saturated image is an observation loss, not an endpoint: if the flame was still active when observation stopped, use censoring=right and cause=unknown.",
        "The DropletData diameter can keep being measured after the visible flame ended: the last diameter sample is not automatically the flame end.",
        "censoring: event = an exact time of the terminal event is established (last_confirmed_active_s and first_terminal_s equal); interval = only bounds are known (last_confirmed_active_s < first_terminal_s); right = still active when observation ended (last_confirmed_active_s set, first_terminal_s null, cause unknown); unknown = no usable timing.",
        "All times are seconds in the DropletData time base (column DropletTime_sec). Its zero is NASA's processed origin, not necessarily ignition. ColorCam and LLUV series have their own time bases and the table 'Burn time' has an unadjudicated origin: use them for timing only if the dossier supports the alignment, and say so in time_origin_note; otherwise use censoring=unknown.",
        "event_before_support_exit: yes only if the sources establish that the terminal event happened before the droplet left the valid measurement support (valid diameter series, field of view); no if the support was lost first; unknown when the order is not established. Do not infer it from convenience.",
    ] {
        d.note(n);
    }
}

fn build_flex(ctx: &Ctx, it: &ReviewItem, d: &mut Dossier) -> anyhow::Result<()> {
    let num = ctx
        .alias(&it.run_id, "file_flex_number")
        .with_context(|| format!("{}: no FLEX file number", it.run_id))?;
    let cir = ctx.alias(&it.run_id, "cir_test_point").unwrap_or_default();
    if it.field == "flex_endpoint" {
        flex_endpoint_notes(d);
    } else {
        d.note("free_floating = a single droplet not supported by a fiber for the whole observed burn; fiber_supported = supported by a fiber; mixed_or_unclear = changes during the burn, several droplets, or the sources do not decide.");
    }
    d.section("Cited source text");
    d.add(it.cited_text.clone(), Some(it.locator.clone()));
    if let Some(row_no) = ctx.endpoints.get(&it.run_id).and_then(|e| {
        e.evidence
            .iter()
            .find(|l| l.path.as_deref() == Some(flex::SUMMARY))
            .and_then(|l| l.row)
    }) {
        if let Some((raw, sha)) = ctx.assets.read("PSI-69", flex::SUMMARY)? {
            if let Some(row) = flex::parse_summary(&raw)
                .into_iter()
                .find(|r| r.row == row_no)
            {
                d.section("Summary table row (PSI-69_Experimental table_FLEX.csv)");
                row_lines(d, &row, &sha, flex::SUMMARY);
            }
        }
    }
    add_file(
        d,
        &ctx.assets,
        "PSI-69",
        &format!("PSI-69_Raw Data_{num}_TestSheet.txt"),
        "Test sheet",
        200,
    )?;
    if !cir.is_empty() {
        add_file(
            d,
            &ctx.assets,
            "PSI-69",
            &format!("PSI-69_Raw Data_{cir}_Comments.txt"),
            "Operator comments for this CIR test point",
            80,
        )?;
        if it.field == "free_floating" {
            add_file(
                d,
                &ctx.assets,
                "PSI-69",
                &format!("PSI-69_Raw Data_{cir}_MDCA_Testpoint_Settings.txt"),
                "MDCA test point settings",
                40,
            )?;
        }
    }
    if it.field != "flex_endpoint" {
        return Ok(());
    }
    if !cir.is_empty() {
        add_file(
            d,
            &ctx.assets,
            "PSI-69",
            &format!("PSI-69_Raw Data_{cir}.txt"),
            "Instrument log for this CIR test point (timeline)",
            120,
        )?;
    }
    let mut range: Option<(f64, f64)> = None;
    for (kind, tcol, vcol, q, tail) in [
        (
            "DropletData",
            "DropletTime_sec",
            "DropletDiameter_mm",
            Quantity::LiquidDiameter,
            15,
        ),
        (
            "ColorCamData",
            "ColorCamTime_sec",
            "ColorCamFlameDiameter_mm",
            Quantity::VisibleFlameDiameter,
            10,
        ),
        (
            "LLUVData",
            "LLUVTime_sec",
            "LLUVFlameDiameter_mm",
            Quantity::UvFlameDiameter,
            10,
        ),
    ] {
        let name = format!("PSI-69_Raw Data_{num}_{kind}.txt");
        if let Some((a, b)) =
            series_digest(d, &ctx.assets, &it.run_id, &name, kind, tcol, vcol, q, tail)?
        {
            range = Some(range.map_or((a, b), |(x, y)| (x.min(a), y.max(b))));
        }
    }
    d.observed_range = range;
    Ok(())
}

fn acme_notes(d: &mut Dossier, field: &str) {
    if field == "endpoint_cause" {
        d.note("extinction = the established flame ended on its own while fuel still flowed; operator_termination = a commanded or automatic flow stop ended the test (including the burner temperature limit); both_possible = the sources cannot separate the two but exclude anything else; unclear = no determination.");
        d.note("The _TP.txt script timeline is wall-clock with 1 s resolution; '***' means the step did not run. 'Extinction detected' and 'MFCs off' are logged script steps. The table 'burn time' origin is not adjudicated.");
    } else {
        d.note("no_effect = the comment does not change the population, control history or endpoint (bottle notes, timer notes, ordinary descriptions); intervention = an operator or automatic action changed flow or ended the test (flow cut, temperature limit stop, time-out); data_loss = recorded data or images are missing or compromised; non_spherical_or_irregular = the flame was not spherical or regular; other = the comment changes membership, control or endpoint in a way not listed (e.g. not an established hot flame).");
    }
}

fn build_acme(ctx: &Ctx, it: &ReviewItem, d: &mut Dossier) -> anyhow::Result<()> {
    acme_notes(d, &it.field);
    d.section("Cited source text");
    d.add(it.cited_text.clone(), Some(it.locator.clone()));
    let path = it.locator.path.clone().unwrap_or_default();
    if let (Some((raw, sha)), Some(row_no)) = (ctx.assets.read("PSI-159", &path)?, it.locator.row) {
        if let Some(row) = table_rows(&decode_utf8_sig_lossy(&raw), false)
            .into_iter()
            .find(|r| r.row == row_no)
        {
            d.section(format!("Experimental table row ({path})"));
            row_lines(d, &row, &sha, &path);
        }
    }
    let tid = acme_test_id(&it.run_id);
    let name = format!("PSI-159_Raw Data_{tid}_TP.txt");
    if !add_file(
        d,
        &ctx.assets,
        "PSI-159",
        &name,
        "Script timeline (_TP.txt)",
        120,
    )? {
        d.section("Script timeline (_TP.txt)");
        d.add(
            "no _TP.txt timeline is available for this test",
            ctx.assets
                .listed("PSI-159", &name)
                .map(|(row, _)| ctx.assets.listing_locator("PSI-159", row, &name)),
        );
    }
    for (k, title) in [
        ("Comments", "Operator comments"),
        ("ImagesComments", "Image comments"),
        ("ImageComments", "Image comments"),
    ] {
        add_file(
            d,
            &ctx.assets,
            "PSI-159",
            &format!("PSI-159_Raw Data_{tid}_{k}.txt"),
            title,
            40,
        )?;
    }
    Ok(())
}

fn bass_notes(d: &mut Dossier) {
    d.note("Candidates: a run id means the video records that run; not_a_test_recording = the file is not a recording of a test burn; unclear = file name, listing and tables do not decide. No video frames are given, so a decision needs agreement of the name with the tables.");
    d.note("BASS-II (PSI-25): the on-board video display date is one day later than the actual date (leap-year bug, documented in the investigation ReadMe), so a file GMT day equal to table day + 1 is expected.");
    d.note("Sample numbers can be written with or without the M suffix and specimens can burn more than once.");
}

fn sample_key(s: &str) -> String {
    let t: String = s
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    t.trim_start_matches('0').to_string()
}

fn caps(cell: &'static OnceLock<Regex>, pat: &str, s: &str) -> Option<String> {
    cell.get_or_init(|| Regex::new(pat).unwrap())
        .captures(s)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn neighbours(ctx: &Ctx, d: &mut Dossier, inv: &str, name: &str, keep: impl Fn(&str) -> bool) {
    let files = ctx
        .assets
        .listings
        .get(inv)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    d.section(format!(
        "{inv} listing entries of this file and related files"
    ));
    let mut shown = 0;
    for (i, f) in files.iter().enumerate() {
        if f.file_name == name || (keep(&f.file_name) && shown < 15) {
            shown += usize::from(f.file_name != name);
            d.add(
                format!(
                    "listing: {} ({} bytes, version {})",
                    f.file_name,
                    f.size_bytes.unwrap_or(0),
                    f.file_version.unwrap_or(0)
                ),
                Some(ctx.assets.listing_locator(inv, i as i64 + 1, &f.file_name)),
            );
        }
    }
}

fn build_bass25(ctx: &Ctx, it: &ReviewItem, d: &mut Dossier) -> anyhow::Result<()> {
    static G: OnceLock<Regex> = OnceLock::new();
    static S: OnceLock<Regex> = OnceLock::new();
    static T: OnceLock<Regex> = OnceLock::new();
    let name = &it.cited_text;
    let doy: Option<i64> = caps(&G, r"(?i)GMT(\d+)", name).and_then(|x| x.parse().ok());
    let sample = caps(&S, r"(?i)SAMPLE_?([0-9A-Za-z]+)", name).unwrap_or_default();
    let test = caps(&T, r"(?i)TEST[_-]?([A-Za-z]+\d+[a-z]?)", name)
        .unwrap_or_default()
        .to_uppercase();
    neighbours(ctx, d, "PSI-25", name, |n| {
        (!sample.is_empty() && n.contains(&format!("SAMPLE_{sample}")))
            || (!test.is_empty() && n.to_uppercase().contains(&format!("TEST_{test}")))
    });
    let Some((raw, sha)) = ctx.assets.read("PSI-25", bass::T25)? else {
        return Ok(());
    };
    let rows = table_rows(&decode_utf8_sig_lossy(&raw), true);
    let key = sample_key(&sample);
    let score = |r: &Row| -> usize {
        let t = strip(r.get("Test #")).to_uppercase();
        let s = sample_key(strip(r.get("Sample #")));
        let day: Option<i64> = crate::common::parse_number(r.get("GMT")).map(|x| x.trunc() as i64);
        usize::from(!test.is_empty() && t == test) * 4
            + usize::from(!key.is_empty() && s == key) * 2
            + usize::from(matches!((day, doy), (Some(a), Some(b)) if a == b || a + 1 == b))
    };
    let mut hits: Vec<&Row> = rows.iter().filter(|r| score(r) > 0).collect();
    hits.sort_by_key(|r| std::cmp::Reverse(score(r)));
    d.section(format!(
        "Experimental table rows matching test {test:?}, sample {sample:?} or GMT day {doy:?} ({})",
        bass::T25
    ));
    for r in hits.into_iter().take(30) {
        d.add(
            format!("table row {}: {}", r.row, compact_row(r)),
            Some(loc(&sha, bass::T25, Some(r.row), None)),
        );
        d.candidates.insert(
            d.candidates.len().saturating_sub(2),
            format!("PSI-25:{}", strip(r.get("Test #"))),
        );
    }
    Ok(())
}

fn build_bass26(ctx: &Ctx, it: &ReviewItem, d: &mut Dossier) -> anyhow::Result<()> {
    static TS: OnceLock<Regex> = OnceLock::new();
    let name = &it.cited_text;
    let ts = TS
        .get_or_init(|| Regex::new(r"(\d{8}) (\d{2})\.(\d{2})\.(\d{2})").unwrap())
        .captures(name);
    let ymd = ts.as_ref().map(|c| c[1].to_string()).unwrap_or_default();
    neighbours(ctx, d, "PSI-26", name, |n| {
        !ymd.is_empty() && n.contains(&ymd)
    });
    let Some(c) = ts else {
        d.note("This file name carries no timestamp; the as-run matrix cannot be matched by time.");
        return Ok(());
    };
    let Ok(date) = chrono::NaiveDate::parse_from_str(&ymd, "%Y%m%d") else {
        return Ok(());
    };
    let minutes = |h: u32, m: u32| h as i64 * 60 + m as i64;
    let vt = minutes(c[2].parse().unwrap_or(0), c[3].parse().unwrap_or(0));
    let Some(src) = mf::by_name("PSI-26")?.remove(bass::ASRUN26) else {
        return Ok(());
    };
    let sha = mf::sha_of(&src);
    let asrun = bass::parse_asrun26(&mf::local_path(&src))?;
    let table = match ctx.assets.read("PSI-26", bass::T26)? {
        Some((raw, s)) => Some((table_rows(&decode_utf8_sig_lossy(&raw), true), s)),
        None => None,
    };
    let mut same: Vec<(i64, &bass::AsRun)> = asrun
        .iter()
        .filter(|a| a.video_date == Some(date))
        .map(|a| {
            (
                a.start
                    .map(|s| vt - minutes(chrono::Timelike::hour(&s), chrono::Timelike::minute(&s)))
                    .unwrap_or(i64::MAX),
                a,
            )
        })
        .collect();
    same.sort_by_key(|(dt, _)| dt.abs());
    d.section("As-run matrix rows with the same date on videos");
    for (dt, a) in same.iter().take(30) {
        let st = a
            .start
            .map(|s| s.format("%H:%M").to_string())
            .unwrap_or_else(|| "(missing)".into());
        let delta = if *dt == i64::MAX {
            "unknown".to_string()
        } else {
            format!("{dt:+} min")
        };
        d.add(
            format!("as-run sheet {} row {}: as_run={}; test={}; date_on_videos={date}; approx_start={st}; video time minus approx start = {delta}", a.sheet, a.row, a.asrun, a.test),
            Some(loc(&sha, bass::ASRUN26, Some(a.row), Some(&a.sheet))),
        );
        if let Some((rows, ts)) = &table {
            if let Some(r) = rows.iter().find(|r| {
                strip(r.get("As Run Test #")) == a.asrun && strip(r.get("Test #")) == a.test
            }) {
                d.add(
                    format!("master table row {}: {}", r.row, compact_row(r)),
                    Some(loc(ts, bass::T26, Some(r.row), None)),
                );
                d.candidates.insert(
                    d.candidates.len().saturating_sub(2),
                    format!("PSI-26:asrun{}:test{}", a.asrun, a.test),
                );
            }
        }
    }
    Ok(())
}

pub fn build(ctx: &Ctx, it: &ReviewItem) -> anyhow::Result<Dossier> {
    let mut d = Dossier::new(it);
    match it.family.as_str() {
        "flex" => build_flex(ctx, it, &mut d)?,
        "acme_normal" | "acme_inverse" => build_acme(ctx, it, &mut d)?,
        _ => {
            bass_notes(&mut d);
            d.section("Cited source text");
            d.add(it.cited_text.clone(), Some(it.locator.clone()));
            if it.cited_text.starts_with("PSI-25") {
                build_bass25(ctx, it, &mut d)?;
            } else {
                build_bass26(ctx, it, &mut d)?;
            }
        }
    }
    if let Some(screen) = &it.deterministic_screen {
        d.note(format!(
            "Why this item was routed here (a screen, not an answer): {screen}"
        ));
    }
    Ok(d)
}

/// File-name-safe id for dumped dossiers.
pub fn slug(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn dump(dir: &Path, dossiers: &[Dossier]) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    for d in dossiers {
        std::fs::write(dir.join(format!("{}.txt", slug(&d.item_id))), d.render())?;
    }
    Ok(())
}

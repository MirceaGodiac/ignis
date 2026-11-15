//! Sensor and settings text of the investigations beyond the modelled four → [`DatasetBundle`]
//! Parquet (workboard R2b).
//!
//! The atlas stores 24,482 sensor/settings text files of the 19 flight investigations (LANES_V2 §10).
//! FLEX (PSI-69), ACME CFI-G (PSI-159) and BASS (PSI-25/26) have family bundles built from curated
//! records (`dataset.rs`); this module converts the raw sensor text of the other investigations,
//! one bundle per investigation, without curation and without interpreting any value.
//!
//! **Formats read** (decided from the content, never from the file name):
//! * `mdca`: FCF/MDCA telemetry exports of FLEX-2 (PSI-68) and CFI (PSI-39), e.g.
//!   `A109D03_Chamber_Readings.txt`, `F068A03_Radiometer_Readings.txt`, `*_Testpoint_Settings.txt`:
//!   a quoted CSV header `"<rack> TimeStamp","NAME(UNIT)",…` then one row per sample with a
//!   `YYYY-mm-dd HH:MM:SS.ffffff` timestamp. Values are numbers, words (`No`, `Off`) or timestamps.
//! * `acme_log`: ACME rack logs of PSI-10/20/21/22/23 (`<test>_data.csv`): a first line with the date,
//!   day of year and settings (`Test Point: …`), a header line `Time,<channel>,…` (units only where
//!   the header writes them, `MFCA1Setpoint(V)`), then rows `HH:MM:SS:mmm,…`.
//!
//! Anything else is skipped and reported in the quality report with its reason.
//!
//! **What a value keeps** (view `sensor_samples`, one row per sample × channel): the exact text of
//! the cell (`raw_value`), its number in the raw unit (`raw_number`), the raw unit as written
//! (`raw_unit`), the SI value and unit when the unit is known (`value_si`, `unit_si`), a `status`,
//! and a locator `sha256:<file>#line=<n>&column=<k>` (1-based) of the original cell. Statuses are
//! explicit, never a zero:
//! `ok` · `missing` (empty cell) · `text` (a word or timestamp, kept verbatim) · `unit_not_stated`
//! (the file gives no unit: ACME logs; `raw_number` only) · `unit_not_applicable` (the header says
//! `N/A`) · `unit_not_converted` (a unit this table does not convert; raw kept) · `conflicting` (two
//! rows of one file carry the same timestamp with different values for a channel; both are kept).
//!
//! **Time.** `time_s` is seconds since the first sample of the same file (`time_origin =
//! first_sample_of_file`): the only origin every file has. The absolute time of each sample is
//! `time_utc`. The view `sensor_files` gives, per file, the first sample's time and the ignition time
//! when the test's own sheet states it (ACME `<test>_TP.txt` "Ignition and retraction"; FLEX-2/CFI
//! `<test>.txt` "AS Burn to Completion/Igniter On"), as `ignition_s` since the first sample; absent
//! when no sheet states it. Times in the files are mission clock (GMT) as written; no zone shift.
//!
//! **Partition.** These investigations have no frozen split and hold no held-out outcome (the four
//! lockbox investigations are refused here), so a bundle declares the single partition `train` with
//! `split_id = unsplit`: every row is builder-visible. It is not a modelling split. Once a frozen
//! split assigns any run of the investigation, the split wins: `DatasetStore` withholds the unsplit
//! bundle (`dataset::unsplit_superseded`) until it is rebuilt on the split.
//!
//! **Cost.** One file at a time (a few MB), one Parquet row group per file, written to disk as it
//! goes and hashed after; an investigation is never held in memory.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context};
use arrow_array::builder::{Float64Builder, StringBuilder};
use arrow_array::{ArrayRef, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use ignis_core::contracts::{
    ArtifactId, ArtifactRef, BundleAudience, ColumnRole, ColumnSpec, DatasetBundle, DatasetId,
    DatasetView, Digest, Family, Origin, Partition, Producer, Timestamp,
};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;
use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};

use crate::dataset::{DatasetStore, PARQUET};

/// Layout version of the sensor bundles (part of their inclusion criteria).
pub const SENSOR_VERSION: &str = "1";
pub const TIME_ORIGIN: &str = "first_sample_of_file";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Mdca,
    AcmeLog,
}

impl Format {
    pub fn as_str(self) -> &'static str {
        match self {
            Format::Mdca => "mdca",
            Format::AcmeLog => "acme_log",
        }
    }
}

/// The format of a file's text, from its first two lines.
pub fn detect(text: &str) -> Option<Format> {
    let mut lines = text.lines();
    let first = lines.next()?.trim_start_matches('\u{feff}');
    let second = lines.next().unwrap_or_default();
    if first.starts_with('"') && first.contains("TimeStamp\"") {
        return Some(Format::Mdca);
    }
    if first.contains("Test Point:") && second.starts_with("Time,") {
        return Some(Format::AcmeLog);
    }
    None
}

/// `(unit_si, factor, offset)`: SI = raw × factor + offset. `None`: not converted here.
pub fn si_unit(raw: &str) -> Option<(&'static str, f64, f64)> {
    let u = raw.trim();
    Some(match u.to_ascii_lowercase().as_str() {
        "psia" | "psig" | "psid" | "psi" => ("Pa", 6894.757293168, 0.0),
        "kpa" => ("Pa", 1000.0, 0.0),
        "pa" => ("Pa", 1.0, 0.0),
        "torr" => ("Pa", 133.322368421, 0.0),
        "deg c" | "degc" | "°c" | "c" => ("K", 1.0, 273.15),
        "deg f" | "degf" | "°f" | "f" => ("K", 5.0 / 9.0, 273.15 - 32.0 * 5.0 / 9.0),
        "deg k" | "k" => ("K", 1.0, 0.0),
        "vdc" | "v" | "vac" | "volts" => ("V", 1.0, 0.0),
        "mv" => ("V", 1e-3, 0.0),
        "adc" | "a" | "amps" => ("A", 1.0, 0.0),
        "ma" => ("A", 1e-3, 0.0),
        "slpm" => ("Sm3/s", 1.0 / 60_000.0, 0.0),
        "sccm" => ("Sm3/s", 1.0 / 60_000_000.0, 0.0),
        "%" => ("1", 0.01, 0.0),
        "s" | "sec" | "seconds" => ("s", 1.0, 0.0),
        "ms" | "msec" => ("s", 1e-3, 0.0),
        "hz" => ("Hz", 1.0, 0.0),
        "w" | "watts" => ("W", 1.0, 0.0),
        _ => return None,
    })
}

/// The test point of a PSI file name: `PSI-10_Raw Data_19115A1_data.csv` → `19115A1`,
/// `A109D03_Chamber_Readings.txt` → `A109D03`.
pub fn test_point(file_name: &str) -> String {
    let base = match file_name.split_once("_Raw Data_") {
        Some((_, rest)) => rest,
        None => file_name,
    };
    let stem = base.rsplit_once('.').map_or(base, |(s, _)| s);
    stem.split('_').next().unwrap_or(stem).to_string()
}

/// The stream of a file within its test: `…_19115A1_data.csv` → `data`,
/// `A109D03_Chamber_Readings.txt` → `Chamber_Readings`.
pub fn stream_of(file_name: &str, test: &str) -> String {
    let base = file_name
        .split_once("_Raw Data_")
        .map_or(file_name, |(_, r)| r);
    let stem = base.rsplit_once('.').map_or(base, |(s, _)| s);
    stem.strip_prefix(test)
        .map(|s| s.trim_start_matches('_'))
        .filter(|s| !s.is_empty())
        .unwrap_or(stem)
        .to_string()
}

fn csv_fields(line: &str) -> Vec<String> {
    // The exports quote only header names and the row timestamp; no field holds a comma.
    line.split(',')
        .map(|f| f.trim().trim_matches('"').to_string())
        .collect()
}

/// `NAME(UNIT)` → (`NAME`, `UNIT`).
fn name_unit(h: &str) -> (String, Option<String>) {
    match (h.rfind('('), h.ends_with(')')) {
        (Some(i), true) if i > 0 => (
            h[..i].trim().to_string(),
            Some(h[i + 1..h.len() - 1].to_string()),
        ),
        _ => (h.to_string(), None),
    }
}

fn parse_mdca_ts(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f").ok()
}

fn fmt_ts(t: NaiveDateTime) -> String {
    t.format("%Y-%m-%dT%H:%M:%S%.6f").to_string()
}

#[derive(Default)]
struct Cols {
    run_id: StringBuilder,
    stream: StringBuilder,
    channel: StringBuilder,
    time_s: Float64Builder,
    time_utc: StringBuilder,
    raw_value: StringBuilder,
    raw_number: Float64Builder,
    raw_unit: StringBuilder,
    value_si: Float64Builder,
    unit_si: StringBuilder,
    status: StringBuilder,
    locator: StringBuilder,
    rows: usize,
}

/// Columns of `sensor_samples` (name, role, unit, nullable, type).
fn sample_specs() -> Vec<(ColumnSpec, DataType)> {
    let c = |name: &str, role, unit: Option<&str>, nullable, t| {
        (
            ColumnSpec {
                name: name.into(),
                role,
                quantity: None,
                unit: unit.map(String::from),
                nullable,
            },
            t,
        )
    };
    use ColumnRole::*;
    use DataType::{Float64 as F, Utf8 as S};
    vec![
        c("run_id", RunId, None, false, S),
        c("stream", Condition, None, false, S),
        c("channel", Condition, None, false, S),
        c("time_s", Time, Some("s"), false, F),
        c("time_utc", Flag, None, false, S),
        c("raw_value", Value, None, false, S),
        c("raw_number", Value, None, true, F),
        c("raw_unit", Flag, None, true, S),
        c("value_si", Value, None, true, F),
        c("unit_si", Flag, None, true, S),
        c("status", Flag, None, false, S),
        c("locator", Locator, None, false, S),
    ]
}

fn file_specs() -> Vec<(ColumnSpec, DataType)> {
    let c = |name: &str, role, unit: Option<&str>, nullable, t| {
        (
            ColumnSpec {
                name: name.into(),
                role,
                quantity: None,
                unit: unit.map(String::from),
                nullable,
            },
            t,
        )
    };
    use ColumnRole::*;
    use DataType::{Float64 as F, Utf8 as S};
    vec![
        c("run_id", RunId, None, false, S),
        c("stream", Condition, None, false, S),
        c("format", Flag, None, false, S),
        c("file_name", Locator, None, false, S),
        c("source_id", Locator, None, false, S),
        c("first_sample_utc", Flag, None, false, S),
        c("last_sample_utc", Flag, None, false, S),
        c("ignition_utc", Flag, None, true, S),
        c("ignition_s", Time, Some("s"), true, F),
        c("ignition_locator", Locator, None, true, S),
        c("samples", Value, None, false, F),
        c("channels", Value, None, false, F),
    ]
}

fn schema(specs: &[(ColumnSpec, DataType)]) -> Arc<Schema> {
    Arc::new(Schema::new(
        specs
            .iter()
            .map(|(s, t)| Field::new(&s.name, t.clone(), s.nullable))
            .collect::<Vec<_>>(),
    ))
}

impl Cols {
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        run: &str,
        stream: &str,
        channel: &str,
        t: f64,
        utc: &str,
        raw: &str,
        unit: Option<&str>,
        status_override: Option<&'static str>,
        loc: String,
        counts: &mut BTreeMap<&'static str, u64>,
    ) {
        let trimmed = raw.trim();
        let number = trimmed.parse::<f64>().ok().filter(|x| x.is_finite());
        let (status, si): (&'static str, Option<(f64, &'static str)>) = if trimmed.is_empty() {
            ("missing", None)
        } else if number.is_none() {
            ("text", None)
        } else if let Some(s) = status_override {
            (s, None)
        } else {
            match unit {
                None => ("unit_not_stated", None),
                Some(u) if u.trim().eq_ignore_ascii_case("n/a") => ("unit_not_applicable", None),
                Some(u) => match si_unit(u) {
                    Some((usi, f, o)) => ("ok", number.map(|x| (x * f + o, usi))),
                    None => ("unit_not_converted", None),
                },
            }
        };
        *counts.entry(status).or_default() += 1;
        self.run_id.append_value(run);
        self.stream.append_value(stream);
        self.channel.append_value(channel);
        self.time_s.append_value(t);
        self.time_utc.append_value(utc);
        self.raw_value.append_value(raw);
        self.raw_number.append_option(number);
        self.raw_unit.append_option(unit);
        self.value_si.append_option(si.map(|(v, _)| v));
        self.unit_si.append_option(si.map(|(_, u)| u));
        self.status.append_value(status);
        self.locator.append_value(loc);
        self.rows += 1;
    }

    fn finish(&mut self) -> anyhow::Result<RecordBatch> {
        let arrays: Vec<ArrayRef> = vec![
            Arc::new(self.run_id.finish()),
            Arc::new(self.stream.finish()),
            Arc::new(self.channel.finish()),
            Arc::new(self.time_s.finish()),
            Arc::new(self.time_utc.finish()),
            Arc::new(self.raw_value.finish()),
            Arc::new(self.raw_number.finish()),
            Arc::new(self.raw_unit.finish()),
            Arc::new(self.value_si.finish()),
            Arc::new(self.unit_si.finish()),
            Arc::new(self.status.finish()),
            Arc::new(self.locator.finish()),
        ];
        self.rows = 0;
        Ok(RecordBatch::try_new(schema(&sample_specs()), arrays)?)
    }
}

/// One parsed file's summary (a row of `sensor_files`).
#[derive(Debug, Clone)]
pub struct FileSummary {
    pub run_id: String,
    pub stream: String,
    pub format: Format,
    pub file_name: String,
    pub source_id: String,
    pub first: NaiveDateTime,
    pub last: NaiveDateTime,
    pub ignition: Option<(NaiveDateTime, String)>,
    pub samples: u64,
    pub channels: u64,
    pub statuses: BTreeMap<&'static str, u64>,
}

fn sha256_id(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

/// Ignition stated by the test's own sheet next to the file, with its locator.
fn ignition(dir: &Path, inv: &str, test: &str) -> Option<(NaiveDateTime, String)> {
    let candidates = [
        (
            format!("{inv}_Raw Data_{test}_TP.txt"),
            "Ignition and retraction:",
        ),
        (format!("{test}_TP.txt"), "Ignition and retraction:"),
        (format!("{test}.txt"), "AS Burn to Completion/Igniter On:"),
        (
            format!("{inv}_Raw Data_{test}.txt"),
            "AS Burn to Completion/Igniter On:",
        ),
    ];
    for (name, key) in candidates {
        let Ok(bytes) = std::fs::read(dir.join(&name)) else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        for (i, line) in text.lines().enumerate() {
            if let Some(v) = line.trim().strip_prefix(key) {
                let v = v.trim();
                let t = NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S%.f")
                    .or_else(|_| NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S"));
                if let Ok(t) = t {
                    return Some((t, format!("{}#line={}", sha256_id(&bytes), i + 1)));
                }
                // The key is there but states no time (`***`): no ignition, by the sheet's own word.
                return None;
            }
        }
    }
    None
}

/// Parses one file into `cols`; `None` when its format is not one this module reads.
pub fn parse_file(
    path: &Path,
    inv: &str,
    cols: &mut SampleWriter,
) -> anyhow::Result<Option<FileSummary>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes);
    let Some(format) = detect(&text) else {
        return Ok(None);
    };
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let source_id = sha256_id(&bytes);
    let test = test_point(&file_name);
    let stream = stream_of(&file_name, &test);
    let run_id = format!("{inv}:{test}");
    let mut statuses = BTreeMap::new();
    let mut lines = text.lines().enumerate();
    let (channels, units, start_line, date): (
        Vec<String>,
        Vec<Option<String>>,
        usize,
        Option<NaiveDate>,
    ) = match format {
        Format::Mdca => {
            let (_, h) = lines.next().context("empty file")?;
            let hs = csv_fields(h.trim_start_matches('\u{feff}'));
            let (c, u): (Vec<_>, Vec<_>) = hs[1..].iter().map(|x| name_unit(x)).unzip();
            (c, u, 2, None)
        }
        Format::AcmeLog => {
            let (_, meta) = lines.next().context("empty file")?;
            let date = meta
                .split_whitespace()
                .next()
                .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                .context("ACME log without a date on its first line")?;
            let (_, h) = lines.next().context("no header line")?;
            // Rack logs name channels without units (`TCA`); the setpoint logs give them
            // (`MFCA1Setpoint(V)`).
            let hs = csv_fields(h);
            let (c, u): (Vec<_>, Vec<_>) = hs[1..].iter().map(|x| name_unit(x)).unzip();
            (c, u, 3, Some(date))
        }
    };
    let _ = start_line;
    let mut first: Option<NaiveDateTime> = None;
    let mut last: Option<NaiveDateTime> = None;
    let mut prev: Option<(NaiveDateTime, Vec<String>)> = None;
    let mut day = 0i64;
    let mut samples = 0u64;
    // Rows are buffered per timestamp so a repeated timestamp can mark both rows conflicting.
    let mut pending: Vec<(usize, NaiveDateTime, Vec<String>, BTreeSet<usize>)> = Vec::new();
    let flush = |pending: &mut Vec<(usize, NaiveDateTime, Vec<String>, BTreeSet<usize>)>,
                 cols: &mut SampleWriter,
                 statuses: &mut BTreeMap<&'static str, u64>,
                 first: NaiveDateTime|
     -> anyhow::Result<()> {
        for (line_no, t, fields, conflicts) in pending.drain(..) {
            let ts = (t - first).num_microseconds().unwrap_or(0) as f64 / 1e6;
            let utc = fmt_ts(t);
            for (k, ch) in channels.iter().enumerate() {
                let raw = fields.get(k + 1).map(String::as_str).unwrap_or("");
                let over = if conflicts.contains(&k) {
                    Some("conflicting")
                } else {
                    None
                };
                cols.cols.push(
                    &run_id,
                    &stream,
                    ch,
                    ts,
                    &utc,
                    raw,
                    units[k].as_deref(),
                    over,
                    format!("{source_id}#line={}&column={}", line_no + 1, k + 2),
                    statuses,
                );
            }
        }
        cols.maybe_flush()
    };
    for (i, line) in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields = csv_fields(line);
        let t = match format {
            Format::Mdca => parse_mdca_ts(&fields[0]),
            Format::AcmeLog => {
                let p: Vec<&str> = fields[0].split(':').collect();
                let tm = (p.len() == 4)
                    .then(|| {
                        NaiveTime::from_hms_milli_opt(
                            p[0].parse().ok()?,
                            p[1].parse().ok()?,
                            p[2].parse().ok()?,
                            p[3].parse().ok()?,
                        )
                    })
                    .flatten();
                tm.map(|tm| {
                    let mut t = date.unwrap().and_time(tm) + Duration::days(day);
                    // A clock that runs past midnight: the next day.
                    if let Some(l) = last {
                        if t < l - Duration::hours(12) {
                            day += 1;
                            t += Duration::days(1);
                        }
                    }
                    t
                })
            }
        };
        let Some(t) = t else {
            bail!(
                "{file_name} line {}: unreadable timestamp {:?}",
                i + 1,
                fields[0]
            );
        };
        let first_t = *first.get_or_insert(t);
        let mut conflicts = BTreeSet::new();
        if let Some((pt, pf)) = &prev {
            if *pt == t {
                for k in 0..channels.len() {
                    if pf.get(k + 1) != fields.get(k + 1) {
                        conflicts.insert(k);
                    }
                }
                if let Some(last_row) = pending.last_mut() {
                    last_row.3.extend(conflicts.iter().copied());
                }
            } else {
                flush(&mut pending, cols, &mut statuses, first_t)?;
            }
        }
        last = Some(t);
        samples += 1;
        prev = Some((t, fields.clone()));
        pending.push((i, t, fields, conflicts));
    }
    let Some(first) = first else {
        bail!("{file_name}: no samples");
    };
    flush(&mut pending, cols, &mut statuses, first)?;
    let dir = path.parent().unwrap_or(Path::new("."));
    Ok(Some(FileSummary {
        run_id,
        stream,
        format,
        file_name,
        source_id,
        first,
        last: last.unwrap_or(first),
        ignition: ignition(dir, inv, &test),
        samples,
        channels: channels.len() as u64,
        statuses,
    }))
}

/// Streams `sensor_samples` into a Parquet file, one row group per ~1 M rows.
pub struct SampleWriter {
    cols: Cols,
    writer: ArrowWriter<std::fs::File>,
}

const ROW_GROUP: usize = 1 << 20;

fn props(meta: &Value) -> WriterProperties {
    WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .set_key_value_metadata(Some(vec![KeyValue::new(
            "ignis".to_string(),
            meta.to_string(),
        )]))
        .build()
}

impl SampleWriter {
    fn create(path: &Path, meta: &Value) -> anyhow::Result<SampleWriter> {
        let f = std::fs::File::create(path)?;
        Ok(SampleWriter {
            cols: Cols::default(),
            writer: ArrowWriter::try_new(f, schema(&sample_specs()), Some(props(meta)))?,
        })
    }

    fn maybe_flush(&mut self) -> anyhow::Result<()> {
        if self.cols.rows >= ROW_GROUP {
            let b = self.cols.finish()?;
            self.writer.write(&b)?;
        }
        Ok(())
    }

    fn close(mut self) -> anyhow::Result<u64> {
        let b = self.cols.finish()?;
        if b.num_rows() > 0 {
            self.writer.write(&b)?;
        }
        let meta = self.writer.close()?;
        Ok(meta.file_metadata().num_rows() as u64)
    }
}

fn files_parquet(rows: &[FileSummary], meta: &Value) -> anyhow::Result<Vec<u8>> {
    let mut s: Vec<StringBuilder> = (0..8).map(|_| StringBuilder::new()).collect();
    let mut ign_s = Float64Builder::new();
    let mut ign_loc = StringBuilder::new();
    let mut samples = Float64Builder::new();
    let mut chans = Float64Builder::new();
    for r in rows {
        s[0].append_value(&r.run_id);
        s[1].append_value(&r.stream);
        s[2].append_value(r.format.as_str());
        s[3].append_value(&r.file_name);
        s[4].append_value(&r.source_id);
        s[5].append_value(fmt_ts(r.first));
        s[6].append_value(fmt_ts(r.last));
        s[7].append_option(r.ignition.as_ref().map(|(t, _)| fmt_ts(*t)));
        ign_s.append_option(
            r.ignition
                .as_ref()
                .map(|(t, _)| (*t - r.first).num_milliseconds() as f64 / 1e3),
        );
        ign_loc.append_option(r.ignition.as_ref().map(|(_, l)| l.as_str()));
        samples.append_value(r.samples as f64);
        chans.append_value(r.channels as f64);
    }
    let mut it = s.into_iter();
    let mut next = || Arc::new(it.next().unwrap().finish()) as ArrayRef;
    let arrays: Vec<ArrayRef> = vec![
        next(),
        next(),
        next(),
        next(),
        next(),
        next(),
        next(),
        next(),
        Arc::new(ign_s.finish()),
        Arc::new(ign_loc.finish()),
        Arc::new(samples.finish()),
        Arc::new(chans.finish()),
    ];
    let schema = schema(&file_specs());
    let batch = RecordBatch::try_new(schema.clone(), arrays)?;
    let mut out = Vec::new();
    let mut w = ArrowWriter::try_new(&mut out, schema, Some(props(meta)))?;
    w.write(&batch)?;
    w.close()?;
    Ok(out)
}

fn artifact_ref(
    sha: Digest,
    bytes: u64,
    media_type: &str,
    contract: Option<&str>,
    store: &DatasetStore,
) -> anyhow::Result<ArtifactRef> {
    if let Some(a) = store.existing(BundleAudience::Builder, &sha) {
        return Ok(a);
    }
    let hex = ignis_core::hashing::hex_part(sha.as_str()).to_string();
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(format!("art-{}", &hex[..32]))?,
        sha256: sha,
        media_type: media_type.into(),
        byte_length: bytes,
        contract: contract.map(String::from),
        produced_by: Producer::default(),
        inputs: Vec::new(),
        origin: Origin::System,
        created_at: Timestamp::now(),
    })
}

fn sha_of_file(p: &Path) -> anyhow::Result<(Digest, u64)> {
    let mut f = std::fs::File::open(p)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut n = 0u64;
    loop {
        let k = f.read(&mut buf)?;
        if k == 0 {
            break;
        }
        h.update(&buf[..k]);
        n += k as u64;
    }
    Ok((
        Digest::parse(&format!("sha256:{}", hex::encode(h.finalize())))?,
        n,
    ))
}

/// The dataset id of an investigation's sensor bundle.
pub fn dataset_id(inv: &str) -> String {
    format!("sensors-{}", inv.to_ascii_lowercase())
}

/// What a build did.
pub struct SensorBuild {
    pub bundle: DatasetBundle,
    pub path: PathBuf,
    pub files: Vec<FileSummary>,
    pub skipped: Vec<(String, String)>,
    pub quality: Value,
}

/// Builds and saves the sensor bundle of `inv` from the files under `raw_dir`.
pub fn build_investigation(
    inv: &str,
    raw_dir: &Path,
    store: &DatasetStore,
) -> anyhow::Result<SensorBuild> {
    if let Some(f) = crate::sources::storage::flight(inv) {
        anyhow::ensure!(
            !f.lockbox,
            "{inv} holds outcomes of held-out tests (lockbox): its data goes through its family bundle, not R2b"
        );
    } else {
        bail!("{inv} is not one of the 19 flight investigations");
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(raw_dir)
        .with_context(|| format!("{}", raw_dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("txt") || x.eq_ignore_ascii_case("csv"))
        })
        .collect();
    paths.sort();
    let meta = json!({"producer": "ignis-data sensors", "version": SENSOR_VERSION, "investigation": inv, "time_origin": TIME_ORIGIN});
    let art_dir = store.builder_root.join("artifacts");
    std::fs::create_dir_all(&art_dir)?;
    let tmp = art_dir.join(format!("sensors-{inv}-{}.parquet.part", std::process::id()));
    let mut w = SampleWriter::create(&tmp, &meta)?;
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        match parse_file(p, inv, &mut w) {
            Ok(Some(s)) => files.push(s),
            Ok(None) => skipped.push((name, "not a sensor table this module reads".into())),
            Err(e) => skipped.push((name, format!("{e:#}"))),
        }
    }
    let rows = w.close()?;
    if files.is_empty() {
        let _ = std::fs::remove_file(&tmp);
        bail!(
            "no sensor table under {} ({} files skipped)",
            raw_dir.display(),
            skipped.len()
        );
    }
    let (sha, len) = sha_of_file(&tmp)?;
    let h = ignis_core::hashing::hex_part(sha.as_str()).to_string();
    let data = art_dir.join(format!("{h}.parquet"));
    if data.exists() {
        std::fs::remove_file(&tmp)?;
    } else {
        std::fs::rename(&tmp, &data)?;
    }
    let samples_ref = artifact_ref(sha, len, PARQUET, None, store)?;
    let files_bytes = files_parquet(&files, &meta)?;
    let files_ref = artifact_ref(
        Digest::of_bytes(&files_bytes),
        files_bytes.len() as u64,
        PARQUET,
        None,
        store,
    )?;

    let mut totals: BTreeMap<&str, u64> = BTreeMap::new();
    for f in &files {
        for (k, v) in &f.statuses {
            *totals.entry(k).or_default() += v;
        }
    }
    let quality = json!({
        "investigation": inv,
        "sensor_bundle_version": SENSOR_VERSION,
        "time_origin": TIME_ORIGIN,
        "files_read": files.len(),
        "files_skipped": skipped.iter().map(|(n, r)| json!({"file": n, "reason": r})).collect::<Vec<_>>(),
        "sample_rows": rows,
        "status_counts": totals,
        "per_file": files.iter().map(|f| json!({
            "file": f.file_name, "run_id": f.run_id, "stream": f.stream, "format": f.format.as_str(),
            "samples": f.samples, "channels": f.channels, "status_counts": f.statuses,
            "ignition_stated": f.ignition.is_some(),
        })).collect::<Vec<_>>(),
        "units_converted": "PSIA/PSIG/PSID/psi/kPa/Torr -> Pa; Deg C/Deg F/Deg K -> K; VDC/mV -> V; Adc/mA -> A; SLPM/SCCM -> Sm3/s; % -> 1; s/ms -> s; Hz; W",
    });
    let q_bytes = serde_json::to_vec_pretty(&quality)?;
    let q_ref = artifact_ref(
        Digest::of_bytes(&q_bytes),
        q_bytes.len() as u64,
        "application/json",
        None,
        store,
    )?;

    let mut manifest = Sha256::new();
    for f in &files {
        manifest.update(format!("{}\t{}\n", f.file_name, f.source_id).as_bytes());
    }
    let bundle = DatasetBundle {
        schema_version: Default::default(),
        dataset_id: DatasetId::new(dataset_id(inv))?,
        family: Family::Other,
        audience: BundleAudience::Builder,
        cohort_id: format!("{inv}:all-sensor-files"),
        split_id: crate::dataset::UNSPLIT.into(),
        partitions: [Partition::Train].into_iter().collect(),
        source_manifest_hash: format!("sha256:{}", hex::encode(manifest.finalize())),
        time_origin: TIME_ORIGIN.into(),
        views: vec![
            DatasetView {
                name: "sensor_samples".into(),
                artifact: samples_ref.clone(),
                columns: sample_specs().into_iter().map(|(s, _)| s).collect(),
                row_count: rows,
            },
            DatasetView {
                name: "sensor_files".into(),
                artifact: files_ref.clone(),
                columns: file_specs().into_iter().map(|(s, _)| s).collect(),
                row_count: files.len() as u64,
            },
        ],
        quality_report: Some(q_ref.clone()),
        inclusion_criteria: BTreeMap::from([
            ("investigation".into(), inv.into()),
            ("files".into(), "Raw Data sensor/settings text in the mdca or acme_log format; every other file skipped with its reason".into()),
            ("partition".into(), "train = the whole investigation: no frozen split exists and the investigation holds no held-out outcome (not a lockbox investigation)".into()),
            ("sensor_bundle_version".into(), SENSOR_VERSION.into()),
            ("values".into(), "raw text, raw number and raw unit kept; SI only for units in the conversion table; explicit status per value".into()),
        ]),
        origin: Origin::System,
        created_at: Timestamp::now(),
    };
    bundle.validate()?;
    let built = crate::dataset::BuiltBundle {
        bundle: bundle.clone(),
        // The samples Parquet is already in place (streamed); the store writes the rest.
        files: vec![(files_ref, files_bytes), (q_ref, q_bytes)],
        quality: quality.clone(),
    };
    store.write_ref(&samples_ref)?;
    let path = store.save(&built)?;
    Ok(SensorBuild {
        bundle,
        path,
        files,
        skipped,
        quality,
    })
}

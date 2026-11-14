//! FLEX (PSI-69) ingest: summary table, test sheets, DropletData / ColorCam / LLUV series.
//!
//! Real source structure (PSI-69 v5, verified 2026-11-07):
//!
//! * `PSI-69_Experimental table_FLEX.csv` (cp1252, 274 rows): `FLEX Test #, FLEX Identifier` (the CIR
//!   test-point id, e.g. `20CAL13`, `C08M401`), `Test Date` (m/d/yyyy or m/d/yy), `Test GMT`, `Fuel`,
//!   `Ambient pressure; mmHg`, `O`/`N`/`CO`/`He initial ambient composition; mole fraction`,
//!   `Droplet initial diameter; mm`, `Visible flame extinction diameter; mm`, `Burning rate; mm`
//!   (sic, the quantity is mm^2/s), `Burn time; s`, `Test end` (Extinction/Disruption/Completion).
//!   Missing cells are empty or an en dash; one cell holds an undecodable byte.
//! * `PSI-69_Raw Data_FLEX-NNN_TestSheet.txt`: date, GMT time, `CIR Test Number`, pressure (atm),
//!   O2/N2/CO2/He mole fractions, temperature (K), fuel, `D0 (mm)`, `kavg (mm2/s)`, `Fiber: Yes/No`,
//!   `Dext`, free-text `COMMENTS`.
//! * `..._DropletData.txt`: CR line endings, tab separated `DropletTime_sec, DropletDiameter_mm,
//!   ZeroedDropletXpos_mm, ZeroedDropletYpos_mm` (empty cells allowed). Time zero is NASA's processed
//!   origin, not necessarily ignition.
//! * `..._ColorCamData.txt` / `..._LLUVData.txt`: other objects, own time base.
//! * `PSI-69_Raw Data_<CIR>_MDCA_Testpoint_Settings.txt`: instrument log with `TP_PARAMETER_FIBER_USAGE_FLAG`.
//!
//! Identity: the `FLEX-NNN` number in a file name is *not* the table's `FLEX Test #`. A file group is
//! bound to a table row by CIR id + test date (+ GMT time as corroboration); all aliases are kept.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

use ignis_core::contracts::{
    Alias, Censoring, Derivation, EndpointRecord, EvidenceClass, ExperimentRecord, Family,
    Measured, Method, Observation, Quantity, RawValue, Resolution, Session, Source,
};
use ignis_core::interpret::FieldRule;
use ignis_core::prefix::{ProspectivePolicy, ALWAYS_FORBIDDEN};
use ignis_core::views::FLEX_CAUSES;
use indexmap::IndexMap;
use regex::Regex;
use serde_json::json;

use super::{r, view_rules, Kept};
use crate::common::{
    det, loc, loc_chars, measured, parse_number_opt, text_value, Bundle, ReviewItem, M,
};
use crate::sources::manifest as mf;
use crate::text::{
    char_index, decode_utf8_or_cp1252, lower, normalize_newlines, py_csv, py_float_parse, strip,
    table_rows, Row,
};

pub const INV: &str = "PSI-69";
const FAM: &str = "flex";
const V: &str = "1";
pub const SUMMARY: &str = "PSI-69_Experimental table_FLEX.csv";
pub const PROSPECTIVE: [&str; 9] = [
    "pressure",
    "x_o2",
    "x_n2",
    "x_co2",
    "x_he",
    "ambient_temperature",
    "initial_diameter",
    "fuel",
    "fiber",
];

pub fn policy() -> ProspectivePolicy {
    let mut forbidden: Vec<&str> = vec![
        "k_full",
        "burning_rate",
        "kavg",
        "final_diameter",
        "extinction_diameter",
        "burn_time",
        "test_end",
        "dext",
    ];
    forbidden.extend(ALWAYS_FORBIDDEN);
    ProspectivePolicy::new(Family::Flex, &PROSPECTIVE, &forbidden)
}

/// The summary header reconciliation (documented in IGNIS_V3.1 §3.2).
const HEADER_FIX: [(&str, &str, &str); 4] = [
    ("O initial ambient composition; mole fraction", "x_o2",
     "header 'O' reconciled to O2: test sheets of the same runs report 'O2 Mole Fraction' with equal values"),
    ("N initial ambient composition; mole fraction", "x_n2", "header 'N' reconciled to N2 (test sheet 'N2')"),
    ("CO initial ambient composition; mole fraction", "x_co2",
     "header 'CO' reconciled to CO2: FLEX used CO2 as diluent/suppressant; test sheets say 'CO2 Mole Fraction'"),
    ("He initial ambient composition; mole fraction", "x_he", "He as written"),
];
const BURNING_RATE_HEADER: &str = "Burning rate; mm";

pub fn rules() -> Vec<FieldRule> {
    use Method::*;
    let mut v = vec![
        r(FAM, "summary_row", Deterministic, "flex.summary.csv",
          "CSV row parse; cp1252; header names kept verbatim, row = 1-based line number in the file"),
        r(FAM, "testsheet", Deterministic, "flex.testsheet.txt",
          "line-oriented 'Label: value' / 'Label = value' parse; comments block kept verbatim with offsets"),
        r(FAM, "series", Deterministic, "flex.series.tsv",
          "CR/CRLF/LF lines, tab separated, first line = header; empty cell -> None (not zero)"),
        r(FAM, "identity", Deterministic, "flex.identity.cir_date_gmt",
          "file group -> summary row by exact CIR id + same test date, GMT within 120 s corroborates or \
           disambiguates; a placeholder test-sheet date (year < 2008, e.g. 'December 31, 0002') is not evidence: \
           then CIR unique in the table + GMT within 120 s resolves. File-name FLEX number is an alias only"),
        r(FAM, "liquid_diameter", Deterministic, "flex.obs.droplet_diameter",
          "DropletDiameter_mm (NASA image-derived liquid geometry) -> m; flags: non_positive, non_finite, \
           missing, duplicate_timestamp, gap_gt_1s, non_monotonic_time; flagged points are kept, never dropped"),
        r(FAM, "conditions", Deterministic, "flex.conditions.summary_vs_testsheet",
          "pre-run conditions from the summary row (mmHg, mole fractions with header reconciliation), \
           cross-checked against the test sheet; disagreement beyond tolerance -> unresolved_conflict, both kept"),
        r(FAM, "post_run", Deterministic, "flex.post.summary",
          "terminal / full-history facts (burning rate mm^2/s despite header 'mm', burn time, extinction \
           diameter, test end, kavg, Dext) — stored outside conditions, forbidden in prefixes"),
        r(FAM, "free_floating", Semantic, "flex.cohort.free_floating_review",
          "structured Fiber flag (test sheet, MDCA TP_PARAMETER_FIBER_USAGE_FLAG) vs narrative COMMENTS; a \
           deterministic keyword screen only routes items; disagreement or missing evidence -> review item, \
           unresolved until an adjudication names agent/reviewer"),
        r(FAM, "endpoint", Deterministic, "flex.endpoint.draft",
          "draft from the original 'Test end' label: Extinction/Completion -> hot_phase_end, Disruption -> \
           disruption; event times are NOT set (time origin of the table burn time vs DropletTime is not \
           adjudicated) so censoring stays unknown"),
        r(FAM, "flex_endpoint", Semantic, "flex.endpoint.auto_adjudication",
          "cause, censoring, event bounds and event_before_support_exit from two independent LLM passes over the \
           run's evidence dossier (test sheet, summary row, series digests); every value cites dossier locators \
           checked by code; agreement -> resolved, disagreement or invalid -> unresolved_conflict (run leaves the \
           cohort). Times are in the DropletData processed time base"),
    ];
    v.extend(view_rules(FAM));
    v.sort_by(|a, b| a.field.cmp(b.field));
    v
}

pub fn decode(raw: &[u8]) -> String {
    decode_utf8_or_cp1252(raw)
}

pub fn parse_summary(raw: &[u8]) -> Vec<Row> {
    table_rows(&decode(raw), false)
}

/// Parsed test sheet (`parse_testsheet` in the Python pipeline).
#[derive(Debug, Clone, Default)]
pub struct Sheet {
    pub label: String,
    pub fields: HashMap<String, String>,
    pub lines: HashMap<String, i64>,
    pub comments: Option<String>,
    pub comments_span: Option<(i64, i64)>,
    pub test_date_iso: Option<String>,
    pub test_time_gmt: Option<String>,
}

impl Sheet {
    pub fn get(&self, k: &str) -> Option<&str> {
        self.fields.get(k).map(|s| s.as_str())
    }
    fn line(&self, k: &str) -> Option<i64> {
        self.lines.get(k).copied()
    }
}

const TS_FIELDS: [(&str, &str); 17] = [
    ("Test Date", "test_date"),
    ("Test Time", "test_time"),
    ("CIR Test Number", "cir_test_number"),
    ("Pressure (atm)", "pressure_atm"),
    ("O2 Mole Fraction", "x_o2"),
    ("N2 Mole Fraction", "x_n2"),
    ("CO2 Mole Fraction", "x_co2"),
    ("He Mole Fraction", "x_he"),
    ("Temperature (K)", "temperature_k"),
    ("Fuel", "fuel"),
    ("D0  (mm)", "d0_mm"),
    ("D0 (mm)", "d0_mm"),
    ("kavg  (mm2/s)", "kavg_mm2s"),
    ("kavg (mm2/s)", "kavg_mm2s"),
    ("Fiber", "fiber"),
    ("Dext  (mm)", "dext_mm"),
    ("Dext (mm)", "dext_mm"),
];

fn re(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).unwrap())
}

const WEEKDAYS: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];
const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// `datetime.strptime(s, "%A, %B %d, %Y").date()` (weekday parsed, not checked, like Python).
fn strptime_long(s: &str) -> Option<chrono::NaiveDate> {
    static R: OnceLock<Regex> = OnceLock::new();
    let c = re(
        &R,
        r"(?i)^([a-z]+),\s+([a-z]+)\s+(3[01]|[12]\d|0[1-9]|[1-9]| [1-9]),\s+(\d\d\d\d)$",
    )
    .captures(s)?;
    if !WEEKDAYS.contains(&lower(&c[1]).as_str()) {
        return None;
    }
    let m = MONTHS.iter().position(|x| *x == lower(&c[2]))? as u32 + 1;
    chrono::NaiveDate::from_ymd_opt(c[4].parse().ok()?, m, c[3].trim().parse().ok()?)
}

pub fn parse_testsheet(raw: &[u8]) -> Sheet {
    static LINE: OnceLock<Regex> = OnceLock::new();
    static GMT: OnceLock<Regex> = OnceLock::new();
    let text = normalize_newlines(&decode(raw));
    let mut out = Sheet::default();
    let lines: Vec<&str> = text.split('\n').collect();
    out.label = strip(lines.first().copied().unwrap_or("")).to_string();
    let lre = re(&LINE, r"^\s*([^:=]+?)\s*[:=]\s*(.*?)\s*$");
    for (n, line) in lines.iter().enumerate() {
        let Some(m) = lre.captures(line) else {
            continue;
        };
        let key = m[1].trim_end();
        for (label, name) in TS_FIELDS {
            if key == label && !out.fields.contains_key(name) {
                out.fields.insert(name.into(), strip(&m[2]).to_string());
                out.lines.insert(name.into(), n as i64 + 1);
            }
        }
    }
    if let Some(i) = char_index(&text, "COMMENTS:") {
        let start = i + "COMMENTS:".chars().count();
        let body: String = text.chars().skip(start).collect();
        let stripped = strip(&body).to_string();
        let span = if stripped.is_empty() {
            (start, start)
        } else {
            let first: String = stripped.chars().take(1).collect();
            let off = char_index(&body, &first).unwrap();
            (start + off, start + off + stripped.chars().count())
        };
        out.comments = Some(stripped);
        out.comments_span = Some((span.0 as i64, span.1 as i64));
    }
    if let Some(d) = out.get("test_date") {
        out.test_date_iso = strptime_long(d)
            .filter(|d| chrono::Datelike::year(d) >= 2008)
            .map(|d| d.to_string());
    }
    if let Some(t) = out.get("test_time") {
        out.test_time_gmt = re(&GMT, r"^(\d{1,2}:\d{2}:\d{2})")
            .captures(t)
            .map(|c| c[1].to_string());
    }
    out
}

pub type SeriesRow = (i64, Vec<Option<f64>>);

pub fn parse_series(raw: &[u8]) -> (Vec<String>, Vec<SeriesRow>) {
    let text = normalize_newlines(&decode(raw));
    let lines: Vec<&str> = text.split('\n').collect();
    let header: Vec<String> = lines[0].split('\t').map(|h| strip(h).to_string()).collect();
    let mut rows = Vec::new();
    for (i, line) in lines.iter().enumerate().skip(1) {
        if strip(line).is_empty() {
            continue;
        }
        let mut vals: Vec<Option<f64>> = line
            .split('\t')
            .take(header.len())
            .map(|c| {
                let c = strip(c);
                if c.is_empty() {
                    None
                } else {
                    Some(py_float_parse(c).unwrap_or(f64::NAN))
                }
            })
            .collect();
        vals.resize(header.len(), None);
        rows.push((i as i64 + 1, vals));
    }
    (header, rows)
}

/// `strptime(s, "%m/%d/%Y")` then `"%m/%d/%y"`.
pub fn parse_us_date(s: &str) -> Option<chrono::NaiveDate> {
    static R4: OnceLock<Regex> = OnceLock::new();
    static R2: OnceLock<Regex> = OnceLock::new();
    let s = strip(s);
    let md = r"(1[0-2]|0[1-9]|[1-9])/(3[01]|[12]\d|0[1-9]|[1-9]| [1-9])/";
    if let Some(c) = re(&R4, &format!("^{md}(\\d\\d\\d\\d)$")).captures(s) {
        if let Some(d) = chrono::NaiveDate::from_ymd_opt(
            c[3].parse().ok()?,
            c[1].parse().ok()?,
            c[2].trim().parse().ok()?,
        ) {
            return Some(d);
        }
    }
    let c = re(&R2, &format!("^{md}(\\d\\d)$")).captures(s)?;
    let yy: i32 = c[3].parse().ok()?;
    let y = if yy < 69 { 2000 + yy } else { 1900 + yy };
    chrono::NaiveDate::from_ymd_opt(y, c[1].parse().ok()?, c[2].trim().parse().ok()?)
}

fn zfill8(s: &str) -> String {
    format!("{s:0>8}")
}

/// `time.fromisoformat` for the H:MM:SS / HH:MM[:SS[.ffffff]] spellings present in FLEX.
fn iso_seconds(s: &str) -> Option<i64> {
    static R: OnceLock<Regex> = OnceLock::new();
    let c = re(&R, r"^(\d{2})(?::(\d{2})(?::(\d{2})(?:\.\d{1,6})?)?)?$").captures(s)?;
    let h: i64 = c[1].parse().ok()?;
    let m: i64 = c.get(2).map_or(Some(0), |x| x.as_str().parse().ok())?;
    let sec: i64 = c.get(3).map_or(Some(0), |x| x.as_str().parse().ok())?;
    (h < 24 && m < 60 && sec < 60).then_some(h * 3600 + m * 60 + sec)
}

fn gmt_dt(g1: Option<&str>, g2: Option<&str>) -> Option<i64> {
    let (g1, g2) = (g1.filter(|s| !s.is_empty())?, g2.filter(|s| !s.is_empty())?);
    let a = iso_seconds(&zfill8(g1))?;
    let b = iso_seconds(&zfill8(strip(g2)))?;
    let d = (a - b).abs();
    Some(d.min(86400 - d))
}

fn row_date_iso(r: &Row) -> Option<String> {
    parse_us_date(r.get("Test Date")).map(|d| d.to_string())
}

pub fn match_identity<'a>(
    sheet: &Sheet,
    summary: &'a [Row],
) -> (Option<&'a Row>, Resolution, String) {
    use Resolution::*;
    let cir = strip(sheet.get("cir_test_number").unwrap_or("")).to_string();
    if cir.is_empty() {
        return (None, Unknown, "test sheet has no CIR test number".into());
    }
    let cands: Vec<&Row> = summary
        .iter()
        .filter(|r| strip(r.get("FLEX Identifier")) == cir)
        .collect();
    if cands.is_empty() {
        return (None, Unknown, format!("CIR {cir} not in summary table"));
    }
    let g = sheet.test_time_gmt.as_deref();
    let Some(sd) = sheet.test_date_iso.as_deref() else {
        let ok: Vec<&Row> = cands
            .iter()
            .copied()
            .filter(|r| matches!(gmt_dt(g, Some(r.get("Test GMT"))), Some(d) if d <= 120))
            .collect();
        let raw = crate::text::py_repr(sheet.get("test_date").unwrap_or("None"));
        let raw = if sheet.get("test_date").is_none() {
            "None".to_string()
        } else {
            raw
        };
        if cands.len() == 1 && ok.len() == 1 {
            return (
                Some(ok[0]),
                Resolved,
                format!("CIR {cir} unique + GMT within 120 s; test-sheet date invalid ({raw})"),
            );
        }
        return (
            if cands.len() == 1 {
                Some(cands[0])
            } else {
                None
            },
            UnresolvedConflict,
            format!("test-sheet date invalid ({raw}) and CIR/GMT do not single out a row"),
        );
    };
    let same_day: Vec<&Row> = cands
        .iter()
        .copied()
        .filter(|r| row_date_iso(r).as_deref() == Some(sd))
        .collect();
    if same_day.len() > 1 {
        let ok: Vec<&Row> = same_day
            .iter()
            .copied()
            .filter(|r| matches!(gmt_dt(g, Some(r.get("Test GMT"))), Some(d) if d <= 120))
            .collect();
        if ok.len() == 1 {
            return (
                Some(ok[0]),
                Resolved,
                format!(
                    "CIR {cir} has {} rows on {sd}; GMT singles out row {}",
                    same_day.len(),
                    ok[0].row
                ),
            );
        }
        return (
            None,
            UnresolvedConflict,
            format!(
                "CIR {cir} has {} rows on {sd}; GMT does not disambiguate",
                same_day.len()
            ),
        );
    }
    if same_day.len() == 1 {
        let r = same_day[0];
        return match gmt_dt(g, Some(r.get("Test GMT"))) {
            None => (
                Some(r),
                Resolved,
                format!("CIR {cir} + date {sd}; GMT missing on one side"),
            ),
            Some(dt) if dt > 120 => (
                Some(r),
                UnresolvedConflict,
                format!("CIR+date match but GMT differs by {dt} s"),
            ),
            Some(dt) => (
                Some(r),
                Resolved,
                format!("CIR {cir} + date {sd} + GMT within {dt} s"),
            ),
        };
    }
    if cands.len() == 1 {
        return (
            Some(cands[0]),
            UnresolvedConflict,
            format!(
                "CIR {cir} matches one row but dates differ ({sd} vs {})",
                cands[0].get("Test Date")
            ),
        );
    }
    (
        None,
        UnresolvedConflict,
        format!("CIR {cir} has {} rows, none on {sd}", cands.len()),
    )
}

fn key_bits(t: f64) -> u64 {
    if t == 0.0 {
        0
    } else {
        t.to_bits()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn series_observations(
    run_id: &str,
    sha: &str,
    file_name: &str,
    header: &[String],
    rows: &[SeriesRow],
    time_col: &str,
    value_col: &str,
    quantity: Quantity,
    unit: &str,
    channel: &str,
) -> Vec<Observation> {
    let ti = header.iter().position(|h| h == time_col).unwrap();
    let vi = header.iter().position(|h| h == value_col).unwrap();
    let mut out = Vec::new();
    let mut seen: HashSet<u64> = HashSet::new();
    let mut prev_t: Option<f64> = None;
    let der = Derivation {
        method: Method::Deterministic,
        method_id: format!("flex.obs.{channel}.{value_col}"),
        method_version: V.into(),
        inputs: vec![format!("sha256:{sha}")],
        agent: None,
        notes: Some(
            "time_s is NASA's processed time base for this channel (origin not adjudicated)".into(),
        ),
    };
    let unit_si = ignis_core::units::to_si(1.0, unit).unwrap().1;
    for (line_no, vals) in rows {
        let (t, v) = (vals[ti], vals[vi]);
        let Some(t) = t.filter(|t| t.is_finite()) else {
            continue;
        };
        let mut flags: Vec<String> = vec![];
        if !seen.insert(key_bits(t)) {
            flags.push("duplicate_timestamp".into());
        }
        if let Some(p) = prev_t {
            if t < p {
                flags.push("non_monotonic_time".into());
            } else if t - p > 1.0 {
                flags.push("gap_gt_1s".into());
            }
        }
        prev_t = Some(match prev_t {
            Some(p) if t <= p => p,
            _ => t,
        });
        let mut si = None;
        match v {
            None => flags.push("missing".into()),
            Some(x) if !x.is_finite() => flags.push("non_finite".into()),
            Some(x) => {
                if x <= 0.0 {
                    flags.push("non_positive".into());
                }
                si = Some(ignis_core::units::to_si(x, unit).unwrap().0);
            }
        }
        let raw_value = match v {
            Some(x) if x.is_finite() => Some(RawValue::Number(x)),
            None => None,
            Some(_) => Some(RawValue::Text("nan".into())),
        };
        out.push(Observation {
            id: format!("{run_id}/{channel}/{value_col}/{line_no}"),
            run_id: run_id.into(),
            quantity,
            channel: Some(channel.into()),
            time_s: t,
            available_at_s: t,
            raw_value,
            raw_unit: Some(unit.into()),
            value_si: si,
            unit_si: unit_si.into(),
            uncertainty_si: None,
            calibration_id: None,
            quality_flags: flags,
            evidence_class: EvidenceClass::SourceDerived,
            locator: loc(sha, file_name, Some(*line_no), Some(value_col)),
            derivation: der.clone(),
        });
    }
    out
}

fn with_flag(mut m: Measured, f: String) -> Measured {
    m.flags.push(f);
    m
}

fn cross(
    m: Measured,
    sheet: &Sheet,
    ts_key: &str,
    ts_unit: Option<&str>,
    tol_rel: f64,
    tol_abs: f64,
) -> Measured {
    let tv = parse_number_opt(sheet.get(ts_key));
    let (Some(v), Some(tv)) = (m.value_si, tv) else {
        return m;
    };
    let tsi = match ts_unit {
        Some(u) => ignis_core::units::to_si(tv, u).unwrap().0,
        None => tv,
    };
    if (tsi - v).abs() > tol_abs.max(tol_rel * v.abs()) {
        let raw = sheet.get(ts_key).unwrap_or("None");
        let mut m = with_flag(
            m,
            strip(&format!(
                "testsheet_disagrees:{raw} {}",
                ts_unit.unwrap_or("")
            ))
            .to_string(),
        );
        m.resolution = Resolution::UnresolvedConflict;
        return m;
    }
    with_flag(m, "testsheet_agrees".into())
}

fn py_opt(s: Option<&str>) -> String {
    s.map(String::from).unwrap_or_else(|| "None".into())
}

pub fn conditions(
    row: &Row,
    sheet: &Sheet,
    sum_sha: &str,
    ts_sha: &str,
    ts_name: &str,
) -> BTreeMap<String, Measured> {
    let rl = |col: &str| Some(loc(sum_sha, SUMMARY, Some(row.row), Some(col)));
    let tl = |name: &str| Some(loc(ts_sha, ts_name, sheet.line(name), None));
    let mut c = BTreeMap::new();
    let p = measured(
        row.get("Ambient pressure; mmHg"),
        Some("mmHg"),
        rl("Ambient pressure; mmHg"),
        M::new("flex.cond.pressure")
            .notes("absolute chamber pressure; mmHg conventional (133.322387415 Pa)"),
    );
    c.insert(
        "pressure".to_string(),
        cross(p, sheet, "pressure_atm", Some("atm"), 0.03, 0.0),
    );
    for (col, key, why) in HEADER_FIX {
        let first = col.split(' ').next().unwrap();
        let m = measured(
            row.get(col),
            Some("1"),
            rl(col),
            M::new("flex.cond.header_reconciliation")
                .notes(why)
                .flags_owned(vec![format!("original_header:{first}")]),
        );
        c.insert(key.to_string(), cross(m, sheet, key, None, 0.0, 0.011));
    }
    let s: f64 = ["x_o2", "x_n2", "x_co2", "x_he"]
        .iter()
        .filter_map(|k| c[*k].value_si)
        .sum();
    if (s - 1.0).abs() > 0.011 {
        let m = c.remove("x_o2").unwrap();
        c.insert(
            "x_o2".into(),
            with_flag(m, format!("composition_sum={s:.3}")),
        );
    }
    if sheet.fields.contains_key("temperature_k") {
        c.insert(
            "ambient_temperature".into(),
            measured(
                sheet.get("temperature_k"),
                Some("K"),
                tl("temperature_k"),
                M::new("flex.cond.temperature"),
            ),
        );
    }
    let d0 = measured(
        row.get("Droplet initial diameter; mm"),
        Some("mm"),
        rl("Droplet initial diameter; mm"),
        M::new("flex.cond.initial_diameter")
            .evidence(EvidenceClass::SourceDerived)
            .notes("initial liquid diameter as tabulated (known at ignition)"),
    );
    c.insert(
        "initial_diameter".into(),
        cross(d0, sheet, "d0_mm", Some("mm"), 0.05, 0.0),
    );
    c.insert(
        "fuel".into(),
        text_value(
            row.get("Fuel"),
            rl("Fuel"),
            "flex.cond.fuel",
            None,
            None,
            &[],
        ),
    );
    c.insert(
        "fiber".into(),
        text_value(
            sheet.get("fiber"),
            tl("fiber"),
            "flex.cond.fiber_flag",
            Some("structured 'Fiber:' field of the test sheet".into()),
            None,
            &[],
        ),
    );
    if lower(strip(sheet.get("fuel").unwrap_or(""))) != lower(strip(row.get("Fuel"))) {
        let f = c.get_mut("fuel").unwrap();
        f.resolution = Resolution::UnresolvedConflict;
        f.flags = vec![format!("testsheet_fuel:{}", py_opt(sheet.get("fuel")))];
    }
    c
}

pub fn post_run(
    row: &Row,
    sheet: &Sheet,
    sum_sha: &str,
    ts_sha: &str,
    ts_name: &str,
) -> IndexMap<String, Measured> {
    let rl = |col: &str| Some(loc(sum_sha, SUMMARY, Some(row.row), Some(col)));
    let mut out = IndexMap::new();
    out.insert(
        "burning_rate".into(),
        measured(
            row.get(BURNING_RATE_HEADER),
            Some("mm2/s"),
            rl(BURNING_RATE_HEADER),
            M::new("flex.post.burning_rate")
                .evidence(EvidenceClass::SourceDerived)
                .notes("header says 'mm'; a burning-rate constant is length^2/time (mm^2/s)")
                .flags(&["unit_corrected_from_header:mm"]),
        ),
    );
    out.insert(
        "burn_time".into(),
        measured(
            row.get("Burn time; s"),
            Some("s"),
            rl("Burn time; s"),
            M::new("flex.post.burn_time"),
        ),
    );
    out.insert(
        "extinction_diameter".into(),
        measured(
            row.get("Visible flame extinction diameter; mm"),
            Some("mm"),
            rl("Visible flame extinction diameter; mm"),
            M::new("flex.post.extinction_diameter"),
        ),
    );
    out.insert(
        "test_end".into(),
        text_value(
            row.get("Test end"),
            rl("Test end"),
            "flex.post.test_end",
            None,
            None,
            &[],
        ),
    );
    out.insert(
        "kavg".into(),
        measured(
            sheet.get("kavg_mm2s"),
            Some("mm2/s"),
            Some(loc(ts_sha, ts_name, sheet.line("kavg_mm2s"), None)),
            M::new("flex.post.kavg").evidence(EvidenceClass::SourceDerived),
        ),
    );
    out.insert(
        "dext".into(),
        text_value(
            sheet.get("dext_mm"),
            Some(loc(ts_sha, ts_name, sheet.line("dext_mm"), None)),
            "flex.post.dext",
            None,
            None,
            &[],
        ),
    );
    out
}

pub fn free_floating(sheet: &Sheet, mdca_flag: Option<&str>) -> (Resolution, Option<bool>, String) {
    static FIB: OnceLock<Regex> = OnceLock::new();
    static FREE: OnceLock<Regex> = OnceLock::new();
    static MULTI: OnceLock<Regex> = OnceLock::new();
    let flag = lower(strip(sheet.get("fiber").unwrap_or("")));
    let comments = sheet.comments.as_deref().unwrap_or("");
    let narr_fiber = re(&FIB, r"(?i)\b(fiber[- ]supported|tethered|on the fiber|fiber[- ]mounted|supported by (a|the) fiber)\b").is_match(comments);
    let narr_free = re(
        &FREE,
        r"(?i)\b(free[- ]?float\w*|unsupported|free droplet)\b",
    )
    .is_match(comments);
    let yn = |s: &str| match s {
        "no" => Some(false),
        "yes" => Some(true),
        _ => None,
    };
    let Some(structured) = yn(&flag) else {
        return (Resolution::Unknown, None, "no structured fiber flag".into());
    };
    let mdca = yn(&lower(strip(mdca_flag.unwrap_or(""))));
    if matches!(mdca, Some(m) if m != structured) {
        return (
            Resolution::UnresolvedConflict,
            None,
            format!(
                "test sheet Fiber={flag} vs MDCA log fiber usage={}",
                py_opt(mdca_flag)
            ),
        );
    }
    if !structured && narr_fiber {
        return (
            Resolution::UnresolvedConflict,
            None,
            "Fiber: No but narrative mentions fiber support".into(),
        );
    }
    if structured && narr_free {
        return (
            Resolution::UnresolvedConflict,
            None,
            "Fiber: Yes but narrative says free-floating".into(),
        );
    }
    if re(
        &MULTI,
        r"(?i)\b(two droplets|multiple droplets|second droplet)\b",
    )
    .is_match(comments)
    {
        return (
            Resolution::UnresolvedConflict,
            None,
            "narrative mentions more than one droplet".into(),
        );
    }
    (
        Resolution::Resolved,
        Some(!structured),
        "structured flag consistent with narrative screen".into(),
    )
}

pub fn endpoint_draft(
    run_id: &str,
    row: Option<&Row>,
    sheet: &Sheet,
    sum_sha: &str,
    ts_sha: &str,
    ts_name: &str,
) -> EndpointRecord {
    let label = row.map(|r| strip(r.get("Test end")).to_string());
    let l = lower(label.as_deref().unwrap_or(""));
    let mut causes: Vec<String> = match l.trim_end_matches('.') {
        "extinction" | "completion" => vec!["hot_phase_end".into()],
        "disruption" => vec!["disruption".into()],
        _ => vec![],
    };
    if label
        .as_deref()
        .is_some_and(|s| !s.is_empty() && lower(s).starts_with("disrupt"))
    {
        causes = vec!["disruption".into()];
    }
    assert!(causes.iter().all(|c| FLEX_CAUSES.contains(&c.as_str())));
    let mut ev = vec![];
    if let Some(r) = row {
        ev.push(loc(sum_sha, SUMMARY, Some(r.row), Some("Test end")));
    }
    if let Some((a, b)) = sheet.comments_span {
        ev.push(loc_chars(ts_sha, ts_name, Some(a), Some(b)));
    }
    EndpointRecord {
        run_id: run_id.into(),
        original_label: label,
        original_comment: sheet.comments.clone(),
        allowed_causes: causes,
        censoring: Censoring::Unknown,
        last_confirmed_active_s: None,
        first_terminal_s: None,
        intervention_s: None,
        intervention_kind: None,
        observation_state: None,
        resolution: Resolution::Unknown,
        evidence: ev,
        derivation: det(
            "flex.endpoint.draft",
            V,
            vec![],
            Some("draft label map; times need adjudication from series/video".into()),
        ),
    }
}

pub fn mdca_fiber_flag(raw: &[u8]) -> Option<String> {
    let text = normalize_newlines(&decode(raw));
    let lines: Vec<&str> = text.split('\n').filter(|l| !strip(l).is_empty()).collect();
    if lines.len() < 2 {
        return None;
    }
    let header = py_csv(lines[0]).into_iter().next().unwrap_or_default();
    let values = py_csv(lines[1]).into_iter().next().unwrap_or_default();
    let i = header
        .iter()
        .position(|h| h == "TP_PARAMETER_FIBER_USAGE_FLAG(N/A)")?;
    values.get(i).map(|v| strip(v).to_string())
}

fn file_sha(srcs: &BTreeMap<String, Source>, name: &str) -> Option<String> {
    srcs.get(name).map(mf::sha_of)
}

fn read(src: &Source) -> anyhow::Result<Vec<u8>> {
    let p = mf::local_path(src);
    std::fs::read(&p).map_err(|e| anyhow::anyhow!("{}: {e}", p.display()))
}

pub fn ingest() -> anyhow::Result<Bundle> {
    static SHEET: OnceLock<Regex> = OnceLock::new();
    static NUM: OnceLock<Regex> = OnceLock::new();
    // Only assets whose bytes are present locally (a manifest may list more than this checkout holds).
    let srcs: BTreeMap<String, Source> = mf::by_name(INV)?
        .into_iter()
        .filter(|(_, s)| mf::local_path(s).exists())
        .collect();
    let mut b = Bundle::new(FAM);
    let Some(sum_src) = srcs.get(SUMMARY) else {
        anyhow::bail!("FLEX summary table not in manifest; run `ignis sources samples PSI-69`");
    };
    let sum_sha = mf::sha_of(sum_src);
    let summary = parse_summary(&read(sum_src)?);
    let sheet_re = re(&SHEET, r"^PSI-69_Raw Data_FLEX-\d+_TestSheet\.txt$");
    let sheets: Vec<&String> = srcs.keys().filter(|n| sheet_re.is_match(n)).collect();
    let mut used_rows: HashMap<i64, String> = HashMap::new();
    let mut used_ids: HashMap<String, Option<i64>> = HashMap::new();
    for ts_name in sheets {
        let num = re(&NUM, r"FLEX-(\d+)_").captures(ts_name).unwrap()[1].to_string();
        let ts_sha = file_sha(&srcs, ts_name).unwrap();
        let sheet = parse_testsheet(&read(&srcs[ts_name])?);
        let (row, mut res, mut why) = match_identity(&sheet, &summary);
        let cir = strip(sheet.get("cir_test_number").unwrap_or("")).to_string();
        let day = sheet
            .test_date_iso
            .clone()
            .or_else(|| row.and_then(row_date_iso));
        let mut run_id = if !cir.is_empty() {
            format!("{INV}:{cir}:{}", day.as_deref().unwrap_or("None"))
        } else {
            format!("{INV}:file:FLEX-{num}")
        };
        if let Some(other_row) = used_ids.get(&run_id) {
            match (row, other_row) {
                (Some(r), Some(o)) if r.row != *o => {
                    let g = strip(r.get("Test GMT"));
                    run_id = format!(
                        "{run_id}T{}",
                        if g.is_empty() {
                            "nogmt".to_string()
                        } else {
                            g.replace(':', "")
                        }
                    );
                }
                _ => {
                    res = Resolution::UnresolvedConflict;
                    why += &format!("; id {run_id} already used by another file group");
                    run_id = format!("{run_id}#FLEX-{num}");
                }
            }
        }
        used_ids.insert(run_id.clone(), row.map(|r| r.row));
        let mut aliases = vec![
            Alias {
                name: format!("FLEX-{num}"),
                scheme: "file_flex_number".into(),
                locator: Some(loc(&ts_sha, ts_name, None, None)),
            },
            Alias {
                name: sheet.label.clone(),
                scheme: "testsheet_header".into(),
                locator: Some(loc(&ts_sha, ts_name, Some(1), None)),
            },
        ];
        if !cir.is_empty() {
            aliases.push(Alias {
                name: cir.clone(),
                scheme: "cir_test_point".into(),
                locator: Some(loc(&ts_sha, ts_name, sheet.line("cir_test_number"), None)),
            });
        }
        let mut flags: Vec<String> = vec![];
        if let Some(r) = row {
            aliases.push(Alias {
                name: r.get("FLEX Test #").into(),
                scheme: "summary_flex_number".into(),
                locator: Some(loc(&sum_sha, SUMMARY, Some(r.row), Some("FLEX Test #"))),
            });
            if strip(r.get("FLEX Test #")).trim_start_matches('0') != num.trim_start_matches('0') {
                flags.push(format!(
                    "file_number_differs_from_summary:{num}!={}",
                    r.get("FLEX Test #")
                ));
            }
            if let Some(o) = used_rows.get(&r.row) {
                res = Resolution::UnresolvedConflict;
                why += &format!("; summary row also claimed by {o}");
            }
            used_rows.insert(r.row, run_id.clone());
        }
        let session_id = day.as_ref().map(|d| format!("{INV}:day:{d}"));
        if let Some(sid) = &session_id {
            if !b.sessions.contains_key(sid) {
                let gmt = sheet
                    .test_time_gmt
                    .as_ref()
                    .map(|g| format!("{}T{}Z", day.as_ref().unwrap(), zfill8(g)));
                b.sessions.insert(
                    sid.clone(),
                    Session {
                        id: sid.clone(),
                        investigation_id: INV.into(),
                        campaign_id: None,
                        test_day: day.clone(),
                        gmt_start: gmt,
                        evidence: vec![loc(&ts_sha, ts_name, sheet.line("test_date"), None)],
                        resolution: Resolution::Resolved,
                    },
                );
            }
        }
        let mut conds = match row {
            Some(r) => conditions(r, &sheet, &sum_sha, &ts_sha, ts_name),
            None => BTreeMap::new(),
        };
        let mdca_name = format!("PSI-69_Raw Data_{cir}_MDCA_Testpoint_Settings.txt");
        let mut mdca_flag = None;
        if let Some(src) = srcs.get(&mdca_name) {
            mdca_flag = mdca_fiber_flag(&read(src)?);
            if let Some(f) = &mdca_flag {
                conds.insert(
                    "fiber_mdca_log".into(),
                    text_value(
                        f.as_str(),
                        Some(loc(
                            &mf::sha_of(src),
                            &mdca_name,
                            Some(3),
                            Some("TP_PARAMETER_FIBER_USAGE_FLAG(N/A)"),
                        )),
                        "flex.cond.mdca_fiber_flag",
                        None,
                        None,
                        &[],
                    ),
                );
            }
        }
        let (ff_res, ff, ff_why) = free_floating(&sheet, mdca_flag.as_deref());
        if ff_res != Resolution::Resolved {
            if let Some(comments) = &sheet.comments {
                let (a, bb) = sheet
                    .comments_span
                    .map(|(a, b)| (Some(a), Some(b)))
                    .unwrap_or((None, None));
                let mut ctx = serde_json::Map::new();
                ctx.insert("structured_fiber".into(), json!(sheet.get("fiber")));
                ctx.insert("mdca_fiber_usage_flag".into(), json!(mdca_flag));
                b.review.push(ReviewItem {
                    id: format!("{run_id}#free_floating"), family: FAM.into(), run_id: run_id.clone(), field: "free_floating".into(),
                    question: "Is this a single free-floating (not fiber-supported) droplet for the whole observed burn?".into(),
                    cited_text: comments.clone(), locator: loc_chars(&ts_sha, ts_name, a, bb),
                    candidates: vec!["free_floating".into(), "fiber_supported".into(), "mixed_or_unclear".into()],
                    deterministic_screen: Some(ff_why), context: ctx,
                });
            }
        }
        flags.push(format!(
            "free_floating:{}",
            match ff {
                Some(true) => "yes",
                Some(false) => "no",
                None => "unresolved",
            }
        ));
        b.records.push(ExperimentRecord {
            id: run_id.clone(),
            investigation_id: INV.into(),
            family: Family::Flex,
            session_id,
            specimen_id: None,
            aliases,
            fuel_or_material: match row {
                Some(r) => Some(strip(r.get("Fuel")).to_string()),
                None => sheet.get("fuel").map(String::from),
            },
            configuration: Some("droplet".into()),
            gravity: None,
            conditions: conds,
            identity_resolution: res,
            identity_evidence: vec![det(
                "flex.identity.cir_date_gmt",
                V,
                vec![format!("sha256:{ts_sha}"), format!("sha256:{sum_sha}")],
                Some(why),
            )],
            quality_flags: flags,
            notes: None,
        });
        if let Some(r) = row {
            b.post.insert(
                run_id.clone(),
                post_run(r, &sheet, &sum_sha, &ts_sha, ts_name),
            );
        }
        b.endpoints.push(endpoint_draft(
            &run_id, row, &sheet, &sum_sha, &ts_sha, ts_name,
        ));
        for (kind, tcol, vcol, q) in [
            (
                "DropletData",
                "DropletTime_sec",
                "DropletDiameter_mm",
                Quantity::LiquidDiameter,
            ),
            (
                "ColorCamData",
                "ColorCamTime_sec",
                "ColorCamFlameDiameter_mm",
                Quantity::VisibleFlameDiameter,
            ),
            (
                "LLUVData",
                "LLUVTime_sec",
                "LLUVFlameDiameter_mm",
                Quantity::UvFlameDiameter,
            ),
        ] {
            let name = format!("PSI-69_Raw Data_FLEX-{num}_{kind}.txt");
            let Some(src) = srcs.get(&name) else { continue };
            let (header, rows) = parse_series(&read(src)?);
            if !header.iter().any(|h| h == tcol) || !header.iter().any(|h| h == vcol) {
                b.extra
                    .entry("series_errors".into())
                    .or_default()
                    .push(json!({"run_id": run_id, "file": name, "header": header}));
                continue;
            }
            let sha = mf::sha_of(src);
            b.observations.extend(series_observations(
                &run_id, &sha, &name, &header, &rows, tcol, vcol, q, "mm", kind,
            ));
            let ti = header.iter().position(|h| h == tcol).unwrap();
            let lost = rows
                .iter()
                .filter(|(_, v)| !v[ti].is_some_and(|t| t.is_finite()))
                .count();
            if lost > 0 {
                b.extra
                    .entry("rows_without_timestamp".into())
                    .or_default()
                    .push(json!({"run_id": run_id, "file": name, "rows": lost}));
            }
        }
    }
    let items = endpoint_review_items(&b);
    b.review.extend(items);
    Ok(b)
}

pub const ENDPOINT_QUESTION: &str = "How did the observed hot phase of this free-floating droplet end, and what is known about the \
    time of that end in the DropletData time base?";

/// One `flex_endpoint` item per run that passes every non-endpoint criterion, including the runs
/// held out only by an unresolved free-floating status (the item set does not move with that review).
fn endpoint_review_items(b: &Bundle) -> Vec<ReviewItem> {
    let n_valid = valid_diameter_points(b);
    let eps: HashMap<&str, &EndpointRecord> =
        b.endpoints.iter().map(|e| (e.run_id.as_str(), e)).collect();
    let mut items = vec![];
    for r in &b.records {
        if non_endpoint_exclusion(r, &n_valid, false).is_some() {
            continue;
        }
        let Some(e) = eps.get(r.id.as_str()) else {
            continue;
        };
        let Some(locator) = e.evidence.first().cloned() else {
            continue;
        };
        let label = e.original_label.clone().unwrap_or_default();
        let mut context = serde_json::Map::new();
        context.insert("test_end".into(), json!(label));
        if let Some(bt) = b.post.get(&r.id).and_then(|p| p.get("burn_time")) {
            context.insert(
                "burn_time_s_table".into(),
                json!(crate::common::raw_str(bt)),
            );
        }
        items.push(ReviewItem {
            id: format!("{}#flex_endpoint", r.id),
            family: FAM.into(),
            run_id: r.id.clone(),
            field: "flex_endpoint".into(),
            question: ENDPOINT_QUESTION.into(),
            cited_text: format!(
                "Test end={label}; comments={}",
                e.original_comment.as_deref().unwrap_or("")
            ),
            locator,
            candidates: ["hot_phase_end", "disruption", "unknown"]
                .map(String::from)
                .to_vec(),
            deterministic_screen: Some(
                "draft from the original 'Test end' label only; no event times set".into(),
            ),
            context,
        });
    }
    items
}

pub const COHORT_CRITERIA: &[(&str, &str)] = &[
    ("fuel", "methanol (summary and test sheet agree)"),
    ("configuration", "single free-floating droplet: structured Fiber=No, MDCA fiber usage not contradicting, \
                       narrative screen not contradicting (else unresolved -> excluded pending review)"),
    ("identity", "CIR + date (+GMT) bound to exactly one summary row"),
    ("series", "DropletData present with >= 10 valid liquid-diameter points"),
    ("phase", "observed after igniter retraction — not yet adjudicated; models must apply their own entry rule"),
];

fn valid_diameter_points(b: &Bundle) -> HashMap<&str, usize> {
    let mut n_valid: HashMap<&str, usize> = HashMap::new();
    for o in &b.observations {
        if o.quantity == Quantity::LiquidDiameter
            && o.value_si.is_some()
            && !o.quality_flags.iter().any(|f| {
                ["non_positive", "duplicate_timestamp", "non_monotonic_time"].contains(&f.as_str())
            })
        {
            *n_valid.entry(o.run_id.as_str()).or_default() += 1;
        }
    }
    n_valid
}

/// Why a run fails a criterion unrelated to its endpoint (None = passes).
fn non_endpoint_exclusion(
    r: &ExperimentRecord,
    n_valid: &HashMap<&str, usize>,
    count_unresolved_free_floating: bool,
) -> Option<String> {
    match r.conditions.get("fuel") {
        None => Some("fuel is not methanol".to_string()),
        Some(f) if lower(strip(&crate::common::raw_str(f))) != "methanol" => {
            Some("fuel is not methanol".into())
        }
        Some(f) if f.resolution != Resolution::Resolved => {
            Some("fuel conflict between summary and test sheet".into())
        }
        _ if r.identity_resolution != Resolution::Resolved => {
            Some(format!("identity {}", res_str(r.identity_resolution)))
        }
        _ if r.quality_flags.iter().any(|f| f == "free_floating:no") => {
            Some("fiber supported".into())
        }
        _ if count_unresolved_free_floating
            && r.quality_flags
                .iter()
                .any(|f| f == "free_floating:unresolved") =>
        {
            Some("free-floating status unresolved (review item pending)".into())
        }
        _ if n_valid.get(r.id.as_str()).copied().unwrap_or(0) < 10 => {
            Some("no DropletData sample fetched or < 10 valid points".into())
        }
        _ => None,
    }
}

pub fn cohort_filter(b: &Bundle) -> Kept {
    let n_valid = valid_diameter_points(b);
    let (mut keep, mut out) = (vec![], vec![]);
    for r in &b.records {
        let why = non_endpoint_exclusion(r, &n_valid, true).or_else(|| {
            // Only an adjudication writes this flag: without one the cohort is unchanged.
            r.quality_flags
                .iter()
                .any(|f| f == "flex_endpoint:unresolved")
                .then(|| {
                    "endpoint unresolved (adjudication passes disagree or invalid)".to_string()
                })
        });
        match why {
            Some(w) => out.push((r.id.clone(), w)),
            None => keep.push(r.id.clone()),
        }
    }
    (keep, out)
}

pub fn res_str(r: Resolution) -> String {
    serde_json::to_value(r)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

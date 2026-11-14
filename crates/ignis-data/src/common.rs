//! Shared curation helpers: SI-normalized [`Measured`] values, locators, bundles, JSONL I/O.

use std::io::Write;
use std::path::Path;

use anyhow::Context;
use ignis_core::contracts::{
    Derivation, EndpointRecord, EvidenceClass, Exclusion, ExperimentRecord, Measured, Method,
    Observation, RawValue, Resolution, Session, SourceLocator, Specimen,
};
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::{json, Value};

use crate::text::{lower, strip};

const MISSING_TOKENS: &[&str] = &[
    "", "-", "–", "—", "n/a", "na", "none", "#value!", "#div/0!", "#n/a", "?", "\u{fffd}", "tbd",
];

/// A raw cell as the Python pipeline saw it: absent, a string, or an already-parsed number.
#[derive(Debug, Clone, PartialEq)]
pub enum Raw {
    None,
    Str(String),
    Num(f64),
}

impl From<&str> for Raw {
    fn from(s: &str) -> Raw {
        Raw::Str(s.to_string())
    }
}
impl From<Option<&str>> for Raw {
    fn from(s: Option<&str>) -> Raw {
        s.map(|s| Raw::Str(s.to_string())).unwrap_or(Raw::None)
    }
}
impl From<f64> for Raw {
    fn from(x: f64) -> Raw {
        Raw::Num(x)
    }
}

impl Raw {
    fn stripped(&self) -> Raw {
        match self {
            Raw::Str(s) => Raw::Str(strip(s).to_string()),
            o => o.clone(),
        }
    }
    fn to_raw_value(&self) -> Option<RawValue> {
        match self {
            Raw::None => None,
            Raw::Str(s) => Some(RawValue::Text(s.clone())),
            Raw::Num(x) => Some(RawValue::Number(*x)),
        }
    }
    fn is_none_or_empty(&self) -> bool {
        matches!(self, Raw::None) || matches!(self, Raw::Str(s) if s.is_empty())
    }
}

pub fn is_missing(raw: &Raw) -> bool {
    match raw {
        Raw::None => true,
        Raw::Str(s) => MISSING_TOKENS.contains(&lower(strip(s)).as_str()),
        Raw::Num(_) => false,
    }
}

fn num_re() -> &'static regex::Regex {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"^[+-]?(\d+(\.\d*)?|\.\d+)([eE][+-]?\d+)?$").unwrap())
}

/// Strict: a cell is a number only if the whole cell is one. `'0.9   1.3'` is not a number.
pub fn parse_number_raw(raw: &Raw) -> Option<f64> {
    match raw {
        Raw::None => None,
        Raw::Num(x) => x.is_finite().then_some(*x),
        Raw::Str(s) => parse_number(s),
    }
}

pub fn parse_number(s: &str) -> Option<f64> {
    let s = strip(s);
    if num_re().is_match(s) {
        s.parse::<f64>().ok()
    } else {
        None
    }
}

pub fn parse_number_opt(s: Option<&str>) -> Option<f64> {
    s.and_then(parse_number)
}

pub fn det(
    method_id: &str,
    version: &str,
    inputs: Vec<String>,
    notes: Option<String>,
) -> Derivation {
    Derivation {
        method: Method::Deterministic,
        method_id: method_id.into(),
        method_version: version.into(),
        inputs,
        agent: None,
        notes,
    }
}

pub fn loc(sha: &str, path: &str, row: Option<i64>, column: Option<&str>) -> SourceLocator {
    SourceLocator {
        asset_sha256: sha.into(),
        path: Some(path.into()),
        row,
        column: column.map(String::from),
        page: None,
        frame: None,
        pts_s: None,
        char_start: None,
        char_end: None,
        ..Default::default()
    }
}

pub fn loc_chars(sha: &str, path: &str, a: Option<i64>, b: Option<i64>) -> SourceLocator {
    SourceLocator {
        char_start: a,
        char_end: b,
        ..loc(sha, path, None, None)
    }
}

/// Options of the Python `measured(...)` keyword arguments.
pub struct M<'a> {
    pub method_id: &'a str,
    pub version: &'a str,
    pub evidence: EvidenceClass,
    pub notes: Option<String>,
    pub flags: Vec<String>,
    pub uncertainty_reason: &'a str,
}

impl<'a> M<'a> {
    pub fn new(method_id: &'a str) -> M<'a> {
        M {
            method_id,
            version: "1",
            evidence: EvidenceClass::OriginalObservation,
            notes: None,
            flags: vec![],
            uncertainty_reason: "not reported by source",
        }
    }
    pub fn notes(mut self, n: impl Into<String>) -> Self {
        self.notes = Some(n.into());
        self
    }
    pub fn flags(mut self, f: &[&str]) -> Self {
        self.flags = f.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn flags_owned(mut self, f: Vec<String>) -> Self {
        self.flags = f;
        self
    }
    pub fn evidence(mut self, e: EvidenceClass) -> Self {
        self.evidence = e;
        self
    }
}

fn base(
    raw_value: Option<RawValue>,
    unit: Option<&str>,
    loc: Option<SourceLocator>,
    o: &M,
) -> Measured {
    Measured {
        raw_value,
        raw_unit: unit.map(String::from),
        value_si: None,
        unit_si: None,
        uncertainty_si: None,
        uncertainty_reason: Some(o.uncertainty_reason.to_string()),
        evidence_class: o.evidence,
        locator: loc,
        derivation: Some(det(o.method_id, o.version, vec![], o.notes.clone())),
        resolution: Resolution::Resolved,
        flags: o.flags.clone(),
    }
}

/// Parse + convert to SI. Missing/non-numeric cells stay missing with the reason in flags.
pub fn measured(
    raw: impl Into<Raw>,
    unit: Option<&str>,
    loc: Option<SourceLocator>,
    o: M,
) -> Measured {
    let raw_s = raw.into().stripped();
    if is_missing(&raw_s) {
        let rv = if raw_s.is_none_or_empty() {
            None
        } else {
            raw_s.to_raw_value()
        };
        let mut m = base(rv, unit, loc, &o);
        m.resolution = Resolution::Unknown;
        m.flags.push("missing_in_source".into());
        return m;
    }
    let Some(num) = parse_number_raw(&raw_s) else {
        let mut m = base(raw_s.to_raw_value(), unit, loc, &o);
        m.resolution = Resolution::Unknown;
        m.flags.push("non_numeric_cell".into());
        return m;
    };
    let Some(unit) = unit else {
        return base(Some(RawValue::Number(num)), None, loc, &o);
    };
    let (si, usi) = ignis_core::units::to_si(num, unit).expect("registered unit");
    let mut m = base(Some(RawValue::Number(num)), Some(unit), loc, &o);
    m.value_si = Some(si);
    m.unit_si = Some(usi.to_string());
    m
}

/// Python `text_value(...)`.
pub fn text_value(
    raw: impl Into<Raw>,
    loc: Option<SourceLocator>,
    method_id: &str,
    notes: Option<String>,
    resolution: Option<Resolution>,
    flags: &[&str],
) -> Measured {
    let raw_s = raw.into().stripped();
    let missing = is_missing(&raw_s);
    let rv = if missing && raw_s.is_none_or_empty() {
        None
    } else {
        raw_s.to_raw_value()
    };
    let mut fl: Vec<String> = flags.iter().map(|s| s.to_string()).collect();
    if missing {
        fl.push("missing_in_source".into());
    }
    Measured {
        raw_value: rv,
        raw_unit: None,
        value_si: None,
        unit_si: None,
        uncertainty_si: None,
        uncertainty_reason: None,
        evidence_class: EvidenceClass::OriginalObservation,
        locator: loc,
        derivation: Some(det(method_id, "1", vec![], notes)),
        resolution: resolution.unwrap_or(if missing {
            Resolution::Unknown
        } else {
            Resolution::Resolved
        }),
        flags: fl,
    }
}

/// `str(m.raw_value)` as Python would print it (None -> "None").
pub fn raw_str(m: &Measured) -> String {
    match &m.raw_value {
        None => "None".into(),
        Some(RawValue::Text(s)) => s.clone(),
        Some(RawValue::Number(x)) => crate::text::py_float(*x),
    }
}

/// A semantic question about cited source text. Written to `data/review/<family>/items.jsonl`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewItem {
    pub id: String,
    pub family: String,
    pub run_id: String,
    pub field: String,
    pub question: String,
    pub cited_text: String,
    pub locator: SourceLocator,
    pub candidates: Vec<String>,
    /// What the keyword/code screen found, never the answer.
    pub deterministic_screen: Option<String>,
    pub context: serde_json::Map<String, Value>,
}

impl ReviewItem {
    pub fn to_json(&self) -> Value {
        let mut locv = serde_json::to_value(&self.locator).unwrap();
        if let Value::Object(m) = &mut locv {
            m.retain(|_, v| !v.is_null());
        }
        json!({
            "id": self.id, "family": self.family, "run_id": self.run_id, "field": self.field,
            "question": self.question, "cited_text": self.cited_text, "locator": locv,
            "candidates": self.candidates, "deterministic_screen": self.deterministic_screen,
            "context": Value::Object(self.context.clone()),
        })
    }
    pub fn from_json(v: &Value) -> anyhow::Result<ReviewItem> {
        let s = |k: &str| {
            v[k].as_str()
                .map(String::from)
                .with_context(|| format!("review item field {k}"))
        };
        Ok(ReviewItem {
            id: s("id")?,
            family: s("family")?,
            run_id: s("run_id")?,
            field: s("field")?,
            question: s("question")?,
            cited_text: s("cited_text")?,
            locator: serde_json::from_value(v["locator"].clone())?,
            candidates: serde_json::from_value(v["candidates"].clone())?,
            deterministic_screen: v["deterministic_screen"].as_str().map(String::from),
            context: v["context"].as_object().cloned().unwrap_or_default(),
        })
    }
}

/// Everything one family ingest produces. `post` holds non-prospective run facts (never conditions).
#[derive(Debug, Clone, Default)]
pub struct Bundle {
    pub family: String,
    pub records: Vec<ExperimentRecord>,
    pub sessions: IndexMap<String, Session>,
    pub specimens: IndexMap<String, Specimen>,
    pub observations: Vec<Observation>,
    pub endpoints: Vec<EndpointRecord>,
    pub post: IndexMap<String, IndexMap<String, Measured>>,
    pub review: Vec<ReviewItem>,
    pub exclusions: Vec<Exclusion>,
    /// e.g. BASS `video_map`, ACME `control_history`, FLEX `series_errors`.
    pub extra: IndexMap<String, Vec<Value>>,
}

impl Bundle {
    pub fn new(family: &str) -> Bundle {
        Bundle {
            family: family.into(),
            ..Default::default()
        }
    }
}

/// One JSON line per row, keys sorted recursively (the Python `sort_keys=True` layout).
pub fn write_jsonl<T: Serialize>(
    path: &Path,
    rows: impl IntoIterator<Item = T>,
) -> anyhow::Result<usize> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut w = std::io::BufWriter::new(
        std::fs::File::create(path).with_context(|| path.display().to_string())?,
    );
    let mut n = 0;
    for r in rows {
        let v = ignis_core::hashing::sorted(&serde_json::to_value(&r)?);
        w.write_all(crate::text::py_dumps(&v).as_bytes())?;
        w.write_all(b"\n")?;
        n += 1;
    }
    w.flush()?;
    Ok(n)
}

pub fn read_jsonl_values(path: &Path) -> anyhow::Result<Vec<Value>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let text = std::fs::read_to_string(path).with_context(|| path.display().to_string())?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l).with_context(|| format!("{}: bad JSON line", path.display()))
        })
        .collect()
}

pub fn read_jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> anyhow::Result<Vec<T>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let text = std::fs::read_to_string(path).with_context(|| path.display().to_string())?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).with_context(|| format!("{}: bad record", path.display())))
        .collect()
}

/// `max(a, b)` / `min` helpers on f64 for Python parity (Python's max of floats).
pub fn fmax(a: f64, b: f64) -> f64 {
    if b > a {
        b
    } else {
        a
    }
}

/// The serde (snake_case) name of a contract enum value.
pub fn enum_str<T: Serialize>(t: T) -> String {
    serde_json::to_value(t)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

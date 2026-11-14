//! R9a: the conditions table of the 19 flight investigations (`ConditionsTable`, dataset.v2).
//!
//! Seeded from NASA's own experimental tables (one or two CSVs per investigation, the ground truth
//! the challenge asks to compare). Two sources, one rule:
//!
//! * **FLEX, CFI-G, BASS-II, BASS** already have typed family records (`data/work/records/<fam>/
//!   experiments.jsonl`) whose conditions *are* these table cells with locators; their rows reuse the
//!   records' run ids, identity and values, so the conditions join with the datasets and the lockbox.
//!   A key the records lack is read from the same table row (found through the records' locators).
//! * **The other 15** are read through a declarative [`Spec`] per table: which column is which key,
//!   in which unit.
//!
//! Honesty rules (the contract's): a value keeps raw value, raw unit, SI value and the locator of its
//! cell; a list or range in one cell ("21; 26.5; 40", "1.5 - 5") does not say which run used which
//! value, so the run-level cell is `missing` and quotes the text, while the investigation's tested
//! **envelope** takes the values; approximate cells ("~ 21.5") keep the flag `approximate`; a key the
//! table does not state is `missing` with a reason, never zero or assumed (no table states a gravity
//! level). Outcome columns (extinction diameter, burn time, spread rate) are never read: conditions
//! are inputs, so this table carries no held-out outcome.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Context;
use ignis_core::contracts::{
    ConditionCell, ConditionKey, ConditionsRow, ConditionsTable, DatasetSchema, Derivation,
    EnvelopeEntry, EvidenceClass, ExperimentRecord, Finite, InvestigationEnvelope, Measured,
    Method, Origin, RawValue, Resolution, RunId, RunIdentity, SourceLocator,
};
use regex::Regex;

use crate::common::loc;
use crate::sources::manifest as mf;
use crate::text::{decode_utf8_or_cp1252, table_rows, Row};

pub const METHOD: &str = "conditions.experimental_table";
pub const VERSION: &str = "1";

use ConditionKey as K;

/// Every key, in the column order of the table.
pub const KEYS: [ConditionKey; 10] = [
    K::OxygenMoleFraction,
    K::Pressure,
    K::FlowVelocity,
    K::GravityLevel,
    K::AmbientTemperature,
    K::Fuel,
    K::Diluent,
    K::SampleGeometry,
    K::Hardware,
    K::Diagnostics,
];

/// How a column's numbers are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Mole fraction (dimensionless, 0..1).
    Fraction,
    Percent,
    Atm,
    Bar,
    MmHg,
    CmPerS,
    Celsius,
    /// The unit is written in the cell ("1 atm", "30 cm/s", "~21% O2").
    Inline,
    /// Categorical text.
    Text,
}

#[derive(Debug, Clone, Copy)]
pub struct Col {
    pub header: &'static str,
    pub key: ConditionKey,
    pub unit: Unit,
}

const fn c(header: &'static str, key: ConditionKey, unit: Unit) -> Col {
    Col { header, key, unit }
}

/// What one row of a table is.
#[derive(Debug, Clone, Copy)]
pub enum RowKind {
    /// One run, identified by these columns (joined with `:`), under an optional label.
    Run(&'static [&'static str]),
    /// A campaign or test day (several runs); no run identity.
    Campaign,
}

#[derive(Debug, Clone, Copy)]
pub struct Spec {
    pub investigation: &'static str,
    pub file: &'static str,
    /// Run-id label between the investigation and the id (`PSI-10:normal:19115A1`).
    pub label: Option<&'static str>,
    pub rows: RowKind,
    pub cols: &'static [Col],
    /// Keys that do not apply to the investigation at all, with why.
    pub not_applicable: &'static [(ConditionKey, &'static str)],
    /// Flags added to every value of a key (e.g. which stream an O2 fraction belongs to).
    pub flags: &'static [(ConditionKey, &'static str)],
}

const NONE: &[(ConditionKey, &str)] = &[];
const INVERSE_O2: &[(ConditionKey, &str)] = &[(K::OxygenMoleFraction, "oxidizer_stream:burner")];

/// The 16 tables of the 15 investigations without family records.
pub const SPECS: [Spec; 16] = [
    Spec {
        investigation: "PSI-10",
        file: "PSI-10_Experimental table_Flame Design_Normal Flame.csv",
        label: Some("normal"),
        rows: RowKind::Run(&["Test"]),
        cols: &[
            c("Fuel", K::Fuel, Unit::Text),
            c("XO2 (init)", K::OxygenMoleFraction, Unit::Fraction),
            c("pmean (atm)", K::Pressure, Unit::Atm),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-10",
        file: "PSI-10_Experimental table_Flame Design_Inverse Flame.csv",
        label: Some("inverse"),
        rows: RowKind::Run(&["Test"]),
        cols: &[
            c("XO2", K::OxygenMoleFraction, Unit::Fraction),
            c("pmean (atm)", K::Pressure, Unit::Atm),
        ],
        not_applicable: NONE,
        flags: INVERSE_O2,
    },
    Spec {
        investigation: "PSI-39",
        file: "PSI-39_Experimental table_CFI.csv",
        label: None,
        rows: RowKind::Run(&["Flex Number"]),
        cols: &[
            c("Pressure", K::Pressure, Unit::Inline),
            c("Oxygen", K::OxygenMoleFraction, Unit::Fraction),
            c("Fuel", K::Fuel, Unit::Text),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-68",
        file: "PSI-68_Experimental table_Flex2.csv",
        label: None,
        rows: RowKind::Campaign,
        cols: &[
            c("Sample (Fuel Type)", K::Fuel, Unit::Text),
            c("Ambient Gas Mixture", K::Diluent, Unit::Text),
            c(
                "Oxygen Mole Fraction",
                K::OxygenMoleFraction,
                Unit::Fraction,
            ),
            c("Ambient Pressure (atm)", K::Pressure, Unit::Atm),
            c(
                "Ambient Temperature (C)",
                K::AmbientTemperature,
                Unit::Celsius,
            ),
            c("Flow Velocity (cm/s)", K::FlowVelocity, Unit::CmPerS),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-20",
        file: "PSI-20_Experimental table_ACME BRE.csv",
        label: None,
        rows: RowKind::Campaign,
        cols: &[
            c("Fuel Bottle", K::Fuel, Unit::Text),
            c(
                "Oxygen Conc (percent)",
                K::OxygenMoleFraction,
                Unit::Percent,
            ),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-21",
        file: "PSI-21_Experimental table_CLD Flame.csv",
        label: None,
        rows: RowKind::Campaign,
        cols: &[c("Fuel Bottle", K::Fuel, Unit::Text)],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-22",
        file: "PSI-22_Experimental table_ACME E-FIELD Flames.csv",
        label: None,
        rows: RowKind::Campaign,
        cols: &[c("Fuel Bottle", K::Fuel, Unit::Text)],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-23",
        file: "PSI-23_Experimental table_ACME s-Flame.csv",
        label: None,
        rows: RowKind::Campaign,
        cols: &[
            c("Fuel Bottle", K::Fuel, Unit::Text),
            c("Atmosphere*", K::OxygenMoleFraction, Unit::Inline),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-107",
        file: "PSI-107_Experimental table_Spice.csv",
        label: None,
        rows: RowKind::Run(&["GMT date", "GMT ignition"]),
        cols: &[
            c("Fuel", K::Fuel, Unit::Text),
            c("Air Rate (cm/s)", K::FlowVelocity, Unit::CmPerS),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-106",
        file: "PSI-106_Experimental table_Slice.csv",
        label: None,
        rows: RowKind::Campaign,
        cols: &[c("Fuel Type", K::Fuel, Unit::Text)],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-98",
        file: "PSI-98_Experimental table_SAFFIRE-1.csv",
        label: None,
        rows: RowKind::Run(&["Sample Number"]),
        cols: &[
            c("Material", K::Fuel, Unit::Text),
            c("Air Flow (cm/s)", K::FlowVelocity, Unit::CmPerS),
            c("Percent O2", K::OxygenMoleFraction, Unit::Percent),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-99",
        file: "PSI-99_Experimental table_SAFFIRE-2.csv",
        label: None,
        rows: RowKind::Run(&["Sample Number"]),
        cols: &[
            c("Material", K::Fuel, Unit::Text),
            c("Air Flow (cm/s)", K::FlowVelocity, Unit::Inline),
            c("Percent O2 (Note 1)", K::OxygenMoleFraction, Unit::Percent),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-100",
        file: "PSI-100_Experimental table_SAFFIRE-3.csv",
        label: None,
        rows: RowKind::Run(&["Sample Number"]),
        cols: &[
            c("Material", K::Fuel, Unit::Text),
            c("Air Flow", K::FlowVelocity, Unit::Inline),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-102",
        file: "PSI-102_Experimental table_SAME.csv",
        label: None,
        rows: RowKind::Run(&["Sample #"]),
        cols: &[
            c("Sample Material", K::Fuel, Unit::Text),
            c("Air Flow (cm/s)", K::FlowVelocity, Unit::CmPerS),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-101",
        file: "PSI-101_Experimental table_SAME-R.csv",
        label: None,
        rows: RowKind::Run(&["SAME Test #"]),
        cols: &[
            c("Sample Type", K::Fuel, Unit::Text),
            c("Inlet velocity (cm/s)", K::FlowVelocity, Unit::CmPerS),
        ],
        not_applicable: NONE,
        flags: NONE,
    },
    Spec {
        investigation: "PSI-47",
        file: "PSI-47_Experimental table_DAFT.csv",
        label: None,
        rows: RowKind::Run(&["Session Name"]),
        cols: &[],
        not_applicable: &[(
            K::Fuel,
            "DAFT sampled dust and aerosols for smoke-detector tests; nothing was burned",
        )],
        flags: NONE,
    },
];

/// Families whose records carry FLEX (PSI-69), CFI-G (PSI-159), BASS-II and BASS (PSI-25, PSI-26).
pub const FAMILIES: [&str; 4] = ["flex", "acme_normal", "acme_inverse", "bass"];

/// Record condition names per key, first present wins.
fn record_names(key: ConditionKey) -> &'static [&'static str] {
    match key {
        K::OxygenMoleFraction => &["x_o2", "x_o2_role"],
        K::Pressure => &["pressure"],
        K::FlowVelocity => &["flow_velocity"],
        K::AmbientTemperature => &["ambient_temperature"],
        K::Fuel => &["fuel", "fuel_species"],
        _ => &[],
    }
}

/// Table columns that fill a family key the records lack, per table file.
const FAMILY_EXTRAS: [(&str, &str, ConditionKey); 2] = [
    (
        "PSI-25_Experimental table_BASS-II.csv",
        "Fuel Sample Material",
        K::Fuel,
    ),
    (
        "PSI-26_Experimental table_BASS.csv",
        "Fuel Sample Material",
        K::Fuel,
    ),
];

// ------------------------------------------------------------------ cell parsing

/// A parsed numeric cell.
#[derive(Debug, Clone, PartialEq)]
pub enum Parsed {
    Empty,
    One {
        x: f64,
        unit: Option<String>,
        approx: bool,
    },
    /// A list or a range: several tested values, none of them the run's.
    Many {
        xs: Vec<f64>,
        unit: Option<String>,
    },
    Unparsed,
}

fn number_re() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(\d+(?:\.\d+)?|\.\d+)\s*(%|atm\b|bar\b|mmhg\b|psia?\b|cm/s|mm/s|m/s|°?c\b|k\b)?",
        )
        .unwrap()
    })
}

pub fn parse_cell(s: &str) -> Parsed {
    let t = s.trim();
    if t.is_empty()
        || ["-", "–", "—", "n/a", "na", "?", "none"].contains(&t.to_lowercase().as_str())
    {
        return Parsed::Empty;
    }
    let approx = t.starts_with('~') || t.starts_with('≈');
    // A digit glued to a letter belongs to a formula ("O2", "N2", "CH4"), not to a number.
    let caps: Vec<_> = number_re()
        .captures_iter(t)
        .filter(|c| {
            let start = c.get(0).map_or(0, |m| m.start());
            !t[..start]
                .chars()
                .next_back()
                .is_some_and(char::is_alphabetic)
        })
        .collect();
    if caps.is_empty() {
        return Parsed::Unparsed;
    }
    let xs: Vec<f64> = caps.iter().filter_map(|c| c[1].parse().ok()).collect();
    let unit = caps
        .iter()
        .rev()
        .find_map(|c| c.get(2).map(|u| u.as_str().to_lowercase()));
    if xs.len() == 1 {
        Parsed::One {
            x: xs[0],
            unit,
            approx,
        }
    } else {
        Parsed::Many { xs, unit }
    }
}

/// SI value and unit of `x` written in `unit` (column unit, or the cell's for `Inline`).
fn to_si(key: ConditionKey, col: Unit, cell_unit: Option<&str>, x: f64) -> Option<(f64, String)> {
    let raw = match col {
        Unit::Fraction => "1",
        Unit::Percent => "%",
        Unit::Atm => "atm",
        Unit::Bar => "bar",
        Unit::MmHg => "mmHg",
        Unit::CmPerS => "cm/s",
        Unit::Celsius => "C",
        Unit::Inline => cell_unit?,
        Unit::Text => return None,
    };
    let si = match (key, raw.to_lowercase().trim_start_matches('°')) {
        (K::OxygenMoleFraction, "1") => x,
        (K::OxygenMoleFraction, "%") => x / 100.0,
        (K::Pressure, "atm") => x * 101_325.0,
        (K::Pressure, "bar") => x * 1e5,
        (K::Pressure, "mmhg") => x * 133.322_387_415,
        (K::Pressure, "psi" | "psia") => x * 6_894.757_293,
        (K::FlowVelocity, "cm/s") => x * 0.01,
        (K::FlowVelocity, "mm/s") => x * 0.001,
        (K::FlowVelocity, "m/s") => x,
        (K::AmbientTemperature, "c") => x + 273.15,
        (K::AmbientTemperature, "k") => x,
        _ => return None,
    };
    let unit_si = key.quantity()?.unit_si().to_string();
    Some((si, unit_si))
}

fn derivation(notes: Option<String>) -> Derivation {
    Derivation {
        method: Method::Deterministic,
        method_id: METHOD.into(),
        method_version: VERSION.into(),
        inputs: vec![],
        agent: None,
        notes,
    }
}

fn measured(
    raw: RawValue,
    raw_unit: Option<String>,
    si: Option<(f64, String)>,
    l: SourceLocator,
    flags: Vec<String>,
) -> Measured {
    Measured {
        raw_value: Some(raw),
        raw_unit,
        value_si: si.as_ref().map(|s| s.0),
        unit_si: si.map(|s| s.1),
        uncertainty_si: None,
        uncertainty_reason: None,
        evidence_class: EvidenceClass::OriginalObservation,
        locator: Some(l),
        derivation: Some(derivation(None)),
        resolution: Resolution::Resolved,
        flags,
    }
}

/// What one table cell contributes: the row's cell and the values for the envelope.
struct Contribution {
    cell: ConditionCell,
    numbers: Vec<f64>,
    text: Option<String>,
}

fn missing(key: ConditionKey, reason: impl Into<String>) -> ConditionCell {
    ConditionCell::Missing {
        key,
        reason: reason.into(),
    }
}

fn from_cell(col: &Col, raw: &str, l: SourceLocator, extra_flags: &[String]) -> Contribution {
    let key = col.key;
    let at = format!("{} row {}", col.header, l.row.unwrap_or(0));
    if col.unit == Unit::Text {
        let t = raw.trim();
        return if parse_cell(t) == Parsed::Empty {
            Contribution {
                cell: missing(key, format!("empty cell ({at})")),
                numbers: vec![],
                text: None,
            }
        } else {
            Contribution {
                cell: ConditionCell::Value {
                    key,
                    value: measured(
                        RawValue::Text(t.into()),
                        None,
                        None,
                        l,
                        extra_flags.to_vec(),
                    ),
                },
                numbers: vec![],
                text: Some(t.to_string()),
            }
        };
    }
    match parse_cell(raw) {
        Parsed::Empty => Contribution {
            cell: missing(key, format!("empty cell ({at})")),
            numbers: vec![],
            text: None,
        },
        Parsed::Unparsed => Contribution {
            cell: missing(key, format!("not a number: {:?} ({at})", raw.trim())),
            numbers: vec![],
            text: None,
        },
        Parsed::Many { xs, unit } => {
            let si: Vec<f64> = xs
                .iter()
                .filter_map(|&x| to_si(key, col.unit, unit.as_deref(), x).map(|s| s.0))
                .collect();
            Contribution {
                cell: missing(
                    key,
                    format!(
                        "the cell lists several tested values, not this row's: {:?} ({at})",
                        raw.trim()
                    ),
                ),
                numbers: if si.len() == xs.len() { si } else { vec![] },
                text: None,
            }
        }
        Parsed::One { x, unit, approx } => match to_si(key, col.unit, unit.as_deref(), x) {
            None => Contribution {
                cell: missing(
                    key,
                    format!(
                        "unit not stated or not convertible: {:?} ({at})",
                        raw.trim()
                    ),
                ),
                numbers: vec![],
                text: None,
            },
            Some((si, unit_si)) => {
                let raw_unit = match col.unit {
                    Unit::Inline => unit,
                    Unit::Fraction => Some("1".into()),
                    Unit::Percent => Some("%".into()),
                    Unit::Atm => Some("atm".into()),
                    Unit::Bar => Some("bar".into()),
                    Unit::MmHg => Some("mmHg".into()),
                    Unit::CmPerS => Some("cm/s".into()),
                    Unit::Celsius => Some("C".into()),
                    Unit::Text => None,
                };
                let mut flags = extra_flags.to_vec();
                if approx {
                    flags.push("approximate".into());
                }
                Contribution {
                    cell: ConditionCell::Value {
                        key,
                        value: measured(
                            RawValue::Number(x),
                            raw_unit,
                            Some((si, unit_si)),
                            l,
                            flags,
                        ),
                    },
                    numbers: vec![si],
                    text: None,
                }
            }
        },
    }
}

/// A cell that must hold, or a `missing` cell carrying the reason it does not.
/// Physical bounds of an SI value for a combustion test atmosphere. A value outside them is kept
/// out (never corrected): NASA's BRE table has "3400%" for 34 %, a spreadsheet format slip.
pub fn plausible(key: ConditionKey, si: f64) -> bool {
    match key {
        K::OxygenMoleFraction => (0.0..=1.0).contains(&si),
        K::Pressure => si > 0.0 && si <= 1e7,
        K::FlowVelocity => (0.0..=100.0).contains(&si),
        K::AmbientTemperature => si > 0.0 && si <= 1500.0,
        K::GravityLevel => (0.0..=10.0).contains(&si),
        _ => true,
    }
}

/// Flags whose values describe something other than the test atmosphere (kept on the row, left
/// out of the envelope).
const NOT_ATMOSPHERE: [&str; 1] = ["oxidizer_stream:burner"];

/// A cell that must hold, or a `missing` cell carrying the reason it does not.
fn checked(c: Contribution) -> Contribution {
    let key = c.cell.key();
    if let ConditionCell::Value { value, .. } = &c.cell {
        if let Some(si) = value.value_si.filter(|&x| !plausible(key, x)) {
            let raw = match &value.raw_value {
                Some(RawValue::Number(x)) => format!("{x}"),
                Some(RawValue::Text(t)) => t.clone(),
                None => "?".into(),
            };
            let at = value
                .locator
                .as_ref()
                .map(|l| {
                    format!(
                        "{} row {}",
                        l.column.as_deref().unwrap_or("?"),
                        l.row.unwrap_or(0)
                    )
                })
                .unwrap_or_default();
            return Contribution {
                cell: missing(
                    key,
                    format!(
                        "physically implausible as written: {raw} {} = {si} {} ({at}); left out pending review",
                        value.raw_unit.as_deref().unwrap_or(""),
                        value.unit_si.as_deref().unwrap_or("")
                    ),
                ),
                numbers: vec![],
                text: None,
            };
        }
    }
    let mut c = match c.cell.validate() {
        Ok(()) => c,
        Err(e) => Contribution {
            cell: missing(key, format!("value refused: {}", e.reason)),
            numbers: vec![],
            text: None,
        },
    };
    if c.numbers.iter().any(|&x| !plausible(key, x)) {
        c.numbers.clear();
    }
    if let ConditionCell::Value { value, .. } = &c.cell {
        if value
            .flags
            .iter()
            .any(|f| NOT_ATMOSPHERE.contains(&f.as_str()))
        {
            c.numbers.clear();
        }
    }
    c
}

// ------------------------------------------------------------------ building

struct Built {
    row: ConditionsRow,
    contributions: Vec<Contribution>,
}

fn default_reason(key: ConditionKey) -> &'static str {
    match key {
        K::GravityLevel => "the experimental table states no gravity level",
        K::Hardware => "the experimental table names no hardware",
        K::Diagnostics => "the experimental table names no diagnostics",
        K::SampleGeometry => "the experimental table gives no sample geometry class",
        K::Diluent => "the experimental table names no diluent",
        _ => "the experimental table has no column for it",
    }
}

fn assemble(
    investigation: &str,
    run: RunIdentity,
    mut got: BTreeMap<ConditionKey, Contribution>,
    na: &[(ConditionKey, &str)],
    from_record: bool,
) -> Built {
    let mut contributions = Vec::new();
    for key in KEYS {
        let c = got.remove(&key).unwrap_or_else(|| {
            let cell = match na.iter().find(|(k, _)| *k == key) {
                Some((_, why)) => ConditionCell::NotApplicable {
                    key,
                    reason: (*why).into(),
                },
                None if from_record && !record_names(key).is_empty() => {
                    missing(key, "the family record holds no value for it")
                }
                None => missing(key, default_reason(key)),
            };
            Contribution {
                cell,
                numbers: vec![],
                text: None,
            }
        });
        contributions.push(c);
    }
    Built {
        row: ConditionsRow {
            investigation_id: investigation.into(),
            run,
            cells: vec![],
        },
        contributions,
    }
}

fn read_table(investigation: &str, file: &str) -> anyhow::Result<(String, Vec<Row>)> {
    let srcs = mf::by_name(investigation)?;
    let src = srcs.get(file).with_context(|| {
        format!("{investigation}: {file} is not in the manifest; fetch it first")
    })?;
    let p = mf::local_path(src);
    let raw = std::fs::read(&p).with_context(|| format!("{}", p.display()))?;
    Ok((
        mf::sha_of(src),
        table_rows(&decode_utf8_or_cp1252(&raw), true),
    ))
}

fn spec_rows(spec: &Spec) -> anyhow::Result<Vec<Built>> {
    let (sha, rows) = read_table(spec.investigation, spec.file)?;
    if let Some(first) = rows.first() {
        for h in spec.cols.iter().map(|c| c.header).chain(match spec.rows {
            RowKind::Run(ids) => ids.iter().copied().collect::<Vec<_>>(),
            RowKind::Campaign => vec![],
        }) {
            anyhow::ensure!(
                first.cells.contains_key(h),
                "{}: no column {h:?}",
                spec.file
            );
        }
    }
    let flags = |key: ConditionKey| -> Vec<String> {
        spec.flags
            .iter()
            .filter(|(k, _)| *k == key)
            .map(|(_, f)| f.to_string())
            .collect()
    };
    let mut seen = HashMap::new();
    let mut out = Vec::new();
    for r in &rows {
        let run = match spec.rows {
            RowKind::Campaign => RunIdentity {
                run_id: None,
                resolution: Resolution::NotApplicable,
            },
            RowKind::Run(ids) => {
                let parts: Vec<String> = ids
                    .iter()
                    .map(|h| r.opt(h).trim().replace(char::is_whitespace, "_"))
                    .collect();
                if parts.iter().any(String::is_empty) {
                    RunIdentity {
                        run_id: None,
                        resolution: Resolution::Unknown,
                    }
                } else {
                    let mut id = match spec.label {
                        Some(l) => format!("{}:{l}:{}", spec.investigation, parts.join(":")),
                        None => format!("{}:{}", spec.investigation, parts.join(":")),
                    };
                    let mut res = Resolution::Resolved;
                    let n = seen.entry(id.clone()).or_insert(0);
                    *n += 1;
                    if *n > 1 {
                        res = Resolution::UnresolvedConflict;
                        id = format!("{id}#row{}", r.row);
                    }
                    RunIdentity {
                        run_id: RunId::new(id).ok(),
                        resolution: res,
                    }
                }
            }
        };
        let mut got = BTreeMap::new();
        for col in spec.cols {
            let l = loc(&sha, spec.file, Some(r.row), Some(col.header));
            got.insert(
                col.key,
                checked(from_cell(col, r.opt(col.header), l, &flags(col.key))),
            );
        }
        out.push(assemble(
            spec.investigation,
            run,
            got,
            spec.not_applicable,
            false,
        ));
    }
    Ok(out)
}

fn read_records(family: &str) -> anyhow::Result<Vec<ExperimentRecord>> {
    let p = crate::paths::work(&["records", family]).join("experiments.jsonl");
    anyhow::ensure!(
        p.exists(),
        "{}: no records; run `ignis data ingest {family}` first",
        p.display()
    );
    crate::common::read_jsonl(&p)
}

fn table_row_of(r: &ExperimentRecord, file: &str) -> Option<i64> {
    let locs = r
        .conditions
        .values()
        .filter_map(|m| m.locator.as_ref())
        .chain(r.aliases.iter().filter_map(|a| a.locator.as_ref()));
    locs.filter(|l| l.path.as_deref() == Some(file))
        .find_map(|l| l.row)
}

fn family_rows(family: &str) -> anyhow::Result<Vec<Built>> {
    let records = read_records(family)?;
    let inverse = family == "acme_inverse";
    // Extra table columns (BASS material), by table file and row.
    let mut extras: HashMap<(String, i64), Vec<(ConditionKey, Contribution)>> = HashMap::new();
    for (file, header, key) in FAMILY_EXTRAS {
        let inv = file.split('_').next().unwrap_or("");
        if !records.iter().any(|r| r.investigation_id == inv) {
            continue;
        }
        let (sha, rows) = read_table(inv, file)?;
        for r in &rows {
            let col = Col {
                header,
                key,
                unit: Unit::Text,
            };
            let l = loc(&sha, file, Some(r.row), Some(header));
            extras
                .entry((file.to_string(), r.row))
                .or_default()
                .push((key, checked(from_cell(&col, r.opt(header), l, &[]))));
        }
    }
    let mut out = Vec::new();
    for r in &records {
        let run = RunIdentity {
            run_id: RunId::new(r.id.clone()).ok(),
            resolution: r.identity_resolution,
        };
        let mut got = BTreeMap::new();
        for key in KEYS {
            let Some((name, m)) = record_names(key)
                .iter()
                .find_map(|n| r.conditions.get(*n).map(|m| (*n, m)))
            else {
                continue;
            };
            let mut m = m.clone();
            if key == K::OxygenMoleFraction && inverse && name == "x_o2_role" {
                m.flags.push("oxidizer_stream:burner".into());
            }
            let numbers = m
                .value_si
                .filter(|_| key.quantity().is_some())
                .into_iter()
                .collect();
            let text = match &m.raw_value {
                Some(RawValue::Text(t)) if key.quantity().is_none() => Some(t.clone()),
                _ => None,
            };
            let c = if m.resolution == Resolution::Resolved {
                Contribution {
                    cell: ConditionCell::Value { key, value: m },
                    numbers,
                    text,
                }
            } else {
                Contribution {
                    cell: missing(
                        key,
                        format!("the family record marks {name} {:?}", m.resolution),
                    ),
                    numbers: vec![],
                    text: None,
                }
            };
            got.insert(key, checked(c));
        }
        for (file, _, _) in FAMILY_EXTRAS {
            if !file.starts_with(&format!("{}_", r.investigation_id)) {
                continue;
            }
            if let Some(row) = table_row_of(r, file) {
                for (key, c) in extras.remove(&(file.to_string(), row)).unwrap_or_default() {
                    got.entry(key).or_insert(c);
                }
            }
        }
        out.push(assemble(&r.investigation_id, run, got, NONE, true));
    }
    Ok(out)
}

fn envelope(investigation: &str, built: &[&Built]) -> InvestigationEnvelope {
    let mut entries = Vec::new();
    for (i, key) in KEYS.iter().enumerate() {
        let mut nums = Vec::new();
        let mut texts = BTreeSet::new();
        let mut runs = 0u64;
        for b in built {
            let c = &b.contributions[i];
            if !c.numbers.is_empty() || c.text.is_some() {
                runs += 1;
            }
            nums.extend(c.numbers.iter().copied());
            texts.extend(c.text.clone());
        }
        let entry = match key.quantity() {
            Some(q) if !nums.is_empty() => {
                let (lo, hi) = nums
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &x| {
                        (a.min(x), b.max(x))
                    });
                match (Finite::new(lo), Finite::new(hi)) {
                    (Ok(min_si), Ok(max_si)) => EnvelopeEntry::Range {
                        key: *key,
                        min_si,
                        max_si,
                        unit_si: q.unit_si().into(),
                        runs,
                    },
                    _ => EnvelopeEntry::Missing {
                        key: *key,
                        reason: "non-finite values".into(),
                    },
                }
            }
            None if !texts.is_empty() => EnvelopeEntry::Values {
                key: *key,
                values: texts,
                runs,
            },
            _ => EnvelopeEntry::Missing {
                key: *key,
                reason: match built.first().map(|b| &b.contributions[i].cell) {
                    Some(ConditionCell::NotApplicable { reason, .. }) => {
                        format!("not applicable: {reason}")
                    }
                    _ => "no row of the experimental table states it".into(),
                },
            },
        };
        entries.push(entry);
    }
    InvestigationEnvelope {
        investigation_id: investigation.into(),
        runs: built.len() as u64,
        entries,
    }
}

/// The full conditions table of the 19 flight investigations. An investigation that cannot be
/// read is an error, never a silently missing block.
pub fn build() -> anyhow::Result<ConditionsTable> {
    build_from(&FAMILIES, &SPECS)
}

/// The table over the given families (record files) and table specs.
pub fn build_from(families: &[&str], specs: &[Spec]) -> anyhow::Result<ConditionsTable> {
    let mut built: Vec<Built> = Vec::new();
    for family in families {
        built.extend(family_rows(family)?);
    }
    for spec in specs {
        built.extend(spec_rows(spec)?);
    }
    let mut by_inv: BTreeMap<String, Vec<&Built>> = BTreeMap::new();
    for b in &built {
        by_inv
            .entry(b.row.investigation_id.clone())
            .or_default()
            .push(b);
    }
    let envelopes = by_inv.iter().map(|(inv, bs)| envelope(inv, bs)).collect();
    let rows: Vec<ConditionsRow> = built
        .into_iter()
        .map(|mut b| {
            b.row.cells = b.contributions.into_iter().map(|c| c.cell).collect();
            b.row
        })
        .collect();
    let table = ConditionsTable {
        schema_version: DatasetSchema::V2,
        keys: KEYS.to_vec(),
        total_count: Some(rows.len() as u64),
        rows,
        envelopes,
        corpus_snapshot: None,
        next_cursor: None,
        origin: Origin::System,
    };
    table.validate().map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(table)
}

// ------------------------------------------------------------------ query (conditions_query)

const DEFAULT_PAGE: usize = 1000;

fn offset_of(c: Option<&ignis_core::contracts::PageCursor>) -> Result<usize, String> {
    match c {
        None => Ok(0),
        Some(c) => c
            .as_str()
            .strip_prefix("conditions:")
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| format!("{c} is not a conditions cursor")),
    }
}

fn value_text(c: &ConditionCell) -> Option<&str> {
    match c {
        ConditionCell::Value { value, .. } => match &value.raw_value {
            Some(RawValue::Text(t)) => Some(t),
            _ => None,
        },
        _ => None,
    }
}

fn keeps(
    row: &ConditionsRow,
    f: &ignis_core::contracts::ConditionFilter,
    include_unknown: bool,
) -> bool {
    let Some(c) = row.cells.iter().find(|c| c.key() == f.key) else {
        return false;
    };
    let v = match c {
        ConditionCell::Value { value, .. } => value,
        ConditionCell::NotApplicable { .. } => return false,
        _ => return include_unknown,
    };
    // A burner-stream O2 is not the test atmosphere: it never matches an atmosphere filter.
    if v.flags.iter().any(|x| NOT_ATMOSPHERE.contains(&x.as_str())) {
        return false;
    }
    if f.any_of.is_empty() {
        let Some(x) = v.value_si else {
            return false;
        };
        f.min_si.is_none_or(|m| x >= m.get()) && f.max_si.is_none_or(|m| x <= m.get())
    } else {
        value_text(c).is_some_and(|t| f.any_of.contains(t))
    }
}

/// The atmosphere envelope of visible rows (value cells only, atmosphere values only).
fn envelope_of_rows(investigation: &str, rows: &[&ConditionsRow]) -> InvestigationEnvelope {
    let built: Vec<Built> = rows
        .iter()
        .map(|r| Built {
            row: (*r).clone(),
            contributions: KEYS
                .iter()
                .map(|k| {
                    let cell = r
                        .cells
                        .iter()
                        .find(|c| c.key() == *k)
                        .cloned()
                        .unwrap_or_else(|| missing(*k, "absent"));
                    let (numbers, text) = match &cell {
                        ConditionCell::Value { value, .. } => {
                            let atmosphere = !value
                                .flags
                                .iter()
                                .any(|f| NOT_ATMOSPHERE.contains(&f.as_str()));
                            let n: Vec<f64> = value
                                .value_si
                                .filter(|&x| atmosphere && plausible(*k, x))
                                .into_iter()
                                .collect();
                            (
                                n,
                                value_text(&cell)
                                    .map(String::from)
                                    .filter(|_| k.quantity().is_none()),
                            )
                        }
                        _ => (vec![], None),
                    };
                    Contribution {
                        cell,
                        numbers,
                        text,
                    }
                })
                .collect(),
        })
        .collect();
    envelope(investigation, &built.iter().collect::<Vec<_>>())
}

/// Answer `q` over the full table. `visible` says whether the scope may see a run (the lockbox);
/// rows without a run identity (campaigns) are always visible. Envelopes of an investigation with
/// withheld rows are recomputed from its visible rows, so they cannot reveal what was withheld.
pub fn query(
    full: &ConditionsTable,
    q: &ignis_core::contracts::ConditionsQuery,
    visible: impl Fn(&RunId) -> bool,
) -> Result<ConditionsTable, String> {
    q.validate().map_err(|e| e.to_string())?;
    let keys: Vec<ConditionKey> = if q.keys.is_empty() {
        full.keys.clone()
    } else {
        let mut seen = BTreeSet::new();
        q.keys.iter().copied().filter(|k| seen.insert(*k)).collect()
    };
    let invs: BTreeSet<&str> = q.investigation_ids.iter().map(String::as_str).collect();
    let runs: BTreeSet<&RunId> = q.run_ids.iter().collect();
    let mut withheld_in: BTreeSet<String> = BTreeSet::new();
    let mut visible_by_inv: BTreeMap<String, Vec<&ConditionsRow>> = BTreeMap::new();
    let mut matching: Vec<&ConditionsRow> = Vec::new();
    for r in &full.rows {
        if !invs.is_empty() && !invs.contains(r.investigation_id.as_str()) {
            continue;
        }
        if let Some(id) = &r.run.run_id {
            if !visible(id) {
                withheld_in.insert(r.investigation_id.clone());
                continue;
            }
        }
        visible_by_inv
            .entry(r.investigation_id.clone())
            .or_default()
            .push(r);
        if !runs.is_empty() && !r.run.run_id.as_ref().is_some_and(|id| runs.contains(id)) {
            continue;
        }
        if q.filters.iter().all(|f| keeps(r, f, q.include_unknown)) {
            matching.push(r);
        }
    }
    let offset = offset_of(q.cursor.as_ref())?;
    let size = q.page_size.map_or(DEFAULT_PAGE, |n| n as usize);
    let page: Vec<ConditionsRow> = matching
        .iter()
        .skip(offset)
        .take(size)
        .map(|r| ConditionsRow {
            investigation_id: r.investigation_id.clone(),
            run: r.run.clone(),
            cells: keys
                .iter()
                .filter_map(|k| r.cells.iter().find(|c| c.key() == *k).cloned())
                .collect(),
        })
        .collect();
    let next = offset + page.len();
    let envelopes = if !q.include_envelopes {
        vec![]
    } else {
        full.envelopes
            .iter()
            .filter(|e| invs.is_empty() || invs.contains(e.investigation_id.as_str()))
            .filter_map(|e| {
                let env = if withheld_in.contains(&e.investigation_id) {
                    envelope_of_rows(
                        &e.investigation_id,
                        visible_by_inv.get(&e.investigation_id)?,
                    )
                } else {
                    e.clone()
                };
                Some(InvestigationEnvelope {
                    entries: env
                        .entries
                        .into_iter()
                        .filter(|x| {
                            let k = match x {
                                EnvelopeEntry::Range { key, .. }
                                | EnvelopeEntry::Values { key, .. }
                                | EnvelopeEntry::Missing { key, .. } => *key,
                            };
                            keys.contains(&k)
                        })
                        .collect(),
                    ..env
                })
            })
            .collect()
    };
    let out = ConditionsTable {
        schema_version: DatasetSchema::V2,
        keys,
        rows: page,
        envelopes,
        corpus_snapshot: full.corpus_snapshot.clone(),
        next_cursor: (next < matching.len())
            .then(|| ignis_core::contracts::PageCursor::new(format!("conditions:{next}")).ok())
            .flatten(),
        total_count: Some(matching.len() as u64),
        origin: full.origin,
    };
    out.validate().map_err(|e| e.to_string())?;
    Ok(out)
}

/// Where `ignis data conditions` writes the full table (and the service reads it).
pub fn table_path() -> std::path::PathBuf {
    crate::paths::work(&["conditions"]).join("conditions_table.json")
}

//! Numeric grounding (task M7b): a number a claim states is read back from the cell it is bound to
//! and compared, with no model involved.
//!
//! A [`ClaimValue`] names a JSON artifact pinned by digest, a chain of row selectors (key fields,
//! never positions), and a column. [`read_cell`] walks the document: a pointer that steps through an
//! array by position is refused, every selector must match exactly one row, and the cell must be a
//! number whose unit the artifact states (or an object with `value_si` + `unit_si` / `value` +
//! `unit`). [`compare`] then checks the stated value against the cell in SI. The only tolerance is
//! [`CONVERSION_REL_TOL`], for the floating-point error of a unit conversion; identical units are
//! compared exactly, and a value stated to `n` significant digits must equal the cell rounded to
//! `n` digits.

use ignis_core::contracts::{
    AnswerProblemKind, CellReading, ConditionCell, ConditionKey, ConditionsRow, ConditionsTable,
    KeyEquals, KeyValue, RawValue, RowSelector, RunId, SourceLocator,
};
use ignis_core::units;
use serde_json::Value;

/// Relative tolerance of a comparison in SI after a unit conversion: the error of one
/// multiply-add, not a measure of agreement. Identical units are compared exactly.
pub(crate) const CONVERSION_REL_TOL: f64 = 1e-12;

/// The number in a cell and the unit the artifact states for it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Cell {
    pub value: f64,
    pub unit: String,
}

pub(crate) type Problem = (AnswerProblemKind, String);

fn row(detail: impl Into<String>) -> Problem {
    (AnswerProblemKind::UnresolvedRow, detail.into())
}

fn column(detail: impl Into<String>) -> Problem {
    (AnswerProblemKind::UnknownColumn, detail.into())
}

/// RFC 6901 reference tokens of a well-formed pointer (validated by the contract).
fn tokens(pointer: &str) -> impl Iterator<Item = String> + '_ {
    pointer
        .split('/')
        .skip(1)
        .map(|t| t.replace("~1", "/").replace("~0", "~"))
}

/// Follow `pointer` from `from` through objects only, pushing every object stepped through onto
/// `chain` (the enclosing objects, nearest last). Returns the value and the last token.
fn walk<'a>(
    from: &'a Value,
    pointer: &str,
    chain: &mut Vec<&'a Value>,
) -> Result<(&'a Value, Option<String>), String> {
    let mut cur = from;
    let mut last = None;
    for t in tokens(pointer) {
        match cur {
            Value::Object(o) => {
                chain.push(cur);
                cur = o
                    .get(&t)
                    .ok_or_else(|| format!("{pointer:?}: no field {t:?} (fields: {})", fields(cur)))?;
            }
            Value::Array(_) => {
                return Err(format!(
                    "{pointer:?} steps through an array at {t:?}; positions are not stable, select the element with a row selector (key fields)"
                ))
            }
            _ => return Err(format!("{pointer:?}: {t:?} is under a value that is not an object")),
        }
        last = Some(t);
    }
    Ok((cur, last))
}

fn fields(v: &Value) -> String {
    match v.as_object() {
        Some(o) if !o.is_empty() => o.keys().cloned().collect::<Vec<_>>().join(", "),
        Some(_) => "none".into(),
        None => "not an object".into(),
    }
}

fn key_matches(v: &Value, want: &KeyValue) -> bool {
    match (want, v) {
        (KeyValue::Bool(b), Value::Bool(x)) => b == x,
        (KeyValue::Number(n), Value::Number(x)) => x.as_f64() == Some(n.get()),
        (KeyValue::Text(s), Value::String(x)) => s == x,
        _ => false,
    }
}

fn describe(keys: &[KeyEquals]) -> String {
    keys.iter()
        .map(|k| {
            let v = match &k.equals {
                KeyValue::Bool(b) => b.to_string(),
                KeyValue::Number(n) => n.get().to_string(),
                KeyValue::Text(s) => format!("{s:?}"),
            };
            format!("{} = {v}", k.field)
        })
        .collect::<Vec<_>>()
        .join(" and ")
}

/// A selected row: a record (row layout) or one index across a table's column arrays.
enum Selected<'a> {
    Record(&'a Value),
    Columns { table: &'a Value, index: usize },
}

/// Read the cell `column` of the row `rows` select in `doc`.
pub(crate) fn read_cell(
    doc: &Value,
    rows: &[RowSelector],
    column_ptr: &str,
) -> Result<Cell, Problem> {
    let mut chain: Vec<&Value> = Vec::new();
    let mut cur = Selected::Record(doc);
    for (n, sel) in rows.iter().enumerate() {
        let Selected::Record(record) = cur else {
            return Err(row(
                "a column-layout table selects the last row; no selector can follow it",
            ));
        };
        let (table, _) = walk(record, &sel.table, &mut chain).map_err(row)?;
        let keys = describe(&sel.keys);
        cur = match table {
            Value::Array(items) => {
                let hits: Vec<&Value> = items
                    .iter()
                    .filter(|it| {
                        sel.keys.iter().all(|k| {
                            walk(it, &k.field, &mut Vec::new())
                                .is_ok_and(|(v, _)| key_matches(v, &k.equals))
                        })
                    })
                    .collect();
                match hits.as_slice() {
                    [one] => Selected::Record(one),
                    [] => {
                        return Err(row(format!(
                            "no row of {:?} has {keys} ({} rows)",
                            sel.table,
                            items.len()
                        )))
                    }
                    many => {
                        return Err(row(format!(
                            "{} rows of {:?} have {keys}; add key fields until exactly one matches",
                            many.len(),
                            sel.table
                        )))
                    }
                }
            }
            Value::Object(_) => {
                let mut len = None;
                let mut hits: Option<Vec<bool>> = None;
                for k in &sel.keys {
                    let (col, _) = walk(table, &k.field, &mut Vec::new()).map_err(row)?;
                    let Some(col) = col.as_array() else {
                        return Err(row(format!(
                            "key column {} of {:?} is not an array (column layout keys rows by parallel arrays)",
                            k.field, sel.table
                        )));
                    };
                    if len.is_some_and(|l| l != col.len()) {
                        return Err(row(format!(
                            "key columns of {:?} differ in length",
                            sel.table
                        )));
                    }
                    len = Some(col.len());
                    let m: Vec<bool> = col.iter().map(|v| key_matches(v, &k.equals)).collect();
                    hits = Some(match hits {
                        None => m,
                        Some(h) => h.iter().zip(m).map(|(a, b)| *a && b).collect(),
                    });
                }
                let idx: Vec<usize> = hits
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, h)| h.then_some(i))
                    .collect();
                match idx.as_slice() {
                    [i] => {
                        chain.push(table);
                        Selected::Columns { table, index: *i }
                    }
                    [] => return Err(row(format!("no row of {:?} has {keys}", sel.table))),
                    many => {
                        return Err(row(format!(
                            "{} rows of {:?} have {keys}; add key fields until exactly one matches",
                            many.len(),
                            sel.table
                        )))
                    }
                }
            }
            _ => {
                return Err(row(format!(
                    "selector {n}: {:?} is neither an array of rows nor an object of column arrays",
                    sel.table
                )))
            }
        };
    }
    let (value, field, holder) = match cur {
        Selected::Record(record) => {
            let (v, field) = walk(record, column_ptr, &mut chain).map_err(column)?;
            (v, field, None)
        }
        Selected::Columns { table, index } => {
            if column_ptr.is_empty() {
                return Err(column(
                    "a column-layout row needs a column (e.g. \"/mean\")",
                ));
            }
            let mut inner = Vec::new();
            let (col, field) = walk(table, column_ptr, &mut inner).map_err(column)?;
            // The column's own object is where its unit sits; `walk` pushed it last.
            let holder = inner.last().copied();
            chain.extend(inner.iter().skip(1));
            let Some(arr) = col.as_array() else {
                return Err(column(format!(
                    "{column_ptr:?} is not a column array of the table"
                )));
            };
            let v = arr.get(index).ok_or_else(|| {
                column(format!(
                    "{column_ptr:?} has {} entries, shorter than the key columns",
                    arr.len()
                ))
            })?;
            (v, field, holder)
        }
    };
    cell_of(value, field.as_deref(), holder, &chain, column_ptr)
}

const GENERIC_UNIT_KEYS: &[&str] = &["unit", "si_unit", "units", "unit_si"];

fn text<'a>(o: &'a Value, key: &str) -> Option<&'a str> {
    o.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|u| !u.is_empty())
}

/// The number at the cell and its stated unit.
fn cell_of(
    v: &Value,
    field: Option<&str>,
    holder: Option<&Value>,
    chain: &[&Value],
    column_ptr: &str,
) -> Result<Cell, Problem> {
    let no_unit = || {
        (
            AnswerProblemKind::UnitMismatch,
            format!(
                "{column_ptr:?} holds a number with no stated unit; bind a cell whose unit the artifact states"
            ),
        )
    };
    if let Value::Object(_) = v {
        for (val, unit) in [("value_si", "unit_si"), ("value", "unit")] {
            if let Some(x) = v.get(val).and_then(Value::as_f64) {
                let u = text(v, unit).ok_or_else(no_unit)?;
                return Ok(Cell {
                    value: x,
                    unit: u.into(),
                });
            }
        }
        return Err(column(format!(
            "{column_ptr:?} is an object without value_si + unit_si or value + unit (fields: {}); point at the number",
            fields(v)
        )));
    }
    let Some(x) = v.as_f64() else {
        let what = match v {
            Value::Null => "null (missing), not a number".to_string(),
            other => format!("{other}, not a number"),
        };
        return Err(column(format!("{column_ptr:?} is {what}")));
    };
    // The object the number sits in (its row, or the table holding its column).
    let parent = holder.or_else(|| chain.last().copied());
    let specific = match field {
        Some("raw_value") => Some("raw_unit"),
        Some(f) if f == "value_si" || f.ends_with("_si") => Some("unit_si"),
        _ => None,
    };
    if let Some(key) = specific {
        // A raw value has only its raw unit; an SI value its SI unit (here or enclosing).
        let found = if key == "raw_unit" {
            parent.and_then(|p| text(p, key))
        } else {
            chain.iter().rev().find_map(|o| text(o, key))
        };
        let u = found.ok_or_else(no_unit)?;
        return Ok(Cell {
            value: x,
            unit: u.into(),
        });
    }
    // A field whose name states its unit (`pressure_gap_pa`) is read in that unit before any
    // generic `unit` of an enclosing object, which describes that object's value, not this field.
    if let Some(u) = field.and_then(unit_of_field_name) {
        return Ok(Cell {
            value: x,
            unit: u.into(),
        });
    }
    let u = parent
        .into_iter()
        .chain(chain.iter().rev().copied())
        .find_map(|o| GENERIC_UNIT_KEYS.iter().find_map(|k| text(o, k)))
        .ok_or_else(no_unit)?;
    Ok(Cell {
        value: x,
        unit: u.into(),
    })
}

/// Fields that are ratios by their contract (`Nearness`: an oxygen mole-fraction gap, a distance
/// in fractions of the target).
const DIMENSIONLESS_FIELDS: &[&str] = &["oxygen_gap", "distance"];

/// The unit a field name states with an SI-symbol suffix, as IGNIS artifacts name their columns
/// (`pressure_gap_pa`, `burn_time_s`, `diameter_mm`, `temperature_k`), or `1` for a ratio
/// (`*_fraction`, and the [`DIMENSIONLESS_FIELDS`]). `None`: the name states no unit.
fn unit_of_field_name(field: &str) -> Option<&'static str> {
    if DIMENSIONLESS_FIELDS.contains(&field) || field.ends_with("_fraction") {
        return Some("1");
    }
    const SUFFIXES: &[(&str, &str)] = &[
        ("_kpa", "kPa"),
        ("_mpa", "MPa"),
        ("_pa", "Pa"),
        ("_ms", "ms"),
        ("_s", "s"),
        ("_mm", "mm"),
        ("_cm", "cm"),
        ("_m", "m"),
        ("_k", "K"),
        ("_pct", "%"),
        ("_percent", "%"),
    ];
    let f = field.to_ascii_lowercase();
    SUFFIXES
        .iter()
        .find(|(s, _)| f.ends_with(s) && f.len() > s.len())
        .map(|(_, u)| *u)
}

/// Same cell of the same source: `asset_sha256`, `path` and `row` (the identity M11 publishes).
fn same_place(a: &SourceLocator, b: &SourceLocator) -> bool {
    a.asset_sha256 == b.asset_sha256 && a.path == b.path && a.row == b.row
}

fn place(l: &SourceLocator) -> String {
    format!(
        "{}{}{}",
        l.asset_sha256,
        l.path.as_ref().map(|p| format!(" {p}")).unwrap_or_default(),
        l.row.map(|r| format!(" row {r}")).unwrap_or_default()
    )
}

/// The cell of a conditions table a `ValueSource::ConditionsCell` names: the one row of
/// `investigation_id` with `run_id` (or, without one, whose `key` cell sits at `locator`), its `key`
/// cell, and for a conflicting cell the candidate at `locator`. Only a measured value is a number.
pub(crate) fn conditions_cell(
    table: &ConditionsTable,
    investigation_id: &str,
    run_id: Option<&RunId>,
    key: ConditionKey,
    locator: Option<&SourceLocator>,
    reading: CellReading,
) -> Result<Cell, Problem> {
    fn cell_in(r: &ConditionsRow, key: ConditionKey) -> Option<&ConditionCell> {
        r.cells.iter().find(|c| c.key() == key)
    }
    let cell_of = |r| cell_in(r, key);
    let at = |c: &ConditionCell| -> Vec<SourceLocator> {
        match c {
            ConditionCell::Value { value, .. } => value.locator.iter().cloned().collect(),
            ConditionCell::Conflicting { candidates, .. } => candidates
                .iter()
                .filter_map(|m| m.locator.clone())
                .collect(),
            _ => vec![],
        }
    };
    let rows: Vec<&ConditionsRow> = table
        .rows
        .iter()
        .filter(|r| r.investigation_id == investigation_id)
        .filter(|r| match run_id {
            Some(id) => r.run.run_id.as_ref() == Some(id),
            None => {
                r.run.run_id.is_none()
                    && cell_of(r).is_some_and(|c| {
                        at(c)
                            .iter()
                            .any(|l| locator.is_some_and(|w| same_place(l, w)))
                    })
            }
        })
        .collect();
    let who = match run_id {
        Some(id) => format!("run {id} of {investigation_id}"),
        None => format!(
            "the run-less row of {investigation_id} at {}",
            locator.map(place).unwrap_or_default()
        ),
    };
    let found = match rows.as_slice() {
        [one] => *one,
        [] => return Err(row(format!("no conditions row for {who}"))),
        many => {
            return Err(row(format!(
                "{} conditions rows for {who}; name the cell's locator",
                many.len()
            )))
        }
    };
    let Some(cell) = cell_of(found) else {
        return Err(column(format!("{who} has no {key:?} cell")));
    };
    let measured = match cell {
        ConditionCell::Value { value, .. } => {
            if let (Some(want), Some(got)) = (locator, value.locator.as_ref()) {
                if !same_place(want, got) {
                    return Err(row(format!(
                        "the {key:?} cell of {who} sits at {}, not at {}",
                        place(got),
                        place(want)
                    )));
                }
            }
            value
        }
        ConditionCell::Conflicting {
            candidates, reason, ..
        } => {
            let Some(want) = locator else {
                return Err(row(format!(
                    "the {key:?} cell of {who} is conflicting ({reason}); pick a candidate by its locator: {}",
                    candidates
                        .iter()
                        .filter_map(|m| m.locator.as_ref())
                        .map(place)
                        .collect::<Vec<_>>()
                        .join("; ")
                )));
            };
            candidates
                .iter()
                .find(|m| m.locator.as_ref().is_some_and(|l| same_place(l, want)))
                .ok_or_else(|| {
                    row(format!(
                        "no candidate of the conflicting {key:?} cell of {who} sits at {}",
                        place(want)
                    ))
                })?
        }
        ConditionCell::Missing { reason, .. } | ConditionCell::NotApplicable { reason, .. } => {
            return Err(column(format!(
                "the {key:?} cell of {who} holds no value ({reason}); bind a number only to a measured value"
            )))
        }
    };
    let (value, unit, what) = match reading {
        CellReading::Si => (
            measured.value_si,
            measured.unit_si.clone(),
            "value_si + unit_si",
        ),
        CellReading::Raw => (
            match measured.raw_value {
                Some(RawValue::Number(x)) => Some(x),
                _ => None,
            },
            measured.raw_unit.clone(),
            "numeric raw_value + raw_unit",
        ),
    };
    match (value, unit.filter(|u| !u.trim().is_empty())) {
        (Some(value), Some(unit)) => Ok(Cell { value, unit }),
        (Some(_), None) => Err((
            AnswerProblemKind::UnitMismatch,
            format!("the {key:?} cell of {who} states no unit for that number"),
        )),
        (None, _) => Err(column(format!(
            "the {key:?} cell of {who} has no {what} (a categorical or unconverted value)"
        ))),
    }
}

/// `x` rounded to `digits` significant decimal digits.
fn round_sig(x: f64, digits: u8) -> f64 {
    let s = format!("{:.*e}", usize::from(digits.max(1) - 1), x);
    s.parse().unwrap_or(f64::NAN)
}

/// Check a stated value against the cell. `Err` names the problem and what the cell holds.
pub(crate) fn compare(
    stated: f64,
    stated_unit: &str,
    significant_digits: Option<u8>,
    cell: &Cell,
) -> Result<(), Problem> {
    let (su, cu) = (stated_unit.trim(), cell.unit.trim());
    let same = su == cu;
    let mismatch = |cell_in_stated: f64| {
        (
            AnswerProblemKind::ValueMismatch,
            match significant_digits {
                Some(n) => format!(
                    "stated {stated} {su} ({n} significant digits), but the cell holds {} {cu} = {} {su} at {n} digits",
                    cell.value,
                    round_sig(cell_in_stated, n)
                ),
                None => format!(
                    "stated {stated} {su}, but the cell holds {} {cu}{}; state the cell's value (or round it with significant_digits)",
                    cell.value,
                    if same { String::new() } else { format!(" = {cell_in_stated} {su}") }
                ),
            },
        )
    };
    if same {
        let ok = match significant_digits {
            None => stated == cell.value,
            Some(n) => round_sig(cell.value, n) == stated,
        };
        return if ok {
            Ok(())
        } else {
            Err(mismatch(cell.value))
        };
    }
    let (Some(s), Some(c)) = (units::lookup(su), units::lookup(cu)) else {
        let unknown = if units::lookup(su).is_none() { su } else { cu };
        return Err((
            AnswerProblemKind::UnitMismatch,
            format!(
                "unit {unknown:?} is not registered, so {su} and {cu} cannot be compared; state the value in the cell's unit {cu:?}"
            ),
        ));
    };
    if s.dimension != c.dimension || s.symbol_si != c.symbol_si {
        return Err((
            AnswerProblemKind::UnitMismatch,
            format!(
                "stated unit {su:?} measures {}, the cell's {cu:?} measures {}",
                s.dimension, c.dimension
            ),
        ));
    }
    let cell_si = cell.value * c.factor + c.offset;
    let cell_in_stated = (cell_si - s.offset) / s.factor;
    let ok = match significant_digits {
        None => {
            let stated_si = stated * s.factor + s.offset;
            (stated_si - cell_si).abs() <= CONVERSION_REL_TOL * stated_si.abs().max(cell_si.abs())
        }
        Some(n) => round_sig(cell_in_stated, n) == stated,
    };
    if ok {
        Ok(())
    } else {
        Err(mismatch(cell_in_stated))
    }
}

/// The numbers written in `text` (decimal, optional sign and exponent; a digit glued to a letter,
/// as in `O2` or `CO2`, is part of a name, not a number), each with the byte offset after it.
fn numbers_in(text: &str) -> Vec<(f64, &str, usize)> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let glued = i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'.');
        let sign = (b[i] == b'-' || b[i] == b'+') && b.get(i + 1).is_some_and(u8::is_ascii_digit);
        if !(b[i].is_ascii_digit() || sign) || glued {
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_digit() {
            i += 1;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
        }
        if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
            let mut j = i + 1;
            if j < b.len() && (b[j] == b'-' || b[j] == b'+') {
                j += 1;
            }
            if j < b.len() && b[j].is_ascii_digit() {
                while j < b.len() && b[j].is_ascii_digit() {
                    j += 1;
                }
                i = j;
            }
        }
        let token = &text[start..i];
        if let Ok(x) = token.parse::<f64>() {
            out.push((x, token, i));
        }
    }
    out
}

/// `text` with the spellings of one number folded together: the Unicode minus `−` becomes `-`, and
/// a thousands separator (`,`, thin or narrow no-break space) between a 1–3 digit group and an
/// exact 3-digit group is dropped (`101,325` → `101325`). A list such as `10,20` is left alone.
fn fold_number_spellings(text: &str) -> String {
    let c: Vec<char> = text
        .chars()
        .map(|ch| if ch == '−' { '-' } else { ch })
        .collect();
    let mut out = String::with_capacity(text.len());
    // Length of the digit run just written, and whether it continues a thousands-grouped number.
    let (mut run, mut grouped) = (0usize, false);
    for i in 0..c.len() {
        let ch = c[i];
        if ch.is_ascii_digit() {
            run += 1;
            out.push(ch);
            continue;
        }
        let separator = matches!(ch, ',' | '\u{2009}' | '\u{202F}');
        let lead_ok = (1..=3).contains(&run) || (grouped && run == 3);
        // The leading group is an integer part, not the fraction of a decimal (`0.5,100`).
        let starts_run = i == run || c[i - run - 1] != '.';
        let group_follows = (1..=3).all(|k| c.get(i + k).is_some_and(char::is_ascii_digit))
            && !c.get(i + 4).is_some_and(char::is_ascii_digit);
        if separator && run > 0 && lead_ok && (grouped || starts_run) && group_follows {
            grouped = true;
            run = 0;
            continue;
        }
        grouped = false;
        run = 0;
        out.push(ch);
    }
    out
}

/// Byte length of a number at the very start of `s` (optional sign, digits, fraction, exponent).
fn number_prefix(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = usize::from(matches!(b.first(), Some(b'-' | b'+')));
    let digits = i;
    while b.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    if i == digits {
        return None;
    }
    if b.get(i) == Some(&b'.') && b.get(i + 1).is_some_and(u8::is_ascii_digit) {
        i += 1;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
    }
    Some(i)
}

/// What follows a number once the partner of a range or an uncertainty is passed over: in
/// `4.3–10.2 psi`, `3.05±0.1 mm` or `(3.05 ± 0.10) mm` the unit after the partner is the unit of
/// both numbers.
fn after_partners(mut rest: &str) -> &str {
    loop {
        let t = rest.trim_start();
        let joined = ["±", "+/-", "–", "—", "-", "~", "to "]
            .iter()
            .find_map(|sep| t.strip_prefix(sep));
        let Some(after) = joined else {
            let t = t.strip_prefix(')').unwrap_or(t);
            return t.trim_start();
        };
        let after = after.trim_start();
        match number_prefix(after) {
            Some(n) => rest = &after[n..],
            None => return rest.trim_start(),
        }
    }
}

/// The unit token at the start of `s`: up to whitespace or a closing punctuation mark, without a
/// sentence-ending dot.
fn unit_token(s: &str) -> &str {
    let end = s
        .find(|c: char| c.is_whitespace() || ",;:)]}|".contains(c))
        .unwrap_or(s.len());
    s[..end].trim_end_matches('.')
}

/// A spelling the unit registry knows, directly or through an alias.
fn is_unit(token: &str) -> bool {
    !token.is_empty()
        && (units::lookup(token).is_some() || units::lookup(units::alias(token)).is_some())
}

/// `rest` (what follows a number, partners passed) begins with `unit`, as the same token or a
/// spelling of the same unit (`psia` for `psi`).
fn begins_with_unit(rest: &str, unit: &str) -> bool {
    let token = unit_token(rest);
    if !token.is_empty() && units::same_spelling(token, unit) {
        return true;
    }
    rest.strip_prefix(unit)
        .is_some_and(|after| !after.chars().next().is_some_and(|c| c.is_alphanumeric()))
}

/// The unit a header names: `(psi)`, `[mm]`, `, s` at the end of a cell, or a cell that is a unit.
fn header_unit(cell: &str) -> Option<&str> {
    let cell = cell.trim();
    for (open, close) in [('(', ')'), ('[', ']')] {
        if let (Some(a), Some(b)) = (cell.rfind(open), cell.rfind(close)) {
            if a < b {
                let inner = cell[a + open.len_utf8()..b].trim();
                if is_unit(inner) {
                    return Some(inner);
                }
            }
        }
    }
    if let Some((_, tail)) = cell.rsplit_once(',') {
        if is_unit(tail.trim()) {
            return Some(tail.trim());
        }
    }
    is_unit(cell).then_some(cell)
}

/// For a number at `start` that no unit follows: the unit a table states for it. On its own line,
/// the nearest preceding bracketed unit (`Pressure (psi)  4.3  5.1`); in a `|`- or tab-separated
/// table, the header cell of its column (the first row of the block of rows around it).
fn table_unit(text: &str, start: usize) -> Option<&str> {
    let line_start = text[..start].rfind('\n').map_or(0, |i| i + 1);
    let line_end = text[start..].find('\n').map_or(text.len(), |i| start + i);
    let before = &text[line_start..start];
    for sep in ['|', '\t'] {
        if !text[line_start..line_end].contains(sep) {
            continue;
        }
        let column = before.matches(sep).count();
        // The block of rows that share the separator, ending at this line; its first row is the header.
        let mut header = line_start;
        while header > 0 {
            let prev_start = text[..header - 1].rfind('\n').map_or(0, |i| i + 1);
            if !text[prev_start..header - 1].contains(sep) {
                break;
            }
            header = prev_start;
        }
        if header == line_start {
            break;
        }
        let header_line = text[header..].lines().next().unwrap_or("");
        return header_line.split(sep).nth(column).and_then(header_unit);
    }
    // Nearest preceding bracketed unit on the same line.
    let mut cut = before.len();
    while let Some(close) = before[..cut].rfind([')', ']']) {
        let open_ch = if before[close..].starts_with(')') {
            '('
        } else {
            '['
        };
        let Some(open) = before[..close].rfind(open_ch) else {
            break;
        };
        let inner = before[open + 1..close].trim();
        if is_unit(inner) {
            return Some(inner);
        }
        cut = open;
    }
    None
}

/// Text-level grounding: `value`, written as a number and followed by `unit` (spaces allowed, the
/// unit not running into a longer word), occurs in the quote a retrieval tool returned. It shows
/// the number is in the passage, not what it measures. The passage's own conventions count: a
/// range or `±` partner before the unit (`4.3–10.2 psi`), thousands separators (`101,325 Pa`), the
/// Unicode minus, unit spellings of one unit (`psia` for `psi`), and, for a number with no unit
/// after it, the unit its table header states.
pub(crate) fn find_in_quote(quote: &str, value: f64, unit: &str) -> Result<(), Problem> {
    let unit = unit.trim();
    let text = fold_number_spellings(quote);
    let found = numbers_in(&text);
    let same: Vec<(usize, usize)> = found
        .iter()
        .filter(|(x, _, _)| *x == value)
        .map(|(_, t, end)| (*end - t.len(), *end))
        .collect();
    if same.is_empty() {
        let written: Vec<&str> = found.iter().map(|(_, t, _)| *t).collect();
        return Err((
            AnswerProblemKind::ValueMismatch,
            format!(
                "the quote does not contain {value} (numbers in it: {}); quote the number as the passage writes it",
                if written.is_empty() { "none".into() } else { written.join(", ") }
            ),
        ));
    }
    let grounded = |(start, end): (usize, usize)| {
        let direct = text[end..].trim_start();
        if begins_with_unit(direct, unit) {
            return true;
        }
        let rest = after_partners(&text[end..]);
        if begins_with_unit(rest, unit) {
            return true;
        }
        // A number followed by a unit of its own is not re-labelled by a header.
        if is_unit(unit_token(direct)) || is_unit(unit_token(rest)) {
            return false;
        }
        table_unit(&text, start).is_some_and(|h| units::same_spelling(h, unit))
    };
    // A dimensionless number is often written without a unit.
    if unit == "1" || same.iter().any(|&s| grounded(s)) {
        Ok(())
    } else {
        Err((
            AnswerProblemKind::UnitMismatch,
            format!(
                "{value} occurs in the quote, but not followed by {unit:?}; state the unit the passage writes"
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(v: f64, u: &str) -> Cell {
        Cell {
            value: v,
            unit: u.into(),
        }
    }

    /// The comparison is exact in the same unit, tolerant only of conversion error across units,
    /// and rounding is explicit.
    #[test]
    fn comparison_has_no_fudge_factor() {
        assert!(compare(3.2147, "s", None, &cell(3.2147, "s")).is_ok());
        assert!(compare(3.2148, "s", None, &cell(3.2147, "s")).is_err());
        assert!(compare(3.21, "s", Some(3), &cell(3.2147, "s")).is_ok());
        assert!(compare(3.2, "s", Some(3), &cell(3.2147, "s")).is_err());
        assert!(compare(1.5, "mm", None, &cell(0.0015, "m")).is_ok());
        assert!(compare(1.5000001, "mm", None, &cell(0.0015, "m")).is_err());
        assert!(compare(101.325, "kPa", None, &cell(1.0, "atm")).is_ok());
        assert!(compare(25.0, "C", None, &cell(298.15, "K")).is_ok());
        let (k, _) = compare(1.5, "s", None, &cell(0.0015, "m")).unwrap_err();
        assert_eq!(k, AnswerProblemKind::UnitMismatch);
        let (k, _) = compare(1.5, "furlong", None, &cell(0.0015, "m")).unwrap_err();
        assert_eq!(k, AnswerProblemKind::UnitMismatch);
    }

    /// A quoted number is found as written, with its unit, never inside a name.
    #[test]
    fn a_quoted_number_is_found_as_written() {
        let q = "In 21% O2 at 1 atm the droplet (d0 = 3.05 mm) burned for 12.4 s.";
        assert!(find_in_quote(q, 3.05, "mm").is_ok());
        assert!(find_in_quote(q, 12.4, "s").is_ok());
        assert!(find_in_quote(q, 21.0, "%").is_ok());
        let (k, _) = find_in_quote(q, 3.5, "mm").unwrap_err();
        assert_eq!(k, AnswerProblemKind::ValueMismatch);
        let (k, _) = find_in_quote(q, 2.0, "1").unwrap_err();
        assert_eq!(k, AnswerProblemKind::ValueMismatch, "the 2 of O2 is a name");
        let (k, _) = find_in_quote(q, 12.4, "ms").unwrap_err();
        assert_eq!(k, AnswerProblemKind::UnitMismatch);
        let (k, _) = find_in_quote(q, 1.0, "a").unwrap_err();
        assert_eq!(
            k,
            AnswerProblemKind::UnitMismatch,
            "a unit does not run into a word"
        );
    }

    /// The passage's own number conventions ground a number; a different number never does.
    #[test]
    fn a_quoted_number_follows_the_passage_conventions() {
        // Range and uncertainty partners carry the unit for both numbers.
        let q = "Chamber pressure was held at 4.3–10.2 psi; d0 = 3.05±0.1 mm, t = (12.4 ± 0.3) s.";
        assert!(find_in_quote(q, 4.3, "psi").is_ok());
        assert!(find_in_quote(q, 10.2, "psi").is_ok());
        assert!(find_in_quote(q, 3.05, "mm").is_ok());
        assert!(find_in_quote(q, 12.4, "s").is_ok());
        assert!(find_in_quote("from 4.3 to 10.2 psi", 4.3, "psi").is_ok());
        // Thousands separators.
        assert!(find_in_quote("ambient 101,325 Pa", 101325.0, "Pa").is_ok());
        assert!(find_in_quote("ambient 101\u{202F}325 Pa", 101325.0, "Pa").is_ok());
        assert!(find_in_quote("1,013,250 Pa", 1013250.0, "Pa").is_ok());
        let (k, _) = find_in_quote("tests 10,20 mm", 1020.0, "mm").unwrap_err();
        assert_eq!(k, AnswerProblemKind::ValueMismatch, "a list is not grouped");
        // Unicode minus.
        assert!(find_in_quote("cooled to −40 °C", -40.0, "C").is_ok());
        // Spellings of one unit.
        assert!(find_in_quote("at 14.7 psia", 14.7, "psi").is_ok());
        assert!(find_in_quote("at 2.0 psig", 2.0, "psi").is_ok());
        // Table passages: the unit is in the column header or a header token on the line.
        let table = "| Run | d0 (mm) | Burn time [s] |\n|---|---|---|\n| 7 | 3.05 | 12.4 |";
        assert!(find_in_quote(table, 3.05, "mm").is_ok());
        assert!(find_in_quote(table, 12.4, "s").is_ok());
        let (k, _) = find_in_quote(table, 3.05, "s").unwrap_err();
        assert_eq!(
            k,
            AnswerProblemKind::UnitMismatch,
            "the column, not any header"
        );
        let tsv = "Run\tPressure, psi\n7\t4.3";
        assert!(find_in_quote(tsv, 4.3, "psi").is_ok());
        assert!(find_in_quote("Pressure (psi)  4.3  5.1  6.0", 5.1, "psi").is_ok());
    }

    /// A conditions row's `nearness` cells resolve their unit from the field name; a name that
    /// states no unit still needs a stated one.
    #[test]
    fn nearness_cells_take_their_unit_from_the_field_name() {
        let row = serde_json::json!({
            "run_id": "FLEX-001",
            "unit": "mm",
            "nearness": {
                "inside": false,
                "oxygen_gap": 0.045,
                "pressure_gap_pa": 46_000.0,
                "distance": 0.5,
            },
            "o2_fraction": 0.21,
            "burn_time_s": 12.4,
            "count": 3,
        });
        let cell = |p: &str| read_cell(&row, &[], p);
        let c = cell("/nearness/pressure_gap_pa").unwrap();
        assert_eq!((c.value, c.unit.as_str()), (46_000.0, "Pa"));
        assert!(compare(46.0, "kPa", None, &c).is_ok());
        assert_eq!(cell("/nearness/oxygen_gap").unwrap().unit, "1");
        assert_eq!(cell("/nearness/distance").unwrap().unit, "1");
        assert_eq!(cell("/o2_fraction").unwrap().unit, "1");
        assert_eq!(cell("/burn_time_s").unwrap().unit, "s");
        // No unit in the name: the stated `unit` applies, as before.
        assert_eq!(cell("/count").unwrap().unit, "mm");
        let bare = serde_json::json!({"nearness": {"inside": true, "rank": 1}});
        let (k, _) = read_cell(&bare, &[], "/nearness/rank").unwrap_err();
        assert_eq!(k, AnswerProblemKind::UnitMismatch);
    }

    /// The relaxations do not ground a different number or re-label a number's own unit.
    #[test]
    fn conventions_do_not_ground_a_different_number() {
        let q = "Pressure (psi) 4.3–10.2; burn time 12.4 s";
        let (k, _) = find_in_quote(q, 7.0, "psi").unwrap_err();
        assert_eq!(
            k,
            AnswerProblemKind::ValueMismatch,
            "inside a range is not in it"
        );
        let (k, _) = find_in_quote(q, 4.4, "psi").unwrap_err();
        assert_eq!(k, AnswerProblemKind::ValueMismatch);
        let (k, _) = find_in_quote(q, 12.4, "psi").unwrap_err();
        assert_eq!(
            k,
            AnswerProblemKind::UnitMismatch,
            "a number's own unit beats the header"
        );
        let (k, _) = find_in_quote("at 14.7 psia", 14.7, "kPa").unwrap_err();
        assert_eq!(k, AnswerProblemKind::UnitMismatch);
        let (k, _) = find_in_quote("−40 °C", 40.0, "C").unwrap_err();
        assert_eq!(k, AnswerProblemKind::ValueMismatch, "the sign is kept");
    }
}

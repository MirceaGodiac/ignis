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

/// Text-level grounding: `value`, written as a number and followed by `unit` (spaces allowed, the
/// unit not running into a longer word), occurs in the quote a retrieval tool returned. It shows
/// the number is in the passage, not what it measures.
pub(crate) fn find_in_quote(quote: &str, value: f64, unit: &str) -> Result<(), Problem> {
    let unit = unit.trim();
    let found = numbers_in(quote);
    let same: Vec<usize> = found
        .iter()
        .filter(|(x, _, _)| *x == value)
        .map(|(_, _, end)| *end)
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
    let followed = |end: usize| {
        let rest = quote[end..].trim_start();
        rest.strip_prefix(unit).is_some_and(|after| {
            unit == "1" || !after.chars().next().is_some_and(|c| c.is_alphanumeric())
        })
    };
    // A dimensionless number is often written without a unit.
    if unit == "1" || same.iter().any(|&e| followed(e)) {
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
}

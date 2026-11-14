//! Byte decoding, CSV reading and string helpers with the exact semantics of the Python pipeline
//! they replace (`bytes.decode`, `csv.reader` excel dialect, `str.strip`, `repr`), so that row
//! numbers, cell values and cited text are identical to the records produced before the port.

/// Python `str.isspace` (Unicode White_Space plus the ASCII information separators 0x1c..0x1f).
pub fn py_isspace(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python `str.strip()`.
pub fn strip(s: &str) -> &str {
    s.trim_matches(py_isspace)
}

/// Python `str.rstrip()`.
pub fn rstrip(s: &str) -> &str {
    s.trim_end_matches(py_isspace)
}

/// Python `str.lower()` (full Unicode lowercase).
pub fn lower(s: &str) -> String {
    s.to_lowercase()
}

/// Python `repr(str)`.
pub fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32))
            }
            c if (0x80..0xa0).contains(&(c as u32)) => {
                out.push_str(&format!("\\x{:02x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// Python `repr(list[str])`.
pub fn py_repr_list(items: &[&str]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|s| py_repr(s))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Python `repr(float)` for the finite values that reach messages (shortest round trip).
pub fn py_float(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == x.trunc() && x.abs() < 1e16 {
        return format!("{x:.1}");
    }
    let s = format!("{x:e}");
    let (mant, exp) = s.split_once('e').unwrap();
    let e: i32 = exp.parse().unwrap();
    if (-4..16).contains(&e) {
        format!("{x}")
    } else {
        format!("{mant}e{}{:02}", if e < 0 { '-' } else { '+' }, e.abs())
    }
}

/// Python `bytes.decode("cp1252", errors="replace")` (undefined bytes become U+FFFD, unlike WHATWG).
pub fn decode_cp1252(raw: &[u8]) -> String {
    const HI: [u32; 32] = [
        0x20ac, 0xfffd, 0x201a, 0x0192, 0x201e, 0x2026, 0x2020, 0x2021, 0x02c6, 0x2030, 0x0160,
        0x2039, 0x0152, 0xfffd, 0x017d, 0xfffd, 0xfffd, 0x2018, 0x2019, 0x201c, 0x201d, 0x2022,
        0x2013, 0x2014, 0x02dc, 0x2122, 0x0161, 0x203a, 0x0153, 0xfffd, 0x017e, 0x0178,
    ];
    raw.iter()
        .map(|&b| {
            if (0x80..0xa0).contains(&b) {
                char::from_u32(HI[(b - 0x80) as usize]).unwrap()
            } else {
                b as char
            }
        })
        .collect()
}

/// FLEX decoding: UTF-8 when valid, else cp1252 with replacement.
pub fn decode_utf8_or_cp1252(raw: &[u8]) -> String {
    match std::str::from_utf8(raw) {
        Ok(s) => s.to_string(),
        Err(_) => decode_cp1252(raw),
    }
}

/// Python `bytes.decode("utf-8-sig", errors="replace")`.
pub fn decode_utf8_sig_lossy(raw: &[u8]) -> String {
    let raw = raw.strip_prefix(b"\xef\xbb\xbf").unwrap_or(raw);
    String::from_utf8_lossy(raw).into_owned()
}

/// `text.replace("\r\n", "\n").replace("\r", "\n")`
pub fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

/// Python `csv.reader(io.StringIO(text))` with the default excel dialect (strict=False).
/// Blank lines yield empty records, exactly like Python, so record indices match.
pub fn py_csv(text: &str) -> Vec<Vec<String>> {
    py_csv_delim(text, ',')
}

/// `csv.DictReader(io.StringIO(text), delimiter=...)`: header row + one map per non-empty record.
pub fn py_dict_reader(text: &str, delim: char) -> Vec<indexmap::IndexMap<String, String>> {
    let rows = py_csv_delim(text, delim);
    let Some(h) = rows.first() else { return vec![] };
    rows[1..]
        .iter()
        .filter(|r| !r.is_empty())
        .map(|r| {
            h.iter()
                .enumerate()
                .map(|(j, k)| (k.clone(), r.get(j).cloned().unwrap_or_default()))
                .collect()
        })
        .collect()
}

pub fn py_csv_delim(text: &str, delim: char) -> Vec<Vec<String>> {
    #[derive(PartialEq, Clone, Copy)]
    enum S {
        StartRecord,
        StartField,
        InField,
        InQuoted,
        QuoteInQuoted,
        EatCrnl,
    }
    let mut out = Vec::new();
    let mut fields: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut state = S::StartRecord;
    // StringIO iterates on '\n' only; each line keeps its terminator.
    let mut lines: Vec<&str> = text.split_inclusive('\n').collect();
    if lines.is_empty() {
        lines.push("");
    }
    for line in lines {
        if line.is_empty() {
            continue;
        }
        for c in line.chars().map(Some).chain(std::iter::once(None)) {
            // None = end of line (EOL sentinel)
            match state {
                S::StartRecord | S::StartField => {
                    if state == S::StartRecord {
                        match c {
                            None => {
                                // empty line: record [] (handled below when state is StartRecord)
                                continue;
                            }
                            Some('\n') | Some('\r') => {
                                state = S::EatCrnl;
                                continue;
                            }
                            _ => state = S::StartField,
                        }
                    }
                    match c {
                        None | Some('\n') | Some('\r') => {
                            fields.push(std::mem::take(&mut field));
                            state = if c.is_none() {
                                S::StartRecord
                            } else {
                                S::EatCrnl
                            };
                        }
                        Some('"') => state = S::InQuoted,
                        Some(d) if d == delim => fields.push(std::mem::take(&mut field)),
                        Some(ch) => {
                            field.push(ch);
                            state = S::InField;
                        }
                    }
                }
                S::InField => match c {
                    None | Some('\n') | Some('\r') => {
                        fields.push(std::mem::take(&mut field));
                        state = if c.is_none() {
                            S::StartRecord
                        } else {
                            S::EatCrnl
                        };
                    }
                    Some(d) if d == delim => {
                        fields.push(std::mem::take(&mut field));
                        state = S::StartField;
                    }
                    Some(ch) => field.push(ch),
                },
                S::InQuoted => match c {
                    None => {}
                    Some('"') => state = S::QuoteInQuoted,
                    Some(ch) => field.push(ch),
                },
                S::QuoteInQuoted => match c {
                    Some('"') => {
                        field.push('"');
                        state = S::InQuoted;
                    }
                    Some(d) if d == delim => {
                        fields.push(std::mem::take(&mut field));
                        state = S::StartField;
                    }
                    None | Some('\n') | Some('\r') => {
                        fields.push(std::mem::take(&mut field));
                        state = if c.is_none() {
                            S::StartRecord
                        } else {
                            S::EatCrnl
                        };
                    }
                    Some(ch) => {
                        field.push(ch);
                        state = S::InField;
                    }
                },
                S::EatCrnl => match c {
                    Some('\n') | Some('\r') => {}
                    None => state = S::StartRecord,
                    Some(ch) => {
                        // Python raises "new-line character seen in unquoted field"; keep going as a new field.
                        field.push(ch);
                        state = S::InField;
                    }
                },
            }
        }
        if state == S::StartRecord {
            out.push(std::mem::take(&mut fields));
        }
    }
    if state != S::StartRecord && (!fields.is_empty() || !field.is_empty()) {
        fields.push(std::mem::take(&mut field));
        out.push(fields);
    }
    out
}

/// One table row: 1-based record number (header = 1) and the header→cell map (header order kept;
/// a repeated header keeps its first position and the last value, like a Python dict).
#[derive(Debug, Clone)]
pub struct Row {
    pub row: i64,
    pub cells: indexmap::IndexMap<String, String>,
}

impl Row {
    pub fn get(&self, k: &str) -> &str {
        self.cells
            .get(k)
            .map(|s| s.as_str())
            .unwrap_or_else(|| panic!("column {k:?} missing"))
    }
    pub fn opt(&self, k: &str) -> &str {
        self.cells.get(k).map(|s| s.as_str()).unwrap_or("")
    }
}

/// `[{"_row": i, **{h: r[j] ...}} for i, r in enumerate(rows[1:], start=2) if any(c.strip() for c in r)]`
pub fn table_rows(text: &str, strip_cells: bool) -> Vec<Row> {
    let rows = py_csv(text);
    let Some(first) = rows.first() else {
        return vec![];
    };
    let header: Vec<String> = first
        .iter()
        .map(|h| {
            if strip_cells {
                strip(h).to_string()
            } else {
                h.clone()
            }
        })
        .collect();
    let mut out = Vec::new();
    for (i, r) in rows.iter().enumerate().skip(1) {
        if !r.iter().any(|c| !strip(c).is_empty()) {
            continue;
        }
        let mut cells = indexmap::IndexMap::new();
        for (j, h) in header.iter().enumerate() {
            let v = r
                .get(j)
                .map(|c| {
                    if strip_cells {
                        strip(c).to_string()
                    } else {
                        c.clone()
                    }
                })
                .unwrap_or_default();
            cells.insert(h.clone(), v);
        }
        out.push(Row {
            row: i as i64 + 1,
            cells,
        });
    }
    out
}

/// Python `float(s)` for an already-stripped token (inf/nan spellings included).
pub fn py_float_parse(s: &str) -> Option<f64> {
    let t = s.trim_matches(py_isspace);
    if t.is_empty() || t.contains('_') {
        return None;
    }
    t.parse::<f64>().ok()
}

/// Index of the first char of `needle` in `hay`, counted in Unicode scalar values (Python indices).
pub fn char_index(hay: &str, needle: &str) -> Option<usize> {
    hay.find(needle).map(|b| hay[..b].chars().count())
}

/// Python `json.dumps(v, ensure_ascii=False)` with the default separators (", ", ": "); object keys are
/// written in the map's order (sort them first for `sort_keys=True`). Floats use Python `repr`.
pub fn py_dumps(v: &serde_json::Value) -> String {
    let mut out = String::new();
    py_dumps_into(v, &mut out, ", ", ": ", None, 0);
    out
}

/// Python `json.dumps(v, indent=2, ensure_ascii=False)`.
pub fn py_dumps_indent(v: &serde_json::Value) -> String {
    let mut out = String::new();
    py_dumps_into(v, &mut out, ",", ": ", Some(2), 0);
    out
}

fn py_dumps_into(
    v: &serde_json::Value,
    out: &mut String,
    item_sep: &str,
    key_sep: &str,
    indent: Option<usize>,
    level: usize,
) {
    use serde_json::Value;
    let nl = |out: &mut String, lvl: usize| {
        if let Some(n) = indent {
            out.push('\n');
            out.push_str(&" ".repeat(n * lvl));
        }
    };
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                out.push_str(&i.to_string());
            } else if let Some(u) = n.as_u64() {
                out.push_str(&u.to_string());
            } else {
                out.push_str(&py_float(n.as_f64().unwrap()));
            }
        }
        Value::String(s) => py_json_str(s, out),
        Value::Array(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push_str(item_sep);
                }
                nl(out, level + 1);
                py_dumps_into(x, out, item_sep, key_sep, indent, level + 1);
            }
            nl(out, level);
            out.push(']');
        }
        Value::Object(m) => {
            if m.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, x)) in m.iter().enumerate() {
                if i > 0 {
                    out.push_str(item_sep);
                }
                nl(out, level + 1);
                py_json_str(k, out);
                out.push_str(key_sep);
                py_dumps_into(x, out, item_sep, key_sep, indent, level + 1);
            }
            nl(out, level);
            out.push('}');
        }
    }
}

/// `json.dumps(dict, indent=2, ensure_ascii=False)` for an insertion-ordered top-level dict.
pub fn py_dumps_ordered_indent(pairs: &[(&str, serde_json::Value)]) -> String {
    let mut out = String::from("{");
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("\n  ");
        py_json_str(k, &mut out);
        out.push_str(": ");
        py_dumps_into(v, &mut out, ",", ": ", Some(2), 1);
    }
    out.push_str("\n}");
    out
}

fn py_json_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

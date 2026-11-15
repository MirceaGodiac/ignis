//! The literal edit language: the text an author model writes, compiled to the contract's
//! [`LiteralOp`]s. It can only name a tensor, select elements and state numbers.
//!
//! ```text
//! # comment to end of line
//! rate.b0 = -0.598                          # scalar tensor (shape [])
//! res.W1[0, 3] = 1.5                        # one element          -> set
//! res.W1[2, :] = 0.0                        # constant over a box  -> fill_box
//! res.b1[0:4] = [0.5, -0.5, 0.25, 0.0]      # literal list         -> set_box (row-major)
//! res.W2[0:2, 0:3] = [[1, 0, -1],
//!                     [0, 2,  0]]           # nested list, may span lines
//! ```
//!
//! There are no expressions, names on the right-hand side, arithmetic, ranges with steps, negative
//! indices, repetition operators, includes or directives: a value is a decimal literal the author
//! typed. Anything else is a parse error for the whole file; nothing is skipped or repaired.
//! Selectors follow NumPy for the supported subset: an integer picks one index (and removes that
//! axis from the shape a list must have), `a:b` is half-open, `:` is the whole axis.

use ignis_core::contracts::{Finite, LiteralOp, ModelSpec};

/// A compile error with the 1-based source line it was found on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "edits line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for EditError {}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Name(String),
    Num(f64),
    Int(u32),
    LBracket,
    RBracket,
    Comma,
    Colon,
    Equals,
    Newline,
}

struct Lexed {
    tok: Tok,
    line: usize,
}

fn err<T>(line: usize, message: impl Into<String>) -> Result<T, EditError> {
    Err(EditError {
        line,
        message: message.into(),
    })
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.'
}

/// A decimal literal: `[+-] digits [. digits] [e [+-] digits]`. Hex, `inf`, `nan`, underscores and
/// a bare leading or trailing dot are refused, so every accepted token reads as one number.
fn lex_number(s: &[char], i: &mut usize, line: usize) -> Result<Tok, EditError> {
    let start = *i;
    if matches!(s.get(*i), Some('+' | '-')) {
        *i += 1;
    }
    let digits = |i: &mut usize| {
        let a = *i;
        while s.get(*i).is_some_and(|c| c.is_ascii_digit()) {
            *i += 1;
        }
        *i > a
    };
    if !digits(i) {
        return err(line, "a number needs a digit before any '.' or exponent");
    }
    let mut integral = true;
    if s.get(*i) == Some(&'.') {
        integral = false;
        *i += 1;
        if !digits(i) {
            return err(line, "a '.' in a number must be followed by a digit");
        }
    }
    if matches!(s.get(*i), Some('e' | 'E')) {
        integral = false;
        *i += 1;
        if matches!(s.get(*i), Some('+' | '-')) {
            *i += 1;
        }
        if !digits(i) {
            return err(line, "an exponent needs digits");
        }
    }
    if s.get(*i).is_some_and(|c| is_name_char(*c)) {
        return err(
            line,
            "a number is followed by a letter, '_' or '.': only decimal literals are accepted",
        );
    }
    let text: String = s[start..*i].iter().collect();
    if integral && !text.starts_with(['+', '-']) {
        if let Ok(n) = text.parse::<u32>() {
            return Ok(Tok::Int(n));
        }
    }
    match text.parse::<f64>() {
        Ok(v) if v.is_finite() => Ok(Tok::Num(v)),
        _ => err(line, format!("{text:?} is not a finite number")),
    }
}

fn lex(src: &str) -> Result<Vec<Lexed>, EditError> {
    let s: Vec<char> = src.chars().collect();
    let (mut i, mut line) = (0usize, 1usize);
    let mut out = Vec::new();
    while i < s.len() {
        let c = s[i];
        let single = match c {
            '[' => Some(Tok::LBracket),
            ']' => Some(Tok::RBracket),
            ',' => Some(Tok::Comma),
            ':' => Some(Tok::Colon),
            '=' => Some(Tok::Equals),
            _ => None,
        };
        if let Some(tok) = single {
            out.push(Lexed { tok, line });
            i += 1;
        } else if c == '\n' {
            out.push(Lexed {
                tok: Tok::Newline,
                line,
            });
            line += 1;
            i += 1;
        } else if c == '#' {
            while i < s.len() && s[i] != '\n' {
                i += 1;
            }
        } else if c == ' ' || c == '\t' || c == '\r' {
            i += 1;
        } else if is_name_start(c) {
            let a = i;
            while i < s.len() && is_name_char(s[i]) {
                i += 1;
            }
            out.push(Lexed {
                tok: Tok::Name(s[a..i].iter().collect()),
                line,
            });
        } else if c.is_ascii_digit() || c == '+' || c == '-' {
            let tok = lex_number(&s, &mut i, line)?;
            out.push(Lexed { tok, line });
        } else {
            return err(line, format!("unexpected character {c:?}"));
        }
    }
    out.push(Lexed {
        tok: Tok::Newline,
        line,
    });
    Ok(out)
}

/// One axis selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sel {
    Index(u32),
    /// Half-open `[start, end)`; `None` bounds are the axis ends.
    Slice(Option<u32>, Option<u32>),
}

/// A right-hand side: one literal or a rectangular (possibly nested) list of literals.
#[derive(Debug, Clone, PartialEq)]
enum Rhs {
    Scalar(f64),
    /// Nesting shape and the values in row-major order.
    List(Vec<u32>, Vec<f64>),
}

struct Parser<'a> {
    toks: &'a [Lexed],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }
    fn line(&self) -> usize {
        self.toks[self.pos].line
    }
    fn bump(&mut self) -> &Tok {
        let t = &self.toks[self.pos].tok;
        self.pos += 1;
        t
    }
    /// Newlines are layout inside brackets and statement ends outside them.
    fn skip_newlines(&mut self) {
        while self.pos + 1 < self.toks.len() && *self.peek() == Tok::Newline {
            self.pos += 1;
        }
    }

    fn selectors(&mut self) -> Result<Vec<Sel>, EditError> {
        let mut out = Vec::new();
        loop {
            let line = self.line();
            let lo = match self.peek() {
                Tok::Int(n) => {
                    let n = *n;
                    self.pos += 1;
                    Some(n)
                }
                _ => None,
            };
            if *self.peek() == Tok::Colon {
                self.pos += 1;
                let hi = match self.peek() {
                    Tok::Int(n) => {
                        let n = *n;
                        self.pos += 1;
                        Some(n)
                    }
                    _ => None,
                };
                out.push(Sel::Slice(lo, hi));
            } else if let Some(i) = lo {
                out.push(Sel::Index(i));
            } else {
                return err(line, "a selector is a non-negative integer or a slice a:b");
            }
            match self.bump() {
                Tok::Comma => continue,
                Tok::RBracket => return Ok(out),
                _ => return err(line, "expected ',' or ']' in a selector"),
            }
        }
    }

    fn value(&mut self) -> Result<Rhs, EditError> {
        let line = self.line();
        match self.bump().clone() {
            Tok::Num(v) => Ok(Rhs::Scalar(v)),
            Tok::Int(n) => Ok(Rhs::Scalar(n as f64)),
            Tok::LBracket => {
                let mut items = Vec::new();
                loop {
                    self.skip_newlines();
                    if *self.peek() == Tok::RBracket {
                        self.pos += 1;
                        break;
                    }
                    items.push(self.value()?);
                    self.skip_newlines();
                    match self.bump() {
                        Tok::Comma => continue,
                        Tok::RBracket => break,
                        _ => return err(line, "expected ',' or ']' in a list"),
                    }
                }
                let Some(first) = items.first() else {
                    return err(line, "an empty list assigns nothing");
                };
                let inner = match first {
                    Rhs::Scalar(_) => vec![],
                    Rhs::List(s, _) => s.clone(),
                };
                let mut flat = Vec::new();
                for it in &items {
                    match it {
                        Rhs::Scalar(v) if inner.is_empty() => flat.push(*v),
                        Rhs::List(s, v) if *s == inner => flat.extend_from_slice(v),
                        _ => return err(line, "a nested list must be rectangular"),
                    }
                }
                let mut shape = vec![items.len() as u32];
                shape.extend(inner);
                Ok(Rhs::List(shape, flat))
            }
            Tok::Name(n) => err(
                line,
                format!("{n:?} on the right-hand side: only numeric literals are accepted"),
            ),
            _ => err(line, "expected a number or a list of numbers"),
        }
    }
}

fn finite(line: usize, v: f64) -> Result<Finite, EditError> {
    Finite::new(v).map_err(|e| EditError {
        line,
        message: e.to_string(),
    })
}

/// Compiles an edits file against a spec's tensor shapes. Bounds, fill rules, duplicates and
/// completeness are the contract's checks (`HandwrittenWeightPatch::validate_against`), not this
/// function's: here a statement only has to be a well-formed literal assignment to a known tensor.
pub fn compile(src: &str, spec: &ModelSpec) -> Result<Vec<LiteralOp>, EditError> {
    let toks = lex(src)?;
    let mut p = Parser {
        toks: &toks,
        pos: 0,
    };
    let mut ops = Vec::new();
    loop {
        p.skip_newlines();
        if p.pos + 1 >= toks.len() {
            break;
        }
        let line = p.line();
        let Tok::Name(name) = p.bump().clone() else {
            return err(line, "a statement starts with a tensor name");
        };
        let Some(t) = spec.tensor(&name) else {
            return err(line, format!("unknown tensor {name:?}"));
        };
        let sels = if *p.peek() == Tok::LBracket {
            p.pos += 1;
            p.selectors()?
        } else {
            vec![]
        };
        if *p.bump() != Tok::Equals {
            return err(line, "expected '=' after the target");
        }
        let rhs = p.value()?;
        if *p.bump() != Tok::Newline {
            return err(
                line,
                "one assignment per statement: unexpected text after the value",
            );
        }
        if sels.len() != t.shape.len() {
            return err(
                line,
                format!(
                    "{name} has rank {} but the target names {} axes (a scalar is written `{name} = v`)",
                    t.shape.len(),
                    sels.len()
                ),
            );
        }
        let (mut start, mut bshape, mut squeezed) = (vec![], vec![], vec![]);
        for (axis, (sel, &dim)) in sels.iter().zip(&t.shape).enumerate() {
            let (a, b) = match *sel {
                Sel::Index(i) => (i, i.saturating_add(1)),
                Sel::Slice(lo, hi) => {
                    let (a, b) = (lo.unwrap_or(0), hi.unwrap_or(dim));
                    squeezed.push(b.saturating_sub(a));
                    (a, b)
                }
            };
            if a >= b || b > dim {
                return err(
                    line,
                    format!("{name}: axis {axis} selects [{a}, {b}) outside 0..{dim} or empty"),
                );
            }
            start.push(a);
            bshape.push(b - a);
        }
        let all_index = sels.iter().all(|s| matches!(s, Sel::Index(_)));
        let op = match rhs {
            Rhs::Scalar(v) if all_index => LiteralOp::Set {
                tensor: name,
                index: start,
                value: finite(line, v)?,
            },
            Rhs::Scalar(v) => LiteralOp::FillBox {
                tensor: name,
                start,
                shape: bshape,
                value: finite(line, v)?,
            },
            Rhs::List(lshape, values) => {
                let cells: u64 = bshape.iter().map(|&n| n as u64).product();
                let flat_ok = lshape.len() == 1 && lshape[0] as u64 == cells;
                if !(lshape == squeezed || lshape == bshape || flat_ok) {
                    return err(
                        line,
                        format!(
                            "{name}: a list of shape {lshape:?} cannot fill the selected box {squeezed:?}"
                        ),
                    );
                }
                LiteralOp::SetBox {
                    tensor: name,
                    start,
                    shape: bshape,
                    values: values
                        .into_iter()
                        .map(|v| finite(line, v))
                        .collect::<Result<_, _>>()?,
                }
            }
        };
        ops.push(op);
    }
    if ops.is_empty() {
        return err(1, "the edits file assigns nothing");
    }
    Ok(ops)
}

/// Shortest decimal text that parses back to exactly `v` (Rust's `{}` for `f64`), with a decimal
/// point or exponent so the token never reads as an index.
pub fn literal(v: f64) -> String {
    let s = format!("{v}");
    if s.contains(['.', 'e', 'E']) {
        s
    } else {
        format!("{s}.0")
    }
}

/// Renders one op back in the edit language (inspection and round-trip checks).
pub fn render(op: &LiteralOp) -> String {
    let join = |v: &[Finite]| {
        v.iter()
            .map(|x| literal(x.get()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    match op {
        LiteralOp::Set {
            tensor,
            index,
            value,
        } => {
            if index.is_empty() {
                format!("{tensor} = {}", literal(value.get()))
            } else {
                let idx: Vec<String> = index.iter().map(u32::to_string).collect();
                format!("{tensor}[{}] = {}", idx.join(", "), literal(value.get()))
            }
        }
        LiteralOp::SetBox {
            tensor,
            start,
            shape,
            values,
        } => format!("{tensor}[{}] = [{}]", slices(start, shape), join(values)),
        LiteralOp::FillBox {
            tensor,
            start,
            shape,
            value,
        } => format!(
            "{tensor}[{}] = {}",
            slices(start, shape),
            literal(value.get())
        ),
    }
}

fn slices(start: &[u32], shape: &[u32]) -> String {
    start
        .iter()
        .zip(shape)
        .map(|(s, n)| format!("{s}:{}", s + n))
        .collect::<Vec<_>>()
        .join(", ")
}

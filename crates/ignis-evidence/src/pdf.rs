//! PDF parsing with page numbers, character spans and figure/table caption context (V3.1 §18.1).
//!
//! Page text definition (the coordinate system of every char span): the concatenation, in MuPDF's
//! structured-text order, of the text of all text blocks of the page, each line ending with `\n`
//! (exactly what PyMuPDF's `page.get_text("blocks")` returns, flags
//! `PRESERVE_LIGATURES | PRESERVE_WHITESPACE | MEDIABOX_CLIP | USE_CID_FOR_UNKNOWN_UNICODE`).
//! [`page_text`] reproduces it, so a locator can always be re-checked against the original bytes.
//! Spans count Unicode scalar values (Python `str` indices), not bytes. OCR is not used (textual PDFs
//! only); a page without text is reported.
//!
//! Chunks never cross pages and break at block boundaries (long blocks split at sentence ends).
//! Figure/table captions become their own `caption` chunks and are attached as context to the text
//! chunks of the same page. Table-structure detection (PyMuPDF `find_tables`) is not ported: it found
//! no tables in the reference NTRS PDFs, and `context.tables` stays empty.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use ignis_core::contracts::{OffsetUnit, SourceLocator};
use ignis_core::hashing::sha256_hex;
use mupdf::text_page::TextBlockType;
use mupdf::{Document, MetadataName, TextPageFlags};
use regex::Regex;
use serde_json::{json, Value};

use crate::records::{parse_view_id, Chunk, Publication};
use crate::PARSER_VERSION;

fn caption_rx() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)^\s*(fig\.?|figure|table)\s*[0-9IVX]+[a-z]?\b").unwrap())
}

fn heading_rx() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        // A numbered heading needs an upper-case first word ("3.3 Flame Extinction"). The Python
        // port matched `[A-Z]` case-insensitively, so list items such as "2.2 second tower, ..."
        // became section titles whenever MuPDF put them in a block of their own.
        Regex::new(concat!(
            r"^\s*((?i:\d+(\.\d+)*\.?|[IVX]+\.)\s+[A-Z][^\n]{1,70}|(?i:(abstract|introduction|background|nomenclature|",
            r"experiment(al)?( (setup|apparatus|methods?|hardware))?|apparatus|methods?|results?( and discussion)?|",
            r"discussion|conclusions?|summary|references|acknowledge?ments?|bibliography)\s*:?))\s*$"
        ))
        .unwrap()
    })
}

/// Sections whose text describes setup rather than how runs ended. Everything else (results,
/// conclusions, abstract, unknown) is outcome-bearing: unknown is treated conservatively.
fn non_outcome_rx() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)introduction|background|nomenclature|apparatus|hardware|method|setup|references|bibliography|acknowledg")
            .unwrap()
    })
}

const FAMILY_HINTS: [(&str, &str); 3] = [
    ("acme", r"\bACME\b|gaseous flame"),
    ("bass", r"\bBASS\b|flame spread over|solid fuel"),
    ("flex", r"\bFLEX\b|droplet"),
];

pub fn text_flags() -> TextPageFlags {
    TextPageFlags::PRESERVE_LIGATURES
        | TextPageFlags::PRESERVE_WHITESPACE
        | TextPageFlags::CLIP
        | TextPageFlags::USE_CID_FOR_UNKNOWN_UNICODE
}

/// Text blocks of one page plus the box of every character of their concatenation.
///
/// `rects[i]` is the box of the i-th Unicode scalar of `blocks.concat()` in page coordinates
/// (`None` for the line breaks the block text adds and for glyphs with an empty box). `bounds` is
/// the displayed page (after /Rotate), so a box divided by it is the locator's normalized `bbox`.
pub struct PageLayout {
    pub blocks: Vec<String>,
    pub rects: Vec<Option<[f32; 4]>>,
    pub bounds: [f32; 4],
}

impl PageLayout {
    /// Normalized `[x0, y0, x1, y1]` (origin top-left, y down) of the characters `[a, b)`, or `None`
    /// when none of them has a box.
    pub fn bbox(&self, a: usize, b: usize) -> Option<[f64; 4]> {
        let mut u: Option<[f32; 4]> = None;
        for r in self.rects.get(a..b.min(self.rects.len()))?.iter().flatten() {
            u = Some(match u {
                None => *r,
                Some(l) => [
                    l[0].min(r[0]),
                    l[1].min(r[1]),
                    l[2].max(r[2]),
                    l[3].max(r[3]),
                ],
            });
        }
        let [bx0, by0, bx1, by1] = self.bounds;
        let (w, h) = ((bx1 - bx0) as f64, (by1 - by0) as f64);
        let u = u?;
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let nx = |x: f32| ((x - bx0) as f64 / w).clamp(0.0, 1.0);
        let ny = |y: f32| ((y - by0) as f64 / h).clamp(0.0, 1.0);
        let b = [nx(u[0]), ny(u[1]), nx(u[2]), ny(u[3])];
        (b[0] < b[2] && b[1] < b[3]).then_some(b)
    }
}

/// One page as PyMuPDF `get_text("blocks")` renders it, with character boxes.
pub fn page_layout(doc: &Document, page_no: usize) -> Result<PageLayout> {
    let page = doc.load_page(page_no as i32 - 1)?;
    let b = page.bounds()?;
    let tp = page.to_text_page(text_flags())?;
    let mut blocks = Vec::new();
    let mut rects = Vec::new();
    for block in tp.blocks() {
        if block.r#type() != TextBlockType::Text {
            continue;
        }
        let mut text = String::new();
        let mut block_rects: Vec<Option<[f32; 4]>> = Vec::new();
        let mut last_char: u32 = 0;
        let mut block_nonempty = false;
        for line in block.lines() {
            let mut line_rect: Option<[f32; 4]> = None;
            for ch in line.chars() {
                let q = ch.quad();
                let xs = [q.ul.x, q.ur.x, q.ll.x, q.lr.x];
                let ys = [q.ul.y, q.ur.y, q.ll.y, q.lr.y];
                let r = [
                    xs.iter().cloned().fold(f32::INFINITY, f32::min),
                    ys.iter().cloned().fold(f32::INFINITY, f32::min),
                    xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
                    ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max),
                ];
                // JM_rects_overlap(mediabox, char bbox)
                if r[0] >= b.x1 || r[1] >= b.y1 || r[2] <= b.x0 || r[3] <= b.y0 {
                    continue;
                }
                let c = ch.char().unwrap_or('\u{FFFD}');
                text.push(c);
                last_char = c as u32;
                // fz_union_rect: an empty rect is the identity
                let nonempty = r[0] < r[2] && r[1] < r[3];
                block_rects.push(nonempty.then_some(r));
                if nonempty {
                    line_rect = Some(match line_rect {
                        None => r,
                        Some(l) => [
                            l[0].min(r[0]),
                            l[1].min(r[1]),
                            l[2].max(r[2]),
                            l[3].max(r[3]),
                        ],
                    });
                }
            }
            if line_rect.is_some() {
                block_nonempty = true;
                if last_char != 10 {
                    text.push('\n');
                    block_rects.push(None);
                }
            }
        }
        if block_nonempty {
            blocks.push(text);
            rects.extend(block_rects);
        }
    }
    Ok(PageLayout {
        blocks,
        rects,
        bounds: [b.x0, b.y0, b.x1, b.y1],
    })
}

/// One text block of a page, as PyMuPDF `get_text("blocks")` renders it.
pub fn page_blocks(doc: &Document, page_no: usize) -> Result<Vec<String>> {
    Ok(page_layout(doc, page_no)?.blocks)
}

/// Text of 1-based `page_no` in the char-span coordinate system used by every locator.
pub fn page_text(doc: &Document, page_no: usize) -> Result<String> {
    Ok(page_blocks(doc, page_no)?.concat())
}

pub fn open_pdf(path: &Path) -> Result<(Document, Vec<u8>)> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let doc = Document::from_bytes(&bytes, "application/pdf")
        .with_context(|| format!("open {}", path.display()))?;
    Ok((doc, bytes))
}

/// Regexes that attribute text to runs; `alias` is formatted with the (upper-cased) match groups.
#[derive(Debug, Clone, Default)]
pub struct RunPatterns {
    pub patterns: Vec<(Regex, String)>,
}

impl RunPatterns {
    pub fn load(path: &Path) -> Result<RunPatterns> {
        if !path.exists() {
            return Ok(RunPatterns::default());
        }
        let v: toml::Value = toml::from_str(&std::fs::read_to_string(path)?)?;
        let mut patterns = Vec::new();
        for p in v
            .get("pattern")
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default()
        {
            let rx = p
                .get("regex")
                .and_then(|x| x.as_str())
                .context("pattern.regex")?;
            let alias = p
                .get("alias")
                .and_then(|x| x.as_str())
                .context("pattern.alias")?;
            patterns.push((
                Regex::new(rx).with_context(|| format!("run pattern {rx}"))?,
                alias.to_string(),
            ));
        }
        Ok(RunPatterns { patterns })
    }

    pub fn default_path() -> PathBuf {
        ignis_core::resources::configs_dir()
            .join("evidence")
            .join("run_patterns.toml")
    }

    /// Alias of the match that starts last in `text` (the run a following chunk could continue).
    pub fn last(&self, text: &str) -> Option<String> {
        let mut best: Option<(usize, String)> = None;
        for (rx, alias) in &self.patterns {
            for caps in rx.captures_iter(text) {
                let start = caps.get(0).map_or(0, |m| m.start());
                if best.as_ref().is_some_and(|(b, _)| *b > start) {
                    continue;
                }
                let mut s = alias.clone();
                for i in 1..caps.len() {
                    let g = caps
                        .get(i)
                        .map(|m| m.as_str().to_uppercase())
                        .unwrap_or_default();
                    s = s.replace(&format!("{{{}}}", i - 1), &g);
                }
                best = Some((start, s));
            }
        }
        best.map(|(_, s)| s)
    }

    pub fn find(&self, text: &str) -> Vec<String> {
        let mut out = BTreeSet::new();
        for (rx, alias) in &self.patterns {
            for caps in rx.captures_iter(text) {
                let mut s = alias.clone();
                for i in 1..caps.len() {
                    let g = caps
                        .get(i)
                        .map(|m| m.as_str().to_uppercase())
                        .unwrap_or_default();
                    s = s.replace(&format!("{{{}}}", i - 1), &g);
                }
                out.insert(s);
            }
        }
        out.into_iter().collect()
    }
}

/// NTRS citation JSON next to the PDF (`<stem>.json`), if fetched.
#[derive(Debug, Default)]
struct Sidecar {
    title: Option<String>,
    authors: Vec<String>,
    year: Option<i64>,
    ntrs_id: Option<String>,
    canonical_url: Option<String>,
    rights_status: Option<String>,
    license: Option<String>,
}

fn sidecar(pdf: &Path) -> Result<Sidecar> {
    let p = pdf.with_extension("json");
    if !p.exists() {
        return Ok(Sidecar::default());
    }
    let d: Value = serde_json::from_str(&std::fs::read_to_string(&p)?)?;
    let authors = d["authorAffiliations"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x["meta"]["author"]["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let mut year = None;
    // as in the Python port: the first dated entry of each list wins, a later list overrides
    for k in ["publications", "meetings"] {
        for x in d[k].as_array().into_iter().flatten() {
            let dt = x["publicationDate"]
                .as_str()
                .filter(|s| !s.is_empty())
                .or(x["startDate"].as_str().filter(|s| !s.is_empty()));
            if let Some(dt) = dt {
                year = dt.get(..4).and_then(|y| y.parse().ok());
                break;
            }
        }
    }
    if year.is_none() {
        year = d["submittedDate"]
            .as_str()
            .and_then(|s| s.get(..4))
            .and_then(|y| y.parse().ok());
    }
    let id = match &d["id"] {
        Value::Null => None,
        Value::String(s) if s.is_empty() => None,
        Value::Number(n) if n.as_i64() == Some(0) => None,
        Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    };
    let rights = d["copyright"]["determinationType"]
        .as_str()
        .map(String::from);
    Ok(Sidecar {
        title: d["title"].as_str().map(String::from),
        authors,
        year,
        canonical_url: id
            .as_ref()
            .map(|i| format!("https://ntrs.nasa.gov/citations/{i}")),
        ntrs_id: id,
        rights_status: Some(
            if rights.as_deref() == Some("PUBLIC_USE_PERMITTED") {
                "verified"
            } else {
                "unverified"
            }
            .into(),
        ),
        license: rights,
    })
}

/// Python `str.rfind(". ", s, e)` on a char vector.
fn rfind_dot_space(t: &[char], s: usize, e: usize) -> Option<usize> {
    let e = e.min(t.len());
    if e < s + 2 {
        return None;
    }
    (s..=e - 2).rev().find(|&i| t[i] == '.' && t[i + 1] == ' ')
}

fn split_long(text: &[char], start: usize, max_chars: usize) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut s = 0usize;
    while text.len() - s > max_chars {
        let cut = match rfind_dot_space(text, s, s + max_chars) {
            Some(c) if c > s + max_chars / 3 => c + 2,
            _ => s + max_chars,
        };
        spans.push((start + s, start + cut));
        s = cut;
    }
    spans.push((start + s, start + text.len()));
    spans
}

/// Python `str.strip()` whitespace (Unicode White_Space plus the ASCII separators \x1c-\x1f).
fn py_space(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub target_chars: usize,
    pub max_chars: usize,
    pub investigation_ids: Vec<String>,
}

impl Default for ParseOptions {
    fn default() -> Self {
        ParseOptions {
            target_chars: 900,
            max_chars: 1800,
            investigation_ids: Vec::new(),
        }
    }
}

/// The parse view of one PDF: the publication record, its chunks and the exact text of every
/// physical page (index 0 = page 1; empty for a page without text), the coordinate system of the
/// chunks' character offsets.
#[derive(Debug, Clone)]
pub struct ParsedPdf {
    pub publication: Publication,
    pub chunks: Vec<Chunk>,
    pub pages: Vec<String>,
    pub report: Value,
}

pub fn parse_pdf(path: &Path, opts: &ParseOptions, rp: &RunPatterns) -> Result<ParsedPdf> {
    let (doc, bytes) = open_pdf(path)?;
    let sha = sha256_hex(&bytes);
    let side = sidecar(path)?;
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let pub_id = format!("pub-{}", &sha[..16]);
    let view_id = parse_view_id(&pub_id, PARSER_VERSION);
    let mut pages: Vec<String> = Vec::new();
    let n_pages = doc.page_count()? as usize;
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut pages_without_text = Vec::new();
    let mut n_captions = 0usize;
    let mut section: Option<String> = None;
    let mut all_runs = BTreeSet::new();
    let mut family_text: Vec<String> = Vec::new();
    // last run named so far in reading order (carried across chunks and pages)
    let mut carry: Option<String> = None;

    let mut add = |chunks: &mut Vec<Chunk>,
                   section: &Option<String>,
                   kind: &str,
                   page: usize,
                   cs: usize,
                   ce: usize,
                   bbox: Option<[f64; 4]>,
                   text: String,
                   ctx: Value| {
        let runs = rp.find(&text);
        all_runs.extend(runs.iter().cloned());
        let continues_runs: Vec<String> = carry.iter().cloned().collect();
        if let Some(l) = rp.last(&text) {
            carry = Some(l);
        }
        let outcome = !section
            .as_deref()
            .is_some_and(|s| !s.is_empty() && non_outcome_rx().is_match(s));
        let locator = SourceLocator {
            asset_sha256: sha.clone(),
            path: Some(file_name.clone()),
            row: None,
            column: None,
            page: Some(page as i64),
            frame: None,
            pts_s: None,
            char_start: Some(cs as i64),
            char_end: Some(ce as i64),
            parse_view_id: Some(view_id.clone()),
            offset_unit: Some(OffsetUnit::UnicodeScalar),
            bbox,
            ..Default::default()
        };
        let ordinal = chunks.len();
        chunks.push(Chunk {
            id: format!("{view_id}:p{page}:{ordinal:05}"),
            publication_id: pub_id.clone(),
            ordinal: ordinal as i64,
            kind: kind.into(),
            page: page as i64,
            char_start: Some(cs as i64),
            char_end: Some(ce as i64),
            section: section.clone(),
            text,
            context: ctx,
            locator,
            describes_runs: runs,
            continues_runs,
            outcome_bearing: outcome,
            evidence_class: "source_derived".into(),
        });
    };

    for pno in 1..=n_pages {
        let layout = page_layout(&doc, pno)?;
        let blocks = &layout.blocks;
        let full_s = blocks.concat();
        pages.push(full_s.clone());
        if full_s.trim_matches(py_space).is_empty() {
            pages_without_text.push(pno);
            continue;
        }
        let full: Vec<char> = full_s.chars().collect();
        family_text.push(full[..full.len().min(4000)].iter().collect());
        let captions: Vec<String> = blocks
            .iter()
            .filter(|b| caption_rx().is_match(b))
            .map(|b| b.trim_matches(py_space).to_string())
            .collect();
        let ctx = json!({"captions": captions.iter().take(8).collect::<Vec<_>>(), "tables": []});
        let mut offset = 0usize;
        let mut buf_start: Option<usize> = None;
        let mut buf_end = 0usize;

        let spans = |buf_start: &mut Option<usize>, buf_end: usize| -> Vec<(usize, usize)> {
            match buf_start.take() {
                Some(bs) if !full[bs..buf_end].iter().all(|c| py_space(*c)) => {
                    split_long(&full[bs..buf_end], bs, opts.max_chars)
                }
                _ => Vec::new(),
            }
        };
        macro_rules! flush {
            () => {
                for (a, b) in spans(&mut buf_start, buf_end) {
                    add(
                        &mut chunks,
                        &section,
                        "text",
                        pno,
                        a,
                        b,
                        layout.bbox(a, b),
                        full[a..b].iter().collect(),
                        ctx.clone(),
                    );
                }
            };
        }

        for t in blocks {
            let len = t.chars().count();
            let (a, e) = (offset, offset + len);
            offset = e;
            let lines: Vec<&str> = t
                .trim_matches(py_space)
                .split('\n')
                .map(|x| x.trim_matches(py_space))
                .filter(|x| !x.is_empty())
                .collect();
            let cand = if !lines.is_empty() && lines.len() <= 2 {
                lines.join(" ")
            } else {
                String::new()
            };
            if !cand.is_empty() && cand.chars().count() <= 80 && heading_rx().is_match(&cand) {
                flush!();
                section = Some(cand);
            }
            if caption_rx().is_match(t) {
                flush!();
                add(
                    &mut chunks,
                    &section,
                    "caption",
                    pno,
                    a,
                    e,
                    layout.bbox(a, e),
                    t.clone(),
                    json!({"captions": [], "tables": []}),
                );
                n_captions += 1;
                continue;
            }
            if buf_start.is_none() {
                buf_start = Some(a);
            }
            buf_end = e;
            if buf_end - buf_start.unwrap() >= opts.target_chars {
                flush!();
            }
        }
        flush!();
    }
    let joined = family_text.join(" ");
    let head: String = joined.chars().take(20000).collect();
    let families: Vec<String> = FAMILY_HINTS
        .iter()
        .filter(|(_, rx)| Regex::new(rx).unwrap().is_match(&head))
        .map(|(f, _)| f.to_string())
        .collect();
    let meta_title = doc.metadata(MetadataName::Title).unwrap_or_default();
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let title = side
        .title
        .clone()
        .filter(|t| !t.is_empty())
        .or(Some(meta_title).filter(|t| !t.is_empty()))
        .unwrap_or(stem);
    let publication = Publication {
        id: pub_id.clone(),
        source_sha256: sha.clone(),
        title: Some(title),
        authors: side.authors,
        year: side.year,
        ntrs_id: side.ntrs_id,
        doi: None,
        canonical_url: side.canonical_url,
        investigation_ids: opts.investigation_ids.clone(),
        families,
        file_name: Some(file_name.clone()),
        n_pages: n_pages as i64,
        rights_status: side.rights_status.unwrap_or_else(|| "unverified".into()),
        license: side.license,
        describes_runs: all_runs.into_iter().collect(),
        outcome_bearing: true,
        ingested_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false),
        parser_version: PARSER_VERSION.into(),
    };
    let report = json!({"parse_view_id": view_id, "pages_without_text": pages_without_text, "tables": 0,
        "captions": n_captions, "chunks": chunks.len()});
    Ok(ParsedPdf {
        publication,
        chunks,
        pages,
        report,
    })
}

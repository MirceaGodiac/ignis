//! Section outlines (R7b): a deterministic [`SourceOutline`] per document, built from the active
//! parse view's chunks in the catalog. No model is involved.
//!
//! **Headings.** A Docling view marks headings (`section_header`/`title` items → chunk kind
//! `heading`); those are used as they are. A PDFium text-layer view has no structure, so headings
//! are recognised from the text layer:
//! * numbered: `3`, `3.2`, `3.2.1` (at most 4 levels, first number ≤ 40), roman `IV.`, or
//!   `Appendix A` / `Chapter 2` / `Section 4`, followed by a word that starts with a capital letter;
//! * set in capitals (≥ 90 % upper-case letters; two or more words, or one word of 5+ letters);
//! * set larger than the body: a line height ≥ 1.3 × the document's median line height.
//!
//! In every case a heading is short (≤ 120 characters, ≤ 14 words, at most 2 lines), does not end
//! like a sentence (`.`, `,`, `;`), and is not a running header or footer: the same text (digits
//! ignored, so a footer's page number does not hide it) on 3 or more pages is dropped. Numbered
//! headings repeat their numbers, so they are never taken for footers unless the text repeats too.
//!
//! **Levels and nesting.** The level is the numbering depth (`3.2.1` → 3; roman, appendix, chapter
//! → 1); an unnumbered heading is level 1. A section runs from its heading's page to the page of the
//! next heading at the same or a higher level (inclusive: that heading may start mid-page), or to
//! the last page; children nest under the nearest open heading of a lower level. These page spans
//! satisfy `SourceOutline::validate` by construction (a child never ends after its parent).
//!
//! **Ids.** `s<page>-<8 hex of sha256(normalized title)>`, with `-2`, `-3`… for repeats: stable across
//! rebuilds of the same view and unaffected by headings elsewhere in the document.
//!
//! A document with no recognised heading gets one node spanning every page, titled with the
//! publication title (or file name), so `sources_outline` always answers with the page range.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use anyhow::Result;
use ignis_core::contracts::{OutlineNode, OutlineNodeId};
use regex::Regex;
use rusqlite::{params, Connection};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// One chunk of the active view, as the outline (and the lockbox) needs it.
#[derive(Debug, Clone)]
pub struct OutlineChunk {
    pub kind: String,
    pub page: i64,
    pub text: String,
    /// Normalized line heights (`bbox[3] - bbox[1]`), when the view records lines.
    pub line_heights: Vec<f64>,
    /// Docling item label (`title`, `section_header`, …), when recorded.
    pub label: Option<String>,
    pub outcome_bearing: bool,
    pub describes_runs: Vec<String>,
    pub continues_runs: Vec<String>,
}

/// How the headings of an outline were found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    DoclingHeadings,
    TextLayer,
    /// No heading recognised: one node for the whole document.
    WholeDocument,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Method::DoclingHeadings => "docling_headings",
            Method::TextLayer => "text_layer",
            Method::WholeDocument => "whole_document",
        }
    }
}

/// The chunks of a publication's active view, in reading order.
pub fn load_chunks(con: &Connection, publication_id: &str) -> Result<Vec<OutlineChunk>> {
    let mut st = con.prepare(
        "SELECT kind, page, text, context_json, outcome_bearing, describes_runs_json, continues_runs_json \
         FROM chunks WHERE publication_id = ?1 ORDER BY page, ordinal",
    )?;
    let rows = st.query_map(params![publication_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, i64>(4)? != 0,
            r.get::<_, String>(5)?,
            r.get::<_, String>(6)?,
        ))
    })?;
    let arr = |s: &str| serde_json::from_str::<Vec<String>>(s).unwrap_or_default();
    let mut out = Vec::new();
    for row in rows {
        let (kind, page, text, ctx, ob, runs, cont) = row?;
        let ctx: Value = serde_json::from_str(&ctx).unwrap_or(Value::Null);
        let line_heights = ctx["lines"]
            .as_array()
            .map(|ls| {
                ls.iter()
                    .filter_map(|l| {
                        let b = l["bbox"].as_array()?;
                        Some(b.get(3)?.as_f64()? - b.get(1)?.as_f64()?)
                    })
                    .filter(|h| *h > 0.0)
                    .collect()
            })
            .unwrap_or_default();
        out.push(OutlineChunk {
            kind,
            page,
            text,
            line_heights,
            label: ctx["label"].as_str().map(String::from),
            outcome_bearing: ob,
            describes_runs: arr(&runs),
            continues_runs: arr(&cont),
        });
    }
    Ok(out)
}

fn numbering_rx() -> &'static Regex {
    static RX: OnceLock<Regex> = OnceLock::new();
    RX.get_or_init(|| {
        Regex::new(
            r"^(?:(?P<num>\d{1,2}(?:\.\d{1,2}){0,3})\.?|(?P<roman>[IVX]{1,5})\.|(?P<word>Appendix|APPENDIX|Chapter|CHAPTER|Section|SECTION)\s+[A-Z0-9]{1,3}[.:]?)\s+(?P<rest>\S.*)$",
        )
        .unwrap()
    })
}

/// Numbering level of a heading text (`3.2` → 2, `IV.` → 1, `Appendix A` → 1), if numbered.
pub fn numbering_level(text: &str) -> Option<usize> {
    let c = numbering_rx().captures(text.trim())?;
    let rest = c.name("rest")?.as_str();
    if !rest.chars().next().is_some_and(|ch| ch.is_uppercase()) {
        return None;
    }
    if let Some(n) = c.name("num") {
        let parts: Vec<&str> = n.as_str().split('.').collect();
        let first: u32 = parts[0].parse().ok()?;
        if first == 0 || first > 40 {
            return None;
        }
        return Some(parts.len());
    }
    Some(1)
}

fn short(text: &str, lines: usize) -> bool {
    let t = text.trim();
    let words = t.split_whitespace().count();
    (2..=120).contains(&t.chars().count())
        && words <= 14
        && lines <= 2
        && t.chars().any(|c| c.is_alphabetic())
        && !t.ends_with(['.', ',', ';'])
}

fn capitals(text: &str) -> bool {
    let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    let upper = letters.iter().filter(|c| c.is_uppercase()).count();
    let words = text
        .split_whitespace()
        .filter(|w| w.chars().any(char::is_alphabetic))
        .count();
    // One capitalised word is a heading only when it is a word, not an acronym (`ABSTRACT`, not
    // `NASA`).
    let one_word =
        words == 1 && letters.len() >= 5 && text.trim().chars().all(|c| c.is_alphabetic());
    (words >= 2 && letters.len() >= 4 || one_word)
        && upper * 10 >= letters.len() * 9
        && text.chars().count() <= 80
}

fn norm(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// `(page, level, title)` of every heading, in reading order, and how they were found.
pub fn headings(chunks: &[OutlineChunk]) -> (Vec<(i64, usize, String)>, Method) {
    let tidy = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    if chunks.iter().any(|c| c.kind == "heading") {
        let hs = chunks
            .iter()
            .filter(|c| c.kind == "heading" && !c.text.trim().is_empty())
            .map(|c| (c.page, numbering_level(&c.text).unwrap_or(1), tidy(&c.text)))
            .collect();
        return (hs, Method::DoclingHeadings);
    }
    // Running headers and footers: the same short text on 3+ pages, numbers ignored (a footer
    // carries the page number: `NASA/TP-2015-216046 3`, `… 4`).
    let shape = |t: &str| -> String {
        norm(t)
            .chars()
            .map(|c| if c.is_ascii_digit() { '#' } else { c })
            .collect()
    };
    let mut pages_of: HashMap<String, std::collections::BTreeSet<i64>> = HashMap::new();
    for c in chunks {
        if c.text.chars().count() <= 120 {
            pages_of.entry(shape(&c.text)).or_default().insert(c.page);
        }
    }
    let mut heights: Vec<f64> = chunks
        .iter()
        .flat_map(|c| c.line_heights.iter().copied())
        .collect();
    heights.sort_by(|a, b| a.total_cmp(b));
    let median = heights.get(heights.len() / 2).copied();
    let mut hs = Vec::new();
    for c in chunks.iter().filter(|c| c.kind == "text") {
        let lines = c.line_heights.len().max(1);
        if !short(&c.text, lines) {
            continue;
        }
        if pages_of.get(&shape(&c.text)).is_some_and(|p| p.len() >= 3) {
            continue;
        }
        let numbered = numbering_level(&c.text);
        let large = median.is_some_and(|m| {
            !c.line_heights.is_empty()
                && c.line_heights.iter().copied().fold(0.0, f64::max) >= 1.3 * m
                && c.text.chars().count() <= 100
        });
        if numbered.is_some() || capitals(&c.text) || large {
            hs.push((c.page, numbered.unwrap_or(1), tidy(&c.text)));
        }
    }
    let method = if hs.is_empty() {
        Method::WholeDocument
    } else {
        Method::TextLayer
    };
    (hs, method)
}

/// Stable id of a heading.
pub fn node_id(page: i64, title: &str, seen: &mut BTreeMap<String, usize>) -> String {
    let h = hex::encode(Sha256::digest(norm(title).as_bytes()));
    let base = format!("s{page}-{}", &h[..8]);
    let n = seen.entry(base.clone()).or_insert(0);
    *n += 1;
    if *n == 1 {
        base
    } else {
        format!("{base}-{n}")
    }
}

/// The section tree of a document with `n_pages` pages.
pub fn build(
    heads: &[(i64, usize, String)],
    n_pages: i64,
    whole_title: &str,
) -> Result<Vec<OutlineNode>> {
    let n_pages = n_pages.max(1) as u32;
    if heads.is_empty() {
        return Ok(vec![OutlineNode {
            node_id: OutlineNodeId::new("s1-whole")?,
            title: whole_title.to_string(),
            page_start: 1,
            page_end: n_pages,
            summary: None,
            children: vec![],
            children_omitted: false,
        }]);
    }
    let mut seen = BTreeMap::new();
    // Flat nodes with their level and end page.
    let mut flat: Vec<(usize, OutlineNode)> = Vec::new();
    for (i, (page, level, title)) in heads.iter().enumerate() {
        let start = (*page).clamp(1, n_pages as i64) as u32;
        let end = heads[i + 1..]
            .iter()
            .find(|(_, l, _)| l <= level)
            .map_or(n_pages, |(p, _, _)| (*p).clamp(1, n_pages as i64) as u32)
            .max(start);
        flat.push((
            *level,
            OutlineNode {
                node_id: OutlineNodeId::new(node_id(*page, title, &mut seen))?,
                title: title.clone(),
                page_start: start,
                page_end: end,
                summary: None,
                children: vec![],
                children_omitted: false,
            },
        ));
    }
    // Nest: each node goes under the nearest open node of a lower level.
    let mut roots: Vec<OutlineNode> = Vec::new();
    let mut stack: Vec<(usize, OutlineNode)> = Vec::new();
    let close = |stack: &mut Vec<(usize, OutlineNode)>, roots: &mut Vec<OutlineNode>| {
        let (_, n) = stack.pop().unwrap();
        match stack.last_mut() {
            Some((_, parent)) => parent.children.push(n),
            None => roots.push(n),
        }
    };
    for (level, node) in flat {
        while stack.last().is_some_and(|(l, _)| *l >= level) {
            close(&mut stack, &mut roots);
        }
        stack.push((level, node));
    }
    while !stack.is_empty() {
        close(&mut stack, &mut roots);
    }
    Ok(roots)
}

/// The subtree at `node_id` (or every root), cut `depth` levels below it; cut children are
/// flagged with `children_omitted`, never dropped silently.
pub fn select(
    roots: Vec<OutlineNode>,
    node_id: Option<&OutlineNodeId>,
    depth: Option<u32>,
) -> Option<Vec<OutlineNode>> {
    fn find(nodes: Vec<OutlineNode>, id: &OutlineNodeId) -> Option<OutlineNode> {
        for n in nodes {
            if &n.node_id == id {
                return Some(n);
            }
            if let Some(f) = find(n.children, id) {
                return Some(f);
            }
        }
        None
    }
    fn cut(mut n: OutlineNode, left: u32) -> OutlineNode {
        if left == 0 {
            if !n.children.is_empty() {
                n.children.clear();
                n.children_omitted = true;
            }
        } else {
            n.children = n.children.into_iter().map(|c| cut(c, left - 1)).collect();
        }
        n
    }
    let base = match node_id {
        Some(id) => vec![find(roots, id)?],
        None => roots,
    };
    Some(match depth {
        // `depth` levels below the root(s): the roots' own children are level 1.
        // A subtree root's children are level 1; without one the document is the root, so the
        // top-level sections are level 1.
        Some(d) => {
            let below = if node_id.is_some() { d } else { d - 1 };
            base.into_iter().map(|n| cut(n, below)).collect()
        }
        None => base,
    })
}

/// Number of nodes in a tree.
pub fn count(nodes: &[OutlineNode]) -> usize {
    nodes.iter().map(|n| 1 + count(&n.children)).sum()
}

// ------------------------------------------------------------------ section summaries

/// Static prefix of a section-summary request (cacheable; its digest is the answers' prompt hash).
pub const SUMMARY_SYSTEM: &str = "You summarise one section of a NASA microgravity-combustion document for navigation. \
Write 1 to 3 plain sentences saying what the section covers (topics, quantities, tests, figures or tables it holds). \
Do not state results, numbers or conclusions as facts: the summary is navigation help and is never cited. \
If the section text is empty or unreadable, answer exactly: (no readable text).";

/// Output budget of one summary.
pub const SUMMARY_MAX_TOKENS: u32 = 160;

fn walk_nodes<'a>(nodes: &'a [OutlineNode], leaves_only: bool, out: &mut Vec<&'a OutlineNode>) {
    for n in nodes {
        if !leaves_only || n.children.is_empty() {
            out.push(n);
        }
        walk_nodes(&n.children, leaves_only, out);
    }
}

/// One request per section (leaves only by default: a parent's pages repeat its children's) of
/// each publication's active view. `custom_id` = `<parse view id>#<node id>`.
pub fn summary_requests(
    con: &Connection,
    pubs: &[crate::records::Publication],
    leaves_only: bool,
) -> Result<Vec<crate::llm_pass::Request>> {
    let mut out = Vec::new();
    for p in pubs {
        let view = crate::records::parse_view_id(&p.id, &p.parser_version);
        let chunks = load_chunks(con, &p.id)?;
        let (heads, _) = headings(&chunks);
        let title = p
            .title
            .clone()
            .or_else(|| p.file_name.clone())
            .unwrap_or_default();
        let roots = build(&heads, p.n_pages, &title)?;
        let mut nodes = Vec::new();
        walk_nodes(&roots, leaves_only, &mut nodes);
        for n in nodes {
            let pages = crate::catalog::page_texts(
                con,
                &p.id,
                &view,
                n.page_start as i64,
                n.page_end as i64,
            )?;
            let text: Vec<String> = pages.into_iter().map(|(_, t)| t).collect();
            out.push(crate::llm_pass::Request {
                custom_id: format!("{view}#{}", n.node_id),
                task: crate::llm_pass::Task::SectionSummary,
                system: SUMMARY_SYSTEM.into(),
                user: format!(
                    "Document: {title}\nSection: {} (pages {}-{})\n\n{}",
                    n.title,
                    n.page_start,
                    n.page_end,
                    text.join("\n")
                ),
                max_output_tokens: SUMMARY_MAX_TOKENS,
            });
        }
    }
    Ok(out)
}

/// Section summaries from a pass's answers file, keyed by (parse view id, node id).
pub fn load_summaries(
    path: &std::path::Path,
) -> Result<HashMap<(String, String), ignis_core::contracts::GeneratedText>> {
    let mut out = HashMap::new();
    for a in crate::llm_pass::read_answers(path)? {
        if a.task != crate::llm_pass::Task::SectionSummary {
            continue;
        }
        let Some((view, node)) = a.custom_id.split_once('#') else {
            continue;
        };
        let text = a.text.trim();
        if text.is_empty() || text == "(no readable text)" {
            continue;
        }
        out.insert(
            (view.to_string(), node.to_string()),
            ignis_core::contracts::GeneratedText {
                text: text.to_string(),
                model_id: a.model_id,
                prompt_hash: Some(a.prompt_hash),
            },
        );
    }
    Ok(out)
}

/// Puts each node's summary (generated text, labelled with its model) in place.
pub fn attach(
    nodes: &mut [OutlineNode],
    view: &str,
    summaries: &HashMap<(String, String), ignis_core::contracts::GeneratedText>,
) {
    for n in nodes {
        n.summary = summaries
            .get(&(view.to_string(), n.node_id.as_str().to_string()))
            .cloned();
        attach(&mut n.children, view, summaries);
    }
}

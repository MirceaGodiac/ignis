//! Import of a Docling parse view (`ignis.docling_view.v1`, produced offline by
//! `sidecar/docling/ignis_docling.py`) as a catalog parse view, R7a.
//!
//! The view becomes the publication's active view; any other view of the same PDF (the MuPDF one)
//! stays stored, so citations made against it still resolve (catalog v3).
//!
//! Page text of this view (the coordinate system of its char spans): the chunk texts of the page in
//! Docling's reading order, each followed by `\n`. Spans count Unicode scalar values.
//!
//! Chunks: one per text item (paragraph, list item, caption, heading, footnote); one per table **row**
//! (`table_row`: the row's cells joined with ` | `, `locator.table_id`, the cells with their own boxes
//! in `context.cells`, so a viewer can highlight the exact cell); one `table` chunk for a table whose
//! structure Docling discarded (`structure: text_only`, never shown as rows); one `figure` chunk per
//! captioned figure (its caption text, the figure's box). Attribution (runs, outcome-bearing sections,
//! families, investigation) uses the same rules as MuPDF views ([`crate::attribution`]).
//!
//! Refused: a view whose PDF bytes do not hash to `source.sha256`, a view with `failed_pages` (the
//! document keeps its current view rather than getting a partial one), a box outside `[0, 1]`.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Result};
use ignis_core::contracts::{OffsetUnit, SourceLocator};
use ignis_core::hashing::sha256_hex;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::attribution::{families_in, non_outcome_rx, publication_investigations, RunPatterns};
use crate::records::{parse_view_id, Chunk, Publication};

pub const FORMAT: &str = "ignis.docling_view.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoclingView {
    pub format: String,
    pub parser: Value,
    pub source: ViewSource,
    pub pages: Vec<ViewPage>,
    pub items: Vec<ViewItem>,
    pub tables: Vec<ViewTable>,
    pub figures: Vec<ViewFigure>,
    pub failed_pages: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewSource {
    pub file_name: String,
    pub sha256: String,
    pub n_pages: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewPage {
    pub page: i64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewItem {
    pub kind: String,
    pub page: i64,
    pub bbox: [f64; 4],
    pub text: String,
    #[serde(default)]
    pub table: Option<usize>,
    #[serde(default)]
    pub row: Option<i64>,
    #[serde(default)]
    pub col: Option<i64>,
    #[serde(default)]
    pub row_span: Option<i64>,
    #[serde(default)]
    pub col_span: Option<i64>,
    #[serde(default)]
    pub header: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewTable {
    pub table: usize,
    pub page: i64,
    pub bbox: [f64; 4],
    pub n_rows: i64,
    pub n_cols: i64,
    pub structure: String,
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewFigure {
    pub figure: usize,
    pub page: i64,
    pub bbox: [f64; 4],
    pub caption: Option<String>,
}

/// A parse view ready for `catalog::upsert_parse_view`.
pub struct ImportedView {
    pub publication: Publication,
    pub chunks: Vec<Chunk>,
    pub pages: Vec<String>,
}

fn check_box(b: &[f64; 4], what: &str) -> Result<()> {
    ensure(
        b.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)) && b[0] <= b[2] && b[1] <= b[3],
        format!("{what}: box {b:?} is not normalized [x0, y0, x1, y1] in [0, 1]"),
    )
}

fn ensure(cond: bool, msg: String) -> Result<()> {
    if cond {
        Ok(())
    } else {
        bail!(msg)
    }
}

fn union(boxes: impl Iterator<Item = [f64; 4]>) -> Option<[f64; 4]> {
    boxes.reduce(|a, b| {
        [
            a[0].min(b[0]),
            a[1].min(b[1]),
            a[2].max(b[2]),
            a[3].max(b[3]),
        ]
    })
}

/// One unit of the page text before offsets are known.
struct Unit {
    page: i64,
    kind: &'static str,
    text: String,
    bbox: [f64; 4],
    table_id: Option<String>,
    /// Row of a table row chunk (a locator's `table_id` needs it).
    row: Option<i64>,
    context: Value,
    heading: bool,
}

pub fn parser_version(view: &DoclingView) -> String {
    format!(
        "docling-{}",
        view.parser["version"].as_str().unwrap_or("unknown")
    )
}

/// Turn a Docling view of `pdf_path` (whose bytes are `pdf_bytes`) into a catalog parse view.
pub fn import(
    view: &DoclingView,
    pdf_path: &Path,
    pdf_bytes: &[u8],
    rp: &RunPatterns,
    investigations: &[String],
) -> Result<ImportedView> {
    ensure(
        view.format == FORMAT,
        format!("format {:?}, expected {FORMAT}", view.format),
    )?;
    let sha = sha256_hex(pdf_bytes);
    ensure(
        sha == view.source.sha256,
        format!(
            "the view was made from other bytes ({} != {sha})",
            view.source.sha256
        ),
    )?;
    ensure(
        view.failed_pages.is_empty(),
        format!(
            "{}: pages {:?} failed to parse; the document keeps its current view",
            view.source.file_name, view.failed_pages
        ),
    )?;
    let n_pages = view.source.n_pages;
    for it in &view.items {
        check_box(&it.bbox, &format!("item on page {}", it.page))?;
        ensure(
            (1..=n_pages).contains(&it.page),
            format!("item on page {} of {n_pages}", it.page),
        )?;
    }
    for t in &view.tables {
        check_box(&t.bbox, &format!("table {}", t.table))?;
    }
    for f in &view.figures {
        check_box(&f.bbox, &format!("figure {}", f.figure))?;
    }

    // Units in reading order: table cells become one unit per row at the position of the table.
    let tables: BTreeMap<usize, &ViewTable> = view.tables.iter().map(|t| (t.table, t)).collect();
    let mut cells: BTreeMap<usize, BTreeMap<i64, Vec<&ViewItem>>> = BTreeMap::new();
    for it in view.items.iter().filter(|i| i.kind == "table_cell") {
        if let (Some(t), Some(r)) = (it.table, it.row) {
            cells.entry(t).or_default().entry(r).or_default().push(it);
        }
    }
    let captions: std::collections::BTreeSet<(i64, String)> = view
        .tables
        .iter()
        .filter_map(|t| t.caption.clone().map(|c| (t.page, c)))
        .chain(
            view.figures
                .iter()
                .filter_map(|f| f.caption.clone().map(|c| (f.page, c))),
        )
        .collect();
    let mut units: Vec<Unit> = Vec::new();
    let mut emitted_tables = std::collections::BTreeSet::new();
    let mut emit_table = |t: usize, units: &mut Vec<Unit>| {
        if !emitted_tables.insert(t) {
            return;
        }
        let Some(tab) = tables.get(&t) else { return };
        let rows = cells.get(&t).cloned().unwrap_or_default();
        let header: Vec<String> = rows
            .values()
            .next()
            .filter(|r| r.iter().all(|c| c.header == Some(true)))
            .map(|r| r.iter().map(|c| c.text.clone()).collect())
            .unwrap_or_default();
        let table_id = format!("t{t}");
        if tab.structure == "text_only" {
            let text: String = rows
                .values()
                .flatten()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if !text.trim().is_empty() {
                units.push(Unit {
                    page: tab.page,
                    kind: "table",
                    text,
                    bbox: tab.bbox,
                    // no rows to point at: the table is named in the context, not the locator
                    table_id: None,
                    row: None,
                    context: json!({"structure": "text_only", "table_id": table_id, "caption": tab.caption}),
                    heading: false,
                });
            }
            return;
        }
        for (r, mut row) in rows {
            row.sort_by_key(|c| c.col.unwrap_or(0));
            let text = row
                .iter()
                .map(|c| c.text.trim())
                .collect::<Vec<_>>()
                .join(" | ");
            if text.trim_matches([' ', '|']).is_empty() {
                continue;
            }
            units.push(Unit {
                page: tab.page,
                kind: "table_row",
                text,
                bbox: union(row.iter().map(|c| c.bbox)).unwrap_or(tab.bbox),
                table_id: Some(table_id.clone()),
                row: Some(r),
                context: json!({
                    "row": r, "n_rows": tab.n_rows, "n_cols": tab.n_cols, "header": header, "caption": tab.caption,
                    "cells": row.iter().map(|c| json!({"col": c.col, "row_span": c.row_span,
                        "col_span": c.col_span, "text": c.text, "bbox": c.bbox})).collect::<Vec<_>>(),
                }),
                heading: false,
            });
        }
    };
    let mut figures_by_page: BTreeMap<i64, Vec<&ViewFigure>> = BTreeMap::new();
    for f in &view.figures {
        figures_by_page.entry(f.page).or_default().push(f);
    }
    for it in &view.items {
        if it.kind == "table_cell" {
            if let Some(t) = it.table {
                emit_table(t, &mut units);
            }
            continue;
        }
        if it.text.trim().is_empty() {
            continue;
        }
        // A caption that is also a figure's/table's caption is cited through that object.
        if it.kind == "caption" && captions.contains(&(it.page, it.text.clone())) {
            continue;
        }
        let heading = matches!(it.kind.as_str(), "section_header" | "title");
        units.push(Unit {
            page: it.page,
            kind: if heading {
                "heading"
            } else if it.kind == "caption" {
                "caption"
            } else {
                "text"
            },
            text: it.text.clone(),
            bbox: it.bbox,
            table_id: None,
            row: None,
            context: json!({"label": it.kind}),
            heading,
        });
    }
    for t in tables.keys().copied().collect::<Vec<_>>() {
        emit_table(t, &mut units); // tables without cells in the item stream
    }
    for f in &view.figures {
        if let Some(c) = f.caption.clone().filter(|c| !c.trim().is_empty()) {
            units.push(Unit {
                page: f.page,
                kind: "figure",
                text: c,
                bbox: f.bbox,
                table_id: None,
                row: None,
                context: json!({"figure": f.figure}),
                heading: false,
            });
        }
    }
    // Stable page order; within a page, reading order as produced above.
    units.sort_by_key(|u| u.page);

    let pub_id = format!("pub-{}", &sha[..16]);
    let pv = parser_version(view);
    let view_id = parse_view_id(&pub_id, &pv);
    let file_name = pdf_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| view.source.file_name.clone());
    let mut pages: Vec<String> = vec![String::new(); n_pages as usize];
    let mut lens: Vec<usize> = vec![0; n_pages as usize];
    let mut chunks = Vec::new();
    let mut section: Option<String> = None;
    let mut carry: Option<String> = None;
    let mut all_runs = std::collections::BTreeSet::new();
    for u in units {
        if u.heading {
            section = Some(u.text.trim().to_string());
        }
        let i = (u.page - 1) as usize;
        let cs = lens[i];
        let ce = cs + u.text.chars().count();
        pages[i].push_str(&u.text);
        pages[i].push('\n');
        lens[i] = ce + 1;
        let runs = rp.find(&u.text);
        all_runs.extend(runs.iter().cloned());
        let continues_runs: Vec<String> = carry.iter().cloned().collect();
        if let Some(l) = rp.last(&u.text) {
            carry = Some(l);
        }
        let outcome = !section
            .as_deref()
            .is_some_and(|s| !s.is_empty() && non_outcome_rx().is_match(s));
        let ordinal = chunks.len();
        let mut context = u.context;
        if let Some(figs) = figures_by_page.get(&u.page) {
            context["figures"] = json!(figs
                .iter()
                .map(|f| json!({"bbox": f.bbox, "caption": f.caption}))
                .collect::<Vec<_>>());
        }
        chunks.push(Chunk {
            id: format!("{view_id}:p{}:{ordinal:05}", u.page),
            publication_id: pub_id.clone(),
            ordinal: ordinal as i64,
            kind: u.kind.into(),
            page: u.page,
            char_start: Some(cs as i64),
            char_end: Some(ce as i64),
            section: section.clone(),
            text: u.text,
            context,
            locator: SourceLocator {
                asset_sha256: sha.clone(),
                path: Some(file_name.clone()),
                page: Some(u.page),
                char_start: Some(cs as i64),
                char_end: Some(ce as i64),
                parse_view_id: Some(view_id.clone()),
                offset_unit: Some(OffsetUnit::UnicodeScalar),
                bbox: Some(u.bbox),
                table_id: u.table_id,
                row: u.row,
                ..Default::default()
            },
            describes_runs: runs,
            continues_runs,
            outcome_bearing: outcome,
            evidence_class: "source_derived".into(),
        });
    }
    let head: String = pages.join(" ").chars().take(20000).collect();
    let publication = Publication {
        id: pub_id,
        source_sha256: sha,
        title: Some(
            pdf_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
        ),
        authors: vec![],
        year: None,
        ntrs_id: None,
        doi: None,
        canonical_url: None,
        investigation_ids: publication_investigations(pdf_path, investigations),
        families: families_in(&head),
        file_name: Some(file_name),
        n_pages,
        rights_status: "unverified".into(),
        license: None,
        describes_runs: all_runs.into_iter().collect(),
        outcome_bearing: true,
        ingested_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false),
        parser_version: pv,
    };
    Ok(ImportedView {
        publication,
        chunks,
        pages,
    })
}

//! Citation verification: re-derive a stored parse view from the original bytes.
//!
//! A citation is verifiable only against the asset and parse view it names. [`verify_view`] checks,
//! for one publication: the bytes hash to the asset id; the parser that wrote the view is the one
//! running (otherwise the page text cannot be re-derived and the view is reported stale, not
//! "verified"); every stored page text equals the text MuPDF extracts now; every chunk is exactly
//! `page_text[char_start..char_end]`; every locator passes the C0 contract checks
//! ([`SourceLocator::validate`](ignis_core::contracts::SourceLocator::validate)).

use anyhow::Result;
use ignis_core::hashing::sha256_hex;
use mupdf::Document;
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::catalog;
use crate::pdf::page_text;
use crate::records::{parse_view_id, Publication};
use crate::PARSER_VERSION;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ViewCheck {
    pub publication_id: String,
    pub source_id: String,
    pub parse_view_id: String,
    pub bytes_match: bool,
    /// false when the view was written by another parser version (not re-derivable here)
    pub parser_current: bool,
    pub pages_checked: usize,
    pub page_mismatches: Vec<i64>,
    pub chunks_checked: usize,
    pub chunk_mismatches: Vec<String>,
    pub invalid_locators: Vec<String>,
    pub chunks_with_bbox: usize,
}

impl ViewCheck {
    pub fn passed(&self) -> bool {
        self.bytes_match
            && self.parser_current
            && self.page_mismatches.is_empty()
            && self.chunk_mismatches.is_empty()
            && self.invalid_locators.is_empty()
    }
}

/// Check the stored parse view of `p` against `bytes` (the original asset).
pub fn verify_view(con: &Connection, p: &Publication, bytes: &[u8]) -> Result<ViewCheck> {
    let mut r = ViewCheck {
        publication_id: p.id.clone(),
        source_id: format!("sha256:{}", p.source_sha256),
        parse_view_id: parse_view_id(&p.id, &p.parser_version),
        bytes_match: sha256_hex(bytes) == p.source_sha256,
        parser_current: p.parser_version == PARSER_VERSION,
        ..Default::default()
    };
    if !r.bytes_match || !r.parser_current {
        return Ok(r);
    }
    let doc = Document::from_bytes(bytes, "application/pdf")?;
    let mut texts: Vec<Vec<char>> = Vec::new();
    for pno in 1..=doc.page_count()? as usize {
        texts.push(page_text(&doc, pno)?.chars().collect());
    }
    for (page, stored) in catalog::page_texts(con, &p.id, 1, i64::MAX)? {
        r.pages_checked += 1;
        let now = texts.get(page as usize - 1);
        if now.is_none_or(|t| t.iter().collect::<String>() != stored) {
            r.page_mismatches.push(page);
        }
    }
    let mut st = con.prepare(
        "SELECT id, page, char_start, char_end, text, locator_json FROM chunks \
         WHERE publication_id = ?1 ORDER BY ordinal",
    )?;
    let rows = st.query_map(params![p.id], |x| {
        Ok((
            x.get::<_, String>(0)?,
            x.get::<_, i64>(1)?,
            x.get::<_, Option<i64>>(2)?,
            x.get::<_, Option<i64>>(3)?,
            x.get::<_, String>(4)?,
            x.get::<_, String>(5)?,
        ))
    })?;
    for row in rows {
        let (id, page, a, b, text, loc) = row?;
        r.chunks_checked += 1;
        let loc: ignis_core::contracts::SourceLocator = serde_json::from_str(&loc)?;
        let loc_ok = loc.validate().is_ok()
            && loc.asset_sha256 == p.source_sha256
            && loc.page == Some(page)
            && loc.parse_view_id.as_deref() == Some(r.parse_view_id.as_str())
            && (loc.char_start, loc.char_end) == (a, b);
        if !loc_ok {
            r.invalid_locators.push(id.clone());
        }
        if loc.bbox.is_some() {
            r.chunks_with_bbox += 1;
        }
        if let (Some(a), Some(b)) = (a, b) {
            let span: Option<String> = texts
                .get(page as usize - 1)
                .and_then(|t| t.get(a as usize..b as usize))
                .map(|c| c.iter().collect());
            if span.as_deref() != Some(text.as_str()) {
                r.chunk_mismatches.push(id);
            }
        }
    }
    Ok(r)
}

//! Attribution rules shared by every parser (MuPDF views and imported Docling views): which runs a
//! text names (`RunPatterns`), which sections are not outcome-bearing, which evidence families a
//! document's text hints at, and which investigation a stored document belongs to. No PDF library
//! is needed here, so imported parse views use exactly the rules MuPDF views use.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;

pub fn caption_rx() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?i)^\s*(fig\.?|figure|table)\s*[0-9IVX]+[a-z]?\b").unwrap())
}

pub fn heading_rx() -> &'static Regex {
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
pub fn non_outcome_rx() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)introduction|background|nomenclature|apparatus|hardware|method|setup|references|bibliography|acknowledg")
            .unwrap()
    })
}

pub const FAMILY_HINTS: [(&str, &str); 3] = [
    ("acme", r"\bACME\b|gaseous flame"),
    ("bass", r"\bBASS\b|flame spread over|solid fuel"),
    ("flex", r"\bFLEX\b|droplet"),
];

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

/// Investigations a stored PSI document belongs to: the caller's, plus the one its storage location
/// names (`raw/samples/<PSI-xx>/...`, or a `<PSI-xx>_` file-name prefix). The lockbox scope of a
/// publication then never depends on the caller remembering `--investigation` or on keyword hints:
/// an unattributed outcome passage of a held-out investigation's document is withheld by construction.
pub fn publication_investigations(path: &Path, given: &[String]) -> Vec<String> {
    let is_psi = |s: &str| {
        s.strip_prefix("PSI-")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    };
    let mut out: Vec<String> = given.to_vec();
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string());
    let prefix = path
        .file_name()
        .and_then(|s| s.to_string_lossy().split('_').next().map(String::from));
    for cand in [parent, prefix].into_iter().flatten() {
        if is_psi(&cand) && !out.contains(&cand) {
            out.push(cand);
        }
    }
    out
}

/// Evidence families whose hints appear in `head` (the first pages of a document).
pub fn families_in(head: &str) -> Vec<String> {
    FAMILY_HINTS
        .iter()
        .filter(|(_, rx)| Regex::new(rx).unwrap().is_match(head))
        .map(|(f, _)| f.to_string())
        .collect()
}

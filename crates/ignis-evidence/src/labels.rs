//! Document labels (R8): a two-pass model labeller producing [`SourceLabel`]s, the gold-set tooling
//! that measures it, and nothing that runs a model without `--execute`.
//!
//! **Two passes.** Each document is labelled twice from two different views of it, so the passes
//! are independent: pass `a` sees the first pages and the section outline (R7b); pass `b` sees the
//! outline, the middle pages and the last pages. A label is `resolved` only when both passes agree
//! on the document type and the investigations; otherwise it is `unresolved_conflict` and keeps
//! pass `a`'s answer visible with the disagreement (contract rule: a model label is resolved only by
//! two agreeing passes). Labels describe; the runs they name are mentions, never identities.
//!
//! **Pluggable model.** The passes are [`crate::llm_pass::Request`]s answered by any
//! [`crate::llm_pass::Model`]; `ignis evidence labels --execute` uses `claude -p`. Answers are
//! appended to `label_answers.jsonl` (resumable); [`build_labels`] turns them into `SourceLabel`s.
//!
//! **Gold set.** [`gold_select`] picks ~100 documents stratified across the 19 investigations and
//! document kinds (a deterministic, seeded order; no randomness at run time), and
//! [`gold_template`] writes them as rows with **empty** labels for a person to fill in: no gold label
//! is ever produced by code. [`gold_eval`] scores a labeller's `SourceLabel`s against the filled rows
//! (document-type accuracy, investigation-set accuracy, family F1), skipping unfilled rows.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use ignis_core::contracts::{
    DocType, EvidenceSchema, Family, Finite, LabelAgreement, LabelCounts, Labeller, Method, Origin,
    Quantity, Resolution, RunId, SourceId, SourceLabel,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::llm_pass::{Answer, Request, Task};
use crate::records::Publication;

/// Static prefix of a label request (its digest is the labels' prompt hash).
pub const LABEL_SYSTEM: &str = r#"You classify one document of NASA's microgravity-combustion archive (PSI flight investigations and NTRS reports).
Answer with one JSON object and nothing else:
{"doc_type": one of "technical_report","journal_article","conference_paper","presentation","thesis","requirements_document","engineering_document","operations_document","data_description","data_table","outreach","other",
 "families": subset of ["flex","acme_normal","acme_inverse","bass","other"],
 "investigation_ids": PSI accessions the document is about (e.g. "PSI-69"), [] if none is clear,
 "runs": run or test identifiers the document names (e.g. "FLEX-039", "19115A1"), as written,
 "conditions_mentioned": subset of ["pressure","mole_fraction","temperature","flow_velocity","gravity_level","mass_flow","volume_flow","std_volume_flow"],
 "confidence": a number in [0,1]}
Only use what the text shows; when unsure, use "other" and empty lists. Never guess an accession from a topic alone."#;

/// Output budget of one label pass.
pub const LABEL_MAX_TOKENS: u32 = 300;

/// What one pass answered.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LabelDraft {
    pub doc_type: DocType,
    #[serde(default)]
    pub families: BTreeSet<Family>,
    #[serde(default)]
    pub investigation_ids: BTreeSet<String>,
    #[serde(default)]
    pub runs: Vec<String>,
    #[serde(default)]
    pub conditions_mentioned: BTreeSet<Quantity>,
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// The JSON object in a model answer (the first `{` to its matching `}`), parsed as a draft.
pub fn parse_label(text: &str) -> Result<LabelDraft> {
    let start = text.find('{').context("no JSON object in the answer")?;
    let mut depth = 0i32;
    let mut end = None;
    let mut in_str = false;
    let mut esc = false;
    for (i, c) in text[start..].char_indices() {
        if in_str {
            match (esc, c) {
                (true, _) => esc = false,
                (false, '\\') => esc = true,
                (false, '"') => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(start + i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.context("unterminated JSON object in the answer")?;
    let mut d: LabelDraft =
        serde_json::from_str(&text[start..end]).context("the answer is not a label object")?;
    d.investigation_ids = d
        .investigation_ids
        .into_iter()
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    if let Some(c) = d.confidence {
        if !(0.0..=1.0).contains(&c) {
            d.confidence = None;
        }
    }
    Ok(d)
}

fn outline_titles(con: &Connection, p: &Publication) -> Result<String> {
    let chunks = crate::outline::load_chunks(con, &p.id)?;
    let (heads, _) = crate::outline::headings(&chunks);
    Ok(heads
        .iter()
        .map(|(page, level, t)| format!("{}{t} (p. {page})", "  ".repeat(level.saturating_sub(1))))
        .collect::<Vec<_>>()
        .join("\n"))
}

fn pages_text(con: &Connection, p: &Publication, pages: &[i64]) -> Result<String> {
    let view = crate::records::parse_view_id(&p.id, &p.parser_version);
    let mut out = Vec::new();
    for &n in pages {
        if n < 1 || n > p.n_pages {
            continue;
        }
        for (pg, t) in crate::catalog::page_texts(con, &p.id, &view, n, n)? {
            out.push(format!("[page {pg}]\n{t}"));
        }
    }
    Ok(out.join("\n"))
}

/// The pages each pass sees: `a` the first three; `b` two around the middle and the last two.
pub fn pass_pages(n_pages: i64) -> (Vec<i64>, Vec<i64>) {
    let a: Vec<i64> = (1..=n_pages.min(3)).collect();
    let mid = (n_pages + 1) / 2;
    let mut b: BTreeSet<i64> = [mid, mid + 1, n_pages - 1, n_pages]
        .into_iter()
        .filter(|p| *p >= 1 && *p <= n_pages)
        .collect();
    if b.is_empty() {
        b.insert(1);
    }
    (a, b.into_iter().collect())
}

/// Two requests per publication: `<sha256>#a` and `<sha256>#b`.
pub fn label_requests(con: &Connection, pubs: &[Publication]) -> Result<Vec<Request>> {
    let mut out = Vec::new();
    for p in pubs {
        let outline = outline_titles(con, p)?;
        let (a, b) = pass_pages(p.n_pages);
        let head = format!(
            "File: {}\nTitle: {}\nPages: {}\nOutline:\n{}\n",
            p.file_name.clone().unwrap_or_default(),
            p.title.clone().unwrap_or_default(),
            p.n_pages,
            if outline.is_empty() {
                "(none)".into()
            } else {
                outline
            }
        );
        for (pass, pages) in [("a", a), ("b", b)] {
            out.push(Request {
                custom_id: format!("{}#{pass}", p.source_sha256),
                task: Task::SourceLabel,
                system: LABEL_SYSTEM.into(),
                user: format!("{head}\n{}", pages_text(con, p, &pages)?),
                max_output_tokens: LABEL_MAX_TOKENS,
            });
        }
    }
    Ok(out)
}

/// Result of turning answers into labels.
#[derive(Debug)]
pub struct Built {
    pub labels: Vec<SourceLabel>,
    pub counts: LabelCounts,
    /// Documents with an unusable answer (unparseable or missing pass), and why.
    pub problems: Vec<(String, String)>,
}

/// Combines the two passes of every document whose answers are present.
pub fn build_labels(answers: &[Answer]) -> Result<Built> {
    let mut by_doc: BTreeMap<String, BTreeMap<String, &Answer>> = BTreeMap::new();
    for a in answers.iter().filter(|a| a.task == Task::SourceLabel) {
        let Some((sha, pass)) = a.custom_id.split_once('#') else {
            continue;
        };
        by_doc
            .entry(sha.to_string())
            .or_default()
            .insert(pass.to_string(), a);
    }
    let mut labels = Vec::new();
    let mut problems = Vec::new();
    for (sha, passes) in by_doc {
        let (Some(a), Some(b)) = (passes.get("a"), passes.get("b")) else {
            problems.push((sha, "one pass is missing".into()));
            continue;
        };
        if a.model_id != b.model_id {
            problems.push((sha, "the passes used different models".into()));
            continue;
        }
        // Pass a may come from the combined pass (another prompt); the label names both prompts.
        let prompt = if a.prompt_hash == b.prompt_hash {
            a.prompt_hash.clone()
        } else {
            ignis_core::contracts::Digest::of_bytes(
                format!("{}+{}", a.prompt_hash, b.prompt_hash).as_bytes(),
            )
        };
        let (da, db) = match (parse_label(&a.text), parse_label(&b.text)) {
            (Ok(x), Ok(y)) => (x, y),
            (Err(e), _) | (_, Err(e)) => {
                problems.push((sha, format!("{e:#}")));
                continue;
            }
        };
        let agreed = da.doc_type == db.doc_type && da.investigation_ids == db.investigation_ids;
        let confidence = match (da.confidence, db.confidence) {
            (Some(x), Some(y)) => Some(x.min(y)),
            (x, y) => x.or(y),
        };
        // Mentions both passes saw; a disagreement keeps pass a's answer, flagged.
        let mut runs: BTreeSet<String> = da.runs.iter().cloned().collect();
        if agreed {
            runs.extend(db.runs.iter().cloned());
        }
        let label = SourceLabel {
            schema_version: EvidenceSchema::V2,
            source_id: SourceId::new(format!("sha256:{sha}"))?,
            doc_type: da.doc_type,
            families: if agreed {
                da.families.intersection(&db.families).copied().collect()
            } else {
                da.families.clone()
            },
            investigation_ids: da.investigation_ids.clone(),
            runs: runs
                .into_iter()
                .filter_map(|r| RunId::new(r.replace(char::is_whitespace, "_")).ok())
                .collect(),
            conditions_mentioned: if agreed {
                da.conditions_mentioned
                    .intersection(&db.conditions_mentioned)
                    .copied()
                    .collect()
            } else {
                da.conditions_mentioned.clone()
            },
            evidence: vec![],
            labeller: Labeller {
                method: Method::Semantic,
                model_id: Some(a.model_id.clone()),
                prompt_hash: Some(prompt),
            },
            confidence: confidence.and_then(|c| Finite::new(c).ok()),
            agreement: LabelAgreement { passes: 2, agreed },
            status: if agreed {
                Resolution::Resolved
            } else {
                Resolution::UnresolvedConflict
            },
            origin: Origin::System,
        };
        label.validate()?;
        labels.push(label);
    }
    let resolved = labels
        .iter()
        .filter(|l| l.status == Resolution::Resolved)
        .count() as u64;
    let counts = LabelCounts {
        total: labels.len() as u64,
        resolved,
        unresolved_conflict: labels.len() as u64 - resolved,
    };
    Ok(Built {
        labels,
        counts,
        problems,
    })
}

// ------------------------------------------------------------------ gold set

/// A document's kind as its file name suggests (only for stratifying the gold selection; never a
/// label).
pub fn kind_hint(p: &Publication) -> &'static str {
    let n = p.file_name.clone().unwrap_or_default().to_lowercase();
    let has = |ks: &[&str]| ks.iter().any(|k| n.contains(k));
    if has(&[".ppt", "presentation", "slides", "briefing"]) {
        "presentation"
    } else if has(&["thesis", "dissertation"]) {
        "thesis"
    } else if has(&["srd", "requirement", "test matrix", "test_matrix"]) {
        "requirements"
    } else if has(&["procedure", "operations", "timeline", "as run"]) {
        "operations"
    } else if has(&["readme", "data dictionary", "description"]) {
        "data_description"
    } else if has(&["ntrs-", "report", "_tm", "_tp", "_cr", "nasa"]) {
        "report_or_paper"
    } else {
        "other"
    }
}

/// One row of the gold labelling template. The label fields stay empty until a person fills them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldRow {
    pub source_id: String,
    pub file_name: Option<String>,
    pub n_pages: i64,
    /// Strata the row was selected from (navigation for the labeller, not a label).
    pub stratum_investigation: String,
    pub stratum_kind: String,
    /// Gold fields: null / empty until labelled by a person.
    pub doc_type: Option<DocType>,
    #[serde(default)]
    pub families: BTreeSet<Family>,
    #[serde(default)]
    pub investigation_ids: BTreeSet<String>,
    #[serde(default)]
    pub runs: Vec<String>,
    /// Who labelled the row (a person), and when; null until then.
    pub labelled_by: Option<String>,
    pub labelled_at: Option<String>,
    #[serde(default)]
    pub notes: String,
}

/// `n` documents stratified across (investigation, kind hint): strata are visited round-robin in
/// a fixed order, and within a stratum documents are ordered by `sha256(seed, source)`, so the
/// selection is reproducible and does not favour any file order.
pub fn gold_select(pubs: &[Publication], n: usize, seed: u64) -> Vec<GoldRow> {
    let mut strata: BTreeMap<(String, String), Vec<&Publication>> = BTreeMap::new();
    for p in pubs {
        let inv = p
            .investigation_ids
            .first()
            .cloned()
            .unwrap_or_else(|| "none".into());
        strata
            .entry((inv, kind_hint(p).to_string()))
            .or_default()
            .push(p);
    }
    let key = |p: &Publication| hex::encode(Sha256::digest(format!("{seed}:{}", p.source_sha256)));
    for v in strata.values_mut() {
        v.sort_by_key(|p| key(p));
    }
    let mut out = Vec::new();
    let mut round = 0;
    while out.len() < n {
        let mut took = false;
        for ((inv, kind), v) in &strata {
            if out.len() == n {
                break;
            }
            if let Some(p) = v.get(round) {
                out.push(GoldRow {
                    source_id: format!("sha256:{}", p.source_sha256),
                    file_name: p.file_name.clone(),
                    n_pages: p.n_pages,
                    stratum_investigation: inv.clone(),
                    stratum_kind: kind.clone(),
                    doc_type: None,
                    families: BTreeSet::new(),
                    investigation_ids: BTreeSet::new(),
                    runs: vec![],
                    labelled_by: None,
                    labelled_at: None,
                    notes: String::new(),
                });
                took = true;
            }
        }
        if !took {
            break;
        }
        round += 1;
    }
    out
}

/// Reads a gold file (JSONL of [`GoldRow`]).
pub fn read_gold(path: &std::path::Path) -> Result<Vec<GoldRow>> {
    let text = std::fs::read_to_string(path).with_context(|| path.display().to_string())?;
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| {
            serde_json::from_str(l).with_context(|| format!("{} line {}", path.display(), i + 1))
        })
        .collect()
}

/// Scores of a labeller on the filled gold rows.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GoldScore {
    /// Rows a person labelled (doc_type set and `labelled_by` named).
    pub labelled: usize,
    pub unlabelled: usize,
    /// Labelled rows the labeller has a label for.
    pub scored: usize,
    pub missing_labels: usize,
    pub doc_type_accuracy: Option<f64>,
    /// Exact match of the investigation sets.
    pub investigation_accuracy: Option<f64>,
    /// Micro-averaged F1 over families.
    pub family_f1: Option<f64>,
    /// Scores over the labeller's `resolved` labels only, and how many that is.
    pub resolved_scored: usize,
    pub resolved_doc_type_accuracy: Option<f64>,
    pub confusion: BTreeMap<String, BTreeMap<String, usize>>,
}

fn enum_str<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

/// Scores `labels` against the person-labelled rows of `gold`. Refuses when nothing is labelled.
pub fn gold_eval(gold: &[GoldRow], labels: &[SourceLabel]) -> Result<GoldScore> {
    let filled: Vec<&GoldRow> = gold
        .iter()
        .filter(|g| {
            g.doc_type.is_some()
                && g.labelled_by
                    .as_deref()
                    .is_some_and(|s| !s.trim().is_empty())
        })
        .collect();
    if filled.is_empty() {
        bail!(
            "the gold set has no labelled rows ({} rows, all empty): a person fills doc_type, investigation_ids, families and labelled_by first",
            gold.len()
        );
    }
    let by_source: BTreeMap<&str, &SourceLabel> =
        labels.iter().map(|l| (l.source_id.as_str(), l)).collect();
    let (mut scored, mut missing, mut dt, mut inv) = (0usize, 0usize, 0usize, 0usize);
    let (mut tp, mut fp, mut fneg) = (0usize, 0usize, 0usize);
    let (mut rs, mut rdt) = (0usize, 0usize);
    let mut confusion: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for g in &filled {
        let Some(l) = by_source.get(g.source_id.as_str()) else {
            missing += 1;
            continue;
        };
        scored += 1;
        let gd = g.doc_type.unwrap();
        let ok = gd == l.doc_type;
        dt += ok as usize;
        if l.status == Resolution::Resolved {
            rs += 1;
            rdt += ok as usize;
        }
        *confusion
            .entry(enum_str(&gd))
            .or_default()
            .entry(enum_str(&l.doc_type))
            .or_default() += 1;
        let gi: BTreeSet<String> = g
            .investigation_ids
            .iter()
            .map(|s| s.to_uppercase())
            .collect();
        inv += (gi == l.investigation_ids) as usize;
        tp += g.families.intersection(&l.families).count();
        fp += l.families.difference(&g.families).count();
        fneg += g.families.difference(&l.families).count();
    }
    let ratio = |a: usize, b: usize| (b > 0).then(|| a as f64 / b as f64);
    let f1 = if tp + fp + fneg == 0 {
        None
    } else {
        Some(2.0 * tp as f64 / (2 * tp + fp + fneg) as f64)
    };
    Ok(GoldScore {
        labelled: filled.len(),
        unlabelled: gold.len() - filled.len(),
        scored,
        missing_labels: missing,
        doc_type_accuracy: ratio(dt, scored),
        investigation_accuracy: ratio(inv, scored),
        family_f1: f1,
        resolved_scored: rs,
        resolved_doc_type_accuracy: ratio(rdt, rs),
        confusion,
    })
}

/// Reads `SourceLabel` JSONL.
pub fn read_labels(path: &std::path::Path) -> Result<Vec<SourceLabel>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    for (i, l) in text.lines().enumerate() {
        if l.trim().is_empty() {
            continue;
        }
        let v: SourceLabel = serde_json::from_str(l)
            .with_context(|| format!("{} line {}", path.display(), i + 1))?;
        v.validate()?;
        out.push(v);
    }
    Ok(out)
}

/// Keeps `Value` in the public surface tidy for callers that print reports.
pub fn to_value<T: Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

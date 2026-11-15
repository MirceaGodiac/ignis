//! The combined model pass (R7b + R8 + R12b): one call per leaf section of every document returns
//! that section's summary, the expansion vocabulary of each of its pages, and, in a document's first
//! section, label pass `a`. Spending on it needs the user's approval; `ignis evidence llm-pass`
//! prints the estimate and runs only with `--execute` (`docs/implementation/LLM_PASS.md`).
//!
//! Why per section and not per chunk: a chunk is a paragraph or a table row (tens of thousands in
//! the corpus); a leaf section groups a few pages of them, so one call sees enough context to
//! summarise and to name vocabulary, the static prefix is paid once per section, and the call count
//! drops by an order of magnitude. Expansion terms are still produced per page, the unit
//! `ignis evidence enrich` loads.
//!
//! [`split`] turns the answers into what each consumer reads:
//! * section summaries → `outline_summaries.jsonl` (answers of task `section_summary`, read by
//!   `sources_outline`);
//! * expansion vocabulary → a pack directory (`pages.jsonl` + `MANIFEST.json`) for
//!   `ignis evidence enrich` (R12b), never cited;
//! * label pass `a` → `label_answers.jsonl` (task `source_label`), completed by
//!   `ignis evidence labels --execute`, which runs only the missing pass `b`.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::json;

use crate::llm_pass::{prompt_hash, Answer, Request, Task};
use crate::records::Publication;

/// Static prefix of a combined request.
pub const COMBINED_SYSTEM: &str = r#"You read one section of a NASA microgravity-combustion document (pages marked [page N]).
Answer with one JSON object and nothing else:
{"summary": 1 to 3 plain sentences saying what the section covers, never stating results or numbers as facts,
 "pages": [{"page": N, "expansion": up to 12 terms a researcher might search for that this page is about but may phrase differently (synonyms, abbreviations, instruments, materials, quantities)}],
 "label": only when asked, the label object described below, else null}
Label object (only when the request says LABEL: yes):
{"doc_type": one of "technical_report","journal_article","conference_paper","presentation","thesis","requirements_document","engineering_document","operations_document","data_description","data_table","outreach","other",
 "families": subset of ["flex","acme_normal","acme_inverse","bass","other"], "investigation_ids": PSI accessions the document is about, [] if unclear,
 "runs": run or test identifiers as written, "conditions_mentioned": subset of ["pressure","mole_fraction","temperature","flow_velocity","gravity_level","mass_flow","volume_flow","std_volume_flow"],
 "confidence": a number in [0,1]}
Only use what the text shows."#;

/// Output budget of one combined call.
pub const COMBINED_MAX_TOKENS: u32 = 900;

/// One request per leaf section; `custom_id` = `<parse view id>#<node id>#<sha256>[#label]`.
pub fn combined_requests(con: &Connection, pubs: &[Publication]) -> Result<Vec<Request>> {
    let mut out = Vec::new();
    for p in pubs {
        let view = crate::records::parse_view_id(&p.id, &p.parser_version);
        let chunks = crate::outline::load_chunks(con, &p.id)?;
        let (heads, _) = crate::outline::headings(&chunks);
        let title = p
            .title
            .clone()
            .or_else(|| p.file_name.clone())
            .unwrap_or_default();
        let roots = crate::outline::build(&heads, p.n_pages, &title)?;
        let mut leaves = Vec::new();
        collect_leaves(&roots, &mut leaves);
        for (i, n) in leaves.iter().enumerate() {
            let pages = crate::catalog::page_texts(
                con,
                &p.id,
                &view,
                n.page_start as i64,
                n.page_end as i64,
            )?;
            let text: Vec<String> = pages
                .into_iter()
                .map(|(pg, t)| format!("[page {pg}]\n{t}"))
                .collect();
            let label = i == 0;
            out.push(Request {
                custom_id: format!(
                    "{view}#{}#{}{}",
                    n.node_id,
                    p.source_sha256,
                    if label { "#label" } else { "" }
                ),
                task: Task::Combined,
                system: COMBINED_SYSTEM.into(),
                user: format!(
                    "Document: {title}\nFile: {}\nSection: {} (pages {}-{})\nLABEL: {}\n\n{}",
                    p.file_name.clone().unwrap_or_default(),
                    n.title,
                    n.page_start,
                    n.page_end,
                    if label { "yes" } else { "no" },
                    text.join("\n")
                ),
                max_output_tokens: COMBINED_MAX_TOKENS,
            });
        }
    }
    Ok(out)
}

fn collect_leaves<'a>(
    nodes: &'a [ignis_core::contracts::OutlineNode],
    out: &mut Vec<&'a ignis_core::contracts::OutlineNode>,
) {
    for n in nodes {
        if n.children.is_empty() {
            out.push(n);
        } else {
            collect_leaves(&n.children, out);
        }
    }
}

#[derive(Deserialize)]
struct CombinedAnswer {
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    pages: Vec<PageTerms>,
    #[serde(default)]
    label: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct PageTerms {
    page: i64,
    #[serde(default)]
    expansion: Vec<String>,
}

/// What [`split`] wrote.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SplitReport {
    pub summaries: usize,
    pub pack_pages: usize,
    pub label_passes: usize,
    pub unparseable: Vec<String>,
}

/// Splits combined answers into summaries, an expansion pack and label pass-`a` answers.
pub fn split(
    answers: &[Answer],
    summaries_out: &Path,
    pack_dir: &Path,
    labels_out: &Path,
) -> Result<SplitReport> {
    let mut report = SplitReport {
        summaries: 0,
        pack_pages: 0,
        label_passes: 0,
        unparseable: vec![],
    };
    let mut summaries = Vec::new();
    let mut labels = Vec::new();
    let mut pages: BTreeMap<(String, i64), Vec<String>> = BTreeMap::new();
    for a in answers.iter().filter(|a| a.task == Task::Combined) {
        let parts: Vec<&str> = a.custom_id.split('#').collect();
        let (view, node, sha) = match parts.as_slice() {
            [v, n, s, ..] => (*v, *n, *s),
            _ => {
                report.unparseable.push(a.custom_id.clone());
                continue;
            }
        };
        let json_text = a
            .text
            .find('{')
            .and_then(|s| a.text.rfind('}').map(|e| &a.text[s..=e]));
        let Some(c) = json_text.and_then(|t| serde_json::from_str::<CombinedAnswer>(t).ok()) else {
            report.unparseable.push(a.custom_id.clone());
            continue;
        };
        if let Some(s) = c.summary.filter(|s| !s.trim().is_empty()) {
            summaries.push(Answer {
                custom_id: format!("{view}#{node}"),
                task: Task::SectionSummary,
                model_id: a.model_id.clone(),
                prompt_hash: a.prompt_hash.clone(),
                text: s,
            });
        }
        for pt in c.pages {
            let e = pages.entry((sha.to_string(), pt.page)).or_default();
            for t in pt.expansion {
                let t = t.trim().to_string();
                if !t.is_empty() && !e.contains(&t) {
                    e.push(t);
                }
            }
        }
        if parts.get(3) == Some(&"label") {
            if let Some(l) = c.label.filter(|l| !l.is_null()) {
                labels.push(Answer {
                    custom_id: format!("{sha}#a"),
                    task: Task::SourceLabel,
                    model_id: a.model_id.clone(),
                    // The prompt that produced this pass (the combined one); `build_labels` names
                    // both prompts when pass b used the label prompt.
                    prompt_hash: a.prompt_hash.clone(),
                    text: l.to_string(),
                });
            }
        }
    }
    let append = |path: &Path, rows: &[Answer]| -> Result<()> {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        for r in rows {
            serde_json::to_writer(&mut f, r)?;
            f.write_all(b"\n")?;
        }
        Ok(())
    };
    append(summaries_out, &summaries)?;
    append(labels_out, &labels)?;
    report.summaries = summaries.len();
    report.label_passes = labels.len();
    std::fs::create_dir_all(pack_dir)?;
    let mut f = std::io::BufWriter::new(
        std::fs::File::create(pack_dir.join("pages.jsonl")).context("pack pages.jsonl")?,
    );
    for ((sha, page), terms) in &pages {
        serde_json::to_writer(
            &mut f,
            &json!({"source_sha256": [sha], "page": page, "expansion": terms, "outcome_bearing": false}),
        )?;
        f.write_all(b"\n")?;
    }
    f.flush()?;
    report.pack_pages = pages.len();
    let model_ids: std::collections::BTreeSet<&str> =
        answers.iter().map(|a| a.model_id.as_str()).collect();
    std::fs::write(
        pack_dir.join("MANIFEST.json"),
        serde_json::to_vec_pretty(&json!({
            "producer": "ignis evidence llm-pass (combined)",
            "prompt_hash": prompt_hash(COMBINED_SYSTEM),
            "models": model_ids,
            "pages": pages.len(),
            "note": "generated vocabulary for retrieval only; never quoted or cited",
        }))?,
    )?;
    Ok(report)
}

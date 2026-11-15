//! Retrieval evaluation and the τ/w sweep of SIRA (LANES_V2 R12b): nDCG@10 and recall@10 per
//! retrieval question type (single-hop, multi-hop, figure) for four arms over the same catalog:
//!
//! | arm | score |
//! |---|---|
//! | `bm25` | `bm25(text; query)` |
//! | `query_expansion` | `bm25(text; query) + Σ w_e·bm25(text; e)` (the agent's `expected_terms`) |
//! | `doc_expansion` | `bm25(text; query) + w·bm25(expansion; query)` (R12b, at τ) |
//! | `query_doc_expansion` | both |
//!
//! **Dev split only.** The task file, the expected-terms file, the pack, the catalog and the output
//! are refused under a `holdout/` directory; a task file holding any task whose `split` is not
//! `dev` (or a `GoldSet` whose split is not `dev`) is refused as a whole; an evaluation scope, or any
//! scope that grants the test partition, is refused ([`check_scope`]). The hold-out is scored once,
//! by the evaluator.
//!
//! **Tasks.** The T10 format: a JSON array of `GoldTask` (`benchmarks/t10/dev/tasks.json`), a
//! `GoldSet` object (`.../set.json`) or JSONL of `GoldTask`. [`GoldTaskView`] reads only what this
//! tool needs (task id, split, question, retrieval kind, the answer key's cited locators, origin);
//! it is a read-only projection of K1.14's `GoldTask`, which is on `dev` but not yet on `main`. When
//! the contract reaches this branch, the projection is replaced by
//! `ignis_core::contracts::GoldTask` and its `validate()`.
//!
//! **Relevance** ([`relevant`]): a hit is relevant to a cited locator (`answer.support` of a
//! documentary task, `answer.accepted` of a citation task) when both name the same file
//! (`asset_sha256`) and
//! 1. *span*: the cited locator has `parse_view_id`, `char_start` and `char_end`, the hit has the
//!    same parse view and a span, and the two half-open spans overlap on the same page; else
//! 2. *page*: the cited locator has a page and the hit is on that page; else
//! 3. *table row*: the cited locator has a row (and, if given, a `table_id`) and the hit is that row
//!    of that table (the column is not compared: a row chunk holds every cell); else
//! 4. *path*: the cited locator has a path and the hit carries the same path.
//!
//! A cited locator with none of these grades nothing (`ungradable`); one whose file is not in the
//! catalog is counted as `asset_not_in_catalog`. Both are reported, never scored as zero: a task
//! with no gradable locator is excluded with its reason.
//!
//! **Metrics** ([`ndcg_recall`]): binary gain; a hit gains 1 when it covers at least one cited
//! locator no higher hit covered (several chunks of one cited page count once);
//! `IDCG = Σ_{i<min(n,10)} 1/log2(i+2)` for `n` gradable cited locators; `recall@10` = covered / `n`.
//! When one chunk covers two cited locators, nDCG understates (recall does not).
//!
//! **Sweep.** τ changes the index, so each τ re-enriches a working copy of the catalog
//! (`VACUUM INTO <out>/work.sqlite`, deleted afterwards) from the pack; the input catalog is opened
//! read-only and never changed. w is a query argument ([`crate::catalog::lexical_weighted`]). The
//! chosen pair is written as a **proposal** (`proposal.json`); nothing is applied to `sira.rs` or
//! `configs/`. Numbers from a task set with `origin = synthetic_fixture` are never results.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use ignis_core::contracts::{
    ExecutionScope, ExpectedTerm, Origin, Partition, ScopeRole, SourceLocator, TermImportance,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{catalog, sira};

/// Cut-off of both metrics.
pub const K: usize = 10;
/// Default grids (comma lists on the command line override them).
pub const DEFAULT_TAUS: &[f64] = &[0.01, 0.02, 0.05, 0.1, 0.2];
pub const DEFAULT_WS: &[f64] = &[0.1, 0.2, 0.4, 0.6, 0.8, 1.0, 2.0, 4.0];

/// Retrieval sub-set of a gold task (K1.14 `RetrievalKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalKind {
    SingleHop,
    MultiHop,
    Figure,
}

/// K1.14 `GoldSplit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoldSplit {
    Dev,
    Holdout,
}

/// The fields of a K1.14 `GoldTask` this tool reads (see the module note).
#[derive(Debug, Clone, Deserialize)]
pub struct GoldTaskView {
    pub task_id: String,
    pub split: GoldSplit,
    pub question: String,
    #[serde(default)]
    pub retrieval: Option<RetrievalKind>,
    pub answer: Value,
    pub origin: Origin,
}

#[derive(Debug, Clone, Deserialize)]
struct GoldEvidenceView {
    locator: SourceLocator,
}

impl GoldTaskView {
    /// The cited locators of the answer key (documentary `support`, citation `accepted`),
    /// deduplicated. Analytical and predictive tasks cite none.
    pub fn cited(&self) -> Result<Vec<SourceLocator>> {
        let field = match self.answer.get("kind").and_then(Value::as_str) {
            Some("documentary") => "support",
            Some("citation") => "accepted",
            _ => return Ok(vec![]),
        };
        let ev: Vec<GoldEvidenceView> = match self.answer.get(field) {
            Some(v) => serde_json::from_value(v.clone())
                .with_context(|| format!("{}: answer.{field}", self.task_id))?,
            None => vec![],
        };
        let mut out: Vec<SourceLocator> = Vec::new();
        for e in ev {
            if !out.contains(&e.locator) {
                out.push(e.locator);
            }
        }
        Ok(out)
    }
}

/// Refuse a path under a `holdout/` directory, however it is spelled.
pub fn check_not_holdout(what: &str, path: &Path) -> Result<()> {
    if path.components().any(
        |c| matches!(c, Component::Normal(s) if s.to_string_lossy().eq_ignore_ascii_case("holdout")),
    ) {
        bail!(
            "{what} {} is under holdout/: the hold-out is read only by the evaluator",
            path.display()
        );
    }
    Ok(())
}

/// Refuse an evaluation scope and any scope that may read the test partition: tuning on what the
/// evaluator scores would turn the test into development.
pub fn check_scope(scope: &ExecutionScope) -> Result<()> {
    // `partitions.contains` as well: an invalid scope that lists test fails `require_partition`
    // (it fails validation) but must not pass here.
    if scope.role == ScopeRole::Evaluation
        || scope.partitions.contains(&Partition::Test)
        || scope.require_partition(Partition::Test).is_ok()
    {
        bail!(
            "retrieval-eval runs in a builder scope only: scope {} ({:?}) may read the test \
             partition; only the evaluator scores the hold-out",
            scope.scope_id.as_str(),
            scope.role
        );
    }
    Ok(())
}

/// Read a dev task file (JSON array, `GoldSet` object or JSONL); refuses hold-out tasks.
pub fn load_tasks(path: &Path) -> Result<Vec<GoldTaskView>> {
    check_not_holdout("task file", path)?;
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let tasks: Vec<GoldTaskView> = match serde_json::from_str::<Value>(&raw) {
        Ok(Value::Array(a)) => serde_json::from_value(Value::Array(a)).context("gold tasks")?,
        Ok(Value::Object(o)) => {
            if let Some(split) = o.get("split") {
                let s: GoldSplit =
                    serde_json::from_value(split.clone()).context("GoldSet.split")?;
                if s != GoldSplit::Dev {
                    bail!("{} is a hold-out gold set; refused", path.display());
                }
            }
            let Some(t) = o.get("tasks") else {
                bail!("{}: neither a task array nor a GoldSet", path.display());
            };
            serde_json::from_value(t.clone()).context("GoldSet.tasks")?
        }
        _ => {
            let mut v = Vec::new();
            for (i, line) in raw.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                v.push(serde_json::from_str(line).with_context(|| format!("line {}", i + 1))?);
            }
            v
        }
    };
    let held: Vec<&str> = tasks
        .iter()
        .filter(|t| t.split != GoldSplit::Dev)
        .map(|t| t.task_id.as_str())
        .collect();
    if !held.is_empty() {
        bail!(
            "{} holds {} task(s) outside the dev split ({}); the whole file is refused",
            path.display(),
            held.len(),
            held.join(", ")
        );
    }
    let mut ids = std::collections::HashSet::new();
    for t in &tasks {
        if !ids.insert(t.task_id.as_str()) {
            bail!("duplicate task id {}", t.task_id);
        }
    }
    Ok(tasks)
}

/// The agent's expected-response sketches per task: `{"<task_id>": [ExpectedTerm, ...]}`. Recorded
/// from the `expected_terms` of the agent's `sources_search` calls on the dev tasks (M10 traces) or
/// written by hand; this tool never calls a model to produce them.
pub fn load_expected_terms(path: &Path) -> Result<BTreeMap<String, Vec<ExpectedTerm>>> {
    check_not_holdout("expected-terms file", path)?;
    let raw = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&raw).context("expected terms: {\"<task_id>\": [{\"term\": ...}]}")
}

/// Does chunk `hit` (its catalog locator) support cited locator `gold`? The rule of the module note.
pub fn relevant(hit: &SourceLocator, gold: &SourceLocator) -> bool {
    if hit.asset_sha256 != gold.asset_sha256 {
        return false;
    }
    if let (Some(view), Some(gs), Some(ge)) = (&gold.parse_view_id, gold.char_start, gold.char_end)
    {
        if hit.parse_view_id.as_ref() == Some(view) {
            if let (Some(hs), Some(he)) = (hit.char_start, hit.char_end) {
                let same_page = match (gold.page, hit.page) {
                    (Some(a), Some(b)) => a == b,
                    _ => true,
                };
                return same_page && hs < ge && gs < he;
            }
        }
    }
    if let Some(p) = gold.page {
        return hit.page == Some(p);
    }
    if let Some(r) = gold.row {
        let same_table = gold.table_id.is_none() || hit.table_id == gold.table_id;
        return same_table && hit.row == Some(r);
    }
    if let Some(p) = &gold.path {
        return hit.path.as_ref() == Some(p);
    }
    false
}

/// A cited locator that can grade a hit at all (rules 1–4).
pub fn gradable(gold: &SourceLocator) -> bool {
    gold.page.is_some()
        || gold.row.is_some()
        || gold.path.is_some()
        || (gold.parse_view_id.is_some() && gold.char_start.is_some() && gold.char_end.is_some())
}

/// nDCG@k and recall@k of one ranking. `covers[i]` lists the cited locators (indices `< n`) hit `i`
/// covers, best hit first.
pub fn ndcg_recall(covers: &[Vec<usize>], n: usize, k: usize) -> (f64, f64) {
    if n == 0 {
        return (0.0, 0.0);
    }
    let mut seen = vec![false; n];
    let mut dcg = 0.0;
    for (i, c) in covers.iter().take(k).enumerate() {
        if c.iter().any(|&g| !seen[g]) {
            dcg += 1.0 / ((i + 2) as f64).log2();
        }
        for &g in c {
            seen[g] = true;
        }
    }
    let idcg: f64 = (0..n.min(k)).map(|i| 1.0 / ((i + 2) as f64).log2()).sum();
    let covered = seen.iter().filter(|s| **s).count();
    (dcg / idcg, covered as f64 / n as f64)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    Bm25,
    QueryExpansion,
    DocExpansion,
    QueryDocExpansion,
}

impl Arm {
    pub const ALL: [Arm; 4] = [
        Arm::Bm25,
        Arm::QueryExpansion,
        Arm::DocExpansion,
        Arm::QueryDocExpansion,
    ];
    fn uses_query(self) -> bool {
        matches!(self, Arm::QueryExpansion | Arm::QueryDocExpansion)
    }
    fn uses_doc(self) -> bool {
        matches!(self, Arm::DocExpansion | Arm::QueryDocExpansion)
    }
    pub fn name(self) -> &'static str {
        match self {
            Arm::Bm25 => "bm25",
            Arm::QueryExpansion => "query_expansion",
            Arm::DocExpansion => "doc_expansion",
            Arm::QueryDocExpansion => "query_doc_expansion",
        }
    }
}

/// A task ready to search: the query, the gradable cited locators, the finalizer's expressions.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub task_id: String,
    pub kind: RetrievalKind,
    pub query: String,
    pub cited: Vec<SourceLocator>,
    /// `None` when the expected-terms file has no sketch for the task.
    pub expressions: Option<Vec<(String, f64)>>,
}

/// Why a task is not scored.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Excluded {
    pub task_id: String,
    pub reason: String,
}

/// What [`prepare`] made of the task file.
#[derive(Debug, Clone, Serialize)]
pub struct Preparation {
    #[serde(skip)]
    pub tasks: Vec<Prepared>,
    pub excluded: Vec<Excluded>,
    pub cited_ungradable: usize,
    pub cited_asset_not_in_catalog: usize,
    /// Cited locators no chunk visible to the scope matches (the lockbox withholds them, or the
    /// catalog has no chunk there): the agent cannot retrieve them by design, so they are not
    /// scored as retrieval misses.
    pub cited_not_visible: usize,
    pub origin: Origin,
}

fn assets_in_catalog(con: &Connection) -> Result<std::collections::HashSet<String>> {
    let mut st = con.prepare("SELECT source_sha256 FROM publications")?;
    let v = st
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// Is any chunk of the file that `gold` names both visible to the scope and relevant to it?
fn visible_match(con: &Connection, gold: &SourceLocator, withheld: Withheld) -> Result<bool> {
    let mut st = con.prepare(&format!(
        "SELECT c.id, c.locator_json FROM chunks c JOIN publications p ON p.id = c.publication_id \
         WHERE p.source_sha256 = ?1 AND {}",
        catalog::ACTIVE_VIEW
    ))?;
    let rows = st
        .query_map([&gold.asset_sha256], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (id, loc) in rows {
        let loc: SourceLocator =
            serde_json::from_str(&loc).with_context(|| format!("{id}: locator"))?;
        if relevant(&loc, gold) && !withheld(&id)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Keep the retrieval sub-set with at least one gradable cited locator whose file is catalogued.
pub fn prepare(
    con: &Connection,
    tasks: &[GoldTaskView],
    terms: &BTreeMap<String, Vec<ExpectedTerm>>,
    withheld: Withheld,
) -> Result<Preparation> {
    for id in terms.keys() {
        if !tasks.iter().any(|t| &t.task_id == id) {
            bail!("expected terms for {id}, which is not a task of the dev file");
        }
    }
    let assets = assets_in_catalog(con)?;
    let mut prep = Preparation {
        tasks: vec![],
        excluded: vec![],
        cited_ungradable: 0,
        cited_asset_not_in_catalog: 0,
        cited_not_visible: 0,
        origin: Origin::combine(tasks.iter().map(|t| t.origin)),
    };
    for t in tasks {
        let ex = |reason: &str| Excluded {
            task_id: t.task_id.clone(),
            reason: reason.into(),
        };
        let Some(kind) = t.retrieval else {
            prep.excluded.push(ex("not in the retrieval sub-set"));
            continue;
        };
        let all = t.cited()?;
        if all.is_empty() {
            prep.excluded.push(ex("the answer key cites no locator"));
            continue;
        }
        let mut cited = Vec::new();
        for l in all {
            if !gradable(&l) {
                prep.cited_ungradable += 1;
            } else if !assets.contains(&l.asset_sha256) {
                prep.cited_asset_not_in_catalog += 1;
            } else if !visible_match(con, &l, withheld)? {
                prep.cited_not_visible += 1;
            } else {
                cited.push(l);
            }
        }
        if cited.is_empty() {
            prep.excluded.push(ex(
                "no cited locator is gradable and visible to this scope (ungradable, file not catalogued, or withheld by the lockbox)",
            ));
            continue;
        }
        let expressions = match terms.get(&t.task_id) {
            None => None,
            Some(ts) => {
                let proposals: Vec<(String, f64)> = ts
                    .iter()
                    .map(|e| (e.term.clone(), TermImportance::weight(e.importance)))
                    .collect();
                Some(
                    sira::select_expressions(con, &proposals)?
                        .into_iter()
                        .filter_map(|c| c.weight.map(|w| (c.expression, w)))
                        .collect(),
                )
            }
        };
        prep.tasks.push(Prepared {
            task_id: t.task_id.clone(),
            kind,
            query: t.question.clone(),
            cited,
            expressions,
        });
    }
    Ok(prep)
}

/// A filter deciding which chunks the scope may not see (the service's lockbox).
pub type Withheld<'a> = &'a dyn Fn(&str) -> Result<bool>;

/// The top `K` visible chunks of one arm, with the cited locators each covers.
pub fn rank(
    con: &Connection,
    t: &Prepared,
    arm: Arm,
    w: f64,
    withheld: Withheld,
) -> Result<Option<(Vec<String>, Vec<Vec<usize>>)>> {
    let exprs: &[(String, f64)] = if arm.uses_query() {
        match &t.expressions {
            Some(e) => e,
            None => return Ok(None),
        }
    } else {
        &[]
    };
    let doc_w = if arm.uses_doc() { w } else { 0.0 };
    let none = catalog::Filters::default();
    let mut ids = Vec::new();
    let mut offset = 0;
    let window = 4 * K;
    loop {
        let hits = catalog::lexical_weighted(con, &t.query, exprs, doc_w, &none, offset, window)?;
        let n = hits.len();
        for h in hits {
            if !withheld(&h.chunk_id)? {
                ids.push(h.chunk_id);
                if ids.len() == K {
                    break;
                }
            }
        }
        if ids.len() == K || n < window {
            break;
        }
        offset += window;
    }
    let rows = catalog::get_chunks(con, &ids)?;
    let mut covers = Vec::new();
    for id in &ids {
        let loc: SourceLocator = serde_json::from_value(rows[id]["locator"].clone())
            .with_context(|| format!("{id}: locator"))?;
        covers.push(
            t.cited
                .iter()
                .enumerate()
                .filter(|(_, g)| relevant(&loc, g))
                .map(|(i, _)| i)
                .collect(),
        );
    }
    Ok(Some((ids, covers)))
}

/// One task under one arm.
#[derive(Debug, Clone, Serialize)]
pub struct TaskScore {
    pub task_id: String,
    pub kind: RetrievalKind,
    pub ndcg_at_10: f64,
    pub recall_at_10: f64,
    pub top: Vec<String>,
}

/// Mean metrics of one arm over one group of tasks; `n` is the number scored. For the arms that
/// need expected terms, `bm25_ndcg_same_tasks` is plain BM25 on exactly those tasks (a paired
/// baseline).
#[derive(Debug, Clone, Serialize)]
pub struct GroupScore {
    pub n: usize,
    pub ndcg_at_10: Option<f64>,
    pub recall_at_10: Option<f64>,
    pub bm25_ndcg_same_tasks: Option<f64>,
}

/// Every arm at one (τ, w).
#[derive(Debug, Clone, Serialize)]
pub struct Cell {
    pub tau: Option<f64>,
    pub w: f64,
    /// arm → group (`single_hop`, `multi_hop`, `figure`, `all`) → score
    pub arms: BTreeMap<Arm, BTreeMap<String, GroupScore>>,
    pub per_task: BTreeMap<Arm, Vec<TaskScore>>,
}

fn group_of(k: RetrievalKind) -> &'static str {
    match k {
        RetrievalKind::SingleHop => "single_hop",
        RetrievalKind::MultiHop => "multi_hop",
        RetrievalKind::Figure => "figure",
    }
}

fn mean(v: impl Iterator<Item = f64>) -> Option<f64> {
    let v: Vec<f64> = v.collect();
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

/// Score `arms` on every prepared task at document weight `w`.
pub fn evaluate(
    con: &Connection,
    tasks: &[Prepared],
    arms: &[Arm],
    tau: Option<f64>,
    w: f64,
    withheld: Withheld,
) -> Result<Cell> {
    let mut per_task: BTreeMap<Arm, Vec<TaskScore>> = BTreeMap::new();
    for &arm in arms {
        let mut v = Vec::new();
        for t in tasks {
            if let Some((top, covers)) = rank(con, t, arm, w, withheld)? {
                let (ndcg, recall) = ndcg_recall(&covers, t.cited.len(), K);
                v.push(TaskScore {
                    task_id: t.task_id.clone(),
                    kind: t.kind,
                    ndcg_at_10: ndcg,
                    recall_at_10: recall,
                    top,
                });
            }
        }
        per_task.insert(arm, v);
    }
    Ok(cell_from(tau, w, per_task, None))
}

fn cell_from(
    tau: Option<f64>,
    w: f64,
    per_task: BTreeMap<Arm, Vec<TaskScore>>,
    bm25: Option<&[TaskScore]>,
) -> Cell {
    let bm25: Vec<TaskScore> = bm25
        .map(|b| b.to_vec())
        .or_else(|| per_task.get(&Arm::Bm25).cloned())
        .unwrap_or_default();
    let mut arms = BTreeMap::new();
    for (arm, scores) in &per_task {
        let mut groups = BTreeMap::new();
        for g in ["single_hop", "multi_hop", "figure", "all"] {
            let sel: Vec<&TaskScore> = scores
                .iter()
                .filter(|s| g == "all" || group_of(s.kind) == g)
                .collect();
            let paired = arm.uses_query().then(|| {
                mean(
                    bm25.iter()
                        .filter(|b| sel.iter().any(|s| s.task_id == b.task_id))
                        .map(|b| b.ndcg_at_10),
                )
            });
            groups.insert(
                g.to_string(),
                GroupScore {
                    n: sel.len(),
                    ndcg_at_10: mean(sel.iter().map(|s| s.ndcg_at_10)),
                    recall_at_10: mean(sel.iter().map(|s| s.recall_at_10)),
                    bm25_ndcg_same_tasks: paired.flatten(),
                },
            );
        }
        arms.insert(*arm, groups);
    }
    Cell {
        tau,
        w,
        arms,
        per_task,
    }
}

/// What the sweep chose: a proposal for a person to apply, never applied by this tool.
#[derive(Debug, Clone, Serialize)]
pub struct Proposal {
    pub status: &'static str,
    pub applied: bool,
    pub target: &'static str,
    pub current: Value,
    pub proposed: Option<Value>,
    pub objective: String,
    pub n_tasks: usize,
    pub origin: Origin,
    pub note: String,
}

/// The whole sweep, written as `report.json` / `report.md` / `proposal.json`.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub tasks_file: String,
    pub tasks_sha256: String,
    pub catalog: String,
    pub catalog_generation: String,
    pub pack: Option<String>,
    pub pack_manifest_sha256: Option<String>,
    pub expected_terms_file: Option<String>,
    pub origin: Origin,
    pub preparation: Preparation,
    pub relevance_rule: &'static str,
    /// τ- and w-independent arms (`bm25`, `query_expansion`).
    pub baseline: Cell,
    /// One cell per (τ, w) with the document-expansion arms.
    pub grid: Vec<Cell>,
    pub proposal: Proposal,
}

pub const RELEVANCE_RULE: &str = "same asset_sha256 and (overlapping char span on the same parse view and page | same page | same table row | same path); first hit covering a not-yet-covered cited locator gains 1";

/// Inputs of [`sweep`].
pub struct SweepArgs<'a> {
    pub tasks: &'a Path,
    pub db: &'a Path,
    pub pack: Option<&'a Path>,
    pub expected_terms: Option<&'a Path>,
    pub taus: Option<Vec<f64>>,
    pub ws: Vec<f64>,
    pub out: &'a Path,
}

/// Run the evaluation over the grid; writes the report and the proposal under `args.out`.
pub fn sweep(args: &SweepArgs, scope: &ExecutionScope, withheld: Withheld) -> Result<Report> {
    check_scope(scope)?;
    check_not_holdout("catalog", args.db)?;
    check_not_holdout("output", args.out)?;
    if let Some(p) = args.pack {
        check_not_holdout("pack", p)?;
    }
    let tasks = load_tasks(args.tasks)?;
    let terms = match args.expected_terms {
        Some(p) => load_expected_terms(p)?,
        None => BTreeMap::new(),
    };
    if args.taus.is_some() && args.pack.is_none() {
        bail!("--tau changes the expansion index: pass --pack to rebuild it at each τ");
    }
    for &x in args.taus.iter().flatten().chain(args.ws.iter()) {
        if !x.is_finite() || x < 0.0 {
            bail!("τ and w must be finite and non-negative (got {x})");
        }
    }
    std::fs::create_dir_all(args.out)?;
    let work = args.out.join("work.sqlite");
    let _ = std::fs::remove_file(&work);
    {
        let src = catalog::connect(args.db, true)?;
        src.execute("VACUUM INTO ?1", [work.display().to_string()])
            .context("copy the catalog into the working copy")?;
    }
    let result = sweep_on(args, &tasks, &terms, &work, withheld);
    let _ = std::fs::remove_file(&work);
    let report = result?;
    write_report(args.out, &report)?;
    Ok(report)
}

fn sweep_on(
    args: &SweepArgs,
    tasks: &[GoldTaskView],
    terms: &BTreeMap<String, Vec<ExpectedTerm>>,
    work: &Path,
    withheld: Withheld,
) -> Result<Report> {
    let mut con = catalog::connect(work, false)?;
    let prep = prepare(&con, tasks, terms, withheld)?;
    let baseline = evaluate(
        &con,
        &prep.tasks,
        &[Arm::Bm25, Arm::QueryExpansion],
        None,
        0.0,
        withheld,
    )?;
    let bm25 = baseline
        .per_task
        .get(&Arm::Bm25)
        .cloned()
        .unwrap_or_default();
    let existing: Option<Value> = con
        .query_row("SELECT value FROM meta WHERE key = 'expansion'", [], |r| {
            r.get::<_, String>(0)
        })
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    let mut grid = Vec::new();
    let mut pack_manifest = None;
    let taus: Vec<Option<f64>> = match (&args.taus, args.pack) {
        (Some(t), Some(_)) => t.iter().copied().map(Some).collect(),
        (None, Some(_)) => DEFAULT_TAUS.iter().copied().map(Some).collect(),
        (_, None) => {
            let n: i64 = con.query_row("SELECT count(*) FROM expansion_fts", [], |r| r.get(0))?;
            if n == 0 {
                vec![]
            } else {
                vec![existing.as_ref().and_then(|e| e["tau"].as_f64())]
            }
        }
    };
    for tau in taus {
        if let (Some(t), Some(p)) = (tau, args.pack) {
            let info = sira::enrich(&mut con, p, t)?;
            pack_manifest = info["pack_manifest_sha256"].as_str().map(String::from);
        }
        for &w in &args.ws {
            let c = evaluate(
                &con,
                &prep.tasks,
                &[Arm::DocExpansion, Arm::QueryDocExpansion],
                tau,
                w,
                withheld,
            )?;
            grid.push(cell_from(tau, w, c.per_task, Some(&bm25)));
        }
    }
    let proposal = propose(&prep, &grid);
    let raw = std::fs::read(args.tasks)?;
    Ok(Report {
        schema: "ignis.retrieval_eval/1",
        tasks_file: args.tasks.display().to_string(),
        tasks_sha256: ignis_core::hashing::sha256_hex(&raw),
        catalog: args.db.display().to_string(),
        catalog_generation: sira::generation(&con)?,
        pack: args.pack.map(|p| p.display().to_string()),
        pack_manifest_sha256: pack_manifest.or_else(|| {
            existing.and_then(|e| e["pack_manifest_sha256"].as_str().map(String::from))
        }),
        expected_terms_file: args.expected_terms.map(|p| p.display().to_string()),
        origin: prep.origin,
        preparation: prep,
        relevance_rule: RELEVANCE_RULE,
        baseline,
        grid,
        proposal,
    })
}

/// Best (τ, w) by mean nDCG@10 over every scored task, then recall@10, then the smaller w and τ.
/// The objective is `query_doc_expansion` when every task has expected terms, else `doc_expansion`.
pub fn propose(prep: &Preparation, grid: &[Cell]) -> Proposal {
    let all_terms = prep.tasks.iter().all(|t| t.expressions.is_some()) && !prep.tasks.is_empty();
    let arm = if all_terms {
        Arm::QueryDocExpansion
    } else {
        Arm::DocExpansion
    };
    let current = serde_json::json!({"tau": sira::DEFAULT_TAU, "w": sira::DOC_EXPANSION_WEIGHT});
    let score = |c: &Cell| -> Option<(f64, f64)> {
        let g = c.arms.get(&arm)?.get("all")?;
        Some((g.ndcg_at_10?, g.recall_at_10?))
    };
    let mut best: Option<(&Cell, (f64, f64))> = None;
    for c in grid {
        let Some(s) = score(c) else { continue };
        let better = match &best {
            None => true,
            Some((b, bs)) => {
                s.0 > bs.0 + 1e-12
                    || ((s.0 - bs.0).abs() <= 1e-12
                        && (s.1 > bs.1 + 1e-12
                            || ((s.1 - bs.1).abs() <= 1e-12
                                && (c.w, c.tau.unwrap_or(0.0)) < (b.w, b.tau.unwrap_or(0.0)))))
            }
        };
        if better {
            best = Some((c, s));
        }
    }
    let no_grid = best.is_none();
    // a grid that finds nothing anywhere has no optimum to propose
    let no_signal = best.is_some_and(|(_, (n, r))| n == 0.0 && r == 0.0);
    if no_signal {
        best = None;
    }
    let synthetic = prep.origin == Origin::SyntheticFixture;
    let status = if synthetic {
        "synthetic_fixture_not_a_result"
    } else if no_grid {
        "no_grid"
    } else if no_signal {
        "no_signal"
    } else {
        "proposal_not_applied"
    };
    let mut note = String::from(
        "A proposal only: nothing is applied. To adopt it, change DEFAULT_TAU and \
         DOC_EXPANSION_WEIGHT in crates/ignis-evidence/src/sira.rs in a reviewed PR, then re-run \
         `ignis evidence enrich` with the new τ. Values tuned on the dev set overstate dev scores.",
    );
    if synthetic {
        note.push_str(" The task set is a synthetic fixture: these numbers are never results.");
    }
    if no_signal {
        note.push_str(" No cell of the grid retrieved any cited locator: nothing is proposed.");
    }
    if no_grid {
        note.push_str(" No document-expansion grid ran (no pack and an empty expansion index).");
    }
    Proposal {
        status,
        applied: false,
        target: "crates/ignis-evidence/src/sira.rs (DEFAULT_TAU, DOC_EXPANSION_WEIGHT)",
        current,
        proposed: best.map(|(c, (n, r))| {
            serde_json::json!({"tau": c.tau, "w": c.w, "ndcg_at_10": n, "recall_at_10": r})
        }),
        objective: format!("mean nDCG@10 of {} over all scored tasks", arm.name()),
        n_tasks: prep.tasks.len(),
        origin: prep.origin,
        note,
    }
}

fn fmt(x: Option<f64>) -> String {
    x.map_or("–".into(), |v| format!("{v:.3}"))
}

/// The report as Markdown (the per-group table of the baseline arms and the proposed cell, then
/// the grid on `all`).
pub fn markdown(r: &Report) -> String {
    let mut s = String::new();
    s.push_str("# Retrieval evaluation (R12b)\n\n");
    if r.origin == Origin::SyntheticFixture {
        s.push_str(
            "> **Synthetic fixture.** These numbers test the tool; they are never results.\n\n",
        );
    }
    s.push_str(&format!(
        "- tasks: `{}` (sha256 `{}`), {} scored, {} excluded; cited locators ungradable: {}, file not in catalog: {}, not visible to this scope: {}\n- catalog: `{}` ({})\n- relevance: {}\n\n",
        r.tasks_file,
        &r.tasks_sha256[..16.min(r.tasks_sha256.len())],
        r.preparation.tasks.len(),
        r.preparation.excluded.len(),
        r.preparation.cited_ungradable,
        r.preparation.cited_asset_not_in_catalog,
        r.preparation.cited_not_visible,
        r.catalog,
        r.catalog_generation,
        r.relevance_rule,
    ));
    let chosen = r.proposal.proposed.as_ref().and_then(|p| {
        r.grid
            .iter()
            .find(|c| Some(c.w) == p["w"].as_f64() && c.tau == p["tau"].as_f64())
    });
    s.push_str("| arm | τ | w | group | n | nDCG@10 | recall@10 | BM25 nDCG@10 (same tasks) |\n|---|---|---|---|---|---|---|---|\n");
    let mut rows = |c: &Cell| {
        for (arm, groups) in &c.arms {
            for (g, gs) in groups {
                s.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
                    arm.name(),
                    fmt(c.tau),
                    if arm.uses_doc() {
                        format!("{:.2}", c.w)
                    } else {
                        "–".into()
                    },
                    g,
                    gs.n,
                    fmt(gs.ndcg_at_10),
                    fmt(gs.recall_at_10),
                    fmt(gs.bm25_ndcg_same_tasks)
                ));
            }
        }
    };
    rows(&r.baseline);
    if let Some(c) = chosen {
        rows(c);
    }
    if !r.grid.is_empty() {
        s.push_str("\n## Grid (all tasks, nDCG@10 / recall@10)\n\n| τ | w | doc_expansion | query_doc_expansion |\n|---|---|---|---|\n");
        for c in &r.grid {
            let cell = |a: Arm| {
                c.arms
                    .get(&a)
                    .and_then(|g| g.get("all"))
                    .map_or("–".into(), |g| {
                        format!("{} / {}", fmt(g.ndcg_at_10), fmt(g.recall_at_10))
                    })
            };
            s.push_str(&format!(
                "| {} | {:.2} | {} | {} |\n",
                fmt(c.tau),
                c.w,
                cell(Arm::DocExpansion),
                cell(Arm::QueryDocExpansion)
            ));
        }
    }
    s.push_str(&format!(
        "\n## Proposal ({})\n\nCurrent τ = {}, w = {}. Proposed: {}.\n\n{}\n",
        r.proposal.status,
        sira::DEFAULT_TAU,
        sira::DOC_EXPANSION_WEIGHT,
        r.proposal
            .proposed
            .as_ref()
            .map_or("none".into(), |p| p.to_string()),
        r.proposal.note
    ));
    if !r.preparation.excluded.is_empty() {
        s.push_str("\n## Excluded tasks\n\n");
        for e in &r.preparation.excluded {
            s.push_str(&format!("- `{}`: {}\n", e.task_id, e.reason));
        }
    }
    s
}

fn write_report(out: &Path, r: &Report) -> Result<()> {
    std::fs::write(out.join("report.json"), serde_json::to_vec_pretty(r)?)?;
    std::fs::write(out.join("report.md"), markdown(r))?;
    std::fs::write(
        out.join("proposal.json"),
        serde_json::to_vec_pretty(&r.proposal)?,
    )?;
    Ok(())
}

/// Parse a comma list of numbers (`0.1,0.2`).
pub fn parse_list(s: &str) -> Result<Vec<f64>> {
    s.split(',')
        .map(|x| {
            x.trim()
                .parse::<f64>()
                .with_context(|| format!("not a number: {x}"))
        })
        .collect()
}

/// Default output directory.
pub fn default_out() -> PathBuf {
    catalog::root().join("retrieval_eval")
}

//! [`KnowledgeService`] over the evidence catalog (workboard R3): the one search/read implementation
//! that the CLI (`ignis evidence search|read|resolve`), the HTTP API and the MCP tools project.
//!
//! **Search.** `lexical (FTS5) + dense (exact cosine) → RRF k=60 → scope filter → rerank head`.
//! The scope filter runs before the reranker, so withheld text never reaches the provider. Each
//! backend is reported `used` / `unavailable` (with the reason) / `not_requested`; a lexical-only
//! fallback is visible in [`SearchPage::degraded`]. Hits are [`EvidenceRef`]s with a snippet; the
//! passage is read with [`KnowledgeService::read`]. Scores: `bm25` is the positive BM25 (FTS5's value
//! negated: larger = more relevant), `cosine`, `rrf`, `rerank` as computed; none is a probability.
//!
//! **Paging without a semantic cap.** Pages slice one deterministic order: the fused (and reranked)
//! candidates, then the lexical matches beyond the fused depth in BM25 order. The cursor is opaque
//! and binds the corpus snapshot, the request and the scope's lockbox; it carries the offset. Orders
//! are cached in memory (a bounded number of searches: a memory budget, not a result cap); a cache
//! miss recomputes the same order, and refuses with `conflict` if the reranker that ordered the first
//! page is unavailable now (it never splices two different orders). `total_count` is set only when
//! the count is exhaustive (lexical mode with every match ranked).
//!
//! **Lockbox.** A scope reads the outcomes of its granted partitions only. Passages that can reveal
//! how a run ended (`outcome_bearing`) are withheld when they are attributed to a run of a partition
//! the scope lacks, or when they are unattributed (or attributed through an alias that does not
//! resolve) and their publication covers an investigation or family with such runs. Methods,
//! apparatus and background passages stay readable. Unknown attribution is withheld, never guessed.
//! The partitions come from the frozen splits (`data/splits/`); aliases from the ingested records.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use ignis_core::contracts::{
    BackendReport, BackendStatus, Capability, CorpusSnapshotId, ErrorCode, ErrorEnvelope,
    EvidenceClass, EvidenceId, EvidenceRef, EvidenceSchema, ExecutionScope, Family, Finite,
    OffsetUnit, Origin, OutlineRequest, PageCursor, ParseViewId, Partition, ReadRange, ReadRequest,
    ReadResult, ReadSegment, RetrievalBackend, RetrievalScore, RetryPolicy, ScoreKind, SearchHit,
    SearchMode, SearchPage, SearchRequest, SourceId, SourceKind, SourceLocator, SourceOutline,
    Split,
};
use ignis_core::hashing::{canonical_json, sha256_hex};
use ignis_core::services::{KnowledgeService, ServiceResult};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

use crate::catalog::{self, Filters};
use crate::corpus::{self, CorpusSnapshot};
use crate::dense::RetrievalCfg;
use crate::records::parse_view_id;
use crate::search::{Ranked, RankedHit, Retriever, SearchOpts};

/// Section summaries next to the catalog (answers of `ignis evidence outline-summaries --execute`).
pub const SUMMARIES_FILE: &str = "outline_summaries.jsonl";

/// Label pass answers and the combined labels next to the catalog (R8).
pub const LABEL_ANSWERS_FILE: &str = "label_answers.jsonl";
pub const LABELS_FILE: &str = "source_labels.jsonl";

/// Requested page size when the request names none.
pub const DEFAULT_PAGE_SIZE: u32 = 10;
/// Characters of page text per read response (more pages follow with `next`).
pub const READ_BUDGET_CHARS: usize = 60_000;
/// Searches whose ranked order is kept for paging (memory budget).
const ORDER_CACHE: usize = 64;
const SNIPPET_CHARS: usize = 280;

fn err(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

/// The parse view of a chunk: its id is `<view>:p<page>:<ordinal>`.
fn chunk_view(chunk_id: &str) -> String {
    chunk_id
        .rsplitn(3, ':')
        .nth(2)
        .unwrap_or(chunk_id)
        .to_string()
}

fn internal(e: anyhow::Error) -> ErrorEnvelope {
    err(ErrorCode::Internal, format!("{e:#}"))
}

// ------------------------------------------------------------------ lockbox

/// Which runs each partition holds (frozen splits) and how report aliases name them.
#[derive(Debug, Clone, Default)]
pub struct Lockbox {
    run_partition: HashMap<String, Partition>,
    alias_to_run: HashMap<String, String>,
    /// investigation (`PSI-25`) / evidence family (`bass`) -> partitions of its runs
    covered: HashMap<String, BTreeSet<Partition>>,
}

fn evidence_family(f: Family) -> &'static str {
    match f {
        Family::Flex => "flex",
        Family::AcmeNormal | Family::AcmeInverse => "acme",
        Family::Bass => "bass",
        Family::Other => "other",
    }
}

impl Lockbox {
    /// From explicit splits (each with its cohort's family) and alias map.
    pub fn new(splits: &[(Split, Family)], alias_to_run: HashMap<String, String>) -> Lockbox {
        let mut lb = Lockbox {
            alias_to_run,
            ..Default::default()
        };
        for (s, fam) in splits {
            for run in s.run_to_group.keys() {
                let Some(p) = s.partition_of(run) else {
                    continue;
                };
                lb.run_partition.insert(run.clone(), p);
                if let Some((inv, _)) = run.split_once(':') {
                    lb.covered.entry(inv.to_string()).or_default().insert(p);
                }
                lb.covered
                    .entry(evidence_family(*fam).to_string())
                    .or_default()
                    .insert(p);
            }
        }
        lb
    }

    /// Every frozen split under `data/splits/` (hash-verified) and the aliases of the ingested
    /// records. Missing records only make attribution more conservative.
    pub fn load() -> anyhow::Result<Lockbox> {
        let dir = ignis_data::paths::splits();
        let mut splits = Vec::new();
        if dir.exists() {
            let mut files: Vec<_> = std::fs::read_dir(&dir)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect();
            files.sort();
            for f in files {
                let (split, cohort) =
                    ignis_data::cohort::load_split_with_cohort(&f.display().to_string())?;
                splits.push((split, cohort.family));
            }
        }
        Ok(Lockbox::new(&splits, record_aliases()?))
    }

    /// Partitions whose runs' outcomes `scope` may not read.
    pub fn sealed(&self, scope: &ExecutionScope) -> BTreeSet<Partition> {
        [Partition::Train, Partition::Validation, Partition::Test]
            .into_iter()
            .filter(|p| scope.require_partition(*p).is_err())
            .collect()
    }

    /// True when a chunk must be withheld from a scope that cannot read `sealed`.
    pub fn withholds(
        &self,
        sealed: &BTreeSet<Partition>,
        outcome_bearing: bool,
        describes_runs: &[String],
        continues_runs: &[String],
        publication_scope: &[String],
    ) -> bool {
        if sealed.is_empty() || !outcome_bearing {
            return false;
        }
        // a chunk that may continue a sealed run's text (split table row) is withheld; a continued
        // run never counts as an attribution that would make the chunk readable
        if continues_runs.iter().any(|r| {
            let run = self.alias_to_run.get(r).unwrap_or(r);
            self.run_partition
                .get(run)
                .is_some_and(|p| sealed.contains(p))
        }) {
            return true;
        }
        let mut unresolved = describes_runs.is_empty();
        for r in describes_runs {
            let (run, known) = match self.alias_to_run.get(r) {
                Some(run) => (run, true),
                None => (r, self.run_partition.contains_key(r)),
            };
            match self.run_partition.get(run) {
                Some(p) if sealed.contains(p) => return true,
                // a known run outside every split (excluded from the cohort) is not in the lockbox
                _ if known => {}
                _ => unresolved = true,
            }
        }
        unresolved
            && publication_scope.iter().any(|k| {
                self.covered
                    .get(k)
                    .is_some_and(|ps| ps.iter().any(|p| sealed.contains(p)))
            })
    }

    /// Stable digest of what a scope cannot see (part of the cursor and cache key).
    fn sealed_key(sealed: &BTreeSet<Partition>) -> String {
        format!("{sealed:?}")
    }
}

/// Run-pattern aliases (`configs/evidence/run_patterns.toml`) -> run ids, from the ingested records:
/// `alias:bass2:test<T>` / `alias:bass2:sample<S>:test<T>` (PSI-25) and `alias:flex:FLEX-<NNN>`
/// (PSI-69). An alias without a record stays unresolved (and is treated conservatively).
fn record_aliases() -> anyhow::Result<HashMap<String, String>> {
    let mut out = HashMap::new();
    for family in ["bass", "flex"] {
        let p = ignis_data::paths::work(&["records", family, "experiments.jsonl"]);
        if !p.exists() {
            continue;
        }
        for l in std::fs::read_to_string(&p)?.lines() {
            let v: Value = serde_json::from_str(l).with_context(|| p.display().to_string())?;
            let Some(id) = v["id"].as_str() else {
                continue;
            };
            let alias = |scheme: &str| {
                v["aliases"].as_array().and_then(|a| {
                    a.iter()
                        .find(|x| x["scheme"] == scheme)
                        .and_then(|x| x["name"].as_str())
                        .map(str::to_uppercase)
                })
            };
            if id.starts_with("PSI-25:") {
                if let Some(t) = alias("test_number") {
                    out.insert(format!("alias:bass2:test{t}"), id.to_string());
                    if let Some(s) = alias("sample_number") {
                        out.insert(format!("alias:bass2:sample{s}:test{t}"), id.to_string());
                    }
                }
            }
            if id.starts_with("PSI-69:") {
                if let Some(n) = alias("file_flex_number") {
                    out.insert(format!("alias:flex:{n}"), id.to_string());
                }
            }
        }
    }
    Ok(out)
}

// ------------------------------------------------------------------ service

/// What the lockbox needs to know about one chunk and its publication.
#[derive(Debug, Clone)]
struct ChunkRow {
    id: String,
    publication_id: String,
    page: i64,
    text: String,
    locator: SourceLocator,
    evidence_class: String,
    outcome_bearing: bool,
    describes_runs: Vec<String>,
    continues_runs: Vec<String>,
    title: Option<String>,
    /// investigations + evidence families of the publication
    publication_scope: Vec<String>,
}

pub struct EvidenceService {
    retriever: Mutex<Retriever>,
    /// separate read-only connection for chunk lookups (the scope filter runs while the retriever
    /// is borrowed)
    meta: Mutex<Connection>,
    snapshot: CorpusSnapshot,
    lockbox: Lockbox,
    /// Section summaries (R7b pass answers, `outline_summaries.jsonl` next to the catalog), read once.
    summaries_path: std::path::PathBuf,
    summaries: std::sync::OnceLock<
        std::collections::HashMap<(String, String), ignis_core::contracts::GeneratedText>,
    >,
    orders: Mutex<VecDeque<(String, Arc<Ranked>)>>,
    /// record API spend in the cost ledger (`runs/`); off in tests
    pub ledger: bool,
}

/// Decoded search cursor (`c1.<hex json>`).
#[derive(Debug, PartialEq)]
struct Cursor {
    snapshot: String,
    request: String,
    offset: usize,
    reranked: bool,
}

impl Cursor {
    fn encode(&self) -> PageCursor {
        let j =
            json!({"s": self.snapshot, "q": self.request, "o": self.offset, "r": self.reranked});
        PageCursor::new(format!("c1.{}", hex::encode(j.to_string()))).expect("hex has no spaces")
    }

    fn decode(c: &PageCursor) -> Option<Cursor> {
        let h = c.as_str().strip_prefix("c1.")?;
        let j: Value = serde_json::from_slice(&hex::decode(h).ok()?).ok()?;
        Some(Cursor {
            snapshot: j["s"].as_str()?.to_string(),
            request: j["q"].as_str()?.to_string(),
            offset: usize::try_from(j["o"].as_u64()?).ok()?,
            reranked: j["r"].as_bool()?,
        })
    }
}

impl EvidenceService {
    /// Public constructor for the composition root: the catalog at `db`, the retrieval config and
    /// the lockbox. The API key, if any, is read from the environment by the retriever.
    pub fn open(db: &Path, cfg: RetrievalCfg, lockbox: Lockbox) -> anyhow::Result<EvidenceService> {
        let retriever = Retriever::open(db, cfg)?;
        let meta = catalog::connect(db, true)?;
        let snapshot = corpus::snapshot(&meta)?;
        Ok(EvidenceService {
            retriever: Mutex::new(retriever),
            meta: Mutex::new(meta),
            snapshot,
            lockbox,
            summaries_path: db.with_file_name(SUMMARIES_FILE),
            summaries: std::sync::OnceLock::new(),
            orders: Mutex::new(VecDeque::new()),
            ledger: true,
        })
    }

    /// Default catalog, `configs/evidence/retrieval.toml`, lockbox from the frozen splits.
    ///
    /// Fails closed when no frozen split is found: an empty lockbox seals nothing, so a scope
    /// without the test partition would read test outcomes.
    pub fn open_default() -> anyhow::Result<EvidenceService> {
        let cfg = RetrievalCfg::load(&RetrievalCfg::default_path())?;
        let lockbox = Lockbox::load()?;
        anyhow::ensure!(
            !lockbox.run_partition.is_empty(),
            "no frozen splits under {}: refusing to serve passages without a lockbox",
            ignis_data::paths::splits().display()
        );
        EvidenceService::open(&catalog::default_db(), cfg, lockbox)
    }

    /// Run without network access to the embedding/rerank provider (declared as unavailable).
    pub fn offline(self) -> Self {
        self.retriever.lock().expect("retriever").api_key = None;
        self
    }

    pub fn snapshot(&self) -> &CorpusSnapshot {
        &self.snapshot
    }

    fn snapshot_id(&self) -> CorpusSnapshotId {
        CorpusSnapshotId::new(self.snapshot.snapshot_id.clone()).expect("corpus-<hex>")
    }

    fn check_scope(&self, scope: &ExecutionScope) -> ServiceResult<()> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        scope.require(Capability::ReadSources)?;
        if let Some(s) = &scope.corpus_snapshot {
            self.check_snapshot(s)?;
        }
        Ok(())
    }

    fn check_snapshot(&self, s: &CorpusSnapshotId) -> ServiceResult<()> {
        if s.as_str() != self.snapshot.snapshot_id {
            return Err(err(
                ErrorCode::Conflict,
                format!(
                    "corpus snapshot {s} is not served here (current: {})",
                    self.snapshot.snapshot_id
                ),
            ));
        }
        Ok(())
    }

    fn chunk(&self, id: &str) -> anyhow::Result<Option<ChunkRow>> {
        let con = self.meta.lock().expect("meta");
        let row = con
            .query_row(
                "SELECT c.id, c.publication_id, c.page, c.ordinal, c.text, c.locator_json, c.evidence_class, \
                 c.outcome_bearing, c.describes_runs_json, p.title, p.investigation_ids_json, p.families_json, \
                 c.continues_runs_json \
                 FROM chunks c JOIN publications p ON p.id = c.publication_id WHERE c.id = ?1",
                params![id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, i64>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, String>(6)?,
                        r.get::<_, i64>(7)? != 0,
                        r.get::<_, String>(8)?,
                        r.get::<_, Option<String>>(9)?,
                        r.get::<_, String>(10)?,
                        r.get::<_, String>(11)?,
                        r.get::<_, String>(12)?,
                    ))
                },
            )
            .optional()?;
        let Some((id, pid, page, _ordinal, text, loc, class, ob, runs, title, invs, fams, cont)) =
            row
        else {
            return Ok(None);
        };
        let arr = |s: &str| serde_json::from_str::<Vec<String>>(s).unwrap_or_default();
        let mut scope = arr(&invs);
        scope.extend(arr(&fams));
        Ok(Some(ChunkRow {
            id,
            publication_id: pid,
            page,
            text,
            locator: serde_json::from_str(&loc)?,
            evidence_class: class,
            outcome_bearing: ob,
            describes_runs: arr(&runs),
            continues_runs: arr(&cont),
            title,
            publication_scope: scope,
        }))
    }

    fn withheld(&self, sealed: &BTreeSet<Partition>, c: &ChunkRow) -> bool {
        self.lockbox.withholds(
            sealed,
            c.outcome_bearing,
            &c.describes_runs,
            &c.continues_runs,
            &c.publication_scope,
        )
    }

    fn evidence_ref(&self, c: &ChunkRow, quote: bool) -> ServiceResult<EvidenceRef> {
        let class: EvidenceClass =
            serde_json::from_value(json!(c.evidence_class)).map_err(|e| internal(e.into()))?;
        let pv = c.locator.parse_view_id.clone().ok_or_else(|| {
            err(
                ErrorCode::SourceMissing,
                format!(
                    "{}: legacy locator without a parse view; re-ingest the source",
                    c.id
                ),
            )
        })?;
        let e = EvidenceRef {
            evidence_id: EvidenceId::new(c.id.clone())?,
            source_id: SourceId::new(format!("sha256:{}", c.locator.asset_sha256))?,
            parse_view_id: Some(ParseViewId::new(pv)?),
            locator: Some(c.locator.clone()),
            evidence_class: class,
            quote: quote.then(|| c.text.clone()),
            origin: Origin::System,
        };
        e.validate()?;
        Ok(e)
    }

    /// The ranked order of a request under a lockbox (cached by request digest).
    fn ranked(
        &self,
        digest: &str,
        query: &str,
        filters: &Filters,
        opts: &SearchOpts,
        sealed: &BTreeSet<Partition>,
    ) -> ServiceResult<Arc<Ranked>> {
        if let Some((_, r)) = self
            .orders
            .lock()
            .expect("orders")
            .iter()
            .find(|(k, _)| k == digest)
        {
            return Ok(r.clone());
        }
        let keep = |id: &str| match self.chunk(id) {
            Ok(Some(c)) => !self.withheld(sealed, &c),
            _ => false, // unreadable metadata: withhold (fail closed)
        };
        let ranked = self
            .retriever
            .lock()
            .expect("retriever")
            .rank(query, filters, opts, &keep)
            .map_err(|e| match e.to_string().as_str() {
                "empty query" => err(
                    ErrorCode::InvalidRequest,
                    "the query has no searchable terms",
                ),
                _ => internal(e),
            })?;
        self.record_usage(&ranked, query);
        let r = Arc::new(ranked);
        let mut q = self.orders.lock().expect("orders");
        q.push_back((digest.to_string(), r.clone()));
        while q.len() > ORDER_CACHE {
            q.pop_front();
        }
        Ok(r)
    }

    fn record_usage(&self, r: &Ranked, query: &str) {
        if !self.ledger || r.usage.is_empty() {
            return;
        }
        let sum = |k: &str| r.usage.values().filter_map(|x| x[k].as_f64()).sum::<f64>();
        if let Ok(prof) = ignis_core::resources::load_profile("cpu-local") {
            if let Ok(mut job) = ignis_core::resources::Job::start(
                "evidence.search",
                &prof,
                json!({"query_chars": query.chars().count()}),
            ) {
                job.set_api_cost(sum("estimated_usd"), sum("billed_usd"));
                job.finish(json!({"stages": r.stages, "usage": r.usage, "timings": r.timings}));
            }
        }
    }

    /// Lexical matches past the fused depth, in BM25 order, skipping ranked and withheld chunks:
    /// the `skip`-th to `skip + take`-th such match (and whether one more exists).
    fn lexical_tail(
        &self,
        r: &Ranked,
        query: &str,
        filters: &Filters,
        sealed: &BTreeSet<Partition>,
        skip: usize,
        take: usize,
    ) -> ServiceResult<(Vec<RankedHit>, bool)> {
        let seen: BTreeSet<&str> = r.hits.iter().map(|h| h.chunk_id.as_str()).collect();
        let mut out = Vec::new();
        let mut kept = 0usize;
        let mut pos = r.lexical_depth;
        // Each window re-runs the ranked query (FTS5 + expansion, grouped and sorted in full), so
        // the window doubles: a page behind many withheld or filtered-out matches (lockbox
        // investigations) costs O(log n) passes instead of n/100. Same order, same pages.
        let mut window = r.lexical_depth.max(100);
        loop {
            let batch = {
                let ret = self.retriever.lock().expect("retriever");
                catalog::lexical_expanded(&ret.con, query, &r.expansions, filters, pos, window)
                    .map_err(internal)?
            };
            if batch.is_empty() {
                return Ok((out, false));
            }
            pos += batch.len();
            window = window.saturating_mul(2);
            for h in batch {
                if seen.contains(h.chunk_id.as_str()) {
                    continue;
                }
                match self.chunk(&h.chunk_id).map_err(internal)? {
                    Some(c) if !self.withheld(sealed, &c) => {}
                    _ => continue,
                }
                if kept >= skip {
                    if out.len() == take {
                        return Ok((out, true));
                    }
                    out.push(RankedHit {
                        chunk_id: h.chunk_id,
                        bm25: Some(h.bm25),
                        lexical_rank: Some(h.lexical_rank),
                        cosine: None,
                        dense_rank: None,
                        rrf: None,
                        rerank: None,
                    });
                }
                kept += 1;
            }
        }
    }

    fn filters(&self, req: &SearchRequest) -> Option<Filters> {
        let f = &req.filters;
        // the catalog holds documents (publications, protocols, calibration reports)
        let doc_kinds = [
            SourceKind::Publication,
            SourceKind::Protocol,
            SourceKind::Calibration,
        ];
        if !f.source_kinds.is_empty() && !f.source_kinds.iter().any(|k| doc_kinds.contains(k)) {
            return None;
        }
        let families: BTreeSet<String> = f
            .families
            .iter()
            .map(|x| evidence_family(*x).to_string())
            .collect();
        Some(Filters {
            families: families.into_iter().collect(),
            investigations: f.investigation_ids.clone(),
            source_sha256: f
                .source_ids
                .iter()
                .map(|s| s.as_str().trim_start_matches("sha256:").to_string())
                .collect(),
            ..Default::default()
        })
    }

    /// What this deployment can run besides lexical search: dense retrieval needs a vector set in
    /// the catalog and the provider key (to embed the query); reranking needs the key. Dense and
    /// rerank are out by decision (LANES_V2 R12a: lexical + SIRA), so normally both are false.
    fn deployed(&self) -> (bool, bool) {
        let ret = self.retriever.lock().expect("retriever");
        let key = ret.api_key.is_some();
        let vectors = catalog::embedding_set_id(&ret.con).ok().flatten().is_some();
        (vectors && key, key)
    }

    /// The backends a mode runs here: `lexical` = lexical; `dense` = dense (reported unavailable
    /// when it cannot run); `hybrid` = lexical + SIRA expansion, plus dense and rerank only where
    /// this deployment has them. A backend the deployment lacks is not requested by `hybrid`, so a
    /// default search never reports itself degraded for a stage that was never part of it.
    fn requested(mode: SearchMode, (dense, rerank): (bool, bool)) -> (bool, bool, bool) {
        match mode {
            SearchMode::Lexical => (true, false, false),
            SearchMode::Dense => (false, true, false),
            SearchMode::Hybrid => (true, dense, rerank),
        }
    }

    fn backends(
        &self,
        mode: SearchMode,
        r: Option<&Ranked>,
        sealed: &BTreeSet<Partition>,
    ) -> Vec<BackendReport> {
        let deployed = self.deployed();
        let (want_lex, want_dense, want_rerank) = Self::requested(mode, deployed);
        let absent = |b: &str| {
            (mode == SearchMode::Hybrid).then(|| {
                format!("{b} is not part of this deployment (hybrid = lexical + SIRA expansion)")
            })
        };
        let stage = |k: &str| {
            r.and_then(|r| r.stages.get(k))
                .and_then(Value::as_str)
                .map(String::from)
        };
        let report = |b: RetrievalBackend,
                      requested: bool,
                      used: bool,
                      detail: Option<String>,
                      not_requested: Option<String>| BackendReport {
            backend: b,
            status: match (requested, used) {
                (false, _) => BackendStatus::NotRequested,
                (true, true) => BackendStatus::Used,
                (true, false) => BackendStatus::Unavailable,
            },
            detail: if requested { detail } else { not_requested },
        };
        let lockbox = (!sealed.is_empty()).then(|| {
            format!(
                "scope lockbox: outcome-bearing passages about runs of {sealed:?} (or unattributed, \
                 in publications covering them) are withheld"
            )
        });
        vec![
            report(
                RetrievalBackend::Lexical,
                want_lex,
                r.is_some_and(|r| r.lexical_used),
                lockbox.or_else(|| stage("lexical")),
                None,
            ),
            report(
                RetrievalBackend::Dense,
                want_dense,
                r.is_some_and(|r| r.dense_used),
                stage("dense"),
                absent("dense retrieval").filter(|_| !deployed.0),
            ),
            report(
                RetrievalBackend::Rerank,
                want_rerank,
                r.is_some_and(|r| r.rerank_used),
                stage("rerank"),
                absent("reranking").filter(|_| !deployed.1),
            ),
        ]
    }

    fn hit(&self, rank: usize, h: &RankedHit) -> ServiceResult<SearchHit> {
        let c = self.chunk(&h.chunk_id).map_err(internal)?.ok_or_else(|| {
            err(
                ErrorCode::Internal,
                format!("ranked chunk {} vanished", h.chunk_id),
            )
        })?;
        let flat: String = c.text.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut snippet: String = flat.chars().take(SNIPPET_CHARS).collect();
        if flat.chars().count() > SNIPPET_CHARS {
            snippet.push('…');
        }
        let mut scores = Vec::new();
        let mut add = |kind: ScoreKind, v: Option<f64>| -> ServiceResult<()> {
            if let Some(v) = v {
                scores.push(RetrievalScore {
                    kind,
                    value: Finite::new(v).map_err(|e| err(ErrorCode::Internal, e.to_string()))?,
                });
            }
            Ok(())
        };
        add(ScoreKind::Bm25, h.bm25.map(|b| -b))?;
        add(ScoreKind::Cosine, h.cosine)?;
        add(ScoreKind::Rrf, h.rrf)?;
        add(ScoreKind::Rerank, h.rerank)?;
        Ok(SearchHit {
            rank: rank as u32,
            evidence: self.evidence_ref(&c, false)?,
            title: c.title.clone(),
            snippet,
            scores,
        })
    }

    fn publication_of(&self, source: &SourceId) -> ServiceResult<crate::records::Publication> {
        let sha = source.as_str().trim_start_matches("sha256:");
        let con = self.meta.lock().expect("meta");
        catalog::publications(&con)
            .map_err(internal)?
            .into_iter()
            .find(|p| p.source_sha256 == sha)
            .ok_or_else(|| {
                err(
                    ErrorCode::NotFound,
                    format!("source {source} is not in the corpus"),
                )
            })
    }
}

impl KnowledgeService for EvidenceService {
    fn search(&self, scope: &ExecutionScope, req: &SearchRequest) -> ServiceResult<SearchPage> {
        self.check_scope(scope)?;
        req.validate()?;
        if let Some(s) = &req.corpus_snapshot {
            self.check_snapshot(s)?;
        }
        let sealed = self.lockbox.sealed(scope);
        let page_size = req.page_size.unwrap_or(DEFAULT_PAGE_SIZE) as usize;
        let digest = sha256_hex(
            canonical_json(&json!({
                "snapshot": self.snapshot.snapshot_id, "query": req.query, "mode": req.mode,
                "filters": req.filters, "sealed": Lockbox::sealed_key(&sealed),
                "expected_terms": req.expected_terms,
            }))
            .as_bytes(),
        )[..32]
            .to_string();
        let cursor = match &req.cursor {
            None => None,
            Some(c) => {
                let c = Cursor::decode(c)
                    .ok_or_else(|| err(ErrorCode::InvalidRequest, "malformed cursor"))?;
                if c.snapshot != self.snapshot.snapshot_id {
                    return Err(err(
                        ErrorCode::Conflict,
                        "the cursor belongs to another corpus snapshot",
                    ));
                }
                if c.request != digest {
                    return Err(err(
                        ErrorCode::InvalidRequest,
                        "the cursor belongs to another search (query, mode, filters or scope differ)",
                    ));
                }
                Some(c)
            }
        };
        let offset = cursor.as_ref().map_or(0, |c| c.offset);
        let Some(filters) = self.filters(req) else {
            return Ok(SearchPage {
                schema_version: EvidenceSchema::V1,
                corpus_snapshot: self.snapshot_id(),
                mode: req.mode,
                hits: Vec::new(),
                next_cursor: None,
                total_count: Some(0),
                backends: self
                    .backends(req.mode, None, &sealed)
                    .into_iter()
                    .map(|mut b| {
                        if b.status == BackendStatus::Unavailable {
                            b.status = BackendStatus::NotRequested;
                            b.detail = Some("the requested source kinds are not documents".into());
                        }
                        b
                    })
                    .collect(),
                origin: Origin::System,
            });
        };
        // SIRA query side: the finalizer keeps the expected terms the corpus holds (sira.rs)
        let expansions: Vec<(String, f64)> = if req.expected_terms.is_empty() {
            Vec::new()
        } else {
            let proposals: Vec<(String, f64)> = req
                .expected_terms
                .iter()
                .map(|t| {
                    (
                        t.term.clone(),
                        ignis_core::contracts::TermImportance::weight(t.importance),
                    )
                })
                .collect();
            let ret = self.retriever.lock().expect("retriever");
            crate::sira::select_expressions(&ret.con, &proposals)
                .map_err(internal)?
                .into_iter()
                .filter_map(|c| c.weight.map(|w| (c.expression, w)))
                .collect()
        };
        let (use_lexical, use_dense, use_rerank) = Self::requested(req.mode, self.deployed());
        let opts = SearchOpts {
            limit: usize::MAX,
            use_lexical,
            use_dense,
            use_rerank,
            rerank_top: None,
            rerank_timeout_s: None,
            expansions,
        };
        let ranked = self.ranked(&digest, &req.query, &filters, &opts, &sealed)?;
        if cursor
            .as_ref()
            .is_some_and(|c| c.reranked != ranked.rerank_used)
        {
            // drop the order so a later retry can recompute it
            self.orders
                .lock()
                .expect("orders")
                .retain(|(k, _)| *k != digest);
            return Err(ErrorEnvelope::new(
                ErrorCode::Conflict,
                "the order of this cursor's first page cannot be reproduced (reranker availability \
                 changed); restart the search",
                RetryPolicy::AfterUserAction,
            ));
        }
        let n = ranked.hits.len();
        let mut picked: Vec<RankedHit> = ranked
            .hits
            .iter()
            .skip(offset)
            .take(page_size)
            .cloned()
            .collect();
        let tail_possible = ranked.lexical_used && ranked.lexical_count >= ranked.lexical_depth;
        let mut more = offset + picked.len() < n;
        if picked.len() < page_size && tail_possible {
            let skip = offset.saturating_sub(n);
            let (tail, tail_more) = self.lexical_tail(
                &ranked,
                &req.query,
                &filters,
                &sealed,
                skip,
                page_size - picked.len(),
            )?;
            picked.extend(tail);
            more = tail_more;
        }
        let hits = picked
            .iter()
            .enumerate()
            .map(|(i, h)| self.hit(offset + i + 1, h))
            .collect::<ServiceResult<Vec<_>>>()?;
        let next_cursor = more.then(|| {
            Cursor {
                snapshot: self.snapshot.snapshot_id.clone(),
                request: digest.clone(),
                offset: offset + hits.len(),
                reranked: ranked.rerank_used,
            }
            .encode()
        });
        let exhaustive = req.mode == SearchMode::Lexical && !tail_possible;
        let page = SearchPage {
            schema_version: EvidenceSchema::V1,
            corpus_snapshot: self.snapshot_id(),
            mode: req.mode,
            hits,
            next_cursor,
            total_count: exhaustive.then_some(n as u64),
            backends: self.backends(req.mode, Some(&ranked), &sealed),
            origin: Origin::System,
        };
        // A family filter never hides silently: documents left out only because they have no
        // family label yet (R8 labels fill them) are counted on the lexical backend's note.
        let unlabelled = {
            let ret = self.retriever.lock().expect("retriever");
            catalog::unlabelled_family_matches(&ret.con, &req.query, &filters).map_err(internal)?
        };
        let mut page = page;
        if unlabelled > 0 {
            if let Some(b) = page
                .backends
                .iter_mut()
                .find(|b| b.backend == RetrievalBackend::Lexical)
            {
                let note = format!(
                    "family filter: excluded_unlabelled: {unlabelled} documents match the query but have no family label yet; omit filters.families to include them"
                );
                b.detail = Some(match b.detail.take() {
                    Some(d) => format!("{d}; {note}"),
                    None => note,
                });
            }
        }
        page.validate()?;
        Ok(page)
    }

    /// Every label of one source (R8 `source_labels.jsonl` next to the catalog), conflicts
    /// included. Labels describe a document; they never resolve run identities.
    fn labels(
        &self,
        scope: &ExecutionScope,
        source: &SourceId,
    ) -> ServiceResult<Vec<ignis_core::contracts::SourceLabel>> {
        self.check_scope(scope)?;
        self.publication_of(source)?;
        let path = self.summaries_path.with_file_name(LABELS_FILE);
        let all = crate::labels::read_labels(&path).map_err(internal)?;
        Ok(all.into_iter().filter(|l| &l.source_id == source).collect())
    }

    /// The section tree of the source's active view (R7b, [`crate::outline`]). Headings a scope's
    /// lockbox withholds are left out (their section merges into the one before), so a heading
    /// never reveals what the passages under it would not.
    fn outline(
        &self,
        scope: &ExecutionScope,
        req: &OutlineRequest,
    ) -> ServiceResult<SourceOutline> {
        self.check_scope(scope)?;
        req.validate()?;
        let p = self.publication_of(&req.source_id)?;
        let active = parse_view_id(&p.id, &p.parser_version);
        if let Some(v) = &req.parse_view_id {
            if v.as_str() != active {
                return Err(err(
                    ErrorCode::Unsupported,
                    format!("outlines are built from the active view ({active}), not {v}"),
                ));
            }
        }
        let chunks = {
            let con = self.meta.lock().expect("meta");
            crate::outline::load_chunks(&con, &p.id).map_err(internal)?
        };
        let sealed = self.lockbox.sealed(scope);
        let mut pub_scope = p.investigation_ids.clone();
        pub_scope.extend(p.families.iter().cloned());
        let total = chunks.len();
        let visible: Vec<_> = chunks
            .into_iter()
            .filter(|c| {
                !self.lockbox.withholds(
                    &sealed,
                    c.outcome_bearing,
                    &c.describes_runs,
                    &c.continues_runs,
                    &pub_scope,
                )
            })
            .collect();
        let visible_all = visible.len() == total;
        let (heads, _method) = crate::outline::headings(&visible);
        let title = p
            .title
            .clone()
            .or_else(|| p.file_name.clone())
            .unwrap_or_else(|| "(whole document)".into());
        let roots = crate::outline::build(&heads, p.n_pages, &title).map_err(internal)?;
        let mut roots = roots;
        // Summaries are generated text about whole sections: shown only when this scope's lockbox
        // withholds nothing of the document, so they can never paraphrase a withheld passage.
        if visible_all {
            let summaries = self.summaries.get_or_init(|| {
                crate::outline::load_summaries(&self.summaries_path).unwrap_or_else(|e| {
                    eprintln!("outline summaries not loaded: {e:#}");
                    Default::default()
                })
            });
            crate::outline::attach(&mut roots, &active, summaries);
        }
        let nodes =
            crate::outline::select(roots, req.node_id.as_ref(), req.depth).ok_or_else(|| {
                err(
                    ErrorCode::NotFound,
                    format!(
                        "{} is not a section of {} (node ids come from sources_outline)",
                        req.node_id.as_ref().map(|n| n.as_str()).unwrap_or_default(),
                        req.source_id
                    ),
                )
            })?;
        let out = SourceOutline {
            schema_version: EvidenceSchema::V2,
            source_id: req.source_id.clone(),
            parse_view_id: ParseViewId::new(active)?,
            nodes,
            origin: Origin::System,
        };
        out.validate()?;
        Ok(out)
    }

    fn read(&self, scope: &ExecutionScope, req: &ReadRequest) -> ServiceResult<ReadResult> {
        self.check_scope(scope)?;
        req.validate()?;
        let p = self.publication_of(&req.source_id)?;
        let active = parse_view_id(&p.id, &p.parser_version);
        // An older view stays readable, so a citation made against it still opens.
        let view = match &req.parse_view_id {
            None => active.clone(),
            Some(v) if v.as_str() == active => active.clone(),
            Some(v) => {
                let stored = {
                    let con = self.meta.lock().expect("meta");
                    catalog::views(&con, &p.id).map_err(internal)?
                };
                if !stored.iter().any(|s| s == v.as_str()) {
                    return Err(err(
                        ErrorCode::NotFound,
                        format!(
                            "parse view {v} is not stored (active view of this source: {active})"
                        ),
                    ));
                }
                v.as_str().to_string()
            }
        };
        let sealed = self.lockbox.sealed(scope);
        let view_id = ParseViewId::new(view.clone())?;
        let page_locator = |page: i64, len: usize| SourceLocator {
            asset_sha256: p.source_sha256.clone(),
            path: p.file_name.clone(),
            page: Some(page),
            char_start: Some(0),
            char_end: Some(len as i64),
            parse_view_id: Some(view.clone()),
            offset_unit: Some(OffsetUnit::UnicodeScalar),
            ..Default::default()
        };
        let mut segments = Vec::new();
        let (complete, next) = match &req.range {
            ReadRange::Pages { first, last } => {
                let (first, last) = (*first as i64, (*last as i64).min(p.n_pages));
                if first > p.n_pages {
                    return Err(err(
                        ErrorCode::InvalidRequest,
                        format!("page {first} is past the last page ({})", p.n_pages),
                    ));
                }
                let pages = {
                    let con = self.meta.lock().expect("meta");
                    catalog::page_texts(&con, &p.id, &view, first, last).map_err(internal)?
                };
                if pages.len() as i64 != last - first + 1 {
                    return Err(err(
                        ErrorCode::SourceMissing,
                        format!(
                            "{}: page texts are not stored for this parse view; re-ingest",
                            p.id
                        ),
                    ));
                }
                let mut used = 0usize;
                let mut next_page = None;
                for (page, text) in pages {
                    if used > 0 && used + text.chars().count() > READ_BUDGET_CHARS {
                        next_page = Some(page);
                        break;
                    }
                    used += text.chars().count();
                    let ids = {
                        let con = self.meta.lock().expect("meta");
                        catalog::chunk_ids_on_page(&con, &p.id, &view, page).map_err(internal)?
                    };
                    let chunks = ids
                        .iter()
                        .map(|i| self.chunk(i))
                        .collect::<anyhow::Result<Vec<_>>>()
                        .map_err(internal)?
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>();
                    if chunks.iter().any(|c| self.withheld(&sealed, c)) {
                        // a page with withheld passages is returned passage by passage
                        for c in chunks.iter().filter(|c| !self.withheld(&sealed, c)) {
                            segments.push(ReadSegment {
                                locator: c.locator.clone(),
                                text: c.text.clone(),
                            });
                        }
                    } else {
                        segments.push(ReadSegment {
                            locator: page_locator(page, text.chars().count()),
                            text,
                        });
                    }
                }
                match next_page {
                    Some(np) => (
                        false,
                        Some(ReadRange::Pages {
                            first: np as u32,
                            last: last as u32,
                        }),
                    ),
                    None => (true, None),
                }
            }
            ReadRange::Passage { evidence_id } => {
                let c = self
                    .chunk(evidence_id.as_str())
                    .map_err(internal)?
                    .filter(|c| c.publication_id == p.id)
                    .ok_or_else(|| {
                        err(
                            ErrorCode::NotFound,
                            format!("{evidence_id} is not a passage of {}", req.source_id),
                        )
                    })?;
                if self.withheld(&sealed, &c) {
                    return Err(err(
                        ErrorCode::ScopeDenied,
                        format!(
                            "{evidence_id}: outcome-bearing passage outside the scope's partitions"
                        ),
                    ));
                }
                let ids = {
                    let con = self.meta.lock().expect("meta");
                    catalog::chunk_ids_on_page(&con, &p.id, &chunk_view(&c.id), c.page)
                        .map_err(internal)?
                };
                let pos = ids.iter().position(|i| *i == c.id).unwrap_or(0);
                for i in pos.saturating_sub(1)..(pos + 2).min(ids.len()) {
                    if let Some(n) = self.chunk(&ids[i]).map_err(internal)? {
                        if !self.withheld(&sealed, &n) {
                            segments.push(ReadSegment {
                                locator: n.locator.clone(),
                                text: n.text.clone(),
                            });
                        }
                    }
                }
                (true, None)
            }
            ReadRange::Table { table_id } => {
                return Err(err(
                    ErrorCode::Unsupported,
                    format!("table {table_id}: parse view {view} detects no tables"),
                ))
            }
        };
        let r = ReadResult {
            schema_version: EvidenceSchema::V1,
            source_id: req.source_id.clone(),
            parse_view_id: view_id,
            segments,
            complete,
            next,
            origin: Origin::System,
        };
        r.validate()?;
        Ok(r)
    }

    fn resolve_evidence(
        &self,
        scope: &ExecutionScope,
        id: &EvidenceId,
    ) -> ServiceResult<EvidenceRef> {
        self.check_scope(scope)?;
        let c = self.chunk(id.as_str()).map_err(internal)?.ok_or_else(|| {
            err(
                ErrorCode::NotFound,
                format!(
                    "evidence {id} is not in corpus snapshot {}",
                    self.snapshot.snapshot_id
                ),
            )
        })?;
        if self.withheld(&self.lockbox.sealed(scope), &c) {
            return Err(err(
                ErrorCode::ScopeDenied,
                format!("{id}: outcome-bearing passage outside the scope's partitions"),
            ));
        }
        self.evidence_ref(&c, true)
    }
}

/// Group a page's withheld state for reports (used by the CLI `corpus` report).
pub fn withheld_by_publication(
    svc: &EvidenceService,
    scope: &ExecutionScope,
) -> anyhow::Result<BTreeMap<String, (usize, usize)>> {
    let sealed = svc.lockbox.sealed(scope);
    let ids: Vec<String> = {
        let con = svc.meta.lock().expect("meta");
        let mut st = con.prepare("SELECT id FROM chunks ORDER BY rowid")?;
        let v = st
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        v
    };
    let mut out: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for id in ids {
        if let Some(c) = svc.chunk(&id)? {
            let e = out.entry(c.publication_id.clone()).or_default();
            e.0 += 1;
            if svc.withheld(&sealed, &c) {
                e.1 += 1;
            }
        }
    }
    Ok(out)
}

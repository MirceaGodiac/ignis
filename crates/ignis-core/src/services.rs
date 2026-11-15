//! Domain service interfaces of the C0 surface. Each lane implements its service in its own crate;
//! the composition root (`ignis-cli`, Mircea) injects them into the HTTP API, the MCP transport and
//! the CLI, which only project these calls. No HTTP, MCP, SQLite or async runtime types appear here.
//!
//! The traits are synchronous on purpose: implementations are CPU-bound (forward passes, scoring) or
//! do local blocking I/O (Tantivy, SQLite, Parquet). Async transports call them on a bounded
//! blocking pool (`tokio::task::spawn_blocking` or a dedicated rayon pool), never on reactor
//! threads, and long calls become jobs. The `ExecutionScope` is always a separate argument derived
//! by the host; implementations must check it and must not accept a scope from request bodies.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

use crate::contracts::{
    AnalysisArtifact, ArtifactId, ArtifactRef, Author, ColumnSpec, ConditionsQuery,
    ConditionsTable, DatasetBundle, DatasetId, Digest, EntityRef, EpisodeId, ErrorCode,
    ErrorEnvelope, EvaluationRequest, EvaluationResult, EvidenceId, EvidenceRef, ExecutionScope,
    ExternalSourceCandidate, ForecastRequest, GraphNeighborsPage, GraphNeighborsRequest, KeyValue,
    MetricValue, ModelRelease, Origin, OutlineRequest, Partition, Producer, ReadRequest,
    ReadResult, ReleaseId, RetryPolicy, RunId, SearchPage, SearchRequest, SourceAccess, SourceId,
    SourceLabel, SourceOutline, WebFetchRequest, WebSearchPage, WebSearchRequest, WikiPatch,
    WikiReadRequest, WikiReadResult, WikiRevision,
};

pub type ServiceResult<T> = Result<T, ErrorEnvelope>;

/// Default body of a trait method added after C0: the implementation does not provide it yet. The
/// host maps it to "not configured" in `/capabilities`, and the tool is not registered.
pub fn not_provided<T>(what: &str) -> ServiceResult<T> {
    Err(ErrorEnvelope::new(
        ErrorCode::CapabilityUnavailable,
        format!("{what} is not provided by this service"),
        RetryPolicy::Never,
    ))
}

/// Corpus search and reading, the combustion graph, document labels, the wiki and web discovery
/// (Raul). One implementation serves CLI, HTTP and MCP. Methods after `resolve_evidence` (wave K1)
/// default to `capability_unavailable`, so existing implementations keep compiling.
pub trait KnowledgeService: Send + Sync {
    fn search(&self, scope: &ExecutionScope, req: &SearchRequest) -> ServiceResult<SearchPage>;
    fn read(&self, scope: &ExecutionScope, req: &ReadRequest) -> ServiceResult<ReadResult>;
    fn resolve_evidence(
        &self,
        scope: &ExecutionScope,
        id: &EvidenceId,
    ) -> ServiceResult<EvidenceRef>;

    fn graph_neighbors(
        &self,
        _scope: &ExecutionScope,
        _req: &GraphNeighborsRequest,
    ) -> ServiceResult<GraphNeighborsPage> {
        not_provided("graph_neighbors")
    }

    /// The section tree of one source (`sources_outline`). Summaries are generated text, not evidence.
    fn outline(
        &self,
        _scope: &ExecutionScope,
        _req: &OutlineRequest,
    ) -> ServiceResult<SourceOutline> {
        not_provided("sources_outline")
    }

    /// How to open the original without IGNIS storing it (mode per source; a signed URL is
    /// resolved at call time and never stored in a citation).
    fn access(&self, _scope: &ExecutionScope, _source: &SourceId) -> ServiceResult<SourceAccess> {
        not_provided("source access")
    }

    /// Every label of one source (one per labelling batch), conflicts included.
    fn labels(
        &self,
        _scope: &ExecutionScope,
        _source: &SourceId,
    ) -> ServiceResult<Vec<SourceLabel>> {
        not_provided("labels")
    }

    /// Items the scope may not read are withheld and counted, never silently dropped.
    fn wiki_read(
        &self,
        _scope: &ExecutionScope,
        _req: &WikiReadRequest,
    ) -> ServiceResult<WikiReadResult> {
        not_provided("wiki_read")
    }

    /// Compare-and-swap on `base_revision` (`conflict` when the head moved). `author` and `episode`
    /// come from the host, as does the access of the written items (from `scope`).
    fn wiki_patch(
        &self,
        _scope: &ExecutionScope,
        _author: &Author,
        _episode: Option<&EpisodeId>,
        _patch: &WikiPatch,
    ) -> ServiceResult<WikiRevision> {
        not_provided("wiki_patch")
    }

    /// Refused (scope_denied) for evaluation scopes and without `network_fetch`.
    fn web_search(
        &self,
        _scope: &ExecutionScope,
        _req: &WebSearchRequest,
    ) -> ServiceResult<WebSearchPage> {
        not_provided("web_search")
    }

    /// Fetches under the SSRF policy into a snapshot artifact; never promotes into the corpus.
    fn web_fetch(
        &self,
        _scope: &ExecutionScope,
        _req: &WebFetchRequest,
    ) -> ServiceResult<ExternalSourceCandidate> {
        not_provided("web_fetch")
    }
}

/// Curated datasets and experimental conditions (Raul).
pub trait DatasetService: Send + Sync {
    /// Refuses (scope_denied) bundles whose partitions the scope does not grant.
    fn dataset(&self, scope: &ExecutionScope, id: &DatasetId) -> ServiceResult<DatasetBundle>;

    /// Per-run conditions (inputs, not outcomes) and tested envelopes (wave K1).
    fn conditions(
        &self,
        _scope: &ExecutionScope,
        _q: &ConditionsQuery,
    ) -> ServiceResult<ConditionsTable> {
        not_provided("conditions_query")
    }

    /// Identity of the data [`DatasetService::conditions`] answers from (the conditions table and
    /// every input that decides which runs a scope sees). It changes when, and only when, an
    /// answer could change, so a citation of a cell resolves later against exactly this data.
    fn conditions_snapshot(&self, _scope: &ExecutionScope) -> ServiceResult<Digest> {
        not_provided("conditions_snapshot")
    }

    /// The partitions `run` holds in the frozen splits (one per split that assigns it, deduplicated;
    /// empty when no split assigns it a partition). The host checks every one against the scope
    /// before it mounts anything whose lineage names the run (M9); an empty answer is an unknown
    /// partition there and is refused.
    fn run_partitions(
        &self,
        _scope: &ExecutionScope,
        _run: &RunId,
    ) -> ServiceResult<BTreeSet<Partition>> {
        not_provided("run_partitions")
    }

    /// Every bundle the scope may read, ordered by id (metadata only; the numeric tables stay
    /// behind their view `ArtifactRef`s). Bundles holding a partition the scope does not grant are
    /// not listed, and the evaluator root is listed for evaluation scopes only (M11).
    fn datasets(&self, _scope: &ExecutionScope) -> ServiceResult<Vec<DatasetBundle>> {
        not_provided("datasets_list")
    }

    /// Bytes `[offset, offset + len)` of one view's artifact (Parquet), streamed, after the same
    /// scope checks as [`DatasetService::dataset`]; returns the view's `ArtifactRef` with the reader.
    fn open_view(
        &self,
        _scope: &ExecutionScope,
        _id: &DatasetId,
        _view: &str,
        _offset: u64,
        _len: Option<u64>,
    ) -> ServiceResult<(ArtifactRef, Box<dyn Read + Send>)> {
        not_provided("dataset_view_bytes")
    }

    /// Rows of one bundle view that match `q.keys` (identity columns, never a row index), read from
    /// the view whose artifact has digest `q.view_sha256` (the pinned snapshot), after the same
    /// scope checks as [`DatasetService::dataset`]. Streams the view and stops after `q.limit`
    /// matches (`truncated` says more exist). An unknown key or column fails closed and names the
    /// permitted ones. Used to ground a number cited from a view's cell (M7b).
    fn view_rows(&self, _scope: &ExecutionScope, _q: &ViewRowsQuery) -> ServiceResult<ViewRows> {
        not_provided("dataset_view_rows")
    }
}

/// A key-predicate read of one dataset view ([`DatasetService::view_rows`]).
#[derive(Debug, Clone, PartialEq)]
pub struct ViewRowsQuery {
    /// Digest of the view's Parquet artifact: the snapshot the reader pinned.
    pub view_sha256: Digest,
    /// `column == value`, all must hold; at least one, on identity columns (run id, condition,
    /// time, flag, locator), never on a measured value.
    pub keys: Vec<(String, KeyValue)>,
    /// Columns to return (besides the keys).
    pub columns: Vec<String>,
    /// Also return the unit columns the view has (`unit_si`, `raw_unit`, `unit`), so a value
    /// comes back with the unit the view states for it in the same scan.
    pub with_unit_columns: bool,
    /// Most rows returned.
    pub limit: usize,
}

/// One cell read by [`DatasetService::view_rows`], as stored.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewCell {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
}

/// What [`DatasetService::view_rows`] found.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewRows {
    pub dataset_id: DatasetId,
    pub view: String,
    pub artifact: ArtifactRef,
    /// The view's column specs (roles, SI units) for the columns returned.
    pub columns: Vec<ColumnSpec>,
    /// Each matching row: key and requested columns by name.
    pub rows: Vec<BTreeMap<String, ViewCell>>,
    /// More rows matched than `limit`.
    pub truncated: bool,
}

/// Model releases and strict forecasts (Iancu).
pub trait ModelService: Send + Sync {
    fn releases(&self, scope: &ExecutionScope) -> ServiceResult<Vec<ModelRelease>>;
    fn release(&self, scope: &ExecutionScope, id: &ReleaseId) -> ServiceResult<ModelRelease>;
    /// Runs the release's forward on the admitted prefix. Scientific refusals (out of domain,
    /// insufficient prefix) are artifacts with the corresponding status, not errors.
    fn forecast(
        &self,
        scope: &ExecutionScope,
        req: &ForecastRequest,
    ) -> ServiceResult<AnalysisArtifact>;
}

/// What a producer declares when storing bytes; the store computes digest and length.
#[derive(Debug, Clone)]
pub struct ArtifactDraft {
    pub media_type: String,
    pub contract: Option<String>,
    pub produced_by: Producer,
    pub inputs: Vec<EntityRef>,
    pub origin: Origin,
}

/// Immutable content-addressed bytes (Mircea). Streams in and out; never loads whole artifacts to
/// hand them around. Publication is atomic: a returned `ArtifactRef` never points at incomplete bytes.
pub trait ArtifactStore: Send + Sync {
    fn put(
        &self,
        scope: &ExecutionScope,
        draft: ArtifactDraft,
        bytes: &mut dyn Read,
    ) -> ServiceResult<ArtifactRef>;
    fn stat(&self, scope: &ExecutionScope, id: &ArtifactId) -> ServiceResult<ArtifactRef>;
    /// Reads `[offset, offset + len)` (to the end when `len` is `None`).
    fn open(
        &self,
        scope: &ExecutionScope,
        id: &ArtifactId,
        offset: u64,
        len: Option<u64>,
    ) -> ServiceResult<Box<dyn Read + Send>>;
}

/// Independent scoring (Tudor). Runs in the evaluator's own process and scope; a request for the
/// test partition needs an evaluation scope that grants it.
pub trait EvaluationService: Send + Sync {
    fn evaluate(
        &self,
        scope: &ExecutionScope,
        req: &EvaluationRequest,
    ) -> ServiceResult<EvaluationResult>;
    /// Development comparison of results on identical cases and metrics; refuses mismatches.
    fn compare(
        &self,
        scope: &ExecutionScope,
        results: &[EvaluationResult],
    ) -> ServiceResult<Vec<MetricValue>>;
}

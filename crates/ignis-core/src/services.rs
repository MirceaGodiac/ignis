//! Domain service interfaces of the C0 surface. Each lane implements its service in its own crate;
//! the composition root (`ignis-cli`, Mircea) injects them into the HTTP API, the MCP transport and
//! the CLI, which only project these calls. No HTTP, MCP, SQLite or async runtime types appear here.
//!
//! The traits are synchronous on purpose: implementations are CPU-bound (forward passes, scoring) or
//! do local blocking I/O (Tantivy, SQLite, Parquet). Async transports call them on a bounded
//! blocking pool (`tokio::task::spawn_blocking` or a dedicated rayon pool), never on reactor
//! threads, and long calls become jobs. The `ExecutionScope` is always a separate argument derived
//! by the host; implementations must check it and must not accept a scope from request bodies.

use std::io::Read;

use crate::contracts::{
    AnalysisArtifact, ArtifactId, ArtifactRef, DatasetBundle, DatasetId, EntityRef, ErrorEnvelope,
    EvaluationRequest, EvaluationResult, EvidenceId, EvidenceRef, ExecutionScope, ForecastRequest,
    MetricValue, ModelRelease, Origin, Producer, ReadRequest, ReadResult, ReleaseId, SearchPage,
    SearchRequest,
};

pub type ServiceResult<T> = Result<T, ErrorEnvelope>;

/// Corpus search and reading (Raul). One implementation serves CLI, HTTP and MCP.
pub trait KnowledgeService: Send + Sync {
    fn search(&self, scope: &ExecutionScope, req: &SearchRequest) -> ServiceResult<SearchPage>;
    fn read(&self, scope: &ExecutionScope, req: &ReadRequest) -> ServiceResult<ReadResult>;
    fn resolve_evidence(
        &self,
        scope: &ExecutionScope,
        id: &EvidenceId,
    ) -> ServiceResult<EvidenceRef>;
}

/// Curated datasets (Raul).
pub trait DatasetService: Send + Sync {
    /// Refuses (scope_denied) bundles whose partitions the scope does not grant.
    fn dataset(&self, scope: &ExecutionScope, id: &DatasetId) -> ServiceResult<DatasetBundle>;
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

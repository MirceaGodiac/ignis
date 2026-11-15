//! The tool dispatcher: one synchronous implementation of every entry of
//! [`tool_catalog`] over the lanes' services. Transports (the MCP server in [`crate::http`], and the
//! HTTP routes that share an operation with a tool) call it; none re-implements search, physics or
//! scoring.
//!
//! A call decodes its arguments into the contract its descriptor declares (`deny_unknown_fields`,
//! then `validate()`), checks the capabilities the descriptor requires against the host-derived
//! scope, calls the owner's service with that scope and returns a [`ToolResult`]: `completed` with the
//! service's answer, `accepted` with a job for long work, `refused` for scope/input/unsupported, and
//! `failed` for technical errors. Scientific outcomes stay in the payload; a wrapper never assesses
//! them (`scientific_status = not_assessed`).
//!
//! A retrieval call made for an episode (the runtime's grant names it) also records the evidence its
//! result carries, so `answer_submit` ([`crate::answer`]) resolves citations against exactly what that
//! episode's tools returned.
//!
//! A tool whose service is not configured is not registered at all, and is reported in capability
//! status (see [`catalog`]); it never answers with a negative result.

use std::sync::Arc;

use ignis_core::contracts::{
    tool_catalog, AnalysisExecuteRequest, AnswerSubmission, ArtifactReadRequest, ArtifactRef,
    Causation, ClientRequestId, ConditionsQuery, DatasetRef, EntityRef, EpisodeId, ErrorCode,
    ErrorEnvelope, EvidenceRef, ExecutionScope, ExecutionStatus, ForecastRequest, GroundingMode,
    InlinePayload, JobLookup, JobRef, Origin, OutlineRequest, Producer, ReadRequest, ReleaseRef,
    ResponseFormat, RetryPolicy, ScientificStatus, SearchRequest, ToolCallId, ToolDescriptor,
    ToolResult, ToolsSchema,
};
use ignis_core::services::{
    ArtifactDraft, ArtifactStore, DatasetService, KnowledgeService, ModelService, ServiceResult,
};
use ignis_research::sandbox::Sandbox;
use ignis_research::store::{JobPriority, NewJob};
use ignis_research::ResearchStore;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::answer::{AnswerHost, Submitted};

/// The tools whose results are evidence the knowledge/data services retrieved, and so the only ones
/// whose `EvidenceRef`s an episode may cite. Never `artifacts_read` or `jobs_get`: an artifact can
/// be written by agent-authored analysis code (a forged `EvidenceRef`) or belong to another episode
/// (its answer's evidence), and reading it must not make that evidence citable here.
const RETRIEVES_EVIDENCE: &[&str] = &[
    "sources_search",
    "sources_read",
    "datasets_inspect",
    "conditions_query",
];

/// Called after the host appended events to an episode log (wakes its SSE readers).
pub type CommitHook = Arc<dyn Fn(&EpisodeId, u64) + Send + Sync>;

/// Job kind of a strict forecast (the job runner executes it).
pub const FORECAST_JOB: &str = "models_forecast";

/// Job kind of an isolated analysis run (the job runner hands it to the [`Sandbox`]).
pub const ANALYSIS_JOB: &str = "analysis_execute";

/// Results larger than this (serialized) are stored as an artifact; the tool result carries a
/// preview that points at it, so a model context never receives an unbounded payload.
pub const INLINE_RESULT_BYTES: usize = 64 << 10;

/// Domain services the composition root injects (each optional until its lane delivers it).
#[derive(Default, Clone)]
pub struct ToolServices {
    pub knowledge: Option<Arc<dyn KnowledgeService>>,
    pub datasets: Option<Arc<dyn DatasetService>>,
    pub models: Option<Arc<dyn ModelService>>,
    /// The isolated analysis runner; present only when it passed preflight.
    pub sandbox: Option<Arc<Sandbox>>,
    /// The deployment switched the model families (FLEX/ACME/BASS) off. Their contracts and tool
    /// descriptors stay in the catalog; the model tools are not registered and report this reason.
    pub models_disabled: bool,
    /// The DatasetService can answer `conditions_query` (its conditions table is built). Without
    /// it the tool is not registered and capability status says why.
    pub conditions: bool,
}

/// Why `conditions_query` is absent while datasets are configured.
pub const CONDITIONS_NOT_BUILT: &str =
    "conditions table not built in this deployment (run `ignis data conditions`)";

/// Why the model tools are absent when the deployment switched the model families off.
pub const MODELS_DISABLED: &str =
    "model families (FLEX/ACME/BASS) disabled in this deployment (start with --enable-models)";

/// A catalog entry and whether this deployment can serve it.
#[derive(Debug, Clone)]
pub struct ToolAvailability {
    pub descriptor: ToolDescriptor,
    /// Why the tool is not registered, when it is not.
    pub unavailable: Option<&'static str>,
}

/// The catalog as this deployment serves it: the single source of `/capabilities` tool status and
/// of the MCP tool list.
pub fn catalog(services: &ToolServices) -> Vec<ToolAvailability> {
    tool_catalog()
        .into_iter()
        .map(|d| {
            let unavailable = match d.name.as_str() {
                "sources_search" | "sources_read" | "sources_outline"
                    if services.knowledge.is_none() =>
                {
                    Some("KnowledgeService not configured")
                }
                "datasets_inspect" | "conditions_query" if services.datasets.is_none() => {
                    Some("DatasetService not configured")
                }
                "conditions_query" if !services.conditions => Some(CONDITIONS_NOT_BUILT),
                "models_inspect" | "models_forecast" if services.models_disabled => {
                    Some(MODELS_DISABLED)
                }
                "models_inspect" | "models_forecast" if services.models.is_none() => {
                    Some("ModelService not configured")
                }
                "analysis_execute" if services.sandbox.is_none() => {
                    Some("isolated runner not configured or failed preflight")
                }
                "wiki_patch" => Some("wiki service not implemented"),
                "sources_search" | "sources_read" | "sources_outline" | "datasets_inspect"
                | "conditions_query" | "models_inspect" | "models_forecast"
                | "analysis_execute" | "artifacts_read" | "jobs_get" | "answer_submit" => None,
                // graph_neighbors, wiki_read, web_search, web_fetch: their
                // KnowledgeService methods are not implemented on main yet (trait defaults).
                _ => Some("no adapter for this tool"),
            };
            ToolAvailability {
                descriptor: d,
                unavailable,
            }
        })
        .collect()
}

/// What a tool call needs from the host besides the services.
pub struct ToolHost {
    services: ToolServices,
    store: Arc<ResearchStore>,
    artifacts: Arc<dyn ArtifactStore>,
    /// Origin of what this host creates (`synthetic_fixture` in fixture mode).
    origin: Origin,
    /// Wakes the job runner after a submission.
    wake_jobs: Arc<dyn Fn() + Send + Sync>,
    /// Wakes episode readers after `answer_submit` appended `answer.completed`.
    on_commit: Option<CommitHook>,
    /// What `answer_submit` does with a number that does not ground (M7b); `log` by default.
    grounding: GroundingMode,
    registered: Vec<ToolDescriptor>,
}

fn envelope(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

/// A refusal is the host or service declining (scope, input, unsupported, missing); anything else
/// is a technical failure.
fn refusal(code: ErrorCode) -> bool {
    matches!(
        code,
        ErrorCode::InvalidRequest
            | ErrorCode::ScopeDenied
            | ErrorCode::NotFound
            | ErrorCode::SourceMissing
            | ErrorCode::SourceGone
            | ErrorCode::Unsupported
            | ErrorCode::CapabilityUnavailable
            | ErrorCode::Conflict
    )
}

/// Decode tool arguments; a failure names the JSON path of the offending field (`claims[0].kind`),
/// so the model fixes that field instead of guessing where it belongs.
fn decode<T: DeserializeOwned>(contract: &str, args: Value) -> ServiceResult<T> {
    serde_path_to_error::deserialize(args).map_err(|e| {
        let path = e.path().to_string();
        let at = if path.is_empty() || path == "." {
            "at the top level".to_string()
        } else {
            format!("at `{path}`")
        };
        envelope(
            ErrorCode::InvalidRequest,
            format!(
                "arguments are not a valid {contract}: {at}: {}. The tool's input schema is the \
                 {contract} contract; fix that field and call again.",
                e.inner()
            ),
        )
    })
}

/// `concise` (the default) or `detailed`. It changes the size of each item, never which items
/// are returned or their order (`ResponseFormat`).
fn concise(f: Option<ResponseFormat>) -> bool {
    f.unwrap_or(ResponseFormat::Concise) == ResponseFormat::Concise
}

fn checked(r: Result<(), ignis_core::contracts::ContractError>) -> ServiceResult<()> {
    r.map_err(ErrorEnvelope::from)
}

fn value<T: serde::Serialize>(v: ServiceResult<T>) -> ServiceResult<Outcome> {
    v.and_then(|x| {
        serde_json::to_value(x)
            .map(Outcome::Value)
            .map_err(|e| envelope(ErrorCode::Internal, e.to_string()))
    })
}

fn new_call_id() -> ToolCallId {
    ToolCallId::new(format!("call_{}", uuid::Uuid::new_v4().simple())).expect("generated id")
}

/// What one tool produced before it is shaped into a [`ToolResult`].
enum Outcome {
    Value(Value),
    Inline(InlinePayload),
    Job(JobRef),
    /// A value already stored as `artifact` (inline when small, otherwise a preview of it).
    Stored {
        value: Value,
        artifact: ArtifactRef,
    },
    /// Declined, with an artifact that details why (e.g. an `AnswerRefusal`).
    Refused {
        error: ErrorEnvelope,
        artifact: ArtifactRef,
    },
}

impl ToolHost {
    pub fn new(
        services: ToolServices,
        store: Arc<ResearchStore>,
        artifacts: Arc<dyn ArtifactStore>,
        origin: Origin,
        wake_jobs: Arc<dyn Fn() + Send + Sync>,
    ) -> Self {
        let registered = catalog(&services)
            .into_iter()
            .filter(|t| t.unavailable.is_none())
            .map(|t| t.descriptor)
            .collect();
        ToolHost {
            services,
            store,
            artifacts,
            origin,
            wake_jobs,
            on_commit: None,
            grounding: GroundingMode::default(),
            registered,
        }
    }

    /// Wake episode readers when a tool appends to an episode log.
    pub fn with_commit_hook(mut self, hook: CommitHook) -> Self {
        self.on_commit = Some(hook);
        self
    }

    /// Refuse answers whose numbers do not ground (`enforce`), or store them with the problems
    /// recorded (`log`, the default).
    pub fn with_grounding(mut self, mode: GroundingMode) -> Self {
        self.grounding = mode;
        self
    }

    /// The tools this host serves.
    pub fn registered(&self) -> &[ToolDescriptor] {
        &self.registered
    }

    /// Run one call with the host-derived `scope`. Never panics on input; always returns a valid
    /// [`ToolResult`].
    pub fn call(&self, scope: &ExecutionScope, tool: &str, args: Value) -> ToolResult {
        self.call_in(scope, None, tool, args)
    }

    /// [`Self::call`] for the episode a runtime grant is bound to (host-derived, never an argument).
    pub fn call_in(
        &self,
        scope: &ExecutionScope,
        episode: Option<&EpisodeId>,
        tool: &str,
        args: Value,
    ) -> ToolResult {
        let call_id = new_call_id();
        let outcome = match self.registered.iter().find(|d| d.name == tool) {
            None => Err(envelope(
                ErrorCode::NotFound,
                format!(
                    "no tool {tool:?} is registered in this deployment; registered: {}",
                    self.registered
                        .iter()
                        .map(|d| d.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )),
            Some(d) => d
                .capabilities
                .iter()
                .try_for_each(|c| scope.require(*c).map_err(ErrorEnvelope::from))
                .and_then(|()| self.dispatch(scope, episode, d, &call_id, args)),
        };
        if let (Some(ep), Ok(o), true) = (episode, &outcome, RETRIEVES_EVIDENCE.contains(&tool)) {
            self.record_evidence(scope, ep, o);
        }
        self.shape(scope, call_id, tool, outcome)
    }

    /// Remember the evidence a result carries for the episode (best effort: a lost record makes a
    /// later citation of it refused, never accepted).
    fn record_evidence(&self, scope: &ExecutionScope, ep: &EpisodeId, o: &Outcome) {
        let v = match o {
            Outcome::Value(v) => v,
            Outcome::Inline(p) => &p.value,
            _ => return,
        };
        let mut found = Vec::new();
        collect_evidence(v, &mut found);
        if let Err(e) = self.store.record_episode_evidence(scope, ep, &found) {
            tracing::warn!(episode = %ep, error = %e.message, "evidence of a tool result not recorded");
        }
    }

    fn dispatch(
        &self,
        scope: &ExecutionScope,
        episode: Option<&EpisodeId>,
        d: &ToolDescriptor,
        call_id: &ToolCallId,
        args: Value,
    ) -> ServiceResult<Outcome> {
        let contract = d.input_contract.as_str();
        match d.name.as_str() {
            "sources_search" => {
                let req: SearchRequest = decode(contract, args)?;
                checked(req.validate())?;
                let mut page = self.knowledge()?.search(scope, &req)?;
                if concise(req.response_format) {
                    // Ranking scores are for comparing hits, not for citing them.
                    page.hits.iter_mut().for_each(|h| h.scores.clear());
                }
                value(Ok(page))
            }
            "sources_read" => {
                let req: ReadRequest = decode(contract, args)?;
                checked(req.validate())?;
                value(self.knowledge()?.read(scope, &req))
            }
            "sources_outline" => {
                let req: OutlineRequest = decode(contract, args)?;
                checked(req.validate())?;
                value(self.knowledge()?.outline(scope, &req))
            }
            "datasets_inspect" => {
                let req: DatasetRef = decode(contract, args)?;
                value(self.datasets()?.dataset(scope, &req.dataset_id))
            }
            "conditions_query" => {
                let q: ConditionsQuery = decode(contract, args)?;
                checked(q.validate())?;
                value(self.datasets()?.conditions(scope, &q))
            }
            "models_inspect" => {
                let req: ReleaseRef = decode(contract, args)?;
                value(self.models()?.release(scope, &req.release_id))
            }
            "models_forecast" => {
                let req: ForecastRequest = decode(contract, args)?;
                self.models()?;
                let job = self.submit_forecast(scope, &req, None, None, call_id)?;
                Ok(Outcome::Job(job))
            }
            "analysis_execute" => {
                let req: AnalysisExecuteRequest = decode(contract, args)?;
                checked(req.validate())?;
                self.sandbox()?;
                // Refuse a reference the scope cannot read (partition, lineage) before queueing.
                ignis_research::sandbox::check_inputs(
                    scope,
                    &req,
                    self.artifacts.as_ref(),
                    self.services.datasets.as_deref(),
                )?;
                let job = self.submit_analysis(scope, &req, call_id)?;
                Ok(Outcome::Job(job))
            }
            "artifacts_read" => {
                let req: ArtifactReadRequest = decode(contract, args)?;
                ignis_research::artifacts::read_for_tool(self.artifacts.as_ref(), scope, &req)
                    .map(Outcome::Inline)
            }
            "jobs_get" => {
                let req: JobLookup = decode(contract, args)?;
                value(self.store.job(scope, &req.job_id))
            }
            "answer_submit" => {
                let ep = episode.ok_or_else(|| {
                    envelope(
                        ErrorCode::InvalidRequest,
                        "answer_submit stores the answer of an episode; this session's grant is not bound to one",
                    )
                })?;
                let sub: AnswerSubmission = decode(contract, args)?;
                let host = AnswerHost {
                    store: &self.store,
                    artifacts: self.artifacts.as_ref(),
                    origin: self.origin,
                    grounding: self.grounding,
                    datasets: self.services.datasets.as_deref(),
                };
                match host.submit(scope, ep, call_id, sub)? {
                    Submitted::Stored {
                        answer,
                        artifact,
                        appended,
                    } => {
                        if let (Some(seq), Some(hook)) = (appended, &self.on_commit) {
                            hook(ep, seq);
                        }
                        let value = serde_json::to_value(&answer)
                            .map_err(|e| envelope(ErrorCode::Internal, e.to_string()))?;
                        Ok(Outcome::Stored { value, artifact })
                    }
                    Submitted::Refused { error, refusal } => Ok(Outcome::Refused {
                        error,
                        artifact: refusal,
                    }),
                }
            }
            other => Err(envelope(
                ErrorCode::Unsupported,
                format!("no adapter for tool {other}"),
            )),
        }
    }

    /// Queue a strict forecast for the job runner. Shared with `POST /api/v1/analyses`; idempotent on
    /// the request's `client_request_id` (or `idempotency_key`).
    pub fn submit_forecast(
        &self,
        scope: &ExecutionScope,
        req: &ForecastRequest,
        episode: Option<EpisodeId>,
        idempotency_key: Option<ClientRequestId>,
        call_id: &ToolCallId,
    ) -> ServiceResult<JobRef> {
        checked(req.validate())?;
        let input =
            serde_json::to_value(req).map_err(|e| envelope(ErrorCode::Internal, e.to_string()))?;
        let job = self.store.submit_job(
            scope,
            NewJob {
                kind: FORECAST_JOB.into(),
                input,
                episode_id: episode,
                client_request_id: req.client_request_id.clone().or(idempotency_key),
                idempotent: true,
                priority: JobPriority::Interactive,
                causation: Causation {
                    tool_call_id: Some(call_id.clone()),
                    ..Default::default()
                },
            },
        )?;
        (self.wake_jobs)();
        Ok(job)
    }

    /// Queue an isolated analysis run. Idempotent on the request's `client_request_id`. Inputs are
    /// checked when the job runs, with the same scope (an artifact of another workspace is not
    /// found there).
    pub fn submit_analysis(
        &self,
        scope: &ExecutionScope,
        req: &AnalysisExecuteRequest,
        call_id: &ToolCallId,
    ) -> ServiceResult<JobRef> {
        let input =
            serde_json::to_value(req).map_err(|e| envelope(ErrorCode::Internal, e.to_string()))?;
        let job = self.store.submit_job(
            scope,
            NewJob {
                kind: ANALYSIS_JOB.into(),
                input,
                episode_id: None,
                client_request_id: req.client_request_id.clone(),
                // No effect outside its own artifacts: a lost lease re-runs it.
                idempotent: true,
                priority: JobPriority::Interactive,
                causation: Causation {
                    tool_call_id: Some(call_id.clone()),
                    ..Default::default()
                },
            },
        )?;
        (self.wake_jobs)();
        Ok(job)
    }

    fn sandbox(&self) -> ServiceResult<&Arc<Sandbox>> {
        self.services.sandbox.as_ref().ok_or_else(|| {
            envelope(
                ErrorCode::CapabilityUnavailable,
                "isolated runner not configured",
            )
        })
    }

    fn knowledge(&self) -> ServiceResult<&Arc<dyn KnowledgeService>> {
        self.services.knowledge.as_ref().ok_or_else(|| {
            envelope(
                ErrorCode::CapabilityUnavailable,
                "KnowledgeService not configured",
            )
        })
    }

    fn datasets(&self) -> ServiceResult<&Arc<dyn DatasetService>> {
        self.services.datasets.as_ref().ok_or_else(|| {
            envelope(
                ErrorCode::CapabilityUnavailable,
                "DatasetService not configured",
            )
        })
    }

    fn models(&self) -> ServiceResult<&Arc<dyn ModelService>> {
        self.services.models.as_ref().ok_or_else(|| {
            envelope(
                ErrorCode::CapabilityUnavailable,
                "ModelService not configured",
            )
        })
    }

    fn shape(
        &self,
        scope: &ExecutionScope,
        call_id: ToolCallId,
        tool: &str,
        outcome: ServiceResult<Outcome>,
    ) -> ToolResult {
        let base = |execution_status, error| ToolResult {
            schema_version: ToolsSchema::V1,
            call_id: call_id.clone(),
            tool: tool.to_string(),
            execution_status,
            scientific_status: ScientificStatus::NotAssessed,
            result: None,
            job: None,
            artifact_refs: vec![],
            error,
            origin: self.origin,
        };
        let failed = |e: ErrorEnvelope| {
            let status = if refusal(e.code) {
                ExecutionStatus::Refused
            } else if e.code == ErrorCode::Cancelled {
                ExecutionStatus::Cancelled
            } else {
                ExecutionStatus::Failed
            };
            base(status, Some(e))
        };
        let r = match outcome {
            Err(e) => failed(e),
            Ok(Outcome::Job(job)) => {
                // An idempotent resubmission may find the job already finished.
                let status = match job.status {
                    ignis_core::contracts::JobStatus::Succeeded => ExecutionStatus::Completed,
                    _ => ExecutionStatus::Accepted,
                };
                let mut r = base(status, None);
                r.artifact_refs = job.outputs.clone();
                r.job = Some(job);
                r
            }
            Ok(Outcome::Refused { error, artifact }) => {
                let mut r = failed(error);
                r.artifact_refs = vec![artifact];
                r
            }
            Ok(Outcome::Stored { value, artifact }) => {
                let small = serde_json::to_vec(&value)
                    .map(|b| b.len() <= INLINE_RESULT_BYTES)
                    .unwrap_or(false);
                let p = if small {
                    InlinePayload {
                        value,
                        partial: false,
                        full: None,
                    }
                } else {
                    InlinePayload {
                        value: json!({
                            "stored_as_artifact": artifact.artifact_id,
                            "how_to_read": "artifacts_read with this artifact_id and a json_pointer",
                        }),
                        partial: true,
                        full: Some(artifact.clone()),
                    }
                };
                let mut r = base(ExecutionStatus::Completed, None);
                r.artifact_refs = vec![artifact];
                r.result = Some(p);
                r
            }
            Ok(Outcome::Inline(p)) => {
                let mut r = base(ExecutionStatus::Completed, None);
                r.artifact_refs = p.full.iter().cloned().collect();
                r.result = Some(p);
                r
            }
            Ok(Outcome::Value(v)) => match self.inline_or_store(scope, &call_id, tool, v) {
                Ok(p) => {
                    let mut r = base(ExecutionStatus::Completed, None);
                    r.artifact_refs = p.full.iter().cloned().collect();
                    r.result = Some(p);
                    r
                }
                Err(e) => failed(e),
            },
        };
        match r.validate() {
            Ok(()) => r,
            Err(e) => failed(envelope(
                ErrorCode::Internal,
                format!("tool {tool} produced an invalid ToolResult: {e}"),
            )),
        }
    }

    /// Small values travel inline; large ones become an artifact with a preview that says how to
    /// read the rest.
    fn inline_or_store(
        &self,
        scope: &ExecutionScope,
        call_id: &ToolCallId,
        tool: &str,
        v: Value,
    ) -> ServiceResult<InlinePayload> {
        let bytes =
            serde_json::to_vec(&v).map_err(|e| envelope(ErrorCode::Internal, e.to_string()))?;
        if bytes.len() <= INLINE_RESULT_BYTES {
            return Ok(InlinePayload {
                value: v,
                partial: false,
                full: None,
            });
        }
        let contract = self
            .registered
            .iter()
            .find(|d| d.name == tool)
            .map(|d| d.output_contract.clone());
        let full = self.artifacts.put(
            scope,
            ArtifactDraft {
                media_type: "application/json".into(),
                contract,
                produced_by: Producer {
                    episode_id: None,
                    job_id: None,
                    tool_call_id: Some(call_id.clone()),
                    release_id: None,
                },
                inputs: Vec::<EntityRef>::new(),
                origin: self.origin,
            },
            &mut bytes.as_slice(),
        )?;
        let keys: Vec<&String> = v
            .as_object()
            .map(|o| o.keys().collect())
            .unwrap_or_default();
        Ok(InlinePayload {
            value: json!({
                "stored_as_artifact": full.artifact_id,
                "total_bytes": bytes.len(),
                "top_level_fields": keys,
                "how_to_read": "artifacts_read with this artifact_id and a json_pointer (e.g. \"/hits/0\"), or by byte range",
            }),
            partial: true,
            full: Some(full),
        })
    }
}

/// Every evidence reference inside a retrieval tool's result value ([`RETRIEVES_EVIDENCE`]),
/// checked against the contract.
fn collect_evidence(v: &Value, out: &mut Vec<EvidenceRef>) {
    match v {
        Value::Object(o) => {
            if o.contains_key("evidence_id") && o.contains_key("evidence_class") {
                if let Ok(e) = serde_json::from_value::<EvidenceRef>(v.clone()) {
                    if e.validate().is_ok() {
                        out.push(e);
                        return;
                    }
                }
            }
            o.values().for_each(|x| collect_evidence(x, out));
        }
        Value::Array(a) => a.iter().for_each(|x| collect_evidence(x, out)),
        _ => {}
    }
}

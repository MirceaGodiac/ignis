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
//! A tool whose service is not configured is not registered at all, and is reported in capability
//! status (see [`catalog`]); it never answers with a negative result.

use std::sync::Arc;

use ignis_core::contracts::{
    tool_catalog, AnalysisExecuteRequest, ArtifactReadRequest, Causation, ClientRequestId,
    DatasetRef, EntityRef, EpisodeId, ErrorCode, ErrorEnvelope, ExecutionScope, ExecutionStatus,
    ForecastRequest, InlinePayload, JobLookup, JobRef, Origin, Producer, ReadRequest, ReleaseRef,
    RetryPolicy, ScientificStatus, SearchRequest, ToolCallId, ToolDescriptor, ToolResult,
    ToolsSchema,
};
use ignis_core::services::{
    ArtifactDraft, ArtifactStore, DatasetService, KnowledgeService, ModelService, ServiceResult,
};
use ignis_research::sandbox::Sandbox;
use ignis_research::store::{JobPriority, NewJob};
use ignis_research::ResearchStore;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

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
}

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
                "sources_search" | "sources_read" if services.knowledge.is_none() => {
                    Some("KnowledgeService not configured")
                }
                "datasets_inspect" if services.datasets.is_none() => {
                    Some("DatasetService not configured")
                }
                "models_inspect" | "models_forecast" if services.models.is_none() => {
                    Some("ModelService not configured")
                }
                "analysis_execute" if services.sandbox.is_none() => {
                    Some("isolated runner not configured or failed preflight")
                }
                "wiki_patch" => Some("wiki service not implemented"),
                "sources_search" | "sources_read" | "datasets_inspect" | "models_inspect"
                | "models_forecast" | "analysis_execute" | "artifacts_read" | "jobs_get" => None,
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
            | ErrorCode::Unsupported
            | ErrorCode::CapabilityUnavailable
            | ErrorCode::Conflict
    )
}

fn decode<T: DeserializeOwned>(contract: &str, args: Value) -> ServiceResult<T> {
    serde_json::from_value(args).map_err(|e| {
        envelope(
            ErrorCode::InvalidRequest,
            format!("arguments are not a valid {contract}: {e}"),
        )
    })
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
            registered,
        }
    }

    /// The tools this host serves.
    pub fn registered(&self) -> &[ToolDescriptor] {
        &self.registered
    }

    /// Run one call with the host-derived `scope`. Never panics on input; always returns a valid
    /// [`ToolResult`].
    pub fn call(&self, scope: &ExecutionScope, tool: &str, args: Value) -> ToolResult {
        let call_id = new_call_id();
        let outcome = match self.registered.iter().find(|d| d.name == tool) {
            None => Err(envelope(
                ErrorCode::NotFound,
                format!("no tool {tool:?} is registered in this deployment"),
            )),
            Some(d) => d
                .capabilities
                .iter()
                .try_for_each(|c| scope.require(*c).map_err(ErrorEnvelope::from))
                .and_then(|()| self.dispatch(scope, d, &call_id, args)),
        };
        self.shape(scope, call_id, tool, outcome)
    }

    fn dispatch(
        &self,
        scope: &ExecutionScope,
        d: &ToolDescriptor,
        call_id: &ToolCallId,
        args: Value,
    ) -> ServiceResult<Outcome> {
        let contract = d.input_contract.as_str();
        match d.name.as_str() {
            "sources_search" => {
                let req: SearchRequest = decode(contract, args)?;
                checked(req.validate())?;
                value(self.knowledge()?.search(scope, &req))
            }
            "sources_read" => {
                let req: ReadRequest = decode(contract, args)?;
                checked(req.validate())?;
                value(self.knowledge()?.read(scope, &req))
            }
            "datasets_inspect" => {
                let req: DatasetRef = decode(contract, args)?;
                value(self.datasets()?.dataset(scope, &req.dataset_id))
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

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
use std::time::{Duration, Instant};

use ignis_core::contracts::{
    tool_catalog, AnalysisExecuteRequest, AnswerSubmission, ArtifactReadRequest, ArtifactRef,
    Author, AuthorKind, Causation, ClientRequestId, ConditionsQuery, DatasetRef, EntityRef,
    EpisodeId, ErrorCode, ErrorEnvelope, EvidenceRef, ExecutionRecord, ExecutionScope,
    ExecutionStatus, ForecastRequest, GraphNeighborsRequest, GroundingMode, InlinePayload,
    JobLookup, JobRef, Origin, OutlineRequest, Producer, ReadRequest, ReleaseRef, ResponseFormat,
    RetryPolicy, ScientificStatus, SearchRequest, ToolCallId, ToolDescriptor, ToolResult,
    ToolsSchema, WebFetchRequest, WebSearchRequest, WikiPatch, WikiReadRequest,
};
use ignis_core::contracts::{SkillRef, SkillText};
use ignis_core::services::{
    ArtifactDraft, ArtifactStore, DatasetService, KnowledgeService, ModelService, ServiceResult,
};
use ignis_research::runtime::HarnessShelf;
use ignis_research::sandbox::Sandbox;
use ignis_research::store::{JobPriority, NewJob};
use ignis_research::ResearchStore;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::answer::{AnswerHost, Submitted};

mod conditions;

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

/// How long a job tool waits by default for its job to end before answering with the job's
/// current state. Every agent poll is a full model request that re-sends the conversation, so the
/// host waits instead; bounded below the runtime's MCP call timeout (Codex: 60 s).
pub const JOB_WAIT: Duration = Duration::from_secs(25);

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
    /// Web discovery (`web_search`, `web_fetch`) the KnowledgeService serves. Off by default:
    /// nothing networked runs unless the deployment opts in.
    pub web: WebTools,
    /// The KnowledgeService has a wiki store attached (`wiki_read`, `wiki_patch`). Without it
    /// the wiki tools are not registered and capability status says why.
    pub wiki: bool,
    /// A session of this deployment can hold `wiki_write`. Research sessions never do (they read
    /// the wiki only), so by default `wiki_patch` is not registered: an agent would carry its
    /// schema in every request for a tool every call of which is refused.
    pub wiki_write: bool,
}

/// Which web discovery tools the deployment enabled (`IGNIS_ENABLE_WEB`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WebTools {
    #[default]
    Disabled,
    /// `web_fetch` only: no search provider is configured.
    FetchOnly,
    SearchAndFetch,
}

/// Why the web tools are absent when the deployment did not enable them.
pub const WEB_DISABLED: &str =
    "web discovery disabled in this deployment (set IGNIS_ENABLE_WEB=1; network access)";

/// Why `web_search` is absent when web discovery is on without a provider.
pub const WEB_SEARCH_NO_PROVIDER: &str =
    "no web search provider configured (set PARALLEL_API_KEY with IGNIS_ENABLE_WEB=1)";

/// Why the wiki tools are absent while knowledge is configured.
pub const WIKI_NOT_CONFIGURED: &str = "wiki store not attached to the KnowledgeService";

/// Why `wiki_patch` is absent while the wiki is attached.
pub const WIKI_WRITE_NOT_GRANTED: &str =
    "no session of this deployment is granted wiki_write (research sessions read the wiki only)";

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
                "sources_search" | "sources_read" | "sources_outline" | "graph_neighbors"
                    if services.knowledge.is_none() =>
                {
                    Some("KnowledgeService not configured")
                }
                "datasets_inspect" | "conditions_query" if services.datasets.is_none() => {
                    Some("DatasetService not configured")
                }
                "conditions_query" if !services.conditions => Some(CONDITIONS_NOT_BUILT),
                "wiki_read" | "wiki_patch" if services.knowledge.is_none() => {
                    Some("KnowledgeService not configured")
                }
                "wiki_read" | "wiki_patch" if !services.wiki => Some(WIKI_NOT_CONFIGURED),
                "wiki_patch" if !services.wiki_write => Some(WIKI_WRITE_NOT_GRANTED),
                "models_inspect" | "models_forecast" if services.models_disabled => {
                    Some(MODELS_DISABLED)
                }
                "models_inspect" | "models_forecast" if services.models.is_none() => {
                    Some("ModelService not configured")
                }
                "analysis_execute" if services.sandbox.is_none() => {
                    Some("isolated runner not configured or failed preflight")
                }
                "web_search" | "web_fetch" if services.knowledge.is_none() => {
                    Some("KnowledgeService not configured")
                }
                "web_search" | "web_fetch" if services.web == WebTools::Disabled => {
                    Some(WEB_DISABLED)
                }
                "web_search" if services.web == WebTools::FetchOnly => Some(WEB_SEARCH_NO_PROVIDER),
                "web_search" | "web_fetch" => None,
                "sources_search" | "sources_read" | "sources_outline" | "graph_neighbors"
                | "datasets_inspect" | "conditions_query" | "models_inspect"
                | "models_forecast" | "analysis_execute" | "artifacts_read" | "jobs_get"
                | "answer_submit" | "wiki_read" | "wiki_patch" => None,
                // The host always holds the harness bundle of its runtime (see `harness`).
                "skills_read" => None,
                // KnowledgeService methods are not implemented on main yet (trait defaults).
                _ => Some("no adapter for this tool"),
            };
            let mut d = d;
            // The served releases and their inputs come from the model service (its served list).
            if matches!(d.name.as_str(), "models_inspect" | "models_forecast")
                && unavailable.is_none()
            {
                if let Some(note) = services.models.as_ref().and_then(|m| m.tool_note()) {
                    d.summary = format!("{} {note}", d.summary);
                }
            }
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
    /// How long a job tool waits server-side for its job to end ([`JOB_WAIT`] by default).
    job_wait: Duration,
    registered: Vec<ToolDescriptor>,
    /// The harness bundles this host's runtimes were started with: what `skills_read` serves.
    harness: HarnessShelf,
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
    /// A compact view for the agent (`shown`) of the service's answer (`full`): the evidence is
    /// recorded from `full`, and `full` is what is stored when `shown` is too large to inline.
    Concise {
        shown: Value,
        full: Value,
    },
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
            job_wait: JOB_WAIT,
            registered,
            harness: HarnessShelf::default(),
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

    /// How long `jobs_get`, `models_forecast` and `analysis_execute` wait for their job to end
    /// before answering with its current state (`Duration::ZERO`: answer at once).
    pub fn with_job_wait(mut self, wait: Duration) -> Self {
        self.job_wait = wait;
        self
    }

    /// The tools this host serves.
    pub fn registered(&self) -> &[ToolDescriptor] {
        &self.registered
    }

    /// Where the composition root puts the harness bundle it renders for the runtime, so
    /// `skills_read` serves the skills of the bundle an episode is pinned to.
    pub fn harness(&self) -> &HarnessShelf {
        &self.harness
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
            Outcome::Value(v) | Outcome::Concise { full: v, .. } => v,
            Outcome::Inline(p) => &p.value,
            _ => return,
        };
        let mut found = Vec::new();
        collect_evidence(v, &mut found);
        if found.is_empty() {
            return;
        }
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
                let page = self.knowledge()?.search(scope, &req)?;
                if !concise(req.response_format) {
                    return value(Ok(page));
                }
                let shown = concise_page(&page);
                let full = serde_json::to_value(&page)
                    .map_err(|e| envelope(ErrorCode::Internal, e.to_string()))?;
                Ok(Outcome::Concise { shown, full })
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
            "web_search" => {
                let req: WebSearchRequest = decode(contract, args)?;
                checked(req.validate())?;
                value(self.knowledge()?.web_search(scope, &req))
            }
            "web_fetch" => {
                let req: WebFetchRequest = decode(contract, args)?;
                checked(req.validate())?;
                value(self.knowledge()?.web_fetch(scope, &req))
            }
            "wiki_read" => {
                let req: WikiReadRequest = decode(contract, args)?;
                checked(req.validate())?;
                value(self.knowledge()?.wiki_read(scope, &req))
            }
            "wiki_patch" => {
                let patch: WikiPatch = decode(contract, args)?;
                checked(patch.validate())?;
                // The author is the scope's principal acting through the agent; the episode is the
                // runtime grant's. Neither is ever taken from the arguments.
                let author = Author {
                    kind: AuthorKind::Agent,
                    id: scope.principal.as_str().to_string(),
                };
                value(
                    self.knowledge()?
                        .wiki_patch(scope, &author, episode, &patch),
                )
            }
            "graph_neighbors" => {
                let req: GraphNeighborsRequest = decode(contract, args)?;
                checked(req.validate())?;
                value(self.knowledge()?.graph_neighbors(scope, &req))
            }
            "datasets_inspect" => {
                let req: DatasetRef = decode(contract, args)?;
                value(self.datasets()?.dataset(scope, &req.dataset_id))
            }
            "conditions_query" => {
                let q: ConditionsQuery = decode(contract, args)?;
                checked(q.validate())?;
                self.conditions_result(scope, episode, call_id, &q)
                    .map(Outcome::Inline)
            }
            "models_inspect" => {
                let req: ReleaseRef = decode(contract, args)?;
                value(self.models()?.release(scope, &req.release_id))
            }
            "models_forecast" => {
                let req: ForecastRequest = decode(contract, args)?;
                self.models()?;
                let job = self.submit_forecast(scope, &req, None, None, call_id)?;
                Ok(Outcome::Job(self.settle(scope, job)?))
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
                Ok(Outcome::Job(self.settle(scope, job)?))
            }
            "artifacts_read" => {
                let req: ArtifactReadRequest = decode(contract, args)?;
                ignis_research::artifacts::read_for_tool(self.artifacts.as_ref(), scope, &req)
                    .map(Outcome::Inline)
            }
            "jobs_get" => {
                let req: JobLookup = decode(contract, args)?;
                let job = self.store.job(scope, &req.job_id)?;
                value(self.settle(scope, job))
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
                    knowledge: self.services.knowledge.as_deref(),
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
                        // An acknowledgement, not the answer: the agent wrote it, the artifact
                        // keeps it whole, and readers get it from `answer.completed`.
                        let mut value = json!({
                            "answer_id": answer.answer_id,
                            "status": answer.status,
                            "stored_as_artifact": artifact.artifact_id,
                            "claims": answer.claims.len(),
                            "evidence": answer.evidence.len(),
                            "artifacts": answer.artifacts.len(),
                        });
                        if let Some(g) = &answer.grounding {
                            value["grounding"] = serde_json::to_value(g)
                                .map_err(|e| envelope(ErrorCode::Internal, e.to_string()))?;
                        }
                        Ok(Outcome::Stored { value, artifact })
                    }
                    Submitted::Refused { error, refusal } => Ok(Outcome::Refused {
                        error,
                        artifact: refusal,
                    }),
                }
            }
            "skills_read" => {
                let req: SkillRef = decode(contract, args)?;
                value(self.skill(scope, episode, &req.name))
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

    /// `skills_read`: the skill from the bundle the episode is pinned to (never another bundle: the
    /// pin is what the trace reproduces), or, outside an episode, from the host's current bundle.
    fn skill(
        &self,
        scope: &ExecutionScope,
        episode: Option<&EpisodeId>,
        name: &str,
    ) -> ServiceResult<SkillText> {
        let bundle = match episode {
            Some(ep) => {
                let pin = self.store.episode(scope, ep)?.pins.harness.ok_or_else(|| {
                    envelope(
                        ErrorCode::CapabilityUnavailable,
                        "this episode is not pinned to a harness bundle yet, so it has no skills to read",
                    )
                })?;
                self.harness.get(&pin.digest).ok_or_else(|| {
                    envelope(
                        ErrorCode::CapabilityUnavailable,
                        format!(
                            "this episode is pinned to harness {} sha256:{}, which this host does not hold; its skills cannot be served here",
                            pin.label,
                            pin.digest.hex()
                        ),
                    )
                })?
            }
            None => self.harness.current().ok_or_else(|| {
                envelope(
                    ErrorCode::CapabilityUnavailable,
                    "no harness bundle is loaded in this host",
                )
            })?,
        };
        bundle.skill_text(name).ok_or_else(|| {
            let names: Vec<&str> = bundle.skills().iter().map(|s| s.name.as_str()).collect();
            envelope(
                ErrorCode::NotFound,
                format!(
                    "no skill {name:?} in this episode's harness; its skills are: {}",
                    if names.is_empty() {
                        "none".to_string()
                    } else {
                        names.join(", ")
                    }
                ),
            )
        })
    }

    /// Wait (bounded by `job_wait`) for `job` to reach a terminal state and return its latest
    /// state either way, so the agent does not spend a model request per poll. Blocks the calling
    /// thread: transports run tool calls on the blocking pool (see [`crate::http`]), never on a
    /// reactor thread. Reads the store, so it also sees a runner in another process.
    fn settle(&self, scope: &ExecutionScope, job: JobRef) -> ServiceResult<JobRef> {
        if job.status.is_terminal() || self.job_wait.is_zero() {
            return Ok(job);
        }
        let deadline = Instant::now() + self.job_wait;
        let mut pause = Duration::from_millis(50);
        let mut job = job;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Ok(job);
            }
            std::thread::sleep(pause.min(left));
            pause = (pause * 2).min(Duration::from_millis(500));
            job = self.store.job(scope, &job.job_id)?;
            if job.status.is_terminal() {
                return Ok(job);
            }
        }
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
                // The job may have ended while the host waited (or before an idempotent
                // resubmission): say how, with the job's own error.
                // The job carries its outputs; they are not repeated in `artifact_refs`.
                use ignis_core::contracts::JobStatus;
                let mut r = match (job.status, &job.error) {
                    (JobStatus::Succeeded, _) => {
                        let mut r = base(ExecutionStatus::Completed, None);
                        if job.kind == ANALYSIS_JOB {
                            r.result = Some(self.analysis_summary(scope, &job));
                        }
                        r
                    }
                    (JobStatus::Failed | JobStatus::Cancelled, Some(e)) => failed(e.clone()),
                    (JobStatus::Failed, None) => failed(envelope(
                        ErrorCode::Internal,
                        format!("job {} failed without recording why", job.job_id),
                    )),
                    (JobStatus::Cancelled, None) => failed(envelope(
                        ErrorCode::Cancelled,
                        format!("job {} was cancelled before it finished", job.job_id),
                    )),
                    _ => base(ExecutionStatus::Accepted, None),
                };
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
                // A read of a stored artifact (already its own reference): one that is still
                // too large to inline answers a preview of itself, pointing at the same artifact.
                let p = match (size_of(&p.value) > INLINE_RESULT_BYTES, p.full) {
                    (true, Some(full)) => {
                        let (head, cut) = preview_of(&p.value, PREVIEW_BYTES);
                        InlinePayload {
                            value: json!({
                                "artifact_id": full.artifact_id,
                                "total_bytes": size_of(&p.value),
                                "preview": head,
                                "cut": cut,
                                "how_to_read": HOW_TO_READ_LARGE_READ,
                            }),
                            partial: true,
                            full: Some(full),
                        }
                    }
                    (_, full) => InlinePayload { full, ..p },
                };
                let mut r = base(ExecutionStatus::Completed, None);
                r.artifact_refs = p.full.iter().cloned().collect();
                r.result = Some(p);
                r
            }
            Ok(Outcome::Concise { shown, full }) => {
                let p = if size_of(&shown) <= INLINE_RESULT_BYTES {
                    Ok(InlinePayload {
                        value: shown,
                        partial: false,
                        full: None,
                    })
                } else {
                    // The agent previews the view it reads; the artifact keeps the full answer.
                    serde_json::to_vec(&full)
                        .map_err(|e| envelope(ErrorCode::Internal, e.to_string()))
                        .and_then(|bytes| {
                            self.store_previewed(scope, &call_id, tool, bytes, &shown)
                        })
                };
                match p {
                    Ok(p) => {
                        let mut r = base(ExecutionStatus::Completed, None);
                        r.artifact_refs = p.full.iter().cloned().collect();
                        r.result = Some(p);
                        r
                    }
                    Err(e) => failed(e),
                }
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

    /// What a succeeded `analysis_execute` job answers: its [`ExecutionRecord`] reference, how the
    /// program ended, each kept output by path, and the `derivation` a computed claim binds
    /// (`computation` = the record, `inputs` = exactly the mounted inputs), so the agent needs no
    /// extra `artifacts_read` of the record to cite a number.
    fn analysis_summary(&self, scope: &ExecutionScope, job: &JobRef) -> InlinePayload {
        let id = |a: &ArtifactRef| json!(a.artifact_id);
        let found = job
            .outputs
            .iter()
            .find(|a| a.contract.as_deref() == Some("ExecutionRecord"));
        let record = found
            .ok_or_else(|| "the job stored no ExecutionRecord".to_string())
            .and_then(|meta| {
                ignis_research::artifacts::read_json_pointer(
                    self.artifacts.as_ref(),
                    scope,
                    meta,
                    "",
                )
                .map_err(|e| e.message)
                .and_then(|p| {
                    serde_json::from_value::<ExecutionRecord>(p.value).map_err(|e| e.to_string())
                })
                .map(|r| (meta, r))
            });
        let (meta, rec) = match record {
            Ok(x) => x,
            Err(why) => {
                return InlinePayload {
                    value: json!({
                        "outputs": job.outputs.iter().map(id).collect::<Vec<_>>(),
                        "note": format!("the run's ExecutionRecord could not be read ({why}); read job.outputs with artifacts_read"),
                    }),
                    partial: false,
                    full: None,
                }
            }
        };
        let stream = |s: &ignis_core::contracts::CapturedStream| {
            s.artifact.as_ref().map(|a| {
                json!({"artifact_id": a.artifact_id, "total_bytes": s.total_bytes, "truncated": s.truncated})
            })
        };
        let mut value = json!({
            "execution_record": meta.artifact_id,
            "outcome": rec.outcome,
            "outputs": rec.outputs.iter().map(|o| json!({
                "path": o.path,
                "artifact_id": o.artifact.artifact_id,
                "sha256": o.artifact.sha256,
            })).collect::<Vec<_>>(),
            "stdout": stream(&rec.stdout),
            "stderr": stream(&rec.stderr),
            "derivation": {
                "computation": meta.artifact_id,
                "inputs": rec.inputs.iter().map(|m| id(&m.artifact)).collect::<Vec<_>>(),
            },
        });
        if !rec.rejected_outputs.is_empty() {
            value["rejected_outputs"] = json!(rec.rejected_outputs);
        }
        if size_of(&value) <= INLINE_RESULT_BYTES {
            return InlinePayload {
                value,
                partial: false,
                full: None,
            };
        }
        let (head, cut) = preview_of(&value, PREVIEW_BYTES);
        InlinePayload {
            value: json!({
                "artifact_id": meta.artifact_id,
                "preview": head,
                "cut": cut,
                "how_to_read": "the run's summary is too large to inline; the ExecutionRecord artifact holds it whole: read its /outputs, /inputs or /rejected_outputs with artifacts_read and item_offset",
            }),
            partial: true,
            full: Some(meta.clone()),
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
        self.store_previewed(scope, call_id, tool, bytes, &v)
    }

    /// Store `full` (its serialized `bytes`) as an artifact and answer a preview of `shown`, the
    /// view the agent reads (the full value itself, or its concise view), paired with that artifact.
    fn store_previewed(
        &self,
        scope: &ExecutionScope,
        call_id: &ToolCallId,
        tool: &str,
        bytes: Vec<u8>,
        shown: &Value,
    ) -> ServiceResult<InlinePayload> {
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
        let (head, cut) = preview_of(shown, PREVIEW_BYTES);
        Ok(InlinePayload {
            value: json!({
                "stored_as_artifact": full.artifact_id,
                "total_bytes": bytes.len(),
                "preview": head,
                "cut": cut,
                "how_to_read": HOW_TO_READ_STORED,
            }),
            partial: true,
            full: Some(full),
        })
    }
}

const HOW_TO_READ_STORED: &str = "the preview holds every small field whole and the first items of each long list (`cut` says how many of how many); the stored artifact is the full (detailed) result: read the rest with artifacts_read, this artifact_id and a json_pointer to an item (\"/rows/40\") or to a list with item_offset (\"/rows\", item_offset 40), or narrow the call (filters, a page_size) and call again";

const HOW_TO_READ_LARGE_READ: &str = "this read is larger than one tool result carries inline; the preview holds every small field whole and the first items of each long list (`cut` says how many of how many); read a narrower part of the same artifact: a json_pointer to a list with item_offset/item_count, a deeper json_pointer, or a byte_length of at most 24576";

/// How much of an oversized result travels inline as its preview.
pub const PREVIEW_BYTES: usize = 24 << 10;

fn size_of(v: &Value) -> usize {
    serde_json::to_vec(v).map_or(usize::MAX, |b| b.len())
}

/// The head of a large result: every top-level field that is small travels whole; each long
/// array keeps its first items while the budget lasts; an object too large to keep is listed by
/// its keys. `cut` names each shortened field with how many items were kept of how many, so the
/// model knows what it has not seen and where to read it.
fn preview_of(v: &Value, budget: usize) -> (Value, Value) {
    if let Value::Array(items) = v {
        let kept = head_items(items, budget);
        let cut = json!({"kept": kept.len(), "of": items.len()});
        return (Value::Array(kept), cut);
    }
    let Some(o) = v.as_object() else {
        return (
            Value::Null,
            json!({"whole": "not an object; read it by byte range"}),
        );
    };
    let mut head = serde_json::Map::new();
    let mut cut = serde_json::Map::new();
    let mut used = 0usize;
    let mut big: Vec<(&String, &Value)> = Vec::new();
    for (k, x) in o {
        let n = size_of(x);
        if n <= 2048 {
            used += n + k.len() + 4;
            head.insert(k.clone(), x.clone());
        } else {
            big.push((k, x));
        }
    }
    // Smaller fields first, so one huge list does not crowd out the rest.
    big.sort_by_key(|(_, x)| size_of(x));
    let share = budget.saturating_sub(used) / big.len().max(1);
    for (k, x) in big {
        match x {
            Value::Array(items) => {
                let kept = head_items(items, share);
                cut.insert(k.clone(), json!({"kept": kept.len(), "of": items.len()}));
                head.insert(k.clone(), Value::Array(kept));
            }
            Value::Object(m) if size_of(x) > share => {
                cut.insert(k.clone(), json!({"keys": m.keys().collect::<Vec<_>>()}));
            }
            _ if size_of(x) > share => {
                cut.insert(k.clone(), json!({"bytes": size_of(x)}));
            }
            _ => {
                head.insert(k.clone(), x.clone());
            }
        }
    }
    (Value::Object(head), Value::Object(cut))
}

/// The first `items` that fit `budget` serialized bytes.
fn head_items(items: &[Value], budget: usize) -> Vec<Value> {
    let mut kept = Vec::new();
    let mut n = 2usize;
    for it in items {
        let m = size_of(it).saturating_add(1);
        if n.saturating_add(m) > budget {
            break;
        }
        n += m;
        kept.push(it.clone());
    }
    kept
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

/// The `concise` view of a search page an agent reads: per hit what it needs to choose, read and
/// cite a passage (rank, evidence id and class, source id, page, snippet), each source's title
/// once, and only the backends that changed the search (degraded, or used with a note). Locator
/// internals (asset digest, offsets, bbox, parse view), per-hit origin equal to the page's, and
/// backends that were not part of the search are left out; `detailed` returns the `SearchPage`.
/// The host records the full `EvidenceRef` of every hit, so citations resolve exactly as before.
fn concise_page(page: &ignis_core::contracts::SearchPage) -> Value {
    use ignis_core::contracts::BackendStatus;
    use serde_json::Map;
    let mut sources = Map::new();
    let hits: Vec<Value> = page
        .hits
        .iter()
        .map(|h| {
            let e = &h.evidence;
            let mut o = Map::new();
            o.insert("rank".into(), json!(h.rank));
            o.insert("evidence_id".into(), json!(e.evidence_id));
            o.insert("evidence_class".into(), json!(e.evidence_class));
            o.insert("source_id".into(), json!(e.source_id));
            if let Some(l) = &e.locator {
                if let Some(p) = l.page {
                    o.insert("page".into(), json!(p));
                }
                if let Some(p) = &l.page_label {
                    o.insert("page_label".into(), json!(p));
                }
                if let Some(t) = &l.table_id {
                    o.insert("table_id".into(), json!(t));
                }
            }
            o.insert("snippet".into(), json!(h.snippet));
            if let Some(q) = &e.quote {
                o.insert("quote".into(), json!(q));
            }
            if e.origin != page.origin {
                o.insert("origin".into(), json!(e.origin));
            }
            if let Some(t) = &h.title {
                sources
                    .entry(e.source_id.as_str().to_string())
                    .or_insert_with(|| json!(t));
            }
            Value::Object(o)
        })
        .collect();
    let backends: Vec<_> = page
        .backends
        .iter()
        .filter(|b| match b.status {
            BackendStatus::Unavailable => true,
            BackendStatus::Used => b.detail.is_some(),
            BackendStatus::NotRequested => false,
        })
        .collect();
    let mut out = Map::new();
    out.insert("hits".into(), Value::Array(hits));
    out.insert("sources".into(), Value::Object(sources));
    if let Some(c) = &page.next_cursor {
        out.insert("next_cursor".into(), json!(c));
    }
    if let Some(n) = page.total_count {
        out.insert("total_count".into(), json!(n));
    }
    if !backends.is_empty() {
        out.insert("backends".into(), json!(backends));
    }
    out.insert("origin".into(), json!(page.origin));
    Value::Object(out)
}

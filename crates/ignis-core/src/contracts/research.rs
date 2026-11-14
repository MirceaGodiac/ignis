//! Research runtime contracts: episodes, ordered events, jobs, tool results, answers and the tool
//! catalog. Semantic owner: Mircea (Tudor reviews for the UI).
//!
//! Codex App Server owns the conversation and the model/tool loop. IGNIS keeps only what it needs to
//! map, replay and audit an episode: the thread mapping, its own ordered event log, tool/job receipts
//! and artifacts. Codex's native thread history is not copied into a second authoritative format.
//!
//! Wave K1 (`research.v2`) adds what the live harness and the workspace need: tool arguments and the
//! model's own step rationale in the log, the answer the agent submits for host validation, the
//! closed catalog of answer views, support checks on claims, the harness an episode ran with, the
//! per-episode trace, and the `/api/v1` request/response shapes that used to live only in the server.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de, Deserialize, Deserializer, Serialize};

use super::common::{
    check_id, ensure, opaque_id, schema_tag, AnswerId, ApprovalId, ArtifactId, ArtifactRef,
    Capability, ClaimId, ClientRequestId, ContractError, CorpusSnapshotId, Digest, EpisodeId,
    ErrorEnvelope, EventId, EvidenceId, Finite, InlinePayload, JobId, MessageId, Origin, ReleaseId,
    RuntimeBuildId, ScopeId, SourceId, Timestamp, ToolCallId, WorkspaceId,
};
use super::{EvidenceClass, EvidenceRef, Partition};

/// Wire tag of episodes, events, answers and traces. Wave K1 writes `research.v2`; `research.v1`
/// (C0) documents are still read. A v2 addition (tool arguments, step rationale, a view block, a
/// support check, a harness pin) is refused under the v1 tag, so a v1 reader never receives a
/// document it would misread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub enum ResearchSchema {
    /// C0. Still read; new documents are written as v2.
    #[serde(rename = "research.v1")]
    V1,
    #[default]
    #[serde(rename = "research.v2")]
    V2,
}

impl ResearchSchema {
    fn require_v2(self, contract: &'static str, what: &str) -> Result<(), ContractError> {
        ensure(self == ResearchSchema::V2, contract, || {
            format!("{what} needs schema_version research.v2")
        })
    }
}

opaque_id! {
    /// One episode trace (`TraceBundle`).
    TraceId;
}

schema_tag!(
    /// Wire tag of tool results and descriptors.
    ToolsSchema = "tools.v1"
);

// ---------------------------------------------------------------- episodes

/// Operational state of an episode. It constrains actions, not reasoning: inside `running` the agent
/// decides whether to read, compute, ask or answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeStatus {
    Ready,
    Running,
    WaitingTools,
    WaitingApproval,
    WaitingUser,
    Paused,
    /// The last turn finished; the episode can be continued by a new message.
    Completed,
    Cancelled,
    Failed,
}

/// Agent runtime hosting an episode (ADR 17): the pinned Codex App Server, or Claude Code in
/// headless mode. Each owns its own model/tool loop; IGNIS maps, stores and audits both the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    CodexAppServer,
    ClaudeCode,
}

/// Mapping of an episode to its runtime thread. Turn/item ids travel in `Causation`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuntimeThreadRef {
    pub runtime: RuntimeKind,
    pub runtime_build_id: RuntimeBuildId,
    pub thread_id: String,
}

/// Where a branched episode left its parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EpisodeBranchPoint {
    pub episode_id: EpisodeId,
    /// Last parent event visible to the child.
    pub at_seq: u64,
}

/// The harness an episode runs with (K1.6): the instruction/skill bundle by digest plus the runtime
/// build, model and effort it was injected into. Every benchmark number is reported with this.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HarnessRef {
    /// Digest of the harness bundle (`harness/`: instructions, skills, agents) as injected.
    pub digest: Digest,
    /// Human label of the bundle or variant (`h1`, `h1-no-wiki`); not an identity.
    pub label: String,
    pub runtime_build: RuntimeBuildId,
    /// Model id as the runtime reports it.
    pub model: String,
    /// Reasoning effort as the runtime names it, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
}

impl HarnessRef {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "HarnessRef";
        ensure(!self.label.trim().is_empty(), c, || "empty label".into())?;
        ensure(!self.model.trim().is_empty(), c, || "empty model".into())?;
        ensure(
            self.effort.as_ref().is_none_or(|e| !e.trim().is_empty()),
            c,
            || "an effort, when present, is named".into(),
        )
    }
}

/// Versions an episode is pinned to; switching is an explicit, visible event.
///
/// v2 replaces C0's `skills_version` (never written: it was always absent) with `harness`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EpisodePins {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus_snapshot: Option<CorpusSnapshotId>,
    #[serde(default)]
    pub model_releases: Vec<ReleaseId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<HarnessRef>,
}

/// One scientific investigation. Holds no provider keys and no copy of the runtime transcript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchEpisode {
    pub schema_version: ResearchSchema,
    pub episode_id: EpisodeId,
    pub workspace_id: WorkspaceId,
    pub goal: String,
    pub status: EpisodeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeThreadRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<EpisodeBranchPoint>,
    pub pins: EpisodePins,
    pub scope_id: ScopeId,
    /// Optimistic-concurrency revision of this record.
    pub revision: u64,
    /// Highest committed event `seq` (0 before the first event).
    pub last_seq: u64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub origin: Origin,
}

impl ResearchEpisode {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ResearchEpisode";
        if let Some(h) = &self.pins.harness {
            self.schema_version.require_v2(c, "a harness pin")?;
            h.validate()?;
        }
        ensure(
            self.created_at.as_str() <= self.updated_at.as_str(),
            c,
            || "updated before it was created".into(),
        )
    }
}

// ---------------------------------------------------------------- events

/// Why an event happened. Ids from the runtime are kept for correlation, but a scientific job is
/// never keyed only by a Codex id.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Causation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<ClientRequestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_turn_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<ToolCallId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<JobId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caused_by_seq: Option<u64>,
}

impl Causation {
    fn is_empty(&self) -> bool {
        *self == Causation::default()
    }
}

/// The arguments of a tool call as shown in the log (K1.1). The host redacts what the viewer's
/// scope may not see before the event is written; the pointers say what was removed, so a missing
/// field is never mistaken for an argument the agent did not send. Large arguments (a program's
/// source) are a partial preview pointing at the full artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolArguments {
    pub value: InlinePayload,
    /// RFC 6901 JSON Pointers of the members removed by redaction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redacted: Vec<String>,
}

impl ToolArguments {
    pub fn validate(&self) -> Result<(), ContractError> {
        self.value.validate()?;
        for p in &self.redacted {
            ensure(is_json_pointer(p) && !p.is_empty(), "ToolArguments", || {
                format!("{p:?} is not a JSON Pointer to a member")
            })?;
        }
        Ok(())
    }
}

/// Where a step rationale comes from. Closed on purpose: only the model's own explanation, written
/// for the user, is shown; hidden chain-of-thought is never recorded as a rationale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RationaleSource {
    ModelExplanation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Approved,
    Denied,
    Expired,
    Cancelled,
}

/// Event payloads. Large content is referenced through artifacts, never inlined.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "payload", deny_unknown_fields)]
pub enum EventBody {
    #[serde(rename = "episode.status_changed")]
    StatusChanged {
        from: EpisodeStatus,
        to: EpisodeStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    #[serde(rename = "message.user")]
    UserMessage { message_id: MessageId, text: String },
    /// Incremental assistant text. Never the only copy: `message.completed` carries the snapshot.
    #[serde(rename = "message.delta")]
    AssistantDelta { message_id: MessageId, text: String },
    #[serde(rename = "message.completed")]
    AssistantMessage { message_id: MessageId, text: String },
    #[serde(rename = "tool.started")]
    ToolStarted {
        call_id: ToolCallId,
        tool: String,
        /// v2: the call's arguments, redacted by scope (absent in v1 logs).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arguments: Option<ToolArguments>,
    },
    /// v2: the model's own short explanation of a step (why this search, why this computation),
    /// tied to the tool call it introduces when there is one.
    #[serde(rename = "step.rationale")]
    StepRationale {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        call_id: Option<ToolCallId>,
        text: String,
        source: RationaleSource,
    },
    #[serde(rename = "tool.finished")]
    ToolFinished { result: ToolResult },
    #[serde(rename = "job.updated")]
    JobUpdated { job: JobRef },
    #[serde(rename = "artifact.created")]
    ArtifactCreated { artifact: ArtifactRef },
    /// A runtime-initiated approval request; resolved explicitly by the host, never by the UI alone.
    #[serde(rename = "approval.requested")]
    ApprovalRequested {
        approval_id: ApprovalId,
        action: String,
        effect: ToolEffect,
    },
    #[serde(rename = "approval.resolved")]
    ApprovalResolved {
        approval_id: ApprovalId,
        decision: ApprovalDecision,
    },
    /// The final answer of a turn, stored as a `ResearchAnswer` artifact.
    #[serde(rename = "answer.completed")]
    AnswerCompleted {
        answer_id: AnswerId,
        status: AnswerStatus,
        answer: ArtifactRef,
    },
    /// An upstream runtime notification IGNIS does not interpret, archived for diagnostics. The UI
    /// may show it as opaque; nobody derives domain meaning from it.
    #[serde(rename = "runtime.opaque")]
    RuntimeOpaque {
        method: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<ArtifactRef>,
    },
}

/// One entry of an episode's append-only log. `seq` is gap-free and monotonic per episode and is
/// the replay authority: SSE sends it as the event `id`, clients resume with `after=<seq>` and
/// deduplicate by `(episode_id, seq)`. Reconnecting never re-executes tools.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchEvent {
    pub schema_version: ResearchSchema,
    pub event_id: EventId,
    pub episode_id: EpisodeId,
    pub seq: u64,
    pub occurred_at: Timestamp,
    #[serde(default, skip_serializing_if = "Causation::is_empty")]
    pub causation: Causation,
    pub body: EventBody,
    pub origin: Origin,
}

impl ResearchEvent {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ResearchEvent";
        ensure(self.seq >= 1, c, || "seq starts at 1".into())?;
        match &self.body {
            EventBody::ToolStarted {
                arguments: Some(a), ..
            } => {
                self.schema_version
                    .require_v2(c, "tool.started arguments")?;
                a.validate()
            }
            EventBody::StepRationale { text, .. } => {
                self.schema_version.require_v2(c, "step.rationale")?;
                ensure(!text.trim().is_empty(), c, || "empty rationale".into())
            }
            EventBody::ToolFinished { result } => result.validate(),
            EventBody::JobUpdated { job } => job.validate(),
            EventBody::ArtifactCreated { artifact }
            | EventBody::AnswerCompleted {
                answer: artifact, ..
            } => artifact.validate(),
            _ => Ok(()),
        }
    }
}

/// Checks a contiguous slice of one episode's log (a full log or a replay after a cursor): one
/// episode, gap-free `seq`, unique event ids, status changes that start where the previous one
/// ended, and jobs that never leave a terminal state.
pub fn check_event_log(events: &[ResearchEvent]) -> Result<(), ContractError> {
    let err = |m: String| Err(ContractError::new("ResearchEvent log", m));
    let Some(first) = events.first() else {
        return Ok(());
    };
    let mut ids = BTreeSet::new();
    let mut status: Option<EpisodeStatus> = None;
    let mut jobs: BTreeMap<&JobId, JobStatus> = BTreeMap::new();
    for (i, e) in events.iter().enumerate() {
        e.validate()?;
        if e.episode_id != first.episode_id {
            return err(format!("event {} belongs to another episode", e.event_id));
        }
        if e.seq != first.seq + i as u64 {
            return err(format!(
                "seq {} at position {i}: expected {} (gap or duplicate)",
                e.seq,
                first.seq + i as u64
            ));
        }
        if !ids.insert(&e.event_id) {
            return err(format!("duplicate event_id {}", e.event_id));
        }
        match &e.body {
            EventBody::StatusChanged { from, to, .. } => {
                if status.is_some_and(|s| s != *from) {
                    return err(format!(
                        "seq {}: status change from {from:?} but episode is {:?}",
                        e.seq,
                        status.unwrap()
                    ));
                }
                status = Some(*to);
            }
            EventBody::JobUpdated { job } => {
                if let Some(prev) = jobs.get(&job.job_id) {
                    if prev.is_terminal() && *prev != job.status {
                        return err(format!("job {} left terminal state {prev:?}", job.job_id));
                    }
                }
                jobs.insert(&job.job_id, job.status);
            }
            _ => {}
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- jobs

/// Public job state. Leases, fencing tokens and the scheduler stay internal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    /// Cancellation asked; the work may still be running.
    CancelRequested,
    Succeeded,
    Failed,
    Cancelled,
    /// The effect of an interrupted attempt is uncertain and is being reconciled. Not `failed`, and
    /// not re-run blindly.
    Reconciling,
}

impl JobStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            JobStatus::Succeeded | JobStatus::Failed | JobStatus::Cancelled
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JobProgress {
    pub completed: u64,
    /// Unknown totals stay unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JobRef {
    pub job_id: JobId,
    /// Operation name, e.g. `models_forecast`, `analysis_execute`.
    pub kind: String,
    pub status: JobStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobProgress>,
    #[serde(default)]
    pub outputs: Vec<ArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorEnvelope>,
    pub submitted_at: Timestamp,
    pub updated_at: Timestamp,
}

impl JobRef {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "JobRef";
        ensure(
            self.status != JobStatus::Failed || self.error.is_some(),
            c,
            || "a failed job carries an error".into(),
        )?;
        ensure(
            self.status != JobStatus::Succeeded || self.error.is_none(),
            c,
            || "a succeeded job carries no error".into(),
        )?;
        if let Some(p) = &self.progress {
            ensure(p.total.is_none_or(|t| p.completed <= t), c, || {
                "progress.completed exceeds total".into()
            })?;
        }
        self.outputs.iter().try_for_each(ArtifactRef::validate)
    }
}

// ---------------------------------------------------------------- tools

/// Side-effect class of a tool. Approval and idempotency policy key on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolEffect {
    Read,
    WorkspaceWrite,
    Compute,
    NetworkFetch,
    ExternalWrite,
}

/// Did the call run? Independent of what it found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    /// Long work started; follow `job`.
    Accepted,
    Completed,
    /// The host or service declined (scope, invalid input, unsupported).
    Refused,
    /// Technical failure.
    Failed,
    Cancelled,
}

/// What a completed call established. `insufficient_evidence` and `out_of_domain` are valid
/// scientific conclusions of a successful execution, not errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScientificStatus {
    NotAssessed,
    Conclusive,
    Inconclusive,
    InsufficientEvidence,
    OutOfDomain,
}

/// Result envelope of every IGNIS tool (MCP or internal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolResult {
    pub schema_version: ToolsSchema,
    pub call_id: ToolCallId,
    pub tool: String,
    pub execution_status: ExecutionStatus,
    pub scientific_status: ScientificStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<InlinePayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job: Option<JobRef>,
    #[serde(default)]
    pub artifact_refs: Vec<ArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorEnvelope>,
    pub origin: Origin,
}

impl ToolResult {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ToolResult";
        let not_assessed = self.scientific_status == ScientificStatus::NotAssessed;
        match self.execution_status {
            ExecutionStatus::Accepted => {
                ensure(self.job.is_some(), c, || {
                    "accepted needs the job handle".into()
                })?;
                ensure(self.result.is_none() && self.error.is_none(), c, || {
                    "accepted has no result or error yet".into()
                })?;
                ensure(not_assessed, c, || {
                    "nothing is assessed before the job finishes".into()
                })?;
            }
            ExecutionStatus::Completed => {
                ensure(self.error.is_none(), c, || {
                    "a completed call has no error".into()
                })?;
                ensure(
                    self.job
                        .as_ref()
                        .is_none_or(|j| j.status == JobStatus::Succeeded),
                    c,
                    || "completed with an unfinished job".into(),
                )?;
            }
            ExecutionStatus::Refused | ExecutionStatus::Failed | ExecutionStatus::Cancelled => {
                ensure(self.error.is_some(), c, || {
                    "refused/failed/cancelled calls carry an error".into()
                })?;
                ensure(self.result.is_none(), c, || {
                    "refused/failed/cancelled calls carry no result".into()
                })?;
                ensure(not_assessed, c, || {
                    "a call that did not complete cannot assess anything".into()
                })?;
            }
        }
        if let Some(r) = &self.result {
            r.validate()?;
        }
        if let Some(j) = &self.job {
            j.validate()?;
        }
        self.artifact_refs
            .iter()
            .try_for_each(ArtifactRef::validate)
    }
}

/// Lane that implements a tool's domain logic (the MCP transport is always Mircea's).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Lane {
    Iancu,
    Mircea,
    Raul,
    Tudor,
}

/// Static description of a tool: wire name, effect, idempotency, required capabilities and the
/// contract types of its arguments and result payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolDescriptor {
    pub schema_version: ToolsSchema,
    /// Portable wire name (`^[a-z][a-z0-9_]*$`).
    pub name: String,
    pub summary: String,
    pub effect: ToolEffect,
    pub idempotent: bool,
    pub capabilities: Vec<Capability>,
    /// Exported contract type of the arguments.
    pub input_contract: String,
    /// Exported contract type of `ToolResult.result.value` (or of the job output artifact).
    pub output_contract: String,
    /// May answer `accepted` with a job.
    pub may_return_job: bool,
    pub domain_owner: Lane,
}

/// The tools fixed by C0: wire names, effects and contracts that the MCP transport (Mircea) exposes
/// over the owners' services. A tool is registered with the runtime only once its service exists;
/// this list is the contract, not a claim that the tools run.
pub fn tool_catalog() -> Vec<ToolDescriptor> {
    use Capability as C;
    use ToolEffect as E;
    let t = |name: &str,
             summary: &str,
             effect,
             idempotent,
             caps: &[Capability],
             i: &str,
             o: &str,
             job,
             owner| ToolDescriptor {
        schema_version: ToolsSchema::V1,
        name: name.into(),
        summary: summary.into(),
        effect,
        idempotent,
        capabilities: caps.to_vec(),
        input_contract: i.into(),
        output_contract: o.into(),
        may_return_job: job,
        domain_owner: owner,
    };
    vec![
        t("sources_search", "Hybrid search over the pinned corpus snapshot; returns a page of evidence hits, not documents. To widen recall, add expected_terms (schema evidence.v2, the default): 3 to 8 concepts, equivalent phrasings, abbreviations, instruments or materials a relevant passage would contain, each with importance high, medium or low (e.g. {\"term\": \"limiting oxygen index\", \"importance\": \"high\"}, {\"term\": \"LOI\"}); never the answer you expect. Terms the corpus does not hold are ignored.", E::Read, true, &[C::ReadSources], "SearchRequest", "SearchPage", false, Lane::Raul),
        t("sources_read", "Read a page range, passage or table of one source version; says how to continue.", E::Read, true, &[C::ReadSources], "ReadRequest", "ReadResult", false, Lane::Raul),
        t("datasets_inspect", "Describe a dataset bundle: views, columns, units, time origin, partitions.", E::Read, true, &[C::ReadData], "DatasetRef", "DatasetBundle", false, Lane::Raul),
        t("models_inspect", "A model release with its construction manifest, domain and state.", E::Read, true, &[C::RunModels], "ReleaseRef", "ModelRelease", false, Lane::Iancu),
        t("models_forecast", "Strict forward of a pinned release on an admitted prefix.", E::Compute, true, &[C::RunModels, C::ReadData], "ForecastRequest", "AnalysisArtifact", true, Lane::Iancu),
        t("analysis_execute", "Run a Python program once in the isolated runner (no network, read-only inputs from workspace artifacts); stdout, stderr and files written under /out become artifacts of an ExecutionRecord. Returns a job; follow it with jobs_get.", E::Compute, true, &[C::ExecuteCode], "AnalysisExecuteRequest", "ExecutionRecord", true, Lane::Mircea),
        t("artifacts_read", "Read a byte range or JSON Pointer of a stored artifact.", E::Read, true, &[], "ArtifactReadRequest", "InlinePayload", false, Lane::Mircea),
        t("jobs_get", "State of a job this workspace submitted (e.g. by models_forecast): status, progress, output artifacts or error.", E::Read, true, &[], "JobLookup", "JobRef", false, Lane::Mircea),
        t("wiki_patch", "Write a wiki revision against a base revision (compare-and-swap).", E::WorkspaceWrite, false, &[C::WikiWrite], "WikiPatch", "WikiRevision", false, Lane::Raul),
        // Wave K1 (K1.9). Request/response types stay in their families.
        t("answer_submit", "Submit this turn's answer: text and view blocks bound to claims, claims bound to evidence or artifact values returned in this episode. The host validates it and stores the ResearchAnswer; a refusal lists each problem so the answer can be fixed and resubmitted.", E::Compute, true, &[], "AnswerSubmission", "ResearchAnswer", false, Lane::Mircea),
        t("sources_outline", "Section tree of one source version (titles, page spans; summaries labelled generated).", E::Read, true, &[C::ReadSources], "OutlineRequest", "SourceOutline", false, Lane::Raul),
        t("conditions_query", "Experimental conditions per run and the tested envelope per investigation, gaps explicit (missing, conflicting, not applicable).", E::Read, true, &[C::ReadData], "ConditionsQuery", "ConditionsTable", false, Lane::Raul),
        t("graph_neighbors", "Typed links around a run, source, release or concept in the combustion graph.", E::Read, true, &[C::ReadSources], "GraphNeighborsRequest", "GraphNeighborsPage", false, Lane::Raul),
        t("wiki_read", "Read a wiki page (itemized, with its revision) or list pages.", E::Read, true, &[C::ReadSources], "WikiReadRequest", "WikiReadResult", false, Lane::Raul),
        t("web_search", "Search the web for candidate sources outside the corpus; results are candidates, never corpus evidence.", E::NetworkFetch, true, &[C::NetworkFetch], "WebSearchRequest", "WebSearchPage", false, Lane::Raul),
        t("web_fetch", "Fetch one http(s) page into a snapshot artifact under the SSRF policy; never promoted into the corpus.", E::NetworkFetch, true, &[C::NetworkFetch], "WebFetchRequest", "ExternalSourceCandidate", false, Lane::Raul),
    ]
}

/// Argument of `datasets_inspect`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DatasetRef {
    pub dataset_id: super::DatasetId,
}

/// Argument of `jobs_get`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JobLookup {
    pub job_id: JobId,
}

// ---------------------------------------------------------------- analysis execution

/// Language of an `analysis_execute` program. Closed: a new value needs a runner image that
/// provides it (and a preflight that proves it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisLanguage {
    /// CPython of the pinned runner image, run isolated (`python -I`).
    Python,
}

/// One input of an analysis: an artifact of the caller's workspace, mounted read-only at
/// `/in/<name>`. Only artifacts reach the runner; it never sees the host filesystem or other stores.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisInput {
    /// File name under `/in` (`[A-Za-z0-9._-]`, not starting with `.`).
    pub name: String,
    pub artifact_id: ArtifactId,
}

/// Argument of `analysis_execute`: a program run once in the isolated runner (no network, read-only
/// root, separate uid, no host secrets). Its stdout, stderr and the regular files it writes under
/// `/out` become artifacts of an [`ExecutionRecord`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisExecuteRequest {
    pub language: AnalysisLanguage,
    /// The program, executed as written.
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<AnalysisInput>,
    /// Idempotency key: a retry with the same key returns the same job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<ClientRequestId>,
}

/// Whether `name` is a plain file name the runner may create: no separators, no `.`/`..`, no hidden
/// or empty names, at most 255 bytes (the file-system limit).
pub fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && !name.starts_with('.')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

impl AnalysisExecuteRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(
            !self.source.trim().is_empty(),
            "AnalysisExecuteRequest",
            || "source is empty".into(),
        )?;
        let mut seen = BTreeSet::new();
        for i in &self.inputs {
            ensure(
                is_plain_file_name(&i.name),
                "AnalysisExecuteRequest",
                || {
                    format!(
                        "input name {:?} is not a plain file name ([A-Za-z0-9._-], not starting with '.')",
                        i.name
                    )
                },
            )?;
            ensure(
                seen.insert(i.name.as_str()),
                "AnalysisExecuteRequest",
                || format!("input name {:?} is used twice", i.name),
            )?;
        }
        Ok(())
    }
}

/// Isolation the runner actually provided. Recorded on every execution and in capability status;
/// never inferred from configuration alone (the preflight probe proves it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunnerIsolation {
    /// OCI container under gVisor (`runsc`): a user-space kernel between the program and the host.
    Gvisor,
    /// OCI container under the host kernel (namespaces, seccomp, no capabilities). Development
    /// isolation; the public deployment uses `gvisor`.
    Container,
}

/// What a program ran in: the content address of the image, the OCI runtime and its isolation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunnerEnvironment {
    /// Image id (`sha256:` of the image config), pinned by the host.
    pub image_id: Digest,
    /// OCI runtime name as the container engine reports it (`runsc`, `runc`).
    pub oci_runtime: String,
    pub isolation: RunnerIsolation,
    /// Interpreter version reported by the image during preflight.
    pub interpreter: String,
}

/// Resource budget of one execution, set by the host (never by the request).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunnerLimits {
    pub wall_seconds: u64,
    pub memory_bytes: u64,
    /// CPU quota in thousandths of a core.
    pub cpu_millis: u64,
    pub pids: u64,
    /// Scratch space (`/tmp`, in memory).
    pub tmp_bytes: u64,
    /// Total bytes of `/out` files kept as artifacts.
    pub output_bytes: u64,
    /// Bytes of stdout and of stderr each kept as an artifact (the rest is counted, not stored).
    pub stream_bytes: u64,
}

/// How the program ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExecutionOutcome {
    /// The process exited by itself (any code; non-zero is the program's failure, not the host's).
    Exited { exit_code: i64 },
    /// Killed at the wall-clock budget.
    TimedOut,
    /// Killed by the memory budget.
    OutOfMemory,
}

/// A captured output stream. `truncated` means the stream exceeded the budget: `total_bytes` were
/// written, only the first `stream_bytes` are in `artifact`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapturedStream {
    pub total_bytes: u64,
    pub truncated: bool,
    /// Absent when the stream was empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ArtifactRef>,
}

/// A file the program wrote under `/out`, kept as an artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisOutput {
    /// Path relative to `/out`, `/`-separated.
    pub path: String,
    pub artifact: ArtifactRef,
}

/// An input as the program saw it: `/in/<name>` held exactly these bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MountedInput {
    pub name: String,
    pub artifact: ArtifactRef,
}

/// A path under `/out` that was not kept, and why (symbolic link, special file, budget).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RejectedOutput {
    pub path: String,
    pub reason: String,
}

/// The record of one `analysis_execute` run (the job's first output artifact): code identity,
/// environment, inputs, budget, outcome and the artifacts produced. A computed value cited in an
/// answer points into one of `outputs` (or `stdout`), and this record says how it was computed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRecord {
    pub schema_version: ToolsSchema,
    pub language: AnalysisLanguage,
    /// The program exactly as executed.
    pub program: ArtifactRef,
    pub environment: RunnerEnvironment,
    /// Inputs as mounted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<MountedInput>,
    pub limits: RunnerLimits,
    pub outcome: ExecutionOutcome,
    pub stdout: CapturedStream,
    pub stderr: CapturedStream,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<AnalysisOutput>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_outputs: Vec<RejectedOutput>,
    pub started_at: Timestamp,
    pub finished_at: Timestamp,
    /// `synthetic_fixture` when any input is.
    pub origin: Origin,
}

fn is_relative_out_path(p: &str) -> bool {
    !p.is_empty()
        && !p.starts_with('/')
        && p.split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != ".." && !seg.contains('\\'))
}

impl ExecutionRecord {
    pub fn validate(&self) -> Result<(), ContractError> {
        let l = &self.limits;
        ensure(
            [l.wall_seconds, l.memory_bytes, l.cpu_millis, l.pids]
                .iter()
                .all(|v| *v > 0),
            "ExecutionRecord",
            || "a runner budget of zero cannot run anything".into(),
        )?;
        for s in [&self.stdout, &self.stderr] {
            ensure(
                s.truncated == (s.total_bytes > l.stream_bytes),
                "ExecutionRecord",
                || "a stream is truncated exactly when it exceeded stream_bytes".into(),
            )?;
            ensure(
                s.artifact.as_ref().map_or(0, |a| a.byte_length)
                    == s.total_bytes.min(l.stream_bytes),
                "ExecutionRecord",
                || "a stream artifact holds the kept prefix of the stream".into(),
            )?;
        }
        for i in &self.inputs {
            ensure(is_plain_file_name(&i.name), "ExecutionRecord", || {
                format!("input {:?} is not a plain file name", i.name)
            })?;
        }
        let mut seen = BTreeSet::new();
        for o in &self.outputs {
            ensure(is_relative_out_path(&o.path), "ExecutionRecord", || {
                format!("output path {:?} is not relative to /out", o.path)
            })?;
            ensure(seen.insert(o.path.as_str()), "ExecutionRecord", || {
                format!("output path {:?} listed twice", o.path)
            })?;
        }
        ensure(
            self.outputs
                .iter()
                .map(|o| o.artifact.byte_length)
                .sum::<u64>()
                <= l.output_bytes,
            "ExecutionRecord",
            || "kept outputs exceed the output budget".into(),
        )?;
        ensure(
            self.started_at.as_str() <= self.finished_at.as_str(),
            "ExecutionRecord",
            || "finished before it started".into(),
        )
    }
}

/// Argument of `models_inspect`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReleaseRef {
    pub release_id: ReleaseId,
}

// ---------------------------------------------------------------- answers

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnswerStatus {
    Complete,
    Partial,
    /// A useful conclusion: the data to answer does not exist (yet), and the answer says what is missing.
    InsufficientEvidence,
    Failed,
}

/// What a claim asserts. The kind fixes what may support it, so a prediction cannot be reported as
/// a measurement and a computed value cannot be cited to prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    /// "Paper X reports ..." (evidence).
    SourceReports,
    /// A measured value (evidence of class original_observation or source_derived).
    Observed,
    /// Computed by IGNIS (an artifact value).
    Computed,
    /// A model prediction (a value in an `AnalysisArtifact`).
    Predicted,
    /// The agent's interpretation; may cite anything or nothing.
    Interpretation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClaimSupport {
    Evidence {
        evidence_id: EvidenceId,
    },
    /// A value inside an artifact, addressed by an RFC 6901 JSON Pointer, so numbers in the text are
    /// bound to the computed field instead of being transcribed.
    ArtifactValue {
        artifact_id: ArtifactId,
        pointer: String,
    },
}

/// How a claim's support was checked (K1.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportCheckMethod {
    /// A deterministic host rule (the cited value resolves, the quote occurs at the locator).
    HostRule,
    /// An entailment (NLI) model over the cited passage.
    NliModel,
    /// A language model judging the cited passage.
    LlmJudge,
    /// A person.
    Human,
}

/// Does the cited support carry the claim's statement?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportVerdict {
    Supported,
    PartiallySupported,
    NotSupported,
    /// The check could not decide (support unreadable, out of the checker's reach).
    Unverifiable,
}

/// The result of checking a claim against its cited support. Set by the host or the evaluator, never
/// by the agent that wrote the claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SupportCheck {
    pub method: SupportCheckMethod,
    /// The checking model; present exactly for the model methods.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    pub verdict: SupportVerdict,
    /// The checker's own confidence in [0, 1], when it reports one; never invented.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Finite>,
}

impl SupportCheck {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "SupportCheck";
        let model = matches!(
            self.method,
            SupportCheckMethod::NliModel | SupportCheckMethod::LlmJudge
        );
        ensure(model == self.model_id.is_some(), c, || {
            format!(
                "{:?}: model_id is present exactly for model checks",
                self.method
            )
        })?;
        ensure(
            self.model_id.as_ref().is_none_or(|m| !m.trim().is_empty()),
            c,
            || "empty model_id".into(),
        )?;
        ensure(
            self.confidence
                .is_none_or(|p| (0.0..=1.0).contains(&p.get())),
            c,
            || "confidence is in [0, 1]".into(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub claim_id: ClaimId,
    pub kind: ClaimKind,
    pub text: String,
    #[serde(default)]
    pub support: Vec<ClaimSupport>,
    /// v2: how the support was checked, when it was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_by: Option<SupportCheck>,
}

/// What a plotted series is. Fixed against the artifact it reads: a predicted series comes only from
/// an `AnalysisArtifact`, and an observed or computed one never does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SeriesRole {
    Observed,
    Computed,
    Predicted,
}

/// An uncertainty band around a series (e.g. two quantile rows of an `AnalysisArtifact`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SeriesBand {
    pub lower_pointer: String,
    pub upper_pointer: String,
}

/// One series of a `series_comparison` view, read from one JSON artifact by pointers. Times are
/// seconds since the time origin the artifact itself states; the view checks that every series shares
/// it. Values are SI in the unit the artifact states; display conversion happens in the chart only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SeriesRef {
    pub artifact_id: ArtifactId,
    pub role: SeriesRole,
    /// Array of times (s since the time origin), e.g. `/times_s`.
    pub time_pointer: String,
    /// Array of values, same length as the times, e.g. `/mean`.
    pub value_pointer: String,
    /// The SI unit string, e.g. `/unit`.
    pub unit_pointer: String,
    /// The time origin string, e.g. `/time_origin`.
    pub time_origin_pointer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub band: Option<SeriesBand>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// The closed catalog of answer views (K1.4, co-designed with the workspace). The agent places a
/// component by naming it with references; it never sends markup. Every reference must be one the
/// answer lists (evidence, artifacts, releases), so a view cannot show what the answer does not cite.
/// A new kind is a contract change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ViewSpec {
    /// The cited passage, opened at its locator in the original.
    SourceExcerpt { evidence_id: EvidenceId },
    /// Observation versus prediction (or any series) on one time axis.
    SeriesComparison { series: Vec<SeriesRef> },
    /// Conditions per run with gaps explicit (a `ConditionsTable` artifact).
    ConditionsMatrix { table: ArtifactId },
    /// Tested O2/pressure envelope against habitat atmospheres (a `ConditionsTable` artifact).
    AtmosphereMap { table: ArtifactId },
    /// A source's video with our extracted contour drawn over it (a `VisualObservationBundle`).
    VideoOverlay {
        source_id: SourceId,
        bundle: ArtifactId,
    },
    /// A model release with its construction and qualification state.
    ReleaseCard { release_id: ReleaseId },
    /// A table (array of rows or object) inside a JSON artifact.
    ArtifactTable {
        artifact_id: ArtifactId,
        pointer: String,
    },
}

/// The contract a view requires of an artifact it reads, if any.
fn view_contract(view: &ViewSpec) -> Option<(&ArtifactId, &'static str)> {
    match view {
        ViewSpec::ConditionsMatrix { table } | ViewSpec::AtmosphereMap { table } => {
            Some((table, "ConditionsTable"))
        }
        ViewSpec::VideoOverlay { bundle, .. } => Some((bundle, "VisualObservationBundle")),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnswerBlock {
    Text {
        text: String,
        #[serde(default)]
        claim_ids: Vec<ClaimId>,
    },
    Artifact {
        artifact_id: ArtifactId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
    },
    /// v2: a component from the closed view catalog, bound to the claims it illustrates.
    View {
        view: ViewSpec,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
        #[serde(default)]
        claim_ids: Vec<ClaimId>,
    },
}

/// The answer of a turn: text blocks bound to claims, claims bound to evidence or artifact values,
/// limitations, and the releases used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchAnswer {
    pub schema_version: ResearchSchema,
    pub answer_id: AnswerId,
    pub episode_id: EpisodeId,
    pub status: AnswerStatus,
    pub blocks: Vec<AnswerBlock>,
    #[serde(default)]
    pub claims: Vec<Claim>,
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
    #[serde(default)]
    pub artifacts: Vec<ArtifactRef>,
    #[serde(default)]
    pub model_releases: Vec<ReleaseId>,
    #[serde(default)]
    pub limitations: Vec<String>,
    pub origin: Origin,
    pub created_at: Timestamp,
}

/// Why a submitted answer was refused (K1.3). Each is actionable: it names what to fix.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AnswerProblemKind {
    /// Cited evidence was not returned by a tool result in this episode.
    UnknownEvidence,
    /// A cited artifact does not exist in this episode's workspace.
    UnknownArtifact,
    /// A JSON Pointer is malformed, does not resolve, or resolves to a value without a unit.
    UnresolvedPointer,
    /// The support does not fit the claim kind (a prediction cited as an observation, a computed
    /// value cited to prose, a predicted series read from a non-model artifact).
    KindSupportMismatch,
    /// A release is used but not pinned on the episode (or not listed in `model_releases`).
    UnpinnedRelease,
    /// A block names a claim the answer does not define.
    UnknownClaim,
    /// A view reads an artifact of the wrong contract.
    ViewContractMismatch,
    /// Anything else structural (duplicate ids, empty answer, missing limitations, a support check
    /// written by the agent).
    InvalidStructure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnswerProblem {
    pub kind: AnswerProblemKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<ClaimId>,
    /// 0-based index into `blocks`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block: Option<u32>,
    pub detail: String,
}

impl AnswerProblem {
    fn claim(kind: AnswerProblemKind, id: &ClaimId, detail: String) -> Self {
        AnswerProblem {
            kind,
            claim_id: Some(id.clone()),
            block: None,
            detail,
        }
    }
    fn block(kind: AnswerProblemKind, i: usize, detail: String) -> Self {
        AnswerProblem {
            kind,
            claim_id: None,
            block: Some(i as u32),
            detail,
        }
    }
}

/// The structured refusal of `answer_submit`: every problem found, so the agent fixes all of them in
/// one resubmission. Stored as an artifact (contract `AnswerRefusal`) referenced by the refused
/// `ToolResult`, whose `error.message` renders the same list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnswerRefusal {
    pub schema_version: ResearchSchema,
    pub problems: Vec<AnswerProblem>,
    pub origin: Origin,
}

impl AnswerRefusal {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "AnswerRefusal";
        self.schema_version.require_v2(c, c)?;
        ensure(!self.problems.is_empty(), c, || {
            "a refusal names at least one problem".into()
        })?;
        ensure(
            self.problems.iter().all(|p| !p.detail.trim().is_empty()),
            c,
            || "every problem says what to fix".into(),
        )
    }
}

/// What the agent submits through `answer_submit` (K1.3). It names evidence, artifacts and releases
/// only by id: the host resolves them against what tool results returned in this episode and the
/// episode's pins, builds the `ResearchAnswer` (ids, timestamps, origin, full references) and stores
/// it. The agent cannot write a support check or a reference the host did not issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnswerSubmission {
    pub status: AnswerStatus,
    pub blocks: Vec<AnswerBlock>,
    #[serde(default)]
    pub claims: Vec<Claim>,
    #[serde(default)]
    pub model_releases: Vec<ReleaseId>,
    /// Required for `partial` and `insufficient_evidence`: what is missing.
    #[serde(default)]
    pub limitations: Vec<String>,
    /// Idempotency key: a retry with the same key returns the same answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<ClientRequestId>,
}

/// RFC 6901: empty (the whole document) or `/`-prefixed, with `~` only as `~0` / `~1`.
pub fn is_json_pointer(p: &str) -> bool {
    if p.is_empty() {
        return true;
    }
    if !p.starts_with('/') {
        return false;
    }
    let b = p.as_bytes();
    b.iter()
        .enumerate()
        .all(|(i, &c)| c != b'~' || matches!(b.get(i + 1), Some(b'0' | b'1')))
}

fn view_pointers(view: &ViewSpec) -> Vec<&str> {
    match view {
        ViewSpec::SeriesComparison { series } => series
            .iter()
            .flat_map(|s| {
                let mut v = vec![
                    s.time_pointer.as_str(),
                    s.value_pointer.as_str(),
                    s.unit_pointer.as_str(),
                    s.time_origin_pointer.as_str(),
                ];
                if let Some(b) = &s.band {
                    v.extend([b.lower_pointer.as_str(), b.upper_pointer.as_str()]);
                }
                v
            })
            .collect(),
        ViewSpec::ArtifactTable { pointer, .. } => vec![pointer.as_str()],
        _ => vec![],
    }
}

fn view_artifacts(view: &ViewSpec) -> Vec<&ArtifactId> {
    match view {
        ViewSpec::SeriesComparison { series } => series.iter().map(|s| &s.artifact_id).collect(),
        ViewSpec::ConditionsMatrix { table } | ViewSpec::AtmosphereMap { table } => vec![table],
        ViewSpec::VideoOverlay { bundle, .. } => vec![bundle],
        ViewSpec::ArtifactTable { artifact_id, .. } => vec![artifact_id],
        ViewSpec::SourceExcerpt { .. } | ViewSpec::ReleaseCard { .. } => vec![],
    }
}

/// Checks that need no resolution: ids, pointers, the support shape each claim kind requires, block
/// references and release cards. Shared by the submission and the stored answer.
fn structural_problems(
    blocks: &[AnswerBlock],
    claims: &[Claim],
    releases: &[ReleaseId],
) -> Vec<AnswerProblem> {
    use AnswerProblemKind as K;
    let mut out = Vec::new();
    let mut ids = BTreeSet::new();
    for claim in claims {
        let id = &claim.claim_id;
        if !ids.insert(id) {
            out.push(AnswerProblem::claim(
                K::InvalidStructure,
                id,
                format!("claim {id} is defined twice"),
            ));
        }
        if let Some(check) = &claim.checked_by {
            if let Err(e) = check.validate() {
                out.push(AnswerProblem::claim(K::InvalidStructure, id, e.to_string()));
            }
        }
        let mut evidence = false;
        let mut value = false;
        for s in &claim.support {
            match s {
                ClaimSupport::Evidence { .. } => evidence = true,
                ClaimSupport::ArtifactValue { pointer, .. } => {
                    value = true;
                    if !is_json_pointer(pointer) {
                        out.push(AnswerProblem::claim(
                            K::UnresolvedPointer,
                            id,
                            format!("{pointer:?} is not a JSON Pointer (RFC 6901)"),
                        ));
                    }
                }
            }
        }
        let need = match claim.kind {
            ClaimKind::SourceReports => (!evidence).then_some("cite evidence"),
            ClaimKind::Observed if !evidence => Some("cite the measured value's evidence"),
            ClaimKind::Observed if value => Some(
                "cite evidence only (an artifact value is computed or predicted, not observed)",
            ),
            ClaimKind::Computed => (!value).then_some("cite the artifact value it reports"),
            ClaimKind::Predicted => {
                (!value).then_some("cite the AnalysisArtifact value it reports")
            }
            _ => None,
        };
        if let Some(need) = need {
            out.push(AnswerProblem::claim(
                K::KindSupportMismatch,
                id,
                format!("{:?} claim {id} must {need}", claim.kind),
            ));
        }
    }
    let pinned: BTreeSet<&ReleaseId> = releases.iter().collect();
    for (i, b) in blocks.iter().enumerate() {
        let refs = match b {
            AnswerBlock::Text { claim_ids, .. } | AnswerBlock::View { claim_ids, .. } => {
                claim_ids.as_slice()
            }
            AnswerBlock::Artifact { .. } => &[],
        };
        for id in refs.iter().filter(|id| !ids.contains(id)) {
            out.push(AnswerProblem::block(
                K::UnknownClaim,
                i,
                format!("block references unknown claim {id}"),
            ));
        }
        let AnswerBlock::View { view, .. } = b else {
            continue;
        };
        if let ViewSpec::SeriesComparison { series } = view {
            if series.is_empty() {
                out.push(AnswerProblem::block(
                    K::InvalidStructure,
                    i,
                    "a series_comparison needs at least one series".into(),
                ));
            }
        }
        for p in view_pointers(view)
            .into_iter()
            .filter(|p| !is_json_pointer(p))
        {
            out.push(AnswerProblem::block(
                K::UnresolvedPointer,
                i,
                format!("{p:?} is not a JSON Pointer (RFC 6901)"),
            ));
        }
        if let ViewSpec::ReleaseCard { release_id } = view {
            if !pinned.contains(release_id) {
                out.push(AnswerProblem::block(
                    K::UnpinnedRelease,
                    i,
                    format!("release {release_id} is shown but not listed in model_releases"),
                ));
            }
        }
    }
    out
}

impl AnswerSubmission {
    /// Every problem visible without resolving references. The host adds the resolution problems
    /// (unknown evidence/artifact, unresolved pointer or unit, unpinned release) and refuses with
    /// all of them.
    pub fn problems(&self) -> Vec<AnswerProblem> {
        use AnswerProblemKind as K;
        let mut out = structural_problems(&self.blocks, &self.claims, &self.model_releases);
        if self.blocks.is_empty() && self.status != AnswerStatus::Failed {
            out.push(AnswerProblem {
                kind: K::InvalidStructure,
                claim_id: None,
                block: None,
                detail: "an answer has at least one block".into(),
            });
        }
        if matches!(
            self.status,
            AnswerStatus::Partial | AnswerStatus::InsufficientEvidence
        ) && !self.limitations.iter().any(|l| !l.trim().is_empty())
        {
            out.push(AnswerProblem {
                kind: K::InvalidStructure,
                claim_id: None,
                block: None,
                detail: format!(
                    "a {:?} answer states what is missing in limitations",
                    self.status
                ),
            });
        }
        for c in self.claims.iter().filter(|c| c.checked_by.is_some()) {
            out.push(AnswerProblem::claim(
                K::InvalidStructure,
                &c.claim_id,
                "checked_by is set by the host or evaluator, not submitted".into(),
            ));
        }
        out
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        match self.problems().into_iter().next() {
            Some(p) => Err(ContractError::new("AnswerSubmission", p.detail)),
            None => Ok(()),
        }
    }

    /// Evidence ids the host must resolve (claim support and source excerpts).
    pub fn cited_evidence(&self) -> BTreeSet<&EvidenceId> {
        let claims = self
            .claims
            .iter()
            .flat_map(|c| &c.support)
            .filter_map(|s| match s {
                ClaimSupport::Evidence { evidence_id } => Some(evidence_id),
                ClaimSupport::ArtifactValue { .. } => None,
            });
        let views = self.blocks.iter().filter_map(|b| match b {
            AnswerBlock::View {
                view: ViewSpec::SourceExcerpt { evidence_id },
                ..
            } => Some(evidence_id),
            _ => None,
        });
        claims.chain(views).collect()
    }

    /// Artifact ids the host must resolve (claim values, artifact blocks, views).
    pub fn cited_artifacts(&self) -> BTreeSet<&ArtifactId> {
        let claims = self
            .claims
            .iter()
            .flat_map(|c| &c.support)
            .filter_map(|s| match s {
                ClaimSupport::ArtifactValue { artifact_id, .. } => Some(artifact_id),
                ClaimSupport::Evidence { .. } => None,
            });
        let blocks = self.blocks.iter().flat_map(|b| match b {
            AnswerBlock::Artifact { artifact_id, .. } => vec![artifact_id],
            AnswerBlock::View { view, .. } => view_artifacts(view),
            AnswerBlock::Text { .. } => vec![],
        });
        claims.chain(blocks).collect()
    }
}

impl ResearchAnswer {
    /// Every problem of the stored answer: the structural ones plus those that need the resolved
    /// references it carries (evidence classes, artifact contracts, releases of predictions).
    pub fn problems(&self) -> Vec<AnswerProblem> {
        use AnswerProblemKind as K;
        let mut out = structural_problems(&self.blocks, &self.claims, &self.model_releases);
        let evidence: BTreeMap<_, _> = self.evidence.iter().map(|e| (&e.evidence_id, e)).collect();
        let artifacts: BTreeMap<_, _> =
            self.artifacts.iter().map(|a| (&a.artifact_id, a)).collect();
        let pinned: BTreeSet<&ReleaseId> = self.model_releases.iter().collect();
        let is_model = |a: &ArtifactRef| a.contract.as_deref() == Some("AnalysisArtifact");
        for claim in &self.claims {
            let id = &claim.claim_id;
            let mut prediction = false;
            for s in &claim.support {
                match s {
                    ClaimSupport::Evidence { evidence_id } => match evidence.get(evidence_id) {
                        None => out.push(AnswerProblem::claim(
                            K::UnknownEvidence,
                            id,
                            format!("claim {id} cites unknown evidence {evidence_id}"),
                        )),
                        Some(e)
                            if claim.kind == ClaimKind::Observed
                                && !matches!(
                                    e.evidence_class,
                                    EvidenceClass::OriginalObservation
                                        | EvidenceClass::SourceDerived
                                ) =>
                        {
                            out.push(AnswerProblem::claim(
                                K::KindSupportMismatch,
                                id,
                                format!(
                                    "observed claim {id} cites {:?} evidence",
                                    e.evidence_class
                                ),
                            ))
                        }
                        Some(_) => {}
                    },
                    ClaimSupport::ArtifactValue { artifact_id, .. } => {
                        match artifacts.get(artifact_id) {
                            None => out.push(AnswerProblem::claim(
                                K::UnknownArtifact,
                                id,
                                format!("claim {id} cites unknown artifact {artifact_id}"),
                            )),
                            Some(a) if is_model(a) => {
                                prediction = true;
                                if let Some(r) = &a.produced_by.release_id {
                                    if !pinned.contains(r) {
                                        out.push(AnswerProblem::claim(
                                            K::UnpinnedRelease,
                                            id,
                                            format!("claim {id} uses release {r}, not listed in model_releases"),
                                        ));
                                    }
                                }
                            }
                            Some(_) => {}
                        }
                    }
                }
            }
            let has_value = claim
                .support
                .iter()
                .any(|s| matches!(s, ClaimSupport::ArtifactValue { .. }));
            if claim.kind == ClaimKind::Predicted && has_value && !prediction {
                out.push(AnswerProblem::claim(
                    K::KindSupportMismatch,
                    id,
                    format!("predicted claim {id} cites no AnalysisArtifact value"),
                ));
            }
        }
        for (i, b) in self.blocks.iter().enumerate() {
            let view = match b {
                AnswerBlock::Artifact { artifact_id, .. } => {
                    if !artifacts.contains_key(artifact_id) {
                        out.push(AnswerProblem::block(
                            K::UnknownArtifact,
                            i,
                            format!("block references unknown artifact {artifact_id}"),
                        ));
                    }
                    continue;
                }
                AnswerBlock::Text { .. } => continue,
                AnswerBlock::View { view, .. } => view,
            };
            if let ViewSpec::SourceExcerpt { evidence_id } = view {
                if !evidence.contains_key(evidence_id) {
                    out.push(AnswerProblem::block(
                        K::UnknownEvidence,
                        i,
                        format!("excerpt of unknown evidence {evidence_id}"),
                    ));
                }
            }
            for a in view_artifacts(view) {
                if !artifacts.contains_key(a) {
                    out.push(AnswerProblem::block(
                        K::UnknownArtifact,
                        i,
                        format!("view reads unknown artifact {a}"),
                    ));
                }
            }
            if let Some((a, want)) = view_contract(view) {
                if let Some(r) = artifacts.get(a) {
                    if r.contract.as_deref() != Some(want) {
                        out.push(AnswerProblem::block(
                            K::ViewContractMismatch,
                            i,
                            format!("view needs a {want} artifact, {a} is {:?}", r.contract),
                        ));
                    }
                }
            }
            if let ViewSpec::SeriesComparison { series } = view {
                for s in series {
                    let Some(a) = artifacts.get(&s.artifact_id) else {
                        continue;
                    };
                    let ok = match s.role {
                        SeriesRole::Predicted => is_model(a),
                        SeriesRole::Observed | SeriesRole::Computed => !is_model(a),
                    };
                    if !ok {
                        out.push(AnswerProblem::block(
                            K::KindSupportMismatch,
                            i,
                            format!(
                                "a {:?} series cannot read {} ({:?}): predictions come only from AnalysisArtifact, and only predictions do",
                                s.role, s.artifact_id, a.contract
                            ),
                        ));
                    }
                }
            }
        }
        out
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ResearchAnswer";
        let evidence: BTreeSet<_> = self.evidence.iter().map(|e| &e.evidence_id).collect();
        let artifacts: BTreeSet<_> = self.artifacts.iter().map(|a| &a.artifact_id).collect();
        ensure(evidence.len() == self.evidence.len(), c, || {
            "duplicate evidence_id".into()
        })?;
        ensure(artifacts.len() == self.artifacts.len(), c, || {
            "duplicate artifact_id".into()
        })?;
        self.evidence.iter().try_for_each(EvidenceRef::validate)?;
        self.artifacts.iter().try_for_each(ArtifactRef::validate)?;
        if self
            .blocks
            .iter()
            .any(|b| matches!(b, AnswerBlock::View { .. }))
        {
            self.schema_version.require_v2(c, "a view block")?;
        }
        if self.claims.iter().any(|cl| cl.checked_by.is_some()) {
            self.schema_version.require_v2(c, "a support check")?;
        }
        match self.problems().into_iter().next() {
            Some(p) => Err(ContractError::new(c, p.detail)),
            None => Ok(()),
        }
    }
}

// ---------------------------------------------------------------- traces (K1.8)

/// A contiguous range of one episode's log, both ends included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EventRange {
    pub first_seq: u64,
    pub last_seq: u64,
}

/// One tool call as traced: where it sits in the log and its full input and output as artifacts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TraceToolCall {
    pub call_id: ToolCallId,
    pub tool: String,
    pub started_seq: u64,
    /// Absent when the call had not finished when the trace was cut.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_seq: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_status: Option<ExecutionStatus>,
    /// The arguments as the tool received them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<ArtifactRef>,
    /// The `ToolResult` as returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<ArtifactRef>,
}

/// Token usage of one model over the traced range, as the runtime reported it (measured, not
/// estimated). Unknown counts stay absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TokenUsage {
    pub model: String,
    pub input_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_input_tokens: Option<u64>,
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
}

/// Where a cost figure comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CostBasis {
    /// The provider or runtime reported the cost of these calls.
    ProviderReported,
    /// Measured tokens multiplied by a dated price list (named in `price_list`).
    PriceList,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CostFigure {
    pub amount: Finite,
    /// ISO 4217 code, e.g. `USD`.
    pub currency: String,
    pub basis: CostBasis,
    /// Identity of the price list (provider page and date); present exactly for `price_list`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_list: Option<String>,
}

/// Which benchmark run an episode belongs to (`ignis harness run --variant <v> --tasks <set>`).
/// `task_id` names a gold task of the evaluator's set; the trace never carries the task's answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HarnessRun {
    pub variant: String,
    pub task_id: String,
    /// 0-based repetition of the same task under the same variant.
    pub repetition: u32,
}

/// Everything needed to audit, cost and compare one episode (K1.8): the harness it ran with, its
/// event range, every tool call's input and output, measured usage and cost, and the partitions it
/// touched. Benchmark scorecards read traces; they never re-run an episode to learn its cost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TraceBundle {
    pub schema_version: ResearchSchema,
    pub trace_id: TraceId,
    pub episode_id: EpisodeId,
    pub harness: HarnessRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeThreadRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<HarnessRun>,
    pub events: EventRange,
    /// The traced events as JSONL (`ResearchEvent` per line).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_log: Option<ArtifactRef>,
    #[serde(default)]
    pub tool_calls: Vec<TraceToolCall>,
    #[serde(default)]
    pub usage: Vec<TokenUsage>,
    /// Absent when no cost could be established; never a guess.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<CostFigure>,
    /// Data partitions any tool call of the range read.
    #[serde(default)]
    pub partitions_touched: Vec<Partition>,
    /// The `ResearchAnswer` the range ended with, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<ArtifactRef>,
    pub wall_time_s: Finite,
    pub started_at: Timestamp,
    pub finished_at: Timestamp,
    pub origin: Origin,
}

impl TraceBundle {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "TraceBundle";
        self.schema_version.require_v2(c, c)?;
        self.harness.validate()?;
        let r = self.events;
        ensure(r.first_seq >= 1 && r.first_seq <= r.last_seq, c, || {
            "event range starts at seq >= 1 and is not empty".into()
        })?;
        let within = |s: u64| (r.first_seq..=r.last_seq).contains(&s);
        let mut calls = BTreeSet::new();
        for t in &self.tool_calls {
            ensure(calls.insert(&t.call_id), c, || {
                format!("tool call {} traced twice", t.call_id)
            })?;
            ensure(within(t.started_seq), c, || {
                format!("tool call {} starts outside the range", t.call_id)
            })?;
            ensure(
                t.finished_seq
                    .is_none_or(|f| within(f) && f >= t.started_seq),
                c,
                || format!("tool call {} finishes outside the range", t.call_id),
            )?;
            ensure(
                t.finished_seq.is_some() == t.execution_status.is_some(),
                c,
                || {
                    format!(
                        "tool call {}: a finished call has a status, an unfinished one none",
                        t.call_id
                    )
                },
            )?;
            for a in [&t.arguments, &t.result].into_iter().flatten() {
                a.validate()?;
            }
        }
        let mut models = BTreeSet::new();
        for u in &self.usage {
            ensure(!u.model.trim().is_empty(), c, || {
                "usage without a model".into()
            })?;
            ensure(models.insert(&u.model), c, || {
                format!("usage of {} listed twice", u.model)
            })?;
            ensure(
                u.cached_input_tokens.is_none_or(|k| k <= u.input_tokens),
                c,
                || "cached input tokens exceed input tokens".into(),
            )?;
        }
        if let Some(cost) = &self.cost {
            ensure(cost.amount.get() >= 0.0, c, || "negative cost".into())?;
            ensure(
                cost.currency.len() == 3 && cost.currency.bytes().all(|b| b.is_ascii_uppercase()),
                c,
                || format!("{:?} is not an ISO 4217 code", cost.currency),
            )?;
            ensure(
                (cost.basis == CostBasis::PriceList) == cost.price_list.is_some(),
                c,
                || "a price-list cost names its price list (and only then)".into(),
            )?;
        }
        let parts: BTreeSet<_> = self.partitions_touched.iter().collect();
        ensure(parts.len() == self.partitions_touched.len(), c, || {
            "a partition is listed twice".into()
        })?;
        if let Some(run) = &self.run {
            ensure(
                !run.variant.trim().is_empty() && !run.task_id.trim().is_empty(),
                c,
                || "a harness run names its variant and task".into(),
            )?;
        }
        if let Some(a) = &self.answer {
            a.validate()?;
            ensure(a.contract.as_deref() == Some("ResearchAnswer"), c, || {
                "answer is a ResearchAnswer artifact".into()
            })?;
        }
        if let Some(a) = &self.event_log {
            a.validate()?;
        }
        ensure(self.wall_time_s.get() >= 0.0, c, || {
            "negative wall time".into()
        })?;
        ensure(
            self.started_at.as_str() <= self.finished_at.as_str(),
            c,
            || "finished before it started".into(),
        )
    }
}

// ---------------------------------------------------------------- /api/v1 shapes (K1.7)
//
// Request and response bodies of `/api/v1` that used to be defined only in the server. They carry no
// `schema_version` (HTTP envelopes and tool arguments do not); their meaning is fixed by the route.

/// `POST /api/v1/episodes`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateEpisode {
    pub goal: String,
    /// Idempotency key: retrying the same request returns the same episode.
    pub client_request_id: ClientRequestId,
    /// Model releases to pin (verified against the `ModelService`). The corpus snapshot is pinned by
    /// the host from the caller's scope, never taken from the client.
    #[serde(default)]
    pub model_releases: Vec<ReleaseId>,
}

/// `POST /api/v1/episodes/{id}/messages`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PostMessage {
    pub text: String,
    pub client_request_id: ClientRequestId,
}

/// Answer of `POST /api/v1/episodes/{id}/messages` (202).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MessageAccepted {
    pub episode_id: EpisodeId,
    pub event_id: EventId,
    pub seq: u64,
    /// The key was seen before; nothing new was appended.
    pub duplicate: bool,
}

/// A host decision on an approval the runtime requested: the subset of [`ApprovalDecision`] a
/// person may send (`expired` and `cancelled` are the host's own).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HostDecision {
    Approved,
    Denied,
}

impl From<HostDecision> for ApprovalDecision {
    fn from(d: HostDecision) -> Self {
        match d {
            HostDecision::Approved => ApprovalDecision::Approved,
            HostDecision::Denied => ApprovalDecision::Denied,
        }
    }
}

/// `POST /api/v1/episodes/{id}/approvals/{approval_id}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolveApproval {
    pub decision: HostDecision,
}

/// Answer of `POST /api/v1/episodes/{id}/approvals/{approval_id}` (202).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApprovalAccepted {
    pub episode_id: EpisodeId,
    pub approval_id: ApprovalId,
    pub decision: HostDecision,
}

/// `POST /api/v1/episodes/{id}/{action}`: the status changes a person may request. The runtime
/// bridge drives the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeAction {
    Pause,
    Resume,
    Cancel,
}

/// Whether a service is configured in this deployment, and why not when it is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ServiceStatus {
    pub available: bool,
    pub detail: String,
}

/// A catalog tool and whether this deployment serves it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolStatus {
    pub name: String,
    pub effect: ToolEffect,
    pub available: bool,
    /// Why it is not served; present exactly when `available` is false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Tool and sandbox profile of the agent runtime. Closed: a wider profile (shell, web search) needs
/// demonstrated isolation first (master design §6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProfile {
    /// No shell, no web search, no browser/computer use, read-only sandbox; approvals go to the host.
    Public,
}

/// How the agent runtime authenticates (never the credential itself).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCredentials {
    /// A key or token forwarded from the host's environment.
    Environment,
    /// A signed-in plan account in a shared login home.
    SharedLoginHome,
    /// Nothing configured yet: turns fail until a credential is set.
    None,
}

impl RuntimeCredentials {
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeCredentials::Environment => "environment",
            RuntimeCredentials::SharedLoginHome => "shared_login_home",
            RuntimeCredentials::None => "none",
        }
    }
}

impl fmt::Display for RuntimeCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Identity and configuration of the runtime build that hosts episodes (no secrets), produced by the
/// runtime preflight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuntimeBuildManifest {
    pub runtime_build_id: RuntimeBuildId,
    pub runtime: RuntimeKind,
    pub version: String,
    /// Hex sha256 of the runtime binary.
    pub binary_sha256: String,
    /// The binary hash was checked against a configured pin (otherwise only recorded).
    pub binary_hash_pinned: bool,
    /// Hex sha256 over the protocol schema bundle generated by this binary.
    pub protocol_schema_sha256: String,
    pub user_agent: String,
    pub transport: String,
    pub profile: RuntimeProfile,
    pub config_overrides: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Names (never values) of the environment variables forwarded to the runtime.
    pub forwarded_env: Vec<String>,
    pub credentials: RuntimeCredentials,
    pub preflight_at: Timestamp,
}

/// `GET /api/v1/capabilities`: what this deployment can serve and why not when it cannot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub agent_runtime: ServiceStatus,
    /// Identity of the pinned runtime build when the runtime is available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_build: Option<RuntimeBuildManifest>,
    pub knowledge: ServiceStatus,
    pub datasets: ServiceStatus,
    pub models: ServiceStatus,
    pub artifact_store: ServiceStatus,
    /// The IGNIS MCP endpoint the agent runtime reaches the tools through (loopback only).
    pub mcp: ServiceStatus,
    /// The isolated runner behind `analysis_execute`.
    pub analysis_runner: ServiceStatus,
    /// What programs run in, when the runner passed preflight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_environment: Option<RunnerEnvironment>,
    /// The tool catalog with what this deployment can actually serve.
    pub tools: Vec<ToolStatus>,
}

impl Capabilities {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "Capabilities";
        let mut names = BTreeSet::new();
        for t in &self.tools {
            ensure(names.insert(t.name.as_str()), c, || {
                format!("tool {} listed twice", t.name)
            })?;
            ensure(t.available != t.reason.is_some(), c, || {
                format!(
                    "tool {}: a reason is given exactly when it is unavailable",
                    t.name
                )
            })?;
        }
        Ok(())
    }
}

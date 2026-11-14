//! Research runtime contracts: episodes, ordered events, jobs, tool results, answers and the tool
//! catalog. Semantic owner: Mircea (Tudor reviews for the UI).
//!
//! Codex App Server owns the conversation and the model/tool loop. IGNIS keeps only what it needs to
//! map, replay and audit an episode: the thread mapping, its own ordered event log, tool/job receipts
//! and artifacts. Codex's native thread history is not copied into a second authoritative format.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::{
    ensure, schema_tag, AnswerId, ApprovalId, ArtifactId, ArtifactRef, Capability, ClaimId,
    ClientRequestId, ContractError, CorpusSnapshotId, EpisodeId, ErrorEnvelope, EventId,
    InlinePayload, JobId, MessageId, Origin, ReleaseId, RuntimeBuildId, ScopeId, Timestamp,
    ToolCallId, WorkspaceId,
};
use super::{EvidenceClass, EvidenceRef};

schema_tag!(
    /// Wire tag of episodes, events and answers.
    ResearchSchema = "research.v1"
);
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

/// Agent runtime hosting an episode. One value today; a second one needs an ADR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    CodexAppServer,
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

/// Versions an episode is pinned to; switching is an explicit, visible event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EpisodePins {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus_snapshot: Option<CorpusSnapshotId>,
    #[serde(default)]
    pub model_releases: Vec<ReleaseId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills_version: Option<String>,
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
    ToolStarted { call_id: ToolCallId, tool: String },
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
        ensure(self.seq >= 1, "ResearchEvent", || "seq starts at 1".into())?;
        match &self.body {
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
        t("sources_search", "Hybrid search over the pinned corpus snapshot; returns a page of evidence hits, not documents.", E::Read, true, &[C::ReadSources], "SearchRequest", "SearchPage", false, Lane::Raul),
        t("sources_read", "Read a page range, passage or table of one source version; says how to continue.", E::Read, true, &[C::ReadSources], "ReadRequest", "ReadResult", false, Lane::Raul),
        t("datasets_inspect", "Describe a dataset bundle: views, columns, units, time origin, partitions.", E::Read, true, &[C::ReadData], "DatasetRef", "DatasetBundle", false, Lane::Raul),
        t("models_inspect", "A model release with its construction manifest, domain and state.", E::Read, true, &[C::RunModels], "ReleaseRef", "ModelRelease", false, Lane::Iancu),
        t("models_forecast", "Strict forward of a pinned release on an admitted prefix.", E::Compute, true, &[C::RunModels, C::ReadData], "ForecastRequest", "AnalysisArtifact", true, Lane::Iancu),
        t("artifacts_read", "Read a byte range or JSON Pointer of a stored artifact.", E::Read, true, &[], "ArtifactReadRequest", "InlinePayload", false, Lane::Mircea),
        t("jobs_get", "State of a job this workspace submitted (e.g. by models_forecast): status, progress, output artifacts or error.", E::Read, true, &[], "JobLookup", "JobRef", false, Lane::Mircea),
        t("wiki_patch", "Write a wiki revision against a base revision (compare-and-swap).", E::WorkspaceWrite, false, &[C::WikiWrite], "WikiPatch", "WikiRevision", false, Lane::Raul),
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
        evidence_id: super::EvidenceId,
    },
    /// A value inside an artifact, addressed by an RFC 6901 JSON Pointer, so numbers in the text are
    /// bound to the computed field instead of being transcribed.
    ArtifactValue {
        artifact_id: ArtifactId,
        pointer: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub claim_id: ClaimId,
    pub kind: ClaimKind,
    pub text: String,
    #[serde(default)]
    pub support: Vec<ClaimSupport>,
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

impl ResearchAnswer {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ResearchAnswer";
        let evidence: BTreeMap<_, _> = self.evidence.iter().map(|e| (&e.evidence_id, e)).collect();
        let artifacts: BTreeMap<_, _> =
            self.artifacts.iter().map(|a| (&a.artifact_id, a)).collect();
        ensure(evidence.len() == self.evidence.len(), c, || {
            "duplicate evidence_id".into()
        })?;
        ensure(artifacts.len() == self.artifacts.len(), c, || {
            "duplicate artifact_id".into()
        })?;
        self.evidence.iter().try_for_each(EvidenceRef::validate)?;
        self.artifacts.iter().try_for_each(ArtifactRef::validate)?;
        let mut claim_ids = BTreeSet::new();
        for claim in &self.claims {
            ensure(claim_ids.insert(&claim.claim_id), c, || {
                format!("duplicate claim {}", claim.claim_id)
            })?;
            let mut has_evidence = false;
            let mut has_value = false;
            let mut has_prediction = false;
            for s in &claim.support {
                match s {
                    ClaimSupport::Evidence { evidence_id } => {
                        let Some(e) = evidence.get(evidence_id) else {
                            return Err(ContractError::new(
                                c,
                                format!(
                                    "claim {} cites unknown evidence {evidence_id}",
                                    claim.claim_id
                                ),
                            ));
                        };
                        if claim.kind == ClaimKind::Observed {
                            ensure(
                                matches!(
                                    e.evidence_class,
                                    EvidenceClass::OriginalObservation
                                        | EvidenceClass::SourceDerived
                                ),
                                c,
                                || {
                                    format!(
                                        "observed claim {} cites {:?} evidence",
                                        claim.claim_id, e.evidence_class
                                    )
                                },
                            )?;
                        }
                        has_evidence = true;
                    }
                    ClaimSupport::ArtifactValue {
                        artifact_id,
                        pointer,
                    } => {
                        let Some(a) = artifacts.get(artifact_id) else {
                            return Err(ContractError::new(
                                c,
                                format!(
                                    "claim {} cites unknown artifact {artifact_id}",
                                    claim.claim_id
                                ),
                            ));
                        };
                        ensure(pointer.is_empty() || pointer.starts_with('/'), c, || {
                            format!("{pointer:?} is not a JSON Pointer")
                        })?;
                        has_value = true;
                        has_prediction |= a.contract.as_deref() == Some("AnalysisArtifact");
                    }
                }
            }
            let ok = match claim.kind {
                ClaimKind::SourceReports => has_evidence,
                ClaimKind::Observed => has_evidence && !has_value,
                ClaimKind::Computed => has_value,
                ClaimKind::Predicted => has_prediction,
                ClaimKind::Interpretation => true,
            };
            ensure(ok, c, || {
                format!(
                    "claim {} ({:?}) lacks the support its kind requires",
                    claim.claim_id, claim.kind
                )
            })?;
        }
        for b in &self.blocks {
            match b {
                AnswerBlock::Text { claim_ids: ids, .. } => {
                    for id in ids {
                        ensure(claim_ids.contains(id), c, || {
                            format!("block references unknown claim {id}")
                        })?;
                    }
                }
                AnswerBlock::Artifact { artifact_id, .. } => {
                    ensure(artifacts.contains_key(artifact_id), c, || {
                        format!("block references unknown artifact {artifact_id}")
                    })?;
                }
            }
        }
        Ok(())
    }
}

//! Shared building blocks of every C0 contract family: typed ids, digests, timestamps, finite
//! numbers, payload origin, the host-derived execution scope, artifact references and the error
//! envelope. Semantic owner: shared (changes need a reviewer from every consuming lane).

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use super::Partition;

// ---------------------------------------------------------------- errors of contract validation

/// A payload that parses but violates a documented invariant.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{contract}: {reason}")]
pub struct ContractError {
    pub contract: &'static str,
    pub reason: String,
}

impl ContractError {
    pub fn new(contract: &'static str, reason: impl Into<String>) -> Self {
        ContractError {
            contract,
            reason: reason.into(),
        }
    }
}

/// `Err(ContractError)` when `cond` is false.
pub(crate) fn ensure(
    cond: bool,
    contract: &'static str,
    reason: impl FnOnce() -> String,
) -> Result<(), ContractError> {
    if cond {
        Ok(())
    } else {
        Err(ContractError::new(contract, reason()))
    }
}

// ---------------------------------------------------------------- schema family tags

/// A single-valued wire tag (`"research.v1"`, ...). A payload of another family or version fails to
/// parse, and the generated TypeScript type is the literal string.
macro_rules! schema_tag {
    ($(#[$m:meta])* $name:ident = $wire:literal) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
        pub enum $name {
            #[default]
            #[serde(rename = $wire)]
            V1,
        }
    };
}
pub(crate) use schema_tag;

// ---------------------------------------------------------------- opaque ids

pub(crate) fn check_id(s: &str) -> Result<(), &'static str> {
    if s.is_empty() {
        Err("id must not be empty")
    } else if s.chars().any(|c| c.is_whitespace() || c.is_control()) {
        Err("id must not contain whitespace or control characters")
    } else {
        Ok(())
    }
}

/// Opaque identifiers. Each kind is its own type, so a source id cannot be passed where a source
/// *version* or an artifact id is expected. On the wire they are plain non-empty strings without
/// whitespace; consumers must not parse them.
macro_rules! opaque_id {
    ($($(#[$m:meta])* $name:ident;)+) => {$(
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(s: impl Into<String>) -> Result<Self, ContractError> {
                let s = s.into();
                check_id(&s).map_err(|e| ContractError::new(stringify!($name), e))?;
                Ok($name(s))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let s = String::deserialize(d)?;
                check_id(&s).map_err(de::Error::custom)?;
                Ok($name(s))
            }
        }
        impl JsonSchema for $name {
            fn schema_name() -> Cow<'static, str> {
                stringify!($name).into()
            }
            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({"type": "string", "minLength": 1, "pattern": "^\\S+$"})
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    )+};
}
/// Family files declare their own ids with it (same wire rules); shared ids stay listed here.
pub(crate) use opaque_id;

opaque_id! {
    /// Workspace (tenant of episodes, artifacts and wiki).
    WorkspaceId;
    /// Authenticated user or service principal.
    PrincipalId;
    /// Host-derived execution scope.
    ScopeId;
    EpisodeId;
    EventId;
    MessageId;
    ApprovalId;
    /// One tool invocation (MCP call id or IGNIS call id).
    ToolCallId;
    JobId;
    ArtifactId;
    /// Idempotency key supplied by a client for requests that could create duplicates.
    ClientRequestId;
    /// Identity of the pinned agent-runtime build (`RuntimeBuildManifest`, Mircea's lane).
    RuntimeBuildId;
    /// A `Source.id`: one *version* of a source (bytes, listing or metadata record, see `IdBasis`).
    SourceId;
    /// A parse view of an asset (the text and layout whose offsets a locator counts).
    ParseViewId;
    EvidenceId;
    /// Immutable corpus snapshot (index generation + source set) used by search.
    CorpusSnapshotId;
    /// Opaque continuation token of a paged response.
    PageCursor;
    WikiPageId;
    LinkId;
    DatasetId;
    VisualBundleId;
    RunId;
    ObservationId;
    /// `ModelRelease.id`.
    ReleaseId;
    SpecId;
    PatchId;
    EvaluationId;
    AnswerId;
    ClaimId;
}

// ---------------------------------------------------------------- digests and timestamps

/// `sha256:<64 lowercase hex>`. Only content-derived identities use this type; a listing id or a
/// name is never a `Digest`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Digest(String);

impl Digest {
    pub fn parse(s: &str) -> Result<Self, ContractError> {
        let ok = s.strip_prefix("sha256:").is_some_and(|h| {
            h.len() == 64
                && h.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        });
        ensure(ok, "Digest", || {
            format!("{s:?} is not sha256:<64 lowercase hex>")
        })?;
        Ok(Digest(s.to_string()))
    }
    /// Digest of the canonical JSON (recursively sorted keys) of `v`.
    pub fn of_json<T: Serialize + ?Sized>(v: &T) -> Self {
        Digest(crate::hashing::content_hash(v, &[]))
    }
    /// Digest of the canonical JSON of `v` with the given top-level keys removed.
    pub fn of_json_excluding<T: Serialize + ?Sized>(v: &T, exclude: &[&str]) -> Self {
        Digest(crate::hashing::content_hash(v, exclude))
    }
    pub fn of_bytes(b: &[u8]) -> Self {
        Digest(format!("sha256:{}", crate::hashing::sha256_hex(b)))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// The bare hex digest.
    pub fn hex(&self) -> &str {
        &self.0["sha256:".len()..]
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Digest::parse(&String::deserialize(d)?).map_err(de::Error::custom)
    }
}

impl JsonSchema for Digest {
    fn schema_name() -> Cow<'static, str> {
        "Digest".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({"type": "string", "pattern": "^sha256:[0-9a-f]{64}$"})
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// System time: RFC 3339 in UTC with a `Z` suffix. Experimental times are never timestamps; they are
/// seconds since an explicit `time_origin`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Timestamp(String);

impl Timestamp {
    pub fn parse(s: &str) -> Result<Self, ContractError> {
        let ok = s.ends_with('Z') && chrono::DateTime::parse_from_rfc3339(s).is_ok();
        ensure(ok, "Timestamp", || {
            format!("{s:?} is not an RFC 3339 UTC timestamp ending in Z")
        })?;
        Ok(Timestamp(s.to_string()))
    }
    pub fn now() -> Self {
        Timestamp(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Timestamp::parse(&String::deserialize(d)?).map_err(de::Error::custom)
    }
}

impl JsonSchema for Timestamp {
    fn schema_name() -> Cow<'static, str> {
        "Timestamp".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({"type": "string", "format": "date-time", "pattern": "Z$"})
    }
}

// ---------------------------------------------------------------- finite numbers

/// An `f64` that is finite on both sides of the wire. serde_json writes NaN/Infinity as `null`,
/// which a reader would take for "missing"; this type refuses to serialize them instead, so a
/// non-finite value surfaces as an error at the producer. Missing values are `Option<Finite>`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Finite(f64);

impl Finite {
    pub fn new(v: f64) -> Result<Self, ContractError> {
        ensure(v.is_finite(), "Finite", || format!("{v} is not finite"))?;
        Ok(Finite(v))
    }
    pub fn get(self) -> f64 {
        self.0
    }
}

impl Serialize for Finite {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if !self.0.is_finite() {
            return Err(serde::ser::Error::custom(format!(
                "non-finite number {}",
                self.0
            )));
        }
        s.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for Finite {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Finite::new(f64::deserialize(d)?).map_err(de::Error::custom)
    }
}

impl JsonSchema for Finite {
    fn inline_schema() -> bool {
        true
    }
    fn schema_name() -> Cow<'static, str> {
        "Finite".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({"type": "number"})
    }
}

// ---------------------------------------------------------------- origin

/// Who produced a payload. `synthetic_fixture` marks hand-made contract examples; such payloads (and
/// anything computed from them) can never be presented as NASA results or qualify a model.
/// Produced-by-the-system payloads are `system` even when their *data* is synthetic (that is
/// `EvidenceClass::Synthetic` / `TrainedOn::Synthetic`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    System,
    SyntheticFixture,
}

impl Origin {
    /// Origin of a result computed from several inputs: a fixture anywhere taints the result.
    pub fn combine(inputs: impl IntoIterator<Item = Origin>) -> Origin {
        if inputs.into_iter().any(|o| o == Origin::SyntheticFixture) {
            Origin::SyntheticFixture
        } else {
            Origin::System
        }
    }
}

// ---------------------------------------------------------------- execution scope

/// What a scope is for. Only `evaluation` may read the final-test partition; `authoring` (WSP
/// weight authoring) may not execute code, so no fitter can hide behind a shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScopeRole {
    Research,
    Authoring,
    Evaluation,
    Ingestion,
}

/// Actions a scope may perform. Tools declare the capabilities they need (`ToolDescriptor`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    ReadSources,
    ReadData,
    RunModels,
    ExecuteCode,
    NetworkFetch,
    WorkspaceWrite,
    WikiWrite,
    ExternalWrite,
}

/// The data and actions a request may touch. Derived by the host from authentication and episode
/// membership and passed to services as a separate argument: it is never part of a client request
/// body, a tool argument or a prompt, so a model or a document cannot widen it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutionScope {
    pub scope_id: ScopeId,
    pub workspace_id: WorkspaceId,
    pub principal: PrincipalId,
    pub role: ScopeRole,
    /// Split partitions whose *outcomes* may be read. Absent partitions are refused.
    pub partitions: BTreeSet<Partition>,
    pub capabilities: BTreeSet<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus_snapshot: Option<CorpusSnapshotId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_scope_id: Option<ScopeId>,
}

/// A refused access, mapped to `ErrorCode::ScopeDenied`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("scope {scope}: {reason}")]
pub struct ScopeDenied {
    pub scope: ScopeId,
    pub reason: String,
}

impl From<ScopeDenied> for ErrorEnvelope {
    fn from(d: ScopeDenied) -> Self {
        ErrorEnvelope::new(ErrorCode::ScopeDenied, d.to_string(), RetryPolicy::Never)
    }
}

impl ExecutionScope {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(
            !self.partitions.contains(&Partition::Test) || self.role == ScopeRole::Evaluation,
            "ExecutionScope",
            || "only an evaluation scope may read the test partition".into(),
        )?;
        ensure(
            !(self.role == ScopeRole::Authoring
                && self.capabilities.contains(&Capability::ExecuteCode)),
            "ExecutionScope",
            || "an authoring scope cannot execute code".into(),
        )
    }

    fn deny(&self, reason: String) -> ScopeDenied {
        ScopeDenied {
            scope: self.scope_id.clone(),
            reason,
        }
    }

    /// Refuses reading outcomes of `p` unless granted (and, for `test`, unless this is a valid
    /// evaluation scope: a malformed scope never grants the lockbox).
    pub fn require_partition(&self, p: Partition) -> Result<(), ScopeDenied> {
        if !self.partitions.contains(&p) {
            return Err(self.deny(format!("partition {p:?} is not granted")));
        }
        self.validate().map_err(|e| self.deny(e.reason))
    }

    pub fn require(&self, c: Capability) -> Result<(), ScopeDenied> {
        if self.capabilities.contains(&c) {
            Ok(())
        } else {
            Err(self.deny(format!("capability {c:?} is not granted")))
        }
    }

    /// A child scope (subagent, job) with at most the parent's partitions and capabilities, in the
    /// same workspace, principal, role and corpus snapshot.
    pub fn derive_child(
        &self,
        scope_id: ScopeId,
        partitions: BTreeSet<Partition>,
        capabilities: BTreeSet<Capability>,
    ) -> Result<ExecutionScope, ScopeDenied> {
        if !partitions.is_subset(&self.partitions) {
            return Err(self.deny("a child scope cannot add partitions".into()));
        }
        if !capabilities.is_subset(&self.capabilities) {
            return Err(self.deny("a child scope cannot add capabilities".into()));
        }
        Ok(ExecutionScope {
            scope_id,
            workspace_id: self.workspace_id.clone(),
            principal: self.principal.clone(),
            role: self.role,
            partitions,
            capabilities,
            corpus_snapshot: self.corpus_snapshot.clone(),
            parent_scope_id: Some(self.scope_id.clone()),
        })
    }
}

// ---------------------------------------------------------------- artifacts

/// Reference to immutable bytes in the artifact store. Never a filesystem path or a URL: a download
/// link is resolved through the access service, may expire, and does not change the artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRef {
    pub artifact_id: ArtifactId,
    pub sha256: Digest,
    /// IANA media type, e.g. `application/json`, `application/vnd.apache.parquet`, `image/png`.
    pub media_type: String,
    pub byte_length: u64,
    /// Exported contract type of the content when it is a JSON/JSONL contract document
    /// (`"AnalysisArtifact"`, `"ResearchAnswer"`, ...). Consumers dispatch on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<String>,
    #[serde(default, skip_serializing_if = "Producer::is_empty")]
    pub produced_by: Producer,
    /// Entities this artifact was computed from (lineage, not access grants).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<EntityRef>,
    pub origin: Origin,
    pub created_at: Timestamp,
}

impl ArtifactRef {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(
            self.media_type
                .split_once('/')
                .is_some_and(|(a, b)| !a.is_empty() && !b.is_empty()),
            "ArtifactRef",
            || format!("{:?} is not a media type", self.media_type),
        )
    }
}

/// What produced an artifact (all optional: an ingestion artifact has no episode).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<EpisodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<JobId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<ToolCallId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_id: Option<ReleaseId>,
}

impl Producer {
    pub fn is_empty(&self) -> bool {
        *self == Producer::default()
    }
}

/// A typed reference to any IGNIS entity, used for lineage and provenance links.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum EntityRef {
    Source(SourceId),
    Evidence(EvidenceId),
    Run(RunId),
    Observation(ObservationId),
    Dataset(DatasetId),
    Artifact(ArtifactId),
    Release(ReleaseId),
    WikiPage(WikiPageId),
    Episode(EpisodeId),
}

/// Inline content returned together with (or instead of) a full artifact. `partial = true` means
/// this is a preview: the complete content is `full`, and nobody may treat the preview as the result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InlinePayload {
    pub value: serde_json::Value,
    pub partial: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full: Option<ArtifactRef>,
}

impl InlinePayload {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(
            !self.partial || self.full.is_some(),
            "InlinePayload",
            || "a partial preview must reference the full artifact".into(),
        )?;
        self.full.as_ref().map_or(Ok(()), ArtifactRef::validate)
    }
}

/// `artifacts_read`: a byte range or a JSON Pointer into a stored artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReadRequest {
    pub artifact_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_length: Option<u64>,
    /// RFC 6901 pointer into a JSON artifact (`""` = whole document).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json_pointer: Option<String>,
}

// ---------------------------------------------------------------- errors

/// Technical failure classes. Scientific outcomes (insufficient evidence, out of domain) are *not*
/// errors; they are `ScientificStatus` values of a completed call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    ScopeDenied,
    NotFound,
    /// A referenced source/asset is not materialized (listing or metadata only).
    SourceMissing,
    Unsupported,
    /// A required service or backend is not available in this deployment.
    CapabilityUnavailable,
    Conflict,
    RateLimited,
    UpstreamUnavailable,
    Cancelled,
    Internal,
}

/// What a caller may do after an error. `reconcile` means the effect is uncertain: inspect state
/// before any retry, never re-run blindly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RetryPolicy {
    Never,
    AfterDelay { seconds: u32 },
    AfterUserAction,
    Reconcile,
}

/// The one error shape of services, tools, jobs and the HTTP API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, thiserror::Error)]
#[serde(deny_unknown_fields)]
#[error("{code:?}: {message}")]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    pub message: String,
    pub retry: RetryPolicy,
    /// Redacted log artifact, when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_ref: Option<ArtifactId>,
}

impl ErrorEnvelope {
    pub fn new(code: ErrorCode, message: impl Into<String>, retry: RetryPolicy) -> Self {
        ErrorEnvelope {
            code,
            message: message.into(),
            retry,
            log_ref: None,
        }
    }
}

impl From<ContractError> for ErrorEnvelope {
    fn from(e: ContractError) -> Self {
        ErrorEnvelope::new(ErrorCode::InvalidRequest, e.to_string(), RetryPolicy::Never)
    }
}

//! Independent evaluation: request and result. Semantic owner: Tudor (scientific review before the
//! protocol freeze; Iancu may review definitions without seeing final outcomes).
//!
//! Development feedback and the final test are the same contract with a different `purpose`. Only a
//! final-test result produced by the system (not a fixture) can qualify a release, and nothing
//! rewrites an old result: a changed protocol is a new protocol version.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::{
    ensure, schema_tag, ArtifactRef, ClientRequestId, ContractError, DatasetId, Digest,
    EvaluationId, Finite, Origin, ReleaseId, Timestamp,
};
use super::{Partition, Quantity};
use crate::analysis::ForecastSettings;

schema_tag!(
    /// Wire tag of evaluation requests and results.
    EvaluationSchema = "evaluation.v1"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationPurpose {
    /// Repeatable feedback on train/validation, shown to authors (W1).
    DevelopmentFeedback,
    /// The lockbox run on the frozen protocol. Results are not shown to authors as feedback.
    FinalTest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProtocolRef {
    pub protocol_id: String,
    pub version: String,
    /// Digest of the frozen protocol (targets, groups, metrics, cutoffs). Required for a final test.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_hash: Option<Digest>,
}

/// A model under evaluation: release, forward code identity and the settings it runs with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelUnderTest {
    pub release_id: ReleaseId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_hash: Option<Digest>,
    #[serde(default)]
    pub settings: ForecastSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MetricSpec {
    /// Scorer id in `ignis-lab/src/eval` (e.g. `crps`, `pinball`, `interval_coverage`, `brier_ipcw`).
    pub metric: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvaluationRequest {
    pub schema_version: EvaluationSchema,
    pub request_id: EvaluationId,
    pub purpose: EvaluationPurpose,
    pub protocol: ProtocolRef,
    pub target: Quantity,
    /// SI unit of the target; must equal `target`'s.
    pub unit: String,
    pub dataset_id: DatasetId,
    /// `DatasetBundle::content_digest` of the bundle evaluated.
    pub dataset_hash: Digest,
    pub split_id: String,
    pub partition: Partition,
    pub candidate: ModelUnderTest,
    #[serde(default)]
    pub baselines: Vec<ModelUnderTest>,
    /// Prefix cutoffs in seconds since the dataset's time origin, strictly increasing.
    pub cutoffs_s: Vec<Finite>,
    pub metrics: Vec<MetricSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<ClientRequestId>,
}

impl EvaluationRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "EvaluationRequest";
        ensure(self.unit == self.target.unit_si(), c, || {
            format!("{:?} is scored in {}", self.target, self.target.unit_si())
        })?;
        match self.purpose {
            EvaluationPurpose::DevelopmentFeedback => {
                ensure(self.partition != Partition::Test, c, || {
                    "development feedback never uses the test partition".into()
                })?
            }
            EvaluationPurpose::FinalTest => {
                ensure(self.partition == Partition::Test, c, || {
                    "the final test runs on the test partition".into()
                })?;
                ensure(self.protocol.frozen_hash.is_some(), c, || {
                    "the final test needs a frozen protocol".into()
                })?;
            }
        }
        ensure(!self.metrics.is_empty(), c, || "no metrics".into())?;
        ensure(!self.cutoffs_s.is_empty(), c, || "no cutoffs".into())?;
        ensure(
            self.cutoffs_s.first().is_some_and(|t| t.get() >= 0.0)
                && self.cutoffs_s.windows(2).all(|w| w[0] < w[1]),
            c,
            || "cutoffs are non-negative and strictly increasing".into(),
        )?;
        self.candidate
            .settings
            .validate()
            .map_err(|r| ContractError::new(c, r.to_string()))
    }

    pub fn request_hash(&self) -> Digest {
        Digest::of_json(self)
    }
}

/// Every case of the protocol is counted: failures and refusals stay in the denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaseAccounting {
    pub total: u64,
    pub scored: u64,
    pub refused: u64,
    pub failed: u64,
}

/// Population a metric is computed over. Conditional quality (scored cases only) is always shown
/// next to coverage over all cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Population {
    ScoredCases,
    AllCases,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MetricValue {
    pub metric: String,
    pub version: String,
    pub subject: ReleaseId,
    /// Group key (e.g. `{"fuel": "heptane"}`); absent for the overall value.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub group: BTreeMap<String, String>,
    pub population: Population,
    /// Absent when undefined (no cases), never zero by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Finite>,
    pub unit: String,
    pub n: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseFailureKind {
    Refused,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaseFailure {
    pub case_id: String,
    pub subject: ReleaseId,
    pub kind: CaseFailureKind,
    pub reason: String,
}

/// What the result says about the candidate's qualification for the declared target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    NotAssessed,
    Inconclusive,
    NotQualified,
    /// Only from a system-produced final test on a frozen protocol.
    Qualified,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComputeUsage {
    pub wall_time_s: Finite,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_time_s: Option<Finite>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvaluationResult {
    pub schema_version: EvaluationSchema,
    pub result_id: EvaluationId,
    /// The request, embedded so a result is self-describing.
    pub request: EvaluationRequest,
    pub request_hash: Digest,
    pub cases: CaseAccounting,
    /// `scored / total` for the candidate (0 when there are no cases).
    pub coverage: Finite,
    pub metrics: Vec<MetricValue>,
    /// Per-case outputs and scores (Parquet/JSONL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_case: Option<ArtifactRef>,
    /// Candidate failures; when `per_case` is absent this list is complete.
    #[serde(default)]
    pub failures: Vec<CaseFailure>,
    pub verdict: Verdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compute: Option<ComputeUsage>,
    pub origin: Origin,
    pub created_at: Timestamp,
}

impl EvaluationResult {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "EvaluationResult";
        self.request.validate()?;
        ensure(self.request_hash == self.request.request_hash(), c, || {
            "request_hash does not match the embedded request".into()
        })?;
        let k = self.cases;
        ensure(k.total == k.scored + k.refused + k.failed, c, || {
            "total = scored + refused + failed".into()
        })?;
        let expected = if k.total == 0 {
            0.0
        } else {
            k.scored as f64 / k.total as f64
        };
        ensure((self.coverage.get() - expected).abs() <= 1e-12, c, || {
            format!(
                "coverage {} != scored/total {expected}",
                self.coverage.get()
            )
        })?;
        let candidate = &self.request.candidate.release_id;
        let candidate_failures = self
            .failures
            .iter()
            .filter(|f| &f.subject == candidate)
            .count() as u64;
        if self.per_case.is_none() {
            ensure(candidate_failures == k.refused + k.failed, c, || {
                "without per_case, every refused/failed case is listed".into()
            })?;
        }
        for m in self.metrics.iter().filter(|m| &m.subject == candidate) {
            let cap = match m.population {
                Population::ScoredCases => k.scored,
                Population::AllCases => k.total,
            };
            ensure(m.n <= cap, c, || {
                format!(
                    "{} counts {} cases, more than its population",
                    m.metric, m.n
                )
            })?;
        }
        if self.verdict == Verdict::Qualified {
            ensure(
                self.request.purpose == EvaluationPurpose::FinalTest,
                c,
                || "only a final test qualifies".into(),
            )?;
            ensure(self.origin == Origin::System, c, || {
                "a fixture never qualifies a model".into()
            })?;
            ensure(k.scored > 0 && !self.metrics.is_empty(), c, || {
                "a qualification needs scored cases and metrics".into()
            })?;
        }
        self.per_case.as_ref().map_or(Ok(()), ArtifactRef::validate)
    }
}

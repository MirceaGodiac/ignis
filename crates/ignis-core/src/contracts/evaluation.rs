//! Independent evaluation: gold tasks, requests and results. Semantic owner: Tudor (scientific review
//! before the protocol freeze; Iancu may review definitions without seeing final outcomes).
//!
//! Development feedback and the final test are the same contract with a different `purpose`. Only a
//! final-test result produced by the system (not a fixture) can qualify a release, and nothing
//! rewrites an old result: a changed protocol is a new protocol version.
//!
//! Wave K1 (`evaluation.v2`) adds the benchmark: `GoldTask` / `GoldSet` (K1.14) and the harness
//! comparison (K1.15), as a third `purpose` of the same request and result. A harness benchmark
//! compares harness variants on gold tasks, so the model-only request fields are absent there and
//! the comparison lives in its own section of the result; `validate()` admits exactly one shape per
//! purpose. The private hold-out is never given to a builder: the runner-facing form of a task
//! (`GoldPrompt`) carries no answer key, and a hold-out set has no builder form at all.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de, Deserialize, Deserializer, Serialize};

use super::common::{
    check_id, ensure, opaque_id, ArtifactRef, ClientRequestId, ContractError, DatasetId, Digest,
    EvaluationId, Finite, Origin, ReleaseId, RuntimeBuildId, SourceId, Timestamp,
};
use super::research::TraceId;
use super::{Partition, Quantity, SourceLocator};
use crate::analysis::ForecastSettings;

/// Wire tag of gold tasks, evaluation requests and results. Wave K1 writes `evaluation.v2`;
/// `evaluation.v1` (C0) documents are still read and keep their `request_hash`, because a document
/// is re-serialized with the tag it was read with and v1 documents never carry a v2 field. A v2
/// addition (a gold task or set, a harness benchmark) is refused under the v1 tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub enum EvaluationSchema {
    /// C0. Still read; new documents are written as v2.
    #[serde(rename = "evaluation.v1")]
    V1,
    #[default]
    #[serde(rename = "evaluation.v2")]
    V2,
}

impl EvaluationSchema {
    fn require_v2(self, contract: &'static str, what: &str) -> Result<(), ContractError> {
        ensure(self == EvaluationSchema::V2, contract, || {
            format!("{what} needs schema_version evaluation.v2")
        })
    }
}

opaque_id! {
    /// One gold task. `HarnessRun.task_id` (research.v2) names a task by this id.
    GoldTaskId;
    /// One frozen set of gold tasks (a dev set or the private hold-out).
    GoldSetId;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationPurpose {
    /// Repeatable feedback on train/validation, shown to authors (W1).
    DevelopmentFeedback,
    /// The lockbox run on the frozen protocol. Results are not shown to authors as feedback.
    FinalTest,
    /// v2: harness variants compared on a gold set (H0/H1 and ablations). Not a model evaluation:
    /// it never qualifies a release.
    HarnessBenchmark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProtocolRef {
    pub protocol_id: String,
    pub version: String,
    /// Digest of the frozen protocol (targets, groups, metrics, cutoffs). Required for a final test
    /// and for anything that touches the hold-out.
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
    /// Scorer id in `ignis-lab/src/eval` (e.g. `crps`, `pinball`, `interval_coverage`, `brier_ipcw`;
    /// for a harness benchmark e.g. `accuracy`, `precision`, `citation_support`, `check_pass`).
    pub metric: String,
    pub version: String,
}

// ---------------------------------------------------------------- gold tasks (K1.14)

/// How a task is answered and scored (`LANES_V2.md` §6, T10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GoldTaskClass {
    /// Multiple choice with an "insufficient evidence" option (LitQA2 format); scored on accuracy
    /// and on precision, so abstaining counts.
    Documentary,
    /// A statement to support: scored on whether the cited locator supports it (ALCE/FACT style).
    Citation,
    /// A computation with an executable check (ScienceAgentBench/BixBench style).
    Analytical,
    /// Only through the frozen evaluator protocol; the task carries no answer of its own.
    Predictive,
}

/// What the task is about (master design §24.2), so a pilot covers every kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GoldTopic {
    DefinitionsProtocols,
    TablesFigures,
    Synthesis,
    Computation,
    ModelsDiagnostics,
    PersistentResearch,
}

/// Retrieval sub-set of the benchmark (decides R10 upgrades); absent for tasks outside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalKind {
    SingleHop,
    MultiHop,
    Figure,
}

/// Which half of the gold tasks: the dev set shared with the builders, or the private hold-out kept
/// in the evaluator vault, run once on a frozen protocol (LAB-Bench style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GoldSplit {
    Dev,
    Holdout,
}

/// A place in a source that supports an answer. Gold evidence is fixed by the task author, not
/// returned by an episode, so it names the source and locator directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GoldEvidence {
    pub source_id: SourceId,
    pub locator: SourceLocator,
}

/// One number an executable check must reproduce, in SI, with an explicit tolerance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectedValue {
    pub name: String,
    pub value_si: Finite,
    pub unit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abs_tol: Option<Finite>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rel_tol: Option<Finite>,
}

/// The answer key. Never part of what an agent under test receives (`GoldPrompt`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnswerKey {
    Documentary {
        /// The choices shown, without the "insufficient evidence" option, which is always added.
        choices: Vec<String>,
        /// 0-based index of the right choice; absent when "insufficient evidence" is the right answer.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        correct: Option<u32>,
        /// Where the right choice is supported (required when one is right).
        #[serde(default)]
        support: Vec<GoldEvidence>,
    },
    Citation {
        statement: String,
        /// Locators that support the statement; a cited locator is scored against these.
        accepted: Vec<GoldEvidence>,
    },
    Analytical {
        /// The check program (run by the evaluator in the isolated runner).
        check: ArtifactRef,
        #[serde(default)]
        inputs: Vec<ArtifactRef>,
        expected: Vec<ExpectedValue>,
    },
    Predictive {
        /// The frozen evaluator protocol that scores it (targets, groups, cutoffs).
        protocol: ProtocolRef,
        target: Quantity,
    },
}

impl AnswerKey {
    fn class(&self) -> GoldTaskClass {
        match self {
            AnswerKey::Documentary { .. } => GoldTaskClass::Documentary,
            AnswerKey::Citation { .. } => GoldTaskClass::Citation,
            AnswerKey::Analytical { .. } => GoldTaskClass::Analytical,
            AnswerKey::Predictive { .. } => GoldTaskClass::Predictive,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GoldTask {
    pub schema_version: EvaluationSchema,
    pub task_id: GoldTaskId,
    pub class: GoldTaskClass,
    pub topic: GoldTopic,
    pub split: GoldSplit,
    pub question: String,
    /// Investigations the task is about (PSI ids); at least one.
    pub investigations: Vec<String>,
    /// Cluster for paired statistics (cluster bootstrap): tasks that share a study or run group
    /// share a cluster, because they are not independent.
    pub cluster: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval: Option<RetrievalKind>,
    pub answer: AnswerKey,
    pub authored_by: String,
    /// Independent reviewers of the key (e.g. citations reviewed by the knowledge lane).
    #[serde(default)]
    pub reviewed_by: Vec<String>,
    pub origin: Origin,
    pub created_at: Timestamp,
}

fn nonempty(s: &str) -> bool {
    !s.trim().is_empty()
}

impl GoldTask {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "GoldTask";
        self.schema_version.require_v2(c, "a gold task")?;
        ensure(nonempty(&self.question), c, || "empty question".into())?;
        ensure(nonempty(&self.cluster), c, || "empty cluster".into())?;
        ensure(nonempty(&self.authored_by), c, || "no author".into())?;
        ensure(
            !self.investigations.is_empty()
                && self.investigations.iter().all(|i| check_id(i).is_ok()),
            c,
            || "a task names at least one investigation id".into(),
        )?;
        ensure(self.answer.class() == self.class, c, || {
            format!(
                "a {:?} task needs a {:?} answer key",
                self.class, self.class
            )
        })?;
        let locators = |ev: &[GoldEvidence]| -> Result<(), ContractError> {
            ev.iter().try_for_each(|e| e.locator.validate())
        };
        match &self.answer {
            AnswerKey::Documentary {
                choices,
                correct,
                support,
            } => {
                ensure(
                    choices.len() >= 2 && choices.iter().all(|x| nonempty(x)),
                    c,
                    || "at least two non-empty choices".into(),
                )?;
                let unique: BTreeSet<&str> = choices.iter().map(|x| x.trim()).collect();
                ensure(unique.len() == choices.len(), c, || {
                    "duplicate choices".into()
                })?;
                if let Some(i) = correct {
                    ensure((*i as usize) < choices.len(), c, || {
                        "correct index outside the choices".into()
                    })?;
                    ensure(!support.is_empty(), c, || {
                        "a right choice needs supporting evidence".into()
                    })?;
                }
                locators(support)?;
            }
            AnswerKey::Citation {
                statement,
                accepted,
            } => {
                ensure(nonempty(statement), c, || "empty statement".into())?;
                ensure(!accepted.is_empty(), c, || {
                    "a citation task needs accepted locators".into()
                })?;
                locators(accepted)?;
            }
            AnswerKey::Analytical {
                check,
                inputs,
                expected,
            } => {
                check.validate()?;
                inputs.iter().try_for_each(ArtifactRef::validate)?;
                ensure(!expected.is_empty(), c, || {
                    "an analytical task needs expected values".into()
                })?;
                for e in expected {
                    ensure(nonempty(&e.name) && nonempty(&e.unit), c, || {
                        "an expected value needs a name and a unit".into()
                    })?;
                    ensure(e.abs_tol.is_some() || e.rel_tol.is_some(), c, || {
                        format!("{}: no tolerance", e.name)
                    })?;
                    ensure(
                        [e.abs_tol, e.rel_tol]
                            .iter()
                            .flatten()
                            .all(|t| t.get() >= 0.0),
                        c,
                        || format!("{}: negative tolerance", e.name),
                    )?;
                }
            }
            AnswerKey::Predictive { protocol, .. } => {
                ensure(protocol.frozen_hash.is_some(), c, || {
                    "a predictive task is scored only through a frozen protocol".into()
                })?;
            }
        }
        Ok(())
    }

    /// What the agent under test receives: everything it needs to do the task, nothing that grades
    /// it. Documentary: the choices plus the "insufficient evidence" option. Citation: the statement
    /// to support. Analytical: the input artifacts (the check program and expected values stay
    /// hidden). Predictive: the target quantity (the protocol scores it).
    pub fn prompt(&self) -> GoldPrompt {
        let mut p = GoldPrompt {
            task_id: self.task_id.clone(),
            class: self.class,
            question: self.question.clone(),
            choices: vec![],
            statement: None,
            inputs: vec![],
            target: None,
        };
        match &self.answer {
            AnswerKey::Documentary { choices, .. } => {
                p.choices = choices
                    .iter()
                    .cloned()
                    .chain([INSUFFICIENT.to_string()])
                    .collect()
            }
            AnswerKey::Citation { statement, .. } => p.statement = Some(statement.clone()),
            AnswerKey::Analytical { inputs, .. } => p.inputs = inputs.clone(),
            AnswerKey::Predictive { target, .. } => p.target = Some(*target),
        }
        p
    }
}

/// The option every documentary task offers, last.
pub const INSUFFICIENT: &str = "Insufficient evidence to answer";

/// The runner-facing form of a gold task: the task material only; no answer key, no supporting
/// evidence, no check program, no expected values, no reviewers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GoldPrompt {
    pub task_id: GoldTaskId,
    pub class: GoldTaskClass,
    pub question: String,
    /// Documentary: the choices, the last one "insufficient evidence".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<String>,
    /// Citation: the statement to support with a cited locator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statement: Option<String>,
    /// Analytical: the artifacts to compute on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<ArtifactRef>,
    /// Predictive: the quantity to forecast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<Quantity>,
}

/// A frozen set of gold tasks. `set_hash` is the digest of `tasks` and is what a benchmark request
/// pins. A dev set goes to the builders (`benchmarks/`); a hold-out set stays in the evaluator vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GoldSet {
    pub schema_version: EvaluationSchema,
    pub set_id: GoldSetId,
    pub split: GoldSplit,
    pub protocol: ProtocolRef,
    pub tasks: Vec<GoldTask>,
    pub set_hash: Digest,
    pub origin: Origin,
    pub created_at: Timestamp,
}

impl GoldSet {
    pub fn tasks_hash(tasks: &[GoldTask]) -> Digest {
        Digest::of_json(tasks)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "GoldSet";
        self.schema_version.require_v2(c, "a gold set")?;
        ensure(!self.tasks.is_empty(), c, || "an empty set".into())?;
        let mut ids = BTreeSet::new();
        for t in &self.tasks {
            t.validate()?;
            ensure(ids.insert(&t.task_id), c, || {
                format!("task {} twice", t.task_id.as_str())
            })?;
            ensure(t.split == self.split, c, || {
                format!(
                    "task {} is {:?} in a {:?} set",
                    t.task_id.as_str(),
                    t.split,
                    self.split
                )
            })?;
        }
        if self.split == GoldSplit::Holdout {
            ensure(self.protocol.frozen_hash.is_some(), c, || {
                "the hold-out is frozen with its protocol".into()
            })?;
        }
        ensure(self.set_hash == Self::tasks_hash(&self.tasks), c, || {
            "set_hash does not match the tasks".into()
        })
    }

    /// The prompts a builder's harness runner may receive. A hold-out set has no builder form.
    pub fn builder_prompts(&self) -> Result<GoldPromptSet, ContractError> {
        ensure(self.split == GoldSplit::Dev, "GoldSet", || {
            "the hold-out is never given to a builder".into()
        })?;
        Ok(GoldPromptSet {
            schema_version: EvaluationSchema::V2,
            set_id: self.set_id.clone(),
            set_hash: self.set_hash.clone(),
            prompts: self.tasks.iter().map(GoldTask::prompt).collect(),
            origin: self.origin,
        })
    }
}

/// The prompts of a dev set, as a builder's harness runner reads them (M10). Always a dev set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GoldPromptSet {
    pub schema_version: EvaluationSchema,
    pub set_id: GoldSetId,
    /// The `set_hash` of the dev set the prompts come from.
    pub set_hash: Digest,
    pub prompts: Vec<GoldPrompt>,
    pub origin: Origin,
}

impl GoldPromptSet {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "GoldPromptSet";
        self.schema_version.require_v2(c, "a prompt set")?;
        ensure(!self.prompts.is_empty(), c, || "no prompts".into())?;
        let ids: BTreeSet<&GoldTaskId> = self.prompts.iter().map(|p| &p.task_id).collect();
        ensure(ids.len() == self.prompts.len(), c, || "a task twice".into())
    }
}

// ---------------------------------------------------------------- harness benchmark request (K1.15)

/// The gold set a benchmark runs on, pinned by digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GoldSetRef {
    pub set_id: GoldSetId,
    pub set_hash: Digest,
    pub split: GoldSplit,
    pub task_count: u32,
}

/// One harness arm. Refers to the harness bundle by digest (research.v2 `HarnessRef`), never by
/// embedding it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HarnessVariant {
    /// Short name (`h0`, `h1`, `h1-no-wiki`); unique in the benchmark.
    pub name: String,
    pub harness_digest: Digest,
    pub runtime_build: RuntimeBuildId,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
}

/// How tasks are grouped for the cluster bootstrap of paired deltas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClusterBy {
    /// The task's `cluster` field (study or run group).
    TaskCluster,
    /// Each task its own cluster.
    Task,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HarnessBenchmark {
    pub gold_set: GoldSetRef,
    /// One arm measures a harness; two or more compare harnesses. All arms share the runtime
    /// build, model and effort, so only the harness differs.
    pub variants: Vec<HarnessVariant>,
    /// The arm deltas are measured against (H0).
    pub baseline: String,
    /// Runs per task and arm (seeds); deltas are paired on (task, repetition).
    pub repetitions: u32,
    pub cluster_by: ClusterBy,
}

impl HarnessBenchmark {
    fn validate(&self, c: &'static str) -> Result<(), ContractError> {
        let names: BTreeSet<&str> = self.variants.iter().map(|v| v.name.as_str()).collect();
        // One arm is a measurement of one harness (the T10 pilot, R10 retrieval numbers); a
        // comparison needs two or more, and deltas only exist then.
        ensure(
            !self.variants.is_empty() && names.len() == self.variants.len(),
            c,
            || "at least one arm, with unique names".into(),
        )?;
        ensure(
            self.variants
                .iter()
                .all(|v| nonempty(&v.name) && nonempty(&v.model)),
            c,
            || "an arm needs a name and a model".into(),
        )?;
        ensure(names.contains(self.baseline.as_str()), c, || {
            format!("baseline {} is not an arm", self.baseline)
        })?;
        let first = &self.variants[0];
        ensure(
            self.variants.iter().all(|v| {
                v.runtime_build == first.runtime_build
                    && v.model == first.model
                    && v.effort == first.effort
            }),
            c,
            || "arms differ only in the harness: same runtime build, model and effort".into(),
        )?;
        ensure(self.repetitions >= 1, c, || {
            "at least one repetition".into()
        })?;
        ensure(self.gold_set.task_count >= 1, c, || {
            "an empty gold set".into()
        })
    }

    /// Every (task, arm, repetition) run the benchmark must report.
    pub fn planned_runs(&self) -> u64 {
        u64::from(self.gold_set.task_count)
            * self.variants.len() as u64
            * u64::from(self.repetitions)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvaluationRequest {
    pub schema_version: EvaluationSchema,
    pub request_id: EvaluationId,
    pub purpose: EvaluationPurpose,
    pub protocol: ProtocolRef,
    /// Model purposes only (absent for a harness benchmark).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<Quantity>,
    /// SI unit of the target; must equal `target`'s. Model purposes only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<DatasetId>,
    /// `DatasetBundle::content_digest` of the bundle evaluated. Model purposes only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_hash: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub split_id: Option<String>,
    /// For a harness benchmark: `validation` on a dev set, `test` on the hold-out.
    pub partition: Partition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate: Option<ModelUnderTest>,
    #[serde(default)]
    pub baselines: Vec<ModelUnderTest>,
    /// Prefix cutoffs in seconds since the dataset's time origin, strictly increasing. Model
    /// purposes only (empty for a harness benchmark).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cutoffs_s: Vec<Finite>,
    pub metrics: Vec<MetricSpec>,
    /// v2, harness benchmark only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<HarnessBenchmark>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<ClientRequestId>,
}

/// The model-evaluation fields of a request, all present.
#[derive(Debug, Clone, Copy)]
pub struct ModelEvaluation<'a> {
    pub target: Quantity,
    pub unit: &'a str,
    pub dataset_id: &'a DatasetId,
    pub dataset_hash: &'a Digest,
    pub split_id: &'a str,
    pub candidate: &'a ModelUnderTest,
}

impl EvaluationRequest {
    /// The model view of a development-feedback or final-test request; an error for a harness
    /// benchmark or a request missing a model field.
    pub fn model(&self) -> Result<ModelEvaluation<'_>, ContractError> {
        let c = "EvaluationRequest";
        ensure(
            self.purpose != EvaluationPurpose::HarnessBenchmark,
            c,
            || "a harness benchmark has no model under test".into(),
        )?;
        let missing = |f: &str| ContractError::new(c, format!("a model evaluation needs {f}"));
        Ok(ModelEvaluation {
            target: self.target.ok_or_else(|| missing("target"))?,
            unit: self.unit.as_deref().ok_or_else(|| missing("unit"))?,
            dataset_id: self
                .dataset_id
                .as_ref()
                .ok_or_else(|| missing("dataset_id"))?,
            dataset_hash: self
                .dataset_hash
                .as_ref()
                .ok_or_else(|| missing("dataset_hash"))?,
            split_id: self
                .split_id
                .as_deref()
                .ok_or_else(|| missing("split_id"))?,
            candidate: self
                .candidate
                .as_ref()
                .ok_or_else(|| missing("candidate"))?,
        })
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "EvaluationRequest";
        ensure(!self.metrics.is_empty(), c, || "no metrics".into())?;
        if self.purpose == EvaluationPurpose::HarnessBenchmark {
            self.schema_version.require_v2(c, "a harness benchmark")?;
            let h = self.harness.as_ref().ok_or_else(|| {
                ContractError::new(c, "a harness benchmark needs its harness section")
            })?;
            ensure(
                self.target.is_none()
                    && self.unit.is_none()
                    && self.dataset_id.is_none()
                    && self.dataset_hash.is_none()
                    && self.split_id.is_none()
                    && self.candidate.is_none()
                    && self.baselines.is_empty()
                    && self.cutoffs_s.is_empty(),
                c,
                || "a harness benchmark carries no model-evaluation fields".into(),
            )?;
            h.validate(c)?;
            return match h.gold_set.split {
                GoldSplit::Dev => ensure(self.partition == Partition::Validation, c, || {
                    "a dev-set benchmark runs on the validation partition".into()
                }),
                GoldSplit::Holdout => {
                    ensure(self.partition == Partition::Test, c, || {
                        "the hold-out benchmark is the test partition".into()
                    })?;
                    ensure(self.protocol.frozen_hash.is_some(), c, || {
                        "the hold-out runs once, on a frozen protocol".into()
                    })
                }
            };
        }
        ensure(self.harness.is_none(), c, || {
            "only a harness benchmark has a harness section".into()
        })?;
        let m = self.model()?;
        ensure(m.unit == m.target.unit_si(), c, || {
            format!("{:?} is scored in {}", m.target, m.target.unit_si())
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
            EvaluationPurpose::HarnessBenchmark => unreachable!(),
        }
        ensure(!self.cutoffs_s.is_empty(), c, || "no cutoffs".into())?;
        ensure(
            self.cutoffs_s.first().is_some_and(|t| t.get() >= 0.0)
                && self.cutoffs_s.windows(2).all(|w| w[0] < w[1]),
            c,
            || "cutoffs are non-negative and strictly increasing".into(),
        )?;
        m.candidate
            .settings
            .validate()
            .map_err(|r| ContractError::new(c, r.to_string()))
    }

    pub fn request_hash(&self) -> Digest {
        Digest::of_json(self)
    }
}

// ---------------------------------------------------------------- results

/// Every case of the protocol is counted: failures and refusals stay in the denominator. For a
/// harness benchmark a case is one (task, arm, repetition) run.
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

// ---------------------------------------------------------------- harness comparison (K1.15)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome {
    /// The run finished and was scored.
    Scored,
    /// The runtime or host declined (scope, input, unsupported).
    Refused,
    /// A technical failure or a timeout: no answer to score.
    Failed,
}

/// One (task, arm, repetition) run. Its trace (research.v2 `TraceBundle`) holds the cost and
/// tokens; the run refers to it by id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HarnessRunResult {
    pub variant: String,
    pub task_id: GoldTaskId,
    pub repetition: u32,
    pub outcome: RunOutcome,
    /// Task score in [0, 1]; present exactly for a scored run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<Finite>,
    /// False when the agent chose "insufficient evidence" (precision counts only answered runs).
    pub answered: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
}

/// A two-sided interval at `level` (e.g. 0.95).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    pub lo: Finite,
    pub hi: Finite,
    pub level: Finite,
}

impl Interval {
    fn holds(&self, x: f64) -> bool {
        self.lo.get() <= x && x <= self.hi.get() && self.level.get() > 0.0 && self.level.get() < 1.0
    }
}

/// A money amount summed from traces. Absent when any run's cost is unknown: never a guess.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CostTotal {
    pub amount: Finite,
    /// ISO 4217 code, e.g. `USD`.
    pub currency: String,
}

/// One metric of one arm over its runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VariantScore {
    pub variant: String,
    pub metric: String,
    pub version: String,
    /// Absent when undefined (no runs it applies to).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Finite>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<Interval>,
    pub n: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<CostTotal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u64>,
}

/// `variant − baseline` on one metric, paired on (task, repetition), with a cluster-bootstrap
/// interval over `clusters` clusters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PairedDelta {
    pub metric: String,
    pub baseline: String,
    pub variant: String,
    pub delta: Finite,
    pub interval: Interval,
    pub pairs: u64,
    pub clusters: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HarnessComparison {
    /// Every planned run (task × arm × repetition), including refused and failed ones.
    pub runs: Vec<HarnessRunResult>,
    pub scores: Vec<VariantScore>,
    #[serde(default)]
    pub deltas: Vec<PairedDelta>,
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
    /// `scored / total` for the candidate, or over all runs of a harness benchmark (0 when there are
    /// no cases).
    pub coverage: Finite,
    /// Model metrics (subject = a release). Empty for a harness benchmark, whose metrics are in
    /// `harness`.
    pub metrics: Vec<MetricValue>,
    /// Per-case outputs and scores (Parquet/JSONL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_case: Option<ArtifactRef>,
    /// Candidate failures; when `per_case` is absent this list is complete. Empty for a harness
    /// benchmark, whose refused and failed runs are in `harness.runs`.
    #[serde(default)]
    pub failures: Vec<CaseFailure>,
    pub verdict: Verdict,
    /// v2, harness benchmark only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<HarnessComparison>,
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
        if self.request.purpose == EvaluationPurpose::HarnessBenchmark {
            self.schema_version.require_v2(c, "a harness comparison")?;
            ensure(self.verdict == Verdict::NotAssessed, c, || {
                "a harness benchmark never qualifies a release".into()
            })?;
            ensure(
                self.metrics.is_empty() && self.failures.is_empty(),
                c,
                || "harness metrics and failures live in the harness section".into(),
            )?;
            let h = self.harness.as_ref().ok_or_else(|| {
                ContractError::new(c, "a harness benchmark result needs its comparison")
            })?;
            let spec = self.request.harness.as_ref().expect("validated request");
            validate_comparison(c, h, spec, &self.request.metrics, k)?;
            ensure(spec.variants.len() >= 2 || h.deltas.is_empty(), c, || {
                "a one-arm measurement has no deltas".into()
            })?;
            return self.per_case.as_ref().map_or(Ok(()), ArtifactRef::validate);
        }
        ensure(self.harness.is_none(), c, || {
            "only a harness benchmark has a comparison".into()
        })?;
        let candidate = &self.request.model()?.candidate.release_id;
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

fn validate_comparison(
    c: &'static str,
    h: &HarnessComparison,
    spec: &HarnessBenchmark,
    metrics: &[MetricSpec],
    k: CaseAccounting,
) -> Result<(), ContractError> {
    let arms: BTreeSet<&str> = spec.variants.iter().map(|v| v.name.as_str()).collect();
    // Every planned run is reported once; the case accounting is the runs'.
    ensure(h.runs.len() as u64 == spec.planned_runs(), c, || {
        format!(
            "{} runs reported, {} planned",
            h.runs.len(),
            spec.planned_runs()
        )
    })?;
    let mut seen = BTreeSet::new();
    let mut tasks = BTreeSet::new();
    let (mut scored, mut refused, mut failed) = (0u64, 0u64, 0u64);
    for r in &h.runs {
        ensure(arms.contains(r.variant.as_str()), c, || {
            format!("run of unknown arm {}", r.variant)
        })?;
        ensure(r.repetition < spec.repetitions, c, || {
            format!("repetition {} out of range", r.repetition)
        })?;
        ensure(
            seen.insert((r.variant.as_str(), &r.task_id, r.repetition)),
            c,
            || "a run reported twice".into(),
        )?;
        tasks.insert(&r.task_id);
        match r.outcome {
            RunOutcome::Scored => {
                scored += 1;
                ensure(
                    r.score.is_some_and(|s| (0.0..=1.0).contains(&s.get())),
                    c,
                    || "a scored run has a score in [0, 1]".into(),
                )?;
            }
            RunOutcome::Refused | RunOutcome::Failed => {
                if r.outcome == RunOutcome::Refused {
                    refused += 1
                } else {
                    failed += 1
                }
                ensure(r.score.is_none(), c, || {
                    "an unscored run has no score".into()
                })?;
            }
        }
    }
    ensure(
        tasks.len() as u64 == u64::from(spec.gold_set.task_count),
        c,
        || "runs do not cover the gold set's tasks".into(),
    )?;
    ensure(
        (k.total, k.scored, k.refused, k.failed) == (h.runs.len() as u64, scored, refused, failed),
        c,
        || "case accounting does not match the runs".into(),
    )?;
    let declared: BTreeSet<(&str, &str)> = metrics
        .iter()
        .map(|m| (m.metric.as_str(), m.version.as_str()))
        .collect();
    for s in &h.scores {
        ensure(arms.contains(s.variant.as_str()), c, || {
            format!("score of unknown arm {}", s.variant)
        })?;
        ensure(
            declared.contains(&(s.metric.as_str(), s.version.as_str())),
            c,
            || format!("{} v{} was not requested", s.metric, s.version),
        )?;
        let runs_of_arm = h.runs.iter().filter(|r| r.variant == s.variant).count() as u64;
        ensure(s.n <= runs_of_arm, c, || {
            format!("{} counts more runs than its arm has", s.metric)
        })?;
        if let (Some(v), Some(i)) = (s.value, s.interval) {
            ensure(i.holds(v.get()), c, || {
                format!("{}: value outside its interval", s.metric)
            })?;
        }
        ensure(s.value.is_some() || s.interval.is_none(), c, || {
            "an interval without a value".into()
        })?;
        if let Some(cost) = &s.cost {
            ensure(
                cost.amount.get() >= 0.0 && cost.currency.len() == 3,
                c,
                || "a cost is a non-negative amount with an ISO 4217 code".into(),
            )?;
        }
    }
    for d in &h.deltas {
        ensure(d.baseline == spec.baseline, c, || {
            "deltas are measured against the declared baseline".into()
        })?;
        ensure(
            arms.contains(d.variant.as_str()) && d.variant != d.baseline,
            c,
            || format!("delta of unknown arm {}", d.variant),
        )?;
        ensure(declared.iter().any(|(m, _)| *m == d.metric), c, || {
            format!("{} was not requested", d.metric)
        })?;
        ensure(d.interval.holds(d.delta.get()), c, || {
            format!("{}: delta outside its interval", d.metric)
        })?;
        ensure(
            d.pairs >= 1 && d.clusters >= 1 && d.clusters <= d.pairs,
            c,
            || "1 ≤ clusters ≤ pairs".into(),
        )?;
    }
    Ok(())
}

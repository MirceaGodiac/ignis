//! Literal weight-space programming (WSP): model specification, handwritten weight patch, build
//! manifest, the release classification rules, and the forecast request. Semantic owner: Iancu
//! (Tudor reviews evaluability).
//!
//! Protocol (strict): the author model (Fable/Astra) writes the numbers. A patch contains only
//! literal assignments (an element, a box of elements with a literal list, or one literal constant
//! over a box). No expressions, code, randomness, seeds, solvers or `.npy` without a literal patch.
//! The assembler applies the assignments and serializes; it never searches or estimates. The
//! assembler/compiler itself is Iancu's lane (`ignis-wsp`); this module fixes what it consumes and
//! produces and the checks every release must pass.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::{
    ensure, schema_tag, ClientRequestId, ContractError, DatasetId, Digest, EvaluationId, Finite,
    Origin, PatchId, ReleaseId, RunId, SourceId, SpecId,
};
use super::{
    Family, Lineage, ModelRelease, Partition, QualificationState, Quantity, TensorEntry, TrainedOn,
};
use crate::analysis::ForecastSettings;

schema_tag!(
    /// Wire tag of model specs, patches, build manifests and forecast requests.
    WspSchema = "wsp.v1"
);

// ---------------------------------------------------------------- spec

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TargetSpec {
    pub quantity: Quantity,
    /// SI unit; must equal `quantity`'s.
    pub unit: String,
    pub description: String,
    /// What the model explicitly does not predict.
    #[serde(default)]
    pub excludes: Vec<String>,
}

/// One input feature, in model order. Order, unit and transform are part of the contract hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FeatureSpec {
    pub name: String,
    pub unit: String,
    /// Where the value comes from, e.g. `condition:pressure`, `observation:liquid_diameter@prefix`.
    pub source: String,
    /// Declared deterministic transform id and version (e.g. `log@1`, `identity@1`).
    pub transform: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DomainBound {
    pub feature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<Finite>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<Finite>,
}

/// Role of a tensor. Every scalar that influences a prediction is inventoried, whether or not it is
/// called a weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TensorRole {
    Weight,
    Bias,
    PhysicalParameter,
    NoiseParameter,
    HazardParameter,
    /// Fixed by the spec, not written by the author.
    Constant,
}

/// What an element holds when a root patch does not write it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "rule", rename_all = "snake_case", deny_unknown_fields)]
pub enum TensorFill {
    /// Every element must be written by the root patch; nothing is filled tacitly.
    AuthorWritesAll,
    /// Unwritten elements are zero by declaration of the spec (sparse weights).
    StructuralZero,
    /// Values fixed by the spec (row-major); the author cannot write them.
    Fixed { values: Vec<Finite> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValueBounds {
    pub min: Finite,
    pub max: Finite,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TensorSpec {
    pub name: String,
    /// Row-major shape; `[]` is a scalar.
    pub shape: Vec<u32>,
    pub role: TensorRole,
    pub fill: TensorFill,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Physical validity range; a patch outside it is rejected, not clipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<ValueBounds>,
    pub description: String,
}

impl TensorSpec {
    pub fn numel(&self) -> usize {
        self.shape.iter().map(|&d| d as usize).product()
    }
}

/// Inference profile of the forward pass. Only one exists: fixed weights, deterministic feature
/// extraction, explicit numerical integration; no gradient, MAP, NUTS, least squares or any
/// optimizer over parameters or local latents at request time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForwardProfile {
    StrictNoFitting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForwardContract {
    /// Code entry point, e.g. `ignis_wsp::flex::forward`.
    pub entry_point: String,
    pub version: String,
    pub profile: ForwardProfile,
}

/// Complete specification an author writes against: target, domain, features in order, every
/// tensor with its role and fill rule, and the forward contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelSpec {
    pub schema_version: WspSchema,
    pub spec_id: SpecId,
    pub family: Family,
    pub model_name: String,
    pub target: TargetSpec,
    pub features: Vec<FeatureSpec>,
    #[serde(default)]
    pub domain: Vec<DomainBound>,
    pub tensors: Vec<TensorSpec>,
    pub forward: ForwardContract,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl ModelSpec {
    /// The contract hash recorded by patches and manifests.
    pub fn spec_hash(&self) -> Digest {
        Digest::of_json(self)
    }

    pub fn tensor(&self, name: &str) -> Option<&TensorSpec> {
        self.tensors.iter().find(|t| t.name == name)
    }

    /// Spec ↔ release coherence: the release was built from this spec, keeps its feature order and
    /// units, and has exactly the spec's tensors with the spec's shapes. A forward or a handler can
    /// then rely on the spec instead of re-deriving matrix layouts.
    pub fn check_release(&self, release: &ModelRelease) -> Result<(), ContractError> {
        let c = "ModelSpec/ModelRelease";
        release.check_construction()?;
        let Some(m) = &release.construction else {
            return Err(ContractError::new(
                c,
                "release has no construction manifest",
            ));
        };
        ensure(
            m.spec_hash == self.spec_hash() && m.spec_id == self.spec_id,
            c,
            || "release was built from another spec".into(),
        )?;
        ensure(release.family == self.family, c, || {
            "family differs from the spec".into()
        })?;
        let names: Vec<&str> = self.features.iter().map(|f| f.name.as_str()).collect();
        ensure(
            release
                .feature_order
                .iter()
                .map(String::as_str)
                .eq(names.iter().copied()),
            c,
            || {
                format!(
                    "feature order {:?} != spec {names:?}",
                    release.feature_order
                )
            },
        )?;
        for f in &self.features {
            ensure(release.units.get(&f.name) == Some(&f.unit), c, || {
                format!("unit of {} differs from the spec", f.name)
            })?;
        }
        ensure(release.tensors.len() == self.tensors.len(), c, || {
            "tensor set differs from the spec".into()
        })?;
        for t in &self.tensors {
            let Some(e) = release.tensors.iter().find(|e| e.name == t.name) else {
                return Err(ContractError::new(
                    c,
                    format!("release lacks tensor {}", t.name),
                ));
            };
            ensure(
                e.shape
                    .iter()
                    .map(|&d| d as u64)
                    .eq(t.shape.iter().map(|&d| d as u64)),
                c,
                || format!("{}: shape {:?} != spec {:?}", t.name, e.shape, t.shape),
            )?;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ModelSpec";
        ensure(
            self.target.unit == self.target.quantity.unit_si(),
            c,
            || {
                format!(
                    "target {:?} is in {}",
                    self.target.quantity,
                    self.target.quantity.unit_si()
                )
            },
        )?;
        let mut features = BTreeSet::new();
        for f in &self.features {
            ensure(features.insert(f.name.as_str()), c, || {
                format!("duplicate feature {}", f.name)
            })?;
        }
        for d in &self.domain {
            ensure(features.contains(d.feature.as_str()), c, || {
                format!("domain bound on unknown feature {}", d.feature)
            })?;
            if let (Some(lo), Some(hi)) = (d.min, d.max) {
                ensure(lo < hi, c, || format!("domain of {} is empty", d.feature))?;
            }
        }
        let mut names = BTreeSet::new();
        for t in &self.tensors {
            ensure(names.insert(t.name.as_str()), c, || {
                format!("duplicate tensor {}", t.name)
            })?;
            ensure(t.shape.iter().all(|&d| d >= 1), c, || {
                format!("{}: zero-sized dimension", t.name)
            })?;
            if let Some(b) = &t.bounds {
                ensure(b.min < b.max, c, || format!("{}: empty bounds", t.name))?;
            }
            match &t.fill {
                TensorFill::Fixed { values } => {
                    ensure(values.len() == t.numel(), c, || {
                        format!(
                            "{}: {} fixed values for {} elements",
                            t.name,
                            values.len(),
                            t.numel()
                        )
                    })?;
                    if let Some(b) = &t.bounds {
                        ensure(values.iter().all(|v| b.min <= *v && *v <= b.max), c, || {
                            format!("{}: fixed value outside bounds", t.name)
                        })?;
                    }
                }
                TensorFill::StructuralZero => {
                    ensure(t.role != TensorRole::PhysicalParameter, c, || {
                        format!("{}: physical parameters are never filled tacitly", t.name)
                    })?;
                    if let Some(b) = &t.bounds {
                        ensure(
                            b.min <= Finite::new(0.0).unwrap()
                                && Finite::new(0.0).unwrap() <= b.max,
                            c,
                            || format!("{}: structural zero outside bounds", t.name),
                        )?;
                    }
                }
                TensorFill::AuthorWritesAll => {}
            }
            ensure(
                (t.role == TensorRole::Constant) == matches!(t.fill, TensorFill::Fixed { .. }),
                c,
                || {
                    format!(
                        "{}: constants (and only constants) have fixed values",
                        t.name
                    )
                },
            )?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- patch

/// Model that wrote a patch: provider and the exact model id actually used (not a marketing name).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorModel {
    pub provider: String,
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExposedDataset {
    pub dataset_id: DatasetId,
    pub partitions: BTreeSet<Partition>,
}

/// Everything the author saw. Development feedback and other candidates are listed, so W0 (no
/// feedback) and W1 (feedback) stay distinguishable. The test partition never appears here.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DataExposure {
    #[serde(default)]
    pub sources: Vec<SourceId>,
    #[serde(default)]
    pub datasets: Vec<ExposedDataset>,
    /// Development `EvaluationResult`s shown to the author.
    #[serde(default)]
    pub development_feedback: Vec<EvaluationId>,
    /// Other candidates (e.g. the other author's) shown to the author.
    #[serde(default)]
    pub prior_candidates: Vec<ReleaseId>,
}

impl DataExposure {
    pub fn validate(&self) -> Result<(), ContractError> {
        ensure(
            self.datasets
                .iter()
                .all(|d| !d.partitions.contains(&Partition::Test)),
            "DataExposure",
            || "an author never sees the test partition".into(),
        )
    }
    fn is_independent(&self) -> bool {
        self.development_feedback.is_empty() && self.prior_candidates.is_empty()
    }
}

/// One literal assignment. Boxes are `start` + `shape` per dimension (half-open), values row-major.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum LiteralOp {
    Set {
        tensor: String,
        index: Vec<u32>,
        value: Finite,
    },
    SetBox {
        tensor: String,
        start: Vec<u32>,
        shape: Vec<u32>,
        values: Vec<Finite>,
    },
    FillBox {
        tensor: String,
        start: Vec<u32>,
        shape: Vec<u32>,
        value: Finite,
    },
}

impl LiteralOp {
    pub fn tensor(&self) -> &str {
        match self {
            LiteralOp::Set { tensor, .. }
            | LiteralOp::SetBox { tensor, .. }
            | LiteralOp::FillBox { tensor, .. } => tensor,
        }
    }

    /// Row-major element offsets and their values, after bounds checks against `shape`.
    pub fn assignments(&self, shape: &[u32]) -> Result<Vec<(usize, Finite)>, String> {
        let flat = |idx: &[u32]| {
            idx.iter()
                .zip(shape)
                .fold(0usize, |acc, (&i, &d)| acc * d as usize + i as usize)
        };
        let boxed = |start: &[u32], bshape: &[u32]| -> Result<Vec<usize>, String> {
            if start.len() != shape.len() || bshape.len() != shape.len() {
                return Err(format!(
                    "box rank {} for tensor rank {}",
                    start.len().max(bshape.len()),
                    shape.len()
                ));
            }
            for ((&s, &n), &d) in start.iter().zip(bshape).zip(shape) {
                if n == 0 || s as u64 + n as u64 > d as u64 {
                    return Err(format!(
                        "box [{s}, {}) outside dimension {d} or empty",
                        s as u64 + n as u64
                    ));
                }
            }
            let mut out = Vec::with_capacity(bshape.iter().map(|&n| n as usize).product());
            let mut idx: Vec<u32> = start.to_vec();
            loop {
                out.push(flat(&idx));
                let mut k = idx.len();
                loop {
                    if k == 0 {
                        return Ok(out);
                    }
                    k -= 1;
                    idx[k] += 1;
                    if idx[k] < start[k] + bshape[k] {
                        break;
                    }
                    idx[k] = start[k];
                }
            }
        };
        match self {
            LiteralOp::Set { index, value, .. } => {
                if index.len() != shape.len() || index.iter().zip(shape).any(|(&i, &d)| i >= d) {
                    return Err(format!("index {index:?} outside shape {shape:?}"));
                }
                Ok(vec![(flat(index), *value)])
            }
            LiteralOp::SetBox {
                start,
                shape: b,
                values,
                ..
            } => {
                let cells = boxed(start, b)?;
                if cells.len() != values.len() {
                    return Err(format!(
                        "{} values for a box of {} elements",
                        values.len(),
                        cells.len()
                    ));
                }
                Ok(cells.into_iter().zip(values.iter().copied()).collect())
            }
            LiteralOp::FillBox {
                start,
                shape: b,
                value,
                ..
            } => Ok(boxed(start, b)?.into_iter().map(|i| (i, *value)).collect()),
        }
    }
}

/// The literal output of an author model for one spec: parent (for W1 revisions), author, exposure
/// and the assignments. Within one patch every element is written at most once; overwriting a
/// parent's value is what a revision does.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HandwrittenWeightPatch {
    pub schema_version: WspSchema,
    pub patch_id: PatchId,
    pub spec_hash: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_release_id: Option<ReleaseId>,
    pub author: AuthorModel,
    pub exposure: DataExposure,
    pub ops: Vec<LiteralOp>,
    /// Public explanation of the roles of neurons/blocks (not private reasoning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub origin: Origin,
}

impl HandwrittenWeightPatch {
    pub fn patch_hash(&self) -> Digest {
        Digest::of_json(self)
    }

    /// Checks the patch against its spec: matching spec hash, known writable tensors, in-bounds
    /// indices and boxes, values inside declared bounds, no element written twice, and, for a root
    /// patch, every `author_writes_all` tensor completely written. Any violation rejects the whole
    /// patch; nothing is skipped.
    pub fn validate_against(&self, spec: &ModelSpec) -> Result<(), ContractError> {
        let c = "HandwrittenWeightPatch";
        spec.validate()?;
        self.exposure.validate()?;
        ensure(self.spec_hash == spec.spec_hash(), c, || {
            format!(
                "written for {}, spec is {}",
                self.spec_hash,
                spec.spec_hash()
            )
        })?;
        ensure(!self.ops.is_empty(), c, || "empty patch".into())?;
        let mut written: BTreeMap<&str, Vec<bool>> = BTreeMap::new();
        for (i, op) in self.ops.iter().enumerate() {
            let Some(t) = spec.tensor(op.tensor()) else {
                return Err(ContractError::new(
                    c,
                    format!("op {i}: unknown tensor {}", op.tensor()),
                ));
            };
            ensure(!matches!(t.fill, TensorFill::Fixed { .. }), c, || {
                format!("op {i}: {} is fixed by the spec", t.name)
            })?;
            let cells = op
                .assignments(&t.shape)
                .map_err(|e| ContractError::new(c, format!("op {i} on {}: {e}", t.name)))?;
            let mask = written
                .entry(t.name.as_str())
                .or_insert_with(|| vec![false; t.numel()]);
            for (cell, v) in cells {
                ensure(!mask[cell], c, || {
                    format!("op {i}: element {cell} of {} written twice", t.name)
                })?;
                mask[cell] = true;
                if let Some(b) = &t.bounds {
                    ensure(b.min <= v && v <= b.max, c, || {
                        format!("op {i}: {} outside the bounds of {}", v.get(), t.name)
                    })?;
                }
            }
        }
        if self.parent_release_id.is_none() {
            for t in spec
                .tensors
                .iter()
                .filter(|t| t.fill == TensorFill::AuthorWritesAll)
            {
                let n = written
                    .get(t.name.as_str())
                    .map_or(0, |m| m.iter().filter(|w| **w).count());
                ensure(n == t.numel(), c, || {
                    format!(
                        "root patch writes {n} of {} elements of {}",
                        t.numel(),
                        t.name
                    )
                })?;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- build manifest and release rules

/// How a release's parameters came to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionMethod {
    /// W0: one literal patch from the spec and admitted knowledge, without evaluation feedback.
    LiteralOneShot,
    /// W1: a literal patch over a parent release, written with development feedback.
    LiteralRevision,
    /// Gradient/MCMC fitting (baselines and legacy experts). Never presented as WSP.
    ConventionalTraining,
}

impl ConstructionMethod {
    pub fn is_literal(self) -> bool {
        matches!(
            self,
            ConstructionMethod::LiteralOneShot | ConstructionMethod::LiteralRevision
        )
    }
}

/// Meaning of any uncertainty a release outputs. Literal releases cannot claim calibrated
/// predictive intervals: calibrating from data is a separate, separately reported step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyKind {
    None,
    AssumedNoise,
    ScenarioSpread,
    CalibratedPrediction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildCheckKind {
    SpecValid,
    PatchValid,
    TensorsComplete,
    /// The real forecast route (features, local state, noise/hazard, fallbacks) uses no optimizer.
    StrictForwardAudit,
    FiniteForwardOnFixture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Failed,
    NotRun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildCheck {
    pub check: BuildCheckKind,
    pub status: CheckStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// How a release was built. Embedded in `ModelRelease.construction`, so it is covered by the
/// release's content id and cannot drift from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelBuildManifest {
    pub schema_version: WspSchema,
    pub construction_method: ConstructionMethod,
    pub spec_id: SpecId,
    pub spec_hash: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_id: Option<PatchId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_hash: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_release_id: Option<ReleaseId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<AuthorModel>,
    pub exposure: DataExposure,
    /// [`weights_digest`] of the release's tensor table.
    pub weights_hash: Digest,
    pub forward: ForwardContract,
    /// Identity of the forward code (source/binary digest), distinct from the weights.
    pub forward_hash: Digest,
    pub code_version: String,
    pub uncertainty_kind: UncertaintyKind,
    #[serde(default)]
    pub checks: Vec<BuildCheck>,
    pub origin: Origin,
}

impl ModelBuildManifest {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ModelBuildManifest";
        self.exposure.validate()?;
        let m = self.construction_method;
        if m.is_literal() {
            ensure(
                self.patch_hash.is_some() && self.patch_id.is_some(),
                c,
                || "a literal release names its patch".into(),
            )?;
            ensure(self.author.is_some(), c, || {
                "a literal release names its author model".into()
            })?;
            ensure(
                self.forward.profile == ForwardProfile::StrictNoFitting,
                c,
                || "literal releases run the strict forward".into(),
            )?;
            ensure(
                self.uncertainty_kind != UncertaintyKind::CalibratedPrediction,
                c,
                || "literal values cannot claim calibrated predictive uncertainty".into(),
            )?;
        } else {
            ensure(
                self.patch_hash.is_none() && self.patch_id.is_none(),
                c,
                || "a trained release has no literal patch".into(),
            )?;
        }
        match m {
            ConstructionMethod::LiteralOneShot => {
                ensure(self.parent_release_id.is_none(), c, || {
                    "W0 has no parent".into()
                })?;
                ensure(self.exposure.is_independent(), c, || {
                    "W0 sees no evaluation feedback and no other candidate".into()
                })?;
            }
            ConstructionMethod::LiteralRevision => {
                ensure(self.parent_release_id.is_some(), c, || {
                    "W1 revises a parent release".into()
                })?
            }
            ConstructionMethod::ConventionalTraining => {}
        }
        ensure(
            self.checks.iter().all(|k| k.status != CheckStatus::Failed),
            c,
            || "a release cannot carry a failed build check".into(),
        )
    }
}

/// Digest of a tensor table (names, shapes, dtypes, value hashes), independent of order and of the
/// `trainable` flag.
pub fn weights_digest(tensors: &[TensorEntry]) -> Digest {
    let mut rows: Vec<_> = tensors
        .iter()
        .map(|t| (&t.name, &t.shape, &t.dtype, &t.sha256))
        .collect();
    rows.sort();
    Digest::of_json(&rows)
}

impl ModelRelease {
    /// Classification rules applied on every registry save and load:
    /// * `not_trained` requires a literal construction manifest, and a literal manifest requires
    ///   `not_trained` (no handwritten release can be labelled NASA-trained, and no trained release
    ///   can be labelled handwritten);
    /// * a literal release has no posterior, starts `experimental`, has lineage
    ///   `literal_authoring` (W0) or `weight_edit` (W1), and its manifest matches its tensors and
    ///   parent.
    pub fn check_construction(&self) -> Result<(), ContractError> {
        let c = "ModelRelease";
        let literal = self
            .construction
            .as_ref()
            .filter(|m| m.construction_method.is_literal());
        ensure(
            (self.trained_on == TrainedOn::NotTrained) == literal.is_some(),
            c,
            || {
                format!(
                    "trained_on {:?} contradicts construction {:?}",
                    self.trained_on,
                    self.construction.as_ref().map(|m| m.construction_method)
                )
            },
        )?;
        let Some(m) = &self.construction else {
            return Ok(());
        };
        m.validate()?;
        ensure(
            m.parent_release_id
                == self
                    .parent_id
                    .as_ref()
                    .map(|p| ReleaseId::new(p.clone()))
                    .transpose()?,
            c,
            || "manifest parent differs from parent_id".into(),
        )?;
        if literal.is_none() {
            return Ok(());
        }
        ensure(m.weights_hash == weights_digest(&self.tensors), c, || {
            "manifest weights_hash does not match the tensors".into()
        })?;
        ensure(
            self.posterior.is_empty() && self.posterior_parameters.is_empty(),
            c,
            || "a literal release has no posterior draws".into(),
        )?;
        ensure(self.state == QualificationState::Experimental, c, || {
            format!(
                "a literal release is created experimental, not {:?}",
                self.state
            )
        })?;
        let lineage_ok = match m.construction_method {
            ConstructionMethod::LiteralOneShot => self.lineage == Lineage::LiteralAuthoring,
            _ => self.lineage == Lineage::WeightEdit,
        };
        ensure(lineage_ok, c, || {
            format!(
                "lineage {:?} does not match {:?}",
                self.lineage, m.construction_method
            )
        })
    }
}

// ---------------------------------------------------------------- forecast request

/// Input of a forecast: a stored run cut at `cutoff_s` (the numeric service builds the
/// `PrefixBundle`), or an already admitted prefix by its `input_hash`. Never a prefix assembled by a
/// language model from text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ForecastInput {
    StoredRun { run_id: RunId, cutoff_s: Finite },
    Prefix { input_hash: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForecastRequest {
    pub schema_version: WspSchema,
    pub release_id: ReleaseId,
    pub input: ForecastInput,
    #[serde(default)]
    pub settings: ForecastSettings,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<ClientRequestId>,
}

impl ForecastRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        if let ForecastInput::StoredRun { cutoff_s, .. } = self.input {
            ensure(cutoff_s.get() >= 0.0, "ForecastRequest", || {
                "cutoff_s is non-negative".into()
            })?;
        }
        self.settings
            .validate()
            .map_err(|r| ContractError::new("ForecastRequest", r.to_string()))
    }
}

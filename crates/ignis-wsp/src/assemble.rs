//! The literal assembler: spec + patch (+ parent release) -> tensors -> immutable release.
//!
//! It applies assignments and serializes. It holds no optimizer, solver, sampler, seed or expression
//! evaluator, and it reads no dataset: the only numbers that reach a tensor are the spec's `fixed`
//! values, a parent release's stored values (W1) and the literals of the patch.
//!
//! Order of construction (handoff I2): zero-init -> `fixed` values of the spec -> parent tensors
//! (revision only) -> the patch's assignments. Any contract violation rejects the whole patch.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, ensure, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use ignis_core::contracts::{
    weights_digest, AuthorModel, BuildCheck, BuildCheckKind, CheckStatus, ConstructionMethod,
    DataExposure, Digest, HandwrittenWeightPatch, JsonMap, Lineage, LiteralOp, ModelBuildManifest,
    ModelRelease, ModelSpec, Origin, PatchId, QualificationState, ReleaseId, TensorFill, TrainedOn,
    UncertaintyKind, WspSchema,
};
use ignis_core::registry::{self, LoadedRelease, Tensor, TensorMap};

/// Provenance of an edits file: everything a [`HandwrittenWeightPatch`] carries besides the ops.
/// Kept next to the edits so the author's literal output stays a plain text artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchMeta {
    pub patch_id: PatchId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_release_id: Option<ReleaseId>,
    pub author: AuthorModel,
    pub exposure: DataExposure,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub origin: Origin,
}

/// Binds compiled ops and their provenance to a spec.
pub fn make_patch(
    spec: &ModelSpec,
    meta: PatchMeta,
    ops: Vec<LiteralOp>,
) -> HandwrittenWeightPatch {
    HandwrittenWeightPatch {
        schema_version: WspSchema::V1,
        patch_id: meta.patch_id,
        spec_hash: spec.spec_hash(),
        parent_release_id: meta.parent_release_id,
        author: meta.author,
        exposure: meta.exposure,
        ops,
        rationale: meta.rationale,
        origin: meta.origin,
    }
}

/// Tensors of a spec before any author assignment: zeros, and the spec's `fixed` values.
pub fn zero_init(spec: &ModelSpec) -> TensorMap {
    spec.tensors
        .iter()
        .map(|t| {
            let data = match &t.fill {
                TensorFill::Fixed { values } => values.iter().map(|v| v.get()).collect(),
                _ => vec![0.0; t.numel()],
            };
            let shape = t.shape.iter().map(|&d| d as usize).collect();
            (t.name.clone(), Tensor::new(shape, data))
        })
        .collect()
}

/// Spec + patch (+ parent) -> tensors. A root patch (W0) starts from [`zero_init`]; a revision (W1)
/// starts from its parent's tensors, which must have been built from the same spec.
pub fn materialize(
    spec: &ModelSpec,
    patch: &HandwrittenWeightPatch,
    parent: Option<&LoadedRelease>,
) -> anyhow::Result<TensorMap> {
    patch.validate_against(spec)?;
    let mut tensors = match (&patch.parent_release_id, parent) {
        (None, None) => zero_init(spec),
        (Some(id), Some(p)) => {
            ensure!(
                p.release.id == id.as_str(),
                "patch revises {id}, the loaded parent is {}",
                p.release.id
            );
            spec.check_release(&p.release)
                .context("a revision keeps its parent's spec; a new architecture is a new spec")?;
            let mut base = zero_init(spec);
            for (name, t) in base.iter_mut() {
                let stored = p
                    .params
                    .get(name)
                    .with_context(|| format!("parent lacks tensor {name}"))?;
                ensure!(
                    stored.shape == t.shape,
                    "parent tensor {name} has another shape"
                );
                if !matches!(
                    spec.tensor(name).map(|s| &s.fill),
                    Some(TensorFill::Fixed { .. })
                ) {
                    t.data.clone_from(&stored.data);
                }
            }
            base
        }
        (Some(id), None) => bail!("patch revises {id} but no parent release was loaded"),
        (None, Some(_)) => bail!("a root patch has no parent"),
    };
    for op in &patch.ops {
        let shape = &spec.tensor(op.tensor()).expect("validated").shape;
        let t = tensors.get_mut(op.tensor()).expect("validated");
        for (i, v) in op.assignments(shape).map_err(anyhow::Error::msg)? {
            t.data[i] = v.get();
        }
    }
    ensure!(
        tensors
            .values()
            .all(|t| t.data.iter().all(|v| v.is_finite())),
        "non-finite tensor value"
    );
    Ok(tensors)
}

/// Parameter counts of an assembled model, per tensor and in total.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Counts {
    /// Elements the author may write (everything except `fixed` constants).
    pub authored_total: usize,
    pub authored_nonzero: usize,
    /// Elements fixed by the spec.
    pub constant_total: usize,
    pub per_tensor: BTreeMap<String, (usize, usize)>,
}

pub fn counts(spec: &ModelSpec, tensors: &TensorMap) -> Counts {
    let mut c = Counts {
        authored_total: 0,
        authored_nonzero: 0,
        constant_total: 0,
        per_tensor: BTreeMap::new(),
    };
    for t in &spec.tensors {
        let data = &tensors[&t.name].data;
        let nz = data.iter().filter(|v| **v != 0.0).count();
        c.per_tensor.insert(t.name.clone(), (data.len(), nz));
        if matches!(t.fill, TensorFill::Fixed { .. }) {
            c.constant_total += data.len();
        } else {
            c.authored_total += data.len();
            c.authored_nonzero += nz;
        }
    }
    c
}

/// What a family adds to a literal release beyond the spec, the patch and the tensors.
#[derive(Debug, Clone)]
pub struct ReleaseContext {
    /// Population the release is meant to be evaluated on. It was not fitted on it.
    pub cohort_id: String,
    pub split_id: String,
    pub source_manifest_hash: String,
    pub applicability: JsonMap,
    pub observation_model: JsonMap,
    pub numerics: JsonMap,
    pub uncertainty_kind: UncertaintyKind,
    /// Digest of the forward source that will run this release.
    pub forward_hash: Digest,
    /// Result of the strict-route audit and of a finite forward on a fixture.
    pub checks: Vec<BuildCheck>,
}

fn passed(check: BuildCheckKind, detail: impl Into<String>) -> BuildCheck {
    BuildCheck {
        check,
        status: CheckStatus::Passed,
        detail: Some(detail.into()),
    }
}

/// Builds the release record of a literal model. Deterministic for identical inputs: `code_version`
/// is derived from the crate version and the forward digest, not from the git working tree, so
/// re-assembling the same spec and patch with the same forward reproduces the release id.
pub fn build_release(
    spec: &ModelSpec,
    patch: &HandwrittenWeightPatch,
    tensors: &TensorMap,
    parent: Option<&LoadedRelease>,
    ctx: ReleaseContext,
) -> anyhow::Result<ModelRelease> {
    let frozen: Vec<String> = tensors.keys().cloned().collect();
    let entries = registry::tensor_entries(tensors, &frozen);
    let method = if patch.parent_release_id.is_some() {
        ConstructionMethod::LiteralRevision
    } else {
        ConstructionMethod::LiteralOneShot
    };
    let n = counts(spec, tensors);
    let mut checks = vec![
        passed(BuildCheckKind::SpecValid, "ModelSpec::validate"),
        passed(
            BuildCheckKind::PatchValid,
            "HandwrittenWeightPatch::validate_against",
        ),
        passed(
            BuildCheckKind::TensorsComplete,
            format!(
                "{} authored elements ({} nonzero), {} constants",
                n.authored_total, n.authored_nonzero, n.constant_total
            ),
        ),
    ];
    checks.extend(ctx.checks);
    let code_version = format!(
        "ignis-wsp@{}+fwd.{}",
        env!("CARGO_PKG_VERSION"),
        &ctx.forward_hash.hex()[..16]
    );
    let construction = ModelBuildManifest {
        schema_version: WspSchema::V1,
        construction_method: method,
        spec_id: spec.spec_id.clone(),
        spec_hash: spec.spec_hash(),
        patch_id: Some(patch.patch_id.clone()),
        patch_hash: Some(patch.patch_hash()),
        parent_release_id: patch.parent_release_id.clone(),
        author: Some(patch.author.clone()),
        exposure: patch.exposure.clone(),
        weights_hash: weights_digest(&entries),
        forward: spec.forward.clone(),
        forward_hash: ctx.forward_hash,
        code_version: code_version.clone(),
        uncertainty_kind: ctx.uncertainty_kind,
        checks,
        origin: patch.origin,
    };
    let architecture: Value = json!({
        "kind": "literal_wsp",
        "spec_id": spec.spec_id,
        "spec_hash": spec.spec_hash(),
        "forward": spec.forward,
        "parameters": n,
    });
    let release = ModelRelease {
        id: String::new(),
        family: spec.family,
        model_name: spec.model_name.clone(),
        is_baseline: false,
        architecture: architecture.as_object().cloned().unwrap_or_default(),
        config: JsonMap::new(),
        feature_order: spec.features.iter().map(|f| f.name.clone()).collect(),
        units: spec
            .features
            .iter()
            .map(|f| (f.name.clone(), f.unit.clone()))
            .collect(),
        normalization: BTreeMap::new(),
        priors: JsonMap::new(),
        observation_model: ctx.observation_model,
        numerics: ctx.numerics,
        tensors: entries,
        posterior: vec![],
        posterior_parameters: vec![],
        cohort_id: ctx.cohort_id,
        split_id: ctx.split_id,
        source_manifest_hash: ctx.source_manifest_hash,
        code_version,
        applicability: ctx.applicability,
        evaluation: JsonMap::new(),
        state: QualificationState::Experimental,
        parent_id: parent.map(|p| p.release.id.clone()),
        lineage: match method {
            ConstructionMethod::LiteralOneShot => Lineage::LiteralAuthoring,
            _ => Lineage::WeightEdit,
        },
        lineage_note: patch.rationale.as_ref().map(|r| {
            r.lines()
                .next()
                .unwrap_or_default()
                .chars()
                .take(240)
                .collect()
        }),
        posterior_valid_for_parameters: false,
        trained_on: TrainedOn::NotTrained,
        resource_record: None,
        created_at: String::new(),
        construction: Some(construction),
    };
    release.check_construction()?;
    spec.check_release(&release)?;
    Ok(release)
}

/// Stores a literal release in the content-addressed registry (idempotent) and returns it with its
/// id. The stored tensor table is re-read and compared, so what is served is what was assembled.
pub fn save(
    root: &Path,
    release: ModelRelease,
    tensors: &TensorMap,
) -> anyhow::Result<ModelRelease> {
    let frozen: Vec<String> = tensors.keys().cloned().collect();
    let saved = registry::save_release(root, release, tensors, &TensorMap::new(), &frozen)?;
    let loaded = registry::load(root, &saved.id)?;
    ensure!(
        &loaded.params == tensors,
        "stored tensors differ from the assembled tensors"
    );
    Ok(saved)
}

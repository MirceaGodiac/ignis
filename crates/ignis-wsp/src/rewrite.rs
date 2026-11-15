//! A complete literal rewrite for a new model specification.
//!
//! Ordinary WSP revisions inherit tensors and therefore keep the same spec. A rewrite starts from
//! the destination spec's zero/fixed initialization and requires a complete literal patch. The
//! actual prior release is retained only as ancestry and is never read for tensor values.

use anyhow::{bail, ensure, Context};

use ignis_core::contracts::{
    ConstructionMethod, HandwrittenWeightPatch, ModelSpec, ReleaseId, TrainedOn,
};
use ignis_core::registry::{LoadedRelease, TensorMap};

use crate::{assemble, Binding};

pub const RATIONALE_PREFIX: &str =
    "Data-informed new-spec full literal rewrite; parent records ancestry, not tensor inheritance.";

/// Check the provenance and destination-patch requirements for a cross-spec literal rewrite.
/// This is intentionally separate from ordinary same-spec revision materialization.
pub fn validate(
    spec: &ModelSpec,
    patch: &HandwrittenWeightPatch,
    parent: &LoadedRelease,
    binding: &Binding,
) -> anyhow::Result<()> {
    spec.validate()?;
    patch
        .validate_against(spec)
        .context("destination patch does not match the new spec")?;

    let parent_id = patch
        .parent_release_id
        .as_ref()
        .context("a new-spec rewrite must name its actual parent release")?;
    ensure!(
        parent_id.as_str() == parent.release.id,
        "patch parent {parent_id} differs from loaded release {}",
        parent.release.id
    );
    require_literal_parent(parent)?;
    ensure!(
        parent.release.family == spec.family,
        "parent family {:?} differs from destination family {:?}",
        parent.release.family,
        spec.family
    );
    require_new_spec(spec, parent)?;
    ensure!(
        patch
            .exposure
            .prior_candidates
            .contains(&ReleaseId::new(parent.release.id.clone())?),
        "exposure must list the actual parent as a prior candidate"
    );
    ensure!(
        patch
            .rationale
            .as_deref()
            .is_some_and(|r| r.starts_with(RATIONALE_PREFIX)),
        "rationale must begin: {RATIONALE_PREFIX}"
    );
    ensure!(
        binding.cohort_id == parent.release.cohort_id
            && binding.split_id == parent.release.split_id
            && binding.source_manifest_hash == parent.release.source_manifest_hash,
        "rewrite must retain the parent's cohort, split and source manifest binding"
    );

    // A parentless copy invokes the existing root completeness rule without changing the original
    // patch, its exposure, or the serialized construction method.
    let mut coverage = patch.clone();
    coverage.parent_release_id = None;
    coverage
        .validate_against(spec)
        .context("destination patch does not completely cover the new spec")?;
    Ok(())
}

/// Materialize a rewrite from only the new spec and its complete literal assignments.
///
/// The parent is checked as ancestry, but none of its tensors are copied or read here.
pub fn materialize(
    spec: &ModelSpec,
    patch: &HandwrittenWeightPatch,
    parent: &LoadedRelease,
    binding: &Binding,
) -> anyhow::Result<TensorMap> {
    validate(spec, patch, parent, binding)?;
    let mut coverage = patch.clone();
    coverage.parent_release_id = None;
    assemble::materialize(spec, &coverage, None)
        .context("fresh destination tensors plus complete literal assignments")
}

/// Verify the release metadata produced for the rewrite before it is saved.
pub fn check_release_parent(
    patch: &HandwrittenWeightPatch,
    release: &ignis_core::contracts::ModelRelease,
    parent: &LoadedRelease,
) -> anyhow::Result<()> {
    let actual_parent = ReleaseId::new(parent.release.id.clone())?;
    ensure!(
        patch.parent_release_id.as_ref() == Some(&actual_parent),
        "patch does not identify the loaded parent"
    );
    ensure!(
        release.parent_id.as_deref() == Some(parent.release.id.as_str()),
        "rewrite release does not preserve parent ancestry"
    );
    let manifest = release
        .construction
        .as_ref()
        .context("rewrite release lacks its construction manifest")?;
    ensure!(
        manifest.construction_method == ConstructionMethod::LiteralRevision
            && manifest.parent_release_id.as_ref() == Some(&actual_parent),
        "rewrite must serialize as a literal revision with its actual parent"
    );
    Ok(())
}

/// Reject attempts to send a same-spec patch through the specialized rewrite route.
pub fn require_new_spec(spec: &ModelSpec, parent: &LoadedRelease) -> anyhow::Result<()> {
    let manifest = parent
        .release
        .construction
        .as_ref()
        .context("parent has no construction manifest")?;
    ensure!(
        manifest.spec_hash != spec.spec_hash(),
        "destination spec must differ from the parent's spec"
    );
    ensure!(
        parent.release.model_name != spec.model_name,
        "a rewrite must have a new model name"
    );
    Ok(())
}

/// Kept for an explicit diagnostic if callers pass an unsupported parent to the rewrite path.
pub fn require_literal_parent(parent: &LoadedRelease) -> anyhow::Result<()> {
    let Some(manifest) = parent.release.construction.as_ref() else {
        bail!("parent has no literal construction manifest")
    };
    ensure!(
        manifest.construction_method.is_literal()
            && parent.release.trained_on == TrainedOn::NotTrained,
        "parent must be a literal, not_trained release"
    );
    parent.release.check_construction()?;
    Ok(())
}

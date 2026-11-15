//! Synthetic provenance and materialization checks for new-spec literal rewrites.

mod common;

use ignis_core::contracts::{Family, HandwrittenWeightPatch, ReleaseId, SpecId, TrainedOn};
use ignis_core::registry::LoadedRelease;
use ignis_wsp::{assemble, flex, rewrite, Binding};

fn destination_spec() -> ignis_core::contracts::ModelSpec {
    let mut spec = flex::spec();
    spec.spec_id = SpecId::new("flex-rewrite-test").unwrap();
    spec.model_name = "flex.wsp.rewrite-test".into();
    spec.forward.version = "rewrite-test@1".into();
    spec
}

fn rewrite_patch(
    parent: &LoadedRelease,
    spec: &ignis_core::contracts::ModelSpec,
) -> HandwrittenWeightPatch {
    let (_, mut patch) = common::flex_patch("rate.b0 = -1.0");
    patch.spec_hash = spec.spec_hash();
    patch.parent_release_id = Some(ReleaseId::new(parent.release.id.clone()).unwrap());
    patch
        .exposure
        .prior_candidates
        .push(ReleaseId::new(parent.release.id.clone()).unwrap());
    patch.rationale = Some(format!(
        "{} Synthetic route test.",
        rewrite::RATIONALE_PREFIX
    ));
    patch
}

fn parent() -> LoadedRelease {
    let mut parent = common::flex_release("rate.b0 = -0.6931471805599453");
    // `build_release` leaves the content-addressed id empty until registry save; this in-memory
    // fixture gives the synthetic parent a stable opaque identity for ancestry checks.
    parent.release.id = "synthetic-parent-release".into();
    parent
}

fn binding() -> Binding {
    common::binding()
}

#[test]
fn rewrite_uses_only_complete_destination_literals_and_keeps_parent_as_ancestry() {
    let parent = parent();
    let before = parent.params.clone();
    let spec = destination_spec();
    let patch = rewrite_patch(&parent, &spec);

    let tensors = rewrite::materialize(&spec, &patch, &parent, &binding()).unwrap();
    assert_eq!(tensors["rate.b0"].data, [-1.0]);
    assert_eq!(parent.params, before, "parent tensor table was mutated");

    // The normal revision route still rejects cross-spec inheritance.
    assert!(assemble::materialize(&spec, &patch, Some(&parent)).is_err());
}

#[test]
fn rewrite_requires_full_coverage_even_though_it_has_a_parent() {
    let parent = parent();
    let spec = destination_spec();
    let mut patch = rewrite_patch(&parent, &spec);
    let index = patch
        .ops
        .iter()
        .position(|op| op.tensor() == "haz.log_disruption")
        .unwrap();
    patch.ops.remove(index);
    assert!(rewrite::materialize(&spec, &patch, &parent, &binding()).is_err());
}

#[test]
fn rewrite_requires_the_actual_parent_in_exposure() {
    let parent = parent();
    let spec = destination_spec();
    let mut patch = rewrite_patch(&parent, &spec);
    patch.exposure.prior_candidates.clear();
    assert!(rewrite::validate(&spec, &patch, &parent, &binding()).is_err());
}

#[test]
fn rewrite_rejects_a_wrong_loaded_parent_id() {
    let parent = parent();
    let spec = destination_spec();
    let mut patch = rewrite_patch(&parent, &spec);
    patch.parent_release_id = Some(ReleaseId::new("another-parent").unwrap());
    assert!(rewrite::validate(&spec, &patch, &parent, &binding()).is_err());
}

#[test]
fn rewrite_rejects_a_nonliteral_parent() {
    let mut parent = parent();
    parent.release.construction = None;
    parent.release.trained_on = TrainedOn::NotTrained;
    let spec = destination_spec();
    let patch = rewrite_patch(&parent, &spec);
    assert!(rewrite::validate(&spec, &patch, &parent, &binding()).is_err());
}

#[test]
fn rewrite_rejects_a_parent_from_another_family() {
    let mut parent = parent();
    parent.release.family = Family::AcmeNormal;
    let spec = destination_spec();
    let patch = rewrite_patch(&parent, &spec);
    assert!(rewrite::validate(&spec, &patch, &parent, &binding()).is_err());
}

#[test]
fn rewrite_rejects_a_changed_population_binding() {
    let parent = parent();
    let spec = destination_spec();
    let patch = rewrite_patch(&parent, &spec);
    let mut binding = binding();
    binding.split_id.push_str("-other");
    assert!(rewrite::validate(&spec, &patch, &parent, &binding).is_err());
}

#[test]
fn rewrite_route_rejects_same_spec_revisions() {
    let parent = parent();
    let spec = flex::spec();
    assert!(rewrite::require_new_spec(&spec, &parent).is_err());
}

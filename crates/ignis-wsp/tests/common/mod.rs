//! Test-only weights. These are deliberately not a scientific candidate: a constant burning rate, no
//! extinction, no encoder gain, no noise. Tests add the few lines they need on top.
#![allow(dead_code)]

use ignis_core::contracts::{
    AuthorModel, DataExposure, Family, HandwrittenWeightPatch, ModelSpec, Origin, PatchId,
};
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::assemble::{self, PatchMeta};
use ignis_wsp::{flex, text, Binding};

/// K = 0.5e-6 m2/s everywhere, hazards off, encoder gain 0, zero spread and noise.
pub const TOY_FLEX: &str = "
rate.b0 = -0.6931471805599453
haz.b0 = -50.0
haz.link_b = 0.0
haz.log_disruption = -20.0
enc.gain_w[0:3] = 0.0
enc.gain_b = -50.0
enc.mix[0:2] = [1.0, 0.0]
unc.s_pop = 0.0
unc.s_floor = 0.0
unc.sigma_d = 0.0
post.k_evap = 0.0
";

pub fn meta(id: &str) -> PatchMeta {
    PatchMeta {
        patch_id: PatchId::new(id).unwrap(),
        parent_release_id: None,
        author: AuthorModel {
            provider: "test".into(),
            model_id: "test-only-weights".into(),
            effort: None,
        },
        exposure: DataExposure::default(),
        rationale: Some("test-only weights, not a candidate".into()),
        origin: Origin::SyntheticFixture,
    }
}

pub fn binding() -> Binding {
    Binding {
        cohort_id: "test-cohort".into(),
        split_id: "test-split".into(),
        source_manifest_hash: "test-manifest".into(),
    }
}

/// `skip` rows default to zero in the toy model; `extra` statements must not rewrite an element the
/// base already wrote (a patch writes each element once), so the base leaves the skips to `extra`
/// when `extra` names them.
pub fn flex_patch(extra: &str) -> (ModelSpec, HandwrittenWeightPatch) {
    let spec = flex::spec();
    let mut src = String::new();
    for line in TOY_FLEX.lines() {
        let name = line.split(['[', ' ', '=']).next().unwrap_or_default();
        if !name.is_empty() && extra.lines().any(|l| l.trim_start().starts_with(name)) {
            continue;
        }
        src.push_str(line);
        src.push('\n');
    }
    for skip in ["rate.skip", "haz.skip"] {
        if !extra.contains(skip) {
            src.push_str(&format!("{skip}[0:7] = 0.0\n"));
        }
    }
    src.push_str(extra);
    let ops = text::compile(&src, &spec).unwrap_or_else(|e| panic!("{e}\n{src}"));
    let patch = assemble::make_patch(&spec, meta("test-flex"), ops);
    (spec, patch)
}

/// An in-memory release of the toy weights plus `extra`.
pub fn flex_release(extra: &str) -> LoadedRelease {
    let (spec, patch) = flex_patch(extra);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = ignis_wsp::build_release(Family::Flex, &patch, &tensors, None, &binding())
        .expect("release");
    LoadedRelease {
        release,
        params: tensors,
        posterior: TensorMap::new(),
    }
}

//! Literal weight-space programming (WSP) for the IGNIS scientific models.
//!
//! An author model writes numbers; this crate turns them into a release and runs it.
//!
//! * [`text`] — the edit language (`tensor[i, j] = 0.75`, literal lists, constant fills) compiled to
//!   the contract's `LiteralOp`s. No expressions.
//! * [`assemble`] — spec + patch (+ parent) -> tensors -> immutable, content-addressed release with
//!   its construction manifest. No estimation of any kind.
//! * [`flex`], [`acme`], [`bass`] — the frozen specs and strict forwards of `flex.wsp`,
//!   `acme_normal.wsp`, `acme_inverse.wsp` and `bass.wsp`; [`flex_tools`], [`acme_tools`],
//!   [`bass_tools`] — their fixtures and read-only inspection.
//! * [`droplet`] — the multistate droplet forward of `flex_methanol.wsp` and `flex_heptane.wsp`
//!   (hot flame, cool flame, no flame); [`droplet_tools`] — its fixture and inspection.
//! * [`droplet3`] — the third-generation droplet forward of `flex_methanol.wsp2` and
//!   `flex_heptane.wsp2` (its own identity; fixture and declarations from [`droplet_tools`]).
//! * [`flame3`] — the growth-clock forward of `acme_normal.wsp3` and `acme_inverse.wsp3`;
//!   [`droplet4`] — the fourth-generation droplet forward of `flex_methanol.wsp3` and
//!   `flex_heptane.wsp3`. Each has its own identity; earlier forwards are untouched.
//! * [`droplet6`] — the sixth droplet forward, of `flex_methanol.wsp5` and `flex_heptane.wsp5`:
//!   `droplet5` with a self-cooling law of the unlit droplet, a dying flame and the cool-flame end
//!   in oxygen partial pressure, each off by its own entry (its own identity).
//! * [`droplet8`] — `droplet6` for eight fuels (methanol, n-heptane, n-octane, iso-octane,
//!   n-decane, n-dodecane, farnesane, ethanol), models `flex_<fuel>.wsp7`, with the domain of CFI
//!   and FLEX-2 and a 200 s horizon (its own identity); [`droplet8_tools`] — fixture and declarations.
//! * [`droplet9`] — `droplet8` with the mechanisms of sections 3.2 and 3.3 of
//!   `docs/wsp/research/phys_droplet_fuels.md` (composition and staged burning, warm flame, direct
//!   cool ignition, fibre), each off by its own switch; eight pure fuels and three mixtures, models
//!   `flex_<fuel>.wsp8`; [`droplet9_tools`] — fixture and declarations.
//! * [`front`] — the flame-front forward of `bass_front.wsp` (tip of a concurrent flame over a
//!   thin fuel, the flow history as a known control); [`front_tools`] — its fixture and the
//!   reader of the front-track files of the measurement study.
//! * [`front2`] — the second flame-front forward, of `bass_front.wsp2`: `front` with two oxygen
//!   powers, a slow growth of the overhang, a slow-down downstream and flow paths, each off by its
//!   own entry (its own identity; fixture and track reader from [`front_tools`]).
//! * [`flame4`] — the hinge-law clocks of `flame` with the burner-heating law of `flame3` as their
//!   burner-limit clock, of `acme_normal.wsp8` and `acme_inverse.wsp8`; [`committee2`] — the
//!   committee of `acme_normal.wsp7` and `acme_inverse.wsp7`: ten member slots of the three
//!   burner-flame forwards, weights by the era of the test, three readings of the planned stop
//!   (each its own identity; `committee` is untouched).
//! * [`fuel`] — the typed fuel table that replaces the two fuel indicators; [`flame5`], [`flame6`],
//!   [`flame7`] — the laws of `flame`, `flame3` and `flame4` reading it, of `acme_*.wsp9`,
//!   `acme_*.wsp10` and `acme_*.wsp11`; [`committee3`] — the committee of `committee2` with those
//!   members, of `acme_*.wsp12` (each its own identity; the earlier forwards are untouched).
//! * [`fuel2`] — the second fuel table with a diluent table (helium, hydrogen, blends, the burner
//!   limit as an input); [`flame8`], [`flame9`], [`flame10`] — the laws of `flame5`, `flame6` and
//!   `flame7` reading it, with the heating law of `phys_sflame_f3.md`, of `acme_normal.wsp14`,
//!   `wsp15` and `wsp16`; [`committee4`] — the committee of `committee3` with those members and two
//!   oxygen candidates, of `acme_normal.wsp13` (normal flames only; earlier forwards untouched).
//! * [`front5`] — the fifth flame-front forward, of `bass_front.wsp5`: the tip of `front` unchanged,
//!   its quench hazard replaced by the boundary law of ignition and quench (a 41-node mixture over the
//!   boundary error); [`front5_tools`] — its declarations and its track reader (pressure, flow
//!   direction and material as optional fields).
//! * [`models`] — the literal models by key; a release finds its forward through its `model_name`.
//! * [`audit`] — forward identity (`forward_hash`) and the strict-route check.
//! * [`cli`] — `ignis wsp spec | compile | check | assemble | inspect | forward | forward-hash`.
//!
//! The crate's only workspace dependency is `ignis-core`. The fitted experts, their optimizers and
//! samplers (`ignis-science`, `ignis-flex`, `ignis-acme`, `ignis-bass`) are not linked, so no
//! request can fall back to them.

// Comparisons such as `!(x > 0.0)` are written that way on purpose: they are also true for NaN,
// so a non-finite input is refused instead of slipping through.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

pub mod acme;
pub mod acme_tools;
pub mod assemble;
pub mod astra_tools;
pub mod audit;
pub mod bass;
pub mod bass_tools;
pub mod blend;
pub mod cli;
pub mod committee;
pub mod committee2;
pub mod committee3;
pub mod committee4;
pub mod common;
pub mod condition_committee;
pub mod droplet;
pub mod droplet3;
pub mod droplet4;
pub mod droplet5;
pub mod droplet6;
pub mod droplet7;
pub mod droplet8;
pub mod droplet8_tools;
pub mod droplet9;
pub mod droplet9_tools;
pub mod droplet_tools;
pub mod flame;
pub mod flame10;
pub mod flame3;
pub mod flame4;
pub mod flame5;
pub mod flame6;
pub mod flame7;
pub mod flame8;
pub mod flame9;
pub mod flame_tools;
pub mod flex;
pub mod flex_tools;
pub mod front;
pub mod front2;
pub mod front3;
pub mod front4;
// bass_front.wsp5: the boundary law of ignition and quench on the front of front.rs
pub mod front5;
pub mod front5_tools;
pub mod front_tools;
pub mod fuel;
pub mod fuel2;
// <slug>.wsp: the generic law forward, its slug table and its tools
pub mod law;
pub mod law_slugs;
pub mod law_tools;
pub mod models;
pub mod nn;
pub mod physical_core;
pub mod property_aerosol_v2;
pub mod property_aerosol_w0;
pub mod property_bre_w0;
pub mod property_catalog;
pub mod property_droplet_cleanroom;
pub mod property_droplet_release;
pub mod property_jets_v2;
pub mod property_jets_w0;
pub mod property_materials;
pub mod property_solids_v2;
pub mod property_solids_w0;
pub mod property_spherical_w0;
pub mod quad;
pub mod rewrite;
pub mod service;
pub mod spec;
pub mod text;
pub mod thermal;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use ignis_core::contracts::{
    AnalysisArtifact, BuildCheck, BuildCheckKind, CheckStatus, Family, HandwrittenWeightPatch,
    JsonMap, ModelRelease, ModelSpec, PrefixBundle, UncertaintyKind,
};
use ignis_core::expert::{Expert, Refusal};
use ignis_core::registry::{LoadedRelease, TensorMap};

pub use cli::{run, WspCmd};
pub use service::{MemoryPrefixes, PrefixSource, Provenance, ViewSourcePrefixes, WspModelService};

/// Families that have a frozen spec and a strict forward here.
pub const FAMILIES: [Family; 4] = [
    Family::Flex,
    Family::AcmeNormal,
    Family::AcmeInverse,
    Family::Bass,
];

/// Every literal expert compiled into this crate, for the `ignis` binary's expert list.
pub fn experts() -> Vec<Box<dyn Expert>> {
    let mut v: Vec<Box<dyn Expert>> = vec![
        Box::new(flex::FlexWsp::new()),
        Box::new(acme::AcmeWsp::new(Family::AcmeNormal)),
        Box::new(acme::AcmeWsp::new(Family::AcmeInverse)),
        Box::new(bass::BassWsp::new()),
        Box::new(droplet::DropletWsp::new(droplet::Fuel::Methanol)),
        Box::new(droplet::DropletWsp::new(droplet::Fuel::Heptane)),
        Box::new(flame::FlameWsp::new(Family::AcmeNormal)),
        Box::new(flame::FlameWsp::new(Family::AcmeInverse)),
        Box::new(droplet3::DropletWsp::new(droplet3::Fuel::Methanol)),
        Box::new(droplet3::DropletWsp::new(droplet3::Fuel::Heptane)),
        Box::new(front::FrontWsp::new()),
        Box::new(front2::FrontWsp::new()),
        Box::new(flame3::FlameWsp::new(Family::AcmeNormal)),
        Box::new(flame3::FlameWsp::new(Family::AcmeInverse)),
        Box::new(droplet4::DropletWsp::new(droplet4::Fuel::Methanol)),
        Box::new(droplet4::DropletWsp::new(droplet4::Fuel::Heptane)),
        Box::new(blend::BlendWsp::new(Family::AcmeNormal)),
        Box::new(blend::BlendWsp::new(Family::AcmeInverse)),
        Box::new(committee::CommitteeWsp::new(Family::AcmeNormal)),
        Box::new(committee::CommitteeWsp::new(Family::AcmeInverse)),
        Box::new(droplet5::DropletWsp::new(droplet5::Fuel::Methanol)),
        Box::new(droplet5::DropletWsp::new(droplet5::Fuel::Heptane)),
        Box::new(droplet6::DropletWsp::new(droplet6::Fuel::Methanol)),
        Box::new(droplet6::DropletWsp::new(droplet6::Fuel::Heptane)),
        Box::new(droplet7::DropletWsp::new(droplet7::Fuel::Methanol)),
        Box::new(droplet7::DropletWsp::new(droplet7::Fuel::Heptane)),
        Box::new(front3::FrontWsp::new()),
        Box::new(front4::FrontWsp::new()),
        Box::new(condition_committee::ConditionCommitteeWsp::new(
            Family::AcmeNormal,
        )),
        Box::new(condition_committee::ConditionCommitteeWsp::new(
            Family::AcmeInverse,
        )),
        Box::new(flame4::FlameWsp::new(Family::AcmeNormal)),
        Box::new(flame4::FlameWsp::new(Family::AcmeInverse)),
        Box::new(committee2::Committee2Wsp::new(Family::AcmeNormal)),
        Box::new(committee2::Committee2Wsp::new(Family::AcmeInverse)),
        Box::new(flame5::FlameWsp::new(Family::AcmeNormal)),
        Box::new(flame5::FlameWsp::new(Family::AcmeInverse)),
        Box::new(flame6::FlameWsp::new(Family::AcmeNormal)),
        Box::new(flame6::FlameWsp::new(Family::AcmeInverse)),
        Box::new(flame7::FlameWsp::new(Family::AcmeNormal)),
        Box::new(flame7::FlameWsp::new(Family::AcmeInverse)),
        Box::new(committee3::Committee3Wsp::new(Family::AcmeNormal)),
        Box::new(committee3::Committee3Wsp::new(Family::AcmeInverse)),
        Box::new(flame8::FlameWsp::new(Family::AcmeNormal)),
        Box::new(flame9::FlameWsp::new(Family::AcmeNormal)),
        Box::new(flame10::FlameWsp::new(Family::AcmeNormal)),
        Box::new(committee4::Committee4Wsp::new(Family::AcmeNormal)),
        // bass_front.wsp5
        Box::new(front5::FrontWsp::new()),
    ];
    // droplet8.rs: the sixth droplet law for eight fuels
    v.extend(
        droplet8::Fuel::ALL.map(|f| Box::new(droplet8::DropletWsp::new(f)) as Box<dyn Expert>),
    );
    // droplet9.rs: droplet8 with the mechanisms of sections 3.2 and 3.3
    v.extend(
        droplet9::Fuel::ALL.map(|f| Box::new(droplet9::DropletWsp::new(f)) as Box<dyn Expert>),
    );
    v
}

/// The frozen spec of a family.
pub fn spec_of(family: Family) -> anyhow::Result<ModelSpec> {
    match family {
        Family::Flex => Ok(flex::spec()),
        Family::AcmeNormal | Family::AcmeInverse => Ok(acme::spec(family)),
        Family::Bass => Ok(bass::spec()),
        other => bail!("no literal spec for family {}", other.as_str()),
    }
}

/// The strict forward of a family as an artifact: a forecast, or a scientific refusal that keeps the
/// target and carries no numbers.
pub fn artifact_of(
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<AnalysisArtifact, Refusal> {
    match models::by_name(&release.release.model_name) {
        Some(model) => model.artifact(release, prefix, settings),
        None => Err(Refusal::unsupported(
            "no_literal_forward",
            format!(
                "{} is not a literal model of this build",
                release.release.model_name
            ),
        )),
    }
}

/// The evaluation population a literal release is declared for. The release was not fitted on it;
/// the ids let the existing forecast and evaluation commands find the runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub cohort_id: String,
    pub split_id: String,
    pub source_manifest_hash: String,
}

fn object(v: Value) -> JsonMap {
    v.as_object().cloned().unwrap_or_default()
}

/// Assembles the release record of a family: tensors -> release with every build check, including a
/// finite forward on the family's synthetic fixture prefix.
pub fn build_release(
    family: Family,
    patch: &HandwrittenWeightPatch,
    tensors: &TensorMap,
    parent: Option<&LoadedRelease>,
    binding: &Binding,
) -> anyhow::Result<ModelRelease> {
    let spec = spec_of(family)?;
    let (a, o, n) = match family {
        Family::Flex => flex_tools::release_notes(),
        Family::Bass => bass_tools::release_notes(),
        _ => acme_tools::release_notes(family),
    };
    let applicability: JsonMap = a.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    let (observation_model, numerics) = (object(o), object(n));
    let ctx = |checks: Vec<BuildCheck>| assemble::ReleaseContext {
        cohort_id: binding.cohort_id.clone(),
        split_id: binding.split_id.clone(),
        source_manifest_hash: binding.source_manifest_hash.clone(),
        applicability: applicability.clone(),
        observation_model: observation_model.clone(),
        numerics: numerics.clone(),
        uncertainty_kind: UncertaintyKind::AssumedNoise,
        forward_hash: audit::forward_hash(family),
        checks,
    };
    let strict = audit::strict_route_check(family);
    // First pass: a release the forward can bind, used only to run the fixture.
    let probe = assemble::build_release(&spec, patch, tensors, parent, ctx(vec![strict.clone()]))
        .context("release record")?;
    let fixture = fixture_check(
        family,
        &LoadedRelease {
            release: probe,
            params: tensors.clone(),
            posterior: TensorMap::new(),
        },
    );
    assemble::build_release(&spec, patch, tensors, parent, ctx(vec![strict, fixture]))
}

/// Runs the strict forward on the family's synthetic fixture prefix and reports whether it produced
/// a valid, finite, admitted artifact. The fixture is generated in code and is never NASA data.
pub fn fixture_check(family: Family, release: &LoadedRelease) -> BuildCheck {
    let outcome: anyhow::Result<String> = (|| {
        let prefix = fixture_prefix(family)?;
        let a = artifact_of(release, &prefix, &Value::Null).map_err(|r| anyhow::anyhow!("{r}"))?;
        a.validate().map_err(anyhow::Error::msg)?;
        if !a.support.admitted {
            bail!("fixture refused: {}", a.support.reasons.join("; "));
        }
        Ok(format!(
            "synthetic fixture prefix: {} forecast times, valid finite artifact",
            a.times_s.len()
        ))
    })();
    let (status, detail) = match outcome {
        Ok(detail) => (CheckStatus::Passed, detail),
        Err(e) => (CheckStatus::Failed, format!("{e:#}")),
    };
    BuildCheck {
        check: BuildCheckKind::FiniteForwardOnFixture,
        status,
        detail: Some(detail),
    }
}

/// A deterministic synthetic prefix of a family (`evidence_class = synthetic`), for build checks,
/// tests and benchmarks.
pub fn fixture_prefix(family: Family) -> anyhow::Result<PrefixBundle> {
    match family {
        Family::Flex => Ok(flex_tools::fixture_prefix(
            3.0e-3, 0.55e-6, 3.0, 0.21, 0.0, 0.0, 101_325.0,
        )),
        Family::AcmeNormal | Family::AcmeInverse => Ok(acme_tools::default_fixture(family)),
        Family::Bass => Ok(bass_tools::fixture_prefix(4.0, 0.002, 0.001, false)),
        other => bail!("no fixture for family {}", other.as_str()),
    }
}

//! The literal models compiled into this crate, by key.
//!
//! A family can have more than one literal model (the first FLEX model and the multistate droplet
//! models are all `Family::Flex`). Commands take a model key; a release finds its forward through
//! its `model_name`. The four first-generation models keep their family names as keys, so existing
//! commands, files and release ids are unchanged.

use anyhow::{bail, Context};
use serde_json::Value;

use ignis_core::contracts::{
    AnalysisArtifact, BuildCheck, BuildCheckKind, CheckStatus, Digest, Family,
    HandwrittenWeightPatch, JsonMap, ModelRelease, ModelSpec, PrefixBundle, UncertaintyKind,
};
use ignis_core::expert::Refusal;
use ignis_core::registry::{LoadedRelease, TensorMap};

use crate::droplet::Fuel;
use crate::droplet6;
use crate::front2;
use crate::{
    acme, acme_tools, assemble, audit, bass, bass_tools, blend, committee, droplet, droplet3,
    droplet4, droplet5, droplet_tools, flame, flame3, flame_tools, flex, flex_tools, front,
    front_tools, Binding,
};
use crate::{astra_tools, condition_committee, droplet7, front3, front4};
use crate::{committee2, flame4};
use crate::{committee3, flame5, flame6, flame7};
use crate::{committee4, flame10, flame8, flame9};
// bass_front.wsp5
use crate::{droplet8, droplet8_tools};
use crate::{droplet9, droplet9_tools};
use crate::{front5, front5_tools};

/// Which forward a model runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forward {
    /// `flex.rs`: one hot-flame state with an extinction intensity.
    Flex1,
    /// `acme.rs`: hazard networks with elapsed time as an input.
    Acme1,
    /// `bass.rs`: cosine-mode front operator on metric contours.
    Bass1,
    /// `droplet.rs`: hot flame, cool flame, no flame.
    Droplet(Fuel),
    /// `flame.rs`: physical clocks of the burner flames.
    Flame,
    /// `droplet3.rs`: the third-generation droplet forward.
    Droplet3(droplet3::Fuel),
    /// `flame3.rs`: the growth clock of the burner flames.
    Flame3,
    /// `front.rs`: the flame front of a thin fuel in concurrent flow.
    Front,
    /// `front2.rs`: the second flame-front forward (`front.rs` with four additions, each off by
    /// its own entry).
    Front2,
    /// `droplet4.rs`: the fourth-generation droplet forward.
    Droplet4(droplet4::Fuel),
    /// `blend.rs`: the weighted mean of a `flame.rs` member and a `flame3.rs` member.
    Blend,
    /// `committee.rs`: up to six weighted members of the two burner-flame forwards.
    Committee,
    /// `droplet5.rs`: the multistate droplet forward that also reads the flame cameras.
    Droplet5(droplet5::Fuel),
    /// `droplet6.rs`: the sixth droplet forward (`droplet5.rs` with a self-cooling unlit droplet,
    /// a dying flame and the cool-flame end in oxygen partial pressure, each off by its own entry).
    Droplet6(droplet6::Fuel),
    /// Coupled mass-energy evolution after flame loss; neutral mode preserves droplet6.
    Droplet7(droplet7::Fuel),
    /// Oxygen-budget steady speed and coherent active-route competing events.
    Front3,
    Front4,
    /// Persistent flow/pressure scenarios of complete burner-flame committees.
    ConditionCommittee,
    /// `flame4.rs`: the hinge-law clocks of `flame.rs` with the burner-heating law of `flame3.rs`.
    Flame4,
    /// `committee2.rs`: ten weighted members of the three burner-flame forwards, with weights by
    /// the era of the test and three readings of the script's planned stop.
    Committee2,
    /// `flame5.rs`: the hinge laws of `flame.rs` reading a typed fuel table.
    Flame5,
    /// `flame6.rs`: the growth laws of `flame3.rs` reading a typed fuel table.
    Flame6,
    /// `flame7.rs`: the laws of `flame4.rs` reading a typed fuel table.
    Flame7,
    /// `committee3.rs`: the committee of `committee2.rs` with `flame5`, `flame6`, `flame7` members.
    Committee3,
    /// `flame8.rs`: the hinge laws reading the second fuel table and the diluent table.
    Flame8,
    /// `flame9.rs`: the growth laws with the second tables and the heating law of phys_sflame_f3.md.
    Flame9,
    /// `flame10.rs`: the hinge laws with the heating law, the second tables and its new terms.
    Flame10,
    /// `committee4.rs`: the committee of `committee3.rs` with `flame8`, `flame9`, `flame10` members
    /// and two oxygen candidates.
    Committee4,
    /// `front5.rs`: the tip of `front.rs` with the boundary law of ignition and quench.
    Front5,
    /// `droplet8.rs`: the law of `droplet6.rs` for eight fuels, with the domain of CFI and FLEX-2.
    Droplet8(droplet8::Fuel),
    /// `droplet9.rs`: `droplet8.rs` with the mechanisms of sections 3.2 and 3.3, each off by its switch.
    Droplet9(droplet9::Fuel),
    /// `law.rs`: the generic law forward of slug `i` of `law_slugs::SLUGS`.
    Law(usize),
}

/// One literal model: its command-line key, release `model_name`, family and forward.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Model {
    pub key: &'static str,
    pub model_name: &'static str,
    pub family: Family,
    pub forward: Forward,
}

const fn model(
    key: &'static str,
    model_name: &'static str,
    family: Family,
    forward: Forward,
) -> Model {
    Model {
        key,
        model_name,
        family,
        forward,
    }
}

pub const MODELS: [Model; 66] = [
    model("flex", "flex.wsp", Family::Flex, Forward::Flex1),
    model(
        "acme_normal",
        "acme_normal.wsp",
        Family::AcmeNormal,
        Forward::Acme1,
    ),
    model(
        "acme_inverse",
        "acme_inverse.wsp",
        Family::AcmeInverse,
        Forward::Acme1,
    ),
    model("bass", "bass.wsp", Family::Bass, Forward::Bass1),
    model(
        "flex_methanol",
        "flex_methanol.wsp",
        Family::Flex,
        Forward::Droplet(Fuel::Methanol),
    ),
    model(
        "flex_heptane",
        "flex_heptane.wsp",
        Family::Flex,
        Forward::Droplet(Fuel::Heptane),
    ),
    model(
        "acme_normal2",
        "acme_normal.wsp2",
        Family::AcmeNormal,
        Forward::Flame,
    ),
    model(
        "acme_inverse2",
        "acme_inverse.wsp2",
        Family::AcmeInverse,
        Forward::Flame,
    ),
    model(
        "flex_methanol2",
        droplet3::Fuel::Methanol.model_name(),
        Family::Flex,
        Forward::Droplet3(droplet3::Fuel::Methanol),
    ),
    model(
        "flex_heptane2",
        droplet3::Fuel::Heptane.model_name(),
        Family::Flex,
        Forward::Droplet3(droplet3::Fuel::Heptane),
    ),
    model(
        "acme_normal3",
        "acme_normal.wsp3",
        Family::AcmeNormal,
        Forward::Flame3,
    ),
    model(
        "acme_inverse3",
        "acme_inverse.wsp3",
        Family::AcmeInverse,
        Forward::Flame3,
    ),
    model(
        "flex_methanol3",
        droplet4::Fuel::Methanol.model_name(),
        Family::Flex,
        Forward::Droplet4(droplet4::Fuel::Methanol),
    ),
    model(
        "flex_heptane3",
        droplet4::Fuel::Heptane.model_name(),
        Family::Flex,
        Forward::Droplet4(droplet4::Fuel::Heptane),
    ),
    model(
        "bass_front",
        front::MODEL_NAME,
        Family::Bass,
        Forward::Front,
    ),
    model(
        "bass_front2",
        front2::MODEL_NAME,
        Family::Bass,
        Forward::Front2,
    ),
    model(
        "acme_normal4",
        "acme_normal.wsp4",
        Family::AcmeNormal,
        Forward::Blend,
    ),
    model(
        "acme_inverse4",
        "acme_inverse.wsp4",
        Family::AcmeInverse,
        Forward::Blend,
    ),
    model(
        "acme_normal5",
        "acme_normal.wsp5",
        Family::AcmeNormal,
        Forward::Committee,
    ),
    model(
        "acme_inverse5",
        "acme_inverse.wsp5",
        Family::AcmeInverse,
        Forward::Committee,
    ),
    model(
        "flex_methanol4",
        droplet5::Fuel::Methanol.model_name(),
        Family::Flex,
        Forward::Droplet5(droplet5::Fuel::Methanol),
    ),
    model(
        "flex_heptane4",
        droplet5::Fuel::Heptane.model_name(),
        Family::Flex,
        Forward::Droplet5(droplet5::Fuel::Heptane),
    ),
    model(
        "flex_methanol5",
        droplet6::Fuel::Methanol.model_name(),
        Family::Flex,
        Forward::Droplet6(droplet6::Fuel::Methanol),
    ),
    model(
        "flex_heptane5",
        droplet6::Fuel::Heptane.model_name(),
        Family::Flex,
        Forward::Droplet6(droplet6::Fuel::Heptane),
    ),
    model(
        "flex_methanol6",
        "flex_methanol.wsp6",
        Family::Flex,
        Forward::Droplet7(droplet7::Fuel::Methanol),
    ),
    model(
        "flex_heptane6",
        "flex_heptane.wsp6",
        Family::Flex,
        Forward::Droplet7(droplet7::Fuel::Heptane),
    ),
    model(
        "bass_front3",
        "bass_front.wsp3",
        Family::Bass,
        Forward::Front3,
    ),
    model(
        "bass_front4",
        "bass_front.wsp4",
        Family::Bass,
        Forward::Front4,
    ),
    model(
        "acme_normal6",
        "acme_normal.wsp6",
        Family::AcmeNormal,
        Forward::ConditionCommittee,
    ),
    model(
        "acme_inverse6",
        "acme_inverse.wsp6",
        Family::AcmeInverse,
        Forward::ConditionCommittee,
    ),
    model(
        "acme_normal7",
        "acme_normal.wsp7",
        Family::AcmeNormal,
        Forward::Committee2,
    ),
    model(
        "acme_inverse7",
        "acme_inverse.wsp7",
        Family::AcmeInverse,
        Forward::Committee2,
    ),
    model(
        "acme_normal8",
        "acme_normal.wsp8",
        Family::AcmeNormal,
        Forward::Flame4,
    ),
    model(
        "acme_inverse8",
        "acme_inverse.wsp8",
        Family::AcmeInverse,
        Forward::Flame4,
    ),
    model(
        "acme_normal9",
        "acme_normal.wsp9",
        Family::AcmeNormal,
        Forward::Flame5,
    ),
    model(
        "acme_inverse9",
        "acme_inverse.wsp9",
        Family::AcmeInverse,
        Forward::Flame5,
    ),
    model(
        "acme_normal10",
        "acme_normal.wsp10",
        Family::AcmeNormal,
        Forward::Flame6,
    ),
    model(
        "acme_inverse10",
        "acme_inverse.wsp10",
        Family::AcmeInverse,
        Forward::Flame6,
    ),
    model(
        "acme_normal11",
        "acme_normal.wsp11",
        Family::AcmeNormal,
        Forward::Flame7,
    ),
    model(
        "acme_inverse11",
        "acme_inverse.wsp11",
        Family::AcmeInverse,
        Forward::Flame7,
    ),
    model(
        "acme_normal12",
        "acme_normal.wsp12",
        Family::AcmeNormal,
        Forward::Committee3,
    ),
    model(
        "acme_inverse12",
        "acme_inverse.wsp12",
        Family::AcmeInverse,
        Forward::Committee3,
    ),
    model(
        "acme_normal13",
        "acme_normal.wsp13",
        Family::AcmeNormal,
        Forward::Committee4,
    ),
    model(
        "acme_normal14",
        "acme_normal.wsp14",
        Family::AcmeNormal,
        Forward::Flame8,
    ),
    model(
        "acme_normal15",
        "acme_normal.wsp15",
        Family::AcmeNormal,
        Forward::Flame9,
    ),
    model(
        "acme_normal16",
        "acme_normal.wsp16",
        Family::AcmeNormal,
        Forward::Flame10,
    ),
    // bass_front.wsp5
    model(
        "bass_front5",
        front5::MODEL_NAME,
        Family::Bass,
        Forward::Front5,
    ),
    // droplet8.rs: the sixth droplet law for eight fuels
    model(
        "flex_methanol7",
        droplet8::Fuel::Methanol.model_name(),
        Family::Flex,
        Forward::Droplet8(droplet8::Fuel::Methanol),
    ),
    model(
        "flex_heptane7",
        droplet8::Fuel::Heptane.model_name(),
        Family::Flex,
        Forward::Droplet8(droplet8::Fuel::Heptane),
    ),
    model(
        "flex_octane7",
        droplet8::Fuel::Octane.model_name(),
        Family::Flex,
        Forward::Droplet8(droplet8::Fuel::Octane),
    ),
    model(
        "flex_isooctane7",
        droplet8::Fuel::Isooctane.model_name(),
        Family::Flex,
        Forward::Droplet8(droplet8::Fuel::Isooctane),
    ),
    model(
        "flex_decane7",
        droplet8::Fuel::Decane.model_name(),
        Family::Flex,
        Forward::Droplet8(droplet8::Fuel::Decane),
    ),
    model(
        "flex_dodecane7",
        droplet8::Fuel::Dodecane.model_name(),
        Family::Flex,
        Forward::Droplet8(droplet8::Fuel::Dodecane),
    ),
    model(
        "flex_farnesane7",
        droplet8::Fuel::Farnesane.model_name(),
        Family::Flex,
        Forward::Droplet8(droplet8::Fuel::Farnesane),
    ),
    model(
        "flex_ethanol7",
        droplet8::Fuel::Ethanol.model_name(),
        Family::Flex,
        Forward::Droplet8(droplet8::Fuel::Ethanol),
    ),
    // droplet9.rs: droplet8 with the mechanisms of sections 3.2 and 3.3
    model(
        "flex_methanol8",
        droplet9::Fuel::Methanol.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::Methanol),
    ),
    model(
        "flex_heptane8",
        droplet9::Fuel::Heptane.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::Heptane),
    ),
    model(
        "flex_octane8",
        droplet9::Fuel::Octane.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::Octane),
    ),
    model(
        "flex_isooctane8",
        droplet9::Fuel::Isooctane.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::Isooctane),
    ),
    model(
        "flex_decane8",
        droplet9::Fuel::Decane.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::Decane),
    ),
    model(
        "flex_dodecane8",
        droplet9::Fuel::Dodecane.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::Dodecane),
    ),
    model(
        "flex_farnesane8",
        droplet9::Fuel::Farnesane.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::Farnesane),
    ),
    model(
        "flex_ethanol8",
        droplet9::Fuel::Ethanol.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::Ethanol),
    ),
    model(
        "flex_heptane_isooctane8",
        droplet9::Fuel::HeptaneIsooctane.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::HeptaneIsooctane),
    ),
    model(
        "flex_dodecane_isododecane8",
        droplet9::Fuel::DodecaneIsododecane.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::DodecaneIsododecane),
    ),
    model(
        "flex_propanol_glycerol8",
        droplet9::Fuel::PropanolGlycerol.model_name(),
        Family::Flex,
        Forward::Droplet9(droplet9::Fuel::PropanolGlycerol),
    ),
];

pub fn by_key(key: &str) -> anyhow::Result<Model> {
    if let Some(m) = crate::law_tools::model_by_key(key) {
        return Ok(m);
    }
    MODELS
        .iter()
        .copied()
        .find(|m| m.key == key)
        .with_context(|| {
            format!(
                "unknown model {key:?} ({})",
                MODELS.map(|m| m.key).join(", ")
            )
        })
}

pub fn by_name(model_name: &str) -> Option<Model> {
    if let Some(m) = crate::law_tools::model_by_name(model_name) {
        return Some(m);
    }
    MODELS.iter().copied().find(|m| m.model_name == model_name)
}

/// The first-generation model of a family (the one the family-keyed functions mean).
pub fn first_of(family: Family) -> anyhow::Result<Model> {
    MODELS
        .iter()
        .copied()
        .find(|m| m.family == family)
        .with_context(|| format!("no literal model for family {}", family.as_str()))
}

impl Model {
    pub fn spec(self) -> ModelSpec {
        match self.forward {
            Forward::Flex1 => flex::spec(),
            Forward::Acme1 => acme::spec(self.family),
            Forward::Bass1 => bass::spec(),
            Forward::Droplet(fuel) => droplet::spec(fuel),
            Forward::Flame => flame::spec(self.family),
            Forward::Droplet3(fuel) => droplet3::spec(fuel),
            Forward::Flame3 => flame3::spec(self.family),
            Forward::Front => front::spec(),
            Forward::Front2 => front2::spec(),
            Forward::Droplet4(fuel) => droplet4::spec(fuel),
            Forward::Blend => blend::spec(self.family),
            Forward::Committee => committee::spec(self.family),
            Forward::Droplet5(fuel) => droplet5::spec(fuel),
            Forward::Droplet6(fuel) => droplet6::spec(fuel),
            Forward::Droplet7(fuel) => droplet7::spec(fuel),
            Forward::Front3 => front3::spec(),
            Forward::Front4 => front4::spec(),
            Forward::ConditionCommittee => condition_committee::spec(self.family),
            Forward::Flame4 => flame4::spec(self.family),
            Forward::Committee2 => committee2::spec(self.family),
            Forward::Flame5 => flame5::spec(self.family),
            Forward::Flame6 => flame6::spec(self.family),
            Forward::Flame7 => flame7::spec(self.family),
            Forward::Committee3 => committee3::spec(self.family),
            Forward::Flame8 => flame8::spec(self.family),
            Forward::Flame9 => flame9::spec(self.family),
            Forward::Flame10 => flame10::spec(self.family),
            Forward::Committee4 => committee4::spec(self.family),
            Forward::Front5 => front5::spec(),
            Forward::Droplet8(fuel) => droplet8::spec(fuel),
            Forward::Droplet9(fuel) => droplet9::spec(fuel),
            Forward::Law(i) => crate::law::spec(i),
        }
    }

    pub fn forward_hash(self) -> Digest {
        audit::forward_hash_of(self.model_name)
    }

    /// The strict forward as an artifact: a forecast, or a scientific refusal that keeps the target.
    pub fn artifact(
        self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> Result<AnalysisArtifact, Refusal> {
        let spec = self.spec();
        match self.forward {
            Forward::Flex1 => flex::artifact(&spec, release, prefix, settings),
            Forward::Acme1 => acme::artifact(&spec, release, prefix, settings),
            Forward::Bass1 => bass::artifact(&spec, release, prefix, settings),
            Forward::Droplet(_) => droplet::artifact(&spec, release, prefix, settings),
            Forward::Flame => flame::artifact(&spec, release, prefix, settings),
            Forward::Droplet3(_) => droplet3::artifact(&spec, release, prefix, settings),
            Forward::Flame3 => flame3::artifact(&spec, release, prefix, settings),
            Forward::Front => front::artifact(&spec, release, prefix, settings),
            Forward::Front2 => front2::artifact(&spec, release, prefix, settings),
            Forward::Droplet4(_) => droplet4::artifact(&spec, release, prefix, settings),
            Forward::Blend => blend::artifact(&spec, release, prefix, settings),
            Forward::Committee => committee::artifact(&spec, release, prefix, settings),
            Forward::Droplet5(_) => droplet5::artifact(&spec, release, prefix, settings),
            Forward::Droplet6(_) => droplet6::artifact(&spec, release, prefix, settings),
            Forward::Droplet7(_) => droplet7::artifact(&spec, release, prefix, settings),
            Forward::Front3 => front3::artifact(&spec, release, prefix, settings),
            Forward::Front4 => front4::artifact(&spec, release, prefix, settings),
            Forward::ConditionCommittee => {
                condition_committee::artifact(&spec, release, prefix, settings)
            }
            Forward::Flame4 => flame4::artifact(&spec, release, prefix, settings),
            Forward::Committee2 => committee2::artifact(&spec, release, prefix, settings),
            Forward::Flame5 => flame5::artifact(&spec, release, prefix, settings),
            Forward::Flame6 => flame6::artifact(&spec, release, prefix, settings),
            Forward::Flame7 => flame7::artifact(&spec, release, prefix, settings),
            Forward::Committee3 => committee3::artifact(&spec, release, prefix, settings),
            Forward::Flame8 => flame8::artifact(&spec, release, prefix, settings),
            Forward::Flame9 => flame9::artifact(&spec, release, prefix, settings),
            Forward::Flame10 => flame10::artifact(&spec, release, prefix, settings),
            Forward::Committee4 => committee4::artifact(&spec, release, prefix, settings),
            Forward::Front5 => front5::artifact(&spec, release, prefix, settings),
            Forward::Droplet8(_) => droplet8::artifact(&spec, release, prefix, settings),
            Forward::Droplet9(_) => droplet9::artifact(&spec, release, prefix, settings),
            Forward::Law(i) => crate::law::artifact(i, &spec, release, prefix, settings),
        }
    }

    /// A deterministic synthetic prefix (`evidence_class = synthetic`), for build checks and tests.
    pub fn fixture_prefix(self) -> PrefixBundle {
        match self.forward {
            Forward::Flex1 => {
                flex_tools::fixture_prefix(3.0e-3, 0.55e-6, 3.0, 0.21, 0.0, 0.0, 101_325.0)
            }
            Forward::Acme1
            | Forward::Flame
            | Forward::Flame3
            | Forward::Blend
            | Forward::Committee
            | Forward::ConditionCommittee
            | Forward::Flame4
            | Forward::Committee2
            | Forward::Flame5
            | Forward::Flame6
            | Forward::Flame7
            | Forward::Committee3
            | Forward::Flame8
            | Forward::Flame9
            | Forward::Flame10
            | Forward::Committee4 => acme_tools::default_fixture(self.family),
            Forward::Bass1 => bass_tools::fixture_prefix(4.0, 0.002, 0.001, false),
            Forward::Front => front_tools::fixture_prefix(10.0, 0.001),
            Forward::Front2 => front_tools::fixture_prefix(10.0, 0.001),
            Forward::Front3 => front_tools::fixture_prefix(10.0, 0.001),
            Forward::Front4 => front_tools::fixture_prefix(10.0, 0.001),
            Forward::Front5 => front_tools::fixture_prefix(10.0, 0.001),
            Forward::Droplet(fuel) => droplet_tools::fixture_prefix(fuel),
            Forward::Droplet3(fuel) => droplet_tools::fixture_prefix(droplet_tools::fuel_of3(fuel)),
            Forward::Droplet4(fuel) => droplet_tools::fixture_prefix(droplet_tools::fuel_of4(fuel)),
            Forward::Droplet5(fuel) => droplet_tools::fixture_prefix(droplet_tools::fuel_of5(fuel)),
            Forward::Droplet6(fuel) => droplet_tools::fixture_prefix(droplet_tools::fuel_of6(fuel)),
            Forward::Droplet7(fuel) => droplet_tools::fixture_prefix(astra_tools::fuel_of7(fuel)),
            Forward::Droplet8(fuel) => droplet8_tools::fixture_prefix(fuel),
            Forward::Droplet9(fuel) => droplet9_tools::fixture_prefix(fuel),
            Forward::Law(i) => crate::law_tools::fixture_prefix(i),
        }
    }

    fn release_notes(self) -> (JsonMap, JsonMap, JsonMap) {
        let (a, o, n) = match self.forward {
            Forward::Flex1 => flex_tools::release_notes(),
            Forward::Acme1 => acme_tools::release_notes(self.family),
            Forward::Bass1 => bass_tools::release_notes(),
            Forward::Droplet(fuel) => droplet_tools::release_notes(fuel),
            Forward::Flame => flame_tools::release_notes(self.family),
            Forward::Droplet3(fuel) => droplet_tools::release_notes3(fuel),
            Forward::Flame3 => flame_tools::release_notes3(self.family),
            Forward::Front => front_tools::release_notes(),
            Forward::Front2 => front_tools::release_notes2(),
            Forward::Droplet4(fuel) => droplet_tools::release_notes4(fuel),
            Forward::Blend => flame_tools::release_notes4(self.family),
            Forward::Committee => flame_tools::release_notes5(self.family),
            Forward::Droplet5(fuel) => droplet_tools::release_notes5(fuel),
            Forward::Droplet6(fuel) => droplet_tools::release_notes6(fuel),
            Forward::Droplet7(fuel) => astra_tools::droplet_notes(fuel),
            Forward::Front3 => astra_tools::front_notes(),
            Forward::Front4 => astra_tools::front4_notes(),
            Forward::ConditionCommittee => astra_tools::condition_notes(self.family),
            Forward::Flame4 => flame_tools::release_notes8(self.family),
            Forward::Committee2 => flame_tools::release_notes7(self.family),
            Forward::Flame5 => flame_tools::release_notes9(self.family),
            Forward::Flame6 => flame_tools::release_notes10(self.family),
            Forward::Flame7 => flame_tools::release_notes11(self.family),
            Forward::Committee3 => flame_tools::release_notes12(self.family),
            Forward::Flame8 => flame_tools::release_notes14(self.family),
            Forward::Flame9 => flame_tools::release_notes15(self.family),
            Forward::Flame10 => flame_tools::release_notes16(self.family),
            Forward::Committee4 => flame_tools::release_notes13(self.family),
            Forward::Front5 => front5_tools::release_notes(),
            Forward::Droplet8(fuel) => droplet8_tools::release_notes(fuel),
            Forward::Droplet9(fuel) => droplet9_tools::release_notes(fuel),
            Forward::Law(i) => crate::law_tools::release_notes(i),
        };
        let object = |v: Value| v.as_object().cloned().unwrap_or_default();
        (
            a.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            object(o),
            object(n),
        )
    }

    /// Runs the strict forward on the synthetic fixture and reports whether it produced a valid,
    /// finite, admitted artifact.
    pub fn fixture_check(self, release: &LoadedRelease) -> BuildCheck {
        if let Forward::Law(i) = self.forward {
            return crate::law_tools::fixture_check(i, self, release);
        }
        let outcome: anyhow::Result<String> = (|| {
            let prefix = self.fixture_prefix();
            let a = self
                .artifact(release, &prefix, &Value::Null)
                .map_err(|r| anyhow::anyhow!("{r}"))?;
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

    /// Assembles the release record: tensors -> release with every build check, including a finite
    /// forward on the synthetic fixture prefix.
    pub fn build_release(
        self,
        patch: &HandwrittenWeightPatch,
        tensors: &TensorMap,
        parent: Option<&LoadedRelease>,
        binding: &Binding,
    ) -> anyhow::Result<ModelRelease> {
        let spec = self.spec();
        let (applicability, observation_model, numerics) = self.release_notes();
        let ctx = |checks: Vec<BuildCheck>| assemble::ReleaseContext {
            cohort_id: binding.cohort_id.clone(),
            split_id: binding.split_id.clone(),
            source_manifest_hash: binding.source_manifest_hash.clone(),
            applicability: applicability.clone(),
            observation_model: observation_model.clone(),
            numerics: numerics.clone(),
            uncertainty_kind: UncertaintyKind::AssumedNoise,
            forward_hash: self.forward_hash(),
            checks,
        };
        let strict = audit::strict_route_check_of(self.model_name);
        // First pass: a release the forward can bind, used only to run the fixture.
        let probe =
            assemble::build_release(&spec, patch, tensors, parent, ctx(vec![strict.clone()]))
                .context("release record")?;
        let fixture = self.fixture_check(&LoadedRelease {
            release: probe,
            params: tensors.clone(),
            posterior: TensorMap::new(),
        });
        assemble::build_release(&spec, patch, tensors, parent, ctx(vec![strict, fixture]))
    }
}

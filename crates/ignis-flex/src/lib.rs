//! FLEX family: physics-neural droplet model, V1 and D² baselines (V3.1 chapters 6–7, §23).
//!
//! `experts()` is collected by the `ignis` binary; every FLEX expert also exposes its
//! candidate-measurement model for VOI ranking (§11.2) through `Expert::candidate_predictive`.

pub mod data;
pub mod dynamics;
pub mod expert;
pub mod inference;
pub mod structure;
pub mod support;
pub mod synthetic;

pub use expert::FlexExpert;

pub fn experts() -> Vec<Box<dyn ignis_core::expert::Expert>> {
    structure::MODELS
        .iter()
        .map(|n| Box::new(FlexExpert::new(n)) as Box<dyn ignis_core::expert::Expert>)
        .collect()
}

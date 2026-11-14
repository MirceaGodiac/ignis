//! ACME competing-risk experts (V3.1 §8, §10.1). Two separate configuration-specific models.
//!
//! Names: `acme_<configuration>.<variant>` with configuration ∈ {normal, inverse} and variant ∈
//! {hazard, constant_baseline, piecewise_baseline, linear_baseline}.
//!
//! * [`model`]: the additive hazard, encoding, marginal likelihood with analytic gradient, forecast.
//! * [`expert`]: fitting (L-BFGS MAP, NUTS FP64 when qualified), serving, refusals.
//! * [`optim`], [`sampler`]: thin L-BFGS / NUTS (nuts-rs) wrappers pending `ignis_science` consolidation.

pub mod common;
pub mod expert;
pub mod features;
pub mod model;
pub mod optim;
pub mod sampler;
pub mod synthetic;

use ignis_core::expert::Expert;

pub fn experts() -> Vec<Box<dyn Expert>> {
    let mut out: Vec<Box<dyn Expert>> = vec![];
    for c in ["normal", "inverse"] {
        for v in expert::VARIANTS {
            out.push(Box::new(
                expert::AcmeExpert::new(c, v).expect("ACME expert configuration"),
            ));
        }
    }
    out
}

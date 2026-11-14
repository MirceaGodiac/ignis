//! BASS visible-front experts (V3.1 §9): cosine neural operator ensemble and baselines.
//!
//! `bass.operator` (5-member group-bootstrap AdamW MAP ensemble, members trained in parallel),
//! `bass.persistence`, `bass.constant_velocity`. Hand-written reverse mode through the operator, the
//! front PDE (second-order ENO2 / local Lax–Friedrichs, SSP-RK2) and the Student-t contour
//! likelihood with one run-level scale error (adaptive Gauss–Hermite).

pub mod data;
pub mod dynamics;
pub mod expert;
pub mod model;
pub mod observation;
pub mod operator;
pub mod synthetic;

use ignis_core::expert::Expert;

pub fn experts() -> Vec<Box<dyn Expert>> {
    vec![
        Box::new(expert::BassOperatorExpert::new().expect("BASS operator configuration")),
        Box::new(
            expert::BassBaselineExpert::new("persistence").expect("BASS baseline configuration"),
        ),
        Box::new(
            expert::BassBaselineExpert::new("constant_velocity")
                .expect("BASS baseline configuration"),
        ),
    ]
}

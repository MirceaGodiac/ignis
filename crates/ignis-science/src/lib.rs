//! Shared numerics for the IGNIS experts. Module ownership during the port:
//! science-a lane: ad, transforms, studentt, integrate, events, optim, nuts, diagnostics, predictive;
//! science-b lane: hazards, gauss_hermite, flows.

pub mod ad;
pub mod diagnostics;
pub mod events;
pub mod flows;
pub mod gauss_hermite;
pub mod hazards;
pub mod integrate;
pub mod nuts;
pub mod optim;
pub mod predictive;
pub mod studentt;
pub mod transforms;

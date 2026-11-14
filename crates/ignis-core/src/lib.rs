//! IGNIS core: the single authority for shared scientific definitions and the machinery every
//! other crate builds on.
//!
//! * [`contracts`]: typed records, releases, artifacts and the C0 contract families (research,
//!   evidence, dataset, wsp, evaluation); JSON Schemas via [`schema`].
//! * [`services`]: synchronous domain service traits injected by the composition root.
//! * [`prefix`]: causal prefix construction and information-isolation checks.
//! * [`views`]: array views handed from curation to the experts, and the [`views::ViewSource`] trait.
//! * [`expert`]: the common [`expert::Expert`] interface around specialised fitting algorithms.
//! * [`analysis`]: the one forecast path shared by `ignis serve` and `ignis infer` (prefix, settings,
//!   artifact validation, write-once analysis store) and [`records`], the run-record registry.
//! * [`registry`]: immutable content-addressed model releases (npz tensors, verified on load).
//! * [`resources`]: resource profiles, the opt-in execution guard and the job/cost ledger.
//! * [`units`], [`hashing`], [`interpret`], [`npz`], [`kernels`], [`batch`].

pub mod analysis;
pub mod batch;
pub mod contracts;
pub mod expert;
pub mod hashing;
pub mod interpret;
pub mod kernels;
pub mod npz;
pub mod prefix;
pub mod records;
pub mod registry;
pub mod resources;
pub mod schema;
pub mod services;
pub mod units;
pub mod views;

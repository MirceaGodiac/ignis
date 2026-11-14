//! IGNIS sealed evaluation and checkpoint laboratory (`ignis eval ...`, `ignis lab ...`).
//!
//! * [`eval`]: vault (deposit/seal/reveal with a reveal log), proper scores (CRPS, coverage, log
//!   density, MAE), censoring-corrected Brier/IBS/calibration for competing events, masked front
//!   metrics, refusal accounting, group-cluster bootstrap (Rayon), compare and report.
//! * [`infer`]: forecasts from the CLI through the same path as the API (`ignis infer forecast`,
//!   `ignis infer forecast-cohort`).
//! * [`lab`]: show/tensor/lineage/diff, probes through the expert hooks, reproducible edits into new
//!   releases (declared transforms and `.npy` tensors with recorded hashes), resume via `ignis fit`.
//!
//! The binary injects the experts and the view source: `eval::run(cmd, &experts, &views)` and
//! `lab::run(cmd, &experts, &views)` return the process exit code.

pub mod access;
pub mod eval;
pub mod infer;
pub mod lab;
pub mod util;

pub use eval::EvalCmd;
pub use infer::InferCmd;
pub use lab::LabCmd;

use ignis_core::contracts::{Cohort, Family, Split};
use ignis_core::views::{View, ViewSource};

/// A view source with no data, for binaries built without the data lane. Probes refuse cleanly.
pub struct NoViews;

impl ViewSource for NoViews {
    fn cohort(&self, cohort_id: &str) -> anyhow::Result<Cohort> {
        anyhow::bail!("no view source in this build (cohort {cohort_id})")
    }
    fn split(&self, split_id: &str) -> anyhow::Result<Split> {
        anyhow::bail!("no view source in this build (split {split_id})")
    }
    fn views(&self, _family: Family, cohort_id: &str) -> anyhow::Result<Vec<View>> {
        anyhow::bail!("no view source in this build (cohort {cohort_id})")
    }
}

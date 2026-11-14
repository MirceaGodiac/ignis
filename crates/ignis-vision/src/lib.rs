//! BASS visual metrology (V3.1 §3.4, §4.4–4.5, §9.1–9.3).
//!
//! Pipeline: decode with container PTS → optional lens undistortion → admitted ROI → segmentation
//! (classical comparator in Rust, or SAM2.1 in the Python sidecar; both strictly causal) →
//! leading-luminous boundary on a fixed transverse grid → homography to the specimen plane →
//! [`ContourObservation`](ignis_core::contracts::ContourObservation) with status and scale
//! uncertainty. Topology changes are reported as status, never smoothed away.
//!
//! Modules: [`video`] (probe, decode, frame cache), [`calibration`], [`segment`], [`sam2`]
//! (sidecar client), [`contour`], [`pipeline`], [`cli`] (`ignis vision …`).

#![allow(clippy::needless_range_loop)]

pub mod calibration;
pub mod cli;
pub mod contour;
pub mod imageio;
pub mod pipeline;
pub mod sam2;
pub mod segment;
pub mod video;

pub use cli::{run, VisionCmd};

pub const METHOD_VERSION: &str = "0.2.0";

/// A locator into a video asset.
pub fn locator(
    sha256: &str,
    path: Option<&str>,
    frame: Option<i64>,
    pts_s: Option<f64>,
) -> ignis_core::contracts::SourceLocator {
    ignis_core::contracts::SourceLocator {
        asset_sha256: sha256.to_string(),
        path: path.map(String::from),
        row: None,
        column: None,
        page: None,
        frame,
        pts_s,
        char_start: None,
        char_end: None,
        ..Default::default()
    }
}

//! IGNIS data lane: NASA PSI sources (read-only connector, manifests, bounded samples), the
//! international adapters, and curation — family ingest, field rules, cohorts, frozen grouped
//! splits, Parquet views ([`views::DataStore`] implements `ignis_core::views::ViewSource`) and the
//! semantic review queue.
//!
//! CLI groups: [`cli::SourcesCmd`] (`ignis sources ...`) and [`cli::DataCmd`] (`ignis data ...`).

pub mod auto;
pub mod cli;
pub mod cohort;
pub mod common;
pub mod dossier;
pub mod families;
pub mod npyrng;
pub mod paths;
pub mod review;
pub mod sources;
pub mod store;
pub mod text;
pub mod views;

pub use cli::{DataCmd, SourcesCmd};
pub use views::DataStore;

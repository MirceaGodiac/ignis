//! Generic training and decision drivers.
//!
//! * [`fit`]: `ignis fit list|plan|run` over any collected `&[Box<dyn Expert>]` and a
//!   `&dyn ViewSource` (the binary supplies both; fixtures are passed as [`Fixture`]s).
//! * [`decide`]: `ignis decide rank|replay` — observation-value ranking (V3.1 chapter 11) through
//!   the `CandidatePredictive` interface; the continuation is never an input.
//!
//! This crate depends on no model crate.

pub mod config;
pub mod decide;
pub mod fit;

use ignis_core::contracts::{Cohort, Family, Split};
use ignis_core::views::{View, ViewSource};

/// A generated archive (e.g. the synthetic FLEX fixture). Never NASA data.
pub struct FixtureData {
    pub source: Box<dyn ViewSource + Send + Sync>,
    pub cohort_id: String,
    pub split_id: String,
}

/// A named fixture builder for one family.
pub struct Fixture {
    pub name: &'static str,
    pub family: Family,
    pub build: fn(n_runs: usize, seed: u64) -> anyhow::Result<FixtureData>,
}

pub fn find_fixture<'a>(
    fixtures: &'a [Fixture],
    name: &str,
    family: Family,
) -> anyhow::Result<&'a Fixture> {
    fixtures
        .iter()
        .find(|f| f.name == name && f.family == family)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no fixture {name:?} for family {}; available: {:?}",
                family.as_str(),
                fixtures
                    .iter()
                    .map(|f| (f.name, f.family.as_str()))
                    .collect::<Vec<_>>()
            )
        })
}

/// Placeholder data store until the data lane's `ViewSource` is wired into the binary.
pub struct NoData;

impl ViewSource for NoData {
    fn cohort(&self, _: &str) -> anyhow::Result<Cohort> {
        anyhow::bail!(
            "no curated data store in this build (data lane not wired); use --fixture synthetic"
        )
    }
    fn split(&self, _: &str) -> anyhow::Result<Split> {
        anyhow::bail!(
            "no curated data store in this build (data lane not wired); use --fixture synthetic"
        )
    }
    fn views(&self, _: Family, _: &str) -> anyhow::Result<Vec<View>> {
        anyhow::bail!(
            "no curated data store in this build (data lane not wired); use --fixture synthetic"
        )
    }
}

pub fn family_of(model: &str) -> Option<Family> {
    match model.split('.').next()? {
        "flex" => Some(Family::Flex),
        "acme_normal" => Some(Family::AcmeNormal),
        "acme_inverse" => Some(Family::AcmeInverse),
        "bass" => Some(Family::Bass),
        _ => None,
    }
}

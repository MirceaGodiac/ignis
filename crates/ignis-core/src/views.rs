//! Training views: the array-shaped hand-off between curation (`ignis-data`) and the experts.
//!
//! Experts never read raw NASA files; they read these views plus the frozen [`Split`]. All values
//! are SI. Missing values are NaN *with* a mask; never zero-filled. `ignis-data` writes them as
//! Parquet under `data/work/views/<cohort-hex>/` and implements [`ViewSource`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::contracts::{
    Cohort, ContourObservation, ControlHistory, EndpointRecord, ExperimentRecord, Family,
    Observation, Split,
};

pub const FLEX_CAUSES: [&str; 2] = ["hot_phase_end", "disruption"];
pub const ACME_CAUSES: [&str; 2] = ["extinction", "operator_termination"];

/// Time origins of the curated views (recorded in every release fitted on them).
pub const FLEX_TIME_ORIGIN: &str =
    "NASA DropletData processed time base (phase entry not adjudicated)";
pub const ACME_TIME_ORIGIN: &str =
    "hot ignition per table 'burn time (sec)' (not adjudicated against logs)";
pub const BASS_TIME_ORIGIN: &str = "video PTS of the mapped main-view recording (attached by the vision lane; not yet adjudicated)";

/// Quality flags that make a measurement invalid (masked, never deleted).
pub const INVALID_FLAGS: [&str; 5] = [
    "non_positive",
    "non_finite",
    "missing",
    "duplicate_timestamp",
    "non_monotonic_time",
];

/// The declared time origin of a family's curated views (the fallback when a run has no control history).
pub fn family_time_origin(family: Family) -> &'static str {
    match family {
        Family::Flex => FLEX_TIME_ORIGIN,
        Family::AcmeNormal | Family::AcmeInverse => ACME_TIME_ORIGIN,
        Family::Bass => BASS_TIME_ORIGIN,
        Family::Other => "unadjudicated_source_origin",
    }
}

/// Whether a stored measurement is usable: a value, and no invalidating flag.
pub fn measurement_valid(o: &Observation) -> bool {
    o.value_si.is_some_and(f64::is_finite)
        && !o
            .quality_flags
            .iter()
            .any(|f| INVALID_FLAGS.contains(&f.as_str()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexTrajectory {
    pub run_id: String,
    pub group_id: String,
    /// Real timestamps since the declared time origin.
    pub t_s: Vec<f64>,
    pub available_at_s: Vec<f64>,
    /// Liquid diameter (m); NaN where flagged.
    pub d_m: Vec<f64>,
    /// Flagged / non-finite points are false, never deleted silently.
    pub valid: Vec<bool>,
    /// Prospective conditions only (SI).
    pub conditions: BTreeMap<String, f64>,
    pub censoring: String,
    pub t_last_active_s: Option<f64>,
    pub t_first_terminal_s: Option<f64>,
    pub allowed_causes: Vec<String>,
    /// Review established that the event preceded exit from the support region.
    #[serde(default)]
    pub event_before_support_exit: Option<bool>,
    pub time_origin: String,
    pub evidence_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcmeEventRow {
    pub run_id: String,
    /// Session (test day) for frailties.
    pub group_id: String,
    /// "normal" | "inverse"
    pub configuration: String,
    /// Role-aware pre-event features, SI.
    pub features: BTreeMap<String, f64>,
    /// Established-phase entry (left truncation).
    pub t_entry_s: f64,
    pub censoring: String,
    pub t_lower_s: Option<f64>,
    pub t_upper_s: Option<f64>,
    pub allowed_causes: Vec<String>,
    /// Deterministic operator stop time when the protocol fixes one.
    pub scheduled_stop_s: Option<f64>,
    pub time_origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BassSequence {
    pub run_id: String,
    /// Specimen / session.
    pub group_id: String,
    /// (T,)
    pub t_s: Vec<f64>,
    /// (S,) common transverse grid.
    pub s_m: Vec<f64>,
    /// (T, S) row-major front position; NaN where unresolved.
    pub f_m: Vec<f64>,
    /// (T, S) row-major mask.
    pub valid: Vec<bool>,
    /// Per-frame contour status.
    pub status: Vec<String>,
    pub conditions: BTreeMap<String, f64>,
    pub geometry: BTreeMap<String, f64>,
    /// Control stages known for the run (flow changes), seconds since the time origin.
    #[serde(default)]
    pub control_changes_s: Vec<f64>,
    #[serde(default)]
    pub scale_uncertainty_rel: Vec<f64>,
    pub time_origin: String,
}

impl BassSequence {
    pub fn n_t(&self) -> usize {
        self.t_s.len()
    }
    pub fn n_s(&self) -> usize {
        self.s_m.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum View {
    Flex(FlexTrajectory),
    Acme(AcmeEventRow),
    Bass(BassSequence),
}

impl View {
    pub fn run_id(&self) -> &str {
        match self {
            View::Flex(v) => &v.run_id,
            View::Acme(v) => &v.run_id,
            View::Bass(v) => &v.run_id,
        }
    }
    pub fn group_id(&self) -> &str {
        match self {
            View::Flex(v) => &v.group_id,
            View::Acme(v) => &v.group_id,
            View::Bass(v) => &v.group_id,
        }
    }
    /// Count used for cost estimates: valid points for trajectories/fronts, one per event row.
    pub fn n_observations(&self) -> usize {
        match self {
            View::Flex(v) => v.valid.iter().filter(|b| **b).count(),
            View::Acme(_) => 1,
            View::Bass(v) => v.valid.iter().filter(|b| **b).count(),
        }
    }
}

/// One run's stored records: what the prefix builder and the deposit step read. The endpoint and
/// the observations after a cutoff are held here for the evaluator and for the deposit step; only
/// [`crate::analysis::build_run_prefix`] turns an archive into something a model sees.
#[derive(Debug, Clone)]
pub struct RunArchive {
    pub record: ExperimentRecord,
    /// Sorted by (time_s, id), like `build_prefix`.
    pub observations: Vec<Observation>,
    pub contours: Vec<ContourObservation>,
    pub control: Option<ControlHistory>,
    pub endpoint: Option<EndpointRecord>,
    pub time_origin: String,
}

/// Where experts get their data. Implemented by `ignis-data`; synthetic fixtures implement it too.
pub trait ViewSource {
    fn cohort(&self, cohort_id: &str) -> anyhow::Result<Cohort>;
    fn split(&self, split_id: &str) -> anyhow::Result<Split>;
    fn views(&self, family: Family, cohort_id: &str) -> anyhow::Result<Vec<View>>;
    /// Stored records of every run of a family (observations, controls, endpoints).
    fn run_archives(&self, family: Family) -> anyhow::Result<Vec<RunArchive>> {
        anyhow::bail!(
            "this data source holds no run records for family {}",
            family.as_str()
        )
    }
    /// The stored records of one run, if it exists in this family.
    fn run_archive(&self, family: Family, run_id: &str) -> anyhow::Result<Option<RunArchive>> {
        Ok(self
            .run_archives(family)?
            .into_iter()
            .find(|a| a.record.id == run_id))
    }
}

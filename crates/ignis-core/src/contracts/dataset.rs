//! Data handed from the data lane to models and evaluation: dataset bundles and visual observation
//! bundles. Semantic owner: Raul (Iancu and Tudor review numeric consumption).
//!
//! Bundles reuse the existing records ([`Cohort`](super::Cohort), [`Split`](super::Split),
//! [`Observation`](super::Observation), [`ContourObservation`](super::ContourObservation)); numeric
//! tables travel as Parquet artifacts, not as JSON arrays. A `dataset_id` is never a local path.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::{
    ensure, ArtifactRef, ContractError, CorpusSnapshotId, DatasetId, Finite, Origin, PageCursor,
    RunId, SourceId, Timestamp, VisualBundleId,
};
use super::{
    ContourStatus, Derivation, EvidenceClass, Family, Measured, Partition, Quantity, RawValue,
    Resolution, SourceLocator,
};

/// Wire tag of dataset, visual observation and conditions documents. Wave K1 writes `dataset.v2`;
/// `dataset.v1` (C0) documents are still read, and keep their content digests because a
/// document is re-serialized with the tag it was read with. New document types require v2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub enum DatasetSchema {
    /// C0. Still read; new documents are written as v2.
    #[serde(rename = "dataset.v1")]
    V1,
    #[default]
    #[serde(rename = "dataset.v2")]
    V2,
}

impl DatasetSchema {
    fn require_v2(self, contract: &'static str) -> Result<(), ContractError> {
        ensure(self == DatasetSchema::V2, contract, || {
            format!("{contract} needs schema_version dataset.v2")
        })
    }
}

/// Who may receive a bundle. Builder bundles (models, lab, authoring) never contain test outcomes;
/// those live in a separate evaluator bundle that is not mounted in builder workspaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BundleAudience {
    Builder,
    Evaluator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColumnRole {
    RunId,
    /// Seconds since the bundle's `time_origin`.
    Time,
    /// Time the value became available to an observer (seconds since `time_origin`).
    AvailableAt,
    Value,
    Uncertainty,
    Condition,
    Flag,
    /// Serialized `SourceLocator` / evidence id of the row.
    Locator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColumnSpec {
    pub name: String,
    pub role: ColumnRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Quantity>,
    /// SI unit of the stored values; display conversions never change it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub nullable: bool,
}

/// One columnar view of the bundle (Parquet artifact).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DatasetView {
    pub name: String,
    pub artifact: ArtifactRef,
    pub columns: Vec<ColumnSpec>,
    pub row_count: u64,
}

/// A curated dataset for one family/target: cohort, split and the partitions it contains, source
/// manifest hash, time origin, columnar views with units, a quality report and the inclusion
/// criteria. Content-addressed by [`DatasetBundle::content_digest`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DatasetBundle {
    pub schema_version: DatasetSchema,
    pub dataset_id: DatasetId,
    pub family: Family,
    pub audience: BundleAudience,
    pub cohort_id: String,
    pub split_id: String,
    pub partitions: BTreeSet<Partition>,
    pub source_manifest_hash: String,
    /// Event that defines t = 0 for every time column (e.g. `ignition`, `igniter_retracted`).
    pub time_origin: String,
    pub views: Vec<DatasetView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_report: Option<ArtifactRef>,
    #[serde(default)]
    pub inclusion_criteria: BTreeMap<String, String>,
    pub origin: Origin,
    pub created_at: Timestamp,
}

impl DatasetBundle {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "DatasetBundle";
        ensure(!self.partitions.is_empty(), c, || {
            "a bundle contains at least one partition".into()
        })?;
        ensure(
            self.audience == BundleAudience::Evaluator
                || !self.partitions.contains(&Partition::Test),
            c,
            || "a builder bundle cannot contain the test partition".into(),
        )?;
        let mut names = BTreeSet::new();
        for v in &self.views {
            ensure(names.insert(&v.name), c, || {
                format!("duplicate view {}", v.name)
            })?;
            v.artifact.validate()?;
            let mut cols = BTreeSet::new();
            for col in &v.columns {
                ensure(cols.insert(&col.name), c, || {
                    format!("{}: duplicate column {}", v.name, col.name)
                })?;
                if let Some(q) = col.quantity {
                    ensure(col.unit.as_deref() == Some(q.unit_si()), c, || {
                        format!(
                            "{}.{}: {q:?} is stored in {}",
                            v.name,
                            col.name,
                            q.unit_si()
                        )
                    })?;
                }
                if matches!(col.role, ColumnRole::Time | ColumnRole::AvailableAt) {
                    ensure(col.unit.as_deref() == Some("s"), c, || {
                        format!("{}.{}: time columns are seconds", v.name, col.name)
                    })?;
                }
            }
        }
        Ok(())
    }

    /// Identity of the bundle's content (everything but `created_at`).
    pub fn content_digest(&self) -> super::Digest {
        super::Digest::of_json_excluding(self, &["created_at"])
    }
}

// ---------------------------------------------------------------- visual observations

/// Whether contour coordinates are metric. `pixel_only` never becomes metric downstream (a chart
/// axis or a smarter model does not calibrate a camera).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnitsStatus {
    Metric,
    PixelOnly,
    Normalized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimestampBasis {
    ContainerPts,
    NominalFrameRate,
    Telemetry,
}

/// Which run a clip shows; uncertain identity is explicit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunIdentity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<RunId>,
    pub resolution: Resolution,
}

/// Content contract of metric contour records (`ContourObservation` JSONL, metres).
pub const METRIC_CONTOURS: &str = "ContourObservation";
/// Content contract of image-space contour records ([`ImageContourObservation`] JSONL).
pub const IMAGE_CONTOURS: &str = "ImageContourObservation";

/// Contours extracted from one video, with camera, time basis and calibration status. Metric
/// contours are `ContourObservation` JSONL (meters); `pixel_only` / `normalized` contours are
/// [`ImageContourObservation`] JSONL and can never be read as `ContourObservation`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VisualObservationBundle {
    pub schema_version: DatasetSchema,
    pub bundle_id: VisualBundleId,
    pub video_source_id: SourceId,
    pub run: RunIdentity,
    pub camera_id: String,
    pub timestamp_basis: TimestampBasis,
    pub time_origin: String,
    pub units_status: UnitsStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_id: Option<String>,
    /// Extraction method and version (segmenter, contour sampler).
    pub method: String,
    pub contours: ArtifactRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_report: Option<ArtifactRef>,
    pub origin: Origin,
    pub created_at: Timestamp,
}

impl VisualObservationBundle {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "VisualObservationBundle";
        self.contours.validate()?;
        let metric_contract = self.contours.contract.as_deref() == Some("ContourObservation");
        match self.units_status {
            UnitsStatus::Metric => {
                ensure(self.calibration_id.is_some(), c, || {
                    "metric contours need a calibration".into()
                })?;
                ensure(metric_contract, c, || {
                    "metric contours are ContourObservation records".into()
                })
            }
            _ => ensure(
                self.contours.contract.as_deref() == Some(IMAGE_CONTOURS),
                c,
                || {
                    format!(
                        "{:?} contours are {IMAGE_CONTOURS} records (ContourObservation is metres)",
                        self.units_status
                    )
                },
            ),
        }
    }

    /// Guard for consumers that need metres on a known run (BASS forecasting, metric plots).
    pub fn require_metric(&self) -> Result<&RunId, ContractError> {
        self.validate()?;
        ensure(
            self.units_status == UnitsStatus::Metric,
            "VisualObservationBundle",
            || format!("contours are {:?}, not metric", self.units_status),
        )?;
        match (&self.run.run_id, self.run.resolution) {
            (Some(r), Resolution::Resolved) => Ok(r),
            _ => Err(ContractError::new(
                "VisualObservationBundle",
                "the clip's run identity is not resolved",
            )),
        }
    }
}

/// One frame's leading-edge contour in **image** coordinates, the record type of `pixel_only` and
/// `normalized` [`VisualObservationBundle`]s. It is never metres: a consumer that needs metres
/// requires a metric bundle ([`VisualObservationBundle::require_metric`]), whose records are
/// `ContourObservation`. The run identity is the bundle's ([`RunIdentity`]), not repeated here.
///
/// `s` is the transverse grid (image units, e.g. pixel rows), `f` the leading-edge coordinate along
/// the propagation axis at each `s` (absent = no edge at that grid line). Times are seconds since the
/// bundle's `time_origin`; `pts_s` is the container presentation time of `frame` (0-based).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageContourObservation {
    pub id: String,
    pub frame: i64,
    pub pts_s: Finite,
    pub time_s: Finite,
    pub available_at_s: Finite,
    /// `pixel_only` or `normalized`; never `metric`.
    pub units: UnitsStatus,
    pub s: Vec<Finite>,
    pub f: Vec<Option<Finite>>,
    pub status: ContourStatus,
    /// Frames the segmentation mask of this frame depended on (all `<= frame`: causal).
    pub mask_dependency_frames: Vec<i64>,
    pub evidence_class: EvidenceClass,
    pub locator: SourceLocator,
    pub derivation: Derivation,
    pub origin: Origin,
}

impl ImageContourObservation {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ImageContourObservation";
        ensure(self.units != UnitsStatus::Metric, c, || {
            "image contours are not metres (metric contours are ContourObservation)".into()
        })?;
        ensure(self.s.len() == self.f.len(), c, || {
            "s and f must align".into()
        })?;
        ensure(self.frame >= 0, c, || "frame is a 0-based index".into())?;
        ensure(self.locator.frame == Some(self.frame), c, || {
            "the locator points at the record's frame".into()
        })?;
        ensure(
            self.mask_dependency_frames.iter().all(|&f| f <= self.frame),
            c,
            || "the mask depends on a future frame (non-causal)".into(),
        )?;
        ensure(self.available_at_s.get() >= self.time_s.get(), c, || {
            "available before observed".into()
        })?;
        self.locator.validate()
    }
}

// ---------------------------------------------------------------- experimental conditions

/// A condition the challenge and the graph join on. Numeric keys are stored in the SI unit of their
/// [`Quantity`]; the others are categorical text (`n-heptane`, `flat sheet`, `CIR/MDCA`). The
/// catalog is closed: a new key is a contract change.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ConditionKey {
    OxygenMoleFraction,
    Pressure,
    FlowVelocity,
    GravityLevel,
    AmbientTemperature,
    Fuel,
    Diluent,
    SampleGeometry,
    Hardware,
    Diagnostics,
}

impl ConditionKey {
    /// The quantity of a numeric key; `None` for a categorical one.
    pub fn quantity(self) -> Option<Quantity> {
        use ConditionKey::*;
        match self {
            OxygenMoleFraction => Some(Quantity::MoleFraction),
            Pressure => Some(Quantity::Pressure),
            FlowVelocity => Some(Quantity::FlowVelocity),
            GravityLevel => Some(Quantity::GravityLevel),
            AmbientTemperature => Some(Quantity::Temperature),
            Fuel | Diluent | SampleGeometry | Hardware | Diagnostics => None,
        }
    }
}

fn check_measured(c: &'static str, key: ConditionKey, m: &Measured) -> Result<(), ContractError> {
    let l = m.locator.as_ref().ok_or_else(|| {
        ContractError::new(
            c,
            format!("{key:?}: a value keeps the locator of its original"),
        )
    })?;
    l.validate()?;
    match key.quantity() {
        Some(q) => {
            ensure(matches!(m.raw_value, Some(RawValue::Number(_))), c, || {
                format!("{key:?}: a numeric condition keeps its raw number")
            })?;
            ensure(m.value_si.is_some_and(f64::is_finite), c, || {
                format!("{key:?}: missing or non-finite SI value")
            })?;
            ensure(m.unit_si.as_deref() == Some(q.unit_si()), c, || {
                format!("{key:?} is stored in {}", q.unit_si())
            })
        }
        None => ensure(
            matches!(&m.raw_value, Some(RawValue::Text(t)) if !t.trim().is_empty())
                && m.value_si.is_none(),
            c,
            || format!("{key:?}: a categorical condition is text without an SI value"),
        ),
    }
}

/// One cell of the conditions matrix. A gap is a status with a reason, never an absent cell or a
/// zero. Conditions are experimental inputs, not outcomes; durations and endpoints never appear here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConditionCell {
    /// Raw value, raw unit, SI value and locator of the original.
    Value {
        key: ConditionKey,
        value: Measured,
    },
    /// Sources disagree; every candidate is kept.
    Conflicting {
        key: ConditionKey,
        candidates: Vec<Measured>,
        reason: String,
    },
    Missing {
        key: ConditionKey,
        reason: String,
    },
    NotApplicable {
        key: ConditionKey,
        reason: String,
    },
}

impl ConditionCell {
    pub fn key(&self) -> ConditionKey {
        match self {
            ConditionCell::Value { key, .. }
            | ConditionCell::Conflicting { key, .. }
            | ConditionCell::Missing { key, .. }
            | ConditionCell::NotApplicable { key, .. } => *key,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ConditionCell";
        match self {
            ConditionCell::Value { key, value } => check_measured(c, *key, value),
            ConditionCell::Conflicting {
                key,
                candidates,
                reason,
            } => {
                ensure(candidates.len() >= 2, c, || {
                    format!("{key:?}: a conflict has at least two candidates")
                })?;
                ensure(!reason.trim().is_empty(), c, || {
                    "a conflict says why".into()
                })?;
                candidates
                    .iter()
                    .try_for_each(|m| check_measured(c, *key, m))
            }
            ConditionCell::Missing { key, reason }
            | ConditionCell::NotApplicable { key, reason } => {
                ensure(!reason.trim().is_empty(), c, || {
                    format!("{key:?}: a gap says why")
                })
            }
        }
    }
}

/// The conditions of one run (or of an investigation-level test point when the run is unresolved).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConditionsRow {
    pub investigation_id: String,
    pub run: RunIdentity,
    pub cells: Vec<ConditionCell>,
}

/// What an investigation tested for one key, over its runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum EnvelopeEntry {
    /// Numeric key: closed range of the measured SI values.
    Range {
        key: ConditionKey,
        min_si: Finite,
        max_si: Finite,
        unit_si: String,
        runs: u64,
    },
    /// Categorical key: the values tested.
    Values {
        key: ConditionKey,
        values: BTreeSet<String>,
        runs: u64,
    },
    Missing {
        key: ConditionKey,
        reason: String,
    },
}

impl EnvelopeEntry {
    fn key(&self) -> ConditionKey {
        match self {
            EnvelopeEntry::Range { key, .. }
            | EnvelopeEntry::Values { key, .. }
            | EnvelopeEntry::Missing { key, .. } => *key,
        }
    }
}

/// The tested envelope of one investigation: what a habitat atmosphere is compared against. Outside
/// it is "outside the tested envelope", never an extrapolation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvestigationEnvelope {
    pub investigation_id: String,
    pub runs: u64,
    pub entries: Vec<EnvelopeEntry>,
}

impl InvestigationEnvelope {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "InvestigationEnvelope";
        let mut keys = BTreeSet::new();
        for e in &self.entries {
            let k = e.key();
            ensure(keys.insert(k), c, || format!("{k:?} listed twice"))?;
            match e {
                EnvelopeEntry::Range {
                    min_si,
                    max_si,
                    unit_si,
                    runs,
                    ..
                } => {
                    let q = k.quantity().ok_or_else(|| {
                        ContractError::new(c, format!("{k:?} is categorical, not a range"))
                    })?;
                    ensure(unit_si == q.unit_si(), c, || {
                        format!("{k:?} is stored in {}", q.unit_si())
                    })?;
                    ensure(min_si.get() <= max_si.get(), c, || {
                        format!("{k:?}: min above max")
                    })?;
                    ensure(*runs >= 1 && *runs <= self.runs, c, || {
                        format!("{k:?}: run count outside the investigation's")
                    })?;
                }
                EnvelopeEntry::Values { values, runs, .. } => {
                    ensure(k.quantity().is_none(), c, || {
                        format!("{k:?} is numeric: give a range")
                    })?;
                    ensure(!values.is_empty(), c, || format!("{k:?}: no values"))?;
                    ensure(*runs >= 1 && *runs <= self.runs, c, || {
                        format!("{k:?}: run count outside the investigation's")
                    })?;
                }
                EnvelopeEntry::Missing { reason, .. } => {
                    ensure(!reason.trim().is_empty(), c, || {
                        format!("{k:?}: a gap says why")
                    })?;
                }
            }
        }
        Ok(())
    }
}

/// A filter on one key. A filter keeps rows whose cell is a value inside it; rows whose cell is
/// missing or conflicting are kept only with `ConditionsQuery::include_unknown`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConditionFilter {
    pub key: ConditionKey,
    /// Numeric keys, SI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_si: Option<Finite>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_si: Option<Finite>,
    /// Categorical keys.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub any_of: BTreeSet<String>,
}

/// `conditions_query`: per-run conditions and, on request, each investigation's tested envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConditionsQuery {
    pub schema_version: DatasetSchema,
    /// Empty = every investigation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub investigation_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run_ids: Vec<RunId>,
    /// Columns; empty = every key.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<ConditionKey>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<ConditionFilter>,
    #[serde(default)]
    pub include_unknown: bool,
    #[serde(default)]
    pub include_envelopes: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<PageCursor>,
}

impl ConditionsQuery {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ConditionsQuery";
        self.schema_version.require_v2(c)?;
        ensure(self.page_size != Some(0), c, || {
            "page_size must be positive".into()
        })?;
        for f in &self.filters {
            let k = f.key;
            let bounds = f.min_si.is_some() || f.max_si.is_some();
            if k.quantity().is_some() {
                ensure(bounds && f.any_of.is_empty(), c, || {
                    format!("{k:?} filters by min_si/max_si")
                })?;
                if let (Some(a), Some(b)) = (f.min_si, f.max_si) {
                    ensure(a.get() <= b.get(), c, || format!("{k:?}: min above max"))?;
                }
            } else {
                ensure(!bounds && !f.any_of.is_empty(), c, || {
                    format!("{k:?} filters by any_of")
                })?;
            }
        }
        Ok(())
    }
}

/// A page of the conditions matrix. Every row has exactly one cell per key in `keys`, so a gap is
/// always visible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConditionsTable {
    pub schema_version: DatasetSchema,
    pub keys: Vec<ConditionKey>,
    pub rows: Vec<ConditionsRow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub envelopes: Vec<InvestigationEnvelope>,
    /// Sources the table was built from (PSI experimental tables, extracted passages).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corpus_snapshot: Option<CorpusSnapshotId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<PageCursor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_count: Option<u64>,
    pub origin: Origin,
}

impl ConditionsTable {
    pub fn validate(&self) -> Result<(), ContractError> {
        let c = "ConditionsTable";
        self.schema_version.require_v2(c)?;
        let keys: BTreeSet<ConditionKey> = self.keys.iter().copied().collect();
        ensure(!keys.is_empty() && keys.len() == self.keys.len(), c, || {
            "keys are non-empty and distinct".into()
        })?;
        ensure(
            self.total_count.is_none_or(|t| t >= self.rows.len() as u64),
            c,
            || "total_count below row count".into(),
        )?;
        for r in &self.rows {
            ensure(!r.investigation_id.trim().is_empty(), c, || {
                "empty investigation id".into()
            })?;
            let got: Vec<ConditionKey> = r.cells.iter().map(ConditionCell::key).collect();
            let got_set: BTreeSet<_> = got.iter().copied().collect();
            ensure(got.len() == got_set.len() && got_set == keys, c, || {
                format!(
                    "{} {:?}: one cell per key, gaps as statuses",
                    r.investigation_id, r.run.run_id
                )
            })?;
            r.cells.iter().try_for_each(ConditionCell::validate)?;
        }
        let mut invs = BTreeSet::new();
        for e in &self.envelopes {
            ensure(invs.insert(&e.investigation_id), c, || {
                format!("envelope of {} listed twice", e.investigation_id)
            })?;
            e.validate()?;
        }
        Ok(())
    }
}

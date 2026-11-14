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
    ensure, schema_tag, ArtifactRef, ContractError, DatasetId, Finite, Origin, RunId, SourceId,
    Timestamp, VisualBundleId,
};
use super::{
    ContourStatus, Derivation, EvidenceClass, Family, Partition, Quantity, Resolution,
    SourceLocator,
};

schema_tag!(
    /// Wire tag of dataset and visual observation bundles.
    DatasetSchema = "dataset.v1"
);

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
    #[serde(default)]
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
    #[serde(default)]
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

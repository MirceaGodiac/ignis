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
    ensure, schema_tag, ArtifactRef, ContractError, DatasetId, Origin, RunId, SourceId, Timestamp,
    VisualBundleId,
};
use super::{Family, Partition, Quantity, Resolution};

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

/// Contours extracted from one video, with camera, time basis and calibration status. Metric
/// contours are `ContourObservation` JSONL (meters); other unit states use another contract and can
/// never be read as `ContourObservation`.
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
            _ => ensure(!metric_contract, c, || {
                "ContourObservation is metres; pixel/normalized contours use another contract"
                    .into()
            }),
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

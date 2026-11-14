//! IGNIS shared contracts — the single authority for schemas (version [`SCHEMA_VERSION`]).
//!
//! `ignis schemas export` writes one JSON Schema per exported type to `schemas/` (consumed by the
//! web client and the SAM2 sidecar). Historically these mirrored Python pydantic models; the rules
//! below are kept: Every Python `Strict` model forbids extra fields, so
//! every struct here is `deny_unknown_fields`. Required-but-nullable Python fields (`x: T | None`
//! without a default) use [`nullable`] so that a *missing* key is rejected while `null` is accepted;
//! Python defaults are reproduced with `#[serde(default = ...)]`. `tests/core.rs` checks that the
//! committed `schemas/*.json` equal the exported schemas; `tests/contracts.rs` checks the C0 fixtures.
//!
//! Datetimes are kept as the exact RFC 3339 strings Python produced (identity is hash-based and the
//! runtime never does calendar arithmetic on them).
//!
//! # C0 contract families
//!
//! The records in this file are the scientific data model (schema family [`SCHEMA_VERSION`]). The C0
//! surface added for the four parallel lanes lives in submodules, one semantic owner per file,
//! re-exported here so every consumer imports `ignis_core::contracts::*`:
//!
//! | module | wire tag | semantic owner |
//! |---|---|---|
//! | [`common`] | none | shared: ids, time, finite numbers, origin, scope, artifacts, errors |
//! | [`research`] | `research.v1`, `tools.v1` | Mircea: episodes, events, jobs, tool results, answers, tool catalog |
//! | [`evidence`] | `evidence.v1` | Raul: evidence refs, search, read, provenance links, wiki |
//! | [`dataset`] | `dataset.v1` | Raul: dataset and visual observation bundles |
//! | [`wsp`] | `wsp.v1` | Iancu: model spec, literal patch, build manifest, forecast request |
//! | [`evaluation`] | `evaluation.v1` | Tudor: evaluation request and result |
//!
//! Conventions of the new surface (`docs/implementation/CONTRACTS.md`): every struct is
//! `deny_unknown_fields`; optional fields are omitted when absent and absence never means zero;
//! numbers that must be finite use [`common::Finite`]; ids are typed newtypes; every top-level
//! document carries its family `schema_version` tag, so a payload of another family or version fails
//! to parse instead of being misread.

pub mod common;
pub mod dataset;
pub mod evaluation;
pub mod evidence;
pub mod research;
pub mod wsp;

pub use common::*;
pub use dataset::*;
pub use evaluation::*;
pub use evidence::*;
pub use research::*;
pub use wsp::*;

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// Version of the scientific data model in this file. 2.1.0: `TrainedOn` has no implicit default and
/// gained `not_trained`; `ModelRelease.construction`; `QualificationState::Experimental`;
/// `Lineage::LiteralAuthoring`; optional `SourceLocator` fields (parse view, page label, bbox,
/// table). Legacy 2.0.0 release files stay readable through [`crate::registry::parse_release`].
pub const SCHEMA_VERSION: &str = "2.1.0";

pub type JsonMap = serde_json::Map<String, serde_json::Value>;

/// Deserialize a field that Python declares as required but nullable (`T | None`, no default).
pub fn nullable<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d)
}

/// What a `Source.id` hashes: the fetched bytes, a listing entry (url+size+provider checksum, not yet
/// fetched) or a provider metadata record (access-gated). Only `Bytes` proves content identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IdBasis {
    #[default]
    Bytes,
    Listing,
    MetadataRecord,
}

fn yes() -> bool {
    true
}

// ---------------------------------------------------------------- enums

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Flex,
    AcmeNormal,
    AcmeInverse,
    Bass,
    Other,
}

impl Family {
    pub fn as_str(self) -> &'static str {
        match self {
            Family::Flex => "flex",
            Family::AcmeNormal => "acme_normal",
            Family::AcmeInverse => "acme_inverse",
            Family::Bass => "bass",
            Family::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    #[default]
    OriginalObservation,
    SourceDerived,
    IgnisDerived,
    ModelOutput,
    Synthetic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Deterministic,
    Numerical,
    Semantic,
    HumanReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    #[default]
    Resolved,
    UnresolvedConflict,
    Unknown,
    Quarantined,
    NotApplicable,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Quantity {
    LiquidDiameter,
    VisibleFlameDiameter,
    UvFlameDiameter,
    DropletPosition,
    Pressure,
    MoleFraction,
    Temperature,
    MassFlow,
    VolumeFlow,
    StdVolumeFlow,
    FrontContour,
    FrontPosition,
    SpreadRate,
    FlowVelocity,
    GravityLevel,
}

impl Quantity {
    /// SI unit fixed by `ignis.contracts.quantities.SPECS`.
    pub fn unit_si(self) -> &'static str {
        use Quantity::*;
        match self {
            LiquidDiameter | VisibleFlameDiameter | UvFlameDiameter | DropletPosition
            | FrontContour | FrontPosition => "m",
            Pressure => "Pa",
            MoleFraction | GravityLevel => "1",
            Temperature => "K",
            MassFlow => "kg/s",
            VolumeFlow => "m3/s",
            StdVolumeFlow => "Sm3/s",
            SpreadRate | FlowVelocity => "m/s",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContourStatus {
    SingleValued,
    Multivalued,
    Detached,
    Occluded,
    OutOfField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Censoring {
    Event,
    Interval,
    Right,
    ScheduledStop,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QualificationState {
    SourceOnly,
    MeasurementQualified,
    #[default]
    FitOnly,
    /// Executes, not qualified by any evaluation. Every literal (WSP) release is created in this
    /// state; qualification is recorded by an `EvaluationResult`, never by rewriting the release.
    Experimental,
    ForecastQualified,
    EventQualified,
    AcquisitionEvaluated,
    Unsupported,
    InsufficientInference,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Lineage {
    #[default]
    InitialFit,
    /// Root of a literal weight-space-programming release (W0): tensors come from a
    /// `HandwrittenWeightPatch`, not from a fit. Literal revisions (W1) are `WeightEdit` children.
    LiteralAuthoring,
    WeightEdit,
    ArchitectureChange,
    FineTune,
    ObjectiveChange,
    PreprocessingChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    DatasetFile,
    Publication,
    Video,
    Protocol,
    Calibration,
    Webpage,
    Archive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RightsStatus {
    Verified,
    #[default]
    Unverified,
    Restricted,
    Noncommercial,
}

/// What data conventionally trained the release's parameters. There is deliberately no default:
/// schema 2.0.0 defaulted to `nasa`, which labels any release that omits the field (a handwritten
/// one, say) as NASA-trained. Writers must state it. Legacy files without the field are read by
/// [`crate::registry::parse_release`] with their old meaning, and only when they carry no
/// `construction` record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrainedOn {
    Nasa,
    Synthetic,
    Pretrained,
    /// No conventional training: the values were written literally (WSP). Requires a literal
    /// `ModelRelease.construction` manifest.
    NotTrained,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    PrefixForecast,
    EventProbabilities,
    Acquisition,
    Comparison,
    Metrology,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportGate {
    Semantic,
    Measurement,
    Condition,
    Numerical,
    Horizon,
    Passed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DecisionPolicy {
    Voi,
    Random,
    Uniform,
    MaxVariance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SplitPartition {
    Validation,
    Test,
}

// ---------------------------------------------------------------- provenance

/// `Measured.raw_value` / `Observation.raw_value`: `float | str | None`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RawValue {
    Number(f64),
    Text(String),
}

/// Where a value or passage sits inside one asset. A single locator type serves observations,
/// chunks, calibrations and evidence references.
///
/// Conventions fixed in C0 and checked by [`SourceLocator::validate`]:
/// * `asset_sha256` is the bare 64-hex digest of the asset bytes.
/// * `page` is the **physical** 1-based page index; the printed label goes in `page_label`.
/// * `char_start`/`char_end` are half-open offsets into the text of `parse_view_id`, counted in
///   `offset_unit`. Offsets without a parse view and unit are legacy and not citable.
/// * `bbox` is `[x0, y0, x1, y1]` in fractions of the displayed page (after its /Rotate), origin
///   top-left, x right, y down.
/// * `table_id` scopes `row`/`column` to a table inside the asset (CSV rows otherwise).
///
/// Fields added in 2.1.0 are omitted when absent, so the serialized form of older locators, and
/// therefore every prefix `input_hash` that embeds them, is unchanged.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceLocator {
    pub asset_sha256: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub row: Option<i64>,
    #[serde(default)]
    pub column: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub frame: Option<i64>,
    #[serde(default)]
    pub pts_s: Option<f64>,
    #[serde(default)]
    pub char_start: Option<i64>,
    #[serde(default)]
    pub char_end: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_view_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset_unit: Option<OffsetUnit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[f64; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table_id: Option<String>,
}

/// Unit of `SourceLocator.char_start/char_end`. A Unicode-scalar offset is not a byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OffsetUnit {
    Utf8Byte,
    UnicodeScalar,
    Utf16CodeUnit,
}

impl SourceLocator {
    /// Coherence checks for locators used as citations. Stored observation locators are not
    /// re-validated retroactively.
    pub fn validate(&self) -> Result<(), ContractError> {
        let err = |m: &str| Err(ContractError::new("SourceLocator", m));
        if !crate::hashing::is_digest(&self.asset_sha256) || self.asset_sha256.contains(':') {
            return err("asset_sha256 must be 64 lowercase hex characters");
        }
        if self.page.is_some_and(|p| p < 1) {
            return err("page is the physical 1-based page index");
        }
        match (self.char_start, self.char_end) {
            (None, None) => {}
            (Some(a), Some(b)) if 0 <= a && a <= b => {
                if self.parse_view_id.is_none() || self.offset_unit.is_none() {
                    return err("character offsets need parse_view_id and offset_unit");
                }
            }
            _ => return err("char_start/char_end must both be present with 0 <= start <= end"),
        }
        if let Some([x0, y0, x1, y1]) = self.bbox {
            if self.page.is_none() {
                return err("bbox needs a page");
            }
            let unit = |v: f64| v.is_finite() && (0.0..=1.0).contains(&v);
            if !(unit(x0) && unit(y0) && unit(x1) && unit(y1) && x0 < x1 && y0 < y1) {
                return err("bbox is [x0, y0, x1, y1] in [0, 1] with x0 < x1 and y0 < y1");
            }
        }
        if self.frame.is_some_and(|f| f < 0) || self.row.is_some_and(|r| r < 0) {
            return err("frame and row are non-negative");
        }
        if self.pts_s.is_some_and(|t| !(t.is_finite() && t >= 0.0)) {
            return err("pts_s must be finite and non-negative");
        }
        if self.table_id.is_some() && self.row.is_none() && self.column.is_none() {
            return err("table_id needs a row or a column");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Derivation {
    pub method: Method,
    pub method_id: String,
    pub method_version: String,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Measured {
    #[serde(deserialize_with = "nullable")]
    #[schemars(required)]
    pub raw_value: Option<RawValue>,
    #[serde(default)]
    pub raw_unit: Option<String>,
    #[serde(default)]
    pub value_si: Option<f64>,
    #[serde(default)]
    pub unit_si: Option<String>,
    #[serde(default)]
    pub uncertainty_si: Option<f64>,
    #[serde(default)]
    pub uncertainty_reason: Option<String>,
    #[serde(default)]
    pub evidence_class: EvidenceClass,
    #[serde(default)]
    pub locator: Option<SourceLocator>,
    #[serde(default)]
    pub derivation: Option<Derivation>,
    #[serde(default)]
    pub resolution: Resolution,
    #[serde(default)]
    pub flags: Vec<String>,
}

// ---------------------------------------------------------------- records

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rights {
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub status: RightsStatus,
    #[serde(default)]
    pub statement_locator: Option<SourceLocator>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    #[serde(default)]
    pub id_basis: IdBasis,
    pub kind: SourceKind,
    #[serde(default)]
    pub investigation_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub canonical_url: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub file_version: Option<String>,
    #[serde(default)]
    pub investigation_version: Option<String>,
    #[serde(default)]
    pub provider_version: Option<String>,
    #[serde(default)]
    pub doi: Option<String>,
    #[serde(default)]
    pub ntrs_id: Option<String>,
    #[serde(default)]
    pub size_bytes: Option<i64>,
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    pub retrieved_at: Option<String>,
    #[serde(default)]
    pub rights: Rights,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Alias {
    pub name: String,
    pub scheme: String,
    #[serde(default)]
    pub locator: Option<SourceLocator>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExperimentRecord {
    pub id: String,
    pub investigation_id: String,
    pub family: Family,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub specimen_id: Option<String>,
    #[serde(default)]
    pub aliases: Vec<Alias>,
    #[serde(default)]
    pub fuel_or_material: Option<String>,
    #[serde(default)]
    pub configuration: Option<String>,
    #[serde(default)]
    pub gravity: Option<Measured>,
    #[serde(default)]
    pub conditions: BTreeMap<String, Measured>,
    #[serde(default)]
    pub identity_resolution: Resolution,
    #[serde(default)]
    pub identity_evidence: Vec<Derivation>,
    #[serde(default)]
    pub quality_flags: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlStage {
    pub t_start_s: f64,
    #[serde(default)]
    pub t_end_s: Option<f64>,
    pub control: String,
    #[serde(default)]
    pub value: Option<Measured>,
    #[serde(default)]
    pub operator_action: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlHistory {
    pub run_id: String,
    pub time_origin: String,
    #[serde(default)]
    pub stages: Vec<ControlStage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Investigation {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub families: Vec<Family>,
    #[serde(default)]
    pub source_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    pub id: String,
    pub investigation_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
}

/// Physical session (test day / calibration group). Splits are grouped at or above this level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub id: String,
    pub investigation_id: String,
    #[serde(default)]
    pub campaign_id: Option<String>,
    #[serde(default)]
    pub test_day: Option<String>,
    #[serde(default)]
    pub gmt_start: Option<String>,
    #[serde(default)]
    pub evidence: Vec<SourceLocator>,
    #[serde(default)]
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Specimen {
    pub id: String,
    pub investigation_id: String,
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub geometry: BTreeMap<String, Measured>,
    /// Specimens burned more than once share a group.
    #[serde(default)]
    pub reuse_group: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationKind {
    PixelScale,
    Homography,
    TimeOffset,
    LensDistortion,
    Instrument,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Calibration {
    pub id: String,
    pub kind: CalibrationKind,
    /// Run ids or session ids.
    pub applies_to: Vec<String>,
    pub parameters: JsonMap,
    #[serde(default)]
    pub uncertainty: BTreeMap<String, f64>,
    #[serde(default)]
    pub locator: Option<SourceLocator>,
    pub derivation: Derivation,
    #[serde(default)]
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Exclusion {
    pub run_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cohort {
    /// Content hash of (family, criteria, run_ids, source_manifest_hash).
    pub id: String,
    pub family: Family,
    pub version: String,
    pub criteria: BTreeMap<String, String>,
    pub run_ids: Vec<String>,
    #[serde(default)]
    pub excluded: Vec<Exclusion>,
    pub source_manifest_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub id: String,
    pub run_id: String,
    pub quantity: Quantity,
    #[serde(default)]
    pub channel: Option<String>,
    pub time_s: f64,
    pub available_at_s: f64,
    #[serde(deserialize_with = "nullable")]
    #[schemars(required)]
    pub raw_value: Option<RawValue>,
    #[serde(deserialize_with = "nullable")]
    #[schemars(required)]
    pub raw_unit: Option<String>,
    #[serde(deserialize_with = "nullable")]
    #[schemars(required)]
    pub value_si: Option<f64>,
    pub unit_si: String,
    #[serde(default)]
    pub uncertainty_si: Option<f64>,
    #[serde(default)]
    pub calibration_id: Option<String>,
    #[serde(default)]
    pub quality_flags: Vec<String>,
    pub evidence_class: EvidenceClass,
    pub locator: SourceLocator,
    pub derivation: Derivation,
}

impl Observation {
    /// Python `build_prefix` admission rule: taken *and* available at the cutoff.
    pub fn admitted_at(&self, cutoff_s: f64) -> bool {
        self.available_at_s <= cutoff_s && self.time_s <= cutoff_s
    }
}

fn ignis_derived() -> EvidenceClass {
    EvidenceClass::IgnisDerived
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContourObservation {
    pub id: String,
    pub run_id: String,
    pub time_s: f64,
    pub available_at_s: f64,
    pub s_m: Vec<f64>,
    pub f_m: Vec<Option<f64>>,
    pub status: ContourStatus,
    pub calibration_id: String,
    #[serde(default)]
    pub scale_uncertainty_rel: Option<f64>,
    pub mask_dependency_frames: Vec<i64>,
    pub frame: i64,
    #[serde(default = "ignis_derived")]
    pub evidence_class: EvidenceClass,
    pub locator: SourceLocator,
    pub derivation: Derivation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndpointRecord {
    pub run_id: String,
    #[serde(deserialize_with = "nullable")]
    #[schemars(required)]
    pub original_label: Option<String>,
    #[serde(default)]
    pub original_comment: Option<String>,
    #[serde(default)]
    pub allowed_causes: Vec<String>,
    pub censoring: Censoring,
    #[serde(default)]
    pub last_confirmed_active_s: Option<f64>,
    #[serde(default)]
    pub first_terminal_s: Option<f64>,
    #[serde(default)]
    pub intervention_s: Option<f64>,
    #[serde(default)]
    pub intervention_kind: Option<String>,
    #[serde(default)]
    pub observation_state: Option<String>,
    pub resolution: Resolution,
    #[serde(default)]
    pub evidence: Vec<SourceLocator>,
    pub derivation: Derivation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Split {
    pub id: String,
    pub cohort_id: String,
    pub group_key: String,
    pub rule: String,
    pub seed: i64,
    pub run_to_group: BTreeMap<String, String>,
    pub train: Vec<String>,
    pub validation: Vec<String>,
    pub test: Vec<String>,
    pub frozen_at: String,
}

/// Partition of a frozen split. `Test` is the lockbox: only an evaluation scope may read it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Partition {
    Train,
    Validation,
    Test,
}

impl Split {
    /// Mirror of Python `Split.partition_of`.
    pub fn partition_of(&self, run_id: &str) -> Option<Partition> {
        let g = self.run_to_group.get(run_id)?;
        if self.train.contains(g) {
            Some(Partition::Train)
        } else if self.validation.contains(g) {
            Some(Partition::Validation)
        } else if self.test.contains(g) {
            Some(Partition::Test)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------- prefix

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrefixBundle {
    pub family: Family,
    pub run_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    pub cutoff_s: f64,
    pub time_origin: String,
    pub known_conditions: BTreeMap<String, Measured>,
    #[serde(default)]
    pub known_controls: Vec<ControlStage>,
    pub observations: Vec<Observation>,
    #[serde(default)]
    pub contours: Vec<ContourObservation>,
    #[serde(default)]
    pub phase_established_at_s: Option<f64>,
    #[serde(default)]
    pub support_evidence: Vec<String>,
    #[serde(default)]
    pub transform_versions: BTreeMap<String, String>,
    pub input_hash: String,
}

// ---------------------------------------------------------------- artifacts

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TensorEntry {
    pub name: String,
    pub shape: Vec<i64>,
    pub dtype: String,
    pub sha256: String,
    #[serde(default = "yes")]
    pub trainable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelRelease {
    pub id: String,
    pub family: Family,
    pub model_name: String,
    pub is_baseline: bool,
    pub architecture: JsonMap,
    pub config: JsonMap,
    pub feature_order: Vec<String>,
    pub units: BTreeMap<String, String>,
    pub normalization: BTreeMap<String, BTreeMap<String, f64>>,
    #[serde(default)]
    pub priors: JsonMap,
    #[serde(default)]
    pub observation_model: JsonMap,
    #[serde(default)]
    pub numerics: JsonMap,
    #[serde(default)]
    pub tensors: Vec<TensorEntry>,
    #[serde(default)]
    pub posterior: Vec<TensorEntry>,
    #[serde(default)]
    pub posterior_parameters: Vec<String>,
    pub cohort_id: String,
    pub split_id: String,
    pub source_manifest_hash: String,
    pub code_version: String,
    #[serde(default)]
    pub applicability: JsonMap,
    #[serde(default)]
    pub evaluation: JsonMap,
    #[serde(default)]
    pub state: QualificationState,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub lineage: Lineage,
    #[serde(default)]
    pub lineage_note: Option<String>,
    #[serde(default = "yes")]
    pub posterior_valid_for_parameters: bool,
    /// Required since 2.1.0; see [`TrainedOn`].
    pub trained_on: TrainedOn,
    #[serde(default)]
    pub resource_record: Option<String>,
    pub created_at: String,
    /// How the parameters were constructed. Required when `trained_on = not_trained`; omitted for
    /// legacy fitted releases so their content ids are unchanged. Checked by
    /// [`ModelRelease::check_construction`] on every registry save and load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction: Option<ModelBuildManifest>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Support {
    pub admitted: bool,
    pub gate: SupportGate,
    #[serde(default)]
    pub reasons: Vec<String>,
}

fn model_output() -> EvidenceClass {
    EvidenceClass::ModelOutput
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisArtifact {
    pub id: String,
    pub kind: ArtifactKind,
    pub model_release_id: String,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub time_origin: Option<String>,
    pub prefix_hash: String,
    pub settings_hash: String,
    pub status: QualificationState,
    #[serde(default)]
    pub quantity: Option<Quantity>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub times_s: Vec<f64>,
    #[serde(default)]
    pub quantile_levels: Vec<f64>,
    #[serde(default)]
    pub quantiles: Vec<Vec<f64>>,
    #[serde(default)]
    pub mean: Vec<f64>,
    #[serde(default)]
    pub space_s_m: Vec<f64>,
    #[serde(default)]
    pub field_quantiles: Vec<Vec<Vec<f64>>>,
    #[serde(default)]
    pub field_mean: Vec<Vec<f64>>,
    #[serde(default)]
    pub event_categories: Vec<String>,
    #[serde(default)]
    pub event_cdf: Vec<Vec<f64>>,
    pub support: Support,
    #[serde(default)]
    pub diagnostics: JsonMap,
    #[serde(default = "model_output")]
    pub evidence_class: EvidenceClass,
    pub created_at: String,
}

impl AnalysisArtifact {
    /// Structural validation the runtime applies to every artifact before serving it (§20.3):
    /// shapes, finiteness, probability range, unit/quantity agreement.
    pub fn validate(&self) -> Result<(), String> {
        let n = self.times_s.len();
        if self.times_s.iter().any(|t| !t.is_finite()) {
            return Err("non-finite time".into());
        }
        if self.times_s.windows(2).any(|w| w[1] < w[0]) {
            return Err("times_s not sorted".into());
        }
        if self.quantiles.len() != self.quantile_levels.len() {
            return Err("quantiles rows != quantile_levels".into());
        }
        if self.quantile_levels.iter().any(|q| !(*q > 0.0 && *q < 1.0)) {
            return Err("quantile level outside (0,1)".into());
        }
        if self.quantile_levels.windows(2).any(|w| w[1] <= w[0]) {
            return Err("quantile levels not strictly increasing".into());
        }
        for row in &self.quantiles {
            if row.len() != n || row.iter().any(|v| !v.is_finite()) {
                return Err("quantile row shape/finiteness".into());
            }
        }
        if !self.mean.is_empty()
            && (self.mean.len() != n || self.mean.iter().any(|v| !v.is_finite()))
        {
            return Err("mean shape/finiteness".into());
        }
        if self.event_cdf.len() != self.event_categories.len() {
            return Err("event_cdf rows != event_categories".into());
        }
        for row in &self.event_cdf {
            if row.len() != n
                || row
                    .iter()
                    .any(|p| !(p.is_finite() && (-1e-9..=1.0 + 1e-9).contains(p)))
            {
                return Err("event_cdf shape/range".into());
            }
        }
        if !self.event_cdf.is_empty() {
            for j in 0..n {
                let s: f64 = self.event_cdf.iter().map(|r| r[j]).sum();
                if s > 1.0 + 1e-6 {
                    return Err(format!("event cdf mass {s} > 1 at index {j}"));
                }
            }
        }
        if let (Some(q), Some(u)) = (self.quantity, &self.unit) {
            if q.unit_si() != u {
                return Err(format!("unit {u} does not match {q:?} ({})", q.unit_si()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AcquisitionAction {
    pub id: String,
    pub run_id: String,
    pub observation_id: String,
    pub time_s: f64,
    pub cost: f64,
    #[serde(default)]
    pub quality: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CandidateScore {
    pub action_id: String,
    pub expected_risk: f64,
    pub utility: f64,
    #[serde(default)]
    pub mc_standard_error: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecisionTrace {
    pub id: String,
    pub prefix_hash: String,
    pub model_release_id: String,
    pub target: String,
    pub loss: String,
    pub cost_weight: f64,
    pub mc_settings: JsonMap,
    pub candidates: Vec<CandidateScore>,
    #[serde(deserialize_with = "nullable")]
    #[schemars(required)]
    pub chosen_action_id: Option<String>,
    pub policy: DecisionPolicy,
    #[serde(default)]
    pub updated_artifact_id: Option<String>,
    #[serde(default)]
    pub realized_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SealedPrediction {
    pub artifact_id: String,
    pub artifact_hash: String,
    pub sealed_at: String,
    pub split_partition: SplitPartition,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub split_id: Option<String>,
    #[serde(default)]
    pub prefix_hash: Option<String>,
}

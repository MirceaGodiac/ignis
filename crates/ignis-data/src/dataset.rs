//! Dataset bundles (workboard R2): the curated numeric material handed to models and evaluation as
//! [`DatasetBundle`]s, and [`DatasetStore`], the one [`DatasetService`] implementation.
//!
//! A bundle is built from the ingested records of one family, its frozen cohort and split, for an
//! explicit set of partitions. Views are flat Parquet tables (one row per run, per condition value,
//! per endpoint, per observation); every column has a role and, for numbers, its SI unit; time
//! columns are seconds since the bundle's `time_origin` (the same origin the model views use, stated
//! with its adjudication status). Original values stay linked to their SI value: each observation
//! and condition row keeps `raw_value`, `raw_unit`, the SI value, the locator of the original cell
//! and the derivation method. Missing values are nulls with a flag, never zeros.
//!
//! | view | rows | content |
//! |---|---|---|
//! | `runs` | run | partition, group, configuration, material, numeric conditions (SI, one column per key) |
//! | `conditions` | run × condition | raw value/unit, SI value/unit, resolution, evidence class, locator, method |
//! | `endpoints` | run | censoring, last active / first terminal / intervention times, causes, evidence |
//! | `obs_<quantity>` (FLEX) | observation | time, available-at, SI value, uncertainty, raw value/unit, validity, flags, locator |
//!
//! **Audiences.** A `builder` bundle holds only the partitions it names and never `test`; dataset ids
//! say which (`flex-<split16>-train`, `flex-<split16>-train-validation`). The `evaluator` bundle with
//! test outcomes is written under the lockbox root (`IGNIS_HOLDOUT`, default `holdout/datasets/`),
//! separately from builder bundles, and [`DatasetStore`] opens it only for an evaluation scope.
//!
//! **Identity.** Parquet bytes are content-addressed (`art-<sha32>`); an unchanged artifact keeps its
//! first `created_at`, so [`DatasetBundle::content_digest`] is stable across rebuilds and changes when
//! (and only when) a value, a column, a run or a criterion changes. The committed copy under
//! `data/datasets/` is rewritten only when that digest changes.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context};
use arrow_array::builder::{BooleanBuilder, Float64Builder, StringBuilder};
use arrow_array::{ArrayRef, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use ignis_core::contracts::{
    ArtifactId, ArtifactRef, BundleAudience, Capability, Cohort, ColumnRole, ColumnSpec,
    DatasetBundle, DatasetId, DatasetSchema, DatasetView, Digest, ErrorCode, ErrorEnvelope,
    ExecutionScope, Family, Origin, Partition, Producer, Quantity, RawValue, RetryPolicy,
    ScopeRole, Split, Timestamp,
};
use ignis_core::contracts::{ConditionsQuery, ConditionsTable, RunId};
use ignis_core::services::{DatasetService, ServiceResult};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;
use serde_json::{json, Value};

use crate::common::{enum_str, Bundle};
use crate::paths;

/// Layout version of the bundle views (part of every bundle's inclusion criteria).
pub const BUNDLE_VERSION: &str = "1";
pub const PARQUET: &str = "application/vnd.apache.parquet";

// ------------------------------------------------------------------ tables

/// One column's values.
#[derive(Debug, Clone)]
pub enum Col {
    Str(Vec<Option<String>>),
    F64(Vec<Option<f64>>),
    Bool(Vec<Option<bool>>),
}

impl Col {
    fn len(&self) -> usize {
        match self {
            Col::Str(v) => v.len(),
            Col::F64(v) => v.len(),
            Col::Bool(v) => v.len(),
        }
    }
    fn dtype(&self) -> DataType {
        match self {
            Col::Str(_) => DataType::Utf8,
            Col::F64(_) => DataType::Float64,
            Col::Bool(_) => DataType::Boolean,
        }
    }
    fn has_null(&self) -> bool {
        match self {
            Col::Str(v) => v.iter().any(Option::is_none),
            Col::F64(v) => v.iter().any(Option::is_none),
            Col::Bool(v) => v.iter().any(Option::is_none),
        }
    }
    fn array(&self) -> ArrayRef {
        match self {
            Col::Str(v) => {
                let mut b = StringBuilder::new();
                v.iter().for_each(|x| b.append_option(x.as_deref()));
                Arc::new(b.finish())
            }
            Col::F64(v) => {
                let mut b = Float64Builder::new();
                v.iter().for_each(|x| b.append_option(*x));
                Arc::new(b.finish())
            }
            Col::Bool(v) => {
                let mut b = BooleanBuilder::new();
                v.iter().for_each(|x| b.append_option(*x));
                Arc::new(b.finish())
            }
        }
    }
}

/// A named table whose columns carry their contract spec.
#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub cols: Vec<(ColumnSpec, Col)>,
}

fn spec(
    name: &str,
    role: ColumnRole,
    quantity: Option<Quantity>,
    unit: Option<&str>,
) -> ColumnSpec {
    ColumnSpec {
        name: name.into(),
        role,
        quantity,
        unit: unit.map(String::from),
        nullable: false, // set from the data in `Table::push`
    }
}

impl Table {
    fn new(name: &str) -> Table {
        Table {
            name: name.into(),
            cols: Vec::new(),
        }
    }

    fn push(&mut self, mut s: ColumnSpec, c: Col) {
        s.nullable = c.has_null();
        self.cols.push((s, c));
    }

    pub fn rows(&self) -> usize {
        self.cols.first().map_or(0, |(_, c)| c.len())
    }

    /// Deterministic Parquet bytes (Snappy, schema + bundle metadata in `ignis` key-value).
    pub fn to_parquet(&self, meta: &Value) -> anyhow::Result<Vec<u8>> {
        let n = self.rows();
        for (s, c) in &self.cols {
            anyhow::ensure!(
                c.len() == n,
                "{}.{}: {} rows, table has {n}",
                self.name,
                s.name,
                c.len()
            );
        }
        let fields: Vec<Field> = self
            .cols
            .iter()
            .map(|(s, c)| Field::new(&s.name, c.dtype(), s.nullable))
            .collect();
        let schema = Arc::new(Schema::new(fields));
        let batch = RecordBatch::try_new(
            schema.clone(),
            self.cols.iter().map(|(_, c)| c.array()).collect(),
        )?;
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .set_key_value_metadata(Some(vec![KeyValue::new(
                "ignis".to_string(),
                meta.to_string(),
            )]))
            .build();
        let mut out = Vec::new();
        let mut w = ArrowWriter::try_new(&mut out, schema, Some(props))?;
        w.write(&batch)?;
        w.close()?;
        Ok(out)
    }
}

// ------------------------------------------------------------------ building

/// What a bundle contains: an audience and the partitions whose runs (with outcomes) it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleSpec {
    pub audience: BundleAudience,
    pub partitions: BTreeSet<Partition>,
}

impl BundleSpec {
    pub fn builder(partitions: &[Partition]) -> BundleSpec {
        BundleSpec {
            audience: BundleAudience::Builder,
            partitions: partitions.iter().copied().collect(),
        }
    }

    pub fn evaluator() -> BundleSpec {
        BundleSpec {
            audience: BundleAudience::Evaluator,
            partitions: [Partition::Train, Partition::Validation, Partition::Test].into(),
        }
    }

    fn suffix(&self) -> String {
        match self.audience {
            BundleAudience::Evaluator => "evaluator".into(),
            BundleAudience::Builder => self
                .partitions
                .iter()
                .map(|p| enum_str(*p))
                .collect::<Vec<_>>()
                .join("-"),
        }
    }
}

/// `<family>-<split hex16>-<partitions|evaluator>`: stable across rebuilds, never a path.
pub fn dataset_id(family: Family, split: &Split, s: &BundleSpec) -> String {
    let h = ignis_core::hashing::hex_part(&split.id);
    format!(
        "{}-{}-{}",
        family.as_str(),
        &h[..h.len().min(16)],
        s.suffix()
    )
}

/// A built bundle: the contract document and the bytes of every artifact it references.
#[derive(Debug, Clone)]
pub struct BuiltBundle {
    pub bundle: DatasetBundle,
    pub files: Vec<(ArtifactRef, Vec<u8>)>,
    pub quality: Value,
}

fn raw_str(r: &Option<RawValue>) -> Option<String> {
    r.as_ref().map(|v| match v {
        RawValue::Number(x) => x.to_string(),
        RawValue::Text(s) => s.clone(),
    })
}

fn js<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).expect("json")
}

/// Wraps bytes as an artifact; `existing` keeps the first creation time of identical bytes.
fn artifact(
    bytes: &[u8],
    media_type: &str,
    contract: Option<&str>,
    existing: &dyn Fn(&Digest) -> Option<ArtifactRef>,
) -> anyhow::Result<ArtifactRef> {
    let sha = Digest::of_bytes(bytes);
    if let Some(a) = existing(&sha) {
        return Ok(a);
    }
    let hex = ignis_core::hashing::hex_part(sha.as_str()).to_string();
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(format!("art-{}", &hex[..32]))?,
        sha256: sha,
        media_type: media_type.into(),
        byte_length: bytes.len() as u64,
        contract: contract.map(String::from),
        produced_by: Producer::default(),
        inputs: Vec::new(),
        origin: Origin::System,
        created_at: Timestamp::now(),
    })
}

/// Build a bundle of `family` from its records, frozen cohort and split. Pure: nothing is written.
/// `existing` returns a stored artifact with the same digest (to keep its identity), if any.
pub fn build(
    records: &Bundle,
    cohort: &Cohort,
    split: &Split,
    s: &BundleSpec,
    existing: &dyn Fn(&Digest) -> Option<ArtifactRef>,
) -> anyhow::Result<BuiltBundle> {
    let family = cohort.family;
    if s.partitions.is_empty() {
        bail!("a bundle needs at least one partition");
    }
    if s.audience == BundleAudience::Builder && s.partitions.contains(&Partition::Test) {
        bail!(
            "a builder bundle cannot contain the test partition (only the evaluator bundle does)"
        );
    }
    if split.cohort_id != cohort.id {
        bail!(
            "split {} belongs to cohort {}, not {}",
            split.id,
            split.cohort_id,
            cohort.id
        );
    }
    // the cohort's runs in the requested partitions, in a stable order
    let mut runs: Vec<(String, Partition)> = cohort
        .run_ids
        .iter()
        .filter_map(|r| split.partition_of(r).map(|p| (r.clone(), p)))
        .filter(|(_, p)| s.partitions.contains(p))
        .collect();
    runs.sort();
    let in_bundle: BTreeMap<&str, Partition> = runs.iter().map(|(r, p)| (r.as_str(), *p)).collect();
    let recs: BTreeMap<&str, &ignis_core::contracts::ExperimentRecord> =
        records.records.iter().map(|r| (r.id.as_str(), r)).collect();
    let eps: BTreeMap<&str, &ignis_core::contracts::EndpointRecord> = records
        .endpoints
        .iter()
        .map(|e| (e.run_id.as_str(), e))
        .collect();
    for (r, _) in &runs {
        anyhow::ensure!(
            recs.contains_key(r.as_str()),
            "cohort run {r} has no ingested record"
        );
    }
    let mut quality = json!({"family": family.as_str(), "bundle_version": BUNDLE_VERSION,
        "runs": runs.len(), "runs_by_partition": {}, "issues": []});
    let mut issues: Vec<Value> = Vec::new();
    for p in &s.partitions {
        quality["runs_by_partition"][enum_str(*p)] =
            json!(runs.iter().filter(|(_, q)| q == p).count());
    }

    // ---- runs (wide numeric conditions, one SI unit per key)
    let mut keys: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut numeric: BTreeSet<String> = BTreeSet::new();
    for (r, _) in &runs {
        for (k, m) in &recs[r.as_str()].conditions {
            let e = keys.entry(k.clone()).or_default();
            if let (Some(_), Some(u)) = (m.value_si, &m.unit_si) {
                e.insert(u.clone());
                numeric.insert(k.clone());
            }
        }
    }
    for (k, units) in &keys {
        anyhow::ensure!(
            units.len() <= 1,
            "condition {k} has SI values in several units: {units:?}"
        );
    }
    let rr = |f: &dyn Fn(&ignis_core::contracts::ExperimentRecord) -> Option<String>| {
        Col::Str(runs.iter().map(|(r, _)| f(recs[r.as_str()])).collect())
    };
    let mut t_runs = Table::new("runs");
    t_runs.push(
        spec("run_id", ColumnRole::RunId, None, None),
        rr(&|x| Some(x.id.clone())),
    );
    t_runs.push(
        spec("partition", ColumnRole::Flag, None, None),
        Col::Str(runs.iter().map(|(_, p)| Some(enum_str(*p))).collect()),
    );
    t_runs.push(
        spec("group_id", ColumnRole::Condition, None, None),
        Col::Str(
            runs.iter()
                .map(|(r, _)| split.run_to_group.get(r).cloned())
                .collect(),
        ),
    );
    t_runs.push(
        spec("investigation_id", ColumnRole::Condition, None, None),
        rr(&|x| Some(x.investigation_id.clone())),
    );
    t_runs.push(
        spec("configuration", ColumnRole::Condition, None, None),
        rr(&|x| x.configuration.clone()),
    );
    t_runs.push(
        spec("fuel_or_material", ColumnRole::Condition, None, None),
        rr(&|x| x.fuel_or_material.clone()),
    );
    t_runs.push(
        spec("identity_resolution", ColumnRole::Flag, None, None),
        rr(&|x| Some(enum_str(x.identity_resolution))),
    );
    for k in &numeric {
        let unit = keys[k].iter().next().cloned();
        let vals: Vec<Option<f64>> = runs
            .iter()
            .map(|(r, _)| {
                recs[r.as_str()]
                    .conditions
                    .get(k)
                    .and_then(|m| m.value_si)
                    .filter(|v| v.is_finite())
            })
            .collect();
        let missing = vals.iter().filter(|v| v.is_none()).count();
        if missing > 0 {
            issues.push(
                json!({"view": "runs", "column": format!("{k}_si"), "missing_runs": missing}),
            );
        }
        t_runs.push(
            spec(
                &format!("{k}_si"),
                ColumnRole::Condition,
                None,
                unit.as_deref(),
            ),
            Col::F64(vals),
        );
    }
    for k in keys.keys().filter(|k| !numeric.contains(*k)) {
        // textual conditions (fuel, fiber, ...) as written in the source
        let vals: Vec<Option<String>> = runs
            .iter()
            .map(|(r, _)| {
                recs[r.as_str()]
                    .conditions
                    .get(k)
                    .and_then(|m| raw_str(&m.raw_value))
            })
            .collect();
        t_runs.push(spec(k, ColumnRole::Condition, None, None), Col::Str(vals));
    }

    // ---- conditions (long provenance: raw + SI + locator per value)
    let mut c_rows: Vec<(&str, &str, &ignis_core::contracts::Measured)> = Vec::new();
    for (r, _) in &runs {
        for (k, m) in &recs[r.as_str()].conditions {
            c_rows.push((r.as_str(), k.as_str(), m));
        }
    }
    let mut t_cond = Table::new("conditions");
    t_cond.push(
        spec("run_id", ColumnRole::RunId, None, None),
        Col::Str(c_rows.iter().map(|x| Some(x.0.to_string())).collect()),
    );
    t_cond.push(
        spec("key", ColumnRole::Condition, None, None),
        Col::Str(c_rows.iter().map(|x| Some(x.1.to_string())).collect()),
    );
    t_cond.push(
        spec("raw_value", ColumnRole::Value, None, None),
        Col::Str(c_rows.iter().map(|x| raw_str(&x.2.raw_value)).collect()),
    );
    t_cond.push(
        spec("raw_unit", ColumnRole::Flag, None, None),
        Col::Str(c_rows.iter().map(|x| x.2.raw_unit.clone()).collect()),
    );
    t_cond.push(
        spec("value_si", ColumnRole::Value, None, None),
        Col::F64(
            c_rows
                .iter()
                .map(|x| x.2.value_si.filter(|v| v.is_finite()))
                .collect(),
        ),
    );
    t_cond.push(
        spec("unit_si", ColumnRole::Flag, None, None),
        Col::Str(c_rows.iter().map(|x| x.2.unit_si.clone()).collect()),
    );
    t_cond.push(
        spec("uncertainty_si", ColumnRole::Uncertainty, None, None),
        Col::F64(c_rows.iter().map(|x| x.2.uncertainty_si).collect()),
    );
    t_cond.push(
        spec("resolution", ColumnRole::Flag, None, None),
        Col::Str(
            c_rows
                .iter()
                .map(|x| Some(enum_str(x.2.resolution)))
                .collect(),
        ),
    );
    t_cond.push(
        spec("evidence_class", ColumnRole::Flag, None, None),
        Col::Str(
            c_rows
                .iter()
                .map(|x| Some(enum_str(x.2.evidence_class)))
                .collect(),
        ),
    );
    t_cond.push(
        spec("flags", ColumnRole::Flag, None, None),
        Col::Str(c_rows.iter().map(|x| Some(js(&x.2.flags))).collect()),
    );
    t_cond.push(
        spec("locator", ColumnRole::Locator, None, None),
        Col::Str(
            c_rows
                .iter()
                .map(|x| x.2.locator.as_ref().map(js))
                .collect(),
        ),
    );
    t_cond.push(
        spec("method_id", ColumnRole::Locator, None, None),
        Col::Str(
            c_rows
                .iter()
                .map(|x| x.2.derivation.as_ref().map(|d| d.method_id.clone()))
                .collect(),
        ),
    );

    // ---- endpoints (outcomes of the bundle's runs only)
    let mut t_ep = Table::new("endpoints");
    let ep = |r: &str| eps.get(r).copied();
    let missing_ep = runs.iter().filter(|(r, _)| ep(r).is_none()).count();
    if missing_ep > 0 {
        issues.push(json!({"view": "endpoints", "missing_runs": missing_ep}));
    }
    let e_rows: Vec<&ignis_core::contracts::EndpointRecord> =
        runs.iter().filter_map(|(r, _)| ep(r)).collect();
    t_ep.push(
        spec("run_id", ColumnRole::RunId, None, None),
        Col::Str(e_rows.iter().map(|e| Some(e.run_id.clone())).collect()),
    );
    t_ep.push(
        spec("censoring", ColumnRole::Flag, None, None),
        Col::Str(e_rows.iter().map(|e| Some(enum_str(e.censoring))).collect()),
    );
    t_ep.push(
        spec("last_confirmed_active_s", ColumnRole::Time, None, Some("s")),
        Col::F64(e_rows.iter().map(|e| e.last_confirmed_active_s).collect()),
    );
    t_ep.push(
        spec("first_terminal_s", ColumnRole::Time, None, Some("s")),
        Col::F64(e_rows.iter().map(|e| e.first_terminal_s).collect()),
    );
    t_ep.push(
        spec("intervention_s", ColumnRole::Time, None, Some("s")),
        Col::F64(e_rows.iter().map(|e| e.intervention_s).collect()),
    );
    t_ep.push(
        spec("intervention_kind", ColumnRole::Flag, None, None),
        Col::Str(e_rows.iter().map(|e| e.intervention_kind.clone()).collect()),
    );
    t_ep.push(
        spec("allowed_causes", ColumnRole::Flag, None, None),
        Col::Str(e_rows.iter().map(|e| Some(js(&e.allowed_causes))).collect()),
    );
    t_ep.push(
        spec("observation_state", ColumnRole::Flag, None, None),
        Col::Str(e_rows.iter().map(|e| e.observation_state.clone()).collect()),
    );
    t_ep.push(
        spec("original_label", ColumnRole::Flag, None, None),
        Col::Str(e_rows.iter().map(|e| e.original_label.clone()).collect()),
    );
    t_ep.push(
        spec("resolution", ColumnRole::Flag, None, None),
        Col::Str(
            e_rows
                .iter()
                .map(|e| Some(enum_str(e.resolution)))
                .collect(),
        ),
    );
    t_ep.push(
        spec("evidence", ColumnRole::Locator, None, None),
        Col::Str(e_rows.iter().map(|e| Some(js(&e.evidence))).collect()),
    );
    t_ep.push(
        spec("method_id", ColumnRole::Locator, None, None),
        Col::Str(
            e_rows
                .iter()
                .map(|e| Some(e.derivation.method_id.clone()))
                .collect(),
        ),
    );

    let mut tables = vec![t_runs, t_cond, t_ep];

    // ---- observations, one view per quantity (FLEX)
    let mut by_q: BTreeMap<Quantity, Vec<&ignis_core::contracts::Observation>> = BTreeMap::new();
    for o in &records.observations {
        if in_bundle.contains_key(o.run_id.as_str()) {
            by_q.entry(o.quantity).or_default().push(o);
        }
    }
    let mut obs_quality = serde_json::Map::new();
    for (q, mut obs) in by_q {
        obs.sort_by(|a, b| {
            a.run_id
                .cmp(&b.run_id)
                .then(a.time_s.total_cmp(&b.time_s))
                .then_with(|| a.id.cmp(&b.id))
        });
        let unit = q.unit_si();
        for o in &obs {
            anyhow::ensure!(
                o.unit_si == unit,
                "observation {} of {q:?} is in {}, not {unit}",
                o.id,
                o.unit_si
            );
            anyhow::ensure!(
                o.time_s.is_finite() && o.available_at_s.is_finite(),
                "observation {}: non-finite time",
                o.id
            );
        }
        let valid: Vec<bool> = obs
            .iter()
            .map(|o| ignis_core::views::measurement_valid(o))
            .collect();
        let mut flags: BTreeMap<String, usize> = BTreeMap::new();
        for o in &obs {
            for f in &o.quality_flags {
                *flags.entry(f.clone()).or_default() += 1;
            }
        }
        let early = obs.iter().filter(|o| o.available_at_s < o.time_s).count();
        if early > 0 {
            issues.push(
                json!({"view": format!("obs_{}", enum_str(q)), "available_before_observed": early}),
            );
        }
        let qn = enum_str(q);
        obs_quality.insert(qn.clone(), json!({"rows": obs.len(), "invalid": valid.iter().filter(|v| !**v).count(),
            "flags": flags, "runs": obs.iter().map(|o| o.run_id.as_str()).collect::<BTreeSet<_>>().len()}));
        let mut t = Table::new(&format!("obs_{qn}"));
        t.push(
            spec("run_id", ColumnRole::RunId, None, None),
            Col::Str(obs.iter().map(|o| Some(o.run_id.clone())).collect()),
        );
        t.push(
            spec("observation_id", ColumnRole::Locator, None, None),
            Col::Str(obs.iter().map(|o| Some(o.id.clone())).collect()),
        );
        t.push(
            spec("channel", ColumnRole::Condition, None, None),
            Col::Str(obs.iter().map(|o| o.channel.clone()).collect()),
        );
        t.push(
            spec("time_s", ColumnRole::Time, None, Some("s")),
            Col::F64(obs.iter().map(|o| Some(o.time_s)).collect()),
        );
        t.push(
            spec("available_at_s", ColumnRole::AvailableAt, None, Some("s")),
            Col::F64(obs.iter().map(|o| Some(o.available_at_s)).collect()),
        );
        t.push(
            spec(
                &format!("{qn}_{unit}"),
                ColumnRole::Value,
                Some(q),
                Some(unit),
            ),
            Col::F64(
                obs.iter()
                    .map(|o| o.value_si.filter(|v| v.is_finite()))
                    .collect(),
            ),
        );
        t.push(
            spec(
                &format!("{qn}_unc_{unit}"),
                ColumnRole::Uncertainty,
                Some(q),
                Some(unit),
            ),
            Col::F64(
                obs.iter()
                    .map(|o| o.uncertainty_si.filter(|v| v.is_finite()))
                    .collect(),
            ),
        );
        t.push(
            spec("raw_value", ColumnRole::Value, None, None),
            Col::Str(obs.iter().map(|o| raw_str(&o.raw_value)).collect()),
        );
        t.push(
            spec("raw_unit", ColumnRole::Flag, None, None),
            Col::Str(obs.iter().map(|o| o.raw_unit.clone()).collect()),
        );
        t.push(
            spec("valid", ColumnRole::Flag, None, None),
            Col::Bool(valid.iter().map(|v| Some(*v)).collect()),
        );
        t.push(
            spec("quality_flags", ColumnRole::Flag, None, None),
            Col::Str(obs.iter().map(|o| Some(js(&o.quality_flags))).collect()),
        );
        t.push(
            spec("calibration_id", ColumnRole::Flag, None, None),
            Col::Str(obs.iter().map(|o| o.calibration_id.clone()).collect()),
        );
        t.push(
            spec("evidence_class", ColumnRole::Flag, None, None),
            Col::Str(
                obs.iter()
                    .map(|o| Some(enum_str(o.evidence_class)))
                    .collect(),
            ),
        );
        t.push(
            spec("locator", ColumnRole::Locator, None, None),
            Col::Str(obs.iter().map(|o| Some(js(&o.locator))).collect()),
        );
        t.push(
            spec("method_id", ColumnRole::Locator, None, None),
            Col::Str(
                obs.iter()
                    .map(|o| Some(o.derivation.method_id.clone()))
                    .collect(),
            ),
        );
        tables.push(t);
    }
    if !obs_quality.is_empty() {
        quality["observations"] = Value::Object(obs_quality);
    }
    quality["issues"] = json!(issues);
    quality["excluded_from_cohort"] = json!(cohort.excluded.len());

    let id = dataset_id(family, split, s);
    let meta = json!({"dataset_id": id, "family": family.as_str(), "audience": s.audience,
        "partitions": s.partitions, "time_origin": ignis_core::views::family_time_origin(family),
        "bundle_version": BUNDLE_VERSION, "units": "SI"});
    let mut files = Vec::new();
    let mut views = Vec::new();
    for t in &tables {
        let bytes = t.to_parquet(&meta)?;
        let a = artifact(&bytes, PARQUET, None, existing)?;
        views.push(DatasetView {
            name: t.name.clone(),
            artifact: a.clone(),
            columns: t.cols.iter().map(|(s, _)| s.clone()).collect(),
            row_count: t.rows() as u64,
        });
        files.push((a, bytes));
    }
    let qbytes = (serde_json::to_string_pretty(&quality)? + "\n").into_bytes();
    let qref = artifact(
        &qbytes,
        "application/json",
        Some("DatasetQualityReport"),
        existing,
    )?;
    files.push((qref.clone(), qbytes));
    let mut criteria = cohort.criteria.clone();
    criteria.insert("bundle_version".into(), BUNDLE_VERSION.into());
    criteria.insert("cohort_version".into(), cohort.version.clone());
    criteria.insert(
        "partitions".into(),
        s.partitions
            .iter()
            .map(|p| enum_str(*p))
            .collect::<Vec<_>>()
            .join(","),
    );
    let bundle = DatasetBundle {
        schema_version: DatasetSchema::V1,
        dataset_id: DatasetId::new(id)?,
        family,
        audience: s.audience,
        cohort_id: cohort.id.clone(),
        split_id: split.id.clone(),
        partitions: s.partitions.clone(),
        source_manifest_hash: cohort.source_manifest_hash.clone(),
        time_origin: ignis_core::views::family_time_origin(family).to_string(),
        views,
        quality_report: Some(qref),
        inclusion_criteria: criteria,
        origin: Origin::System,
        created_at: Timestamp::now(),
    };
    bundle.validate()?;
    Ok(BuiltBundle {
        bundle,
        files,
        quality,
    })
}

// ------------------------------------------------------------------ store and service

/// Where bundles live: builder bundles under `data/work/datasets/`, the evaluator bundle under the
/// lockbox root. Each root holds `<dataset_id>.json` and `artifacts/<sha256>.{parquet,json}` (+ the
/// `ArtifactRef` next to the bytes).
#[derive(Debug, Clone)]
pub struct DatasetStore {
    pub builder_root: PathBuf,
    pub evaluator_root: PathBuf,
}

fn err(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

fn ext(media_type: &str) -> &'static str {
    if media_type == PARQUET {
        "parquet"
    } else {
        "json"
    }
}

impl DatasetStore {
    /// `data/work/datasets/` and `$IGNIS_HOLDOUT/datasets/` (default `holdout/datasets/`).
    pub fn default_roots() -> DatasetStore {
        let holdout = std::env::var_os("IGNIS_HOLDOUT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("holdout"));
        DatasetStore {
            builder_root: paths::work(&["datasets"]),
            evaluator_root: holdout.join("datasets"),
        }
    }

    fn root(&self, a: BundleAudience) -> &Path {
        match a {
            BundleAudience::Builder => &self.builder_root,
            BundleAudience::Evaluator => &self.evaluator_root,
        }
    }

    fn artifact_paths(root: &Path, sha: &Digest, media_type: &str) -> (PathBuf, PathBuf) {
        let h = ignis_core::hashing::hex_part(sha.as_str());
        let d = root.join("artifacts");
        (
            d.join(format!("{h}.{}", ext(media_type))),
            d.join(format!("{h}.ref.json")),
        )
    }

    /// A stored artifact with this digest in the audience's root, if any.
    pub fn existing(&self, a: BundleAudience, sha: &Digest) -> Option<ArtifactRef> {
        let d = self.root(a).join("artifacts");
        let h = ignis_core::hashing::hex_part(sha.as_str());
        let p = d.join(format!("{h}.ref.json"));
        let r: ArtifactRef = serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()?;
        (r.sha256 == *sha).then_some(r)
    }

    /// Write a built bundle (artifacts first, then the bundle document; each write is atomic).
    pub fn save(&self, b: &BuiltBundle) -> anyhow::Result<PathBuf> {
        let root = self.root(b.bundle.audience);
        for (a, bytes) in &b.files {
            anyhow::ensure!(
                Digest::of_bytes(bytes) == a.sha256,
                "artifact {} bytes do not match",
                a.artifact_id
            );
            let (data, refp) = Self::artifact_paths(root, &a.sha256, &a.media_type);
            std::fs::create_dir_all(data.parent().unwrap())?;
            if !data.exists() {
                atomic_write(&data, bytes)?;
            }
            atomic_write(&refp, (serde_json::to_string_pretty(a)? + "\n").as_bytes())?;
        }
        let p = root.join(format!("{}.json", b.bundle.dataset_id));
        atomic_write(
            &p,
            (serde_json::to_string_pretty(&b.bundle)? + "\n").as_bytes(),
        )?;
        Ok(p)
    }

    /// Writes the `ArtifactRef` of builder bytes already placed under `artifacts/` (a view streamed to
    /// disk, e.g. the R2b sensor samples), after checking it names those bytes' file.
    pub fn write_ref(&self, a: &ArtifactRef) -> anyhow::Result<()> {
        let (data, refp) = Self::artifact_paths(&self.builder_root, &a.sha256, &a.media_type);
        anyhow::ensure!(
            std::fs::metadata(&data).is_ok_and(|m| m.len() == a.byte_length),
            "{} is not in place for artifact {}",
            data.display(),
            a.artifact_id
        );
        atomic_write(
            &refp,
            (serde_json::to_string_pretty(a)?
                + "
")
            .as_bytes(),
        )
    }

    fn load(&self, a: BundleAudience, id: &DatasetId) -> Option<anyhow::Result<DatasetBundle>> {
        let p = self.root(a).join(format!("{id}.json"));
        if !p.exists() {
            return None;
        }
        Some((|| {
            let b: DatasetBundle = serde_json::from_str(&std::fs::read_to_string(&p)?)
                .with_context(|| p.display().to_string())?;
            anyhow::ensure!(
                b.dataset_id == *id,
                "{} holds bundle {}",
                p.display(),
                b.dataset_id
            );
            anyhow::ensure!(
                b.audience == a,
                "{} is a {:?} bundle in the {a:?} root",
                p.display(),
                b.audience
            );
            b.validate()?;
            Ok(b)
        })())
    }

    /// Every builder bundle id in the store.
    pub fn list_builder(&self) -> anyhow::Result<Vec<String>> {
        let mut v = Vec::new();
        if self.builder_root.exists() {
            for e in std::fs::read_dir(&self.builder_root)? {
                let p = e?.path();
                if p.extension().is_some_and(|x| x == "json") {
                    v.push(p.file_stem().unwrap().to_string_lossy().to_string());
                }
            }
        }
        v.sort();
        Ok(v)
    }

    /// Local bytes of a view's artifact after the same scope checks as [`DatasetService::dataset`]
    /// (for the composition root, which streams them into the `ArtifactStore`).
    pub fn view_path(
        &self,
        scope: &ExecutionScope,
        id: &DatasetId,
        view: &str,
    ) -> ServiceResult<PathBuf> {
        let b = self.dataset(scope, id)?;
        let v = b
            .views
            .iter()
            .find(|v| v.name == view)
            .ok_or_else(|| err(ErrorCode::NotFound, format!("{id} has no view {view}")))?;
        let (p, _) = Self::artifact_paths(
            self.root(b.audience),
            &v.artifact.sha256,
            &v.artifact.media_type,
        );
        let bytes = std::fs::read(&p)
            .map_err(|e| err(ErrorCode::SourceMissing, format!("{}: {e}", p.display())))?;
        if Digest::of_bytes(&bytes) != v.artifact.sha256 {
            return Err(err(
                ErrorCode::Internal,
                format!("{}: bytes do not match {}", p.display(), v.artifact.sha256),
            ));
        }
        Ok(p)
    }
}

fn atomic_write(p: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    let tmp = p.with_extension("part");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, p)?;
    Ok(())
}

/// The parsed conditions table, kept while its file is unchanged (same length and modification time).
/// The table is a file built offline (`ignis data conditions`, about 5 MB); parsing it on every request
/// cost most of a request's time. A rebuilt file is read again on the next request.
///
/// The entry also holds the digest of the very bytes it parsed, and `conditions_snapshot` reports
/// that digest: the snapshot an answer names is always the data its cells came from, even when a
/// rewrite keeps the file's length and modification time (the cache would then serve the old
/// table, and the snapshot names the old table too).
fn cached_table(p: &Path) -> ServiceResult<(Arc<ConditionsTable>, Digest)> {
    type Key = (u64, Option<std::time::SystemTime>);
    type Entry = (PathBuf, Key, Arc<ConditionsTable>, Digest);
    static TABLE: std::sync::Mutex<Option<Entry>> = std::sync::Mutex::new(None);
    let meta = std::fs::metadata(p).map_err(|_| {
        err(
            ErrorCode::CapabilityUnavailable,
            format!("{} is not built; run `ignis data conditions`", p.display()),
        )
    })?;
    let key: Key = (meta.len(), meta.modified().ok());
    let mut slot = TABLE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((path, k, t, d)) = slot.as_ref() {
        if path == p && *k == key {
            return Ok((t.clone(), d.clone()));
        }
    }
    let bytes =
        std::fs::read(p).map_err(|e| err(ErrorCode::Internal, format!("{}: {e}", p.display())))?;
    let digest = Digest::of_bytes(&bytes);
    let table: Arc<ConditionsTable> = Arc::new(
        serde_json::from_slice(&bytes)
            .map_err(|e| err(ErrorCode::Internal, format!("{}: {e}", p.display())))?,
    );
    *slot = Some((p.to_path_buf(), key, table.clone(), digest.clone()));
    Ok((table, digest))
}

impl DatasetService for DatasetStore {
    fn dataset(&self, scope: &ExecutionScope, id: &DatasetId) -> ServiceResult<DatasetBundle> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        scope.require(Capability::ReadData)?;
        // the evaluator root is not even opened for another role
        let found = match self.load(BundleAudience::Builder, id) {
            Some(b) => Some(b),
            None if scope.role == ScopeRole::Evaluation => self.load(BundleAudience::Evaluator, id),
            None => None,
        };
        let b = found
            .ok_or_else(|| {
                err(
                    ErrorCode::NotFound,
                    format!("dataset {id} is not in this store"),
                )
            })?
            .map_err(|e| err(ErrorCode::Internal, format!("{e:#}")))?;
        if b.audience == BundleAudience::Evaluator && scope.role != ScopeRole::Evaluation {
            return Err(err(
                ErrorCode::ScopeDenied,
                "evaluator bundles are served to evaluation scopes only",
            ));
        }
        for p in &b.partitions {
            scope.require_partition(*p)?;
        }
        Ok(b)
    }

    /// Conditions of the flight investigations (inputs, never outcomes). A run in a frozen split
    /// partition the scope does not hold is withheld, like every other run-level answer.
    fn conditions(
        &self,
        scope: &ExecutionScope,
        q: &ConditionsQuery,
    ) -> ServiceResult<ConditionsTable> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        scope.require(Capability::ReadData)?;
        let p = crate::conditions::table_path();
        let (full, _) = cached_table(&p)?;
        let splits: Vec<Split> = crate::cohort::split_files()
            .iter()
            .map(|f| {
                crate::cohort::load_split_with_cohort(&f.display().to_string()).map(|(s, _)| s)
            })
            .collect::<anyhow::Result<_>>()
            .map_err(|e| err(ErrorCode::Internal, format!("{e:#}")))?;
        let visible = |id: &RunId| {
            splits.iter().all(|s| {
                s.partition_of(id.as_str())
                    .is_none_or(|part| scope.require_partition(part).is_ok())
            })
        };
        crate::conditions::query(&full, q, visible).map_err(|e| err(ErrorCode::InvalidRequest, e))
    }

    /// Digest over the conditions table the service answers from (the bytes its cached copy was
    /// parsed from) and every frozen split file (name and bytes): the inputs that decide both the
    /// cells and which runs a scope sees.
    /// The partition of `run` in every frozen split that assigns one (the same splits that decide
    /// which runs `conditions` withholds).
    fn run_partitions(
        &self,
        scope: &ExecutionScope,
        run: &RunId,
    ) -> ServiceResult<BTreeSet<Partition>> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        scope.require(Capability::ReadData)?;
        let mut out = BTreeSet::new();
        for f in crate::cohort::split_files() {
            let (s, _) = crate::cohort::load_split_with_cohort(&f.display().to_string())
                .map_err(|e| err(ErrorCode::Internal, format!("{e:#}")))?;
            out.extend(s.partition_of(run.as_str()));
        }
        Ok(out)
    }

    fn conditions_snapshot(&self, scope: &ExecutionScope) -> ServiceResult<Digest> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        scope.require(Capability::ReadData)?;
        let (_, table) = cached_table(&crate::conditions::table_path())?;
        let mut parts = vec![format!("conditions_table {table}")];
        for f in crate::cohort::split_files() {
            let bytes = std::fs::read(&f).map_err(|e| {
                err(
                    ErrorCode::CapabilityUnavailable,
                    format!("{}: {e}", f.display()),
                )
            })?;
            let name = f
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            parts.push(format!("split {name} {}", Digest::of_bytes(&bytes)));
        }
        Ok(Digest::of_bytes(parts.join("\n").as_bytes()))
    }

    /// Bundles the scope may read: a bundle refused for a partition is not listed; the evaluator
    /// root is listed for evaluation scopes only. A bundle that fails to load fails the listing.
    fn datasets(&self, scope: &ExecutionScope) -> ServiceResult<Vec<DatasetBundle>> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        scope.require(Capability::ReadData)?;
        let ids = |root: &Path| -> ServiceResult<Vec<String>> {
            let mut v = Vec::new();
            if root.exists() {
                let rd = std::fs::read_dir(root)
                    .map_err(|e| err(ErrorCode::Internal, format!("{}: {e}", root.display())))?;
                for e in rd {
                    let p = e
                        .map_err(|e| err(ErrorCode::Internal, e.to_string()))?
                        .path();
                    if p.is_file() && p.extension().is_some_and(|x| x == "json") {
                        v.push(p.file_stem().unwrap().to_string_lossy().to_string());
                    }
                }
            }
            Ok(v)
        };
        let mut all = ids(&self.builder_root)?;
        if scope.role == ScopeRole::Evaluation {
            all.extend(ids(&self.evaluator_root)?);
        }
        all.sort();
        all.dedup();
        let mut out = Vec::new();
        for id in all {
            let id = DatasetId::new(&id).map_err(|e| err(ErrorCode::Internal, e.to_string()))?;
            match self.dataset(scope, &id) {
                Ok(b) => out.push(b),
                Err(e) if e.code == ErrorCode::ScopeDenied => {}
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }

    /// The view's bytes, streamed from the content-addressed file (its length is checked against
    /// the `ArtifactRef`; the digest travels with the ref for the reader to verify).
    fn open_view(
        &self,
        scope: &ExecutionScope,
        id: &DatasetId,
        view: &str,
        offset: u64,
        len: Option<u64>,
    ) -> ServiceResult<(ArtifactRef, Box<dyn std::io::Read + Send>)> {
        use std::io::{Read, Seek, SeekFrom};
        let b = self.dataset(scope, id)?;
        let v = b.views.iter().find(|v| v.name == view).ok_or_else(|| {
            let names: Vec<&str> = b.views.iter().map(|v| v.name.as_str()).collect();
            err(
                ErrorCode::NotFound,
                format!(
                    "{id} has no view {view}; its views are {}",
                    names.join(", ")
                ),
            )
        })?;
        let (p, _) = Self::artifact_paths(
            self.root(b.audience),
            &v.artifact.sha256,
            &v.artifact.media_type,
        );
        let mut f = std::fs::File::open(&p)
            .map_err(|e| err(ErrorCode::SourceMissing, format!("{}: {e}", p.display())))?;
        let size = f
            .metadata()
            .map_err(|e| err(ErrorCode::Internal, format!("{}: {e}", p.display())))?
            .len();
        if size != v.artifact.byte_length {
            return Err(err(
                ErrorCode::Internal,
                format!(
                    "{}: {size} bytes, {} declares {}",
                    p.display(),
                    v.artifact.artifact_id,
                    v.artifact.byte_length
                ),
            ));
        }
        f.seek(SeekFrom::Start(offset.min(size)))
            .map_err(|e| err(ErrorCode::Internal, e.to_string()))?;
        let take = len.unwrap_or(u64::MAX);
        Ok((v.artifact.clone(), Box::new(f.take(take))))
    }
}

// ------------------------------------------------------------------ CLI helper

/// Build and save the bundle of `family` for `spec` from the frozen split under `data/splits/`.
pub fn build_and_save(
    family: Family,
    s: &BundleSpec,
    store: &DatasetStore,
) -> anyhow::Result<(BuiltBundle, PathBuf)> {
    let split_file = crate::cohort::split_files()
        .into_iter()
        .find(|f| {
            f.file_name().is_some_and(|n| {
                n.to_string_lossy()
                    .starts_with(&format!("{}-", family.as_str()))
            })
        })
        .with_context(|| {
            format!(
                "no frozen split for {} under {}",
                family.as_str(),
                paths::splits().display()
            )
        })?;
    let (split, cohort) = crate::cohort::load_split_with_cohort(&split_file.display().to_string())?;
    let records = crate::store::load(family.as_str())?;
    let built = build(&records, &cohort, &split, s, &|d| {
        store.existing(s.audience, d)
    })?;
    let p = store.save(&built)?;
    Ok((built, p))
}

/// Committed copy of a builder bundle (`data/datasets/<id>.json`), rewritten only when its content
/// digest changes, so the repository diff shows real changes only. Evaluator bundles are never copied.
pub fn publish_builder_copy(b: &DatasetBundle) -> anyhow::Result<(PathBuf, bool)> {
    anyhow::ensure!(
        b.audience == BundleAudience::Builder,
        "only builder bundles are committed"
    );
    let p = paths::root()
        .join("datasets")
        .join(format!("{}.json", b.dataset_id));
    if let Ok(s) = std::fs::read_to_string(&p) {
        if let Ok(old) = serde_json::from_str::<DatasetBundle>(&s) {
            if old.content_digest() == b.content_digest() {
                return Ok((p, false));
            }
        }
    }
    atomic_write(&p, (serde_json::to_string_pretty(b)? + "\n").as_bytes())?;
    Ok((p, true))
}

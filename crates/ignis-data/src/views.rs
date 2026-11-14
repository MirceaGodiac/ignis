//! Array views ([`ignis_core::views`]) as Parquet under `data/work/views/<cohort-hex>/`.
//!
//! Physical SI scaling only. No statistical normalization (models fit that on train only). Experts
//! read them through [`DataStore`] (`impl ViewSource`). One file per family view, one row per run:
//!
//! * `flex_trajectory.parquet` — `t_s`, `available_at_s`, `d_m` (NaN where invalid), `valid` lists.
//! * `acme_event_rows.parquet` — role-aware features (JSON object), interval bounds, causes.
//! * `bass_sequence.parquet` — metadata + (empty until the vision lane attaches them) front arrays.
//!
//! Conditions/features/geometry maps are stored as JSON text columns (small, sorted keys).

use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use arrow_array::builder::{BooleanBuilder, Float64Builder, ListBuilder, StringBuilder};
use arrow_array::cast::AsArray;
use arrow_array::types::Float64Type;
use arrow_array::{Array, ArrayRef, BooleanArray, Float64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use ignis_core::contracts::{Cohort, ExperimentRecord, Family, Quantity, Split};
use ignis_core::records::Registry;
use ignis_core::views::{AcmeEventRow, BassSequence, FlexTrajectory, RunArchive, View, ViewSource};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;
use serde_json::json;

use crate::cohort::{groups, load_cohort, load_split};
use crate::common::raw_str;
use crate::families::get;
use crate::{paths, store};

pub const VIEW_VERSION: &str = "2";
use ignis_core::views::INVALID_FLAGS;
pub use ignis_core::views::{ACME_TIME_ORIGIN, BASS_TIME_ORIGIN, FLEX_TIME_ORIGIN};

pub fn view_dir(cohort_id: &str) -> PathBuf {
    paths::work(&["views", ignis_core::hashing::hex_part(cohort_id)])
}

fn file_for(fam: Family) -> &'static str {
    match fam {
        Family::Flex => "flex_trajectory.parquet",
        Family::AcmeNormal | Family::AcmeInverse => "acme_event_rows.parquet",
        _ => "bass_sequence.parquet",
    }
}

fn numeric_conditions(
    rec: &ExperimentRecord,
    allowed: &std::collections::BTreeSet<String>,
) -> BTreeMap<String, f64> {
    rec.conditions
        .iter()
        .filter(|(k, _)| allowed.contains(*k))
        .filter_map(|(k, m)| m.value_si.filter(|v| v.is_finite()).map(|v| (k.clone(), v)))
        .collect()
}

fn flag_value(rec: &ExperimentRecord, prefix: &str) -> Option<bool> {
    rec.quality_flags
        .iter()
        .rev()
        .find_map(|f| f.strip_prefix(prefix))
        .and_then(|v| match v {
            "yes" => Some(true),
            "no" => Some(false),
            _ => None,
        })
}

/// Build the typed views of a cohort from the ingested records (no files written).
pub fn build_views(cohort_id: &str) -> anyhow::Result<(Cohort, Vec<View>, serde_json::Value)> {
    let coh = load_cohort(cohort_id)?;
    let spec = get(coh.family)?;
    let b = store::load(coh.family.as_str())?;
    let r2g = groups(&b, &coh.run_ids, spec.group_keys);
    let recs: BTreeMap<&str, &ExperimentRecord> =
        b.records.iter().map(|r| (r.id.as_str(), r)).collect();
    let eps: BTreeMap<&str, &ignis_core::contracts::EndpointRecord> =
        b.endpoints.iter().map(|e| (e.run_id.as_str(), e)).collect();
    let allowed = &spec.policy.allowed_conditions;
    let mut meta = json!({"cohort_id": coh.id, "family": coh.family.as_str(), "view_version": VIEW_VERSION,
                          "policy_allowed": allowed, "units": "SI"});
    let mut views = vec![];
    match coh.family {
        Family::Flex => {
            let mut by_run: BTreeMap<&str, Vec<&ignis_core::contracts::Observation>> =
                BTreeMap::new();
            for o in &b.observations {
                if o.quantity == Quantity::LiquidDiameter && r2g.contains_key(&o.run_id) {
                    by_run.entry(o.run_id.as_str()).or_default().push(o);
                }
            }
            for rid in &coh.run_ids {
                let mut obs = by_run.remove(rid.as_str()).unwrap_or_default();
                obs.sort_by(|a, b| a.time_s.total_cmp(&b.time_s).then_with(|| a.id.cmp(&b.id)));
                let valid: Vec<bool> = obs
                    .iter()
                    .map(|o| {
                        o.value_si.is_some()
                            && !o
                                .quality_flags
                                .iter()
                                .any(|f| INVALID_FLAGS.contains(&f.as_str()))
                    })
                    .collect();
                let e = eps[rid.as_str()];
                views.push(View::Flex(FlexTrajectory {
                    run_id: rid.clone(),
                    group_id: r2g[rid].clone(),
                    t_s: obs.iter().map(|o| o.time_s).collect(),
                    available_at_s: obs.iter().map(|o| o.available_at_s).collect(),
                    d_m: obs
                        .iter()
                        .zip(&valid)
                        .map(|(o, v)| if *v { o.value_si.unwrap() } else { f64::NAN })
                        .collect(),
                    valid,
                    conditions: numeric_conditions(recs[rid.as_str()], allowed),
                    censoring: crate::common::enum_str(e.censoring),
                    t_last_active_s: e.last_confirmed_active_s,
                    t_first_terminal_s: e.first_terminal_s,
                    allowed_causes: e.allowed_causes.clone(),
                    event_before_support_exit: flag_value(
                        recs[rid.as_str()],
                        "event_before_support_exit:",
                    ),
                    time_origin: FLEX_TIME_ORIGIN.into(),
                    evidence_class: "source_derived".into(),
                }));
            }
            meta["time_origin"] = json!(FLEX_TIME_ORIGIN);
        }
        Family::AcmeNormal | Family::AcmeInverse => {
            let species: std::collections::BTreeSet<String> = coh
                .run_ids
                .iter()
                .map(|r| raw_str(&recs[r.as_str()].conditions["fuel_species"]))
                .collect();
            for rid in &coh.run_ids {
                let (r, e) = (recs[rid.as_str()], eps[rid.as_str()]);
                let mut feats = BTreeMap::new();
                for k in [
                    "pressure",
                    "injected_mass_flow",
                    "x_o2_role",
                    "x_fuel_role",
                    "tad",
                    "zst",
                ] {
                    if let Some(v) = r.conditions[k].value_si {
                        feats.insert(k.to_string(), v);
                    }
                }
                let sp = raw_str(&r.conditions["fuel_species"]);
                for s in &species {
                    feats.insert(format!("fuel_is_{s}"), if *s == sp { 1.0 } else { 0.0 });
                }
                views.push(View::Acme(AcmeEventRow {
                    run_id: rid.clone(),
                    group_id: r2g[rid].clone(),
                    configuration: r.configuration.clone().unwrap_or_default(),
                    features: feats,
                    t_entry_s: 0.0,
                    censoring: crate::common::enum_str(e.censoring),
                    t_lower_s: e.last_confirmed_active_s,
                    t_upper_s: e.first_terminal_s,
                    allowed_causes: e.allowed_causes.clone(),
                    scheduled_stop_s: None,
                    time_origin: ACME_TIME_ORIGIN.into(),
                }));
            }
            meta["time_origin"] = json!(ACME_TIME_ORIGIN);
            meta["fuel_reference"] = json!(species);
        }
        Family::Bass => {
            let mut vids: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for v in crate::families::bass::video_map(&b) {
                if let Some(rid) = v.run_id.filter(|r| r2g.contains_key(r)) {
                    if v.resolution == "resolved" && v.kind == "video" {
                        vids.entry(rid).or_default().push(v.file_name);
                    }
                }
            }
            let mut controls: BTreeMap<String, Vec<f64>> = BTreeMap::new();
            for h in b.extra.get("control_history").into_iter().flatten() {
                if let Ok(ch) =
                    serde_json::from_value::<ignis_core::contracts::ControlHistory>(h.clone())
                {
                    controls
                        .entry(ch.run_id)
                        .or_default()
                        .extend(ch.stages.iter().map(|s| s.t_start_s));
                }
            }
            let mut videos = serde_json::Map::new();
            for rid in &coh.run_ids {
                let r = recs[rid.as_str()];
                let geom: BTreeMap<String, f64> = r
                    .specimen_id
                    .as_ref()
                    .and_then(|s| b.specimens.get(s))
                    .map(|sp| {
                        sp.geometry
                            .iter()
                            .filter_map(|(k, m)| m.value_si.map(|v| (k.clone(), v)))
                            .collect()
                    })
                    .unwrap_or_default();
                let mut cc = controls.remove(rid).unwrap_or_default();
                cc.sort_by(f64::total_cmp);
                let mut vs = vids.remove(rid).unwrap_or_default();
                vs.sort();
                videos.insert(rid.clone(), json!(vs));
                views.push(View::Bass(BassSequence {
                    run_id: rid.clone(),
                    group_id: r2g[rid].clone(),
                    t_s: vec![],
                    s_m: vec![],
                    f_m: vec![],
                    valid: vec![],
                    status: vec![],
                    conditions: numeric_conditions(r, allowed),
                    geometry: geom,
                    control_changes_s: cc,
                    scale_uncertainty_rel: vec![],
                    time_origin: BASS_TIME_ORIGIN.into(),
                }));
            }
            meta["note"] = json!("metadata only: contours f(s,t) are attached by the vision lane");
            meta["time_origin"] = json!(BASS_TIME_ORIGIN);
            meta["videos"] = serde_json::Value::Object(videos);
        }
        Family::Other => anyhow::bail!("no views for family 'other'"),
    }
    Ok((coh, views, meta))
}

/// `ignis data views <cohort>`: write the Parquet view file; returns its directory.
pub fn build(cohort_id: &str) -> anyhow::Result<PathBuf> {
    let (coh, views, meta) = build_views(cohort_id)?;
    let out = view_dir(&coh.id);
    std::fs::create_dir_all(&out)?;
    write_parquet(&out.join(file_for(coh.family)), &views, &meta)?;
    Ok(out)
}

// ------------------------------------------------------------------ parquet columns

fn s_col(v: impl IntoIterator<Item = String>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(v))
}
fn f_col(v: impl IntoIterator<Item = f64>) -> ArrayRef {
    Arc::new(Float64Array::from_iter_values(v))
}
fn fo_col(v: impl IntoIterator<Item = Option<f64>>) -> ArrayRef {
    Arc::new(v.into_iter().collect::<Float64Array>())
}
fn bo_col(v: impl IntoIterator<Item = Option<bool>>) -> ArrayRef {
    Arc::new(v.into_iter().collect::<BooleanArray>())
}
fn fl_col<'a>(v: impl IntoIterator<Item = &'a Vec<f64>>) -> ArrayRef {
    let mut b = ListBuilder::new(Float64Builder::new());
    for xs in v {
        b.values().append_slice(xs);
        b.append(true);
    }
    Arc::new(b.finish())
}
fn bl_col<'a>(v: impl IntoIterator<Item = &'a Vec<bool>>) -> ArrayRef {
    let mut b = ListBuilder::new(BooleanBuilder::new());
    for xs in v {
        b.values().append_slice(xs);
        b.append(true);
    }
    Arc::new(b.finish())
}
fn sl_col<'a>(v: impl IntoIterator<Item = &'a Vec<String>>) -> ArrayRef {
    let mut b = ListBuilder::new(StringBuilder::new());
    for xs in v {
        for x in xs {
            b.values().append_value(x);
        }
        b.append(true);
    }
    Arc::new(b.finish())
}
fn js(m: &BTreeMap<String, f64>) -> String {
    serde_json::to_string(m).unwrap()
}

fn list(dt: DataType) -> DataType {
    DataType::List(Arc::new(Field::new("item", dt, true)))
}

pub fn write_parquet(path: &Path, views: &[View], meta: &serde_json::Value) -> anyhow::Result<()> {
    let f = |n: &str, dt: DataType, null: bool| Field::new(n, dt, null);
    let (fields, cols): (Vec<Field>, Vec<ArrayRef>) = match views.first() {
        _ if path.ends_with("flex_trajectory.parquet") => {
            let v: Vec<&FlexTrajectory> = views
                .iter()
                .filter_map(|v| if let View::Flex(x) = v { Some(x) } else { None })
                .collect();
            (
                vec![
                    f("run_id", DataType::Utf8, false),
                    f("group_id", DataType::Utf8, false),
                    f("t_s", list(DataType::Float64), true),
                    f("available_at_s", list(DataType::Float64), true),
                    f("d_m", list(DataType::Float64), true),
                    f("valid", list(DataType::Boolean), true),
                    f("conditions", DataType::Utf8, false),
                    f("censoring", DataType::Utf8, false),
                    f("t_last_active_s", DataType::Float64, true),
                    f("t_first_terminal_s", DataType::Float64, true),
                    f("allowed_causes", list(DataType::Utf8), true),
                    f("event_before_support_exit", DataType::Boolean, true),
                    f("time_origin", DataType::Utf8, false),
                    f("evidence_class", DataType::Utf8, false),
                ],
                vec![
                    s_col(v.iter().map(|x| x.run_id.clone())),
                    s_col(v.iter().map(|x| x.group_id.clone())),
                    fl_col(v.iter().map(|x| &x.t_s)),
                    fl_col(v.iter().map(|x| &x.available_at_s)),
                    fl_col(v.iter().map(|x| &x.d_m)),
                    bl_col(v.iter().map(|x| &x.valid)),
                    s_col(v.iter().map(|x| js(&x.conditions))),
                    s_col(v.iter().map(|x| x.censoring.clone())),
                    fo_col(v.iter().map(|x| x.t_last_active_s)),
                    fo_col(v.iter().map(|x| x.t_first_terminal_s)),
                    sl_col(v.iter().map(|x| &x.allowed_causes)),
                    bo_col(v.iter().map(|x| x.event_before_support_exit)),
                    s_col(v.iter().map(|x| x.time_origin.clone())),
                    s_col(v.iter().map(|x| x.evidence_class.clone())),
                ],
            )
        }
        _ if path.ends_with("acme_event_rows.parquet") => {
            let v: Vec<&AcmeEventRow> = views
                .iter()
                .filter_map(|v| if let View::Acme(x) = v { Some(x) } else { None })
                .collect();
            (
                vec![
                    f("run_id", DataType::Utf8, false),
                    f("group_id", DataType::Utf8, false),
                    f("configuration", DataType::Utf8, false),
                    f("features", DataType::Utf8, false),
                    f("t_entry_s", DataType::Float64, false),
                    f("censoring", DataType::Utf8, false),
                    f("t_lower_s", DataType::Float64, true),
                    f("t_upper_s", DataType::Float64, true),
                    f("allowed_causes", list(DataType::Utf8), true),
                    f("scheduled_stop_s", DataType::Float64, true),
                    f("time_origin", DataType::Utf8, false),
                ],
                vec![
                    s_col(v.iter().map(|x| x.run_id.clone())),
                    s_col(v.iter().map(|x| x.group_id.clone())),
                    s_col(v.iter().map(|x| x.configuration.clone())),
                    s_col(v.iter().map(|x| js(&x.features))),
                    f_col(v.iter().map(|x| x.t_entry_s)),
                    s_col(v.iter().map(|x| x.censoring.clone())),
                    fo_col(v.iter().map(|x| x.t_lower_s)),
                    fo_col(v.iter().map(|x| x.t_upper_s)),
                    sl_col(v.iter().map(|x| &x.allowed_causes)),
                    fo_col(v.iter().map(|x| x.scheduled_stop_s)),
                    s_col(v.iter().map(|x| x.time_origin.clone())),
                ],
            )
        }
        _ => {
            let v: Vec<&BassSequence> = views
                .iter()
                .filter_map(|v| if let View::Bass(x) = v { Some(x) } else { None })
                .collect();
            let videos = meta.get("videos").cloned().unwrap_or_default();
            let vids: Vec<Vec<String>> = v
                .iter()
                .map(|x| {
                    serde_json::from_value(videos.get(&x.run_id).cloned().unwrap_or(json!([])))
                        .unwrap_or_default()
                })
                .collect();
            (
                vec![
                    f("run_id", DataType::Utf8, false),
                    f("group_id", DataType::Utf8, false),
                    f("t_s", list(DataType::Float64), true),
                    f("s_m", list(DataType::Float64), true),
                    f("f_m", list(DataType::Float64), true),
                    f("valid", list(DataType::Boolean), true),
                    f("status", list(DataType::Utf8), true),
                    f("conditions", DataType::Utf8, false),
                    f("geometry", DataType::Utf8, false),
                    f("control_changes_s", list(DataType::Float64), true),
                    f("scale_uncertainty_rel", list(DataType::Float64), true),
                    f("time_origin", DataType::Utf8, false),
                    f("videos", list(DataType::Utf8), true),
                ],
                vec![
                    s_col(v.iter().map(|x| x.run_id.clone())),
                    s_col(v.iter().map(|x| x.group_id.clone())),
                    fl_col(v.iter().map(|x| &x.t_s)),
                    fl_col(v.iter().map(|x| &x.s_m)),
                    fl_col(v.iter().map(|x| &x.f_m)),
                    bl_col(v.iter().map(|x| &x.valid)),
                    sl_col(v.iter().map(|x| &x.status)),
                    s_col(v.iter().map(|x| js(&x.conditions))),
                    s_col(v.iter().map(|x| js(&x.geometry))),
                    fl_col(v.iter().map(|x| &x.control_changes_s)),
                    fl_col(v.iter().map(|x| &x.scale_uncertainty_rel)),
                    s_col(v.iter().map(|x| x.time_origin.clone())),
                    sl_col(vids.iter()),
                ],
            )
        }
    };
    let schema = Arc::new(Schema::new(fields));
    let batch = RecordBatch::try_new(schema.clone(), cols)?;
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .set_key_value_metadata(Some(vec![KeyValue::new(
            "ignis".to_string(),
            meta.to_string(),
        )]))
        .build();
    let tmp = path.with_extension("parquet.part");
    let mut w = ArrowWriter::try_new(File::create(&tmp)?, schema, Some(props))?;
    w.write(&batch)?;
    w.close()?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

struct Cols<'a>(&'a RecordBatch);

impl<'a> Cols<'a> {
    fn c(&self, n: &str) -> anyhow::Result<&'a ArrayRef> {
        self.0
            .column_by_name(n)
            .with_context(|| format!("view column {n} missing"))
    }
    fn s(&self, n: &str, i: usize) -> anyhow::Result<String> {
        Ok(self.c(n)?.as_string::<i32>().value(i).to_string())
    }
    fn fo(&self, n: &str, i: usize) -> anyhow::Result<Option<f64>> {
        let a = self.c(n)?.as_primitive::<Float64Type>();
        Ok(a.is_valid(i).then(|| a.value(i)))
    }
    fn fl(&self, n: &str, i: usize) -> anyhow::Result<Vec<f64>> {
        let l = self.c(n)?.as_list::<i32>().value(i);
        Ok(l.as_primitive::<Float64Type>().values().to_vec())
    }
    fn bl(&self, n: &str, i: usize) -> anyhow::Result<Vec<bool>> {
        let l = self.c(n)?.as_list::<i32>().value(i);
        Ok(l.as_boolean().iter().map(|b| b.unwrap_or(false)).collect())
    }
    fn sl(&self, n: &str, i: usize) -> anyhow::Result<Vec<String>> {
        let l = self.c(n)?.as_list::<i32>().value(i);
        Ok(l.as_string::<i32>()
            .iter()
            .map(|s| s.unwrap_or_default().to_string())
            .collect())
    }
    fn bo(&self, n: &str, i: usize) -> anyhow::Result<Option<bool>> {
        let a = self.c(n)?.as_boolean();
        Ok(a.is_valid(i).then(|| a.value(i)))
    }
    fn map(&self, n: &str, i: usize) -> anyhow::Result<BTreeMap<String, f64>> {
        Ok(serde_json::from_str(&self.s(n, i)?)?)
    }
}

/// `view_version` recorded in a view file's `ignis` metadata, if readable.
pub fn stored_view_version(path: &Path) -> Option<String> {
    let f = File::open(path).ok()?;
    let b = ParquetRecordBatchReaderBuilder::try_new(f).ok()?;
    let kv = b.metadata().file_metadata().key_value_metadata()?;
    let meta = kv.iter().find(|k| k.key == "ignis")?.value.clone()?;
    let v: serde_json::Value = serde_json::from_str(&meta).ok()?;
    v.get("view_version")?.as_str().map(str::to_string)
}

pub fn read_parquet(path: &Path, fam: Family) -> anyhow::Result<Vec<View>> {
    let f = File::open(path)
        .with_context(|| format!("{}: run `ignis data views <cohort>`", path.display()))?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(f)?
        .with_batch_size(1 << 16)
        .build()?;
    let mut out = vec![];
    for batch in reader {
        let batch = batch?;
        let c = Cols(&batch);
        for i in 0..batch.num_rows() {
            out.push(match fam {
                Family::Flex => View::Flex(FlexTrajectory {
                    run_id: c.s("run_id", i)?,
                    group_id: c.s("group_id", i)?,
                    t_s: c.fl("t_s", i)?,
                    available_at_s: c.fl("available_at_s", i)?,
                    d_m: c.fl("d_m", i)?,
                    valid: c.bl("valid", i)?,
                    conditions: c.map("conditions", i)?,
                    censoring: c.s("censoring", i)?,
                    t_last_active_s: c.fo("t_last_active_s", i)?,
                    t_first_terminal_s: c.fo("t_first_terminal_s", i)?,
                    allowed_causes: c.sl("allowed_causes", i)?,
                    event_before_support_exit: c.bo("event_before_support_exit", i)?,
                    time_origin: c.s("time_origin", i)?,
                    evidence_class: c.s("evidence_class", i)?,
                }),
                Family::AcmeNormal | Family::AcmeInverse => View::Acme(AcmeEventRow {
                    run_id: c.s("run_id", i)?,
                    group_id: c.s("group_id", i)?,
                    configuration: c.s("configuration", i)?,
                    features: c.map("features", i)?,
                    t_entry_s: c.fo("t_entry_s", i)?.unwrap_or(0.0),
                    censoring: c.s("censoring", i)?,
                    t_lower_s: c.fo("t_lower_s", i)?,
                    t_upper_s: c.fo("t_upper_s", i)?,
                    allowed_causes: c.sl("allowed_causes", i)?,
                    scheduled_stop_s: c.fo("scheduled_stop_s", i)?,
                    time_origin: c.s("time_origin", i)?,
                }),
                _ => View::Bass(BassSequence {
                    run_id: c.s("run_id", i)?,
                    group_id: c.s("group_id", i)?,
                    t_s: c.fl("t_s", i)?,
                    s_m: c.fl("s_m", i)?,
                    f_m: c.fl("f_m", i)?,
                    valid: c.bl("valid", i)?,
                    status: c.sl("status", i)?,
                    conditions: c.map("conditions", i)?,
                    geometry: c.map("geometry", i)?,
                    control_changes_s: c.fl("control_changes_s", i)?,
                    scale_uncertainty_rel: c.fl("scale_uncertainty_rel", i)?,
                    time_origin: c.s("time_origin", i)?,
                }),
            });
        }
    }
    Ok(out)
}

/// The data lane's [`ViewSource`]: cohorts from `data/work/cohorts` (or embedded in frozen splits),
/// verified frozen splits from `data/splits`, views from Parquet (built on first use if absent).
#[derive(Debug, Clone, Default)]
pub struct DataStore {
    /// Optional data root; `None` uses `IGNIS_DATA` / `data`.
    pub root: Option<PathBuf>,
}

impl DataStore {
    pub fn new() -> DataStore {
        DataStore { root: None }
    }
    fn scoped<T>(&self, f: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<T> {
        match &self.root {
            None => f(),
            Some(r) => {
                // SAFETY of the env swap: single-threaded CLI/test use; paths read IGNIS_DATA on each call.
                let old = std::env::var_os("IGNIS_DATA");
                std::env::set_var("IGNIS_DATA", r);
                let out = f();
                match old {
                    Some(o) => std::env::set_var("IGNIS_DATA", o),
                    None => std::env::remove_var("IGNIS_DATA"),
                }
                out
            }
        }
    }
}

impl ViewSource for DataStore {
    fn cohort(&self, cohort_id: &str) -> anyhow::Result<Cohort> {
        self.scoped(|| load_cohort(cohort_id))
    }
    fn split(&self, split_id: &str) -> anyhow::Result<Split> {
        self.scoped(|| load_split(split_id))
    }
    fn views(&self, family: Family, cohort_id: &str) -> anyhow::Result<Vec<View>> {
        self.scoped(|| {
            let p = view_dir(cohort_id).join(file_for(family));
            // Views are derived data: rebuild when absent or written by another view version.
            if !p.exists() || stored_view_version(&p).as_deref() != Some(VIEW_VERSION) {
                build(cohort_id)?;
            }
            read_parquet(&p, family)
        })
    }
    fn run_archives(&self, family: Family) -> anyhow::Result<Vec<RunArchive>> {
        self.scoped(|| {
            let dir = paths::work(&["records", family.as_str()]);
            anyhow::ensure!(
                dir.join("experiments.jsonl").exists(),
                "no ingested records for {}; run `ignis data ingest {}`",
                family.as_str(),
                family.as_str()
            );
            let reg = Registry::load(&dir);
            anyhow::ensure!(
                reg.errors.is_empty(),
                "records of {} do not match the contracts ({} errors), first: {}",
                family.as_str(),
                reg.errors.len(),
                reg.errors[0]
            );
            Ok(reg
                .experiments
                .keys()
                .filter_map(|id| reg.archive(id))
                .collect())
        })
    }
}

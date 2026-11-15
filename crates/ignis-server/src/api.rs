//! HTTP API (V3.1 §20.2). Every response body is JSON; refusals are typed (see `refusal.rs`).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
pub use ignis_core::analysis::ForecastSettings;
use ignis_core::analysis::{self, cache_key};
use ignis_core::batch::{score_batch, ResidualSeries};
use ignis_core::contracts::*;
use ignis_core::hashing::{content_hash, hex_part, key_of};
use ignis_core::prefix::assert_isolated;
use ignis_core::registry::LoadedRelease;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::watch;
use tower::ServiceExt;

use crate::refusal::{ApiResult, Refusal};
use crate::state::{AnalysisContext, DataOrigin, JobState, Origin, Stored};

type S = State<Arc<Origin>>;

pub fn router(origin: Arc<Origin>) -> Router {
    let label = origin.origin;
    let mut r = Router::new();
    if origin.internal_routes {
        // Evaluation and admin tools: never part of the public runtime by default.
        r = r
            .route("/seal", post(seal))
            .route("/reveal", post(reveal))
            .route("/admin/reload", post(reload));
    } else {
        // A typed refusal, not a bare 404, so clients can tell "disabled" from "server down".
        for p in ["/seal", "/reveal", "/admin/reload"] {
            r = r.route(p, post(internal_route_disabled));
        }
    }
    r.route("/health", get(health))
        .route("/experiments", get(list_experiments))
        .route("/experiments/{id}", get(get_experiment))
        .route("/experiments/{id}/observations", get(get_observations))
        .route("/experiments/{id}/series", get(get_series))
        .route("/experiments/{id}/overlay", get(get_overlay))
        .route("/experiments/{id}/video/{file}", get(get_video))
        .route("/cohorts", get(list_cohorts))
        .route("/evidence/search", post(evidence_search))
        .route("/compare", post(compare))
        .route("/models", get(list_models))
        .route("/models/{id}", get(get_model))
        .route("/forecast", post(forecast))
        .route("/jobs/{id}", get(get_job))
        .route("/analysis/{id}", get(get_analysis))
        .route("/acquisition", post(acquisition))
        .route("/brief", post(brief))
        .route("/score/student-t", post(score_student_t))
        .layer(axum::middleware::map_response(
            move |mut r: Response| async move {
                r.headers_mut().insert(
                    "x-ignis-data-origin",
                    HeaderValue::from_static(match label {
                        DataOrigin::Registry => "registry",
                        DataOrigin::DevelopmentFixture => "development-fixture-synthetic",
                    }),
                );
                r
            },
        ))
        .with_state(origin)
}

/// Strict JSON body: unknown fields and type errors are typed `invalid_request` refusals.
pub struct Strict<T>(pub T);

impl<T: serde::de::DeserializeOwned, St: Send + Sync> axum::extract::FromRequest<St> for Strict<T> {
    type Rejection = Refusal;
    async fn from_request(req: axum::extract::Request, state: &St) -> Result<Self, Refusal> {
        let bytes = axum::body::Bytes::from_request(req, state)
            .await
            .map_err(|e| Refusal::invalid("body", e.to_string()))?;
        serde_json::from_slice(&bytes)
            .map(Strict)
            .map_err(|e| Refusal::invalid("schema", e.to_string()))
    }
}

fn now_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// RFC 3339 UTC timestamp (Python `datetime` parses it) from unix seconds; civil-from-days.
pub fn rfc3339_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

fn origin_value(o: &Origin) -> Value {
    json!({"kind": o.origin, "label": o.origin.label()})
}

// ---------------------------------------------------------------- partition scope

/// Partitions `run` is assigned to across the frozen splits.
fn run_partitions(reg: &ignis_core::records::Registry, run: &str) -> BTreeSet<Partition> {
    reg.splits
        .iter()
        .filter_map(|s| s.partition_of(run))
        .collect()
}

/// May the outcomes of `run` (observations past any cutoff, contours, control history, archive
/// extent) be served under the origin's public scope? Every partition the run is in must be
/// granted; unassigned runs have no evaluation role. The synthetic fixture origin is exempt.
fn outcomes_allowed(o: &Origin, reg: &ignis_core::records::Registry, run: &str) -> bool {
    o.origin == DataOrigin::DevelopmentFixture
        || run_partitions(reg, run)
            .into_iter()
            .all(|p| o.public_scope.require_partition(p).is_ok())
}

fn require_outcomes(o: &Origin, reg: &ignis_core::records::Registry, run: &str) -> ApiResult<()> {
    if o.origin == DataOrigin::DevelopmentFixture {
        return Ok(());
    }
    for p in run_partitions(reg, run) {
        if let Err(d) = o.public_scope.require_partition(p) {
            return Err(Refusal::scope_denied(
                "partition_not_granted",
                format!(
                    "run {run} is in a partition whose outcomes this scope cannot read ({}); \
                     validation continuations open through seal and reveal, test only in the final evaluation",
                    d.reason
                ),
            ));
        }
    }
    Ok(())
}

/// Runs under sealed evaluation (validation or test partition of any frozen split).
fn sealed_runs(o: &Origin) -> BTreeSet<String> {
    if o.origin == DataOrigin::DevelopmentFixture {
        return BTreeSet::new();
    }
    let reg = o.registry();
    reg.experiments
        .keys()
        .filter(|r| {
            run_partitions(&reg, r)
                .iter()
                .any(|p| matches!(p, Partition::Validation | Partition::Test))
        })
        .cloned()
        .collect()
}

fn hit_runs(v: &Value) -> Vec<String> {
    match v {
        Value::Array(a) => a
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        Value::String(s) => match serde_json::from_str::<Vec<String>>(s) {
            Ok(list) => list,
            Err(_) if s.trim().is_empty() => vec![],
            Err(_) => vec![s.clone()],
        },
        _ => vec![],
    }
}

/// Same rule as `ignis_evidence::firewall::strip_for_sealed` with its conservative default: while
/// runs are under sealed evaluation, outcome-bearing hits that describe a sealed run, or no run at
/// all, are dropped. Returns the kept hits and how many were stripped. (This duplicate FTS engine is
/// removed once `KnowledgeService` lands, task M3.)
fn strip_sealed_hits(
    hits: Vec<crate::evidence::Hit>,
    sealed: &BTreeSet<String>,
) -> (Vec<crate::evidence::Hit>, usize) {
    if sealed.is_empty() {
        return (hits, 0);
    }
    let n = hits.len();
    let kept: Vec<_> = hits
        .into_iter()
        .filter(|h| {
            let outcome =
                h.outcome_bearing == Value::from(1) || h.outcome_bearing == Value::Bool(true);
            if !outcome {
                return true;
            }
            let runs = hit_runs(&h.describes_runs);
            !(runs.is_empty() || runs.iter().any(|r| sealed.contains(r)))
        })
        .collect();
    let stripped = n - kept.len();
    (kept, stripped)
}

// ---------------------------------------------------------------- health

async fn health(State(o): S) -> Json<Value> {
    let reg = o.registry();
    let ev = {
        let o = o.clone();
        tokio::task::spawn_blocking(move || o.evidence.status())
            .await
            .ok()
    };
    Json(json!({
        "data_origin": origin_value(&o),
        "schema_version": SCHEMA_VERSION,
        "registry": {"dir": reg.dir.display().to_string(), "files": reg.files, "experiments": reg.experiments.len(),
                      "errors": reg.errors.iter().take(50).collect::<Vec<_>>(), "error_count": reg.errors.len()},
        "models_root": o.models_root.display().to_string(),
        "store_root": o.store_root.display().to_string(),
        "evidence_catalog": ev,
        "experts": o.experts.iter().map(|e| json!({"model_name": e.model_name(), "family": e.family(),
                    "is_baseline": e.is_baseline()})).collect::<Vec<_>>(),
        "execution": {"mode": "in_process", "compute_threads": o.pool.current_num_threads(),
                      "acquisition_scorer": o.acquisition.is_some()},
    }))
}

async fn internal_route_disabled() -> Refusal {
    Refusal::unsupported(
        "internal_route_disabled",
        "seal, reveal and admin reload are evaluation tools, off in the public runtime; \
         start the server with --internal-routes (the fixture origin always has them)",
    )
}

async fn reload(State(o): S) -> Json<Value> {
    let r = o.reload();
    o.cache.invalidate_all();
    Json(json!({"experiments": r.experiments.len(), "errors": r.errors.len()}))
}

// ---------------------------------------------------------------- experiments

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpQuery {
    family: Option<Family>,
    q: Option<String>,
    /// Cohort id or a 16+ hex prefix: only runs of that cohort.
    cohort: Option<String>,
    /// With `split`: `train`, `validation` or `test`.
    partition: Option<Partition>,
    split: Option<String>,
}

fn experiment_summary(
    o: &Origin,
    r: &ExperimentRecord,
    release_families: &BTreeSet<Family>,
) -> Value {
    let reg = o.registry();
    let obs = reg.observations(&r.id);
    let mut by_q: BTreeMap<Quantity, (usize, f64, f64)> = BTreeMap::new();
    for x in obs {
        let e = by_q
            .entry(x.quantity)
            .or_insert((0, f64::INFINITY, f64::NEG_INFINITY));
        e.0 += 1;
        e.1 = e.1.min(x.time_s);
        e.2 = e.2.max(x.time_s);
    }
    let mut gaps = Vec::new();
    if obs.is_empty() && reg.contours(&r.id).is_empty() {
        gaps.push("no observations in the registry".to_string());
    }
    if !reg.endpoints.contains_key(&r.id) {
        gaps.push("no adjudicated endpoint record".to_string());
    }
    if r.identity_resolution != Resolution::Resolved {
        gaps.push(format!("identity {:?}", r.identity_resolution).to_lowercase());
    }
    if !reg.controls.contains_key(&r.id) {
        gaps.push("no control history (time origin unadjudicated)".to_string());
    }
    for (k, m) in &r.conditions {
        if m.value_si.is_none() {
            gaps.push(format!("condition {k} has no SI value ({:?})", m.resolution).to_lowercase());
        }
    }
    if r.gravity.as_ref().and_then(|g| g.value_si).is_none() {
        gaps.push("gravity level not established".to_string());
    }
    if !release_families.contains(&r.family) {
        gaps.push(format!("no model release for family {}", r.family.as_str()));
    }
    // Counts and archive extent of a sealed run reveal its duration (the outcome).
    let withheld = !outcomes_allowed(o, &reg, &r.id);
    if withheld {
        by_q.clear();
    }
    json!({
        "id": r.id, "investigation_id": r.investigation_id, "family": r.family, "fuel_or_material": r.fuel_or_material,
        "configuration": r.configuration, "session_id": r.session_id,
        "gravity": r.gravity.as_ref().map(|g| json!({"value_si": g.value_si, "unit_si": g.unit_si})),
        "conditions": r.conditions.iter().map(|(k, m)| (k.clone(), json!({"value_si": m.value_si, "unit_si": m.unit_si,
            "resolution": m.resolution, "evidence_class": m.evidence_class}))).collect::<serde_json::Map<_, _>>(),
        "quantities": by_q.iter().map(|(q, (n, a, b))| json!({"quantity": q, "unit_si": q.unit_si(), "count": n,
            "archive_t_min_s": a, "archive_t_max_s": b})).collect::<Vec<_>>(),
        "n_observations": (!withheld).then_some(obs.len()), "n_contours": (!withheld).then(|| reg.contours(&r.id).len()),
        "outcomes_withheld": withheld,
        "has_endpoint": reg.endpoints.contains_key(&r.id), "identity_resolution": r.identity_resolution,
        "quality_flags": r.quality_flags, "synthetic": reg.is_synthetic(&r.id), "gaps": gaps,
        // NASA's own names for the run (test sheet number, table test, as-run number), names only.
        "aliases": r.aliases.iter().map(|a| json!({"scheme": a.scheme, "name": a.name})).collect::<Vec<_>>(),
    })
}

/// Find by full id or a hex prefix of at least 16 characters.
fn find_by_id<'a, T>(items: impl Iterator<Item = (&'a str, T)>, id: &str) -> Option<T> {
    let want = hex_part(id);
    items
        .filter(|(k, _)| hex_part(k) == want || (want.len() >= 16 && hex_part(k).starts_with(want)))
        .map(|(_, v)| v)
        .next()
}

async fn list_experiments(State(o): S, Query(q): Query<ExpQuery>) -> ApiResult<Json<Value>> {
    let reg = o.registry();
    let fams: BTreeSet<Family> = o.list_releases().0.iter().map(|r| r.family).collect();
    let needle = q.q.as_deref().map(str::to_lowercase);
    let cohort: Option<BTreeSet<&str>> = match &q.cohort {
        None => None,
        Some(c) => {
            let found = find_by_id(reg.cohorts.iter().map(|(k, v)| (k.as_str(), v)), c)
                .ok_or_else(|| Refusal::not_found("cohort_not_found", format!("cohort {c}")))?;
            Some(found.run_ids.iter().map(String::as_str).collect())
        }
    };
    let split = match (&q.split, q.partition) {
        (None, None) => None,
        (Some(s), Some(p)) => {
            let found = find_by_id(reg.splits.iter().map(|x| (x.id.as_str(), x)), s)
                .ok_or_else(|| Refusal::not_found("split_not_found", format!("split {s}")))?;
            Some((found, p))
        }
        _ => {
            return Err(Refusal::invalid(
                "partition",
                "pass split and partition together",
            ))
        }
    };
    let items: Vec<Value> = reg
        .experiments
        .values()
        .filter(|r| q.family.is_none_or(|f| f == r.family))
        .filter(|r| cohort.as_ref().is_none_or(|c| c.contains(r.id.as_str())))
        .filter(|r| split.is_none_or(|(s, p)| s.partition_of(&r.id) == Some(p)))
        .filter(|r| {
            needle.as_ref().is_none_or(|n| {
                [
                    Some(&r.id),
                    Some(&r.investigation_id),
                    r.fuel_or_material.as_ref(),
                    r.configuration.as_ref(),
                ]
                .into_iter()
                .flatten()
                .any(|s| s.to_lowercase().contains(n))
            })
        })
        .map(|r| experiment_summary(&o, r, &fams))
        .collect();
    Ok(Json(
        json!({"data_origin": origin_value(&o), "count": items.len(), "experiments": items,
                   "registry_errors": reg.errors.len()}),
    ))
}

/// Cohorts and the frozen splits of each, for the Explore filter.
async fn list_cohorts(State(o): S) -> Json<Value> {
    let reg = o.registry();
    let cohorts: Vec<Value> = reg
        .cohorts
        .values()
        .map(|c| {
            let splits: Vec<Value> = reg
                .splits
                .iter()
                .filter(|s| s.cohort_id == c.id)
                .map(|s| {
                    let count = |p: Partition| c.run_ids.iter().filter(|r| s.partition_of(r) == Some(p)).count();
                    json!({"id": s.id, "seed": s.seed, "frozen_at": s.frozen_at, "group_key": s.group_key,
                           "runs": {"train": count(Partition::Train), "validation": count(Partition::Validation), "test": count(Partition::Test)}})
                })
                .collect();
            json!({"id": c.id, "family": c.family, "version": c.version, "criteria": c.criteria, "n_runs": c.run_ids.len(),
                   "n_excluded": c.excluded.len(), "source_manifest_hash": c.source_manifest_hash, "splits": splits})
        })
        .collect();
    Json(json!({"data_origin": origin_value(&o), "count": cohorts.len(), "cohorts": cohorts}))
}

async fn get_experiment(State(o): S, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let reg = o.registry();
    let r = reg
        .experiments
        .get(&id)
        .ok_or_else(|| Refusal::not_found("run_not_found", format!("run {id}")))?;
    let fams: BTreeSet<Family> = o.list_releases().0.iter().map(|r| r.family).collect();
    // Outcomes (control history, how the run ended) only for runs whose every partition this
    // scope may read; a validation or test run gets null, never a hint of its duration.
    let outcomes = outcomes_allowed(&o, &reg, &id);
    let releases: Vec<Value> = o
        .list_releases()
        .0
        .iter()
        .filter(|m| m.family == r.family)
        .map(release_summary)
        .collect();
    Ok(Json(json!({
        "data_origin": origin_value(&o),
        "record": r,
        "summary": experiment_summary(&o, r, &fams),
        "control_history": if outcomes { json!(reg.controls.get(&id)) } else { Value::Null },
        "endpoint": if outcomes { json!(reg.endpoints.get(&id)) } else { Value::Null },
        "releases": releases,
        "note": "Observation values are served per cutoff (/experiments/{id}/observations). The endpoint record is \
                 served here only for runs whose outcomes this scope reads (train); for validation runs the \
                 continuation and the endpoint open through /reveal after a sealed prediction.",
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObsQuery {
    cutoff_s: f64,
}

async fn get_observations(
    State(o): S,
    Path(id): Path<String>,
    Query(q): Query<ObsQuery>,
) -> ApiResult<Json<Value>> {
    if !q.cutoff_s.is_finite() {
        return Err(Refusal::invalid("cutoff", "cutoff_s must be finite"));
    }
    let reg = o.registry();
    if !reg.experiments.contains_key(&id) {
        return Err(Refusal::not_found("run_not_found", format!("run {id}")));
    }
    require_outcomes(&o, &reg, &id)?;
    let obs = reg.observations(&id);
    let admitted: Vec<&Observation> = obs.iter().filter(|x| x.admitted_at(q.cutoff_s)).collect();
    let pending: Vec<Value> = obs
        .iter()
        .filter(|x| x.time_s <= q.cutoff_s && x.available_at_s > q.cutoff_s)
        .map(|x| json!({"id": x.id, "time_s": x.time_s, "quantity": x.quantity, "available_at_s": x.available_at_s}))
        .collect();
    let hidden = obs.iter().filter(|x| x.time_s > q.cutoff_s).count();
    let contours: Vec<&ContourObservation> = reg
        .contours(&id)
        .iter()
        .filter(|c| c.available_at_s <= q.cutoff_s && c.time_s <= q.cutoff_s)
        .collect();
    Ok(Json(
        json!({"run_id": id, "cutoff_s": q.cutoff_s, "admitted": admitted, "contours": contours,
                   "pending": pending, "hidden_after_cutoff": hidden}),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SeriesQuery {
    cutoff_s: f64,
    quantity: Option<Quantity>,
}

/// The admitted measurements at a cutoff, as compact columns per (quantity, channel), for plotting.
async fn get_series(
    State(o): S,
    Path(id): Path<String>,
    Query(q): Query<SeriesQuery>,
) -> ApiResult<Json<Value>> {
    if !q.cutoff_s.is_finite() {
        return Err(Refusal::invalid("cutoff", "cutoff_s must be finite"));
    }
    let reg = o.registry();
    if !reg.experiments.contains_key(&id) {
        return Err(Refusal::not_found("run_not_found", format!("run {id}")));
    }
    require_outcomes(&o, &reg, &id)?;
    let mut series: BTreeMap<(Quantity, Option<String>), Vec<&Observation>> = BTreeMap::new();
    for x in reg
        .observations(&id)
        .iter()
        .filter(|x| x.admitted_at(q.cutoff_s) && q.quantity.is_none_or(|w| w == x.quantity))
    {
        series
            .entry((x.quantity, x.channel.clone()))
            .or_default()
            .push(x);
    }
    let rows: Vec<Value> = series
        .into_iter()
        .map(|((quantity, channel), v)| {
            json!({"quantity": quantity, "channel": channel, "unit_si": quantity.unit_si(), "n": v.len(),
                   "t_s": v.iter().map(|x| x.time_s).collect::<Vec<_>>(),
                   "value_si": v.iter().map(|x| x.value_si).collect::<Vec<_>>(),
                   "valid": v.iter().map(|x| ignis_core::views::measurement_valid(x)).collect::<Vec<_>>()})
        })
        .collect();
    let hidden = reg
        .observations(&id)
        .iter()
        .filter(|x| x.time_s > q.cutoff_s)
        .count();
    Ok(Json(
        json!({"run_id": id, "cutoff_s": q.cutoff_s, "series": rows, "hidden_after_cutoff": hidden}),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverlayQuery {
    cutoff_s: f64,
}

/// A run's recording belongs to the evaluation when it is in a validation or test partition: the
/// clip would show the sealed continuation, so its file and links are withheld.
fn recording_sealed(reg: &ignis_core::records::Registry, run: &str) -> bool {
    reg.splits.iter().any(|s| {
        matches!(
            s.partition_of(run),
            Some(Partition::Validation | Partition::Test)
        )
    })
}

fn local_video(o: &Origin, row: &Value) -> Option<std::path::PathBuf> {
    let (inv, file) = (
        row.get("investigation")?.as_str()?,
        row.get("file_name")?.as_str()?,
    );
    if file.contains(['/', '\\']) || file.contains("..") {
        return None;
    }
    let path = o.raw_samples_root.join(inv).join(file);
    path.is_file().then_some(path)
}

/// Source of the BASS video/contour overlay: the run's mapped recordings and the contours admitted
/// at the cutoff (`artifacts/vision/.../contours.jsonl`).
async fn get_overlay(
    State(o): S,
    Path(id): Path<String>,
    Query(q): Query<OverlayQuery>,
) -> ApiResult<Json<Value>> {
    if !q.cutoff_s.is_finite() {
        return Err(Refusal::invalid("cutoff", "cutoff_s must be finite"));
    }
    let reg = o.registry();
    let rec = reg
        .experiments
        .get(&id)
        .ok_or_else(|| Refusal::not_found("run_not_found", format!("run {id}")))?;
    require_outcomes(&o, &reg, &id)?;
    let sealed = recording_sealed(&reg, &id);
    let videos: Vec<Value> = reg
        .videos
        .get(&id)
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .map(|row| {
            let file = row.get("file_name").and_then(Value::as_str).unwrap_or_default();
            let local = local_video(&o, row).is_some();
            json!({"file_name": file, "kind": row.get("kind"), "variant": row.get("variant"), "size_bytes": row.get("size_bytes"),
                   "resolution": row.get("resolution"), "evidence": row.get("evidence"), "available_locally": local,
                   "video_url": (local && !sealed).then(|| format!("/experiments/{id}/video/{file}")),
                   "nasa_url": (!sealed).then(|| row.get("stable_url")).flatten()})
        })
        .collect();
    let contours: Vec<&ContourObservation> = reg
        .contours(&id)
        .iter()
        .filter(|c| c.available_at_s <= q.cutoff_s && c.time_s <= q.cutoff_s)
        .collect();
    let hidden = reg
        .contours(&id)
        .iter()
        .filter(|c| c.time_s > q.cutoff_s)
        .count();
    let note = if sealed {
        "validation/test run: recording withheld (it would show the sealed continuation)"
    } else {
        "recording served for development runs"
    };
    Ok(Json(
        json!({"run_id": id, "family": rec.family, "cutoff_s": q.cutoff_s, "videos": videos, "contours": contours,
                   "hidden_after_cutoff": hidden, "sealed_recording": sealed, "time_base": "contour time_s and video PTS share the run's time origin", "note": note}),
    ))
}

/// Serve a mapped recording of a non-sealed run (byte ranges, so the browser can seek).
async fn get_video(
    State(o): S,
    Path((id, file)): Path<(String, String)>,
    req: axum::extract::Request,
) -> ApiResult<Response> {
    let reg = o.registry();
    if !reg.experiments.contains_key(&id) {
        return Err(Refusal::not_found("run_not_found", format!("run {id}")));
    }
    if recording_sealed(&reg, &id) {
        return Err(Refusal::unsupported("recording_sealed", "this run is in a validation/test partition; its recording would show the sealed continuation"));
    }
    let row = reg.videos.get(&id).and_then(|v| {
        v.iter()
            .find(|r| r.get("file_name").and_then(Value::as_str) == Some(file.as_str()))
    });
    let path = row.and_then(|r| local_video(&o, r)).ok_or_else(|| {
        Refusal::not_found(
            "video_not_found",
            format!("{file} is not a local recording of {id}"),
        )
    })?;
    let served = tower_http::services::ServeFile::new(path)
        .oneshot(req)
        .await
        .map_err(|e| Refusal::failed("video_io", e.to_string()))?;
    Ok(served.into_response())
}

// ---------------------------------------------------------------- evidence

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchReq {
    query: String,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    outcome_bearing: Option<bool>,
}

async fn run_search(
    o: &Arc<Origin>,
    query: String,
    limit: usize,
    ob: Option<bool>,
) -> ApiResult<Vec<crate::evidence::Hit>> {
    let o = o.clone();
    tokio::task::spawn_blocking(move || o.evidence.search(&query, limit, ob))
        .await
        .map_err(|e| Refusal::failed("join", e.to_string()))?
}

async fn evidence_search(State(o): S, Strict(r): Strict<SearchReq>) -> ApiResult<Json<Value>> {
    if r.query.len() > 2000 {
        return Err(Refusal::invalid(
            "query_too_long",
            "query is limited to 2000 bytes",
        ));
    }
    let limit = r.limit.unwrap_or(20).clamp(1, 100);
    let hits = run_search(&o, r.query.clone(), limit, r.outcome_bearing).await?;
    let (hits, stripped) = strip_sealed_hits(hits, &sealed_runs(&o));
    let reg = o.registry();
    let hits: Vec<Value> = hits
        .into_iter()
        .map(|h| {
            let src = h.source_id.as_str().and_then(|s| reg.sources.get(s));
            json!({"hit": h, "source": src})
        })
        .collect();
    Ok(Json(
        json!({"query": r.query, "ranking": "fts5 bm25 (lexical)", "hits": hits, "stripped_for_sealed_runs": stripped,
                   "highlight": {"open": crate::evidence::HIT_OPEN, "close": crate::evidence::HIT_CLOSE}}),
    ))
}

// ---------------------------------------------------------------- compare

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompareReq {
    run_ids: Vec<String>,
    #[serde(default)]
    quantity: Option<Quantity>,
}

async fn compare(State(o): S, Strict(r): Strict<CompareReq>) -> ApiResult<Json<Value>> {
    if r.run_ids.len() < 2 || r.run_ids.len() > 8 {
        return Err(Refusal::invalid("run_count", "compare takes 2 to 8 runs"));
    }
    let reg = o.registry();
    let recs: Vec<&ExperimentRecord> = r
        .run_ids
        .iter()
        .map(|id| {
            reg.experiments
                .get(id)
                .ok_or_else(|| Refusal::not_found("run_not_found", format!("run {id}")))
        })
        .collect::<ApiResult<_>>()?;
    let families: BTreeSet<Family> = recs.iter().map(|x| x.family).collect();
    let mut refusals: Vec<Refusal> = Vec::new();
    if families.len() > 1 {
        refusals.push(Refusal::unsupported(
            "incompatible_family",
            format!(
                "runs span families {:?}; observables and endpoints are defined per family, so only a contextual \
                 (side-by-side) comparison is shown",
                families.iter().map(|f| f.as_str()).collect::<Vec<_>>()
            ),
        ));
    }
    if let Some(q) = r.quantity {
        let missing: Vec<&str> = recs
            .iter()
            .filter(|x| !reg.observations(&x.id).iter().any(|ob| ob.quantity == q))
            .map(|x| x.id.as_str())
            .collect();
        if !missing.is_empty() {
            refusals.push(Refusal::unsupported(
                "incompatible_quantity",
                format!("{q:?} is not observed in {missing:?}; equal units do not make other quantities interchangeable"),
            ));
        }
    }
    if recs.iter().any(|x| reg.is_synthetic(&x.id)) && recs.iter().any(|x| !reg.is_synthetic(&x.id))
    {
        refusals.push(Refusal::unsupported(
            "synthetic_mixed",
            "synthetic and non-synthetic runs are never compared numerically",
        ));
    }
    let keys: BTreeSet<&String> = recs.iter().flat_map(|x| x.conditions.keys()).collect();
    let rows: Vec<Value> = keys
        .iter()
        .map(|k| {
            let cells: Vec<Option<&Measured>> = recs.iter().map(|x| x.conditions.get(*k)).collect();
            let units: BTreeSet<Option<&String>> = cells.iter().map(|c| c.and_then(|m| m.unit_si.as_ref())).collect();
            let reason = if cells.iter().any(Option::is_none) {
                Some("not recorded for every run")
            } else if cells.iter().any(|c| c.unwrap().value_si.is_none()) {
                Some("a value is unresolved (no SI value)")
            } else if units.len() > 1 {
                Some("SI units differ")
            } else if families.len() > 1 {
                Some("families differ")
            } else {
                None
            };
            json!({"key": k, "values": cells.iter().map(|c| c.map(|m| json!({"value_si": m.value_si, "unit_si": m.unit_si,
                "resolution": m.resolution, "evidence_class": m.evidence_class}))).collect::<Vec<_>>(),
                "comparable": reason.is_none(), "reason": reason})
        })
        .collect();
    let numeric_allowed = refusals.is_empty();
    let kind = if families.len() > 1 {
        "contextual"
    } else {
        "same_family_conditions"
    };
    Ok(Json(json!({
        "run_ids": r.run_ids, "quantity": r.quantity, "comparison_kind": kind,
        "columns": recs.iter().map(|x| json!({"id": x.id, "family": x.family, "investigation_id": x.investigation_id,
            "fuel_or_material": x.fuel_or_material, "configuration": x.configuration,
            "gravity": x.gravity.as_ref().and_then(|g| g.value_si), "synthetic": reg.is_synthetic(&x.id)})).collect::<Vec<_>>(),
        "rows": rows,
        "numeric_comparison": {"allowed": numeric_allowed, "refusals": refusals,
            "note": "condition differences are model-conditioned descriptions, never causal effects"},
    })))
}

// ---------------------------------------------------------------- models

fn release_summary(r: &ModelRelease) -> Value {
    json!({"id": r.id, "family": r.family, "model_name": r.model_name, "is_baseline": r.is_baseline, "state": r.state,
           "trained_on": r.trained_on, "created_at": r.created_at, "cohort_id": r.cohort_id, "split_id": r.split_id,
           "code_version": r.code_version, "parent_id": r.parent_id, "lineage": r.lineage, "n_tensors": r.tensors.len(),
           "n_posterior": r.posterior.len(), "feature_order": r.feature_order, "evaluation": r.evaluation})
}

async fn list_models(State(o): S) -> Json<Value> {
    let (rels, errs) = o.list_releases();
    let experts: Vec<Value> = o
        .experts
        .iter()
        .map(|e| {
            let mine: Vec<&ModelRelease> = rels.iter().filter(|r| r.model_name == e.model_name()).collect();
            let latest = mine.iter().max_by(|a, b| a.created_at.cmp(&b.created_at));
            json!({"model_name": e.model_name(), "family": e.family(), "is_baseline": e.is_baseline(),
                   "has_candidate_model": e.candidate_predictive().is_some(), "n_releases": mine.len(),
                   "state": latest.map_or(json!("no_release"), |r| json!(r.state)),
                   "trained_on": latest.map(|r| r.trained_on), "latest_release_id": latest.map(|r| r.id.clone()),
                   "latest_created_at": latest.map(|r| r.created_at.clone())})
        })
        .collect();
    Json(
        json!({"data_origin": origin_value(&o), "experts": experts, "models": rels.iter().map(release_summary).collect::<Vec<_>>(),
                "errors": errs}),
    )
}

async fn get_model(State(o): S, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let rel = o.load_release(&id)?;
    let dir = o.release_path(&id)?;
    let rel2 = rel.clone();
    let verification = tokio::task::spawn_blocking(move || verify_tensors(&dir, &rel2))
        .await
        .map_err(|e| Refusal::failed("join", e.to_string()))?;
    Ok(Json(json!({"release": rel, "verification": verification})))
}

fn verify_tensors(dir: &std::path::Path, rel: &ModelRelease) -> Value {
    use ignis_core::npz::{read_npz, tensor_sha256};
    let mut problems = Vec::new();
    for (file, entries) in [
        ("params.npz", &rel.tensors),
        ("posterior.npz", &rel.posterior),
    ] {
        let p = dir.join(file);
        if entries.is_empty() && !p.exists() {
            continue;
        }
        match read_npz(&p) {
            Err(e) => problems.push(format!("{file}: {e}")),
            Ok(t) => {
                let names: BTreeSet<&String> = t.keys().collect();
                let declared: BTreeSet<&String> = entries.iter().map(|e| &e.name).collect();
                if names != declared {
                    problems.push(format!("{file}: tensor set differs from manifest"));
                }
                for e in entries.iter() {
                    if let Some(a) = t.get(&e.name) {
                        if tensor_sha256(a) != e.sha256 {
                            problems.push(format!("{file}:{} hash mismatch", e.name));
                        }
                    }
                }
            }
        }
    }
    json!({"verified": problems.is_empty(), "problems": problems, "method": "npz read in Rust; sha256(dtype|shape|bytes)"})
}

// ---------------------------------------------------------------- forecast

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ForecastReq {
    run_id: String,
    cutoff_s: f64,
    model_release_id: String,
    #[serde(default)]
    settings: ForecastSettings,
    /// Wait up to this long for the job before answering 202 (default 2000, max 10000).
    #[serde(default)]
    wait_ms: Option<u64>,
}

fn artifact_response(s: &Stored, cached: bool) -> Response {
    let refused = !s.artifact.support.admitted
        || matches!(
            s.artifact.status,
            QualificationState::Unsupported
                | QualificationState::InsufficientInference
                | QualificationState::Failed
        );
    Json(json!({"outcome": "artifact", "cached": cached, "refused_by_expert": refused, "artifact": s.artifact,
                "context": s.context}))
    .into_response()
}

async fn forecast(State(o): S, Strict(r): Strict<ForecastReq>) -> ApiResult<Response> {
    let reg = o.registry();
    let archive = reg
        .archive(&r.run_id)
        .ok_or_else(|| Refusal::not_found("run_not_found", format!("run {}", r.run_id)))?;
    let rel = o.load_release(&r.model_release_id)?;
    let synthetic_allowed = o.origin == DataOrigin::DevelopmentFixture;
    // The prefix is built with the expert's own prospective policy; only observations and contours
    // admitted at the cutoff are handed to the expert (`ignis_core::analysis`, shared with the CLI).
    let (ei, prefix, settings_v) = analysis::plan_forecast(
        &o.experts,
        &rel,
        &archive,
        r.cutoff_s,
        &r.settings,
        synthetic_allowed,
        o.origin.label(),
    )
    .map_err(Refusal::from)?;
    let settings_key = key_of(&settings_v);
    let cache_key = cache_key(&rel.id, &prefix.input_hash, &settings_key);
    if let Some(s) = o.cache.get(&cache_key).await {
        return Ok(artifact_response(&s, true));
    }
    let job_id = format!("job-{}", &hex_part(&cache_key)[..20]);
    let rx = {
        let mut jobs = o.jobs.write().unwrap();
        match jobs.get(&job_id) {
            Some(rx) if matches!(*rx.borrow(), JobState::Running) => rx.clone(),
            _ => {
                let (tx, rx) = watch::channel(JobState::Running);
                jobs.insert(job_id.clone(), rx.clone());
                let ctx = AnalysisContext::new(&rel, &prefix, &settings_v, o.origin.label());
                let o2 = o.clone();
                tokio::spawn(async move {
                    let st = run_forecast(&o2, ei, ctx, prefix, settings_v).await;
                    let _ = tx.send(st);
                });
                rx
            }
        }
    };
    let wait = Duration::from_millis(r.wait_ms.unwrap_or(2000).min(10_000));
    let mut rx2 = rx.clone();
    let _ = tokio::time::timeout(wait, rx2.wait_for(|s| !matches!(s, JobState::Running))).await;
    let state = rx.borrow().clone();
    job_outcome(&o, &job_id, state).await
}

/// Run a blocking computation on the bounded Rayon pool without blocking a Tokio worker.
async fn on_pool<T: Send + 'static>(
    o: &Arc<Origin>,
    f: impl FnOnce(&Origin) -> T + Send + 'static,
) -> ApiResult<T> {
    let o = o.clone();
    tokio::task::spawn_blocking(move || {
        let pool = o.pool.clone();
        pool.install(|| std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&o))))
    })
    .await
    .map_err(|e| Refusal::failed("join", e.to_string()))?
    .map_err(|p| {
        let msg = p
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()));
        Refusal::failed(
            "expert_panic",
            format!("computation panicked: {}", msg.unwrap_or_default()),
        )
    })
}

/// Load (and verify) the release, run the expert, validate the artifact. Blocking.
fn forecast_blocking(
    o: &Origin,
    ei: usize,
    rel_id: &str,
    prefix: &PrefixBundle,
    settings: &Value,
) -> ApiResult<AnalysisArtifact> {
    let loaded: Arc<LoadedRelease> = o.loaded_release(rel_id)?;
    analysis::run_expert(o.experts[ei].as_ref(), &loaded, prefix, settings).map_err(Refusal::from)
}

async fn run_forecast(
    o: &Arc<Origin>,
    ei: usize,
    mut ctx: AnalysisContext,
    prefix: PrefixBundle,
    settings: Value,
) -> JobState {
    let prefix = Arc::new(prefix);
    let (p2, rel_id) = (prefix.clone(), ctx.model_release_id.clone());
    let artifact = match on_pool(o, move |o| {
        forecast_blocking(o, ei, &rel_id, &p2, &settings)
    })
    .await
    {
        Ok(Ok(a)) => a,
        Ok(Err(refusal)) | Err(refusal) => return JobState::Refused { refusal },
    };
    ctx.analysis_id = artifact.id.clone();
    ctx.computed_at_unix_s = now_s();
    let stored = Stored {
        artifact,
        context: ctx,
    };
    if let Err(refusal) = o.save(&stored, &serde_json::to_value(&*prefix).unwrap()) {
        return JobState::Refused { refusal };
    }
    let id = stored.artifact.id.clone();
    o.cache
        .insert(stored.context.cache_key.clone(), Arc::new(stored))
        .await;
    JobState::Completed { analysis_id: id }
}

async fn job_outcome(o: &Arc<Origin>, job_id: &str, state: JobState) -> ApiResult<Response> {
    match state {
        JobState::Running => Ok((
            StatusCode::ACCEPTED,
            Json(json!({"outcome": "job", "job_id": job_id, "state": "running"})),
        )
            .into_response()),
        JobState::Completed { analysis_id } => {
            Ok(artifact_response(&o.load_stored(&analysis_id)?, false))
        }
        JobState::Refused { refusal } => Err(refusal),
    }
}

async fn get_job(State(o): S, Path(id): Path<String>) -> ApiResult<Response> {
    let state = o
        .jobs
        .read()
        .unwrap()
        .get(&id)
        .map(|rx| rx.borrow().clone())
        .ok_or_else(|| Refusal::not_found("job_not_found", format!("job {id}")))?;
    job_outcome(&o, &id, state).await
}

async fn get_analysis(State(o): S, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let s = o.load_stored(&id)?;
    let seal = o.load_seal(&id)?;
    let revealed = o
        .store_root
        .join("reveals")
        .join(format!("{}.json", hex_part(&id)))
        .exists();
    Ok(Json(
        json!({"artifact": s.artifact, "context": s.context, "sealed": seal, "revealed": revealed,
                   "served_from": "store (precomputed earlier; see context.computed_at_unix_s)"}),
    ))
}

// ---------------------------------------------------------------- acquisition

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AcqReq {
    analysis_id: String,
    #[serde(default)]
    cost_per_observation: Option<f64>,
}

async fn acquisition(State(o): S, Strict(r): Strict<AcqReq>) -> ApiResult<Json<Value>> {
    let s = o.load_stored(&r.analysis_id)?;
    let cost = r.cost_per_observation.unwrap_or(1.0);
    if !(cost.is_finite() && cost >= 0.0) {
        return Err(Refusal::invalid(
            "cost",
            "cost_per_observation must be finite and >= 0",
        ));
    }
    let reg = o.registry();
    let cutoff = s.context.cutoff_s;
    // Pool: recorded at or before the cutoff but not yet admitted. Values never leave the server here.
    let candidates: Vec<AcquisitionAction> = reg
        .observations(&s.context.run_id)
        .iter()
        .filter(|x| x.time_s <= cutoff && x.available_at_s > cutoff)
        .map(|x| AcquisitionAction {
            id: format!("acq:{}", x.id),
            run_id: x.run_id.clone(),
            observation_id: x.id.clone(),
            time_s: x.time_s,
            cost,
            quality: x.quality_flags.first().cloned(),
        })
        .collect();
    let scoring = if candidates.is_empty() {
        json!({"status": "no_candidates", "reason": "every observation recorded before the cutoff is already admitted"})
    } else {
        match score_acquisition(&o, &r.analysis_id, &s, candidates.clone()).await {
            Ok(t) => json!({"status": "scored", "trace": t}),
            Err(refusal) => json!({"status": "refused", "refusal": refusal}),
        }
    };
    Ok(Json(
        json!({"analysis_id": r.analysis_id, "run_id": s.context.run_id, "cutoff_s": cutoff,
                   "candidates": candidates, "cost_note": if r.cost_per_observation.is_none() {
                       "uniform placeholder cost 1.0 per observation (no cost model supplied)" } else { "cost supplied by request" },
                   "scoring": scoring}),
    ))
}

/// Score candidates with the injected decision function (see [`crate::AcquisitionScorer`]).
async fn score_acquisition(
    o: &Arc<Origin>,
    analysis_id: &str,
    s: &Stored,
    candidates: Vec<AcquisitionAction>,
) -> ApiResult<DecisionTrace> {
    let Some(scorer) = o.acquisition.clone() else {
        return Err(Refusal::unsupported(
            "decision_unavailable",
            "no acquisition scoring function is injected into this server",
        ));
    };
    let prefix: PrefixBundle =
        serde_json::from_value(o.load_prefix(analysis_id)?).map_err(|e| {
            Refusal::failed(
                "store_contract",
                format!("stored prefix violates the contract: {e}"),
            )
        })?;
    assert_isolated(&prefix, &[])
        .map_err(|e| Refusal::failed("prefix_integrity", e.to_string()))?;
    let rel_id = s.context.model_release_id.clone();
    let settings = s.context.settings.clone();
    let trace = on_pool(o, move |o| -> ApiResult<DecisionTrace> {
        let rel = o.loaded_release(&rel_id)?;
        scorer(&rel, &prefix, &candidates, &settings).map_err(|e| {
            match ignis_core::expert::Refusal::from_error(&e) {
                Some(r) => Refusal::from(r.clone()),
                None => Refusal::failed("decision_error", format!("{e:#}")),
            }
        })
    })
    .await??;
    if trace.prefix_hash != s.artifact.prefix_hash
        || trace.model_release_id != s.context.model_release_id
    {
        return Err(Refusal::failed(
            "trace_mismatch",
            "decision trace does not refer to this analysis' prefix and release",
        ));
    }
    Ok(trace)
}

// ---------------------------------------------------------------- seal / reveal

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealReq {
    analysis_id: String,
    #[serde(default)]
    split_partition: Option<SplitPartition>,
}

/// Identity of the whole stored artifact (every field, `created_at` included): what a seal commits to.
pub fn artifact_hash(a: &AnalysisArtifact) -> String {
    content_hash(a, &[])
}

async fn seal(State(o): S, Strict(r): Strict<SealReq>) -> ApiResult<Json<Value>> {
    let s = o.load_stored(&r.analysis_id)?;
    if let Some(existing) = o.load_seal(&r.analysis_id)? {
        return Ok(Json(json!({"sealed": existing, "already_sealed": true})));
    }
    let reg = o.registry();
    let partition =
        match reg.splits.iter().find(|sp| sp.id == s.context.split_id) {
            Some(sp) => match sp.partition_of(&s.context.run_id) {
                Some(Partition::Train) => {
                    return Err(Refusal::unsupported(
                        "train_partition",
                        "run is in the release's training partition; a sealed \
                                                held-out reveal is not meaningful",
                    ))
                }
                Some(Partition::Validation) => SplitPartition::Validation,
                Some(Partition::Test) => SplitPartition::Test,
                None => {
                    return Err(Refusal::unsupported(
                        "not_in_split",
                        "run is not assigned in the release's split",
                    ))
                }
            },
            // The partition is never taken from the client: without the frozen split there is no
            // trustworthy partition to commit to.
            None => return Err(Refusal::precondition(
                "split_unknown",
                "the release's split is not in the registry; seal where the frozen split is loaded",
            )),
        };
    if let (Some(req), true) = (
        r.split_partition,
        reg.splits.iter().any(|sp| sp.id == s.context.split_id),
    ) {
        if req != partition {
            return Err(Refusal::invalid(
                "partition_mismatch",
                "split_partition contradicts the frozen split",
            ));
        }
    }
    let hash = artifact_hash(&s.artifact);
    let sealed = SealedPrediction {
        artifact_id: s.artifact.id.clone(),
        artifact_hash: hash,
        sealed_at: rfc3339_utc(now_s()),
        split_partition: partition,
        run_id: s
            .artifact
            .run_id
            .clone()
            .or_else(|| Some(s.context.run_id.clone())),
        split_id: Some(s.context.split_id.clone()),
        prefix_hash: Some(s.artifact.prefix_hash.clone()),
    };
    let path = o.seal_path(&r.analysis_id)?;
    crate::state::write_json(&path, &serde_json::to_value(&sealed).unwrap())
        .map_err(|e| Refusal::failed("store_io", e.to_string()))?;
    Ok(Json(json!({"sealed": sealed, "already_sealed": false})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevealReq {
    analysis_id: String,
}

/// Every attempt, granted or refused, is appended to the reveal log before anything is returned; a
/// granted reveal whose log entry cannot be written is refused.
async fn reveal(State(o): S, Strict(r): Strict<RevealReq>) -> ApiResult<Json<Value>> {
    let result = reveal_inner(&o, &r);
    let mut entry = json!({
        "at": rfc3339_utc(now_s()), "user": o.public_scope.principal, "host": "ignis-server",
        "artifact_id": r.analysis_id, "final_test": false, "purpose": "server_reveal",
        "data_origin": o.origin, "granted": result.is_ok(),
    });
    match &result {
        Ok(v) => {
            entry["run_id"] = v["sealed"]["run_id"].clone();
            entry["split_id"] = v["sealed"]["split_id"].clone();
            entry["partition"] = v["sealed"]["split_partition"].clone();
        }
        Err(e) => entry["reason"] = json!(format!("{}: {}", e.code, e.reason)),
    }
    if let Err(e) = append_jsonl(&o.reveal_log, &entry) {
        return Err(Refusal::failed(
            "reveal_log",
            format!(
                "reveal log {} unwritable ({e}); nothing revealed",
                o.reveal_log.display()
            ),
        ));
    }
    result.map(Json)
}

fn append_jsonl(path: &std::path::Path, v: &Value) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{v}")?;
    f.sync_data()
}

fn reveal_inner(o: &Origin, r: &RevealReq) -> ApiResult<Value> {
    let s = o.load_stored(&r.analysis_id)?;
    let sealed = o.load_seal(&r.analysis_id)?.ok_or_else(|| {
        Refusal::precondition(
            "not_sealed",
            "reveal requires a SealedPrediction for this artifact first (POST /seal)",
        )
    })?;
    let hash = artifact_hash(&s.artifact);
    if hash != sealed.artifact_hash || sealed.artifact_id != s.artifact.id {
        return Err(Refusal::failed(
            "seal_mismatch",
            "stored artifact no longer matches its seal",
        ));
    }
    if o.origin == DataOrigin::Registry && sealed.split_partition == SplitPartition::Test {
        return Err(Refusal::precondition(
            "test_partition",
            "the test partition is opened once, by the team, with `ignis eval reveal --final-test`",
        ));
    }
    let reg = o.registry();
    let cutoff = s.context.cutoff_s;
    let continuation: Vec<&Observation> = reg
        .observations(&s.context.run_id)
        .iter()
        .filter(|x| !x.admitted_at(cutoff))
        .collect();
    let contours: Vec<&ContourObservation> = reg
        .contours(&s.context.run_id)
        .iter()
        .filter(|c| !(c.available_at_s <= cutoff && c.time_s <= cutoff))
        .collect();
    let endpoint = reg.endpoints.get(&s.context.run_id);
    let log = json!({"analysis_id": r.analysis_id, "revealed_at_unix_s": now_s(), "sealed": sealed,
                     "n_continuation": continuation.len()});
    crate::state::write_json(
        &o.store_root
            .join("reveals")
            .join(format!("{}.json", hex_part(&r.analysis_id))),
        &log,
    )
    .map_err(|e| Refusal::failed("store_io", e.to_string()))?;
    Ok(
        json!({"analysis_id": r.analysis_id, "sealed": sealed, "cutoff_s": cutoff, "continuation": continuation,
                   "contours": contours, "endpoint": endpoint, "evidence_class_note": "revealed continuation: recorded \
                   observations that were not admitted at the cutoff"}),
    )
}

// ---------------------------------------------------------------- brief

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BriefReq {
    question: String,
    #[serde(default)]
    run_ids: Vec<String>,
    #[serde(default)]
    analysis_ids: Vec<String>,
    #[serde(default)]
    limit: Option<usize>,
}

async fn brief(State(o): S, Strict(r): Strict<BriefReq>) -> ApiResult<Json<Value>> {
    if r.run_ids.len() > 20 || r.analysis_ids.len() > 20 || r.question.len() > 2000 {
        return Err(Refusal::invalid(
            "too_large",
            "at most 20 runs, 20 analyses and 2000 bytes of question",
        ));
    }
    let limit = r.limit.unwrap_or(10).clamp(1, 50);
    let (evidence, evidence_status) = match run_search(&o, r.question.clone(), limit, None).await {
        Ok(h) => {
            let (h, stripped) = strip_sealed_hits(h, &sealed_runs(&o));
            (
                h,
                json!({"status": "ok", "ranking": "fts5 bm25 (lexical)", "stripped_for_sealed_runs": stripped}),
            )
        }
        Err(e) => (Vec::new(), json!({"status": "unavailable", "refusal": e})),
    };
    let reg = o.registry();
    let fams: BTreeSet<Family> = o.list_releases().0.iter().map(|r| r.family).collect();
    let mut gaps = Vec::new();
    let experiments: Vec<Value> = r
        .run_ids
        .iter()
        .map(|id| match reg.experiments.get(id) {
            Some(x) => experiment_summary(&o, x, &fams),
            None => {
                gaps.push(format!("run {id} is not in the registry"));
                Value::Null
            }
        })
        .collect();
    let analyses: Vec<Value> = r
        .analysis_ids
        .iter()
        .map(|id| match o.load_stored(id) {
            Ok(s) => json!({"analysis_id": s.artifact.id, "kind": s.artifact.kind, "status": s.artifact.status,
                            "support": s.artifact.support, "model_release_id": s.artifact.model_release_id,
                            "model_name": s.context.model_name, "run_id": s.context.run_id, "cutoff_s": s.context.cutoff_s,
                            "prefix_hash": s.artifact.prefix_hash, "evidence_class": s.artifact.evidence_class}),
            Err(_) => {
                gaps.push(format!("analysis {id} not found"));
                Value::Null
            }
        })
        .collect();
    if evidence.is_empty() {
        gaps.push("no indexed passage matched the question".into());
    }
    if !evidence
        .iter()
        .any(|h| h.outcome_bearing == Value::from(1) || h.outcome_bearing == Value::Bool(true))
        && !evidence.is_empty()
    {
        gaps.push("no matched passage is outcome-bearing".into());
    }
    Ok(Json(
        json!({"question": r.question, "generated_by": "structured assembly, no language model",
                   "evidence": evidence, "evidence_status": evidence_status, "experiments": experiments,
                   "analyses": analyses, "gaps": gaps,
                   "highlight": {"open": crate::evidence::HIT_OPEN, "close": crate::evidence::HIT_CLOSE}}),
    ))
}

#[cfg(test)]
mod tests {
    fn hit(outcome: bool, runs: serde_json::Value) -> crate::evidence::Hit {
        let n = serde_json::Value::Null;
        crate::evidence::Hit {
            chunk_id: serde_json::json!(format!("{runs}")),
            source_id: n.clone(),
            title: n.clone(),
            canonical_url: n.clone(),
            page: n.clone(),
            char_start: n.clone(),
            char_end: n.clone(),
            section: n.clone(),
            locator: n.clone(),
            snippet: String::new(),
            outcome_bearing: serde_json::json!(if outcome { 1 } else { 0 }),
            describes_runs: runs,
            evidence_class: n,
            bm25: 0.0,
        }
    }

    #[test]
    fn sealed_hits_are_stripped_conservatively() {
        use serde_json::json;
        let sealed = std::collections::BTreeSet::from(["run-test".to_string()]);
        let hits = vec![
            hit(false, json!(["run-test"])),    // not outcome-bearing: kept
            hit(true, json!(["run-train"])),    // outcome of an open run: kept
            hit(true, json!(["run-test"])),     // outcome of a sealed run: stripped
            hit(true, json!("[\"run-test\"]")), // same, stored as JSON text
            hit(true, json!(null)),             // unattributed outcome: stripped
        ];
        let (kept, stripped) = super::strip_sealed_hits(hits.clone(), &sealed);
        assert_eq!((kept.len(), stripped), (2, 3));
        let (all, none) = super::strip_sealed_hits(hits, &Default::default());
        assert_eq!((all.len(), none), (5, 0));
    }

    #[test]
    fn rfc3339() {
        assert_eq!(super::rfc3339_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(super::rfc3339_utc(1_794_669_045), "2026-11-14T15:10:45Z");
    }
}

// ---------------------------------------------------------------- frozen kernel endpoint

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScoreReq {
    series: Vec<ResidualSeries>,
}

async fn score_student_t(State(o): S, Strict(r): Strict<ScoreReq>) -> ApiResult<Json<Value>> {
    if r.series.len() > 10_000 || r.series.iter().any(|s| s.times_s.len() > 100_000) {
        return Err(Refusal::invalid(
            "too_large",
            "at most 10000 series of 100000 points",
        ));
    }
    let pool = o.pool.clone();
    let scores = tokio::task::spawn_blocking(move || score_batch(&pool, &r.series))
        .await
        .map_err(|e| Refusal::failed("join", e.to_string()))?;
    Ok(Json(
        json!({"nu": ignis_core::kernels::NU, "kernel": "exponential + nugget, OU/Kalman O(n)", "scores": scores}),
    ))
}

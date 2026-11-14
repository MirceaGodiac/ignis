//! `/api/v1` routes for runs, datasets and conditions (task M11): thin projections of the
//! `DatasetService`, so the UI can retire `/api/experiments*`.
//!
//! | route | answer |
//! |---|---|
//! | `GET /datasets` | [`DatasetPage`]: the bundles the caller's scope may read (metadata only) |
//! | `GET /datasets/{id}` | the `DatasetBundle`; its numeric tables are views behind Parquet `ArtifactRef`s |
//! | `GET /datasets/{id}/views/{view}` | the view's Parquet bytes, streamed (`offset`/`len`), never loaded whole |
//! | `GET /runs` | a `ConditionsTable` page: one row per run (or investigation-level test point) |
//! | `GET /runs/{id}` | a `ConditionsTable` holding that run's row; `404` when absent or withheld |
//! | `POST /conditions` | a `ConditionsTable` for a `ConditionsQuery` (keys, filters, envelopes, paging) |
//!
//! The scope is the host-derived [`ExecutionScope`] of the caller ([`Caller`]); nothing in a path,
//! query or body widens it. Partition isolation is the service's rule: a bundle holding a partition
//! the scope does not grant is not listed and is refused when named, the evaluator root is never
//! opened outside an evaluation scope, and conditions of a run in a non-granted partition are
//! withheld. Conditions keep their statuses (`value`, `conflicting`, `missing`, `not_applicable`)
//! with raw value, raw unit, SI value and locator exactly as the service returns them.

use std::io::Read;
use std::sync::Arc;

use aide::axum::routing::{get_with, post_with};
use aide::axum::ApiRouter;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use ignis_core::contracts::{
    Capability, ConditionsQuery, ConditionsTable, DatasetBundle, DatasetId, DatasetSchema,
    ErrorCode, ErrorEnvelope, ExecutionScope, PageCursor, RunId,
};
use ignis_core::services::DatasetService;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::io::{ApiError, ApiResult, Body, Caller};
use super::{blocking, St, V1};

/// Bytes per chunk of a streamed view.
const CHUNK: usize = 256 * 1024;
/// Bundles per page when the client does not say.
const DEFAULT_PAGE: usize = 50;
const DATASET_CURSOR: &str = "datasets:";

/// Short commands (metadata and conditions pages); they share the 30 s command timeout.
pub(super) fn routes() -> ApiRouter<Arc<V1>> {
    ApiRouter::new()
        .api_route(
            "/datasets",
            get_with(list_datasets, |o| {
                o.summary("Dataset bundles the caller's scope may read (paginated)")
            }),
        )
        .api_route(
            "/datasets/{id}",
            get_with(get_dataset, |o| {
                o.summary("A dataset bundle (views are Parquet ArtifactRefs)")
            }),
        )
        .api_route(
            "/runs",
            get_with(list_runs, |o| {
                o.summary("Runs with their conditions (ConditionsTable page)")
            }),
        )
        .api_route(
            "/runs/{id}",
            get_with(get_run, |o| o.summary("One run's conditions")),
        )
        .api_route(
            "/conditions",
            post_with(query_conditions, |o| {
                o.summary("Conditions matrix for a ConditionsQuery (gaps as explicit statuses)")
            }),
        )
}

/// View bytes: streamed, so no request timeout (a large table is not a slow command).
pub(super) fn streams() -> ApiRouter<Arc<V1>> {
    ApiRouter::new().api_route(
        "/datasets/{id}/views/{view}",
        get_with(read_view, |o| {
            o.summary("A dataset view's Parquet bytes, streamed (offset/len)")
        }),
    )
}

// ------------------------------------------------------------------------------ types

/// A page of dataset bundles, ordered by `dataset_id`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DatasetPage {
    pub items: Vec<DatasetBundle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<PageCursor>,
    /// Bundles the scope may read, over all pages.
    pub total_count: u64,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PageQuery {
    /// Bundles per page (positive; default 50).
    page_size: Option<u32>,
    /// `next_cursor` of the previous page.
    cursor: Option<PageCursor>,
}

#[derive(Deserialize, JsonSchema)]
struct DatasetPath {
    id: DatasetId,
}

#[derive(Deserialize, JsonSchema)]
struct ViewPath {
    id: DatasetId,
    view: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ViewQuery {
    offset: Option<u64>,
    len: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RunsQuery {
    /// Only this investigation's runs.
    investigation_id: Option<String>,
    /// Rows per page (positive; the service's default otherwise).
    page_size: Option<u32>,
    /// `next_cursor` of the previous page.
    cursor: Option<PageCursor>,
    /// Add each investigation's tested envelope (over the runs the scope may see).
    #[serde(default)]
    include_envelopes: bool,
}

#[derive(Deserialize, JsonSchema)]
struct RunPath {
    id: RunId,
}

// ------------------------------------------------------------------------------ handlers

fn datasets(v: &V1) -> ApiResult<Arc<dyn DatasetService>> {
    v.services
        .datasets
        .clone()
        .ok_or_else(|| ApiError::unavailable("DatasetService"))
}

fn read_data(scope: &ExecutionScope) -> ApiResult<()> {
    scope
        .require(Capability::ReadData)
        .map_err(|e| ApiError::from(ErrorEnvelope::from(e)))
}

fn offset_of(c: Option<&PageCursor>) -> ApiResult<usize> {
    match c {
        None => Ok(0),
        Some(c) => c
            .as_str()
            .strip_prefix(DATASET_CURSOR)
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::InvalidRequest,
                    format!("{c} is not a datasets cursor"),
                )
            }),
    }
}

async fn list_datasets(
    State(v): St,
    c: Caller,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<DatasetPage>> {
    let d = datasets(&v)?;
    read_data(&c.scope)?;
    if q.page_size == Some(0) {
        return Err(ApiError::new(
            ErrorCode::InvalidRequest,
            "page_size must be positive",
        ));
    }
    let offset = offset_of(q.cursor.as_ref())?;
    let size = q.page_size.map_or(DEFAULT_PAGE, |n| n as usize);
    let all = blocking(move || d.datasets(&c.scope)).await?;
    let total = all.len();
    let items: Vec<DatasetBundle> = all.into_iter().skip(offset).take(size).collect();
    let next = offset + items.len();
    Ok(Json(DatasetPage {
        next_cursor: (next < total && !items.is_empty())
            .then(|| PageCursor::new(format!("{DATASET_CURSOR}{next}")).expect("generated cursor")),
        total_count: total as u64,
        items,
    }))
}

async fn get_dataset(
    State(v): St,
    c: Caller,
    Path(p): Path<DatasetPath>,
) -> ApiResult<Json<DatasetBundle>> {
    let d = datasets(&v)?;
    read_data(&c.scope)?;
    Ok(Json(blocking(move || d.dataset(&c.scope, &p.id)).await?))
}

async fn read_view(
    State(v): St,
    c: Caller,
    Path(p): Path<ViewPath>,
    Query(q): Query<ViewQuery>,
) -> ApiResult<Response> {
    let d = datasets(&v)?;
    read_data(&c.scope)?;
    let offset = q.offset.unwrap_or(0);
    let (meta, reader) =
        blocking(move || d.open_view(&c.scope, &p.id, &p.view, offset, q.len)).await?;
    let remaining = meta.byte_length.saturating_sub(offset);
    let length = q.len.map_or(remaining, |l| l.min(remaining));
    // Each chunk is read on the blocking pool; the reader moves in and out of the task.
    let stream = futures_util::stream::unfold(Some(reader), |r| async move {
        let mut r = r?;
        let step = tokio::task::spawn_blocking(move || {
            let mut buf = vec![0u8; CHUNK];
            match r.read(&mut buf) {
                Ok(0) => None,
                Ok(n) => {
                    buf.truncate(n);
                    Some((Ok(axum::body::Bytes::from(buf)), r))
                }
                Err(e) => Some((Err(e), r)),
            }
        })
        .await;
        match step {
            Ok(Some((Ok(b), r))) => Some((Ok::<_, std::io::Error>(b), Some(r))),
            // A read error ends the body early; the client sees a short read against the length.
            Ok(Some((Err(e), _))) => Some((Err(e), None)),
            Ok(None) | Err(_) => None,
        }
    });
    let mut resp = axum::body::Body::from_stream(stream).into_response();
    *resp.status_mut() = StatusCode::OK;
    let h = resp.headers_mut();
    if let Ok(v) = meta.media_type.parse() {
        h.insert(header::CONTENT_TYPE, v);
    }
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(length));
    if let Ok(v) = meta.sha256.as_str().parse() {
        h.insert("x-ignis-artifact-sha256", v);
    }
    h.insert(
        "x-ignis-artifact-length",
        HeaderValue::from(meta.byte_length),
    );
    if let Ok(v) = meta.artifact_id.as_str().parse() {
        h.insert("x-ignis-artifact-id", v);
    }
    Ok(resp)
}

fn conditions_query(investigation: Option<String>, runs: Vec<RunId>) -> ConditionsQuery {
    ConditionsQuery {
        schema_version: DatasetSchema::V2,
        investigation_ids: investigation.into_iter().collect(),
        run_ids: runs,
        keys: vec![],
        filters: vec![],
        include_unknown: false,
        include_envelopes: false,
        page_size: None,
        cursor: None,
    }
}

async fn conditions(
    v: &V1,
    scope: ExecutionScope,
    q: ConditionsQuery,
) -> ApiResult<ConditionsTable> {
    let d = datasets(v)?;
    read_data(&scope)?;
    q.validate()
        .map_err(|e| ApiError::from(ErrorEnvelope::from(e)))?;
    blocking(move || d.conditions(&scope, &q)).await
}

async fn list_runs(
    State(v): St,
    c: Caller,
    Query(q): Query<RunsQuery>,
) -> ApiResult<Json<ConditionsTable>> {
    let mut cq = conditions_query(q.investigation_id, vec![]);
    cq.page_size = q.page_size;
    cq.cursor = q.cursor;
    cq.include_envelopes = q.include_envelopes;
    Ok(Json(conditions(&v, c.scope, cq).await?))
}

async fn get_run(
    State(v): St,
    c: Caller,
    Path(p): Path<RunPath>,
) -> ApiResult<Json<ConditionsTable>> {
    let id = p.id.clone();
    let t = conditions(&v, c.scope, conditions_query(None, vec![p.id])).await?;
    // Absent and withheld look the same: a run in a partition the scope lacks is not revealed.
    if t.rows.is_empty() {
        return Err(ApiError::new(
            ErrorCode::NotFound,
            format!("run {id} has no conditions this scope can read"),
        ));
    }
    Ok(Json(t))
}

async fn query_conditions(
    State(v): St,
    c: Caller,
    Body(q): Body<ConditionsQuery>,
) -> ApiResult<Json<ConditionsTable>> {
    Ok(Json(conditions(&v, c.scope, q).await?))
}

//! `/api/v1` observability (task M5): metrics derived from the event log by
//! [`ignis_research::metrics`], priced with the process's versioned price list.
//!
//! | route | answer |
//! |---|---|
//! | `GET /episodes/{id}/metrics` | [`EpisodeMetrics`]: tool latency p50/p95, tokens and cache hit per turn and question, cost |
//! | `GET /metrics` | [`WorkspaceMetrics`]: the same over every episode of the caller's workspace |
//!
//! Nothing here is stored or estimated: latencies come from the host's `tool.started` /
//! `tool.finished` stamps, tokens from `usage.reported`, money from the price list (a model it does
//! not list is `unpriced`, never zero). The scope is the caller's host-derived one.

use std::sync::Arc;

use aide::axum::routing::get_with;
use aide::axum::ApiRouter;
use axum::extract::{Path, State};
use axum::Json;
use ignis_core::contracts::{EpisodeMetrics, WorkspaceMetrics};

use super::io::{ApiResult, Caller};
use super::{blocking, EpisodePath, St, V1};

pub(super) fn routes() -> ApiRouter<Arc<V1>> {
    ApiRouter::new()
        .api_route(
            "/episodes/{id}/metrics",
            get_with(episode_metrics, |o| {
                o.summary("Measured tool latency, tokens, cache hit and cost of an episode")
            }),
        )
        .api_route(
            "/metrics",
            get_with(workspace_metrics, |o| {
                o.summary("Measured tool latency, tokens and cost over the caller's episodes")
            }),
        )
}

async fn episode_metrics(
    State(v): St,
    c: Caller,
    Path(p): Path<EpisodePath>,
) -> ApiResult<Json<EpisodeMetrics>> {
    v.seed(&c.scope).await?;
    let store = v.store.clone();
    let m = blocking(move || {
        let prices = ignis_research::metrics::prices();
        ignis_research::metrics::measure_episode(&store, &c.scope, &p.id, &prices)
    })
    .await?;
    Ok(Json(m.metrics))
}

async fn workspace_metrics(State(v): St, c: Caller) -> ApiResult<Json<WorkspaceMetrics>> {
    v.seed(&c.scope).await?;
    let store = v.store.clone();
    Ok(Json(
        blocking(move || {
            let prices = ignis_research::metrics::prices();
            ignis_research::metrics::measure_workspace(&store, &c.scope, &prices)
        })
        .await?,
    ))
}

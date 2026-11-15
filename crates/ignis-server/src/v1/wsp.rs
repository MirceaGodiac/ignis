//! Routes of the served hand-written models besides `/models` and `/analyses`: their cards
//! (what each forecasts, its inputs, zero-shot or data-informed, reported scores and limits) and
//! the admission of a typed scenario. Thin adapters over [`ignis_core::services::ModelService`].

use std::sync::Arc;

use aide::axum::routing::{get_with, post_with};
use aide::axum::ApiRouter;
use axum::extract::{Path, State};
use axum::Json;
use ignis_core::contracts::{
    Capability, ErrorCode, ErrorEnvelope, ModelCard, ReleaseId, RetryPolicy, ScenarioAdmitted,
    ScenarioRequest,
};
use schemars::JsonSchema;
use serde::Deserialize;

use super::io::{ApiError, ApiResult, Body, Caller};
use super::{blocking, models, V1};

#[derive(Deserialize, JsonSchema)]
struct ScenarioPath {
    id: ReleaseId,
}

async fn cards(State(v): State<Arc<V1>>, c: Caller) -> ApiResult<Json<Vec<ModelCard>>> {
    let m = models(&v)?;
    Ok(Json(blocking(move || m.cards(&c.scope)).await?))
}

/// Admits conditions a person typed (the forecast lab) as a synthetic prefix of one release;
/// forecast it through `/analyses` with `input = { kind: prefix, input_hash }`.
async fn admit(
    State(v): State<Arc<V1>>,
    c: Caller,
    Path(p): Path<ScenarioPath>,
    Body(req): Body<ScenarioRequest>,
) -> ApiResult<Json<ScenarioAdmitted>> {
    let m = models(&v)?;
    if req.release_id != p.id {
        return Err(ErrorEnvelope::new(
            ErrorCode::InvalidRequest,
            "release_id of the body differs from the path",
            RetryPolicy::Never,
        )
        .into());
    }
    c.scope
        .require(Capability::RunModels)
        .map_err(|e| ApiError::from(ErrorEnvelope::from(e)))?;
    Ok(Json(
        blocking(move || m.admit_scenario(&c.scope, &req)).await?,
    ))
}

/// The routes of this module, merged into the `/api/v1` command router.
pub fn routes() -> ApiRouter<Arc<V1>> {
    ApiRouter::new()
        .api_route(
            "/model-cards",
            get_with(cards, |o| {
                o.summary(
                    "Served hand-written releases: inputs, authorship, reported scores, limits",
                )
            }),
        )
        .api_route(
            "/models/{id}/scenarios",
            post_with(admit, |o| {
                o.summary("Admit conditions typed by a person as a synthetic prefix (input_hash)")
            }),
        )
}

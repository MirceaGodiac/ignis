//! Transport pieces of `/api/v1`: the error type (always an `ErrorEnvelope`), the strict JSON body,
//! the authenticated caller (from which the host derives the `ExecutionScope`), and the SSE output,
//! each with the `aide` impls that put them in the generated OpenAPI document.

use std::collections::BTreeSet;
use std::convert::Infallible;

use aide::generate::GenContext;
use aide::openapi::{MediaType, Operation, Response as ApiResponse, StatusCode as ApiStatus};
use aide::operation::{OperationInput, OperationOutput};
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::Stream;
use ignis_core::contracts::{
    Capability, ErrorCode, ErrorEnvelope, ExecutionScope, Partition, PrincipalId, RetryPolicy,
    ScopeId, ScopeRole, WorkspaceId,
};
use ignis_core::hashing::sha256_hex;
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

/// An `ErrorEnvelope` with its HTTP status.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub envelope: ErrorEnvelope,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        ErrorEnvelope::new(code, message, RetryPolicy::Never).into()
    }

    /// `410 Gone`: the event log no longer covers the client's cursor.
    pub fn gone(last_seq: u64) -> Self {
        ApiError {
            status: StatusCode::GONE,
            envelope: ErrorEnvelope::new(
                ErrorCode::Conflict,
                format!("the event log does not cover this cursor; reload the episode and resume after seq {last_seq}"),
                RetryPolicy::AfterUserAction,
            ),
        }
    }

    pub fn unavailable(what: &str) -> Self {
        ErrorEnvelope::new(
            ErrorCode::CapabilityUnavailable,
            format!("{what} is not configured in this deployment (see GET /api/v1/capabilities)"),
            RetryPolicy::Never,
        )
        .into()
    }
}

pub fn status_of(code: ErrorCode) -> StatusCode {
    match code {
        ErrorCode::InvalidRequest => StatusCode::UNPROCESSABLE_ENTITY,
        ErrorCode::ScopeDenied => StatusCode::FORBIDDEN,
        ErrorCode::NotFound | ErrorCode::SourceMissing => StatusCode::NOT_FOUND,
        ErrorCode::Unsupported => StatusCode::NOT_IMPLEMENTED,
        ErrorCode::CapabilityUnavailable | ErrorCode::UpstreamUnavailable => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        ErrorCode::Conflict => StatusCode::CONFLICT,
        ErrorCode::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        ErrorCode::Cancelled => StatusCode::CONFLICT,
        ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl From<ErrorEnvelope> for ApiError {
    fn from(envelope: ErrorEnvelope) -> Self {
        ApiError {
            status: status_of(envelope.code),
            envelope,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.envelope)).into_response()
    }
}

impl OperationOutput for ApiError {
    type Inner = ErrorEnvelope;

    fn operation_response(ctx: &mut GenContext, op: &mut Operation) -> Option<ApiResponse> {
        Json::<ErrorEnvelope>::operation_response(ctx, op)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        op: &mut Operation,
    ) -> Vec<(Option<ApiStatus>, ApiResponse)> {
        Self::operation_response(ctx, op)
            .map(|r| vec![(None, r)])
            .unwrap_or_default()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

/// Strict JSON body: unknown fields and type errors are `invalid_request` envelopes.
pub struct Body<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> FromRequest<S> for Body<T> {
    type Rejection = ApiError;
    async fn from_request(req: Request, state: &S) -> Result<Self, ApiError> {
        let bytes = axum::body::Bytes::from_request(req, state)
            .await
            .map_err(|e| ApiError::new(ErrorCode::InvalidRequest, e.to_string()))?;
        serde_json::from_slice(&bytes)
            .map(Body)
            .map_err(|e| ApiError::new(ErrorCode::InvalidRequest, e.to_string()))
    }
}

impl<T: JsonSchema> OperationInput for Body<T> {
    fn operation_input(ctx: &mut GenContext, op: &mut Operation) {
        Json::<T>::operation_input(ctx, op)
    }
}

// ------------------------------------------------------------------------------ caller and scope

/// How callers are identified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum AuthMode {
    /// One local user (loopback development). Every request is the principal `local`.
    Local,
    /// Behind Cloudflare Access and the Worker: both Cloudflare-signed assertions are verified on
    /// every request ([`super::access`]); the user's e-mail in the verified user assertion is the
    /// principal. No unsigned header is trusted.
    CloudflareAccess,
}

/// The authenticated caller and the scope the host derives for it. Never read from a body.
#[derive(Debug, Clone)]
pub struct Caller {
    pub scope: ExecutionScope,
}

/// The research scope of a principal: its own workspace, outcomes of the training partition only
/// (validation is opened through seal → reveal, test never in this runtime), and the research
/// capabilities. No code execution on the host: `execute_code` grants `ExecuteCode`, which only the
/// isolated runner serves, and the host passes it only when that runner passed preflight.
pub fn research_scope(principal_key: &str, execute_code: bool) -> ExecutionScope {
    let h = &sha256_hex(principal_key.as_bytes())[..16];
    ExecutionScope {
        scope_id: ScopeId::new(format!("scope_{h}_research")).expect("generated id"),
        workspace_id: WorkspaceId::new(format!("ws_{h}")).expect("generated id"),
        principal: PrincipalId::new(format!("user_{h}")).expect("generated id"),
        role: ScopeRole::Research,
        partitions: BTreeSet::from([Partition::Train]),
        capabilities: BTreeSet::from([
            Capability::ReadSources,
            Capability::ReadData,
            Capability::RunModels,
            Capability::WorkspaceWrite,
        ])
        .into_iter()
        .chain(execute_code.then_some(Capability::ExecuteCode))
        .collect(),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

pub trait HasAuth {
    fn auth(&self) -> AuthMode;
    /// The Access verifier (`--auth cloudflare-access`); without one every request is refused.
    fn access(&self) -> Option<&super::access::AccessVerifier>;
    /// Whether callers may run code (only in the isolated runner).
    fn execute_code(&self) -> bool {
        false
    }
}

impl From<super::access::AccessError> for ApiError {
    fn from(e: super::access::AccessError) -> Self {
        use super::access::AccessError;
        match e {
            AccessError::Denied(m) => ApiError {
                status: StatusCode::UNAUTHORIZED,
                envelope: ErrorEnvelope::new(
                    ErrorCode::ScopeDenied,
                    format!("not authenticated: {m}"),
                    RetryPolicy::AfterUserAction,
                ),
            },
            AccessError::Unavailable(m) => ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                envelope: ErrorEnvelope::new(
                    ErrorCode::UpstreamUnavailable,
                    format!("identity verification unavailable: {m}"),
                    RetryPolicy::AfterDelay { seconds: 30 },
                ),
            },
        }
    }
}

fn header<'a>(parts: &'a Parts, name: &str) -> Option<&'a str> {
    parts.headers.get(name).and_then(|v| v.to_str().ok())
}

impl<S: HasAuth + Send + Sync> FromRequestParts<S> for Caller {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        let key = match state.auth() {
            AuthMode::Local => "local".to_string(),
            AuthMode::CloudflareAccess => {
                let v = state
                    .access()
                    .ok_or(super::access::AccessError::Unavailable(
                        "access verification is not configured".into(),
                    ))?;
                v.identify(
                    header(parts, super::access::ORIGIN_ASSERTION),
                    header(parts, super::access::USER_ASSERTION),
                )
                .await?
            }
        };
        Ok(Caller {
            scope: research_scope(&key, state.execute_code()),
        })
    }
}

impl OperationInput for Caller {}

// ------------------------------------------------------------------------------ SSE

/// `text/event-stream` of `ResearchEvent`s (`id: <seq>`, `event: <body.kind>`, `data: <json>`), with
/// heartbeat comments.
pub struct EventStream<St>(pub St);

impl<St> IntoResponse for EventStream<St>
where
    St: Stream<Item = Result<Event, Infallible>> + Send + 'static,
{
    fn into_response(self) -> Response {
        let mut r = Sse::new(self.0)
            .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15)))
            .into_response();
        // Proxies (nginx-style buffering, Cloudflare) must not hold events back.
        r.headers_mut().insert(
            "x-accel-buffering",
            axum::http::HeaderValue::from_static("no"),
        );
        r.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-cache"),
        );
        r
    }
}

impl<St> OperationOutput for EventStream<St> {
    type Inner = ();

    fn operation_response(ctx: &mut GenContext, _op: &mut Operation) -> Option<ApiResponse> {
        let schema = ctx
            .schema
            .subschema_for::<ignis_core::contracts::ResearchEvent>();
        Some(ApiResponse {
            description: "Server-sent events: `id` = seq, `event` = body.kind, `data` = ResearchEvent JSON. Resume with `Last-Event-ID` or `?after=`.".into(),
            content: IndexMap::from_iter([(
                "text/event-stream".into(),
                MediaType {
                    schema: Some(aide::openapi::SchemaObject {
                        json_schema: schema,
                        example: None,
                        external_docs: None,
                    }),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        })
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        op: &mut Operation,
    ) -> Vec<(Option<ApiStatus>, ApiResponse)> {
        Self::operation_response(ctx, op)
            .map(|r| vec![(Some(ApiStatus::Code(200)), r)])
            .unwrap_or_default()
    }
}

/// `202 Accepted` with a JSON body.
pub struct Accepted<T>(pub T);

impl<T: serde::Serialize> IntoResponse for Accepted<T> {
    fn into_response(self) -> Response {
        (StatusCode::ACCEPTED, Json(self.0)).into_response()
    }
}

impl<T: JsonSchema> OperationOutput for Accepted<T> {
    type Inner = T;

    fn operation_response(ctx: &mut GenContext, op: &mut Operation) -> Option<ApiResponse> {
        Json::<T>::operation_response(ctx, op)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        op: &mut Operation,
    ) -> Vec<(Option<ApiStatus>, ApiResponse)> {
        Self::operation_response(ctx, op)
            .map(|r| vec![(Some(ApiStatus::Code(202)), r)])
            .unwrap_or_default()
    }
}

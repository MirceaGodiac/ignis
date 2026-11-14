//! Typed refusals: first-class API responses, never a bare 500.
//!
//! `unsupported` and `insufficient_inference` are scientific outcomes (HTTP 200, `outcome: refusal`):
//! the request was valid and the system declines to produce a number. The other kinds are request or
//! service problems with their own HTTP status, but the body has the same typed shape.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalKind {
    /// Outside the expert's domain/support, incompatible comparison, missing expert.
    Unsupported,
    /// The expert could not condition reliably (diagnostics failed, too few observations).
    #[allow(dead_code)] // part of the typed API; experts report it inside artifacts today
    InsufficientInference,
    /// The request violates the strict schema.
    InvalidRequest,
    /// The host-derived scope does not grant this data (e.g. outcomes of a sealed partition).
    ScopeDenied,
    NotFound,
    /// A precondition of the protocol is not met (e.g. reveal before a seal).
    PreconditionFailed,
    /// A dependency is absent (evidence catalog).
    Unavailable,
    /// Unexpected failure; message says where.
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct Refusal {
    pub kind: RefusalKind,
    pub code: String,
    pub reason: String,
}

impl Refusal {
    pub fn new(kind: RefusalKind, code: impl Into<String>, reason: impl Into<String>) -> Self {
        Refusal {
            kind,
            code: code.into(),
            reason: reason.into(),
        }
    }
    pub fn unsupported(code: &str, reason: impl Into<String>) -> Self {
        Self::new(RefusalKind::Unsupported, code, reason)
    }
    pub fn scope_denied(code: &str, reason: impl Into<String>) -> Self {
        Self::new(RefusalKind::ScopeDenied, code, reason)
    }
    pub fn invalid(code: &str, reason: impl Into<String>) -> Self {
        Self::new(RefusalKind::InvalidRequest, code, reason)
    }
    pub fn not_found(code: &str, reason: impl Into<String>) -> Self {
        Self::new(RefusalKind::NotFound, code, reason)
    }
    pub fn precondition(code: &str, reason: impl Into<String>) -> Self {
        Self::new(RefusalKind::PreconditionFailed, code, reason)
    }
    pub fn unavailable(code: &str, reason: impl Into<String>) -> Self {
        Self::new(RefusalKind::Unavailable, code, reason)
    }
    pub fn failed(code: &str, reason: impl Into<String>) -> Self {
        Self::new(RefusalKind::Failed, code, reason)
    }
    pub fn status(&self) -> StatusCode {
        match self.kind {
            RefusalKind::Unsupported | RefusalKind::InsufficientInference => StatusCode::OK,
            RefusalKind::InvalidRequest => StatusCode::UNPROCESSABLE_ENTITY,
            RefusalKind::ScopeDenied => StatusCode::FORBIDDEN,
            RefusalKind::NotFound => StatusCode::NOT_FOUND,
            RefusalKind::PreconditionFailed => StatusCode::CONFLICT,
            RefusalKind::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            RefusalKind::Failed => StatusCode::BAD_GATEWAY,
        }
    }
}

impl IntoResponse for Refusal {
    fn into_response(self) -> Response {
        (
            self.status(),
            Json(json!({"outcome": "refusal", "refusal": self})),
        )
            .into_response()
    }
}

impl From<ignis_core::expert::Refusal> for Refusal {
    fn from(r: ignis_core::expert::Refusal) -> Self {
        use ignis_core::expert::RefusalKind as K;
        let kind = match r.kind {
            K::Unsupported => RefusalKind::Unsupported,
            K::InsufficientInference => RefusalKind::InsufficientInference,
            K::InvalidRequest => RefusalKind::InvalidRequest,
            K::NotFound => RefusalKind::NotFound,
            K::Failed => RefusalKind::Failed,
        };
        Refusal {
            kind,
            code: r.code,
            reason: r.reason,
        }
    }
}

pub type ApiResult<T> = Result<T, Refusal>;

//! `ignis-research`: the durable state of the research runtime and its agent-runtime bridge
//! (Mircea's lane, tasks M2 and M1).
//!
//! - [`store::ResearchStore`]: SQLite in WAL mode with one writer connection. Episodes, their
//!   append-only event logs (gap-free `seq` allocated in the same transaction as the append),
//!   client idempotency keys, and durable jobs (transactional claim, lease, fencing token,
//!   heartbeat, `reconciling` for ambiguous effects).
//! - [`artifacts::LocalArtifactStore`]: content-addressed bytes with atomic publication
//!   (staging → hash verified → immutable object → manifest), implementing
//!   [`ignis_core::services::ArtifactStore`].
//! - [`sandbox::Sandbox`]: the isolated analysis runner behind `analysis_execute` (M3).
//! - [`runtime`]: the agent runtime (M1): [`runtime::AgentRuntime`] and the Codex App Server bridge
//!   (pinned binary, preflight, private stdio JSON-RPC, approvals, thread/turn/item → events).
//!
//! Everything here is synchronous: the HTTP host calls it on the blocking pool, never on reactor
//! threads. The [`ignis_core::contracts::ExecutionScope`] is always derived by the host and passed
//! separately; records of another workspace are reported as not found, so their existence does not
//! leak.

pub mod artifacts;
pub mod backup;
pub mod r2;
pub mod runtime;
pub mod s3;
pub mod sandbox;
pub mod store;

use std::time::{SystemTime, UNIX_EPOCH};

use ignis_core::contracts::{ErrorCode, ErrorEnvelope, RetryPolicy};

pub use artifacts::LocalArtifactStore;
pub use store::ResearchStore;

pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub(crate) fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
}

pub(crate) fn internal(context: &str, e: impl std::fmt::Display) -> ErrorEnvelope {
    ErrorEnvelope::new(
        ErrorCode::Internal,
        format!("{context}: {e}"),
        RetryPolicy::Never,
    )
}

pub(crate) fn not_found(what: impl std::fmt::Display) -> ErrorEnvelope {
    ErrorEnvelope::new(
        ErrorCode::NotFound,
        format!("{what} not found"),
        RetryPolicy::Never,
    )
}

pub(crate) fn conflict(msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(ErrorCode::Conflict, msg, RetryPolicy::Never)
}

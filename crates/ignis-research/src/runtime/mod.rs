//! The agent runtime of IGNIS (task M1, ADR 01): Codex App Server owns the conversation and the
//! model/tool loop; this module maps, stores and audits it. It is not a second agent loop, not a
//! compactor and not an LLM gateway.
//!
//! - [`AgentRuntime`]: what the host calls (start a turn for recorded messages, interrupt for
//!   pause/cancel, resume with reconciliation, resolve approvals, recover after a restart).
//! - [`CodexBridge`]: the implementation over a pinned `codex app-server` reached through a private
//!   stdio JSON-RPC connection ([`rpc`], [`framing`]), one process per workspace.
//! - [`preflight`]: version/hash pin, protocol schema generated from the binary, handshake probe;
//!   the result is the [`RuntimeBuildManifest`] (or a reason the host reports as unavailable).
//! - [`mapping`]: Codex thread/turn/item → `ResearchEvent` records; unknown → `runtime.opaque`.
//! - [`grants`]: per-process bearer tokens binding the IGNIS MCP endpoint (M3) to a workspace scope.
//!
//! Everything is synchronous (threads, not Tokio), like the rest of this crate; the host calls these
//! methods on its blocking pool. Each call takes the host-derived [`ExecutionScope`] separately.

mod bridge;
#[doc(hidden)]
pub mod fake;
pub mod framing;
pub mod grants;
pub mod mapping;
pub mod preflight;
pub mod rpc;

use ignis_core::contracts::{ApprovalDecision, ApprovalId, EpisodeId, ExecutionScope};
use ignis_core::services::ServiceResult;

pub use bridge::{CodexBridge, CommitHook};
pub use grants::{McpAttachment, McpGrants, MCP_TOKEN_ENV};
pub use mapping::StopReason;
pub use preflight::{
    preflight, CodexConfig, RuntimeBuildManifest, RuntimeProfile, PINNED_CODEX_VERSION,
};

/// The host's view of the agent runtime. Methods return quickly; turns run on runtime-owned threads
/// and their events reach the episode log through the state store.
pub trait AgentRuntime: Send + Sync {
    /// Identity of the runtime build that hosts episodes.
    fn manifest(&self) -> &RuntimeBuildManifest;

    /// Start a turn for the episode's user messages that no turn has received yet, unless a turn is
    /// already open (its successor then picks them up) or the episode is paused or ended.
    fn kick(&self, scope: &ExecutionScope, episode: &EpisodeId) -> ServiceResult<()>;

    /// Stop the open turn (after the host recorded pause or cancel): interrupt it at the runtime's
    /// next checkpoint and cancel its pending approvals.
    fn interrupt(
        &self,
        scope: &ExecutionScope,
        episode: &EpisodeId,
        why: StopReason,
    ) -> ServiceResult<()>;

    /// After the host recorded resume: reconcile an open turn with the runtime thread (never
    /// re-sending its messages), then start a turn for messages that arrived meanwhile. Returns
    /// at once; the outcome is recorded in the episode log.
    fn resume(&self, scope: &ExecutionScope, episode: &EpisodeId) -> ServiceResult<()>;

    /// Resolve a pending approval. Only `approved` and `denied` come from callers; approving needs
    /// the capability the escalation's effect requires in `scope`.
    fn resolve_approval(
        &self,
        scope: &ExecutionScope,
        episode: &EpisodeId,
        approval: &ApprovalId,
        decision: ApprovalDecision,
    ) -> ServiceResult<()>;

    /// Reconcile every turn left open by a previous process (call once at startup).
    fn recover(&self) -> ServiceResult<()>;
}

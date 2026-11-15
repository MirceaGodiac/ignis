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
//! - [`ClaudeBridge`]: the same contract over Claude Code in headless mode (ADR 17), one process per
//!   turn; its stream-json is mapped onto the same events.
//! - [`harness`]: the versioned instruction bundle (M6) both runtimes start with, pinned per episode.
//! - [`grants`]: per-process bearer tokens binding the IGNIS MCP endpoint (M3) to a workspace scope.
//!
//! Everything is synchronous (threads, not Tokio), like the rest of this crate; the host calls these
//! methods on its blocking pool. Each call takes the host-derived [`ExecutionScope`] separately.

mod bridge;
mod claude;
#[doc(hidden)]
pub mod fake;
pub mod framing;
pub mod grants;
pub mod harness;
pub mod mapping;
pub mod preflight;
mod recorder;
pub mod rpc;

use ignis_core::contracts::{ApprovalDecision, ApprovalId, EpisodeId, ExecutionScope};
use ignis_core::services::ServiceResult;

pub use bridge::{CodexBridge, CommitHook};
pub use claude::{claude_preflight, turn_snapshot, ClaudeBridge, ClaudeConfig, Translator};
pub use grants::{McpAttachment, McpGrant, McpGrants, MCP_EPISODE_HEADER, MCP_TOKEN_ENV};
pub use harness::{HarnessBundle, HarnessShelf, HarnessSource, Skill, ToolDescriptions};
pub use mapping::StopReason;
pub use preflight::{
    codex_base_instructions, preflight, CodexConfig, RuntimeBuildManifest, RuntimeProfile,
    CODEX_BASE_LABEL, PINNED_CODEX_VERSION, REASONING_EFFORTS,
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

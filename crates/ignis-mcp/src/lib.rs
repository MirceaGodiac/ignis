//! `ignis-mcp`: the IGNIS tools for the agent runtime (Mircea's lane, task M3).
//!
//! - [`tools`]: [`ToolHost`], the one implementation of every catalog tool over the lanes' services
//!   (decode the declared contract, check the scope's capabilities, call the service, shape a
//!   `ToolResult`). Transports and HTTP routes call it; nothing re-implements domain logic.
//! - [`http`]: the rmcp streamable-HTTP server, mounted by the host on a loopback-only listener,
//!   authenticated by per-process bearer grants ([`ignis_research::runtime::McpGrants`]) that map
//!   to the host-derived [`ignis_core::contracts::ExecutionScope`].
//! - [`schemas`]: tool input schemas generated from the contract types.
//!
//! The runtime side (registering the server with `codex app-server`, issuing and revoking the
//! grants) lives in the Codex bridge (`ignis_research::runtime`).

pub mod http;
pub mod schemas;
pub mod tools;

pub use http::{router, IgnisMcp};
pub use tools::{catalog, ToolAvailability, ToolHost, ToolServices, ANALYSIS_JOB, FORECAST_JOB};

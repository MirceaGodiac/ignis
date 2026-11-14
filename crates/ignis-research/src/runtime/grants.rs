//! Host-issued bearer tokens that bind the IGNIS MCP endpoint to a workspace scope (task M3).
//!
//! MCP itself carries no user context (master design §8.4). The bridge issues one token per
//! `codex app-server` process it spawns, for the [`ExecutionScope`] that process serves (one process
//! per workspace, and a workspace belongs to one principal), passes it to the child as an environment
//! variable named in the runtime's `mcp_servers.ignis.bearer_token_env_var`, and revokes it when the
//! process goes away. The MCP server resolves every request's bearer token here; the scope never
//! comes from a tool argument, a prompt or a header the model can set.
//!
//! Only `sha256(token)` is kept, so a dump of this registry does not reveal usable tokens.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use ignis_core::contracts::ExecutionScope;
use ignis_core::hashing::sha256_hex;

/// Environment variable through which the runtime child receives its token.
pub const MCP_TOKEN_ENV: &str = "IGNIS_MCP_TOKEN";

/// Name of the IGNIS server in the runtime's MCP configuration.
pub use super::mapping::IGNIS_MCP_SERVER;

/// Live grants: token digest → scope.
#[derive(Default)]
pub struct McpGrants {
    by_digest: Mutex<HashMap<String, ExecutionScope>>,
}

/// A bearer token for one runtime process. Not `Debug`/`Display`: it is a secret.
pub struct McpToken(String);

impl McpToken {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl McpGrants {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// A fresh token (244 random bits) granting exactly `scope`.
    pub fn issue(&self, scope: &ExecutionScope) -> McpToken {
        let token = format!(
            "ignis_mcp_{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        self.lock().insert(digest(&token), scope.clone());
        McpToken(token)
    }

    /// The scope a bearer token grants, if it is live.
    pub fn resolve(&self, token: &str) -> Option<ExecutionScope> {
        self.lock().get(&digest(token)).cloned()
    }

    pub fn revoke(&self, token: &McpToken) {
        self.lock().remove(&digest(&token.0));
    }

    /// A token for `scope` that is revoked when the lease is dropped.
    pub fn lease(self: &Arc<Self>, scope: &ExecutionScope) -> McpLease {
        McpLease {
            token: self.issue(scope),
            grants: self.clone(),
        }
    }

    /// Number of live grants (diagnostics and tests).
    pub fn live(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, ExecutionScope>> {
        self.by_digest.lock().unwrap_or_else(|p| p.into_inner())
    }
}

/// A token held by one runtime process; dropping it revokes the grant.
pub struct McpLease {
    token: McpToken,
    grants: Arc<McpGrants>,
}

impl McpLease {
    pub fn token(&self) -> &McpToken {
        &self.token
    }
}

impl Drop for McpLease {
    fn drop(&mut self) {
        self.grants.revoke(&self.token);
    }
}

fn digest(token: &str) -> String {
    sha256_hex(token.as_bytes())
}

/// Where the runtime reaches the IGNIS MCP server: a loopback streamable-HTTP URL served by the host,
/// the registry its tokens come from, and which tools the runtime may call.
#[derive(Clone)]
pub struct McpAttachment {
    pub url: String,
    pub grants: Arc<McpGrants>,
    /// The registered tools (the runtime exposes no other tool of this server).
    pub enabled_tools: Vec<String>,
    /// Tools the runtime calls without asking (read / scoped compute: the host checks scope and
    /// capabilities on every call). Every other tool asks, and the bridge declines what it cannot
    /// route to the host, so a new write tool fails closed until it gets a host approval path.
    pub approved_tools: Vec<String>,
}

impl fmt::Debug for McpAttachment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpAttachment")
            .field("url", &self.url)
            .field("enabled_tools", &self.enabled_tools)
            .field("approved_tools", &self.approved_tools)
            .field("live_grants", &self.grants.live())
            .finish()
    }
}

impl McpAttachment {
    /// `-c` overrides that register the server with `codex app-server` (values are TOML).
    pub fn config_overrides(&self) -> Vec<String> {
        let q = |s: &str| serde_json::to_string(s).expect("a string serializes");
        let key = |k: &str| format!("mcp_servers.{IGNIS_MCP_SERVER}.{k}");
        let mut o = vec![
            format!("{}={}", key("url"), q(&self.url)),
            format!("{}={}", key("bearer_token_env_var"), q(MCP_TOKEN_ENV)),
            format!(
                "{}={}",
                key("enabled_tools"),
                serde_json::to_string(&self.enabled_tools).expect("strings serialize")
            ),
            format!("{}={}", key("default_tools_approval_mode"), q("prompt")),
        ];
        o.extend(self.approved_tools.iter().map(|t| {
            format!(
                "{}={}",
                key(&format!("tools.{t}.approval_mode")),
                q("approve")
            )
        }));
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(ws: &str) -> ExecutionScope {
        let mut s: ExecutionScope = serde_json::from_str(include_str!(
            "../../../../fixtures/contracts/common/scope_research.json"
        ))
        .unwrap();
        s.workspace_id = ignis_core::contracts::WorkspaceId::new(ws).unwrap();
        s
    }

    #[test]
    fn tokens_resolve_to_their_own_scope_until_revoked() {
        let g = McpGrants::default();
        let (a, b) = (g.issue(&scope("ws_a")), g.issue(&scope("ws_b")));
        assert_ne!(a.expose(), b.expose());
        assert_eq!(g.resolve(a.expose()).unwrap().workspace_id.as_str(), "ws_a");
        assert_eq!(g.resolve(b.expose()).unwrap().workspace_id.as_str(), "ws_b");
        assert!(g.resolve("ignis_mcp_guess").is_none());
        g.revoke(&a);
        assert!(g.resolve(a.expose()).is_none());
        assert!(g.resolve(b.expose()).is_some());
    }

    #[test]
    fn the_runtime_config_allows_only_registered_tools_and_asks_for_the_rest() {
        let a = McpAttachment {
            url: "http://127.0.0.1:1/registry/mcp".into(),
            grants: McpGrants::new(),
            enabled_tools: vec!["sources_search".into(), "wiki_patch".into()],
            approved_tools: vec!["sources_search".into()],
        };
        assert_eq!(
            a.config_overrides(),
            [
                r#"mcp_servers.ignis.url="http://127.0.0.1:1/registry/mcp""#,
                r#"mcp_servers.ignis.bearer_token_env_var="IGNIS_MCP_TOKEN""#,
                r#"mcp_servers.ignis.enabled_tools=["sources_search","wiki_patch"]"#,
                r#"mcp_servers.ignis.default_tools_approval_mode="prompt""#,
                r#"mcp_servers.ignis.tools.sources_search.approval_mode="approve""#,
            ]
        );
        // No secret in the configuration: the token travels only in the child's environment.
        assert!(!format!("{a:?}").contains("ignis_mcp_"));
    }

    #[test]
    fn a_dropped_lease_revokes_its_token() {
        let g = McpGrants::new();
        let lease = g.lease(&scope("ws_a"));
        let token = lease.token().expose().to_string();
        assert!(g.resolve(&token).is_some());
        drop(lease);
        assert!(g.resolve(&token).is_none());
        assert_eq!(g.live(), 0);
    }
}

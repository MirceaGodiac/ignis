//! The MCP server (rmcp, streamable HTTP) the agent runtime reaches the IGNIS tools through.
//!
//! - Bound to loopback only by the host and never routed through the public API or the tunnel
//!   (master design §8.4: Codex/MCP/shell are not exposed to the internet). rmcp additionally
//!   rejects non-loopback `Host` headers (DNS rebinding).
//! - Every request carries a bearer token the host issued to one runtime process
//!   ([`McpGrants`]); the token resolves to that process's [`ExecutionScope`]. A request without a
//!   live token is rejected before MCP sees it. Tool arguments never carry a scope.
//! - `tools/list` serves the tools whose service is configured, with input schemas generated from
//!   the contract types (inlined, without `$ref`, so clients that flatten schemas keep the shape).
//! - `tools/call` runs [`ToolHost::call`] on the blocking pool and returns the [`ToolResult`] as
//!   structured content (and as its JSON text, for clients that only read text content).

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Router;
use ignis_core::contracts::{ExecutionScope, ExecutionStatus, ToolDescriptor, ToolEffect};
use ignis_research::runtime::McpGrants;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ServerCapabilities, Tool,
    ToolAnnotations,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData as McpError, ServerHandler};
use serde_json::{Map, Value};

use crate::schemas::input_schema;
use crate::tools::ToolHost;

const INSTRUCTIONS: &str = "IGNIS research tools over NASA microgravity-combustion sources, datasets and \
model releases. Every result is a ToolResult: execution_status says whether the call ran, \
scientific_status is never assessed by the tool itself. Long work (models_forecast) answers \
`accepted` with a job: follow it with jobs_get, then read its output artifacts with artifacts_read. \
A partial result points at the full artifact; read more of it instead of treating the preview as \
the whole answer.";

/// One MCP session's handler. Cheap to clone; every session shares the host.
#[derive(Clone)]
pub struct IgnisMcp {
    host: Arc<ToolHost>,
    tools: Arc<Vec<Tool>>,
}

fn mcp_tool(d: &ToolDescriptor) -> Tool {
    let schema = input_schema(&d.input_contract)
        .unwrap_or_else(|| panic!("no schema for {} (checked by tests)", d.input_contract));
    let Value::Object(schema) = schema else {
        panic!("{} schema is not an object", d.input_contract)
    };
    let read = d.effect == ToolEffect::Read;
    Tool::new(d.name.clone(), d.summary.clone(), Arc::new(schema)).with_annotations(
        ToolAnnotations::new()
            .read_only(read)
            .destructive(false)
            .idempotent(d.idempotent)
            .open_world(matches!(
                d.effect,
                ToolEffect::NetworkFetch | ToolEffect::ExternalWrite
            )),
    )
}

impl IgnisMcp {
    pub fn new(host: Arc<ToolHost>) -> Self {
        let tools = host.registered().iter().map(mcp_tool).collect();
        IgnisMcp {
            host,
            tools: Arc::new(tools),
        }
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }
}

impl ServerHandler for IgnisMcp {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("ignis", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(self.tools.to_vec()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.iter().find(|t| t.name == name).cloned()
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        // Put there by `require_grant` from the request's bearer token; never from the call.
        let scope = context
            .extensions
            .get::<axum::http::request::Parts>()
            .and_then(|p| p.extensions.get::<ExecutionScope>())
            .cloned()
            .ok_or_else(|| McpError::invalid_request("no granted scope on this request", None))?;
        let host = self.host.clone();
        let name = request.name.to_string();
        let args = Value::Object(request.arguments.unwrap_or_else(Map::new));
        let result = tokio::task::spawn_blocking(move || host.call(&scope, &name, args))
            .await
            .map_err(|e| McpError::internal_error(format!("tool task: {e}"), None))?;
        let value = serde_json::to_value(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let is_error = matches!(
            result.execution_status,
            ExecutionStatus::Refused | ExecutionStatus::Failed | ExecutionStatus::Cancelled
        );
        let mut out = if is_error {
            CallToolResult::structured_error(value.clone())
        } else {
            CallToolResult::structured(value.clone())
        };
        out.content = vec![ContentBlock::text(value.to_string())];
        Ok(out.into())
    }
}

/// Rejects requests without a live grant; attaches the granted scope for [`IgnisMcp::call_tool`].
async fn require_grant(
    State(grants): State<Arc<McpGrants>>,
    mut req: Request,
    next: Next,
) -> Response {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim);
    match token.and_then(|t| grants.resolve(t)) {
        Some(scope) => {
            req.extensions_mut().insert(scope);
            next.run(req).await
        }
        None => (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            "no live IGNIS MCP grant for this request",
        )
            .into_response(),
    }
}

/// The MCP endpoint as a router (mount it at a path of a loopback-only listener).
pub fn router(host: Arc<ToolHost>, grants: Arc<McpGrants>) -> Router {
    let handler = IgnisMcp::new(host);
    let service = StreamableHttpService::new(
        move || Ok(handler.clone()),
        Arc::new(LocalSessionManager::default()),
        // Loopback `Host` only (the default), keep-alive pings for long calls.
        StreamableHttpServerConfig::default(),
    );
    Router::new()
        .fallback_service(service)
        .layer(axum::middleware::from_fn_with_state(grants, require_grant))
}

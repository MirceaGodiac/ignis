//! The MCP endpoint over the wire (JSON-RPC on streamable HTTP, as the runtime speaks it): a request
//! needs a live host-issued grant, the grant decides the scope, and two runtime sessions of
//! different workspaces never see each other's jobs.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use ignis_research::runtime::McpGrants;
use serde_json::{json, Value};
use tower::ServiceExt;

const PROTOCOL: &str = "2025-06-18";

struct Client {
    router: axum::Router,
    token: Option<String>,
    session: Option<String>,
    next_id: u64,
}

/// The JSON-RPC message of a response (plain JSON, or the `data:` of an SSE stream).
fn message(content_type: &str, body: &[u8]) -> Option<Value> {
    let text = std::str::from_utf8(body).ok()?;
    if content_type.starts_with("text/event-stream") {
        text.lines()
            .filter_map(|l| l.strip_prefix("data:"))
            .filter_map(|d| serde_json::from_str::<Value>(d.trim()).ok())
            .find(|v| v.get("id").is_some())
    } else {
        serde_json::from_str(text).ok()
    }
}

impl Client {
    fn new(router: &axum::Router, token: Option<&str>) -> Self {
        Client {
            router: router.clone(),
            token: token.map(str::to_string),
            session: None,
            next_id: 1,
        }
    }

    async fn post(&mut self, body: Value) -> (StatusCode, Option<Value>) {
        let mut req = Request::post("/")
            .header(header::HOST, "127.0.0.1")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "application/json, text/event-stream")
            .header("mcp-protocol-version", PROTOCOL);
        if let Some(t) = &self.token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        if let Some(s) = &self.session {
            req = req.header("mcp-session-id", s);
        }
        let resp = self
            .router
            .clone()
            .oneshot(req.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        if let Some(s) = resp.headers().get("mcp-session-id") {
            self.session = Some(s.to_str().unwrap().to_string());
        }
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        // Read until the response message arrives (an SSE response stays open for keep-alives).
        let mut body = resp.into_body();
        let mut buf = Vec::new();
        loop {
            if let Some(m) = message(&ct, &buf) {
                return (status, Some(m));
            }
            match tokio::time::timeout(std::time::Duration::from_secs(10), body.frame()).await {
                Ok(Some(Ok(f))) => {
                    if let Ok(d) = f.into_data() {
                        buf.extend_from_slice(&d);
                    }
                }
                _ => return (status, message(&ct, &buf)),
            }
        }
    }

    async fn request(&mut self, method: &str, params: Value) -> (StatusCode, Option<Value>) {
        let id = self.next_id;
        self.next_id += 1;
        self.post(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .await
    }

    async fn initialize(&mut self) -> StatusCode {
        let (status, _) = self
            .request(
                "initialize",
                json!({"protocolVersion": PROTOCOL, "capabilities": {},
                       "clientInfo": {"name": "ignis-test", "version": "0"}}),
            )
            .await;
        if status == StatusCode::OK {
            self.post(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))
                .await;
        }
        status
    }

    /// `tools/call` → the `ToolResult` in `structuredContent`.
    async fn call(&mut self, tool: &str, args: Value) -> Value {
        let (status, msg) = self
            .request("tools/call", json!({"name": tool, "arguments": args}))
            .await;
        assert_eq!(status, StatusCode::OK);
        let msg = msg.expect("a response");
        msg["result"]["structuredContent"].clone()
    }
}

#[tokio::test]
async fn requests_without_a_live_grant_are_rejected() {
    let h = host();
    let grants = McpGrants::new();
    let router = ignis_mcp::router(h.host.clone(), grants.clone());

    assert_eq!(
        Client::new(&router, None).initialize().await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        Client::new(&router, Some("ignis_mcp_forged"))
            .initialize()
            .await,
        StatusCode::UNAUTHORIZED
    );

    let lease = grants.lease(&scope("ws_a"));
    let token = lease.token().expose().to_string();
    let mut c = Client::new(&router, Some(&token));
    assert_eq!(c.initialize().await, StatusCode::OK);
    // The runtime process went away: its token stops working mid-session.
    drop(lease);
    let (status, _) = c.request("tools/list", json!({})).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn tools_list_serves_the_registered_tools_with_their_schemas() {
    let h = host();
    let grants = McpGrants::new();
    let router = ignis_mcp::router(h.host.clone(), grants.clone());
    let lease = grants.lease(&scope("ws_a"));
    let mut c = Client::new(&router, Some(lease.token().expose()));
    assert_eq!(c.initialize().await, StatusCode::OK);
    let (_, msg) = c.request("tools/list", json!({})).await;
    let tools = msg.unwrap()["result"]["tools"].as_array().unwrap().clone();
    let mut names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    names.sort();
    let mut expected: Vec<&str> = h
        .host
        .registered()
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    expected.sort();
    assert_eq!(names, expected);
    let search = tools
        .iter()
        .find(|t| t["name"] == "sources_search")
        .unwrap();
    assert_eq!(search["inputSchema"]["type"], "object");
    assert_eq!(search["annotations"]["readOnlyHint"], true);
    let forecast = tools
        .iter()
        .find(|t| t["name"] == "models_forecast")
        .unwrap();
    assert_eq!(forecast["annotations"]["readOnlyHint"], false);
}

#[tokio::test]
async fn two_runtime_sessions_never_mix_workspaces() {
    let h = host();
    let grants = McpGrants::new();
    let router = ignis_mcp::router(h.host.clone(), grants.clone());
    let (la, lb) = (grants.lease(&scope("ws_a")), grants.lease(&scope("ws_b")));
    let mut a = Client::new(&router, Some(la.token().expose()));
    let mut b = Client::new(&router, Some(lb.token().expose()));
    assert_eq!(a.initialize().await, StatusCode::OK);
    assert_eq!(b.initialize().await, StatusCode::OK);

    let submitted = a.call("models_forecast", forecast_args("crid-a")).await;
    assert_eq!(submitted["execution_status"], "accepted");
    let job_id = submitted["job"]["job_id"].clone();

    // Concurrent sessions: A sees its job, B does not (not found, not denied: no existence leak).
    let (ra, rb) = tokio::join!(
        a.call("jobs_get", json!({"job_id": job_id})),
        b.call("jobs_get", json!({"job_id": job_id})),
    );
    assert_eq!(ra["execution_status"], "completed");
    assert_eq!(ra["result"]["value"]["job_id"], job_id);
    assert_eq!(rb["execution_status"], "refused");
    assert_eq!(rb["error"]["code"], "not_found");

    // The job was recorded under A's workspace by the host, whatever the arguments said.
    let stored = h
        .store
        .job(&scope("ws_a"), &serde_json::from_value(job_id).unwrap())
        .unwrap();
    assert_eq!(stored.kind, ignis_mcp::FORECAST_JOB);
}

#[tokio::test]
async fn a_refused_call_is_an_mcp_tool_error_with_the_envelope() {
    let h = host();
    let grants = McpGrants::new();
    let router = ignis_mcp::router(h.host.clone(), grants.clone());
    let lease = grants.lease(&scope("ws_a"));
    let mut c = Client::new(&router, Some(lease.token().expose()));
    assert_eq!(c.initialize().await, StatusCode::OK);
    let (_, msg) = c
        .request(
            "tools/call",
            json!({"name": "sources_search", "arguments": {"query": 1, "scope": "ws_b"}}),
        )
        .await;
    let result = &msg.unwrap()["result"];
    assert_eq!(result["isError"], true);
    assert_eq!(result["structuredContent"]["execution_status"], "refused");
    assert_eq!(
        result["structuredContent"]["error"]["code"],
        "invalid_request"
    );
    // The text content is the same ToolResult (for clients that read only text).
    let text = result["content"][0]["text"].as_str().unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(text).unwrap(),
        result["structuredContent"]
    );
}

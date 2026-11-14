//! Opt-in check against a REAL `codex` binary (ignored by default; never run in CI). No model call
//! and no credentials: the bridge starts the pinned app-server with the IGNIS MCP server attached and
//! opens a thread; Codex then connects to the loopback endpoint with its host-issued token and lists
//! the tools. Proves the `-c mcp_servers.ignis.*` registration, the bearer grant and that the pinned
//! Codex accepts the generated tool schemas.
//!
//! ```bash
//! IGNIS_CODEX_LIVE_BIN=/path/to/native/codex cargo test -p ignis-mcp --test live_codex -- --ignored
//! ```

mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::Request;
use axum::middleware::Next;
use common::*;
use ignis_core::contracts::{ClientRequestId, Origin};
use ignis_research::runtime::{
    preflight, AgentRuntime, CodexBridge, CodexConfig, McpAttachment, McpGrants,
};

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs IGNIS_CODEX_LIVE_BIN (a real codex binary)"]
async fn the_pinned_codex_lists_the_ignis_tools_with_its_grant() {
    let Some(bin) = std::env::var_os("IGNIS_CODEX_LIVE_BIN") else {
        eprintln!("IGNIS_CODEX_LIVE_BIN not set; skipped");
        return;
    };
    let h = host();
    let grants = McpGrants::new();
    // Record the JSON-RPC methods that reach the endpoint with a valid grant.
    let seen: Arc<Mutex<Vec<String>>> = Default::default();
    let log = seen.clone();
    let router = ignis_mcp::router(h.host.clone(), grants.clone()).layer(
        axum::middleware::from_fn(move |req: Request, next: Next| {
            let log = log.clone();
            async move {
                let (parts, body) = req.into_parts();
                let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
                if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    if let Some(m) = v["method"].as_str() {
                        log.lock().unwrap().push(m.to_string());
                    }
                }
                next.run(Request::from_parts(parts, axum::body::Body::from(bytes)))
                    .await
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    let dir = tempfile::tempdir().unwrap();
    let mut cfg = CodexConfig::new(bin, dir.path().join("codex"));
    cfg.mcp = Some(McpAttachment {
        url: format!("http://127.0.0.1:{port}/"),
        grants: grants.clone(),
        enabled_tools: h.host.registered().iter().map(|d| d.name.clone()).collect(),
        approved_tools: h.host.registered().iter().map(|d| d.name.clone()).collect(),
    });
    let (store, artifacts) = (h.store.clone(), h.artifacts.clone());
    let sc = scope("ws_live");
    let seen2 = seen.clone();
    let grants2 = grants.clone();
    tokio::task::spawn_blocking(move || {
        let manifest = preflight(&cfg).expect("preflight of the real binary");
        let bridge = CodexBridge::new(cfg, manifest, store.clone(), artifacts, Arc::new(|_, _| {}));
        let (ep, _) = store
            .create_episode(
                &sc,
                "live mcp",
                &ClientRequestId::new("live-1").unwrap(),
                Default::default(),
                Origin::System,
            )
            .unwrap();
        // No message: process + thread only, no turn and no model call.
        bridge.kick(&sc, &ep.episode_id).unwrap();
        let t0 = Instant::now();
        while !seen2.lock().unwrap().iter().any(|m| m == "tools/list") {
            assert!(
                t0.elapsed() < Duration::from_secs(90),
                "the runtime never listed the IGNIS tools; seen: {:?}",
                seen2.lock().unwrap()
            );
            std::thread::sleep(Duration::from_millis(200));
        }
        assert_eq!(grants2.live(), 1, "one grant for the one runtime process");
        bridge.shutdown();
        assert_eq!(grants2.live(), 0, "shutdown revokes the process's grant");
    })
    .await
    .unwrap();
    let methods = seen.lock().unwrap().clone();
    eprintln!("MCP methods from the real runtime: {methods:?}");
    assert!(methods.iter().any(|m| m == "initialize"));
}

/// A REAL model turn that calls an IGNIS tool (costs tokens or plan usage; explicit opt-in).
///
/// ```bash
/// IGNIS_CODEX_LIVE_BIN=... IGNIS_CODEX_LIVE_TURN=1 \
///   [IGNIS_CODEX_LIVE_LOGIN_HOME=~/.codex] [IGNIS_CODEX_LIVE_MODEL=...] \
///   cargo test -p ignis-mcp --test live_codex -- --ignored --nocapture live_turn
/// ```
#[tokio::test(flavor = "multi_thread")]
#[ignore = "calls a real model: needs IGNIS_CODEX_LIVE_BIN and IGNIS_CODEX_LIVE_TURN=1"]
async fn live_turn_calls_an_ignis_tool() {
    use ignis_core::contracts::{EpisodeStatus, EventBody};
    use ignis_research::store::Replay;
    let (Some(bin), Some(_)) = (
        std::env::var_os("IGNIS_CODEX_LIVE_BIN"),
        std::env::var_os("IGNIS_CODEX_LIVE_TURN"),
    ) else {
        eprintln!("IGNIS_CODEX_LIVE_BIN / IGNIS_CODEX_LIVE_TURN not set; skipped");
        return;
    };
    let h = host();
    let grants = McpGrants::new();
    let router = ignis_mcp::router(h.host.clone(), grants.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    let dir = tempfile::tempdir().unwrap();
    let mut cfg = CodexConfig::new(bin, dir.path().join("codex"));
    cfg.login_home = std::env::var_os("IGNIS_CODEX_LIVE_LOGIN_HOME").map(Into::into);
    cfg.model = std::env::var("IGNIS_CODEX_LIVE_MODEL").ok();
    let names: Vec<String> = h.host.registered().iter().map(|d| d.name.clone()).collect();
    cfg.mcp = Some(McpAttachment {
        url: format!("http://127.0.0.1:{port}/"),
        grants: grants.clone(),
        enabled_tools: names.clone(),
        approved_tools: names,
    });
    let (store, artifacts) = (h.store.clone(), h.artifacts.clone());
    let knowledge = h.knowledge.clone();
    tokio::task::spawn_blocking(move || {
        let manifest = preflight(&cfg).expect("preflight of the real binary");
        let bridge = CodexBridge::new(cfg, manifest, store.clone(), artifacts, Arc::new(|_, _| {}));
        let sc = scope("ws_live");
        let (ep, _) = store
            .create_episode(
                &sc,
                "live tool turn",
                &ClientRequestId::new("live-turn-1").unwrap(),
                Default::default(),
                Origin::System,
            )
            .unwrap();
        let ep = ep.episode_id;
        store
            .post_message(
                &sc,
                &ep,
                "Call the IGNIS tool sources_search exactly once with arguments \
                 {\"schema_version\":\"evidence.v1\",\"query\":\"flame spread in microgravity\"} \
                 (add any other required fields), then reply with one sentence stating its \
                 execution_status and how many hits it returned. Do not call any other tool.",
                &ClientRequestId::new("live-msg-1").unwrap(),
            )
            .unwrap();
        bridge.kick(&sc, &ep).unwrap();
        let t0 = Instant::now();
        let events = loop {
            let evs = match store.replay(&sc, &ep, 0, 100_000).unwrap() {
                Replay::Events { events, .. } => events,
                Replay::Gone { .. } => panic!("gone"),
            };
            let status = evs.iter().rev().find_map(|e| match e.body {
                EventBody::StatusChanged { to, .. } => Some(to),
                _ => None,
            });
            if matches!(
                status,
                Some(EpisodeStatus::Completed | EpisodeStatus::Failed | EpisodeStatus::WaitingUser)
            ) {
                break evs;
            }
            assert!(
                t0.elapsed() < Duration::from_secs(300),
                "turn did not finish"
            );
            std::thread::sleep(Duration::from_millis(500));
        };
        bridge.shutdown();
        ignis_core::contracts::check_event_log(&events).unwrap();
        for e in &events {
            let line = match &e.body {
                EventBody::StatusChanged { from, to, reason } => {
                    format!(
                        "status {from:?} -> {to:?} {}",
                        reason.clone().unwrap_or_default()
                    )
                }
                EventBody::ToolStarted { tool, call_id, .. } => {
                    format!("tool.started {tool} ({call_id})")
                }
                EventBody::ToolFinished { result } => format!(
                    "tool.finished {} ({}) {:?} {}",
                    result.tool,
                    result.call_id,
                    result.execution_status,
                    result
                        .error
                        .as_ref()
                        .map(|e| e.message.clone())
                        .unwrap_or_default()
                ),
                EventBody::AssistantMessage { text, .. } => format!("message.completed: {text}"),
                EventBody::UserMessage { .. } => "message.user".into(),
                EventBody::RuntimeOpaque { method, .. } => format!("runtime.opaque {method}"),
                other => format!("{:?}", std::mem::discriminant(other)),
            };
            eprintln!("[{}] {line}", e.seq);
        }
        let finished = events.iter().find_map(|e| match &e.body {
            EventBody::ToolFinished { result } if result.tool == "sources_search" => {
                Some(result.clone())
            }
            _ => None,
        });
        let result =
            finished.expect("the model called sources_search through the IGNIS MCP server");
        assert_eq!(
            result.execution_status,
            ignis_core::contracts::ExecutionStatus::Completed,
            "{result:#?}"
        );
        assert_eq!(
            knowledge.calls.lock().unwrap().as_slice(),
            &[sc],
            "service got the host scope"
        );
    })
    .await
    .unwrap();
}

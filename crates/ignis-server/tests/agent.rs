//! `/api/v1` with the agent runtime (M1), against the fake app-server (no model, no network):
//! capability status for a missing / wrong / pinned binary, and one turn on the fixture episode in
//! fixture mode — message → runtime events on the SSE stream → an approval denied through the API.

use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use ignis_server::v1::agent::CodexArgs;
use ignis_server::v1::io::AuthMode;
use ignis_server::{app, fixture, ServeArgs, Services};
use serde_json::{json, Value};
use tower::ServiceExt;

mod support;

fn args(dir: &Path, codex: CodexArgs) -> ServeArgs {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ServeArgs {
        port: 0,
        host: "127.0.0.1".into(),
        root: crate_dir.join("../.."),
        data: dir.join("no-registry"),
        cohorts: dir.join("no-cohorts"),
        splits: dir.join("no-splits"),
        raw_samples: dir.join("no-raw"),
        artifacts: dir.join("artifacts"),
        evidence_db: dir.join("none.sqlite"),
        compute_threads: 2,
        dev: true,
        fixtures: crate_dir.join("fixtures"),
        web_dist: dir.join("no-dist"),
        state_dir: dir.join("state"),
        auth: AuthMode::Local,
        internal_routes: false,
        enable_models: true,
        answer_grounding: Default::default(),
        max_upload_bytes: 1 << 20,
        contract_fixtures: "fixtures/contracts".into(),
        codex,
        mcp_port: 0,
        runner: Default::default(),
        sources: Default::default(),
        price_list: None,
        artifact_backend: Default::default(),
        r2: Default::default(),
        access: Default::default(),
    }
}

fn fake() -> CodexArgs {
    CodexArgs {
        bin: Some(env!("CARGO_BIN_EXE_ignis-server-fake-codex").into()),
        ..Default::default()
    }
}

fn router(dir: &Path, codex: CodexArgs) -> axum::Router {
    app(
        args(dir, codex),
        Services {
            experts: Vec::new(),
            fixture_experts: fixture::experts(),
            acquisition: None,
            research: Default::default(),
        },
    )
}

async fn call(
    r: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let req = Request::builder().method(method).uri(path);
    let req = match body {
        Some(b) => req
            .header("content-type", "application/json")
            .body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let resp = r.clone().oneshot(req).await.unwrap();
    let st = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 24)
        .await
        .unwrap();
    (st, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

/// Read the SSE stream after `after` until `done` holds for the events read so far.
async fn sse_until(r: &axum::Router, path: &str, done: impl Fn(&[Value]) -> bool) -> Vec<Value> {
    let resp = r
        .clone()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let mut body = resp.into_body();
    let (mut text, mut out) = (String::new(), Vec::new());
    // One deadline for the whole wait: heartbeats keep the stream busy, so a per-frame timeout
    // would never fire.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while !done(&out) {
        let frame = tokio::time::timeout_at(deadline, body.frame())
            .await
            .unwrap_or_else(|_| panic!("condition not met within 30 s; got {out:#?}"))
            .expect("stream open")
            .unwrap();
        if let Ok(data) = frame.into_data() {
            text.push_str(std::str::from_utf8(&data).unwrap());
        }
        while let Some(i) = text.find("\n\n") {
            let block: String = text.drain(..i + 2).collect();
            let data: String = block
                .lines()
                .filter_map(|l| l.strip_prefix("data:"))
                .map(str::trim_start)
                .collect();
            if !data.is_empty() {
                out.push(serde_json::from_str(&data).unwrap());
            }
        }
    }
    out
}

fn kind(e: &Value) -> &str {
    e["body"]["kind"].as_str().unwrap()
}

fn to_status(e: &Value) -> Option<&str> {
    (kind(e) == "episode.status_changed").then(|| e["body"]["payload"]["to"].as_str().unwrap())
}

#[tokio::test(flavor = "multi_thread")]
async fn capabilities_report_the_runtime_without_crashing() {
    let dir = tempfile::tempdir().unwrap();
    let (_, caps) = call(
        &router(dir.path(), CodexArgs::default()),
        "GET",
        "/api/v1/capabilities",
        None,
    )
    .await;
    assert_eq!(caps["agent_runtime"]["available"], false);
    assert!(caps["agent_runtime"]["detail"]
        .as_str()
        .unwrap()
        .contains("not configured"));

    // A reasoning effort Codex does not know leaves the runtime unavailable, saying why.
    let dir = tempfile::tempdir().unwrap();
    let effort = CodexArgs {
        reasoning_effort: Some("turbo".into()),
        ..fake()
    };
    let (_, caps) = call(
        &router(dir.path(), effort),
        "GET",
        "/api/v1/capabilities",
        None,
    )
    .await;
    assert_eq!(caps["agent_runtime"]["available"], false);
    assert!(caps["agent_runtime"]["detail"]
        .as_str()
        .unwrap()
        .contains("reasoning effort"));

    let wrong = CodexArgs {
        sha256: Some("00".repeat(32)),
        ..fake()
    };
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), wrong);
    let (_, caps) = call(&r, "GET", "/api/v1/capabilities", None).await;
    assert_eq!(caps["agent_runtime"]["available"], false);
    assert!(caps["agent_runtime"]["detail"]
        .as_str()
        .unwrap()
        .contains("does not match the pinned"));
    assert!(caps.get("runtime_build").is_none());
    // Messages are still recorded; resolving an approval says the runtime is unavailable.
    let (st, ep) = call(
        &r,
        "POST",
        "/api/v1/episodes",
        Some(json!({"goal": "g", "client_request_id": "c1"})),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED);
    let id = ep["episode_id"].as_str().unwrap();
    let (st, _) = call(
        &r,
        "POST",
        &format!("/api/v1/episodes/{id}/messages"),
        Some(json!({"text": "hi", "client_request_id": "m1"})),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED);
    let (st, err) = call(
        &r,
        "POST",
        &format!("/api/v1/episodes/{id}/approvals/apr_x"),
        Some(json!({"decision": "approved"})),
    )
    .await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE, "{err}");

    // A shared signed-in Codex home (a ChatGPT plan) also serves a deployment behind Access,
    // and capabilities say which credentials the runtime uses.
    let dir = tempfile::tempdir().unwrap();
    let login = dir.path().join("login");
    std::fs::create_dir_all(&login).unwrap();
    std::fs::write(login.join("auth.json"), "{}").unwrap();
    let mut a = args(
        dir.path(),
        CodexArgs {
            login_home: Some(login),
            ..fake()
        },
    );
    a.auth = AuthMode::CloudflareAccess;
    a.access = support::access().args.clone();
    let r = app(
        a,
        Services {
            experts: Vec::new(),
            fixture_experts: fixture::experts(),
            acquisition: None,
            research: Default::default(),
        },
    );
    let mut req = Request::get("/api/v1/capabilities");
    for (k, v) in support::access().headers("someone@example.org") {
        req = req.header(k, v);
    }
    let req = req.body(Body::empty()).unwrap();
    let caps: Value = serde_json::from_slice(
        &axum::body::to_bytes(r.oneshot(req).await.unwrap().into_body(), 1 << 20)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(caps["agent_runtime"]["available"], true, "{caps}");
    assert!(caps["agent_runtime"]["detail"]
        .as_str()
        .unwrap()
        .contains("credentials: shared_login_home"));
    let dir = tempfile::tempdir().unwrap();
    let (_, caps) = call(
        &router(dir.path(), fake()),
        "GET",
        "/api/v1/capabilities",
        None,
    )
    .await;
    assert_eq!(caps["agent_runtime"]["available"], true, "{caps}");
    assert!(caps["agent_runtime"]["detail"]
        .as_str()
        .unwrap()
        .contains("harness h1 sha256:"));
    let build = &caps["runtime_build"];
    assert_eq!(build["version"], "0.169.0");
    assert_eq!(build["transport"], "stdio");
    assert_eq!(build["profile"], "public");
    assert!(build["config_overrides"]
        .as_array()
        .unwrap()
        .contains(&json!("features.shell_tool=false")));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_turn_on_the_fixture_episode_streams_runtime_events_and_a_denied_approval() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), fake());
    let base = "/api/fixtures/v1";
    let (_, eps) = call(&r, "GET", &format!("{base}/episodes"), None).await;
    let ep = &eps.as_array().unwrap()[0];
    let id = ep["episode_id"].as_str().unwrap().to_string();
    let after = ep["last_seq"].as_u64().unwrap();

    let (st, acc) = call(
        &r,
        "POST",
        &format!("{base}/episodes/{id}/messages"),
        Some(json!({"text": "search with a tool, then ask for an approval", "client_request_id": "turn-1"})),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{acc}");
    let events_path = format!("{base}/episodes/{id}/events?after={after}");
    let evs = sse_until(&r, &events_path, |e| {
        e.iter().any(|x| kind(x) == "approval.requested")
    })
    .await;
    let approval = evs
        .iter()
        .find(|x| kind(x) == "approval.requested")
        .unwrap()["body"]["payload"]["approval_id"]
        .as_str()
        .unwrap()
        .to_string();
    let (st, res) = call(
        &r,
        "POST",
        &format!("{base}/episodes/{id}/approvals/{approval}"),
        Some(json!({"decision": "denied"})),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{res}");
    let (st, _) = call(
        &r,
        "POST",
        &format!("{base}/episodes/{id}/approvals/{approval}"),
        Some(json!({"decision": "approved"})),
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND, "an approval is resolved once");

    let evs = sse_until(&r, &events_path, |e| {
        e.iter().any(|x| to_status(x) == Some("completed"))
    })
    .await;
    let kinds: Vec<&str> = evs.iter().map(kind).collect();
    let seqs: Vec<u64> = evs.iter().map(|e| e["seq"].as_u64().unwrap()).collect();
    assert_eq!(
        seqs,
        (after + 1..after + 1 + evs.len() as u64).collect::<Vec<_>>(),
        "gap-free"
    );
    for k in [
        "message.user",
        "tool.started",
        "tool.finished",
        "approval.requested",
        "approval.resolved",
        "message.delta",
        "message.completed",
    ] {
        assert!(kinds.contains(&k), "{k} missing from {kinds:?}");
    }
    let resolved = evs.iter().find(|x| kind(x) == "approval.resolved").unwrap();
    assert_eq!(resolved["body"]["payload"]["decision"], "denied");
    let tool = evs.iter().find(|x| kind(x) == "tool.finished").unwrap();
    assert_eq!(
        tool["body"]["payload"]["result"]["origin"],
        "synthetic_fixture"
    );
    let answer = evs.iter().find(|x| kind(x) == "message.completed").unwrap();
    assert_eq!(
        answer["body"]["payload"]["text"],
        "item/fileChange/requestApproval -> decline"
    );
    assert!(evs.iter().all(|e| e["origin"] == "synthetic_fixture"));

    // The episode now names its runtime thread; the registry instance is untouched.
    let (_, rec) = call(&r, "GET", &format!("{base}/episodes/{id}"), None).await;
    assert_eq!(rec["runtime"]["runtime"], "codex_app_server");
    assert_eq!(rec["status"], "completed");
    // The harness bundle the runtime started with is pinned on the episode and said in the log.
    let pin = &rec["pins"]["harness"];
    assert_eq!(pin["label"], "h1", "{rec}");
    assert_eq!(pin["runtime_build"], rec["runtime"]["runtime_build_id"]);
    let digest = pin["digest"].as_str().unwrap();
    let short = format!("harness h1 {}", &digest[..7 + 12]);
    assert!(
        evs.iter().any(|e| to_status(e) == Some("running")
            && e["body"]["payload"]["reason"]
                .as_str()
                .is_some_and(|r| r.contains(&short))),
        "{evs:?}"
    );
    let (_, other) = call(&r, "GET", "/api/v1/episodes", None).await;
    assert!(other
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["episode_id"] != json!(id)));
}

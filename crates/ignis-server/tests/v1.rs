//! `/api/v1` conformance: idempotent creation, workspace isolation, resumable SSE (cursor, header,
//! 410), jobs with a fake `ModelService`, artifact reads, capability status, OpenAPI, restart.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use ignis_core::contracts::*;
use ignis_core::services::{ModelService, ServiceResult};
use ignis_server::v1::io::AuthMode;
use ignis_server::v1::ResearchServices;
use ignis_server::{app, fixture, ServeArgs, Services};
use serde_json::{json, Value};
use tower::ServiceExt;

#[path = "../../ignis-research/tests/s3fake/mod.rs"]
mod s3fake;
mod support;

const RELEASE: &str = "sha256:db4ebefdf4b70e96202de6c51e1b9f7690005302cc73374ef03644abdd51e6cc";

fn args(dir: &Path, auth: AuthMode) -> ServeArgs {
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
        auth,
        internal_routes: false,
        max_upload_bytes: 1 << 20,
        contract_fixtures: "fixtures/contracts".into(),
        codex: Default::default(),
        mcp_port: 0,
        runner: Default::default(),
        artifact_backend: Default::default(),
        r2: Default::default(),
        access: support::access().args.clone(),
    }
}

/// The two Access assertions the Worker forwards for `user`.
fn as_user(mut req: axum::http::request::Builder, user: &str) -> axum::http::request::Builder {
    for (k, v) in support::access().headers(user) {
        req = req.header(k, v);
    }
    req
}

fn router(dir: &Path, auth: AuthMode, models: Option<Arc<dyn ModelService>>) -> axum::Router {
    app(
        args(dir, auth),
        Services {
            experts: Vec::new(),
            fixture_experts: fixture::experts(),
            acquisition: None,
            research: ResearchServices {
                models,
                ..Default::default()
            },
        },
    )
}

async fn call(
    r: &axum::Router,
    method: &str,
    path: &str,
    user: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(u) = user {
        req = as_user(req, u);
    }
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

/// Open the SSE stream and collect `(id, event, data)` frames until `n` events arrived.
async fn sse(
    r: &axum::Router,
    path: &str,
    last_event_id: Option<u64>,
    n: usize,
) -> Vec<(u64, String, Value)> {
    let mut req = Request::get(path);
    if let Some(l) = last_event_id {
        req = req.header("last-event-id", l.to_string());
    }
    let resp = r
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()["content-type"].to_str().unwrap(),
        "text/event-stream"
    );
    let mut body = resp.into_body();
    let mut text = String::new();
    let mut out = Vec::new();
    while out.len() < n {
        let frame = tokio::time::timeout(Duration::from_secs(20), body.frame())
            .await
            .expect("event within 20 s")
            .expect("stream open")
            .unwrap();
        if let Ok(data) = frame.into_data() {
            text.push_str(std::str::from_utf8(&data).unwrap());
        }
        while let Some(i) = text.find("\n\n") {
            let block: String = text.drain(..i + 2).collect();
            let (mut id, mut ev, mut data) = (None, String::new(), String::new());
            for line in block.lines() {
                if let Some(v) = line.strip_prefix("id:") {
                    id = Some(v.trim().parse::<u64>().unwrap());
                } else if let Some(v) = line.strip_prefix("event:") {
                    ev = v.trim().to_string();
                } else if let Some(v) = line.strip_prefix("data:") {
                    data.push_str(v.trim_start());
                }
            }
            if let Some(id) = id {
                out.push((id, ev, serde_json::from_str(&data).unwrap()));
            }
        }
    }
    out
}

#[tokio::test(flavor = "multi_thread")]
async fn episodes_messages_and_resumable_stream() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), AuthMode::Local, None);
    let create = json!({"goal": "Why does the flame shrink?", "client_request_id": "req-1"});
    let (st, ep) = call(&r, "POST", "/api/v1/episodes", None, Some(create.clone())).await;
    assert_eq!(st, StatusCode::ACCEPTED);
    let id = ep["episode_id"].as_str().unwrap().to_string();
    assert_eq!(ep["status"], "ready");
    let (st, again) = call(&r, "POST", "/api/v1/episodes", None, Some(create)).await;
    assert_eq!(st, StatusCode::ACCEPTED);
    assert_eq!(again["episode_id"], ep["episode_id"]);

    let (st, err) = call(
        &r,
        "POST",
        "/api/v1/episodes",
        None,
        Some(json!({"goal": "x", "client_request_id": "req-1", "extra": 1})),
    )
    .await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(err["code"], "invalid_request");

    let msg = |crid: &str, text: &str| json!({"client_request_id": crid, "text": text});
    let path = format!("/api/v1/episodes/{id}/messages");
    let (st, m1) = call(&r, "POST", &path, None, Some(msg("m-1", "hello"))).await;
    assert_eq!(st, StatusCode::ACCEPTED);
    assert_eq!(
        (m1["seq"].as_u64(), m1["duplicate"].as_bool()),
        (Some(1), Some(false))
    );
    let (_, dup) = call(&r, "POST", &path, None, Some(msg("m-1", "hello"))).await;
    assert_eq!(
        (dup["seq"].as_u64(), dup["duplicate"].as_bool()),
        (Some(1), Some(true))
    );
    call(&r, "POST", &path, None, Some(msg("m-2", "second"))).await;
    let (st, paused) = call(
        &r,
        "POST",
        &format!("/api/v1/episodes/{id}/pause"),
        None,
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(paused["status"], "paused");

    // Full replay, then resume after a cursor (query and Last-Event-ID), then live delivery.
    let events = sse(&r, &format!("/api/v1/episodes/{id}/events"), None, 3).await;
    assert_eq!(
        events.iter().map(|e| e.0).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(events[0].1, "message.user");
    assert_eq!(events[2].1, "episode.status_changed");
    let parsed: Vec<ResearchEvent> = events
        .iter()
        .map(|e| serde_json::from_value(e.2.clone()).unwrap())
        .collect();
    check_event_log(&parsed).unwrap();

    let tail = sse(
        &r,
        &format!("/api/v1/episodes/{id}/events?after=1"),
        None,
        2,
    )
    .await;
    assert_eq!(tail.iter().map(|e| e.0).collect::<Vec<_>>(), vec![2, 3]);

    let live = {
        let r = r.clone();
        let url = format!("/api/v1/episodes/{id}/events");
        tokio::spawn(async move { sse(&r, &url, Some(3), 1).await })
    };
    tokio::time::sleep(Duration::from_millis(200)).await;
    call(
        &r,
        "POST",
        &format!("/api/v1/episodes/{id}/resume"),
        None,
        None,
    )
    .await;
    let live = live.await.unwrap();
    assert_eq!(live[0].0, 4);
    assert_eq!(live[0].2["body"]["payload"]["to"], "ready");

    let (st, gone) = call(
        &r,
        "GET",
        &format!("/api/v1/episodes/{id}/events?after=99"),
        None,
        None,
    )
    .await;
    assert_eq!(st, StatusCode::GONE);
    assert_eq!(gone["code"], "conflict");

    let (st, bad) = call(
        &r,
        "POST",
        &format!("/api/v1/episodes/{id}/explode"),
        None,
        None,
    )
    .await;
    assert!(st.is_client_error(), "{st} {bad}");

    // The state survives a restart of the server.
    drop(r);
    let r2 = router(dir.path(), AuthMode::Local, None);
    let (st, ep2) = call(&r2, "GET", &format!("/api/v1/episodes/{id}"), None, None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(ep2["last_seq"], 4);
}

#[tokio::test(flavor = "multi_thread")]
async fn two_users_never_see_each_other() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), AuthMode::CloudflareAccess, None);
    let (st, err) = call(&r, "GET", "/api/v1/episodes", None, None).await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
    assert_eq!(err["code"], "scope_denied");

    let body = json!({"goal": "g", "client_request_id": "same-key"});
    let (_, a) = call(
        &r,
        "POST",
        "/api/v1/episodes",
        Some("ana@example.org"),
        Some(body.clone()),
    )
    .await;
    let (_, b) = call(
        &r,
        "POST",
        "/api/v1/episodes",
        Some("bob@example.org"),
        Some(body),
    )
    .await;
    assert_ne!(a["episode_id"], b["episode_id"]);
    assert_ne!(a["workspace_id"], b["workspace_id"]);
    let a_id = a["episode_id"].as_str().unwrap();
    for (m, p) in [
        ("GET", format!("/api/v1/episodes/{a_id}")),
        ("GET", format!("/api/v1/episodes/{a_id}/events")),
        ("POST", format!("/api/v1/episodes/{a_id}/cancel")),
    ] {
        let (st, e) = call(&r, m, &p, Some("bob@example.org"), None).await;
        assert_eq!(st, StatusCode::NOT_FOUND, "{m} {p}");
        assert_eq!(e["code"], "not_found");
    }
    let (_, list) = call(
        &r,
        "GET",
        "/api/v1/episodes",
        Some("ANA@example.org "),
        None,
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    // No identity material in the ids.
    assert!(!a["workspace_id"].as_str().unwrap().contains("ana"));
}

#[tokio::test(flavor = "multi_thread")]
async fn unconfigured_services_are_capability_status_not_results() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), AuthMode::Local, None);
    let (st, caps) = call(&r, "GET", "/api/v1/capabilities", None, None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(caps["knowledge"]["available"], false);
    assert_eq!(caps["agent_runtime"]["available"], false);
    let tools = caps["tools"].as_array().unwrap();
    assert_eq!(tools.len(), tool_catalog().len());
    let artifacts_read = tools
        .iter()
        .find(|t| t["name"] == "artifacts_read")
        .unwrap();
    assert_eq!(artifacts_read["available"], true);
    // No runner configured: no code execution anywhere, and the reason is visible.
    assert_eq!(caps["analysis_runner"]["available"], false);
    let analysis = tools
        .iter()
        .find(|t| t["name"] == "analysis_execute")
        .unwrap();
    assert_eq!(analysis["available"], false);
    assert!(caps.get("runner_environment").is_none());

    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let search: Value = serde_json::from_str(
        &std::fs::read_to_string(
            crate_dir.join("../../fixtures/contracts/evidence/search_request.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let (st, e) = call(&r, "POST", "/api/v1/search", None, Some(search)).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(e["code"], "capability_unavailable");
    let (st, e) = call(&r, "GET", "/api/v1/models", None, None).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(e["code"], "capability_unavailable");

    let (st, doc) = call(&r, "GET", "/api/v1/openapi.json", None, None).await;
    assert_eq!(st, StatusCode::OK);
    let paths = doc["paths"].as_object().unwrap();
    for p in [
        "/episodes",
        "/episodes/{id}/events",
        "/analyses",
        "/artifacts/{id}",
        "/jobs/{id}",
        "/uploads/initiate",
        "/uploads/{id}/content",
        "/uploads/complete",
    ] {
        assert!(
            paths.contains_key(p),
            "{p} missing from OpenAPI: {:?}",
            paths.keys()
        );
    }
    let events = &doc["paths"]["/episodes/{id}/events"]["get"]["responses"]["200"]["content"];
    assert!(events.get("text/event-stream").is_some(), "{events}");
}

/// A `ModelService` returning a fixed artifact (a real one, produced by the synthetic fixture
/// expert through the legacy route) and recording the scope it was called with.
struct FakeModels {
    artifact: AnalysisArtifact,
    seen: Mutex<Vec<ExecutionScope>>,
}

impl ModelService for FakeModels {
    fn releases(&self, _: &ExecutionScope) -> ServiceResult<Vec<ModelRelease>> {
        Ok(vec![])
    }
    fn release(&self, _: &ExecutionScope, id: &ReleaseId) -> ServiceResult<ModelRelease> {
        Err(ErrorEnvelope::new(
            ErrorCode::NotFound,
            format!("{id}"),
            RetryPolicy::Never,
        ))
    }
    fn forecast(
        &self,
        scope: &ExecutionScope,
        req: &ForecastRequest,
    ) -> ServiceResult<AnalysisArtifact> {
        self.seen.lock().unwrap().push(scope.clone());
        if matches!(&req.input, ForecastInput::StoredRun { run_id, .. } if run_id.as_str() == "broken")
        {
            return Err(ErrorEnvelope::new(
                ErrorCode::Internal,
                "synthetic failure",
                RetryPolicy::Never,
            ));
        }
        Ok(self.artifact.clone())
    }
}

fn forecast_req(run: &str, crid: &str) -> Value {
    serde_json::to_value(ForecastRequest {
        schema_version: Default::default(),
        release_id: ReleaseId::new(RELEASE).unwrap(),
        input: ForecastInput::StoredRun {
            run_id: RunId::new(run).unwrap(),
            cutoff_s: Finite::new(2.0).unwrap(),
        },
        settings: Default::default(),
        client_request_id: Some(ClientRequestId::new(crid).unwrap()),
    })
    .unwrap()
}

async fn wait_job(r: &axum::Router, id: &str) -> Value {
    for _ in 0..200 {
        let (_, j) = call(r, "GET", &format!("/api/v1/jobs/{id}"), None, None).await;
        if ["succeeded", "failed", "cancelled"].contains(&j["status"].as_str().unwrap_or("")) {
            return j;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("job {id} did not finish");
}

#[tokio::test(flavor = "multi_thread")]
async fn analyses_run_as_jobs_and_outputs_are_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    // A real AnalysisArtifact from the synthetic expert (legacy fixture route).
    let r0 = router(dir.path(), AuthMode::Local, None);
    let (st, v) = call(&r0, "POST", "/api/fixtures/forecast", None,
        Some(json!({"run_id": "fixture-flex-3", "cutoff_s": 2.0, "model_release_id": RELEASE, "wait_ms": 10000}))).await;
    assert_eq!(st, StatusCode::OK, "{v}");
    let artifact: AnalysisArtifact = serde_json::from_value(v["artifact"].clone()).unwrap();
    drop(r0);

    let models = Arc::new(FakeModels {
        artifact: artifact.clone(),
        seen: Mutex::new(vec![]),
    });
    let r = router(dir.path(), AuthMode::Local, Some(models.clone()));
    let (_, ep) = call(
        &r,
        "POST",
        "/api/v1/episodes",
        None,
        Some(json!({"goal": "forecast", "client_request_id": "e"})),
    )
    .await;
    let ep_id = ep["episode_id"].as_str().unwrap().to_string();

    let url = format!("/api/v1/analyses?episode_id={ep_id}");
    let (st, tr) = call(
        &r,
        "POST",
        &url,
        None,
        Some(forecast_req("fixture-flex-3", "f-1")),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{tr}");
    let tool: ToolResult = serde_json::from_value(tr.clone()).unwrap();
    tool.validate().unwrap();
    assert_eq!(tool.execution_status, ExecutionStatus::Accepted);
    let job_id = tool.job.unwrap().job_id.to_string();
    // Retrying the submission does not create a second job.
    let (_, tr2) = call(
        &r,
        "POST",
        &url,
        None,
        Some(forecast_req("fixture-flex-3", "f-1")),
    )
    .await;
    assert_eq!(tr2["job"]["job_id"], json!(job_id));

    let job = wait_job(&r, &job_id).await;
    assert_eq!(job["status"], "succeeded", "{job}");
    let out = &job["outputs"][0];
    assert_eq!(out["contract"], "AnalysisArtifact");
    let art_id = out["artifact_id"].as_str().unwrap();
    assert_eq!(
        models.seen.lock().unwrap()[0].partitions.len(),
        1,
        "research scope: train only"
    );

    let (st, pv) = call(
        &r,
        "GET",
        &format!("/api/v1/artifacts/{art_id}?pointer=/id"),
        None,
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(pv["value"], json!(artifact.id));
    assert_eq!(pv["partial"], false);
    let resp = r
        .clone()
        .oneshot(
            Request::get(format!("/api/v1/artifacts/{art_id}?offset=0&len=1"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.headers()["content-type"], "application/json");
    assert_eq!(
        axum::body::to_bytes(resp.into_body(), 16)
            .await
            .unwrap()
            .as_ref(),
        b"{"
    );

    // The job's lifecycle is in the episode log.
    let events = sse(&r, &format!("/api/v1/episodes/{ep_id}/events"), None, 3).await;
    let statuses: Vec<&str> = events
        .iter()
        .map(|e| e.2["body"]["payload"]["job"]["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, vec!["queued", "running", "succeeded"]);

    // A service failure is a failed job with an error envelope, not a 500.
    let (_, tr) = call(
        &r,
        "POST",
        "/api/v1/analyses",
        None,
        Some(forecast_req("broken", "f-2")),
    )
    .await;
    let job = wait_job(&r, tr["job"]["job_id"].as_str().unwrap()).await;
    assert_eq!(job["status"], "failed");
    assert_eq!(job["error"]["code"], "internal");
}

async fn put_bytes(
    r: &axum::Router,
    path: &str,
    user: Option<&str>,
    bytes: &[u8],
) -> (StatusCode, Value) {
    let mut req = Request::put(path);
    if let Some(u) = user {
        req = as_user(req, u);
    }
    let resp = r
        .clone()
        .oneshot(req.body(Body::from(bytes.to_vec())).unwrap())
        .await
        .unwrap();
    let st = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .unwrap();
    (st, serde_json::from_slice(&b).unwrap_or(Value::Null))
}

#[tokio::test(flavor = "multi_thread")]
async fn uploads_are_verified_then_published_as_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    upload_flow(router(dir.path(), AuthMode::CloudflareAccess, None)).await;
}

/// The same flow with the deployment's backend: bytes and manifests end up in the (stand-in) R2
/// bucket under the instance's prefix, and reads stream back from it.
#[tokio::test(flavor = "multi_thread")]
async fn uploads_land_in_the_r2_bucket_with_the_r2_backend() {
    let f = s3fake::fake();
    let dir = tempfile::tempdir().unwrap();
    let mut a = args(dir.path(), AuthMode::CloudflareAccess);
    a.artifact_backend = ignis_server::ops::ArtifactBackend::R2;
    a.r2 = ignis_server::ops::R2Args {
        endpoint: Some(f.endpoint.clone()),
        bucket: Some("ignis-test".into()),
        access_key_id: Some(s3fake::KEY_ID.into()),
        secret_access_key: Some(s3fake::SECRET.into()),
    };
    let r = app(
        a,
        Services {
            experts: Vec::new(),
            fixture_experts: fixture::experts(),
            acquisition: None,
            research: Default::default(),
        },
    );
    upload_flow(r).await;
    let b = f.bucket.lock().unwrap();
    assert!(b.refused.is_empty(), "{:?}", b.refused);
    assert!(
        b.objects.keys().any(|k| k.starts_with("registry/objects/")),
        "{:?}",
        b.objects.keys()
    );
    assert!(b
        .objects
        .keys()
        .any(|k| k.starts_with("registry/manifests/")));
}

async fn upload_flow(r: axum::Router) {
    let ana = Some("ana@example.org");
    let bytes = b"t_s,d_m\n0,0.001\n".to_vec();
    let decl = json!({"media_type": "text/csv", "byte_length": bytes.len(),
                      "sha256": Digest::of_bytes(&bytes), "client_request_id": "up-1"});
    let (st, t) = call(
        &r,
        "POST",
        "/api/v1/uploads/initiate",
        ana,
        Some(decl.clone()),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{t}");
    assert_eq!(t["state"], "initiated");
    let (_, t2) = call(&r, "POST", "/api/v1/uploads/initiate", ana, Some(decl)).await;
    assert_eq!(t2["upload_id"], t["upload_id"], "idempotent initiate");
    let put = format!("/api/v1/{}", t["put_path"].as_str().unwrap());
    let id = t["upload_id"].as_str().unwrap().to_string();

    // Too early, wrong bytes, too many bytes, another user: all refused.
    let (st, e) = call(
        &r,
        "POST",
        "/api/v1/uploads/complete",
        ana,
        Some(json!({"upload_id": id})),
    )
    .await;
    assert_eq!(
        (st, e["code"].as_str()),
        (StatusCode::CONFLICT, Some("conflict"))
    );
    let mut wrong = bytes.clone();
    wrong[0] = b'x';
    let (st, _) = put_bytes(&r, &put, ana, &wrong).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    let (st, _) = put_bytes(&r, &put, ana, &[bytes.as_slice(), b"!"].concat()).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    let (st, _) = put_bytes(&r, &put, Some("bob@example.org"), &bytes).await;
    assert_eq!(st, StatusCode::NOT_FOUND);

    let (st, t) = put_bytes(&r, &put, ana, &bytes).await;
    assert_eq!(st, StatusCode::OK, "{t}");
    assert_eq!(t["state"], "received");
    let (st, a) = call(
        &r,
        "POST",
        "/api/v1/uploads/complete",
        ana,
        Some(json!({"upload_id": id})),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{a}");
    let art: ArtifactRef = serde_json::from_value(a.clone()).unwrap();
    assert_eq!(art.sha256, Digest::of_bytes(&bytes));
    assert_eq!(art.media_type, "text/csv");
    let (_, again) = call(
        &r,
        "POST",
        "/api/v1/uploads/complete",
        ana,
        Some(json!({"upload_id": id})),
    )
    .await;
    assert_eq!(again, a, "complete is idempotent");
    let (st, _) = put_bytes(&r, &put, ana, &bytes).await;
    assert_eq!(
        st,
        StatusCode::CONFLICT,
        "completed uploads take no more bytes"
    );

    let req = as_user(
        Request::get(format!("/api/v1/artifacts/{}", art.artifact_id)),
        "ana@example.org",
    )
    .body(Body::empty())
    .unwrap();
    let got = axum::body::to_bytes(r.clone().oneshot(req).await.unwrap().into_body(), 1 << 20)
        .await
        .unwrap();
    assert_eq!(got.as_ref(), bytes.as_slice());

    // The declared size is checked against the deployment budget up front (1 MiB in these tests).
    let big = json!({"media_type": "video/mp4", "byte_length": 2u64 << 20,
                     "sha256": Digest::of_bytes(b"")});
    let (st, e) = call(&r, "POST", "/api/v1/uploads/initiate", ana, Some(big)).await;
    assert_eq!(
        (st, e["code"].as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, Some("invalid_request"))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn versions_are_pinned_by_the_host() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), AuthMode::Local, None);
    // The client cannot pin a corpus snapshot (there is no such field) ...
    let (st, e) = call(
        &r,
        "POST",
        "/api/v1/episodes",
        None,
        Some(json!({"goal": "g", "client_request_id": "p1", "pins": {"corpus_snapshot": "mine"}})),
    )
    .await;
    assert_eq!(
        (st, e["code"].as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, Some("invalid_request"))
    );
    // ... and model releases are verified, which needs the ModelService.
    let (st, e) = call(
        &r,
        "POST",
        "/api/v1/episodes",
        None,
        Some(json!({"goal": "g", "client_request_id": "p2", "model_releases": [RELEASE]})),
    )
    .await;
    assert_eq!(
        (st, e["code"].as_str()),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Some("capability_unavailable")
        )
    );
    let (_, ep) = call(
        &r,
        "POST",
        "/api/v1/episodes",
        None,
        Some(json!({"goal": "g", "client_request_id": "p3"})),
    )
    .await;
    assert_eq!(ep["pins"], json!({"model_releases": []}));
    assert_eq!(ep["origin"], "system");
}

#[tokio::test(flavor = "multi_thread")]
async fn fixture_mode_serves_the_c0_contract_fixtures_through_the_same_routes() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), AuthMode::Local, None);
    let base = "/api/fixtures/v1";
    let (st, eps) = call(&r, "GET", &format!("{base}/episodes"), None, None).await;
    assert_eq!(st, StatusCode::OK, "{eps}");
    let ep = &eps.as_array().unwrap()[0];
    assert_eq!(ep["origin"], "synthetic_fixture");
    assert_eq!(
        (ep["status"].as_str(), ep["last_seq"].as_u64()),
        (Some("completed"), Some(15))
    );
    let id = ep["episode_id"].as_str().unwrap().to_string();
    assert!(id.starts_with("fixture-episode-01~"));

    let events = sse(&r, &format!("{base}/episodes/{id}/events"), None, 15).await;
    assert_eq!(events.last().unwrap().1, "episode.status_changed");
    assert_eq!(events[13].1, "answer.completed");

    // The answer artifact of the fixture log is readable; JSON Pointer reads work on it.
    let answer = events[13].2["body"]["payload"]["answer"]["artifact_id"]
        .as_str()
        .unwrap()
        .to_string();
    let (st, v) = call(
        &r,
        "GET",
        &format!("{base}/artifacts/{answer}?pointer=/status"),
        None,
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["value"], "partial");

    let (_, caps) = call(&r, "GET", &format!("{base}/capabilities"), None, None).await;
    assert_eq!(caps["knowledge"]["available"], true);
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let search: Value = serde_json::from_str(
        &std::fs::read_to_string(
            crate_dir.join("../../fixtures/contracts/evidence/search_request.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let (st, page) = call(&r, "POST", &format!("{base}/search"), None, Some(search)).await;
    assert_eq!(st, StatusCode::OK, "{page}");
    serde_json::from_value::<SearchPage>(page).unwrap();
    let (_, models) = call(&r, "GET", &format!("{base}/models"), None, None).await;
    let release_id = models[0]["id"].as_str().unwrap().to_string();

    // Continuing the fixture episode appends after its log; a forecast is a failed job (no forward).
    let (st, m) = call(
        &r,
        "POST",
        &format!("{base}/episodes/{id}/messages"),
        None,
        Some(json!({"client_request_id": "fx-1", "text": "follow-up"})),
    )
    .await;
    assert_eq!((st, m["seq"].as_u64()), (StatusCode::ACCEPTED, Some(16)));
    let mut req = forecast_req("fixture-run", "fx-f");
    req["release_id"] = json!(release_id);
    let (st, tr) = call(&r, "POST", &format!("{base}/analyses"), None, Some(req)).await;
    assert_eq!(st, StatusCode::ACCEPTED, "{tr}");
    let job_id = tr["job"]["job_id"].as_str().unwrap().to_string();
    let mut job = Value::Null;
    for _ in 0..200 {
        job = call(&r, "GET", &format!("{base}/jobs/{job_id}"), None, None)
            .await
            .1;
        if job["status"] == "failed" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(job["status"], "failed");
    assert_eq!(job["error"]["code"], "unsupported");

    // The registry-side /api/v1 is untouched by the fixture.
    let (_, real) = call(&r, "GET", "/api/v1/episodes", None, None).await;
    assert_eq!(real, json!([]));
}

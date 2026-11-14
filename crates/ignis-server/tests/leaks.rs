//! The legacy registry routes no longer leak outcomes of sealed partitions (INTEGRATION.md audit).
//! The fixture registry is mounted as the *registry* origin here, so the lockbox rules apply:
//! fixture-flex-1 is train, fixture-flex-2 validation, fixture-flex-3 test.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_server::v1::io::AuthMode;
use ignis_server::{app, ServeArgs, Services};
use serde_json::{json, Value};
use tower::ServiceExt;

fn router(dir: &Path, internal_routes: bool) -> axum::Router {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let args = ServeArgs {
        port: 0,
        host: "127.0.0.1".into(),
        root: dir.to_path_buf(),
        data: crate_dir.join("fixtures/registry"),
        cohorts: dir.join("no-cohorts"),
        splits: dir.join("no-splits"),
        raw_samples: dir.join("no-raw"),
        artifacts: dir.join("artifacts"),
        evidence_db: dir.join("none.sqlite"),
        compute_threads: 1,
        dev: false,
        fixtures: crate_dir.join("fixtures"),
        web_dist: dir.join("no-dist"),
        state_dir: dir.join("state"),
        auth: AuthMode::Local,
        internal_routes,
        enable_models: true,
        answer_grounding: Default::default(),
        max_upload_bytes: 1 << 20,
        contract_fixtures: "fixtures/contracts".into(),
        codex: Default::default(),
        mcp_port: 0,
        runner: Default::default(),
        artifact_backend: Default::default(),
        r2: Default::default(),
        access: Default::default(),
    };
    app(args, Services::default())
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

#[tokio::test(flavor = "multi_thread")]
async fn sealed_partitions_are_not_served() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), false);

    // Train run: served as before.
    for route in ["observations", "series", "overlay"] {
        let (st, v) = call(
            &r,
            "GET",
            &format!("/api/experiments/fixture-flex-1/{route}?cutoff_s=1e9"),
            None,
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{route}: {v}");
    }
    // Validation and test runs: refused at any cutoff (a large cutoff would be the continuation).
    for run in ["fixture-flex-2", "fixture-flex-3"] {
        for route in ["observations", "series", "overlay"] {
            let (st, v) = call(
                &r,
                "GET",
                &format!("/api/experiments/{run}/{route}?cutoff_s=1e9"),
                None,
            )
            .await;
            assert_eq!(st, StatusCode::FORBIDDEN, "{run}/{route}: {v}");
            assert_eq!(v["refusal"]["kind"], "scope_denied");
        }
    }

    // Summaries keep membership but withhold counts/extent (they reveal the run duration).
    let (_, list) = call(&r, "GET", "/api/experiments", None).await;
    let by_id = |id: &str| {
        list["experiments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .cloned()
            .unwrap()
    };
    assert_eq!(by_id("fixture-flex-1")["outcomes_withheld"], false);
    assert!(by_id("fixture-flex-1")["n_observations"].as_u64().unwrap() > 0);
    for run in ["fixture-flex-2", "fixture-flex-3"] {
        let e = by_id(run);
        assert_eq!(e["outcomes_withheld"], true);
        assert_eq!(e["n_observations"], Value::Null);
        assert_eq!(e["quantities"], json!([]));
    }
    let (_, one) = call(&r, "GET", "/api/experiments/fixture-flex-3", None).await;
    assert_eq!(one["control_history"], Value::Null);

    // Internal routes are not part of the public runtime by default.
    for path in ["/api/seal", "/api/reveal", "/api/admin/reload"] {
        let (st, v) = call(&r, "POST", path, Some(json!({"analysis_id": "sha256:00"}))).await;
        // A typed refusal (so clients can tell "disabled" from "server down").
        assert_eq!(st, StatusCode::OK, "{path}");
        assert_eq!(v["refusal"]["code"], "internal_route_disabled", "{path}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn every_reveal_attempt_is_logged() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), true);
    let id = format!("sha256:{}", "ab".repeat(32));
    let (st, v) = call(&r, "POST", "/api/reveal", Some(json!({"analysis_id": id}))).await;
    assert!(st.is_client_error(), "{st} {v}");
    let log = std::fs::read_to_string(dir.path().join("runs/evaluation/reveal_log.jsonl")).unwrap();
    let entries: Vec<Value> = log
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["artifact_id"], json!(id));
    assert_eq!(entries[0]["granted"], false);
    assert!(
        entries[0]["reason"].as_str().unwrap().contains("analysis"),
        "{}",
        entries[0]
    );
}

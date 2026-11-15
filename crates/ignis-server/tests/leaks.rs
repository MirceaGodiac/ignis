//! The legacy registry routes no longer leak outcomes of sealed partitions (INTEGRATION.md audit).
//! The fixture registry is mounted as the *registry* origin here, so the lockbox rules apply:
//! fixture-flex-1 is train, fixture-flex-2 validation, fixture-flex-3 test.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_core::contracts::{
    AnalysisArtifact, ExecutionScope, ForecastRequest, ModelRelease, Partition, ReleaseId,
};
use ignis_core::services::{not_provided, ModelService, ServiceResult};
use ignis_server::v1::io::AuthMode;
use ignis_server::{app, ServeArgs, Services};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tower::ServiceExt;

fn router(dir: &Path, internal_routes: bool) -> axum::Router {
    router_with(dir, internal_routes, Services::default())
}

fn router_with(dir: &Path, internal_routes: bool, services: Services) -> axum::Router {
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
        sources: Default::default(),
        price_list: None,
        artifact_backend: Default::default(),
        r2: Default::default(),
        access: Default::default(),
    };
    app(args, services)
}

/// A served list that opened the test partition of the fixture split (and nothing else).
struct OpenedTest;

impl ModelService for OpenedTest {
    fn releases(&self, _: &ExecutionScope) -> ServiceResult<Vec<ModelRelease>> {
        Ok(vec![])
    }
    fn release(&self, _: &ExecutionScope, _: &ReleaseId) -> ServiceResult<ModelRelease> {
        not_provided("release")
    }
    fn forecast(&self, _: &ExecutionScope, _: &ForecastRequest) -> ServiceResult<AnalysisArtifact> {
        not_provided("forecast")
    }
    fn opened_splits(&self) -> BTreeMap<String, BTreeSet<Partition>> {
        BTreeMap::from([(
            "fixture-split".to_string(),
            BTreeSet::from([Partition::Test]),
        )])
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn partitions_the_served_list_opened_are_served_and_the_rest_stay_sealed() {
    let dir = tempfile::tempdir().unwrap();
    let services = Services {
        research: ignis_server::v1::ResearchServices {
            models: Some(Arc::new(OpenedTest)),
            ..Default::default()
        },
        ..Default::default()
    };
    let r = router_with(dir.path(), false, services);
    // The opened test run: its recorded series and how it ended are served like a train run's.
    for route in ["observations", "series", "overlay"] {
        let (st, v) = call(
            &r,
            "GET",
            &format!("/api/experiments/fixture-flex-3/{route}?cutoff_s=1e9"),
            None,
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{route}: {v}");
    }
    let (_, one) = call(&r, "GET", "/api/experiments/fixture-flex-3", None).await;
    assert_eq!(one["summary"]["outcomes_withheld"], false, "{one}");
    // Validation was not opened: still refused.
    let (st, v) = call(
        &r,
        "GET",
        "/api/experiments/fixture-flex-2/observations?cutoff_s=1e9",
        None,
    )
    .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "{v}");
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
    // NASA's names for a run travel with the summary (names and schemes, no locators).
    for e in list["experiments"].as_array().unwrap() {
        for a in e["aliases"].as_array().unwrap() {
            assert_eq!(a.as_object().unwrap().len(), 2, "{a}");
        }
    }
    for run in ["fixture-flex-2", "fixture-flex-3"] {
        let e = by_id(run);
        assert_eq!(e["outcomes_withheld"], true);
        assert_eq!(e["n_observations"], Value::Null);
        assert_eq!(e["quantities"], json!([]));
    }
    // How a run ended is an outcome: served for a train run, null for validation and test runs.
    let (_, one) = call(&r, "GET", "/api/experiments/fixture-flex-1", None).await;
    assert_eq!(one["endpoint"]["run_id"], "fixture-flex-1", "{one}");
    for run in ["fixture-flex-2", "fixture-flex-3"] {
        let (_, one) = call(&r, "GET", &format!("/api/experiments/{run}"), None).await;
        assert_eq!(one["control_history"], Value::Null, "{run}");
        assert_eq!(one["endpoint"], Value::Null, "{run}");
    }

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

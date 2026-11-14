//! `ignis infer forecast` and `POST /forecast` are the same computation: same artifact id.

use std::path::PathBuf;

use axum::body::Body;
use axum::http::Request;
use ignis_core::analysis::{AnalysisStore, ForecastSettings};
use ignis_core::records::Registry;
use ignis_core::registry::load;
use ignis_lab::infer::forecast_run;
use ignis_server::{app, fixture, ServeArgs, Services};
use serde_json::{json, Value};
use tower::ServiceExt;

const RELEASE: &str = "sha256:db4ebefdf4b70e96202de6c51e1b9f7690005302cc73374ef03644abdd51e6cc";

#[tokio::test(flavor = "multi_thread")]
async fn cli_and_server_produce_the_same_artifact() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ignis-server/fixtures");
    let tmp = std::env::temp_dir().join(format!("ignis-parity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);

    let args = ServeArgs {
        port: 0,
        host: "127.0.0.1".into(),
        root: fixtures.join("../../.."),
        data: tmp.join("no-registry"),
        cohorts: tmp.join("no-cohorts"),
        splits: tmp.join("no-splits"),
        raw_samples: tmp.join("no-raw"),
        artifacts: tmp.join("server"),
        evidence_db: tmp.join("none.sqlite"),
        compute_threads: 2,
        dev: true,
        fixtures: fixtures.clone(),
        web_dist: tmp.join("no-dist"),
        state_dir: tmp.join("state"),
        auth: ignis_server::v1::io::AuthMode::Local,
        internal_routes: false,
        max_upload_bytes: 1 << 20,
        contract_fixtures: "fixtures/contracts".into(),
        codex: Default::default(),
        mcp_port: 0,
        runner: Default::default(),
    };
    let router = app(
        args,
        Services {
            experts: vec![],
            fixture_experts: fixture::experts(),
            acquisition: None,
            research: Default::default(),
        },
    );
    let settings = json!({"seed": 0, "n_points": 40});
    let req = Request::post("/api/fixtures/forecast")
        .header("content-type", "application/json")
        .body(Body::from(json!({"run_id": "fixture-flex-3", "cutoff_s": 2.0, "model_release_id": RELEASE, "settings": settings, "wait_ms": 10000}).to_string()))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1 << 24)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["outcome"], "artifact", "{body}");

    let registry = Registry::load(&fixtures.join("registry"));
    let archive = registry.archive("fixture-flex-3").unwrap();
    let loaded = load(&fixtures.join("store"), RELEASE).unwrap();
    let parsed: ForecastSettings = serde_json::from_value(settings).unwrap();
    let store = AnalysisStore::new(tmp.join("cli"));
    let stored = forecast_run(
        &fixture::experts(),
        &loaded,
        &archive,
        2.0,
        &parsed,
        &store,
        true,
    )
    .unwrap();

    assert_eq!(stored.artifact.id, body["artifact"]["id"].as_str().unwrap());
    assert_eq!(
        stored.artifact.prefix_hash,
        body["artifact"]["prefix_hash"].as_str().unwrap()
    );
    assert_eq!(
        stored.artifact.settings_hash,
        body["artifact"]["settings_hash"].as_str().unwrap()
    );
    assert!(store
        .dir(&stored.artifact.id)
        .unwrap()
        .join("artifact.json")
        .exists());
    let _ = std::fs::remove_dir_all(&tmp);
}

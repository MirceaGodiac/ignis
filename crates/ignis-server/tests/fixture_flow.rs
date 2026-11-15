//! End to end through the router with the in-process synthetic expert (`/api/fixtures`).

use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_core::contracts::AnalysisArtifact;
use ignis_core::hashing::content_hash;
use ignis_server::{app, fixture, ServeArgs, Services};
use serde_json::{json, Value};
use tower::ServiceExt;

const RELEASE: &str = "sha256:db4ebefdf4b70e96202de6c51e1b9f7690005302cc73374ef03644abdd51e6cc";

fn router(tag: &str) -> (axum::Router, PathBuf) {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let artifacts =
        std::env::temp_dir().join(format!("ignis-server-test-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&artifacts);
    let args = ServeArgs {
        port: 0,
        host: "127.0.0.1".into(),
        root: crate_dir.join("../.."),
        data: artifacts.join("no-registry"),
        cohorts: artifacts.join("no-cohorts"),
        splits: artifacts.join("no-splits"),
        raw_samples: artifacts.join("no-raw"),
        artifacts: artifacts.clone(),
        evidence_db: artifacts.join("none.sqlite"),
        compute_threads: 2,
        dev: true,
        fixtures: crate_dir.join("fixtures"),
        web_dist: artifacts.join("no-dist"),
        state_dir: artifacts.join("state"),
        auth: ignis_server::v1::io::AuthMode::Local,
        internal_routes: false,
        enable_models: true,
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
    let services = Services {
        experts: Vec::new(),
        fixture_experts: fixture::experts(),
        acquisition: None,
        research: Default::default(),
    };
    (app(args, services), artifacts)
}

async fn post(r: &axum::Router, path: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::post(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = r.clone().oneshot(req).await.unwrap();
    let st = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 24)
        .await
        .unwrap();
    (st, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test(flavor = "multi_thread")]
async fn forecast_seal_reveal_round_trip() {
    let (r, store) = router("flow");
    let req = json!({"run_id": "fixture-flex-3", "cutoff_s": 2.0, "model_release_id": RELEASE, "wait_ms": 10000});
    let (st, v) = post(&r, "/api/fixtures/forecast", req.clone()).await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["outcome"], "artifact", "{v}");
    assert_eq!(v["refused_by_expert"], false, "{v}");
    assert_eq!(v["cached"], false);
    let art: AnalysisArtifact = serde_json::from_value(v["artifact"].clone()).unwrap();
    assert_eq!(serde_json::to_value(art.status).unwrap(), "fit_only");
    assert_eq!(art.quantiles.len(), 5);
    assert!(art.times_s.len() == 60 && art.times_s[0] == 2.0);
    assert!(art.support.reasons.iter().any(|x| x.contains("synthetic")));
    assert_eq!(
        art.id,
        content_hash(&art, &["id", "created_at"]),
        "{}",
        serde_json::to_string(&v["artifact"]).unwrap()
    );
    assert_eq!(
        v["context"]["engine"],
        "in-process expert fixture.synthetic_d2"
    );
    // Lower quantile below upper quantile everywhere.
    assert!(art.quantiles[0]
        .iter()
        .zip(&art.quantiles[4])
        .all(|(a, b)| a <= b));

    // Content-addressed cache.
    let (_, v2) = post(&r, "/api/fixtures/forecast", req).await;
    assert_eq!(v2["cached"], true);
    assert_eq!(v2["artifact"]["id"], v["artifact"]["id"]);

    // Seal: hash is the Rust content hash of the stored artifact, and stable across calls / reloads.
    let id = art.id.clone();
    let (st, s1) = post(&r, "/api/fixtures/seal", json!({"analysis_id": id})).await;
    assert_eq!(st, StatusCode::OK, "{s1}");
    assert_eq!(s1["already_sealed"], false);
    assert_eq!(s1["sealed"]["split_partition"], "test");
    let stored: AnalysisArtifact = serde_json::from_str(
        &std::fs::read_to_string(
            store
                .join("dev-fixture-runs/analysis")
                .join(&id[7..])
                .join("artifact.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(s1["sealed"]["artifact_hash"], content_hash(&stored, &[]));
    assert_eq!(content_hash(&stored, &[]), content_hash(&art, &[]));
    let (_, s2) = post(&r, "/api/fixtures/seal", json!({"analysis_id": id})).await;
    assert_eq!(s2["already_sealed"], true);
    assert_eq!(s2["sealed"]["artifact_hash"], s1["sealed"]["artifact_hash"]);

    let (st, rv) = post(&r, "/api/fixtures/reveal", json!({"analysis_id": id})).await;
    assert_eq!(st, StatusCode::OK, "{rv}");
    assert!(!rv["continuation"].as_array().unwrap().is_empty());
    assert!(rv["continuation"]
        .as_array()
        .unwrap()
        .iter()
        .all(
            |o| o["time_s"].as_f64().unwrap() > 2.0 || o["available_at_s"].as_f64().unwrap() > 2.0
        ));

    // Acquisition without an injected scorer: typed `unsupported` inside `scoring` (or no candidates).
    let (st, acq) = post(&r, "/api/fixtures/acquisition", json!({"analysis_id": id})).await;
    assert_eq!(st, StatusCode::OK, "{acq}");
    let status = acq["scoring"]["status"].as_str().unwrap();
    assert!(
        status == "no_candidates" || acq["scoring"]["refusal"]["code"] == "decision_unavailable",
        "{acq}"
    );
    let _ = std::fs::remove_dir_all(&store);
}

#[tokio::test(flavor = "multi_thread")]
async fn acquisition_scoring_never_reads_hidden_values() {
    use std::sync::Arc;

    use ignis_core::contracts::{DecisionPolicy, DecisionTrace};

    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let artifacts =
        std::env::temp_dir().join(format!("ignis-server-test-acq-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&artifacts);
    let args = ServeArgs {
        port: 0,
        host: "127.0.0.1".into(),
        root: crate_dir.join("../.."),
        data: artifacts.join("no-registry"),
        cohorts: artifacts.join("no-cohorts"),
        splits: artifacts.join("no-splits"),
        raw_samples: artifacts.join("no-raw"),
        artifacts: artifacts.clone(),
        evidence_db: artifacts.join("none.sqlite"),
        compute_threads: 2,
        dev: true,
        fixtures: crate_dir.join("fixtures"),
        web_dist: artifacts.join("no-dist"),
        state_dir: artifacts.join("state"),
        auth: ignis_server::v1::io::AuthMode::Local,
        internal_routes: false,
        enable_models: true,
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
    // A scorer that records what it was given.
    let seen: Arc<std::sync::Mutex<Option<(Vec<String>, Vec<Value>)>>> = Default::default();
    let log = seen.clone();
    let scorer: ignis_server::AcquisitionScorer =
        Arc::new(move |rel, prefix, actions, _settings| {
            *log.lock().unwrap() = Some((
                prefix.observations.iter().map(|o| o.id.clone()).collect(),
                actions
                    .iter()
                    .map(|a| serde_json::to_value(a).unwrap())
                    .collect(),
            ));
            Ok(DecisionTrace {
                id: "sha256:0".into(),
                prefix_hash: prefix.input_hash.clone(),
                model_release_id: rel.release.id.clone(),
                target: "t".into(),
                loss: "l".into(),
                cost_weight: 0.0,
                mc_settings: Default::default(),
                candidates: vec![],
                chosen_action_id: None,
                policy: DecisionPolicy::Voi,
                updated_artifact_id: None,
                realized_score: None,
            })
        });
    let r = app(
        args,
        Services {
            experts: Vec::new(),
            fixture_experts: fixture::experts(),
            acquisition: Some(scorer),
            research: Default::default(),
        },
    );
    // fixture-flex-3-d031 was taken at 2.41 s and becomes available at 2.59 s: pending at 2.5 s.
    let (st, v) = post(&r, "/api/fixtures/forecast", json!({"run_id": "fixture-flex-3", "cutoff_s": 2.5, "model_release_id": RELEASE, "wait_ms": 10000})).await;
    assert_eq!(st, StatusCode::OK, "{v}");
    let (st, acq) = post(
        &r,
        "/api/fixtures/acquisition",
        json!({"analysis_id": v["artifact"]["id"]}),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{acq}");
    assert_eq!(acq["scoring"]["status"], "scored", "{acq}");

    let (prefix_ids, actions) = seen.lock().unwrap().clone().unwrap();
    assert!(actions
        .iter()
        .any(|a| a["observation_id"] == "fixture-flex-3-d031"));
    assert!(
        !prefix_ids.iter().any(|id| id == "fixture-flex-3-d031"),
        "a pending observation entered the prefix"
    );
    assert!(
        actions
            .iter()
            .all(|a| a.get("value_si").is_none() && a.get("raw_value").is_none()),
        "an action carries a value"
    );
    // The response itself carries no value of any candidate either.
    assert!(!acq.to_string().contains("value_si"), "{acq}");
    let _ = std::fs::remove_dir_all(&artifacts);
}

#[tokio::test(flavor = "multi_thread")]
async fn release_family_mismatch_is_a_typed_refusal() {
    let (r, store) = router("family");
    let (st, v) = post(
        &r,
        "/api/fixtures/forecast",
        json!({"run_id": "fixture-bass-1", "cutoff_s": 3.0, "model_release_id": RELEASE}),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["outcome"], "refusal");
    assert_eq!(v["refusal"]["kind"], "unsupported");
    assert_eq!(v["refusal"]["code"], "family_mismatch");

    // The registry origin has no experts and never sees the synthetic one.
    let (_, h) = {
        let resp = r
            .clone()
            .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let b = axum::body::to_bytes(resp.into_body(), 1 << 20)
            .await
            .unwrap();
        ((), serde_json::from_slice::<Value>(&b).unwrap())
    };
    assert_eq!(h["experts"], json!([]));
    let _ = std::fs::remove_dir_all(&store);
}

#[tokio::test(flavor = "multi_thread")]
async fn insufficient_prefix_is_refused_by_the_expert() {
    let (r, store) = router("early");
    let (st, v) = post(&r, "/api/fixtures/forecast",
        json!({"run_id": "fixture-flex-1", "cutoff_s": 0.05, "model_release_id": RELEASE, "wait_ms": 10000})).await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["refused_by_expert"], true, "{v}");
    assert_eq!(v["artifact"]["status"], "insufficient_inference");
    let _ = std::fs::remove_dir_all(&store);
}

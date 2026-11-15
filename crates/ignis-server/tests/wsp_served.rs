//! The served hand-written releases (`configs/models/served`) through the real stack: HTTP routes
//! and the MCP tool host -> `ignis_wsp_serve::ServedModels` -> the strict forwards of `ignis-wsp`
//! -> job runner -> stored `AnalysisArtifact`. Inputs are typed scenarios and synthetic stored-run
//! prefixes; nothing here reads NASA records or scores anything.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_core::contracts::*;
use ignis_core::hashing::content_hash;
use ignis_core::services::ModelService;
use ignis_mcp::{ToolHost, ToolServices};
use ignis_research::{LocalArtifactStore, ResearchStore};
use ignis_server::v1::{self, io::research_scope, io::AuthMode, V1};
use ignis_wsp::{MemoryPrefixes, Provenance};
use ignis_wsp_serve::ServedModels;
use serde_json::{json, Value};
use tower::ServiceExt;

const FLEX_METHANOL: &str =
    "sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654";
const ACME_NORMAL: &str = "sha256:a9986fb93275fe264d76ee1b632f59898e51052ba4bfdb89980bf51a0a3b6177";
const ACME_INVERSE: &str =
    "sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7";
const BASS_FRONT: &str = "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c";
const FLEX_HEPTANE: &str =
    "sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef";

fn served_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/models/served")
}

/// Synthetic stored runs: the methanol fixture as a train run, and the same bytes as a test run.
fn stored() -> MemoryPrefixes {
    let mut train = ignis_wsp::models::by_name("flex_methanol.wsp2")
        .unwrap()
        .fixture_prefix();
    train.run_id = "synthetic-train-run".into();
    train.input_hash = content_hash(&train, &["input_hash"]);
    let mut test = train.clone();
    test.run_id = "synthetic-test-run".into();
    test.input_hash = content_hash(&test, &["input_hash"]);
    MemoryPrefixes::new()
        .with_run(train, Provenance::Partition(Partition::Train))
        .with_run(test, Provenance::Partition(Partition::Test))
}

struct Harness {
    _dir: tempfile::TempDir,
    state: Arc<V1>,
    router: axum::Router,
    scope: ExecutionScope,
}

fn harness() -> Harness {
    let dir = tempfile::tempdir().unwrap();
    let models: Arc<dyn ModelService> =
        Arc::new(ServedModels::open(served_dir(), stored()).unwrap());
    let services = ToolServices {
        models: Some(models),
        ..Default::default()
    };
    let store = Arc::new(ResearchStore::open(dir.path().join("state.sqlite")).unwrap());
    let artifacts = Arc::new(LocalArtifactStore::open(dir.path().join("artifacts")).unwrap());
    let runner_wake = Arc::new(tokio::sync::Notify::new());
    let hub = Arc::new(v1::Hub::default());
    let (wake, notify) = (runner_wake.clone(), hub.clone());
    let tools = Arc::new(
        ToolHost::new(
            services.clone(),
            store.clone(),
            artifacts.clone(),
            Origin::System,
            Arc::new(move || wake.notify_one()),
        )
        .with_commit_hook(Arc::new(move |ep, seq| notify.notify(ep, seq))),
    );
    let state = Arc::new(V1 {
        store,
        artifacts,
        services,
        tools,
        hub,
        auth: AuthMode::Local,
        access: None,
        runner_wake,
        origin: Origin::System,
        uploads_dir: dir.path().join("uploads"),
        max_upload_bytes: 1 << 20,
        fixture: None,
        runtime: None,
        runtime_report: v1::agent::RuntimeReport {
            available: false,
            detail: "offline test".into(),
            manifest: None,
        },
        runner_report: v1::sandbox::RunnerReport {
            available: false,
            detail: "forecasts need no analysis sandbox".into(),
            environment: None,
        },
        mcp_url: None,
        artifact_backend: Default::default(),
        sources: None,
    });
    // `build` also starts the job runner that executes forecasts.
    let router = v1::build(state.clone());
    Harness {
        _dir: dir,
        state,
        router,
        scope: research_scope("local", false),
    }
}

async fn http(h: &Harness, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(body.map_or_else(Body::empty, |v| Body::from(v.to_string())))
        .unwrap();
    let response = h.router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn finished(h: &Harness, job: &str) -> JobRef {
    tokio::time::timeout(Duration::from_secs(120), async {
        loop {
            let (_, j) = http(h, "GET", &format!("/jobs/{job}"), None).await;
            let job: JobRef = serde_json::from_value(j).unwrap();
            if job.status.is_terminal() {
                return job;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("forecast job finished")
}

/// `/analyses` -> job -> the stored artifact, read back over HTTP.
async fn forecast_http(h: &Harness, release: &str, input: Value, key: &str) -> AnalysisArtifact {
    let (st, tr) = http(
        h,
        "POST",
        "/analyses",
        Some(
            json!({"schema_version": "wsp.v1", "release_id": release, "input": input,
                    "client_request_id": key}),
        ),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{tr}");
    let job = finished(h, tr["job"]["job_id"].as_str().unwrap()).await;
    assert_eq!(job.status, JobStatus::Succeeded, "{job:#?}");
    let out = &job.outputs[0];
    assert_eq!(out.contract.as_deref(), Some("AnalysisArtifact"));
    assert_eq!(
        out.produced_by.release_id.as_ref().unwrap().as_str(),
        release
    );
    let (st, v) = http(h, "GET", &format!("/artifacts/{}", out.artifact_id), None).await;
    assert_eq!(st, StatusCode::OK);
    let a: AnalysisArtifact = serde_json::from_value(v).unwrap();
    a.validate().unwrap();
    assert_eq!(a.evidence_class, EvidenceClass::ModelOutput);
    assert_eq!(a.model_release_id, release);
    a
}

async fn admit(h: &Harness, release: &str, body: Value) -> (StatusCode, Value) {
    let mut body = body;
    body["schema_version"] = json!("wsp.v1");
    body["release_id"] = json!(release);
    http(
        h,
        "POST",
        &format!("/models/{release}/scenarios"),
        Some(body),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn served_releases_and_cards_are_listed_honestly() {
    let h = harness();
    let (st, list) = http(&h, "GET", "/models", None).await;
    assert_eq!(st, StatusCode::OK);
    let releases: Vec<ModelRelease> = serde_json::from_value(list).unwrap();
    let ids: Vec<&str> = releases.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(
        ids[..5],
        [
            FLEX_METHANOL,
            FLEX_HEPTANE,
            ACME_NORMAL,
            ACME_INVERSE,
            BASS_FRONT
        ]
    );
    assert_eq!(ids.len(), 14, "the final release of every investigation");
    let (st, cards) = http(&h, "GET", "/model-cards", None).await;
    assert_eq!(st, StatusCode::OK, "{cards}");
    let cards: Vec<ModelCard> = serde_json::from_value(cards).unwrap();
    assert_eq!(cards.len(), 14);
    for (c, r) in cards.iter().zip(&releases) {
        assert_eq!(c.release_id.as_str(), r.id);
        assert_eq!(c.trained_on, TrainedOn::NotTrained);
        assert_eq!(c.state, QualificationState::Experimental);
        assert!(!c.scores.is_empty() && c.scores.iter().all(|s| !s.source.is_empty()));
        assert!(!c.limits.is_empty() && !c.inputs.is_empty());
    }
    // A release that is not served is not found, even though it would be a valid id.
    let (st, _) = http(
        &h,
        "GET",
        "/models/sha256:0000000000000000000000000000000000000000000000000000000000000000",
        None,
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn typed_scenarios_run_the_strict_forward_of_each_family_type() {
    let h = harness();
    // Droplet forward (droplet3.rs): conditions and an early diameter prefix.
    let diameters: Vec<Value> = (0..=30)
        .map(|i| {
            let t = i as f64 * 0.1;
            json!({"time_s": t, "value_si": (9.0e-6 - 0.55e-6 * t).sqrt()})
        })
        .collect();
    let (st, a) = admit(
        &h,
        FLEX_METHANOL,
        json!({"cutoff_s": 3.0, "phase_established_at_s": 0.0, "observations": diameters,
               "conditions": {"fuel": "Methanol", "pressure": 101325.0, "x_o2": 0.21, "x_co2": 0.0, "x_he": 0.0}}),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{a}");
    assert_eq!(a["evidence_class"], "synthetic");
    let hash = a["input_hash"].as_str().unwrap();
    let art = forecast_http(
        &h,
        FLEX_METHANOL,
        json!({"kind": "prefix", "input_hash": hash}),
        "flex",
    )
    .await;
    assert!(art.support.admitted, "{:?}", art.support);
    assert!(!art.quantiles.is_empty());
    assert_eq!(art.prefix_hash, hash);

    // Committee forward (committee3.rs): conditions only, both burner families.
    for (release, flow, tad, xf, xo, zst) in [
        (ACME_NORMAL, 1e-6, 2300.0, 0.3, 0.3, 0.25),
        (ACME_INVERSE, 5e-6, 2400.0, 0.1, 0.5, 0.45),
    ] {
        let (st, a) = admit(
            &h,
            release,
            json!({"cutoff_s": 3.0, "phase_established_at_s": 0.0, "conditions": {
                "fuel_species": "propane", "injected_mass_flow": flow, "pressure": 101325.0,
                "tad": tad, "x_fuel_role": xf, "x_o2_role": xo, "zst": zst}}),
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{a}");
        let hash = a["input_hash"].as_str().unwrap();
        let art = forecast_http(
            &h,
            release,
            json!({"kind": "prefix", "input_hash": hash}),
            release,
        )
        .await;
        assert!(art.support.admitted, "{:?}", art.support);
        assert_eq!(art.kind, ArtifactKind::EventProbabilities);
        assert!(!art.event_cdf.is_empty());
    }

    // Front forward (front.rs): conditions, fan history and tip positions.
    let tips: Vec<Value> = (0..10)
        .map(|i| json!({"time_s": 11.0 + i as f64, "value_si": 0.002 * (i as f64 + 1.0)}))
        .collect();
    let (st, a) = admit(
        &h,
        BASS_FRONT,
        json!({"cutoff_s": 20.0, "phase_established_at_s": 10.0, "observations": tips,
               "conditions": {"x_o2": 0.19, "width": 0.022, "length": 0.1},
               "controls": [{"control": "igniter", "t_start_s": 4.0, "t_end_s": 10.0},
                            {"control": "flow_velocity", "t_start_s": 0.0, "t_end_s": 22.0, "value_si": 0.1},
                            {"control": "flow_velocity", "t_start_s": 22.0, "value_si": 0.03}]}),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{a}");
    let hash = a["input_hash"].as_str().unwrap();
    let art = forecast_http(
        &h,
        BASS_FRONT,
        json!({"kind": "prefix", "input_hash": hash}),
        "bass",
    )
    .await;
    assert!(art.support.admitted, "{:?}", art.support);

    // Out of domain: a refusal artifact with its reason, never a number.
    let (_, a) = admit(
        &h,
        ACME_NORMAL,
        json!({"cutoff_s": 3.0, "phase_established_at_s": 0.0, "conditions": {
            "fuel_species": "propane", "injected_mass_flow": 1e-6, "pressure": 101325.0,
            "tad": 9000.0, "x_fuel_role": 0.3, "x_o2_role": 0.3, "zst": 0.25}}),
    )
    .await;
    let hash = a["input_hash"].as_str().unwrap();
    let art = forecast_http(
        &h,
        ACME_NORMAL,
        json!({"kind": "prefix", "input_hash": hash}),
        "ood",
    )
    .await;
    assert!(!art.support.admitted);
    assert!(!art.support.reasons.is_empty());
    assert!(art.event_cdf.is_empty() && art.quantiles.is_empty());

    // Generic law forward (law.rs): typed features in, each declared target out.
    let (_, cards) = http(&h, "GET", "/model-cards", None).await;
    let cards: Vec<ModelCard> = serde_json::from_value(cards).unwrap();
    for card in cards.iter().filter(|c| c.family == Family::Other) {
        let i = ignis_wsp_serve::law_index(&card.model_name).unwrap();
        let fixture = ignis_wsp::law_tools::fixture_prefix(i);
        let conditions: serde_json::Map<String, Value> = fixture
            .known_conditions
            .iter()
            .map(|(k, m)| (k.clone(), json!(m.value_si.unwrap())))
            .collect();
        let (st, a) = admit(
            &h,
            card.release_id.as_str(),
            json!({"cutoff_s": 0.0, "conditions": conditions}),
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{}: {a}", card.model_name);
        let hash = a["input_hash"].as_str().unwrap().to_string();
        let list = ignis_wsp_serve::ServedList::read(&served_dir().join("served.json")).unwrap();
        let served = list
            .releases
            .iter()
            .find(|e| e.release_id == card.release_id)
            .and_then(|e| e.targets.clone());
        for t in &ignis_wsp::law::decl(i).targets {
            if served.as_ref().is_some_and(|s| !s.contains(&t.name)) {
                // A declared target that is not the model is refused, never forecast.
                let (_, tr) = http(
                    &h,
                    "POST",
                    "/analyses",
                    Some(json!({"schema_version": "wsp.v1",
                    "release_id": card.release_id, "input": {"kind": "prefix", "input_hash": hash},
                    "settings": {"target": t.name, "times_s": [1.0, 10.0, 100.0]},
                    "client_request_id": format!("{}-{}-refused", card.model_name, t.name)})),
                )
                .await;
                let job = finished(&h, tr["job"]["job_id"].as_str().unwrap()).await;
                assert_eq!(
                    job.status,
                    JobStatus::Failed,
                    "{} {}",
                    card.model_name,
                    t.name
                );
                continue;
            }
            let (st, tr) = http(
                &h,
                "POST",
                "/analyses",
                Some(json!({"schema_version": "wsp.v1",
                "release_id": card.release_id, "input": {"kind": "prefix", "input_hash": hash},
                "settings": {"target": t.name, "times_s": [1.0, 10.0, 100.0]},
                "client_request_id": format!("{}-{}", card.model_name, t.name)})),
            )
            .await;
            assert_eq!(st, StatusCode::ACCEPTED, "{tr}");
            let job = finished(&h, tr["job"]["job_id"].as_str().unwrap()).await;
            assert_eq!(
                job.status,
                JobStatus::Succeeded,
                "{} {}: {job:#?}",
                card.model_name,
                t.name
            );
        }
        // A stored run is refused for a law release, with the reason.
        let (_, tr) = http(&h, "POST", "/analyses", Some(json!({"schema_version": "wsp.v1",
            "release_id": card.release_id, "input": {"kind": "stored_run", "run_id": "any", "cutoff_s": 1.0},
            "client_request_id": format!("{}-stored", card.model_name)}))).await;
        let job = finished(&h, tr["job"]["job_id"].as_str().unwrap()).await;
        assert_eq!(job.error.unwrap().code, ErrorCode::CapabilityUnavailable);
    }

    // A condition the release does not read is refused at admission.
    let (st, e) = admit(
        &h,
        FLEX_METHANOL,
        json!({"cutoff_s": 1.0, "conditions": {"programmed_stop": 5.0}}),
    )
    .await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY, "{e}");
    assert_eq!(e["code"], "invalid_request");
}

#[tokio::test(flavor = "multi_thread")]
async fn stored_runs_respect_the_partition_and_the_mcp_tools_reach_the_forward() {
    let h = harness();
    // Stored run of the train partition: the strict forward answers.
    let art = forecast_http(
        &h,
        FLEX_METHANOL,
        json!({"kind": "stored_run", "run_id": "synthetic-train-run", "cutoff_s": 3.0}),
        "train",
    )
    .await;
    assert!(art.support.admitted, "{:?}", art.support);

    // MCP: the descriptions name what is served and what each release needs.
    let tools = ignis_mcp::catalog(&h.state.services);
    for name in ["models_inspect", "models_forecast"] {
        let t = tools.iter().find(|t| t.descriptor.name == name).unwrap();
        assert!(t.unavailable.is_none());
        assert!(t.descriptor.summary.contains("acme_normal.wsp13"), "{name}");
        assert!(
            t.descriptor.summary.contains("never substitute a number") || name == "models_inspect"
        );
    }
    let inspect = h.state.tools.call(
        &h.scope,
        "models_inspect",
        json!({"release_id": ACME_INVERSE}),
    );
    assert_eq!(
        inspect.execution_status,
        ExecutionStatus::Completed,
        "{inspect:#?}"
    );
    // A release record is larger than an inline result: the tool stores it and points at it.
    let preview = inspect.result.unwrap().value;
    let stored = preview["stored_as_artifact"].as_str().unwrap().to_string();
    for (pointer, want) in [
        ("/model_name", "acme_inverse.wsp12"),
        ("/trained_on", "not_trained"),
        ("/state", "experimental"),
    ] {
        let read = h.state.tools.call(
            &h.scope,
            "artifacts_read",
            json!({"artifact_id": stored, "json_pointer": pointer}),
        );
        assert_eq!(read.result.unwrap().value, json!(want), "{pointer}");
    }

    let args = json!({"schema_version": "wsp.v1", "release_id": FLEX_METHANOL,
        "input": {"kind": "stored_run", "run_id": "synthetic-train-run", "cutoff_s": 3.0},
        "client_request_id": "mcp-1"});
    let queued = h.state.tools.call(&h.scope, "models_forecast", args);
    assert_eq!(
        queued.execution_status,
        ExecutionStatus::Accepted,
        "{queued:#?}"
    );
    let job = finished(&h, queued.job.unwrap().job_id.as_str()).await;
    assert_eq!(job.status, JobStatus::Succeeded, "{job:#?}");
    let read = h.state.tools.call(
        &h.scope,
        "artifacts_read",
        json!({"artifact_id": job.outputs[0].artifact_id, "json_pointer": "/evidence_class"}),
    );
    assert_eq!(
        read.execution_status,
        ExecutionStatus::Completed,
        "{read:#?}"
    );
    assert_eq!(read.result.unwrap().value, json!("model_output"));

    // A test-partition run is refused before its prefix is built: a failed job, no artifact.
    let args = json!({"schema_version": "wsp.v1", "release_id": FLEX_METHANOL,
        "input": {"kind": "stored_run", "run_id": "synthetic-test-run", "cutoff_s": 3.0},
        "client_request_id": "mcp-test"});
    let queued = h.state.tools.call(&h.scope, "models_forecast", args);
    let job = finished(&h, queued.job.unwrap().job_id.as_str()).await;
    assert_eq!(job.status, JobStatus::Failed);
    assert_eq!(job.error.unwrap().code, ErrorCode::ScopeDenied);
    assert!(job.outputs.is_empty());

    // An agent cannot forecast an input hash nobody admitted.
    let args = json!({"schema_version": "wsp.v1", "release_id": FLEX_METHANOL,
        "input": {"kind": "prefix", "input_hash": "sha256:1111111111111111111111111111111111111111111111111111111111111111"},
        "client_request_id": "mcp-unknown"});
    let queued = h.state.tools.call(&h.scope, "models_forecast", args);
    let job = finished(&h, queued.job.unwrap().job_id.as_str()).await;
    assert_eq!(job.status, JobStatus::Failed);
    assert_eq!(job.error.unwrap().code, ErrorCode::NotFound);
}

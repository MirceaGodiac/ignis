//! M11: `/api/v1` runs, datasets and conditions, HTTP route → `DatasetService` (`ignis_data`'s
//! `DatasetStore`) → files, on synthetic bundles and a synthetic conditions table laid over a real
//! frozen FLEX split. The HTTP caller's scope is host-derived (research: train only), so the test
//! partition, the validation partition and the evaluator root never reach it: bundles holding them
//! are not listed and are refused by id, their view bytes are refused, and a run of a held-out
//! partition has no conditions row, no envelope contribution and no `GET /runs/{id}`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_core::contracts::{
    Cohort, ConditionsTable, DatasetId, Digest, EndpointRecord, ExperimentRecord, Family,
    Observation, Partition, Split,
};
use ignis_core::services::DatasetService;
use ignis_data::common::Bundle;
use ignis_data::dataset::{self, BuiltBundle, BundleSpec, DatasetStore};
use ignis_server::v1::io::AuthMode;
use ignis_server::v1::ResearchServices;
use ignis_server::{app, fixture, ServeArgs, Services};
use serde_json::{json, Value};
use tower::ServiceExt;

mod support;

/// A real frozen split (committed); its runs are the ids the synthetic conditions rows use.
const SPLIT: &str = "flex-660b7d9ede629b30.json";
const TRAIN_RUN: &str = "PSI-69:053F001:2010-05-05";
const TEST_RUN: &str = "PSI-69:190F003:2010-01-29";
/// Pressure of the held-out row; must never appear in an answer to the research scope.
const SENTINEL_PA: f64 = 77777.125;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Serializes the tests that read (or rewrite) the shared `IGNIS_DATA` files.
static DATA: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// `IGNIS_DATA` for this test binary: the real split plus a synthetic conditions table whose two
/// rows are a train run and a test run of that split. Set once, before any request.
fn data_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = tempfile::tempdir().unwrap().keep().join("data");
        std::fs::create_dir_all(root.join("splits")).unwrap();
        std::fs::copy(
            repo().join("data/splits").join(SPLIT),
            root.join("splits").join(SPLIT),
        )
        .unwrap();
        let mut t: Value = serde_json::from_str(
            &std::fs::read_to_string(repo().join("fixtures/contracts/data/conditions_table.json"))
                .unwrap(),
        )
        .unwrap();
        for (row, run) in t["rows"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .zip([TRAIN_RUN, TEST_RUN])
        {
            row["investigation_id"] = json!("PSI-69");
            row["run"]["run_id"] = json!(run);
        }
        for e in t["envelopes"].as_array_mut().unwrap() {
            e["investigation_id"] = json!("PSI-69");
        }
        // The held-out row gets a measured pressure of its own (a copy of the train row's cell).
        let mut pressure = t["rows"][0]["cells"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["key"] == "pressure")
            .unwrap()
            .clone();
        pressure["value"]["value_si"] = json!(SENTINEL_PA);
        for c in t["rows"][1]["cells"].as_array_mut().unwrap() {
            if c["key"] == "pressure" {
                *c = pressure.clone();
            }
        }
        let table: ConditionsTable = serde_json::from_value(t).unwrap();
        table.validate().unwrap();
        let dir = root.join("work/conditions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("conditions_table.json"),
            serde_json::to_string_pretty(&table).unwrap(),
        )
        .unwrap();
        std::env::set_var("IGNIS_DATA", &root);
        root
    })
}

// ------------------------------------------------------------------ synthetic bundles

const RUNS: [(&str, &str); 3] = [
    ("PSI-69:R1", "g1"),
    ("PSI-69:R2", "g2"),
    ("PSI-69:R3", "g3"),
];

fn loc(row: i64) -> Value {
    json!({"asset_sha256": "ab".repeat(32), "path": "synthetic-droplet-table.txt",
           "row": row, "column": "DropletDiameter_mm", "page": null, "frame": null, "pts_s": null,
           "char_start": null, "char_end": null})
}

fn det(id: &str) -> Value {
    json!({"method": "deterministic", "method_id": id, "method_version": "1", "inputs": [], "agent": null, "notes": null})
}

fn records() -> (Bundle, Cohort, Split) {
    let mut b = Bundle::new("flex");
    for (k, (r, _)) in RUNS.iter().enumerate() {
        let rec: ExperimentRecord = serde_json::from_value(json!({
            "id": r, "investigation_id": "PSI-69", "family": "flex", "session_id": null, "specimen_id": null,
            "aliases": [], "fuel_or_material": "Heptane", "configuration": "droplet", "gravity": null,
            "conditions": {
                "pressure": {"raw_value": 760.0 + k as f64, "raw_unit": "mmHg",
                    "value_si": (760.0 + k as f64) * 133.322387415, "unit_si": "Pa", "uncertainty_si": null,
                    "uncertainty_reason": "not reported by source", "evidence_class": "original_observation",
                    "locator": loc(2), "derivation": det("flex.cond.pressure"), "resolution": "resolved", "flags": []}
            },
            "identity_resolution": "resolved", "identity_evidence": [], "quality_flags": [], "notes": null
        }))
        .unwrap();
        b.records.push(rec);
        let ep: EndpointRecord = serde_json::from_value(json!({
            "run_id": r, "original_label": "Extinction", "original_comment": null, "allowed_causes": ["extinction"],
            "censoring": "interval", "last_confirmed_active_s": 2.0, "first_terminal_s": 2.1, "intervention_s": null,
            "intervention_kind": null, "observation_state": null, "resolution": "resolved", "evidence": [loc(2)],
            "derivation": det("flex.endpoint.draft")
        }))
        .unwrap();
        b.endpoints.push(ep);
        for (i, t) in [0.0, 1.0].into_iter().enumerate() {
            let o: Observation = serde_json::from_value(json!({
                "id": format!("{r}/DropletData/DropletDiameter_mm/{i}"), "run_id": r, "quantity": "liquid_diameter",
                "channel": "DropletData", "time_s": t, "available_at_s": t,
                "raw_value": 2.0 - t, "raw_unit": "mm", "value_si": (2.0 - t) / 1000.0, "unit_si": "m",
                "uncertainty_si": null, "calibration_id": null, "quality_flags": [],
                "evidence_class": "source_derived", "locator": loc(i as i64 + 2),
                "derivation": det("flex.obs.DropletDiameter_mm")
            }))
            .unwrap();
            b.observations.push(o);
        }
    }
    let cohort = Cohort {
        id: "sha256:cohort".into(),
        family: Family::Flex,
        version: "1".into(),
        criteria: BTreeMap::from([("fuel".to_string(), "heptane".to_string())]),
        run_ids: RUNS.iter().map(|(r, _)| r.to_string()).collect(),
        excluded: vec![],
        source_manifest_hash: "sha256:manifests".into(),
    };
    let split = Split {
        id: "sha256:0123456789abcdef0123".into(),
        cohort_id: cohort.id.clone(),
        group_key: "session".into(),
        rule: "fixture".into(),
        seed: 0,
        run_to_group: RUNS
            .iter()
            .map(|(r, g)| (r.to_string(), g.to_string()))
            .collect(),
        train: vec!["g1".into()],
        validation: vec!["g2".into()],
        test: vec!["g3".into()],
        frozen_at: "2026-11-14T00:00:00Z".into(),
    };
    (b, cohort, split)
}

fn build(spec: &BundleSpec) -> BuiltBundle {
    let (b, c, s) = records();
    dataset::build(&b, &c, &s, spec, &|_| None).unwrap()
}

struct Store {
    train: BuiltBundle,
    dev: BuiltBundle,
    eval: BuiltBundle,
    service: Arc<dyn DatasetService>,
}

fn store(dir: &Path) -> Store {
    let s = DatasetStore {
        builder_root: dir.join("work/datasets"),
        evaluator_root: dir.join("holdout/datasets"),
    };
    let train = build(&BundleSpec::builder(&[Partition::Train]));
    let dev = build(&BundleSpec::builder(&[
        Partition::Train,
        Partition::Validation,
    ]));
    let eval = build(&BundleSpec::evaluator());
    for b in [&train, &dev, &eval] {
        s.save(b).unwrap();
    }
    Store {
        train,
        dev,
        eval,
        service: Arc::new(s),
    }
}

// ------------------------------------------------------------------ server

fn args(dir: &Path) -> ServeArgs {
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
        enable_models: false,
        answer_grounding: Default::default(),
        max_upload_bytes: 1 << 20,
        contract_fixtures: "fixtures/contracts".into(),
        codex: Default::default(),
        mcp_port: 0,
        runner: Default::default(),
        sources: Default::default(),
        artifact_backend: Default::default(),
        r2: Default::default(),
        access: support::access().args.clone(),
    }
}

fn router(dir: &Path, datasets: Option<Arc<dyn DatasetService>>) -> axum::Router {
    app(
        args(dir),
        Services {
            experts: Vec::new(),
            fixture_experts: fixture::experts(),
            acquisition: None,
            research: ResearchServices {
                datasets,
                ..Default::default()
            },
        },
    )
}

async fn raw(
    r: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let req = Request::builder().method(method).uri(path);
    let req = match body {
        Some(b) => req
            .header("content-type", "application/json")
            .body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let resp = r.clone().oneshot(req).await.unwrap();
    let (st, h) = (resp.status(), resp.headers().clone());
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 26)
        .await
        .unwrap();
    (st, h, bytes.to_vec())
}

async fn call(
    r: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let (st, _, b) = raw(r, method, path, body).await;
    (st, serde_json::from_slice(&b).unwrap_or(Value::Null))
}

fn enc(s: &str) -> String {
    s.replace(':', "%3A")
}

// ------------------------------------------------------------------ tests

#[tokio::test(flavor = "multi_thread")]
async fn datasets_are_paginated_metadata_and_views_stream_as_parquet() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(dir.path());
    let r = router(dir.path(), Some(s.service.clone()));
    let train_id = s.train.bundle.dataset_id.to_string();

    let (st, page) = call(&r, "GET", "/api/v1/datasets", None).await;
    assert_eq!(st, StatusCode::OK, "{page}");
    let ids: Vec<&str> = page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["bundle"]["dataset_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [train_id.as_str()],
        "only what the research scope may read"
    );
    assert_eq!(page["total_count"], 1);
    assert!(page.get("next_cursor").is_none());

    // pagination: page_size 1 over a store with two readable bundles (a second train bundle)
    let (st, page) = call(&r, "GET", "/api/v1/datasets?page_size=0", None).await;
    assert_eq!(
        (st, page["code"].as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, Some("invalid_request"))
    );
    let (st, page) = call(&r, "GET", "/api/v1/datasets?cursor=bogus", None).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY, "{page}");

    let (st, b) = call(&r, "GET", &format!("/api/v1/datasets/{train_id}"), None).await;
    assert_eq!(st, StatusCode::OK, "{b}");
    assert_eq!(b, serde_json::to_value(&s.train.bundle).unwrap());
    let view = b["views"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "obs_liquid_diameter")
        .unwrap()
        .clone();
    assert_eq!(view["artifact"]["media_type"], dataset::PARQUET);

    let path = format!("/api/v1/datasets/{train_id}/views/obs_liquid_diameter");
    let (st, h, bytes) = raw(&r, "GET", &path, None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(h["content-type"], dataset::PARQUET);
    assert_eq!(
        bytes.len() as u64,
        view["artifact"]["byte_length"].as_u64().unwrap()
    );
    assert_eq!(
        h["content-length"].to_str().unwrap(),
        bytes.len().to_string()
    );
    assert_eq!(&bytes[..4], b"PAR1");
    assert_eq!(&bytes[bytes.len() - 4..], b"PAR1");
    let digest = Digest::of_bytes(&bytes);
    assert_eq!(
        digest.as_str(),
        view["artifact"]["sha256"].as_str().unwrap()
    );
    assert_eq!(
        h["x-ignis-artifact-sha256"].to_str().unwrap(),
        digest.as_str()
    );

    // a range of the same bytes
    let (st, h, part) = raw(&r, "GET", &format!("{path}?offset=4&len=10"), None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(part, bytes[4..14]);
    assert_eq!(h["content-length"], "10");

    let (st, e) = call(
        &r,
        "GET",
        &format!("/api/v1/datasets/{train_id}/views/nope"),
        None,
    )
    .await;
    assert_eq!(
        (st, e["code"].as_str()),
        (StatusCode::NOT_FOUND, Some("not_found"))
    );

    // a second readable bundle: two pages of one
    let extra = dir
        .path()
        .join("work/datasets")
        .join("flex-0123456789abcdef-copy.json");
    let mut copy = s.train.bundle.clone();
    copy.dataset_id = DatasetId::new("flex-0123456789abcdef-copy").unwrap();
    std::fs::write(&extra, serde_json::to_string(&copy).unwrap()).unwrap();
    let (_, p1) = call(&r, "GET", "/api/v1/datasets?page_size=1", None).await;
    assert_eq!(p1["total_count"], 2, "{p1}");
    assert_eq!(p1["items"].as_array().unwrap().len(), 1);
    let cursor = p1["next_cursor"].as_str().unwrap();
    let (_, p2) = call(
        &r,
        "GET",
        &format!("/api/v1/datasets?page_size=1&cursor={cursor}"),
        None,
    )
    .await;
    assert_eq!(p2["items"].as_array().unwrap().len(), 1);
    assert_ne!(
        p1["items"][0]["bundle"]["dataset_id"],
        p2["items"][0]["bundle"]["dataset_id"]
    );
    assert!(p2.get("next_cursor").is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn held_out_partitions_never_reach_a_research_caller() {
    let _data = DATA.lock().await;
    data_root();
    let dir = tempfile::tempdir().unwrap();
    let s = store(dir.path());
    let r = router(dir.path(), Some(s.service.clone()));
    let dev = s.dev.bundle.dataset_id.to_string();
    let eval = s.eval.bundle.dataset_id.to_string();
    assert!(s.eval.bundle.partitions.contains(&Partition::Test));

    // listing: neither the validation-holding builder bundle nor the evaluator bundle
    let (_, page) = call(&r, "GET", "/api/v1/datasets", None).await;
    let text = page.to_string();
    assert!(!text.contains(&dev) && !text.contains(&eval), "{text}");

    // by id: the validation bundle is refused, the evaluator root is not even opened
    let (st, e) = call(&r, "GET", &format!("/api/v1/datasets/{dev}"), None).await;
    assert_eq!(
        (st, e["code"].as_str()),
        (StatusCode::FORBIDDEN, Some("scope_denied"))
    );
    let (st, e) = call(&r, "GET", &format!("/api/v1/datasets/{eval}"), None).await;
    assert_eq!(
        (st, e["code"].as_str()),
        (StatusCode::NOT_FOUND, Some("not_found"))
    );
    for id in [&dev, &eval] {
        let (st, h, body) = raw(
            &r,
            "GET",
            &format!("/api/v1/datasets/{id}/views/obs_liquid_diameter"),
            None,
        )
        .await;
        assert!(
            st == StatusCode::FORBIDDEN || st == StatusCode::NOT_FOUND,
            "{st}"
        );
        assert_eq!(h["content-type"], "application/json");
        assert!(!body.starts_with(b"PAR1"));
    }

    // conditions: the test run's row is withheld from lists, queries, envelopes and by id
    let (st, t) = call(&r, "GET", "/api/v1/runs?include_envelopes=true", None).await;
    assert_eq!(st, StatusCode::OK, "{t}");
    let runs: Vec<&str> = t["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["run"]["run_id"].as_str().unwrap())
        .collect();
    assert_eq!(runs, [TRAIN_RUN]);
    assert!(!t.to_string().contains(TEST_RUN));
    assert!(
        !t.to_string().contains("77777"),
        "held-out value leaked: {t}"
    );
    let (st, e) = call(&r, "GET", &format!("/api/v1/runs/{}", enc(TEST_RUN)), None).await;
    assert_eq!(
        (st, e["code"].as_str()),
        (StatusCode::NOT_FOUND, Some("not_found"))
    );
    let q =
        json!({"schema_version": "dataset.v2", "run_ids": [TEST_RUN], "include_envelopes": true});
    let (st, t) = call(&r, "POST", "/api/v1/conditions", Some(q)).await;
    assert_eq!(st, StatusCode::OK, "{t}");
    assert_eq!(t["rows"], json!([]));
    assert!(!t.to_string().contains("77777"), "{t}");
}

#[tokio::test(flavor = "multi_thread")]
async fn conditions_keep_statuses_raw_si_and_locator() {
    let _data = DATA.lock().await;
    data_root();
    let dir = tempfile::tempdir().unwrap();
    let s = store(dir.path());
    let r = router(dir.path(), Some(s.service.clone()));

    let (st, t) = call(&r, "GET", &format!("/api/v1/runs/{}", enc(TRAIN_RUN)), None).await;
    assert_eq!(st, StatusCode::OK, "{t}");
    let table: ConditionsTable = serde_json::from_value(t.clone()).unwrap();
    table.validate().unwrap();
    let cells = t["rows"][0]["cells"].as_array().unwrap();
    let cell = |k: &str| cells.iter().find(|c| c["key"] == k).unwrap();
    assert_eq!(cell("gravity_level")["status"], "missing");
    assert!(cell("gravity_level")["reason"].as_str().unwrap().len() > 3);
    let p = &cell("pressure")["value"];
    assert_eq!(
        (p["raw_value"].as_f64(), p["raw_unit"].as_str()),
        (Some(14.7), Some("psi"))
    );
    assert_eq!(
        (p["value_si"].as_f64(), p["unit_si"].as_str()),
        (Some(101352.9), Some("Pa"))
    );
    assert_eq!(p["locator"]["column"], "P (psia)");
    assert_eq!(t["origin"], "synthetic_fixture");

    // key projection and strict bodies
    let q = json!({"schema_version": "dataset.v2", "keys": ["pressure"]});
    let (st, t) = call(&r, "POST", "/api/v1/conditions", Some(q)).await;
    assert_eq!(st, StatusCode::OK, "{t}");
    assert_eq!(t["keys"], json!(["pressure"]));
    let q = json!({"schema_version": "dataset.v2", "scope": {"role": "evaluation"}});
    let (st, e) = call(&r, "POST", "/api/v1/conditions", Some(q)).await;
    assert_eq!(
        (st, e["code"].as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, Some("invalid_request"))
    );
    let (st, _) = call(&r, "GET", "/api/v1/runs?partition=test", None).await;
    assert_eq!(
        st,
        StatusCode::BAD_REQUEST,
        "unknown query fields are refused"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn without_a_dataset_service_the_routes_say_so_and_openapi_lists_them() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), None);
    for p in [
        "/api/v1/datasets",
        "/api/v1/runs",
        "/api/v1/datasets/x/views/runs",
    ] {
        let (st, e) = call(&r, "GET", p, None).await;
        assert_eq!(
            (st, e["code"].as_str()),
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Some("capability_unavailable")
            ),
            "{p}"
        );
    }
    let (_, doc) = call(&r, "GET", "/api/v1/openapi.json", None).await;
    let paths = doc["paths"].as_object().unwrap();
    for p in [
        "/datasets",
        "/datasets/{id}",
        "/datasets/{id}/views/{view}",
        "/runs",
        "/runs/{id}",
        "/conditions",
    ] {
        assert!(paths.contains_key(p), "{p} missing from OpenAPI");
    }
    assert!(doc["components"]["schemas"]
        .as_object()
        .unwrap()
        .contains_key("DatasetPage"));
}

#[tokio::test(flavor = "multi_thread")]
async fn fixture_mode_serves_the_synthetic_dataset_and_conditions() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), None);
    let base = "/api/fixtures/v1";
    let (st, page) = call(&r, "GET", &format!("{base}/datasets"), None).await;
    assert_eq!(st, StatusCode::OK, "{page}");
    assert_eq!(page["items"][0]["bundle"]["origin"], "synthetic_fixture");
    let id = page["items"][0]["bundle"]["dataset_id"]
        .as_str()
        .unwrap()
        .to_string();
    let (st, b) = call(&r, "GET", &format!("{base}/datasets/{id}"), None).await;
    assert_eq!(
        (st, b["origin"].as_str()),
        (StatusCode::OK, Some("synthetic_fixture"))
    );
    let (st, e) = call(
        &r,
        "GET",
        &format!("{base}/datasets/{id}/views/observations"),
        None,
    )
    .await;
    assert_eq!(
        (st, e["code"].as_str()),
        (StatusCode::NOT_FOUND, Some("not_found"))
    );

    let (st, t) = call(&r, "GET", &format!("{base}/runs?page_size=1"), None).await;
    assert_eq!(st, StatusCode::OK, "{t}");
    assert_eq!(
        (
            t["rows"].as_array().unwrap().len(),
            t["total_count"].as_u64()
        ),
        (1, Some(2))
    );
    assert_eq!(t["origin"], "synthetic_fixture");
    let (st, t) = call(&r, "GET", &format!("{base}/runs/fixture-run-02"), None).await;
    assert_eq!(st, StatusCode::OK, "{t}");
    assert_eq!(t["rows"][0]["run"]["run_id"], "fixture-run-02");
    let (st, _) = call(&r, "GET", &format!("{base}/runs/fixture-run-99"), None).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

/// The M11 answers are shaped for a later host-side numeric grounding check (not built here):
/// each names the data version it was read from, rows are addressed by identity columns, unknown
/// names fail with the permitted ones, and a change of the underlying data changes the snapshot.
#[tokio::test(flavor = "multi_thread")]
async fn answers_are_citable_against_a_named_snapshot() {
    let _data = DATA.lock().await;
    let root = data_root();
    let dir = tempfile::tempdir().unwrap();
    let s = store(dir.path());
    let r = router(dir.path(), Some(s.service.clone()));
    let train = &s.train.bundle;
    let digest = train.content_digest();

    // datasets: page entries carry the content digest and per-view row identity columns
    let (_, page) = call(&r, "GET", "/api/v1/datasets", None).await;
    let e = &page["items"][0];
    assert_eq!(e["content_digest"], digest.as_str());
    for v in &train.views {
        let ids = e["row_identity"][&v.name].as_array().unwrap();
        assert!(!ids.is_empty(), "view {} has no identity columns", v.name);
        for c in ids {
            assert!(v.columns.iter().any(|col| col.name == c.as_str().unwrap()));
        }
        assert!(
            ids.iter().any(|c| c == "run_id"),
            "{}: rows are addressed by run id",
            v.name
        );
    }
    let path = format!("/api/v1/datasets/{}", train.dataset_id);
    let (st, h, _) = raw(&r, "GET", &path, None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(h["x-ignis-snapshot"], digest.as_str());
    assert_eq!(h["etag"].to_str().unwrap(), format!("\"{digest}\""));
    let (st, h, _) = raw(&r, "GET", &format!("{path}/views/runs"), None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(h["x-ignis-snapshot"], digest.as_str());
    // an unknown view fails closed and names the permitted ones
    let (st, e) = call(&r, "GET", &format!("{path}/views/nope"), None).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    let msg = e["message"].as_str().unwrap();
    assert!(
        msg.contains("obs_liquid_diameter") && msg.contains("runs"),
        "{msg}"
    );

    // conditions: one snapshot across routes, equal to the service's
    let want = s
        .service
        .conditions_snapshot(&ignis_server::v1::io::research_scope("local", false))
        .unwrap();
    let (st, h, b) = raw(&r, "GET", "/api/v1/runs", None).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(h["x-ignis-snapshot"], want.as_str());
    let q = json!({"schema_version": "dataset.v2", "keys": ["pressure"]});
    let (_, h2, _) = raw(&r, "POST", "/api/v1/conditions", Some(q)).await;
    assert_eq!(h2["x-ignis-snapshot"], want.as_str());
    let (_, h3, _) = raw(&r, "GET", &format!("/api/v1/runs/{}", enc(TRAIN_RUN)), None).await;
    assert_eq!(h3["x-ignis-snapshot"], want.as_str());
    // rows are distinct by (investigation_id, run_id)
    let t: Value = serde_json::from_slice(&b).unwrap();
    let keys: std::collections::BTreeSet<(String, String)> = t["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["investigation_id"].to_string(),
                r["run"]["run_id"].to_string(),
            )
        })
        .collect();
    assert_eq!(keys.len(), t["rows"].as_array().unwrap().len());
    // an unknown condition key fails closed and names the permitted keys
    let q = json!({"schema_version": "dataset.v2", "keys": ["flame_colour"]});
    let (st, e) = call(&r, "POST", "/api/v1/conditions", Some(q)).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    let msg = e["message"].as_str().unwrap();
    assert!(
        msg.contains("flame_colour") && msg.contains("pressure"),
        "{msg}"
    );

    // the same answer after a change to the data names a different snapshot
    let table = root.join("work/conditions/conditions_table.json");
    let before = std::fs::read(&table).unwrap();
    let mut t: Value = serde_json::from_slice(&before).unwrap();
    t["rows"][0]["cells"][1]["value"]["value_si"] = json!(101353.0);
    std::fs::write(&table, serde_json::to_vec_pretty(&t).unwrap()).unwrap();
    let (_, h4, _) = raw(&r, "GET", "/api/v1/runs", None).await;
    std::fs::write(&table, &before).unwrap();
    assert_ne!(h4["x-ignis-snapshot"], want.as_str());
    let (_, h5, _) = raw(&r, "GET", "/api/v1/runs", None).await;
    assert_eq!(h5["x-ignis-snapshot"], want.as_str());
}

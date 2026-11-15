//! M5 end to end: two turns of the fake Codex app-server (a tool call that takes 150 ms and
//! running `thread/tokenUsage/updated` totals) → `GET /api/v1/episodes/{id}/metrics` and
//! `GET /api/v1/metrics`: measured p50/p95, cache hit per turn, question and episode, cost from the
//! `--price-list` file, and `unpriced` for a model the list does not price.

use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_server::v1::agent::CodexArgs;
use ignis_server::v1::io::AuthMode;
use ignis_server::{app, fixture, ServeArgs, Services};
use serde_json::{json, Value};
use tower::ServiceExt;

fn args(dir: &Path, price_list: PathBuf) -> ServeArgs {
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
        dev: false,
        fixtures: crate_dir.join("fixtures"),
        web_dist: dir.join("no-dist"),
        state_dir: dir.join("state"),
        auth: AuthMode::Local,
        internal_routes: false,
        enable_models: false,
        max_upload_bytes: 1 << 20,
        contract_fixtures: "fixtures/contracts".into(),
        codex: CodexArgs {
            bin: Some(env!("CARGO_BIN_EXE_ignis-server-fake-codex").into()),
            ..Default::default()
        },
        mcp_port: 0,
        runner: Default::default(),
        sources: Default::default(),
        price_list: Some(price_list),
        artifact_backend: Default::default(),
        r2: Default::default(),
        access: Default::default(),
    }
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

/// Post a message and wait until the episode is `completed` with `turns` turns done.
async fn turn(r: &axum::Router, id: &str, text: &str, key: &str, turns: usize) {
    let (st, acc) = call(
        r,
        "POST",
        &format!("/api/v1/episodes/{id}/messages"),
        Some(json!({"text": text, "client_request_id": key})),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{acc}");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let (_, m) = call(r, "GET", &format!("/api/v1/episodes/{id}/metrics"), None).await;
        let (_, ep) = call(r, "GET", &format!("/api/v1/episodes/{id}"), None).await;
        let done = m["turns"].as_array().map_or(0, |t| {
            t.iter().filter(|t| t["tokens"]["reports"] == 2).count()
        });
        if ep["status"] == "completed" && done >= turns {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "turn {key} not done: {ep} {m}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn pct(v: &Value) -> f64 {
    v["cache_hit_pct"].as_f64().unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[tokio::test(flavor = "multi_thread")]
async fn the_metrics_routes_measure_latency_tokens_cache_and_cost_of_real_turns() {
    let dir = tempfile::tempdir().unwrap();
    // The fake runtime answers with model "fake"; this list prices it (synthetic prices).
    let list = dir.path().join("prices.toml");
    std::fs::write(
        &list,
        "name = \"fixture-prices\"\nversion = \"7\"\ncurrency = \"USD\"\nsource = \"synthetic fixture (not a real price)\"\n\
         [[models]]\nid = \"fake\"\ninput_per_mtok = 1.0\ncached_input_per_mtok = 0.1\noutput_per_mtok = 2.0\n",
    )
    .unwrap();
    let r = app(
        args(dir.path(), list),
        Services {
            experts: Vec::new(),
            fixture_experts: fixture::experts(),
            acquisition: None,
            research: Default::default(),
        },
    );
    let (st, ep) = call(
        &r,
        "POST",
        "/api/v1/episodes",
        Some(json!({"goal": "metrics", "client_request_id": "c-metrics"})),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{ep}");
    let id = ep["episode_id"].as_str().unwrap().to_string();

    // Before any turn: no usage, not zero dollars.
    let (st, m) = call(&r, "GET", &format!("/api/v1/episodes/{id}/metrics"), None).await;
    assert_eq!(st, StatusCode::OK, "{m}");
    assert_eq!(m["cost"]["status"], "no_usage");

    turn(&r, &id, "slowtool search, then usage", "m1", 1).await;
    turn(&r, &id, "slowtool search again, then usage", "m2", 2).await;

    let (st, m) = call(&r, "GET", &format!("/api/v1/episodes/{id}/metrics"), None).await;
    assert_eq!(st, StatusCode::OK, "{m}");
    let parsed: ignis_core::contracts::EpisodeMetrics = serde_json::from_value(m.clone()).unwrap();
    parsed.validate().unwrap();
    assert_eq!(m["price_list"], "fixture-prices@7");
    assert_eq!(m["harness"]["model"], "fake");

    // Latency measured from the log: both calls took at least the fake's 150 ms.
    let tools = m["tools"].as_array().unwrap();
    let search = tools
        .iter()
        .find(|t| t["tool"] == "sources_search")
        .unwrap();
    assert_eq!(
        (search["calls"].as_u64(), search["unfinished"].as_u64()),
        (Some(2), Some(0))
    );
    let lat = &search["latency"];
    assert_eq!(lat["samples"], 2);
    let (p50, p95, max) = (
        lat["p50_ms"].as_f64().unwrap(),
        lat["p95_ms"].as_f64().unwrap(),
        lat["max_ms"].as_f64().unwrap(),
    );
    assert!(p50 >= 150.0 && p50 <= p95 && close(p95, max), "{lat}");

    // Per turn: the thread totals become per-turn usage (the partial total is not counted twice).
    // Turn 1: 1000 in, 400 cached (40 %); turn 2: 1000 more, 800 more cached (80 %).
    let turns = m["turns"].as_array().unwrap();
    assert_eq!(turns.len(), 2, "{m}");
    let (t1, t2) = (&turns[0]["tokens"], &turns[1]["tokens"]);
    assert_eq!(
        (
            t1["input_tokens"].as_u64(),
            t1["cached_input_tokens"].as_u64()
        ),
        (Some(1000), Some(400))
    );
    assert_eq!(
        (
            t2["input_tokens"].as_u64(),
            t2["cached_input_tokens"].as_u64()
        ),
        (Some(1000), Some(800))
    );
    assert_eq!(
        (
            t1["output_tokens"].as_u64(),
            t1["reasoning_tokens"].as_u64()
        ),
        (Some(100), Some(40))
    );
    assert!(close(pct(t1), 40.0) && close(pct(t2), 80.0), "{t1} {t2}");
    assert!(close(pct(&m["tokens"]), 60.0));

    // Per question: one turn and one call each; cost from the list.
    // Turn 1: (600 × 1 + 400 × 0.1 + 100 × 2) / 1e6; turn 2: (200 × 1 + 800 × 0.1 + 100 × 2) / 1e6.
    let qs = m["questions"].as_array().unwrap();
    assert_eq!(qs.len(), 2);
    let want = [840e-6, 480e-6];
    for (q, w) in qs.iter().zip(want) {
        assert_eq!(
            (q["turns"].as_u64(), q["tool_calls"].as_u64()),
            (Some(1), Some(1))
        );
        assert_eq!(q["cost"]["status"], "priced", "{q}");
        assert_eq!(q["cost"]["cost"]["basis"], "price_list");
        assert_eq!(q["cost"]["cost"]["price_list"], "fixture-prices@7");
        assert!(
            close(q["cost"]["cost"]["amount"].as_f64().unwrap(), w),
            "{q}"
        );
        assert!(q["wall_time_s"].as_f64().unwrap() >= 0.15);
    }

    // The workspace aggregate: every call, the token sums and the mean cost per question.
    let (st, w) = call(&r, "GET", "/api/v1/metrics", None).await;
    assert_eq!(st, StatusCode::OK, "{w}");
    assert_eq!(
        (w["episodes"].as_u64(), w["questions"].as_u64()),
        (Some(1), Some(2))
    );
    assert_eq!(w["tools"][0]["latency"]["samples"], 2);
    assert!(
        close(w["cost"]["cost"]["amount"].as_f64().unwrap(), 1320e-6),
        "{w}"
    );
    assert!(
        close(
            w["cost_per_question"]["cost"]["amount"].as_f64().unwrap(),
            660e-6
        ),
        "{w}"
    );

    // An unknown episode is a 404, not empty metrics.
    let (st, _) = call(&r, "GET", "/api/v1/episodes/ep_missing/metrics", None).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

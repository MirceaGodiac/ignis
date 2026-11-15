//! Tool dispatcher invariants (M3): registration follows configured services, the scope comes
//! from the host and gates every call, arguments are the declared contracts, jobs and artifacts stay
//! in their workspace, and large results never travel inline.

mod common;

use std::sync::atomic::Ordering;

use common::*;
use ignis_core::contracts::*;
use ignis_core::services::{ArtifactDraft, ArtifactStore};
use ignis_mcp::schemas::input_schema;
use serde_json::{json, Value};

fn ok(r: &ToolResult) {
    r.validate().unwrap();
    assert!(
        matches!(
            r.execution_status,
            ExecutionStatus::Completed | ExecutionStatus::Accepted
        ),
        "{r:#?}"
    );
}

fn refused(r: &ToolResult, code: ErrorCode) {
    r.validate().unwrap();
    assert_eq!(r.execution_status, ExecutionStatus::Refused, "{r:#?}");
    assert_eq!(r.error.as_ref().unwrap().code, code, "{r:#?}");
    assert_eq!(r.scientific_status, ScientificStatus::NotAssessed);
}

#[test]
fn every_catalog_input_has_an_inlined_object_schema() {
    for t in tool_catalog() {
        let s = input_schema(&t.input_contract)
            .unwrap_or_else(|| panic!("{}: no schema for {}", t.name, t.input_contract));
        assert_eq!(s["type"], "object", "{}", t.name);
        let text = s.to_string();
        assert!(
            !text.contains("$ref") && !text.contains("$defs"),
            "{}: schema keeps references",
            t.name
        );
    }
}

/// Presence lint over every catalog description (not prose quality), after the smell taxonomies of
/// Hasan et al. (arXiv 2602.14878) and Wang et al. (arXiv 2602.18914): states its usage (when to
/// use it, or what to use instead), a limitation (what it never returns, paging, refusal,
/// partial results), and names at least one of its own arguments as `name`; never repeats its
/// own tool name; no few-shot examples (a description documents, Hsieh et al. arXiv 2308.00675).
#[test]
fn every_tool_description_states_usage_limits_and_arguments() {
    let usage = ["Use ", "use "];
    let limits = [
        "never", "not ", "refused", "partial", "Paged", "paged", "only", "withheld",
    ];
    for t in tool_catalog() {
        let d = &t.summary;
        let n = &t.name;
        assert!(!d.contains(n.as_str()), "{n}: repeats its own name");
        assert!(
            usage.iter().any(|u| d.contains(u)),
            "{n}: no usage guideline"
        );
        assert!(
            limits.iter().any(|l| d.contains(l)),
            "{n}: no stated limitation"
        );
        assert!(
            !d.contains("Example") && !d.contains("e.g. {"),
            "{n}: few-shot example in the description"
        );
        let schema = input_schema(&t.input_contract).unwrap();
        let args: Vec<&String> = schema["properties"]
            .as_object()
            .map(|o| o.keys().filter(|k| *k != "schema_version").collect())
            .unwrap_or_default();
        assert!(
            args.iter().any(|a| d.contains(&format!("`{a}`"))),
            "{n}: names none of its arguments {args:?}"
        );
    }
}

#[test]
fn only_tools_with_a_configured_service_are_registered() {
    let h = host();
    let names: Vec<&str> = h
        .host
        .registered()
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    for n in [
        "sources_search",
        "sources_read",
        "models_inspect",
        "models_forecast",
        "artifacts_read",
        "jobs_get",
    ] {
        assert!(names.contains(&n), "{n} missing from {names:?}");
    }
    // DatasetService is not configured and no wiki store is attached: not tools, not negative results.
    assert!(!names.contains(&"datasets_inspect"));
    assert!(!names.contains(&"wiki_patch"));
    // No runner passed preflight: code execution is not offered at all (and never on the host).
    assert!(!names.contains(&"analysis_execute"));
    let why = ignis_mcp::catalog(&Default::default())
        .into_iter()
        .find(|t| t.descriptor.name == "analysis_execute")
        .and_then(|t| t.unavailable);
    assert_eq!(
        why,
        Some("isolated runner not configured or failed preflight")
    );
    let r = h.host.call(
        &scope("ws_a"),
        "analysis_execute",
        json!({"language": "python", "source": "print(1)"}),
    );
    refused(&r, ErrorCode::NotFound);
    let r = h.host.call(
        &scope("ws_a"),
        "datasets_inspect",
        json!({"dataset_id": "x"}),
    );
    refused(&r, ErrorCode::NotFound);
}

#[test]
fn the_service_receives_the_host_scope_and_returns_its_contract() {
    let h = host();
    let s = scope("ws_a");
    let r = h.host.call(&s, "sources_search", detailed_search_args());
    ok(&r);
    assert_eq!(r.execution_status, ExecutionStatus::Completed);
    let page: SearchPage = serde_json::from_value(r.result.unwrap().value).unwrap();
    page.validate().unwrap();
    assert_eq!(h.knowledge.calls.lock().unwrap().as_slice(), &[s]);
}

fn detailed_search_args() -> Value {
    let mut a = search_args();
    a["schema_version"] = json!("evidence.v2");
    a["response_format"] = json!("detailed");
    a
}

#[test]
fn search_results_are_concise_by_default_and_detailed_on_request() {
    let h = host();
    let s = scope("ws_a");
    // Ten hits of the fixture page, as a real first page.
    h.knowledge.repeat.store(10, Ordering::SeqCst);
    h.knowledge.quote_less.store(true, Ordering::SeqCst);
    let concise = h.host.call(&s, "sources_search", search_args());
    ok(&concise);
    let concise = concise.result.unwrap().value;
    let detailed = h.host.call(&s, "sources_search", detailed_search_args());
    ok(&detailed);
    let detailed = detailed.result.unwrap().value;
    let page: SearchPage = serde_json::from_value(detailed.clone()).unwrap();
    assert!(page.hits.iter().any(|h| !h.scores.is_empty()));

    // The same hits in the same order either way, with what the agent needs to choose and cite.
    let hits = concise["hits"].as_array().unwrap();
    assert_eq!(hits.len(), page.hits.len());
    for (c, d) in hits.iter().zip(&page.hits) {
        assert_eq!(c["rank"], json!(d.rank));
        assert_eq!(c["evidence_id"], json!(d.evidence.evidence_id));
        assert_eq!(c["evidence_class"], json!(d.evidence.evidence_class));
        assert_eq!(c["source_id"], json!(d.evidence.source_id));
        assert_eq!(c["snippet"], json!(d.snippet));
        let l = d.evidence.locator.as_ref().unwrap();
        assert_eq!(c["page"], json!(l.page));
        assert_eq!(c["page_label"], json!(l.page_label));
        // Locator internals, scores and the page's own origin stay out of the concise view.
        for k in [
            "locator",
            "scores",
            "origin",
            "title",
            "evidence",
            "parse_view_id",
        ] {
            assert!(c.get(k).is_none(), "{k} in a concise hit: {c}");
        }
    }
    // Each source's title once, and only the backends that changed the search.
    let title = page.hits[0].title.clone().unwrap();
    assert_eq!(
        concise["sources"],
        json!({ page.hits[0].evidence.source_id.as_str(): title })
    );
    let backends: Vec<&str> = concise["backends"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["backend"].as_str().unwrap())
        .collect();
    assert_eq!(
        backends,
        ["dense"],
        "the degraded backend is reported, the rest not"
    );
    assert_eq!(concise["next_cursor"], json!(page.next_cursor));
    assert_eq!(concise["origin"], json!(page.origin));
    let (c, d) = (concise.to_string().len(), detailed.to_string().len());
    eprintln!("ten fixture hits: concise {c} bytes, detailed {d} bytes");
    assert!(
        c * 2 < d,
        "concise {c} bytes is not compact next to detailed {d}"
    );

    // A v2 field under the v1 tag is refused with what to change.
    let mut v1 = search_args();
    v1["response_format"] = json!("detailed");
    let r = h.host.call(&s, "sources_search", v1);
    refused(&r, ErrorCode::InvalidRequest);
    assert!(r.error.unwrap().message.contains("evidence.v2"));
}

#[test]
fn conditions_query_is_a_thin_adapter_over_the_dataset_service() {
    let datasets = std::sync::Arc::new(FakeDatasets::default());
    let h = host_with_datasets(datasets.clone());
    let s = scope("ws_a");
    assert!(h
        .host
        .registered()
        .iter()
        .any(|d| d.name == "conditions_query"));
    let r = h.host.call(
        &s,
        "conditions_query",
        json!({"schema_version": "dataset.v2", "include_envelopes": true}),
    );
    ok(&r);
    // The table is stored as a ConditionsTable artifact (citable by id and digest) and travels
    // inline when small; this service reports no snapshot, and the result says so.
    let stored = r.artifact_refs[0].clone();
    assert_eq!(stored.contract.as_deref(), Some("ConditionsTable"));
    let v = r.result.unwrap().value;
    assert_eq!(v["artifact_id"], json!(stored.artifact_id));
    assert_eq!(v["sha256"], json!(stored.sha256));
    assert_eq!(v["conditions_snapshot"], Value::Null);
    assert!(v["how_to_cite"]
        .as_str()
        .unwrap()
        .contains("no conditions snapshot"));
    let table: ConditionsTable = serde_json::from_value(v["table"].clone()).unwrap();
    table.validate().unwrap();
    let q = datasets.queries.lock().unwrap().clone();
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].0, s, "the host scope reaches the service");
    assert!(q[0].1.include_envelopes);
    // Invalid queries are refused before the service runs; so is a scope without read_data.
    let r = h.host.call(
        &s,
        "conditions_query",
        json!({"schema_version": "dataset.v2", "page_size": 0}),
    );
    refused(&r, ErrorCode::InvalidRequest);
    let mut no_data = s.clone();
    no_data.capabilities.remove(&Capability::ReadData);
    let r = h.host.call(
        &no_data,
        "conditions_query",
        json!({"schema_version": "dataset.v2"}),
    );
    refused(&r, ErrorCode::ScopeDenied);
    assert_eq!(datasets.queries.lock().unwrap().len(), 1);
    // Datasets present but the conditions table not built: not registered, and status says why.
    let services = ignis_mcp::ToolServices {
        datasets: Some(datasets.clone()),
        conditions: false,
        ..Default::default()
    };
    let why = ignis_mcp::catalog(&services)
        .into_iter()
        .find(|t| t.descriptor.name == "conditions_query")
        .and_then(|t| t.unavailable);
    assert_eq!(why, Some(ignis_mcp::CONDITIONS_NOT_BUILT));
}

#[test]
fn errors_say_what_to_do_next() {
    let h = host();
    let r = h.host.call(&scope("ws_a"), "sources_serach", json!({}));
    refused(&r, ErrorCode::NotFound);
    assert!(r.error.unwrap().message.contains("sources_search"));
    let r = h
        .host
        .call(&scope("ws_a"), "sources_search", json!({"query": "x"}));
    refused(&r, ErrorCode::InvalidRequest);
    let m = r.error.unwrap().message;
    assert!(
        m.contains("schema_version") && m.contains("SearchRequest"),
        "{m}"
    );
    // A closed value that is not permitted is refused with the permitted values.
    let mut args = search_args();
    args["mode"] = json!("fuzzy");
    let r = h.host.call(&scope("ws_a"), "sources_search", args);
    refused(&r, ErrorCode::InvalidRequest);
    let m = r.error.unwrap().message;
    assert!(m.contains("hybrid") && m.contains("lexical"), "{m}");
}

#[test]
fn a_scope_without_the_capability_is_refused_before_the_service_runs() {
    let h = host();
    let mut s = scope("ws_a");
    s.capabilities.remove(&Capability::ReadSources);
    let r = h.host.call(&s, "sources_search", search_args());
    refused(&r, ErrorCode::ScopeDenied);
    assert!(h.knowledge.calls.lock().unwrap().is_empty());
}

#[test]
fn arguments_cannot_carry_a_scope_or_unknown_fields() {
    let h = host();
    let mut args = search_args();
    args["workspace_id"] = json!("ws_other");
    let r = h.host.call(&scope("ws_a"), "sources_search", args);
    refused(&r, ErrorCode::InvalidRequest);
    let r = h
        .host
        .call(&scope("ws_a"), "sources_search", json!("not an object"));
    refused(&r, ErrorCode::InvalidRequest);
    assert!(h.knowledge.calls.lock().unwrap().is_empty());
}

#[test]
fn forecast_is_a_job_visible_only_in_its_workspace() {
    let h = host();
    let (a, b) = (scope("ws_a"), scope("ws_b"));
    let r = h.host.call(&a, "models_forecast", forecast_args("crid-1"));
    ok(&r);
    assert_eq!(r.execution_status, ExecutionStatus::Accepted);
    let job = r.job.clone().unwrap();
    assert_eq!(job.kind, ignis_mcp::FORECAST_JOB);
    assert_eq!(h.wakes.load(Ordering::SeqCst), 1, "the runner is woken");

    // Same idempotency key: the same job, not a second one.
    let again = h.host.call(&a, "models_forecast", forecast_args("crid-1"));
    assert_eq!(again.job.unwrap().job_id, job.job_id);

    let get = h.host.call(&a, "jobs_get", json!({"job_id": job.job_id}));
    ok(&get);
    assert_eq!(get.result.unwrap().value["job_id"], json!(job.job_id));
    // Another workspace does not see it (and cannot tell it exists).
    let other = h.host.call(&b, "jobs_get", json!({"job_id": job.job_id}));
    refused(&other, ErrorCode::NotFound);
}

#[test]
fn a_forecast_without_run_models_is_refused_and_queues_nothing() {
    let h = host();
    let mut s = scope("ws_a");
    s.capabilities.remove(&Capability::RunModels);
    let r = h.host.call(&s, "models_forecast", forecast_args("crid-2"));
    refused(&r, ErrorCode::ScopeDenied);
    assert_eq!(h.wakes.load(Ordering::SeqCst), 0);
}

fn put_json(h: &Host, s: &ExecutionScope, v: &Value) -> ArtifactRef {
    h.artifacts
        .put(
            s,
            ArtifactDraft {
                media_type: "application/json".into(),
                contract: None,
                produced_by: Producer {
                    episode_id: None,
                    job_id: None,
                    tool_call_id: None,
                    release_id: None,
                },
                inputs: vec![],
                origin: Origin::SyntheticFixture,
            },
            &mut serde_json::to_vec(v).unwrap().as_slice(),
        )
        .unwrap()
}

#[test]
fn artifacts_read_pages_text_and_resolves_pointers_within_the_workspace() {
    let h = host();
    let (a, b) = (scope("ws_a"), scope("ws_b"));
    let doc = json!({"series": [{"t": 0.5, "d_mm": 1.25}], "note": "é".repeat(40_000)});
    let art = put_json(&h, &a, &doc);

    let r = h.host.call(
        &a,
        "artifacts_read",
        json!({"artifact_id": art.artifact_id, "json_pointer": "/series/0/d_mm"}),
    );
    ok(&r);
    assert_eq!(r.result.as_ref().unwrap().value, json!(1.25));

    // A page that stops before the end is partial, ends on a character boundary, and says where
    // the next one starts; following the offsets reassembles the bytes exactly.
    let mut text = String::new();
    let mut offset = Some(0u64);
    let mut pages = 0;
    while let Some(o) = offset {
        let r = h.host.call(
            &a,
            "artifacts_read",
            json!({"artifact_id": art.artifact_id, "byte_offset": o, "byte_length": 30_001}),
        );
        ok(&r);
        let p = r.result.unwrap();
        assert!(p.full.is_some());
        text.push_str(p.value["text"].as_str().unwrap());
        offset = p.value["next_offset"].as_u64();
        pages += 1;
        assert!(pages < 10);
    }
    assert!(pages > 1);
    assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), doc);

    let r = h.host.call(
        &b,
        "artifacts_read",
        json!({"artifact_id": art.artifact_id}),
    );
    refused(&r, ErrorCode::NotFound);
}

#[test]
fn a_large_result_is_stored_and_previewed_not_inlined() {
    let h = host();
    h.knowledge.repeat.store(2_000, Ordering::SeqCst);
    let s = scope("ws_a");
    let r = h.host.call(&s, "sources_search", search_args());
    ok(&r);
    let p = r.result.clone().unwrap();
    assert!(p.partial);
    let full = p.full.clone().unwrap();
    assert_eq!(full.contract.as_deref(), Some("SearchPage"));
    assert_eq!(r.artifact_refs, vec![full.clone()]);
    assert!(serde_json::to_vec(&r).unwrap().len() < ignis_mcp::tools::INLINE_RESULT_BYTES);
    // The preview is of the concise view the agent reads (no locator internals per hit) ...
    let preview = &p.value["preview"]["hits"];
    assert!(!preview.as_array().unwrap().is_empty());
    assert!(preview[0].get("evidence_id").is_some(), "{preview:#}");
    assert!(preview[0].get("evidence").is_none(), "{preview:#}");
    // ... while the stored artifact is the service's full answer, readable by the agent.
    let first = h.host.call(
        &s,
        "artifacts_read",
        json!({"artifact_id": full.artifact_id, "json_pointer": "/hits/0"}),
    );
    ok(&first);
    assert!(first.result.unwrap().value.get("evidence").is_some());
}

/// Every read of a large artifact stays a bounded inline page: a pointer to a long array answers a
/// slice that says where the next one starts, a pointer to a large object a preview of it, and a
/// byte page without a length a page well under the inline limit.
#[test]
fn artifacts_read_never_inlines_a_whole_large_artifact() {
    let h = host();
    let s = scope("ws_a");
    let rows: Vec<Value> = (0..2_000)
        .map(|i| json!({"run": format!("r{i:04}"), "value": i, "note": "x".repeat(70)}))
        .collect();
    let doc = json!({"rows": rows, "meta": {"n": 2000}});
    let art = put_json(&h, &s, &doc);
    assert!(art.byte_length > 200_000, "{}", art.byte_length);
    let limit = ignis_mcp::tools::INLINE_RESULT_BYTES;
    let read = |args: Value| {
        let r = h.host.call(&s, "artifacts_read", args);
        ok(&r);
        assert!(
            serde_json::to_vec(&r).unwrap().len() < limit,
            "inline: {r:?}"
        );
        r.result.unwrap()
    };

    // A pointer to the list: a default slice under the page budget, then the next ones by offset.
    let p = read(json!({"artifact_id": art.artifact_id, "json_pointer": "/rows"}));
    assert!(p.partial);
    assert_eq!(p.full.as_ref().unwrap().artifact_id, art.artifact_id);
    assert_eq!(p.value["of"], json!(2000));
    assert_eq!(p.value["item_offset"], json!(0));
    let first = p.value["items"].as_array().unwrap().len();
    assert!(first > 10 && first < 2000, "{first}");
    assert_eq!(p.value["next_item"], json!(first));
    let mut seen = first;
    let mut next = p.value["next_item"].as_u64();
    while let Some(o) = next {
        let p = read(
            json!({"artifact_id": art.artifact_id, "json_pointer": "/rows", "item_offset": o}),
        );
        let items = p.value["items"].as_array().unwrap();
        assert_eq!(items[0]["value"], json!(o));
        seen += items.len();
        next = p.value["next_item"].as_u64();
    }
    assert_eq!(seen, 2000, "following next_item reads every item once");

    // An explicit slice.
    let p = read(
        json!({"artifact_id": art.artifact_id, "json_pointer": "/rows", "item_offset": 1995, "item_count": 10}),
    );
    assert_eq!(p.value["items"].as_array().unwrap().len(), 5);
    assert_eq!(p.value["next_item"], Value::Null);
    let p = read(
        json!({"artifact_id": art.artifact_id, "json_pointer": "/rows", "item_offset": 10, "item_count": 3}),
    );
    assert_eq!(p.value["items"][0]["value"], json!(10));
    assert_eq!(p.value["next_item"], json!(13));

    // The whole document: a preview of itself that points at the same artifact.
    let p = read(json!({"artifact_id": art.artifact_id, "json_pointer": ""}));
    assert!(p.partial);
    assert_eq!(p.full.as_ref().unwrap().artifact_id, art.artifact_id);
    assert_eq!(p.value["preview"]["meta"], json!({"n": 2000}));
    assert_eq!(p.value["cut"]["rows"]["of"], json!(2000));

    // A byte page without a length.
    let p = read(json!({"artifact_id": art.artifact_id}));
    assert!(p.value["byte_length"].as_u64().unwrap() <= 24 << 10);
    assert!(p.value["next_offset"].as_u64().is_some());

    // Slicing needs a pointer to an array.
    let r = h.host.call(
        &s,
        "artifacts_read",
        json!({"artifact_id": art.artifact_id, "json_pointer": "/meta", "item_count": 1}),
    );
    refused(&r, ErrorCode::InvalidRequest);
    let r = h.host.call(
        &s,
        "artifacts_read",
        json!({"artifact_id": art.artifact_id, "item_offset": 1}),
    );
    refused(&r, ErrorCode::InvalidRequest);
    let r = h.host.call(
        &s,
        "artifacts_read",
        json!({"artifact_id": art.artifact_id, "json_pointer": "/rows", "item_offset": 2001}),
    );
    refused(&r, ErrorCode::InvalidRequest);
}

/// A second host over the same store that waits up to `wait` for jobs (as `ignis serve` does).
fn waiting_host(h: &Host, wait: std::time::Duration) -> ignis_mcp::ToolHost {
    ignis_mcp::ToolHost::new(
        ignis_mcp::ToolServices {
            knowledge: Some(h.knowledge.clone()),
            models: Some(std::sync::Arc::new(FakeModels)),
            ..Default::default()
        },
        h.store.clone(),
        h.artifacts.clone(),
        Origin::SyntheticFixture,
        std::sync::Arc::new(|| {}),
    )
    .with_job_wait(wait)
}

/// What the server's job runner does: claim the next forecast after `delay`, then end it.
fn run_forecast_after(
    h: &Host,
    delay: std::time::Duration,
    outcome: ignis_research::store::JobOutcome,
) -> std::thread::JoinHandle<()> {
    let store = h.store.clone();
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        let job = store
            .claim_job(
                "test-runner",
                &[ignis_mcp::FORECAST_JOB],
                std::time::Duration::from_secs(60),
            )
            .unwrap()
            .expect("a queued forecast");
        store.finish_job(&job.job_id, job.fence, outcome).unwrap();
    })
}

#[test]
fn jobs_get_waits_for_the_job_so_one_call_returns_it_finished() {
    use ignis_research::store::JobOutcome;
    use std::time::{Duration, Instant};
    let h = host();
    let a = scope("ws_a");
    let waiting = waiting_host(&h, Duration::from_secs(20));

    // Queued without waiting (no runner yet), then finished by the runner while jobs_get waits.
    let r = h
        .host
        .call(&a, "models_forecast", forecast_args("crid-wait-1"));
    assert_eq!(r.execution_status, ExecutionStatus::Accepted);
    let job = r.job.unwrap();
    let output = put_json(&h, &a, &json!({"forecast": "synthetic"}));
    let runner = run_forecast_after(
        &h,
        Duration::from_millis(300),
        JobOutcome::Succeeded {
            outputs: vec![output.clone()],
        },
    );
    let started = Instant::now();
    let get = waiting.call(&a, "jobs_get", json!({"job_id": job.job_id}));
    runner.join().unwrap();
    ok(&get);
    let got: JobRef = serde_json::from_value(get.result.unwrap().value).unwrap();
    assert_eq!(
        got.status,
        JobStatus::Succeeded,
        "one call returns the finished job"
    );
    assert_eq!(got.outputs, vec![output]);
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "returned when the job ended"
    );

    // A job that ends within the wait answers its submission directly, failures with their error.
    let runner = run_forecast_after(
        &h,
        Duration::from_millis(200),
        JobOutcome::Failed {
            error: ErrorEnvelope::new(
                ErrorCode::Internal,
                "synthetic runner failure",
                RetryPolicy::Never,
            ),
        },
    );
    let r = waiting.call(&a, "models_forecast", forecast_args("crid-wait-2"));
    runner.join().unwrap();
    r.validate().unwrap();
    assert_eq!(r.execution_status, ExecutionStatus::Failed, "{r:#?}");
    assert_eq!(r.job.as_ref().unwrap().status, JobStatus::Failed);
    assert!(r
        .error
        .unwrap()
        .message
        .contains("synthetic runner failure"));

    // Nothing runs it: the wait is bounded and answers with the current state.
    let short = waiting_host(&h, Duration::from_millis(300));
    let r = h
        .host
        .call(&a, "models_forecast", forecast_args("crid-wait-3"));
    let started = Instant::now();
    let get = short.call(&a, "jobs_get", json!({"job_id": r.job.unwrap().job_id}));
    assert!(started.elapsed() >= Duration::from_millis(300));
    ok(&get);
    assert_eq!(get.result.unwrap().value["status"], json!("queued"));
}

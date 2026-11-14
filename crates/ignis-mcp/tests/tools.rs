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
    // DatasetService is not configured and the wiki has no service: not tools, not negative results.
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
    let r = h.host.call(&s, "sources_search", search_args());
    ok(&r);
    assert_eq!(r.execution_status, ExecutionStatus::Completed);
    let page: SearchPage = serde_json::from_value(r.result.unwrap().value).unwrap();
    page.validate().unwrap();
    assert_eq!(h.knowledge.calls.lock().unwrap().as_slice(), &[s]);
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
    // The full page is the service's answer, readable by the agent.
    let first = h.host.call(
        &s,
        "artifacts_read",
        json!({"artifact_id": full.artifact_id, "json_pointer": "/hits/0"}),
    );
    ok(&first);
}

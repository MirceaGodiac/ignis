//! The tool input schemas the model receives are slimmed from the contracts' schemas
//! (`ignis_mcp::schemas`): far fewer bytes on every model request, and still every call the
//! contract accepts. A minimal call per tool, and the contract fixtures, validate against the slim
//! schema and decode into the contract the dispatcher decodes.

use ignis_core::contracts::*;
use ignis_mcp::schemas::{full_input_schema, input_schema};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

fn fixture(rel: &str) -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/contracts")
        .join(rel);
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap()
}

fn decodes<T: DeserializeOwned>(v: &Value) -> Result<(), String> {
    serde_json::from_value::<T>(v.clone())
        .map(drop)
        .map_err(|e| e.to_string())
}

/// What `ToolHost` decodes for each input contract.
fn decode(contract: &str, v: &Value) -> Result<(), String> {
    match contract {
        "SearchRequest" => decodes::<SearchRequest>(v),
        "ReadRequest" => decodes::<ReadRequest>(v),
        "DatasetRef" => decodes::<DatasetRef>(v),
        "ReleaseRef" => decodes::<ReleaseRef>(v),
        "ForecastRequest" => decodes::<ForecastRequest>(v),
        "ArtifactReadRequest" => decodes::<ArtifactReadRequest>(v),
        "JobLookup" => decodes::<JobLookup>(v),
        "AnalysisExecuteRequest" => decodes::<AnalysisExecuteRequest>(v),
        "WikiPatch" => decodes::<WikiPatch>(v),
        "AnswerSubmission" => decodes::<AnswerSubmission>(v),
        "OutlineRequest" => decodes::<OutlineRequest>(v),
        "ConditionsQuery" => decodes::<ConditionsQuery>(v),
        "GraphNeighborsRequest" => decodes::<GraphNeighborsRequest>(v),
        "WikiReadRequest" => decodes::<WikiReadRequest>(v),
        "WebSearchRequest" => decodes::<WebSearchRequest>(v),
        "WebFetchRequest" => decodes::<WebFetchRequest>(v),
        "SkillRef" => decodes::<SkillRef>(v),
        other => Err(format!("no decoder for {other}")),
    }
}

/// Minimal valid calls per input contract, plus the committed contract fixtures.
fn calls(contract: &str) -> Vec<Value> {
    let fx = fixture;
    let ev = "evidence.v2";
    match contract {
        "SearchRequest" => vec![
            json!({"schema_version": ev, "query": "flame extinction"}),
            fx("evidence/search_request.json"),
            fx("evidence/search_request_concise.json"),
        ],
        "ReadRequest" => vec![
            json!({"schema_version": ev, "source_id": "src-1", "range": {"kind": "pages", "first": 3, "last": 4}}),
            json!({"schema_version": ev, "source_id": "src-1", "range": {"kind": "passage", "evidence_id": "ev-1"}}),
        ],
        "DatasetRef" => vec![json!({"dataset_id": "flex"})],
        "ReleaseRef" => vec![json!({"release_id": "rel-1"})],
        "ForecastRequest" => vec![json!({
            "schema_version": "wsp.v1", "release_id": "rel-1",
            "input": {"kind": "stored_run", "run_id": "FLEX-093", "cutoff_s": 2.0},
            "settings": {"target": "d2", "times_s": [2.5, 3.0]}
        })],
        "ArtifactReadRequest" => vec![json!({"artifact_id": "art-1", "json_pointer": "/hits/0"})],
        "JobLookup" => vec![json!({"job_id": "job-1"})],
        "AnalysisExecuteRequest" => vec![
            json!({"language": "python", "source": "print(1)"}),
            json!({"language": "python", "source": "print(1)", "inputs": [
                {"name": "v.parquet", "dataset_view": {"dataset_id": "flex", "view": "droplet"}},
                {"name": "c.json", "conditions": {"schema_version": "dataset.v2",
                    "filters": [{"key": "pressure", "min_si": 50000.0}]}},
                {"name": "a.json", "artifact_id": "art-1"}
            ]}),
        ],
        "WikiPatch" => vec![
            json!({"schema_version": ev, "page_id": "p-1", "title": "Time zero", "ops": [
                {"op": "add", "item": {"item_id": "i-1", "kind": "note", "text": "x",
                    "refs": [{"kind": "run", "id": "FLEX-093"}, {"concept": "hardware", "id": "hw:mdca"}]}},
                {"op": "remove", "item_id": "i-0", "reason": "superseded"}
            ]}),
            fx("evidence/wiki_patch.json"),
        ],
        "AnswerSubmission" => vec![
            json!({"status": "insufficient_evidence", "blocks": [{"type": "text", "text": "No source."}],
                "limitations": ["no passage reports it"]}),
            json!({"status": "complete", "blocks": [{"type": "text", "text": "x", "claim_ids": ["c1"]}],
                "claims": [{"claim_id": "c1", "kind": "source_reports", "text": "x",
                    "support": [{"kind": "evidence", "evidence_id": "ev-1"}],
                    "values": [{"source": {"kind": "evidence_quote", "evidence_id": "ev-1"}, "value": 3.0, "unit": "mm"}]}]}),
            fx("research/answer_submission.json"),
        ],
        "OutlineRequest" => vec![json!({"schema_version": ev, "source_id": "src-1"})],
        "ConditionsQuery" => vec![json!({"schema_version": "dataset.v2", "run_ids": ["FLEX-093"]})],
        "GraphNeighborsRequest" => vec![
            json!({"schema_version": ev, "node": {"concept": "investigation", "id": "PSI-69"}}),
            json!({"schema_version": ev, "node": {"kind": "run", "id": "FLEX-093"}, "relations": ["part_of"], "direction": "incoming"}),
        ],
        "WikiReadRequest" => vec![json!({"schema_version": ev})],
        "WebSearchRequest" => vec![json!({"schema_version": ev, "query": "candle flame"})],
        "WebFetchRequest" => vec![json!({"schema_version": ev, "url": "https://example.org/a"})],
        "SkillRef" => vec![json!({"name": "source-grounding"})],
        _ => vec![],
    }
}

#[test]
fn slim_input_schemas_still_accept_and_decode_valid_calls() {
    let (mut full_total, mut slim_total) = (0, 0);
    for t in tool_catalog() {
        let c = t.input_contract.as_str();
        let full = full_input_schema(c).unwrap();
        let slim = input_schema(c).unwrap();
        let (fb, sb) = (full.to_string().len(), slim.to_string().len());
        eprintln!("{:<18} {c:<24} {fb:>6} -> {sb:>6} bytes", t.name);
        assert!(sb <= fb, "{}: the slim schema grew", t.name);
        (full_total, slim_total) = (full_total + fb, slim_total + sb);

        let validator = jsonschema::validator_for(&slim)
            .unwrap_or_else(|e| panic!("{}: slim schema is not a JSON Schema: {e}", t.name));
        let calls = calls(c);
        assert!(!calls.is_empty(), "{}: no minimal call for {c}", t.name);
        for call in &calls {
            let errors: Vec<String> = validator.iter_errors(call).map(|e| e.to_string()).collect();
            assert!(
                errors.is_empty(),
                "{}: slim schema rejects {call}: {errors:?}",
                t.name
            );
            decode(c, call).unwrap_or_else(|e| panic!("{}: {call} does not decode: {e}", t.name));
        }
        assert!(
            slim.get("title").is_none() && slim.get("description").is_none(),
            "{}",
            t.name
        );
    }
    eprintln!("total {full_total} -> {slim_total} bytes");
    assert!(slim_total * 10 < full_total * 6, "less than 40% saved");

    // Fields the host writes and refuses in a submission are not offered to the model.
    let answer = input_schema("AnswerSubmission").unwrap().to_string();
    assert!(!answer.contains("checked_by") && !answer.contains("\"resolved\""));
    // Required fields and enums stay.
    let s = input_schema("AnswerSubmission").unwrap();
    assert_eq!(s["required"], json!(["status", "blocks"]));
    let kinds = &s["properties"]["claims"]["items"]["properties"]["kind"]["enum"];
    let full = full_input_schema("AnswerSubmission").unwrap();
    let declared: Vec<Value> = full["properties"]["claims"]["items"]["properties"]["kind"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["const"].clone())
        .collect();
    assert_eq!(
        kinds.as_array().unwrap(),
        &declared,
        "every claim kind stays"
    );
}

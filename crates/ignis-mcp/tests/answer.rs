//! `answer_submit` (M7) through the tool host: a submission becomes a stored `ResearchAnswer` and
//! `answer.completed` only when every citation resolves against what *its own episode's* tools
//! returned, every pointer resolves to a value with a unit, every release is pinned, and (M7b) every
//! stated number equals the cell it is bound to in the pinned artifact. Otherwise one refusal lists
//! all the problems.

mod common;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use common::*;
use ignis_core::contracts::*;
use ignis_core::services::{ArtifactDraft, ArtifactStore};
use ignis_research::store::{JobPriority, NewJob, Replay};
use serde_json::{json, Value};

const RELEASE: &str = "sha256:ddc6469cf4445fb618716adf05b39a34f454000fe114a785f6e1adb3c19094e9";

struct World {
    h: Host,
    host: ignis_mcp::ToolHost,
    commits: Arc<AtomicU64>,
    sc: ExecutionScope,
    ep: EpisodeId,
    observed: ArtifactId,
    forecast: ArtifactRef,
    conditions: ArtifactId,
}

fn put_ref(
    h: &Host,
    sc: &ExecutionScope,
    contract: Option<&str>,
    produced_by: Producer,
    v: Value,
) -> ArtifactRef {
    let bytes = serde_json::to_vec(&v).unwrap();
    h.artifacts
        .put(
            sc,
            ArtifactDraft {
                media_type: "application/json".into(),
                contract: contract.map(str::to_string),
                produced_by,
                inputs: vec![],
                origin: Origin::SyntheticFixture,
            },
            &mut bytes.as_slice(),
        )
        .unwrap()
}

fn put(
    h: &Host,
    sc: &ExecutionScope,
    contract: &str,
    release: Option<&str>,
    v: Value,
) -> ArtifactId {
    let producer = Producer {
        release_id: release.map(|r| ReleaseId::new(r).unwrap()),
        ..Default::default()
    };
    put_ref(h, sc, Some(contract), producer, v).artifact_id
}

fn episode(h: &Host, sc: &ExecutionScope, crid: &str) -> EpisodeId {
    let pins = EpisodePins {
        model_releases: vec![ReleaseId::new(RELEASE).unwrap()],
        ..Default::default()
    };
    h.store
        .create_episode(
            sc,
            "Synthetic fixture goal",
            &ClientRequestId::new(crid).unwrap(),
            pins,
            Origin::SyntheticFixture,
        )
        .unwrap()
        .0
        .episode_id
}

fn world() -> World {
    let h = host();
    let commits = Arc::new(AtomicU64::new(0));
    let c = commits.clone();
    // A second host over the same store and artifacts, with a commit hook (as `ignis serve` wires).
    let host = ignis_mcp::ToolHost::new(
        ignis_mcp::ToolServices {
            knowledge: Some(h.knowledge.clone()),
            ..Default::default()
        },
        h.store.clone(),
        h.artifacts.clone(),
        Origin::SyntheticFixture,
        Arc::new(|| {}),
    )
    .with_commit_hook(Arc::new(move |_, seq| c.store(seq, Ordering::SeqCst)))
    // These tests read grounding problems in refusals; the log mode has its own test.
    .with_grounding(GroundingMode::Enforce);
    let sc = scope("ws_answer");
    let ep = episode(&h, &sc, "crid-episode-a");
    let observed = put(
        &h,
        &sc,
        "DatasetSlice",
        None,
        json!({"times_s": [0.0, 1.0, 2.0, 3.0], "values": [1.0, 0.9, 0.8, 0.7],
               "unit": "m", "time_origin": "ignition"}),
    );
    let forecast = put_ref(
        &h,
        &sc,
        Some("AnalysisArtifact"),
        Producer {
            release_id: Some(ReleaseId::new(RELEASE).unwrap()),
            ..Default::default()
        },
        json!({"times_s": [0.0, 1.0, 2.0, 3.0], "mean": [1.0, 0.88, 0.79, 0.69],
               "quantiles": [[0.9, 0.8, 0.7, 0.6], [1.0, 0.88, 0.79, 0.69], [1.1, 0.95, 0.85, 0.75]],
               "unit": "m", "time_origin": "ignition"}),
    );
    let conditions = put(&h, &sc, "ConditionsTable", None, json!({"rows": []}));
    World {
        h,
        host,
        commits,
        sc,
        ep,
        observed,
        forecast,
        conditions,
    }
}

/// The fixture submission rebound to this world's artifacts (and the forecast's digest); both
/// evidence citations name the evidence the fixture search returns.
fn submission(w: &World) -> Value {
    let mut s: Value = serde_json::to_value(fixture::<AnswerSubmission>(
        "research/answer_submission.json",
    ))
    .unwrap();
    let text = s
        .to_string()
        .replace("fixture-artifact-observed-01", w.observed.as_str())
        .replace(
            "fixture-artifact-forecast-01",
            w.forecast.artifact_id.as_str(),
        )
        .replace(
            "sha256:f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1",
            w.forecast.sha256.as_str(),
        )
        .replace("fixture-artifact-conditions-01", w.conditions.as_str())
        .replace("fixture-evidence-02", "fixture-evidence-01");
    s = serde_json::from_str(&text).unwrap();
    s
}

fn events(w: &World, ep: &EpisodeId) -> Vec<ResearchEvent> {
    match w.h.store.replay(&w.sc, ep, 0, 1000).unwrap() {
        Replay::Events { events, .. } => events,
        Replay::Gone { .. } => panic!("log gone"),
    }
}

fn answers(w: &World, ep: &EpisodeId) -> Vec<(AnswerId, ArtifactRef)> {
    events(w, ep)
        .into_iter()
        .filter_map(|e| match e.body {
            EventBody::AnswerCompleted {
                answer_id, answer, ..
            } => Some((answer_id, answer)),
            _ => None,
        })
        .collect()
}

fn refusal(w: &World, r: &ToolResult) -> AnswerRefusal {
    r.validate().unwrap();
    assert_eq!(r.execution_status, ExecutionStatus::Refused, "{r:#?}");
    assert_eq!(r.error.as_ref().unwrap().code, ErrorCode::InvalidRequest);
    let a = &r.artifact_refs[0];
    assert_eq!(a.contract.as_deref(), Some("AnswerRefusal"));
    let refusal: AnswerRefusal =
        serde_json::from_reader(w.h.artifacts.open(&w.sc, &a.artifact_id, 0, None).unwrap())
            .unwrap();
    refusal.validate().unwrap();
    refusal
}

fn kinds(r: &AnswerRefusal) -> Vec<AnswerProblemKind> {
    r.problems.iter().map(|p| p.kind).collect()
}

fn search(w: &World, ep: &EpisodeId) {
    let r = w
        .host
        .call_in(&w.sc, Some(ep), "sources_search", search_args());
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");
}

#[test]
fn an_answer_is_stored_only_once_its_citations_were_returned_in_its_episode() {
    let w = world();

    // Before any tool returned the evidence: refused, nothing stored, the reason is actionable.
    let r = w
        .host
        .call_in(&w.sc, Some(&w.ep), "answer_submit", submission(&w));
    let refused = refusal(&w, &r);
    assert_eq!(
        kinds(&refused),
        vec![AnswerProblemKind::UnknownEvidence; 3],
        "{refused:#?}"
    );
    assert!(r
        .error
        .as_ref()
        .unwrap()
        .message
        .contains("returned in this episode"));
    assert!(answers(&w, &w.ep).is_empty());

    // Evidence returned in *another* episode of the same workspace does not count.
    let other = episode(&w.h, &w.sc, "crid-episode-b");
    search(&w, &other);
    let r = w
        .host
        .call_in(&w.sc, Some(&w.ep), "answer_submit", submission(&w));
    assert!(kinds(&refusal(&w, &r)).contains(&AnswerProblemKind::UnknownEvidence));

    // Once this episode's search returned it, the answer is stored and announced.
    search(&w, &w.ep);
    let r = w
        .host
        .call_in(&w.sc, Some(&w.ep), "answer_submit", submission(&w));
    r.validate().unwrap();
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");
    let stored = answers(&w, &w.ep);
    assert_eq!(stored.len(), 1);
    let (answer_id, artifact) = &stored[0];
    assert_eq!(artifact.contract.as_deref(), Some("ResearchAnswer"));
    assert_eq!(r.artifact_refs, vec![artifact.clone()]);
    assert_eq!(
        w.commits.load(Ordering::SeqCst),
        events(&w, &w.ep).last().unwrap().seq,
        "readers were woken at the answer's seq"
    );
    let answer: ResearchAnswer = serde_json::from_reader(
        w.h.artifacts
            .open(&w.sc, &artifact.artifact_id, 0, None)
            .unwrap(),
    )
    .unwrap();
    answer.validate().unwrap();
    assert_eq!(&answer.answer_id, answer_id);
    assert_eq!(answer.episode_id, w.ep);
    assert_eq!(answer.status, AnswerStatus::Partial);
    // The host filled in the full references the agent only named.
    assert_eq!(answer.evidence.len(), 1);
    assert_eq!(
        answer.evidence[0].evidence_class,
        EvidenceClass::SourceDerived
    );
    assert_eq!(answer.artifacts.len(), 3);
    assert_eq!(
        r.result.as_ref().unwrap().value,
        serde_json::to_value(&answer).unwrap()
    );

    // A retry with the same key returns the same answer and appends nothing.
    let again = w
        .host
        .call_in(&w.sc, Some(&w.ep), "answer_submit", submission(&w));
    assert_eq!(again.execution_status, ExecutionStatus::Completed);
    assert_eq!(again.artifact_refs, vec![artifact.clone()]);
    assert_eq!(answers(&w, &w.ep).len(), 1);

    // The same key for a different answer is a conflict, not a silent overwrite.
    let mut changed = submission(&w);
    changed["limitations"] = json!(["A different limitation."]);
    let r = w.host.call_in(&w.sc, Some(&w.ep), "answer_submit", changed);
    assert_eq!(r.execution_status, ExecutionStatus::Refused);
    assert_eq!(r.error.as_ref().unwrap().code, ErrorCode::Conflict);
    assert_eq!(answers(&w, &w.ep).len(), 1);
}

/// Evidence-shaped objects inside an artifact (written by agent-authored analysis code, or another
/// episode's stored answer) never become citable by reading the artifact.
#[test]
fn reading_an_artifact_never_makes_its_evidence_citable() {
    let w = world();
    // The evidence the fixture search returns, as another episode saw it.
    let other = episode(&w.h, &w.sc, "crid-episode-b");
    let r = w
        .host
        .call_in(&w.sc, Some(&other), "sources_search", search_args());
    let mut found = Vec::new();
    fn walk(v: &Value, out: &mut Vec<Value>) {
        match v {
            Value::Object(o)
                if o.contains_key("evidence_id") && o.contains_key("evidence_class") =>
            {
                out.push(v.clone())
            }
            Value::Object(o) => o.values().for_each(|x| walk(x, out)),
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    walk(&r.result.as_ref().unwrap().value, &mut found);
    assert!(!found.is_empty(), "{r:#?}");
    // An artifact carrying those objects (as an analysis program could write), read in this episode.
    let laundered = put(
        &w.h,
        &w.sc,
        "AnalysisArtifact",
        None,
        json!({ "evidence": found }),
    );
    let r = w.host.call_in(
        &w.sc,
        Some(&w.ep),
        "artifacts_read",
        json!({"artifact_id": laundered.as_str(), "json_pointer": ""}),
    );
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");

    let r = w
        .host
        .call_in(&w.sc, Some(&w.ep), "answer_submit", submission(&w));
    let refused = refusal(&w, &r);
    assert!(
        kinds(&refused).contains(&AnswerProblemKind::UnknownEvidence),
        "{refused:#?}"
    );
    assert!(answers(&w, &w.ep).is_empty());
}

#[test]
fn a_refusal_lists_every_problem_at_once() {
    let w = world();
    search(&w, &w.ep);
    let no_unit = put(
        &w.h,
        &w.sc,
        "DatasetSlice",
        None,
        json!({"diameter": [1.0, 0.9]}),
    );
    let mut s = submission(&w);
    s["client_request_id"] = json!("crid-bad");
    // Numbers with no unit anywhere above them.
    s["claims"][4]["support"][0] =
        json!({"kind": "artifact_value", "artifact_id": no_unit.as_str(), "pointer": "/diameter"});
    // A pointer that does not resolve.
    s["claims"][2]["support"] = json!([{"kind": "artifact_value", "artifact_id": w.forecast.artifact_id.as_str(), "pointer": "/mean/99"}]);
    // A prediction read from an artifact that is not a model output.
    s["claims"].as_array_mut().unwrap().push(json!({
        "claim_id": "claim-mismatch", "kind": "predicted", "text": "Synthetic.",
        "support": [{"kind": "artifact_value", "artifact_id": w.observed.as_str(), "pointer": "/values/0"}]
    }));
    // A release the episode did not pin, and an artifact that does not exist.
    s["model_releases"].as_array_mut().unwrap().push(json!(
        "sha256:0000000000000000000000000000000000000000000000000000000000000000"
    ));
    s["blocks"]
        .as_array_mut()
        .unwrap()
        .push(json!({"type": "artifact", "artifact_id": "artifact-missing"}));

    let r = w.host.call_in(&w.sc, Some(&w.ep), "answer_submit", s);
    let refused = refusal(&w, &r);
    let k = kinds(&refused);
    for want in [
        AnswerProblemKind::UnresolvedPointer,
        AnswerProblemKind::UngroundedNumber,
        AnswerProblemKind::KindSupportMismatch,
        AnswerProblemKind::UnpinnedRelease,
        AnswerProblemKind::UnknownArtifact,
    ] {
        assert!(k.contains(&want), "{want:?} missing from {refused:#?}");
    }
    assert_eq!(
        k.iter()
            .filter(|k| **k == AnswerProblemKind::UnresolvedPointer)
            .count(),
        2,
        "{refused:#?}"
    );
    let msg = &r.error.as_ref().unwrap().message;
    assert!(
        msg.contains("no stated unit") && msg.contains("does not resolve"),
        "{msg}"
    );
    assert!(answers(&w, &w.ep).is_empty());
}

#[test]
fn answer_submit_needs_an_episode_bound_grant() {
    let w = world();
    let r = w.host.call(&w.sc, "answer_submit", submission(&w));
    r.validate().unwrap();
    assert_eq!(r.execution_status, ExecutionStatus::Refused);
    assert_eq!(r.error.as_ref().unwrap().code, ErrorCode::InvalidRequest);
    // An episode of another workspace is not found from this scope.
    let r = w.host.call_in(
        &scope("ws_other"),
        Some(&w.ep),
        "answer_submit",
        submission(&w),
    );
    assert_eq!(r.execution_status, ExecutionStatus::Refused);
    assert_eq!(r.error.as_ref().unwrap().code, ErrorCode::NotFound);
}

// ---------------------------------------------------------------- M7b: numeric grounding

/// A synthetic conditions table in row layout: runs keyed by `run_id`, cells keyed by `key`, each
/// value with its raw value, raw unit, SI value and SI unit.
fn conditions_table(corpus: &str) -> Value {
    json!({
        "keys": ["pressure", "oxygen_fraction"],
        "corpus_snapshot": corpus,
        "origin": "synthetic_fixture",
        "rows": [
            {"investigation_id": "FIXTURE", "run": {"run_id": "fixture-run-01"}, "cells": [
                {"status": "value", "key": "pressure",
                 "value": {"raw_value": 1.0, "raw_unit": "atm", "value_si": 101325.0, "unit_si": "Pa"}},
                {"status": "missing", "key": "oxygen_fraction", "reason": "Synthetic fixture: not reported."}
            ]},
            {"investigation_id": "FIXTURE", "run": {"run_id": "fixture-run-02"}, "cells": [
                {"status": "value", "key": "pressure",
                 "value": {"raw_value": 0.5, "raw_unit": "atm", "value_si": 50662.5, "unit_si": "Pa"}},
                {"status": "value", "key": "oxygen_fraction",
                 "value": {"raw_value": 21.0, "raw_unit": "%", "value_si": 0.21, "unit_si": "1"}}
            ]}
        ]
    })
}

const CORPUS: &str = "corpus-00112233445566aa";

/// A value bound to the `key` cell of run `run` in a conditions table.
fn cell(table: &ArtifactRef, run: &str, key: &str, column: &str, value: f64, unit: &str) -> Value {
    json!({
        "value": value, "unit": unit,
        "source": {
            "kind": "artifact_cell",
            "artifact_id": table.artifact_id.as_str(),
            "sha256": table.sha256.as_str(),
            "rows": [
                {"table": "/rows", "where": [{"field": "/run/run_id", "equals": run}]},
                {"table": "/cells", "where": [{"field": "/key", "equals": key}]}
            ],
            "column": column
        }
    })
}

/// One observed claim (cited to the fixture passage) stating `values`.
fn observed(values: Vec<Value>) -> Value {
    json!({
        "status": "complete",
        "blocks": [{"type": "text", "text": "Synthetic fixture.", "claim_ids": ["claim-p"]}],
        "claims": [{
            "claim_id": "claim-p", "kind": "observed",
            "text": "Synthetic fixture: run 01 burned at 101.325 kPa.",
            "support": [{"kind": "evidence", "evidence_id": "fixture-evidence-01"}],
            "values": values
        }]
    })
}

fn submit(w: &World, ep: &EpisodeId, s: Value) -> ToolResult {
    w.host.call_in(&w.sc, Some(ep), "answer_submit", s)
}

fn table(w: &World) -> ArtifactRef {
    put_ref(
        &w.h,
        &w.sc,
        Some("ConditionsTable"),
        Producer::default(),
        conditions_table(CORPUS),
    )
}

#[test]
fn a_stated_number_is_stored_only_when_it_equals_its_cell() {
    let w = world();
    search(&w, &w.ep);
    let table = table(&w);

    // Wrong transcription, unknown column, stale snapshot, ambiguous row, a missing cell, a unit of
    // another dimension, a positional pointer: one refusal names each, and nothing is stored.
    let mut stale = cell(
        &table,
        "fixture-run-01",
        "pressure",
        "/value",
        101325.0,
        "Pa",
    );
    stale["source"]["sha256"] = json!(w.forecast.sha256.as_str());
    let mut ambiguous = cell(
        &table,
        "fixture-run-01",
        "pressure",
        "/value",
        101325.0,
        "Pa",
    );
    ambiguous["source"]["rows"][0]["where"] =
        json!([{"field": "/investigation_id", "equals": "FIXTURE"}]);
    let mut s = observed(vec![
        cell(&table, "fixture-run-01", "pressure", "/value", 101.3, "kPa"),
        cell(
            &table,
            "fixture-run-01",
            "pressure",
            "/value/value_mean",
            101325.0,
            "Pa",
        ),
        stale,
        ambiguous,
        cell(
            &table,
            "fixture-run-01",
            "oxygen_fraction",
            "/value",
            0.21,
            "1",
        ),
        cell(&table, "fixture-run-01", "pressure", "/value", 101.325, "s"),
    ]);
    s["claims"].as_array_mut().unwrap().push(json!({
        "claim_id": "claim-pos", "kind": "computed", "text": "Synthetic.",
        "support": [{"kind": "artifact_value", "artifact_id": table.artifact_id.as_str(),
                     "pointer": "/rows/0/cells/0/value/value_si"}]
    }));
    s["blocks"][0]["claim_ids"] = json!(["claim-p", "claim-pos"]);
    let r = submit(&w, &w.ep, s);
    let refused = refusal(&w, &r);
    use AnswerProblemKind as K;
    let mut got = kinds(&refused);
    got.sort();
    let mut want = vec![
        K::ValueMismatch,
        K::UnknownColumn,
        K::StaleSnapshot,
        K::UnresolvedRow,
        K::UnknownColumn,
        K::UnitMismatch,
        K::UngroundedNumber,
    ];
    want.sort();
    assert_eq!(got, want, "{refused:#?}");
    let msg = &r.error.as_ref().unwrap().message;
    for needle in [
        "stated 101.3 kPa, but the cell holds 101325 Pa = 101.325 kPa",
        "no field \"value_mean\"",
        "re-read the artifact and bind its current digest",
        "2 rows of \"/rows\"",
        "no field \"value\"",
        "measures time",
        "move it to the claim's values",
    ] {
        assert!(msg.contains(needle), "{needle:?} missing from {msg}");
    }
    assert!(answers(&w, &w.ep).is_empty());

    // The same cells stated correctly: in another unit (compared in SI), rounded to stated
    // significant digits, as the raw value in its raw unit, and a fraction as a percentage.
    let mut rounded = cell(&table, "fixture-run-01", "pressure", "/value", 101.3, "kPa");
    rounded["significant_digits"] = json!(4);
    let r = submit(
        &w,
        &w.ep,
        observed(vec![
            cell(
                &table,
                "fixture-run-01",
                "pressure",
                "/value",
                101.325,
                "kPa",
            ),
            rounded,
            cell(
                &table,
                "fixture-run-01",
                "pressure",
                "/value/raw_value",
                1.0,
                "atm",
            ),
            cell(
                &table,
                "fixture-run-02",
                "oxygen_fraction",
                "/value/value_si",
                21.0,
                "%",
            ),
        ]),
    );
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");
    let stored = answers(&w, &w.ep);
    assert_eq!(stored.len(), 1);
    let answer: ResearchAnswer = serde_json::from_reader(
        w.h.artifacts
            .open(&w.sc, &stored[0].1.artifact_id, 0, None)
            .unwrap(),
    )
    .unwrap();
    answer.validate().unwrap();
    // The host recorded what each reference resolved to: the cell, in the unit the table states.
    let resolved: Vec<(f64, &str)> = answer.claims[0]
        .values
        .iter()
        .map(|v| {
            let r = v.resolved.as_ref().unwrap();
            assert_eq!(r.level, GroundingLevel::Cell);
            (r.value.get(), r.unit.as_str())
        })
        .collect();
    assert_eq!(
        resolved,
        vec![
            (101325.0, "Pa"),
            (101325.0, "Pa"),
            (1.0, "atm"),
            (0.21, "1")
        ]
    );
    let report = answer.grounding.as_ref().unwrap();
    assert_eq!(
        (report.mode, report.grounded, report.problems.len()),
        (GroundingMode::Enforce, 4, 0)
    );
    assert!(answer.artifacts.iter().any(|a| a == &table));
}

#[test]
fn a_table_built_on_another_corpus_snapshot_is_stale() {
    let w = world();
    let pins = EpisodePins {
        corpus_snapshot: Some(CorpusSnapshotId::new("corpus-ffeeddccbbaa9988").unwrap()),
        ..Default::default()
    };
    let ep =
        w.h.store
            .create_episode(
                &w.sc,
                "Synthetic fixture goal",
                &ClientRequestId::new("crid-episode-pinned").unwrap(),
                pins,
                Origin::SyntheticFixture,
            )
            .unwrap()
            .0
            .episode_id;
    search(&w, &ep);
    let table = table(&w);
    let r = submit(
        &w,
        &ep,
        observed(vec![cell(
            &table,
            "fixture-run-01",
            "pressure",
            "/value",
            101325.0,
            "Pa",
        )]),
    );
    let refused = refusal(&w, &r);
    assert_eq!(kinds(&refused), vec![AnswerProblemKind::StaleSnapshot]);
    assert!(refused.problems[0]
        .detail
        .contains("corpus-ffeeddccbbaa9988"));
}

/// A number written by `analysis_execute` is grounded twice: by its cell and by the run that
/// computed it, with exactly the inputs that run mounted.
#[test]
fn a_derived_number_cites_its_computation_and_inputs() {
    let w = world();
    search(&w, &w.ep);
    let job =
        w.h.store
            .submit_job(
                &w.sc,
                NewJob {
                    kind: "analysis_execute".into(),
                    input: json!({}),
                    episode_id: Some(w.ep.clone()),
                    client_request_id: None,
                    idempotent: true,
                    priority: JobPriority::Interactive,
                    causation: Causation::default(),
                },
            )
            .unwrap();
    let by_job = Producer {
        job_id: Some(job.job_id.clone()),
        ..Default::default()
    };
    let program = put_ref(
        &w.h,
        &w.sc,
        None,
        by_job.clone(),
        json!("synthetic program"),
    );
    let output = put_ref(
        &w.h,
        &w.sc,
        None,
        by_job.clone(),
        json!({"burn_rate": {"value": 0.4231, "unit": "mm2/s"}}),
    );
    let mut record: ExecutionRecord = fixture("research/execution_record.json");
    record.program = program;
    record.inputs = vec![MountedInput {
        name: "series.json".into(),
        artifact: w.h.artifacts.stat(&w.sc, &w.observed).unwrap(),
    }];
    record.stdout = CapturedStream {
        total_bytes: 0,
        truncated: false,
        artifact: None,
    };
    record.outputs = vec![AnalysisOutput {
        path: "burn_rate.json".into(),
        artifact: output.clone(),
    }];
    let record = put_ref(
        &w.h,
        &w.sc,
        Some("ExecutionRecord"),
        by_job,
        serde_json::to_value(&record).unwrap(),
    );

    let claim = |derivation: Option<Value>| {
        let mut v = json!({
            "value": 0.4231, "unit": "mm2/s",
            "source": {"kind": "artifact_cell", "artifact_id": output.artifact_id.as_str(),
                       "sha256": output.sha256.as_str(), "column": "/burn_rate"}
        });
        if let Some(d) = derivation {
            v["derivation"] = d;
        }
        json!({
            "status": "complete",
            "blocks": [{"type": "text", "text": "Synthetic fixture.", "claim_ids": ["claim-k"]}],
            "claims": [{"claim_id": "claim-k", "kind": "computed",
                        "text": "Synthetic fixture: burn-rate constant 0.4231 mm2/s.",
                        "support": [], "values": [v]}]
        })
    };
    let only = |r: &ToolResult| {
        let refused = refusal(&w, r);
        assert_eq!(
            kinds(&refused),
            vec![AnswerProblemKind::UngroundedDerivation],
            "{refused:#?}"
        );
        refused.problems[0].detail.clone()
    };
    let with = |computation: &ArtifactId, inputs: &[&ArtifactId]| {
        let inputs: Vec<&str> = inputs.iter().map(|a| a.as_str()).collect();
        claim(Some(
            json!({"computation": computation.as_str(), "inputs": inputs}),
        ))
    };

    // No derivation, a derivation missing the input, one naming an extra input, one naming
    // another artifact as the computation: each refused with what to fix.
    let d = only(&submit(&w, &w.ep, claim(None)));
    assert!(d.contains("cite that run's ExecutionRecord"), "{d}");
    let d = only(&submit(&w, &w.ep, with(&record.artifact_id, &[])));
    assert!(d.contains(&format!("missing: {}", w.observed)), "{d}");
    let d = only(&submit(
        &w,
        &w.ep,
        with(&record.artifact_id, &[&w.observed, &w.conditions]),
    ));
    assert!(d.contains(&format!("not mounted: {}", w.conditions)), "{d}");
    let d = only(&submit(&w, &w.ep, with(&w.conditions, &[&w.observed])));
    assert!(d.contains("not the ExecutionRecord"), "{d}");
    assert!(answers(&w, &w.ep).is_empty());

    // A derivation on a number no run computed is refused too.
    let table = table(&w);
    let mut v = cell(
        &table,
        "fixture-run-01",
        "pressure",
        "/value",
        101325.0,
        "Pa",
    );
    v["derivation"] =
        json!({"computation": record.artifact_id.as_str(), "inputs": [w.observed.as_str()]});
    let d = only(&submit(&w, &w.ep, observed(vec![v])));
    assert!(d.contains("not written by an analysis_execute run"), "{d}");

    // The computation and its exact inputs: stored, and the answer lists all three artifacts.
    let r = submit(&w, &w.ep, with(&record.artifact_id, &[&w.observed]));
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");
    let answer: ResearchAnswer =
        serde_json::from_value(r.result.as_ref().unwrap().value.clone()).unwrap();
    let ids: Vec<&ArtifactId> = answer.artifacts.iter().map(|a| &a.artifact_id).collect();
    for want in [&output.artifact_id, &record.artifact_id, &w.observed] {
        assert!(ids.contains(&want), "{want} missing from {ids:?}");
    }
}

/// Under `log` (the default) an answer whose numbers do not ground is stored with each problem
/// in its grounding report, and the host still shows the cell's value; other problems still refuse.
#[test]
fn the_log_mode_stores_the_answer_and_records_what_did_not_ground() {
    let w = world();
    let host = ignis_mcp::ToolHost::new(
        ignis_mcp::ToolServices {
            knowledge: Some(w.h.knowledge.clone()),
            ..Default::default()
        },
        w.h.store.clone(),
        w.h.artifacts.clone(),
        Origin::SyntheticFixture,
        Arc::new(|| {}),
    );
    search(&w, &w.ep);
    let table = table(&w);
    let r = host.call_in(
        &w.sc,
        Some(&w.ep),
        "answer_submit",
        observed(vec![
            cell(&table, "fixture-run-01", "pressure", "/value", 101.3, "kPa"),
            cell(
                &table,
                "fixture-run-01",
                "pressure",
                "/value/value_mean",
                101325.0,
                "Pa",
            ),
        ]),
    );
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");
    let answer: ResearchAnswer =
        serde_json::from_value(r.result.as_ref().unwrap().value.clone()).unwrap();
    answer.validate().unwrap();
    let report = answer.grounding.as_ref().unwrap();
    assert_eq!((report.mode, report.grounded), (GroundingMode::Log, 0));
    let kinds: Vec<_> = report.problems.iter().map(|p| p.kind).collect();
    assert_eq!(
        kinds,
        vec![
            AnswerProblemKind::ValueMismatch,
            AnswerProblemKind::UnknownColumn
        ]
    );
    let v = &answer.claims[0].values;
    assert_eq!(
        v[0].resolved
            .as_ref()
            .map(|r| (r.value.get(), r.unit.as_str())),
        Some((101325.0, "Pa")),
        "the cell is shown, not the misstated value"
    );
    assert!(v[1].resolved.is_none());

    // A non-grounding problem still refuses under log.
    let mut s = observed(vec![]);
    s["claims"][0]["support"] = json!([]);
    let r = host.call_in(&w.sc, Some(&w.ep), "answer_submit", s);
    assert_eq!(r.execution_status, ExecutionStatus::Refused);
}

/// A number quoted from a PDF passage is grounded at text level: it must be written, with its
/// unit, in the quote a retrieval tool returned in this episode.
#[test]
fn a_quoted_number_must_be_written_in_the_returned_quote() {
    let w = world();
    *w.h.knowledge.quote.lock().unwrap() =
        Some("Synthetic fixture passage: the droplet (d0 = 3.05 mm) burned for 12.4 s.".into());
    search(&w, &w.ep);
    let quoted = |value: f64, unit: &str| {
        json!({"value": value, "unit": unit,
               "source": {"kind": "evidence_quote", "evidence_id": "fixture-evidence-01"}})
    };
    let r = submit(
        &w,
        &w.ep,
        observed(vec![quoted(3.5, "mm"), quoted(12.4, "ms")]),
    );
    let refused = refusal(&w, &r);
    assert_eq!(
        kinds(&refused),
        vec![
            AnswerProblemKind::ValueMismatch,
            AnswerProblemKind::UnitMismatch
        ],
        "{refused:#?}"
    );
    assert!(refused.problems[0]
        .detail
        .contains("numbers in it: 3.05, 12.4"));

    let r = submit(
        &w,
        &w.ep,
        observed(vec![quoted(3.05, "mm"), quoted(12.4, "s")]),
    );
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");
    let answer: ResearchAnswer =
        serde_json::from_value(r.result.as_ref().unwrap().value.clone()).unwrap();
    for v in &answer.claims[0].values {
        assert_eq!(v.resolved.as_ref().unwrap().level, GroundingLevel::Text);
    }
    assert_eq!(answer.grounding.as_ref().unwrap().grounded, 2);
}

/// The C0 conditions table behind a snapshot the test can move (M11's `conditions_snapshot`).
struct Conditions {
    snapshot: std::sync::Mutex<Digest>,
}

impl ignis_core::services::DatasetService for Conditions {
    fn dataset(
        &self,
        _: &ExecutionScope,
        id: &DatasetId,
    ) -> ignis_core::services::ServiceResult<DatasetBundle> {
        Err(ErrorEnvelope::new(
            ErrorCode::NotFound,
            format!("{id}"),
            RetryPolicy::Never,
        ))
    }
    fn conditions(
        &self,
        _: &ExecutionScope,
        _: &ConditionsQuery,
    ) -> ignis_core::services::ServiceResult<ConditionsTable> {
        Ok(fixture("data/conditions_table.json"))
    }
    fn conditions_snapshot(
        &self,
        _: &ExecutionScope,
    ) -> ignis_core::services::ServiceResult<Digest> {
        Ok(self.snapshot.lock().unwrap().clone())
    }
}

const SNAP_A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SNAP_B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn condition(run: &str, key: &str, value: f64, unit: &str) -> Value {
    json!({"value": value, "unit": unit,
           "source": {"kind": "conditions_cell", "snapshot": SNAP_A,
                      "investigation_id": "fixture-investigation-a", "run_id": run, "key": key}})
}

/// A number read from the experimental conditions resolves against the conditions service under
/// the snapshot it was read with: the run's key cell, a conflicting cell's candidate by locator,
/// never a missing cell.
#[test]
fn a_condition_resolves_against_the_conditions_snapshot() {
    let w = world();
    let datasets = Arc::new(Conditions {
        snapshot: std::sync::Mutex::new(Digest::parse(SNAP_A).unwrap()),
    });
    let host = ignis_mcp::ToolHost::new(
        ignis_mcp::ToolServices {
            knowledge: Some(w.h.knowledge.clone()),
            datasets: Some(datasets.clone()),
            ..Default::default()
        },
        w.h.store.clone(),
        w.h.artifacts.clone(),
        Origin::SyntheticFixture,
        Arc::new(|| {}),
    )
    .with_grounding(GroundingMode::Enforce);
    search(&w, &w.ep);
    let submit = |s: Value| host.call_in(&w.sc, Some(&w.ep), "answer_submit", s);
    let candidate = json!({"asset_sha256": "2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e",
                           "path": "fixture-experiment-table.csv", "row": 5});

    // A conflicting cell without a locator, a missing cell, a misread value, a condition stated
    // as a prediction: each named.
    let mut s = observed(vec![
        condition("fixture-run-02", "oxygen_mole_fraction", 0.4, "1"),
        condition("fixture-run-01", "gravity_level", 1.0, "g0"),
        condition("fixture-run-01", "pressure", 101.3, "kPa"),
    ]);
    s["claims"].as_array_mut().unwrap().push(json!({
        "claim_id": "claim-pred", "kind": "predicted", "text": "Synthetic.",
        "values": [condition("fixture-run-01", "pressure", 101352.9, "Pa")]
    }));
    s["blocks"][0]["claim_ids"] = json!(["claim-p", "claim-pred"]);
    let r = submit(s);
    let refused = refusal(&w, &r);
    use AnswerProblemKind as K;
    let mut got = kinds(&refused);
    got.sort();
    let mut want = vec![
        K::UnresolvedRow,
        K::UnknownColumn,
        K::ValueMismatch,
        K::KindSupportMismatch,
    ];
    want.sort();
    assert_eq!(got, want, "{refused:#?}");
    let msg = &r.error.as_ref().unwrap().message;
    assert!(msg.contains("pick a candidate by its locator"), "{msg}");
    assert!(msg.contains("holds no value"), "{msg}");

    // The same cells bound correctly: SI rounded to stated digits, the raw value as written, and
    // the conflicting cell's candidate picked by its locator.
    let mut rounded = condition("fixture-run-01", "pressure", 101.4, "kPa");
    rounded["significant_digits"] = json!(4);
    let mut raw = condition("fixture-run-01", "pressure", 14.7, "psi");
    raw["source"]["reading"] = json!("raw");
    let mut picked = condition("fixture-run-02", "oxygen_mole_fraction", 40.0, "%");
    picked["source"]["locator"] = candidate;
    let good = observed(vec![rounded, raw, picked]);
    let r = submit(good.clone());
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");
    let answer: ResearchAnswer =
        serde_json::from_value(r.result.as_ref().unwrap().value.clone()).unwrap();
    let resolved: Vec<(f64, &str)> = answer.claims[0]
        .values
        .iter()
        .map(|v| {
            let r = v.resolved.as_ref().unwrap();
            (r.value.get(), r.unit.as_str())
        })
        .collect();
    assert_eq!(resolved, vec![(101352.9, "Pa"), (14.7, "psi"), (0.4, "1")]);

    // Once the conditions move to another snapshot, the same binding is stale.
    *datasets.snapshot.lock().unwrap() = Digest::parse(SNAP_B).unwrap();
    let mut again = good;
    again["client_request_id"] = json!("crid-after-snapshot-moved");
    let refused = refusal(&w, &submit(again));
    assert_eq!(kinds(&refused), vec![K::StaleSnapshot; 3], "{refused:#?}");
}

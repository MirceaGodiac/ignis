//! `answer_submit` (M7) through the tool host: a submission becomes a stored `ResearchAnswer` and
//! `answer.completed` only when every citation resolves against what *its own episode's* tools
//! returned, every pointer resolves to a value with a unit, and every release is pinned. Otherwise
//! one refusal lists all the problems.

mod common;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use common::*;
use ignis_core::contracts::*;
use ignis_core::services::{ArtifactDraft, ArtifactStore};
use ignis_research::store::Replay;
use serde_json::{json, Value};

const RELEASE: &str = "sha256:ddc6469cf4445fb618716adf05b39a34f454000fe114a785f6e1adb3c19094e9";

struct World {
    h: Host,
    host: ignis_mcp::ToolHost,
    commits: Arc<AtomicU64>,
    sc: ExecutionScope,
    ep: EpisodeId,
    observed: ArtifactId,
    forecast: ArtifactId,
    conditions: ArtifactId,
}

fn put(
    h: &Host,
    sc: &ExecutionScope,
    contract: &str,
    release: Option<&str>,
    v: Value,
) -> ArtifactId {
    let bytes = serde_json::to_vec(&v).unwrap();
    h.artifacts
        .put(
            sc,
            ArtifactDraft {
                media_type: "application/json".into(),
                contract: Some(contract.into()),
                produced_by: Producer {
                    release_id: release.map(|r| ReleaseId::new(r).unwrap()),
                    ..Default::default()
                },
                inputs: vec![],
                origin: Origin::SyntheticFixture,
            },
            &mut bytes.as_slice(),
        )
        .unwrap()
        .artifact_id
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
    .with_commit_hook(Arc::new(move |_, seq| c.store(seq, Ordering::SeqCst)));
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
    let forecast = put(
        &h,
        &sc,
        "AnalysisArtifact",
        Some(RELEASE),
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

/// The fixture submission rebound to this world's artifacts; both evidence citations name the
/// evidence the fixture search returns.
fn submission(w: &World) -> Value {
    let mut s: Value = serde_json::to_value(fixture::<AnswerSubmission>(
        "research/answer_submission.json",
    ))
    .unwrap();
    let text = s
        .to_string()
        .replace("fixture-artifact-observed-01", w.observed.as_str())
        .replace("fixture-artifact-forecast-01", w.forecast.as_str())
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
    // A number with no unit anywhere above it.
    s["claims"][4]["support"][0] = json!({"kind": "artifact_value", "artifact_id": no_unit.as_str(), "pointer": "/diameter/1"});
    // A pointer that does not resolve.
    s["claims"][2]["support"][0]["pointer"] = json!("/mean/99");
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

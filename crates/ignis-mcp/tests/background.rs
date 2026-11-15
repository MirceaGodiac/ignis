//! `answer_submit` with the model's own knowledge: `background` claims are stored next to grounded
//! ones, labelled and recorded as unverified by the host (a number in their text is never shown as
//! grounded), and one that names a run of a validation or test partition is refused, because a
//! run's outcome comes only from what the tools return.

mod common;

use std::collections::BTreeSet;
use std::sync::Arc;

use common::*;
use ignis_core::contracts::*;
use ignis_core::services::{ArtifactStore, DatasetService, ServiceResult};
use serde_json::{json, Value};

const SEALED_RUN: &str = "PSI-69:053F001:2010-05-05";
const TRAIN_RUN: &str = "PSI-69:175F001:2010-03-29";

/// Datasets that only answer which partition a run holds.
struct Splits;

impl DatasetService for Splits {
    fn dataset(&self, _: &ExecutionScope, _: &DatasetId) -> ServiceResult<DatasetBundle> {
        Ok(fixture("data/dataset_bundle.json"))
    }
    fn run_partitions(
        &self,
        _: &ExecutionScope,
        run: &RunId,
    ) -> ServiceResult<BTreeSet<Partition>> {
        Ok(match run.as_str() {
            SEALED_RUN => [Partition::Validation].into(),
            TRAIN_RUN => [Partition::Train].into(),
            _ => BTreeSet::new(),
        })
    }
}

fn submission(background_run: &str) -> Value {
    json!({
        "status": "complete",
        "blocks": [{"type": "text", "text": "Synthetic fixture: a mixed answer.",
                    "claim_ids": ["c-source", "c-background", "c-number"]}],
        "claims": [
            {"claim_id": "c-source", "kind": "source_reports",
             "text": "Synthetic fixture: the passage reports a trend.",
             "support": [{"kind": "evidence", "evidence_id": "fixture-evidence-01"}]},
            {"claim_id": "c-background", "kind": "background",
             "text": format!("Synthetic fixture: run {background_run} burned a heptane droplet.")},
            {"claim_id": "c-number", "kind": "background",
             "text": "Synthetic fixture: such tests ran at about 1 atm and 21% oxygen."}
        ],
        "limitations": ["Synthetic fixture: background claims are unverified model knowledge."]
    })
}

#[test]
fn background_claims_are_stored_unverified_and_never_name_a_sealed_run() {
    let h = host_full(None, Some(Arc::new(Splits) as Arc<dyn DatasetService>));
    let sc = scope("ws_background");
    let ep = h
        .store
        .create_episode(
            &sc,
            "Synthetic fixture goal",
            &ClientRequestId::new("crid-background").unwrap(),
            EpisodePins::default(),
            Origin::SyntheticFixture,
        )
        .unwrap()
        .0
        .episode_id;
    let r = h
        .host
        .call_in(&sc, Some(&ep), "sources_search", search_args());
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");

    // A background claim naming a validation run: refused, with what to do instead.
    let r = h
        .host
        .call_in(&sc, Some(&ep), "answer_submit", submission(SEALED_RUN));
    assert_eq!(r.execution_status, ExecutionStatus::Refused, "{r:#?}");
    let a = &r.artifact_refs[0];
    assert_eq!(a.contract.as_deref(), Some("AnswerRefusal"));
    let refusal: AnswerRefusal =
        serde_json::from_reader(h.artifacts.open(&sc, &a.artifact_id, 0, None).unwrap()).unwrap();
    assert_eq!(refusal.problems.len(), 1, "{refusal:#?}");
    let p = &refusal.problems[0];
    assert_eq!(p.kind, AnswerProblemKind::KindSupportMismatch);
    assert_eq!(p.claim_id.as_ref().unwrap().as_str(), "c-background");
    assert!(
        p.detail.contains(SEALED_RUN) && p.detail.contains("what the tools return"),
        "{}",
        p.detail
    );

    // Naming a training run is not a sealed outcome: the mixed answer is stored.
    let r = h
        .host
        .call_in(&sc, Some(&ep), "answer_submit", submission(TRAIN_RUN));
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");
    let a = &r.artifact_refs[0];
    assert_eq!(a.contract.as_deref(), Some("ResearchAnswer"));
    let answer: ResearchAnswer =
        serde_json::from_reader(h.artifacts.open(&sc, &a.artifact_id, 0, None).unwrap()).unwrap();
    answer.validate().unwrap();
    assert_eq!(answer.status, AnswerStatus::Complete);
    let kinds: Vec<ClaimKind> = answer.claims.iter().map(|c| c.kind).collect();
    assert_eq!(
        kinds,
        [
            ClaimKind::SourceReports,
            ClaimKind::Background,
            ClaimKind::Background
        ]
    );
    // The grounded claim keeps its evidence and no host verdict; every background claim, the one
    // with numbers in its text included, is recorded as unverified and binds no value.
    assert!(answer.claims[0].checked_by.is_none());
    assert_eq!(answer.evidence.len(), 1);
    for c in &answer.claims[1..] {
        let check = c.checked_by.as_ref().unwrap();
        assert_eq!(check.method, SupportCheckMethod::HostRule);
        assert_eq!(check.verdict, SupportVerdict::Unverifiable);
        assert!(c.support.is_empty() && c.values.is_empty());
    }
    assert!(
        answer.grounding.is_none(),
        "no number was bound, so none is reported as grounded"
    );

    // A background claim that tries to bind its number as grounded data is refused.
    let mut bound = submission(TRAIN_RUN);
    bound["claims"][2]["values"] = json!([{"value": 1.0, "unit": "atm",
        "source": {"kind": "evidence_quote", "evidence_id": "fixture-evidence-01"}}]);
    bound["client_request_id"] = json!("crid-bound");
    let r = h.host.call_in(&sc, Some(&ep), "answer_submit", bound);
    assert_eq!(r.execution_status, ExecutionStatus::Refused, "{r:#?}");
}

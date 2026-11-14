//! `answer_submit` (task M7): the host turns an [`AnswerSubmission`] into a stored
//! [`ResearchAnswer`] and appends `answer.completed`, or refuses with every problem it found.
//!
//! The agent names evidence, artifacts and releases only by id. The host resolves them:
//! - evidence against what this episode's tool results returned (the store's episode evidence),
//!   so an answer cannot cite a passage its episode never saw;
//! - artifacts against this workspace's artifact store, and every JSON Pointer against the
//!   artifact's bytes (a number must sit under an object that states its `unit`, so a value is never
//!   cited without one);
//! - releases against the episode's pins.
//!
//! Then the contract rules of the assembled answer run ([`ResearchAnswer::problems`]: evidence class
//! of observed claims, predictions only from `AnalysisArtifact`, view contracts). The refusal lists
//! every problem at once (an `AnswerRefusal` artifact plus the same list in the error message), so
//! one resubmission can fix them all. A retry with the same `client_request_id` returns the answer
//! stored the first time.

use std::collections::BTreeMap;

use ignis_core::contracts::{
    AnswerId, AnswerProblem, AnswerProblemKind, AnswerRefusal, AnswerSubmission, ArtifactId,
    ArtifactRef, Causation, ClaimSupport, EntityRef, EpisodeId, ErrorCode, ErrorEnvelope,
    EventBody, ExecutionScope, Origin, Producer, ResearchAnswer, ResearchSchema, RetryPolicy,
    Timestamp, ToolCallId, ViewSpec,
};
use ignis_core::hashing::sha256_hex;
use ignis_core::services::{ArtifactDraft, ArtifactStore, ServiceResult};
use ignis_research::ResearchStore;
use serde_json::Value;

/// What a submission became.
pub(crate) enum Submitted {
    Stored {
        answer: ResearchAnswer,
        artifact: ArtifactRef,
        /// The episode's last seq when `answer.completed` was appended now (the host wakes its
        /// readers); `None` when an earlier identical submission already holds.
        appended: Option<u64>,
    },
    Refused {
        error: ErrorEnvelope,
        refusal: ArtifactRef,
    },
}

pub(crate) struct AnswerHost<'a> {
    pub store: &'a ResearchStore,
    pub artifacts: &'a dyn ArtifactStore,
    pub origin: Origin,
}

fn internal(msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(ErrorCode::Internal, msg, RetryPolicy::Never)
}

/// Keys under which an artifact states the unit of the numbers below it.
const UNIT_KEYS: &[&str] = &["unit", "si_unit", "units"];

fn problem(kind: AnswerProblemKind, detail: String) -> AnswerProblem {
    AnswerProblem {
        kind,
        claim_id: None,
        block: None,
        detail,
    }
}

impl AnswerHost<'_> {
    pub fn submit(
        &self,
        scope: &ExecutionScope,
        episode: &EpisodeId,
        call_id: &ToolCallId,
        sub: AnswerSubmission,
    ) -> ServiceResult<Submitted> {
        let crid = sub.client_request_id.clone();
        let fingerprint = {
            let mut s = sub.clone();
            s.client_request_id = None;
            let text = serde_json::to_string(&s).map_err(|e| internal(e.to_string()))?;
            sha256_hex(text.as_bytes())
        };
        if let Some(crid) = &crid {
            if let Some(ev) = self
                .store
                .prior_answer(scope, episode, crid, &fingerprint)?
            {
                return self.stored_from(scope, ev.body, None);
            }
        }
        let ep = self.store.episode(scope, episode)?;
        let mut problems = sub.problems();

        // Evidence: only what this episode's tool results returned. Artifacts: this workspace.
        let cited_evidence: Vec<_> = sub.cited_evidence().into_iter().cloned().collect();
        let found = self
            .store
            .episode_evidence(scope, episode, cited_evidence.iter())?;
        let mut artifacts: BTreeMap<ArtifactId, ArtifactRef> = BTreeMap::new();
        for id in sub.cited_artifacts() {
            match self.artifacts.stat(scope, id) {
                Ok(a) => {
                    artifacts.insert(id.clone(), a);
                }
                Err(e) if e.code == ErrorCode::NotFound => {}
                Err(e) => return Err(e),
            }
        }
        for r in sub
            .model_releases
            .iter()
            .filter(|r| !ep.pins.model_releases.contains(r))
        {
            let pinned: Vec<&str> = ep.pins.model_releases.iter().map(|r| r.as_str()).collect();
            problems.push(problem(
                AnswerProblemKind::UnpinnedRelease,
                format!(
                    "release {r} is not pinned on episode {episode} (pinned: {}); cite only pinned releases",
                    if pinned.is_empty() { "none".into() } else { pinned.join(", ") }
                ),
            ));
        }
        problems.extend(self.pointer_problems(scope, &sub, &artifacts));

        let answer = ResearchAnswer {
            schema_version: ResearchSchema::V2,
            answer_id: answer_id(episode, crid.as_ref().map(|c| c.as_str()))?,
            episode_id: episode.clone(),
            status: sub.status,
            blocks: sub.blocks,
            claims: sub.claims,
            evidence: cited_evidence
                .iter()
                .filter_map(|id| found.get(id).cloned())
                .collect(),
            artifacts: artifacts.into_values().collect(),
            model_releases: sub.model_releases,
            limitations: sub.limitations,
            origin: self.origin,
            created_at: Timestamp::now(),
        };
        for p in answer.problems().into_iter().map(explain) {
            if !problems.contains(&p) {
                problems.push(p);
            }
        }
        if problems.is_empty() {
            if let Err(e) = answer.validate() {
                problems.push(problem(AnswerProblemKind::InvalidStructure, e.to_string()));
            }
        }
        let produced_by = Producer {
            episode_id: Some(episode.clone()),
            job_id: None,
            tool_call_id: Some(call_id.clone()),
            release_id: None,
        };
        if !problems.is_empty() {
            return self.refuse(scope, produced_by, problems);
        }

        let inputs = answer
            .evidence
            .iter()
            .map(|e| EntityRef::Evidence(e.evidence_id.clone()))
            .chain(
                answer
                    .artifacts
                    .iter()
                    .map(|a| EntityRef::Artifact(a.artifact_id.clone())),
            )
            .chain(
                answer
                    .model_releases
                    .iter()
                    .cloned()
                    .map(EntityRef::Release),
            )
            .collect();
        let artifact = self.put_json(
            scope,
            "ResearchAnswer",
            produced_by,
            inputs,
            &serde_json::to_vec(&answer).map_err(|e| internal(e.to_string()))?,
        )?;
        let (ev, new) = self.store.complete_answer(
            scope,
            episode,
            crid.as_ref().map(|c| (c, fingerprint.as_str())),
            Causation {
                tool_call_id: Some(call_id.clone()),
                client_request_id: crid.clone(),
                ..Default::default()
            },
            EventBody::AnswerCompleted {
                answer_id: answer.answer_id.clone(),
                status: answer.status,
                answer: artifact.clone(),
            },
        )?;
        if !new {
            // A concurrent identical submission was recorded first: return that one.
            return self.stored_from(scope, ev.body, None);
        }
        Ok(Submitted::Stored {
            answer,
            artifact,
            appended: Some(ev.seq),
        })
    }

    /// The answer an earlier `answer.completed` event stored.
    fn stored_from(
        &self,
        scope: &ExecutionScope,
        body: EventBody,
        appended: Option<u64>,
    ) -> ServiceResult<Submitted> {
        let EventBody::AnswerCompleted {
            answer: artifact, ..
        } = body
        else {
            return Err(internal("idempotency key points at a non-answer event"));
        };
        let r = self.artifacts.open(scope, &artifact.artifact_id, 0, None)?;
        let answer: ResearchAnswer = serde_json::from_reader(r)
            .map_err(|e| internal(format!("stored answer {}: {e}", artifact.artifact_id)))?;
        Ok(Submitted::Stored {
            answer,
            artifact,
            appended,
        })
    }

    fn refuse(
        &self,
        scope: &ExecutionScope,
        produced_by: Producer,
        problems: Vec<AnswerProblem>,
    ) -> ServiceResult<Submitted> {
        let refusal = AnswerRefusal {
            schema_version: ResearchSchema::V2,
            problems,
            origin: self.origin,
        };
        refusal.validate().map_err(ErrorEnvelope::from)?;
        let mut msg = format!(
            "answer refused: {} problem(s). Fix all of them and call answer_submit again (nothing was stored):",
            refusal.problems.len()
        );
        for p in &refusal.problems {
            let kind = serde_json::to_value(p.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default();
            let at = match (&p.claim_id, p.block) {
                (Some(c), _) => format!(" claim {c}"),
                (None, Some(b)) => format!(" block {b}"),
                (None, None) => String::new(),
            };
            msg.push_str(&format!("\n- [{kind}]{at}: {}", p.detail));
        }
        let artifact = self.put_json(
            scope,
            "AnswerRefusal",
            produced_by,
            vec![],
            &serde_json::to_vec(&refusal).map_err(|e| internal(e.to_string()))?,
        )?;
        Ok(Submitted::Refused {
            error: ErrorEnvelope::new(ErrorCode::InvalidRequest, msg, RetryPolicy::Never),
            refusal: artifact,
        })
    }

    fn put_json(
        &self,
        scope: &ExecutionScope,
        contract: &str,
        produced_by: Producer,
        inputs: Vec<EntityRef>,
        bytes: &[u8],
    ) -> ServiceResult<ArtifactRef> {
        self.artifacts.put(
            scope,
            ArtifactDraft {
                media_type: "application/json".into(),
                contract: Some(contract.into()),
                produced_by,
                inputs,
                origin: self.origin,
            },
            &mut &bytes[..],
        )
    }

    /// Every JSON Pointer of the submission, resolved against the artifact it names. Malformed
    /// pointers and unknown artifacts are reported elsewhere; here only well-formed pointers into
    /// existing artifacts are read.
    fn pointer_problems(
        &self,
        scope: &ExecutionScope,
        sub: &AnswerSubmission,
        artifacts: &BTreeMap<ArtifactId, ArtifactRef>,
    ) -> Vec<AnswerProblem> {
        use ignis_core::contracts::{is_json_pointer, AnswerBlock};
        let mut docs: BTreeMap<ArtifactId, Result<Value, String>> = BTreeMap::new();
        let mut doc = |id: &ArtifactId| -> Option<Result<Value, String>> {
            let a = artifacts.get(id)?;
            Some(
                docs.entry(id.clone())
                    .or_insert_with(|| self.read_json(scope, a))
                    .clone(),
            )
        };
        let mut out = Vec::new();
        for c in &sub.claims {
            for s in &c.support {
                let ClaimSupport::ArtifactValue {
                    artifact_id,
                    pointer,
                } = s
                else {
                    continue;
                };
                if !is_json_pointer(pointer) {
                    continue;
                }
                let Some(d) = doc(artifact_id) else { continue };
                if let Err(detail) = d.and_then(|d| resolve(&d, pointer, Want::Cited)) {
                    out.push(AnswerProblem {
                        kind: AnswerProblemKind::UnresolvedPointer,
                        claim_id: Some(c.claim_id.clone()),
                        block: None,
                        detail: format!("{artifact_id} {pointer:?}: {detail}"),
                    });
                }
            }
        }
        for (i, b) in sub.blocks.iter().enumerate() {
            let AnswerBlock::View { view, .. } = b else {
                continue;
            };
            let mut checks: Vec<(&ArtifactId, &str, Want)> = vec![];
            match view {
                ViewSpec::SeriesComparison { series } => {
                    for s in series {
                        let a = &s.artifact_id;
                        checks.extend([
                            (a, s.time_pointer.as_str(), Want::NumberArray),
                            (a, s.value_pointer.as_str(), Want::NumberArray),
                            (a, s.unit_pointer.as_str(), Want::Text),
                            (a, s.time_origin_pointer.as_str(), Want::Text),
                        ]);
                        if let Some(band) = &s.band {
                            checks.extend([
                                (a, band.lower_pointer.as_str(), Want::NumberArray),
                                (a, band.upper_pointer.as_str(), Want::NumberArray),
                            ]);
                        }
                    }
                }
                ViewSpec::ArtifactTable {
                    artifact_id,
                    pointer,
                } => checks.push((artifact_id, pointer.as_str(), Want::Table)),
                _ => {}
            }
            for (a, p, want) in checks {
                if !is_json_pointer(p) {
                    continue;
                }
                let Some(d) = doc(a) else { continue };
                if let Err(detail) = d.and_then(|d| resolve(&d, p, want)) {
                    out.push(AnswerProblem {
                        kind: AnswerProblemKind::UnresolvedPointer,
                        claim_id: None,
                        block: Some(i as u32),
                        detail: format!("{a} {p:?}: {detail}"),
                    });
                }
            }
        }
        out
    }

    fn read_json(&self, scope: &ExecutionScope, a: &ArtifactRef) -> Result<Value, String> {
        if !a.media_type.contains("json") {
            return Err(format!(
                "the artifact is {}, not JSON; pointers address JSON artifacts only",
                a.media_type
            ));
        }
        let r = self
            .artifacts
            .open(scope, &a.artifact_id, 0, None)
            .map_err(|e| format!("could not be read: {}", e.message))?;
        serde_json::from_reader(std::io::BufReader::new(r))
            .map_err(|e| format!("is not valid JSON: {e}"))
    }
}

/// What a pointer must resolve to.
#[derive(Clone, Copy)]
enum Want {
    /// A cited value: anything, but numbers need a stated unit.
    Cited,
    /// A (possibly nested) array of numbers or nulls.
    NumberArray,
    Text,
    /// An array or object.
    Table,
}

fn resolve(doc: &Value, pointer: &str, want: Want) -> Result<(), String> {
    let Some(v) = doc.pointer(pointer) else {
        return Err("does not resolve in the artifact".into());
    };
    match want {
        Want::Text if !v.is_string() => Err("must resolve to a string".into()),
        Want::Table if !(v.is_array() || v.is_object()) => {
            Err("must resolve to an array or an object".into())
        }
        Want::NumberArray if !is_number_array(v) => {
            Err("must resolve to an array of numbers".into())
        }
        Want::Cited if is_numeric(v) && !has_unit(doc, pointer, v) => Err(format!(
            "resolves to a number with no stated unit (no {} field on the value or an enclosing object); cite the field that carries its unit, or a dimensionless value stated as unit \"1\"",
            UNIT_KEYS.join("/")
        )),
        _ => Ok(()),
    }
}

fn is_number_array(v: &Value) -> bool {
    v.as_array().is_some_and(|a| {
        a.iter()
            .all(|x| x.is_number() || x.is_null() || is_number_array(x))
    })
}

fn is_numeric(v: &Value) -> bool {
    v.is_number() || is_number_array(v)
}

fn unit_of(v: &Value) -> bool {
    v.as_object().is_some_and(|o| {
        UNIT_KEYS.iter().any(|k| {
            o.get(*k)
                .and_then(Value::as_str)
                .is_some_and(|u| !u.trim().is_empty())
        })
    })
}

/// The value itself or one of the objects enclosing it states a unit.
fn has_unit(doc: &Value, pointer: &str, v: &Value) -> bool {
    if unit_of(v) {
        return true;
    }
    let mut p = pointer;
    while let Some(cut) = p.rfind('/') {
        p = &p[..cut];
        if doc.pointer(p).is_some_and(unit_of) {
            return true;
        }
    }
    false
}

/// Contract details name the problem; add what the agent should do about the resolution ones.
fn explain(mut p: AnswerProblem) -> AnswerProblem {
    match p.kind {
        AnswerProblemKind::UnknownEvidence => p.detail.push_str(
            " (cite only evidence a tool returned in this episode, e.g. a sources_search hit)",
        ),
        AnswerProblemKind::UnknownArtifact => {
            p.detail.push_str(" (no such artifact in this workspace)")
        }
        _ => {}
    }
    p
}

/// Stable under an idempotency key (a retry names the same answer), fresh otherwise.
fn answer_id(episode: &EpisodeId, crid: Option<&str>) -> ServiceResult<AnswerId> {
    let tail = match crid {
        Some(c) => sha256_hex(format!("{episode}\n{c}").as_bytes())[..24].to_string(),
        None => uuid::Uuid::new_v4().simple().to_string(),
    };
    AnswerId::new(format!("ans_{tail}")).map_err(ErrorEnvelope::from)
}

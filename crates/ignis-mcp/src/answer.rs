//! `answer_submit` (task M7): the host turns an [`AnswerSubmission`] into a stored
//! [`ResearchAnswer`] and appends `answer.completed`, or refuses with every problem it found.
//!
//! The agent names evidence, artifacts and releases only by id. The host resolves them:
//! - evidence against what this episode's tool results returned (the store's episode evidence),
//!   so an answer cannot cite a passage its episode never saw;
//! - artifacts against this workspace's artifact store, and every JSON Pointer against the
//!   artifact's bytes (a number must sit under an object that states its `unit`, so a value is never
//!   cited without one);
//! - releases against the episode's pins and the releases its own forecasts ran;
//! - every number a claim binds (task M7b) against its reference ([`crate::grounding`]): a table
//!   cell (the artifact's digest must be the pinned one, the row keys must select exactly one row;
//!   a number written by `analysis_execute` must also cite that run's `ExecutionRecord` and its
//!   exact inputs), or a quote a retrieval tool returned in this episode (text level only). The
//!   host records what each reference resolved to; a stated value must equal it in SI. A single
//!   number cited by pointer alone states nothing the host could check. Under
//!   [`GroundingMode::Log`] these grounding problems are recorded in the stored answer; under
//!   `Enforce` they refuse it.
//!
//! Then the contract rules of the assembled answer run ([`ResearchAnswer::problems`]: evidence class
//! of observed claims, predictions only from `AnalysisArtifact`, view contracts). The refusal lists
//! every problem at once (an `AnswerRefusal` artifact plus the same list in the error message), so
//! one resubmission can fix them all. A retry with the same `client_request_id` returns the answer
//! stored the first time.

use std::collections::{BTreeMap, BTreeSet};

use ignis_core::contracts::{
    AnswerId, AnswerProblem, AnswerProblemKind, AnswerRefusal, AnswerSubmission, ArtifactId,
    ArtifactRef, Causation, Claim, ClaimSupport, ConditionsQuery, ConditionsTable, DatasetSchema,
    Digest, EntityRef, EpisodeId, EpisodePins, ErrorCode, ErrorEnvelope, EventBody, EvidenceId,
    EvidenceRef, ExecutionOutcome, ExecutionRecord, ExecutionScope, Finite, GroundingLevel,
    GroundingMode, GroundingReport, Origin, Producer, ResearchAnswer, ResearchSchema,
    ResolvedValue, RetryPolicy, Timestamp, ToolCallId, ValueDerivation, ValueSource, ViewSpec,
};
use ignis_core::hashing::sha256_hex;
use ignis_core::services::{
    ArtifactDraft, ArtifactStore, DatasetService, KnowledgeService, ServiceResult,
};
use ignis_research::ResearchStore;
use serde_json::Value;

use crate::grounding;
use crate::tools::ANALYSIS_JOB;

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
    /// Whether a grounding problem refuses the answer or is only recorded in it (M7b).
    pub grounding: GroundingMode,
    /// Conditions of experimental runs, to read a `conditions_cell` back (absent: not served).
    pub datasets: Option<&'a dyn DatasetService>,
    /// The corpus, to read the passage of a cited evidence id whose recorded reference carries no
    /// quote (search hits never do, and `sources_read` returns segments, not references).
    pub knowledge: Option<&'a dyn KnowledgeService>,
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
                // Not in the workspace: it may be a view of a dataset bundle this scope reads (the
                // ids `datasets_inspect` returns), resolved through the dataset service.
                Err(e) if e.code == ErrorCode::NotFound => {
                    if let Some(v) = self.dataset_view_ref(scope, id) {
                        artifacts.insert(id.clone(), v);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        // A release is citable when it is pinned on the episode or when a forecast of this episode
        // ran it successfully (the agent cannot pin; the job record is the proof it was used).
        let ran = if sub
            .model_releases
            .iter()
            .any(|r| !ep.pins.model_releases.contains(r))
        {
            self.store
                .episode_job_releases(scope, episode, crate::tools::FORECAST_JOB)?
        } else {
            Default::default()
        };
        for r in sub
            .model_releases
            .iter()
            .filter(|r| !ep.pins.model_releases.contains(r) && !ran.contains(r.as_str()))
        {
            let pinned: Vec<&str> = ep.pins.model_releases.iter().map(|r| r.as_str()).collect();
            let used: Vec<&str> = ran.iter().map(String::as_str).collect();
            problems.push(problem(
                AnswerProblemKind::UnpinnedRelease,
                format!(
                    "release {r} is neither pinned on episode {episode} (pinned: {}) nor used by a successful models_forecast of this episode (used: {}); cite only those releases",
                    if pinned.is_empty() { "none".into() } else { pinned.join(", ") },
                    if used.is_empty() { "none".into() } else { used.join(", ") }
                ),
            ));
        }
        problems.extend(self.pointer_problems(scope, &sub, &artifacts));
        let mut claims = sub.claims;
        let (grounded, grounding_problems) =
            self.ground_values(scope, &ep.pins, &mut claims, &artifacts, &found)?;
        problems.extend(grounding_problems);
        problems.extend(self.background(scope, &mut claims));

        let answer = ResearchAnswer {
            schema_version: ResearchSchema::V2,
            answer_id: answer_id(episode, crid.as_ref().map(|c| c.as_str()))?,
            episode_id: episode.clone(),
            status: sub.status,
            blocks: sub.blocks,
            claims,
            evidence: cited_evidence
                .iter()
                .filter_map(|id| found.get(id).cloned())
                .collect(),
            artifacts: artifacts.into_values().collect(),
            model_releases: sub.model_releases,
            limitations: sub.limitations,
            grounding: None,
            origin: self.origin,
            created_at: Timestamp::now(),
        };
        let mut answer = answer;
        for p in answer.problems().into_iter().map(explain) {
            if !problems.contains(&p) {
                problems.push(p);
            }
        }
        // Grounding problems refuse under `enforce`; under `log` the answer records them.
        let logged: Vec<AnswerProblem> = match self.grounding {
            GroundingMode::Enforce => vec![],
            GroundingMode::Log => {
                let (g, rest) = problems.into_iter().partition(|p| p.kind.is_grounding());
                problems = rest;
                g
            }
        };
        let states_numbers = answer.claims.iter().any(|c| !c.values.is_empty());
        if states_numbers || !logged.is_empty() {
            answer.grounding = Some(GroundingReport {
                mode: self.grounding,
                grounded,
                problems: logged,
            });
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

    /// `background` claims (the model's own knowledge): the host records that it did not verify
    /// them (`host_rule` / `unverifiable`, so a number in their text is never shown as grounded)
    /// and refuses one that names a run of the validation or test partition, whose outcome may
    /// only come from what the tools return (model weights may have memorized published results
    /// of the runs the lockbox holds). Run ids always contain `:` (`PSI-69:053F001:2010-05-05`),
    /// so only such tokens are looked up; without a dataset service the check cannot run.
    fn background(&self, scope: &ExecutionScope, claims: &mut [Claim]) -> Vec<AnswerProblem> {
        use ignis_core::contracts::{
            ClaimKind, Partition, RunId, SupportCheck, SupportCheckMethod, SupportVerdict,
        };
        let mut out = Vec::new();
        for claim in claims
            .iter_mut()
            .filter(|c| c.kind == ClaimKind::Background)
        {
            claim.checked_by = Some(SupportCheck {
                method: SupportCheckMethod::HostRule,
                model_id: None,
                verdict: SupportVerdict::Unverifiable,
                confidence: None,
            });
            let Some(datasets) = self.datasets else {
                continue;
            };
            let tokens: BTreeSet<&str> = claim
                .text
                .split(|c: char| c.is_whitespace() || ",;()[]{}\"'`".contains(c))
                .map(|t| t.trim_matches(|c: char| c == '.' || c == ':'))
                .filter(|t| t.contains(':'))
                .collect();
            for t in tokens {
                let Ok(run) = RunId::new(t) else { continue };
                let Ok(parts) = datasets.run_partitions(scope, &run) else {
                    continue;
                };
                if let Some(p) = parts
                    .iter()
                    .find(|p| matches!(p, Partition::Validation | Partition::Test))
                {
                    out.push(AnswerProblem {
                        kind: AnswerProblemKind::KindSupportMismatch,
                        claim_id: Some(claim.claim_id.clone()),
                        block: None,
                        detail: format!(
                            "background claim {} names run {t} of the {p:?} partition: a run's outcome is never stated from the model's own knowledge; state only what the tools return about it (as an observed or source_reports claim with support), or drop the run from the claim",
                            claim.claim_id
                        ),
                    });
                }
            }
        }
        out
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
                if d.as_ref()
                    .is_ok_and(|d| d.pointer(pointer).is_some_and(Value::is_number))
                {
                    out.push(AnswerProblem {
                        kind: AnswerProblemKind::UngroundedNumber,
                        claim_id: Some(c.claim_id.clone()),
                        block: None,
                        detail: format!(
                            "{artifact_id} {pointer:?} is a single number cited by position without stating it; move it to the claim's values (stated value, unit, row keys, column) so the host can check it"
                        ),
                    });
                    continue;
                }
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

    /// Resolve every value reference (M7b) and record what it resolved to in `resolved`. Returns
    /// how many resolved without a problem, and the problems. Malformed bindings, unknown
    /// artifacts and unknown evidence are reported elsewhere and skipped here.
    fn ground_values(
        &self,
        scope: &ExecutionScope,
        pins: &EpisodePins,
        claims: &mut [Claim],
        artifacts: &BTreeMap<ArtifactId, ArtifactRef>,
        evidence: &BTreeMap<EvidenceId, EvidenceRef>,
    ) -> ServiceResult<(u32, Vec<AnswerProblem>)> {
        let mut docs: BTreeMap<ArtifactId, Result<Value, String>> = BTreeMap::new();
        // The conditions snapshot, read once per submission from the service (never the client).
        let mut conditions: Option<Result<(Digest, ConditionsTable), String>> = None;
        let mut out = Vec::new();
        let mut grounded = 0u32;
        for c in claims.iter_mut() {
            let claim_id = c.claim_id.clone();
            for (n, v) in c.values.iter_mut().enumerate() {
                if v.validate().is_err() {
                    continue;
                }
                let before = out.len();
                let mut push = |kind: AnswerProblemKind, detail: String| {
                    out.push(AnswerProblem {
                        kind,
                        claim_id: Some(claim_id.clone()),
                        block: None,
                        detail,
                    })
                };
                let stated = v
                    .value
                    .map(|x| format!(" ({} {})", x.get(), v.unit.as_deref().unwrap_or_default()))
                    .unwrap_or_default();
                match &v.source {
                    ValueSource::EvidenceQuote { evidence_id } => {
                        let Some(e) = evidence.get(evidence_id) else {
                            continue;
                        };
                        let (Some(x), Some(unit)) = (v.value, v.unit.as_deref()) else {
                            continue;
                        };
                        let at = format!("value {n}{stated} quoted from {evidence_id}");
                        // The id was returned in this episode; its passage text comes from the
                        // corpus under this scope (a withheld passage stays unreadable).
                        let passage = match (&e.quote, self.knowledge) {
                            (Some(q), _) => Ok(q.clone()),
                            (None, Some(k)) => k
                                .resolve_evidence(scope, evidence_id)
                                .map_err(|e| e.message)
                                .and_then(|r| {
                                    r.quote
                                        .ok_or_else(|| "the corpus returns no text".to_string())
                                }),
                            (None, None) => Err("no knowledge service is configured".to_string()),
                        };
                        let quote = match passage {
                            Ok(q) => q,
                            Err(why) => {
                                push(
                                    AnswerProblemKind::UngroundedNumber,
                                    format!(
                                        "{at}: the passage text could not be read ({why}); bind the number to a reference that resolves"
                                    ),
                                );
                                continue;
                            }
                        };
                        match grounding::find_in_quote(&quote, x.get(), unit) {
                            Ok(()) => {
                                v.resolved = Some(ResolvedValue {
                                    level: GroundingLevel::Text,
                                    value: x,
                                    unit: unit.trim().to_string(),
                                })
                            }
                            Err((kind, detail)) => push(kind, format!("{at}: {detail}")),
                        }
                    }
                    ValueSource::ConditionsCell {
                        snapshot,
                        investigation_id,
                        run_id,
                        key,
                        locator,
                        reading,
                    } => {
                        let at = format!(
                            "value {n}{stated} from the {key:?} condition of {investigation_id}"
                        );
                        let read = conditions
                            .get_or_insert_with(|| self.read_conditions(scope))
                            .as_ref();
                        let (current, table) = match read {
                            Ok(t) => t,
                            Err(e) => {
                                push(AnswerProblemKind::UnresolvedRow, format!("{at}: {e}"));
                                continue;
                            }
                        };
                        if current != snapshot {
                            push(
                                AnswerProblemKind::StaleSnapshot,
                                format!(
                                    "{at}: read under conditions snapshot {snapshot}, but the conditions are now {current}; query them again and bind the current snapshot"
                                ),
                            );
                            continue;
                        }
                        let cell = match grounding::conditions_cell(
                            table,
                            investigation_id,
                            run_id.as_ref(),
                            *key,
                            locator.as_ref(),
                            *reading,
                        ) {
                            Ok(cell) => cell,
                            Err((kind, detail)) => {
                                push(kind, format!("{at}: {detail}"));
                                continue;
                            }
                        };
                        v.resolved = Finite::new(cell.value).ok().map(|value| ResolvedValue {
                            level: GroundingLevel::Cell,
                            value,
                            unit: cell.unit.clone(),
                        });
                        if let (Some(x), Some(unit)) = (v.value, v.unit.as_deref()) {
                            if let Err((kind, detail)) =
                                grounding::compare(x.get(), unit, v.significant_digits, &cell)
                            {
                                push(kind, format!("{at}: {detail}"));
                            }
                        }
                    }
                    ValueSource::ArtifactCell {
                        artifact_id,
                        sha256,
                        rows,
                        column,
                    } => {
                        let Some(a) = artifacts.get(artifact_id) else {
                            continue;
                        };
                        let at = format!("value {n}{stated} from {artifact_id}");
                        if let Some(problem) =
                            self.derivation_problem(scope, a, v.derivation.as_ref(), artifacts)?
                        {
                            push(
                                AnswerProblemKind::UngroundedDerivation,
                                format!("{at}: {problem}"),
                            );
                        }
                        if &a.sha256 != sha256 {
                            push(
                                AnswerProblemKind::StaleSnapshot,
                                format!(
                                    "{at}: bound to snapshot {sha256}, but the artifact is {}; re-read the artifact and bind its current digest",
                                    a.sha256
                                ),
                            );
                            continue;
                        }
                        let cell = if a.media_type == PARQUET_MEDIA_TYPE {
                            match self.view_cell(scope, a, rows, column) {
                                Ok(cell) => cell,
                                Err((kind, detail)) => {
                                    push(kind, format!("{at}: {detail}"));
                                    continue;
                                }
                            }
                        } else {
                            let doc = docs
                                .entry(artifact_id.clone())
                                .or_insert_with(|| self.read_json(scope, a));
                            let doc = match doc {
                                Ok(d) => &*d,
                                Err(e) => {
                                    push(AnswerProblemKind::UnresolvedRow, format!("{at}: {e}"));
                                    continue;
                                }
                            };
                            if let (Some(pinned), Some(built)) = (
                                &pins.corpus_snapshot,
                                doc.get("corpus_snapshot").and_then(Value::as_str),
                            ) {
                                if pinned.as_str() != built {
                                    push(
                                    AnswerProblemKind::StaleSnapshot,
                                    format!(
                                        "{at}: the table was built from corpus snapshot {built}, the episode is pinned to {pinned}; rebuild it on the pinned snapshot"
                                    ),
                                );
                                    continue;
                                }
                            }
                            let cell = match grounding::read_cell(doc, rows, column) {
                                Ok(cell) => cell,
                                Err((kind, detail)) => {
                                    push(kind, format!("{at}: {detail}"));
                                    continue;
                                }
                            };
                            cell
                        };
                        // The cell is the truth the answer shows, whatever was stated.
                        v.resolved = Finite::new(cell.value).ok().map(|value| ResolvedValue {
                            level: GroundingLevel::Cell,
                            value,
                            unit: cell.unit.clone(),
                        });
                        if let (Some(x), Some(unit)) = (v.value, v.unit.as_deref()) {
                            if let Err((kind, detail)) =
                                grounding::compare(x.get(), unit, v.significant_digits, &cell)
                            {
                                push(kind, format!("{at}: {detail}"));
                            }
                        }
                    }
                }
                if out.len() == before && v.resolved.is_some() {
                    grounded += 1;
                }
            }
        }
        Ok((grounded, out))
    }

    /// The conditions the scope sees, every page, under one snapshot: read before and after the
    /// query, so a table that changed while it was read is not taken for either version.
    fn read_conditions(&self, scope: &ExecutionScope) -> Result<(Digest, ConditionsTable), String> {
        let Some(ds) = self.datasets else {
            return Err(
                "this host serves no experimental conditions (conditions_query is not configured)"
                    .into(),
            );
        };
        let msg = |e: ErrorEnvelope| format!("the conditions could not be read: {}", e.message);
        let before = ds.conditions_snapshot(scope).map_err(msg)?;
        let mut q = ConditionsQuery {
            schema_version: DatasetSchema::V2,
            investigation_ids: vec![],
            run_ids: vec![],
            keys: vec![],
            filters: vec![],
            include_unknown: true,
            include_envelopes: false,
            near: None,
            page_size: None,
            cursor: None,
        };
        let mut table = ds.conditions(scope, &q).map_err(msg)?;
        while let Some(cursor) = table.next_cursor.take() {
            q.cursor = Some(cursor);
            let page = ds.conditions(scope, &q).map_err(msg)?;
            table.rows.extend(page.rows);
            table.next_cursor = page.next_cursor;
        }
        let after = ds.conditions_snapshot(scope).map_err(msg)?;
        if before != after {
            return Err(format!(
                "the conditions changed while they were read ({before} then {after}); submit again"
            ));
        }
        Ok((after, table))
    }

    /// Whether the cell's artifact was computed by `analysis_execute`, and if so whether the
    /// derivation cites that run's record and exactly the inputs it mounted. `None` when it holds.
    fn derivation_problem(
        &self,
        scope: &ExecutionScope,
        cell: &ArtifactRef,
        derivation: Option<&ValueDerivation>,
        artifacts: &BTreeMap<ArtifactId, ArtifactRef>,
    ) -> ServiceResult<Option<String>> {
        let job = match &cell.produced_by.job_id {
            Some(j) => match self.store.job(scope, j) {
                Ok(job) => Some(job),
                Err(e) if e.code == ErrorCode::NotFound => None,
                Err(e) => return Err(e),
            },
            None => None,
        };
        let executed = job.as_ref().is_some_and(|j| j.kind == ANALYSIS_JOB);
        let id = &cell.artifact_id;
        let d = match (executed, derivation) {
            (false, None) => return Ok(None),
            (false, Some(_)) => {
                return Ok(Some(format!(
                    "{id} was not written by an analysis_execute run; drop the derivation"
                )))
            }
            (true, None) => {
                return Ok(Some(format!(
                    "{id} was computed by analysis_execute job {}; cite that run's ExecutionRecord as derivation.computation and every input it mounted as derivation.inputs",
                    job.map(|j| j.job_id.to_string()).unwrap_or_default()
                )))
            }
            (true, Some(d)) => d,
        };
        // An unknown computation is already an unknown artifact.
        let Some(comp) = artifacts.get(&d.computation) else {
            return Ok(None);
        };
        if comp.contract.as_deref() != Some("ExecutionRecord") {
            return Ok(Some(format!(
                "derivation.computation {} is {:?}, not the ExecutionRecord of the run",
                d.computation, comp.contract
            )));
        }
        let record: ExecutionRecord = match self.read_json(scope, comp).and_then(|v| {
            serde_json::from_value(v).map_err(|e| format!("is not an ExecutionRecord: {e}"))
        }) {
            Ok(r) => r,
            Err(e) => {
                return Ok(Some(format!(
                    "derivation.computation {}: {e}",
                    d.computation
                )))
            }
        };
        let wrote = record.outputs.iter().any(|o| &o.artifact.artifact_id == id)
            || record
                .stdout
                .artifact
                .as_ref()
                .is_some_and(|a| &a.artifact_id == id);
        if !wrote {
            return Ok(Some(format!(
                "the run recorded in {} did not write {id}; cite the ExecutionRecord of the run that did",
                d.computation
            )));
        }
        if record.outcome != (ExecutionOutcome::Exited { exit_code: 0 }) {
            return Ok(Some(format!(
                "the run recorded in {} ended {:?}; a failed run grounds no number",
                d.computation, record.outcome
            )));
        }
        let mounted: BTreeSet<&ArtifactId> = record
            .inputs
            .iter()
            .map(|m| &m.artifact.artifact_id)
            .collect();
        let cited: BTreeSet<&ArtifactId> = d.inputs.iter().collect();
        if mounted != cited {
            let list = |s: Vec<&&ArtifactId>| {
                if s.is_empty() {
                    "none".to_string()
                } else {
                    s.iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            };
            return Ok(Some(format!(
                "derivation.inputs must be exactly the run's mounted inputs; missing: {}; not mounted: {}",
                list(mounted.difference(&cited).collect()),
                list(cited.difference(&mounted).collect())
            )));
        }
        Ok(None)
    }

    /// The `ArtifactRef` of a view of a dataset bundle the scope reads, if `id` names one. Its
    /// contract is the bundle's (`DatasetBundle`): a view is one of a bundle's data tables.
    fn dataset_view_ref(&self, scope: &ExecutionScope, id: &ArtifactId) -> Option<ArtifactRef> {
        let ds = self.datasets?;
        let bundles = ds.datasets(scope).ok()?;
        bundles.iter().flat_map(|b| b.views.iter()).find_map(|v| {
            (v.artifact.artifact_id == *id).then(|| ArtifactRef {
                contract: Some("DatasetBundle".into()),
                ..v.artifact.clone()
            })
        })
    }

    /// The cell of a dataset view (a bundle's Parquet artifact) a value is bound to: exactly one
    /// row selector over the view itself (`table: ""`) whose keys name identity columns
    /// (`/run_id`, `/channel`, `/time_s`), and a column (`/value_si`, `/raw_number`, …). The row is
    /// read through `DatasetService::view_rows` at the pinned digest, never by position; its unit
    /// is the one the view states (`unit_si` for an SI column, `raw_unit` for a raw one, else the
    /// column's own unit).
    fn view_cell(
        &self,
        scope: &ExecutionScope,
        a: &ArtifactRef,
        rows: &[ignis_core::contracts::RowSelector],
        column: &str,
    ) -> Result<grounding::Cell, grounding::Problem> {
        use ignis_core::services::{ViewCell, ViewRowsQuery};
        use AnswerProblemKind as K;
        let Some(ds) = self.datasets else {
            return Err((
                K::UnresolvedRow,
                "this host serves no datasets, so a dataset view cell cannot be read".into(),
            ));
        };
        let one = |p: &str| {
            let t = p.strip_prefix('/')?;
            (!t.is_empty() && !t.contains('/')).then(|| t.replace("~1", "/").replace("~0", "~"))
        };
        let [sel] = rows else {
            return Err((
                K::UnresolvedRow,
                r#"a dataset view cell takes exactly one row selector: {"table": "", "where": [key columns]}"#.into(),
            ));
        };
        if !sel.table.is_empty() {
            return Err((
                K::UnresolvedRow,
                format!(
                    r#"the table of a dataset view is the view itself ("table": ""), not {:?}"#,
                    sel.table
                ),
            ));
        }
        let mut keys = Vec::new();
        for k in &sel.keys {
            let name = one(&k.field).ok_or_else(|| {
                (
                    K::UnresolvedRow,
                    format!(
                        r#"key {:?} must name one column of the view (e.g. "/run_id")"#,
                        k.field
                    ),
                )
            })?;
            keys.push((name, k.equals.clone()));
        }
        let col = one(column).ok_or_else(|| {
            (
                K::UnknownColumn,
                format!(r#"column {column:?} must name one column of the view (e.g. "/value_si")"#),
            )
        })?;
        let q = ViewRowsQuery {
            view_sha256: a.sha256.clone(),
            keys,
            columns: vec![col.clone()],
            with_unit_columns: true,
            limit: 2,
        };
        let found = ds.view_rows(scope, &q).map_err(|e| {
            let kind = if e.message.contains("no column") {
                K::UnknownColumn
            } else {
                K::UnresolvedRow
            };
            (kind, e.message)
        })?;
        let row = match (found.rows.as_slice(), found.truncated) {
            ([one], false) => one,
            ([], _) => {
                return Err((
                    K::UnresolvedRow,
                    format!("no row of {}.{} has those keys", found.dataset_id, found.view),
                ))
            }
            _ => {
                return Err((
                    K::UnresolvedRow,
                    format!(
                        "several rows of {}.{} have those keys; add key columns until exactly one matches",
                        found.dataset_id, found.view
                    ),
                ))
            }
        };
        let json = |c: Option<&ViewCell>| match c {
            Some(ViewCell::Number(x)) if x.is_finite() => serde_json::json!(x),
            Some(ViewCell::Text(t)) => Value::String(t.clone()),
            Some(ViewCell::Bool(b)) => Value::Bool(*b),
            _ => Value::Null,
        };
        let mut rec = serde_json::Map::new();
        rec.insert(col.clone(), json(row.get(&col)));
        if col.ends_with("_si") {
            rec.insert("unit_si".into(), json(row.get("unit_si")));
        } else if col.starts_with("raw_") {
            let u = json(row.get("raw_unit"));
            rec.insert("raw_unit".into(), u.clone());
            rec.insert("unit".into(), u);
        } else if let Some(u) = found
            .columns
            .iter()
            .find(|c| c.name == col)
            .and_then(|c| c.unit.clone())
            .or_else(|| match row.get("unit") {
                Some(ViewCell::Text(t)) => Some(t.clone()),
                _ => None,
            })
        {
            rec.insert("unit".into(), Value::String(u));
        }
        let ptr = format!("/{}", col.replace('~', "~0").replace('/', "~1"));
        grounding::read_cell(&Value::Object(rec), &[], &ptr)
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

/// Media type of a dataset bundle's views (`ignis_data::dataset::PARQUET`).
const PARQUET_MEDIA_TYPE: &str = "application/vnd.apache.parquet";

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

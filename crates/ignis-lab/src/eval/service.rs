//! `EvaluationService` implementation (T2): a request in, a deterministic `EvaluationResult` out.
//!
//! The evaluator reuses the scorers of this module (`scoring::common_spec`, `scoring::score_against`,
//! `bootstrap::cluster_bootstrap`); it does not re-implement any score.
//!
//! Order of work, failing closed before any forecast runs:
//! 1. request contract, evaluation scope and partition grant, purpose (no final test in T2: no
//!    qualification criteria are frozen yet), frozen protocol when one is referenced;
//! 2. dataset bundle identity (content digest, split, partition), releases and forward hashes,
//!    metric table;
//! 3. the case list, which carries no outcomes (run, group, cutoff);
//! 4. every forecast, through `ModelService`, with no continuation in memory;
//! 5. a commitment (artifact hash) per admitted artifact, then its continuation from the
//!    `CaseSource` (the only path to outcomes), then scoring.
//!
//! Case accounting (every case counted once, per subject):
//! * `failed`: the forecast call errored, the artifact is invalid or forecasts another
//!   release/quantity/unit, or an admitted artifact could not be scored (no continuation point inside
//!   its horizon, different time origin, no scorer). A model cannot avoid scoring by a short horizon.
//! * `refused`: the release's own support gate did not admit the case, or its status is
//!   unsupported / failed / insufficient inference.
//! * `scored`: everything else.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ignis_core::contracts::{
    AnalysisArtifact, CaseAccounting, CaseFailure, CaseFailureKind, DatasetBundle, Digest,
    ErrorCode, ErrorEnvelope, EvaluationId, EvaluationPurpose, EvaluationRequest, EvaluationResult,
    EvaluationSchema, ExecutionScope, Finite, ForecastInput, ForecastRequest, MetricValue,
    ModelRelease, Origin, Partition, Population, RetryPolicy, RunId, ScopeRole, Timestamp, Verdict,
    WspSchema,
};
use ignis_core::services::{DatasetService, EvaluationService, ModelService, ServiceResult};
use rayon::prelude::*;
use serde_json::Value;

use super::bootstrap::{cluster_bootstrap, clusters};
use super::protocol::FrozenProtocol;
use super::scoring::{common_spec, score_against, Case, Spec};
use super::vault::artifact_hash;
use crate::util::fmt_g;

/// Bootstrap replicates behind each point estimate (the estimate itself does not depend on them;
/// replicates are keyed by (seed, b), so the result is independent of the thread count).
const N_BOOT: usize = 1000;
const BOOT_SEED: u64 = 0;

// ------------------------------------------------------------------------------------- cases

/// One evaluation case, without its outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalCase {
    pub case_id: String,
    pub run_id: String,
    /// Independent group (split grouping key); cases of one group form one bootstrap cluster.
    pub group_id: Option<String>,
    pub cutoff_s: f64,
}

/// Proof that a forecast existed before its outcome was read.
#[derive(Debug, Clone, PartialEq)]
pub struct Commitment {
    pub release_id: String,
    pub artifact_hash: String,
}

/// The evaluator's only access to outcomes. A vault-backed implementation seals, reveals and logs;
/// a fixture implementation must refuse the test partition.
pub trait CaseSource: Send + Sync {
    /// Cases of `partition` at `cutoffs_s`, in a stable order. No outcomes.
    fn cases(
        &self,
        scope: &ExecutionScope,
        bundle: &DatasetBundle,
        partition: Partition,
        cutoffs_s: &[f64],
    ) -> ServiceResult<Vec<EvalCase>>;
    /// The continuation of `case` (vault JSON shape: `observations` with `time_s`, `value_si`,
    /// `quantity`, `valid`; `cutoff_s`; `time_origin`), only after a commitment.
    fn reveal(
        &self,
        scope: &ExecutionScope,
        case: &EvalCase,
        commitment: &Commitment,
    ) -> ServiceResult<Value>;
    /// `synthetic_fixture` when the outcomes are synthetic.
    fn origin(&self) -> Origin;
}

// ----------------------------------------------------------------------------------- metrics

/// Unit of a metric value relative to the target.
#[derive(Debug, Clone, Copy, PartialEq)]
enum MetricUnit {
    Target,
    Dimensionless,
    LogDensity,
}

/// Supported (metric id, version) → key in `Case.metrics` and unit. Anything else is unsupported.
fn metric_table(metric: &str, version: &str) -> Option<(String, MetricUnit)> {
    if version != "1" {
        return None;
    }
    match metric {
        "crps" | "mae_median" | "mae_mean" => Some((metric.to_string(), MetricUnit::Target)),
        "lpd" => Some(("lpd".into(), MetricUnit::LogDensity)),
        m => {
            // Interval coverage / width at a central level, e.g. "coverage@0.9". The key is built
            // with the same formatter as `scoring` (`fmt_g` of the nominal level), so "0.90" and
            // "0.9" name the same interval.
            let (kind, level) = m.split_once('@')?;
            let lv: f64 = level.parse().ok()?;
            if !(lv > 0.0 && lv < 1.0) {
                return None;
            }
            let key = format!("{kind}@{}", fmt_g(lv));
            match kind {
                "coverage" => Some((key, MetricUnit::Dimensionless)),
                "width" => Some((key, MetricUnit::Target)),
                _ => None,
            }
        }
    }
}

fn unit_label(u: MetricUnit, target_unit: &str) -> String {
    match u {
        MetricUnit::Target => target_unit.to_string(),
        MetricUnit::Dimensionless => "1".into(),
        MetricUnit::LogDensity => format!("log(1/{target_unit})"),
    }
}

// ----------------------------------------------------------------------------------- service

pub type Clock = Arc<dyn Fn() -> Timestamp + Send + Sync>;

pub struct Evaluator {
    pub datasets: Arc<dyn DatasetService>,
    pub models: Arc<dyn ModelService>,
    pub cases: Arc<dyn CaseSource>,
    /// Frozen protocols by (protocol_id, version) (`protocol::load_frozen`).
    pub protocols: BTreeMap<(String, String), FrozenProtocol>,
    pub clock: Clock,
}

fn err(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

fn finite(v: f64) -> Option<Finite> {
    Finite::new(v).ok()
}

/// Outcome of one (subject, case) before scoring.
enum Outcome {
    Failed(String),
    Refused(String),
    Admitted(AnalysisArtifact),
}

struct SubjectRun {
    release_id: String,
    outcomes: Vec<Outcome>,
}

impl Evaluator {
    fn check_scope(&self, scope: &ExecutionScope, partition: Partition) -> ServiceResult<()> {
        if scope.role != ScopeRole::Evaluation {
            return Err(err(
                ErrorCode::ScopeDenied,
                format!(
                    "scope {}: evaluation runs only in an evaluation scope",
                    scope.scope_id
                ),
            ));
        }
        scope
            .require_partition(partition)
            .map_err(ErrorEnvelope::from)
    }

    fn check_protocol(&self, req: &EvaluationRequest) -> ServiceResult<()> {
        let key = (
            req.protocol.protocol_id.clone(),
            req.protocol.version.clone(),
        );
        // A frozen id@version always binds the request, with or without `frozen_hash`: leaving the
        // hash out must not let changed content run under a frozen name (`check_request` refuses a
        // missing hash). Only a name that was never frozen is an unfrozen development protocol.
        let Some(frozen) = self.protocols.get(&key) else {
            return match &req.protocol.frozen_hash {
                None => Ok(()),
                Some(_) => Err(err(
                    ErrorCode::NotFound,
                    format!("protocol {}@{} is not frozen here", key.0, key.1),
                )),
            };
        };
        frozen
            .verify()
            .and_then(|_| frozen.protocol.check_request(req))
            .map_err(|m| err(ErrorCode::InvalidRequest, m))
    }

    fn check_bundle(&self, req: &EvaluationRequest, b: &DatasetBundle) -> ServiceResult<()> {
        b.validate()
            .map_err(|e| err(ErrorCode::InvalidRequest, e.to_string()))?;
        let digest = b.content_digest();
        if digest != req.dataset_hash {
            return Err(err(
                ErrorCode::Conflict,
                format!(
                    "dataset {} has digest {digest}, the request names {}",
                    req.dataset_id, req.dataset_hash
                ),
            ));
        }
        if b.split_id != req.split_id {
            return Err(err(
                ErrorCode::Conflict,
                format!(
                    "dataset split {} != request split {}",
                    b.split_id, req.split_id
                ),
            ));
        }
        if !b.partitions.contains(&req.partition) {
            return Err(err(
                ErrorCode::InvalidRequest,
                format!(
                    "dataset {} does not contain the {:?} partition",
                    req.dataset_id, req.partition
                ),
            ));
        }
        Ok(())
    }

    fn check_release(
        &self,
        scope: &ExecutionScope,
        m: &ignis_core::contracts::ModelUnderTest,
    ) -> ServiceResult<ModelRelease> {
        let r = self.models.release(scope, &m.release_id)?;
        // Fail closed: a requested forward identity must be proven by the release's manifest.
        match (
            &m.forward_hash,
            r.construction.as_ref().map(|c| &c.forward_hash),
        ) {
            (Some(want), Some(have)) if want != have => Err(err(
                ErrorCode::Conflict,
                format!(
                    "release {}: forward {have}, the request names {want}",
                    m.release_id
                ),
            )),
            (Some(want), None) => Err(err(
                ErrorCode::Conflict,
                format!(
                    "release {} has no construction manifest, so forward {want} cannot be verified",
                    m.release_id
                ),
            )),
            _ => Ok(r),
        }
    }

    fn forecast_one(
        &self,
        scope: &ExecutionScope,
        req: &EvaluationRequest,
        m: &ignis_core::contracts::ModelUnderTest,
        c: &EvalCase,
    ) -> Outcome {
        let fr = ForecastRequest {
            schema_version: WspSchema::default(),
            release_id: m.release_id.clone(),
            input: ForecastInput::StoredRun {
                run_id: match RunId::new(c.run_id.clone()) {
                    Ok(r) => r,
                    Err(e) => return Outcome::Failed(format!("run id: {e}")),
                },
                cutoff_s: match Finite::new(c.cutoff_s) {
                    Ok(f) => f,
                    Err(e) => return Outcome::Failed(format!("cutoff: {e}")),
                },
            },
            settings: m.settings.clone(),
            client_request_id: None,
        };
        let a = match self.models.forecast(scope, &fr) {
            Ok(a) => a,
            Err(e) => {
                return Outcome::Failed(format!("forecast error {:?}: {}", e.code, e.message))
            }
        };
        if let Err(e) = a.validate() {
            return Outcome::Failed(format!("invalid artifact: {e}"));
        }
        if a.model_release_id != m.release_id.as_str() {
            return Outcome::Failed(format!("artifact forecasts release {}", a.model_release_id));
        }
        // Fail closed: a missing quantity or unit is never scored as the target.
        if a.quantity != Some(req.target) || a.unit.as_deref() != Some(req.unit.as_str()) {
            return Outcome::Failed(format!(
                "artifact forecasts {:?} in {:?}, the protocol scores {:?} in {}",
                a.quantity, a.unit, req.target, req.unit
            ));
        }
        use ignis_core::contracts::QualificationState as Q;
        if !a.support.admitted {
            return Outcome::Refused(format!(
                "support gate {:?}: {}",
                a.support.gate,
                a.support.reasons.join("; ")
            ));
        }
        if matches!(
            a.status,
            Q::Unsupported | Q::Failed | Q::InsufficientInference
        ) {
            return Outcome::Refused(format!("status {:?}", a.status));
        }
        Outcome::Admitted(a)
    }
}

impl EvaluationService for Evaluator {
    fn evaluate(
        &self,
        scope: &ExecutionScope,
        req: &EvaluationRequest,
    ) -> ServiceResult<EvaluationResult> {
        // 1. contract, scope, purpose, protocol
        req.validate()
            .map_err(|e| err(ErrorCode::InvalidRequest, e.to_string()))?;
        self.check_scope(scope, req.partition)?;
        if req.purpose == EvaluationPurpose::FinalTest {
            return Err(err(
                ErrorCode::Unsupported,
                "final test: no qualification criteria are frozen yet; only development feedback runs",
            ));
        }
        self.check_protocol(req)?;

        // 2. identities
        let bundle = self.datasets.dataset(scope, &req.dataset_id)?;
        self.check_bundle(req, &bundle)?;
        let subjects: Vec<&ignis_core::contracts::ModelUnderTest> = std::iter::once(&req.candidate)
            .chain(req.baselines.iter())
            .collect();
        let mut seen = BTreeSet::new();
        let mut releases = vec![];
        for m in &subjects {
            if !seen.insert(m.release_id.as_str()) {
                return Err(err(
                    ErrorCode::InvalidRequest,
                    format!("release {} listed twice", m.release_id),
                ));
            }
            releases.push(self.check_release(scope, m)?);
        }
        let mut metrics = vec![];
        for ms in &req.metrics {
            let (key, unit) = metric_table(&ms.metric, &ms.version).ok_or_else(|| {
                err(
                    ErrorCode::Unsupported,
                    format!("metric {}@{} has no scorer", ms.metric, ms.version),
                )
            })?;
            metrics.push((ms, key, unit));
        }

        // 3. cases (no outcomes)
        let cutoffs: Vec<f64> = req.cutoffs_s.iter().map(|c| c.get()).collect();
        let cases = self.cases.cases(scope, &bundle, req.partition, &cutoffs)?;
        let mut ids = BTreeSet::new();
        for c in &cases {
            if !ids.insert(c.case_id.as_str()) {
                return Err(err(
                    ErrorCode::Internal,
                    format!("case {} listed twice", c.case_id),
                ));
            }
            if !cutoffs.contains(&c.cutoff_s) {
                return Err(err(
                    ErrorCode::Internal,
                    format!(
                        "case {} at cutoff {} not in the request",
                        c.case_id, c.cutoff_s
                    ),
                ));
            }
        }

        // 4. forecasts, with no outcome in memory
        let runs: Vec<SubjectRun> = subjects
            .iter()
            .map(|m| SubjectRun {
                release_id: m.release_id.to_string(),
                outcomes: cases
                    .par_iter()
                    .map(|c| self.forecast_one(scope, req, m, c))
                    .collect(),
            })
            .collect();

        // One spec over every admitted artifact of every subject: like-for-like estimators.
        let admitted: Vec<&AnalysisArtifact> = runs
            .iter()
            .flat_map(|r| r.outcomes.iter())
            .filter_map(|o| {
                if let Outcome::Admitted(a) = o {
                    Some(a)
                } else {
                    None
                }
            })
            .collect();
        let spec: Spec = common_spec(&admitted, None);

        // 5. commitments and reveals, all before any scoring. A reveal that fails stops the
        // evaluation before a result exists, and the error names every continuation already opened
        // (a vault source has logged them), so a retry is a visible, deliberate second reveal.
        let mut truths: Vec<Vec<Option<Value>>> = Vec::with_capacity(runs.len());
        let mut opened: Vec<String> = vec![];
        for run in &runs {
            let mut row = Vec::with_capacity(cases.len());
            for (c, o) in cases.iter().zip(&run.outcomes) {
                let Outcome::Admitted(a) = o else {
                    row.push(None);
                    continue;
                };
                let commitment = Commitment {
                    release_id: run.release_id.clone(),
                    artifact_hash: artifact_hash(a),
                };
                match self.cases.reveal(scope, c, &commitment) {
                    Ok(t) => {
                        opened.push(format!("{}@{}", c.case_id, run.release_id));
                        row.push(Some(t));
                    }
                    Err(e) => {
                        return Err(ErrorEnvelope {
                            message: format!(
                                "reveal of {} for {} failed ({}); no result was written; {} continuation(s) already revealed: [{}]",
                                c.case_id,
                                run.release_id,
                                e.message,
                                opened.len(),
                                opened.join(", ")
                            ),
                            ..e
                        });
                    }
                }
            }
            truths.push(row);
        }

        // 6. scores
        let partition = serde_json::to_value(req.partition).unwrap_or(Value::Null);
        let mut scored_by_subject: Vec<Vec<(usize, Case)>> = vec![];
        let mut failures: Vec<CaseFailure> = vec![];
        let mut accounting: Vec<CaseAccounting> = vec![];
        for (run, truth_row) in runs.iter().zip(&truths) {
            let mut acc = CaseAccounting {
                total: cases.len() as u64,
                scored: 0,
                refused: 0,
                failed: 0,
            };
            let mut scored = vec![];
            for (i, (c, o)) in cases.iter().zip(&run.outcomes).enumerate() {
                let fail = |kind, reason: String| CaseFailure {
                    case_id: c.case_id.clone(),
                    subject: ignis_core::contracts::ReleaseId::new(run.release_id.clone())
                        .expect("release id from the request"),
                    kind,
                    reason,
                };
                match o {
                    Outcome::Failed(r) => {
                        acc.failed += 1;
                        failures.push(fail(CaseFailureKind::Failed, r.clone()));
                    }
                    Outcome::Refused(r) => {
                        acc.refused += 1;
                        failures.push(fail(CaseFailureKind::Refused, r.clone()));
                    }
                    Outcome::Admitted(a) => {
                        let truth = truth_row[i]
                            .as_ref()
                            .expect("every admitted case was revealed above");
                        let mut case = score_against(
                            a,
                            truth,
                            &c.run_id,
                            partition.clone(),
                            c.group_id.clone(),
                            &spec,
                        );
                        case.case_id = c.case_id.clone();
                        if case.scored {
                            acc.scored += 1;
                            scored.push((i, case));
                        } else {
                            acc.failed += 1;
                            failures.push(fail(
                                CaseFailureKind::Failed,
                                format!(
                                    "admitted but not scorable: {}",
                                    case.reason.unwrap_or_default()
                                ),
                            ));
                        }
                    }
                }
            }
            accounting.push(acc);
            scored_by_subject.push(scored);
        }

        // 7. metric values: overall and per cutoff, from the cluster bootstrap estimate
        let mut values = vec![];
        for ((run, scored), acc) in runs.iter().zip(&scored_by_subject).zip(&accounting) {
            let subject = ignis_core::contracts::ReleaseId::new(run.release_id.clone())
                .expect("release id from the request");
            let mut groups: BTreeMap<String, Vec<&Case>> = BTreeMap::new();
            groups.insert(String::new(), scored.iter().map(|(_, c)| c).collect());
            for (i, c) in scored {
                groups
                    .entry(format!("{:?}", cases[*i].cutoff_s))
                    .or_default()
                    .push(c);
            }
            for (ms, key, unit) in &metrics {
                for (g, members) in &groups {
                    let (value, n) = estimate(members, key);
                    let mut group = BTreeMap::new();
                    if !g.is_empty() {
                        group.insert("cutoff_s".to_string(), g.clone());
                    }
                    values.push(MetricValue {
                        metric: ms.metric.clone(),
                        version: ms.version.clone(),
                        subject: subject.clone(),
                        group,
                        population: Population::ScoredCases,
                        value,
                        unit: unit_label(*unit, &req.unit),
                        n,
                    });
                }
            }
            values.push(MetricValue {
                metric: "case_coverage".into(),
                version: "1".into(),
                subject,
                group: BTreeMap::new(),
                population: Population::AllCases,
                value: finite(if acc.total == 0 {
                    0.0
                } else {
                    acc.scored as f64 / acc.total as f64
                }),
                unit: "1".into(),
                n: acc.total,
            });
        }

        let cand = accounting[0];
        let origin = if self.cases.origin() == Origin::SyntheticFixture
            || bundle.origin == Origin::SyntheticFixture
            || releases.iter().any(|r| {
                r.construction
                    .as_ref()
                    .is_some_and(|c| c.origin == Origin::SyntheticFixture)
            }) {
            Origin::SyntheticFixture
        } else {
            Origin::System
        };
        let mut result = EvaluationResult {
            schema_version: EvaluationSchema::default(),
            result_id: EvaluationId::new("pending").expect("static id"),
            request: req.clone(),
            request_hash: req.request_hash(),
            cases: cand,
            coverage: finite(if cand.total == 0 {
                0.0
            } else {
                cand.scored as f64 / cand.total as f64
            })
            .expect("finite ratio"),
            metrics: values,
            per_case: None,
            failures,
            verdict: Verdict::NotAssessed,
            compute: None,
            origin,
            created_at: (self.clock)(),
        };
        // Content-addressed id: identical inputs give an identical result.
        let digest = Digest::of_json_excluding(&result, &["result_id", "created_at"]);
        result.result_id =
            EvaluationId::new(format!("eval-{}", &digest.hex()[..24])).expect("hex id");
        result.validate().map_err(|e| {
            err(
                ErrorCode::Internal,
                format!("evaluator produced an invalid result: {e}"),
            )
        })?;
        Ok(result)
    }

    fn compare(
        &self,
        scope: &ExecutionScope,
        results: &[EvaluationResult],
    ) -> ServiceResult<Vec<MetricValue>> {
        if scope.role != ScopeRole::Evaluation {
            return Err(err(
                ErrorCode::ScopeDenied,
                "comparison runs only in an evaluation scope",
            ));
        }
        let Some(first) = results.first() else {
            return Ok(vec![]);
        };
        let same = |r: &EvaluationResult| -> Vec<String> {
            let (a, b) = (&first.request, &r.request);
            let mut d = vec![];
            if a.purpose != b.purpose {
                d.push("purpose")
            }
            if a.protocol != b.protocol {
                d.push("protocol")
            }
            if a.target != b.target || a.unit != b.unit {
                d.push("target/unit")
            }
            if a.dataset_hash != b.dataset_hash {
                d.push("dataset_hash")
            }
            if a.split_id != b.split_id || a.partition != b.partition {
                d.push("split/partition")
            }
            if a.cutoffs_s != b.cutoffs_s {
                d.push("cutoffs")
            }
            if a.metrics != b.metrics {
                d.push("metrics")
            }
            if r.cases.total != first.cases.total {
                d.push("case count")
            }
            d.into_iter().map(String::from).collect()
        };
        let mut subjects = BTreeSet::new();
        for r in results {
            r.validate()
                .map_err(|e| err(ErrorCode::InvalidRequest, e.to_string()))?;
            let d = same(r);
            if !d.is_empty() {
                return Err(err(
                    ErrorCode::InvalidRequest,
                    format!(
                        "result {} is not comparable with {}: {}",
                        r.result_id,
                        first.result_id,
                        d.join(", ")
                    ),
                ));
            }
            if !subjects.insert(r.request.candidate.release_id.clone()) {
                return Err(err(
                    ErrorCode::InvalidRequest,
                    format!("candidate {} appears twice", r.request.candidate.release_id),
                ));
            }
        }
        Ok(results
            .iter()
            .flat_map(|r| {
                let c = r.request.candidate.release_id.clone();
                r.metrics.iter().filter(move |m| m.subject == c).cloned()
            })
            .collect())
    }
}

/// Cluster-bootstrap point estimate of `key` over `cases` (mean of cluster means; clusters by
/// run/group/session exactly as `scoring::aggregate`). `n` counts finite values only.
pub fn estimate(cases: &[&Case], key: &str) -> (Option<Finite>, u64) {
    let vals: Vec<f64> = cases
        .iter()
        .map(|c| *c.metrics.get(key).unwrap_or(&f64::NAN))
        .collect();
    let labels = clusters(
        &cases.iter().map(|c| c.run_id.clone()).collect::<Vec<_>>(),
        &cases.iter().map(|c| c.group_id.clone()).collect::<Vec<_>>(),
        &cases
            .iter()
            .map(|c| c.session_id.as_str().map(String::from))
            .collect::<Vec<_>>(),
    );
    let b = cluster_bootstrap(&vals, &labels, N_BOOT, BOOT_SEED, 0.95);
    let n = b.get("n_cases").and_then(Value::as_u64).unwrap_or(0);
    let v = b.get("estimate").and_then(Value::as_f64).and_then(finite);
    (v, n)
}

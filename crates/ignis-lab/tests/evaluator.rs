//! T2: the evaluator service on the C0 fixtures. Fakes stand in for the model (Iancu) and case
//! (vault) services; the request, bundle, release and expected accounting are the committed
//! fixtures in `fixtures/contracts/`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ignis_core::contracts::{
    AnalysisArtifact, CaseFailureKind, DatasetBundle, DatasetId, ErrorCode, ErrorEnvelope,
    EvaluationRequest, EvaluationResult, ExecutionScope, ForecastInput, ForecastRequest,
    ModelRelease, Origin, Partition, Population, ReleaseId, RetryPolicy, ScopeRole, Timestamp,
    Verdict,
};
use ignis_core::hashing::content_hash;
use ignis_core::services::{DatasetService, EvaluationService, ModelService, ServiceResult};
use ignis_lab::eval::protocol::{freeze, load_frozen, Protocol};
use ignis_lab::eval::scorecard;
use ignis_lab::eval::scoring::{aggregate, common_spec, score_against};
use ignis_lab::eval::service::{estimate, CaseSource, Commitment, EvalCase, Evaluator};
use serde_json::{json, Value};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture<T: serde::de::DeserializeOwned>(rel: &str) -> T {
    let p = repo().join("fixtures/contracts").join(rel);
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap()
}

fn eval_scope(partitions: &[Partition]) -> ExecutionScope {
    let mut s: ExecutionScope = fixture("common/scope_research.json");
    s.role = ScopeRole::Evaluation;
    s.partitions = partitions.iter().copied().collect();
    s
}

const BASELINE: &str = "sha256:baba000000000000000000000000000000000000000000000000000000000000";

/// Truth used by the synthetic world: d(t) = 1e-3 (2 - 0.1 t) m.
fn truth(t: f64) -> f64 {
    1e-3 * (2.0 - 0.1 * t)
}

// ------------------------------------------------------------------------------------- fakes

struct Datasets;
impl DatasetService for Datasets {
    fn dataset(&self, _s: &ExecutionScope, id: &DatasetId) -> ServiceResult<DatasetBundle> {
        let b: DatasetBundle = fixture("data/dataset_bundle.json");
        if b.dataset_id != *id {
            return Err(ErrorEnvelope::new(
                ErrorCode::NotFound,
                format!("dataset {id}"),
                RetryPolicy::Never,
            ));
        }
        Ok(b)
    }
}

#[derive(Default)]
struct Models {
    /// (run, cutoff) whose forecast call errors.
    error_on: Option<(String, f64)>,
    /// Forecast horizon in s after the cutoff (a short horizon leaves nothing to score).
    horizon: Option<f64>,
    calls: AtomicUsize,
}

impl Models {
    fn artifact(&self, release: &str, run: &str, cutoff: f64) -> AnalysisArtifact {
        let h = self.horizon.unwrap_or(2.0);
        let t: Vec<f64> = (0..5).map(|i| cutoff + h * i as f64 / 4.0).collect();
        let shift = if release == BASELINE { 5e-5 } else { 1e-5 };
        let med: Vec<f64> = t.iter().map(|&x| truth(x) + shift).collect();
        let refused = run == "fixture-run-03" && cutoff == 1.0;
        let mut a: AnalysisArtifact = serde_json::from_value(json!({
            "id": "", "kind": "prefix_forecast", "model_release_id": release, "prefix_hash": format!("prefix-{run}-{cutoff}"),
            "settings_hash": "s", "status": "experimental", "quantity": "liquid_diameter", "unit": "m", "times_s": t,
            "quantile_levels": [0.05, 0.5, 0.95],
            "quantiles": [med.iter().map(|m| m - 1e-4).collect::<Vec<_>>(), med.clone(), med.iter().map(|m| m + 1e-4).collect::<Vec<_>>()],
            "support": if refused {
                json!({"admitted": false, "gate": "measurement", "reasons": ["prefix holds 1 observation, the forward needs 2"]})
            } else {
                json!({"admitted": true, "gate": "passed"})
            },
            "created_at": "2026-11-13T00:00:00Z"}))
        .unwrap();
        a.id = content_hash(&a, &["id", "created_at"]);
        a
    }
}

impl ModelService for Models {
    fn releases(&self, s: &ExecutionScope) -> ServiceResult<Vec<ModelRelease>> {
        Ok(vec![self.release(s, &ReleaseId::new(BASELINE).unwrap())?])
    }
    fn release(&self, _s: &ExecutionScope, id: &ReleaseId) -> ServiceResult<ModelRelease> {
        let mut r: ModelRelease = fixture("models/release_w0.json");
        if id.as_str() == BASELINE {
            r.id = BASELINE.into();
        } else if r.id != id.as_str() {
            return Err(ErrorEnvelope::new(
                ErrorCode::NotFound,
                format!("release {id}"),
                RetryPolicy::Never,
            ));
        }
        Ok(r)
    }
    fn forecast(
        &self,
        _s: &ExecutionScope,
        req: &ForecastRequest,
    ) -> ServiceResult<AnalysisArtifact> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let ForecastInput::StoredRun { run_id, cutoff_s } = &req.input else {
            panic!("stored runs only")
        };
        if self
            .error_on
            .as_ref()
            .is_some_and(|(r, c)| r == run_id.as_str() && *c == cutoff_s.get())
        {
            return Err(ErrorEnvelope::new(
                ErrorCode::Internal,
                "forward crashed",
                RetryPolicy::Never,
            ));
        }
        Ok(self.artifact(req.release_id.as_str(), run_id.as_str(), cutoff_s.get()))
    }
}

struct Cases {
    models: Arc<Models>,
    /// (case_id, release, artifact hash, forecasts done when revealed)
    reveals: Mutex<Vec<(String, String, String, usize)>>,
}

impl CaseSource for Cases {
    fn cases(
        &self,
        _s: &ExecutionScope,
        _b: &DatasetBundle,
        p: Partition,
        cutoffs: &[f64],
    ) -> ServiceResult<Vec<EvalCase>> {
        if p == Partition::Test {
            return Err(ErrorEnvelope::new(
                ErrorCode::ScopeDenied,
                "fixture source has no lockbox",
                RetryPolicy::Never,
            ));
        }
        let mut out = vec![];
        for (run, group) in [("fixture-run-02", "g2"), ("fixture-run-03", "g3")] {
            for &c in cutoffs {
                out.push(EvalCase {
                    case_id: format!("{run}@{c:?}"),
                    run_id: run.into(),
                    group_id: Some(group.into()),
                    cutoff_s: c,
                });
            }
        }
        Ok(out)
    }
    fn reveal(&self, _s: &ExecutionScope, case: &EvalCase, c: &Commitment) -> ServiceResult<Value> {
        self.reveals.lock().unwrap().push((
            case.case_id.clone(),
            c.release_id.clone(),
            c.artifact_hash.clone(),
            self.models.calls.load(Ordering::SeqCst),
        ));
        let obs: Vec<Value> = (1..=8)
            .map(|i| {
                let t = case.cutoff_s + 0.25 * i as f64;
                json!({"time_s": t, "value_si": truth(t), "quantity": "liquid_diameter", "valid": true})
            })
            .collect();
        Ok(json!({"cutoff_s": case.cutoff_s, "observations": obs}))
    }
    fn origin(&self) -> Origin {
        Origin::SyntheticFixture
    }
}

fn evaluator_with(models: Models) -> (Evaluator, Arc<Cases>) {
    let models = Arc::new(models);
    let cases = Arc::new(Cases {
        models: models.clone(),
        reveals: Mutex::new(vec![]),
    });
    let e = Evaluator {
        datasets: Arc::new(Datasets),
        models,
        cases: cases.clone(),
        protocols: BTreeMap::new(),
        clock: Arc::new(|| Timestamp::parse("2026-11-13T12:00:00Z").unwrap()),
    };
    (e, cases)
}

fn request() -> EvaluationRequest {
    fixture("evaluation/request_dev.json")
}

fn code(r: ServiceResult<EvaluationResult>) -> ErrorCode {
    r.expect_err("expected a refusal").code
}

// ------------------------------------------------------------------------------------- tests

#[test]
fn the_fixture_request_gives_the_fixture_accounting() {
    let (e, cases) = evaluator_with(Models::default());
    let r = e
        .evaluate(
            &eval_scope(&[Partition::Train, Partition::Validation]),
            &request(),
        )
        .unwrap();
    r.validate().unwrap();
    let expected: EvaluationResult = fixture("evaluation/result_dev.json");
    assert_eq!(r.request_hash, expected.request_hash);
    assert_eq!(r.cases, expected.cases, "4 cases: 3 scored, 1 refused");
    assert_eq!(r.coverage.get(), 0.75);
    assert_eq!(r.verdict, Verdict::NotAssessed);
    assert_eq!(r.origin, Origin::SyntheticFixture);
    assert_eq!(r.failures.len(), 1);
    assert_eq!(r.failures[0].case_id, "fixture-run-03@1.0");
    assert_eq!(r.failures[0].kind, CaseFailureKind::Refused);

    let crps: Vec<_> = r.metrics.iter().filter(|m| m.metric == "crps").collect();
    let overall = crps.iter().find(|m| m.group.is_empty()).unwrap();
    assert_eq!(
        (overall.n, overall.unit.as_str(), overall.population),
        (3, "m", Population::ScoredCases)
    );
    assert!(overall.value.unwrap().get() > 0.0);
    let by_cutoff: BTreeMap<_, _> = crps
        .iter()
        .filter(|m| !m.group.is_empty())
        .map(|m| (m.group["cutoff_s"].clone(), m.n))
        .collect();
    assert_eq!(
        by_cutoff,
        BTreeMap::from([("1.0".to_string(), 1), ("3.0".to_string(), 2)])
    );
    let cov = r
        .metrics
        .iter()
        .find(|m| m.metric == "case_coverage")
        .unwrap();
    assert_eq!(
        (cov.population, cov.n, cov.value.unwrap().get()),
        (Population::AllCases, 4, 0.75)
    );

    // Outcomes were read only after every forecast, only for admitted artifacts, each with a commitment.
    let rv = cases.reveals.lock().unwrap();
    assert_eq!(rv.len(), 3);
    assert!(rv
        .iter()
        .all(|(_, _, h, done)| *done == 4 && h.starts_with("sha256:")));
    assert!(!rv.iter().any(|(id, _, _, _)| id == "fixture-run-03@1.0"));
}

#[test]
fn the_result_is_deterministic_across_runs_and_thread_counts() {
    let run = |threads: usize| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        pool.install(|| {
            let (e, _) = evaluator_with(Models::default());
            let mut req = request();
            req.baselines = vec![serde_json::from_value(json!({"release_id": BASELINE})).unwrap()];
            serde_json::to_string(
                &e.evaluate(&eval_scope(&[Partition::Validation]), &req)
                    .unwrap(),
            )
            .unwrap()
        })
    };
    let one = run(1);
    assert_eq!(one, run(1));
    assert_eq!(one, run(4));
    let r: EvaluationResult = serde_json::from_str(&one).unwrap();
    assert!(r.result_id.as_str().starts_with("eval-"));
}

#[test]
fn baselines_are_scored_on_the_same_cases_with_the_same_estimators() {
    let (e, _) = evaluator_with(Models::default());
    let mut req = request();
    req.baselines = vec![serde_json::from_value(json!({"release_id": BASELINE})).unwrap()];
    let r = e
        .evaluate(&eval_scope(&[Partition::Validation]), &req)
        .unwrap();
    r.validate().unwrap();
    let crps = |s: &str| {
        r.metrics
            .iter()
            .find(|m| m.metric == "crps" && m.group.is_empty() && m.subject.as_str() == s)
            .unwrap()
            .clone()
    };
    let (c, b) = (crps(req.candidate.release_id.as_str()), crps(BASELINE));
    assert_eq!((c.n, b.n), (3, 3));
    assert!(
        b.value.unwrap().get() > c.value.unwrap().get(),
        "the baseline is shifted further from the truth"
    );
    assert_eq!(
        r.failures
            .iter()
            .filter(|f| f.subject.as_str() == BASELINE)
            .count(),
        1
    );
}

#[test]
fn the_estimate_equals_the_cli_aggregate_on_the_same_cases() {
    let m = Models::default();
    let arts: Vec<(AnalysisArtifact, Value, &str, &str)> = [
        ("fixture-run-02", "g2", 1.0),
        ("fixture-run-02", "g2", 3.0),
        ("fixture-run-03", "g3", 3.0),
    ]
    .iter()
    .map(|&(run, g, c)| {
        let obs: Vec<Value> = (1..=8)
            .map(|i| {
                let t = c + 0.25 * i as f64;
                json!({"time_s": t, "value_si": truth(t), "valid": true})
            })
            .collect();
        (
            m.artifact(request().candidate.release_id.as_str(), run, c),
            json!({"cutoff_s": c, "observations": obs}),
            run,
            g,
        )
    })
    .collect();
    let spec = common_spec(&arts.iter().map(|a| &a.0).collect::<Vec<_>>(), None);
    let mut cases: Vec<_> = arts
        .iter()
        .map(|(a, t, run, g)| {
            score_against(a, t, run, json!("validation"), Some(g.to_string()), &spec)
        })
        .collect();
    let (ours, n) = estimate(&cases.iter().collect::<Vec<_>>(), "crps");
    let agg = aggregate(&mut cases, &spec, 200, 0);
    assert_eq!(n, 3);
    assert_eq!(
        ours.unwrap().get(),
        agg["metrics"]["crps"]["estimate"].as_f64().unwrap()
    );
}

#[test]
fn failed_and_unscorable_cases_stay_in_the_denominator() {
    let (e, _) = evaluator_with(Models {
        error_on: Some(("fixture-run-02".into(), 3.0)),
        ..Default::default()
    });
    let r = e
        .evaluate(&eval_scope(&[Partition::Validation]), &request())
        .unwrap();
    assert_eq!(
        (
            r.cases.total,
            r.cases.scored,
            r.cases.refused,
            r.cases.failed
        ),
        (4, 2, 1, 1)
    );
    assert!(r
        .failures
        .iter()
        .any(|f| f.kind == CaseFailureKind::Failed && f.reason.contains("forward crashed")));

    // A horizon that ends before the first continuation point cannot avoid the denominator.
    let (e, _) = evaluator_with(Models {
        horizon: Some(0.1),
        ..Default::default()
    });
    let r = e
        .evaluate(&eval_scope(&[Partition::Validation]), &request())
        .unwrap();
    r.validate().unwrap();
    assert_eq!((r.cases.scored, r.cases.refused, r.cases.failed), (0, 1, 3));
    assert!(r
        .failures
        .iter()
        .filter(|f| f.kind == CaseFailureKind::Failed)
        .all(|f| f.reason.starts_with("admitted but not scorable")));
    let crps = r
        .metrics
        .iter()
        .find(|m| m.metric == "crps" && m.group.is_empty())
        .unwrap();
    assert_eq!((crps.value, crps.n), (None, 0), "undefined, never zero");
}

#[test]
fn refusals_happen_before_any_forecast() {
    let (e, _) = evaluator_with(Models::default());
    let mut research = eval_scope(&[Partition::Validation]);
    research.role = ScopeRole::Research;
    assert_eq!(
        code(e.evaluate(&research, &request())),
        ErrorCode::ScopeDenied
    );
    assert_eq!(
        code(e.evaluate(&eval_scope(&[Partition::Train]), &request())),
        ErrorCode::ScopeDenied,
        "partition not granted"
    );

    let mut on_test = request();
    on_test.partition = Partition::Test;
    assert_eq!(
        code(e.evaluate(&eval_scope(&[Partition::Test]), &on_test)),
        ErrorCode::InvalidRequest
    );

    let mut other_data = request();
    other_data.dataset_hash = ignis_core::contracts::Digest::of_bytes(b"other");
    assert_eq!(
        code(e.evaluate(&eval_scope(&[Partition::Validation]), &other_data)),
        ErrorCode::Conflict
    );

    let mut metric = request();
    metric.metrics[0].metric = "accuracy".into();
    assert_eq!(
        code(e.evaluate(&eval_scope(&[Partition::Validation]), &metric)),
        ErrorCode::Unsupported
    );

    let mut forward = request();
    forward.candidate.forward_hash =
        Some(ignis_core::contracts::Digest::of_bytes(b"another forward"));
    assert_eq!(
        code(e.evaluate(&eval_scope(&[Partition::Validation]), &forward)),
        ErrorCode::Conflict
    );

    let mut final_test = request();
    final_test.purpose = ignis_core::contracts::EvaluationPurpose::FinalTest;
    final_test.partition = Partition::Test;
    final_test.protocol.frozen_hash = Some(ignis_core::contracts::Digest::of_bytes(b"p"));
    assert_eq!(
        code(e.evaluate(&eval_scope(&[Partition::Test]), &final_test)),
        ErrorCode::Unsupported
    );

    let (e2, cases) = evaluator_with(Models::default());
    let _ = e2.evaluate(&research, &request());
    assert!(cases.reveals.lock().unwrap().is_empty());
    assert_eq!(
        cases.models.calls.load(Ordering::SeqCst),
        0,
        "no forecast ran for a refused request"
    );
}

fn protocol_from(req: &EvaluationRequest) -> Protocol {
    serde_json::from_value(json!({
        "schema_version": "evaluation_protocol.v1",
        "protocol_id": "flex-diameter-dev", "version": "1",
        "target": req.target, "unit": req.unit, "dataset_hash": req.dataset_hash, "split_id": req.split_id,
        "partition": req.partition, "cutoffs_s": req.cutoffs_s, "metrics": req.metrics,
        "notes": "FLEX liquid diameter after the cutoff; groups = split groups."
    }))
    .unwrap()
}

#[test]
fn a_frozen_protocol_is_write_once_and_binds_the_request() {
    let dir = std::env::temp_dir().join(format!("ignis-protocols-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let p = protocol_from(&request());

    // Formatting does not change the hash; content does.
    let reparsed: Protocol =
        serde_json::from_str(&serde_json::to_string_pretty(&p).unwrap()).unwrap();
    assert_eq!(reparsed.frozen_hash(), p.frozen_hash());
    let mut changed = p.clone();
    changed.cutoffs_s.pop();
    assert_ne!(changed.frozen_hash(), p.frozen_hash());

    let f = freeze(&dir, &p).unwrap();
    assert_eq!(
        freeze(&dir, &p).unwrap(),
        f,
        "idempotent for identical content"
    );
    assert!(
        freeze(&dir, &changed).is_err(),
        "same id and version, different content"
    );

    let (mut e, _) = evaluator_with(Models::default());
    e.protocols = load_frozen(&dir).unwrap();
    let mut req = request();
    req.protocol = p.reference();
    e.evaluate(&eval_scope(&[Partition::Validation]), &req)
        .unwrap()
        .validate()
        .unwrap();

    let mut drift = req.clone();
    drift.cutoffs_s.pop();
    let err = e
        .evaluate(&eval_scope(&[Partition::Validation]), &drift)
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidRequest);
    assert!(err.message.contains("cutoffs_s"), "{}", err.message);

    let mut unknown = req.clone();
    unknown.protocol.version = "2".into();
    assert_eq!(
        code(e.evaluate(&eval_scope(&[Partition::Validation]), &unknown)),
        ErrorCode::NotFound
    );

    // A frozen file edited by hand no longer loads.
    let path = std::fs::read_dir(&dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace("groups = split groups", "groups = runs");
    std::fs::write(&path, text).unwrap();
    assert!(load_frozen(&dir).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_scorecard_shows_accounting_next_to_metrics() {
    let (e, _) = evaluator_with(Models::default());
    let r = e
        .evaluate(&eval_scope(&[Partition::Validation]), &request())
        .unwrap();
    let md = scorecard::render(&r);
    assert!(md.contains("Synthetic fixture"));
    assert!(md.contains("4 cases: 3 scored, 1 refused, 0 failed. Coverage 75.0 %"));
    assert!(md.contains("**not frozen**"));
    assert!(md.contains("`fixture-run-03@1.0`"));
    assert!(md.contains("| crps (v1) | all | scored cases |"));
    assert!(md.contains("| case_coverage (v1) | all | all cases | 0.7500 | 4 |"));
}

#[test]
fn compare_refuses_results_that_are_not_like_for_like() {
    let (e, _) = evaluator_with(Models::default());
    let scope = eval_scope(&[Partition::Validation]);
    let a = e.evaluate(&scope, &request()).unwrap();
    let mut other = request();
    other.candidate.release_id = ReleaseId::new(BASELINE).unwrap();
    other.candidate.forward_hash = None;
    let b = e.evaluate(&scope, &other).unwrap();
    let rows = e.compare(&scope, &[a.clone(), b.clone()]).unwrap();
    assert!(rows.iter().any(|m| m.subject.as_str() == BASELINE));
    assert!(rows
        .iter()
        .any(|m| m.subject == a.request.candidate.release_id));

    let mut fewer = request();
    fewer.cutoffs_s.pop();
    let c = e.evaluate(&scope, &fewer).unwrap();
    assert_eq!(
        e.compare(&scope, &[a.clone(), c]).unwrap_err().code,
        ErrorCode::InvalidRequest
    );
    assert_eq!(
        e.compare(&scope, &[a.clone(), a]).unwrap_err().code,
        ErrorCode::InvalidRequest,
        "same candidate twice"
    );
}

#[test]
fn every_supported_metric_gives_a_finite_value() {
    let (e, _) = evaluator_with(Models::default());
    let mut req = request();
    let names = [
        "crps",
        "mae_median",
        "lpd",
        "coverage@0.9",
        "coverage@0.90",
        "width@0.9",
    ];
    req.metrics = names
        .iter()
        .map(|m| serde_json::from_value(json!({"metric": m, "version": "1"})).unwrap())
        .collect();
    let r = e
        .evaluate(&eval_scope(&[Partition::Validation]), &req)
        .unwrap();
    r.validate().unwrap();
    for name in names {
        let m = r
            .metrics
            .iter()
            .find(|m| m.metric == name && m.group.is_empty())
            .unwrap();
        assert_eq!(m.n, 3, "{name}");
        assert!(
            m.value.is_some(),
            "{name} has no value: its key does not match the scorer"
        );
    }
    let unit = |n: &str| {
        r.metrics
            .iter()
            .find(|m| m.metric == n)
            .unwrap()
            .unit
            .clone()
    };
    assert_eq!(
        (unit("crps"), unit("coverage@0.9"), unit("width@0.9")),
        ("m".into(), "1".into(), "m".into())
    );
    // mae_mean needs a mean forecast, which these artifacts do not have: refused as unscorable, not zero.
    req.metrics =
        vec![serde_json::from_value(json!({"metric": "mae_mean", "version": "1"})).unwrap()];
    let r = e
        .evaluate(&eval_scope(&[Partition::Validation]), &req)
        .unwrap();
    let m = r
        .metrics
        .iter()
        .find(|m| m.metric == "mae_mean" && m.group.is_empty())
        .unwrap();
    assert_eq!((m.value, m.n), (None, 0));
}

#[test]
fn an_artifact_without_quantity_or_unit_is_failed_not_scored_as_metres() {
    struct Bare(Models, &'static str);
    impl ModelService for Bare {
        fn releases(&self, s: &ExecutionScope) -> ServiceResult<Vec<ModelRelease>> {
            self.0.releases(s)
        }
        fn release(&self, s: &ExecutionScope, id: &ReleaseId) -> ServiceResult<ModelRelease> {
            self.0.release(s, id)
        }
        fn forecast(
            &self,
            s: &ExecutionScope,
            r: &ForecastRequest,
        ) -> ServiceResult<AnalysisArtifact> {
            let mut a = self.0.forecast(s, r)?;
            match self.1 {
                "quantity" => a.quantity = None,
                _ => a.unit = None,
            }
            Ok(a)
        }
    }
    for missing in ["quantity", "unit"] {
        let (mut e, _) = evaluator_with(Models::default());
        e.models = Arc::new(Bare(Models::default(), missing));
        let r = e
            .evaluate(&eval_scope(&[Partition::Validation]), &request())
            .unwrap();
        r.validate().unwrap();
        assert_eq!(
            (r.cases.scored, r.cases.refused, r.cases.failed),
            (0, 0, 4),
            "{missing}"
        );
        // The malformed artifact is checked before its support gate: even a refusal fails here.
        assert!(
            r.failures
                .iter()
                .all(|f| f.kind == CaseFailureKind::Failed
                    && f.reason.contains("the protocol scores"))
        );
    }
}

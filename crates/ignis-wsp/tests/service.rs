//! The literal `ModelService`: which releases it serves, how a forecast is routed, cached, scoped
//! and refused. Every prefix is a synthetic fixture; every registry is a temporary directory.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::json;

use ignis_core::analysis::{build_run_prefix, ForecastSettings};
use ignis_core::contracts::{
    Capability, Cohort, Digest, ErrorCode, ExecutionScope, ExperimentRecord, Family, Finite,
    ForecastInput, ForecastRequest, ModelRelease, Partition, PrefixBundle, PrincipalId, ReleaseId,
    RunId, ScopeId, ScopeRole, Split, TrainedOn, WorkspaceId,
};
use ignis_core::expert::find;
use ignis_core::hashing::content_hash;
use ignis_core::registry::{self, TensorMap};
use ignis_core::services::ModelService;
use ignis_core::views::{RunArchive, View, ViewSource};
use ignis_wsp::assemble;
use ignis_wsp::models::{self, MODELS};
use ignis_wsp::{MemoryPrefixes, Provenance, ViewSourcePrefixes, WspModelService};

/// A registry directory under the system temp dir, removed on drop. Never `artifacts/`.
struct TempRegistry(PathBuf);

impl TempRegistry {
    fn new(name: &str) -> Self {
        let p =
            std::env::temp_dir().join(format!("ignis-wsp-service-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        TempRegistry(p)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRegistry {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Saves the toy FLEX weights (plus `extra`) into `root` and returns the stored release.
fn save_toy(root: &Path, extra: &str) -> ModelRelease {
    let loaded = common::flex_release(extra);
    assemble::save(root, loaded.release, &loaded.params).unwrap()
}

/// Saves a modified copy of a toy release (new id) with the toy tensors.
fn save_variant(root: &Path, f: impl FnOnce(&mut ModelRelease)) -> ModelRelease {
    let loaded = common::flex_release("");
    let mut r = loaded.release.clone();
    f(&mut r);
    let frozen: Vec<String> = loaded.params.keys().cloned().collect();
    registry::save_release(root, r, &loaded.params, &TensorMap::new(), &frozen).unwrap()
}

fn scope(role: ScopeRole, partitions: &[Partition]) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new("scope-test").unwrap(),
        workspace_id: WorkspaceId::new("ws-test").unwrap(),
        principal: PrincipalId::new("user-test").unwrap(),
        role,
        partitions: partitions.iter().copied().collect(),
        capabilities: BTreeSet::from([Capability::ReadData, Capability::RunModels]),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn research() -> ExecutionScope {
    scope(ScopeRole::Research, &[Partition::Train])
}

fn request(release: &ModelRelease, input: ForecastInput) -> ForecastRequest {
    ForecastRequest {
        schema_version: Default::default(),
        release_id: ReleaseId::new(release.id.clone()).unwrap(),
        input,
        settings: ForecastSettings::default(),
        client_request_id: None,
    }
}

fn by_hash(p: &PrefixBundle) -> ForecastInput {
    ForecastInput::Prefix {
        input_hash: p.input_hash.clone(),
    }
}

fn stored(run: &str, cutoff: f64) -> ForecastInput {
    ForecastInput::StoredRun {
        run_id: RunId::new(run).unwrap(),
        cutoff_s: Finite::new(cutoff).unwrap(),
    }
}

fn flex_fixture() -> PrefixBundle {
    models::by_name("flex.wsp").unwrap().fixture_prefix()
}

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

fn code<T: std::fmt::Debug>(r: Result<T, ignis_core::contracts::ErrorEnvelope>) -> ErrorCode {
    r.expect_err("expected an error").code
}

#[test]
fn releases_lists_exactly_the_runnable_literal_releases() {
    let reg = TempRegistry::new("list");
    let a = save_toy(reg.path(), "");
    let b = save_toy(reg.path(), "unc.sigma_d = 1.0e-6\n");
    assert_ne!(a.id, b.id);
    // A literal release of a model this build does not have, and a fitted release that borrows a
    // literal model name but carries no construction manifest: neither is served.
    let unknown = save_variant(reg.path(), |r| {
        r.model_name = "flex.not_compiled.wsp".into()
    });
    let fitted = save_variant(reg.path(), |r| {
        r.construction = None;
        r.trained_on = TrainedOn::Nasa;
    });
    assert_eq!(registry::list_releases(reg.path()).len(), 4);

    let svc = WspModelService::new(reg.path(), MemoryPrefixes::new());
    let listed = svc.releases(&research()).unwrap();
    let mut want = vec![a.id.clone(), b.id.clone()];
    want.sort_by_key(|id| {
        let r = if *id == a.id { &a } else { &b };
        (r.created_at.clone(), r.id.clone())
    });
    assert_eq!(
        listed.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
        want
    );
    assert_eq!(svc.releases(&research()).unwrap(), listed, "stable order");

    // `release` by id, and the contract's not-found otherwise.
    let got = svc
        .release(&research(), &ReleaseId::new(a.id.clone()).unwrap())
        .unwrap();
    assert_eq!(got, a);
    for id in [
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        "../../models".to_string(),
        unknown.id.clone(),
        fitted.id.clone(),
    ] {
        assert_eq!(
            code(svc.release(&research(), &ReleaseId::new(id.clone()).unwrap())),
            ErrorCode::NotFound,
            "{id}"
        );
    }
}

#[test]
fn forecast_on_an_admitted_prefix_returns_the_forward_artifact() {
    let reg = TempRegistry::new("admitted");
    let rel = save_toy(reg.path(), "");
    let prefix = flex_fixture();
    let svc = WspModelService::new(
        reg.path(),
        MemoryPrefixes::new().with_prefix(prefix.clone(), Provenance::SyntheticFixture),
    );
    let a = svc
        .forecast(&research(), &request(&rel, by_hash(&prefix)))
        .unwrap();
    a.validate().unwrap();
    assert!(a.support.admitted, "{:?}", a.support);
    assert_eq!(a.model_release_id, rel.id);
    assert_eq!(a.prefix_hash, prefix.input_hash);
    assert!(!a.times_s.is_empty());
}

#[test]
fn a_scientific_refusal_is_an_artifact_not_an_error() {
    let reg = TempRegistry::new("refused");
    let rel = save_toy(reg.path(), "");
    let mut p = flex_fixture();
    p.phase_established_at_s = None; // the forward refuses: phase not established at the cutoff
    let p = rehash(p);
    let svc = WspModelService::new(
        reg.path(),
        MemoryPrefixes::new().with_prefix(p.clone(), Provenance::SyntheticFixture),
    );
    let a = svc
        .forecast(&research(), &request(&rel, by_hash(&p)))
        .unwrap();
    a.validate().unwrap();
    assert!(!a.support.admitted);
    assert!(a.quantiles.is_empty() || a.quantiles.iter().all(Vec::is_empty));
    assert_eq!(a.prefix_hash, p.input_hash);
}

#[test]
fn a_release_is_loaded_once_across_repeated_and_concurrent_forecasts() {
    let reg = TempRegistry::new("cache");
    let rel = save_toy(reg.path(), "");
    let prefix = flex_fixture();
    let svc = WspModelService::new(
        reg.path(),
        MemoryPrefixes::new().with_prefix(prefix.clone(), Provenance::SyntheticFixture),
    );
    let req = request(&rel, by_hash(&prefix));
    // Artifacts carry `created_at`; the content id is what must repeat.
    let first = svc.forecast(&research(), &req).unwrap().id;
    for _ in 0..3 {
        assert_eq!(svc.forecast(&research(), &req).unwrap().id, first);
    }
    std::thread::scope(|s| {
        let hs: Vec<_> = (0..8)
            .map(|_| s.spawn(|| svc.forecast(&research(), &req).unwrap().id))
            .collect();
        for h in hs {
            assert_eq!(h.join().unwrap(), first);
        }
    });
    svc.release(&research(), &req.release_id).unwrap();
    assert_eq!(svc.release_loads(), 1);

    // A fresh service on a cold cache: concurrent first requests still load once.
    let svc = WspModelService::new(
        reg.path(),
        MemoryPrefixes::new().with_prefix(prefix.clone(), Provenance::SyntheticFixture),
    );
    std::thread::scope(|s| {
        for _ in 0..8 {
            s.spawn(|| svc.forecast(&research(), &req).unwrap());
        }
    });
    assert_eq!(svc.release_loads(), 1);
}

#[test]
fn unknown_and_unrunnable_releases_are_errors() {
    let reg = TempRegistry::new("unknown");
    let rel = save_toy(reg.path(), "");
    let prefix = flex_fixture();
    let svc = WspModelService::new(
        reg.path(),
        MemoryPrefixes::new().with_prefix(prefix.clone(), Provenance::SyntheticFixture),
    );
    let mut req = request(&rel, by_hash(&prefix));
    req.release_id =
        ReleaseId::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
            .unwrap();
    assert_eq!(code(svc.forecast(&research(), &req)), ErrorCode::NotFound);

    // A literal release whose forward is not this build's: the forward refuses to run it at all,
    // which is the contract's `unsupported`, not an artifact.
    let foreign = save_variant(reg.path(), |r| {
        r.construction.as_mut().unwrap().forward_hash = Digest::of_bytes(b"another forward");
    });
    let e = svc
        .forecast(&research(), &request(&foreign, by_hash(&prefix)))
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::Unsupported, "{e}");
    assert!(e.message.contains("forward_mismatch"), "{e}");

    // A literal release of a model this build does not compile.
    let unknown = save_variant(reg.path(), |r| {
        r.model_name = "flex.not_compiled.wsp".into()
    });
    assert_eq!(
        code(svc.forecast(&research(), &request(&unknown, by_hash(&prefix)))),
        ErrorCode::Unsupported
    );
}

#[test]
fn a_run_of_a_partition_the_scope_does_not_grant_is_refused() {
    let reg = TempRegistry::new("scope");
    let rel = save_toy(reg.path(), "");
    let p = |run: &str| {
        let mut p = flex_fixture();
        p.run_id = run.into();
        rehash(p)
    };
    let svc = WspModelService::new(
        reg.path(),
        MemoryPrefixes::new()
            .with_run(p("run-train"), Provenance::Partition(Partition::Train))
            .with_run(
                p("run-validation"),
                Provenance::Partition(Partition::Validation),
            )
            .with_run(p("run-test"), Provenance::Partition(Partition::Test)),
    );
    let cutoff = flex_fixture().cutoff_s;
    let go = |s: &ExecutionScope, run: &str| svc.forecast(s, &request(&rel, stored(run, cutoff)));

    assert!(go(&research(), "run-train").unwrap().support.admitted);
    assert_eq!(
        code(go(&research(), "run-validation")),
        ErrorCode::ScopeDenied
    );
    assert_eq!(code(go(&research(), "run-test")), ErrorCode::ScopeDenied);
    // Granting test to a non-evaluation scope does not open it.
    let widened = scope(ScopeRole::Research, &[Partition::Train, Partition::Test]);
    assert_eq!(code(go(&widened, "run-test")), ErrorCode::ScopeDenied);
    // Only an evaluation scope that grants test reaches it.
    let eval = scope(ScopeRole::Evaluation, &[Partition::Test]);
    assert!(go(&eval, "run-test").is_ok());
    // A scope without the model capability is refused before anything runs.
    let mut no_models = research();
    no_models.capabilities.remove(&Capability::RunModels);
    assert_eq!(code(go(&no_models, "run-train")), ErrorCode::ScopeDenied);
}

#[test]
fn a_prefix_must_be_what_it_claims_to_be() {
    let reg = TempRegistry::new("claims");
    let rel = save_toy(reg.path(), "");
    // Declared synthetic but carrying an original observation: its partition is unknown.
    let mut real = flex_fixture();
    real.observations[0].evidence_class = ignis_core::contracts::EvidenceClass::OriginalObservation;
    let real = rehash(real);
    // Carrying a condition the model's prospective policy does not admit.
    let mut extra = flex_fixture();
    let any = extra.known_conditions.values().next().unwrap().clone();
    extra
        .known_conditions
        .insert("not_a_prospective_key".into(), any);
    let extra = rehash(extra);
    // Modified after its hash was computed.
    let mut tampered = flex_fixture();
    tampered.cutoff_s += 0.5;
    let svc = WspModelService::new(
        reg.path(),
        MemoryPrefixes::new()
            .with_prefix(real.clone(), Provenance::SyntheticFixture)
            .with_prefix(extra.clone(), Provenance::SyntheticFixture)
            .with_prefix(tampered.clone(), Provenance::SyntheticFixture),
    );
    let go = |p: &PrefixBundle| svc.forecast(&research(), &request(&rel, by_hash(p)));
    assert_eq!(code(go(&real)), ErrorCode::ScopeDenied);
    assert_eq!(code(go(&extra)), ErrorCode::Unsupported);
    assert_eq!(code(go(&tampered)), ErrorCode::Internal);
}

#[test]
fn every_model_fixture_carries_only_conditions_its_policy_admits() {
    // The service refuses a prefix with a condition outside the model's policy; the fixtures the
    // build checks and tests use must therefore pass it.
    let experts = ignis_wsp::experts();
    for m in MODELS {
        let policy = find(&experts, m.model_name)
            .unwrap_or_else(|| panic!("no expert for {}", m.model_name))
            .policy();
        let p = m.fixture_prefix();
        for k in p.known_conditions.keys() {
            assert!(
                policy.allowed_conditions.contains(k),
                "{}: fixture condition {k} is outside its policy",
                m.model_name
            );
        }
    }
}

#[test]
fn an_invalid_request_is_the_validation_error() {
    let reg = TempRegistry::new("invalid");
    let rel = save_toy(reg.path(), "");
    let prefix = flex_fixture();
    let svc = WspModelService::new(
        reg.path(),
        MemoryPrefixes::new().with_prefix(prefix.clone(), Provenance::SyntheticFixture),
    );
    let negative = request(&rel, stored("run", -1.0));
    assert_eq!(
        code(svc.forecast(&research(), &negative)),
        ErrorCode::InvalidRequest
    );
    for levels in [vec![0.5, 0.4], vec![0.0, 0.5], vec![]] {
        let mut req = request(&rel, by_hash(&prefix));
        req.settings.quantile_levels = Some(levels.clone());
        assert_eq!(
            code(svc.forecast(&research(), &req)),
            ErrorCode::InvalidRequest,
            "{levels:?}"
        );
    }
    assert_eq!(
        svc.release_loads(),
        0,
        "validation precedes any registry read"
    );
}

// ---------------------------------------------------------------- the ViewSource prefix source

/// Two synthetic FLEX runs in a frozen split (one train, one test) and one run in no split.
struct FakeRecords {
    archives: Vec<RunArchive>,
    split: Split,
    reads: Arc<AtomicUsize>,
}

impl ViewSource for FakeRecords {
    fn cohort(&self, _: &str) -> anyhow::Result<Cohort> {
        anyhow::bail!("not used")
    }
    fn split(&self, split_id: &str) -> anyhow::Result<Split> {
        anyhow::ensure!(split_id == self.split.id, "unknown split {split_id}");
        Ok(self.split.clone())
    }
    fn views(&self, _: Family, _: &str) -> anyhow::Result<Vec<View>> {
        anyhow::bail!("not used")
    }
    fn run_archives(&self, family: Family) -> anyhow::Result<Vec<RunArchive>> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(self
            .archives
            .iter()
            .filter(|a| a.record.family == family)
            .cloned()
            .collect())
    }
}

fn archive_from_fixture(run: &str) -> RunArchive {
    let f = flex_fixture();
    let record: ExperimentRecord = serde_json::from_value(json!({
        "id": run,
        "investigation_id": "synthetic",
        "family": "flex",
        "conditions": f.known_conditions,
        "quality_flags": ["synthetic"],
    }))
    .unwrap();
    let observations = f
        .observations
        .iter()
        .cloned()
        .map(|mut o| {
            o.id = o.id.replace(&f.run_id, run);
            o.run_id = run.into();
            o
        })
        .collect();
    RunArchive {
        record,
        observations,
        contours: vec![],
        control: None,
        endpoint: None,
        time_origin: f.time_origin.clone(),
    }
}

#[test]
fn stored_runs_through_a_view_source_use_the_split_and_the_model_policy() {
    let reg = TempRegistry::new("views");
    let rel = save_toy(reg.path(), "");
    let split = Split {
        id: rel.split_id.clone(),
        cohort_id: rel.cohort_id.clone(),
        group_key: "session".into(),
        rule: "synthetic".into(),
        seed: 0,
        run_to_group: BTreeMap::from([
            ("syn-train".to_string(), "g1".to_string()),
            ("syn-test".to_string(), "g2".to_string()),
        ]),
        train: vec!["g1".into()],
        validation: vec![],
        test: vec!["g2".into()],
        frozen_at: "2026-11-14T00:00:00Z".into(),
    };
    let reads = Arc::new(AtomicUsize::new(0));
    let source = FakeRecords {
        archives: ["syn-train", "syn-test", "syn-orphan"]
            .into_iter()
            .map(archive_from_fixture)
            .collect(),
        split,
        reads: reads.clone(),
    };
    let svc = WspModelService::new(reg.path(), ViewSourcePrefixes::new(source));
    let cutoff = flex_fixture().cutoff_s;

    // A train run: the prefix is the shared builder's, with the release's own model policy.
    let a = svc
        .forecast(&research(), &request(&rel, stored("syn-train", cutoff)))
        .unwrap();
    a.validate().unwrap();
    assert!(a.support.admitted, "{:?}", a.support);
    let experts = ignis_wsp::experts();
    let policy = find(&experts, &rel.model_name).unwrap().policy();
    let want = build_run_prefix(&archive_from_fixture("syn-train"), cutoff, policy).unwrap();
    assert_eq!(a.prefix_hash, want.input_hash);
    assert_eq!(a.run_id.as_deref(), Some("syn-train"));

    // A test run for a research scope is refused before its records are read.
    let before = reads.load(Ordering::SeqCst);
    assert_eq!(
        code(svc.forecast(&research(), &request(&rel, stored("syn-test", cutoff)))),
        ErrorCode::ScopeDenied
    );
    assert_eq!(
        reads.load(Ordering::SeqCst),
        before,
        "no record of a refused run was read"
    );
    // The evaluation scope that grants test reaches it.
    let eval = scope(ScopeRole::Evaluation, &[Partition::Test]);
    assert!(svc
        .forecast(&eval, &request(&rel, stored("syn-test", cutoff)))
        .is_ok());
    // A run outside the release's split (stored or not) has no partition: no scope grants it.
    for run in ["syn-orphan", "syn-none"] {
        assert_eq!(
            code(svc.forecast(&research(), &request(&rel, stored(run, cutoff)))),
            ErrorCode::ScopeDenied,
            "{run}"
        );
    }
    // No admitted-prefix store exists behind a ViewSource.
    assert_eq!(
        code(svc.forecast(&research(), &request(&rel, by_hash(&want)))),
        ErrorCode::CapabilityUnavailable
    );
}

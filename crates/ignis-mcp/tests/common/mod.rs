//! Shared test host: a temp state store, the local artifact store, and fake services that return
//! the C0 fixtures (and record the scopes they were called with).

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ignis_core::contracts::*;
use ignis_core::services::{DatasetService, KnowledgeService, ModelService, ServiceResult};
use ignis_mcp::{ToolHost, ToolServices};
use ignis_research::{LocalArtifactStore, ResearchStore};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

pub fn fixture<T: DeserializeOwned>(rel: &str) -> T {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/contracts")
        .join(rel);
    serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap()
}

/// The research fixture scope moved to workspace `ws` (distinct scope id per workspace).
pub fn scope(ws: &str) -> ExecutionScope {
    let mut s: ExecutionScope = fixture("common/scope_research.json");
    s.workspace_id = WorkspaceId::new(ws).unwrap();
    s.scope_id = ScopeId::new(format!("scope_{ws}")).unwrap();
    s
}

#[derive(Default)]
pub struct FakeKnowledge {
    pub calls: Mutex<Vec<ExecutionScope>>,
    /// Hits repeated into the page (to make a large result).
    pub repeat: AtomicUsize,
}

impl KnowledgeService for FakeKnowledge {
    fn search(&self, scope: &ExecutionScope, _: &SearchRequest) -> ServiceResult<SearchPage> {
        self.calls.lock().unwrap().push(scope.clone());
        let mut page: SearchPage = fixture("evidence/search_page.json");
        let n = self.repeat.load(Ordering::SeqCst);
        if n > 1 {
            let hits = page.hits.clone();
            page.hits = (0..n).flat_map(|_| hits.clone()).collect();
        }
        Ok(page)
    }
    fn read(&self, scope: &ExecutionScope, _: &ReadRequest) -> ServiceResult<ReadResult> {
        self.calls.lock().unwrap().push(scope.clone());
        Ok(fixture("evidence/read_result.json"))
    }
    fn resolve_evidence(&self, _: &ExecutionScope, id: &EvidenceId) -> ServiceResult<EvidenceRef> {
        Err(ErrorEnvelope::new(
            ErrorCode::NotFound,
            format!("{id}"),
            RetryPolicy::Never,
        ))
    }
}

/// Datasets and conditions from the C0/K1 fixtures; records the queries it answered.
#[derive(Default)]
pub struct FakeDatasets {
    pub queries: Mutex<Vec<(ExecutionScope, ConditionsQuery)>>,
}

impl DatasetService for FakeDatasets {
    fn dataset(&self, _: &ExecutionScope, _: &DatasetId) -> ServiceResult<DatasetBundle> {
        Ok(fixture("data/dataset_bundle.json"))
    }
    fn conditions(
        &self,
        scope: &ExecutionScope,
        q: &ConditionsQuery,
    ) -> ServiceResult<ConditionsTable> {
        self.queries
            .lock()
            .unwrap()
            .push((scope.clone(), q.clone()));
        Ok(fixture("data/conditions_table.json"))
    }
}

pub struct FakeModels;

impl ModelService for FakeModels {
    fn releases(&self, _: &ExecutionScope) -> ServiceResult<Vec<ModelRelease>> {
        Ok(vec![fixture("models/release_w0.json")])
    }
    fn release(&self, _: &ExecutionScope, id: &ReleaseId) -> ServiceResult<ModelRelease> {
        let r: ModelRelease = fixture("models/release_w0.json");
        if r.id == id.as_str() {
            Ok(r)
        } else {
            Err(ErrorEnvelope::new(
                ErrorCode::NotFound,
                format!("release {id}"),
                RetryPolicy::Never,
            ))
        }
    }
    fn forecast(&self, _: &ExecutionScope, _: &ForecastRequest) -> ServiceResult<AnalysisArtifact> {
        unreachable!("the tool only queues forecasts; the job runner executes them")
    }
}

pub struct Host {
    pub _dir: tempfile::TempDir,
    pub host: Arc<ToolHost>,
    pub store: Arc<ResearchStore>,
    pub artifacts: Arc<LocalArtifactStore>,
    pub knowledge: Arc<FakeKnowledge>,
    pub wakes: Arc<AtomicUsize>,
}

/// A host with knowledge and models configured (datasets not, no analysis runner).
pub fn host() -> Host {
    host_with(None)
}

/// [`host`] with an analysis runner (a sandbox that passed preflight).
pub fn host_with(sandbox: Option<Arc<ignis_research::sandbox::Sandbox>>) -> Host {
    host_full(sandbox, None)
}

/// [`host`] with a DatasetService.
pub fn host_with_datasets(datasets: Arc<FakeDatasets>) -> Host {
    host_full(None, Some(datasets as Arc<dyn DatasetService>))
}

/// A host with an optional runner and any DatasetService.
pub fn host_full(
    sandbox: Option<Arc<ignis_research::sandbox::Sandbox>>,
    datasets: Option<Arc<dyn DatasetService>>,
) -> Host {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(ResearchStore::open(dir.path().join("state.sqlite")).unwrap());
    let artifacts = Arc::new(LocalArtifactStore::open(dir.path().join("artifacts")).unwrap());
    let knowledge = Arc::new(FakeKnowledge::default());
    let wakes = Arc::new(AtomicUsize::new(0));
    let w = wakes.clone();
    let host = Arc::new(ToolHost::new(
        ToolServices {
            knowledge: Some(knowledge.clone()),
            datasets,
            models: Some(Arc::new(FakeModels)),
            sandbox,
            models_disabled: false,
            conditions: true,
        },
        store.clone(),
        artifacts.clone(),
        Origin::SyntheticFixture,
        Arc::new(move || {
            w.fetch_add(1, Ordering::SeqCst);
        }),
    ));
    Host {
        _dir: dir,
        host,
        store,
        artifacts,
        knowledge,
        wakes,
    }
}

pub fn search_args() -> Value {
    serde_json::to_value(fixture::<SearchRequest>("evidence/search_request.json")).unwrap()
}

pub fn forecast_args(crid: &str) -> Value {
    let release: ModelRelease = fixture("models/release_w0.json");
    json!({
        "schema_version": "wsp.v1",
        "release_id": release.id,
        "input": {"kind": "stored_run", "run_id": "run-fixture-01", "cutoff_s": 2.0},
        "client_request_id": crid,
    })
}

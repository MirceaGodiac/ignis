//! Per-origin application state. The server holds one `Origin` for the real registry and, in dev
//! mode, a second one for the synthetic `/fixtures` dataset; they never share caches, stores or
//! experts, so a fixture result cannot be served as a NASA result.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use ignis_core::analysis::AnalysisStore;
pub use ignis_core::analysis::{write_json, AnalysisContext, StoredAnalysis as Stored};
use ignis_core::contracts::{ModelRelease, SealedPrediction};
use ignis_core::expert::Expert;
use ignis_core::hashing::{hex_part, is_digest};
use ignis_core::records::Registry;
use ignis_core::registry::LoadedRelease;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::watch;

use crate::evidence::Evidence;
use crate::refusal::{ApiResult, Refusal};
use crate::AcquisitionScorer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataOrigin {
    /// The project registry (NASA-derived records as curated by the data lane).
    Registry,
    /// Development fixture — synthetic. Never mixed with the registry.
    DevelopmentFixture,
}

impl DataOrigin {
    pub fn label(self) -> &'static str {
        match self {
            DataOrigin::Registry => "registry",
            DataOrigin::DevelopmentFixture => "development fixture — synthetic",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum JobState {
    Running,
    Completed { analysis_id: String },
    Refused { refusal: Refusal },
}

pub struct Origin {
    pub origin: DataOrigin,
    pub registry: RwLock<Arc<Registry>>,
    pub registry_paths: ignis_core::records::RegistryPaths,
    pub models_root: PathBuf,
    pub store_root: PathBuf,
    /// `data/raw/samples` (BASS recordings for the overlay).
    pub raw_samples_root: PathBuf,
    /// Experts injected by the binary for this origin (looked up by release `model_name`).
    pub experts: Vec<Box<dyn Expert>>,
    pub acquisition: Option<AcquisitionScorer>,
    /// Releases loaded once and tensor-verified (`ignis_core::registry::load`).
    pub releases: Mutex<HashMap<String, Arc<LoadedRelease>>>,
    pub evidence: Evidence,
    pub cache: moka::future::Cache<String, Arc<Stored>>,
    pub jobs: RwLock<HashMap<String, watch::Receiver<JobState>>>,
    pub pool: Arc<rayon::ThreadPool>,
    /// Scope of the unauthenticated legacy routes: outcomes of the training partition only.
    /// Validation outcomes are opened through seal → reveal; test never in this runtime.
    pub public_scope: ignis_core::contracts::ExecutionScope,
    /// Partitions of frozen splits the deployment opened to research for its served releases
    /// (split id -> partitions; `ModelService::opened_splits`, from the served list with its
    /// reason). Outcomes of a run every one of whose partitions is granted or opened are served
    /// like train outcomes, so the lab can show what a held-out run recorded beside its forecast.
    pub opened: std::collections::BTreeMap<
        String,
        std::collections::BTreeSet<ignis_core::contracts::Partition>,
    >,
    /// `/seal`, `/reveal`, `/admin/reload` served (always for the synthetic fixture origin).
    pub internal_routes: bool,
    /// Every reveal attempt, granted or refused (registry: `runs/evaluation/reveal_log.jsonl`).
    pub reveal_log: PathBuf,
}

impl Origin {
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.read().unwrap().clone()
    }

    pub fn reload(&self) -> Arc<Registry> {
        let dir = self.registry().dir.clone();
        let r = Arc::new(Registry::load_with(&dir, &self.registry_paths));
        *self.registry.write().unwrap() = r.clone();
        r
    }

    // ------------------------------------------------------------ releases

    pub fn release_path(&self, id: &str) -> ApiResult<PathBuf> {
        if !is_digest(id) {
            return Err(Refusal::invalid(
                "bad_release_id",
                "release ids are sha256 digests",
            ));
        }
        Ok(self.models_root.join("models").join(hex_part(id)))
    }

    pub fn load_release(&self, id: &str) -> ApiResult<ModelRelease> {
        let dir = self.release_path(id)?;
        let text = std::fs::read(dir.join("release.json")).map_err(|_| {
            Refusal::not_found(
                "release_not_found",
                format!("release {id} not in {}", self.models_root.display()),
            )
        })?;
        let text = String::from_utf8(text).map_err(|_| {
            Refusal::failed(
                "release_encoding",
                "release.json is not UTF-8 (written with a non-UTF-8 platform encoding)",
            )
        })?;
        let r: ModelRelease = ignis_core::registry::parse_release(&text).map_err(|e| {
            Refusal::failed(
                "release_contract",
                format!("release.json violates the contract: {e:#}"),
            )
        })?;
        if hex_part(&r.id) != hex_part(id) {
            return Err(Refusal::failed(
                "release_id_mismatch",
                "release.json id differs from its directory",
            ));
        }
        Ok(r)
    }

    /// Load and verify every tensor of a release (blocking; cached per release id).
    pub fn loaded_release(&self, id: &str) -> ApiResult<Arc<LoadedRelease>> {
        if let Some(r) = self.releases.lock().unwrap().get(id) {
            return Ok(r.clone());
        }
        self.release_path(id)?;
        let r = ignis_core::registry::load(&self.models_root, id).map_err(|e| {
            Refusal::failed(
                "release_verification",
                format!("release {id} failed verification: {e:#}"),
            )
        })?;
        let r = Arc::new(r);
        self.releases
            .lock()
            .unwrap()
            .insert(id.to_string(), r.clone());
        Ok(r)
    }

    pub fn list_releases(&self) -> (Vec<ModelRelease>, Vec<String>) {
        let mut out = Vec::new();
        let mut errs = Vec::new();
        let Ok(rd) = std::fs::read_dir(self.models_root.join("models")) else {
            return (out, errs);
        };
        let mut dirs: Vec<PathBuf> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        for d in dirs {
            let name = d.file_name().unwrap().to_string_lossy().to_string();
            if !is_digest(&name) {
                continue; // .tmp directories of an interrupted save
            }
            match self.load_release(&name) {
                Ok(r) => out.push(r),
                Err(e) => errs.push(format!("{name}: {}", e.reason)),
            }
        }
        (out, errs)
    }

    // ------------------------------------------------------------ analysis store

    fn store(&self) -> AnalysisStore {
        AnalysisStore::new(&self.store_root)
    }

    pub fn save(&self, s: &Stored, prefix: &Value) -> ApiResult<()> {
        self.store()
            .save(s, prefix)
            .map(|_| ())
            .map_err(Refusal::from)
    }

    pub fn load_stored(&self, id: &str) -> ApiResult<Stored> {
        self.store().load(id).map_err(Refusal::from)
    }

    pub fn load_prefix(&self, id: &str) -> ApiResult<Value> {
        self.store().load_prefix(id).map_err(Refusal::from)
    }

    pub fn seal_path(&self, id: &str) -> ApiResult<PathBuf> {
        self.store().dir(id).map_err(Refusal::from)?;
        Ok(self
            .store_root
            .join("sealed")
            .join(format!("{}.json", hex_part(id))))
    }

    pub fn load_seal(&self, id: &str) -> ApiResult<Option<SealedPrediction>> {
        let p = self.seal_path(id)?;
        match std::fs::read_to_string(p) {
            Ok(t) => serde_json::from_str(&t)
                .map(Some)
                .map_err(|e| Refusal::failed("seal_contract", e.to_string())),
            Err(_) => Ok(None),
        }
    }
}

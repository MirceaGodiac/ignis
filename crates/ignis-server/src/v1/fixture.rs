//! Fixture mode of `/api/v1` (`--dev`, mounted at `/api/fixtures/v1`): the same routes and contracts,
//! backed by the hand-made C0 fixtures in `fixtures/contracts/` (every payload
//! `origin = synthetic_fixture`). It lets the UI lane build against real routes before the agent
//! runtime and the lanes' services exist (CONTRACTS.md, "Fixture-uri").
//!
//! - Each workspace sees the fixture episode (`research/episode.json` + `events.json`), imported on
//!   first access under a workspace-suffixed id, and can continue it or create new episodes.
//! - `KnowledgeService` answers every search with `evidence/search_page.json` and every read with
//!   `evidence/read_result.json`.
//! - `ModelService` lists `models/release_w0.json`; a forecast is refused (the fixture has no forward),
//!   which exercises the failed-job path.
//! - The fixture answer artifact is readable; the other fixture artifacts have no bytes.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ignis_core::contracts::{
    AnalysisArtifact, ArtifactId, ArtifactRef, EpisodeId, ErrorCode, ErrorEnvelope, EventBody,
    EvidenceId, EvidenceRef, ExecutionScope, ForecastRequest, ModelRelease, ReadRequest,
    ReadResult, ReleaseId, ResearchEpisode, ResearchEvent, RetryPolicy, SearchPage, SearchRequest,
};
use ignis_core::services::{
    ArtifactDraft, ArtifactStore, KnowledgeService, ModelService, ServiceResult,
};
use serde::de::DeserializeOwned;

use super::ResearchServices;

fn read<T: DeserializeOwned>(root: &Path, rel: &str) -> anyhow::Result<T> {
    let p = root.join(rel);
    let text =
        std::fs::read_to_string(&p).map_err(|e| anyhow::anyhow!("fixture {}: {e}", p.display()))?;
    serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("fixture {}: {e}", p.display()))
}

fn unsupported(msg: &str) -> ErrorEnvelope {
    ErrorEnvelope::new(ErrorCode::Unsupported, msg, RetryPolicy::Never)
}

/// The episode every fixture workspace starts with.
pub struct FixtureSeed {
    episode: ResearchEpisode,
    events: Vec<ResearchEvent>,
}

impl FixtureSeed {
    /// The fixture episode and log under an id unique to `scope`'s workspace.
    pub fn for_scope(&self, scope: &ExecutionScope) -> (ResearchEpisode, Vec<ResearchEvent>) {
        let id = EpisodeId::new(format!(
            "{}~{}",
            self.episode.episode_id, scope.workspace_id
        ))
        .expect("fixture id");
        let mut ep = self.episode.clone();
        ep.episode_id = id.clone();
        let events = self
            .events
            .iter()
            .cloned()
            .map(|mut e| {
                e.episode_id = id.clone();
                e
            })
            .collect();
        (ep, events)
    }
}

pub struct FixtureKnowledge {
    page: SearchPage,
    read: ReadResult,
}

impl KnowledgeService for FixtureKnowledge {
    fn search(&self, _: &ExecutionScope, _: &SearchRequest) -> ServiceResult<SearchPage> {
        Ok(self.page.clone())
    }
    fn read(&self, _: &ExecutionScope, _: &ReadRequest) -> ServiceResult<ReadResult> {
        Ok(self.read.clone())
    }
    fn resolve_evidence(&self, _: &ExecutionScope, id: &EvidenceId) -> ServiceResult<EvidenceRef> {
        let v = serde_json::to_value(&self.page).unwrap_or_default();
        let found = v["hits"].as_array().into_iter().flatten().find_map(|h| {
            let e = h.get("evidence")?;
            (e.get("evidence_id")?.as_str()? == id.as_str())
                .then(|| serde_json::from_value::<EvidenceRef>(e.clone()).ok())
                .flatten()
        });
        found.ok_or_else(|| {
            ErrorEnvelope::new(
                ErrorCode::NotFound,
                format!("evidence {id} is not in the fixture"),
                RetryPolicy::Never,
            )
        })
    }
}

pub struct FixtureModels {
    release: ModelRelease,
}

impl ModelService for FixtureModels {
    fn releases(&self, _: &ExecutionScope) -> ServiceResult<Vec<ModelRelease>> {
        Ok(vec![self.release.clone()])
    }
    fn release(&self, _: &ExecutionScope, id: &ReleaseId) -> ServiceResult<ModelRelease> {
        if self.release.id == id.as_str() {
            Ok(self.release.clone())
        } else {
            Err(ErrorEnvelope::new(
                ErrorCode::NotFound,
                format!("release {id} is not in the fixture"),
                RetryPolicy::Never,
            ))
        }
    }
    fn forecast(&self, _: &ExecutionScope, _: &ForecastRequest) -> ServiceResult<AnalysisArtifact> {
        Err(unsupported(
            "the C0 fixture has no forward; models_forecast is served by the ModelService (I2)",
        ))
    }
}

/// Fixture artifacts referenced by the fixture log, over a real store for everything new.
pub struct FixtureArtifacts {
    inner: Arc<dyn ArtifactStore>,
    known: Vec<(ArtifactRef, Option<PathBuf>)>,
}

impl FixtureArtifacts {
    fn find(&self, id: &ArtifactId) -> Option<&(ArtifactRef, Option<PathBuf>)> {
        self.known.iter().find(|(r, _)| &r.artifact_id == id)
    }
}

impl ArtifactStore for FixtureArtifacts {
    fn put(
        &self,
        scope: &ExecutionScope,
        draft: ArtifactDraft,
        bytes: &mut dyn Read,
    ) -> ServiceResult<ArtifactRef> {
        self.inner.put(scope, draft, bytes)
    }
    fn stat(&self, scope: &ExecutionScope, id: &ArtifactId) -> ServiceResult<ArtifactRef> {
        match self.find(id) {
            Some((r, _)) => Ok(r.clone()),
            None => self.inner.stat(scope, id),
        }
    }
    fn open(
        &self,
        scope: &ExecutionScope,
        id: &ArtifactId,
        offset: u64,
        len: Option<u64>,
    ) -> ServiceResult<Box<dyn Read + Send>> {
        let Some((_, path)) = self.find(id) else {
            return self.inner.open(scope, id, offset, len);
        };
        let Some(path) = path else {
            return Err(ErrorEnvelope::new(
                ErrorCode::NotFound,
                format!("fixture artifact {id} has no bytes (only its reference is a fixture)"),
                RetryPolicy::Never,
            ));
        };
        let bytes = std::fs::read(path).map_err(|e| {
            ErrorEnvelope::new(ErrorCode::Internal, e.to_string(), RetryPolicy::Never)
        })?;
        let start = (offset as usize).min(bytes.len());
        let end = len.map_or(bytes.len(), |l| (start + l as usize).min(bytes.len()));
        Ok(Box::new(std::io::Cursor::new(bytes[start..end].to_vec())))
    }
}

/// Everything fixture mode needs, loaded from `fixtures/contracts/`.
pub struct Fixture {
    pub seed: FixtureSeed,
    pub services: ResearchServices,
    known: Vec<(ArtifactRef, Option<PathBuf>)>,
}

impl Fixture {
    pub fn load(root: &Path) -> anyhow::Result<Fixture> {
        let episode: ResearchEpisode = read(root, "research/episode.json")?;
        let events: Vec<ResearchEvent> = read(root, "research/events.json")?;
        let page: SearchPage = read(root, "evidence/search_page.json")?;
        let read_result: ReadResult = read(root, "evidence/read_result.json")?;
        let release: ModelRelease = read(root, "models/release_w0.json")?;
        let answer_file = root.join("research/answer.json");
        let known = events
            .iter()
            .filter_map(|e| match &e.body {
                EventBody::AnswerCompleted { answer, .. } => {
                    Some((answer.clone(), Some(answer_file.clone())))
                }
                EventBody::ArtifactCreated { artifact } => Some((artifact.clone(), None)),
                _ => None,
            })
            .collect();
        Ok(Fixture {
            seed: FixtureSeed { episode, events },
            services: ResearchServices {
                knowledge: Some(Arc::new(FixtureKnowledge {
                    page,
                    read: read_result,
                })),
                datasets: None,
                models: Some(Arc::new(FixtureModels { release })),
                sandbox: None,
                models_disabled: false,
            },
            known,
        })
    }

    pub fn artifacts(&self, inner: Arc<dyn ArtifactStore>) -> FixtureArtifacts {
        FixtureArtifacts {
            inner,
            known: self.known.clone(),
        }
    }
}

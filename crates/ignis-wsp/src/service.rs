//! The [`ModelService`] of the literal models: the one entry point the HTTP API, the MCP transport
//! and the evaluator use to list literal releases and run their strict forwards.
//!
//! * A release is routed by its `model_name` to the forward compiled into this crate
//!   ([`crate::artifact_of`]); the family alone never chooses a forward.
//! * A release is read from the content-addressed registry once per id and kept (verified tensors
//!   included) in the service; concurrent requests for the same id wait for the one load.
//! * The admitted prefix comes from a [`PrefixSource`]. The service tells the source the release
//!   and the prospective policy of the release's own model, asks it first where the data comes from
//!   ([`Provenance`]), checks that against the host's [`ExecutionScope`], and only then asks for the
//!   prefix. A test-partition run is refused before its prefix is built.
//! * Scientific refusals of a forward are artifacts (`support.admitted = false`) and are returned as
//!   they are. A forward that cannot run the release at all (no literal forward, forward or spec
//!   mismatch) is an `unsupported` [`ErrorEnvelope`]: there is no artifact a forward produced.
//!
//! Nothing here fits, defaults or falls back: the crate links no fitted expert, and whatever the
//! forward refuses stays refused.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ignis_core::analysis::{build_run_prefix, validate_artifact};
use ignis_core::contracts::{
    AnalysisArtifact, Capability, ErrorCode, ErrorEnvelope, EvidenceClass, ExecutionScope,
    ForecastInput, ForecastRequest, ModelRelease, Partition, PrefixBundle, ReleaseId, RetryPolicy,
};
use ignis_core::expert::{Refusal, RefusalKind};
use ignis_core::hashing::{hex_part, is_digest};
use ignis_core::prefix::{assert_isolated, ProspectivePolicy, ALWAYS_FORBIDDEN};
use ignis_core::registry::{self, LoadedRelease};
use ignis_core::services::{ModelService, ServiceResult};
use ignis_core::views::ViewSource;

use crate::models;

fn envelope(code: ErrorCode, message: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, message, RetryPolicy::Never)
}

/// A refusal of the forward (or of the shared prefix builder) that carries no artifact, as the
/// contract's error. `unsupported` and `insufficient_inference` without an artifact are the release
/// or the input being outside what this build can run; `failed` is a broken release or a bug.
pub fn envelope_of(r: Refusal) -> ErrorEnvelope {
    let code = match r.kind {
        RefusalKind::Unsupported | RefusalKind::InsufficientInference => ErrorCode::Unsupported,
        RefusalKind::InvalidRequest => ErrorCode::InvalidRequest,
        RefusalKind::NotFound => ErrorCode::NotFound,
        RefusalKind::Failed => ErrorCode::Internal,
    };
    envelope(code, format!("{}: {}", r.code, r.reason))
}

// ---------------------------------------------------------------- prefix sources

/// Where the data of a forecast input comes from, as the source knows it. The service enforces it
/// against the scope; a source never decides access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// A run of the release's frozen split, in this partition.
    Partition(Partition),
    /// A synthetic fixture of no split (`evidence_class = synthetic` throughout). The service checks
    /// that every observation and contour of the prefix is synthetic; anything else is refused.
    SyntheticFixture,
}

/// "Give me the admitted prefix of this request." Implemented by the host's data access; the
/// service calls [`PrefixSource::provenance`] first and [`PrefixSource::prefix`] only when the scope
/// grants that provenance. `policy` is the prospective policy of the release's own model: a prefix
/// built from stored records must be built with it (it decides which conditions the prefix carries).
pub trait PrefixSource: Send + Sync {
    fn provenance(
        &self,
        scope: &ExecutionScope,
        release: &ModelRelease,
        input: &ForecastInput,
    ) -> ServiceResult<Provenance>;

    fn prefix(
        &self,
        scope: &ExecutionScope,
        release: &ModelRelease,
        policy: &ProspectivePolicy,
        input: &ForecastInput,
    ) -> ServiceResult<PrefixBundle>;
}

/// Prefixes held in memory: admitted prefixes by `input_hash` and stored-run prefixes by
/// `(run_id, cutoff_s)`, each with its provenance. For tests and fixtures; it ignores `policy`
/// (its prefixes are already built), and the service checks the prefix against the policy anyway.
#[derive(Debug, Default, Clone)]
pub struct MemoryPrefixes {
    by_hash: HashMap<String, (PrefixBundle, Provenance)>,
    by_run: HashMap<(String, u64), (PrefixBundle, Provenance)>,
}

impl MemoryPrefixes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Serves `prefix` for `ForecastInput::Prefix { input_hash: prefix.input_hash }`.
    pub fn with_prefix(mut self, prefix: PrefixBundle, provenance: Provenance) -> Self {
        self.by_hash
            .insert(prefix.input_hash.clone(), (prefix, provenance));
        self
    }

    /// Serves `prefix` for `ForecastInput::StoredRun { run_id: prefix.run_id, cutoff_s: prefix.cutoff_s }`.
    pub fn with_run(mut self, prefix: PrefixBundle, provenance: Provenance) -> Self {
        let key = (prefix.run_id.clone(), prefix.cutoff_s.to_bits());
        self.by_run.insert(key, (prefix, provenance));
        self
    }

    fn find(&self, input: &ForecastInput) -> ServiceResult<&(PrefixBundle, Provenance)> {
        match input {
            ForecastInput::Prefix { input_hash } => self.by_hash.get(input_hash).ok_or_else(|| {
                envelope(
                    ErrorCode::NotFound,
                    format!("no admitted prefix {input_hash}"),
                )
            }),
            ForecastInput::StoredRun { run_id, cutoff_s } => self
                .by_run
                .get(&(run_id.as_str().to_string(), cutoff_s.get().to_bits()))
                .ok_or_else(|| {
                    envelope(
                        ErrorCode::NotFound,
                        format!("no stored run {run_id} at cutoff {}", cutoff_s.get()),
                    )
                }),
        }
    }
}

impl PrefixSource for MemoryPrefixes {
    fn provenance(
        &self,
        _: &ExecutionScope,
        _: &ModelRelease,
        input: &ForecastInput,
    ) -> ServiceResult<Provenance> {
        Ok(self.find(input)?.1)
    }

    fn prefix(
        &self,
        _: &ExecutionScope,
        _: &ModelRelease,
        _: &ProspectivePolicy,
        input: &ForecastInput,
    ) -> ServiceResult<PrefixBundle> {
        Ok(self.find(input)?.0.clone())
    }
}

/// Stored runs through a [`ViewSource`] (in production `ignis_data::views::DataStore`): the
/// partition of a run is its partition in the release's own frozen split (`release.split_id`), and
/// the prefix is `ignis_core::analysis::build_run_prefix` with the model's policy, the same builder
/// `ignis infer` and `wsp-dev` use, so the same run and cutoff give the same `input_hash`.
///
/// It serves no `ForecastInput::Prefix`: no store of admitted prefixes by `input_hash` exists in
/// `ignis-core`, so such a request is `capability_unavailable`.
pub struct ViewSourcePrefixes<V> {
    source: V,
}

impl<V: ViewSource + Send + Sync> ViewSourcePrefixes<V> {
    pub fn new(source: V) -> Self {
        ViewSourcePrefixes { source }
    }
}

impl<V: ViewSource + Send + Sync> PrefixSource for ViewSourcePrefixes<V> {
    fn provenance(
        &self,
        _: &ExecutionScope,
        release: &ModelRelease,
        input: &ForecastInput,
    ) -> ServiceResult<Provenance> {
        let ForecastInput::StoredRun { run_id, .. } = input else {
            return Err(envelope(
                ErrorCode::CapabilityUnavailable,
                "this deployment keeps no admitted prefixes by input_hash; forecast a stored run",
            ));
        };
        let split = self.source.split(&release.split_id).map_err(|e| {
            envelope(
                ErrorCode::CapabilityUnavailable,
                format!(
                    "split {} of release {} is not available, so the partition of run {run_id} \
                     cannot be established: {e:#}",
                    release.split_id, release.id
                ),
            )
        })?;
        split
            .partition_of(run_id.as_str())
            .map(Provenance::Partition)
            .ok_or_else(|| {
                envelope(
                    ErrorCode::ScopeDenied,
                    format!(
                        "run {run_id} is not in split {} of release {}: no partition, so no scope \
                         grants it",
                        split.id, release.id
                    ),
                )
            })
    }

    fn prefix(
        &self,
        _: &ExecutionScope,
        release: &ModelRelease,
        policy: &ProspectivePolicy,
        input: &ForecastInput,
    ) -> ServiceResult<PrefixBundle> {
        let ForecastInput::StoredRun { run_id, cutoff_s } = input else {
            return Err(envelope(
                ErrorCode::CapabilityUnavailable,
                "this deployment keeps no admitted prefixes by input_hash; forecast a stored run",
            ));
        };
        let archive = self
            .source
            .run_archive(release.family, run_id.as_str())
            .map_err(|e| {
                envelope(
                    ErrorCode::CapabilityUnavailable,
                    format!(
                        "records of family {} are not available: {e:#}",
                        release.family.as_str()
                    ),
                )
            })?
            .ok_or_else(|| {
                envelope(
                    ErrorCode::NotFound,
                    format!(
                        "run {run_id} is not a stored {} run",
                        release.family.as_str()
                    ),
                )
            })?;
        build_run_prefix(&archive, cutoff_s.get(), policy).map_err(envelope_of)
    }
}

// ---------------------------------------------------------------- the service

type Slot = Arc<Mutex<Option<Arc<LoadedRelease>>>>;

/// The literal models' [`ModelService`], generic over where prefixes come from.
pub struct WspModelService<P> {
    root: PathBuf,
    prefixes: P,
    /// Prospective policy of every literal model, by `model_name` (from [`crate::experts`]).
    policies: HashMap<String, ProspectivePolicy>,
    /// Loaded releases by the hex part of their id. Only successful loads are kept.
    cache: Mutex<HashMap<String, Slot>>,
    loads: AtomicU64,
}

impl<P: PrefixSource> WspModelService<P> {
    /// `registry_root` is the root of the content-addressed registry (`registry::default_root()`
    /// in production: `IGNIS_ARTIFACTS` or `artifacts`); `prefixes` builds admitted prefixes.
    pub fn new(registry_root: impl Into<PathBuf>, prefixes: P) -> Self {
        let policies = crate::experts()
            .iter()
            .map(|e| (e.model_name().to_string(), e.policy().clone()))
            .collect();
        WspModelService {
            root: registry_root.into(),
            prefixes,
            policies,
            cache: Mutex::new(HashMap::new()),
            loads: AtomicU64::new(0),
        }
    }

    pub fn registry_root(&self) -> &Path {
        &self.root
    }

    /// How many times a release was read from the registry (each id once while the service lives).
    pub fn release_loads(&self) -> u64 {
        self.loads.load(Ordering::SeqCst)
    }

    /// The release, verified and loaded once per id.
    fn loaded(&self, id: &ReleaseId) -> ServiceResult<Arc<LoadedRelease>> {
        let not_found = || {
            envelope(
                ErrorCode::NotFound,
                format!("release {id} is not in the registry"),
            )
        };
        // Only a digest names a registry directory; anything else is never a path.
        if !is_digest(id.as_str()) {
            return Err(not_found());
        }
        let key = hex_part(id.as_str()).to_string();
        let slot = {
            let mut cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
            cache.entry(key.clone()).or_default().clone()
        };
        let mut slot = slot.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(r) = slot.as_ref() {
            return Ok(r.clone());
        }
        if !registry::release_dir(&self.root, &key)
            .join("release.json")
            .is_file()
        {
            return Err(not_found());
        }
        let loaded = registry::load(&self.root, &key).map_err(|e| {
            envelope(
                ErrorCode::Internal,
                format!("release {id} could not be loaded: {e:#}"),
            )
        })?;
        self.loads.fetch_add(1, Ordering::SeqCst);
        let loaded = Arc::new(loaded);
        *slot = Some(loaded.clone());
        Ok(loaded)
    }

    fn policy_of(&self, release: &ModelRelease) -> Option<&ProspectivePolicy> {
        runnable(release)
            .then(|| self.policies.get(&release.model_name))
            .flatten()
    }
}

/// A literal release this build can run: its `model_name` is a literal model compiled here and it
/// carries a construction manifest.
pub fn runnable(release: &ModelRelease) -> bool {
    release.construction.is_some() && models::by_name(&release.model_name).is_some()
}

/// The prefix the source returned, checked against what was asked and against the model's policy.
fn check_prefix(
    prefix: &PrefixBundle,
    release: &ModelRelease,
    policy: &ProspectivePolicy,
    input: &ForecastInput,
    provenance: Provenance,
) -> ServiceResult<()> {
    assert_isolated(prefix, &[])
        .map_err(|e| envelope(ErrorCode::Internal, format!("prefix source: {e}")))?;
    let matches = match input {
        ForecastInput::Prefix { input_hash } => prefix.input_hash == *input_hash,
        ForecastInput::StoredRun { run_id, cutoff_s } => {
            prefix.run_id == run_id.as_str() && prefix.cutoff_s == cutoff_s.get()
        }
    };
    if !matches {
        return Err(envelope(
            ErrorCode::Internal,
            "prefix source returned a prefix for a different input",
        ));
    }
    if prefix.family != release.family {
        return Err(envelope(
            ErrorCode::Unsupported,
            format!(
                "release {} serves {}, prefix {} is {}",
                release.model_name,
                release.family.as_str(),
                prefix.input_hash,
                prefix.family.as_str()
            ),
        ));
    }
    if let Some(k) = prefix.known_conditions.keys().find(|k| {
        !policy.allowed_conditions.contains(*k)
            || policy.forbidden_conditions.contains(*k)
            || ALWAYS_FORBIDDEN.contains(&k.as_str())
    }) {
        return Err(envelope(
            ErrorCode::Unsupported,
            format!(
                "prefix {} carries condition {k:?}, which the prospective policy of {} does not \
                 admit: it was not built for this model",
                prefix.input_hash, release.model_name
            ),
        ));
    }
    if provenance == Provenance::SyntheticFixture {
        let real = prefix
            .observations
            .iter()
            .map(|o| (&o.id, o.evidence_class))
            .chain(prefix.contours.iter().map(|c| (&c.id, c.evidence_class)))
            .find(|(_, e)| *e != EvidenceClass::Synthetic);
        if let Some((id, e)) = real {
            return Err(envelope(
                ErrorCode::ScopeDenied,
                format!(
                    "prefix {} was declared a synthetic fixture but {id} is {e:?}: its partition \
                     is unknown",
                    prefix.input_hash
                ),
            ));
        }
    }
    Ok(())
}

impl<P: PrefixSource> ModelService for WspModelService<P> {
    fn releases(&self, _: &ExecutionScope) -> ServiceResult<Vec<ModelRelease>> {
        let mut v: Vec<ModelRelease> = registry::list_releases(&self.root)
            .into_iter()
            .filter(runnable)
            .collect();
        v.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
        Ok(v)
    }

    fn release(&self, _: &ExecutionScope, id: &ReleaseId) -> ServiceResult<ModelRelease> {
        let loaded = self.loaded(id)?;
        if !runnable(&loaded.release) {
            return Err(envelope(
                ErrorCode::NotFound,
                format!("release {id} is not a literal release this build can run"),
            ));
        }
        Ok(loaded.release.clone())
    }

    fn forecast(
        &self,
        scope: &ExecutionScope,
        req: &ForecastRequest,
    ) -> ServiceResult<AnalysisArtifact> {
        req.validate()?;
        scope.validate().map_err(|e| {
            envelope(
                ErrorCode::ScopeDenied,
                format!("scope {}: {}", scope.scope_id, e.reason),
            )
        })?;
        scope.require(Capability::RunModels)?;
        scope.require(Capability::ReadData)?;
        let loaded = self.loaded(&req.release_id)?;
        let release = &loaded.release;
        let policy = self.policy_of(release).ok_or_else(|| {
            envelope(
                ErrorCode::Unsupported,
                format!(
                    "no_literal_forward: {} ({}) is not a literal release this build can run",
                    release.id, release.model_name
                ),
            )
        })?;
        let provenance = self.prefixes.provenance(scope, release, &req.input)?;
        if let Provenance::Partition(p) = provenance {
            scope.require_partition(p)?;
        }
        let prefix = self.prefixes.prefix(scope, release, policy, &req.input)?;
        check_prefix(&prefix, release, policy, &req.input, provenance)?;
        let artifact =
            crate::artifact_of(&loaded, &prefix, &req.settings.value()).map_err(envelope_of)?;
        validate_artifact(&artifact, release, &prefix).map_err(envelope_of)?;
        Ok(artifact)
    }
}

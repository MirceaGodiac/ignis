//! Common experiment interface around specialised fitting implementations.
//!
//! Each family keeps its own fitting algorithm (NUTS, MAP, bootstrap ensembles). What is shared is
//! the surface the CLI, evaluator, decision layer, server and lab tools rely on. Model crates expose
//! `pub fn experts() -> Vec<Box<dyn Expert>>`; the `ignis` binary collects them.

use std::path::PathBuf;

use serde_json::Value;

use crate::contracts::{AnalysisArtifact, Family, ModelRelease, PrefixBundle, Split, Support};
use crate::prefix::ProspectivePolicy;
use crate::registry::{LoadedRelease, TensorMap};
use crate::resources::Profile;
use crate::views::View;

/// Everything a fit may use. Experts must fit on `train` only; `validation` is for selection /
/// early stopping. The test partition is never handed to `fit`.
pub struct FitData<'a> {
    pub views: &'a [View],
    pub split: &'a Split,
    pub cohort_id: &'a str,
    pub source_manifest_hash: &'a str,
    pub synthetic: bool,
}

impl FitData<'_> {
    pub fn partition(&self, name: &str) -> Vec<&View> {
        self.views
            .iter()
            .filter(|v| {
                let p = self.split.partition_of(v.run_id());
                matches!(
                    (name, p),
                    ("train", Some(crate::contracts::Partition::Train))
                        | ("validation", Some(crate::contracts::Partition::Validation))
                )
            })
            .collect()
    }
}

/// Execution context handed to `fit`.
pub struct FitContext {
    pub profile: Profile,
    pub pool: rayon::ThreadPool,
    pub seed: u64,
    pub registry_root: PathBuf,
    /// Directory for resumable checkpoints of this job (`runs/checkpoints/<job>`).
    pub checkpoint_dir: PathBuf,
    /// Warm start / resume from this release (a `fine_tune` child is produced).
    pub resume: Option<LoadedRelease>,
    pub job_id: Option<String>,
}

pub type ExpertResult<T> = anyhow::Result<T>;

/// A scientific refusal an expert returns (as the error of an [`ExpertResult`]) when it must not
/// answer: the server maps it to a typed `unsupported` / `insufficient_inference` response instead
/// of `failed`. Return it with `Err(Refusal::unsupported("code", "reason").into())`.
#[derive(Debug, Clone, thiserror::Error, serde::Serialize)]
#[error("{kind:?} {code}: {reason}")]
pub struct Refusal {
    pub kind: RefusalKind,
    pub code: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalKind {
    Unsupported,
    InsufficientInference,
    InvalidRequest,
    NotFound,
    Failed,
}

impl Refusal {
    pub fn unsupported(code: &str, reason: impl Into<String>) -> Self {
        Refusal {
            kind: RefusalKind::Unsupported,
            code: code.into(),
            reason: reason.into(),
        }
    }
    pub fn insufficient(code: &str, reason: impl Into<String>) -> Self {
        Refusal {
            kind: RefusalKind::InsufficientInference,
            code: code.into(),
            reason: reason.into(),
        }
    }
    pub fn invalid(code: &str, reason: impl Into<String>) -> Self {
        Refusal {
            kind: RefusalKind::InvalidRequest,
            code: code.into(),
            reason: reason.into(),
        }
    }
    pub fn not_found(code: &str, reason: impl Into<String>) -> Self {
        Refusal {
            kind: RefusalKind::NotFound,
            code: code.into(),
            reason: reason.into(),
        }
    }
    pub fn failed(code: &str, reason: impl Into<String>) -> Self {
        Refusal {
            kind: RefusalKind::Failed,
            code: code.into(),
            reason: reason.into(),
        }
    }
    /// Recover a refusal from an expert error, if that is what it is.
    pub fn from_error(e: &anyhow::Error) -> Option<&Refusal> {
        e.downcast_ref::<Refusal>()
    }
}

pub trait Expert: Send + Sync {
    fn family(&self) -> Family;
    fn model_name(&self) -> &str;
    fn is_baseline(&self) -> bool;
    fn policy(&self) -> &ProspectivePolicy;
    /// Full configuration (from `configs/models/...` merged over built-in defaults).
    fn default_config(&self) -> ExpertResult<Value>;
    /// Rough declared planning estimate (hours) for `n_runs`/`n_obs` under a profile.
    fn estimate_hours(&self, config: &Value, n_runs: usize, n_obs: usize, profile: &Profile)
        -> f64;

    fn fit(
        &self,
        data: &FitData<'_>,
        config: &Value,
        ctx: &mut FitContext,
    ) -> ExpertResult<ModelRelease>;
    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact>;
    /// Held-out log predictive density of one view (evaluation, lab).
    fn log_likelihood(&self, release: &LoadedRelease, view: &View) -> ExpertResult<f64>;

    /// Differentiable objective for the lab tools: value and gradient w.r.t. every parameter tensor.
    /// `partition` is "train" or "validation"; never "test".
    fn objective(
        &self,
        _release: &LoadedRelease,
        _params: &TensorMap,
        _views: &[&View],
    ) -> Option<ExpertResult<(f64, TensorMap)>> {
        None
    }
    /// Named intermediate arrays for one view.
    fn activations(
        &self,
        _release: &LoadedRelease,
        _params: &TensorMap,
        _view: &View,
    ) -> Option<ExpertResult<TensorMap>> {
        None
    }
    /// Candidate-measurement model used by observation-value ranking, when this expert defines one.
    fn candidate_predictive(&self) -> Option<&dyn CandidatePredictive> {
        None
    }
}

/// Current posterior particles and the exact predictive of each candidate measurement (V3.1 ch. 11).
#[derive(Debug, Clone)]
pub struct VoiParticles {
    pub support: Support,
    /// Normalised particle weights (empty when refused).
    pub w: Vec<f64>,
    /// (P, J) location / squared scale of the candidate measurement given the prefix.
    pub pred_loc: Vec<Vec<f64>>,
    pub pred_scale2: Vec<Vec<f64>>,
    /// (P,) degrees of freedom.
    pub dof: Vec<f64>,
    pub horizon_s: f64,
    pub particles: Value,
    /// (P, K) event-category probabilities at the horizon (event models only).
    pub z_event: Option<Vec<Vec<f64>>>,
    pub event_categories: Vec<String>,
    pub z_diameter: Vec<f64>,
    pub z_diameter_weight: Vec<f64>,
}

/// A model that can predict a not-yet-admitted pre-cutoff measurement from the prefix alone.
/// Candidate *values* are never an input.
pub trait CandidatePredictive: Send + Sync {
    fn model_name(&self) -> &str;
    fn has_event_model(&self) -> bool;
    fn voi_particles(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        candidate_times: &[f64],
        settings: &Value,
    ) -> anyhow::Result<VoiParticles>;
}

/// Find an expert by `model_name` in a collected list.
pub fn find<'a>(experts: &'a [Box<dyn Expert>], name: &str) -> Option<&'a dyn Expert> {
    experts
        .iter()
        .find(|e| e.model_name() == name)
        .map(|b| b.as_ref())
}

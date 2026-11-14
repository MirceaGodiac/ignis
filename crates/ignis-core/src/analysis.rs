//! The forecast path shared by `ignis serve` and `ignis infer`: forecast settings, the causal prefix
//! of a stored run at a cutoff, the expert call with artifact validation, and the write-once analysis
//! store. Serving and the CLI call these functions, so the same run, cutoff, settings and release give
//! the same content-addressed artifact.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::contracts::{AnalysisArtifact, ModelRelease, PrefixBundle, TrainedOn};
use crate::expert::{Expert, Refusal};
use crate::hashing::{hex_part, is_digest, key_of};
use crate::prefix::{build_prefix, PrefixInputs, ProspectivePolicy};
use crate::registry::LoadedRelease;
use crate::resources::configs_dir;
use crate::views::{measurement_valid, RunArchive};

type Refused<T> = Result<T, Refusal>;

/// Settings of a forecast request (strict: unknown fields are refused).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForecastSettings {
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub quantile_levels: Option<Vec<f64>>,
    #[serde(default)]
    pub horizon_s: Option<f64>,
    #[serde(default)]
    pub n_points: Option<u32>,
    /// Explicit forecast times (s since the time origin). Lets cases at different cutoffs share one
    /// time grid so that event metrics compare like with like.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub times_s: Option<Vec<f64>>,
}

impl ForecastSettings {
    pub fn parse(json: Option<&str>) -> Refused<ForecastSettings> {
        match json {
            None => Ok(ForecastSettings::default()),
            Some(t) => serde_json::from_str(t)
                .map_err(|e| Refusal::invalid("settings", format!("--settings: {e}"))),
        }
    }

    pub fn validate(&self) -> Refused<()> {
        if let Some(l) = &self.quantile_levels {
            if l.is_empty()
                || l.len() > 21
                || l.iter().any(|q| !(*q > 0.0 && *q < 1.0))
                || l.windows(2).any(|w| w[1] <= w[0])
            {
                return Err(Refusal::invalid(
                    "quantile_levels",
                    "1-21 strictly increasing levels in (0,1)",
                ));
            }
        }
        if self.horizon_s.is_some_and(|h| !(h.is_finite() && h > 0.0)) {
            return Err(Refusal::invalid(
                "horizon_s",
                "horizon_s must be positive and finite",
            ));
        }
        if self.n_points.is_some_and(|n| !(2..=2000).contains(&n)) {
            return Err(Refusal::invalid("n_points", "n_points in [2, 2000]"));
        }
        if let Some(t) = &self.times_s {
            if t.is_empty()
                || t.len() > 2000
                || t.iter().any(|x| !x.is_finite())
                || t.windows(2).any(|w| w[1] <= w[0])
            {
                return Err(Refusal::invalid(
                    "times_s",
                    "1-2000 strictly increasing finite times",
                ));
            }
        }
        Ok(())
    }

    /// The JSON value handed to the expert and hashed into the artifact.
    pub fn value(&self) -> Value {
        serde_json::to_value(self).expect("settings serialize")
    }
}

/// Phase entry at the cutoff, if the modelled phase is established then (V3.1 §23.4): the time origin
/// of the curated views is the phase entry, and activity must be confirmed at the cutoff. Activity is
/// one bit known to an observer at the cutoff (was the run still burning then); the end time itself
/// never enters the prefix.
pub fn phase_established_at(a: &RunArchive, cutoff_s: f64) -> Option<f64> {
    if !(cutoff_s.is_finite() && cutoff_s >= 0.0) {
        return None;
    }
    let (terminal, confirmed) = a.endpoint.as_ref().map_or((None, None), |e| {
        (e.first_terminal_s, e.last_confirmed_active_s)
    });
    if terminal.is_some_and(|t| t <= cutoff_s) {
        return None;
    }
    let active = match (confirmed, terminal) {
        (Some(c), _) => c >= cutoff_s,
        (None, Some(_)) => true,
        (None, None) => {
            a.observations
                .iter()
                .filter(|o| measurement_valid(o))
                .map(|o| o.time_s)
                .fold(f64::NEG_INFINITY, f64::max)
                >= cutoff_s
        }
    };
    active.then_some(0.0)
}

/// A run enters a landmark cohort only while its phase is established at the cutoff.
pub fn at_risk(a: &RunArchive, cutoff_s: f64) -> bool {
    phase_established_at(a, cutoff_s).is_some()
}

/// The causal prefix of a stored run at a cutoff, with the expert's own prospective policy.
pub fn build_run_prefix(
    a: &RunArchive,
    cutoff_s: f64,
    policy: &ProspectivePolicy,
) -> Refused<PrefixBundle> {
    let contours: Vec<_> = a
        .contours
        .iter()
        .filter(|c| c.available_at_s <= cutoff_s && c.time_s <= cutoff_s)
        .cloned()
        .collect();
    let known_controls: Vec<_> = a.control.as_ref().map_or(vec![], |c| {
        c.stages
            .iter()
            .filter(|s| s.t_start_s <= cutoff_s)
            .cloned()
            .collect()
    });
    let phase = phase_established_at(a, cutoff_s);
    let support_evidence = if phase.is_some() {
        vec!["activity confirmed at the cutoff by the run record".to_string()]
    } else {
        vec![]
    };
    let prefix = build_prefix(
        &a.record,
        &a.observations,
        cutoff_s,
        policy,
        &a.time_origin,
        PrefixInputs {
            contours: &contours,
            known_controls: &known_controls,
            phase_established_at_s: phase,
            support_evidence,
            ..Default::default()
        },
    )
    .map_err(|e| Refusal::unsupported("prefix_refused", format!("prefix refused: {e}")))?;
    if prefix.observations.iter().any(|x| !x.admitted_at(cutoff_s)) {
        return Err(Refusal::failed(
            "prefix_mismatch",
            "prefix holds an observation not admitted at the cutoff",
        ));
    }
    Ok(prefix)
}

/// Check every precondition of a forecast and build the prefix. Returns the expert's position in
/// `experts`, the prefix and the settings value.
pub fn plan_forecast(
    experts: &[Box<dyn Expert>],
    release: &ModelRelease,
    archive: &RunArchive,
    cutoff_s: f64,
    settings: &ForecastSettings,
    allow_synthetic_release: bool,
    origin_label: &str,
) -> Refused<(usize, PrefixBundle, Value)> {
    if !cutoff_s.is_finite() {
        return Err(Refusal::invalid("cutoff", "cutoff_s must be finite"));
    }
    settings.validate()?;
    let rec = &archive.record;
    if release.family != rec.family {
        return Err(Refusal::unsupported(
            "family_mismatch",
            format!(
                "release {} serves {}, run {} is {}",
                release.model_name,
                release.family.as_str(),
                rec.id,
                rec.family.as_str()
            ),
        ));
    }
    if release.trained_on == TrainedOn::Synthetic && !allow_synthetic_release {
        return Err(Refusal::unsupported(
            "synthetic_release",
            "a synthetic release cannot forecast registry runs",
        ));
    }
    let ei = experts
        .iter()
        .position(|e| e.model_name() == release.model_name)
        .ok_or_else(|| {
            Refusal::unsupported(
                "no_expert",
                format!(
                    "no expert serving model_name {:?} is registered for the {origin_label} origin",
                    release.model_name
                ),
            )
        })?;
    let expert = &experts[ei];
    if expert.family() != release.family {
        return Err(Refusal::unsupported(
            "expert_family_mismatch",
            format!(
                "expert {} serves {}, release is {}",
                release.model_name,
                expert.family().as_str(),
                release.family.as_str()
            ),
        ));
    }
    let prefix = build_run_prefix(archive, cutoff_s, expert.policy())?;
    Ok((ei, prefix, settings.value()))
}

/// Run the expert, map its scientific refusals and validate the artifact.
pub fn run_expert(
    expert: &dyn Expert,
    loaded: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Refused<AnalysisArtifact> {
    let artifact =
        expert
            .forecast(loaded, prefix, settings)
            .map_err(|e| match Refusal::from_error(&e) {
                Some(r) => r.clone(),
                None => Refusal::failed(
                    "expert_error",
                    format!("{} failed: {e:#}", expert.model_name()),
                ),
            })?;
    validate_artifact(&artifact, &loaded.release, prefix)?;
    Ok(artifact)
}

pub fn validate_artifact(
    a: &AnalysisArtifact,
    rel: &ModelRelease,
    prefix: &PrefixBundle,
) -> Refused<()> {
    a.validate()
        .map_err(|e| Refusal::failed("artifact_validation", e))?;
    if !is_digest(&a.id) {
        return Err(Refusal::failed(
            "artifact_id",
            "artifact id is not a content hash",
        ));
    }
    if a.model_release_id != rel.id {
        return Err(Refusal::failed(
            "artifact_release",
            "artifact model_release_id does not match the release",
        ));
    }
    if a.prefix_hash != prefix.input_hash {
        return Err(Refusal::failed(
            "artifact_prefix",
            "artifact prefix_hash does not match the prefix input_hash",
        ));
    }
    if a.run_id.as_ref().is_some_and(|r| *r != prefix.run_id) {
        return Err(Refusal::failed(
            "artifact_run",
            "artifact run_id does not match the prefix",
        ));
    }
    Ok(())
}

/// Provenance stored next to every artifact (what the dashboard's drawer shows).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    pub analysis_id: String,
    pub run_id: String,
    pub cutoff_s: f64,
    pub model_release_id: String,
    pub model_name: String,
    pub release_state: Value,
    pub trained_on: Value,
    pub release_created_at: String,
    pub release_code_version: String,
    pub cohort_id: String,
    pub split_id: String,
    pub source_manifest_hash: String,
    pub prefix_hash: String,
    pub time_origin: String,
    pub admitted_observation_ids: Vec<String>,
    pub admitted_contour_ids: Vec<String>,
    pub known_condition_keys: Vec<String>,
    pub settings: Value,
    pub settings_key: String,
    pub cache_key: String,
    pub computed_at_unix_s: u64,
    pub data_origin: String,
    /// What produced the artifact (in-process expert `model_name`; older stores: the worker command).
    #[serde(default, alias = "worker")]
    pub engine: String,
}

impl AnalysisContext {
    /// The context of a planned forecast; `analysis_id` is filled once the artifact exists.
    pub fn new(
        release: &ModelRelease,
        prefix: &PrefixBundle,
        settings: &Value,
        origin_label: &str,
    ) -> Self {
        let settings_key = key_of(settings);
        AnalysisContext {
            analysis_id: String::new(),
            run_id: prefix.run_id.clone(),
            cutoff_s: prefix.cutoff_s,
            model_release_id: release.id.clone(),
            model_name: release.model_name.clone(),
            release_state: serde_json::to_value(release.state).unwrap(),
            trained_on: serde_json::to_value(release.trained_on).unwrap(),
            release_created_at: release.created_at.clone(),
            release_code_version: release.code_version.clone(),
            cohort_id: release.cohort_id.clone(),
            split_id: release.split_id.clone(),
            source_manifest_hash: release.source_manifest_hash.clone(),
            prefix_hash: prefix.input_hash.clone(),
            time_origin: prefix.time_origin.clone(),
            admitted_observation_ids: prefix.observations.iter().map(|x| x.id.clone()).collect(),
            admitted_contour_ids: prefix.contours.iter().map(|x| x.id.clone()).collect(),
            known_condition_keys: prefix.known_conditions.keys().cloned().collect(),
            settings: settings.clone(),
            cache_key: cache_key(&release.id, &prefix.input_hash, &settings_key),
            settings_key,
            computed_at_unix_s: 0,
            data_origin: origin_label.to_string(),
            engine: format!("in-process expert {}", release.model_name),
        }
    }
}

/// Key of a forecast request: the same (release, prefix, settings) is the same computation.
pub fn cache_key(release_id: &str, prefix_hash: &str, settings_key: &str) -> String {
    key_of(
        &serde_json::json!({"release": release_id, "prefix": prefix_hash, "settings": settings_key}),
    )
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredAnalysis {
    pub artifact: AnalysisArtifact,
    pub context: AnalysisContext,
}

pub fn write_json(path: &Path, v: &Value) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(v).unwrap() + "\n")
}

/// `<root>/analysis/<artifact-hex>/{artifact,context,prefix}.json`, write-once.
#[derive(Debug, Clone)]
pub struct AnalysisStore {
    root: PathBuf,
}

impl AnalysisStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        AnalysisStore { root: root.into() }
    }

    pub fn dir(&self, id: &str) -> Refused<PathBuf> {
        if !is_digest(id) {
            return Err(Refusal::invalid(
                "bad_analysis_id",
                "analysis ids are sha256 digests",
            ));
        }
        Ok(self.root.join("analysis").join(hex_part(id)))
    }

    pub fn save(&self, s: &StoredAnalysis, prefix: &Value) -> Refused<PathBuf> {
        let dir = self.dir(&s.artifact.id)?;
        if dir.join("artifact.json").exists() {
            return Ok(dir); // write-once: the same id is the same content
        }
        let tmp = dir.with_extension("tmp");
        let io = |e: std::io::Error| Refusal::failed("store_io", e.to_string());
        std::fs::create_dir_all(&tmp).map_err(io)?;
        write_json(
            &tmp.join("artifact.json"),
            &serde_json::to_value(&s.artifact).unwrap(),
        )
        .map_err(io)?;
        write_json(
            &tmp.join("context.json"),
            &serde_json::to_value(&s.context).unwrap(),
        )
        .map_err(io)?;
        write_json(&tmp.join("prefix.json"), prefix).map_err(io)?;
        if std::fs::rename(&tmp, &dir).is_err() {
            let _ = std::fs::remove_dir_all(&tmp); // another request stored it first
        }
        Ok(dir)
    }

    pub fn load(&self, id: &str) -> Refused<StoredAnalysis> {
        let dir = self.dir(id)?;
        let read = |f: &str| -> Refused<String> {
            std::fs::read_to_string(dir.join(f)).map_err(|_| {
                Refusal::not_found("analysis_not_found", format!("analysis {id} not found"))
            })
        };
        let artifact = serde_json::from_str(&read("artifact.json")?)
            .map_err(|e| Refusal::failed("store_contract", e.to_string()))?;
        let context = serde_json::from_str(&read("context.json")?)
            .map_err(|e| Refusal::failed("store_contract", e.to_string()))?;
        Ok(StoredAnalysis { artifact, context })
    }

    pub fn load_prefix(&self, id: &str) -> Refused<Value> {
        let p = self.dir(id)?.join("prefix.json");
        let t = std::fs::read_to_string(p)
            .map_err(|_| Refusal::not_found("analysis_not_found", "prefix not stored"))?;
        serde_json::from_str(&t).map_err(|e| Refusal::failed("store_contract", e.to_string()))
    }
}

/// Run a planned forecast and store the artifact: the whole CLI path after [`plan_forecast`].
pub fn execute_forecast(
    expert: &dyn Expert,
    loaded: &LoadedRelease,
    prefix: PrefixBundle,
    settings: Value,
    origin_label: &str,
    store: &AnalysisStore,
    computed_at_unix_s: u64,
) -> Refused<StoredAnalysis> {
    let mut context = AnalysisContext::new(&loaded.release, &prefix, &settings, origin_label);
    let artifact = run_expert(expert, loaded, &prefix, &settings)?;
    context.analysis_id = artifact.id.clone();
    context.computed_at_unix_s = computed_at_unix_s;
    let stored = StoredAnalysis { artifact, context };
    store.save(&stored, &serde_json::to_value(&prefix).unwrap())?;
    Ok(stored)
}

/// Development landmarks of a family (`configs/evaluation/landmarks.toml`, V3.1 §23.4): settings to
/// test, not validated intervals.
pub fn landmarks(family: crate::contracts::Family) -> anyhow::Result<Vec<f64>> {
    let path = configs_dir().join("evaluation").join("landmarks.toml");
    let text =
        std::fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
    let v: toml::Value = toml::from_str(&text)?;
    let row = v
        .get(family.as_str())
        .and_then(|t| t.get("cutoffs_s"))
        .and_then(|c| c.as_array());
    let Some(row) = row else {
        anyhow::bail!(
            "{} declares no [{}].cutoffs_s",
            path.display(),
            family.as_str()
        )
    };
    Ok(row
        .iter()
        .filter_map(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64)))
        .collect())
}

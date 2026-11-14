//! Small helpers shared by the ACME experts (config merge, artifact/release construction).
//! Mirrored in `ignis-bass`; candidates for `ignis-core` (see the lane report).

use std::collections::BTreeMap;

use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, EvidenceClass, Family, JsonMap, ModelRelease, PrefixBundle,
    QualificationState, Support, SupportGate, TrainedOn,
};
use ignis_core::hashing::content_hash;
use serde_json::Value;

pub fn deep_merge(a: &Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            let mut out = x.clone();
            for (k, v) in y {
                let merged = match out.get(k) {
                    Some(old) if old.is_object() && v.is_object() => deep_merge(old, v),
                    _ => v.clone(),
                };
                out.insert(k.clone(), merged);
            }
            Value::Object(out)
        }
        (_, Value::Null) => a.clone(),
        _ => b.clone(),
    }
}

pub fn load_toml(path: &std::path::Path) -> anyhow::Result<Option<Value>> {
    if !path.exists() {
        return Ok(None);
    }
    let t: toml::Value = toml::from_str(&std::fs::read_to_string(path)?)?;
    Ok(Some(serde_json::to_value(t)?))
}

pub fn obj(v: Value) -> JsonMap {
    match v {
        Value::Object(m) => m,
        Value::Null => JsonMap::new(),
        other => {
            let mut m = JsonMap::new();
            m.insert("value".into(), other);
            m
        }
    }
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

/// Content-addressed artifact: `fill` sets the arrays; id = hash of everything but id/created_at.
pub fn artifact(
    kind: ArtifactKind,
    release: &ModelRelease,
    prefix: &PrefixBundle,
    settings: &Value,
    diagnostics: Value,
    fill: impl FnOnce(&mut AnalysisArtifact),
) -> AnalysisArtifact {
    let mut a = AnalysisArtifact {
        id: String::new(),
        kind,
        model_release_id: release.id.clone(),
        run_id: Some(prefix.run_id.clone()),
        time_origin: Some(prefix.time_origin.clone()),
        prefix_hash: prefix.input_hash.clone(),
        settings_hash: ignis_core::hashing::key_of(settings),
        status: release.state,
        quantity: None,
        unit: None,
        times_s: vec![],
        quantile_levels: vec![],
        quantiles: vec![],
        mean: vec![],
        space_s_m: vec![],
        field_quantiles: vec![],
        field_mean: vec![],
        event_categories: vec![],
        event_cdf: vec![],
        support: Support {
            admitted: true,
            gate: SupportGate::Passed,
            reasons: vec![],
        },
        diagnostics: obj(diagnostics),
        evidence_class: EvidenceClass::ModelOutput,
        created_at: String::new(),
    };
    fill(&mut a);
    a.id = content_hash(&a, &["id", "created_at"]);
    a.created_at = now();
    a
}

/// A release skeleton; `save_release` fills id/tensors/created_at.
#[allow(clippy::too_many_arguments)]
pub fn release(
    family: Family,
    model_name: &str,
    is_baseline: bool,
    feature_order: Vec<String>,
    cohort_id: &str,
    split_id: &str,
    source_manifest_hash: &str,
    synthetic: bool,
) -> ModelRelease {
    ModelRelease {
        id: String::new(),
        family,
        model_name: model_name.into(),
        is_baseline,
        architecture: JsonMap::new(),
        config: JsonMap::new(),
        feature_order,
        units: BTreeMap::new(),
        normalization: BTreeMap::new(),
        priors: JsonMap::new(),
        observation_model: JsonMap::new(),
        numerics: JsonMap::new(),
        tensors: vec![],
        posterior: vec![],
        posterior_parameters: vec![],
        cohort_id: cohort_id.into(),
        split_id: split_id.into(),
        source_manifest_hash: source_manifest_hash.into(),
        code_version: String::new(),
        applicability: JsonMap::new(),
        evaluation: JsonMap::new(),
        state: QualificationState::FitOnly,
        parent_id: None,
        lineage: Default::default(),
        lineage_note: None,
        posterior_valid_for_parameters: false,
        trained_on: if synthetic {
            TrainedOn::Synthetic
        } else {
            TrainedOn::Nasa
        },
        resource_record: None,
        created_at: String::new(),
        // Conventionally fitted (legacy expert path): no WSP construction manifest.
        construction: None,
    }
}

/// numpy.percentile with linear interpolation.
pub fn percentile(v: &[f64], pct: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    if s.is_empty() {
        return 0.0;
    }
    let pos = pct / 100.0 * (s.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    s[lo] + (s[hi] - s[lo]) * (pos - lo as f64)
}

/// Measured SI values of the prefix conditions.
pub fn condition_values(prefix: &PrefixBundle) -> BTreeMap<String, f64> {
    prefix
        .known_conditions
        .iter()
        .filter_map(|(k, m)| m.value_si.map(|v| (k.clone(), v)))
        .collect()
}

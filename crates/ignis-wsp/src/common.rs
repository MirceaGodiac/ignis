//! Pieces shared by the family forwards: forecast settings, condition access, support results and
//! the artifact envelope. No parameter is read or changed here.

use serde_json::{json, Value};

use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, EvidenceClass, JsonMap, Measured, PrefixBundle,
    QualificationState, Quantity, Resolution, Support, SupportGate,
};
use ignis_core::expert::Refusal;
use ignis_core::hashing::content_hash;
use ignis_core::registry::LoadedRelease;

pub const DEFAULT_LEVELS: [f64; 5] = [0.05, 0.25, 0.5, 0.75, 0.95];

pub fn refuse(gate: SupportGate, reason: impl Into<String>) -> Support {
    Support {
        admitted: false,
        gate,
        reasons: vec![reason.into()],
    }
}

pub fn admitted() -> Support {
    Support {
        admitted: true,
        gate: SupportGate::Passed,
        reasons: vec![],
    }
}

pub fn gate_code(g: SupportGate) -> &'static str {
    match g {
        SupportGate::Semantic => "semantic",
        SupportGate::Measurement => "measurement",
        SupportGate::Condition => "condition",
        SupportGate::Numerical => "numerical",
        SupportGate::Horizon => "horizon",
        SupportGate::Passed => "passed",
    }
}

/// A support refusal as the error the legacy forecast path expects.
pub fn refusal_of(s: &Support) -> Refusal {
    Refusal::unsupported(gate_code(s.gate), s.reasons.join("; "))
}

/// A resolved numeric prospective condition in SI, or the reason it cannot be used. A missing,
/// unresolved or non-finite condition is a refusal, never a default.
pub fn condition(prefix: &PrefixBundle, key: &str) -> Result<f64, String> {
    let m: &Measured = prefix
        .known_conditions
        .get(key)
        .ok_or_else(|| format!("condition {key:?} missing from the prefix"))?;
    if m.resolution != Resolution::Resolved {
        return Err(format!("condition {key:?} is {:?}", m.resolution));
    }
    match m.value_si {
        Some(v) if v.is_finite() => Ok(v),
        _ => Err(format!("condition {key:?} has no finite SI value")),
    }
}

/// Quantile levels of a request (`null` or absent means the default set).
pub fn levels_of(settings: &Value) -> Vec<f64> {
    settings
        .get("quantile_levels")
        .and_then(|v| serde_json::from_value::<Vec<f64>>(v.clone()).ok())
        .filter(|l| !l.is_empty())
        .unwrap_or_else(|| DEFAULT_LEVELS.to_vec())
}

/// Forecast times of a request: explicit `times_s`, or `n_points` equally spaced times from the
/// cutoff to `min(cutoff + horizon_s, t_max)`.
pub fn times_of(settings: &Value, cutoff_s: f64, default_horizon_s: f64, t_max: f64) -> Vec<f64> {
    if let Some(mut t) = settings
        .get("times_s")
        .and_then(|v| serde_json::from_value::<Vec<f64>>(v.clone()).ok())
    {
        t.sort_by(f64::total_cmp);
        return t;
    }
    let h = settings
        .get("horizon_s")
        .and_then(Value::as_f64)
        .unwrap_or(default_horizon_s);
    let n = settings
        .get("n_points")
        .and_then(Value::as_u64)
        .unwrap_or(41)
        .max(2) as usize;
    let hi = (cutoff_s + h).min(t_max);
    (0..n)
        .map(|i| cutoff_s + (hi - cutoff_s) * i as f64 / (n - 1) as f64)
        .collect()
}

/// What a forward hands back for the artifact body.
#[derive(Debug, Clone, Default)]
pub struct Body {
    pub kind: Option<ArtifactKind>,
    pub quantity: Option<Quantity>,
    pub times_s: Vec<f64>,
    pub quantile_levels: Vec<f64>,
    pub quantiles: Vec<Vec<f64>>,
    pub mean: Vec<f64>,
    pub event_categories: Vec<String>,
    pub event_cdf: Vec<Vec<f64>>,
    pub diagnostics: Value,
}

/// The content-addressed artifact of one forecast. A refused forecast (`support.admitted = false`)
/// keeps the target quantity and unit and carries no numbers, with status `unsupported`.
pub fn artifact(
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
    support: Support,
    body: Body,
) -> AnalysisArtifact {
    let status = if support.admitted {
        release.release.state
    } else {
        QualificationState::Unsupported
    };
    let diagnostics: JsonMap = match body.diagnostics {
        Value::Object(m) => m,
        Value::Null => JsonMap::new(),
        other => json!({ "value": other }).as_object().cloned().unwrap(),
    };
    let mut a = AnalysisArtifact {
        id: String::new(),
        kind: body.kind.unwrap_or(ArtifactKind::PrefixForecast),
        model_release_id: release.release.id.clone(),
        run_id: Some(prefix.run_id.clone()),
        time_origin: Some(prefix.time_origin.clone()),
        prefix_hash: prefix.input_hash.clone(),
        settings_hash: content_hash(settings, &[]),
        status,
        quantity: body.quantity,
        unit: body.quantity.map(|q| q.unit_si().to_string()),
        times_s: body.times_s,
        quantile_levels: if body.quantiles.is_empty() {
            vec![]
        } else {
            body.quantile_levels
        },
        quantiles: body.quantiles,
        mean: body.mean,
        space_s_m: vec![],
        field_quantiles: vec![],
        field_mean: vec![],
        event_categories: body.event_categories,
        event_cdf: body.event_cdf,
        support,
        diagnostics,
        evidence_class: EvidenceClass::ModelOutput,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
    };
    a.id = content_hash(&a, &["id", "created_at"]);
    a
}

/// Weighted quantiles of point masses with linear interpolation between mass centres.
/// `atoms` are `(value, weight)` with positive weights; they are sorted in place.
pub fn atom_quantiles(atoms: &mut [(f64, f64)], levels: &[f64], out: &mut [f64]) {
    debug_assert_eq!(levels.len(), out.len());
    atoms.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
    let total: f64 = atoms.iter().map(|a| a.1).sum();
    let mut li = 0;
    let mut cum = 0.0;
    let mut prev: Option<(f64, f64)> = None; // (centre, value)
    for &(v, w) in atoms.iter() {
        let centre = (cum + 0.5 * w) / total;
        while li < levels.len() && levels[li] <= centre {
            out[li] = match prev {
                Some((pc, pv)) if centre > pc => pv + (v - pv) * (levels[li] - pc) / (centre - pc),
                _ => v,
            };
            li += 1;
        }
        cum += w;
        prev = Some((centre, v));
    }
    let last = prev.map_or(f64::NAN, |p| p.1);
    for o in out.iter_mut().skip(li) {
        *o = last;
    }
}

//! FLEX admission gates in the fixed order of V3.1 §23.3:
//! 1 semantic, 2 measurement, 3 condition support, 4 numerical, 5 horizon.
//!
//! The condition gate is the declared initial heuristic: at least `k` compatible training
//! neighbours from at least `min_groups` distinct groups inside a radius fixed on the training
//! partition as the `radius_percentile` of leave-one-group-out nearest-neighbour distances. It is an
//! operational refusal mechanism — not an OOD probability and not a coverage guarantee.

use std::collections::BTreeSet;

use serde_json::{json, Value};

use ignis_core::contracts::{
    Family, ModelRelease, PrefixBundle, QualificationState, Support, SupportGate, TrainedOn,
};

use crate::data::PrefixData;

fn percentile(v: &[f64], q: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    let pos = q / 100.0 * (s.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    s[lo] + (s[hi] - s[lo]) * (pos - lo as f64)
}

fn dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f64>()
        .sqrt()
}

/// X: normalised condition coordinates of the train runs (finite).
pub fn fit_condition_support(
    x: &[Vec<f64>],
    groups: &[String],
    k: usize,
    min_groups: usize,
    radius_percentile: f64,
) -> Value {
    let mut d_logo = Vec::new();
    for i in 0..x.len() {
        let d = (0..x.len())
            .filter(|&j| groups[j] != groups[i])
            .map(|j| dist(&x[i], &x[j]))
            .fold(f64::INFINITY, f64::min);
        if d.is_finite() {
            d_logo.push(d);
        }
    }
    let radius = if d_logo.is_empty() {
        0.0
    } else {
        percentile(&d_logo, radius_percentile)
    };
    let ng: BTreeSet<&String> = groups.iter().collect();
    json!({"rule": "knn_groups_v1", "k": k, "min_groups": min_groups, "radius_percentile": radius_percentile,
           "radius": radius, "points": x, "groups": groups, "n_groups": ng.len(),
           "meaning": "operational refusal heuristic, not an OOD probability"})
}

pub fn condition_gate(policy: &Value, x: &[f64]) -> (bool, Vec<String>) {
    if x.iter().any(|v| !v.is_finite()) {
        return (false, vec!["condition vector not finite".into()]);
    }
    let pts: Vec<Vec<f64>> = serde_json::from_value(policy["points"].clone()).unwrap_or_default();
    let groups: Vec<String> = serde_json::from_value(policy["groups"].clone()).unwrap_or_default();
    let radius = policy["radius"].as_f64().unwrap_or(0.0);
    let k = policy["k"].as_u64().unwrap_or(3) as usize;
    let mg = policy["min_groups"].as_u64().unwrap_or(2) as usize;
    let inside: Vec<usize> = (0..pts.len())
        .filter(|&i| dist(&pts[i], x) <= radius)
        .collect();
    let gs: BTreeSet<&String> = inside.iter().map(|&i| &groups[i]).collect();
    let ok = inside.len() >= k && gs.len() >= mg;
    let reasons = if ok {
        vec![]
    } else {
        vec![format!(
            "{} training neighbours from {} groups within radius {:.3} (need >= {k} from >= {mg})",
            inside.len(),
            gs.len(),
            radius
        )]
    };
    (ok, reasons)
}

fn refuse(gate: SupportGate, reasons: Vec<String>) -> Support {
    Support {
        admitted: false,
        gate,
        reasons,
    }
}

pub fn gate_prefix(
    release: &ModelRelease,
    prefix: &PrefixBundle,
    pd: &PrefixData,
    x_cond: &[f64],
    settings: &Value,
) -> Support {
    let app = &release.applicability;
    // 1. semantic
    let mut r = Vec::new();
    if prefix.family != Family::Flex || release.family != Family::Flex {
        r.push(format!("family {:?} is not FLEX", prefix.family));
    }
    if matches!(
        release.state,
        QualificationState::Failed | QualificationState::Unsupported
    ) {
        r.push(format!("release state {:?}", release.state));
    }
    if prefix.phase_established_at_s.is_none() {
        r.push(
            "modelled phase not established at the cutoff (phase_established_at_s missing)".into(),
        );
    }
    if let Some(tor) = app.get("time_origin").and_then(|v| v.as_str()) {
        if prefix.time_origin != tor {
            r.push(format!(
                "time origin {:?} differs from release {tor:?}",
                prefix.time_origin
            ));
        }
    }
    if let (Some(f), Some(af)) = (
        prefix.known_conditions.get("fuel"),
        app.get("fuel").and_then(|v| v.as_str()),
    ) {
        let raw = match &f.raw_value {
            Some(ignis_core::contracts::RawValue::Text(s)) => s.clone(),
            Some(ignis_core::contracts::RawValue::Number(x)) => x.to_string(),
            None => String::new(),
        };
        if raw.to_lowercase() != af.to_lowercase() {
            r.push(format!("fuel {raw:?} outside release population {af:?}"));
        }
    }
    if release.trained_on == TrainedOn::Synthetic && !pd.synthetic {
        r.push(
            "release trained on synthetic data cannot forecast non-synthetic observations".into(),
        );
    }
    if !r.is_empty() {
        return refuse(SupportGate::Semantic, r);
    }
    // 2. measurement
    let mut r = pd.problems.clone();
    let need = settings
        .get("min_prefix_observations")
        .and_then(|v| v.as_u64())
        .unwrap_or(2) as usize;
    if pd.n_valid() < need {
        r.push(format!(
            "{} valid liquid-diameter observations in prefix (< {need})",
            pd.n_valid()
        ));
    }
    if !r.is_empty() {
        return refuse(SupportGate::Measurement, r);
    }
    // 3. condition support
    if let Some(pol) = app.get("condition_support").filter(|v| v.is_object()) {
        let (ok, r) = condition_gate(pol, x_cond);
        if !ok {
            return refuse(SupportGate::Condition, r);
        }
    }
    // 4. numerical (release-level; particle ESS is checked after inference)
    let inf = release
        .numerics
        .get("inference")
        .cloned()
        .unwrap_or(Value::Null);
    if inf["method"] == "nuts"
        && inf["pass"] != Value::Bool(true)
        && settings.get("allow_unconverged") != Some(&Value::Bool(true))
    {
        return refuse(
            SupportGate::Numerical,
            vec!["release NUTS diagnostics did not pass (R-hat/ESS/divergences)".into()],
        );
    }
    // 5. horizon
    let t_valid = release
        .numerics
        .get("t_valid_s")
        .and_then(|v| v.as_f64())
        .unwrap_or(f64::INFINITY);
    if pd.tau() >= t_valid {
        return refuse(
            SupportGate::Horizon,
            vec![format!("cutoff {} s >= T_valid {t_valid} s", pd.tau())],
        );
    }
    let d_min = release
        .numerics
        .get("d_min_m")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let last = pd.last_valid();
    if last.is_finite() && last <= d_min {
        return refuse(
            SupportGate::Horizon,
            vec![format!(
                "last diameter {last:.3e} m <= support floor {d_min} m"
            )],
        );
    }
    Support {
        admitted: true,
        gate: SupportGate::Passed,
        reasons: vec![],
    }
}

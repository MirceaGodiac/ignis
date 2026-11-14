//! Role-aware admissible pre-event features (V3.1 §8.2) and their train-only normalisation.
//!
//! Eight nominal features per configuration, in a frozen order (`feature_order` of the release):
//! log(p/p_ref), log(mdot_injected/mdot_ref) (injected stream is FUEL in normal and OXIDISER in
//! inverse), oxygen fraction in its explicit role, fuel fraction in its explicit role, normalised
//! T_ad, Z_st, two fuel indicators against a declared reference fuel. Missing critical features
//! restrict the cohort; they are never filled with typical values.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Never admissible as ACME predictors (§8.2).
pub const ACME_FORBIDDEN: &[&str] = &[
    "hc_burnrate",
    "hc_burn_rate",
    "tfp",
    "peak_burner_temperature",
    "after_test_composition",
    "final_duration",
    "duration_s",
    "extinction_time",
    "ignition_success",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureSpec {
    pub name: String,
    pub source: String,
    pub transform: String,
    #[serde(default, rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<f64>,
}

pub fn transform(f: &FeatureSpec, x: f64) -> anyhow::Result<f64> {
    Ok(match f.transform.as_str() {
        "log_ratio" => {
            if !(x > 0.0) {
                anyhow::bail!("{}: log of non-positive value", f.source);
            }
            (x / f.reference.unwrap_or(1.0)).ln()
        }
        "ratio" => x / f.reference.unwrap_or(1.0),
        "identity" | "indicator" => x,
        t => anyhow::bail!("unknown transform {t}"),
    })
}

pub fn missing(features: &[FeatureSpec], values: &BTreeMap<String, f64>) -> Vec<String> {
    features
        .iter()
        .filter(|f| !values.get(&f.source).is_some_and(|v| v.is_finite()))
        .map(|f| f.source.clone())
        .collect()
}

pub fn raw_row(
    features: &[FeatureSpec],
    values: &BTreeMap<String, f64>,
) -> anyhow::Result<Vec<f64>> {
    features
        .iter()
        .map(|f| transform(f, values[&f.source]))
        .collect()
}

pub type Norm = BTreeMap<String, BTreeMap<String, f64>>;

/// Centre/scale from TRAIN rows only; indicators keep their declared reference (0) and unit scale.
pub fn fit_normalization(features: &[FeatureSpec], x: &[Vec<f64>]) -> Norm {
    let n = x.len() as f64;
    features
        .iter()
        .enumerate()
        .map(|(j, f)| {
            let (center, scale) = if f.transform == "indicator" {
                (0.0, 1.0)
            } else {
                let m = x.iter().map(|r| r[j]).sum::<f64>() / n;
                let sd = (x.iter().map(|r| (r[j] - m).powi(2)).sum::<f64>() / n).sqrt();
                (m, if sd > 1e-12 { sd } else { 1.0 })
            };
            let mut e =
                BTreeMap::from([("center".to_string(), center), ("scale".to_string(), scale)]);
            if let Some(r) = f.reference {
                e.insert("ref".into(), r);
            }
            (f.name.clone(), e)
        })
        .collect()
}

pub fn apply_normalization(features: &[FeatureSpec], norm: &Norm, x: &[f64]) -> Vec<f64> {
    features
        .iter()
        .enumerate()
        .map(|(j, f)| (x[j] - norm[&f.name]["center"]) / norm[&f.name]["scale"])
        .collect()
}

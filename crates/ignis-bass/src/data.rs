//! BASS views → run arrays, condition vectors and geometric support checks (V3.1 §9.1, §9.3).
//!
//! Conditions for FiLM (frozen order): normalised O₂, log pressure, flow-speed magnitude, flow-direction
//! sign, log width, log length, log thickness, time since the last KNOWN control change. Scales are
//! fixed in the manifest; centring/scaling is fitted on train only. Missing conditions are a refusal,
//! never filled. Control changes come from `BassSequence.control_changes_s` (training) and from the
//! prefix's `known_controls` (serving; protocol-fixed future stages only, see `ignis_core::prefix`).

use std::collections::BTreeMap;

use ignis_core::views::BassSequence;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::dynamics::Conditions;
use crate::observation::{self as obs, FramePre};

pub const SINGLE: &str = "single_valued";
pub const TOPOLOGY_REFUSAL: [&str; 2] = ["multivalued", "detached"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureSpec {
    pub name: String,
    pub source: String,
    pub transform: String,
    #[serde(default, rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<f64>,
}

/// Raw static condition values (time-since entries are 0 and filled per time step).
pub fn static_raw(
    features: &[FeatureSpec],
    values: &BTreeMap<String, f64>,
) -> Result<Vec<f64>, String> {
    let mut out = Vec::with_capacity(features.len());
    for f in features {
        if f.transform == "time_since" {
            out.push(0.0);
            continue;
        }
        let x = *values
            .get(&f.source)
            .filter(|x| x.is_finite())
            .ok_or_else(|| f.source.clone())?;
        let r = f.reference.unwrap_or(1.0);
        out.push(match f.transform.as_str() {
            "log_ratio" => (x / r).ln(),
            "ratio" => x / r,
            "abs_ratio" => x.abs() / r,
            "sign" => {
                if x > 0.0 {
                    1.0
                } else if x < 0.0 {
                    -1.0
                } else {
                    0.0
                }
            }
            "identity" => x,
            t => return Err(format!("unknown transform {t}")),
        });
    }
    Ok(out)
}

/// Control-change time in effect at t: the latest change ≤ t; before the first change the first
/// change (time-since = 0 until it); without changes the first frame time.
pub fn change_at(changes: &[f64], t0: f64, t: f64) -> f64 {
    if changes.is_empty() {
        return t0;
    }
    changes
        .iter()
        .cloned()
        .filter(|c| *c <= t)
        .fold(None, |a: Option<f64>, b| Some(a.map_or(b, |x| x.max(b))))
        .unwrap_or(changes[0])
}

/// Normalisation arrays in feature order.
#[derive(Debug, Clone)]
pub struct NormArrays {
    pub center: Vec<f64>,
    pub scale: Vec<f64>,
    pub is_time: Vec<bool>,
    pub reference: Vec<f64>,
}

pub type Norm = BTreeMap<String, BTreeMap<String, f64>>;

pub fn norm_arrays(features: &[FeatureSpec], norm: &Norm) -> NormArrays {
    NormArrays {
        center: features.iter().map(|f| norm[&f.name]["center"]).collect(),
        scale: features.iter().map(|f| norm[&f.name]["scale"]).collect(),
        is_time: features
            .iter()
            .map(|f| f.transform == "time_since")
            .collect(),
        reference: features
            .iter()
            .map(|f| {
                if f.transform == "time_since" {
                    f.reference.unwrap_or(1.0)
                } else {
                    1.0
                }
            })
            .collect(),
    }
}

/// c(t) of one run.
pub struct RunConditions<'a> {
    pub c_static: &'a [f64],
    pub changes: &'a [f64],
    pub t0: f64,
    pub norm: &'a NormArrays,
}

impl Conditions for RunConditions<'_> {
    fn at(&self, t: f64, out: &mut [f64]) {
        let tc = change_at(self.changes, self.t0, t);
        for j in 0..self.c_static.len() {
            let raw = if self.norm.is_time[j] {
                ((t - tc).max(0.0) / self.norm.reference[j]).ln_1p()
            } else {
                self.c_static[j]
            };
            out[j] = (raw - self.norm.center[j]) / self.norm.scale[j];
        }
    }
}

/// One prepared run (trimmed to its first usable frame).
#[derive(Debug, Clone)]
pub struct RunData {
    pub run_id: String,
    pub group: String,
    pub t: Vec<f64>,
    pub xi: Vec<f64>,
    /// (T × S), zero where invalid
    pub f: Vec<f64>,
    pub valid: Vec<bool>,
    pub use_: Vec<bool>,
    pub u: Vec<f64>,
    pub l: f64,
    pub w: f64,
    pub c_static: Vec<f64>,
    pub changes: Vec<f64>,
    /// fallback origin of the time-since feature (first frame of the untrimmed sequence)
    pub t0: f64,
    pub pre: Vec<FramePre>,
}

impl RunData {
    pub fn n_t(&self) -> usize {
        self.t.len()
    }
    pub fn n_s(&self) -> usize {
        self.xi.len()
    }
    pub fn cond<'a>(&'a self, norm: &'a NormArrays) -> RunConditions<'a> {
        RunConditions {
            c_static: &self.c_static,
            changes: &self.changes,
            t0: self.t0,
            norm,
        }
    }
}

pub fn merged_values(seq: &BassSequence) -> BTreeMap<String, f64> {
    let mut v = seq.geometry.clone();
    v.extend(seq.conditions.iter().map(|(k, x)| (k.clone(), *x)));
    v
}

#[allow(clippy::too_many_arguments)]
pub fn build_run(
    run_id: &str,
    group: &str,
    t: &[f64],
    s: &[f64],
    f_raw: &[f64],
    valid_raw: &[bool],
    status: &[String],
    u_rel: &[f64],
    c_static: Vec<f64>,
    l: f64,
    changes: Vec<f64>,
    cfg: &Value,
    k: usize,
) -> Result<RunData, String> {
    let ns = s.len();
    if ns < 2 || s.windows(2).any(|w| w[1] <= w[0]) {
        return Err("transverse grid not increasing".into());
    }
    let nt = t.len();
    let min_valid = cfg["min_valid"].as_u64().unwrap_or(12) as usize;
    let valid: Vec<bool> = (0..nt * ns)
        .map(|i| valid_raw[i] && f_raw[i].is_finite())
        .collect();
    let use_: Vec<bool> = (0..nt)
        .map(|i| {
            status[i] == SINGLE
                && valid[i * ns..(i + 1) * ns].iter().filter(|b| **b).count() >= min_valid
        })
        .collect();
    if use_.iter().filter(|b| **b).count() < 2 {
        return Err("fewer than two usable single-valued frames".into());
    }
    let first = use_.iter().position(|b| *b).unwrap();
    let w = s[ns - 1] - s[0];
    let xi: Vec<f64> = s.iter().map(|x| (x - s[0]) / w).collect();
    let f: Vec<f64> = (first * ns..nt * ns)
        .map(|i| if valid[i] { f_raw[i] } else { 0.0 })
        .collect();
    let u: Vec<f64> = if u_rel.len() == nt {
        u_rel[first..].to_vec()
    } else {
        vec![0.0; nt - first]
    };
    let ell = cfg["ell_s"].as_f64().unwrap_or(0.1);
    let (valid, use_) = (valid[first * ns..].to_vec(), use_[first..].to_vec());
    let pre = obs::precompute(&xi, &f, &valid, &use_, &u, k, ell);
    Ok(RunData {
        run_id: run_id.into(),
        group: group.into(),
        t: t[first..].to_vec(),
        xi,
        f,
        valid,
        use_,
        u,
        l,
        w,
        c_static,
        changes,
        t0: t[0],
        pre,
    })
}

/// Validated, trimmed runs and the excluded ones (no imputation).
pub fn prepare(
    views: &[&BassSequence],
    features: &[FeatureSpec],
    cfg: &Value,
    k: usize,
) -> (Vec<RunData>, Vec<Value>) {
    let mut keep = vec![];
    let mut excluded = vec![];
    let lk = cfg["length_key"].as_str().unwrap_or("length");
    for v in views {
        let vals = merged_values(v);
        let cs = match static_raw(features, &vals) {
            Ok(c) => c,
            Err(e) => {
                excluded.push(json!({"run_id": v.run_id, "reason": format!("missing condition {e:?} (no imputation)")}));
                continue;
            }
        };
        let Some(l) = vals.get(lk).cloned().filter(|l| *l > 0.0) else {
            excluded.push(json!({"run_id": v.run_id, "reason": "specimen length unknown"}));
            continue;
        };
        let mut changes = v.control_changes_s.clone();
        changes.sort_by(|a, b| a.total_cmp(b));
        match build_run(
            &v.run_id,
            &v.group_id,
            &v.t_s,
            &v.s_m,
            &v.f_m,
            &v.valid,
            &v.status,
            &v.scale_uncertainty_rel,
            cs,
            l,
            changes,
            cfg,
            k,
        ) {
            Ok(r) => keep.push(r),
            Err(e) => excluded.push(json!({"run_id": v.run_id, "reason": e})),
        }
    }
    (keep, excluded)
}

pub fn fit_normalization(features: &[FeatureSpec], runs: &[RunData]) -> Norm {
    features
        .iter()
        .enumerate()
        .map(|(j, f)| {
            let vals: Vec<f64> = if f.transform == "time_since" {
                runs.iter()
                    .flat_map(|r| {
                        r.t.iter().map(move |&t| {
                            ((t - change_at(&r.changes, r.t0, t)).max(0.0)
                                / f.reference.unwrap_or(1.0))
                            .ln_1p()
                        })
                    })
                    .collect()
            } else {
                runs.iter().map(|r| r.c_static[j]).collect()
            };
            let n = vals.len() as f64;
            let m = vals.iter().sum::<f64>() / n;
            let sd = (vals.iter().map(|v| (v - m).powi(2)).sum::<f64>() / n).sqrt();
            let mut e = BTreeMap::from([
                ("center".to_string(), m),
                ("scale".to_string(), if sd > 1e-12 { sd } else { 1.0 }),
                (
                    "min".to_string(),
                    vals.iter().cloned().fold(f64::INFINITY, f64::min),
                ),
                (
                    "max".to_string(),
                    vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                ),
            ]);
            if let Some(r) = f.reference {
                e.insert("ref".into(), r);
            }
            (f.name.clone(), e)
        })
        .collect()
}

/// Topology/status from the views → refusal. Returns (ok, code, reason).
pub fn geometric_status_check(
    statuses: &[String],
    valid_fraction: f64,
    min_fraction: f64,
) -> (bool, &'static str, String) {
    if statuses.is_empty() {
        return (
            false,
            "no_contour",
            "no admitted contour at the cutoff".into(),
        );
    }
    let mut bad: Vec<&str> = statuses
        .iter()
        .map(|s| s.as_str())
        .filter(|s| TOPOLOGY_REFUSAL.contains(s))
        .collect();
    bad.sort();
    bad.dedup();
    if !bad.is_empty() {
        return (
            false,
            "topology",
            format!("front topology {bad:?} outside the single-valued model class"),
        );
    }
    if statuses.last().unwrap() != SINGLE {
        return (
            false,
            "latest_contour",
            format!(
                "latest admitted contour is {}; no interpolation through it",
                statuses.last().unwrap()
            ),
        );
    }
    if valid_fraction < min_fraction {
        return (
            false,
            "resolved_fraction",
            format!("resolved-sample fraction {valid_fraction:.2} < {min_fraction}"),
        );
    }
    (true, "passed", "single-valued, resolved".into())
}

pub fn condition_support(
    cs: &[f64],
    features: &[FeatureSpec],
    norm: &Norm,
    margin: f64,
) -> (bool, String) {
    let bad: Vec<&str> = features
        .iter()
        .enumerate()
        .filter(|(_, f)| f.transform != "time_since")
        .filter(|(j, f)| {
            let n = &norm[&f.name];
            cs[*j] < n["min"] - margin * n["scale"] || cs[*j] > n["max"] + margin * n["scale"]
        })
        .map(|(_, f)| f.name.as_str())
        .collect();
    if bad.is_empty() {
        (true, "inside train condition ranges".into())
    } else {
        (
            false,
            format!("outside train range (±{margin} sd): {bad:?}"),
        )
    }
}

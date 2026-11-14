//! Packing FLEX views and prefixes into per-run arrays (masks, never zero-filled values).
//!
//! Declared observation-model choices (recorded in `ModelRelease.observation_model`):
//! * observations with t < 0 (before adjudicated phase entry) or t > T_valid are masked (§7.1);
//! * exact duplicate timestamps keep the first valid value and mask the rest (counted, §7.4);
//! * calibration nuisances s_g = 1, delta_g = 0, Delta t_g = 0 (published calibration assumed
//!   applied upstream by the data lane; sensitivity scenarios are a separate release).

use std::collections::{BTreeMap, HashSet};

use ignis_core::contracts::{EvidenceClass, PrefixBundle, Quantity, Resolution};
use ignis_core::views::{FlexTrajectory, FLEX_CAUSES};

use crate::dynamics::RunData;
use crate::structure::{known_features, D_REF_M};

#[derive(Debug, Clone, Default)]
pub struct Packed {
    pub run_ids: Vec<String>,
    pub group_ids: Vec<String>,
    pub runs: Vec<RunData>,
    /// First valid diameter (proxy / init only).
    pub y_first: Vec<f64>,
    pub duplicate_timestamps_masked: usize,
    pub runs_without_event_likelihood: Vec<String>,
}

impl Packed {
    pub fn n(&self) -> usize {
        self.runs.len()
    }
    pub fn logd0_proxy(&self) -> Vec<f64> {
        self.y_first
            .iter()
            .map(|y| {
                if y.is_finite() {
                    (y / D_REF_M).ln()
                } else {
                    f64::NAN
                }
            })
            .collect()
    }
    pub fn notes(&self) -> serde_json::Value {
        serde_json::json!({"duplicate_timestamps_masked": self.duplicate_timestamps_masked,
                           "runs_without_event_likelihood": self.runs_without_event_likelihood})
    }
}

fn clean_series(
    t: &[f64],
    y: &[f64],
    valid: &[bool],
    t_valid: f64,
) -> (Vec<f64>, Vec<f64>, Vec<bool>, usize) {
    let n = t.len();
    let m0: Vec<bool> = (0..n)
        .map(|i| {
            valid.get(i).copied().unwrap_or(false)
                && t[i].is_finite()
                && y[i].is_finite()
                && t[i] >= 0.0
                && t[i] <= t_valid
                && y[i] > 0.0
        })
        .collect();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| t[a].total_cmp(&t[b]));
    let (ts, ys): (Vec<f64>, Vec<f64>) = order.iter().map(|&i| (t[i], y[i])).unzip();
    let mut ms: Vec<bool> = order.iter().map(|&i| m0[i]).collect();
    let mut seen = HashSet::new();
    let mut dup = 0;
    for i in 0..n {
        if ms[i] {
            if !seen.insert(ts[i].to_bits()) {
                ms[i] = false;
                dup += 1;
            }
        }
    }
    (ts, ys, ms, dup)
}

/// (code, L, R, cause mask) of a view's endpoint.
pub fn endpoint_code(v: &FlexTrajectory) -> (u8, f64, f64, [f64; 2]) {
    let cm = [0, 1].map(|k| {
        if v.allowed_causes.iter().any(|c| c == FLEX_CAUSES[k]) {
            1.0
        } else {
            0.0
        }
    });
    let any = cm[0] + cm[1] > 0.0;
    match v.censoring.as_str() {
        "event" if v.t_first_terminal_s.is_some() && any => {
            let t = v.t_first_terminal_s.unwrap();
            (1, t, t, cm)
        }
        "interval" if v.t_last_active_s.is_some() && v.t_first_terminal_s.is_some() && any => (
            2,
            v.t_last_active_s.unwrap(),
            v.t_first_terminal_s.unwrap(),
            cm,
        ),
        "right" => {
            let c = v.t_last_active_s.unwrap_or_else(|| {
                (0..v.t_s.len())
                    .filter(|&i| v.valid[i] && v.d_m[i].is_finite())
                    .map(|i| v.t_s[i])
                    .fold(f64::NAN, f64::max)
            });
            let c = if c.is_finite() { c } else { 0.0 };
            (3, c, c, [0.0; 2])
        }
        _ => (0, 0.0, 0.0, [0.0; 2]),
    }
}

pub fn pack_views(
    views: &[&FlexTrajectory],
    keys: &BTreeMap<String, String>,
    t_valid: f64,
) -> Packed {
    let mut p = Packed::default();
    for v in views {
        let (t, y, mask, dup) = clean_series(&v.t_s, &v.d_m, &v.valid, t_valid);
        let (code, l, r, cm) = endpoint_code(v);
        if code == 0 {
            p.runs_without_event_likelihood.push(v.run_id.clone());
        }
        p.y_first
            .push((0..t.len()).find(|&i| mask[i]).map_or(f64::NAN, |i| y[i]));
        p.duplicate_timestamps_masked += dup;
        p.run_ids.push(v.run_id.clone());
        p.group_ids.push(v.group_id.clone());
        p.runs.push(RunData {
            t,
            y,
            mask,
            ck: known_features(&v.conditions, keys),
            code,
            l,
            r,
            cause_mask: cm,
            tau: 0.0,
            use_tau: false,
        });
    }
    p
}

/// Events beyond the model horizon are administratively right-censored at T_valid (declared).
pub fn admin_censor(p: &mut Packed, t_valid: f64) {
    for r in p.runs.iter_mut() {
        if r.code != 0 && r.l >= t_valid {
            r.code = 3;
            r.l = t_valid;
            r.r = t_valid;
            r.cause_mask = [0.0; 2];
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrefixData {
    pub run: RunData,
    pub y_first: f64,
    pub obs_ids: Vec<String>,
    pub problems: Vec<String>,
    pub synthetic: bool,
}

impl PrefixData {
    pub fn tau(&self) -> f64 {
        self.run.tau
    }
    pub fn n_valid(&self) -> usize {
        self.run.mask.iter().filter(|b| **b).count()
    }
    pub fn last_valid(&self) -> f64 {
        (0..self.run.t.len())
            .rev()
            .find(|&i| self.run.mask[i])
            .map_or(f64::NAN, |i| self.run.y[i])
    }
}

/// Liquid-diameter observations admitted in the prefix + prospective conditions. The run data is
/// the prefix target (use_tau = true, no event term).
pub fn prefix_arrays(
    prefix: &PrefixBundle,
    keys: &BTreeMap<String, String>,
    t_valid: f64,
) -> PrefixData {
    let mut problems = Vec::new();
    let mut conds = BTreeMap::new();
    for (k, m) in &prefix.known_conditions {
        if let Some(v) = m.value_si {
            conds.insert(k.clone(), v);
        }
    }
    for (role, key) in keys {
        match prefix.known_conditions.get(key) {
            None => problems.push(format!("condition {key:?} ({role}) missing from prefix")),
            Some(m) if m.value_si.is_none() => problems.push(format!(
                "condition {key:?} has no SI value ({:?})",
                m.resolution
            )),
            Some(m) if m.resolution != Resolution::Resolved => problems.push(format!(
                "condition {key:?} not resolved ({:?})",
                m.resolution
            )),
            _ => {}
        }
    }
    let obs: Vec<_> = prefix
        .observations
        .iter()
        .filter(|o| o.quantity == Quantity::LiquidDiameter)
        .collect();
    let synthetic = !obs.is_empty()
        && obs
            .iter()
            .all(|o| o.evidence_class == EvidenceClass::Synthetic);
    let (mut ts, mut ys, mut ms, mut ids) = (vec![], vec![], vec![], vec![]);
    let mut by_time: BTreeMap<u64, f64> = BTreeMap::new();
    for o in &obs {
        let val = o.value_si.filter(|v| v.is_finite() && *v > 0.0);
        let mut ok = val.is_some() && o.quality_flags.is_empty();
        if ok {
            if let Some(prev) = by_time.get(&o.time_s.to_bits()) {
                if Some(*prev) != val {
                    problems.push(format!("conflicting duplicate timestamp t={} s", o.time_s));
                }
            }
        }
        if ok && (o.time_s < 0.0 || o.time_s > t_valid) {
            ok = false;
        }
        if ok {
            by_time.insert(o.time_s.to_bits(), val.unwrap());
        }
        let repeated = ts.iter().any(|t: &f64| *t == o.time_s);
        ts.push(o.time_s);
        ys.push(o.value_si.filter(|v| !v.is_nan()).unwrap_or(0.0));
        ms.push(ok && !repeated);
        ids.push(o.id.clone());
    }
    if ts.is_empty() {
        ts.push(0.0);
        ys.push(0.0);
        ms.push(false);
    }
    let y_first = (0..ts.len()).find(|&i| ms[i]).map_or(f64::NAN, |i| ys[i]);
    let run = RunData {
        t: ts,
        y: ys,
        mask: ms,
        ck: known_features(&conds, keys),
        code: 0,
        l: 0.0,
        r: 0.0,
        cause_mask: [0.0; 2],
        tau: prefix.cutoff_s,
        use_tau: true,
    };
    PrefixData {
        run,
        y_first,
        obs_ids: ids,
        problems,
        synthetic,
    }
}

//! Proper scores and error measures (V3.1 §13.3–13.5). Every function documents its estimator.
//!
//! Conventions: SI units throughout; NaN marks "not computable" and is always accompanied by a reason
//! at the caller. Lower is better except for log predictive density.

use serde_json::{json, Map, Value};

use crate::util::{interp, linspace, mean, pairwise_sum, quantile_sorted};

// ------------------------------------------------------------------------------- scalar forecasts

/// CRPS from (optionally importance-weighted) samples.
///
/// Equal weights: (1/M) Σ|X_m − y| − 1/(2M²) Σ_{m,n}|X_m − X_n|. With normalised weights both sums
/// are weighted; the pair term uses the sorted O(M log M) identity
/// Σ_{i,j} w_i w_j |x_i − x_j| = 2 Σ_i w_i x_i (2 W_i + w_i − 1), W_i = Σ_{j<i} w_j.
pub fn crps_samples(samples: &[f64], y: f64, weights: Option<&[f64]>) -> f64 {
    let m = samples.len();
    let w0: Vec<f64> = match weights {
        Some(w) => w.to_vec(),
        None => vec![1.0 / m as f64; m],
    };
    let mut x = Vec::with_capacity(m);
    let mut w = Vec::with_capacity(m);
    for (xi, wi) in samples.iter().zip(&w0) {
        if xi.is_finite() && wi.is_finite() && *wi > 0.0 {
            x.push(*xi);
            w.push(*wi);
        }
    }
    if x.is_empty() || !y.is_finite() {
        return f64::NAN;
    }
    let tot = pairwise_sum(&w);
    for wi in &mut w {
        *wi /= tot;
    }
    let mut order: Vec<usize> = (0..x.len()).collect();
    order.sort_by(|&a, &b| x[a].partial_cmp(&x[b]).unwrap());
    let mut cum = 0.0;
    let mut terms = Vec::with_capacity(x.len());
    for &i in &order {
        terms.push(w[i] * x[i] * (2.0 * cum + w[i] - 1.0));
        cum += w[i];
    }
    let pair = 2.0 * pairwise_sum(&terms);
    let abs: Vec<f64> = x
        .iter()
        .zip(&w)
        .map(|(xi, wi)| wi * (xi - y).abs())
        .collect();
    pairwise_sum(&abs) - 0.5 * pair
}

pub fn pinball(q: f64, y: f64, tau: f64) -> f64 {
    (tau - if y < q { 1.0 } else { 0.0 }) * (y - q)
}

fn sort_levels(levels: &[f64], q: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut o: Vec<usize> = (0..levels.len()).collect();
    o.sort_by(|&a, &b| {
        levels[a]
            .partial_cmp(&levels[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    (
        o.iter().map(|&i| levels[i]).collect(),
        o.iter().map(|&i| q[i]).collect(),
    )
}

/// CRPS from a quantile set: CRPS = 2 ∫₀¹ ρ_τ(y − q_τ) dτ with midpoint weights
/// w_k = (τ_{k+1} − τ_{k−1})/2, τ₀ = 0, τ_{K+1} = 1. Only comparable on identical level sets.
pub fn crps_quantiles(levels: &[f64], q: &[f64], y: f64) -> f64 {
    if levels.is_empty() || !y.is_finite() || q.iter().any(|v| !v.is_finite()) {
        return f64::NAN;
    }
    let (t, qq) = sort_levels(levels, q);
    let mut ext = vec![0.0];
    ext.extend(&t);
    ext.push(1.0);
    let mut s = 0.0;
    for k in 0..t.len() {
        let wk = (ext[k + 2] - ext[k]) / 2.0;
        s += wk * pinball(qq[k], y, t[k]);
    }
    2.0 * s
}

#[derive(Debug, Clone)]
pub struct Interval {
    pub nominal: f64,
    pub lower: f64,
    pub upper: f64,
    pub covered: bool,
    pub width: f64,
}

/// Every central interval (τ, 1−τ) present in the level set: coverage indicator and width.
pub fn central_intervals(levels: &[f64], q: &[f64], y: f64) -> Vec<Interval> {
    let tol = 1e-9;
    let mut out = vec![];
    for (i, &lo) in levels.iter().enumerate() {
        if lo >= 0.5 {
            continue;
        }
        for (j, &hi) in levels.iter().enumerate() {
            if (hi - (1.0 - lo)).abs() < tol {
                let (a, b) = (q[i], q[j]);
                let nominal = ((1.0 - 2.0 * lo) * 1e9).round() / 1e9;
                out.push(Interval {
                    nominal,
                    lower: a,
                    upper: b,
                    covered: a <= y && y <= b,
                    width: b - a,
                });
            }
        }
    }
    out
}

/// Approximate log predictive density from quantiles (piecewise-linear CDF between knots, exponential
/// tails with the adjacent knot gap as scale). Exact densities in diagnostics take precedence.
pub fn lpd_quantiles(levels: &[f64], q: &[f64], y: f64) -> f64 {
    let (t, qq) = sort_levels(levels, q);
    if t.len() < 2 || !y.is_finite() {
        return f64::NAN;
    }
    let gaps: Vec<f64> = qq.windows(2).map(|w| w[1] - w[0]).collect();
    if gaps.iter().any(|g| !(*g > 0.0)) {
        return f64::NAN;
    }
    let n = t.len();
    if y < qq[0] {
        let s = gaps[0];
        return (t[0] / s).ln() - (qq[0] - y) / s;
    }
    if y > qq[n - 1] {
        let s = gaps[n - 2];
        return ((1.0 - t[n - 1]) / s).ln() - (y - qq[n - 1]) / s;
    }
    let k = (qq.partition_point(|&v| v <= y) as isize - 1)
        .min(n as isize - 2)
        .max(0) as usize;
    ((t[k + 1] - t[k]) / gaps[k]).ln()
}

pub fn lpd_student_t(y: f64, loc: f64, scale: f64, df: f64) -> f64 {
    let z = (y - loc) / scale;
    lgamma((df + 1.0) / 2.0)
        - lgamma(df / 2.0)
        - 0.5 * (df * std::f64::consts::PI).ln()
        - scale.ln()
        - (df + 1.0) / 2.0 * (z * z / df).ln_1p()
}

pub fn lpd_gaussian(y: f64, loc: f64, scale: f64) -> f64 {
    -0.5 * (2.0 * std::f64::consts::PI * scale * scale).ln() - 0.5 * ((y - loc) / scale).powi(2)
}

fn lgamma(x: f64) -> f64 {
    libm::lgamma(x)
}

// ---------------------------------------------------------------------------- competing events

/// Observed endpoint of one case, in the artifact's time frame.
///
/// `censoring`: event | interval | right | scheduled_stop | unknown. For interval censoring the event
/// lies in (t_lower, t_upper]. `causes` is the adjudicated admissible cause set.
#[derive(Debug, Clone, PartialEq)]
pub struct EventOutcome {
    pub censoring: String,
    pub t_lower: Option<f64>,
    pub t_upper: Option<f64>,
    pub causes: Vec<String>,
}

/// Kaplan–Meier of the censoring distribution G(t) = P(C > t). Events count as survivors of C.
pub fn km_censoring(times: &[f64], censored: &[bool]) -> (Vec<f64>, Vec<f64>) {
    let mut uniq: Vec<f64> = times
        .iter()
        .zip(censored)
        .filter(|(_, c)| **c)
        .map(|(t, _)| *t)
        .collect();
    uniq.sort_by(|a, b| a.partial_cmp(b).unwrap());
    uniq.dedup();
    let mut g = vec![];
    let mut surv = 1.0;
    for &u in &uniq {
        let at_risk = times.iter().filter(|&&t| t >= u).count() as f64;
        let d = times
            .iter()
            .zip(censored)
            .filter(|(t, c)| **t == u && **c)
            .count() as f64;
        surv *= 1.0 - d / at_risk;
        g.push(surv);
    }
    (uniq, g)
}

fn g_at(knots: &[f64], vals: &[f64], t: f64, left: bool) -> f64 {
    if knots.is_empty() {
        return 1.0;
    }
    let pos = if left {
        knots.partition_point(|&k| k < t)
    } else {
        knots.partition_point(|&k| k <= t)
    };
    if pos == 0 {
        1.0
    } else {
        vals[pos - 1]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
    Event(f64),
    Alive,
    Censored,
    Unknown,
}

fn status_at(o: &EventOutcome, h: f64) -> Status {
    match o.censoring.as_str() {
        "unknown" => Status::Unknown,
        "event" | "scheduled_stop" => match o.t_upper.or(o.t_lower) {
            Some(te) if te <= h => Status::Event(te),
            _ => Status::Alive,
        },
        "interval" => {
            if let Some(u) = o.t_upper {
                if u <= h {
                    return Status::Event(u);
                }
            }
            if let Some(l) = o.t_lower {
                if l > h {
                    return Status::Alive;
                }
            }
            Status::Censored
        }
        "right" => match o.t_lower.or(o.t_upper) {
            Some(c) if c > h => Status::Alive,
            _ => Status::Censored,
        },
        _ => Status::Unknown,
    }
}

pub struct IpcwTerms {
    pub eligible: Vec<bool>,
    pub weight: Vec<f64>,
    pub indicator: Vec<f64>,
    pub unknown: Vec<bool>,
    pub ambiguous: Vec<bool>,
}

/// Per-case IPCW eligibility, weight and indicator at `horizon` (Graf/Gerds estimator).
///
/// w_i = 1/G(T_i−) for events by h, 1/G(h) for cases still active at h, 0 for cases censored before h
/// (an interval straddling h counts as censored). G is the KM of right-censoring times (censoring
/// assumed independent of the event process). Unknown endpoints and, for cause-specific scores,
/// events with several admissible causes are not eligible and are counted separately.
pub fn ipcw_terms(outcomes: &[EventOutcome], cause: Option<&str>, horizon: f64) -> IpcwTerms {
    let mut times = vec![];
    let mut cens = vec![];
    for o in outcomes {
        match o.censoring.as_str() {
            "right" => {
                times.push(o.t_lower.or(o.t_upper).unwrap_or(f64::INFINITY));
                cens.push(true);
            }
            "event" | "scheduled_stop" | "interval" => {
                times.push(o.t_upper.or(o.t_lower).unwrap_or(f64::INFINITY));
                cens.push(false);
            }
            _ => {}
        }
    }
    let (knots, vals) = km_censoring(&times, &cens);
    let n = outcomes.len();
    let mut t = IpcwTerms {
        eligible: vec![false; n],
        weight: vec![0.0; n],
        indicator: vec![0.0; n],
        unknown: vec![false; n],
        ambiguous: vec![false; n],
    };
    for (i, o) in outcomes.iter().enumerate() {
        let st = status_at(o, horizon);
        if st == Status::Unknown {
            t.unknown[i] = true;
            continue;
        }
        if cause.is_some() && matches!(st, Status::Event(_)) && o.causes.len() != 1 {
            t.ambiguous[i] = true;
            continue;
        }
        t.eligible[i] = true;
        let g = match st {
            Status::Censored => continue,
            Status::Event(te) => g_at(&knots, &vals, te, true),
            _ => g_at(&knots, &vals, horizon, false),
        };
        if g > 0.0 {
            t.weight[i] = 1.0 / g;
        }
        let is_event = matches!(st, Status::Event(_));
        t.indicator[i] = if is_event && cause.map_or(true, |c| o.causes[0] == c) {
            1.0
        } else {
            0.0
        };
    }
    t
}

/// Cause-specific (`cause`) or any-event (None) IPCW Brier score at `horizon`.
///
/// BS(h) = (1/n) Σ_i w_i (I_i(h) − F_i(h))² over eligible cases; n includes eligible cases censored
/// before h (weight 0), so the denominator is not silently shrunk.
pub fn ipcw_brier(
    cif: &[f64],
    outcomes: &[EventOutcome],
    cause: Option<&str>,
    horizon: f64,
) -> Value {
    let t = ipcw_terms(outcomes, cause, horizon);
    let m: Vec<bool> = (0..cif.len())
        .map(|i| t.eligible[i] && cif[i].is_finite())
        .collect();
    let n = m.iter().filter(|b| **b).count();
    let terms: Vec<f64> = (0..cif.len())
        .filter(|&i| m[i])
        .map(|i| t.weight[i] * (t.indicator[i] - cif[i]).powi(2))
        .collect();
    let val = if n > 0 {
        pairwise_sum(&terms) / n as f64
    } else {
        f64::NAN
    };
    let cens = (0..cif.len())
        .filter(|&i| m[i] && t.weight[i] == 0.0)
        .count();
    let unk = t.unknown.iter().filter(|b| **b).count()
        + (0..cif.len())
            .filter(|&i| t.eligible[i] && !cif[i].is_finite())
            .count();
    json!({"brier": val, "n": n, "censored_before_horizon": cens, "excluded_unknown": unk,
           "excluded_ambiguous_cause": t.ambiguous.iter().filter(|b| **b).count()})
}

/// Trapezoidal integral of BS(h) over the horizon grid divided by its span (NaN entries dropped).
pub fn integrated_brier(horizons: &[f64], briers: &[f64]) -> f64 {
    let (h, b): (Vec<f64>, Vec<f64>) = horizons
        .iter()
        .zip(briers)
        .filter(|(_, b)| b.is_finite())
        .map(|(h, b)| (*h, *b))
        .unzip();
    if h.len() < 2 || h[h.len() - 1] <= h[0] {
        return f64::NAN;
    }
    let mut s = 0.0;
    let parts: Vec<f64> = (1..h.len())
        .map(|i| (h[i] - h[i - 1]) * (b[i] + b[i - 1]) / 2.0)
        .collect();
    s += pairwise_sum(&parts);
    s / (h[h.len() - 1] - h[0])
}

/// Time-dependent calibration at `horizon`: mean predicted vs IPCW-observed incidence per bin.
///
/// Bins are predicted-risk quantiles over eligible cases; observed incidence = Σ w_i I_i / n_bin.
pub fn time_calibration(
    cif: &[f64],
    outcomes: &[EventOutcome],
    cause: Option<&str>,
    horizon: f64,
    bins: usize,
) -> Value {
    let t = ipcw_terms(outcomes, cause, horizon);
    let idx: Vec<usize> = (0..cif.len())
        .filter(|&i| t.eligible[i] && cif[i].is_finite())
        .collect();
    if idx.is_empty() {
        return json!({"bins": [], "calibration_in_the_large": f64::NAN});
    }
    let p: Vec<f64> = idx.iter().map(|&i| cif[i]).collect();
    let w: Vec<f64> = idx.iter().map(|&i| t.weight[i]).collect();
    let ind: Vec<f64> = idx.iter().map(|&i| t.indicator[i]).collect();
    let mut sorted = p.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut edges: Vec<f64> = linspace(0.0, 1.0, bins + 1)
        .iter()
        .map(|&q| quantile_sorted(&sorted, q))
        .collect();
    edges.sort_by(|a, b| a.partial_cmp(b).unwrap());
    edges.dedup();
    let maxb = edges.len().saturating_sub(2) as isize;
    let which: Vec<isize> = p
        .iter()
        .map(|&x| (edges.partition_point(|&e| e <= x) as isize - 1).clamp(0, maxb.max(0)))
        .collect();
    let mut uniq = which.clone();
    uniq.sort();
    uniq.dedup();
    let mut out = vec![];
    for b in uniq {
        let k: Vec<usize> = (0..p.len()).filter(|&i| which[i] == b).collect();
        let pk: Vec<f64> = k.iter().map(|&i| p[i]).collect();
        let obs: Vec<f64> = k.iter().map(|&i| w[i] * ind[i]).collect();
        out.push(json!({
            "lo": pk.iter().cloned().fold(f64::INFINITY, f64::min),
            "hi": pk.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            "n": k.len(), "predicted": mean(&pk), "observed_ipcw": pairwise_sum(&obs) / k.len() as f64,
        }));
    }
    let wi: Vec<f64> = w.iter().zip(&ind).map(|(a, b)| a * b).collect();
    json!({"bins": out, "calibration_in_the_large": mean(&p) - pairwise_sum(&wi) / p.len() as f64})
}

// ------------------------------------------------------------------------------- BASS geometry

/// Masked geometric front errors on the calibrated plane (metres), arrays shaped (T, S).
///
/// MAE/RMSE/signed bias over jointly valid samples, per-frame MAE, symmetric Hausdorff distance per
/// frame, a least-squares advance scale factor and the best aligning time shift of the mean front,
/// so scale bias, timing error and shape error are reported separately.
pub fn front_errors(
    p: &[Vec<f64>],
    o: &[Vec<f64>],
    s_m: Option<&[f64]>,
    t_s: Option<&[f64]>,
) -> anyhow::Result<Value> {
    let nt = p.len();
    if o.len() != nt || p.iter().zip(o).any(|(a, b)| a.len() != b.len()) {
        anyhow::bail!("front shapes differ");
    }
    let ns = p.first().map_or(0, |r| r.len());
    let mv: Vec<Vec<bool>> = (0..nt)
        .map(|i| {
            (0..ns)
                .map(|k| o[i][k].is_finite() && p[i][k].is_finite())
                .collect()
        })
        .collect();
    let n: usize = mv.iter().map(|r| r.iter().filter(|b| **b).count()).sum();
    let size = (nt * ns).max(1);
    let mut out = Map::new();
    out.insert("n_valid".into(), json!(n));
    out.insert("valid_fraction".into(), json!(n as f64 / size as f64));
    if n == 0 {
        out.insert("reason".into(), json!("no jointly valid front samples"));
        return Ok(Value::Object(out));
    }
    let mut e = vec![];
    for i in 0..nt {
        for k in 0..ns {
            if mv[i][k] {
                e.push(p[i][k] - o[i][k]);
            }
        }
    }
    let abs: Vec<f64> = e.iter().map(|x| x.abs()).collect();
    let sq: Vec<f64> = e.iter().map(|x| x * x).collect();
    out.insert("mae_m".into(), json!(mean(&abs)));
    out.insert("rmse_m".into(), json!(mean(&sq).sqrt()));
    out.insert("bias_m".into(), json!(mean(&e)));
    let per_frame: Vec<Value> = (0..nt)
        .map(|i| {
            let v: Vec<f64> = (0..ns)
                .filter(|&k| mv[i][k])
                .map(|k| (p[i][k] - o[i][k]).abs())
                .collect();
            if v.is_empty() {
                Value::Null
            } else {
                json!(mean(&v))
            }
        })
        .collect();
    out.insert("per_frame_mae_m".into(), Value::Array(per_frame));
    let s: Vec<f64> = s_m
        .map(|s| s.to_vec())
        .unwrap_or_else(|| (0..ns).map(|k| k as f64).collect());
    let haus: Vec<Value> = (0..nt)
        .map(|i| {
            let ks: Vec<usize> = (0..ns).filter(|&k| mv[i][k]).collect();
            if ks.is_empty() {
                return Value::Null;
            }
            let d =
                |a: usize, b: usize| ((s[a] - s[b]).powi(2) + (p[i][a] - o[i][b]).powi(2)).sqrt();
            let ab = ks
                .iter()
                .map(|&a| ks.iter().map(|&b| d(a, b)).fold(f64::INFINITY, f64::min))
                .fold(f64::NEG_INFINITY, f64::max);
            let ba = ks
                .iter()
                .map(|&b| ks.iter().map(|&a| d(a, b)).fold(f64::INFINITY, f64::min))
                .fold(f64::NEG_INFINITY, f64::max);
            json!(ab.max(ba))
        })
        .collect();
    out.insert("hausdorff_m".into(), Value::Array(haus));
    let rows: Vec<usize> = (0..nt).filter(|&i| mv[i].iter().any(|b| *b)).collect();
    let row_mean = |a: &[Vec<f64>], i: usize| {
        mean(
            &(0..ns)
                .filter(|&k| mv[i][k])
                .map(|k| a[i][k])
                .collect::<Vec<_>>(),
        )
    };
    let pm: Vec<f64> = rows.iter().map(|&i| row_mean(p, i)).collect();
    let om: Vec<f64> = rows.iter().map(|&i| row_mean(o, i)).collect();
    let pa: Vec<f64> = pm.iter().map(|x| x - pm[0]).collect();
    let oa: Vec<f64> = om.iter().map(|x| x - om[0]).collect();
    let paa: f64 = pa.iter().map(|x| x * x).sum();
    let pao: f64 = pa.iter().zip(&oa).map(|(a, b)| a * b).sum();
    out.insert(
        "advance_scale".into(),
        if paa > 0.0 {
            json!(pao / paa)
        } else {
            Value::Null
        },
    );
    if let Some(ts) = t_s {
        if rows.len() >= 3 {
            let t: Vec<f64> = rows.iter().map(|&i| ts[i]).collect();
            let span = t[t.len() - 1] - t[0];
            let shifts = linspace(-0.5 * span, 0.5 * span, 101);
            let errs: Vec<f64> = shifts
                .iter()
                .map(|sh| {
                    let v: Vec<f64> = t
                        .iter()
                        .zip(&om)
                        .filter(|(tt, _)| **tt + sh >= t[0] && **tt + sh <= t[t.len() - 1])
                        .map(|(tt, omj)| (interp(tt + sh, &t, &pm) - omj).abs())
                        .collect();
                    if v.len() >= 2 {
                        mean(&v)
                    } else {
                        f64::INFINITY
                    }
                })
                .collect();
            let mut best = 0;
            for (i, e) in errs.iter().enumerate() {
                if *e < errs[best] {
                    best = i;
                }
            }
            out.insert("time_shift_s".into(), json!(shifts[best]));
        }
    }
    Ok(Value::Object(out))
}

// --------------------------------------------------------------------------- refusal accounting

/// Coverage of the evaluation (V3.1 §23.6). The denominator of every predictive metric is the
/// prospectively admitted set; admitted-but-unscorable cases stay in it and are listed with reasons.
pub struct AccountingCase {
    pub case_id: String,
    pub admitted: bool,
    pub scored: bool,
    pub reason: Option<String>,
}

pub fn refusal_accounting(cases: &[AccountingCase]) -> Value {
    let total = cases.len();
    let admitted: Vec<&AccountingCase> = cases.iter().filter(|c| c.admitted).collect();
    let refused: Vec<&AccountingCase> = cases.iter().filter(|c| !c.admitted).collect();
    let scored = admitted.iter().filter(|c| c.scored).count();
    let mut reasons = Map::new();
    for c in &refused {
        let r = c
            .reason
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unspecified".into());
        let n = reasons.get(&r).and_then(|v| v.as_u64()).unwrap_or(0) + 1;
        reasons.insert(r, json!(n));
    }
    let unscorable: Vec<Value> = admitted
        .iter()
        .filter(|c| !c.scored)
        .map(|c| json!({"case_id": c.case_id, "reason": c.reason}))
        .collect();
    json!({
        "presented": total,
        "refused": refused.len(),
        "refusal_rate": if total > 0 { json!(refused.len() as f64 / total as f64) } else { Value::Null },
        "refusal_reasons": reasons,
        "admitted_denominator": admitted.len(),
        "scored": scored,
        "admitted_unscorable": unscorable,
        "scored_fraction_of_admitted": if admitted.is_empty() { Value::Null } else { json!(scored as f64 / admitted.len() as f64) },
    })
}

//! Weighted particle summaries, posterior-predictive utilities and the candidate-measurement
//! interface used by observation-value ranking (V3.1 §10.1, §11.2).
//!
//! Importance weights are normalised in log space jointly over every particle index (global draw
//! and local sample together). Quantiles, means and proper scores are weighted; nothing here drops
//! particles that left support (that is the caller's partition, see `events`).

use rand::{Rng, RngExt};
use serde_json::{json, Value};

pub use ignis_core::expert::{CandidatePredictive, VoiParticles};

/// Returns (normalised weights, log of the mean unnormalised weight).
pub fn normalize_log_weights(logw: &[f64]) -> (Vec<f64>, f64) {
    let m = logw
        .iter()
        .copied()
        .filter(|x| x.is_finite())
        .fold(f64::NEG_INFINITY, f64::max);
    if !m.is_finite() {
        return (vec![0.0; logw.len()], f64::NEG_INFINITY);
    }
    let s: f64 = logw
        .iter()
        .filter(|x| x.is_finite())
        .map(|x| (x - m).exp())
        .sum();
    let lse = m + s.ln();
    let w = logw
        .iter()
        .map(|&x| if x.is_finite() { (x - lse).exp() } else { 0.0 })
        .collect();
    (w, lse - (logw.len() as f64).ln())
}

pub fn ess(w: &[f64]) -> f64 {
    let s2: f64 = w.iter().map(|x| x * x).sum();
    let s: f64 = w.iter().sum();
    if s2 > 0.0 {
        s * s / s2
    } else {
        0.0
    }
}

pub fn weight_diagnostics(w: &[f64]) -> Value {
    json!({"ess": ess(w), "max_weight": w.iter().copied().fold(0.0, f64::max), "n_particles": w.len(),
           "n_nonzero": w.iter().filter(|x| **x > 0.0).count()})
}

/// `np.interp` (clamped linear interpolation; xp increasing).
pub fn interp(x: f64, xp: &[f64], fp: &[f64]) -> f64 {
    if xp.is_empty() {
        return f64::NAN;
    }
    if x <= xp[0] {
        return fp[0];
    }
    let n = xp.len();
    if x >= xp[n - 1] {
        return fp[n - 1];
    }
    let j = xp.partition_point(|&v| v <= x);
    let (x0, x1) = (xp[j - 1], xp[j]);
    if x1 == x0 {
        return fp[j];
    }
    fp[j - 1] + (fp[j] - fp[j - 1]) * (x - x0) / (x1 - x0)
}

/// Weighted quantiles of one column: midpoint rule (unbiased for equal weights). NaN without mass.
pub fn weighted_quantiles_1d(x: &[f64], w: &[f64], levels: &[f64]) -> Vec<f64> {
    let mut idx: Vec<usize> = (0..x.len()).filter(|&i| w[i] > 0.0).collect();
    if idx.is_empty() {
        return vec![f64::NAN; levels.len()];
    }
    idx.sort_by(|&a, &b| x[a].total_cmp(&x[b]));
    let xs: Vec<f64> = idx.iter().map(|&i| x[i]).collect();
    let mut c = Vec::with_capacity(idx.len());
    let mut acc = 0.0;
    for &i in &idx {
        acc += w[i];
        c.push(acc);
    }
    let tot = acc;
    for (k, &i) in idx.iter().enumerate() {
        c[k] = (c[k] - 0.5 * w[i]) / tot;
    }
    levels.iter().map(|&q| interp(q, &c, &xs)).collect()
}

/// x: per particle rows (P, T); w: (P, T) column weights -> (levels, T).
pub fn weighted_quantiles(x: &[Vec<f64>], w: &[Vec<f64>], levels: &[f64]) -> Vec<Vec<f64>> {
    let nt = x.first().map_or(0, |r| r.len());
    let mut out = vec![vec![f64::NAN; nt]; levels.len()];
    let mut col = vec![0.0; x.len()];
    let mut wc = vec![0.0; x.len()];
    for j in 0..nt {
        for p in 0..x.len() {
            col[p] = x[p][j];
            wc[p] = w[p][j];
        }
        for (l, v) in weighted_quantiles_1d(&col, &wc, levels)
            .into_iter()
            .enumerate()
        {
            out[l][j] = v;
        }
    }
    out
}

pub fn weighted_mean_var(x: &[f64], w: &[f64]) -> (f64, f64) {
    let tot: f64 = w.iter().sum();
    if !(tot > 0.0) {
        return (0.0, 0.0);
    }
    let m: f64 = x.iter().zip(w).map(|(a, b)| a * b / tot).sum();
    let v: f64 = x
        .iter()
        .zip(w)
        .map(|(a, b)| b / tot * (a - m) * (a - m))
        .sum();
    (m, v)
}

/// CRPS for a weighted sample (V3.1 §13.3 estimator with importance weights in both sums).
pub fn weighted_crps(samples: &[f64], w: &[f64], y: f64) -> f64 {
    let tot: f64 = w.iter().sum();
    let wn: Vec<f64> = w.iter().map(|v| v / tot).collect();
    let a: f64 = samples
        .iter()
        .zip(&wn)
        .map(|(x, w)| w * (x - y).abs())
        .sum();
    let mut b = 0.0;
    for (i, xi) in samples.iter().enumerate() {
        for (j, xj) in samples.iter().enumerate() {
            b += wn[i] * wn[j] * (xi - xj).abs();
        }
    }
    a - 0.5 * b
}

pub fn systematic_resample<G: Rng + ?Sized>(w: &[f64], n: usize, rng: &mut G) -> Vec<usize> {
    let tot: f64 = w.iter().sum();
    let mut c = Vec::with_capacity(w.len());
    let mut acc = 0.0;
    for x in w {
        acc += x / tot;
        c.push(acc);
    }
    let u0: f64 = rng.random();
    (0..n)
        .map(|i| {
            let u = (u0 + i as f64) / n as f64;
            c.partition_point(|&v| v < u).min(w.len() - 1)
        })
        .collect()
}

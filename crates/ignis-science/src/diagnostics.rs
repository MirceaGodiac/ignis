//! Convergence diagnostics (V3.1 §12.3, §13.6): rank-normalised split R-hat (Vehtari et al. 2021),
//! bulk and tail ESS (Geyer initial monotone sequence, as NumPyro/Stan), divergences, E-BFMI.
//! Thresholds (R-hat <= 1.01 etc.) are engineering alarms, not proof of convergence.

use serde_json::{json, Map, Value};

/// Inverse standard-normal CDF (Acklam's rational approximation + one Halley step, |err| < 1e-15).
pub fn ndtri(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    const A: [f64; 6] = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383577518672690e2,
        -3.066479806614716e1,
        2.506628277459239,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838,
        -2.549732539343734,
        4.374664141464968,
        2.938163982698783,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996,
        3.754408661907416,
    ];
    let pl = 0.02425;
    let x = if p < pl {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= 1.0 - pl {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    let e = 0.5 * libm::erfc(-x / std::f64::consts::SQRT_2) - p;
    let u = e * (2.0 * std::f64::consts::PI).sqrt() * (x * x / 2.0).exp();
    x - u / (1.0 + x * u / 2.0)
}

/// (chains, draws) as rows.
pub type Draws = Vec<Vec<f64>>;

fn split(x: &Draws) -> Draws {
    let n = x[0].len();
    let h = n / 2;
    let mut out = Vec::with_capacity(2 * x.len());
    for c in x {
        out.push(c[..h].to_vec());
    }
    for c in x {
        out.push(c[n - h..].to_vec());
    }
    out
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn var(v: &[f64], ddof: f64) -> f64 {
    let m = mean(v);
    v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() as f64 - ddof)
}

fn rhat_basic(x: &Draws) -> f64 {
    let (c, n) = (x.len(), x[0].len());
    if n < 2 {
        return f64::NAN;
    }
    let means: Vec<f64> = x.iter().map(|r| mean(r)).collect();
    let w = x.iter().map(|r| var(r, 1.0)).sum::<f64>() / c as f64;
    let b = if c > 1 {
        n as f64 * var(&means, 1.0)
    } else {
        0.0
    };
    let v = (n as f64 - 1.0) / n as f64 * w + b / n as f64;
    if w > 0.0 {
        (v / w).sqrt()
    } else {
        f64::NAN
    }
}

/// scipy `rankdata(method="average")` then ndtri((r - 3/8)/(S + 1/4)).
fn rank_normalize(x: &Draws) -> Draws {
    let flat: Vec<f64> = x.iter().flatten().copied().collect();
    let s = flat.len();
    let mut idx: Vec<usize> = (0..s).collect();
    idx.sort_by(|&a, &b| flat[a].total_cmp(&flat[b]));
    let mut ranks = vec![0.0; s];
    let mut i = 0;
    while i < s {
        let mut j = i;
        while j + 1 < s && flat[idx[j + 1]] == flat[idx[i]] {
            j += 1;
        }
        let r = (i + j) as f64 / 2.0 + 1.0;
        for k in i..=j {
            ranks[idx[k]] = r;
        }
        i = j + 1;
    }
    let n = x[0].len();
    ranks
        .chunks(n)
        .map(|c| {
            c.iter()
                .map(|r| ndtri((r - 0.375) / (s as f64 + 0.25)))
                .collect()
        })
        .collect()
}

/// max(bulk, folded) rank-normalised split R-hat for one scalar.
pub fn rank_rhat(x: &Draws) -> f64 {
    let xs = split(x);
    if xs[0].len() < 2 || xs.iter().flatten().all(|v| *v == xs[0][0]) {
        return f64::NAN;
    }
    let bulk = rhat_basic(&rank_normalize(&xs));
    let mut flat: Vec<f64> = xs.iter().flatten().copied().collect();
    flat.sort_by(f64::total_cmp);
    let k = flat.len();
    let med = if k % 2 == 1 {
        flat[k / 2]
    } else {
        0.5 * (flat[k / 2 - 1] + flat[k / 2])
    };
    let folded: Draws = xs
        .iter()
        .map(|r| r.iter().map(|v| (v - med).abs()).collect())
        .collect();
    bulk.max(rhat_basic(&rank_normalize(&folded)))
}

/// NumPyro `effective_sample_size` (Geyer initial monotone sequence on the multi-chain
/// autocorrelation), with lazily computed lags (exact: lags after the first non-positive pair do
/// not contribute).
pub fn ess_raw(x: &Draws) -> f64 {
    let (c, n) = (x.len(), x[0].len());
    if n < 2 {
        return f64::NAN;
    }
    let centered: Vec<Vec<f64>> = x
        .iter()
        .map(|r| {
            let m = mean(r);
            r.iter().map(|v| v - m).collect()
        })
        .collect();
    let chain_mean: Vec<f64> = x.iter().map(|r| mean(r)).collect();
    let var_within0 = x.iter().map(|r| var(r, 1.0)).sum::<f64>() / c as f64;
    let mut var_est = var_within0 * (n as f64 - 1.0) / n as f64;
    let var_within = if c > 1 {
        var_est += var(&chain_mean, 1.0);
        var_within0
    } else {
        var_est
    };
    if !(var_est > 0.0) {
        return f64::NAN;
    }
    let gamma = |k: usize| -> f64 {
        centered
            .iter()
            .map(|r| (0..n - k).map(|t| r[t] * r[t + k]).sum::<f64>() / n as f64)
            .sum::<f64>()
            / c as f64
    };
    let rho = |k: usize| {
        if k == 0 {
            1.0
        } else {
            1.0 - (var_within - gamma(k)) / var_est
        }
    };
    // Rho_k = rho[2k] + rho[2k+1] for 2k+1 < n (pairs from rho[:-1:2] and rho[1::2])
    let npairs = n / 2;
    let first = rho(0) + rho(1);
    let mut tau_half = first;
    let mut run_min = f64::INFINITY;
    for k in 1..npairs {
        let p = (rho(2 * k) + rho(2 * k + 1)).max(0.0);
        run_min = run_min.min(p);
        if run_min <= 0.0 {
            break;
        }
        tau_half += run_min;
    }
    let tau = -1.0 + 2.0 * tau_half;
    (c * n) as f64 / tau
}

pub fn bulk_ess(x: &Draws) -> f64 {
    let xs = split(x);
    if xs[0].len() < 4 {
        f64::NAN
    } else {
        ess_raw(&rank_normalize(&xs))
    }
}

pub fn tail_ess(x: &Draws) -> f64 {
    let xs = split(x);
    if xs[0].len() < 4 {
        return f64::NAN;
    }
    let mut flat: Vec<f64> = xs.iter().flatten().copied().collect();
    flat.sort_by(f64::total_cmp);
    let q = |p: f64| {
        let pos = p * (flat.len() - 1) as f64;
        let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
        flat[lo] + (flat[hi] - flat[lo]) * (pos - lo as f64)
    };
    let (q05, q95) = (q(0.05), q(0.95));
    let ind = |t: f64| -> Draws {
        xs.iter()
            .map(|r| r.iter().map(|v| if *v <= t { 1.0 } else { 0.0 }).collect())
            .collect()
    };
    ess_raw(&ind(q05)).min(ess_raw(&ind(q95)))
}

pub fn ebfmi(energy: &[Vec<f64>]) -> Vec<f64> {
    energy
        .iter()
        .map(|e| {
            if e.len() <= 2 {
                return f64::NAN;
            }
            let m = mean(e);
            let num: f64 = e.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
            num / e.iter().map(|v| (v - m) * (v - m)).sum::<f64>()
        })
        .collect()
}

/// Sampler statistics per chain and draw.
#[derive(Debug, Clone, Default)]
pub struct ChainStats {
    pub diverging: Vec<bool>,
    pub energy: Vec<f64>,
    pub num_steps: Vec<u64>,
    pub depth: Vec<u64>,
    pub maxdepth_reached: Vec<bool>,
    pub step_size: f64,
}

/// samples[block] = (names, per-scalar draws as (chains, draws)). Worst-case per block + pass flags.
pub fn summarize(
    blocks: &[(String, Vec<Draws>)],
    stats: &[ChainStats],
    rhat_max: f64,
    min_ess: f64,
) -> Value {
    let mut out_blocks = Map::new();
    let (mut worst_rhat, mut worst_bulk, mut worst_tail) = (0.0f64, f64::INFINITY, f64::INFINITY);
    for (name, scalars) in blocks {
        let rh: Vec<f64> = scalars
            .iter()
            .map(rank_rhat)
            .filter(|v| v.is_finite())
            .collect();
        let be: Vec<f64> = scalars
            .iter()
            .map(bulk_ess)
            .filter(|v| v.is_finite())
            .collect();
        let te: Vec<f64> = scalars
            .iter()
            .map(tail_ess)
            .filter(|v| v.is_finite())
            .collect();
        let mx = rh.iter().copied().fold(f64::NAN, f64::max);
        let mb = be.iter().copied().fold(f64::NAN, f64::min);
        let mt = te.iter().copied().fold(f64::NAN, f64::min);
        out_blocks.insert(
            name.clone(),
            json!({"rhat_max": fin(mx), "bulk_ess_min": fin(mb), "tail_ess_min": fin(mt)}),
        );
        if mx.is_finite() {
            worst_rhat = worst_rhat.max(mx);
        }
        if mb.is_finite() {
            worst_bulk = worst_bulk.min(mb);
        }
        if mt.is_finite() {
            worst_tail = worst_tail.min(mt);
        }
    }
    let n_chains = blocks
        .first()
        .and_then(|b| b.1.first())
        .map_or(0, |d| d.len());
    let div: usize = stats
        .iter()
        .map(|s| s.diverging.iter().filter(|b| **b).count())
        .sum();
    let total: usize = stats.iter().map(|s| s.diverging.len()).sum();
    let steps: Vec<f64> = stats
        .iter()
        .flat_map(|s| s.num_steps.iter().map(|&x| x as f64))
        .collect();
    let depth_sat: usize = stats
        .iter()
        .map(|s| s.maxdepth_reached.iter().filter(|b| **b).count())
        .sum();
    let pass = n_chains >= 2
        && worst_rhat <= rhat_max
        && worst_bulk >= min_ess
        && worst_tail >= min_ess
        && div == 0;
    let mut out = json!({
        "blocks": out_blocks, "rhat_max": fin(worst_rhat), "bulk_ess_min": fin(worst_bulk), "tail_ess_min": fin(worst_tail),
        "thresholds": {"rhat_max": rhat_max, "min_ess": min_ess},
        "divergences": div, "divergence_fraction": if total > 0 { div as f64 / total as f64 } else { 0.0 },
        "ebfmi": ebfmi(&stats.iter().map(|s| s.energy.clone()).collect::<Vec<_>>()).into_iter().map(fin).collect::<Vec<_>>(),
        "mean_leapfrog_steps": if steps.is_empty() { Value::Null } else { json!(steps.iter().sum::<f64>() / steps.len() as f64) },
        "tree_depth_saturated_fraction": if total > 0 { depth_sat as f64 / total as f64 } else { 0.0 },
        "step_size": stats.iter().map(|s| s.step_size).collect::<Vec<_>>(),
        "pass": pass,
    });
    if n_chains < 2 {
        out["note"] =
            json!("single chain: R-hat between chains is not estimable; not a convergence pass");
    }
    out
}

fn fin(x: f64) -> Value {
    if x.is_finite() {
        json!(x)
    } else {
        Value::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_distr::{Distribution, StandardNormal};

    #[test]
    fn ndtri_inverts_the_normal_cdf() {
        for p in [1e-12, 0.01, 0.3, 0.5, 0.77, 0.999] {
            let x = ndtri(p);
            assert!(
                (0.5 * libm::erfc(-x / std::f64::consts::SQRT_2) - p).abs()
                    < 1e-15 * p.max(1e-3) * 10.0
            );
        }
    }

    #[test]
    fn iid_chains_have_rhat_one_and_full_ess() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(3);
        let x: Draws = (0..4)
            .map(|_| (0..1000).map(|_| StandardNormal.sample(&mut rng)).collect())
            .collect();
        assert!((rank_rhat(&x) - 1.0).abs() < 0.01);
        let e = bulk_ess(&x);
        assert!(e > 2500.0 && e < 6000.0, "{e}");
        let shifted: Draws = x
            .iter()
            .enumerate()
            .map(|(c, r)| r.iter().map(|v| v + c as f64).collect())
            .collect();
        assert!(rank_rhat(&shifted) > 1.5);
    }
}

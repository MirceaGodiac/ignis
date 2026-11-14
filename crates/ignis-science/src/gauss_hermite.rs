//! Gauss–Hermite quadrature for correlated Gaussian local effects and joint normalisation (V3.1 §10.1).
//!
//! For x_i, w_i the nodes/weights of ∫ exp(−x²) g(x) dx, a d-dimensional Gaussian u ~ N(0, L Lᵀ) is
//! integrated with nodes u = √2 · L · (x_{i1}, …, x_{id}) and prior weights Π w / π^{d/2}.
//!
//! Prefix weights over (global draw g, node n) are normalised JOINTLY over g and n (never per g and
//! then averaged with equal global weight): the prefix also informs the global distribution.
//! The same log-sum-exp normalisation and ESS = 1/Σ w̄² serve the importance-sampling paths (§10.5).
//!
//! `hermite_rule` reproduces NumPy's `hermgauss` (Newton-polished roots of the normalised Hermite
//! recurrence, the same weight scaling and symmetrisation), so nodes/weights agree to rounding.

use serde_json::{json, Value};

/// Normalised Hermite function value h_n(x) (NumPy `_normed_hermite_n`).
fn normed_hermite(x: f64, n: usize) -> f64 {
    let r = 1.0 / std::f64::consts::PI.sqrt().sqrt();
    if n == 0 {
        return r;
    }
    let mut c0 = 0.0;
    let mut c1 = r;
    let mut nd = n as f64;
    for _ in 0..n - 1 {
        let tmp = c0;
        c0 = -c1 * ((nd - 1.0) / nd).sqrt();
        c1 = tmp + c1 * x * (2.0 / nd).sqrt();
        nd -= 1.0;
    }
    c0 + c1 * x * 2f64.sqrt()
}

/// Physicists' Gauss–Hermite nodes (ascending) and weights for weight exp(−x²), FP64.
pub fn hermite_rule(n: usize) -> (Vec<f64>, Vec<f64>) {
    assert!(n >= 1, "at least one node");
    if n == 1 {
        return (vec![0.0], vec![std::f64::consts::PI.sqrt()]);
    }
    // Initial guesses (Numerical Recipes `gauher`, largest root first), then Newton on h_n.
    let nf = n as f64;
    let mut roots = vec![0.0; n];
    let half = n.div_ceil(2);
    let mut z = 0.0f64;
    for i in 0..half {
        z = match i {
            0 => (2.0 * nf + 1.0).sqrt() - 1.85575 * (2.0 * nf + 1.0).powf(-0.16667),
            1 => z - 1.14 * nf.powf(0.426) / z,
            2 => 1.86 * z - 0.86 * roots[0],
            3 => 1.91 * z - 0.91 * roots[1],
            _ => 2.0 * z - roots[i - 2],
        };
        for _ in 0..100 {
            let dy = normed_hermite(z, n);
            let df = normed_hermite(z, n - 1) * (2.0 * nf).sqrt();
            let step = dy / df;
            z -= step;
            if step.abs() <= 1e-15 * z.abs().max(1.0) {
                break;
            }
        }
        roots[i] = z;
    }
    let mut x = vec![0.0; n];
    for i in 0..half {
        x[n - 1 - i] = roots[i];
        x[i] = -roots[i];
    }
    if n % 2 == 1 {
        x[n / 2] = 0.0;
    }
    // One more Newton application exactly as NumPy does after its eigenvalue solve.
    for xi in x.iter_mut() {
        let dy = normed_hermite(*xi, n);
        let df = normed_hermite(*xi, n - 1) * (2.0 * nf).sqrt();
        *xi -= dy / df;
    }
    let mut fm: Vec<f64> = x.iter().map(|&xi| normed_hermite(xi, n - 1)).collect();
    let mx = fm.iter().fold(0.0f64, |a, b| a.max(b.abs()));
    fm.iter_mut().for_each(|f| *f /= mx);
    let w0: Vec<f64> = fm.iter().map(|f| 1.0 / (f * f)).collect();
    let mut w: Vec<f64> = (0..n).map(|i| (w0[i] + w0[n - 1 - i]) / 2.0).collect();
    let xs: Vec<f64> = (0..n).map(|i| (x[i] - x[n - 1 - i]) / 2.0).collect();
    let s: f64 = w.iter().sum();
    let scale = std::f64::consts::PI.sqrt() / s;
    w.iter_mut().for_each(|v| *v *= scale);
    (xs, w)
}

/// Tensor-product standard-normal nodes z (n^dim × dim, row-major; last index fastest as in
/// `itertools.product`) and log prior weights (n^dim). Σ weights = 1 and Σ w z zᵀ = I for n ≥ 2.
pub fn standard_grid(n: usize, dim: usize) -> (Vec<f64>, Vec<f64>) {
    let (x, w) = hermite_rule(n);
    let total = n.pow(dim as u32);
    let mut z = vec![0.0; total * dim];
    let mut logw = vec![0.0; total];
    let lw: Vec<f64> = w.iter().map(|v| v.ln()).collect();
    let c = 0.5 * dim as f64 * std::f64::consts::PI.ln();
    let mut idx = vec![0usize; dim];
    for p in 0..total {
        let mut rem = p;
        for d in (0..dim).rev() {
            idx[d] = rem % n;
            rem /= n;
        }
        let mut s = 0.0;
        for d in 0..dim {
            z[p * dim + d] = 2f64.sqrt() * x[idx[d]];
            s += lw[idx[d]];
        }
        logw[p] = s - c;
    }
    (z, logw)
}

/// u = L z for every node: `chol` (d×d row-major lower), `z` (N×d) → (N×d).
pub fn correlated_nodes(chol: &[f64], z: &[f64], d: usize) -> Vec<f64> {
    let n = z.len() / d;
    let mut out = vec![0.0; n * d];
    for p in 0..n {
        for i in 0..d {
            let mut s = 0.0;
            for j in 0..d {
                s += chol[i * d + j] * z[p * d + j];
            }
            out[p * d + i] = s;
        }
    }
    out
}

/// log Σ exp(a); −∞ for an empty or all −∞ input.
pub fn logsumexp(a: &[f64]) -> f64 {
    let m = a.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if !m.is_finite() {
        return if m == f64::INFINITY {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        };
    }
    m + a.iter().map(|v| (v - m).exp()).sum::<f64>().ln()
}

/// Normalise log weights over ALL entries jointly. Returns (weights, log normaliser).
pub fn joint_normalize(log_w: &[f64]) -> (Vec<f64>, f64) {
    let lz = logsumexp(log_w);
    (log_w.iter().map(|v| (v - lz).exp()).collect(), lz)
}

pub struct JointWeights {
    /// (G × N) row-major joint weights.
    pub w: Vec<f64>,
    /// (G,) global marginal.
    pub global: Vec<f64>,
    pub log_evidence: f64,
}

/// W_{g n} ∝ w_g^train · w_n^prior · L(prefix | Θ_g, u_{g n}) normalised jointly over (g, n).
/// `log_lik` is (G × N) row-major (includes S(τ)/S(entry) and support indicators as −∞).
pub fn joint_global_local_weights(
    log_w_global: &[f64],
    log_prior_nodes: &[f64],
    log_lik: &[f64],
) -> JointWeights {
    let g = log_w_global.len();
    let n = log_prior_nodes.len();
    assert_eq!(log_lik.len(), g * n);
    let mut lw = vec![0.0; g * n];
    let mut prior = vec![0.0; g * n];
    for a in 0..g {
        for b in 0..n {
            prior[a * n + b] = log_w_global[a] + log_prior_nodes[b];
            lw[a * n + b] = prior[a * n + b] + log_lik[a * n + b];
        }
    }
    let (w, lz) = joint_normalize(&lw);
    let global = (0..g).map(|a| w[a * n..(a + 1) * n].iter().sum()).collect();
    JointWeights {
        w,
        global,
        log_evidence: lz - logsumexp(&prior),
    }
}

/// Kish effective sample size 1/Σ w̄² of (not necessarily normalised) weights.
pub fn ess(weights: &[f64]) -> f64 {
    let s: f64 = weights.iter().sum();
    1.0 / weights.iter().map(|w| (w / s) * (w / s)).sum::<f64>()
}

/// ESS, ESS fraction, largest normalised weight and finite fraction for raw log weights.
pub fn weight_diagnostics(log_w: &[f64]) -> Value {
    let n = log_w.len();
    let finite: Vec<bool> = log_w.iter().map(|v| v.is_finite()).collect();
    let nf = finite.iter().filter(|b| **b).count();
    if nf == 0 {
        return json!({"ess": 0.0, "ess_fraction": 0.0, "max_weight": null, "finite_fraction": 0.0, "n": n});
    }
    let m = log_w
        .iter()
        .zip(&finite)
        .filter(|(_, f)| **f)
        .map(|(v, _)| *v)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut w: Vec<f64> = log_w
        .iter()
        .zip(&finite)
        .map(|(v, f)| if *f { (v - m).exp() } else { 0.0 })
        .collect();
    let s: f64 = w.iter().sum();
    w.iter_mut().for_each(|x| *x /= s);
    let e = 1.0 / w.iter().map(|x| x * x).sum::<f64>();
    let mx = w.iter().cloned().fold(0.0, f64::max);
    json!({"ess": e, "ess_fraction": e / n as f64, "max_weight": mx,
           "finite_fraction": nf as f64 / n as f64, "n": n})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_integrates_polynomials_and_normal_moments() {
        for n in [2usize, 5, 12, 24, 48] {
            let (x, w) = hermite_rule(n);
            assert!(x.windows(2).all(|p| p[1] > p[0]));
            let s: f64 = w.iter().sum();
            assert!((s - std::f64::consts::PI.sqrt()).abs() < 1e-13);
            // ∫ x² e^{-x²} = √π/2
            let m2: f64 = x.iter().zip(&w).map(|(a, b)| a * a * b).sum();
            assert!(
                (m2 - std::f64::consts::PI.sqrt() / 2.0).abs() < 1e-12,
                "n={n} m2={m2}"
            );
            let (z, lw) = standard_grid(n, 2);
            let tot: f64 = lw.iter().map(|v| v.exp()).sum();
            assert!((tot - 1.0).abs() < 1e-13);
            let c01: f64 = (0..lw.len())
                .map(|p| lw[p].exp() * z[2 * p] * z[2 * p + 1])
                .sum();
            let c00: f64 = (0..lw.len())
                .map(|p| lw[p].exp() * z[2 * p] * z[2 * p])
                .sum();
            assert!(c01.abs() < 1e-13 && (c00 - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn joint_weights_normalise_over_both_axes() {
        let jw = joint_global_local_weights(
            &[-(2f64).ln(); 2],
            &[-(3f64).ln(); 3],
            &[0.0, 1.0, 2.0, -1.0, 0.0, 0.5],
        );
        assert!((jw.w.iter().sum::<f64>() - 1.0).abs() < 1e-15);
        assert!((jw.global.iter().sum::<f64>() - 1.0).abs() < 1e-15);
        assert!(jw.global[0] > jw.global[1]);
    }
}

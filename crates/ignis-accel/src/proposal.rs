//! Defensive-mixture proposal and importance reweighting (V3.1 §10.5). The target never changes.
//!
//!   q_ε(b | Θ, Y) = (1 − ε̂) q_ψ(b | Θ, Y) + ε̂ p₀(b | Θ)
//!   log w = log p₀(b|Θ) + log L(Y|Θ,b) + log S(τ|Θ,b) − log q_ε(b|Θ,Y) + log 1_support
//!
//! The allocation is stratified (⌈εn⌉ draws from p₀), so the mixture weight in the density is the
//! REALISED fraction ε̂ = ⌈εn⌉/n, not the nominal ε (review-B fix 3: E[ŵ] = evidence). The density
//! includes BOTH components for every particle (log-sum-exp). ESS = 1/Σ w̄²; below `refine_below`
//! (20% of particles, an engineering setting) the runtime refines — never by changing the likelihood.

use ignis_science::gauss_hermite as gh;
use serde_json::Value;

pub fn defensive_counts(n: usize, epsilon: f64) -> (usize, usize) {
    let n_def = if epsilon > 0.0 {
        (epsilon * n as f64).ceil() as usize
    } else {
        0
    };
    (n - n_def, n_def)
}

/// Fraction of prior draws actually allocated (⌈εn⌉/n).
pub fn realised_epsilon(n: usize, epsilon: f64) -> f64 {
    defensive_counts(n, epsilon).1 as f64 / n as f64
}

fn logaddexp(a: f64, b: f64) -> f64 {
    let m = a.max(b);
    if m == f64::NEG_INFINITY {
        return m;
    }
    m + ((a - m).exp() + (b - m).exp()).ln()
}

/// log q_ε for particles drawn by the stratified allocation of `n` particles.
pub fn defensive_log_density(
    log_q_psi: &[f64],
    log_p0: &[f64],
    epsilon: f64,
    n: usize,
) -> Vec<f64> {
    let eps = if epsilon > 0.0 {
        realised_epsilon(n, epsilon)
    } else {
        0.0
    };
    if eps <= 0.0 {
        return log_q_psi.to_vec();
    }
    log_q_psi
        .iter()
        .zip(log_p0)
        .map(|(q, p)| logaddexp((-eps).ln_1p() + q, eps.ln() + p))
        .collect()
}

pub struct Reweighted {
    pub weights: Vec<f64>,
    pub log_weights: Vec<f64>,
    pub log_evidence_estimate: f64,
    pub diagnostics: Value,
    pub refine: bool,
}

/// Self-normalised importance weights for an unnormalised target.
pub fn reweight(
    log_target: &[f64],
    log_q: &[f64],
    log_global_ratio: f64,
    refine_below: f64,
) -> Reweighted {
    let lw: Vec<f64> = log_target
        .iter()
        .zip(log_q)
        .map(|(t, q)| {
            let v = t + log_global_ratio - q;
            if v.is_finite() {
                v
            } else {
                f64::NEG_INFINITY
            }
        })
        .collect();
    let (w, lz) = gh::joint_normalize(&lw);
    let d = gh::weight_diagnostics(&lw);
    let refine = d["ess_fraction"].as_f64().unwrap_or(0.0) < refine_below;
    Reweighted {
        weights: w,
        log_evidence_estimate: lz - (lw.len() as f64).ln(),
        log_weights: lw,
        diagnostics: d,
        refine,
    }
}

pub fn gaussian_log_density(x: &[f64], mean: &[f64], sd: &[f64]) -> f64 {
    x.iter()
        .zip(mean)
        .zip(sd)
        .map(|((x, m), s)| {
            -0.5 * ((x - m) / s).powi(2) - s.ln() - 0.5 * (2.0 * std::f64::consts::PI).ln()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, RngExt, SeedableRng};
    use rand_distr::StandardNormal;

    /// Prior N(0,1), y | b ~ N(b, 0.5²), y = 1.4: posterior mean y/(1 + 0.25). Biased, narrow proposal.
    #[test]
    fn defensive_mixture_importance_correction() {
        let (y, s) = (1.4, 0.5);
        let analytic = y / (1.0 + s * s);
        let n = 100_000;
        let eps = 0.1;
        let (n_psi, n_def) = defensive_counts(n, eps);
        let mut rng = StdRng::seed_from_u64(20261105);
        let x: Vec<f64> = (0..n)
            .map(|i| {
                if i < n_psi {
                    0.9 + 0.35 * rng.sample::<f64, _>(StandardNormal)
                } else {
                    rng.sample::<f64, _>(StandardNormal)
                }
            })
            .collect();
        assert_eq!(n_def, 10_000);
        let lq: Vec<f64> = x
            .iter()
            .map(|v| gaussian_log_density(&[*v], &[0.9], &[0.35]))
            .collect();
        let lp: Vec<f64> = x
            .iter()
            .map(|v| gaussian_log_density(&[*v], &[0.0], &[1.0]))
            .collect();
        let ld = defensive_log_density(&lq, &lp, eps, n);
        for i in 0..10 {
            let both = logaddexp(0.9f64.ln() + lq[i], 0.1f64.ln() + lp[i]);
            assert!(
                (ld[i] - both).abs() < 1e-12,
                "both components in the denominator"
            );
        }
        let target: Vec<f64> = x
            .iter()
            .zip(&lp)
            .map(|(b, p)| p - 0.5 * ((y - b) / s).powi(2))
            .collect();
        let r = reweight(&target, &ld, 0.0, 0.2);
        let est: f64 = r.weights.iter().zip(&x).map(|(w, b)| w * b).sum();
        assert!((est - analytic).abs() < 1e-2, "{est} vs {analytic}");
        // omitting the prior component biases the estimate
        let naive = reweight(&target, &lq, 0.0, 0.2);
        let est_naive: f64 = naive.weights.iter().zip(&x).map(|(w, b)| w * b).sum();
        assert!((est_naive - analytic).abs() > (est - analytic).abs());
        let ef = r.diagnostics["ess_fraction"].as_f64().unwrap();
        assert!(ef > 0.0 && ef <= 1.0 && r.diagnostics["max_weight"].as_f64().unwrap() < 0.01);
    }

    #[test]
    fn realised_fraction_makes_the_evidence_unbiased() {
        // exact expectation of the unnormalised weight under the stratified allocation equals Z
        let (y, s) = (1.3f64, 0.5f64);
        let z_true = (-0.5 * y * y / (1.0 + s * s)).exp()
            / (2.0 * std::f64::consts::PI * (1.0 + s * s)).sqrt();
        for (n, eps) in [(10usize, 0.15), (64, 0.1)] {
            let (n_psi, n_def) = defensive_counts(n, eps);
            let (pm, ps) = (y / (1.0 + s * s) + 0.6, 0.25);
            // ∫ [(n_psi q + n_def p0)/n] · p0 L / q_ε dx by midpoint quadrature
            let mut acc = 0.0;
            let h = 1e-4;
            let mut xg = -12.0;
            while xg < 12.0 {
                let lq = gaussian_log_density(&[xg], &[pm], &[ps]);
                let lp = gaussian_log_density(&[xg], &[0.0], &[1.0]);
                let qe = defensive_log_density(&[lq], &[lp], eps, n)[0].exp();
                let sampling = (n_psi as f64 * lq.exp() + n_def as f64 * lp.exp()) / n as f64;
                let lik = (-0.5 * ((y - xg) / s).powi(2)).exp()
                    / (s * (2.0 * std::f64::consts::PI).sqrt());
                acc += sampling * lp.exp() * lik / qe * h;
                xg += h;
            }
            assert!(
                (acc / z_true - 1.0).abs() < 1e-6,
                "n={n} eps={eps}: ratio {}",
                acc / z_true
            );
        }
    }
}

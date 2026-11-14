//! Frozen numeric kernels (V3.1 §7.2–7.4).
//!
//! Student-t process on irregular times: `e ~ t_ν(0, Σ)`, `Σ_jk = σ_c² exp(-|t_j-t_k|/ℓ) + σ_n² 1[j=k]`,
//! ν = 4. `Σ` is a *scale* matrix. The exponential kernel has an exact OU state-space form, so one
//! Kalman pass gives `log|Σ| = Σ log F_j` and `δ = eᵀΣ⁻¹e = Σ v_j²/F_j` in O(n); these enter the
//! closed Student-t density. No Gaussian noise is simulated in place of the Student-t.

use thiserror::Error;

/// Degrees of freedom fixed by the spec. Changing it requires a new manifest (§14.3).
pub const NU: f64 = 4.0;

#[derive(Debug, Error, PartialEq)]
pub enum KernelError {
    #[error("times and residuals differ in length ({0} vs {1})")]
    Length(usize, usize),
    #[error("timestamps must be strictly increasing (index {0}); resolve duplicates in the data")]
    NotIncreasing(usize),
    #[error("non-finite input at index {0}")]
    NonFinite(usize),
    #[error("parameters must satisfy sigma_c>0, ell>0, sigma_n>=0 and nu>0")]
    Params,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OuParams {
    pub sigma_c: f64,
    pub ell: f64,
    pub sigma_n: f64,
}

/// Gaussian sufficient statistics of Σ for residuals `e` at `t`: (log|Σ|, eᵀΣ⁻¹e).
pub fn ou_kalman_stats(t: &[f64], e: &[f64], p: OuParams) -> Result<(f64, f64), KernelError> {
    if t.len() != e.len() {
        return Err(KernelError::Length(t.len(), e.len()));
    }
    if !(p.sigma_c > 0.0 && p.ell > 0.0 && p.sigma_n >= 0.0)
        || !p.sigma_c.is_finite()
        || !p.ell.is_finite()
    {
        return Err(KernelError::Params);
    }
    let var_c = p.sigma_c * p.sigma_c;
    let var_n = p.sigma_n * p.sigma_n;
    let (mut m, mut pv) = (0.0_f64, var_c);
    let (mut logdet, mut delta) = (0.0_f64, 0.0_f64);
    for j in 0..t.len() {
        if !t[j].is_finite() || !e[j].is_finite() {
            return Err(KernelError::NonFinite(j));
        }
        if j > 0 {
            let dt = t[j] - t[j - 1];
            if dt <= 0.0 {
                return Err(KernelError::NotIncreasing(j));
            }
            let phi = (-dt / p.ell).exp();
            // 1 - φ² computed as -expm1(-2Δt/ℓ) for close timestamps.
            let q = -libm::expm1(-2.0 * dt / p.ell);
            m *= phi;
            pv = phi * phi * pv + var_c * q;
        }
        let v = e[j] - m;
        let f = pv + var_n;
        if !(f > 0.0) {
            return Err(KernelError::Params);
        }
        logdet += f.ln();
        delta += v * v / f;
        let k = pv / f;
        m += k * v;
        pv -= k * pv;
    }
    Ok((logdet, delta))
}

/// Normalised multivariate Student-t log density from (n, log|Σ|, δ).
pub fn student_t_logpdf(n: usize, logdet: f64, delta: f64, nu: f64) -> f64 {
    let nf = n as f64;
    libm::lgamma((nu + nf) / 2.0)
        - libm::lgamma(nu / 2.0)
        - nf / 2.0 * (nu * std::f64::consts::PI).ln()
        - 0.5 * logdet
        - (nu + nf) / 2.0 * (delta / nu).ln_1p()
}

/// Student-t process log likelihood (ν = [`NU`]) of residuals on irregular times, O(n).
pub fn student_t_ou_loglik(t: &[f64], e: &[f64], p: OuParams) -> Result<f64, KernelError> {
    let (logdet, delta) = ou_kalman_stats(t, e, p)?;
    Ok(student_t_logpdf(t.len(), logdet, delta, NU))
}

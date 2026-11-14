//! Student-t process observation model on irregular times (V3.1 chapter 7).
//!
//! e ~ t_nu(0, Sigma), Sigma_jk = sigma_c^2 exp(-|t_j - t_k| / ell) + sigma_n^2 1[j=k], nu = 4.
//! Sigma is a *scale* matrix (covariance nu Sigma / (nu - 2)). Construction: omega ~ Gamma(nu/2,
//! nu/2) shared by the whole run, e | omega ~ N(0, Sigma / omega).
//!
//! Two evaluations of the same density: dense algebra (reference, O(n^3)) and the OU state-space /
//! Kalman pass (O(n), generic over [`Real`] so it is differentiated on the tape):
//! log|Sigma| = sum log F_j, delta = sum v_j^2 / F_j. Masked entries are skipped, never zero-filled.

use rand::Rng;
use rand_distr::{Distribution, Gamma, StandardNormal};

use crate::ad::Real;

pub const NU_DEFAULT: f64 = 4.0;

fn lgamma(x: f64) -> f64 {
    libm::lgamma(x)
}

/// §7.3: log density of an n-dimensional t_nu(0, Sigma) from log|Sigma| and e' Sigma^-1 e.
pub fn student_t_from_terms<R: Real>(logdet: R, delta: R, n: f64, nu: f64) -> R {
    let c = lgamma(0.5 * (nu + n)) - lgamma(0.5 * nu) - 0.5 * n * (nu * std::f64::consts::PI).ln();
    logdet * (-0.5) - (delta / nu).ln_1p() * (0.5 * (nu + n)) + c
}

/// Filter state after an O(n) pass.
#[derive(Debug, Clone, Copy)]
pub struct Kalman<R> {
    pub logdet: R,
    pub delta: R,
    pub n: usize,
    pub m_last: R,
    pub p_last: R,
    pub t_last: f64,
    pub started: bool,
}

/// O(n) pass (§7.4): x_j = phi_j x_{j-1} + eta_j, phi_j = exp(-dt_j/ell),
/// Var(eta_j) = sigma_c^2 (1 - phi_j^2) computed as -expm1(-2 dt/ell); e_j = x_j + eps_j.
/// Times must be non-decreasing among unmasked points (strictly increasing for a valid density).
pub fn ou_kalman<R: Real>(
    t: &[f64],
    e: &[R],
    sigma_c: R,
    sigma_n: R,
    ell: R,
    mask: &[bool],
) -> Kalman<R> {
    let s2c = sigma_c * sigma_c;
    let s2n = sigma_n * sigma_n;
    let inv_ell = ell.recip();
    let (mut m, mut p) = (R::cst(0.0), s2c);
    let mut tp = 0.0;
    let mut started = false;
    let mut logdet = R::cst(0.0);
    let mut delta = R::cst(0.0);
    let mut n = 0usize;
    for j in 0..t.len() {
        if !mask[j] {
            continue;
        }
        let (m_pred, p_pred) = if started {
            let dt = t[j] - tp;
            let a = inv_ell * (-dt);
            let phi = a.exp();
            let q = s2c * (-(a * 2.0).expm1());
            (phi * m, phi * phi * p + q)
        } else {
            (R::cst(0.0), s2c)
        };
        let f = p_pred + s2n;
        let v = e[j] - m_pred;
        let finv = f.recip();
        logdet += f.ln();
        delta += v * v * finv;
        m = m_pred + p_pred * finv * v;
        p = p_pred * s2n * finv;
        tp = t[j];
        started = true;
        n += 1;
    }
    Kalman {
        logdet,
        delta,
        n,
        m_last: m,
        p_last: p,
        t_last: tp,
        started,
    }
}

pub fn ou_loglik<R: Real>(
    t: &[f64],
    e: &[R],
    sigma_c: R,
    sigma_n: R,
    ell: R,
    nu: f64,
    mask: &[bool],
) -> R {
    let k = ou_kalman(t, e, sigma_c, sigma_n, ell, mask);
    student_t_from_terms(k.logdet, k.delta, k.n as f64, nu)
}

/// Exact conditional of e(t_new) | prefix for t_new >= t_last (§7.5 via the Markov structure).
/// Returns (loc, scale2_latent_x, scale2_obs, dof): the marginal of each future e(t) is
/// t_dof(loc, scale2_obs) with dof = nu + n_A and scale multiplied by (nu + delta_A)/(nu + n_A).
pub fn kalman_predict(
    k: &Kalman<f64>,
    t_new: f64,
    sigma_c: f64,
    sigma_n: f64,
    ell: f64,
    nu: f64,
) -> (f64, f64, f64, f64) {
    let (s2c, s2n) = (sigma_c * sigma_c, sigma_n * sigma_n);
    let dt = (t_new - k.t_last).max(0.0);
    let phi = (-dt / ell).exp();
    let (loc, var_x) = if k.started {
        (
            phi * k.m_last,
            phi * phi * k.p_last + s2c * (-(-2.0 * dt / ell).exp_m1()),
        )
    } else {
        (0.0, s2c)
    };
    let mult = (nu + k.delta) / (nu + k.n as f64);
    (loc, mult * var_x, mult * (var_x + s2n), nu + k.n as f64)
}

// ------------------------------------------------------------------------- dense reference

/// Row-major n x n matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct Mat {
    pub n: usize,
    pub m: usize,
    pub a: Vec<f64>,
}

impl Mat {
    pub fn zeros(n: usize, m: usize) -> Mat {
        Mat {
            n,
            m,
            a: vec![0.0; n * m],
        }
    }
    #[inline]
    pub fn at(&self, i: usize, j: usize) -> f64 {
        self.a[i * self.m + j]
    }
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, v: f64) {
        self.a[i * self.m + j] = v;
    }
    pub fn sub(&self, rows: &[usize], cols: &[usize]) -> Mat {
        let mut o = Mat::zeros(rows.len(), cols.len());
        for (a, &i) in rows.iter().enumerate() {
            for (b, &j) in cols.iter().enumerate() {
                o.set(a, b, self.at(i, j));
            }
        }
        o
    }
}

pub fn exp_kernel(ta: &[f64], tb: &[f64], sigma_c: f64, ell: f64) -> Mat {
    let mut k = Mat::zeros(ta.len(), tb.len());
    for (i, a) in ta.iter().enumerate() {
        for (j, b) in tb.iter().enumerate() {
            k.set(i, j, sigma_c * sigma_c * (-(a - b).abs() / ell).exp());
        }
    }
    k
}

/// Sigma on times t. Masked rows/cols are replaced by the identity (contribute log 1 and 0).
pub fn dense_scale_matrix(
    t: &[f64],
    sigma_c: f64,
    sigma_n: f64,
    ell: f64,
    mask: Option<&[bool]>,
) -> Mat {
    let n = t.len();
    let mut s = exp_kernel(t, t, sigma_c, ell);
    for i in 0..n {
        s.set(i, i, s.at(i, i) + sigma_n * sigma_n);
    }
    if let Some(m) = mask {
        for i in 0..n {
            for j in 0..n {
                if !(m[i] && m[j]) {
                    s.set(i, j, if i == j { 1.0 } else { 0.0 });
                }
            }
        }
    }
    s
}

/// Lower Cholesky factor; Err if not positive definite.
pub fn cholesky(s: &Mat) -> Result<Mat, String> {
    let n = s.n;
    let mut l = Mat::zeros(n, n);
    for j in 0..n {
        let mut d = s.at(j, j);
        for k in 0..j {
            d -= l.at(j, k) * l.at(j, k);
        }
        if !(d > 0.0) {
            return Err(format!("matrix not positive definite at {j}"));
        }
        let dj = d.sqrt();
        l.set(j, j, dj);
        for i in j + 1..n {
            let mut v = s.at(i, j);
            for k in 0..j {
                v -= l.at(i, k) * l.at(j, k);
            }
            l.set(i, j, v / dj);
        }
    }
    Ok(l)
}

/// Solve L z = b (L lower).
pub fn solve_lower(l: &Mat, b: &[f64]) -> Vec<f64> {
    let n = l.n;
    let mut z = b.to_vec();
    for i in 0..n {
        let mut v = z[i];
        for k in 0..i {
            v -= l.at(i, k) * z[k];
        }
        z[i] = v / l.at(i, i);
    }
    z
}

/// Solve L' x = z (L lower).
pub fn solve_upper_t(l: &Mat, z: &[f64]) -> Vec<f64> {
    let n = l.n;
    let mut x = z.to_vec();
    for i in (0..n).rev() {
        let mut v = x[i];
        for k in i + 1..n {
            v -= l.at(k, i) * x[k];
        }
        x[i] = v / l.at(i, i);
    }
    x
}

pub fn log_diag_sum(l: &Mat) -> f64 {
    (0..l.n).map(|i| l.at(i, i).ln()).sum()
}

/// (log|Sigma|, delta, n) by Cholesky of the dense scale matrix.
pub fn dense_terms(
    t: &[f64],
    e: &[f64],
    sigma_c: f64,
    sigma_n: f64,
    ell: f64,
    mask: Option<&[bool]>,
) -> (f64, f64, usize) {
    let s = dense_scale_matrix(t, sigma_c, sigma_n, ell, mask);
    let em: Vec<f64> = e
        .iter()
        .enumerate()
        .map(|(i, &x)| if mask.map_or(true, |m| m[i]) { x } else { 0.0 })
        .collect();
    let l = cholesky(&s).expect("scale matrix is positive definite");
    let z = solve_lower(&l, &em);
    let n = mask.map_or(t.len(), |m| m.iter().filter(|b| **b).count());
    (2.0 * log_diag_sum(&l), z.iter().map(|x| x * x).sum(), n)
}

pub fn dense_loglik(
    t: &[f64],
    e: &[f64],
    sigma_c: f64,
    sigma_n: f64,
    ell: f64,
    nu: f64,
    mask: Option<&[bool]>,
) -> f64 {
    let (ld, d, n) = dense_terms(t, e, sigma_c, sigma_n, ell, mask);
    student_t_from_terms(ld, d, n as f64, nu)
}

/// Result of exact §7.5 conditioning.
#[derive(Debug, Clone)]
pub struct Conditional {
    pub loc: Vec<f64>,
    pub scale: Mat,
    pub dof: f64,
    pub delta_a: f64,
    pub n_a: usize,
}

/// §7.5: e_B | e_A ~ t_{nu+n_A}(S_BA S_AA^-1 e_A, (nu+delta_A)/(nu+n_A) Schur). Masked A entries
/// must already be identity rows in S_AA (see [`dense_scale_matrix`]); their S_BA columns are zeroed.
pub fn conditional_t(
    e_a: &[f64],
    s_aa: &Mat,
    s_ba: &Mat,
    s_bb: &Mat,
    nu: f64,
    mask_a: Option<&[bool]>,
) -> Conditional {
    let keep = |j: usize| mask_a.map_or(true, |m| m[j]);
    let ea: Vec<f64> = e_a
        .iter()
        .enumerate()
        .map(|(j, &x)| if keep(j) { x } else { 0.0 })
        .collect();
    let n_a = (0..e_a.len()).filter(|&j| keep(j)).count();
    let l = cholesky(s_aa).expect("S_AA positive definite");
    let alpha = solve_upper_t(&l, &solve_lower(&l, &ea));
    let delta: f64 = ea.iter().zip(&alpha).map(|(a, b)| a * b).sum();
    let nb = s_ba.n;
    let mut loc = vec![0.0; nb];
    // V = L^-1 S_AB (columns = rows of S_BA)
    let mut v = Mat::zeros(nb, e_a.len());
    for i in 0..nb {
        let row: Vec<f64> = (0..e_a.len())
            .map(|j| if keep(j) { s_ba.at(i, j) } else { 0.0 })
            .collect();
        loc[i] = row.iter().zip(&alpha).map(|(a, b)| a * b).sum();
        let z = solve_lower(&l, &row);
        for (j, zj) in z.into_iter().enumerate() {
            v.set(i, j, zj);
        }
    }
    let mult = (nu + delta) / (nu + n_a as f64);
    let mut scale = Mat::zeros(nb, nb);
    for i in 0..nb {
        for j in 0..nb {
            let vv: f64 = (0..e_a.len()).map(|k| v.at(i, k) * v.at(j, k)).sum();
            scale.set(i, j, mult * (s_bb.at(i, j) - vv));
        }
    }
    Conditional {
        loc,
        scale,
        dof: nu + n_a as f64,
        delta_a: delta,
        n_a,
    }
}

/// Univariate Student-t log density with scale^2 = scale2.
pub fn t_logpdf_1d(x: f64, loc: f64, scale2: f64, dof: f64) -> f64 {
    let z2 = (x - loc) * (x - loc) / scale2;
    lgamma(0.5 * (dof + 1.0))
        - lgamma(0.5 * dof)
        - 0.5 * (dof * std::f64::consts::PI * scale2).ln()
        - 0.5 * (dof + 1.0) * (z2 / dof).ln_1p()
}

/// Multivariate t log density (dense).
pub fn mvt_logpdf(x: &[f64], loc: &[f64], scale: &Mat, dof: f64) -> f64 {
    let l = cholesky(scale).expect("scale positive definite");
    let d: Vec<f64> = x.iter().zip(loc).map(|(a, b)| a - b).collect();
    let z = solve_lower(&l, &d);
    student_t_from_terms(
        2.0 * log_diag_sum(&l),
        z.iter().map(|v| v * v).sum::<f64>(),
        x.len() as f64,
        dof,
    )
}

/// Exact draw via the Gamma–Normal mixture: one omega per run (§7.2). O(n) via the OU recursion.
pub fn sample_student_t_process<G: Rng + ?Sized>(
    rng: &mut G,
    t: &[f64],
    sigma_c: f64,
    sigma_n: f64,
    ell: f64,
    nu: f64,
) -> Vec<f64> {
    let omega: f64 = Gamma::new(0.5 * nu, 2.0 / nu).expect("gamma").sample(rng);
    let s = 1.0 / omega.sqrt();
    let mut x = 0.0;
    let mut out = Vec::with_capacity(t.len());
    for j in 0..t.len() {
        let z: f64 = StandardNormal.sample(rng);
        if j == 0 {
            x = sigma_c * z;
        } else {
            let dt = t[j] - t[j - 1];
            let phi = (-dt / ell).exp();
            x = phi * x + sigma_c * (-(-2.0 * dt / ell).exp_m1()).sqrt() * z;
        }
        let eps: f64 = StandardNormal.sample(rng);
        out.push(s * (x + sigma_n * eps));
    }
    out
}

/// Student-t draw t_dof(0, 1).
pub fn sample_t<G: Rng + ?Sized>(rng: &mut G, dof: f64) -> f64 {
    let z: f64 = StandardNormal.sample(rng);
    let g: f64 = Gamma::new(0.5 * dof, 2.0 / dof).expect("gamma").sample(rng);
    z / g.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NU: f64 = 4.0;
    const SC: f64 = 2e-5;
    const SN: f64 = 1.5e-5;
    const ELL: f64 = 0.3;

    fn data() -> (Vec<f64>, Vec<f64>) {
        let t = vec![0.11, 0.42, 0.9, 1.31, 1.33, 2.0, 2.71, 3.05, 3.9];
        let e = vec![
            1.2e-5, -3.1e-5, 2.2e-5, 0.4e-5, -1.0e-5, 4.4e-5, -2.5e-5, 0.9e-5, 3.0e-5,
        ];
        (t, e)
    }

    /// Joint = marginal + conditional (§7.5), dense algebra.
    #[test]
    fn joint_equals_marginal_plus_conditional() {
        let (t, e) = data();
        let s = dense_scale_matrix(&t, SC, SN, ELL, None);
        let a: Vec<usize> = (0..5).collect();
        let b: Vec<usize> = (5..9).collect();
        let joint = mvt_logpdf(&e, &[0.0; 9], &s, NU);
        let ea: Vec<f64> = a.iter().map(|&i| e[i]).collect();
        let eb: Vec<f64> = b.iter().map(|&i| e[i]).collect();
        let marg = mvt_logpdf(&ea, &[0.0; 5], &s.sub(&a, &a), NU);
        let c = conditional_t(
            &ea,
            &s.sub(&a, &a),
            &s.sub(&b, &a),
            &s.sub(&b, &b),
            NU,
            None,
        );
        let cond = mvt_logpdf(&eb, &c.loc, &c.scale, c.dof);
        assert!(
            (joint - (marg + cond)).abs() < 1e-10,
            "{joint} vs {}",
            marg + cond
        );
    }

    /// OU/Kalman = dense with a mask, and the Markov predictive = dense conditioning.
    #[test]
    fn ou_kalman_equals_dense_with_mask() {
        let (t, e) = data();
        let mut mask = vec![true; 9];
        mask[2] = false;
        mask[6] = false;
        let (ld, q, n) = dense_terms(&t, &e, SC, SN, ELL, Some(&mask));
        let k = ou_kalman(&t, &e, SC, SN, ELL, &mask);
        assert_eq!((k.n, n), (7, 7));
        assert!((k.logdet - ld).abs() < 1e-12 * ld.abs());
        assert!((k.delta - q).abs() < 1e-10 * q.abs());
        let m: Vec<usize> = (0..9).filter(|&i| mask[i]).collect();
        let tm: Vec<f64> = m.iter().map(|&i| t[i]).collect();
        let em: Vec<f64> = m.iter().map(|&i| e[i]).collect();
        let tn = [t[8] + 0.05, t[8] + 0.4];
        let sa = dense_scale_matrix(&tm, SC, SN, ELL, None);
        let sba = exp_kernel(&tn, &tm, SC, ELL);
        let mut sbb = exp_kernel(&tn, &tn, SC, ELL);
        for i in 0..2 {
            sbb.set(i, i, sbb.at(i, i) + SN * SN);
        }
        let c = conditional_t(&em, &sa, &sba, &sbb, NU, None);
        for (i, &tt) in tn.iter().enumerate() {
            let (loc, _, s2y, dof) = kalman_predict(&k, tt, SC, SN, ELL, NU);
            assert!((loc - c.loc[i]).abs() < 1e-9 * c.loc[i].abs() + 1e-15);
            assert!((s2y - c.scale.at(i, i)).abs() < 1e-9 * s2y);
            assert_eq!(dof, NU + 7.0);
        }
    }

    /// Student-t conditioning: a large residual inflates the conditional scale beyond the prior one.
    #[test]
    fn surprising_observation_widens_uncertainty() {
        let s = Mat {
            n: 1,
            m: 1,
            a: vec![2.0],
        };
        let z = Mat {
            n: 1,
            m: 1,
            a: vec![0.0],
        };
        for (ea, wider) in [(0.1, false), (8.0, true)] {
            let c = conditional_t(&[ea], &s, &z, &s, NU, None);
            assert_eq!(c.scale.at(0, 0) > 2.0, wider);
        }
    }
}

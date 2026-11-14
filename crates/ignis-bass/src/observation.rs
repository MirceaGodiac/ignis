//! Calibrated contour observation likelihood (V3.1 §9.7, review-B fix 5).
//!
//! Per frame, the valid (resolved, single-valued) samples are projected on the fixed basis φ_0..φ_7 of
//! ξ by masked least squares, ĉ = G f, G = (Φᵀ M Φ + 10⁻¹² I)⁻¹ Φᵀ M. The model contour, interpolated
//! to the same samples, is projected with the SAME G (so P_t = G_t · Interp is an 8 × K matrix fixed per
//! frame). Under a multivariate Student-t node error with scale Σ_S = σ_c² R(ℓ_s) + σ_n² I (R
//! exponential in ξ, ℓ_s fixed), the 8 coefficients are Student-t (ν = 4) with scale G Σ_S Gᵀ.
//!
//! The relative scale (calibration) error is ONE per run (§7.1: never independent noise on each
//! frame): ĉ_obs,t = (1 + u_t z) ĉ_model,t + G e_t, z ~ N(0, 1), u_t² = u_view,t² + s_rel², and
//!   log ∫ N(z) Π_t t_ν(ĉ_obs,t − (1 + u_t z) ĉ_model,t; G Σ_S Gᵀ) dz
//! is integrated by adaptive Gauss–Hermite (16 nodes centred at the Newton mode, scaled by the
//! curvature; centre and scale carry no gradient). Frames with fewer valid samples than `min_valid` or
//! a non-single-valued status contribute nothing. Given the state and z, frames are independent.

use crate::operator as op;
use ignis_science::gauss_hermite as gh;

pub const NU: f64 = 4.0;
pub const D: usize = op::N_FRONT_COEFFS;
pub const SCALE_NODES: usize = 16;
/// Newton steps of the Python reference (kept for documentation; see `SCALE_NEWTON_MAX`).
pub const SCALE_NEWTON: usize = 12;
/// Iteration cap of the safeguarded mode search (converges in a few steps on regular cases).
pub const SCALE_NEWTON_MAX: usize = 100;

/// Lower Cholesky of an n × n SPD matrix (row-major), in place.
pub fn cholesky(a: &mut [f64], n: usize) -> bool {
    for j in 0..n {
        let mut s = a[j * n + j];
        for k in 0..j {
            s -= a[j * n + k] * a[j * n + k];
        }
        if !(s > 0.0) {
            return false;
        }
        let d = s.sqrt();
        a[j * n + j] = d;
        for i in j + 1..n {
            let mut s = a[i * n + j];
            for k in 0..j {
                s -= a[i * n + k] * a[j * n + k];
            }
            a[i * n + j] = s / d;
        }
        for k in j + 1..n {
            a[j * n + k] = 0.0;
        }
    }
    true
}

pub fn solve_lower(l: &[f64], n: usize, b: &mut [f64]) {
    for i in 0..n {
        let mut s = b[i];
        for k in 0..i {
            s -= l[i * n + k] * b[k];
        }
        b[i] = s / l[i * n + i];
    }
}

pub fn solve_upper_t(l: &[f64], n: usize, b: &mut [f64]) {
    for i in (0..n).rev() {
        let mut s = b[i];
        for k in i + 1..n {
            s -= l[k * n + i] * b[k];
        }
        b[i] = s / l[i * n + i];
    }
}

/// G (8 × S) for one frame; zero columns at unresolved samples.
pub fn projector(xi_obs: &[f64], valid: &[bool]) -> Vec<f64> {
    let s = xi_obs.len();
    let mut phi = op::cosine_basis(xi_obs, D);
    for i in 0..s {
        if !valid[i] {
            phi[i * D..(i + 1) * D].iter_mut().for_each(|v| *v = 0.0);
        }
    }
    let mut gram = vec![0.0; D * D];
    for a in 0..D {
        for b in 0..D {
            gram[a * D + b] = (0..s).map(|i| phi[i * D + a] * phi[i * D + b]).sum::<f64>()
                + if a == b { 1e-12 } else { 0.0 };
        }
    }
    assert!(cholesky(&mut gram, D), "projector Gram matrix not SPD");
    let mut g = vec![0.0; D * s];
    let mut col = [0.0; D];
    for i in 0..s {
        col.copy_from_slice(&phi[i * D..(i + 1) * D]);
        solve_lower(&gram, D, &mut col);
        solve_upper_t(&gram, D, &mut col);
        for a in 0..D {
            g[a * s + i] = col[a];
        }
    }
    g
}

/// Linear interpolation weights of `x` on the K-node grid (jnp.interp): (index, weight) pairs.
pub fn interp_weights(x: f64, k: usize) -> [(usize, f64); 2] {
    let xp = |i: usize| i as f64 / (k - 1) as f64;
    if x <= 0.0 {
        return [(0, 1.0), (1, 0.0)];
    }
    if x >= 1.0 {
        return [(k - 1, 1.0), (k - 2, 0.0)];
    }
    // i = searchsorted(xp, x, 'right') clipped to [1, k−1]
    let mut i0 = ((x * (k - 1) as f64).floor() as usize).min(k - 1);
    while i0 + 1 < k && xp(i0 + 1) <= x {
        i0 += 1;
    }
    while i0 > 0 && xp(i0) > x {
        i0 -= 1;
    }
    let i = (i0 + 1).clamp(1, k - 1);
    let (lo, hi) = (xp(i - 1), xp(i));
    let w = (x - lo) / (hi - lo);
    [(i - 1, 1.0 - w), (i, w)]
}

/// Data-only precomputation of one frame.
#[derive(Debug, Clone)]
pub struct FramePre {
    pub frame: usize,
    /// P = G · Interp (8 × K)
    pub p: Vec<f64>,
    pub c_obs: [f64; D],
    pub a_r: [f64; D * D],
    pub a_i: [f64; D * D],
    pub u_view: f64,
}

/// Precompute the used frames of a run. `f_obs`/`valid` are (T × S), `use_` (T).
pub fn precompute(
    xi_obs: &[f64],
    f_obs: &[f64],
    valid: &[bool],
    use_: &[bool],
    u_view: &[f64],
    k: usize,
    ell_s: f64,
) -> Vec<FramePre> {
    let s = xi_obs.len();
    let mut r = vec![0.0; s * s];
    for i in 0..s {
        for j in 0..s {
            r[i * s + j] = (-(xi_obs[i] - xi_obs[j]).abs() / ell_s).exp();
        }
    }
    let mut out = vec![];
    for t in 0..use_.len() {
        if !use_[t] {
            continue;
        }
        let v = &valid[t * s..(t + 1) * s];
        let g = projector(xi_obs, v);
        let mut p = vec![0.0; D * k];
        let mut c_obs = [0.0; D];
        for i in 0..s {
            let wts = interp_weights(xi_obs[i], k);
            for a in 0..D {
                let gi = g[a * s + i];
                if gi == 0.0 {
                    continue;
                }
                for (j, w) in wts {
                    p[a * k + j] += gi * w;
                }
                if v[i] {
                    c_obs[a] += gi * f_obs[t * s + i];
                }
            }
        }
        let mut gr = vec![0.0; D * s];
        for a in 0..D {
            for j in 0..s {
                gr[a * s + j] = (0..s).map(|i| g[a * s + i] * r[i * s + j]).sum();
            }
        }
        let mut a_r = [0.0; D * D];
        let mut a_i = [0.0; D * D];
        for a in 0..D {
            for b in 0..D {
                a_r[a * D + b] = (0..s).map(|j| gr[a * s + j] * g[b * s + j]).sum();
                a_i[a * D + b] = (0..s).map(|j| g[a * s + j] * g[b * s + j]).sum();
            }
        }
        out.push(FramePre {
            frame: t,
            p,
            c_obs,
            a_r,
            a_i,
            u_view: u_view[t],
        });
    }
    out
}

fn lgamma(x: f64) -> f64 {
    libm::lgamma(x)
}

pub fn mvt_const() -> f64 {
    let d = D as f64;
    lgamma((NU + d) / 2.0) - lgamma(NU / 2.0) - 0.5 * d * (NU * std::f64::consts::PI).ln()
}

/// Observation parameters (log σ_c, log σ_n, log s_rel).
#[derive(Debug, Clone, Copy)]
pub struct ObsParams {
    pub log_sigma_c: f64,
    pub log_sigma_n: f64,
    pub log_scale_rel: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ObsGrad {
    pub log_sigma_c: f64,
    pub log_sigma_n: f64,
    pub log_scale_rel: f64,
}

struct FrameState {
    chol: [f64; D * D],
    logdet: f64,
    c_mod: [f64; D],
    e: [f64; D],
    a: [f64; D],
    u: f64,
}

/// Sequence log-likelihood; with `frames_bar` the adjoints of the frame states (T × K) are added.
pub fn sequence_loglik(
    frames: &[f64],
    k: usize,
    pre: &[FramePre],
    obs: ObsParams,
    mut grad: Option<(&mut [f64], &mut ObsGrad)>,
) -> f64 {
    let sc2 = (2.0 * obs.log_sigma_c).exp();
    let sn2 = (2.0 * obs.log_sigma_n).exp();
    let srel2 = (2.0 * obs.log_scale_rel).exp();
    let cst = mvt_const();
    let nud = NU + D as f64;
    let st: Vec<FrameState> = pre
        .iter()
        .map(|fp| {
            let f = &frames[fp.frame * k..(fp.frame + 1) * k];
            let mut c_mod = [0.0; D];
            for a in 0..D {
                c_mod[a] = (0..k).map(|j| fp.p[a * k + j] * f[j]).sum();
            }
            let mut chol = [0.0; D * D];
            for i in 0..D * D {
                chol[i] = sc2 * fp.a_r[i] + sn2 * fp.a_i[i];
            }
            for i in 0..D {
                chol[i * D + i] += 1e-18;
            }
            assert!(cholesky(&mut chol, D), "frame covariance not SPD");
            let logdet = 2.0 * (0..D).map(|i| chol[i * D + i].ln()).sum::<f64>();
            let u = (fp.u_view * fp.u_view + srel2).sqrt();
            let mut e = [0.0; D];
            for a in 0..D {
                e[a] = fp.c_obs[a] - c_mod[a];
            }
            solve_lower(&chol, D, &mut e);
            let mut aa = c_mod;
            solve_lower(&chol, D, &mut aa);
            aa.iter_mut().for_each(|v| *v *= u);
            FrameState {
                chol,
                logdet,
                c_mod,
                e,
                a: aa,
                u,
            }
        })
        .collect();
    // h(z), h'(z), h''(z)
    let h = |z: f64| -> (f64, f64, f64) {
        let mut v = -0.5 * z * z - 0.5 * (2.0 * std::f64::consts::PI).ln();
        let mut d1 = -z;
        let mut d2 = -1.0;
        for s in &st {
            let mut q = 0.0;
            let mut ae = 0.0;
            let mut a2 = 0.0;
            for i in 0..D {
                let r = s.e[i] - z * s.a[i];
                q += r * r;
                ae += s.a[i] * r;
                a2 += s.a[i] * s.a[i];
            }
            v += cst - 0.5 * s.logdet - 0.5 * nud * (q / NU).ln_1p();
            let q1 = -2.0 * ae;
            let q2 = 2.0 * a2;
            d1 += -0.5 * nud * q1 / (NU + q);
            d2 += -0.5 * nud * (q2 / (NU + q) - q1 * q1 / ((NU + q) * (NU + q)));
        }
        (v, d1, d2)
    };
    let num: f64 = st
        .iter()
        .map(|s| (0..D).map(|i| s.a[i] * s.e[i]).sum::<f64>())
        .sum();
    let den: f64 = 1.0
        + st.iter()
            .map(|s| (0..D).map(|i| s.a[i] * s.a[i]).sum::<f64>())
            .sum::<f64>();
    // Mode of h: the damped Newton step of the reference (z += h'/max(−h'', 1)) with backtracking so
    // that h never decreases, iterated to convergence (the reference's fixed 12 steps can stop far
    // from the mode under heavy-tailed misfit, which makes the stop-gradient centre matter).
    let mut zs = num / den;
    let (mut hv, mut d1, mut d2) = h(zs);
    for _ in 0..SCALE_NEWTON_MAX {
        if d1.abs() <= 1e-10 * (1.0 + hv.abs()) {
            break;
        }
        let mut step = d1 / (-d2).max(1.0);
        let mut accepted = false;
        for _ in 0..60 {
            let (v, a, b) = h(zs + step);
            if v >= hv {
                zs += step;
                (hv, d1, d2) = (v, a, b);
                accepted = true;
                break;
            }
            step *= 0.5;
        }
        if !accepted {
            break;
        }
    }
    let sd = 1.0 / (-d2).max(1.0).sqrt();
    let (x, w) = gh::hermite_rule(SCALE_NODES);
    let zn: Vec<f64> = x.iter().map(|xi| zs + 2f64.sqrt() * sd * xi).collect();
    let terms: Vec<f64> = (0..SCALE_NODES)
        .map(|i| w[i].ln() + x[i] * x[i] + h(zn[i]).0)
        .collect();
    let lse = gh::logsumexp(&terms);
    let ll = lse + (2f64.sqrt() * sd).ln();
    let Some((fbar, og)) = grad.as_mut() else {
        return ll;
    };
    let omega: Vec<f64> = terms.iter().map(|t| (t - lse).exp()).collect();
    for (fp, s) in pre.iter().zip(&st) {
        let mut cbar = [0.0; D];
        let mut ubar = 0.0;
        let mut sbar = [0.0; D * D];
        // −½ Σ⁻¹ (Σ ω = 1): Σ⁻¹ columns via solves
        let mut sinv = [0.0; D * D];
        for j in 0..D {
            let mut col = [0.0; D];
            col[j] = 1.0;
            solve_lower(&s.chol, D, &mut col);
            solve_upper_t(&s.chol, D, &mut col);
            for i in 0..D {
                sinv[i * D + j] = col[i];
            }
        }
        for i in 0..D * D {
            sbar[i] = -0.5 * sinv[i];
        }
        for n in 0..SCALE_NODES {
            let z = zn[n];
            let om = omega[n];
            if om == 0.0 {
                continue;
            }
            let mut r = [0.0; D];
            for i in 0..D {
                r[i] = fp.c_obs[i] - (1.0 + s.u * z) * s.c_mod[i];
            }
            let mut sr = r;
            solve_lower(&s.chol, D, &mut sr);
            let q: f64 = sr.iter().map(|v| v * v).sum();
            solve_upper_t(&s.chol, D, &mut sr); // Σ⁻¹ r
            let beta = nud / (NU + q);
            for i in 0..D {
                cbar[i] += om * (1.0 + s.u * z) * beta * sr[i];
            }
            ubar += om * z * beta * (0..D).map(|i| s.c_mod[i] * sr[i]).sum::<f64>();
            for i in 0..D {
                for j in 0..D {
                    sbar[i * D + j] += om * 0.5 * beta * sr[i] * sr[j];
                }
            }
        }
        let tr_r: f64 = (0..D * D).map(|i| sbar[i] * fp.a_r[i]).sum();
        let tr_i: f64 = (0..D * D).map(|i| sbar[i] * fp.a_i[i]).sum();
        og.log_sigma_c += 2.0 * sc2 * tr_r;
        og.log_sigma_n += 2.0 * sn2 * tr_i;
        og.log_scale_rel += ubar * srel2 / s.u;
        let fb = &mut fbar[fp.frame * k..(fp.frame + 1) * k];
        for a in 0..D {
            if cbar[a] == 0.0 {
                continue;
            }
            for j in 0..k {
                fb[j] += fp.p[a * k + j] * cbar[a];
            }
        }
    }
    ll
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_loglik_gradient_matches_finite_differences() {
        let (nt, s, k) = (5, 30, 32);
        let xi: Vec<f64> = (0..s).map(|i| i as f64 / (s - 1) as f64).collect();
        let mut frames = vec![0.0; nt * k];
        let mut f_obs = vec![0.0; nt * s];
        let mut valid = vec![true; nt * s];
        for t in 0..nt {
            for j in 0..k {
                frames[t * k + j] =
                    0.02 + 0.001 * t as f64 + 0.002 * (3.0 * j as f64 / k as f64).cos();
            }
            for i in 0..s {
                f_obs[t * s + i] = 0.02
                    + 0.001 * t as f64
                    + 0.002 * (3.0 * xi[i]).cos() * 1.01
                    + 4e-5 * ((t * 7 + i * 3) % 5) as f64;
                valid[t * s + i] = (t + i) % 7 != 0;
            }
        }
        let use_ = vec![true, true, false, true, true];
        let pre = precompute(&xi, &f_obs, &valid, &use_, &vec![0.02; nt], k, 0.1);
        let o = ObsParams {
            log_sigma_c: 1e-4f64.ln(),
            log_sigma_n: 5e-5f64.ln(),
            log_scale_rel: 1e-2f64.ln(),
        };
        let mut fb = vec![0.0; nt * k];
        let mut og = ObsGrad::default();
        let v0 = sequence_loglik(&frames, k, &pre, o, Some((&mut fb, &mut og)));
        let h = 1e-9;
        for &idx in &[0usize, 5, 40, 100, 140] {
            let mut a = frames.clone();
            let mut b = frames.clone();
            a[idx] += h;
            b[idx] -= h;
            let fd = (sequence_loglik(&a, k, &pre, o, None)
                - sequence_loglik(&b, k, &pre, o, None))
                / (2.0 * h);
            assert!(
                (fd - fb[idx]).abs() < 1e-4 * fb[idx].abs().max(1.0),
                "frame {idx}: fd {fd} an {}",
                fb[idx]
            );
        }
        let hh = 1e-6;
        for (which, an) in [
            (0, og.log_sigma_c),
            (1, og.log_sigma_n),
            (2, og.log_scale_rel),
        ] {
            let pert = |d: f64| {
                let mut q = o;
                match which {
                    0 => q.log_sigma_c += d,
                    1 => q.log_sigma_n += d,
                    _ => q.log_scale_rel += d,
                }
                sequence_loglik(&frames, k, &pre, q, None)
            };
            let fd = (pert(hh) - pert(-hh)) / (2.0 * hh);
            assert!(
                (fd - an).abs() < 1e-5 * an.abs().max(1.0),
                "obs {which}: fd {fd} an {an}"
            );
        }
        assert!(v0.is_finite());
    }

    /// Review-B fix 5 reference: ONE relative scale error z ~ N(0,1) shared by all frames, the z
    /// integral by dense quadrature (independent of the adaptive Gauss–Hermite path).
    #[test]
    fn scale_error_is_common_to_the_run() {
        let (nt, s, k, ell) = (6usize, 24usize, 32usize, 0.1);
        let xi: Vec<f64> = (0..s).map(|i| i as f64 / (s - 1) as f64).collect();
        let mut frames = vec![0.0; nt * k];
        let mut fo = vec![0.0; nt * s];
        let mut valid = vec![true; nt * s];
        let mut seed = 9u64;
        let mut noise = || {
            let mut a = 0.0;
            for _ in 0..12 {
                seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                a += (seed >> 11) as f64 / (1u64 << 53) as f64;
            }
            a - 6.0
        };
        for t in 0..nt {
            for j in 0..k {
                frames[t * k + j] = 0.03
                    + 0.002 * t as f64
                    + 0.01 * (std::f64::consts::PI * j as f64 / (k - 1) as f64).cos();
            }
            for i in 0..s {
                let w = interp_weights(xi[i], k);
                let fm = w[0].1 * frames[t * k + w[0].0] + w[1].1 * frames[t * k + w[1].0];
                fo[t * s + i] = 1.03 * fm + 1e-4 * noise();
                valid[t * s + i] = (t * 5 + i * 3) % 7 != 0;
            }
        }
        let mut use_ = vec![true; nt];
        use_[2] = false;
        let u_view = vec![0.02; nt];
        let o = ObsParams {
            log_sigma_c: 1e-4f64.ln(),
            log_sigma_n: 5e-5f64.ln(),
            log_scale_rel: 1e-2f64.ln(),
        };
        let pre = precompute(&xi, &fo, &valid, &use_, &u_view, k, ell);
        let code = sequence_loglik(&frames, k, &pre, o, None);
        // dense reference
        let (sc2, sn2, sr2) = (
            (2.0 * o.log_sigma_c).exp(),
            (2.0 * o.log_sigma_n).exp(),
            (2.0 * o.log_scale_rel).exp(),
        );
        let cst = mvt_const();
        let ll = |z: f64| -> f64 {
            let mut tot = 0.0;
            for fp in &pre {
                let f = &frames[fp.frame * k..(fp.frame + 1) * k];
                let cm: Vec<f64> = (0..D)
                    .map(|a| (0..k).map(|j| fp.p[a * k + j] * f[j]).sum())
                    .collect();
                let mut ch = [0.0; D * D];
                for i in 0..D * D {
                    ch[i] = sc2 * fp.a_r[i] + sn2 * fp.a_i[i];
                }
                for i in 0..D {
                    ch[i * D + i] += 1e-18;
                }
                cholesky(&mut ch, D);
                let u = (fp.u_view * fp.u_view + sr2).sqrt();
                let mut r: Vec<f64> = (0..D)
                    .map(|a| fp.c_obs[a] - (1.0 + u * z) * cm[a])
                    .collect();
                solve_lower(&ch, D, &mut r);
                let q: f64 = r.iter().map(|v| v * v).sum();
                let logdet = 2.0 * (0..D).map(|i| ch[i * D + i].ln()).sum::<f64>();
                tot += cst - 0.5 * logdet - 0.5 * (NU + D as f64) * (q / NU).ln_1p();
            }
            tot - 0.5 * z * z - 0.5 * (2.0 * std::f64::consts::PI).ln()
        };
        let n = 200_001;
        let (a, b) = (-12.0, 12.0);
        let hstep = (b - a) / (n - 1) as f64;
        let vals: Vec<f64> = (0..n)
            .map(|i| ll(a + i as f64 * hstep) + hstep.ln())
            .collect();
        let reference = gh::logsumexp(&vals);
        assert!(
            (code - reference).abs() < 1e-6,
            "adaptive GH {code} vs dense {reference}"
        );
    }
}

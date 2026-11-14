//! FLEX posterior densities: training joint density and prefix-conditioned local targets.
//!
//! Training (§5.6, §12.3): log p(Theta) + sum_i [log p(b_i, D0_i | Theta) + log L_i^obs + log L_i^event],
//! full batch, non-centred locals, FP64. Runs are evaluated in parallel, one tape per run.
//!
//! Prefix target (§10.2), per global draw Theta_g:
//! pi_tau(v | Theta_g) ∝ p0(v | Theta_g) L(Y_<=tau | Theta_g, v) S(tau | Theta_g, v) 1[support],
//! v = (eps_b, log D0). Serving path: importance sampling with a defensive mixture proposal
//! q = alpha p0 + (1 - alpha) N(Laplace mode, inflated covariance) per draw; weights normalised
//! jointly over (g, m) (§10.1). The Laplace Hessian uses central differences of exact gradients.

use std::f64::consts::PI;

use rand::{RngExt, SeedableRng};
use rand_distr::{Distribution, StandardNormal};
use rayon::prelude::*;

use ignis_core::registry::{Tensor, TensorMap};
use ignis_science::ad::{self, Real, Var};
use ignis_science::nuts::LogDensity;
use ignis_science::transforms::{Layout, ParamSpec};

use crate::dynamics::{natural_globals, run_loglik, Model, RunData};
use crate::structure::{Priors, D_REF_M, K_REF_M2_S};

// ------------------------------------------------------------------------- tensors <-> flat

pub fn flat_from_tensors(layout: &Layout, t: &TensorMap) -> anyhow::Result<Vec<f64>> {
    let mut x = vec![0.0; layout.size];
    for (s, &o) in layout.specs.iter().zip(&layout.offsets) {
        let v = t
            .get(&s.name)
            .ok_or_else(|| anyhow::anyhow!("tensor {} missing", s.name))?;
        anyhow::ensure!(
            v.data.len() == s.size(),
            "tensor {} has {} values, expected {}",
            s.name,
            v.data.len(),
            s.size()
        );
        x[o..o + s.size()].copy_from_slice(&v.data);
    }
    Ok(x)
}

pub fn tensors_from_flat(layout: &Layout, x: &[f64], out: &mut TensorMap) {
    for (s, &o) in layout.specs.iter().zip(&layout.offsets) {
        out.insert(
            s.name.clone(),
            Tensor::new(s.shape.clone(), x[o..o + s.size()].to_vec()),
        );
    }
}

/// Stack draws (D, size) of every block into tensors with a leading draw axis.
pub fn stacked_tensors(layout: &Layout, draws: &[Vec<f64>], out: &mut TensorMap) {
    for (s, &o) in layout.specs.iter().zip(&layout.offsets) {
        let mut data = Vec::with_capacity(draws.len() * s.size());
        for d in draws {
            data.extend_from_slice(&d[o..o + s.size()]);
        }
        let mut shape = vec![draws.len()];
        shape.extend(&s.shape);
        out.insert(s.name.clone(), Tensor::new(shape, data));
    }
}

// ------------------------------------------------------------------------- joint density

/// Flat joint vector: [globals, eps_b (N x n_b, row-major), log_d0 (N)].
pub struct Joint<'a> {
    pub m: &'a Model,
    pub runs: &'a [RunData],
    pub pr: &'a Priors,
}

impl Joint<'_> {
    pub fn n(&self) -> usize {
        self.runs.len()
    }
    pub fn dim(&self) -> usize {
        self.m.n_g() + self.n() * (self.m.n_b() + 1)
    }
    pub fn local_index(&self, i: usize, k: usize) -> usize {
        let (ng, nb, n) = (self.m.n_g(), self.m.n_b(), self.n());
        if k < nb {
            ng + i * nb + k
        } else {
            ng + n * nb + i
        }
    }
    fn locals(&self, x: &[f64], i: usize) -> Vec<f64> {
        (0..=self.m.n_b())
            .map(|k| x[self.local_index(i, k)])
            .collect()
    }
    fn local_prior(&self, v: &[f64], g: Option<&mut [f64]>) -> f64 {
        let nb = self.m.n_b();
        let (mu, sd) = (self.pr["log_d0_loc"], self.pr["log_d0_sd"]);
        let mut lp = 0.0;
        for k in 0..nb {
            lp += -0.5 * v[k] * v[k] - 0.5 * (2.0 * PI).ln();
        }
        let z = (v[nb] - mu) / sd;
        lp += -0.5 * z * z - sd.ln() - 0.5 * (2.0 * PI).ln();
        if let Some(g) = g {
            for k in 0..nb {
                g[k] = -v[k];
            }
            g[nb] = -z / sd;
        }
        lp
    }

    /// Per-run log likelihood values (no priors).
    pub fn per_run(&self, x: &[f64]) -> Vec<f64> {
        let ng = self.m.n_g();
        let nb = self.m.n_b();
        (0..self.n())
            .into_par_iter()
            .map(|i| {
                let v = self.locals(x, i);
                run_loglik(self.m, &x[..ng], &self.runs[i], &v[..nb], v[nb])
            })
            .collect()
    }

    pub fn logp(&self, x: &[f64]) -> f64 {
        let ng = self.m.n_g();
        let lp = self.m.layout.log_prior(&x[..ng]);
        let ll: f64 = self.per_run(x).iter().sum();
        let lpl: f64 = (0..self.n())
            .map(|i| self.local_prior(&self.locals(x, i), None))
            .sum();
        lp + lpl + ll
    }

    /// Value and full gradient; runs in parallel, one tape session per run.
    pub fn value_grad(&self, x: &[f64], grad: &mut [f64]) -> f64 {
        let ng = self.m.n_g();
        let nb = self.m.n_b();
        let (lp, gp) = ad::value_and_grad(&x[..ng], |p: &[Var]| self.m.layout.log_prior(p));
        let per: Vec<(f64, Vec<f64>)> = (0..self.n())
            .into_par_iter()
            .map(|i| {
                let mut z = x[..ng].to_vec();
                z.extend(self.locals(x, i));
                ad::value_and_grad(&z, |zs: &[Var]| {
                    run_loglik(
                        self.m,
                        &zs[..ng],
                        &self.runs[i],
                        &zs[ng..ng + nb],
                        zs[ng + nb],
                    )
                })
            })
            .collect();
        grad.iter_mut().for_each(|g| *g = 0.0);
        grad[..ng].copy_from_slice(&gp);
        let mut total = lp;
        let mut gl = vec![0.0; nb + 1];
        for (i, (v, g)) in per.iter().enumerate() {
            total += v;
            for k in 0..ng {
                grad[k] += g[k];
            }
            let lv = self.locals(x, i);
            total += self.local_prior(&lv, Some(&mut gl));
            for k in 0..=nb {
                grad[self.local_index(i, k)] += g[ng + k] + gl[k];
            }
        }
        if !total.is_finite() {
            return f64::NEG_INFINITY;
        }
        total
    }

    /// Probes for the FD Hessian: every global alone, then local k of all runs together.
    pub fn hessian_probes(&self) -> Vec<Vec<usize>> {
        let mut p: Vec<Vec<usize>> = (0..self.m.n_g()).map(|j| vec![j]).collect();
        for k in 0..=self.m.n_b() {
            p.push((0..self.n()).map(|i| self.local_index(i, k)).collect());
        }
        p
    }

    /// Which perturbed coordinate a gradient entry `i` responds to under a probe.
    pub fn attribute(&self, i: usize, probe: &[usize]) -> Option<usize> {
        let ng = self.m.n_g();
        if probe.len() == 1 {
            // global probe: every coordinate's response is attributable
            return Some(probe[0]);
        }
        if i < ng {
            return None; // mixed over runs; recovered by symmetry from the global probes
        }
        let run = self.run_of(i);
        probe.iter().copied().find(|&j| self.run_of(j) == run)
    }

    fn run_of(&self, idx: usize) -> usize {
        let (ng, nb, n) = (self.m.n_g(), self.m.n_b(), self.n());
        if idx < ng + n * nb {
            (idx - ng) / nb.max(1)
        } else {
            idx - ng - n * nb
        }
    }
}

impl LogDensity for Joint<'_> {
    fn dim(&self) -> usize {
        Joint::dim(self)
    }
    fn logp_grad(&self, x: &[f64], grad: &mut [f64]) -> f64 {
        self.value_grad(x, grad)
    }
}

/// The same joint density with *centred* random effects: the local block holds b_i = L_b eps_i
/// instead of eps_i. Exact reparameterisation: p(eps(b)) |d eps/d b|, log|d eps/d b| = -log det L_b
/// per run. When every droplet pins its own offsets (hundreds of points per run) the non-centred
/// coordinates lie on curved ridges eps_i = L_b^-1 b_i along the scale parameters that no linear
/// metric straightens (measured: NUTS tree depth saturated); centred coordinates are close to
/// Gaussian there. The centred joint has no finite mode (sigma_b -> 0 funnel), so the Laplace
/// metric is taken at the non-centred MAP mapped into these coordinates, never re-optimised here.
pub struct Centered<'a> {
    pub j: &'a Joint<'a>,
}

fn l_b<R: Real>(m: &Model, g: &[R]) -> [[R; 2]; 2] {
    natural_globals(m, g).l_b
}

impl Centered<'_> {
    /// eps-coordinates -> b-coordinates (same layout).
    pub fn from_eps(&self, x: &[f64]) -> Vec<f64> {
        let (m, ng, nb) = (self.j.m, self.j.m.n_g(), self.j.m.n_b());
        let l = l_b(m, &x[..ng]);
        let mut y = x.to_vec();
        for i in 0..self.j.n() {
            let e: Vec<f64> = (0..nb).map(|k| x[self.j.local_index(i, k)]).collect();
            if nb >= 1 {
                y[self.j.local_index(i, 0)] = l[0][0] * e[0];
            }
            if nb == 2 {
                y[self.j.local_index(i, 1)] = l[1][0] * e[0] + l[1][1] * e[1];
            }
        }
        y
    }
    /// b-coordinates -> eps-coordinates.
    pub fn to_eps(&self, y: &[f64]) -> Vec<f64> {
        let (m, ng, nb) = (self.j.m, self.j.m.n_g(), self.j.m.n_b());
        let l = l_b(m, &y[..ng]);
        let mut x = y.to_vec();
        for i in 0..self.j.n() {
            let b: Vec<f64> = (0..nb).map(|k| y[self.j.local_index(i, k)]).collect();
            if nb >= 1 {
                let e0 = b[0] / l[0][0];
                x[self.j.local_index(i, 0)] = e0;
                if nb == 2 {
                    x[self.j.local_index(i, 1)] = (b[1] - l[1][0] * e0) / l[1][1];
                }
            }
        }
        x
    }
    pub fn value_grad(&self, y: &[f64], grad: &mut [f64]) -> f64 {
        let j = self.j;
        let (m, ng, nb) = (j.m, j.m.n_g(), j.m.n_b());
        let (lp, gp) = ad::value_and_grad(&y[..ng], |p: &[Var]| m.layout.log_prior(p));
        let per: Vec<(f64, Vec<f64>)> = (0..j.n())
            .into_par_iter()
            .map(|i| {
                let mut z = y[..ng].to_vec();
                z.extend(j.locals(y, i));
                ad::value_and_grad(&z, |zs: &[Var]| {
                    let g = &zs[..ng];
                    let b = &zs[ng..ng + nb];
                    let l = l_b(m, g);
                    let mut eps: Vec<Var> = Vec::with_capacity(nb);
                    let mut jac = Var::cst(0.0);
                    if nb >= 1 {
                        eps.push(b[0] / l[0][0]);
                        jac -= l[0][0].ln();
                    }
                    if nb == 2 {
                        eps.push((b[1] - l[1][0] * eps[0]) / l[1][1]);
                        jac -= l[1][1].ln();
                    }
                    let mut v = eps.clone();
                    v.push(zs[ng + nb]);
                    local_prior_logpdf(nb, &v, j.pr)
                        + jac
                        + run_loglik(m, g, &j.runs[i], &eps, zs[ng + nb])
                })
            })
            .collect();
        grad.iter_mut().for_each(|g| *g = 0.0);
        grad[..ng].copy_from_slice(&gp);
        let mut total = lp;
        for (i, (v, g)) in per.iter().enumerate() {
            total += v;
            for k in 0..ng {
                grad[k] += g[k];
            }
            for k in 0..=nb {
                grad[j.local_index(i, k)] += g[ng + k];
            }
        }
        if total.is_finite() {
            total
        } else {
            f64::NEG_INFINITY
        }
    }
}

impl LogDensity for Centered<'_> {
    fn dim(&self) -> usize {
        self.j.dim()
    }
    fn logp_grad(&self, x: &[f64], grad: &mut [f64]) -> f64 {
        self.value_grad(x, grad)
    }
}

/// Least-squares q = q0 - K t on valid points (init / proposal centring only).
pub fn prefix_d2_estimate(run: &RunData) -> (Option<f64>, Option<f64>) {
    let idx: Vec<usize> = (0..run.t.len()).filter(|&i| run.mask[i]).collect();
    if idx.len() < 2 {
        return (None, idx.first().map(|&i| run.y[i] * run.y[i]));
    }
    let n = idx.len() as f64;
    let (mut st, mut sq, mut stt, mut stq) = (0.0, 0.0, 0.0, 0.0);
    for &i in &idx {
        let (t, q) = (run.t[i], run.y[i] * run.y[i]);
        st += t;
        sq += q;
        stt += t * t;
        stq += t * q;
    }
    let den = n * stt - st * st;
    let slope = (n * stq - st * sq) / den;
    let q0 = (sq - slope * st) / n;
    let k = -slope;
    let q0 = q0.max(run.y[idx[0]] * run.y[idx[0]]);
    (if k > 0.0 { Some(k) } else { None }, Some(q0))
}

fn nanmedian(v: &[f64]) -> Option<f64> {
    let mut f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if f.is_empty() {
        return None;
    }
    f.sort_by(f64::total_cmp);
    let k = f.len();
    Some(if k % 2 == 1 {
        f[k / 2]
    } else {
        0.5 * (f[k / 2 - 1] + f[k / 2])
    })
}

/// Deterministic, in-support initialisation (prior means; locals from per-run D^2 fits; NN weights
/// from the prior, away from the all-zero point §6.7).
pub fn init_params(m: &Model, runs: &[RunData], pr: &Priors, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let est: Vec<(Option<f64>, Option<f64>)> = runs.iter().map(prefix_d2_estimate).collect();
    let logk: Vec<f64> = est
        .iter()
        .map(|(k, _)| k.map_or(f64::NAN, |k| (k / K_REF_M2_S).ln()))
        .collect();
    let beta0 = nanmedian(&logk).unwrap_or(pr["beta0_loc"]);
    let mut x = Vec::with_capacity(m.n_g() + runs.len() * (m.n_b() + 1));
    for s in &m.layout.specs {
        let n = s.size();
        let v: Vec<f64> = match s.name.as_str() {
            "beta" => (0..n).map(|i| if i == 0 { beta0 } else { 0.0 }).collect(),
            "b.log_sigma" => vec![0.3f64.ln(); n],
            "hazard.gamma0" => vec![pr["gamma0_loc"]],
            "hazard.zeta" => vec![pr["zeta_loc"]],
            "obs.log_sigma_c" | "obs.log_sigma_n" => vec![pr["log_sigma_loc"]],
            "obs.log_ell" => vec![pr["log_ell_loc"]],
            nm if nm.starts_with("residual.W") => (0..n)
                .map(|i| {
                    let z: f64 = StandardNormal.sample(&mut rng);
                    z * s.prior.scale(i)
                })
                .collect(),
            _ => vec![0.0; n],
        };
        x.extend(v);
    }
    let lin0 = if m.st.z_mode == "full" {
        beta0
    } else {
        m.st.beta0_fixed
    };
    let sig0 = if m.st.b0 == "hier" {
        0.3
    } else {
        m.st.b0_fixed_scale
    };
    let nb = m.n_b();
    for (i, _) in runs.iter().enumerate() {
        for k in 0..nb {
            x.push(if k == 0 {
                let e = if logk[i].is_finite() {
                    (logk[i] - lin0) / sig0
                } else {
                    0.0
                };
                e.clamp(-3.0, 3.0) - 0.3 // slightly slower: in support
            } else {
                0.0
            });
        }
    }
    for (_, q0) in &est {
        let d0 = q0.map_or(pr["log_d0_loc"].exp(), f64::sqrt);
        x.push((d0 * 1.02).ln());
    }
    x
}

/// Move runs whose initial likelihood is -inf (observation/event outside support) to slower rates.
pub fn repair_support(j: &Joint<'_>, x: &mut [f64], max_iter: usize) -> usize {
    let mut moved = 0;
    for _ in 0..max_iter {
        let ll = j.per_run(x);
        let bad: Vec<usize> = (0..ll.len()).filter(|&i| !ll[i].is_finite()).collect();
        if bad.is_empty() {
            break;
        }
        moved += bad.len();
        for i in bad {
            if j.m.n_b() > 0 {
                x[j.local_index(i, 0)] -= 0.5;
            }
            x[j.local_index(i, j.m.n_b())] += 0.02;
        }
    }
    moved
}

// ------------------------------------------------------------------------- prefix-conditioned locals

pub fn local_prior_logpdf<R: Real>(nb: usize, v: &[R], pr: &Priors) -> R {
    let mut lp = R::cst(-0.5 * nb as f64 * (2.0 * PI).ln());
    for k in 0..nb {
        lp -= v[k] * v[k] * 0.5;
    }
    let sd = pr["log_d0_sd"];
    let z = (v[nb] - pr["log_d0_loc"]) / sd;
    lp - z * z * 0.5 - (sd.ln() + 0.5 * (2.0 * PI).ln())
}

/// log target of locals v for one run under one global draw p.
pub fn local_target<R: Real>(m: &Model, pr: &Priors, p: &[R], v: &[R], run: &RunData) -> R {
    let nb = m.n_b();
    local_prior_logpdf(nb, v, pr) + run_loglik(m, p, run, &v[..nb], v[nb])
}

fn local_vg(m: &Model, pr: &Priors, p: &[f64], v: &[f64], run: &RunData) -> (f64, Vec<f64>) {
    let pc: Vec<Var> = p.iter().map(|&x| Var::cst(x)).collect();
    ad::value_and_grad(v, |vs: &[Var]| local_target(m, pr, &pc, vs, run))
}

/// Symmetric eigen-decomposition (cyclic Jacobi) of a small d x d matrix: (values, U columns).
pub fn eigh(a: &[Vec<f64>]) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = a.len();
    let mut a: Vec<Vec<f64>> = a.to_vec();
    let mut u: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    for _ in 0..100 {
        let off: f64 = (0..n)
            .flat_map(|i| (0..n).filter(move |&j| j != i).map(move |j| (i, j)))
            .map(|(i, j)| a[i][j] * a[i][j])
            .sum();
        if off < 1e-30 * (0..n).map(|i| a[i][i] * a[i][i]).sum::<f64>().max(1e-300) {
            break;
        }
        for p in 0..n {
            for q in p + 1..n {
                if a[p][q] == 0.0 {
                    continue;
                }
                let th = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = th.signum() / (th.abs() + (th * th + 1.0).sqrt());
                let t = if th == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..n {
                    let (akp, akq) = (a[k][p], a[k][q]);
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let (apk, aqk) = (a[p][k], a[q][k]);
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
                for k in 0..n {
                    let (ukp, ukq) = (u[k][p], u[k][q]);
                    u[k][p] = c * ukp - s * ukq;
                    u[k][q] = s * ukp + c * ukq;
                }
            }
        }
    }
    ((0..n).map(|i| a[i][i]).collect(), u)
}

/// Damped saddle-free Newton with a 7-point line search on v -> logt(p, v). The covariance carried
/// out is the inverse negative Hessian at the last probed iterate. Returns (mode, cov, ok).
pub fn newton_mode(
    m: &Model,
    pr: &Priors,
    p: &[f64],
    run: &RunData,
    v0: &[f64],
    iters: usize,
) -> (Vec<f64>, Vec<Vec<f64>>, bool) {
    let d = v0.len();
    let steps = [1.0, 0.5, 0.25, 0.1, 0.03, 0.01, 0.001];
    let mut v = v0.to_vec();
    let mut cov: Vec<Vec<f64>> = (0..d)
        .map(|i| (0..d).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    let mut ok = false;
    let pc: Vec<f64> = p.to_vec();
    let logt = |x: &[f64]| local_target(m, pr, &pc, x, run);
    for _ in 0..iters {
        let (val, g) = local_vg(m, pr, p, &v, run);
        let h: Vec<f64> = v.iter().map(|x| 1e-4 * x.abs().max(1.0)).collect();
        let mut gm = vec![vec![0.0; d]; d];
        for i in 0..d {
            let mut a = v.clone();
            a[i] += h[i];
            let (_, ga) = local_vg(m, pr, p, &a, run);
            a[i] = v[i] - h[i];
            let (_, gb) = local_vg(m, pr, p, &a, run);
            for j in 0..d {
                gm[i][j] = (ga[j] - gb[j]) / (2.0 * h[i]);
            }
        }
        let hm: Vec<Vec<f64>> = (0..d)
            .map(|i| (0..d).map(|j| -0.5 * (gm[i][j] + gm[j][i])).collect())
            .collect();
        let (ev, u) = eigh(&hm);
        ok = ev.iter().all(|e| *e > 0.0) && val.is_finite() && ev.iter().all(|e| e.is_finite());
        cov = (0..d)
            .map(|i| {
                (0..d)
                    .map(|j| {
                        (0..d)
                            .map(|k| u[i][k] * u[j][k] / if ev[k] > 0.0 { ev[k] } else { 1.0 })
                            .sum()
                    })
                    .collect()
            })
            .collect();
        let evf: Vec<f64> = ev.iter().map(|e| e.abs().max(0.5)).collect();
        let ug: Vec<f64> = (0..d)
            .map(|k| (0..d).map(|i| u[i][k] * g[i]).sum::<f64>() / evf[k])
            .collect();
        let mut step: Vec<f64> = (0..d)
            .map(|i| (0..d).map(|k| u[i][k] * ug[k]).sum())
            .collect();
        if step.iter().any(|s| !s.is_finite()) {
            step = vec![0.0; d];
        }
        let mut best = (f64::NEG_INFINITY, 0usize);
        let cands: Vec<Vec<f64>> = steps
            .iter()
            .map(|s| v.iter().zip(&step).map(|(a, b)| a + s * b).collect())
            .collect();
        for (k, c) in cands.iter().enumerate() {
            let lv = logt(c);
            let lv = if lv.is_finite() {
                lv
            } else {
                f64::NEG_INFINITY
            };
            if lv > best.0 {
                best = (lv, k);
            }
        }
        let prev = v.clone();
        if best.0 > val || !val.is_finite() {
            v = cands[best.1].clone();
        }
        if v == prev && val.is_finite() {
            // fixed point: further iterations repeat the same probe exactly
            break;
        }
    }
    (v, cov, ok)
}

/// Starting v for one draw from the prefix D^2 estimate.
pub fn local_init(
    m: &Model,
    p: &[f64],
    ck: &[f64; 4],
    k_hat: Option<f64>,
    q0_hat: f64,
) -> Vec<f64> {
    let logd0 = 0.5 * q0_hat.ln() + 0.01;
    let nb = m.n_b();
    if nb == 0 {
        return vec![logd0];
    }
    let c5 = logd0 - D_REF_M.ln();
    let mut c = [ck[0], ck[1], ck[2], ck[3], c5];
    for j in 0..5 {
        if !m.fs.c_active[j] {
            c[j] = 0.0;
        }
    }
    let zs = [
        1.0,
        c[0],
        c[1],
        c[2],
        c[3],
        c[4],
        c[0] * c[0],
        c[0] * c[1],
        c[0] * c[3],
    ];
    let lin = if m.st.z_mode == "full" {
        let ob = m.off.beta.unwrap();
        m.z_idx
            .iter()
            .enumerate()
            .map(|(k, &zi)| zs[zi] * p[ob + k])
            .sum()
    } else {
        m.st.beta0_fixed
    };
    let l = natural_globals(m, p).l_b;
    let b0 = k_hat.map_or(0.0, |k| (k / K_REF_M2_S).ln() - lin);
    let eps0 = (b0 / l[0][0]).clamp(-3.0, 3.0) - 0.2;
    let mut v = vec![eps0];
    if nb == 2 {
        v.push(-l[1][0] * eps0 / l[1][1]);
    }
    v.push(logd0);
    v
}

fn chol(c: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = c.len();
    let mut l = vec![vec![0.0; n]; n];
    for j in 0..n {
        let mut s = c[j][j];
        for k in 0..j {
            s -= l[j][k] * l[j][k];
        }
        if !(s > 0.0) {
            return None;
        }
        l[j][j] = s.sqrt();
        for i in j + 1..n {
            let mut v = c[i][j];
            for k in 0..j {
                v -= l[i][k] * l[j][k];
            }
            l[i][j] = v / l[j][j];
        }
    }
    Some(l)
}

/// Importance-sampled locals for every global draw.
pub struct IsResult {
    /// (G*M, d) particle locals, g-major.
    pub v: Vec<Vec<f64>>,
    /// Unnormalised log weights (G*M).
    pub logw: Vec<f64>,
    pub ok: Vec<bool>,
    pub modes: Vec<Vec<f64>>,
    pub m: usize,
}

/// Defensive-mixture importance sampling of locals for every global draw (parallel over draws).
#[allow(clippy::too_many_arguments)]
pub fn importance_locals(
    m: &Model,
    pr: &Priors,
    globals: &[Vec<f64>],
    run: &RunData,
    n_local: usize,
    seed: u64,
    alpha: f64,
    inflate: f64,
    newton_iters: usize,
) -> IsResult {
    let dim = m.n_b() + 1;
    let (k_hat, q0) = prefix_d2_estimate(run);
    let q0 = q0.unwrap_or((2.0 * pr["log_d0_loc"]).exp());
    let mut prior_mu = vec![0.0; m.n_b()];
    prior_mu.push(pr["log_d0_loc"]);
    let mut prior_sd = vec![1.0; m.n_b()];
    prior_sd.push(pr["log_d0_sd"]);
    let log_q_prior = |x: &[f64]| -> f64 {
        (0..dim)
            .map(|i| {
                let z = (x[i] - prior_mu[i]) / prior_sd[i];
                -0.5 * z * z - prior_sd[i].ln() - 0.5 * (2.0 * PI).ln()
            })
            .sum()
    };
    let per: Vec<(Vec<Vec<f64>>, Vec<f64>, bool, Vec<f64>)> = globals
        .par_iter()
        .enumerate()
        .map(|(g, p)| {
            let v0 = local_init(m, p, &run.ck, k_hat, q0);
            let (mut mode, cov, ok) = newton_mode(m, pr, p, run, &v0, newton_iters);
            let cov: Vec<Vec<f64>> = if ok {
                cov.iter()
                    .map(|r| r.iter().map(|c| c * inflate * inflate).collect())
                    .collect()
            } else {
                mode = prior_mu.clone();
                (0..dim)
                    .map(|i| {
                        (0..dim)
                            .map(|j| {
                                if i == j {
                                    prior_sd[i] * prior_sd[i]
                                } else {
                                    0.0
                                }
                            })
                            .collect()
                    })
                    .collect()
            };
            let l = chol(&cov).unwrap_or_else(|| {
                (0..dim)
                    .map(|i| {
                        (0..dim)
                            .map(|j| if i == j { prior_sd[i] } else { 0.0 })
                            .collect()
                    })
                    .collect()
            });
            let logdet_l: f64 = (0..dim).map(|i| l[i][i].ln()).sum();
            let mut rng = rand::rngs::StdRng::seed_from_u64(
                seed.wrapping_mul(1_000_003).wrapping_add(g as u64),
            );
            let mut vs = Vec::with_capacity(n_local);
            let mut lw = Vec::with_capacity(n_local);
            for _ in 0..n_local {
                let from_prior = rng.random::<f64>() < alpha;
                let z: Vec<f64> = (0..dim).map(|_| StandardNormal.sample(&mut rng)).collect();
                let x: Vec<f64> = if from_prior {
                    (0..dim).map(|i| prior_mu[i] + prior_sd[i] * z[i]).collect()
                } else {
                    (0..dim)
                        .map(|i| mode[i] + (0..=i).map(|k| l[i][k] * z[k]).sum::<f64>())
                        .collect()
                };
                // log q of the mixture
                let d: Vec<f64> = (0..dim).map(|i| x[i] - mode[i]).collect();
                let mut zz = vec![0.0; dim];
                for i in 0..dim {
                    let mut s = d[i];
                    for k in 0..i {
                        s -= l[i][k] * zz[k];
                    }
                    zz[i] = s / l[i][i];
                }
                let lg = -0.5 * zz.iter().map(|v| v * v).sum::<f64>()
                    - logdet_l
                    - 0.5 * dim as f64 * (2.0 * PI).ln();
                let lp = log_q_prior(&x);
                let lq = Real::logaddexp(alpha.ln() + lp, (-alpha).ln_1p() + lg);
                let lt = local_target(m, pr, p, &x, run);
                lw.push(if lt.is_finite() {
                    lt - lq
                } else {
                    f64::NEG_INFINITY
                });
                vs.push(x);
            }
            (vs, lw, ok, mode)
        })
        .collect();
    let mut out = IsResult {
        v: vec![],
        logw: vec![],
        ok: vec![],
        modes: vec![],
        m: n_local,
    };
    for (vs, lw, ok, mode) in per {
        out.v.extend(vs);
        out.logw.extend(lw);
        out.ok.push(ok);
        out.modes.push(mode);
    }
    out
}

/// Global draws from the release posterior (equal weight), or the point estimate when absent.
pub fn select_global_draws(
    layout: &Layout,
    params: &TensorMap,
    posterior: &TensorMap,
    n: usize,
    seed: u64,
) -> anyhow::Result<Vec<Vec<f64>>> {
    let names = layout.names();
    if !posterior.is_empty() && names.iter().all(|k| posterior.contains_key(k)) {
        let d = posterior[&names[0]].shape[0];
        let idx: Vec<usize> = if n < d {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut all: Vec<usize> = (0..d).collect();
            for i in 0..n {
                let j = rng.random_range(i..d);
                all.swap(i, j);
            }
            all.truncate(n);
            all
        } else {
            (0..d).collect()
        };
        return Ok(idx
            .into_iter()
            .map(|k| {
                let mut x = vec![0.0; layout.size];
                for (s, &o) in layout.specs.iter().zip(&layout.offsets) {
                    let t = &posterior[&s.name];
                    x[o..o + s.size()].copy_from_slice(&t.data[k * s.size()..(k + 1) * s.size()]);
                }
                x
            })
            .collect());
    }
    Ok(vec![flat_from_tensors(layout, params)?])
}

pub fn spec_layout(specs: Vec<ParamSpec>) -> Layout {
    Layout::new(specs)
}

//! NUTS through `nuts-rs` (the sampler behind nutpie) on an explicit log density with gradient.
//!
//! Geometry (the reason the earlier NumPyro runs saturated tree depth): the FLEX posterior mixes
//! scales over ~4 orders of magnitude (log D0 is pinned by hundreds of points to ~1e-4 while the
//! network weights are prior-dominated) and couples locals to globals. Two remedies, both exact:
//!
//! 1. **Affine preconditioning from the MAP Laplace approximation**: x = x0 + L^-T z with
//!    H = L L^T the (regularised) negative Hessian at the MAP. NUTS runs on z; the Jacobian is
//!    constant, so the target is unchanged. This is a fixed *dense* metric.
//! 2. **Adapted mass matrix** on top (diagonal, or nutpie's low-rank-plus-diagonal), with step size
//!    dual averaging to `target_accept`.
//!
//! Chains run in parallel on the profile's bounded Rayon pool; each chain's density may itself fan
//! out over runs on the same pool.

use std::collections::HashMap;
use std::time::Instant;

use anyhow::Context;
use nuts_rs::{
    Chain, CpuLogpFunc, CpuMath, CpuMathError, DiagNutsSettings, HasDims, LogpError,
    LowRankNutsSettings, Settings,
};
use rand::SeedableRng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::diagnostics::ChainStats;
use crate::studentt::Mat;

/// Log density and gradient on unconstrained coordinates. `-inf`/NaN means outside support.
pub trait LogDensity: Sync {
    fn dim(&self) -> usize;
    fn logp_grad(&self, x: &[f64], grad: &mut [f64]) -> f64;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NutsConfig {
    pub chains: usize,
    pub warmup: u64,
    pub samples: u64,
    pub target_accept: f64,
    pub max_tree_depth: u64,
    /// "diag" | "low_rank"
    pub mass_matrix: String,
    /// "laplace" (dense MAP Hessian metric) | "none"
    pub precondition: String,
    pub init_jitter: f64,
    pub rhat_max: f64,
    pub min_ess: f64,
}

impl Default for NutsConfig {
    fn default() -> Self {
        NutsConfig {
            chains: 4,
            warmup: 1000,
            samples: 1500,
            target_accept: 0.9,
            max_tree_depth: 10,
            mass_matrix: "diag".into(),
            precondition: "laplace".into(),
            init_jitter: 0.05,
            rhat_max: 1.01,
            min_ess: 100.0,
        }
    }
}

/// x = x0 + A z (A = identity, or the saddle-free inverse square root of the negative Hessian).
#[derive(Debug, Clone)]
pub struct Precond {
    pub x0: Vec<f64>,
    /// A (n x n) and A^-1, row-major.
    pub a: Option<(Mat, Mat)>,
    /// Number of eigen-directions whose curvature was non-positive or floored.
    pub ridge: f64,
}

fn matvec(m: &Mat, v: &[f64]) -> Vec<f64> {
    (0..m.n)
        .map(|i| (0..m.m).map(|k| m.at(i, k) * v[k]).sum())
        .collect()
}
fn matvec_t(m: &Mat, v: &[f64]) -> Vec<f64> {
    let mut o = vec![0.0; m.m];
    for i in 0..m.n {
        for k in 0..m.m {
            o[k] += m.at(i, k) * v[i];
        }
    }
    o
}

impl Precond {
    pub fn identity(x0: &[f64]) -> Precond {
        Precond {
            x0: x0.to_vec(),
            a: None,
            ridge: 0.0,
        }
    }
    /// From the negative Hessian `h`: symmetrised, scaled to unit diagonal (D = sqrt|H_ii|), then
    /// eigen-decomposed; curvatures are replaced by max(|lambda|, 1e-2) (saddle-free metric: a
    /// negative or vanishing direction gets a unit-scale metric instead of a global ridge that
    /// would flatten every weakly curved direction). Non-finite entries count as zero.
    pub fn from_neg_hessian(x0: &[f64], h: &Mat) -> Precond {
        let n = h.n;
        let d: Vec<f64> = (0..n)
            .map(|i| {
                let a = h.at(i, i).abs();
                if a.is_finite() && a > 1e-12 {
                    a.sqrt()
                } else {
                    1.0
                }
            })
            .collect();
        let c = faer::Mat::<f64>::from_fn(n, n, |i, j| {
            let v = 0.5 * (h.at(i, j) + h.at(j, i)) / (d[i] * d[j]);
            if v.is_finite() {
                v
            } else {
                0.0
            }
        });
        let Ok(evd) = c.self_adjoint_eigen(faer::Side::Lower) else {
            return Precond::identity(x0);
        };
        let u = evd.U();
        let lam: Vec<f64> = (0..n).map(|k| evd.S().column_vector()[k]).collect();
        let mut floored = 0;
        let lp: Vec<f64> = lam
            .iter()
            .map(|&l| {
                if !(l > 1e-2) {
                    floored += 1;
                }
                if l.is_finite() {
                    l.abs().max(1e-2)
                } else {
                    1.0
                }
            })
            .collect();
        let mut a = Mat::zeros(n, n);
        let mut ai = Mat::zeros(n, n);
        for i in 0..n {
            for k in 0..n {
                // A = D^-1 U diag(lp^-1/2) ; A^-1 = diag(lp^1/2) U^T D
                a.set(i, k, u[(i, k)] / (d[i] * lp[k].sqrt()));
                ai.set(k, i, lp[k].sqrt() * u[(i, k)] * d[i]);
            }
        }
        Precond {
            x0: x0.to_vec(),
            a: Some((a, ai)),
            ridge: floored as f64,
        }
    }
    pub fn to_x(&self, z: &[f64]) -> Vec<f64> {
        match &self.a {
            None => self.x0.iter().zip(z).map(|(a, b)| a + b).collect(),
            Some((a, _)) => matvec(a, z)
                .iter()
                .zip(&self.x0)
                .map(|(a, b)| a + b)
                .collect(),
        }
    }
    pub fn to_z(&self, x: &[f64]) -> Vec<f64> {
        let d: Vec<f64> = x.iter().zip(&self.x0).map(|(a, b)| a - b).collect();
        match &self.a {
            None => d,
            Some((_, ai)) => matvec(ai, &d),
        }
    }
    /// grad_z = A^T grad_x.
    pub fn grad_z(&self, gx: &[f64]) -> Vec<f64> {
        match &self.a {
            None => gx.to_vec(),
            Some((a, _)) => matvec_t(a, gx),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("log density outside support")]
pub struct OutOfSupport;

impl LogpError for OutOfSupport {
    fn is_recoverable(&self) -> bool {
        true
    }
}

struct Func<'a> {
    model: &'a dyn LogDensity,
    pre: &'a Precond,
    gx: Vec<f64>,
}

impl HasDims for Func<'_> {
    fn dim_sizes(&self) -> HashMap<String, u64> {
        [(
            "unconstrained_parameter".to_string(),
            self.model.dim() as u64,
        )]
        .into_iter()
        .collect()
    }
}

impl CpuLogpFunc for Func<'_> {
    type LogpError = OutOfSupport;
    type FlowParameters = ();
    type ExpandedVector = Vec<f64>;
    fn dim(&self) -> usize {
        self.model.dim()
    }
    fn logp(&mut self, z: &[f64], grad: &mut [f64]) -> Result<f64, OutOfSupport> {
        let x = self.pre.to_x(z);
        self.gx.resize(x.len(), 0.0);
        let v = self.model.logp_grad(&x, &mut self.gx);
        if !v.is_finite() || self.gx.iter().any(|g| !g.is_finite()) {
            return Err(OutOfSupport);
        }
        grad.copy_from_slice(&self.pre.grad_z(&self.gx));
        Ok(v)
    }
    fn expand_vector<R: rand::Rng + ?Sized>(
        &mut self,
        _rng: &mut R,
        z: &[f64],
    ) -> Result<Vec<f64>, CpuMathError> {
        Ok(self.pre.to_x(z))
    }
}

#[derive(Debug, Clone)]
pub struct NutsResult {
    /// (chain, draw, dim) on the original x coordinates.
    pub draws: Vec<Vec<Vec<f64>>>,
    pub stats: Vec<ChainStats>,
    pub wall_s: f64,
    pub gradient_evaluations: u64,
}

fn run_chain<S: Settings>(
    settings: &S,
    maxdepth: u64,
    chain: u64,
    model: &dyn LogDensity,
    pre: &Precond,
    init_z: &[f64],
    seed: u64,
) -> anyhow::Result<(Vec<Vec<f64>>, ChainStats, u64)> {
    let func = Func {
        model,
        pre,
        gx: vec![],
    };
    let mut rng = rand::rngs::StdRng::seed_from_u64(
        seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(chain),
    );
    let mut sampler = settings
        .new_chain(chain, CpuMath::new(func), &mut rng)
        .context("nuts settings")?;
    sampler
        .set_position(init_z)
        .context("initial position outside support")?;
    let total = settings.hint_num_tune() + settings.hint_num_draws();
    let mut draws = Vec::with_capacity(settings.hint_num_draws());
    let mut st = ChainStats::default();
    let mut evals = 0;
    for i in 0..total {
        let (z, info) = sampler.draw().context("unrecoverable sampler error")?;
        evals += info.num_gradients;
        if (i + 1) % 250 == 0 {
            eprintln!(
                "nuts chain {chain}: {}/{total} {} step_size={:.3e} leapfrog={}",
                i + 1,
                if info.tuning { "tune" } else { "draw" },
                info.step_size,
                info.num_steps
            );
        }
        if i >= settings.hint_num_tune() {
            draws.push(pre.to_x(&z));
            st.diverging.push(info.diverging);
            st.num_steps.push(info.num_steps);
            let depth = (info.num_steps as f64 + 1.0).log2().ceil() as u64;
            st.depth.push(depth);
            st.maxdepth_reached
                .push(info.num_steps + 1 >= 1u64 << maxdepth);
            st.step_size = info.step_size;
        }
    }
    Ok((draws, st, evals))
}

/// Run `cfg.chains` chains in parallel on `pool`, initialised at `init` (+ jitter in z-space).
pub fn run_nuts(
    model: &dyn LogDensity,
    init: &[f64],
    pre: &Precond,
    cfg: &NutsConfig,
    seed: u64,
    pool: &rayon::ThreadPool,
) -> anyhow::Result<NutsResult> {
    let t0 = Instant::now();
    let z0 = pre.to_z(init);
    let mut jr = rand::rngs::StdRng::seed_from_u64(seed ^ 0x5eed);
    let inits: Vec<Vec<f64>> = (0..cfg.chains)
        .map(|_c| {
            z0.iter()
                .map(|v| {
                    let e: f64 =
                        rand_distr::Distribution::sample(&rand_distr::StandardNormal, &mut jr);
                    v + cfg.init_jitter * e
                })
                .collect()
        })
        .collect();
    let out: Vec<anyhow::Result<(Vec<Vec<f64>>, ChainStats, u64)>> = pool.install(|| {
        inits
            .par_iter()
            .enumerate()
            .map(|(c, z)| {
                if cfg.mass_matrix == "low_rank" {
                    let mut s = LowRankNutsSettings::default();
                    s.num_tune = cfg.warmup;
                    s.num_draws = cfg.samples;
                    s.maxdepth = cfg.max_tree_depth;
                    s.adapt_options.step_size_settings.target_accept = cfg.target_accept;
                    s.seed = seed;
                    run_chain(&s, cfg.max_tree_depth, c as u64, model, pre, z, seed)
                } else {
                    let mut s = DiagNutsSettings::default();
                    s.num_tune = cfg.warmup;
                    s.num_draws = cfg.samples;
                    s.maxdepth = cfg.max_tree_depth;
                    s.adapt_options.step_size_settings.target_accept = cfg.target_accept;
                    s.seed = seed;
                    run_chain(&s, cfg.max_tree_depth, c as u64, model, pre, z, seed)
                }
            })
            .collect()
    });
    let mut draws = Vec::new();
    let mut stats = Vec::new();
    let mut evals = 0;
    for r in out {
        let (d, s, e) = r?;
        draws.push(d);
        stats.push(s);
        evals += e;
    }
    Ok(NutsResult {
        draws,
        stats,
        wall_s: t0.elapsed().as_secs_f64(),
        gradient_evaluations: evals,
    })
}

/// Dense negative Hessian by central differences of exact gradients. `directions[k]` lists the
/// coordinates perturbed *together* in probe k (graph colouring: coordinates whose gradients do
/// not interact, e.g. the same local of different runs); `owner(i, k)` says whether the gradient
/// coordinate i responds to probe k's perturbation of coordinate `j` (returns that j).
pub fn neg_hessian_fd(
    x: &[f64],
    grad: &(dyn Fn(&[f64]) -> Vec<f64> + Sync),
    probes: &[Vec<usize>],
    attribute: &(dyn Fn(usize, &[usize]) -> Option<usize> + Sync),
    pool: &rayon::ThreadPool,
) -> Mat {
    let n = x.len();
    let cols: Vec<(Vec<f64>, Vec<f64>, Vec<f64>)> = pool.install(|| {
        probes
            .iter()
            .map(|p| {
                let h: Vec<f64> = p.iter().map(|&j| 1e-5 * x[j].abs().max(1.0)).collect();
                let mut xp = x.to_vec();
                let mut xm = x.to_vec();
                for (k, &j) in p.iter().enumerate() {
                    xp[j] += h[k];
                    xm[j] -= h[k];
                }
                (grad(&xp), grad(&xm), h)
            })
            .collect()
    });
    let mut hm = Mat::zeros(n, n);
    for (p, (gp, gm, h)) in probes.iter().zip(&cols) {
        for i in 0..n {
            if let Some(j) = attribute(i, p) {
                let k = p.iter().position(|&q| q == j).unwrap();
                hm.set(i, j, -(gp[i] - gm[i]) / (2.0 * h[k]));
            }
        }
    }
    // symmetrise from the filled entries
    for i in 0..n {
        for j in 0..i {
            let (a, b) = (hm.at(i, j), hm.at(j, i));
            let v = if a == 0.0 {
                b
            } else if b == 0.0 {
                a
            } else {
                0.5 * (a + b)
            };
            hm.set(i, j, v);
            hm.set(j, i, v);
        }
    }
    hm
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Badly scaled correlated Gaussian: preconditioned NUTS recovers mean/variance.
    struct Gauss;
    impl LogDensity for Gauss {
        fn dim(&self) -> usize {
            2
        }
        fn logp_grad(&self, x: &[f64], g: &mut [f64]) -> f64 {
            // precision [[1e6, 990], [990, 1]] / det-ish
            let (a, b, c) = (1e6, 999.0, 1.0);
            g[0] = -(a * x[0] + b * x[1]);
            g[1] = -(b * x[0] + c * x[1]);
            -0.5 * (a * x[0] * x[0] + 2.0 * b * x[0] * x[1] + c * x[1] * x[1])
        }
    }

    #[test]
    fn laplace_preconditioned_nuts_on_ill_conditioned_gaussian() {
        let h = Mat {
            n: 2,
            m: 2,
            a: vec![1e6, 999.0, 999.0, 1.0],
        };
        let pre = Precond::from_neg_hessian(&[0.0, 0.0], &h);
        let cfg = NutsConfig {
            chains: 2,
            warmup: 300,
            samples: 500,
            ..Default::default()
        };
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        let r = run_nuts(&Gauss, &[0.0, 0.0], &pre, &cfg, 1, &pool).unwrap();
        let x1: Vec<f64> = r.draws.iter().flatten().map(|d| d[1]).collect();
        let var1 = x1.iter().map(|v| v * v).sum::<f64>() / x1.len() as f64;
        // marginal variance of x1 = a / (a c - b^2) = 1e6 / 2001
        assert!((var1 / (1e6 / 2001.0) - 1.0).abs() < 0.25, "{var1}");
        let mean_steps =
            r.stats.iter().flat_map(|s| s.num_steps.iter()).sum::<u64>() as f64 / 1000.0;
        assert!(mean_steps < 8.0, "{mean_steps}");
    }
}

//! BASS member objective, priors and local state inference (V3.1 §9.5, §9.7, §10.2, §10.5).
//!
//! Per member: 5825 network weights + 9 β_v + ζ + 3 observational parameters = 5838 global fit
//! parameters, plus 12 local parameters per run (a_0..a_7 initial front, b_0..b_3 velocity variation).
//! Priors [IGNIS, design]: a_0 ~ N(0,1), a_m ~ N(0,(0.5/m²)²); b_m ~ N(0,(0.25/(1+m))²); ζ ~ N(0,1);
//! β_v ~ N(0, s_β²); network weights N(0, (s/√fan_in)²), biases N(0, 0.1²); observation scales
//! log-normal. Group bootstrap: a group drawn w times is w copies of its runs, so the multiplicity
//! weights local prior + likelihood together (review-B fix 4).
//!
//! Local inference for a prefix (per member, globals fixed): MAP + Laplace, then a DEFENSIVE mixture
//! q = (1−ε̂) N(b*, H⁻¹) + ε̂ p₀ with ε̂ = ⌈εn⌉/n the fraction actually drawn (review-B fix 3),
//! importance weights p₀ L / q (both components in the denominator), normalised jointly over
//! (member, particle). The ensemble is an empirical MAP ensemble, not a Bayesian posterior.

use rand::{rngs::StdRng, RngExt, SeedableRng};
use rand_distr::StandardNormal;
use rayon::prelude::*;
use serde_json::{json, Value};

use crate::data::{NormArrays, RunData};
use crate::dynamics::{self as dyn_, Ctx, Numerics, PhysOffsets, SimWork, Trajectory};
use crate::observation::{self as obs, ObsGrad, ObsParams};
use crate::operator::{self as op, Grid, NetOffsets};

pub const N_LOCAL: usize = op::N_FRONT_COEFFS + op::N_VELOCITY_COEFFS;

/// Flat layout of one member's globals, blocks in sorted-name order (registry `TensorMap` order).
#[derive(Debug, Clone)]
pub struct GlobalLayout {
    pub blocks: Vec<(String, Vec<usize>, usize)>,
    pub len: usize,
    pub net: NetOffsets,
    pub phys: PhysOffsets,
    pub lsc: usize,
    pub lsn: usize,
    pub lsr: usize,
}

pub fn global_shapes() -> Vec<(String, Vec<usize>)> {
    let mut s = op::param_shapes();
    s.push(("physics.beta_v".into(), vec![1 + op::N_CONDITIONS]));
    s.push(("physics.zeta".into(), vec![]));
    for n in ["obs.log_sigma_c", "obs.log_sigma_n", "obs.log_scale_rel"] {
        s.push((n.into(), vec![]));
    }
    s
}

impl GlobalLayout {
    pub fn new() -> GlobalLayout {
        let mut sh = global_shapes();
        sh.sort_by(|a, b| a.0.cmp(&b.0));
        let mut off = 0;
        let mut blocks = vec![];
        for (n, s) in sh {
            let l: usize = s.iter().product();
            blocks.push((n, s, off));
            off += l;
        }
        let f = |n: &str| blocks.iter().find(|b| b.0 == n).unwrap().2;
        let net = NetOffsets {
            lift_w: f("lift.W"),
            lift_b: f("lift.b"),
            blk_w: [f("block0.W"), f("block1.W"), f("block2.W")],
            blk_b: [f("block0.b"), f("block1.b"), f("block2.b")],
            blk_a: [f("block0.A"), f("block1.A"), f("block2.A")],
            head_w: f("head.W"),
            head_b: f("head.b"),
            film_w1: f("film.W1"),
            film_b1: f("film.b1"),
            film_w2: f("film.W2"),
            film_b2: f("film.b2"),
        };
        GlobalLayout {
            phys: PhysOffsets {
                beta_v: f("physics.beta_v"),
                zeta: f("physics.zeta"),
            },
            lsc: f("obs.log_sigma_c"),
            lsn: f("obs.log_sigma_n"),
            lsr: f("obs.log_scale_rel"),
            net,
            len: off,
            blocks,
        }
    }
    pub fn obs(&self, p: &[f64]) -> ObsParams {
        ObsParams {
            log_sigma_c: p[self.lsc],
            log_sigma_n: p[self.lsn],
            log_scale_rel: p[self.lsr],
        }
    }
}

impl Default for GlobalLayout {
    fn default() -> Self {
        Self::new()
    }
}

pub fn n_global() -> usize {
    global_shapes()
        .iter()
        .map(|(_, s)| s.iter().product::<usize>())
        .sum()
}

pub fn local_sd() -> [f64; N_LOCAL] {
    let mut s = [0.0; N_LOCAL];
    for m in 0..op::N_FRONT_COEFFS {
        s[m] = if m == 0 { 1.0 } else { 0.5 / (m * m) as f64 };
    }
    for m in 0..op::N_VELOCITY_COEFFS {
        s[op::N_FRONT_COEFFS + m] = 0.25 / (1.0 + m as f64);
    }
    s
}

const LN_2PI: f64 = 1.8378770664093453;

pub fn log_prior_local(x: &[f64], grad: Option<&mut [f64]>, weight: f64) -> f64 {
    let sd = local_sd();
    let mut lp = 0.0;
    for i in 0..N_LOCAL {
        let z = x[i] / sd[i];
        lp += -0.5 * z * z - sd[i].ln() - 0.5 * LN_2PI;
    }
    if let Some(g) = grad {
        for i in 0..N_LOCAL {
            g[i] -= weight * x[i] / (sd[i] * sd[i]);
        }
    }
    lp
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Priors {
    pub weight_scale: f64,
    pub bias_sd: f64,
    pub beta_v_sd: f64,
    pub obs: [(f64, f64); 3],
}

impl Priors {
    pub fn from_json(v: &Value) -> Priors {
        let g = |k: &str, d: f64| v.get(k).and_then(|x| x.as_f64()).unwrap_or(d);
        let o = |k: &str, m: f64, s: f64| {
            (
                v[k]["mean"].as_f64().unwrap_or(m),
                v[k]["sd"].as_f64().unwrap_or(s),
            )
        };
        Priors {
            weight_scale: g("weight_scale", 1.0),
            bias_sd: g("bias_sd", 0.1),
            beta_v_sd: g("beta_v_sd", 2.0),
            obs: [
                o("obs.log_sigma_c", 1e-4f64.ln(), 1.5),
                o("obs.log_sigma_n", 5e-5f64.ln(), 1.5),
                o("obs.log_scale_rel", 1e-2f64.ln(), 1.0),
            ],
        }
    }
}

pub fn default_priors() -> Value {
    json!({"weight_scale": 1.0, "bias_sd": 0.1, "beta_v_sd": 2.0,
           "obs.log_sigma_c": {"mean": 1e-4f64.ln(), "sd": 1.5},
           "obs.log_sigma_n": {"mean": 5e-5f64.ln(), "sd": 1.5},
           "obs.log_scale_rel": {"mean": 1e-2f64.ln(), "sd": 1.0}})
}

fn normal(p: &[f64], g: &mut Option<&mut [f64]>, off: usize, len: usize, mu: f64, sd: f64) -> f64 {
    let c = -sd.ln() - 0.5 * LN_2PI;
    let mut s = 0.0;
    for i in off..off + len {
        let z = (p[i] - mu) / sd;
        s += -0.5 * z * z + c;
        if let Some(g) = g.as_deref_mut() {
            g[i] -= z / sd;
        }
    }
    s
}

pub fn log_prior_global(
    p: &[f64],
    lay: &GlobalLayout,
    pr: &Priors,
    mut g: Option<&mut [f64]>,
) -> f64 {
    let mut lp = 0.0;
    let net: Vec<String> = op::param_shapes().into_iter().map(|b| b.0).collect();
    for (name, shape, off) in &lay.blocks {
        let len: usize = shape.iter().product();
        if net.contains(name) {
            let bias = (name.ends_with('b') || name.ends_with("b1") || name.ends_with("b2"))
                && shape.len() == 1;
            let sd = if bias {
                pr.bias_sd
            } else {
                pr.weight_scale / (*shape.last().unwrap() as f64).sqrt()
            };
            lp += normal(p, &mut g, *off, len, 0.0, sd);
        }
    }
    lp += normal(
        p,
        &mut g,
        lay.phys.beta_v,
        1 + op::N_CONDITIONS,
        0.0,
        pr.beta_v_sd,
    );
    lp += normal(p, &mut g, lay.phys.zeta, 1, 0.0, 1.0);
    for (i, off) in [lay.lsc, lay.lsn, lay.lsr].into_iter().enumerate() {
        lp += normal(p, &mut g, off, 1, pr.obs[i].0, pr.obs[i].1);
    }
    lp
}

/// Per-thread buffers of one run evaluation.
#[derive(Default)]
pub struct RunWork {
    pub sim: SimWork,
    pub tr: Trajectory,
    pub fbar: Vec<f64>,
}

pub struct RunEval {
    pub ll: f64,
    pub unreached: f64,
    pub max_cfl: f64,
}

/// log p(frames | globals, locals) of one run; with `grad` = (ĝ globals, x̄ locals) adjoints scaled
/// by `weight` are added.
#[allow(clippy::too_many_arguments)]
pub fn run_loglik(
    p: &[f64],
    lay: &GlobalLayout,
    grid: &Grid,
    num: Numerics,
    run: &RunData,
    norm: &NormArrays,
    x: &[f64],
    grad: Option<(&mut [f64], &mut [f64], f64)>,
    w: &mut RunWork,
) -> RunEval {
    let cx = Ctx {
        p,
        net: &lay.net,
        phys: &lay.phys,
        grid,
        num,
        l: run.l,
        w: run.w,
    };
    let cond = run.cond(norm);
    let (a, b) = x.split_at(op::N_FRONT_COEFFS);
    dyn_::simulate(
        &cx,
        a,
        b,
        &run.t,
        &cond,
        grad.is_some(),
        &mut w.sim,
        &mut w.tr,
    );
    let k = grid.k;
    let o = lay.obs(p);
    let ev = |ll| RunEval {
        ll,
        unreached: w.tr.unreached,
        max_cfl: w.tr.max_cfl,
    };
    let Some((gp, gx, weight)) = grad else {
        let ll = obs::sequence_loglik(&w.tr.frames, k, &run.pre, o, None);
        return RunEval {
            ll,
            unreached: w.tr.unreached,
            max_cfl: w.tr.max_cfl,
        };
    };
    w.fbar.clear();
    w.fbar.resize(w.tr.frames.len(), 0.0);
    let mut og = ObsGrad::default();
    let ll = obs::sequence_loglik(&w.tr.frames, k, &run.pre, o, Some((&mut w.fbar, &mut og)));
    if weight != 1.0 {
        w.fbar.iter_mut().for_each(|v| *v *= weight);
    }
    gp[lay.lsc] += weight * og.log_sigma_c;
    gp[lay.lsn] += weight * og.log_sigma_n;
    gp[lay.lsr] += weight * og.log_scale_rel;
    let (ga, gb) = gx.split_at_mut(op::N_FRONT_COEFFS);
    let fbar = std::mem::take(&mut w.fbar);
    dyn_::simulate_backward(&cx, a, b, &cond, &w.tr, &fbar, gp, ga, gb, &mut w.sim);
    w.fbar = fbar;
    ev(ll)
}

pub struct MemberEval {
    pub value: f64,
    pub loglik: f64,
    pub unreached: f64,
    pub max_cfl: f64,
}

/// Negative log joint of one member: −[log p(g) + Σ_i w_i (log p(x_i) + log L_i)], with gradients
/// (globals into `gg`, locals (R × 12) into `gx`). Runs evaluated in parallel.
#[allow(clippy::too_many_arguments)]
pub fn member_objective(
    p: &[f64],
    x: &[f64],
    runs: &[RunData],
    weights: &[f64],
    lay: &GlobalLayout,
    grid: &Grid,
    num: Numerics,
    norm: &NormArrays,
    pr: &Priors,
    gg: &mut [f64],
    gx: &mut [f64],
) -> MemberEval {
    gg.iter_mut().for_each(|v| *v = 0.0);
    gx.iter_mut().for_each(|v| *v = 0.0);
    let parts: Vec<(f64, f64, f64, f64, Vec<f64>)> = runs
        .par_iter()
        .zip(gx.par_chunks_mut(N_LOCAL))
        .enumerate()
        .map_init(RunWork::default, |w, (i, (run, gxi))| {
            if weights[i] == 0.0 {
                return (0.0, 0.0, 0.0, 0.0, vec![]);
            }
            let xi = &x[i * N_LOCAL..(i + 1) * N_LOCAL];
            let mut g = vec![0.0; lay.len];
            let ev = run_loglik(
                p,
                lay,
                grid,
                num,
                run,
                norm,
                xi,
                Some((&mut g, gxi, weights[i])),
                w,
            );
            let lpl = log_prior_local(xi, Some(gxi), weights[i]);
            (
                weights[i] * (lpl + ev.ll),
                weights[i] * ev.ll,
                ev.unreached,
                ev.max_cfl,
                g,
            )
        })
        .collect();
    let mut total = log_prior_global(p, lay, pr, Some(gg));
    let mut ll = 0.0;
    let (mut unr, mut cfl) = (0.0f64, 0.0f64);
    for (v, l, u, c, g) in &parts {
        total += v;
        ll += l;
        unr = unr.max(*u);
        cfl = cfl.max(*c);
        for (a, b) in gg.iter_mut().zip(g) {
            *a += b;
        }
    }
    gg.iter_mut().for_each(|v| *v = -*v);
    gx.iter_mut().for_each(|v| *v = -*v);
    MemberEval {
        value: -total,
        loglik: ll,
        unreached: unr,
        max_cfl: cfl,
    }
}

/// a from masked least squares of logit(f/L) at the first usable frame; b = 0.
pub fn init_locals(run: &RunData) -> [f64; N_LOCAL] {
    let s = run.n_s();
    let phi = op::cosine_basis(&run.xi, op::N_FRONT_COEFFS);
    let d = op::N_FRONT_COEFFS;
    let mut gram = vec![0.0; d * d];
    let mut rhs = vec![0.0; d];
    for i in 0..s {
        if !run.valid[i] {
            continue;
        }
        let eta = (run.f[i] / run.l).clamp(1e-3, 1.0 - 1e-3);
        let y = (eta / (1.0 - eta)).ln();
        for a in 0..d {
            rhs[a] += phi[i * d + a] * y;
            for b in 0..d {
                gram[a * d + b] += phi[i * d + a] * phi[i * d + b];
            }
        }
    }
    for a in 0..d {
        gram[a * d + a] += 1e-9;
    }
    obs::cholesky(&mut gram, d);
    obs::solve_lower(&gram, d, &mut rhs);
    obs::solve_upper_t(&gram, d, &mut rhs);
    let mut x = [0.0; N_LOCAL];
    x[..d].copy_from_slice(&rhs);
    x
}

/// Median displacement speed between the first and last usable frames (β_v initialisation).
pub fn speed_estimate(runs: &[RunData]) -> f64 {
    let mut out = vec![];
    for r in runs {
        let idx: Vec<usize> = (0..r.n_t()).filter(|&i| r.use_[i]).collect();
        if idx.len() < 2 {
            continue;
        }
        let (a, z) = (idx[0], *idx.last().unwrap());
        let s = r.n_s();
        let both: Vec<usize> = (0..s)
            .filter(|&j| r.valid[a * s + j] && r.valid[z * s + j])
            .collect();
        if !both.is_empty() && r.t[z] > r.t[a] {
            let m = both
                .iter()
                .map(|&j| r.f[z * s + j] - r.f[a * s + j])
                .sum::<f64>()
                / both.len() as f64;
            out.push(m / (r.t[z] - r.t[a]));
        }
    }
    if out.is_empty() {
        return 0.0;
    }
    out.sort_by(|a, b| a.total_cmp(b));
    let n = out.len();
    if n % 2 == 1 {
        out[n / 2]
    } else {
        0.5 * (out[n / 2 - 1] + out[n / 2])
    }
}

// ---------------------------------------------------------------- local inference

/// Symmetric eigen-decomposition (cyclic Jacobi) of an n × n matrix: (values, vectors as columns).
pub fn sym_eigen(a: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut m = a.to_vec();
    let mut v = vec![0.0; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }
    for _ in 0..100 {
        let off: f64 = (0..n)
            .flat_map(|i| (0..n).filter(move |&j| j != i).map(move |j| (i, j)))
            .map(|(i, j)| m[i * n + j].powi(2))
            .sum();
        if off
            < 1e-30
                * (0..n)
                    .map(|i| m[i * n + i].powi(2))
                    .sum::<f64>()
                    .max(1e-300)
        {
            break;
        }
        for p in 0..n {
            for q in p + 1..n {
                let apq = m[p * n + q];
                if apq.abs() < 1e-300 {
                    continue;
                }
                let theta = (m[q * n + q] - m[p * n + p]) / (2.0 * apq);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..n {
                    let (mkp, mkq) = (m[k * n + p], m[k * n + q]);
                    m[k * n + p] = c * mkp - s * mkq;
                    m[k * n + q] = s * mkp + c * mkq;
                }
                for k in 0..n {
                    let (mpk, mqk) = (m[p * n + k], m[q * n + k]);
                    m[p * n + k] = c * mpk - s * mqk;
                    m[q * n + k] = s * mpk + c * mqk;
                }
                for k in 0..n {
                    let (vkp, vkq) = (v[k * n + p], v[k * n + q]);
                    v[k * n + p] = c * vkp - s * vkq;
                    v[k * n + q] = s * vkp + c * vkq;
                }
            }
        }
    }
    ((0..n).map(|i| m[i * n + i]).collect(), v)
}

pub struct LocalPosterior {
    /// (P × 12)
    pub particles: Vec<f64>,
    pub logw: Vec<f64>,
    pub map: [f64; N_LOCAL],
    pub map_converged: bool,
    pub diagnostics: Value,
}

/// −(log p₀(x) + log L(x)) and its gradient w.r.t. the locals.
fn negpost(
    p: &[f64],
    lay: &GlobalLayout,
    grid: &Grid,
    num: Numerics,
    run: &RunData,
    norm: &NormArrays,
    x: &[f64],
    g: &mut [f64],
    w: &mut RunWork,
    gp: &mut Vec<f64>,
) -> f64 {
    gp.clear();
    gp.resize(lay.len, 0.0);
    g.iter_mut().for_each(|v| *v = 0.0);
    let ev = run_loglik(p, lay, grid, num, run, norm, x, Some((gp, g, 1.0)), w);
    let lp = log_prior_local(x, Some(g), 1.0);
    g.iter_mut().for_each(|v| *v = -*v);
    let v = -(lp + ev.ll);
    if v.is_finite() {
        v
    } else {
        f64::INFINITY
    }
}

/// Defensive-mixture importance sampling of the 12 locals for one member (globals fixed).
#[allow(clippy::too_many_arguments)]
pub fn local_inference(
    p: &[f64],
    lay: &GlobalLayout,
    grid: &Grid,
    num: Numerics,
    run: &RunData,
    norm: &NormArrays,
    n_particles: usize,
    epsilon: f64,
    seed: u64,
    maxiter: usize,
) -> LocalPosterior {
    let mut w = RunWork::default();
    let mut gp = vec![];
    let x0 = init_locals(run);
    let res = ignis_acme::optim::lbfgs(
        |x, g| negpost(p, lay, grid, num, run, norm, x, g, &mut w, &mut gp),
        &x0,
        maxiter,
        10,
    );
    let xs: [f64; N_LOCAL] = res.x.clone().try_into().unwrap();
    // Hessian by central differences of the analytic gradient
    let n = N_LOCAL;
    let sd = local_sd();
    let mut h = vec![0.0; n * n];
    let (mut gpl, mut gmi) = (vec![0.0; n], vec![0.0; n]);
    for i in 0..n {
        let e = 1e-5 * sd[i];
        let mut xp = xs;
        let mut xm = xs;
        xp[i] += e;
        xm[i] -= e;
        negpost(p, lay, grid, num, run, norm, &xp, &mut gpl, &mut w, &mut gp);
        negpost(p, lay, grid, num, run, norm, &xm, &mut gmi, &mut w, &mut gp);
        for j in 0..n {
            h[i * n + j] = (gpl[j] - gmi[j]) / (2.0 * e);
        }
    }
    for i in 0..n {
        for j in 0..i {
            let s = 0.5 * (h[i * n + j] + h[j * n + i]);
            h[i * n + j] = s;
            h[j * n + i] = s;
        }
    }
    let (ev, vecs) = sym_eigen(&h, n);
    let floor = 1.0 / (10.0 * sd.iter().cloned().fold(0.0, f64::max)).powi(2); // Laplace never wider than 10x the prior
    let lam: Vec<f64> = ev.iter().map(|v| v.max(floor)).collect();
    let mut cov = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            cov[i * n + j] = (0..n)
                .map(|k| vecs[i * n + k] * vecs[j * n + k] / lam[k])
                .sum();
        }
    }
    let mut chol = cov.clone();
    for i in 0..n {
        chol[i * n + i] += 1e-14;
    }
    obs::cholesky(&mut chol, n);
    let logdet: f64 = 2.0 * (0..n).map(|i| chol[i * n + i].ln()).sum::<f64>();
    let n_def = if epsilon > 0.0 {
        (epsilon * n_particles as f64).ceil() as usize
    } else {
        0
    };
    let eps = n_def as f64 / n_particles as f64; // the fraction actually drawn (review-B fix 3)
    let mut rng = StdRng::seed_from_u64(seed);
    let mut xpart = vec![0.0; n_particles * n];
    for pi in 0..n_particles {
        let z: Vec<f64> = (0..n)
            .map(|_| rng.sample::<f64, _>(StandardNormal))
            .collect();
        for i in 0..n {
            xpart[pi * n + i] = if pi < n_particles - n_def {
                xs[i] + (0..=i).map(|k| chol[i * n + k] * z[k]).sum::<f64>()
            } else {
                z[i] * sd[i]
            };
        }
    }
    let logw: Vec<f64> = xpart
        .par_chunks(n)
        .map_init(RunWork::default, |w, x| {
            let mut d: Vec<f64> = (0..n).map(|i| x[i] - xs[i]).collect();
            obs::solve_lower(&chol, n, &mut d);
            let q: f64 = d.iter().map(|v| v * v).sum();
            let log_lap = -0.5 * q - 0.5 * logdet - 0.5 * n as f64 * LN_2PI;
            let log_p0 = log_prior_local(x, None, 1.0);
            let log_q = if n_def > 0 {
                logaddexp((-eps).ln_1p() + log_lap, eps.ln() + log_p0)
            } else {
                log_lap
            };
            let ll = run_loglik(p, lay, grid, num, run, norm, x, None, w).ll;
            let lw = log_p0 + ll - log_q;
            if lw.is_finite() {
                lw
            } else {
                f64::NEG_INFINITY
            }
        })
        .collect();
    LocalPosterior {
        diagnostics: ignis_science::gauss_hermite::weight_diagnostics(&logw),
        particles: xpart,
        logw,
        map: xs,
        map_converged: res.converged,
    }
}

pub fn logaddexp(a: f64, b: f64) -> f64 {
    let m = a.max(b);
    if m == f64::NEG_INFINITY {
        return m;
    }
    m + ((a - m).exp() + (b - m).exp()).ln()
}

/// Spatial mean front position Σ_j q_j f_j (trapezoid, m).
pub fn mean_position(f: &[f64]) -> f64 {
    let q = op::trapezoid_weights(f.len());
    f.iter().zip(&q).map(|(a, b)| a * b).sum()
}

/// Weighted quantiles of x (P) with weights w (P) (NumPy-port interpolation of mid-point CDF).
pub fn weighted_quantiles(x: &[f64], w: &[f64], levels: &[f64]) -> Vec<f64> {
    let n = x.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|a, b| x[*a].total_cmp(&x[*b]));
    let xs: Vec<f64> = order.iter().map(|&i| x[i]).collect();
    let ws: Vec<f64> = order.iter().map(|&i| w[i]).collect();
    let tot: f64 = ws.iter().sum();
    let mut cw = vec![0.0; n];
    let mut acc = 0.0;
    for i in 0..n {
        acc += ws[i];
        cw[i] = (acc - 0.5 * ws[i]) / tot;
    }
    levels
        .iter()
        .map(|&q| {
            let cnt = cw.iter().filter(|c| **c < q).count() as isize - 1;
            let idx = cnt.clamp(0, n as isize - 2) as usize;
            let t = ((q - cw[idx]) / (cw[idx + 1] - cw[idx]).max(1e-300)).clamp(0.0, 1.0);
            xs[idx] + t * (xs[idx + 1] - xs[idx])
        })
        .collect()
}

/// A member's network initialisation (the scales of the Python reference; own RNG).
pub fn init_network(lay: &GlobalLayout, seed: u64) -> Vec<f64> {
    let mut p = vec![0.0; lay.len];
    let mut rng = StdRng::seed_from_u64(seed);
    let net: Vec<String> = op::param_shapes().into_iter().map(|b| b.0).collect();
    for (name, shape, off) in &lay.blocks {
        if !net.contains(name) {
            continue;
        }
        let len: usize = shape.iter().product();
        let scale = if name.ends_with(".b") || name.ends_with(".b1") || name.ends_with(".b2") {
            0.0
        } else if name.ends_with(".A") {
            1.0 / ((op::WIDTH * op::N_MODES) as f64).sqrt()
        } else if name == "film.W2" {
            0.01
        } else if name == "head.W" {
            0.1 / (op::WIDTH as f64).sqrt()
        } else {
            1.0 / (*shape.last().unwrap() as f64).sqrt()
        };
        for i in 0..len {
            p[off + i] = scale * rng.sample::<f64, _>(StandardNormal);
        }
    }
    p
}

pub fn numerics_from(cfg: &Value, forecast: bool) -> Numerics {
    let n = &cfg["numerics"];
    let scheme = match n["scheme"].as_str().unwrap_or("eno2") {
        "llf1" => dyn_::Scheme::Llf1,
        _ => dyn_::Scheme::Eno2,
    };
    let boundary = match n["boundary"].as_str().unwrap_or("linear") {
        "linear" => dyn_::Boundary::Linear,
        _ => dyn_::Boundary::Quadratic,
    };
    Numerics {
        cfl: n["cfl"].as_f64().unwrap_or(0.4),
        feedback: n["feedback"].as_f64().unwrap_or(0.25),
        dt_max_s: n["dt_max_s"].as_f64().unwrap_or(1.0),
        max_steps: n[if forecast {
            "forecast_max_steps"
        } else {
            "max_steps"
        }]
        .as_u64()
        .unwrap_or(if forecast { 128 } else { 32 }) as usize,
        scheme,
        boundary,
    }
}

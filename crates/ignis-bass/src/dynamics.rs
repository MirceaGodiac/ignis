//! Kinematic evolution of the visible front (V3.1 §9.5–§9.6). Not a reacting-flow solver.
//!
//!   ∂_t f = [v_b(s,c,b) + v_r tanh N_θ(f,c,G)] √(1 + (∂_s f)²) + ν_f ∂_ss f / (1 + (∂_s f)²),  ν_f ≥ 0
//!   v_b = v_ref [β_vᵀ(1,c) + Σ_{m=0}^{3} b_m φ_m(s/W)],  v_ref = v_r = 10⁻³ m/s,  ν_f = ν_ref e^ζ, ν_ref = 10⁻⁶ m²/s
//!   f_0(s) = L · sigmoid(Σ_{m=0}^{7} a_m φ_m(s/W))
//!
//! Spatial schemes (`Scheme`):
//! * `Llf1` — the previous implementation: one-sided first differences p∓, local Lax–Friedrichs
//!   Ĝ = V √(1 + p̄²) + (α/2)(p⁺ − p⁻) with α = max_stencil |V|, linear ghosts. First order in space
//!   (kept for parity with the Python reference).
//! * `Eno2` (default) — second-order ENO reconstruction of p∓ for Hamilton–Jacobi (Osher & Shu:
//!   p⁻ = D⁻f + ½Δs·minmod(D²f_{j−1}, D²f_j), p⁺ = D⁺f − ½Δs·minmod(D²f_j, D²f_{j+1})) with the local
//!   LF dissipation α = max_stencil|V| · ψ(max|p∓|), ψ(p) = p/√(1+p²) ≥ |∂H/∂p| (not |V|), so the
//!   dissipation is O(Δs²) on smooth fronts. Curvature is the central stencil in both.
//! Boundary ghosts (`Boundary`): linear extrapolation (documented convention of §9.6) or quadratic
//! extrapolation (third-order ghost values; the "justified alternative" for the refinement study).
//!
//! Time: SSP-RK2 (Heun); each stage re-evaluates the velocity on the current state and
//!   Δt = min( C / (α_max/Δs + 2ν_f/Δs²),  c_fb Δs / max|∂_t f|,  Δt_max,  t_target − t ),
//! taken without gradient (as `stop_gradient` in JAX). The step loop has a fixed maximum count; the
//! unreached time is returned and a non-zero value is a numerical refusal. Reverse mode recomputes
//! each stage from the stored stage states (checkpointing), so memory is O(steps · K).

use crate::operator::{self as op, Grid, NetOffsets, Tape};

pub const V_REF: f64 = 1.0e-3;
pub const NU_REF: f64 = 1.0e-6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scheme {
    Llf1,
    Eno2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Boundary {
    Linear,
    Quadratic,
}

#[derive(Debug, Clone, Copy)]
pub struct Numerics {
    pub cfl: f64,
    pub feedback: f64,
    pub dt_max_s: f64,
    pub max_steps: usize,
    pub scheme: Scheme,
    pub boundary: Boundary,
}

impl Default for Numerics {
    fn default() -> Self {
        Numerics {
            cfl: 0.4,
            feedback: 0.25,
            dt_max_s: 1.0,
            max_steps: 32,
            scheme: Scheme::Eno2,
            boundary: Boundary::Linear,
        }
    }
}

/// Offsets of the physics block in a member's flat vector.
#[derive(Debug, Clone, Default)]
pub struct PhysOffsets {
    pub beta_v: usize,
    pub zeta: usize,
}

// ---------------------------------------------------------------- transport

#[derive(Debug, Clone, Default)]
pub struct TransportTape {
    k: usize,
    ds: f64,
    nu: f64,
    e: Vec<f64>,
    pm: Vec<f64>,
    pp: Vec<f64>,
    fss: Vec<f64>,
    alpha: Vec<f64>,
    vmax: Vec<f64>,
    pmax: Vec<f64>,
    v: Vec<f64>,
    /// minmod selections (−1: zero, 0: first, 1: second) for p⁻ and p⁺
    sel_m: Vec<i8>,
    sel_p: Vec<i8>,
}

fn ghost_coeffs(b: Boundary) -> ([f64; 3], [f64; 3]) {
    // value at −1 and −2 as combinations of f0, f1, f2
    match b {
        Boundary::Linear => ([2.0, -1.0, 0.0], [3.0, -2.0, 0.0]),
        Boundary::Quadratic => ([3.0, -3.0, 1.0], [6.0, -8.0, 3.0]),
    }
}

fn minmod(a: f64, b: f64) -> (f64, i8) {
    if a * b <= 0.0 {
        (0.0, -1)
    } else if a.abs() <= b.abs() {
        (a, 0)
    } else {
        (b, 1)
    }
}

/// max with balanced-tie gradient weights (as `jnp.maximum`).
#[inline]
fn bmax(a: f64, b: f64) -> (f64, f64, f64) {
    if a > b {
        (a, 1.0, 0.0)
    } else if b > a {
        (b, 0.0, 1.0)
    } else {
        (a, 0.5, 0.5)
    }
}

/// Writes ∂_t f (K) for a given velocity V (K) and ν; returns max α.
pub fn transport(
    f: &[f64],
    v: &[f64],
    nu: f64,
    ds: f64,
    num: &Numerics,
    t: &mut TransportTape,
    g: &mut [f64],
) -> f64 {
    let k = f.len();
    t.k = k;
    t.ds = ds;
    t.nu = nu;
    t.e.resize(k + 4, 0.0);
    let (c1, c2) = ghost_coeffs(num.boundary);
    t.e[2..k + 2].copy_from_slice(f);
    t.e[1] = c1[0] * f[0] + c1[1] * f[1] + c1[2] * f[2.min(k - 1)];
    t.e[0] = c2[0] * f[0] + c2[1] * f[1] + c2[2] * f[2.min(k - 1)];
    t.e[k + 2] = c1[0] * f[k - 1] + c1[1] * f[k - 2] + c1[2] * f[k.saturating_sub(3)];
    t.e[k + 3] = c2[0] * f[k - 1] + c2[1] * f[k - 2] + c2[2] * f[k.saturating_sub(3)];
    for vec in [
        &mut t.pm,
        &mut t.pp,
        &mut t.fss,
        &mut t.alpha,
        &mut t.vmax,
        &mut t.pmax,
    ] {
        vec.resize(k, 0.0);
    }
    t.sel_m.resize(k, -1);
    t.sel_p.resize(k, -1);
    t.v.clear();
    t.v.extend_from_slice(v);
    let e = &t.e;
    let d = |i: usize| (e[i + 1] - e[i]) / ds;
    let dd = |i: usize| (e[i + 1] - 2.0 * e[i] + e[i - 1]) / (ds * ds);
    let ve = |i: usize| -> f64 {
        // |V| of the extended (duplicated-edge) array, i ∈ 0..k+2
        v[i.saturating_sub(1).min(k - 1)].abs()
    };
    let mut amax: f64 = 0.0;
    for j in 0..k {
        let x = j + 2;
        let mut pm = d(x - 1);
        let mut pp = d(x);
        if num.scheme == Scheme::Eno2 {
            let (mm, sm) = minmod(dd(x - 1), dd(x));
            let (mp, sp) = minmod(dd(x), dd(x + 1));
            pm += 0.5 * ds * mm;
            pp -= 0.5 * ds * mp;
            t.sel_m[j] = sm;
            t.sel_p[j] = sp;
        }
        t.pm[j] = pm;
        t.pp[j] = pp;
        t.fss[j] = dd(x);
        let (a1, _, _) = bmax(ve(j), ve(j + 1));
        let (vmax, _, _) = bmax(a1, ve(j + 2));
        t.vmax[j] = vmax;
        let alpha = match num.scheme {
            Scheme::Llf1 => vmax,
            Scheme::Eno2 => {
                let p = pm.abs().max(pp.abs());
                t.pmax[j] = p;
                vmax * p / (1.0 + p * p).sqrt()
            }
        };
        t.alpha[j] = alpha;
        amax = amax.max(alpha);
        let pbar = 0.5 * (pm + pp);
        let s = 1.0 + pbar * pbar;
        g[j] = v[j] * s.sqrt() + 0.5 * alpha * (pp - pm) + nu * t.fss[j] / s;
    }
    amax
}

/// Adjoint of [`transport`]: f̄ += , V̄ += , returns ν̄.
pub fn transport_backward(
    t: &TransportTape,
    num: &Numerics,
    gbar: &[f64],
    fbar: &mut [f64],
    vbar: &mut [f64],
) -> f64 {
    let k = t.k;
    let ds = t.ds;
    let mut ebar = vec![0.0; k + 4];
    let mut nubar = 0.0;
    let v = &t.v;
    let sgn = |x: f64| {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            0.0
        }
    };
    for j in 0..k {
        let gb = gbar[j];
        if gb == 0.0 {
            continue;
        }
        let x = j + 2;
        let (pm, pp) = (t.pm[j], t.pp[j]);
        let pbar = 0.5 * (pm + pp);
        let s = 1.0 + pbar * pbar;
        let fss = t.fss[j];
        vbar[j] += gb * s.sqrt();
        let pbar_bar = gb * (v[j] * pbar / s.sqrt() - t.nu * fss * 2.0 * pbar / (s * s));
        let alpha_bar = 0.5 * gb * (pp - pm);
        let mut pp_bar = 0.5 * gb * t.alpha[j] + 0.5 * pbar_bar;
        let mut pm_bar = -0.5 * gb * t.alpha[j] + 0.5 * pbar_bar;
        let fss_bar = gb * t.nu / s;
        nubar += gb * fss / s;
        // α
        let vmax_bar = match num.scheme {
            Scheme::Llf1 => alpha_bar,
            Scheme::Eno2 => {
                let p = t.pmax[j];
                let psi = p / (1.0 + p * p).sqrt();
                let dpsi = (1.0 + p * p).powf(-1.5);
                let pb = alpha_bar * t.vmax[j] * dpsi;
                // p = max(|pm|, |pp|) (balanced ties)
                let (_, wm, wp) = bmax(pm.abs(), pp.abs());
                pm_bar += pb * wm * sgn(pm);
                pp_bar += pb * wp * sgn(pp);
                alpha_bar * psi
            }
        };
        if vmax_bar != 0.0 {
            let idx = |i: usize| i.saturating_sub(1).min(k - 1);
            let ve = |i: usize| v[idx(i)].abs();
            let (a1, w0, w1) = bmax(ve(j), ve(j + 1));
            let (_, wa, w2) = bmax(a1, ve(j + 2));
            for (i, w) in [(j, wa * w0), (j + 1, wa * w1), (j + 2, w2)] {
                if w != 0.0 {
                    vbar[idx(i)] += vmax_bar * w * sgn(v[idx(i)]);
                }
            }
        }
        // p⁻ = D(x−1) [+ ½Δs mm(DD(x−1), DD(x))], p⁺ = D(x) [− ½Δs mm(DD(x), DD(x+1))]
        let add_d = |eb: &mut [f64], i: usize, w: f64| {
            eb[i + 1] += w / ds;
            eb[i] -= w / ds;
        };
        let add_dd = |eb: &mut [f64], i: usize, w: f64| {
            let c = w / (ds * ds);
            eb[i + 1] += c;
            eb[i] -= 2.0 * c;
            eb[i - 1] += c;
        };
        add_d(&mut ebar, x - 1, pm_bar);
        add_d(&mut ebar, x, pp_bar);
        add_dd(&mut ebar, x, fss_bar);
        if num.scheme == Scheme::Eno2 {
            match t.sel_m[j] {
                0 => add_dd(&mut ebar, x - 1, 0.5 * ds * pm_bar),
                1 => add_dd(&mut ebar, x, 0.5 * ds * pm_bar),
                _ => {}
            }
            match t.sel_p[j] {
                0 => add_dd(&mut ebar, x, -0.5 * ds * pp_bar),
                1 => add_dd(&mut ebar, x + 1, -0.5 * ds * pp_bar),
                _ => {}
            }
        }
    }
    let (c1, c2) = ghost_coeffs(num.boundary);
    for j in 0..k {
        fbar[j] += ebar[j + 2];
    }
    for (gidx, c) in [(1usize, c1), (0usize, c2)] {
        for (i, w) in c.iter().enumerate() {
            fbar[i.min(k - 1)] += w * ebar[gidx];
        }
    }
    for (gidx, c) in [(k + 2, c1), (k + 3, c2)] {
        for (i, w) in c.iter().enumerate() {
            fbar[k - 1 - i.min(k - 1)] += w * ebar[gidx];
        }
    }
    nubar
}

// ---------------------------------------------------------------- full right-hand side

/// Everything one right-hand-side evaluation needs (shared by every member and run).
pub struct Ctx<'a> {
    pub p: &'a [f64],
    pub net: &'a NetOffsets,
    pub phys: &'a PhysOffsets,
    pub grid: &'a Grid,
    pub num: Numerics,
    pub l: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Default)]
pub struct RhsTape {
    pub op: Tape,
    pub tr: TransportTape,
    pub tanh_n: Vec<f64>,
    pub v: Vec<f64>,
    pub alpha_max: f64,
}

/// ∂_t f at every node (into `g`) for local velocity coefficients `b` (4) and conditions `c` (8).
pub fn rhs(cx: &Ctx, b: &[f64], f: &[f64], c: &[f64], t: &mut RhsTape, g: &mut [f64]) {
    let k = f.len();
    op::channels(f, cx.l, cx.grid, &mut t.op.chan);
    op::forward(cx.p, cx.net, cx.grid, c, &mut t.op);
    let beta = &cx.p[cx.phys.beta_v..cx.phys.beta_v + 1 + op::N_CONDITIONS];
    let mut lin = beta[0];
    for i in 0..op::N_CONDITIONS {
        lin += beta[1 + i] * c[i];
    }
    t.tanh_n.resize(k, 0.0);
    t.v.resize(k, 0.0);
    for j in 0..k {
        let mut vb = lin;
        for m in 0..op::N_VELOCITY_COEFFS {
            vb += cx.grid.phi_v[j * op::N_VELOCITY_COEFFS + m] * b[m];
        }
        t.tanh_n[j] = t.op.out[j].tanh();
        t.v[j] = V_REF * vb + V_REF * t.tanh_n[j];
    }
    let nu = NU_REF * cx.p[cx.phys.zeta].exp();
    let ds = cx.w / (k - 1) as f64;
    let v = std::mem::take(&mut t.v);
    t.alpha_max = transport(f, &v, nu, ds, &cx.num, &mut t.tr, g);
    t.v = v;
}

#[derive(Default)]
pub struct Scratch {
    vbar: Vec<f64>,
    nbar: Vec<f64>,
    chan_bar: Vec<f64>,
    op: Vec<f64>,
}

/// Adjoint of [`rhs`]: adds θ̄ into `gp`, b̄ into `bbar`, f̄ into `fbar`.
#[allow(clippy::too_many_arguments)]
pub fn rhs_backward(
    cx: &Ctx,
    c: &[f64],
    t: &RhsTape,
    gbar: &[f64],
    gp: &mut [f64],
    bbar: &mut [f64],
    fbar: &mut [f64],
    s: &mut Scratch,
) {
    let k = gbar.len();
    s.vbar.clear();
    s.vbar.resize(k, 0.0);
    let nubar = transport_backward(&t.tr, &cx.num, gbar, fbar, &mut s.vbar);
    let nu = NU_REF * cx.p[cx.phys.zeta].exp();
    gp[cx.phys.zeta] += nubar * nu;
    let vsum: f64 = s.vbar.iter().sum();
    gp[cx.phys.beta_v] += V_REF * vsum;
    for i in 0..op::N_CONDITIONS {
        gp[cx.phys.beta_v + 1 + i] += V_REF * vsum * c[i];
    }
    s.nbar.resize(k, 0.0);
    for j in 0..k {
        for m in 0..op::N_VELOCITY_COEFFS {
            bbar[m] += V_REF * cx.grid.phi_v[j * op::N_VELOCITY_COEFFS + m] * s.vbar[j];
        }
        s.nbar[j] = s.vbar[j] * V_REF * (1.0 - t.tanh_n[j] * t.tanh_n[j]);
    }
    op::backward(
        cx.p,
        cx.net,
        cx.grid,
        c,
        &t.op,
        &s.nbar,
        gp,
        &mut s.chan_bar,
        &mut s.op,
    );
    op::channels_backward(&s.chan_bar, k, cx.l, fbar);
}

pub fn stable_dt(rate: &[f64], alpha_max: f64, nu: f64, ds: f64, num: &Numerics) -> f64 {
    let dt_cfl = num.cfl / (alpha_max / ds + 2.0 * nu / (ds * ds) + 1e-30);
    let rmax = rate.iter().fold(0.0f64, |a, b| a.max(b.abs()));
    let dt_fb = num.feedback * ds / (rmax + 1e-30);
    dt_cfl.min(dt_fb).min(num.dt_max_s)
}

pub fn initial_front(a: &[f64], l: f64, grid: &Grid, f: &mut [f64]) {
    for j in 0..grid.k {
        let s: f64 = (0..op::N_FRONT_COEFFS)
            .map(|m| grid.phi_a[j * op::N_FRONT_COEFFS + m] * a[m])
            .sum();
        f[j] = l / (1.0 + (-s).exp());
    }
}

/// Condition vector at time t (normalised; time-since-control entries evaluated at t).
pub trait Conditions {
    fn at(&self, t: f64, out: &mut [f64]);
}

/// Forward trajectory record for the adjoint.
#[derive(Debug, Clone, Default)]
pub struct Trajectory {
    /// frame states (T × K)
    pub frames: Vec<f64>,
    /// per step: (frame interval index, t, dt) and the stage-0 state (K)
    pub steps: Vec<(usize, f64, f64)>,
    pub states: Vec<f64>,
    pub unreached: f64,
    pub max_cfl: f64,
}

#[derive(Default)]
pub struct SimWork {
    pub tape: RhsTape,
    pub tape1: RhsTape,
    pub r0: Vec<f64>,
    pub r1: Vec<f64>,
    pub f: Vec<f64>,
    pub f1: Vec<f64>,
    pub c: Vec<f64>,
    pub scratch: Scratch,
}

/// Front at every time in `times`, starting from f_0(a) at times[0]. With `record`, the stage states
/// are kept for [`simulate_backward`].
pub fn simulate(
    cx: &Ctx,
    a: &[f64],
    b: &[f64],
    times: &[f64],
    cond: &dyn Conditions,
    record: bool,
    w: &mut SimWork,
    tr: &mut Trajectory,
) {
    let k = cx.grid.k;
    let ds = cx.w / (k - 1) as f64;
    let nu = NU_REF * cx.p[cx.phys.zeta].exp();
    w.f.resize(k, 0.0);
    w.f1.resize(k, 0.0);
    w.r0.resize(k, 0.0);
    w.r1.resize(k, 0.0);
    w.c.resize(op::N_CONDITIONS, 0.0);
    tr.frames.clear();
    tr.steps.clear();
    tr.states.clear();
    tr.unreached = 0.0;
    tr.max_cfl = 0.0;
    initial_front(a, cx.l, cx.grid, &mut w.f);
    tr.frames.extend_from_slice(&w.f);
    for fi in 1..times.len() {
        let mut t = times[fi - 1];
        let target = times[fi];
        for _ in 0..cx.num.max_steps {
            let remaining = (target - t).max(0.0);
            if remaining == 0.0 {
                break; // a zero step is the identity (values and adjoints)
            }
            cond.at(t, &mut w.c);
            rhs(cx, b, &w.f, &w.c, &mut w.tape, &mut w.r0);
            let dt = stable_dt(&w.r0, w.tape.alpha_max, nu, ds, &cx.num).min(remaining);
            tr.max_cfl = tr.max_cfl.max(dt * w.tape.alpha_max / ds);
            if record {
                tr.steps.push((fi, t, dt));
                tr.states.extend_from_slice(&w.f);
            }
            for j in 0..k {
                w.f1[j] = w.f[j] + dt * w.r0[j];
            }
            cond.at(t + dt, &mut w.c);
            rhs(cx, b, &w.f1, &w.c, &mut w.tape, &mut w.r1);
            for j in 0..k {
                w.f[j] = 0.5 * (w.f[j] + w.f1[j] + dt * w.r1[j]);
            }
            t += dt;
        }
        tr.unreached = tr.unreached.max((target - t).max(0.0));
        tr.frames.extend_from_slice(&w.f);
    }
}

/// Reverse pass of [`simulate`]: `frames_bar` (T × K) adjoints of the frame states → θ̄ (into `gp`),
/// ā (8) and b̄ (4).
#[allow(clippy::too_many_arguments)]
pub fn simulate_backward(
    cx: &Ctx,
    a: &[f64],
    b: &[f64],
    cond: &dyn Conditions,
    tr: &Trajectory,
    frames_bar: &[f64],
    gp: &mut [f64],
    abar: &mut [f64],
    bbar: &mut [f64],
    w: &mut SimWork,
) {
    let k = cx.grid.k;
    let nt = frames_bar.len() / k;
    let mut fb = frames_bar[(nt - 1) * k..nt * k].to_vec();
    let mut f1b = vec![0.0; k];
    let mut rb = vec![0.0; k];
    w.f1.resize(k, 0.0);
    w.r0.resize(k, 0.0);
    w.r1.resize(k, 0.0);
    w.c.resize(op::N_CONDITIONS, 0.0);
    let mut si = tr.steps.len();
    for fi in (1..nt).rev() {
        // steps of interval fi (stored in order)
        while si > 0 && tr.steps[si - 1].0 == fi {
            si -= 1;
            let (_, t, dt) = tr.steps[si];
            let f0 = &tr.states[si * k..(si + 1) * k];
            // recompute stage 0 and stage 1 (tapes kept for both)
            cond.at(t, &mut w.c);
            let c0 = w.c.clone();
            rhs(cx, b, f0, &c0, &mut w.tape, &mut w.r0);
            for j in 0..k {
                w.f1[j] = f0[j] + dt * w.r0[j];
            }
            cond.at(t + dt, &mut w.c);
            rhs(cx, b, &w.f1, &w.c, &mut w.tape1, &mut w.r1);
            // f_new = ½(f + f1 + dt r1)
            for j in 0..k {
                f1b[j] = 0.5 * fb[j];
                rb[j] = 0.5 * dt * fb[j];
                fb[j] *= 0.5;
            }
            rhs_backward(cx, &w.c, &w.tape1, &rb, gp, bbar, &mut f1b, &mut w.scratch);
            // f1 = f + dt r0
            for j in 0..k {
                fb[j] += f1b[j];
                rb[j] = dt * f1b[j];
            }
            rhs_backward(cx, &c0, &w.tape, &rb, gp, bbar, &mut fb, &mut w.scratch);
        }
        for j in 0..k {
            fb[j] += frames_bar[(fi - 1) * k + j];
        }
    }
    // f0 = L σ(Φ a)
    let mut s = vec![0.0; k];
    initial_front(a, cx.l, cx.grid, &mut s);
    for j in 0..k {
        let sig = s[j] / cx.l;
        let d = fb[j] * cx.l * sig * (1.0 - sig);
        for m in 0..op::N_FRONT_COEFFS {
            abar[m] += cx.grid.phi_a[j * op::N_FRONT_COEFFS + m] * d;
        }
    }
}

impl RhsTape {
    /// Local Lax–Friedrichs speeds of the last evaluation (inspection, tests).
    pub fn tr_alpha(&self) -> Vec<f64> {
        self.tr.alpha.clone()
    }
}

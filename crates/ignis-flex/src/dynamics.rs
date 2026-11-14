//! FLEX dynamics, separable integration, hazards and the per-run likelihood (V3.1 §6.2–6.6, ch. 7).
//!
//! ```text
//! dq/dt = -K_ref exp[z(c)'beta + theta1 w + theta2 w^2 + theta3 w^3 + b0 + b1 w + R_phi(c, w)],
//! q = D^2,  r = q/q0,  w = 1 - r.
//! ```
//!
//! Separable integration in r (dt/dr < 0: positive time integrals use reversed limits and the
//! positive Jacobian |dt/du|, §23.2):
//!
//! ```text
//! t(r) = A int_r^1 exp[-g(1-u) - b1 (1-u) - R(c, 1-u)] du,   A = (q0/K_ref) exp(-z'beta - b0).
//! ```
//!
//! Cumulative hazard to t(r): Lambda_H(r) = int_r^1 lambda_H(w(u)) |dt/du| du; lambda_R constant.
//! Both are the two columns of one panel table per (draw, run).
//!
//! Everything is generic over [`Real`]: `f64` for values, [`ignis_science::ad::Var`] for gradients.
//! The integrand sums are one hand-differentiated tape node per Gauss–Legendre rule
//! ([`Integ::gl`]); the rest is recorded op by op.

use ignis_science::ad::Real;
use ignis_science::integrate::{build_table, Integrand, PanelTable, Rule};
use ignis_science::studentt::{kalman_predict, ou_kalman, student_t_from_terms, Kalman};
use ignis_science::transforms::Layout;

use crate::structure::{FeatureSpec, Structure, D_REF_M, K_REF_M2_S, T_REF_S};

pub const MAX_HIDDEN: usize = 32;
const MAX_PARENTS: usize = 8 + 4 * MAX_HIDDEN;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Numerics {
    pub n_panels: usize,
    pub n_nodes: usize,
    pub newton_steps: usize,
    pub event_nodes: usize,
    /// measurement/model floor q_min = d_min^2 [IGNIS, declared]
    pub d_min_m: f64,
    /// model horizon T_valid [IGNIS, declared]
    pub t_valid_s: f64,
    pub nu: f64,
}

impl Default for Numerics {
    fn default() -> Self {
        Numerics {
            n_panels: 16,
            n_nodes: 8,
            newton_steps: 8,
            event_nodes: 16,
            d_min_m: 2.5e-4,
            t_valid_s: 30.0,
            nu: 4.0,
        }
    }
}

/// Offsets of the global blocks in the flat global vector.
#[derive(Debug, Clone, Default)]
pub struct GOff {
    pub beta: Option<usize>,
    pub theta: Option<usize>,
    pub b_log_sigma: Option<usize>,
    pub b_atanh_rho: Option<usize>,
    pub gamma0: Option<usize>,
    pub gamma: Option<usize>,
    pub h_a: Option<usize>,
    pub link: Option<usize>,
    pub zeta: Option<usize>,
    pub sc: usize,
    pub sn: usize,
    pub ell: usize,
    pub w1: Option<usize>,
    pub a1: Option<usize>,
    pub w2: Option<usize>,
    pub a2: Option<usize>,
}

impl GOff {
    pub fn new(l: &Layout) -> GOff {
        let o = |n: &str| l.range(n).map(|r| r.start);
        GOff {
            beta: o("beta"),
            theta: o("theta"),
            b_log_sigma: o("b.log_sigma"),
            b_atanh_rho: o("b.atanh_rho"),
            gamma0: o("hazard.gamma0"),
            gamma: o("hazard.gamma"),
            h_a: o("hazard.a"),
            link: o("hazard.link"),
            zeta: o("hazard.zeta"),
            sc: o("obs.log_sigma_c").expect("obs block"),
            sn: o("obs.log_sigma_n").expect("obs block"),
            ell: o("obs.log_ell").expect("obs block"),
            w1: o("residual.W1"),
            a1: o("residual.a1"),
            w2: o("residual.W2"),
            a2: o("residual.a2"),
        }
    }
}

/// Static model description shared by every run of one fit / release.
#[derive(Debug, Clone)]
pub struct Model {
    pub st: Structure,
    pub fs: FeatureSpec,
    pub num: Numerics,
    pub layout: Layout,
    pub off: GOff,
    pub c_idx: Vec<usize>,
    pub z_idx: Vec<usize>,
}

impl Model {
    pub fn new(st: Structure, fs: FeatureSpec, num: Numerics, layout: Layout) -> Model {
        assert!(
            st.hidden <= MAX_HIDDEN,
            "residual hidden width > {MAX_HIDDEN}"
        );
        let off = GOff::new(&layout);
        let (c_idx, z_idx) = (fs.c_idx(), fs.z_idx());
        Model {
            st,
            fs,
            num,
            layout,
            off,
            c_idx,
            z_idx,
        }
    }
    pub fn n_g(&self) -> usize {
        self.layout.size
    }
    pub fn n_b(&self) -> usize {
        self.st.n_b()
    }
}

/// Natural globals used outside the dynamics: Cholesky factor of the b covariance and the kernel.
pub struct Natural<R> {
    pub l_b: [[R; 2]; 2],
    pub sigma_c: R,
    pub sigma_n: R,
    pub ell: R,
}

pub fn natural_globals<R: Real>(m: &Model, p: &[R]) -> Natural<R> {
    let z = R::cst(0.0);
    let mut l_b = [[z; 2]; 2];
    match m.st.b0.as_str() {
        "hier" => {
            let o = m.off.b_log_sigma.unwrap();
            let s0 = p[o].exp();
            l_b[0][0] = s0;
            if m.n_b() == 2 {
                let s1 = p[o + 1].exp();
                let rho = p[m.off.b_atanh_rho.unwrap()].tanh();
                l_b[1][0] = s1 * rho;
                l_b[1][1] = s1 * (R::cst(1.0) - rho * rho).sqrt();
            }
        }
        "fixed" => l_b[0][0] = R::cst(m.st.b0_fixed_scale),
        _ => {}
    }
    Natural {
        l_b,
        sigma_c: p[m.off.sc].exp(),
        sigma_n: p[m.off.sn].exp(),
        ell: p[m.off.ell].exp(),
    }
}

/// Integrand parameters of one (draw, run).
#[derive(Debug, Clone)]
pub struct Integ<R> {
    pub log_a: R,
    pub theta: [R; 3],
    pub b1: R,
    /// Residual: pre-activations with c (pc) and with c = 0 (p0 = a1), w-column of W1, W2, a2.
    pub hidden: usize,
    pub pc: Vec<R>,
    pub p0: Vec<R>,
    pub w1w: Vec<R>,
    pub w2: Vec<R>,
    pub a2: R,
    pub residual: bool,
    pub hazards: bool,
    pub base_h: R,
    pub ha: [R; 2],
    // f64 copies for the value path
    v: IntegV,
}

#[derive(Debug, Clone, Default)]
struct IntegV {
    log_a: f64,
    theta: [f64; 3],
    b1: f64,
    pc: Vec<f64>,
    p0: Vec<f64>,
    w1w: Vec<f64>,
    w2: Vec<f64>,
    a2: f64,
    base_h: f64,
    ha: [f64; 2],
}

/// tanh through expm1 (|rel err| < 1e-15, ~30% faster than the CRT tanh on MSVC).
#[inline(always)]
fn tanh_fast(x: f64) -> f64 {
    if x.abs() > 20.0 {
        x.signum()
    } else {
        let e = (2.0 * x).exp_m1();
        e / (e + 2.0)
    }
}

/// Value, d/du and partials (w.r.t. the parent list of [`Integ::parents`]) of both components.
struct Node {
    f: [f64; 2],
    dfdu: [f64; 2],
}

impl<R: Real> Integ<R> {
    fn snapshot(&mut self) {
        let v = |x: &[R]| x.iter().map(|y| y.val()).collect::<Vec<_>>();
        self.v = IntegV {
            log_a: self.log_a.val(),
            theta: self.theta.map(|x| x.val()),
            b1: self.b1.val(),
            pc: v(&self.pc),
            p0: v(&self.p0),
            w1w: v(&self.w1w),
            w2: v(&self.w2),
            a2: self.a2.val(),
            base_h: self.base_h.val(),
            ha: self.ha.map(|x| x.val()),
        };
    }

    /// Parent order: log_a, theta(3), b1, [pc(H), p0(H), w1w(H), w2(H), a2], [base_h, ha(2)].
    fn parents(&self, out: &mut Vec<R>) {
        out.clear();
        out.push(self.log_a);
        out.extend_from_slice(&self.theta);
        out.push(self.b1);
        if self.residual {
            out.extend_from_slice(&self.pc);
            out.extend_from_slice(&self.p0);
            out.extend_from_slice(&self.w1w);
            out.extend_from_slice(&self.w2);
            out.push(self.a2);
        }
        if self.hazards {
            out.push(self.base_h);
            out.extend_from_slice(&self.ha);
        }
    }

    /// Evaluate the integrand at u; when `d` is given, also the partials (2 x n_parents, row-major
    /// by component: d[0..np] for component 0, d[np..2np] for component 1).
    #[inline]
    fn node(&self, u: f64, d: Option<(&mut [f64], usize)>) -> Node {
        let v = &self.v;
        let w = 1.0 - u;
        let h = self.hidden;
        let (mut r, mut dr) = (0.0, 0.0);
        let mut tc = [0.0; MAX_HIDDEN];
        let mut t0 = [0.0; MAX_HIDDEN];
        let (mut hc, mut h0) = (0.0, 0.0);
        if self.residual {
            let (mut sc, mut s0) = (v.a2, v.a2);
            let (mut dsc, mut ds0) = (0.0, 0.0);
            for j in 0..h {
                tc[j] = tanh_fast(v.pc[j] + v.w1w[j] * w);
                t0[j] = tanh_fast(v.p0[j] + v.w1w[j] * w);
                sc += v.w2[j] * tc[j];
                s0 += v.w2[j] * t0[j];
                dsc += v.w2[j] * (1.0 - tc[j] * tc[j]) * v.w1w[j];
                ds0 += v.w2[j] * (1.0 - t0[j] * t0[j]) * v.w1w[j];
            }
            hc = tanh_fast(sc);
            h0 = tanh_fast(s0);
            r = 0.5 * w * (hc - h0);
            dr = 0.5 * (hc - h0) + 0.5 * w * ((1.0 - hc * hc) * dsc - (1.0 - h0 * h0) * ds0);
        }
        let (w2_, w3) = (w * w, w * w * w);
        let s = v.theta[0] * w + v.theta[1] * w2_ + v.theta[2] * w3 + v.b1 * w + r;
        let ds = v.theta[0] + 2.0 * v.theta[1] * w + 3.0 * v.theta[2] * w2_ + v.b1 + dr;
        let g = (v.log_a - s).exp();
        let dg_dw = -g * ds;
        let (lam, dlam_dw) = if self.hazards {
            let l = (v.base_h + v.ha[0] * w + v.ha[1] * w2_).exp();
            (l, l * (v.ha[0] + 2.0 * v.ha[1] * w))
        } else {
            (0.0, 0.0)
        };
        let f1 = lam * g;
        let node = Node {
            f: [g, f1],
            dfdu: [-dg_dw, -(dlam_dw * g + lam * dg_dw)],
        };
        if let Some((d, np)) = d {
            // d f0 / d param = g * dlog g/d param ; f1 = lam * g
            let mut k = 0;
            let mut put = |k: usize, dg: f64, dl: f64| {
                // dg: d log g; dl: d log lam (hazard params only)
                d[k] += g * dg;
                d[np + k] += f1 * (dg + dl);
            };
            put(k, 1.0, 0.0);
            k += 1;
            put(k, -w, 0.0);
            put(k + 1, -w2_, 0.0);
            put(k + 2, -w3, 0.0);
            k += 3;
            put(k, -w, 0.0);
            k += 1;
            if self.residual {
                let (gc, g0) = (1.0 - hc * hc, 1.0 - h0 * h0);
                for j in 0..h {
                    let ac = gc * v.w2[j] * (1.0 - tc[j] * tc[j]);
                    let a0 = g0 * v.w2[j] * (1.0 - t0[j] * t0[j]);
                    put(k + j, -0.5 * w * ac, 0.0);
                    put(k + h + j, 0.5 * w * a0, 0.0);
                    put(k + 2 * h + j, -0.5 * w * (ac - a0) * w, 0.0);
                    put(k + 3 * h + j, -0.5 * w * (gc * tc[j] - g0 * t0[j]), 0.0);
                }
                k += 4 * h;
                put(k, -0.5 * w * (gc - g0), 0.0);
                k += 1;
            }
            if self.hazards {
                put(k, 0.0, 1.0);
                put(k + 1, 0.0, w);
                put(k + 2, 0.0, w2_);
            }
        }
        node
    }

    fn n_parents(&self) -> usize {
        5 + if self.residual {
            4 * self.hidden + 1
        } else {
            0
        } + if self.hazards { 3 } else { 0 }
    }

    /// f at a single (differentiable) point.
    pub fn at(&self, u: R) -> [R; 2] {
        let uv = u.val();
        if !R::GRAD {
            let n = self.node(uv, None);
            return [R::cst(n.f[0]), R::cst(n.f[1])];
        }
        let np = self.n_parents();
        let mut d = [0.0; 2 * MAX_PARENTS + 2];
        let n = self.node(uv, Some((&mut d[..2 * np], np)));
        let mut ps = Vec::with_capacity(np + 1);
        self.parents(&mut ps);
        ps.push(u);
        let mut p0 = d[..np].to_vec();
        p0.push(n.dfdu[0]);
        let mut p1 = d[np..2 * np].to_vec();
        p1.push(n.dfdu[1]);
        [R::custom(n.f[0], &ps, &p0), R::custom(n.f[1], &ps, &p1)]
    }

    /// log lambda_H at w.
    pub fn log_lam_h(&self, w: R) -> R {
        if !self.hazards {
            return R::cst(f64::NEG_INFINITY);
        }
        self.base_h + self.ha[0] * w + self.ha[1] * w * w
    }

    pub fn log_shape(&self, w: f64) -> f64 {
        let v = &self.v;
        let mut s = v.theta[0] * w + v.theta[1] * w * w + v.theta[2] * w * w * w + v.b1 * w;
        if self.residual {
            let (mut sc, mut s0) = (v.a2, v.a2);
            for j in 0..self.hidden {
                sc += v.w2[j] * (v.pc[j] + v.w1w[j] * w).tanh();
                s0 += v.w2[j] * (v.p0[j] + v.w1w[j] * w).tanh();
            }
            s += 0.5 * w * (sc.tanh() - s0.tanh());
        }
        s
    }
}

impl<R: Real> Integrand<R, 2> for Integ<R> {
    #[inline]
    fn value(&self, u: f64) -> [f64; 2] {
        self.node(u, None).f
    }

    fn gl_value(&self, a: f64, b: f64, rule: &Rule) -> [f64; 2] {
        let mid = 0.5 * (a + b);
        let half = 0.5 * (b - a);
        let mut acc = [0.0; 2];
        for i in 0..rule.len() {
            let f = self.node(mid + half * rule.x[i], None).f;
            let wt = half * rule.w[i];
            acc[0] += wt * f[0];
            acc[1] += wt * f[1];
        }
        acc
    }

    fn gl(&self, a: R, b: R, rule: &Rule) -> [R; 2] {
        let (av, bv) = (a.val(), b.val());
        if !R::GRAD {
            let v = self.gl_value(av, bv, rule);
            return [R::cst(v[0]), R::cst(v[1])];
        }
        let np = self.n_parents();
        let mid = 0.5 * (av + bv);
        let half = 0.5 * (bv - av);
        let mut d = [0.0; 2 * MAX_PARENTS];
        let mut tmp = [0.0; 2 * MAX_PARENTS];
        let (mut v0, mut v1) = (0.0, 0.0);
        let (mut da0, mut da1, mut db0, mut db1) = (0.0, 0.0, 0.0, 0.0);
        for i in 0..rule.len() {
            let x = rule.x[i];
            let wt = half * rule.w[i];
            tmp[..2 * np].iter_mut().for_each(|t| *t = 0.0);
            let n = self.node(mid + half * x, Some((&mut tmp[..2 * np], np)));
            v0 += wt * n.f[0];
            v1 += wt * n.f[1];
            for k in 0..2 * np {
                d[k] += wt * tmp[k];
            }
            let (ua, ub) = (0.5 * (1.0 - x), 0.5 * (1.0 + x));
            da0 += -0.5 * rule.w[i] * n.f[0] + wt * n.dfdu[0] * ua;
            da1 += -0.5 * rule.w[i] * n.f[1] + wt * n.dfdu[1] * ua;
            db0 += 0.5 * rule.w[i] * n.f[0] + wt * n.dfdu[0] * ub;
            db1 += 0.5 * rule.w[i] * n.f[1] + wt * n.dfdu[1] * ub;
        }
        let mut ps = Vec::with_capacity(np + 2);
        self.parents(&mut ps);
        ps.push(a);
        ps.push(b);
        let mut p0 = [0.0; MAX_PARENTS + 2];
        let mut p1 = [0.0; MAX_PARENTS + 2];
        p0[..np].copy_from_slice(&d[..np]);
        p1[..np].copy_from_slice(&d[np..2 * np]);
        p0[np] = da0;
        p0[np + 1] = db0;
        p1[np] = da1;
        p1[np + 1] = db1;
        [
            R::custom(v0, &ps, &p0[..np + 2]),
            R::custom(v1, &ps, &p1[..np + 2]),
        ]
    }
}

/// Everything a run needs under one draw: table, D0, hazards, support boundary.
#[derive(Debug, Clone)]
pub struct RunState<R> {
    pub integ: Integ<R>,
    pub table: PanelTable<R, 2>,
    pub d0: R,
    pub c: Vec<R>,
    pub b0: R,
    pub b1: R,
    pub lam_r: R,
    pub t_rmin: R,
    pub b: R,
    pub r_min: R,
}

fn rmin<R: Real>(x: R, lo: f64, hi: f64) -> R {
    if x.val() < lo {
        R::cst(lo)
    } else if x.val() > hi {
        R::cst(hi)
    } else {
        x
    }
}

/// Build the per-(draw, run) state. ck: known features (NaN-free), eps_b: (n_b,), log_d0.
pub fn run_state<R: Real>(
    m: &Model,
    p: &[R],
    ck: &[f64; 4],
    eps_b: &[R],
    log_d0: R,
) -> RunState<R> {
    let (st, o) = (&m.st, &m.off);
    let zero = R::cst(0.0);
    let mut c_full = [zero; 5];
    for j in 0..4 {
        c_full[j] = R::cst(if m.fs.c_active[j] { ck[j] } else { 0.0 });
    }
    c_full[4] = if m.fs.c_active[4] {
        log_d0 - D_REF_M.ln()
    } else {
        zero
    };
    let c: Vec<R> = m.c_idx.iter().map(|&i| c_full[i]).collect();
    let lin = if st.z_mode == "full" {
        let ob = o.beta.unwrap();
        let mut acc = zero;
        for (k, &zi) in m.z_idx.iter().enumerate() {
            let z = match zi {
                0 => R::cst(1.0),
                1..=5 => c_full[zi - 1],
                6 => c_full[0] * c_full[0],
                7 => c_full[0] * c_full[1],
                _ => c_full[0] * c_full[3],
            };
            acc += z * p[ob + k];
        }
        acc
    } else {
        R::cst(st.beta0_fixed)
    };
    let nat = natural_globals(m, p);
    let (b0, b1) = match m.n_b() {
        0 => (zero, zero),
        1 => (nat.l_b[0][0] * eps_b[0], zero),
        _ => (
            nat.l_b[0][0] * eps_b[0],
            nat.l_b[1][0] * eps_b[0] + nat.l_b[1][1] * eps_b[1],
        ),
    };
    let d0 = log_d0.exp();
    let log_a = log_d0 * 2.0 - K_REF_M2_S.ln() - lin - b0;
    let theta = match o.theta {
        Some(t) => [p[t], p[t + 1], p[t + 2]],
        None => [zero; 3],
    };
    let h = st.hidden;
    let nc = c.len();
    let (mut pc, mut p0, mut w1w, mut w2, mut a2) = (vec![], vec![], vec![], vec![], zero);
    if st.residual {
        let (ow1, oa1, ow2) = (o.w1.unwrap(), o.a1.unwrap(), o.w2.unwrap());
        let nin = nc + 1;
        for j in 0..h {
            let mut acc = p[oa1 + j];
            for k in 0..nc {
                acc += p[ow1 + j * nin + k] * c[k];
            }
            pc.push(acc);
            p0.push(p[oa1 + j]);
            w1w.push(p[ow1 + j * nin + nc]);
            w2.push(p[ow2 + j]);
        }
        a2 = p[o.a2.unwrap()];
    }
    let (base_h, ha, lam_r) = if st.hazards {
        let og = o.gamma.unwrap();
        let mut bh = p[o.gamma0.unwrap()];
        for k in 0..nc {
            bh += p[og + k] * c[k];
        }
        let ol = o.link.unwrap();
        bh = bh + p[ol] * b0 + p[ol + 1] * b1 - T_REF_S.ln();
        let oa = o.h_a.unwrap();
        (bh, [p[oa], p[oa + 1]], p[o.zeta.unwrap()].exp() / T_REF_S)
    } else {
        (zero, [zero; 2], zero)
    };
    let mut integ = Integ {
        log_a,
        theta,
        b1,
        hidden: h,
        pc,
        p0,
        w1w,
        w2,
        a2,
        residual: st.residual,
        hazards: st.hazards,
        base_h,
        ha,
        v: IntegV::default(),
    };
    integ.snapshot();
    let ratio = R::cst(m.num.d_min_m) / d0;
    let r_min = rmin(ratio * ratio, 1e-12, 1.0 - 1e-9);
    let table = build_table(&integ, r_min, R::cst(1.0), m.num.n_panels, m.num.n_nodes);
    let t_rmin = table.total()[0];
    let b = if t_rmin.val() < m.num.t_valid_s {
        t_rmin
    } else {
        R::cst(m.num.t_valid_s)
    };
    RunState {
        integ,
        table,
        d0,
        c,
        b0,
        b1,
        lam_r,
        t_rmin,
        b,
        r_min,
    }
}

impl<R: Real> RunState<R> {
    /// Inverse trajectory r(t) for t in [0, t(r_min)] (implicit gradient).
    pub fn r_of_t(&self, num: &Numerics, t: R) -> R {
        self.table
            .invert_decreasing(&self.integ, t, num.newton_steps, 0)
    }
    /// (log S at t(u), t(u)): -(Lambda_H(u) + lambda_R t(u)).
    pub fn log_survival_at_u(&self, u: R) -> (R, R) {
        let v = self.table.upper(&self.integ, u);
        (-(v[1] + self.lam_r * v[0]), v[0])
    }
    /// (P_H, P_R) = int over t(u_hi) -> t(u_lo) of S(t) lambda_k(t) dt, computed in u on [u_lo, u_hi].
    pub fn cause_integral(&self, num: &Numerics, u_lo: R, u_hi: R, cause_mask: [f64; 2]) -> [R; 2] {
        let rule = Rule::cached(num.event_nodes);
        let mid = (u_lo + u_hi) * 0.5;
        let half = (u_hi - u_lo) * 0.5;
        let (mut ph, mut pr) = (R::cst(0.0), R::cst(0.0));
        for i in 0..rule.len() {
            let u = mid + half * rule.x[i];
            let wt = half * rule.w[i];
            let (log_s, _) = self.log_survival_at_u(u);
            let fu = self.integ.at(u);
            let s = log_s.exp();
            ph += wt * s * fu[1];
            pr += wt * s * self.lam_r * fu[0];
        }
        [ph * cause_mask[0], pr * cause_mask[1]]
    }
}

/// One run's data (masked, never zero-filled values).
#[derive(Debug, Clone, Default)]
pub struct RunData {
    pub t: Vec<f64>,
    pub y: Vec<f64>,
    pub mask: Vec<bool>,
    pub ck: [f64; 4],
    /// 0 none, 1 exact at L, 2 interval (L, R], 3 right-censored at L.
    pub code: u8,
    pub l: f64,
    pub r: f64,
    pub cause_mask: [f64; 2],
    pub tau: f64,
    pub use_tau: bool,
}

fn safe_log<R: Real>(x: R, ok: bool) -> R {
    if ok && x.val() > 0.0 {
        x.ln()
    } else {
        R::cst(f64::NEG_INFINITY)
    }
}

fn clip<R: Real>(x: R, lo: R, hi: R) -> R {
    if x.val() < lo.val() {
        lo
    } else if x.val() > hi.val() {
        hi
    } else {
        x
    }
}

fn min<R: Real>(a: R, b: R) -> R {
    if a.val() <= b.val() {
        a
    } else {
        b
    }
}

/// Observation operator y_j = sqrt(q(t_j)) + e_j (calibration fixed, §7.1) and its Student-t terms.
/// Returns (log lik, Kalman state, d_pred at valid points, inside support).
pub fn obs_terms<R: Real>(
    rs: &RunState<R>,
    m: &Model,
    nat: &Natural<R>,
    run: &RunData,
) -> (R, Kalman<R>, Vec<R>, bool) {
    let tr = rs.t_rmin.val();
    if !(tr.is_finite() && tr > 0.0) || !rs.d0.val().is_finite() {
        // parameters outside the numerically representable region (e.g. overflowed rates)
        let k = Kalman {
            logdet: R::cst(f64::NAN),
            delta: R::cst(f64::NAN),
            n: 0,
            m_last: R::cst(0.0),
            p_last: R::cst(0.0),
            t_last: 0.0,
            started: false,
        };
        return (R::cst(f64::NEG_INFINITY), k, vec![], false);
    }
    let inside = run.t.iter().zip(&run.mask).all(|(t, &ok)| !ok || *t <= tr);
    let mut e = vec![R::cst(0.0); run.t.len()];
    let mut dp = Vec::new();
    for j in 0..run.t.len() {
        if run.mask[j] {
            let tc = run.t[j].clamp(0.0, tr);
            let u = rs.r_of_t(&m.num, R::cst(tc));
            let d = rs.d0 * u.sqrt();
            e[j] = R::cst(run.y[j]) - d;
            dp.push(d);
        }
    }
    let kal = ou_kalman(&run.t, &e, nat.sigma_c, nat.sigma_n, nat.ell, &run.mask);
    let ll = student_t_from_terms(kal.logdet, kal.delta, kal.n as f64, m.num.nu);
    (
        if inside {
            ll
        } else {
            R::cst(f64::NEG_INFINITY)
        },
        kal,
        dp,
        inside,
    )
}

/// Event contribution (§6.5–6.6).
pub fn event_loglik<R: Real>(rs: &RunState<R>, m: &Model, run: &RunData) -> R {
    let num = &m.num;
    let b = rs.b;
    let zero = R::cst(0.0);
    let l = R::cst(run.l);
    let ok1 = run.l < b.val();
    match run.code {
        1 | 2 => {
            let ul = rs.r_of_t(num, clip(l, zero, b));
            if run.code == 1 {
                if !ok1 {
                    return R::cst(f64::NEG_INFINITY);
                }
                let (log_s, _) = rs.log_survival_at_u(ul);
                let lam_sum = rs.integ.log_lam_h(R::cst(1.0) - ul).exp() * run.cause_mask[0]
                    + rs.lam_r * run.cause_mask[1];
                log_s + safe_log(lam_sum, ok1)
            } else {
                let hi = clip(min(R::cst(run.r), b), zero, b);
                let uh = rs.r_of_t(num, hi);
                let pk = rs.cause_integral(num, uh, ul, run.cause_mask);
                safe_log(pk[0] + pk[1], ok1)
            }
        }
        3 => {
            let uc = rs.r_of_t(num, min(l, b));
            rs.log_survival_at_u(uc).0
        }
        _ => zero,
    }
}

/// log p(observations, event | globals, locals) of one run; plus log S(tau) for prefix targets.
pub fn run_loglik<R: Real>(m: &Model, p: &[R], run: &RunData, eps_b: &[R], log_d0: R) -> R {
    let rs = run_state(m, p, &run.ck, eps_b, log_d0);
    let nat = natural_globals(m, p);
    let (mut ll, _, _, _) = obs_terms(&rs, m, &nat, run);
    if !ll.val().is_finite() {
        return R::cst(f64::NEG_INFINITY);
    }
    if m.st.hazards {
        ll += event_loglik(&rs, m, run);
    }
    if run.use_tau {
        if !(run.tau < rs.b.val()) {
            return R::cst(f64::NEG_INFINITY);
        }
        if m.st.hazards {
            let u = rs.r_of_t(&m.num, clip(R::cst(run.tau), R::cst(0.0), rs.b));
            ll += rs.log_survival_at_u(u).0;
        }
    }
    ll
}

/// Per-particle forecast quantities after the cutoff (all relative to S(tau)).
#[derive(Debug, Clone)]
pub struct Particle {
    pub d: Vec<f64>,
    pub before_b: Vec<bool>,
    pub s_rel: Vec<f64>,
    pub s_b_rel: f64,
    /// (2, T)
    pub f_rel: Vec<Vec<f64>>,
    pub b: f64,
    pub t_rmin: f64,
    pub e_loc: Vec<f64>,
    pub e_scale2: Vec<f64>,
    pub dof: f64,
    pub d0: f64,
    pub b0: f64,
    pub b1: f64,
}

pub fn forecast_particle(
    m: &Model,
    p: &[f64],
    run: &RunData,
    eps_b: &[f64],
    log_d0: f64,
    times: &[f64],
) -> Particle {
    let num = &m.num;
    let rs = run_state(m, p, &run.ck, eps_b, log_d0);
    let nat = natural_globals(m, p);
    let (_, kal, _, _) = obs_terms(&rs, m, &nat, run);
    let b = rs.b;
    let u_t: Vec<f64> = times
        .iter()
        .map(|&t| rs.r_of_t(num, t.min(b).clamp(0.0, b)))
        .collect();
    let u_tau = rs.r_of_t(num, run.tau.clamp(0.0, b));
    let u_b = rs.r_of_t(num, b);
    let (mut log_s_t, mut log_s_tau, mut log_s_b): (Vec<f64>, f64, f64) = (
        u_t.iter().map(|&u| rs.log_survival_at_u(u).0).collect(),
        rs.log_survival_at_u(u_tau).0,
        rs.log_survival_at_u(u_b).0,
    );
    if !m.st.hazards {
        log_s_t.iter_mut().for_each(|v| *v = 0.0);
        log_s_tau = 0.0;
        log_s_b = 0.0;
    }
    let nt = times.len();
    let mut f_rel = vec![vec![0.0; nt]; 2];
    if m.st.hazards {
        let (mut acc_h, mut acc_r) = (0.0, 0.0);
        let mut prev = u_tau;
        let inv = (-log_s_tau).exp();
        for j in 0..nt {
            let seg = rs.cause_integral(num, u_t[j], prev, [1.0, 1.0]);
            acc_h += seg[0];
            acc_r += seg[1];
            f_rel[0][j] = acc_h * inv;
            f_rel[1][j] = acc_r * inv;
            prev = u_t[j];
        }
    }
    let (mut e_loc, mut e_s2) = (Vec::with_capacity(nt), Vec::with_capacity(nt));
    let mut dof = 0.0;
    for &t in times {
        let (loc, _, s2y, df) = kalman_predict(&kal, t, nat.sigma_c, nat.sigma_n, nat.ell, num.nu);
        e_loc.push(loc);
        e_s2.push(s2y);
        dof = df;
    }
    Particle {
        d: u_t.iter().map(|u| rs.d0 * u.sqrt()).collect(),
        before_b: times.iter().map(|&t| t < b).collect(),
        s_rel: log_s_t.iter().map(|l| (l - log_s_tau).exp()).collect(),
        s_b_rel: (log_s_b - log_s_tau).exp(),
        f_rel,
        b,
        t_rmin: rs.t_rmin,
        e_loc,
        e_scale2: e_s2,
        dof,
        d0: rs.d0,
        b0: rs.b0,
        b1: rs.b1,
    }
}

/// Latent D(t) for t in support (VOI candidate times <= tau).
pub fn trajectory_d(rs: &RunState<f64>, num: &Numerics, times: &[f64]) -> Vec<f64> {
    times
        .iter()
        .map(|&t| rs.d0 * rs.r_of_t(num, t.clamp(0.0, rs.t_rmin)).sqrt())
        .collect()
}

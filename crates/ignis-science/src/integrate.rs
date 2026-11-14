//! Gauss–Legendre integration, cumulative panel tables and monotone inversion with implicit
//! gradients (V3.1 §6.4).
//!
//! A [`PanelTable`] stores, for a vector integrand f(u) on [lo, hi] split into P equal panels,
//! C_k = int_{e_k}^{hi} f(u) du at every panel edge. Evaluating int_u^{hi} f for arbitrary u costs
//! one local n-node rule on [u, e_{k+1}]. Tables are rebuilt whenever the parameters change; they
//! are values of the draw, never caches shared between draws.
//!
//! Monotone inversion of T(u) = int_u^{hi} f_c (f_c > 0, T strictly decreasing) brackets on the edge
//! table then runs safeguarded Newton inside the panel, on plain `f64`. The returned root carries
//! the implicit derivative du*/dθ = -∂θF / ∂uF (F = T(u;θ) - t): only F(u*; θ) is recorded on the
//! tape; the bracket decisions and Newton iterates are never differentiated.

use std::sync::OnceLock;

use crate::ad::Real;

/// n-point Gauss–Legendre rule on [-1, 1] (ascending nodes, as NumPy `leggauss`).
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub x: Vec<f64>,
    pub w: Vec<f64>,
}

impl Rule {
    pub fn new(n: usize) -> Rule {
        let mut x = vec![0.0; n];
        let mut w = vec![0.0; n];
        for i in 0..n {
            // i-th root from the top, refined by Newton on P_n
            let mut z = (std::f64::consts::PI * (i as f64 + 0.75) / (n as f64 + 0.5)).cos();
            let mut dp = 0.0;
            for _ in 0..100 {
                let (p, d) = legendre(n, z);
                dp = d;
                let dz = p / d;
                z -= dz;
                if dz.abs() < 1e-16 {
                    break;
                }
            }
            let (_, d) = legendre(n, z);
            dp = if d != 0.0 { d } else { dp };
            x[n - 1 - i] = z;
            w[n - 1 - i] = 2.0 / ((1.0 - z * z) * dp * dp);
        }
        Rule { x, w }
    }

    /// Cached rules for small n.
    pub fn cached(n: usize) -> &'static Rule {
        static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
        let rules = RULES.get_or_init(|| {
            (0..=64)
                .map(|k| {
                    if k == 0 {
                        Rule {
                            x: vec![],
                            w: vec![],
                        }
                    } else {
                        Rule::new(k)
                    }
                })
                .collect()
        });
        &rules[n]
    }
    pub fn len(&self) -> usize {
        self.x.len()
    }
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }
}

/// (P_n(z), P_n'(z)).
fn legendre(n: usize, z: f64) -> (f64, f64) {
    let (mut p0, mut p1) = (1.0, z);
    if n == 0 {
        return (1.0, 0.0);
    }
    for k in 2..=n {
        let p2 = ((2 * k - 1) as f64 * z * p1 - (k - 1) as f64 * p0) / k as f64;
        p0 = p1;
        p1 = p2;
    }
    (p1, n as f64 * (z * p1 - p0) / (z * z - 1.0))
}

/// Nodes and weights of the rule mapped to [a, b] (values only).
#[inline]
pub fn gl_nodes(a: f64, b: f64, rule: &Rule, u: &mut [f64], w: &mut [f64]) {
    let mid = 0.5 * (a + b);
    let half = 0.5 * (b - a);
    for i in 0..rule.len() {
        u[i] = mid + half * rule.x[i];
        w[i] = half * rule.w[i];
    }
}

/// A vector integrand with `M` components.
pub trait Integrand<R: Real, const M: usize> {
    /// f(u) as plain values (Newton iterations, |dT/du|).
    fn value(&self, u: f64) -> [f64; M];
    /// int_a^b f(u) du by the n-point rule, as differentiable values (θ and the limits).
    fn gl(&self, a: R, b: R, rule: &Rule) -> [R; M];
    /// Values-only rule (fast path for Newton).
    fn gl_value(&self, a: f64, b: f64, rule: &Rule) -> [f64; M] {
        let mid = 0.5 * (a + b);
        let half = 0.5 * (b - a);
        let mut acc = [0.0; M];
        for i in 0..rule.len() {
            let f = self.value(mid + half * rule.x[i]);
            for k in 0..M {
                acc[k] += half * rule.w[i] * f[k];
            }
        }
        acc
    }
}

#[derive(Debug, Clone)]
pub struct PanelTable<R, const M: usize> {
    pub lo: R,
    pub hi: R,
    pub edges: Vec<R>,
    /// int_{e_k}^{hi} f (P+1 rows, last row zero).
    pub cum: Vec<[R; M]>,
    pub edges_v: Vec<f64>,
    pub cum_v: Vec<[f64; M]>,
    /// f at every edge (values): slopes for the cubic Hermite initial guess of the inversion.
    pub f_edges: Vec<[f64; M]>,
    pub n_nodes: usize,
}

pub fn build_table<R: Real, F: Integrand<R, M>, const M: usize>(
    f: &F,
    lo: R,
    hi: R,
    n_panels: usize,
    n_nodes: usize,
) -> PanelTable<R, M> {
    let rule = Rule::cached(n_nodes);
    let step = 1.0 / n_panels as f64;
    let span = hi - lo;
    let edges: Vec<R> = (0..=n_panels)
        .map(|k| {
            if k == n_panels {
                lo + span
            } else {
                lo + span * (k as f64 * step)
            }
        })
        .collect();
    let mut cum = vec![[R::cst(0.0); M]; n_panels + 1];
    let mut acc = [R::cst(0.0); M];
    for k in (0..n_panels).rev() {
        let p = f.gl(edges[k], edges[k + 1], rule);
        for c in 0..M {
            acc[c] = if k == n_panels - 1 {
                p[c]
            } else {
                acc[c] + p[c]
            };
        }
        cum[k] = acc;
    }
    let edges_v: Vec<f64> = edges.iter().map(|e| e.val()).collect();
    let cum_v = cum.iter().map(|r| r.map(|x| x.val())).collect();
    let f_edges = edges_v.iter().map(|&e| f.value(e)).collect();
    PanelTable {
        lo,
        hi,
        edges,
        cum,
        edges_v,
        cum_v,
        f_edges,
        n_nodes,
    }
}

impl<R: Real, const M: usize> PanelTable<R, M> {
    pub fn n_panels(&self) -> usize {
        self.edges.len() - 1
    }
    pub fn total(&self) -> [R; M] {
        self.cum[0]
    }
    pub fn panel_of(&self, u: f64) -> usize {
        let p = self.n_panels();
        let h = (self.hi.val() - self.lo.val()) / p as f64;
        let k = ((u - self.lo.val()) / h).floor();
        if k.is_nan() || k < 0.0 {
            0
        } else {
            (k as usize).min(p - 1)
        }
    }
    /// int_u^{hi} f(v) dv for u in [lo, hi].
    pub fn upper<F: Integrand<R, M>>(&self, f: &F, u: R) -> [R; M] {
        let k = self.panel_of(u.val());
        let g = f.gl(
            u,
            self.edges[k + 1],
            crate::integrate::Rule::cached(self.n_nodes),
        );
        let mut o = self.cum[k + 1];
        for c in 0..M {
            o[c] += g[c];
        }
        o
    }
    pub fn upper_value<F: Integrand<R, M>>(&self, f: &F, u: f64) -> [f64; M] {
        let k = self.panel_of(u);
        let g = f.gl_value(u, self.edges_v[k + 1], Rule::cached(self.n_nodes));
        let mut o = self.cum_v[k + 1];
        for c in 0..M {
            o[c] += g[c];
        }
        o
    }

    /// Solve T(u) = t, T(u) = int_u^{hi} f_comp, for t in [0, T(lo)] (clipped to the ends; callers
    /// flag support separately). Value by bracket + safeguarded Newton; gradient by the implicit
    /// function theorem.
    pub fn invert_decreasing<F: Integrand<R, M>>(
        &self,
        f: &F,
        t: R,
        newton_steps: usize,
        comp: usize,
    ) -> R {
        let t0 = self.cum[0][comp];
        let tc = if t.val() < 0.0 {
            R::cst(0.0)
        } else if t.val() > t0.val() {
            t0
        } else {
            t
        };
        let u_star = self.solve_value(f, tc.val(), newton_steps, comp);
        let fv = self.upper(f, R::cst(u_star))[comp] - tc;
        if !R::GRAD {
            return R::cst(u_star);
        }
        let df = -f.value(u_star)[comp];
        R::cst(u_star) - (fv - fv.val()) / df
    }

    /// Plain-value root of T(u) = t.
    pub fn solve_value<F: Integrand<R, M>>(
        &self,
        f: &F,
        t: f64,
        newton_steps: usize,
        comp: usize,
    ) -> f64 {
        let p = self.n_panels();
        let tv = |k: usize| self.cum_v[k][comp];
        let cnt = (0..=p).filter(|&k| tv(k) > t).count() as isize;
        let k = (cnt - 1).clamp(0, p as isize - 1) as usize;
        let (a, b) = (self.edges_v[k], self.edges_v[k + 1]);
        let (ta, tb) = (tv(k), tv(k + 1));
        // Initial guess: cubic Hermite interpolation of the inverse u(T) on the panel (slopes
        // du/dT = -1/f at the edges), error O(h^4); then safeguarded Newton. Once a Newton step is
        // below 1e-10 the quadratic convergence puts the next error below 1e-18: accept it.
        let (fa, fb) = (self.f_edges[k][comp], self.f_edges[k + 1][comp]);
        let mut u = if ta > tb && fa > 0.0 && fb > 0.0 && fa.is_finite() && fb.is_finite() {
            let dt = tb - ta;
            let s = ((t - ta) / dt).clamp(0.0, 1.0);
            let (ma, mb) = (-dt / fa, -dt / fb);
            let (s2, s3) = (s * s, s * s * s);
            let v = (2.0 * s3 - 3.0 * s2 + 1.0) * a
                + (s3 - 2.0 * s2 + s) * ma
                + (-2.0 * s3 + 3.0 * s2) * b
                + (s3 - s2) * mb;
            v.clamp(a, b)
        } else {
            a + (b - a) * if ta > tb { (ta - t) / (ta - tb) } else { 0.5 }
        };
        for _ in 0..newton_steps {
            let tu = self.upper_value(f, u)[comp];
            let d = -f.value(u)[comp];
            let un = (u - (tu - t) / d).clamp(a, b);
            let step = (un - u).abs();
            u = un;
            if step <= 1e-10 * (b - a).max(1e-300) {
                break;
            }
        }
        u
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ad::{self, Var};

    #[test]
    fn rule_integrates_polynomials_exactly() {
        let r = Rule::new(8);
        let s: f64 = r.x.iter().zip(&r.w).map(|(x, w)| w * x.powi(14)).sum();
        assert!((s - 2.0 / 15.0).abs() < 1e-15);
        assert!(r.x.windows(2).all(|w| w[0] < w[1]));
    }

    /// f(u) = (exp(th u), th) generic; analytic integral and implicit gradient vs FD.
    struct E<R>(R);
    impl<R: Real> Integrand<R, 1> for E<R> {
        fn value(&self, u: f64) -> [f64; 1] {
            [(self.0.val() * u).exp()]
        }
        fn gl(&self, a: R, b: R, rule: &Rule) -> [R; 1] {
            let half = (b - a) * 0.5;
            let mid = (a + b) * 0.5;
            let mut acc = R::cst(0.0);
            for i in 0..rule.len() {
                acc += (self.0 * (mid + half * rule.x[i])).exp() * (half * rule.w[i]);
            }
            [acc]
        }
    }

    #[test]
    fn inversion_and_implicit_gradient() {
        let th = 0.7;
        let inv = |thv: f64, t: f64| {
            let f = E(thv);
            let tab = build_table(&f, 0.1, 1.0, 16, 8);
            tab.invert_decreasing(&f, t, 30, 0)
        };
        // T(u) = (e^{th} - e^{th u})/th
        let t = 0.5;
        let u = inv(th, t);
        let exact = ((th.exp() - th * t).ln()) / th;
        assert!((u - exact).abs() < 1e-13);
        let (_, g) = ad::value_and_grad(&[th], |x: &[Var]| {
            let f = E(x[0]);
            let tab = build_table(&f, Var::cst(0.1), Var::cst(1.0), 16, 8);
            tab.invert_decreasing(&f, Var::cst(t), 30, 0)
        });
        let h = 1e-6;
        let fd = (inv(th + h, t) - inv(th - h, t)) / (2.0 * h);
        assert!((g[0] - fd).abs() < 1e-8, "{} vs {fd}", g[0]);
    }
}

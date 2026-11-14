//! First- and quasi-second-order optimisers for MAP fits on flat FP64 vectors.
//!
//! * [`Adam`]: optax semantics (`clip_by_global_norm` then `adam(lr)`, b1 = 0.9, b2 = 0.999,
//!   eps = 1e-8, bias-corrected), optional decoupled weight decay (AdamW). Non-finite gradient
//!   entries are zeroed before the update (as the Python driver did). State is serialisable so MAP
//!   fits checkpoint and resume at epoch boundaries.
//! * [`lbfgs`]: limited-memory BFGS with a strong-Wolfe line search (bracketing + zoom).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdamConfig {
    pub learning_rate: f64,
    pub b1: f64,
    pub b2: f64,
    pub eps: f64,
    /// 0 disables clipping.
    pub clip_norm: f64,
    /// Decoupled weight decay (AdamW); 0 = Adam.
    pub weight_decay: f64,
}

impl Default for AdamConfig {
    fn default() -> Self {
        AdamConfig {
            learning_rate: 1e-2,
            b1: 0.9,
            b2: 0.999,
            eps: 1e-8,
            clip_norm: 10.0,
            weight_decay: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdamState {
    pub t: u64,
    pub mu: Vec<f64>,
    pub nu: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct Adam {
    pub cfg: AdamConfig,
    pub state: AdamState,
}

impl Adam {
    pub fn new(cfg: AdamConfig, dim: usize) -> Adam {
        Adam {
            cfg,
            state: AdamState {
                t: 0,
                mu: vec![0.0; dim],
                nu: vec![0.0; dim],
            },
        }
    }
    /// One update of `x` from the gradient `g` of the loss (minimisation). `g` is modified.
    pub fn step(&mut self, x: &mut [f64], g: &mut [f64]) {
        for v in g.iter_mut() {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
        if self.cfg.clip_norm > 0.0 {
            let n = g.iter().map(|v| v * v).sum::<f64>().sqrt();
            if n >= self.cfg.clip_norm {
                let s = self.cfg.clip_norm / n;
                g.iter_mut().for_each(|v| *v *= s);
            }
        }
        let st = &mut self.state;
        st.t += 1;
        let (b1, b2) = (self.cfg.b1, self.cfg.b2);
        let c1 = 1.0 - b1.powi(st.t as i32);
        let c2 = 1.0 - b2.powi(st.t as i32);
        let lr = self.cfg.learning_rate;
        for i in 0..x.len() {
            st.mu[i] = b1 * st.mu[i] + (1.0 - b1) * g[i];
            st.nu[i] = b2 * st.nu[i] + (1.0 - b2) * g[i] * g[i];
            let mh = st.mu[i] / c1;
            let vh = st.nu[i] / c2;
            let mut u = -lr * mh / (vh.sqrt() + self.cfg.eps);
            if self.cfg.weight_decay > 0.0 {
                u -= lr * self.cfg.weight_decay * x[i];
            }
            x[i] += u;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LbfgsConfig {
    pub max_iter: usize,
    pub history: usize,
    /// Stop when ||g||_inf <= gtol.
    pub gtol: f64,
    /// Stop when the relative decrease of f is below ftol.
    pub ftol: f64,
    pub c1: f64,
    pub c2: f64,
}

impl Default for LbfgsConfig {
    fn default() -> Self {
        LbfgsConfig {
            max_iter: 500,
            history: 10,
            gtol: 1e-6,
            ftol: 1e-12,
            c1: 1e-4,
            c2: 0.9,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LbfgsResult {
    pub x: Vec<f64>,
    pub f: f64,
    pub iterations: usize,
    pub evaluations: usize,
    pub converged: bool,
    pub message: String,
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Minimise `f` (returns value and gradient). Non-finite values are treated as +inf (the line
/// search backs off).
pub fn lbfgs(
    mut f: impl FnMut(&[f64]) -> (f64, Vec<f64>),
    x0: &[f64],
    cfg: &LbfgsConfig,
) -> LbfgsResult {
    let n = x0.len();
    let mut x = x0.to_vec();
    let (mut fx, mut g) = f(&x);
    let mut evals = 1;
    if !fx.is_finite() {
        return LbfgsResult {
            x,
            f: fx,
            iterations: 0,
            evaluations: evals,
            converged: false,
            message: "non-finite start".into(),
        };
    }
    let mut s_hist: Vec<Vec<f64>> = Vec::new();
    let mut y_hist: Vec<Vec<f64>> = Vec::new();
    let mut msg = "max_iter".to_string();
    let mut converged = false;
    let mut it = 0;
    while it < cfg.max_iter {
        if g.iter().fold(0.0f64, |m, v| m.max(v.abs())) <= cfg.gtol {
            converged = true;
            msg = "gtol".into();
            break;
        }
        // two-loop recursion
        let mut q = g.clone();
        let k = s_hist.len();
        let mut alpha = vec![0.0; k];
        for i in (0..k).rev() {
            let rho = 1.0 / dot(&y_hist[i], &s_hist[i]);
            alpha[i] = rho * dot(&s_hist[i], &q);
            for j in 0..n {
                q[j] -= alpha[i] * y_hist[i][j];
            }
        }
        let gamma = if k > 0 {
            dot(&s_hist[k - 1], &y_hist[k - 1]) / dot(&y_hist[k - 1], &y_hist[k - 1])
        } else {
            1.0 / g.iter().map(|v| v * v).sum::<f64>().sqrt().max(1.0)
        };
        q.iter_mut().for_each(|v| *v *= gamma);
        for i in 0..k {
            let rho = 1.0 / dot(&y_hist[i], &s_hist[i]);
            let b = rho * dot(&y_hist[i], &q);
            for j in 0..n {
                q[j] += s_hist[i][j] * (alpha[i] - b);
            }
        }
        let mut d: Vec<f64> = q.iter().map(|v| -v).collect();
        let mut dg = dot(&d, &g);
        if !(dg < 0.0) {
            d = g.iter().map(|v| -v).collect();
            dg = dot(&d, &g);
            s_hist.clear();
            y_hist.clear();
        }
        // strong-Wolfe line search
        let phi = |a: f64, f: &mut dyn FnMut(&[f64]) -> (f64, Vec<f64>), ev: &mut usize| {
            let xa: Vec<f64> = x.iter().zip(&d).map(|(xi, di)| xi + a * di).collect();
            let (v, gv) = f(&xa);
            *ev += 1;
            let v = if v.is_finite() { v } else { f64::INFINITY };
            (v, dot(&gv, &d), xa, gv)
        };
        let (mut a_lo, mut f_lo, mut a_hi) = (0.0, fx, f64::NAN);
        let mut a = 1.0;
        let mut found: Option<(f64, Vec<f64>, Vec<f64>)> = None;
        for _ in 0..40 {
            let (fa, da, xa, ga) = phi(a, &mut f, &mut evals);
            if fa > fx + cfg.c1 * a * dg || (fa >= f_lo && a_lo > 0.0) {
                a_hi = a;
            } else if da.abs() <= -cfg.c2 * dg {
                found = Some((fa, xa, ga));
                break;
            } else if da >= 0.0 {
                a_hi = a_lo;
                a_lo = a;
                f_lo = fa;
                found = Some((fa, xa, ga));
            } else {
                a_lo = a;
                f_lo = fa;
                found = Some((fa, xa, ga));
            }
            a = if a_hi.is_nan() {
                2.0 * a
            } else {
                0.5 * (a_lo + a_hi)
            };
            if (a_hi - a_lo).abs() < 1e-16 {
                break;
            }
        }
        let Some((fn_, xn, gn)) = found else {
            msg = "line search failed".into();
            break;
        };
        let s: Vec<f64> = xn.iter().zip(&x).map(|(a, b)| a - b).collect();
        let y: Vec<f64> = gn.iter().zip(&g).map(|(a, b)| a - b).collect();
        let rel = (fx - fn_).abs() / fx.abs().max(fn_.abs()).max(1.0);
        x = xn;
        g = gn;
        let old = fx;
        fx = fn_;
        it += 1;
        if dot(&s, &y) > 1e-12 * dot(&y, &y).sqrt() * dot(&s, &s).sqrt() {
            s_hist.push(s);
            y_hist.push(y);
            if s_hist.len() > cfg.history {
                s_hist.remove(0);
                y_hist.remove(0);
            }
        }
        if rel <= cfg.ftol && fx <= old {
            converged = true;
            msg = "ftol".into();
            break;
        }
    }
    LbfgsResult {
        x,
        f: fx,
        iterations: it,
        evaluations: evals,
        converged,
        message: msg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rosen(x: &[f64]) -> (f64, Vec<f64>) {
        let (a, b) = (x[0], x[1]);
        let f = (1.0 - a).powi(2) + 100.0 * (b - a * a).powi(2);
        (
            f,
            vec![
                -2.0 * (1.0 - a) - 400.0 * a * (b - a * a),
                200.0 * (b - a * a),
            ],
        )
    }

    #[test]
    fn lbfgs_and_adam_minimise_rosenbrock() {
        let r = lbfgs(rosen, &[-1.2, 1.0], &LbfgsConfig::default());
        assert!(r.converged && (r.x[0] - 1.0).abs() < 1e-5, "{r:?}");
        let mut x = vec![-1.2, 1.0];
        let mut opt = Adam::new(
            AdamConfig {
                learning_rate: 0.02,
                ..Default::default()
            },
            2,
        );
        for _ in 0..20000 {
            let (_, mut g) = rosen(&x);
            opt.step(&mut x, &mut g);
        }
        assert!((x[0] - 1.0).abs() < 1e-2, "{x:?}");
    }
}

//! Thin L-BFGS (two-loop recursion, strong-Wolfe line search; Nocedal & Wright Alg. 7.4/3.5–3.6) and
//! MCMC diagnostics (split R-hat, ESS as in NumPyro). Kept local to this lane until the shared
//! `ignis_science::{optim, diagnostics}` modules land; to be consolidated there.

pub struct LbfgsResult {
    pub x: Vec<f64>,
    pub f: f64,
    pub iterations: usize,
    pub evaluations: usize,
    pub converged: bool,
    pub message: String,
}

/// Minimise f. `fg(x, g) -> f` writes the gradient. Stops at max|g| ≤ pgtol or a relative reduction
/// below `ftol` (SciPy L-BFGS-B defaults: pgtol 1e-5, factr 1e7 → ftol ≈ 2.2e-9).
pub fn lbfgs<F: FnMut(&[f64], &mut [f64]) -> f64>(
    mut fg: F,
    x0: &[f64],
    maxiter: usize,
    m: usize,
) -> LbfgsResult {
    let n = x0.len();
    let pgtol = 1e-5;
    let ftol = 1e7 * f64::EPSILON;
    let mut x = x0.to_vec();
    let mut g = vec![0.0; n];
    let mut f = fg(&x, &mut g);
    let mut evals = 1;
    let mut s_hist: Vec<Vec<f64>> = vec![];
    let mut y_hist: Vec<Vec<f64>> = vec![];
    let mut rho: Vec<f64> = vec![];
    let mut d = vec![0.0; n];
    let mut xn = vec![0.0; n];
    let mut gn = vec![0.0; n];
    let inf_norm = |v: &[f64]| v.iter().fold(0.0f64, |a, b| a.max(b.abs()));
    if !f.is_finite() {
        return LbfgsResult {
            x,
            f,
            iterations: 0,
            evaluations: evals,
            converged: false,
            message: "non-finite initial objective".into(),
        };
    }
    for it in 0..maxiter {
        if inf_norm(&g) <= pgtol {
            return LbfgsResult {
                x,
                f,
                iterations: it,
                evaluations: evals,
                converged: true,
                message: "projected gradient below pgtol".into(),
            };
        }
        // two-loop recursion
        d.iter_mut().zip(&g).for_each(|(a, b)| *a = -b);
        let k = s_hist.len();
        let mut alpha = vec![0.0; k];
        for i in (0..k).rev() {
            alpha[i] = rho[i] * dot(&s_hist[i], &d);
            axpy(-alpha[i], &y_hist[i], &mut d);
        }
        if k > 0 {
            let gamma = dot(&s_hist[k - 1], &y_hist[k - 1]) / dot(&y_hist[k - 1], &y_hist[k - 1]);
            d.iter_mut().for_each(|v| *v *= gamma);
        } else {
            let gn0 = dot(&g, &g).sqrt();
            d.iter_mut().for_each(|v| *v /= gn0.max(1.0));
        }
        for i in 0..k {
            let b = rho[i] * dot(&y_hist[i], &d);
            axpy(alpha[i] - b, &s_hist[i], &mut d);
        }
        let mut dg = dot(&d, &g);
        if dg >= 0.0 {
            // not a descent direction: restart from steepest descent
            s_hist.clear();
            y_hist.clear();
            rho.clear();
            d.iter_mut()
                .zip(&g)
                .for_each(|(a, b)| *a = -b / dot(&g, &g).sqrt().max(1.0));
            dg = dot(&d, &g);
        }
        let (step, fnew, ne) = wolfe(&mut fg, &x, f, &g, &d, dg, &mut xn, &mut gn);
        evals += ne;
        if step == 0.0 {
            return LbfgsResult {
                x,
                f,
                iterations: it,
                evaluations: evals,
                converged: false,
                message: "line search failed".into(),
            };
        }
        let s: Vec<f64> = xn.iter().zip(&x).map(|(a, b)| a - b).collect();
        let y: Vec<f64> = gn.iter().zip(&g).map(|(a, b)| a - b).collect();
        let sy = dot(&s, &y);
        let fold = f;
        x.copy_from_slice(&xn);
        g.copy_from_slice(&gn);
        f = fnew;
        if sy > 1e-12 * dot(&y, &y).sqrt() * dot(&s, &s).sqrt() {
            if s_hist.len() == m {
                s_hist.remove(0);
                y_hist.remove(0);
                rho.remove(0);
            }
            rho.push(1.0 / sy);
            s_hist.push(s);
            y_hist.push(y);
        }
        if (fold - f) / fold.abs().max(f.abs()).max(1.0) <= ftol {
            return LbfgsResult {
                x,
                f,
                iterations: it + 1,
                evaluations: evals,
                converged: true,
                message: "relative reduction below ftol".into(),
            };
        }
    }
    LbfgsResult {
        x,
        f,
        iterations: maxiter,
        evaluations: evals,
        converged: false,
        message: "maximum iterations".into(),
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn axpy(a: f64, x: &[f64], y: &mut [f64]) {
    y.iter_mut().zip(x).for_each(|(yi, xi)| *yi += a * xi);
}

#[allow(clippy::too_many_arguments)]
fn wolfe<F: FnMut(&[f64], &mut [f64]) -> f64>(
    fg: &mut F,
    x: &[f64],
    f0: f64,
    _g0: &[f64],
    d: &[f64],
    dg0: f64,
    xn: &mut [f64],
    gn: &mut [f64],
) -> (f64, f64, usize) {
    let (c1, c2) = (1e-4, 0.9);
    let mut evals = 0;
    let mut eval = |a: f64, xn: &mut [f64], gn: &mut [f64], evals: &mut usize| {
        for i in 0..x.len() {
            xn[i] = x[i] + a * d[i];
        }
        *evals += 1;
        let f = fg(xn, gn);
        (f, dot(gn, d))
    };
    let mut a_prev = 0.0;
    let mut f_prev = f0;
    let mut dg_prev = dg0;
    let mut a = 1.0;
    for i in 0..30 {
        let (f, dg) = eval(a, xn, gn, &mut evals);
        if !f.is_finite() {
            a = 0.5 * (a_prev + a);
            continue;
        }
        if f > f0 + c1 * a * dg0 || (i > 0 && f >= f_prev) {
            return zoom(
                &mut eval, a_prev, a, f_prev, dg_prev, f0, dg0, xn, gn, &mut evals,
            );
        }
        if dg.abs() <= -c2 * dg0 {
            return (a, f, evals);
        }
        if dg >= 0.0 {
            return zoom(&mut eval, a, a_prev, f, dg, f0, dg0, xn, gn, &mut evals);
        }
        a_prev = a;
        f_prev = f;
        dg_prev = dg;
        a *= 2.0;
    }
    (0.0, f0, evals)
}

#[allow(clippy::too_many_arguments)]
fn zoom<E: FnMut(f64, &mut [f64], &mut [f64], &mut usize) -> (f64, f64)>(
    eval: &mut E,
    mut lo: f64,
    mut hi: f64,
    mut f_lo: f64,
    mut dg_lo: f64,
    f0: f64,
    dg0: f64,
    xn: &mut [f64],
    gn: &mut [f64],
    evals: &mut usize,
) -> (f64, f64, usize) {
    let (c1, c2) = (1e-4, 0.9);
    let mut best = (0.0, f0);
    for _ in 0..40 {
        let _ = dg_lo;
        let a = lo + 0.5 * (hi - lo); // bisection (robust; the bracket is small near convergence)
        let (f, dg) = eval(a, xn, gn, evals);
        if f.is_finite() && f < best.1 {
            best = (a, f);
        }
        if !f.is_finite() || f > f0 + c1 * a * dg0 || f >= f_lo {
            hi = a;
        } else {
            if dg.abs() <= -c2 * dg0 {
                return (a, f, *evals);
            }
            if dg * (hi - lo) >= 0.0 {
                hi = lo;
            }
            lo = a;
            f_lo = f;
            dg_lo = dg;
        }
        if (hi - lo).abs() < 1e-14 {
            break;
        }
    }
    if best.0 > 0.0 {
        let (f, _) = eval(best.0, xn, gn, evals);
        return (best.0, f, *evals);
    }
    (0.0, f0, *evals)
}

// ---------------------------------------------------------------- MCMC diagnostics (NumPyro)

/// `samples[c][s][d]` chains × draws × dims.
pub fn split_gelman_rubin(samples: &[Vec<Vec<f64>>]) -> Vec<f64> {
    let c = samples.len();
    let n = samples[0].len();
    let half = n / 2;
    let mut split: Vec<Vec<Vec<f64>>> = vec![];
    for ch in samples {
        split.push(ch[..half].to_vec());
        split.push(ch[n - half..].to_vec());
    }
    let _ = c;
    gelman_rubin(&split)
}

fn gelman_rubin(samples: &[Vec<Vec<f64>>]) -> Vec<f64> {
    let c = samples.len() as f64;
    let n = samples[0].len() as f64;
    let dims = samples[0][0].len();
    (0..dims)
        .map(|d| {
            let means: Vec<f64> = samples
                .iter()
                .map(|ch| ch.iter().map(|s| s[d]).sum::<f64>() / n)
                .collect();
            let vars: Vec<f64> = samples
                .iter()
                .zip(&means)
                .map(|(ch, m)| ch.iter().map(|s| (s[d] - m).powi(2)).sum::<f64>() / (n - 1.0))
                .collect();
            let w = vars.iter().sum::<f64>() / c;
            let gm = means.iter().sum::<f64>() / c;
            let b_over_n = means.iter().map(|m| (m - gm).powi(2)).sum::<f64>() / (c - 1.0);
            let var_plus = (n - 1.0) / n * w + b_over_n;
            (var_plus / w).sqrt()
        })
        .collect()
}

fn autocovariance(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    let m = x.iter().sum::<f64>() / n as f64;
    (0..n)
        .map(|lag| {
            (0..n - lag)
                .map(|i| (x[i] - m) * (x[i + lag] - m))
                .sum::<f64>()
                / n as f64
        })
        .collect()
}

/// Effective sample size (Geyer initial monotone sequence over chain-averaged autocorrelations).
pub fn effective_sample_size(samples: &[Vec<Vec<f64>>]) -> Vec<f64> {
    let c = samples.len();
    let n = samples[0].len();
    let dims = samples[0][0].len();
    (0..dims)
        .map(|d| {
            let ac: Vec<Vec<f64>> = samples
                .iter()
                .map(|ch| autocovariance(&ch.iter().map(|s| s[d]).collect::<Vec<_>>()))
                .collect();
            let mean_var =
                ac.iter().map(|a| a[0]).sum::<f64>() / c as f64 * n as f64 / (n as f64 - 1.0);
            let var_plus = if c > 1 {
                let means: Vec<f64> = samples
                    .iter()
                    .map(|ch| ch.iter().map(|s| s[d]).sum::<f64>() / n as f64)
                    .collect();
                let gm = means.iter().sum::<f64>() / c as f64;
                mean_var * (n as f64 - 1.0) / n as f64
                    + means.iter().map(|m| (m - gm).powi(2)).sum::<f64>() / (c as f64 - 1.0)
            } else {
                mean_var * (n as f64 - 1.0) / n as f64
            };
            if !(var_plus > 0.0) {
                return n as f64 * c as f64;
            }
            let rho = |lag: usize| {
                1.0 - (mean_var - ac.iter().map(|a| a[lag]).sum::<f64>() / c as f64) / var_plus
            };
            let mut tau = -1.0;
            let mut prev_pair = f64::INFINITY;
            let mut t = 0;
            while t + 1 < n {
                let mut pair = rho(t) + rho(t + 1);
                if pair < 0.0 {
                    break;
                }
                pair = pair.min(prev_pair);
                prev_pair = pair;
                tau += 2.0 * pair;
                t += 2;
            }
            (c * n) as f64 / tau.max(1e-12)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lbfgs_rosenbrock() {
        let r = lbfgs(
            |x, g| {
                let (a, b) = (x[0], x[1]);
                g[0] = -2.0 * (1.0 - a) - 400.0 * a * (b - a * a);
                g[1] = 200.0 * (b - a * a);
                (1.0 - a).powi(2) + 100.0 * (b - a * a).powi(2)
            },
            &[-1.2, 1.0],
            500,
            10,
        );
        assert!(
            r.converged && (r.x[0] - 1.0).abs() < 1e-4,
            "{:?} {}",
            r.x,
            r.message
        );
    }
}

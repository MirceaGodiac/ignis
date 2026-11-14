//! Rational-quadratic spline transforms and coupling masks (V3.1 §10.4, Durkan et al. [L22]). FP64.
//!
//! Per transformed component: K bins on [−B, B] with identity tails outside, K widths, K heights and
//! K−1 interior derivatives (3K−1 = 23 raw outputs for K = 8); boundary derivatives are fixed at 1.
//! Within a bin with width w, height h, δ = h/w, ξ = (x − x₀)/w:
//!
//!   y     = y₀ + h [δ ξ² + d₀ ξ(1−ξ)] / [δ + (d₁ + d₀ − 2δ) ξ(1−ξ)]
//!   dy/dx = δ² [d₁ ξ² + 2δ ξ(1−ξ) + d₀ (1−ξ)²] / [δ + (d₁ + d₀ − 2δ) ξ(1−ξ)]²  > 0
//!
//! The inverse solves the bin quadratic in closed form (numerically stable root). Forward and inverse
//! return the log |Jacobian| of the map they apply. This scalar FP64 version is the reference; the
//! trainable tensor version in `ignis-accel` is checked against it.

pub const DEFAULT_BINS: usize = 8;
pub const DEFAULT_BOUND: f64 = 5.0;
pub const MIN_WIDTH: f64 = 1e-3;
pub const MIN_HEIGHT: f64 = 1e-3;
pub const MIN_DERIVATIVE: f64 = 1e-3;

pub fn n_spline_params(bins: usize) -> usize {
    3 * bins - 1
}

/// Knot positions (K+1) and sizes (K) from raw logits (softmax with a minimum share).
fn knots(raw: &[f64], bound: f64, minimum: f64, cum: &mut [f64], size: &mut [f64]) {
    let k = raw.len();
    let mx = raw.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let s: f64 = raw.iter().map(|r| (r - mx).exp()).sum();
    let mut acc = 0.0;
    cum[0] = 0.0;
    for i in 0..k {
        acc += minimum + (1.0 - minimum * k as f64) * ((raw[i] - mx).exp() / s);
        cum[i + 1] = acc;
    }
    for c in cum.iter_mut() {
        *c = 2.0 * bound * *c - bound;
    }
    cum[0] = -bound;
    cum[k] = bound;
    for i in 0..k {
        size[i] = cum[i + 1] - cum[i];
    }
}

fn softplus(x: f64) -> f64 {
    // jax.nn.softplus = logaddexp(x, 0)
    x.max(0.0) + (-x.abs()).exp().ln_1p()
}

/// Knots/derivatives of one spline from its raw (3K−1) conditioner outputs.
pub struct SplineParts {
    pub cw: Vec<f64>,
    pub w: Vec<f64>,
    pub ch: Vec<f64>,
    pub h: Vec<f64>,
    pub d: Vec<f64>,
}

pub fn spline_parts(raw: &[f64], bins: usize, bound: f64) -> SplineParts {
    assert_eq!(raw.len(), n_spline_params(bins));
    let mut p = SplineParts {
        cw: vec![0.0; bins + 1],
        w: vec![0.0; bins],
        ch: vec![0.0; bins + 1],
        h: vec![0.0; bins],
        d: vec![1.0; bins + 1],
    };
    knots(&raw[..bins], bound, MIN_WIDTH, &mut p.cw, &mut p.w);
    knots(&raw[bins..2 * bins], bound, MIN_HEIGHT, &mut p.ch, &mut p.h);
    for i in 0..bins - 1 {
        p.d[i + 1] = MIN_DERIVATIVE + softplus(raw[2 * bins + i]);
    }
    p
}

fn bin_index(knots: &[f64], v: f64, bins: usize) -> usize {
    // clip(sum(v >= knots[1:-1]), 0, bins-1)
    let n = knots[1..bins].iter().filter(|&&k| v >= k).count();
    n.min(bins - 1)
}

/// y = g(x) for one component; returns (y, log dy/dx).
pub fn rq_forward(x: f64, raw: &[f64], bins: usize, bound: f64) -> (f64, f64) {
    if !(x > -bound && x < bound) {
        return (x, 0.0);
    }
    let p = spline_parts(raw, bins, bound);
    let k = bin_index(&p.cw, x, bins);
    let (x0, wk, y0, hk, d0, d1) = (p.cw[k], p.w[k], p.ch[k], p.h[k], p.d[k], p.d[k + 1]);
    let delta = hk / wk;
    let xi = ((x - x0) / wk).clamp(0.0, 1.0);
    let om = xi * (1.0 - xi);
    let den = delta + (d1 + d0 - 2.0 * delta) * om;
    let y = y0 + hk * (delta * xi * xi + d0 * om) / den;
    let dydx = delta * delta * (d1 * xi * xi + 2.0 * delta * om + d0 * (1.0 - xi) * (1.0 - xi))
        / (den * den);
    (y, dydx.ln())
}

/// x = g⁻¹(y); returns (x, log dx/dy) = (x, −log g'(x)).
pub fn rq_inverse(y: f64, raw: &[f64], bins: usize, bound: f64) -> (f64, f64) {
    if !(y > -bound && y < bound) {
        return (y, 0.0);
    }
    let p = spline_parts(raw, bins, bound);
    let k = bin_index(&p.ch, y, bins);
    let (x0, wk, y0, hk, d0, d1) = (p.cw[k], p.w[k], p.ch[k], p.h[k], p.d[k], p.d[k + 1]);
    let delta = hk / wk;
    let dy = y - y0;
    let s = d1 + d0 - 2.0 * delta;
    let a = hk * (delta - d0) + dy * s;
    let b = hk * d0 - dy * s;
    let c = -delta * dy;
    let disc = (b * b - 4.0 * a * c).max(0.0);
    let xi = (2.0 * c / (-b - disc.sqrt())).clamp(0.0, 1.0);
    let x = x0 + xi * wk;
    let om = xi * (1.0 - xi);
    let den = delta + s * om;
    let dydx = delta * delta * (d1 * xi * xi + 2.0 * delta * om + d0 * (1.0 - xi) * (1.0 - xi))
        / (den * den);
    (x, -dydx.ln())
}

/// Alternating static masks (true = transformed). Even layers transform the first ⌊d/2⌋ (≥ 1)
/// dimensions, odd layers the rest; for d = 1 every layer transforms the dimension.
pub fn coupling_masks(dim: usize, layers: usize) -> Vec<Vec<bool>> {
    if dim == 1 {
        return vec![vec![true]; layers];
    }
    let first = (dim / 2).max(1);
    let base: Vec<bool> = (0..dim).map(|i| i < first).collect();
    (0..layers)
        .map(|l| {
            if l % 2 == 0 {
                base.clone()
            } else {
                base.iter().map(|b| !b).collect()
            }
        })
        .collect()
}

pub fn standard_normal_logpdf(z: &[f64]) -> f64 {
    z.iter()
        .map(|v| -0.5 * v * v - 0.5 * (2.0 * std::f64::consts::PI).ln())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // approx standard normal from 12 uniforms
        let mut s = 0.0;
        for _ in 0..12 {
            *seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            s += (*seed >> 11) as f64 / (1u64 << 53) as f64;
        }
        s - 6.0
    }

    #[test]
    fn inverse_and_derivative() {
        let mut seed = 7u64;
        let n = 400;
        let mut inv_err: f64 = 0.0;
        let mut ld_err: f64 = 0.0;
        let mut der_err: f64 = 0.0;
        for i in 0..n {
            let raw: Vec<f64> = (0..n_spline_params(8))
                .map(|_| 2.0 * lcg(&mut seed))
                .collect();
            let x = -6.0 + 12.0 * i as f64 / (n - 1) as f64;
            let (y, ld) = rq_forward(x, &raw, 8, 5.0);
            let (xb, ldi) = rq_inverse(y, &raw, 8, 5.0);
            inv_err = inv_err.max((xb - x).abs());
            ld_err = ld_err.max((ld + ldi).abs());
            if x.abs() < 5.0 - 1e-3 {
                let h = 1e-6;
                let fd = (rq_forward(x + h, &raw, 8, 5.0).0 - rq_forward(x - h, &raw, 8, 5.0).0)
                    / (2.0 * h);
                der_err = der_err.max((fd - ld.exp()).abs() / (1e-8 + 1e-6 * ld.exp()));
                assert!(ld.exp() > 0.0);
            } else if x.abs() > 5.0 {
                assert_eq!(y, x);
            }
        }
        assert!(inv_err < 1e-10, "{inv_err}");
        assert!(ld_err < 1e-9, "{ld_err}");
        assert!(der_err <= 1.0, "{der_err}"); // |fd − g'| ≤ 1e-8 + 1e-6 |g'| (the Python check)
    }
}

//! Dense kernels of the strict forwards. Plain loops over borrowed slices, no allocation: callers
//! own the buffers and reuse them across integration steps.

/// `out[j] = b[j] + sum_k w[j * n_in + k] * x[k]` for a row-major `[n_out, n_in]` matrix.
#[inline]
pub fn affine(w: &[f64], b: &[f64], x: &[f64], out: &mut [f64]) {
    let n_in = x.len();
    debug_assert_eq!(w.len(), out.len() * n_in);
    debug_assert_eq!(b.len(), out.len());
    for (j, o) in out.iter_mut().enumerate() {
        let row = &w[j * n_in..(j + 1) * n_in];
        let mut acc = b[j];
        for (wk, xk) in row.iter().zip(x) {
            acc += wk * xk;
        }
        *o = acc;
    }
}

#[inline]
pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    debug_assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[inline]
pub fn relu(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}

#[inline]
pub fn relu_in_place(v: &mut [f64]) {
    for x in v {
        *x = relu(*x);
    }
}

/// Logistic function, stable for large |x|.
#[inline]
pub fn sigmoid(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let e = x.exp();
        e / (1.0 + e)
    }
}

/// `ln(1 + e^x)` without overflow.
#[inline]
pub fn softplus(x: f64) -> f64 {
    if x > 30.0 {
        x
    } else {
        x.exp().ln_1p()
    }
}

/// One hidden ReLU layer with a linear skip path:
/// `y = b_out + skip . x + sum_j w_out[j] * relu(b_hidden[j] + W_hidden[j, :] . x)`.
///
/// With every hidden weight zero this is the linear law `b_out + skip . x`, so a hidden unit only
/// ever adds a hinge the author placed. `hidden` is scratch of length `b_hidden.len()`.
#[inline]
pub fn hinge_net(
    w_hidden: &[f64],
    b_hidden: &[f64],
    w_out: &[f64],
    skip: &[f64],
    b_out: f64,
    x: &[f64],
    hidden: &mut [f64],
) -> f64 {
    affine(w_hidden, b_hidden, x, hidden);
    let mut y = b_out + dot(skip, x);
    for (h, w) in hidden.iter().zip(w_out) {
        y += w * relu(*h);
    }
    y
}

//! BASS compact neural operator on a NONPERIODIC domain with cosine modes (V3.1 §9.3–§9.4).
//!
//! Fixed coordinates: ξ = s/W ∈ [0, 1], η = f/L; K nodes ξ_j = j/(K−1) (K = 32 nominal). Trapezoid
//! weights q_0 = q_{K−1} = 1/[2(K−1)], q_j = 1/(K−1) inside. Basis φ_0 = 1, φ_m = √2 cos(mπξ).
//! Coefficients ĥ_m = Σ_j q_j φ_m(ξ_j) h(ξ_j) and 𝒦_ℓ h(ξ) = Σ_{m=0}^{3} φ_m(ξ) A_{ℓm} ĥ_m. With these
//! weights the discrete basis is exactly orthonormal for m, n < K−1 (DCT-I identity). No library DCT.
//!
//! Network (per ensemble member): lift 8→16 (144), three blocks
//!   h_{ℓ+1} = tanh((1 + γ_ℓ(c)) ⊙ [W_ℓ h_ℓ + 𝒦_ℓ h_ℓ + b_ℓ] + β_ℓ(c))      (3 · 1296 = 3888)
//! FiLM 8→16→96 (tanh hidden) for γ, β (1776), head 16→1 (17). Total 5825 weights. Input channels
//! (8): η, η_ξ, η_ξξ (linear ghost extrapolation), ξ, static admitted-domain mask, x_min/L, x_max/L,
//! distance to the nearest lateral edge /W (constant for a rectangle; declared redundancy).
//!
//! The operator is defined for any K (pointwise layers + trapezoid projections), which the
//! resolution study uses. Reverse mode is hand-written ([`backward`]), checked against JAX.

pub const N_NODES: usize = 32;
pub const WIDTH: usize = 16;
pub const N_MODES: usize = 4;
pub const N_BLOCKS: usize = 3;
pub const N_CHANNELS: usize = 8;
pub const N_CONDITIONS: usize = 8;
pub const FILM_HIDDEN: usize = 16;
pub const N_FRONT_COEFFS: usize = 8;
pub const N_VELOCITY_COEFFS: usize = 4;
pub const FILM_OUT: usize = 2 * WIDTH * N_BLOCKS;

pub fn nodes(k: usize) -> Vec<f64> {
    (0..k).map(|j| j as f64 / (k - 1) as f64).collect()
}

pub fn trapezoid_weights(k: usize) -> Vec<f64> {
    let mut q = vec![1.0 / (k - 1) as f64; k];
    q[0] = 1.0 / (2 * (k - 1)) as f64;
    q[k - 1] = q[0];
    q
}

/// φ_m(ξ) for each ξ: (len × n_modes) row-major; φ_0 = 1, φ_m = √2 cos(mπξ).
pub fn cosine_basis(xi: &[f64], n_modes: usize) -> Vec<f64> {
    let mut out = vec![0.0; xi.len() * n_modes];
    for (i, x) in xi.iter().enumerate() {
        out[i * n_modes] = 1.0;
        for m in 1..n_modes {
            out[i * n_modes + m] = 2f64.sqrt() * (std::f64::consts::PI * m as f64 * x).cos();
        }
    }
    out
}

/// Parameter blocks of one member's network (Python order).
pub fn param_shapes() -> Vec<(String, Vec<usize>)> {
    let mut s = vec![
        ("lift.W".to_string(), vec![WIDTH, N_CHANNELS]),
        ("lift.b".into(), vec![WIDTH]),
    ];
    for b in 0..N_BLOCKS {
        s.push((format!("block{b}.W"), vec![WIDTH, WIDTH]));
        s.push((format!("block{b}.b"), vec![WIDTH]));
        s.push((format!("block{b}.A"), vec![N_MODES, WIDTH, WIDTH]));
    }
    s.push(("head.W".into(), vec![1, WIDTH]));
    s.push(("head.b".into(), vec![1]));
    s.push(("film.W1".into(), vec![FILM_HIDDEN, N_CONDITIONS]));
    s.push(("film.b1".into(), vec![FILM_HIDDEN]));
    s.push(("film.W2".into(), vec![FILM_OUT, FILM_HIDDEN]));
    s.push(("film.b2".into(), vec![FILM_OUT]));
    s
}

pub fn n_weights() -> usize {
    param_shapes()
        .iter()
        .map(|(_, s)| s.iter().product::<usize>())
        .sum()
}

/// Offsets of every block inside a member's flat global vector (see `model::GlobalLayout`).
#[derive(Debug, Clone, Default)]
pub struct NetOffsets {
    pub lift_w: usize,
    pub lift_b: usize,
    pub blk_w: [usize; N_BLOCKS],
    pub blk_b: [usize; N_BLOCKS],
    pub blk_a: [usize; N_BLOCKS],
    pub head_w: usize,
    pub head_b: usize,
    pub film_w1: usize,
    pub film_b1: usize,
    pub film_w2: usize,
    pub film_b2: usize,
}

/// Fixed geometry of a K-node grid (basis, weighted basis).
#[derive(Debug, Clone)]
pub struct Grid {
    pub k: usize,
    pub xi: Vec<f64>,
    /// φ (K × M)
    pub phi: Vec<f64>,
    /// q φ (K × M)
    pub qphi: Vec<f64>,
    /// φ4 for velocity variation (K × 4)
    pub phi_v: Vec<f64>,
    /// φ8 for the initial front (K × 8)
    pub phi_a: Vec<f64>,
}

impl Grid {
    pub fn new(k: usize) -> Grid {
        let xi = nodes(k);
        let q = trapezoid_weights(k);
        let phi = cosine_basis(&xi, N_MODES);
        let qphi = (0..k * N_MODES).map(|i| phi[i] * q[i / N_MODES]).collect();
        Grid {
            phi_v: cosine_basis(&xi, N_VELOCITY_COEFFS),
            phi_a: cosine_basis(&xi, N_FRONT_COEFFS),
            k,
            xi,
            phi,
            qphi,
        }
    }
}

/// ĥ_m = Σ_j q_j φ_m(ξ_j) h_j for a column vector h (K) → (M).
pub fn coefficients(grid: &Grid, h: &[f64], n_modes: usize) -> Vec<f64> {
    let q = trapezoid_weights(grid.k);
    let phi = cosine_basis(&grid.xi, n_modes);
    (0..n_modes)
        .map(|m| {
            (0..grid.k)
                .map(|j| q[j] * phi[j * n_modes + m] * h[j])
                .sum()
        })
        .collect()
}

/// Activations of one forward evaluation (reused buffers).
#[derive(Debug, Clone, Default)]
pub struct Tape {
    pub chan: Vec<f64>,
    pub film_h: Vec<f64>,
    pub film_o: Vec<f64>,
    /// inputs of each block and the final hidden: (N_BLOCKS+1) × K × WIDTH
    pub h: Vec<f64>,
    /// pre-activations (W h + K h + b) per block: N_BLOCKS × K × WIDTH
    pub pre: Vec<f64>,
    /// spectral coefficients per block: N_BLOCKS × M × WIDTH
    pub hat: Vec<f64>,
    pub out: Vec<f64>,
}

/// Eight input channels from the physical front f (K) [m] with linear ghost extrapolation.
pub fn channels(f: &[f64], l: f64, grid: &Grid, chan: &mut Vec<f64>) {
    let k = f.len();
    chan.resize(k * N_CHANNELS, 0.0);
    let d = 1.0 / (k - 1) as f64;
    let eta = |i: isize| -> f64 {
        if i < 0 {
            2.0 * f[0] / l - f[1] / l
        } else if i as usize >= k {
            2.0 * f[k - 1] / l - f[k - 2] / l
        } else {
            f[i as usize] / l
        }
    };
    for j in 0..k {
        let (em, e0, ep) = (eta(j as isize - 1), f[j] / l, eta(j as isize + 1));
        let x = grid.xi[j];
        let c = &mut chan[j * N_CHANNELS..(j + 1) * N_CHANNELS];
        c[0] = e0;
        c[1] = (ep - em) / (2.0 * d);
        c[2] = (ep - 2.0 * e0 + em) / (d * d);
        c[3] = x;
        c[4] = 1.0;
        c[5] = 0.0;
        c[6] = 1.0;
        c[7] = x.min(1.0 - x);
    }
}

/// f̄ += ∂channels/∂f ᵀ chan_bar (only η, η_ξ, η_ξξ depend on f).
pub fn channels_backward(chan_bar: &[f64], k: usize, l: f64, f_bar: &mut [f64]) {
    let d = 1.0 / (k - 1) as f64;
    // adjoint w.r.t. the extended η (ghosts at −1 and K)
    let mut e = vec![0.0; k + 2];
    for j in 0..k {
        let c = &chan_bar[j * N_CHANNELS..j * N_CHANNELS + 3];
        e[j + 1] += c[0] - 2.0 * c[2] / (d * d);
        e[j + 2] += c[1] / (2.0 * d) + c[2] / (d * d);
        e[j] += -c[1] / (2.0 * d) + c[2] / (d * d);
    }
    let mut eta = vec![0.0; k];
    eta.copy_from_slice(&e[1..k + 1]);
    eta[0] += 2.0 * e[0];
    eta[1] -= e[0];
    eta[k - 1] += 2.0 * e[k + 1];
    eta[k - 2] -= e[k + 1];
    for j in 0..k {
        f_bar[j] += eta[j] / l;
    }
}

/// N_θ(f, c, G) at every node: channels in `tape.chan` (K × 8), c (8) → `tape.out` (K).
pub fn forward(p: &[f64], o: &NetOffsets, grid: &Grid, c: &[f64], tape: &mut Tape) {
    let k = grid.k;
    let w = WIDTH;
    tape.film_h.resize(FILM_HIDDEN, 0.0);
    tape.film_o.resize(FILM_OUT, 0.0);
    for i in 0..FILM_HIDDEN {
        let mut s = p[o.film_b1 + i];
        for j in 0..N_CONDITIONS {
            s += p[o.film_w1 + i * N_CONDITIONS + j] * c[j];
        }
        tape.film_h[i] = s.tanh();
    }
    for i in 0..FILM_OUT {
        let mut s = p[o.film_b2 + i];
        for j in 0..FILM_HIDDEN {
            s += p[o.film_w2 + i * FILM_HIDDEN + j] * tape.film_h[j];
        }
        tape.film_o[i] = s;
    }
    tape.h.resize((N_BLOCKS + 1) * k * w, 0.0);
    tape.pre.resize(N_BLOCKS * k * w, 0.0);
    tape.hat.resize(N_BLOCKS * N_MODES * w, 0.0);
    tape.out.resize(k, 0.0);
    {
        let h0 = &mut tape.h[..k * w];
        for j in 0..k {
            let ch = &tape.chan[j * N_CHANNELS..(j + 1) * N_CHANNELS];
            for i in 0..w {
                let mut s = p[o.lift_b + i];
                let wr = &p[o.lift_w + i * N_CHANNELS..o.lift_w + (i + 1) * N_CHANNELS];
                for cc in 0..N_CHANNELS {
                    s += wr[cc] * ch[cc];
                }
                h0[j * w + i] = s;
            }
        }
    }
    let mut y = [0.0f64; N_MODES * WIDTH];
    for b in 0..N_BLOCKS {
        let (hin_all, hout_all) = tape.h.split_at_mut((b + 1) * k * w);
        let hin = &hin_all[b * k * w..];
        let hout = &mut hout_all[..k * w];
        let hat = &mut tape.hat[b * N_MODES * w..(b + 1) * N_MODES * w];
        hat.iter_mut().for_each(|v| *v = 0.0);
        for j in 0..k {
            for m in 0..N_MODES {
                let qp = grid.qphi[j * N_MODES + m];
                for i in 0..w {
                    hat[m * w + i] += qp * hin[j * w + i];
                }
            }
        }
        // y[m][i] = Σ_j A[m][i][j] hat[m][j]
        for m in 0..N_MODES {
            for i in 0..w {
                let a = &p[o.blk_a[b] + (m * w + i) * w..o.blk_a[b] + (m * w + i + 1) * w];
                let mut s = 0.0;
                for jj in 0..w {
                    s += a[jj] * hat[m * w + jj];
                }
                y[m * w + i] = s;
            }
        }
        let pre = &mut tape.pre[b * k * w..(b + 1) * k * w];
        let gam = &tape.film_o[b * 2 * w..b * 2 * w + w];
        let bet = &tape.film_o[b * 2 * w + w..(b + 1) * 2 * w];
        for j in 0..k {
            let hr = &hin[j * w..(j + 1) * w];
            for i in 0..w {
                let wr = &p[o.blk_w[b] + i * w..o.blk_w[b] + (i + 1) * w];
                let mut s = p[o.blk_b[b] + i];
                for jj in 0..w {
                    s += wr[jj] * hr[jj];
                }
                for m in 0..N_MODES {
                    s += grid.phi[j * N_MODES + m] * y[m * w + i];
                }
                pre[j * w + i] = s;
                hout[j * w + i] = ((1.0 + gam[i]) * s + bet[i]).tanh();
            }
        }
    }
    let hl = &tape.h[N_BLOCKS * k * w..];
    for j in 0..k {
        let mut s = p[o.head_b];
        for i in 0..w {
            s += p[o.head_w + i] * hl[j * w + i];
        }
        tape.out[j] = s;
    }
}

/// Reverse pass: adds ∂/∂θ to `g` and writes ∂/∂chan into `chan_bar` (K × 8) for an output adjoint.
pub fn backward(
    p: &[f64],
    o: &NetOffsets,
    grid: &Grid,
    c: &[f64],
    tape: &Tape,
    out_bar: &[f64],
    g: &mut [f64],
    chan_bar: &mut Vec<f64>,
    scratch: &mut Vec<f64>,
) {
    let k = grid.k;
    let w = WIDTH;
    scratch.resize(2 * k * w, 0.0);
    let (hbar, hbar2) = scratch.split_at_mut(k * w);
    let hl = &tape.h[N_BLOCKS * k * w..];
    for j in 0..k {
        g[o.head_b] += out_bar[j];
        for i in 0..w {
            g[o.head_w + i] += out_bar[j] * hl[j * w + i];
            hbar[j * w + i] = out_bar[j] * p[o.head_w + i];
        }
    }
    let mut obar = [0.0f64; FILM_OUT];
    let mut ybar = [0.0f64; N_MODES * WIDTH];
    let mut hatbar = [0.0f64; N_MODES * WIDTH];
    let (mut cur, mut nxt) = (hbar, hbar2);
    for b in (0..N_BLOCKS).rev() {
        let hin = &tape.h[b * k * w..(b + 1) * k * w];
        let hout = &tape.h[(b + 1) * k * w..(b + 2) * k * w];
        let pre = &tape.pre[b * k * w..(b + 1) * k * w];
        let hat = &tape.hat[b * N_MODES * w..(b + 1) * N_MODES * w];
        let gam = &tape.film_o[b * 2 * w..b * 2 * w + w];
        ybar.iter_mut().for_each(|v| *v = 0.0);
        nxt.iter_mut().for_each(|v| *v = 0.0);
        for j in 0..k {
            for i in 0..w {
                let ho = hout[j * w + i];
                let z = cur[j * w + i] * (1.0 - ho * ho);
                obar[b * 2 * w + i] += z * pre[j * w + i];
                obar[b * 2 * w + w + i] += z;
                let pb = z * (1.0 + gam[i]);
                g[o.blk_b[b] + i] += pb;
                let wr = o.blk_w[b] + i * w;
                let hr = &hin[j * w..(j + 1) * w];
                for jj in 0..w {
                    g[wr + jj] += pb * hr[jj];
                    nxt[j * w + jj] += pb * p[wr + jj];
                }
                for m in 0..N_MODES {
                    ybar[m * w + i] += grid.phi[j * N_MODES + m] * pb;
                }
            }
        }
        hatbar.iter_mut().for_each(|v| *v = 0.0);
        for m in 0..N_MODES {
            for i in 0..w {
                let yb = ybar[m * w + i];
                let base = o.blk_a[b] + (m * w + i) * w;
                for jj in 0..w {
                    g[base + jj] += yb * hat[m * w + jj];
                    hatbar[m * w + jj] += yb * p[base + jj];
                }
            }
        }
        for j in 0..k {
            for m in 0..N_MODES {
                let qp = grid.qphi[j * N_MODES + m];
                for jj in 0..w {
                    nxt[j * w + jj] += qp * hatbar[m * w + jj];
                }
            }
        }
        std::mem::swap(&mut cur, &mut nxt);
    }
    // lift
    chan_bar.resize(k * N_CHANNELS, 0.0);
    chan_bar.iter_mut().for_each(|v| *v = 0.0);
    for j in 0..k {
        let ch = &tape.chan[j * N_CHANNELS..(j + 1) * N_CHANNELS];
        for i in 0..w {
            let hb = cur[j * w + i];
            g[o.lift_b + i] += hb;
            for cc in 0..N_CHANNELS {
                g[o.lift_w + i * N_CHANNELS + cc] += hb * ch[cc];
                chan_bar[j * N_CHANNELS + cc] += hb * p[o.lift_w + i * N_CHANNELS + cc];
            }
        }
    }
    // FiLM
    let mut hfbar = [0.0f64; FILM_HIDDEN];
    for i in 0..FILM_OUT {
        g[o.film_b2 + i] += obar[i];
        for j in 0..FILM_HIDDEN {
            g[o.film_w2 + i * FILM_HIDDEN + j] += obar[i] * tape.film_h[j];
            hfbar[j] += obar[i] * p[o.film_w2 + i * FILM_HIDDEN + j];
        }
    }
    for i in 0..FILM_HIDDEN {
        let z = hfbar[i] * (1.0 - tape.film_h[i] * tape.film_h[i]);
        g[o.film_b1 + i] += z;
        for j in 0..N_CONDITIONS {
            g[o.film_w1 + i * N_CONDITIONS + j] += z * c[j];
        }
    }
}

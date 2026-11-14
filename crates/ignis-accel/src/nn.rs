//! Prefix encoder (V3.1 §10.3) and conditional rational-quadratic coupling flow (§10.4) in candle
//! (autograd; FP32 training by default, FP64 for checks; CPU, or CUDA with the `cuda` feature).
//!
//! Encoder: token (24) → projection 24→64 (1600), two pre-LN transformer blocks (width 64, 4 heads,
//! FFN 128, tanh-GELU; 66944), final LayerNorm (128), masked mean pooling, projection [64 ‖ 16] → 64
//! (5184): 73856. Attention runs among ADMITTED tokens only (padding masked out of keys and pooling).
//! Flow: 8 coupling layers alternating the transformed dimensions; conditioner
//! [unchanged ‖ context64 ‖ global summary16] → 64 → 64 → (#transformed · 23), tanh hidden units;
//! splines with 8 bins on [−5, 5], identity tails, boundary slopes 1 (`ignis_science::flows`). The
//! last conditioner layer starts at zero with a derivative offset, so the initial flow is the identity.
//! Counts (encoder + flow): FLEX d=3 → 167316, ACME d=2 → 161080, BASS d=12 → 223440.
//!
//! Density convention: z are standardised local coordinates (z = L⁻¹u for ACME frailties, x/σ_prior
//! for BASS locals); q(local) = q_z(z) · |det ∂z/∂local|, which the caller adds explicitly.

use std::collections::BTreeMap;

use candle_core::{DType, Device, Result, Tensor, Var, D};
use ignis_science::flows as fl;
use rand::{rngs::StdRng, RngExt, SeedableRng};
use rand_distr::StandardNormal;

pub const TOKEN_DIM: usize = 24;
pub const META_DIM: usize = 16;
pub const WIDTH: usize = 64;
pub const HEADS: usize = 4;
pub const FFN: usize = 128;
pub const BLOCKS: usize = 2;
pub const LAYERS: usize = 8;
pub const HIDDEN: usize = 64;
pub const GLOBAL_DIM: usize = 16;
pub const BINS: usize = fl::DEFAULT_BINS;
pub const BOUND: f64 = fl::DEFAULT_BOUND;

pub const TOKEN_FIELDS: [&str; 24] = [
    "log1p_t",
    "log1p_gap",
    "value",
    "log_scale",
    "scale_known",
    "valid",
    "phase_confirmed",
    "control_change_known",
    "channel_0",
    "channel_1",
    "channel_2",
    "channel_3",
    "channel_4",
    "channel_5",
    "channel_6",
    "channel_7",
    "condition_0",
    "condition_1",
    "condition_2",
    "condition_3",
    "condition_4",
    "condition_5",
    "condition_6",
    "condition_7",
];
pub const METADATA_FIELDS: [&str; 16] = [
    "condition_0",
    "condition_1",
    "condition_2",
    "condition_3",
    "condition_4",
    "condition_5",
    "condition_6",
    "condition_7",
    "cutoff",
    "log1p_n_obs",
    "first_time",
    "last_time",
    "missing_fraction",
    "spatial_calibration",
    "temporal_calibration",
    "phase_confirmed",
];

pub fn encoder_shapes() -> Vec<(String, Vec<usize>)> {
    let mut s = vec![
        ("enc.proj.W".to_string(), vec![WIDTH, TOKEN_DIM]),
        ("enc.proj.b".into(), vec![WIDTH]),
    ];
    for b in 0..BLOCKS {
        let p = format!("enc.block{b}.");
        for n in ["ln1.g", "ln1.b", "ln2.g", "ln2.b"] {
            s.push((p.clone() + n, vec![WIDTH]));
        }
        for n in ["q", "k", "v", "o"] {
            s.push((format!("{p}attn.{n}.W"), vec![WIDTH, WIDTH]));
            s.push((format!("{p}attn.{n}.b"), vec![WIDTH]));
        }
        s.push((p.clone() + "ffn.W1", vec![FFN, WIDTH]));
        s.push((p.clone() + "ffn.b1", vec![FFN]));
        s.push((p.clone() + "ffn.W2", vec![WIDTH, FFN]));
        s.push((p + "ffn.b2", vec![WIDTH]));
    }
    s.push(("enc.ln.g".into(), vec![WIDTH]));
    s.push(("enc.ln.b".into(), vec![WIDTH]));
    s.push(("enc.ctx.W".into(), vec![WIDTH, WIDTH + META_DIM]));
    s.push(("enc.ctx.b".into(), vec![WIDTH]));
    s
}

fn layer_split(dim: usize, layer: usize) -> (Vec<u32>, Vec<u32>) {
    let m = &fl::coupling_masks(dim, LAYERS)[layer];
    (
        (0..dim as u32).filter(|i| m[*i as usize]).collect(),
        (0..dim as u32).filter(|i| !m[*i as usize]).collect(),
    )
}

pub fn flow_shapes(dim: usize) -> Vec<(String, Vec<usize>)> {
    let per = fl::n_spline_params(BINS);
    let mut s = vec![];
    for i in 0..LAYERS {
        let (t, u) = layer_split(dim, i);
        let din = u.len() + WIDTH + GLOBAL_DIM;
        let p = format!("flow.layer{i}.");
        s.push((p.clone() + "W1", vec![HIDDEN, din]));
        s.push((p.clone() + "b1", vec![HIDDEN]));
        s.push((p.clone() + "W2", vec![HIDDEN, HIDDEN]));
        s.push((p.clone() + "b2", vec![HIDDEN]));
        s.push((p.clone() + "W3", vec![t.len() * per, HIDDEN]));
        s.push((p + "b3", vec![t.len() * per]));
    }
    s
}

fn count(s: &[(String, Vec<usize>)]) -> usize {
    s.iter().map(|(_, v)| v.iter().product::<usize>()).sum()
}
pub fn encoder_params() -> usize {
    count(&encoder_shapes())
}
pub fn flow_params(dim: usize) -> usize {
    count(&flow_shapes(dim))
}
pub fn nominal_local_dim(family: &str) -> Option<usize> {
    match family {
        "flex" => Some(3),
        "acme_normal" | "acme_inverse" => Some(2),
        "bass" => Some(12),
        _ => None,
    }
}
pub fn total_params(family: &str) -> usize {
    encoder_params() + flow_params(nominal_local_dim(family).unwrap())
}

/// Trainable parameters by name.
pub struct Params {
    pub vars: BTreeMap<String, Var>,
    pub dim: usize,
    pub dtype: DType,
    pub device: Device,
}

impl Params {
    /// Initialisation of the Python reference (own RNG): LN gains 1, biases 0, weights N(0, 1/fan_in),
    /// last conditioner layer zero with a softplus⁻¹ derivative offset (identity flow).
    pub fn init(dim: usize, seed: u64, dtype: DType, device: &Device) -> Result<Params> {
        let mut rng = StdRng::seed_from_u64(seed);
        let d_offset = (1.0 - fl::MIN_DERIVATIVE).exp_m1().ln();
        let per = fl::n_spline_params(BINS);
        let mut vars = BTreeMap::new();
        for (name, shape) in encoder_shapes().into_iter().chain(flow_shapes(dim)) {
            let n: usize = shape.iter().product();
            let data: Vec<f64> = if name.ends_with(".g") {
                vec![1.0; n]
            } else if name.ends_with("W3") {
                vec![0.0; n]
            } else if name.ends_with("b3") {
                (0..n)
                    .map(|i| if i % per >= 2 * BINS { d_offset } else { 0.0 })
                    .collect()
            } else if shape.len() == 1 {
                vec![0.0; n]
            } else {
                let fan = shape[1] as f64;
                (0..n)
                    .map(|_| rng.sample::<f64, _>(StandardNormal) / fan.sqrt())
                    .collect()
            };
            let t = Tensor::from_vec(data, shape, device)?.to_dtype(dtype)?;
            vars.insert(name, Var::from_tensor(&t)?);
        }
        Ok(Params {
            vars,
            dim,
            dtype,
            device: device.clone(),
        })
    }
    pub fn from_f64(
        map: &BTreeMap<String, (Vec<usize>, Vec<f64>)>,
        dim: usize,
        dtype: DType,
        device: &Device,
    ) -> Result<Params> {
        let mut vars = BTreeMap::new();
        for (k, (s, v)) in map {
            vars.insert(
                k.clone(),
                Var::from_tensor(
                    &Tensor::from_vec(v.clone(), s.clone(), device)?.to_dtype(dtype)?,
                )?,
            );
        }
        Ok(Params {
            vars,
            dim,
            dtype,
            device: device.clone(),
        })
    }
    pub fn to_f64(&self) -> Result<BTreeMap<String, (Vec<usize>, Vec<f64>)>> {
        self.vars
            .iter()
            .map(|(k, v)| {
                Ok((
                    k.clone(),
                    (
                        v.dims().to_vec(),
                        v.as_tensor()
                            .to_dtype(DType::F64)?
                            .flatten_all()?
                            .to_vec1::<f64>()?,
                    ),
                ))
            })
            .collect()
    }
    fn t(&self, n: &str) -> &Tensor {
        self.vars
            .get(n)
            .unwrap_or_else(|| panic!("parameter {n}"))
            .as_tensor()
    }
    pub fn all_vars(&self) -> Vec<Var> {
        self.vars.values().cloned().collect()
    }
}

fn lin(p: &Params, name: &str, x: &Tensor) -> Result<Tensor> {
    x.broadcast_matmul(&p.t(&format!("{name}.W")).t()?)?
        .broadcast_add(p.t(&format!("{name}.b")))
}

fn layer_norm(x: &Tensor, g: &Tensor, b: &Tensor) -> Result<Tensor> {
    let mu = x.mean_keepdim(D::Minus1)?;
    let xc = x.broadcast_sub(&mu)?;
    let var = xc.sqr()?.mean_keepdim(D::Minus1)?;
    xc.broadcast_div(&(var + 1e-5)?.sqrt()?)?
        .broadcast_mul(g)?
        .broadcast_add(b)
}

/// tokens (B, T, 24), mask (B, T) in {0, 1}, meta (B, 16) → context (B, 64).
pub fn encode(p: &Params, tokens: &Tensor, mask: &Tensor, meta: &Tensor) -> Result<Tensor> {
    let (b, t, _) = tokens.dims3()?;
    let dh = WIDTH / HEADS;
    let mut h = lin(p, "enc.proj", tokens)?;
    let keys = mask
        .ne(0.0)?
        .reshape((b, 1, 1, t))?
        .broadcast_as((b, HEADS, t, t))?;
    let neg = Tensor::full(-1e30f64, (b, HEADS, t, t), &p.device)?.to_dtype(p.dtype)?;
    for blk in 0..BLOCKS {
        let pre = format!("enc.block{blk}.");
        let x = layer_norm(&h, p.t(&format!("{pre}ln1.g")), p.t(&format!("{pre}ln1.b")))?;
        let split = |n: &str| -> Result<Tensor> {
            lin(p, &format!("{pre}attn.{n}"), &x)?
                .reshape((b, t, HEADS, dh))?
                .transpose(1, 2)?
                .contiguous()
        };
        let (q, k, v) = (split("q")?, split("k")?, split("v")?);
        let logits = (q.matmul(&k.transpose(2, 3)?.contiguous()?)? / (dh as f64).sqrt())?;
        let logits = keys.where_cond(&logits, &neg)?;
        let att = candle_nn::ops::softmax(&logits, D::Minus1)?;
        let o = att
            .matmul(&v)?
            .transpose(1, 2)?
            .contiguous()?
            .reshape((b, t, WIDTH))?;
        h = (h + lin(p, &format!("{pre}attn.o"), &o)?)?;
        let x = layer_norm(&h, p.t(&format!("{pre}ln2.g")), p.t(&format!("{pre}ln2.b")))?;
        let f1 = x
            .broadcast_matmul(&p.t(&format!("{pre}ffn.W1")).t()?)?
            .broadcast_add(p.t(&format!("{pre}ffn.b1")))?
            .gelu()?;
        let f2 = f1
            .broadcast_matmul(&p.t(&format!("{pre}ffn.W2")).t()?)?
            .broadcast_add(p.t(&format!("{pre}ffn.b2")))?;
        h = (h + f2)?;
    }
    let h = layer_norm(&h, p.t("enc.ln.g"), p.t("enc.ln.b"))?;
    let m = mask.unsqueeze(2)?;
    let pooled = h
        .broadcast_mul(&m)?
        .sum(1)?
        .broadcast_div(&m.sum(1)?.maximum(1.0)?)?;
    lin(p, "enc.ctx", &Tensor::cat(&[&pooled, meta], 1)?)
}

fn softplus(x: &Tensor) -> Result<Tensor> {
    // max(x, 0) + log(1 + exp(−|x|))
    x.relu()? + (x.abs()?.neg()?.exp()? + 1.0)?.log()?
}

struct Knots {
    cw: Tensor,
    w: Tensor,
    ch: Tensor,
    h: Tensor,
    d: Tensor,
}

fn knots(u: &Tensor, minimum: f64) -> Result<(Tensor, Tensor)> {
    let (b, n, k) = u.dims3()?;
    let size = ((candle_nn::ops::softmax(u, D::Minus1)? * (1.0 - minimum * k as f64))? + minimum)?;
    let cum = size.cumsum(2)?;
    let inner = ((cum.narrow(2, 0, k - 1)? * (2.0 * BOUND))? - BOUND)?;
    let lo = (Tensor::ones((b, n, 1), u.dtype(), u.device())? * -BOUND)?;
    let hi = (Tensor::ones((b, n, 1), u.dtype(), u.device())? * BOUND)?;
    let c = Tensor::cat(&[&lo, &inner, &hi], 2)?;
    let w = (c.narrow(2, 1, k)? - c.narrow(2, 0, k)?)?;
    Ok((c, w))
}

fn parts(raw: &Tensor) -> Result<Knots> {
    let (b, n, _) = raw.dims3()?;
    let (cw, w) = knots(&raw.narrow(2, 0, BINS)?, fl::MIN_WIDTH)?;
    let (ch, h) = knots(&raw.narrow(2, BINS, BINS)?, fl::MIN_HEIGHT)?;
    let inner = (softplus(&raw.narrow(2, 2 * BINS, BINS - 1)?)? + fl::MIN_DERIVATIVE)?;
    let one = Tensor::ones((b, n, 1), raw.dtype(), raw.device())?;
    Ok(Knots {
        cw,
        w,
        ch,
        h,
        d: Tensor::cat(&[&one, &inner, &one], 2)?,
    })
}

/// (k, k+1) bin indices (B, n, 1), no gradient.
fn bin_index(knots: &Tensor, v: &Tensor) -> Result<(Tensor, Tensor)> {
    let interior = knots.narrow(2, 1, BINS - 1)?;
    let ge = v
        .unsqueeze(2)?
        .broadcast_ge(&interior)?
        .to_dtype(DType::F32)?
        .sum_keepdim(2)?;
    let k = ge.clamp(0f32, (BINS - 1) as f32)?;
    Ok((k.to_dtype(DType::U32)?, (k + 1.0)?.to_dtype(DType::U32)?))
}

fn take(t: &Tensor, idx: &Tensor) -> Result<Tensor> {
    t.gather(idx, 2)?.squeeze(2)
}

/// Elementwise y = g(x): x (B, n), raw (B, n, 23) → (y, log dy/dx).
pub fn rq_forward(x: &Tensor, raw: &Tensor) -> Result<(Tensor, Tensor)> {
    let k = parts(raw)?;
    let inside = x.abs()?.lt(BOUND)?;
    let xc = x.clamp(-BOUND, BOUND)?;
    let (idx, idx1) = bin_index(&k.cw, &xc)?;
    let (x0, wk, y0, hk, d0, d1) = (
        take(&k.cw, &idx)?,
        take(&k.w, &idx)?,
        take(&k.ch, &idx)?,
        take(&k.h, &idx)?,
        take(&k.d, &idx)?,
        take(&k.d, &idx1)?,
    );
    let delta = (&hk / &wk)?;
    let xi = ((&xc - &x0)? / &wk)?.clamp(0.0, 1.0)?;
    let om = (&xi * (1.0 - &xi)?)?;
    let s = ((&d1 + &d0)? - (&delta * 2.0)?)?;
    let den = (&delta + (&s * &om)?)?;
    let y = (&y0 + (&hk * ((&delta * xi.sqr()?)? + (&d0 * &om)?)?)?.div(&den)?)?;
    let num = ((&d1 * xi.sqr()?)? + ((&delta * 2.0)? * &om)?)?;
    let num = (num + (&d0 * (1.0 - &xi)?.sqr()?)?)?;
    let dydx = ((delta.sqr()? * num)? / den.sqr()?)?;
    Ok((
        inside.where_cond(&y, x)?,
        inside.where_cond(&dydx.log()?, &x.zeros_like()?)?,
    ))
}

/// x = g⁻¹(y): returns (x, log dx/dy).
pub fn rq_inverse(y: &Tensor, raw: &Tensor) -> Result<(Tensor, Tensor)> {
    let k = parts(raw)?;
    let inside = y.abs()?.lt(BOUND)?;
    let yc = y.clamp(-BOUND, BOUND)?;
    let (idx, idx1) = bin_index(&k.ch, &yc)?;
    let (x0, wk, y0, hk, d0, d1) = (
        take(&k.cw, &idx)?,
        take(&k.w, &idx)?,
        take(&k.ch, &idx)?,
        take(&k.h, &idx)?,
        take(&k.d, &idx)?,
        take(&k.d, &idx1)?,
    );
    let delta = (&hk / &wk)?;
    let dy = (&yc - &y0)?;
    let s = ((&d1 + &d0)? - (&delta * 2.0)?)?;
    let a = ((&hk * (&delta - &d0)?)? + (&dy * &s)?)?;
    let b = ((&hk * &d0)? - (&dy * &s)?)?;
    let c = (&delta * &dy)?.neg()?;
    let disc = (b.sqr()? - ((&a * &c)? * 4.0)?)?.relu()?;
    let xi = ((&c * 2.0)? / (b.neg()? - disc.sqrt()?)?)?.clamp(0.0, 1.0)?;
    let x = (&x0 + (&xi * &wk)?)?;
    let om = (&xi * (1.0 - &xi)?)?;
    let den = (&delta + (&s * &om)?)?;
    let num = ((&d1 * xi.sqr()?)? + ((&delta * 2.0)? * &om)?)?;
    let num = (num + (&d0 * (1.0 - &xi)?.sqr()?)?)?;
    let dydx = ((delta.sqr()? * num)? / den.sqr()?)?;
    Ok((
        inside.where_cond(&x, y)?,
        inside.where_cond(&dydx.log()?.neg()?, &y.zeros_like()?)?,
    ))
}

fn conditioner(
    p: &Params,
    i: usize,
    x_u: Option<&Tensor>,
    ctx: &Tensor,
    gsum: &Tensor,
    dt: usize,
) -> Result<Tensor> {
    let pre = format!("flow.layer{i}.");
    let inp = match x_u {
        Some(u) => Tensor::cat(&[u, ctx, gsum], 1)?,
        None => Tensor::cat(&[ctx, gsum], 1)?,
    };
    let h = inp
        .broadcast_matmul(&p.t(&format!("{pre}W1")).t()?)?
        .broadcast_add(p.t(&format!("{pre}b1")))?
        .tanh()?;
    let h = h
        .broadcast_matmul(&p.t(&format!("{pre}W2")).t()?)?
        .broadcast_add(p.t(&format!("{pre}b2")))?
        .tanh()?;
    let o = h
        .broadcast_matmul(&p.t(&format!("{pre}W3")).t()?)?
        .broadcast_add(p.t(&format!("{pre}b3")))?;
    let b = o.dim(0)?;
    o.reshape((b, dt, fl::n_spline_params(BINS)))
}

fn assemble(dim: usize, t_idx: &[u32], new_t: &Tensor, z: &Tensor) -> Result<Tensor> {
    // the transformed/unchanged sets are contiguous ranges (first ⌊d/2⌋ or the rest)
    if t_idx.len() == dim {
        return Ok(new_t.clone());
    }
    let first = t_idx[0] as usize;
    if first == 0 {
        Tensor::cat(&[new_t, &z.narrow(1, t_idx.len(), dim - t_idx.len())?], 1)
    } else {
        Tensor::cat(&[&z.narrow(1, 0, first)?, new_t], 1)
    }
}

/// ε = T(z) and log|det ∂ε/∂z| for a batch z (B, d).
pub fn to_base(p: &Params, z: &Tensor, ctx: &Tensor, gsum: &Tensor) -> Result<(Tensor, Tensor)> {
    let dim = p.dim;
    let b = z.dim(0)?;
    let mut z = z.clone();
    let mut logdet = Tensor::zeros(b, p.dtype, &p.device)?;
    for i in 0..LAYERS {
        let (t, u) = layer_split(dim, i);
        let ti = Tensor::new(t.as_slice(), &p.device)?;
        let xu = if u.is_empty() {
            None
        } else {
            Some(z.index_select(&Tensor::new(u.as_slice(), &p.device)?, 1)?)
        };
        let raw = conditioner(p, i, xu.as_ref(), ctx, gsum, t.len())?;
        let (y, ld) = rq_forward(&z.index_select(&ti, 1)?, &raw)?;
        z = assemble(dim, &t, &y, &z)?;
        logdet = (logdet + ld.sum(1)?)?;
    }
    Ok((z, logdet))
}

/// z = T⁻¹(ε) and log|det ∂z/∂ε|.
pub fn from_base(
    p: &Params,
    eps: &Tensor,
    ctx: &Tensor,
    gsum: &Tensor,
) -> Result<(Tensor, Tensor)> {
    let dim = p.dim;
    let b = eps.dim(0)?;
    let mut z = eps.clone();
    let mut logdet = Tensor::zeros(b, p.dtype, &p.device)?;
    for i in (0..LAYERS).rev() {
        let (t, u) = layer_split(dim, i);
        let ti = Tensor::new(t.as_slice(), &p.device)?;
        let xu = if u.is_empty() {
            None
        } else {
            Some(z.index_select(&Tensor::new(u.as_slice(), &p.device)?, 1)?)
        };
        let raw = conditioner(p, i, xu.as_ref(), ctx, gsum, t.len())?;
        let (x, ld) = rq_inverse(&z.index_select(&ti, 1)?, &raw)?;
        z = assemble(dim, &t, &x, &z)?;
        logdet = (logdet + ld.sum(1)?)?;
    }
    Ok((z, logdet))
}

fn std_normal_logpdf(e: &Tensor) -> Result<Tensor> {
    ((e.sqr()? * -0.5)? - 0.5 * (2.0 * std::f64::consts::PI).ln())?.sum(1)
}

/// log q(z | context) for a batch.
pub fn log_prob(p: &Params, z: &Tensor, ctx: &Tensor, gsum: &Tensor) -> Result<Tensor> {
    let (e, ld) = to_base(p, z, ctx, gsum)?;
    std_normal_logpdf(&e)? + ld
}

/// Samples z (B, d) and their exact log q(z), from base draws `eps`.
pub fn sample_from(
    p: &Params,
    eps: &Tensor,
    ctx: &Tensor,
    gsum: &Tensor,
) -> Result<(Tensor, Tensor)> {
    let (z, ld) = from_base(p, eps, ctx, gsum)?;
    Ok((z, (std_normal_logpdf(eps)? - ld)?))
}

/// Mean −log q(latent | prefix) of a batch (training objective).
pub fn batch_nll(
    p: &Params,
    tokens: &Tensor,
    mask: &Tensor,
    meta: &Tensor,
    gsum: &Tensor,
    latent: &Tensor,
) -> Result<Tensor> {
    let ctx = encode(p, tokens, mask, meta)?;
    log_prob(p, latent, &ctx, gsum)?.mean_all()?.neg()
}

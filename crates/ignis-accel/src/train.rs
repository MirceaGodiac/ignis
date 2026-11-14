//! Train the optional encoder + spline flow on model-generated tasks (V3.1 §10.6, §12.6). FP32 by
//! default. Objective: mean −log q(latent | prefix) of the KNOWN generated latent. Adam (candle-nn
//! AdamW with zero decay) after global-norm clipping, as the optax reference. The result is a release
//! with `trained_on = "synthetic"`; it never changes the expert's target density.

use std::path::Path;
use std::time::Instant;

use anyhow::Context;
use candle_core::{backprop::GradStore, DType, Device, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use ignis_core::contracts::{Family, ModelRelease};
use ignis_core::registry::{save_release, Tensor as RTensor, TensorMap};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use serde_json::{json, Value};

use crate::nn::{self, Params};
use crate::tasks::Arrays;

pub fn device(gpu: bool) -> anyhow::Result<Device> {
    if gpu {
        Ok(Device::new_cuda(0).context("CUDA device (build with --features cuda)")?)
    } else {
        Ok(Device::Cpu)
    }
}

/// Rows `idx` of the task arrays as tensors (tokens, mask, meta, gsum, latent).
pub fn batch(
    arr: &Arrays,
    idx: &[usize],
    dtype: DType,
    dev: &Device,
) -> anyhow::Result<[Tensor; 5]> {
    let take = |name: &str| -> anyhow::Result<Tensor> {
        let (shape, v) = &arr[name];
        let row: usize = shape[1..].iter().product();
        let mut data = Vec::with_capacity(idx.len() * row);
        for &i in idx {
            data.extend_from_slice(&v[i * row..(i + 1) * row]);
        }
        let mut s = vec![idx.len()];
        s.extend(&shape[1..]);
        Ok(Tensor::from_vec(data, s, dev)?.to_dtype(dtype)?)
    };
    Ok([
        take("tokens")?,
        take("mask")?,
        take("meta")?,
        take("gsum")?,
        take("latent")?,
    ])
}

pub fn nll(p: &Params, b: &[Tensor; 5]) -> anyhow::Result<Tensor> {
    Ok(nn::batch_nll(p, &b[0], &b[1], &b[2], &b[3], &b[4])?)
}

fn clip(grads: &mut GradStore, vars: &[candle_core::Var], max_norm: f64) -> anyhow::Result<f64> {
    let mut sq = 0.0;
    for v in vars {
        if let Some(g) = grads.get(v.as_tensor()) {
            sq += g
                .sqr()?
                .sum_all()?
                .to_dtype(DType::F64)?
                .to_scalar::<f64>()?;
        }
    }
    let norm = sq.sqrt();
    if norm > max_norm {
        let s = max_norm / norm;
        for v in vars {
            if let Some(g) = grads.remove(v.as_tensor()) {
                grads.insert(v.as_tensor(), (g * s)?);
            }
        }
    }
    Ok(norm)
}

pub struct TrainResult {
    pub params: Params,
    pub history: Vec<f64>,
    pub validation_nll: f64,
    pub n_validation: usize,
    pub seconds: f64,
}

pub fn train(arr: &Arrays, dim: usize, cfg: &Value, gpu: bool) -> anyhow::Result<TrainResult> {
    let t0 = Instant::now();
    let dtype = if cfg["precision"].as_str().unwrap_or("fp32") == "fp32" {
        DType::F32
    } else {
        DType::F64
    };
    let dev = device(gpu)?;
    let seed = cfg["seed"].as_u64().unwrap_or(0);
    let p = Params::init(dim, seed, dtype, &dev)?;
    let n = arr["latent"].0[0];
    let n_val = ((cfg["validation_fraction"].as_f64().unwrap_or(0.1) * n as f64) as usize).max(1);
    let mut idx: Vec<usize> = (0..n).collect();
    let mut rng = StdRng::seed_from_u64(seed);
    idx.shuffle(&mut rng);
    let (va, tr) = idx.split_at(n_val);
    let vars = p.all_vars();
    let mut opt = AdamW::new(
        vars.clone(),
        ParamsAdamW {
            lr: cfg["lr"].as_f64().unwrap_or(1e-3),
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.0,
        },
    )?;
    let bs = (cfg["batch_size"].as_u64().unwrap_or(256) as usize).min(tr.len());
    let steps = cfg["steps"].as_u64().unwrap_or(10000) as usize;
    let max_norm = cfg["clip_norm"].as_f64().unwrap_or(1.0);
    let mut hist = Vec::with_capacity(steps);
    let mut pool: Vec<usize> = tr.to_vec();
    let mut pos = pool.len();
    for _ in 0..steps {
        if pos + bs > pool.len() {
            pool.shuffle(&mut rng);
            pos = 0;
        }
        let b = batch(arr, &pool[pos..pos + bs], dtype, &dev)?;
        pos += bs;
        let loss = nll(&p, &b)?;
        hist.push(loss.to_dtype(DType::F64)?.to_scalar::<f64>()?);
        let mut grads = loss.backward()?;
        clip(&mut grads, &vars, max_norm)?;
        opt.step(&grads)?;
    }
    let mut vsum = 0.0;
    for chunk in va.chunks(1024) {
        let b = batch(arr, chunk, dtype, &dev)?;
        vsum += nll(&p, &b)?.to_dtype(DType::F64)?.to_scalar::<f64>()? * chunk.len() as f64;
    }
    Ok(TrainResult {
        params: p,
        history: hist,
        validation_nll: vsum / va.len() as f64,
        n_validation: va.len(),
        seconds: t0.elapsed().as_secs_f64(),
    })
}

pub fn save(
    root: &Path,
    res: &TrainResult,
    manifest: &Value,
    cfg: &Value,
) -> anyhow::Result<ModelRelease> {
    let fam: Family = serde_json::from_value(manifest["family"].clone())?;
    let dim = res.params.dim;
    let mut rel = ignis_acme::common::release(
        fam,
        &format!("accel.{}.encoder_flow", fam.as_str()),
        false,
        nn::TOKEN_FIELDS.iter().map(|s| s.to_string()).collect(),
        "model-generated",
        "model-generated",
        manifest["release_id"].as_str().unwrap_or(""),
        true,
    );
    rel.architecture = ignis_acme::common::obj(
        json!({"encoder_params": nn::encoder_params(), "flow_params": nn::flow_params(dim), "latent_dim": dim,
        "coupling_layers": nn::LAYERS, "bins": nn::BINS, "tail_bound": nn::BOUND, "token_fields": nn::TOKEN_FIELDS,
        "metadata_fields": nn::METADATA_FIELDS, "for_release": manifest["release_id"]}),
    );
    rel.config = ignis_acme::common::obj(cfg.clone());
    rel.numerics = ignis_acme::common::obj(
        json!({"precision": cfg["precision"].as_str().unwrap_or("fp32"), "steps": res.history.len(),
        "final_train_nll": res.history.last(), "seconds": res.seconds, "steps_per_s": res.history.len() as f64 / res.seconds.max(1e-9)}),
    );
    rel.applicability = ignis_acme::common::obj(
        json!({"role": "proposal only; runtime reweights to the expert target with a defensive mixture", "tasks": manifest["label"]}),
    );
    rel.evaluation = ignis_acme::common::obj(
        json!({"validation_nll": res.validation_nll, "n_validation": res.n_validation}),
    );
    let params: TensorMap = res
        .params
        .to_f64()?
        .into_iter()
        .map(|(k, (s, v))| (k, RTensor::new(s, v)))
        .collect();
    save_release(root, rel, &params, &TensorMap::new(), &[])
}

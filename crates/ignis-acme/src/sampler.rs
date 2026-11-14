//! NUTS FP64 on the unconstrained population vector via `nuts-rs` (diagonal mass matrix, dual
//! averaging), chains in parallel. Thin wrapper; to be consolidated with `ignis_science::nuts`.

use std::collections::HashMap;

use nuts_rs::{
    Chain, CpuLogpFunc, CpuMath, CpuMathError, DiagNutsSettings, HasDims, LogpError, Settings,
};
use rand::{rngs::StdRng, RngExt, SeedableRng};
use rand_distr::StandardNormal;
use rayon::prelude::*;
use serde_json::{json, Value};

use crate::optim::{effective_sample_size, split_gelman_rubin};

#[derive(Debug, thiserror::Error)]
#[error("non-finite log density")]
pub struct NonFinite;
impl LogpError for NonFinite {
    fn is_recoverable(&self) -> bool {
        true
    }
}

struct Density<'a, F: Fn(&[f64], &mut [f64]) -> f64 + Sync> {
    f: &'a F,
    dim: usize,
}

impl<F: Fn(&[f64], &mut [f64]) -> f64 + Sync> HasDims for Density<'_, F> {
    fn dim_sizes(&self) -> HashMap<String, u64> {
        [("unconstrained_parameter".to_string(), self.dim as u64)]
            .into_iter()
            .collect()
    }
}

impl<F: Fn(&[f64], &mut [f64]) -> f64 + Sync> CpuLogpFunc for Density<'_, F> {
    type LogpError = NonFinite;
    type FlowParameters = ();
    type ExpandedVector = Vec<f64>;
    fn dim(&self) -> usize {
        self.dim
    }
    fn logp(&mut self, x: &[f64], g: &mut [f64]) -> Result<f64, NonFinite> {
        let v = (self.f)(x, g);
        if v.is_finite() && g.iter().all(|x| x.is_finite()) {
            Ok(v)
        } else {
            Err(NonFinite)
        }
    }
    fn expand_vector<R: nuts_rs::rand::Rng + ?Sized>(
        &mut self,
        _rng: &mut R,
        x: &[f64],
    ) -> Result<Vec<f64>, CpuMathError> {
        Ok(x.to_vec())
    }
}

pub struct NutsConfig {
    pub num_warmup: u64,
    pub num_samples: u64,
    pub num_chains: usize,
    pub target_accept: f64,
    pub seed: u64,
    pub max_rhat: f64,
    pub min_ess: f64,
    pub max_divergences: usize,
}

impl NutsConfig {
    pub fn from_json(c: &Value) -> NutsConfig {
        let f = |k: &str, d: f64| c.get(k).and_then(|v| v.as_f64()).unwrap_or(d);
        NutsConfig {
            num_warmup: f("num_warmup", 500.0) as u64,
            num_samples: f("num_samples", 500.0) as u64,
            num_chains: f("num_chains", 4.0) as usize,
            target_accept: f("target_accept", 0.85),
            seed: f("seed", 0.0) as u64,
            max_rhat: f("max_rhat", 1.01),
            min_ess: f("min_ess", 400.0),
            max_divergences: f("max_divergences", 0.0) as usize,
        }
    }
}

/// Sample `logp` (value + gradient) with chains started at `init` + 1e-3 jitter. Returns the flat draws
/// (chains concatenated) and the qualification diagnostics.
pub fn nuts<F: Fn(&[f64], &mut [f64]) -> f64 + Sync>(
    logp: &F,
    init: &[f64],
    cfg: &NutsConfig,
) -> (Vec<Vec<f64>>, Value) {
    let dim = init.len();
    let t0 = std::time::Instant::now();
    let chains: Vec<anyhow::Result<(Vec<Vec<f64>>, usize, u64)>> = (0..cfg.num_chains)
        .into_par_iter()
        .map(|c| {
            let mut settings = DiagNutsSettings::default();
            settings.num_tune = cfg.num_warmup;
            settings.num_draws = cfg.num_samples;
            settings.seed = cfg.seed;
            settings.adapt_options.step_size_settings.target_accept = cfg.target_accept;
            let math = CpuMath::new(Density { f: logp, dim });
            let mut rng = StdRng::seed_from_u64(
                cfg.seed
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(c as u64),
            );
            let start: Vec<f64> = init
                .iter()
                .map(|x| x + 1e-3 * rng.sample::<f64, _>(StandardNormal))
                .collect();
            let mut chain = settings.new_chain(c as u64, math, &mut rng)?;
            chain.set_position(&start)?;
            let mut draws = Vec::with_capacity(cfg.num_samples as usize);
            let mut div = 0;
            let mut grads = 0;
            for _ in 0..cfg.num_warmup + cfg.num_samples {
                let (d, info) = chain.draw()?;
                grads += info.num_gradients;
                if !info.tuning {
                    div += info.diverging as usize;
                    draws.push(d.to_vec());
                }
            }
            Ok((draws, div, grads))
        })
        .collect();
    let mut samples = vec![];
    let mut div = 0;
    let mut grads = 0;
    for ch in chains {
        match ch {
            Ok((d, v, g)) => {
                samples.push(d);
                div += v;
                grads += g;
            }
            Err(e) => return (vec![], json!({"qualified": false, "error": e.to_string()})),
        }
    }
    let rhat = if samples[0].len() >= 4 {
        split_gelman_rubin(&samples).into_iter().fold(0.0, f64::max)
    } else {
        f64::INFINITY
    };
    let ess = effective_sample_size(&samples)
        .into_iter()
        .fold(f64::INFINITY, f64::min);
    let qualified = rhat <= cfg.max_rhat && ess >= cfg.min_ess && div <= cfg.max_divergences;
    let diag = json!({"max_rhat": if rhat.is_finite() { json!(rhat) } else { Value::Null }, "min_ess": ess, "divergences": div,
        "qualified": qualified, "num_chains": cfg.num_chains, "num_samples": cfg.num_samples,
        "gradient_evaluations": grads, "seconds": t0.elapsed().as_secs_f64(), "sampler": "nuts-rs 0.19 diag mass matrix"});
    (samples.concat(), diag)
}

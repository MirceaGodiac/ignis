//! Cost per qualified forecast: proposal vs reference (V3.1 §10.5, §12.6).
//!
//! For ACME tasks the exact reference is the 2-D Gauss–Hermite posterior of the frailty given the
//! prefix (48 nodes/dim); we compare prior-proposal IS and (if given) flow-defensive-mixture IS on ESS,
//! largest weight, posterior-mean error against the reference and latency. The target density is the
//! same in every arm.

use std::time::Instant;

use anyhow::Context;
use candle_core::{DType, Device, Tensor};
use ignis_acme::model as am;
use ignis_core::registry::LoadedRelease;
use ignis_science::gauss_hermite as gh;
use ignis_science::hazards as hz;
use rand::{rngs::StdRng, RngExt, SeedableRng};
use rand_distr::StandardNormal;
use serde_json::{json, Value};

use crate::nn::{self, Params};
use crate::proposal as pp;
use crate::tasks::Arrays;

fn mean_abs_err(w: &[f64], z: &[f64], k: usize, reference: &[f64]) -> f64 {
    (0..k)
        .map(|j| ((0..w.len()).map(|i| w[i] * z[i * k + j]).sum::<f64>() - reference[j]).abs())
        .fold(0.0, f64::max)
}

pub fn acme_benchmark(
    r: &LoadedRelease,
    arr: &Arrays,
    accel: Option<&LoadedRelease>,
    n_tasks: usize,
    particles: usize,
    epsilon: f64,
    seed: u64,
) -> anyhow::Result<Value> {
    let b = ignis_acme::expert::bind(r)?;
    let k = b.spec.k();
    let (zr, lwr) = gh::standard_grid(48, k);
    let acc = match accel {
        Some(a) => {
            let map = a
                .params
                .iter()
                .map(|(n, t)| (n.clone(), (t.shape.clone(), t.data.clone())))
                .collect();
            Some(Params::from_f64(&map, k, DType::F64, &Device::Cpu)?)
        }
        None => None,
    };
    let f = arr["c"].0[1];
    let mut rng = StdRng::seed_from_u64(seed);
    let mut rows = vec![];
    for i in 0..n_tasks.min(arr["latent"].0[0]) {
        let g = arr["draw"].1[i] as usize;
        let p = &b.draws[g];
        let c = &arr["c"].1[i * f..(i + 1) * f];
        let tau = arr["tau"].1[i];
        // base cumulative hazard over (0, τ]
        let cells = hz::cell_grid(0.0, tau, &b.spec.edges(), b.spec.cells_per_bin).nonzero();
        let x: Vec<f64> = cells
            .t_mid
            .iter()
            .map(|t| (t / b.spec.t_ref_s).ln_1p())
            .collect();
        let mut lam = vec![0.0; cells.len() * k];
        am::base_hazards(p, &b.spec, &b.lay, c, &x, &cells.bin, &mut lam, None);
        let mut hb = [0.0; 2];
        for (ci, dt) in cells.dt.iter().enumerate() {
            for j in 0..k {
                hb[j] += lam[ci * k + j] * dt;
            }
        }
        let l = am::frailty_chol(p, &b.spec, &b.lay);
        let loglik = |z: &[f64]| -> f64 {
            let u = if k == 1 {
                vec![l[0] * z[0]]
            } else {
                vec![l[0] * z[0], l[2] * z[0] + l[3] * z[1]]
            };
            -(0..k).map(|j| u[j].exp() * hb[j]).sum::<f64>()
        };
        let lref: Vec<f64> = (0..lwr.len())
            .map(|n| lwr[n] + loglik(&zr[n * k..(n + 1) * k]))
            .collect();
        let (wref, _) = gh::joint_normalize(&lref);
        let ref_mean: Vec<f64> = (0..k)
            .map(|j| (0..wref.len()).map(|n| wref[n] * zr[n * k + j]).sum())
            .collect();
        let zero = vec![0.0; k];
        let one = vec![1.0; k];
        let t = Instant::now();
        let z: Vec<f64> = (0..particles * k)
            .map(|_| rng.sample::<f64, _>(StandardNormal))
            .collect();
        let lp0: Vec<f64> = z
            .chunks(k)
            .map(|v| pp::gaussian_log_density(v, &zero, &one))
            .collect();
        let target: Vec<f64> = z.chunks(k).zip(&lp0).map(|(v, a)| a + loglik(v)).collect();
        let rp = pp::reweight(&target, &lp0, 0.0, 0.2);
        let lat_prior = t.elapsed().as_secs_f64();
        let mut row = json!({"ref_mean": ref_mean, "prior_ess_fraction": rp.diagnostics["ess_fraction"],
            "prior_abs_error": mean_abs_err(&rp.weights, &z, k, &ref_mean), "prior_latency_s": lat_prior});
        if let Some(ap) = &acc {
            let t = Instant::now();
            let row_t = |name: &str| -> anyhow::Result<Tensor> {
                let (s, v) = &arr[name];
                let len: usize = s[1..].iter().product();
                let mut sh = vec![1];
                sh.extend(&s[1..]);
                Ok(Tensor::from_vec(
                    v[i * len..(i + 1) * len].to_vec(),
                    sh,
                    &Device::Cpu,
                )?)
            };
            let ctx = nn::encode(ap, &row_t("tokens")?, &row_t("mask")?, &row_t("meta")?)?;
            let gs = row_t("gsum")?;
            let (n_psi, n_def) = pp::defensive_counts(particles, epsilon);
            let eps_t = Tensor::from_vec(
                (0..n_psi * k)
                    .map(|_| rng.sample::<f64, _>(StandardNormal))
                    .collect::<Vec<f64>>(),
                (n_psi, k),
                &Device::Cpu,
            )?;
            let (zf, _) = nn::sample_from(
                ap,
                &eps_t,
                &ctx.broadcast_as((n_psi, nn::WIDTH))?.contiguous()?,
                &gs.broadcast_as((n_psi, nn::GLOBAL_DIM))?.contiguous()?,
            )?;
            let mut zall: Vec<f64> = zf.flatten_all()?.to_vec1()?;
            zall.extend((0..n_def * k).map(|_| rng.sample::<f64, _>(StandardNormal)));
            let zt = Tensor::from_vec(zall.clone(), (particles, k), &Device::Cpu)?;
            let lq: Vec<f64> = nn::log_prob(
                ap,
                &zt,
                &ctx.broadcast_as((particles, nn::WIDTH))?.contiguous()?,
                &gs.broadcast_as((particles, nn::GLOBAL_DIM))?.contiguous()?,
            )?
            .to_vec1()?;
            let lp: Vec<f64> = zall
                .chunks(k)
                .map(|v| pp::gaussian_log_density(v, &zero, &one))
                .collect();
            let ld = pp::defensive_log_density(&lq, &lp, epsilon, particles);
            let target: Vec<f64> = zall
                .chunks(k)
                .zip(&lp)
                .map(|(v, a)| a + loglik(v))
                .collect();
            let rf = pp::reweight(&target, &ld, 0.0, 0.2);
            row["flow_ess_fraction"] = rf.diagnostics["ess_fraction"].clone();
            row["flow_abs_error"] = json!(mean_abs_err(&rf.weights, &zall, k, &ref_mean));
            row["flow_latency_s"] = json!(t.elapsed().as_secs_f64());
            row["flow_refine"] = json!(rf.refine);
        }
        rows.push(row);
    }
    let first = rows.first().context("no tasks")?;
    let mut summary = serde_json::Map::new();
    for key in first
        .as_object()
        .unwrap()
        .keys()
        .filter(|k| *k != "ref_mean" && *k != "flow_refine")
    {
        let v: Vec<f64> = rows.iter().filter_map(|r| r[key].as_f64()).collect();
        summary.insert(
            key.clone(),
            json!(v.iter().sum::<f64>() / v.len().max(1) as f64),
        );
    }
    Ok(
        json!({"release_id": r.release.id, "accel_id": accel.map(|a| a.release.id.clone()), "n_tasks": rows.len(), "particles": particles,
        "epsilon": epsilon, "epsilon_realised": pp::realised_epsilon(particles, epsilon), "reference": "Gauss-Hermite 48^K", "summary": summary}),
    )
}

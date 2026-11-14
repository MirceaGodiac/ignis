//! NUTS geometry diagnosis on the synthetic fixture: MAP quality, curvature spectrum of the
//! posterior in the Laplace-whitened coordinates, and a short NUTS pilot.
//! `cargo run --release -p ignis-flex --example geometry [model] [n_runs] [warmup] [samples] [precondition]`

use ignis_core::views::FlexTrajectory;
use ignis_flex::data::{admin_censor, pack_views};
use ignis_flex::dynamics::{Model, Numerics};
use ignis_flex::inference::{init_params, repair_support, Centered, Joint};
use ignis_flex::structure::{
    default_condition_keys, fit_feature_spec, global_specs, merged_priors, Structure,
};
use ignis_flex::synthetic::make_dataset;
use ignis_science::nuts::LogDensity;
use ignis_science::nuts::{neg_hessian_fd, run_nuts, NutsConfig, Precond};
use ignis_science::optim::{lbfgs, Adam, AdamConfig, LbfgsConfig};
use ignis_science::transforms::Layout;

fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let name = a.get(1).map(String::as_str).unwrap_or("flex.v1");
    let n_runs: usize = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(24);
    let warmup: u64 = a.get(3).map(|s| s.parse().unwrap()).unwrap_or(200);
    let samples: u64 = a.get(4).map(|s| s.parse().unwrap()).unwrap_or(200);
    let pre_kind = a.get(5).cloned().unwrap_or("laplace".into());
    let centered = a.get(6).map(|s| s == "centered").unwrap_or(true);
    let ds = make_dataset(n_runs, 0, 3, 0.1)?;
    let views: Vec<&FlexTrajectory> = ds.views.iter().collect();
    let keys = default_condition_keys();
    let mut pk = pack_views(&views, &keys, 30.0);
    admin_censor(&mut pk, 30.0);
    // optional: drop the last observations near the modelled floor (support-cliff diagnosis)
    if std::env::var("GEOMETRY_TRIM").is_ok() {
        for r in pk.runs.iter_mut() {
            for k in 0..r.t.len() {
                if r.y[k] < 3.0 * 2.5e-4 {
                    r.mask[k] = false;
                }
            }
        }
    }
    let fs = fit_feature_spec(
        &pk.runs.iter().map(|r| r.ck).collect::<Vec<_>>(),
        &pk.logd0_proxy(),
        &keys,
        1e-9,
    );
    let pr = merged_priors(None);
    let st = Structure::get(name).unwrap();
    let m = Model::new(
        st.clone(),
        fs.clone(),
        Numerics::default(),
        Layout::new(global_specs(&st, &fs, &pr)),
    );
    let j = Joint {
        m: &m,
        runs: &pk.runs,
        pr: &pr,
    };
    let mut x = init_params(&m, &pk.runs, &pr, 0);
    repair_support(&j, &mut x, 20);
    let pool = rayon::ThreadPoolBuilder::new().build()?;
    pool.install(|| -> anyhow::Result<()> {
        let c = Centered { j: &j };
        let mut g = vec![0.0; x.len()];
        let t0 = std::time::Instant::now();
        let mut opt = Adam::new(AdamConfig { learning_rate: 0.01, ..Default::default() }, x.len());
        for _ in 0..2000 {
            j.value_grad(&x, &mut g);
            g.iter_mut().for_each(|v| *v = -*v);
            opt.step(&mut x, &mut g);
        }
        let v = j.value_grad(&x, &mut g);
        let gn = g.iter().map(|v| v * v).sum::<f64>().sqrt();
        println!("adam 2000 steps: logp {v:.3} |g| {gn:.3e} ({:.1}s)", t0.elapsed().as_secs_f64());
        let t1 = std::time::Instant::now();
        let r = lbfgs(|y| {
            let mut gg = vec![0.0; y.len()];
            let v = j.value_grad(y, &mut gg);
            (-v, gg.into_iter().map(|a| -a).collect())
        }, &x, &LbfgsConfig { max_iter: 3000, gtol: 1e-3, ..Default::default() });
        x = r.x.clone();
        let v = j.value_grad(&x, &mut g);
        let gn = g.iter().map(|v| v * v).sum::<f64>().sqrt();
        println!("lbfgs: logp {v:.3} |g| {gn:.3e} iters {} evals {} {} ({:.1}s)", r.iterations, r.evaluations, r.message, t1.elapsed().as_secs_f64());
        let dens: &dyn LogDensity = if centered { &c } else { &j };
        if centered {
            x = c.from_eps(&x);
        }
        let grad = |y: &[f64]| {
            let mut gg = vec![0.0; y.len()];
            dens.logp_grad(y, &mut gg);
            gg
        };

        println!("centered: {centered}");
        let h = neg_hessian_fd(&x, &grad, &j.hessian_probes(), &|i, p| j.attribute(i, p), &pool);
        let pre = if pre_kind == "laplace" { Precond::from_neg_hessian(&x, &h) } else { Precond::identity(&x) };
        // curvature along whitened axes: should be ~1
        let z0 = pre.to_z(&x);
        let gz0 = pre.grad_z(&grad(&x));
        let mut curv = vec![];
        for i in (0..z0.len()).step_by((z0.len() / 40).max(1)) {
            let mut z = z0.clone();
            z[i] += 0.3;
            let gz = pre.grad_z(&grad(&pre.to_x(&z)));
            curv.push(-(gz[i] - gz0[i]) / 0.3);
        }
        curv.sort_by(f64::total_cmp);
        println!("whitened curvature (finite step 0.3) min {:.3} median {:.3} max {:.3}", curv[0], curv[curv.len() / 2], curv[curv.len() - 1]);
        let cfg = NutsConfig { chains: 4, warmup, samples, ..Default::default() };
        let r = run_nuts(dens, &x, &pre, &cfg, 0, &pool)?;
        let steps: Vec<u64> = r.stats.iter().flat_map(|s| s.num_steps.clone()).collect();
        let div: usize = r.stats.iter().map(|s| s.diverging.iter().filter(|b| **b).count()).sum();
        println!("nuts {} chains {}+{}: wall {:.1}s, grads {}, mean leapfrog {:.1}, step sizes {:?}, divergences {div}",
                 cfg.chains, warmup, samples, r.wall_s, r.gradient_evaluations, steps.iter().sum::<u64>() as f64 / steps.len() as f64,
                 r.stats.iter().map(|s| (s.step_size * 1e3).round() / 1e3).collect::<Vec<_>>());
        let blocks: Vec<(String, Vec<Vec<Vec<f64>>>)> = (0..x.len()).step_by(1).map(|k| (format!("x{k}"), vec![r.draws.iter().map(|c| c.iter().map(|d| d[k]).collect()).collect()])).collect();
        let s = ignis_science::diagnostics::summarize(&blocks, &r.stats, 1.01, 100.0);
        println!("rhat_max {} bulk_ess_min {} tail_ess_min {} pass {}", s["rhat_max"], s["bulk_ess_min"], s["tail_ess_min"], s["pass"]);
        Ok(())
    })
}

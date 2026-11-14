//! Speed measurements: joint value+gradient at the nominal planning size (synthetic, 100 runs),
//! single-thread vs the profile pool, and the fused integrand node vs an op-by-op tape of the same
//! Gauss–Legendre sum.  `cargo run --release -p ignis-flex --example bench [n_runs]`

use std::time::Instant;

use ignis_core::views::FlexTrajectory;
use ignis_flex::data::{admin_censor, pack_views};
use ignis_flex::dynamics::{run_state, Model, Numerics};
use ignis_flex::inference::{init_params, repair_support, Joint};
use ignis_flex::structure::{
    default_condition_keys, fit_feature_spec, global_specs, merged_priors, Structure,
};
use ignis_flex::synthetic::make_dataset;
use ignis_science::ad::{self, Real, Var};
use ignis_science::integrate::{Integrand, Rule};
use ignis_science::transforms::Layout;

fn time<F: FnMut()>(n: usize, mut f: F) -> f64 {
    f();
    let t = Instant::now();
    for _ in 0..n {
        f();
    }
    t.elapsed().as_secs_f64() / n as f64
}

/// The FLEX integrand sum recorded op by op (reference for the fused node).
fn gl_ops(rs: &ignis_flex::dynamics::Integ<Var>, a: Var, b: Var, rule: &Rule) -> [Var; 2] {
    let (mid, half) = ((a + b) * 0.5, (b - a) * 0.5);
    let (mut s0, mut s1) = (Var::cst(0.0), Var::cst(0.0));
    for i in 0..rule.len() {
        let u = mid + half * rule.x[i];
        let w = Var::cst(1.0) - u;
        let mut sc = rs.a2;
        let mut s00 = rs.a2;
        for j in 0..rs.hidden {
            sc += rs.w2[j] * (rs.pc[j] + rs.w1w[j] * w).tanh();
            s00 += rs.w2[j] * (rs.p0[j] + rs.w1w[j] * w).tanh();
        }
        let r = w * 0.5 * (sc.tanh() - s00.tanh());
        let s = rs.theta[0] * w + rs.theta[1] * w * w + rs.theta[2] * w * w * w + rs.b1 * w + r;
        let g = (rs.log_a - s).exp();
        let lam = (rs.base_h + rs.ha[0] * w + rs.ha[1] * w * w).exp();
        let wt = half * rule.w[i];
        s0 += wt * g;
        s1 += wt * lam * g;
    }
    [s0, s1]
}

fn main() -> anyhow::Result<()> {
    let n_runs: usize = std::env::args()
        .nth(1)
        .map(|s| s.parse().unwrap())
        .unwrap_or(100);
    let ds = make_dataset(n_runs, 5, 3, 0.05)?;
    let views: Vec<&FlexTrajectory> = ds.views.iter().collect();
    let keys = default_condition_keys();
    let mut pk = pack_views(&views, &keys, 30.0);
    admin_censor(&mut pk, 30.0);
    let n_obs: usize = pk
        .runs
        .iter()
        .map(|r| r.mask.iter().filter(|b| **b).count())
        .sum();
    let fs = fit_feature_spec(
        &pk.runs.iter().map(|r| r.ck).collect::<Vec<_>>(),
        &pk.logd0_proxy(),
        &keys,
        1e-9,
    );
    let pr = merged_priors(None);
    for name in ["flex.v3", "flex.v1", "flex.d2_population"] {
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
        let mut g = vec![0.0; x.len()];
        let one = rayon::ThreadPoolBuilder::new().num_threads(1).build()?;
        let all = rayon::ThreadPoolBuilder::new().build()?;
        let t1 = one.install(|| {
            time(3, || {
                j.value_grad(&x, &mut g);
            })
        });
        let tn = all.install(|| {
            time(20, || {
                j.value_grad(&x, &mut g);
            })
        });
        let tv = all.install(|| {
            time(20, || {
                j.logp(&x);
            })
        });
        println!("{{\"model\": \"{name}\", \"runs\": {n_runs}, \"obs\": {n_obs}, \"dim\": {}, \"grad_s_1thread\": {t1:.4}, \"grad_s_{}threads\": {tn:.4}, \"grad_per_s\": {:.1}, \"value_only_s\": {tv:.4}}}",
                 x.len(), all.current_num_threads(), 1.0 / tn);
    }
    // fused integrand node vs op-by-op tape for one 8-node rule
    let st = Structure::get("flex.v3").unwrap();
    let m = Model::new(
        st.clone(),
        fs.clone(),
        Numerics::default(),
        Layout::new(global_specs(&st, &fs, &pr)),
    );
    let x = init_params(&m, &pk.runs, &pr, 0);
    let ng = m.n_g();
    let rule = Rule::cached(8);
    let reps = 2000;
    let mut gbuf = vec![0.0; ng];
    let fused = time(reps, || {
        ad::reset();
        let p = ad::leaves(&x[..ng]);
        let e = [Var::cst(0.1), Var::cst(-0.2)];
        let rs = run_state(&m, &p, &pk.runs[0].ck, &e, Var::cst(-5.8));
        let v = rs.integ.gl(Var::cst(0.3), Var::cst(0.35), rule);
        ad::accumulate_grad(v[0] + v[1], &p, &mut gbuf);
    });
    let ops = time(reps, || {
        ad::reset();
        let p = ad::leaves(&x[..ng]);
        let e = [Var::cst(0.1), Var::cst(-0.2)];
        let rs = run_state(&m, &p, &pk.runs[0].ck, &e, Var::cst(-5.8));
        let v = gl_ops(&rs.integ, Var::cst(0.3), Var::cst(0.35), rule);
        ad::accumulate_grad(v[0] + v[1], &p, &mut gbuf);
    });
    // both include building one run state (table of 16 panels); report the difference per rule too
    let base = time(reps, || {
        ad::reset();
        let p = ad::leaves(&x[..ng]);
        let e = [Var::cst(0.1), Var::cst(-0.2)];
        let rs = run_state(&m, &p, &pk.runs[0].ck, &e, Var::cst(-5.8));
        ad::accumulate_grad(rs.t_rmin, &p, &mut gbuf);
    });
    let (f1, o1) = ((fused - base).max(1e-12), (ops - base).max(1e-12));
    println!("{{\"run_state_build_s\": {base:.3e}, \"one_rule_fused_s\": {f1:.3e}, \"one_rule_op_tape_s\": {o1:.3e}, \"speedup\": {:.1}}}", o1 / f1);
    let _ = <f64 as Real>::cst(0.0);
    Ok(())
}

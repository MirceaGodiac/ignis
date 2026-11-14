//! Parity with the retired Python/JAX implementation (fixture `data/python_parity.json`, produced by
//! the JAX code before it was deleted): run log likelihoods and gradients for every endpoint code,
//! inverse trajectories r(t), forecast particles (D(t), survival, event CDFs, Kalman predictive) and
//! the joint training density with its gradient on packed synthetic runs.

use std::collections::BTreeMap;

use ignis_core::views::FlexTrajectory;
use ignis_flex::data::{admin_censor, pack_views};
use ignis_flex::dynamics::{forecast_particle, run_loglik, run_state, Model, Numerics, RunData};
use ignis_flex::inference::Joint;
use ignis_flex::structure::{
    fit_feature_spec, global_specs, merged_priors, FeatureSpec, Structure,
};
use ignis_science::ad::{self, Var};
use ignis_science::transforms::Layout;
use serde_json::Value;

fn fixture() -> Value {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/python_parity.json");
    serde_json::from_str(&std::fs::read_to_string(p).expect("parity fixture")).unwrap()
}

fn vecf(v: &Value) -> Vec<f64> {
    match v {
        Value::Array(a) => a.iter().flat_map(vecf).collect(),
        Value::Number(n) => vec![n.as_f64().unwrap()],
        _ => vec![],
    }
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1e-300)
}

/// Max relative error, with an absolute floor scaled to the vector's magnitude.
fn max_rel(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    let scale = b.iter().fold(0.0f64, |m, x| m.max(x.abs()));
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs() / y.abs().max(1e-6 * scale).max(1e-300))
        .fold(0.0, f64::max)
}

fn model(name: &str, fs: FeatureSpec) -> Model {
    let st = Structure::get(name).unwrap();
    let pr = merged_priors(None);
    Model::new(
        st.clone(),
        fs.clone(),
        Numerics::default(),
        Layout::new(global_specs(&st, &fs, &pr)),
    )
}

fn flat(m: &Model, params: &Value) -> Vec<f64> {
    m.layout
        .specs
        .iter()
        .flat_map(|s| vecf(&params[&s.name]))
        .collect()
}

#[test]
fn run_likelihood_gradient_trajectory_and_event_cdfs_match_jax() {
    let fx = fixture();
    let keys = ignis_flex::structure::default_condition_keys();
    let mut worst = BTreeMap::new();
    for case in fx["cases"].as_array().unwrap() {
        let name = case["model"].as_str().unwrap();
        let m = model(name, FeatureSpec::full(keys.clone()));
        let p = flat(&m, &case["params"]);
        let eps = vecf(&case["eps"]);
        let ld0 = case["log_d0"].as_f64().unwrap();
        let ck: [f64; 4] = vecf(&case["ck"]).try_into().unwrap();
        let rs = run_state(&m, &p, &ck, &eps, ld0);
        assert!(
            rel(rs.t_rmin, case["t_rmin"].as_f64().unwrap()) < 1e-12,
            "{name} t_rmin"
        );
        let times = vecf(&case["times"]);
        let r: Vec<f64> = times.iter().map(|&t| rs.r_of_t(&m.num, t)).collect();
        let e_r = max_rel(&r, &vecf(&case["r_of_t"]));
        assert!(e_r < 1e-12, "{name} r(t) {e_r}");
        let mask: Vec<bool> = case["mask"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b.as_bool().unwrap())
            .collect();
        let base = RunData {
            t: vecf(&case["t"]),
            y: vecf(&case["y"]),
            mask,
            ck,
            code: 0,
            l: 0.0,
            r: 0.0,
            cause_mask: [0.0; 2],
            tau: 2.0,
            use_tau: false,
        };
        let mut e_v: f64 = 0.0;
        let mut e_g: f64 = 0.0;
        for rc in case["runs"].as_array().unwrap() {
            let run = RunData {
                code: rc["code"].as_u64().unwrap() as u8,
                l: rc["L"].as_f64().unwrap(),
                r: rc["R"].as_f64().unwrap(),
                cause_mask: vecf(&rc["cause_mask"]).try_into().unwrap(),
                use_tau: rc["use_tau"].as_bool().unwrap(),
                ..base.clone()
            };
            let ng = m.n_g();
            let mut x = p.clone();
            x.extend(&eps);
            x.push(ld0);
            let (v, g) = ad::value_and_grad(&x, |z: &[Var]| {
                run_loglik(
                    &m,
                    &z[..ng],
                    &run,
                    &z[ng..ng + eps.len()],
                    z[ng + eps.len()],
                )
            });
            let want = rc["value"].as_f64().unwrap();
            if !want.is_finite() {
                assert!(!v.is_finite());
                continue;
            }
            e_v = e_v.max(rel(v, want));
            let mut gw: Vec<f64> = m
                .layout
                .specs
                .iter()
                .flat_map(|s| vecf(&rc["grad_globals"][&s.name]))
                .collect();
            gw.extend(vecf(&rc["grad_eps"]));
            gw.push(rc["grad_log_d0"].as_f64().unwrap());
            e_g = e_g.max(max_rel(&g, &gw));
        }
        assert!(e_v < 1e-10, "{name} loglik rel err {e_v}");
        assert!(e_g < 1e-8, "{name} gradient rel err {e_g}");
        let fp = forecast_particle(
            &m,
            &p,
            &RunData {
                use_tau: true,
                ..base.clone()
            },
            &eps,
            ld0,
            &times,
        );
        let fw = &case["forecast"];
        let mut e_f: f64 = 0.0;
        e_f = e_f.max(max_rel(&fp.d, &vecf(&fw["d"])));
        e_f = e_f.max(max_rel(&fp.s_rel, &vecf(&fw["S_rel"])));
        let fr: Vec<f64> = fp.f_rel.iter().flatten().copied().collect();
        if m.st.hazards {
            e_f = e_f.max(max_rel(&fr, &vecf(&fw["F_rel"])));
            e_f = e_f.max(rel(fp.s_b_rel, fw["S_B_rel"].as_f64().unwrap()));
        }
        e_f = e_f.max(max_rel(&fp.e_loc, &vecf(&fw["e_loc"])));
        e_f = e_f.max(max_rel(&fp.e_scale2, &vecf(&fw["e_scale2"])));
        assert!(e_f < 1e-10, "{name} forecast particle rel err {e_f}");
        worst.insert(name.to_string(), (e_v, e_g, e_r, e_f));
    }
    eprintln!("parity (loglik, grad, r(t), forecast) max rel err: {worst:?}");
}

#[test]
fn joint_training_density_and_gradient_match_jax() {
    let fx = fixture();
    let keys: BTreeMap<String, String> = serde_json::from_value(fx["keys"].clone()).unwrap();
    let views: Vec<FlexTrajectory> = fx["views"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let mut v = v.clone();
            v["event_before_support_exit"] = Value::Null;
            serde_json::from_value(v).unwrap()
        })
        .collect();
    let refs: Vec<&FlexTrajectory> = views.iter().collect();
    for j in fx["joint"].as_array().unwrap() {
        let name = j["model"].as_str().unwrap();
        let mut pk = pack_views(&refs, &keys, 30.0);
        admin_censor(&mut pk, 30.0);
        let fs = fit_feature_spec(
            &pk.runs.iter().map(|r| r.ck).collect::<Vec<_>>(),
            &pk.logd0_proxy(),
            &keys,
            1e-9,
        );
        let ca: Vec<bool> = serde_json::from_value(j["c_active"].clone()).unwrap();
        assert_eq!(fs.c_active, ca);
        let m = model(name, fs);
        let pr = merged_priors(None);
        let joint = Joint {
            m: &m,
            runs: &pk.runs,
            pr: &pr,
        };
        let mut x = flat(&m, &j["params"]);
        if m.n_b() > 0 {
            x.extend(vecf(&j["params"]["local.eps_b"]));
        }
        x.extend(vecf(&j["params"]["local.log_d0"]));
        let mut g = vec![0.0; x.len()];
        let t0 = std::time::Instant::now();
        let n = 20;
        let mut v = 0.0;
        for _ in 0..n {
            v = joint.value_grad(&x, &mut g);
        }
        let dt = t0.elapsed().as_secs_f64() / n as f64;
        let want = j["value"].as_f64().unwrap();
        let mut gw: Vec<f64> = m
            .layout
            .specs
            .iter()
            .flat_map(|s| vecf(&j["grad"][&s.name]))
            .collect();
        if m.n_b() > 0 {
            gw.extend(vecf(&j["grad"]["local.eps_b"]));
        }
        gw.extend(vecf(&j["grad"]["local.log_d0"]));
        let (ev, eg) = (rel(v, want), max_rel(&g, &gw));
        eprintln!("{name}: joint rel err {ev:.2e}, grad max rel err {eg:.2e}; value+grad {:.3} ms (JAX jit {:.3} ms), {} obs",
                  dt * 1e3, j["jax_value_and_grad_s"].as_f64().unwrap() * 1e3, j["n_obs"]);
        assert!(ev < 1e-10 && eg < 1e-8, "{name}: {ev} {eg}");
    }
}

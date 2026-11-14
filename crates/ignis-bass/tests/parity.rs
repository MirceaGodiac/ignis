//! Parity with the frozen Python/JAX reference (tools/parity/bass_parity.py) at fixed parameters:
//! operator outputs, the right-hand side and its reverse mode, PDE frames (legacy LLF1 / linear
//! ghosts), the run-level scale-error sequence likelihood and the member objective + gradient.

use ignis_bass::data::{self, FeatureSpec};
use ignis_bass::dynamics::{self as dy, Boundary, Ctx, Numerics, RhsTape, Scheme};
use ignis_bass::model::{self as m, GlobalLayout};
use ignis_bass::operator::{self as op, Grid};
use ignis_core::views::BassSequence;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/bass_parity.json"
        ))
        .unwrap(),
    )
    .unwrap()
}
fn vecf(v: &Value) -> Vec<f64> {
    v.as_array()
        .unwrap()
        .iter()
        .flat_map(|x| {
            if x.is_array() {
                vecf(x)
            } else {
                vec![x.as_f64().unwrap_or(f64::NAN)]
            }
        })
        .collect()
}
pub fn config() -> Value {
    let t: toml::Value = toml::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../configs/models/bass/operator.toml"
        ))
        .unwrap(),
    )
    .unwrap();
    serde_json::to_value(t).unwrap()
}
fn sequences(fx: &Value) -> Vec<BassSequence> {
    fx["sequences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let valid: Vec<bool> = s["valid"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|r| r.as_array().unwrap().iter().map(|b| b.as_bool().unwrap()))
                .collect();
            BassSequence {
                run_id: s["run_id"].as_str().unwrap().into(),
                group_id: s["group_id"].as_str().unwrap().into(),
                t_s: vecf(&s["t_s"]),
                s_m: vecf(&s["s_m"]),
                f_m: vecf(&s["f_m"]),
                valid,
                status: s["status"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect(),
                conditions: serde_json::from_value(s["conditions"].clone()).unwrap(),
                geometry: serde_json::from_value(s["geometry"].clone()).unwrap(),
                control_changes_s: vec![s["conditions"]["last_control_change_s"].as_f64().unwrap()],
                scale_uncertainty_rel: vecf(&s["scale_uncertainty_rel"]),
                time_origin: "t0".into(),
            }
        })
        .collect()
}
fn close(name: &str, a: &[f64], b: &[f64], tol: f64) -> f64 {
    assert_eq!(a.len(), b.len(), "{name} length");
    let scale = b.iter().fold(0.0f64, |x, y| x.max(y.abs())).max(1e-300);
    let e = a
        .iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f64::max)
        / scale;
    assert!(e < tol, "{name}: rel err {e:e}");
    eprintln!("{name:>18}: max rel err {e:.2e}");
    e
}

#[test]
fn bass_parity() {
    let fx = fixture();
    let cfg = config();
    let feats: Vec<FeatureSpec> = serde_json::from_value(cfg["features"].clone()).unwrap();
    let seqs = sequences(&fx);
    let refs: Vec<&BassSequence> = seqs.iter().collect();
    let (runs, excl) = data::prepare(&refs, &feats, &cfg, op::N_NODES);
    assert!(excl.is_empty(), "{excl:?}");
    let norm = data::fit_normalization(&feats, &runs);
    for (k, v) in fx["normalization"].as_object().unwrap() {
        for (kk, vv) in v.as_object().unwrap() {
            let r = vv.as_f64().unwrap();
            assert!(
                (norm[k][kk] - r).abs() <= 1e-12 * r.abs().max(1.0),
                "norm {k}.{kk}"
            );
        }
    }
    let na = data::norm_arrays(&feats, &norm);
    let lay = GlobalLayout::new();
    assert_eq!(lay.len, 5838);
    let mut p = vec![0.0; lay.len];
    for (name, _, off) in &lay.blocks {
        let v = vecf(&fx["params"][name]);
        p[*off..off + v.len()].copy_from_slice(&v);
    }
    let x = vecf(&fx["x"]);
    let grid = Grid::new(op::N_NODES);
    let num = Numerics {
        cfl: 0.4,
        feedback: 0.25,
        dt_max_s: 1.0,
        max_steps: 32,
        scheme: Scheme::Llf1,
        boundary: Boundary::Linear,
    };
    let r0 = &runs[0];
    let cx = Ctx {
        p: &p,
        net: &lay.net,
        phys: &lay.phys,
        grid: &grid,
        num,
        l: r0.l,
        w: r0.w,
    };
    let c = vecf(&fx["c"]);
    {
        let mut cc = vec![0.0; 8];
        use ignis_bass::dynamics::Conditions;
        r0.cond(&na).at(r0.t[1] + 0.3, &mut cc);
        close("conditions", &cc, &c, 1e-12);
    }
    let mut f0 = vec![0.0; 32];
    dy::initial_front(&x[..8], r0.l, &grid, &mut f0);
    close("initial front", &f0, &vecf(&fx["f0"]), 1e-14);
    let mut tape = RhsTape::default();
    op::channels(&f0, r0.l, &grid, &mut tape.op.chan);
    op::forward(&p, &lay.net, &grid, &c, &mut tape.op);
    close(
        "operator out",
        &tape.op.out,
        &vecf(&fx["operator_out"]),
        1e-12,
    );
    let mut g = vec![0.0; 32];
    dy::rhs(&cx, &x[8..12], &f0, &c, &mut tape, &mut g);
    close("rhs", &g, &vecf(&fx["rhs"]), 1e-12);
    close("lf alpha", &tape.tr_alpha(), &vecf(&fx["alpha"]), 1e-12);
    // rhs reverse mode
    let proj = vecf(&fx["rhs_proj"]);
    let mut gp = vec![0.0; lay.len];
    let mut bb = vec![0.0; 4];
    let mut fb = vec![0.0; 32];
    let mut sc = dy::Scratch::default();
    dy::rhs_backward(&cx, &c, &tape, &proj, &mut gp, &mut bb, &mut fb, &mut sc);
    close("rhs grad f", &fb, &vecf(&fx["rhs_grad_f"]), 1e-10);
    close("rhs grad b", &bb, &vecf(&fx["rhs_grad_b"]), 1e-10);
    let mut rg = vec![0.0; lay.len];
    for (name, _, off) in &lay.blocks {
        let v = vecf(&fx["rhs_grad_g"][name]);
        rg[*off..off + v.len()].copy_from_slice(&v);
    }
    close("rhs grad theta", &gp, &rg, 1e-10);
    // PDE frames
    let mut w = dy::SimWork::default();
    let mut tr = dy::Trajectory::default();
    dy::simulate(
        &cx,
        &x[..8],
        &x[8..12],
        &r0.t,
        &r0.cond(&na),
        false,
        &mut w,
        &mut tr,
    );
    close("PDE frames", &tr.frames, &vecf(&fx["frames"]), 1e-12);
    assert_eq!(tr.unreached, fx["unreached"].as_f64().unwrap());
    assert!((tr.max_cfl - fx["max_cfl"].as_f64().unwrap()).abs() < 1e-12);
    let sl = ignis_bass::observation::sequence_loglik(&tr.frames, 32, &r0.pre, lay.obs(&p), None);
    close(
        "sequence loglik",
        &[sl],
        &[fx["sequence_loglik"].as_f64().unwrap()],
        1e-11,
    );
    // member objective + gradient
    let pr = m::Priors::from_json(&cfg["priors"]);
    let pr = if cfg["priors"].is_null() {
        m::Priors::from_json(&m::default_priors())
    } else {
        pr
    };
    let mut gg = vec![0.0; lay.len];
    let mut gx = vec![0.0; x.len()];
    let ev = m::member_objective(
        &p,
        &x,
        &runs,
        &vecf(&fx["weights"]),
        &lay,
        &grid,
        num,
        &na,
        &pr,
        &mut gg,
        &mut gx,
    );
    close(
        "member objective",
        &[ev.value],
        &[fx["objective"].as_f64().unwrap()],
        1e-11,
    );
    let mut rg = vec![0.0; lay.len];
    for (name, _, off) in &lay.blocks {
        let v = vecf(&fx["grad_g"][name]);
        rg[*off..off + v.len()].copy_from_slice(&v);
    }
    close("grad globals", &gg, &rg, 1e-10);
    close("grad locals", &gx, &vecf(&fx["grad_x"]), 1e-10);
    // speed
    let t = std::time::Instant::now();
    let mut n = 0;
    while t.elapsed().as_secs_f64() < 2.0 {
        m::member_objective(
            &p,
            &x,
            &runs,
            &vecf(&fx["weights"]),
            &lay,
            &grid,
            num,
            &na,
            &pr,
            &mut gg,
            &mut gx,
        );
        n += 1;
    }
    eprintln!(
        "rust member value+grad/s (2 runs, llf1): {:.1} vs jax {:.1}",
        n as f64 / t.elapsed().as_secs_f64(),
        fx["jax_member_value_and_grad_per_s"].as_f64().unwrap()
    );
}

//! FLEX synthetic identities (V3.1 §13.1, chapter 26). Not NASA validation.

use ignis_flex::dynamics::{forecast_particle, run_state, Model, Numerics, RunData};
use ignis_flex::inference::flat_from_tensors;
use ignis_flex::structure::{
    default_condition_keys, global_specs, merged_priors, FeatureSpec, Structure, K_REF_M2_S,
};
use ignis_flex::synthetic::truth_params;
use ignis_science::ad::{self, Real, Var};
use ignis_science::transforms::Layout;

const CK: [f64; 4] = [0.4, 0.3, -0.2, -0.1];
const EPS: [f64; 2] = [0.5, -0.7];

fn model(name: &str) -> (Model, Vec<f64>) {
    let st = Structure::get(name).unwrap();
    let fs = FeatureSpec::full(default_condition_keys());
    let m = Model::new(
        st.clone(),
        fs.clone(),
        Numerics::default(),
        Layout::new(global_specs(&st, &fs, &merged_priors(None))),
    );
    let mut t = truth_params(3);
    for s in &m.layout.specs {
        t.entry(s.name.clone()).or_insert_with(|| {
            ignis_core::registry::Tensor::new(s.shape.clone(), vec![0.0; s.size()])
        });
    }
    let p = flat_from_tensors(&m.layout, &t).unwrap();
    (m, p)
}

#[test]
fn constant_d2_limit_recovers_the_si_law() {
    let (m, p) = model("flex.d2_population");
    let ld0 = 3.0e-3f64.ln();
    let rs = run_state(&m, &p, &CK, &[], ld0);
    let c = [CK[0], CK[1], CK[2], CK[3], (3e-3f64 / 0.002).ln()];
    let z = [
        1.0,
        c[0],
        c[1],
        c[2],
        c[3],
        c[4],
        c[0] * c[0],
        c[0] * c[1],
        c[0] * c[3],
    ];
    let k = K_REF_M2_S * z.iter().zip(&p[..9]).map(|(a, b)| a * b).sum::<f64>().exp();
    let q0 = 9.0e-6;
    for t in [0.0, 1.0, 4.0, 9.0] {
        let q = q0 * rs.r_of_t(&m.num, t);
        assert!((q - (q0 - k * t)).abs() < 1e-18, "t={t}");
    }
    assert!(((rs.t_rmin - (q0 - m.num.d_min_m.powi(2)) / k) / rs.t_rmin).abs() < 1e-13);
}

#[test]
fn implicit_trajectory_gradient_matches_finite_differences() {
    let (m, p) = model("flex.v3");
    let ld0 = 3.0e-3f64.ln();
    let i_th = m.layout.range("theta").unwrap().start;
    let i_w = m.layout.range("residual.W1").unwrap().start + (m.c_idx.len() + 1) + 1; // W1[1,1]
    let r_at = |x: &[f64]| run_state(&m, x, &CK, &EPS, ld0).r_of_t(&m.num, 6.0);
    let (_, g) = ad::value_and_grad(&p, |z: &[Var]| {
        let e = [Var::cst(EPS[0]), Var::cst(EPS[1])];
        run_state(&m, z, &CK, &e, Var::cst(ld0)).r_of_t(&m.num, Var::cst(6.0))
    });
    for i in [i_th, i_w] {
        let h = 1e-6;
        let (mut a, mut b) = (p.clone(), p.clone());
        a[i] += h;
        b[i] -= h;
        let fd = (r_at(&a) - r_at(&b)) / (2.0 * h);
        assert!(
            g[i] != 0.0 && (g[i] - fd).abs() < 1e-8 * fd.abs().max(1.0),
            "{i}: {} vs {fd}",
            g[i]
        );
    }
}

#[test]
fn support_mass_partition_and_known_cause_interval() {
    let (m, p) = model("flex.v3");
    let ld0 = 3.0e-3f64.ln();
    let rs = run_state(&m, &p, &CK, &EPS, ld0);
    let tau = 2.0;
    let mut times: Vec<f64> = (0..23)
        .map(|i| tau + (rs.b + 5.0 - tau) * i as f64 / 22.0)
        .collect();
    times.push(rs.b);
    times.sort_by(f64::total_cmp);
    let t: Vec<f64> = (0..32).map(|i| tau * i as f64 / 31.0).collect();
    let y: Vec<f64> = t
        .iter()
        .map(|&x| 3e-3 * rs.r_of_t(&m.num, x).sqrt())
        .collect();
    let run = RunData {
        t,
        y,
        mask: vec![true; 32],
        ck: CK,
        tau,
        use_tau: true,
        ..Default::default()
    };
    let fp = forecast_particle(&m, &p, &run, &EPS, ld0, &times);
    let mut u_last = 0.0;
    for j in 0..times.len() {
        let a = if fp.before_b[j] { fp.s_rel[j] } else { 0.0 };
        let u = if fp.before_b[j] { 0.0 } else { fp.s_b_rel };
        u_last = u;
        assert!((a + u + fp.f_rel[0][j] + fp.f_rel[1][j] - 1.0).abs() < 1e-10);
        if j > 0 {
            assert!(fp.f_rel[0][j] >= fp.f_rel[0][j - 1] - 1e-15);
        }
    }
    assert!(u_last > 0.0);
    // one known cause over (L, R] is not the all-cause probability S(L) - S(R)
    let (ul, ur) = (rs.r_of_t(&m.num, 3.0), rs.r_of_t(&m.num, 6.0));
    let (sl, sr) = (
        rs.log_survival_at_u(ul).0.exp(),
        rs.log_survival_at_u(ur).0.exp(),
    );
    let pk = rs.cause_integral(&m.num, ur, ul, [1.0, 1.0]);
    assert!(((pk[0] + pk[1]) - (sl - sr)).abs() < 1e-10 * (sl - sr));
    assert!(pk[1] > 0.0 && pk[1] < sl - sr && pk[0] > 0.0 && pk[0] < sl - sr);
}

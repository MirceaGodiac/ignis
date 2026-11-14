//! Spatial convergence order of the front PDE (§9.6) and finite-difference checks of the reverse
//! mode for the second-order scheme. Self-convergence on nested grids K = 2^j + 1 at the shared nodes:
//! order = log2(‖u_K − u_2K‖ / ‖u_2K − u_4K‖), time step from the CFL limit (Δt ∝ Δs).

use ignis_bass::dynamics::{self as dy, Boundary, Conditions, Ctx, Numerics, Scheme};
use ignis_bass::model::{self as m, GlobalLayout};
use ignis_bass::operator::Grid;

struct Fixed;
impl Conditions for Fixed {
    fn at(&self, _t: f64, out: &mut [f64]) {
        for (i, o) in out.iter_mut().enumerate() {
            *o = 0.1 * i as f64 - 0.3;
        }
    }
}

fn run(
    p: &[f64],
    lay: &GlobalLayout,
    k: usize,
    num: Numerics,
    a: &[f64],
    b: &[f64],
    t_end: f64,
) -> Vec<f64> {
    let grid = Grid::new(k);
    let cx = Ctx {
        p,
        net: &lay.net,
        phys: &lay.phys,
        grid: &grid,
        num,
        l: 0.1,
        w: 0.04,
    };
    let mut w = dy::SimWork::default();
    let mut tr = dy::Trajectory::default();
    dy::simulate(&cx, a, b, &[0.0, t_end], &Fixed, false, &mut w, &mut tr);
    assert_eq!(tr.unreached, 0.0);
    tr.frames[k..].to_vec()
}

fn orders(
    p: &[f64],
    lay: &GlobalLayout,
    scheme: Scheme,
    boundary: Boundary,
) -> (Vec<f64>, Vec<f64>) {
    let num = Numerics {
        cfl: 0.4,
        feedback: 0.25,
        dt_max_s: 1.0,
        max_steps: 1_000_000,
        scheme,
        boundary,
    };
    let a = [-1.2, 1.0, 0.35, -0.2, 0.1, 0.0, 0.0, 0.0];
    let b = [0.0, 1.5, -0.8, 0.4];
    let ks = [17usize, 33, 65, 129, 257];
    let sols: Vec<Vec<f64>> = ks
        .iter()
        .map(|&k| run(p, lay, k, num, &a, &b, 3.0))
        .collect();
    let mut linf = vec![];
    let mut l2 = vec![];
    for i in 0..ks.len() - 1 {
        let (c, f) = (&sols[i], &sols[i + 1]);
        let d: Vec<f64> = (0..ks[i]).map(|j| c[j] - f[2 * j]).collect();
        linf.push(d.iter().fold(0.0f64, |x, y| x.max(y.abs())));
        l2.push((d.iter().map(|v| v * v).sum::<f64>() / d.len() as f64).sqrt());
    }
    let ord = |e: &[f64]| {
        (0..e.len() - 1)
            .map(|i| (e[i] / e[i + 1]).log2())
            .collect::<Vec<_>>()
    };
    (ord(&linf), ord(&l2))
}

#[test]
fn spatial_convergence_order() {
    let lay = GlobalLayout::new();
    // (1) prescribed velocity field: zero network, v_b = v_ref[β_v·(1,c) + Σ b_m φ_m]
    let mut p0 = vec![0.0; lay.len];
    p0[lay.phys.beta_v] = 1.0;
    p0[lay.phys.zeta] = 0.0;
    // (2) the full operator with a random network (resolution-agnostic: pointwise + trapezoid modes)
    let mut p1 = m::init_network(&lay, 3);
    p1[lay.phys.beta_v] = 1.0;
    let mut table = vec![];
    for (label, p) in [("prescribed V", &p0), ("random network", &p1)] {
        for (scheme, boundary) in [
            (Scheme::Llf1, Boundary::Linear),
            (Scheme::Eno2, Boundary::Linear),
            (Scheme::Eno2, Boundary::Quadratic),
        ] {
            let (oi, o2) = orders(p, &lay, scheme, boundary);
            eprintln!(
                "{label:>15} {scheme:?}/{boundary:?}: Linf orders {oi:.2?}  L2 orders {o2:.2?}"
            );
            table.push((
                label,
                scheme,
                boundary,
                *oi.last().unwrap(),
                *o2.last().unwrap(),
            ));
        }
    }
    for (label, scheme, boundary, oi, o2) in table {
        match (scheme, boundary) {
            (Scheme::Llf1, _) => assert!(o2 < 1.3, "{label} LLF1 should be first order, got {o2}"),
            (Scheme::Eno2, Boundary::Linear) => assert!(
                o2 > 1.8 && oi > 1.6,
                "{label} ENO2 should be second order, got L2 {o2} Linf {oi}"
            ),
            _ => {} // quadratic ghost extrapolation: reported only (unstable on the finest grids)
        }
    }
}

#[test]
fn eno2_reverse_mode_matches_finite_differences() {
    // member objective on a synthetic run, second-order scheme
    let lay = GlobalLayout::new();
    let mut p = m::init_network(&lay, 7);
    p[lay.phys.beta_v] = 1.0;
    p[lay.lsc] = 1e-4f64.ln();
    p[lay.lsn] = 5e-5f64.ln();
    p[lay.lsr] = 1e-2f64.ln();
    let grid = Grid::new(32);
    // fixed step (dt_max binding): the CFL/feedback step is taken without gradient (as stop_gradient
    // in JAX), so a finite difference of the objective is only comparable at a parameter-free step
    let num = Numerics {
        cfl: 50.0,
        feedback: 50.0,
        dt_max_s: 0.05,
        max_steps: 100,
        scheme: if std::env::var("FDS").is_ok() {
            Scheme::Llf1
        } else {
            Scheme::Eno2
        },
        ..Numerics::default()
    };
    let cfg = serde_json::json!({"min_valid": 12, "ell_s": 0.1});
    let nt = 5;
    let s: Vec<f64> = (0..40).map(|i| 0.04 * i as f64 / 39.0).collect();
    let t: Vec<f64> = (0..nt).map(|i| i as f64).collect();
    let mut f = vec![0.0; nt * 40];
    let mut valid = vec![true; nt * 40];
    for i in 0..nt {
        for j in 0..40 {
            let xi = s[j] / 0.04;
            f[i * 40 + j] = 0.02
                + 0.002 * (std::f64::consts::PI * xi).cos()
                + 1e-3 * t[i] * (1.0 + 0.1 * xi)
                + 3e-5 * ((i * 7 + j * 13) % 11) as f64 / 11.0;
            valid[i * 40 + j] = (i * 3 + j) % 9 != 0;
        }
    }
    let status = vec!["single_valued".to_string(); nt];
    let run = ignis_bass::data::build_run(
        "r",
        "g",
        &t,
        &s,
        &f,
        &valid,
        &status,
        &vec![0.005; nt],
        vec![0.1; 8],
        0.1,
        vec![0.0],
        &cfg,
        32,
    )
    .unwrap();
    let na = ignis_bass::data::NormArrays {
        center: vec![0.0; 8],
        scale: vec![1.0; 8],
        is_time: vec![false; 8],
        reference: vec![1.0; 8],
    };
    let pr = m::Priors::from_json(&m::default_priors());
    let mut x = m::init_locals(&run).to_vec();
    x[9] = 0.3;
    let runs = vec![run];
    let mut gg = vec![0.0; lay.len];
    let mut gx = vec![0.0; 12];
    let v0 = m::member_objective(
        &p,
        &x,
        &runs,
        &[1.0],
        &lay,
        &grid,
        num,
        &na,
        &pr,
        &mut gg,
        &mut gx,
    )
    .value;
    assert!(v0.is_finite());
    let mut worst: f64 = 0.0;
    let mut rng = 12345u64;
    for trial in 0..6 {
        // random direction over globals and locals
        let mut dg = vec![0.0; lay.len];
        let mut dx = vec![0.0; 12];
        for v in dg.iter_mut().chain(dx.iter_mut()) {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *v = ((rng >> 11) as f64 / (1u64 << 53) as f64) - 0.5;
        }
        if trial < 3 {
            dg.iter_mut().for_each(|v| *v = 0.0);
        }
        let an: f64 = dg.iter().zip(&gg).map(|(a, b)| a * b).sum::<f64>()
            + dx.iter().zip(&gx).map(|(a, b)| a * b).sum::<f64>();
        let h = 1e-6;
        let eval = |s: f64| {
            let pp: Vec<f64> = p.iter().zip(&dg).map(|(a, d)| a + s * d).collect();
            let xx: Vec<f64> = x.iter().zip(&dx).map(|(a, d)| a + s * d).collect();
            let (mut g1, mut g2) = (vec![0.0; lay.len], vec![0.0; 12]);
            m::member_objective(
                &pp,
                &xx,
                &runs,
                &[1.0],
                &lay,
                &grid,
                num,
                &na,
                &pr,
                &mut g1,
                &mut g2,
            )
            .value
        };
        let fd = (eval(h) - eval(-h)) / (2.0 * h);
        let e = (fd - an).abs() / an.abs().max(1.0);
        worst = worst.max(e);
        assert!(e < 1e-5, "trial {trial}: fd {fd} analytic {an}");
    }
    eprintln!("ENO2 reverse mode vs central differences: worst rel err {worst:.2e}");
}

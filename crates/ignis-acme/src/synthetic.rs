//! Synthetic ACME event rows for smoke tests, benchmarks and statistical task generation. Labelled
//! synthetic; never enters a NASA cohort (`trained_on = "synthetic"`).
//!
//! Generated from a declared piecewise-constant hazard with correlated session frailties, exact
//! competing-risk sampling, scheduled stops as atoms, and a mix of exact / interval / right censoring.
//! Feature keys are the real view keys of `configs/models/acme/*.toml`.

use std::collections::BTreeMap;

use ignis_core::views::AcmeEventRow;
use rand::{rngs::StdRng, RngExt, SeedableRng};
use rand_distr::{Exp1, StandardNormal};

pub fn synthetic_features(rng: &mut StdRng) -> BTreeMap<String, f64> {
    let n = |r: &mut StdRng| r.sample::<f64, _>(StandardNormal);
    let fuel = rng.random_range(0..3);
    BTreeMap::from([
        ("pressure".into(), 101325.0 * (0.3 * n(rng)).exp()),
        ("injected_mass_flow".into(), 1e-6 * (0.4 * n(rng)).exp()),
        ("x_o2_role".into(), rng.random_range(0.15..0.4)),
        ("x_fuel_role".into(), rng.random_range(0.2..1.0)),
        ("tad".into(), rng.random_range(1600.0..2400.0)),
        ("zst".into(), rng.random_range(0.05..0.3)),
        ("fuel_is_ethane".into(), (fuel == 1) as u8 as f64),
        ("fuel_is_butane".into(), (fuel == 2) as u8 as f64),
    ])
}

#[allow(clippy::too_many_arguments)]
pub fn synthetic_rows(
    configuration: &str,
    n_sessions: usize,
    runs_per_session: usize,
    seed: u64,
    t_entry_s: f64,
    stop_s: f64,
    rate_e: f64,
    rate_o: f64,
    sigma_u: (f64, f64),
    rho: f64,
) -> Vec<AcmeEventRow> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut rows = vec![];
    for g in 0..n_sessions {
        let (z0, z1): (f64, f64) = (rng.sample(StandardNormal), rng.sample(StandardNormal));
        let u = (
            sigma_u.0 * z0,
            sigma_u.1 * (rho * z0 + (1.0 - rho * rho).sqrt() * z1),
        );
        for i in 0..runs_per_session {
            let f = synthetic_features(&mut rng);
            let lp = (f["pressure"] / 101325.0).ln();
            let lam_e = rate_e * (0.5 * lp + 1.5 * (f["x_o2_role"] - 0.27) + u.0).exp();
            let lam_o = rate_o * u.1.exp();
            let lam = lam_e + lam_o;
            let t = t_entry_s + rng.sample::<f64, _>(Exp1) / lam;
            let cause = if rng.random::<f64>() < lam_e / lam {
                "extinction"
            } else {
                "operator_termination"
            };
            let scheduled = (i % 2 == 0).then_some(stop_s);
            let mode: f64 = rng.random();
            let row =
                |cens: &str, lo: Option<f64>, hi: Option<f64>, causes: Vec<&str>| AcmeEventRow {
                    run_id: format!("{configuration}-s{g}-r{i}"),
                    group_id: format!("session-{g}"),
                    configuration: configuration.into(),
                    features: f.clone(),
                    t_entry_s,
                    censoring: cens.into(),
                    t_lower_s: lo,
                    t_upper_s: hi,
                    allowed_causes: causes.into_iter().map(String::from).collect(),
                    scheduled_stop_s: scheduled,
                    time_origin: "synthetic".into(),
                };
            if scheduled.is_some_and(|s| t >= s) {
                rows.push(row(
                    "scheduled_stop",
                    scheduled,
                    scheduled,
                    vec!["operator_termination"],
                ));
            } else if mode < 0.15 {
                let c = t_entry_s + (t - t_entry_s) * rng.random_range(0.3..0.9);
                rows.push(row("right", Some(c), None, vec![]));
            } else if mode < 0.55 {
                let lo = t_entry_s.max(t - rng.random_range(0.5..3.0));
                let mut hi = t + rng.random_range(0.5..3.0);
                if let Some(s) = scheduled {
                    hi = hi.min(s - 1e-6);
                }
                let causes = if mode < 0.25 {
                    vec!["extinction", "operator_termination"]
                } else {
                    vec![cause]
                };
                rows.push(row("interval", Some(lo), Some(hi), causes));
            } else {
                rows.push(row("event", Some(t), Some(t), vec![cause]));
            }
        }
    }
    rows
}

/// Default synthetic cohort (8 sessions × 6 runs, as in the Python fixture).
pub fn default_rows(configuration: &str, seed: u64) -> Vec<AcmeEventRow> {
    synthetic_rows(
        configuration,
        8,
        6,
        seed,
        2.0,
        60.0,
        0.03,
        0.01,
        (0.3, 0.3),
        0.3,
    )
}

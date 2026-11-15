//! The FLEX strict forward on synthetic prefixes with test-only weights: units and feature order,
//! the encoder's sign, the physics invariants of the burn/extinction mixture, and refusals.

mod common;

use ignis_core::contracts::{Measured, PrefixBundle, Resolution, SupportGate};
use ignis_core::expert::Expert;
use ignis_core::hashing::content_hash;
use ignis_wsp::flex::{self, Forecast};
use ignis_wsp::flex_tools;
use serde_json::{json, Value};

const D0: f64 = 3.0e-3;

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

/// A D^2-law droplet burning at `k` in air at 101325 Pa, observed to 3 s.
fn prefix(k: f64) -> PrefixBundle {
    flex_tools::fixture_prefix(D0, k, 3.0, 0.21, 0.0, 0.0, 101_325.0)
}

fn run(extra: &str, p: &PrefixBundle, settings: Value) -> Forecast {
    let release = common::flex_release(extra);
    flex::forecast(&flex::spec(), &release, p, &settings)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn median(f: &Forecast) -> &[f64] {
    let i = f
        .levels
        .iter()
        .position(|p| *p == 0.5)
        .expect("median level");
    &f.quantiles[i]
}

fn d2(k: f64, t: f64) -> f64 {
    (D0 * D0 - k * t).sqrt()
}

#[test]
fn a_constant_rate_reproduces_the_d2_law_in_si_units() {
    // toy weights: K = 0.5e-6 m2/s; the droplet burns at exactly that rate
    let f = run(
        "",
        &prefix(0.5e-6),
        json!({"horizon_s": 8.0, "n_points": 17}),
    );
    assert_eq!(f.times_s[0], 3.0);
    for (j, t) in f.times_s.iter().enumerate() {
        let want = d2(0.5e-6, *t);
        assert!((median(&f)[j] - want).abs() < 1e-9 * want, "t = {t}");
        assert!((f.mean[j] - want).abs() < 1e-9 * want);
        // no assumed spread and no noise: every quantile is the trajectory
        assert!(f
            .quantiles
            .iter()
            .all(|q| (q[j] - want).abs() < 1e-9 * want));
    }
    assert!((f.encoded.q0_m2 - D0 * D0).abs() < 1e-12 * D0 * D0);
    assert!(f.encoded.e_late.abs() < 1e-9 && f.encoded.e_early.abs() < 1e-9);
}

#[test]
fn each_condition_reaches_its_own_column_with_its_declared_transform() {
    // One weight at a time on the linear law; the droplet is made to burn at the rate that weight
    // implies, so a swapped column or a wrong unit shows up as a non-zero innovation.
    let cases: [(usize, f64, PrefixBundle); 5] = [
        (
            0,
            (0.30 - 0.21) / 0.05,
            flex_tools::fixture_prefix(D0, 0.0, 3.0, 0.30, 0.0, 0.0, 1.0e5),
        ),
        (
            1,
            0.40 / 0.20,
            flex_tools::fixture_prefix(D0, 0.0, 3.0, 0.21, 0.40, 0.0, 1.0e5),
        ),
        (
            2,
            0.30 / 0.20,
            flex_tools::fixture_prefix(D0, 0.0, 3.0, 0.21, 0.0, 0.30, 1.0e5),
        ),
        (
            3,
            3.0_f64.ln(),
            flex_tools::fixture_prefix(D0, 0.0, 3.0, 0.21, 0.0, 0.0, 3.0e5),
        ),
        (
            4,
            (D0 / 2.0e-3).ln(),
            flex_tools::fixture_prefix(D0, 0.0, 3.0, 0.21, 0.0, 0.0, 1.0e5),
        ),
    ];
    for (col, x, shell) in cases {
        let k = 0.5e-6 * (0.3 * x).exp();
        let mut p = flex_tools::fixture_prefix(D0, k, 3.0, 0.21, 0.0, 0.0, 1.0e5);
        p.known_conditions = shell.known_conditions;
        let p = rehash(p);
        let mut skip = [0.0; 7];
        skip[col] = 0.3;
        let f = run(
            &format!("rate.skip[0:7] = {skip:?}"),
            &p,
            json!({"horizon_s": 5.0}),
        );
        // c_logd0 comes from the prefix (head-window mean advanced to t = 0 in one pass), the
        // other four straight from the conditions
        let tol = if col == 4 { 1e-4 } else { 1e-9 };
        assert!(
            (f.encoded.c[col] - x).abs() < tol,
            "feature {col}: {} != {x}",
            f.encoded.c[col]
        );
        assert!(
            f.encoded.e_late.abs() < 1e-4,
            "column {col}: e_late = {}",
            f.encoded.e_late
        );
        let last = f.times_s.len() - 1;
        let want = d2(k, f.times_s[last]);
        assert!(
            (median(&f)[last] - want).abs() < 1e-5 * want,
            "column {col}"
        );
    }
}

#[test]
fn the_encoder_moves_the_rate_towards_what_the_prefix_shows_and_never_fits() {
    // population law 0.5e-6, the droplet burns 60 % faster
    let p = prefix(0.8e-6);
    let t_end = 8.0;
    let settings = json!({"times_s": [t_end]});
    // gain 0: the population law, whatever the prefix shows
    let off = run("", &p, settings.clone());
    assert!((off.encoded.e_late - 1.6_f64.ln()).abs() < 1e-6);
    assert!(off.encoded.b.abs() < 1e-12);
    let pop = (D0 * D0 - 0.8e-6 * 3.0 - 0.5e-6 * (t_end - 3.0)).sqrt();
    assert!((median(&off)[0] - pop).abs() < 1e-8);
    // gain 1 on the late difference: the observed rate
    let on = run("enc.gain_b = 50.0", &p, settings.clone());
    assert!((on.encoded.b - 1.6_f64.ln()).abs() < 1e-6);
    assert!((median(&on)[0] - d2(0.8e-6, t_end)).abs() < 1e-8);
    // half gain sits strictly between
    let half = run("enc.gain_b = 0.0", &p, settings);
    assert!(median(&on)[0] < median(&half)[0] && median(&half)[0] < median(&off)[0]);
    // the innovation is clamped: a prefix that does not shrink cannot drag the rate to zero
    let flat = run(
        "enc.gain_b = 50.0",
        &prefix(0.0),
        json!({"times_s": [t_end]}),
    );
    assert_eq!(flat.encoded.e_late, -1.0);
}

#[test]
fn extinction_freezes_the_droplet_and_probabilities_stay_a_partition() {
    // extinction intensity ln(eta) = 3 + 12 (w - 0.5): the flame goes out around half burned
    let haz =
        "haz.b0 = -3.0\nhaz.skip[0:7] = [0, 0, 0, 0, 0, 12.0, 0]\nhaz.log_disruption = -3.0\n\
               unc.s_pop = 0.2\nunc.sigma_d = 2.0e-5";
    let f = run(
        haz,
        &prefix(0.5e-6),
        json!({"horizon_s": 27.0, "n_points": 55}),
    );
    let n = f.times_s.len();
    for j in 0..n {
        let total: f64 = f.events.iter().map(|r| r[j]).sum();
        assert!(
            (total - 1.0).abs() < 1e-9,
            "partition at t = {}: {total}",
            f.times_s[j]
        );
        assert!(f.events.iter().all(|r| (0.0..=1.0).contains(&r[j])));
        for l in 1..f.levels.len() {
            assert!(
                f.quantiles[l][j] >= f.quantiles[l - 1][j],
                "quantile crossing"
            );
        }
        if j > 0 {
            // cumulative incidences never decrease, the active mass never grows, nothing regrows
            assert!(f.events[1][j] >= f.events[1][j - 1] - 1e-12);
            assert!(f.events[2][j] >= f.events[2][j - 1] - 1e-12);
            assert!(f.events[0][j] <= f.events[0][j - 1] + 1e-12);
            assert!(median(&f)[j] <= median(&f)[j - 1] + 1e-12);
        }
    }
    assert_eq!(f.events[0][0], 1.0);
    assert!(f.partition_max_error < 1e-9);
    // by the end nearly every scenario has extinguished near w = 0.5 and stayed there (k_evap = 0)
    assert!(f.events[1][n - 1] > 0.5 && f.events[2][n - 1] > 0.02);
    let frozen = median(&f)[n - 1];
    let w = 1.0 - frozen * frozen / (D0 * D0);
    assert!((0.35..0.65).contains(&w), "frozen at w = {w}");
    assert!((median(&f)[n - 1] - median(&f)[n - 8]).abs() < 1e-7);
    // without extinction the same droplet would be far smaller by then
    assert!(frozen > 1.2 * d2(0.5e-6, 16.0));

    // an extinguished droplet keeps evaporating at k_evap
    let slow = run(
        &format!("{haz}\npost.k_evap = 5.0e-8"),
        &prefix(0.5e-6),
        json!({"horizon_s": 27.0, "n_points": 55}),
    );
    assert!(median(&slow)[n - 1] < frozen);
}

#[test]
fn refusals_name_their_gate_and_the_expert_exposes_no_fitting_route() {
    let release = common::flex_release("");
    let spec = flex::spec();
    let refused = |p: PrefixBundle, settings: Value| {
        flex::forecast(&spec, &release, &rehash(p), &settings)
            .expect("runnable")
            .expect_err("should be refused")
    };
    // a stored pressure of zero (it occurs in the curated records) is a refusal, not ln(0)
    let mut p = prefix(0.5e-6);
    p.known_conditions.get_mut("pressure").unwrap().value_si = Some(0.0);
    assert_eq!(refused(p, Value::Null).gate, SupportGate::Condition);
    // a missing or unresolved condition is never defaulted
    let mut p = prefix(0.5e-6);
    p.known_conditions.remove("x_he");
    assert_eq!(refused(p, Value::Null).gate, SupportGate::Condition);
    let mut p = prefix(0.5e-6);
    let m: &mut Measured = p.known_conditions.get_mut("x_o2").unwrap();
    m.resolution = Resolution::UnresolvedConflict;
    assert_eq!(refused(p, Value::Null).gate, SupportGate::Condition);
    // outside the declared physical domain
    let p = flex_tools::fixture_prefix(D0, 0.5e-6, 3.0, 0.60, 0.0, 0.0, 101_325.0);
    assert_eq!(refused(p, Value::Null).gate, SupportGate::Condition);
    // phase not established, too few observations, beyond the horizon
    let mut p = prefix(0.5e-6);
    p.phase_established_at_s = None;
    assert_eq!(refused(p, Value::Null).gate, SupportGate::Semantic);
    let mut p = prefix(0.5e-6);
    p.observations.truncate(1);
    assert_eq!(refused(p, Value::Null).gate, SupportGate::Measurement);
    assert_eq!(
        refused(prefix(0.5e-6), json!({"times_s": [5.0, 31.0]})).gate,
        SupportGate::Horizon
    );
    // observations before the time origin are not used
    let mut p = prefix(0.5e-6);
    for o in p.observations.iter_mut().take(20) {
        o.time_s -= 5.0;
        o.available_at_s = o.time_s;
    }
    p.observations.sort_by(|a, b| a.time_s.total_cmp(&b.time_s));
    let f = flex::forecast(&spec, &release, &rehash(p), &Value::Null)
        .unwrap()
        .unwrap();
    assert_eq!(f.encoded.n_valid, 91 - 20);

    // a refusal artifact keeps the target and carries no numbers
    let mut p = prefix(0.5e-6);
    p.phase_established_at_s = None;
    let a = flex::artifact(&spec, &release, &rehash(p), &Value::Null).unwrap();
    assert!(!a.support.admitted && a.quantiles.is_empty() && a.mean.is_empty());
    assert_eq!(a.unit.as_deref(), Some("m"));
    a.validate().unwrap();

    // a release of another spec is not reinterpreted
    let mut other = common::flex_release("");
    other.release.construction.as_mut().unwrap().spec_hash =
        ignis_core::contracts::Digest::of_bytes(b"another spec");
    assert!(flex::forecast(&spec, &other, &prefix(0.5e-6), &Value::Null).is_err());

    // the expert cannot be fitted and offers the lab no gradient
    let expert = flex::FlexWsp::new();
    assert!(expert.objective(&release, &release.params, &[]).is_none());
    assert!(expert.candidate_predictive().is_none());
    let artifact = expert
        .forecast(&release, &prefix(0.5e-6), &Value::Null)
        .unwrap();
    artifact.validate().unwrap();
    assert_eq!(
        artifact.event_categories,
        [
            "active_supported",
            "hot_phase_end",
            "disruption",
            "unresolved"
        ]
    );
}

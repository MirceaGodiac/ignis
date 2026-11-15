//! Structural, synthetic checks of the oxygen closure; no dataset or scoring access.
//! The values below are test instrumentation, not an authored scientific candidate.

mod common;

use ignis_core::contracts::TensorFill;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::{assemble, front2, front3, models, text};

const PREDECESSOR: &str =
    include_str!("../../../configs/models/bass/front2/baseline-persistence/edits.wsp");

const OXYGEN: &str = "
oxygen.mix = 1.0
oxygen.member_weight = 1.0
oxygen.rho_kg_m3 = 1.0
oxygen.nu_m2_s = 0.0001
oxygen.alpha_m2_s = 0.0002
oxygen.molar_o2_kg_mol = 0.032
oxygen.molar_carrier_kg_mol = 0.028
oxygen.fuel_areal_kg_m2 = 0.02
oxygen.faces = 2.0
oxygen.stoich_kg_o2_kg_fuel = 1.2
oxygen.residence_s = 4.0
oxygen.reference_width_m = 0.025
";

const LEVELS: [f64; 3] = [0.1, 0.5, 0.9];

/// Replace named tensor assignments in test-only source, never candidate files.
fn replace(base: &str, extra: &str) -> String {
    let name = |line: &str| {
        line.trim_start()
            .split(['[', ' ', '='])
            .next()
            .unwrap_or_default()
            .to_string()
    };
    let mut result = String::new();
    for line in base.lines() {
        if !extra.lines().any(|e| name(e) == name(line)) {
            result.push_str(line);
            result.push('\n');
        }
    }
    result.push_str(extra);
    result
}

fn release(key: &str, source: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("registered model");
    let spec = model.spec();
    let ops = text::compile(source, &spec).expect("test literals compile");
    let patch = assemble::make_patch(&spec, common::meta("test-front3"), ops);
    let params = assemble::materialize(&spec, &patch, None).expect("complete test tensors");
    let release = model
        .build_release(&patch, &params, None, &common::binding())
        .expect("synthetic release");
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

fn physical(extra: &str) -> LoadedRelease {
    release(
        "bass_front3",
        &replace(&format!("{PREDECESSOR}\n{OXYGEN}"), extra),
    )
}

fn laws(r: &LoadedRelease, u_cm_s: f64, x: f64, width_ratio: f64) -> front3::Laws {
    front3::laws_of(&front3::spec(), r, u_cm_s, x, width_ratio).expect("law")
}

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance * expected.abs().max(f64::MIN_POSITIVE),
        "actual {actual:.17e}, expected {expected:.17e}"
    );
}

fn input() -> front3::FrontInput {
    front3::FrontInput {
        cutoff_s: 20.0,
        igniter_on_s: -3.0,
        x_o2: 0.2,
        width_ratio: 0.75,
        window_w: Some(20.0),
        flow: vec![(-3.0, 5.0), (7.0, 2.0), (24.0, 0.0), (35.0, 3.0)],
        tip: vec![(0.0, 0.2), (3.0, 0.5), (17.0, 0.9), (20.0, 1.1)],
        base: vec![(0.0, 0.0), (3.0, 0.0), (17.0, 0.2), (20.0, 0.3)],
        times: vec![20.0, 21.0, 30.0, 40.0, 60.0],
    }
}

#[test]
fn every_new_property_is_explicit_and_no_physical_defaults_exist() {
    let spec = front3::spec();
    spec.validate().expect("valid schema");
    assert_eq!(spec.spec_id.as_str(), "bass_front-wsp-v3");
    assert_eq!(spec.model_name, "bass_front.wsp3");
    assert_eq!(spec.forward.version, "3");
    let new: Vec<_> = spec
        .tensors
        .iter()
        .filter(|t| t.name.starts_with("oxygen."))
        .collect();
    assert_eq!(new.len(), 12);
    for tensor in new {
        assert!(tensor.shape.is_empty());
        assert!(matches!(tensor.fill, TensorFill::AuthorWritesAll));
    }
    let ops = text::compile(PREDECESSOR, &spec).unwrap();
    let patch = assemble::make_patch(&spec, common::meta("test-missing-oxygen"), ops);
    assert!(assemble::materialize(&spec, &patch, None).is_err());
}

#[test]
fn steady_quadratic_balance_uses_mass_fraction_faces_si_and_width() {
    let r = physical("");
    let x = 0.2;
    let y = x * 0.032 / (x * 0.032 + (1.0 - x) * 0.028);
    let a_supply = 2.0 * 1.0 * y / (0.02 * 1.2);
    for u_m_s in [0.0, 0.000001, 0.05, 1.0] {
        let law = laws(&r, u_m_s * 100.0, x, 0.5);
        let speed_m_s = law.v * (0.025 * 0.5);
        let rhs = a_supply * a_supply * (0.0001 * speed_m_s * 4.0 * u_m_s + 0.0002_f64.powi(2));
        close(speed_m_s.powi(2), rhs, 2e-14);
        close(law.v_zero_flow * (0.025 * 0.5), a_supply * 0.0002, 2e-14);
    }
    // Width only changes the ruler: the physical two-dimensional speed is unchanged.
    close(
        laws(&r, 5.0, x, 0.5).v,
        2.0 * laws(&r, 5.0, x, 1.0).v,
        1e-14,
    );
    // Doubling total loading AND exposed faces leaves per-face loading unchanged.
    let doubled = physical("oxygen.faces = 4.0\noxygen.fuel_areal_kg_m2 = 0.04\n");
    close(
        laws(&doubled, 5.0, x, 1.0).v,
        laws(&r, 5.0, x, 1.0).v,
        1e-14,
    );
}

#[test]
fn zero_low_and_high_flow_limits_and_zero_oxygen() {
    let r = physical("");
    let x = 0.2;
    let y = x * 0.032 / (x * 0.032 + (1.0 - x) * 0.028);
    let a_supply: f64 = 2.0 * y / (0.02 * 1.2);
    let zero = a_supply * 0.0002;
    let slope = a_supply.powi(2) * 0.0001 * 4.0;
    close(laws(&r, 0.0, x, 1.0).v * 0.025, zero, 1e-14);
    let du_m_s = 1e-7;
    let small = laws(&r, du_m_s * 100.0, x, 1.0).v * 0.025;
    close((small - zero) / du_m_s, 0.5 * slope, 2e-6);
    let u_m_s = 1e6;
    let high = laws(&r, u_m_s * 100.0, x, 1.0).v * 0.025;
    let a = slope * u_m_s;
    close(high, a + zero * zero / a, 1e-14);
    assert!(high > small && small > zero);
    let no_oxygen = laws(&r, 10.0, 0.0, 1.0);
    assert_eq!(no_oxygen.v, 0.0);
    assert_eq!(no_oxygen.v_zero_flow, 0.0);
    // Transport speed zero does not switch the inherited extinction law off.
    assert!(no_oxygen.quench > 0.0);
}

#[test]
fn physical_speed_does_not_read_legacy_speed_weights_or_powers() {
    let r = physical("");
    let altered = physical(
        "speed.b0 = 4.0\nspeed.skip[0:3] = [0.1, -0.2, 0.3]\n\
         speed.pow[0:2] = [5.0, -5.0]\nspeed.split = 1.0\nspeed.pow_i = 6.0\n",
    );
    let before = laws(&r, 4.0, 0.17, 0.5);
    let after = laws(&altered, 4.0, 0.17, 0.5);
    assert_eq!(
        serde_json::to_string(&before).unwrap(),
        serde_json::to_string(&after).unwrap()
    );
}

#[test]
fn neutral_branch_preserves_complete_predecessor_forecast() {
    // Exercise all predecessor additions, including persistent flow paths and prefix anchoring.
    let legacy = replace(
        PREDECESSOR,
        "speed.b0 = 0.1\nspeed.split = 0.6\nspeed.pow_i = 2.0\n\
        speed.kappa = 0.05\nover.b0 = 0.4\nover.tau_up_s = 3.0\n\
        unc.flow_abs_cm_s = 0.2\nunc.flow_rel = 0.1\nunc.flow_w[0:3] = [1.0, 2.0, 1.0]\n",
    );
    let old = release("bass_front2", &legacy);
    let new = release(
        "bass_front3",
        &format!("{legacy}\n{}", replace(OXYGEN, "oxygen.mix = 0.0\n")),
    );
    let mut inp = input();
    for with_prefix in [true, false] {
        if !with_prefix {
            inp.tip.clear();
            inp.base.clear();
        }
        let before = front2::forecast_track(&front2::spec(), &old, &inp, &LEVELS)
            .unwrap()
            .unwrap();
        let after = front3::forecast_track(&front3::spec(), &new, &inp, &LEVELS)
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_string(&before).unwrap(),
            serde_json::to_string(&after).unwrap()
        );
    }
    let prefix = ignis_wsp::front_tools::fixture_prefix(20.0, 0.001);
    let before = front2::forecast(&front2::spec(), &old, &prefix, &serde_json::Value::Null)
        .unwrap()
        .unwrap();
    let after = front3::forecast(&front3::spec(), &new, &prefix, &serde_json::Value::Null)
        .unwrap()
        .unwrap();
    assert_eq!(before.width_m, after.width_m);
    assert_eq!(
        serde_json::to_string(&before.forecast).unwrap(),
        serde_json::to_string(&after.forecast).unwrap()
    );
}

#[test]
fn active_metric_adapter_uses_authored_reference_width() {
    let r = physical("");
    let prefix = ignis_wsp::front_tools::fixture_prefix(20.0, 0.001);
    let output = front3::forecast(&front3::spec(), &r, &prefix, &serde_json::Value::Null)
        .unwrap()
        .unwrap();
    assert_eq!(output.width_m, 0.022);
    let speed_si = output.forecast.encoded.laws_at_cutoff.v * output.width_m;
    close(speed_si, laws(&r, 3.0, 0.19, 1.0).v * 0.025, 1e-14);
}

#[test]
fn invalid_properties_are_rejected_without_silent_defaults() {
    let spec = front3::spec();
    let invalid = replace(
        &format!("{PREDECESSOR}\n{OXYGEN}"),
        "oxygen.fuel_areal_kg_m2 = 0.0\n",
    );
    let ops = text::compile(&invalid, &spec).unwrap();
    let patch = assemble::make_patch(&spec, common::meta("test-invalid-oxygen"), ops);
    assert!(assemble::materialize(&spec, &patch, None).is_err());
    let mut r = physical("");
    r.params.get_mut("oxygen.residence_s").unwrap().data[0] = 0.0;
    assert!(front3::laws_of(&spec, &r, 1.0, 0.2, 1.0).is_err());
    assert!(front3::forecast_track(&spec, &r, &input(), &LEVELS)
        .unwrap()
        .is_err());
    let mut r = physical("");
    r.params.get_mut("oxygen.reference_width_m").unwrap().data[0] = f64::MAX;
    assert!(front3::laws_of(&spec, &r, 1.0, 0.2, 2.0).is_err());
}

#[test]
fn oxygen_branch_retains_event_mass_and_independent_overhang_and_burnout() {
    let r = physical("over.b0 = 0.5\nover.tau_up_s = 3.0\nunc.flow_w[0:3] = [1.0, 2.0, 1.0]\nunc.flow_abs_cm_s = 0.2\n");
    let forecast = front3::forecast_track(&front3::spec(), &r, &input(), &LEVELS)
        .unwrap()
        .unwrap();
    assert!(forecast.quantiles.iter().flatten().all(|v| v.is_finite()));
    assert!(forecast.trace.front_w.windows(2).all(|p| p[1] >= p[0]));
    for (&quench, &end) in forecast.p_quenched.iter().zip(&forecast.p_reached_end) {
        assert!((0.0..=1.0).contains(&quench));
        assert!((0.0..=1.0).contains(&end));
        close(
            quench + (1.0 - quench) * end + (1.0 - quench) * (1.0 - end),
            1.0,
            1e-14,
        );
    }
    let changed = physical("oxygen.residence_s = 8.0\n");
    let before = laws(&physical(""), 5.0, 0.2, 1.0);
    let after = laws(&changed, 5.0, 0.2, 1.0);
    assert!(after.v > before.v);
    assert_eq!(before.tau_b, after.tau_b);
    assert_eq!(before.quench, after.quench);
    assert_eq!(before.h_ss, after.h_ss);
}

// Synthetic first-event apparatus: constant quench hazard, no spread or encoder influence.
const EVENT_TOY: &str = "
oxygen.reference_width_m = 0.022
quench.b0 = -2.302585092994046
end.linger_s = 0.0
unc.s_pop = 0.0
unc.s_floor = 0.0
unc.sigma_w = 0.0
unc.sigma_x0_w = 0.0
unc.rel_over = 0.0
unc.drift_w = 0.0
enc.gain_b = -20.0
";

fn event_input(times: Vec<f64>) -> front3::FrontInput {
    front3::FrontInput {
        cutoff_s: 0.0,
        igniter_on_s: 0.0,
        x_o2: 0.2,
        width_ratio: 1.0,
        window_w: Some(0.2),
        flow: vec![(0.0, 5.0)],
        tip: vec![],
        base: vec![],
        times,
    }
}

fn run_events(release: &LoadedRelease, inp: &front3::FrontInput) -> front3::Forecast {
    front3::forecast_track(&front3::spec(), release, inp, &LEVELS)
        .unwrap()
        .unwrap()
}

fn check_cifs(f: &front3::Forecast) {
    let mut order: Vec<_> = (0..f.times_s.len()).collect();
    order.sort_by(|&a, &b| f.times_s[a].total_cmp(&f.times_s[b]));
    let (mut previous_q, mut previous_end, mut previous_active) = (0.0, 0.0, 1.0);
    for i in order {
        let q = f.p_quenched[i];
        let end = (1.0 - q) * f.p_reached_end[i];
        let active = (1.0 - q) * (1.0 - f.p_reached_end[i]);
        assert!(q + 1e-14 >= previous_q && end + 1e-14 >= previous_end);
        assert!(active <= previous_active + 1e-14);
        close(active + q + end, 1.0, 1e-14);
        (previous_q, previous_end, previous_active) = (q, end, active);
    }
}

#[test]
fn end_event_is_absorbing_and_does_not_erase_pre_atom_quenching() {
    let r = physical(EVENT_TOY);
    let inp = event_input(vec![0.0, 0.5, 1.0, 10.0, 50.0, 200.0]);
    let f = run_events(&r, &inp);
    check_cifs(&f);
    let l = laws(&r, 5.0, 0.2, 1.0);
    let stop_grid_s = ((0.2 / l.v) / 0.25).ceil() * 0.25;
    let end_mass = (-l.quench * stop_grid_s).exp();
    assert!(f.p_quenched[3] > 0.0);
    close(
        (1.0 - f.p_quenched[3]) * f.p_reached_end[3],
        end_mass,
        1e-14,
    );
    assert_eq!(f.p_quenched[3], f.p_quenched[5]);
    assert_eq!(f.p_reached_end[3], f.p_reached_end[5]);
    let mut dense = inp.clone();
    dense.times = vec![200.0, 0.0, 10.0, 3.125, 1.0, 50.0, 0.5, 1.0];
    let g = run_events(&r, &dense);
    check_cifs(&g);
    for (i, t) in inp.times.iter().enumerate() {
        let j = dense.times.iter().position(|s| s == t).unwrap();
        assert_eq!(f.p_quenched[i], g.p_quenched[j]);
        assert_eq!(f.p_reached_end[i], g.p_reached_end[j]);
    }
    let mut initial_end = event_input(vec![0.0, 10.0]);
    initial_end.igniter_on_s = -3.0;
    initial_end.flow = vec![(-3.0, 5.0)];
    let already_ended = run_events(&r, &initial_end);
    assert_eq!(already_ended.p_quenched, vec![0.0, 0.0]);
    assert_eq!(already_ended.p_reached_end, vec![1.0, 1.0]);
}

#[test]
fn joint_mixes_position_cdfs_and_has_coherent_reinterpreted_events() {
    let source = replace(
        EVENT_TOY,
        "oxygen.member_weight = 0.5\nunc.sigma_x0_w = 0.02\n",
    );
    let joint = physical(&source);
    let mut inp = event_input(vec![0.0, 1.0, 10.0, 50.0, 200.0]);
    let f = run_events(&joint, &inp);
    check_cifs(&f);
    assert_eq!(f.paths.len(), 2);
    assert_eq!(f.paths[0].weight, 0.5);
    assert_eq!(f.paths[1].weight, 0.5);
    // Position CDF mixture has exactly the original predecessor member, with shared ruler.
    let old_source = replace(PREDECESSOR, "quench.b0 = -2.302585092994046\nend.linger_s = 0.0\nunc.s_pop = 0.0\nunc.s_floor = 0.0\nunc.sigma_w = 0.0\nunc.sigma_x0_w = 0.02\nunc.rel_over = 0.0\nunc.drift_w = 0.0\nenc.gain_b = -20.0\n");
    let old = release("bass_front2", &old_source);
    let old_forecast = front2::forecast_track(&front2::spec(), &old, &inp, &LEVELS)
        .unwrap()
        .unwrap();
    assert_eq!(f.paths[0].tip_median_w[1], old_forecast.trace.front_w[1]);
    // For an unbounded observation, every returned mixture quantile inverts the weighted CDF.
    inp.window_w = None;
    let uncut = run_events(&joint, &inp);
    for j in 0..inp.times.len() {
        for (k, &level) in LEVELS.iter().enumerate() {
            let x = uncut.quantiles[k][j];
            let cdf: f64 = uncut
                .paths
                .iter()
                .map(|path| {
                    path.weight
                        * ignis_wsp::quad::normal_cdf((x - path.tip_median_w[j]) / path.tip_sd_w[j])
                })
                .sum();
            close(cdf, level, 2e-10);
        }
        close(uncut.p_quenched[j], -(-0.1 * inp.times[j]).exp_m1(), 2e-13);
    }
    // With two causes, coherent reinterpreted end mass persists after all samples reach the end.
    for j in [3, 4] {
        let end = (1.0 - f.p_quenched[j]) * f.p_reached_end[j];
        let end_paths: f64 = f
            .paths
            .iter()
            .map(|p| p.weight * (1.0 - p.p_quenched[j]) * p.p_reached_end[j])
            .sum();
        close(end, end_paths, 1e-14);
    }
    assert!(old_forecast.p_quenched[4] > f.paths[0].p_quenched[4]);
    let zero_member = physical(&replace(
        EVENT_TOY,
        "oxygen.member_weight = 0.0\nunc.sigma_x0_w = 0.02\n",
    ));
    let original_input = event_input(vec![0.0, 1.0, 10.0, 50.0, 200.0]);
    assert_eq!(
        serde_json::to_string(&run_events(&zero_member, &original_input)).unwrap(),
        serde_json::to_string(&old_forecast).unwrap()
    );
}

#[test]
fn physical_route_requires_flow_prehistory_but_neutral_route_remains_compatible() {
    let mut inp = event_input(vec![1.0, 10.0]);
    inp.igniter_on_s = -3.0;
    let active = physical(EVENT_TOY);
    assert!(
        front3::forecast_track(&front3::spec(), &active, &inp, &LEVELS)
            .unwrap()
            .is_err()
    );
    let neutral = physical(&replace(EVENT_TOY, "oxygen.member_weight = 0.0\n"));
    assert!(
        front3::forecast_track(&front3::spec(), &neutral, &inp, &LEVELS)
            .unwrap()
            .is_ok()
    );
}

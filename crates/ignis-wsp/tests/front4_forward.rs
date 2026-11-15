//! Synthetic observation-operator checks. No NASA files, fitting or scoring.

mod common;

use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::{assemble, front3, front4, models, text};

const BASE: &str =
    include_str!("../../../configs/models/bass/front2/baseline-persistence/edits.wsp");
// Instrumentation only, not a scientific candidate or a change to inherited release weights.
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
oxygen.reference_width_m = 0.022
";
const LEVELS: [f64; 3] = [0.1, 0.5, 0.9];

fn source(extra: &str) -> String {
    let name = |line: &str| line.split(['=', '[']).next().unwrap().trim().to_owned();
    let original = format!("{BASE}\n{OXYGEN}");
    let mut result: Vec<_> = original
        .lines()
        .filter(|l| !extra.lines().any(|e| name(e) == name(l)))
        .collect();
    result.push(extra);
    result.join("\n")
}

fn release(key: &str, source: &str) -> LoadedRelease {
    let model = models::by_key(key).unwrap();
    let spec = model.spec();
    let patch = assemble::make_patch(
        &spec,
        common::meta("test-front4"),
        text::compile(source, &spec).unwrap(),
    );
    let params = assemble::materialize(&spec, &patch, None).unwrap();
    let release = model
        .build_release(&patch, &params, None, &common::binding())
        .unwrap();
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

fn input() -> front4::FrontInput {
    front4::FrontInput {
        cutoff_s: 20.0,
        igniter_on_s: -3.0,
        x_o2: 0.2,
        width_ratio: 1.0,
        window_w: None,
        flow: vec![(-3.0, 5.0), (2.0, 2.0), (17.0, 8.0), (26.0, 1.0)],
        tip: vec![
            (0.0, 0.1),
            (3.5, 0.4),
            (16.25, 1.0),
            (18.0, 1.2),
            (20.0, 1.4),
        ],
        base: vec![(1.5, 0.1), (4.0, 0.2), (16.5, 0.6), (19.5, 0.8)],
        times: vec![20.0, 24.0, 40.0],
    }
}

fn old(r: &LoadedRelease, i: &front4::FrontInput) -> front4::Forecast {
    front3::forecast_track(&front3::spec(), r, i, &LEVELS)
        .unwrap()
        .unwrap()
}

fn new(r: &LoadedRelease, i: &front4::FrontInput) -> front4::Forecast {
    front4::forecast_track(&front4::spec(), r, i, &LEVELS)
        .unwrap()
        .unwrap()
}

fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() < 1e-12 * a.abs().max(b.abs()).max(1.0),
        "{a:.17e} != {b:.17e}"
    );
}

fn forecasts_close(a: &front4::Forecast, b: &front4::Forecast) {
    for (x, y) in a
        .quantiles
        .iter()
        .flatten()
        .zip(b.quantiles.iter().flatten())
    {
        close(*x, *y);
    }
    for (x, y) in a.base_median.iter().zip(&b.base_median) {
        close(*x, *y);
    }
    for (x, y) in a.p_quenched.iter().zip(&b.p_quenched) {
        close(*x, *y);
    }
    for (x, y) in a.p_reached_end.iter().zip(&b.p_reached_end) {
        close(*x, *y);
    }
    close(a.encoded.e, b.encoded.e);
    close(a.encoded.gain, b.encoded.gain);
}

#[test]
fn new_identity_preserves_every_tensor_definition_except_prose() {
    let a = front3::spec();
    let b = front4::spec();
    b.validate().unwrap();
    assert_ne!(a.spec_id, b.spec_id);
    assert_eq!(b.model_name, "bass_front.wsp4");
    let strip_prose = |spec: &ignis_core::contracts::ModelSpec| {
        let mut ts = serde_json::to_value(&spec.tensors).unwrap();
        for tensor in ts.as_array_mut().unwrap() {
            tensor.as_object_mut().unwrap().remove("description");
        }
        ts
    };
    assert_eq!(strip_prose(&a), strip_prose(&b));
    assert_ne!(
        models::by_key("bass_front3").unwrap().forward_hash(),
        models::by_key("bass_front4").unwrap().forward_hash()
    );
}

#[test]
fn affine_paths_and_constant_overhang_match_the_old_operator() {
    let src = source("oxygen.member_weight = 0.5\n");
    let a = release("bass_front3", &src);
    let b = release("bass_front4", &src);
    let mut i = input();
    i.flow = vec![(-3.0, 5.0)];
    // Base windows must also stay on an affine portion of the delayed path.
    i.base.clear();
    forecasts_close(&old(&a, &i), &new(&b, &i));
}

#[test]
fn no_prefix_and_singleton_windows_preserve_old_predictions() {
    let src = source("over.b0 = 0.4\nover.tau_s = 5.0\noxygen.member_weight = 0.5\n");
    let a = release("bass_front3", &src);
    let b = release("bass_front4", &src);
    let mut i = input();
    i.tip.clear();
    i.base.clear();
    assert_eq!(
        serde_json::to_value(old(&a, &i)).unwrap(),
        serde_json::to_value(new(&b, &i)).unwrap()
    );
    i.tip = vec![(0.0, 0.1), (20.0, 1.4)];
    i.base = vec![(1.5, 0.1), (19.5, 0.8)];
    assert_eq!(
        serde_json::to_value(old(&a, &i)).unwrap(),
        serde_json::to_value(new(&b, &i)).unwrap()
    );
}

#[test]
fn irregular_nonlinear_samples_recover_the_known_trajectory_without_innovation() {
    let src = source("over.b0 = 0.4\nover.tau_s = 5.0\nburn.b0 = 0.0\nenc.mix[0:2] = [0.8, 0.2]\nunc.rel_over = 0.2\n");
    let a = release("bass_front3", &src);
    let b = release("bass_front4", &src);
    let mut i = input();
    let mut truth_input = i.clone();
    truth_input.cutoff_s = 0.0;
    truth_input.tip.clear();
    truth_input.base.clear();
    // Independent frozen predecessor path, sampled at the channel's own timestamps.
    truth_input.times = i
        .tip
        .iter()
        .map(|p| p.0)
        .chain(i.base.iter().map(|p| p.0 - 1.0))
        .chain(i.times.iter().copied())
        .collect();
    let truth = old(&a, &truth_input);
    for (j, obs) in i.tip.iter_mut().enumerate() {
        obs.1 = truth.mean[j];
    }
    let n_tip = i.tip.len();
    for (j, obs) in i.base.iter_mut().enumerate() {
        obs.1 = truth.trace.front_w[n_tip + j];
    }
    let corrected = new(&b, &i);
    close(corrected.encoded.e_tip.unwrap(), 0.0);
    close(corrected.encoded.e_base.unwrap(), 0.0);
    close(corrected.encoded.e, 0.0);
    for (j, value) in corrected.mean.iter().enumerate() {
        close(*value, truth.mean[n_tip + i.base.len() + j]);
    }
    assert!(
        old(&a, &i).encoded.e_tip.unwrap().abs() > 1e-6,
        "toy must expose the old averaging error"
    );
    // Prefix state and predictions cannot see future observations in either channel.
    i.tip.push((21.0, -999.0));
    i.base.push((21.0, 999.0));
    assert_eq!(
        serde_json::to_value(&corrected).unwrap(),
        serde_json::to_value(new(&b, &i)).unwrap()
    );
}

#[test]
fn registered_forward_returns_a_metric_artifact_and_absorbing_events() {
    let src = source("oxygen.member_weight = 0.5\nover.b0 = 0.4\n");
    let r = release("bass_front4", &src);
    let model = models::by_key("bass_front4").unwrap();
    let artifact = model
        .artifact(&r, &model.fixture_prefix(), &serde_json::Value::Null)
        .unwrap();
    let value = serde_json::to_value(artifact).unwrap();
    assert!(value
        .to_string()
        .contains("window_mean_at_admitted_still_times"));
    let mut i = input();
    i.window_w = Some(2.0);
    let f = new(&r, &i);
    let (mut previous_q, mut previous_end) = (0.0, 0.0);
    for (q, e) in f.p_quenched.iter().zip(f.p_reached_end) {
        assert!((0.0..=1.0).contains(q) && (0.0..=1.0).contains(&e));
        let end = (1.0 - q) * e;
        assert!(*q >= previous_q && end >= previous_end);
        (previous_q, previous_end) = (*q, end);
    }
}

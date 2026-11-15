//! The flame-front forward of `bass_front.wsp` (`front.rs`) with test-only weights: a steady front
//! is continued and its run offset read exactly; a step down in flow pulls the tip back by the
//! overhang change at the written relaxation time while the base keeps the old speed for one
//! burn-through time; the cut at the end of the sample and a "reached end" probability that never
//! comes back down; a flow-gated quench hazard; event rows that partition; the metric route on the
//! synthetic fixture (conversion by the sample width, the prefix clock) and its refusals; the reading
//! rules of the track files; the committed spec is the spec the forward was compiled with.

mod common;

use ignis_core::contracts::{CheckStatus, ModelSpec, PrefixBundle, SupportGate};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::front::{self, Forecast, FrontInput};
use ignis_wsp::front_tools::{self, rules, Track};
use ignis_wsp::{assemble, models, text};
use serde_json::{json, Value};

/// Test-only weights, not a candidate: a constant spread rate of 0.1 sample widths per second, no
/// overhang, a burn-through time of 5 s, no quench, the igniter piloting at the law's rate from the
/// moment it is switched on, an encoder gain of about 2e-9, no spread at all.
const TOY: &str = "
speed.b0 = 0.1
speed.skip[0:3] = 0.0
speed.pow[0:2] = 0.0
over.b0 = 0.0
over.skip[0:3] = 0.0
over.pow[0:2] = 0.0
over.tau_s = 1.0
burn.b0 = 1.6094379124341003
burn.skip[0:3] = 0.0
quench.b0 = -40.0
quench.skip[0:3] = 0.0
ign.gain = 1.0
ign.delay_s = 0.0
end.linger_s = 0.0
enc.gain_w[0:2] = 0.0
enc.gain_b = -20.0
enc.mix[0:2] = [1.0, 0.0]
enc.e_clip[0:2] = 1.0
unc.s_pop = 0.0
unc.s_floor = 0.0
unc.s_far[0:2] = 0.0
unc.sigma_w = 0.0
unc.sigma_x0_w = 0.0
unc.rel_over = 0.0
unc.drift_w = 0.0
";

const LEVELS: [f64; 5] = [0.05, 0.25, 0.5, 0.75, 0.95];
const MED: usize = 2;

/// The toy weights with `extra` statements replacing the base statements of the tensors they name.
fn release(extra: &str) -> LoadedRelease {
    let model = models::by_key("bass_front").expect("model");
    let spec = model.spec();
    let mut src = String::new();
    let name_of = |l: &str| {
        l.trim_start()
            .split(['[', ' ', '='])
            .next()
            .unwrap_or_default()
            .to_string()
    };
    for line in TOY.lines() {
        let name = name_of(line);
        if name.is_empty() || !extra.lines().any(|l| name_of(l) == name) {
            src.push_str(line);
            src.push('\n');
        }
    }
    src.push_str(extra);
    let ops = text::compile(&src, &spec).unwrap_or_else(|e| panic!("{e}\n{src}"));
    let patch = assemble::make_patch(&spec, common::meta("test-front"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("release");
    LoadedRelease {
        release,
        params: tensors,
        posterior: TensorMap::new(),
    }
}

fn run(rel: &LoadedRelease, input: &FrontInput) -> Forecast {
    front::forecast_track(&front::spec(), rel, input, &LEVELS)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

/// A burn of the reference sample at the reference oxygen, igniter on from -6 s, with one still per
/// second from 0 to `last_still_s` whose tip and base follow `tip` and `base`.
fn input(
    cutoff_s: f64,
    flow: Vec<(f64, f64)>,
    last_still_s: usize,
    tip: impl Fn(f64) -> f64,
    base: impl Fn(f64) -> f64,
    times: Vec<f64>,
) -> FrontInput {
    let ts: Vec<f64> = (0..=last_still_s).map(|i| i as f64).collect();
    FrontInput {
        cutoff_s,
        igniter_on_s: -6.0,
        x_o2: 0.19,
        width_ratio: 1.0,
        window_w: None,
        flow,
        tip: ts.iter().map(|t| (*t, tip(*t))).collect(),
        base: ts.iter().map(|t| (*t, base(*t))).collect(),
        times,
    }
}

fn grid(from: f64, to: f64, step: f64) -> Vec<f64> {
    let n = ((to - from) / step).round() as usize;
    (0..=n).map(|i| from + step * i as f64).collect()
}

#[test]
fn a_steady_front_is_continued_and_a_faster_prefix_is_carried_when_the_gain_is_one() {
    // the law: x = 0.1 (t + 6) from the igniter being switched on, no overhang
    let times = vec![10.0, 20.0, 40.0, 70.0];
    let law = |t: f64| 0.1 * (t + 6.0);
    let rel = release("");
    let f = run(
        &rel,
        &input(
            10.0,
            vec![(-60.0, 5.0)],
            10,
            law,
            |t| law(t) - 1.0,
            times.clone(),
        ),
    );
    let e = &f.encoded;
    assert!(e.e_tip.unwrap().abs() < 1e-9, "{e:?}");
    assert!(e.e.abs() < 1e-9 && e.gain < 1e-8, "{e:?}");
    assert_eq!(e.stills_used, 11);
    for (j, t) in times.iter().enumerate() {
        assert!(
            (f.quantiles[MED][j] - law(*t)).abs() < 1e-9,
            "median at {t}"
        );
        // no spread: every quantile is the median
        assert!((f.quantiles[0][j] - f.quantiles[4][j]).abs() < 1e-12);
    }

    // a prefix 20 % faster than the law, trusted fully: the forecast carries the 20 %
    let fast = |t: f64| 0.12 * (t + 6.0);
    let trusting = release("enc.gain_b = 20.0");
    let g = run(
        &trusting,
        &input(
            10.0,
            vec![(-60.0, 5.0)],
            10,
            fast,
            |t| fast(t) - 1.0,
            times.clone(),
        ),
    );
    assert!((g.encoded.e_tip.unwrap() - 1.2_f64.ln()).abs() < 1e-9);
    assert!((g.encoded.e - 1.2_f64.ln()).abs() < 1e-8);
    for (j, t) in times.iter().enumerate() {
        assert!(
            (g.quantiles[MED][j] - fast(*t)).abs() < 1e-6,
            "median at {t}"
        );
    }
}

#[test]
fn a_step_down_pulls_the_tip_back_by_the_overhang_change_and_the_base_keeps_the_old_speed() {
    // v = 0.1 + 0.05 u, h_ss = 0.5 + 0.3 u, u = U / 5 - 1: at 10 cm/s v = 0.15 and h = 0.8, at
    // 2 cm/s v = 0.07 and h = 0.32; tau_b = 5 s; the flow steps down at 20 s
    let rel = release(
        "speed.skip[0:3] = [0.05, 0.0, 0.0]\nover.b0 = 0.5\nover.skip[0:3] = [0.3, 0.0, 0.0]",
    );
    let (v0, v1, h0, h1, tau_b) = (0.15, 0.07, 0.8, 0.32, 5.0);
    let x = |t: f64| v0 * (t + 6.0);
    let times = grid(15.0, 40.0, 0.25);
    let f = run(
        &rel,
        &input(
            15.0,
            vec![(-60.0, 10.0), (20.0, 2.0)],
            15,
            |t| x(t) + h0,
            |t| x(t - tau_b),
            times.clone(),
        ),
    );
    let (t_end, tip_end) = (f.encoded.t_end_s, f.encoded.tip_end_w.unwrap());
    assert_eq!(t_end, 13.0);
    for (j, &t) in times.iter().enumerate() {
        let pre_line = tip_end + v0 * (t - t_end);
        let slower = (v0 - v1) * (t - 20.0).max(0.0);
        let deficit = pre_line - slower - f.quantiles[MED][j];
        let want = (h0 - h1) * (1.0 - (-(t - 20.0).max(0.0)).exp());
        assert!(
            (deficit - want).abs() < 1e-3,
            "tip deficit at {t}: {deficit} vs {want}"
        );
    }
    // within five relaxation times the whole overhang change is gone
    let j25 = times.iter().position(|t| *t == 25.0).unwrap();
    let deficit = tip_end + v0 * (25.0 - t_end) - (v0 - v1) * 5.0 - f.quantiles[MED][j25];
    assert!((deficit - (h0 - h1)).abs() < 0.01 * (h0 - h1));
    // the base: the old speed for tau_b seconds after the step, then the new one
    let at = |t: f64| f.base_median[times.iter().position(|s| *s == t).unwrap()];
    assert!((at(20.0) - x(15.0)).abs() < 1e-6);
    assert!(((at(25.0) - at(20.0)) / 5.0 - v0).abs() < 1e-9);
    assert!(((at(35.0) - at(25.0)) / 10.0 - v1).abs() < 1e-9);
}

#[test]
fn the_base_offset_reads_the_burn_through_time_at_the_later_window() {
    // tau_b = 4 s at 5 cm/s and 8 s at 10 cm/s; v = 0.15 at 10 cm/s and 0.1 at 5 cm/s. The flow is 10
    // until 4 s, 5 until 14 s, 10 after: the later window (8-12 s) burns at 5 cm/s, the cutoff at 10.
    let rel = release(
        "speed.skip[0:3] = [0.05, 0.0, 0.0]\nburn.b0 = 1.3862943611198906\n\
         burn.skip[0:3] = [0.6931471805599453, 0.0, 0.0]\nenc.mix[0:2] = [0.0, 1.0]",
    );
    let x = |t: f64| {
        if t <= 4.0 {
            0.15 * (t + 6.0)
        } else {
            1.5 + 0.1 * (t - 4.0)
        }
    };
    let f = run(
        &rel,
        &input(
            16.0,
            vec![(-60.0, 10.0), (4.0, 5.0), (14.0, 10.0)],
            12,
            x,
            |t| x(t - 4.0),
            vec![16.0],
        ),
    );
    // the base generated with the later window's burn-through time has no offset
    assert!(f.encoded.e_base.unwrap().abs() < 1e-9, "{:?}", f.encoded);
}

#[test]
fn a_flow_change_before_the_igniter_is_on_does_not_count_as_settling_time() {
    // stills at 0, 1, 2 s: the last window is centred at 1 s, 7 s after the igniter was switched on
    let f = run(
        &release(""),
        &input(
            2.0,
            vec![(-60.0, 10.0), (-20.0, 5.0)],
            2,
            |t| 0.1 * (t + 6.0),
            |t| 0.1 * (t + 1.0),
            vec![2.0],
        ),
    );
    assert!((f.encoded.settled - 0.7).abs() < 1e-12, "{:?}", f.encoded);
}

#[test]
fn a_fan_flicker_is_not_a_change_of_flow_but_a_slow_ramp_and_a_step_are() {
    // stills 0..20 s: the last window is centred at t_e = 18 s
    let settled = |flow: Vec<(f64, f64)>| {
        run(
            &release(""),
            &input(
                20.0,
                flow,
                20,
                |t| 0.1 * (t + 6.0),
                |t| 0.1 * (t + 1.0),
                vec![20.0],
            ),
        )
        .encoded
        .settled
    };
    // a one-unit flicker of the fan display, about 0.6 cm/s back and forth every 2 s
    let flicker: Vec<(f64, f64)> = (0..45)
        .map(|i| (-60.0 + 2.0 * i as f64, if i % 2 == 0 { 5.0 } else { 5.6 }))
        .collect();
    assert_eq!(settled(flicker), 1.0);
    // a ramp of 0.3 cm/s per second from 5 s: at 18 s the flow is 9.2 cm/s, the entry of 15 s
    // (8.3 cm/s) is the most recent one 0.8 cm/s away, so the flow last changed at 16 s
    let mut ramp = vec![(-60.0, 5.0)];
    ramp.extend((5..=20).map(|t| (t as f64, 5.0 + 0.3 * (t as f64 - 4.0))));
    assert!((settled(ramp) - 0.2).abs() < 1e-9);
    // a step of 2 cm/s at 15 s
    assert!((settled(vec![(-60.0, 5.0), (15.0, 7.0)]) - 0.3).abs() < 1e-12);
}

#[test]
fn the_tip_is_cut_at_the_end_of_the_sample_and_reaching_it_never_becomes_less_likely() {
    let rel = release("unc.sigma_w = 0.05\nunc.drift_w = 0.02");
    let times = grid(10.0, 80.0, 1.0);
    let mut inp = input(
        10.0,
        vec![(-60.0, 5.0)],
        10,
        |t| 0.1 * (t + 6.0),
        |t| 0.1 * (t + 1.0),
        times.clone(),
    );
    inp.window_w = Some(4.0);
    let f = run(&rel, &inp);
    for q in &f.quantiles {
        assert!(q.iter().all(|v| *v <= 4.0 + 1e-12));
    }
    let last = times.len() - 1;
    assert_eq!(f.quantiles[0][last], 4.0);
    assert!(
        (f.quantiles[MED][10] - 2.6).abs() < 1e-9,
        "uncut before the end"
    );
    assert!(f.p_reached_end[0] < 1e-6);
    assert!(f.p_reached_end[last] > 0.999);
    assert!(f.p_reached_end.windows(2).all(|w| w[1] >= w[0]));

    // a step down pulls a tip that was at the end back from it: the probability of having reached
    // the end keeps its earlier value, in time order even when the times are not sorted
    let rel = release(
        "speed.b0 = 0.01\nover.b0 = 0.5\nover.skip[0:3] = [0.3, 0.0, 0.0]\nunc.sigma_w = 0.05",
    );
    let times = vec![30.0, 15.0, 20.0, 25.0, 18.0];
    let mut inp = input(
        10.0,
        vec![(-60.0, 10.0), (20.0, 2.0)],
        10,
        |t| 0.01 * (t + 6.0) + 0.8,
        |t| 0.01 * (t + 1.0),
        times.clone(),
    );
    inp.window_w = Some(1.05);
    let f = run(&rel, &inp);
    let p = |t: f64| f.p_reached_end[times.iter().position(|s| *s == t).unwrap()];
    assert!(p(20.0) > 0.4, "the tip is at the end at 20 s: {}", p(20.0));
    assert!(p(15.0) <= p(18.0) && p(18.0) <= p(20.0) && p(20.0) <= p(25.0));
    assert_eq!(
        p(30.0),
        p(20.0),
        "the tip has pulled back, the probability stays"
    );
}

#[test]
fn a_flow_gated_quench_hazard_takes_the_flame_only_below_its_threshold() {
    // ln hazard = -40 + 300 relu(-0.6 - u): off above 2 cm/s, e^20 per second at 1 cm/s
    let gated = "quench.W1[0, 0] = -1.0\nquench.b1[0] = -0.6\nquench.w2[0] = 300.0";
    let rel = release(gated);
    let law = |t: f64| 0.1 * (t + 6.0);
    let times = vec![10.0, 19.0, 20.5, 21.0, 40.0];
    let f = run(
        &rel,
        &input(
            10.0,
            vec![(-60.0, 5.0), (20.0, 1.0)],
            10,
            law,
            law,
            times.clone(),
        ),
    );
    assert!(f.p_quenched[0] < 1e-12 && f.p_quenched[1] < 1e-12);
    assert!(f.p_quenched[2] > 0.999 && f.p_quenched[3] > 0.999 && f.p_quenched[4] > 0.999);
    let steady = run(
        &rel,
        &input(10.0, vec![(-60.0, 5.0)], 10, law, law, vec![10.0, 70.0]),
    );
    assert!(steady.p_quenched[1] < 1e-9);

    // the four event rows of the artifact partition the outcomes at every time
    let rel = release(&format!("{gated}\nunc.sigma_w = 0.05\nunc.drift_w = 0.02"));
    let p = front_tools::fixture_prefix(10.0, 0.001);
    let a = front::artifact(
        &front::spec(),
        &rel,
        &p,
        &json!({"times_s": [20.0, 50.0, 80.0, 110.0, 140.0]}),
    )
    .unwrap();
    a.validate().unwrap();
    assert!(a.support.admitted);
    assert_eq!(a.event_cdf.len(), 4);
    for j in 0..a.times_s.len() {
        let s: f64 = a.event_cdf.iter().map(|r| r[j]).sum();
        assert!((s - 1.0).abs() < 1e-12, "rows sum to {s} at {j}");
    }
    assert!(
        a.event_cdf[2].last().unwrap() > &0.5,
        "the tip reaches the end"
    );
}

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

#[test]
fn the_metric_route_scales_by_the_sample_width_on_the_prefix_clock_and_refuses_what_it_lacks() {
    let rel = release("unc.sigma_w = 0.05\nunc.drift_w = 0.02\nunc.s_pop = 0.2");
    let spec = front::spec();
    let p = front_tools::fixture_prefix(10.0, 0.001);
    let a = front::artifact(&spec, &rel, &p, &Value::Null).unwrap();
    a.validate().unwrap();
    assert!(a.support.admitted, "{:?}", a.support.reasons);
    assert_eq!(a.unit.as_deref(), Some("m"));
    // times on the prefix clock: the cutoff is 20 s there (the igniter is off at 10 s)
    assert_eq!(a.times_s[0], 20.0);
    let widths = a.diagnostics["tip_quantiles_sample_widths"]
        .as_array()
        .unwrap();
    for (l, row) in a.quantiles.iter().enumerate() {
        for (j, q) in row.iter().enumerate() {
            let w = widths[l][j].as_f64().unwrap();
            assert!((q - w * 0.022).abs() < 1e-12, "level {l} time {j}");
        }
    }
    // the encoder read the last window on the igniter-off clock and in sample widths: stills at
    // 6..10 s with tips 0.026..0.030 m
    let enc = &a.diagnostics["encoded"];
    assert_eq!(enc["t_end_s"].as_f64().unwrap(), 8.0);
    assert!((enc["tip_end_w"].as_f64().unwrap() - 0.028 / 0.022).abs() < 1e-12);

    let refused = |p: PrefixBundle| {
        front::forecast(&spec, &rel, &rehash(p), &Value::Null)
            .expect("runnable")
            .err()
            .expect("should be refused")
    };
    for key in ["width", "x_o2"] {
        let mut q = p.clone();
        q.known_conditions.remove(key);
        let s = refused(q);
        assert_eq!(s.gate, SupportGate::Condition);
        assert!(s.reasons.iter().any(|r| r.contains(key)), "{:?}", s.reasons);
    }
    let mut q = p.clone();
    q.known_controls.retain(|s| s.control != "igniter");
    assert!(refused(q).reasons[0].contains("igniter"));
    let mut q = p.clone();
    q.known_controls
        .iter_mut()
        .filter(|s| s.control == "igniter")
        .for_each(|s| s.t_end_s = None);
    assert!(refused(q).reasons[0].contains("end time"));
    let mut q = p.clone();
    q.known_controls.retain(|s| s.control != "flow_velocity");
    let s = refused(q);
    assert_eq!(s.gate, SupportGate::Condition);
    assert!(s.reasons[0].contains("flow_velocity"));
    // a refusal artifact keeps the metric target and carries no numbers
    let mut q = p.clone();
    q.known_conditions.remove("width");
    let r = front::artifact(&spec, &rel, &rehash(q), &Value::Null).unwrap();
    assert!(!r.support.admitted && r.quantiles.is_empty() && r.unit.as_deref() == Some("m"));
}

fn track(v: Value) -> Track {
    serde_json::from_value(v).expect("track")
}

#[test]
fn the_flow_speed_and_the_width_class_follow_the_reading_rules() {
    let cal = [(38.0, 0.0), (46.5, 5.033), (55.0, 8.556)];
    // a calibrated value wins
    assert_eq!(front_tools::flow_speed(&cal, 50.0, Some(6.0)), Some(6.0));
    // linear through the rows, and extended above the last
    let u = front_tools::flow_speed(&cal, 50.0, None).unwrap();
    assert!((u - (5.033 + 3.5 * (8.556 - 5.033) / 8.5)).abs() < 1e-12);
    let u = front_tools::flow_speed(&cal, 60.0, None).unwrap();
    assert!((u - (8.556 + 5.0 * (8.556 - 5.033) / 8.5)).abs() < 1e-12);
    // a block without a zero-flow row gets one at the stall display
    let no_zero = [(46.5, 5.033), (55.0, 8.556)];
    let u = front_tools::flow_speed(&no_zero, 42.0, None).unwrap();
    assert!((u - 4.0 * 5.033 / (46.5 - rules::FAN_ZERO)).abs() < 1e-12);
    assert_eq!(front_tools::flow_speed(&no_zero, 37.0, None), Some(0.0));
    // no rows: no speed
    assert_eq!(front_tools::flow_speed(&[], 50.0, None), None);

    let base = |label: &str, band: Option<f64>| {
        json!({
            "schema": "wsp-bass-front-track/1",
            "run_id": "synthetic",
            "partition": null,
            "position_unit": "sample_width",
            "conditions": {"x_o2_initial": 18.8, "sample_label": label},
            "geometry": {"band_width_px": band, "window_length_w": null},
            "fan": [{"t_s": -30.0, "fan": 50, "u_cm_s": null}, {"t_s": 10.0, "fan": 50, "u_cm_s": null},
                    {"t_s": 20.0, "fan": 42, "u_cm_s": 2.0}],
            "fan_calibration": [[46.5, 5.033], [55, 8.556]],
            "stills": [{"t_s": -1.0, "tip_w": null, "base_w": null, "state": "igniter", "quality": null},
                       {"t_s": 1.0, "tip_w": 0.5, "base_w": 0.1, "state": "flame", "quality": "ok"},
                       {"t_s": 2.0, "tip_w": 0.6, "base_w": null, "state": "flame", "quality": "edge"}],
            "end": {"kind": "extinction", "t_s": null},
        })
    };
    // the measured band decides the width class, over the label
    let t = track(base("2 cm SIBAL", Some(31.0)));
    let (x_o2, ratio, window) = front_tools::conditions_of(&t).unwrap();
    assert!(
        (x_o2 - 0.188).abs() < 1e-12,
        "a percentage is read as a fraction"
    );
    assert_eq!(ratio, rules::NARROW_RATIO);
    assert!((window - rules::WINDOW_REF_W / rules::NARROW_RATIO).abs() < 1e-12);
    let t = track(base("1 cm SIBAL", Some(56.0)));
    assert_eq!(front_tools::conditions_of(&t).unwrap().1, 1.0);
    // without a band, the label
    let t = track(base("1 cm SIBAL", None));
    assert_eq!(
        front_tools::conditions_of(&t).unwrap().1,
        rules::NARROW_RATIO
    );
    assert!(front_tools::conditions_of(&track(base("2 cm Nomex", None))).is_err());
    // a width stated in metres decides the ratio, over band and label, by the forwards' reference
    let mut wide = base("SIBAL fabric 40.6 x 94 cm", Some(56.0));
    wide["geometry"]["sample_width_m"] = json!(0.406);
    wide["geometry"]["window_length_w"] = json!(94.0 / 40.6);
    let (_, ratio, window) = front_tools::conditions_of(&track(wide.clone())).unwrap();
    assert!((ratio - 0.406 / rules::REF_WIDTH_M).abs() < 1e-12);
    assert!((window - 94.0 / 40.6).abs() < 1e-12);
    // an opposed-flow sheet is refused, not read as concurrent
    wide["conditions"]["flow"] = json!("opposed");
    assert!(front_tools::conditions_of(&track(wide)).is_err());
    // the fan history as a step function, repeated speeds merged; stills with null fields read
    let flow = front_tools::flow_of(&t).unwrap();
    assert_eq!(flow.len(), 2);
    assert_eq!(flow[1], (20.0, 2.0));
    assert_eq!(front_tools::flame_stills(&t), vec![(1.0, 0.5, 0.1)]);
}

#[test]
fn the_committed_spec_is_the_spec_the_forward_was_compiled_with() {
    let spec = front::spec();
    spec.validate().expect("spec valid");
    assert_eq!(
        models::by_key("bass_front").unwrap().spec().spec_hash(),
        spec.spec_hash()
    );
    let path = format!(
        "{}/../../configs/models/bass/front/spec.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let on_disk: ModelSpec =
        serde_json::from_str(&std::fs::read_to_string(&path).expect(&path)).unwrap();
    assert_eq!(
        on_disk.spec_hash(),
        spec.spec_hash(),
        "{path} drifted from front::spec(); a changed spec is a new spec id (re-export with \
         `ignis wsp spec --family bass_front --out …`), never an in-place edit"
    );
    // the toy weights assemble into a release whose build checks all pass
    let r = release("");
    let checks = &r.release.construction.as_ref().unwrap().checks;
    assert!(
        checks.iter().all(|c| c.status == CheckStatus::Passed),
        "{checks:?}"
    );
}

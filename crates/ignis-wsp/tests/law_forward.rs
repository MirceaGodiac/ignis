//! The generic law forward (`law.rs`, models `<slug>.wsp`), on synthetic declarations and
//! test-only weights. No NASA outcome is read or needed.
//!
//! * every registered declaration parses, its spec validates, its model is found by key and name,
//!   no key or name collides with another literal model, its forward identity is its own;
//! * the neutral root generated from the spec compiles, assembles, and forecasts the same thing
//!   whatever the inputs; a placeholder declaration cannot be assembled;
//! * each target kind reproduces a hand computation (the committed example of
//!   `docs/wsp/research/law_forward.md`); hinge units, transforms, refusals and bounds behave as
//!   declared; competing risks sum to one, reduce to one clock with one cause, and agree with a fine
//!   quadrature; the scheduled stop takes the active mass.

mod common;

use ignis_core::contracts::{ControlStage, Quantity, ReleaseId, SupportGate};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::law::{self, Clock};
use ignis_wsp::law_slugs::{self, SLUGS};
use ignis_wsp::law_tools;
use ignis_wsp::models::{self, Forward, MODELS};
use ignis_wsp::quad::normal_cdf;
use ignis_wsp::{assemble, audit, text};
use serde_json::{json, Value};

const EXAMPLE: &str = include_str!("../../../configs/models/law/synthetic_demo/example/edits.wsp");
const DEMO: &str = "synthetic_demo";

fn loaded(mut release: ignis_core::contracts::ModelRelease, params: TensorMap) -> LoadedRelease {
    release.id = content_hash(&release, &["id", "created_at"]);
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

fn root(key: &str) -> anyhow::Result<LoadedRelease> {
    let model = models::by_key(key)?;
    let spec = model.spec();
    let src = law_tools::root_edits(&spec);
    let ops = text::compile(&src, &spec)?;
    let patch = assemble::make_patch(&spec, law_tools::root_meta(&spec), ops);
    let tensors = assemble::materialize(&spec, &patch, None)?;
    let release = model.build_release(&patch, &tensors, None, &common::binding())?;
    Ok(loaded(release, tensors))
}

fn revise(key: &str, parent: &LoadedRelease, src: &str) -> anyhow::Result<LoadedRelease> {
    let model = models::by_key(key)?;
    let spec = model.spec();
    let mut meta = common::meta("test-law-revision");
    meta.parent_release_id = Some(ReleaseId::new(parent.release.id.clone()).unwrap());
    let ops = text::compile(src, &spec)?;
    let patch = assemble::make_patch(&spec, meta, ops);
    let tensors = assemble::materialize(&spec, &patch, Some(parent))?;
    let release = model.build_release(&patch, &tensors, Some(parent), &common::binding())?;
    Ok(loaded(release, tensors))
}

fn demo_index() -> usize {
    law_slugs::by_key(DEMO).unwrap()
}

/// A synthetic prefix of the demo: pressure (Pa), x_o2, fuel_flow (kg/s).
fn prefix(p: f64, o2: f64, mdot: f64) -> ignis_core::contracts::PrefixBundle {
    law_tools::prefix_with(demo_index(), &[p, o2, mdot], "law-test")
}

fn run(
    rel: &LoadedRelease,
    prefix: &ignis_core::contracts::PrefixBundle,
    settings: Value,
) -> ignis_core::contracts::AnalysisArtifact {
    let a = models::by_key(DEMO)
        .unwrap()
        .artifact(rel, prefix, &settings)
        .unwrap_or_else(|r| panic!("not runnable: {r}"));
    a.validate().expect("valid artifact");
    a
}

fn admitted(
    rel: &LoadedRelease,
    prefix: &ignis_core::contracts::PrefixBundle,
    settings: Value,
) -> ignis_core::contracts::AnalysisArtifact {
    let a = run(rel, prefix, settings);
    assert!(a.support.admitted, "refused: {:?}", a.support.reasons);
    a
}

fn example() -> LoadedRelease {
    revise(DEMO, &root(DEMO).unwrap(), EXAMPLE).expect("example revision")
}

fn close(a: f64, b: f64, rel: f64, what: &str) {
    assert!(
        (a - b).abs() <= rel * b.abs().max(1e-300),
        "{what}: {a} against {b}"
    );
}

/// Standard normal quantiles at the default levels (tables; independent of the forward's routine).
const Z: [f64; 5] = [
    -1.6448536269514722,
    -0.6744897501960817,
    0.0,
    0.6744897501960817,
    1.6448536269514722,
];

#[test]
fn every_declaration_parses_and_its_spec_validates() {
    let existing_keys: Vec<&str> = MODELS.iter().map(|m| m.key).collect();
    let existing_names: Vec<&str> = MODELS.iter().map(|m| m.model_name).collect();
    for (i, s) in SLUGS.iter().enumerate() {
        let spec = law::spec(i);
        spec.validate()
            .unwrap_or_else(|e| panic!("{}: {e}", s.slug));
        assert_eq!(spec.model_name, s.model_name);
        assert!(
            !existing_keys.contains(&s.slug),
            "{} collides with a model key",
            s.slug
        );
        assert!(
            !existing_names.contains(&s.model_name),
            "{} collides",
            s.model_name
        );
        let m = models::by_key(s.slug).unwrap();
        assert_eq!(m.forward, Forward::Law(i));
        assert_eq!(models::by_name(s.model_name).unwrap(), m);
        let sources = audit::forward_sources_of(s.model_name).unwrap();
        assert!(sources.iter().any(|(n, _)| *n == "law.rs"));
        assert!(sources
            .iter()
            .any(|(n, t)| *n == s.decl_path && *t == s.decl));
        assert_eq!(
            audit::strict_route_check_of(s.model_name).status,
            ignis_core::contracts::CheckStatus::Passed
        );
    }
    // the ten investigations of the brief are registered
    for slug in [
        "sflame_event",
        "bre",
        "cld",
        "efield",
        "slice",
        "spice",
        "samer",
        "same",
        "daft",
        "bass1",
    ] {
        assert!(law_slugs::by_key(slug).is_some(), "{slug}");
    }
    // each slug has its own identity: the declaration is a forward source
    assert_ne!(
        audit::forward_hash_of("bre.wsp"),
        audit::forward_hash_of("cld.wsp")
    );
    assert_ne!(
        audit::forward_hash_of("bre.wsp"),
        audit::forward_hash_of("synthetic_demo.wsp")
    );
}

#[test]
fn the_demo_spec_has_the_documented_tensors() {
    let spec = law::spec(demo_index());
    let t = |n: &str| spec.tensor(n).unwrap_or_else(|| panic!("{n} missing"));
    assert_eq!(t("x.pressure.centre").shape, Vec::<u32>::new());
    assert_eq!(t("flame_length.median.skip").shape, vec![3]);
    assert_eq!(t("flame_length.median.W1").shape, vec![8, 3]);
    assert_eq!(t("flame_length.spread.w2").shape, vec![8]);
    assert_eq!(
        t("flame_end.extinction.median.b0").unit.as_deref(),
        Some("ln(s)")
    );
    assert_eq!(t("flame_end.stop.offset_w").shape, vec![3]);
    assert!(
        spec.tensor("burnout.stop.offset_s").is_none(),
        "no stop declared for burnout"
    );
    assert_eq!(t("x.fuel_flow.scale").unit.as_deref(), Some("1/(kg/s)"));
    assert_eq!(spec.features.len(), 3);
    assert_eq!(spec.features[0].name, "pressure");
    // the first target with a shared quantity names the spec's target
    assert_eq!(spec.target.quantity, Quantity::SpreadRate);
    // a placeholder (all lane-local) says why it names phase_end_time
    let bre = law::spec(law_slugs::by_key("bre").unwrap());
    assert_eq!(bre.target.quantity, Quantity::PhaseEndTime);
    assert!(bre.target.excludes[0].contains("only because ModelSpec.target requires"));
    assert!(bre
        .target
        .excludes
        .iter()
        .any(|e| e.starts_with("bre:outcome")));
}

#[test]
fn a_placeholder_cannot_be_assembled() {
    for slug in ["bre", "sflame_event", "bass1"] {
        let e = root(slug).err().expect("placeholder must not assemble");
        assert!(
            format!("{e:#}").contains("failed build check")
                || format!("{e:#}").contains("placeholder"),
            "{slug}: {e:#}"
        );
    }
}

#[test]
fn the_neutral_root_is_neutral() {
    let r = root(DEMO).expect("root");
    let inputs = [
        (101_325.0, 0.21, 1e-6),
        (5_000.0, 0.9, 3e-3),
        (2.0e6, 0.0, -1.0),
    ];
    let mut seen: Vec<Value> = Vec::new();
    for (p, o2, m) in inputs {
        let pf = prefix(p, o2, m);
        let fl = admitted(&r, &pf, json!({"target": "flame_length"}));
        // median 1 m, ln-sd 10 (the spread's upper bound), no tail
        assert_eq!(fl.quantile_levels, vec![0.05, 0.25, 0.5, 0.75, 0.95]);
        assert!((fl.quantiles[2][0] - 1.0).abs() < 1e-15);
        for (l, z) in Z.iter().enumerate() {
            close(fl.quantiles[l][0], (10.0 * z).exp(), 1e-7, "root quantile");
        }
        let ig = admitted(&r, &pf, json!({"target": "ignites"}));
        assert_eq!(ig.event_cdf, vec![vec![0.5], vec![0.5]]);
        let fe = admitted(
            &r,
            &pf,
            json!({"target": "flame_end", "times_s": [0.5, 1.0, 2.0]}),
        );
        // two clocks of median 1 s and shape 0.1, no stop
        let s = |t: f64| 1.0 / (1.0 + t.powf(0.1)).powi(2);
        for (j, t) in [0.5, 1.0, 2.0].iter().enumerate() {
            close(fe.event_cdf[0][j], s(*t), 1e-12, "root active");
            close(
                fe.event_cdf[1][j],
                fe.event_cdf[2][j],
                1e-12,
                "root causes equal",
            );
            assert_eq!(fe.event_cdf[3][j], 0.0);
        }
        seen.push(json!([fl.quantiles, ig.event_cdf, fe.event_cdf]));
    }
    assert!(
        seen.windows(2).all(|w| w[0] == w[1]),
        "the root depends on an input"
    );
    // the root's edits name every authored tensor once and nothing else
    let spec = law::spec(demo_index());
    let src = law_tools::root_edits(&spec);
    assert!(src.contains("flame_length.spread.b0 = 10.0"));
    assert!(src.contains("x.pressure.lo = -1e30"));
    assert!(!src.contains(".W1"), "no hinge in the root");
}

/// The example's normalised features, by hand.
fn z_of(p: f64, o2: f64, mdot: f64) -> [f64; 3] {
    [
        (p / 101_325.0).ln() / 1.0,
        (o2 - 0.21) / 0.1,
        (1.0 / mdot - 1.0 / 0.000001) / 1_000_000.0,
    ]
}

fn relu(x: f64) -> f64 {
    x.max(0.0)
}

#[test]
fn transforms_and_features_are_reported_as_declared() {
    let rel = example();
    let a = admitted(
        &rel,
        &prefix(50_662.5, 0.35, 5e-7),
        json!({"target": "ignites"}),
    );
    let z = z_of(50_662.5, 0.35, 5e-7);
    let got: Vec<f64> = a.diagnostics["features"]["z"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    for k in 0..3 {
        close(got[k], z[k], 1e-14, "z");
    }
    assert_eq!(
        a.diagnostics["features"]["names"],
        json!(["pressure", "x_o2", "fuel_flow"])
    );
}

#[test]
fn a_positive_target_reproduces_the_hand_computation() {
    let rel = example();
    for (p, o2, mdot) in [
        (50_662.5, 0.35, 5e-7),
        (101_325.0, 0.30, 1e-6),
        (150_000.0, 0.25, 2e-6),
    ] {
        let z = z_of(p, o2, mdot);
        // median: b0 + skip . z + w2[0] relu(b1[0] + W1[0, 1] z1): the hinge opens at x_o2 = 0.31
        let m = -3.9 + (-0.5 * z[0] + 0.2 * z[1] + 0.0 * z[2]) + 0.3 * relu(-1.0 + 1.0 * z[1]);
        let (sd, w, k) = (0.25, 0.1, 3.0);
        let a = admitted(
            &rel,
            &prefix(p, o2, mdot),
            json!({"target": "flame_length"}),
        );
        assert_eq!(a.quantity, None, "flame length is lane-local");
        assert_eq!(a.unit, None);
        assert_eq!(a.diagnostics["law"]["label"], "synthetic_demo:flame_length");
        assert_eq!(a.diagnostics["law"]["unit"], "m");
        assert_eq!(a.diagnostics["law"]["shared_quantity"], Value::Null);
        assert_eq!(a.times_s, vec![0.0]);
        close(
            a.diagnostics["outcome"]["ln_median"].as_f64().unwrap(),
            m,
            1e-13,
            "ln median",
        );
        close(a.quantiles[2][0], m.exp(), 1e-12, "median");
        // every quantile solves the mixture CDF at its level
        for (l, level) in a.quantile_levels.iter().enumerate() {
            let q = (a.quantiles[l][0]).ln() - m;
            let f = (1.0 - w) * normal_cdf(q / sd) + w * normal_cdf(q / (k * sd));
            assert!((f - level).abs() < 1e-12, "level {level}: CDF {f}");
            // and lies between the core and the wide component's quantiles
            let (c1, c2) = (sd * Z[l], k * sd * Z[l]);
            assert!(q >= c1.min(c2) - 1e-12 && q <= c1.max(c2) + 1e-12);
        }
        let mean =
            m.exp() * ((1.0 - w) * (0.5 * sd * sd).exp() + w * (0.5 * k * k * sd * sd).exp());
        close(a.mean[0], mean, 1e-12, "mean");
    }
    // the hinge: below x_o2 = 0.31 the slope in z1 is 0.2, above it 0.5
    let at = |o2: f64| {
        admitted(
            &rel,
            &prefix(101_325.0, o2, 1e-6),
            json!({"target": "flame_length"}),
        )
        .diagnostics["outcome"]["ln_median"]
            .as_f64()
            .unwrap()
    };
    close(
        at(0.30) - at(0.25),
        0.2 * 0.5,
        1e-12,
        "slope below the hinge",
    );
    close(
        at(0.40) - at(0.35),
        0.5 * 0.5,
        1e-12,
        "slope above the hinge",
    );
}

#[test]
fn one_log_normal_is_closed_form_and_custom_levels_are_honoured() {
    let rel = revise(DEMO, &example(), "spread.median.b0 = -2.0\nspread.spread.b0 = 0.4\nspread.median.skip[0:3] = [0.0, 0.0, 0.0]\nspread.spread.skip[0:3] = [0.1, 0.0, 0.0]\n").unwrap();
    let pf = prefix(50_662.5, 0.21, 1e-6);
    let a = admitted(
        &rel,
        &pf,
        json!({"target": "spread", "quantile_levels": [0.1, 0.5, 0.9]}),
    );
    assert_eq!(a.quantity, Some(Quantity::SpreadRate), "a shared quantity");
    assert_eq!(a.unit.as_deref(), Some("m/s"));
    let sd = 0.4 + 0.1 * (0.5_f64).ln();
    let z90 = 1.2815515655446004;
    close(a.quantiles[0][0], (-2.0 - sd * z90).exp(), 1e-8, "q10");
    close(a.quantiles[1][0], (-2.0_f64).exp(), 1e-14, "q50");
    close(a.quantiles[2][0], (-2.0 + sd * z90).exp(), 1e-8, "q90");
}

#[test]
fn a_binary_target_is_the_logistic_of_its_network() {
    let rel = example();
    for o2 in [0.15, 0.21, 0.33] {
        let z = z_of(101_325.0, o2, 1e-6);
        let p = 1.0 / (1.0 + (-(1.5 + 2.0 * z[1])).exp());
        let a = admitted(
            &rel,
            &prefix(101_325.0, o2, 1e-6),
            json!({"target": "ignites"}),
        );
        assert_eq!(a.event_categories, vec!["yes", "no"]);
        close(a.event_cdf[0][0], p, 1e-14, "P(yes)");
        close(a.event_cdf[0][0] + a.event_cdf[1][0], 1.0, 1e-15, "sum");
    }
}

fn s_ll(t: f64, tau: f64, k: f64) -> f64 {
    1.0 / (1.0 + (t / tau).powf(k))
}

#[test]
fn one_cause_is_one_clock_exactly() {
    let rel = revise(
        DEMO,
        &example(),
        "burnout.burnout.median.b0 = 3.0\nburnout.burnout.median.skip[0:3] = [0.0, 0.0, 1.0]\nburnout.burnout.shape = 2.5\n",
    )
    .unwrap();
    for (mdot, cutoff) in [(1e-6, 0.0), (5e-7, 12.0)] {
        let mut pf = prefix(101_325.0, 0.21, mdot);
        pf.cutoff_s = cutoff;
        let times = [
            cutoff,
            cutoff + 3.0,
            cutoff + 20.0,
            cutoff + 60.0,
            cutoff + 500.0,
        ];
        let a = admitted(&rel, &pf, json!({"target": "burnout", "times_s": times}));
        assert_eq!(
            a.event_categories,
            vec!["active", "burnout", "scheduled_stop"]
        );
        let tau = (3.0 + z_of(101_325.0, 0.21, mdot)[2]).exp();
        for (j, t) in times.iter().enumerate() {
            let s = s_ll(*t, tau, 2.5) / s_ll(cutoff, tau, 2.5);
            close(a.event_cdf[0][j], s, 1e-12, "active");
            assert!(
                (a.event_cdf[1][j] - (1.0 - s)).abs() < 1e-12,
                "CIF {} against {}",
                a.event_cdf[1][j],
                1.0 - s
            );
            assert_eq!(a.event_cdf[2][j], 0.0);
        }
    }
}

#[test]
fn competing_risks_sum_to_one_and_agree_with_a_fine_quadrature() {
    let rel = example();
    let mut pf = prefix(101_325.0, 0.30, 1e-6);
    pf.cutoff_s = 5.0;
    let times: Vec<f64> = vec![5.0, 10.0, 40.0, 90.0, 200.0, 600.0, 3000.0];
    let a = admitted(&rel, &pf, json!({"target": "flame_end", "times_s": times}));
    assert_eq!(
        a.event_categories,
        vec!["active", "extinction", "blowoff", "scheduled_stop"]
    );
    let z = z_of(101_325.0, 0.30, 1e-6);
    let c1 = (4.5 + 0.8 * z[1], 3.0);
    let c2 = (6.0 - 0.4 * z[2], 2.0);
    let h = |t: f64, (l, k): (f64, f64)| {
        let r = (t / l.exp()).powf(k);
        (k / t) * r / (1.0 + r)
    };
    let s = |t: f64| s_ll(t, c1.0.exp(), c1.1) * s_ll(t, c2.0.exp(), c2.1);
    for (j, t) in times.iter().enumerate() {
        let total: f64 = (0..4).map(|r| a.event_cdf[r][j]).sum();
        assert!((total - 1.0).abs() < 1e-12, "mass {total}");
        close(a.event_cdf[0][j], s(*t) / s(5.0), 1e-12, "active");
        // Simpson on [5, t] with 200 000 intervals
        if *t > 5.0 {
            let n = 200_000;
            let dx = (t - 5.0) / n as f64;
            let mut acc = 0.0;
            for i in 0..=n {
                let u = 5.0 + i as f64 * dx;
                let wgt = if i == 0 || i == n {
                    1.0
                } else if i % 2 == 1 {
                    4.0
                } else {
                    2.0
                };
                acc += wgt * h(u, c1) * s(u);
            }
            let cif = acc * dx / 3.0 / s(5.0);
            assert!(
                (a.event_cdf[1][j] - cif).abs() < 2e-5,
                "t {t}: {} against {cif}",
                a.event_cdf[1][j]
            );
        }
        if j > 0 {
            for r in 1..3 {
                assert!(
                    a.event_cdf[r][j] >= a.event_cdf[r][j - 1],
                    "a cumulative incidence decreased"
                );
            }
        }
    }
    // two identical causes split the lost mass equally
    let twin = revise(
        DEMO,
        &root(DEMO).unwrap(),
        "flame_end.extinction.median.b0 = 3.0\nflame_end.blowoff.median.b0 = 3.0\nflame_end.extinction.shape = 1.7\nflame_end.blowoff.shape = 1.7\n",
    )
    .unwrap();
    let b = admitted(
        &twin,
        &prefix(1.0, 0.0, 1.0),
        json!({"target": "flame_end", "times_s": [1.0, 20.0, 100.0]}),
    );
    for j in 0..3 {
        close(b.event_cdf[1][j], b.event_cdf[2][j], 1e-12, "twins");
        close(
            b.event_cdf[1][j],
            0.5 * (1.0 - b.event_cdf[0][j]),
            1e-12,
            "half each",
        );
    }
}

#[test]
fn the_scheduled_stop_takes_the_active_mass() {
    let rel = example();
    let idx = demo_index();
    let times = vec![10.0, 49.0, 50.0, 80.0];
    // a control stage
    let mut pf = prefix(101_325.0, 0.30, 1e-6);
    pf.known_controls.push(ControlStage {
        t_start_s: 50.0,
        t_end_s: None,
        control: "scheduled_stop".into(),
        value: None,
        operator_action: false,
    });
    let a = admitted(&rel, &pf, json!({"target": "flame_end", "times_s": times}));
    let z = z_of(101_325.0, 0.30, 1e-6);
    let clocks = [
        Clock {
            ln_tau: 4.5 + 0.8 * z[1],
            shape: 3.0,
        },
        Clock {
            ln_tau: 6.0 - 0.4 * z[2],
            shape: 2.0,
        },
    ];
    let s50 = (law::ln_survival(&clocks, 50.0)).exp();
    for j in 2..4 {
        assert_eq!(a.event_cdf[0][j], 0.0, "nothing active after the stop");
        close(a.event_cdf[3][j], s50, 1e-12, "the stop takes S(stop)");
        assert_eq!(
            a.event_cdf[1][j], a.event_cdf[1][2],
            "causes frozen after the stop"
        );
    }
    assert_eq!(a.event_cdf[3][1], 0.0);
    // the declared condition with two offset nodes of weight 0.5: the mean of two single-stop tables
    let mut pf = prefix(101_325.0, 0.30, 1e-6);
    let mut m = pf.known_conditions["x_o2"].clone();
    m.value_si = Some(40.0);
    m.unit_si = Some("s".into());
    pf.known_conditions.insert("programmed_stop".into(), m);
    let b = admitted(&rel, &pf, json!({"target": "flame_end", "times_s": times}));
    assert_eq!(b.diagnostics["outcome"]["stop_condition_read"], true);
    let t1 = law::event_table(&clocks, 0.0, Some(40.0), &times, 512);
    let t2 = law::event_table(&clocks, 0.0, Some(45.0), &times, 512);
    for r in 0..4 {
        for j in 0..4 {
            close(
                b.event_cdf[r][j] + 1e-300,
                0.5 * (t1[r][j] + t2[r][j]) + 1e-300,
                1e-12,
                "node mixture",
            );
        }
    }
    // a stop at or before the cutoff is refused
    let mut late = pf.clone();
    late.cutoff_s = 41.0;
    let c = run(
        &rel,
        &late,
        json!({"target": "flame_end", "times_s": [50.0]}),
    );
    assert!(!c.support.admitted);
    assert!(c.support.reasons[0].contains("at or before the cutoff"));
    let _ = idx;
}

#[test]
fn refusals_name_the_feature_and_keep_the_target() {
    let rel = example();
    // missing condition
    let mut pf = prefix(101_325.0, 0.21, 1e-6);
    pf.known_conditions.remove("x_o2");
    let a = run(&rel, &pf, json!({"target": "flame_length"}));
    assert!(!a.support.admitted);
    assert_eq!(a.support.gate, SupportGate::Condition);
    assert!(
        a.support.reasons[0].starts_with("feature x_o2:"),
        "{:?}",
        a.support.reasons
    );
    assert!(a.quantiles.is_empty() && a.event_cdf.is_empty());
    assert_eq!(a.diagnostics["law"]["label"], "synthetic_demo:flame_length");
    // outside the declared domain, at both ends; the ends themselves are inside
    for (p, inside) in [
        (19_999.0, false),
        (20_000.0, true),
        (200_000.0, true),
        (200_001.0, false),
    ] {
        let a = run(&rel, &prefix(p, 0.21, 1e-6), json!({"target": "ignites"}));
        assert_eq!(a.support.admitted, inside, "p = {p}");
        if !inside {
            assert!(
                a.support.reasons[0].contains("feature pressure")
                    && a.support.reasons[0].contains("outside its declared domain")
            );
        }
    }
    // the transform's own domain, on the root (whose declared domain is the tensor bounds)
    let r = root(DEMO).unwrap();
    let a = run(&r, &prefix(-5.0, 0.21, 1e-6), json!({"target": "ignites"}));
    assert!(
        a.support.reasons[0].contains("feature pressure")
            && a.support.reasons[0].contains("transform ln")
    );
    let a = run(&r, &prefix(1.0, 0.21, 0.0), json!({"target": "ignites"}));
    assert!(
        a.support.reasons[0].contains("feature fuel_flow")
            && a.support.reasons[0].contains("reciprocal")
    );
    // an unknown target, an event without times, a prefix of another family
    let pf = prefix(101_325.0, 0.21, 1e-6);
    let a = run(&rel, &pf, json!({"target": "smoke_point"}));
    assert_eq!(a.support.gate, SupportGate::Semantic);
    assert!(a.support.reasons[0].contains("smoke_point"));
    let a = run(&rel, &pf, json!({"target": "flame_end"}));
    assert_eq!(a.support.gate, SupportGate::Horizon);
    let mut other = pf.clone();
    other.family = ignis_core::contracts::Family::Bass;
    assert!(!run(&rel, &other, json!({})).support.admitted);
    // a spread that is not positive at the conditions is refused, not floored
    let neg = revise(DEMO, &example(), "spread.median.b0 = 0.0\nspread.spread.b0 = 0.1\nspread.median.skip[0:3] = [0.0, 0.0, 0.0]\nspread.spread.skip[0:3] = [0.0, -1.0, 0.0]\n").unwrap();
    assert!(
        admitted(
            &neg,
            &prefix(101_325.0, 0.21, 1e-6),
            json!({"target": "spread"})
        )
        .quantiles
        .len()
            == 5
    );
    let a = run(
        &neg,
        &prefix(101_325.0, 0.35, 1e-6),
        json!({"target": "spread"}),
    );
    assert_eq!(a.support.gate, SupportGate::Numerical);
    assert!(a.support.reasons[0].contains("target spread"));
}

#[test]
fn bounds_are_enforced_on_the_patch() {
    let spec = law::spec(demo_index());
    let mut meta = common::meta("test-law-bounds");
    meta.parent_release_id = Some(ReleaseId::new(root(DEMO).unwrap().release.id.clone()).unwrap());
    for bad in [
        "flame_length.tail_weight = 1.5",
        "flame_length.tail_width = 0.5",
        "flame_length.spread.b0 = 0.0",
        "flame_end.extinction.shape = 0.05",
        "ignites.logit.b0 = 41.0",
        "x.pressure.centre = 0.0",
        "x.x_o2.scale = 0.0",
        "flame_end.stop.offset_w[0] = 2.0",
        "burnout.stop.offset_s[0] = 1.0",
        "num.substeps = 10.0",
    ] {
        let ok = text::compile(bad, &spec)
            .map_err(anyhow::Error::from)
            .and_then(|ops| {
                Ok(assemble::make_patch(&spec, meta.clone(), ops).validate_against(&spec)?)
            });
        assert!(ok.is_err(), "{bad} was accepted");
    }
}

#[test]
fn a_release_built_for_one_slug_is_refused_by_another() {
    let rel = example();
    let other = models::by_key("bre").unwrap();
    let e = other
        .artifact(&rel, &prefix(101_325.0, 0.21, 1e-6), &Value::Null)
        .err();
    assert!(e.is_some(), "spec mismatch must refuse");
}

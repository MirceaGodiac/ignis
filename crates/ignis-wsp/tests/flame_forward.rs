//! The ACME clock forward on synthetic prefixes with test-only weights: the clocks reproduce their
//! closed forms, the floor holds, a programmed stop takes the written share, the latent classes are
//! re-weighted by survival, the rows partition, and the configurations stay apart.

mod common;

use ignis_core::contracts::{Family, PrefixBundle};
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::flame::{self, Forecast};
use ignis_wsp::{acme_tools, assemble, models, text};
use serde_json::{json, Value};

/// One self-extinction clock (median 30 s, shape 2); no weak class, no burner limit, no programmed
/// stop, no background hazard. Not a candidate.
const TOY: &str = "
se.tau.b0 = 3.4011973816621555
se.tau.skip[0:8] = 0.0
se.flash.b0 = -40.0
se.flash.skip[0:8] = 0.0
ft.tau.b0 = 13.9
ft.tau.skip[0:8] = 0.0
se.floor_s = 0.0
se.shape = 2.0
se.flash_tau_s = 1.0
se.flash_shape = 4.0
ft.shape = 2.0
ft.atom_t_s[0:8] = 0.0
ft.atom_logit[0:8] = -40.0
ft.log_base = -40.0
";

fn key(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse2",
        _ => "acme_normal2",
    }
}

/// Toy weights with `extra` statements replacing the base statements of the tensors they name.
fn release(family: Family, extra: &str) -> LoadedRelease {
    let model = models::by_key(key(family)).expect("model");
    let spec = model.spec();
    let mut src = String::new();
    for line in TOY.lines() {
        let name = line.split(['[', ' ', '=']).next().unwrap_or_default();
        let replaced = !name.is_empty()
            && extra.lines().any(|l| {
                l.trim_start()
                    .split(['[', ' ', '='])
                    .next()
                    .is_some_and(|n| n == name)
            });
        if !replaced {
            src.push_str(line);
            src.push('\n');
        }
    }
    src.push_str(extra);
    let ops = text::compile(&src, &spec).unwrap_or_else(|e| panic!("{e}\n{src}"));
    let patch = assemble::make_patch(&spec, common::meta("test-flame"), ops);
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

/// A normal-flame prefix at the reference state of every feature (c = 0).
fn reference(cutoff: f64) -> PrefixBundle {
    acme_tools::fixture_prefix(
        Family::AcmeNormal,
        cutoff,
        101_325.0,
        1.0e-6,
        0.30,
        0.30,
        2200.0,
        0.25,
        "propane",
    )
}

fn run(extra: &str, p: &PrefixBundle, times: &[f64]) -> Forecast {
    let r = release(Family::AcmeNormal, extra);
    let spec = flame::spec(Family::AcmeNormal);
    flame::forecast(&spec, &r, p, &json!({ "times_s": times }))
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn survival(t: f64, tau: f64, k: f64) -> f64 {
    1.0 / (1.0 + (t / tau).powf(k))
}

#[test]
fn a_single_clock_reproduces_the_log_logistic_distribution() {
    let times = [5.0, 15.0, 30.0, 60.0, 120.0];
    let f = run("", &reference(1.0), &times);
    for (j, t) in times.iter().enumerate() {
        let want = 1.0 - survival(*t, 30.0, 2.0) / survival(1.0, 30.0, 2.0);
        assert!(
            (f.table[1][j] - want).abs() < 2e-4,
            "t = {t}: {} vs {want}",
            f.table[1][j]
        );
        assert!(f.table[2][j] < 1e-6, "no flow termination was written");
        let sum: f64 = (0..4).map(|r| f.table[r][j]).sum();
        assert!((sum - 1.0).abs() < 1e-12);
    }
    // the features of the reference prefix are all zero, so the clock median is exp(b0)
    assert!(f.features.iter().all(|c| c.abs() < 1e-9));
    assert!((f.clocks.radiative_tau_s - 30.0).abs() < 1e-9);
}

#[test]
fn nothing_self_extinguishes_before_the_floor() {
    let times = [5.0, 9.9, 40.0];
    let f = run("se.floor_s = 10.0\n", &reference(1.0), &times);
    // (the weak class is switched off with a logit of -40, i.e. 4e-18, not with a hard zero)
    assert!(f.table[1][0] < 1e-12 && f.table[1][1] < 1e-12);
    // floor + median: half of the flames are out
    assert!((f.table[1][2] - 0.5).abs() < 2e-4, "{}", f.table[1][2]);
}

#[test]
fn a_programmed_stop_takes_its_written_share_of_the_flames_still_burning() {
    // no self-extinction at all; a stop at 20 s with probability 0.5
    let extra = "se.tau.b0 = 11.9\nft.atom_t_s[0:8] = [20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]\nft.atom_logit[0:8] = [0.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0]\n";
    let times = [19.0, 21.0, 100.0];
    let f = run(extra, &reference(1.0), &times);
    assert!(f.table[2][0] < 1e-6);
    assert!((f.table[2][1] - 0.5).abs() < 1e-5, "{}", f.table[2][1]);
    assert!((f.table[0][2] - 0.5).abs() < 1e-5);
    // the stop depends on the conditions through its row of ft.atom_w: twice the flow, logit -2
    let extra = format!("{extra}ft.atom_w[0, 1] = -2.8853900817779268\n");
    let mut p = acme_tools::fixture_prefix(
        Family::AcmeNormal,
        1.0,
        101_325.0,
        2.0e-6,
        0.30,
        0.30,
        2200.0,
        0.25,
        "propane",
    );
    p.input_hash = ignis_core::hashing::content_hash(&p, &["input_hash"]);
    let f = run(&extra, &p, &times);
    let want = 1.0 / (1.0 + 2.0_f64.exp());
    assert!(
        (f.table[2][1] - want).abs() < 1e-6,
        "{} vs {want}",
        f.table[2][1]
    );
}

#[test]
fn a_flame_alive_at_the_cutoff_is_rarely_of_the_weak_class() {
    // half of the flames are weak a priori and go out near 1 s
    let extra = "se.flash.b0 = 0.0\n";
    let f = run(extra, &reference(5.0), &[10.0, 30.0]);
    let (s_weak, s_main) = (survival(5.0, 1.0, 4.0), survival(5.0, 30.0, 2.0));
    let want = 0.5 * s_weak / (0.5 * s_weak + 0.5 * s_main);
    assert!(
        (f.p_weak_at_cutoff - want).abs() < 0.02 * want,
        "{} vs {want}",
        f.p_weak_at_cutoff
    );
    // at the time origin the prior stands
    let f0 = run(extra, &reference(0.0), &[10.0]);
    assert!((f0.p_weak_at_cutoff - 0.5).abs() < 1e-9);
    assert!(f0.table[1][0] > 0.5, "the weak half is gone by 10 s");
}

#[test]
fn the_burner_limit_competes_with_self_extinction() {
    // two clocks with the same median and shape share the lost mass evenly
    let extra = "ft.tau.b0 = 3.4011973816621555\n";
    let f = run(extra, &reference(0.0), &[30.0, 300.0]);
    for j in 0..2 {
        assert!((f.table[1][j] - f.table[2][j]).abs() < 1e-9);
    }
    // both clocks at their median: the flame survives each with probability one half
    assert!((f.table[0][0] - 0.25).abs() < 5e-4, "{}", f.table[0][0]);
    // incidences never decrease
    assert!(f.table[1][1] >= f.table[1][0] && f.table[2][1] >= f.table[2][0]);
}

#[test]
fn the_configurations_stay_apart() {
    let r = release(Family::AcmeNormal, "");
    let inverse = acme_tools::default_fixture(Family::AcmeInverse);
    let out = flame::forecast(&flame::spec(Family::AcmeNormal), &r, &inverse, &Value::Null)
        .expect("runnable");
    assert!(out.is_err_and(|s| s.reasons.iter().any(|m| m.contains("never pooled"))));
    // and an inverse release runs an inverse prefix
    let r = release(Family::AcmeInverse, "");
    let out = flame::forecast(
        &flame::spec(Family::AcmeInverse),
        &r,
        &inverse,
        &Value::Null,
    )
    .expect("runnable");
    assert!(out.is_ok());
}

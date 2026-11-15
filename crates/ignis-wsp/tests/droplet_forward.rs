//! The multistate droplet forward on synthetic prefixes with test-only weights: units, the two
//! extinction branches, the cool-flame stage, the validity gate and the state filter at the cutoff.

mod common;

use ignis_core::contracts::PrefixBundle;
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::droplet::{self, Forecast, Fuel};
use ignis_wsp::{assemble, flex_tools, models, text};
use serde_json::{json, Value};

const D0: f64 = 3.0e-3;

/// K_h = 0.5e-6 m2/s, no extinction of either kind, no evaporation, encoder off, no spread or noise.
const TOY: &str = "
rate.b0 = -0.6931471805599453
rate.skip[0:7] = 0.0
crit.b0 = 5.0
crit.skip[0:4] = 0.0
class.kappa = 20.0
class.bias = 0.0
diff.b0 = 12.0
diff.skip[0:6] = 0.0
diff.sharp = 40.0
rad.b0 = 0.0
rad.skip[0:6] = 0.0
rad.sharp = 4.0
post.k_inf = 0.0
post.k_hot = 0.0
post.tau0_s = 1.0
post.size_exp = 0.5
enc.gain_w[0:3] = 0.0
enc.gain_b = -50.0
enc.mix[0:2] = [1.0, 0.0]
enc.e_clip[0:2] = [1.0, 1.0]
filt.sd_evap = 0.2
filt.l_floor = 0.0
unc.s_pop = 0.0
unc.s_floor = 0.0
unc.sigma_d = 0.0
unc.sigma_frame = 0.0
haz.log_disruption = -20.0
";

/// No cool flame after a radiative extinction unless a test says so.
const TOY_COOL: &str = "
cool.p.b0 = -30.0
cool.p.skip[0:6] = 0.0
cool.k.b0 = -1.2039728043259361
cool.k.skip[0:5] = 0.0
cool.d.b0 = -0.6931471805599453
cool.d.skip[0:5] = 0.0
cool.d.sd = 0.0
post.cool_scale = 1.0
";

/// Toy weights with `extra` statements replacing the base statements of the tensors they name.
fn release(fuel: Fuel, extra: &str) -> LoadedRelease {
    let model = models::by_name(fuel.model_name()).expect("model");
    let spec = model.spec();
    let base = if fuel.has_cool() {
        format!("{TOY}{TOY_COOL}")
    } else {
        TOY.to_string()
    };
    let mut src = String::new();
    for line in base.lines() {
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
    let patch = assemble::make_patch(&spec, common::meta("test-droplet"), ops);
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

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

/// A D^2-law droplet of `fuel` burning at `k` in air, observed to `cutoff`.
fn prefix(fuel: Fuel, k: f64, cutoff: f64) -> PrefixBundle {
    let mut p = flex_tools::fixture_prefix(D0, k, cutoff, 0.21, 0.0, 0.0, 101_325.0);
    if let Some(f) = p.known_conditions.get_mut("fuel") {
        f.raw_value = Some(ignis_core::contracts::RawValue::Text(fuel.label().into()));
    }
    rehash(p)
}

fn run(fuel: Fuel, extra: &str, p: &PrefixBundle, settings: Value) -> Forecast {
    let r = release(fuel, extra);
    droplet::forecast(&droplet::spec(fuel), &r, p, &settings)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn median(f: &Forecast) -> &[f64] {
    let i = f.levels.iter().position(|p| *p == 0.5).expect("median");
    &f.quantiles[i]
}

#[test]
fn a_constant_rate_reproduces_the_d2_law_for_both_fuels() {
    for fuel in [Fuel::Methanol, Fuel::Heptane] {
        let f = run(
            fuel,
            "",
            &prefix(fuel, 0.5e-6, 3.0),
            json!({"horizon_s": 8.0, "n_points": 17}),
        );
        for (j, t) in f.times_s.iter().enumerate() {
            let want = (D0 * D0 - 0.5e-6 * t).sqrt();
            assert!(
                (median(&f)[j] - want).abs() < 2e-7 * want,
                "{fuel:?} t = {t}: {} vs {want}",
                median(&f)[j]
            );
            assert!(f
                .quantiles
                .iter()
                .all(|q| (q[j] - want).abs() < 2e-7 * want));
        }
        assert!(f.partition_max_error < 1e-9);
        assert!((f.encoded.state[0] - 1.0).abs() < 1e-9, "hot at the cutoff");
        assert_eq!(f.encoded.n_gated, 0);
    }
}

#[test]
fn a_diffusive_extinction_stops_the_droplet_at_the_written_progress() {
    // w_D = 0.5 with a sharp law: the flame goes out when D^2 has fallen to half, the droplet then
    // keeps that diameter (no evaporation in the toy weights)
    let f = run(
        Fuel::Methanol,
        "diff.b0 = 0.0\ndiff.sharp = 60.0\n",
        &prefix(Fuel::Methanol, 0.5e-6, 3.0),
        json!({"horizon_s": 12.0, "n_points": 25}),
    );
    let q_ext = 0.5 * D0 * D0;
    let t_ext = q_ext / 0.5e-6;
    for (j, t) in f.times_s.iter().enumerate() {
        if *t > t_ext + 1.0 {
            assert!(
                (median(&f)[j] * median(&f)[j] - q_ext).abs() < 0.03 * q_ext,
                "t = {t}"
            );
            assert!(f.state_probability[2][j] > 0.99);
        }
        if *t < t_ext - 1.0 {
            assert!(f.state_probability[0][j] > 0.99);
        }
    }
    // cumulative incidence of the end of the hot phase never decreases and ends near one
    assert!(f.events[1].windows(2).all(|w| w[1] >= w[0]));
    assert!(*f.events[1].last().unwrap() > 0.99);
}

#[test]
fn a_radiative_extinction_is_set_by_the_clock_and_a_cool_flame_follows_it() {
    // D_crit far below D0: every droplet is in the radiative class. s_R = 0.5: the hot flame goes
    // out at t = 0.5 * D0^2 / K_ref = 4.5 s. A cool flame always follows, burns at 0.3e-6 m2/s
    // and goes out at D_ce = 1 mm.
    let extra =
        "crit.b0 = -3.0\nrad.b0 = -0.6931471805599453\nrad.sharp = 30.0\ncool.p.b0 = 30.0\n";
    let f = run(
        Fuel::Heptane,
        extra,
        &prefix(Fuel::Heptane, 0.5e-6, 3.0),
        json!({"horizon_s": 30.0, "n_points": 121}),
    );
    let t_hot = 0.5 * D0 * D0 / 1.0e-6;
    let q_hot = D0 * D0 - 0.5e-6 * t_hot;
    let t_cool_end = t_hot + (q_hot - 1.0e-6) / 0.3e-6;
    for (j, t) in f.times_s.iter().enumerate() {
        let q = median(&f)[j] * median(&f)[j];
        if *t < t_hot - 0.6 {
            assert!(
                (q - (D0 * D0 - 0.5e-6 * t)).abs() < 1e-3 * q,
                "hot, t = {t}"
            );
        } else if *t > t_hot + 0.6 && *t < t_cool_end - 0.6 {
            let want = q_hot - 0.3e-6 * (t - t_hot);
            assert!(
                (q - want).abs() < 0.05 * want,
                "cool, t = {t}: {q} vs {want}"
            );
            assert!(f.state_probability[1][j] > 0.9);
        } else if *t > t_cool_end + 0.6 {
            assert!(
                (q - 1.0e-6).abs() < 0.05e-6,
                "after the cool flame, t = {t}: {q}"
            );
            assert!(f.state_probability[2][j] > 0.9);
        }
    }
    assert!(f.partition_max_error < 1e-9);
    // the same droplet as methanol (no cool stage in the spec) simply stops at the hot extinction
    let extra_m = "crit.b0 = -3.0\nrad.b0 = -0.6931471805599453\nrad.sharp = 30.0\n";
    let m = run(
        Fuel::Methanol,
        extra_m,
        &prefix(Fuel::Methanol, 0.5e-6, 3.0),
        json!({"horizon_s": 12.0, "n_points": 13}),
    );
    let last = *median(&m).last().unwrap();
    assert!((last * last - q_hot).abs() < 0.03 * q_hot);
}

#[test]
fn the_validity_gate_drops_what_later_frames_contradict() {
    // igniter glare: the first frames report a collapsed diameter, then the tracker recovers
    let mut p = prefix(Fuel::Methanol, 0.5e-6, 3.0);
    for o in p.observations.iter_mut().take(4) {
        o.value_si = Some(0.55 * D0);
    }
    let p = rehash(p);
    let f = run(
        Fuel::Methanol,
        "",
        &p,
        json!({"horizon_s": 4.0, "n_points": 5}),
    );
    assert_eq!(f.encoded.n_gated, 4);
    assert!((f.encoded.q0_m2 - D0 * D0).abs() < 0.01 * D0 * D0);
    for (j, t) in f.times_s.iter().enumerate() {
        let want = (D0 * D0 - 0.5e-6 * t).sqrt();
        assert!((median(&f)[j] - want).abs() < 1e-3 * want, "t = {t}");
    }
    // a single high outlier late in the prefix does not invalidate the frames before it; the outlier
    // itself goes, because no flame could have burned it down to the frames after it
    let mut p = prefix(Fuel::Methanol, 0.5e-6, 3.0);
    let n = p.observations.len();
    p.observations[n - 5].value_si = Some(1.2 * D0);
    let f = run(
        Fuel::Methanol,
        "",
        &rehash(p),
        json!({"horizon_s": 4.0, "n_points": 5}),
    );
    assert_eq!(f.encoded.n_gated, 1);
    assert!((f.encoded.q0_m2 - D0 * D0).abs() < 0.01 * D0 * D0);
}

#[test]
fn a_droplet_that_stopped_shrinking_is_read_as_extinguished() {
    // burns at the population rate for 2 s, then keeps its diameter to the 4 s cutoff
    let mut p = prefix(Fuel::Methanol, 0.5e-6, 4.0);
    let q_stop = D0 * D0 - 0.5e-6 * 2.0;
    for o in p.observations.iter_mut().filter(|o| o.time_s > 2.0) {
        o.value_si = Some(q_stop.sqrt());
    }
    let p = rehash(p);
    // encoder on (gain near one), an extinction law that makes extinction by now plausible, and
    // spreads that let the filter weigh the late rate
    let extra = "enc.gain_b = 6.0\ndiff.b0 = -1.5\ndiff.sharp = 6.0\nunc.s_pop = 0.2\nunc.sigma_frame = 0.000005\nfilt.sd_evap = 0.15\n";
    let f = run(
        Fuel::Methanol,
        extra,
        &p,
        json!({"horizon_s": 6.0, "n_points": 7}),
    );
    assert!(
        f.encoded.r_late.abs() < 0.15,
        "late rate ratio {}",
        f.encoded.r_late
    );
    assert!(
        f.encoded.state[2] > 0.95,
        "no-flame probability {:?} from prior {:?}",
        f.encoded.state,
        f.encoded.prior
    );
    for q in median(&f) {
        assert!((q * q - q_stop).abs() < 0.01 * q_stop);
    }
    // the same prefix read by weights whose law says extinction is impossible keeps burning
    let f = run(
        Fuel::Methanol,
        "enc.gain_b = 6.0\nunc.s_pop = 0.2\nunc.sigma_frame = 0.000005\n",
        &p,
        json!({"horizon_s": 6.0, "n_points": 7}),
    );
    assert!(f.encoded.state[0] > 0.99);
}

#[test]
fn the_predictive_band_is_the_mixture_of_measurement_gaussians() {
    // one atom and sigma_d = 20 um: the 5 % and 95 % quantiles sit 1.645 sigma from the median
    let f = run(
        Fuel::Methanol,
        "unc.sigma_d = 0.00002\n",
        &prefix(Fuel::Methanol, 0.5e-6, 3.0),
        json!({"horizon_s": 2.0, "n_points": 3, "quantile_levels": [0.05, 0.5, 0.95]}),
    );
    for j in 0..f.times_s.len() {
        let (lo, med, hi) = (f.quantiles[0][j], f.quantiles[1][j], f.quantiles[2][j]);
        assert!(((med - lo) - 1.6449e-5 * 2.0).abs() < 2e-8, "{}", med - lo);
        assert!(((hi - med) - 1.6449e-5 * 2.0).abs() < 2e-8);
        // latent quantiles carry no measurement noise
        assert!((f.latent_quantiles[0][j] - f.latent_quantiles[2][j]).abs() < 1e-12);
    }
}

#[test]
fn a_release_of_one_fuel_refuses_the_other() {
    let r = release(Fuel::Methanol, "");
    let p = prefix(Fuel::Heptane, 0.5e-6, 3.0);
    let out =
        droplet::forecast(&droplet::spec(Fuel::Methanol), &r, &p, &Value::Null).expect("runnable");
    assert!(out.is_err_and(|s| s.reasons.iter().any(|m| m.contains("fuel"))));
}

#[test]
fn the_validity_gate_drops_values_no_flame_could_have_burned_away() {
    // a soot shell counted as droplet: the first frames report 1.3 D0, then the true diameter
    let mut p = prefix(Fuel::Heptane, 0.5e-6, 3.0);
    for o in p.observations.iter_mut().take(6) {
        o.value_si = Some(1.3 * D0);
    }
    let f = run(
        Fuel::Heptane,
        "",
        &rehash(p),
        json!({"horizon_s": 4.0, "n_points": 5}),
    );
    assert_eq!(f.encoded.n_gated, 6);
    assert!((f.encoded.q0_m2 - D0 * D0).abs() < 0.01 * D0 * D0);
    for (j, t) in f.times_s.iter().enumerate() {
        let want = (D0 * D0 - 0.5e-6 * t).sqrt();
        assert!((median(&f)[j] - want).abs() < 1e-3 * want, "t = {t}");
    }
}

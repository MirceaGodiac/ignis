//! The droplet forward that also reads the flame cameras (`droplet5.rs`) on synthetic prefixes with
//! test-only weights: with its two additions switched off it is the third-generation forward, number
//! for number, flame diameters in the prefix or not; cameras that still show the flame make the
//! state hot and cameras that lost it make it not hot, with the typed trust; stray frames after the
//! flame, and too few frames, are not a flame; a flame burning past its clock stays in the radiative
//! class only when the clock is allowed to be wrong; the committed carries hold the third-generation
//! numbers unchanged.

mod common;

use ignis_core::contracts::{ModelRelease, ModelSpec, PrefixBundle, Quantity};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::droplet5::{self, Forecast, Fuel};
use ignis_wsp::{assemble, droplet3, flex_tools, models, text};
use serde_json::{json, Value};

const D0: f64 = 3.0e-3;
const K_HOT: f64 = 0.5e-6;

/// The toy weights of the third-generation tests: K_h = 0.5e-6 m2/s, no extinction of either kind,
/// no evaporation, encoder and state evidence off, no spread or noise.
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
post.hot_exp = 0.0
post.tau_exp = 1.0
enc.gain_w[0:3] = 0.0
enc.gain_b = -50.0
enc.mix[0:2] = [1.0, 0.0]
enc.e_clip[0:2] = [1.0, 1.0]
enc.cool_gain = 0.0
filt.sd_hot = 0.2
filt.sd_evap = 0.2
filt.l_floor = 0.0
filt.gain_w[0:3] = 0.0
filt.gain_b = -50.0
unc.s_pop = 0.0
unc.s_floor = 0.0
unc.sigma_d = 0.0
unc.sigma_frame = 0.0
haz.log_disruption = -20.0
";

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

/// The entries this spec adds, switched off.
const OFF: &str = "
rob.eps_r = 0.0
fl.trust = 0.0
fl.t_min_s = 0.3
fl.gap_s = 0.25
fl.min_points = 5.0
fl.evap_mix = 0.5
";

fn source(base: &str, extra: &str) -> String {
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
    src
}

fn build(key: &str, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{e}\n{src}"));
    let patch = assemble::make_patch(&spec, common::meta("test-droplet5"), ops);
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

fn toy(fuel: Fuel) -> String {
    if fuel.has_cool() {
        format!("{TOY}{TOY_COOL}")
    } else {
        TOY.to_string()
    }
}

fn key5(fuel: Fuel) -> &'static str {
    match fuel {
        Fuel::Methanol => "flex_methanol4",
        Fuel::Heptane => "flex_heptane4",
    }
}

fn release5(fuel: Fuel, extra: &str) -> LoadedRelease {
    build(key5(fuel), &source(&format!("{}{OFF}", toy(fuel)), extra))
}

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

/// A droplet of `fuel` whose D^2 is `q(t)`, observed at 30 Hz to `cutoff`, in air.
fn prefix(fuel: Fuel, cutoff: f64, q: impl Fn(f64) -> f64) -> PrefixBundle {
    let mut p = flex_tools::fixture_prefix(D0, K_HOT, cutoff, 0.21, 0.0, 0.0, 101_325.0);
    if let Some(f) = p.known_conditions.get_mut("fuel") {
        f.raw_value = Some(ignis_core::contracts::RawValue::Text(fuel.label().into()));
    }
    for o in &mut p.observations {
        o.value_si = Some(q(o.time_s).sqrt());
    }
    rehash(p)
}

/// The prefix with UV flame diameters of 12 mm at the given times.
fn with_flame(mut p: PrefixBundle, times: impl IntoIterator<Item = f64>) -> PrefixBundle {
    let template = p.observations[0].clone();
    for (i, t) in times.into_iter().enumerate() {
        let mut o = template.clone();
        o.id = format!("uv/{i}");
        o.quantity = Quantity::UvFlameDiameter;
        o.channel = Some("LLUVData".into());
        o.time_s = t;
        o.available_at_s = t;
        o.value_si = Some(0.012);
        p.observations.push(o);
    }
    rehash(p)
}

/// Frames at 30 Hz from `from` to `to`.
fn frames(from: f64, to: f64) -> Vec<f64> {
    let n = ((to - from) * 30.0).floor() as usize;
    (0..=n).map(|i| from + i as f64 / 30.0).collect()
}

fn run5(fuel: Fuel, extra: &str, p: &PrefixBundle, settings: &Value) -> Forecast {
    droplet5::forecast(&droplet5::spec(fuel), &release5(fuel, extra), p, settings)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

const CUTOFF: f64 = 5.0;
const T_EXT: f64 = 3.5;
const K_EVAP: f64 = 0.02e-6;

fn q_law(t: f64) -> f64 {
    D0 * D0 - K_HOT * t
}

/// Hot flame until `T_EXT`, then evaporation at `K_EVAP`.
fn q_out(t: f64) -> f64 {
    if t <= T_EXT {
        q_law(t)
    } else {
        q_law(T_EXT) - K_EVAP * (t - T_EXT)
    }
}

/// A diffusive law that leaves the prior at roughly 0.9 hot at the cutoff, the state evidence on,
/// an unlit droplet that evaporates, and some spread: a forecast in which every part of the forward
/// has something to do.
const BUSY: &str = "diff.b0 = 0.0\ndiff.sharp = 6.0\nfilt.gain_b = 10.0\npost.k_inf = 2.0e-8\npost.k_hot = 1.0e-7\nunc.s_floor = 0.05\nunc.sigma_d = 1.0e-5\n";

fn settings() -> Value {
    json!({ "times_s": [5.5, 6.0, 7.0, 9.0, 12.0], "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95] })
}

#[test]
fn with_its_additions_off_it_is_the_third_generation_forward() {
    for fuel in [Fuel::Methanol, Fuel::Heptane] {
        let fuel3 = match fuel {
            Fuel::Methanol => droplet3::Fuel::Methanol,
            Fuel::Heptane => droplet3::Fuel::Heptane,
        };
        let key3 = match fuel {
            Fuel::Methanol => "flex_methanol2",
            Fuel::Heptane => "flex_heptane2",
        };
        for q in [q_law as fn(f64) -> f64, q_out] {
            let bare = prefix(fuel, CUTOFF, q);
            let old = droplet3::forecast(
                &droplet3::spec(fuel3),
                &build(key3, &source(&toy(fuel), BUSY)),
                &bare,
                &settings(),
            )
            .unwrap()
            .unwrap();
            // flame diameters in the prefix or not: nothing is read with fl.trust = 0
            for p in [bare.clone(), with_flame(bare.clone(), frames(0.3, 3.0))] {
                let new = run5(fuel, BUSY, &p, &settings());
                assert_eq!(new.quantiles, old.quantiles);
                assert_eq!(new.state_probability, old.state_probability);
                assert_eq!(new.encoded.state, old.encoded.state);
                assert!(!new.encoded.flame.used);
            }
        }
    }
}

#[test]
fn cameras_that_still_show_the_flame_make_the_state_hot() {
    let on = format!("{BUSY}fl.trust = 0.95\n");
    // the diameter says the flame went out at 3.5 s; the cameras show it to the cutoff
    let p = with_flame(prefix(Fuel::Methanol, CUTOFF, q_out), frames(0.3, CUTOFF));
    let diameter_only = run5(Fuel::Methanol, BUSY, &p, &settings());
    assert!(
        diameter_only.encoded.state[0] < 0.2,
        "{:?}",
        diameter_only.encoded.state
    );
    let f = run5(Fuel::Methanol, &on, &p, &settings());
    assert!(f.encoded.flame.used && f.encoded.flame.seen);
    assert_eq!(f.encoded.state_before_cameras, diameter_only.encoded.state);
    let want = 0.95 + 0.05 * diameter_only.encoded.state[0];
    assert!(
        (f.encoded.state[0] - want).abs() < 1e-12,
        "{:?}",
        f.encoded.state
    );
    assert!((f.encoded.state.iter().sum::<f64>() - 1.0).abs() < 1e-12);
}

#[test]
fn cameras_that_lost_the_flame_make_the_state_not_hot() {
    let on = format!("{BUSY}fl.trust = 0.95\n");
    // the diameter says the droplet is burning; the cameras lost the flame at 3 s
    let p = with_flame(prefix(Fuel::Methanol, CUTOFF, q_law), frames(0.3, 3.0));
    let diameter_only = run5(Fuel::Methanol, BUSY, &p, &settings());
    assert!(
        diameter_only.encoded.state[0] > 0.8,
        "{:?}",
        diameter_only.encoded.state
    );
    let f = run5(Fuel::Methanol, &on, &p, &settings());
    assert!(f.encoded.flame.used && !f.encoded.flame.seen);
    assert!((f.encoded.flame.t_last_s.unwrap() - 3.0).abs() < 0.04);
    let want = 0.05 * diameter_only.encoded.state[0];
    assert!(
        (f.encoded.state[0] - want).abs() < 1e-12,
        "{:?}",
        f.encoded.state
    );
    assert!((f.encoded.state.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    // an unlit droplet shrinks more slowly than a burning one: the median ends higher
    let mid = |f: &Forecast| f.quantiles[2][f.times_s.len() - 1];
    assert!(mid(&f) > mid(&diameter_only) + 1.0e-4);
    // the heat the flame left has decayed for two seconds: less of it than the end rate suggests
    let all_end_rate = run5(
        Fuel::Methanol,
        &format!("{on}fl.evap_mix = 0.0\n"),
        &p,
        &settings(),
    );
    assert!(mid(&f) > mid(&all_end_rate));
}

#[test]
fn stray_frames_and_too_few_frames_are_not_a_flame() {
    let on = format!("{BUSY}fl.trust = 0.95\n");
    let base = prefix(Fuel::Methanol, CUTOFF, q_law);
    // two stray frames after a flame lost at 3 s: still lost at 3 s
    let mut t = frames(0.3, 3.0);
    t.extend([4.6, 4.95]);
    let f = run5(
        Fuel::Methanol,
        &on,
        &with_flame(base.clone(), t),
        &settings(),
    );
    assert!(f.encoded.flame.used && !f.encoded.flame.seen);
    assert!((f.encoded.flame.t_last_s.unwrap() - 3.0).abs() < 0.04);
    // three frames within the gap, the last at the cutoff: seen
    let mut t = frames(0.3, 3.0);
    t.extend([4.85, 4.92, 4.99]);
    let f = run5(
        Fuel::Methanol,
        &on,
        &with_flame(base.clone(), t),
        &settings(),
    );
    assert!(f.encoded.flame.seen);
    // four frames in all: the cameras did not record this flame, nothing changes
    let few = with_flame(base.clone(), [0.4, 0.5, 0.6, 0.7]);
    let f = run5(Fuel::Methanol, &on, &few, &settings());
    let plain = run5(Fuel::Methanol, BUSY, &base, &settings());
    assert!(!f.encoded.flame.used);
    assert_eq!(f.quantiles, plain.quantiles);
    // frames before fl.t_min_s do not count either
    let glare = with_flame(base.clone(), frames(0.0, 0.25));
    assert!(
        !run5(Fuel::Methanol, &on, &glare, &settings())
            .encoded
            .flame
            .used
    );
}

#[test]
fn a_flame_burning_past_its_clock_stays_radiative_only_when_the_clock_can_be_wrong() {
    // every droplet is of the radiative class, and the clock says the flame is gone long before
    // the cutoff: s_R = exp(-3) = 0.05 against s = 0.56 at 5 s
    let late =
        "class.kappa = 0.0\nclass.bias = 10.0\nrad.b0 = -3.0\nrad.sharp = 6.0\nfl.trust = 0.95\n";
    let p = with_flame(prefix(Fuel::Methanol, CUTOFF, q_law), frames(0.3, CUTOFF));
    let sharp = run5(Fuel::Methanol, late, &p, &settings());
    // the only hot droplets the law leaves are the 1 in 22000 it puts in the diffusive class
    assert!(sharp.encoded.prior[0] < 1e-4);
    // the cameras put the droplet in the hot state, and the law then concludes that it was never
    // of the radiative class: a flame the clock had taken cannot be a radiative one
    assert!(sharp.encoded.state[0] > 0.94);
    assert!(
        sharp.encoded.class_r_given_hot < 1e-9,
        "{}",
        sharp.encoded.class_r_given_hot
    );
    let wide = run5(
        Fuel::Methanol,
        &format!("{late}rob.eps_r = 0.1\n"),
        &p,
        &settings(),
    );
    // one in ten that the clock is not the law's: the prior is no longer zero
    let s_cut = CUTOFF * 1.0e-6 / (D0 * D0);
    let want = 0.1 / (1.0 + s_cut / (-3.0f64).exp());
    assert!(
        (wide.encoded.prior[0] - want).abs() < 1e-4,
        "{}",
        wide.encoded.prior[0]
    );
    assert!(wide.encoded.class_r_given_hot > 0.99);
    // and a flame that has burned this long is expected to burn on for a while
    let hot = &wide.state_probability[0];
    assert!(hot[0] > 0.85 && hot[hot.len() - 1] > 0.35, "{hot:?}");
    assert!(hot.windows(2).all(|w| w[1] <= w[0] + 1e-12));
}

#[test]
fn the_committed_specs_and_carries_are_what_the_forward_was_compiled_with() {
    let config = |path: &str| -> String {
        let path = format!(
            "{}/../../configs/models/flex/{path}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    let read = |path: &str| -> ModelRelease {
        serde_json::from_str(&config(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    for (fuel, dir, carry, old) in [
        (Fuel::Methanol, "methanol", "carry-d3", "d3-fable"),
        (Fuel::Heptane, "heptane", "carry-d2", "d2-fable"),
    ] {
        let spec = droplet5::spec(fuel);
        spec.validate().expect("spec valid");
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp4/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/flex/wsp4/{dir}/spec.json drifted from droplet5::spec(); a changed spec \
             is a new spec id (re-export with `ignis wsp spec --family {} --out …`), never an \
             in-place edit",
            key5(fuel)
        );
        // the carry holds every tensor of the third-generation candidate, unchanged
        let (new, third) = (
            read(&format!("wsp4/{dir}/{carry}/release.json")),
            read(&format!("wsp2/{dir}/{old}/release.json")),
        );
        for t in &third.tensors {
            let have = new
                .tensors
                .iter()
                .find(|e| e.name == t.name)
                .unwrap_or_else(|| panic!("{dir}: the carry lacks {}", t.name));
            assert_eq!(
                (&have.shape, &have.sha256),
                (&t.shape, &t.sha256),
                "{dir}: {} differs from the third generation",
                t.name
            );
        }
        // and the candidates differ from it only in the entries this spec adds
        for cand in ["r1-fable", "f1-fable"] {
            let c = read(&format!("wsp4/{dir}/{cand}/release.json"));
            for t in &new.tensors {
                let have = c.tensors.iter().find(|e| e.name == t.name).unwrap();
                let added = t.name.starts_with("fl.") || t.name.starts_with("rob.");
                assert!(
                    added || have.sha256 == t.sha256,
                    "{dir}/{cand}: {} differs from the carry",
                    t.name
                );
            }
        }
    }
}

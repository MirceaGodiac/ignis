//! The third-generation droplet forward on synthetic prefixes with test-only weights: the state
//! evidence and the level at the cutoff read from the end rate, the cap on the level carry, the
//! size scalings of the post-extinction transient and the run offset of a cool flame.

mod common;

use ignis_core::contracts::PrefixBundle;
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::droplet3::{self, Forecast, Fuel};
use ignis_wsp::{assemble, flex_tools, models, text};
use serde_json::{json, Value};

const D0: f64 = 3.0e-3;
const K_HOT: f64 = 0.5e-6;
/// `enc.end_frac` of the spec.
const END_FRAC: f64 = 0.15;

/// K_h = 0.5e-6 m2/s, no extinction of either kind, no evaporation, encoder and state evidence off
/// (exponents ~0), no spread or noise.
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

/// No cool flame after a radiative extinction unless a test says so; K_c = 0.3e-6, D_ce = 1 mm.
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

fn key(fuel: Fuel) -> &'static str {
    match fuel {
        Fuel::Methanol => "flex_methanol2",
        Fuel::Heptane => "flex_heptane2",
    }
}

/// Toy weights with `extra` statements replacing the base statements of the tensors they name.
fn release(fuel: Fuel, extra: &str) -> LoadedRelease {
    let model = models::by_key(key(fuel)).expect("model");
    assert_eq!(model.model_name, fuel.model_name());
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
    let patch = assemble::make_patch(&spec, common::meta("test-droplet3"), ops);
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

/// A D^2-law droplet of `fuel` with initial diameter `d0`, burning at `k` in air, observed at 30 Hz
/// to `cutoff`.
fn prefix(fuel: Fuel, d0: f64, k: f64, cutoff: f64) -> PrefixBundle {
    let mut p = flex_tools::fixture_prefix(d0, k, cutoff, 0.21, 0.0, 0.0, 101_325.0);
    if let Some(f) = p.known_conditions.get_mut("fuel") {
        f.raw_value = Some(ignis_core::contracts::RawValue::Text(fuel.label().into()));
    }
    rehash(p)
}

/// The prefix with D^2 rewritten as `q(t)` at every observation.
fn reshape(mut p: PrefixBundle, q: impl Fn(f64) -> f64) -> PrefixBundle {
    for o in &mut p.observations {
        o.value_si = Some(q(o.time_s).sqrt());
    }
    rehash(p)
}

fn run(fuel: Fuel, extra: &str, p: &PrefixBundle, settings: Value) -> Forecast {
    let r = release(fuel, extra);
    droplet3::forecast(&droplet3::spec(fuel), &r, p, &settings)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn median(f: &Forecast) -> &[f64] {
    let i = f.levels.iter().position(|p| *p == 0.5).expect("median");
    &f.quantiles[i]
}

/// Mean D^2 over the last end window of a prefix (no observation is gated in these prefixes).
fn last_end_window(p: &PrefixBundle) -> (f64, f64) {
    let obs: Vec<(f64, f64)> = p
        .observations
        .iter()
        .map(|o| (o.time_s, o.value_si.unwrap().powi(2)))
        .collect();
    let (t1, tn) = (obs[0].0, obs[obs.len() - 1].0);
    let lo = tn - END_FRAC * (tn - t1);
    let w: Vec<&(f64, f64)> = obs.iter().filter(|o| o.0 >= lo && o.0 <= tn).collect();
    let n = w.len() as f64;
    (
        w.iter().map(|o| o.0).sum::<f64>() / n,
        w.iter().map(|o| o.1).sum::<f64>() / n,
    )
}

// Hot flame until T_EXT, then evaporation at K_EVAP, to a 5 s cutoff.
const CUTOFF_1: f64 = 5.0;
const T_EXT: f64 = 3.5;
const K_EVAP: f64 = 0.02e-6;

fn q_out(t: f64) -> f64 {
    if t <= T_EXT {
        D0 * D0 - K_HOT * t
    } else {
        D0 * D0 - K_HOT * T_EXT - K_EVAP * (t - T_EXT)
    }
}

/// A diffusive law that leaves the prior at roughly 0.9 hot at the cutoff, and the state evidence on
/// (exponent ~1) with a hot spread that separates hot from no flame.
const EVIDENCE_ON: &str = "diff.b0 = 0.0\ndiff.sharp = 6.0\nfilt.gain_b = 10.0\n";

#[test]
fn a_droplet_that_went_out_before_the_cutoff_is_read_from_the_end_rate() {
    let p = reshape(prefix(Fuel::Methanol, D0, K_HOT, CUTOFF_1), q_out);
    let f = run(
        Fuel::Methanol,
        EVIDENCE_ON,
        &p,
        json!({"horizon_s": 4.0, "n_points": 5}),
    );
    let e = &f.encoded;
    assert_eq!(e.n_gated, 0);
    assert!(e.gain_state > 0.99, "evidence exponent {}", e.gain_state);
    assert!(
        e.prior[0] > 0.85,
        "prior favours a hot flame: {:?}",
        e.prior
    );
    // the end windows both lie after the extinction; the mid -> tail difference straddles it
    assert!(
        (e.k_obs_end_m2_s - K_EVAP).abs() < 1e-12,
        "end rate {}",
        e.k_obs_end_m2_s
    );
    assert!(
        e.r_late > 0.4 && e.r_late > 10.0 * e.r_end,
        "r_late {} vs r_end {}: the late rate alone would not say the flame is out",
        e.r_late,
        e.r_end
    );
    assert!(
        e.state[2] > 0.9,
        "no-flame probability {:?} from prior {:?}",
        e.state,
        e.prior
    );
    // the same droplet still burning at the cutoff stays hot under the same weights
    let f = run(
        Fuel::Methanol,
        EVIDENCE_ON,
        &prefix(Fuel::Methanol, D0, K_HOT, CUTOFF_1),
        json!({"horizon_s": 4.0, "n_points": 5}),
    );
    assert!(
        f.encoded.state[0] > 0.9,
        "hot probability {:?}",
        f.encoded.state
    );
}

#[test]
fn the_level_at_the_cutoff_is_the_last_end_window_carried_along_the_end_rate() {
    // an exact D^2 law: the carried level is exact
    let f = run(
        Fuel::Methanol,
        "",
        &prefix(Fuel::Methanol, D0, K_HOT, 3.0),
        json!({"horizon_s": 2.0, "n_points": 3}),
    );
    let want = D0 * D0 - K_HOT * 3.0;
    let rel = (f.encoded.q_cutoff_m2 - want).abs() / want;
    assert!(rel < 1e-9, "linear prefix: relative error {rel:e}");
    // a droplet out since 1.5 s before the cutoff: carried along the evaporation rate, not along the
    // mid -> tail rate that still holds the flame
    let p = reshape(prefix(Fuel::Methanol, D0, K_HOT, CUTOFF_1), q_out);
    let f = run(
        Fuel::Methanol,
        EVIDENCE_ON,
        &p,
        json!({"horizon_s": 2.0, "n_points": 3}),
    );
    let e = &f.encoded;
    let want = q_out(CUTOFF_1);
    let rel = (e.q_cutoff_m2 - want).abs() / want;
    assert!(rel < 1e-9, "extinguished prefix: relative error {rel:e}");
    // the second generation's carry (tail mean along the mid -> tail rate) lands visibly lower
    let gen2 = e.q_tail_m2 - e.k_obs_late_m2_s * (CUTOFF_1 - e.t_tail_s);
    assert!(
        want - gen2 > 0.01 * want,
        "gen-2 carry {gen2} vs truth {want}"
    );
}

#[test]
fn the_level_is_not_carried_faster_than_twice_the_mid_tail_rate() {
    // a droplet that kept its size through the mid and tail windows, then lost 1.9e-6 m2/s in the
    // last end window: under gate.k_max_m2_s, so the gate keeps it, but the end rate is several times
    // the mid -> tail rate
    let t_drop = CUTOFF_1 * (1.0 - END_FRAC);
    let q_flat = D0 * D0 - K_HOT * 1.0;
    let p = reshape(prefix(Fuel::Methanol, D0, K_HOT, CUTOFF_1), |t| {
        if t <= 1.0 {
            D0 * D0 - K_HOT * t
        } else if t <= t_drop {
            q_flat
        } else {
            q_flat - 1.9e-6 * (t - t_drop)
        }
    });
    let f = run(
        Fuel::Methanol,
        "",
        &p,
        json!({"horizon_s": 2.0, "n_points": 3}),
    );
    let e = &f.encoded;
    assert_eq!(e.n_gated, 0);
    assert!(
        e.k_obs_end_m2_s > 2.0 * e.k_obs_late_m2_s,
        "end rate {} vs late rate {}",
        e.k_obs_end_m2_s,
        e.k_obs_late_m2_s
    );
    let (t_level, q_level) = last_end_window(&p);
    assert!((t_level - e.t_level_s).abs() < 1e-12);
    let dt = CUTOFF_1 - e.t_level_s;
    let capped = q_level - 2.0 * e.k_obs_late_m2_s * dt;
    let uncapped = q_level - e.k_obs_end_m2_s * dt;
    assert!(
        (e.q_cutoff_m2 - capped).abs() < 1e-9 * capped,
        "carried {} vs cap {capped}",
        e.q_cutoff_m2
    );
    assert!(e.q_cutoff_m2 > uncapped + 0.01 * capped);
}

#[test]
fn an_unlit_droplet_sheds_heat_with_the_written_size_scalings() {
    // every droplet in the radiative class, its clock long past s_R: no flame at the cutoff by the
    // prior; state evidence exponent ~0, so the transient keeps half of post.k_hot
    let (k_inf, k_hot, tau0, m, h, n) = (0.03e-6, 0.3e-6, 2.0, 0.5, 1.5, 0.8);
    let extra = format!(
        "crit.b0 = -3.0\nrad.b0 = -4.0\npost.k_inf = {k_inf:e}\npost.k_hot = {k_hot:e}\n\
         post.tau0_s = {tau0}\npost.size_exp = {m}\npost.hot_exp = {h}\npost.tau_exp = {n}\n"
    );
    for d0 in [D0, 1.6e-3] {
        let f = run(
            Fuel::Methanol,
            &extra,
            &prefix(Fuel::Methanol, d0, K_HOT, 3.0),
            json!({"horizon_s": 6.0, "n_points": 7}),
        );
        let e = &f.encoded;
        assert!(e.state[2] > 1.0 - 1e-12, "{d0}: state {:?}", e.state);
        assert!(e.gain_state < 1e-15);
        let q_e = e.q_cutoff_m2;
        let r = q_e / 4.0e-6;
        let tau_e = tau0 * r.powf(n);
        let drop = |dt: f64| {
            k_inf * r.powf(m) * dt + 0.5 * k_hot * r.powf(h) * tau_e * (1.0 - (-dt / tau_e).exp())
        };
        // the second generation's scalings (amplitude r^0, decay time r^1) give another curve
        let gen2 = |dt: f64| {
            let tau = tau0 * r;
            k_inf * r.powf(m) * dt + 0.5 * k_hot * tau * (1.0 - (-dt / tau).exp())
        };
        for (j, t) in f.times_s.iter().enumerate() {
            let dt = t - 3.0;
            let got = q_e - median(&f)[j].powi(2);
            assert!(
                (got - drop(dt)).abs() < 1e-9 * q_e,
                "D0 {d0}, t = {t}: drop {got} vs {}",
                drop(dt)
            );
            if dt > 0.5 {
                assert!((gen2(dt) - drop(dt)).abs() > 0.05 * drop(dt));
            }
        }
    }
}

#[test]
fn a_cool_flame_at_the_cutoff_keeps_its_end_rate_innovation_as_run_offset() {
    // radiative class, hot flame out long ago, a cool flame always follows and is alive (D > D_ce)
    let laws = "crit.b0 = -3.0\nrad.b0 = -4.0\ncool.p.b0 = 30.0\nfilt.gain_b = 0.4\n";
    let k_obs = 0.45e-6;
    let k_cool = 0.3e-6;
    let p = prefix(Fuel::Heptane, D0, k_obs, 3.0);
    for (gain, clip_hi) in [(0.7, 1.0), (0.7, 0.2), (0.0, 1.0)] {
        let extra = format!("{laws}enc.cool_gain = {gain}\nenc.e_clip[0:2] = [1.0, {clip_hi}]\n");
        let f = run(
            Fuel::Heptane,
            &extra,
            &p,
            json!({"horizon_s": 4.0, "n_points": 5}),
        );
        let e = &f.encoded;
        assert!(e.prior[1] > 0.999, "prior {:?}", e.prior);
        assert!((e.laws.k_cool_m2_s - k_cool).abs() < 1e-15);
        let g = e.gain_state;
        assert!(g > 0.5 && g < 0.7, "evidence exponent {g}");
        let want = gain * g * (k_obs / k_cool).ln().clamp(-1.0, clip_hi);
        assert!(
            (e.b_cool - want).abs() < 1e-9,
            "gain {gain}, clip {clip_hi}: b_cool {} vs {want}",
            e.b_cool
        );
        if gain == 0.0 {
            assert_eq!(e.b_cool, 0.0);
        }
        // the forecast burns the cool flame at K_c exp(b_cool) from the level at the cutoff
        let k_c = k_cool * e.b_cool.exp();
        for (j, t) in f.times_s.iter().enumerate() {
            let want = e.q_cutoff_m2 - k_c * (t - 3.0);
            let got = median(&f)[j].powi(2);
            assert!((got - want).abs() < 1e-9 * want, "t = {t}: {got} vs {want}");
        }
    }
}

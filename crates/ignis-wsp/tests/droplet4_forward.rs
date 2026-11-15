//! The fourth-generation droplet forward: with its new entries neutral it is the third-generation
//! forward number for number (on the committed candidates of both generations), the artefact flags
//! read the prefix and widen only the measurement band, the helium term speeds up only the
//! evaporation after a hot flame that ends inside the horizon, and the state partition still sums
//! to one. Test-only weights are synthetic; nothing here scores a candidate.

mod common;

use ignis_core::contracts::{PrefixBundle, RawValue, ReleaseId};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::{assemble, droplet3, droplet4, flex_tools, models, text};
use serde_json::{json, Value};

const M3_BASE: &str =
    include_str!("../../../configs/models/flex/wsp2/methanol/baseline-d2/edits.wsp");
const M3_CAND: &str = include_str!("../../../configs/models/flex/wsp2/methanol/d3-fable/edits.wsp");
const H3_BASE: &str =
    include_str!("../../../configs/models/flex/wsp2/heptane/baseline-d2/edits.wsp");
const H3_CAND: &str = include_str!("../../../configs/models/flex/wsp2/heptane/d2-fable/edits.wsp");
const M4_BASE: &str =
    include_str!("../../../configs/models/flex/wsp3/methanol/baseline-d2/edits.wsp");
/// The neutral carry of the third-generation candidate (stays neutral; the d4 folder holds the
/// author's fourth-generation numbers).
const M4_CARRY: &str =
    include_str!("../../../configs/models/flex/wsp3/methanol/carry-d3/edits.wsp");
const H4_BASE: &str =
    include_str!("../../../configs/models/flex/wsp3/heptane/baseline-d2/edits.wsp");
const H4_CARRY: &str = include_str!("../../../configs/models/flex/wsp3/heptane/carry-d2/edits.wsp");

const D0: f64 = 3.0e-3;
const K_HOT: f64 = 0.5e-6;

/// K_h = 0.5e-6 m2/s, no extinction of either kind, no evaporation, encoder and state evidence off,
/// no spread or noise; the fourth-generation entries neutral.
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
art.min_gated = 1000.0
art.d0_tol = 5.0
art.rise_tol = 5.0
art.weight[0:4] = [0.0, 0.0, 0.0, 0.0]
art.r_band[0:2] = [-1000.0, 1000.0]
art.sigma_m = 0.0
art.sigma_rate_m_s = 0.0
post.he_w = 0.0
post.seen_max = 3.0
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

fn key3(fuel: droplet3::Fuel) -> &'static str {
    match fuel {
        droplet3::Fuel::Methanol => "flex_methanol2",
        droplet3::Fuel::Heptane => "flex_heptane2",
    }
}

fn key4(fuel: droplet4::Fuel) -> &'static str {
    match fuel {
        droplet4::Fuel::Methanol => "flex_methanol3",
        droplet4::Fuel::Heptane => "flex_heptane3",
    }
}

/// An in-memory release, with the content id the registry would give it (a revision names it).
fn loaded(mut release: ignis_core::contracts::ModelRelease, params: TensorMap) -> LoadedRelease {
    release.id = content_hash(&release, &["id", "created_at"]);
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

/// The root release of a model from the edits `base`, built in memory.
fn root(key: &str, base: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let ops = text::compile(base, &spec).unwrap_or_else(|e| panic!("{key} base: {e}"));
    let patch = assemble::make_patch(&spec, common::meta("test-base"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize base");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("base release");
    loaded(release, tensors)
}

/// A literal revision of `parent` by the edits `src`, built in memory.
fn revise(key: &str, parent: &LoadedRelease, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let mut meta = common::meta("test-revision");
    meta.parent_release_id = Some(ReleaseId::new(parent.release.id.clone()).unwrap());
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key} revision: {e}"));
    let patch = assemble::make_patch(&spec, meta, ops);
    let tensors = assemble::materialize(&spec, &patch, Some(parent)).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, Some(parent), &common::binding())
        .expect("revision release");
    loaded(release, tensors)
}

/// The committed baseline and candidate of a model.
fn lineage(key: &str, base: &str, cand: &str) -> (LoadedRelease, LoadedRelease) {
    let b = root(key, base);
    let c = revise(key, &b, cand);
    (b, c)
}

/// Toy weights with `extra` statements replacing the base statements of the tensors they name.
fn toy(fuel: droplet4::Fuel, extra: &str) -> LoadedRelease {
    let model = models::by_key(key4(fuel)).expect("model");
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
    let patch = assemble::make_patch(&spec, common::meta("test-droplet4"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("release");
    loaded(release, tensors)
}

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

/// A D^2-law droplet observed at 30 Hz to `cutoff`, in an atmosphere of `x_o2` and `x_he`.
fn prefix(label: &str, d0: f64, k: f64, cutoff: f64, x_o2: f64, x_he: f64) -> PrefixBundle {
    let mut p = flex_tools::fixture_prefix(d0, k, cutoff, x_o2, 0.0, x_he, 101_325.0);
    if let Some(f) = p.known_conditions.get_mut("fuel") {
        f.raw_value = Some(RawValue::Text(label.into()));
    }
    rehash(p)
}

/// The prefix with D^2 rewritten as `q(t, D^2)` at every observation.
fn reshape(mut p: PrefixBundle, q: impl Fn(f64, f64) -> f64) -> PrefixBundle {
    for o in &mut p.observations {
        let d = o.value_si.unwrap();
        o.value_si = Some(q(o.time_s, d * d).sqrt());
    }
    rehash(p)
}

/// The prefix with the test sheet's initial diameter `d` among its conditions.
fn with_sheet_d0(mut p: PrefixBundle, d: f64) -> PrefixBundle {
    let mut m = p.known_conditions["pressure"].clone();
    m.raw_value = Some(RawValue::Number(d * 1e3));
    m.raw_unit = Some("mm".into());
    m.value_si = Some(d);
    m.unit_si = Some("m".into());
    p.known_conditions.insert("initial_diameter".into(), m);
    rehash(p)
}

fn run4(
    fuel: droplet4::Fuel,
    r: &LoadedRelease,
    p: &PrefixBundle,
    s: &Value,
) -> droplet4::Forecast {
    droplet4::forecast(&droplet4::spec(fuel), r, p, s)
        .expect("runnable")
        .unwrap_or_else(|e| panic!("refused: {:?}", e.reasons))
}

fn run3(
    fuel: droplet3::Fuel,
    r: &LoadedRelease,
    p: &PrefixBundle,
    s: &Value,
) -> droplet3::Forecast {
    droplet3::forecast(&droplet3::spec(fuel), r, p, s)
        .expect("runnable")
        .unwrap_or_else(|e| panic!("refused: {:?}", e.reasons))
}

fn level(levels: &[f64], q: &[Vec<f64>], p: f64) -> Vec<f64> {
    q[levels.iter().position(|l| *l == p).expect("level")].clone()
}

fn max_diff(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .flat_map(|(x, y)| {
            assert_eq!(x.len(), y.len());
            x.iter().zip(y).map(|(u, v)| (u - v).abs())
        })
        .fold(0.0, f64::max)
}

/// Prefixes that reach every branch of the forward with the committed candidates: burning, gone out
/// before the cutoff, in helium, with a few impossible frames, with the test sheet's D0.
fn equivalence_prefixes(label: &str) -> Vec<PrefixBundle> {
    let out = |t: f64, q: f64| {
        if t <= 3.5 {
            q
        } else {
            q + K_HOT * (t - 3.5) - 0.02e-6 * (t - 3.5)
        }
    };
    vec![
        prefix(label, D0, K_HOT, 3.0, 0.21, 0.0),
        prefix(label, 2.2e-3, 0.6e-6, 1.0, 0.21, 0.0),
        reshape(prefix(label, D0, K_HOT, 5.0, 0.21, 0.0), out),
        prefix(label, 2.5e-3, 0.7e-6, 2.0, 0.19, 0.3),
        prefix(label, 3.5e-3, 0.45e-6, 4.0, 0.16, 0.0),
        reshape(prefix(label, D0, K_HOT, 4.0, 0.21, 0.0), |t, q| {
            if (t - 1.0).abs() < 1e-9 || (t - 2.0).abs() < 1e-9 {
                0.25 * q
            } else {
                q
            }
        }),
        with_sheet_d0(prefix(label, D0, K_HOT, 3.0, 0.21, 0.35), 2.0e-3),
    ]
}

#[test]
fn with_neutral_new_entries_the_committed_candidates_give_the_third_generation_numbers() {
    let settings = json!({"horizon_s": 12.0, "n_points": 25});
    let cases = [
        (
            droplet3::Fuel::Methanol,
            droplet4::Fuel::Methanol,
            (M3_BASE, M3_CAND),
            (M4_BASE, M4_CARRY),
        ),
        (
            droplet3::Fuel::Heptane,
            droplet4::Fuel::Heptane,
            (H3_BASE, H3_CAND),
            (H4_BASE, H4_CARRY),
        ),
    ];
    for (f3, f4, (b3, c3), (b4, c4)) in cases {
        let (base3, cand3) = lineage(key3(f3), b3, c3);
        let (base4, cand4) = lineage(key4(f4), b4, c4);
        let releases = [(base3, base4), (cand3, cand4)];
        let mut cool_seen = false;
        for (r3, r4) in &releases {
            for p in equivalence_prefixes(f4.label()) {
                let a = run3(f3, r3, &p, &settings);
                let b = run4(f4, r4, &p, &settings);
                assert_eq!(b.encoded.art_weight, 0.0);
                assert!(!b.encoded.art_rate);
                assert_eq!(b.encoded.he_scale, 1.0);
                let d = [
                    max_diff(&a.quantiles, &b.quantiles),
                    max_diff(&a.latent_quantiles, &b.latent_quantiles),
                    max_diff(&[a.mean.clone()], &[b.mean.clone()]),
                    max_diff(&a.events, &b.events),
                    max_diff(&a.state_probability, &b.state_probability),
                ];
                assert!(
                    d.iter().all(|v| *v <= 1e-12),
                    "{} cutoff {}: differences {d:?}",
                    f4.label(),
                    p.cutoff_s
                );
                cool_seen |= b.state_probability[1].iter().any(|v| *v > 1e-3);
            }
        }
        if f4.has_cool() {
            assert!(cool_seen, "the heptane cases never reached a cool flame");
        }
    }
}

const GATED: &str =
    "unc.sigma_d = 2.0e-5\nart.min_gated = 2.0\nart.weight[0:4] = [0.5, 0.0, 0.0, 0.0]\n\
                     art.sigma_m = 2.0e-4\nart.sigma_rate_m_s = 2.0e-5\n";

#[test]
fn a_few_impossible_frames_after_the_igniter_widen_the_band_and_a_clean_prefix_does_not() {
    let fuel = droplet4::Fuel::Methanol;
    let settings = json!({"horizon_s": 4.0, "n_points": 5});
    let on = toy(fuel, GATED);
    let off = toy(
        fuel,
        &GATED.replace(
            "art.weight[0:4] = [0.5, 0.0, 0.0, 0.0]",
            "art.weight[0:4] = [0.0, 0.0, 0.0, 0.0]",
        ),
    );
    // two frames read at half the diameter (partial occlusion): every later frame contradicts them
    let clean = prefix(fuel.label(), D0, K_HOT, 3.0, 0.21, 0.0);
    let gated = reshape(clean.clone(), |t, q| {
        if (t - 1.0).abs() < 1e-9 || (t - 2.0).abs() < 1e-9 {
            0.25 * q
        } else {
            q
        }
    });
    let g = run4(fuel, &on, &gated, &settings);
    let e = &g.encoded;
    assert_eq!((e.n_gated, e.n_gated_late), (2, 2));
    assert!(e.art_gated && !e.art_size && !e.art_growth && !e.art_rate);
    assert_eq!(e.art_weight, 0.5);
    let g0 = run4(fuel, &off, &gated, &settings);
    assert!(g0.encoded.art_gated && g0.encoded.art_weight == 0.0);
    // the latent mixture and the mean are the same; the measurement band is wider at every time
    assert_eq!(g.latent_quantiles, g0.latent_quantiles);
    assert_eq!(g.mean, g0.mean);
    let width = |f: &droplet4::Forecast| -> Vec<f64> {
        let (lo, hi) = (
            level(&f.levels, &f.quantiles, 0.05),
            level(&f.levels, &f.quantiles, 0.95),
        );
        lo.iter().zip(&hi).map(|(a, b)| b - a).collect()
    };
    let (w, w0) = (width(&g), width(&g0));
    for (j, (a, b)) in w.iter().zip(&w0).enumerate() {
        assert!(*a > 2.0 * b, "t = {}: 90 % band {a} vs {b}", g.times_s[j]);
    }
    // and it widens with the lead time, as art.sigma_rate_m_s says
    assert!(w.last().unwrap() > &w[0]);
    // frames dropped in the igniter window do not count
    let early = reshape(clean.clone(), |t, q| {
        if (t - 0.1).abs() < 1e-9 || (t - 0.15).abs() < 0.02 {
            0.25 * q
        } else {
            q
        }
    });
    let f = run4(fuel, &on, &early, &settings);
    assert!(f.encoded.n_gated >= 2 && f.encoded.n_gated_late == 0);
    assert!(!f.encoded.art_gated && f.encoded.art_weight == 0.0);
    // a clean prefix fires nothing and gives exactly the band without the broad component
    let c = run4(fuel, &on, &clean, &settings);
    let c0 = run4(fuel, &off, &clean, &settings);
    assert_eq!(c.encoded.n_gated, 0);
    assert!(!c.encoded.art_gated && c.encoded.art_weight == 0.0);
    assert_eq!(c.quantiles, c0.quantiles);
}

#[test]
fn the_size_and_growth_flags_read_the_prefix_head_and_its_windows() {
    let fuel = droplet4::Fuel::Methanol;
    let settings = json!({"horizon_s": 2.0, "n_points": 3});
    let flags = |d0_tol: f64, rise_tol: f64| {
        toy(
            fuel,
            &format!(
                "art.d0_tol = {d0_tol}\nart.rise_tol = {rise_tol}\nart.weight[0:4] = [0.0, 0.3, 0.2, 0.0]\n"
            ),
        )
    };
    // the toy rate is the fixture rate, so the head of the prefix carried to t = 0 is D0 itself
    let p = with_sheet_d0(prefix(fuel.label(), D0, K_HOT, 3.0, 0.21, 0.0), 1.5 * D0);
    let f = run4(fuel, &flags(0.3, 5.0), &p, &settings);
    let e = &f.encoded;
    assert!(
        (e.d0_ln - (1.0_f64 / 1.5).ln()).abs() < 1e-9,
        "d0_ln {}",
        e.d0_ln
    );
    assert!(e.art_size && !e.art_growth && !e.art_gated);
    assert!((e.art_weight - 0.3).abs() < 1e-15);
    assert!(!run4(fuel, &flags(0.5, 5.0), &p, &settings).encoded.art_size);
    // without the condition the size flag cannot fire
    let bare = prefix(fuel.label(), D0, K_HOT, 3.0, 0.21, 0.0);
    let e = run4(fuel, &flags(0.0, 5.0), &bare, &settings).encoded;
    assert!(e.d0_ln.is_nan() && !e.art_size);
    // a shrinking droplet never rises; a 4 % jump of D^2 over the last 0.4 s (under the gate's
    // tolerance, so every frame is kept) does
    assert!(e.rise < 0.0 && !e.art_growth);
    let jump = reshape(bare, |t, q| if t >= 2.6 - 1e-9 { 1.04 * q } else { q });
    let low = run4(fuel, &flags(5.0, 0.01), &jump, &settings).encoded;
    assert_eq!(low.n_gated, 0);
    assert!(low.rise > 0.01 && low.rise < 0.04, "rise {}", low.rise);
    assert!(low.art_growth && !low.art_size);
    assert!((low.art_weight - 0.2).abs() < 1e-15);
    assert!(
        !run4(fuel, &flags(5.0, 0.05), &jump, &settings)
            .encoded
            .art_growth
    );
    // two flags combine as 1 - (1 - 0.3)(1 - 0.2)
    let both = with_sheet_d0(jump, 1.5 * D0);
    let e = run4(fuel, &flags(0.3, 0.01), &both, &settings).encoded;
    assert!(e.art_size && e.art_growth);
    assert!((e.art_weight - (1.0 - 0.7 * 0.8)).abs() < 1e-15);
}

/// A diffusive extinction near 30 % burn progress (about 5.4 s for D0 = 3 mm), then evaporation with
/// a hot transient.
const GOES_OUT: &str = "diff.b0 = -0.8472978603872037\ndiff.sharp = 40.0\npost.k_inf = 0.02e-6\n\
                        post.k_hot = 0.3e-6\npost.tau0_s = 2.0\n";

#[test]
fn helium_speeds_up_the_evaporation_after_a_hot_flame_ends_and_only_with_helium() {
    let fuel = droplet4::Fuel::Methanol;
    let settings = json!({"horizon_s": 8.0, "n_points": 17});
    let plain = toy(fuel, GOES_OUT);
    let he = toy(fuel, &format!("{GOES_OUT}post.he_w = 2.0\n"));
    let in_he = prefix(fuel.label(), D0, K_HOT, 3.0, 0.21, 0.3);
    let a = run4(fuel, &plain, &in_he, &settings);
    let b = run4(fuel, &he, &in_he, &settings);
    assert!(
        a.encoded.state[0] > 0.99,
        "hot at the cutoff: {:?}",
        a.encoded.state
    );
    assert!((b.encoded.he_scale - 0.6_f64.exp()).abs() < 1e-12);
    let (ma, mb) = (
        level(&a.levels, &a.quantiles, 0.5),
        level(&b.levels, &b.quantiles, 0.5),
    );
    // identical while the flame burns, lower once the median droplet has gone out
    for (j, t) in a.times_s.iter().enumerate() {
        assert!(mb[j] <= ma[j] + 1e-15, "t = {t}: {} > {}", mb[j], ma[j]);
        if *t <= 5.0 {
            assert_eq!(mb[j], ma[j], "t = {t}");
        }
    }
    let last = ma.len() - 1;
    assert!(
        ma[last] - mb[last] > 2.0e-5,
        "median at the horizon {} vs {}",
        mb[last],
        ma[last]
    );
    // the hot-flame and state laws are untouched
    assert_eq!(a.state_probability[0], b.state_probability[0]);
    assert_eq!(a.events, b.events);
    // without helium the term is 1
    let air = prefix(fuel.label(), D0, K_HOT, 3.0, 0.21, 0.0);
    let a = run4(fuel, &plain, &air, &settings);
    let b = run4(fuel, &he, &air, &settings);
    assert_eq!(b.encoded.he_scale, 1.0);
    assert_eq!(a.quantiles, b.quantiles);
    assert_eq!(a.mean, b.mean);
}

#[test]
fn the_state_partition_sums_to_one_with_the_new_terms_on() {
    let settings = json!({"horizon_s": 10.0, "n_points": 21});
    let on = format!(
        "{GOES_OUT}{GATED}post.he_w = 3.0\npost.seen_max = 8.0\nart.d0_tol = 0.1\n\
         art.rise_tol = 0.0\nart.weight[0:4] = [0.5, 0.4, 0.3, 0.6]\n"
    );
    let on = on.replace("art.weight[0:4] = [0.5, 0.0, 0.0, 0.0]\n", "");
    let gated = |label: &str, x_he: f64| {
        with_sheet_d0(
            reshape(prefix(label, D0, K_HOT, 3.0, 0.21, x_he), |t, q| {
                if (t - 1.0).abs() < 1e-9 || (t - 2.0).abs() < 1e-9 {
                    0.25 * q
                } else {
                    q
                }
            }),
            2.0e-3,
        )
    };
    let mut runs = Vec::new();
    for fuel in [droplet4::Fuel::Methanol, droplet4::Fuel::Heptane] {
        let r = toy(fuel, &on);
        for p in [gated(fuel.label(), 0.3), gated(fuel.label(), 0.0)] {
            runs.push(run4(fuel, &r, &p, &settings));
        }
    }
    for f in &runs {
        let e = &f.encoded;
        assert!(e.art_gated && e.art_size && e.art_weight > 0.5);
    }
    // the committed candidates (heptane with cool flames) with every new term on
    let terms = "art.min_gated = 1.0
art.d0_tol = 0.1
art.rise_tol = 0.0
                 art.weight[0:4] = [0.5, 0.4, 0.3, 0.6]
art.sigma_m = 3.0e-4
                 art.sigma_rate_m_s = 5.0e-5
post.he_w = 3.0
post.seen_max = 8.0
";
    for (fuel, base, cand) in [
        (droplet4::Fuel::Methanol, M4_BASE, M4_CARRY),
        (droplet4::Fuel::Heptane, H4_BASE, H4_CARRY),
    ] {
        let (_, c) = lineage(key4(fuel), base, cand);
        let r = revise(key4(fuel), &c, terms);
        for p in equivalence_prefixes(fuel.label()) {
            runs.push(run4(fuel, &r, &p, &settings));
        }
    }
    assert!(runs
        .iter()
        .any(|f| f.state_probability[1].iter().any(|v| *v > 1e-3)));
    for f in &runs {
        let e = &f.encoded;
        assert!(
            (e.state.iter().sum::<f64>() - 1.0).abs() < 1e-12,
            "{:?}",
            e.state
        );
        assert!(f.partition_max_error < 1e-9, "{}", f.partition_max_error);
        for j in 0..f.times_s.len() {
            let s: f64 = f.state_probability.iter().map(|r| r[j]).sum();
            assert!((s - 1.0).abs() < 1e-9, "states at {}: {s}", f.times_s[j]);
            let ev: f64 = f.events.iter().map(|r| r[j]).sum();
            assert!((ev - 1.0).abs() < 1e-9, "events at {}: {ev}", f.times_s[j]);
        }
        for row in &f.quantiles {
            assert!(row.iter().all(|v| v.is_finite() && *v >= 0.0));
        }
        for l in 1..f.levels.len() {
            for j in 0..f.times_s.len() {
                assert!(f.quantiles[l][j] >= f.quantiles[l - 1][j] - 1e-15);
            }
        }
    }
}

#[test]
fn the_rate_flag_fires_on_an_end_rate_outside_its_band() {
    let fuel = droplet4::Fuel::Methanol;
    let settings = json!({"horizon_s": 2.0, "n_points": 3});
    let band = |lo: f64, hi: f64| {
        toy(
            fuel,
            &format!(
                "art.r_band[0:2] = [{lo}, {hi}]\nart.min_gated = 2.0\n\
                 art.weight[0:4] = [0.5, 0.0, 0.0, 0.7]\nunc.sigma_d = 2.0e-5\nart.sigma_m = 2.0e-4\n"
            ),
        )
    };
    // the law burns at K_HOT; a prefix burning at 3.6 times it (under gate.k_max_m2_s, so nothing is
    // gated) shows r_end = 3.6
    let fast = prefix(fuel.label(), D0, 3.6 * K_HOT, 2.0, 0.21, 0.0);
    let f = run4(fuel, &band(-1.5, 3.0), &fast, &settings);
    let e = &f.encoded;
    assert_eq!(e.n_gated, 0);
    assert!((e.r_end - 3.6).abs() < 1e-6, "r_end {}", e.r_end);
    assert!(e.art_rate && !e.art_gated && !e.art_size && !e.art_growth);
    assert!((e.art_weight - 0.7).abs() < 1e-15);
    // strictly outside: a band that holds 3.6 fires nothing, and the band is that of the flag-off
    // release
    let g = run4(fuel, &band(-1.5, 3.7), &fast, &settings);
    assert!(!g.encoded.art_rate && g.encoded.art_weight == 0.0);
    let wide = |f: &droplet4::Forecast| {
        let (lo, hi) = (
            level(&f.levels, &f.quantiles, 0.05),
            level(&f.levels, &f.quantiles, 0.95),
        );
        hi[0] - lo[0]
    };
    assert!(wide(&f) > 2.0 * wide(&g));
    assert_eq!(f.latent_quantiles, g.latent_quantiles);
    // below the band: the lower edge above r_end = 1 of a droplet burning at its law
    let lawful = prefix(fuel.label(), D0, K_HOT, 2.0, 0.21, 0.0);
    let e = run4(fuel, &band(1.1, 3.0), &lawful, &settings).encoded;
    assert!((e.r_end - 1.0).abs() < 1e-6 && e.art_rate);
    let e = run4(fuel, &band(-1.5, 3.0), &lawful, &settings).encoded;
    assert!(!e.art_rate && e.art_weight == 0.0);
    // with the gated flag as well: 1 - (1 - 0.5)(1 - 0.7)
    let both = reshape(fast, |t, q| {
        if (t - 1.0).abs() < 1e-9 || (t - 1.5).abs() < 1e-9 {
            0.25 * q
        } else {
            q
        }
    });
    let e = run4(fuel, &band(-1.5, 3.0), &both, &settings).encoded;
    assert!(e.art_gated && e.art_rate);
    assert!((e.art_weight - (1.0 - 0.5 * 0.3)).abs() < 1e-15);
}

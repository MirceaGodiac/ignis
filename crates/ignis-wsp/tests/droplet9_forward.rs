//! The droplet forward with the mechanisms of sections 3.2 and 3.3 (`droplet9.rs`): with every new
//! switch off it is `droplet8.rs`, number for number, on every committed `flex_<fuel>.wsp7` release
//! and on test-only weights; with them on, each mechanism does what the spec says on synthetic
//! prefixes: the composition input, staged burning at D_c (rate factor, flash extinction, a droplet
//! already in stage 2 at the cutoff), a warm stage after a radiative loss, a cool flame lit by the
//! igniter, fibre support. Every weight typed here is test-only; every prefix is synthetic.

mod common;

use ignis_core::contracts::{
    CheckStatus, Measured, ModelRelease, ModelSpec, PrefixBundle, RawValue, ReleaseId, Resolution,
    TensorFill,
};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::droplet9::{self, Fuel};
use ignis_wsp::{assemble, audit, droplet8, flex_tools, models, text};
use serde_json::{json, Value};

const K_HOT: f64 = 0.5e-6;
const D0: f64 = 3.0e-3;

fn config(path: &str) -> String {
    let path = format!(
        "{}/../../configs/models/flex/{path}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn dir(fuel: Fuel) -> &'static str {
    match fuel {
        Fuel::Methanol => "methanol",
        Fuel::Heptane => "heptane",
        Fuel::Octane => "octane",
        Fuel::Isooctane => "isooctane",
        Fuel::Decane => "decane",
        Fuel::Dodecane => "dodecane",
        Fuel::Farnesane => "farnesane",
        Fuel::Ethanol => "ethanol",
        Fuel::HeptaneIsooctane => "heptane_isooctane",
        Fuel::DodecaneIsododecane => "dodecane_isododecane",
        Fuel::PropanolGlycerol => "propanol_glycerol",
    }
}

/// The fuel of `droplet8.rs` with the same name, for the pure fuels.
fn fuel8(fuel: Fuel) -> Option<droplet8::Fuel> {
    droplet8::Fuel::ALL
        .into_iter()
        .find(|f| f.model_name().replace(".wsp7", ".wsp8") == fuel.model_name())
}

/// The droplet8 fuel whose tensor set a fuel uses.
fn anchor8(fuel: Fuel) -> droplet8::Fuel {
    if fuel.has_cool() {
        droplet8::Fuel::Heptane
    } else {
        droplet8::Fuel::Methanol
    }
}

fn loaded(mut release: ModelRelease, params: TensorMap) -> LoadedRelease {
    release.id = content_hash(&release, &["id", "created_at"]);
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

fn root(key: &str, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key}: {e}"));
    let patch = assemble::make_patch(&spec, common::meta("test-droplet9"), ops);
    let tensors =
        assemble::materialize(&spec, &patch, None).unwrap_or_else(|e| panic!("{key}: {e:#}"));
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("release");
    loaded(release, tensors)
}

fn revise(key: &str, parent: &LoadedRelease, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let mut meta = common::meta("test-revision");
    meta.parent_release_id = Some(ReleaseId::new(parent.release.id.clone()).unwrap());
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key} revision: {e}"));
    let patch = assemble::make_patch(&spec, meta, ops);
    let tensors = assemble::materialize(&spec, &patch, Some(parent))
        .unwrap_or_else(|e| panic!("{key} revision: {e:#}"));
    let release = model
        .build_release(&patch, &tensors, Some(parent), &common::binding())
        .expect("revision release");
    loaded(release, tensors)
}

/// The block of the committed wsp8 roots that switches the new entries off.
fn neutral_block(fuel: Fuel) -> String {
    let root = config(&format!("wsp8/{}/baseline-d2/edits.wsp", dir(fuel)));
    let at = root
        .find("\n# Entries of droplet9.rs")
        .expect("the switched-off block");
    root[at..].to_string()
}

/// A committed wsp7 chain on its own forward, and on this one with the switched-off block added to
/// the root.
fn chains(fuel: Fuel, steps: &[&str]) -> (Vec<LoadedRelease>, Vec<LoadedRelease>) {
    let d = dir(fuel);
    let (k7, k9) = (format!("flex_{d}7"), format!("flex_{d}8"));
    let (mut a, mut b): (Vec<LoadedRelease>, Vec<LoadedRelease>) = (Vec::new(), Vec::new());
    for step in steps {
        let src = config(&format!("wsp7/{d}/{step}/edits.wsp"));
        match (a.last(), b.last()) {
            (None, None) => {
                a.push(root(&k7, &src));
                b.push(root(&k9, &format!("{src}{}", neutral_block(fuel))));
            }
            (Some(pa), Some(pb)) => {
                let (ra, rb) = (revise(&k7, pa, &src), revise(&k9, pb, &src));
                a.push(ra);
                b.push(rb);
            }
            _ => unreachable!(),
        }
    }
    (a, b)
}

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

#[derive(Clone, Copy)]
struct Gas {
    x_o2: f64,
    x_co2: f64,
    x_he: f64,
    p: f64,
}

const AIR: Gas = Gas {
    x_o2: 0.21,
    x_co2: 0.0,
    x_he: 0.0,
    p: 101_325.0,
};
const HELIUM_CO2: Gas = Gas {
    x_o2: 0.21,
    x_co2: 0.10,
    x_he: 0.20,
    p: 150_000.0,
};

fn prefix(label: &str, d0: f64, cutoff: f64, gas: Gas, q: impl Fn(f64) -> f64) -> PrefixBundle {
    let mut p = flex_tools::fixture_prefix(d0, K_HOT, cutoff, gas.x_o2, gas.x_co2, gas.x_he, gas.p);
    if let Some(f) = p.known_conditions.get_mut("fuel") {
        f.raw_value = Some(RawValue::Text(label.into()));
    }
    for o in &mut p.observations {
        o.value_si = Some(q(o.time_s).sqrt());
    }
    rehash(p)
}

/// The prefix with one more numeric condition.
fn with_condition(mut p: PrefixBundle, key: &str, value: f64, unit: &str) -> PrefixBundle {
    let mut m: Measured = p.known_conditions["x_o2"].clone();
    m.raw_value = Some(RawValue::Number(value));
    m.raw_unit = Some(unit.into());
    m.value_si = Some(value);
    m.unit_si = Some(unit.into());
    m.resolution = Resolution::Resolved;
    p.known_conditions.insert(key.into(), m);
    rehash(p)
}

fn run9(
    fuel: Fuel,
    r: &LoadedRelease,
    p: &PrefixBundle,
    settings: &Value,
) -> Result<droplet9::Forecast, Vec<String>> {
    droplet9::forecast(&droplet9::spec(fuel), r, p, settings)
        .expect("runnable")
        .map_err(|s| s.reasons)
}

#[test]
fn the_specs_are_those_of_droplet8_with_one_more_input_and_the_new_entries() {
    // the law networks that read c_y
    let wide = ["rate", "crit", "diff", "rad", "cool.p", "cool.k", "cool.d"];
    for fuel in Fuel::ALL {
        let spec = droplet9::spec(fuel);
        spec.validate().expect("spec valid");
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp8/{}/spec.json", dir(fuel)))).unwrap();
        assert_eq!(on_disk.spec_hash(), spec.spec_hash(), "{}", dir(fuel));
        let old = droplet8::spec(anchor8(fuel));
        for t in &old.tensors {
            let n = spec
                .tensors
                .iter()
                .find(|x| x.name == t.name)
                .unwrap_or_else(|| panic!("{}: {} missing", dir(fuel), t.name));
            assert_eq!(
                (&n.role, &n.fill, &n.bounds),
                (&t.role, &t.fill, &t.bounds),
                "{}",
                t.name
            );
            let net = t.name.rsplit_once('.').map(|(a, _)| a).unwrap_or("");
            let last = t.name.rsplit('.').next().unwrap();
            if wide.contains(&net) && (last == "W1" || last == "skip") {
                let mut s = t.shape.clone();
                *s.last_mut().unwrap() += 1;
                assert_eq!(n.shape, s, "{}", t.name);
            } else {
                assert_eq!(n.shape, t.shape, "{}", t.name);
            }
        }
        let mut new: Vec<&str> = spec
            .tensors
            .iter()
            .filter(|t| !old.tensors.iter().any(|o| o.name == t.name))
            .map(|t| t.name.as_str())
            .collect();
        new.sort();
        let mut want = vec![
            "fib.crit",
            "fib.on",
            "fib.rate",
            "mix.on",
            "mix.rho_a",
            "mix.rho_b",
            "num.dc_count",
            "num.dc_step",
            "st.dc_sd",
            "st.dip_amp",
            "st.dip_clock",
            "st.drift",
            "st.eps",
            "st.flash.W1",
            "st.flash.b0",
            "st.flash.b1",
            "st.flash.skip",
            "st.flash.w2",
            "st.k2",
            "st.log_disruption2",
            "st.on",
        ];
        if fuel.has_cool() {
            want.extend([
                "ce.beta_y",
                "ce.g0_y",
                "ig.cool.W1",
                "ig.cool.b0",
                "ig.cool.b1",
                "ig.cool.skip",
                "ig.cool.w2",
                "ig.e_ref",
                "ig.on",
                "num.warm_count",
                "warm.k",
                "warm.on",
                "warm.p.W1",
                "warm.p.b0",
                "warm.p.b1",
                "warm.p.skip",
                "warm.p.w2",
                "warm.s.W1",
                "warm.s.b0",
                "warm.s.b1",
                "warm.s.skip",
                "warm.s.w2",
                "warm.sharp",
            ]);
        }
        want.sort();
        assert_eq!(new, want, "{}", dir(fuel));
        assert_eq!(spec.domain, old.domain);
        // the committed root passed every construction check with this forward
        let r: ModelRelease = serde_json::from_str(&config(&format!(
            "wsp8/{}/baseline-d2/release.json",
            dir(fuel)
        )))
        .unwrap();
        let c = r.construction.as_ref().unwrap();
        assert_eq!(c.forward_hash, audit::forward_hash_of(fuel.model_name()));
        assert_eq!(c.spec_hash, spec.spec_hash());
        assert!(c.checks.iter().all(|k| k.status == CheckStatus::Passed));
        let built = root(
            &format!("flex_{}8", dir(fuel)),
            &config(&format!("wsp8/{}/baseline-d2/edits.wsp", dir(fuel))),
        );
        assert_eq!(
            c.weights_hash,
            built.release.construction.as_ref().unwrap().weights_hash
        );
    }
    assert!(Fuel::HeptaneIsooctane.accepts("heptane+isooctane"));
    assert!(Fuel::HeptaneIsooctane.accepts("n-Heptane + iso-Octane"));
    assert!(Fuel::DodecaneIsododecane.accepts("dodecane+iso_dodecane"));
    assert!(Fuel::PropanolGlycerol.accepts("propanol+glycerol"));
    assert!(Fuel::PropanolGlycerol.accepts("1-Propanol + Glycerol"));
    assert!(!Fuel::Dodecane.accepts("dodecane+iso_dodecane"));
    for f in Fuel::ALL {
        assert!(f.accepts(f.label()));
        for o in Fuel::ALL.into_iter().filter(|o| *o != f) {
            assert!(!o.accepts(f.label()));
        }
    }
}

fn assert_same(a: &droplet8::Forecast, b: &droplet9::Forecast, what: &str) {
    assert_eq!(b.times_s, a.times_s, "{what}");
    assert_eq!(b.quantiles, a.quantiles, "{what}");
    assert_eq!(b.latent_quantiles, a.latent_quantiles, "{what}");
    assert_eq!(b.mean, a.mean, "{what}");
    assert_eq!(b.events, a.events, "{what}");
    assert_eq!(b.state_probability, a.state_probability, "{what}");
    assert_eq!(b.encoded.state, a.encoded.state, "{what}");
    assert_eq!(b.encoded.prior, a.encoded.prior, "{what}");
    assert_eq!(
        (b.rk4_steps, b.max_atoms),
        (a.rk4_steps, a.max_atoms),
        "{what}"
    );
}

fn prefixes(label: &str) -> Vec<PrefixBundle> {
    let hot_then = |t1: f64, k1: f64, t2: f64, k2: f64, d0: f64| {
        move |t: f64| {
            let q = d0 * d0 - K_HOT * t.min(t1);
            q - k1 * (t.min(t2) - t1).max(0.0) - k2 * (t - t2).max(0.0)
        }
    };
    let big = 4.0e-3;
    vec![
        prefix(label, D0, 3.0, AIR, |t| D0 * D0 - K_HOT * t),
        prefix(
            label,
            big,
            5.0,
            HELIUM_CO2,
            hot_then(9.0, 0.0, 9.0, 0.0, big),
        ),
        prefix(label, big, 6.0, AIR, hot_then(4.0, 0.03e-6, 9.0, 0.0, big)),
        prefix(label, big, 7.0, AIR, hot_then(4.0, 0.25e-6, 9.0, 0.0, big)),
        prefix(
            label,
            big,
            8.0,
            AIR,
            hot_then(3.0, 0.25e-6, 6.0, 0.02e-6, big),
        ),
        prefix(
            label,
            D0,
            5.0,
            HELIUM_CO2,
            hot_then(3.5, 0.02e-6, 9.0, 0.02e-6, D0),
        ),
        // inside the widened domain of droplet8.rs
        prefix(
            label,
            big,
            4.0,
            Gas {
                p: 506_600.0,
                ..AIR
            },
            |t| big * big - K_HOT * t,
        ),
        with_condition(
            prefix(label, D0, 3.0, AIR, |t| D0 * D0 - K_HOT * t),
            "fiber",
            0.0,
            "1",
        ),
    ]
}

#[test]
fn with_every_switch_off_it_is_droplet8_on_every_committed_wsp7_release() {
    let settings = json!({ "horizon_s": 14.0, "n_points": 15 });
    let late = json!({ "times_s": [10.0, 40.0, 90.0] });
    for fuel in Fuel::ALL {
        let Some(f8) = fuel8(fuel) else { continue };
        let steps: &[&str] = match fuel {
            Fuel::Methanol => &["baseline-d2", "carry-d3", "carry-p0", "carry-p1"],
            Fuel::Heptane => &["baseline-d2", "carry-d2", "carry-p0", "carry-p1"],
            _ => &["baseline-d2", "w0-fable"],
        };
        let (a, b) = chains(fuel, steps);
        for (i, (ra, rb)) in a.iter().zip(&b).enumerate() {
            assert!(rb
                .release
                .construction
                .as_ref()
                .unwrap()
                .checks
                .iter()
                .all(|k| k.status == CheckStatus::Passed));
            for p in prefixes(f8.label()) {
                for s in [&settings, &late] {
                    let x = droplet8::forecast(&droplet8::spec(f8), ra, &p, s)
                        .unwrap()
                        .unwrap_or_else(|r| panic!("{}: {:?}", dir(fuel), r.reasons));
                    let y = run9(fuel, rb, &p, s).unwrap();
                    assert_same(
                        &x,
                        &y,
                        &format!("{} {} cutoff {}", dir(fuel), steps[i], p.cutoff_s),
                    );
                    assert!(y.encoded.stage.is_none());
                }
            }
        }
    }
}

// ---- test-only weights over the switched-off heptane root of this forward

/// K_h = 0.5e-6 m2/s, no extinction, no evaporation, encoder and state evidence off, no spread.
const TOY: &str = "
rate.b0 = -0.6931471805599453
rate.skip[0:8] = 0.0
crit.b0 = 5.0
crit.skip[0:5] = 0.0
class.kappa = 20.0
class.bias = 0.0
diff.b0 = 12.0
diff.skip[0:7] = 0.0
diff.sharp = 40.0
rad.b0 = 0.0
rad.skip[0:7] = 0.0
rad.sharp = 4.0
cool.p.b0 = -30.0
cool.p.skip[0:7] = 0.0
cool.k.b0 = -1.2039728043259361
cool.k.skip[0:6] = 0.0
cool.d.b0 = -0.6931471805599453
cool.d.skip[0:6] = 0.0
cool.d.sd = 0.0
post.cool_scale = 1.0
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
rob.eps_r = 0.0
fl.trust = 0.0
fl.t_min_s = 0.3
fl.gap_s = 0.25
fl.min_points = 5.0
fl.evap_mix = 0.5
sc.on = 0.0
sc.k0_hot_mm2_s = 0.0
sc.k0_cool_mm2_s = 0.0
sc.tstar_k = 0.0
sc.cl_j_kg_k = 0.0
sc.lat_j_kg = 0.0
sc.gas[0:3] = 0.0
sc.k_cold_mm2_s = 0.0
sc.k_warm_mm2_s = 0.0
sc.tau_warm_s = 1.0
sc.diff_scale = 1.0
dy.frac = 0.0
dy.tau_s = 1.0
ce.on = 0.0
ce.g0 = 0.0
ce.beta_atm = 0.0
ce.p_ref_atm = 0.0
ce.eps_atm = 0.01
ce.c_he = 0.0
ce.d0_exp = 1.0
mix.on = 0.0
mix.rho_a = 1000.0
mix.rho_b = 1000.0
ce.beta_y = 0.0
ce.g0_y = 0.0
st.on = 0.0
st.eps = 1.0
st.dc_sd = 0.0
st.drift = 0.0
st.k2 = 0.0
st.dip_amp = 0.0
st.dip_clock = 1.0
st.flash.skip[0:7] = 0.0
st.flash.b0 = -30.0
st.log_disruption2 = -20.0
warm.on = 0.0
warm.p.skip[0:7] = 0.0
warm.p.b0 = -30.0
warm.k = 0.0
warm.s.skip[0:7] = 0.0
warm.s.b0 = 0.0
warm.sharp = 1.0
ig.on = 0.0
ig.cool.skip[0:8] = 0.0
ig.cool.b0 = -30.0
fib.on = 0.0
fib.rate = 0.0
fib.crit = 0.0
";

/// The statements of `TOY` with those of `extra` in place of the ones they name.
fn toy(extra: &str) -> LoadedRelease {
    try_toy(extra).unwrap_or_else(|e| panic!("{e}"))
}

/// As [`toy`], or why it cannot be assembled (a release cannot carry a failed build check).
fn try_toy(extra: &str) -> Result<LoadedRelease, String> {
    let name = |l: &str| -> String {
        l.trim_start()
            .split(['[', ' ', '='])
            .next()
            .unwrap_or_default()
            .to_string()
    };
    let lines: Vec<&str> = TOY.lines().chain(extra.lines()).collect();
    let mut src = String::new();
    for (i, line) in lines.iter().enumerate() {
        let n = name(line);
        if n.is_empty() || lines[i + 1..].iter().any(|l| name(l) == n) {
            continue;
        }
        src.push_str(line);
        src.push('\n');
    }
    let model = models::by_key("flex_heptane_isooctane8").unwrap();
    let spec = model.spec();
    let ops = text::compile(&src, &spec).map_err(|e| e.to_string())?;
    let patch = assemble::make_patch(&spec, common::meta("test-droplet9"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).map_err(|e| format!("{e:#}"))?;
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .map_err(|e| format!("{e:#}"))?;
    Ok(loaded(release, tensors))
}

const MIX: &str = "n-Heptane + iso-Octane";

fn median(f: &droplet9::Forecast, j: usize) -> f64 {
    f.latent_quantiles[2][j]
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol * b.abs()
}

fn burning(cutoff: f64) -> PrefixBundle {
    prefix(MIX, D0, cutoff, AIR, |t| D0 * D0 - K_HOT * t)
}

const LATE: [f64; 4] = [4.0, 5.0, 6.0, 8.0];

fn late() -> Value {
    json!({ "times_s": LATE, "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95] })
}

#[test]
fn the_composition_enters_every_law_as_its_last_input() {
    let fuel = Fuel::HeptaneIsooctane;
    // rate.skip[7] = ln 2: a droplet of pure B burns twice as fast
    let r = toy("mix.on = 1.0\nmix.rho_a = 680.0\nmix.rho_b = 690.0\nrate.skip[0:8] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.6931471805599453]\n");
    let p = with_condition(burning(3.0), "y_b_mass", 0.5, "kg/kg");
    let f = run9(fuel, &r, &p, &late()).unwrap();
    assert_eq!(f.encoded.run.y_b, 0.5);
    let k = K_HOT * 2f64.powf(0.5);
    for (j, t) in LATE.iter().enumerate() {
        let want = (f.encoded.q_cutoff_m2 - k * (t - 3.0)).sqrt();
        assert!(
            close(median(&f, j), want, 1e-9),
            "t {t}: {} {want}",
            median(&f, j)
        );
    }
    // a volume fraction is converted with the two densities
    let pv = with_condition(burning(3.0), "y_b_volume", 0.5, "m3/m3");
    let fv = run9(fuel, &r, &pv, &late()).unwrap();
    assert!(close(
        fv.encoded.run.y_b,
        690.0 * 0.5 / (680.0 * 0.5 + 690.0 * 0.5),
        1e-15
    ));
    assert_eq!(fv.encoded.run.y_basis, Some("volume"));
    // no composition, both, or out of range: refused
    for bad in [
        burning(3.0),
        with_condition(
            with_condition(burning(3.0), "y_b_mass", 0.5, "kg/kg"),
            "y_b_volume",
            0.5,
            "1",
        ),
        with_condition(burning(3.0), "y_b_mass", 1.5, "kg/kg"),
    ] {
        assert!(run9(fuel, &r, &bad, &late()).is_err());
    }
    // with mix.on = 0 the composition is not read
    let off = toy("rate.skip[0:8] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.6931471805599453]\n");
    let f0 = run9(fuel, &off, &p, &late()).unwrap();
    assert_eq!(f0.encoded.run.y_b, 0.0);
    assert!(close(
        median(&f0, 0),
        (f0.encoded.q_cutoff_m2 - K_HOT * (LATE[0] - 3.0)).sqrt(),
        1e-9
    ));
}

/// D_c of the toy staging: y_b = 0.2, st.eps = 0.5 -> D_c = D0 (1 + 3 * 0.5 * 4)^(-1/3) = D0 / 7^(1/3).
const STAGE: &str = "mix.on = 1.0\nst.on = 1.0\nst.eps = 0.5\nst.k2 = -0.6931471805599453\n";

fn staged(cutoff: f64) -> PrefixBundle {
    with_condition(burning(cutoff), "y_b_mass", 0.2, "kg/kg")
}

#[test]
fn a_hot_flame_that_reaches_d_c_burns_at_the_stage_2_rate_or_is_lost_in_a_flash() {
    let fuel = Fuel::HeptaneIsooctane;
    let q_c = D0 * D0 / 7f64.powf(2.0 / 3.0);
    let t_c = (D0 * D0 - q_c) / K_HOT; // 12.3 s
    let times = [10.0, 14.0, 16.0];
    let settings = json!({ "times_s": times });
    // no flash, no dip: K_HOT until D_c, half of it after
    let f = run9(fuel, &toy(STAGE), &staged(3.0), &settings).unwrap();
    let st = f.encoded.stage.as_ref().expect("staging");
    assert!(close(st.d_c_m[2], q_c.sqrt(), 1e-12));
    for (j, t) in times.iter().enumerate() {
        let q = if *t <= t_c {
            D0 * D0 - K_HOT * t
        } else {
            q_c - 0.5 * K_HOT * (t - t_c)
        };
        assert!(
            close(median(&f, j), q.sqrt(), 1e-6),
            "t {t}: {} {}",
            median(&f, j),
            q.sqrt()
        );
        assert!(f.state_probability[0][j] > 0.999);
    }
    // a dip: slower than the stage-2 rate right after D_c
    let dip = run9(
        fuel,
        &toy(&format!("{STAGE}st.dip_amp = 0.8\nst.dip_clock = 0.5\n")),
        &staged(3.0),
        &settings,
    )
    .unwrap();
    assert!(median(&dip, 1) > median(&f, 1));
    // a flash at D_c: the hot flame ends there and the (toy, non-evaporating) droplet stays at D_c
    let fl = run9(
        fuel,
        &toy(&format!("{STAGE}st.flash.b0 = 30.0\n")),
        &staged(3.0),
        &settings,
    )
    .unwrap();
    assert!(fl.state_probability[0][0] > 0.999);
    for j in 1..3 {
        assert!(fl.state_probability[0][j] < 1e-9 && fl.state_probability[2][j] > 0.999);
        assert!(close(median(&fl, j), q_c.sqrt(), 1e-3));
        assert!(fl.events[1][j] > 0.999);
    }
    // below D_c at the cutoff: stage 2 from the start, and a flash has already happened
    let p = with_condition(
        prefix(MIX, D0, 13.5, AIR, |t| {
            D0 * D0 - K_HOT * t.min(t_c) - 0.5 * K_HOT * (t - t_c).max(0.0)
        }),
        "y_b_mass",
        0.2,
        "kg/kg",
    );
    let g = run9(fuel, &toy(STAGE), &p, &json!({ "times_s": [14.0, 16.0] })).unwrap();
    assert!(g.encoded.stage.as_ref().unwrap().stage2_at_cutoff[2]);
    let q_cut = g.encoded.q_cutoff_m2;
    assert!(close(
        median(&g, 1),
        (q_cut - 0.5 * K_HOT * (16.0 - 13.5)).sqrt(),
        1e-6
    ));
    let gf = run9(
        fuel,
        &toy(&format!("{STAGE}st.flash.b0 = 30.0\n")),
        &p,
        &json!({ "times_s": [14.0] }),
    )
    .unwrap();
    assert!(gf.encoded.prior[0] < 1e-9, "{:?}", gf.encoded.prior);
    // y_b = 0: no stage 2
    let z = run9(
        fuel,
        &toy(STAGE),
        &with_condition(burning(3.0), "y_b_mass", 0.0, "kg/kg"),
        &settings,
    )
    .unwrap();
    assert!(z.encoded.stage.is_none());
    // st.on without mix.on is refused at binding: such a release cannot even be assembled
    assert!(try_toy("st.on = 1.0\n").is_err());
}

#[test]
fn a_cool_flame_that_reaches_d_c_burns_on_at_the_stage_2_factor_and_ends_at_d_ce() {
    let fuel = Fuel::HeptaneIsooctane;
    // every droplet lost its hot flame long before the cutoff and burns in a cool flame of
    // K_c = 0.3e-6 m2/s that ends at D_ce = 2 mm exp(0.2); D_c = D0 / 1.535^(1/3) lies between
    let cool_now = "crit.b0 = -4.0\nclass.kappa = 60.0\nclass.bias = 20.0\nrad.b0 = -3.0\n\
                    rad.sharp = 6.0\ncool.p.b0 = 30.0\ncool.d.b0 = 0.2\n";
    let r = toy(&format!("{cool_now}{STAGE}st.eps = 0.17833333333333334\n"));
    let p = with_condition(burning(3.0), "y_b_mass", 0.5, "kg/kg");
    let times = [5.0, 8.0, 14.0];
    let f = run9(fuel, &r, &p, &json!({ "times_s": times })).unwrap();
    assert!(f.encoded.state[1] > 0.999, "{:?}", f.encoded.state);
    let q_cut = f.encoded.q_cutoff_m2;
    let st = f.encoded.stage.as_ref().unwrap();
    let q_c = st.d_c_m[2] * st.d_c_m[2];
    let q_x = (2.0e-3 * 0.2f64.exp()).powi(2);
    assert!(q_x < q_c && q_c < q_cut);
    let k_c = 0.3e-6;
    let t_c = 3.0 + (q_cut - q_c) / k_c;
    let want = |t: f64| {
        if t <= t_c {
            q_cut - k_c * (t - 3.0)
        } else {
            (q_c - 0.5 * k_c * (t - t_c)).max(q_x)
        }
    };
    for (j, t) in times.iter().enumerate() {
        assert!(
            close(median(&f, j), want(*t).sqrt(), 1e-9),
            "t {t}: {} {}",
            median(&f, j),
            want(*t).sqrt()
        );
    }
    assert!(f.state_probability[1][1] > 0.999 && f.state_probability[2][2] > 0.999);
}

/// Every droplet is of the radiative class and loses its hot flame at about 6.25 s (as the
/// droplet6 tests), no cool flame follows.
const RAD_LOSS: &str = "crit.b0 = -4.0\nclass.kappa = 60.0\nclass.bias = 20.0\n\
                        rad.b0 = -0.36464311358790924\nrad.sharp = 40.0\n";

#[test]
fn a_warm_flame_after_a_radiative_loss_burns_for_its_clock_then_the_droplet_goes_on() {
    let fuel = Fuel::HeptaneIsooctane;
    let p = burning(5.0);
    let late = || json!({ "times_s": [7.0, 8.0, 10.0, 14.0] });
    let still = run9(fuel, &toy(RAD_LOSS), &p, &late()).unwrap();
    let d_loss = median(&still, 0);
    assert!((0..LATE.len()).all(|j| median(&still, j) == d_loss));
    // a warm flame every time, K_c exp(warm.k) = 0.3e-6 m2/s, median clock 1 / 9 (1 s at D0 = 3 mm)
    let warm = format!(
        "{RAD_LOSS}warm.on = 1.0\nwarm.p.b0 = 30.0\nwarm.k = 0.0\nwarm.s.b0 = -2.1972245773362196\nwarm.sharp = 20.0\n"
    );
    let f = run9(fuel, &toy(&warm), &p, &late()).unwrap();
    let a = 20.0f64;
    let rel = 1.0 + (1.0 + (-a).exp()).ln() / a;
    let dt_med = rel * (1.0 / 9.0) * D0 * D0 / 1.0e-6;
    let q_end = d_loss * d_loss - 0.3e-6 * dt_med;
    // at 8 s the median droplet has finished its warm stage and stays (no evaporation, no cool flame)
    assert!(
        close(median(&f, 2), q_end.sqrt(), 2e-3),
        "{} {}",
        median(&f, 2),
        q_end.sqrt()
    );
    assert!(median(&f, 3) < d_loss);
    // the warm flame counts as a cool flame for the state
    let during = run9(fuel, &toy(&warm), &p, &json!({ "times_s": [7.0] })).unwrap();
    assert!(
        during.state_probability[1][0] > 0.5,
        "{:?}",
        during.state_probability
    );
    assert!(median(&during, 0) < d_loss);
}

#[test]
fn a_cool_flame_lit_by_the_igniter_takes_its_share_from_the_hot_flame() {
    let fuel = Fuel::HeptaneIsooctane;
    let r = toy("ig.on = 1.0\nig.cool.b0 = 0.0\n");
    let p = with_condition(
        with_condition(burning(3.0), "igniter_power", 4.0, "W"),
        "igniter_time",
        1.5,
        "s",
    );
    let f = run9(fuel, &r, &p, &late()).unwrap();
    assert_eq!(f.encoded.run.c_ign, Some(6f64.ln()));
    // half hot, half cool (D_ce = 1 mm is below the droplet at the cutoff)
    assert!(close(f.encoded.prior[0], 0.5, 1e-12) && close(f.encoded.prior[1], 0.5, 1e-12));
    // c_ign enters its network
    let r2 = toy("ig.on = 1.0\nig.cool.b0 = 0.0\nig.cool.skip[0:8] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]\n");
    let f2 = run9(fuel, &r2, &p, &late()).unwrap();
    let want = 1.0 / (1.0 + (-(6f64.ln())).exp());
    assert!(close(f2.encoded.prior[1], want, 1e-12));
    // without the igniter conditions it is refused
    assert!(run9(fuel, &r, &burning(3.0), &late()).is_err());
}

#[test]
fn a_fibre_moves_the_rate_and_an_unsupported_release_refuses_a_supported_droplet() {
    let fuel = Fuel::HeptaneIsooctane;
    let on = toy("fib.on = 1.0\nfib.rate = 0.6931471805599453\n");
    let fixed = with_condition(burning(3.0), "fiber", 1.0, "1");
    let free = with_condition(burning(3.0), "fiber", 0.0, "1");
    let f = run9(fuel, &on, &fixed, &late()).unwrap();
    for (j, t) in LATE.iter().enumerate() {
        let want = (f.encoded.q_cutoff_m2 - 2.0 * K_HOT * (t - 3.0)).sqrt();
        assert!(
            close(median(&f, j), want, 1e-9),
            "t {t}: {} {want}",
            median(&f, j)
        );
    }
    let g = run9(fuel, &on, &free, &late()).unwrap();
    assert!(close(
        median(&g, 0),
        (g.encoded.q_cutoff_m2 - K_HOT * (LATE[0] - 3.0)).sqrt(),
        1e-9
    ));
    assert!(run9(fuel, &on, &burning(3.0), &late()).is_err());
    let off = toy("");
    assert!(run9(fuel, &off, &fixed, &late()).is_err());
    assert!(run9(fuel, &off, &free, &late()).is_ok());
}

#[test]
fn a_switch_that_is_neither_off_nor_on_is_refused() {
    let fuel = Fuel::HeptaneIsooctane;
    for s in [
        "mix.on = 0.5\n",
        "st.on = 0.5\n",
        "warm.on = 0.5\n",
        "ig.on = 0.5\n",
        "fib.on = 0.5\n",
    ] {
        assert!(try_toy(s).is_err(), "{s} assembled");
    }
    assert!(run9(fuel, &toy(""), &burning(3.0), &late()).is_ok());
    // the constants the spec fixes
    let spec = droplet9::spec(fuel);
    let c = |n: &str| match &spec.tensors.iter().find(|t| t.name == n).unwrap().fill {
        TensorFill::Fixed { values } => values[0].get(),
        _ => panic!(),
    };
    assert_eq!(
        (
            c("num.dc_count"),
            c("num.dc_step"),
            c("num.warm_count"),
            c("ig.e_ref")
        ),
        (5.0, 1.0, 5.0, 1.0)
    );
}

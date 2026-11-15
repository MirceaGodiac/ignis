//! The sixth droplet forward (`droplet6.rs`) on synthetic prefixes: with its three switches off it
//! is `droplet5.rs`, number for number, on test-only weights that reach every branch and on the
//! committed carries; with `sc.on = 1` an unlit droplet follows the self-cooling closed form after
//! a hot flame, after a diffusive extinction (`sc.diff_scale`), after a cool flame and at the
//! cutoff, with and without the gas factor; the dying term stands only after a radiative loss that
//! no cool flame follows; with `ce.on = 1` the cool flame ends where the closed form in oxygen
//! partial pressure says, capped near `ce.beta_atm`, and a cool flame of zero length leaves a
//! droplet that comes straight from the hot flame. Every weight typed here is test-only.

mod common;

use ignis_core::contracts::{
    BuildCheckKind, CheckStatus, ModelRelease, ModelSpec, PrefixBundle, Quantity, RawValue,
    ReleaseId,
};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::droplet6::{self, Forecast, Fuel};
use ignis_wsp::{assemble, audit, droplet5, flex_tools, models, text};
use serde_json::{json, Value};

const D0: f64 = 3.0e-3;
const D_REF: f64 = 2.0e-3;
/// Hot-flame rate of the toy weights and of the fixture prefixes.
const K_HOT: f64 = 0.5e-6;
/// Cool-flame rate of the toy weights.
const K_COOL: f64 = 0.3e-6;
const CUTOFF: f64 = 5.0;

/// The toy weights of the earlier droplet tests: K_h = 0.5e-6 m2/s, no extinction of either kind,
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

/// K_c = 0.3e-6 m2/s, no cool flame after a radiative loss, D_ce = 1 mm.
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

/// The entries `droplet5.rs` added, switched off.
const OFF5: &str = "
rob.eps_r = 0.0
fl.trust = 0.0
fl.t_min_s = 0.3
fl.gap_s = 0.25
fl.min_points = 5.0
fl.evap_mix = 0.5
";

/// The entries this forward adds, switched off.
const OFF6: &str = "
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
";

// ---- test-only values of the self-cooling law
const K0_HOT: f64 = 0.4e-6;
const K0_COOL: f64 = 0.2e-6;
const TSTAR: f64 = 30.0;
const CL: f64 = 2200.0;
const LAT: f64 = 3.2e5;
const K_COLD: f64 = 0.01e-6;
const K_WARM: f64 = 0.05e-6;
const TAU_WARM: f64 = 4.0;
const SC: &str = "sc.on = 1.0\nsc.k0_hot_mm2_s = 0.4\nsc.k0_cool_mm2_s = 0.2\nsc.tstar_k = 30.0\n\
                  sc.cl_j_kg_k = 2200.0\nsc.lat_j_kg = 320000.0\nsc.k_cold_mm2_s = 0.01\n\
                  sc.k_warm_mm2_s = 0.05\nsc.tau_warm_s = 4.0\n";
const GAS_COEF: [f64; 3] = [1.5, -0.8, 0.5];
const SC_GAS: &str = "sc.gas[0:3] = [1.5, -0.8, 0.5]\n";

// ---- test-only values of the dying flame
const DY_FRAC: f64 = 0.6;
const DY_TAU: f64 = 1.5;
const DY: &str = "dy.frac = 0.6\ndy.tau_s = 1.5\n";

/// Every droplet is of the radiative class (D_crit = 2 mm x exp(-4), far below any droplet) and
/// loses its hot flame at about 6.25 s (D0 = 3 mm: s_R = 6.25 / 9), most of them inside the
/// extinction bin [6.0, 6.5] s.
const RAD_LOSS: &str = "crit.b0 = -4.0\nclass.kappa = 60.0\nclass.bias = 20.0\n\
                        rad.b0 = -0.36464311358790924\nrad.sharp = 40.0\n";
/// Every droplet is of the diffusive class (the toy D_crit = 2 mm x exp(5), far above any droplet)
/// and its hot flame burns out at about 6.25 s (progress 0.347 at K_h = 0.5e-6 m2/s), most of
/// them inside the extinction bin [6.0, 6.5] s.
const DIFF_LOSS: &str = "class.kappa = 60.0\nclass.bias = -20.0\ndiff.b0 = -0.6312717768418578\n\
                         diff.sharp = 80.0\n";
/// Every droplet lost its hot flame to radiation long before the cutoff and burns in a cool flame
/// that ends at D_ce = 2 mm x exp(0.2).
const COOL_NOW: &str = "crit.b0 = -4.0\nclass.kappa = 60.0\nclass.bias = 20.0\nrad.b0 = -3.0\n\
                        rad.sharp = 6.0\ncool.p.b0 = 30.0\ncool.d.b0 = 0.2\n";

/// The ambient gas of a fixture prefix.
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

impl Gas {
    /// The gas factor of the self-cooling law, as the author wrote it.
    fn factor(self) -> f64 {
        (GAS_COEF[0] * self.x_he + GAS_COEF[1] * self.x_co2 + GAS_COEF[2] * (self.p / 1.0e5).ln())
            .exp()
    }
}

/// The statements of `base` and of `extra`, one per line, in which the last statement of a tensor
/// stands for that tensor (a patch writes each element once).
fn source(base: &str, extra: &str) -> String {
    let name = |l: &str| -> String {
        let n = l.trim_start().split(['[', ' ', '=']).next();
        n.unwrap_or_default().to_string()
    };
    let lines: Vec<&str> = base.lines().chain(extra.lines()).collect();
    let mut src = String::new();
    for (i, line) in lines.iter().enumerate() {
        let n = name(line);
        if n.is_empty() || lines[i + 1..].iter().any(|l| name(l) == n) {
            continue;
        }
        src.push_str(line);
        src.push('\n');
    }
    src
}

/// An in-memory release, with the content id the registry would give it (a revision names it).
fn loaded(mut release: ModelRelease, params: TensorMap) -> LoadedRelease {
    release.id = content_hash(&release, &["id", "created_at"]);
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

/// The root release of a model from the edits `src`, built in memory.
fn root(key: &str, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key}: {e}\n{src}"));
    let patch = assemble::make_patch(&spec, common::meta("test-droplet6"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("release");
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

fn toy(cool: bool) -> String {
    if cool {
        format!("{TOY}{TOY_COOL}{OFF5}")
    } else {
        format!("{TOY}{OFF5}")
    }
}

fn key6(fuel: Fuel) -> &'static str {
    match fuel {
        Fuel::Methanol => "flex_methanol5",
        Fuel::Heptane => "flex_heptane5",
    }
}

fn fuel5(fuel: Fuel) -> (droplet5::Fuel, &'static str) {
    match fuel {
        Fuel::Methanol => (droplet5::Fuel::Methanol, "flex_methanol4"),
        Fuel::Heptane => (droplet5::Fuel::Heptane, "flex_heptane4"),
    }
}

/// The toy release of this forward: the new entries off, then `extra` over everything.
fn release6(fuel: Fuel, extra: &str) -> LoadedRelease {
    root(
        key6(fuel),
        &source(&format!("{}{OFF6}", toy(fuel.has_cool())), extra),
    )
}

/// The same toy release on `droplet5.rs`.
fn release5(fuel: Fuel, extra: &str) -> LoadedRelease {
    root(fuel5(fuel).1, &source(&toy(fuel.has_cool()), extra))
}

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

/// A droplet of `fuel` and initial diameter `d0` whose D^2 is `q(t)`, observed at 30 Hz to
/// `cutoff`, in `gas`.
fn prefix(fuel: Fuel, d0: f64, cutoff: f64, gas: Gas, q: impl Fn(f64) -> f64) -> PrefixBundle {
    let mut p = flex_tools::fixture_prefix(d0, K_HOT, cutoff, gas.x_o2, gas.x_co2, gas.x_he, gas.p);
    if let Some(f) = p.known_conditions.get_mut("fuel") {
        f.raw_value = Some(RawValue::Text(fuel.label().into()));
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

fn run6(fuel: Fuel, r: &LoadedRelease, p: &PrefixBundle, settings: &Value) -> Forecast {
    droplet6::forecast(&droplet6::spec(fuel), r, p, settings)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn run5(fuel: Fuel, r: &LoadedRelease, p: &PrefixBundle, settings: &Value) -> droplet5::Forecast {
    droplet5::forecast(&droplet5::spec(fuel5(fuel).0), r, p, settings)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

/// The toy forecast of this forward for `extra` over the switched-off toy weights.
fn toy6(fuel: Fuel, extra: &str, p: &PrefixBundle) -> Forecast {
    run6(fuel, &release6(fuel, extra), p, &late())
}

fn q_law(t: f64) -> f64 {
    D0 * D0 - K_HOT * t
}

/// Hot flame until 3.5 s, then a D^2 falling at `k` m2/s.
fn q_out(k: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        if t <= 3.5 {
            q_law(t)
        } else {
            q_law(3.5) - k * (t - 3.5)
        }
    }
}

/// Forecast times after the extinction bin [6.0, 6.5] s has closed.
const LATE: [f64; 4] = [7.0, 8.0, 10.0, 14.0];

fn late() -> Value {
    json!({ "times_s": LATE, "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95] })
}

/// The latent median diameter at forecast time `j`.
fn median(f: &Forecast, j: usize) -> f64 {
    f.latent_quantiles[2][j]
}

fn close(have: f64, want: f64) -> bool {
    (have - want).abs() <= 1.0e-9 * want.abs()
}

fn assert_follows(f: &Forecast, what: &str, want: impl Fn(f64) -> f64) {
    for (j, &t) in f.times_s.iter().enumerate() {
        let (have, want) = (median(f, j), want(t));
        assert!(close(have, want), "{what}, t = {t} s: {have:e} != {want:e}");
    }
}

/// The amplitudes of the self-cooling law in m2/s, gas factor and diffusive factor applied.
#[derive(Clone, Copy)]
struct Rates {
    k0: f64,
    cold: f64,
    warm: f64,
}

fn after_hot(g: f64) -> Rates {
    Rates {
        k0: g * K0_HOT,
        cold: g * K_COLD,
        warm: K_WARM,
    }
}

fn after_cool(g: f64) -> Rates {
    Rates {
        k0: g * K0_COOL,
        ..after_hot(g)
    }
}

fn times(r: Rates, f: f64) -> Rates {
    Rates {
        k0: f * r.k0,
        cold: f * r.cold,
        warm: f * r.warm,
    }
}

/// The author's closed form: fall of D^2 over the first `u` seconds after the flame left the
/// droplet at `q_e`.
fn sc_fall(r: Rates, q_e: f64, u: f64) -> f64 {
    let heat = if r.k0 > 0.0 {
        let t0 = TSTAR * CL * q_e / (1.5 * r.k0 * LAT);
        r.k0 * t0 * (1.0 + u / t0).ln()
    } else {
        0.0
    };
    heat + r.cold * u + (r.warm - r.cold) * TAU_WARM * (1.0 - (-u / TAU_WARM).exp())
}

/// The author's dying term: fall of D^2 over the first `u` seconds.
fn die_fall(u: f64) -> f64 {
    DY_FRAC * K_COOL * DY_TAU * (1.0 - (-u / DY_TAU).exp())
}

/// Where most droplets lose their hot flame under the loss weights `loss`: the time and D^2 of the
/// extinction bin that holds the median. The toy unlit droplet does not shrink, so the forecast
/// shows D^2 at the loss, and the toy hot flame burns at K_HOT from the diameter at the cutoff.
fn loss_point(fuel: Fuel, loss: &str, p: &PrefixBundle) -> (f64, f64) {
    let still = toy6(fuel, loss, p);
    let d = median(&still, 0);
    assert!((0..LATE.len()).all(|j| median(&still, j) == d));
    let q_e = d * d;
    let t_e = still.encoded.cutoff_s + (still.encoded.q_cutoff_m2 - q_e) / K_HOT;
    assert!(t_e > 6.0 && t_e < 6.5, "loss at {t_e} s");
    (t_e, q_e)
}

fn config(path: &str) -> String {
    let path = format!(
        "{}/../../configs/models/flex/{path}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The committed lineage root and carry of a generation (`wsp4` on `droplet5.rs`, `wsp5` here).
fn committed(
    key: &str,
    generation: &str,
    dir: &str,
    carry: &str,
) -> (LoadedRelease, LoadedRelease) {
    let base = root(
        key,
        &config(&format!("{generation}/{dir}/baseline-d2/edits.wsp")),
    );
    let cand = revise(
        key,
        &base,
        &config(&format!("{generation}/{dir}/{carry}/edits.wsp")),
    );
    (base, cand)
}

/// The names of the tensors this forward adds, in the order of the spec.
const ADDED: [&str; 20] = [
    "sc.on",
    "sc.k0_hot_mm2_s",
    "sc.k0_cool_mm2_s",
    "sc.tstar_k",
    "sc.cl_j_kg_k",
    "sc.lat_j_kg",
    "sc.gas",
    "sc.k_cold_mm2_s",
    "sc.k_warm_mm2_s",
    "sc.tau_warm_s",
    "sc.diff_scale",
    "dy.frac",
    "dy.tau_s",
    "ce.on",
    "ce.g0",
    "ce.beta_atm",
    "ce.p_ref_atm",
    "ce.eps_atm",
    "ce.c_he",
    "ce.d0_exp",
];

#[test]
fn the_committed_specs_are_what_the_forward_was_compiled_with() {
    for (fuel, dir) in [(Fuel::Methanol, "methanol"), (Fuel::Heptane, "heptane")] {
        let spec = droplet6::spec(fuel);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(key6(fuel)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp5/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/flex/wsp5/{dir}/spec.json drifted from droplet6::spec(); a changed spec \
             is a new spec id (re-export with `ignis wsp spec --family {} --out …`), never an \
             in-place edit",
            key6(fuel)
        );
        // every tensor of droplet5.rs, unchanged and in its order, then the new entries
        let old = droplet5::spec(fuel5(fuel).0);
        let n = old.tensors.len();
        assert_eq!(&spec.tensors[..n], &old.tensors[..]);
        let added: Vec<&str> = spec.tensors[n..].iter().map(|t| t.name.as_str()).collect();
        assert_eq!(added, ADDED);
        assert_eq!(spec.features, old.features);
        assert_eq!(spec.domain, old.domain);
    }
}

#[test]
fn with_its_switches_off_it_is_droplet5_on_weights_that_reach_every_branch() {
    // an unlit droplet that evaporates, a cool flame half of the time with a spread wide enough
    // for cool flames of zero length, some spread and noise, the state evidence on
    let busy = "filt.gain_b = 10.0\npost.k_inf = 2.0e-8\npost.k_hot = 1.0e-7\nunc.s_floor = 0.05\n\
                unc.sigma_d = 1.0e-5\n";
    let cool_half = "cool.p.b0 = 0.0\ncool.d.b0 = 0.1\ncool.d.sd = 0.3\npost.cool_scale = 0.4\n";
    let cameras = "fl.trust = 0.9\nrob.eps_r = 0.1\n";
    let settings = json!({ "horizon_s": 12.0, "n_points": 25 });
    for fuel in [Fuel::Methanol, Fuel::Heptane] {
        let cool = if fuel.has_cool() { cool_half } else { "" };
        let weights = [
            format!("diff.b0 = 0.0\ndiff.sharp = 6.0\n{busy}{cool}"),
            format!("{RAD_LOSS}{busy}{cool}"),
            format!("{RAD_LOSS}{busy}{cool}{cameras}"),
            format!("{DIFF_LOSS}{busy}{cool}{cameras}"),
            // half of the droplets in each class, both kinds of extinction in the forecast
            format!(
                "{RAD_LOSS}{DIFF_LOSS}class.kappa = 0.0\nclass.bias = 0.0\ndiff.sharp = 20.0\n\
                 {busy}{cool}{cameras}"
            ),
            if fuel.has_cool() {
                format!("{COOL_NOW}{busy}cool.d.sd = 0.3\npost.cool_scale = 0.4\n")
            } else {
                format!("{RAD_LOSS}rad.b0 = -3.0\n{busy}")
            },
        ];
        let burning = prefix(fuel, D0, CUTOFF, AIR, q_law);
        let out = prefix(fuel, D0, CUTOFF, HELIUM_CO2, q_out(0.02e-6));
        let prefixes = [
            burning.clone(),
            out.clone(),
            with_flame(burning, frames(0.3, 3.0)),
            with_flame(out, frames(0.3, CUTOFF)),
        ];
        let (mut zero_length, mut unlit, mut cool_flame) = (false, false, false);
        for extra in &weights {
            let (old, new) = (release5(fuel, extra), release6(fuel, extra));
            for p in &prefixes {
                let (a, b) = (
                    run5(fuel, &old, p, &settings),
                    run6(fuel, &new, p, &settings),
                );
                assert_eq!(b.quantiles, a.quantiles);
                assert_eq!(b.latent_quantiles, a.latent_quantiles);
                assert_eq!(b.mean, a.mean);
                assert_eq!(b.events, a.events);
                assert_eq!(b.state_probability, a.state_probability);
                assert_eq!(b.encoded.state, a.encoded.state);
                assert_eq!(b.encoded.prior, a.encoded.prior);
                assert_eq!(b.encoded.laws.d_ce_m, a.encoded.laws.d_ce_m);
                assert_eq!((b.rk4_steps, b.max_atoms), (a.rk4_steps, a.max_atoms));
                let added = &b.encoded.added;
                assert!(
                    added.self_cooling.is_none()
                        && added.dying.is_none()
                        && added.cool_end.is_none()
                );
                assert_eq!(b.unlit_at_cutoff.law, "post");
                // what the cases reached
                let l = &b.encoded.laws;
                zero_length |= l.p_cool > 0.1
                    && l.d_ce_m * (2.0 * l.sd_ce).exp() > b.encoded.q_cutoff_m2.sqrt();
                unlit |= b.state_probability[2].iter().any(|v| *v > 0.5);
                cool_flame |= b.state_probability[1].iter().any(|v| *v > 0.1);
            }
        }
        assert!(unlit, "no case reached a droplet without a flame");
        if fuel.has_cool() {
            assert!(cool_flame, "no heptane case reached a cool flame");
            assert!(
                zero_length,
                "no heptane case had a cool flame of zero length"
            );
        }
    }
}

#[test]
fn the_committed_carries_give_the_forecasts_of_the_carries_they_copy() {
    let settings = json!({ "horizon_s": 12.0, "n_points": 25 });
    for (fuel, dir, carry) in [
        (Fuel::Methanol, "methanol", "carry-d3"),
        (Fuel::Heptane, "heptane", "carry-d2"),
    ] {
        let old = committed(fuel5(fuel).1, "wsp4", dir, carry);
        let new = committed(key6(fuel), "wsp5", dir, carry);
        // every construction check of the assembled releases passed
        for r in [&new.0, &new.1] {
            let c = r.release.construction.as_ref().expect("a literal release");
            assert!(
                c.checks.iter().all(|k| k.status == CheckStatus::Passed),
                "{:?}",
                c.checks
            );
            for kind in [
                BuildCheckKind::SpecValid,
                BuildCheckKind::PatchValid,
                BuildCheckKind::TensorsComplete,
                BuildCheckKind::StrictForwardAudit,
                BuildCheckKind::FiniteForwardOnFixture,
            ] {
                assert!(c.checks.iter().any(|k| k.check == kind), "{kind:?} missing");
            }
        }
        // the tensors both generations have are equal, and the new ones hold the switched-off block
        for (a, b) in [(&old.0, &new.0), (&old.1, &new.1)] {
            for (name, t) in &a.params {
                assert_eq!(&b.params[name], t, "{dir}: {name} differs from wsp4");
            }
            assert_eq!(b.params.len(), a.params.len() + ADDED.len());
            for name in ["sc.on", "dy.frac", "ce.on"] {
                assert_eq!(b.params[name].data, [0.0], "{dir}: {name} is not off");
            }
        }
        // the committed release records hold these weights, were assembled with this forward and
        // passed every construction check
        for (step, built) in [("baseline-d2", &new.0), (carry, &new.1)] {
            let path = format!("wsp5/{dir}/{step}/release.json");
            let on_disk: ModelRelease =
                serde_json::from_str(&config(&path)).unwrap_or_else(|e| panic!("{path}: {e}"));
            let c = on_disk.construction.as_ref().expect("a literal release");
            let b = built.release.construction.as_ref().unwrap();
            assert_eq!(c.weights_hash, b.weights_hash, "{path}: other weights");
            assert_eq!(c.spec_hash, droplet6::spec(fuel).spec_hash(), "{path}");
            assert_eq!(
                c.forward_hash,
                audit::forward_hash_of(fuel.model_name()),
                "{path} was assembled with another droplet6.rs: a forward is frozen with its \
                 releases (re-assemble with tools/wsp/reassemble.py if it has none in use)"
            );
            assert!(
                c.checks.iter().all(|k| k.status == CheckStatus::Passed),
                "{path}: {:?}",
                c.checks
            );
        }
        // prefixes that end in a hot flame, after its loss, in a slower burn (a cool flame, for
        // heptane) and after that, in air and in a diluted atmosphere, small and large droplets
        let hot_then = |t1: f64, k1: f64, t2: f64, k2: f64, d0: f64| {
            move |t: f64| {
                let q = d0 * d0 - K_HOT * t.min(t1);
                q - k1 * (t.min(t2) - t1).max(0.0) - k2 * (t - t2).max(0.0)
            }
        };
        let big = 4.0e-3;
        let burning = prefix(fuel, D0, 3.0, AIR, q_law);
        let lost = prefix(fuel, big, 6.0, AIR, hot_then(4.0, 0.03e-6, 9.0, 0.0, big));
        let prefixes = [
            burning.clone(),
            prefix(
                fuel,
                big,
                5.0,
                HELIUM_CO2,
                hot_then(9.0, 0.0, 9.0, 0.0, big),
            ),
            lost.clone(),
            prefix(fuel, big, 7.0, AIR, hot_then(4.0, 0.25e-6, 9.0, 0.0, big)),
            prefix(
                fuel,
                big,
                8.0,
                AIR,
                hot_then(3.0, 0.25e-6, 6.0, 0.02e-6, big),
            ),
            prefix(fuel, D0, CUTOFF, HELIUM_CO2, q_out(0.02e-6)),
            with_flame(burning, frames(0.3, 3.0)),
            with_flame(lost, frames(0.3, 4.0)),
        ];
        let mut reached = [false; 3];
        for (a, b) in [(&old.0, &new.0), (&old.1, &new.1)] {
            for p in &prefixes {
                let (f5, f6) = (run5(fuel, a, p, &settings), run6(fuel, b, p, &settings));
                assert_eq!(f6.quantiles, f5.quantiles, "{dir}, cutoff {}", p.cutoff_s);
                assert_eq!(f6.state_probability, f5.state_probability);
                assert_eq!(f6.latent_quantiles, f5.latent_quantiles);
                assert_eq!(f6.mean, f5.mean);
                assert_eq!(f6.events, f5.events);
                assert_eq!(f6.encoded.state, f5.encoded.state);
                for (i, r) in reached.iter_mut().enumerate() {
                    *r |= f6.state_probability[i].iter().any(|v| *v > 0.3);
                }
            }
        }
        assert!(reached[0], "{dir}: no case with a hot flame");
        assert!(reached[2], "{dir}: no case without a flame");
        if fuel.has_cool() {
            assert!(reached[1], "{dir}: no case with a cool flame");
        }
    }
}

#[test]
fn an_unlit_droplet_follows_the_self_cooling_law_after_a_hot_flame() {
    // a hot flame lost to radiation in the forecast: K0 from sc.k0_hot_mm2_s, u from the loss
    for (gas, coef) in [(AIR, ""), (HELIUM_CO2, SC_GAS)] {
        let g = if coef.is_empty() { 1.0 } else { gas.factor() };
        let p = prefix(Fuel::Methanol, D0, CUTOFF, gas, q_law);
        let (t_e, q_e) = loss_point(Fuel::Methanol, RAD_LOSS, &p);
        let f = toy6(Fuel::Methanol, &format!("{RAD_LOSS}{SC}{coef}"), &p);
        assert_follows(&f, "after a radiative loss", |t| {
            (q_e - sc_fall(after_hot(g), q_e, t - t_e)).sqrt()
        });
        // the law is at work: the droplet has lost a tenth of its D^2 or more by 14 s
        assert!(median(&f, 3).powi(2) < 0.9 * q_e);
        // and the diagnostics show it
        let sc = f.encoded.added.self_cooling.expect("the law is on");
        assert!(close(sc.gas_factor, g));
        assert!(close(sc.k0_hot_m2_s, g * K0_HOT));
        assert!(close(sc.k_floor_end_m2_s, g * K_COLD));
        assert_eq!(sc.k_floor_start_m2_s, K_WARM);
        let q_cut = f.encoded.q_cutoff_m2;
        assert!(close(
            sc.t0_hot_s.unwrap(),
            TSTAR * CL * q_cut / (1.5 * g * K0_HOT * LAT)
        ));
    }
    // without the amplitude the first term is absent and the floor alone is left
    let p = prefix(Fuel::Methanol, D0, CUTOFF, AIR, q_law);
    let (t_e, q_e) = loss_point(Fuel::Methanol, RAD_LOSS, &p);
    let f = toy6(
        Fuel::Methanol,
        &format!("{RAD_LOSS}{SC}sc.k0_hot_mm2_s = 0.0\n"),
        &p,
    );
    let floor = Rates {
        k0: 0.0,
        ..after_hot(1.0)
    };
    assert_follows(&f, "the floor alone", |t| {
        (q_e - sc_fall(floor, q_e, t - t_e)).sqrt()
    });
}

#[test]
fn a_diffusive_extinction_scales_the_self_cooling_law_and_a_radiative_loss_does_not() {
    let half = format!("{SC}{SC_GAS}sc.diff_scale = 0.5\n");
    let whole = format!("{SC}{SC_GAS}");
    let p = prefix(Fuel::Methanol, D0, CUTOFF, HELIUM_CO2, q_law);
    let g = HELIUM_CO2.factor();
    // burn-out: half of K0 and half of both floors
    let (t_e, q_e) = loss_point(Fuel::Methanol, DIFF_LOSS, &p);
    let f = toy6(Fuel::Methanol, &format!("{DIFF_LOSS}{half}"), &p);
    assert_follows(&f, "after a diffusive extinction, scale 0.5", |t| {
        (q_e - sc_fall(times(after_hot(g), 0.5), q_e, t - t_e)).sqrt()
    });
    let f = toy6(Fuel::Methanol, &format!("{DIFF_LOSS}{whole}"), &p);
    assert_follows(&f, "after a diffusive extinction, scale 1", |t| {
        (q_e - sc_fall(after_hot(g), q_e, t - t_e)).sqrt()
    });
    let sc = toy6(Fuel::Methanol, &format!("{DIFF_LOSS}{half}"), &p)
        .encoded
        .added
        .self_cooling
        .unwrap();
    assert!(close(sc.k0_hot_diffusive_m2_s, 0.5 * g * K0_HOT));
    assert!(close(sc.k0_hot_m2_s, g * K0_HOT));
    // a radiative loss: the factor is not applied
    let a = toy6(Fuel::Methanol, &format!("{RAD_LOSS}{half}"), &p);
    let b = toy6(Fuel::Methanol, &format!("{RAD_LOSS}{whole}"), &p);
    assert_eq!(a.quantiles, b.quantiles);
    assert_eq!(a.latent_quantiles, b.latent_quantiles);
    // nor after a cool flame, nor with the law off
    let p = prefix(Fuel::Heptane, D0, CUTOFF, HELIUM_CO2, q_law);
    let a = toy6(Fuel::Heptane, &format!("{COOL_NOW}{half}"), &p);
    let b = toy6(Fuel::Heptane, &format!("{COOL_NOW}{whole}"), &p);
    assert_eq!(a.latent_quantiles, b.latent_quantiles);
    let a = toy6(
        Fuel::Methanol,
        &format!("{DIFF_LOSS}sc.diff_scale = 0.5\n"),
        &p_methanol(),
    );
    let b = toy6(Fuel::Methanol, DIFF_LOSS, &p_methanol());
    assert_eq!(a.latent_quantiles, b.latent_quantiles);
}

fn p_methanol() -> PrefixBundle {
    prefix(Fuel::Methanol, D0, CUTOFF, AIR, q_law)
}

#[test]
fn an_unlit_droplet_follows_the_self_cooling_law_after_a_cool_flame() {
    let settings = json!({ "times_s": [6.0, 7.0, 8.0, 10.0, 14.0] });
    for (gas, coef) in [(AIR, ""), (HELIUM_CO2, SC_GAS)] {
        let g = if coef.is_empty() { 1.0 } else { gas.factor() };
        let p = prefix(Fuel::Heptane, D0, CUTOFF, gas, q_law);
        let r = release6(Fuel::Heptane, &format!("{COOL_NOW}{SC}{coef}"));
        let f = run6(Fuel::Heptane, &r, &p, &settings);
        assert!(f.encoded.state[1] > 1.0 - 1e-8, "{:?}", f.encoded.state);
        // the cool flame burns at K_c from the cutoff to D_ce, then the droplet is unlit
        let q_cut = f.encoded.q_cutoff_m2;
        let q_x = (D_REF * 0.2f64.exp()).powi(2);
        let t_x = CUTOFF + (q_cut - q_x) / K_COOL;
        assert!(t_x > 6.5 && t_x < 7.0, "{t_x}");
        assert_follows(&f, "after a cool flame", |t| {
            if t <= t_x {
                (q_cut - K_COOL * (t - CUTOFF)).sqrt()
            } else {
                (q_x - sc_fall(after_cool(g), q_x, t - t_x)).sqrt()
            }
        });
        assert!(f.state_probability[1][0] > 1.0 - 1e-8 && f.state_probability[1][1] < 1e-8);
        let sc = f.encoded.added.self_cooling.unwrap();
        assert!(close(sc.k0_cool_m2_s, g * K0_COOL));
    }
}

#[test]
fn a_droplet_unlit_at_the_cutoff_restarts_the_law_or_runs_it_from_the_time_the_flame_was_lost() {
    // the cameras lost the flame at 3 s and are trusted: the droplet has no flame at the cutoff
    let p = with_flame(
        prefix(Fuel::Methanol, D0, CUTOFF, HELIUM_CO2, q_out(0.33e-6)),
        frames(0.3, 3.0),
    );
    let g = HELIUM_CO2.factor();
    let hot = after_hot(g);
    let cameras = format!("{SC}{SC_GAS}fl.trust = 1.0\n");
    // all weight on the time the flame was lost: u runs from t_last, D_e is the diameter at the cutoff
    let f = toy6(Fuel::Methanol, &format!("{cameras}fl.evap_mix = 1.0\n"), &p);
    assert_eq!(f.encoded.state, [0.0, 0.0, 1.0]);
    let (q_cut, t_x) = (f.encoded.q_cutoff_m2, f.encoded.flame.t_last_s.unwrap());
    assert!((t_x - 3.0).abs() < 0.04);
    let since_lost = |t: f64| sc_fall(hot, q_cut, t - t_x) - sc_fall(hot, q_cut, CUTOFF - t_x);
    assert_follows(&f, "from the time the flame was lost", |t| {
        (q_cut - since_lost(t)).sqrt()
    });
    assert_eq!(f.unlit_at_cutoff.law, "self_cooling");
    assert_eq!(f.unlit_at_cutoff.origin, "hot_flame");
    assert!(close(
        f.unlit_at_cutoff.since_flame_lost_s.unwrap(),
        CUTOFF - t_x
    ));
    // no weight on it, state evidence off: the law restarts at the cutoff with half of K0
    let f = toy6(Fuel::Methanol, &format!("{cameras}fl.evap_mix = 0.0\n"), &p);
    let restart = |share: f64| Rates {
        k0: share * hot.k0,
        ..hot
    };
    assert_follows(&f, "restarted with half of K0", |t| {
        (q_cut - sc_fall(restart(0.5), q_cut, t - CUTOFF)).sqrt()
    });
    assert_eq!(f.unlit_at_cutoff.share_from_end_rate, 0.5);
    // state evidence on: the share of K0 is what the end rate shows above the warm floor
    let seen = format!("{cameras}fl.evap_mix = 0.25\nfilt.gain_b = 50.0\nfilt.l_floor = 1.0\n");
    let f = toy6(Fuel::Methanol, &seen, &p);
    assert_eq!(f.encoded.state, [0.0, 0.0, 1.0]);
    let share = (f.encoded.k_obs_end_m2_s - K_WARM) / hot.k0;
    assert!(share > 0.3 && share < 0.6, "{share}");
    assert!(close(f.unlit_at_cutoff.share_from_end_rate, share));
    assert!(close(f.unlit_at_cutoff.k0_m2_s.unwrap(), share * hot.k0));
    assert_follows(&f, "a quarter from the time the flame was lost", |t| {
        let by_rate = sc_fall(restart(share), q_cut, t - CUTOFF);
        (q_cut - (0.25 * since_lost(t) + 0.75 * by_rate)).sqrt()
    });
    // the state filter expects the unlit droplet at the floor plus half of K0
    // (half of the droplets burning at the cutoff by the laws, the evidence decides)
    let filter = "class.kappa = 60.0\nclass.bias = -20.0\ndiff.b0 = -1.04\ndiff.sharp = 6.0\n\
                  filt.gain_b = 50.0\n";
    let bare = prefix(Fuel::Methanol, D0, CUTOFF, AIR, q_out(0.4e-6));
    let on = toy6(Fuel::Methanol, &format!("{filter}{SC}"), &bare);
    let off = toy6(Fuel::Methanol, filter, &bare);
    assert!(
        on.encoded.prior[2] > 0.2 && on.encoded.prior[2] < 0.8,
        "{:?}",
        on.encoded.prior
    );
    assert_eq!(on.encoded.prior, off.encoded.prior);
    // odds of no flame against a hot flame: the prior's times the likelihood ratio of the end
    // rate, whose no-flame mean is (k_warm + K0 / 2) / K_h under the self-cooling law and 0
    // under the toy post.* law (both sds are 0.2 in the toy weights)
    let odds = |f: &Forecast| f.encoded.state[2] / f.encoded.state[0];
    let like = |x: f64, mean: f64| (-0.5 * ((x - mean) / 0.2).powi(2)).exp();
    let r = on.encoded.r_end;
    assert!((r - 0.8).abs() < 1e-6, "{r}");
    let prior = on.encoded.prior[2] / on.encoded.prior[0];
    let mean = (K_WARM + 0.5 * K0_HOT) / on.encoded.k_model_end_m2_s;
    assert!(close(odds(&on), prior * like(r, mean) / like(r, 1.0)));
    assert!(close(odds(&off), prior * like(r, 0.0) / like(r, 1.0)));
}

#[test]
fn the_dying_term_stands_only_after_a_radiative_loss_that_no_cool_flame_follows() {
    let p = prefix(Fuel::Heptane, D0, CUTOFF, AIR, q_law);
    let (t_e, q_e) = loss_point(Fuel::Heptane, RAD_LOSS, &p);
    // with the post.* law (which, in the toy weights, leaves the droplet as it is)
    let f = toy6(Fuel::Heptane, &format!("{RAD_LOSS}{DY}"), &p);
    assert_follows(&f, "dying, post law", |t| (q_e - die_fall(t - t_e)).sqrt());
    let dying = f.encoded.added.dying.expect("the term is on");
    assert!(close(dying.k_die_m2_s, DY_FRAC * K_COOL));
    // with the self-cooling law
    let f = toy6(Fuel::Heptane, &format!("{RAD_LOSS}{DY}{SC}"), &p);
    assert_follows(&f, "dying, self-cooling law", |t| {
        let u = t - t_e;
        (q_e - sc_fall(after_hot(1.0), q_e, u) - die_fall(u)).sqrt()
    });
    // nowhere else: after a diffusive extinction, after a cool flame that burned (started in the
    // forecast or burning at the cutoff), after a cool flame of zero length while the cool.d
    // network stands, for a droplet unlit at the cutoff, and for a fuel without a cool-flame law
    let burned = format!("{RAD_LOSS}cool.p.b0 = 30.0\ncool.d.b0 = 0.15\n");
    let zero = format!("{RAD_LOSS}cool.p.b0 = 30.0\ncool.d.b0 = 0.5\n");
    let lost = with_flame(p.clone(), frames(0.3, 3.0));
    // (the cool flame of the first burns from the loss to about 7.8 s; that of the second would
    // end at 3.3 mm, above the droplet, and never burns)
    let cool = toy6(Fuel::Heptane, &burned, &p).state_probability[1].clone();
    assert!(cool[0] > 0.8 && cool[2] < 1e-9, "{cool:?}");
    let never = toy6(Fuel::Heptane, &zero, &p);
    assert!(never.state_probability[1].iter().all(|v| *v == 0.0));
    assert!(never.state_probability[2][3] > 0.99);
    let cases: [(Fuel, &str, &PrefixBundle); 6] = [
        (Fuel::Heptane, DIFF_LOSS, &p),
        (Fuel::Heptane, &burned, &p),
        (Fuel::Heptane, COOL_NOW, &p),
        (Fuel::Heptane, &zero, &p),
        (Fuel::Heptane, "fl.trust = 1.0\n", &lost),
        (Fuel::Methanol, RAD_LOSS, &p_methanol()),
    ];
    for (fuel, weights, p) in cases {
        for law in ["", SC] {
            let with = toy6(fuel, &format!("{weights}{law}{DY}"), p);
            let without = toy6(fuel, &format!("{weights}{law}"), p);
            assert_eq!(with.quantiles, without.quantiles, "{weights}");
            assert_eq!(with.latent_quantiles, without.latent_quantiles, "{weights}");
            assert_eq!(with.mean, without.mean, "{weights}");
        }
    }
    assert!(
        toy6(Fuel::Methanol, &format!("{RAD_LOSS}{DY}"), &p_methanol())
            .encoded
            .added
            .dying
            .is_none()
    );
}

// ---- test-only values of the cool-flame end
const CE: &str = "ce.on = 1.0\nce.g0 = 0.7\nce.beta_atm = 0.05\nce.p_ref_atm = 0.21\n\
                  ce.eps_atm = 0.01\nce.c_he = 0.5\nce.d0_exp = 1.2\n";

/// The author's closed form of the median cool-flame end diameter.
#[allow(clippy::too_many_arguments)]
fn d_ce(d0: f64, gas: Gas, g0: f64, beta: f64, p_ref: f64, eps: f64, c_he: f64, n: f64) -> f64 {
    let po2 = gas.x_o2 * gas.p / 101_325.0;
    d0 * (d0 / D_REF).powf(n - 1.0)
        * g0
        * ((p_ref - beta) / (po2 - beta).max(eps)).sqrt()
        * (1.0 + c_he * gas.x_he)
}

#[test]
fn the_cool_flame_ends_where_the_closed_form_in_oxygen_partial_pressure_says() {
    let lean = Gas {
        x_o2: 0.18,
        x_co2: 0.0,
        x_he: 0.30,
        p: 70_000.0,
    };
    for gas in [AIR, HELIUM_CO2, lean] {
        let p = prefix(Fuel::Heptane, D0, CUTOFF, gas, q_law);
        let f = toy6(Fuel::Heptane, &format!("{COOL_NOW}{CE}"), &p);
        let d0 = f.encoded.q0_m2.sqrt();
        assert!((d0 - D0).abs() < 1e-9);
        let want = d_ce(d0, gas, 0.7, 0.05, 0.21, 0.01, 0.5, 1.2);
        assert!(
            close(f.encoded.laws.d_ce_m, want),
            "{} {want}",
            f.encoded.laws.d_ce_m
        );
        // the network's median is reported and not used
        assert!(close(f.encoded.laws.d_ce_net_m, D_REF * 0.2f64.exp()));
        let end = f.encoded.added.cool_end.expect("the form is on");
        assert!(close(end.po2_atm, gas.x_o2 * gas.p / 101_325.0));
        assert!(!end.capped);
        assert!(close(end.d_ce_m, want));
    }
    // the forecast uses it: a cool flame burning at the cutoff ends at that diameter, and the toy
    // unlit droplet then stays there
    let p = prefix(Fuel::Heptane, D0, CUTOFF, AIR, q_law);
    let f = toy6(Fuel::Heptane, &format!("{COOL_NOW}{CE}"), &p);
    let want = d_ce(D0, AIR, 0.7, 0.05, 0.21, 0.01, 0.5, 1.2);
    let q_cut = f.encoded.q_cutoff_m2;
    let t_x = CUTOFF + (q_cut - want * want) / K_COOL;
    assert!(t_x > 8.0 && t_x < 10.0, "{t_x}");
    assert_follows(&f, "a cool flame to the closed form", |t| {
        if t <= t_x {
            (q_cut - K_COOL * (t - CUTOFF)).sqrt()
        } else {
            want
        }
    });
    let cool = &f.state_probability[1];
    assert!(cool[0] > 1.0 - 1e-8 && cool[1] > 1.0 - 1e-8 && cool[2] < 1e-8 && cool[3] < 1e-8);
    // with the form off the network stands, as in droplet5.rs
    let f = toy6(Fuel::Heptane, COOL_NOW, &p);
    assert_eq!(f.encoded.laws.d_ce_m, f.encoded.laws.d_ce_net_m);
    // a fuel without a cool-flame law does not read the form
    let f = toy6(Fuel::Methanol, CE, &p_methanol());
    assert!(f.encoded.added.cool_end.is_none());
    assert_eq!(f.encoded.laws.d_ce_m, 0.0);
}

#[test]
fn close_to_the_limit_oxygen_pressure_the_cool_flame_end_is_capped() {
    let near =
        "ce.on = 1.0\nce.g0 = 0.2\nce.beta_atm = 0.15\nce.p_ref_atm = 0.21\nce.eps_atm = 0.02\n\
                ce.c_he = 0.5\nce.d0_exp = 1.0\n";
    let at = |x_o2: f64| {
        let gas = Gas { x_o2, ..AIR };
        let p = prefix(Fuel::Heptane, D0, CUTOFF, gas, q_law);
        toy6(Fuel::Heptane, &format!("{COOL_NOW}{near}"), &p)
    };
    // 0.16 atm is 0.01 above the limit, 0.12 atm is below it: both stand at the cap, limit + 0.02
    let capped = D0 * 0.2 * (0.06f64 / 0.02).sqrt();
    for x_o2 in [0.16, 0.12] {
        let f = at(x_o2);
        let end = f.encoded.added.cool_end.unwrap();
        assert!(end.capped, "pO2 = {}", end.po2_atm);
        assert!(
            close(f.encoded.laws.d_ce_m, capped),
            "{}",
            f.encoded.laws.d_ce_m
        );
    }
    // 0.19 atm is clear of it
    let f = at(0.19);
    assert!(!f.encoded.added.cool_end.unwrap().capped);
    assert!(close(
        f.encoded.laws.d_ce_m,
        D0 * 0.2 * (0.06f64 / 0.04).sqrt()
    ));
}

#[test]
fn a_cool_flame_of_zero_length_leaves_a_droplet_straight_from_the_hot_flame() {
    let p = prefix(Fuel::Heptane, D0, CUTOFF, AIR, q_law);
    let (t_e, q_e) = loss_point(Fuel::Heptane, RAD_LOSS, &p);
    // every radiative loss is followed by a cool flame, and under the closed form that flame would
    // end at 3.25 mm, above the 2.4 mm at which the hot flame is lost: zero length
    let form =
        "cool.p.b0 = 30.0\nce.on = 1.0\nce.g0 = 1.0\nce.beta_atm = 0.05\nce.p_ref_atm = 0.21\n\
                ce.eps_atm = 0.01\nce.c_he = 0.0\nce.d0_exp = 1.2\n";
    let f = toy6(Fuel::Heptane, &format!("{RAD_LOSS}{form}{SC}{DY}"), &p);
    assert!(f.encoded.laws.d_ce_m > q_e.sqrt() && f.encoded.laws.p_cool > 1.0 - 1e-9);
    assert_eq!(
        f.encoded.added.zero_length_cool_flame_counts_as,
        "hot_flame"
    );
    assert_follows(&f, "zero length under the closed form", |t| {
        let u = t - t_e;
        (q_e - sc_fall(after_hot(1.0), q_e, u) - die_fall(u)).sqrt()
    });
    assert!(f.state_probability[1].iter().all(|v| *v == 0.0));
    // with the post.* law it is the droplet of the no-cool-flame branch: the hot amplitude, not
    // post.cool_scale times it
    let post = "post.k_hot = 1.0e-7\npost.cool_scale = 0.3\n";
    let a = toy6(Fuel::Heptane, &format!("{RAD_LOSS}{form}{post}{DY}"), &p);
    let b = toy6(Fuel::Heptane, &format!("{RAD_LOSS}{post}{DY}"), &p);
    assert!(b.encoded.laws.p_cool < 1e-9);
    for j in 0..LATE.len() {
        assert_eq!(median(&a, j), median(&b, j));
    }
    // while the cool.d network stands (D_ce = 3.3 mm, zero length as well) the droplet counts as
    // coming from a cool flame, as in droplet5.rs: the cool amplitude and no dying flame
    let net = "cool.p.b0 = 30.0\ncool.d.b0 = 0.5\n";
    let f = toy6(Fuel::Heptane, &format!("{RAD_LOSS}{net}{SC}{DY}"), &p);
    assert_eq!(
        f.encoded.added.zero_length_cool_flame_counts_as,
        "cool_flame"
    );
    assert_follows(&f, "zero length under the network", |t| {
        (q_e - sc_fall(after_cool(1.0), q_e, t - t_e)).sqrt()
    });
}

#[test]
fn a_tensor_out_of_bounds_and_a_switch_that_is_neither_off_nor_on_are_refused() {
    let spec = droplet6::spec(Fuel::Heptane);
    let base = format!("{}{OFF6}", toy(true));
    let materialize = |extra: &str| {
        let ops = text::compile(&source(&base, extra), &spec).expect("literal statements");
        let patch = assemble::make_patch(&spec, common::meta("test-bounds"), ops);
        assemble::materialize(&spec, &patch, None)
    };
    assert!(materialize("").is_ok());
    for bad in [
        "sc.on = 2.0",
        "sc.k0_hot_mm2_s = 2.5",
        "sc.tstar_k = -1.0",
        "sc.lat_j_kg = 6000000.0",
        "sc.gas[0:3] = [0.0, 11.0, 0.0]",
        "sc.tau_warm_s = 0.001",
        "sc.diff_scale = 2.5",
        "dy.frac = 2.1",
        "dy.tau_s = 0.0",
        "ce.on = -1.0",
        "ce.beta_atm = 0.31",
        "ce.eps_atm = 0.0",
        "ce.d0_exp = 2.5",
    ] {
        assert!(materialize(bad).is_err(), "{bad} was accepted");
    }
    // a root patch must state the new entries too
    let ops = text::compile(&toy(true), &spec).unwrap();
    let patch = assemble::make_patch(&spec, common::meta("test-missing"), ops);
    let err = assemble::materialize(&spec, &patch, None)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("sc.") || err.contains("dy.") || err.contains("ce."),
        "{err}"
    );
    // inside the bounds and still not a model: a switch that is neither 0 nor 1, a closed form
    // that cannot be evaluated. The forward refuses to run them, so the fixture check fails and
    // no such release is assembled.
    let p = prefix(Fuel::Heptane, D0, CUTOFF, AIR, q_law);
    for (bad, code) in [
        ("sc.on = 0.5\n", "switch_value"),
        ("ce.on = 0.5\n", "switch_value"),
        (
            "ce.on = 1.0\nce.beta_atm = 0.15\nce.p_ref_atm = 0.10\n",
            "cool_end_form",
        ),
    ] {
        let model = models::by_key(key6(Fuel::Heptane)).unwrap();
        let tensors = materialize(bad).expect("inside the bounds");
        let ops = text::compile(&source(&base, bad), &spec).unwrap();
        let patch = assemble::make_patch(&spec, common::meta("test-switch"), ops);
        let err = model
            .build_release(&patch, &tensors, None, &common::binding())
            .expect_err("assembled");
        assert!(format!("{err:#}").contains("failed build check"), "{err:#}");
        let mut r = release6(Fuel::Heptane, "");
        r.params = tensors;
        let refusal = droplet6::forecast(&spec, &r, &p, &late())
            .err()
            .expect("the forward ran it");
        assert_eq!(refusal.code, code, "{bad}");
    }
}

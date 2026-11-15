//! The hinge-law clocks with the burner-heating law (`flame4.rs`) on synthetic prefixes with
//! test-only weights: its spec is the `flame.rs` spec without the old limit clock plus the burner
//! tensors of the `flame3.rs` spec; with no limit weight it answers as a `flame.rs` release whose
//! limit clock is out of reach; its burner block is the one `flame3.rs` reports for the same
//! numbers; the burner limit alone gives the closed-form incidence; a scheduled stop and the
//! refusals are those of `flame.rs`, with the burner-side ones of `flame3.rs`; the committed roots
//! hold their two source releases' numbers unchanged.

mod common;

use ignis_core::contracts::{
    CheckStatus, ControlStage, Family, ModelRelease, ModelSpec, PrefixBundle, SupportGate,
};
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::quad::normal_quantile;
use ignis_wsp::{acme_tools, assemble, audit, flame, flame3, flame4, models, text};
use serde_json::json;

const FAMILIES: [(Family, &str); 2] = [
    (Family::AcmeNormal, "normal"),
    (Family::AcmeInverse, "inverse"),
];

/// The hinge laws both `flame.rs` and `flame4.rs` hold, test-only: one self-extinction clock (median
/// 30 s, shape 2), no weak class, no programmed stop, no background hazard. Not a candidate.
const HINGE: &str = "
se.tau.b0 = 3.4011973816621555
se.tau.skip[0:8] = 0.0
se.flash.b0 = -40.0
se.flash.skip[0:8] = 0.0
se.floor_s = 0.0
se.shape = 2.0
se.flash_tau_s = 1.0
se.flash_shape = 4.0
ft.atom_t_s[0:8] = 0.0
ft.atom_logit[0:8] = -40.0
ft.log_base = -40.0
";

/// The limit clock only `flame.rs` holds (its test-only value; the tests push it out of reach).
const OLD_CLOCK: &str = "
ft.tau.b0 = 13.9
ft.tau.skip[0:8] = 0.0
ft.shape = 2.0
";

/// The burner tensors both `flame3.rs` and `flame4.rs` hold, test-only: dT_eq = e^6 K (403 K) above
/// t0 = 20 degC with tau = 50 s, no spread, shape 8, no limit weight (the clock is off).
const BURNER: &str = "
bt.rise.b0 = 6.0
bt.rise.skip[0:7] = 0.0
se.rho_d_g_m_s = 0.05
bt.tau_s = 50.0
bt.t0_c = 20.0
bt.sigma = 0.0
bt.shape = 8.0
bt.limit_c[0:3] = 0.0
bt.limit_w[0:3] = 0.0
";

/// What only `flame3.rs` holds besides (its test-only growth law; irrelevant to the burner block).
const GROWTH: &str = "
se.flux.b0 = -0.2
se.flux.skip[0:8] = 0.0
se.diff.b0 = 2.4
se.diff.skip[0:8] = 0.0
se.osc.b0 = 13.9
se.osc.skip[0:8] = 0.0
se.sigma_phi = 0.0
se.hold_log_rate = -40.0
se.osc_shape = 4.0
se.flash.b0 = -40.0
se.flash.skip[0:8] = 0.0
se.floor_s = 0.0
se.shape = 2.0
se.flash_tau_s = 1.0
se.flash_shape = 4.0
ft.atom_t_s[0:8] = 0.0
ft.atom_logit[0:8] = -40.0
ft.log_base = -40.0
";

/// Every hinge cause at once, with a dependence on the conditions: weak class, floor, a programmed
/// stop at 66 s and a background hazard.
const BUSY: &str = "se.tau.skip[0:8] = [0.2, -0.4, 0.3, 0.1, -0.2, 0.1, 0.3, -0.3]
se.tau.W1[0, 1] = 1.0
se.tau.b1[0] = 0.2
se.tau.w2[0] = -0.5
se.flash.b0 = -1.0
se.flash.skip[0:8] = [0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
se.floor_s = 4.0
ft.atom_t_s[0:8] = [66.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
ft.atom_logit[0:8] = [0.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0]
ft.atom_w[0, 1] = 0.7
ft.log_base = -7.0
";

/// A burner-heating clock that depends on the burner features, with a spread on dT_eq and three
/// limits, the last one beyond the reach of every heating node.
const HEATED: &str = "bt.rise.skip[0:7] = [0.3, -0.2, 0.1, 0.05, -0.1, 0.2, -0.3]
bt.rise.W1[0, 0] = 0.5
bt.rise.b1[0] = 0.1
bt.rise.w2[0] = 0.2
bt.sigma = 0.3
bt.limit_c[0:3] = [300.0, 380.0, 1400.0]
bt.limit_w[0:3] = [0.2, 0.6, 0.2]
";

fn key(family: Family, generation: u8) -> String {
    let name = if family == Family::AcmeInverse {
        "acme_inverse"
    } else {
        "acme_normal"
    };
    format!("{name}{generation}")
}

/// A root release of `key`: the statements of `base`, with `extra` statements replacing the base
/// statements of the tensors they name.
fn build(key: &str, base: &str, extra: &str) -> Result<LoadedRelease, String> {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let name_of = |l: &str| {
        l.trim_start()
            .split(['[', ' ', '='])
            .next()
            .unwrap_or_default()
            .to_string()
    };
    let mut src = String::new();
    for line in base.lines() {
        let name = name_of(line);
        if name.is_empty() || !extra.lines().any(|l| name_of(l) == name) {
            src.push_str(line);
            src.push('\n');
        }
    }
    src.push_str(extra);
    let ops = text::compile(&src, &spec).unwrap_or_else(|e| panic!("{key}: {e}\n{src}"));
    let patch = assemble::make_patch(&spec, common::meta("test-flame4"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .map_err(|e| format!("{e:#}"))?;
    Ok(LoadedRelease {
        release,
        params: tensors,
        posterior: TensorMap::new(),
    })
}

/// A `flame4.rs` release: the toy hinge laws and burner tensors with `extra` on top.
fn hybrid(family: Family, extra: &str) -> LoadedRelease {
    build(&key(family, 8), &format!("{HINGE}{BURNER}"), extra).expect("release")
}

fn set(r: &mut LoadedRelease, tensor: &str, data: &[f64]) {
    let t = r.params.get_mut(tensor).expect("tensor");
    assert_eq!(t.data.len(), data.len());
    t.data = data.to_vec();
}

/// A `flame.rs` release with the same hinge laws and its burner-limit clock out of reach: a median
/// of e^60 s with zero skip and no hidden unit. That value is outside the bounds an author may type,
/// so it is set on the assembled tensors.
fn hinge_twin(family: Family, extra: &str) -> LoadedRelease {
    let hinge_only: String = extra
        .lines()
        .filter(|l| !flame4::is_burner_tensor(l.split(['[', ' ', '=']).next().unwrap_or_default()))
        .map(|l| format!("{l}\n"))
        .collect();
    let mut r = build(&key(family, 2), &format!("{HINGE}{OLD_CLOCK}"), &hinge_only).expect("twin");
    set(&mut r, "ft.tau.b0", &[60.0]);
    r
}

/// A `flame3.rs` release with the same burner tensors.
fn burner_twin(family: Family, extra: &str) -> LoadedRelease {
    let burner_only: String = extra
        .lines()
        .filter(|l| flame4::is_burner_tensor(l.split(['[', ' ', '=']).next().unwrap_or_default()))
        .map(|l| format!("{l}\n"))
        .collect();
    build(&key(family, 3), &format!("{GROWTH}{BURNER}"), &burner_only).expect("twin")
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

/// Prefixes away from the reference state: the default fixture of the family and a butane test at
/// another pressure and flow.
fn prefixes(family: Family) -> Vec<PrefixBundle> {
    let other = match family {
        Family::AcmeInverse => acme_tools::fixture_prefix(
            family, 5.0, 150_000.0, 7.0e-6, 0.40, 0.15, 2500.0, 0.50, "butane",
        ),
        _ => acme_tools::fixture_prefix(
            family, 5.0, 150_000.0, 1.6e-6, 0.35, 0.50, 2350.0, 0.30, "butane",
        ),
    };
    vec![acme_tools::default_fixture(family), other]
}

fn rehash(p: &mut PrefixBundle) {
    p.input_hash = ignis_core::hashing::content_hash(&*p, &["input_hash"]);
}

fn stop_at(p: &mut PrefixBundle, t: f64) {
    p.known_controls.push(ControlStage {
        t_start_s: t,
        t_end_s: None,
        control: "scheduled_stop".into(),
        value: None,
        operator_action: false,
    });
    rehash(p);
}

fn with(p: &PrefixBundle, key: &str, v: f64) -> PrefixBundle {
    let mut p = p.clone();
    let m = p.known_conditions.get_mut(key).expect("condition");
    m.value_si = Some(v);
    m.raw_value = Some(ignis_core::contracts::RawValue::Number(v));
    rehash(&mut p);
    p
}

fn run4(r: &LoadedRelease, p: &PrefixBundle, times: &[f64]) -> flame4::Forecast {
    flame4::forecast(&flame4::spec(p.family), r, p, &json!({ "times_s": times }))
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn run2(r: &LoadedRelease, p: &PrefixBundle, times: &[f64]) -> flame::Forecast {
    flame::forecast(&flame::spec(p.family), r, p, &json!({ "times_s": times }))
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn run3(r: &LoadedRelease, p: &PrefixBundle, times: &[f64]) -> flame3::Forecast {
    flame3::forecast(&flame3::spec(p.family), r, p, &json!({ "times_s": times }))
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn survival(t: f64, tau: f64, k: f64) -> f64 {
    1.0 / (1.0 + (t / tau).powf(k))
}

fn check_partition(table: &[Vec<f64>; 4]) {
    for j in 0..table[0].len() {
        let sum: f64 = (0..4).map(|r| table[r][j]).sum();
        assert!((sum - 1.0).abs() < 1e-12, "rows sum to {sum}");
        assert!((0..4).all(|r| table[r][j] >= 0.0));
    }
}

fn config(path: &str) -> String {
    let path = format!(
        "{}/../../configs/models/acme/{path}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

const TIMES: [f64; 12] = [
    3.0, 5.0, 6.0, 15.0, 40.0, 65.8, 66.4, 90.0, 200.0, 500.0, 600.0, 610.0,
];

#[test]
fn the_spec_is_the_hinge_spec_without_its_limit_clock_plus_the_burner_tensors() {
    for (family, dir) in FAMILIES {
        let spec = flame4::spec(family);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(&key(family, 8)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp8/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/acme/wsp8/{dir}/spec.json drifted from flame4::spec(); a changed spec \
             is a new spec id (re-export with `ignis wsp spec --family {} --out …`), never an \
             in-place edit",
            key(family, 8)
        );
        let (hinge, heat) = (flame::spec(family), flame3::spec(family));
        // every tensor of the hinge spec except its limit clock, as it stands there
        let kept: Vec<_> = hinge
            .tensors
            .iter()
            .filter(|t| !t.name.starts_with("ft.tau.") && t.name != "ft.shape")
            .collect();
        assert_eq!(kept.len(), hinge.tensors.len() - 6);
        for (have, want) in spec.tensors.iter().zip(&kept) {
            assert_eq!(&have, want);
        }
        // then what the burner-limit clock of the growth-clock spec reads, as it stands there
        let burner = &spec.tensors[kept.len()..];
        let names: Vec<&str> = burner.iter().map(|t| t.name.as_str()).collect();
        for want in [
            "bt.rise.W1",
            "bt.rise.b1",
            "bt.rise.w2",
            "bt.rise.skip",
            "bt.rise.b0",
            "se.rho_d_g_m_s",
            "bt.tau_s",
            "bt.t0_c",
            "bt.sigma",
            "bt.shape",
            "bt.limit_c",
            "bt.limit_w",
            "norm.r_ref_mm",
            "norm.mtot_ref_kg_s",
            "phys.mw_ethane",
            "phys.mw_propane",
            "phys.mw_butane",
            "phys.mw_n2",
            "phys.mw_o2",
            "num.bt_nodes",
        ] {
            assert!(names.contains(&want), "{want} missing: {names:?}");
        }
        assert_eq!(burner.len(), 20);
        for t in burner {
            assert_eq!(heat.tensor(&t.name), Some(t), "{}", t.name);
        }
        assert_eq!(spec.features, heat.features);
        assert_eq!(spec.domain, heat.domain);
        assert_eq!(spec.target, hinge.target);
        // the toy weights assemble into a release whose build checks all pass
        let r = hybrid(family, "");
        let checks = &r.release.construction.as_ref().unwrap().checks;
        assert!(
            checks.iter().all(|c| c.status == CheckStatus::Passed),
            "{checks:?}"
        );
    }
}

#[test]
fn without_a_limit_weight_it_answers_as_the_hinge_forward_with_its_limit_clock_out_of_reach() {
    for (family, _) in FAMILIES {
        for extra in [
            "",
            BUSY,
            &format!("{BUSY}bt.sigma = 0.3\nbt.rise.b0 = 4.0\n"),
        ] {
            let (r4, r2) = (hybrid(family, extra), hinge_twin(family, extra));
            for p in prefixes(family) {
                let (f4, f2) = (run4(&r4, &p, &TIMES), run2(&r2, &p, &TIMES));
                assert_eq!(f4.times_s, f2.times_s);
                assert_eq!(f4.features, f2.features);
                for k in 0..4 {
                    for j in 0..f4.times_s.len() {
                        assert!(
                            (f4.table[k][j] - f2.table[k][j]).abs() < 1e-12,
                            "row {k} at {} s: {} against {}",
                            f4.times_s[j],
                            f4.table[k][j],
                            f2.table[k][j]
                        );
                    }
                }
                assert!((f4.p_weak_at_cutoff - f2.p_weak_at_cutoff).abs() < 1e-12);
                assert_eq!(f4.steps, f2.steps);
                assert_eq!(f4.clocks.p_limit_never, 1.0);
                assert!(f4.clocks.burner_limits.is_empty());
                assert_eq!(f4.clocks.radiative_tau_s, f2.clocks.radiative_tau_s);
                assert_eq!(f4.clocks.programmed, f2.clocks.programmed);
                check_partition(&f4.table);
            }
        }
    }
    // the hinge laws themselves: a single clock is its log-logistic distribution, the floor holds,
    // a programmed stop takes its written share, survivors are rarely of the weak class
    let times = [5.0, 15.0, 30.0, 60.0, 120.0];
    let f = run4(&hybrid(Family::AcmeNormal, ""), &reference(1.0), &times);
    for (j, t) in times.iter().enumerate() {
        let want = 1.0 - survival(*t, 30.0, 2.0) / survival(1.0, 30.0, 2.0);
        assert!((f.table[1][j] - want).abs() < 2e-4, "t = {t}");
        assert!(f.table[2][j] < 1e-6, "no flow termination was written");
    }
    assert!(f.features.iter().all(|c| c.abs() < 1e-9));
    let f = run4(
        &hybrid(Family::AcmeNormal, "se.floor_s = 10.0\n"),
        &reference(1.0),
        &[5.0, 9.9, 40.0],
    );
    assert!(f.table[1][0] < 1e-12 && f.table[1][1] < 1e-12);
    assert!((f.table[1][2] - 0.5).abs() < 2e-4, "{}", f.table[1][2]);
    let stop = "se.tau.b0 = 11.9\nft.atom_t_s[0:8] = [20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]\nft.atom_logit[0:8] = [0.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0]\n";
    let f = run4(
        &hybrid(Family::AcmeNormal, stop),
        &reference(1.0),
        &[19.0, 21.0, 100.0],
    );
    assert!(f.table[2][0] < 1e-6);
    assert!((f.table[2][1] - 0.5).abs() < 1e-5, "{}", f.table[2][1]);
    let f = run4(
        &hybrid(Family::AcmeNormal, "se.flash.b0 = 0.0\n"),
        &reference(5.0),
        &[10.0, 30.0],
    );
    let (s_weak, s_main) = (survival(5.0, 1.0, 4.0), survival(5.0, 30.0, 2.0));
    let want = 0.5 * s_weak / (0.5 * s_weak + 0.5 * s_main);
    assert!((f.p_weak_at_cutoff - want).abs() < 0.02 * want);
}

#[test]
fn the_burner_block_is_the_one_the_growth_clock_forward_reports_for_the_same_numbers() {
    for (family, _) in FAMILIES {
        for extra in [
            HEATED.to_string(),
            HEATED.replace("bt.sigma = 0.3", "bt.sigma = 0.0"),
            // a limit that no heating node reaches, alone
            HEATED.replace("[0.2, 0.6, 0.2]", "[0.0, 0.0, 1.0]"),
        ] {
            let (r4, r3) = (hybrid(family, &extra), burner_twin(family, &extra));
            for p in prefixes(family) {
                let (f4, f3) = (run4(&r4, &p, &TIMES), run3(&r3, &p, &TIMES));
                let (k4, k3) = (&f4.clocks, &f3.clocks);
                assert_eq!(k4.burner_limits, k3.burner_limits);
                assert_eq!(k4.p_limit_never, k3.p_limit_never);
                assert_eq!(k4.burner_rise_k, k3.burner_rise_k);
                assert_eq!(k4.burner_features, k3.burner_features);
                assert_eq!(k4.r_ss_mm, k3.r_ss_mm);
                assert_eq!(k4.burner_heat_tau_s, k3.burner_heat_tau_s);
                assert_eq!(k4.burner_t0_c, k3.burner_t0_c);
                assert_eq!(
                    (f4.burner.m_r_kg_s, f4.burner.m_tot_kg_s, f4.burner.l),
                    (f3.burner.m_r_kg_s, f3.burner.m_tot_kg_s, f3.burner.l)
                );
                assert_eq!(f4.features, f3.features);
                // and the artifact carries that block in place of the old clock's median
                let a4 = models::by_key(&key(family, 8))
                    .unwrap()
                    .artifact(&r4, &p, &json!({ "times_s": TIMES }))
                    .expect("artifact");
                a4.validate().expect("valid artifact");
                let a3 = models::by_key(&key(family, 3))
                    .unwrap()
                    .artifact(&r3, &p, &json!({ "times_s": TIMES }))
                    .expect("artifact");
                let (d4, d3) = (&a4.diagnostics, &a3.diagnostics);
                for field in [
                    "burner_limits",
                    "p_limit_never",
                    "burner_rise_k",
                    "burner_features",
                    "burner_heat_tau_s",
                    "burner_t0_c",
                ] {
                    assert_eq!(d4["clocks"][field], d3["clocks"][field], "{field}");
                    assert!(!d4["clocks"][field].is_null(), "{field}");
                }
                assert_eq!(d4["burner"], d3["burner"]);
                assert!(d4["clocks"].get("burner_tau_s").is_none());
            }
        }
    }
    // the limits are reported as written, and a share of the pairs never reaches its limit
    let f = run4(
        &hybrid(Family::AcmeNormal, HEATED),
        &reference(1.0),
        &[60.0],
    );
    assert_eq!(f.clocks.burner_limits.len(), 3);
    assert_eq!(f.clocks.burner_limits[0].0, 300.0);
    assert!((f.clocks.burner_limits[1].1 - 0.6).abs() < 1e-12);
    assert_eq!(f.clocks.burner_limits[2].2, None);
    assert!(f.clocks.p_limit_never >= 0.2 - 1e-12 && f.clocks.p_limit_never < 1.0);
}

/// S_bt(t) of the toy burner at the reference prefix, by hand from the spec's formulas: ln dT_j =
/// 6 + sigma z_j on seven equally likely nodes, t0 = 20 degC, tau = 50 s, shape 8.
fn s_bt_hand(t: f64, sigma: f64, limits: &[(f64, f64)]) -> f64 {
    let n = 7;
    let total: f64 = limits.iter().map(|(_, w)| w).sum();
    let mut s = 0.0;
    for (limit, w) in limits {
        for j in 0..n {
            let dt = (6.0 + sigma * normal_quantile((j as f64 + 0.5) / n as f64)).exp();
            let pair = w / total / n as f64;
            s += pair
                * if dt > limit - 20.0 {
                    survival(t, -50.0 * (1.0 - (limit - 20.0) / dt).ln(), 8.0)
                } else {
                    1.0
                };
        }
    }
    s
}

#[test]
fn the_burner_limit_alone_gives_the_closed_form_incidence() {
    let times = [10.0, 25.0, 40.0, 59.0, 80.0, 200.0, 590.0];
    for (sigma, limits) in [
        (0.0, vec![(300.0, 1.0)]),
        (0.0, vec![(300.0, 0.2), (380.0, 0.6)]),
        (0.3, vec![(300.0, 0.5), (380.0, 0.3), (500.0, 0.2)]),
    ] {
        let mut c = [0.0; 3];
        let mut w = [0.0; 3];
        for (i, (limit, weight)) in limits.iter().enumerate() {
            (c[i], w[i]) = (*limit, *weight);
        }
        let extra = format!(
            "bt.sigma = {sigma:?}\nbt.limit_c[0:3] = [{:?}, {:?}, {:?}]\nbt.limit_w[0:3] = [{:?}, {:?}, {:?}]\n",
            c[0], c[1], c[2], w[0], w[1], w[2]
        );
        // a main-class flame with a self-extinction clock of e^60 s (set on the assembled tensors:
        // an author cannot type it), no programmed stop, no background
        let mut r = hybrid(Family::AcmeNormal, &extra);
        set(&mut r, "se.tau.b0", &[60.0]);
        for cut in [1.0, 20.0] {
            let f = run4(&r, &reference(cut), &times);
            // times before the cutoff are not forecast
            assert_eq!(f.times_s.len(), times.iter().filter(|t| **t >= cut).count());
            for (j, t) in f.times_s.iter().enumerate() {
                let want = 1.0 - s_bt_hand(*t, sigma, &limits) / s_bt_hand(cut, sigma, &limits);
                assert!(
                    (f.table[2][j] - want).abs() < 1e-9,
                    "sigma {sigma}, cutoff {cut}, t = {t}: {} against {want}",
                    f.table[2][j]
                );
                assert!(f.table[1][j] < 1e-12, "no self-extinction is left");
            }
            check_partition(&f.table);
        }
    }
    // the trip competes with self-extinction: both rise, and the rows still partition
    let both = format!("{BUSY}{HEATED}");
    let f = run4(&hybrid(Family::AcmeNormal, &both), &reference(2.0), &TIMES);
    check_partition(&f.table);
    let quiet = run4(&hybrid(Family::AcmeNormal, BUSY), &reference(2.0), &TIMES);
    let j = TIMES.iter().position(|t| *t == 200.0).unwrap();
    assert!(
        f.table[2][j] > quiet.table[2][j] + 1e-3,
        "the trip adds flow terminations"
    );
    assert!(
        f.table[1][j] < quiet.table[1][j],
        "and takes flames self-extinction would have had"
    );
    for j in 1..TIMES.len() {
        assert!(f.table[1][j] >= f.table[1][j - 1] - 1e-15);
        assert!(f.table[2][j] >= f.table[2][j - 1] - 1e-15);
    }
}

#[test]
fn a_scheduled_stop_and_the_refusals_are_those_of_the_hinge_forward() {
    let family = Family::AcmeNormal;
    let (r4, r2) = (hybrid(family, BUSY), hinge_twin(family, BUSY));
    let spec4 = flame4::spec(family);
    let spec2 = flame::spec(family);
    let settings = json!({ "times_s": TIMES });
    // a scheduled stop at 40 s: nothing changes before it, then the active mass is an operator stop
    let free = run4(&r4, &reference(2.0), &TIMES);
    let mut p = reference(2.0);
    stop_at(&mut p, 40.0);
    let (g4, g2) = (run4(&r4, &p, &TIMES), run2(&r2, &p, &TIMES));
    assert_eq!(g4.scheduled_stop_s, Some(40.0));
    assert_eq!(g4.scheduled_stop_s, g2.scheduled_stop_s);
    check_partition(&g4.table);
    let i40 = TIMES.iter().position(|t| *t == 40.0).unwrap();
    for j in 0..TIMES.len() {
        for k in 0..4 {
            assert!((g4.table[k][j] - g2.table[k][j]).abs() < 1e-12);
        }
        if j < i40 {
            for k in 0..4 {
                assert!((g4.table[k][j] - free.table[k][j]).abs() < 1e-12);
            }
        } else {
            assert_eq!(g4.table[0][j], 0.0);
            assert_eq!(g4.table[3][j], 0.0);
            assert!((g4.table[1][j] - free.table[1][i40]).abs() < 1e-12);
            assert!((g4.table[2][j] - (free.table[2][i40] + free.table[0][i40])).abs() < 1e-12);
        }
    }
    // past the horizon what is still active is unresolved support
    let ih = TIMES.iter().position(|t| *t == 600.0).unwrap();
    for j in ih..TIMES.len() {
        assert_eq!(free.table[0][j], 0.0);
        assert!(free.table[3][j] > 0.0);
    }

    // every refusal of the hinge forward, with the same gate and reasons
    let mut early = reference(2.0);
    stop_at(&mut early, 1.5);
    let mut unconfirmed = reference(2.0);
    unconfirmed.phase_established_at_s = None;
    let mut missing = reference(2.0);
    missing.known_conditions.remove("tad");
    let mut unknown_fuel = reference(2.0);
    unknown_fuel
        .known_conditions
        .get_mut("fuel_species")
        .unwrap()
        .raw_value = Some(ignis_core::contracts::RawValue::Text("methane".into()));
    for (what, p, gate) in [
        (
            "a stop at or before the cutoff",
            early,
            SupportGate::Semantic,
        ),
        ("phase not established", unconfirmed, SupportGate::Semantic),
        (
            "the other configuration",
            acme_tools::default_fixture(Family::AcmeInverse),
            SupportGate::Semantic,
        ),
        ("a missing condition", missing, SupportGate::Condition),
        ("an unknown fuel", unknown_fuel, SupportGate::Condition),
        (
            "a flow that is not positive",
            with(&reference(2.0), "injected_mass_flow", 0.0),
            SupportGate::Condition,
        ),
        (
            "outside the domain",
            with(&reference(2.0), "pressure", 5.0e5),
            SupportGate::Condition,
        ),
        (
            "a cutoff past the horizon",
            reference(700.0),
            SupportGate::Horizon,
        ),
    ] {
        let s4 = flame4::forecast(&spec4, &r4, &p, &settings)
            .expect("runnable")
            .expect_err(what);
        let s2 = flame::forecast(&spec2, &r2, &p, &settings)
            .expect("runnable")
            .expect_err(what);
        assert_eq!(s4.gate, gate, "{what}: {:?}", s4.reasons);
        assert_eq!((s4.gate, &s4.reasons), (s2.gate, &s2.reasons), "{what}");
    }
    let none = flame4::forecast(&spec4, &r4, &reference(2.0), &json!({ "times_s": [1.0] }))
        .expect("runnable")
        .expect_err("no time after the cutoff");
    assert_eq!(none.gate, SupportGate::Horizon);
    // a hinge release is not a release of this forward
    assert!(flame4::forecast(&spec4, &r2, &reference(2.0), &settings).is_err());

    // the burner side refuses what the growth-clock forward refuses
    let r3 = burner_twin(family, "");
    let spec3 = flame3::spec(family);
    let mut two = reference(2.0);
    for k in ["fuel_is_ethane", "fuel_is_butane"] {
        let mut v = two.known_conditions["zst"].clone();
        v.value_si = Some(1.0);
        two.known_conditions.insert(k.into(), v);
    }
    for (needle, p) in [
        ("zst", with(&reference(2.0), "zst", 1.0)),
        ("mole fraction", with(&reference(2.0), "x_fuel_role", 1.5)),
        ("both 1", two),
    ] {
        let s4 = flame4::forecast(&spec4, &r4, &p, &settings)
            .expect("runnable")
            .expect_err(needle);
        assert!(
            s4.reasons.iter().any(|r| r.contains(needle)),
            "{:?}",
            s4.reasons
        );
        let s3 = flame3::forecast(&spec3, &r3, &p, &settings)
            .expect("runnable")
            .expect_err(needle);
        assert_eq!((s4.gate, &s4.reasons), (s3.gate, &s3.reasons), "{needle}");
    }
    // a weighted limit at or below the burner temperature at ignition: the forward refuses it, so
    // such a release cannot be assembled (its finite-forward build check fails)
    let at_t0 = "bt.limit_c[0:3] = [10.0, 0.0, 0.0]\nbt.limit_w[0:3] = [1.0, 0.0, 0.0]\n";
    let err = build(&key(family, 8), &format!("{HINGE}{BURNER}"), at_t0)
        .err()
        .expect("a limit below t0 must not assemble");
    assert!(err.contains("failed build check"), "{err}");
    let mut cold = hybrid(family, "");
    set(&mut cold, "bt.limit_c", &[10.0, 0.0, 0.0]);
    set(&mut cold, "bt.limit_w", &[1.0, 0.0, 0.0]);
    let s = flame4::forecast(&spec4, &cold, &reference(2.0), &settings)
        .expect("runnable")
        .expect_err("limit below t0");
    assert_eq!(s.gate, SupportGate::Semantic);
    assert!(
        s.reasons[0].contains("at or below bt.t0_c"),
        "{:?}",
        s.reasons
    );
    // a rise network that overflows at the conditions is refused, not replaced
    let mut hot = hybrid(family, HEATED);
    set(&mut hot, "bt.rise.b0", &[800.0]);
    let s = flame4::forecast(&spec4, &hot, &reference(2.0), &settings)
        .expect("runnable")
        .expect_err("overflow");
    assert_eq!(s.gate, SupportGate::Condition);
    assert!(
        s.reasons.iter().any(|r| r.contains("burner rise")),
        "{:?}",
        s.reasons
    );
}

#[test]
fn the_forward_has_its_own_identity_and_the_committed_roots_hold_their_sources_numbers() {
    for name in ["acme_normal.wsp8", "acme_inverse.wsp8"] {
        let files: Vec<&str> = audit::forward_sources_of(name)
            .expect("a literal model")
            .iter()
            .map(|(n, _)| *n)
            .collect();
        assert!(files.contains(&"flame4.rs"), "{files:?}");
        assert!(!files.contains(&"flame.rs") && !files.contains(&"flame3.rs"));
        assert_eq!(audit::banned_hits_of(name), Vec::<String>::new());
    }
    let h = audit::forward_hash_of("acme_normal.wsp8");
    assert_eq!(h, audit::forward_hash_of("acme_inverse.wsp8"));
    assert_ne!(h, audit::forward_hash_of("acme_normal.wsp2"));
    assert_ne!(h, audit::forward_hash_of("acme_normal.wsp3"));

    let read = |path: &str| -> ModelRelease {
        serde_json::from_str(&config(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    for (family, dir) in FAMILIES {
        let root = read(&format!("wsp8/{dir}/baseline-clock-theory/release.json"));
        let hinge = read(&format!("wsp2/{dir}/baseline-clock/release.json"));
        let heat = read(&format!("wsp3/{dir}/baseline-theory/release.json"));
        assert_eq!(root.tensors.len(), flame4::spec(family).tensors.len());
        for t in &root.tensors {
            let source = if flame4::is_burner_tensor(&t.name) {
                &heat
            } else {
                &hinge
            };
            let want = source
                .tensors
                .iter()
                .find(|e| e.name == t.name)
                .unwrap_or_else(|| panic!("{dir}: {} is not in {}", t.name, source.id));
            assert_eq!(
                (&t.shape, &t.sha256),
                (&want.shape, &want.sha256),
                "{dir}: {} differs from {}",
                t.name,
                source.id
            );
        }
        let c = root.construction.as_ref().expect("a literal release");
        assert_eq!(
            c.forward_hash,
            audit::forward_hash_of(&root.model_name),
            "{dir}: re-assemble (tools/wsp/reassemble.py)"
        );
        assert!(c.checks.iter().all(|k| k.status == CheckStatus::Passed));
        // the committed edits are the committed release
        let from_edits = build(
            &key(family, 8),
            &config(&format!("wsp8/{dir}/baseline-clock-theory/edits.wsp")),
            "",
        )
        .expect("root");
        let weights = &from_edits
            .release
            .construction
            .as_ref()
            .unwrap()
            .weights_hash;
        assert_eq!(weights, &c.weights_hash);
    }
}

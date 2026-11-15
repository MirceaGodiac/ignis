//! The growth-clock forward of the ACME burner flames (`flame3.rs`) on synthetic prefixes with
//! test-only weights: the burner-side quantities, the radii, the reduced flow and the growth clock
//! reproduce an independent hand computation for both configurations; a flame that settles inside
//! its extinction radius is taken only by the hold hazard or the weak class; the spread over the
//! reduced flow holds a share of the nodes; survival to the cutoff re-weights the latent classes;
//! the rows partition; out-of-range conditions and overflowing clocks are refused; the committed
//! specs are the specs the forward was compiled with.

mod common;

use ignis_core::contracts::{ControlStage, Family, ModelSpec, PrefixBundle};
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::flame3::{self, Forecast};
use ignis_wsp::{acme_tools, assemble, models, text};
use serde_json::json;

/// Test-only weights, not a candidate: ln m_cr = -0.2 (0.82 g m-2 s-1), ln D = 2.4 (11 mm2/s),
/// rhoD = 0.05 g m-1 s-1, no spread on Phi, growth-clock shape 2, no floor, no weak class, no hold
/// hazard, the oscillation clock at about 1e6 s with shape 4 (off), the burner-heating clock off
/// (every limit weight zero), no programmed stop, no background hazard.
const TOY: &str = "
se.flux.b0 = -0.2
se.flux.skip[0:8] = 0.0
se.diff.b0 = 2.4
se.diff.skip[0:8] = 0.0
se.flash.b0 = -40.0
se.flash.skip[0:8] = 0.0
se.osc.b0 = 13.9
se.osc.skip[0:8] = 0.0
bt.rise.b0 = 6.0
bt.rise.skip[0:7] = 0.0
se.rho_d_g_m_s = 0.05
se.sigma_phi = 0.0
se.hold_log_rate = -40.0
se.floor_s = 0.0
se.shape = 2.0
se.osc_shape = 4.0
se.flash_tau_s = 1.0
se.flash_shape = 4.0
bt.tau_s = 50.0
bt.t0_c = 20.0
bt.sigma = 0.0
bt.shape = 8.0
bt.limit_c[0:3] = 0.0
bt.limit_w[0:3] = 0.0
ft.atom_t_s[0:8] = 0.0
ft.atom_logit[0:8] = -40.0
ft.log_base = -40.0
";

// The spec's physical constants, restated here so the hand computation does not read the forward.
const MW_PROPANE: f64 = 44.10;
const MW_O2: f64 = 31.998;
const MW_N2: f64 = 28.013;
const NODES: usize = 15;
const TAU_CAP_S: f64 = 1.0e5;

fn key(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse3",
        _ => "acme_normal3",
    }
}

/// The toy weights with `extra` statements replacing the base statements of the tensors they name.
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
    let patch = assemble::make_patch(&spec, common::meta("test-flame3"), ops);
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

/// A normal-flame prefix at the reference state of every feature (c = 0): 1 atm, 1 mg/s of
/// propane at 30 % in N2, 30 % O2 ambient, T_ad 2200 K, Z_st 0.25.
fn normal(cutoff: f64) -> PrefixBundle {
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

/// An inverse-flame prefix: 1 atm, 5 mg/s of O2 injected at 50 % in N2, 10 % propane ambient,
/// T_ad 2400 K, Z_st 0.45.
fn inverse(cutoff: f64) -> PrefixBundle {
    acme_tools::fixture_prefix(
        Family::AcmeInverse,
        cutoff,
        101_325.0,
        5.0e-6,
        0.50,
        0.10,
        2400.0,
        0.45,
        "propane",
    )
}

fn outcome(
    family: Family,
    extra: &str,
    p: &PrefixBundle,
    times: &[f64],
) -> Result<Forecast, Vec<String>> {
    let r = release(family, extra);
    let spec = flame3::spec(family);
    flame3::forecast(&spec, &r, p, &json!({ "times_s": times }))
        .expect("runnable")
        .map_err(|s| s.reasons)
}

fn run(family: Family, extra: &str, p: &PrefixBundle, times: &[f64]) -> Forecast {
    outcome(family, extra, p, times).unwrap_or_else(|r| panic!("refused: {r:?}"))
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs()
}

/// erfc^-1 by bisection on `libm::erfc` (decreasing on the real line).
fn erfc_inv(y: f64) -> f64 {
    let (mut lo, mut hi) = (-10.0_f64, 10.0_f64);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if libm::erfc(mid) > y {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Standard-normal quantile from the same bisection: Phi^-1(p) = -sqrt(2) erfc^-1(2 p).
fn z_of(p: f64) -> f64 {
    -std::f64::consts::SQRT_2 * erfc_inv(2.0 * p)
}

/// The growth law by hand, in SI, from the reactant flow, its mole fraction and molar mass in N2,
/// L, and the typed numbers (ln m_cr in g m-2 s-1, ln D in mm2/s, rhoD in g m-1 s-1).
struct Hand {
    m_tot: f64,
    r_ss_mm: f64,
    r_ext_mm: f64,
    phi: f64,
    t_g: f64,
}

fn hand(m_r: f64, x_r: f64, mw_r: f64, l: f64, ln_flux: f64, ln_diff: f64, rho_d: f64) -> Hand {
    let y = x_r * mw_r / (x_r * mw_r + (1.0 - x_r) * MW_N2);
    let m_tot = m_r / y;
    let pi4 = 4.0 * std::f64::consts::PI;
    let r_ss = m_tot / (pi4 * rho_d * 1.0e-3 * l);
    let r_ext = (m_r / (pi4 * ln_flux.exp() * 1.0e-3)).sqrt();
    let d = ln_diff.exp() * 1.0e-6;
    Hand {
        m_tot,
        r_ss_mm: r_ss * 1.0e3,
        r_ext_mm: r_ext * 1.0e3,
        phi: (r_ss / r_ext).powi(2),
        t_g: r_ext * r_ext / (4.0 * d),
    }
}

/// Time for the growing flame to reach r_ext at reduced flow `phi` (None: held or beyond the cap).
fn clock(phi: f64, t_g: f64) -> Option<f64> {
    if phi <= 1.0 {
        return None;
    }
    let e = erfc_inv(1.0 / phi.sqrt());
    let t = t_g / (e * e);
    (t < TAU_CAP_S).then_some(t)
}

fn survival(t: f64, tau: f64, k: f64) -> f64 {
    1.0 / (1.0 + (t / tau).powf(k))
}

fn check_partition(f: &Forecast) {
    for j in 0..f.times_s.len() {
        let sum: f64 = (0..4).map(|r| f.table[r][j]).sum();
        assert!(
            (sum - 1.0).abs() < 1e-12,
            "t = {}: rows sum to {sum}",
            f.times_s[j]
        );
        assert!((0..4).all(|r| f.table[r][j] >= -1e-15 && f.table[r][j] <= 1.0 + 1e-12));
    }
}

#[test]
fn a_normal_flame_reproduces_the_hand_computed_growth_law() {
    let times = [5.0, 15.0, 30.0, 60.0, 120.0, 300.0];
    let f = run(Family::AcmeNormal, "", &normal(1.0), &times);
    assert!(f.features.iter().all(|c| c.abs() < 1e-12));
    // normal: the burner stream is the fuel stream, L = ln(1 / (1 - Z_st))
    let l = (1.0 / (1.0 - 0.25_f64)).ln();
    let h = hand(1.0e-6, 0.30, MW_PROPANE, l, -0.2, 2.4, 0.05);
    assert!(rel(f.burner.m_r_kg_s, 1.0e-6) < 1e-12);
    assert!(rel(f.burner.m_tot_kg_s, h.m_tot) < 1e-9);
    assert!(rel(f.burner.l, l) < 1e-12);
    let k = &f.clocks;
    assert!(rel(k.m_cr_g_m2_s, (-0.2_f64).exp()) < 1e-12);
    assert!(rel(k.d_mm2_s, 2.4_f64.exp()) < 1e-12);
    assert!(
        rel(k.r_ss_mm, h.r_ss_mm) < 1e-6,
        "{} vs {}",
        k.r_ss_mm,
        h.r_ss_mm
    );
    assert!(
        rel(k.r_ext_mm, h.r_ext_mm) < 1e-6,
        "{} vs {}",
        k.r_ext_mm,
        h.r_ext_mm
    );
    assert!(rel(k.phi, h.phi) < 1e-6, "{} vs {}", k.phi, h.phi);
    assert!(
        h.phi > 1.5 && h.phi < 2.5,
        "the toy sits in the growth regime"
    );
    let tau = clock(h.phi, h.t_g).expect("Phi > 1");
    let got = k.radiative_tau_s.expect("not held");
    assert!(rel(got, tau) < 1e-6, "{got} vs {tau}");
    assert!(tau > 20.0 && tau < 60.0);
    // no spread: every node carries the same clock, none is held
    assert_eq!(k.node_tau_s.len(), NODES);
    assert!(k
        .node_tau_s
        .iter()
        .all(|t| t.is_some_and(|t| rel(t, got) < 1e-12)));
    assert_eq!(k.p_held, 0.0);
    // the burner-heating clock and the oscillation clock are off
    assert_eq!(k.p_limit_never, 1.0);
    assert!(k.burner_limits.is_empty());
    assert!(rel(k.osc_tau_s, 13.9_f64.exp()) < 1e-12);
    // and the incidence of extinction is that log-logistic clock conditioned on the cutoff (the
    // regression of the restructure: closed form, so to 1e-6; the oscillation clock adds ~1e-11)
    for (j, t) in times.iter().enumerate() {
        let want = 1.0 - survival(*t, tau, 2.0) / survival(1.0, tau, 2.0);
        assert!(
            (f.table[1][j] - want).abs() < 1e-6,
            "t = {t}: {} vs {want}",
            f.table[1][j]
        );
        assert!(f.table[2][j] < 1e-12, "no flow termination is written");
    }
    check_partition(&f);
}

#[test]
fn an_inverse_flame_takes_the_oxidiser_stream_as_the_burner_stream() {
    let times = [5.0, 30.0, 120.0];
    let f = run(
        Family::AcmeInverse,
        "se.flux.b0 = 1.0\n",
        &inverse(1.0),
        &times,
    );
    // inverse: the burner-side reactant is the injected O2 (injected_mass_flow, the O2 alone), the
    // burner stream is O2 in N2, and L = ln(1 / Z_st)
    let l = (1.0 / 0.45_f64).ln();
    let h = hand(5.0e-6, 0.50, MW_O2, l, 1.0, 2.4, 0.05);
    assert!(rel(f.burner.m_r_kg_s, 5.0e-6) < 1e-12);
    assert!(rel(f.burner.m_tot_kg_s, h.m_tot) < 1e-9);
    assert!(rel(f.burner.l, l) < 1e-12);
    let k = &f.clocks;
    assert!(
        rel(k.r_ss_mm, h.r_ss_mm) < 1e-6,
        "{} vs {}",
        k.r_ss_mm,
        h.r_ss_mm
    );
    assert!(
        rel(k.r_ext_mm, h.r_ext_mm) < 1e-6,
        "{} vs {}",
        k.r_ext_mm,
        h.r_ext_mm
    );
    assert!(rel(k.phi, h.phi) < 1e-6, "{} vs {}", k.phi, h.phi);
    assert!(h.phi > 1.5);
    let tau = clock(h.phi, h.t_g).expect("Phi > 1");
    let got = k.radiative_tau_s.expect("not held");
    assert!(rel(got, tau) < 1e-6, "{got} vs {tau}");
    for (j, t) in times.iter().enumerate() {
        let want = 1.0 - survival(*t, tau, 2.0) / survival(1.0, tau, 2.0);
        assert!((f.table[1][j] - want).abs() < 1e-6, "t = {t}");
    }
    check_partition(&f);
    // the fuel species does not enter the inverse burner stream
    let mut p = inverse(1.0);
    p.known_conditions
        .get_mut("fuel_species")
        .unwrap()
        .raw_value = Some(ignis_core::contracts::RawValue::Text("butane".into()));
    let g = run(Family::AcmeInverse, "se.flux.b0 = 1.0\n", &p, &times);
    assert!(rel(g.clocks.phi, k.phi) < 1e-12);
}

#[test]
fn a_flame_inside_its_extinction_radius_is_taken_only_by_the_hold_hazard_or_the_weak_class() {
    // ln m_cr = -3: Phi is about 0.12 and stays below 1 at every node of a spread of 0.5
    let base = "se.flux.b0 = -3.0\nse.sigma_phi = 0.5\n";
    let times = [5.0, 60.0, 300.0, 590.0];
    let f = run(Family::AcmeNormal, base, &normal(1.0), &times);
    let z_max = z_of((NODES as f64 - 0.5) / NODES as f64);
    assert!(f.clocks.phi * (0.5 * z_max).exp() < 1.0);
    assert!(f.clocks.radiative_tau_s.is_none());
    assert!(f.clocks.node_tau_s.iter().all(Option::is_none));
    assert_eq!(f.clocks.p_held, 1.0);
    assert!((f.p_held_at_cutoff - 1.0).abs() < 1e-12);
    assert!(f.table[1].iter().all(|p| *p < 1e-12), "{:?}", f.table[1]);
    check_partition(&f);

    // with the weak class switched on, extinction is the weak class alone
    let g = run(
        Family::AcmeNormal,
        &format!("{base}se.flash.b0 = 0.0\n"),
        &normal(1.0),
        &times,
    );
    let (s_w, s_main) = (survival(1.0, 1.0, 4.0), 1.0);
    let p_weak = 0.5 * s_w / (0.5 * s_w + 0.5 * s_main);
    assert!(
        (g.p_weak_at_cutoff - p_weak).abs() < 1e-9,
        "{}",
        g.p_weak_at_cutoff
    );
    for (j, t) in times.iter().enumerate() {
        let want = p_weak * (1.0 - survival(*t, 1.0, 4.0) / s_w);
        assert!(
            (g.table[1][j] - want).abs() < 1e-6,
            "t = {t}: {} vs {want}",
            g.table[1][j]
        );
    }

    // with a hold hazard of 0.01/s after a floor of 10 s, extinction is that constant hazard
    let hold = format!("{base}se.hold_log_rate = -4.605170185988091\nse.floor_s = 10.0\n");
    let times = [9.9, 20.0, 100.0, 400.0];
    let h = run(Family::AcmeNormal, &hold, &normal(1.0), &times);
    assert!(h.table[1][0] < 1e-12, "nothing before the floor");
    for (j, t) in times.iter().enumerate().skip(1) {
        let want = 1.0 - (-0.01 * (t - 10.0)).exp();
        assert!(
            (h.table[1][j] - want).abs() < 1e-6,
            "t = {t}: {} vs {want}",
            h.table[1][j]
        );
    }
}

#[test]
fn the_spread_on_the_reduced_flow_holds_a_share_of_the_nodes() {
    // ln m_cr = -0.81: Phi is about 1.05
    let near = "se.flux.b0 = -0.81\n";
    let f = run(Family::AcmeNormal, near, &normal(1.0), &[60.0]);
    let l = (1.0 / 0.75_f64).ln();
    let h = hand(1.0e-6, 0.30, MW_PROPANE, l, -0.81, 2.4, 0.05);
    assert!(rel(f.clocks.phi, h.phi) < 1e-6);
    assert!(h.phi > 1.0 && h.phi < 1.1);
    // sigma_phi = 0: one clock at every node
    let first = f.clocks.node_tau_s[0].expect("Phi > 1");
    assert!(f.clocks.node_tau_s.iter().all(|t| *t == Some(first)));
    assert_eq!(f.clocks.p_held, 0.0);

    // sigma_phi = 0.3: the nodes below ln Phi = 0 are held, the others have their own clock
    let g = run(
        Family::AcmeNormal,
        &format!("{near}se.sigma_phi = 0.3\n"),
        &normal(1.0),
        &[60.0],
    );
    let mut held = 0;
    for i in 0..NODES {
        let z = z_of((i as f64 + 0.5) / NODES as f64);
        let want = clock(h.phi * (0.3 * z).exp(), h.t_g);
        match (want, g.clocks.node_tau_s[i]) {
            (None, None) => held += 1,
            (Some(a), Some(b)) => assert!(rel(b, a) < 1e-6, "node {i}: {b} vs {a}"),
            (a, b) => panic!("node {i}: hand {a:?}, forward {b:?}"),
        }
    }
    assert!(held > 0 && held < NODES, "{held}");
    assert!((g.clocks.p_held - held as f64 / NODES as f64).abs() < 1e-12);
    // the centre of the distribution is the median node
    assert_eq!(g.clocks.radiative_tau_s, g.clocks.node_tau_s[NODES / 2]);
}

#[test]
fn survivors_are_more_likely_held_and_less_likely_weak() {
    // Phi about 1.05, a wide spread and a fast diffusivity: the nodes that are not held go out
    // within minutes; half of the flames are weak a priori
    let extra = "se.flux.b0 = -0.81\nse.diff.b0 = 5.0\nse.sigma_phi = 1.0\nse.flash.b0 = 0.0\n";
    let cutoffs = [0.0, 5.0, 20.0, 60.0];
    let fs: Vec<Forecast> = cutoffs
        .iter()
        .map(|c| run(Family::AcmeNormal, extra, &normal(*c), &[c + 1.0]))
        .collect();
    let prior = fs[0].clocks.p_held;
    assert!(prior > 0.0 && prior < 1.0);
    // at the time origin the prior stands
    assert!((fs[0].p_weak_at_cutoff - 0.5).abs() < 1e-12);
    assert!((fs[0].p_held_at_cutoff - 0.5 * prior).abs() < 1e-12);
    for w in fs.windows(2) {
        assert!(w[1].p_held_at_cutoff > w[0].p_held_at_cutoff);
        assert!(w[1].p_weak_at_cutoff < w[0].p_weak_at_cutoff);
    }
    // a survivor at 60 s is more likely held than a flame drawn from the prior
    assert!(fs[3].p_held_at_cutoff > prior);
    // and the weights are prior times survival, normalised (by hand, from the node clocks)
    let f = &fs[2];
    let tau = 20.0;
    let mut weights = vec![0.5 * survival(tau, 1.0, 4.0)];
    let mut held = 0.0;
    for t in &f.clocks.node_tau_s {
        let w = 0.5 / NODES as f64 * t.map_or(1.0, |t| survival(tau, t, 2.0));
        if t.is_none() {
            held += w;
        }
        weights.push(w);
    }
    let total: f64 = weights.iter().sum();
    assert!(
        rel(f.p_held_at_cutoff, held / total) < 1e-9,
        "{} vs {}",
        f.p_held_at_cutoff,
        held / total
    );
    assert!(rel(f.p_weak_at_cutoff, weights[0] / total) < 1e-9);
}

/// Every cause at once: weak class, spread, hold hazard, floor, oscillation clock (90 s), the
/// burner-heating clock (two limits, spread on dT_eq), a programmed stop at 66 s and a background
/// hazard.
const BUSY: &str = "se.flash.b0 = -1.0
se.sigma_phi = 0.5
se.hold_log_rate = -6.214608098422191
se.floor_s = 4.0
se.osc.b0 = 4.5
se.osc_shape = 3.0
bt.sigma = 0.3
bt.limit_c[0:3] = [300.0, 380.0, 0.0]
bt.limit_w[0:3] = [0.5, 0.5, 0.0]
ft.atom_t_s[0:8] = [66.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
ft.atom_logit[0:8] = [0.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0]
ft.log_base = -7.0
";

fn stop_at(p: &mut PrefixBundle, t: f64) {
    p.known_controls.push(ControlStage {
        t_start_s: t,
        t_end_s: None,
        control: "scheduled_stop".into(),
        value: None,
        operator_action: false,
    });
    p.input_hash = ignis_core::hashing::content_hash(&*p, &["input_hash"]);
}

#[test]
fn the_rows_partition_are_monotone_and_a_scheduled_stop_takes_what_is_left() {
    let mut times: Vec<f64> = (0..90).map(|i| 2.0 + 7.0 * i as f64).collect();
    times.extend([50.0, 600.0, 610.0]);
    times.sort_by(f64::total_cmp);
    let f = run(Family::AcmeNormal, BUSY, &normal(2.0), &times);
    check_partition(&f);
    for j in 1..times.len() {
        assert!(
            f.table[1][j] >= f.table[1][j - 1] - 1e-15,
            "extinction falls at {}",
            times[j]
        );
        assert!(
            f.table[2][j] >= f.table[2][j - 1] - 1e-15,
            "termination falls at {}",
            times[j]
        );
        if times[j] < 600.0 {
            assert!(f.table[0][j] <= f.table[0][j - 1] + 1e-15);
        }
    }
    assert!(f.table[1].last().unwrap() > &0.01 && f.table[2].last().unwrap() > &0.01);
    // the programmed stop at 66 s takes half of the flames still burning: past it, every class
    // has half the survival it has without the stop
    let busy_no_atom = BUSY.replace("ft.atom_logit[0:8] = [0.0,", "ft.atom_logit[0:8] = [-40.0,");
    let f0 = run(Family::AcmeNormal, &busy_no_atom, &normal(2.0), &times);
    for (j, t) in times.iter().enumerate() {
        let ratio = f.table[0][j] / f0.table[0][j];
        if *t < 65.0 {
            assert!((ratio - 1.0).abs() < 1e-9, "t = {t}: {ratio}");
        } else if *t > 67.0 && *t < 600.0 {
            assert!((ratio - 0.5).abs() < 1e-6, "t = {t}: {ratio}");
        }
    }
    // past the horizon what is still active is unresolved support
    let ih = times.iter().position(|t| *t == 600.0).unwrap();
    for j in ih..times.len() {
        assert_eq!(f.table[0][j], 0.0);
        assert!(f.table[3][j] > 0.0);
    }

    // a scheduled stop at 50 s: nothing changes before it, then the active mass is an operator stop
    let mut p = normal(2.0);
    stop_at(&mut p, 50.0);
    let g = run(Family::AcmeNormal, BUSY, &p, &times);
    check_partition(&g);
    assert_eq!(g.scheduled_stop_s, Some(50.0));
    let i50 = times.iter().position(|t| *t == 50.0).unwrap();
    for j in 0..times.len() {
        if j < i50 {
            for r in 0..4 {
                assert!((g.table[r][j] - f.table[r][j]).abs() < 1e-12);
            }
        } else {
            assert_eq!(g.table[0][j], 0.0);
            assert_eq!(g.table[3][j], 0.0);
            assert!((g.table[1][j] - f.table[1][i50]).abs() < 1e-12);
            assert!((g.table[2][j] - (f.table[2][i50] + f.table[0][i50])).abs() < 1e-12);
        }
    }
    // a stop at or before the cutoff refuses the forecast
    let mut p = normal(2.0);
    stop_at(&mut p, 1.5);
    assert!(outcome(Family::AcmeNormal, BUSY, &p, &times).is_err());

    // with all three mechanisms on, a later cutoff lowers the weak share and keeps the partition
    let later = run(
        Family::AcmeNormal,
        BUSY,
        &normal(5.0),
        &[10.0, 100.0, 300.0],
    );
    check_partition(&later);
    assert!(later.p_weak_at_cutoff < f.p_weak_at_cutoff);
    assert!(f.clocks.p_limit_never > 0.0 && f.clocks.p_limit_never < 1.0);
}

fn with(p: &PrefixBundle, key: &str, v: f64) -> PrefixBundle {
    let mut p = p.clone();
    let m = p.known_conditions.get_mut(key).expect("condition");
    m.value_si = Some(v);
    m.raw_value = Some(ignis_core::contracts::RawValue::Number(v));
    p.input_hash = ignis_core::hashing::content_hash(&p, &["input_hash"]);
    p
}

fn refused_for(family: Family, extra: &str, p: &PrefixBundle, needle: &str) {
    match outcome(family, extra, p, &[60.0]) {
        Ok(_) => panic!("admitted, expected a refusal about {needle:?}"),
        Err(reasons) => assert!(
            reasons.iter().any(|r| r.contains(needle)),
            "{needle:?} not in {reasons:?}"
        ),
    }
}

#[test]
fn out_of_range_conditions_are_refused_not_defaulted() {
    let n = normal(1.0);
    for zst in [0.0, 1.0, 1.2, -0.1] {
        refused_for(Family::AcmeNormal, "", &with(&n, "zst", zst), "zst");
    }
    // the burner-side reactant of a normal flame is the injected fuel
    refused_for(
        Family::AcmeNormal,
        "",
        &with(&n, "x_fuel_role", 1.5),
        "mole fraction",
    );
    refused_for(
        Family::AcmeNormal,
        "",
        &with(&n, "x_fuel_role", 0.0),
        "x_fuel_role",
    );
    // pure fuel is a mole fraction of one, and admitted
    assert!(outcome(
        Family::AcmeNormal,
        "",
        &with(&n, "x_fuel_role", 1.0),
        &[60.0]
    )
    .is_ok());
    // of an inverse flame, the injected O2
    let i = inverse(1.0);
    refused_for(
        Family::AcmeInverse,
        "",
        &with(&i, "x_o2_role", 1.2),
        "mole fraction",
    );
    refused_for(
        Family::AcmeInverse,
        "",
        &with(&i, "x_o2_role", 0.0),
        "mole fraction",
    );
    refused_for(Family::AcmeInverse, "", &with(&i, "zst", 1.0), "zst");
    // a missing condition is a refusal
    let mut m = n.clone();
    m.known_conditions.remove("zst");
    m.input_hash = ignis_core::hashing::content_hash(&m, &["input_hash"]);
    assert!(outcome(Family::AcmeNormal, "", &m, &[60.0]).is_err());
    // two fuels at once
    let mut two = n.clone();
    for k in ["fuel_is_ethane", "fuel_is_butane"] {
        let mut v = two.known_conditions["zst"].clone();
        v.value_si = Some(1.0);
        two.known_conditions.insert(k.into(), v);
    }
    two.input_hash = ignis_core::hashing::content_hash(&two, &["input_hash"]);
    refused_for(Family::AcmeNormal, "", &two, "both 1");
    // a clock that overflows at the conditions (ln D = 2.4 + 2000 ln 2) is refused, not held
    let fast = "se.diff.skip[0:8] = [0.0, 2000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]\n";
    refused_for(
        Family::AcmeNormal,
        fast,
        &with(&n, "injected_mass_flow", 2.0e-6),
        "growth diffusivity",
    );
    // and the configurations are never pooled
    refused_for(Family::AcmeNormal, "", &inverse(1.0), "never pooled");
}

#[test]
fn the_committed_specs_are_the_specs_the_forward_was_compiled_with() {
    for (family, dir) in [
        (Family::AcmeNormal, "normal"),
        (Family::AcmeInverse, "inverse"),
    ] {
        let spec = flame3::spec(family);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(key(family)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let path = format!(
            "{}/../../configs/models/acme/wsp3/{dir}/spec.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&std::fs::read_to_string(&path).expect(&path)).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "{path} drifted from flame3::spec(); a changed spec is a new spec id (re-export with \
             `ignis wsp spec --family {} --out …`), never an in-place edit",
            key(family)
        );
        // the toy weights assemble into a release whose build checks all pass
        let r = release(family, "");
        let checks = &r.release.construction.as_ref().unwrap().checks;
        assert!(
            checks
                .iter()
                .all(|c| c.status == ignis_core::contracts::CheckStatus::Passed),
            "{checks:?}"
        );
    }
}

#[test]
fn the_oscillation_clock_alone_is_a_log_logistic_after_the_floor() {
    // growth held (Phi about 0.12), hold off, burner off; osc median 40 s, shape 3, floor 5 s
    let extra =
        "se.flux.b0 = -3.0\nse.osc.b0 = 3.6888794541139363\nse.osc_shape = 3.0\nse.floor_s = 5.0\n";
    let times = [4.0, 5.0, 20.0, 45.0, 120.0, 500.0];
    let f = run(Family::AcmeNormal, extra, &normal(1.0), &times);
    assert!(f.clocks.node_tau_s.iter().all(Option::is_none));
    assert!(rel(f.clocks.osc_tau_s, 40.0) < 1e-12);
    for (j, t) in times.iter().enumerate() {
        let want = 1.0 - survival((t - 5.0).max(0.0), 40.0, 3.0);
        assert!(
            (f.table[1][j] - want).abs() < 1e-9,
            "t = {t}: {} vs {want}",
            f.table[1][j]
        );
    }
    assert_eq!(f.table[1][0], 0.0, "nothing before the floor");
    check_partition(&f);
}

/// The burner-heating clock alone: growth held, oscillation and weak class off; dT_eq = e^6 K
/// (403 K) above t0 = 20 degC with tau = 50 s, bt.sigma = 0, shape 8.
const HEAT: &str = "se.flux.b0 = -3.0\nbt.rise.b0 = 6.0\n";

/// Time for the toy burner to reach `limit` degC by hand: T_b = t0 + dT (1 - exp(-t / tau)).
fn limit_hand(limit: f64) -> Option<f64> {
    let (t0, tau, dt) = (20.0, 50.0, 6.0_f64.exp());
    (dt > limit - t0).then(|| -tau * (1.0 - (limit - t0) / dt).ln())
}

#[test]
fn the_burner_heating_clock_reaches_its_limits_by_the_heating_law() {
    let times = [10.0, 40.0, 59.0, 80.0, 200.0, 590.0];
    let cut = 1.0;
    // one limit that is reached
    let one =
        format!("{HEAT}bt.limit_c[0:3] = [300.0, 0.0, 0.0]\nbt.limit_w[0:3] = [1.0, 0.0, 0.0]\n");
    let f = run(Family::AcmeNormal, &one, &normal(cut), &times);
    let t_star = limit_hand(300.0).expect("reached");
    assert!(rel(f.clocks.burner_rise_k, 6.0_f64.exp()) < 1e-12);
    assert_eq!(f.clocks.burner_limits.len(), 1);
    let (lim, w, t_c) = f.clocks.burner_limits[0];
    assert_eq!((lim, w), (300.0, 1.0));
    assert!(rel(t_c.expect("reached"), t_star) < 1e-12);
    assert_eq!(f.clocks.p_limit_never, 0.0);
    for (j, t) in times.iter().enumerate() {
        let want = 1.0 - survival(*t, t_star, 8.0) / survival(cut, t_star, 8.0);
        assert!(
            (f.table[2][j] - want).abs() < 1e-9,
            "t = {t}: {} vs {want}",
            f.table[2][j]
        );
        assert!(f.table[1][j] < 1e-12, "no self-extinction is written");
    }
    check_partition(&f);

    // a limit above t0 + dT_eq is never reached
    let never =
        format!("{HEAT}bt.limit_c[0:3] = [500.0, 0.0, 0.0]\nbt.limit_w[0:3] = [1.0, 0.0, 0.0]\n");
    assert!(limit_hand(500.0).is_none());
    let g = run(Family::AcmeNormal, &never, &normal(cut), &times);
    assert!((g.clocks.p_limit_never - 1.0).abs() < 1e-12);
    assert_eq!(g.clocks.burner_limits[0].2, None);
    assert!(g.table[2].iter().all(|p| *p < 1e-12));

    // two limits with weights 1 : 3 give the weighted mixture
    let two =
        format!("{HEAT}bt.limit_c[0:3] = [300.0, 380.0, 0.0]\nbt.limit_w[0:3] = [0.2, 0.6, 0.0]\n");
    let h = run(Family::AcmeNormal, &two, &normal(cut), &times);
    let (t1, t2) = (limit_hand(300.0).unwrap(), limit_hand(380.0).unwrap());
    let s = |t: f64| 0.25 * survival(t, t1, 8.0) + 0.75 * survival(t, t2, 8.0);
    for (j, t) in times.iter().enumerate() {
        let want = 1.0 - s(*t) / s(cut);
        assert!(
            (h.table[2][j] - want).abs() < 1e-9,
            "t = {t}: {} vs {want}",
            h.table[2][j]
        );
    }
    assert!((h.clocks.burner_limits[0].1 - 0.25).abs() < 1e-12);

    // one reached and one never, weights 1 : 1: half of the pairs never reach
    let half =
        format!("{HEAT}bt.limit_c[0:3] = [300.0, 500.0, 0.0]\nbt.limit_w[0:3] = [0.5, 0.5, 0.0]\n");
    let k = run(Family::AcmeNormal, &half, &normal(cut), &times);
    assert!((k.clocks.p_limit_never - 0.5).abs() < 1e-12);
    let s = |t: f64| 0.5 * survival(t, t1, 8.0) + 0.5;
    let last = times.len() - 1;
    assert!((k.table[2][last] - (1.0 - s(590.0) / s(cut))).abs() < 1e-9);

    // with a spread on dT_eq, the share that never reaches is the share of nodes below the limit
    let spread = format!("{one}bt.sigma = 0.5\n");
    let m = run(Family::AcmeNormal, &spread, &normal(cut), &times);
    let n = 7;
    let below = (0..n)
        .filter(|j| (6.0 + 0.5 * z_of((*j as f64 + 0.5) / n as f64)).exp() <= 280.0)
        .count() as f64
        / n as f64;
    assert!(below > 0.0 && below < 1.0);
    assert!((m.clocks.p_limit_never - below).abs() < 1e-12);

    // a weighted limit at or below t0 is refused: the forward refuses it on the fixture, so the
    // release cannot even be assembled (its finite-forward build check fails)
    let at_t0 =
        format!("{HEAT}bt.limit_c[0:3] = [10.0, 0.0, 0.0]\nbt.limit_w[0:3] = [1.0, 0.0, 0.0]\n");
    let model = models::by_key(key(Family::AcmeNormal)).unwrap();
    let spec = model.spec();
    let src: String = TOY
        .lines()
        .filter(|l| {
            !l.starts_with("bt.limit_")
                && !l.starts_with("se.flux.b0")
                && !l.starts_with("bt.rise.b0")
        })
        .map(|l| format!("{l}\n"))
        .collect::<String>()
        + &at_t0;
    let ops = text::compile(&src, &spec).unwrap_or_else(|e| panic!("{e}\n{src}"));
    let patch = assemble::make_patch(&spec, common::meta("test-flame3-t0"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let err = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect_err("a limit below t0 must not assemble");
    assert!(format!("{err:#}").contains("failed build check"), "{err:#}");
}

#[test]
fn the_burner_features_are_what_the_spec_says() {
    let pi4 = 4.0 * std::f64::consts::PI;
    // normal: propane 30 % in N2, 1 mg/s, Z_st 0.25
    let f = run(Family::AcmeNormal, "", &normal(1.0), &[60.0]);
    let y = 0.30 * MW_PROPANE / (0.30 * MW_PROPANE + 0.70 * MW_N2);
    let m_tot = 1.0e-6 / y;
    let r_ss_mm = m_tot / (pi4 * 0.05e-3 * (1.0 / 0.75_f64).ln()) * 1.0e3;
    let b = f.clocks.burner_features;
    assert!((b[0] - (r_ss_mm / 25.0).ln()).abs() < 1e-9);
    assert!((b[1] - (m_tot / 3.0e-6).ln()).abs() < 1e-9);
    let c = f.features;
    assert_eq!(&b[2..], &[c[4], c[3], c[0], c[6], c[7]]);
    // inverse: O2 50 % in N2, 5 mg/s of O2, Z_st 0.45, T_ad 2400 K (c_tad = 2/3)
    let g = run(Family::AcmeInverse, "", &inverse(1.0), &[60.0]);
    let y = 0.50 * MW_O2 / (0.50 * MW_O2 + 0.50 * MW_N2);
    let m_tot = 5.0e-6 / y;
    let r_ss_mm = m_tot / (pi4 * 0.05e-3 * (1.0 / 0.45_f64).ln()) * 1.0e3;
    let b = g.clocks.burner_features;
    assert!((b[0] - (r_ss_mm / 25.0).ln()).abs() < 1e-9);
    assert!((b[1] - (m_tot / 3.0e-6).ln()).abs() < 1e-9);
    assert!((b[2] - 2.0 / 3.0).abs() < 1e-12);
    let c = g.features;
    assert_eq!(&b[2..], &[c[4], c[3], c[0], c[6], c[7]]);
}

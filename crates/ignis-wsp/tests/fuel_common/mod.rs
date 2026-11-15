//! Shared test data of the forwards with the fuel table (`flame5.rs`, `flame6.rs`, `flame7.rs`,
//! `committee3.rs`): toy law weights that read the two indicator entries through skips and hinges,
//! the neutral table, the tables of section 3.4 of `docs/wsp/research/phys_spherical_fuels.md`
//! (copied, not typed here), and prefixes. Test data only, not a candidate.
#![allow(dead_code)]

use ignis_core::contracts::{Family, PrefixBundle, RawValue, ReleaseId, SupportGate};
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::{acme_tools, assemble, models, text};

pub const FAMILIES: [(Family, &str); 2] = [
    (Family::AcmeNormal, "normal"),
    (Family::AcmeInverse, "inverse"),
];

/// Forecast times on both sides of the programmed stop at 66 s.
pub const TIMES: [f64; 12] = [
    3.0, 5.0, 6.0, 15.0, 40.0, 65.8, 66.4, 90.0, 200.0, 500.0, 600.0, 610.0,
];

/// Hinge laws (`flame.rs`, and the hinge part of `flame4.rs`) that read both indicators through
/// their skips and hidden units, with a weak class, a floor, a programmed stop whose logit reads the
/// indicators, and a background hazard.
pub const HINGE: &str = "
se.tau.b0 = 3.4
se.tau.skip[0:8] = [0.2, -0.4, 0.3, 0.1, -0.2, 0.1, 0.3, -0.3]
se.tau.W1[0, 1] = 1.0
se.tau.W1[0, 6] = 0.8
se.tau.W1[0, 7] = -0.6
se.tau.b1[0] = 0.2
se.tau.w2[0] = -0.5
se.flash.b0 = -1.0
se.flash.skip[0:8] = [0.0, 0.5, 0.0, 0.0, 0.0, 0.0, -0.7, 0.9]
se.flash.W1[0, 7] = 2.0
se.flash.b1[0] = -0.5
se.flash.w2[0] = 1.5
se.floor_s = 4.0
se.shape = 2.0
se.flash_tau_s = 1.0
se.flash_shape = 4.0
ft.atom_t_s[0:8] = [66.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
ft.atom_logit[0:8] = [0.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0]
ft.atom_w[0, 1] = 0.7
ft.atom_w[0, 6] = -1.2
ft.atom_w[0, 7] = 0.9
ft.log_base = -7.0
";

/// The burner-limit clock of `flame.rs` (median e^4 s at c = 0), reading both indicators.
pub const LIMIT_CLOCK: &str = "
ft.tau.b0 = 4.0
ft.tau.skip[0:8] = [0.0, -0.3, 0.0, 0.0, 0.0, 0.0, 0.4, -0.5]
ft.tau.W1[1, 6] = 1.0
ft.tau.b1[1] = -0.3
ft.tau.w2[1] = 0.7
ft.shape = 3.0
";

/// The burner-heating clock (`flame3.rs`, `flame4.rs`): dT_eq about e^6 K above 20 degC, a spread
/// on it, three limits, and a rise network that reads both indicators.
pub const BURNER: &str = "
bt.rise.b0 = 6.0
bt.rise.skip[0:7] = [0.3, -0.2, 0.1, 0.05, -0.1, 0.2, -0.3]
bt.rise.W1[0, 5] = 1.0
bt.rise.W1[0, 6] = 0.5
bt.rise.b1[0] = 0.1
bt.rise.w2[0] = 0.2
se.rho_d_g_m_s = 0.05
bt.tau_s = 50.0
bt.t0_c = 20.0
bt.sigma = 0.3
bt.shape = 8.0
bt.limit_c[0:3] = [300.0, 380.0, 1400.0]
bt.limit_w[0:3] = [0.2, 0.6, 0.2]
";

/// The growth laws of `flame3.rs` besides the burner clock, each reading both indicators.
pub const GROWTH: &str = "
se.flux.b0 = -0.2
se.flux.skip[0:8] = [0.0, 0.1, 0.0, 0.0, 0.0, 0.0, 0.3, -0.2]
se.flux.W1[0, 7] = 1.0
se.flux.b1[0] = -0.5
se.flux.w2[0] = 0.4
se.diff.b0 = 2.4
se.diff.skip[0:8] = [0.0, 0.0, 0.1, 0.0, 0.0, 0.0, -0.2, 0.25]
se.diff.W1[0, 6] = 1.0
se.diff.b1[0] = -0.2
se.diff.w2[0] = 0.3
se.osc.b0 = 4.0
se.osc.skip[0:8] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, -0.4]
se.osc.W1[0, 7] = 1.0
se.osc.b1[0] = 0.0
se.osc.w2[0] = -0.3
se.sigma_phi = 0.3
se.hold_log_rate = -6.0
se.osc_shape = 4.0
se.flash.b0 = -1.0
se.flash.skip[0:8] = [0.0, 0.5, 0.0, 0.0, 0.0, 0.0, -0.7, 0.9]
se.flash.W1[0, 7] = 2.0
se.flash.b1[0] = -0.5
se.flash.w2[0] = 1.5
se.floor_s = 4.0
se.shape = 2.0
se.flash_tau_s = 1.0
se.flash_shape = 4.0
ft.atom_t_s[0:8] = [66.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
ft.atom_logit[0:8] = [0.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0]
ft.atom_w[0, 1] = 0.7
ft.atom_w[0, 6] = -1.2
ft.atom_w[0, 7] = 0.9
ft.log_base = -7.0
";

/// The neutral table of a root (the committed roots type the same lines).
pub const NEUTRAL: &str = "
fuel.admit[0:6] = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0]
fuel.mw_g_mol[0:6] = [30.07, 44.1, 58.12, 30.07, 44.1, 58.12]
fuel.s[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
fuel.se_coord[0:6, 0:2] = [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]
fuel.ft_coord[0:6, 0:2] = [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]
fuel.dosc[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
fuel.tad_min_k[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
";

/// The normal-flame table of section 3.4 of the design, copied line by line.
pub const NORMAL_34: &str = "
fuel.admit[0:6] = [1.0, 1.0, 1.0, 1.0, 1.0, 0.0]
fuel.mw_g_mol[0:6] = [30.07, 44.10, 58.12, 28.054, 16.043, 2.016]
fuel.s[0:6] = [0.0, 0.0, 0.0, -0.085, 0.068, 0.757]
fuel.se_coord[0, 0:2] = [1.0, 0.0]
fuel.se_coord[1, 0:2] = [0.0, 0.0]
fuel.se_coord[2, 0:2] = [0.0, 1.0]
fuel.se_coord[3, 0:2] = [0.5, 0.5]
fuel.se_coord[4, 0:2] = [1.0, 0.0]
fuel.se_coord[5, 0:2] = [0.0, 0.0]
fuel.ft_coord[0, 0:2] = [1.0, 0.0]
fuel.ft_coord[1, 0:2] = [0.0, 0.0]
fuel.ft_coord[2, 0:2] = [0.0, 1.0]
fuel.ft_coord[3, 0:2] = [1.0, 0.0]
fuel.ft_coord[4, 0:2] = [1.9, -0.9]
fuel.ft_coord[5, 0:2] = [0.0, 0.0]
fuel.dosc[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
fuel.tad_min_k[0:6] = [0.0, 0.0, 0.0, 1900.0, 1900.0, 0.0]
";

/// The inverse-flame table of section 3.4 of the design, copied line by line.
pub const INVERSE_34: &str = "
fuel.admit[0:6] = [1.0, 1.0, 1.0, 1.0, 0.0, 0.0]
fuel.mw_g_mol[0:6] = [30.07, 44.10, 58.12, 28.054, 16.043, 2.016]
fuel.s[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
fuel.se_coord[0, 0:2] = [1.0, 0.0]
fuel.se_coord[1, 0:2] = [0.0, 0.0]
fuel.se_coord[2, 0:2] = [0.0, 1.0]
fuel.se_coord[3, 0:2] = [1.0, 0.0]
fuel.se_coord[4, 0:2] = [0.0, 0.0]
fuel.se_coord[5, 0:2] = [0.0, 0.0]
fuel.ft_coord[0, 0:2] = [1.0, 0.0]
fuel.ft_coord[1, 0:2] = [0.0, 0.0]
fuel.ft_coord[2, 0:2] = [0.0, 1.0]
fuel.ft_coord[3, 0:2] = [0.5, 0.0]
fuel.ft_coord[4, 0:2] = [0.0, 0.0]
fuel.ft_coord[5, 0:2] = [0.0, 0.0]
fuel.dosc[0:6] = [0.0, 0.0, 0.0, 0.7, 0.0, 0.0]
fuel.tad_min_k[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
";

pub fn key(family: Family, generation: u8) -> String {
    let name = if family == Family::AcmeInverse {
        "acme_inverse"
    } else {
        "acme_normal"
    };
    format!("{name}{generation}")
}

pub fn config(path: &str) -> String {
    let path = format!(
        "{}/../../configs/models/acme/{path}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn name_of(l: &str) -> String {
    l.trim_start()
        .split(['[', ' ', '='])
        .next()
        .unwrap_or_default()
        .to_string()
}

/// The statements of `base` without those of the tensors `extra` writes, then `extra`.
pub fn merged(base: &str, extra: &str) -> String {
    let written: Vec<String> = extra
        .lines()
        .map(name_of)
        .filter(|n| !n.is_empty())
        .collect();
    let mut src = String::new();
    for line in base.lines() {
        let name = name_of(line);
        if name.is_empty() || !written.contains(&name) {
            src.push_str(line);
            src.push('\n');
        }
    }
    src.push_str(extra);
    src
}

/// An in-memory root release of `key` from an edits text.
pub fn try_release(key: &str, src: &str) -> Result<LoadedRelease, String> {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let ops = text::compile(src, &spec).map_err(|e| format!("{key}: {e}\n{src}"))?;
    let patch = assemble::make_patch(&spec, crate::common::meta("test-fuel"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).map_err(|e| format!("{e:#}"))?;
    let mut release = model
        .build_release(&patch, &tensors, None, &crate::common::binding())
        .map_err(|e| format!("{e:#}"))?;
    release.id = ignis_core::hashing::content_hash(&release, &["id", "created_at"]);
    let _ = ReleaseId::new(release.id.clone());
    Ok(LoadedRelease {
        release,
        params: tensors,
        posterior: TensorMap::new(),
    })
}

pub fn release(key: &str, src: &str) -> LoadedRelease {
    try_release(key, src).unwrap_or_else(|e| panic!("{e}"))
}

pub fn set(r: &mut LoadedRelease, tensor: &str, data: &[f64]) {
    let t = r.params.get_mut(tensor).expect("tensor");
    assert_eq!(t.data.len(), data.len());
    t.data = data.to_vec();
}

pub fn rehash(p: &mut PrefixBundle) {
    p.input_hash = ignis_core::hashing::content_hash(&*p, &["input_hash"]);
}

/// A prefix of `family` at mid-range conditions for the fuel named `species`.
pub fn prefix(family: Family, species: &str) -> PrefixBundle {
    match family {
        Family::AcmeInverse => acme_tools::fixture_prefix(
            family, 3.0, 101_325.0, 5.0e-6, 0.50, 0.10, 2400.0, 0.45, species,
        ),
        _ => acme_tools::fixture_prefix(
            family, 3.0, 101_325.0, 1.0e-6, 0.30, 0.30, 2300.0, 0.25, species,
        ),
    }
}

/// A second prefix away from the first: another pressure, flow, oxygen, temperature.
pub fn other_prefix(family: Family, species: &str) -> PrefixBundle {
    match family {
        Family::AcmeInverse => acme_tools::fixture_prefix(
            family, 5.0, 150_000.0, 7.0e-6, 0.40, 0.15, 2500.0, 0.50, species,
        ),
        _ => acme_tools::fixture_prefix(
            family, 5.0, 150_000.0, 1.6e-6, 0.35, 0.50, 2350.0, 0.30, species,
        ),
    }
}

/// The prefix with the stored indicators `(fuel_is_ethane, fuel_is_butane)` added, and without its
/// species when `keep_species` is false.
pub fn with_indicators(p: &PrefixBundle, e: f64, b: f64, keep_species: bool) -> PrefixBundle {
    let mut p = p.clone();
    let mut m = p.known_conditions["pressure"].clone();
    m.raw_unit = Some("1".into());
    m.unit_si = Some("1".into());
    for (k, v) in [("fuel_is_ethane", e), ("fuel_is_butane", b)] {
        let mut m = m.clone();
        m.raw_value = Some(RawValue::Number(v));
        m.value_si = Some(v);
        p.known_conditions.insert(k.into(), m);
    }
    if !keep_species {
        p.known_conditions.remove("fuel_species");
    }
    rehash(&mut p);
    p
}

/// The prefix with one numeric condition changed.
pub fn with(p: &PrefixBundle, key: &str, v: f64) -> PrefixBundle {
    let mut p = p.clone();
    let m = p.known_conditions.get_mut(key).expect("condition");
    m.value_si = Some(v);
    m.raw_value = Some(RawValue::Number(v));
    rehash(&mut p);
    p
}

/// The alkane prefixes the neutrality tests run: by species, by stored indicators only, and by
/// both, at two states.
pub fn alkane_prefixes(family: Family) -> Vec<(String, PrefixBundle)> {
    let mut out = Vec::new();
    for (name, e, b) in [
        ("ethane", 1.0, 0.0),
        ("propane", 0.0, 0.0),
        ("butane", 0.0, 1.0),
    ] {
        for (state, p) in [
            ("mid", prefix(family, name)),
            ("other", other_prefix(family, name)),
        ] {
            out.push((format!("{name} {state} by species"), p.clone()));
            out.push((
                format!("{name} {state} by indicators"),
                with_indicators(&p, e, b, false),
            ));
            out.push((
                format!("{name} {state} by both"),
                with_indicators(&p, e, b, true),
            ));
        }
    }
    out
}

/// `N(x) = b0 + skip . x + sum_j w2[j] relu(b1[j] + W1[j, :] . x)` by hand, from the release.
pub fn net(r: &LoadedRelease, name: &str, x: &[f64]) -> f64 {
    let v = |t: &str| r.params[&format!("{name}.{t}")].data.clone();
    let (w1, b1, w2, skip) = (v("W1"), v("b1"), v("w2"), v("skip"));
    let b0 = v("b0")[0];
    let n = x.len();
    let mut y = b0;
    for k in 0..n {
        y += skip[k] * x[k];
    }
    for j in 0..b1.len() {
        let mut h = b1[j];
        for k in 0..n {
            h += w1[j * n + k] * x[k];
        }
        y += w2[j] * h.max(0.0);
    }
    y
}

/// `x` with its indicator entries `(ie, ib)` set to `v`.
pub fn at(x: &[f64], ie: usize, ib: usize, v: [f64; 2]) -> Vec<f64> {
    let mut y = x.to_vec();
    y[ie] = v[0];
    y[ib] = v[1];
    y
}

/// The vertex-weighted network output by hand: `e N(E) + (1 - e - b) N(P) + b N(B)`.
pub fn weighed_by_hand(
    r: &LoadedRelease,
    name: &str,
    x: &[f64],
    ie: usize,
    ib: usize,
    coord: [f64; 2],
) -> f64 {
    let [e, b] = coord;
    e * net(r, name, &at(x, ie, ib, [1.0, 0.0]))
        + (1.0 - e - b) * net(r, name, &at(x, ie, ib, [0.0, 0.0]))
        + b * net(r, name, &at(x, ie, ib, [0.0, 1.0]))
}

pub fn close(have: f64, want: f64, what: &str) {
    assert!(
        (have - want).abs() <= 1e-12 * want.abs().max(1.0),
        "{what}: {have} against {want}"
    );
}

pub fn check_partition(table: &[Vec<f64>; 4]) {
    for j in 0..table[0].len() {
        let sum: f64 = (0..4).map(|r| table[r][j]).sum();
        assert!((sum - 1.0).abs() < 1e-12, "rows sum to {sum}");
        assert!((0..4).all(|r| table[r][j] >= 0.0));
    }
}

/// A condition-gate refusal whose reasons contain `needle`.
pub fn assert_refused(out: Result<(), ignis_core::contracts::Support>, needle: &str, what: &str) {
    match out {
        Ok(()) => panic!("{what}: admitted, expected a refusal with {needle:?}"),
        Err(s) => {
            assert!(!s.admitted);
            assert_eq!(s.gate, SupportGate::Condition, "{what}: {:?}", s.reasons);
            assert!(
                s.reasons.iter().any(|r| r.contains(needle)),
                "{what}: {:?} has no {needle:?}",
                s.reasons
            );
        }
    }
}

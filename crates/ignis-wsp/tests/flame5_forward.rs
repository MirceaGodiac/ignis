//! The hinge-law clocks with the fuel table (`flame5.rs`) on synthetic prefixes with test-only
//! weights: its spec is the `flame.rs` spec plus the fuel table; with the neutral table, and with the
//! tables of section 3.4 of the design, every alkane forecast is that of `flame.rs` bit for bit; a
//! unit coordinate evaluates one vertex, a zero weight none; the ethylene and methane rows give the
//! weighed network outputs a hand computation gives; an unknown fuel, a fuel not admitted,
//! disagreeing indicators and a temperature below the bound are refused at the condition gate; the
//! committed roots hold their source's numbers.

mod common;
mod fuel_common;

use fuel_common::*;
use ignis_core::contracts::{CheckStatus, ControlStage, Family, ModelRelease, ModelSpec, Support};
use ignis_core::registry::LoadedRelease;
use ignis_wsp::nn::sigmoid;
use ignis_wsp::{audit, flame, flame5, fuel, models};
use serde_json::json;

fn run2(r: &LoadedRelease, p: &ignis_core::contracts::PrefixBundle) -> flame::Forecast {
    flame::forecast(&flame::spec(p.family), r, p, &json!({ "times_s": TIMES }))
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn try5(
    r: &LoadedRelease,
    p: &ignis_core::contracts::PrefixBundle,
) -> Result<flame5::Forecast, Support> {
    flame5::forecast(&flame5::spec(p.family), r, p, &json!({ "times_s": TIMES })).expect("runnable")
}

fn run5(r: &LoadedRelease, p: &ignis_core::contracts::PrefixBundle) -> flame5::Forecast {
    try5(r, p).unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn old(family: Family) -> LoadedRelease {
    release(&key(family, 2), &format!("{HINGE}{LIMIT_CLOCK}"))
}

fn new(family: Family, table: &str) -> LoadedRelease {
    release(&key(family, 9), &format!("{HINGE}{LIMIT_CLOCK}{table}"))
}

fn table_of(family: Family) -> &'static str {
    if family == Family::AcmeInverse {
        INVERSE_34
    } else {
        NORMAL_34
    }
}

fn vertices(f: &flame5::Forecast, law: &str) -> Vec<&'static str> {
    f.clocks
        .laws
        .iter()
        .find(|e| e.law == law)
        .unwrap_or_else(|| panic!("no law {law}"))
        .vertices
        .clone()
}

#[test]
fn the_spec_is_the_hinge_spec_plus_the_fuel_table() {
    for (family, dir) in FAMILIES {
        let spec = flame5::spec(family);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(&key(family, 9)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp9/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/acme/wsp9/{dir}/spec.json drifted from flame5::spec(); a changed spec \
             is a new spec id (re-export with `ignis wsp spec --family {} --out …`)",
            key(family, 9)
        );
        let hinge = flame::spec(family);
        let n = hinge.tensors.len();
        assert_eq!(&spec.tensors[..n], &hinge.tensors[..]);
        let table: Vec<&str> = spec.tensors[n..].iter().map(|t| t.name.as_str()).collect();
        assert_eq!(table, fuel::TENSORS);
        assert_eq!(spec.tensors[n..].to_vec(), fuel::tensors());
        assert_eq!(spec.domain, hinge.domain);
        assert_eq!(spec.target, hinge.target);
        let r = new(family, NEUTRAL);
        let checks = &r.release.construction.as_ref().unwrap().checks;
        assert!(
            checks.iter().all(|c| c.status == CheckStatus::Passed),
            "{checks:?}"
        );
    }
}

#[test]
fn with_the_neutral_table_or_the_tables_of_the_design_every_alkane_forecast_is_bit_for_bit() {
    for (family, _) in FAMILIES {
        let o = old(family);
        for table in [NEUTRAL, table_of(family)] {
            let n = new(family, table);
            for (what, p) in alkane_prefixes(family) {
                let (a, b) = (run2(&o, &p), run5(&n, &p));
                assert_eq!(a.table, b.table, "{family:?} {what}");
                assert_eq!(a.times_s, b.times_s);
                assert_eq!(a.p_weak_at_cutoff.to_bits(), b.p_weak_at_cutoff.to_bits());
                assert_eq!(a.features, b.features, "{what}");
                check_partition(&b.table);
                // one vertex per law, the fuel's own
                let own = match what.split(' ').next().unwrap() {
                    "ethane" => "ethane",
                    "propane" => "propane",
                    _ => "butane",
                };
                for law in ["se.tau", "se.flash", "ft.tau"] {
                    assert_eq!(vertices(&b, law), vec![own], "{what} {law}");
                }
                // a scheduled stop is read as before
                let mut q = p.clone();
                q.known_controls.push(ControlStage {
                    t_start_s: 50.0,
                    t_end_s: None,
                    control: "scheduled_stop".into(),
                    value: None,
                    operator_action: false,
                });
                rehash(&mut q);
                assert_eq!(run2(&o, &q).table, run5(&n, &q).table, "{what} with a stop");
            }
            // both spellings of butane, and case and blanks, are one row
            let b = run5(&n, &prefix(family, "butane"));
            assert_eq!(run5(&n, &prefix(family, " N-Butane ")).table, b.table);
        }
    }
}

#[test]
fn a_unit_coordinate_evaluates_one_vertex_and_a_zero_weight_none() {
    let n = new(Family::AcmeNormal, NORMAL_34);
    // ethylene: self-extinction (0.5, 0.5), so propane's weight 1 - 0.5 - 0.5 is zero and its vertex
    // is not evaluated; burner (1, 0)
    let f = run5(&n, &prefix(Family::AcmeNormal, "ethylene"));
    assert_eq!(vertices(&f, "se.tau"), vec!["ethane", "butane"]);
    assert_eq!(vertices(&f, "se.flash"), vec!["ethane", "butane"]);
    assert_eq!(vertices(&f, "ft.tau"), vec!["ethane"]);
    // methane: burner (1.9, -0.9). In IEEE arithmetic 1 - 1.9 + 0.9 is 1.1e-16, not 0: the propane
    // vertex is evaluated with that weight (the most literal reading of "weight zero"; it moves the
    // clock by about 1e-16 of the propane output)
    let f = run5(&n, &prefix(Family::AcmeNormal, "methane"));
    assert_eq!(vertices(&f, "se.tau"), vec!["ethane"]);
    let ft = f.clocks.laws.iter().find(|e| e.law == "ft.tau").unwrap();
    assert_eq!(ft.vertices, vec!["ethane", "propane", "butane"]);
    assert_eq!(ft.weights, [1.9, 1.0 - 1.9 - -0.9, -0.9]);
    assert!(ft.weights[1] != 0.0 && ft.weights[1].abs() < 1e-15);
}

#[test]
fn the_ethylene_and_methane_rows_give_the_weighed_network_outputs_by_hand() {
    let (ie, ib) = (flame5::I_ETHANE, flame5::I_BUTANE);
    let (e_v, b_v) = ([1.0, 0.0], [0.0, 1.0]);
    // normal flames, methane: se (1, 0), ft (1.9, -0.9), s = 0.068
    let n = new(Family::AcmeNormal, NORMAL_34);
    let f = run5(&n, &prefix(Family::AcmeNormal, "methane"));
    let c = f.features;
    let mut c_tau = c.to_vec();
    c_tau[1] += 0.068;
    close(
        f.clocks.radiative_tau_s.ln(),
        net(&n, "se.tau", &at(&c_tau, ie, ib, e_v)),
        "methane ln tau_R",
    );
    close(
        f.clocks.p_weak,
        sigmoid(net(&n, "se.flash", &at(&c, ie, ib, e_v))),
        "methane p_weak",
    );
    close(
        f.clocks.burner_tau_s.ln(),
        weighed_by_hand(&n, "ft.tau", &c, ie, ib, [1.9, -0.9]),
        "methane ln tau_B",
    );
    // the stop logit reads the coordinates: 0 + 0.7 c_logm - 1.2 (1.9) + 0.9 (-0.9)
    let p_stop = sigmoid(0.7 * c[1] - 1.2 * 1.9 + 0.9 * -0.9);
    close(f.clocks.programmed[0].1, p_stop, "methane stop probability");
    assert_eq!(&c[6..], &[1.9, -0.9]);
    check_partition(&f.table);
    // normal flames, ethylene: se (0.5, 0.5), ft (1, 0), s = -0.085
    let f = run5(&n, &prefix(Family::AcmeNormal, "ethylene"));
    let c = f.features;
    let mut c_tau = c.to_vec();
    c_tau[1] += -0.085;
    close(
        f.clocks.radiative_tau_s.ln(),
        weighed_by_hand(&n, "se.tau", &c_tau, ie, ib, [0.5, 0.5]),
        "ethylene ln tau_R",
    );
    close(
        f.clocks.p_weak,
        sigmoid(
            0.5 * net(&n, "se.flash", &at(&c, ie, ib, e_v))
                + 0.5 * net(&n, "se.flash", &at(&c, ie, ib, b_v)),
        ),
        "ethylene p_weak",
    );
    close(
        f.clocks.burner_tau_s.ln(),
        net(&n, "ft.tau", &at(&c, ie, ib, e_v)),
        "ethylene ln tau_B",
    );
    // inverse flames, ethylene: se (1, 0), ft (0.5, 0); the inverse laws do not read fuel.s
    let mut n = new(Family::AcmeInverse, INVERSE_34);
    let f = run5(&n, &prefix(Family::AcmeInverse, "ethylene"));
    let c = f.features;
    close(
        f.clocks.burner_tau_s.ln(),
        weighed_by_hand(&n, "ft.tau", &c, ie, ib, [0.5, 0.0]),
        "inverse ethylene ln tau_B",
    );
    close(
        f.clocks.radiative_tau_s.ln(),
        net(&n, "se.tau", &at(&c, ie, ib, e_v)),
        "inverse ethylene ln tau_R",
    );
    set(&mut n, "fuel.s", &[0.0, 0.0, 0.0, 0.5, 0.0, 0.0]);
    let g = run5(&n, &prefix(Family::AcmeInverse, "ethylene"));
    assert_eq!(g.clocks.radiative_tau_s, f.clocks.radiative_tau_s);
    assert_eq!(g.table, f.table);
    assert_eq!(g.clocks.radiative_logm_shift, 0.0);
}

#[test]
fn an_unknown_fuel_a_fuel_not_admitted_disagreeing_indicators_and_a_cool_flame_are_refused() {
    for (family, _) in FAMILIES {
        let neutral = new(family, NEUTRAL);
        let design = new(family, table_of(family));
        let gate = |r: &LoadedRelease, p| try5(r, &p).map(|_| ());
        let p = prefix(family, "ethylene");
        assert_refused(
            gate(&neutral, p.clone()),
            "does not answer for ethylene",
            "neutral ethylene",
        );
        assert!(try5(&design, &p).is_ok(), "the design admits ethylene");
        assert_refused(
            gate(&design, prefix(family, "pentane")),
            "is not a fuel of the table",
            "pentane",
        );
        assert_refused(
            gate(&design, prefix(family, "hydrogen")),
            "does not answer for hydrogen",
            "hydrogen",
        );
        // stored indicators that are not the row's own
        let ethane = prefix(family, "ethane");
        assert_refused(
            gate(&neutral, with_indicators(&ethane, 0.0, 1.0, true)),
            "disagree",
            "ethane named butane",
        );
        assert_refused(
            gate(&design, with_indicators(&p, 1.0, 0.0, true)),
            "fuel_is_ethane = 1 is not that of fuel_species \"ethylene\"",
            "ethylene named ethane",
        );
        assert!(try5(&design, &with_indicators(&p, 0.0, 0.0, true)).is_ok());
        // indicators alone that name no fuel, or only one of them
        assert_refused(
            gate(&neutral, with_indicators(&ethane, 1.0, 1.0, false)),
            "name no fuel",
            "both indicators",
        );
        assert_refused(
            gate(&neutral, with_indicators(&ethane, 0.5, 0.0, false)),
            "name no fuel",
            "a fractional indicator",
        );
        let mut one = with_indicators(&ethane, 1.0, 0.0, false);
        one.known_conditions.remove("fuel_is_butane");
        rehash(&mut one);
        assert_refused(
            gate(&neutral, one),
            "fuel_is_butane missing",
            "one indicator",
        );
    }
    // the weak-flame bound of the normal table: 1900 K for ethylene and methane, refused below it
    let design = new(Family::AcmeNormal, NORMAL_34);
    for fuel in ["ethylene", "methane"] {
        let p = with(&prefix(Family::AcmeNormal, fuel), "tad", 1899.0);
        assert_refused(try5(&design, &p).map(|_| ()), "is below the bound", fuel);
        assert!(try5(&design, &with(&p, "tad", 1900.0)).is_ok());
    }
    let p = with(&prefix(Family::AcmeNormal, "propane"), "tad", 1899.0);
    assert!(try5(&design, &p).is_ok(), "propane has no bound");
}

#[test]
fn the_forward_has_its_own_identity_and_the_committed_roots_hold_their_sources_numbers() {
    for name in ["acme_normal.wsp9", "acme_inverse.wsp9"] {
        let files: Vec<&str> = audit::forward_sources_of(name)
            .expect("a literal model")
            .iter()
            .map(|(n, _)| *n)
            .collect();
        assert!(
            files.contains(&"flame5.rs") && files.contains(&"fuel.rs"),
            "{files:?}"
        );
        assert!(!files.contains(&"flame.rs"));
        assert_eq!(audit::banned_hits_of(name), Vec::<String>::new());
    }
    let h = audit::forward_hash_of("acme_normal.wsp9");
    assert_eq!(h, audit::forward_hash_of("acme_inverse.wsp9"));
    assert_ne!(h, audit::forward_hash_of("acme_normal.wsp2"));
    let read = |path: &str| -> ModelRelease {
        serde_json::from_str(&config(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    for (family, dir) in FAMILIES {
        let root = read(&format!("wsp9/{dir}/baseline-clock/release.json"));
        let source = read(&format!("wsp2/{dir}/baseline-clock/release.json"));
        assert_eq!(root.tensors.len(), flame5::spec(family).tensors.len());
        for t in root
            .tensors
            .iter()
            .filter(|t| !fuel::is_fuel_tensor(&t.name))
        {
            let want = source.tensors.iter().find(|e| e.name == t.name).unwrap();
            assert_eq!(
                (&t.shape, &t.sha256),
                (&want.shape, &want.sha256),
                "{}",
                t.name
            );
        }
        let c = root.construction.as_ref().expect("a literal release");
        assert_eq!(c.forward_hash, audit::forward_hash_of(&root.model_name));
        assert!(c.checks.iter().all(|k| k.status == CheckStatus::Passed));
        let from_edits = release(
            &key(family, 9),
            &config(&format!("wsp9/{dir}/baseline-clock/edits.wsp")),
        );
        assert_eq!(
            from_edits
                .release
                .construction
                .as_ref()
                .unwrap()
                .weights_hash,
            c.weights_hash
        );
        // the root's table is the neutral one
        let neutral = new(family, NEUTRAL);
        for t in fuel::TENSORS {
            assert_eq!(from_edits.params[t].data, neutral.params[t].data, "{t}");
        }
    }
}

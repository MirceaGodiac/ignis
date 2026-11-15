//! The growth-clock laws with the fuel table (`flame6.rs`) on synthetic prefixes with test-only
//! weights: its spec is the `flame3.rs` spec without the three alkane molar masses plus the fuel
//! table; with the neutral table, and with the tables of section 3.4 of the design, every alkane
//! forecast is that of `flame3.rs` bit for bit; a unit coordinate evaluates one vertex, a zero weight
//! none; the ethylene and methane rows give the weighed network outputs, the shifts and the molar
//! mass a hand computation gives; the refusals of the fuel are made at the condition gate; the
//! committed roots hold their source's numbers.

mod common;
mod fuel_common;

use fuel_common::*;
use ignis_core::contracts::{CheckStatus, Family, ModelRelease, ModelSpec, PrefixBundle, Support};
use ignis_core::registry::LoadedRelease;
use ignis_wsp::nn::sigmoid;
use ignis_wsp::{audit, flame3, flame6, fuel, models};
use serde_json::json;

fn run3(r: &LoadedRelease, p: &PrefixBundle) -> flame3::Forecast {
    flame3::forecast(&flame3::spec(p.family), r, p, &json!({ "times_s": TIMES }))
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn try6(r: &LoadedRelease, p: &PrefixBundle) -> Result<flame6::Forecast, Support> {
    flame6::forecast(&flame6::spec(p.family), r, p, &json!({ "times_s": TIMES })).expect("runnable")
}

fn run6(r: &LoadedRelease, p: &PrefixBundle) -> flame6::Forecast {
    try6(r, p).unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn old(family: Family) -> LoadedRelease {
    release(&key(family, 3), &format!("{GROWTH}{BURNER}"))
}

fn new(family: Family, table: &str) -> LoadedRelease {
    release(&key(family, 10), &format!("{GROWTH}{BURNER}{table}"))
}

fn table_of(family: Family) -> &'static str {
    if family == Family::AcmeInverse {
        INVERSE_34
    } else {
        NORMAL_34
    }
}

fn vertices(f: &flame6::Forecast, law: &str) -> Vec<&'static str> {
    f.clocks
        .laws
        .iter()
        .find(|e| e.law == law)
        .unwrap_or_else(|| panic!("no law {law}"))
        .vertices
        .clone()
}

const LAWS: [&str; 5] = ["se.flux", "se.diff", "se.flash", "se.osc", "bt.rise"];

#[test]
fn the_spec_is_the_growth_spec_with_the_fuel_table_in_place_of_the_alkane_molar_masses() {
    for (family, dir) in FAMILIES {
        let spec = flame6::spec(family);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(&key(family, 10)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp10/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/acme/wsp10/{dir}/spec.json drifted from flame6::spec()"
        );
        let growth = flame3::spec(family);
        let kept: Vec<_> = growth
            .tensors
            .iter()
            .filter(|t| {
                !["phys.mw_ethane", "phys.mw_propane", "phys.mw_butane"].contains(&t.name.as_str())
            })
            .cloned()
            .collect();
        assert_eq!(kept.len(), growth.tensors.len() - 3);
        assert_eq!(&spec.tensors[..kept.len()], &kept[..]);
        assert_eq!(spec.tensors[kept.len()..].to_vec(), fuel::tensors());
        assert!(spec.tensor("phys.mw_n2").is_some() && spec.tensor("phys.mw_o2").is_some());
        assert_eq!(spec.domain, growth.domain);
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
                let (a, b) = (run3(&o, &p), run6(&n, &p));
                assert_eq!(a.table, b.table, "{family:?} {what}");
                assert_eq!(a.p_weak_at_cutoff.to_bits(), b.p_weak_at_cutoff.to_bits());
                assert_eq!(a.p_held_at_cutoff.to_bits(), b.p_held_at_cutoff.to_bits());
                assert_eq!(
                    a.burner.m_tot_kg_s.to_bits(),
                    b.burner.m_tot_kg_s.to_bits(),
                    "{what}"
                );
                assert_eq!(a.features, b.features);
                assert_eq!(a.clocks.burner_features, b.clocks.burner_features);
                check_partition(&b.table);
                let own = match what.split(' ').next().unwrap() {
                    "ethane" => "ethane",
                    "propane" => "propane",
                    _ => "butane",
                };
                for law in LAWS {
                    assert_eq!(vertices(&b, law), vec![own], "{what} {law}");
                }
            }
        }
    }
}

#[test]
fn the_ethylene_and_methane_rows_give_the_weighed_outputs_shifts_and_molar_mass_by_hand() {
    let (ie, ib) = (flame6::I_ETHANE, flame6::I_BUTANE);
    let (jb_e, jb_b) = (flame6::IB_ETHANE, flame6::IB_BUTANE);
    let e_v = [1.0, 0.0];
    // normal flames, ethylene: se (0.5, 0.5), ft (1, 0), s = -0.085, molar mass 28.054
    let n = new(Family::AcmeNormal, NORMAL_34);
    let p = prefix(Family::AcmeNormal, "ethylene");
    let f = run6(&n, &p);
    let c = f.features;
    close(
        (f.clocks.m_cr_g_m2_s).ln(),
        weighed_by_hand(&n, "se.flux", &c, ie, ib, [0.5, 0.5]) - -0.085,
        "ethylene ln m_cr",
    );
    close(
        f.clocks.d_mm2_s.ln(),
        weighed_by_hand(&n, "se.diff", &c, ie, ib, [0.5, 0.5]),
        "ethylene ln D",
    );
    close(
        f.clocks.p_weak,
        sigmoid(weighed_by_hand(&n, "se.flash", &c, ie, ib, [0.5, 0.5])),
        "ethylene p_weak",
    );
    close(
        f.clocks.osc_tau_s.ln(),
        weighed_by_hand(&n, "se.osc", &c, ie, ib, [0.5, 0.5]),
        "ethylene ln tau_osc",
    );
    assert_eq!(vertices(&f, "se.flux"), vec!["ethane", "butane"]);
    let bf = f.clocks.burner_features;
    close(
        f.clocks.burner_rise_k.ln(),
        net(&n, "bt.rise", &at(&bf, jb_e, jb_b, e_v)),
        "ethylene ln dT",
    );
    // Y_R of 30 % ethylene in nitrogen from the table's molar mass
    let (x, mw, mw_n2) = (0.30, 28.054, 28.013);
    let y = x * mw / (x * mw + (1.0 - x) * mw_n2);
    close(f.burner.m_tot_kg_s, 1.0e-6 / y, "ethylene m_tot");
    check_partition(&f.table);
    // normal flames, methane: se (1, 0), ft (1.9, -0.9), s = +0.068, molar mass 16.043
    let f = run6(&n, &prefix(Family::AcmeNormal, "methane"));
    let c = f.features;
    close(
        f.clocks.m_cr_g_m2_s.ln(),
        net(&n, "se.flux", &at(&c, ie, ib, e_v)) - 0.068,
        "methane ln m_cr",
    );
    let bf = f.clocks.burner_features;
    close(
        f.clocks.burner_rise_k.ln(),
        weighed_by_hand(&n, "bt.rise", &bf, jb_e, jb_b, [1.9, -0.9]),
        "methane ln dT",
    );
    let y = x * 16.043 / (x * 16.043 + (1.0 - x) * mw_n2);
    close(f.burner.m_tot_kg_s, 1.0e-6 / y, "methane m_tot");
    close(
        f.clocks.programmed[0].1,
        sigmoid(0.7 * c[1] - 1.2 * 1.9 + 0.9 * -0.9),
        "methane stop probability",
    );
    // inverse flames, ethylene: se (1, 0), ft (0.5, 0), dosc = +0.7; no oxygen-demand shift, the
    // burner-side molar mass is oxygen's
    let mut n = new(Family::AcmeInverse, INVERSE_34);
    let p = prefix(Family::AcmeInverse, "ethylene");
    let f = run6(&n, &p);
    let c = f.features;
    close(
        f.clocks.osc_tau_s.ln(),
        net(&n, "se.osc", &at(&c, ie, ib, e_v)) + 0.7,
        "inverse ethylene ln tau_osc",
    );
    let bf = f.clocks.burner_features;
    close(
        f.clocks.burner_rise_k.ln(),
        weighed_by_hand(&n, "bt.rise", &bf, jb_e, jb_b, [0.5, 0.0]),
        "inverse ethylene ln dT",
    );
    let propane = run6(&n, &prefix(Family::AcmeInverse, "propane"));
    assert_eq!(
        f.burner.m_tot_kg_s, propane.burner.m_tot_kg_s,
        "oxygen is the burner side"
    );
    set(&mut n, "fuel.s", &[0.0, 0.0, 0.0, 0.5, 0.0, 0.0]);
    let g = run6(&n, &p);
    assert_eq!(g.table, f.table, "the inverse laws do not read fuel.s");
    assert_eq!(g.clocks.flux_shift, 0.0);
}

#[test]
fn the_refusals_of_the_fuel_are_made_at_the_condition_gate() {
    for (family, _) in FAMILIES {
        let neutral = new(family, NEUTRAL);
        let design = new(family, table_of(family));
        let gate = |r: &LoadedRelease, p: PrefixBundle| try6(r, &p).map(|_| ());
        assert_refused(
            gate(&neutral, prefix(family, "ethylene")),
            "does not answer for ethylene",
            "neutral ethylene",
        );
        assert_refused(
            gate(&design, prefix(family, "propene")),
            "is not a fuel of the table",
            "propene",
        );
        assert_refused(
            gate(
                &neutral,
                with_indicators(&prefix(family, "butane"), 1.0, 0.0, true),
            ),
            "disagree",
            "butane named ethane",
        );
        assert_refused(
            gate(
                &neutral,
                with_indicators(&prefix(family, "butane"), 1.0, 1.0, false),
            ),
            "name no fuel",
            "two indicators",
        );
    }
    let design = new(Family::AcmeNormal, NORMAL_34);
    let p = with(&prefix(Family::AcmeNormal, "methane"), "tad", 1850.0);
    assert_refused(
        try6(&design, &p).map(|_| ()),
        "is below the bound",
        "cool methane",
    );
}

#[test]
fn the_forward_has_its_own_identity_and_the_committed_roots_hold_their_sources_numbers() {
    for name in ["acme_normal.wsp10", "acme_inverse.wsp10"] {
        let files: Vec<&str> = audit::forward_sources_of(name)
            .unwrap()
            .iter()
            .map(|(n, _)| *n)
            .collect();
        assert!(
            files.contains(&"flame6.rs") && files.contains(&"fuel.rs"),
            "{files:?}"
        );
        assert!(!files.contains(&"flame3.rs"));
        assert_eq!(audit::banned_hits_of(name), Vec::<String>::new());
    }
    assert_ne!(
        audit::forward_hash_of("acme_normal.wsp10"),
        audit::forward_hash_of("acme_normal.wsp3")
    );
    let read = |path: &str| -> ModelRelease { serde_json::from_str(&config(path)).unwrap() };
    for (family, dir) in FAMILIES {
        let root = read(&format!("wsp10/{dir}/baseline-theory/release.json"));
        let source = read(&format!("wsp3/{dir}/baseline-theory/release.json"));
        assert_eq!(root.tensors.len(), flame6::spec(family).tensors.len());
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
        let c = root.construction.as_ref().unwrap();
        assert_eq!(c.forward_hash, audit::forward_hash_of(&root.model_name));
        assert!(c.checks.iter().all(|k| k.status == CheckStatus::Passed));
        let from_edits = release(
            &key(family, 10),
            &config(&format!("wsp10/{dir}/baseline-theory/edits.wsp")),
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
        // the table's alkane molar masses are the constants the growth root stored
        let stored = release(
            &key(family, 3),
            &config(&format!("wsp3/{dir}/baseline-theory/edits.wsp")),
        );
        let mw = &from_edits.params["fuel.mw_g_mol"].data;
        for (k, name) in ["phys.mw_ethane", "phys.mw_propane", "phys.mw_butane"]
            .iter()
            .enumerate()
        {
            assert_eq!(
                mw[k].to_bits(),
                stored.params[*name].data[0].to_bits(),
                "{name}"
            );
            assert_eq!(mw[k + 3].to_bits(), mw[k].to_bits());
        }
    }
}

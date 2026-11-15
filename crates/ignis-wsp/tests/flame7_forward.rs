//! The hinge-law clocks with the burner-heating law and the fuel table (`flame7.rs`) on synthetic
//! prefixes with test-only weights: its spec is the `flame4.rs` spec without the three alkane molar
//! masses plus the fuel table; with the neutral table, and with the tables of section 3.4 of the
//! design, every alkane forecast is that of `flame4.rs` bit for bit; the hinge part reads the
//! hinge-law lines of the design (the oxygen-demand shift in the radiative clock) and the burner part
//! the growth-law burner lines (the table's molar mass, the rise network weighed by the burner
//! coordinates), as a hand computation shows; the fuel's refusals are made at the condition gate; the
//! committed roots hold their source's numbers.

mod common;
mod fuel_common;

use fuel_common::*;
use ignis_core::contracts::{CheckStatus, Family, ModelRelease, ModelSpec, PrefixBundle, Support};
use ignis_core::registry::LoadedRelease;
use ignis_wsp::nn::sigmoid;
use ignis_wsp::{audit, flame4, flame7, fuel, models};
use serde_json::json;

fn run4(r: &LoadedRelease, p: &PrefixBundle) -> flame4::Forecast {
    flame4::forecast(&flame4::spec(p.family), r, p, &json!({ "times_s": TIMES }))
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn try7(r: &LoadedRelease, p: &PrefixBundle) -> Result<flame7::Forecast, Support> {
    flame7::forecast(&flame7::spec(p.family), r, p, &json!({ "times_s": TIMES })).expect("runnable")
}

fn run7(r: &LoadedRelease, p: &PrefixBundle) -> flame7::Forecast {
    try7(r, p).unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn old(family: Family) -> LoadedRelease {
    release(&key(family, 8), &format!("{HINGE}{BURNER}"))
}

fn new(family: Family, table: &str) -> LoadedRelease {
    release(&key(family, 11), &format!("{HINGE}{BURNER}{table}"))
}

fn table_of(family: Family) -> &'static str {
    if family == Family::AcmeInverse {
        INVERSE_34
    } else {
        NORMAL_34
    }
}

fn vertices(f: &flame7::Forecast, law: &str) -> Vec<&'static str> {
    f.clocks
        .laws
        .iter()
        .find(|e| e.law == law)
        .unwrap_or_else(|| panic!("no law {law}"))
        .vertices
        .clone()
}

#[test]
fn the_spec_is_the_flame4_spec_with_the_fuel_table_in_place_of_the_alkane_molar_masses() {
    for (family, dir) in FAMILIES {
        let spec = flame7::spec(family);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(&key(family, 11)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp11/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/acme/wsp11/{dir}/spec.json drifted from flame7::spec()"
        );
        let parent = flame4::spec(family);
        let kept: Vec<_> = parent
            .tensors
            .iter()
            .filter(|t| !fuel::is_alkane_molar_mass(&t.name))
            .cloned()
            .collect();
        assert_eq!(kept.len(), parent.tensors.len() - 3);
        assert_eq!(&spec.tensors[..kept.len()], &kept[..]);
        assert_eq!(spec.tensors[kept.len()..].to_vec(), fuel::tensors());
        assert!(!flame7::is_burner_tensor("phys.mw_ethane"));
        assert!(flame7::is_burner_tensor("bt.rise.b0") && flame7::is_burner_tensor("phys.mw_n2"));
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
                let (a, b) = (run4(&o, &p), run7(&n, &p));
                assert_eq!(a.table, b.table, "{family:?} {what}");
                assert_eq!(a.p_weak_at_cutoff.to_bits(), b.p_weak_at_cutoff.to_bits());
                assert_eq!(a.burner.m_tot_kg_s.to_bits(), b.burner.m_tot_kg_s.to_bits());
                assert_eq!(a.clocks.burner_features, b.clocks.burner_features);
                check_partition(&b.table);
                let own = match what.split(' ').next().unwrap() {
                    "ethane" => "ethane",
                    "propane" => "propane",
                    _ => "butane",
                };
                for law in ["se.tau", "se.flash", "bt.rise"] {
                    assert_eq!(vertices(&b, law), vec![own], "{what} {law}");
                }
            }
        }
    }
}

#[test]
fn the_hinge_part_reads_the_hinge_lines_and_the_burner_part_the_burner_lines() {
    let (ie, ib) = (flame7::I_ETHANE, flame7::I_BUTANE);
    let (jb_e, jb_b) = (flame7::IB_ETHANE, flame7::IB_BUTANE);
    let e_v = [1.0, 0.0];
    let n = new(Family::AcmeNormal, NORMAL_34);
    // methane: hinge part se (1, 0) with c_logm + 0.068; burner part ft (1.9, -0.9), molar mass 16.043
    let f = run7(&n, &prefix(Family::AcmeNormal, "methane"));
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
    let bf = f.clocks.burner_features;
    close(
        f.clocks.burner_rise_k.ln(),
        weighed_by_hand(&n, "bt.rise", &bf, jb_e, jb_b, [1.9, -0.9]),
        "methane ln dT",
    );
    let (x, mw_n2) = (0.30, 28.013);
    let y = x * 16.043 / (x * 16.043 + (1.0 - x) * mw_n2);
    close(f.burner.m_tot_kg_s, 1.0e-6 / y, "methane m_tot");
    check_partition(&f.table);
    // ethylene: hinge part se (0.5, 0.5) with c_logm - 0.085
    let f = run7(&n, &prefix(Family::AcmeNormal, "ethylene"));
    let c = f.features;
    let mut c_tau = c.to_vec();
    c_tau[1] += -0.085;
    close(
        f.clocks.radiative_tau_s.ln(),
        weighed_by_hand(&n, "se.tau", &c_tau, ie, ib, [0.5, 0.5]),
        "ethylene ln tau_R",
    );
    assert_eq!(vertices(&f, "bt.rise"), vec!["ethane"]);
    // this forward has no quench clock: fuel.dosc changes nothing
    let mut m = new(Family::AcmeInverse, INVERSE_34);
    let p = prefix(Family::AcmeInverse, "ethylene");
    let f = run7(&m, &p);
    set(&mut m, "fuel.dosc", &[0.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
    assert_eq!(run7(&m, &p).table, f.table);
    let bf = f.clocks.burner_features;
    close(
        f.clocks.burner_rise_k.ln(),
        weighed_by_hand(&m, "bt.rise", &bf, jb_e, jb_b, [0.5, 0.0]),
        "inverse ethylene ln dT",
    );
}

#[test]
fn the_refusals_of_the_fuel_are_made_at_the_condition_gate() {
    for (family, _) in FAMILIES {
        let neutral = new(family, NEUTRAL);
        let gate = |r: &LoadedRelease, p: PrefixBundle| try7(r, &p).map(|_| ());
        assert_refused(
            gate(&neutral, prefix(family, "methane")),
            "does not answer for methane",
            "neutral methane",
        );
        assert_refused(
            gate(&neutral, prefix(family, "acetylene")),
            "is not a fuel of the table",
            "acetylene",
        );
        assert_refused(
            gate(
                &neutral,
                with_indicators(&prefix(family, "propane"), 0.0, 1.0, true),
            ),
            "disagree",
            "propane named butane",
        );
    }
    let design = new(Family::AcmeNormal, NORMAL_34);
    let p = with(&prefix(Family::AcmeNormal, "ethylene"), "tad", 1700.0);
    assert_refused(
        try7(&design, &p).map(|_| ()),
        "is below the bound",
        "cool ethylene",
    );
}

#[test]
fn the_forward_has_its_own_identity_and_the_committed_roots_hold_their_sources_numbers() {
    for name in ["acme_normal.wsp11", "acme_inverse.wsp11"] {
        let files: Vec<&str> = audit::forward_sources_of(name)
            .unwrap()
            .iter()
            .map(|(n, _)| *n)
            .collect();
        assert!(
            files.contains(&"flame7.rs") && files.contains(&"fuel.rs"),
            "{files:?}"
        );
        assert!(!files.contains(&"flame4.rs"));
        assert_eq!(audit::banned_hits_of(name), Vec::<String>::new());
    }
    let read = |path: &str| -> ModelRelease { serde_json::from_str(&config(path)).unwrap() };
    for (family, dir) in FAMILIES {
        let root = read(&format!("wsp11/{dir}/baseline-clock-theory/release.json"));
        let source = read(&format!("wsp8/{dir}/baseline-clock-theory/release.json"));
        assert_eq!(root.tensors.len(), flame7::spec(family).tensors.len());
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
            &key(family, 11),
            &config(&format!("wsp11/{dir}/baseline-clock-theory/edits.wsp")),
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
    }
}

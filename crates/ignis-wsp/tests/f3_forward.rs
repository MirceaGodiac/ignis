//! The forwards of the second fuel table (`fuel2.rs`, `flame8.rs`, `flame9.rs`, `flame10.rs`,
//! `committee4.rs`; `acme_normal.wsp13` to `wsp16`, design `docs/wsp/research/phys_sflame_f3.md`) on
//! synthetic prefixes with test-only weights:
//!
//! * with the new columns inert, the nitrogen row the identity, the heating terms off and no limit
//!   input, every member and the committee return the tables of the `flame5`/`flame6`/`flame7` and
//!   `committee3` forwards bit for bit; so does a limit input equal to the member's typed setting;
//! * a lower limit moves every weighted limit by the difference, and the time to it is the heating
//!   law's;
//! * a diluent that is not a row, or not admitted, is refused and named;
//! * two oxygen candidates give (1 - w) P_first + w P_second; a refusal at either refuses; one or two
//!   of the three keys is a refusal;
//! * a blend's columns are the oxygen-demand shares of its rows, its molar mass mole-weighted;
//! * the forward identity covers the new files and the committed specs have not drifted.
//!
//! Every number typed here is test data on synthetic prefixes, not a candidate.

mod common;
mod fuel_common;

use fuel_common::*;
use ignis_core::contracts::{Family, ModelSpec, PrefixBundle, RawValue, Support};
use ignis_core::registry::LoadedRelease;
use ignis_wsp::{
    audit, cli, committee3, committee4, flame10, flame5, flame6, flame7, flame8, flame9, fuel2,
    models,
};
use serde_json::json;

const N: Family = Family::AcmeNormal;

/// The five new columns, inert (sigma: property values; read only for blends).
const NEW_INERT: &str = "
fuel.sigma[0:6] = [3.725, 3.628, 3.579, 3.422, 3.989, 7.936]
fuel.dtad_k[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
fuel.k_rel[0:6] = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
fuel.zst0[0:6] = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
fuel.gz[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
";

/// The diluent table with nitrogen as the identity and helium refused.
const DIL_NEUTRAL: &str = "
dil.admit[0:2] = [1.0, 0.0]
dil.mw_g_mol[0:2] = [28.013, 28.013]
dil.rho_d[0:2] = [1.0, 1.0]
dil.d_far[0:2] = [1.0, 1.0]
dil.le_o2[0:2] = [1.0, 1.0]
dil.dflux[0:2] = [0.0, 0.0]
dil.dlogm[0:2] = [0.0, 0.0]
dil.dlnr[0:2] = [0.0, 0.0]
dil.k_rel[0:2] = [1.0, 1.0]
";

/// The helium row of Part A of the design (copied), nitrogen the identity.
const DIL_HELIUM: &str = "
dil.admit[0:2] = [1.0, 1.0]
dil.mw_g_mol[0:2] = [28.013, 4.0026]
dil.rho_d[0:2] = [1.0, 1.15]
dil.d_far[0:2] = [1.0, 3.5]
dil.le_o2[0:2] = [1.0, 1.37]
dil.dflux[0:2] = [0.0, 0.3]
dil.dlogm[0:2] = [0.0, -1.9]
dil.dlnr[0:2] = [0.0, -1.25]
dil.k_rel[0:2] = [1.0, 5.8]
";

/// The heating terms off and the typed setting of an early-era member.
const HEAT_OFF: &str = "
bt.kappa_exp = 0.0
bt.tflame_ref_k = 0.0
bt.limit_script_c = 500.0
";

fn table() -> String {
    format!("{NORMAL_34}{NEW_INERT}{DIL_NEUTRAL}")
}

fn m5() -> LoadedRelease {
    release(&key(N, 9), &format!("{HINGE}{LIMIT_CLOCK}{NORMAL_34}"))
}
fn m6() -> LoadedRelease {
    release(&key(N, 10), &format!("{GROWTH}{BURNER}{NORMAL_34}"))
}
fn m7() -> LoadedRelease {
    release(&key(N, 11), &format!("{HINGE}{BURNER}{NORMAL_34}"))
}
fn m8(extra: &str) -> LoadedRelease {
    release(
        &key(N, 14),
        &merged(&format!("{HINGE}{LIMIT_CLOCK}{}", table()), extra),
    )
}
fn m9(extra: &str) -> LoadedRelease {
    release(
        &key(N, 15),
        &merged(&format!("{GROWTH}{BURNER}{}{HEAT_OFF}", table()), extra),
    )
}
fn m10(extra: &str) -> LoadedRelease {
    release(
        &key(N, 16),
        &merged(&format!("{HINGE}{BURNER}{}{HEAT_OFF}", table()), extra),
    )
}

fn settings() -> serde_json::Value {
    json!({ "times_s": TIMES })
}

fn text_cond(p: &PrefixBundle, key: &str, v: &str) -> PrefixBundle {
    let mut p = p.clone();
    let mut m = p.known_conditions["pressure"].clone();
    m.raw_value = Some(RawValue::Text(v.into()));
    m.value_si = None;
    p.known_conditions.insert(key.into(), m);
    rehash(&mut p);
    p
}

fn num_cond(p: &PrefixBundle, key: &str, v: f64) -> PrefixBundle {
    let mut p = p.clone();
    let mut m = p.known_conditions["pressure"].clone();
    m.raw_value = Some(RawValue::Number(v));
    m.value_si = Some(v);
    p.known_conditions.insert(key.into(), m);
    rehash(&mut p);
    p
}

/// The prefixes on which the carries are compared: the alkanes, ethylene and methane at two states.
fn prefixes() -> Vec<(String, PrefixBundle)> {
    let mut v = alkane_prefixes(N);
    for f in ["ethylene", "methane"] {
        v.push((format!("{f} mid"), prefix(N, f)));
        v.push((format!("{f} other"), other_prefix(N, f)));
    }
    v
}

type Run<T> = Result<T, Support>;

fn r5(r: &LoadedRelease, p: &PrefixBundle) -> Run<flame5::Forecast> {
    flame5::forecast(&flame5::spec(N), r, p, &settings()).expect("runnable")
}
fn r6(r: &LoadedRelease, p: &PrefixBundle) -> Run<flame6::Forecast> {
    flame6::forecast(&flame6::spec(N), r, p, &settings()).expect("runnable")
}
fn r7(r: &LoadedRelease, p: &PrefixBundle) -> Run<flame7::Forecast> {
    flame7::forecast(&flame7::spec(N), r, p, &settings()).expect("runnable")
}
fn r8(r: &LoadedRelease, p: &PrefixBundle) -> Run<flame8::Forecast> {
    flame8::forecast(&flame8::spec(N), r, p, &settings()).expect("runnable")
}
fn r9(r: &LoadedRelease, p: &PrefixBundle) -> Run<flame9::Forecast> {
    flame9::forecast(&flame9::spec(N), r, p, &settings()).expect("runnable")
}
fn r10(r: &LoadedRelease, p: &PrefixBundle) -> Run<flame10::Forecast> {
    flame10::forecast(&flame10::spec(N), r, p, &settings()).expect("runnable")
}

fn table_of<T>(r: Run<T>, f: impl Fn(&T) -> [Vec<f64>; 4], what: &str) -> [Vec<f64>; 4] {
    match r {
        Ok(x) => f(&x),
        Err(s) => panic!("{what}: refused {:?}", s.reasons),
    }
}

#[test]
fn the_specs_are_their_parents_with_the_two_tables_and_the_committed_spec_has_not_drifted() {
    let names = |s: &ModelSpec| s.tensors.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
    let s8 = flame8::spec(N);
    for t in fuel2::FUEL_TENSORS
        .iter()
        .chain(fuel2::DIL_TENSORS.iter())
        .chain([fuel2::T_STREAM].iter())
    {
        assert!(s8.tensor(t).is_some(), "flame8 lacks {t}");
    }
    for s in [flame9::spec(N), flame10::spec(N)] {
        let n = names(&s);
        for t in fuel2::MEMBER_SCALARS {
            assert!(n.iter().any(|x| x == t), "{} lacks {t}", s.spec_id);
        }
        for t in [
            "phys.mw_n2",
            "phys.mw_ethane",
            "phys.mw_propane",
            "phys.mw_butane",
        ] {
            assert!(!n.iter().any(|x| x == t), "{} still holds {t}", s.spec_id);
        }
    }
    let b = fuel2::tensors();
    let s = b.iter().find(|t| t.name == "fuel.s").unwrap();
    assert_eq!(
        (
            s.bounds.as_ref().unwrap().min.get(),
            s.bounds.as_ref().unwrap().max.get()
        ),
        (-3.0, 3.0)
    );
    let spec = committee4::spec(N);
    spec.validate().expect("valid");
    let on_disk: ModelSpec =
        serde_json::from_str(&config("wsp13/normal/spec.json")).expect("spec.json");
    assert_eq!(
        on_disk.spec_hash(),
        spec.spec_hash(),
        "configs/models/acme/wsp13/normal/spec.json drifted from committee4::spec()"
    );
    assert!(committee4::OWN_TENSORS.contains(&"mix.o2_b_w"));
    assert_eq!(
        models::by_key("acme_normal13").unwrap().spec().spec_hash(),
        spec.spec_hash()
    );
}

#[test]
fn with_inert_columns_and_the_heating_terms_off_every_member_is_its_parent_bit_for_bit() {
    let (a5, a6, a7) = (m5(), m6(), m7());
    let (a8, a9, a10) = (m8(""), m9(""), m10(""));
    // a conductivity power changes nothing on a stream of unit relative conductivity
    let a9k = m9("bt.kappa_exp = 0.45\n");
    let a10k = m10("bt.kappa_exp = 0.45\n");
    for (what, p) in prefixes() {
        // the limit input equal to the member's typed setting is the same as no input
        let same = num_cond(&p, "burner_limit_c", 500.0);
        for q in [&p, &same] {
            let a = table_of(r5(&a5, q), |f| f.table.clone(), &what);
            assert_eq!(
                a,
                table_of(r8(&a8, q), |f| f.table.clone(), &what),
                "flame8 {what}"
            );
            let a = table_of(r6(&a6, q), |f| f.table.clone(), &what);
            assert_eq!(
                a,
                table_of(r9(&a9, q), |f| f.table.clone(), &what),
                "flame9 {what}"
            );
            assert_eq!(
                a,
                table_of(r9(&a9k, q), |f| f.table.clone(), &what),
                "flame9 kappa {what}"
            );
            let a = table_of(r7(&a7, q), |f| f.table.clone(), &what);
            assert_eq!(
                a,
                table_of(r10(&a10, q), |f| f.table.clone(), &what),
                "flame10 {what}"
            );
            assert_eq!(
                a,
                table_of(r10(&a10k, q), |f| f.table.clone(), &what),
                "flame10 kappa {what}"
            );
        }
        // an explicit nitrogen is the absent diluent
        let n2 = text_cond(&p, "diluent_species", "Nitrogen");
        assert_eq!(
            table_of(r9(&a9, &p), |f| f.table.clone(), &what),
            table_of(r9(&a9, &n2), |f| f.table.clone(), &what)
        );
    }
}

/// The edits of a committee: `own`, then each slot's member statement by statement.
fn committee_edits(specs: [&ModelSpec; 3], m: [&LoadedRelease; 3], own: &str) -> String {
    let mut src = own.to_string();
    for (i, slot) in committee3::SLOT_PREFIX.iter().enumerate() {
        let k = if i < 2 {
            0
        } else if i < 6 {
            1
        } else {
            2
        };
        for line in cli::edits_of(specs[k], m[k]).lines() {
            src.push_str(slot);
            src.push_str(line);
            src.push('\n');
        }
    }
    src
}

const OWN: &str = "mix.w_early[0:10] = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]
mix.w_late[0:10] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
mix.ws_early[0:10] = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
mix.ws_late[0:10] = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
era.last_early_day = 2021235.0
era.first_late_day = 2021256.0
era.p_late_between = 0.5
era.p_late_unknown = 0.15
script.offset_s[0:3] = [-1.0, 0.0, 1.0]
script.offset_w[0:3] = [0.25, 0.5, 0.25]
";

fn wsp12() -> LoadedRelease {
    let s = [&flame5::spec(N), &flame6::spec(N), &flame7::spec(N)];
    release(&key(N, 12), &committee_edits(s, [&m5(), &m6(), &m7()], OWN))
}

fn wsp13(extra_member: &str, w: f64) -> LoadedRelease {
    let s = [&flame8::spec(N), &flame9::spec(N), &flame10::spec(N)];
    let own = format!("{OWN}mix.o2_b_w = {w:?}\n");
    let a = m8(extra_member);
    let b = m9(extra_member);
    let c = m10(extra_member);
    release(&key(N, 13), &committee_edits(s, [&a, &b, &c], &own))
}

fn run12(r: &LoadedRelease, p: &PrefixBundle) -> committee3::Forecast {
    committee3::forecast(&committee3::spec(N), r, p, &settings())
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused {:?}", s.reasons))
}

fn try13(r: &LoadedRelease, p: &PrefixBundle) -> Run<committee4::Forecast> {
    committee4::forecast(&committee4::spec(N), r, p, &settings()).expect("runnable")
}

#[test]
fn the_carry_committee_is_the_third_committee_bit_for_bit() {
    let (old, new) = (wsp12(), wsp13("", 0.5));
    for (what, p) in prefixes() {
        for q in [
            p.clone(),
            num_cond(&p, "programmed_stop", 40.0),
            num_cond(&p, "burner_limit_c", 500.0),
        ] {
            let a = run12(&old, &q);
            let b = try13(&new, &q).unwrap_or_else(|s| panic!("{what}: {:?}", s.reasons));
            assert_eq!(a.table, b.table, "{what}");
            assert!(b.o2_candidates.is_empty());
        }
    }
}

#[test]
fn a_lower_limit_setting_moves_every_weighted_limit_and_the_time_is_the_heating_laws() {
    let r = m9("");
    let p = prefix(N, "propane");
    let base = r9(&r, &p).expect("admitted");
    let low = r9(&r, &num_cond(&p, "burner_limit_c", 400.0)).expect("admitted");
    assert_ne!(base.table, low.table);
    let (rise, t0, tau) = (
        low.clocks.burner_rise_k,
        low.clocks.burner_t0_c,
        low.clocks.burner_heat_tau_s,
    );
    let want = [300.0 - 100.0, 380.0 - 100.0, 1400.0 - 100.0];
    for ((lim, _w, t), want) in low.clocks.burner_limits.iter().zip(want) {
        assert_eq!(*lim, want);
        let frac = (want - t0) / rise;
        let hand = (frac < 1.0).then(|| -tau * (-frac).ln_1p());
        match (t, hand) {
            (Some(a), Some(b)) => close(*a, b, "time to the shifted limit"),
            (None, None) => {}
            other => panic!("limit {want}: {other:?}"),
        }
    }
    // the same on the hinge member with the heating law
    let r = m10("");
    let low = r10(&r, &num_cond(&p, "burner_limit_c", 400.0)).expect("admitted");
    assert_eq!(low.clocks.burner_limits[0].0, 200.0);
    // a setting that puts a weighted limit at or below the burner's start is refused
    let s = r9(&m9(""), &num_cond(&p, "burner_limit_c", 210.0)).map(|_| ());
    match s {
        Ok(()) => panic!("admitted a limit below the burner's start"),
        Err(s) => assert!(
            s.reasons.iter().any(|x| x.contains("at or below bt.t0_c")),
            "{:?}",
            s.reasons
        ),
    }
}

#[test]
fn a_diluent_not_in_the_table_or_not_admitted_is_refused_and_named() {
    let p = prefix(N, "propane");
    for (text, needle) in [
        ("argon", "\"argon\" is not a diluent of the table"),
        (
            "nitrogen+helium",
            "\"nitrogen+helium\" is not a diluent of the table",
        ),
        (
            "helium in nitrogen",
            "\"helium in nitrogen\" is not a diluent of the table",
        ),
        ("helium", "does not answer for diluent helium"),
    ] {
        let q = text_cond(&p, "diluent_species", text);
        assert_refused(r8(&m8(""), &q).map(|_| ()), needle, text);
        assert_refused(r9(&m9(""), &q).map(|_| ()), needle, text);
        assert_refused(r10(&m10(""), &q).map(|_| ()), needle, text);
    }
    // with the helium row of the design the flame temperature is the Lewis-number formula
    let r = m9(DIL_HELIUM);
    let q = text_cond(&p, "diluent_species", "helium");
    let f = r9(&r, &q).expect("admitted");
    let (tad, zst) = (2300.0, 0.25);
    let t1 = 298.15 + (tad - 298.15) * (1.0 - (1.0_f64 - zst).powf(1.0 / 1.37)) / zst;
    close(f.clocks.inputs.t1_k, t1, "T_1 in helium");
    assert_eq!(f.clocks.inputs.diluent.name, "helium");
}

#[test]
fn two_oxygen_candidates_are_mixed_with_their_weights_and_a_refusal_at_either_refuses() {
    for w in [0.5, 0.25] {
        let r = wsp13("", w);
        let p = prefix(N, "ethylene");
        let second = num_cond(
            &num_cond(&num_cond(&p, "x_o2_role", 0.21), "tad", 2150.0),
            "zst",
            0.22,
        );
        let both = num_cond(
            &num_cond(&num_cond(&p, "x_o2_role_b", 0.21), "tad_b", 2150.0),
            "zst_b",
            0.22,
        );
        let a = try13(&r, &p).expect("first");
        let b = try13(&r, &second).expect("second");
        let m = try13(&r, &both).expect("mixture");
        for k in 0..4 {
            let want: Vec<f64> = a.table[k]
                .iter()
                .zip(&b.table[k])
                .map(|(x, y)| (1.0 - w) * x + w * y)
                .collect();
            assert_eq!(m.table[k], want, "row {k}, w {w}");
        }
        assert_eq!(m.o2_candidates, vec![(1.0 - w, 0.30), (w, 0.21)]);
        check_partition(&m.table);
    }
    // a refusal of the second candidate (methane below its bound there) refuses the forecast
    let r = wsp13("", 0.5);
    let p = prefix(N, "methane");
    let both = num_cond(
        &num_cond(&num_cond(&p, "x_o2_role_b", 0.21), "tad_b", 1850.0),
        "zst_b",
        0.22,
    );
    match try13(&r, &both) {
        Ok(_) => panic!("admitted"),
        Err(s) => assert!(
            s.reasons
                .iter()
                .any(|x| x.starts_with("oxygen candidate 2:")),
            "{:?}",
            s.reasons
        ),
    }
    // one or two of the three keys
    let partial = num_cond(&num_cond(&p, "x_o2_role_b", 0.21), "tad_b", 2150.0);
    assert_refused(
        try13(&r, &partial).map(|_| ()),
        "needs x_o2_role_b, tad_b and zst_b",
        "partial",
    );
}

#[test]
fn a_blend_takes_the_oxygen_demand_shares_of_its_rows() {
    // the design's rows with hydrogen admitted (test data)
    let admit = "fuel.admit[0:6] = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0]\n\
                 fuel.dtad_k[0:6] = [0.0, 0.0, 0.0, 0.0, -350.0, 600.0]\n\
                 fuel.k_rel[0:6] = [1.0, 1.0, 1.0, 1.0, 1.0, 7.0]\n";
    let r = m9(admit);
    let x = 0.444;
    let p = num_cond(&prefix(N, "hydrogen+methane"), "fuel_blend_x", x);
    let f = r9(&r, &p).expect("admitted");
    let fuel = &f.clocks.inputs.fuel;
    let (wh, wm, sh, sm) = (2.016, 16.043, 7.936, 3.989);
    let y_a = x * wh / (x * wh + (1.0 - x) * wm);
    let w_a = y_a * sh / (y_a * sh + (1.0 - y_a) * sm);
    let w_b = 1.0 - w_a;
    let (sh_, sm_) = (0.757, 0.068); // fuel.s of NORMAL_34: hydrogen, methane
    close(fuel.shares.unwrap()[0], w_a, "share of hydrogen");
    close(fuel.mw_g_mol, x * wh + (1.0 - x) * wm, "molar mass");
    close(fuel.s, w_a * sh_ + w_b * sm_, "fuel.s");
    close(fuel.dtad_k, w_a * 600.0 + w_b * -350.0, "dtad");
    close(fuel.tad_min_k, w_a * 0.0 + w_b * 1900.0, "bound");
    close(fuel.ft_coord[0], w_a * 0.0 + w_b * 1.9, "ft coordinate e");
    close(fuel.ft_coord[1], w_a * 0.0 + w_b * -0.9, "ft coordinate b");
    assert_eq!(fuel.k_rel, vec![(x, 7.0), (1.0 - x, 1.0)]);
    // the stream conductivity of a blend in nitrogen: x_fuel split by the blend's fractions
    let xf = 0.30;
    close(
        fuel2::kappa(fuel, &f.clocks.inputs.diluent, xf),
        xf * x * 6.0,
        "kappa",
    );
    // refusals: no fraction, a species that is not a row, a fraction with a single fuel, a row not admitted
    assert_refused(
        r9(&r, &prefix(N, "hydrogen+methane")).map(|_| ()),
        "has no fuel_blend_x",
        "no fraction",
    );
    assert_refused(
        r9(
            &r,
            &num_cond(&prefix(N, "hydrogen+argon"), "fuel_blend_x", 0.5),
        )
        .map(|_| ()),
        "\"argon\" is not a fuel of the table",
        "argon",
    );
    assert_refused(
        r9(&r, &num_cond(&prefix(N, "methane"), "fuel_blend_x", 0.5)).map(|_| ()),
        "is given for a single fuel",
        "single",
    );
    assert_refused(
        r9(&m9(""), &p).map(|_| ()),
        "does not answer for fuel hydrogen",
        "hydrogen refused",
    );
}

#[test]
fn the_forward_identity_covers_the_new_files() {
    let want: [(&str, &[&str]); 4] = [
        ("acme_normal.wsp14", &["fuel.rs", "fuel2.rs", "flame8.rs"]),
        ("acme_normal.wsp15", &["fuel.rs", "fuel2.rs", "flame9.rs"]),
        ("acme_normal.wsp16", &["fuel.rs", "fuel2.rs", "flame10.rs"]),
        (
            "acme_normal.wsp13",
            &[
                "fuel.rs",
                "fuel2.rs",
                "flame8.rs",
                "flame9.rs",
                "flame10.rs",
                "committee4.rs",
            ],
        ),
    ];
    let mut hashes = Vec::new();
    for (name, files) in want {
        let have: Vec<&str> = audit::forward_sources_of(name)
            .unwrap_or_else(|| panic!("{name}"))
            .iter()
            .map(|(n, _)| *n)
            .collect();
        for f in files {
            assert!(have.contains(f), "{name}: {f} missing from {have:?}");
        }
        for old in ["flame5.rs", "flame6.rs", "flame7.rs", "committee3.rs"] {
            assert!(!have.contains(&old), "{name} names {old}");
        }
        assert_eq!(audit::banned_hits_of(name), Vec::<String>::new(), "{name}");
        hashes.push(audit::forward_hash_of(name));
    }
    hashes.push(audit::forward_hash_of("acme_normal.wsp12"));
    let n = hashes.len();
    hashes.sort_by_key(|h| h.to_string());
    hashes.dedup();
    assert_eq!(hashes.len(), n, "forward hashes are distinct");
}

#[test]
fn the_heating_law_takes_the_flame_temperature_factor_and_the_stream_conductivity_power() {
    let p = prefix(N, "propane");
    let base = r9(&m9(""), &p).expect("admitted").clocks.burner_rise_k;
    // the temperature factor (T_1 - T_b) / (bt.tflame_ref_k - T_b), T_b = bt.t0_c + 273.15
    let f = r9(&m9("bt.tflame_ref_k = 2650.0\n"), &p).expect("admitted");
    let t_b = 20.0 + 273.15;
    close(
        f.clocks.burner_rise_k,
        base * (2300.0 - t_b) / (2650.0 - t_b),
        "temperature factor",
    );
    // a factor at T_1 = bt.tflame_ref_k is 1
    let f = r9(&m9("bt.tflame_ref_k = 2300.0\n"), &p).expect("admitted");
    close(f.clocks.burner_rise_k, base, "factor 1 at the reference");
    // T_1 at or below the burner's temperature: refused
    assert_refused(
        r9(&m9("bt.tflame_ref_k = 2650.0\n"), &with(&p, "tad", 280.0)).map(|_| ()),
        "the heating law has no clock",
        "cold flame",
    );
    // the conductivity power on a helium stream: (1 + (1 - x_fuel)(k_He - 1))^kappa_exp
    let q = text_cond(&p, "diluent_species", "helium");
    let he = format!("{DIL_HELIUM}");
    let a = r9(&m9(&he), &q).expect("admitted");
    let b = r9(&m9(&format!("{he}bt.kappa_exp = 0.45\n")), &q).expect("admitted");
    let kappa = (1.0 - 0.30) * (5.8 - 1.0);
    close(
        b.clocks.burner_rise_k,
        a.clocks.burner_rise_k * (1.0_f64 + kappa).powf(0.45),
        "conductivity power",
    );
}

//! The third committee of the ACME burner flames (`committee3.rs`): its spec is the ten member specs
//! of the fuel-table forwards under ten slot prefixes plus the committee's own tensors; with members
//! that carry the neutral table it answers as the second committee with the same numbers, bit for
//! bit, on every path (eras, the planned stop at three offsets); each member reads the fuel by its own
//! table and a member's refusal refuses the forecast; one member on one node answers with its own
//! table; the forward identity covers the fuel table and the three member sources; the committed
//! roots hold their source releases' numbers.
//!
//! Every number typed here is test data on synthetic prefixes, not a candidate.

mod common;
mod fuel_common;

use fuel_common::*;
use ignis_core::contracts::{CheckStatus, Family, ModelRelease, ModelSpec, PrefixBundle, RawValue};
use ignis_core::registry::LoadedRelease;
use ignis_wsp::committee3::{SLOTS, SLOT_PREFIX};
use ignis_wsp::{
    audit, cli, committee2, committee3, flame, flame3, flame4, flame5, flame6, flame7, models, text,
};
use serde_json::json;

fn list(v: &[f64]) -> String {
    v.iter()
        .map(|x| text::literal(*x))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The committee's own lines: weights of both eras on and off the script path, the era rule and the
/// offsets.
fn own(
    w_early: [f64; SLOTS],
    w_late: [f64; SLOTS],
    ws_early: [f64; SLOTS],
    ws_late: [f64; SLOTS],
) -> String {
    format!(
        "mix.w_early[0:10] = [{}]\nmix.w_late[0:10] = [{}]\nmix.ws_early[0:10] = [{}]\n\
         mix.ws_late[0:10] = [{}]\nera.last_early_day = 2021242.0\nera.first_late_day = 2021270.0\n\
         era.p_late_between = 0.5\nera.p_late_unknown = 0.15\nscript.offset_s[0:3] = [0.0, 1.0, 2.0]\n\
         script.offset_w[0:3] = [0.25, 0.5, 0.25]\n",
        list(&w_early),
        list(&w_late),
        list(&ws_early),
        list(&ws_late)
    )
}

const W_EARLY: [f64; SLOTS] = [1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
const W_LATE: [f64; SLOTS] = [0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0];
const WS_EARLY: [f64; SLOTS] = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
const WS_LATE: [f64; SLOTS] = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// The member releases of one kind of forward, by slot kind.
struct Members {
    a: LoadedRelease,
    b: LoadedRelease,
    c: LoadedRelease,
}

/// The indicator members of the second committee (two kinds of hinge members differ in their slots
/// by the stop atom of a1, so slots are not all the same release).
fn old_members(family: Family) -> Members {
    Members {
        a: release(&key(family, 2), &format!("{HINGE}{LIMIT_CLOCK}")),
        b: release(&key(family, 3), &format!("{GROWTH}{BURNER}")),
        c: release(&key(family, 8), &format!("{HINGE}{BURNER}")),
    }
}

/// The same law numbers with a fuel table.
fn new_members(family: Family, table: &str) -> Members {
    Members {
        a: release(&key(family, 9), &format!("{HINGE}{LIMIT_CLOCK}{table}")),
        b: release(&key(family, 10), &format!("{GROWTH}{BURNER}{table}")),
        c: release(&key(family, 11), &format!("{HINGE}{BURNER}{table}")),
    }
}

/// The edits of a committee: `own`, then each slot's member statement by statement. `bend` changes
/// slot b1's flux law so that two slots of one kind are told apart.
fn committee_edits(specs: [&ModelSpec; 3], m: &Members, own: &str, bend: bool) -> String {
    let mut src = own.to_string();
    for (i, slot) in SLOT_PREFIX.iter().enumerate() {
        let (spec, r) = if i < 2 {
            (specs[0], &m.a)
        } else if i < 6 {
            (specs[1], &m.b)
        } else {
            (specs[2], &m.c)
        };
        for line in cli::edits_of(spec, r).lines() {
            let line = if bend && *slot == "b1." && line.starts_with("se.flux.b0 = ") {
                "se.flux.b0 = 0.1".to_string()
            } else {
                line.to_string()
            };
            src.push_str(slot);
            src.push_str(&line);
            src.push('\n');
        }
    }
    src
}

fn wsp7(family: Family, own: &str) -> LoadedRelease {
    let m = old_members(family);
    let specs = [
        &flame::spec(family),
        &flame3::spec(family),
        &flame4::spec(family),
    ];
    release(&key(family, 7), &committee_edits(specs, &m, own, true))
}

fn wsp12(family: Family, own: &str, table: &str) -> LoadedRelease {
    let m = new_members(family, table);
    let specs = [
        &flame5::spec(family),
        &flame6::spec(family),
        &flame7::spec(family),
    ];
    release(&key(family, 12), &committee_edits(specs, &m, own, true))
}

fn with_stop(p: &PrefixBundle, t: f64) -> PrefixBundle {
    let mut p = p.clone();
    let mut stop = p.known_conditions["pressure"].clone();
    stop.raw_value = Some(RawValue::Number(t));
    stop.value_si = Some(t);
    stop.raw_unit = Some("s".into());
    stop.unit_si = Some("s".into());
    p.known_conditions
        .insert(committee3::PROGRAMMED_STOP.into(), stop);
    rehash(&mut p);
    p
}

fn dated(p: &PrefixBundle, session: &str) -> PrefixBundle {
    let mut p = p.clone();
    p.session_id = Some(session.into());
    rehash(&mut p);
    p
}

fn settings() -> serde_json::Value {
    json!({ "times_s": TIMES })
}

fn run7(r: &LoadedRelease, p: &PrefixBundle) -> committee2::Forecast {
    committee2::forecast(&committee2::spec(p.family), r, p, &settings())
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn try12(
    r: &LoadedRelease,
    p: &PrefixBundle,
) -> Result<committee3::Forecast, ignis_core::contracts::Support> {
    committee3::forecast(&committee3::spec(p.family), r, p, &settings()).expect("runnable")
}

fn run12(r: &LoadedRelease, p: &PrefixBundle) -> committee3::Forecast {
    try12(r, p).unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

#[test]
fn the_spec_is_the_fuel_table_member_specs_under_ten_prefixes() {
    for (family, dir) in FAMILIES {
        let spec = committee3::spec(family);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(&key(family, 12)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp12/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/acme/wsp12/{dir}/spec.json drifted from committee3::spec()"
        );
        let old = committee2::spec(family);
        let n_own = committee3::OWN_TENSORS.len();
        let names = |s: &ModelSpec| {
            s.tensors[..n_own]
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&spec), names(&old));
        for (a, b) in spec.tensors[..n_own].iter().zip(&old.tensors[..n_own]) {
            assert_eq!(
                (&a.shape, a.role, &a.bounds, &a.fill),
                (&b.shape, b.role, &b.bounds, &b.fill)
            );
        }
        let mut at = n_own;
        for (i, slot) in SLOT_PREFIX.iter().enumerate() {
            let member = if i < 2 {
                flame5::spec(family)
            } else if i < 6 {
                flame6::spec(family)
            } else {
                flame7::spec(family)
            };
            for t in &member.tensors {
                let have = &spec.tensors[at];
                assert_eq!(have.name, format!("{slot}{}", t.name));
                assert_eq!(
                    (&have.shape, have.role, &have.fill),
                    (&t.shape, t.role, &t.fill)
                );
                at += 1;
            }
            assert!(spec.tensor(&format!("{slot}fuel.ft_coord")).is_some());
        }
        assert_eq!(at, spec.tensors.len());
    }
}

#[test]
fn with_neutral_members_it_answers_as_the_second_committee_bit_for_bit_on_every_path() {
    for (family, _) in FAMILIES {
        let own = own(W_EARLY, W_LATE, WS_EARLY, WS_LATE);
        let old = wsp7(family, &own);
        for table in [
            NEUTRAL,
            if family == Family::AcmeInverse {
                INVERSE_34
            } else {
                NORMAL_34
            },
        ] {
            let new = wsp12(family, &own, table);
            for (what, p) in alkane_prefixes(family) {
                for (era, q) in [
                    ("no date", p.clone()),
                    ("early", dated(&p, "S 2021-03-04")),
                    ("between", dated(&p, "S 2021-09-10")),
                    ("late", dated(&p, "S 2021-10-01")),
                ] {
                    for (path, r) in [("off script", q.clone()), ("script", with_stop(&q, 40.0))] {
                        let (a, b) = (run7(&old, &r), run12(&new, &r));
                        assert_eq!(a.table, b.table, "{family:?} {what} {era} {path}");
                        assert_eq!(a.p_late, b.p_late);
                        assert_eq!(a.members.len(), b.members.len());
                        check_partition(&b.table);
                    }
                }
            }
        }
    }
}

#[test]
fn each_member_reads_the_fuel_by_its_own_table_and_its_refusal_refuses() {
    let family = Family::AcmeNormal;
    let own = own(W_EARLY, W_LATE, WS_EARLY, WS_LATE);
    let neutral = wsp12(family, &own, NEUTRAL);
    let p = prefix(family, "ethylene");
    let s = try12(&neutral, &p).map(|_| ()).expect_err("refused");
    assert!(
        s.reasons
            .iter()
            .any(|r| r.starts_with("member a0: the release does not answer for ethylene")),
        "{:?}",
        s.reasons
    );
    // the design's table in every slot: ethylene is answered, each member with its own vertices
    let design = wsp12(family, &own, NORMAL_34);
    let f = run12(&design, &p);
    assert!(!f.members.is_empty());
    for m in &f.members {
        assert_eq!(
            m.clocks["clocks"]["fuel"]["name"]
                .as_str()
                .or(m.clocks["fuel"]["name"].as_str()),
            Some("ethylene"),
            "{}",
            m.slot
        );
    }
    check_partition(&f.table);
    // one slot with the neutral table among design tables: that member refuses, and so the forecast
    let mut mixed = wsp12(family, &own, NORMAL_34);
    set(&mut mixed, "c0.fuel.admit", &[1.0, 1.0, 1.0, 0.0, 1.0, 0.0]);
    let s = try12(&mixed, &p).map(|_| ()).expect_err("refused");
    assert!(
        s.reasons
            .iter()
            .any(|r| r.starts_with("member c0: the release does not answer for ethylene")),
        "{:?}",
        s.reasons
    );
}

#[test]
fn one_member_on_one_node_answers_with_its_own_table() {
    for (family, _) in FAMILIES {
        let mut w = [0.0; SLOTS];
        w[3] = 1.0;
        let own = own(w, w, [0.0; SLOTS], [0.0; SLOTS]);
        let r = wsp12(family, &own, NEUTRAL);
        let m = new_members(family, NEUTRAL);
        let mut b1 = m.b;
        set(&mut b1, "se.flux.b0", &[0.1]);
        for (_, p) in alkane_prefixes(family).into_iter().take(6) {
            let f = run12(&r, &p);
            assert_eq!(f.members.len(), 1);
            let alone = flame6::forecast(&flame6::spec(family), &b1, &p, &settings())
                .unwrap()
                .expect("admitted");
            assert_eq!(f.table, alone.table);
        }
    }
}

#[test]
fn the_forward_identity_covers_the_fuel_table_and_the_three_member_sources() {
    for name in ["acme_normal.wsp12", "acme_inverse.wsp12"] {
        let files: Vec<&str> = audit::forward_sources_of(name)
            .unwrap()
            .iter()
            .map(|(n, _)| *n)
            .collect();
        for f in [
            "fuel.rs",
            "flame5.rs",
            "flame6.rs",
            "flame7.rs",
            "committee3.rs",
        ] {
            assert!(files.contains(&f), "{f} {files:?}");
        }
        for f in ["flame.rs", "flame3.rs", "flame4.rs", "committee2.rs"] {
            assert!(!files.contains(&f), "{f} {files:?}");
        }
        assert_eq!(audit::banned_hits_of(name), Vec::<String>::new());
    }
    assert_ne!(
        audit::forward_hash_of("acme_normal.wsp12"),
        audit::forward_hash_of("acme_normal.wsp7")
    );
}

#[test]
fn the_committed_roots_hold_their_source_releases_numbers() {
    let read = |path: &str| -> ModelRelease { serde_json::from_str(&config(path)).unwrap() };
    let hashes = |r: &ModelRelease, prefix: &str| -> Vec<(String, Vec<i64>, String)> {
        r.tensors
            .iter()
            .filter_map(|t| {
                t.name
                    .strip_prefix(prefix)
                    .map(|n| (n.to_string(), t.shape.clone(), t.sha256.clone()))
            })
            .collect()
    };
    for (family, dir) in FAMILIES {
        let root = read(&format!("wsp12/{dir}/baseline-set/release.json"));
        let a = read(&format!("wsp9/{dir}/baseline-clock/release.json"));
        let b = read(&format!("wsp10/{dir}/baseline-theory/release.json"));
        let c = read(&format!("wsp11/{dir}/baseline-clock-theory/release.json"));
        for (i, slot) in SLOT_PREFIX.iter().enumerate() {
            let source = if i < 2 {
                &a
            } else if i < 6 {
                &b
            } else {
                &c
            };
            assert_eq!(hashes(&root, slot), hashes(source, ""), "{dir} {slot}");
        }
        // the committee's own tensors are those of the root of the second committee
        let w7 = read(&format!("wsp7/{dir}/baseline-set/release.json"));
        for name in committee3::OWN_TENSORS {
            let have = root.tensors.iter().find(|t| t.name == name).unwrap();
            let want = w7.tensors.iter().find(|t| t.name == name).unwrap();
            assert_eq!(
                (&have.shape, &have.sha256),
                (&want.shape, &want.sha256),
                "{name}"
            );
        }
        let k = root.construction.as_ref().unwrap();
        assert_eq!(k.forward_hash, audit::forward_hash_of(&root.model_name));
        assert!(k.checks.iter().all(|x| x.status == CheckStatus::Passed));
        let from_edits = release(
            &key(family, 12),
            &config(&format!("wsp12/{dir}/baseline-set/edits.wsp")),
        );
        assert_eq!(
            from_edits
                .release
                .construction
                .as_ref()
                .unwrap()
                .weights_hash,
            k.weights_hash
        );
    }
}

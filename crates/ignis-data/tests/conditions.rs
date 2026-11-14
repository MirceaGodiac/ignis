//! R9a conditions table: the parser keeps raw value, unit, SI value and locator; lists, ranges,
//! unreadable and physically impossible cells become explicit gaps (never zero, never corrected);
//! envelopes describe the test atmosphere only; every flight investigation is covered by exactly one
//! source. Real NASA tables (tests/fixtures, verbatim, hashes = data/manifests).

use std::collections::BTreeSet;
use std::path::PathBuf;

use ignis_core::contracts::{ConditionCell, ConditionKey as K, EnvelopeEntry, RawValue};
use ignis_core::contracts::{ConditionFilter, ConditionsQuery, DatasetSchema, Finite, RunId};
use ignis_data::conditions::{build_from, parse_cell, plausible, query, Parsed, FAMILIES, SPECS};
use ignis_data::sources::storage::FLIGHTS;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn cells_parse_without_inventing_values() {
    assert_eq!(
        parse_cell("~21% O2 with N2"),
        Parsed::One {
            x: 21.0,
            unit: Some("%".into()),
            approx: true
        },
        "digits of O2/N2 are formulas, not numbers"
    );
    assert_eq!(
        parse_cell("1 atm"),
        Parsed::One {
            x: 1.0,
            unit: Some("atm".into()),
            approx: false
        }
    );
    assert_eq!(
        parse_cell(".33 cm"),
        Parsed::One {
            x: 0.33,
            unit: None,
            approx: false
        }
    );
    for many in [
        "21; 26.5; 34",
        "1.5 - 5",
        "22.0 to 21.9",
        "~30% and ~21% O2 with He",
    ] {
        assert!(matches!(parse_cell(many), Parsed::Many { .. }), "{many}");
    }
    assert_eq!(
        parse_cell("-10 C"),
        Parsed::One {
            x: -10.0,
            unit: Some("c".into()),
            approx: false
        },
        "a leading minus is a sign"
    );
    assert_eq!(
        parse_cell("10-20"),
        Parsed::Many {
            xs: vec![10.0, 20.0],
            unit: None
        },
        "a dash between numbers is a range, not a sign"
    );
    assert_eq!(parse_cell(""), Parsed::Empty);
    assert_eq!(parse_cell(" – "), Parsed::Empty);
    assert_eq!(parse_cell("RAMP"), Parsed::Unparsed);
    // NASA's BRE table writes 34 % as "3400%": a mole fraction of 34 is impossible.
    assert!(!plausible(K::OxygenMoleFraction, 34.0));
    assert!(!plausible(K::Pressure, 0.0));
    assert!(plausible(K::OxygenMoleFraction, 0.21) && plausible(K::Pressure, 101_325.0));
}

#[test]
fn every_flight_investigation_has_exactly_one_conditions_source() {
    // Family records cover FLEX (PSI-69), CFI-G (PSI-159), BASS-II (PSI-25) and BASS (PSI-26).
    assert_eq!(FAMILIES, ["flex", "acme_normal", "acme_inverse", "bass"]);
    let by_family: BTreeSet<&str> = ["PSI-69", "PSI-159", "PSI-25", "PSI-26"].into();
    let by_spec: BTreeSet<&str> = SPECS.iter().map(|s| s.investigation).collect();
    assert!(
        by_family.is_disjoint(&by_spec),
        "an investigation has two sources"
    );
    let all: BTreeSet<&str> = by_family.union(&by_spec).copied().collect();
    let flights: BTreeSet<&str> = FLIGHTS.iter().map(|f| f.accession).collect();
    assert_eq!(all, flights);
    // Every spec names a file NASA lists as that investigation's experimental table.
    for s in &SPECS {
        let listing = std::fs::read_to_string(
            repo().join(format!("data/manifests/{}.listing.jsonl", s.investigation)),
        )
        .unwrap();
        assert!(
            listing.lines().any(|l| {
                let v: serde_json::Value = serde_json::from_str(l).unwrap();
                v["file_name"] == s.file && v["category"] == "Experimental table"
            }),
            "{}: {} is not an experimental table in the listing",
            s.investigation,
            s.file
        );
    }
}

static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// IGNIS_DATA root with the manifests of the fixture tables and the tables themselves. The
/// environment is process-global, so callers hold LOCK for the whole test.
fn data_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!("ignis-conditions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for inv in ["PSI-20", "PSI-23", "PSI-68", "PSI-98"] {
        let fx = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(inv);
        let raw = root.join("raw/samples").join(inv);
        std::fs::create_dir_all(&raw).unwrap();
        for e in std::fs::read_dir(&fx).unwrap().flatten() {
            std::fs::copy(e.path(), raw.join(e.file_name())).unwrap();
        }
        let m = root.join("manifests");
        std::fs::create_dir_all(&m).unwrap();
        let all =
            std::fs::read_to_string(repo().join(format!("data/manifests/{inv}.jsonl"))).unwrap();
        let lines: Vec<&str> = all
            .lines()
            .filter(|l| l.contains("Experimental table"))
            .collect();
        std::fs::write(m.join(format!("{inv}.jsonl")), lines.join("\n") + "\n").unwrap();
    }
    root
}

#[test]
fn real_tables_give_explicit_gaps_and_atmosphere_envelopes() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("IGNIS_DATA", data_root());
    let specs: Vec<_> = SPECS
        .iter()
        .copied()
        .filter(|s| ["PSI-20", "PSI-23", "PSI-68"].contains(&s.investigation))
        .collect();
    let t = build_from(&[], &specs).unwrap();
    t.validate().unwrap();
    fn rows<'a>(
        t: &'a ignis_core::contracts::ConditionsTable,
        inv: &'a str,
    ) -> impl Iterator<Item = &'a ignis_core::contracts::ConditionsRow> + 'a {
        t.rows.iter().filter(move |r| r.investigation_id == inv)
    }
    let cell = |r: &ignis_core::contracts::ConditionsRow, k: K| {
        r.cells.iter().find(|c| c.key() == k).unwrap().clone()
    };

    // s-Flame: "~21% O2 with N2" is one approximate value with its raw text and cell locator.
    let r = rows(&t, "PSI-23").next().unwrap();
    let ConditionCell::Value { value, .. } = cell(r, K::OxygenMoleFraction) else {
        panic!("s-Flame O2 should be a value")
    };
    assert_eq!(value.raw_value, Some(RawValue::Number(21.0)));
    assert_eq!(value.raw_unit.as_deref(), Some("%"));
    assert_eq!(value.value_si, Some(0.21));
    assert!(value.flags.contains(&"approximate".to_string()));
    let l = value.locator.unwrap();
    assert_eq!(l.column.as_deref(), Some("Atmosphere*"));
    assert_eq!(
        l.path.as_deref(),
        Some("PSI-23_Experimental table_ACME s-Flame.csv")
    );

    // BRE: lists are gaps at row level; "3400%" is refused, not read as 34 nor corrected.
    let bre: Vec<_> = rows(&t, "PSI-20")
        .map(|r| cell(r, K::OxygenMoleFraction))
        .collect();
    assert!(bre
        .iter()
        .all(|c| !matches!(c, ConditionCell::Value { .. })));
    assert!(bre.iter().any(|c| matches!(c, ConditionCell::Missing { reason, .. } if reason.contains("several tested values"))));
    assert!(bre.iter().any(|c| matches!(c, ConditionCell::Missing { reason, .. } if reason.contains("implausible") && reason.contains("3400"))));
    // Every row has a cell for every key, and no table states a gravity level.
    for r in &t.rows {
        assert_eq!(r.cells.len(), t.keys.len());
        assert!(matches!(
            cell(r, K::GravityLevel),
            ConditionCell::Missing { .. }
        ));
    }

    // Envelopes: the listed values become the investigation's tested range, in SI.
    let env = |inv: &str, k: K| {
        t.envelopes
            .iter()
            .find(|e| e.investigation_id == inv)
            .unwrap()
            .entries
            .iter()
            .find(|e| match e {
                EnvelopeEntry::Range { key, .. }
                | EnvelopeEntry::Values { key, .. }
                | EnvelopeEntry::Missing { key, .. } => *key == k,
            })
            .unwrap()
            .clone()
    };
    let EnvelopeEntry::Range {
        min_si,
        max_si,
        unit_si,
        ..
    } = env("PSI-20", K::OxygenMoleFraction)
    else {
        panic!("BRE O2 envelope")
    };
    assert_eq!(
        (min_si.get(), max_si.get(), unit_si.as_str()),
        (0.21, 0.40, "1")
    );
    let EnvelopeEntry::Range { min_si, max_si, .. } = env("PSI-68", K::Pressure) else {
        panic!("FLEX-2 pressure envelope")
    };
    assert!(
        (min_si.get() - 0.5 * 101_325.0).abs() < 1e-6
            && (max_si.get() - 3.0 * 101_325.0).abs() < 1e-6
    );
    assert!(matches!(
        env("PSI-20", K::Pressure),
        EnvelopeEntry::Missing { .. }
    ));
}

#[test]
fn queries_filter_project_page_and_keep_the_lockbox() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("IGNIS_DATA", data_root());
    let specs: Vec<_> = SPECS
        .iter()
        .copied()
        .filter(|s| ["PSI-23", "PSI-98"].contains(&s.investigation))
        .collect();
    let full = build_from(&[], &specs).unwrap();
    let q = |f: &dyn Fn(&mut ConditionsQuery)| {
        let mut q = ConditionsQuery {
            schema_version: DatasetSchema::V2,
            investigation_ids: vec![],
            run_ids: vec![],
            keys: vec![],
            filters: vec![],
            include_unknown: false,
            include_envelopes: true,
            page_size: None,
            cursor: None,
        };
        f(&mut q);
        q
    };
    let all = |_: &RunId| true;

    // "O2 at least 30 %": only s-Flame rows stated as one value of ~30 % qualify; a missing O2 does not.
    let o2_30 = ConditionFilter {
        key: K::OxygenMoleFraction,
        min_si: Some(Finite::new(0.3).unwrap()),
        max_si: None,
        any_of: Default::default(),
    };
    let hi = query(&full, &q(&|q| q.filters = vec![o2_30.clone()]), all).unwrap();
    assert!(hi.rows.iter().all(|r| r.investigation_id == "PSI-23"));
    assert!(!hi.rows.is_empty());
    // With include_unknown, rows whose O2 is not a single value are returned too, never as matches.
    let with_unknown = query(
        &full,
        &q(&|q| {
            q.filters = vec![o2_30.clone()];
            q.include_unknown = true
        }),
        all,
    )
    .unwrap();
    assert!(with_unknown.total_count > hi.total_count);

    // Projection to two keys keeps one cell per kept key.
    let proj = query(
        &full,
        &q(&|q| q.keys = vec![K::Fuel, K::OxygenMoleFraction]),
        all,
    )
    .unwrap();
    assert_eq!(proj.keys, vec![K::Fuel, K::OxygenMoleFraction]);
    assert!(proj.rows.iter().all(|r| r.cells.len() == 2));

    // Paging: pages of 5 cover every row exactly once.
    let mut seen = 0;
    let mut cursor = None;
    loop {
        let page = query(
            &full,
            &q(&|q| {
                q.page_size = Some(5);
                q.cursor = cursor.clone()
            }),
            all,
        )
        .unwrap();
        seen += page.rows.len();
        match page.next_cursor {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    assert_eq!(seen, full.rows.len());

    // Lockbox: a run the scope may not see is withheld, and the envelope is rebuilt without it.
    let hidden = RunId::new("PSI-98:S1").unwrap();
    let t = query(
        &full,
        &q(&|q| q.investigation_ids = vec!["PSI-98".into()]),
        |id| *id != hidden,
    )
    .unwrap();
    assert!(t
        .rows
        .iter()
        .all(|r| r.run.run_id.as_ref() != Some(&hidden)));
    assert_eq!(t.total_count, Some(3));
    assert_eq!(t.envelopes.len(), 1);
    assert_eq!(
        t.envelopes[0].runs, 3,
        "the envelope counts only visible rows"
    );
}

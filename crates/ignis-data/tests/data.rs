//! Essential data checks: one real NASA record per family parsed, SI-normalized and traceable to
//! the committed CC0 bytes; split groups never cross partitions.
//!
//! The data root holds only the verbatim fixtures (`tests/fixtures/<investigation>/`) and their
//! committed manifest records, so every parsed value is traceable to the NASA asset and locator.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ignis_core::contracts::{Censoring, Family, Quantity, RawValue, Resolution};
use ignis_core::prefix::{build_prefix, PrefixInputs};
use ignis_core::views::{View, ViewSource};
use ignis_data::families::{acme, bass, flex};
use ignis_data::{cohort, store, DataStore};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One shared data root for the whole test binary (IGNIS_DATA is process-global).
fn data_root() -> &'static PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("ignis-data-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let fix = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let man = repo().join("data/manifests");
        std::fs::create_dir_all(root.join("manifests")).unwrap();
        for inv in ["PSI-69", "PSI-159", "PSI-25"] {
            let raw = root.join("raw/samples").join(inv);
            std::fs::create_dir_all(&raw).unwrap();
            let mut names = vec![];
            for e in std::fs::read_dir(fix.join(inv)).unwrap().flatten() {
                std::fs::copy(e.path(), raw.join(e.file_name())).unwrap();
                names.push(e.file_name().to_string_lossy().to_string());
            }
            let text = std::fs::read_to_string(man.join(format!("{inv}.jsonl"))).unwrap();
            let keep: Vec<&str> = text
                .lines()
                .filter(|l| {
                    let v: serde_json::Value = serde_json::from_str(l).unwrap();
                    names.iter().any(|n| v["file_name"] == *n)
                })
                .collect();
            assert_eq!(
                keep.len(),
                names.len(),
                "{inv}: every fixture must have a committed Source record"
            );
            for l in &keep {
                // the fixture bytes are the NASA bytes named in the manifest
                let v: serde_json::Value = serde_json::from_str(l).unwrap();
                let p = raw.join(v["file_name"].as_str().unwrap());
                assert_eq!(
                    v["id"].as_str().unwrap(),
                    format!("sha256:{}", ignis_core::hashing::file_sha256(&p).unwrap())
                );
                let url = v["canonical_url"].as_str().unwrap();
                assert!(url.contains("geode-py/ws/studies") && !url.contains("Signature="));
            }
            std::fs::write(
                root.join("manifests").join(format!("{inv}.jsonl")),
                keep.join("\n") + "\n",
            )
            .unwrap();
            for extra in [
                format!("{inv}.investigation.json"),
                format!("{inv}.listing.jsonl"),
            ] {
                std::fs::copy(man.join(&extra), root.join("manifests").join(&extra)).unwrap();
            }
        }
        std::fs::copy(
            man.join("PSI-26.listing.jsonl"),
            root.join("manifests/PSI-26.listing.jsonl"),
        )
        .unwrap();
        std::env::set_var("IGNIS_DATA", &root);
        root
    })
}

/// Line `row` (1-based) of a cp1252/CR-terminated NASA text file.
fn line(path: &Path, row: i64) -> String {
    let text = ignis_data::text::normalize_newlines(&ignis_data::text::decode_cp1252(
        &std::fs::read(path).unwrap(),
    ));
    text.split('\n').nth(row as usize - 1).unwrap().to_string()
}

fn num(v: &Option<RawValue>) -> f64 {
    match v {
        Some(RawValue::Number(x)) => *x,
        o => panic!("not a number: {o:?}"),
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * a.abs().max(b.abs())
}

#[test]
fn flex_real_record() {
    let root = data_root();
    let b = flex::ingest().unwrap();
    let rec = b
        .records
        .iter()
        .find(|r| r.id == "PSI-69:192F001:2009-10-24")
        .expect("FLEX-020 bound to its summary row");
    assert_eq!(rec.identity_resolution, Resolution::Resolved);
    for s in ["file_flex_number", "cir_test_point", "summary_flex_number"] {
        assert!(rec.aliases.iter().any(|a| a.scheme == s));
    }
    let p = &rec.conditions["pressure"];
    assert_eq!(p.raw_unit.as_deref(), Some("mmHg"));
    assert!(close(p.value_si.unwrap(), 539.5 * 133.322387415));
    assert!(rec.conditions["x_o2"]
        .flags
        .contains(&"original_header:O".to_string()));
    assert_eq!(rec.conditions["x_o2"].value_si, Some(0.34));
    // The summary row is traceable: the cited row holds the raw value.
    let l = p.locator.as_ref().unwrap();
    assert!(line(
        &root
            .join("raw/samples/PSI-69")
            .join(l.path.as_ref().unwrap()),
        l.row.unwrap()
    )
    .contains("539.5"));
    // Burning rate is post-run, in m^2/s, and cannot enter a prefix.
    let k = &b.post[&rec.id]["burning_rate"];
    assert_eq!(k.unit_si.as_deref(), Some("m2/s"));
    assert!(close(k.value_si.unwrap(), 0.53e-6) && !rec.conditions.contains_key("burning_rate"));
    let mut obs: Vec<_> = b
        .observations
        .iter()
        .filter(|o| o.run_id == rec.id && o.quantity == Quantity::LiquidDiameter)
        .cloned()
        .collect();
    obs.sort_by(|a, b| a.time_s.total_cmp(&b.time_s));
    assert!(obs.len() > 50);
    let o = &obs[10];
    assert!(close(o.value_si.unwrap(), num(&o.raw_value) * 1e-3) && o.unit_si == "m");
    let dd = root
        .join("raw/samples/PSI-69")
        .join(o.locator.path.as_ref().unwrap());
    assert_eq!(
        o.locator.asset_sha256,
        ignis_core::hashing::file_sha256(&dd).unwrap()
    );
    let cells: Vec<f64> = line(&dd, o.locator.row.unwrap())
        .split('\t')
        .take(2)
        .map(|c| c.trim().parse().unwrap())
        .collect();
    assert_eq!((cells[0], cells[1]), (o.time_s, num(&o.raw_value)));
    let pol = flex::policy();
    let prefix = build_prefix(
        rec,
        &obs,
        2.0,
        &pol,
        "nasa_processed",
        PrefixInputs::default(),
    )
    .unwrap();
    assert!(
        prefix.observations.iter().all(|x| x.time_s <= 2.0)
            && prefix.known_conditions.contains_key("fuel")
    );
    let mut bad = rec.clone();
    bad.conditions
        .insert("burn_time".into(), b.post[&rec.id]["burn_time"].clone());
    assert!(build_prefix(
        &bad,
        &obs,
        2.0,
        &pol,
        "nasa_processed",
        PrefixInputs::default()
    )
    .is_err());
}

#[test]
fn acme_real_record() {
    let root = data_root();
    let b = acme::ingest(Family::AcmeNormal).unwrap();
    assert!(b.records.iter().all(|r| r.family == Family::AcmeNormal) && b.records.len() == 179);
    let rid = "PSI-159:acme_normal:21088C1";
    let rec = b.records.iter().find(|r| r.id == rid).unwrap();
    assert_eq!(rec.session_id.as_deref(), Some("PSI-159:day:2021-03-29"));
    assert!(close(
        rec.conditions["pressure"].value_si.unwrap(),
        0.9443956e5
    ));
    assert!(close(
        rec.conditions["injected_mass_flow"].value_si.unwrap(),
        2e-6
    )); // mHC 2 mg/s, fuel injected
    assert!(close(
        rec.conditions["x_o2_role"].value_si.unwrap(),
        0.386321188
    )); // chamber O2 before test
    assert!(
        !rec.conditions.contains_key("x_o2_after")
            && rec.conditions["tad"]
                .flags
                .contains(&"derived_mixture_property".into())
    );
    // SE label vs 'hit burner T limit' stays unresolved, both causes admitted, both cited.
    let ep = b.endpoints.iter().find(|e| e.run_id == rid).unwrap();
    assert_eq!(ep.resolution, Resolution::UnresolvedConflict);
    assert_eq!(
        ep.allowed_causes,
        vec!["extinction", "operator_termination"]
    );
    assert_eq!(ep.censoring, Censoring::Interval);
    assert_eq!(
        (ep.last_confirmed_active_s, ep.first_terminal_s),
        (Some(41.5), Some(42.5))
    );
    let ev = &ep.evidence[2];
    assert!(line(
        &root
            .join("raw/samples/PSI-159")
            .join(ev.path.as_ref().unwrap()),
        ev.row.unwrap()
    )
    .contains("21088 C1"));
    assert!(b
        .review
        .iter()
        .any(|i| i.run_id == rid && i.field == "endpoint_cause"));
    let hist = b.extra["control_history"]
        .iter()
        .find(|h| h["run_id"] == rid)
        .unwrap();
    let off = hist["stages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["control"] == "MFCs off")
        .unwrap();
    assert!(off["operator_action"].as_bool().unwrap() && off["t_start_s"].as_f64() == Some(52.0));
}

#[test]
fn bass_real_record() {
    data_root();
    let b = bass::ingest().unwrap();
    let rec = b.records.iter().find(|r| r.id == "PSI-25:B1").unwrap();
    assert!(
        close(rec.conditions["x_o2"].value_si.unwrap(), 0.222)
            && close(rec.conditions["flow_velocity"].value_si.unwrap(), 0.05)
    );
    assert!(close(
        b.specimens[rec.specimen_id.as_ref().unwrap()].geometry["thickness"]
            .value_si
            .unwrap(),
        100e-6
    ));
    assert!(
        close(b.post[&rec.id]["x_o2_final"].value_si.unwrap(), 0.222)
            && !rec.conditions.contains_key("x_o2_final")
    );
    let vm = bass::video_map(&b);
    let v = vm
        .iter()
        .find(|v| v.file_name == "PSI-25_Raw Data_BASS_GMT50_SAMPLE_147_TEST_B1.mp4")
        .unwrap();
    assert_eq!(
        (
            v.run_id.as_deref(),
            v.resolution.as_str(),
            v.variant.as_str()
        ),
        (Some("PSI-25:B1"), "resolved", "main")
    );
    assert!(vm.iter().any(|x| x.resolution != "resolved")); // unmapped files are explicit
}

#[test]
fn split_groups_never_cross_partitions() {
    data_root();
    // The freezer reproduces numpy default_rng(seed).permutation bit for bit (vectors from numpy 2.5.4),
    // which keeps every split frozen before the Rust port valid.
    assert_eq!(
        ignis_data::npyrng::permutation(20261107, 11),
        vec![2, 4, 7, 5, 0, 8, 10, 6, 9, 3, 1]
    );
    assert_eq!(
        ignis_data::npyrng::permutation(7, 11),
        vec![4, 6, 10, 0, 1, 3, 8, 7, 2, 5, 9]
    );
    store::save(&acme::ingest(Family::AcmeNormal).unwrap()).unwrap();
    let (coh, _) = cohort::build("acme_normal").unwrap();
    let (split, _) = cohort::freeze(&coh.id, 7).unwrap();
    let parts = [&split.train, &split.validation, &split.test];
    let groups: std::collections::BTreeSet<&String> = split.run_to_group.values().collect();
    let all: std::collections::BTreeSet<&String> = parts.iter().flat_map(|p| p.iter()).collect();
    assert_eq!(all, groups);
    assert_eq!(parts.iter().map(|p| p.len()).sum::<usize>(), groups.len()); // each group in exactly one
    assert!(coh.run_ids.iter().all(|r| !groups.contains(r))); // partitions hold groups, not runs
    let recs = store::load("acme_normal").unwrap().records;
    let mut by_session: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        Default::default();
    for rid in split.run_to_group.keys() {
        let s = recs
            .iter()
            .find(|r| &r.id == rid)
            .unwrap()
            .session_id
            .clone()
            .unwrap();
        by_session
            .entry(s)
            .or_default()
            .insert(format!("{:?}", split.partition_of(rid).unwrap()));
    }
    assert!(by_session.values().all(|p| p.len() == 1)); // no test day crosses partitions
    assert!(cohort::freeze(&coh.id, 7).is_err()); // refuses to refreeze
                                                  // Experts load the same split and views through the ViewSource.
    let ds = DataStore::new();
    assert_eq!(ds.split(&split.id).unwrap(), split);
    let views = ds.views(Family::AcmeNormal, &coh.id).unwrap();
    assert_eq!(views.len(), coh.run_ids.len());
    assert!(views.iter().all(|v| matches!(v, View::Acme(a) if a.group_id == split.run_to_group[&a.run_id] && !a.time_origin.is_empty())));
}

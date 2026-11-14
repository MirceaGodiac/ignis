//! The R6 storage rule reproduces Iancu's atlas exactly: every file the atlas says the server stores
//! is classified as stored by `storage_class`, in the right group, for the same 19 investigations and
//! the same byte totals. A drift here would silently drop documents from the corpus or start storing
//! NASA media.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::PathBuf;

use ignis_data::sources::storage::{flight, storage_class, StorageClass, FLIGHTS};

fn atlas(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/wsp/atlas")
        .join(name)
}

/// Minimal RFC 4180 line splitter (the atlas quotes the `updated` column, which contains commas).
fn split(line: &str) -> Vec<String> {
    let (mut out, mut cur, mut quoted) = (Vec::new(), String::new(), false);
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

struct Row {
    investigation: String,
    category: String,
    file_name: String,
    size_bytes: u64,
}

fn rows(text: &str) -> Vec<Row> {
    let mut lines = text.lines();
    let header = split(lines.next().unwrap());
    let col = |n: &str| header.iter().position(|h| h == n).unwrap();
    let (i, c, f, s) = (
        col("investigation"),
        col("category"),
        col("file_name"),
        col("size_bytes"),
    );
    lines
        .filter(|l| !l.is_empty())
        .map(|l| {
            let v = split(l);
            Row {
                investigation: v[i].clone(),
                category: v[c].clone(),
                file_name: v[f].clone(),
                size_bytes: v[s].parse().unwrap(),
            }
        })
        .collect()
}

fn check(rows: &[Row], want: StorageClass) -> (BTreeMap<&str, usize>, u64) {
    let mut per = BTreeMap::new();
    let mut bytes = 0;
    for r in rows {
        assert_eq!(
            storage_class(&r.category, &r.file_name),
            want,
            "{} / {} / {}",
            r.investigation,
            r.category,
            r.file_name
        );
        assert!(flight(&r.investigation).is_some(), "{}", r.investigation);
        *per.entry(r.investigation.as_str()).or_insert(0) += 1;
        bytes += r.size_bytes;
    }
    (per, bytes)
}

#[test]
fn storage_rule_matches_the_atlas() {
    let docs = rows(&std::fs::read_to_string(atlas("server_documents.csv")).unwrap());
    let mut gz = String::new();
    flate2::read::GzDecoder::new(std::fs::File::open(atlas("server_sensor_text.csv.gz")).unwrap())
        .read_to_string(&mut gz)
        .unwrap();
    let sensor = rows(&gz);

    let (doc_per, doc_bytes) = check(&docs, StorageClass::Document);
    let (_, sensor_bytes) = check(&sensor, StorageClass::SensorText);

    // Totals of SERVER_STORAGE.md ("In one table"), MB = 10^6 bytes as the atlas rounds them.
    assert_eq!(docs.len(), 636);
    assert_eq!(sensor.len(), 24_482);
    assert_eq!((doc_bytes as f64 / 1e6).round() as u64, 1630);
    assert_eq!((sensor_bytes as f64 / 1e6).round() as u64, 12_256);

    // Per-investigation document counts of the atlas table, and the same 19 investigations.
    let want: BTreeMap<&str, usize> = [
        ("PSI-69", 107),
        ("PSI-68", 12),
        ("PSI-39", 40),
        ("PSI-159", 4),
        ("PSI-10", 9),
        ("PSI-23", 39),
        ("PSI-20", 42),
        ("PSI-21", 1),
        ("PSI-22", 58),
        ("PSI-107", 15),
        ("PSI-106", 1),
        ("PSI-25", 8),
        ("PSI-26", 38),
        ("PSI-98", 7),
        ("PSI-99", 5),
        ("PSI-100", 11),
        ("PSI-102", 95),
        ("PSI-101", 119),
        ("PSI-47", 25),
    ]
    .into_iter()
    .collect();
    assert_eq!(doc_per, want);
    let registry: BTreeSet<&str> = FLIGHTS.iter().map(|f| f.accession).collect();
    assert_eq!(registry, want.keys().copied().collect());
}

#[test]
fn media_and_archives_stay_at_nasa() {
    for (category, name) in [
        ("Raw Data", "PSI-26_Raw Data_frame_0001.tif"),
        ("Raw Data", "PSI-159_Raw Data_run12.ts"),
        ("Raw Data", "PSI-25_Raw Data_images.zip"),
        ("Presentations", "talk.mp4"),
        ("Calibration", "PSI-25_Calibration_images.zip"),
        ("Some Future Category", "notes.pdf"),
    ] {
        assert_eq!(
            storage_class(category, name),
            StorageClass::ManifestOnly,
            "{category} / {name}"
        );
    }
    assert_eq!(
        storage_class("Reports", "PSI-69_Reports_TP.PDF"),
        StorageClass::Document
    );
}

#[test]
fn the_registry_flags_what_needs_care() {
    // PSI-10's licence is blank in NASA's metadata: it must be verified, not assumed CC0.
    assert_eq!(flight("PSI-10").unwrap().licence, None);
    // Documents of these four hold outcomes of held-out tests (atlas, "What the server has to take care of").
    let lockbox: BTreeSet<&str> = FLIGHTS
        .iter()
        .filter(|f| f.lockbox)
        .map(|f| f.accession)
        .collect();
    assert_eq!(
        lockbox,
        ["PSI-69", "PSI-159", "PSI-25", "PSI-26"]
            .into_iter()
            .collect()
    );
}

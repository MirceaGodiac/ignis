//! FLEX processed series: the sample set takes every series NASA lists, and a series file the
//! ingest cannot read becomes an explicit gap of its run, not a failed ingest.
//!
//! Own test binary: `IGNIS_DATA` is process-global and this root holds deliberately broken files.

use std::collections::BTreeMap;
use std::path::Path;

use ignis_core::contracts::Quantity;
use ignis_data::families::flex;
use ignis_data::sources::{psi, samples};

fn repo() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn every_listed_flex_series_is_in_the_sample_set() {
    let files = psi::read_listing(&repo().join("data/manifests/PSI-69.listing.jsonl")).unwrap();
    let sheet = regex::Regex::new(r"^PSI-69_Raw Data_FLEX-(\d+)_TestSheet\.txt$").unwrap();
    let series =
        regex::Regex::new(r"^PSI-69_Raw Data_FLEX-\d+_(DropletData|ColorCamData|LLUVData)\.txt$")
            .unwrap();
    // CIR ids come from fetched test sheets; without them the series are still selected.
    let per_run: BTreeMap<String, String> = files
        .iter()
        .filter_map(|f| {
            sheet
                .captures(&f.file_name)
                .map(|c| (c[1].to_string(), String::new()))
        })
        .collect();
    let got = samples::flex_stage2(&files, &per_run);
    let listed: std::collections::BTreeSet<&str> = files
        .iter()
        .filter(|f| series.is_match(&f.file_name))
        .map(|f| f.file_name.as_str())
        .collect();
    let picked: std::collections::BTreeSet<&str> =
        got.iter().map(|f| f.file_name.as_str()).collect();
    assert!(
        listed.len() > 700,
        "listing holds {} series files",
        listed.len()
    );
    assert_eq!(picked, listed, "every listed series file, nothing else");
    assert_eq!(got.len(), picked.len(), "one entry per file");
    // All pass the per-file cap, so none is silently skipped by `capped`.
    let (ok, skipped) = samples::capped(got);
    assert!(skipped.is_empty() && ok.len() == listed.len());
}

fn record(template: &serde_json::Value, name: &str, path: &Path) -> String {
    let mut v = template.clone();
    v["id"] = format!("sha256:{}", ignis_core::hashing::file_sha256(path).unwrap()).into();
    v["file_name"] = name.into();
    v["size_bytes"] = std::fs::metadata(path).unwrap().len().into();
    serde_json::to_string(&v).unwrap()
}

#[test]
fn unreadable_series_files_are_gaps() {
    let root = std::env::temp_dir().join(format!("ignis-flex-series-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let raw = root.join("raw/samples/PSI-69");
    std::fs::create_dir_all(&raw).unwrap();
    std::fs::create_dir_all(root.join("manifests")).unwrap();
    let fix = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/PSI-69");
    let man = repo().join("data/manifests");
    let committed = std::fs::read_to_string(man.join("PSI-69.jsonl")).unwrap();
    let mut lines = vec![];
    let mut template = None;
    for e in std::fs::read_dir(&fix).unwrap().flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        std::fs::copy(e.path(), raw.join(&name)).unwrap();
        let l = committed
            .lines()
            .find(|l| l.contains(&format!("\"file_name\":\"{name}\"")))
            .unwrap();
        template.get_or_insert_with(|| serde_json::from_str::<serde_json::Value>(l).unwrap());
        lines.push(l.to_string());
    }
    // Test-local stand-ins (not NASA bytes): a spreadsheet saved under .txt and a header-only file.
    let colour = "PSI-69_Raw Data_FLEX-020_ColorCamData.txt";
    std::fs::write(raw.join(colour), b"PK\x03\x04\x14\x00\x06\x00binary").unwrap();
    let uv = "PSI-69_Raw Data_FLEX-020_LLUVData.txt";
    std::fs::write(
        raw.join(uv),
        "LLUVTime_sec\tLLUVFlameDiameter_mm\r\r\t3.1\r",
    )
    .unwrap();
    for n in [colour, uv] {
        lines.push(record(template.as_ref().unwrap(), n, &raw.join(n)));
    }
    std::fs::write(root.join("manifests/PSI-69.jsonl"), lines.join("\n") + "\n").unwrap();
    std::env::set_var("IGNIS_DATA", &root);

    let b = flex::ingest().expect("broken series files must not fail the ingest");
    let run = "PSI-69:192F001:2009-10-24";
    assert!(
        b.observations
            .iter()
            .filter(|o| o.run_id == run && o.quantity == Quantity::LiquidDiameter)
            .count()
            > 50
    );
    assert!(!b
        .observations
        .iter()
        .any(|o| o.quantity != Quantity::LiquidDiameter));
    let errs = &b.extra["series_errors"];
    let files: Vec<&str> = errs.iter().map(|e| e["file"].as_str().unwrap()).collect();
    assert_eq!(files, [colour, uv]);
    assert!(errs
        .iter()
        .all(|e| e["run_id"] == run && e["reason"].is_string()));
    let _ = std::fs::remove_dir_all(&root);
}

//! Bounded sample selection per investigation (no bulk ingestion: no video, images or archives).
//!
//! The selection is a deterministic function of the remote listing (and, for FLEX series, of the
//! already-fetched test sheets), so `ignis sources samples` reproduces the same working sample.
//! FLEX takes every run's processed series (the models read them); ACME and BASS a representative set.

use regex::Regex;

use super::psi::RemoteFile;

/// Per-file cap for samples; anything larger needs the guarded full fetch.
pub const MAX_FILE_BYTES: i64 = 12_000_000;

fn tables() -> Regex {
    Regex::new(r"_(Experimental table|metadata)_").unwrap()
}

fn named(files: &[RemoteFile], pred: impl Fn(&str) -> bool) -> Vec<RemoteFile> {
    files
        .iter()
        .filter(|f| pred(&f.file_name))
        .cloned()
        .collect()
}

/// Summary table, ISA metadata and every FLEX test sheet (text form, ~1 kB each).
pub fn flex_stage1(files: &[RemoteFile]) -> Vec<RemoteFile> {
    let t = tables();
    named(files, |n| t.is_match(n) || n.ends_with("_TestSheet.txt"))
}

/// NASA's processed series of a FLEX run, as `families::flex::ingest` reads them: liquid diameter
/// (`DropletData`), visible and UV flame diameter (`ColorCamData`, `LLUVData`). Text, < 0.3 MB each.
pub const FLEX_SERIES: [&str; 3] = ["DropletData", "ColorCamData", "LLUVData"];

/// Every published processed series of every FLEX run with a test sheet (`per_run_cir`: file
/// number -> CIR test point), plus the run's CIR test-point logs (record, comments, MDCA settings;
/// the MDCA fiber flag enters the free-floating screen). No run subset: a run whose series NASA
/// does not publish simply has none here.
pub fn flex_stage2(
    files: &[RemoteFile],
    per_run_cir: &std::collections::BTreeMap<String, String>,
) -> Vec<RemoteFile> {
    let mut want = std::collections::BTreeSet::new();
    for (num, cir) in per_run_cir {
        for kind in FLEX_SERIES {
            want.insert(format!("PSI-69_Raw Data_FLEX-{num}_{kind}.txt"));
        }
        if !cir.is_empty() {
            want.insert(format!("PSI-69_Raw Data_{cir}.txt"));
            want.insert(format!("PSI-69_Raw Data_{cir}_Comments.txt"));
            want.insert(format!("PSI-69_Raw Data_{cir}_MDCA_Testpoint_Settings.txt"));
        }
    }
    // NASA's listing repeats a few rows: one entry per (name, version).
    let mut seen = std::collections::BTreeSet::new();
    named(files, |n| want.contains(n))
        .into_iter()
        .filter(|f| seen.insert((f.file_name.clone(), f.file_version)))
        .collect()
}

pub const ACME_TESTS: [&str; 7] = [
    "21088A1", "21088B1", "21088C1", "21088D1", "21196A2", "21196F1", "21196J1",
];

/// Both tables, the chamber calculator and per-test logs (data/TP/setpoints/comments) for 7 tests.
pub fn acme(files: &[RemoteFile]) -> Vec<RemoteFile> {
    let t = tables();
    let mut keep = Vec::new();
    for f in files {
        let n = f.file_name.as_str();
        if t.is_match(n) || n.contains("Final ACME Chamber Calculator") {
            keep.push(f.clone());
            continue;
        }
        for t in ACME_TESTS {
            let names = [
                format!("PSI-159_Raw Data_{t}_data.csv"),
                format!("PSI-159_Raw Data_{t}_TP.txt"),
                format!("PSI-159_Raw Data_{t}_mfcSetpoints.csv"),
                format!("PSI-159_Raw Data_{t}_Comments.txt"),
                format!("PSI-159_Raw Data_{t}_ImageComments.txt"),
                format!("PSI-159_Raw Data_{t}_ImagesComments.txt"),
                format!("PSI-159_Raw Data_as run{t}.txt"),
            ];
            if names.iter().any(|x| x == n) {
                keep.push(f.clone());
            }
        }
    }
    keep
}

/// Master tables, test matrices, read-me and fan calibration. Videos are listed, never sampled here.
pub fn bass(files: &[RemoteFile]) -> Vec<RemoteFile> {
    let t = tables();
    const KEYS: [&str; 5] = [
        "Test.Matrices.xlsx",
        "BASS-II_ReadMe.docx",
        "Fan calibration data.xlsx",
        "As Run_Test Matrix.xlsx",
        "Planned_Test Matrix.xlsx",
    ];
    named(files, |n| {
        t.is_match(n) || KEYS.iter().any(|k| n.contains(k))
    })
}

pub fn capped(files: Vec<RemoteFile>) -> (Vec<RemoteFile>, Vec<RemoteFile>) {
    files
        .into_iter()
        .partition(|f| !f.restricted && f.size_bytes.unwrap_or(0) <= MAX_FILE_BYTES)
}

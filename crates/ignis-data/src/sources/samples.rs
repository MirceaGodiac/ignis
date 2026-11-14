//! Bounded, representative sample selection per investigation (no bulk ingestion).
//!
//! The selection is a deterministic function of the remote listing (and, for FLEX series, of the
//! already-fetched test sheets), so `ignis sources samples` reproduces the same working sample.

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

/// DropletData + visible/UV channels for chosen methanol runs, plus the CIR test-point logs.
pub fn flex_stage2(
    files: &[RemoteFile],
    numbers: &[String],
    per_run_cir: &std::collections::BTreeMap<String, String>,
) -> Vec<RemoteFile> {
    let mut out = Vec::new();
    for num in numbers {
        let rx = Regex::new(&format!(
            r"^PSI-69_Raw Data_FLEX-{}_(DropletData|ColorCamData|LLUVData)\.txt$",
            regex::escape(num)
        ))
        .unwrap();
        out.extend(named(files, |n| rx.is_match(n)));
        if let Some(cir) = per_run_cir.get(num).filter(|c| !c.is_empty()) {
            let want = [
                format!("PSI-69_Raw Data_{cir}.txt"),
                format!("PSI-69_Raw Data_{cir}_Comments.txt"),
                format!("PSI-69_Raw Data_{cir}_MDCA_Testpoint_Settings.txt"),
            ];
            out.extend(named(files, |n| want.iter().any(|w| w == n)));
        }
    }
    out
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

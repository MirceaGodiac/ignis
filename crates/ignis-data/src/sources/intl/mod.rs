//! International (non-NASA) microgravity-combustion sources, kept as SEPARATE investigations.
//!
//! Nothing here enters NASA FLEX/ACME/BASS cohorts: every record carries `Family::Other` and its own
//! investigation id. Manifests live in `data/manifests/intl/`; raw probe files in `data/raw/intl/` (ignored).
//!
//! ```text
//! JAXA-GCEM            JAXA DARTS, ISS Kibo Group Combustion (darts.rs)
//! ZARM-FLARE           ZARM/DLR FLARE-G II/III IR, Zenodo 18479713 = register D20 (zenodo.rs)
//! ESA-HREDA-*          ESA HREDA, login-gated: metadata only (hreda.rs)
//! MENDELEY-2GW4PJHKXZ  Rojas-Alva & Jomaas evidence index (mendeley.rs)
//! ```

pub mod common;
pub mod darts;
pub mod hreda;
pub mod mendeley;
pub mod zenodo;

/// Registered investigations, in listing order.
pub fn investigations() -> Vec<(String, String)> {
    let mut v = vec![
        (
            darts::INVESTIGATION_ID.to_string(),
            "JAXA DARTS - ISS Kibo Group Combustion (GCEM)".to_string(),
        ),
        (
            zenodo::INVESTIGATION_ID.into(),
            "ZARM/DLR FLARE-G II/III IR drop-tower data (Zenodo 18479713, register D20)".into(),
        ),
    ];
    for (k, _, doi) in hreda::TARGETS {
        v.push((
            k.to_string(),
            format!("ESA HREDA {doi} (login required; metadata only)"),
        ));
    }
    v.push((
        mendeley::INVESTIGATION_ID.into(),
        "Rojas-Alva & Jomaas solid-combustion microgravity evidence index".into(),
    ));
    v
}

/// Default probe selection for `fetch` (globs over manifest file_name) and per-investigation byte caps.
pub fn probe(inv: &str) -> Option<(&'static [&'static str], u64)> {
    match inv {
        darts::INVESTIGATION_ID => Some((
            &[
                "documents/*.tsv",
                "documents/gcem_*.pdf",
                "documents/gc_parameter.pdf",
                "documents/gc_experiment_overview.pdf",
                "telemetry/170216-01_2-1.zip",
                "telemetry/170217-01_1-52.zip",
            ],
            30_000_000,
        )),
        zenodo::INVESTIGATION_ID => Some((&["index.csv"], 50_000_000)),
        mendeley::INVESTIGATION_ID => Some((&["*.pdf"], 10_000_000)),
        _ => None,
    }
}

/// Python `fnmatch.fnmatch` (`*`, `?`, `[...]`; case-insensitive on Windows like `os.path.normcase`).
pub fn fnmatch(name: &str, pat: &str) -> bool {
    let mut rx = String::from("(?s");
    if cfg!(windows) {
        rx.push('i');
    }
    rx.push_str(")^");
    let mut chars = pat.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' => rx.push_str(".*"),
            '?' => rx.push('.'),
            '[' => {
                let mut cls = String::new();
                let mut closed = false;
                for d in chars.by_ref() {
                    if d == ']' && !cls.is_empty() {
                        closed = true;
                        break;
                    }
                    cls.push(d);
                }
                if closed {
                    let cls = cls
                        .strip_prefix('!')
                        .map(|c| format!("^{c}"))
                        .unwrap_or(cls);
                    rx.push_str(&format!("[{}]", cls.replace('\\', "\\\\")));
                } else {
                    rx.push_str(&regex::escape(&format!("[{cls}")));
                }
            }
            c => rx.push_str(&regex::escape(&c.to_string())),
        }
    }
    rx.push('$');
    let name = if cfg!(windows) {
        name.replace('/', "\\")
    } else {
        name.to_string()
    };
    let rx = if cfg!(windows) {
        rx.replace('/', "\\\\")
    } else {
        rx
    };
    regex::Regex::new(&rx)
        .map(|r| r.is_match(&name))
        .unwrap_or(false)
}

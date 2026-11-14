//! Data directory layout. Everything is relative to `IGNIS_DATA` (default `data`).
//!
//! ```text
//! raw/samples/<investigation>/   bytes fetched from NASA, never modified (gitignored)
//! raw/samples/_meta/             investigation metadata and listing responses (gitignored)
//! raw/intl/<investigation>/      international probe files (gitignored)
//! manifests/<investigation>.jsonl          Source records of fetched assets (committed)
//! manifests/<investigation>.listing.jsonl  remote inventory, no bytes (committed)
//! manifests/intl/<investigation>.jsonl     international manifests (committed)
//! review/<family>/               semantic review items and adjudications (committed)
//! splits/<family>-<hex16>.json   frozen splits (committed)
//! work/                          derived, rebuildable (records, cohorts, views; gitignored)
//! ```

use std::path::PathBuf;

pub fn root() -> PathBuf {
    std::env::var_os("IGNIS_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
}

pub fn raw(investigation: Option<&str>) -> PathBuf {
    let p = root().join("raw").join("samples");
    match investigation {
        Some(i) => p.join(i),
        None => p,
    }
}

pub fn meta() -> PathBuf {
    raw(None).join("_meta")
}

pub fn manifests() -> PathBuf {
    root().join("manifests")
}

pub fn review(family: &str) -> PathBuf {
    root().join("review").join(family)
}

pub fn splits() -> PathBuf {
    root().join("splits")
}

pub fn work(parts: &[&str]) -> PathBuf {
    let mut p = root().join("work");
    for x in parts {
        p = p.join(x);
    }
    p
}

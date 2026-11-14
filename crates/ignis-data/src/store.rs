//! Persist / reload a family [`Bundle`] under `data/work/records/<family>/` (rebuildable).
//!
//! File names are shared with `ignis-server`'s registry loader (`experiments.jsonl`,
//! `observations.jsonl`, `endpoints.jsonl`, ...): one contract record per line.

use ignis_core::contracts::Measured;
use indexmap::IndexMap;
use serde_json::{json, Value};

use crate::common::{read_jsonl, read_jsonl_values, write_jsonl, Bundle, ReviewItem};
use crate::paths;

pub fn save(b: &Bundle) -> anyhow::Result<IndexMap<String, usize>> {
    let d = paths::work(&["records", &b.family]);
    let mut n = IndexMap::new();
    n.insert(
        "records".into(),
        write_jsonl(&d.join("experiments.jsonl"), &b.records)?,
    );
    n.insert(
        "sessions".into(),
        write_jsonl(&d.join("sessions.jsonl"), b.sessions.values())?,
    );
    n.insert(
        "specimens".into(),
        write_jsonl(&d.join("specimens.jsonl"), b.specimens.values())?,
    );
    n.insert(
        "observations".into(),
        write_jsonl(&d.join("observations.jsonl"), &b.observations)?,
    );
    n.insert(
        "endpoints".into(),
        write_jsonl(&d.join("endpoints.jsonl"), &b.endpoints)?,
    );
    n.insert(
        "post".into(),
        write_jsonl(
            &d.join("post.jsonl"),
            b.post.iter().map(|(k, v)| json!({"run_id": k, "facts": v})),
        )?,
    );
    n.insert(
        "review".into(),
        write_jsonl(
            &d.join("review.jsonl"),
            b.review.iter().map(|r| r.to_json()),
        )?,
    );
    n.insert(
        "exclusions".into(),
        write_jsonl(&d.join("exclusions.jsonl"), &b.exclusions)?,
    );
    // Stale extras from a previous ingest must not survive.
    if let Ok(rd) = std::fs::read_dir(&d) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("extra_") && name.ends_with(".jsonl") {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    for (k, rows) in &b.extra {
        n.insert(
            k.clone(),
            write_jsonl(&d.join(format!("extra_{k}.jsonl")), rows)?,
        );
    }
    Ok(n)
}

pub fn load(family: &str) -> anyhow::Result<Bundle> {
    let d = paths::work(&["records", family]);
    anyhow::ensure!(
        d.join("experiments.jsonl").exists(),
        "no ingested records for {family}; run `ignis data ingest {family}`"
    );
    let mut b = Bundle::new(family);
    b.records = read_jsonl(&d.join("experiments.jsonl"))?;
    b.sessions = read_jsonl::<ignis_core::contracts::Session>(&d.join("sessions.jsonl"))?
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect();
    b.specimens = read_jsonl::<ignis_core::contracts::Specimen>(&d.join("specimens.jsonl"))?
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect();
    b.observations = read_jsonl(&d.join("observations.jsonl"))?;
    b.endpoints = read_jsonl(&d.join("endpoints.jsonl"))?;
    for r in read_jsonl_values(&d.join("post.jsonl"))? {
        let facts: IndexMap<String, Measured> = serde_json::from_value(r["facts"].clone())?;
        b.post
            .insert(r["run_id"].as_str().unwrap_or_default().to_string(), facts);
    }
    b.review = read_jsonl_values(&d.join("review.jsonl"))?
        .iter()
        .map(ReviewItem::from_json)
        .collect::<anyhow::Result<_>>()?;
    b.exclusions = read_jsonl(&d.join("exclusions.jsonl"))?;
    let mut extras: Vec<_> = std::fs::read_dir(&d)?.flatten().map(|e| e.path()).collect();
    extras.sort();
    for p in extras {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if let Some(k) = name
            .strip_prefix("extra_")
            .and_then(|s| s.strip_suffix(".jsonl"))
        {
            let rows: Vec<Value> = read_jsonl_values(&p)?;
            b.extra.insert(k.to_string(), rows);
        }
    }
    Ok(b)
}

/// Python `repr(dict[str, int])` for CLI summaries.
pub fn counts_repr(n: &IndexMap<String, usize>) -> String {
    let parts: Vec<String> = n.iter().map(|(k, v)| format!("'{k}': {v}")).collect();
    format!("{{{}}}", parts.join(", "))
}

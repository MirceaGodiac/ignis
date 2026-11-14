//! Cohort building (declared criteria -> run ids + exclusions) and grouped, frozen splits.
//!
//! Identities are content hashes (`ignis_core::hashing::content_hash`) of exactly the same inputs
//! the Python pipeline hashed, and the group shuffle is a bit-exact port of numpy's
//! `default_rng(seed).permutation` ([`crate::npyrng`]), so cohorts and splits frozen before the
//! port keep their ids.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use anyhow::Context;
use ignis_core::contracts::{Cohort, Exclusion, Split};
use ignis_core::hashing::{content_hash, hex_part};
use serde::Serialize;
use serde_json::json;

use crate::common::Bundle;
use crate::families::{family_of, get};
use crate::sources::manifest::manifest_hash;
use crate::{paths, store};

pub const COHORT_VERSION: &str = "1";
pub const SPLIT_RULE: &str =
    "60/20/20 of groups (round half to even), groups shuffled by numpy default_rng(seed)";

pub fn build(family: &str) -> anyhow::Result<(Cohort, PathBuf)> {
    let spec = get(family_of(family)?)?;
    let b = store::load(family)?;
    let (mut keep, mut out) = (spec.cohort_filter)(&b);
    keep.sort();
    out.sort();
    let mut criteria: BTreeMap<String, String> = spec
        .criteria
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    criteria.extend(crate::review::adjudication_criteria(&b));
    let smh = manifest_hash(spec.investigations)?;
    let id = content_hash(
        &json!({"family": spec.family, "criteria": criteria, "run_ids": keep, "source_manifest_hash": smh}),
        &["id"],
    );
    let coh = Cohort {
        id,
        family: spec.family,
        version: COHORT_VERSION.into(),
        criteria,
        run_ids: keep,
        excluded: out
            .into_iter()
            .map(|(run_id, reason)| Exclusion { run_id, reason })
            .collect(),
        source_manifest_hash: smh,
    };
    let p = cohort_path(&coh.id);
    std::fs::create_dir_all(p.parent().unwrap())?;
    std::fs::write(&p, serde_json::to_string_pretty(&coh)?)?;
    Ok((coh, p))
}

pub fn cohort_path(cohort_id: &str) -> PathBuf {
    paths::work(&["cohorts", &format!("{}.json", hex_part(cohort_id))])
}

fn split_files() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(paths::splits())
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    v.sort();
    v
}

pub fn load_cohort(cohort_id: &str) -> anyhow::Result<Cohort> {
    let p = cohort_path(cohort_id);
    if p.exists() {
        return Ok(serde_json::from_str(&std::fs::read_to_string(&p)?)?);
    }
    for sp in split_files() {
        // frozen splits embed their cohort
        let d: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&sp)?)?;
        let id = d["cohort"]["id"].as_str().unwrap_or_default();
        if id == cohort_id || id.ends_with(cohort_id) {
            return Ok(serde_json::from_value(d["cohort"].clone())?);
        }
    }
    anyhow::bail!("cohort {cohort_id} not found; run `ignis data cohort build`")
}

/// Union-find over the grouping keys: runs that share a session or a reused specimen share a group.
pub fn groups(b: &Bundle, run_ids: &[String], keys: &[&str]) -> BTreeMap<String, String> {
    let recs: HashMap<&str, &ignis_core::contracts::ExperimentRecord> =
        b.records.iter().map(|r| (r.id.as_str(), r)).collect();
    let mut parent: HashMap<String, String> = HashMap::new();
    fn find(parent: &mut HashMap<String, String>, x: &str) -> String {
        let mut x = x.to_string();
        loop {
            let p = parent.entry(x.clone()).or_insert_with(|| x.clone()).clone();
            if p == x {
                return x;
            }
            let gp = parent.get(&p).cloned().unwrap_or_else(|| p.clone());
            parent.insert(x.clone(), gp.clone());
            x = gp;
        }
    }
    fn union(parent: &mut HashMap<String, String>, a: &str, c: &str) {
        let (ra, rc) = (find(parent, a), find(parent, c));
        if ra != rc {
            let (hi, lo) = if ra > rc { (ra, rc) } else { (rc, ra) };
            parent.insert(hi, lo);
        }
    }
    for rid in run_ids {
        let r = recs[rid.as_str()];
        let node = format!("run:{rid}");
        find(&mut parent, &node);
        if keys.contains(&"session") {
            if let Some(s) = &r.session_id {
                union(&mut parent, &node, &format!("session:{s}"));
            }
        }
        if keys.contains(&"specimen_reuse_group") {
            if let Some(sid) = &r.specimen_id {
                let g = b
                    .specimens
                    .get(sid)
                    .and_then(|sp| sp.reuse_group.clone())
                    .unwrap_or_else(|| sid.clone());
                union(&mut parent, &node, &format!("specimen:{g}"));
            }
        }
    }
    let nodes: Vec<String> = parent.keys().cloned().collect();
    let mut members: HashMap<String, BTreeSet<String>> = HashMap::new();
    for k in nodes {
        let root = find(&mut parent, &k);
        if !k.starts_with("run:") {
            members.entry(root).or_default().insert(k);
        }
    }
    let mut out = BTreeMap::new();
    let mut first_run: HashMap<String, String> = HashMap::new();
    for rid in run_ids {
        let root = find(&mut parent, &format!("run:{rid}"));
        first_run.entry(root.clone()).or_insert_with(|| rid.clone());
        let gid = match members.get(&root) {
            Some(m) if !m.is_empty() => {
                let first = m.iter().next().unwrap().clone();
                if m.len() > 1 {
                    format!("{first}+{}", m.len() - 1)
                } else {
                    first
                }
            }
            _ => format!("run:{}", first_run[&root]),
        };
        out.insert(rid.clone(), gid);
    }
    out
}

#[derive(Serialize)]
struct Frozen<'a> {
    split: &'a Split,
    cohort: &'a Cohort,
    content_hash: String,
}

pub fn now_micros() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

/// Freeze a 60/20/20 grouped split of a cohort (refuses to refreeze an existing id).
pub fn freeze(cohort_id: &str, seed: i64) -> anyhow::Result<(Split, PathBuf)> {
    let coh = load_cohort(cohort_id)?;
    let spec = get(coh.family)?;
    let b = store::load(coh.family.as_str())?;
    let r2g = groups(&b, &coh.run_ids, spec.group_keys);
    let gids: Vec<String> = r2g
        .values()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let n = gids.len();
    anyhow::ensure!(
        n >= 3,
        "cohort has {n} groups; a grouped 3-way split needs at least 3"
    );
    anyhow::ensure!(seed >= 0, "seed must be non-negative (numpy SeedSequence)");
    let order: Vec<String> = crate::npyrng::permutation(seed as u64, n)
        .into_iter()
        .map(|i| gids[i].clone())
        .collect();
    let mut n_tr = ((0.6 * n as f64).round_ties_even() as usize).max(1);
    let n_va = ((0.2 * n as f64).round_ties_even() as usize).max(1);
    if n_tr + n_va >= n {
        n_tr = n - n_va - 1;
    }
    let sorted = |s: &[String]| {
        let mut v = s.to_vec();
        v.sort();
        v
    };
    let (train, val, test) = (
        sorted(&order[..n_tr]),
        sorted(&order[n_tr..n_tr + n_va]),
        sorted(&order[n_tr + n_va..]),
    );
    let key = spec.group_keys.join("+");
    let id = content_hash(
        &json!({"cohort_id": coh.id, "group_key": key, "rule": SPLIT_RULE, "seed": seed, "run_to_group": r2g,
                "train": train, "validation": val, "test": test}),
        &["id"],
    );
    let split = Split {
        id: id.clone(),
        cohort_id: coh.id.clone(),
        group_key: key,
        rule: SPLIT_RULE.into(),
        seed,
        run_to_group: r2g,
        train,
        validation: val,
        test,
        frozen_at: now_micros(),
    };
    let p = paths::splits().join(format!(
        "{}-{}.json",
        coh.family.as_str(),
        &hex_part(&id)[..16]
    ));
    anyhow::ensure!(
        !p.exists(),
        "split {id} is already frozen at {}; refusing to refreeze",
        p.display()
    );
    let ch = content_hash(&json!({"split": split, "cohort": coh}), &[]);
    std::fs::create_dir_all(paths::splits())?;
    std::fs::write(
        &p,
        serde_json::to_string_pretty(&Frozen {
            split: &split,
            cohort: &coh,
            content_hash: ch,
        })? + "\n",
    )?;
    Ok((split, p))
}

/// Find a frozen split file by id (any prefix of the hex) or path.
pub fn split_path(split_id_or_path: &str) -> anyhow::Result<PathBuf> {
    let p = Path::new(split_id_or_path);
    if p.exists() {
        return Ok(p.to_path_buf());
    }
    let h = hex_part(split_id_or_path);
    let h = &h[..h.len().min(16)];
    split_files()
        .into_iter()
        .find(|f| {
            f.file_stem()
                .is_some_and(|s| s.to_string_lossy().ends_with(&format!("-{h}")))
        })
        .with_context(|| {
            format!(
                "split {split_id_or_path} not found under {}",
                paths::splits().display()
            )
        })
}

/// Load and verify a frozen split (its content hash must match).
pub fn load_split_with_cohort(split_id_or_path: &str) -> anyhow::Result<(Split, Cohort)> {
    let p = split_path(split_id_or_path)?;
    let d: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p)?)?;
    let body = json!({"split": d["split"], "cohort": d["cohort"]});
    anyhow::ensure!(
        content_hash(&body, &[]) == d["content_hash"].as_str().unwrap_or_default(),
        "{}: content hash mismatch (frozen split was modified)",
        p.display()
    );
    Ok((
        serde_json::from_value(d["split"].clone())?,
        serde_json::from_value(d["cohort"].clone())?,
    ))
}

pub fn load_split(split_id_or_path: &str) -> anyhow::Result<Split> {
    Ok(load_split_with_cohort(split_id_or_path)?.0)
}

//! Score several releases on exactly the same admissible development cases.
//!
//! Cases are (run, prefix) pairs with a sealed artifact from every compared release. The default
//! partition is validation; test is refused unless `final_test`. Releases must share family and
//! split; feature order, units and cohort must match unless explicitly allowed (and reported).

use std::collections::{BTreeMap, BTreeSet};

use ignis_core::contracts::ModelRelease;
use ignis_core::registry::load_release;
use serde_json::{json, Map, Value};

use super::bootstrap::cluster_bootstrap;
use super::scoring::{aggregate, common_spec, public_case, score_all, Case};
use super::{vault, VaultAccess};
use crate::util::{artifacts_root, refused};

#[derive(Debug, Clone, Default)]
pub struct Allow {
    pub feature_mismatch: bool,
    pub unit_mismatch: bool,
    pub cohort_mismatch: bool,
}

/// Refuse incomparable releases; return the list of explicitly allowed mismatches.
pub fn check_compatible(releases: &[ModelRelease], allow: &Allow) -> anyhow::Result<Vec<String>> {
    let r0 = &releases[0];
    let mut allowed = vec![];
    for r in &releases[1..] {
        if r.family != r0.family {
            return Err(refused(format!(
                "family differs: {}={}, {}={}",
                r0.id,
                r0.family.as_str(),
                r.id,
                r.family.as_str()
            )));
        }
        if r.split_id != r0.split_id {
            return Err(refused(format!(
                "split differs: {} vs {}; cases would not be the same",
                r0.split_id, r.split_id
            )));
        }
        if r.feature_order != r0.feature_order {
            let msg = format!(
                "feature order differs: {:?} vs {:?} ({})",
                r0.feature_order, r.feature_order, r.id
            );
            if !allow.feature_mismatch {
                return Err(refused(format!(
                    "{msg}; pass --allow-feature-mismatch to compare anyway"
                )));
            }
            allowed.push(msg);
        }
        if r.units != r0.units {
            let msg = format!("units differ: {:?} vs {:?} ({})", r0.units, r.units, r.id);
            if !allow.unit_mismatch {
                return Err(refused(format!(
                    "{msg}; pass --allow-unit-mismatch to compare anyway"
                )));
            }
            allowed.push(msg);
        }
        if r.cohort_id != r0.cohort_id {
            let msg = format!(
                "cohort differs: {} vs {} ({})",
                r0.cohort_id, r.cohort_id, r.id
            );
            if !allow.cohort_mismatch {
                return Err(refused(format!(
                    "{msg}; pass --allow-cohort-mismatch to compare anyway"
                )));
            }
            allowed.push(msg);
        }
    }
    Ok(allowed)
}

pub struct CompareOpts<'a> {
    pub partition: &'a str,
    pub final_test: bool,
    pub allow: Allow,
    pub intersect: bool,
    pub horizons: Option<Vec<f64>>,
    pub n_boot: usize,
    pub seed: u64,
}

pub fn compare(
    access: &VaultAccess,
    release_ids: &[String],
    o: &CompareOpts<'_>,
) -> anyhow::Result<Value> {
    if release_ids.len() < 2 {
        return Err(refused("compare needs at least two releases"));
    }
    if !matches!(o.partition, "validation" | "test") {
        return Err(refused(
            "compare scores held-out partitions only (validation, or test with --final-test)",
        ));
    }
    if o.partition == "test" && !o.final_test {
        return Err(refused(
            "comparing on the test partition requires --final-test",
        ));
    }
    let root = artifacts_root();
    let releases: Vec<ModelRelease> = release_ids
        .iter()
        .map(|r| load_release(&root, r, None).map(|x| x.0))
        .collect::<anyhow::Result<_>>()?;
    let ids: Vec<String> = releases.iter().map(|r| r.id.clone()).collect();
    if ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err(refused("the same release was given twice"));
    }
    let allowed = check_compatible(&releases, &o.allow)?;

    let mut table: BTreeMap<(String, String), BTreeMap<String, Vec<String>>> = BTreeMap::new();
    for s in vault::list_seals() {
        let rid = s["model_release_id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        if ids.contains(&rid) && s["sealed"]["split_partition"].as_str() == Some(o.partition) {
            let key = (
                s["run_id"].as_str().unwrap_or_default().to_string(),
                s["prefix_hash"].as_str().unwrap_or_default().to_string(),
            );
            table.entry(key).or_default().entry(rid).or_default().push(
                s["sealed"]["artifact_id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            );
        }
    }
    let dup: Vec<&(String, String)> = table
        .iter()
        .filter(|(_, v)| v.values().any(|a| a.len() > 1))
        .map(|(k, _)| k)
        .collect();
    if let Some(d) = dup.first() {
        return Err(refused(format!(
            "{} case(s) have several sealed artifacts from one release (e.g. {d:?}); compare one artifact per release and case",
            dup.len()
        )));
    }
    let idset: BTreeSet<&String> = ids.iter().collect();
    let incomplete: BTreeMap<(String, String), Vec<String>> = table
        .iter()
        .filter(|(_, v)| v.keys().collect::<BTreeSet<_>>() != idset)
        .map(|(k, v)| {
            (
                k.clone(),
                ids.iter()
                    .filter(|i| !v.contains_key(*i))
                    .cloned()
                    .collect(),
            )
        })
        .collect();
    if !incomplete.is_empty() && !o.intersect {
        let (k, miss) = incomplete.iter().next().unwrap();
        return Err(refused(format!(
            "{} case(s) are not sealed by every release (e.g. run {} missing {miss:?}); seal them or pass --intersect to restrict to common cases",
            incomplete.len(),
            k.0
        )));
    }
    let keys: Vec<(String, String)> = table
        .keys()
        .filter(|k| !incomplete.contains_key(*k))
        .cloned()
        .collect();
    if keys.is_empty() {
        return Err(refused(format!(
            "no {} case is sealed by all releases",
            o.partition
        )));
    }

    let mut arts = BTreeMap::new();
    for k in &keys {
        for rid in &ids {
            arts.insert(
                (k.clone(), rid.clone()),
                vault::sealed_artifact(&table[k][rid][0])?,
            );
        }
    }
    let spec = common_spec(&arts.values().collect::<Vec<_>>(), o.horizons.as_deref());
    let mut per_release = Map::new();
    let mut case_rows: BTreeMap<String, Vec<Case>> = BTreeMap::new();
    for rid in &ids {
        let mut revealed = vec![];
        for k in &keys {
            let a = &arts[&(k.clone(), rid.clone())];
            revealed.push(vault::reveal(access, &a.id, o.final_test, "compare")?);
        }
        let mut cases = score_all(&revealed, &spec);
        per_release.insert(rid.clone(), aggregate(&mut cases, &spec, o.n_boot, o.seed));
        case_rows.insert(rid.clone(), cases);
    }

    let reference = &ids[0];
    let mut paired = Map::new();
    for rid in &ids[1..] {
        let mut diffs = Map::new();
        let ref_metrics = per_release[reference]["metrics"]
            .as_object()
            .cloned()
            .unwrap_or_default();
        for m in ref_metrics.keys() {
            if per_release[rid]["metrics"].get(m).is_none() {
                continue;
            }
            let mut vals = vec![];
            let mut labs = vec![];
            for (a, b) in case_rows[reference].iter().zip(&case_rows[rid]) {
                if a.scored && b.scored {
                    if let (Some(x), Some(y)) = (a.metrics.get(m), b.metrics.get(m)) {
                        vals.push(y - x);
                        labs.push(a.cluster.clone().unwrap_or_default());
                    }
                }
            }
            if !vals.is_empty() {
                diffs.insert(
                    m.clone(),
                    cluster_bootstrap(&vals, &labs, o.n_boot, o.seed, 0.95),
                );
            }
        }
        paired.insert(
            rid.clone(),
            json!({"minus": reference, "differences": diffs}),
        );
    }
    let cases: Map<String, Value> = case_rows
        .iter()
        .map(|(rid, rows)| {
            (
                rid.clone(),
                Value::Array(rows.iter().map(public_case).collect()),
            )
        })
        .collect();
    Ok(json!({
        "partition": o.partition, "final_test": o.final_test, "releases": ids, "reference": reference,
        "allowed_mismatches": allowed, "n_cases": keys.len(),
        "excluded_incomplete_cases": incomplete.iter().map(|(k, v)| json!({"run_id": k.0, "prefix_hash": k.1, "missing": v})).collect::<Vec<_>>(),
        "spec": spec.to_value(), "per_release": per_release, "paired_vs_reference": paired, "cases": cases,
    }))
}

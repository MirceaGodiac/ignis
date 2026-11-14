//! Group-cluster bootstrap over independent units (V3.1 §13.7).
//!
//! The unit of independence is never a frame or a run when runs share a session: clusters are the
//! connected components of "same split group" ∪ "same session", so runs from one session always fall
//! in one cluster even if a split grouped them differently.

use std::collections::{BTreeMap, HashMap};

use rayon::prelude::*;
use serde_json::{json, Map, Value};

use crate::util::{mean, quantile_sorted, Rng};

/// Cluster label per case: union-find over shared split group or shared session (same operations,
/// hence the same root labels, as the Python implementation).
pub fn clusters(
    run_ids: &[String],
    groups: &[Option<String>],
    sessions: &[Option<String>],
) -> Vec<String> {
    let mut parent: HashMap<String, String> = HashMap::new();
    fn find(parent: &mut HashMap<String, String>, x: &str) -> String {
        let mut x = x.to_string();
        loop {
            let p = parent.entry(x.clone()).or_insert_with(|| x.clone()).clone();
            if p == x {
                return x;
            }
            let gp = parent.entry(p.clone()).or_insert_with(|| p.clone()).clone();
            parent.insert(x.clone(), gp.clone());
            x = gp;
        }
    }
    let union = |parent: &mut HashMap<String, String>, a: &str, b: &str| {
        let ra = find(parent, a);
        let rb = find(parent, b);
        parent.insert(ra, rb);
    };
    for ((r, g), s) in run_ids.iter().zip(groups).zip(sessions) {
        let node = format!("run:{r}");
        find(&mut parent, &node);
        if let Some(g) = g {
            union(&mut parent, &node, &format!("group:{g}"));
        }
        if let Some(s) = s {
            union(&mut parent, &node, &format!("session:{s}"));
        }
    }
    run_ids
        .iter()
        .map(|r| find(&mut parent, &format!("run:{r}")))
        .collect()
}

/// Percentile CI of the group-equal mean (mean of per-cluster means), resampling clusters with
/// replacement. Replicates run in parallel; replicate `b` draws from its own stream keyed by
/// (seed, b), so the result does not depend on the thread count.
pub fn cluster_bootstrap(
    values: &[f64],
    labels: &[String],
    n_boot: usize,
    seed: u64,
    level: f64,
) -> Value {
    let mut by: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    let mut kept = vec![];
    for (v, l) in values.iter().zip(labels) {
        if v.is_finite() {
            by.entry(l.as_str()).or_default().push(*v);
            kept.push(*v);
        }
    }
    let mut out = Map::new();
    out.insert("n_cases".into(), json!(kept.len()));
    out.insert("n_clusters".into(), json!(by.len()));
    if by.is_empty() {
        out.insert("estimate".into(), Value::Null);
        out.insert("ci".into(), Value::Null);
        return Value::Object(out);
    }
    let means: Vec<f64> = by.values().map(|v| mean(v)).collect();
    let est = mean(&means);
    out.insert("estimate".into(), json!(est));
    out.insert("case_mean".into(), json!(mean(&kept)));
    let k = means.len();
    if k < 2 {
        out.insert("ci".into(), Value::Null);
        out.insert(
            "note".into(),
            json!("one independent cluster: no sampling uncertainty estimable"),
        );
        return Value::Object(out);
    }
    let mut boots: Vec<f64> = (0..n_boot)
        .into_par_iter()
        .map(|b| {
            let mut rng = Rng::keyed(seed, &format!("cluster_bootstrap/{b}"));
            let draw: Vec<f64> = (0..k).map(|_| means[rng.below(k)]).collect();
            mean(&draw)
        })
        .collect();
    boots.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let a = (1.0 - level) / 2.0;
    out.insert(
        "ci".into(),
        json!([quantile_sorted(&boots, a), quantile_sorted(&boots, 1.0 - a)]),
    );
    out.insert("level".into(), json!(level));
    out.insert("n_boot".into(), json!(n_boot));
    out.insert("seed".into(), json!(seed));
    if k < 10 {
        out.insert(
            "note".into(),
            json!(format!(
                "only {k} independent clusters; interval is unstable"
            )),
        );
    }
    Value::Object(out)
}

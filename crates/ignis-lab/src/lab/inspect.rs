//! Read-only inspection of releases: summary, tensor slices/histograms, lineage, diffs.

use std::collections::BTreeSet;
use std::path::Path;

use ignis_core::contracts::ModelRelease;
use ignis_core::registry::{load_release, release_dir, Tensor, TensorMap};
use serde_json::{json, Map, Value};

use super::nd;
use crate::access::LabSession;
use crate::util::{artifacts_root, linspace, refused};

/// Elements printed inline at most; larger requests must use `--out`.
pub const MAX_INLINE: usize = 4096;

pub fn tensor_stats(t: &Tensor) -> Value {
    let mut st = Map::new();
    st.insert("shape".into(), json!(t.shape));
    st.insert("dtype".into(), json!("float64"));
    st.insert("size".into(), json!(t.data.len()));
    if t.data.is_empty() {
        return Value::Object(st);
    }
    let g: Vec<f64> = t.data.iter().cloned().filter(|x| x.is_finite()).collect();
    st.insert("non_finite".into(), json!(t.data.len() - g.len()));
    if !g.is_empty() {
        let mean = crate::util::mean(&g);
        let var = crate::util::mean(&g.iter().map(|x| (x - mean).powi(2)).collect::<Vec<_>>());
        st.insert("l2".into(), json!(nd::l2(&g)));
        st.insert(
            "linf".into(),
            json!(g.iter().fold(0.0f64, |a, x| a.max(x.abs()))),
        );
        st.insert(
            "min".into(),
            json!(g.iter().cloned().fold(f64::INFINITY, f64::min)),
        );
        st.insert(
            "max".into(),
            json!(g.iter().cloned().fold(f64::NEG_INFINITY, f64::max)),
        );
        st.insert("mean".into(), json!(mean));
        st.insert("std".into(), json!(var.sqrt()));
        st.insert(
            "sparsity".into(),
            json!(t.data.iter().filter(|x| **x == 0.0).count() as f64 / t.data.len() as f64),
        );
    }
    Value::Object(st)
}

pub(crate) fn load(id: &str) -> anyhow::Result<(ModelRelease, TensorMap, TensorMap)> {
    let root = artifacts_root();
    if !release_dir(&root, id).join("release.json").exists() {
        return Err(refused(format!(
            "release {id} not found under {}/models",
            root.display()
        )));
    }
    load_release(&root, id, None)
}

fn field(r: &ModelRelease, name: &str) -> Value {
    serde_json::to_value(r)
        .ok()
        .and_then(|v| v.get(name).cloned())
        .unwrap_or(Value::Null)
}

pub fn show(id: &str, posterior: bool) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    let (r, params, post) = load(id)?;
    let tensors: Vec<Value> = r
        .tensors
        .iter()
        .map(|e| {
            let mut row = Map::new();
            row.insert("name".into(), json!(e.name));
            row.insert("trainable".into(), json!(e.trainable));
            row.insert("sha256".into(), json!(e.sha256));
            if let Value::Object(s) = tensor_stats(&params[&e.name]) {
                row.extend(s);
            }
            Value::Object(row)
        })
        .collect();
    let n_params: usize = r.tensors.iter().map(|e| params[&e.name].len()).sum();
    let n_train: usize = r
        .tensors
        .iter()
        .filter(|e| e.trainable)
        .map(|e| params[&e.name].len())
        .sum();
    let mut out = Map::new();
    for k in [
        "id",
        "family",
        "model_name",
        "is_baseline",
        "state",
        "trained_on",
        "created_at",
        "architecture",
        "config",
        "priors",
        "observation_model",
        "numerics",
        "feature_order",
        "units",
        "normalization",
        "cohort_id",
        "split_id",
        "code_version",
        "applicability",
        "evaluation",
        "parent_id",
        "posterior_parameters",
        "posterior_valid_for_parameters",
    ] {
        out.insert(k.into(), field(&r, k));
    }
    out.insert("lineage".into(), lineage_chain(id)?["chain"].clone());
    out.insert("n_parameters".into(), json!(n_params));
    out.insert("n_trainable".into(), json!(n_train));
    out.insert("tensors".into(), Value::Array(tensors));
    if posterior {
        let rows: Vec<Value> = r
            .posterior
            .iter()
            .map(|e| {
                let t = &post[&e.name];
                let mut row = Map::new();
                row.insert("name".into(), json!(e.name));
                row.insert("draws".into(), json!(t.shape.first().copied().unwrap_or(0)));
                if let Value::Object(s) = tensor_stats(t) {
                    row.extend(s);
                }
                Value::Object(row)
            })
            .collect();
        out.insert("posterior".into(), Value::Array(rows));
    }
    Ok(Value::Object(out))
}

pub fn histogram(v: &[f64], bins: usize) -> Value {
    let f: Vec<f64> = v.iter().cloned().filter(|x| x.is_finite()).collect();
    if f.is_empty() || bins == 0 {
        return Value::Null;
    }
    let (mut lo, mut hi) = (
        f.iter().cloned().fold(f64::INFINITY, f64::min),
        f.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    );
    if lo == hi {
        lo -= 0.5;
        hi += 0.5;
    }
    let edges = linspace(lo, hi, bins + 1);
    let mut counts = vec![0u64; bins];
    for x in &f {
        // NumPy: bins are half-open except the last, which includes the right edge.
        let mut b = (((x - lo) / (hi - lo)) * bins as f64) as usize;
        if b >= bins {
            b = bins - 1;
        }
        while b > 0 && *x < edges[b] {
            b -= 1;
        }
        while b + 1 < bins && *x >= edges[b + 1] {
            b += 1;
        }
        counts[b] += 1;
    }
    json!({"counts": counts, "edges": edges})
}

pub fn tensor(
    id: &str,
    name: &str,
    slice: Option<&str>,
    hist: Option<usize>,
    out: Option<&Path>,
    posterior: bool,
    max_elements: usize,
) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    let (r, params, post) = load(id)?;
    let src = if posterior { &post } else { &params };
    let Some(t) = src.get(name) else {
        return Err(refused(format!(
            "no {}tensor {name:?}; available: {:?}",
            if posterior { "posterior " } else { "" },
            src.keys().collect::<Vec<_>>()
        )));
    };
    let sel = if slice.is_some_and(|s| !s.is_empty()) {
        nd::get(
            t,
            &nd::select(&t.shape, &nd::parse_slice(slice, t.shape.len())?)?,
        )
    } else {
        t.clone()
    };
    let mut res = Map::new();
    res.insert("release".into(), json!(r.id));
    res.insert("name".into(), json!(name));
    res.insert("posterior".into(), json!(posterior));
    res.insert("slice".into(), json!(slice));
    res.insert("stats".into(), tensor_stats(&sel));
    if let Some(b) = hist.filter(|b| *b > 0) {
        let h = histogram(&sel.data, b);
        if !h.is_null() {
            res.insert("histogram".into(), h);
        }
    }
    if let Some(o) = out {
        nd::write_npy(o, &sel)?;
        res.insert("written".into(), json!(o.display().to_string()));
    } else if slice.is_some_and(|s| !s.is_empty()) {
        if sel.data.len() > max_elements {
            return Err(refused(format!(
                "slice has {} elements (> {max_elements}); narrow it or use --out file.npy",
                sel.data.len()
            )));
        }
        res.insert("values".into(), nd::to_json(&sel));
    }
    Ok(Value::Object(res))
}

fn read_release(id: &str) -> Option<ModelRelease> {
    let p = release_dir(&artifacts_root(), id).join("release.json");
    serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()
}

pub fn edit_manifest(note: Option<&str>) -> Option<Value> {
    let h = note?.strip_prefix("sha256:")?;
    if !ignis_core::hashing::is_digest(h) {
        return None;
    }
    let p = artifacts_root().join("edits").join(format!("{h}.json"));
    serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()
}

fn lineage_chain(id: &str) -> anyhow::Result<Value> {
    let mut chain = vec![];
    let mut cur = Some(id.to_string());
    let mut seen = BTreeSet::new();
    while let Some(c) = cur {
        if !seen.insert(c.clone()) {
            return Err(refused(format!("lineage cycle at {c}")));
        }
        let Some(r) = read_release(&c) else {
            chain.push(json!({"id": c, "missing": true}));
            break;
        };
        let man = edit_manifest(r.lineage_note.as_deref());
        chain.push(json!({
            "id": r.id, "model_name": r.model_name, "lineage": r.lineage, "lineage_note": r.lineage_note,
            "parent_id": r.parent_id, "state": r.state, "trained_on": r.trained_on,
            "posterior_valid_for_parameters": r.posterior_valid_for_parameters, "created_at": r.created_at,
            "edit": man.map(|m| json!({
                "ops": m["ops"].as_array().map(|a| a.iter().map(|o| o["op"].clone()).collect::<Vec<_>>()),
                "touched": m["touched"], "lineage_kinds": m["lineage_kinds"],
            })),
        }));
        cur = r.parent_id.clone();
    }
    let kinds: Vec<Value> = chain
        .iter()
        .filter(|c| c.get("missing").is_none())
        .map(|c| c["lineage"].clone())
        .collect();
    Ok(json!({"release": id, "depth": chain.len(), "chain": chain, "kinds": kinds}))
}

pub fn lineage(id: &str) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    lineage_chain(id)
}

fn dict_diff(a: &Value, b: &Value, path: &str, out: &mut Vec<Value>) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            let keys: BTreeSet<&String> = x.keys().chain(y.keys()).collect();
            for k in keys {
                let p = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                match (x.get(k), y.get(k)) {
                    (None, Some(v)) => out.push(json!({"path": p, "added": v})),
                    (Some(v), None) => out.push(json!({"path": p, "removed": v})),
                    (Some(u), Some(v)) => dict_diff(u, v, &p, out),
                    _ => {}
                }
            }
        }
        _ if a == b => {}
        _ => out.push(json!({"path": path, "a": a, "b": b})),
    }
}

pub fn diff(a_id: &str, b_id: &str) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    let (ra, pa, qa) = load(a_id)?;
    let (rb, pb, qb) = load(b_id)?;
    let ta: std::collections::BTreeMap<_, _> =
        ra.tensors.iter().map(|e| (e.name.clone(), e)).collect();
    let tb: std::collections::BTreeMap<_, _> =
        rb.tensors.iter().map(|e| (e.name.clone(), e)).collect();
    let names: BTreeSet<&String> = ta.keys().chain(tb.keys()).collect();
    let mut tens = vec![];
    for n in names {
        match (ta.get(n), tb.get(n)) {
            (Some(_), None) => {
                tens.push(json!({"name": n, "status": "removed", "shape_a": pa[n].shape}))
            }
            (None, Some(_)) => {
                tens.push(json!({"name": n, "status": "added", "shape_b": pb[n].shape}))
            }
            (Some(ea), Some(eb)) => {
                let mut row = Map::new();
                row.insert("name".into(), json!(n));
                row.insert("trainable".into(), json!([ea.trainable, eb.trainable]));
                if ea.sha256 == eb.sha256 {
                    row.insert("status".into(), json!("identical"));
                } else if pa[n].shape != pb[n].shape {
                    row.insert("status".into(), json!("reshaped"));
                    row.insert("shape_a".into(), json!(pa[n].shape));
                    row.insert("shape_b".into(), json!(pb[n].shape));
                } else {
                    let d: Vec<f64> = pb[n]
                        .data
                        .iter()
                        .zip(&pa[n].data)
                        .map(|(y, x)| y - x)
                        .collect();
                    let na = nd::l2(&pa[n].data);
                    let dl = nd::l2(&d);
                    row.insert("status".into(), json!("changed"));
                    row.insert("l2_delta".into(), json!(dl));
                    row.insert(
                        "linf_delta".into(),
                        json!(d.iter().fold(0.0f64, |a, x| a.max(x.abs()))),
                    );
                    row.insert(
                        "relative_l2".into(),
                        if na > 0.0 {
                            json!(dl / na)
                        } else {
                            Value::Null
                        },
                    );
                    row.insert(
                        "changed_fraction".into(),
                        json!(if d.is_empty() {
                            0.0
                        } else {
                            d.iter().filter(|x| **x != 0.0).count() as f64 / d.len() as f64
                        }),
                    );
                }
                tens.push(Value::Object(row));
            }
            _ => {}
        }
    }
    let la = lineage_chain(a_id)?;
    let lb = lineage_chain(b_id)?;
    let ids = |l: &Value| -> Vec<String> {
        l["chain"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|c| c["id"].as_str().map(String::from))
            .collect()
    };
    let (ia, ib) = (ids(&la), ids(&lb));
    let rel = if ib.contains(&ra.id) {
        "b descends from a"
    } else if ia.contains(&rb.id) {
        "a descends from b"
    } else if ia.iter().any(|x| ib.contains(x)) {
        "share ancestor"
    } else {
        "unrelated"
    };
    let kinds_between = |l: &Value, ids: &[String], anc: &str| -> Vec<Value> {
        let k = ids.iter().position(|x| x == anc).unwrap();
        l["chain"].as_array().unwrap()[..k]
            .iter()
            .map(|c| c["lineage"].clone())
            .collect()
    };
    let path_kinds = if ib.contains(&ra.id) {
        kinds_between(&lb, &ib, &ra.id)
    } else if ia.contains(&rb.id) {
        kinds_between(&la, &ia, &rb.id)
    } else {
        vec![]
    };
    let mut fields = Map::new();
    for f in [
        "architecture",
        "config",
        "priors",
        "observation_model",
        "numerics",
        "normalization",
        "units",
        "applicability",
    ] {
        let (x, y) = (field(&ra, f), field(&rb, f));
        if x != y {
            let mut out = vec![];
            dict_diff(&x, &y, "", &mut out);
            fields.insert(f.into(), Value::Array(out));
        }
    }
    let feq = ra.feature_order == rb.feature_order;
    Ok(json!({
        "a": ra.id, "b": rb.id, "relation": rel, "lineage_kinds_between": path_kinds,
        "model_name": [ra.model_name, rb.model_name], "family": [ra.family, rb.family],
        "feature_order_equal": feq,
        "feature_order": if feq { Value::Null } else { json!([ra.feature_order, rb.feature_order]) },
        "cohort_equal": ra.cohort_id == rb.cohort_id, "split_equal": ra.split_id == rb.split_id,
        "state": [ra.state, rb.state],
        "posterior_valid_for_parameters": [ra.posterior_valid_for_parameters, rb.posterior_valid_for_parameters],
        "fields": fields, "tensors": tens,
        "posterior": {"a": qa.keys().collect::<Vec<_>>(), "b": qb.keys().collect::<Vec<_>>()},
    }))
}

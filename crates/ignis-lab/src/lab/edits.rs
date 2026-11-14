//! Reproducible, JSON-described checkpoint edits. Every edit creates a NEW release; the parent's
//! directory is never written.
//!
//! An edit file is one op, a list of ops, or `{"ops": [...], "note": str, "lineage": kind?}`. Ops:
//!
//! ```text
//! {"op": "set",   "tensor": n, "value": v, "slice": "0:4,:"?}        assign (scalar or nested list, broadcast)
//! {"op": "add",   "tensor": n, "value": v, "slice"?}                  add
//! {"op": "scale", "tensor": n, "factor": f, "slice"?}                 multiply
//! {"op": "clip",  "tensor": n, "min"?: x, "max"?: y, "slice"?}        clamp values
//! {"op": "low_rank_add", "tensor": n, "slice"?, "A": list|file.npy, "B": list|file.npy, "alpha": 1.0}
//! {"op": "low_rank_add", "tensor": n, "rank": r, "seed": s, "std": 0.01, "alpha": 1.0}   seeded factors
//! {"op": "reinit", "tensor": n | "tensors": [..] | "pattern": regex, "seed": s,
//!  "init": normal|uniform|zeros|ones|glorot_normal|glorot_uniform|he_normal, "std"?: x}
//! {"op": "reparameterize", "tensor": n, "map": log|exp|softplus|inv_softplus|logit|sigmoid|sqrt|square|affine,
//!  "a"?: x, "b"?: y}      values become φ = map(θ); declared in architecture.reparameterizations
//! {"op": "freeze" | "unfreeze", "tensors": [..] | "pattern": regex}
//! {"op": "set_field", "field": config|architecture|priors|observation_model|numerics|normalization|applicability,
//!  "key": "a.b.c", "value": v, "kind"?: lineage kind (required for unclassified config keys)}
//! {"op": "from_npy", "tensor": n, "file": "path.npy", "slice"?, "mode"?: set|add}
//!      apply a tensor file produced by any tool (file and tensor sha256 recorded). Without a slice
//!      the tensor is replaced (or created when `n` is new; a new shape is an architecture change).
//! Declared structural transforms (architecture_change when they change names/shapes):
//! {"op": "create",  "tensor": n, "shape": [..], "init"?: kind, "seed"?, "std"?, "value"?: v}
//! {"op": "remove",  "tensor" | "tensors" | "pattern"}
//! {"op": "rename",  "tensor": n, "to": m}
//! {"op": "reshape", "tensor": n, "shape": [..] (one -1 allowed)}
//! {"op": "pad",     "tensor": n, "pad": [[before, after], ...], "value"?: 0}
//! {"op": "select",  "tensor": n, "slice": "0:4,:"}   keep only a sub-tensor (truncate)
//! ```
//!
//! Lineage: weight_edit for value changes; architecture_change when tensor names/shapes change, the
//! architecture or numerics change or a tensor is reparameterized; objective_change for priors /
//! observation model / config.objective*|loss*; preprocessing_change for normalization /
//! config.preprocessing*. The release lineage is the most structural kind present
//! (preprocessing > objective > architecture > weight). Posterior draws stay attached for
//! inspection, but `posterior_valid_for_parameters` becomes false when the edit touches the
//! parameters underlying them or changes the model they were drawn under.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{Lineage, ModelRelease, QualificationState};
use ignis_core::hashing::{content_hash, hex_part};
use ignis_core::registry::{save_release, Tensor, TensorMap};
use serde_json::{json, Map, Value};

use super::inspect::load;
use super::nd;
use crate::access::LabSession;
use crate::util::{artifacts_root, refused, Rng};

pub const FIELDS: [&str; 7] = [
    "config",
    "architecture",
    "priors",
    "observation_model",
    "numerics",
    "normalization",
    "applicability",
];
const MAPS: [&str; 8] = [
    "log",
    "exp",
    "softplus",
    "inv_softplus",
    "logit",
    "sigmoid",
    "sqrt",
    "square",
];

fn rank(l: Lineage) -> Option<u8> {
    match l {
        Lineage::WeightEdit => Some(0),
        Lineage::ArchitectureChange => Some(1),
        Lineage::ObjectiveChange => Some(2),
        Lineage::PreprocessingChange => Some(3),
        _ => None,
    }
}

fn of_rank(r: u8) -> Lineage {
    [
        Lineage::WeightEdit,
        Lineage::ArchitectureChange,
        Lineage::ObjectiveChange,
        Lineage::PreprocessingChange,
    ][r as usize]
}

fn lineage_str(l: Lineage) -> String {
    serde_json::to_value(l)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

fn parse_lineage(s: &str) -> anyhow::Result<Lineage> {
    serde_json::from_value(json!(s)).map_err(|_| refused(format!("unknown lineage kind {s:?}")))
}

pub fn normalize_edit(spec: Value) -> anyhow::Result<Map<String, Value>> {
    let spec = match spec {
        Value::Array(a) => json!({"ops": a}),
        Value::Object(o) if o.contains_key("op") => json!({"ops": [Value::Object(o)]}),
        v => v,
    };
    match spec {
        Value::Object(o)
            if o.get("ops")
                .and_then(Value::as_array)
                .is_some_and(|a| !a.is_empty()) =>
        {
            Ok(o)
        }
        _ => Err(refused(
            "edit must be an op, a list of ops, or {'ops': [...]}",
        )),
    }
}

fn resolve_path(v: &str, base: Option<&Path>) -> PathBuf {
    let p = PathBuf::from(v);
    if !p.is_absolute() && !p.exists() {
        if let Some(b) = base {
            return b.join(p);
        }
    }
    p
}

fn array(v: &Value, base: Option<&Path>) -> anyhow::Result<(Tensor, Value)> {
    if let Some(s) = v.as_str() {
        let f = nd::read_npy(&resolve_path(s, base))?;
        let sha = f.tensor.sha256();
        return Ok((
            f.tensor,
            json!({"file": s, "file_sha256": f.file_sha256, "dtype": f.dtype, "sha256": sha}),
        ));
    }
    let t = nd::from_json(v)?;
    let sha = t.sha256();
    Ok((t, json!({"inline": true, "sha256": sha})))
}

fn targets(op: &Map<String, Value>, names: &[String]) -> anyhow::Result<Vec<String>> {
    let t: Vec<String> = if let Some(t) = op.get("tensor").and_then(Value::as_str) {
        vec![t.to_string()]
    } else if let Some(ts) = op.get("tensors").and_then(Value::as_array) {
        ts.iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect()
    } else if let Some(p) = op.get("pattern").and_then(Value::as_str) {
        let rx = regex::Regex::new(p).map_err(|e| refused(format!("bad pattern {p:?}: {e}")))?;
        let t: Vec<String> = names.iter().filter(|n| rx.is_match(n)).cloned().collect();
        if t.is_empty() {
            return Err(refused(format!("pattern {p:?} matches no tensor")));
        }
        t
    } else {
        return Err(refused(format!(
            "op {} needs 'tensor', 'tensors' or 'pattern'",
            op.get("op").unwrap_or(&Value::Null)
        )));
    };
    let missing: Vec<&String> = t.iter().filter(|n| !names.contains(n)).collect();
    if !missing.is_empty() {
        return Err(refused(format!(
            "unknown tensor(s) {missing:?}; available: {names:?}"
        )));
    }
    Ok(t)
}

fn one_target(op: &Map<String, Value>, names: &[String]) -> anyhow::Result<String> {
    let t = targets(op, names)?;
    if t.len() != 1 {
        return Err(refused(format!("op {} takes exactly one tensor", op["op"])));
    }
    Ok(t.into_iter().next().unwrap())
}

fn set_dotted(d: &mut Value, key: &str, value: Value) {
    let parts: Vec<&str> = key.split('.').collect();
    let mut cur = d;
    for p in &parts[..parts.len() - 1] {
        if !cur.is_object() {
            *cur = json!({});
        }
        let m = cur.as_object_mut().unwrap();
        if !m.get(*p).is_some_and(Value::is_object) {
            m.insert(p.to_string(), json!({}));
        }
        cur = m.get_mut(*p).unwrap();
    }
    if !cur.is_object() {
        *cur = json!({});
    }
    cur.as_object_mut()
        .unwrap()
        .insert(parts[parts.len() - 1].to_string(), value);
}

fn field_kind(field: &str, key: &str, declared: Option<&str>) -> anyhow::Result<Lineage> {
    if let Some(d) = declared {
        let l = parse_lineage(d)?;
        if rank(l).is_none() {
            return Err(refused(format!(
                "set_field kind {d:?} is not a lab lineage kind"
            )));
        }
        return Ok(l);
    }
    let head = key.split('.').next().unwrap_or("");
    if field == "normalization" || (field == "config" && head.starts_with("preprocessing")) {
        return Ok(Lineage::PreprocessingChange);
    }
    if field == "priors"
        || field == "observation_model"
        || (field == "config" && (head.starts_with("objective") || head.starts_with("loss")))
    {
        return Ok(Lineage::ObjectiveChange);
    }
    if field == "architecture" || field == "numerics" {
        return Ok(Lineage::ArchitectureChange);
    }
    if field == "applicability" {
        return Ok(Lineage::WeightEdit);
    }
    Err(refused(format!(
        "set_field {field}.{key}: give 'kind' (objective_change, preprocessing_change, architecture_change) — the lineage kind cannot be inferred for this key"
    )))
}

fn init(kind: &str, shape: &[usize], rng: &mut Rng, std: Option<f64>) -> anyhow::Result<Tensor> {
    let n: usize = shape.iter().product();
    let fan_in = *shape.last().unwrap_or(&1) as f64;
    let fan_out = if shape.len() >= 2 {
        shape[0] as f64
    } else {
        fan_in
    };
    let normal = |rng: &mut Rng, s: f64| (0..n).map(|_| s * rng.normal()).collect::<Vec<_>>();
    let uniform = |rng: &mut Rng, s: f64| {
        (0..n)
            .map(|_| -s + 2.0 * s * rng.uniform())
            .collect::<Vec<_>>()
    };
    let data = match kind {
        "zeros" => vec![0.0; n],
        "ones" => vec![1.0; n],
        "normal" => normal(rng, std.unwrap_or(0.02)),
        "uniform" => uniform(rng, std.unwrap_or(0.05)),
        "glorot_normal" => normal(rng, (2.0 / (fan_in + fan_out)).sqrt()),
        "glorot_uniform" => uniform(rng, (6.0 / (fan_in + fan_out)).sqrt()),
        "he_normal" => normal(rng, (2.0 / fan_in).sqrt()),
        k => return Err(refused(format!("unknown init {k:?}"))),
    };
    Ok(Tensor::new(shape.to_vec(), data))
}

fn forward_map(m: &str) -> fn(f64, f64, f64) -> f64 {
    match m {
        "log" => |x, _, _| x.ln(),
        "exp" => |x, _, _| x.exp(),
        "softplus" => |x, _, _| x.max(0.0) + (-x.abs()).exp().ln_1p(),
        "inv_softplus" => |y, _, _| y + (-(-y).exp_m1()).ln(),
        "logit" => |p, _, _| p.ln() - (-p).ln_1p(),
        "sigmoid" => |z, _, _| 1.0 / (1.0 + (-z).exp()),
        "sqrt" => |x, _, _| x.sqrt(),
        "square" => |x, _, _| x * x,
        _ => |x, a, b| (x - b) / a,
    }
}

fn shape_of(v: Option<&Value>) -> anyhow::Result<Vec<i64>> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|x| {
                    x.as_i64()
                        .ok_or_else(|| refused("shape entries must be integers"))
                })
                .collect()
        })
        .unwrap_or_else(|| Err(refused("op needs 'shape': [..]")))
}

fn f(op: &Map<String, Value>, k: &str) -> Option<f64> {
    op.get(k).and_then(Value::as_f64)
}

fn release_fields(r: &ModelRelease) -> BTreeMap<&'static str, Value> {
    let v = serde_json::to_value(r).expect("release serializes");
    FIELDS.iter().map(|k| (*k, v[*k].clone())).collect()
}

/// Parameter names the posterior draws are draws *of*: `release.posterior_parameters`, else
/// `config|architecture.posterior_parameters` (list, or {posterior_tensor: [names]}), else
/// posterior tensors named like a parameter, else — conservatively — every parameter.
fn posterior_basis(r: &ModelRelease, params: &TensorMap, post: &TensorMap) -> BTreeSet<String> {
    if post.is_empty() {
        return BTreeSet::new();
    }
    if !r.posterior_parameters.is_empty() {
        return r.posterior_parameters.iter().cloned().collect();
    }
    match r
        .config
        .get("posterior_parameters")
        .or_else(|| r.architecture.get("posterior_parameters"))
    {
        Some(Value::Object(m)) => {
            return m
                .values()
                .flat_map(|v| match v {
                    Value::String(s) => vec![s.clone()],
                    Value::Array(a) => a
                        .iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect(),
                    _ => vec![],
                })
                .collect()
        }
        Some(Value::Array(a)) => {
            return a
                .iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        }
        _ => {}
    }
    let same: BTreeSet<String> = post
        .keys()
        .filter(|k| params.contains_key(*k))
        .cloned()
        .collect();
    if same.is_empty() {
        params.keys().cloned().collect()
    } else {
        same
    }
}

pub fn apply_edit(
    release_id: &str,
    spec: Value,
    base: Option<&Path>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    let spec = normalize_edit(spec)?;
    let root = artifacts_root();
    let (parent, params0, post) = load(release_id)?;
    let mut params = params0.clone();
    let trainable0: BTreeSet<String> = parent
        .tensors
        .iter()
        .filter(|e| e.trainable)
        .map(|e| e.name.clone())
        .collect();
    let mut trainable = trainable0.clone();
    let mut explicit_frozen: BTreeSet<String> = BTreeSet::new();
    let parent_fields = release_fields(&parent);
    let mut fields = parent_fields.clone();
    let mut kinds: BTreeSet<u8> = BTreeSet::new();
    let mut records = vec![];
    let mut declared: BTreeSet<String> = BTreeSet::new();

    for (i, op) in spec["ops"].as_array().unwrap().iter().enumerate() {
        let Some(op) = op
            .as_object()
            .filter(|o| o.get("op").and_then(Value::as_str).is_some())
        else {
            return Err(refused(format!("op #{i} is not an object with 'op'")));
        };
        let kind = op["op"].as_str().unwrap();
        let mut rec = op.clone();
        let names: Vec<String> = params.keys().cloned().collect();
        let slice = op.get("slice").and_then(Value::as_str);
        match kind {
            "set" | "add" | "scale" | "clip" => {
                let t = one_target(op, &names)?;
                let a = params.get_mut(&t).unwrap();
                let sel = nd::select(&a.shape, &nd::parse_slice(slice, a.shape.len())?)?;
                let cur = nd::get(a, &sel);
                let new: Vec<f64> = match kind {
                    "scale" => {
                        let k = f(op, "factor").ok_or_else(|| refused("scale needs 'factor'"))?;
                        cur.data.iter().map(|x| x * k).collect()
                    }
                    "clip" => {
                        let (lo, hi) = (f(op, "min").unwrap_or(f64::NEG_INFINITY), f(op, "max").unwrap_or(f64::INFINITY));
                        if lo > hi {
                            return Err(refused("clip needs min <= max"));
                        }
                        cur.data.iter().map(|x| x.max(lo).min(hi)).collect()
                    }
                    _ => {
                        let v = nd::from_json(op.get("value").ok_or_else(|| refused(format!("{kind} needs 'value'")))?)?;
                        let b = nd::broadcast(&v, &sel.shape)?;
                        if kind == "set" { b } else { cur.data.iter().zip(&b).map(|(x, y)| x + y).collect() }
                    }
                };
                nd::put(a, &sel, &new);
                declared.insert(t);
                kinds.insert(0);
            }
            "low_rank_add" => {
                let t = one_target(op, &names)?;
                let a = params.get_mut(&t).unwrap();
                let sel = nd::select(&a.shape, &nd::parse_slice(slice, a.shape.len())?)?;
                if sel.shape.len() != 2 {
                    return Err(refused(format!(
                        "low_rank_add needs a 2-D tensor or slice; {t}{} has shape {:?}",
                        slice.unwrap_or(""),
                        sel.shape
                    )));
                }
                let (m, n) = (sel.shape[0], sel.shape[1]);
                let (fa, fb) = if op.contains_key("A") || op.contains_key("B") {
                    let (fa, ia) = array(op.get("A").unwrap_or(&Value::Null), base)?;
                    let (fb, ib) = array(op.get("B").unwrap_or(&Value::Null), base)?;
                    rec.insert("factors".into(), json!({"A": ia, "B": ib}));
                    (fa, fb)
                } else {
                    let r = op.get("rank").and_then(Value::as_u64).ok_or_else(|| refused("low_rank_add needs A/B or rank+seed"))? as usize;
                    let seed = op.get("seed").and_then(Value::as_u64).ok_or_else(|| refused("seeded low_rank_add needs 'seed'"))?;
                    let std = f(op, "std").unwrap_or(0.01);
                    let mut rng = Rng::keyed(seed, &t);
                    let fa = Tensor::new(vec![m, r], (0..m * r).map(|_| std * rng.normal()).collect());
                    let fb = Tensor::new(vec![r, n], (0..r * n).map(|_| std * rng.normal()).collect());
                    rec.insert("factors".into(), json!({"A": {"seeded": true, "sha256": fa.sha256()}, "B": {"seeded": true, "sha256": fb.sha256()}}));
                    (fa, fb)
                };
                if fa.shape.len() != 2 || fb.shape.len() != 2 || fa.shape[0] != m || fb.shape[1] != n || fa.shape[1] != fb.shape[0] {
                    return Err(refused(format!("factor shapes {:?}@{:?} do not fit {:?}", fa.shape, fb.shape, sel.shape)));
                }
                let alpha = f(op, "alpha").unwrap_or(1.0);
                let r = fa.shape[1];
                let cur = nd::get(a, &sel);
                let mut new = cur.data.clone();
                for i in 0..m {
                    for j in 0..n {
                        let mut s = 0.0;
                        for k in 0..r {
                            s += fa.data[i * r + k] * fb.data[k * n + j];
                        }
                        new[i * n + j] += alpha * s;
                    }
                }
                nd::put(a, &sel, &new);
                declared.insert(t);
                kinds.insert(0);
            }
            "reinit" => {
                let ts = targets(op, &names)?;
                let seed = op.get("seed").and_then(Value::as_u64).ok_or_else(|| refused("reinit needs 'seed'"))?;
                let ik = op.get("init").and_then(Value::as_str).unwrap_or("normal");
                for t in &ts {
                    let shape = params[t].shape.clone();
                    params.insert(t.clone(), init(ik, &shape, &mut Rng::keyed(seed, t), f(op, "std"))?);
                }
                rec.insert("resolved_tensors".into(), json!(ts));
                declared.extend(ts);
                kinds.insert(0);
            }
            "reparameterize" => {
                let t = one_target(op, &names)?;
                let m = op.get("map").and_then(Value::as_str).unwrap_or("");
                let (a, b) = (f(op, "a").unwrap_or(1.0), f(op, "b").unwrap_or(0.0));
                if m == "affine" && a == 0.0 {
                    return Err(refused("affine map needs a != 0"));
                }
                if m != "affine" && !MAPS.contains(&m) {
                    let mut d: Vec<&str> = MAPS.to_vec();
                    d.sort();
                    d.push("affine");
                    return Err(refused(format!("undeclared map {m:?}; declared: {d:?}")));
                }
                let fwd = forward_map(m);
                let x = params.get_mut(&t).unwrap();
                let new: Vec<f64> = x.data.iter().map(|&v| fwd(v, a, b)).collect();
                if new.iter().any(|v| !v.is_finite()) {
                    return Err(refused(format!("map {m} produced non-finite values on {t} (not defined on all values)")));
                }
                x.data = new;
                let arch = fields.get_mut("architecture").unwrap();
                let mut entry = json!({"map": m});
                if m == "affine" {
                    entry["a"] = json!(a);
                    entry["b"] = json!(b);
                }
                let reps = arch.as_object_mut().unwrap().entry("reparameterizations").or_insert_with(|| json!({}));
                if !reps.is_object() {
                    *reps = json!({});
                }
                let list = reps.as_object_mut().unwrap().entry(t.clone()).or_insert_with(|| json!([]));
                if let Value::Array(l) = list {
                    l.push(entry); // stored value = maps applied in order to θ
                }
                declared.insert(t);
                kinds.insert(1);
            }
            "freeze" | "unfreeze" => {
                let ts = targets(op, &names)?;
                for t in &ts {
                    if kind == "freeze" {
                        trainable.remove(t);
                        explicit_frozen.insert(t.clone());
                    } else {
                        trainable.insert(t.clone());
                        explicit_frozen.remove(t);
                    }
                }
                rec.insert("resolved_tensors".into(), json!(ts));
                kinds.insert(0);
            }
            "set_field" => {
                let fl = op.get("field").and_then(Value::as_str).unwrap_or("");
                if !FIELDS.contains(&fl) {
                    return Err(refused(format!("set_field: field must be one of {FIELDS:?}")));
                }
                let key = op.get("key").and_then(Value::as_str).ok_or_else(|| refused("set_field needs 'key'"))?;
                let k = field_kind(fl, key, op.get("kind").and_then(Value::as_str))?;
                set_dotted(fields.get_mut(fl).unwrap(), key, op.get("value").cloned().unwrap_or(Value::Null));
                kinds.insert(rank(k).unwrap());
            }
            "from_npy" => {
                let t = op.get("tensor").and_then(Value::as_str).ok_or_else(|| refused("from_npy needs 'tensor'"))?.to_string();
                let file = op.get("file").and_then(Value::as_str).ok_or_else(|| refused("from_npy needs 'file'"))?;
                let nf = nd::read_npy(&resolve_path(file, base))?;
                let mode = op.get("mode").and_then(Value::as_str).unwrap_or("set");
                if !matches!(mode, "set" | "add") {
                    return Err(refused("from_npy mode must be set|add"));
                }
                rec.insert("source".into(), json!({"file": file, "file_sha256": nf.file_sha256, "dtype": nf.dtype,
                    "shape": nf.tensor.shape, "sha256": nf.tensor.sha256()}));
                match params.get_mut(&t) {
                    Some(a) if slice.is_some() || mode == "add" => {
                        let sel = nd::select(&a.shape, &nd::parse_slice(slice, a.shape.len())?)?;
                        let b = nd::broadcast(&nf.tensor, &sel.shape)?;
                        let new: Vec<f64> = if mode == "add" { nd::get(a, &sel).data.iter().zip(&b).map(|(x, y)| x + y).collect() } else { b };
                        nd::put(a, &sel, &new);
                    }
                    Some(a) => *a = nf.tensor,
                    None if slice.is_none() && mode == "set" => {
                        params.insert(t.clone(), nf.tensor);
                    }
                    None => return Err(refused(format!("from_npy: tensor {t:?} does not exist (a new tensor takes no slice and mode set)"))),
                }
                declared.insert(t);
                kinds.insert(0);
            }
            "create" => {
                let t = op.get("tensor").and_then(Value::as_str).ok_or_else(|| refused("create needs 'tensor'"))?.to_string();
                if params.contains_key(&t) {
                    return Err(refused(format!("tensor {t:?} already exists")));
                }
                let shape: Vec<usize> = shape_of(op.get("shape"))?.iter().map(|&d| d.max(0) as usize).collect();
                let new = if let Some(v) = op.get("value") {
                    Tensor::new(shape.clone(), nd::broadcast(&nd::from_json(v)?, &shape)?)
                } else {
                    let seed = op.get("seed").and_then(Value::as_u64).unwrap_or(0);
                    init(op.get("init").and_then(Value::as_str).unwrap_or("zeros"), &shape, &mut Rng::keyed(seed, &t), f(op, "std"))?
                };
                params.insert(t.clone(), new);
                declared.insert(t);
                kinds.insert(1);
            }
            "remove" => {
                let ts = targets(op, &names)?;
                for t in &ts {
                    params.remove(t);
                    trainable.remove(t);
                }
                rec.insert("resolved_tensors".into(), json!(ts));
                declared.extend(ts);
                kinds.insert(1);
            }
            "rename" => {
                let t = one_target(op, &names)?;
                let to = op.get("to").and_then(Value::as_str).ok_or_else(|| refused("rename needs 'to'"))?.to_string();
                if params.contains_key(&to) {
                    return Err(refused(format!("tensor {to:?} already exists")));
                }
                let v = params.remove(&t).unwrap();
                params.insert(to.clone(), v);
                if trainable.remove(&t) {
                    trainable.insert(to.clone());
                } else {
                    explicit_frozen.insert(to.clone());
                }
                explicit_frozen.remove(&t);
                declared.insert(t);
                declared.insert(to);
                kinds.insert(1);
            }
            "reshape" => {
                let t = one_target(op, &names)?;
                let a = params.get_mut(&t).unwrap();
                let mut shape = shape_of(op.get("shape"))?;
                let known: i64 = shape.iter().filter(|&&d| d >= 0).product();
                let neg = shape.iter().filter(|&&d| d < 0).count();
                if neg > 1 {
                    return Err(refused("reshape: at most one -1"));
                }
                if neg == 1 {
                    if known == 0 || a.data.len() as i64 % known != 0 {
                        return Err(refused(format!("cannot reshape {:?} into {shape:?}", a.shape)));
                    }
                    for d in shape.iter_mut() {
                        if *d < 0 {
                            *d = a.data.len() as i64 / known;
                        }
                    }
                }
                let new: Vec<usize> = shape.iter().map(|&d| d as usize).collect();
                if new.iter().product::<usize>() != a.data.len() {
                    return Err(refused(format!("cannot reshape {:?} into {new:?}", a.shape)));
                }
                a.shape = new;
                declared.insert(t);
                kinds.insert(1);
            }
            "pad" => {
                let t = one_target(op, &names)?;
                let a = params.get_mut(&t).unwrap();
                let pads: Vec<(usize, usize)> = op
                    .get("pad")
                    .and_then(Value::as_array)
                    .ok_or_else(|| refused("pad needs 'pad': [[before, after], ...]"))?
                    .iter()
                    .map(|p| {
                        let b = p.get(0).and_then(Value::as_u64);
                        let e = p.get(1).and_then(Value::as_u64);
                        match (b, e) {
                            (Some(b), Some(e)) => Ok((b as usize, e as usize)),
                            _ => Err(refused("pad entries are [before, after] non-negative integers")),
                        }
                    })
                    .collect::<anyhow::Result<_>>()?;
                if pads.len() != a.shape.len() {
                    return Err(refused(format!("pad needs one [before, after] per dimension ({})", a.shape.len())));
                }
                let fill = f(op, "value").unwrap_or(0.0);
                let new_shape: Vec<usize> = a.shape.iter().zip(&pads).map(|(d, (b, e))| d + b + e).collect();
                let mut out = Tensor::new(new_shape.clone(), vec![fill; new_shape.iter().product()]);
                let idx: Vec<nd::Idx> = pads.iter().zip(&a.shape).map(|((b, _), d)| nd::Idx::Range(Some(*b as i64), Some((*b + *d) as i64), None)).collect();
                let sel = nd::select(&new_shape, &idx)?;
                nd::put(&mut out, &sel, &a.data);
                *a = out;
                declared.insert(t);
                kinds.insert(1);
            }
            "select" => {
                let t = one_target(op, &names)?;
                let s = slice.ok_or_else(|| refused("select needs 'slice'"))?;
                let a = params.get_mut(&t).unwrap();
                *a = nd::get(a, &nd::select(&a.shape, &nd::parse_slice(Some(s), a.shape.len())?)?);
                declared.insert(t);
                kinds.insert(1);
            }
            "python" => {
                return Err(refused(
                    "python transforms are not executed; write the tensor to a .npy file and use {\"op\": \"from_npy\"} \
                     (its hash is recorded), or a declared transform op (create/remove/rename/reshape/pad/select)",
                ))
            }
            k => return Err(refused(format!("unknown op {k:?}"))),
        }
        records.push(Value::Object(rec));
    }

    // --- what actually changed
    let before: BTreeMap<&str, &ignis_core::contracts::TensorEntry> = parent
        .tensors
        .iter()
        .map(|e| (e.name.as_str(), e))
        .collect();
    let after: BTreeMap<String, String> = params
        .iter()
        .map(|(k, v)| (k.clone(), v.sha256()))
        .collect();
    let added: Vec<String> = params
        .keys()
        .filter(|k| !before.contains_key(k.as_str()))
        .cloned()
        .collect();
    let removed: Vec<String> = before
        .keys()
        .filter(|k| !params.contains_key(**k))
        .map(|k| k.to_string())
        .collect();
    let reshaped: Vec<String> = params
        .iter()
        .filter(|(k, v)| {
            before
                .get(k.as_str())
                .is_some_and(|e| e.shape.iter().map(|&d| d as usize).collect::<Vec<_>>() != v.shape)
        })
        .map(|(k, _)| k.clone())
        .collect();
    let changed: Vec<String> = params
        .keys()
        .filter(|k| {
            before
                .get(k.as_str())
                .is_some_and(|e| e.sha256 != after[*k])
        })
        .cloned()
        .collect();
    let touched: BTreeSet<String> = changed
        .iter()
        .chain(&added)
        .chain(&removed)
        .cloned()
        .collect();
    trainable.retain(|t| params.contains_key(t));
    for a in &added {
        if !explicit_frozen.contains(a) {
            trainable.insert(a.clone()); // new tensors are trainable unless frozen in this edit
        }
    }
    if !added.is_empty() || !removed.is_empty() || !reshaped.is_empty() {
        kinds.insert(1);
    }
    let field_changes: Vec<&str> = FIELDS
        .iter()
        .filter(|f| fields[**f] != parent_fields[**f])
        .cloned()
        .collect();
    if touched.is_empty() && field_changes.is_empty() && trainable == trainable0 {
        return Err(refused("edit changes nothing (no tensor value, field or trainable flag differs from the parent)"));
    }
    let mut lineage = of_rank(*kinds.iter().max().unwrap_or(&0));
    if let Some(want) = spec.get("lineage").and_then(Value::as_str) {
        let w = parse_lineage(want)?;
        let Some(rw) = rank(w) else {
            return Err(refused(format!(
                "lab edits cannot be labelled {want}; fine_tune comes from `ignis lab resume`"
            )));
        };
        if rw < rank(lineage).unwrap() {
            return Err(refused(format!(
                "declared lineage {want} understates the edit (detected {})",
                lineage_str(lineage)
            )));
        }
        lineage = w;
    }

    let basis = posterior_basis(&parent, &params0, &post);
    let touched_basis: Vec<String> = basis.intersection(&touched).cloned().collect();
    let structural = kinds.iter().any(|k| *k > 0)
        || !added.is_empty()
        || !removed.is_empty()
        || !reshaped.is_empty();
    let valid = if post.is_empty() {
        parent.posterior_valid_for_parameters
    } else {
        parent.posterior_valid_for_parameters && touched_basis.is_empty() && !structural
    };

    let per_tensor: Map<String, Value> = touched
        .iter()
        .map(|n| (n.clone(), json!({"before": before.get(n.as_str()).map(|e| e.sha256.clone()), "after": after.get(n)})))
        .collect();
    let manifest = json!({
        "schema": "ignis.lab.edit/1",
        "parent_id": parent.id,
        "note": spec.get("note").cloned().unwrap_or(Value::Null),
        "ops": records,
        "declared_targets": declared,
        "touched": touched, "added": added, "removed": removed, "reshaped": reshaped,
        "per_tensor": per_tensor,
        "trainable_before": trainable0,
        "trainable_after": trainable,
        "field_changes": field_changes,
        "lineage": lineage, "lineage_kinds": kinds.iter().map(|k| lineage_str(of_rank(*k))).collect::<BTreeSet<_>>(),
        "posterior": {"present": !post.is_empty(), "basis": basis, "touched_basis": touched_basis,
                      "structural_change": structural, "valid_for_parameters": valid,
                      "parent_valid": parent.posterior_valid_for_parameters},
        "state": QualificationState::FitOnly,
        "executes_local_code": false,
    });
    let mhash = content_hash(&manifest, &[]);
    let mut result = json!({"parent_id": parent.id, "manifest_hash": mhash, "manifest": manifest, "dry_run": dry_run});
    if dry_run {
        return Ok(result);
    }
    let mpath = root
        .join("edits")
        .join(format!("{}.json", hex_part(&mhash)));
    std::fs::create_dir_all(mpath.parent().unwrap())?;
    if !mpath.exists() {
        std::fs::write(
            &mpath,
            serde_json::to_string_pretty(&ignis_core::hashing::sorted(&result["manifest"]))?,
        )?;
    }

    let mut child = parent.clone();
    for fl in &field_changes {
        let v = fields[*fl].clone();
        let bad = |e: serde_json::Error| {
            refused(format!(
                "field {fl} has an invalid value after the edit: {e}"
            ))
        };
        match *fl {
            "config" => child.config = serde_json::from_value(v).map_err(bad)?,
            "architecture" => child.architecture = serde_json::from_value(v).map_err(bad)?,
            "priors" => child.priors = serde_json::from_value(v).map_err(bad)?,
            "observation_model" => {
                child.observation_model = serde_json::from_value(v).map_err(bad)?
            }
            "numerics" => child.numerics = serde_json::from_value(v).map_err(bad)?,
            "normalization" => child.normalization = serde_json::from_value(v).map_err(bad)?,
            "applicability" => child.applicability = serde_json::from_value(v).map_err(bad)?,
            _ => unreachable!(),
        }
    }
    child.id = String::new();
    child.created_at = String::new();
    child.code_version = String::new();
    child.resource_record = None;
    child.parent_id = Some(parent.id.clone());
    child.lineage = lineage;
    child.lineage_note = Some(mhash.clone());
    child.posterior_valid_for_parameters = valid;
    child.state = QualificationState::FitOnly;
    child.evaluation = serde_json::from_value(
        json!({"inherited": false, "note": "validation metrics of the parent do not apply; re-score"}),
    )?;
    let frozen: Vec<String> = params
        .keys()
        .filter(|k| !trainable.contains(*k))
        .cloned()
        .collect();
    let saved = save_release(&root, child, &params, &post, &frozen)?;
    result["release_id"] = json!(saved.id);
    result["lineage"] = json!(saved.lineage);
    result["posterior_valid_for_parameters"] = json!(valid);
    result["manifest_path"] = json!(mpath.display().to_string());
    Ok(result)
}

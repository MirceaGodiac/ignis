//! Probe a release on train/validation views through the expert's optional lab hooks
//! ([`Expert::objective`] → value and gradient per tensor, [`Expert::activations`]).
//!
//! Views come from the injected [`ViewSource`] and pass the partition guard, so a probe can never
//! see a test run.

use std::collections::BTreeMap;
use std::path::Path;

use ignis_core::contracts::{ModelRelease, Split};
use ignis_core::expert::{find, Expert};
use ignis_core::npz::{read_npz, write_npz, NpyArray};
use ignis_core::registry::{load, LoadedRelease, Tensor, TensorMap};
use ignis_core::views::{View, ViewSource};
use rayon::prelude::*;
use serde_json::{json, Map, Value};

use super::guard::{assert_lab_partition, filter_views};
use super::inspect::tensor_stats;
use super::nd::l2;
use crate::access::LabSession;
use crate::util::{artifacts_root, linspace, load_split_file, py_float, refused, Rng};

pub struct Probe<'a> {
    pub release: LoadedRelease,
    pub expert: &'a dyn Expert,
    pub views: Vec<View>,
    pub warnings: Vec<String>,
}

fn split_of(reference: &str, source: &dyn ViewSource) -> anyhow::Result<Split> {
    if let Some(s) = load_split_file(reference)? {
        return Ok(s);
    }
    source
        .split(reference)
        .map_err(|e| refused(format!("split {reference:?} not found ({e})")))
}

pub fn setup<'a>(
    release_id: &str,
    cohort: &str,
    split: &str,
    partition: &str,
    limit: Option<usize>,
    experts: &'a [Box<dyn Expert>],
    source: &dyn ViewSource,
) -> anyhow::Result<Probe<'a>> {
    assert_lab_partition(partition)?;
    super::inspect::load(release_id)?; // friendly "not found"
    let release = load(&artifacts_root(), release_id)?;
    let r: &ModelRelease = &release.release;
    let names: Vec<&str> = experts.iter().map(|e| e.model_name()).collect();
    let expert = find(experts, &r.model_name).ok_or_else(|| {
        refused(format!(
            "no expert with model_name={:?}; registered: {names:?}",
            r.model_name
        ))
    })?;
    let sp = split_of(split, source)?;
    let all = source.views(r.family, cohort)?;
    let mut views = filter_views(all, &sp, partition)?;
    if views.is_empty() {
        return Err(refused(format!(
            "no {partition} views for cohort {cohort} under split {}",
            sp.id
        )));
    }
    if let Some(l) = limit.filter(|l| *l > 0) {
        views.truncate(l);
    }
    let mut warnings = vec![];
    if cohort != r.cohort_id {
        warnings.push(format!(
            "cohort {cohort} differs from the release cohort {}",
            r.cohort_id
        ));
    }
    if sp.id != r.split_id {
        warnings.push(format!(
            "split {} differs from the release split {}; partitions may overlap its train set",
            sp.id, r.split_id
        ));
    }
    Ok(Probe {
        release,
        expert,
        views,
        warnings,
    })
}

fn need<T>(x: Option<anyhow::Result<T>>, expert: &dyn Expert, hook: &str) -> anyhow::Result<T> {
    match x {
        None => Err(refused(format!(
            "expert {} does not expose {hook}()",
            expert.model_name()
        ))),
        Some(r) => r,
    }
}

fn objective(
    p: &Probe<'_>,
    params: &TensorMap,
    views: &[&View],
) -> anyhow::Result<(f64, TensorMap)> {
    need(
        p.expert.objective(&p.release, params, views),
        p.expert,
        "objective",
    )
}

fn save_npz(out: &Path, m: &BTreeMap<String, Tensor>) -> anyhow::Result<()> {
    if let Some(d) = out.parent() {
        std::fs::create_dir_all(d)?;
    }
    let npy = m
        .iter()
        .map(|(k, t)| (k.clone(), NpyArray::from_f64(t.shape.clone(), &t.data)))
        .collect();
    write_npz(out, &npy)?;
    Ok(())
}

pub struct Ctx<'a> {
    pub experts: &'a [Box<dyn Expert>],
    pub source: &'a dyn ViewSource,
}

pub fn activations(
    c: &Ctx<'_>,
    release: &str,
    cohort: &str,
    split: &str,
    partition: &str,
    limit: usize,
    out: Option<&Path>,
) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    let p = setup(
        release,
        cohort,
        split,
        partition,
        Some(limit),
        c.experts,
        c.source,
    )?;
    let mut per_view = vec![];
    let mut store = BTreeMap::new();
    for v in &p.views {
        let acts = need(
            p.expert.activations(&p.release, &p.release.params, v),
            p.expert,
            "activations",
        )?;
        let stats: Map<String, Value> = acts
            .iter()
            .map(|(k, a)| (k.clone(), tensor_stats(a)))
            .collect();
        per_view.push(json!({"run_id": v.run_id(), "activations": stats}));
        for (k, a) in acts {
            store.insert(format!("{}/{k}", v.run_id()), a);
        }
    }
    let mut res = json!({"release": p.release.release.id, "partition": partition, "n_views": p.views.len(),
                         "warnings": p.warnings, "views": per_view});
    if let Some(o) = out {
        save_npz(o, &store)?;
        res["written"] = json!(o.display().to_string());
    }
    Ok(res)
}

pub fn grad(
    c: &Ctx<'_>,
    release: &str,
    cohort: &str,
    split: &str,
    partition: &str,
    out: Option<&Path>,
) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    let p = setup(release, cohort, split, partition, None, c.experts, c.source)?;
    let vs: Vec<&View> = p.views.iter().collect();
    let (val, g) = objective(&p, &p.release.params, &vs)?;
    let trainable: BTreeMap<&str, bool> = p
        .release
        .release
        .tensors
        .iter()
        .map(|e| (e.name.as_str(), e.trainable))
        .collect();
    let mut rows: Vec<Value> = g
        .iter()
        .map(|(k, ga)| {
            let pn = p.release.params.get(k).map_or(0.0, |t| l2(&t.data));
            let gn = l2(&ga.data);
            json!({"name": k, "trainable": trainable.get(k.as_str()).copied().unwrap_or(true), "grad_l2": gn,
                   "grad_linf": ga.data.iter().fold(0.0f64, |a, x| a.max(x.abs())), "param_l2": pn,
                   "relative": if pn > 0.0 { json!(gn / pn) } else { Value::Null },
                   "non_finite": ga.data.iter().filter(|x| !x.is_finite()).count()})
        })
        .collect();
    rows.sort_by(|a, b| {
        b["grad_l2"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["grad_l2"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let global = rows
        .iter()
        .map(|r| r["grad_l2"].as_f64().unwrap_or(0.0).powi(2))
        .sum::<f64>()
        .sqrt();
    let mut res = json!({"release": p.release.release.id, "partition": partition, "n_views": p.views.len(), "objective": val,
                         "global_grad_l2": global, "warnings": p.warnings, "tensors": rows});
    if let Some(o) = out {
        save_npz(o, &g)?;
        res["written"] = json!(o.display().to_string());
    }
    Ok(res)
}

fn direction(
    kind: &str,
    p: &Probe<'_>,
    names: &[String],
    seed: u64,
    normalize: &str,
) -> anyhow::Result<TensorMap> {
    let base = &p.release.params;
    let mut d: TensorMap = match kind {
        "random" => names
            .iter()
            .map(|k| {
                let mut rng = Rng::keyed(seed, k);
                (
                    k.clone(),
                    Tensor::new(
                        base[k].shape.clone(),
                        (0..base[k].len()).map(|_| rng.normal()).collect(),
                    ),
                )
            })
            .collect(),
        "grad" => {
            let vs: Vec<&View> = p.views.iter().collect();
            let (_, g) = objective(p, base, &vs)?;
            names
                .iter()
                .filter_map(|k| {
                    g.get(k).map(|t| {
                        (
                            k.clone(),
                            Tensor::new(t.shape.clone(), t.data.iter().map(|x| -x).collect()),
                        )
                    })
                })
                .collect()
        }
        file => {
            let path = Path::new(file);
            if !path.is_file() {
                return Err(refused(format!(
                    "direction must be 'random', 'grad' or an .npz file; got {file:?}"
                )));
            }
            let z = read_npz(path)?;
            let d: TensorMap = names
                .iter()
                .filter_map(|k| {
                    z.get(k).map(|a| {
                        (
                            k.clone(),
                            Tensor {
                                shape: a.shape.clone(),
                                data: a.to_f64(),
                            },
                        )
                    })
                })
                .collect();
            if d.is_empty() {
                return Err(refused(format!(
                    "{file} contains none of the selected tensors"
                )));
            }
            d
        }
    };
    for (k, t) in &d {
        if t.shape != base[k].shape {
            return Err(refused(format!(
                "direction for {k} has shape {:?}, tensor has {:?}",
                t.shape, base[k].shape
            )));
        }
    }
    match normalize {
        "filter" => {
            // per-tensor norm matched to the parameter (Li et al., 2018)
            for (k, t) in d.iter_mut() {
                let (dn, pn) = (l2(&t.data), l2(&base[k].data));
                if dn > 0.0 {
                    t.data.iter_mut().for_each(|x| *x *= pn / dn);
                }
            }
        }
        "global" => {
            let tot = d
                .values()
                .map(|t| t.data.iter().map(|x| x * x).sum::<f64>())
                .sum::<f64>()
                .sqrt();
            if tot > 0.0 {
                d.values_mut()
                    .for_each(|t| t.data.iter_mut().for_each(|x| *x /= tot));
            }
        }
        "none" => {}
        _ => return Err(refused("normalize must be filter|global|none")),
    }
    Ok(d)
}

pub struct SensitivityOpts<'a> {
    pub partition: &'a str,
    pub direction: &'a str,
    pub direction2: Option<&'a str>,
    pub span: f64,
    pub points: usize,
    pub seed: u64,
    pub normalize: &'a str,
    pub tensors: Option<&'a str>,
}

/// Loss along θ + αd (1-D) or θ + αd₁ + βd₂ (2-D); trainable tensors unless a `tensors` regex is given.
pub fn sensitivity(
    c: &Ctx<'_>,
    release: &str,
    cohort: &str,
    split: &str,
    o: &SensitivityOpts<'_>,
) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    let p = setup(
        release,
        cohort,
        split,
        o.partition,
        None,
        c.experts,
        c.source,
    )?;
    let tensors = &p.release.release.tensors;
    let names: Vec<String> = match o.tensors {
        Some(rx) => {
            let rx =
                regex::Regex::new(rx).map_err(|e| refused(format!("bad tensors regex: {e}")))?;
            tensors
                .iter()
                .filter(|e| rx.is_match(&e.name))
                .map(|e| e.name.clone())
                .collect()
        }
        None => tensors
            .iter()
            .filter(|e| e.trainable)
            .map(|e| e.name.clone())
            .collect(),
    };
    if names.is_empty() {
        return Err(refused("no tensors selected for the direction"));
    }
    let d1 = direction(o.direction, &p, &names, o.seed, o.normalize)?;
    let d2 = match o.direction2 {
        Some(k) => Some(direction(k, &p, &names, o.seed + 1, o.normalize)?),
        None => None,
    };
    let alphas = linspace(-o.span, o.span, o.points);
    let vs: Vec<&View> = p.views.iter().collect();
    let at = |a: f64, b: f64| -> anyhow::Result<f64> {
        let mut q = p.release.params.clone();
        for (k, t) in q.iter_mut() {
            if let Some(dd) = d1.get(k) {
                t.data
                    .iter_mut()
                    .zip(&dd.data)
                    .for_each(|(x, y)| *x += a * y);
            }
            if let Some(dd) = d2.as_ref().and_then(|d| d.get(k)) {
                t.data
                    .iter_mut()
                    .zip(&dd.data)
                    .for_each(|(x, y)| *x += b * y);
            }
        }
        Ok(objective(&p, &q, &vs)?.0)
    };
    let mut res = json!({"release": p.release.release.id, "partition": o.partition, "n_views": p.views.len(), "tensors": names,
                         "direction": o.direction, "direction2": o.direction2, "normalize": o.normalize, "seed": o.seed,
                         "alphas": alphas, "objective_at_origin": at(0.0, 0.0)?, "warnings": p.warnings});
    if d2.is_none() {
        let loss: Vec<f64> = alphas
            .par_iter()
            .map(|&a| at(a, 0.0))
            .collect::<anyhow::Result<_>>()?;
        res["loss"] = json!(loss);
    } else {
        let grid: Vec<Vec<f64>> = alphas
            .par_iter()
            .map(|&a| {
                alphas
                    .iter()
                    .map(|&b| at(a, b))
                    .collect::<anyhow::Result<Vec<f64>>>()
            })
            .collect::<anyhow::Result<_>>()?;
        res["betas"] = json!(alphas);
        res["loss_grid"] = json!(grid); // [alpha][beta]
    }
    Ok(res)
}

fn conditions(v: &View) -> BTreeMap<String, f64> {
    match v {
        View::Flex(x) => x.conditions.clone(),
        View::Acme(x) => x.features.clone(),
        View::Bass(x) => x.conditions.clone(),
    }
}

/// Per-run objective grouped by conditions, VALIDATION partition only.
pub fn residuals(
    c: &Ctx<'_>,
    release: &str,
    cohort: &str,
    split: &str,
    by: Option<Vec<String>>,
) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    let p = setup(
        release,
        cohort,
        split,
        "validation",
        None,
        c.experts,
        c.source,
    )?;
    let rows: Vec<Value> = p
        .views
        .par_iter()
        .map(|v| {
            let (val, _) = objective(&p, &p.release.params, &[v])?;
            Ok(json!({"run_id": v.run_id(), "group_id": v.group_id(), "conditions": conditions(v), "objective": val}))
        })
        .collect::<anyhow::Result<_>>()?;
    let keys: Vec<String> = by.unwrap_or_else(|| {
        let mut k: Vec<String> = rows
            .iter()
            .flat_map(|r| r["conditions"].as_object().unwrap().keys().cloned())
            .collect();
        k.sort();
        k.dedup();
        k
    });
    let mut groups: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for r in &rows {
        let g = keys
            .iter()
            .map(|k| {
                format!(
                    "{k}={}",
                    r["conditions"]
                        .get(k)
                        .and_then(Value::as_f64)
                        .map(py_float)
                        .unwrap_or_else(|| "None".into())
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        groups
            .entry(g)
            .or_default()
            .push(r["objective"].as_f64().unwrap_or(f64::NAN));
    }
    let summary: Vec<Value> = groups
        .iter()
        .map(|(g, ob)| json!({"condition": g, "n_runs": ob.len(), "objective_mean": crate::util::mean(ob)}))
        .collect();
    Ok(
        json!({"release": p.release.release.id, "partition": "validation", "n_views": p.views.len(), "group_by": keys,
              "warnings": p.warnings, "by_condition": summary, "runs": rows}),
    )
}

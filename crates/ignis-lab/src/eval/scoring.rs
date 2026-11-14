//! Score revealed cases: one artifact against its continuation, then batch aggregation.
//!
//! A *case* is one sealed artifact (run × prefix). Per-case values are aggregated with equal weight
//! per independent cluster (see [`super::bootstrap`]). Every case that passed the prospective support
//! gate stays in the denominator; unscorable ones carry a reason.

use std::collections::{BTreeMap, BTreeSet};

use ignis_core::contracts::{AnalysisArtifact, QualificationState};
use rayon::prelude::*;
use serde_json::{json, Map, Value};

use super::bootstrap::{cluster_bootstrap, clusters};
use super::scores as S;
use super::vault::Revealed;
use crate::util::{fmt_g, interp, mat_f64, mean, vec_f64};

const NON_CAUSE: [&str; 4] = ["active_supported", "active", "survival", "unresolved"];

fn enum_str<T: serde::Serialize>(x: &T) -> String {
    match serde_json::to_value(x) {
        Ok(Value::String(s)) => s,
        Ok(v) => v.to_string(),
        Err(_) => String::new(),
    }
}

fn opt_str<T: serde::Serialize>(x: &Option<T>) -> String {
    x.as_ref().map(enum_str).unwrap_or_else(|| "None".into())
}

// ---------------------------------------------------------------------------------- metric spec

struct Caps {
    kind: String,
    levels: Vec<f64>,
    samples: bool,
    predictive: Option<String>,
    causes: Vec<String>,
    times: Vec<f64>,
    unit: String,
    quantity: String,
}

fn capabilities(a: &AnalysisArtifact) -> Caps {
    let d = &a.diagnostics;
    Caps {
        kind: enum_str(&a.kind),
        levels: a.quantile_levels.clone(),
        samples: d.contains_key("samples"),
        predictive: d
            .get("predictive")
            .and_then(|p| p.get("family"))
            .and_then(Value::as_str)
            .map(String::from),
        causes: a
            .event_categories
            .iter()
            .filter(|c| !NON_CAUSE.contains(&c.as_str()))
            .cloned()
            .collect(),
        times: a.times_s.clone(),
        unit: a.unit.clone().unwrap_or_else(|| "None".into()),
        quantity: opt_str(&a.quantity),
    }
}

/// Metric definitions every admitted artifact in the set supports (so scores are like-for-like).
#[derive(Debug, Clone)]
pub struct Spec {
    pub kinds: Vec<String>,
    pub levels: Vec<f64>,
    pub crps: Option<String>,
    pub lpd: Option<String>,
    pub causes: Vec<String>,
    pub horizons: Vec<f64>,
    pub units: Option<Vec<String>>,
    pub quantities: Option<Vec<String>>,
}

impl Spec {
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("kinds".into(), json!(self.kinds));
        m.insert("levels".into(), json!(self.levels));
        m.insert("crps".into(), json!(self.crps));
        m.insert("lpd".into(), json!(self.lpd));
        m.insert("causes".into(), json!(self.causes));
        m.insert("horizons".into(), json!(self.horizons));
        if let Some(u) = &self.units {
            m.insert("units".into(), json!(u));
        }
        if let Some(q) = &self.quantities {
            m.insert("quantities".into(), json!(q));
        }
        Value::Object(m)
    }
}

fn intersect_f64(sets: &[&Vec<f64>]) -> Vec<f64> {
    let mut out: Vec<f64> = sets[0].clone();
    out.retain(|x| sets.iter().all(|s| s.contains(x)));
    out.sort_by(|a, b| a.partial_cmp(b).unwrap());
    out.dedup();
    out
}

pub fn common_spec(artifacts: &[&AnalysisArtifact], horizons: Option<&[f64]>) -> Spec {
    let caps: Vec<Caps> = artifacts
        .iter()
        .filter(|a| admitted(a).0)
        .map(|a| capabilities(a))
        .collect();
    if caps.is_empty() {
        return Spec {
            kinds: vec![],
            levels: vec![],
            crps: None,
            lpd: None,
            causes: vec![],
            horizons: vec![],
            units: None,
            quantities: None,
        };
    }
    let kinds: BTreeSet<String> = caps.iter().map(|c| c.kind.clone()).collect();
    let pf: Vec<&Caps> = caps
        .iter()
        .filter(|c| c.kind == "prefix_forecast")
        .collect();
    let level_sets: Vec<&Vec<f64>> = pf.iter().map(|c| &c.levels).collect();
    let levels = if level_sets.is_empty() {
        vec![]
    } else {
        intersect_f64(&level_sets)
    };
    let samples = pf.iter().all(|c| c.samples);
    let preds: BTreeSet<Option<String>> = pf.iter().map(|c| c.predictive.clone()).collect();
    let ev: Vec<&Caps> = caps
        .iter()
        .filter(|c| c.kind == "event_probabilities")
        .collect();
    let causes: Vec<String> = if ev.is_empty() {
        vec![]
    } else {
        let mut s: BTreeSet<String> = ev[0].causes.iter().cloned().collect();
        for c in &ev[1..] {
            let o: BTreeSet<String> = c.causes.iter().cloned().collect();
            s = s.intersection(&o).cloned().collect();
        }
        s.into_iter().collect()
    };
    let horizons: Vec<f64> = match horizons {
        Some(h) => h.to_vec(),
        None if ev.is_empty() => vec![],
        None => intersect_f64(&ev.iter().map(|c| &c.times).collect::<Vec<_>>()),
    };
    let crps = if samples && !level_sets.is_empty() {
        Some("samples".to_string())
    } else if !levels.is_empty() {
        Some("quantiles".to_string())
    } else {
        None
    };
    let lpd = if preds.len() == 1 && !preds.contains(&None) {
        Some("exact".to_string())
    } else if levels.len() >= 2 {
        Some("quantile_piecewise".to_string())
    } else {
        None
    };
    Spec {
        kinds: kinds.into_iter().collect(),
        levels,
        crps,
        lpd,
        causes,
        horizons,
        units: Some(
            caps.iter()
                .map(|c| c.unit.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        ),
        quantities: Some(
            caps.iter()
                .map(|c| c.quantity.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        ),
    }
}

fn admitted(a: &AnalysisArtifact) -> (bool, Option<String>) {
    if !a.support.admitted {
        return (
            false,
            Some(format!(
                "support gate {}: {}",
                enum_str(&a.support.gate),
                a.support.reasons.join("; ")
            )),
        );
    }
    if matches!(
        a.status,
        QualificationState::Unsupported
            | QualificationState::Failed
            | QualificationState::InsufficientInference
    ) {
        return (false, Some(format!("status {}", enum_str(&a.status))));
    }
    (true, None)
}

// --------------------------------------------------------------------------------- single case

#[derive(Debug, Clone)]
pub struct Case {
    pub case_id: String,
    pub run_id: String,
    pub release_id: String,
    pub prefix_hash: String,
    pub partition: Value,
    pub group_id: Option<String>,
    pub session_id: Value,
    pub kind: String,
    pub metrics: BTreeMap<String, f64>,
    pub detail: Value,
    pub admitted: bool,
    pub scored: bool,
    pub reason: Option<String>,
    pub endpoint: Option<S::EventOutcome>,
    pub cif_at_horizons: BTreeMap<String, Vec<f64>>,
    pub cluster: Option<String>,
}

/// rows [k][time] -> [k][len(t)] linear in time; NaN outside the forecast range.
fn interp_row(times: &[f64], row: &[f64], t: &[f64]) -> Vec<f64> {
    t.iter()
        .map(|&x| {
            if times.is_empty() || x < times[0] || x > times[times.len() - 1] {
                f64::NAN
            } else {
                interp(x, times, row)
            }
        })
        .collect()
}

type Metrics = (BTreeMap<String, f64>, Value, Option<String>);

fn forecast_metrics(a: &AnalysisArtifact, truth: &Value, spec: &Spec) -> Metrics {
    let empty = || (BTreeMap::new(), json!({}), None::<String>);
    let obs = truth
        .get("observations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let cutoff = truth.get("cutoff_s").and_then(Value::as_f64);
    let q = a.quantity.as_ref().map(enum_str);
    if a.times_s.is_empty() {
        let (m, d, _) = empty();
        return (m, d, Some("artifact has no forecast times".into()));
    }
    let times = &a.times_s;
    let (t0, t1) = (times[0], times[times.len() - 1]);
    let mut t = vec![];
    let mut y = vec![];
    for o in &obs {
        let valid = o.get("valid").and_then(Value::as_bool).unwrap_or(true);
        let Some(v) = o.get("value_si").and_then(Value::as_f64) else {
            continue;
        };
        let oq = o.get("quantity").and_then(Value::as_str);
        let qok = q.is_none() || oq.is_none() || oq == q.as_deref();
        let Some(ts) = o.get("time_s").and_then(Value::as_f64) else {
            continue;
        };
        if valid && qok && cutoff.map_or(true, |c| ts > c) && t0 <= ts && ts <= t1 {
            t.push(ts);
            y.push(v);
        }
    }
    if t.is_empty() {
        let (m, d, _) = empty();
        return (
            m,
            d,
            Some("no valid continuation observation inside the forecast horizon".into()),
        );
    }
    let n = t.len();
    let mut m = BTreeMap::new();
    let detail = json!({"n_points": n});
    let levels = &a.quantile_levels;
    if !levels.is_empty() {
        let qi: Vec<Vec<f64>> = a
            .quantiles
            .iter()
            .map(|r| interp_row(times, r, &t))
            .collect();
        let use_: Vec<usize> = spec
            .levels
            .iter()
            .filter_map(|lv| levels.iter().position(|x| x == lv))
            .collect();
        let lv: Vec<f64> = use_.iter().map(|&i| levels[i]).collect();
        let col = |j: usize| -> Vec<f64> { use_.iter().map(|&i| qi[i][j]).collect() };
        if let Some(mi) = levels.iter().position(|&x| x == 0.5) {
            let e: Vec<f64> = (0..n).map(|j| (qi[mi][j] - y[j]).abs()).collect();
            m.insert("mae_median".into(), mean(&e));
        }
        if spec.crps.as_deref() == Some("quantiles") {
            let v: Vec<f64> = (0..n)
                .map(|j| S::crps_quantiles(&lv, &col(j), y[j]))
                .collect();
            m.insert("crps".into(), mean(&v));
        }
        if spec.lpd.as_deref() == Some("quantile_piecewise") {
            let v: Vec<f64> = (0..n)
                .map(|j| S::lpd_quantiles(&lv, &col(j), y[j]))
                .collect();
            m.insert("lpd".into(), mean(&v));
        }
        let mut cov: Vec<(String, Vec<(bool, f64)>)> = vec![];
        for j in 0..n {
            for iv in S::central_intervals(&lv, &col(j), y[j]) {
                let k = fmt_g(iv.nominal);
                match cov.iter_mut().find(|(kk, _)| *kk == k) {
                    Some((_, v)) => v.push((iv.covered, iv.width)),
                    None => cov.push((k, vec![(iv.covered, iv.width)])),
                }
            }
        }
        for (k, vals) in cov {
            let c: Vec<f64> = vals
                .iter()
                .map(|(c, _)| if *c { 1.0 } else { 0.0 })
                .collect();
            let w: Vec<f64> = vals.iter().map(|(_, w)| *w).collect();
            m.insert(format!("coverage@{k}"), mean(&c));
            m.insert(format!("width@{k}"), mean(&w));
        }
    }
    let d = &a.diagnostics;
    if spec.crps.as_deref() == Some("samples") {
        if let Some(smp) = d.get("samples") {
            let smp = mat_f64(smp);
            let wts = d.get("sample_weights").map(vec_f64);
            let si: Vec<Vec<f64>> = smp.iter().map(|r| interp_row(times, r, &t)).collect();
            let v: Vec<f64> = (0..n)
                .map(|j| {
                    S::crps_samples(
                        &si.iter().map(|r| r[j]).collect::<Vec<_>>(),
                        y[j],
                        wts.as_deref(),
                    )
                })
                .collect();
            m.insert("crps".into(), mean(&v));
        }
    }
    if spec.lpd.as_deref() == Some("exact") {
        if let Some(pr) = d.get("predictive") {
            let loc = vec_f64(&pr["loc"]);
            let sc = vec_f64(&pr["scale"]);
            let fam = pr.get("family").and_then(Value::as_str);
            let lj = |j: usize| interp(t[j], times, &loc);
            let sj = |j: usize| interp(t[j], times, &sc);
            match fam {
                Some("student_t") => {
                    let df = pr.get("df").and_then(Value::as_f64).unwrap_or(f64::NAN);
                    let v: Vec<f64> = (0..n)
                        .map(|j| S::lpd_student_t(y[j], lj(j), sj(j), df))
                        .collect();
                    m.insert("lpd".into(), mean(&v));
                }
                Some("gaussian") => {
                    let v: Vec<f64> = (0..n)
                        .map(|j| S::lpd_gaussian(y[j], lj(j), sj(j)))
                        .collect();
                    m.insert("lpd".into(), mean(&v));
                }
                _ => {}
            }
        }
    }
    if !a.mean.is_empty() {
        let e: Vec<f64> = (0..n)
            .map(|j| (interp(t[j], times, &a.mean) - y[j]).abs())
            .collect();
        m.insert("mae_mean".into(), mean(&e));
    }
    (m, detail, None)
}

fn front_metrics(a: &AnalysisArtifact, truth: &Value) -> Metrics {
    let d = &a.diagnostics;
    let fail = |r: &str| (BTreeMap::new(), json!({}), Some(r.to_string()));
    let Some(fmean) = d.get("front_mean_m") else {
        return fail("artifact carries no front prediction");
    };
    let fr = match truth.get("front") {
        Some(f) if f.is_object() && !f.as_object().unwrap().is_empty() => f,
        _ => return fail("continuation has no front observation"),
    };
    let os = vec_f64(&fr["s_m"]);
    let ps = d
        .get("front_s_m")
        .map(vec_f64)
        .unwrap_or_else(|| os.clone());
    if ps.len() != os.len()
        || ps
            .iter()
            .zip(&os)
            .any(|(a, b)| !((a - b).abs() <= 1e-8 + 1e-5 * b.abs()))
    {
        return fail("predicted and observed fronts use different transverse grids");
    }
    let pt = d
        .get("front_t_s")
        .map(vec_f64)
        .unwrap_or_else(|| a.times_s.clone());
    let ot = vec_f64(&fr["t_s"]);
    let pf = mat_f64(fmean);
    let of = mat_f64(&fr["f_m"]);
    if pt.is_empty() {
        return fail("no observed frame inside the forecast horizon");
    }
    let inside: Vec<usize> = (0..ot.len())
        .filter(|&i| ot[i] >= pt[0] && ot[i] <= pt[pt.len() - 1])
        .collect();
    if inside.is_empty() {
        return fail("no observed frame inside the forecast horizon");
    }
    let ns = pf.first().map_or(0, |r| r.len());
    let pi: Vec<Vec<f64>> = inside
        .iter()
        .map(|&i| {
            (0..ns)
                .map(|k| interp(ot[i], &pt, &pf.iter().map(|r| r[k]).collect::<Vec<_>>()))
                .collect()
        })
        .collect();
    let oi: Vec<Vec<f64>> = inside.iter().map(|&i| of[i].clone()).collect();
    let ti: Vec<f64> = inside.iter().map(|&i| ot[i]).collect();
    let res = match S::front_errors(&pi, &oi, Some(&os), Some(&ti)) {
        Ok(r) => r,
        Err(e) => return fail(&e.to_string()),
    };
    if res.get("n_valid").and_then(Value::as_u64).unwrap_or(0) == 0 {
        let why = res
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("no valid front samples")
            .to_string();
        return (BTreeMap::new(), res, Some(why));
    }
    let mut m = BTreeMap::new();
    for k in ["mae_m", "rmse_m", "bias_m"] {
        if let Some(v) = res.get(k).and_then(Value::as_f64) {
            m.insert(k.to_string(), v);
        }
    }
    (m, res, None)
}

pub fn score_case(a: &AnalysisArtifact, revealed: &Revealed, spec: &Spec) -> Case {
    let truth = &revealed.continuation;
    let seal = &revealed.seal;
    let mut case = Case {
        case_id: a.id.clone(),
        run_id: seal["run_id"].as_str().unwrap_or_default().to_string(),
        release_id: a.model_release_id.clone(),
        prefix_hash: a.prefix_hash.clone(),
        partition: seal["sealed"]["split_partition"].clone(),
        group_id: revealed.group_id.clone(),
        session_id: truth.get("session_id").cloned().unwrap_or(Value::Null),
        kind: enum_str(&a.kind),
        metrics: BTreeMap::new(),
        detail: json!({}),
        admitted: false,
        scored: false,
        reason: None,
        endpoint: None,
        cif_at_horizons: BTreeMap::new(),
        cluster: None,
    };
    let (ok, why) = admitted(a);
    case.admitted = ok;
    if !ok {
        case.reason = why;
        return case;
    }
    let to = truth
        .get("time_origin")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    let ao = a
        .diagnostics
        .get("time_origin")
        .and_then(Value::as_str)
        .or(a.time_origin.as_deref())
        .filter(|s| !s.is_empty());
    if let (Some(to), Some(ao)) = (to, ao) {
        if to != ao {
            case.reason = Some(format!("time origin differs: artifact {ao}, truth {to}"));
            return case;
        }
    }
    let (m, det, why) = match case.kind.as_str() {
        "prefix_forecast" | "metrology" => {
            if a.diagnostics
                .get("front_mean_m")
                .is_some_and(|v| !v.is_null())
            {
                front_metrics(a, truth)
            } else {
                forecast_metrics(a, truth, spec)
            }
        }
        "event_probabilities" => {
            let mut det = Map::new();
            let mut why = None;
            match truth
                .get("endpoint")
                .filter(|e| e.as_object().is_some_and(|o| !o.is_empty()))
            {
                None => why = Some("continuation has no endpoint".to_string()),
                Some(ep) => {
                    case.endpoint = Some(S::EventOutcome {
                        censoring: ep
                            .get("censoring")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown")
                            .to_string(),
                        t_lower: ep.get("t_lower_s").and_then(Value::as_f64),
                        t_upper: ep.get("t_upper_s").and_then(Value::as_f64),
                        causes: ep
                            .get("allowed_causes")
                            .and_then(Value::as_array)
                            .map(|a| {
                                a.iter()
                                    .filter_map(|c| c.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    });
                    let h = &spec.horizons;
                    let row_of = |c: &str| -> Vec<f64> {
                        match a.event_categories.iter().position(|x| x == c) {
                            Some(i) if !a.times_s.is_empty() => {
                                interp_row(&a.times_s, &a.event_cdf[i], h)
                            }
                            _ => vec![f64::NAN; h.len()],
                        }
                    };
                    for c in &spec.causes {
                        case.cif_at_horizons.insert(c.clone(), row_of(c));
                    }
                    if a.event_categories.iter().any(|c| c == "unresolved") && !a.times_s.is_empty()
                    {
                        det.insert(
                            "unresolved_mass_at_horizons".into(),
                            json!(row_of("unresolved")),
                        );
                    }
                }
            }
            (BTreeMap::new(), Value::Object(det), why)
        }
        k => (
            BTreeMap::new(),
            json!({}),
            Some(format!("kind {k} has no scorer")),
        ),
    };
    case.metrics = m;
    case.detail = det;
    case.scored = why.is_none();
    case.reason = why;
    case
}

// --------------------------------------------------------------------------------------- batch

fn event_block(cases: &mut [Case], spec: &Spec) -> Value {
    let ev: Vec<usize> = (0..cases.len())
        .filter(|&i| cases[i].admitted && cases[i].endpoint.is_some())
        .collect();
    if ev.is_empty() {
        return json!({});
    }
    let outs: Vec<S::EventOutcome> = ev
        .iter()
        .map(|&i| cases[i].endpoint.clone().unwrap())
        .collect();
    let h = &spec.horizons;
    let mut block = Map::new();
    block.insert("horizons".into(), json!(h));
    block.insert(
        "estimator".into(),
        json!("IPCW Brier, KM censoring (Graf/Gerds)"),
    );
    let mut causes = Map::new();
    let mut any_event = vec![vec![0.0; h.len()]; ev.len()];
    for cause in &spec.causes {
        let f: Vec<Vec<f64>> = ev
            .iter()
            .map(|&i| cases[i].cif_at_horizons[cause].clone())
            .collect();
        for (r, fr) in any_event.iter_mut().zip(&f) {
            for (x, y) in r.iter_mut().zip(fr) {
                *x += y;
            }
        }
        let mut per_h = vec![];
        let mut contrib = vec![vec![f64::NAN; h.len()]; ev.len()];
        let mut cal = vec![];
        for (j, &hj) in h.iter().enumerate() {
            let col: Vec<f64> = f.iter().map(|r| r[j]).collect();
            per_h.push(S::ipcw_brier(&col, &outs, Some(cause), hj));
            let t = S::ipcw_terms(&outs, Some(cause), hj);
            for i in 0..ev.len() {
                if t.eligible[i] && col[i].is_finite() {
                    contrib[i][j] = t.weight[i] * (t.indicator[i] - col[i]).powi(2);
                }
            }
            let mut c = Map::new();
            c.insert("horizon".into(), json!(hj));
            if let Value::Object(tc) = S::time_calibration(&col, &outs, Some(cause), hj, 4) {
                c.extend(tc);
            }
            cal.push(Value::Object(c));
        }
        let briers: Vec<f64> = per_h
            .iter()
            .map(|p| p["brier"].as_f64().unwrap_or(f64::NAN))
            .collect();
        let ib = S::integrated_brier(h, &briers);
        causes.insert(
            cause.clone(),
            json!({"brier": per_h, "integrated_brier": ib, "calibration": cal}),
        );
        for (k, &i) in ev.iter().enumerate() {
            let row = &contrib[k];
            let v = if h.len() > 1 {
                S::integrated_brier(h, row)
            } else if h.len() == 1 {
                row[0]
            } else {
                f64::NAN
            };
            cases[i].metrics.insert(format!("ibs_contrib[{cause}]"), v);
        }
    }
    block.insert("causes".into(), Value::Object(causes));
    if !spec.causes.is_empty() {
        let any: Vec<Value> = h
            .iter()
            .enumerate()
            .map(|(j, &hj)| {
                S::ipcw_brier(
                    &any_event.iter().map(|r| r[j]).collect::<Vec<_>>(),
                    &outs,
                    None,
                    hj,
                )
            })
            .collect();
        block.insert("any_event".into(), json!(any));
    }
    for &i in &ev {
        cases[i].scored = cases[i].reason.is_none();
    }
    Value::Object(block)
}

pub fn aggregate(cases: &mut [Case], spec: &Spec, n_boot: usize, seed: u64) -> Value {
    let events = event_block(cases, spec);
    let labels = clusters(
        &cases.iter().map(|c| c.run_id.clone()).collect::<Vec<_>>(),
        &cases.iter().map(|c| c.group_id.clone()).collect::<Vec<_>>(),
        &cases
            .iter()
            .map(|c| c.session_id.as_str().map(String::from))
            .collect::<Vec<_>>(),
    );
    for (c, l) in cases.iter_mut().zip(&labels) {
        c.cluster = Some(l.clone());
    }
    let scored: Vec<&Case> = cases.iter().filter(|c| c.scored).collect();
    let names: BTreeSet<String> = scored
        .iter()
        .flat_map(|c| c.metrics.keys().cloned())
        .collect();
    let metrics: Map<String, Value> = names
        .into_par_iter()
        .map(|k| {
            let vals: Vec<f64> = scored
                .iter()
                .map(|c| *c.metrics.get(&k).unwrap_or(&f64::NAN))
                .collect();
            let labs: Vec<String> = scored
                .iter()
                .map(|c| c.cluster.clone().unwrap_or_default())
                .collect();
            let b = cluster_bootstrap(&vals, &labs, n_boot, seed, 0.95);
            (k, b)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .collect();
    let acc = S::refusal_accounting(
        &cases
            .iter()
            .map(|c| S::AccountingCase {
                case_id: c.case_id.clone(),
                admitted: c.admitted,
                scored: c.scored,
                reason: c.reason.clone(),
            })
            .collect::<Vec<_>>(),
    );
    let n_clusters = labels.iter().collect::<BTreeSet<_>>().len();
    json!({"spec": spec.to_value(), "accounting": acc, "metrics": metrics, "events": events, "n_clusters": n_clusters})
}

/// Score revealed cases in parallel (reveals themselves happen before, sequentially and logged).
pub fn score_all(revealed: &[Revealed], spec: &Spec) -> Vec<Case> {
    revealed
        .par_iter()
        .map(|r| score_case(&r.artifact, r, spec))
        .collect()
}

pub fn public_case(c: &Case) -> Value {
    let mut m = Map::new();
    m.insert("case_id".into(), json!(c.case_id));
    m.insert("run_id".into(), json!(c.run_id));
    m.insert("release_id".into(), json!(c.release_id));
    m.insert("prefix_hash".into(), json!(c.prefix_hash));
    m.insert("partition".into(), c.partition.clone());
    m.insert("group_id".into(), json!(c.group_id));
    m.insert("session_id".into(), c.session_id.clone());
    m.insert("kind".into(), json!(c.kind));
    m.insert("metrics".into(), json!(c.metrics));
    m.insert("detail".into(), c.detail.clone());
    m.insert("admitted".into(), json!(c.admitted));
    m.insert("scored".into(), json!(c.scored));
    m.insert("reason".into(), json!(c.reason));
    if let Some(e) = &c.endpoint {
        m.insert("endpoint".into(), json!({"censoring": e.censoring, "t_lower_s": e.t_lower, "t_upper_s": e.t_upper, "causes": e.causes}));
    }
    if let Some(cl) = &c.cluster {
        m.insert("cluster".into(), json!(cl));
    }
    Value::Object(m)
}

//! ACME experts: two configuration-specific instances (normal, inverse) of the hazard model and its
//! baselines, implementing `ignis_core::expert::Expert`.
//!
//! Interpretation (V3.1 §8.1, §8.6): curves are *observational event predictions under the observed
//! experimental policy*. Operator terminations are not spontaneous extinction; the counterfactual
//! "natural lifetime without the operator" estimand is DISABLED and refused until intervention
//! histories, causal assumptions and overlap are identified separately.
//!
//! Scheduled stop: taken from `PrefixBundle.known_controls` (a commanded flow stop: an operator
//! action named "MFCs off…", or a stage named `scheduled_stop`/`flow_stop`), or from the explicit
//! `scheduled_stop_s` setting. Session id: `PrefixBundle.session_id` (or `session_id` setting).

use std::collections::BTreeMap;
use std::time::Instant;

use anyhow::{bail, Context};
use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, ControlStage, Family, ModelRelease, PrefixBundle, RawValue,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::{ProspectivePolicy, ALWAYS_FORBIDDEN};
use ignis_core::registry::{save_release, LoadedRelease, Tensor, TensorMap};
use ignis_core::resources::Profile;
use ignis_core::views::{AcmeEventRow, View, ACME_CAUSES};
use rand::{rngs::StdRng, RngExt, SeedableRng};
use rand_distr::StandardNormal;
use rayon::prelude::*;
use serde_json::{json, Value};

use crate::common::{self, deep_merge, obj};
use crate::features::{self as feat, FeatureSpec, Norm};
use crate::model::{self as m, AcmeSpec, Encoded, Kind, Layout, Quad};
use crate::optim::lbfgs;
use crate::sampler::{nuts, NutsConfig};

pub const CATEGORIES: [&str; 4] = [
    "active_supported",
    "extinction",
    "operator_termination",
    "unresolved_support",
];
pub const INTERPRETATION: &str = "observational event prediction under the observed experimental policy; operator termination is not spontaneous extinction";
pub const COUNTERFACTUAL_STATUS: &str = "disabled: the natural-lifetime-without-operator estimand needs intervention histories, causal assumptions, overlap and sensitivity analysis (V3.1 §8.1); not identified";
pub const VARIANTS: [&str; 4] = [
    "hazard",
    "constant_baseline",
    "piecewise_baseline",
    "linear_baseline",
];

fn defaults() -> Value {
    json!({
        "bin_lower_edges_s": [0.0, 5.0, 10.0, 20.0, 40.0, 80.0],
        "t_ref_s": 10.0, "hidden": 8, "cells_per_bin": 8,
        "operator_hazard": "auto",
        "train_quadrature_nodes": 12, "forecast_quadrature_nodes": 24,
        "inference": "nuts_if_qualified",
        "map": {"maxiter": 2000, "seed": 0},
        "nuts": {"num_warmup": 500, "num_samples": 500, "num_chains": 4, "target_accept": 0.85, "seed": 0,
                 "max_rhat": 1.01, "min_ess": 400.0, "max_divergences": 0},
        "priors": {"rate_ref_per_s": 0.01, "a_sd": 1.5, "beta_sd": 0.5, "b1_sd": 0.1, "b2_sd": 0.1,
                   "sigma_halfnormal": 0.5, "lkj_eta": 2.0},
        "support": {"min_neighbors": 3, "min_groups": 2, "radius_percentile": 95.0},
        "features": [],
        "trained_on": "nasa",
    })
}

fn variant_overrides(v: &str) -> Value {
    match v {
        "hazard" => json!({"linear": true, "neural": true, "frailty": true}),
        "constant_baseline" => {
            json!({"linear": false, "neural": false, "frailty": false, "bin_lower_edges_s": [0.0], "inference": "map"})
        }
        "piecewise_baseline" => {
            json!({"linear": false, "neural": false, "frailty": false, "inference": "map"})
        }
        "linear_baseline" => {
            json!({"linear": true, "neural": false, "frailty": false, "inference": "map"})
        }
        _ => Value::Null,
    }
}

pub struct AcmeExpert {
    pub configuration: String,
    pub variant: String,
    family: Family,
    name: String,
    baseline: bool,
    policy: ProspectivePolicy,
}

pub fn features_of(cfg: &Value) -> anyhow::Result<Vec<FeatureSpec>> {
    Ok(serde_json::from_value(cfg["features"].clone())?)
}

fn f(cfg: &Value, path: &[&str], d: f64) -> f64 {
    let mut v = cfg;
    for p in path {
        v = &v[*p];
    }
    v.as_f64().unwrap_or(d)
}

fn spec_from_cfg(cfg: &Value, causes: Vec<String>, n_features: usize) -> AcmeSpec {
    AcmeSpec {
        causes,
        bin_lower_edges_s: cfg["bin_lower_edges_s"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_f64()).collect())
            .unwrap_or_default(),
        t_ref_s: f(cfg, &["t_ref_s"], 10.0),
        n_features,
        linear: cfg["linear"].as_bool().unwrap_or(false),
        neural: cfg["neural"].as_bool().unwrap_or(false),
        frailty: cfg["frailty"].as_bool().unwrap_or(false),
        hidden: f(cfg, &["hidden"], 8.0) as usize,
        cells_per_bin: f(cfg, &["cells_per_bin"], 8.0) as usize,
    }
}

pub fn spec_from_release(r: &ModelRelease) -> anyhow::Result<AcmeSpec> {
    let a = Value::Object(r.architecture.clone());
    Ok(AcmeSpec {
        causes: serde_json::from_value(a["causes"].clone())?,
        bin_lower_edges_s: serde_json::from_value(a["bin_lower_edges_s"].clone())?,
        t_ref_s: a["t_ref_s"].as_f64().context("t_ref_s")?,
        n_features: a["n_features"].as_u64().context("n_features")? as usize,
        linear: a["linear"].as_bool().unwrap_or(false),
        neural: a["neural"].as_bool().unwrap_or(false),
        frailty: a["frailty"].as_bool().unwrap_or(false),
        hidden: a["hidden"].as_u64().unwrap_or(8) as usize,
        cells_per_bin: a["cells_per_bin"].as_u64().unwrap_or(8) as usize,
    })
}

/// Commanded flow stop known in the prefix (earliest), if any.
pub fn scheduled_stop_from_controls(controls: &[ControlStage]) -> Option<f64> {
    controls
        .iter()
        .filter(|s| {
            let c = s.control.to_lowercase();
            c == "scheduled_stop"
                || c == "flow_stop"
                || (s.operator_action && c.starts_with("mfcs off"))
        })
        .map(|s| s.t_start_s)
        .fold(None, |a: Option<f64>, b| Some(a.map_or(b, |x| x.min(b))))
}

/// Everything a bound release needs at serving time.
pub struct Bound {
    pub spec: AcmeSpec,
    pub lay: Layout,
    pub cfg: Value,
    pub norm: Norm,
    pub features: Vec<FeatureSpec>,
    pub point: Vec<f64>,
    pub draws: Vec<Vec<f64>>,
}

pub fn bind(r: &LoadedRelease) -> anyhow::Result<Bound> {
    let spec = spec_from_release(&r.release)?;
    let lay = Layout::new(&spec);
    let cfg = Value::Object(r.release.config.clone());
    let point = lay.from_tensors(&r.params)?;
    let draws = if r.posterior.is_empty() {
        vec![point.clone()]
    } else {
        lay.draws_from_tensors(&r.posterior)?
    };
    Ok(Bound {
        features: features_of(&cfg)?,
        spec,
        lay,
        norm: r.release.normalization.clone(),
        cfg,
        point,
        draws,
    })
}

impl AcmeExpert {
    pub fn new(configuration: &str, variant: &str) -> anyhow::Result<AcmeExpert> {
        if !["normal", "inverse"].contains(&configuration) || !VARIANTS.contains(&variant) {
            bail!("unknown ACME expert {configuration}.{variant}");
        }
        let mut e = AcmeExpert {
            configuration: configuration.into(),
            variant: variant.into(),
            family: if configuration == "normal" {
                Family::AcmeNormal
            } else {
                Family::AcmeInverse
            },
            name: format!("acme_{configuration}.{variant}"),
            baseline: variant != "hazard",
            policy: ProspectivePolicy::new(Family::AcmeNormal, &[], feat::ACME_FORBIDDEN),
        };
        let cfg = e.default_config()?;
        let mut allowed: Vec<String> = features_of(&cfg)?.into_iter().map(|f| f.source).collect();
        allowed.push("fuel_species".into()); // known before the run; expands to the fuel_is_* indicators
        let allowed: Vec<&str> = allowed.iter().map(|s| s.as_str()).collect();
        e.policy = ProspectivePolicy::new(e.family, &allowed, feat::ACME_FORBIDDEN);
        Ok(e)
    }

    fn select<'a>(
        &self,
        views: impl IntoIterator<Item = &'a View>,
    ) -> (Vec<&'a AcmeEventRow>, Vec<Value>) {
        let mut rows = vec![];
        let mut excluded = vec![];
        for v in views {
            let View::Acme(r) = v else { continue };
            if r.configuration != self.configuration {
                continue;
            }
            if Kind::from_censoring(&r.censoring).is_none() {
                excluded.push(json!({"run_id": r.run_id, "reason": format!("censoring {} not eligible", r.censoring)}));
                continue;
            }
            rows.push(r);
        }
        (rows, excluded)
    }

    fn with_features<'a>(
        &self,
        feats: &[FeatureSpec],
        rows: Vec<&'a AcmeEventRow>,
    ) -> anyhow::Result<(Vec<&'a AcmeEventRow>, Vec<Value>)> {
        let mut keep = vec![];
        let mut excluded = vec![];
        for r in rows {
            if let Some(l) = feats.iter().find(|f| {
                (feat::ACME_FORBIDDEN.contains(&f.source.as_str())
                    || ALWAYS_FORBIDDEN.contains(&f.source.as_str()))
                    && r.features.contains_key(&f.source)
            }) {
                bail!("forbidden feature {:?} configured as predictor", l.source);
            }
            let miss = feat::missing(feats, &r.features);
            if miss.is_empty() {
                keep.push(r);
            } else {
                excluded.push(json!({"run_id": r.run_id, "reason": format!("missing critical features {miss:?} (cohort restriction)")}));
            }
        }
        Ok((keep, excluded))
    }

    fn causes(&self, cfg: &Value, rows: &[&AcmeEventRow]) -> (Vec<String>, &'static str) {
        let all: Vec<String> = ACME_CAUSES.iter().map(|s| s.to_string()).collect();
        match cfg["operator_hazard"].as_str().unwrap_or("auto") {
            "on" => return (all, "configured on"),
            "off" => {
                return (
                    vec!["extinction".into()],
                    "configured off: operator terminations only as scheduled-stop atoms",
                )
            }
            _ => {}
        }
        for r in rows {
            if !r.allowed_causes.iter().any(|c| c == m::OPERATOR)
                || r.censoring == "right"
                || r.censoring == "scheduled_stop"
            {
                continue;
            }
            let t = r.t_upper_s.or(r.t_lower_s);
            let lo = r.t_lower_s.or(t);
            let explained = matches!((r.scheduled_stop_s, lo, t), (Some(st), Some(lo), Some(t)) if lo - 1e-9 <= st && st <= t + 1e-9);
            if !explained {
                return (all, "auto: unscheduled operator terminations present");
            }
        }
        (
            vec!["extinction".into()],
            "auto: every operator termination is a scheduled stop; continuous O hazard removed",
        )
    }

    fn init_params(&self, spec: &AcmeSpec, lay: &Layout, enc: &Encoded, seed: u64) -> Vec<f64> {
        let mut rng = StdRng::seed_from_u64(seed);
        let exposure: f64 = enc.rows.iter().map(|r| (r.ta - r.entry).max(1e-6)).sum();
        let mut p = vec![0.0; lay.len];
        let k = spec.k();
        for j in 0..k {
            let n_ev = enc
                .rows
                .iter()
                .filter(|r| r.kind != Kind::Right)
                .map(|r| r.cmask[j])
                .sum::<f64>()
                .max(1.0);
            let a = (n_ev / exposure * spec.t_ref_s).ln();
            for b in 0..spec.n_bins() {
                p[lay.a + j * spec.n_bins() + b] = a;
            }
        }
        let mut normal = |sd: f64| sd * rng.sample::<f64, _>(StandardNormal);
        if let Some(w1) = lay.w1 {
            let fan = (spec.n_features + 1) as f64;
            for i in 0..spec.hidden * (spec.n_features + 1) {
                p[w1 + i] = normal(0.5 / fan.sqrt()); // away from the all-zero saddle
            }
        }
        if let Some(w2) = lay.w2 {
            for i in 0..k * spec.hidden {
                p[w2 + i] = normal(0.05);
            }
        }
        if let Some(ls) = lay.log_sigma {
            for j in 0..k {
                p[ls + j] = 0.3f64.ln();
            }
        }
        p
    }

    /// Normalised feature vector of a value map.
    fn normalised(&self, b: &Bound, values: &BTreeMap<String, f64>) -> anyhow::Result<Vec<f64>> {
        Ok(feat::apply_normalization(
            &b.features,
            &b.norm,
            &feat::raw_row(&b.features, values)?,
        ))
    }

    /// Held-out log predictive density of one row for a NEW session (population frailty).
    pub fn row_predictive(
        &self,
        b: &Bound,
        row: &AcmeEventRow,
        n_nodes: usize,
    ) -> anyhow::Result<f64> {
        let c = self.normalised(b, &row.features)?;
        let enc = m::encode_rows(&[row], &[c], &b.spec, Some(&[row.group_id.clone()]))?;
        let q = m::quadrature(&b.spec, n_nodes);
        let per: Vec<f64> = b
            .draws
            .par_iter()
            .map(|p| {
                let ll = m::session_loglik_nodes(p, &b.spec, &b.lay, &enc, &q);
                let v: Vec<f64> = ll.iter().zip(&q.logw).map(|(a, w)| a + w).collect();
                ignis_science::gauss_hermite::logsumexp(&v)
            })
            .collect();
        Ok(ignis_science::gauss_hermite::logsumexp(&per) - (per.len() as f64).ln())
    }
}

fn support_radius(c: &[Vec<f64>], group: &[usize], pct: f64) -> f64 {
    let nn: Vec<f64> = (0..c.len())
        .filter_map(|i| {
            let d = (0..c.len())
                .filter(|&j| group[j] != group[i])
                .map(|j| {
                    c[i].iter()
                        .zip(&c[j])
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>()
                        .sqrt()
                })
                .fold(f64::INFINITY, f64::min);
            d.is_finite().then_some(d)
        })
        .collect();
    if nn.is_empty() {
        0.0
    } else {
        common::percentile(&nn, pct)
    }
}

fn condition_support(c: &[f64], train: &Tensor, group: &Tensor, sup: &Value) -> (bool, String) {
    let f = train.shape.get(1).copied().unwrap_or(c.len());
    let radius = sup["radius"].as_f64().unwrap_or(0.0);
    let mut n = 0;
    let mut groups = std::collections::BTreeSet::new();
    for (i, row) in train.data.chunks(f).enumerate() {
        let d = row
            .iter()
            .zip(c)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        if d <= radius + 1e-12 {
            n += 1;
            groups.insert(group.data[i] as i64);
        }
    }
    let (mn, mg) = (
        sup["min_neighbors"].as_u64().unwrap_or(3) as usize,
        sup["min_groups"].as_u64().unwrap_or(2) as usize,
    );
    let ok = n >= mn && groups.len() >= mg;
    (ok, format!("{n} compatible train neighbours on {} sessions within radius {radius:.3} (need {mn} on {mg}); operational gate, not an OOD probability", groups.len()))
}

/// Records carry the fuel as a species name; the features are indicators against that name (the same
/// expansion the curated views apply). A prefix that already states an indicator keeps its value.
fn fill_fuel_indicators(
    values: &mut BTreeMap<String, f64>,
    features: &[FeatureSpec],
    prefix: &PrefixBundle,
) {
    let Some(RawValue::Text(species)) = prefix
        .known_conditions
        .get("fuel_species")
        .and_then(|m| m.raw_value.as_ref())
    else {
        return;
    };
    for f in features {
        if let Some(fuel) = f.source.strip_prefix("fuel_is_") {
            values
                .entry(f.source.clone())
                .or_insert(f64::from(u8::from(fuel == species)));
        }
    }
}

fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            if i == n - 1 {
                b
            } else {
                a + (b - a) * i as f64 / (n - 1) as f64
            }
        })
        .collect()
}

impl Expert for AcmeExpert {
    fn family(&self) -> Family {
        self.family
    }
    fn model_name(&self) -> &str {
        &self.name
    }
    fn is_baseline(&self) -> bool {
        self.baseline
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }
    fn default_config(&self) -> ExpertResult<Value> {
        let mut cfg = defaults();
        let p = ignis_core::resources::configs_dir()
            .join("models")
            .join("acme")
            .join(format!("{}.toml", self.configuration));
        if let Some(t) = common::load_toml(&p)? {
            cfg = deep_merge(&cfg, &t);
        }
        let own = cfg
            .get("variants")
            .and_then(|v| v.get(&self.variant))
            .cloned()
            .unwrap_or(Value::Null);
        Ok(deep_merge(
            &deep_merge(&cfg, &variant_overrides(&self.variant)),
            &own,
        ))
    }
    fn estimate_hours(
        &self,
        config: &Value,
        n_runs: usize,
        _n_obs: usize,
        profile: &Profile,
    ) -> f64 {
        let nodes = f(config, &["train_quadrature_nodes"], 12.0);
        // measured order of one value+gradient on 32 CPU threads, scaled to the profile's threads
        let grad_s = 2e-7 * n_runs as f64 * nodes * nodes * 32.0 / profile.threads().max(1) as f64;
        let iters = f(config, &["map", "maxiter"], 2000.0);
        let nuts = if config["inference"] == "nuts_if_qualified" && !self.baseline {
            f(config, &["nuts", "num_chains"], 4.0)
                * (f(config, &["nuts", "num_warmup"], 500.0)
                    + f(config, &["nuts", "num_samples"], 500.0))
                * 31.0
                / 8.0
        } else {
            0.0
        };
        (iters * 1.5 + nuts) * grad_s / 3600.0
    }

    fn fit(
        &self,
        data: &FitData<'_>,
        config: &Value,
        ctx: &mut FitContext,
    ) -> ExpertResult<ModelRelease> {
        let t0 = Instant::now();
        let cfg = deep_merge(&self.default_config()?, config);
        let feats = features_of(&cfg)?;
        let (rows, excl) = self.select(data.partition("train"));
        let (rows, excl2) = self.with_features(&feats, rows)?;
        if rows.len() < 2 {
            bail!("{}: fewer than two eligible train rows", self.name);
        }
        let (causes, cause_note) = self.causes(&cfg, &rows);
        let spec = spec_from_cfg(&cfg, causes.clone(), feats.len());
        let lay = Layout::new(&spec);
        let x: Vec<Vec<f64>> = rows
            .iter()
            .map(|r| feat::raw_row(&feats, &r.features))
            .collect::<anyhow::Result<_>>()?;
        let norm = feat::fit_normalization(&feats, &x);
        let c: Vec<Vec<f64>> = x
            .iter()
            .map(|r| feat::apply_normalization(&feats, &norm, r))
            .collect();
        let enc = m::encode_rows(&rows, &c, &spec, None)?;
        let n_train = f(&cfg, &["train_quadrature_nodes"], 12.0) as usize;
        let q = m::quadrature(&spec, n_train);
        let pr = m::prior_settings(&spec, &cfg);
        let seed = cfg["map"]["seed"].as_u64().unwrap_or(0) ^ ctx.seed;
        let p0 = match &ctx.resume {
            Some(r) => lay.from_tensors(&r.params)?,
            None => self.init_params(&spec, &lay, &enc, seed),
        };
        let neg = |x: &[f64], g: &mut [f64]| {
            let v = m::log_posterior(x, &spec, &lay, &pr, &enc, &q, Some(g));
            g.iter_mut().for_each(|v| *v = -*v);
            -v
        };
        let maxiter = f(&cfg, &["map", "maxiter"], 2000.0) as usize;
        let t_map = Instant::now();
        let res = ctx.pool.install(|| lbfgs(neg, &p0, maxiter, 10));
        let mut inference = json!({"method": "map", "converged": res.converged, "neg_log_posterior": res.f, "iterations": res.iterations,
            "gradient_evaluations": res.evaluations, "message": res.message, "map_seconds": t_map.elapsed().as_secs_f64()});
        let point = res.x.clone();
        let mut posterior: Vec<Vec<f64>> = vec![];
        if cfg["inference"] == "nuts_if_qualified" && !self.baseline {
            let ncfg = NutsConfig::from_json(&cfg["nuts"]);
            let lp =
                |x: &[f64], g: &mut [f64]| m::log_posterior(x, &spec, &lay, &pr, &enc, &q, Some(g));
            let (draws, diag) = ctx.pool.install(|| nuts(&lp, &point, &ncfg));
            let qualified = diag["qualified"].as_bool().unwrap_or(false);
            inference["nuts"] = diag;
            if qualified {
                posterior = draws;
                inference["method"] = json!("nuts");
            } else {
                inference["note"] =
                    json!("NUTS not qualified; MAP release (posterior_valid_for_parameters=false)");
            }
        }
        let draws = if posterior.is_empty() {
            vec![point.clone()]
        } else {
            posterior.clone()
        };
        let g = enc.n_sessions();
        let tables: Vec<Vec<f64>> = ctx.pool.install(|| {
            draws
                .iter()
                .map(|p| m::session_loglik_nodes(p, &spec, &lay, &enc, &q))
                .collect()
        });
        let mut params = lay.to_tensors(&point);
        params.insert(
            "derived.session_loglik_nodes".into(),
            Tensor::new(vec![draws.len(), g, q.n()], tables.concat()),
        );
        params.insert(
            "support.train_features".into(),
            Tensor::new(vec![c.len(), feats.len()], c.concat()),
        );
        let groups: Vec<usize> = enc.rows.iter().map(|r| r.session).collect();
        params.insert(
            "support.train_group".into(),
            Tensor::vector(groups.iter().map(|&x| x as f64).collect()),
        );
        let sup = &cfg["support"];
        let radius = support_radius(&c, &groups, f(sup, &["radius_percentile"], 95.0));
        let horizon = enc
            .rows
            .iter()
            .map(|r| r.tb.max(if r.stop.is_finite() { r.stop } else { 0.0 }))
            .fold(0.0, f64::max);
        let mut rel = common::release(
            self.family,
            &self.name,
            self.baseline,
            feats.iter().map(|f| f.name.clone()).collect(),
            data.cohort_id,
            &data.split.id,
            data.source_manifest_hash,
            data.synthetic || cfg["trained_on"] == "synthetic",
        );
        rel.architecture = obj(
            json!({"kind": "acme_additive_hazard", "variant": self.variant, "causes": causes, "cause_note": cause_note,
            "bin_lower_edges_s": spec.bin_lower_edges_s, "last_bin": "[e_last, inf)", "t_ref_s": spec.t_ref_s, "n_features": spec.n_features,
            "linear": spec.linear, "neural": spec.neural, "frailty": spec.frailty, "hidden": spec.hidden, "cells_per_bin": spec.cells_per_bin,
            "network": if spec.neural { json!(format!("{}->{} tanh->{}", spec.n_features + 1, spec.hidden, spec.k())) } else { Value::Null },
            "n_population_parameters": m::n_params(&spec), "session_ids": enc.session_ids, "train_run_ids": enc.rows.iter().map(|r| r.run_id.clone()).collect::<Vec<_>>()}),
        );
        rel.config = obj(cfg.clone());
        rel.units = feats
            .iter()
            .map(|f| {
                (
                    f.name.clone(),
                    if f.transform == "log_ratio" {
                        format!("log(x/{})", f.reference.unwrap_or(1.0))
                    } else {
                        "1".into()
                    },
                )
            })
            .collect();
        rel.units.insert("time".into(), "s".into());
        rel.normalization = norm.clone();
        rel.priors = obj(
            json!({"settings": pr, "frailty": "u_g ~ N(0, diag(s) Omega diag(s)); s ~ HalfNormal; Omega ~ LKJ"}),
        );
        rel.observation_model = obj(
            json!({"event_likelihood": "exact/interval/right/scheduled-stop atom, left-truncated at entry",
            "known_cause_set": "sum of cause-specific incidences over the admitted set",
            "scheduled_stop": "probability atom S(stop-)/S(entry); post-stop survival 0",
            "frailty_integration": format!("Gauss-Hermite {n_train}^K (train)")}),
        );
        let mut support = sup.clone();
        support["radius"] = json!(radius);
        rel.applicability = obj(
            json!({"configuration": self.configuration, "horizon_s": horizon, "support": support,
            "estimands": {"observational": INTERPRETATION, "counterfactual": COUNTERFACTUAL_STATUS},
            "excluded_runs": excl.into_iter().chain(excl2).collect::<Vec<_>>()}),
        );
        rel.posterior_valid_for_parameters = !posterior.is_empty();
        let bound = Bound {
            spec: spec.clone(),
            lay: lay.clone(),
            cfg: cfg.clone(),
            norm,
            features: feats.clone(),
            point: point.clone(),
            draws,
        };
        let (vrows, _) = self.select(data.partition("validation"));
        let (vrows, _) = self.with_features(&feats, vrows)?;
        if !vrows.is_empty() {
            let nf = f(&cfg, &["forecast_quadrature_nodes"], 24.0) as usize;
            let lls: Vec<f64> = ctx.pool.install(|| {
                vrows
                    .iter()
                    .map(|r| self.row_predictive(&bound, r, nf))
                    .collect::<anyhow::Result<_>>()
            })?;
            rel.evaluation = obj(
                json!({"validation_mean_log_lik_new_session": lls.iter().sum::<f64>() / lls.len() as f64, "n_validation": lls.len()}),
            );
        }
        rel.numerics = obj(
            json!({"x64": true, "cells_per_bin": spec.cells_per_bin, "inference": inference, "train_quadrature_nodes": n_train,
            "forecast_quadrature_nodes": cfg["forecast_quadrature_nodes"], "fit_seconds": t0.elapsed().as_secs_f64(), "implementation": "rust analytic gradients"}),
        );
        let post = if posterior.is_empty() {
            TensorMap::new()
        } else {
            lay.draws_to_tensors(&posterior)
        };
        rel.posterior_parameters = if posterior.is_empty() {
            vec![]
        } else {
            lay.blocks.iter().map(|b| b.0.clone()).collect()
        };
        let frozen = vec![
            "derived.session_loglik_nodes".into(),
            "support.train_features".into(),
            "support.train_group".into(),
        ];
        save_release(&ctx.registry_root, rel, &params, &post, &frozen)
    }

    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact> {
        let rel = &release.release;
        let estimand = settings["estimand"].as_str().unwrap_or("observational");
        if estimand != "observational" {
            return Err(Refusal::unsupported(
                "counterfactual_disabled",
                format!("estimand {estimand:?}: {COUNTERFACTUAL_STATUS}"),
            )
            .into());
        }
        if prefix.family != rel.family {
            return Err(Refusal::unsupported(
                "family_mismatch",
                format!(
                    "{:?} prefix cannot be forecast by the {:?} model",
                    prefix.family, rel.family
                ),
            )
            .into());
        }
        let Some(t_entry) = prefix.phase_established_at_s else {
            return Err(Refusal::unsupported(
                "phase_not_established",
                "established phase is not confirmed at the cutoff (population criterion)",
            )
            .into());
        };
        let b = bind(release)?;
        let mut values = common::condition_values(prefix);
        fill_fuel_indicators(&mut values, &b.features, prefix);
        let miss = feat::missing(&b.features, &values);
        if !miss.is_empty() {
            return Err(Refusal::unsupported(
                "missing_features",
                format!("missing critical prospective features {miss:?}; no imputation"),
            )
            .into());
        }
        let c = self.normalised(&b, &values)?;
        let app = Value::Object(rel.applicability.clone());
        let (ok, why) = condition_support(
            &c,
            &release.params["support.train_features"],
            &release.params["support.train_group"],
            &app["support"],
        );
        if !ok
            && !settings["override_condition_support"]
                .as_bool()
                .unwrap_or(false)
        {
            return Err(Refusal::unsupported("condition_support", why).into());
        }
        let tau = prefix.cutoff_s;
        let horizon = app["horizon_s"].as_f64().context("horizon_s")?;
        if tau >= horizon {
            return Err(Refusal::unsupported(
                "horizon",
                format!("cutoff {tau} s beyond the fitted support horizon {horizon} s"),
            )
            .into());
        }
        let stop = settings["scheduled_stop_s"]
            .as_f64()
            .or_else(|| scheduled_stop_from_controls(&prefix.known_controls))
            .unwrap_or(f64::INFINITY);
        if stop <= tau {
            return Err(Refusal::unsupported(
                "fed_phase_ended",
                "scheduled stop at or before the cutoff: the fed phase has ended",
            )
            .into());
        }
        let mut times: Vec<f64> = match settings["times_s"].as_array() {
            Some(a) => a.iter().filter_map(|v| v.as_f64()).collect(),
            None => {
                let end = if stop.is_finite() {
                    horizon.min(stop)
                } else {
                    horizon
                };
                let mut t: Vec<f64> = linspace(tau, end, 25)[1..].to_vec();
                if stop.is_finite() && stop <= horizon {
                    t.push(stop);
                }
                t.push(horizon);
                t
            }
        };
        times.retain(|t| *t >= tau);
        times.sort_by(|a, b| a.total_cmp(b));
        times.dedup();
        let gen = settings["generalization"].as_str().unwrap_or("new_session");
        let (q, log_extra): (Quad, Option<Vec<f64>>) = match gen {
            "known_session" => {
                let sid = settings["session_id"]
                    .as_str()
                    .map(String::from)
                    .or_else(|| prefix.session_id.clone());
                let sessions: Vec<String> =
                    serde_json::from_value(rel.architecture["session_ids"].clone())?;
                let Some(gi) = sid
                    .as_ref()
                    .and_then(|s| sessions.iter().position(|x| x == s))
                else {
                    return Err(Refusal::unsupported(
                        "unknown_session",
                        format!("session {sid:?} not in the training sessions; use new_session"),
                    )
                    .into());
                };
                let train_runs: Vec<String> =
                    serde_json::from_value(rel.architecture["train_run_ids"].clone())
                        .unwrap_or_default();
                if train_runs.contains(&prefix.run_id) {
                    // review-B fix 2: the stored session posterior contains this run's own endpoint
                    return Err(Refusal::unsupported("training_run_known_session", format!("run {:?} is a training run of session {:?}: its endpoint is inside the session posterior; known_session is for a new run", prefix.run_id, sid.unwrap())).into());
                }
                let q = m::quadrature(
                    &b.spec,
                    f(&b.cfg, &["train_quadrature_nodes"], 12.0) as usize,
                );
                let tab = &release.params["derived.session_loglik_nodes"];
                let (d, g, n) = (tab.shape[0], tab.shape[1], tab.shape[2]);
                if d != b.draws.len() || n != q.n() {
                    bail!(
                        "session table shape {:?} does not match the draws/quadrature",
                        tab.shape
                    );
                }
                let raw: Vec<f64> = (0..d)
                    .flat_map(|di| {
                        tab.data[(di * g + gi) * n..(di * g + gi + 1) * n]
                            .iter()
                            .cloned()
                    })
                    .collect();
                let t = m::normalise_session_table(&raw, &q);
                (q, Some(t))
            }
            "new_session" => {
                let n = settings["gh_nodes"]
                    .as_u64()
                    .map(|v| v as usize)
                    .unwrap_or(f(&b.cfg, &["forecast_quadrature_nodes"], 24.0) as usize);
                (m::quadrature(&b.spec, n), None)
            }
            other => {
                return Err(Refusal::unsupported(
                    "generalization",
                    format!("generalization {other:?} not supported"),
                )
                .into())
            }
        };
        let out = m::predict_partition(
            &b.draws,
            &b.spec,
            &b.lay,
            &c,
            t_entry,
            tau,
            &times,
            stop,
            horizon,
            &q,
            log_extra.as_deref(),
        );
        let k = b.spec.k();
        let nt = times.len();
        let mut cats: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        cats.insert("active_supported", out.part.active.clone());
        cats.insert("unresolved_support", out.part.unresolved.clone());
        cats.insert("extinction", vec![0.0; nt]);
        cats.insert("operator_termination", vec![0.0; nt]);
        for (j, cause) in b.spec.causes.iter().enumerate() {
            cats.insert(
                if cause == "extinction" {
                    "extinction"
                } else {
                    "operator_termination"
                },
                (0..nt).map(|t| out.part.f[t * k + j]).collect(),
            );
        }
        let op = cats.get_mut("operator_termination").unwrap();
        for t in 0..nt {
            op[t] += out.part.atom[t];
        }
        let rows: Vec<Vec<f64>> = CATEGORIES.iter().map(|c| cats[c].clone()).collect();
        let n_nodes = (q.n() as f64).powf(1.0 / k as f64).round();
        let diag = json!({"interpretation": INTERPRETATION, "counterfactual_estimand": COUNTERFACTUAL_STATUS,
            "policy": "observed experimental policy", "evidence": "model output", "condition_support": why,
            "posterior_global_weights": out.global, "quadrature_nodes_per_dim": n_nodes, "generalization": gen,
            "operator_atom_at_stop": if stop.is_finite() { *out.part.atom.last().unwrap_or(&0.0) } else { 0.0 },
            "scheduled_stop_s": if stop.is_finite() { json!(stop) } else { Value::Null },
            "continuous_causes": b.spec.causes, "max_partition_error": out.part.max_partition_error(),
            "inference": rel.numerics.get("inference").and_then(|i| i.get("method")).cloned()});
        Ok(common::artifact(
            ArtifactKind::EventProbabilities,
            rel,
            prefix,
            settings,
            diag,
            |a| {
                a.times_s = times.clone();
                a.event_categories = CATEGORIES.iter().map(|s| s.to_string()).collect();
                a.event_cdf = rows;
            },
        ))
    }

    fn log_likelihood(&self, release: &LoadedRelease, view: &View) -> ExpertResult<f64> {
        let View::Acme(row) = view else {
            bail!("not an ACME view")
        };
        let b = bind(release)?;
        self.row_predictive(
            &b,
            row,
            f(&b.cfg, &["forecast_quadrature_nodes"], 24.0) as usize,
        )
    }

    fn objective(
        &self,
        release: &LoadedRelease,
        params: &TensorMap,
        views: &[&View],
    ) -> Option<ExpertResult<(f64, TensorMap)>> {
        Some((|| {
            let b = bind(release)?;
            let (rows, _) = self.select(views.iter().copied());
            let (rows, _) = self.with_features(&b.features, rows)?;
            let c: Vec<Vec<f64>> = rows
                .iter()
                .map(|r| self.normalised(&b, &r.features))
                .collect::<anyhow::Result<_>>()?;
            let enc = m::encode_rows(&rows, &c, &b.spec, None)?;
            let q = m::quadrature(
                &b.spec,
                f(&b.cfg, &["train_quadrature_nodes"], 12.0) as usize,
            );
            let pr = m::prior_settings(&b.spec, &b.cfg);
            let p = b.lay.from_tensors(params)?;
            let mut g = vec![0.0; b.lay.len];
            let v = m::log_posterior(&p, &b.spec, &b.lay, &pr, &enc, &q, Some(&mut g));
            g.iter_mut().for_each(|x| *x = -*x);
            Ok((-v, b.lay.to_tensors(&g)))
        })())
    }

    fn activations(
        &self,
        release: &LoadedRelease,
        params: &TensorMap,
        view: &View,
    ) -> Option<ExpertResult<TensorMap>> {
        Some((|| {
            let View::Acme(row) = view else {
                bail!("not an ACME view")
            };
            let b = bind(release)?;
            let p = b.lay.from_tensors(params)?;
            let c = self.normalised(&b, &row.features)?;
            let hi = [
                row.t_upper_s,
                row.t_lower_s,
                row.scheduled_stop_s,
                Some(row.t_entry_s + 1.0),
            ]
            .into_iter()
            .flatten()
            .fold(f64::NEG_INFINITY, f64::max);
            let times = linspace(row.t_entry_s, hi, 64);
            let lam = m::hazard_curves(&p, &b.spec, &b.lay, &c, &times);
            let k = b.spec.k();
            let mut out = TensorMap::new();
            out.insert("times_s".into(), Tensor::vector(times.clone()));
            out.insert("features.normalized".into(), Tensor::vector(c));
            for (j, cause) in b.spec.causes.iter().enumerate() {
                out.insert(
                    format!("log_hazard.{cause}"),
                    Tensor::vector((0..times.len()).map(|t| lam[t * k + j].ln()).collect()),
                );
            }
            Ok(out)
        })())
    }
}

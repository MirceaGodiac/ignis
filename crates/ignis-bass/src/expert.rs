//! BASS experts: the five-member group-bootstrap MAP operator ensemble (`bass.operator`) and two
//! baselines (`bass.persistence`, `bass.constant_velocity`). Forecasts are of the calibrated VISIBLE
//! front f(s,t) on the specimen plane — not an isotherm, not the pyrolysis front, not a reacting-flow
//! solution. Members are trained in parallel (Rayon), runs inside a member in parallel too.

use std::collections::BTreeMap;
use std::time::Instant;

use anyhow::{bail, Context};
use ignis_acme::common::{self, deep_merge, obj};
use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, Family, ModelRelease, PrefixBundle, Quantity,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::{save_release, LoadedRelease, Tensor, TensorMap};
use ignis_core::resources::Profile;
use ignis_core::views::{BassSequence, View};
use rand::{rngs::StdRng, RngExt, SeedableRng};
use rayon::prelude::*;
use serde_json::{json, Value};

use crate::data::{self, FeatureSpec, Norm, NormArrays, RunData};
use crate::dynamics::{self as dy, Ctx, Numerics};
use crate::model::{self as m, GlobalLayout, N_LOCAL};
use crate::operator::{self as op, Grid};

pub const INTERPRETATION: &str = "calibrated visible luminous front on the specimen plane under the declared constant/known control schedule; not an isotherm or pyrolysis front";

pub fn defaults() -> Value {
    json!({
        "features": [], "length_key": "length", "min_valid": 12, "min_valid_fraction": 0.5, "ell_s": 0.1,
        "numerics": {"cfl": 0.4, "feedback": 0.25, "dt_max_s": 1.0, "max_steps": 32, "forecast_max_steps": 128,
                     "scheme": "eno2", "boundary": "linear"},
        "train": {"n_members": 5, "steps": 6000, "lr": 1e-3, "clip_norm": 1.0, "weight_decay": 0.0, "seed": 0},
        "priors": m::default_priors(),
        "forecast": {"n_particles": 64, "epsilon": 0.1, "quantile_levels": [0.05, 0.5, 0.95], "seed": 0,
                     "max_unresolved_fraction": 0.5, "validation_particles": 16, "map_maxiter": 200},
        "support": {"margin_sd": 0.5},
        "trained_on": "nasa",
    })
}

pub fn load_config() -> anyhow::Result<Value> {
    let p = ignis_core::resources::configs_dir()
        .join("models")
        .join("bass")
        .join("operator.toml");
    let mut cfg = defaults();
    if let Some(t) = common::load_toml(&p)? {
        cfg = deep_merge(&cfg, &t);
    }
    Ok(cfg)
}

pub fn features_of(cfg: &Value) -> anyhow::Result<Vec<FeatureSpec>> {
    Ok(serde_json::from_value(cfg["features"].clone())?)
}

fn policy(cfg: &Value) -> anyhow::Result<ProspectivePolicy> {
    let mut allowed: Vec<String> = features_of(cfg)?
        .into_iter()
        .filter(|f| f.transform != "time_since")
        .map(|f| f.source)
        .collect();
    allowed.push(cfg["length_key"].as_str().unwrap_or("length").into());
    allowed.push("width".into());
    let a: Vec<&str> = allowed.iter().map(|s| s.as_str()).collect();
    Ok(ProspectivePolicy::new(Family::Bass, &a, &[]))
}

fn fnum(v: &Value, d: f64) -> f64 {
    v.as_f64().unwrap_or(d)
}

/// Prefix contours → one run, or a refusal. Control changes: `known_controls` stage starts (built by
/// `build_prefix` with protocol-fixed future stages only) and an optional `control_changes_s`
/// setting, refused after the cutoff unless `control_schedule_known_at_cutoff` (review-B fix 6).
pub fn prefix_run(
    prefix: &PrefixBundle,
    cfg: &Value,
    settings: &Value,
) -> Result<(RunData, f64, Vec<String>), Refusal> {
    let mut cons: Vec<_> = prefix.contours.iter().collect();
    cons.sort_by(|a, b| a.time_s.total_cmp(&b.time_s).then(a.frame.cmp(&b.frame)));
    if cons.is_empty() {
        return Err(Refusal::unsupported(
            "no_contour",
            "no admitted contour at the cutoff",
        ));
    }
    let statuses: Vec<String> = cons
        .iter()
        .map(|c| {
            serde_json::to_value(c.status)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    let s = cons[0].s_m.clone();
    if cons.iter().any(|c| c.s_m != s) {
        return Err(Refusal::unsupported(
            "grid_mismatch",
            "admitted contours are not on one calibrated transverse grid",
        ));
    }
    let ns = s.len();
    let f: Vec<f64> = cons
        .iter()
        .flat_map(|c| c.f_m.iter().map(|x| x.unwrap_or(f64::NAN)))
        .collect();
    let valid: Vec<bool> = f.iter().map(|x| x.is_finite()).collect();
    let last_frac = valid[(cons.len() - 1) * ns..]
        .iter()
        .filter(|b| **b)
        .count() as f64
        / ns as f64;
    let (ok, code, why) =
        data::geometric_status_check(&statuses, last_frac, fnum(&cfg["min_valid_fraction"], 0.5));
    if !ok {
        return Err(Refusal::unsupported(code, why));
    }
    let mut vals = common::condition_values(prefix);
    if let Some(g) = settings["known_geometry"].as_object() {
        for (k, v) in g {
            if let Some(x) = v.as_f64() {
                vals.insert(k.clone(), x);
            }
        }
    }
    let feats = features_of(cfg).map_err(|e| Refusal::unsupported("config", e.to_string()))?;
    let cs = data::static_raw(&feats, &vals).map_err(|e| {
        Refusal::unsupported(
            "missing_condition",
            format!("missing prospective condition {e:?}; no imputation"),
        )
    })?;
    let Some(l) = vals
        .get(cfg["length_key"].as_str().unwrap_or("length"))
        .cloned()
    else {
        return Err(Refusal::unsupported(
            "missing_condition",
            "specimen length unknown",
        ));
    };
    let mut changes: Vec<f64> = prefix.known_controls.iter().map(|c| c.t_start_s).collect();
    if let Some(extra) = settings["control_changes_s"].as_array() {
        let known = settings["control_schedule_known_at_cutoff"]
            .as_bool()
            .unwrap_or(false);
        for c in extra.iter().filter_map(|v| v.as_f64()) {
            if c > prefix.cutoff_s && !known {
                return Err(Refusal::unsupported("future_control", format!("control change at {c} s is after the cutoff {} s and not declared as a schedule known at the cutoff", prefix.cutoff_s)));
            }
            changes.push(c);
        }
    }
    changes.sort_by(|a, b| a.total_cmp(b));
    let t: Vec<f64> = cons.iter().map(|c| c.time_s).collect();
    let u: Vec<f64> = cons
        .iter()
        .map(|c| c.scale_uncertainty_rel.unwrap_or(0.0))
        .collect();
    let run = data::build_run(
        &prefix.run_id,
        "prefix",
        &t,
        &s,
        &f,
        &valid,
        &statuses,
        &u,
        cs,
        l,
        changes,
        cfg,
        op::N_NODES,
    )
    .map_err(|e| Refusal::unsupported("no_usable_contour", e))?;
    Ok((run, s[0], statuses))
}

fn group_index(runs: &[RunData]) -> (Vec<String>, Vec<usize>) {
    let mut groups: Vec<String> = runs.iter().map(|r| r.group.clone()).collect();
    groups.sort();
    groups.dedup();
    let gidx = runs
        .iter()
        .map(|r| groups.iter().position(|g| *g == r.group).unwrap())
        .collect();
    (groups, gidx)
}

// ---------------------------------------------------------------- operator

pub struct BassOperatorExpert {
    cfg0: Value,
    policy: ProspectivePolicy,
}

pub struct Member {
    pub g: Vec<f64>,
}

pub struct BoundOperator {
    pub cfg: Value,
    pub feats: Vec<FeatureSpec>,
    pub norm: Norm,
    pub na: NormArrays,
    pub members: Vec<Member>,
}

pub fn bind_operator(r: &LoadedRelease) -> anyhow::Result<BoundOperator> {
    let cfg = Value::Object(r.release.config.clone());
    let lay = GlobalLayout::new();
    let n = r.release.architecture["n_members"]
        .as_u64()
        .context("n_members")? as usize;
    let mut members = vec![];
    for k in 0..n {
        let mut g = vec![0.0; lay.len];
        for (name, _, off) in &lay.blocks {
            let t = r
                .params
                .get(&format!("m{k}.{name}"))
                .with_context(|| format!("m{k}.{name}"))?;
            g[*off..off + t.data.len()].copy_from_slice(&t.data);
        }
        members.push(Member { g });
    }
    let feats = features_of(&cfg)?;
    let na = data::norm_arrays(&feats, &r.release.normalization);
    Ok(BoundOperator {
        feats,
        norm: r.release.normalization.clone(),
        na,
        cfg,
        members,
    })
}

/// Joint (member × particle) local posterior of one run.
pub struct Posterior {
    pub locals: Vec<m::LocalPosterior>,
    /// (M × P) joint weights
    pub w: Vec<f64>,
    pub log_z: f64,
}

pub fn posterior(
    b: &BoundOperator,
    run: &RunData,
    n_particles: usize,
    epsilon: f64,
    seed: u64,
    forecast: bool,
) -> Posterior {
    let lay = GlobalLayout::new();
    let grid = Grid::new(op::N_NODES);
    let num = m::numerics_from(&b.cfg, forecast);
    let maxiter = b.cfg["forecast"]["map_maxiter"].as_u64().unwrap_or(200) as usize;
    let locals: Vec<m::LocalPosterior> = b
        .members
        .par_iter()
        .enumerate()
        .map(|(k, mem)| {
            m::local_inference(
                &mem.g,
                &lay,
                &grid,
                num,
                run,
                &b.na,
                n_particles,
                epsilon,
                seed * 7919 + k as u64,
                maxiter,
            )
        })
        .collect();
    let mm = locals.len() as f64;
    let lw: Vec<f64> = locals
        .iter()
        .flat_map(|l| {
            l.logw
                .iter()
                .map(|v| v - mm.ln() - (n_particles as f64).ln())
        })
        .collect();
    let (w, log_z) = ignis_science::gauss_hermite::joint_normalize(&lw);
    Posterior { locals, w, log_z }
}

impl BassOperatorExpert {
    pub fn new() -> anyhow::Result<Self> {
        let cfg0 = load_config()?;
        Ok(BassOperatorExpert {
            policy: policy(&cfg0)?,
            cfg0,
        })
    }

    fn view_loglik(
        &self,
        b: &BoundOperator,
        view: &BassSequence,
        n_particles: usize,
    ) -> Option<f64> {
        let (runs, _) = data::prepare(&[view], &b.feats, &b.cfg, op::N_NODES);
        let run = runs.into_iter().next()?;
        Some(
            posterior(
                b,
                &run,
                n_particles,
                fnum(&b.cfg["forecast"]["epsilon"], 0.1),
                0,
                false,
            )
            .log_z,
        )
    }
}

/// optax.chain(clip_by_global_norm(c), adamw(lr, b1=0.9, b2=0.999, eps=1e-8, weight_decay)).
pub struct AdamW {
    m: Vec<f64>,
    v: Vec<f64>,
    t: i32,
    pub lr: f64,
    pub clip: f64,
    pub wd: f64,
}

impl AdamW {
    pub fn new(n: usize, lr: f64, clip: f64, wd: f64) -> AdamW {
        AdamW {
            m: vec![0.0; n],
            v: vec![0.0; n],
            t: 0,
            lr,
            clip,
            wd,
        }
    }
    pub fn step(&mut self, p: &mut [f64], g: &mut [f64]) {
        let norm = g.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > self.clip {
            let s = self.clip / norm;
            g.iter_mut().for_each(|x| *x *= s);
        }
        self.t += 1;
        let (b1, b2, eps) = (0.9f64, 0.999f64, 1e-8);
        let c1 = 1.0 - b1.powi(self.t);
        let c2 = 1.0 - b2.powi(self.t);
        for i in 0..p.len() {
            self.m[i] = b1 * self.m[i] + (1.0 - b1) * g[i];
            self.v[i] = b2 * self.v[i] + (1.0 - b2) * g[i] * g[i];
            let u = (self.m[i] / c1) / ((self.v[i] / c2).sqrt() + eps) + self.wd * p[i];
            p[i] -= self.lr * u;
        }
    }
}

impl Expert for BassOperatorExpert {
    fn family(&self) -> Family {
        Family::Bass
    }
    fn model_name(&self) -> &str {
        "bass.operator"
    }
    fn is_baseline(&self) -> bool {
        false
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }
    fn default_config(&self) -> ExpertResult<Value> {
        Ok(self.cfg0.clone())
    }
    fn estimate_hours(
        &self,
        config: &Value,
        _n_runs: usize,
        n_obs: usize,
        profile: &Profile,
    ) -> f64 {
        // measured ≈ 6e-5 core-seconds per valid sample per member step (ENO2, 32-thread CPU)
        let steps = fnum(&config["train"]["steps"], 6000.0);
        let members = fnum(&config["train"]["n_members"], 5.0);
        steps * members * n_obs as f64 * 6e-5 / profile.threads().max(1) as f64 / 3600.0
    }

    fn fit(
        &self,
        data_: &FitData<'_>,
        config: &Value,
        ctx: &mut FitContext,
    ) -> ExpertResult<ModelRelease> {
        let t0 = Instant::now();
        let cfg = deep_merge(&self.cfg0, config);
        let feats = features_of(&cfg)?;
        let train: Vec<&BassSequence> = data_
            .partition("train")
            .into_iter()
            .filter_map(|v| if let View::Bass(s) = v { Some(s) } else { None })
            .collect();
        let (runs, excluded) = data::prepare(&train, &feats, &cfg, op::N_NODES);
        if runs.is_empty() {
            bail!("no usable BASS sequences: {excluded:?}");
        }
        let norm = data::fit_normalization(&feats, &runs);
        let na = data::norm_arrays(&feats, &norm);
        let lay = GlobalLayout::new();
        let grid = Grid::new(op::N_NODES);
        let num = m::numerics_from(&cfg, false);
        let pr = m::Priors::from_json(&cfg["priors"]);
        let tc = &cfg["train"];
        let n_members = fnum(&tc["n_members"], 5.0) as usize;
        let steps = fnum(&tc["steps"], 6000.0) as usize;
        let seed = tc["seed"].as_u64().unwrap_or(0) ^ ctx.seed;
        let (groups, gidx) = group_index(&runs);
        let mut rng = StdRng::seed_from_u64(seed);
        let mults: Vec<Vec<f64>> = (0..n_members)
            .map(|_| {
                let mut mult = vec![0.0; groups.len()];
                for _ in 0..groups.len() {
                    mult[rng.random_range(0..groups.len())] += 1.0; // bootstrap of GROUPS, not frames/pixels
                }
                mult
            })
            .collect();
        let speed = m::speed_estimate(&runs);
        let x_init: Vec<f64> = runs
            .iter()
            .flat_map(|r| m::init_locals(r).to_vec())
            .collect();
        let nr = runs.len();
        let t_train = Instant::now();
        let results: Vec<(Vec<f64>, Vec<f64>, Value)> = ctx.pool.install(|| {
            (0..n_members)
                .into_par_iter()
                .map(|k| {
                    let w: Vec<f64> = gidx.iter().map(|&g| mults[k][g]).collect();
                    let mut g = m::init_network(&lay, seed * 1000 + k as u64);
                    g[lay.phys.beta_v] = speed / dy::V_REF;
                    for (i, off) in [lay.lsc, lay.lsn, lay.lsr].into_iter().enumerate() {
                        g[off] = pr.obs[i].0;
                    }
                    let mut x = x_init.clone();
                    let n = lay.len + nr * N_LOCAL;
                    let mut opt = AdamW::new(n, fnum(&tc["lr"], 1e-3), fnum(&tc["clip_norm"], 1.0), fnum(&tc["weight_decay"], 0.0));
                    let mut state = [g, x.clone()].concat();
                    let mut grad = vec![0.0; n];
                    let (mut first, mut last, mut unr, mut cfl) = (f64::NAN, f64::NAN, 0.0, 0.0);
                    for s in 0..steps {
                        let (gp, xp) = state.split_at(lay.len);
                        let (gg, gx) = grad.split_at_mut(lay.len);
                        let ev = m::member_objective(gp, xp, &runs, &w, &lay, &grid, num, &na, &pr, gg, gx);
                        if s == 0 {
                            first = ev.value;
                        }
                        last = ev.value;
                        unr = ev.unreached;
                        cfl = ev.max_cfl;
                        opt.step(&mut state, &mut grad);
                    }
                    x.copy_from_slice(&state[lay.len..]);
                    state.truncate(lay.len);
                    let diag = json!({"initial_objective": first, "final_objective": last, "unreached_s": unr, "max_cfl": cfl,
                        "group_multiplicity": groups.iter().zip(&mults[k]).map(|(g, v)| (g.clone(), json!(*v as i64))).collect::<serde_json::Map<_, _>>()});
                    (state, x, diag)
                })
                .collect()
        });
        let train_seconds = t_train.elapsed().as_secs_f64();
        let mut params = TensorMap::new();
        let mut ensemble = vec![];
        let mut diag_members = vec![];
        for (k, (g, x, d)) in results.iter().enumerate() {
            for (name, shape, off) in &lay.blocks {
                let len: usize = shape.iter().product();
                params.insert(
                    format!("m{k}.{name}"),
                    Tensor::new(shape.clone(), g[*off..off + len].to_vec()),
                );
            }
            let a: Vec<f64> = x
                .chunks(N_LOCAL)
                .flat_map(|c| c[..op::N_FRONT_COEFFS].to_vec())
                .collect();
            let bb: Vec<f64> = x
                .chunks(N_LOCAL)
                .flat_map(|c| c[op::N_FRONT_COEFFS..].to_vec())
                .collect();
            params.insert(
                format!("m{k}.local.a"),
                Tensor::new(vec![nr, op::N_FRONT_COEFFS], a),
            );
            params.insert(
                format!("m{k}.local.b"),
                Tensor::new(vec![nr, op::N_VELOCITY_COEFFS], bb),
            );
            ensemble.push(json!({"index": k, "seed": seed * 1000 + k as u64, "group_multiplicity": d["group_multiplicity"]}));
            diag_members.push(json!({"initial_objective": d["initial_objective"], "final_objective": d["final_objective"],
                "unreached_s": d["unreached_s"], "max_cfl": d["max_cfl"]}));
        }
        let mut rel = common::release(
            Family::Bass,
            "bass.operator",
            false,
            feats.iter().map(|f| f.name.clone()).collect(),
            data_.cohort_id,
            &data_.split.id,
            data_.source_manifest_hash,
            data_.synthetic || cfg["trained_on"] == "synthetic",
        );
        rel.architecture = obj(
            json!({"kind": "bass_cosine_neural_operator", "nodes": op::N_NODES, "width": op::WIDTH, "modes": op::N_MODES,
            "blocks": op::N_BLOCKS, "film": "8->16->96 tanh", "n_network_weights": op::n_weights(), "n_global_fit_per_member": m::n_global(),
            "n_local_per_run": N_LOCAL, "n_members": n_members, "ensemble": ensemble, "train_run_ids": runs.iter().map(|r| r.run_id.clone()).collect::<Vec<_>>(),
            "cosine_convention": "trapezoid q, phi0=1, phim=sqrt2 cos(m pi xi)", "boundary": cfg["numerics"]["boundary"],
            "integrator": format!("SSP-RK2 + local Lax-Friedrichs ({})", cfg["numerics"]["scheme"].as_str().unwrap_or("eno2"))}),
        );
        rel.config = obj(cfg.clone());
        rel.units = BTreeMap::from([
            ("f".into(), "m".into()),
            ("s".into(), "m".into()),
            ("time".into(), "s".into()),
            ("v_ref".into(), "m/s".into()),
            ("nu_ref".into(), "m2/s".into()),
        ]);
        rel.normalization = norm;
        rel.priors = obj(deep_merge(
            &cfg["priors"],
            &json!({"local": {"sd": m::local_sd().to_vec()}}),
        ));
        rel.observation_model = obj(
            json!({"likelihood": "8 masked cosine coefficients per frame, multivariate Student-t nu=4",
            "spatial_kernel": format!("exponential in xi, ell_s={} fixed", cfg["ell_s"]),
            "scale_uncertainty": "one relative scale error per run, var u_view^2 + s_rel^2, adaptive GH", "frames": "conditionally independent"}),
        );
        rel.applicability = obj(
            json!({"interpretation": INTERPRETATION, "excluded_runs": excluded, "topology": "single-valued contours only; multivalued/detached refuse"}),
        );
        let mut bound = BoundOperator {
            feats: feats.clone(),
            na: data::norm_arrays(&feats, &rel.normalization),
            norm: rel.normalization.clone(),
            cfg: cfg.clone(),
            members: results.iter().map(|r| Member { g: r.0.clone() }).collect(),
        };
        let val: Vec<&BassSequence> = data_
            .partition("validation")
            .into_iter()
            .filter_map(|v| if let View::Bass(s) = v { Some(s) } else { None })
            .collect();
        if !val.is_empty() {
            let np = cfg["forecast"]["validation_particles"]
                .as_u64()
                .unwrap_or(16) as usize;
            let lls: Vec<f64> = ctx.pool.install(|| {
                val.iter()
                    .filter_map(|v| self.view_loglik(&bound, v, np))
                    .collect()
            });
            if !lls.is_empty() {
                rel.evaluation = obj(
                    json!({"validation_mean_log_lik": lls.iter().sum::<f64>() / lls.len() as f64, "n_validation": lls.len()}),
                );
            }
        }
        bound.members.clear();
        let mut numerics = cfg["numerics"].clone();
        numerics["x64"] = json!(true);
        numerics["members"] = json!(diag_members);
        numerics["fit_seconds"] = json!(t0.elapsed().as_secs_f64());
        numerics["train_seconds"] = json!(train_seconds);
        numerics["member_steps_per_s"] =
            json!((n_members * steps) as f64 / train_seconds.max(1e-9));
        rel.numerics = obj(numerics);
        save_release(&ctx.registry_root, rel, &params, &TensorMap::new(), &[])
    }

    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact> {
        let rel = &release.release;
        if prefix.family != Family::Bass {
            return Err(Refusal::unsupported(
                "family_mismatch",
                format!("{:?} prefix for a BASS model", prefix.family),
            )
            .into());
        }
        let b = bind_operator(release)?;
        let fc = deep_merge(&b.cfg["forecast"], &settings["forecast"]);
        let (run, s0, _) = prefix_run(prefix, &b.cfg, settings)?;
        let (ok, why_c) = data::condition_support(
            &run.c_static,
            &b.feats,
            &b.norm,
            fnum(&b.cfg["support"]["margin_sd"], 0.5),
        );
        if !ok {
            return Err(Refusal::unsupported("condition_support", why_c).into());
        }
        let tau = prefix.cutoff_s;
        let mut times: Vec<f64> = match settings["times_s"].as_array() {
            Some(a) => a.iter().filter_map(|v| v.as_f64()).collect(),
            None => (1..=10)
                .map(|i| tau + i as f64 * fnum(&settings["step_s"], 1.0))
                .collect(),
        };
        times.retain(|t| *t >= tau);
        times.sort_by(|a, b| a.total_cmp(b));
        let np = fnum(&fc["n_particles"], 64.0) as usize;
        let eps = fnum(&fc["epsilon"], 0.1);
        let post = posterior(&b, &run, np, eps, fc["seed"].as_u64().unwrap_or(0), true);
        let lay = GlobalLayout::new();
        let grid = Grid::new(op::N_NODES);
        let num: Numerics = m::numerics_from(&b.cfg, true);
        let mut tgrid = vec![run.t[0]];
        tgrid.extend(&times);
        let k = grid.k;
        let nt = times.len();
        // simulate every (member, particle)
        let sims: Vec<(Vec<f64>, f64)> = (0..b.members.len() * np)
            .into_par_iter()
            .map_init(dy::SimWork::default, |w, idx| {
                let (mi, pi) = (idx / np, idx % np);
                let x = &post.locals[mi].particles[pi * N_LOCAL..(pi + 1) * N_LOCAL];
                let cx = Ctx {
                    p: &b.members[mi].g,
                    net: &lay.net,
                    phys: &lay.phys,
                    grid: &grid,
                    num,
                    l: run.l,
                    w: run.w,
                };
                let mut tr = dy::Trajectory::default();
                dy::simulate(
                    &cx,
                    &x[..8],
                    &x[8..],
                    &tgrid,
                    &run.cond(&b.na),
                    false,
                    w,
                    &mut tr,
                );
                (tr.frames[k..].to_vec(), tr.unreached)
            })
            .collect();
        let w = &post.w;
        let miss_mass: f64 = sims
            .iter()
            .zip(w)
            .filter(|(s, _)| s.1 > 1e-9)
            .map(|(_, w)| w)
            .sum();
        if miss_mass > 0.01 {
            return Err(Refusal::insufficient(
                "step_limit",
                "step limit reached before the output time (CFL/feedback); refine numerics",
            )
            .into());
        }
        let mut unresolved = vec![0.0; nt];
        let mut exited = vec![vec![false; nt]; sims.len()];
        for (pi, (f, _)) in sims.iter().enumerate() {
            let mut out = false;
            for t in 0..nt {
                out |= f[t * k..(t + 1) * k]
                    .iter()
                    .any(|v| *v <= 0.0 || *v >= run.l);
                exited[pi][t] = out;
                if out {
                    unresolved[t] += w[pi];
                }
            }
        }
        let maxu = fnum(&fc["max_unresolved_fraction"], 0.5);
        let keep: Vec<usize> = (0..nt).filter(|&t| unresolved[t] <= maxu).collect();
        if keep.is_empty() {
            return Err(Refusal::unsupported(
                "horizon",
                "front leaves the specimen domain for most posterior mass",
            )
            .into());
        }
        let levels: Vec<f64> = fc["quantile_levels"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
            .unwrap_or(vec![0.05, 0.5, 0.95]);
        let mut q = vec![vec![0.0; keep.len()]; levels.len()];
        let mut mean = vec![0.0; keep.len()];
        let mut qc = vec![vec![vec![0.0; k]; keep.len()]; levels.len()];
        for (ti, &t) in keep.iter().enumerate() {
            let pos: Vec<f64> = sims
                .iter()
                .map(|(f, _)| m::mean_position(&f[t * k..(t + 1) * k]))
                .collect();
            let qs = m::weighted_quantiles(&pos, w, &levels);
            for (li, v) in qs.into_iter().enumerate() {
                q[li][ti] = v;
            }
            mean[ti] = pos.iter().zip(w).map(|(a, b)| a * b).sum();
            for j in 0..k {
                let col: Vec<f64> = sims.iter().map(|(f, _)| f[t * k + j]).collect();
                for (li, v) in m::weighted_quantiles(&col, w, &levels)
                    .into_iter()
                    .enumerate()
                {
                    qc[li][ti][j] = v;
                }
            }
        }
        let lw_all: Vec<f64> = post.locals.iter().flat_map(|l| l.logw.clone()).collect();
        let member_w: Vec<f64> = (0..b.members.len())
            .map(|mi| w[mi * np..(mi + 1) * np].iter().sum())
            .collect();
        let diag = json!({"interpretation": INTERPRETATION, "uncertainty": "empirical MAP ensemble x local importance posterior; excludes unmodelled metrology/calibration error",
            "evidence": "model output", "condition_support": why_c, "unresolved_fraction": unresolved,
            "importance": ignis_science::gauss_hermite::weight_diagnostics(&lw_all), "member_weights": member_w,
            "log_prefix_evidence": post.log_z, "defensive_epsilon": eps,
            "defensive_epsilon_realised": if eps > 0.0 { (eps * np as f64).ceil() / np as f64 } else { 0.0 },
            "map_converged": post.locals.iter().map(|l| l.map_converged).collect::<Vec<_>>(),
            "contour_quantiles_m": qc, "contour_nodes_s_m": grid.xi.iter().map(|x| s0 + x * run.w).collect::<Vec<_>>(),
            "numerics": {"scheme": b.cfg["numerics"]["scheme"], "boundary": b.cfg["numerics"]["boundary"]},
            "summary": "trapezoid spatial mean of f over the admitted transverse domain"});
        let times_k: Vec<f64> = keep.iter().map(|&t| times[t]).collect();
        Ok(common::artifact(
            ArtifactKind::PrefixForecast,
            rel,
            prefix,
            settings,
            diag,
            |a| {
                a.quantity = Some(Quantity::FrontPosition);
                a.unit = Some("m".into());
                a.times_s = times_k;
                a.quantile_levels = levels;
                a.quantiles = q;
                a.mean = mean;
            },
        ))
    }

    fn log_likelihood(&self, release: &LoadedRelease, view: &View) -> ExpertResult<f64> {
        let View::Bass(s) = view else {
            bail!("not a BASS view")
        };
        let b = bind_operator(release)?;
        let np = b.cfg["forecast"]["n_particles"].as_u64().unwrap_or(64) as usize;
        Ok(self.view_loglik(&b, s, np).unwrap_or(f64::NEG_INFINITY))
    }

    fn objective(
        &self,
        release: &LoadedRelease,
        params: &TensorMap,
        views: &[&View],
    ) -> Option<ExpertResult<(f64, TensorMap)>> {
        Some((|| {
            let rel = &release.release;
            let b = bind_operator(release)?;
            let ids: Vec<String> =
                serde_json::from_value(rel.architecture["train_run_ids"].clone())?;
            let sel: Vec<&BassSequence> = views
                .iter()
                .filter_map(|v| if let View::Bass(s) = v { Some(s) } else { None })
                .filter(|s| ids.contains(&s.run_id))
                .collect();
            let (runs, _) = data::prepare(&sel, &b.feats, &b.cfg, op::N_NODES);
            let order: Vec<usize> = runs
                .iter()
                .map(|r| ids.iter().position(|i| *i == r.run_id).unwrap())
                .collect();
            let lay = GlobalLayout::new();
            let grid = Grid::new(op::N_NODES);
            let num = m::numerics_from(&b.cfg, false);
            let pr = m::Priors::from_json(&b.cfg["priors"]);
            let mut total = 0.0;
            let mut grads = TensorMap::new();
            let ensemble = rel.architecture["ensemble"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            for (k, mem) in ensemble.iter().enumerate() {
                let mut g = vec![0.0; lay.len];
                for (name, _, off) in &lay.blocks {
                    let t = &params[&format!("m{k}.{name}")];
                    g[*off..off + t.data.len()].copy_from_slice(&t.data);
                }
                let (pa, pb) = (
                    &params[&format!("m{k}.local.a")],
                    &params[&format!("m{k}.local.b")],
                );
                let x: Vec<f64> = order
                    .iter()
                    .flat_map(|&i| {
                        [&pa.data[i * 8..(i + 1) * 8], &pb.data[i * 4..(i + 1) * 4]].concat()
                    })
                    .collect();
                let w: Vec<f64> = runs
                    .iter()
                    .map(|r| mem["group_multiplicity"][&r.group].as_f64().unwrap_or(0.0))
                    .collect();
                let (mut gg, mut gx) = (vec![0.0; lay.len], vec![0.0; x.len()]);
                total += m::member_objective(
                    &g, &x, &runs, &w, &lay, &grid, num, &b.na, &pr, &mut gg, &mut gx,
                )
                .value;
                for (name, shape, off) in &lay.blocks {
                    let len: usize = shape.iter().product();
                    grads.insert(
                        format!("m{k}.{name}"),
                        Tensor::new(shape.clone(), gg[*off..off + len].to_vec()),
                    );
                }
                let (mut ga, mut gb) = (vec![0.0; pa.data.len()], vec![0.0; pb.data.len()]);
                for (ri, &i) in order.iter().enumerate() {
                    ga[i * 8..(i + 1) * 8].copy_from_slice(&gx[ri * 12..ri * 12 + 8]);
                    gb[i * 4..(i + 1) * 4].copy_from_slice(&gx[ri * 12 + 8..(ri + 1) * 12]);
                }
                grads.insert(format!("m{k}.local.a"), Tensor::new(pa.shape.clone(), ga));
                grads.insert(format!("m{k}.local.b"), Tensor::new(pb.shape.clone(), gb));
            }
            Ok((total, grads))
        })())
    }
}

// ---------------------------------------------------------------- baselines

const NU: f64 = 4.0;

fn t4_logpdf(x: f64) -> f64 {
    libm::lgamma((NU + 1.0) / 2.0)
        - libm::lgamma(NU / 2.0)
        - 0.5 * (NU * std::f64::consts::PI).ln()
        - (NU + 1.0) / 2.0 * (x * x / NU).ln_1p()
}

/// Student-t (ν = 4) quantile in closed form.
pub fn t4_ppf(p: f64) -> f64 {
    let a = 4.0 * p * (1.0 - p);
    let q = ((a.sqrt()).acos() / 3.0).cos() / a.sqrt();
    (p - 0.5).signum() * 2.0 * (q - 1.0).max(0.0).sqrt()
}

fn coef(xi: &[f64], f: &[f64], valid: &[bool]) -> [f64; 8] {
    let g = crate::observation::projector(xi, valid);
    let s = xi.len();
    let mut c = [0.0; 8];
    for a in 0..8 {
        c[a] = (0..s)
            .filter(|&i| valid[i])
            .map(|i| g[a * s + i] * f[i])
            .sum();
    }
    c
}

fn predict(
    kind: &str,
    t_hist: &[f64],
    f_hist: &[&[f64]],
    v_hist: &[&[bool]],
    t_target: f64,
) -> (Vec<f64>, Vec<bool>) {
    let (f_last, v_last) = (f_hist.last().unwrap(), v_hist.last().unwrap());
    if kind == "persistence" || t_hist.len() < 2 {
        return (f_last.to_vec(), v_last.to_vec());
    }
    let n = t_hist.len().min(3);
    let tt = &t_hist[t_hist.len() - n..];
    let ff = &f_hist[f_hist.len() - n..];
    let vv = &v_hist[v_hist.len() - n..];
    let tm = tt.iter().sum::<f64>() / n as f64;
    let s = f_last.len();
    let denom = tt.iter().map(|t| (t - tm).powi(2)).sum::<f64>().max(1e-300);
    let mut out = vec![0.0; s];
    let mut val = vec![false; s];
    for j in 0..s {
        let fm = ff.iter().map(|f| f[j]).sum::<f64>() / n as f64;
        let slope = (0..n).map(|i| (tt[i] - tm) * (ff[i][j] - fm)).sum::<f64>() / denom;
        out[j] = f_last[j] + slope * (t_target - tt[n - 1]);
        val[j] = v_last[j] && vv.iter().all(|v| v[j]);
    }
    (out, val)
}

fn frame<'a>(r: &'a RunData, i: usize) -> (&'a [f64], &'a [bool]) {
    let s = r.n_s();
    (&r.f[i * s..(i + 1) * s], &r.valid[i * s..(i + 1) * s])
}

fn pairs(kind: &str, runs: &[RunData]) -> (Vec<f64>, Vec<[f64; 8]>) {
    let (mut h, mut res) = (vec![], vec![]);
    for r in runs {
        let idx: Vec<usize> = (0..r.n_t()).filter(|&j| r.use_[j]).collect();
        for (ap, &a) in idx.iter().enumerate() {
            let hist = &idx[ap.saturating_sub(2)..=ap];
            for &bi in &idx[ap + 1..] {
                let th: Vec<f64> = hist.iter().map(|&i| r.t[i]).collect();
                let fh: Vec<&[f64]> = hist.iter().map(|&i| frame(r, i).0).collect();
                let vh: Vec<&[bool]> = hist.iter().map(|&i| frame(r, i).1).collect();
                let (pred, vp) = predict(kind, &th, &fh, &vh, r.t[bi]);
                let (fb, vb) = frame(r, bi);
                let v: Vec<bool> = vp.iter().zip(vb).map(|(a, b)| *a && *b).collect();
                if v.iter().filter(|b| **b).count() < 8 {
                    continue;
                }
                let (co, cp) = (coef(&r.xi, fb, &v), coef(&r.xi, &pred, &v));
                let mut d = [0.0; 8];
                for i in 0..8 {
                    d[i] = co[i] - cp[i];
                }
                res.push(d);
                h.push(r.t[bi] - r.t[a]);
            }
        }
    }
    (h, res)
}

/// −Σ log t₄(R/σ(h))/σ(h), σ_m(h)² = σ0_m² + (σ1_m h)², and its gradient in (log σ0, log σ1).
fn baseline_nll(theta: &[f64], h: &[f64], r: &[[f64; 8]], g: &mut [f64]) -> f64 {
    g.iter_mut().for_each(|v| *v = 0.0);
    let mut v = 0.0;
    for (hi, ri) in h.iter().zip(r) {
        for mm in 0..8 {
            let (s0, s1) = (theta[mm].exp(), theta[8 + mm].exp());
            let var = s0 * s0 + (s1 * hi).powi(2);
            let sc = var.sqrt();
            let x = ri[mm] / sc;
            v -= t4_logpdf(x) - sc.ln();
            // d/dsc of −[logt(r/sc) − log sc] = −[(ν+1) x² / (ν + x²) − 1]/sc
            let dsc = -((NU + 1.0) * x * x / (NU + x * x) - 1.0) / sc;
            g[mm] += dsc * s0 * s0 / sc;
            g[8 + mm] += dsc * (s1 * hi).powi(2) / sc;
        }
    }
    v
}

pub struct BassBaselineExpert {
    kind: String,
    name: String,
    cfg0: Value,
    policy: ProspectivePolicy,
}

impl BassBaselineExpert {
    pub fn new(kind: &str) -> anyhow::Result<Self> {
        if kind != "persistence" && kind != "constant_velocity" {
            bail!("unknown BASS baseline {kind}");
        }
        let cfg0 = load_config()?;
        Ok(BassBaselineExpert {
            kind: kind.into(),
            name: format!("bass.{kind}"),
            policy: policy(&cfg0)?,
            cfg0,
        })
    }
    fn sigmas(r: &LoadedRelease) -> anyhow::Result<(Vec<f64>, Vec<f64>)> {
        let s0 = r
            .params
            .get("noise.log_sigma0")
            .context("noise.log_sigma0")?
            .data
            .iter()
            .map(|v| v.exp())
            .collect();
        let s1 = r
            .params
            .get("noise.log_sigma1")
            .context("noise.log_sigma1")?
            .data
            .iter()
            .map(|v| v.exp())
            .collect();
        Ok((s0, s1))
    }
}

impl Expert for BassBaselineExpert {
    fn family(&self) -> Family {
        Family::Bass
    }
    fn model_name(&self) -> &str {
        &self.name
    }
    fn is_baseline(&self) -> bool {
        true
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }
    fn default_config(&self) -> ExpertResult<Value> {
        Ok(self.cfg0.clone())
    }
    fn estimate_hours(&self, _c: &Value, _n: usize, n_obs: usize, _p: &Profile) -> f64 {
        n_obs as f64 * 1e-7 / 3600.0
    }
    fn fit(
        &self,
        data_: &FitData<'_>,
        config: &Value,
        ctx: &mut FitContext,
    ) -> ExpertResult<ModelRelease> {
        let cfg = deep_merge(&self.cfg0, config);
        let feats = features_of(&cfg)?;
        let train: Vec<&BassSequence> = data_
            .partition("train")
            .into_iter()
            .filter_map(|v| if let View::Bass(s) = v { Some(s) } else { None })
            .collect();
        let (runs, excluded) = data::prepare(&train, &feats, &cfg, op::N_NODES);
        let norm = data::fit_normalization(&feats, &runs);
        let (h, r) = pairs(&self.kind, &runs);
        if h.len() < 4 {
            bail!("too few origin/target pairs to fit the baseline error scale");
        }
        let n = r.len() as f64;
        let mut x0 = vec![0.0; 16];
        for mm in 0..8 {
            let mean = r.iter().map(|v| v[mm]).sum::<f64>() / n;
            let sd = (r.iter().map(|v| (v[mm] - mean).powi(2)).sum::<f64>() / n).sqrt();
            x0[mm] = (sd + 1e-9).ln();
            x0[8 + mm] = x0[mm] - 1.0;
        }
        let res = ignis_acme::optim::lbfgs(|t, g| baseline_nll(t, &h, &r, g), &x0, 15000, 10);
        let mut params = TensorMap::new();
        params.insert(
            "noise.log_sigma0".into(),
            Tensor::vector(res.x[..8].to_vec()),
        );
        params.insert(
            "noise.log_sigma1".into(),
            Tensor::vector(res.x[8..].to_vec()),
        );
        let mut rel = common::release(
            Family::Bass,
            &self.name,
            true,
            feats.iter().map(|f| f.name.clone()).collect(),
            data_.cohort_id,
            &data_.split.id,
            data_.source_manifest_hash,
            data_.synthetic || cfg["trained_on"] == "synthetic",
        );
        rel.architecture = obj(
            json!({"kind": format!("bass_{}", self.kind), "error": "per-mode Student-t on 8 masked coefficients"}),
        );
        rel.config = obj(cfg);
        rel.units = BTreeMap::from([("f".into(), "m".into()), ("time".into(), "s".into())]);
        rel.normalization = norm;
        rel.numerics = obj(json!({"pairs": h.len(), "converged": res.converged}));
        rel.applicability = obj(json!({"excluded_runs": excluded}));
        save_release(&ctx.registry_root, rel, &params, &TensorMap::new(), &[])
    }

    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact> {
        let rel = &release.release;
        if prefix.family != Family::Bass {
            return Err(Refusal::unsupported("family_mismatch", "non-BASS prefix").into());
        }
        let cfg = Value::Object(rel.config.clone());
        let feats = features_of(&cfg)?;
        let (run, _, _) = prefix_run(prefix, &cfg, settings)?;
        let (ok, why_c) = data::condition_support(
            &run.c_static,
            &feats,
            &rel.normalization,
            fnum(&cfg["support"]["margin_sd"], 0.5),
        );
        if !ok {
            return Err(Refusal::unsupported("condition_support", why_c).into());
        }
        let (s0, s1) = Self::sigmas(release)?;
        let tau = prefix.cutoff_s;
        let mut times: Vec<f64> = match settings["times_s"].as_array() {
            Some(a) => a.iter().filter_map(|v| v.as_f64()).collect(),
            None => (1..=10)
                .map(|i| tau + i as f64 * fnum(&settings["step_s"], 1.0))
                .collect(),
        };
        times.retain(|t| *t >= tau);
        times.sort_by(|a, b| a.total_cmp(b));
        let idx: Vec<usize> = (0..run.n_t()).filter(|&i| run.use_[i]).collect();
        let idx = &idx[idx.len().saturating_sub(3)..];
        let th: Vec<f64> = idx.iter().map(|&i| run.t[i]).collect();
        let fh: Vec<&[f64]> = idx.iter().map(|&i| frame(&run, i).0).collect();
        let vh: Vec<&[bool]> = idx.iter().map(|&i| frame(&run, i).1).collect();
        let levels = [0.05, 0.5, 0.95];
        let mut q = vec![vec![0.0; times.len()]; 3];
        let mut mean = vec![0.0; times.len()];
        for (j, &t) in times.iter().enumerate() {
            let (pred, v) = predict(&self.kind, &th, &fh, &vh, t);
            let c = coef(&run.xi, &pred, &v);
            let sc = (s0[0] * s0[0] + (s1[0] * (t - th.last().unwrap())).powi(2)).sqrt();
            mean[j] = c[0];
            for (li, l) in levels.iter().enumerate() {
                q[li][j] = c[0] + sc * t4_ppf(*l);
            }
        }
        let diag = json!({"interpretation": format!("baseline: {}", self.kind), "evidence": "model output", "condition_support": why_c,
            "summary": "masked-LS mode-0 coefficient (spatial mean front position)"});
        Ok(common::artifact(
            ArtifactKind::PrefixForecast,
            rel,
            prefix,
            settings,
            diag,
            |a| {
                a.quantity = Some(Quantity::FrontPosition);
                a.unit = Some("m".into());
                a.times_s = times;
                a.quantile_levels = levels.to_vec();
                a.quantiles = q;
                a.mean = mean;
            },
        ))
    }

    /// Conditional on the origin frame (the operator also scores the first frame; the evaluator must
    /// account for that when comparing).
    fn log_likelihood(&self, release: &LoadedRelease, view: &View) -> ExpertResult<f64> {
        let View::Bass(s) = view else {
            bail!("not a BASS view")
        };
        let cfg = Value::Object(release.release.config.clone());
        let (s0, s1) = Self::sigmas(release)?;
        let (runs, _) = data::prepare(&[s], &features_of(&cfg)?, &cfg, op::N_NODES);
        let Some(r) = runs.first() else {
            return Ok(f64::NEG_INFINITY);
        };
        let idx: Vec<usize> = (0..r.n_t()).filter(|&i| r.use_[i]).collect();
        let mut total = 0.0;
        for pos in 1..idx.len() {
            let hist = &idx[pos.saturating_sub(3)..pos];
            let bi = idx[pos];
            let th: Vec<f64> = hist.iter().map(|&i| r.t[i]).collect();
            let fh: Vec<&[f64]> = hist.iter().map(|&i| frame(r, i).0).collect();
            let vh: Vec<&[bool]> = hist.iter().map(|&i| frame(r, i).1).collect();
            let (pred, vp) = predict(&self.kind, &th, &fh, &vh, r.t[bi]);
            let (fb, vb) = frame(r, bi);
            let v: Vec<bool> = vp.iter().zip(vb).map(|(a, b)| *a && *b).collect();
            if v.iter().filter(|b| **b).count() < 8 {
                continue;
            }
            let (co, cp) = (coef(&r.xi, fb, &v), coef(&r.xi, &pred, &v));
            let dt = r.t[bi] - th.last().unwrap();
            for mm in 0..8 {
                let sc = (s0[mm] * s0[mm] + (s1[mm] * dt).powi(2)).sqrt();
                total += t4_logpdf((co[mm] - cp[mm]) / sc) - sc.ln();
            }
        }
        Ok(total)
    }
}

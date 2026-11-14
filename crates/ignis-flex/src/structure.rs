//! FLEX features, structural variants and the parameter inventory (V3.1 §6.2, §6.7, §23.1).
//!
//! Fixed design constants [IGNIS]: D_ref = 0.002 m, p_ref = 1e5 Pa, K_ref = 1e-6 m^2/s, t_ref = 1 s.
//!
//! ```text
//! c = ((chi_O2 - .21)/.05, chi_CO2/.20, chi_He/.20, log(p/p_ref), log(D0/D_ref))
//! z(c) = (1, c1, c2, c3, c4, c5, c1^2, c1 c2, c1 c4)
//! ```
//!
//! c5 uses the latent D0 of each run, not an observed diameter. Columns without variation in the
//! *train* partition are dropped (training-only rule); the parameter count is recomputed from the
//! retained columns and recorded in the release.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use ignis_science::transforms::{ParamSpec, Prior, Transform};

pub const D_REF_M: f64 = 0.002;
pub const P_REF_PA: f64 = 1.0e5;
pub const K_REF_M2_S: f64 = 1.0e-6;
pub const T_REF_S: f64 = 1.0;

pub const C_NAMES: [&str; 5] = ["c_o2", "c_co2", "c_he", "c_logp", "c_logd0"];
pub const Z_NAMES: [&str; 9] = [
    "1",
    "c_o2",
    "c_co2",
    "c_he",
    "c_logp",
    "c_logd0",
    "c_o2^2",
    "c_o2*c_co2",
    "c_o2*c_logp",
];
/// z column -> constituent c indices (empty for the intercept).
pub const Z_PARTS: [&[usize]; 9] = [&[], &[0], &[1], &[2], &[3], &[4], &[0, 0], &[0, 1], &[0, 3]];
pub const INTERACTION_Z: [usize; 3] = [6, 7, 8];
pub const ROLES: [&str; 4] = ["o2", "co2", "he", "pressure"];

pub fn default_condition_keys() -> BTreeMap<String, String> {
    [
        ("o2", "x_o2"),
        ("co2", "x_co2"),
        ("he", "x_he"),
        ("pressure", "pressure"),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

pub fn normalization() -> BTreeMap<String, BTreeMap<String, f64>> {
    let m = |kv: &[(&str, f64)]| {
        kv.iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect::<BTreeMap<_, _>>()
    };
    [
        ("c_o2", m(&[("center", 0.21), ("scale", 0.05)])),
        ("c_co2", m(&[("center", 0.0), ("scale", 0.20)])),
        ("c_he", m(&[("center", 0.0), ("scale", 0.20)])),
        ("c_logp", m(&[("log_ref", P_REF_PA)])),
        ("c_logd0", m(&[("log_ref", D_REF_M)])),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

/// (c1..c4) from prospective conditions. Missing keys become NaN (refused upstream, never zero).
pub fn known_features(
    conditions: &BTreeMap<String, f64>,
    keys: &BTreeMap<String, String>,
) -> [f64; 4] {
    let g = |r: &str| {
        keys.get(r)
            .and_then(|k| conditions.get(k))
            .copied()
            .unwrap_or(f64::NAN)
    };
    let (o2, co2, he, p) = (g("o2"), g("co2"), g("he"), g("pressure"));
    [
        (o2 - 0.21) / 0.05,
        co2 / 0.20,
        he / 0.20,
        (p / P_REF_PA).ln(),
    ]
}

/// Structural switches shared by the four FLEX experts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Structure {
    pub model_name: String,
    pub is_baseline: bool,
    /// "full": beta over z(c); "intercept_fixed": log K/K_ref = beta0_fixed.
    pub z_mode: String,
    pub curvature: bool,
    /// "hier" | "fixed" | "none"
    pub b0: String,
    pub b0_fixed_scale: f64,
    pub beta0_fixed: f64,
    pub b1: bool,
    pub residual: bool,
    pub hazards: bool,
    pub hidden: usize,
}

impl Structure {
    pub fn n_b(&self) -> usize {
        match self.b0.as_str() {
            "none" => 0,
            "hier" if self.b1 => 2,
            _ => 1,
        }
    }
    pub fn get(name: &str) -> Option<Structure> {
        let base = Structure {
            model_name: name.into(),
            is_baseline: true,
            z_mode: "full".into(),
            curvature: true,
            b0: "hier".into(),
            b0_fixed_scale: 1.5,
            beta0_fixed: 0.5f64.ln(),
            b1: true,
            residual: true,
            hazards: true,
            hidden: 16,
        };
        Some(match name {
            "flex.v3" => Structure {
                is_baseline: false,
                ..base
            },
            "flex.v1" => Structure {
                residual: false,
                ..base
            },
            "flex.d2_population" => Structure {
                curvature: false,
                b0: "none".into(),
                b1: false,
                residual: false,
                hazards: false,
                ..base
            },
            "flex.d2_prefix" => Structure {
                z_mode: "intercept_fixed".into(),
                curvature: false,
                b0: "fixed".into(),
                b1: false,
                residual: false,
                hazards: false,
                ..base
            },
            _ => return None,
        })
    }
}

pub const MODELS: [&str; 4] = ["flex.v3", "flex.v1", "flex.d2_population", "flex.d2_prefix"];

/// Fitted (train-only) structural facts. Stored in `ModelRelease.architecture.feature_spec`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureSpec {
    pub c_active: Vec<bool>,
    pub z_active: Vec<bool>,
    pub condition_keys: BTreeMap<String, String>,
    #[serde(default)]
    pub dropped: Vec<String>,
}

impl FeatureSpec {
    pub fn full(keys: BTreeMap<String, String>) -> FeatureSpec {
        FeatureSpec {
            c_active: vec![true; 5],
            z_active: vec![true; 9],
            condition_keys: keys,
            dropped: vec![],
        }
    }
    pub fn c_idx(&self) -> Vec<usize> {
        (0..5).filter(|&i| self.c_active[i]).collect()
    }
    pub fn z_idx(&self) -> Vec<usize> {
        (0..9).filter(|&i| self.z_active[i]).collect()
    }
    pub fn feature_order(&self) -> Vec<String> {
        self.z_idx()
            .into_iter()
            .map(|i| Z_NAMES[i].to_string())
            .collect()
    }
}

fn nanstd(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| !x.is_nan()).collect();
    if f.is_empty() {
        return f64::NAN;
    }
    let m = f.iter().sum::<f64>() / f.len() as f64;
    (f.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / f.len() as f64).sqrt()
}

/// Training-only rule: drop c columns / z columns without variation among train runs.
/// `logd0_proxy` (log of the first valid diameter / D_ref) only decides whether c5 varies.
pub fn fit_feature_spec(
    ck_train: &[[f64; 4]],
    logd0_proxy: &[f64],
    keys: &BTreeMap<String, String>,
    tol: f64,
) -> FeatureSpec {
    let col = |j: usize| -> Vec<f64> {
        (0..ck_train.len())
            .map(|i| {
                if j < 4 {
                    ck_train[i][j]
                } else {
                    logd0_proxy[i]
                }
            })
            .collect()
    };
    let c_active: Vec<bool> = (0..5)
        .map(|j| {
            let c = col(j);
            c.iter().all(|x| x.is_finite()) && nanstd(&c) > tol
        })
        .collect();
    let mut z_active = Vec::new();
    let mut dropped = Vec::new();
    for (zi, parts) in Z_PARTS.iter().enumerate() {
        if parts.is_empty() {
            z_active.push(true);
            continue;
        }
        let mut ok = parts.iter().all(|&p| c_active[p]);
        if ok && parts.len() == 2 {
            let (a, b) = (col(parts[0]), col(parts[1]));
            let prod: Vec<f64> = a.iter().zip(&b).map(|(x, y)| x * y).collect();
            ok = nanstd(&prod) > tol;
        }
        z_active.push(ok);
        if !ok {
            dropped.push(Z_NAMES[zi].to_string());
        }
    }
    for (j, a) in c_active.iter().enumerate() {
        if !a {
            dropped.push(format!("{} (c)", C_NAMES[j]));
        }
    }
    FeatureSpec {
        c_active,
        z_active,
        condition_keys: keys.clone(),
        dropped,
    }
}

/// §6.7 / §23.1 starting priors [IGNIS]: hypotheses on normalised scales, not NASA measurements.
pub fn default_priors() -> BTreeMap<String, f64> {
    let ln = f64::ln;
    [
        ("beta0_loc", ln(0.5)),
        ("beta0_sd", 1.0),
        ("beta_linear_sd", 0.5),
        ("beta_interaction_sd", 0.25),
        ("theta_sd", 0.35),
        ("b_sigma_scale", 0.5),
        ("b_lkj_eta", 2.0),
        ("gamma0_loc", ln(0.1)),
        ("gamma0_sd", 1.5),
        ("hazard_effect_sd", 0.5),
        ("zeta_loc", ln(0.01)),
        ("zeta_sd", 1.5),
        ("log_sigma_loc", ln(5e-5)),
        ("log_sigma_sd", 1.5),
        ("log_ell_loc", ln(0.2)),
        ("log_ell_sd", 1.0),
        ("nn_bias_sd", 0.1),
        ("log_d0_loc", ln(0.003)),
        ("log_d0_sd", 0.7),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

pub type Priors = BTreeMap<String, f64>;

/// Defaults overridden by a config `[priors]` table.
pub fn merged_priors(over: Option<&Value>) -> Priors {
    let mut p = default_priors();
    if let Some(Value::Object(m)) = over {
        for (k, v) in m {
            if let Some(x) = v.as_f64() {
                p.insert(k.clone(), x);
            }
        }
    }
    p
}

/// Population parameters (unconstrained name -> natural value, prior on the natural coordinate).
pub fn global_specs(st: &Structure, fs: &FeatureSpec, pr: &Priors) -> Vec<ParamSpec> {
    let nz = fs.z_idx().len();
    let nc = fs.c_idx().len();
    let mut specs = Vec::new();
    if st.z_mode == "full" {
        let mut loc = vec![0.0; nz];
        let mut scale: Vec<f64> = fs
            .z_idx()
            .iter()
            .map(|zi| {
                if INTERACTION_Z.contains(zi) {
                    pr["beta_interaction_sd"]
                } else {
                    pr["beta_linear_sd"]
                }
            })
            .collect();
        loc[0] = pr["beta0_loc"];
        scale[0] = pr["beta0_sd"];
        specs.push(
            ParamSpec::new(
                "beta",
                &[nz],
                Transform::Identity,
                Prior::Normal { loc, scale },
            )
            .unit("log(K/K_ref)"),
        );
    }
    if st.curvature {
        specs.push(ParamSpec::new(
            "theta",
            &[3],
            Transform::Identity,
            Prior::normal(0.0, pr["theta_sd"]),
        ));
    }
    if st.b0 == "hier" {
        specs.push(
            ParamSpec::new(
                "b.log_sigma",
                &[st.n_b()],
                Transform::Exp,
                Prior::HalfNormal {
                    scale: vec![pr["b_sigma_scale"]],
                },
            )
            .natural("b.sigma"),
        );
        if st.n_b() == 2 {
            specs.push(
                ParamSpec::new(
                    "b.atanh_rho",
                    &[1],
                    Transform::Tanh,
                    Prior::Lkj2 {
                        eta: pr["b_lkj_eta"],
                    },
                )
                .natural("b.rho"),
            );
        }
    }
    if st.hazards {
        let he = pr["hazard_effect_sd"];
        specs.push(
            ParamSpec::new(
                "hazard.gamma0",
                &[],
                Transform::Identity,
                Prior::normal(pr["gamma0_loc"], pr["gamma0_sd"]),
            )
            .unit("log(1/s)"),
        );
        specs.push(ParamSpec::new(
            "hazard.gamma",
            &[nc],
            Transform::Identity,
            Prior::normal(0.0, he),
        ));
        specs.push(ParamSpec::new(
            "hazard.a",
            &[2],
            Transform::Identity,
            Prior::normal(0.0, he),
        ));
        specs.push(ParamSpec::new(
            "hazard.link",
            &[2],
            Transform::Identity,
            Prior::normal(0.0, he),
        ));
        specs.push(
            ParamSpec::new(
                "hazard.zeta",
                &[],
                Transform::Identity,
                Prior::normal(pr["zeta_loc"], pr["zeta_sd"]),
            )
            .unit("log(1/s)"),
        );
    }
    specs.push(
        ParamSpec::new(
            "obs.log_sigma_c",
            &[],
            Transform::Identity,
            Prior::normal(pr["log_sigma_loc"], pr["log_sigma_sd"]),
        )
        .unit("log(m)"),
    );
    specs.push(
        ParamSpec::new(
            "obs.log_sigma_n",
            &[],
            Transform::Identity,
            Prior::normal(pr["log_sigma_loc"], pr["log_sigma_sd"]),
        )
        .unit("log(m)"),
    );
    specs.push(
        ParamSpec::new(
            "obs.log_ell",
            &[],
            Transform::Identity,
            Prior::normal(pr["log_ell_loc"], pr["log_ell_sd"]),
        )
        .unit("log(s)"),
    );
    if st.residual {
        let nin = nc + 1;
        let h = st.hidden;
        specs.push(ParamSpec::new(
            "residual.W1",
            &[h, nin],
            Transform::Identity,
            Prior::normal(0.0, 1.0 / (nin as f64).sqrt()),
        ));
        specs.push(ParamSpec::new(
            "residual.a1",
            &[h],
            Transform::Identity,
            Prior::normal(0.0, pr["nn_bias_sd"]),
        ));
        specs.push(ParamSpec::new(
            "residual.W2",
            &[1, h],
            Transform::Identity,
            Prior::normal(0.0, 0.25 / (h as f64).sqrt()),
        ));
        specs.push(ParamSpec::new(
            "residual.a2",
            &[1],
            Transform::Identity,
            Prior::normal(0.0, pr["nn_bias_sd"]),
        ));
    }
    specs
}

/// Per-run locals: non-centred eps_b (b = L_b eps) and log(D0 / 1 m).
pub fn local_specs(st: &Structure, n_runs: usize, pr: &Priors) -> Vec<ParamSpec> {
    let mut out = Vec::new();
    if st.n_b() > 0 {
        out.push(ParamSpec::new(
            "local.eps_b",
            &[n_runs, st.n_b()],
            Transform::Identity,
            Prior::normal(0.0, 1.0),
        ));
    }
    out.push(
        ParamSpec::new(
            "local.log_d0",
            &[n_runs],
            Transform::Identity,
            Prior::normal(pr["log_d0_loc"], pr["log_d0_sd"]),
        )
        .unit("log(m)"),
    );
    out
}

pub fn structure_json(st: &Structure) -> Value {
    serde_json::to_value(st).unwrap_or(json!({}))
}

//! `acme_normal.wsp` and `acme_inverse.wsp` — the strict ACME forward: when and why the established
//! hot-flame phase of a burner-stabilised flame ends, from fixed handwritten weights.
//!
//! ```text
//! x = (c_logp, c_logm, c_o2, c_xf, c_tad, c_zst, f_ethane, f_butane, x_t),   x_t = ln(1 + t / t_ref)
//!
//! ln(lambda_ext * 1 s) = min(N_ext(x), cap)        spontaneous extinction
//! ln(lambda_op  * 1 s) = min(N_op(x),  cap)        termination by the operator
//!
//! N(x) = b0 + skip . x + sum_j w2[j] relu(b1[j] + W1[j, :] . x)
//! ```
//!
//! The two causes compete from the cutoff on; the forecast is the partition
//! `active, extinction, operator_termination, unresolved_support` conditional on the flame being
//! established at the cutoff. An operator stop is never counted as an extinction: it has its own
//! hazard, its own row, and a protocol stop known at the cutoff lands on the operator row as an atom.
//!
//! Normal and inverse share this code and nothing else. They have separate specs (other family,
//! other reference scales, and the opposite physical role of the injected stream), separate patches
//! and separate releases; a release of one never runs a prefix of the other.
//!
//! The prefix carries no time series for ACME: the conditions, the cutoff and the control stages
//! known at the cutoff are all the forward reads. Nothing is fitted anywhere on this route.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, ControlStage, DomainBound, Family, FeatureSpec, Finite,
    ForwardContract, ForwardProfile, ModelRelease, ModelSpec, PrefixBundle, QualificationState,
    Quantity, RawValue, SpecId, Support, SupportGate, TargetSpec, TensorRole, WspSchema,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::LoadedRelease;
use ignis_core::resources::Profile;
use ignis_core::views::View;

use crate::common::{self, admitted, refuse, Body};
use crate::nn::hinge_net;
use crate::spec::{authored, constant, sparse, Params};

pub const FORWARD_VERSION: &str = "1";
/// Hidden width of each cause's network.
pub const HIDDEN: usize = 12;
/// Network inputs: eight condition features and elapsed time.
pub const N_IN: usize = 9;
pub const FEATURES: [&str; N_IN] = [
    "c_logp", "c_logm", "c_o2", "c_xf", "c_tad", "c_zst", "f_ethane", "f_butane", "x_t",
];
/// Same names as the fitted ACME experts, so both are scored on the same rows.
pub const CATEGORIES: [&str; 4] = [
    "active_supported",
    "extinction",
    "operator_termination",
    "unresolved_support",
];
/// Never admissible as ACME predictors (the fitted experts' list; part of the prefix policy).
pub const FORBIDDEN: [&str; 9] = [
    "hc_burnrate",
    "hc_burn_rate",
    "tfp",
    "peak_burner_temperature",
    "after_test_composition",
    "final_duration",
    "duration_s",
    "extinction_time",
    "ignition_success",
];
/// Condition keys a prefix may carry (the fitted experts' policy; the prefix hash depends on it).
pub const ALLOWED: [&str; 9] = [
    "pressure",
    "injected_mass_flow",
    "x_o2_role",
    "x_fuel_role",
    "tad",
    "zst",
    "fuel_is_ethane",
    "fuel_is_butane",
    "fuel_species",
];
pub const INTERPRETATION: &str = "observational event prediction under the observed experimental \
    policy; operator termination is not spontaneous extinction";

fn fin(v: f64) -> Finite {
    Finite::new(v).expect("finite literal")
}

pub fn model_name(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse.wsp",
        _ => "acme_normal.wsp",
    }
}

/// Reference scales and declared domain of one configuration.
struct Scales {
    spec_id: &'static str,
    injected: &'static str,
    o2_role: &'static str,
    fuel_role: &'static str,
    mdot_ref: f64,
    o2_ref: f64,
    o2_scale: f64,
    xf_ref: f64,
    zst_ref: f64,
    /// (feature, min, max)
    domain: [(&'static str, f64, f64); 6],
}

fn scales(family: Family) -> Scales {
    match family {
        Family::AcmeInverse => Scales {
            spec_id: "acme_inverse-wsp-v1",
            injected: "the injected OXIDISER mass flow (setpoint)",
            o2_role: "O2 mole fraction of the INJECTED stream",
            fuel_role: "fuel mole fraction of the AMBIENT (chamber) gas",
            mdot_ref: 5.0e-6,
            o2_ref: 0.50,
            o2_scale: 0.25,
            xf_ref: 0.10,
            zst_ref: 0.45,
            domain: [
                ("c_logp", -0.92, 1.25),
                ("c_logm", -1.61, 0.88),
                ("c_o2", -1.6, 1.6),
                ("c_xf", -0.70, 1.61),
                ("c_tad", -2.0, 3.0),
                ("c_zst", -1.85, 1.5),
            ],
        },
        _ => Scales {
            spec_id: "acme_normal-wsp-v1",
            injected: "the injected FUEL mass flow (setpoint, not a post-hoc burn rate)",
            o2_role: "O2 mole fraction of the AMBIENT (chamber) gas",
            fuel_role: "fuel mole fraction of the INJECTED stream",
            mdot_ref: 1.0e-6,
            o2_ref: 0.30,
            o2_scale: 0.10,
            xf_ref: 0.30,
            zst_ref: 0.25,
            domain: [
                ("c_logp", -0.92, 1.25),
                ("c_logm", -2.31, 1.39),
                ("c_o2", -1.5, 1.5),
                ("c_xf", -2.71, 1.21),
                ("c_tad", -2.67, 2.33),
                ("c_zst", -1.1, 2.5),
            ],
        },
    }
}

/// The frozen specification of one configuration.
pub fn spec(family: Family) -> ModelSpec {
    use TensorRole::*;
    let sc = scales(family);
    let feature = |name: &str, source: &str, transform: &str| FeatureSpec {
        name: name.into(),
        unit: "1".into(),
        source: source.into(),
        transform: transform.into(),
    };
    let (h, n) = (HIDDEN as u32, N_IN as u32);
    let mut tensors = Vec::new();
    for (p, what) in [
        ("ext", "spontaneous extinction"),
        ("op", "operator termination"),
    ] {
        tensors.extend([
            sparse(
                &format!("{p}.W1"),
                &[h, n],
                Weight,
                None,
                &format!("Hidden weights of the {what} network; row j reads x in feature order."),
            ),
            sparse(
                &format!("{p}.b1"),
                &[h],
                Bias,
                None,
                &format!("Hidden biases of the {what} network (hinge positions)."),
            ),
            sparse(
                &format!("{p}.w2"),
                &[h],
                Weight,
                None,
                &format!("Output weights of the {what} network."),
            ),
            authored(
                &format!("{p}.skip"),
                &[n],
                Weight,
                None,
                None,
                &format!("Linear part of ln(hazard) of {what} in x; every element is stated."),
            ),
            authored(
                &format!("{p}.b0"),
                &[],
                HazardParameter,
                Some("ln(1/s)"),
                Some((-40.0, 10.0)),
                &format!("ln(hazard * 1 s) of {what} at x = 0 before hinge terms."),
            ),
        ]);
    }
    tensors.extend([
        constant(
            "norm.p_ref_pa",
            101_325.0,
            "Pa",
            "Reference pressure of c_logp.",
        ),
        constant(
            "norm.mdot_ref_kg_s",
            sc.mdot_ref,
            "kg/s",
            "Reference injected mass flow of c_logm.",
        ),
        constant("norm.o2_ref", sc.o2_ref, "1", "Centre of c_o2."),
        constant("norm.o2_scale", sc.o2_scale, "1", "Scale of c_o2."),
        constant(
            "norm.xf_ref",
            sc.xf_ref,
            "1",
            "Reference fuel fraction of c_xf.",
        ),
        constant("norm.tad_ref_k", 2200.0, "K", "Centre of c_tad."),
        constant("norm.tad_scale_k", 300.0, "K", "Scale of c_tad."),
        constant("norm.zst_ref", sc.zst_ref, "1", "Centre of c_zst."),
        constant("norm.zst_scale", 0.20, "1", "Scale of c_zst."),
        constant(
            "norm.t_ref_s",
            10.0,
            "s",
            "Time scale of x_t = ln(1 + t / t_ref).",
        ),
        constant(
            "haz.log_cap",
            6.0,
            "1",
            "Upper clamp of each ln(hazard * 1 s) (numerical safety).",
        ),
        constant(
            "num.dt_s",
            0.1,
            "s",
            "Largest integration step; steps end exactly on forecast times.",
        ),
        constant(
            "num.horizon_s",
            600.0,
            "s",
            "Model horizon since the time origin; probability left there is unresolved_support.",
        ),
    ]);
    ModelSpec {
        schema_version: WspSchema::V1,
        spec_id: SpecId::new(sc.spec_id).expect("id"),
        family,
        model_name: model_name(family).into(),
        target: TargetSpec {
            quantity: Quantity::PhaseEndTime,
            unit: "s".into(),
            description: "End of the established hot-flame phase after the cutoff: cumulative                           incidence of spontaneous extinction and of operator termination, the                           probability of still being active, and the mass left at the model                           horizon, under the observed experimental policy. The artifact is an                           event partition over forecast times (kind event_probabilities)."
                .into(),
            excludes: vec![
                "the counterfactual lifetime without the operator".into(),
                "cool-flame phases after hot-flame extinction".into(),
                "flame size, temperature, radiant emission, soot or species".into(),
                "the other configuration (normal and inverse are never pooled)".into(),
                "a calibrated posterior: the probabilities follow from handwritten hazards".into(),
            ],
        },
        features: vec![
            feature("c_logp", "condition:pressure", "ln(x / norm.p_ref_pa) @1"),
            feature("c_logm", "condition:injected_mass_flow",
                    &format!("ln(x / norm.mdot_ref_kg_s); x is {} @1", sc.injected)),
            feature("c_o2", "condition:x_o2_role",
                    &format!("(x - norm.o2_ref) / norm.o2_scale; x is the {} @1", sc.o2_role)),
            feature("c_xf", "condition:x_fuel_role",
                    &format!("ln(x / norm.xf_ref); x is the {} @1", sc.fuel_role)),
            feature("c_tad", "condition:tad", "(x - norm.tad_ref_k) / norm.tad_scale_k @1"),
            feature("c_zst", "condition:zst", "(x - norm.zst_ref) / norm.zst_scale @1"),
            feature("f_ethane", "condition:fuel_is_ethane|fuel_species", "indicator (propane is the reference fuel) @1"),
            feature("f_butane", "condition:fuel_is_butane|fuel_species", "indicator @1"),
            feature("x_t", "state:elapsed_time", "ln(1 + t / norm.t_ref_s), t in seconds since the time origin @1"),
        ],
        domain: sc
            .domain
            .iter()
            .map(|(f, lo, hi)| DomainBound {
                feature: (*f).into(),
                min: Some(fin(*lo)),
                max: Some(fin(*hi)),
            })
            .collect(),
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::acme::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(
            "No time series enters: conditions, cutoff and control stages known at the cutoff only. \
             Hazards are taken constant over each step at its midpoint; within a step the two causes \
             share the lost mass in proportion to their hazards, so the four rows sum to one exactly. \
             A stop known at the cutoff (control `scheduled_stop`, `flow_stop`, or an operator action \
             starting `MFCs off`) at or before the cutoff refuses the forecast; a later one moves the \
             mass still active at that time to operator_termination. Uncertainty kind: assumed_noise."
                .into(),
        ),
    }
}

#[derive(Clone, Copy)]
struct Net<'a> {
    w1: &'a [f64],
    b1: &'a [f64],
    w2: &'a [f64],
    skip: &'a [f64],
    b0: f64,
}

impl Net<'_> {
    #[inline]
    fn eval(&self, x: &[f64; N_IN], scratch: &mut [f64; HIDDEN]) -> f64 {
        hinge_net(self.w1, self.b1, self.w2, self.skip, self.b0, x, scratch)
    }
}

pub(crate) struct Weights<'a> {
    ext: Net<'a>,
    op: Net<'a>,
    p_ref: f64,
    mdot_ref: f64,
    o2_ref: f64,
    o2_scale: f64,
    xf_ref: f64,
    tad_ref: f64,
    tad_scale: f64,
    zst_ref: f64,
    zst_scale: f64,
    pub(crate) t_ref: f64,
    pub(crate) log_cap: f64,
    dt: f64,
    pub(crate) horizon: f64,
}

impl<'a> Weights<'a> {
    pub(crate) fn bind(p: &Params<'a>) -> Self {
        let net = |prefix: &str| Net {
            w1: p.v(&format!("{prefix}.W1")),
            b1: p.v(&format!("{prefix}.b1")),
            w2: p.v(&format!("{prefix}.w2")),
            skip: p.v(&format!("{prefix}.skip")),
            b0: p.s(&format!("{prefix}.b0")),
        };
        Weights {
            ext: net("ext"),
            op: net("op"),
            p_ref: p.s("norm.p_ref_pa"),
            mdot_ref: p.s("norm.mdot_ref_kg_s"),
            o2_ref: p.s("norm.o2_ref"),
            o2_scale: p.s("norm.o2_scale"),
            xf_ref: p.s("norm.xf_ref"),
            tad_ref: p.s("norm.tad_ref_k"),
            tad_scale: p.s("norm.tad_scale_k"),
            zst_ref: p.s("norm.zst_ref"),
            zst_scale: p.s("norm.zst_scale"),
            t_ref: p.s("norm.t_ref_s"),
            log_cap: p.s("haz.log_cap"),
            dt: p.s("num.dt_s"),
            horizon: p.s("num.horizon_s"),
        }
    }

    /// Both hazards (1/s) at time `t` for condition features `c`.
    #[inline]
    pub(crate) fn hazards(&self, c: &[f64; 8], t: f64, scratch: &mut [f64; HIDDEN]) -> (f64, f64) {
        let x = [
            c[0],
            c[1],
            c[2],
            c[3],
            c[4],
            c[5],
            c[6],
            c[7],
            (1.0 + t / self.t_ref).ln(),
        ];
        (
            self.ext.eval(&x, scratch).min(self.log_cap).exp(),
            self.op.eval(&x, scratch).min(self.log_cap).exp(),
        )
    }

    /// The eight condition features of a prefix, or the reasons they cannot be formed.
    pub(crate) fn features(&self, prefix: &PrefixBundle) -> Result<[f64; 8], Vec<String>> {
        let mut reasons = Vec::new();
        let mut get = |k: &str| match common::condition(prefix, k) {
            Ok(v) => v,
            Err(e) => {
                reasons.push(e);
                f64::NAN
            }
        };
        let (p, m, o2, xf, tad, zst) = (
            get("pressure"),
            get("injected_mass_flow"),
            get("x_o2_role"),
            get("x_fuel_role"),
            get("tad"),
            get("zst"),
        );
        for (name, v) in [
            ("pressure", p),
            ("injected_mass_flow", m),
            ("x_fuel_role", xf),
        ] {
            if v.is_finite() && v <= 0.0 {
                reasons.push(format!("{name} = {v} is not positive"));
            }
        }
        // fuel indicators: stored indicators win; otherwise the species text decides, as in the
        // fitted experts. An unknown species is a refusal, not "propane".
        let species = match prefix
            .known_conditions
            .get("fuel_species")
            .and_then(|m| m.raw_value.as_ref())
        {
            Some(RawValue::Text(s)) => Some(s.to_lowercase()),
            _ => None,
        };
        let mut indicator = |key: &str, fuel: &str| -> f64 {
            if let Some(v) = prefix.known_conditions.get(key).and_then(|m| m.value_si) {
                return v;
            }
            match &species {
                Some(s) if ["ethane", "propane", "butane"].contains(&s.as_str()) => {
                    f64::from(u8::from(s == fuel))
                }
                _ => {
                    reasons.push(format!(
                        "{key} missing and fuel_species is not ethane, propane or butane"
                    ));
                    f64::NAN
                }
            }
        };
        let (fe, fb) = (
            indicator("fuel_is_ethane", "ethane"),
            indicator("fuel_is_butane", "butane"),
        );
        if !reasons.is_empty() {
            reasons.dedup();
            return Err(reasons);
        }
        Ok([
            (p / self.p_ref).ln(),
            (m / self.mdot_ref).ln(),
            (o2 - self.o2_ref) / self.o2_scale,
            (xf / self.xf_ref).ln(),
            (tad - self.tad_ref) / self.tad_scale,
            (zst - self.zst_ref) / self.zst_scale,
            fe,
            fb,
        ])
    }
}

/// Earliest stop known at the cutoff, as the fitted experts read it from the control stages.
fn scheduled_stop(controls: &[ControlStage]) -> Option<f64> {
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

/// The numbers of one forecast.
#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    pub features: [f64; 8],
    pub cutoff_s: f64,
    pub scheduled_stop_s: Option<f64>,
    pub times_s: Vec<f64>,
    /// Rows in [`CATEGORIES`] order.
    pub table: [Vec<f64>; 4],
    /// Hazards (1/s) at the forecast times: extinction, operator termination.
    pub hazard_ext: Vec<f64>,
    pub hazard_op: Vec<f64>,
    pub steps: usize,
}

fn integrate(
    wt: &Weights<'_>,
    c: &[f64; 8],
    tau: f64,
    stop: Option<f64>,
    times: &[f64],
) -> Forecast {
    let nt = times.len();
    let mut table: [Vec<f64>; 4] = std::array::from_fn(|_| vec![0.0; nt]);
    let (mut hz_e, mut hz_o) = (vec![0.0; nt], vec![0.0; nt]);
    let mut scratch = [0.0; HIDDEN];
    let (mut t, mut s, mut f_e, mut f_o) = (tau, 1.0_f64, 0.0_f64, 0.0_f64);
    // Hazards act until the model horizon or a known stop, whichever is first.
    let end = stop.map_or(wt.horizon, |x| x.min(wt.horizon));
    let mut steps = 0usize;
    for (j, &tj) in times.iter().enumerate() {
        let target = tj.min(end);
        if target > t {
            let n_sub = (((target - t) / wt.dt) - 1e-9).ceil().max(1.0) as usize;
            let h = (target - t) / n_sub as f64;
            for _ in 0..n_sub {
                let (le, lo) = wt.hazards(c, t + 0.5 * h, &mut scratch);
                let total = le + lo;
                if total > 0.0 {
                    let lost = s * -(-total * h).exp_m1();
                    f_e += lost * le / total;
                    f_o += lost * lo / total;
                    s -= lost;
                }
                t += h;
                steps += 1;
            }
            t = target;
        }
        let (le, lo) = wt.hazards(c, tj, &mut scratch);
        hz_e[j] = le;
        hz_o[j] = lo;
        let stopped = stop.is_some_and(|x| tj >= x && x <= wt.horizon);
        let beyond = tj >= wt.horizon && !stopped;
        table[0][j] = if stopped || beyond { 0.0 } else { s };
        table[1][j] = f_e;
        table[2][j] = f_o + if stopped { s } else { 0.0 };
        table[3][j] = if beyond { s } else { 0.0 };
    }
    Forecast {
        features: *c,
        cutoff_s: tau,
        scheduled_stop_s: stop,
        times_s: times.to_vec(),
        table,
        hazard_ext: hz_e,
        hazard_op: hz_o,
        steps,
    }
}

/// Strict forward of an ACME literal release on an admitted prefix: `Ok(Err(support))` is a
/// scientific refusal, `Err(refusal)` a release or request that cannot be run at all.
pub fn forecast(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<Result<Forecast, Support>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params);
    let built_with = release
        .release
        .construction
        .as_ref()
        .map(|m| &m.forward_hash);
    let this = crate::audit::forward_hash(spec.family);
    if built_with != Some(&this) {
        return Err(Refusal::unsupported(
            "forward_mismatch",
            format!(
                "release {} was assembled with forward {}, this build runs {this}: a release is its \
                 weights and its forward (re-assemble, or run the build it names)",
                release.release.id,
                built_with.map_or("none".to_string(), |d| d.to_string()),
            ),
        ));
    }
    if matches!(
        release.release.state,
        QualificationState::Failed | QualificationState::Unsupported
    ) {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!("release state {:?}", release.release.state),
        )));
    }
    if prefix.family != spec.family {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!(
                "a {} prefix cannot be forecast by the {} model: the configurations are never pooled",
                prefix.family.as_str(),
                spec.family.as_str()
            ),
        )));
    }
    if prefix.phase_established_at_s.is_none() {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            "established phase is not confirmed at the cutoff (population criterion)",
        )));
    }
    let tau = prefix.cutoff_s;
    let stop = scheduled_stop(&prefix.known_controls);
    if stop.is_some_and(|x| x <= tau) {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            "a stop at or before the cutoff: the fed phase has ended",
        )));
    }
    let c = match wt.features(prefix) {
        Ok(c) => c,
        Err(reasons) => {
            return Ok(Err(Support {
                admitted: false,
                gate: SupportGate::Condition,
                reasons,
            }))
        }
    };
    let mut outside = Vec::new();
    for d in &spec.domain {
        if let Some(i) = FEATURES
            .iter()
            .position(|n| *n == d.feature)
            .filter(|i| *i < 8)
        {
            let lo = d.min.map_or(f64::NEG_INFINITY, Finite::get);
            let hi = d.max.map_or(f64::INFINITY, Finite::get);
            if !(c[i] >= lo && c[i] <= hi) {
                outside.push(format!("{} = {:.4} outside [{lo}, {hi}]", d.feature, c[i]));
            }
        }
    }
    if !outside.is_empty() {
        return Ok(Err(Support {
            admitted: false,
            gate: SupportGate::Condition,
            reasons: outside,
        }));
    }
    if !(tau >= 0.0 && tau < wt.horizon) {
        return Ok(Err(refuse(
            SupportGate::Horizon,
            format!("cutoff {tau} s outside [0, {}) s", wt.horizon),
        )));
    }
    let mut times = common::times_of(settings, tau, 60.0, wt.horizon);
    times.retain(|t| t.is_finite() && *t >= tau);
    times.dedup();
    if times.is_empty() {
        return Ok(Err(refuse(
            SupportGate::Horizon,
            "no forecast time at or after the cutoff",
        )));
    }
    Ok(Ok(integrate(&wt, &c, tau, stop, &times)))
}

fn body(f: Forecast) -> Body {
    let diagnostics = json!({
        "interpretation": INTERPRETATION,
        "features": FEATURES[..8].iter().zip(f.features).map(|(k, v)| (k.to_string(), json!(v))).collect::<serde_json::Map<_, _>>(),
        "scheduled_stop_s": f.scheduled_stop_s,
        "hazard_extinction_per_s": f.hazard_ext,
        "hazard_operator_per_s": f.hazard_op,
        "uncertainty_kind": "assumed_noise",
        "integration_steps": f.steps,
    });
    Body {
        kind: Some(ArtifactKind::EventProbabilities),
        quantity: None,
        times_s: f.times_s,
        quantile_levels: vec![],
        quantiles: vec![],
        mean: vec![],
        event_categories: CATEGORIES.iter().map(|s| s.to_string()).collect(),
        event_cdf: f.table.to_vec(),
        diagnostics,
    }
}

/// The artifact of a forecast or of a scientific refusal.
pub fn artifact(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<AnalysisArtifact, Refusal> {
    Ok(match forecast(spec, release, prefix, settings)? {
        Ok(f) => common::artifact(release, prefix, settings, admitted(), body(f)),
        Err(support) => common::artifact(
            release,
            prefix,
            settings,
            support,
            Body {
                kind: Some(ArtifactKind::EventProbabilities),
                ..Body::default()
            },
        ),
    })
}

/// One ACME configuration behind the common expert interface.
pub struct AcmeWsp {
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl AcmeWsp {
    pub fn new(family: Family) -> Self {
        assert!(matches!(family, Family::AcmeNormal | Family::AcmeInverse));
        AcmeWsp {
            spec: spec(family),
            policy: ProspectivePolicy::new(family, &ALLOWED, &FORBIDDEN),
        }
    }
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }
}

impl Expert for AcmeWsp {
    fn family(&self) -> Family {
        self.spec.family
    }
    fn model_name(&self) -> &str {
        &self.spec.model_name
    }
    fn is_baseline(&self) -> bool {
        false
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }
    fn default_config(&self) -> ExpertResult<Value> {
        Ok(json!({"construction": "literal", "spec_id": self.spec.spec_id}))
    }
    fn estimate_hours(&self, _: &Value, _: usize, _: usize, _: &Profile) -> f64 {
        0.0
    }
    fn fit(&self, _: &FitData<'_>, _: &Value, _: &mut FitContext) -> ExpertResult<ModelRelease> {
        bail!(
            "{} has no fitting procedure: its release is assembled from a literal patch (`ignis wsp assemble`)",
            self.spec.model_name
        )
    }
    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact> {
        match forecast(&self.spec, release, prefix, settings)? {
            Ok(f) => Ok(common::artifact(
                release,
                prefix,
                settings,
                admitted(),
                body(f),
            )),
            Err(support) => Err(common::refusal_of(&support).into()),
        }
    }
    fn log_likelihood(&self, _: &LoadedRelease, _: &View) -> ExpertResult<f64> {
        bail!(
            "{} exposes event probabilities, not a likelihood",
            self.spec.model_name
        )
    }
}

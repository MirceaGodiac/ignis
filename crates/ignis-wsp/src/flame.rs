//! `acme_normal.wsp2` and `acme_inverse.wsp2` — the clock forward of the ACME burner flames.
//!
//! The hot flame of a spherical burner-stabilised flame ends in one of two ways, and each has its own
//! clocks, written here as what they are instead of as a generic hazard surface:
//!
//! ```text
//! c = (c_logp, c_logm, c_o2, c_xf, c_tad, c_zst, f_ethane, f_butane)
//!
//! self-extinction
//!   weak class   probability sigmoid(N_flash(c)): the flame goes out within seconds of ignition,
//!                T ~ log-logistic(flash_tau, flash_shape)
//!   main class   T = floor + R,  R ~ log-logistic(exp(N_tau(c)), shape):
//!                the flame grows until radiation takes it; nothing extinguishes radiatively before
//!                the flame is established (floor)
//! flow termination
//!   burner limit T ~ log-logistic(exp(N_bt(c)), shape_bt): a small flame sits near the burner and
//!                heats it to its temperature limit
//!   programmed   at each written time T_a a flame still burning is stopped with probability
//!                sigmoid(logit_a + W_a . c)  (script durations and time-outs of the campaign)
//!   background   constant hazard exp(log_base)
//!
//! N(x) = b0 + skip . x + sum_j w2[j] relu(b1[j] + W1[j, :] . x)
//! log-logistic hazard: h(t) = (k / tau) (t / tau)^(k-1) / (1 + (t / tau)^k)
//! ```
//!
//! The causes compete; the forecast is the partition `active, extinction, operator_termination,
//! unresolved_support` conditional on the flame burning at the cutoff. The weak and main classes
//! are latent: a flame alive at the cutoff re-weights them by their survival to the cutoff. An
//! operator stop is never counted as an extinction, and `extinction` here is what the PSI-159
//! tables call self-extinction: the end of the hot flame. A cool flame that follows it is outside
//! this model.
//!
//! Normal and inverse share this code and nothing else: separate specs, reference scales, patches
//! and releases. The prefix carries no time series for ACME. Nothing is fitted on this route.

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
use crate::nn::{hinge_net, sigmoid};
use crate::spec::{authored, constant, sparse, Params};

pub const FORWARD_VERSION: &str = "2";
/// Hidden width of each law network.
pub const HIDDEN: usize = 8;
/// Condition features.
pub const N_IN: usize = 8;
/// Programmed-stop atoms.
pub const N_ATOMS: usize = 8;
pub const FEATURES: [&str; N_IN] = [
    "c_logp", "c_logm", "c_o2", "c_xf", "c_tad", "c_zst", "f_ethane", "f_butane",
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
    policy; operator termination is not spontaneous extinction; extinction is the end of the hot flame";

fn fin(v: f64) -> Finite {
    Finite::new(v).expect("finite literal")
}

pub fn model_name(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse.wsp2",
        _ => "acme_normal.wsp2",
    }
}

pub fn family_of_model(name: &str) -> Option<Family> {
    [Family::AcmeNormal, Family::AcmeInverse]
        .into_iter()
        .find(|f| model_name(*f) == name)
}

/// Reference scales and declared domain of one configuration (the same as the first generation).
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
    domain: [(&'static str, f64, f64); 6],
}

fn scales(family: Family) -> Scales {
    match family {
        Family::AcmeInverse => Scales {
            spec_id: "acme_inverse-wsp-v2",
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
            spec_id: "acme_normal-wsp-v2",
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
    let (h, n, a) = (HIDDEN as u32, N_IN as u32, N_ATOMS as u32);
    let mut tensors = Vec::new();
    for (p, what, role, unit, range) in [
        (
            "se.tau",
            "the radiative self-extinction clock ln(tau_R / 1 s) (time after the floor)",
            PhysicalParameter,
            "ln(s)",
            (-5.0, 12.0),
        ),
        (
            "se.flash",
            "the weak-flame class logit",
            HazardParameter,
            "logit(1)",
            (-40.0, 40.0),
        ),
        (
            "ft.tau",
            "the burner-limit clock ln(tau_B / 1 s)",
            PhysicalParameter,
            "ln(s)",
            (-2.0, 14.0),
        ),
    ] {
        tensors.extend([
            sparse(
                &format!("{p}.W1"),
                &[h, n],
                Weight,
                None,
                &format!("Hidden weights of {what}; row j reads c in feature order."),
            ),
            sparse(
                &format!("{p}.b1"),
                &[h],
                Bias,
                None,
                &format!("Hidden biases of {what} (hinge positions)."),
            ),
            sparse(
                &format!("{p}.w2"),
                &[h],
                Weight,
                None,
                &format!("Output weights of {what}."),
            ),
            authored(
                &format!("{p}.skip"),
                &[n],
                Weight,
                None,
                None,
                &format!("Linear part of {what} in c; every element is stated."),
            ),
            authored(
                &format!("{p}.b0"),
                &[],
                role,
                Some(unit),
                Some(range),
                &format!("Value of {what} at c = 0 before hinge terms."),
            ),
        ]);
    }
    tensors.extend([
        authored("se.floor_s", &[], PhysicalParameter, Some("s"), Some((0.0, 60.0)),
                 "Time after ignition before which the main class cannot self-extinguish (flame establishment)."),
        authored("se.shape", &[], HazardParameter, Some("1"), Some((0.5, 30.0)),
                 "Log-logistic shape of the radiative clock (sd of ln R = pi / (shape sqrt 3))."),
        authored("se.flash_tau_s", &[], PhysicalParameter, Some("s"), Some((0.05, 30.0)),
                 "Median life of a weak flame."),
        authored("se.flash_shape", &[], HazardParameter, Some("1"), Some((0.5, 30.0)),
                 "Log-logistic shape of the weak-flame clock."),
        authored("ft.shape", &[], HazardParameter, Some("1"), Some((0.5, 30.0)),
                 "Log-logistic shape of the burner-limit clock."),
        authored("ft.atom_t_s", &[a], PhysicalParameter, Some("s"), Some((0.0, 1000.0)),
                 "Times of programmed stops (script durations and time-outs), seconds since the time origin."),
        authored("ft.atom_logit", &[a], HazardParameter, Some("logit(1)"), Some((-40.0, 40.0)),
                 "Logit of the probability that a flame burning at ft.atom_t_s[a] is stopped there, at c = 0; -40 switches an atom off."),
        sparse("ft.atom_w", &[a, n], Weight, None,
               "Condition dependence of each programmed-stop logit; row a reads c in feature order."),
        authored("ft.log_base", &[], HazardParameter, Some("ln(1/s)"), Some((-40.0, 5.0)),
                 "ln of the constant background hazard of an operator stop."),
        constant("norm.p_ref_pa", 101_325.0, "Pa", "Reference pressure of c_logp."),
        constant("norm.mdot_ref_kg_s", sc.mdot_ref, "kg/s", "Reference injected mass flow of c_logm."),
        constant("norm.o2_ref", sc.o2_ref, "1", "Reference O2 mole fraction of c_o2."),
        constant("norm.o2_scale", sc.o2_scale, "1", "Scale of c_o2."),
        constant("norm.xf_ref", sc.xf_ref, "1", "Reference fuel mole fraction of c_xf."),
        constant("norm.tad_ref_k", 2200.0, "K", "Reference adiabatic flame temperature of c_tad."),
        constant("norm.tad_scale_k", 300.0, "K", "Scale of c_tad."),
        constant("norm.zst_ref", sc.zst_ref, "1", "Reference stoichiometric mixture fraction of c_zst."),
        constant("norm.zst_scale", 0.2, "1", "Scale of c_zst."),
        constant("num.atom_half_s", 0.5, "s", "Half-width over which a programmed stop is spread (burn times are recorded to the second)."),
        constant("num.dt_s", 0.1, "s", "Largest integration step."),
        constant("num.horizon_s", 600.0, "s", "Model horizon since the time origin."),
    ]);
    ModelSpec {
        schema_version: WspSchema::V1,
        spec_id: SpecId::new(sc.spec_id).expect("id"),
        family,
        model_name: model_name(family).into(),
        target: TargetSpec {
            quantity: Quantity::PhaseEndTime,
            unit: "s".into(),
            description: "When and why the established hot flame of a burner-stabilised spherical flame \
                          ends: cumulative incidence of self-extinction and of flow termination by the \
                          operator or the script, as a partition with the flame still active."
                .into(),
            excludes: vec![
                "cool flames after the hot flame (the PSI-159 burn time of a self-extinguished test ends with the hot flame)".into(),
                "flame radius, temperature, radiant emission".into(),
                "a causal effect of an operator action: the stops are the observed campaign policy".into(),
                "a calibrated posterior: the spread is an assumed clock distribution".into(),
            ],
        },
        features: vec![
            feature("c_logp", "condition:pressure", "ln(x / norm.p_ref_pa) @1"),
            feature("c_logm", "condition:injected_mass_flow", &format!("ln(x / norm.mdot_ref_kg_s); x is {} @1", sc.injected)),
            feature("c_o2", "condition:x_o2_role", &format!("(x - norm.o2_ref) / norm.o2_scale; x is the {} @1", sc.o2_role)),
            feature("c_xf", "condition:x_fuel_role", &format!("ln(x / norm.xf_ref); x is the {} @1", sc.fuel_role)),
            feature("c_tad", "condition:tad", "(x - norm.tad_ref_k) / norm.tad_scale_k @1"),
            feature("c_zst", "condition:zst", "(x - norm.zst_ref) / norm.zst_scale @1"),
            feature("f_ethane", "condition:fuel_is_ethane|fuel_species", "indicator (propane is the reference fuel) @1"),
            feature("f_butane", "condition:fuel_is_butane|fuel_species", "indicator @1"),
        ],
        domain: sc
            .domain
            .iter()
            .map(|(f, lo, hi)| DomainBound { feature: (*f).into(), min: Some(fin(*lo)), max: Some(fin(*hi)) })
            .collect(),
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::flame::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(
            "No time series enters: conditions, cutoff and control stages known at the cutoff only. \
             Two latent classes (weak, main) are integrated from the time origin; their weights at \
             the cutoff are their prior probabilities times their survival to the cutoff, normalised. \
             Hazards are taken constant over each step at its midpoint; within a step the causes share \
             the lost mass in proportion to their hazards, so the four rows sum to one exactly. A \
             programmed stop a is a hazard -ln(1 - p_a) / (2 num.atom_half_s) on \
             [T_a - num.atom_half_s, T_a + num.atom_half_s]. A stop known at the cutoff (control \
             `scheduled_stop`, `flow_stop`, or an operator action starting `MFCs off`) at or before \
             the cutoff refuses the forecast; a later one moves the mass still active at that time to \
             operator_termination. Uncertainty kind: assumed_noise."
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
    se_tau: Net<'a>,
    se_flash: Net<'a>,
    ft_tau: Net<'a>,
    floor: f64,
    se_shape: f64,
    flash_tau: f64,
    flash_shape: f64,
    ft_shape: f64,
    atom_t: &'a [f64],
    atom_logit: &'a [f64],
    atom_w: &'a [f64],
    base: f64,
    p_ref: f64,
    mdot_ref: f64,
    o2_ref: f64,
    o2_scale: f64,
    xf_ref: f64,
    tad_ref: f64,
    tad_scale: f64,
    zst_ref: f64,
    zst_scale: f64,
    atom_half: f64,
    dt: f64,
    pub(crate) horizon: f64,
}

/// The condition-level clocks of one flame (reported in the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct Clocks {
    pub p_weak: f64,
    pub weak_tau_s: f64,
    pub floor_s: f64,
    pub radiative_tau_s: f64,
    pub burner_tau_s: f64,
    /// `(time, probability)` of each programmed stop that is switched on.
    pub programmed: Vec<(f64, f64)>,
    pub background_per_s: f64,
}

#[inline]
fn log_logistic(t: f64, tau: f64, k: f64) -> f64 {
    if !(t > 0.0) || !(tau > 0.0) {
        return 0.0;
    }
    let r = (t / tau).powf(k);
    (k / t) * r / (1.0 + r)
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
            se_tau: net("se.tau"),
            se_flash: net("se.flash"),
            ft_tau: net("ft.tau"),
            floor: p.s("se.floor_s"),
            se_shape: p.s("se.shape"),
            flash_tau: p.s("se.flash_tau_s"),
            flash_shape: p.s("se.flash_shape"),
            ft_shape: p.s("ft.shape"),
            atom_t: p.v("ft.atom_t_s"),
            atom_logit: p.v("ft.atom_logit"),
            atom_w: p.v("ft.atom_w"),
            base: p.s("ft.log_base").exp(),
            p_ref: p.s("norm.p_ref_pa"),
            mdot_ref: p.s("norm.mdot_ref_kg_s"),
            o2_ref: p.s("norm.o2_ref"),
            o2_scale: p.s("norm.o2_scale"),
            xf_ref: p.s("norm.xf_ref"),
            tad_ref: p.s("norm.tad_ref_k"),
            tad_scale: p.s("norm.tad_scale_k"),
            zst_ref: p.s("norm.zst_ref"),
            zst_scale: p.s("norm.zst_scale"),
            atom_half: p.s("num.atom_half_s"),
            dt: p.s("num.dt_s"),
            horizon: p.s("num.horizon_s"),
        }
    }

    pub(crate) fn clocks(&self, c: &[f64; N_IN]) -> Clocks {
        let mut scratch = [0.0; HIDDEN];
        let mut programmed = Vec::new();
        for a in 0..N_ATOMS {
            let row = &self.atom_w[a * N_IN..(a + 1) * N_IN];
            let logit = self.atom_logit[a] + row.iter().zip(c).map(|(w, x)| w * x).sum::<f64>();
            let p = sigmoid(logit);
            if p > 1.0e-9 && self.atom_t[a] > 0.0 {
                programmed.push((self.atom_t[a], p.min(1.0 - 1.0e-9)));
            }
        }
        Clocks {
            p_weak: sigmoid(self.se_flash.eval(c, &mut scratch)),
            weak_tau_s: self.flash_tau,
            floor_s: self.floor,
            radiative_tau_s: self.se_tau.eval(c, &mut scratch).exp(),
            burner_tau_s: self.ft_tau.eval(c, &mut scratch).exp(),
            programmed,
            background_per_s: self.base,
        }
    }

    /// Hazards (1/s) of self-extinction and of flow termination at time `t` for one class.
    #[inline]
    fn hazards(&self, k: &Clocks, weak: bool, t: f64) -> (f64, f64) {
        let se = if weak {
            log_logistic(t, k.weak_tau_s, self.flash_shape)
        } else {
            log_logistic(t - k.floor_s, k.radiative_tau_s, self.se_shape)
        };
        let mut ft = log_logistic(t, k.burner_tau_s, self.ft_shape) + k.background_per_s;
        for &(ta, p) in &k.programmed {
            if (t - ta).abs() <= self.atom_half {
                ft += -(1.0 - p).ln() / (2.0 * self.atom_half);
            }
        }
        (se, ft)
    }

    /// The eight condition features of a prefix, or the reasons they cannot be formed.
    pub(crate) fn features(&self, prefix: &PrefixBundle) -> Result<[f64; N_IN], Vec<String>> {
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
        // fuel indicators: stored indicators win; otherwise the species text decides. An unknown
        // species is a refusal, not "propane".
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
    pub features: [f64; N_IN],
    pub clocks: Clocks,
    pub cutoff_s: f64,
    /// Probability of the weak class given a flame burning at the cutoff.
    pub p_weak_at_cutoff: f64,
    pub scheduled_stop_s: Option<f64>,
    pub times_s: Vec<f64>,
    /// Rows in [`CATEGORIES`] order.
    pub table: [Vec<f64>; 4],
    pub steps: usize,
}

/// One class from `t0` (survival 1) to each time: `(survival, incidence of extinction, incidence of
/// termination)` at every requested time, hazards switched off after `end`.
fn run_class(
    wt: &Weights<'_>,
    k: &Clocks,
    weak: bool,
    t0: f64,
    end: f64,
    times: &[f64],
    steps: &mut usize,
) -> Vec<(f64, f64, f64)> {
    let (mut t, mut s, mut f_e, mut f_o) = (t0, 1.0_f64, 0.0_f64, 0.0_f64);
    let mut out = Vec::with_capacity(times.len());
    for &tj in times {
        let target = tj.min(end);
        if target > t {
            let n_sub = (((target - t) / wt.dt) - 1e-9).ceil().max(1.0) as usize;
            let h = (target - t) / n_sub as f64;
            for _ in 0..n_sub {
                let (le, lo) = wt.hazards(k, weak, t + 0.5 * h);
                let total = le + lo;
                if total > 0.0 {
                    let lost = s * -(-total * h).exp_m1();
                    f_e += lost * le / total;
                    f_o += lost * lo / total;
                    s -= lost;
                }
                t += h;
                *steps += 1;
            }
            t = target;
        }
        out.push((s, f_e, f_o));
    }
    out
}

fn integrate(
    wt: &Weights<'_>,
    c: &[f64; N_IN],
    tau: f64,
    stop: Option<f64>,
    times: &[f64],
) -> Forecast {
    let nt = times.len();
    let clocks = wt.clocks(c);
    let end = stop.map_or(wt.horizon, |x| x.min(wt.horizon));
    let mut steps = 0usize;
    // survival of each class from the time origin to the cutoff
    let to_cut =
        |weak: bool, steps: &mut usize| run_class(wt, &clocks, weak, 0.0, tau, &[tau], steps)[0].0;
    let (s_weak, s_main) = (to_cut(true, &mut steps), to_cut(false, &mut steps));
    let (w_weak, w_main) = (clocks.p_weak * s_weak, (1.0 - clocks.p_weak) * s_main);
    let total = w_weak + w_main;
    let p_weak = if total > 0.0 {
        w_weak / total
    } else {
        clocks.p_weak
    };
    let weak = run_class(wt, &clocks, true, tau, end, times, &mut steps);
    let main = run_class(wt, &clocks, false, tau, end, times, &mut steps);
    let mut table: [Vec<f64>; 4] = std::array::from_fn(|_| vec![0.0; nt]);
    for (j, &tj) in times.iter().enumerate() {
        let mix = |a: f64, b: f64| p_weak * a + (1.0 - p_weak) * b;
        let (s, f_e, f_o) = (
            mix(weak[j].0, main[j].0),
            mix(weak[j].1, main[j].1),
            mix(weak[j].2, main[j].2),
        );
        let stopped = stop.is_some_and(|x| tj >= x && x <= wt.horizon);
        let beyond = tj >= wt.horizon && !stopped;
        table[0][j] = if stopped || beyond { 0.0 } else { s };
        table[1][j] = f_e;
        table[2][j] = f_o + if stopped { s } else { 0.0 };
        table[3][j] = if beyond { s } else { 0.0 };
    }
    Forecast {
        features: *c,
        clocks,
        cutoff_s: tau,
        p_weak_at_cutoff: p_weak,
        scheduled_stop_s: stop,
        times_s: times.to_vec(),
        table,
        steps,
    }
}

/// Strict forward of an ACME clock release on an admitted prefix: `Ok(Err(support))` is a scientific
/// refusal, `Err(refusal)` a release or request that cannot be run at all.
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
    let this = crate::audit::forward_hash_of(&spec.model_name);
    if built_with != Some(&this) {
        return Err(Refusal::unsupported(
            "forward_mismatch",
            format!(
                "release {} was assembled with forward {}, this build runs {this}: a release is its weights and its forward (re-assemble, or run the build it names)",
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
        if let Some(i) = FEATURES.iter().position(|n| *n == d.feature) {
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
        "features": FEATURES.iter().zip(f.features).map(|(k, v)| (k.to_string(), json!(v))).collect::<serde_json::Map<_, _>>(),
        "clocks": f.clocks,
        "p_weak_at_cutoff": f.p_weak_at_cutoff,
        "scheduled_stop_s": f.scheduled_stop_s,
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

/// The condition-level clocks of a release for one feature vector (inspection; no data).
pub fn clocks_of(
    spec: &ModelSpec,
    release: &LoadedRelease,
    c: [f64; N_IN],
) -> Result<Clocks, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    Ok(Weights::bind(&params).clocks(&c))
}

/// One ACME configuration behind the common expert interface.
pub struct FlameWsp {
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl FlameWsp {
    pub fn new(family: Family) -> Self {
        assert!(matches!(family, Family::AcmeNormal | Family::AcmeInverse));
        FlameWsp {
            spec: spec(family),
            policy: ProspectivePolicy::new(family, &ALLOWED, &FORBIDDEN),
        }
    }
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }
}

impl Expert for FlameWsp {
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
        bail!("{} has no fitting procedure: its release is assembled from a literal patch (`ignis wsp assemble`)", self.spec.model_name)
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

//! `acme_normal.wsp8` and `acme_inverse.wsp8` — the hinge-law clocks of the ACME burner flames with
//! the burner-heating law as their burner-limit clock.
//!
//! The clock forward (`flame.rs`) writes the time at which a burner reaches its temperature limit as
//! a log-logistic clock in the conditions. That clock knows the fuel flow and nothing of the burner:
//! it gives a limit trip at 15 to 17 s to a cool flame at a low flow. The growth-clock forward
//! (`flame3.rs`) writes the same event as what it is, a burner heated by the flame towards an
//! equilibrium temperature that depends on the total flow of the burner stream and on the radius
//! the flame settles at, tripping when it crosses one of the campaign's limit settings. This forward
//! is the first with the burner side of the second and nothing else changed:
//!
//! ```text
//! c = (c_logp, c_logm, c_o2, c_xf, c_tad, c_zst, f_ethane, f_butane)
//!
//! self-extinction                                                  (as `flame.rs`)
//!   weak class   probability sigmoid(N_flash(c)): the flame goes out within seconds of ignition,
//!                T ~ log-logistic(flash_tau, flash_shape)
//!   main class   T = floor + R,  R ~ log-logistic(exp(N_tau(c)), shape)
//! flow termination
//!   burner limit                                                   (as `flame3.rs`)
//!     burner side  normal: fuel stream, L = ln(1 / (1 - Z_st));  inverse: oxidiser stream, L = ln(1 / Z_st)
//!       m_tot      total mass flow of the burner stream = injected_mass_flow / Y_R (reactant in N2)
//!       r_ss       = m_tot / (4 pi rhoD L), the radius the flame grows towards
//!     b = (b_rss, b_mtot, c_tad, c_xf, c_logp, f_ethane, f_butane),
//!         b_rss = ln(r_ss / norm.r_ref_mm), b_mtot = ln(m_tot / norm.mtot_ref_kg_s)
//!     T_b(t) = t0 + dT (1 - exp(-t / tau_b))          burner heated towards t0 + dT
//!     ln dT_j = N_rise(b) + bt.sigma z_j              num.bt_nodes equally likely heating nodes
//!     t_jl = -tau_b ln(1 - (limit_l - t0) / dT_j)     when dT_j > limit_l - t0, else never
//!     S_bt(t) = sum_jl (w_l / sum w) / n_bt x (1 / (1 + (t / t_jl)^bt.shape), or 1 when never)
//!   programmed   at each written time T_a a flame still burning is stopped with probability
//!                sigmoid(logit_a + W_a . c)                        (as `flame.rs`)
//!   background   constant hazard exp(log_base)                     (as `flame.rs`)
//!
//! N(x) = b0 + skip . x + sum_j w2[j] relu(b1[j] + W1[j, :] . x)
//! log-logistic hazard: h(t) = (k / tau) (t / tau)^(k-1) / (1 + (t / tau)^k)
//! ```
//!
//! The integration is that of `flame.rs`: two latent classes from the time origin, re-weighted by
//! their survival to the cutoff, hazards constant over each step at its midpoint, the causes sharing
//! the lost mass of a step in proportion to their hazards. Where `flame.rs` adds the hazard of its
//! burner-limit clock, a step from `t` to `t + h` adds `ln(S_bt(t) / S_bt(t + h)) / h`: the hazard,
//! constant over the step, whose integral is the loss of burner-limit survival over it. The burner
//! limit does not depend on the self-extinction class, so both classes see the same one.
//!
//! The burner side needs what `flame3.rs` needs of a prefix, and refuses what it refuses: a
//! stoichiometric mixture fraction outside (0, 1), a burner-side reactant mole fraction outside
//! (0, 1], a fuel indicator other than 0 or 1, a weighted limit at or below the burner temperature
//! at ignition.
//!
//! Neither earlier forward is edited or called: this file repeats the lines it shares with them.
//! Its spec takes its tensors from theirs, name for name. Normal and inverse share this code and
//! nothing else. The prefix carries no time series for ACME. Nothing is fitted on this route.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, ControlStage, Family, Finite, ForwardContract, ForwardProfile,
    ModelRelease, ModelSpec, PrefixBundle, QualificationState, RawValue, SpecId, Support,
    SupportGate, TensorSpec,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::LoadedRelease;
use ignis_core::resources::Profile;
use ignis_core::views::View;

use crate::common::{self, admitted, refuse, Body};
use crate::nn::{hinge_net, sigmoid};
use crate::quad::normal_quantile;
use crate::spec::Params;

pub const FORWARD_VERSION: &str = "8";
/// Most nodes of the burner-heating distribution a spec may ask for.
pub const MAX_NODES: usize = 64;
/// Hidden width of each condition law network.
pub const HIDDEN: usize = 8;
/// Condition features.
pub const N_IN: usize = 8;
/// Hidden width of the burner-rise network.
pub const HIDDEN_B: usize = 4;
/// Burner features.
pub const N_B: usize = 7;
/// Limit settings of the burner-heating clock.
pub const N_LIMITS: usize = 3;
/// Programmed-stop atoms.
pub const N_ATOMS: usize = 8;
pub const FEATURES: [&str; N_IN] = [
    "c_logp", "c_logm", "c_o2", "c_xf", "c_tad", "c_zst", "f_ethane", "f_butane",
];
/// The burner features in the order `bt.rise` reads them.
pub const BURNER_FEATURES: [&str; N_B] = [
    "b_rss", "b_mtot", "c_tad", "c_xf", "c_logp", "f_ethane", "f_butane",
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

pub fn model_name(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse.wsp8",
        _ => "acme_normal.wsp8",
    }
}

pub fn family_of_model(name: &str) -> Option<Family> {
    [Family::AcmeNormal, Family::AcmeInverse]
        .into_iter()
        .find(|f| model_name(*f) == name)
}

/// A tensor of the `flame.rs` spec that belongs to its burner-limit clock, which this forward does
/// not have.
pub fn is_old_limit_clock(name: &str) -> bool {
    name.starts_with("ft.tau.") || name == "ft.shape"
}

/// A tensor of the `flame3.rs` spec that its burner-limit clock reads: the rise network and the
/// heating law, rhoD of the flame radius, the reference scales of the two burner features, the
/// molar masses of the burner stream and the number of heating nodes.
pub fn is_burner_tensor(name: &str) -> bool {
    name.starts_with("bt.")
        || name.starts_with("phys.mw_")
        || [
            "se.rho_d_g_m_s",
            "norm.r_ref_mm",
            "norm.mtot_ref_kg_s",
            "num.bt_nodes",
        ]
        .contains(&name)
}

/// The frozen specification of one configuration: the tensors of the `flame.rs` spec without its
/// burner-limit clock, then the tensors the burner-limit clock of the `flame3.rs` spec reads, each
/// as it stands in its own spec (name, shape, role, bounds, fixed value).
pub fn spec(family: Family) -> ModelSpec {
    assert!(matches!(family, Family::AcmeNormal | Family::AcmeInverse));
    // built once per process and family: it is made of two other specs and their hashes
    static NORMAL: std::sync::OnceLock<ModelSpec> = std::sync::OnceLock::new();
    static INVERSE: std::sync::OnceLock<ModelSpec> = std::sync::OnceLock::new();
    let cell = if family == Family::AcmeInverse {
        &INVERSE
    } else {
        &NORMAL
    };
    cell.get_or_init(|| build_spec(family)).clone()
}

fn build_spec(family: Family) -> ModelSpec {
    let hinge = crate::flame::spec(family);
    let heat = crate::flame3::spec(family);
    let mut tensors: Vec<TensorSpec> = hinge
        .tensors
        .iter()
        .filter(|t| !is_old_limit_clock(&t.name))
        .cloned()
        .collect();
    tensors.extend(
        heat.tensors
            .iter()
            .filter(|t| is_burner_tensor(&t.name))
            .cloned(),
    );
    let spec_id = if family == Family::AcmeInverse {
        "acme_inverse-wsp-v8"
    } else {
        "acme_normal-wsp-v8"
    };
    ModelSpec {
        schema_version: hinge.schema_version,
        spec_id: SpecId::new(spec_id).expect("id"),
        family,
        model_name: model_name(family).into(),
        target: hinge.target.clone(),
        features: heat.features.clone(),
        domain: heat.domain.clone(),
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::flame4::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(format!(
            "The hinge-law clocks of spec {} ({}) with the burner-heating law of spec {} ({}) as the \
             burner-limit clock. The tensors are those of the first spec without ft.tau.* and \
             ft.shape, and, from the second, bt.*, se.rho_d_g_m_s, norm.r_ref_mm, \
             norm.mtot_ref_kg_s, phys.mw_* and num.bt_nodes, each with the name, shape, role, bounds \
             and fixed value it has there. The features and the domain listed are those of the \
             second spec; the hinge laws read the first eight features. \
             No time series enters: conditions, cutoff and control stages known at the cutoff only. \
             Two latent classes (weak, main) are integrated from the time origin; their weights at \
             the cutoff are their prior probabilities times their survival to the cutoff, \
             normalised. Self-extinction hazard: weak class, log-logistic(se.flash_tau_s, \
             se.flash_shape) from the time origin; main class, log-logistic(exp(N_tau(c)), se.shape) \
             from se.floor_s; p_weak = sigmoid(N_flash(c)). Flow-termination hazard: exp(ft.log_base), \
             plus for each programmed stop a a hazard -ln(1 - p_a) / (2 num.atom_half_s) on [T_a - \
             num.atom_half_s, T_a + num.atom_half_s], p_a = sigmoid(ft.atom_logit[a] + ft.atom_w[a, :] \
             . c), atoms with T_a = 0 or p_a <= 1e-9 off, plus the burner limit. \
             Burner limit: the reactant mass fraction Y_R of the injected stream from its mole \
             fraction and the phys.mw_* constants (fuel in N2 for normal flames, O2 in N2 for \
             inverse); m_tot = injected_mass_flow / Y_R; normal: L = ln(1 / (1 - zst)); inverse: L = \
             ln(1 / zst); r_ss = m_tot / (4 pi rhoD L) with rhoD = se.rho_d_g_m_s; b = (b_rss, \
             b_mtot, c_tad, c_xf, c_logp, f_ethane, f_butane); ln dT_j = N_rise(b) + bt.sigma z_j \
             (z_j the standard-normal quantile at (j + 1/2) / num.bt_nodes); for each limit l with \
             bt.limit_w[l] > 0: t_jl = -bt.tau_s ln(1 - (bt.limit_c[l] - bt.t0_c) / dT_j) when dT_j > \
             bt.limit_c[l] - bt.t0_c, else never; S_bt(t) = sum_jl (bt.limit_w[l] / sum bt.limit_w) / \
             num.bt_nodes x [1 / (1 + (t / t_jl)^bt.shape), or 1 when never]; all weights zero: S_bt \
             = 1; a weighted limit at or below bt.t0_c is a refusal. Over a step from t to t + h \
             (h <= num.dt_s) the burner limit is the hazard ln(S_bt(t) / S_bt(t + h)) / h; the other \
             hazards are taken at the midpoint of the step and constant over it; within a step the \
             causes share the lost mass in proportion to their hazards, so the four rows sum to one \
             exactly. A stop known at the cutoff (control `scheduled_stop`, `flow_stop`, or an \
             operator action starting `MFCs off`) at or before the cutoff refuses the forecast; a \
             later one moves the mass still active at that time to operator_termination. Beyond \
             num.horizon_s the mass still active is unresolved_support. A stoichiometric mixture \
             fraction outside (0, 1), a burner-side reactant mole fraction outside (0, 1], a fuel \
             indicator other than 0 or 1 or two of them set, a burner quantity that is not finite \
             at the conditions, and a burner-limit survival of zero at the cutoff are refusals. \
             Uncertainty kind: assumed_noise.",
            hinge.spec_id,
            hinge.spec_hash(),
            heat.spec_id,
            heat.spec_hash()
        )),
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
    fn eval(&self, x: &[f64], scratch: &mut [f64]) -> f64 {
        hinge_net(self.w1, self.b1, self.w2, self.skip, self.b0, x, scratch)
    }
}

pub(crate) struct Weights<'a> {
    inverse: bool,
    se_tau: Net<'a>,
    se_flash: Net<'a>,
    bt_rise: Net<'a>,
    floor: f64,
    se_shape: f64,
    flash_tau: f64,
    flash_shape: f64,
    rho_d: f64,
    mw_fuel: [f64; 3],
    mw_n2: f64,
    mw_o2: f64,
    bt_nodes: usize,
    bt_tau: f64,
    bt_t0: f64,
    bt_sigma: f64,
    bt_shape: f64,
    bt_limit: &'a [f64],
    bt_limit_w: &'a [f64],
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
    r_ref_mm: f64,
    mtot_ref: f64,
    atom_half: f64,
    dt: f64,
    pub(crate) horizon: f64,
}

/// Burner-side quantities of one test, in SI.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Burner {
    /// Mass flow of the burner-side reactant, kg/s.
    pub m_r_kg_s: f64,
    /// Total mass flow of the burner stream, kg/s.
    pub m_tot_kg_s: f64,
    /// `ln(1 / (1 - Z_st))` for a normal flame, `ln(1 / Z_st)` for an inverse one.
    pub l: f64,
}

/// The condition-level clocks of one flame (reported in the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct Clocks {
    pub p_weak: f64,
    pub weak_tau_s: f64,
    pub floor_s: f64,
    pub radiative_tau_s: f64,
    /// Quasi-steady radius, mm.
    pub r_ss_mm: f64,
    /// The burner features `b` in [`BURNER_FEATURES`] order.
    pub burner_features: [f64; N_B],
    /// Equilibrium temperature rise of the burner at the centre heating node, K.
    pub burner_rise_k: f64,
    pub burner_heat_tau_s: f64,
    pub burner_t0_c: f64,
    /// `(limit degC, normalised weight, time to that limit at the centre heating node or null)` of
    /// each weighted limit.
    pub burner_limits: Vec<(f64, f64, Option<f64>)>,
    /// Total weight of the (heating node, limit) pairs that never reach their limit (1 when the
    /// burner-heating clock is off).
    pub p_limit_never: f64,
    /// `(time, probability)` of each programmed stop that is switched on.
    pub programmed: Vec<(f64, f64)>,
    pub background_per_s: f64,
    /// `(weight, median time)` of each (heating node, limit) pair that reaches its limit.
    #[serde(skip)]
    pairs: Vec<(f64, f64)>,
}

impl Clocks {
    /// The burner quantities that are not finite (a rise network that overflowed or underflowed at
    /// these conditions): reasons for a refusal, never replaced by a default.
    fn burner_not_finite(&self) -> Vec<String> {
        let mut out: Vec<String> = [
            ("quasi-steady radius r_ss (mm)", self.r_ss_mm),
            ("burner rise dT_eq (K)", self.burner_rise_k),
        ]
        .into_iter()
        .filter(|(_, v)| !(v.is_finite() && *v > 0.0))
        .map(|(name, v)| format!("{name} = {v} is not finite and positive at these conditions"))
        .collect();
        for (name, v) in BURNER_FEATURES.iter().zip(self.burner_features) {
            if !v.is_finite() {
                out.push(format!("burner feature {name} = {v} is not finite"));
            }
        }
        if self
            .pairs
            .iter()
            .any(|(w, t)| !(w.is_finite() && t.is_finite() && *t > 0.0))
        {
            out.push("a time to a burner limit is not finite and positive".into());
        }
        out
    }
}

#[inline]
fn log_logistic(t: f64, tau: f64, k: f64) -> f64 {
    if !(t > 0.0) || !(tau > 0.0) {
        return 0.0;
    }
    let r = (t / tau).powf(k);
    (k / t) * r / (1.0 + r)
}

/// Log-logistic survival `1 / (1 + (t / tau)^k)`; 1 at and before the origin.
#[inline]
fn ll_survival(t: f64, tau: f64, k: f64) -> f64 {
    if !(t > 0.0) {
        return 1.0;
    }
    1.0 / (1.0 + (t / tau).powf(k))
}

/// Standard-normal quantile of node `i` of `n` equally likely nodes (0 for a single node).
#[inline]
fn node_z(i: usize, n: usize) -> f64 {
    if n == 1 {
        0.0
    } else {
        normal_quantile((i as f64 + 0.5) / n as f64)
    }
}

/// Time for a burner heated as `t0 + dT (1 - exp(-t / tau))` to rise by `need` K, given `ln dT`;
/// `None` when `dT <= need`.
#[inline]
fn limit_time(need: f64, ln_dt: f64, tau: f64) -> Option<f64> {
    let frac = need * (-ln_dt).exp();
    (frac < 1.0).then(|| -tau * (-frac).ln_1p())
}

/// `-ln (b / a)` of the burner-limit survival over a step, from its values at both ends (0 when
/// there is no survival left to lose).
#[inline]
fn neg_ln_ratio(a: f64, b: f64) -> f64 {
    if !(a > 0.0) {
        return 0.0;
    }
    let r = (b / a).min(1.0);
    if r > 0.0 {
        -r.ln()
    } else {
        f64::INFINITY
    }
}

impl<'a> Weights<'a> {
    pub(crate) fn bind(p: &Params<'a>, family: Family) -> Self {
        let net = |prefix: &str| Net {
            w1: p.v(&format!("{prefix}.W1")),
            b1: p.v(&format!("{prefix}.b1")),
            w2: p.v(&format!("{prefix}.w2")),
            skip: p.v(&format!("{prefix}.skip")),
            b0: p.s(&format!("{prefix}.b0")),
        };
        Weights {
            inverse: family == Family::AcmeInverse,
            se_tau: net("se.tau"),
            se_flash: net("se.flash"),
            bt_rise: net("bt.rise"),
            floor: p.s("se.floor_s"),
            se_shape: p.s("se.shape"),
            flash_tau: p.s("se.flash_tau_s"),
            flash_shape: p.s("se.flash_shape"),
            rho_d: p.s("se.rho_d_g_m_s"),
            mw_fuel: [
                p.s("phys.mw_ethane"),
                p.s("phys.mw_propane"),
                p.s("phys.mw_butane"),
            ],
            mw_n2: p.s("phys.mw_n2"),
            mw_o2: p.s("phys.mw_o2"),
            bt_nodes: (p.s("num.bt_nodes") as usize).clamp(1, MAX_NODES),
            bt_tau: p.s("bt.tau_s"),
            bt_t0: p.s("bt.t0_c"),
            bt_sigma: p.s("bt.sigma"),
            bt_shape: p.s("bt.shape"),
            bt_limit: p.v("bt.limit_c"),
            bt_limit_w: p.v("bt.limit_w"),
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
            r_ref_mm: p.s("norm.r_ref_mm"),
            mtot_ref: p.s("norm.mtot_ref_kg_s"),
            atom_half: p.s("num.atom_half_s"),
            dt: p.s("num.dt_s"),
            horizon: p.s("num.horizon_s"),
        }
    }

    /// Weighted limits at or below the burner temperature at ignition (a release that cannot be run).
    fn limits_below_t0(&self) -> Vec<String> {
        (0..N_LIMITS)
            .filter(|&l| self.bt_limit_w[l] > 0.0 && self.bt_limit[l] <= self.bt_t0)
            .map(|l| {
                format!(
                    "bt.limit_c[{l}] = {} degC is at or below bt.t0_c = {} degC",
                    self.bt_limit[l], self.bt_t0
                )
            })
            .collect()
    }

    pub(crate) fn clocks(&self, c: &[f64; N_IN], b: &Burner) -> Clocks {
        let mut scratch = [0.0; HIDDEN];
        let mut scratch_b = [0.0; HIDDEN_B];
        let four_pi = 4.0 * std::f64::consts::PI;
        // SI: kg m-1 s-1
        let rho_d = self.rho_d * 1.0e-3;
        let r_ss = b.m_tot_kg_s / (four_pi * rho_d * b.l);
        // burner side
        let bf = [
            (r_ss * 1.0e3 / self.r_ref_mm).ln(),
            (b.m_tot_kg_s / self.mtot_ref).ln(),
            c[4],
            c[3],
            c[0],
            c[6],
            c[7],
        ];
        let ln_rise = self.bt_rise.eval(&bf, &mut scratch_b);
        let w_sum: f64 = self.bt_limit_w.iter().sum();
        let nb = self.bt_nodes;
        let mut pairs = Vec::new();
        let mut never = if w_sum > 0.0 { 0.0 } else { 1.0 };
        let mut burner_limits = Vec::new();
        if w_sum > 0.0 {
            for l in 0..N_LIMITS {
                let w = self.bt_limit_w[l] / w_sum;
                if !(w > 0.0) {
                    continue;
                }
                let need = self.bt_limit[l] - self.bt_t0;
                burner_limits.push((self.bt_limit[l], w, limit_time(need, ln_rise, self.bt_tau)));
                for j in 0..nb {
                    let ln_dt = ln_rise + self.bt_sigma * node_z(j, nb);
                    match limit_time(need, ln_dt, self.bt_tau) {
                        Some(t) => pairs.push((w / nb as f64, t)),
                        None => never += w / nb as f64,
                    }
                }
            }
        }
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
            r_ss_mm: r_ss * 1.0e3,
            burner_features: bf,
            burner_rise_k: ln_rise.exp(),
            burner_heat_tau_s: self.bt_tau,
            burner_t0_c: self.bt_t0,
            burner_limits,
            p_limit_never: never,
            programmed,
            background_per_s: self.base,
            pairs,
        }
    }

    /// Hazards (1/s) at time `t` of self-extinction for one class, and of flow termination without
    /// the burner limit (programmed stops and background).
    #[inline]
    fn hazards(&self, k: &Clocks, weak: bool, t: f64) -> (f64, f64) {
        let se = if weak {
            log_logistic(t, k.weak_tau_s, self.flash_shape)
        } else {
            log_logistic(t - k.floor_s, k.radiative_tau_s, self.se_shape)
        };
        let mut ft = k.background_per_s;
        for &(ta, p) in &k.programmed {
            if (t - ta).abs() <= self.atom_half {
                ft += -(1.0 - p).ln() / (2.0 * self.atom_half);
            }
        }
        (se, ft)
    }

    /// Survival of the burner limit from the time origin.
    #[inline]
    fn s_bt(&self, k: &Clocks, t: f64) -> f64 {
        let bt = k.p_limit_never
            + k.pairs
                .iter()
                .map(|&(w, tau)| w * ll_survival(t, tau, self.bt_shape))
                .sum::<f64>();
        bt.min(1.0)
    }

    /// The eight condition features of a prefix and its burner-side quantities, or the reasons they
    /// cannot be formed.
    pub(crate) fn features(
        &self,
        prefix: &PrefixBundle,
    ) -> Result<([f64; N_IN], Burner), Vec<String>> {
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
        // the indicators choose the fuel's molar mass: anything but one fuel or propane is refused
        for (name, v) in [("fuel_is_ethane", fe), ("fuel_is_butane", fb)] {
            if v.is_finite() && v != 0.0 && v != 1.0 {
                reasons.push(format!("{name} = {v} is not 0 or 1"));
            }
        }
        if fe == 1.0 && fb == 1.0 {
            reasons.push("fuel_is_ethane and fuel_is_butane are both 1".to_string());
        }
        if zst.is_finite() && !(zst > 0.0 && zst < 1.0) {
            reasons.push(format!("zst = {zst} is not inside (0, 1)"));
        }
        // mole fraction of the burner-side reactant in the injected stream
        let x_r = if self.inverse { o2 } else { xf };
        if x_r.is_finite() && !(x_r > 0.0 && x_r <= 1.0) {
            reasons.push(format!(
                "burner-side reactant mole fraction {x_r} is not inside (0, 1]"
            ));
        }
        if !reasons.is_empty() {
            reasons.dedup();
            return Err(reasons);
        }
        let mw_r = if self.inverse {
            self.mw_o2
        } else {
            fe * self.mw_fuel[0] + (1.0 - fe - fb) * self.mw_fuel[1] + fb * self.mw_fuel[2]
        };
        let y_r = x_r * mw_r / (x_r * mw_r + (1.0 - x_r) * self.mw_n2);
        // injected_mass_flow is the reactant alone in both configurations (mHC; O2 flow rate)
        let burner = Burner {
            m_r_kg_s: m,
            m_tot_kg_s: m / y_r,
            l: if self.inverse {
                (1.0 / zst).ln()
            } else {
                (1.0 / (1.0 - zst)).ln()
            },
        };
        Ok((
            [
                (p / self.p_ref).ln(),
                (m / self.mdot_ref).ln(),
                (o2 - self.o2_ref) / self.o2_scale,
                (xf / self.xf_ref).ln(),
                (tad - self.tad_ref) / self.tad_scale,
                (zst - self.zst_ref) / self.zst_scale,
                fe,
                fb,
            ],
            burner,
        ))
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
    pub burner: Burner,
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
    let mut bt_prev = wt.s_bt(k, t0);
    let mut out = Vec::with_capacity(times.len());
    for &tj in times {
        let target = tj.min(end);
        if target > t {
            let n_sub = (((target - t) / wt.dt) - 1e-9).ceil().max(1.0) as usize;
            let h = (target - t) / n_sub as f64;
            for i in 0..n_sub {
                let (le, stops) = wt.hazards(k, weak, t + 0.5 * h);
                // the burner limit over this step: the hazard whose integral is the loss of its
                // survival between the ends of the step
                let tb = if i + 1 == n_sub { target } else { t + h };
                let bt_next = wt.s_bt(k, tb);
                let lo = stops + neg_ln_ratio(bt_prev, bt_next) / h;
                let total = le + lo;
                if total > 0.0 {
                    let lost = s * -(-total * h).exp_m1();
                    if total.is_finite() {
                        f_e += lost * le / total;
                        f_o += lost * lo / total;
                    } else {
                        // no burner-limit survival is left at the end of the step
                        f_o += lost;
                    }
                    s -= lost;
                }
                bt_prev = bt_next;
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
    burner: &Burner,
    clocks: Clocks,
    tau: f64,
    stop: Option<f64>,
    times: &[f64],
) -> Forecast {
    let nt = times.len();
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
        burner: *burner,
        clocks,
        cutoff_s: tau,
        p_weak_at_cutoff: p_weak,
        scheduled_stop_s: stop,
        times_s: times.to_vec(),
        table,
        steps,
    }
}

/// Strict forward of a release on an admitted prefix: `Ok(Err(support))` is a scientific refusal,
/// `Err(refusal)` a release or request that cannot be run at all.
pub fn forecast(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<Result<Forecast, Support>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params, spec.family);
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
    let below = wt.limits_below_t0();
    if !below.is_empty() {
        return Ok(Err(Support {
            admitted: false,
            gate: SupportGate::Semantic,
            reasons: below,
        }));
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
    let (c, burner) = match wt.features(prefix) {
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
    let clocks = wt.clocks(&c, &burner);
    let bad = clocks.burner_not_finite();
    if !bad.is_empty() {
        return Ok(Err(Support {
            admitted: false,
            gate: SupportGate::Condition,
            reasons: bad,
        }));
    }
    let bt = wt.s_bt(&clocks, tau);
    if !(bt > 0.0) {
        return Ok(Err(refuse(
            SupportGate::Condition,
            format!("the release gives no burner-limit survival to the cutoff ({bt}): a flame burning at the cutoff is outside it"),
        )));
    }
    Ok(Ok(integrate(&wt, &c, &burner, clocks, tau, stop, &times)))
}

fn body(f: Forecast) -> Body {
    let diagnostics = json!({
        "interpretation": INTERPRETATION,
        "features": FEATURES.iter().zip(f.features).map(|(k, v)| (k.to_string(), json!(v))).collect::<serde_json::Map<_, _>>(),
        "burner": f.burner,
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

/// The condition-level clocks of a release for one feature vector and its burner-side quantities
/// (inspection; no data).
pub fn clocks_of(
    spec: &ModelSpec,
    release: &LoadedRelease,
    c: [f64; N_IN],
    burner: Burner,
) -> Result<Clocks, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    Ok(Weights::bind(&params, spec.family).clocks(&c, &burner))
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

//! `acme_normal.wsp3` and `acme_inverse.wsp3` — the growth-clock forward of the ACME burner flames.
//!
//! The second generation (`flame.rs`) wrote the radiative self-extinction time as a hinge law in the
//! conditions. This forward writes it as the published physics of a burner-supported spherical
//! diffusion flame, with the constants of that physics as the numbers an author types; next to it
//! an empirical oscillation and quench clock, and a physical burner-heating clock for the
//! temperature limit of the burner:
//!
//! ```text
//! c = (c_logp, c_logm, c_o2, c_xf, c_tad, c_zst, f_ethane, f_butane)
//!
//! burner side      normal: fuel stream,  L = ln(1 / (1 - Z_st));  inverse: oxidiser stream,  L = ln(1 / Z_st)
//!   m_R            mass flow of the burner-side reactant = injected_mass_flow (PSI-159 `mHC` of a
//!                  normal flame, `O2 flow rate` of an inverse one: both are the reactant alone)
//!   m_tot          total mass flow of the burner stream = m_R / Y_R (reactant in N2)
//! quasi-steady     r_ss  = m_tot / (4 pi rhoD L)            the radius the flame grows towards
//! extinction       r_ext = sqrt(m_R / (4 pi m_cr))          the radius at which the reactant flux at the
//!                                                           flame has fallen to the critical flux m_cr
//! reduced flow     Phi   = (r_ss / r_ext)^2                 Phi <= 1: the flame settles inside r_ext and
//!                                                           is never taken by radiation
//! growth           r_f(t) = r_ss erfc(r_f / (2 sqrt(D t)))  far-field diffusion from a point source
//!   =>             R(Phi) = r_ext^2 / (4 D erfcinv(Phi^-1/2)^2)   time for the flame to reach r_ext
//!
//!   ln m_cr = N_flux(c)   ln D = N_diff(c)   rhoD: one typed number
//!
//! 1. self-extinction
//!   weak class   probability p_weak = sigmoid(N_flash(c)): the flame goes out within seconds of
//!                ignition, S_w(t) = 1 / (1 + (t / flash_tau)^flash_shape)
//!   main class   ln Phi ~ Normal(ln Phi_hat, sigma_phi), integrated on num.phi_nodes equally likely
//!                nodes z_i (standard-normal quantiles at (i + 1/2) / n). With t' = max(t - floor, 0):
//!                S_i(t) = G_i(t') O(t') exp(-hold t')
//!                  G_i   growth clock 1 / (1 + (t' / R(Phi_i))^shape), or 1 when node i has no growth
//!                        clock (Phi_i <= 1, or R(Phi_i) >= num.tau_cap_s: "held")
//!                  O     oscillation and quench 1 / (1 + (t' / exp(N_osc(c)))^osc_shape) (supply-tube
//!                        quench hole, a flame too small to live on the burner: an empirical clock in
//!                        the conditions; a median beyond the horizon switches it off)
//!                  hold  exp(hold_log_rate), a constant hazard of an established flame
//!   S_se(t) = p_weak S_w(t) + sum_i ((1 - p_weak) / n) S_i(t)
//!
//! 2. burner limit (flow termination)
//!   b = (b_rss, b_mtot, c_tad, c_xf, c_logp, f_ethane, f_butane),
//!       b_rss = ln(r_ss / norm.r_ref_mm), b_mtot = ln(m_tot / norm.mtot_ref_kg_s)
//!   T_b(t) = t0 + dT (1 - exp(-t / tau_b))          burner heated towards t0 + dT
//!   ln dT_j = N_rise(b) + bt.sigma z_j              num.bt_nodes equally likely heating nodes
//!   t_jl = -tau_b ln(1 - (limit_l - t0) / dT_j)     when dT_j > limit_l - t0, else never
//!   S_bt(t) = sum_jl (w_l / sum w) / n_bt x (1 / (1 + (t / t_jl)^bt.shape), or 1 when never)
//!
//! 3. programmed stops and background (flow termination)
//!   S_ps(t) = exp(-base t) prod_a (1 - p_a)^clamp((t - (T_a - half)) / (2 half), 0, 1),
//!   p_a = sigmoid(logit_a + W_a . c)  (script durations and time-outs of the campaign)
//!
//! N(x) = b0 + skip . x + sum_j w2[j] relu(b1[j] + W1[j, :] . x)
//! ```
//!
//! The three families are independent, so the survival from the time origin is the product
//! S_se S_bt S_ps, each in closed form. Conditioning on a flame burning at the cutoff divides each by
//! its value at the cutoff: this is the same latent-class mixture as re-weighting the weak class and
//! the nodes by their survival to the cutoff (the flow-termination clocks do not depend on the
//! self-extinction class), now with a mixture on the burner side too. On a grid from the cutoff,
//! each step loses S (1 - rho_se rho_bt rho_ps) with rho = S(b) / S(a) per family, shared between
//! extinction and operator termination in proportion to -ln rho_se and -(ln rho_bt + ln rho_ps), so
//! the four rows `active, extinction, operator_termination, unresolved_support` sum to one exactly.
//! An operator stop is never counted as an extinction, and `extinction` here is what the PSI-159
//! tables call self-extinction: the end of the hot flame. A cool flame that follows it is outside
//! this model.
//!
//! Why the spread is on Phi: near Phi = 1 the life diverges, so a small error in the reduced flow
//! decides between a flame that goes out in two minutes and one that is held. A fixed clock width
//! cannot say that; a distribution over Phi says it where the physics puts it.
//!
//! Normal and inverse share this code and nothing else: separate specs, reference scales, patches
//! and releases. The prefix carries no time series for ACME. Nothing is fitted on this route.
//! Sources of the law: the quasi-steady radius and rhoD of Kim et al., Proc. Combust. Inst. 39
//! (2023); the critical reactant flux of Irace et al., ibid.; the far-field growth is the
//! continuous point source in a still medium. They are cited in `docs/wsp/research/`.

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
use crate::quad::normal_quantile;
use crate::spec::{authored, constant, sparse, Params};

pub const FORWARD_VERSION: &str = "3";
/// Most nodes of the reduced-flow or burner-heating distribution a spec may ask for.
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

fn fin(v: f64) -> Finite {
    Finite::new(v).expect("finite literal")
}

pub fn model_name(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse.wsp3",
        _ => "acme_normal.wsp3",
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
            spec_id: "acme_inverse-wsp-v3",
            injected: "the injected O2 mass flow (PSI-159 'O2 flow rate', setpoint; the O2 alone, not the O2 + N2 stream)",
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
            spec_id: "acme_normal-wsp-v3",
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

/// The five tensors of one hinge network: `W1 [h, n]`, `b1 [h]`, `w2 [h]` sparse, `skip [n]` and
/// `b0` authored.
fn net_tensors(
    p: &str,
    what: &str,
    reads: &str,
    h: u32,
    n: u32,
    role: TensorRole,
    unit: &str,
    range: (f64, f64),
) -> [ignis_core::contracts::TensorSpec; 5] {
    use TensorRole::*;
    [
        sparse(
            &format!("{p}.W1"),
            &[h, n],
            Weight,
            None,
            &format!("Hidden weights of {what}; row j reads {reads} in feature order."),
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
            &format!("Linear part of {what} in {reads}; every element is stated."),
        ),
        authored(
            &format!("{p}.b0"),
            &[],
            role,
            Some(unit),
            Some(range),
            &format!("Value of {what} at {reads} = 0 before hinge terms."),
        ),
    ]
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
    let (hb, nb, nl) = (HIDDEN_B as u32, N_B as u32, N_LIMITS as u32);
    let mut tensors = Vec::new();
    for (p, what, role, unit, range) in [
        (
            "se.flux",
            "the critical burner-side reactant mass flux at radiative extinction, ln(m_cr / 1 g m-2 s-1)",
            PhysicalParameter,
            "ln(g m-2 s-1)",
            (-6.0, 6.0),
        ),
        (
            "se.diff",
            "the effective diffusivity of the flame-growth clock, ln(D / 1 mm2 s-1)",
            PhysicalParameter,
            "ln(mm2 s-1)",
            (-3.0, 10.0),
        ),
        (
            "se.flash",
            "the weak-flame class logit",
            HazardParameter,
            "logit(1)",
            (-40.0, 40.0),
        ),
        (
            "se.osc",
            "the median of the oscillation and quench clock after the floor, ln(tau_osc / 1 s)",
            PhysicalParameter,
            "ln(s)",
            (-5.0, 14.0),
        ),
    ] {
        tensors.extend(net_tensors(p, what, "c", h, n, role, unit, range));
    }
    tensors.extend(net_tensors(
        "bt.rise",
        "the equilibrium temperature rise of the burner, ln(dT_eq / 1 K)",
        "b",
        hb,
        nb,
        PhysicalParameter,
        "ln(K)",
        (0.0, 10.0),
    ));
    tensors.extend([
        authored("se.rho_d_g_m_s", &[], PhysicalParameter, Some("g m-1 s-1"), Some((0.001, 1.0)),
                 "rhoD of the quasi-steady flame radius r_ss = m_tot / (4 pi rhoD L); also enters b_rss."),
        authored("se.sigma_phi", &[], NoiseParameter, Some("1"), Some((0.0, 3.0)),
                 "sd of ln Phi: how well the reduced flow of a test is known."),
        authored("se.hold_log_rate", &[], HazardParameter, Some("ln(1/s)"), Some((-40.0, 5.0)),
                 "ln of the constant self-extinction hazard of an established flame after the floor; -40 switches it off."),
        authored("se.floor_s", &[], PhysicalParameter, Some("s"), Some((0.0, 60.0)),
                 "Time after ignition before which the main class cannot self-extinguish (flame establishment)."),
        authored("se.shape", &[], HazardParameter, Some("1"), Some((0.5, 30.0)),
                 "Log-logistic shape of the growth clock at a given Phi (sd of ln R = pi / (shape sqrt 3))."),
        authored("se.osc_shape", &[], HazardParameter, Some("1"), Some((0.5, 30.0)),
                 "Log-logistic shape of the oscillation and quench clock."),
        authored("se.flash_tau_s", &[], PhysicalParameter, Some("s"), Some((0.05, 30.0)),
                 "Median life of a weak flame."),
        authored("se.flash_shape", &[], HazardParameter, Some("1"), Some((0.5, 30.0)),
                 "Log-logistic shape of the weak-flame clock."),
        authored("bt.tau_s", &[], PhysicalParameter, Some("s"), Some((1.0, 1000.0)),
                 "Heating time constant of the burner: T_b(t) = t0 + dT (1 - exp(-t / tau))."),
        authored("bt.t0_c", &[], PhysicalParameter, Some("degC"), Some((-50.0, 500.0)),
                 "Burner temperature at ignition."),
        authored("bt.sigma", &[], NoiseParameter, Some("1"), Some((0.0, 2.0)),
                 "sd of ln dT_eq over num.bt_nodes equally likely heating nodes."),
        authored("bt.shape", &[], HazardParameter, Some("1"), Some((0.5, 60.0)),
                 "Log-logistic shape of the limit time around each (heating node, limit) pair's time."),
        authored("bt.limit_c", &[nl], PhysicalParameter, Some("degC"), Some((0.0, 1500.0)),
                 "Burner temperature limit settings of the campaign."),
        authored("bt.limit_w", &[nl], HazardParameter, Some("1"), Some((0.0, 1.0)),
                 "Weights of the limit settings, normalised by their sum; all zero switches the burner-heating clock off."),
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
        constant("norm.r_ref_mm", 25.0, "mm", "Reference quasi-steady radius of b_rss."),
        constant("norm.mtot_ref_kg_s", 3.0e-6, "kg/s", "Reference burner-stream mass flow of b_mtot."),
        constant("phys.mw_ethane", 30.07, "g/mol", "Molar mass of ethane."),
        constant("phys.mw_propane", 44.10, "g/mol", "Molar mass of propane."),
        constant("phys.mw_butane", 58.12, "g/mol", "Molar mass of n-butane."),
        constant("phys.mw_n2", 28.013, "g/mol", "Molar mass of nitrogen, the diluent of both streams."),
        constant("phys.mw_o2", 31.998, "g/mol", "Molar mass of oxygen."),
        constant("num.phi_nodes", 15.0, "1", "Equally likely values of ln Phi the main class is integrated over (standard-normal quantiles at (i + 1/2) / n)."),
        constant("num.bt_nodes", 7.0, "1", "Equally likely values of ln dT_eq the burner-heating clock is integrated over (standard-normal quantiles at (j + 1/2) / n)."),
        constant("num.tau_cap_s", 1.0e5, "s", "Largest growth-clock median; a longer one is a held flame for every purpose of the horizon."),
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
            feature("b_rss", "derived:r_ss", "ln(r_ss / norm.r_ref_mm), r_ss = m_tot / (4 pi rhoD L) in mm with rhoD = se.rho_d_g_m_s; burner feature 0 @1"),
            feature("b_mtot", "derived:m_tot", "ln(m_tot / norm.mtot_ref_kg_s), m_tot = injected_mass_flow / Y_R; burner feature 1 @1"),
        ],
        domain: sc
            .domain
            .iter()
            .map(|(f, lo, hi)| DomainBound { feature: (*f).into(), min: Some(fin(*lo)), max: Some(fin(*hi)) })
            .collect(),
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::flame3::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(
            "No time series enters: conditions, cutoff and control stages known at the cutoff only. \
             Burner-side quantities: the reactant mass fraction Y_R of the injected stream from its \
             mole fraction and the phys.mw_* constants (fuel in N2 for normal flames, O2 in N2 for \
             inverse); injected_mass_flow is the reactant alone in both configurations (PSI-159 \
             mHC, and O2 flow rate), so m_R = injected_mass_flow and m_tot = m_R / Y_R; normal: \
             L = ln(1 / (1 - zst)); inverse: L = ln(1 / zst). r_ss = m_tot / (4 pi rhoD L), \
             r_ext = sqrt(m_R / (4 pi m_cr)), Phi = (r_ss / r_ext)^2, m_cr = exp(N_flux(c)) g m-2 s-1, \
             D = exp(N_diff(c)) mm2/s, t_g = r_ext^2 / (4 D); growth clock of node i (z_i the \
             standard-normal quantile at (i + 1/2) / num.phi_nodes, ln Phi_i = ln Phi + sigma_phi z_i): \
             R_i = t_g / erfcinv(Phi_i^-1/2)^2 when Phi_i > 1 and R_i < num.tau_cap_s, else none; \
             erfcinv(y) = -normal_quantile(y / 2) / sqrt(2). Self-extinction survival from the time \
             origin, t' = max(t - se.floor_s, 0): S_w(t) = 1 / (1 + (t / se.flash_tau_s)^se.flash_shape); \
             S_i(t) = [1 / (1 + (t' / R_i)^se.shape), or 1 without a growth clock] x \
             1 / (1 + (t' / exp(N_osc(c)))^se.osc_shape) x exp(-exp(se.hold_log_rate) t'); \
             S_se = p_weak S_w + sum_i ((1 - p_weak) / num.phi_nodes) S_i, p_weak = sigmoid(N_flash(c)). \
             Burner limit: b = (b_rss, b_mtot, c_tad, c_xf, c_logp, f_ethane, f_butane); ln dT_j = \
             N_rise(b) + bt.sigma z_j (z_j at (j + 1/2) / num.bt_nodes); for each limit l with \
             bt.limit_w[l] > 0: t_jl = -bt.tau_s ln(1 - (bt.limit_c[l] - bt.t0_c) / dT_j) when \
             dT_j > bt.limit_c[l] - bt.t0_c, else never; S_bt = sum_jl (bt.limit_w[l] / sum \
             bt.limit_w) / num.bt_nodes x [1 / (1 + (t / t_jl)^bt.shape), or 1 when never]; all \
             weights zero: S_bt = 1; a weighted limit at or below bt.t0_c is a refusal. Stops: \
             S_ps = exp(-exp(ft.log_base) t) prod_a (1 - p_a)^clamp((t - (T_a - num.atom_half_s)) / \
             (2 num.atom_half_s), 0, 1), p_a = sigmoid(ft.atom_logit[a] + ft.atom_w[a, :] . c), atoms \
             with T_a = 0 or p_a <= 1e-9 off. From the cutoff, on a grid with step <= num.dt_s up to \
             each requested time: rho_f = S_f(b) / S_f(a) for f = se, bt, ps; lost = S (1 - rho_se \
             rho_bt rho_ps), S = 1 at the cutoff; the lost mass goes to extinction and operator \
             termination in proportion to -ln rho_se and -(ln rho_bt + ln rho_ps), so the four rows sum \
             to one exactly. p_weak_at_cutoff = p_weak S_w(cutoff) / S_se(cutoff); p_held_at_cutoff \
             = the same ratio summed over the nodes with no growth clock. A survival of zero at the \
             cutoff is a refusal. A fuel indicator other than 0 or 1, or two of them set, is a \
             refusal. A clock that is not finite and positive at the conditions (an overflow of a law \
             network) is a refusal. A stop known at the cutoff (control `scheduled_stop`, `flow_stop`, \
             or an operator action starting `MFCs off`) at or before the cutoff refuses the forecast; \
             a later one moves the mass still active at that time to operator_termination. Beyond \
             num.horizon_s the mass still active is unresolved_support. Uncertainty kind: assumed_noise."
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
    fn eval(&self, x: &[f64], scratch: &mut [f64]) -> f64 {
        hinge_net(self.w1, self.b1, self.w2, self.skip, self.b0, x, scratch)
    }
}

pub(crate) struct Weights<'a> {
    inverse: bool,
    se_flux: Net<'a>,
    se_diff: Net<'a>,
    se_flash: Net<'a>,
    se_osc: Net<'a>,
    bt_rise: Net<'a>,
    rho_d: f64,
    sigma_phi: f64,
    hold: f64,
    mw_fuel: [f64; 3],
    mw_n2: f64,
    mw_o2: f64,
    nodes: usize,
    bt_nodes: usize,
    tau_cap: f64,
    floor: f64,
    se_shape: f64,
    osc_shape: f64,
    flash_tau: f64,
    flash_shape: f64,
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
    /// Critical reactant flux, g m-2 s-1, and growth diffusivity, mm2/s.
    pub m_cr_g_m2_s: f64,
    pub d_mm2_s: f64,
    /// Quasi-steady and extinction radii, mm.
    pub r_ss_mm: f64,
    pub r_ext_mm: f64,
    /// Reduced flow at the centre of its distribution.
    pub phi: f64,
    /// Median growth clock at that reduced flow (`None`: no growth clock, the flame is held).
    pub radiative_tau_s: Option<f64>,
    /// Median growth clock of each node of the reduced-flow distribution (`None`: held).
    pub node_tau_s: Vec<Option<f64>>,
    /// Prior probability of the main-class nodes with no growth clock (Phi <= 1, or a growth
    /// clock beyond `num.tau_cap_s`).
    pub p_held: f64,
    /// Median of the oscillation and quench clock after the floor, s.
    pub osc_tau_s: f64,
    pub hold_per_s: f64,
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
    /// The clocks that are not finite and positive (a law network that overflowed or underflowed at
    /// these conditions): reasons for a refusal, never replaced by a default.
    fn not_finite(&self) -> Vec<String> {
        let mut out: Vec<String> = [
            ("critical flux m_cr (g m-2 s-1)", self.m_cr_g_m2_s),
            ("growth diffusivity D (mm2/s)", self.d_mm2_s),
            ("quasi-steady radius r_ss (mm)", self.r_ss_mm),
            ("extinction radius r_ext (mm)", self.r_ext_mm),
            ("reduced flow Phi", self.phi),
            ("oscillation clock (s)", self.osc_tau_s),
            ("burner rise dT_eq (K)", self.burner_rise_k),
        ]
        .into_iter()
        .filter(|(_, v)| !(v.is_finite() && *v > 0.0))
        .map(|(name, v)| format!("{name} = {v} is not finite and positive at these conditions"))
        .collect();
        if !self.p_weak.is_finite() {
            out.push(format!(
                "weak-class probability = {} is not finite",
                self.p_weak
            ));
        }
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

/// Log-logistic survival `1 / (1 + (t / tau)^k)`; 1 at and before the origin.
#[inline]
fn ll_survival(t: f64, tau: f64, k: f64) -> f64 {
    if !(t > 0.0) {
        return 1.0;
    }
    1.0 / (1.0 + (t / tau).powf(k))
}

/// Inverse complementary error function on (0, 1), from the normal quantile.
#[inline]
fn erfcinv(y: f64) -> f64 {
    -normal_quantile(0.5 * y) / std::f64::consts::SQRT_2
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

/// Time for a flame growing as `r_f = r_ss erfc(r_f / (2 sqrt(D t)))` to reach `r_ext`, given the
/// log of the reduced flow `ln_phi = ln (r_ss / r_ext)^2` and the time scale `t_g = r_ext^2 / (4 D)`;
/// `None` when the flame settles inside `r_ext` or the time exceeds `cap`. Taken in logs so that a
/// node far out in the spread cannot overflow Phi and hand 0 to the normal quantile.
#[inline]
fn growth_time(ln_phi: f64, t_g: f64, cap: f64) -> Option<f64> {
    if !(ln_phi > 0.0) || !(t_g > 0.0) {
        return None;
    }
    // r_ext / r_ss, inside (0, 1) for every finite Phi > 1
    let y = (-0.5 * ln_phi).exp();
    if !(y > 0.0 && y < 1.0) {
        return None;
    }
    let e = erfcinv(y);
    if !(e > 0.0) {
        return None;
    }
    let t = t_g / (e * e);
    (t.is_finite() && t < cap).then_some(t)
}

/// Time for a burner heated as `t0 + dT (1 - exp(-t / tau))` to rise by `need` K, given `ln dT`;
/// `None` when `dT <= need`.
#[inline]
fn limit_time(need: f64, ln_dt: f64, tau: f64) -> Option<f64> {
    let frac = need * (-ln_dt).exp();
    (frac < 1.0).then(|| -tau * (-frac).ln_1p())
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
            se_flux: net("se.flux"),
            se_diff: net("se.diff"),
            se_flash: net("se.flash"),
            se_osc: net("se.osc"),
            bt_rise: net("bt.rise"),
            rho_d: p.s("se.rho_d_g_m_s"),
            sigma_phi: p.s("se.sigma_phi"),
            hold: p.s("se.hold_log_rate").exp(),
            mw_fuel: [
                p.s("phys.mw_ethane"),
                p.s("phys.mw_propane"),
                p.s("phys.mw_butane"),
            ],
            mw_n2: p.s("phys.mw_n2"),
            mw_o2: p.s("phys.mw_o2"),
            nodes: (p.s("num.phi_nodes") as usize).clamp(1, MAX_NODES),
            bt_nodes: (p.s("num.bt_nodes") as usize).clamp(1, MAX_NODES),
            tau_cap: p.s("num.tau_cap_s"),
            floor: p.s("se.floor_s"),
            se_shape: p.s("se.shape"),
            osc_shape: p.s("se.osc_shape"),
            flash_tau: p.s("se.flash_tau_s"),
            flash_shape: p.s("se.flash_shape"),
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
        // SI: kg m-2 s-1, m2/s, kg m-1 s-1
        let m_cr = self.se_flux.eval(c, &mut scratch).exp() * 1.0e-3;
        let d = self.se_diff.eval(c, &mut scratch).exp() * 1.0e-6;
        let rho_d = self.rho_d * 1.0e-3;
        let r_ss = b.m_tot_kg_s / (four_pi * rho_d * b.l);
        let r_ext = (b.m_r_kg_s / (four_pi * m_cr)).sqrt();
        let phi = (r_ss / r_ext).powi(2);
        let t_g = r_ext * r_ext / (4.0 * d);
        let n = self.nodes;
        let ln_phi = phi.ln();
        let node_tau_s: Vec<Option<f64>> = (0..n)
            .map(|i| growth_time(ln_phi + self.sigma_phi * node_z(i, n), t_g, self.tau_cap))
            .collect();
        let p_held = node_tau_s.iter().filter(|t| t.is_none()).count() as f64 / n as f64;
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
            m_cr_g_m2_s: m_cr * 1.0e3,
            d_mm2_s: d * 1.0e6,
            r_ss_mm: r_ss * 1.0e3,
            r_ext_mm: r_ext * 1.0e3,
            phi,
            radiative_tau_s: growth_time(ln_phi, t_g, self.tau_cap),
            node_tau_s,
            p_held,
            osc_tau_s: self.se_osc.eval(c, &mut scratch).exp(),
            hold_per_s: self.hold,
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

    /// Survival of the weak class and of main-class node `i` from the time origin.
    #[inline]
    fn s_weak(&self, k: &Clocks, t: f64) -> f64 {
        ll_survival(t, k.weak_tau_s, self.flash_shape)
    }

    #[inline]
    fn s_node(&self, k: &Clocks, i: usize, t: f64) -> f64 {
        let tp = (t - k.floor_s).max(0.0);
        let grow = k.node_tau_s[i].map_or(1.0, |tau| ll_survival(tp, tau, self.se_shape));
        grow * ll_survival(tp, k.osc_tau_s, self.osc_shape) * (-k.hold_per_s * tp).exp()
    }

    /// The three family survivals from the time origin: self-extinction, burner limit, stops.
    fn survivals(&self, k: &Clocks, t: f64) -> (f64, f64, f64) {
        let n = k.node_tau_s.len();
        let main: f64 = (0..n).map(|i| self.s_node(k, i, t)).sum::<f64>() / n as f64;
        let se = k.p_weak * self.s_weak(k, t) + (1.0 - k.p_weak) * main;
        let bt = k.p_limit_never
            + k.pairs
                .iter()
                .map(|&(w, tau)| w * ll_survival(t, tau, self.bt_shape))
                .sum::<f64>();
        let mut ps = (-k.background_per_s * t.max(0.0)).exp();
        for &(ta, p) in &k.programmed {
            let f = ((t - (ta - self.atom_half)) / (2.0 * self.atom_half)).clamp(0.0, 1.0);
            ps *= (1.0 - p).powf(f);
        }
        (se, bt.min(1.0), ps)
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
    /// Probability, given a flame burning at the cutoff, of a main-class node with no growth clock.
    pub p_held_at_cutoff: f64,
    pub scheduled_stop_s: Option<f64>,
    pub times_s: Vec<f64>,
    /// Rows in [`CATEGORIES`] order.
    pub table: [Vec<f64>; 4],
    pub steps: usize,
}

/// `-ln rho` of one family over a step, from its survivals at both ends (0 when there is no
/// survival left to lose).
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
    let (se_cut, _, _) = wt.survivals(&clocks, tau);
    let n = clocks.node_tau_s.len();
    let p_weak = clocks.p_weak * wt.s_weak(&clocks, tau) / se_cut;
    let p_held: f64 = (0..n)
        .filter(|&i| clocks.node_tau_s[i].is_none())
        .map(|i| (1.0 - clocks.p_weak) / n as f64 * wt.s_node(&clocks, i, tau))
        .sum::<f64>()
        / se_cut;
    let mut steps = 0usize;
    let (mut t, mut s, mut f_e, mut f_o) = (tau, 1.0_f64, 0.0_f64, 0.0_f64);
    let mut prev = wt.survivals(&clocks, tau);
    let mut table: [Vec<f64>; 4] = std::array::from_fn(|_| vec![0.0; nt]);
    for (j, &tj) in times.iter().enumerate() {
        let target = tj.min(end);
        if target > t {
            let n_sub = (((target - t) / wt.dt) - 1e-9).ceil().max(1.0) as usize;
            let h = (target - t) / n_sub as f64;
            for k in 0..n_sub {
                let tb = if k + 1 == n_sub {
                    target
                } else {
                    t + h * (k + 1) as f64
                };
                let next = wt.survivals(&clocks, tb);
                let le = neg_ln_ratio(prev.0, next.0);
                let lo = neg_ln_ratio(prev.1, next.1) + neg_ln_ratio(prev.2, next.2);
                let total = le + lo;
                if total > 0.0 {
                    let lost = s * -(-total).exp_m1();
                    let share_e = if total.is_finite() {
                        le / total
                    } else if le.is_infinite() && lo.is_infinite() {
                        0.5
                    } else if le.is_infinite() {
                        1.0
                    } else {
                        0.0
                    };
                    f_e += lost * share_e;
                    f_o += lost * (1.0 - share_e);
                    s -= lost;
                }
                prev = next;
                steps += 1;
            }
            t = target;
        }
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
        p_held_at_cutoff: p_held,
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
    let bad = clocks.not_finite();
    if !bad.is_empty() {
        return Ok(Err(Support {
            admitted: false,
            gate: SupportGate::Condition,
            reasons: bad,
        }));
    }
    let (se, bt, ps) = wt.survivals(&clocks, tau);
    if !(se > 0.0 && bt > 0.0 && ps > 0.0) {
        return Ok(Err(refuse(
            SupportGate::Condition,
            format!("the release gives no survival to the cutoff (self-extinction {se}, burner limit {bt}, stops {ps}): a flame burning at the cutoff is outside it"),
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
        "p_held_at_cutoff": f.p_held_at_cutoff,
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

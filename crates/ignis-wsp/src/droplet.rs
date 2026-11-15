//! `flex_methanol.wsp` and `flex_heptane.wsp` — the multistate droplet forward.
//!
//! A free-floating droplet is in one of three states: burning inside a hot flame, burning inside a
//! cool flame (n-heptane only), or evaporating with no flame. The forward keeps a probability for
//! each and moves it with handwritten laws:
//!
//! ```text
//! q = D^2,  w = 1 - q/q0 (burn progress),  s = t K_ref / q0 (diffusion clock),
//! c = (c_o2, c_co2, c_he, c_logp, c_logd0),   x_exc = c_logd0 - N_crit(c_o2, c_co2, c_he, c_logp)
//!
//! hot        dq/dt = -K_h,        ln(K_h/K_ref) = N_rate(c, w, s) + b + s_z z
//! class      P(radiative) = sigmoid(kappa x_exc + bias)          x_exc = ln(D0 / D_crit)
//! diffusive  flame out at progress W:  Lambda_D(w) = ln2 (exp(a_D (w - w_D)) - exp(-a_D w_D)),
//!                                      w_D = sigmoid(N_diff(c, x_exc))
//! radiative  flame out at clock S:     Lambda_R(s) = ln2 (exp(a_R (s/s_R - 1)) - exp(-a_R)),
//!                                      s_R = exp(N_rad(c, x_exc))
//! cool       after a radiative extinction, with probability sigmoid(N_cool_p(c, x_exc)):
//!            dq/dt = -K_c,  ln(K_c/K_ref) = N_cool_k(c) + b + s_z z,  until D = D_ce,
//!            ln(D_ce/D_ref) = N_cool_d(c) + sd_ce u
//! no flame   D^2 falls by  k_inf (q_e/q_ref)^m dt + k_hot tau_e (1 - exp(-dt/tau_e)),  tau_e = tau0 q_e/q_ref
//!
//! N(x) = b0 + skip . x + sum_j w2[j] relu(b1[j] + W1[j, :] . x)
//! ```
//!
//! Two extinction mechanisms are kept apart because they are different physics. A droplet smaller
//! than the critical diameter of its atmosphere burns until its own history ends the flame (water
//! taken up by methanol, the small size of an alkane droplet): that is decided by burn progress. A
//! larger one loses its flame to radiation once the flame has grown, a fixed fraction of a diffusion
//! time after ignition: that is decided by the clock.
//!
//! The prefix enters three ways, all declared input transforms: the level the forecast starts from
//! (a window mean carried to the cutoff along the observed finite difference), the run's log-rate
//! offset `b` (gain times clamped log-ratios of observed to modelled finite-difference rates), and
//! the state at the cutoff (prior state probabilities from the laws above at the observed progress
//! and clock, multiplied by handwritten Gaussian likelihoods of the observed late rate under each
//! state). Before any of that, a physical validity gate drops diameters that later frames contradict:
//! a droplet does not grow, so a value exceeded by several later values is a tracker artefact
//! (igniter glare in the first frames of the 2009 runs, partial occlusion at the edge of the view).
//!
//! Nothing here fits anything. There is no gradient, least-squares, parameter search, sampler or
//! seed on this route; the hot phase is integrated with a fixed-step RK4 and everything after a
//! transition is closed form.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    AnalysisArtifact, DomainBound, Family, FeatureSpec, Finite, ForwardContract, ForwardProfile,
    ModelRelease, ModelSpec, PrefixBundle, QualificationState, Quantity, SpecId, Support,
    SupportGate, TargetSpec, TensorRole, WspSchema,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::LoadedRelease;
use ignis_core::resources::Profile;
use ignis_core::views::{measurement_valid, View, FLEX_CAUSES};

use crate::common::{self, admitted, refuse, Body};
use crate::nn::{hinge_net, sigmoid};
use crate::quad::normal_cdf;
use crate::spec::{authored, constant, sparse, Params};

pub const FORWARD_VERSION: &str = "2";
/// Hidden width of the rate network.
pub const H_RATE: usize = 16;
/// Hidden width of every law network.
pub const H_LAW: usize = 8;
/// Rate-network inputs: `c_o2, c_co2, c_he, c_logp, c_logd0, w, s`.
pub const N_RATE: usize = 7;
/// Critical-diameter inputs: `c_o2, c_co2, c_he, c_logp`.
pub const N_CRIT: usize = 4;
/// Law inputs: `c_o2, c_co2, c_he, c_logp, c_logd0, x_exc`.
pub const N_LAW: usize = 6;
/// Cool-flame rate and extinction-diameter inputs: `c_o2, c_co2, c_he, c_logp, c_logd0`.
pub const N_COOL: usize = 5;
pub const N_GAIN: usize = 3;
const MAX_Z: usize = 64;
const MAX_CE: usize = 16;

pub const EVENT_ACTIVE: &str = "active_supported";
pub const EVENT_UNRESOLVED: &str = "unresolved";

/// The fuels this forward has a spec for. The forward is shared; the weights are not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fuel {
    Methanol,
    Heptane,
}

impl Fuel {
    pub const fn spec_id(self) -> &'static str {
        match self {
            Fuel::Methanol => "flex_methanol-wsp-v2",
            Fuel::Heptane => "flex_heptane-wsp-v1",
        }
    }
    pub const fn model_name(self) -> &'static str {
        match self {
            Fuel::Methanol => "flex_methanol.wsp",
            Fuel::Heptane => "flex_heptane.wsp",
        }
    }
    /// The fuel as the PSI-69 summary writes it.
    pub const fn label(self) -> &'static str {
        match self {
            Fuel::Methanol => "Methanol",
            Fuel::Heptane => "Heptane",
        }
    }
    pub const fn has_cool(self) -> bool {
        matches!(self, Fuel::Heptane)
    }
    pub fn of_model(name: &str) -> Option<Fuel> {
        [Fuel::Methanol, Fuel::Heptane]
            .into_iter()
            .find(|f| f.model_name() == name)
    }
}

fn fin(v: f64) -> Finite {
    Finite::new(v).expect("finite literal")
}

/// The frozen specification an author writes against.
pub fn spec(fuel: Fuel) -> ModelSpec {
    use TensorRole::*;
    let feature = |name: &str, source: &str, transform: &str| FeatureSpec {
        name: name.into(),
        unit: "1".into(),
        source: source.into(),
        transform: transform.into(),
    };
    let bound = |f: &str, lo: f64, hi: f64| DomainBound {
        feature: f.into(),
        min: Some(fin(lo)),
        max: Some(fin(hi)),
    };
    let (hr, nr) = (H_RATE as u32, N_RATE as u32);
    let (hl, nl, nc, nk) = (H_LAW as u32, N_LAW as u32, N_CRIT as u32, N_COOL as u32);
    let net = |name: &str, h: u32, n: u32, what: &str, inputs: &str| {
        vec![
            sparse(
                &format!("{name}.W1"),
                &[h, n],
                Weight,
                None,
                &format!("Hidden weights of {what}; row j reads x = ({inputs})."),
            ),
            sparse(
                &format!("{name}.b1"),
                &[h],
                Bias,
                None,
                &format!("Hidden biases of {what} (hinge positions)."),
            ),
            sparse(
                &format!("{name}.w2"),
                &[h],
                Weight,
                None,
                &format!("Output weights of {what} (slope changes at the hinges)."),
            ),
            authored(
                &format!("{name}.skip"),
                &[n],
                Weight,
                None,
                None,
                &format!("Linear part of {what} in x; every element is stated, including zeros."),
            ),
        ]
    };
    let law_in = "c_o2, c_co2, c_he, c_logp, c_logd0, x_exc";
    let cool_in = "c_o2, c_co2, c_he, c_logp, c_logd0";
    let mut tensors = Vec::new();
    // ---- hot-flame burning-rate network
    tensors.extend(net(
        "rate",
        hr,
        nr,
        "the hot-flame rate network ln(K_h/K_ref)",
        "c_o2, c_co2, c_he, c_logp, c_logd0, w, s",
    ));
    tensors.push(authored(
        "rate.b0",
        &[],
        PhysicalParameter,
        Some("ln(K / 1e-6 m2/s)"),
        Some((-3.0, 2.0)),
        "ln(K_h/K_ref) at x = 0: air, 1e5 Pa, D0 = 2 mm, w = 0, s = 0, before hinge terms.",
    ));
    // ---- critical diameter of radiative extinction
    tensors.extend(net(
        "crit",
        hl,
        nc,
        "the critical-diameter law ln(D_crit/D_ref)",
        "c_o2, c_co2, c_he, c_logp",
    ));
    tensors.push(authored("crit.b0", &[], PhysicalParameter, Some("ln(D / 2 mm)"), Some((-4.0, 6.0)),
        "ln(D_crit/D_ref) in air at 1e5 Pa, before hinge terms: droplets larger than D_crit lose the hot flame to radiation."));
    tensors.push(authored(
        "class.kappa",
        &[],
        Weight,
        None,
        Some((0.0, 60.0)),
        "Sharpness of the radiative class in x_exc = ln(D0/D_crit).",
    ));
    tensors.push(authored(
        "class.bias",
        &[],
        Bias,
        None,
        Some((-20.0, 20.0)),
        "Logit of the radiative class at D0 = D_crit.",
    ));
    // ---- diffusive branch: progress at which the hot flame goes out
    tensors.extend(net(
        "diff",
        hl,
        nl,
        "the diffusive-extinction law logit(w_D)",
        law_in,
    ));
    tensors.push(authored(
        "diff.b0",
        &[],
        HazardParameter,
        Some("logit(1)"),
        Some((-12.0, 12.0)),
        "logit of the median burn progress at diffusive extinction at x = 0, before hinge terms.",
    ));
    tensors.push(authored(
        "diff.sharp",
        &[],
        HazardParameter,
        Some("1"),
        Some((1.0, 80.0)),
        "a_D: Gompertz sharpness of diffusive extinction in burn progress.",
    ));
    // ---- radiative branch: clock at which the hot flame goes out
    tensors.extend(net(
        "rad",
        hl,
        nl,
        "the radiative-extinction law ln(s_R)",
        law_in,
    ));
    tensors.push(authored(
        "rad.b0",
        &[],
        HazardParameter,
        Some("ln(1)"),
        Some((-8.0, 4.0)),
        "ln of the median diffusion clock at radiative extinction at x = 0, before hinge terms.",
    ));
    tensors.push(authored(
        "rad.sharp",
        &[],
        HazardParameter,
        Some("1"),
        Some((0.5, 40.0)),
        "a_R: Gompertz sharpness of radiative extinction in s/s_R.",
    ));
    if fuel.has_cool() {
        tensors.extend(net(
            "cool.p",
            hl,
            nl,
            "the cool-flame onset logit after a radiative extinction",
            law_in,
        ));
        tensors.push(authored(
            "cool.p.b0",
            &[],
            HazardParameter,
            Some("logit(1)"),
            Some((-30.0, 30.0)),
            "Logit of a cool flame following radiative extinction at x = 0, before hinge terms.",
        ));
        tensors.extend(net(
            "cool.k",
            hl,
            nk,
            "the cool-flame rate law ln(K_c/K_ref)",
            cool_in,
        ));
        tensors.push(authored(
            "cool.k.b0",
            &[],
            PhysicalParameter,
            Some("ln(K / 1e-6 m2/s)"),
            Some((-4.0, 2.0)),
            "ln(K_c/K_ref) at x = 0, before hinge terms.",
        ));
        tensors.extend(net(
            "cool.d",
            hl,
            nk,
            "the cool-flame extinction-diameter law ln(D_ce/D_ref)",
            cool_in,
        ));
        tensors.push(authored(
            "cool.d.b0",
            &[],
            PhysicalParameter,
            Some("ln(D / 2 mm)"),
            Some((-4.0, 3.0)),
            "ln(D_ce/D_ref) at x = 0, before hinge terms.",
        ));
        tensors.push(authored(
            "cool.d.sd",
            &[],
            NoiseParameter,
            Some("1"),
            Some((0.0, 2.0)),
            "Assumed sd of ln D_ce between droplets of one condition.",
        ));
        tensors.push(authored("post.cool_scale", &[], PhysicalParameter, Some("1"), Some((0.0, 10.0)),
            "Multiplier of post.k_hot after a cool flame (the surroundings are cooler than after a hot flame)."));
    }
    tensors.extend([
        // ---- droplet without a flame
        authored("post.k_inf", &[], PhysicalParameter, Some("m2/s"), Some((0.0, 2.0e-7)),
                 "Long-time d(D^2)/dt magnitude of an extinguished droplet of D = D_ref."),
        authored("post.k_hot", &[], PhysicalParameter, Some("m2/s"), Some((0.0, 5.0e-7)),
                 "Extra d(D^2)/dt magnitude right after extinction (the gas is still hot); decays with tau_e."),
        authored("post.tau0_s", &[], PhysicalParameter, Some("s"), Some((0.01, 100.0)),
                 "Decay time of post.k_hot for a droplet extinguished at D = D_ref; scales with D_e^2."),
        authored("post.size_exp", &[], PhysicalParameter, Some("1"), Some((0.0, 2.0)),
                 "Exponent of (q_e/q_ref) in the long-time evaporation rate."),
        // ---- prefix encoder
        authored("enc.gain_w", &[N_GAIN as u32], Weight, None, None,
                 "Weights of the gain logit on (g_span, g_count, g_progress)."),
        authored("enc.gain_b", &[], Bias, None, None, "Bias of the gain logit; gain = sigmoid(logit)."),
        authored("enc.mix", &[2], Weight, None, Some((0.0, 1.0)),
                 "Weights of (e_late, e_early) in the run's log-rate innovation."),
        authored("enc.e_clip", &[2], Weight, None, Some((0.0, 3.0)),
                 "Clamp of the log-rate innovations: (below, above) the population rate."),
        // ---- state at the cutoff
        authored("filt.sd_evap", &[], NoiseParameter, Some("1"), Some((0.01, 2.0)),
                 "Assumed sd of the late-rate ratio r = K_obs/K_h of a droplet without a flame."),
        authored("filt.l_floor", &[], NoiseParameter, Some("1"), Some((0.0, 1.0)),
                 "Density added to every state's likelihood of r, so that an absurd r returns the prior."),
        // ---- assumed spread and noise
        authored("unc.s_pop", &[], NoiseParameter, Some("1"), Some((0.0, 2.0)),
                 "Assumed sd of a run's ln K around the population law before any prefix evidence."),
        authored("unc.s_floor", &[], NoiseParameter, Some("1"), Some((0.0, 2.0)),
                 "Assumed sd of ln K that no prefix removes."),
        authored("unc.sigma_d", &[], NoiseParameter, Some("m"), Some((0.0, 1.0e-3)),
                 "Assumed sd of one diameter measurement around the latent diameter (level error included)."),
        authored("unc.sigma_frame", &[], NoiseParameter, Some("m"), Some((0.0, 1.0e-3)),
                 "Assumed frame-to-frame sd of the diameter (what a finite difference of window means sees)."),
        authored("haz.log_disruption", &[], HazardParameter, Some("ln(1/s)"), Some((-20.0, 3.0)),
                 "ln of the constant disruption hazard of a burning droplet (reported, does not change the diameter)."),
        // ---- constants fixed by the spec
        constant("norm.d_ref_m", 0.002, "m", "Reference diameter of c_logd0, D_crit and D_ce."),
        constant("norm.p_ref_pa", 1.0e5, "Pa", "Reference pressure of c_logp."),
        constant("norm.k_ref_m2_s", 1.0e-6, "m2/s", "Reference burning-rate constant and clock rate."),
        constant("norm.n_ref", 32.0, "1", "Reference number of prefix points of g_count."),
        constant("enc.window_frac", 0.25, "1", "Head, mid and tail windows each span this fraction of the prefix duration."),
        constant("enc.k_floor_m2_s", 2.0e-8, "m2/s", "Floor of an observed rate before its logarithm."),
        constant("enc.w_ref", 0.08, "1", "Progress at which g_progress saturates."),
        constant("enc.min_points", 2.0, "1", "Fewest valid prefix diameters for a forecast."),
        constant("gate.growth_tol", 0.05, "1", "A diameter is dropped when later prefix values of D^2 exceed it by more than this fraction."),
        constant("gate.min_later", 3.0, "1", "Number of later contradicting values that drops a diameter."),
        constant("gate.k_max_m2_s", 2.0e-6, "m2/s", "Largest d(D^2)/dt magnitude any flame of these experiments produces (twice the fastest measured rate): a later value below what this allows contradicts an earlier one, and the level is not carried to the cutoff faster than this."),
        constant("num.dt_s", 0.05, "s", "RK4 step of the hot phase (uniform grid from the cutoff)."),
        constant("num.bin_s", 0.5, "s", "Width of the extinction-time bins the mixture atoms are built from."),
        constant("num.d_min_m", 2.5e-4, "m", "Support floor: the model stops at this diameter."),
        constant("num.t_valid_s", 45.0, "s", "Model horizon since the time origin."),
        constant("num.z_step", 0.5, "1", "Spacing of the scenario nodes of z."),
        constant("num.z_count", 15.0, "1", "Number of scenario nodes of z, centred on 0."),
        constant("num.ce_step", 1.0, "1", "Spacing of the scenario nodes of the cool-flame extinction diameter."),
        constant("num.ce_count", 5.0, "1", "Number of scenario nodes of the cool-flame extinction diameter."),
    ]);
    let mut excludes = vec![
        "visible or UV flame diameter".to_string(),
        "flame temperature, soot, species or radiant emission".to_string(),
        "fibre-supported droplets and other fuels".to_string(),
        "a calibrated posterior: the spread is an assumed scenario distribution".to_string(),
    ];
    if !fuel.has_cool() {
        excludes.push("cool-flame burning (not observed for methanol)".into());
    }
    ModelSpec {
        schema_version: WspSchema::V1,
        spec_id: SpecId::new(fuel.spec_id()).expect("id"),
        family: Family::Flex,
        model_name: fuel.model_name().into(),
        target: TargetSpec {
            quantity: Quantity::LiquidDiameter,
            unit: "m".into(),
            description: format!(
                "Measured liquid-droplet diameter of a free-floating {} droplet at times after the \
                 cutoff, as an unconditional predictive distribution over the droplet's state (hot \
                 flame{}, no flame), plus the cumulative incidence of the end of the hot-flame phase.",
                fuel.label().to_lowercase(),
                if fuel.has_cool() { ", cool flame" } else { "" }
            ),
            excludes,
        },
        features: vec![
            feature("c_o2", "condition:x_o2", "(x - 0.21) / 0.05 @1"),
            feature("c_co2", "condition:x_co2", "x / 0.20 @1"),
            feature("c_he", "condition:x_he", "x / 0.20 @1"),
            feature("c_logp", "condition:pressure", "ln(x / norm.p_ref_pa) @1"),
            feature("c_logd0", "observation:liquid_diameter@prefix_head",
                    "ln(sqrt(q0) / norm.d_ref_m); q0 = mean(D^2) over the head window advanced to t = 0 with the population rate at w = 0 @2"),
            feature("x_exc", "state:supercriticality", "c_logd0 - N_crit(c_o2, c_co2, c_he, c_logp) = ln(D0 / D_crit) @2"),
            feature("w", "state:burn_progress", "clamp(1 - D^2 / q0, 0, 1) @1"),
            feature("s", "state:clock", "t * norm.k_ref_m2_s / q0 with t in seconds since the time origin (igniter energised) @1"),
            feature("e_late", "observation:liquid_diameter@prefix_mid_tail",
                    "clamp(ln(max(K_obs, enc.k_floor_m2_s) / K_model), -enc.e_clip[0], enc.e_clip[1]); K_obs = finite difference of window means of D^2 (mid -> tail); K_model = harmonic Simpson mean of the hot-flame population rate over the same interval @2"),
            feature("e_early", "observation:liquid_diameter@prefix_head_mid", "as e_late for head -> mid; both equal the head -> tail value when the mid window is empty @2"),
            feature("r_late", "observation:liquid_diameter@prefix_mid_tail", "K_obs / K_model of the late finite difference, unclamped (state evidence) @2"),
            feature("g_span", "observation:time@prefix", "ln((mean time of tail window - mean time of head window) / 1 s) @1"),
            feature("g_count", "observation:count@prefix", "ln(n_valid / norm.n_ref) @1"),
            feature("g_progress", "state:burn_progress@prefix_tail", "clamp(w_tail / enc.w_ref, 0, 1) @1"),
        ],
        domain: vec![
            bound("c_o2", -2.2, 4.8),
            bound("c_co2", 0.0, 4.0),
            bound("c_he", 0.0, 3.0),
            bound("c_logp", -1.0, 1.5),
            bound("c_logd0", -0.9, 1.26),
        ],
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::droplet::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(
            "Prefix observations: liquid_diameter, valid, positive, 0 <= t <= cutoff, first of equal \
             time stamps, then the validity gate (a value is dropped when at least gate.min_later later \
             prefix values of D^2 contradict it: larger by more than gate.growth_tol, since a droplet \
             does not grow, or smaller than gate.k_max_m2_s allows in the time between). \
             State at the cutoff: tail-window mean of D^2 carried to the cutoff along the observed \
             mid -> tail finite difference limited to [0, gate.k_max_m2_s] (no weight enters it). State probabilities at the cutoff: \
             prior from the extinction laws at the observed progress and clock, times \
             (N(r_late; mean_state, sd_state) + filt.l_floor)^gain, normalised; means 1 (hot), \
             K_c/K_h (cool), k_evap/K_h (no flame); sds from unc.s_pop, filt.sd_evap and the \
             finite-difference noise implied by unc.sigma_frame. Hot phase: RK4 on a uniform grid of \
             num.dt_s from the cutoff, linear interpolation to the forecast times; extinction mass \
             collected in bins of num.bin_s; everything after a transition is closed form. Scenario \
             nodes z_k = (k - (n-1)/2) * z_step with weights proportional to exp(-z^2/2), for the rate \
             and, independently, for ln D_ce. Predictive quantiles: quantiles of the mixture of Gaussians \
             of sd unc.sigma_d centred on the mixture atoms (atoms pooled on a grid of sigma_d/4; each \
             quantile by bisection of the monotone mixture CDF). Latent quantiles: step quantiles of \
             the atoms. Uncertainty kind: assumed_noise."
                .into(),
        ),
    }
}

/// One hinge network bound to a release.
#[derive(Clone, Copy)]
pub(crate) struct Net<'a> {
    w1: &'a [f64],
    b1: &'a [f64],
    w2: &'a [f64],
    skip: &'a [f64],
    b0: f64,
}

impl Net<'_> {
    #[inline]
    pub(crate) fn eval(&self, x: &[f64], scratch: &mut [f64; H_RATE]) -> f64 {
        hinge_net(
            self.w1,
            self.b1,
            self.w2,
            self.skip,
            self.b0,
            x,
            &mut scratch[..self.b1.len()],
        )
    }
}

struct Cool<'a> {
    p: Net<'a>,
    k: Net<'a>,
    d: Net<'a>,
    sd: f64,
    post_scale: f64,
}

/// Every number of a release the forward reads, by name.
pub(crate) struct Weights<'a> {
    pub(crate) rate: Net<'a>,
    crit: Net<'a>,
    kappa: f64,
    class_bias: f64,
    diff: Net<'a>,
    a_d: f64,
    rad: Net<'a>,
    a_r: f64,
    cool: Option<Cool<'a>>,
    k_inf: f64,
    k_hot: f64,
    tau0: f64,
    size_exp: f64,
    gain_w: &'a [f64],
    gain_b: f64,
    mix: &'a [f64],
    e_lo: f64,
    e_hi: f64,
    sd_evap: f64,
    l_floor: f64,
    s_pop: f64,
    s_floor: f64,
    sigma_d: f64,
    sigma_frame: f64,
    lam_dis: f64,
    d_ref: f64,
    p_ref: f64,
    pub(crate) k_ref: f64,
    n_ref: f64,
    window_frac: f64,
    k_floor: f64,
    w_ref: f64,
    min_points: usize,
    growth_tol: f64,
    min_later: usize,
    k_max: f64,
    dt: f64,
    bin_s: f64,
    q_min: f64,
    t_valid: f64,
    z_step: f64,
    z_count: usize,
    ce_step: f64,
    ce_count: usize,
}

impl<'a> Weights<'a> {
    pub(crate) fn bind(p: &Params<'a>, fuel: Fuel) -> Result<Self, Refusal> {
        let net = |prefix: &str| Net {
            w1: p.v(&format!("{prefix}.W1")),
            b1: p.v(&format!("{prefix}.b1")),
            w2: p.v(&format!("{prefix}.w2")),
            skip: p.v(&format!("{prefix}.skip")),
            b0: p.s(&format!("{prefix}.b0")),
        };
        let d_min = p.s("num.d_min_m");
        let z_count = p.s("num.z_count") as usize;
        let ce_count = p.s("num.ce_count") as usize;
        if !(3..=MAX_Z).contains(&z_count) || z_count.is_multiple_of(2) {
            return Err(Refusal::failed(
                "spec_constants",
                "num.z_count must be odd, 3..=64",
            ));
        }
        if !(1..=MAX_CE).contains(&ce_count) || ce_count.is_multiple_of(2) {
            return Err(Refusal::failed(
                "spec_constants",
                "num.ce_count must be odd, 1..=16",
            ));
        }
        let clip = p.v("enc.e_clip");
        Ok(Weights {
            rate: net("rate"),
            crit: net("crit"),
            kappa: p.s("class.kappa"),
            class_bias: p.s("class.bias"),
            diff: net("diff"),
            a_d: p.s("diff.sharp"),
            rad: net("rad"),
            a_r: p.s("rad.sharp"),
            cool: fuel.has_cool().then(|| Cool {
                p: net("cool.p"),
                k: net("cool.k"),
                d: net("cool.d"),
                sd: p.s("cool.d.sd"),
                post_scale: p.s("post.cool_scale"),
            }),
            k_inf: p.s("post.k_inf"),
            k_hot: p.s("post.k_hot"),
            tau0: p.s("post.tau0_s"),
            size_exp: p.s("post.size_exp"),
            gain_w: p.v("enc.gain_w"),
            gain_b: p.s("enc.gain_b"),
            mix: p.v("enc.mix"),
            e_lo: clip[0],
            e_hi: clip[1],
            sd_evap: p.s("filt.sd_evap"),
            l_floor: p.s("filt.l_floor"),
            s_pop: p.s("unc.s_pop"),
            s_floor: p.s("unc.s_floor"),
            sigma_d: p.s("unc.sigma_d"),
            sigma_frame: p.s("unc.sigma_frame"),
            lam_dis: p.s("haz.log_disruption").exp(),
            d_ref: p.s("norm.d_ref_m"),
            p_ref: p.s("norm.p_ref_pa"),
            k_ref: p.s("norm.k_ref_m2_s"),
            n_ref: p.s("norm.n_ref"),
            window_frac: p.s("enc.window_frac"),
            k_floor: p.s("enc.k_floor_m2_s"),
            w_ref: p.s("enc.w_ref"),
            min_points: p.s("enc.min_points") as usize,
            growth_tol: p.s("gate.growth_tol"),
            min_later: p.s("gate.min_later") as usize,
            k_max: p.s("gate.k_max_m2_s"),
            dt: p.s("num.dt_s"),
            bin_s: p.s("num.bin_s"),
            q_min: d_min * d_min,
            t_valid: p.s("num.t_valid_s"),
            z_step: p.s("num.z_step"),
            z_count,
            ce_step: p.s("num.ce_step"),
            ce_count,
        })
    }

    /// Hot-flame population burning-rate constant (m^2/s), without run offset or scenario.
    #[inline]
    pub(crate) fn k_hot_pop(
        &self,
        c: &[f64; 5],
        w: f64,
        s: f64,
        scratch: &mut [f64; H_RATE],
    ) -> f64 {
        let x = [c[0], c[1], c[2], c[3], c[4], w, s];
        self.k_ref * self.rate.eval(&x, scratch).exp()
    }

    /// Harmonic Simpson mean of the hot-flame population rate between two prefix windows `(t, q)`.
    fn k_mean(
        &self,
        c: &[f64; 5],
        q0: f64,
        a: (f64, f64),
        b: (f64, f64),
        scratch: &mut [f64; H_RATE],
    ) -> f64 {
        let w = |q: f64| (1.0 - q / q0).clamp(0.0, 1.0);
        let s = |t: f64| t * self.k_ref / q0;
        let (wa, wb, sa, sb) = (w(a.1), w(b.1), s(a.0), s(b.0));
        let inv = (1.0 / self.k_hot_pop(c, wa, sa, scratch)
            + 4.0 / self.k_hot_pop(c, 0.5 * (wa + wb), 0.5 * (sa + sb), scratch)
            + 1.0 / self.k_hot_pop(c, wb, sb, scratch))
            / 6.0;
        1.0 / inv
    }

    /// The condition-level laws of one droplet.
    pub(crate) fn laws(&self, c: &[f64; 5], scratch: &mut [f64; H_RATE]) -> Laws {
        let ln_dcrit = self.crit.eval(&c[..N_CRIT], scratch);
        let x_exc = c[4] - ln_dcrit;
        let x = [c[0], c[1], c[2], c[3], c[4], x_exc];
        let pi_r = sigmoid(self.kappa * x_exc + self.class_bias);
        let w_d = sigmoid(self.diff.eval(&x, scratch));
        let s_r = self.rad.eval(&x, scratch).exp();
        let (p_cool, k_cool, d_ce, sd_ce) = match &self.cool {
            Some(cool) => (
                sigmoid(cool.p.eval(&x, scratch)),
                self.k_ref * cool.k.eval(&c[..N_COOL], scratch).exp(),
                self.d_ref * cool.d.eval(&c[..N_COOL], scratch).exp(),
                cool.sd,
            ),
            None => (0.0, 0.0, 0.0, 0.0),
        };
        Laws {
            d_crit_m: self.d_ref * ln_dcrit.exp(),
            x_exc,
            pi_r,
            w_d,
            s_r,
            p_cool,
            k_cool_m2_s: k_cool,
            d_ce_m: d_ce,
            sd_ce,
        }
    }

    /// Cumulative intensity of diffusive extinction at burn progress `w`.
    #[inline]
    fn lam_d(&self, l: &Laws, w: f64) -> f64 {
        std::f64::consts::LN_2 * ((self.a_d * (w - l.w_d)).exp() - (-self.a_d * l.w_d).exp())
    }

    /// Cumulative intensity of radiative extinction at clock `s`.
    #[inline]
    fn lam_r(&self, l: &Laws, s: f64) -> f64 {
        std::f64::consts::LN_2 * ((self.a_r * (s / l.s_r - 1.0)).exp() - (-self.a_r).exp())
    }

    /// Fall of D^2 over `dt` seconds for a droplet that lost its flame at `q_e`.
    #[inline]
    fn evap_drop(&self, dt: f64, q_e: f64, hot_scale: f64) -> f64 {
        let rel = q_e / (self.d_ref * self.d_ref);
        let tau_e = (self.tau0 * rel).max(1.0e-3);
        self.k_inf * rel.powf(self.size_exp) * dt
            + hot_scale * self.k_hot * tau_e * (1.0 - (-dt / tau_e).exp())
    }

    /// d(D^2)/dt magnitude right after extinction at `q_e`.
    #[inline]
    fn evap_rate0(&self, q_e: f64, hot_scale: f64) -> f64 {
        let rel = q_e / (self.d_ref * self.d_ref);
        self.k_inf * rel.powf(self.size_exp) + hot_scale * self.k_hot
    }
}

/// Condition-level laws of one droplet (reported in the diagnostics).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Laws {
    pub d_crit_m: f64,
    /// ln(D0 / D_crit).
    pub x_exc: f64,
    /// Prior probability of the radiative class.
    pub pi_r: f64,
    /// Median burn progress at diffusive extinction.
    pub w_d: f64,
    /// Median clock at radiative extinction.
    pub s_r: f64,
    pub p_cool: f64,
    pub k_cool_m2_s: f64,
    pub d_ce_m: f64,
    pub sd_ce: f64,
}

/// What the handwritten encoder made of a prefix (all of it is reported in the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct Encoded {
    /// `c_o2, c_co2, c_he, c_logp, c_logd0`.
    pub c: [f64; 5],
    pub n_valid: usize,
    /// Diameters dropped by the validity gate.
    pub n_gated: usize,
    pub q0_m2: f64,
    pub t_head_s: f64,
    pub q_head_m2: f64,
    pub t_tail_s: f64,
    pub q_tail_m2: f64,
    pub mid_window_used: bool,
    pub w_tail: f64,
    pub k_obs_late_m2_s: f64,
    pub k_obs_early_m2_s: f64,
    pub k_model_late_m2_s: f64,
    pub e_late: f64,
    pub e_early: f64,
    /// Late observed over modelled hot rate, unclamped.
    pub r_late: f64,
    /// Sd of `r_late` from frame noise alone.
    pub r_noise: f64,
    pub gain_inputs: [f64; N_GAIN],
    pub gain: f64,
    /// Run log-rate offset of the hot flame.
    pub b: f64,
    /// Run log-rate offset of a cool flame burning at the cutoff.
    pub b_cool: f64,
    /// Sd of the scenario variable in ln K.
    pub s_z: f64,
    pub laws: Laws,
    /// Prior probabilities of (hot, cool, no flame) at the cutoff.
    pub prior: [f64; 3],
    /// Probabilities of (hot, cool, no flame) at the cutoff after the rate evidence.
    pub state: [f64; 3],
    /// Probability of the radiative class given a hot flame at the cutoff.
    pub class_r_given_hot: f64,
    /// State at the cutoff.
    pub q_cutoff_m2: f64,
    pub cutoff_s: f64,
}

struct Window {
    t: f64,
    q: f64,
    n: usize,
}

fn window(obs: &[(f64, f64)], lo: f64, hi: f64) -> Window {
    let (mut st, mut sq, mut n) = (0.0, 0.0, 0usize);
    for &(t, q) in obs {
        if t >= lo && t <= hi {
            st += t;
            sq += q;
            n += 1;
        }
    }
    let d = n.max(1) as f64;
    Window {
        t: st / d,
        q: sq / d,
        n,
    }
}

/// The validity gate: keeps `(t, D^2)` unless at least `min_later` later values contradict it. A
/// later value contradicts an earlier one when it is larger by more than `tol` (a droplet does not
/// grow) or smaller than any flame could have made it in the time between (`k_max`, with the same
/// tolerance). Returns the kept observations and the number dropped.
pub(crate) fn validity_gate(
    obs: &[(f64, f64)],
    tol: f64,
    min_later: usize,
    k_max: f64,
) -> (Vec<(f64, f64)>, usize) {
    let n = obs.len();
    let mut keep = Vec::with_capacity(n);
    let mut dropped = 0usize;
    for i in 0..n {
        let (t_i, q_i) = obs[i];
        let grown = q_i * (1.0 + tol);
        let (mut above, mut below) = (0usize, 0usize);
        for o in &obs[i + 1..] {
            if o.1 > grown {
                above += 1;
            } else if o.1 < q_i * (1.0 - tol) - k_max * (o.0 - t_i) {
                below += 1;
            }
            if above >= min_later || below >= min_later {
                break;
            }
        }
        if above >= min_later || below >= min_later {
            dropped += 1;
        } else {
            keep.push(obs[i]);
        }
    }
    (keep, dropped)
}

#[inline]
fn gauss(x: f64, mean: f64, sd: f64) -> f64 {
    let z = (x - mean) / sd;
    (-0.5 * z * z).exp() / (sd * 2.506_628_274_631_000_7)
}

/// Prefix -> local state. Refusals carry the gate that stopped the forecast.
fn encode(wt: &Weights<'_>, spec: &ModelSpec, prefix: &PrefixBundle) -> Result<Encoded, Support> {
    // 1. semantic
    if prefix.family != Family::Flex {
        return Err(refuse(SupportGate::Semantic, "prefix is not a FLEX run"));
    }
    if prefix.phase_established_at_s.is_none() {
        return Err(refuse(
            SupportGate::Semantic,
            "modelled phase not established at the cutoff (phase_established_at_s missing)",
        ));
    }
    // 2. measurement
    let tau = prefix.cutoff_s;
    let mut raw: Vec<(f64, f64)> = Vec::with_capacity(prefix.observations.len());
    for o in &prefix.observations {
        if o.quantity != Quantity::LiquidDiameter || !measurement_valid(o) {
            continue;
        }
        let d = o.value_si.unwrap_or(f64::NAN);
        let usable = d > 0.0 && o.time_s >= 0.0 && o.time_s <= tau;
        if !usable {
            continue;
        }
        if raw.last().is_some_and(|l| l.0 == o.time_s) {
            continue;
        }
        if raw.last().is_some_and(|l| l.0 > o.time_s) {
            return Err(refuse(
                SupportGate::Measurement,
                "prefix observations are not time-sorted",
            ));
        }
        raw.push((o.time_s, d * d));
    }
    let (obs, n_gated) = validity_gate(&raw, wt.growth_tol, wt.min_later, wt.k_max);
    if obs.len() < wt.min_points {
        return Err(refuse(
            SupportGate::Measurement,
            format!(
                "{} valid liquid-diameter observations in [0, cutoff] (< {})",
                obs.len(),
                wt.min_points
            ),
        ));
    }
    // 3. conditions
    let mut reasons = Vec::new();
    let mut get = |k: &str| match common::condition(prefix, k) {
        Ok(v) => v,
        Err(e) => {
            reasons.push(e);
            f64::NAN
        }
    };
    let (o2, co2, he, p) = (get("x_o2"), get("x_co2"), get("x_he"), get("pressure"));
    if p.is_finite() && p <= 0.0 {
        reasons.push(format!("pressure {p} Pa is not positive"));
    }
    if !reasons.is_empty() {
        return Err(Support {
            admitted: false,
            gate: SupportGate::Condition,
            reasons,
        });
    }
    let mut c = [
        (o2 - 0.21) / 0.05,
        co2 / 0.20,
        he / 0.20,
        (p / wt.p_ref).ln(),
        0.0,
    ];
    // prefix windows
    let (t1, tn) = (obs[0].0, obs[obs.len() - 1].0);
    let span = tn - t1;
    let f = wt.window_frac;
    let head = window(&obs, t1, t1 + f * span);
    let tail = window(&obs, tn - f * span, tn);
    let mid = window(
        &obs,
        t1 + 0.5 * (1.0 - f) * span,
        t1 + 0.5 * (1.0 + f) * span,
    );
    let mut scratch = [0.0; H_RATE];
    // D0: head mean advanced to the time origin with the population rate at w = 0 (one pass).
    c[4] = (head.q.sqrt() / wt.d_ref).ln();
    let q0 = head.q + wt.k_hot_pop(&c, 0.0, 0.0, &mut scratch) * head.t;
    c[4] = (q0.sqrt() / wt.d_ref).ln();
    let names = ["c_o2", "c_co2", "c_he", "c_logp", "c_logd0"];
    let mut out = Vec::new();
    for d in &spec.domain {
        if let Some(i) = names.iter().position(|n| *n == d.feature) {
            let lo = d.min.map_or(f64::NEG_INFINITY, Finite::get);
            let hi = d.max.map_or(f64::INFINITY, Finite::get);
            if !(c[i] >= lo && c[i] <= hi) {
                out.push(format!("{} = {:.4} outside [{lo}, {hi}]", d.feature, c[i]));
            }
        }
    }
    if !out.is_empty() {
        return Err(Support {
            admitted: false,
            gate: SupportGate::Condition,
            reasons: out,
        });
    }
    // innovations: observed finite-difference rate against the population rate over the same span
    let mut innovation = |a: &Window, b: &Window| -> (f64, f64, f64, f64) {
        if !(b.t > a.t) {
            return (f64::NAN, 0.0, f64::NAN, f64::NAN);
        }
        let k_obs = (a.q - b.q) / (b.t - a.t);
        let k_mod = wt.k_mean(&c, q0, (a.t, a.q), (b.t, b.q), &mut scratch);
        let e = (k_obs.max(wt.k_floor) / k_mod)
            .ln()
            .clamp(-wt.e_lo, wt.e_hi);
        // frame noise of the finite difference, as a ratio to the modelled rate
        let noise = 2.0
            * b.q.sqrt()
            * wt.sigma_frame
            * (1.0 / a.n.max(1) as f64 + 1.0 / b.n.max(1) as f64).sqrt()
            / ((b.t - a.t) * k_mod);
        (k_obs, e, k_mod, noise)
    };
    let mid_used = mid.n > 0 && mid.t > head.t && tail.t > mid.t;
    let (early, late) = if mid_used {
        (innovation(&head, &mid), innovation(&mid, &tail))
    } else {
        let r = innovation(&head, &tail);
        (r, r)
    };
    let (k_early, e_early) = (early.0, early.1);
    let (k_late, e_late, k_model_late, r_noise) = late;
    let w_of = |q: f64| (1.0 - q / q0).clamp(0.0, 1.0);
    let w_tail = w_of(tail.q);
    let base = tail.t - head.t;
    let (gain_inputs, gain) = if base > 0.0 {
        let g = [
            base.ln(),
            (obs.len() as f64 / wt.n_ref).ln(),
            (w_tail / wt.w_ref).clamp(0.0, 1.0),
        ];
        let logit = wt.gain_b + g.iter().zip(wt.gain_w).map(|(a, b)| a * b).sum::<f64>();
        (g, sigmoid(logit))
    } else {
        ([0.0; N_GAIN], 0.0)
    };
    let b = gain * (wt.mix[0] * e_late + wt.mix[1] * e_early);
    let s_z = (wt.s_floor * wt.s_floor + (1.0 - gain) * wt.s_pop * wt.s_pop).sqrt();
    // State at the cutoff: the tail-window mean carried to the cutoff along the observed late
    // finite difference. No weight enters the level the forecast starts from.
    let local_slope = if k_late.is_finite() {
        k_late.clamp(0.0, wt.k_max)
    } else {
        0.0
    };
    let q_cut = tail.q - local_slope * (tau - tail.t);
    if !(q_cut > wt.q_min) {
        return Err(refuse(
            SupportGate::Horizon,
            format!(
                "diameter at the cutoff {:.3e} m is at or below the support floor {:.3e} m",
                q_cut.max(0.0).sqrt(),
                wt.q_min.sqrt()
            ),
        ));
    }
    if tau >= wt.t_valid {
        return Err(refuse(
            SupportGate::Horizon,
            format!("cutoff {tau} s >= T_valid {} s", wt.t_valid),
        ));
    }
    // condition-level laws and the state at the cutoff
    let laws = wt.laws(&c, &mut scratch);
    let (w_cut, s_cut) = (w_of(q_cut), tau * wt.k_ref / q0);
    let surv_d = (-wt.lam_d(&laws, w_cut)).exp();
    let surv_r = (-wt.lam_r(&laws, s_cut)).exp();
    let hot_r = laws.pi_r * surv_r;
    let hot_d = (1.0 - laws.pi_r) * surv_d;
    let p_hot = hot_r + hot_d;
    // a cool flame that started earlier is still alive when the droplet is larger than D_ce
    let cool_alive = if laws.p_cool > 0.0 && laws.sd_ce > 0.0 {
        normal_cdf((0.5 * q_cut.ln() - laws.d_ce_m.ln()) / laws.sd_ce)
    } else if laws.p_cool > 0.0 {
        f64::from(u8::from(q_cut.sqrt() > laws.d_ce_m))
    } else {
        0.0
    };
    let p_cool = laws.pi_r * (1.0 - surv_r) * laws.p_cool * cool_alive;
    let prior = [p_hot, p_cool, (1.0 - p_hot - p_cool).max(0.0)];
    let mut state = prior;
    let mut b_cool = 0.0;
    if gain > 0.0 && k_late.is_finite() && k_model_late.is_finite() && k_model_late > 0.0 {
        let r = k_late / k_model_late;
        let rho_c = laws.k_cool_m2_s / k_model_late;
        let rho_e = wt.evap_rate0(q_cut, 0.5) / k_model_late;
        let sd = |model: f64| (model * model + r_noise * r_noise).sqrt().max(1.0e-9);
        let like = [
            gauss(r, 1.0, sd(wt.s_pop)) + wt.l_floor,
            gauss(r, rho_c, sd(wt.s_pop * rho_c.max(1.0e-6))) + wt.l_floor,
            gauss(r, rho_e, sd(wt.sd_evap)) + wt.l_floor,
        ];
        let mut total = 0.0;
        for i in 0..3 {
            state[i] = prior[i] * like[i].powf(gain);
            total += state[i];
        }
        if total > 0.0 && total.is_finite() {
            for v in &mut state {
                *v /= total;
            }
        } else {
            state = prior;
        }
        if laws.k_cool_m2_s > 0.0 {
            b_cool = gain
                * (k_late.max(wt.k_floor) / laws.k_cool_m2_s)
                    .ln()
                    .clamp(-wt.e_lo, wt.e_hi);
        }
    }
    let class_r_given_hot = if p_hot > 0.0 {
        hot_r / p_hot
    } else {
        laws.pi_r
    };
    Ok(Encoded {
        c,
        n_valid: obs.len(),
        n_gated,
        q0_m2: q0,
        t_head_s: head.t,
        q_head_m2: head.q,
        t_tail_s: tail.t,
        q_tail_m2: tail.q,
        mid_window_used: mid_used,
        w_tail,
        k_obs_late_m2_s: k_late,
        k_obs_early_m2_s: k_early,
        k_model_late_m2_s: k_model_late,
        e_late,
        e_early,
        r_late: if k_model_late > 0.0 {
            k_late / k_model_late
        } else {
            f64::NAN
        },
        r_noise,
        gain_inputs,
        gain,
        b,
        b_cool,
        s_z,
        laws,
        prior,
        state,
        class_r_given_hot,
        q_cutoff_m2: q_cut,
        cutoff_s: tau,
    })
}

/// Quantiles of point masses sorted by value: the smallest value whose cumulative weight reaches the
/// level.
fn step_quantiles(atoms: &[(f64, f64)], total: f64, levels: &[f64], out: &mut [f64]) {
    for (o, p) in out.iter_mut().zip(levels) {
        let target = p * total;
        let mut cum = 0.0;
        let mut v = atoms.last().map_or(f64::NAN, |a| a.0);
        for a in atoms {
            cum += a.1;
            if cum >= target {
                v = a.0;
                break;
            }
        }
        *o = v;
    }
}

/// Quantiles of the mixture of Gaussians of sd `sigma` centred on the atoms (sorted by value): the
/// predictive distribution of one measurement. Atoms are pooled on a grid of `sigma / 4` (mass and
/// mean kept); each quantile is found by bisection of the mixture CDF, which is monotone. This
/// inverts the predictive distribution; no parameter is involved.
fn mixture_quantiles(
    atoms: &[(f64, f64)],
    total: f64,
    sigma: f64,
    levels: &[f64],
    grid: &mut Vec<(f64, f64)>,
    out: &mut [f64],
) {
    if !(sigma > 0.0) || atoms.is_empty() {
        return step_quantiles(atoms, total, levels, out);
    }
    grid.clear();
    let step = 0.25 * sigma;
    let v0 = atoms[0].0;
    let mut cell_now = i64::MIN;
    for &(v, w) in atoms {
        let cell = ((v - v0) / step).floor() as i64;
        if cell == cell_now {
            let last = grid.last_mut().expect("cell");
            last.0 += w * v;
            last.1 += w;
        } else {
            grid.push((w * v, w));
            cell_now = cell;
        }
    }
    for g in grid.iter_mut() {
        g.0 /= g.1;
    }
    let cdf = |x: f64| -> f64 {
        let mut acc = 0.0;
        for &(v, w) in grid.iter() {
            let z = (x - v) / sigma;
            if z >= 8.0 {
                acc += w;
            } else if z > -8.0 {
                acc += w * normal_cdf(z);
            } else {
                break;
            }
        }
        acc / total
    };
    let (lo0, hi0) = (
        grid[0].0 - 8.0 * sigma,
        grid[grid.len() - 1].0 + 8.0 * sigma,
    );
    for (o, p) in out.iter_mut().zip(levels) {
        let (mut lo, mut hi) = (lo0, hi0);
        for _ in 0..44 {
            let mid = 0.5 * (lo + hi);
            if cdf(mid) < *p {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        *o = 0.5 * (lo + hi);
    }
}

/// The numbers of one forecast, before they are wrapped in an artifact.
#[derive(Debug, Clone)]
pub struct Forecast {
    pub encoded: Encoded,
    pub times_s: Vec<f64>,
    pub levels: Vec<f64>,
    /// `[level][time]`, measurement band.
    pub quantiles: Vec<Vec<f64>>,
    /// `[level][time]`, latent mixture before the measurement noise.
    pub latent_quantiles: Vec<Vec<f64>>,
    pub mean: Vec<f64>,
    /// Rows `active_supported, hot_phase_end, disruption, unresolved`.
    pub events: [Vec<f64>; 4],
    /// Probability of (hot, cool, no flame) at each forecast time.
    pub state_probability: [Vec<f64>; 3],
    pub partition_max_error: f64,
    pub rk4_steps: usize,
    pub max_atoms: usize,
}

/// An extinction-time bin of one hot path: mass (conditional on a hot flame at the cutoff, within
/// the class) and the mass-weighted time and D^2 of the extinction.
#[derive(Clone, Copy, Default)]
struct Bin {
    mass: f64,
    t: f64,
    q: f64,
}

/// RK4 of the hot phase on a uniform grid, then the mixture over states at each forecast time.
fn integrate(wt: &Weights<'_>, enc: &Encoded, times: &[f64], levels: &[f64]) -> Forecast {
    let nt = times.len();
    let nz = wt.z_count;
    let tau = enc.cutoff_s;
    let c = &enc.c;
    let q0 = enc.q0_m2;
    let laws = &enc.laws;
    let t_end = times.iter().copied().fold(tau, f64::max);
    let n_grid = (((t_end - tau) / wt.dt) - 1.0e-9).ceil().max(1.0) as usize;
    let h = wt.dt;
    let per_bin = ((wt.bin_s / h).round() as usize).max(1);
    let n_bins = n_grid.div_ceil(per_bin);
    // scenario nodes
    let nodes = |n: usize, step: f64| -> (Vec<f64>, Vec<f64>) {
        let z: Vec<f64> = (0..n)
            .map(|k| (k as f64 - 0.5 * (n - 1) as f64) * step)
            .collect();
        let mut om: Vec<f64> = z.iter().map(|v| (-0.5 * v * v).exp()).collect();
        let total: f64 = om.iter().sum();
        om.iter_mut().for_each(|w| *w /= total);
        (z, om)
    };
    let (z, om) = nodes(nz, wt.z_step);
    let (zc, omc) = nodes(wt.ce_count, wt.ce_step);
    let q_ce: Vec<f64> = zc
        .iter()
        .map(|u| {
            let d = laws.d_ce_m * (laws.sd_ce * u).exp();
            d * d
        })
        .collect();
    let w_of = |q: f64| (1.0 - q / q0).clamp(0.0, 1.0);
    let s_of = |t: f64| t * wt.k_ref / q0;
    let surv_d0 = (-wt.lam_d(laws, w_of(enc.q_cutoff_m2))).exp().max(1.0e-300);
    let surv_r0 = (-wt.lam_r(laws, s_of(tau))).exp().max(1.0e-300);
    // hot paths: q and the conditional survival of both classes on the grid
    let mut q_grid = vec![0.0; nz * (n_grid + 1)];
    let mut sd_grid = vec![0.0; nz * (n_grid + 1)];
    let mut sr_grid = vec![0.0; n_grid + 1];
    let mut bins_d = vec![Bin::default(); nz * n_bins];
    let mut bins_r = vec![Bin::default(); nz * n_bins];
    for (i, s) in sr_grid.iter_mut().enumerate() {
        *s = (-wt.lam_r(laws, s_of(tau + i as f64 * h))).exp() / surv_r0;
    }
    let mut scratch = [0.0; H_RATE];
    let mut rk4_steps = 0usize;
    for k in 0..nz {
        let shift = enc.b + enc.s_z * z[k];
        let mut deriv = |q: f64, t: f64| -> f64 {
            let x = [c[0], c[1], c[2], c[3], c[4], w_of(q), s_of(t)];
            -wt.k_ref * (wt.rate.eval(&x, &mut scratch) + shift).exp()
        };
        let row = k * (n_grid + 1);
        let mut q = enc.q_cutoff_m2;
        q_grid[row] = q;
        sd_grid[row] = 1.0;
        let mut floor = false;
        for i in 0..n_grid {
            let t = tau + i as f64 * h;
            let q_new = if floor {
                q
            } else {
                let a1 = deriv(q, t);
                let a2 = deriv(q + 0.5 * h * a1, t + 0.5 * h);
                let a3 = deriv(q + 0.5 * h * a2, t + 0.5 * h);
                let a4 = deriv(q + h * a3, t + h);
                rk4_steps += 1;
                q + h / 6.0 * (a1 + 2.0 * a2 + 2.0 * a3 + a4)
            };
            let q_new = if q_new <= wt.q_min {
                floor = true;
                wt.q_min
            } else {
                q_new
            };
            let sd_new = (-wt.lam_d(laws, w_of(q_new))).exp() / surv_d0;
            let (t_mid, q_mid) = (t + 0.5 * h, 0.5 * (q + q_new));
            let bi = k * n_bins + i / per_bin;
            let md = (sd_grid[row + i] - sd_new).max(0.0);
            bins_d[bi].mass += md;
            bins_d[bi].t += md * t_mid;
            bins_d[bi].q += md * q_mid;
            let mr = (sr_grid[i] - sr_grid[i + 1]).max(0.0);
            bins_r[bi].mass += mr;
            bins_r[bi].t += mr * t_mid;
            bins_r[bi].q += mr * q_mid;
            q = q_new;
            q_grid[row + i + 1] = q;
            sd_grid[row + i + 1] = sd_new.min(sd_grid[row + i]);
        }
    }
    let finish = |bins: &mut [Bin]| {
        for b in bins.iter_mut().filter(|b| b.mass > 0.0) {
            b.t /= b.mass;
            b.q /= b.mass;
        }
    };
    finish(&mut bins_d);
    finish(&mut bins_r);
    let cool_scale = wt.cool.as_ref().map_or(1.0, |c| c.post_scale);
    // q of a droplet that lost its hot flame at (t_e, q_e) and has no flame since
    let evap = |t: f64, t_e: f64, q_e: f64, scale: f64| -> f64 {
        (q_e - wt.evap_drop((t - t_e).max(0.0), q_e, scale)).max(wt.q_min)
    };
    // q of a droplet whose cool flame started at (t_e, q_e) and burns at k_c until q_x
    let cool = |t: f64, t_e: f64, q_e: f64, k_c: f64, q_x: f64| -> (f64, bool) {
        if q_x >= q_e || !(k_c > 0.0) {
            return (evap(t, t_e, q_e, cool_scale), false);
        }
        let t_x = t_e + (q_e - q_x) / k_c;
        if t <= t_x {
            ((q_e - k_c * (t - t_e)).max(wt.q_min), true)
        } else {
            (evap(t, t_x, q_x, cool_scale), false)
        }
    };
    let [p_hot, p_cool, p_evap] = enc.state;
    let (cls_r, cls_d) = (enc.class_r_given_hot, 1.0 - enc.class_r_given_hot);
    // evaporation of a droplet already without a flame at the cutoff: the observed late rate is the
    // best evidence of how warm its surroundings still are
    let evap_now_scale = if wt.k_hot > 0.0 && enc.k_obs_late_m2_s.is_finite() {
        let rel = enc.q_cutoff_m2 / (wt.d_ref * wt.d_ref);
        let seen =
            ((enc.k_obs_late_m2_s - wt.k_inf * rel.powf(wt.size_exp)) / wt.k_hot).clamp(0.0, 3.0);
        enc.gain * seen + (1.0 - enc.gain) * 0.5
    } else {
        0.5
    };
    let nl = levels.len();
    let mut quantiles = vec![vec![0.0; nt]; nl];
    let mut latent = vec![vec![0.0; nt]; nl];
    let mut mean = vec![0.0; nt];
    let mut events: [Vec<f64>; 4] = std::array::from_fn(|_| vec![0.0; nt]);
    let mut state_probability: [Vec<f64>; 3] = std::array::from_fn(|_| vec![0.0; nt]);
    let mut atoms: Vec<(f64, f64)> = Vec::new();
    let mut qe = vec![0.0; nl];
    let mut grid: Vec<(f64, f64)> = Vec::new();
    let mut worst = 0.0_f64;
    let mut max_atoms = 0usize;
    const TINY: f64 = 1.0e-12;
    for (j, &tj) in times.iter().enumerate() {
        atoms.clear();
        let (mut hot, mut cool_p) = (0.0, 0.0);
        let pos = ((tj - tau) / h).clamp(0.0, n_grid as f64);
        let i0 = (pos.floor() as usize).min(n_grid.saturating_sub(1));
        let frac = pos - i0 as f64;
        let lerp = |g: &[f64], row: usize| g[row + i0] + frac * (g[row + i0 + 1] - g[row + i0]);
        let sr_now = sr_grid[i0] + frac * (sr_grid[i0 + 1] - sr_grid[i0]);
        let full_bins = i0 / per_bin; // bins that ended at or before grid node i0
        let part_start = full_bins * per_bin;
        if p_hot > TINY {
            for k in 0..nz {
                let row = k * (n_grid + 1);
                let q_now = lerp(&q_grid, row);
                let sd_now = lerp(&sd_grid, row);
                let rate_factor = (enc.b + enc.s_z * z[k]).exp();
                let w_k = p_hot * om[k];
                // still inside the hot flame
                let alive = cls_d * sd_now + cls_r * sr_now;
                if w_k * alive > TINY {
                    atoms.push((q_now.sqrt(), w_k * alive));
                    hot += w_k * alive;
                }
                let emit =
                    |b: Bin, radiative: bool, atoms: &mut Vec<(f64, f64)>, cool_p: &mut f64| {
                        let m = w_k * b.mass * if radiative { cls_r } else { cls_d };
                        if !(m > TINY) {
                            return;
                        }
                        let pc = if radiative { laws.p_cool } else { 0.0 };
                        if (1.0 - pc) * m > TINY {
                            atoms.push((evap(tj, b.t, b.q, 1.0).sqrt(), (1.0 - pc) * m));
                        }
                        if pc * m > TINY {
                            let k_c = laws.k_cool_m2_s * rate_factor;
                            for (u, &qx) in q_ce.iter().enumerate() {
                                let (q, burning) = cool(tj, b.t, b.q, k_c, qx);
                                atoms.push((q.sqrt(), pc * m * omc[u]));
                                if burning {
                                    *cool_p += pc * m * omc[u];
                                }
                            }
                        }
                    };
                for bi in 0..full_bins.min(n_bins) {
                    emit(bins_d[k * n_bins + bi], false, &mut atoms, &mut cool_p);
                    emit(bins_r[k * n_bins + bi], true, &mut atoms, &mut cool_p);
                }
                // the bin in progress: extinction between its start and now
                if full_bins < n_bins && pos > part_start as f64 {
                    let (q_s, t_s) = (q_grid[row + part_start], tau + part_start as f64 * h);
                    let mid = Bin {
                        mass: 0.0,
                        t: 0.5 * (t_s + tj),
                        q: 0.5 * (q_s + q_now),
                    };
                    emit(
                        Bin {
                            mass: (sd_grid[row + part_start] - sd_now).max(0.0),
                            ..mid
                        },
                        false,
                        &mut atoms,
                        &mut cool_p,
                    );
                    emit(
                        Bin {
                            mass: (sr_grid[part_start] - sr_now).max(0.0),
                            ..mid
                        },
                        true,
                        &mut atoms,
                        &mut cool_p,
                    );
                }
            }
        }
        if p_cool > TINY {
            // nodes of D_ce compatible with a cool flame alive at the cutoff
            let ok: f64 = q_ce
                .iter()
                .zip(&omc)
                .filter(|(qx, _)| **qx < enc.q_cutoff_m2)
                .map(|(_, w)| *w)
                .sum();
            for k in 0..nz {
                let k_c = laws.k_cool_m2_s * (enc.b_cool + enc.s_z * z[k]).exp();
                if ok > 0.0 {
                    for (u, &qx) in q_ce.iter().enumerate() {
                        if qx < enc.q_cutoff_m2 {
                            let (q, burning) = cool(tj, tau, enc.q_cutoff_m2, k_c, qx);
                            let m = p_cool * om[k] * omc[u] / ok;
                            atoms.push((q.sqrt(), m));
                            if burning {
                                cool_p += m;
                            }
                        }
                    }
                } else {
                    atoms.push((
                        evap(tj, tau, enc.q_cutoff_m2, cool_scale).sqrt(),
                        p_cool * om[k],
                    ));
                }
            }
        }
        if p_evap > TINY {
            atoms.push((
                evap(tj, tau, enc.q_cutoff_m2, evap_now_scale).sqrt(),
                p_evap,
            ));
        }
        max_atoms = max_atoms.max(atoms.len());
        let total: f64 = atoms.iter().map(|a| a.1).sum();
        worst = worst.max((total - 1.0).abs());
        mean[j] = atoms.iter().map(|a| a.0 * a.1).sum::<f64>() / total;
        atoms.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        step_quantiles(&atoms, total, levels, &mut qe);
        for l in 0..nl {
            latent[l][j] = qe[l];
        }
        mixture_quantiles(&atoms, total, wt.sigma_d, levels, &mut grid, &mut qe);
        for l in 0..nl {
            quantiles[l][j] = qe[l].max(0.0);
        }
        state_probability[0][j] = hot / total;
        state_probability[1][j] = cool_p / total;
        state_probability[2][j] = (1.0 - (hot + cool_p) / total).max(0.0);
    }
    // Events of a droplet burning hot at the cutoff, as competing risks: the hot flame ends
    // (extinction of either kind), or the droplet disrupts while still burning hot. With H the
    // probability of a hot flame ignoring disruption, active = H exp(-lam (t - cutoff)), the
    // disruption incidence is the integral of lam * active, and the rest is the end of the hot
    // phase. The three rows sum to one by construction; a droplet already out at the cutoff counts
    // as ended.
    let mut cum_dis = 0.0;
    let mut prev: Option<(f64, f64)> = None;
    let mut ended = 0.0_f64;
    for (j, &tj) in times.iter().enumerate() {
        let active = state_probability[0][j] * (-wt.lam_dis * (tj - tau)).exp();
        if let Some((tp, ap)) = prev {
            cum_dis += 0.5 * wt.lam_dis * (ap + active) * (tj - tp);
        } else {
            cum_dis = state_probability[0][j] - active;
        }
        prev = Some((tj, active));
        ended = ended.max(1.0 - active - cum_dis).clamp(0.0, 1.0);
        let dis = cum_dis.min(1.0 - ended);
        events[1][j] = ended;
        events[2][j] = dis;
        events[3][j] = 0.0;
        events[0][j] = (1.0 - ended - dis).max(0.0);
    }
    Forecast {
        encoded: enc.clone(),
        times_s: times.to_vec(),
        levels: levels.to_vec(),
        quantiles,
        latent_quantiles: latent,
        mean,
        events,
        state_probability,
        partition_max_error: worst,
        rk4_steps,
        max_atoms,
    }
}

/// Strict forward of a droplet release on an admitted prefix: `Ok(Err(support))` is a scientific
/// refusal, `Err(refusal)` a release or request that cannot be run at all.
pub fn forecast(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<Result<Forecast, Support>, Refusal> {
    let fuel = Fuel::of_model(&spec.model_name).ok_or_else(|| {
        Refusal::unsupported(
            "no_literal_forward",
            format!("{} is not a droplet model", spec.model_name),
        )
    })?;
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params, fuel)?;
    let built_with = release
        .release
        .construction
        .as_ref()
        .map(|m| &m.forward_hash);
    let this = crate::audit::forward_hash_of(fuel.model_name());
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
    if let Some(f) = prefix.known_conditions.get("fuel") {
        let raw = match &f.raw_value {
            Some(ignis_core::contracts::RawValue::Text(s)) => s.trim().to_string(),
            Some(ignis_core::contracts::RawValue::Number(x)) => x.to_string(),
            None => String::new(),
        };
        if !raw.eq_ignore_ascii_case(fuel.label()) {
            return Ok(Err(refuse(
                SupportGate::Semantic,
                format!(
                    "fuel {raw:?} outside the release population {:?}",
                    fuel.label()
                ),
            )));
        }
    }
    if let Some(tor) = release
        .release
        .applicability
        .get("time_origin")
        .and_then(Value::as_str)
    {
        if prefix.time_origin != tor {
            return Ok(Err(refuse(
                SupportGate::Semantic,
                format!(
                    "time origin {:?} differs from the release's {tor:?}",
                    prefix.time_origin
                ),
            )));
        }
    }
    let enc = match encode(&wt, spec, prefix) {
        Ok(e) => e,
        Err(s) => return Ok(Err(s)),
    };
    let levels = common::levels_of(settings);
    let times = common::times_of(settings, enc.cutoff_s, 10.0, wt.t_valid);
    let bad: Vec<f64> = times
        .iter()
        .copied()
        .filter(|t| !(*t >= enc.cutoff_s && *t <= wt.t_valid))
        .collect();
    if !bad.is_empty() || times.is_empty() {
        return Ok(Err(refuse(
            SupportGate::Horizon,
            format!(
                "requested times {bad:?} outside [cutoff, T_valid = {}]",
                wt.t_valid
            ),
        )));
    }
    Ok(Ok(integrate(&wt, &enc, &times, &levels)))
}

fn event_categories() -> Vec<String> {
    let mut v = vec![EVENT_ACTIVE.to_string()];
    v.extend(FLEX_CAUSES.iter().map(|s| s.to_string()));
    v.push(EVENT_UNRESOLVED.into());
    v
}

fn body(f: Forecast) -> Body {
    let diagnostics = json!({
        "encoder": f.encoded,
        "band": "measurement",
        "band_meaning": "unconditional predictive of the measured liquid diameter: mixture over rate scenarios, over the droplet's state at the cutoff and over the times of later transitions",
        "latent_quantiles": f.latent_quantiles,
        "state_probability": {"hot": f.state_probability[0], "cool": f.state_probability[1], "no_flame": f.state_probability[2]},
        "uncertainty_kind": "assumed_noise",
        "partition_max_error": f.partition_max_error,
        "rk4_steps": f.rk4_steps,
        "max_atoms": f.max_atoms,
    });
    Body {
        kind: None,
        quantity: Some(Quantity::LiquidDiameter),
        times_s: f.times_s,
        quantile_levels: f.levels,
        quantiles: f.quantiles,
        mean: f.mean,
        event_categories: event_categories(),
        event_cdf: f.events.to_vec(),
        diagnostics,
    }
}

/// The artifact of a forecast or of a scientific refusal (target quantity kept, no numbers).
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
                quantity: Some(Quantity::LiquidDiameter),
                ..Body::default()
            },
        ),
    })
}

/// A droplet model behind the common expert interface.
pub struct DropletWsp {
    fuel: Fuel,
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl DropletWsp {
    pub fn new(fuel: Fuel) -> Self {
        DropletWsp {
            fuel,
            spec: spec(fuel),
            // Identical to the fitted FLEX experts' policy: the prefix hash, and so the pairing of
            // evaluation cases with the baselines, depends on the allowed set.
            policy: ProspectivePolicy::new(
                Family::Flex,
                &["x_o2", "x_co2", "x_he", "pressure", "fuel"],
                &[],
            ),
        }
    }
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }
    pub fn fuel(&self) -> Fuel {
        self.fuel
    }
}

impl Expert for DropletWsp {
    fn family(&self) -> Family {
        Family::Flex
    }
    fn model_name(&self) -> &str {
        self.fuel.model_name()
    }
    fn is_baseline(&self) -> bool {
        false
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }
    fn default_config(&self) -> ExpertResult<Value> {
        Ok(json!({"construction": "literal", "spec_id": self.fuel.spec_id()}))
    }
    fn estimate_hours(&self, _: &Value, _: usize, _: usize, _: &Profile) -> f64 {
        0.0
    }
    fn fit(&self, _: &FitData<'_>, _: &Value, _: &mut FitContext) -> ExpertResult<ModelRelease> {
        bail!("{} has no fitting procedure: its release is assembled from a literal patch (`ignis wsp assemble`)", self.fuel.model_name())
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
            "{} declares an assumed scenario spread, not a likelihood",
            self.fuel.model_name()
        )
    }
}

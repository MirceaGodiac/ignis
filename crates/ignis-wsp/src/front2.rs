//! `bass_front.wsp2` — where the flame on a thin fuel strip will be, in a low-speed concurrent flow
//! whose history is a known control. The second front forward: `front.rs` with four additions, each
//! switched off by its own entry. With all four off it returns what `front.rs` returns for the same
//! numbers and inputs, bit for bit (`tests/front2_forward.rs`).
//!
//! The flame of a thin fuel in concurrent flow is a band that travels with the flow. Three places
//! on the sample describe it, and each answers the flow on its own time scale:
//!
//! ```text
//! x      the pyrolysis front: where the fuel starts to burn. It advances at the spread rate,
//!        dx/dt = E v(U, X_O2, W), and follows a change of flow at once.
//! tip    the downstream end of the visible flame: tip = x + h, where the overhang h relaxes to
//!        h_ss(U, X_O2, W) with the time `over.tau_s` (gas phase, a second or two). A step down in
//!        flow pulls the tip back while x keeps advancing.
//! base   the upstream end, where the fuel has burned out: base(t) = x(t - tau_b). The base repeats
//!        what the front did one burn-through time earlier, so after a step down it keeps the old
//!        speed for tau_b seconds and the flame shortens from both ends.
//!
//! features   u = U / u_ref - 1,  c_o2 = (X_O2 - o2_ref) / o2_scale,  c_w = ln(W / W_ref)
//! laws       N(c) = b0 + skip . c + sum_j w2[j] relu(b1[j] + W1[j, :] . c)
//!            v     = max(N_speed, v_min) (X_O2 / o2_ref)^speed.pow[0] (W / W_ref)^speed.pow[1]
//!            h_ss  = max(N_over, 0)      (X_O2 / o2_ref)^over.pow[0]  (W / W_ref)^over.pow[1]
//!            tau_b = exp(N_burn),   quench hazard = exp(N_quench)
//! ```
//!
//! Lengths are in sample widths: the sample is its own ruler in the top view, and v and h are
//! written in reference sample widths (the 2 cm class) with a power of the width ratio for other
//! samples. Speeds are in widths per second, the flow in cm/s.
//!
//! The four additions.
//!
//! ```text
//! A  two oxygen powers (speed.split, speed.pow_i). The part of the spread rate that remains with
//!    the fan stopped answers oxygen on its own. With o = X_O2 / o2_ref, N = max(N_speed(c), v_min)
//!    and N0 the same at zero flow speed (the flow feature at the value it has for U = 0, that is
//!    u_min / u_ref - 1; the oxygen and width features unchanged):
//!        v_split = max(N0 o^speed.pow_i + (N - N0) o^speed.pow[0], v_min o^speed.pow_i)
//!                  (W / W_ref)^speed.pow[1]
//!        v       = (1 - speed.split) v_first + speed.split v_split
//!    speed.split = 0 is the expression of the first forward, 1 the split. The floor acts only
//!    when the network falls with the flow (N < N0): the flow part may not take the rate below the
//!    smallest zero-flow rate.
//! B  slow growth, fast shrinking of the overhang (over.tau_up_s). Within an integration step the
//!    target h_ss is constant and the overhang relaxes exactly, h <- h_ss + (h - h_ss)
//!    exp(-dt / tau), so it never crosses its target inside a step and the sign of (h_ss - h) at
//!    the start of the step holds for the whole step. With over.tau_up_s > 0, tau = over.tau_up_s
//!    in a step that starts below its target (growing) and over.tau_s otherwise (shrinking). With
//!    over.tau_up_s = 0, tau = over.tau_s in both directions.
//! C  slow-down downstream (speed.kappa). During the forecast integration the spread-rate law and
//!    the overhang target see U_eff = U max(0, 1 - speed.kappa (x - x_ref)): x is the forecast
//!    front, x_end + E (X(t) - X(t_end)) with X the law path, and x_ref its value at the start of
//!    the forecast integration, the cutoff (igniter-off for a forecast without stills). So
//!    x - x_ref = E (X - X(t_ref)). The burn-through time and the quench hazard keep U. The law
//!    path is integrated to the step that contains t_ref, the prefix is read (E), and the rest is
//!    integrated with the factor evaluated at the start of each step (explicit Euler in X, as the
//!    front itself); the step that contains t_ref runs without it.
//! D  flow paths (unc.flow_abs_cm_s, unc.flow_rel, unc.flow_w). The flow speed assigned to a fan
//!    setting is uncertain. With a positive outer weight the forecast is the mixture of whole runs
//!    of the forward under U_k(t) = max(0, U(t) (1 + s_k unc.flow_rel) + s_k unc.flow_abs_cm_s),
//!    s = (-1, 0, +1), weights unc.flow_w over their sum; each run reads the prefix under its own
//!    flow history, a path with weight zero is not run. Tip quantiles: of the mixture of the
//!    paths' Gaussians, each by halving the bracket of the monotone mixture CDF a fixed number of
//!    times (the arithmetic of the droplet forwards; no parameter is involved), then cut at the end
//!    of the sample. Base: the weighted median of the paths' bases. Quenched: the weighted mean.
//!    Reached the end: among the paths not quenched, sum w (1 - q_k) e_k / sum w (1 - q_k), so
//!    that the event rows are the weighted means of the paths' rows. With no positive outer weight
//!    exactly one run is made, on the flow as read, and returned as it is.
//! ```
//!
//! What a prefix adds. The run offset E is read from how far the front moved in the prefix against
//! how far the law says it should have moved over the same seconds under the same flow history:
//! from the tip (corrected for the modelled overhang) and from the base (the law delayed by tau_b).
//! It is trusted by a written gain that grows with the length of the prefix and with the time since
//! the last change of flow; the ignition transient says little. The position at the cutoff is the
//! measured tip of the last stills.
//!
//! What the forecast is. Conditional on the flow history up to each forecast time (a known control:
//! the operator's steps are an input, not something the model predicts): quantiles of the tip
//! position while a flame exists, the probability that the flame has been quenched (a hazard
//! written against flow and oxygen), and the probability that the tip has reached the end of the
//! sample. Spread is assumed, not calibrated: a relative spread of the run offset, the measurement
//! noise of the tip, a share of the modelled overhang change, a slow drift, and, with flow paths,
//! the spread between the paths.
//!
//! The shared contracts give a front position in metres. A prefix that carries the sample width in
//! metres is forecast in metres; the track files of the measurement study, which have no length
//! calibration, are forecast in sample widths through [`run`] and never enter the shared records.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    AnalysisArtifact, Family, FeatureSpec, ForwardContract, ForwardProfile, ModelRelease,
    ModelSpec, PrefixBundle, QualificationState, Quantity, SpecId, Support, SupportGate,
    TargetSpec, TensorRole, WspSchema,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::LoadedRelease;
use ignis_core::resources::Profile;
use ignis_core::views::View;

use crate::common::{self, admitted, refuse, Body};
use crate::nn::{hinge_net, sigmoid};
use crate::quad::{normal_cdf, normal_quantile};
use crate::spec::{authored, constant, sparse, Params};

pub const SPEC_ID: &str = "bass_front-wsp-v2";
pub const MODEL_NAME: &str = "bass_front.wsp2";
pub const FORWARD_VERSION: &str = "2";
/// Hidden width of each law network.
pub const HIDDEN: usize = 6;
/// Law inputs: `u, c_o2, c_w`.
pub const N_IN: usize = 3;
pub const FEATURES: [&str; N_IN] = ["u", "c_o2", "c_w"];
pub const CATEGORIES: [&str; 4] = [
    "active_supported",
    "extinction",
    "reached_end",
    "unresolved_support",
];

/// The frozen specification.
pub fn spec() -> ModelSpec {
    use TensorRole::*;
    let feature = |name: &str, unit: &str, source: &str, transform: &str| FeatureSpec {
        name: name.into(),
        unit: unit.into(),
        source: source.into(),
        transform: transform.into(),
    };
    let (h, n) = (HIDDEN as u32, N_IN as u32);
    let mut tensors = Vec::new();
    for (p, what, role, unit, range) in [
        (
            "speed",
            "the spread rate of the pyrolysis front, reference sample widths per second",
            PhysicalParameter,
            "1/s",
            (-1.0, 5.0),
        ),
        (
            "over",
            "the steady overhang of the visible flame tip beyond the pyrolysis front, reference sample widths",
            PhysicalParameter,
            "1",
            (-5.0, 20.0),
        ),
        (
            "burn",
            "the burn-through time ln(tau_b / 1 s): how long a place on the sample burns",
            PhysicalParameter,
            "ln(s)",
            (-3.0, 8.0),
        ),
        (
            "quench",
            "the quench hazard ln(rate / 1 per s) of a flame at this flow and oxygen",
            HazardParameter,
            "ln(1/s)",
            (-40.0, 5.0),
        ),
    ] {
        tensors.extend([
            sparse(
                &format!("{p}.W1"),
                &[h, n],
                Weight,
                None,
                &format!("Hidden weights of {what}; row j reads (u, c_o2, c_w)."),
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
                &format!("Linear part of {what} in (u, c_o2, c_w); every element is stated."),
            ),
            authored(
                &format!("{p}.b0"),
                &[],
                role,
                Some(unit),
                Some(range),
                &format!("Value of {what} at u = c_o2 = c_w = 0 before hinge terms."),
            ),
        ]);
    }
    tensors.extend([
        authored("speed.pow", &[2], PhysicalParameter, Some("1"), Some((-5.0, 5.0)),
                 "Powers of (X_O2 / o2_ref) and of (W / W_ref) that multiply the spread rate in sample widths per second (a sample half as wide with the same speed in mm/s has the width power -1)."),
        authored("over.pow", &[2], PhysicalParameter, Some("1"), Some((-5.0, 5.0)),
                 "Powers of (X_O2 / o2_ref) and of (W / W_ref) that multiply the overhang in sample widths."),
        authored("over.tau_s", &[], PhysicalParameter, Some("s"), Some((0.05, 60.0)),
                 "Time over which the overhang relaxes to its steady value after a change of flow."),
        authored("ign.gain", &[], PhysicalParameter, Some("1"), Some((0.0, 5.0)),
                 "Spread rate while the igniter is on, as a multiple of the law (the hot wire pilots the flame)."),
        authored("ign.delay_s", &[], PhysicalParameter, Some("s"), Some((0.0, 30.0)),
                 "Time after the igniter is switched on before the front starts to move."),
        authored("end.linger_s", &[], PhysicalParameter, Some("s"), Some((0.0, 60.0)),
                 "Time the flame is still seen after its tip has reached the end of the sample."),
        authored("enc.gain_w", &[2], Weight, None, Some((-20.0, 20.0)),
                 "Weights of the gain logit on ln(span / 10 s) and on the settled share min(1, seconds since the last change of flow / 10)."),
        authored("enc.gain_b", &[], Bias, None, Some((-20.0, 20.0)),
                 "Bias of the gain logit: gain = sigmoid(gain_b + gain_w . inputs)."),
        authored("enc.mix", &[2], Weight, Some("1"), Some((0.0, 1.0)),
                 "Weights of the tip-based and base-based offsets in the observed run offset (normalised over those available)."),
        authored("enc.e_clip", &[2], PhysicalParameter, Some("1"), Some((0.0, 3.0)),
                 "Largest run offset carried, below and above: E lies in [exp(-e_clip[0]), exp(e_clip[1])]."),
        authored("unc.s_pop", &[], NoiseParameter, Some("1"), Some((0.0, 3.0)),
                 "Relative sd of the spread rate of a run before its prefix says anything."),
        authored("unc.s_floor", &[], NoiseParameter, Some("1"), Some((0.0, 3.0)),
                 "Relative sd of the spread rate that remains when the prefix is fully trusted."),
        authored("unc.s_far", &[2], NoiseParameter, Some("1"), Some((0.0, 3.0)),
                 "Relative sd added (in quadrature) per unit of |c_o2| and of |c_w|: the laws are known less well away from the reference oxygen and sample."),
        authored("unc.sigma_w", &[], NoiseParameter, Some("1"), Some((0.0, 2.0)),
                 "sd of the tip position at the cutoff, sample widths (measurement and the still-to-overlay clock)."),
        authored("unc.sigma_x0_w", &[], NoiseParameter, Some("1"), Some((0.0, 5.0)),
                 "sd of the tip position when no still exists before the cutoff, sample widths."),
        authored("unc.rel_over", &[], NoiseParameter, Some("1"), Some((0.0, 3.0)),
                 "sd of a change of the overhang, as a share of the modelled change."),
        authored("unc.drift_w", &[], NoiseParameter, Some("s^-1/2"), Some((0.0, 1.0)),
                 "Slow drift of the tip: sd in sample widths per square root of a second of lead."),
        // the entries of the second front forward; each addition is off at the value named last
        authored("speed.split", &[], Weight, Some("1"), Some((0.0, 1.0)),
                 "Share of the spread rate given by the split oxygen law, in which the part of the rate that remains at zero flow carries the oxygen power speed.pow_i and only the flow-dependent part carries speed.pow[0]: 1, the split law; between, the weighted mean of the split law and the law of the first front forward; 0: the law of the first front forward, one oxygen power for the whole rate."),
        authored("speed.pow_i", &[], PhysicalParameter, Some("1"), Some((0.0, 6.0)),
                 "Power of (X_O2 / o2_ref) that multiplies the zero-flow part of the spread rate (the rate the law gives with the fan stopped) in the split oxygen law; not read when speed.split is 0."),
        authored("speed.kappa", &[], PhysicalParameter, Some("1"), Some((0.0, 1.0)),
                 "Slow-down downstream, per sample width: the flow speed that the spread-rate law and the overhang target see falls by this share of itself for every sample width the front has advanced since the start of the forecast integration, U_eff = U max(0, 1 - kappa (x - x_ref)); 0: no slow-down."),
        authored("over.tau_up_s", &[], PhysicalParameter, Some("s"), Some((0.0, 120.0)),
                 "Time over which the overhang grows towards a larger steady value; over.tau_s is then the time over which it shrinks towards a smaller one; 0: over.tau_s in both directions."),
        authored("unc.flow_abs_cm_s", &[], NoiseParameter, Some("cm/s"), Some((0.0, 5.0)),
                 "Absolute uncertainty of the flow speed assigned to a fan setting: the lower and the higher flow path shift the flow history by this much."),
        authored("unc.flow_rel", &[], NoiseParameter, Some("1"), Some((0.0, 1.0)),
                 "Relative uncertainty of the flow speed assigned to a fan setting: the lower and the higher flow path scale the flow history by 1 - flow_rel and 1 + flow_rel before the shift."),
        authored("unc.flow_w", &[3], Weight, Some("1"), Some((0.0, 1000.0)),
                 "Weights of the three flow paths (lower, as read, higher), normalised by their sum; a path with weight zero is not run; [0, 1, 0], or no positive outer weight: one run on the flow as read."),
        constant("norm.u_ref_cm_s", 5.0, "cm/s", "Reference flow speed of u."),
        constant("norm.u_min_cm_s", 0.1, "cm/s", "Smallest flow speed the laws are evaluated at."),
        constant("norm.o2_ref", 0.19, "1", "Reference oxygen mole fraction of c_o2 and of the oxygen powers."),
        constant("norm.o2_scale", 0.02, "1", "Scale of c_o2."),
        constant("norm.w_ref_m", 0.022, "m", "Width of the reference sample (the 2 cm class), used only when a prefix gives a width in metres."),
        constant("norm.v_min", 0.001, "1/s", "Smallest spread rate, sample widths per second."),
        constant("enc.end_s", 4.0, "s", "Length of the first and of the last window of stills in the prefix."),
        constant("enc.min_span_s", 3.0, "s", "Shortest time between the two windows for a run offset to be read."),
        constant("enc.change_cm_s", 0.8, "cm/s", "Smallest difference from the flow at the last window counted as a change of flow: the flow last changed at the start of the first entry after the most recent entry that differs from it by at least this much (a one-unit fan flicker is not a change, a slow ramp is one once it has moved this far)."),
        constant("num.dt_s", 0.25, "s", "Integration step."),
        constant("num.max_lead_s", 600.0, "s", "Longest lead time after the cutoff."),
    ]);
    ModelSpec {
        schema_version: WspSchema::V1,
        spec_id: SpecId::new(SPEC_ID).expect("id"),
        family: Family::Bass,
        model_name: MODEL_NAME.into(),
        target: TargetSpec {
            quantity: Quantity::FrontPosition,
            unit: "m".into(),
            description: "Position of the downstream tip of the visible flame along a thin fuel strip in \
                          concurrent flow, measured from the upstream (igniter) end of the sample, after \
                          the cutoff and conditional on the flow history up to each forecast time; with \
                          the probability that the flame has been quenched and that its tip has reached \
                          the end of the sample. The model works in sample widths; metres follow from the \
                          sample width of the prefix."
                .into(),
            excludes: vec![
                "the flow history: the operator's steps are an input, known up to each forecast time".into(),
                "opposed-flow spread, thick fuels, materials other than the cotton-fibreglass fabric".into(),
                "a metric position without a sample width in metres from the data lane".into(),
                "temperature, heat flux, species or soot inferred from image intensity".into(),
                "a calibrated posterior: the spread is assumed".into(),
            ],
        },
        features: vec![
            feature("u", "1", "control:flow_velocity", "U / norm.u_ref_cm_s - 1, U in cm/s, not below norm.u_min_cm_s @1"),
            feature("c_o2", "1", "condition:x_o2", "(x - norm.o2_ref) / norm.o2_scale @1"),
            feature("c_w", "1", "condition:width", "ln(W / W_ref) @1"),
            feature("tip", "sample width", "observation:front_position/tip@prefix", "position / W; stills after the igniter is off and at or before the cutoff @1"),
            feature("base", "sample width", "observation:front_position/base@prefix", "position / W; the same stills @1"),
        ],
        domain: vec![],
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::front2::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(
            "Time is seconds after the igniter is switched off. The law path is integrated from the \
             moment the igniter is switched on (front at the igniter, no overhang) with step num.dt_s: \
             dx = g v dt with g = ign.gain while the igniter is on (after ign.delay_s) and 1 after; \
             h relaxes exactly, h <- h_ss + (h - h_ss) exp(-dt / over.tau_s). Flow is a step function \
             of time. Prefix windows: the stills of the first and of the last enc.end_s seconds of the \
             prefix; window means of time, tip and base. Tip offset: ln of the advance of (tip - h) \
             between the windows over the law's advance in the same interval; base offset: ln of the \
             advance of the base over the law's advance in the interval shifted back by tau_b at the \
             later window. Offsets need enc.min_span_s between the windows and a positive law \
             advance; a non-positive observed advance is taken as -enc.e_clip[0]. E = exp(clamp(gain \
             x observed offset)), gain = sigmoid(enc.gain_b + enc.gain_w . (ln(span / 10 s), settled)), \
             settled = clamp((t_e - t_change) / 10 s, 0, 1) at the last-window time t_e: with U_e the \
             flow at t_e, t_change is the start time of the first flow entry after the most recent \
             entry at or before t_e whose flow differs from U_e by at least enc.change_cm_s, and the \
             time the igniter is switched on when no entry differs (never earlier than it). Position at the cutoff: last-window mean tip minus the modelled \
             overhang there, carried along the law. Spread of ln E: sqrt(gain s_floor^2 + (1 - gain) \
             s_pop^2 + (s_far[0] c_o2)^2 + (s_far[1] c_w)^2). sd of the tip at lead: sqrt(sigma^2 + \
             (s E advance)^2 + (rel_over x overhang change)^2 + drift_w^2 lead), Gaussian quantiles, \
             cut at the end of the sample when its length is known. Quenched by t: 1 - exp(-integral \
             of the quench hazard from the cutoff). Reached the end by t: the probability that the \
             uncut tip was at or beyond the end at t - end.linger_s. Rows: active = (1 - quenched)(1 \
             - reached), extinction = quenched, reached_end = (1 - quenched) reached. \
             Additions of this forward, each off at the value its entry names. (A) speed.split > 0: \
             with o = X_O2 / norm.o2_ref, N = max(N_speed(c), norm.v_min) and N0 the same with the \
             flow feature at its value for U = 0 (norm.u_min_cm_s / norm.u_ref_cm_s - 1), v_split = \
             max(N0 o^speed.pow_i + (N - N0) o^speed.pow[0], norm.v_min o^speed.pow_i) (W / \
             W_ref)^speed.pow[1] and v = (1 - speed.split) v_first + speed.split v_split, v_first the \
             rate above. (B) over.tau_up_s > 0: in an integration step that starts with h below \
             h_ss the relaxation time is over.tau_up_s, otherwise over.tau_s (the exact relaxation \
             never crosses h_ss inside a step). (C) speed.kappa > 0: t_ref is the cutoff, or 0 when \
             no still is admitted; the law path is integrated without slow-down to the end of the \
             step that contains t_ref, E is read, x_ref_law = law path at t_ref, and in every later \
             step v and h_ss are evaluated at U max(0, 1 - speed.kappa E (X_i - x_ref_law)), X_i \
             the law path at the start of the step; tau_b and the quench hazard keep U. (D) \
             unc.flow_w[0] > 0 or unc.flow_w[2] > 0: for each k with unc.flow_w[k] > 0 the whole \
             forward is run with the flow history U_k = max(0, U (1 + s_k unc.flow_rel) + s_k \
             unc.flow_abs_cm_s), s = (-1, 0, 1), the prefix read again; weights unc.flow_w over \
             their sum. Tip quantile at level p: the point where the weighted sum of the paths' \
             Gaussian CDFs (uncut median and sd of each path) reaches p, by 60 halvings of the \
             bracket [min(m_k - 9 sd_k), max(m_k + 9 sd_k)], then cut at the end of the sample; \
             mean: weighted mean of the paths' means; base: weighted median of the paths' bases \
             (linear between mass centres); quenched: weighted mean; reached the end: sum w (1 - \
             q_k) e_k / sum w (1 - q_k) (the weighted mean of e_k when every path is quenched), so \
             that each event row is the weighted mean of the paths' rows. Otherwise one run on the \
             flow as read. Nothing is fitted at request time. Uncertainty kind: assumed_noise."
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
    fn eval(&self, x: &[f64; N_IN]) -> f64 {
        let mut scratch = [0.0; HIDDEN];
        hinge_net(
            self.w1,
            self.b1,
            self.w2,
            self.skip,
            self.b0,
            x,
            &mut scratch,
        )
    }
}

pub(crate) struct Weights<'a> {
    speed: Net<'a>,
    over: Net<'a>,
    burn: Net<'a>,
    quench: Net<'a>,
    speed_pow: &'a [f64],
    over_pow: &'a [f64],
    over_tau: f64,
    ign_gain: f64,
    ign_delay: f64,
    linger: f64,
    gain_w: &'a [f64],
    gain_b: f64,
    mix: &'a [f64],
    e_clip: &'a [f64],
    s_pop: f64,
    s_floor: f64,
    s_far: &'a [f64],
    sigma: f64,
    sigma_x0: f64,
    rel_over: f64,
    drift: f64,
    /// Addition A: share of the split oxygen law, and the oxygen power of the zero-flow part.
    split: f64,
    pow_i: f64,
    /// Addition C: slow-down per sample width of advance.
    kappa: f64,
    /// Addition B: time of a growing overhang (0: `over_tau` in both directions).
    tau_up: f64,
    /// Addition D: shift and scale of the outer flow paths, and the weights of the three paths.
    flow_abs: f64,
    flow_rel: f64,
    flow_w: &'a [f64],
    u_ref: f64,
    u_min: f64,
    o2_ref: f64,
    o2_scale: f64,
    pub(crate) w_ref_m: f64,
    v_min: f64,
    end_s: f64,
    min_span: f64,
    change: f64,
    dt: f64,
    pub(crate) max_lead: f64,
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
            speed: net("speed"),
            over: net("over"),
            burn: net("burn"),
            quench: net("quench"),
            speed_pow: p.v("speed.pow"),
            over_pow: p.v("over.pow"),
            over_tau: p.s("over.tau_s"),
            ign_gain: p.s("ign.gain"),
            ign_delay: p.s("ign.delay_s"),
            linger: p.s("end.linger_s"),
            gain_w: p.v("enc.gain_w"),
            gain_b: p.s("enc.gain_b"),
            mix: p.v("enc.mix"),
            e_clip: p.v("enc.e_clip"),
            s_pop: p.s("unc.s_pop"),
            s_floor: p.s("unc.s_floor"),
            s_far: p.v("unc.s_far"),
            sigma: p.s("unc.sigma_w"),
            sigma_x0: p.s("unc.sigma_x0_w"),
            rel_over: p.s("unc.rel_over"),
            drift: p.s("unc.drift_w"),
            split: p.s("speed.split"),
            pow_i: p.s("speed.pow_i"),
            kappa: p.s("speed.kappa"),
            tau_up: p.s("over.tau_up_s"),
            flow_abs: p.s("unc.flow_abs_cm_s"),
            flow_rel: p.s("unc.flow_rel"),
            flow_w: p.v("unc.flow_w"),
            u_ref: p.s("norm.u_ref_cm_s"),
            u_min: p.s("norm.u_min_cm_s"),
            o2_ref: p.s("norm.o2_ref"),
            o2_scale: p.s("norm.o2_scale"),
            w_ref_m: p.s("norm.w_ref_m"),
            v_min: p.s("norm.v_min"),
            end_s: p.s("enc.end_s"),
            min_span: p.s("enc.min_span_s"),
            change: p.s("enc.change_cm_s"),
            dt: p.s("num.dt_s"),
            max_lead: p.s("num.max_lead_s"),
        }
    }

    /// The laws at one flow speed for the oxygen and sample of a run.
    fn laws(&self, u_cm_s: f64, x_o2: f64, width_ratio: f64) -> Laws {
        let c = [
            u_cm_s.max(self.u_min) / self.u_ref - 1.0,
            (x_o2 - self.o2_ref) / self.o2_scale,
            width_ratio.ln(),
        ];
        let (o, w) = (x_o2 / self.o2_ref, width_ratio);
        // the speed network at the flow of the call and at zero flow speed (the flow feature at the
        // value it has for U = 0; the oxygen and width features unchanged), neither below v_min
        let n = self.speed.eval(&c).max(self.v_min);
        let c0 = [0.0_f64.max(self.u_min) / self.u_ref - 1.0, c[1], c[2]];
        let n0 = self.speed.eval(&c0).max(self.v_min);
        let (oxygen, width) = (o.powf(self.speed_pow[0]), w.powf(self.speed_pow[1]));
        // the rate of the first front forward, in its order of operations, and its zero-flow part
        let (mut v, mut v_zero_flow) = (n * oxygen * width, n0 * oxygen * width);
        if self.split > 0.0 {
            // Addition A: the zero-flow part carries its own oxygen power; the flow part may not
            // take the rate below the smallest zero-flow rate
            let still = n0 * o.powf(self.pow_i);
            let v_split = (still + (n - n0) * oxygen).max(self.v_min * o.powf(self.pow_i)) * width;
            v = (1.0 - self.split) * v + self.split * v_split;
            v_zero_flow = (1.0 - self.split) * v_zero_flow + self.split * still * width;
        }
        Laws {
            v,
            v_zero_flow,
            h_ss: self.over.eval(&c).max(0.0) * o.powf(self.over_pow[0]) * w.powf(self.over_pow[1]),
            tau_b: self.burn.eval(&c).exp(),
            quench: self.quench.eval(&c).exp(),
        }
    }
}

/// The condition-level laws at one flow speed (reported in the diagnostics).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Laws {
    /// Spread rate, sample widths per second.
    pub v: f64,
    /// The part of the spread rate that remains at zero flow speed (the oxygen and width of the
    /// run), sample widths per second; `v - v_zero_flow` is the flow-dependent part.
    pub v_zero_flow: f64,
    /// Steady overhang of the tip, sample widths.
    pub h_ss: f64,
    /// Burn-through time, s.
    pub tau_b: f64,
    /// Quench hazard, 1/s.
    pub quench: f64,
}

/// What the forward needs of one burn, in sample widths and seconds after the igniter is off.
#[derive(Debug, Clone)]
pub struct FrontInput {
    pub cutoff_s: f64,
    /// When the igniter was switched on (at or before 0).
    pub igniter_on_s: f64,
    pub x_o2: f64,
    /// Sample width over the width of the reference sample.
    pub width_ratio: f64,
    /// Length of the exposed sample in sample widths, when known.
    pub window_w: Option<f64>,
    /// Flow speed as a step function `(t_s, u_cm_s)`, sorted by time, covering every forecast time.
    pub flow: Vec<(f64, f64)>,
    /// Tip positions `(t_s, widths)` of the stills at or before the cutoff, after the igniter is off.
    pub tip: Vec<(f64, f64)>,
    /// Base positions of the same stills.
    pub base: Vec<(f64, f64)>,
    /// Forecast times, at or after the cutoff.
    pub times: Vec<f64>,
}

/// What the prefix said (reported in the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct Encoded {
    pub stills_used: usize,
    /// Mean time of the last window (the cutoff without stills), seconds after the igniter is off.
    pub t_end_s: f64,
    /// Mean tip of the last window; `None` without stills.
    pub tip_end_w: Option<f64>,
    pub span_s: f64,
    pub settled: f64,
    pub e_tip: Option<f64>,
    pub e_base: Option<f64>,
    pub gain: f64,
    /// ln of the run offset carried.
    pub e: f64,
    /// sd of ln E.
    pub s: f64,
    /// Pyrolysis front and overhang at `t_end_s`, sample widths.
    pub x_end_w: f64,
    pub h_end_w: f64,
    pub sigma_x_w: f64,
    pub laws_at_cutoff: Laws,
}

/// What the laws and the law path say at each forecast time of one run (reported in the
/// diagnostics, so that a reader sees the additions at work). Sample widths, seconds, cm/s.
#[derive(Debug, Clone, Serialize)]
pub struct Trace {
    /// Start of the forecast integration: the cutoff, or 0 (igniter-off) without stills.
    pub t_ref_s: f64,
    /// The forecast front at `t_ref_s`: the reference of the slow-down downstream.
    pub x_ref_w: f64,
    /// Flow speed of this run at each forecast time.
    pub flow_cm_s: Vec<f64>,
    /// `U_eff / U` at each forecast time: 1 without slow-down (addition C).
    pub flow_factor: Vec<f64>,
    /// The forecast front (median), `x_end + E (X(t) - X(t_end))`.
    pub front_w: Vec<f64>,
    /// Spread rate of the law at the effective flow, and its zero-flow part (addition A).
    pub speed_w_s: Vec<f64>,
    pub speed_zero_flow_w_s: Vec<f64>,
    /// The modelled overhang, its target at the effective flow, and the relaxation time in force
    /// (addition B: `over.tau_up_s` while the overhang is below its target).
    pub overhang_w: Vec<f64>,
    pub overhang_target_w: Vec<f64>,
    pub overhang_tau_s: Vec<f64>,
}

/// One flow path of a mixture forecast (addition D), with what its own run returned.
#[derive(Debug, Clone, Serialize)]
pub struct FlowPath {
    /// -1, 0 or +1: the lower flow, the flow as read, the higher flow.
    pub sign: f64,
    /// Weight of the path, normalised over the paths that were run.
    pub weight: f64,
    pub encoded: Encoded,
    /// Median and sd of the uncut tip at each forecast time.
    pub tip_median_w: Vec<f64>,
    pub tip_sd_w: Vec<f64>,
    pub base_median_w: Vec<f64>,
    pub p_quenched: Vec<f64>,
    pub p_reached_end: Vec<f64>,
    pub trace: Trace,
}

/// The numbers of one forecast, in sample widths.
#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    /// What the prefix said: of the single run, or of the reference path of a mixture (the flow as
    /// read when it has weight, else the heavier outer path, the lower one on a tie).
    pub encoded: Encoded,
    pub times_s: Vec<f64>,
    pub levels: Vec<f64>,
    /// `[level][time]` tip position, sample widths.
    pub quantiles: Vec<Vec<f64>>,
    pub mean: Vec<f64>,
    /// Median base position (the front one burn-through time earlier), sample widths.
    pub base_median: Vec<f64>,
    /// Probability that the flame has been quenched, and that its tip has reached the end (in a
    /// mixture: among the paths not quenched).
    pub p_quenched: Vec<f64>,
    pub p_reached_end: Vec<f64>,
    /// The laws and the law path at the forecast times: of the same run as `encoded`.
    pub trace: Trace,
    /// The flow paths of a mixture forecast; empty when one run was made on the flow as read.
    pub paths: Vec<FlowPath>,
}

/// The law path of one burn on a regular grid from the igniter being switched on.
struct Path {
    t0: f64,
    dt: f64,
    x: Vec<f64>,
    h: Vec<f64>,
    /// Integral of the quench hazard from `t0`.
    lam: Vec<f64>,
}

impl Path {
    fn at(&self, v: &[f64], t: f64) -> f64 {
        let pos = ((t - self.t0) / self.dt).clamp(0.0, (v.len() - 1) as f64);
        let i = (pos.floor() as usize).min(v.len() - 2);
        v[i] + (pos - i as f64) * (v[i + 1] - v[i])
    }
    fn x(&self, t: f64) -> f64 {
        self.at(&self.x, t)
    }
    fn h(&self, t: f64) -> f64 {
        self.at(&self.h, t)
    }
    fn lam(&self, t: f64) -> f64 {
        self.at(&self.lam, t)
    }

    /// Integrates the steps `from..to` under `flow`. `slow = (x_ref, k)` is the slow-down
    /// downstream of addition C on the law path: the spread rate and the overhang target of a step
    /// see the flow times `max(0, 1 - k (x - x_ref))`, `x` the law path at the start of the step
    /// and `k = speed.kappa E`.
    fn integrate(
        &mut self,
        wt: &Weights<'_>,
        inp: &FrontInput,
        flow: &[(f64, f64)],
        (from, to): (usize, usize),
        slow: Option<(f64, f64)>,
    ) {
        for i in from..to {
            let t_mid = self.t0 + (i as f64 + 0.5) * wt.dt;
            let u = flow_at(flow, t_mid);
            let l = wt.laws(u, inp.x_o2, inp.width_ratio);
            let (v, h_ss) = match slow {
                Some((x_ref, k)) if k > 0.0 => {
                    let slowed = wt.laws(
                        u * (1.0 - k * (self.x[i] - x_ref)).max(0.0),
                        inp.x_o2,
                        inp.width_ratio,
                    );
                    (slowed.v, slowed.h_ss)
                }
                _ => (l.v, l.h_ss),
            };
            let g = if t_mid < self.t0 + wt.ign_delay {
                0.0
            } else if t_mid < 0.0 {
                wt.ign_gain
            } else {
                1.0
            };
            self.x[i + 1] = self.x[i] + g * v * wt.dt;
            self.h[i + 1] = if g > 0.0 {
                // Addition B: the exact relaxation never crosses its target inside a step, so the
                // side the overhang starts the step on is the side it stays on
                let tau = if wt.tau_up > 0.0 && self.h[i] < h_ss {
                    wt.tau_up
                } else {
                    wt.over_tau
                };
                h_ss + (self.h[i] - h_ss) * (-wt.dt / tau).exp()
            } else {
                self.h[i]
            };
            self.lam[i + 1] = self.lam[i] + if t_mid >= 0.0 { l.quench * wt.dt } else { 0.0 };
        }
    }
}

fn flow_at(flow: &[(f64, f64)], t: f64) -> f64 {
    let i = flow.partition_point(|s| s.0 <= t);
    flow[i.saturating_sub(1)].1
}

/// Mean time and value of the entries with `lo <= t <= hi`.
fn window(obs: &[(f64, f64)], lo: f64, hi: f64) -> Option<(f64, f64, usize)> {
    let sel: Vec<&(f64, f64)> = obs.iter().filter(|o| o.0 >= lo && o.0 <= hi).collect();
    if sel.is_empty() {
        return None;
    }
    let n = sel.len() as f64;
    Some((
        sel.iter().map(|o| o.0).sum::<f64>() / n,
        sel.iter().map(|o| o.1).sum::<f64>() / n,
        sel.len(),
    ))
}

/// Why a burn cannot be forecast, if it cannot: a scientific refusal.
fn check(wt: &Weights<'_>, inp: &FrontInput) -> Result<(), Support> {
    let bad = |gate, why: String| Err(refuse(gate, why));
    if !(inp.x_o2 > 0.0 && inp.x_o2 < 1.0) {
        return bad(
            SupportGate::Condition,
            format!("oxygen mole fraction {} is not inside (0, 1)", inp.x_o2),
        );
    }
    if !(inp.width_ratio > 0.0 && inp.width_ratio.is_finite()) {
        return bad(
            SupportGate::Condition,
            format!("width ratio {} is not positive", inp.width_ratio),
        );
    }
    if inp.flow.is_empty()
        || inp
            .flow
            .iter()
            .any(|s| !(s.0.is_finite() && s.1.is_finite() && s.1 >= 0.0))
    {
        return bad(SupportGate::Condition, "no usable flow history".into());
    }
    if inp.flow.windows(2).any(|w| w[1].0 < w[0].0) {
        return bad(
            SupportGate::Condition,
            "flow history is not sorted by time".into(),
        );
    }
    if !(inp.igniter_on_s <= 0.0 && inp.igniter_on_s.is_finite()) {
        return bad(
            SupportGate::Condition,
            format!(
                "igniter switched on at {} s, after it was switched off",
                inp.igniter_on_s
            ),
        );
    }
    let tau = inp.cutoff_s;
    if !(tau >= 0.0 && tau.is_finite()) {
        return bad(
            SupportGate::Horizon,
            format!("cutoff {tau} s is before the igniter is off"),
        );
    }
    if inp.times.is_empty()
        || inp
            .times
            .iter()
            .any(|t| !(*t >= tau && *t - tau <= wt.max_lead))
    {
        return bad(
            SupportGate::Horizon,
            format!(
                "forecast times must lie in [cutoff, cutoff + {} s]",
                wt.max_lead
            ),
        );
    }
    Ok(())
}

/// What one run of the forward under one flow history returns.
struct Single {
    encoded: Encoded,
    /// Median and sd of the uncut tip at each forecast time.
    tip_m: Vec<f64>,
    tip_sd: Vec<f64>,
    quantiles: Vec<Vec<f64>>,
    mean: Vec<f64>,
    base_median: Vec<f64>,
    p_quenched: Vec<f64>,
    p_reached_end: Vec<f64>,
    trace: Trace,
}

/// One run of the forward on a checked burn under the flow history `flow`.
fn run_path(wt: &Weights<'_>, inp: &FrontInput, flow: &[(f64, f64)], levels: &[f64]) -> Single {
    let tau = inp.cutoff_s;
    let t_max = inp.times.iter().copied().fold(tau, f64::max);
    // the law path from the igniter being switched on
    let t0 = inp.igniter_on_s;
    let n = (((t_max - t0) / wt.dt).ceil() as usize).max(1) + 1;
    let mut path = Path {
        t0,
        dt: wt.dt,
        x: vec![0.0; n + 1],
        h: vec![0.0; n + 1],
        lam: vec![0.0; n + 1],
    };
    // the stills of the prefix
    let mut tip: Vec<(f64, f64)> = inp
        .tip
        .iter()
        .copied()
        .filter(|o| o.0 >= 0.0 && o.0 <= tau && o.1.is_finite())
        .collect();
    tip.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut base: Vec<(f64, f64)> = inp
        .base
        .iter()
        .copied()
        .filter(|o| o.0 >= 0.0 && o.0 <= tau && o.1.is_finite())
        .collect();
    base.sort_by(|a, b| a.0.total_cmp(&b.0));
    // The forecast integration starts at the cutoff, or when the igniter is switched off for a
    // forecast without stills. The law path is integrated to the end of the step that contains
    // that time before the prefix is read; the rest needs the run offset (addition C).
    let t_ref = if tip.is_empty() { 0.0 } else { tau };
    let first = ((((t_ref - t0) / wt.dt).floor() as usize) + 1).min(n);
    path.integrate(wt, inp, flow, (0, first), None);
    let laws_cut = wt.laws(flow_at(flow, tau), inp.x_o2, inp.width_ratio);
    let c_o2 = (inp.x_o2 - wt.o2_ref) / wt.o2_scale;
    let c_w = inp.width_ratio.ln();
    let far2 = (wt.s_far[0] * c_o2).powi(2) + (wt.s_far[1] * c_w).powi(2);
    // what the prefix says
    let seen = if let (Some(first), Some(last)) = (tip.first(), tip.last()) {
        let (t_first, t_last) = (first.0, last.0);
        let late =
            window(&tip, t_last - wt.end_s, t_last).expect("the last still is in its window");
        let early =
            window(&tip, t_first, t_first + wt.end_s).expect("the first still is in its window");
        let (t_e, p_e) = (late.0, late.1);
        let span = t_e - early.0;
        let mut e_tip = None;
        let mut e_base = None;
        if span >= wt.min_span {
            let law = path.x(t_e) - path.x(early.0);
            if law > 1.0e-9 {
                let seen = (p_e - path.h(t_e)) - (early.1 - path.h(early.0));
                e_tip = Some(if seen > 0.0 {
                    (seen / law).ln()
                } else {
                    -wt.e_clip[0]
                });
            }
            if let (Some(bl), Some(be)) = (
                window(&base, t_last - wt.end_s, t_last),
                window(&base, t_first, t_first + wt.end_s),
            ) {
                // the burn-through time at the later window (the spec note), not at the cutoff
                let tau_b = wt
                    .laws(flow_at(flow, bl.0), inp.x_o2, inp.width_ratio)
                    .tau_b;
                let law = path.x(bl.0 - tau_b) - path.x(be.0 - tau_b);
                if bl.0 - be.0 >= wt.min_span && law > 1.0e-9 {
                    let seen = bl.1 - be.1;
                    e_base = Some(if seen > 0.0 {
                        (seen / law).ln()
                    } else {
                        -wt.e_clip[0]
                    });
                }
            }
        }
        let (mut num, mut den) = (0.0, 0.0);
        for (e, w) in [(e_tip, wt.mix[0]), (e_base, wt.mix[1])] {
            if let Some(e) = e {
                num += w * e;
                den += w;
            }
        }
        // seconds since the flow last changed, at the last window: walking the step function back
        // from t_e, the most recent entry whose flow differs from the flow at t_e by at least
        // enc.change_cm_s; the flow last changed when the entry after it starts. A flicker smaller
        // than that is not a change, a slow ramp is one once it has moved that far. Without such an
        // entry, and never earlier than that, the clock starts when the igniter is switched on.
        let t_change = {
            let k = flow.partition_point(|s| s.0 <= t_e).saturating_sub(1);
            let u_e = flow[k].1;
            flow[..=k]
                .iter()
                .rposition(|s| (s.1 - u_e).abs() >= wt.change)
                .map_or(t0, |i| flow[i + 1].0.max(t0))
        };
        let settled = ((t_e - t_change) / 10.0).clamp(0.0, 1.0);
        let gain = if den > 0.0 {
            sigmoid(
                wt.gain_b + wt.gain_w[0] * (span.max(1.0e-6) / 10.0).ln() + wt.gain_w[1] * settled,
            )
        } else {
            0.0
        };
        let e_obs = if den > 0.0 { num / den } else { 0.0 };
        let e = (gain * e_obs).clamp(-wt.e_clip[0], wt.e_clip[1]);
        Some(Encoded {
            stills_used: tip.len(),
            t_end_s: t_e,
            tip_end_w: Some(p_e),
            span_s: span,
            settled,
            e_tip,
            e_base,
            gain,
            e,
            s: (gain * wt.s_floor.powi(2) + (1.0 - gain) * wt.s_pop.powi(2) + far2).sqrt(),
            x_end_w: p_e - path.h(t_e),
            h_end_w: path.h(t_e),
            sigma_x_w: wt.sigma,
            laws_at_cutoff: laws_cut,
        })
    } else {
        None
    };
    // the rest of the law path: with the slow-down downstream when speed.kappa > 0 (the run offset
    // of a forecast without stills is 1)
    let x_ref_law = path.x(t_ref);
    let slow = wt.kappa * seen.as_ref().map_or(1.0, |enc| enc.e.exp());
    path.integrate(wt, inp, flow, (first, n), Some((x_ref_law, slow)));
    let enc = seen.unwrap_or_else(|| Encoded {
        stills_used: 0,
        t_end_s: tau,
        tip_end_w: None,
        span_s: 0.0,
        settled: 0.0,
        e_tip: None,
        e_base: None,
        gain: 0.0,
        e: 0.0,
        s: (wt.s_pop.powi(2) + far2).sqrt(),
        x_end_w: path.x(tau),
        h_end_w: path.h(tau),
        sigma_x_w: wt.sigma_x0,
        laws_at_cutoff: laws_cut,
    });
    let big_e = enc.e.exp();
    // tip of the uncut band at time t: median and sd
    let tip_at = |t: f64| -> (f64, f64) {
        let adv = big_e * (path.x(t) - path.x(enc.t_end_s));
        let dh = path.h(t) - enc.h_end_w;
        let lead = (t - enc.t_end_s).max(0.0);
        let var = enc.sigma_x_w.powi(2)
            + (enc.s * adv).powi(2)
            + (wt.rel_over * dh).powi(2)
            + wt.drift.powi(2) * lead;
        (enc.x_end_w + adv + path.h(t), var.sqrt())
    };
    let nt = inp.times.len();
    let mut quantiles = vec![vec![0.0; nt]; levels.len()];
    let (mut mean, mut base_median) = (vec![0.0; nt], vec![0.0; nt]);
    let (mut tip_m, mut tip_sd) = (vec![0.0; nt], vec![0.0; nt]);
    let (mut p_q, mut p_end) = (vec![0.0; nt], vec![0.0; nt]);
    let cut = |v: f64| inp.window_w.map_or(v, |w| v.min(w));
    for (j, &t) in inp.times.iter().enumerate() {
        let (m, sd) = tip_at(t);
        (tip_m[j], tip_sd[j]) = (m, sd);
        for (l, p) in levels.iter().enumerate() {
            quantiles[l][j] = cut(m + normal_quantile(*p) * sd);
        }
        mean[j] = cut(m);
        let tau_b = wt.laws(flow_at(flow, t), inp.x_o2, inp.width_ratio).tau_b;
        base_median[j] = (enc.x_end_w + big_e * (path.x(t - tau_b) - path.x(enc.t_end_s))).max(0.0);
        p_q[j] = -(-(path.lam(t) - path.lam(tau)).max(0.0)).exp_m1();
        p_end[j] = match inp.window_w {
            Some(w) => {
                let (m1, sd1) = tip_at((t - wt.linger).max(enc.t_end_s));
                if sd1 > 0.0 {
                    normal_cdf((m1 - w) / sd1)
                } else {
                    f64::from(u8::from(m1 >= w))
                }
            }
            None => 0.0,
        };
    }
    // a probability of having reached the end does not come back down: running maximum in time
    // order (the forecast times need not be sorted)
    let mut order: Vec<usize> = (0..nt).collect();
    order.sort_by(|a, b| inp.times[*a].total_cmp(&inp.times[*b]));
    for k in 1..nt {
        p_end[order[k]] = p_end[order[k]].max(p_end[order[k - 1]]);
    }
    // what a reader needs to see the additions at work
    let front_at = |t: f64| enc.x_end_w + big_e * (path.x(t) - path.x(enc.t_end_s));
    let mut trace = Trace {
        t_ref_s: t_ref,
        x_ref_w: front_at(t_ref),
        flow_cm_s: Vec::with_capacity(nt),
        flow_factor: Vec::with_capacity(nt),
        front_w: Vec::with_capacity(nt),
        speed_w_s: Vec::with_capacity(nt),
        speed_zero_flow_w_s: Vec::with_capacity(nt),
        overhang_w: Vec::with_capacity(nt),
        overhang_target_w: Vec::with_capacity(nt),
        overhang_tau_s: Vec::with_capacity(nt),
    };
    for &t in &inp.times {
        let u = flow_at(flow, t);
        // the factor of the integration step that holds t (1 up to the step that contains t_ref)
        let step = (((t - t0) / wt.dt).floor().max(0.0) as usize).min(n - 1);
        let factor = if slow > 0.0 && step >= first {
            (1.0 - slow * (path.x[step] - x_ref_law)).max(0.0)
        } else {
            1.0
        };
        let l = wt.laws(u * factor, inp.x_o2, inp.width_ratio);
        let h = path.h(t);
        trace.flow_cm_s.push(u);
        trace.flow_factor.push(factor);
        trace.front_w.push(front_at(t));
        trace.speed_w_s.push(l.v);
        trace.speed_zero_flow_w_s.push(l.v_zero_flow);
        trace.overhang_w.push(h);
        trace.overhang_target_w.push(l.h_ss);
        trace.overhang_tau_s.push(if wt.tau_up > 0.0 && h < l.h_ss {
            wt.tau_up
        } else {
            wt.over_tau
        });
    }
    Single {
        encoded: enc,
        tip_m,
        tip_sd,
        quantiles,
        mean,
        base_median,
        p_quenched: p_q,
        p_reached_end: p_end,
        trace,
    }
}

/// Signs of the three flow paths of addition D: the lower flow, the flow as read, the higher flow.
const FLOW_SIGNS: [f64; 3] = [-1.0, 0.0, 1.0];
/// Halvings of the bracket of a mixture quantile, and the half-width of the bracket in sd.
const MIX_HALVINGS: usize = 60;
const MIX_REACH_SD: f64 = 9.0;

/// Quantiles of a mixture of Gaussians `(median, sd, weight)`, weights summing to one: for each
/// level, the point where the weighted sum of the component CDFs reaches it, by a fixed number of
/// halvings of a bracket that holds every component (a component without spread is a step). The CDF
/// is monotone; this inverts the predictive distribution and involves no parameter.
fn mixture_quantiles(comps: &[(f64, f64, f64)], levels: &[f64], out: &mut [f64]) {
    let cdf = |x: f64| -> f64 {
        comps
            .iter()
            .map(|&(m, sd, w)| {
                w * if sd > 0.0 {
                    normal_cdf((x - m) / sd)
                } else {
                    f64::from(u8::from(x >= m))
                }
            })
            .sum()
    };
    let lo0 = comps
        .iter()
        .map(|c| c.0 - MIX_REACH_SD * c.1)
        .fold(f64::INFINITY, f64::min);
    let hi0 = comps
        .iter()
        .map(|c| c.0 + MIX_REACH_SD * c.1)
        .fold(f64::NEG_INFINITY, f64::max);
    for (o, p) in out.iter_mut().zip(levels) {
        let (mut lo, mut hi) = (lo0, hi0);
        for _ in 0..MIX_HALVINGS {
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

/// The forward on one burn in sample widths: `Err` is a scientific refusal. One run on the flow as
/// read, or, with a positive outer weight in `unc.flow_w`, the mixture of the flow paths (addition
/// D).
pub(crate) fn run(wt: &Weights<'_>, inp: &FrontInput, levels: &[f64]) -> Result<Forecast, Support> {
    check(wt, inp)?;
    if !(wt.flow_w[0] > 0.0 || wt.flow_w[2] > 0.0) {
        let one = run_path(wt, inp, &inp.flow, levels);
        return Ok(Forecast {
            encoded: one.encoded,
            times_s: inp.times.clone(),
            levels: levels.to_vec(),
            quantiles: one.quantiles,
            mean: one.mean,
            base_median: one.base_median,
            p_quenched: one.p_quenched,
            p_reached_end: one.p_reached_end,
            trace: one.trace,
            paths: vec![],
        });
    }
    let total: f64 = wt.flow_w.iter().sum();
    let mut runs: Vec<(f64, f64, Single)> = Vec::with_capacity(3);
    for (&sign, &w) in FLOW_SIGNS.iter().zip(wt.flow_w) {
        if !(w > 0.0) {
            continue;
        }
        let one = if sign == 0.0 {
            run_path(wt, inp, &inp.flow, levels)
        } else {
            let flow: Vec<(f64, f64)> = inp
                .flow
                .iter()
                .map(|&(t, u)| {
                    (
                        t,
                        (u * (1.0 + sign * wt.flow_rel) + sign * wt.flow_abs).max(0.0),
                    )
                })
                .collect();
            run_path(wt, inp, &flow, levels)
        };
        runs.push((sign, w / total, one));
    }
    let nt = inp.times.len();
    let mut quantiles = vec![vec![0.0; nt]; levels.len()];
    let (mut mean, mut base_median) = (vec![0.0; nt], vec![0.0; nt]);
    let (mut p_q, mut p_end) = (vec![0.0; nt], vec![0.0; nt]);
    let cut = |v: f64| inp.window_w.map_or(v, |w| v.min(w));
    let mut comps = vec![(0.0, 0.0, 0.0); runs.len()];
    let mut atoms = vec![(0.0, 0.0); runs.len()];
    let mut q = vec![0.0; levels.len()];
    for j in 0..nt {
        for (c, (_, w, one)) in comps.iter_mut().zip(&runs) {
            *c = (one.tip_m[j], one.tip_sd[j], *w);
        }
        mixture_quantiles(&comps, levels, &mut q);
        for (row, v) in quantiles.iter_mut().zip(&q) {
            row[j] = cut(*v);
        }
        mean[j] = runs.iter().map(|(_, w, one)| w * one.mean[j]).sum();
        for (a, (_, w, one)) in atoms.iter_mut().zip(&runs) {
            *a = (one.base_median[j], *w);
        }
        let mut median = [0.0];
        common::atom_quantiles(&mut atoms, &[0.5], &mut median);
        base_median[j] = median[0];
        // the event rows of the mixture are the weighted means of the paths' rows
        p_q[j] = runs.iter().map(|(_, w, one)| w * one.p_quenched[j]).sum();
        let alive: f64 = runs
            .iter()
            .map(|(_, w, one)| w * (1.0 - one.p_quenched[j]))
            .sum();
        p_end[j] = if alive > 0.0 {
            (runs
                .iter()
                .map(|(_, w, one)| w * (1.0 - one.p_quenched[j]) * one.p_reached_end[j])
                .sum::<f64>()
                / alive)
                .min(1.0)
        } else {
            runs.iter()
                .map(|(_, w, one)| w * one.p_reached_end[j])
                .sum()
        };
    }
    // the reference path: the flow as read when it was run, else the heavier outer path
    let reference = runs
        .iter()
        .position(|r| r.0 == 0.0)
        .unwrap_or_else(|| usize::from(runs.len() > 1 && runs[1].1 > runs[0].1));
    let (encoded, trace) = (
        runs[reference].2.encoded.clone(),
        runs[reference].2.trace.clone(),
    );
    Ok(Forecast {
        encoded,
        times_s: inp.times.clone(),
        levels: levels.to_vec(),
        quantiles,
        mean,
        base_median,
        p_quenched: p_q,
        p_reached_end: p_end,
        trace,
        paths: runs
            .into_iter()
            .map(|(sign, weight, one)| FlowPath {
                sign,
                weight,
                encoded: one.encoded,
                tip_median_w: one.tip_m,
                tip_sd_w: one.tip_sd,
                base_median_w: one.base_median,
                p_quenched: one.p_quenched,
                p_reached_end: one.p_reached_end,
                trace: one.trace,
            })
            .collect(),
    })
}

/// The forward of a release on one burn in sample widths (the development route of the track files
/// of the measurement study). Nothing metric is claimed.
pub fn forecast_track(
    spec: &ModelSpec,
    release: &LoadedRelease,
    input: &FrontInput,
    levels: &[f64],
) -> Result<Result<Forecast, Support>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    check_forward(spec, release)?;
    Ok(run(&Weights::bind(&params), input, levels))
}

/// The laws of a release at one flow speed, oxygen fraction and width ratio (inspection; no data).
pub fn laws_of(
    spec: &ModelSpec,
    release: &LoadedRelease,
    u_cm_s: f64,
    x_o2: f64,
    width_ratio: f64,
) -> Result<Laws, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    Ok(Weights::bind(&params).laws(u_cm_s, x_o2, width_ratio))
}

fn check_forward(spec: &ModelSpec, release: &LoadedRelease) -> Result<(), Refusal> {
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
                "release {} was assembled with forward {}, this build runs {this}",
                release.release.id,
                built_with.map_or("none".to_string(), |d| d.to_string()),
            ),
        ));
    }
    Ok(())
}

/// A metric forecast and the sample width it was scaled with.
pub struct Metric {
    pub forecast: Forecast,
    pub width_m: f64,
}

/// Strict forward of a `bass_front.wsp2` release on an admitted prefix with metric observations:
/// `Ok(Err(support))` is a scientific refusal, `Err(refusal)` a release that cannot be run at all.
///
/// The prefix must carry: conditions `x_o2` and `width` (m), optionally `length` (m); control
/// stages `flow_velocity` (m/s, a step function in time) and one `igniter` stage with its end time;
/// observations of `front_position` in metres on channels `tip` and `base`. Times in the prefix are
/// seconds since its time origin; the forward shifts them so the igniter is off at zero.
pub fn forecast(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<Result<Metric, Support>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    check_forward(spec, release)?;
    let wt = Weights::bind(&params);
    if matches!(
        release.release.state,
        QualificationState::Failed | QualificationState::Unsupported
    ) {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!("release state {:?}", release.release.state),
        )));
    }
    if prefix.family != Family::Bass {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            "prefix is not a BASS run",
        )));
    }
    let mut reasons = Vec::new();
    let mut get = |k: &str| match common::condition(prefix, k) {
        Ok(v) => v,
        Err(e) => {
            reasons.push(e);
            f64::NAN
        }
    };
    let (x_o2, width) = (get("x_o2"), get("width"));
    if width.is_finite() && !(width > 0.0) {
        reasons.push(format!("width = {width} m is not positive"));
    }
    if !reasons.is_empty() {
        return Ok(Err(Support {
            admitted: false,
            gate: SupportGate::Condition,
            reasons,
        }));
    }
    let length = common::condition(prefix, "length")
        .ok()
        .filter(|l| *l > 0.0);
    let Some(igniter) = prefix
        .known_controls
        .iter()
        .find(|s| s.control.eq_ignore_ascii_case("igniter"))
    else {
        return Ok(Err(refuse(
            SupportGate::Condition,
            "no igniter stage among the known controls: the time the igniter is switched off is the model's clock",
        )));
    };
    let Some(t_off) = igniter.t_end_s else {
        return Ok(Err(refuse(
            SupportGate::Condition,
            "the igniter stage has no end time",
        )));
    };
    let mut flow: Vec<(f64, f64)> = prefix
        .known_controls
        .iter()
        .filter(|s| s.control.eq_ignore_ascii_case("flow_velocity"))
        .filter_map(|s| {
            let v = s.value.as_ref()?.value_si?;
            v.is_finite().then_some((s.t_start_s - t_off, v * 100.0))
        })
        .collect();
    flow.sort_by(|a, b| a.0.total_cmp(&b.0));
    if flow.is_empty() {
        return Ok(Err(refuse(
            SupportGate::Condition,
            "no flow_velocity stage among the known controls",
        )));
    }
    let series = |channel: &str| -> Vec<(f64, f64)> {
        prefix
            .observations
            .iter()
            .filter(|o| {
                o.quantity == Quantity::FrontPosition
                    && o.channel.as_deref() == Some(channel)
                    && o.time_s <= prefix.cutoff_s
            })
            .filter_map(|o| Some((o.time_s - t_off, o.value_si? / width)))
            .collect()
    };
    let tau = prefix.cutoff_s - t_off;
    let mut times: Vec<f64> = common::times_of(
        settings,
        prefix.cutoff_s,
        60.0,
        prefix.cutoff_s + wt.max_lead,
    )
    .into_iter()
    .map(|t| t - t_off)
    .filter(|t| t.is_finite() && *t >= tau)
    .collect();
    times.dedup();
    let input = FrontInput {
        cutoff_s: tau,
        igniter_on_s: (igniter.t_start_s - t_off).min(0.0),
        x_o2,
        width_ratio: width / wt.w_ref_m,
        window_w: length.map(|l| l / width),
        flow,
        tip: series("tip"),
        base: series("base"),
        times,
    };
    let levels = common::levels_of(settings);
    Ok(run(&wt, &input, &levels).map(|mut f| {
        // back to the prefix clock
        f.times_s.iter_mut().for_each(|t| *t += t_off);
        Metric {
            forecast: f,
            width_m: width,
        }
    }))
}

fn body(m: Metric) -> Body {
    let (f, w) = (m.forecast, m.width_m);
    let scale = |v: &[f64]| v.iter().map(|x| x * w).collect::<Vec<f64>>();
    let nt = f.times_s.len();
    let mut table = vec![vec![0.0; nt]; 4];
    for j in 0..nt {
        let (q, e) = (f.p_quenched[j], f.p_reached_end[j]);
        table[0][j] = (1.0 - q) * (1.0 - e);
        table[1][j] = q;
        table[2][j] = (1.0 - q) * e;
    }
    let diagnostics = json!({
        "encoded": f.encoded,
        "sample_width_m": w,
        "tip_quantiles_sample_widths": f.quantiles,
        "base_median_m": scale(&f.base_median),
        "trace_sample_widths": f.trace,
        "flow_paths": if f.paths.is_empty() {
            json!("one run on the flow as read")
        } else {
            json!(f.paths)
        },
        "observable": "downstream tip of the visible flame along the sample; not a pyrolysis front, a temperature or a species",
        "conditional_on": "the flow history up to each forecast time",
        "uncertainty_kind": "assumed_noise",
    });
    Body {
        kind: None,
        quantity: Some(Quantity::FrontPosition),
        times_s: f.times_s,
        quantile_levels: f.levels,
        quantiles: f.quantiles.iter().map(|q| scale(q)).collect(),
        mean: scale(&f.mean),
        event_categories: CATEGORIES.iter().map(|s| s.to_string()).collect(),
        event_cdf: table,
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
        Ok(m) => common::artifact(release, prefix, settings, admitted(), body(m)),
        Err(support) => common::artifact(
            release,
            prefix,
            settings,
            support,
            Body {
                quantity: Some(Quantity::FrontPosition),
                ..Body::default()
            },
        ),
    })
}

/// `bass_front.wsp2` behind the common expert interface.
pub struct FrontWsp {
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl Default for FrontWsp {
    fn default() -> Self {
        Self::new()
    }
}

impl FrontWsp {
    pub fn new() -> Self {
        FrontWsp {
            spec: spec(),
            policy: ProspectivePolicy::new(
                Family::Bass,
                &[
                    "x_o2",
                    "pressure",
                    "flow_velocity",
                    "width",
                    "length",
                    "thickness",
                ],
                &[],
            ),
        }
    }
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }
}

impl Expert for FrontWsp {
    fn family(&self) -> Family {
        Family::Bass
    }
    fn model_name(&self) -> &str {
        MODEL_NAME
    }
    fn is_baseline(&self) -> bool {
        false
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }
    fn default_config(&self) -> ExpertResult<Value> {
        Ok(json!({"construction": "literal", "spec_id": SPEC_ID}))
    }
    fn estimate_hours(&self, _: &Value, _: usize, _: usize, _: &Profile) -> f64 {
        0.0
    }
    fn fit(&self, _: &FitData<'_>, _: &Value, _: &mut FitContext) -> ExpertResult<ModelRelease> {
        bail!("bass_front.wsp2 has no fitting procedure: its release is assembled from a literal patch (`ignis wsp assemble`)")
    }
    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact> {
        match forecast(&self.spec, release, prefix, settings)? {
            Ok(m) => Ok(common::artifact(
                release,
                prefix,
                settings,
                admitted(),
                body(m),
            )),
            Err(support) => Err(common::refusal_of(&support).into()),
        }
    }
    fn log_likelihood(&self, _: &LoadedRelease, _: &View) -> ExpertResult<f64> {
        bail!("bass_front.wsp2 declares an assumed spread, not a likelihood")
    }
}

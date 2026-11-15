//! `bass_front.wsp5` — where the flame on a thin fuel strip will be, in a low-speed flow whose
//! history is a known control, and whether the sample ignites and stays lit. The fifth front
//! forward: `front.rs` for a flame that is burning, unchanged, with its quench hazard replaced by
//! the boundary law of `docs/wsp/research/phys_quench_w0.md`, section 6.
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
//!            tau_b = exp(N_burn)
//! ```
//!
//! Lengths are in sample widths: the sample is its own ruler in the top view, and v and h are
//! written in reference sample widths (the 2 cm class) with a power of the width ratio for other
//! samples. Speeds are in widths per second, the flow in cm/s. Everything above is `front.rs`, line
//! for line; with the same numbers it returns the same tip, bit for bit (`tests/front5_forward.rs`).
//!
//! Whether there is a flame: the boundary law (section 6.3 of the dossier). Material m (0 SIBAL
//! fabric, 1 PMMA sheet 1 mm), direction d (0 concurrent, 1 opposed), pressure p in atm, duct
//! flow U in cm/s, sample width W in cm (the width ratio times the reference width
//! `norm.w_ref_m`), oxygen X:
//!
//! ```text
//! U_e    = U + us[m] (opposed),   sqrt(U^2 + us[m]^2) (concurrent)
//! A      = ar[m,d] + aw[m] / W
//! X_lim  = x0[m] + cp[m] ln(1/p) + sb[m,d] U_e + A / (p U_e)
//! s^2    = sd_x0[m]^2 + (sd_cp[m] ln(1/p))^2 + (sd_sb[m,d] U_e)^2 + (sd_ar[m,d]^2 + (sd_aw[m] / W)^2) / (p U_e)^2
//! P_sus  = Phi((X - X_lim) / s)
//! P_ign  = Phi((X - X_lim(U_ign) - dx[m]) / sqrt(s(U_ign)^2 + sd_a^2)) Phi(ln(t_on / t_req[m]) / sd_lnt)
//!
//! nodes e_k, weights w_k: 41 equal-probability nodes of a standard normal, e_k the quantile at
//! (k - 0.5) / 41 and w_k = 1/41 (`quad.z`, `quad.w`, constants of the spec); e_k is where the
//! true boundary is, in units of s, and it is
//! the same node at every time of a burn.
//! lit_k      = Phi((X - X_lim(U_ign) - dx[m] - s(U_ign) e_k) / sd_a) Phi(ln(t_on / t_req[m]) / sd_lnt)
//! margin_k   = X - X_lim(U(t)) - s(U(t)) e_k
//! w_r        = r / (r + b),  r = max(A / (p U_e), 0),  b = sb[m,d] U_e   (1/2 when r + b = 0)
//! rate_k     = (w_r / tau[m,0] + (1 - w_r) / tau[m,1]) Phi(-margin_k / ext.width)
//! Lambda_k   = integral of rate_k from igniter-off, exact on the step function of the flow
//! no flame seen in the prefix:  P_alive(t) = sum_k w_k lit_k exp(-Lambda_k(t))
//!                               P_quenched_by(t) = sum_k w_k lit_k (1 - exp(-Lambda_k(t)))
//! a flame seen at t_c (the last still of the prefix): the nodes are weighted by what that says,
//!   pi_k ~ w_k Phi((X - X_lim(U_ign) - dx[m] - s(U_ign) e_k) / sd_a) exp(-Lambda_k(t_c))
//!   (the igniter's time factor is the same at every node and cancels), and
//!   P_alive(t) = sum_k pi_k exp(-(Lambda_k(t) - Lambda_k(t_c))),  P_quenched_by = 1 - P_alive.
//! ```
//!
//! U_ign is the flow in force when the igniter is switched on, t_on the time it was on. The event
//! rows of `front.rs` read the probability that there is no flame at t, `1 - P_alive(t)` (never
//! lit, or gone out), where `front.rs` read its quench hazard.
//!
//! Where the tip law holds. It was written for the cotton-fibreglass fabric in concurrent flow. For
//! an opposed flow or another material the forward returns the event probabilities above and
//! refuses the tip forecast ([`Outcome::EventsOnly`]); that refusal is scientific, not an error.
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
//! position while a flame exists, the probability that there is no flame, and the probability that
//! the tip has reached the end of the sample; P(ignites), P(flame alive at t) and P(quenched by t).
//! Spread is assumed, not calibrated: a relative spread of the run offset, the measurement noise of
//! the tip, a share of the modelled overhang change, and a slow drift. The width of the boundary is
//! the propagated uncertainty of its coefficients, typed by the author.
//!
//! The shared contracts give a front position in metres. A prefix that carries the sample width in
//! metres is forecast in metres; the track files of the measurement study, which have no length
//! calibration, are forecast in sample widths through [`run`] and never enter the shared records.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    AnalysisArtifact, Family, FeatureSpec, Finite, ForwardContract, ForwardProfile, ModelRelease,
    ModelSpec, PrefixBundle, QualificationState, Quantity, SpecId, Support, SupportGate,
    TargetSpec, TensorFill, TensorRole, TensorSpec, WspSchema,
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

pub const SPEC_ID: &str = "bass_front-wsp-v5";
pub const MODEL_NAME: &str = "bass_front.wsp5";
pub const FORWARD_VERSION: &str = "5";
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
/// Event rows when the tip forecast is refused: a flame at t, or none.
pub const CATEGORIES_EVENTS: [&str; 2] = ["active_supported", "extinction"];
/// Materials of the boundary law (axis M of its tensors).
pub const MATERIALS: [&str; 2] = ["SIBAL fabric", "PMMA sheet 1 mm"];
/// Number of nodes of the quadrature over the boundary error.
pub const NODES: usize = 41;
/// Equal-probability nodes of a standard normal variable: `Q41_Z[k - 1]` is the normal quantile at
/// `(k - 0.5) / 41`, k = 1..41, each with weight 1/41 (`Q41_W`). The integrand over the boundary
/// error is a step (a node is inside or outside the limit), so every node carries the same small
/// mass: the long-time survival moves by at most 1/41 when one node crosses the limit, and is
/// within about 1/82 of the closed form. Mathematical constants, computed once to double precision
/// (checked against erfc round-trips in `tests/front5_forward.rs`) and fixed in the spec
/// (`quad.z`, `quad.w`), not authored tensors.
pub const Q41_Z: [f64; NODES] = [
    -2.2509256965027937,
    -1.7917642873785158,
    -1.54663527139923,
    -1.3698564795107977,
    -1.2278262613112725,
    -1.107002880537908,
    -1.0004905456193152,
    -0.9042673435995684,
    -0.8157657139309344,
    -0.7332357831855119,
    -0.6554235052344266,
    -0.5813930065324557,
    -0.5104216426632189,
    -0.44193452638497827,
    -0.3754617702355184,
    -0.31060942538225567,
    -0.24703899073897723,
    -0.1844524370947431,
    -0.12258084388880242,
    -0.06117541103855111,
    0.0,
    0.06117541103855111,
    0.12258084388880242,
    0.1844524370947431,
    0.24703899073897723,
    0.31060942538225567,
    0.3754617702355184,
    0.44193452638497827,
    0.5104216426632189,
    0.5813930065324557,
    0.6554235052344266,
    0.7332357831855119,
    0.8157657139309344,
    0.9042673435995684,
    1.0004905456193152,
    1.107002880537908,
    1.2278262613112725,
    1.3698564795107977,
    1.54663527139923,
    1.7917642873785158,
    2.2509256965027937,
];
pub const Q41_W: [f64; NODES] = [1.0 / 41.0; NODES];
/// Pascal per standard atmosphere: the shared records give pressure in Pa, the law reads atm.
const PA_PER_ATM: f64 = 101_325.0;

/// A constant vector of the spec.
fn constant_vec(name: &str, values: &[f64], unit: &str, description: &str) -> TensorSpec {
    TensorSpec {
        name: name.into(),
        shape: vec![values.len() as u32],
        role: TensorRole::Constant,
        fill: TensorFill::Fixed {
            values: values
                .iter()
                .map(|v| Finite::new(*v).expect("finite constant"))
                .collect(),
        },
        unit: Some(unit.into()),
        bounds: None,
        description: description.into(),
    }
}

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
        // the boundary law (dossier phys_quench_w0.md, section 6.2): axis M material (0 SIBAL
        // fabric, 1 PMMA sheet 1 mm), D direction (0 concurrent, 1 opposed), B branch (0 radiative
        // quench, 1 blow-off); row-major
        authored("sus.x0", &[2], PhysicalParameter, Some("mole fraction"), Some((0.08, 0.30)),
                 "Floor of the oxygen limit, per material [M]."),
        authored("sus.cp", &[2], PhysicalParameter, Some("mole fraction per e-fold of 1/p"), Some((0.0, 0.10)),
                 "Pressure shift of the floor, per material [M]: X_lim gains cp ln(1/p), p in atm."),
        authored("sus.sb", &[2, 2], PhysicalParameter, Some("mole fraction per cm/s"), Some((0.0, 0.01)),
                 "Blow-off slope [M, D]: X_lim gains sb U_e."),
        authored("sus.ar", &[2, 2], PhysicalParameter, Some("mole fraction cm/s atm"), Some((0.0, 0.5)),
                 "Radiative-loss coefficient [M, D]: A = ar + aw / W, X_lim gains A / (p U_e)."),
        authored("sus.aw", &[2], PhysicalParameter, Some("mole fraction cm^2/s atm"), Some((-0.2, 0.3)),
                 "Lateral coefficient [M] (loss positive), divided by the sample width W in cm."),
        authored("sus.us", &[2], PhysicalParameter, Some("cm/s"), Some((0.0, 5.0)),
                 "Self-spread speed [M]: U_e = U + us (opposed), sqrt(U^2 + us^2) (concurrent)."),
        authored("sus.sd_x0", &[2], NoiseParameter, Some("mole fraction"), Some((0.001, 0.05)),
                 "sd of x0 [M]."),
        authored("sus.sd_cp", &[2], NoiseParameter, Some("mole fraction per e-fold of 1/p"), Some((0.0, 0.10)),
                 "sd of cp [M]."),
        authored("sus.sd_sb", &[2, 2], NoiseParameter, Some("mole fraction per cm/s"), Some((0.0, 0.01)),
                 "sd of sb [M, D]."),
        authored("sus.sd_ar", &[2, 2], NoiseParameter, Some("mole fraction cm/s atm"), Some((0.0, 0.5)),
                 "sd of ar [M, D]."),
        authored("sus.sd_aw", &[2], NoiseParameter, Some("mole fraction cm^2/s atm"), Some((0.0, 0.3)),
                 "sd of aw [M]."),
        authored("ign.dx", &[2], PhysicalParameter, Some("mole fraction"), Some((-0.02, 0.05)),
                 "Extra oxygen needed for a kernel to survive [M]."),
        authored("ign.sd_a", &[], NoiseParameter, Some("mole fraction"), Some((0.001, 0.03)),
                 "Run-to-run scatter of ignition."),
        authored("ign.t_req", &[2], PhysicalParameter, Some("s"), Some((0.1, 120.0)),
                 "Igniter on-time at which half the attempts light [M]."),
        authored("ign.sd_lnt", &[], NoiseParameter, Some("ln"), Some((0.1, 2.0)),
                 "Spread of that on-time, in ln of the time."),
        authored("ext.tau", &[2, 2], PhysicalParameter, Some("s"), Some((0.1, 300.0)),
                 "Time for a flame outside the boundary to go out [M, B]: B = 0 radiative quench, 1 blow-off."),
        authored("ext.width", &[], NoiseParameter, Some("mole fraction"), Some((0.0005, 0.01)),
                 "Smoothing of the inside/outside switch: the rate carries Phi(-margin / width)."),
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
        constant_vec("quad.z", &Q41_Z, "1", "Equal-probability nodes of a standard normal variable, the quantiles at (k - 0.5) / 41 for k = 1..41: where the true boundary lies, in units of its sd s."),
        constant_vec("quad.w", &Q41_W, "1", "Weights of the equal-probability nodes, 1/41 each."),
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
                          the probability that the sample ignites, that a flame is alive, that it has \
                          been quenched, and that its tip has reached the end of the sample. The model \
                          works in sample widths; metres follow from the sample width of the prefix. For \
                          opposed flow or a material other than the cotton-fibreglass fabric only the \
                          event probabilities are given."
                .into(),
            excludes: vec![
                "the flow history: the operator's steps are an input, known up to each forecast time".into(),
                "the tip of an opposed-flow flame, thick fuels, and the tip on materials other than the cotton-fibreglass fabric".into(),
                "a metric position without a sample width in metres from the data lane".into(),
                "temperature, heat flux, species or soot inferred from image intensity".into(),
                "a calibrated posterior: the spread of the tip is assumed, the width of the boundary is the author's propagated uncertainty".into(),
            ],
        },
        features: vec![
            feature("u", "1", "control:flow_velocity", "U / norm.u_ref_cm_s - 1, U in cm/s, not below norm.u_min_cm_s @1"),
            feature("c_o2", "1", "condition:x_o2", "(x - norm.o2_ref) / norm.o2_scale @1"),
            feature("c_w", "1", "condition:width", "ln(W / W_ref) @1"),
            feature("tip", "sample width", "observation:front_position/tip@prefix", "position / W; stills after the igniter is off and at or before the cutoff @1"),
            feature("base", "sample width", "observation:front_position/base@prefix", "position / W; the same stills @1"),
            feature("p", "atm", "condition:pressure", "Pa / 101325; 1 atm when the prefix has no pressure @1"),
            feature("dir", "1", "condition:flow_direction", "0 concurrent, 1 opposed; concurrent when absent @1"),
            feature("mat", "1", "condition:material", "0 SIBAL fabric, 1 PMMA sheet 1 mm; 0 when absent @1"),
        ],
        domain: vec![],
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::front5::forecast".into(),
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
             cut at the end of the sample when its length is known. Reached the end by t: the \
             probability that the uncut tip was at or beyond the end at t - end.linger_s. \
             Boundary law (dossier phys_quench_w0.md 6.3): W = width ratio x norm.w_ref_m in cm; U_e = \
             U + us (opposed), sqrt(U^2 + us^2) (concurrent), refused when not positive; A = ar + aw / W; \
             X_lim = x0 + cp ln(1/p) + sb U_e + A / (p U_e); s^2 = sd_x0^2 + (sd_cp ln(1/p))^2 + (sd_sb \
             U_e)^2 + (sd_ar^2 + (sd_aw / W)^2) / (p U_e)^2; P_sus = Phi((X - X_lim) / s); P_ign = Phi((X - \
             X_lim(U_ign) - dx) / sqrt(s(U_ign)^2 + sd_a^2)) Phi(ln(t_on / t_req) / sd_lnt), U_ign the flow \
             when the igniter is switched on, t_on the time it was on. Nodes e_k and weights w_k: \
             quad.z, quad.w (41 equal-probability nodes), the same node at every time. lit_k = Phi((X - X_lim(U_ign) - dx - \
             s(U_ign) e_k) / sd_a) Phi(ln(t_on / t_req) / sd_lnt); margin_k(t) = X - X_lim(U(t)) - s(U(t)) \
             e_k; w_r = r / (r + b) with r = max(A / (p U_e), 0), b = sb U_e, and 1/2 when r + b = 0; \
             rate_k = (w_r / tau[m,0] + (1 - w_r) / tau[m,1]) Phi(-margin_k / ext.width); Lambda_k(t) = \
             integral of rate_k from igniter-off to t, exact on the steps of the flow history. Without a \
             flame still in the prefix: P_alive(t) = sum w_k lit_k exp(-Lambda_k(t)), P_quenched_by(t) = \
             sum w_k lit_k (1 - exp(-Lambda_k(t))). With one, t_c the time of the last flame still: pi_k \
             proportional to w_k Phi((X - X_lim(U_ign) - dx - s(U_ign) e_k) / sd_a) exp(-Lambda_k(t_c)), \
             P_alive(t) = sum pi_k exp(-(Lambda_k(t) - Lambda_k(t_c))), P_quenched_by = 1 - P_alive. No \
             flame at t: 1 - P_alive(t). Rows: active = (1 - no flame)(1 - reached), extinction = no \
             flame, reached_end = (1 - no flame) reached. The tip forecast is given for the fabric \
             (material 0) in concurrent flow only; otherwise the rows are active = P_alive, extinction \
             = 1 - P_alive and the tip is refused. Nothing is fitted at request time. Uncertainty kind: \
             assumed_noise."
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
    law: Law<'a>,
}

/// The tensors of the boundary law, row-major: `[M]`, `[M, D]` or `[M, B]`.
struct Law<'a> {
    x0: &'a [f64],
    cp: &'a [f64],
    sb: &'a [f64],
    ar: &'a [f64],
    aw: &'a [f64],
    us: &'a [f64],
    sd_x0: &'a [f64],
    sd_cp: &'a [f64],
    sd_sb: &'a [f64],
    sd_ar: &'a [f64],
    sd_aw: &'a [f64],
    dx: &'a [f64],
    sd_a: f64,
    t_req: &'a [f64],
    sd_lnt: f64,
    tau: &'a [f64],
    width: f64,
    q_z: &'a [f64],
    q_w: &'a [f64],
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
            law: Law {
                x0: p.v("sus.x0"),
                cp: p.v("sus.cp"),
                sb: p.v("sus.sb"),
                ar: p.v("sus.ar"),
                aw: p.v("sus.aw"),
                us: p.v("sus.us"),
                sd_x0: p.v("sus.sd_x0"),
                sd_cp: p.v("sus.sd_cp"),
                sd_sb: p.v("sus.sd_sb"),
                sd_ar: p.v("sus.sd_ar"),
                sd_aw: p.v("sus.sd_aw"),
                dx: p.v("ign.dx"),
                sd_a: p.s("ign.sd_a"),
                t_req: p.v("ign.t_req"),
                sd_lnt: p.s("ign.sd_lnt"),
                tau: p.v("ext.tau"),
                width: p.s("ext.width"),
                q_z: p.v("quad.z"),
                q_w: p.v("quad.w"),
            },
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
        Laws {
            v: self.speed.eval(&c).max(self.v_min)
                * o.powf(self.speed_pow[0])
                * w.powf(self.speed_pow[1]),
            h_ss: self.over.eval(&c).max(0.0) * o.powf(self.over_pow[0]) * w.powf(self.over_pow[1]),
            tau_b: self.burn.eval(&c).exp(),
        }
    }

    /// The sample width of the boundary law, cm: the width ratio times the reference width.
    fn width_cm(&self, width_ratio: f64) -> f64 {
        width_ratio * self.w_ref_m * 100.0
    }
}

/// Direction of the flow relative to the spread of the flame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// The flow goes where the flame spreads (index 0 of axis D).
    Concurrent,
    /// The flow comes against the spreading flame (index 1 of axis D).
    Opposed,
}

impl Direction {
    pub fn index(self) -> usize {
        match self {
            Direction::Concurrent => 0,
            Direction::Opposed => 1,
        }
    }
}

/// The conditions the boundary law reads at one flow speed.
#[derive(Debug, Clone, Copy)]
pub struct Condition {
    pub x_o2: f64,
    pub p_atm: f64,
    pub direction: Direction,
    pub material: usize,
    pub width_cm: f64,
}

/// The boundary law at one condition and flow speed (6.3, "static").
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Limit {
    pub u_cm_s: f64,
    /// Effective speed U_e, cm/s.
    pub u_e_cm_s: f64,
    /// A = ar + aw / W.
    pub a: f64,
    /// 50 % oxygen limit and its sd, mole fraction.
    pub x_lim: f64,
    pub s: f64,
    /// Share of the radiative branch in the time to go out.
    pub w_r: f64,
    /// Probability that a flame is sustained at this condition, from the conditions alone.
    pub p_sus: f64,
}

impl Law<'_> {
    /// `None` when the effective speed is not positive (no flow and no self-spread speed): the
    /// radiative term of the limit is not defined there.
    fn limit(&self, c: &Condition, u_cm_s: f64) -> Option<Limit> {
        let (m, md) = (c.material, c.material * 2 + c.direction.index());
        let u_e = match c.direction {
            Direction::Opposed => u_cm_s + self.us[m],
            Direction::Concurrent => (u_cm_s * u_cm_s + self.us[m] * self.us[m]).sqrt(),
        };
        if !(u_e > 0.0 && u_e.is_finite()) {
            return None;
        }
        let ln_p = (1.0 / c.p_atm).ln();
        let a = self.ar[md] + self.aw[m] / c.width_cm;
        let pu = c.p_atm * u_e;
        let x_lim = self.x0[m] + self.cp[m] * ln_p + self.sb[md] * u_e + a / pu;
        let s = (self.sd_x0[m].powi(2)
            + (self.sd_cp[m] * ln_p).powi(2)
            + (self.sd_sb[md] * u_e).powi(2)
            + (self.sd_ar[md].powi(2) + (self.sd_aw[m] / c.width_cm).powi(2)) / pu.powi(2))
        .sqrt();
        let (r, b) = ((a / pu).max(0.0), self.sb[md] * u_e);
        let w_r = if r + b > 0.0 { r / (r + b) } else { 0.5 };
        Some(Limit {
            u_cm_s,
            u_e_cm_s: u_e,
            a,
            x_lim,
            s,
            w_r,
            p_sus: normal_cdf((c.x_o2 - x_lim) / s),
        })
    }

    /// The igniter's time factor Phi(ln(t_on / t_req) / sd_lnt).
    fn time_factor(&self, c: &Condition, t_on_s: f64) -> f64 {
        normal_cdf((t_on_s / self.t_req[c.material]).ln() / self.sd_lnt)
    }

    /// P_ign at the condition of ignition (closed form).
    fn p_ign(&self, c: &Condition, at_ign: &Limit, t_on_s: f64) -> f64 {
        normal_cdf(
            (c.x_o2 - at_ign.x_lim - self.dx[c.material])
                / (at_ign.s.powi(2) + self.sd_a.powi(2)).sqrt(),
        ) * self.time_factor(c, t_on_s)
    }

    /// The margin of the kernel at node `k`, in units of `sd_a`: lit_k = Phi(this) x time factor.
    fn kernel_z(&self, c: &Condition, at_ign: &Limit, k: usize) -> f64 {
        (c.x_o2 - at_ign.x_lim - self.dx[c.material] - at_ign.s * self.q_z[k]) / self.sd_a
    }

    /// The rate at which a flame at node `k` goes out under `limit`, 1/s.
    fn rate(&self, c: &Condition, limit: &Limit, k: usize) -> f64 {
        let margin = c.x_o2 - limit.x_lim - limit.s * self.q_z[k];
        let m = c.material;
        (limit.w_r / self.tau[m * 2] + (1.0 - limit.w_r) / self.tau[m * 2 + 1])
            * normal_cdf(-margin / self.width)
    }
}

/// ln Phi(z), finite for every finite z: the direct value while it is representable, the
/// asymptotic series of the lower tail below -30 (relative error below 2e-10 there).
fn ln_normal_cdf(z: f64) -> f64 {
    if z > -30.0 {
        normal_cdf(z).ln()
    } else {
        let z2 = z * z;
        -0.5 * z2 - (-z).ln() - 0.5 * (2.0 * std::f64::consts::PI).ln()
            + (1.0 - 1.0 / z2 + 3.0 / (z2 * z2) - 15.0 / (z2 * z2 * z2)).ln()
    }
}

/// The condition-level laws at one flow speed (reported in the diagnostics).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Laws {
    /// Spread rate, sample widths per second.
    pub v: f64,
    /// Steady overhang of the tip, sample widths.
    pub h_ss: f64,
    /// Burn-through time, s.
    pub tau_b: f64,
}

/// What the forward needs of one burn, in sample widths and seconds after the igniter is off: the
/// input of `front.rs`, and the three conditions the boundary law adds.
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
    /// Pressure, atm (1.0 when the record gives none).
    pub p_atm: f64,
    /// Flow direction (concurrent when the record gives none).
    pub direction: Direction,
    /// Material index of the boundary law (0, the SIBAL fabric, when the record gives none).
    pub material: usize,
}

impl FrontInput {
    /// The input of `front.rs` with the defaults of the three added conditions: 1 atm, concurrent
    /// flow, the SIBAL fabric.
    pub fn from_first(i: crate::front::FrontInput) -> Self {
        FrontInput {
            cutoff_s: i.cutoff_s,
            igniter_on_s: i.igniter_on_s,
            x_o2: i.x_o2,
            width_ratio: i.width_ratio,
            window_w: i.window_w,
            flow: i.flow,
            tip: i.tip,
            base: i.base,
            times: i.times,
            p_atm: 1.0,
            direction: Direction::Concurrent,
            material: 0,
        }
    }
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

/// Whether there is a flame: the boundary law on one burn.
#[derive(Debug, Clone, Serialize)]
pub struct Boundary {
    pub p_atm: f64,
    pub direction: Direction,
    pub material: usize,
    pub width_cm: f64,
    /// Igniter on-time, s, and the limit at the flow in force when it was switched on.
    pub t_on_s: f64,
    pub at_ignition: Limit,
    /// The limit at the flow of the cutoff.
    pub at_cutoff: Limit,
    /// P(ignites) from the conditions alone (closed form of 6.3).
    pub p_ignites: f64,
    /// The same by the nodes, sum w_k lit_k: what P_alive + P_quenched_by add up to without
    /// a flame in the prefix.
    pub p_lit_nodes: f64,
    /// Time of the last flame still of the prefix, when there is one: the forecast is conditional on
    /// a flame alive then.
    pub flame_seen_at_s: Option<f64>,
    /// Weights of the nodes the forecast starts from: w_k lit_k without a flame in the prefix (they
    /// sum to `p_lit_nodes`), the normalised pi_k with one.
    pub node_weights: Vec<f64>,
    /// P(flame alive at t) and P(ignited and quenched by t), at the forecast times.
    pub p_alive: Vec<f64>,
    pub p_quenched_by: Vec<f64>,
}

/// The numbers of one forecast, in sample widths.
#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    pub encoded: Encoded,
    pub times_s: Vec<f64>,
    pub levels: Vec<f64>,
    /// `[level][time]` tip position, sample widths.
    pub quantiles: Vec<Vec<f64>>,
    pub mean: Vec<f64>,
    /// Median base position (the front one burn-through time earlier), sample widths.
    pub base_median: Vec<f64>,
    /// Probability that there is no flame at t (never lit, or gone out), and that its tip has
    /// reached the end.
    pub p_quenched: Vec<f64>,
    pub p_reached_end: Vec<f64>,
    pub boundary: Boundary,
}

/// What the forward returns for a burn it can run.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Outcome {
    /// The tip and the event probabilities.
    Full(Forecast),
    /// The event probabilities alone: the tip law does not hold here (opposed flow, or a material
    /// other than the fabric), and its forecast is refused.
    EventsOnly {
        times_s: Vec<f64>,
        /// Probability that there is no flame at t.
        p_quenched: Vec<f64>,
        boundary: Boundary,
        tip_refusal: Support,
    },
}

impl Outcome {
    pub fn boundary(&self) -> &Boundary {
        match self {
            Outcome::Full(f) => &f.boundary,
            Outcome::EventsOnly { boundary, .. } => boundary,
        }
    }
    pub fn p_quenched(&self) -> &[f64] {
        match self {
            Outcome::Full(f) => &f.p_quenched,
            Outcome::EventsOnly { p_quenched, .. } => p_quenched,
        }
    }
    pub fn times_s(&self) -> &[f64] {
        match self {
            Outcome::Full(f) => &f.times_s,
            Outcome::EventsOnly { times_s, .. } => times_s,
        }
    }
}

/// The law path of one burn on a regular grid from the igniter being switched on.
struct Path {
    t0: f64,
    dt: f64,
    x: Vec<f64>,
    h: Vec<f64>,
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

/// Integral of each node's rate of going out from igniter-off (time 0) to each of `times`, exact on
/// the step function of the flow; `Err` when the limit is not defined at a flow of the history.
fn node_integrals(
    law: &Law<'_>,
    c: &Condition,
    flow: &[(f64, f64)],
    times: &[f64],
) -> Result<Vec<[f64; NODES]>, Support> {
    let t_max = times.iter().copied().fold(0.0, f64::max);
    // segment starts: igniter-off, then every flow entry after it and before the last time
    let mut starts = vec![0.0];
    starts.extend(flow.iter().map(|s| s.0).filter(|t| *t > 0.0 && *t < t_max));
    let mut rates = Vec::with_capacity(starts.len());
    for &a in &starts {
        let u = flow_at(flow, a);
        let Some(limit) = law.limit(c, u) else {
            return Err(refuse(
                SupportGate::Condition,
                format!(
                    "the effective flow speed is zero at {a} s (flow {u} cm/s and no self-spread \
                     speed of material {}): the oxygen limit is not defined there",
                    c.material
                ),
            ));
        };
        let mut r = [0.0; NODES];
        for (k, rk) in r.iter_mut().enumerate() {
            *rk = law.rate(c, &limit, k);
        }
        rates.push(r);
    }
    // the integral at each segment start
    let mut at_start = vec![[0.0; NODES]; starts.len()];
    for i in 1..starts.len() {
        for k in 0..NODES {
            at_start[i][k] = at_start[i - 1][k] + rates[i - 1][k] * (starts[i] - starts[i - 1]);
        }
    }
    Ok(times
        .iter()
        .map(|&t| {
            let t = t.max(0.0);
            let i = starts.partition_point(|a| *a <= t).saturating_sub(1);
            let mut v = [0.0; NODES];
            for (k, vk) in v.iter_mut().enumerate() {
                *vk = at_start[i][k] + rates[i][k] * (t - starts[i]);
            }
            v
        })
        .collect())
}

/// The boundary law on one burn: P(ignites), and P(flame alive) and P(quenched by) at the forecast
/// times, given whether the prefix shows a flame (the time of its last flame still).
fn boundary(
    wt: &Weights<'_>,
    inp: &FrontInput,
    flame_seen_at_s: Option<f64>,
) -> Result<Boundary, Support> {
    let law = &wt.law;
    let c = Condition {
        x_o2: inp.x_o2,
        p_atm: inp.p_atm,
        direction: inp.direction,
        material: inp.material,
        width_cm: wt.width_cm(inp.width_ratio),
    };
    let undefined = |what: &str, u: f64| {
        refuse(
            SupportGate::Condition,
            format!(
                "the effective flow speed is zero {what} (flow {u} cm/s and no self-spread speed of \
                 material {}): the oxygen limit is not defined there",
                c.material
            ),
        )
    };
    let u_ign = flow_at(&inp.flow, inp.igniter_on_s);
    let at_ign = law
        .limit(&c, u_ign)
        .ok_or_else(|| undefined("when the igniter is switched on", u_ign))?;
    let u_cut = flow_at(&inp.flow, inp.cutoff_s);
    let at_cut = law
        .limit(&c, u_cut)
        .ok_or_else(|| undefined("at the cutoff", u_cut))?;
    let t_on = -inp.igniter_on_s;
    let time_factor = law.time_factor(&c, t_on);
    let mut lam_times = inp.times.clone();
    if let Some(t_c) = flame_seen_at_s {
        lam_times.push(t_c);
    }
    let lam = node_integrals(law, &c, &inp.flow, &lam_times)?;
    let nt = inp.times.len();
    let lit: Vec<f64> = (0..NODES)
        .map(|k| law.q_w[k] * normal_cdf(law.kernel_z(&c, &at_ign, k)) * time_factor)
        .collect();
    let (mut p_alive, mut p_quenched_by) = (vec![0.0; nt], vec![0.0; nt]);
    let node_weights = match flame_seen_at_s {
        None => {
            for j in 0..nt {
                for k in 0..NODES {
                    p_alive[j] += lit[k] * (-lam[j][k]).exp();
                    p_quenched_by[j] += lit[k] * -(-lam[j][k]).exp_m1();
                }
                // the weights sum to one to within the rounding of their sum
                p_alive[j] = p_alive[j].min(1.0);
                p_quenched_by[j] = p_quenched_by[j].min(1.0);
            }
            lit.clone()
        }
        Some(_) => {
            // a flame alive at t_c: weights of the nodes by what that says, in logs so that a node
            // the law gives little chance keeps its share (the time factor cancels)
            let lam_c = &lam[nt];
            let ln_w: Vec<f64> = (0..NODES)
                .map(|k| law.q_w[k].ln() + ln_normal_cdf(law.kernel_z(&c, &at_ign, k)) - lam_c[k])
                .collect();
            let top = ln_w.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let raw: Vec<f64> = ln_w.iter().map(|l| (l - top).exp()).collect();
            let total: f64 = raw.iter().sum();
            let pi: Vec<f64> = raw.iter().map(|r| r / total).collect();
            // divided by the sum of the weights in the same order, so that a flame no node loses
            // is alive with probability one exactly
            let sum_pi: f64 = pi.iter().sum();
            for j in 0..nt {
                for k in 0..NODES {
                    let d = (lam[j][k] - lam_c[k]).max(0.0);
                    p_alive[j] += pi[k] * (-d).exp();
                    p_quenched_by[j] += pi[k] * -(-d).exp_m1();
                }
                p_alive[j] = (p_alive[j] / sum_pi).min(1.0);
                p_quenched_by[j] = (p_quenched_by[j] / sum_pi).min(1.0);
            }
            pi
        }
    };
    Ok(Boundary {
        p_atm: c.p_atm,
        direction: c.direction,
        material: c.material,
        width_cm: c.width_cm,
        t_on_s: t_on,
        at_ignition: at_ign,
        at_cutoff: at_cut,
        p_ignites: law.p_ign(&c, &at_ign, t_on),
        p_lit_nodes: lit.iter().sum(),
        flame_seen_at_s,
        node_weights,
        p_alive,
        p_quenched_by,
    })
}

/// The forward on one burn in sample widths: `Err` is a scientific refusal of the whole forecast;
/// [`Outcome::EventsOnly`] refuses the tip alone.
pub(crate) fn run(wt: &Weights<'_>, inp: &FrontInput, levels: &[f64]) -> Result<Outcome, Support> {
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
    if !(inp.p_atm > 0.0 && inp.p_atm.is_finite()) {
        return bad(
            SupportGate::Condition,
            format!("pressure {} atm is not positive", inp.p_atm),
        );
    }
    if inp.material >= MATERIALS.len() {
        return bad(
            SupportGate::Condition,
            format!(
                "material index {} has no row in the boundary law (0 {}, 1 {})",
                inp.material, MATERIALS[0], MATERIALS[1]
            ),
        );
    }
    // whether the prefix shows a flame: the time of its last flame still
    let seen = inp
        .tip
        .iter()
        .filter(|o| o.0 >= 0.0 && o.0 <= tau && o.1.is_finite())
        .map(|o| o.0)
        .fold(None, |m: Option<f64>, t| Some(m.map_or(t, |m| m.max(t))));
    let events = boundary(wt, inp, seen)?;
    if inp.direction != Direction::Concurrent || inp.material != 0 {
        return Ok(Outcome::EventsOnly {
            times_s: inp.times.clone(),
            p_quenched: events.p_alive.iter().map(|a| 1.0 - a).collect(),
            boundary: events,
            tip_refusal: refuse(
                SupportGate::Condition,
                format!(
                    "the tip law of this model is written for the {} in concurrent flow; this burn \
                     is {} flow over the {}: its tip is not forecast, only whether it ignites and \
                     stays lit",
                    MATERIALS[0],
                    match inp.direction {
                        Direction::Concurrent => "concurrent",
                        Direction::Opposed => "opposed",
                    },
                    MATERIALS[inp.material]
                ),
            ),
        });
    }
    let t_max = inp.times.iter().copied().fold(tau, f64::max);
    // the law path from the igniter being switched on
    let t0 = inp.igniter_on_s;
    let n = (((t_max - t0) / wt.dt).ceil() as usize).max(1) + 1;
    let mut path = Path {
        t0,
        dt: wt.dt,
        x: vec![0.0; n + 1],
        h: vec![0.0; n + 1],
    };
    for i in 0..n {
        let t_mid = t0 + (i as f64 + 0.5) * wt.dt;
        let l = wt.laws(flow_at(&inp.flow, t_mid), inp.x_o2, inp.width_ratio);
        let g = if t_mid < t0 + wt.ign_delay {
            0.0
        } else if t_mid < 0.0 {
            wt.ign_gain
        } else {
            1.0
        };
        path.x[i + 1] = path.x[i] + g * l.v * wt.dt;
        path.h[i + 1] = if g > 0.0 {
            l.h_ss + (path.h[i] - l.h_ss) * (-wt.dt / wt.over_tau).exp()
        } else {
            path.h[i]
        };
    }
    let laws_cut = wt.laws(flow_at(&inp.flow, tau), inp.x_o2, inp.width_ratio);
    let c_o2 = (inp.x_o2 - wt.o2_ref) / wt.o2_scale;
    let c_w = inp.width_ratio.ln();
    let far2 = (wt.s_far[0] * c_o2).powi(2) + (wt.s_far[1] * c_w).powi(2);
    // what the prefix says
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
    let enc = if let (Some(first), Some(last)) = (tip.first(), tip.last()) {
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
                    .laws(flow_at(&inp.flow, bl.0), inp.x_o2, inp.width_ratio)
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
            let k = inp.flow.partition_point(|s| s.0 <= t_e).saturating_sub(1);
            let u_e = inp.flow[k].1;
            inp.flow[..=k]
                .iter()
                .rposition(|s| (s.1 - u_e).abs() >= wt.change)
                .map_or(t0, |i| inp.flow[i + 1].0.max(t0))
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
        Encoded {
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
        }
    } else {
        Encoded {
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
        }
    };
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
    let (mut p_q, mut p_end) = (vec![0.0; nt], vec![0.0; nt]);
    let cut = |v: f64| inp.window_w.map_or(v, |w| v.min(w));
    for (j, &t) in inp.times.iter().enumerate() {
        let (m, sd) = tip_at(t);
        for (l, p) in levels.iter().enumerate() {
            quantiles[l][j] = cut(m + normal_quantile(*p) * sd);
        }
        mean[j] = cut(m);
        let tau_b = wt
            .laws(flow_at(&inp.flow, t), inp.x_o2, inp.width_ratio)
            .tau_b;
        base_median[j] = (enc.x_end_w + big_e * (path.x(t - tau_b) - path.x(enc.t_end_s))).max(0.0);
        p_q[j] = 1.0 - events.p_alive[j];
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
    Ok(Outcome::Full(Forecast {
        encoded: enc,
        times_s: inp.times.clone(),
        levels: levels.to_vec(),
        quantiles,
        mean,
        base_median,
        p_quenched: p_q,
        p_reached_end: p_end,
        boundary: events,
    }))
}

/// The forward of a release on one burn in sample widths (the development route of the track files
/// of the measurement study). Nothing metric is claimed.
pub fn forecast_track(
    spec: &ModelSpec,
    release: &LoadedRelease,
    input: &FrontInput,
    levels: &[f64],
) -> Result<Result<Outcome, Support>, Refusal> {
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

/// What the boundary law of a release says at one condition and flow speed, from the conditions
/// alone (inspection; no data): the limit, its sd and P_sus; and P_ign for an igniter on `t_on_s`
/// seconds at this flow. `Ok(None)` when the effective speed is not positive there.
pub fn limit_of(
    spec: &ModelSpec,
    release: &LoadedRelease,
    condition: &Condition,
    u_cm_s: f64,
    t_on_s: f64,
) -> Result<Option<(Limit, f64)>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params);
    if condition.material >= MATERIALS.len() {
        return Err(Refusal::unsupported(
            "condition",
            format!(
                "material index {} has no row in the boundary law",
                condition.material
            ),
        ));
    }
    Ok(wt
        .law
        .limit(condition, u_cm_s)
        .map(|l| (l, wt.law.p_ign(condition, &l, t_on_s))))
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
    pub outcome: Outcome,
    pub width_m: f64,
}

/// An optional numeric condition of the prefix: `Ok(None)` when absent, a refusal reason when
/// present but unresolved or not finite.
fn optional_condition(prefix: &PrefixBundle, key: &str) -> Result<Option<f64>, String> {
    if prefix.known_conditions.contains_key(key) {
        common::condition(prefix, key).map(Some)
    } else {
        Ok(None)
    }
}

/// Strict forward of a `bass_front.wsp5` release on an admitted prefix with metric observations:
/// `Ok(Err(support))` is a scientific refusal, `Err(refusal)` a release that cannot be run at all.
///
/// The prefix must carry: conditions `x_o2` and `width` (m), optionally `length` (m); control
/// stages `flow_velocity` (m/s, a step function in time) and one `igniter` stage with its end time;
/// observations of `front_position` in metres on channels `tip` and `base`. Optional conditions:
/// `pressure` (Pa; 1 atm when absent), `flow_direction` (0 concurrent, 1 opposed; concurrent when
/// absent), `material` (0 SIBAL fabric, 1 PMMA sheet 1 mm; 0 when absent); present but unusable is
/// a refusal, never the default. Times in the prefix are seconds since its time origin; the
/// forward shifts them so the igniter is off at zero.
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
    let mut optional = |k: &str| match optional_condition(prefix, k) {
        Ok(v) => v,
        Err(e) => {
            reasons.push(e);
            None
        }
    };
    let (pressure, flow_direction, material_index) = (
        optional("pressure"),
        optional("flow_direction"),
        optional("material"),
    );
    let p_atm = pressure.map_or(1.0, |pa| pa / PA_PER_ATM);
    let direction = match flow_direction {
        None => Some(Direction::Concurrent),
        Some(d) if d == 0.0 => Some(Direction::Concurrent),
        Some(d) if d == 1.0 => Some(Direction::Opposed),
        Some(d) => {
            reasons.push(format!(
                "flow_direction {d} is neither 0 (concurrent) nor 1 (opposed)"
            ));
            None
        }
    };
    let material = match material_index {
        None => Some(0),
        Some(m) if m == 0.0 || m == 1.0 => Some(m as usize),
        Some(m) => {
            reasons.push(format!(
                "material {m} is neither 0 ({}) nor 1 ({})",
                MATERIALS[0], MATERIALS[1]
            ));
            None
        }
    };
    let (Some(direction), Some(material), true) = (direction, material, reasons.is_empty()) else {
        return Ok(Err(Support {
            admitted: false,
            gate: SupportGate::Condition,
            reasons,
        }));
    };
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
        p_atm,
        direction,
        material,
    };
    let levels = common::levels_of(settings);
    Ok(run(&wt, &input, &levels).map(|mut o| {
        // back to the prefix clock
        match &mut o {
            Outcome::Full(f) => f.times_s.iter_mut().for_each(|t| *t += t_off),
            Outcome::EventsOnly { times_s, .. } => times_s.iter_mut().for_each(|t| *t += t_off),
        }
        if let Some(t) = match &mut o {
            Outcome::Full(f) => &mut f.boundary.flame_seen_at_s,
            Outcome::EventsOnly { boundary, .. } => &mut boundary.flame_seen_at_s,
        } {
            *t += t_off;
        }
        Metric {
            outcome: o,
            width_m: width,
        }
    }))
}

fn body(m: Metric) -> Body {
    let w = m.width_m;
    let scale = |v: &[f64]| v.iter().map(|x| x * w).collect::<Vec<f64>>();
    match m.outcome {
        Outcome::Full(f) => {
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
                "boundary": f.boundary,
                "sample_width_m": w,
                "tip_quantiles_sample_widths": f.quantiles,
                "base_median_m": scale(&f.base_median),
                "observable": "downstream tip of the visible flame along the sample; not a pyrolysis front, a temperature or a species",
                "extinction_row": "no flame at t: never lit, or gone out",
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
        Outcome::EventsOnly {
            times_s,
            p_quenched,
            boundary,
            tip_refusal,
        } => {
            let table = vec![p_quenched.iter().map(|q| 1.0 - q).collect(), p_quenched];
            let diagnostics = json!({
                "boundary": boundary,
                "tip_forecast_refused": tip_refusal,
                "extinction_row": "no flame at t: never lit, or gone out",
                "conditional_on": "the flow history up to each forecast time",
                "uncertainty_kind": "assumed_noise",
            });
            Body {
                kind: None,
                quantity: Some(Quantity::FrontPosition),
                times_s,
                event_categories: CATEGORIES_EVENTS.iter().map(|s| s.to_string()).collect(),
                event_cdf: table,
                diagnostics,
                ..Body::default()
            }
        }
    }
}

/// The artifact of a forecast or of a scientific refusal (target quantity kept, no numbers). A burn
/// whose tip the model does not forecast gives an artifact with the event rows alone and the tip's
/// refusal in its diagnostics.
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

/// `bass_front.wsp5` behind the common expert interface.
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
                    "flow_direction",
                    "material",
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
        bail!("bass_front.wsp5 has no fitting procedure: its release is assembled from a literal patch (`ignis wsp assemble`)")
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
        bail!("bass_front.wsp5 declares an assumed spread, not a likelihood")
    }
}

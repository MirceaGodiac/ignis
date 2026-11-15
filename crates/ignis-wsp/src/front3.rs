//! `bass_front.wsp3`: the frozen front2 evolution with an optional steady oxygen-supply speed.
//!
//! Per-face balance: v (m_total / faces) sigma = rho Y U delta, with
//! delta^2 = nu L/U + (alpha/U)^2 and the STEADY closure L = v tau.
//! Therefore v^2 = A^2 (nu v tau U + alpha^2), A = faces rho Y/(m_total sigma).
//! The positive root is evaluated in SI before division by the authored sample width.
//! Oxygen mass fraction uses the actual oxygen mole fraction and two authored molar masses.
//! There is no empirical minimum flow, oxygen-reference normalization, or speed power on this branch.
//!
//! This is a capture-layer assumption, not a derived transient or extinction model. The retained
//! overhang relaxation, visible-base delay, ignition multiplier, prefix-derived offset and hazard
//! are independent closures. In particular oxygen supply does not establish flame survival.
//! The finite strip / side-entrainment correction is not supplied by the two-dimensional budget.
//! All new numbers, including the face factor and reference width, must be authored even when the
//! branch is off. oxygen.mix=0 preserves the predecessor arithmetic and its input/output types.
//! `oxygen.member_weight` mixes persistent position distributions, not speeds or quantiles.
//! Any active oxygen scenario also activates a coherent first-event closure for every scenario:
//! fixed-grid endpoint exceedance is an assumed first-passage surrogate, converted to competing
//! hazards with the inherited quench law. A CDF reaching one is a right-edge end atom. Initial
//! end mass at cutoff is assigned before future quench exposure. Joint events therefore reinterpret
//! the predecessor-speed scenario; they do not average its nonmonotone legacy event rows.
//! mix=0 or member_weight=0 keeps the full predecessor compatibility behavior.

use anyhow::bail;
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

pub const SPEC_ID: &str = "bass_front-wsp-v3";
pub const MODEL_NAME: &str = "bass_front.wsp3";
pub const FORWARD_VERSION: &str = "3";
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

/// Authored, constant-reference-state closure properties, always in SI.
#[derive(Clone, Copy)]
struct OxygenSupply {
    rho_kg_m3: f64,
    nu_m2_s: f64,
    alpha_m2_s: f64,
    molar_o2_kg_mol: f64,
    molar_carrier_kg_mol: f64,
    fuel_areal_kg_m2: f64,
    faces: f64,
    stoich_kg_o2_kg_fuel: f64,
    residence_s: f64,
    reference_width_m: f64,
}

impl OxygenSupply {
    fn bind(p: &Params<'_>) -> Self {
        Self {
            rho_kg_m3: p.s("oxygen.rho_kg_m3"),
            nu_m2_s: p.s("oxygen.nu_m2_s"),
            alpha_m2_s: p.s("oxygen.alpha_m2_s"),
            molar_o2_kg_mol: p.s("oxygen.molar_o2_kg_mol"),
            molar_carrier_kg_mol: p.s("oxygen.molar_carrier_kg_mol"),
            fuel_areal_kg_m2: p.s("oxygen.fuel_areal_kg_m2"),
            faces: p.s("oxygen.faces"),
            stoich_kg_o2_kg_fuel: p.s("oxygen.stoich_kg_o2_kg_fuel"),
            residence_s: p.s("oxygen.residence_s"),
            reference_width_m: p.s("oxygen.reference_width_m"),
        }
    }

    fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            [
                self.rho_kg_m3,
                self.nu_m2_s,
                self.alpha_m2_s,
                self.molar_o2_kg_mol,
                self.molar_carrier_kg_mol,
                self.fuel_areal_kg_m2,
                self.faces,
                self.stoich_kg_o2_kg_fuel,
                self.residence_s,
                self.reference_width_m
            ]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0),
            "oxygen-supply properties must all be finite and positive"
        );
        Ok(())
    }

    fn speed_m_s(&self, u_m_s: f64, x_o2: f64) -> f64 {
        // Endpoints are exact and avoid 0 * infinity for zero oxygen.
        if x_o2 == 0.0 {
            return 0.0;
        }
        let y_o2 = x_o2 * self.molar_o2_kg_mol
            / (x_o2 * self.molar_o2_kg_mol + (1.0 - x_o2) * self.molar_carrier_kg_mol);
        let a_supply = self.faces * self.rho_kg_m3 * y_o2
            / (self.fuel_areal_kg_m2 * self.stoich_kg_o2_kg_fuel);
        let zero_flow = a_supply * self.alpha_m2_s;
        if u_m_s == 0.0 {
            return zero_flow;
        }
        // a has units m/s. hypot(a/2, A alpha) avoids squaring a large a.
        let half_a = 0.5 * a_supply * a_supply * self.nu_m2_s * self.residence_s * u_m_s;
        half_a + half_a.hypot(zero_flow)
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
    tensors.push(authored(
        "oxygen.mix", &[], Weight, Some("1"), Some((0.0, 1.0)),
        "Share of the steady oxygen-supply speed: 0 is exactly front2, 1 bypasses its speed network and powers. Overhang, prefix adjustment, burnout delay and event laws remain separate inherited closures.",
    ));
    tensors.push(authored(
        "oxygen.member_weight", &[], Weight, Some("1"), Some((0.0, 1.0)),
        "Persistent oxygen-law scenario weight: 1 returns that scenario; 0 returns the full predecessor exactly; intermediate values mix its full position CDF with the predecessor-speed scenario, never quantiles. When both this and oxygen.mix are positive, BOTH scenarios use the new coherent first-event closure. Thus joint event CDFs reinterpret the predecessor hazard and are not a mixture of its old event artifact rows. oxygen.mix=0 remains exact predecessor.",
    ));
    for (name, unit, description) in [
        (
            "oxygen.rho_kg_m3",
            "kg/m3",
            "Gas density at the authored reference state; no temperature or pressure adaptation is inferred.",
        ),
        (
            "oxygen.nu_m2_s",
            "m2/s",
            "Kinematic viscosity in the capture-layer closure; author must disclose reference state and any use of hot-layer transport with ambient density.",
        ),
        (
            "oxygen.alpha_m2_s",
            "m2/s",
            "Thermal diffusivity in the capture-layer closure, not an inferred oxygen molecular diffusivity.",
        ),
        (
            "oxygen.molar_o2_kg_mol",
            "kg/mol",
            "Molar mass of oxygen for converting input oxygen mole fraction to mass fraction.",
        ),
        (
            "oxygen.molar_carrier_kg_mol",
            "kg/mol",
            "Effective molar mass of the non-oxygen carrier; a fixed binary-mixture approximation.",
        ),
        (
            "oxygen.fuel_areal_kg_m2",
            "kg/m2",
            "TOTAL combustible fuel areal loading across the exposed faces; per-face loading equals this divided by oxygen.faces. Inert fiberglass mass is excluded.",
        ),
        (
            "oxygen.faces",
            "1",
            "Explicit effective exposed-face count in the oxygen budget. Each face supplies rho Y U delta; dividing total fuel loading by this gives the per-face balance.",
        ),
        (
            "oxygen.stoich_kg_o2_kg_fuel",
            "kg/kg",
            "Oxygen mass required per combustible fuel mass under the declared stoichiometric chemistry.",
        ),
        (
            "oxygen.residence_s",
            "s",
            "Authored steady flame residence time in L = v tau; independent of the inherited visible-base burnout delay.",
        ),
        (
            "oxygen.reference_width_m",
            "m",
            "Physical reference width for converting m/s into widths/s. For an active oxygen branch width_ratio is actual width divided by this reference, including in the inherited empirical laws. Set to norm.w_ref_m to preserve their width normalization.",
        ),
    ] {
        tensors.push(authored(
            name,
            &[],
            PhysicalParameter,
            Some(unit),
            Some((f64::MIN_POSITIVE, f64::MAX)),
            description,
        ));
    }
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
            entry_point: "ignis_wsp::front3::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(
            "The oxygen branch uses Y = x M_O2 / (x M_O2 + (1-x) M_carrier), A = faces rho Y / (fuel_areal stoich), a = A^2 nu U residence, b = (A alpha)^2 and v = (a + sqrt(a^2 + 4b))/2 in m/s. U is actual nonnegative flow without the empirical flow floor. Divide by oxygen.reference_width_m times width_ratio. oxygen.mix=0 preserves front2 exactly; 1 bypasses its speed laws; intermediate values linearly blend speeds. The steady closure is not transient conservation, and its positive zero-flow branch does not assert flame survival. All properties and reference states are authored. oxygen.member_weight mixes persistent complete position CDFs across predecessor-speed and oxygen-speed scenarios. With oxygen.mix>0 and member_weight>0, BOTH scenarios use the new coherent first-event model: endpoint Gaussian exceedance envelope sampled at cutoff + k*num.dt_s is an assumed first-passage surrogate; finite end and quench exposures are competing constant hazards within each interval; end CDF=1 is a right-boundary atom after interval quenching. Initial end mass is assigned at cutoff. The event formulas later in this note describe the predecessor compatibility path only, retained exactly when either switch is zero; joint events reinterpret its hazard and are not an average of old artifact event rows. Physical scenarios require flow coverage back to igniter-on. Time is seconds after the igniter is switched off. The law path is integrated from the \
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

#[derive(Clone, Copy)]
pub(crate) struct Weights<'a> {
    oxygen_mix: f64,
    oxygen_member_weight: f64,
    oxygen: OxygenSupply,
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
    legacy_w_ref_m: f64,
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
            oxygen_mix: p.s("oxygen.mix"),
            oxygen_member_weight: p.s("oxygen.member_weight"),
            oxygen: OxygenSupply::bind(p),
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
            w_ref_m: if p.s("oxygen.mix") > 0.0 && p.s("oxygen.member_weight") > 0.0 {
                p.s("oxygen.reference_width_m")
            } else {
                p.s("norm.w_ref_m")
            },
            legacy_w_ref_m: p.s("norm.w_ref_m"),
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
        let (mut v, mut v_zero_flow) = (0.0, 0.0);
        if self.oxygen_mix < 1.0 {
            // the speed network at the flow of the call and at zero flow speed (the flow feature at the
            // value it has for U = 0; the oxygen and width features unchanged), neither below v_min
            let n = self.speed.eval(&c).max(self.v_min);
            let c0 = [0.0_f64.max(self.u_min) / self.u_ref - 1.0, c[1], c[2]];
            let n0 = self.speed.eval(&c0).max(self.v_min);
            let (oxygen, width) = (o.powf(self.speed_pow[0]), w.powf(self.speed_pow[1]));
            // the rate of the first front forward, in its order of operations, and its zero-flow part
            v = n * oxygen * width;
            v_zero_flow = n0 * oxygen * width;
            if self.split > 0.0 {
                // Addition A: the zero-flow part carries its own oxygen power; the flow part may not
                // take the rate below the smallest zero-flow rate
                let still = n0 * o.powf(self.pow_i);
                let v_split =
                    (still + (n - n0) * oxygen).max(self.v_min * o.powf(self.pow_i)) * width;
                v = (1.0 - self.split) * v + self.split * v_split;
                v_zero_flow = (1.0 - self.split) * v_zero_flow + self.split * still * width;
            }
        }
        if self.oxygen_mix > 0.0 {
            let physical = self.oxygen.speed_m_s(u_cm_s / 100.0, x_o2);
            let still = self.oxygen.speed_m_s(0.0, x_o2);
            let width_m = self.oxygen.reference_width_m * width_ratio;
            if self.oxygen_mix == 1.0 {
                v = physical / width_m;
                v_zero_flow = still / width_m;
            } else {
                v = (1.0 - self.oxygen_mix) * v + self.oxygen_mix * physical / width_m;
                v_zero_flow =
                    (1.0 - self.oxygen_mix) * v_zero_flow + self.oxygen_mix * still / width_m;
            }
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

// Reuse the frozen predecessor's transport-only DTOs; no shared contract is changed.
pub use crate::front2::{Encoded, FlowPath, Forecast, FrontInput, Laws, Trace};

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
    if let Err(e) = wt.oxygen.validate() {
        return bad(SupportGate::Condition, e.to_string());
    }
    if !(wt.oxygen_mix.is_finite() && (0.0..=1.0).contains(&wt.oxygen_mix)) {
        return bad(SupportGate::Condition, "oxygen.mix must be in [0,1]".into());
    }
    if !(wt.oxygen_member_weight.is_finite() && (0.0..=1.0).contains(&wt.oxygen_member_weight)) {
        return bad(
            SupportGate::Condition,
            "oxygen.member_weight must be in [0,1]".into(),
        );
    }
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
    if wt.oxygen_mix > 0.0 && wt.oxygen_member_weight > 0.0 {
        if inp.flow[0].0 > inp.igniter_on_s {
            return bad(
                SupportGate::Condition,
                "physical scenario needs flow coverage back to igniter-on".into(),
            );
        }
        let width_m = wt.oxygen.reference_width_m * inp.width_ratio;
        if !(width_m.is_finite() && width_m > 0.0) {
            return bad(
                SupportGate::Condition,
                "physical width exceeds finite numerical representation".into(),
            );
        }
        for (_, u) in &inp.flow {
            for sign in [-1.0, 0.0, 1.0] {
                let flow = (u * (1.0 + sign * wt.flow_rel) + sign * wt.flow_abs).max(0.0);
                let speed = wt.oxygen.speed_m_s(flow / 100.0, inp.x_o2) / width_m;
                if !(speed.is_finite() && speed > 0.0) {
                    return bad(
                        SupportGate::Condition,
                        "oxygen law exceeds finite numerical representation".into(),
                    );
                }
            }
        }
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

/// A first-event state on the fixed event grid, independent of requested output times.
#[derive(Clone, Copy)]
struct EventMass {
    active: f64,
    quench: f64,
    end: f64,
}

impl EventMass {
    fn advance(self, quench_h: f64, end_h: Option<f64>, fraction: f64) -> Self {
        let mut next = self;
        match end_h {
            Some(end_h) => {
                let total = quench_h + end_h;
                if total > 0.0 {
                    let lost = self.active * -(-total * fraction).exp_m1();
                    let quenched = lost * (quench_h / total);
                    next.quench += quenched;
                    next.end += lost - quenched;
                    next.active = self.active * (-total * fraction).exp();
                }
            }
            None => {
                // A marginal CDF reaching exactly one is a right-edge stop atom, not an
                // infinite interval hazard: quenching before the atom retains its mass.
                next.quench += self.active * -(-quench_h * fraction).exp_m1();
                next.active = self.active * (-quench_h * fraction).exp();
                if fraction >= 1.0 {
                    next.end += next.active;
                    next.active = 0.0;
                }
            }
        }
        next
    }
}

/// The monotone envelope of endpoint Gaussian exceedance is an ASSUMED first-passage
/// surrogate, not a derived hitting-time law. Sample on cutoff + k*num.dt_s, convert finite
/// conditional end increments and quench exposure to competing constant hazards per interval,
/// and evaluate their exact probability update. CDF=1 is a right-edge atom. Endpoint mass
/// already present at cutoff is assigned to the end cause before subsequent quench exposure.
fn first_events(
    cutoff: f64,
    dt: f64,
    times: &[f64],
    end_cdf: impl Fn(f64) -> f64,
    quench_integral: impl Fn(f64) -> f64,
) -> Result<Vec<EventMass>, Support> {
    let bad = || {
        refuse(
            SupportGate::Condition,
            "nonfinite or invalid first-event input",
        )
    };
    let f0 = end_cdf(cutoff);
    if !(f0.is_finite() && (0.0..=1.0).contains(&f0)) {
        return Err(bad());
    }
    let initial = EventMass {
        active: 1.0 - f0,
        quench: 0.0,
        end: f0,
    };
    let max_time = times.iter().copied().fold(cutoff, f64::max);
    let steps = ((max_time - cutoff) / dt).ceil() as usize;
    let mut nodes = vec![initial];
    let mut exposures = Vec::with_capacity(steps);
    let (mut prev_f, mut prev_q) = (f0, quench_integral(cutoff));
    for k in 1..=steps {
        let t = cutoff + k as f64 * dt;
        let raw_f = end_cdf(t);
        let q = quench_integral(t);
        if !(raw_f.is_finite()
            && (0.0..=1.0).contains(&raw_f)
            && q.is_finite()
            && prev_q.is_finite()
            && q >= prev_q)
        {
            return Err(bad());
        }
        let f = raw_f.max(prev_f);
        let end_h = if f == 1.0 {
            None
        } else {
            Some(((-prev_f).ln_1p() - (-f).ln_1p()).max(0.0))
        };
        let quench_h = q - prev_q;
        nodes.push(nodes[k - 1].advance(quench_h, end_h, 1.0));
        exposures.push((quench_h, end_h));
        (prev_f, prev_q) = (f, q);
    }
    Ok(times
        .iter()
        .map(|&t| {
            let pos = (t - cutoff) / dt;
            let k = (pos.floor() as usize).min(steps);
            if k == steps {
                nodes[k]
            } else {
                nodes[k].advance(exposures[k].0, exposures[k].1, pos - k as f64)
            }
        })
        .collect())
}

/// One run of the forward on a checked burn under the flow history `flow`.
fn run_path(
    wt: &Weights<'_>,
    inp: &FrontInput,
    flow: &[(f64, f64)],
    levels: &[f64],
    coherent_events: bool,
) -> Result<Single, Support> {
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
    if coherent_events {
        let events = first_events(
            tau,
            wt.dt,
            &inp.times,
            |t| {
                inp.window_w.map_or(0.0, |w| {
                    let (m, sd) = tip_at((t - wt.linger).max(enc.t_end_s));
                    if sd > 0.0 {
                        normal_cdf((m - w) / sd)
                    } else {
                        f64::from(u8::from(m >= w))
                    }
                })
            },
            |t| path.lam(t),
        )?;
        for (j, event) in events.into_iter().enumerate() {
            p_q[j] = event.quench;
            // DTO coordinate only. body reconstructs the end CIF as (1-q)*p_end.
            p_end[j] = if event.quench < 1.0 {
                (event.end / (1.0 - event.quench)).min(1.0)
            } else {
                0.0
            };
        }
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
    Ok(Single {
        encoded: enc,
        tip_m,
        tip_sd,
        quantiles,
        mean,
        base_median,
        p_quenched: p_q,
        p_reached_end: p_end,
        trace,
    })
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
fn run_raw(wt: &Weights<'_>, inp: &FrontInput, levels: &[f64]) -> Result<Forecast, Support> {
    check(wt, inp)?;
    let coherent_events = wt.oxygen_mix > 0.0 && wt.oxygen_member_weight > 0.0;
    let mixed_laws = coherent_events && wt.oxygen_member_weight < 1.0;
    let mixed_flow = wt.flow_w[0] > 0.0 || wt.flow_w[2] > 0.0;
    let mut predecessor = *wt;
    predecessor.oxygen_mix = 0.0;
    predecessor.w_ref_m = wt.legacy_w_ref_m;
    if !mixed_flow && !mixed_laws {
        let scenario = if coherent_events { wt } else { &predecessor };
        let one = run_path(scenario, inp, &inp.flow, levels, coherent_events)?;
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
    let scenarios = if mixed_laws {
        vec![
            (1.0 - wt.oxygen_member_weight, predecessor),
            (wt.oxygen_member_weight, *wt),
        ]
    } else if coherent_events {
        vec![(1.0, *wt)]
    } else {
        vec![(1.0, predecessor)]
    };
    let flow_weights = if mixed_flow {
        wt.flow_w
    } else {
        &[0.0, 1.0, 0.0]
    };
    let total: f64 = flow_weights.iter().sum();
    let mut runs: Vec<(f64, f64, Single)> = Vec::with_capacity(3 * scenarios.len());
    // One member describes an entire trajectory, crossed with persistent flow nodes. The order
    // is predecessor-speed paths followed by oxygen-speed paths; sign remains the FLOW sign.
    for (law_weight, scenario) in scenarios {
        let mut scenario_input = inp.clone();
        if scenario.w_ref_m != wt.w_ref_m {
            scenario_input.width_ratio = inp.width_ratio * (wt.w_ref_m / scenario.w_ref_m);
        }
        for (&sign, &w) in FLOW_SIGNS.iter().zip(flow_weights) {
            if !(w > 0.0) {
                continue;
            }
            let one = if sign == 0.0 {
                run_path(
                    &scenario,
                    &scenario_input,
                    &inp.flow,
                    levels,
                    coherent_events,
                )?
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
                run_path(&scenario, &scenario_input, &flow, levels, coherent_events)?
            };
            runs.push((sign, law_weight * w / total, one));
        }
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

fn check_event_rows(times: &[f64], quench: &[f64], end_coordinate: &[f64]) -> Result<(), Support> {
    let mut order: Vec<_> = (0..times.len()).collect();
    order.sort_by(|&a, &b| times[a].total_cmp(&times[b]));
    let (mut prev_q, mut prev_end, mut prev_active) = (0.0, 0.0, 1.0);
    for i in order {
        let (q, coordinate) = (quench[i], end_coordinate[i]);
        let end = (1.0 - q) * coordinate;
        let active = (1.0 - q) * (1.0 - coordinate);
        if !(q.is_finite()
            && coordinate.is_finite()
            && (0.0..=1.0).contains(&q)
            && (0.0..=1.0).contains(&coordinate)
            && q + 1e-12 >= prev_q
            && end + 1e-12 >= prev_end
            && active <= prev_active + 1e-12
            && (active + q + end - 1.0).abs() <= 1e-12)
        {
            return Err(refuse(
                SupportGate::Condition,
                "first-event probabilities failed finite, mass or monotonicity invariant",
            ));
        }
        (prev_q, prev_end, prev_active) = (q, end, active);
    }
    Ok(())
}

pub(crate) fn run(wt: &Weights<'_>, inp: &FrontInput, levels: &[f64]) -> Result<Forecast, Support> {
    let forecast = run_raw(wt, inp, levels)?;
    if wt.oxygen_mix > 0.0 && wt.oxygen_member_weight > 0.0 {
        check_event_rows(
            &forecast.times_s,
            &forecast.p_quenched,
            &forecast.p_reached_end,
        )?;
        for path in &forecast.paths {
            check_event_rows(&forecast.times_s, &path.p_quenched, &path.p_reached_end)?;
        }
        let trace_finite = |trace: &Trace| {
            trace
                .flow_cm_s
                .iter()
                .chain(&trace.flow_factor)
                .chain(&trace.front_w)
                .chain(&trace.speed_w_s)
                .chain(&trace.speed_zero_flow_w_s)
                .chain(&trace.overhang_w)
                .chain(&trace.overhang_target_w)
                .chain(&trace.overhang_tau_s)
                .all(|v| v.is_finite())
        };
        if !forecast
            .quantiles
            .iter()
            .flatten()
            .chain(&forecast.mean)
            .chain(&forecast.base_median)
            .all(|v| v.is_finite())
            || !trace_finite(&forecast.trace)
            || forecast.paths.iter().any(|path| {
                !trace_finite(&path.trace)
                    || !path
                        .tip_median_w
                        .iter()
                        .chain(&path.tip_sd_w)
                        .chain(&path.base_median_w)
                        .all(|v| v.is_finite())
            })
        {
            return Err(refuse(
                SupportGate::Condition,
                "forecast exceeds finite numerical representation",
            ));
        }
    }
    Ok(forecast)
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

/// Inspect the oxygen-law scenario (or predecessor when either switch is zero).
/// A distribution committee has no single equivalent speed law; intermediate member weights
/// affect forecasts, not this condition-law inspection.
pub fn laws_of(
    spec: &ModelSpec,
    release: &LoadedRelease,
    u_cm_s: f64,
    x_o2: f64,
    width_ratio: f64,
) -> Result<Laws, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    check_forward(spec, release)?;
    let mut wt = Weights::bind(&params);
    wt.oxygen
        .validate()
        .map_err(|e| Refusal::failed("oxygen_properties", e.to_string()))?;
    if !(wt.oxygen_mix.is_finite() && (0.0..=1.0).contains(&wt.oxygen_mix)) {
        return Err(Refusal::failed("oxygen_mix", "oxygen.mix must be in [0,1]"));
    }
    if !(wt.oxygen_member_weight.is_finite() && (0.0..=1.0).contains(&wt.oxygen_member_weight)) {
        return Err(Refusal::failed(
            "oxygen_member_weight",
            "oxygen.member_weight must be in [0,1]",
        ));
    }
    if wt.oxygen_member_weight == 0.0 {
        wt.oxygen_mix = 0.0;
    }
    if !(u_cm_s.is_finite()
        && u_cm_s >= 0.0
        && x_o2.is_finite()
        && (0.0..=1.0).contains(&x_o2)
        && width_ratio.is_finite()
        && width_ratio > 0.0)
    {
        return Err(Refusal::unsupported(
            "oxygen_condition",
            "invalid flow, composition or width",
        ));
    }
    if wt.oxygen_mix > 0.0 {
        let width_m = wt.oxygen.reference_width_m * width_ratio;
        let physical = wt.oxygen.speed_m_s(u_cm_s / 100.0, x_o2) / width_m;
        if !(width_m.is_finite()
            && width_m > 0.0
            && physical.is_finite()
            && (physical > 0.0 || x_o2 == 0.0))
        {
            return Err(Refusal::failed(
                "non_finite_oxygen_law",
                "oxygen speed or physical width exceeds numerical representation",
            ));
        }
    }
    let laws = wt.laws(u_cm_s, x_o2, width_ratio);
    if ![laws.v, laws.v_zero_flow, laws.h_ss, laws.tau_b, laws.quench]
        .iter()
        .all(|v| v.is_finite())
    {
        return Err(Refusal::failed(
            "non_finite_law",
            "law exceeds finite numerical representation",
        ));
    }
    Ok(laws)
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
    /// Scenario and event semantics; deliberately local, not a shared contract extension.
    pub mechanism: Value,
}

/// Strict forward of a `bass_front.wsp3` release on an admitted prefix with metric observations:
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
            mechanism: if wt.oxygen_mix > 0.0 && wt.oxygen_member_weight > 0.0 {
                json!({
                    "oxygen_member_weight": wt.oxygen_member_weight,
                    "law_scenario_order": ["predecessor_speed if weight is positive", "oxygen_speed if weight is positive"],
                    "position_mixture": "persistent law x flow scenarios; mixture of complete Gaussian CDFs, not quantiles",
                    "event_model": "new coherent first-event interpretation for every active scenario, including predecessor speed; not a mixture of old event artifact rows",
                    "end_law": "monotone envelope of Gaussian endpoint exceedance on fixed cutoff + k*num.dt_s grid; assumed first-passage surrogate",
                    "event_discretization": "constant competing hazards within finite-CDF intervals; a CDF reaching one is an end atom at the right boundary after interval quenching",
                    "cutoff_end_mass": "initial endpoint-exceedance mass assigned to end before forecast quenching",
                    "paths": "predecessor-speed flow paths first, oxygen-speed flow paths second; sign denotes flow node, never law identity"
                })
            } else {
                json!({"event_model": "exact predecessor compatibility; inherited occupancy interpretation may have nonmonotone reached-end rows"})
            },
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
        "mechanism": m.mechanism,
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

/// `bass_front.wsp3` behind the common expert interface.
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
        bail!(
            "bass_front.wsp3 has no fitting procedure: its release is assembled from a literal patch (`ignis wsp assemble`)"
        )
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
        bail!("bass_front.wsp3 declares an assumed spread, not a likelihood")
    }
}

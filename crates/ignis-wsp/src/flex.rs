//! `flex.wsp` — the strict FLEX forward: liquid-droplet diameter after a cutoff, and the end of the
//! hot-flame phase, from fixed handwritten weights.
//!
//! ```text
//! q = D^2,  w = 1 - q/q0 (burn progress),  s = t K_ref / q0 (dimensionless clock),
//! x = (c_o2, c_co2, c_he, c_logp, c_logd0, w, s)
//!
//! burning        dq/dt = -K,          ln(K/K_ref) = N_rate(x) + b + s_z z
//! extinction     dLambda/dt = eta K/q0,  ln(eta)  = min(N_haz(x) + link_b b, cap)
//! extinguished   dq/dt = -k_evap
//! disruption     constant hazard lam_R (does not change the diameter)
//!
//! N(x) = b0 + skip . x + sum_j w2[j] relu(b1[j] + W1[j, :] . x)      (one hidden ReLU layer + skip)
//! ```
//!
//! `eta` is the extinction intensity per unit of burn progress, so where the flame goes out is
//! decided by how much of the droplet has burned, not by the clock. `b` is the run's own log-rate
//! offset, produced by the handwritten encoder from two finite differences of the prefix; `z` is a
//! standard-normal scenario variable with handwritten scale `s_z` (assumed, not calibrated).
//!
//! Nothing here fits anything: the prefix enters through window means and finite differences that
//! the spec declares as input transforms, the weights are read from the release, and the ODE is
//! integrated with a fixed-step RK4. There is no gradient, least-squares, root search over
//! parameters, sampler or seed on this route, and the crate does not link the code that has them.

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
use crate::quad::normal_quantile;
use crate::spec::{authored, constant, sparse, Params};

pub const SPEC_ID: &str = "flex-wsp-v1";
pub const MODEL_NAME: &str = "flex.wsp";
pub const FORWARD_VERSION: &str = "1";
/// Hidden width of the rate and hazard networks.
pub const HIDDEN: usize = 16;
/// Network inputs: `c_o2, c_co2, c_he, c_logp, c_logd0, w, s`.
pub const N_IN: usize = 7;
/// Encoder gain inputs: `g_span, g_count, g_progress`.
pub const N_GAIN: usize = 3;
const MAX_Z: usize = 64;

pub const EVENT_ACTIVE: &str = "active_supported";
pub const EVENT_UNRESOLVED: &str = "unresolved";

fn fin(v: f64) -> Finite {
    Finite::new(v).expect("finite literal")
}

/// The frozen specification an author writes against.
pub fn spec() -> ModelSpec {
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
    let h = HIDDEN as u32;
    let n = N_IN as u32;
    ModelSpec {
        schema_version: WspSchema::V1,
        spec_id: SpecId::new(SPEC_ID).expect("id"),
        family: Family::Flex,
        model_name: MODEL_NAME.into(),
        target: TargetSpec {
            quantity: Quantity::LiquidDiameter,
            unit: "m".into(),
            description: "Measured liquid-droplet diameter of a free-floating methanol droplet at \
                          times after the cutoff, as an unconditional predictive distribution \
                          (still burning, or extinguished earlier), plus the cumulative incidence \
                          of the end of the hot-flame phase and of disruption."
                .into(),
            excludes: vec![
                "visible or UV flame diameter".into(),
                "flame temperature, soot, species or radiant emission".into(),
                "cool-flame burning after hot-flame extinction".into(),
                "fuels other than methanol and fibre-supported droplets".into(),
                "a calibrated posterior: the spread is an assumed scenario distribution".into(),
            ],
        },
        features: vec![
            feature("c_o2", "condition:x_o2", "(x - 0.21) / 0.05 @1"),
            feature("c_co2", "condition:x_co2", "x / 0.20 @1"),
            feature("c_he", "condition:x_he", "x / 0.20 @1"),
            feature("c_logp", "condition:pressure", "ln(x / norm.p_ref_pa) @1"),
            feature(
                "c_logd0",
                "observation:liquid_diameter@prefix_head",
                "ln(sqrt(q0) / norm.d_ref_m); q0 = mean(D^2) over the head window advanced to t = 0 \
                 with the population rate at w = 0 @1",
            ),
            feature(
                "w",
                "state:burn_progress",
                "clamp(1 - D^2 / q0, 0, 1), integrated by the forward @1",
            ),
            feature(
                "s",
                "state:clock",
                "t * norm.k_ref_m2_s / q0 with t in seconds since the time origin (a diffusion clock:                  the same s is the same fraction of a D0^2 / K_ref lifetime for every droplet) @1",
            ),
            feature(
                "e_late",
                "observation:liquid_diameter@prefix_mid_tail",
                "clamp(ln(max(K_obs, enc.k_floor_m2_s) / K_model), +-enc.e_clip); K_obs = finite \
                 difference of window means of D^2 (mid -> tail); K_model = harmonic Simpson mean of \
                 the population rate over the same progress interval @1",
            ),
            feature(
                "e_early",
                "observation:liquid_diameter@prefix_head_mid",
                "as e_late for head -> mid; both equal the head -> tail value when the mid window \
                 is empty @1",
            ),
            feature(
                "g_span",
                "observation:time@prefix",
                "ln((mean time of tail window - mean time of head window) / 1 s) @1",
            ),
            feature(
                "g_count",
                "observation:count@prefix",
                "ln(n_valid / norm.n_ref) @1",
            ),
            feature(
                "g_progress",
                "state:burn_progress@prefix_tail",
                "clamp(w_tail / enc.w_ref, 0, 1) @1",
            ),
        ],
        domain: vec![
            bound("c_o2", -2.2, 4.8),
            bound("c_co2", 0.0, 4.0),
            bound("c_he", 0.0, 3.0),
            bound("c_logp", -1.0, 1.5),
            bound("c_logd0", -0.7, 1.26),
        ],
        tensors: vec![
            // ---- population burning-rate network: ln(K / K_ref)
            sparse("rate.W1", &[h, n], Weight, None,
                   "Hidden weights of the rate network; row j reads x = (c_o2, c_co2, c_he, c_logp, c_logd0, w, s)."),
            sparse("rate.b1", &[h], Bias, None, "Hidden biases of the rate network (hinge positions)."),
            sparse("rate.w2", &[h], Weight, None, "Output weights of the rate network (slope changes at the hinges)."),
            authored("rate.skip", &[n], Weight, None, None,
                     "Linear law of ln(K/K_ref) in x; every element is stated, including zeros."),
            authored("rate.b0", &[], PhysicalParameter, Some("ln(K / 1e-6 m2/s)"), Some((-3.0, 2.0)),
                     "ln(K/K_ref) at the reference state x = 0: air, 1e5 Pa, D0 = 2 mm, w = 0, s = 0, before hinge terms."),
            // ---- extinction network: ln(eta), eta = extinction intensity per unit burn progress
            sparse("haz.W1", &[h, n], Weight, None,
                   "Hidden weights of the extinction network; same input x as the rate network."),
            sparse("haz.b1", &[h], Bias, None, "Hidden biases of the extinction network."),
            sparse("haz.w2", &[h], Weight, None, "Output weights of the extinction network."),
            authored("haz.skip", &[n], Weight, None, None,
                     "Linear part of ln(eta) in x; every element is stated, including zeros."),
            authored("haz.b0", &[], HazardParameter, Some("ln(1)"), Some((-60.0, 20.0)),
                     "ln(eta) at x = 0 before hinge terms."),
            authored("haz.link_b", &[], Weight, None, Some((-10.0, 10.0)),
                     "Coupling of the run's log-rate offset b into ln(eta)."),
            authored("haz.log_disruption", &[], HazardParameter, Some("ln(1/s)"), Some((-20.0, 3.0)),
                     "ln of the constant disruption hazard."),
            // ---- prefix encoder: run offset b = gain * (mix . (e_late, e_early))
            authored("enc.gain_w", &[N_GAIN as u32], Weight, None, None,
                     "Weights of the gain logit on (g_span, g_count, g_progress)."),
            authored("enc.gain_b", &[], Bias, None, None, "Bias of the gain logit; gain = sigmoid(logit)."),
            authored("enc.mix", &[2], Weight, None, Some((0.0, 1.0)),
                     "Weights of (e_late, e_early) in the run's log-rate innovation."),
            // ---- assumed spread and noise
            authored("unc.s_pop", &[], NoiseParameter, Some("1"), Some((0.0, 2.0)),
                     "Assumed sd of a run's ln K around the population law before any prefix evidence."),
            authored("unc.s_floor", &[], NoiseParameter, Some("1"), Some((0.0, 2.0)),
                     "Assumed sd of ln K that no prefix removes."),
            authored("unc.sigma_d", &[], NoiseParameter, Some("m"), Some((0.0, 1.0e-3)),
                     "Assumed sd of one diameter measurement."),
            // ---- after hot-flame extinction
            authored("post.k_evap", &[], PhysicalParameter, Some("m2/s"), Some((0.0, 2.0e-7)),
                     "d(D^2)/dt magnitude of the extinguished droplet."),
            // ---- constants fixed by the spec
            constant("norm.d_ref_m", 0.002, "m", "Reference diameter of c_logd0."),
            constant("norm.p_ref_pa", 1.0e5, "Pa", "Reference pressure of c_logp."),
            constant("norm.k_ref_m2_s", 1.0e-6, "m2/s", "Reference burning-rate constant."),
            constant("norm.n_ref", 32.0, "1", "Reference number of prefix points of g_count."),
            constant("enc.window_frac", 0.25, "1",
                     "Head, mid and tail windows each span this fraction of the prefix duration."),
            constant("enc.e_clip", 1.0, "1", "Clamp of the log-rate innovations."),
            constant("enc.k_floor_m2_s", 2.0e-8, "m2/s", "Floor of an observed rate before its logarithm."),
            constant("enc.w_ref", 0.08, "1", "Progress at which g_progress saturates."),
            constant("enc.min_points", 2.0, "1", "Fewest valid prefix diameters for a forecast."),
            constant("haz.log_eta_cap", 12.0, "1", "Upper clamp of ln(eta) (numerical safety)."),
            constant("num.dt_s", 0.05, "s", "Largest RK4 step; steps end exactly on forecast times."),
            constant("num.d_min_m", 2.5e-4, "m", "Support floor: the model stops at this diameter."),
            constant("num.t_valid_s", 30.0, "s", "Model horizon since the time origin."),
            constant("num.z_step", 0.5, "1", "Spacing of the scenario nodes of z."),
            constant("num.z_count", 15.0, "1", "Number of scenario nodes of z, centred on 0."),
        ],
        forward: ForwardContract {
            entry_point: "ignis_wsp::flex::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(
            "Prefix observations: liquid_diameter, valid, positive, 0 <= t <= cutoff, first of equal \
             time stamps. State at the cutoff: tail-window mean of D^2 carried to the cutoff along \
             the observed mid -> tail finite difference (no weight enters it). Scenario nodes \
             z_k = (k - (n-1)/2) * z_step with weights proportional to \
             exp(-z^2/2). Predictive quantiles: weighted quantiles of the mixture atoms (burning per \
             scenario; extinguished per scenario and RK4 step), widened around the median by \
             unc.sigma_d in quadrature. Uncertainty kind: assumed_noise."
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
    pub(crate) fn eval(&self, x: &[f64; N_IN], scratch: &mut [f64; HIDDEN]) -> f64 {
        hinge_net(self.w1, self.b1, self.w2, self.skip, self.b0, x, scratch)
    }
}

/// Every number of a release the forward reads, by name.
pub(crate) struct Weights<'a> {
    pub(crate) rate: Net<'a>,
    pub(crate) haz: Net<'a>,
    link_b: f64,
    lam_r: f64,
    gain_w: &'a [f64],
    gain_b: f64,
    mix: &'a [f64],
    s_pop: f64,
    s_floor: f64,
    sigma_d: f64,
    k_evap: f64,
    d_ref: f64,
    p_ref: f64,
    pub(crate) k_ref: f64,
    n_ref: f64,
    window_frac: f64,
    e_clip: f64,
    k_floor: f64,
    w_ref: f64,
    min_points: usize,
    pub(crate) log_eta_cap: f64,
    dt: f64,
    q_min: f64,
    t_valid: f64,
    z_step: f64,
    z_count: usize,
}

impl<'a> Weights<'a> {
    pub(crate) fn bind(p: &Params<'a>) -> Result<Self, Refusal> {
        let net = |prefix: &str| Net {
            w1: p.v(&format!("{prefix}.W1")),
            b1: p.v(&format!("{prefix}.b1")),
            w2: p.v(&format!("{prefix}.w2")),
            skip: p.v(&format!("{prefix}.skip")),
            b0: p.s(&format!("{prefix}.b0")),
        };
        let d_min = p.s("num.d_min_m");
        let z_count = p.s("num.z_count") as usize;
        if !(3..=MAX_Z).contains(&z_count) || z_count.is_multiple_of(2) {
            return Err(Refusal::failed(
                "spec_constants",
                "num.z_count must be odd, 3..=64",
            ));
        }
        Ok(Weights {
            rate: net("rate"),
            haz: net("haz"),
            link_b: p.s("haz.link_b"),
            lam_r: p.s("haz.log_disruption").exp(),
            gain_w: p.v("enc.gain_w"),
            gain_b: p.s("enc.gain_b"),
            mix: p.v("enc.mix"),
            s_pop: p.s("unc.s_pop"),
            s_floor: p.s("unc.s_floor"),
            sigma_d: p.s("unc.sigma_d"),
            k_evap: p.s("post.k_evap"),
            d_ref: p.s("norm.d_ref_m"),
            p_ref: p.s("norm.p_ref_pa"),
            k_ref: p.s("norm.k_ref_m2_s"),
            n_ref: p.s("norm.n_ref"),
            window_frac: p.s("enc.window_frac"),
            e_clip: p.s("enc.e_clip"),
            k_floor: p.s("enc.k_floor_m2_s"),
            w_ref: p.s("enc.w_ref"),
            min_points: p.s("enc.min_points") as usize,
            log_eta_cap: p.s("haz.log_eta_cap"),
            dt: p.s("num.dt_s"),
            q_min: d_min * d_min,
            t_valid: p.s("num.t_valid_s"),
            z_step: p.s("num.z_step"),
            z_count,
        })
    }

    /// Population burning-rate constant (m^2/s) at progress `w` and clock `s`, without run offset
    /// or scenario.
    #[inline]
    fn k_pop(&self, c: &[f64; 5], w: f64, s: f64, scratch: &mut [f64; HIDDEN]) -> f64 {
        let x = [c[0], c[1], c[2], c[3], c[4], w, s];
        self.k_ref * self.rate.eval(&x, scratch).exp()
    }

    /// Harmonic Simpson mean of the population rate between two prefix windows `(t, q)` (what a
    /// finite difference of D^2 between them measures when dq/dt = -K).
    fn k_mean(
        &self,
        c: &[f64; 5],
        q0: f64,
        a: (f64, f64),
        b: (f64, f64),
        scratch: &mut [f64; HIDDEN],
    ) -> f64 {
        let w = |q: f64| (1.0 - q / q0).clamp(0.0, 1.0);
        let s = |t: f64| t * self.k_ref / q0;
        let (wa, wb, sa, sb) = (w(a.1), w(b.1), s(a.0), s(b.0));
        let inv = (1.0 / self.k_pop(c, wa, sa, scratch)
            + 4.0 / self.k_pop(c, 0.5 * (wa + wb), 0.5 * (sa + sb), scratch)
            + 1.0 / self.k_pop(c, wb, sb, scratch))
            / 6.0;
        1.0 / inv
    }
}

/// What the handwritten encoder made of a prefix (all of it is reported in the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct Encoded {
    /// `c_o2, c_co2, c_he, c_logp, c_logd0`.
    pub c: [f64; 5],
    pub n_valid: usize,
    pub q0_m2: f64,
    pub t_head_s: f64,
    pub q_head_m2: f64,
    pub t_tail_s: f64,
    pub q_tail_m2: f64,
    pub mid_window_used: bool,
    pub w_tail: f64,
    pub k_obs_late_m2_s: f64,
    pub k_obs_early_m2_s: f64,
    pub e_late: f64,
    pub e_early: f64,
    pub gain_inputs: [f64; N_GAIN],
    pub gain: f64,
    /// Run log-rate offset.
    pub b: f64,
    /// Sd of the scenario variable in ln K.
    pub s_z: f64,
    pub k_cutoff_m2_s: f64,
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
    let mut obs: Vec<(f64, f64)> = Vec::with_capacity(prefix.observations.len());
    for o in &prefix.observations {
        if o.quantity != Quantity::LiquidDiameter || !measurement_valid(o) {
            continue;
        }
        let d = o.value_si.unwrap_or(f64::NAN);
        let usable = d > 0.0 && o.time_s >= 0.0 && o.time_s <= tau;
        if !usable {
            continue;
        }
        if obs.last().is_some_and(|l| l.0 == o.time_s) {
            continue;
        }
        if obs.last().is_some_and(|l| l.0 > o.time_s) {
            return Err(refuse(
                SupportGate::Measurement,
                "prefix observations are not time-sorted",
            ));
        }
        obs.push((o.time_s, d * d));
    }
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
    let mut scratch = [0.0; HIDDEN];
    // D0: head mean advanced to the time origin with the population rate at w = 0 (one pass).
    c[4] = (head.q.sqrt() / wt.d_ref).ln();
    let q0 = head.q + wt.k_pop(&c, 0.0, 0.0, &mut scratch) * head.t;
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
    let mut innovation = |a: &Window, b: &Window| -> (f64, f64) {
        if !(b.t > a.t) {
            return (f64::NAN, 0.0);
        }
        let k_obs = (a.q - b.q) / (b.t - a.t);
        let k_mod = wt.k_mean(&c, q0, (a.t, a.q), (b.t, b.q), &mut scratch);
        let e = (k_obs.max(wt.k_floor) / k_mod)
            .ln()
            .clamp(-wt.e_clip, wt.e_clip);
        (k_obs, e)
    };
    let mid_used = mid.n > 0 && mid.t > head.t && tail.t > mid.t;
    let ((k_early, e_early), (k_late, e_late)) = if mid_used {
        (innovation(&head, &mid), innovation(&mid, &tail))
    } else {
        let r = innovation(&head, &tail);
        (r, r)
    };
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
    let k_cut = wt.k_pop(&c, w_tail, tail.t * wt.k_ref / q0, &mut scratch) * b.exp();
    let local_slope = if k_late.is_finite() { k_late } else { 0.0 };
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
    Ok(Encoded {
        c,
        n_valid: obs.len(),
        q0_m2: q0,
        t_head_s: head.t,
        q_head_m2: head.q,
        t_tail_s: tail.t,
        q_tail_m2: tail.q,
        mid_window_used: mid_used,
        w_tail,
        k_obs_late_m2_s: k_late,
        k_obs_early_m2_s: k_early,
        e_late,
        e_early,
        gain_inputs,
        gain,
        b,
        s_z,
        k_cutoff_m2_s: k_cut,
        q_cutoff_m2: q_cut,
        cutoff_s: tau,
    })
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
    pub partition_max_error: f64,
    pub rk4_steps: usize,
}

#[derive(Clone, Copy)]
struct Step {
    t_mid: f64,
    q_mid: f64,
    /// Probability (within the scenario) that the flame went out during this step.
    mass: f64,
}

/// Fixed-step RK4 of (q, Lambda) for every scenario, then the mixture at each forecast time.
fn integrate(wt: &Weights<'_>, enc: &Encoded, times: &[f64], levels: &[f64]) -> Forecast {
    let nt = times.len();
    let nz = wt.z_count;
    let tau = enc.cutoff_s;
    let c = &enc.c;
    let q0 = enc.q0_m2;
    // scenario nodes
    let mut z = [0.0; MAX_Z];
    let mut om = [0.0; MAX_Z];
    let mut total = 0.0;
    for k in 0..nz {
        z[k] = (k as f64 - 0.5 * (nz - 1) as f64) * wt.z_step;
        om[k] = (-0.5 * z[k] * z[k]).exp();
        total += om[k];
    }
    for w in om.iter_mut().take(nz) {
        *w /= total;
    }
    // per scenario and output time
    let mut q_out = vec![0.0; nz * nt];
    let mut lam_out = vec![0.0; nz * nt];
    let mut n_steps = vec![0usize; nz * nt];
    let mut exit_t = [f64::INFINITY; MAX_Z];
    let mut steps: Vec<Vec<Step>> = Vec::with_capacity(nz);
    let mut f_h = vec![0.0; nt];
    let mut rk4_steps = 0usize;
    let mut scratch = [0.0; HIDDEN];
    for k in 0..nz {
        let shift = enc.b + enc.s_z * z[k];
        let hz_shift = wt.link_b * enc.b;
        let mut deriv = |q: f64, t: f64| -> (f64, f64) {
            let x = [
                c[0],
                c[1],
                c[2],
                c[3],
                c[4],
                (1.0 - q / q0).clamp(0.0, 1.0),
                t * wt.k_ref / q0,
            ];
            let kk = wt.k_ref * (wt.rate.eval(&x, &mut scratch) + shift).exp();
            let eta = (wt.haz.eval(&x, &mut scratch) + hz_shift)
                .min(wt.log_eta_cap)
                .exp();
            (-kk, eta * kk / q0)
        };
        let (mut t, mut q, mut lam) = (tau, enc.q_cutoff_m2, 0.0_f64);
        let mut exited = false;
        let mut rec: Vec<Step> = Vec::new();
        let mut acc_h = 0.0;
        for (j, &tj) in times.iter().enumerate() {
            if tj > t {
                let n_sub = (((tj - t) / wt.dt) - 1e-9).ceil().max(1.0) as usize;
                let h = (tj - t) / n_sub as f64;
                for _ in 0..n_sub {
                    let (q_new, lam_new) = if exited {
                        (q, lam)
                    } else {
                        let (a1, l1) = deriv(q, t);
                        let (a2, l2) = deriv(q + 0.5 * h * a1, t + 0.5 * h);
                        let (a3, l3) = deriv(q + 0.5 * h * a2, t + 0.5 * h);
                        let (a4, l4) = deriv(q + h * a3, t + h);
                        (
                            q + h / 6.0 * (a1 + 2.0 * a2 + 2.0 * a3 + a4),
                            lam + h / 6.0 * (l1 + 2.0 * l2 + 2.0 * l3 + l4),
                        )
                    };
                    let (q_new, hit) = if !exited && q_new <= wt.q_min {
                        (wt.q_min, true)
                    } else {
                        (q_new, false)
                    };
                    let mass = (-lam).exp() - (-lam_new).exp();
                    if mass > 0.0 {
                        let t_mid = t + 0.5 * h;
                        rec.push(Step {
                            t_mid,
                            q_mid: 0.5 * (q + q_new),
                            mass,
                        });
                        acc_h += om[k] * mass * (-wt.lam_r * (t_mid - tau)).exp();
                    }
                    t += h;
                    q = q_new;
                    lam = lam_new;
                    if hit {
                        exited = true;
                        exit_t[k] = t;
                    }
                    rk4_steps += 1;
                }
                t = tj;
            }
            q_out[k * nt + j] = q;
            lam_out[k * nt + j] = lam;
            n_steps[k * nt + j] = rec.len();
            f_h[j] += acc_h;
        }
        steps.push(rec);
    }
    // mixture at each forecast time
    let nl = levels.len();
    let mut quantiles = vec![vec![0.0; nt]; nl];
    let mut latent = vec![vec![0.0; nt]; nl];
    let mut mean = vec![0.0; nt];
    let mut events: [Vec<f64>; 4] = std::array::from_fn(|_| vec![0.0; nt]);
    let mut atoms: Vec<(f64, f64)> =
        Vec::with_capacity(nz * (steps.iter().map(Vec::len).max().unwrap_or(0) + 1));
    // the requested levels plus the median, ascending, sorted once per forecast time
    let mut ext: Vec<f64> = levels.to_vec();
    if !ext.contains(&0.5) {
        ext.push(0.5);
        ext.sort_by(f64::total_cmp);
    }
    let i_med = ext.iter().position(|p| *p == 0.5).expect("median level");
    let slot: Vec<usize> = levels
        .iter()
        .map(|p| ext.iter().position(|e| e == p).expect("level"))
        .collect();
    let mut qe = vec![0.0; ext.len()];
    let z_levels: Vec<f64> = levels.iter().map(|p| normal_quantile(*p)).collect();
    let mut worst = 0.0_f64;
    for (j, &tj) in times.iter().enumerate() {
        atoms.clear();
        let s_r = (-wt.lam_r * (tj - tau)).exp();
        let (mut active, mut unresolved) = (0.0, 0.0);
        for k in 0..nz {
            let i = k * nt + j;
            for s in &steps[k][..n_steps[i]] {
                let q = (s.q_mid - wt.k_evap * (tj - s.t_mid)).max(wt.q_min);
                atoms.push((q.sqrt(), om[k] * s.mass));
            }
            let surv = (-lam_out[i]).exp();
            if surv > 0.0 {
                atoms.push((q_out[i].sqrt(), om[k] * surv));
            }
            if tj >= exit_t[k] {
                unresolved += om[k] * surv * (-wt.lam_r * (exit_t[k] - tau)).exp();
            } else {
                active += om[k] * surv * s_r;
            }
        }
        mean[j] =
            atoms.iter().map(|a| a.0 * a.1).sum::<f64>() / atoms.iter().map(|a| a.1).sum::<f64>();
        common::atom_quantiles(&mut atoms, &ext, &mut qe);
        let med = qe[i_med];
        for l in 0..nl {
            let ql = qe[slot[l]];
            latent[l][j] = ql;
            let noise = z_levels[l] * wt.sigma_d;
            let widened = ((ql - med) * (ql - med) + noise * noise).sqrt();
            quantiles[l][j] = if levels[l] < 0.5 {
                (med - widened).max(0.0)
            } else if levels[l] > 0.5 {
                med + widened
            } else {
                med
            };
        }
        let disruption = 1.0 - active - f_h[j] - unresolved;
        worst = worst.max((-disruption).max(0.0));
        events[0][j] = active.clamp(0.0, 1.0);
        events[1][j] = f_h[j].clamp(0.0, 1.0);
        events[2][j] = disruption.clamp(0.0, 1.0);
        events[3][j] = unresolved.clamp(0.0, 1.0);
    }
    Forecast {
        encoded: enc.clone(),
        times_s: times.to_vec(),
        levels: levels.to_vec(),
        quantiles,
        latent_quantiles: latent,
        mean,
        events,
        partition_max_error: worst,
        rk4_steps,
    }
}

/// Strict forward of a `flex.wsp` release on an admitted prefix: `Ok(Err(support))` is a scientific
/// refusal, `Err(refusal)` a release or request that cannot be run at all.
pub fn forecast(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<Result<Forecast, Support>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params)?;
    let built_with = release
        .release
        .construction
        .as_ref()
        .map(|m| &m.forward_hash);
    let this = crate::audit::forward_hash(Family::Flex);
    if built_with != Some(&this) {
        return Err(Refusal::unsupported(
            "forward_mismatch",
            format!(
                "release {} was assembled with forward {}, this build runs {this}: a release is its                  weights and its forward (re-assemble, or run the build it names)",
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
    if let (Some(f), Some(want)) = (
        prefix.known_conditions.get("fuel"),
        release
            .release
            .applicability
            .get("fuel")
            .and_then(Value::as_str),
    ) {
        let raw = match &f.raw_value {
            Some(ignis_core::contracts::RawValue::Text(s)) => s.clone(),
            Some(ignis_core::contracts::RawValue::Number(x)) => x.to_string(),
            None => String::new(),
        };
        if !raw.eq_ignore_ascii_case(want) {
            return Ok(Err(refuse(
                SupportGate::Semantic,
                format!("fuel {raw:?} outside the release population {want:?}"),
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
        "band_meaning": "unconditional predictive of the measured liquid diameter: mixture over rate \
                         scenarios and over the time the hot flame went out; disruption does not \
                         change the diameter",
        "latent_quantiles": f.latent_quantiles,
        "uncertainty_kind": "assumed_noise",
        "partition_max_error": f.partition_max_error,
        "rk4_steps": f.rk4_steps,
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

/// `flex.wsp` behind the common expert interface.
pub struct FlexWsp {
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl Default for FlexWsp {
    fn default() -> Self {
        Self::new()
    }
}

impl FlexWsp {
    pub fn new() -> Self {
        FlexWsp {
            spec: spec(),
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
}

impl Expert for FlexWsp {
    fn family(&self) -> Family {
        Family::Flex
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
        bail!("flex.wsp has no fitting procedure: its release is assembled from a literal patch (`ignis wsp assemble`)")
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
        bail!("flex.wsp declares an assumed scenario spread, not a likelihood")
    }
}

//! `flex_<fuel>.wsp8` — `droplet8.rs` (the sixth droplet law for eight fuels) with the mechanisms of
//! `docs/wsp/research/phys_droplet_fuels.md` sections 3.2 and 3.3 that are specified there tightly
//! enough to build: a composition input and staged burning for two-component droplets, a warm-flame
//! stage, a cool flame lit directly by the igniter, and fibre-supported droplets. Each block has its
//! own switch. With every switch off (`mix.on = 0`, `st.on = 0`, `warm.on = 0`, `ig.on = 0`,
//! `fib.on = 0`) and the new skip elements zero, the forecast is that of `droplet8.rs`, number for
//! number (tested on every committed `flex_<fuel>.wsp7` release).
//!
//! Fuels: the eight of `droplet8.rs` and three mixtures, n-heptane + iso-octane and n-dodecane +
//! iso-dodecane (n-heptane tensor set) and 1-propanol + glycerol (methanol tensor set).
//!
//! **Composition (`mix.on = 1`).** The prefix carries `y_b_mass` (mass fraction of component B, the
//! less volatile one or, for equal volatility, the less reactive one) or `y_b_volume` (volume
//! fraction, converted with the two liquid densities `mix.rho_a`, `mix.rho_b`:
//! `y_b = rho_b phi / (rho_a (1 - phi) + rho_b phi)`); a mole fraction is refused (the spec has no
//! molar masses). Every law network reads one more input, last: `c_y = y_b`, except `cool.p` and
//! `cool.k`, which read the surface composition `y_s = clamp(y_b + st.drift w, 0, 1)` at the burn
//! progress of the moment they are evaluated (a radiative loss, or the cutoff). With the closed form of
//! the cool-flame end, `beta = ce.beta_atm + ce.beta_y y_s` and `g0 = ce.g0 exp(ce.g0_y y_s)`.
//!
//! **Staged burning (`st.on = 1`, needs `mix.on = 1`).** For `y_b > 0`, nodes `u` of a grid of
//! `num.dc_count` points spaced `num.dc_step` with normal weights:
//!
//! ```text
//! D_c = D0 [1 + 3 st.eps (1 - y_b)/y_b]^(-1/3) exp(st.dc_sd u)
//! ```
//!
//! A hot flame that reaches `D_c` is lost with probability `sigmoid(N_flash(law inputs))` (the droplet
//! goes on as after a diffusive extinction); otherwise it burns at
//! `K_h exp(st.k2) (1 - st.dip_amp exp(-(s - s_c)/st.dip_clock))`, `s_c` the clock at `D_c`, and its
//! disruption hazard is `exp(st.log_disruption2)`. A cool flame that reaches `D_c` gets the same
//! factor. A droplet below `D_c` at the cutoff is in stage 2 (its flash, if any, happened before the
//! cutoff and is taken out of the hot state; the time it reached `D_c` is not known, so it has no dip).
//! The end of a cool flame in stage 2 is the root of a monotone closed form, found by bisection in
//! time (no parameter is involved).
//!
//! **Warm flame (`warm.on = 1`, fuels with a cool-flame law).** After a radiative loss in the forecast,
//! with probability `sigmoid(N_warm_p)` a warm flame burns at `K_c exp(warm.k)` for a clock length
//! drawn as the radiative clock is (median `exp(N_warm_s)`, Gompertz sharpness `warm.sharp`, on
//! `num.warm_count` equal-mass quantile nodes); then the droplet goes on as after a radiative loss now
//! does (a cool flame with probability `p_cool`, else no flame), with no dying flame. The state filter
//! has no warm state: a warm flame counts as a cool flame.
//!
//! **Direct cool ignition (`ig.on = 1`, fuels with a cool-flame law).** With probability
//! `sigmoid(N_ig(law inputs, c_ign))`, `c_ign = ln(P_ign t_ign / ig.e_ref)` from the conditions
//! `igniter_power` (W) and `igniter_time` (s), the droplet starts in a cool flame at s = 0: the prior
//! at the cutoff moves that share from the hot flame to a cool flame alive when the droplet is still
//! larger than `D_ce`.
//!
//! **Fibre (`fib.on = 1`).** Reads the condition `fiber` (0 or 1); for a supported droplet `fib.rate`
//! is added to ln K_h and `fib.crit` to ln D_crit. With `fib.on = 0` a prefix with `fiber = 1` is
//! refused, as by `droplet8.rs`'s exclusion of fibre-supported droplets.
//!
//! Not built, because section 3.2 or 3.3 does not specify it tightly enough (see the report of the
//! stage that added this file): the re-drawn radiative class of stage 2 (`st.crit2`), reignition of
//! the hot flame (`re.*`) and with it `fib.re`, and the prior of a warm flame begun before the cutoff.
//!
//! The rest of this header is that of `droplet8.rs`, whose own header follows that of `droplet6.rs`.
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
//! no flame   D^2 falls by  k_inf r^m dt + k_hot r^h tau_e (1 - exp(-dt/tau_e)),  tau_e = tau0 r^n,  r = q_e/q_ref
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
//! # A clock that can be wrong
//!
//! The radiative clock is a law typed from the atmospheres of a few dozen runs, with a sharp
//! survival curve around its median. In an atmosphere it was not written from it can be wrong by a
//! large factor, and a sharp curve then gives a flame that is plainly burning a probability of
//! 1e-8: no measurement can move that, and a filter built on it cannot recover. So the survival of
//! the radiative class is a mixture:
//!
//! ```text
//! S_R(s) = (1 - rob.eps_r) exp(-Lambda_R(s)) + rob.eps_r / (1 + s / s_R)
//! ```
//!
//! With probability `rob.eps_r` the clock of this atmosphere is not the law's: same median, a
//! spread of orders of magnitude (a log-logistic of shape 1). A flame seen burning long after the
//! law's clock stays in the radiative class with a clock that is at least as long as it has
//! burned, and a flame lost long before it is not a surprise either. With `rob.eps_r = 0` the
//! survival is the law's, number for number.
//!
//! # Flame cameras
//!
//! Everything above reads the droplet diameter alone, and has to infer from a change of slope that
//! a flame is gone. NASA's files hold what shows it directly: the flame diameter seen by the UV
//! camera (OH* at 310 nm) and by the colour camera. A hot flame is in those traces; a cool flame and
//! an unlit droplet are not. When a prefix carries flame diameters this forward reads them for one
//! thing: whether a camera that recorded this flame still shows it at the cutoff.
//!
//! ```text
//! recorded   at least fl.min_points positive flame diameters in [fl.t_min_s, cutoff] (either camera)
//! t_last     the last positive flame diameter that has two more within fl.gap_s before it (a single
//!            stray frame after the flame is not a flame)
//! seen       recorded, and cutoff - t_last <= fl.gap_s
//! state      P = fl.trust x (what the cameras say) + (1 - fl.trust) x (what the diameter says)
//!            cameras say: hot flame if seen; otherwise no hot flame, with cool flame and no flame
//!            in the proportion the diameter gives them (the law's, if it gave both zero)
//! heat left  a droplet whose flame was lost at t_last < cutoff has exp(-(cutoff - t_last) / tau_e)
//!            of the flame's heat left; weight fl.evap_mix against the estimate from the end rate
//! ```
//!
//! `fl.trust` is the probability that the cameras are right about the hot flame: a flame can drift
//! out of both views while it burns, and a tracker can miss a dim one. With `fl.trust = 0`, or on a
//! prefix without flame diameters, the forecast is that of the diameter alone, number for number.
//! The flame diameter itself, its growth and its collapse are not used: on the train runs they do
//! not separate a flame about to be lost to radiation from one that will burn on (both are near
//! their largest at 5 s), and the shrinking of a flame before a diffusive extinction follows the
//! droplet.
//!
//! # Third generation
//!
//! What the train audits of the second generation showed, and what this forward does about it:
//!
//! * A droplet that lost its flame one or two seconds before the cutoff was still read as burning,
//!   because the mid -> tail finite difference straddles the extinction. The state evidence and the
//!   level at the cutoff now come from the *end rate*: the finite difference between the last two
//!   windows of the prefix (`enc.end_frac` of its duration each). The evidence has its own exponent
//!   (`filt.gain_*`) and its own hot-flame spread (`filt.sd_hot`), so that trusting the run's rate
//!   offset less does not blunt the state filter.
//! * The run offset of a cool flame was read from a window that still held the hot flame or the dip
//!   between the two, and missed the cool rate by more than the law alone. It is now the end-rate
//!   innovation against the cool law, scaled by `enc.cool_gain`.
//! * After extinction the unlit droplet sheds the flame's heat with an amplitude that grows with its
//!   size and a decay time that hardly does; both scalings are written (`post.hot_exp`,
//!   `post.tau_exp`) instead of fixed at 0 and 1.
//!
//! # Sixth forward
//!
//! Three laws for what happens after the hot flame, each behind its own switch.
//!
//! **A. Self-cooling evaporation (`sc.on = 1`).** A droplet that a flame has just left is hot and
//! evaporates on its own stored heat; as it evaporates it cools, so the rate falls hyperbolically,
//! not exponentially, towards the rate the surrounding gas alone sustains. With `u` the time since
//! the flame ended and `D_e` the diameter at that moment:
//!
//! ```text
//! K(u)       = K_floor(u) + K0 / (1 + u / t0)                                  [-d(D^2)/dt]
//! t0         = sc.tstar_k sc.cl_j_kg_k D_e^2 / (1.5 K0 sc.lat_j_kg)            [s; D_e^2 in m2, K0 in m2/s]
//! K0         = g f sc.k0_hot_mm2_s     straight from a hot flame
//!              g   sc.k0_cool_mm2_s    from a cool flame
//! g          = exp(sc.gas[0] x_He + sc.gas[1] x_CO2 + sc.gas[2] ln(p / norm.p_ref_pa))
//! K_floor(u) = g f k_cold + (f k_warm - g f k_cold) exp(-u / sc.tau_warm_s)
//!              k_cold = sc.k_cold_mm2_s,  k_warm = sc.k_warm_mm2_s
//! f          = sc.diff_scale  after a diffusive extinction of the hot flame (the liquid holds
//!              the water it took up, or the little fuel left, and is less volatile); 1 otherwise
//! D^2(u)     = D_e^2 - [ K0 t0 ln(1 + u/t0) + g f k_cold u
//!                        + (f k_warm - g f k_cold) sc.tau_warm_s (1 - exp(-u/sc.tau_warm_s)) ]
//! ```
//!
//! floored at `num.d_min_m`. The rates are typed in mm2/s and converted. With `K0 = 0` the first
//! term is absent. With `sc.on = 1` this law stands wherever `droplet5.rs` uses the `post.*` law,
//! with the same time origin: after an extinction in the forecast (origin: the extinction), after a
//! cool flame (origin: its end), in the expected end rate of the no-flame state of the state filter
//! (`K_floor(0) +` half of `K0`, as the `post.*` law takes half of its transient), and for a droplet
//! without a flame at the cutoff. For that droplet the time since the flame ended is not known from
//! the diameter, and the law is restarted at the cutoff with the share of `K0` the end rate shows
//! (weight: the state-evidence exponent) or half of it; a hyperbola restarted lower is the same
//! hyperbola later, so `t0` is taken with the `K0` that is left. When the cameras lost the flame
//! at `t_last` the law runs from `t_last` itself (`u = t - t_last`), with weight `fl.evap_mix`
//! against the end-rate estimate. In both, `D_e` is the diameter at the cutoff and the droplet
//! counts as coming from a hot flame lost to radiation (`f = 1`): how its flame ended is not known.
//!
//! **B. A dying flame (`dy.frac > 0`).** A hot flame lost to radiation that no sustained cool flame
//! follows does not stop at once. The unlit law gets one more term,
//!
//! ```text
//! K_die(u) = dy.frac K_c exp(-u / dy.tau_s),    fall of D^2: dy.frac K_c dy.tau_s (1 - exp(-u/dy.tau_s))
//! ```
//!
//! with `K_c` the cool-flame rate of this run and scenario (the `cool.k` law times the same run
//! offset and scenario factor the forward gives a cool flame that starts there). It applies in the
//! no-cool-flame branch after a radiative loss and to a cool flame of zero length under law C, with
//! either unlit law; never after a diffusive extinction, after a cool flame that burned, or to a
//! droplet already without a flame at the cutoff. A fuel without a cool-flame law has `K_c = 0`.
//!
//! **C. The cool-flame end in oxygen partial pressure (`ce.on = 1`).** The median diameter at which
//! a cool flame ends is the closed form
//!
//! ```text
//! D_ce = D0 (D0 / norm.d_ref_m)^(ce.d0_exp - 1) ce.g0
//!        sqrt((ce.p_ref_atm - ce.beta_atm) / max(pO2 - ce.beta_atm, ce.eps_atm)) (1 + ce.c_he x_He)
//! pO2  = x_O2 p / 101325 Pa                                                     [atm]
//! ```
//!
//! in place of the median of the `cool.d` network, wherever that is used (the cool flame alive at
//! the cutoff, and the scenario nodes of its end); the spread `cool.d.sd` and the nodes stay. A
//! scenario node whose cool flame would end at or above the diameter at which the hot flame was
//! lost has zero length: under this law the droplet then comes straight from a hot flame (law B
//! applies, and `K0` is the hot one). With `ce.on = 0` such a node keeps what `droplet5.rs` does:
//! the droplet counts as coming from a cool flame.
//!
//! The new laws read the four conditions the forward already reads, in physical units
//! (`x_O2`, `x_CO2`, `x_He`, pressure); nothing else enters them.
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

pub const FORWARD_VERSION: &str = "9";
/// Pascals in one atmosphere: the oxygen partial pressure of the cool-flame end is in atm.
const ATM_PA: f64 = 101_325.0;
/// Square metres in one square millimetre: the rates of the self-cooling law are typed in mm2/s.
const MM2: f64 = 1.0e-6;
/// Hidden width of the rate network.
pub const H_RATE: usize = 16;
/// Hidden width of every law network.
pub const H_LAW: usize = 8;
/// Rate-network inputs: `c_o2, c_co2, c_he, c_logp, c_logd0, w, s, c_y`.
pub const N_RATE: usize = 8;
/// Critical-diameter inputs: `c_o2, c_co2, c_he, c_logp, c_y`.
pub const N_CRIT: usize = 5;
/// Law inputs: `c_o2, c_co2, c_he, c_logp, c_logd0, x_exc, c_y`.
pub const N_LAW: usize = 7;
/// Cool-flame rate and extinction-diameter inputs: `c_o2, c_co2, c_he, c_logp, c_logd0, c_y`.
pub const N_COOL: usize = 6;
/// Direct-ignition inputs: the law inputs and `c_ign`.
pub const N_IGN: usize = 8;
const MAX_DC: usize = 16;
const MAX_WARM: usize = 16;
pub const N_GAIN: usize = 3;
const MAX_Z: usize = 64;
const MAX_CE: usize = 16;

pub const EVENT_ACTIVE: &str = "active_supported";
pub const EVENT_UNRESOLVED: &str = "unresolved";

/// Largest `c_logp` of the domain: ln(547 kPa / 100 kPa), above the 506.6 kPa of CFI.
pub const C_LOGP_MAX: f64 = 1.7;
/// Largest `c_he` of the domain: 85 % helium, above the 83 % of FLEX-2.
pub const C_HE_MAX: f64 = 4.25;
/// Model horizon since the time origin (s).
pub const T_VALID_S: f64 = 200.0;

/// The fuels this forward has a spec for. The forward is shared; the weights are not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fuel {
    Methanol,
    Heptane,
    Octane,
    Isooctane,
    Decane,
    Dodecane,
    Farnesane,
    Ethanol,
    /// n-heptane + iso-octane (FLEX-2); component B is iso-octane (equal volatility, less reactive).
    HeptaneIsooctane,
    /// n-dodecane + iso-dodecane (CFI); component B is n-dodecane (the less volatile).
    DodecaneIsododecane,
    /// 1-propanol + glycerol (FLEX-2); component B is glycerol (the less volatile).
    PropanolGlycerol,
}

impl Fuel {
    pub const ALL: [Fuel; 11] = [
        Fuel::Methanol,
        Fuel::Heptane,
        Fuel::Octane,
        Fuel::Isooctane,
        Fuel::Decane,
        Fuel::Dodecane,
        Fuel::Farnesane,
        Fuel::Ethanol,
        Fuel::HeptaneIsooctane,
        Fuel::DodecaneIsododecane,
        Fuel::PropanolGlycerol,
    ];
    pub const fn spec_id(self) -> &'static str {
        match self {
            Fuel::Methanol => "flex_methanol-wsp-v9",
            Fuel::Heptane => "flex_heptane-wsp-v9",
            Fuel::Octane => "flex_octane-wsp-v2",
            Fuel::Isooctane => "flex_isooctane-wsp-v2",
            Fuel::Decane => "flex_decane-wsp-v2",
            Fuel::Dodecane => "flex_dodecane-wsp-v2",
            Fuel::Farnesane => "flex_farnesane-wsp-v2",
            Fuel::Ethanol => "flex_ethanol-wsp-v2",
            Fuel::HeptaneIsooctane => "flex_heptane_isooctane-wsp-v1",
            Fuel::DodecaneIsododecane => "flex_dodecane_isododecane-wsp-v1",
            Fuel::PropanolGlycerol => "flex_propanol_glycerol-wsp-v1",
        }
    }
    pub const fn model_name(self) -> &'static str {
        match self {
            Fuel::Methanol => "flex_methanol.wsp8",
            Fuel::Heptane => "flex_heptane.wsp8",
            Fuel::Octane => "flex_octane.wsp8",
            Fuel::Isooctane => "flex_isooctane.wsp8",
            Fuel::Decane => "flex_decane.wsp8",
            Fuel::Dodecane => "flex_dodecane.wsp8",
            Fuel::Farnesane => "flex_farnesane.wsp8",
            Fuel::Ethanol => "flex_ethanol.wsp8",
            Fuel::HeptaneIsooctane => "flex_heptane_isooctane.wsp8",
            Fuel::DodecaneIsododecane => "flex_dodecane_isododecane.wsp8",
            Fuel::PropanolGlycerol => "flex_propanol_glycerol.wsp8",
        }
    }
    /// The fuel as a label (for methanol and n-heptane as the PSI-69 summary writes it).
    pub const fn label(self) -> &'static str {
        match self {
            Fuel::Methanol => "Methanol",
            Fuel::Heptane => "Heptane",
            Fuel::Octane => "n-Octane",
            Fuel::Isooctane => "iso-Octane",
            Fuel::Decane => "n-Decane",
            Fuel::Dodecane => "n-Dodecane",
            Fuel::Farnesane => "Farnesane",
            Fuel::Ethanol => "Ethanol",
            Fuel::HeptaneIsooctane => "n-Heptane + iso-Octane",
            Fuel::DodecaneIsododecane => "n-Dodecane + iso-Dodecane",
            Fuel::PropanolGlycerol => "1-Propanol + Glycerol",
        }
    }
    /// The names of the fuel after [`normalised_fuel_name`] (a mixture in either order).
    pub const fn names(self) -> &'static [&'static str] {
        match self {
            Fuel::Methanol => &["methanol"],
            Fuel::Heptane => &["heptane"],
            Fuel::Octane => &["octane"],
            Fuel::Isooctane => &["isooctane", "224trimethylpentane"],
            Fuel::Decane => &["decane"],
            Fuel::Dodecane => &["dodecane"],
            Fuel::Farnesane => &["farnesane", "2610trimethyldodecane"],
            Fuel::Ethanol => &["ethanol"],
            Fuel::HeptaneIsooctane => &["heptane+isooctane", "isooctane+heptane"],
            Fuel::DodecaneIsododecane => &["dodecane+isododecane", "isododecane+dodecane"],
            Fuel::PropanolGlycerol => &[
                "propanol+glycerol",
                "1propanol+glycerol",
                "glycerol+propanol",
                "glycerol+1propanol",
            ],
        }
    }
    /// Component B of a mixture (the one `y_b` measures), `None` for a pure fuel.
    pub const fn component_b(self) -> Option<&'static str> {
        match self {
            Fuel::HeptaneIsooctane => Some("iso-octane"),
            Fuel::DodecaneIsododecane => Some("n-dodecane"),
            Fuel::PropanolGlycerol => Some("glycerol"),
            _ => None,
        }
    }
    /// Whether a prefix's `fuel` condition names this fuel.
    pub fn accepts(self, raw: &str) -> bool {
        let n = normalised_fuel_name(raw);
        self.names().iter().any(|m| *m == n)
    }
    /// The fuel has the cool-flame laws (the n-heptane tensor set); without them it has the
    /// methanol set.
    pub const fn has_cool(self) -> bool {
        !matches!(
            self,
            Fuel::Methanol | Fuel::Ethanol | Fuel::PropanolGlycerol
        )
    }
    pub fn of_model(name: &str) -> Option<Fuel> {
        Fuel::ALL.into_iter().find(|f| f.model_name() == name)
    }
}

/// A fuel name with each `+`-separated component lower-cased, without a leading `n-` (or `n_`,
/// `n `), and without spaces, hyphens, underscores and commas.
pub fn normalised_fuel_name(raw: &str) -> String {
    raw.split('+')
        .map(|part| {
            let s = part.trim().to_lowercase();
            let s = ["n-", "n_", "n "]
                .iter()
                .find_map(|p| s.strip_prefix(p))
                .unwrap_or(&s)
                .to_string();
            s.chars()
                .filter(|c| !matches!(c, ' ' | '-' | '_' | ','))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("+")
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
    let law_in = "c_o2, c_co2, c_he, c_logp, c_logd0, x_exc, c_y";
    let cool_in = "c_o2, c_co2, c_he, c_logp, c_logd0, c_y (y_s in cool.k)";
    let mut tensors = Vec::new();
    // ---- hot-flame burning-rate network
    tensors.extend(net(
        "rate",
        hr,
        nr,
        "the hot-flame rate network ln(K_h/K_ref)",
        "c_o2, c_co2, c_he, c_logp, c_logd0, w, s, c_y",
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
        "c_o2, c_co2, c_he, c_logp, c_y",
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
            "c_o2, c_co2, c_he, c_logp, c_logd0, x_exc, c_y (y_s)",
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
                 "Extra d(D^2)/dt magnitude right after extinction of a droplet of D = D_ref (the gas is still hot); decays with tau_e."),
        authored("post.tau0_s", &[], PhysicalParameter, Some("s"), Some((0.01, 100.0)),
                 "Decay time of post.k_hot for a droplet extinguished at D = D_ref."),
        authored("post.size_exp", &[], PhysicalParameter, Some("1"), Some((0.0, 2.0)),
                 "Exponent of (q_e/q_ref) in the long-time evaporation rate."),
        authored("post.hot_exp", &[], PhysicalParameter, Some("1"), Some((-1.0, 2.0)),
                 "Exponent of (q_e/q_ref) in the amplitude of the extra evaporation right after extinction."),
        authored("post.tau_exp", &[], PhysicalParameter, Some("1"), Some((0.0, 2.0)),
                 "Exponent of (q_e/q_ref) in the decay time of the extra evaporation."),
        // ---- prefix encoder
        authored("enc.gain_w", &[N_GAIN as u32], Weight, None, None,
                 "Weights of the gain logit on (g_span, g_count, g_progress)."),
        authored("enc.gain_b", &[], Bias, None, None, "Bias of the gain logit; gain = sigmoid(logit)."),
        authored("enc.mix", &[2], Weight, None, Some((0.0, 1.0)),
                 "Weights of (e_late, e_early) in the run's log-rate innovation."),
        authored("enc.e_clip", &[2], Weight, None, Some((0.0, 3.0)),
                 "Clamp of the log-rate innovations: (below, above) the population rate."),
        authored("enc.cool_gain", &[], Weight, None, Some((0.0, 1.0)),
                 "Share of the end-rate innovation against the cool-flame law that a cool flame burning at the cutoff keeps as its run offset."),
        // ---- state at the cutoff
        authored("filt.sd_hot", &[], NoiseParameter, Some("1"), Some((0.01, 2.0)),
                 "Assumed sd of the end-rate ratio r = K_obs/K_h of a droplet inside a hot flame (inside a cool flame: times K_c/K_h)."),
        authored("filt.sd_evap", &[], NoiseParameter, Some("1"), Some((0.01, 2.0)),
                 "Assumed sd of the end-rate ratio r = K_obs/K_h of a droplet without a flame."),
        authored("filt.l_floor", &[], NoiseParameter, Some("1"), Some((0.0, 1.0)),
                 "Density added to every state's likelihood of r, so that an absurd r returns the prior."),
        authored("filt.gain_w", &[N_GAIN as u32], Weight, None, None,
                 "Weights of the state-evidence exponent logit on (g_span, g_count, g_progress)."),
        authored("filt.gain_b", &[], Bias, None, None,
                 "Bias of the state-evidence exponent logit; exponent = sigmoid(logit)."),
        // ---- a clock that can be wrong
        authored("rob.eps_r", &[], NoiseParameter, Some("1"), Some((0.0, 0.5)),
                 "Probability that the radiative clock of this atmosphere is not the law's: weight of a clock with the same median s_R and survival 1 / (1 + s / s_R) in the survival of the radiative class (0: the law alone)."),
        // ---- flame cameras
        authored("fl.trust", &[], Weight, Some("1"), Some((0.0, 1.0)),
                 "Probability that the flame cameras are right about the hot flame at the cutoff; the state at the cutoff is this weight on their verdict and the rest on the diameter's (0: the cameras are not read)."),
        authored("fl.t_min_s", &[], NoiseParameter, Some("s"), Some((0.0, 5.0)),
                 "Flame diameters before this time are not used (igniter glare and the ignition flash)."),
        authored("fl.gap_s", &[], NoiseParameter, Some("s"), Some((0.01, 5.0)),
                 "The hot flame counts as seen at the cutoff when the last positive flame diameter that has two more within this time before it lies at most this long before the cutoff."),
        authored("fl.min_points", &[], NoiseParameter, Some("1"), Some((3.0, 200.0)),
                 "Fewest positive flame diameters after fl.t_min_s for the cameras to count as having recorded this flame."),
        authored("fl.evap_mix", &[], Weight, Some("1"), Some((0.0, 1.0)),
                 "For a droplet whose flame the cameras lost before the cutoff: weight of the flame's heat left according to the time since it was lost, against the estimate from the end rate."),
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
        constant("enc.end_frac", 0.15, "1", "The end rate is the finite difference between the means of D^2 over the last two windows of the prefix, each spanning this fraction of its duration; the level at the cutoff is carried from the last of them."),
        constant("enc.k_floor_m2_s", 2.0e-8, "m2/s", "Floor of an observed rate before its logarithm."),
        constant("enc.w_ref", 0.08, "1", "Progress at which g_progress saturates."),
        constant("enc.min_points", 2.0, "1", "Fewest valid prefix diameters for a forecast."),
        constant("gate.growth_tol", 0.05, "1", "A diameter is dropped when later prefix values of D^2 exceed it by more than this fraction."),
        constant("gate.min_later", 3.0, "1", "Number of later contradicting values that drops a diameter."),
        constant("gate.k_max_m2_s", 2.0e-6, "m2/s", "Largest d(D^2)/dt magnitude any flame of these experiments produces (twice the fastest measured rate): a later value below what this allows contradicts an earlier one, and the level is not carried to the cutoff faster than this."),
        constant("num.dt_s", 0.05, "s", "RK4 step of the hot phase (uniform grid from the cutoff)."),
        constant("num.bin_s", 0.5, "s", "Width of the extinction-time bins the mixture atoms are built from."),
        constant("num.d_min_m", 2.5e-4, "m", "Support floor: the model stops at this diameter."),
        constant("num.t_valid_s", T_VALID_S, "s", "Model horizon since the time origin."),
        constant("num.z_step", 0.5, "1", "Spacing of the scenario nodes of z."),
        constant("num.z_count", 15.0, "1", "Number of scenario nodes of z, centred on 0."),
        constant("num.ce_step", 1.0, "1", "Spacing of the scenario nodes of the cool-flame extinction diameter."),
        constant("num.ce_count", 5.0, "1", "Number of scenario nodes of the cool-flame extinction diameter."),
    ]);
    tensors.extend([
        // ---- self-cooling evaporation of an unlit droplet
        authored("sc.on", &[], Weight, Some("1"), Some((0.0, 1.0)),
                 "Switch of the self-cooling law of a droplet without a flame, 0 or 1 (any other value is refused): 1 puts K(u) = K_floor(u) + K0 / (1 + u/t0) in place of the post.* law wherever that is used, u being the time since the flame ended; 0 keeps the post.* law and none of the sc.* entries is read."),
        authored("sc.k0_hot_mm2_s", &[], PhysicalParameter, Some("mm2/s"), Some((0.0, 2.0)),
                 "d(D^2)/dt magnitude, above the floor, of a droplet right after a hot flame left it, in a gas with g = 1: K0 = g * sc.k0_hot_mm2_s (times sc.diff_scale after a diffusive extinction)."),
        authored("sc.k0_cool_mm2_s", &[], PhysicalParameter, Some("mm2/s"), Some((0.0, 2.0)),
                 "As sc.k0_hot_mm2_s for a droplet right after a cool flame left it: K0 = g * sc.k0_cool_mm2_s (not read for a fuel without a cool-flame law)."),
        authored("sc.tstar_k", &[], PhysicalParameter, Some("K"), Some((0.0, 100.0)),
                 "Temperature the liquid has to lose for its evaporation on stored heat to stop, in the cooling time t0 = sc.tstar_k * sc.cl_j_kg_k * D_e^2 / (1.5 * K0 * sc.lat_j_kg) (D_e^2 in m2, K0 in m2/s; 0: no stored heat, the K0 term is absent)."),
        authored("sc.cl_j_kg_k", &[], PhysicalParameter, Some("J/(kg K)"), Some((0.0, 10000.0)),
                 "Specific heat of the liquid in the cooling time t0."),
        authored("sc.lat_j_kg", &[], PhysicalParameter, Some("J/kg"), Some((0.0, 5000000.0)),
                 "Latent heat of evaporation in the cooling time t0 (0: evaporation costs no heat and the rate stays at K0)."),
        authored("sc.gas", &[3], Weight, Some("1"), Some((-10.0, 10.0)),
                 "Coefficients of (x_He, x_CO2, ln(p / norm.p_ref_pa)) in the gas factor g = exp(sc.gas . that) of K0 and of the cold floor; x are ambient mole fractions (x_He = 0.20 c_he, x_CO2 = 0.20 c_co2), p the pressure."),
        authored("sc.k_cold_mm2_s", &[], PhysicalParameter, Some("mm2/s"), Some((0.0, 1.0)),
                 "Long-time d(D^2)/dt magnitude of an unlit droplet in a gas with g = 1: K_floor(u) = g * k_cold + (k_warm - g * k_cold) * exp(-u / sc.tau_warm_s)."),
        authored("sc.k_warm_mm2_s", &[], PhysicalParameter, Some("mm2/s"), Some((0.0, 1.0)),
                 "Floor of d(D^2)/dt right after the flame ended, while the surrounding gas is still warm (not multiplied by g)."),
        authored("sc.tau_warm_s", &[], PhysicalParameter, Some("s"), Some((0.01, 1000.0)),
                 "Time in which the floor goes from sc.k_warm_mm2_s to g * sc.k_cold_mm2_s."),
        authored("sc.diff_scale", &[], PhysicalParameter, Some("1"), Some((0.0, 2.0)),
                 "Multiplier of K0 and of both floor rates (sc.k_cold_mm2_s, sc.k_warm_mm2_s) for a droplet whose hot flame ended diffusively (burn-out or water extinction: the liquid left is less volatile); not applied after a radiative loss, after a cool flame, or to a droplet already without a flame at the cutoff."),
        // ---- a dying flame
        authored("dy.frac", &[], PhysicalParameter, Some("1"), Some((0.0, 2.0)),
                 "A hot flame lost to radiation that no sustained cool flame follows dies away: the unlit droplet loses D^2 at dy.frac * K_c * exp(-u / dy.tau_s) on top of its unlit law, K_c the cool-flame rate of the run (0: no such term; a fuel without a cool-flame law has K_c = 0). Never after a diffusive extinction or a cool flame that burned."),
        authored("dy.tau_s", &[], PhysicalParameter, Some("s"), Some((0.01, 100.0)),
                 "Decay time of the dying flame."),
        // ---- the cool-flame end in oxygen partial pressure
        authored("ce.on", &[], Weight, Some("1"), Some((0.0, 1.0)),
                 "Switch of the closed form of the cool-flame end diameter, 0 or 1 (any other value is refused): 1 puts D_ce = D0 * (D0 / norm.d_ref_m)^(ce.d0_exp - 1) * ce.g0 * sqrt((ce.p_ref_atm - ce.beta_atm) / max(pO2 - ce.beta_atm, ce.eps_atm)) * (1 + ce.c_he * x_He), pO2 = x_O2 * p / 101325 Pa, in place of the median of the cool.d network, and a cool flame of zero length then leaves a droplet that comes straight from a hot flame; 0 keeps the network and none of the ce.* entries is read. Needs ce.p_ref_atm > ce.beta_atm. Not read for a fuel without a cool-flame law."),
        authored("ce.g0", &[], PhysicalParameter, Some("1"), Some((0.0, 2.0)),
                 "D_ce / D0 of a droplet of D0 = norm.d_ref_m at pO2 = ce.p_ref_atm without helium."),
        authored("ce.beta_atm", &[], PhysicalParameter, Some("atm"), Some((0.0, 0.3)),
                 "Oxygen partial pressure at which the closed form of D_ce diverges (no cool flame is sustained below it)."),
        authored("ce.p_ref_atm", &[], PhysicalParameter, Some("atm"), Some((0.0, 1.0)),
                 "Oxygen partial pressure at which D_ce / D0 = ce.g0 for D0 = norm.d_ref_m without helium."),
        authored("ce.eps_atm", &[], PhysicalParameter, Some("atm"), Some((0.0001, 0.1)),
                 "Smallest pO2 - ce.beta_atm the closed form is evaluated at: closer to ce.beta_atm, or below it, D_ce is capped at its value there."),
        authored("ce.c_he", &[], PhysicalParameter, Some("1"), Some((0.0, 5.0)),
                 "Relative growth of D_ce per unit helium mole fraction: the factor (1 + ce.c_he * x_He)."),
        authored("ce.d0_exp", &[], PhysicalParameter, Some("1"), Some((0.0, 2.0)),
                 "Exponent of D0 in D_ce (1: D_ce proportional to D0)."),
    ]);
    // ---- composition of a two-component droplet and staged burning (section 3.2)
    tensors.extend([
        authored("mix.on", &[], Weight, Some("1"), Some((0.0, 1.0)),
                 "Switch of the composition input, 0 or 1 (any other value is refused): 1 reads y_b (y_b_mass, or y_b_volume converted with mix.rho_a and mix.rho_b) and puts c_y = y_b as the last input of every law network (cool.p and cool.k read y_s); 0 reads no composition and c_y = 0."),
        authored("mix.rho_a", &[], PhysicalParameter, Some("kg/m3"), Some((300.0, 2000.0)),
                 "Liquid density of component A at 298 K, for converting a volume fraction to y_b."),
        authored("mix.rho_b", &[], PhysicalParameter, Some("kg/m3"), Some((300.0, 2000.0)),
                 "Liquid density of component B at 298 K, for converting a volume fraction to y_b."),
    ]);
    if fuel.has_cool() {
        tensors.extend([
            authored("ce.beta_y", &[], PhysicalParameter, Some("atm"), Some((-1.0, 1.0)),
                     "With mix.on = 1 the oxygen limit of the closed form of the cool-flame end is ce.beta_atm + ce.beta_y * y_s."),
            authored("ce.g0_y", &[], PhysicalParameter, Some("1"), Some((-5.0, 5.0)),
                     "With mix.on = 1, ce.g0 is multiplied by exp(ce.g0_y * y_s)."),
        ]);
    }
    tensors.extend([
        authored("st.on", &[], Weight, Some("1"), Some((0.0, 1.0)),
                 "Switch of staged burning, 0 or 1 (any other value is refused); needs mix.on = 1. For y_b > 0 a hot or cool flame that reaches D_c = D0 [1 + 3 st.eps (1 - y_b)/y_b]^(-1/3) exp(st.dc_sd u) enters stage 2."),
        authored("st.eps", &[], PhysicalParameter, Some("1"), Some((0.0, 5.0)),
                 "Liquid diffusion number 8 D_l / K of the mixture in stage 1."),
        authored("st.dc_sd", &[], NoiseParameter, Some("1"), Some((0.0, 1.0)),
                 "Assumed sd of ln D_c between droplets (nodes num.dc_count spaced num.dc_step)."),
        authored("st.drift", &[], PhysicalParameter, Some("1"), Some((-1.0, 1.0)),
                 "Surface composition seen by the cool-flame laws: y_s = clamp(y_b + st.drift * w, 0, 1), w the burn progress (0: the liquid keeps its composition). Read with mix.on = 1."),
        authored("st.k2", &[], PhysicalParameter, Some("1"), Some((-3.0, 1.0)),
                 "ln of the stage-2 rate over the stage-1 law at the same state."),
        authored("st.dip_amp", &[], PhysicalParameter, Some("1"), Some((0.0, 1.0)),
                 "At D_c the rate is multiplied by 1 - st.dip_amp * exp(-(s - s_c) / st.dip_clock): the surface heating to its new temperature (section 3.2's st.dip[0])."),
        authored("st.dip_clock", &[], PhysicalParameter, Some("clock"), Some((0.001, 2.0)),
                 "Clock length of the dip at D_c (section 3.2's st.dip[1]; s = t * norm.k_ref_m2_s / q0)."),
    ]);
    tensors.extend(net(
        "st.flash",
        hl,
        nl,
        "the flash-extinction logit at D_c",
        law_in,
    ));
    tensors.extend([
        authored("st.flash.b0", &[], HazardParameter, Some("logit(1)"), Some((-30.0, 30.0)),
                 "Logit of the probability that the hot flame is lost at D_c (flash extinction), before hinge terms."),
        authored("st.log_disruption2", &[], HazardParameter, Some("ln(1/s)"), Some((-20.0, 3.0)),
                 "ln of the disruption hazard of a hot flame in stage 2, in place of haz.log_disruption."),
    ]);
    // ---- large alkanes (section 3.3)
    if fuel.has_cool() {
        tensors.push(authored("warm.on", &[], Weight, Some("1"), Some((0.0, 1.0)),
            "Switch of the warm-flame stage after a radiative loss, 0 or 1 (any other value is refused)."));
        tensors.extend(net(
            "warm.p",
            hl,
            nl,
            "the logit that the stage after a radiative loss begins as a warm flame",
            law_in,
        ));
        tensors.extend([
            authored(
                "warm.p.b0",
                &[],
                HazardParameter,
                Some("logit(1)"),
                Some((-30.0, 30.0)),
                "Logit of a warm flame after a radiative loss at x = 0, before hinge terms.",
            ),
            authored(
                "warm.k",
                &[],
                PhysicalParameter,
                Some("1"),
                Some((-2.0, 2.0)),
                "ln(K_warm / K_c): the warm flame burns at K_c exp(warm.k).",
            ),
        ]);
        tensors.extend(net(
            "warm.s",
            hl,
            nl,
            "the law ln(s_W) of the median clock length of the warm stage",
            law_in,
        ));
        tensors.extend([
            authored("warm.s.b0", &[], HazardParameter, Some("ln(1)"), Some((-8.0, 4.0)),
                     "ln of the median clock length of the warm stage at x = 0, before hinge terms."),
            authored("warm.sharp", &[], HazardParameter, Some("1"), Some((0.5, 40.0)),
                     "Gompertz sharpness of the end of the warm stage in s/s_W, as rad.sharp."),
            authored("ig.on", &[], Weight, Some("1"), Some((0.0, 1.0)),
                     "Switch of direct cool ignition, 0 or 1 (any other value is refused): 1 reads the conditions igniter_power and igniter_time."),
        ]);
        tensors.extend(net(
            "ig.cool",
            hl,
            N_IGN as u32,
            "the logit that ignition gives a cool flame and no hot flame",
            "c_o2, c_co2, c_he, c_logp, c_logd0, x_exc, c_y, c_ign",
        ));
        tensors.push(authored(
            "ig.cool.b0",
            &[],
            HazardParameter,
            Some("logit(1)"),
            Some((-30.0, 30.0)),
            "Logit of a cool flame lit directly by the igniter at x = 0, before hinge terms.",
        ));
    }
    tensors.extend([
        authored("fib.on", &[], Weight, Some("1"), Some((0.0, 1.0)),
                 "Switch of fibre-supported droplets, 0 or 1 (any other value is refused): 1 reads the condition fiber (0 or 1); 0 refuses a prefix with fiber = 1."),
        authored("fib.rate", &[], PhysicalParameter, Some("1"), Some((-2.0, 2.0)),
                 "Added to ln K_h for a supported droplet (heat conducted along the fibre)."),
        authored("fib.crit", &[], PhysicalParameter, Some("1"), Some((-2.0, 2.0)),
                 "Added to ln D_crit for a supported droplet."),
        constant("num.dc_step", 1.0, "1", "Spacing of the scenario nodes of ln D_c."),
        constant("num.dc_count", 5.0, "1", "Number of scenario nodes of ln D_c."),
    ]);
    if fuel.has_cool() {
        tensors.extend([
            constant(
                "num.warm_count",
                5.0,
                "1",
                "Number of equal-mass quantile nodes of the warm-stage clock length.",
            ),
            constant(
                "ig.e_ref",
                1.0,
                "J",
                "Reference igniter energy of c_ign = ln(igniter_power * igniter_time / ig.e_ref).",
            ),
        ]);
    }
    let mut excludes = vec![
        "visible or UV flame diameter".to_string(),
        "flame temperature, soot, species or radiant emission".to_string(),
        "fibre-supported droplets unless fib.on = 1, and other fuels".to_string(),
        "reignition of the hot flame".to_string(),
        "a calibrated posterior: the spread is an assumed scenario distribution".to_string(),
    ];
    match fuel {
        Fuel::Methanol => excludes.push("cool-flame burning (not observed for methanol)".into()),
        f if !f.has_cool() => excludes.push(format!(
            "cool-flame burning (no cool-flame law for {})",
            f.label().to_lowercase()
        )),
        _ => {}
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
            feature("r_late", "observation:liquid_diameter@prefix_mid_tail", "K_obs / K_model of the late finite difference, unclamped (reported) @2"),
            feature("r_end", "observation:liquid_diameter@prefix_end", "K_obs / K_model of the finite difference between the last two end windows (enc.end_frac of the prefix each), unclamped: the state evidence @3"),
            feature("g_span", "observation:time@prefix", "ln((mean time of tail window - mean time of head window) / 1 s) @1"),
            feature("g_count", "observation:count@prefix", "ln(n_valid / norm.n_ref) @1"),
            feature("g_progress", "state:burn_progress@prefix_tail", "clamp(w_tail / enc.w_ref, 0, 1) @1"),
            feature("c_y", "condition:y_b_mass|y_b_volume",
                "mass fraction of component B in the liquid as deployed, y_b (a volume fraction phi converted: rho_b phi / (rho_a (1 - phi) + rho_b phi)); 0 with mix.on = 0 @1"),
        feature("c_ign", "condition:igniter_power,igniter_time",
                "ln(igniter_power * igniter_time / ig.e_ref), read with ig.on = 1 @1"),
        feature("fibre", "condition:fiber", "1 for a fibre-supported droplet, 0 for a free one, read with fib.on = 1 @1"),
        feature("flame_seen", "observation:uv_flame_diameter|visible_flame_diameter@prefix", "1 when the last positive flame diameter of either camera that has two more within fl.gap_s before it lies within fl.gap_s of the cutoff, 0 when the cameras recorded the flame (fl.min_points positive values after fl.t_min_s) and no longer show it; absent when they did not record it @1"),
        ],
        domain: vec![
            bound("c_o2", -2.2, 4.8),
            bound("c_co2", 0.0, 4.0),
            bound("c_he", 0.0, C_HE_MAX),
            bound("c_logp", -1.0, C_LOGP_MAX),
            bound("c_logd0", -0.9, 1.26),
        ],
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::droplet9::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(
            "Prefix observations: liquid_diameter, valid, positive, 0 <= t <= cutoff, first of equal \
             time stamps, then the validity gate (a value is dropped when at least gate.min_later later \
             prefix values of D^2 contradict it: larger by more than gate.growth_tol, since a droplet \
             does not grow, or smaller than gate.k_max_m2_s allows in the time between). \
             Level at the cutoff: mean of D^2 over the last end window carried to the cutoff along the \
             end rate (finite difference of the last two end windows) limited to [0, min(gate.k_max_m2_s, \
             twice the mid -> tail rate)] \
             (no weight enters it). Survival of the radiative class at clock s: S_R(s) = (1 - rob.eps_r) \
             exp(-Lambda_R(s)) + rob.eps_r / (1 + s / s_R), in the prior at the cutoff and, divided by its \
             value at the cutoff, in the forecast. State probabilities at the cutoff: \
             prior from the extinction laws at the observed progress and clock, times \
             (N(r_end; mean_state, sd_state) + filt.l_floor)^g, normalised, g = sigmoid(filt.gain_b + \
             filt.gain_w . (g_span, g_count, g_progress)); means 1 (hot), K_c/K_h (cool), k_evap/K_h \
             (no flame, half of the post-extinction transient left); sds from filt.sd_hot, filt.sd_evap \
             and the finite-difference noise implied by unc.sigma_frame. Run offset of the hot flame: \
             gain times the clamped mid -> tail and head -> mid innovations; of a cool flame burning at \
             the cutoff: enc.cool_gain * g * clamped ln(end rate / K_c). A droplet without a flame at \
             the cutoff evaporates with the transient amplitude its end rate shows (weight g) or half \
             of post.k_hot (weight 1 - g). Flame cameras (fl.trust > 0, and the prefix holds at least \
             fl.min_points positive uv_flame_diameter or visible_flame_diameter values in [fl.t_min_s, \
             cutoff]): t_last is the last of those times that has two more within fl.gap_s before it; \
             the flame is seen when cutoff - t_last <= fl.gap_s. The state probabilities after the rate \
             evidence, s, become fl.trust * v + (1 - fl.trust) * s with v = (1, 0, 0) when the flame is \
             seen and v = (0, s_cool, s_none) / (s_cool + s_none) otherwise (the law's cool-flame \
             probability at the cutoff in place of that ratio when s_cool + s_none = 0). When the flame \
             is not seen the transient amplitude of a droplet without a flame is fl.evap_mix * \
             exp(-(cutoff - t_last) / tau_e) + (1 - fl.evap_mix) * (the end-rate estimate). Nothing \
             else is read from the flame diameters. Self-cooling law (sc.on = 1), in place of the \
             post.* law of a droplet without a flame: fall of D^2 over the first u seconds after the \
             flame ended at D_e^2 = q_e, J(u) = K0 t0 ln(1 + u/t0) + g f k_cold u + (f k_warm - g f \
             k_cold) sc.tau_warm_s (1 - exp(-u/sc.tau_warm_s)), t0 = sc.tstar_k sc.cl_j_kg_k q_e / (1.5 K0 \
             sc.lat_j_kg), g = exp(sc.gas . (x_He, x_CO2, ln(p/norm.p_ref_pa))), K0 = g f sc.k0_hot_mm2_s \
             after a hot flame and g sc.k0_cool_mm2_s after a cool flame (where the post.* law applies \
             post.cool_scale), f = sc.diff_scale after a diffusive extinction and 1 otherwise, rates \
             converted from mm2/s. No-flame mean of the state evidence: (K_floor(0) + K0 / 2) / K_h with \
             the hot K0 and f = 1. A droplet without a flame at the cutoff: J restarted at the cutoff \
             from the diameter at the cutoff with K0 times (g_state * clamp((end rate - K_floor(0)) / \
             K0, 0, 3) + (1 - g_state) / 2); when the cameras lost the flame at t_last, fl.evap_mix * \
             (J(t - t_last) - J(cutoff - t_last)) + (1 - fl.evap_mix) * that. Dying flame (dy.frac > 0): \
             after a radiative loss with no cool flame, and for a cool flame of zero length when ce.on \
             = 1, the fall of D^2 gains dy.frac K_c dy.tau_s (1 - exp(-u/dy.tau_s)), K_c the cool-flame \
             rate of the scenario. Cool-flame end (ce.on = 1): the median D_ce is D0 (D0/norm.d_ref_m)^\
             (ce.d0_exp - 1) ce.g0 sqrt((ce.p_ref_atm - ce.beta_atm) / max(pO2 - ce.beta_atm, \
             ce.eps_atm)) (1 + ce.c_he x_He), pO2 = x_O2 p / 101325 Pa, D0 = sqrt(q0), in place of the \
             cool.d network's; a node with D_ce at or above the diameter at the hot flame's loss then \
             leaves a droplet that comes straight from a hot flame (with ce.on = 0: from a cool flame). \
             With sc.on = 0, dy.frac = 0 and ce.on = 0 the forecast is that of droplet5.rs. The law is \
             that of droplet6.rs; this forward adds fuels, widens c_logp and c_he and moves the horizon. \
             The prefix's fuel condition names this fuel after lower-casing and dropping a leading n- and \
             every space, hyphen, underscore and comma, component by component. Sections 3.2 and 3.3 \
             (droplet9.rs): composition c_y (mix.on), staged burning at D_c with flash extinction, a \
             stage-2 rate factor exp(st.k2) (1 - st.dip_amp exp(-(s - s_c)/st.dip_clock)) and stage-2 \
             disruption (st.on), a warm stage after a radiative loss (warm.on), direct cool ignition \
             (ig.on), fibre-supported droplets (fib.on); with every switch off the forecast is that of \
             droplet8.rs. \
             Hot phase: RK4 on a uniform grid of \
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
    hot_exp: f64,
    tau_exp: f64,
    gain_w: &'a [f64],
    gain_b: f64,
    mix: &'a [f64],
    e_lo: f64,
    e_hi: f64,
    cool_gain: f64,
    sd_hot: f64,
    sd_evap: f64,
    l_floor: f64,
    fgain_w: &'a [f64],
    fgain_b: f64,
    end_frac: f64,
    eps_r: f64,
    fl_trust: f64,
    fl_t_min: f64,
    fl_gap: f64,
    fl_min_points: f64,
    fl_evap_mix: f64,
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
    // self-cooling evaporation (rates in m^2/s)
    sc_on: bool,
    sc_k0_hot: f64,
    sc_k0_cool: f64,
    sc_tstar: f64,
    sc_cl: f64,
    sc_lat: f64,
    sc_gas: &'a [f64],
    sc_k_cold: f64,
    sc_k_warm: f64,
    sc_tau_warm: f64,
    sc_diff_scale: f64,
    // a dying flame
    dy_frac: f64,
    dy_tau: f64,
    // the cool-flame end in oxygen partial pressure
    ce_on: bool,
    ce_g0: f64,
    ce_beta: f64,
    ce_p_ref: f64,
    ce_eps: f64,
    ce_c_he: f64,
    ce_d0_exp: f64,
    // composition and staged burning (section 3.2)
    mix_on: bool,
    rho_a: f64,
    rho_b: f64,
    ce_beta_y: f64,
    ce_g0_y: f64,
    st_on: bool,
    st_eps: f64,
    st_dc_sd: f64,
    st_drift: f64,
    st_k2: f64,
    st_dip_amp: f64,
    st_dip_clock: f64,
    st_flash: Net<'a>,
    lam_dis2: f64,
    dc_step: f64,
    dc_count: usize,
    // large alkanes (section 3.3)
    warm: Option<Warm<'a>>,
    ig: Option<(Net<'a>, f64)>,
    fib_on: bool,
    fib_rate: f64,
    fib_crit: f64,
}

/// The warm-flame stage of a release with `warm.on = 1`.
struct Warm<'a> {
    p: Net<'a>,
    k: f64,
    s: Net<'a>,
    sharp: f64,
    count: usize,
}

/// What a prefix says about the droplet beyond the gas: its composition and whether a fibre holds
/// it. With `mix.on = 0` and `fib.on = 0` every field is neutral.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Run {
    /// Mass fraction of component B, the input `c_y` (0 with `mix.on = 0`).
    pub y_b: f64,
    /// `mass` or `volume`: the basis the prefix stated (absent with `mix.on = 0`).
    pub y_basis: Option<&'static str>,
    /// The condition `fiber`, when the prefix carries it.
    pub fibre: Option<bool>,
    /// `fib.rate` for a supported droplet with `fib.on = 1`.
    pub rate_offset: Option<f64>,
    /// `fib.crit` for a supported droplet with `fib.on = 1`.
    pub crit_offset: Option<f64>,
    /// `ln(igniter_power * igniter_time / ig.e_ref)` with `ig.on = 1`.
    pub c_ign: Option<f64>,
}

impl Run {
    /// The run of `droplet8.rs`: no composition, no fibre, no igniter.
    pub const NEUTRAL: Run = Run {
        y_b: 0.0,
        y_basis: None,
        fibre: None,
        rate_offset: None,
        crit_offset: None,
        c_ign: None,
    };
}

/// The ambient gas in physical units, as the laws added by this forward read it.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Gas {
    pub x_o2: f64,
    pub x_co2: f64,
    pub x_he: f64,
    pub pressure_pa: f64,
}

/// What the flame was that an unlit droplet last had: it sets the amplitude of its evaporation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Last {
    /// A hot flame lost to radiation (or a hot flame whose end is not known).
    Hot,
    /// A hot flame that ended diffusively: burn-out, or water taken up by the liquid.
    HotDiffusive,
    /// A cool flame.
    Cool,
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
        let switch = |name: &str| -> Result<bool, Refusal> {
            let v = p.s(name);
            if v == 0.0 || v == 1.0 {
                Ok(v == 1.0)
            } else {
                Err(Refusal::failed(
                    "switch_value",
                    format!("{name} = {v} is neither 0 (off) nor 1 (on)"),
                ))
            }
        };
        let (sc_on, ce_on) = (switch("sc.on")?, switch("ce.on")?);
        let (mix_on, st_on, fib_on) = (switch("mix.on")?, switch("st.on")?, switch("fib.on")?);
        let (warm_on, ig_on) = if fuel.has_cool() {
            (switch("warm.on")?, switch("ig.on")?)
        } else {
            (false, false)
        };
        // the oxygen limit of the cool-flame end must stay below ce.p_ref_atm for every surface
        // composition the release can see
        let beta_max = if mix_on && fuel.has_cool() {
            p.s("ce.beta_atm") + p.s("ce.beta_y").max(0.0)
        } else if fuel.has_cool() {
            p.s("ce.beta_atm")
        } else {
            0.0
        };
        if ce_on && fuel.has_cool() && !(p.s("ce.p_ref_atm") > beta_max) {
            return Err(Refusal::failed(
                "cool_end_form",
                "ce.on = 1 needs ce.p_ref_atm > ce.beta_atm (+ ce.beta_y when it is positive and mix.on = 1)",
            ));
        }
        if st_on && !mix_on {
            return Err(Refusal::failed(
                "switch_value",
                "st.on = 1 needs mix.on = 1 (staged burning reads the composition)",
            ));
        }
        let dc_count = p.s("num.dc_count") as usize;
        if !(1..=MAX_DC).contains(&dc_count) || dc_count.is_multiple_of(2) {
            return Err(Refusal::failed(
                "spec_constants",
                "num.dc_count must be odd, 1..=16",
            ));
        }
        let warm = if warm_on {
            let count = p.s("num.warm_count") as usize;
            if !(1..=MAX_WARM).contains(&count) {
                return Err(Refusal::failed(
                    "spec_constants",
                    "num.warm_count must be 1..=16",
                ));
            }
            Some(Warm {
                p: net("warm.p"),
                k: p.s("warm.k"),
                s: net("warm.s"),
                sharp: p.s("warm.sharp"),
                count,
            })
        } else {
            None
        };
        let ig = ig_on.then(|| (net("ig.cool"), p.s("ig.e_ref")));
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
            hot_exp: p.s("post.hot_exp"),
            tau_exp: p.s("post.tau_exp"),
            gain_w: p.v("enc.gain_w"),
            gain_b: p.s("enc.gain_b"),
            mix: p.v("enc.mix"),
            e_lo: clip[0],
            e_hi: clip[1],
            cool_gain: p.s("enc.cool_gain"),
            sd_hot: p.s("filt.sd_hot"),
            sd_evap: p.s("filt.sd_evap"),
            l_floor: p.s("filt.l_floor"),
            fgain_w: p.v("filt.gain_w"),
            fgain_b: p.s("filt.gain_b"),
            end_frac: p.s("enc.end_frac"),
            eps_r: p.s("rob.eps_r"),
            fl_trust: p.s("fl.trust"),
            fl_t_min: p.s("fl.t_min_s"),
            fl_gap: p.s("fl.gap_s"),
            fl_min_points: p.s("fl.min_points"),
            fl_evap_mix: p.s("fl.evap_mix"),
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
            sc_on,
            sc_k0_hot: MM2 * p.s("sc.k0_hot_mm2_s"),
            sc_k0_cool: MM2 * p.s("sc.k0_cool_mm2_s"),
            sc_tstar: p.s("sc.tstar_k"),
            sc_cl: p.s("sc.cl_j_kg_k"),
            sc_lat: p.s("sc.lat_j_kg"),
            sc_gas: p.v("sc.gas"),
            sc_k_cold: MM2 * p.s("sc.k_cold_mm2_s"),
            sc_k_warm: MM2 * p.s("sc.k_warm_mm2_s"),
            sc_tau_warm: p.s("sc.tau_warm_s"),
            sc_diff_scale: p.s("sc.diff_scale"),
            dy_frac: p.s("dy.frac"),
            dy_tau: p.s("dy.tau_s"),
            ce_on,
            ce_g0: p.s("ce.g0"),
            ce_beta: p.s("ce.beta_atm"),
            ce_p_ref: p.s("ce.p_ref_atm"),
            ce_eps: p.s("ce.eps_atm"),
            ce_c_he: p.s("ce.c_he"),
            ce_d0_exp: p.s("ce.d0_exp"),
            mix_on,
            rho_a: p.s("mix.rho_a"),
            rho_b: p.s("mix.rho_b"),
            ce_beta_y: if fuel.has_cool() {
                p.s("ce.beta_y")
            } else {
                0.0
            },
            ce_g0_y: if fuel.has_cool() { p.s("ce.g0_y") } else { 0.0 },
            st_on,
            st_eps: p.s("st.eps"),
            st_dc_sd: p.s("st.dc_sd"),
            st_drift: p.s("st.drift"),
            st_k2: p.s("st.k2"),
            st_dip_amp: p.s("st.dip_amp"),
            st_dip_clock: p.s("st.dip_clock"),
            st_flash: net("st.flash"),
            lam_dis2: p.s("st.log_disruption2").exp(),
            dc_step: p.s("num.dc_step"),
            dc_count,
            warm,
            ig,
            fib_on,
            fib_rate: p.s("fib.rate"),
            fib_crit: p.s("fib.crit"),
        })
    }

    /// Surface composition seen by the cool-flame laws at burn progress `w`.
    #[inline]
    fn y_surface(&self, run: &Run, w: f64) -> f64 {
        if self.mix_on {
            (run.y_b + self.st_drift * w).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// `ln(K_h / K_ref)` of the rate network at `x`, with the fibre's offset.
    #[inline]
    fn ln_rate(&self, x: &[f64; N_RATE], run: &Run, scratch: &mut [f64; H_RATE]) -> f64 {
        let v = self.rate.eval(x, scratch);
        match run.rate_offset {
            Some(o) => v + o,
            None => v,
        }
    }

    /// Hot-flame population burning-rate constant (m^2/s), without run offset or scenario.
    #[inline]
    pub(crate) fn k_hot_pop(
        &self,
        c: &[f64; 5],
        w: f64,
        s: f64,
        run: &Run,
        scratch: &mut [f64; H_RATE],
    ) -> f64 {
        let x = [c[0], c[1], c[2], c[3], c[4], w, s, run.y_b];
        self.k_ref * self.ln_rate(&x, run, scratch).exp()
    }

    /// Harmonic Simpson mean of the hot-flame population rate between two prefix windows `(t, q)`.
    fn k_mean(
        &self,
        c: &[f64; 5],
        q0: f64,
        a: (f64, f64),
        b: (f64, f64),
        run: &Run,
        scratch: &mut [f64; H_RATE],
    ) -> f64 {
        let w = |q: f64| (1.0 - q / q0).clamp(0.0, 1.0);
        let s = |t: f64| t * self.k_ref / q0;
        let (wa, wb, sa, sb) = (w(a.1), w(b.1), s(a.0), s(b.0));
        let inv = (1.0 / self.k_hot_pop(c, wa, sa, run, scratch)
            + 4.0 / self.k_hot_pop(c, 0.5 * (wa + wb), 0.5 * (sa + sb), run, scratch)
            + 1.0 / self.k_hot_pop(c, wb, sb, run, scratch))
            / 6.0;
        1.0 / inv
    }

    /// The condition-level laws of one droplet of initial diameter `d0` (m) in `gas`.
    /// The cool-flame laws read the surface composition `y_s`; every other law reads `run.y_b`.
    pub(crate) fn laws(
        &self,
        c: &[f64; 5],
        d0: f64,
        gas: &Gas,
        run: &Run,
        y_s: f64,
        scratch: &mut [f64; H_RATE],
    ) -> Laws {
        let y = run.y_b;
        let ln_dcrit = self.crit.eval(&[c[0], c[1], c[2], c[3], y], scratch);
        let ln_dcrit = match run.crit_offset {
            Some(o) => ln_dcrit + o,
            None => ln_dcrit,
        };
        let x_exc = c[4] - ln_dcrit;
        let x = [c[0], c[1], c[2], c[3], c[4], x_exc, y];
        let pi_r = sigmoid(self.kappa * x_exc + self.class_bias);
        let w_d = sigmoid(self.diff.eval(&x, scratch));
        let s_r = self.rad.eval(&x, scratch).exp();
        let (p_cool, k_cool, d_ce_net, sd_ce) = match &self.cool {
            Some(cool) => (
                sigmoid(
                    cool.p
                        .eval(&[c[0], c[1], c[2], c[3], c[4], x_exc, y_s], scratch),
                ),
                self.k_ref
                    * cool
                        .k
                        .eval(&[c[0], c[1], c[2], c[3], c[4], y_s], scratch)
                        .exp(),
                self.d_ref
                    * cool
                        .d
                        .eval(&[c[0], c[1], c[2], c[3], c[4], y], scratch)
                        .exp(),
                cool.sd,
            ),
            None => (0.0, 0.0, 0.0, 0.0),
        };
        // the cool-flame end: the closed form in oxygen partial pressure, or the network
        let d_ce = if self.ce_on && self.cool.is_some() {
            self.d_ce_form(d0, gas, y_s).d_ce_m
        } else {
            d_ce_net
        };
        let p_flash = if self.st_on {
            sigmoid(self.st_flash.eval(&x, scratch))
        } else {
            0.0
        };
        let (p_warm, s_warm) = match &self.warm {
            Some(wm) => (
                sigmoid(wm.p.eval(&x, scratch)),
                wm.s.eval(&x, scratch).exp(),
            ),
            None => (0.0, 0.0),
        };
        let p_ig = match (&self.ig, run.c_ign) {
            (Some((n, _)), Some(ci)) => {
                sigmoid(n.eval(&[x[0], x[1], x[2], x[3], x[4], x[5], x[6], ci], scratch))
            }
            _ => 0.0,
        };
        Laws {
            p_flash,
            p_warm,
            s_warm,
            p_ignite_cool: p_ig,
            d_crit_m: self.d_ref * ln_dcrit.exp(),
            x_exc,
            pi_r,
            w_d,
            s_r,
            p_cool,
            k_cool_m2_s: k_cool,
            d_ce_m: d_ce,
            sd_ce,
            d_ce_net_m: d_ce_net,
        }
    }

    /// Median diameter at which a cool flame ends, from the closed form in the oxygen partial
    /// pressure: `D0 (D0/D_ref)^(n - 1) g0 sqrt((p_ref - beta) / max(pO2 - beta, eps)) (1 + c_he x_He)`.
    fn d_ce_form(&self, d0: f64, gas: &Gas, y_s: f64) -> CoolEnd {
        let (beta, g0) = if self.mix_on {
            (
                self.ce_beta + self.ce_beta_y * y_s,
                self.ce_g0 * (self.ce_g0_y * y_s).exp(),
            )
        } else {
            (self.ce_beta, self.ce_g0)
        };
        let po2 = gas.x_o2 * gas.pressure_pa / ATM_PA;
        let excess = po2 - beta;
        let d_ce = d0
            * (d0 / self.d_ref).powf(self.ce_d0_exp - 1.0)
            * g0
            * ((self.ce_p_ref - beta) / excess.max(self.ce_eps)).sqrt()
            * (1.0 + self.ce_c_he * gas.x_he);
        CoolEnd {
            po2_atm: po2,
            capped: !(excess >= self.ce_eps),
            d_ce_m: d_ce,
        }
    }

    /// Gas factor `g` of the self-cooling law.
    fn sc_gas_factor(&self, gas: &Gas) -> f64 {
        (self.sc_gas[0] * gas.x_he
            + self.sc_gas[1] * gas.x_co2
            + self.sc_gas[2] * (gas.pressure_pa / self.p_ref).ln())
        .exp()
    }

    /// `(K0, g f k_cold, f k_warm)` in m^2/s of the self-cooling law for a droplet whose last flame
    /// was `last`, in a gas of factor `g`; `f` is `sc.diff_scale` after a diffusive extinction.
    fn sc_rates(&self, g: f64, last: Last) -> [f64; 3] {
        let (k0, f) = match last {
            Last::Hot => (self.sc_k0_hot, 1.0),
            Last::HotDiffusive => (self.sc_k0_hot, self.sc_diff_scale),
            Last::Cool => (self.sc_k0_cool, 1.0),
        };
        [g * f * k0, g * f * self.sc_k_cold, f * self.sc_k_warm]
    }

    /// Cooling time `t0` (s) of a droplet that its flame left at `q_e` evaporating at `k0`: the
    /// stored heat over the rate at which evaporation spends it. Without a latent heat nothing is
    /// spent and the droplet does not cool.
    fn sc_t0(&self, q_e: f64, k0: f64) -> f64 {
        let spent = 1.5 * k0 * self.sc_lat;
        if spent > 0.0 {
            self.sc_tstar * self.sc_cl * q_e / spent
        } else {
            f64::INFINITY
        }
    }

    /// Fall of D^2 over the first `u` seconds without a flame under the self-cooling law, for a
    /// droplet left at `q_e` with `rates = (K0, cold floor, warm floor)`: the integral of
    /// `K0 / (1 + u/t0) + cold + (warm - cold) exp(-u / tau_warm)`.
    fn sc_fall(&self, u: f64, q_e: f64, rates: [f64; 3]) -> f64 {
        let [k0, cold, warm] = rates;
        let u = u.max(0.0);
        let t0 = self.sc_t0(q_e, k0);
        let heat = if !(k0 > 0.0) || !(t0 > 0.0) {
            0.0
        } else if t0.is_finite() {
            k0 * t0 * (u / t0).ln_1p()
        } else {
            k0 * u
        };
        heat + cold * u + (warm - cold) * self.sc_tau_warm * (1.0 - (-u / self.sc_tau_warm).exp())
    }

    /// d(D^2)/dt magnitude under the self-cooling law right after the flame ended at `q_e`, with
    /// the share `left` of `K0`.
    fn sc_rate0(&self, q_e: f64, rates: [f64; 3], left: f64) -> f64 {
        let [k0, _, warm] = rates;
        if k0 > 0.0 && self.sc_t0(q_e, k0) > 0.0 {
            warm + left * k0
        } else {
            warm
        }
    }

    /// Fall of D^2 over the first `u` seconds after a hot flame that dies away from the rate
    /// `k_die = dy.frac K_c`.
    fn die_fall(&self, u: f64, k_die: f64) -> f64 {
        k_die * self.dy_tau * (1.0 - (-u.max(0.0) / self.dy_tau).exp())
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

    /// Survival of the radiative class at clock `s`: the law, and with weight `rob.eps_r` a clock of
    /// the same median and a very wide spread.
    #[inline]
    fn surv_r(&self, l: &Laws, s: f64) -> f64 {
        let law = (-self.lam_r(l, s)).exp();
        if self.eps_r > 0.0 {
            (1.0 - self.eps_r) * law + self.eps_r / (1.0 + s.max(0.0) / l.s_r)
        } else {
            law
        }
    }

    /// Fall of D^2 over `dt` seconds for a droplet that lost its flame at `q_e`.
    #[inline]
    fn evap_drop(&self, dt: f64, q_e: f64, hot_scale: f64) -> f64 {
        let rel = q_e / (self.d_ref * self.d_ref);
        let tau_e = (self.tau0 * rel.powf(self.tau_exp)).max(1.0e-3);
        self.k_inf * rel.powf(self.size_exp) * dt
            + hot_scale * self.hot_amp(rel) * tau_e * (1.0 - (-dt / tau_e).exp())
    }

    /// d(D^2)/dt magnitude right after extinction at `q_e`.
    #[inline]
    fn evap_rate0(&self, q_e: f64, hot_scale: f64) -> f64 {
        let rel = q_e / (self.d_ref * self.d_ref);
        self.k_inf * rel.powf(self.size_exp) + hot_scale * self.hot_amp(rel)
    }

    /// Amplitude of the extra evaporation right after extinction at `rel = q_e / q_ref`.
    #[inline]
    fn hot_amp(&self, rel: f64) -> f64 {
        self.k_hot * rel.powf(self.hot_exp)
    }
}

/// Condition-level laws of one droplet (reported in the diagnostics).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Laws {
    /// Probability of a flash extinction at D_c (0 with `st.on = 0`).
    pub p_flash: f64,
    /// Probability that the stage after a radiative loss begins as a warm flame (0 with
    /// `warm.on = 0`).
    pub p_warm: f64,
    /// Median clock length of the warm stage.
    pub s_warm: f64,
    /// Probability that ignition gives a cool flame and no hot flame (0 with `ig.on = 0`).
    pub p_ignite_cool: f64,
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
    /// Median diameter at which a cool flame ends: the closed form when `ce.on = 1`, else the
    /// `cool.d` network.
    pub d_ce_m: f64,
    pub sd_ce: f64,
    /// What the `cool.d` network gives for that median (not used when `ce.on = 1`).
    pub d_ce_net_m: f64,
}

/// The closed form of the cool-flame end diameter at work.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct CoolEnd {
    /// Oxygen partial pressure, `x_O2 p / 101325 Pa`.
    pub po2_atm: f64,
    /// `pO2 - ce.beta_atm` was below `ce.eps_atm` and the cap stood in for it.
    pub capped: bool,
    pub d_ce_m: f64,
}

/// The self-cooling law of this run (`sc.on = 1`).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SelfCooling {
    /// Gas factor `g`.
    pub gas_factor: f64,
    /// `K0` of a droplet straight from a hot flame lost to radiation.
    pub k0_hot_m2_s: f64,
    /// `K0` after a diffusive extinction of the hot flame (`sc.diff_scale` applied).
    pub k0_hot_diffusive_m2_s: f64,
    /// `K0` after a cool flame.
    pub k0_cool_m2_s: f64,
    /// Cooling time of a droplet that a hot flame leaves at the diameter of the cutoff (absent when
    /// the droplet does not cool).
    pub t0_hot_s: Option<f64>,
    /// Cooling time of a droplet that a cool flame leaves at the diameter of the cutoff.
    pub t0_cool_s: Option<f64>,
    /// Floor right after the flame ended (`sc.k_warm_mm2_s`).
    pub k_floor_start_m2_s: f64,
    /// Floor at long times (`g sc.k_cold_mm2_s`).
    pub k_floor_end_m2_s: f64,
    /// Multiplier of `K0` and of both floors after a diffusive extinction.
    pub diff_scale: f64,
}

/// The dying flame of this run (`dy.frac > 0` and the fuel has a cool-flame law).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Dying {
    /// `dy.frac K_c` at the population cool-flame rate (the forecast applies the run offset and
    /// the scenario of each hot path on top).
    pub k_die_m2_s: f64,
    pub tau_s: f64,
}

/// The laws this forward adds, as they stand for one prefix (reported in the diagnostics). A law
/// that is switched off is absent.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Added {
    pub gas: Gas,
    pub self_cooling: Option<SelfCooling>,
    pub dying: Option<Dying>,
    pub cool_end: Option<CoolEnd>,
    /// What a droplet comes from when its cool flame would have zero length: `hot_flame` under the
    /// closed form of the cool-flame end, `cool_flame` otherwise.
    pub zero_length_cool_flame_counts_as: &'static str,
}

/// How a droplet without a flame at the cutoff is carried forward (reported in the diagnostics).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct UnlitAtCutoff {
    /// `self_cooling` or `post`.
    pub law: &'static str,
    /// The flame the droplet counts as coming from: `hot_flame`.
    pub origin: &'static str,
    /// Share of the transient amplitude (`post.k_hot`, or `K0`) the end rate shows, mixed with one
    /// half by the state-evidence exponent.
    pub share_from_end_rate: f64,
    /// Seconds since the cameras lost the flame, when they did.
    pub since_flame_lost_s: Option<f64>,
    /// Weight of the law run from the time the cameras lost the flame (`fl.evap_mix`, 0 without
    /// that time).
    pub weight_from_flame_lost: f64,
    /// `K0` the law restarts with at the cutoff (self-cooling law only).
    pub k0_m2_s: Option<f64>,
    /// Its cooling time (self-cooling law only; absent when the droplet does not cool).
    pub t0_s: Option<f64>,
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
    /// Finite-difference rate between the last two end windows.
    pub k_obs_end_m2_s: f64,
    pub k_model_end_m2_s: f64,
    /// End observed over modelled hot rate, unclamped: the state evidence.
    pub r_end: f64,
    /// Sd of `r_end` from frame noise alone.
    pub r_end_noise: f64,
    /// Exponent of the state evidence.
    pub gain_state: f64,
    /// Mean time of the window the level at the cutoff is carried from.
    pub t_level_s: f64,
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
    /// What the flame cameras showed in the prefix.
    pub flame: FlameSeen,
    /// Probabilities of (hot, cool, no flame) after the rate evidence, before the cameras.
    pub state_before_cameras: [f64; 3],
    /// The laws this forward adds, as they stand for this prefix.
    pub added: Added,
    /// Composition, fibre and igniter of this prefix (neutral when the release reads none).
    pub run: Run,
    /// Staged burning of this prefix (absent with `st.on = 0` or `y_b = 0`).
    pub stage: Option<Stage>,
    /// State at the cutoff.
    pub q_cutoff_m2: f64,
    pub cutoff_s: f64,
}

/// The flame cameras in a prefix: whether they recorded this flame and whether they still show it.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct FlameSeen {
    /// Positive flame diameters in `[fl.t_min_s, cutoff]`, both cameras.
    pub n: usize,
    /// Time of the last of them that has two more within `fl.gap_s` before it.
    pub t_last_s: Option<f64>,
    /// The cameras recorded this flame (`n >= fl.min_points` and such a time exists).
    pub recorded: bool,
    /// Recorded, and `t_last_s` is within `fl.gap_s` of the cutoff.
    pub seen: bool,
    /// The evidence was applied (`fl.trust > 0` and recorded).
    pub used: bool,
}

fn flame_seen(wt: &Weights<'_>, prefix: &PrefixBundle) -> FlameSeen {
    let tau = prefix.cutoff_s;
    let mut t: Vec<f64> = prefix
        .observations
        .iter()
        .filter(|o| {
            matches!(
                o.quantity,
                Quantity::UvFlameDiameter | Quantity::VisibleFlameDiameter
            ) && measurement_valid(o)
                && o.value_si.is_some_and(|d| d > 0.0)
                && o.time_s >= wt.fl_t_min
                && o.time_s <= tau
        })
        .map(|o| o.time_s)
        .collect();
    t.sort_by(f64::total_cmp);
    // the last time with two more positive values within the gap before it: one or two stray
    // frames after the flame are not a flame
    let t_last = (2..t.len())
        .rev()
        .find(|&i| t[i] - t[i - 2] <= wt.fl_gap)
        .map(|i| t[i]);
    let recorded = t.len() as f64 >= wt.fl_min_points && t_last.is_some();
    FlameSeen {
        n: t.len(),
        t_last_s: t_last,
        recorded,
        seen: recorded && t_last.is_some_and(|x| tau - x <= wt.fl_gap),
        used: recorded && wt.fl_trust > 0.0,
    }
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

/// The composition, fibre and igniter conditions a release reads. Nothing is read that its switches
/// do not ask for, except `fiber`, which refuses a supported droplet when `fib.on = 0`. A missing or
/// unusable condition that is needed is a refusal reason, never a default.
fn run_of(wt: &Weights<'_>, prefix: &PrefixBundle, reasons: &mut Vec<String>) -> Run {
    let mut run = Run::NEUTRAL;
    let has = |k: &str| prefix.known_conditions.contains_key(k);
    if wt.mix_on {
        match (has("y_b_mass"), has("y_b_volume")) {
            (true, false) => match common::condition(prefix, "y_b_mass") {
                Ok(y) if (0.0..=1.0).contains(&y) => {
                    run.y_b = y;
                    run.y_basis = Some("mass");
                }
                Ok(y) => reasons.push(format!("y_b_mass = {y} is not in [0, 1]")),
                Err(e) => reasons.push(e),
            },
            (false, true) => match common::condition(prefix, "y_b_volume") {
                Ok(phi) if (0.0..=1.0).contains(&phi) => {
                    run.y_b = wt.rho_b * phi / (wt.rho_a * (1.0 - phi) + wt.rho_b * phi);
                    run.y_basis = Some("volume");
                }
                Ok(phi) => reasons.push(format!("y_b_volume = {phi} is not in [0, 1]")),
                Err(e) => reasons.push(e),
            },
            (true, true) => reasons.push(
                "both y_b_mass and y_b_volume are in the prefix: the composition is ambiguous".into(),
            ),
            (false, false) => reasons.push(
                "mix.on = 1 needs the composition: condition y_b_mass or y_b_volume (a mole fraction cannot be converted: the spec has no molar masses)".into(),
            ),
        }
    }
    if has("fiber") {
        match common::condition(prefix, "fiber") {
            Ok(v) if v == 0.0 || v == 1.0 => run.fibre = Some(v == 1.0),
            Ok(v) => reasons.push(format!("fiber = {v} is neither 0 nor 1")),
            Err(e) => reasons.push(e),
        }
    }
    match (wt.fib_on, run.fibre) {
        (true, Some(true)) => {
            run.rate_offset = Some(wt.fib_rate);
            run.crit_offset = Some(wt.fib_crit);
        }
        (true, Some(false)) => {}
        (true, None) => {
            reasons.push("fib.on = 1 needs the condition fiber (0 or 1) in the prefix".into())
        }
        (false, Some(true)) => reasons
            .push("fibre-supported droplet outside the release population (fib.on = 0)".into()),
        (false, _) => {}
    }
    if let Some((_, e_ref)) = &wt.ig {
        let power = common::condition(prefix, "igniter_power");
        let time = common::condition(prefix, "igniter_time");
        match (power, time) {
            (Ok(pw), Ok(tm)) if pw > 0.0 && tm > 0.0 => run.c_ign = Some((pw * tm / e_ref).ln()),
            (Ok(pw), Ok(tm)) => reasons.push(format!(
                "igniter_power {pw} W and igniter_time {tm} s must be positive"
            )),
            (a, b) => reasons.extend([a.err(), b.err()].into_iter().flatten()),
        }
    }
    run
}

/// Staged burning of one prefix: the nodes of `D_c` and what the cutoff says about them.
#[derive(Debug, Clone, Serialize)]
pub struct Stage {
    /// `D_c` of each node (m).
    pub d_c_m: Vec<f64>,
    /// Weight of each node (normal weights on the grid).
    pub weight: Vec<f64>,
    /// Weight of each node given a hot flame at the cutoff (a node whose droplet is below `D_c`
    /// at the cutoff has lost `p_flash` of its hot flames).
    pub weight_given_hot: Vec<f64>,
    /// The droplet is below `D_c` at the cutoff: stage 2 began before it.
    pub stage2_at_cutoff: Vec<bool>,
    /// Share of the hot flame that the flashes before the cutoff leave.
    pub hot_share: f64,
    pub p_flash: f64,
}

/// The nodes of `D_c = D0 [1 + 3 st.eps (1 - y_b)/y_b]^(-1/3) exp(st.dc_sd u)`, or `None` without
/// staged burning (`st.on = 0`, or `y_b = 0`: no stage 2).
fn stage_of(wt: &Weights<'_>, run: &Run, q0: f64, q_cut: f64, p_flash: f64) -> Option<Stage> {
    if !(wt.st_on && run.y_b > 0.0) {
        return None;
    }
    let n = wt.dc_count;
    let u: Vec<f64> = (0..n)
        .map(|k| (k as f64 - 0.5 * (n - 1) as f64) * wt.dc_step)
        .collect();
    let mut om: Vec<f64> = u.iter().map(|v| (-0.5 * v * v).exp()).collect();
    let total: f64 = om.iter().sum();
    om.iter_mut().for_each(|w| *w /= total);
    let base = q0.sqrt() * (1.0 + 3.0 * wt.st_eps * (1.0 - run.y_b) / run.y_b).powf(-1.0 / 3.0);
    let d_c_m: Vec<f64> = u.iter().map(|v| base * (wt.st_dc_sd * v).exp()).collect();
    let stage2_at_cutoff: Vec<bool> = d_c_m.iter().map(|d| q_cut < d * d).collect();
    let kept: Vec<f64> = om
        .iter()
        .zip(&stage2_at_cutoff)
        .map(|(w, b)| if *b { w * (1.0 - p_flash) } else { *w })
        .collect();
    let hot_share: f64 = kept.iter().sum();
    let weight_given_hot = if hot_share > 0.0 {
        kept.iter().map(|w| w / hot_share).collect()
    } else {
        om.clone()
    };
    Some(Stage {
        d_c_m,
        weight: om,
        weight_given_hot,
        stage2_at_cutoff,
        hot_share,
        p_flash,
    })
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
    let run = run_of(wt, prefix, &mut reasons);
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
    // the last two end windows: what the droplet is doing now
    let fe = wt.end_frac;
    let end_a = window(&obs, tn - 2.0 * fe * span, tn - fe * span);
    let end_b = window(&obs, tn - fe * span, tn);
    let mut scratch = [0.0; H_RATE];
    // D0: head mean advanced to the time origin with the population rate at w = 0 (one pass).
    c[4] = (head.q.sqrt() / wt.d_ref).ln();
    let q0 = head.q + wt.k_hot_pop(&c, 0.0, 0.0, &run, &mut scratch) * head.t;
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
        let k_mod = wt.k_mean(&c, q0, (a.t, a.q), (b.t, b.q), &run, &mut scratch);
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
    let end_used = end_a.n > 0 && end_b.n > 0 && end_b.t > end_a.t;
    let (k_end, _, k_model_end, r_end_noise) = if end_used {
        innovation(&end_a, &end_b)
    } else {
        late
    };
    let level = if end_used { &end_b } else { &tail };
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
    let gain_state = if base > 0.0 {
        sigmoid(
            wt.fgain_b
                + gain_inputs
                    .iter()
                    .zip(wt.fgain_w)
                    .map(|(a, b)| a * b)
                    .sum::<f64>(),
        )
    } else {
        0.0
    };
    let b = gain * (wt.mix[0] * e_late + wt.mix[1] * e_early);
    let s_z = (wt.s_floor * wt.s_floor + (1.0 - gain) * wt.s_pop * wt.s_pop).sqrt();
    // Level at the cutoff: the mean of the last end window carried to the cutoff along the end
    // rate. The end rate is short and a tracker artefact can make it absurd, so it is not believed
    // beyond twice the mid -> tail rate. No weight enters the level the forecast starts from.
    let slope_cap = if k_late.is_finite() {
        (2.0 * k_late.max(wt.k_floor)).min(wt.k_max)
    } else {
        wt.k_max
    };
    let local_slope = if k_end.is_finite() {
        k_end.clamp(0.0, slope_cap)
    } else {
        0.0
    };
    let q_cut = level.q - local_slope * (tau - level.t);
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
    let gas = Gas {
        x_o2: o2,
        x_co2: co2,
        x_he: he,
        pressure_pa: p,
    };
    let (w_cut, s_cut) = (w_of(q_cut), tau * wt.k_ref / q0);
    let laws = wt.laws(
        &c,
        q0.sqrt(),
        &gas,
        &run,
        wt.y_surface(&run, w_cut),
        &mut scratch,
    );
    let sc_g = wt.sc_gas_factor(&gas);
    let sc_hot = wt.sc_rates(sc_g, Last::Hot);
    let surv_d = (-wt.lam_d(&laws, w_cut)).exp();
    let surv_r = wt.surv_r(&laws, s_cut);
    let hot_r = laws.pi_r * surv_r;
    let hot_d = (1.0 - laws.pi_r) * surv_d;
    // staged burning: a droplet below D_c at the cutoff has passed its flash
    let stage = stage_of(wt, &run, q0, q_cut, laws.p_flash);
    let (hot_r, hot_d) = match &stage {
        Some(st) => (hot_r * st.hot_share, hot_d * st.hot_share),
        None => (hot_r, hot_d),
    };
    // a cool flame lit directly by the igniter takes its share from the hot flame
    let p_ig = laws.p_ignite_cool;
    let (hot_r, hot_d) = if wt.ig.is_some() {
        (hot_r * (1.0 - p_ig), hot_d * (1.0 - p_ig))
    } else {
        (hot_r, hot_d)
    };
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
    let p_cool = if wt.ig.is_some() {
        // the direct cool flame started at s = 0 from D0: alive while the droplet is larger than D_ce
        let alive_direct = if laws.sd_ce > 0.0 {
            normal_cdf((0.5 * q_cut.ln() - laws.d_ce_m.ln()) / laws.sd_ce)
        } else {
            f64::from(u8::from(q_cut.sqrt() > laws.d_ce_m))
        };
        (1.0 - p_ig) * p_cool + p_ig * alive_direct
    } else {
        p_cool
    };
    let prior = [p_hot, p_cool, (1.0 - p_hot - p_cool).max(0.0)];
    let mut state = prior;
    let mut b_cool = 0.0;
    if gain_state > 0.0 && k_end.is_finite() && k_model_end.is_finite() && k_model_end > 0.0 {
        let r = k_end / k_model_end;
        let rho_c = laws.k_cool_m2_s / k_model_end;
        // a droplet without a flame, with half of what the flame left: of the post.* transient,
        // or of K0 under the self-cooling law
        let rate_e = if wt.sc_on {
            wt.sc_rate0(q_cut, sc_hot, 0.5)
        } else {
            wt.evap_rate0(q_cut, 0.5)
        };
        let rho_e = rate_e / k_model_end;
        let sd = |model: f64| {
            (model * model + r_end_noise * r_end_noise)
                .sqrt()
                .max(1.0e-9)
        };
        let like = [
            gauss(r, 1.0, sd(wt.sd_hot)) + wt.l_floor,
            gauss(r, rho_c, sd(wt.sd_hot * rho_c.max(1.0e-6))) + wt.l_floor,
            gauss(r, rho_e, sd(wt.sd_evap)) + wt.l_floor,
        ];
        let mut total = 0.0;
        for i in 0..3 {
            state[i] = prior[i] * like[i].powf(gain_state);
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
            b_cool = wt.cool_gain
                * gain_state
                * (k_end.max(wt.k_floor) / laws.k_cool_m2_s)
                    .ln()
                    .clamp(-wt.e_lo, wt.e_hi);
        }
    }
    let class_r_given_hot = if p_hot > 0.0 {
        hot_r / p_hot
    } else {
        laws.pi_r
    };
    // flame cameras: a hot flame is in the traces, a cool flame and an unlit droplet are not
    let flame = flame_seen(wt, prefix);
    let state_before_cameras = state;
    if flame.used {
        let verdict = if flame.seen {
            [1.0, 0.0, 0.0]
        } else {
            let rest = state[1] + state[2];
            if rest > 0.0 {
                [0.0, state[1] / rest, state[2] / rest]
            } else {
                let pc = (laws.p_cool * cool_alive).clamp(0.0, 1.0);
                [0.0, pc, 1.0 - pc]
            }
        };
        for i in 0..3 {
            state[i] = wt.fl_trust * verdict[i] + (1.0 - wt.fl_trust) * state[i];
        }
    }
    // the added laws as they stand for this prefix (reported; the forecast evaluates them itself)
    let finite = |v: f64| v.is_finite().then_some(v);
    let sc_cool = wt.sc_rates(sc_g, Last::Cool);
    let added = Added {
        gas,
        self_cooling: wt.sc_on.then(|| SelfCooling {
            gas_factor: sc_g,
            k0_hot_m2_s: sc_hot[0],
            k0_hot_diffusive_m2_s: wt.sc_rates(sc_g, Last::HotDiffusive)[0],
            k0_cool_m2_s: sc_cool[0],
            t0_hot_s: finite(wt.sc_t0(q_cut, sc_hot[0])),
            t0_cool_s: finite(wt.sc_t0(q_cut, sc_cool[0])),
            k_floor_start_m2_s: sc_hot[2],
            k_floor_end_m2_s: sc_hot[1],
            diff_scale: wt.sc_diff_scale,
        }),
        dying: (wt.dy_frac > 0.0 && laws.k_cool_m2_s > 0.0).then_some(Dying {
            k_die_m2_s: wt.dy_frac * laws.k_cool_m2_s,
            tau_s: wt.dy_tau,
        }),
        cool_end: (wt.ce_on && wt.cool.is_some())
            .then(|| wt.d_ce_form(q0.sqrt(), &gas, wt.y_surface(&run, w_cut))),
        zero_length_cool_flame_counts_as: if wt.ce_on { "hot_flame" } else { "cool_flame" },
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
        k_obs_end_m2_s: k_end,
        k_model_end_m2_s: k_model_end,
        r_end: if k_model_end > 0.0 {
            k_end / k_model_end
        } else {
            f64::NAN
        },
        r_end_noise,
        gain_state,
        t_level_s: level.t,
        gain_inputs,
        gain,
        b,
        b_cool,
        s_z,
        laws,
        prior,
        state,
        class_r_given_hot,
        flame,
        state_before_cameras,
        added,
        run,
        stage,
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
    /// How a droplet without a flame at the cutoff is carried forward.
    pub unlit_at_cutoff: UnlitAtCutoff,
}

/// An extinction-time bin of one hot path: mass (conditional on a hot flame at the cutoff, within
/// the class) and the mass-weighted time and D^2 of the extinction.
#[derive(Clone, Copy, Default)]
struct Bin {
    mass: f64,
    t: f64,
    q: f64,
}

/// Where a hot path reached `D_c` in the forecast.
#[derive(Clone, Copy)]
struct Crossing {
    t: f64,
    q: f64,
    /// Hot mass of the path (within its class mixture) when it reached `D_c`, before the flash.
    alive: f64,
}

/// The stage of a path, as the cool-flame and stage-2 laws read it.
#[derive(Clone, Copy)]
struct StageCtx {
    q_c: f64,
    /// The droplet was below `D_c` at the cutoff.
    from_cutoff: bool,
    /// When the hot flame of this path reached `D_c` in the forecast.
    t_c: Option<f64>,
}

/// RK4 of the hot phase on a uniform grid, then the mixture over states at each forecast time.
fn integrate(wt: &Weights<'_>, enc: &Encoded, times: &[f64], levels: &[f64]) -> Forecast {
    let nt = times.len();
    let nz = wt.z_count;
    let tau = enc.cutoff_s;
    let c = &enc.c;
    let run = &enc.run;
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
    let q_ce_of = |d_ce: f64| -> Vec<f64> {
        zc.iter()
            .map(|u| {
                let d = d_ce * (laws.sd_ce * u).exp();
                d * d
            })
            .collect()
    };
    let q_ce: Vec<f64> = q_ce_of(laws.d_ce_m);
    let w_of = |q: f64| (1.0 - q / q0).clamp(0.0, 1.0);
    let s_of = |t: f64| t * wt.k_ref / q0;
    let surv_d0 = (-wt.lam_d(laws, w_of(enc.q_cutoff_m2))).exp().max(1.0e-300);
    let surv_r0 = wt.surv_r(laws, s_of(tau)).max(1.0e-300);
    // staged burning: one hot path per rate node and node of D_c (one node that never reaches D_c
    // without staging)
    let stage = enc.stage.as_ref();
    let nu = stage.map_or(1, |s| s.d_c_m.len());
    let n_paths = nz * nu;
    let stage_ctx = |u: usize, t_c: Option<f64>| -> Option<StageCtx> {
        stage.map(|s| StageCtx {
            q_c: s.d_c_m[u] * s.d_c_m[u],
            from_cutoff: s.stage2_at_cutoff[u],
            t_c,
        })
    };
    let k2f = wt.st_k2.exp();
    // the dip at D_c in seconds
    let tau_dip = wt.st_dip_clock * q0 / wt.k_ref;
    let factor2 = |t: f64, t_c: Option<f64>| -> f64 {
        match t_c {
            Some(tc) => k2f * (1.0 - wt.st_dip_amp * (-(t - tc).max(0.0) / tau_dip).exp()),
            None => k2f,
        }
    };
    // fall of D^2 in stage 2 from (t2, ...) to t at a stage-1 rate k, the dip counted from t_c
    let fall2 = |t: f64, t2: f64, k: f64, t_c: Option<f64>| -> f64 {
        let dt = (t - t2).max(0.0);
        match t_c {
            Some(tc) => {
                k * k2f
                    * (dt
                        - wt.st_dip_amp
                            * tau_dip
                            * ((-(t2 - tc).max(0.0) / tau_dip).exp()
                                - (-(t - tc).max(0.0) / tau_dip).exp()))
            }
            None => k * k2f * dt,
        }
    };
    // hot paths: q and the conditional survival of both classes on the grid
    let mut q_grid = vec![0.0; n_paths * (n_grid + 1)];
    let mut sd_grid = vec![0.0; n_paths * (n_grid + 1)];
    let mut mult_grid = if stage.is_some() {
        vec![1.0; n_paths * (n_grid + 1)]
    } else {
        Vec::new()
    };
    let mut crossing: Vec<Option<Crossing>> = vec![None; n_paths];
    let mut sr_grid = vec![0.0; n_grid + 1];
    let mut bins_d = vec![Bin::default(); n_paths * n_bins];
    let mut bins_r = vec![Bin::default(); n_paths * n_bins];
    for (i, s) in sr_grid.iter_mut().enumerate() {
        *s = wt.surv_r(laws, s_of(tau + i as f64 * h)) / surv_r0;
    }
    let (cls_r, cls_d) = (enc.class_r_given_hot, 1.0 - enc.class_r_given_hot);
    let mut scratch = [0.0; H_RATE];
    let mut rk4_steps = 0usize;
    for k in 0..nz {
        for u in 0..nu {
            let path = k * nu + u;
            let shift = enc.b + enc.s_z * z[k];
            let (q_c, mut in2) = match stage_ctx(u, None) {
                Some(s) => (s.q_c, s.from_cutoff),
                None => (f64::NEG_INFINITY, false),
            };
            let mut t_c: Option<f64> = None;
            let mut deriv = |q: f64, t: f64, in2: bool, t_c: Option<f64>| -> f64 {
                let x = [c[0], c[1], c[2], c[3], c[4], w_of(q), s_of(t), run.y_b];
                let rate = -wt.k_ref * (wt.ln_rate(&x, run, &mut scratch) + shift).exp();
                if in2 {
                    rate * factor2(t, t_c)
                } else {
                    rate
                }
            };
            let row = path * (n_grid + 1);
            let mut q = enc.q_cutoff_m2;
            q_grid[row] = q;
            sd_grid[row] = 1.0;
            let mut floor = false;
            let mut mult = 1.0;
            for i in 0..n_grid {
                let t = tau + i as f64 * h;
                let q_new = if floor {
                    q
                } else {
                    let a1 = deriv(q, t, in2, t_c);
                    let a2 = deriv(q + 0.5 * h * a1, t + 0.5 * h, in2, t_c);
                    let a3 = deriv(q + 0.5 * h * a2, t + 0.5 * h, in2, t_c);
                    let a4 = deriv(q + h * a3, t + h, in2, t_c);
                    rk4_steps += 1;
                    q + h / 6.0 * (a1 + 2.0 * a2 + 2.0 * a3 + a4)
                };
                // the hot flame reaches D_c in this step: the rest of the step is stage 2
                let mut crossed_at: Option<f64> = None;
                let q_new = if stage.is_some() && !in2 && q_new <= q_c && q >= q_c && q > q_new {
                    let tc = t + h * ((q - q_c) / (q - q_new)).clamp(0.0, 1.0);
                    let hr = t + h - tc;
                    in2 = true;
                    t_c = Some(tc);
                    crossed_at = Some(tc);
                    let a1 = deriv(q_c, tc, true, t_c);
                    let a2 = deriv(q_c + 0.5 * hr * a1, tc + 0.5 * hr, true, t_c);
                    let a3 = deriv(q_c + 0.5 * hr * a2, tc + 0.5 * hr, true, t_c);
                    let a4 = deriv(q_c + hr * a3, tc + hr, true, t_c);
                    q_c + hr / 6.0 * (a1 + 2.0 * a2 + 2.0 * a3 + a4)
                } else {
                    q_new
                };
                let q_new = if q_new <= wt.q_min {
                    floor = true;
                    wt.q_min
                } else {
                    q_new
                };
                let sd_new = (-wt.lam_d(laws, w_of(q_new))).exp() / surv_d0;
                let (t_mid, q_mid) = (t + 0.5 * h, 0.5 * (q + q_new));
                let bi = path * n_bins + i / per_bin;
                let md = (sd_grid[row + i] - sd_new).max(0.0);
                let mr = (sr_grid[i] - sr_grid[i + 1]).max(0.0);
                // after a flash only 1 - p_flash of the path burns on
                let (md, mr) = if stage.is_some() {
                    (md * mult, mr * mult)
                } else {
                    (md, mr)
                };
                bins_d[bi].mass += md;
                bins_d[bi].t += md * t_mid;
                bins_d[bi].q += md * q_mid;
                bins_r[bi].mass += mr;
                bins_r[bi].t += mr * t_mid;
                bins_r[bi].q += mr * q_mid;
                // a flash where the hot flame reached D_c
                if let Some(tc) = crossed_at {
                    let alive =
                        (cls_d * sd_new.min(sd_grid[row + i]) + cls_r * sr_grid[i + 1]) * mult;
                    crossing[path] = Some(Crossing {
                        t: tc,
                        q: q_c.max(wt.q_min),
                        alive,
                    });
                    mult *= 1.0 - laws.p_flash;
                }
                q = q_new;
                q_grid[row + i + 1] = q;
                sd_grid[row + i + 1] = sd_new.min(sd_grid[row + i]);
                if stage.is_some() {
                    mult_grid[row + i + 1] = mult;
                }
            }
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
    let sc_g = wt.sc_gas_factor(&enc.added.gas);
    // q of a droplet that has had no flame since (t_e, q_e). `last` is the flame it had (the post.*
    // law tells a cool flame from a hot one, the self-cooling law also how the hot flame ended);
    // `k_die` is the rate a dying flame starts from, 0 without one.
    let unlit = |t: f64, t_e: f64, q_e: f64, last: Last, k_die: f64| -> f64 {
        let u = (t - t_e).max(0.0);
        let fall = if wt.sc_on {
            wt.sc_fall(u, q_e, wt.sc_rates(sc_g, last))
        } else {
            let scale = if last == Last::Cool { cool_scale } else { 1.0 };
            wt.evap_drop(u, q_e, scale)
        };
        let fall = if k_die > 0.0 {
            fall + wt.die_fall(u, k_die)
        } else {
            fall
        };
        (q_e - fall).max(wt.q_min)
    };
    // q of a droplet whose cool flame started at (t_e, q_e) and burns at k_c until q_x.
    // `after_loss`: it starts where a hot flame was just lost (not a cool flame burning at the
    // cutoff). A cool flame that would end at or above q_e has zero length: under the closed form
    // of the cool-flame end the droplet then comes straight from the hot flame, which dies away;
    // otherwise it counts as coming from a cool flame. With staged burning a cool flame that
    // reaches D_c (or starts below it) burns on at the stage-2 factor; its end is then the root in
    // time of a monotone closed form, found by bisection.
    let cool = |t: f64,
                t_e: f64,
                q_e: f64,
                k_c: f64,
                q_x: f64,
                after_loss: bool,
                st: Option<StageCtx>|
     -> (f64, bool) {
        if q_x >= q_e || !(k_c > 0.0) {
            return if wt.ce_on && after_loss {
                let k_die = wt.dy_frac * k_c.max(0.0);
                (unlit(t, t_e, q_e, Last::Hot, k_die), false)
            } else {
                (unlit(t, t_e, q_e, Last::Cool, 0.0), false)
            };
        }
        let staged = st.and_then(|s| {
            let started_in2 = s.from_cutoff || s.t_c.is_some_and(|tc| tc <= t_e) || q_e <= s.q_c;
            if started_in2 {
                // the dip is counted from the time the hot flame reached D_c, when known
                Some((t_e, q_e, if s.from_cutoff { None } else { s.t_c }))
            } else if q_x < s.q_c {
                let t2 = t_e + (q_e - s.q_c) / k_c;
                Some((t2, s.q_c, Some(t2)))
            } else {
                None
            }
        });
        let Some((t2, q2, t_dip)) = staged else {
            let t_x = t_e + (q_e - q_x) / k_c;
            return if t <= t_x {
                ((q_e - k_c * (t - t_e)).max(wt.q_min), true)
            } else {
                (unlit(t, t_x, q_x, Last::Cool, 0.0), false)
            };
        };
        if t <= t2 {
            return ((q_e - k_c * (t - t_e)).max(wt.q_min), true);
        }
        let q_at = |s: f64| q2 - fall2(s, t2, k_c, t_dip);
        if q_at(t) > q_x {
            return (q_at(t).max(wt.q_min), true);
        }
        // the end of the cool flame: q_at is decreasing in time
        let (mut lo, mut hi) = (t2, t2 + (q2 - q_x) / (k_c * k2f) + wt.st_dip_amp * tau_dip);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if q_at(mid) > q_x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (unlit(t, 0.5 * (lo + hi), q_x, Last::Cool, 0.0), false)
    };
    // the cool-flame laws at the burn progress of a radiative loss, when the surface composition
    // drifts with it: (p_cool, K_c, nodes of q_ce)
    let drifting = wt.mix_on && wt.st_drift != 0.0 && wt.cool.is_some();
    let cool_at = |q_e: f64| -> (f64, f64, Vec<f64>) {
        let mut scratch = [0.0; H_RATE];
        let y_s = wt.y_surface(run, w_of(q_e));
        let l = wt.laws(c, q0.sqrt(), &enc.added.gas, run, y_s, &mut scratch);
        (l.p_cool, l.k_cool_m2_s, q_ce_of(l.d_ce_m))
    };
    // the warm stage: equal-mass quantile nodes of its clock length (Gompertz in s / s_W), in s
    let warm_dt: Vec<f64> = match &wt.warm {
        Some(wm) => {
            let a = wm.sharp;
            (0..wm.count)
                .map(|i| {
                    let p = (i as f64 + 0.5) / wm.count as f64;
                    let lam = -(1.0 - p).ln();
                    let rel = 1.0 + (lam / std::f64::consts::LN_2 + (-a).exp()).ln() / a;
                    (laws.s_warm * rel).max(0.0) * q0 / wt.k_ref
                })
                .collect()
        }
        None => Vec::new(),
    };
    let warm_k = wt.warm.as_ref().map_or(1.0, |wm| wm.k.exp());
    let [p_hot, p_cool, p_evap] = enc.state;
    // evaporation of a droplet already without a flame at the cutoff: the observed end rate is the
    // best evidence of how warm its surroundings still are
    let evap_now_scale = if wt.k_hot > 0.0 && enc.k_obs_end_m2_s.is_finite() {
        let rel = enc.q_cutoff_m2 / (wt.d_ref * wt.d_ref);
        let amp = wt.hot_amp(rel).max(1.0e-12);
        let seen = ((enc.k_obs_end_m2_s - wt.k_inf * rel.powf(wt.size_exp)) / amp).clamp(0.0, 3.0);
        enc.gain_state * seen + (1.0 - enc.gain_state) * 0.5
    } else {
        0.5
    };
    let evap_by_rate = evap_now_scale;
    // a flame the cameras lost at t_x: the heat it left has decayed for cutoff - t_x
    let evap_now_scale = match enc.flame.t_last_s {
        Some(t_x) if enc.flame.used && !enc.flame.seen => {
            let rel = enc.q_cutoff_m2 / (wt.d_ref * wt.d_ref);
            let tau_e = (wt.tau0 * rel.powf(wt.tau_exp)).max(1.0e-3);
            let left = (-(tau - t_x).max(0.0) / tau_e).exp();
            wt.fl_evap_mix * left + (1.0 - wt.fl_evap_mix) * evap_now_scale
        }
        _ => evap_now_scale,
    };
    // The same droplet under the self-cooling law. How long it has been without a flame is not
    // known from the diameter, so the law restarts at the cutoff with the share of K0 its end rate
    // shows (a hyperbola restarted lower is the same hyperbola later: t0 follows the K0 left).
    // When the cameras lost the flame at t_x the law runs from t_x itself, with weight
    // fl.evap_mix. How its flame ended is not known either: it counts as lost to radiation.
    let sc_now = wt.sc_rates(sc_g, Last::Hot);
    let sc_share = if sc_now[0] > 0.0 && enc.k_obs_end_m2_s.is_finite() {
        let seen = ((enc.k_obs_end_m2_s - sc_now[2]) / sc_now[0].max(1.0e-12)).clamp(0.0, 3.0);
        enc.gain_state * seen + (1.0 - enc.gain_state) * 0.5
    } else {
        0.5
    };
    let sc_restart = [sc_share * sc_now[0], sc_now[1], sc_now[2]];
    let lost_for = match enc.flame.t_last_s {
        Some(t_x) if enc.flame.used && !enc.flame.seen => Some((tau - t_x).max(0.0)),
        _ => None,
    };
    let q_cut = enc.q_cutoff_m2;
    // q of a droplet without a flame at the cutoff
    let unlit_now = |t: f64| -> f64 {
        let dt = (t - tau).max(0.0);
        let fall = if wt.sc_on {
            let by_rate = wt.sc_fall(dt, q_cut, sc_restart);
            match lost_for {
                Some(age) => {
                    let since_lost =
                        wt.sc_fall(age + dt, q_cut, sc_now) - wt.sc_fall(age, q_cut, sc_now);
                    wt.fl_evap_mix * since_lost + (1.0 - wt.fl_evap_mix) * by_rate
                }
                None => by_rate,
            }
        } else {
            wt.evap_drop(dt, q_cut, evap_now_scale)
        };
        (q_cut - fall).max(wt.q_min)
    };
    let finite = |v: f64| v.is_finite().then_some(v);
    let unlit_at_cutoff = UnlitAtCutoff {
        law: if wt.sc_on { "self_cooling" } else { "post" },
        origin: "hot_flame",
        share_from_end_rate: if wt.sc_on { sc_share } else { evap_by_rate },
        since_flame_lost_s: lost_for,
        weight_from_flame_lost: if lost_for.is_some() {
            wt.fl_evap_mix
        } else {
            0.0
        },
        k0_m2_s: wt.sc_on.then_some(sc_restart[0]),
        t0_s: wt
            .sc_on
            .then(|| finite(wt.sc_t0(q_cut, sc_restart[0])))
            .flatten(),
    };
    let nl = levels.len();
    let mut quantiles = vec![vec![0.0; nt]; nl];
    let mut latent = vec![vec![0.0; nt]; nl];
    let mut mean = vec![0.0; nt];
    let mut events: [Vec<f64>; 4] = std::array::from_fn(|_| vec![0.0; nt]);
    let mut state_probability: [Vec<f64>; 3] = std::array::from_fn(|_| vec![0.0; nt]);
    // share of the hot state in stage 2 at each forecast time (staged burning only)
    let mut hot2_share = vec![0.0; nt];
    let mut atoms: Vec<(f64, f64)> = Vec::new();
    let mut qe = vec![0.0; nl];
    let mut grid: Vec<(f64, f64)> = Vec::new();
    let mut worst = 0.0_f64;
    let mut max_atoms = 0usize;
    const TINY: f64 = 1.0e-12;
    for (j, &tj) in times.iter().enumerate() {
        atoms.clear();
        let (mut hot, mut cool_p, mut hot2) = (0.0, 0.0, 0.0);
        let pos = ((tj - tau) / h).clamp(0.0, n_grid as f64);
        let i0 = (pos.floor() as usize).min(n_grid.saturating_sub(1));
        let frac = pos - i0 as f64;
        let lerp = |g: &[f64], row: usize| g[row + i0] + frac * (g[row + i0 + 1] - g[row + i0]);
        let sr_now = sr_grid[i0] + frac * (sr_grid[i0 + 1] - sr_grid[i0]);
        let full_bins = i0 / per_bin; // bins that ended at or before grid node i0
        let part_start = full_bins * per_bin;
        if p_hot > TINY {
            for k in 0..nz {
                for u in 0..nu {
                    let path = k * nu + u;
                    let row = path * (n_grid + 1);
                    let q_now = lerp(&q_grid, row);
                    let sd_now = lerp(&sd_grid, row);
                    let rate_factor = (enc.b + enc.s_z * z[k]).exp();
                    let w_k = match stage {
                        Some(s) => p_hot * om[k] * s.weight_given_hot[u],
                        None => p_hot * om[k],
                    };
                    let mult_now = if stage.is_some() {
                        lerp(&mult_grid, row)
                    } else {
                        1.0
                    };
                    let cross = crossing[path];
                    let t_c = cross.map(|x| x.t);
                    let st = stage_ctx(u, t_c);
                    // still inside the hot flame
                    let alive = cls_d * sd_now + cls_r * sr_now;
                    let alive = if stage.is_some() {
                        alive * mult_now
                    } else {
                        alive
                    };
                    if w_k * alive > TINY {
                        atoms.push((q_now.sqrt(), w_k * alive));
                        hot += w_k * alive;
                        if st.is_some_and(|s| s.from_cutoff || t_c.is_some_and(|tc| tc <= tj)) {
                            hot2 += w_k * alive;
                        }
                    }
                    // a flash at D_c: the droplet goes on as after a diffusive extinction
                    if let Some(x) = cross.filter(|x| x.t <= tj) {
                        let m = w_k * laws.p_flash * x.alive;
                        if m > TINY {
                            atoms.push((unlit(tj, x.t, x.q, Last::HotDiffusive, 0.0).sqrt(), m));
                        }
                    }
                    let emit = |b: Bin,
                                radiative: bool,
                                atoms: &mut Vec<(f64, f64)>,
                                cool_p: &mut f64| {
                        let m = w_k * b.mass * if radiative { cls_r } else { cls_d };
                        if !(m > TINY) {
                            return;
                        }
                        let drift = (radiative && drifting).then(|| cool_at(b.q));
                        let (pc_law, k_law, q_ce_here) = match &drift {
                            Some((p, k, nodes)) => (*p, *k, nodes.as_slice()),
                            None => (laws.p_cool, laws.k_cool_m2_s, q_ce.as_slice()),
                        };
                        let pc = if radiative { pc_law } else { 0.0 };
                        let k_c = k_law * rate_factor;
                        // a warm flame first, with probability p_warm (radiative losses only)
                        let pw = if radiative && wt.warm.is_some() {
                            laws.p_warm
                        } else {
                            0.0
                        };
                        let m_all = m;
                        let m = if pw > 0.0 { (1.0 - pw) * m_all } else { m_all };
                        if (1.0 - pc) * m > TINY {
                            // a flame lost to radiation that no cool flame follows dies away; a
                            // flame that ended diffusively leaves a less volatile droplet
                            let (last, k_die) = if radiative {
                                (Last::Hot, wt.dy_frac * k_c)
                            } else {
                                (Last::HotDiffusive, 0.0)
                            };
                            atoms.push((unlit(tj, b.t, b.q, last, k_die).sqrt(), (1.0 - pc) * m));
                        }
                        if pc * m > TINY {
                            for (v, &qx) in q_ce_here.iter().enumerate() {
                                let (q, burning) = cool(tj, b.t, b.q, k_c, qx, true, st);
                                atoms.push((q.sqrt(), pc * m * omc[v]));
                                if burning {
                                    *cool_p += pc * m * omc[v];
                                }
                            }
                        }
                        if pw > 0.0 {
                            // the warm flame, then what follows a radiative loss, without a dying flame
                            let mw = pw * m_all / warm_dt.len() as f64;
                            let k_w = k_c * warm_k;
                            for &dtw in &warm_dt {
                                if !(mw > TINY) {
                                    break;
                                }
                                let t_w = b.t + dtw;
                                let q_w = (b.q - k_w * dtw).max(wt.q_min);
                                if tj <= t_w {
                                    atoms.push(((b.q - k_w * (tj - b.t)).max(wt.q_min).sqrt(), mw));
                                    *cool_p += mw;
                                    continue;
                                }
                                if (1.0 - pc) * mw > TINY {
                                    atoms.push((
                                        unlit(tj, t_w, q_w, Last::Cool, 0.0).sqrt(),
                                        (1.0 - pc) * mw,
                                    ));
                                }
                                if pc * mw > TINY {
                                    for (v, &qx) in q_ce_here.iter().enumerate() {
                                        let (q, burning) = cool(tj, t_w, q_w, k_c, qx, false, st);
                                        atoms.push((q.sqrt(), pc * mw * omc[v]));
                                        if burning {
                                            *cool_p += pc * mw * omc[v];
                                        }
                                    }
                                }
                            }
                        }
                    };
                    for bi in 0..full_bins.min(n_bins) {
                        emit(bins_d[path * n_bins + bi], false, &mut atoms, &mut cool_p);
                        emit(bins_r[path * n_bins + bi], true, &mut atoms, &mut cool_p);
                    }
                    // the bin in progress: extinction between its start and now
                    if full_bins < n_bins && pos > part_start as f64 {
                        let (q_s, t_s) = (q_grid[row + part_start], tau + part_start as f64 * h);
                        let mid = Bin {
                            mass: 0.0,
                            t: 0.5 * (t_s + tj),
                            q: 0.5 * (q_s + q_now),
                        };
                        let (md, mr) = (
                            (sd_grid[row + part_start] - sd_now).max(0.0),
                            (sr_grid[part_start] - sr_now).max(0.0),
                        );
                        let (md, mr) = if stage.is_some() {
                            (md * mult_now, mr * mult_now)
                        } else {
                            (md, mr)
                        };
                        emit(Bin { mass: md, ..mid }, false, &mut atoms, &mut cool_p);
                        emit(Bin { mass: mr, ..mid }, true, &mut atoms, &mut cool_p);
                    }
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
                for u in 0..nu {
                    let st = stage_ctx(u, None);
                    let wu = stage.map_or(1.0, |s| s.weight[u]);
                    if ok > 0.0 {
                        for (v, &qx) in q_ce.iter().enumerate() {
                            if qx < enc.q_cutoff_m2 {
                                let (q, burning) =
                                    cool(tj, tau, enc.q_cutoff_m2, k_c, qx, false, st);
                                let m = if stage.is_some() {
                                    p_cool * om[k] * omc[v] / ok * wu
                                } else {
                                    p_cool * om[k] * omc[v] / ok
                                };
                                atoms.push((q.sqrt(), m));
                                if burning {
                                    cool_p += m;
                                }
                            }
                        }
                    } else {
                        let m = if stage.is_some() {
                            p_cool * om[k] * wu
                        } else {
                            p_cool * om[k]
                        };
                        atoms.push((unlit(tj, tau, enc.q_cutoff_m2, Last::Cool, 0.0).sqrt(), m));
                    }
                }
            }
        }
        if p_evap > TINY {
            atoms.push((unlit_now(tj).sqrt(), p_evap));
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
        hot2_share[j] = if hot > 0.0 { hot2 / hot } else { 0.0 };
    }
    // Events of a droplet burning hot at the cutoff, as competing risks: the hot flame ends
    // (extinction of either kind), or the droplet disrupts while still burning hot. With H the
    // probability of a hot flame ignoring disruption, active = H exp(-lam (t - cutoff)), the
    // disruption incidence is the integral of lam * active, and the rest is the end of the hot
    // phase. The three rows sum to one by construction; a droplet already out at the cutoff counts
    // as ended. With staged burning the hazard at each time is the mean of the two stages' hazards
    // over the hot state, and its integral is taken by the trapezoid rule on the forecast times.
    let mut cum_dis = 0.0;
    let mut prev: Option<(f64, f64, f64)> = None;
    let mut ended = 0.0_f64;
    let mut big_lam = 0.0;
    for (j, &tj) in times.iter().enumerate() {
        let lam_j = if stage.is_some() {
            wt.lam_dis * (1.0 - hot2_share[j]) + wt.lam_dis2 * hot2_share[j]
        } else {
            wt.lam_dis
        };
        let active = if stage.is_some() {
            big_lam += match prev {
                Some((tp, _, lp)) => 0.5 * (lp + lam_j) * (tj - tp),
                None => lam_j * (tj - tau),
            };
            state_probability[0][j] * (-big_lam).exp()
        } else {
            state_probability[0][j] * (-wt.lam_dis * (tj - tau)).exp()
        };
        if let Some((tp, ap, lp)) = prev {
            cum_dis += if stage.is_some() {
                0.5 * (lp * ap + lam_j * active) * (tj - tp)
            } else {
                0.5 * wt.lam_dis * (ap + active) * (tj - tp)
            };
        } else {
            cum_dis = state_probability[0][j] - active;
        }
        prev = Some((tj, active, lam_j));
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
        unlit_at_cutoff,
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
        if !fuel.accepts(&raw) {
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
        "unlit_at_cutoff": f.unlit_at_cutoff,
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
            // The fitted FLEX experts' policy and the conditions sections 3.2 and 3.3 read: the
            // composition, the fibre and the igniter. The prefix hash depends on the allowed set,
            // so cases of these models do not pair with the baselines'.
            policy: ProspectivePolicy::new(
                Family::Flex,
                &[
                    "x_o2",
                    "x_co2",
                    "x_he",
                    "pressure",
                    "fuel",
                    "y_b_mass",
                    "y_b_volume",
                    "fiber",
                    "igniter_power",
                    "igniter_time",
                ],
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

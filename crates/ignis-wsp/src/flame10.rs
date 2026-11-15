//! `acme_normal.wsp16` — the hinge-law clocks of the ACME burner flames with the burner-heating law
//! (the laws of `flame4.rs`) reading the second fuel table and the diluent table of
//! [`crate::fuel2`], with the heating law of `docs/wsp/research/phys_sflame_f3.md` (E3): the hinge
//! members with the heating law of the committee `acme_normal.wsp13` (slots c0 to c3).
//!
//! The laws and the vertex weighing of `flame7.rs`, unchanged, with the hinge lines of E3 in the
//! hinge part and the burner lines in the burner part:
//!
//! ```text
//! k the fuel's row or blend, d the diluent's row
//! T_1        = tad when dil.le_o2[d] = 1, else T_0 + (tad - T_0) (1 - (1 - zst)^(1 / le)) / zst
//! refuse     T_1 < fuel.tad_min_k
//! c_tad      = (T_1 + fuel.dtad_k - norm.tad_ref_k) / norm.tad_scale_k   entry 4 of c (se.flash; the bound)
//! ln tau_R   = N*_se(se.tau ; c with c_logm + fuel.s + dil.dlogm[d]) + dil.dlnr[d]
//!              (normal flames; the inverse laws do not read fuel.s)
//! burner     Y_R with dil.mw_g_mol[d]; r_ss with rho_d x dil.rho_d[d] (a factor of exactly 1 is not
//!            applied); c_tad_burn = (T_1 - norm.tad_ref_k) / norm.tad_scale_k is entry 2 of b;
//! ln dT_eq   = N*_ft(bt.rise ; b) + bt.kappa_exp ln_1p(kappa)
//!              + ln((T_1 - T_b) / (bt.tflame_ref_k - T_b)), T_b = bt.t0_c + 273.15
//!              (the last term omitted when bt.tflame_ref_k = 0; T_1 <= T_b is a refusal)
//! kappa      = sum over the species i of the burner stream of x_i (k_rel,i - 1)
//! limits     with burner_limit_c: bt.limit_c[l] + (burner_limit_c - bt.limit_script_c)
//! ```
//!
//! Everything after the clocks is that of `flame7.rs`. With nitrogen, inert new columns, a stream of
//! unit relative conductivity, `bt.tflame_ref_k` 0 and no `burner_limit_c` (or the member's own
//! setting) the forecast is that of `flame7.rs` bit for bit.
//!
//! `flame7.rs` is not edited or called: this file repeats the lines it shares with it. Its spec takes
//! the tensors of the `flame4.rs` spec without `phys.mw_ethane`, `phys.mw_propane`, `phys.mw_butane`
//! and `phys.mw_n2`, adds the two tables, the three member scalars of the heating law and the
//! constant `phys.t_stream_k`. Normal flames only. Nothing is fitted on this route.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    AnalysisArtifact, ArtifactKind, ControlStage, Family, Finite, ForwardContract, ForwardProfile,
    ModelRelease, ModelSpec, PrefixBundle, QualificationState, SpecId, Support, SupportGate,
    TensorSpec,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::LoadedRelease;
use ignis_core::resources::Profile;
use ignis_core::views::View;

use crate::common::{self, admitted, refuse, Body};
use crate::fuel::{self, Evaluated};
use crate::fuel2::{self, Inputs};
use crate::nn::{hinge_net, sigmoid};
use crate::quad::normal_quantile;
use crate::spec::Params;

pub const FORWARD_VERSION: &str = "16";
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
/// Positions of the two indicator entries in `c` and in `b`.
pub const I_ETHANE: usize = 6;
pub const I_BUTANE: usize = 7;
pub const IB_ETHANE: usize = 5;
pub const IB_BUTANE: usize = 6;
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
pub const ALLOWED: [&str; 12] = [
    "pressure",
    "injected_mass_flow",
    "x_o2_role",
    "x_fuel_role",
    "tad",
    "zst",
    "fuel_is_ethane",
    "fuel_is_butane",
    "fuel_species",
    fuel2::DILUENT_SPECIES,
    fuel2::FUEL_BLEND_X,
    fuel2::BURNER_LIMIT,
];
pub const INTERPRETATION: &str = "observational event prediction under the observed experimental \
    policy; operator termination is not spontaneous extinction; extinction is the end of the hot flame";

pub fn model_name(family: Family) -> &'static str {
    match family {
        Family::AcmeInverse => "acme_inverse.wsp16",
        _ => "acme_normal.wsp16",
    }
}

pub fn family_of_model(name: &str) -> Option<Family> {
    [Family::AcmeNormal, Family::AcmeInverse]
        .into_iter()
        .find(|f| model_name(*f) == name)
}

/// A tensor of this spec that the burner-limit clock reads (`flame4::is_burner_tensor`, without the
/// alkane and nitrogen constants, with the member scalars of the heating law).
pub fn is_burner_tensor(name: &str) -> bool {
    (crate::flame4::is_burner_tensor(name)
        && !fuel::is_alkane_molar_mass(name)
        && name != "phys.mw_n2")
        || fuel2::MEMBER_SCALARS.contains(&name)
}

/// The frozen specification of one configuration: the tensors of the `flame4.rs` spec without the
/// three alkane molar masses, each as it stands there (name, shape, role, bounds, fixed value),
/// then the fuel table.
pub fn spec(family: Family) -> ModelSpec {
    assert!(
        family == Family::AcmeNormal,
        "the design is written for normal flames"
    );
    // built once per process and family: it is made of another spec and its hash
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
    let parent = crate::flame4::spec(family);
    let mut tensors: Vec<TensorSpec> = parent
        .tensors
        .iter()
        .filter(|t| !fuel::is_alkane_molar_mass(&t.name) && t.name != "phys.mw_n2")
        .cloned()
        .collect();
    tensors.extend(fuel2::tensors());
    tensors.extend(fuel2::member_scalars());
    tensors.push(fuel2::t_stream());
    let spec_id = if family == Family::AcmeInverse {
        "acme_inverse-wsp-v16"
    } else {
        "acme_normal-wsp-v16"
    };
    let mut features = parent.features.clone();
    for f in features.iter_mut() {
        if f.name == "f_ethane" || f.name == "f_butane" {
            f.source = "condition:fuel_species|fuel_is_ethane|fuel_is_butane".into();
            f.transform = "vertex of the fuel-table weighing: each law network is evaluated at \
                           (f_ethane, f_butane) = (1, 0), (0, 0), (0, 1) and its outputs are weighed \
                           with the fuel's coordinates; the programmed-stop logits read \
                           fuel.ft_coord of the fuel's row @1"
                .into();
        }
        if f.name == "b_mtot" {
            f.transform = "ln(m_tot / norm.mtot_ref_kg_s), m_tot = injected_mass_flow / Y_R, Y_R \
                           from the molar mass fuel.mw_g_mol of the fuel's row (normal flames) or \
                           phys.mw_o2 (inverse flames); burner feature 1 @1"
                .into();
        }
    }
    ModelSpec {
        schema_version: parent.schema_version,
        spec_id: SpecId::new(spec_id).expect("id"),
        family,
        model_name: model_name(family).into(),
        target: parent.target.clone(),
        features,
        domain: parent.domain.clone(),
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::flame10::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(format!(
            "The second fuel table, the diluent table and the heating law of \
             docs/wsp/research/phys_sflame_f3.md E3 on the laws below; the tables: {} The \
             first-table rules still in force: {} New lines: Y_R with dil.mw_g_mol in place of \
             phys.mw_n2 (which this spec does not hold); T_1 as in the tables' notes, refusal when T_1 \
             < fuel.tad_min_k; c_tad = (T_1 + fuel.dtad_k - norm.tad_ref_k) / norm.tad_scale_k is \
             entry 4 of c, read by every law network and by the domain bound; c_tad_burn = (T_1 - \
             norm.tad_ref_k) / norm.tad_scale_k is entry 2 of b; rho_d = se.rho_d_g_m_s x dil.rho_d \
             wherever r_ss is formed (a factor of exactly 1 is not applied); {} ln dT_eq = \
             N*_ft(bt.rise ; b) + bt.kappa_exp ln_1p(kappa) + ln((T_1 - T_b) / (bt.tflame_ref_k - \
             T_b)), T_b = bt.t0_c + 273.15, the last term omitted when bt.tflame_ref_k = 0 and T_1 <= \
             T_b a refusal; kappa = sum over the species of the burner stream of x_i (k_rel,i - 1), \
             the fuel species with x_fuel_role split by the fuel's fractions and fuel.k_rel, the \
             diluent with 1 - x_fuel_role and dil.k_rel; with the condition burner_limit_c every \
             weighted limit is bt.limit_c[l] + (burner_limit_c - bt.limit_script_c) and the refusal \
             of a weighted limit at or below bt.t0_c applies to the shifted value. The laws: The hinge-law clocks with the burner-heating law of spec {} ({}) with a typed fuel table \
             in place of the two fuel indicators. The tensors are those of that spec without \
             phys.mw_ethane, phys.mw_propane and phys.mw_butane, each with the name, shape, role, \
             bounds and fixed value it has there, then {}. \
             Hinge part, with (e, b) = fuel.se_coord[k, :]: ln tau_R = N*_se(se.tau ; c with c_logm \
             + fuel.s[k]) for normal flames, N*_se(se.tau ; c) for inverse flames; logit p_weak = \
             N*_se(se.flash ; c); programmed stop a: sigmoid(ft.atom_logit[a] + ft.atom_w[a, :] . c) \
             with the indicator entries of c set to fuel.ft_coord[k, :]. Burner part: molar mass of \
             the burner-side reactant fuel.mw_g_mol[k] for normal flames, phys.mw_o2 for inverse \
             flames; Y_R, m_tot, L, r_ss, b_rss and b_mtot are then as in that spec; ln dT_eq = \
             N*_ft(bt.rise ; b) with (e, b) = fuel.ft_coord[k, :], the vertex set on f_ethane, \
             f_butane of b. fuel.dosc is not read (no quench clock). The domain bounds are checked \
             on the unshifted c. Everything after the clocks is as in the notes of that spec. \
             Uncertainty kind: assumed_noise.",
            fuel2::NOTES,
            fuel::NOTES,
            "ln tau_R = N*_se(se.tau ; c with c_logm + fuel.s + dil.dlogm) + dil.dlnr (fuel.s not read for inverse flames);",
            parent.spec_id,
            parent.spec_hash(),
            fuel2::NOTES,
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
    fuel: fuel2::Table<'a>,
    t0: f64,
    kappa_exp: f64,
    tflame_ref: f64,
    limit_script: f64,
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
    /// The fuel, the diluent, the flame temperature and the script's limit that were read.
    pub inputs: Inputs,
    /// The shift added to `c_logm` in the radiative clock (`fuel.s` + `dil.dlogm`; `fuel.s` is not
    /// read for inverse flames).
    pub radiative_logm_shift: f64,
    /// The shift added to ln tau_R (`dil.dlnr`).
    pub radiative_ln_shift: f64,
    /// The vertices each law network was evaluated at, with their weights.
    pub laws: Vec<Evaluated>,
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

impl Weights<'_> {
    /// rho D at the flame: `se.rho_d_g_m_s`, times the diluent's factor unless it is exactly 1.
    #[inline]
    fn rho_d_flame(&self, dl: &fuel2::Diluent) -> f64 {
        if dl.rho_d == 1.0 {
            self.rho_d
        } else {
            self.rho_d * dl.rho_d
        }
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
            fuel: fuel2::Table::bind(p),
            t0: p.s(fuel2::T_STREAM),
            kappa_exp: p.s("bt.kappa_exp"),
            tflame_ref: p.s("bt.tflame_ref_k"),
            limit_script: p.s("bt.limit_script_c"),
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

    /// The limit settings: `bt.limit_c`, moved by `burner_limit_c - bt.limit_script_c` when the
    /// prefix carries the script's setting.
    fn limits(&self, script: Option<f64>) -> [f64; N_LIMITS] {
        std::array::from_fn(|l| match script {
            Some(x) => self.bt_limit[l] + (x - self.limit_script),
            None => self.bt_limit[l],
        })
    }

    /// Weighted limits at or below the burner temperature at ignition (a release that cannot be run
    /// on this prefix).
    fn limits_below_t0(&self, script: Option<f64>) -> Vec<String> {
        let lim = self.limits(script);
        (0..N_LIMITS)
            .filter(|&l| self.bt_limit_w[l] > 0.0 && lim[l] <= self.bt_t0)
            .map(|l| {
                format!(
                    "the weighted limit {l} = {} degC (bt.limit_c[{l}] = {}, script setting {script:?}, \
                     bt.limit_script_c = {}) is at or below bt.t0_c = {} degC",
                    lim[l], self.bt_limit[l], self.limit_script, self.bt_t0
                )
            })
            .collect()
    }

    /// The clocks of the conditions `c` (indicator entries ignored: each law reads its vertices)
    /// and burner-side quantities `b` for the fuel `f`.
    pub(crate) fn clocks(&self, c: &[f64; N_IN], b: &Burner, inp: &Inputs) -> Clocks {
        let (f, dl) = (&inp.fuel, &inp.diluent);
        let mut scratch = [0.0; HIDDEN];
        let mut scratch_b = [0.0; HIDDEN_B];
        let mut laws = Vec::with_capacity(3);
        // hinge part: the oxygen-demand shift (normal flames only) and the diluent's flow shift enter
        // the radiative clock only
        let s_fuel = if self.inverse { 0.0 } else { f.s };
        let mut c_tau = *c;
        c_tau[1] = c[1] + s_fuel + dl.dlogm;
        let (ln_tau_r, e) = fuel::weighed("se.tau", &c_tau, I_ETHANE, I_BUTANE, f.se_coord, |x| {
            self.se_tau.eval(x, &mut scratch)
        });
        let ln_tau_r = ln_tau_r + dl.dlnr;
        laws.push(e);
        let (logit_weak, e) = fuel::weighed("se.flash", c, I_ETHANE, I_BUTANE, f.se_coord, |x| {
            self.se_flash.eval(x, &mut scratch)
        });
        laws.push(e);
        let four_pi = 4.0 * std::f64::consts::PI;
        // SI: kg m-1 s-1
        let rho_d = self.rho_d_flame(dl) * 1.0e-3;
        let r_ss = b.m_tot_kg_s / (four_pi * rho_d * b.l);
        // burner side
        let bf = [
            (r_ss * 1.0e3 / self.r_ref_mm).ln(),
            (b.m_tot_kg_s / self.mtot_ref).ln(),
            (inp.t1_k - self.tad_ref) / self.tad_scale,
            c[3],
            c[0],
            c[6],
            c[7],
        ];
        let (ln_rise, e) = fuel::weighed("bt.rise", &bf, IB_ETHANE, IB_BUTANE, f.ft_coord, |x| {
            self.bt_rise.eval(x, &mut scratch_b)
        });
        laws.push(e);
        // conduction through the burner stream: its conductivity to a power, and the temperature
        // difference between flame and burner (omitted when bt.tflame_ref_k is 0)
        let kappa = fuel2::kappa(f, dl, inp.x_fuel);
        let ln_rise = ln_rise + self.kappa_exp * kappa.ln_1p();
        let t_b = self.bt_t0 + 273.15;
        let ln_rise = if self.tflame_ref == 0.0 {
            ln_rise
        } else {
            ln_rise + ((inp.t1_k - t_b) / (self.tflame_ref - t_b)).ln()
        };
        let bf = fuel::with_coord(&bf, IB_ETHANE, IB_BUTANE, f.ft_coord);
        let bt_limit = self.limits(inp.burner_limit_c);
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
                let need = bt_limit[l] - self.bt_t0;
                burner_limits.push((bt_limit[l], w, limit_time(need, ln_rise, self.bt_tau)));
                for j in 0..nb {
                    let ln_dt = ln_rise + self.bt_sigma * node_z(j, nb);
                    match limit_time(need, ln_dt, self.bt_tau) {
                        Some(t) => pairs.push((w / nb as f64, t)),
                        None => never += w / nb as f64,
                    }
                }
            }
        }
        // the programmed-stop logits are linear in c: they read the coordinates themselves
        let c_ft = fuel::with_coord(c, I_ETHANE, I_BUTANE, f.ft_coord);
        let mut programmed = Vec::new();
        for a in 0..N_ATOMS {
            let row = &self.atom_w[a * N_IN..(a + 1) * N_IN];
            let logit = self.atom_logit[a] + row.iter().zip(&c_ft).map(|(w, x)| w * x).sum::<f64>();
            let p = sigmoid(logit);
            if p > 1.0e-9 && self.atom_t[a] > 0.0 {
                programmed.push((self.atom_t[a], p.min(1.0 - 1.0e-9)));
            }
        }
        Clocks {
            p_weak: sigmoid(logit_weak),
            weak_tau_s: self.flash_tau,
            floor_s: self.floor,
            radiative_tau_s: ln_tau_r.exp(),
            r_ss_mm: r_ss * 1.0e3,
            burner_features: bf,
            burner_rise_k: ln_rise.exp(),
            burner_heat_tau_s: self.bt_tau,
            burner_t0_c: self.bt_t0,
            burner_limits,
            p_limit_never: never,
            programmed,
            background_per_s: self.base,
            inputs: inp.clone(),
            radiative_logm_shift: s_fuel + dl.dlogm,
            radiative_ln_shift: dl.dlnr,
            laws,
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
    /// (Indicator entries of `c`: the fuel's `fuel.ft_coord`, which the programmed-stop logits read.)
    pub(crate) fn features(
        &self,
        prefix: &PrefixBundle,
    ) -> Result<([f64; N_IN], Burner, Inputs), Vec<String>> {
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
        // the row of the fuel table: the species, else the stored indicators; an unknown fuel is a
        // refusal, not "propane"
        let inp = match self.fuel.inputs(prefix, tad, zst, xf, self.t0) {
            Ok(i) => Some(i),
            Err(r) => {
                reasons.extend(r);
                None
            }
        };
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
        let mut inp = match inp {
            Some(i) if reasons.is_empty() => i,
            _ => {
                reasons.dedup();
                return Err(reasons);
            }
        };
        // T_1 at or below the burner's own temperature leaves the heating law with no clock
        let t_b = self.bt_t0 + 273.15;
        if self.tflame_ref != 0.0 && !(inp.t1_k > t_b) {
            return Err(vec![format!(
                "the flame temperature T_1 = {} K is not above the burner temperature at ignition {t_b} K: the heating law has no clock",
                inp.t1_k
            )]);
        }
        inp.zst = zst;
        inp.x_fuel = xf;
        let mw_r = if self.inverse {
            self.mw_o2
        } else {
            inp.fuel.mw_g_mol
        };
        let mw_dil = inp.diluent.mw_g_mol;
        let y_r = x_r * mw_r / (x_r * mw_r + (1.0 - x_r) * mw_dil);
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
                (inp.t1_k + inp.fuel.dtad_k - self.tad_ref) / self.tad_scale,
                (zst - self.zst_ref) / self.zst_scale,
                inp.fuel.ft_coord[0],
                inp.fuel.ft_coord[1],
            ],
            burner,
            inp,
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
    let below = wt.limits_below_t0(fuel2::burner_limit(prefix).ok().flatten());
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
    let (c, burner, inp) = match wt.features(prefix) {
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
    let clocks = wt.clocks(&c, &burner, &inp);
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

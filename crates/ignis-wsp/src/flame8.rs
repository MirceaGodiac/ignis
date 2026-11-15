//! `acme_normal.wsp14` — the clock forward of the ACME burner flames (the hinge laws of `flame.rs`)
//! reading the second fuel table and the diluent table of [`crate::fuel2`]: the hinge members of the
//! committee `acme_normal.wsp13` (slots a0, a1).
//!
//! The laws and the vertex weighing of `flame5.rs`, unchanged, with the hinge-law lines of section
//! E3 of `docs/wsp/research/phys_sflame_f3.md`:
//!
//! ```text
//! k the fuel's row or blend, d the diluent's row
//! T_1        = tad                                                     dil.le_o2[d] = 1
//!            = T_0 + (tad - T_0) (1 - (1 - zst)^(1 / dil.le_o2[d])) / zst   otherwise, T_0 = phys.t_stream_k
//! refuse     T_1 < fuel.tad_min_k
//! c_tad      = (T_1 + fuel.dtad_k - norm.tad_ref_k) / norm.tad_scale_k     entry 4 of c (se.flash reads it;
//!                                                                          so does the domain bound)
//! ln tau_R   = N*_se(se.tau ; c with c_logm + fuel.s + dil.dlogm[d]) + dil.dlnr[d]
//!              (normal flames; the inverse laws do not read fuel.s)
//! logit weak = N*_se(se.flash ; c)
//! ln tau_B   = N*_ft(ft.tau ; c)
//! programmed sigmoid(logit_a + W_a . c), the indicator entries of c = fuel.ft_coord
//! ```
//!
//! Everything after the clocks is that of `flame5.rs`. With nitrogen (le 1, every shift 0) and a fuel
//! whose new columns are inert (`fuel.dtad_k` 0) the forecast is that of `flame5.rs` with the same
//! seven columns bit for bit: `tad + 0.0`, `+ 0.0` are identities. This law reads none of
//! `fuel.sigma` (except for a blend's shares), `fuel.k_rel`, `fuel.zst0`, `fuel.gz`, `fuel.mw_g_mol`
//! (except for a blend's shares), `fuel.dosc`, `dil.mw_g_mol`, `dil.rho_d`, `dil.d_far`, `dil.dflux`,
//! `dil.k_rel`; its spec holds them so that every member of the committee carries the same tables.
//!
//! `flame5.rs` is not edited or called: this file repeats the lines it shares with it. Its spec takes
//! the tensors of the `flame.rs` spec, name for name, adds the two tables and the constant
//! `phys.t_stream_k`. Nothing is fitted on this route.

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
use crate::spec::Params;

pub const FORWARD_VERSION: &str = "14";
/// Hidden width of each law network.
pub const HIDDEN: usize = 8;
/// Condition features.
pub const N_IN: usize = 8;
/// Programmed-stop atoms.
pub const N_ATOMS: usize = 8;
/// Positions of the two indicator entries in `c`.
pub const I_ETHANE: usize = 6;
pub const I_BUTANE: usize = 7;
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
        Family::AcmeInverse => "acme_inverse.wsp14",
        _ => "acme_normal.wsp14",
    }
}

pub fn family_of_model(name: &str) -> Option<Family> {
    [Family::AcmeNormal, Family::AcmeInverse]
        .into_iter()
        .find(|f| model_name(*f) == name)
}

/// The frozen specification of one configuration: the tensors of the `flame.rs` spec, each as it
/// stands there (name, shape, role, bounds, fixed value), then the two tables and the constant of
/// the flame temperature.
pub fn spec(family: Family) -> ModelSpec {
    assert!(matches!(family, Family::AcmeNormal | Family::AcmeInverse));
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
    let hinge = crate::flame::spec(family);
    let mut tensors: Vec<TensorSpec> = hinge.tensors.clone();
    tensors.extend(fuel2::tensors());
    tensors.push(fuel2::t_stream());
    let spec_id = if family == Family::AcmeInverse {
        "acme_inverse-wsp-v14"
    } else {
        "acme_normal-wsp-v14"
    };
    let mut features = hinge.features.clone();
    for f in features.iter_mut() {
        if f.name == "f_ethane" || f.name == "f_butane" {
            f.source = "condition:fuel_species|fuel_is_ethane|fuel_is_butane".into();
            f.transform = "vertex of the fuel-table weighing: each law network is evaluated at \
                           (f_ethane, f_butane) = (1, 0), (0, 0), (0, 1) and its outputs are weighed \
                           with the fuel's coordinates; the programmed-stop logits read \
                           fuel.ft_coord of the fuel's row @1"
                .into();
        }
    }
    ModelSpec {
        schema_version: hinge.schema_version,
        spec_id: SpecId::new(spec_id).expect("id"),
        family,
        model_name: model_name(family).into(),
        target: hinge.target.clone(),
        features,
        domain: hinge.domain.clone(),
        tensors,
        forward: ForwardContract {
            entry_point: "ignis_wsp::flame8::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(format!(
            "The hinge-law clocks of spec {} ({}) reading the second fuel table and the diluent \
             table. The tensors are those of that spec, each with the name, shape, role, bounds and \
             fixed value it has there, then {} Then {} And the constant phys.t_stream_k. \
             Hinge lines: T_1 as above; refusal when T_1 < fuel.tad_min_k; c_tad = (T_1 + \
             fuel.dtad_k - norm.tad_ref_k) / norm.tad_scale_k is entry 4 of c, read by every law \
             network and by the domain bound; ln tau_R = N*_se(se.tau ; c with c_logm + fuel.s + \
             dil.dlogm) + dil.dlnr for normal flames, N*_se(se.tau ; c with c_logm + dil.dlogm) + \
             dil.dlnr for inverse flames; logit p_weak = N*_se(se.flash ; c); ln tau_B = N*_ft(ft.tau \
             ; c); programmed stop a: sigmoid(ft.atom_logit[a] + ft.atom_w[a, :] . c) with the \
             indicator entries of c set to fuel.ft_coord. Everything after the clocks is as in the \
             notes of that spec. Uncertainty kind: assumed_noise.",
            hinge.spec_id,
            hinge.spec_hash(),
            fuel2::NOTES,
            fuel::NOTES,
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
    fn eval(&self, x: &[f64; N_IN], scratch: &mut [f64; HIDDEN]) -> f64 {
        hinge_net(self.w1, self.b1, self.w2, self.skip, self.b0, x, scratch)
    }
}

pub(crate) struct Weights<'a> {
    inverse: bool,
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
    fuel: fuel2::Table<'a>,
    t0: f64,
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
    /// The fuel, the diluent and the flame temperature that were read.
    pub inputs: Inputs,
    /// The shift added to `c_logm` in the radiative clock (`fuel.s` + `dil.dlogm`; `fuel.s` is not
    /// read for inverse flames).
    pub radiative_logm_shift: f64,
    /// The shift added to ln tau_R (`dil.dlnr`).
    pub radiative_ln_shift: f64,
    /// The vertices each law network was evaluated at, with their weights.
    pub laws: Vec<Evaluated>,
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
            fuel: fuel2::Table::bind(p),
            t0: p.s(fuel2::T_STREAM),
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

    /// The clocks of the conditions `c` (indicator entries ignored: each law reads its vertices)
    /// for the inputs `inp`.
    pub(crate) fn clocks(&self, c: &[f64; N_IN], inp: &Inputs) -> Clocks {
        let (f, d) = (&inp.fuel, &inp.diluent);
        let mut scratch = [0.0; HIDDEN];
        let mut laws = Vec::with_capacity(3);
        // the oxygen-demand shift (normal flames only) and the diluent's flow shift enter the
        // radiative clock only
        let s_fuel = if self.inverse { 0.0 } else { f.s };
        let mut c_tau = *c;
        c_tau[1] = c[1] + s_fuel + d.dlogm;
        let (ln_tau_r, e) = fuel::weighed("se.tau", &c_tau, I_ETHANE, I_BUTANE, f.se_coord, |x| {
            self.se_tau.eval(x, &mut scratch)
        });
        let ln_tau_r = ln_tau_r + d.dlnr;
        laws.push(e);
        let (logit_weak, e) = fuel::weighed("se.flash", c, I_ETHANE, I_BUTANE, f.se_coord, |x| {
            self.se_flash.eval(x, &mut scratch)
        });
        laws.push(e);
        let (ln_tau_b, e) = fuel::weighed("ft.tau", c, I_ETHANE, I_BUTANE, f.ft_coord, |x| {
            self.ft_tau.eval(x, &mut scratch)
        });
        laws.push(e);
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
            burner_tau_s: ln_tau_b.exp(),
            programmed,
            background_per_s: self.base,
            inputs: inp.clone(),
            radiative_logm_shift: s_fuel + d.dlogm,
            radiative_ln_shift: d.dlnr,
            laws,
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

    /// The eight condition features of a prefix (indicator entries: the fuel's `fuel.ft_coord`,
    /// which the programmed-stop logits read; c_tad from `T_1 + fuel.dtad_k`) and what the tables
    /// read, or the reasons they cannot be formed.
    pub(crate) fn features(
        &self,
        prefix: &PrefixBundle,
    ) -> Result<([f64; N_IN], Inputs), Vec<String>> {
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
        let inp = match inp {
            Some(i) if reasons.is_empty() => i,
            _ => {
                reasons.dedup();
                return Err(reasons);
            }
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
    inp: &Inputs,
    tau: f64,
    stop: Option<f64>,
    times: &[f64],
) -> Forecast {
    let nt = times.len();
    let clocks = wt.clocks(c, inp);
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
    let (c, inp) = match wt.features(prefix) {
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
    Ok(Ok(integrate(&wt, &c, &inp, tau, stop, &times)))
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

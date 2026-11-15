//! The second fuel table of the burner-flame forwards, with a diluent table: the inputs of
//! `flame8.rs`, `flame9.rs`, `flame10.rs` (`acme_normal.wsp14`, `wsp15`, `wsp16`) and of their
//! committee `committee4.rs` (`acme_normal.wsp13`). Design: `docs/wsp/research/phys_sflame_f3.md`,
//! section "For the engineer" (E1 to E3).
//!
//! The fuel table of [`crate::fuel`] keeps its seven columns and its six rows (0 ethane, 1 propane,
//! 2 butane, 3 ethylene, 4 methane, 5 hydrogen); `fuel.s` is bounded by (-3, 3) instead of (-1, 1),
//! and five columns are added:
//!
//! ```text
//! fuel.sigma[k]    oxygen demand per gram; read only to form the shares of a blend
//! fuel.dtad_k[k]   added to the flame temperature the weak-flame law reads
//! fuel.k_rel[k]    thermal conductivity relative to nitrogen (heating law)
//! fuel.zst0[k]     mixture fraction above which fuel.gz acts (growth law)
//! fuel.gz[k]       fall of ln of the critical flux per unit of mixture fraction above fuel.zst0
//! ```
//!
//! A diluent table, rows 0 nitrogen, 1 helium: `dil.admit`, `dil.mw_g_mol` (in place of
//! `phys.mw_n2`), `dil.rho_d`, `dil.d_far`, `dil.le_o2`, `dil.dflux`, `dil.dlogm`, `dil.dlnr`,
//! `dil.k_rel`.
//!
//! **Which fuel.** `fuel_species` names one row (as in [`crate::fuel`]) or a blend `a+b` of two rows,
//! which needs `fuel_blend_x`, the mole fraction of `a` within the fuel. A blend is mixed before
//! anything else:
//!
//! ```text
//! y_a = x W_a / (x W_a + (1 - x) W_b), y_b = 1 - y_a           W = fuel.mw_g_mol
//! w_a = y_a sigma_a / (y_a sigma_a + y_b sigma_b), w_b = 1 - w_a
//! mw_g_mol = x W_a + (1 - x) W_b;   k_rel kept per species
//! every other column c = w_a c_a + w_b c_b   (s, se_coord, ft_coord, dosc, tad_min_k, dtad_k, zst0, gz)
//! admitted when both rows are admitted
//! ```
//!
//! A single fuel is its row, read with no arithmetic. **Which diluent.** `diluent_species` (text,
//! `nitrogen` or `helium`; absent: nitrogen). Any other text, a diluent that is not admitted, a blend
//! with a species that is not a row or without `fuel_blend_x`, and `fuel_blend_x` with a single fuel
//! are refusals at the condition gate. The case files write a text that is not a row (for example
//! `nitrogen+helium`, or `helium in nitrogen`) when the burner stream holds both gases or the burner
//! stream and the ambient differ, so that those are refused here as the design asks.
//!
//! **The flame temperature.** `T_1 = tad` when `dil.le_o2[d] = 1`, otherwise
//! `T_0 + (tad - T_0) (1 - (1 - zst)^(1 / dil.le_o2[d])) / zst` with `T_0 = phys.t_stream_k`; a fuel
//! is refused when `T_1 < fuel.tad_min_k` (the bound of [`crate::fuel::check`] read on `T_1`).
//!
//! Nothing is fitted on this route.

use serde::Serialize;

use ignis_core::contracts::{PrefixBundle, RawValue, TensorRole, TensorSpec};

use crate::fuel::{self, Source, FUELS, ROWS};
use crate::spec::{authored, constant, Params};

/// Rows of the diluent table.
pub const DILUENTS: [&str; 2] = ["nitrogen", "helium"];
/// The fuel-table tensors, in spec order: the seven of [`crate::fuel`], then the five new columns.
pub const FUEL_TENSORS: [&str; 12] = [
    "fuel.admit",
    "fuel.mw_g_mol",
    "fuel.s",
    "fuel.se_coord",
    "fuel.ft_coord",
    "fuel.dosc",
    "fuel.tad_min_k",
    "fuel.sigma",
    "fuel.dtad_k",
    "fuel.k_rel",
    "fuel.zst0",
    "fuel.gz",
];
/// The diluent-table tensors, in spec order.
pub const DIL_TENSORS: [&str; 9] = [
    "dil.admit",
    "dil.mw_g_mol",
    "dil.rho_d",
    "dil.d_far",
    "dil.le_o2",
    "dil.dflux",
    "dil.dlogm",
    "dil.dlnr",
    "dil.k_rel",
];
/// The member scalars of the heating law (growth members and hinge members with the heating law).
pub const MEMBER_SCALARS: [&str; 3] = ["bt.kappa_exp", "bt.tflame_ref_k", "bt.limit_script_c"];
/// The constant of the flame-temperature formula.
pub const T_STREAM: &str = "phys.t_stream_k";
/// Conditions this table reads besides those of [`crate::fuel`].
pub const DILUENT_SPECIES: &str = "diluent_species";
pub const FUEL_BLEND_X: &str = "fuel_blend_x";
pub const BURNER_LIMIT: &str = "burner_limit_c";

/// What the notes of every spec with these tables say about them.
pub const NOTES: &str = "the second fuel table (the seven columns of the first with fuel.s bounded by     (-3, 3), then fuel.sigma, fuel.dtad_k, fuel.k_rel, fuel.zst0, fuel.gz) and the diluent table     (dil.admit, dil.mw_g_mol, dil.rho_d, dil.d_far, dil.le_o2, dil.dflux, dil.dlogm, dil.dlnr,     dil.k_rel; rows 0 nitrogen, 1 helium). Fuel: fuel_species names a row as in the first table, or     a blend a+b of two rows that needs fuel_blend_x, the mole fraction of a within the fuel; for a     blend y_a = x W_a / (x W_a + (1 - x) W_b), w_a = y_a sigma_a / (y_a sigma_a + y_b sigma_b), the     molar mass is x W_a + (1 - x) W_b, k_rel is kept per species, every other column is w_a c_a +     w_b c_b, and both rows must be admitted; a single fuel is its row with no arithmetic. Diluent:     diluent_species, nitrogen or helium, absent meaning nitrogen; any other text (the case files     write one for a stream with both gases or for streams that differ) and a diluent with dil.admit     0 are refusals at the condition gate, as are a blend with a species that is not a row, a blend     without fuel_blend_x, fuel_blend_x with a single fuel, a stored indicator other than 0 with a     blend, and a burner_limit_c that is not finite. Flame temperature: T_1 = tad when dil.le_o2 = 1,     else T_0 + (tad - T_0) (1 - (1 - zst)^(1 / dil.le_o2)) / zst with T_0 = phys.t_stream_k (zst     outside (0, 1) is then a refusal); T_1 < fuel.tad_min_k is a refusal.";

/// A tensor of the fuel table or of the diluent table.
pub fn is_table_tensor(name: &str) -> bool {
    name.starts_with("fuel.") || name.starts_with("dil.")
}

/// The fuel table (section E2 of the design) and the diluent table, every element written by the
/// author.
pub fn tensors() -> Vec<TensorSpec> {
    use TensorRole::*;
    let rows = ROWS as u32;
    let order = "rows 0 ethane, 1 propane, 2 butane, 3 ethylene, 4 methane, 5 hydrogen";
    let dil = "rows 0 nitrogen, 1 helium";
    let mut t: Vec<TensorSpec> = fuel::tensors()
        .into_iter()
        .map(|mut t| {
            if t.name == "fuel.s" {
                t = authored(
                    "fuel.s",
                    &[rows],
                    PhysicalParameter,
                    Some("1"),
                    Some((-3.0, 3.0)),
                    &format!("ln of the oxygen demand per gram of the fuel relative to ethane, ln(sigma / sigma_ethane); 0 for the fuels the laws were typed on. Normal flames: added to c_logm in the radiative clock of the hinge law, subtracted from ln m_cr in the growth law ({order})."),
                );
            }
            t
        })
        .collect();
    t.extend([
        authored("fuel.sigma", &[rows], PhysicalParameter, Some("g/g"), Some((1.0, 10.0)),
                 &format!("Oxygen demand per gram of the fuel; read only to form the shares of a blend ({order}).")),
        authored("fuel.dtad_k", &[rows], PhysicalParameter, Some("K"), Some((-1000.0, 1000.0)),
                 &format!("Added to the flame temperature the weak-flame law reads (c_tad of se.flash and its domain bound; not the heating law) ({order}).")),
        authored("fuel.k_rel", &[rows], PhysicalParameter, Some("1"), Some((0.1, 20.0)),
                 &format!("Thermal conductivity of the fuel relative to nitrogen, in the conductivity of the burner stream ({order}).")),
        authored("fuel.zst0", &[rows], PhysicalParameter, Some("1"), Some((0.0, 1.0)),
                 &format!("Stoichiometric mixture fraction above which fuel.gz acts (growth law) ({order}).")),
        authored("fuel.gz", &[rows], PhysicalParameter, Some("1"), Some((-10.0, 10.0)),
                 &format!("Fall of ln of the critical flux per unit of stoichiometric mixture fraction above fuel.zst0 (growth law) ({order}).")),
        authored("dil.admit", &[2], Weight, Some("1"), Some((0.0, 1.0)),
                 &format!("1: the release answers for the diluent of the row; 0: refusal at the condition gate ({dil}).")),
        authored("dil.mw_g_mol", &[2], PhysicalParameter, Some("g/mol"), Some((1.0, 200.0)),
                 &format!("Molar mass of the diluent of both streams; replaces phys.mw_n2 in the reactant mass fraction Y_R ({dil}).")),
        authored("dil.rho_d", &[2], PhysicalParameter, Some("1"), Some((0.1, 10.0)),
                 &format!("Factor on se.rho_d_g_m_s wherever the quasi-steady radius r_ss is formed; a factor of exactly 1 is not applied ({dil}).")),
        authored("dil.d_far", &[2], PhysicalParameter, Some("1"), Some((0.1, 10.0)),
                 &format!("Factor on the growth diffusivity (growth law); a factor of exactly 1 is not applied ({dil}).")),
        authored("dil.le_o2", &[2], PhysicalParameter, Some("1"), Some((0.2, 5.0)),
                 &format!("Lewis number of oxygen relative to nitrogen's: the flame temperature T_1 = T_0 + (tad - T_0) (1 - (1 - zst)^(1 / le)) / zst; exactly 1: T_1 = tad ({dil}).")),
        authored("dil.dflux", &[2], PhysicalParameter, Some("1"), Some((-3.0, 3.0)),
                 &format!("Added to ln m_cr (growth law) ({dil}).")),
        authored("dil.dlogm", &[2], PhysicalParameter, Some("1"), Some((-5.0, 5.0)),
                 &format!("Added to c_logm in the radiative clock se.tau (hinge law) ({dil}).")),
        authored("dil.dlnr", &[2], PhysicalParameter, Some("1"), Some((-5.0, 5.0)),
                 &format!("Added to ln tau_R (hinge law) ({dil}).")),
        authored("dil.k_rel", &[2], PhysicalParameter, Some("1"), Some((0.1, 20.0)),
                 &format!("Thermal conductivity of the diluent relative to nitrogen, in the conductivity of the burner stream ({dil}).")),
    ]);
    t
}

/// The member scalars of the heating law.
pub fn member_scalars() -> Vec<TensorSpec> {
    use TensorRole::*;
    vec![
        authored("bt.kappa_exp", &[], PhysicalParameter, Some("1"), Some((0.0, 2.0)),
                 "Exponent of the conductivity of the burner stream in the equilibrium rise: ln dT_eq += bt.kappa_exp ln(1 + kappa), kappa = sum over the species of the stream of x_i (k_rel,i - 1)."),
        authored("bt.tflame_ref_k", &[], PhysicalParameter, Some("K"), Some((0.0, 4000.0)),
                 "Flame temperature at which the heating law is left as typed: ln dT_eq += ln((T_1 - T_b) / (bt.tflame_ref_k - T_b)), T_b = bt.t0_c + 273.15; 0 switches the factor off."),
        authored("bt.limit_script_c", &[], PhysicalParameter, Some("degC"), Some((0.0, 1500.0)),
                 "The script setting of the burner temperature limit under which this member's limits were typed: with the condition burner_limit_c every weighted limit is bt.limit_c[l] + (burner_limit_c - bt.limit_script_c)."),
    ]
}

/// The constant of the flame-temperature formula.
pub fn t_stream() -> TensorSpec {
    constant(
        T_STREAM,
        298.15,
        "K",
        "Temperature of the streams from which the adiabatic flame temperature is counted (T_0 of T_1).",
    )
}

/// The two tables of a bound release.
#[derive(Clone, Copy)]
pub struct Table<'a> {
    admit: &'a [f64],
    mw: &'a [f64],
    s: &'a [f64],
    se: &'a [f64],
    ft: &'a [f64],
    dosc: &'a [f64],
    tad_min: &'a [f64],
    sigma: &'a [f64],
    dtad: &'a [f64],
    k_rel: &'a [f64],
    zst0: &'a [f64],
    gz: &'a [f64],
    d_admit: &'a [f64],
    d_mw: &'a [f64],
    d_rho_d: &'a [f64],
    d_far: &'a [f64],
    d_le: &'a [f64],
    d_dflux: &'a [f64],
    d_dlogm: &'a [f64],
    d_dlnr: &'a [f64],
    d_k_rel: &'a [f64],
}

/// The fuel a forecast reads: one row, or a blend of two (reported in the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct Fuel {
    /// The name as the prefix gives it (`ethylene`, `hydrogen+methane`).
    pub name: String,
    pub rows: Vec<usize>,
    pub source: Source,
    /// Mole fraction of the first row within the fuel (a blend only).
    pub blend_x: Option<f64>,
    /// Shares of the oxygen demand of the two rows (a blend only).
    pub shares: Option<[f64; 2]>,
    pub mw_g_mol: f64,
    pub s: f64,
    pub se_coord: [f64; 2],
    pub ft_coord: [f64; 2],
    pub dosc: f64,
    pub tad_min_k: f64,
    pub dtad_k: f64,
    pub zst0: f64,
    pub gz: f64,
    /// `(mole fraction within the fuel, k_rel)` of each species of the fuel.
    pub k_rel: Vec<(f64, f64)>,
}

/// The diluent row a forecast reads (reported in the diagnostics).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Diluent {
    pub row: usize,
    pub name: &'static str,
    pub mw_g_mol: f64,
    pub rho_d: f64,
    pub d_far: f64,
    pub le_o2: f64,
    pub dflux: f64,
    pub dlogm: f64,
    pub dlnr: f64,
    pub k_rel: f64,
}

fn admit_reason(what: &str, name: &str, tensor: &str, row: usize, v: f64) -> Option<String> {
    if v == 0.0 {
        Some(format!(
            "the release does not answer for {what} {name} ({tensor}[{row}] = 0)"
        ))
    } else if v != 1.0 {
        Some(format!("{tensor}[{row}] = {v} is not 0 or 1"))
    } else {
        None
    }
}

impl<'a> Table<'a> {
    pub fn bind(p: &Params<'a>) -> Self {
        Table {
            admit: p.v("fuel.admit"),
            mw: p.v("fuel.mw_g_mol"),
            s: p.v("fuel.s"),
            se: p.v("fuel.se_coord"),
            ft: p.v("fuel.ft_coord"),
            dosc: p.v("fuel.dosc"),
            tad_min: p.v("fuel.tad_min_k"),
            sigma: p.v("fuel.sigma"),
            dtad: p.v("fuel.dtad_k"),
            k_rel: p.v("fuel.k_rel"),
            zst0: p.v("fuel.zst0"),
            gz: p.v("fuel.gz"),
            d_admit: p.v("dil.admit"),
            d_mw: p.v("dil.mw_g_mol"),
            d_rho_d: p.v("dil.rho_d"),
            d_far: p.v("dil.d_far"),
            d_le: p.v("dil.le_o2"),
            d_dflux: p.v("dil.dflux"),
            d_dlogm: p.v("dil.dlogm"),
            d_dlnr: p.v("dil.dlnr"),
            d_k_rel: p.v("dil.k_rel"),
        }
    }

    /// The row `k`, read with no arithmetic.
    pub fn single(&self, k: usize, source: Source) -> Fuel {
        Fuel {
            name: FUELS[k].to_string(),
            rows: vec![k],
            source,
            blend_x: None,
            shares: None,
            mw_g_mol: self.mw[k],
            s: self.s[k],
            se_coord: [self.se[2 * k], self.se[2 * k + 1]],
            ft_coord: [self.ft[2 * k], self.ft[2 * k + 1]],
            dosc: self.dosc[k],
            tad_min_k: self.tad_min[k],
            dtad_k: self.dtad[k],
            zst0: self.zst0[k],
            gz: self.gz[k],
            k_rel: vec![(1.0, self.k_rel[k])],
        }
    }

    /// The blend of rows `a` and `b` with mole fraction `x` of `a` within the fuel.
    pub fn blend(&self, a: usize, b: usize, x: f64, name: &str) -> Fuel {
        let (wa_mw, wb_mw) = (self.mw[a], self.mw[b]);
        let y_a = x * wa_mw / (x * wa_mw + (1.0 - x) * wb_mw);
        let y_b = 1.0 - y_a;
        let w_a = y_a * self.sigma[a] / (y_a * self.sigma[a] + y_b * self.sigma[b]);
        let w_b = 1.0 - w_a;
        let mix = |c: &[f64], i: usize, j: usize| w_a * c[i] + w_b * c[j];
        Fuel {
            name: name.to_string(),
            rows: vec![a, b],
            source: Source::FuelSpecies,
            blend_x: Some(x),
            shares: Some([w_a, w_b]),
            mw_g_mol: x * wa_mw + (1.0 - x) * wb_mw,
            s: mix(self.s, a, b),
            se_coord: [
                mix(self.se, 2 * a, 2 * b),
                mix(self.se, 2 * a + 1, 2 * b + 1),
            ],
            ft_coord: [
                mix(self.ft, 2 * a, 2 * b),
                mix(self.ft, 2 * a + 1, 2 * b + 1),
            ],
            dosc: mix(self.dosc, a, b),
            tad_min_k: mix(self.tad_min, a, b),
            dtad_k: mix(self.dtad, a, b),
            zst0: mix(self.zst0, a, b),
            gz: mix(self.gz, a, b),
            k_rel: vec![(x, self.k_rel[a]), (1.0 - x, self.k_rel[b])],
        }
    }

    pub fn diluent(&self, d: usize) -> Diluent {
        Diluent {
            row: d,
            name: DILUENTS[d],
            mw_g_mol: self.d_mw[d],
            rho_d: self.d_rho_d[d],
            d_far: self.d_far[d],
            le_o2: self.d_le[d],
            dflux: self.d_dflux[d],
            dlogm: self.d_dlogm[d],
            dlnr: self.d_dlnr[d],
            k_rel: self.d_k_rel[d],
        }
    }

    /// The fuel and the diluent of a prefix with the admissions of the release applied (the
    /// temperature bound is applied on `T_1` by the caller, [`temperature_reason`]).
    pub fn resolve(&self, prefix: &PrefixBundle) -> Result<(Fuel, Diluent), Vec<String>> {
        let mut reasons = Vec::new();
        let fuel = match fuel_of(prefix) {
            Ok(Spec::Single(k, source)) => {
                if let Some(r) = admit_reason("fuel", FUELS[k], "fuel.admit", k, self.admit[k]) {
                    reasons.push(r);
                }
                Some(self.single(k, source))
            }
            Ok(Spec::Blend(a, b, x, name)) => {
                for k in [a, b] {
                    if let Some(r) = admit_reason("fuel", FUELS[k], "fuel.admit", k, self.admit[k])
                    {
                        reasons.push(r);
                    }
                }
                Some(self.blend(a, b, x, &name))
            }
            Err(r) => {
                reasons.extend(r);
                None
            }
        };
        let dil = match diluent_of(prefix) {
            Ok(d) => {
                if let Some(r) =
                    admit_reason("diluent", DILUENTS[d], "dil.admit", d, self.d_admit[d])
                {
                    reasons.push(r);
                }
                Some(self.diluent(d))
            }
            Err(r) => {
                reasons.push(r);
                None
            }
        };
        match (fuel, dil) {
            (Some(f), Some(d)) if reasons.is_empty() => Ok((f, d)),
            _ => Err(reasons),
        }
    }
}

/// What `fuel_species` names.
enum Spec {
    Single(usize, Source),
    Blend(usize, usize, f64, String),
}

fn fuel_of(prefix: &PrefixBundle) -> Result<Spec, Vec<String>> {
    let text = match prefix
        .known_conditions
        .get("fuel_species")
        .and_then(|m| m.raw_value.as_ref())
    {
        Some(RawValue::Text(s)) => Some(s.clone()),
        _ => None,
    };
    let blend_x = prefix.known_conditions.get(FUEL_BLEND_X);
    let parts: Vec<String> = text
        .as_deref()
        .map(|s| s.split('+').map(|p| p.trim().to_string()).collect())
        .unwrap_or_default();
    if parts.len() <= 1 {
        if blend_x.is_some() {
            return Err(vec![format!(
                "{FUEL_BLEND_X} is given for a single fuel ({})",
                text.as_deref().unwrap_or("no fuel_species")
            )]);
        }
        // a single row: the rule of fuel.rs, unchanged
        return fuel::row_of(prefix).map(|(k, s)| Spec::Single(k, s));
    }
    let text = text.unwrap_or_default();
    let mut reasons = Vec::new();
    if parts.len() != 2 {
        return Err(vec![format!(
            "fuel_species {text:?} names {} species; a blend is two rows of the table",
            parts.len()
        )]);
    }
    let rows: Vec<Option<usize>> = parts.iter().map(|p| fuel::species_row(p)).collect();
    for (p, r) in parts.iter().zip(&rows) {
        if r.is_none() {
            reasons.push(format!(
                "fuel_species {text:?}: {p:?} is not a fuel of the table (ethane, propane, butane or n-butane, ethylene, methane, hydrogen)"
            ));
        }
    }
    let x = match blend_x {
        None => {
            reasons.push(format!(
                "fuel_species {text:?} is a blend and the prefix has no {FUEL_BLEND_X}"
            ));
            None
        }
        Some(m) => match m.value_si {
            Some(v) if v.is_finite() && (0.0..=1.0).contains(&v) => Some(v),
            v => {
                reasons.push(format!(
                    "{FUEL_BLEND_X} = {v:?} is not a mole fraction in [0, 1]"
                ));
                None
            }
        },
    };
    // a blend is neither ethane nor butane: stored indicators, when present, must be 0
    for key in ["fuel_is_ethane", "fuel_is_butane"] {
        if let Some(v) = prefix.known_conditions.get(key).and_then(|m| m.value_si) {
            if v != 0.0 {
                reasons.push(format!(
                    "{key} = {v} is not that of the blend {text:?} (0): the stored indicators and the species disagree"
                ));
            }
        }
    }
    match (rows.as_slice(), x) {
        ([Some(a), Some(b)], Some(x)) if reasons.is_empty() => {
            Ok(Spec::Blend(*a, *b, x, text.trim().to_lowercase()))
        }
        _ => Err(reasons),
    }
}

/// The diluent row of a prefix: `diluent_species`, absent meaning nitrogen.
pub fn diluent_of(prefix: &PrefixBundle) -> Result<usize, String> {
    match prefix.known_conditions.get(DILUENT_SPECIES) {
        None => Ok(0),
        Some(m) => match m.raw_value.as_ref() {
            Some(RawValue::Text(s)) => {
                let name = s.trim().to_lowercase();
                DILUENTS.iter().position(|d| *d == name).ok_or_else(|| {
                    format!(
                        "{DILUENT_SPECIES} {s:?} is not a diluent of the table (nitrogen, helium): one gas in the burner stream and in the ambient"
                    )
                })
            }
            other => Err(format!("{DILUENT_SPECIES} is not a text ({other:?})")),
        },
    }
}

/// The script's burner temperature limit, when the prefix carries it.
pub fn burner_limit(prefix: &PrefixBundle) -> Result<Option<f64>, String> {
    match prefix.known_conditions.get(BURNER_LIMIT) {
        None => Ok(None),
        Some(m) => match m.value_si {
            Some(v) if v.is_finite() => Ok(Some(v)),
            v => Err(format!(
                "{BURNER_LIMIT} = {v:?} is not a finite temperature"
            )),
        },
    }
}

/// The flame temperature `T_1` of the design: `tad` when `le = 1`, otherwise
/// `T_0 + (tad - T_0) (1 - (1 - zst)^(1 / le)) / zst`.
#[inline]
pub fn flame_temperature(tad: f64, zst: f64, le: f64, t0: f64) -> f64 {
    if le == 1.0 {
        tad
    } else {
        t0 + (tad - t0) * (1.0 - (1.0 - zst).powf(1.0 / le)) / zst
    }
}

/// The refusal of a flame temperature below the fuel's bound.
pub fn temperature_reason(fuel: &Fuel, t1: f64, tad: f64) -> Option<String> {
    (t1.is_finite() && t1 < fuel.tad_min_k).then(|| {
        format!(
            "the flame temperature T_1 = {t1} K (tad = {tad} K) is below the bound of {} in this release (fuel.tad_min_k = {} K)",
            fuel.name, fuel.tad_min_k
        )
    })
}

/// `kappa = sum over the species of the burner stream of x_i (k_rel,i - 1)`: the fuel species
/// (their mole fraction `x_fuel` of the stream split by the fuel's own fractions) and the diluent
/// (`1 - x_fuel`).
#[inline]
pub fn kappa(fuel: &Fuel, dil: &Diluent, x_fuel: f64) -> f64 {
    let mut k = 0.0;
    for &(x, k_rel) in &fuel.k_rel {
        k += x_fuel * x * (k_rel - 1.0);
    }
    k + (1.0 - x_fuel) * (dil.k_rel - 1.0)
}

/// What a member reads of a prefix through the two tables (reported in the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct Inputs {
    pub fuel: Fuel,
    pub diluent: Diluent,
    /// The adiabatic flame temperature of the prefix and the flame temperature `T_1`, K.
    pub tad_k: f64,
    pub t1_k: f64,
    /// The script's burner temperature limit, when the prefix carries it.
    pub burner_limit_c: Option<f64>,
    /// The stoichiometric mixture fraction and the fuel mole fraction of the burner stream.
    pub zst: f64,
    pub x_fuel: f64,
}

impl Table<'_> {
    /// The fuel, the diluent, `T_1` and the script's limit of a prefix, with every refusal of the
    /// tables: admissions, the zst that `T_1` needs, the temperature bound on `T_1`, a limit that is
    /// not finite.
    pub fn inputs(
        &self,
        prefix: &PrefixBundle,
        tad: f64,
        zst: f64,
        x_fuel: f64,
        t0: f64,
    ) -> Result<Inputs, Vec<String>> {
        let (fuel, diluent) = self.resolve(prefix)?;
        let mut reasons = Vec::new();
        if diluent.le_o2 != 1.0 && zst.is_finite() && !(zst > 0.0 && zst < 1.0) {
            reasons.push(format!(
                "zst = {zst} is not inside (0, 1): the flame temperature of a {} flame needs it",
                diluent.name
            ));
        }
        let t1 = flame_temperature(tad, zst, diluent.le_o2, t0);
        if let Some(r) = temperature_reason(&fuel, t1, tad) {
            reasons.push(r);
        }
        let limit = match burner_limit(prefix) {
            Ok(l) => l,
            Err(r) => {
                reasons.push(r);
                None
            }
        };
        if reasons.is_empty() {
            Ok(Inputs {
                fuel,
                diluent,
                tad_k: tad,
                t1_k: t1,
                burner_limit_c: limit,
                zst,
                x_fuel,
            })
        } else {
            Err(reasons)
        }
    }
}

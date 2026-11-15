//! The fuel table of the burner-flame forwards `flame5.rs`, `flame6.rs` and `flame7.rs`
//! (`acme_*.wsp9`, `acme_*.wsp10`, `acme_*.wsp11`) and of their committee (`committee3.rs`).
//!
//! The earlier burner-flame forwards know a fuel only as two indicator features, `f_ethane` and
//! `f_butane` (propane is the reference), and refuse every other fuel. A release of these forwards
//! holds a typed table with one row per fuel, fixed by name:
//!
//! ```text
//! rows   0 ethane, 1 propane, 2 butane, 3 ethylene, 4 methane, 5 hydrogen
//!
//! fuel.admit[k]        1: the release answers for fuel k; 0: refusal at the condition gate
//! fuel.mw_g_mol[k]     molar mass (replaces phys.mw_ethane, phys.mw_propane, phys.mw_butane)
//! fuel.s[k]            ln of the oxygen demand per gram relative to ethane
//! fuel.se_coord[k, :]  coordinates (e, b) in the ethane and butane directions, self-extinction laws
//! fuel.ft_coord[k, :]  the same for the flow-termination laws (burner limit, programmed stops)
//! fuel.dosc[k]         shift of ln of the quench-clock median
//! fuel.tad_min_k[k]    below this adiabatic temperature the fuel is refused; 0: no bound
//! ```
//!
//! A law network `N` that reads the two indicator entries is evaluated at the three alkane
//! vertices, E = (1, 0), P = (0, 0), B = (0, 1), and its **outputs** are weighed with the fuel's
//! coordinates:
//!
//! ```text
//! (w_E, w_P, w_B) = (e, 1 - e - b, b)
//! N*(x) = sum over the vertices v with w_v != 0 of w_v N(x with its two indicator entries set to v)
//! ```
//!
//! A vertex with weight zero is not evaluated. For ethane (1, 0), propane (0, 0) and butane (0, 1)
//! the weights are a unit vector and `N*` is one evaluation of the same network on the same numbers
//! times 1: the forecast of an alkane is that of the indicator forward bit for bit.
//!
//! Which row: the prefix condition `fuel_species` (text: ethane, propane, butane or n-butane,
//! ethylene, methane, hydrogen; case and surrounding blanks ignored). Without it, the alkane the two
//! stored indicators `fuel_is_ethane`, `fuel_is_butane` name: (1, 0) ethane, (0, 0) propane,
//! (0, 1) butane; any other pair is a refusal. When both are present, an indicator that is not the
//! row's own (`fuel_is_ethane` is 1 for row 0 only, `fuel_is_butane` for row 2 only) is a refusal.
//! A species the table does not hold, a fuel with `fuel.admit` 0, and an adiabatic temperature below
//! `fuel.tad_min_k` are refusals at the condition gate.
//!
//! Source of the design: `docs/wsp/research/phys_spherical_fuels.md`, section 3. Nothing is fitted
//! on this route.

use serde::Serialize;

use ignis_core::contracts::{PrefixBundle, RawValue, TensorRole, TensorSpec};

use crate::spec::{authored, Params};

/// Rows of the fuel table.
pub const ROWS: usize = 6;
/// The fuel of each row, fixed by the spec.
pub const FUELS: [&str; ROWS] = [
    "ethane", "propane", "butane", "ethylene", "methane", "hydrogen",
];
/// The tensors of the table, in spec order.
pub const TENSORS: [&str; 7] = [
    "fuel.admit",
    "fuel.mw_g_mol",
    "fuel.s",
    "fuel.se_coord",
    "fuel.ft_coord",
    "fuel.dosc",
    "fuel.tad_min_k",
];
/// The molar-mass constants of the growth-clock spec that `fuel.mw_g_mol` replaces.
pub const ALKANE_MOLAR_MASSES: [&str; 3] = ["phys.mw_ethane", "phys.mw_propane", "phys.mw_butane"];
/// The three alkane vertices `(f_ethane, f_butane)` at which a law network is evaluated, in the
/// order the weighed sum runs.
pub const VERTICES: [(&str, [f64; 2]); 3] = [
    ("ethane", [1.0, 0.0]),
    ("propane", [0.0, 0.0]),
    ("butane", [0.0, 1.0]),
];

/// What the notes of every spec with the fuel table say about it.
pub const NOTES: &str = "the fuel table fuel.admit, fuel.mw_g_mol, fuel.s, fuel.se_coord, \
    fuel.ft_coord, fuel.dosc and fuel.tad_min_k (rows 0 ethane, 1 propane, 2 butane, 3 ethylene, 4 \
    methane, 5 hydrogen). The row k of a prefix: its condition fuel_species (text, case and \
    surrounding blanks ignored: ethane, propane, butane or n-butane, ethylene, methane, hydrogen); \
    without it, the stored indicators fuel_is_ethane, fuel_is_butane: (1, 0) row 0, (0, 0) row 1, \
    (0, 1) row 2. Refused at the condition gate: a species not in the table; without a species, an \
    indicator missing or a pair naming no row; with both, a stored indicator that is not the row's \
    own (fuel_is_ethane is 1 for row 0 only, fuel_is_butane for row 2 only); fuel.admit[k] = 0, or \
    a value that is not 0 or 1; tad < fuel.tad_min_k[k]. The prefix carries no condition on the \
    balance gas of either stream or on the constancy of the flow, so those refusals are not made. \
    Weighing of a law network N that reads the indicator entries: (e, b) the coordinates of row k \
    for the law's group, vertex weights (w_E, w_P, w_B) = (e, 1 - e - b, b) in IEEE arithmetic, N*(x) \
    = sum over the vertices E (1, 0), P (0, 0), B (0, 1) in that order, skipping a vertex whose \
    weight is exactly zero, of w_v N(x with the two indicator entries set to the vertex); a single \
    vertex gives w_v N exactly";

/// A tensor of the fuel table.
pub fn is_fuel_tensor(name: &str) -> bool {
    name.starts_with("fuel.")
}

/// A molar-mass constant of the growth-clock spec that the fuel table replaces.
pub fn is_alkane_molar_mass(name: &str) -> bool {
    ALKANE_MOLAR_MASSES.contains(&name)
}

/// The tensors of the fuel table (section 3.2 of the design), every element written by the author.
pub fn tensors() -> Vec<TensorSpec> {
    use TensorRole::*;
    let rows = ROWS as u32;
    let order = "rows 0 ethane, 1 propane, 2 butane, 3 ethylene, 4 methane, 5 hydrogen";
    vec![
        authored(
            "fuel.admit",
            &[rows],
            Weight,
            Some("1"),
            Some((0.0, 1.0)),
            &format!("1: the release answers for the fuel of the row; 0: a prefix of that fuel is refused at the condition gate ({order})."),
        ),
        authored(
            "fuel.mw_g_mol",
            &[rows],
            PhysicalParameter,
            Some("g/mol"),
            Some((1.0, 200.0)),
            &format!("Molar mass of the fuel of the row; replaces phys.mw_ethane, phys.mw_propane and phys.mw_butane ({order})."),
        ),
        authored(
            "fuel.s",
            &[rows],
            PhysicalParameter,
            Some("1"),
            Some((-1.0, 1.0)),
            &format!("ln of the oxygen demand per gram of the fuel relative to ethane, ln(sigma / sigma_ethane); 0 for the fuels the laws were typed on. Normal flames only: added to c_logm in the radiative clock of the hinge law, subtracted from ln m_cr in the growth law ({order})."),
        ),
        authored(
            "fuel.se_coord",
            &[rows, 2],
            Weight,
            Some("1"),
            Some((-2.0, 3.0)),
            &format!("Coordinates (e, b) of the fuel in the ethane and butane directions for the self-extinction laws: their outputs are weighed (e, 1 - e - b, b) at the ethane, propane and butane vertices ({order})."),
        ),
        authored(
            "fuel.ft_coord",
            &[rows, 2],
            Weight,
            Some("1"),
            Some((-3.0, 4.0)),
            &format!("The same for the flow-termination laws (burner limit; the indicator entries of the programmed-stop logits) ({order})."),
        ),
        authored(
            "fuel.dosc",
            &[rows],
            PhysicalParameter,
            Some("ln(1)"),
            Some((-3.0, 3.0)),
            &format!("Shift of ln of the median of the oscillation and quench clock; 0 for the fuels the clock was typed on; read by the growth law only ({order})."),
        ),
        authored(
            "fuel.tad_min_k",
            &[rows],
            PhysicalParameter,
            Some("K"),
            Some((0.0, 3000.0)),
            &format!("Below this adiabatic flame temperature (condition tad) a prefix of the fuel is refused at the condition gate; 0: no bound ({order})."),
        ),
    ]
}

/// The fuel table of a bound release.
#[derive(Clone, Copy)]
pub struct Table<'a> {
    admit: &'a [f64],
    mw: &'a [f64],
    s: &'a [f64],
    se: &'a [f64],
    ft: &'a [f64],
    dosc: &'a [f64],
    tad_min: &'a [f64],
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
        }
    }

    /// The row `k` of the table.
    pub fn fuel(&self, k: usize, source: Source) -> Fuel {
        Fuel {
            row: k,
            name: FUELS[k],
            source,
            mw_g_mol: self.mw[k],
            s: self.s[k],
            se_coord: [self.se[2 * k], self.se[2 * k + 1]],
            ft_coord: [self.ft[2 * k], self.ft[2 * k + 1]],
            dosc: self.dosc[k],
            tad_min_k: self.tad_min[k],
        }
    }
}

/// Where the row of a prefix was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The condition `fuel_species`.
    FuelSpecies,
    /// The stored indicators `fuel_is_ethane`, `fuel_is_butane` (no `fuel_species`).
    Indicators,
}

/// The row of the table a forecast reads (reported in the diagnostics).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Fuel {
    pub row: usize,
    pub name: &'static str,
    pub source: Source,
    pub mw_g_mol: f64,
    pub s: f64,
    pub se_coord: [f64; 2],
    pub ft_coord: [f64; 2],
    pub dosc: f64,
    pub tad_min_k: f64,
}

/// The row of a fuel name: the six names of the table, and `n-butane` for butane.
pub fn species_row(name: &str) -> Option<usize> {
    let name = name.trim().to_lowercase();
    if name == "n-butane" {
        return Some(2);
    }
    FUELS.iter().position(|f| *f == name)
}

/// The row of a prefix and the reasons it is refused, before the release's own gates (admission
/// and the temperature bound, see [`check`]).
pub fn row_of(prefix: &PrefixBundle) -> Result<(usize, Source), Vec<String>> {
    let species = match prefix
        .known_conditions
        .get("fuel_species")
        .and_then(|m| m.raw_value.as_ref())
    {
        Some(RawValue::Text(s)) => Some(s.clone()),
        _ => None,
    };
    let indicator = |key: &str| prefix.known_conditions.get(key).and_then(|m| m.value_si);
    let (fe, fb) = (indicator("fuel_is_ethane"), indicator("fuel_is_butane"));
    let mut reasons = Vec::new();
    let (k, source) = match species {
        Some(s) => match species_row(&s) {
            Some(k) => (k, Source::FuelSpecies),
            None => {
                return Err(vec![format!(
                    "fuel_species {s:?} is not a fuel of the table (ethane, propane, butane or n-butane, ethylene, methane, hydrogen)"
                )])
            }
        },
        None => match (fe, fb) {
            (Some(e), Some(b)) => match (e, b) {
                (e, b) if e == 1.0 && b == 0.0 => (0, Source::Indicators),
                (e, b) if e == 0.0 && b == 0.0 => (1, Source::Indicators),
                (e, b) if e == 0.0 && b == 1.0 => (2, Source::Indicators),
                (e, b) => {
                    return Err(vec![format!(
                        "fuel_is_ethane = {e}, fuel_is_butane = {b} name no fuel (ethane is (1, 0), propane (0, 0), butane (0, 1)) and the prefix has no fuel_species"
                    )])
                }
            },
            _ => {
                for (key, v) in [("fuel_is_ethane", fe), ("fuel_is_butane", fb)] {
                    if v.is_none() {
                        reasons.push(format!("{key} missing and the prefix has no fuel_species"));
                    }
                }
                return Err(reasons);
            }
        },
    };
    // stored indicators that are present must be the row's own
    for (key, v, own) in [
        ("fuel_is_ethane", fe, f64::from(u8::from(k == 0))),
        ("fuel_is_butane", fb, f64::from(u8::from(k == 2))),
    ] {
        if let Some(v) = v {
            if v != own {
                reasons.push(format!(
                    "{key} = {v} is not that of fuel_species {:?} ({own}): the stored indicators and the species disagree",
                    FUELS[k]
                ));
            }
        }
    }
    if reasons.is_empty() {
        Ok((k, source))
    } else {
        Err(reasons)
    }
}

/// The release's own gates for a fuel: its admission and its temperature bound.
pub fn check(fuel: &Fuel, admit: f64, tad: f64) -> Vec<String> {
    let mut reasons = Vec::new();
    if admit == 0.0 {
        reasons.push(format!(
            "the release does not answer for {} (fuel.admit[{}] = 0)",
            fuel.name, fuel.row
        ));
    } else if admit != 1.0 {
        reasons.push(format!("fuel.admit[{}] = {admit} is not 0 or 1", fuel.row));
    }
    if tad.is_finite() && tad < fuel.tad_min_k {
        reasons.push(format!(
            "tad = {tad} K is below the bound of {} in this release (fuel.tad_min_k[{}] = {} K): the weak-flame law is an alkane law",
            fuel.name, fuel.row, fuel.tad_min_k
        ));
    }
    reasons
}

impl Table<'_> {
    /// The row of a prefix, with the release's gates applied; `tad` is the condition in K.
    pub fn resolve(&self, prefix: &PrefixBundle, tad: f64) -> Result<Fuel, Vec<String>> {
        let (k, source) = row_of(prefix)?;
        let fuel = self.fuel(k, source);
        let reasons = check(&fuel, self.admit[k], tad);
        if reasons.is_empty() {
            Ok(fuel)
        } else {
            Err(reasons)
        }
    }
}

/// The vertex weights `(w_E, w_P, w_B) = (e, 1 - e - b, b)` of coordinates `(e, b)`.
#[inline]
pub fn weights(coord: [f64; 2]) -> [f64; 3] {
    let [e, b] = coord;
    [e, 1.0 - e - b, b]
}

/// Which vertices of one law were evaluated, and with what weights (reported in the diagnostics).
#[derive(Debug, Clone, Serialize)]
pub struct Evaluated {
    pub law: &'static str,
    /// `(w_E, w_P, w_B)`.
    pub weights: [f64; 3],
    /// The vertices whose weight is not zero, in the order of the sum.
    pub vertices: Vec<&'static str>,
}

/// `N*(x)`: the outputs of `eval` at the vertices with a non-zero weight, weighed and summed in the
/// order E, P, B, with `x[ie], x[ib]` set to the vertex. A single vertex gives `w N` exactly.
#[inline]
pub fn weighed<const N: usize>(
    law: &'static str,
    x: &[f64; N],
    ie: usize,
    ib: usize,
    coord: [f64; 2],
    mut eval: impl FnMut(&[f64; N]) -> f64,
) -> (f64, Evaluated) {
    let w = weights(coord);
    let mut acc: Option<f64> = None;
    let mut vertices = Vec::with_capacity(3);
    for (wv, (name, v)) in w.iter().zip(VERTICES) {
        if *wv == 0.0 {
            continue;
        }
        let mut xv = *x;
        xv[ie] = v[0];
        xv[ib] = v[1];
        let term = wv * eval(&xv);
        acc = Some(acc.map_or(term, |a| a + term));
        vertices.push(name);
    }
    (
        // every weight zero: e = b = 0 and 1 - e - b = 1 cannot both hold, so this is unreachable for
        // finite coordinates; a sum over no vertex is zero
        acc.unwrap_or(0.0),
        Evaluated {
            law,
            weights: w,
            vertices,
        },
    )
}

/// `x` with its two indicator entries set to the coordinates `(e, b)` (the linear programmed-stop
/// logits read them directly).
#[inline]
pub fn with_coord<const N: usize>(x: &[f64; N], ie: usize, ib: usize, coord: [f64; 2]) -> [f64; N] {
    let mut y = *x;
    y[ie] = coord[0];
    y[ib] = coord[1];
    y
}

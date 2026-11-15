//! Fresh, property-based numerical W0. Self-contained scientific kernel, not a wire contract.
//! See docs/wsp/research/astra_property_droplet_w0.md for derivations and exposure ledger.
//! No fuel lookup, calibration, experimental data, chemistry classification or optical inference.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub &'static str);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

fn finite(x: f64) -> Result<f64> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(Error("nonfinite input or arithmetic"))
    }
}
fn positive(x: f64) -> Result<f64> {
    if finite(x)? > 0.0 {
        Ok(x)
    } else {
        Err(Error("positive value required"))
    }
}
fn nonnegative(x: f64) -> Result<f64> {
    if finite(x)? >= 0.0 {
        Ok(x)
    } else {
        Err(Error("negative value"))
    }
}
fn fraction(x: f64) -> Result<f64> {
    if (0.0..=1.0).contains(&finite(x)?) {
        Ok(x)
    } else {
        Err(Error("fraction outside [0,1]"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    ExternalDetermination,
    ExplicitScenarioAssumption,
    SyntheticFixture,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Provenance {
    pub origin: Origin,
    pub source: String,
    pub locator: String,
    pub dependencies: Vec<String>,
    /// Absolute bound in the field's SI unit, not a confidence interval; None is unquantified.
    pub uncertainty_absolute: Option<f64>,
}
impl Provenance {
    pub fn validate(&self) -> Result<()> {
        if self.source.trim().is_empty()
            || self.locator.trim().is_empty()
            || self.dependencies.iter().any(|s| s.trim().is_empty())
        {
            return Err(Error("missing source, locator or dependency identity"));
        }
        if let Some(bound) = self.uncertainty_absolute {
            nonnegative(bound)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Dimensionless,
    Kelvin,
    Pascal,
    KilogramPerMole,
    KilogramPerCubicMetre,
    JoulePerKilogramKelvin,
    JoulePerKilogram,
    JoulePerMoleKelvin,
    KilogramPerMetreSecond,
    WattPerMetreKelvin,
    KilogramPerSecond,
    WattPerKelvin,
    Watt,
}

/// SI-only input: preserving raw value/unit requires no lossy conversion.
#[derive(Clone, Debug, PartialEq)]
pub struct Supplied {
    pub raw_value: f64,
    pub raw_unit: Unit,
    pub provenance: Provenance,
}
impl Supplied {
    pub fn si(&self, unit: Unit) -> Result<f64> {
        self.provenance.validate()?;
        if self.raw_unit != unit {
            return Err(Error("physical input unit mismatch"));
        }
        finite(self.raw_value)
    }
    fn positive(&self, unit: Unit) -> Result<f64> {
        positive(self.si(unit)?)
    }
}

/// A supplier-declared constant-property approximation domain, not a learned bound.
#[derive(Clone, Debug, PartialEq)]
pub struct Domain {
    pub temperature_kelvin: [f64; 2],
    pub pressure_pascal: [f64; 2],
    pub provenance: Provenance,
}
impl Domain {
    pub fn validate(&self) -> Result<()> {
        self.provenance.validate()?;
        for [lo, hi] in [self.temperature_kelvin, self.pressure_pascal] {
            positive(lo)?;
            positive(hi)?;
            if lo > hi {
                return Err(Error("inverted property domain"));
            }
        }
        Ok(())
    }
    fn require(&self, t: f64, p: f64) -> Result<()> {
        self.validate()?;
        positive(t)?;
        positive(p)?;
        if t < self.temperature_kelvin[0]
            || t > self.temperature_kelvin[1]
            || p < self.pressure_pascal[0]
            || p > self.pressure_pascal[1]
        {
            return Err(Error("state outside declared property domain"));
        }
        Ok(())
    }
}

/// Literal mathematical tensor, supplied by the handwritten patch; no fitted parameters.
#[derive(Clone, Copy, Debug)]
pub struct Literals {
    pub pi: f64,
}
impl Literals {
    pub fn validate(self) -> Result<()> {
        if self.pi != std::f64::consts::PI {
            return Err(Error(
                "pi tensor is not the specified binary64 mathematical constant",
            ));
        }
        Ok(())
    }
}

/// Constant, ideal-solution liquid/vapor property pair. Identity is never interpreted as fuel.
#[derive(Clone, Debug)]
pub struct Component {
    pub identity: String,
    pub domain: Domain,
    pub molar_mass: Supplied,
    pub liquid_density: Supplied,
    pub liquid_cp: Supplied,
    pub vapor_cp: Supplied,
    pub latent_heat_at_reference: Supplied,
    pub saturation_pressure_at_reference: Supplied,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub components: Vec<Component>,
    /// Common sensible-enthalpy zero; latent heat contains the vapor-liquid offset.
    pub reference_temperature: Supplied,
    pub molar_gas_constant: Supplied,
    pub enthalpy_reference_identity: String,
}

#[derive(Clone, Debug)]
pub struct VectorInput {
    pub values: [f64; 3],
    pub frame: String,
    pub provenance: Provenance,
}
impl VectorInput {
    fn validate(&self) -> Result<()> {
        self.provenance.validate()?;
        if self.frame.trim().is_empty() {
            return Err(Error("missing vector frame"));
        }
        for x in self.values {
            finite(x)?;
        }
        Ok(())
    }
    fn is_zero(&self) -> bool {
        self.values.iter().all(|&x| x == 0.0)
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    pub temperature: Supplied,
    pub pressure: Supplied,
    /// All liquid-component keys must be present, including explicit zero entries.
    pub vapor_mass_fractions: BTreeMap<String, Supplied>,
    pub carrier_molar_mass: Supplied,
    pub gravity_m_per_s2: VectorInput,
    pub velocity_m_per_s: VectorInput,
    /// Signed externally supplied heat delivered to the liquid; not gas conduction again.
    pub external_heat: Supplied,
}

#[derive(Clone, Debug)]
pub struct State {
    pub masses_kg: BTreeMap<String, f64>,
    pub temperature_kelvin: f64,
    pub elapsed_seconds: f64,
    pub time_origin: String,
    /// Describes the original declared masses/T/time origin; retained in computed states.
    pub initial_state_provenance: Provenance,
}

#[derive(Clone, Debug)]
pub struct SuppliedTransferEnvelope {
    pub domain: Domain,
    pub radius_m: [f64; 2],
    pub liquid_mole_fraction_bounds: BTreeMap<String, [f64; 2]>,
    pub gravity_m_per_s2: [f64; 3],
    pub velocity_m_per_s: [f64; 3],
    pub frame: String,
    pub ambient_temperature_kelvin: f64,
    pub ambient_pressure_pascal: f64,
    pub ambient_vapor_mass_fractions: BTreeMap<String, f64>,
}

#[derive(Clone, Debug)]
pub enum Transport {
    /// Exact radial integration within constant-property / equal-diffusivity approximation.
    QuiescentSpherical {
        rho_diffusivity: Supplied,
        gas_conductivity: Supplied,
        domain: Domain,
    },
    /// Conditional effective-film law. Does not predict these conductances from gravity.
    SuppliedEffectiveFilm {
        mass_conductance: Supplied,
        thermal_conductance: Supplied,
        envelope: SuppliedTransferEnvelope,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct Numerics {
    /// Relative arithmetic comparison budget, required in [0,1); never used for clipping.
    pub relative: f64,
    pub mass_rate_absolute_kg_per_s: f64,
    pub mass_absolute_kg: f64,
    pub energy_absolute_joule: f64,
}
impl Numerics {
    fn validate(self) -> Result<()> {
        fraction(self.relative)?;
        if self.relative == 1.0 {
            return Err(Error("relative arithmetic budget must be less than one"));
        }
        nonnegative(self.mass_rate_absolute_kg_per_s)?;
        nonnegative(self.mass_absolute_kg)?;
        nonnegative(self.energy_absolute_joule)?;
        Ok(())
    }
    fn require(self, a: f64, b: f64, abs: f64) -> Result<()> {
        finite(a)?;
        finite(b)?;
        let bound = finite(abs + self.relative * a.abs().max(b.abs()))?;
        if finite(a - b)?.abs() > bound {
            Err(Error("conservation residual exceeds numerical budget"))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportMeaning {
    QuiescentSphericalReference,
    ConditionalSuppliedEffectiveFilm,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unavailable {
    NoReactionMechanism,
    NoOpticalModel,
    NoVisibilityCalibration,
}

#[derive(Clone, Debug)]
pub struct Evaluation {
    pub liquid_radius_m: f64,
    pub liquid_diameter_m: f64,
    pub liquid_mole_fractions: BTreeMap<String, f64>,
    pub surface_vapor_mass_fractions: BTreeMap<String, f64>,
    pub outward_component_rates_kg_per_s: BTreeMap<String, f64>,
    pub outward_total_rate_kg_per_s: f64,
    pub gas_heat_into_liquid_watt: f64,
    pub external_heat_watt: f64,
    pub outward_vapor_enthalpy_watt: f64,
    pub liquid_enthalpy_joule: f64,
    pub liquid_enthalpy_rate_watt: f64,
    pub temperature_rate_kelvin_per_s: f64,
    pub species_rate_residual_kg_per_s: f64,
    pub stefan_log_transfer: f64,
    pub thermal_peclet: f64,
    pub transport_meaning: TransportMeaning,
    pub chemical_state: Unavailable,
    pub luminous_diameter: Unavailable,
    pub visibility: Unavailable,
}

struct CheckedComponent<'a> {
    c: &'a Component,
    mass: f64,
    mw: f64,
    rho: f64,
    cl: f64,
    cv: f64,
    l0: f64,
    ps0: f64,
}

fn checked<'a>(
    material: &'a Material,
    state: &State,
    env: &Environment,
) -> Result<(Vec<CheckedComponent<'a>>, f64, f64, f64, f64)> {
    state.initial_state_provenance.validate()?;
    nonnegative(state.elapsed_seconds)?;
    if state.time_origin.trim().is_empty() || material.enthalpy_reference_identity.trim().is_empty()
    {
        return Err(Error("missing time origin or enthalpy reference identity"));
    }
    let t = positive(state.temperature_kelvin)?;
    let t0 = material.reference_temperature.positive(Unit::Kelvin)?;
    let rg = material
        .molar_gas_constant
        .positive(Unit::JoulePerMoleKelvin)?;
    let ta = env.temperature.positive(Unit::Kelvin)?;
    let p = env.pressure.positive(Unit::Pascal)?;
    env.carrier_molar_mass.positive(Unit::KilogramPerMole)?;
    env.external_heat.si(Unit::Watt)?;
    env.gravity_m_per_s2.validate()?;
    env.velocity_m_per_s.validate()?;
    if env.gravity_m_per_s2.frame != env.velocity_m_per_s.frame {
        return Err(Error("gravity and velocity frame mismatch"));
    }
    if material.components.is_empty()
        || material.components.len() != state.masses_kg.len()
        || material.components.len() != env.vapor_mass_fractions.len()
    {
        return Err(Error("component inventory mismatch or empty liquid"));
    }
    let mut components: Vec<_> = material.components.iter().collect();
    components.sort_by(|a, b| a.identity.cmp(&b.identity));
    if components
        .windows(2)
        .any(|pair| pair[0].identity == pair[1].identity)
    {
        return Err(Error("duplicate component identity"));
    }
    let mut out = Vec::with_capacity(components.len());
    for c in components {
        if c.identity.trim().is_empty() {
            return Err(Error("empty component identity"));
        }
        c.domain.require(t, p)?;
        c.domain.require(ta, p)?;
        c.domain.require(t0, p)?;
        let mass = nonnegative(
            *state
                .masses_kg
                .get(&c.identity)
                .ok_or(Error("missing component mass"))?,
        )?;
        fraction(
            env.vapor_mass_fractions
                .get(&c.identity)
                .ok_or(Error("missing ambient component"))?
                .si(Unit::Dimensionless)?,
        )?;
        let mw = c.molar_mass.positive(Unit::KilogramPerMole)?;
        let rho = c.liquid_density.positive(Unit::KilogramPerCubicMetre)?;
        let cl = c.liquid_cp.positive(Unit::JoulePerKilogramKelvin)?;
        let cv = c.vapor_cp.positive(Unit::JoulePerKilogramKelvin)?;
        if cv <= rg / mw {
            return Err(Error("ideal vapor requires cp greater than R/M"));
        }
        let l0 = c
            .latent_heat_at_reference
            .positive(Unit::JoulePerKilogram)?;
        let ps0 = c.saturation_pressure_at_reference.positive(Unit::Pascal)?;
        for bound_t in c.domain.temperature_kelvin {
            positive(l0 + (cv - cl) * (bound_t - t0))?;
        }
        out.push(CheckedComponent {
            c,
            mass,
            mw,
            rho,
            cl,
            cv,
            l0,
            ps0,
        });
    }
    positive(out.iter().map(|v| v.mass).sum())?;
    Ok((out, t0, rg, ta, p))
}

/// z/expm1(z) with its exact zero limit and overflow-resistant positive branch.
/// No tunable small-z cutoff or truncated fitted series is used.
fn bernoulli(z: f64) -> Result<f64> {
    finite(z)?;
    let b = if z == 0.0 {
        1.0
    } else if z > 0.0 {
        z * (-z).exp() / -(-z).exp_m1()
    } else {
        z / z.exp_m1()
    };
    // A positive result underflowing to zero is refused rather than silently losing transport.
    positive(b)
}

fn conductances(
    transport: &Transport,
    state: &State,
    env: &Environment,
    radius: f64,
    x: &BTreeMap<String, f64>,
    pi: f64,
) -> Result<(f64, f64, TransportMeaning)> {
    let p = env.pressure.si(Unit::Pascal)?;
    let ta = env.temperature.si(Unit::Kelvin)?;
    match transport {
        Transport::QuiescentSpherical {
            rho_diffusivity,
            gas_conductivity,
            domain,
        } => {
            if !env.gravity_m_per_s2.is_zero() || !env.velocity_m_per_s.is_zero() {
                return Err(Error(
                    "spherical reference requires exactly zero gravity and imposed velocity",
                ));
            }
            domain.require(state.temperature_kelvin, p)?;
            domain.require(ta, p)?;
            Ok((
                positive(
                    4.0 * pi * radius * rho_diffusivity.positive(Unit::KilogramPerMetreSecond)?,
                )?,
                positive(4.0 * pi * radius * gas_conductivity.positive(Unit::WattPerMetreKelvin)?)?,
                TransportMeaning::QuiescentSphericalReference,
            ))
        }
        Transport::SuppliedEffectiveFilm {
            mass_conductance,
            thermal_conductance,
            envelope: e,
        } => {
            e.domain.require(state.temperature_kelvin, p)?;
            e.domain.require(ta, p)?;
            positive(e.radius_m[0])?;
            positive(e.radius_m[1])?;
            if e.radius_m[0] > e.radius_m[1]
                || radius < e.radius_m[0]
                || radius > e.radius_m[1]
                || e.gravity_m_per_s2 != env.gravity_m_per_s2.values
                || e.velocity_m_per_s != env.velocity_m_per_s.values
                || e.frame != env.gravity_m_per_s2.frame
                || e.ambient_temperature_kelvin != ta
                || e.ambient_pressure_pascal != p
                || e.liquid_mole_fraction_bounds.len() != x.len()
                || e.ambient_vapor_mass_fractions.len() != x.len()
            {
                return Err(Error("state outside supplied transfer envelope"));
            }
            for (id, &xi) in x {
                let &[lo, hi] = e
                    .liquid_mole_fraction_bounds
                    .get(id)
                    .ok_or(Error("missing transfer composition bound"))?;
                fraction(lo)?;
                fraction(hi)?;
                if lo > hi || xi < lo || xi > hi {
                    return Err(Error("composition outside supplied transfer envelope"));
                }
                let yi = env.vapor_mass_fractions[id].si(Unit::Dimensionless)?;
                if e.ambient_vapor_mass_fractions.get(id) != Some(&yi) {
                    return Err(Error(
                        "ambient composition outside supplied transfer envelope",
                    ));
                }
            }
            Ok((
                mass_conductance.positive(Unit::KilogramPerSecond)?,
                thermal_conductance.positive(Unit::WattPerKelvin)?,
                TransportMeaning::ConditionalSuppliedEffectiveFilm,
            ))
        }
    }
}

pub fn evaluate(
    literals: Literals,
    material: &Material,
    state: &State,
    env: &Environment,
    transport: &Transport,
    numerics: Numerics,
) -> Result<Evaluation> {
    literals.validate()?;
    numerics.validate()?;
    let (cs, t0, rg, ta, p) = checked(material, state, env)?;
    let t = state.temperature_kelvin;
    let volume = positive(cs.iter().map(|c| c.mass / c.rho).sum())?;
    let radius = positive((3.0 * volume / (4.0 * literals.pi)).cbrt())?;
    let moles = positive(cs.iter().map(|c| c.mass / c.mw).sum())?;
    let mut x = BTreeMap::new();
    let mut surface_mole = BTreeMap::new();
    let mut surface_sum = 0.0;
    let mut surface_mw_sum = 0.0;
    for c in &cs {
        let xi = fraction((c.mass / c.mw) / moles)?;
        if c.mass > 0.0 && xi == 0.0 {
            return Err(Error("liquid mole fraction underflow"));
        }
        x.insert(c.c.identity.clone(), xi);
        let dcp = c.cv - c.cl;
        let exponent =
            finite(c.mw / rg * ((c.l0 - dcp * t0) * (1.0 / t0 - 1.0 / t) + dcp * (t / t0).ln()))?;
        let psat = positive(c.ps0 * exponent.exp())?;
        let xsi = nonnegative(xi * psat / p)?;
        if xi > 0.0 && xsi == 0.0 {
            return Err(Error("surface mole fraction underflow"));
        }
        surface_sum += xsi;
        surface_mw_sum += xsi * c.mw;
        surface_mole.insert(c.c.identity.clone(), xsi);
    }
    if finite(surface_sum)? >= 1.0 {
        return Err(Error(
            "carrier film unavailable at boiling or pure-vapor surface",
        ));
    }
    let mix_mw = positive(
        surface_mw_sum + (1.0 - surface_sum) * env.carrier_molar_mass.si(Unit::KilogramPerMole)?,
    )?;
    let mut ys = BTreeMap::new();
    let mut ss = 0.0;
    let mut si = 0.0;
    for c in &cs {
        let y = fraction(surface_mole[&c.c.identity] * c.mw / mix_mw)?;
        if surface_mole[&c.c.identity] > 0.0 && y == 0.0 {
            return Err(Error("surface mass fraction underflow"));
        }
        ys.insert(c.c.identity.clone(), y);
        ss += y;
        si += env.vapor_mass_fractions[&c.c.identity].si(Unit::Dimensionless)?;
    }
    if finite(ss)? >= 1.0 || finite(si)? >= 1.0 {
        return Err(Error("positive carrier mass fraction required"));
    }
    let z = finite((-si).ln_1p() - (-ss).ln_1p())?;
    let b = bernoulli(z)?;
    let (cm, ch, meaning) = conductances(transport, state, env, radius, &x, literals.pi)?;
    let mut rates = BTreeMap::new();
    let mut total = 0.0;
    let mut cadv = 0.0;
    let mut vapor_h = 0.0;
    let mut latent_power = 0.0;
    let mut capacity = 0.0;
    for c in &cs {
        let yi = env.vapor_mass_fractions[&c.c.identity].si(Unit::Dimensionless)?;
        let rate = finite(cm * (b * (ys[&c.c.identity] - yi) + z * ys[&c.c.identity]))?;
        if c.mass == 0.0 && rate > 0.0 {
            return Err(Error("absent liquid component has outward mass flux"));
        }
        total += rate;
        cadv += rate * c.cv;
        vapor_h += rate * (c.l0 + c.cv * (t - t0));
        latent_power += rate * (c.l0 + (c.cv - c.cl) * (t - t0));
        capacity += c.mass * c.cl;
        rates.insert(c.c.identity.clone(), rate);
    }
    numerics.require(total, finite(cm * z)?, numerics.mass_rate_absolute_kg_per_s)?;
    positive(capacity)?;
    let pe = finite(cadv / ch)?;
    let qg = finite(ch * (ta - t) * bernoulli(pe)?)?;
    let qe = env.external_heat.si(Unit::Watt)?;
    Ok(Evaluation {
        liquid_radius_m: radius,
        liquid_diameter_m: positive(2.0 * radius)?,
        liquid_mole_fractions: x,
        surface_vapor_mass_fractions: ys,
        outward_component_rates_kg_per_s: rates,
        outward_total_rate_kg_per_s: finite(total)?,
        gas_heat_into_liquid_watt: qg,
        external_heat_watt: qe,
        outward_vapor_enthalpy_watt: finite(vapor_h)?,
        liquid_enthalpy_joule: finite(capacity * (t - t0))?,
        liquid_enthalpy_rate_watt: finite(qg + qe - vapor_h)?,
        temperature_rate_kelvin_per_s: finite((qg + qe - latent_power) / capacity)?,
        species_rate_residual_kg_per_s: finite(total - cm * z)?,
        stefan_log_transfer: z,
        thermal_peclet: pe,
        transport_meaning: meaning,
        chemical_state: Unavailable::NoReactionMechanism,
        luminous_diameter: Unavailable::NoOpticalModel,
        visibility: Unavailable::NoVisibilityCalibration,
    })
}

#[derive(Clone, Debug)]
pub struct Step {
    pub state: State,
    pub start_flux: Evaluation,
    pub component_outflow_kg: BTreeMap<String, f64>,
    pub heat_into_liquid_joule: f64,
    pub vapor_enthalpy_outflow_joule: f64,
    pub energy_residual_joule: f64,
}

/// One caller-requested explicit Euler step, conservative in each mass and total enthalpy.
/// No automatic coefficient/timestep search. Boundary crossing returns an error, not clipping.
pub fn advance(
    literals: Literals,
    material: &Material,
    state: &State,
    env: &Environment,
    transport: &Transport,
    dt_seconds: f64,
    numerics: Numerics,
) -> Result<Step> {
    positive(dt_seconds)?;
    let start = evaluate(literals, material, state, env, transport, numerics)?;
    let (cs, t0, _, _, _) = checked(material, state, env)?;
    let mut next = state.clone();
    next.elapsed_seconds = finite(state.elapsed_seconds + dt_seconds)?;
    if next.elapsed_seconds <= state.elapsed_seconds {
        return Err(Error("time increment lost to rounding"));
    }
    let mut outgoing = BTreeMap::new();
    let mut capacity = 0.0;
    for c in &cs {
        let out = finite(dt_seconds * start.outward_component_rates_kg_per_s[&c.c.identity])?;
        let mass = finite(c.mass - out)?;
        if mass < 0.0 {
            return Err(Error("step crosses component depletion; reduce timestep"));
        }
        numerics.require(mass + out, c.mass, numerics.mass_absolute_kg)?;
        capacity += mass * c.cl;
        next.masses_kg.insert(c.c.identity.clone(), mass);
        outgoing.insert(c.c.identity.clone(), out);
    }
    if finite(capacity)? <= 0.0 {
        return Err(Error(
            "step reaches complete liquid depletion; use reference event or smaller step",
        ));
    }
    let heat = finite(dt_seconds * (start.gas_heat_into_liquid_watt + start.external_heat_watt))?;
    let vapor = finite(dt_seconds * start.outward_vapor_enthalpy_watt)?;
    let expected_h = finite(start.liquid_enthalpy_joule + heat - vapor)?;
    next.temperature_kelvin = positive(t0 + expected_h / capacity)?;
    let actual_h = finite(capacity * (next.temperature_kelvin - t0))?;
    numerics.require(actual_h, expected_h, numerics.energy_absolute_joule)?;
    // Check complete endpoint validity, including transfer envelope and phase boundary.
    evaluate(literals, material, &next, env, transport, numerics)?;
    Ok(Step {
        state: next,
        start_flux: start,
        component_outflow_kg: outgoing,
        heat_into_liquid_joule: heat,
        vapor_enthalpy_outflow_joule: vapor,
        energy_residual_joule: finite(actual_h - expected_h)?,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceDiameter {
    Liquid { diameter_m: f64 },
    Depleted { elapsed_seconds_to_depletion: f64 },
}
#[derive(Clone, Debug)]
pub struct IsothermalReference {
    pub diameter: ReferenceDiameter,
    /// Signed decay rate; negative means condensation and diameter growth.
    pub diameter_squared_decay_m2_per_s: f64,
    /// Additional heat required now beyond gas and supplied external heat.
    pub initial_thermostat_heat_watt: f64,
}

/// Pure-liquid, constant-condition, externally thermostatted analytic reference only.
pub fn isothermal_reference(
    literals: Literals,
    material: &Material,
    state: &State,
    env: &Environment,
    transport: &Transport,
    horizon_seconds: f64,
    numerics: Numerics,
) -> Result<IsothermalReference> {
    nonnegative(horizon_seconds)?;
    if material.components.len() != 1 {
        return Err(Error("analytic d-squared reference requires pure liquid"));
    }
    let Transport::QuiescentSpherical {
        rho_diffusivity, ..
    } = transport
    else {
        return Err(Error(
            "analytic d-squared reference requires spherical transport",
        ));
    };
    let ev = evaluate(literals, material, state, env, transport, numerics)?;
    let (cs, t0, _, _, _) = checked(material, state, env)?;
    let c = &cs[0];
    let k = finite(
        8.0 * rho_diffusivity.si(Unit::KilogramPerMetreSecond)? * ev.stefan_log_transfer / c.rho,
    )?;
    let d2 = positive(ev.liquid_diameter_m * ev.liquid_diameter_m)?;
    let diameter = exact_diameter(d2, k, horizon_seconds)?;
    let latent = positive(c.l0 + (c.cv - c.cl) * (state.temperature_kelvin - t0))?;
    let thermostat = finite(
        ev.outward_total_rate_kg_per_s * latent
            - ev.gas_heat_into_liquid_watt
            - ev.external_heat_watt,
    )?;
    Ok(IsothermalReference {
        diameter,
        diameter_squared_decay_m2_per_s: k,
        initial_thermostat_heat_watt: thermostat,
    })
}

fn exact_diameter(initial_d2: f64, decay: f64, horizon: f64) -> Result<ReferenceDiameter> {
    if decay > 0.0 {
        let depletion = positive(initial_d2 / decay)?;
        if horizon >= depletion {
            return Ok(ReferenceDiameter::Depleted {
                elapsed_seconds_to_depletion: depletion,
            });
        }
    }
    let d2 = positive(initial_d2 - finite(decay * horizon)?)?;
    Ok(ReferenceDiameter::Liquid {
        diameter_m: positive(d2.sqrt())?,
    })
}

/// All heat quantities use sensible-only gas enthalpy plus one separate reaction heat.
/// Pure fuel/no ambient fuel, existing infinite-rate sheet, Le=1 and no radiation are assertions.
#[derive(Clone, Debug)]
pub struct FlameSheetInputs {
    pub liquid_radius_m: f64,
    pub geometry_provenance: Provenance,
    pub surface_temperature: Supplied,
    pub ambient_temperature: Supplied,
    pub pressure: Supplied,
    pub liquid_density: Supplied,
    pub latent_heat_at_surface: Supplied,
    pub gas_cp: Supplied,
    pub gas_conductivity: Supplied,
    pub heat_of_reaction_per_kg_fuel: Supplied,
    pub oxidizer_kg_per_kg_fuel: Supplied,
    pub ambient_oxidizer_mass_fraction: Supplied,
    pub gravity_m_per_s2: VectorInput,
    pub velocity_m_per_s: VectorInput,
    pub property_domain: Domain,
    /// Records the pure-fuel, existing-sheet and ideal reference assumptions explicitly.
    pub reference_assumptions: Provenance,
}

#[derive(Clone, Debug)]
pub struct FlameSheetReference {
    pub liquid_diameter: ReferenceDiameter,
    pub diameter_squared_decay_m2_per_s: f64,
    pub initial_outward_fuel_rate_kg_per_s: f64,
    pub initial_reaction_sheet_radius_m: f64,
    pub reaction_sheet_temperature_kelvin: f64,
    pub luminous_diameter: Unavailable,
    /// A conditional reaction-sheet solution, not a finite-rate ignition/extinction prediction.
    pub conditional_existing_sheet: bool,
}

pub fn flame_sheet_reference(
    literals: Literals,
    input: &FlameSheetInputs,
    horizon_seconds: f64,
) -> Result<FlameSheetReference> {
    literals.validate()?;
    nonnegative(horizon_seconds)?;
    input.geometry_provenance.validate()?;
    input.reference_assumptions.validate()?;
    input.gravity_m_per_s2.validate()?;
    input.velocity_m_per_s.validate()?;
    if input.gravity_m_per_s2.frame != input.velocity_m_per_s.frame
        || !input.gravity_m_per_s2.is_zero()
        || !input.velocity_m_per_s.is_zero()
    {
        return Err(Error(
            "flame-sheet reference requires zero gravity and imposed velocity in one frame",
        ));
    }
    let r = positive(input.liquid_radius_m)?;
    let ts = input.surface_temperature.positive(Unit::Kelvin)?;
    let ta = input.ambient_temperature.positive(Unit::Kelvin)?;
    let p = input.pressure.positive(Unit::Pascal)?;
    input.property_domain.require(ts, p)?;
    input.property_domain.require(ta, p)?;
    let rho = input.liquid_density.positive(Unit::KilogramPerCubicMetre)?;
    let l = input
        .latent_heat_at_surface
        .positive(Unit::JoulePerKilogram)?;
    let cp = input.gas_cp.positive(Unit::JoulePerKilogramKelvin)?;
    let k = input.gas_conductivity.positive(Unit::WattPerMetreKelvin)?;
    let q = input
        .heat_of_reaction_per_kg_fuel
        .positive(Unit::JoulePerKilogram)?;
    let s = input
        .oxidizer_kg_per_kg_fuel
        .positive(Unit::Dimensionless)?;
    let yo = positive(fraction(
        input
            .ambient_oxidizer_mass_fraction
            .si(Unit::Dimensionless)?,
    )?)?;
    let bc = positive((cp * (ta - ts) + q * yo / s) / l)?;
    let log = positive(bc.ln_1p())?;
    let ratio = positive(log / positive((yo / s).ln_1p())?)?;
    let tf = positive(ta + (q - l - cp * (ta - ts)) / (cp * (1.0 + s / yo)))?;
    if ratio <= 1.0 || tf <= ts || tf <= ta {
        return Err(Error("no admissible exterior hot sheet in this reference"));
    }
    input.property_domain.require(tf, p)?;
    let decay = positive(8.0 * k * log / rho / cp)?;
    let diameter = exact_diameter(positive(4.0 * r * r)?, decay, horizon_seconds)?;
    Ok(FlameSheetReference {
        liquid_diameter: diameter,
        diameter_squared_decay_m2_per_s: decay,
        initial_outward_fuel_rate_kg_per_s: positive(4.0 * literals.pi * r * k / cp * log)?,
        initial_reaction_sheet_radius_m: positive(r * ratio)?,
        reaction_sheet_temperature_kelvin: tf,
        luminous_diameter: Unavailable::NoOpticalModel,
        conditional_existing_sheet: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Deliberately fictitious SI properties. No NASA material or dataset is represented.
    fn provenance() -> Provenance {
        Provenance {
            origin: Origin::SyntheticFixture,
            source: "independent conservation fixture".into(),
            locator: "property_droplet_w0::tests".into(),
            dependencies: vec![],
            uncertainty_absolute: None,
        }
    }
    fn supplied(value: f64, unit: Unit) -> Supplied {
        Supplied {
            raw_value: value,
            raw_unit: unit,
            provenance: provenance(),
        }
    }
    fn domain() -> Domain {
        Domain {
            temperature_kelvin: [1.0, 100.0],
            pressure_pascal: [1.0, 100.0],
            provenance: provenance(),
        }
    }
    fn vector(values: [f64; 3]) -> VectorInput {
        VectorInput {
            values,
            frame: "laboratory".into(),
            provenance: provenance(),
        }
    }
    fn component(id: &str) -> Component {
        Component {
            identity: id.into(),
            domain: domain(),
            molar_mass: supplied(1.0, Unit::KilogramPerMole),
            liquid_density: supplied(10.0, Unit::KilogramPerCubicMetre),
            liquid_cp: supplied(2.0, Unit::JoulePerKilogramKelvin),
            vapor_cp: supplied(2.0, Unit::JoulePerKilogramKelvin),
            latent_heat_at_reference: supplied(100.0, Unit::JoulePerKilogram),
            saturation_pressure_at_reference: supplied(2.0, Unit::Pascal),
        }
    }
    fn fixture() -> (Literals, Material, State, Environment, Transport, Numerics) {
        let literals = Literals {
            pi: 3.141592653589793,
        };
        let material = Material {
            components: vec![component("a")],
            reference_temperature: supplied(10.0, Unit::Kelvin),
            molar_gas_constant: supplied(1.0, Unit::JoulePerMoleKelvin),
            enthalpy_reference_identity: "synthetic sensible zero".into(),
        };
        let state = State {
            masses_kg: BTreeMap::from([("a".into(), 10.0)]),
            temperature_kelvin: 10.0,
            elapsed_seconds: 0.0,
            time_origin: "synthetic initial state".into(),
            initial_state_provenance: provenance(),
        };
        let env = Environment {
            temperature: supplied(10.0, Unit::Kelvin),
            pressure: supplied(10.0, Unit::Pascal),
            vapor_mass_fractions: BTreeMap::from([(
                "a".into(),
                supplied(0.0, Unit::Dimensionless),
            )]),
            carrier_molar_mass: supplied(1.0, Unit::KilogramPerMole),
            gravity_m_per_s2: vector([0.0; 3]),
            velocity_m_per_s: vector([0.0; 3]),
            external_heat: supplied(0.0, Unit::Watt),
        };
        let transport = Transport::QuiescentSpherical {
            rho_diffusivity: supplied(0.4, Unit::KilogramPerMetreSecond),
            gas_conductivity: supplied(3.0, Unit::WattPerMetreKelvin),
            domain: domain(),
        };
        let numerics = Numerics {
            relative: 1e-12,
            mass_rate_absolute_kg_per_s: 1e-12,
            mass_absolute_kg: 1e-12,
            energy_absolute_joule: 1e-10,
        };
        (literals, material, state, env, transport, numerics)
    }
    fn close(a: f64, b: f64) {
        assert!(
            (a - b).abs() <= 1e-10 * (1.0 + a.abs().max(b.abs())),
            "{a} != {b}"
        );
    }

    #[test]
    fn exact_equilibrium_and_zero_heat_remain_stationary() {
        let (l, m, s, mut e, tr, n) = fixture();
        e.vapor_mass_fractions.get_mut("a").unwrap().raw_value = 0.2;
        let v = evaluate(l, &m, &s, &e, &tr, n).unwrap();
        assert_eq!(v.outward_total_rate_kg_per_s, 0.0);
        assert_eq!(v.temperature_rate_kelvin_per_s, 0.0);
        assert_eq!(v.gas_heat_into_liquid_watt, 0.0);
        let step = advance(l, &m, &s, &e, &tr, 10.0, n).unwrap();
        assert_eq!(step.state.masses_kg, s.masses_kg);
        assert_eq!(step.state.temperature_kelvin, s.temperature_kelvin);
        assert_eq!(step.energy_residual_joule, 0.0);
        assert_eq!(v.luminous_diameter, Unavailable::NoOpticalModel);
    }

    #[test]
    fn condensation_is_signed_mass_gain_and_latent_heating() {
        let (l, m, s, mut e, tr, n) = fixture();
        e.vapor_mass_fractions.get_mut("a").unwrap().raw_value = 0.4;
        let v = evaluate(l, &m, &s, &e, &tr, n).unwrap();
        assert!(v.outward_total_rate_kg_per_s < 0.0);
        assert!(v.temperature_rate_kelvin_per_s > 0.0);
        let step = advance(l, &m, &s, &e, &tr, 0.001, n).unwrap();
        assert!(step.state.masses_kg["a"] > s.masses_kg["a"]);
        assert!(step.state.temperature_kelvin > s.temperature_kelvin);
        let analytic = isothermal_reference(l, &m, &s, &e, &tr, 1.0, n).unwrap();
        assert!(analytic.diameter_squared_decay_m2_per_s < 0.0);
        let ReferenceDiameter::Liquid { diameter_m } = analytic.diameter else {
            panic!("condensation cannot deplete liquid")
        };
        assert!(diameter_m > v.liquid_diameter_m);
    }

    #[test]
    fn counterdiffusion_at_zero_total_flux_conserves_components_and_enthalpy() {
        let (l, mut m, mut s, mut e, tr, n) = fixture();
        m.components.push(component("b"));
        m.components[1].latent_heat_at_reference.raw_value = 200.0;
        s.masses_kg.insert("a".into(), 5.0);
        s.masses_kg.insert("b".into(), 5.0);
        e.vapor_mass_fractions
            .insert("a".into(), supplied(0.15, Unit::Dimensionless));
        e.vapor_mass_fractions
            .insert("b".into(), supplied(0.05, Unit::Dimensionless));
        let v = evaluate(l, &m, &s, &e, &tr, n).unwrap();
        close(v.stefan_log_transfer, 0.0);
        close(v.outward_total_rate_kg_per_s, 0.0);
        assert!(v.outward_component_rates_kg_per_s["a"] < 0.0);
        assert!(v.outward_component_rates_kg_per_s["b"] > 0.0);
        assert!(v.temperature_rate_kelvin_per_s < 0.0);
        let step = advance(l, &m, &s, &e, &tr, 0.001, n).unwrap();
        for id in ["a", "b"] {
            close(
                step.state.masses_kg[id] + step.component_outflow_kg[id],
                s.masses_kg[id],
            );
        }
        let end = evaluate(l, &m, &step.state, &e, &tr, n).unwrap();
        close(
            end.liquid_enthalpy_joule + step.vapor_enthalpy_outflow_joule,
            v.liquid_enthalpy_joule + step.heat_into_liquid_joule,
        );
        close(step.state.masses_kg.values().sum(), 10.0);
    }

    #[test]
    fn vector_order_and_component_names_do_not_select_physics() {
        let (l, mut m, mut s, mut e, tr, n) = fixture();
        let mut b = component("b");
        b.molar_mass.raw_value = 2.0;
        b.liquid_density.raw_value = 20.0;
        m.components.push(b);
        s.masses_kg.insert("b".into(), 2.0);
        e.vapor_mass_fractions
            .insert("b".into(), supplied(0.01, Unit::Dimensionless));
        let before = evaluate(l, &m, &s, &e, &tr, n).unwrap();
        m.components.reverse();
        let reversed = evaluate(l, &m, &s, &e, &tr, n).unwrap();
        assert_eq!(
            before.outward_component_rates_kg_per_s,
            reversed.outward_component_rates_kg_per_s
        );
        assert_eq!(before.liquid_diameter_m, reversed.liquid_diameter_m);
        m.components
            .iter_mut()
            .find(|c| c.identity == "a")
            .unwrap()
            .identity = "unknown fuel name".into();
        let mass = s.masses_kg.remove("a").unwrap();
        s.masses_kg.insert("unknown fuel name".into(), mass);
        let ambient = e.vapor_mass_fractions.remove("a").unwrap();
        e.vapor_mass_fractions
            .insert("unknown fuel name".into(), ambient);
        let renamed = evaluate(l, &m, &s, &e, &tr, n).unwrap();
        close(
            before.outward_total_rate_kg_per_s,
            renamed.outward_total_rate_kg_per_s,
        );
        close(before.liquid_diameter_m, renamed.liquid_diameter_m);
    }

    #[test]
    fn conservative_steps_converge_to_independent_thermostatted_d_squared_limit() {
        let (l, m, s, mut e, tr, n) = fixture();
        let horizon = 0.2;
        let v = evaluate(l, &m, &s, &e, &tr, n).unwrap();
        // Independently integrate d(r^2)/dt=-2*rhoD*log(1/(1-Ys))/rhoL.
        let expected_r2 =
            v.liquid_radius_m.powi(2) - 2.0 * 0.4 * (1.0_f64 / 0.8).ln() / 10.0 * horizon;
        let exact_d = 2.0 * expected_r2.sqrt();
        let analytic = isothermal_reference(l, &m, &s, &e, &tr, horizon, n).unwrap();
        let ReferenceDiameter::Liquid { diameter_m } = analytic.diameter else {
            panic!("not depleted")
        };
        close(diameter_m, exact_d);
        let mut errors = vec![];
        for count in [10, 100] {
            let mut state = s.clone();
            for _ in 0..count {
                let flux = evaluate(l, &m, &state, &e, &tr, n).unwrap();
                e.external_heat.raw_value =
                    flux.outward_total_rate_kg_per_s * 100.0 - flux.gas_heat_into_liquid_watt;
                state = advance(l, &m, &state, &e, &tr, horizon / f64::from(count), n)
                    .unwrap()
                    .state;
                close(state.temperature_kelvin, 10.0);
            }
            let result = evaluate(l, &m, &state, &e, &tr, n).unwrap();
            errors.push((result.liquid_diameter_m - exact_d).abs());
        }
        assert!(
            errors[1] < errors[0] / 8.0,
            "first-order refinement: {errors:?}"
        );
        let depleted = isothermal_reference(l, &m, &s, &e, &tr, 100.0, n).unwrap();
        assert!(matches!(
            depleted.diameter,
            ReferenceDiameter::Depleted { .. }
        ));
    }

    #[test]
    fn sensible_enthalpy_and_latent_temperature_dependence_are_consistent() {
        let (l, mut m, mut s, mut e, tr, n) = fixture();
        m.components[0].liquid_cp.raw_value = 3.0;
        m.components[0].latent_heat_at_reference.raw_value = 200.0;
        s.temperature_kelvin = 11.0;
        e.temperature.raw_value = 11.0;
        // Keep the intentionally large synthetic latent-heat law below its boiling boundary.
        e.pressure.raw_value = 100.0;
        let v = evaluate(l, &m, &s, &e, &tr, n).unwrap();
        close(v.liquid_enthalpy_joule, 30.0);
        close(
            v.outward_vapor_enthalpy_watt,
            v.outward_total_rate_kg_per_s * 202.0,
        );
        close(
            v.temperature_rate_kelvin_per_s,
            -v.outward_total_rate_kg_per_s * 199.0 / 30.0,
        );
        let step = advance(l, &m, &s, &e, &tr, 0.0001, n).unwrap();
        let expected = v.liquid_enthalpy_joule - step.vapor_enthalpy_outflow_joule;
        close(
            step.state.masses_kg["a"] * 3.0 * (step.state.temperature_kelvin - 10.0),
            expected,
        );
    }

    #[test]
    fn gravity_requires_matching_supplied_conditional_closure() {
        let (l, m, s, mut e, tr, n) = fixture();
        e.gravity_m_per_s2.values = [0.0, 0.0, 1.0];
        assert!(evaluate(l, &m, &s, &e, &tr, n).is_err());
        let supplied_tr = Transport::SuppliedEffectiveFilm {
            mass_conductance: supplied(1.0, Unit::KilogramPerSecond),
            thermal_conductance: supplied(1.0, Unit::WattPerKelvin),
            envelope: SuppliedTransferEnvelope {
                domain: domain(),
                radius_m: [0.1, 2.0],
                liquid_mole_fraction_bounds: BTreeMap::from([("a".into(), [0.0, 1.0])]),
                gravity_m_per_s2: e.gravity_m_per_s2.values,
                velocity_m_per_s: [0.0; 3],
                frame: "laboratory".into(),
                ambient_temperature_kelvin: 10.0,
                ambient_pressure_pascal: 10.0,
                ambient_vapor_mass_fractions: BTreeMap::from([("a".into(), 0.0)]),
            },
        };
        let result = evaluate(l, &m, &s, &e, &supplied_tr, n).unwrap();
        assert_eq!(
            result.transport_meaning,
            TransportMeaning::ConditionalSuppliedEffectiveFilm
        );
        e.gravity_m_per_s2.values[2] = 2.0;
        assert!(evaluate(l, &m, &s, &e, &supplied_tr, n).is_err());
    }

    #[test]
    fn invalid_inputs_and_phase_crossings_fail_closed() {
        let (l, m, s, e, tr, n) = fixture();
        let mut bad = m.clone();
        bad.components[0].liquid_density.raw_unit = Unit::Pascal;
        assert!(evaluate(l, &bad, &s, &e, &tr, n).is_err());
        let mut bad = m.clone();
        bad.components[0].molar_mass.provenance.source.clear();
        assert!(evaluate(l, &bad, &s, &e, &tr, n).is_err());
        let mut bad = m.clone();
        bad.components[0].saturation_pressure_at_reference.raw_value = 10.0;
        assert!(evaluate(l, &bad, &s, &e, &tr, n).is_err());
        let mut bad = m.clone();
        bad.components[0].liquid_cp.raw_value = 10.0;
        assert!(evaluate(l, &bad, &s, &e, &tr, n).is_err());
        let mut bad = s.clone();
        bad.temperature_kelvin = f64::NAN;
        assert!(evaluate(l, &m, &bad, &e, &tr, n).is_err());
        let mut bad = s.clone();
        bad.masses_kg.insert("a".into(), -1.0);
        assert!(evaluate(l, &m, &bad, &e, &tr, n).is_err());
        let mut bad = e.clone();
        bad.vapor_mass_fractions.get_mut("a").unwrap().raw_value = 1.0;
        assert!(evaluate(l, &m, &s, &bad, &tr, n).is_err());
        assert!(advance(l, &m, &s, &e, &tr, 100.0, n).is_err());
        assert!(evaluate(Literals { pi: 3.0 }, &m, &s, &e, &tr, n).is_err());
    }

    #[test]
    fn absent_species_can_condense_but_no_fraction_is_normalized_or_clipped() {
        let (l, mut m, mut s, mut e, tr, n) = fixture();
        m.components.push(component("b"));
        s.masses_kg.insert("b".into(), 0.0);
        e.vapor_mass_fractions
            .insert("b".into(), supplied(0.1, Unit::Dimensionless));
        let v = evaluate(l, &m, &s, &e, &tr, n).unwrap();
        assert!(v.outward_component_rates_kg_per_s["b"] < 0.0);
        let next = advance(l, &m, &s, &e, &tr, 0.001, n).unwrap();
        assert!(next.state.masses_kg["b"] > 0.0);
        e.vapor_mass_fractions.get_mut("a").unwrap().raw_value = 0.95;
        assert!(evaluate(l, &m, &s, &e, &tr, n).is_err());
    }

    fn sheet() -> FlameSheetInputs {
        FlameSheetInputs {
            liquid_radius_m: 1.0,
            geometry_provenance: provenance(),
            surface_temperature: supplied(10.0, Unit::Kelvin),
            ambient_temperature: supplied(10.0, Unit::Kelvin),
            pressure: supplied(10.0, Unit::Pascal),
            liquid_density: supplied(10.0, Unit::KilogramPerCubicMetre),
            latent_heat_at_surface: supplied(100.0, Unit::JoulePerKilogram),
            gas_cp: supplied(2.0, Unit::JoulePerKilogramKelvin),
            gas_conductivity: supplied(3.0, Unit::WattPerMetreKelvin),
            heat_of_reaction_per_kg_fuel: supplied(400.0, Unit::JoulePerKilogram),
            oxidizer_kg_per_kg_fuel: supplied(1.0, Unit::Dimensionless),
            ambient_oxidizer_mass_fraction: supplied(0.5, Unit::Dimensionless),
            gravity_m_per_s2: vector([0.0; 3]),
            velocity_m_per_s: vector([0.0; 3]),
            property_domain: domain(),
            reference_assumptions: provenance(),
        }
    }

    #[test]
    fn flame_sheet_is_conditional_geometry_and_has_exact_radius_scaling() {
        let l = Literals {
            pi: 3.141592653589793,
        };
        let input = sheet();
        let a = flame_sheet_reference(l, &input, 0.0).unwrap();
        close(a.reaction_sheet_temperature_kelvin, 60.0);
        close(a.diameter_squared_decay_m2_per_s, 1.2 * 3.0_f64.ln());
        close(
            a.initial_reaction_sheet_radius_m,
            3.0_f64.ln() / 1.5_f64.ln(),
        );
        let mut larger = input.clone();
        larger.liquid_radius_m = 2.0;
        let b = flame_sheet_reference(l, &larger, 0.0).unwrap();
        close(
            b.initial_outward_fuel_rate_kg_per_s,
            2.0 * a.initial_outward_fuel_rate_kg_per_s,
        );
        close(
            b.initial_reaction_sheet_radius_m,
            2.0 * a.initial_reaction_sheet_radius_m,
        );
        close(
            b.diameter_squared_decay_m2_per_s,
            a.diameter_squared_decay_m2_per_s,
        );
        assert_eq!(a.luminous_diameter, Unavailable::NoOpticalModel);
        let mut missing = input.clone();
        missing.ambient_oxidizer_mass_fraction.raw_value = 0.0;
        assert!(flame_sheet_reference(l, &missing, 0.0).is_err());
        let mut too_cold = input.clone();
        too_cold.heat_of_reaction_per_kg_fuel.raw_value = 100.0;
        assert!(flame_sheet_reference(l, &too_cold, 0.0).is_err());
        let mut domain_failure = input;
        domain_failure.property_domain.temperature_kelvin[1] = 50.0;
        assert!(flame_sheet_reference(l, &domain_failure, 0.0).is_err());
    }
}

//! Clean-room Astra W0: property-driven, well-mixed liquid + spherical Stefan gas film.
//! Internal kernel, not a new wire contract. See docs/wsp/research/astra_droplet_cleanroom.md.
//! No property lookup by species name, empirical convection factor, kinetics, or optical proxy.

use std::collections::BTreeSet;

use anyhow::{ensure, Result};
use ignis_core::contracts::{LiteralOp, ModelSpec};
use ignis_core::registry::TensorMap;

use crate::assemble::{make_patch, materialize, PatchMeta};
use crate::physical_core::ElementalComposition;

const SPEC: &str =
    include_str!("../../../configs/models/droplets/property-w0/cleanroom-astra/spec.json");
const OPS: &str =
    include_str!("../../../configs/models/droplets/property-w0/cleanroom-astra/ops.json");
const META: &str =
    include_str!("../../../configs/models/droplets/property-w0/cleanroom-astra/meta.json");

/// Only the committed, validated literal patch supplies constants and numerical controls.
#[derive(Clone, Debug)]
pub struct Parameters {
    pi: f64,
    gas_constant: f64,
    step_s: f64,
    max_fraction_loss: f64,
    max_fraction_temperature_change: f64,
    relative_error: f64,
    absolute_mass_error_kg: f64,
    absolute_temperature_error_k: f64,
    conservation_relative_error: f64,
}

pub fn specification() -> Result<ModelSpec> {
    let spec: ModelSpec = serde_json::from_str(SPEC)?;
    spec.validate()?;
    Ok(spec)
}

pub fn authored_patch() -> Result<ignis_core::contracts::HandwrittenWeightPatch> {
    let spec = specification()?;
    let meta: PatchMeta = serde_json::from_str(META)?;
    let ops: Vec<LiteralOp> = serde_json::from_str(OPS)?;
    let patch = make_patch(&spec, meta, ops);
    patch.validate_against(&spec)?;
    Ok(patch)
}

impl Parameters {
    pub fn authored() -> Result<Self> {
        let spec = specification()?;
        let tensors = materialize(&spec, &authored_patch()?, None)?;
        Self::from_tensors(&spec, &tensors)
    }

    /// Bind a release's stored tensors to this exact forward specification.
    /// The caller additionally checks the release construction manifest/spec coherence.
    pub fn from_tensors(spec: &ModelSpec, tensors: &TensorMap) -> Result<Self> {
        spec.validate()?;
        ensure!(
            spec.spec_hash() == specification()?.spec_hash(),
            "specification does not identify this forward"
        );
        ensure!(
            tensors.len() == spec.tensors.len(),
            "stored tensor inventory mismatch"
        );
        for tensor in &spec.tensors {
            let stored = tensors
                .get(&tensor.name)
                .ok_or_else(|| anyhow::anyhow!("missing tensor {}", tensor.name))?;
            ensure!(
                stored
                    .shape
                    .iter()
                    .copied()
                    .eq(tensor.shape.iter().map(|&n| n as usize))
                    && stored.data.len() == tensor.numel(),
                "stored tensor shape mismatch: {}",
                tensor.name
            );
            ensure!(
                stored.data.iter().all(|v| v.is_finite()),
                "nonfinite stored tensor"
            );
            if let Some(bounds) = &tensor.bounds {
                ensure!(
                    stored
                        .data
                        .iter()
                        .all(|&v| v >= bounds.min.get() && v <= bounds.max.get()),
                    "stored tensor outside specification bounds: {}",
                    tensor.name
                );
            }
        }
        let get = |name: &str| tensors[name].data[0];
        let parameters = Self {
            pi: get("pi"),
            gas_constant: get("molar_gas_constant"),
            step_s: get("step_seconds"),
            max_fraction_loss: get("max_fraction_mass_loss"),
            max_fraction_temperature_change: get("max_fraction_temperature_change"),
            relative_error: get("relative_local_error"),
            absolute_mass_error_kg: get("absolute_mass_error_kg"),
            absolute_temperature_error_k: get("absolute_temperature_error_kelvin"),
            conservation_relative_error: get("conservation_relative_error"),
        };
        for v in [
            parameters.pi,
            parameters.gas_constant,
            parameters.step_s,
            parameters.absolute_mass_error_kg,
            parameters.absolute_temperature_error_k,
        ] {
            ensure!(v > 0.0, "positive physical/numerical tensor required");
        }
        for v in [
            parameters.max_fraction_loss,
            parameters.max_fraction_temperature_change,
            parameters.relative_error,
            parameters.conservation_relative_error,
        ] {
            ensure!(
                v > 0.0 && v < 1.0,
                "numerical fractional controls must be strictly between zero and one"
            );
        }
        Ok(parameters)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Liquid,
    Gas,
    Interface,
    Condition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Kelvin,
    Pascal,
    Kilogram,
    KilogramPerMole,
    KilogramPerCubicMetre,
    JoulePerKilogramKelvin,
    JoulePerKilogram,
    SquareMetrePerSecond,
    WattPerMetreKelvin,
    PascalSecond,
    MetrePerSecond,
    MetrePerSecondSquared,
    One,
}

impl Unit {
    fn symbol(self) -> &'static str {
        match self {
            Self::Kelvin => "K",
            Self::Pascal => "Pa",
            Self::Kilogram => "kg",
            Self::KilogramPerMole => "kg/mol",
            Self::KilogramPerCubicMetre => "kg/m3",
            Self::JoulePerKilogramKelvin => "J/(kg K)",
            Self::JoulePerKilogram => "J/kg",
            Self::SquareMetrePerSecond => "m2/s",
            Self::WattPerMetreKelvin => "W/(m K)",
            Self::PascalSecond => "Pa s",
            Self::MetrePerSecond => "m/s",
            Self::MetrePerSecondSquared => "m/s2",
            Self::One => "1",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    ExternalDetermination,
    ExplicitScenarioAssumption,
    SyntheticFixture,
}

#[derive(Clone, Debug)]
pub enum Uncertainty {
    Unquantified,
    AbsoluteBoundInSi(f64),
}

/// A supplied constant-domain approximation, not an undocumented global constant.
/// This kernel accepts raw SI only: upstream conversion must retain its own artifact/provenance.
#[derive(Clone, Debug)]
pub struct Supplied {
    pub raw_value: f64,
    pub raw_unit: String,
    pub si_value: f64,
    pub si_unit: Unit,
    pub phase: Phase,
    pub source: String,
    pub locator: String,
    pub origin: Origin,
    pub uncertainty: Uncertainty,
    pub reference_temperature_k: f64,
    pub reference_pressure_pa: f64,
    pub reference_identity: String,
    pub temperature_domain_k: [f64; 2],
    pub pressure_domain_pa: [f64; 2],
}

impl Supplied {
    fn value(&self, unit: Unit, phase: Phase, t: f64, p: f64) -> Result<f64> {
        ensure!(
            self.si_value.is_finite() && self.raw_value.is_finite(),
            "nonfinite property"
        );
        ensure!(
            self.si_unit == unit && self.raw_unit == unit.symbol(),
            "property unit mismatch; raw SI required"
        );
        ensure!(self.raw_value == self.si_value, "raw/SI mismatch");
        ensure!(self.phase == phase, "property phase mismatch");
        ensure!(
            !self.source.trim().is_empty()
                && !self.locator.trim().is_empty()
                && !self.reference_identity.trim().is_empty(),
            "missing property provenance/reference"
        );
        ensure!(
            self.reference_temperature_k.is_finite()
                && self.reference_temperature_k > 0.0
                && self.reference_pressure_pa.is_finite()
                && self.reference_pressure_pa > 0.0,
            "invalid reference state"
        );
        for range in [self.temperature_domain_k, self.pressure_domain_pa] {
            ensure!(
                range[0].is_finite()
                    && range[1].is_finite()
                    && range[0] > 0.0
                    && range[0] <= range[1],
                "invalid property domain"
            );
        }
        ensure!(
            t.is_finite()
                && p.is_finite()
                && t >= self.temperature_domain_k[0]
                && t <= self.temperature_domain_k[1]
                && p >= self.pressure_domain_pa[0]
                && p <= self.pressure_domain_pa[1],
            "outside supplied property domain"
        );
        if let Uncertainty::AbsoluteBoundInSi(bound) = self.uncertainty {
            ensure!(
                bound.is_finite() && bound >= 0.0,
                "invalid uncertainty bound"
            );
        }
        Ok(self.si_value)
    }

    fn positive(&self, unit: Unit, phase: Phase, t: f64, p: f64) -> Result<f64> {
        let v = self.value(unit, phase, t, p)?;
        ensure!(v > 0.0, "nonpositive property");
        Ok(v)
    }
}

#[derive(Clone, Debug)]
pub struct LiquidComponent {
    /// Identity is used for inventory checks only, never as a numerical feature.
    pub identity: String,
    pub atoms: ElementalComposition,
    pub initial_mass: Supplied,
    pub molar_mass: Supplied,
    pub liquid_density: Supplied,
    pub liquid_heat_capacity: Supplied,
    /// Latent heat at the declared shared reference temperature.
    pub latent_heat_at_reference: Supplied,
    pub saturation_pressure_at_reference: Supplied,
    /// Complete combustion to CO2/H2O(v)/N2/SO2, per kg vapor; sensible convention.
    pub complete_combustion_heat: Option<Supplied>,
}

#[derive(Clone, Debug)]
pub enum ActivityClosure {
    /// Raoult gamma=1, additive liquid volume and enthalpy; explicitly selected, never default.
    IdealSolution {
        source: String,
        applicability: String,
    },
    Unsupported {
        reason: String,
    },
}

#[derive(Clone, Debug)]
pub enum Chemistry {
    EvaporationOnly,
    /// Already-established infinitely-fast sheet. Not an ignition/extinction model.
    EstablishedReactionSheet {
        source: String,
        applicability: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportApplicability {
    RequireQuiescentZeroGravity,
    /// Computes the spherical limiting model, with no correction; all outputs labelled extrapolated.
    ExplicitSphericalExtrapolation,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub time_origin: String,
    pub components: Vec<LiquidComponent>,
    pub initial_temperature: Supplied,
    pub ambient_temperature: Supplied,
    pub pressure: Supplied,
    /// Constant representative film values, valid throughout the complete modeled temperature span.
    pub gas_density: Supplied,
    pub gas_heat_capacity: Supplied,
    pub gas_conductivity: Supplied,
    /// One mass diffusivity for every fuel and oxidizer (explicit equal-D closure).
    pub common_diffusivity: Supplied,
    pub gas_viscosity: Supplied,
    /// Nonfuel surface background molar mass. Supplied closure, not inferred from ambient O2 alone.
    pub surface_background_molar_mass: Supplied,
    pub oxygen_molar_mass: Supplied,
    pub ambient_oxygen_mass_fraction: Supplied,
    pub relative_velocity: [Supplied; 3],
    pub gravity: [Supplied; 3],
    pub coordinate_frame: String,
    pub activity: ActivityClosure,
    pub chemistry: Chemistry,
    pub transport: TransportApplicability,
}

#[derive(Clone, Debug)]
pub struct State {
    pub seconds_since_origin: f64,
    pub component_masses_kg: Vec<f64>,
    pub liquid_sensible_enthalpy_j: f64,
    pub cumulative_evaporated_kg: f64,
    pub cumulative_heat_into_liquid_j: f64,
    pub cumulative_vapor_enthalpy_out_j: f64,
    pub cumulative_complete_combustion_heat_j: Option<f64>,
    pub cumulative_oxygen_consumed_kg: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Instantaneous {
    pub liquid_volume_equivalent_diameter_m: f64,
    pub liquid_temperature_k: f64,
    pub liquid_mole_fractions: Vec<f64>,
    pub interface_fuel_mass_fractions: Vec<f64>,
    pub evaporation_rates_kg_s: Vec<f64>,
    pub heat_into_liquid_w: f64,
    pub vapor_enthalpy_out_w: f64,
    pub liquid_enthalpy_rate_w: f64,
    pub reaction_sheet_diameter_m: Option<f64>,
    pub reaction_sheet_temperature_k: Option<f64>,
    pub complete_combustion_heat_release_w: Option<f64>,
    pub oxygen_consumption_kg_s: Option<f64>,
    pub reynolds: f64,
    pub peclet: f64,
    /// g*d^3/nu^2: a gravity/viscosity scale, NOT Grashof (no assumed density contrast).
    pub gravity_viscosity_scale: f64,
    pub extrapolated_spherical_transport: bool,
    pub visible_flame_diameter_m: Option<f64>,
    pub uv_flame_diameter_m: Option<f64>,
    pub finite_rate_extinction_seconds: Option<f64>,
    pub unsupported: Vec<&'static str>,
}

#[derive(Clone, Debug)]
pub struct Trajectory {
    pub states: Vec<State>,
    pub mass_balance_residual_kg: f64,
    pub energy_balance_residual_j: f64,
    pub maximum_step_doubling_error_ratio: f64,
}

fn shared_reference(input: &Input) -> Result<f64> {
    ensure!(!input.components.is_empty(), "empty liquid inventory");
    let first = &input.components[0].liquid_heat_capacity;
    for component in &input.components {
        for property in [
            &component.liquid_heat_capacity,
            &component.latent_heat_at_reference,
            &component.saturation_pressure_at_reference,
        ] {
            ensure!(
                property.reference_temperature_k == first.reference_temperature_k
                    && property.reference_pressure_pa == first.reference_pressure_pa
                    && property.reference_identity == first.reference_identity,
                "incompatible liquid thermodynamic references"
            );
        }
    }
    ensure!(
        input.gas_heat_capacity.reference_temperature_k == first.reference_temperature_k
            && input.gas_heat_capacity.reference_identity == first.reference_identity,
        "incompatible gas sensible reference"
    );
    Ok(first.reference_temperature_k)
}

fn pressure_temperature(input: &Input) -> Result<(f64, f64)> {
    let p = input.pressure.si_value;
    let t = input.initial_temperature.si_value;
    input
        .pressure
        .positive(Unit::Pascal, Phase::Condition, t, p)?;
    input
        .initial_temperature
        .positive(Unit::Kelvin, Phase::Condition, t, p)?;
    Ok((p, t))
}

pub fn initial_state(input: &Input) -> Result<State> {
    ensure!(
        !input.time_origin.trim().is_empty() && !input.coordinate_frame.trim().is_empty(),
        "missing time origin/frame"
    );
    let tref = shared_reference(input)?;
    let (p, t) = pressure_temperature(input)?;
    let mut seen = BTreeSet::new();
    let mut masses = Vec::new();
    let mut capacity = 0.0;
    for c in &input.components {
        ensure!(
            !c.identity.trim().is_empty() && seen.insert(&c.identity),
            "duplicate/empty species identity"
        );
        c.atoms.oxygen_demand_moles_per_mole()?;
        let mass = c
            .initial_mass
            .positive(Unit::Kilogram, Phase::Liquid, t, p)?;
        capacity += mass
            * c.liquid_heat_capacity
                .positive(Unit::JoulePerKilogramKelvin, Phase::Liquid, t, p)?;
        masses.push(mass);
    }
    Ok(State {
        seconds_since_origin: 0.0,
        component_masses_kg: masses,
        liquid_sensible_enthalpy_j: capacity * (t - tref),
        cumulative_evaporated_kg: 0.0,
        cumulative_heat_into_liquid_j: 0.0,
        cumulative_vapor_enthalpy_out_j: 0.0,
        cumulative_complete_combustion_heat_j: matches!(
            input.chemistry,
            Chemistry::EstablishedReactionSheet { .. }
        )
        .then_some(0.0),
        cumulative_oxygen_consumed_kg: matches!(
            input.chemistry,
            Chemistry::EstablishedReactionSheet { .. }
        )
        .then_some(0.0),
    })
}

fn finite_nonnegative(v: f64) -> Result<f64> {
    ensure!(
        v.is_finite() && v >= 0.0,
        "nonfinite/negative computed quantity"
    );
    Ok(v)
}

/// The full gas/liquid closure at a state; no iterative parameter inference or nonlinear solve.
pub fn instantaneous(
    input: &Input,
    state: &State,
    parameters: &Parameters,
) -> Result<Instantaneous> {
    let (pressure, initial_t) = pressure_temperature(input)?;
    let tref = shared_reference(input)?;
    ensure!(
        state.component_masses_kg.len() == input.components.len(),
        "inventory shape mismatch"
    );
    ensure!(
        state.seconds_since_origin.is_finite() && state.seconds_since_origin >= 0.0,
        "invalid state time"
    );
    ensure!(
        state.liquid_sensible_enthalpy_j.is_finite(),
        "nonfinite liquid enthalpy"
    );
    let mut species_identities = BTreeSet::new();
    for component in &input.components {
        ensure!(
            !component.identity.trim().is_empty() && species_identities.insert(&component.identity),
            "duplicate/empty species identity"
        );
    }
    for value in [
        state.cumulative_evaporated_kg,
        state.cumulative_heat_into_liquid_j,
        state.cumulative_vapor_enthalpy_out_j,
    ] {
        ensure!(value.is_finite(), "nonfinite cumulative budget");
    }
    finite_nonnegative(state.cumulative_evaporated_kg)?;
    let has_reaction = matches!(input.chemistry, Chemistry::EstablishedReactionSheet { .. });
    for value in [
        state.cumulative_complete_combustion_heat_j,
        state.cumulative_oxygen_consumed_kg,
    ] {
        ensure!(
            value.is_some() == has_reaction,
            "cumulative chemistry status mismatch"
        );
        if let Some(value) = value {
            finite_nonnegative(value)?;
        }
    }
    match &input.activity {
        ActivityClosure::IdealSolution {
            source,
            applicability,
        } => ensure!(
            !source.trim().is_empty() && !applicability.trim().is_empty(),
            "ideal activity closure must be justified"
        ),
        ActivityClosure::Unsupported { .. } => anyhow::bail!(
            "nonideal activity closure unsupported; supply independently justified implementation"
        ),
    }
    let mut capacity = 0.0;
    let mut volume = 0.0;
    let mut moles = Vec::new();
    for (c, &m) in input.components.iter().zip(&state.component_masses_kg) {
        ensure!(
            m.is_finite() && m > 0.0,
            "component depleted or nonpositive; stop before closure singularity"
        );
        capacity += m * c.liquid_heat_capacity.positive(
            Unit::JoulePerKilogramKelvin,
            Phase::Liquid,
            initial_t,
            pressure,
        )?;
        volume += m / c.liquid_density.positive(
            Unit::KilogramPerCubicMetre,
            Phase::Liquid,
            initial_t,
            pressure,
        )?;
        moles.push(
            m / c
                .molar_mass
                .positive(Unit::KilogramPerMole, Phase::Liquid, initial_t, pressure)?,
        );
    }
    ensure!(
        capacity.is_finite() && capacity > 0.0 && volume.is_finite() && volume > 0.0,
        "invalid inventory capacity/volume"
    );
    let t = tref + state.liquid_sensible_enthalpy_j / capacity;
    ensure!(t.is_finite() && t > 0.0, "invalid liquid temperature");
    let ambient_t =
        input
            .ambient_temperature
            .positive(Unit::Kelvin, Phase::Condition, initial_t, pressure)?;
    let gas = |prop: &Supplied, unit: Unit| -> Result<f64> {
        prop.positive(unit, Phase::Gas, t, pressure)?;
        prop.positive(unit, Phase::Gas, ambient_t, pressure)
    };
    let rho = gas(&input.gas_density, Unit::KilogramPerCubicMetre)?;
    let cp = gas(&input.gas_heat_capacity, Unit::JoulePerKilogramKelvin)?;
    let k = gas(&input.gas_conductivity, Unit::WattPerMetreKelvin)?;
    let diffusivity = gas(&input.common_diffusivity, Unit::SquareMetrePerSecond)?;
    let viscosity = gas(&input.gas_viscosity, Unit::PascalSecond)?;
    let background_m = gas(&input.surface_background_molar_mass, Unit::KilogramPerMole)?;
    let oxygen_m = gas(&input.oxygen_molar_mass, Unit::KilogramPerMole)?;
    let oxygen =
        input
            .ambient_oxygen_mass_fraction
            .value(Unit::One, Phase::Gas, ambient_t, pressure)?;
    ensure!(
        (0.0..=1.0).contains(&oxygen),
        "invalid oxygen mass fraction"
    );
    let radius = (3.0 * volume / (4.0 * parameters.pi)).cbrt();
    let diameter = 2.0 * radius;
    let total_moles: f64 = moles.iter().sum();
    let x: Vec<f64> = moles.iter().map(|n| n / total_moles).collect();
    let mut vapor_mole = Vec::new();
    let mut molecular = Vec::new();
    let mut sensible_vapor = Vec::new();
    for (c, &xi) in input.components.iter().zip(&x) {
        let mass = c
            .molar_mass
            .positive(Unit::KilogramPerMole, Phase::Liquid, t, pressure)?;
        c.liquid_density
            .positive(Unit::KilogramPerCubicMetre, Phase::Liquid, t, pressure)?;
        let cl = c.liquid_heat_capacity.positive(
            Unit::JoulePerKilogramKelvin,
            Phase::Liquid,
            t,
            pressure,
        )?;
        let lr = c.latent_heat_at_reference.positive(
            Unit::JoulePerKilogram,
            Phase::Interface,
            t,
            pressure,
        )?;
        let pr = c.saturation_pressure_at_reference.positive(
            Unit::Pascal,
            Phase::Interface,
            t,
            pressure,
        )?;
        let delta_cp = cp - cl;
        let lt = lr + delta_cp * (t - tref);
        ensure!(
            lt.is_finite() && lt > 0.0,
            "nonpositive latent heat; property closure out of domain"
        );
        // Integrated Clausius-Clapeyron with Kirchhoff constant delta_cp, not a fitted Antoine law.
        let log_pressure_ratio = mass / parameters.gas_constant
            * ((lr - delta_cp * tref) * (1.0 / tref - 1.0 / t) + delta_cp * (t / tref).ln());
        let saturation = pr * log_pressure_ratio.exp();
        ensure!(
            saturation.is_finite() && saturation > 0.0,
            "vapor pressure overflow/underflow"
        );
        vapor_mole.push(xi * saturation / pressure);
        molecular.push(mass);
        sensible_vapor.push(lr + cp * (t - tref));
    }
    let vapor_total: f64 = vapor_mole.iter().sum();
    ensure!(
        vapor_total.is_finite() && vapor_total > 0.0 && vapor_total < 1.0,
        "interface at/above bubble boundary or no resolvable vapor; boiling closure unsupported"
    );
    let denom = (1.0 - vapor_total) * background_m
        + vapor_mole
            .iter()
            .zip(&molecular)
            .map(|(y, m)| y * m)
            .sum::<f64>();
    let ys: Vec<f64> = vapor_mole
        .iter()
        .zip(&molecular)
        .map(|(y, m)| y * m / denom)
        .collect();
    let ys_total: f64 = ys.iter().sum();
    ensure!(
        ys_total > 0.0 && ys_total < 1.0,
        "invalid interface mass fractions"
    );
    let fractions: Vec<f64> = ys.iter().map(|y| y / ys_total).collect();
    let burning = matches!(input.chemistry, Chemistry::EstablishedReactionSheet { .. });
    let mut stoich = 0.0;
    let mut heat = 0.0;
    if let Chemistry::EstablishedReactionSheet {
        source,
        applicability,
    } = &input.chemistry
    {
        ensure!(
            !source.trim().is_empty() && !applicability.trim().is_empty() && oxygen > 0.0,
            "reaction sheet requires justification and oxidizer"
        );
        for ((c, &fraction), &mass) in input.components.iter().zip(&fractions).zip(&molecular) {
            let demand = c.atoms.oxygen_demand_moles_per_mole()?.get();
            ensure!(
                demand > 0.0,
                "reaction-sheet liquid components must all be fuels with positive oxygen demand"
            );
            stoich += fraction * demand * oxygen_m / mass;
            let q = c
                .complete_combustion_heat
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("missing vapor combustion enthalpy"))?;
            ensure!(
                q.reference_identity == input.gas_heat_capacity.reference_identity
                    && q.reference_temperature_k == tref,
                "incompatible combustion/sensible enthalpy convention"
            );
            heat += fraction * q.positive(Unit::JoulePerKilogram, Phase::Gas, t, pressure)?;
        }
        ensure!(
            stoich.is_finite() && stoich > 0.0 && heat.is_finite() && heat > 0.0,
            "nonfinite mixture combustion closure"
        );
    }
    let oxygen_ratio = if burning { oxygen / stoich } else { 0.0 };
    let log_transfer = ((ys_total + oxygen_ratio) / (1.0 - ys_total)).ln_1p();
    let rate = 4.0 * parameters.pi * radius * rho * diffusivity * log_transfer;
    let rates: Vec<f64> = fractions.iter().map(|f| f * rate).collect();
    // Gas sensible-energy ODE integrated separately inside/outside the reaction sheet.
    let a = rate * cp / (4.0 * parameters.pi * radius * k);
    let sheet_radius = if burning {
        Some(radius * log_transfer / oxygen_ratio.ln_1p())
    } else {
        None
    };
    if let Some(rf) = sheet_radius {
        ensure!(
            rf.is_finite() && rf > radius,
            "unresolved reaction-sheet geometry"
        );
        finite_nonnegative(rate * heat)?;
        finite_nonnegative(rate * stoich)?;
    }
    let sheet_a = sheet_radius.map_or(0.0, |rf| a * radius / rf);
    let heat_in = rate * (cp * (ambient_t - t) + heat * sheet_a.exp_m1()) / a.exp_m1();
    let vapor_out: f64 = rates.iter().zip(&sensible_vapor).map(|(m, h)| m * h).sum();
    let sheet_temperature = sheet_radius.map(|rf| {
        let inner_energy_per_capacity = t - heat_in / (rate * cp);
        inner_energy_per_capacity
            + (t - inner_energy_per_capacity) * (a * (1.0 - radius / rf)).exp()
    });
    if let Some(tf) = sheet_temperature {
        ensure!(
            tf.is_finite() && tf > 0.0,
            "invalid reaction-sheet temperature"
        );
        for (property, unit) in [
            (&input.gas_density, Unit::KilogramPerCubicMetre),
            (&input.gas_heat_capacity, Unit::JoulePerKilogramKelvin),
            (&input.gas_conductivity, Unit::WattPerMetreKelvin),
            (&input.common_diffusivity, Unit::SquareMetrePerSecond),
            (&input.gas_viscosity, Unit::PascalSecond),
            (&input.surface_background_molar_mass, Unit::KilogramPerMole),
            (&input.oxygen_molar_mass, Unit::KilogramPerMole),
        ] {
            property.positive(unit, Phase::Gas, tf, pressure)?;
        }
        for component in &input.components {
            component
                .complete_combustion_heat
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("missing vapor combustion enthalpy"))?
                .positive(Unit::JoulePerKilogram, Phase::Gas, tf, pressure)?;
        }
    }
    let norm = |values: &[Supplied; 3], unit| -> Result<f64> {
        let mut square = 0.0;
        for v in values {
            square += v
                .value(unit, Phase::Condition, initial_t, pressure)?
                .powi(2);
        }
        finite_nonnegative(square.sqrt())
    };
    let speed = norm(&input.relative_velocity, Unit::MetrePerSecond)?;
    let gravity = norm(&input.gravity, Unit::MetrePerSecondSquared)?;
    let extrapolated = speed != 0.0 || gravity != 0.0;
    ensure!(
        !extrapolated || input.transport == TransportApplicability::ExplicitSphericalExtrapolation,
        "nonzero motion/gravity violates spherical transport; explicit extrapolation required"
    );
    for v in [rate, diameter, log_transfer, a] {
        finite_nonnegative(v)?;
    }
    ensure!(
        heat_in.is_finite() && vapor_out.is_finite(),
        "nonfinite energy transfer"
    );
    Ok(Instantaneous {
        liquid_volume_equivalent_diameter_m: diameter,
        liquid_temperature_k: t,
        liquid_mole_fractions: x,
        interface_fuel_mass_fractions: ys,
        evaporation_rates_kg_s: rates,
        heat_into_liquid_w: heat_in,
        vapor_enthalpy_out_w: vapor_out,
        liquid_enthalpy_rate_w: heat_in - vapor_out,
        reaction_sheet_diameter_m: sheet_radius.map(|r| 2.0 * r),
        reaction_sheet_temperature_k: sheet_temperature,
        complete_combustion_heat_release_w: burning.then_some(rate * heat),
        oxygen_consumption_kg_s: burning.then_some(rate * stoich),
        reynolds: finite_nonnegative(rho * speed * diameter / viscosity)?,
        peclet: finite_nonnegative(speed * diameter / diffusivity)?,
        gravity_viscosity_scale: finite_nonnegative(gravity * diameter.powi(3) / (viscosity / rho).powi(2))?,
        extrapolated_spherical_transport: extrapolated,
        visible_flame_diameter_m: None,
        uv_flame_diameter_m: None,
        finite_rate_extinction_seconds: None,
        unsupported: vec!["visible/UV emission: no radiative transfer or excited-state/soot chemistry", "finite-rate ignition, extinction and cool flames: no kinetic mechanism", "radiation losses, support conduction, internal gradients, differential diffusion, boiling, nonideal activity, chamber oxygen depletion and spray interactions", "gravity/imposed-flow transport and trajectory dynamics: spherical limit only"],
    })
}

fn advance(state: &State, flux: &Instantaneous, dt: f64) -> State {
    State {
        seconds_since_origin: state.seconds_since_origin + dt,
        component_masses_kg: state
            .component_masses_kg
            .iter()
            .zip(&flux.evaporation_rates_kg_s)
            .map(|(m, rate)| m - dt * rate)
            .collect(),
        liquid_sensible_enthalpy_j: state.liquid_sensible_enthalpy_j
            + dt * flux.liquid_enthalpy_rate_w,
        cumulative_evaporated_kg: state.cumulative_evaporated_kg
            + dt * flux.evaporation_rates_kg_s.iter().sum::<f64>(),
        cumulative_heat_into_liquid_j: state.cumulative_heat_into_liquid_j
            + dt * flux.heat_into_liquid_w,
        cumulative_vapor_enthalpy_out_j: state.cumulative_vapor_enthalpy_out_j
            + dt * flux.vapor_enthalpy_out_w,
        cumulative_complete_combustion_heat_j: state
            .cumulative_complete_combustion_heat_j
            .zip(flux.complete_combustion_heat_release_w)
            .map(|(sum, rate)| sum + dt * rate),
        cumulative_oxygen_consumed_kg: state
            .cumulative_oxygen_consumed_kg
            .zip(flux.oxygen_consumption_kg_s)
            .map(|(sum, rate)| sum + dt * rate),
    }
}

fn midpoint_step(input: &Input, state: &State, dt: f64, parameters: &Parameters) -> Result<State> {
    let flux = instantaneous(input, state, parameters)?;
    for (m, rate) in state
        .component_masses_kg
        .iter()
        .zip(&flux.evaporation_rates_kg_s)
    {
        ensure!(
            dt * rate / m <= parameters.max_fraction_loss,
            "mass-loss step guard failed; no depletion clipping"
        );
    }
    let midpoint = advance(state, &flux, dt / 2.0);
    let middle_flux = instantaneous(input, &midpoint, parameters)?;
    for (mass, rate) in state
        .component_masses_kg
        .iter()
        .zip(&middle_flux.evaporation_rates_kg_s)
    {
        ensure!(
            dt * rate / mass <= parameters.max_fraction_loss,
            "midpoint mass-loss guard failed"
        );
    }
    let next = advance(state, &middle_flux, dt);
    let final_flux = instantaneous(input, &next, parameters)?;
    ensure!(
        (final_flux.liquid_temperature_k - flux.liquid_temperature_k).abs()
            / flux.liquid_temperature_k
            <= parameters.max_fraction_temperature_change,
        "temperature step guard failed"
    );
    Ok(next)
}

/// Fixed-step midpoint integration, independently checked against two half steps.
/// The two-half result is retained. Failure is explicit, never an automatic parameter fit/refinement.
pub fn forward(input: &Input, duration_s: f64, parameters: &Parameters) -> Result<Trajectory> {
    ensure!(
        duration_s.is_finite() && duration_s >= 0.0,
        "invalid duration"
    );
    let initial = initial_state(input)?;
    instantaneous(input, &initial, parameters)?;
    let initial_mass: f64 = initial.component_masses_kg.iter().sum();
    let mut state = initial.clone();
    let mut states = vec![initial.clone()];
    let mut worst: f64 = 0.0;
    while state.seconds_since_origin < duration_s {
        let dt = parameters
            .step_s
            .min(duration_s - state.seconds_since_origin);
        ensure!(
            state.seconds_since_origin + dt > state.seconds_since_origin,
            "time step below floating-point resolution"
        );
        let whole = midpoint_step(input, &state, dt, parameters)?;
        let half = midpoint_step(input, &state, dt / 2.0, parameters)?;
        let mut twice = midpoint_step(input, &half, dt / 2.0, parameters)?;
        for (a, b) in whole
            .component_masses_kg
            .iter()
            .zip(&twice.component_masses_kg)
        {
            worst = worst.max(
                (a - b).abs()
                    / (parameters.absolute_mass_error_kg
                        + parameters.relative_error * a.abs().max(b.abs())),
            );
        }
        let ta = instantaneous(input, &whole, parameters)?.liquid_temperature_k;
        let tb = instantaneous(input, &twice, parameters)?.liquid_temperature_k;
        worst = worst.max(
            (ta - tb).abs()
                / (parameters.absolute_temperature_error_k
                    + parameters.relative_error * ta.abs().max(tb.abs())),
        );
        ensure!(
            worst.is_finite() && worst <= 1.0,
            "local step-doubling error exceeded; trajectory rejected"
        );
        twice.seconds_since_origin = state.seconds_since_origin + dt;
        state = twice;
        states.push(state.clone());
    }
    let mass_residual = state.component_masses_kg.iter().sum::<f64>()
        + state.cumulative_evaporated_kg
        - initial_mass;
    let energy_residual = state.liquid_sensible_enthalpy_j
        - initial.liquid_sensible_enthalpy_j
        - state.cumulative_heat_into_liquid_j
        + state.cumulative_vapor_enthalpy_out_j;
    let energy_scale = state.liquid_sensible_enthalpy_j.abs()
        + initial.liquid_sensible_enthalpy_j.abs()
        + state.cumulative_heat_into_liquid_j.abs()
        + state.cumulative_vapor_enthalpy_out_j.abs();
    ensure!(
        mass_residual.is_finite()
            && mass_residual.abs()
                <= parameters.absolute_mass_error_kg
                    + parameters.conservation_relative_error * initial_mass,
        "mass conservation failed"
    );
    ensure!(
        energy_residual.is_finite()
            && energy_residual.abs() <= parameters.conservation_relative_error * energy_scale,
        "energy conservation failed"
    );
    Ok(Trajectory {
        states,
        mass_balance_residual_kg: mass_residual,
        energy_balance_residual_j: energy_residual,
        maximum_step_doubling_error_ratio: worst,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // All numbers below are deliberately synthetic algebra fixtures, not material data.
    fn prop(value: f64, unit: Unit, phase: Phase) -> Supplied {
        Supplied {
            raw_value: value,
            raw_unit: unit.symbol().into(),
            si_value: value,
            si_unit: unit,
            phase,
            source: "synthetic_fixture:cleanroom-droplet-algebra".into(),
            locator: "property_droplet_cleanroom.rs::tests::fixture".into(),
            origin: Origin::SyntheticFixture,
            uncertainty: Uncertainty::Unquantified,
            reference_temperature_k: 2.0,
            reference_pressure_pa: 10.0,
            reference_identity: "synthetic-sensible-zero-at-2K".into(),
            temperature_domain_k: [1.0, 1000.0],
            pressure_domain_pa: [1.0, 1000.0],
        }
    }

    fn changed(p: &mut Supplied, value: f64) {
        p.si_value = value;
        p.raw_value = value;
    }

    fn fixture() -> Input {
        Input {
            time_origin: "synthetic imposed initial state, seconds since start".into(),
            components: vec![LiquidComponent {
                identity: "synthetic-A".into(),
                atoms: ElementalComposition {
                    carbon: 1,
                    hydrogen: 4,
                    oxygen: 0,
                    nitrogen: 0,
                    sulfur: 0,
                    helium: 0,
                    argon: 0,
                },
                initial_mass: prop(1.0, Unit::Kilogram, Phase::Liquid),
                molar_mass: prop(1.0, Unit::KilogramPerMole, Phase::Liquid),
                liquid_density: prop(10.0, Unit::KilogramPerCubicMetre, Phase::Liquid),
                liquid_heat_capacity: prop(1.0, Unit::JoulePerKilogramKelvin, Phase::Liquid),
                latent_heat_at_reference: prop(1.0, Unit::JoulePerKilogram, Phase::Interface),
                saturation_pressure_at_reference: prop(2.0, Unit::Pascal, Phase::Interface),
                complete_combustion_heat: Some(prop(10.0, Unit::JoulePerKilogram, Phase::Gas)),
            }],
            initial_temperature: prop(2.0, Unit::Kelvin, Phase::Condition),
            ambient_temperature: prop(2.25, Unit::Kelvin, Phase::Condition),
            pressure: prop(10.0, Unit::Pascal, Phase::Condition),
            gas_density: prop(1.0, Unit::KilogramPerCubicMetre, Phase::Gas),
            gas_heat_capacity: prop(1.0, Unit::JoulePerKilogramKelvin, Phase::Gas),
            gas_conductivity: prop(0.01, Unit::WattPerMetreKelvin, Phase::Gas),
            common_diffusivity: prop(0.01, Unit::SquareMetrePerSecond, Phase::Gas),
            gas_viscosity: prop(0.01, Unit::PascalSecond, Phase::Gas),
            surface_background_molar_mass: prop(1.0, Unit::KilogramPerMole, Phase::Gas),
            oxygen_molar_mass: prop(1.0, Unit::KilogramPerMole, Phase::Gas),
            ambient_oxygen_mass_fraction: prop(0.25, Unit::One, Phase::Gas),
            relative_velocity: std::array::from_fn(|_| {
                prop(0.0, Unit::MetrePerSecond, Phase::Condition)
            }),
            gravity: std::array::from_fn(|_| {
                prop(0.0, Unit::MetrePerSecondSquared, Phase::Condition)
            }),
            coordinate_frame: "synthetic-cartesian".into(),
            activity: ActivityClosure::IdealSolution {
                source: "synthetic ideal mixture assumption".into(),
                applicability: "fixture only".into(),
            },
            chemistry: Chemistry::EvaporationOnly,
            transport: TransportApplicability::RequireQuiescentZeroGravity,
        }
    }

    fn mixture() -> Input {
        let mut input = fixture();
        let mut second = input.components[0].clone();
        second.identity = "synthetic-B".into();
        changed(&mut second.saturation_pressure_at_reference, 1.0);
        changed(&mut second.initial_mass, 2.0);
        input.components.push(second);
        input
    }

    fn burning(input: &mut Input) {
        input.chemistry = Chemistry::EstablishedReactionSheet {
            source: "synthetic infinitely-fast closure".into(),
            applicability: "algebra fixture only".into(),
        };
    }

    fn close(a: f64, b: f64, tolerance: f64) {
        assert!((a - b).abs() <= tolerance, "{a} differs from {b}");
    }

    #[test]
    fn literal_spec_patch_materialize_without_outcomes_or_parent() {
        let spec = specification().unwrap();
        let patch = authored_patch().unwrap();
        assert_eq!(patch.author.model_id, "gpt-6-astra");
        assert!(patch.parent_release_id.is_none());
        assert!(patch.exposure.datasets.is_empty());
        assert!(patch.exposure.development_feedback.is_empty());
        assert!(patch.exposure.prior_candidates.is_empty());
        let values = materialize(&spec, &patch, None).unwrap();
        assert_eq!(values.len(), 9);
        assert!(!patch.exposure.sources.is_empty());
        Parameters::from_tensors(&spec, &values).unwrap();
        let mut changed_spec = spec.clone();
        changed_spec.forward.version = "unrelated-forward".into();
        assert!(Parameters::from_tensors(&changed_spec, &values).is_err());
        let mut zero_step = values.clone();
        zero_step.get_mut("step_seconds").unwrap().data[0] = 0.0;
        assert!(Parameters::from_tensors(&spec, &zero_step).is_err());
        let mut missing = values.clone();
        missing.remove("pi");
        assert!(Parameters::from_tensors(&spec, &missing).is_err());
        let mut wrong_shape = values.clone();
        wrong_shape.get_mut("pi").unwrap().shape.push(1);
        assert!(Parameters::from_tensors(&spec, &wrong_shape).is_err());
        let mut nonfinite = values.clone();
        nonfinite.get_mut("pi").unwrap().data[0] = f64::NAN;
        assert!(Parameters::from_tensors(&spec, &nonfinite).is_err());
        let mut stored_variant = values.clone();
        stored_variant.get_mut("step_seconds").unwrap().data[0] = 0.002;
        assert_eq!(
            Parameters::from_tensors(&spec, &stored_variant)
                .unwrap()
                .step_s,
            0.002
        );
        for t in &spec.tensors {
            let b = t.bounds.as_ref().unwrap();
            assert!(b.min < b.max);
        }
    }

    #[test]
    fn pure_isothermal_limit_recovers_d_squared_and_budgets() {
        let input = fixture();
        let par = Parameters::authored().unwrap();
        let first = instantaneous(&input, &initial_state(&input).unwrap(), &par).unwrap();
        // Deliberately designed synthetic energy equilibrium: cp*dT = latent*B, B=1/4.
        close(first.liquid_enthalpy_rate_w, 0.0, 1e-14);
        let result = forward(&input, 0.1, &par).unwrap();
        let last = instantaneous(&input, result.states.last().unwrap(), &par).unwrap();
        let expected = first.liquid_volume_equivalent_diameter_m.powi(2)
            - 8.0 * 0.01 / 10.0 * 1.25_f64.ln() * 0.1;
        close(
            last.liquid_volume_equivalent_diameter_m.powi(2),
            expected,
            1e-11,
        );
        close(last.liquid_temperature_k, 2.0, 1e-12);
        close(result.mass_balance_residual_kg, 0.0, 1e-13);
        close(result.energy_balance_residual_j, 0.0, 1e-13);
    }

    #[test]
    fn mixtures_fractionate_conserve_and_ignore_names_and_permutation() {
        let input = mixture();
        let par = Parameters::authored().unwrap();
        let start = initial_state(&input).unwrap();
        let initial = instantaneous(&input, &start, &par).unwrap();
        close(initial.liquid_mole_fractions[0], 1.0 / 3.0, 1e-15);
        close(initial.interface_fuel_mass_fractions[0], 1.0 / 15.0, 1e-15);
        close(
            initial.evaporation_rates_kg_s[0],
            initial.evaporation_rates_kg_s[1],
            1e-15,
        );
        let result = forward(&input, 0.1, &par).unwrap();
        let end = instantaneous(&input, result.states.last().unwrap(), &par).unwrap();
        assert!(end.liquid_mole_fractions[0] < initial.liquid_mole_fractions[0]);
        assert!(end.liquid_temperature_k > initial.liquid_temperature_k);
        close(result.mass_balance_residual_kg, 0.0, 1e-12);
        close(result.energy_balance_residual_j, 0.0, 1e-12);
        let mut reversed = input.clone();
        reversed.components.reverse();
        reversed.components[0].identity = "name-does-not-select-properties".into();
        let swapped = forward(&reversed, 0.1, &par).unwrap();
        let last_a = result.states.last().unwrap();
        let last_b = swapped.states.last().unwrap();
        close(
            last_a.component_masses_kg[0],
            last_b.component_masses_kg[1],
            1e-13,
        );
        close(
            last_a.component_masses_kg[1],
            last_b.component_masses_kg[0],
            1e-13,
        );
        close(
            last_a.liquid_sensible_enthalpy_j,
            last_b.liquid_sensible_enthalpy_j,
            1e-13,
        );
    }

    #[test]
    fn established_sheet_has_mass_energy_geometry_but_no_optical_or_extinction_alias() {
        let mut input = mixture();
        burning(&mut input);
        let par = Parameters::authored().unwrap();
        let state = initial_state(&input).unwrap();
        let flux = instantaneous(&input, &state, &par).unwrap();
        let rate = flux.evaporation_rates_kg_s.iter().sum::<f64>();
        close(
            flux.complete_combustion_heat_release_w.unwrap(),
            rate * 10.0,
            1e-13,
        );
        close(flux.oxygen_consumption_kg_s.unwrap(), rate * 2.0, 1e-13);
        assert!(flux.reaction_sheet_diameter_m.unwrap() > flux.liquid_volume_equivalent_diameter_m);
        assert!(flux.reaction_sheet_temperature_k.unwrap() > flux.liquid_temperature_k);
        assert!(flux.visible_flame_diameter_m.is_none());
        assert!(flux.uv_flame_diameter_m.is_none());
        assert!(flux.finite_rate_extinction_seconds.is_none());
        let ys = flux.interface_fuel_mass_fractions.iter().sum::<f64>();
        let b = (ys + 0.125) / (1.0 - ys);
        close(
            flux.heat_into_liquid_w,
            rate * (0.25 + 10.0 * 0.125) / b,
            1e-13,
        );
        let result = forward(&input, 0.1, &par).unwrap();
        close(result.mass_balance_residual_kg, 0.0, 1e-12);
        close(result.energy_balance_residual_j, 0.0, 1e-12);
        assert!(
            result
                .states
                .last()
                .unwrap()
                .cumulative_oxygen_consumed_kg
                .unwrap()
                > 0.0
        );
    }

    #[test]
    fn unequal_lewis_number_energy_solution_remains_continuous_at_sheet() {
        let mut input = fixture();
        burning(&mut input);
        changed(&mut input.gas_conductivity, 0.02);
        let par = Parameters::authored().unwrap();
        let flux = instantaneous(&input, &initial_state(&input).unwrap(), &par).unwrap();
        let r = flux.liquid_volume_equivalent_diameter_m / 2.0;
        let rf = flux.reaction_sheet_diameter_m.unwrap() / 2.0;
        let rate = flux.evaporation_rates_kg_s.iter().sum::<f64>();
        let a = rate / (4.0 * par.pi * 0.02);
        let inner = flux.liquid_temperature_k - flux.heat_into_liquid_w / rate;
        let outer = inner + 10.0;
        let tf_outer = outer + (2.25 - outer) * (-a / rf).exp();
        close(flux.reaction_sheet_temperature_k.unwrap(), tf_outer, 1e-12);
        let ts_inner =
            inner + (flux.reaction_sheet_temperature_k.unwrap() - inner) * (a / rf - a / r).exp();
        close(ts_inner, flux.liquid_temperature_k, 1e-12);
    }

    #[test]
    fn gravity_and_motion_are_explicitly_gated_and_never_fabricate_a_correction() {
        let mut input = fixture();
        let par = Parameters::authored().unwrap();
        let baseline = instantaneous(&input, &initial_state(&input).unwrap(), &par).unwrap();
        changed(&mut input.gravity[1], -1.0);
        assert!(forward(&input, 0.0, &par).is_err());
        input.transport = TransportApplicability::ExplicitSphericalExtrapolation;
        changed(&mut input.relative_velocity[0], 0.2);
        let extrapolated = instantaneous(&input, &initial_state(&input).unwrap(), &par).unwrap();
        assert!(extrapolated.extrapolated_spherical_transport);
        assert!(
            extrapolated.gravity_viscosity_scale > 0.0
                && extrapolated.reynolds > 0.0
                && extrapolated.peclet > 0.0
        );
        assert_eq!(
            baseline.evaporation_rates_kg_s,
            extrapolated.evaporation_rates_kg_s
        );
    }

    #[test]
    fn invalid_units_sources_domains_reference_activity_and_boil_boundary_fail_closed() {
        let par = Parameters::authored().unwrap();
        let bad: Vec<Input> = {
            let mut unit = fixture();
            unit.components[0].molar_mass.raw_unit = "g/mol".into();
            let mut mismatch = fixture();
            mismatch.components[0].liquid_density.raw_value = 1000.0;
            let mut source = fixture();
            source.gas_density.locator.clear();
            let mut domain = fixture();
            domain.gas_conductivity.temperature_domain_k = [2.0, 2.1];
            let mut reference = fixture();
            reference.components[0]
                .latent_heat_at_reference
                .reference_identity = "formation-plus-sensible".into();
            let mut activity = fixture();
            activity.activity = ActivityClosure::Unsupported {
                reason: "requires NRTL coefficients".into(),
            };
            let mut boil = fixture();
            changed(
                &mut boil.components[0].saturation_pressure_at_reference,
                10.0,
            );
            let mut nan = fixture();
            changed(&mut nan.ambient_oxygen_mass_fraction, f64::NAN);
            let mut duplicate = mixture();
            duplicate.components[1].identity = duplicate.components[0].identity.clone();
            vec![
                unit, mismatch, source, domain, reference, activity, boil, nan, duplicate,
            ]
        };
        for input in bad {
            assert!(forward(&input, 0.0, &par).is_err());
        }
        let mut input = fixture();
        burning(&mut input);
        changed(&mut input.ambient_oxygen_mass_fraction, 0.0);
        assert!(forward(&input, 0.0, &par).is_err());
    }

    #[test]
    fn numerical_guards_refuse_depletion_and_step_doubling_checks_temperature() {
        let mut input = fixture();
        let par = Parameters::authored().unwrap();
        changed(&mut input.components[0].initial_mass, 1e-15);
        assert!(forward(&input, 0.01, &par).is_err());
        let mut warm = mixture();
        burning(&mut warm);
        let coarse = forward(&warm, 0.01, &par).unwrap();
        let mut finer_numerics = par.clone();
        // A convergence check only: does not modify the committed W0 patch.
        finer_numerics.step_s /= 2.0;
        let fine = forward(&warm, 0.01, &finer_numerics).unwrap();
        let tc = instantaneous(&warm, coarse.states.last().unwrap(), &par)
            .unwrap()
            .liquid_temperature_k;
        let tf = instantaneous(&warm, fine.states.last().unwrap(), &par)
            .unwrap()
            .liquid_temperature_k;
        close(tc, tf, 1e-8);
        assert!(coarse.maximum_step_doubling_error_ratio <= 1.0);
        assert!(fine.maximum_step_doubling_error_ratio <= 1.0);
    }
}

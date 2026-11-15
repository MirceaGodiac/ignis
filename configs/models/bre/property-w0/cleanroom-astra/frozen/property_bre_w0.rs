//! BRE W0: conditional, property-driven virtual-material and real-apparatus balances.
//! Internal scientific kernel, not a shared wire contract or a qualified safety predictor.
//! No future measured response is inferred from a command. See the author dossier for closures.

use crate::physical_core::{
    Finite, PhysicalError, PropertyOrigin, PropertyProvenance, PropertyUncertainty, Result, Vector3,
};

fn number(x: f64) -> Result<f64> {
    Ok(Finite::new(x)?.get())
}
fn nonnegative(x: f64) -> Result<f64> {
    let x = number(x)?;
    if x < 0.0 {
        Err(PhysicalError("negative physical input"))
    } else {
        Ok(x)
    }
}
fn positive(x: f64) -> Result<f64> {
    let x = number(x)?;
    if x <= 0.0 {
        Err(PhysicalError("positive physical input required"))
    } else {
        Ok(x)
    }
}
fn fraction(x: f64) -> Result<f64> {
    let x = nonnegative(x)?;
    if x > 1.0 {
        Err(PhysicalError("fraction outside [0,1]"))
    } else {
        Ok(x)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Kelvin,
    Kilogram,
    SquareMetre,
    JoulePerKilogramKelvin,
    JoulePerKilogram,
    JoulePerKelvin,
    WattPerKelvin,
    WattPerSquareMetre,
    Watt,
    Second,
    KilogramPerSecond,
    Dimensionless,
}

/// The supplier asserts constant-property validity over this environmental box.
/// Gravity and velocity are retained, never converted into an invented transfer correlation.
#[derive(Clone, Debug)]
pub struct Domain {
    pub temperature_kelvin: [f64; 2],
    pub pressure_pascal: [f64; 2],
    pub oxygen_mass_fraction: [f64; 2],
    pub speed_m_per_s: [f64; 2],
    pub gravity_m_per_s2: [f64; 2],
}

#[derive(Clone, Debug)]
pub struct Environment {
    pub ambient_temperature_kelvin: f64,
    pub pressure_pascal: f64,
    pub oxygen_mass_fraction: f64,
    /// Whole oxidizer stream, including diluents; not the oxygen-only flow.
    pub oxidizer_mass_flow_kg_per_s: f64,
    pub velocity_m_per_s: Vector3,
    pub gravity_m_per_s2: Vector3,
    pub provenance: PropertyProvenance,
}

fn provenance(p: &PropertyProvenance) -> Result<()> {
    if p.source.trim().is_empty()
        || p.locator.trim().is_empty()
        || p.dependencies.iter().any(|v| v.trim().is_empty())
    {
        return Err(PhysicalError(
            "missing property source, locator or dependency",
        ));
    }
    if let PropertyUncertainty::AbsoluteBound(v) = p.uncertainty {
        nonnegative(v.get())?;
    }
    Ok(())
}

impl Domain {
    fn check(&self, temperature: f64, e: &Environment) -> Result<()> {
        let values = [
            temperature,
            e.pressure_pascal,
            e.oxygen_mass_fraction,
            e.velocity_m_per_s.magnitude()?.get(),
            e.gravity_m_per_s2.magnitude()?.get(),
        ];
        let ranges = [
            self.temperature_kelvin,
            self.pressure_pascal,
            self.oxygen_mass_fraction,
            self.speed_m_per_s,
            self.gravity_m_per_s2,
        ];
        for (i, ([lo, hi], value)) in ranges.into_iter().zip(values).enumerate() {
            nonnegative(lo)?;
            nonnegative(hi)?;
            number(value)?;
            if lo > hi || value < lo || value > hi || (i < 2 && lo <= 0.0) || (i == 2 && hi > 1.0) {
                return Err(PhysicalError(
                    "property outside declared constant-property domain",
                ));
            }
        }
        Ok(())
    }
}

/// Raw SI value is retained. Unit conversions must occur upstream with their own provenance.
#[derive(Clone, Debug)]
pub struct Property {
    pub value: f64,
    pub unit: Unit,
    pub domain: Domain,
    pub reference: String,
    pub provenance: PropertyProvenance,
}

impl Property {
    fn get(&self, unit: Unit, temperature: f64, e: &Environment) -> Result<f64> {
        if self.unit != unit || self.reference.trim().is_empty() {
            return Err(PhysicalError(
                "property unit or physical reference is missing/wrong",
            ));
        }
        provenance(&self.provenance)?;
        self.domain.check(temperature, e)?;
        self.domain.check(e.ambient_temperature_kelvin, e)?;
        number(self.value)
    }
}

#[derive(Clone, Debug)]
pub struct VirtualMaterial {
    pub exposed_area: Property,
    pub heat_capacity: Property,
    /// Enthalpy difference between departing vapor and material at phase temperature.
    /// Does not include the sensible enthalpy already represented by heat_capacity.
    pub phase_change_enthalpy: Property,
    pub phase_temperature: Property,
    pub sensible_reference_temperature: Property,
    pub linear_loss_conductance: Property,
    pub emissivity: Property,
    /// Required explicit closure. This is not a general pyrolysis/chemical kinetics model.
    pub equilibrium_phase_change_assertion: String,
}

#[derive(Clone, Debug)]
pub struct Apparatus {
    pub thermal_capacity: Property,
    pub exposed_area: Property,
    pub linear_loss_conductance: Property,
    pub emissivity: Property,
    /// Chemical energy per mass actually reacted, with LHV/HHV/reference declared.
    pub heat_of_combustion: Property,
    /// Fraction of chemical heat absorbed by this apparatus thermal node.
    pub absorbed_heat_fraction: Property,
    /// Explicit assumed fraction of oxygen-accessible fuel that reacts. No flame is inferred.
    pub conversion_fraction: Property,
    /// kg oxygen required per kg fuel actually reacted; composition must support this value.
    pub oxygen_required_per_fuel: Property,
    pub actuator_lag: Property,
    pub actuator_maximum_flow: Property,
    pub thermal_limit: Option<Property>,
    pub quasi_steady_combustion_assertion: String,
}

#[derive(Clone, Debug)]
pub struct VirtualState {
    pub remaining_mass_kg: f64,
    pub temperature_kelvin: f64,
}

#[derive(Clone, Debug)]
pub struct State {
    /// Seconds since the explicitly named experimental time origin, not wall-clock time.
    pub seconds: f64,
    pub time_origin: String,
    pub virtual_material: Option<VirtualState>,
    pub apparatus_temperature_kelvin: f64,
    pub actual_fuel_flow_kg_per_s: f64,
    pub provenance: PropertyProvenance,
}

#[derive(Clone, Debug)]
pub enum Command {
    /// Scheduled external command. It is not a predicted response or an observation.
    OpenLoop { fuel_flow: Property },
    /// Emulated material mass loss commands fuel. The supplied flux is conditional future input.
    VirtualMaterialDemand,
}

#[derive(Clone, Debug)]
pub struct Forcing {
    pub environment: Environment,
    pub command: Command,
    /// Absorbed, net incident heating before modeled thermal losses. Not raw sensor voltage,
    /// not incident irradiance and not heat of combustion. None means unavailable.
    pub virtual_absorbed_heat_flux: Option<Property>,
    pub apparatus_external_power: Property,
    pub provenance: PropertyProvenance,
}

#[derive(Clone, Copy, Debug)]
pub struct NumericalControls {
    pub step_seconds: f64,
    pub maximum_step_seconds: f64,
    /// Arithmetic audit tolerance in joules; not physical uncertainty or a tuned coefficient.
    pub energy_absolute_tolerance_joule: f64,
    /// Arithmetic audit tolerance in kilograms.
    pub mass_absolute_tolerance_kg: f64,
    pub relative_tolerance: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Coefficients {
    /// Loaded from the literal W0 patch, not invented or fitted during a request.
    pub stefan_boltzmann_watt_per_m2_kelvin4: f64,
}

#[derive(Clone, Debug)]
pub struct VirtualLedger {
    pub supplied_heat_joule: f64,
    pub thermal_loss_joule: f64,
    pub sensible_storage_change_joule: f64,
    pub departing_sensible_enthalpy_joule: f64,
    pub phase_change_energy_joule: f64,
    pub gasified_mass_kg: f64,
    pub phase_change_begins_after_seconds: Option<f64>,
    pub gasification_rate_during_phase_kg_per_s: f64,
    pub energy_residual_joule: f64,
}

#[derive(Clone, Debug)]
pub struct ApparatusLedger {
    pub delivered_fuel_mass_kg: f64,
    pub reacted_fuel_mass_kg: f64,
    pub unburned_fuel_mass_kg: f64,
    pub oxygen_supplied_mass_kg: f64,
    pub oxygen_consumed_mass_kg: f64,
    pub oxygen_exhaust_mass_kg: f64,
    pub inert_oxidizer_exhaust_mass_kg: f64,
    pub reaction_product_mass_kg: f64,
    pub mass_residual_kg: f64,
    pub chemical_heat_joule: f64,
    pub absorbed_chemical_heat_joule: f64,
    pub chemical_heat_not_absorbed_joule: f64,
    pub external_heat_joule: f64,
    pub thermal_loss_joule: f64,
    pub storage_change_joule: f64,
    pub energy_residual_joule: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThermalLimitAssessment {
    Unavailable,
    WithinDeclaredLimit,
    Exceeded,
}

#[derive(Clone, Debug)]
pub struct SafetyObservable {
    /// Always UNKNOWN: this thermal control-volume model does not establish fire safety.
    pub status: &'static str,
    pub thermal_limit_assessment: ThermalLimitAssessment,
    pub thermal_limit_margin_kelvin: Option<f64>,
    pub unburned_fuel_mass_kg: f64,
    pub unresolved: Vec<&'static str>,
}

#[derive(Clone, Debug)]
pub struct Step {
    pub state: State,
    /// Exact existing Quantity::Temperature target, K: the real apparatus thermal node.
    pub predicted_apparatus_temperature_kelvin: f64,
    /// Interval mean of the requested command; never substituted for the event-timed command.
    pub requested_command_kg_per_s: f64,
    pub requested_command_end_kg_per_s: f64,
    pub command_change_after_seconds: Option<f64>,
    /// Saturated command at the interval end.
    pub saturated_command_kg_per_s: f64,
    pub actual_fuel_flow_end_kg_per_s: f64,
    pub virtual_ledger: Option<VirtualLedger>,
    pub virtual_unavailable_reason: Option<&'static str>,
    pub apparatus_ledger: ApparatusLedger,
    pub safety: SafetyObservable,
    pub qualification: &'static str,
    pub uncertainty: &'static str,
}

fn loss(t: f64, ambient: f64, k: f64, emissivity: f64, area: f64, sigma: f64) -> Result<f64> {
    // Factorization retains the exact zero at equal temperature and avoids subtracting T^4.
    number(
        k * (t - ambient)
            + emissivity
                * sigma
                * area
                * (t - ambient)
                * (t + ambient)
                * (t * t + ambient * ambient),
    )
}

fn audit(residual: f64, scale: f64, absolute: f64, relative: f64) -> Result<()> {
    let bound = number(absolute + relative * scale.abs())?;
    if number(residual)?.abs() > bound {
        Err(PhysicalError(
            "discrete conservation residual exceeds supplied arithmetic tolerance",
        ))
    } else {
        Ok(())
    }
}

fn monotone_step(
    dt: f64,
    capacity: f64,
    conductance: f64,
    emissivity: f64,
    area: f64,
    sigma: f64,
    maximum_temperature: f64,
) -> Result<()> {
    // Exact derivative d(T^4)/dT = 4*T^3; forward-Euler monotonicity condition, no fitted factor.
    let ratio = number(
        dt * (conductance + 4.0 * emissivity * sigma * area * maximum_temperature.powi(3))
            / capacity,
    )?;
    if ratio > 1.0 {
        Err(PhysicalError(
            "thermal step violates monotonicity; reduce explicit step",
        ))
    } else {
        Ok(())
    }
}

/// Exact first-order actuator integration, with no inference about combustion.
fn actuator_mass(start: f64, command: f64, tau: f64, duration: f64) -> Result<(f64, f64)> {
    let response = -(-duration / tau).exp_m1();
    let end = number(start + (command - start) * response)?;
    let mass = number(command * duration + (start - command) * tau * response)?;
    nonnegative(end)?;
    nonnegative(mass)?;
    // Underflow/cancellation cannot silently replace a positive delivered quantity by zero.
    if duration > 0.0 && (start > 0.0 || command > 0.0) && mass == 0.0 {
        return Err(PhysicalError(
            "actuator integral lost a positive contribution",
        ));
    }
    Ok((end, mass))
}

/// Integral min(actual fuel flow, oxygen-limited fuel capacity), with exact crossing split.
fn oxygen_accessible_mass(
    start: f64,
    command: f64,
    tau: f64,
    duration: f64,
    cap: f64,
    end: f64,
    total: f64,
) -> Result<f64> {
    if start <= cap && end <= cap {
        return Ok(total);
    }
    if start >= cap && end >= cap {
        return number(cap * duration);
    }
    let cross = number(-tau * ((cap - command) / (start - command)).ln())?;
    if cross < 0.0 || cross > duration {
        return Err(PhysicalError(
            "oxygen crossing time outside integration interval",
        ));
    }
    let before = actuator_mass(start, command, tau, cross)?.1;
    if start < cap {
        number(before + cap * (duration - cross))
    } else {
        number(cap * cross + total - before)
    }
}

fn virtual_step(
    m: &VirtualMaterial,
    state: &VirtualState,
    q: &Property,
    e: &Environment,
    c: Coefficients,
    n: NumericalControls,
) -> Result<(VirtualState, VirtualLedger)> {
    if m.equilibrium_phase_change_assertion.trim().is_empty() {
        return Err(PhysicalError("virtual phase-change closure is unavailable"));
    }
    let t = positive(state.temperature_kelvin)?;
    let mass = positive(state.remaining_mass_kg)?;
    let area = positive(m.exposed_area.get(Unit::SquareMetre, t, e)?)?;
    let cp = positive(m.heat_capacity.get(Unit::JoulePerKilogramKelvin, t, e)?)?;
    let latent = positive(m.phase_change_enthalpy.get(Unit::JoulePerKilogram, t, e)?)?;
    let phase = positive(m.phase_temperature.get(Unit::Kelvin, t, e)?)?;
    let reference = positive(m.sensible_reference_temperature.get(Unit::Kelvin, t, e)?)?;
    if t > phase
        || reference > phase
        || m.heat_capacity.reference != m.phase_change_enthalpy.reference
    {
        return Err(PhysicalError(
            "virtual state/reference incompatible with phase-change enthalpy",
        ));
    }
    let k = nonnegative(m.linear_loss_conductance.get(Unit::WattPerKelvin, t, e)?)?;
    let emissivity = fraction(m.emissivity.get(Unit::Dimensionless, t, e)?)?;
    let supplied =
        number(area * nonnegative(q.get(Unit::WattPerSquareMetre, t, e)?)? * n.step_seconds)?;
    let lost = number(
        loss(
            t,
            e.ambient_temperature_kelvin,
            k,
            emissivity,
            area,
            c.stefan_boltzmann_watt_per_m2_kelvin4,
        )? * n.step_seconds,
    )?;
    let energy = number(supplied - lost)?;
    let warming = number(mass * cp * (phase - t))?;
    let (next_t, gasified) = if energy <= warming {
        (positive(t + energy / (mass * cp))?, 0.0)
    } else {
        (phase, positive((energy - warming) / latent)?)
    };
    if gasified >= mass {
        return Err(PhysicalError(
            "virtual material exhaustion reached; split/stop before exhaustion",
        ));
    }
    monotone_step(
        n.step_seconds,
        mass * cp,
        k,
        emissivity,
        area,
        c.stefan_boltzmann_watt_per_m2_kelvin4,
        t.max(next_t).max(e.ambient_temperature_kelvin),
    )?;
    for property in [
        &m.exposed_area,
        &m.heat_capacity,
        &m.phase_change_enthalpy,
        &m.phase_temperature,
        &m.sensible_reference_temperature,
        &m.linear_loss_conductance,
        &m.emissivity,
        q,
    ] {
        property.domain.check(next_t, e)?;
    }
    let remaining = positive(mass - gasified)?;
    let storage = number(remaining * cp * (next_t - reference) - mass * cp * (t - reference))?;
    let sensible_out = number(gasified * cp * (phase - reference))?;
    let latent_out = number(gasified * latent)?;
    let residual = number(storage + sensible_out + latent_out - supplied + lost)?;
    audit(
        residual,
        storage.abs() + sensible_out.abs() + latent_out.abs() + supplied.abs() + lost.abs(),
        n.energy_absolute_tolerance_joule,
        n.relative_tolerance,
    )?;
    Ok((
        VirtualState {
            remaining_mass_kg: remaining,
            temperature_kelvin: next_t,
        },
        VirtualLedger {
            supplied_heat_joule: supplied,
            thermal_loss_joule: lost,
            sensible_storage_change_joule: storage,
            departing_sensible_enthalpy_joule: sensible_out,
            phase_change_energy_joule: latent_out,
            gasified_mass_kg: gasified,
            phase_change_begins_after_seconds: if gasified > 0.0 {
                Some(number(warming / energy * n.step_seconds)?)
            } else {
                None
            },
            gasification_rate_during_phase_kg_per_s: if gasified > 0.0 {
                positive(energy / n.step_seconds / latent)?
            } else {
                0.0
            },
            energy_residual_joule: residual,
        },
    ))
}

/// One explicit causal step; call repeatedly with independently supplied future forcing.
/// Missing closures are errors/unavailable, never defaults or zero-valued observations.
pub fn advance(
    coefficients: Coefficients,
    material: Option<&VirtualMaterial>,
    apparatus: &Apparatus,
    initial: &State,
    forcing: &Forcing,
    numerical: NumericalControls,
) -> Result<Step> {
    let n = numerical;
    positive(n.step_seconds)?;
    positive(n.maximum_step_seconds)?;
    nonnegative(n.energy_absolute_tolerance_joule)?;
    nonnegative(n.mass_absolute_tolerance_kg)?;
    nonnegative(n.relative_tolerance)?;
    if n.step_seconds > n.maximum_step_seconds {
        return Err(PhysicalError("step exceeds caller numerical budget"));
    }
    positive(coefficients.stefan_boltzmann_watt_per_m2_kelvin4)?;
    if initial.time_origin.trim().is_empty() {
        return Err(PhysicalError("missing experimental time origin"));
    }
    number(initial.seconds)?;
    provenance(&initial.provenance)?;
    provenance(&forcing.provenance)?;
    let e = &forcing.environment;
    provenance(&e.provenance)?;
    positive(e.ambient_temperature_kelvin)?;
    positive(e.pressure_pascal)?;
    fraction(e.oxygen_mass_fraction)?;
    nonnegative(e.oxidizer_mass_flow_kg_per_s)?;
    if e.velocity_m_per_s.frame() != e.gravity_m_per_s2.frame() {
        return Err(PhysicalError(
            "velocity and gravity coordinate frames differ",
        ));
    }
    let t = positive(initial.apparatus_temperature_kelvin)?;
    let start = nonnegative(initial.actual_fuel_flow_kg_per_s)?;
    let (virtual_state, virtual_ledger, virtual_unavailable_reason) = match (
        material,
        initial.virtual_material.as_ref(),
        forcing.virtual_absorbed_heat_flux.as_ref(),
    ) {
        (Some(m), Some(s), Some(q)) => {
            let (state, ledger) = virtual_step(m, s, q, e, coefficients, n)?;
            (Some(state), Some(ledger), None)
        }
        (_, _, _) => (
            None,
            None,
            Some("virtual material, state or future absorbed heat flux unavailable"),
        ),
    };
    let (requested, requested_end, command_change_after_seconds) = match &forcing.command {
        Command::OpenLoop { fuel_flow } => {
            let requested = nonnegative(fuel_flow.get(Unit::KilogramPerSecond, t, e)?)?;
            (requested, requested, None)
        }
        Command::VirtualMaterialDemand => {
            let ledger = virtual_ledger.as_ref().ok_or(PhysicalError(
                "virtual-demand command unavailable without virtual input closure",
            ))?;
            (
                number(ledger.gasified_mass_kg / n.step_seconds)?,
                ledger.gasification_rate_during_phase_kg_per_s,
                ledger.phase_change_begins_after_seconds,
            )
        }
    };
    if apparatus
        .quasi_steady_combustion_assertion
        .trim()
        .is_empty()
    {
        return Err(PhysicalError(
            "combustion kinetics/mixing closure is unavailable",
        ));
    }
    let props = [
        &apparatus.thermal_capacity,
        &apparatus.exposed_area,
        &apparatus.linear_loss_conductance,
        &apparatus.emissivity,
        &apparatus.heat_of_combustion,
        &apparatus.absorbed_heat_fraction,
        &apparatus.conversion_fraction,
        &apparatus.oxygen_required_per_fuel,
        &apparatus.actuator_lag,
        &apparatus.actuator_maximum_flow,
    ];
    let capacity = positive(props[0].get(Unit::JoulePerKelvin, t, e)?)?;
    let area = positive(props[1].get(Unit::SquareMetre, t, e)?)?;
    let k = nonnegative(props[2].get(Unit::WattPerKelvin, t, e)?)?;
    let emissivity = fraction(props[3].get(Unit::Dimensionless, t, e)?)?;
    let heat = positive(props[4].get(Unit::JoulePerKilogram, t, e)?)?;
    let absorbed = fraction(props[5].get(Unit::Dimensionless, t, e)?)?;
    let conversion = fraction(props[6].get(Unit::Dimensionless, t, e)?)?;
    let oxygen_ratio = positive(props[7].get(Unit::Dimensionless, t, e)?)?;
    let tau = positive(props[8].get(Unit::Second, t, e)?)?;
    let maximum = positive(props[9].get(Unit::KilogramPerSecond, t, e)?)?;
    if start > maximum {
        return Err(PhysicalError("initial actual flow exceeds actuator range"));
    }
    let command = requested_end.min(maximum);
    let oxygen_flow = number(e.oxidizer_mass_flow_kg_per_s * e.oxygen_mass_fraction)?;
    let oxygen_cap = nonnegative(oxygen_flow / oxygen_ratio)?;
    let segment = |initial_flow, held_command, duration| -> Result<(f64, f64, f64)> {
        let (end, mass) = actuator_mass(initial_flow, held_command, tau, duration)?;
        let accessible = oxygen_accessible_mass(
            initial_flow,
            held_command,
            tau,
            duration,
            oxygen_cap,
            end,
            mass,
        )?;
        Ok((end, mass, accessible))
    };
    let (end, delivered, accessible) = if let Some(change) = command_change_after_seconds {
        if change < 0.0 || change >= n.step_seconds {
            return Err(PhysicalError(
                "phase-change command timing is unrepresentable",
            ));
        }
        let (middle, before_mass, before_accessible) = segment(start, 0.0, change)?;
        let (end, after_mass, after_accessible) =
            segment(middle, command, n.step_seconds - change)?;
        (
            end,
            number(before_mass + after_mass)?,
            number(before_accessible + after_accessible)?,
        )
    } else {
        segment(start, command, n.step_seconds)?
    };
    let reacted = nonnegative(accessible * conversion)?;
    let unburned = nonnegative(delivered - reacted)?;
    let oxygen_in = number(oxygen_flow * n.step_seconds)?;
    let oxygen_used = number(reacted * oxygen_ratio)?;
    let oxygen_out = nonnegative(oxygen_in - oxygen_used)?;
    let inert =
        number(e.oxidizer_mass_flow_kg_per_s * (1.0 - e.oxygen_mass_fraction) * n.step_seconds)?;
    let products = number(reacted + oxygen_used)?;
    let total_in = number(delivered + e.oxidizer_mass_flow_kg_per_s * n.step_seconds)?;
    let mass_residual = number(unburned + oxygen_out + inert + products - total_in)?;
    audit(
        mass_residual,
        total_in,
        n.mass_absolute_tolerance_kg,
        n.relative_tolerance,
    )?;
    let chemical = number(heat * reacted)?;
    let absorbed_heat = number(absorbed * chemical)?;
    let external =
        number(forcing.apparatus_external_power.get(Unit::Watt, t, e)? * n.step_seconds)?;
    let lost = number(
        loss(
            t,
            e.ambient_temperature_kelvin,
            k,
            emissivity,
            area,
            coefficients.stefan_boltzmann_watt_per_m2_kelvin4,
        )? * n.step_seconds,
    )?;
    let next_t = positive(t + (absorbed_heat + external - lost) / capacity)?;
    monotone_step(
        n.step_seconds,
        capacity,
        k,
        emissivity,
        area,
        coefficients.stefan_boltzmann_watt_per_m2_kelvin4,
        t.max(next_t).max(e.ambient_temperature_kelvin),
    )?;
    for p in props {
        p.domain.check(next_t, e)?;
    }
    forcing.apparatus_external_power.domain.check(next_t, e)?;
    let storage = number(capacity * (next_t - t))?;
    let residual = number(storage - absorbed_heat - external + lost)?;
    audit(
        residual,
        storage.abs() + absorbed_heat.abs() + external.abs() + lost.abs(),
        n.energy_absolute_tolerance_joule,
        n.relative_tolerance,
    )?;
    let margin = apparatus
        .thermal_limit
        .as_ref()
        .map(|p| -> Result<f64> { Ok(positive(p.get(Unit::Kelvin, next_t, e)?)? - next_t) })
        .transpose()?;
    let assessment = match margin {
        None => ThermalLimitAssessment::Unavailable,
        Some(v) if v < 0.0 => ThermalLimitAssessment::Exceeded,
        Some(_) => ThermalLimitAssessment::WithinDeclaredLimit,
    };
    let mut next = initial.clone();
    next.seconds = number(initial.seconds + n.step_seconds)?;
    if next.seconds <= initial.seconds {
        return Err(PhysicalError("time increment lost to roundoff"));
    }
    next.virtual_material = virtual_state;
    next.apparatus_temperature_kelvin = next_t;
    next.actual_fuel_flow_kg_per_s = end;
    next.provenance = PropertyProvenance {
        origin: PropertyOrigin::ExplicitScenarioAssumption,
        source: "ignis_wsp::property_bre_w0::advance".into(),
        locator: next.seconds.to_string(),
        dependencies: vec![
            initial.provenance.source.clone(),
            forcing.provenance.source.clone(),
        ],
        uncertainty: PropertyUncertainty::Unquantified,
    };
    Ok(Step { state: next, predicted_apparatus_temperature_kelvin: next_t,
        requested_command_kg_per_s: requested, requested_command_end_kg_per_s: requested_end,
        command_change_after_seconds, saturated_command_kg_per_s: command,
        actual_fuel_flow_end_kg_per_s: end, virtual_ledger, virtual_unavailable_reason,
        apparatus_ledger: ApparatusLedger { delivered_fuel_mass_kg: delivered,
            reacted_fuel_mass_kg: reacted, unburned_fuel_mass_kg: unburned,
            oxygen_supplied_mass_kg: oxygen_in, oxygen_consumed_mass_kg: oxygen_used,
            oxygen_exhaust_mass_kg: oxygen_out, inert_oxidizer_exhaust_mass_kg: inert,
            reaction_product_mass_kg: products, mass_residual_kg: mass_residual,
            chemical_heat_joule: chemical, absorbed_chemical_heat_joule: absorbed_heat,
            chemical_heat_not_absorbed_joule: number(chemical-absorbed_heat)?,
            external_heat_joule: external, thermal_loss_joule: lost,
            storage_change_joule: storage, energy_residual_joule: residual },
        safety: SafetyObservable { status: "UNKNOWN", thermal_limit_assessment: assessment,
            thermal_limit_margin_kelvin: margin, unburned_fuel_mass_kg: unburned,
            unresolved: vec!["conversion and heat-return assumptions are not kinetic validation",
                "extinction, ignition, spatial hot spots, toxic species and flame spread unavailable",
                "no predictive uncertainty coverage or experimentally qualified safety envelope"] },
        qualification: "experimental; conditional conservation calculation; not_trained",
        uncertainty: "unquantified model discrepancy; property uncertainty retained but not propagated" })
}

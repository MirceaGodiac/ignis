//! Outcome-blind, constant-transport, unity-Lewis spherical flame-sheet reference.
//!
//! This is a conditional steady burning branch, not an ignition/extinction model or an
//! optical observation model. No dataset, fuel-name dispatch, solver or fitted coefficient.
//! Derivation and applicability: docs/wsp/research/astra_spherical_w0.md.

use crate::physical_core::{
    stoichiometric_mixture_fraction, EnthalpyConvention, Finite, GasMixture, NumericalTolerance,
    Phase, PhysicalError, Positive, PropertyDomain, PropertyProvenance, PropertyUncertainty,
    ReferenceState, Result, ThermodynamicState, Vector3,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Metre,
    Millimetre,
    KilogramPerSecond,
    GramPerSecond,
    KilogramPerMetreSecond,
    JoulePerKilogram,
    KilojoulePerKilogram,
    KilogramPerCubicMetre,
    PerMetre,
    Kelvin,
    WattPerKelvin,
}

impl Unit {
    fn si(self) -> (Self, f64) {
        match self {
            Self::Millimetre => (Self::Metre, 0.001),
            Self::GramPerSecond => (Self::KilogramPerSecond, 0.001),
            Self::KilojoulePerKilogram => (Self::JoulePerKilogram, 1000.0),
            other => (other, 1.0),
        }
    }
}

/// A supplied input, not an authored weight. Raw value/unit and all metadata are retained.
/// A broad domain explicitly declares a constant-property approximation over that domain.
#[derive(Clone, Debug)]
pub struct ScalarInput {
    raw_value: Finite,
    raw_unit: Unit,
    si_value: Finite,
    si_uncertainty: PropertyUncertainty,
    domain: PropertyDomain,
    reference: ReferenceState,
    provenance: PropertyProvenance,
}

impl ScalarInput {
    pub fn new(
        raw_value: Finite,
        raw_unit: Unit,
        domain: PropertyDomain,
        reference: ReferenceState,
        provenance: PropertyProvenance,
    ) -> Result<Self> {
        check_provenance(&provenance)?;
        check_reference(&reference)?;
        if domain.phase != Phase::Gas
            || domain.minimum.temperature_kelvin.get() > domain.maximum.temperature_kelvin.get()
            || domain.minimum.pressure_pascal.get() > domain.maximum.pressure_pascal.get()
        {
            return Err(PhysicalError("invalid gas/scenario property domain"));
        }
        let si_value = Finite::new(raw_value.get() * raw_unit.si().1)?;
        let si_uncertainty = match provenance.uncertainty {
            PropertyUncertainty::Unquantified => PropertyUncertainty::Unquantified,
            PropertyUncertainty::AbsoluteBound(bound) => {
                PropertyUncertainty::AbsoluteBound(Finite::new(bound.get() * raw_unit.si().1)?)
            }
        };
        if raw_value.get() != 0.0 && si_value.get() == 0.0 {
            return Err(PhysicalError("raw-to-SI underflow"));
        }
        Ok(Self {
            raw_value,
            raw_unit,
            si_value,
            si_uncertainty,
            domain,
            reference,
            provenance,
        })
    }

    pub fn raw_value(&self) -> Finite {
        self.raw_value
    }
    pub fn raw_unit(&self) -> Unit {
        self.raw_unit
    }
    pub fn si_value(&self) -> Finite {
        self.si_value
    }
    /// The provenance retains the supplied raw-unit bound; this exposes its SI conversion.
    pub fn si_uncertainty(&self) -> &PropertyUncertainty {
        &self.si_uncertainty
    }
    pub fn domain(&self) -> PropertyDomain {
        self.domain
    }
    pub fn reference(&self) -> &ReferenceState {
        &self.reference
    }
    pub fn provenance(&self) -> &PropertyProvenance {
        &self.provenance
    }

    fn at(&self, unit: Unit, state: ThermodynamicState) -> Result<f64> {
        if self.raw_unit.si().0 != unit || !self.domain.contains(state, Phase::Gas) {
            return Err(PhysicalError(
                "wrong input unit or state outside supplied domain",
            ));
        }
        Ok(self.si_value.get())
    }
}

fn check_provenance(p: &PropertyProvenance) -> Result<()> {
    if p.source.trim().is_empty()
        || p.locator.trim().is_empty()
        || p.dependencies.iter().any(|x| x.trim().is_empty())
    {
        return Err(PhysicalError("missing property provenance"));
    }
    if let PropertyUncertainty::AbsoluteBound(bound) = p.uncertainty {
        if bound.get() < 0.0 {
            return Err(PhysicalError("negative uncertainty bound"));
        }
    }
    Ok(())
}

fn check_reference(r: &ReferenceState) -> Result<()> {
    match &r.convention {
        EnthalpyConvention::IncludingFormation { reference_identity }
            if !reference_identity.trim().is_empty() =>
        {
            Ok(())
        }
        _ => Err(PhysicalError(
            "a named common formation-enthalpy reference is required",
        )),
    }
}

/// Formation enthalpy belongs to this exact supplied mixture, per kg of that mixture.
/// Its source must document composition, product phase and the common elemental reference.
#[derive(Clone, Debug)]
pub struct MixtureThermo {
    pub mixture: GasMixture,
    pub formation_enthalpy: ScalarInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    NormalFuelInjection,
    InverseOxidizerInjection,
}

/// Only the admitted mathematical closure exists; absence of a closure is not a default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BurningClosure {
    EstablishedFastChemistryAdiabaticEqualDiffusivity,
}

/// The three author-written literals are read from the assembled tensor, in this order.
/// Neither is an empirical calibration: solid angle is geometry, unity Lewis is a restriction.
#[derive(Clone, Copy, Debug)]
pub struct Parameters {
    solid_angle: Positive,
    lewis_number: Positive,
    stefan_boltzmann: Positive,
}

impl Parameters {
    pub fn from_literals(values: &[f64]) -> Result<Self> {
        if values.len() != 3 {
            return Err(PhysicalError("expected three authored literals"));
        }
        let p = Self {
            solid_angle: Positive::new(values[0])?,
            lewis_number: Positive::new(values[1])?,
            stefan_boltzmann: Positive::new(values[2])?,
        };
        if p.solid_angle.get() != 4.0 * std::f64::consts::PI || p.lewis_number.get() != 1.0 {
            return Err(PhysicalError(
                "this derived kernel requires sphere geometry and unity Lewis",
            ));
        }
        if p.stefan_boltzmann.get() != 0.00000005670374419 {
            return Err(PhysicalError(
                "Stefan-Boltzmann literal differs from this authored CODATA truncation",
            ));
        }
        Ok(p)
    }
}

#[derive(Clone, Debug)]
pub struct Conditions {
    pub mode: Mode,
    pub closure: BurningClosure,
    pub burner_radius: ScalarInput,
    pub injected_mass_flow: ScalarInput,
    /// Constant rho*D, not D. Density may vary while this product is declared constant.
    pub mass_diffusivity: ScalarInput,
    pub fuel: MixtureThermo,
    pub oxidizer: MixtureThermo,
    /// Complete-reaction products per kg stoichiometric mixture, explicitly supplied.
    pub products: MixtureThermo,
    /// Actual acceleration, m/s2. This kernel admits exactly zero, with no microgravity epsilon.
    pub gravity_m_per_s2: Vector3,
    pub gravity_provenance: PropertyProvenance,
    /// Imposed ambient velocity, m/s; radial injection is accounted for separately.
    pub ambient_velocity_m_per_s: Vector3,
    pub ambient_velocity_provenance: PropertyProvenance,
}

/// Numerical comparison bounds, never tunable physical parameters. No root finder, grid,
/// iteration count or integration timestep exists in this closed-form kernel.
#[derive(Clone, Copy, Debug)]
pub struct Controls {
    pub element_balance_moles_per_kg: NumericalTolerance,
    pub energy_balance_joule_per_kg: NumericalTolerance,
    pub scalar_flux_kg_per_s: NumericalTolerance,
}

#[derive(Clone, Debug)]
pub struct ConditionalReference {
    pub mode: Mode,
    pub stoichiometric_fuel_mass_fraction: Finite,
    /// A reaction-sheet radius. Never an alias for visible/UV flame diameter.
    pub reaction_sheet_radius_m: Positive,
    pub sheet_temperature_kelvin: Positive,
    pub chemical_release_joule_per_kg_stoichiometric_mixture: Positive,
    pub burner_injected_stream_mass_fraction: Finite,
    pub minimum_mass_flow_for_exterior_sheet_kg_per_s: Positive,
    pub energy_residual_joule_per_kg: Finite,
    pub gravity_m_per_s2: Vector3,
    radius: Positive,
    length: Positive,
    mass_flow: Positive,
    gamma: Positive,
    parameters: Parameters,
    reference_temperature: Positive,
    inlet_enthalpies: [Finite; 2],
    formation_enthalpies: [Finite; 3],
    heat_capacities: [Positive; 3],
    temperature_min: Positive,
    temperature_max: Positive,
    controls: Controls,
}

#[derive(Clone, Debug)]
pub struct RadialState {
    pub radius_m: Positive,
    pub fuel_origin_mass_fraction: Finite,
    pub fuel_origin_gradient_per_m: Finite,
    pub reacted_product_mass_fraction: Finite,
    pub unreacted_fuel_stream_mass_fraction: Finite,
    pub unreacted_oxidizer_stream_mass_fraction: Finite,
    pub temperature_kelvin: Positive,
    pub outward_mass_flux_kg_per_m2_s: Positive,
    pub injected_scalar_flow_residual_kg_per_s: Finite,
}

fn cp_mass(m: &GasMixture, reference: &ReferenceState, lo: f64, hi: f64) -> Result<Positive> {
    let mut cp = 0.0;
    for (component, mole_fraction) in m.components().iter().zip(m.mole_fractions()) {
        for property in [component.heat_capacity(), component.molar_mass()] {
            if property.reference() != reference {
                return Err(PhysicalError(
                    "inconsistent species thermochemical reference",
                ));
            }
            for t in [lo, hi, reference.state.temperature_kelvin.get()] {
                let state = ThermodynamicState {
                    temperature_kelvin: Positive::new(t)?,
                    pressure_pascal: m.state().pressure_pascal,
                };
                if !property.domain().contains(state, Phase::Gas) {
                    return Err(PhysicalError(
                        "species constant-property domain excludes temperature path",
                    ));
                }
            }
        }
        cp += mole_fraction.get() * component.heat_capacity_molar()?.get();
    }
    Positive::new(cp / m.molar_mass_kg_per_mol().get())
}

fn enthalpy_temperature(
    z: f64,
    zst: f64,
    tref: f64,
    inlet: [Finite; 2],
    formation: [Finite; 3],
    cp: [Positive; 3],
) -> Result<(Positive, [f64; 3], Finite)> {
    if !(0.0..=1.0).contains(&z) {
        return Err(PhysicalError("mixture fraction outside [0,1]"));
    }
    let h = Finite::new(z * inlet[0].get() + (1.0 - z) * inlet[1].get())?.get();
    temperature_from_enthalpy(z, zst, tref, h, formation, cp)
}

fn temperature_from_enthalpy(
    z: f64,
    zst: f64,
    tref: f64,
    h: f64,
    formation: [Finite; 3],
    cp: [Positive; 3],
) -> Result<(Positive, [f64; 3], Finite)> {
    Finite::new(h)?;
    if !(0.0..=1.0).contains(&z) {
        return Err(PhysicalError("mixture fraction outside [0,1]"));
    }
    let fractions = if z <= zst {
        let product = z / zst;
        [0.0, 1.0 - product, product]
    } else {
        let product = (1.0 - z) / (1.0 - zst);
        [1.0 - product, 0.0, product]
    };
    let hm: f64 = fractions
        .iter()
        .zip(formation)
        .map(|(y, h)| y * h.get())
        .sum();
    let cpm = Positive::new(fractions.iter().zip(cp).map(|(y, c)| y * c.get()).sum())?.get();
    let temperature = Positive::new(tref + (h - hm) / cpm)?;
    let residual = Finite::new(hm + cpm * (temperature.get() - tref) - h)?;
    Ok((temperature, fractions, residual))
}

/// Builds the exact conditional steady reference; all material values are supplied inputs.
pub fn forward(c: &Conditions, p: Parameters, controls: Controls) -> Result<ConditionalReference> {
    check_provenance(&c.gravity_provenance)?;
    check_provenance(&c.ambient_velocity_provenance)?;
    if c.gravity_m_per_s2.frame() != c.ambient_velocity_m_per_s.frame() {
        return Err(PhysicalError("gravity and velocity frames differ"));
    }
    if c.gravity_m_per_s2.magnitude()?.get() != 0.0 {
        return Err(PhysicalError(
            "nonzero gravity needs nonspherical momentum/buoyancy closure",
        ));
    }
    if c.ambient_velocity_m_per_s.magnitude()?.get() != 0.0 {
        return Err(PhysicalError("ambient crossflow breaks spherical symmetry"));
    }
    let reference = c.fuel.formation_enthalpy.reference();
    check_reference(reference)?;
    let pressure = c.fuel.mixture.state().pressure_pascal;
    let tref = reference.state.temperature_kelvin;
    let zst = stoichiometric_mixture_fraction(&c.fuel.mixture, &c.oxidizer.mixture)?;
    let z = zst.get();
    let streams = [&c.fuel, &c.oxidizer, &c.products];
    for (stream, sign) in [(&c.fuel, 1.0), (&c.oxidizer, -1.0)] {
        for component in stream.mixture.components() {
            if sign * component.elements().oxygen_demand_moles_per_mole()?.get() < 0.0 {
                return Err(PhysicalError(
                    "premixed reactant streams are outside this flame-sheet closure",
                ));
            }
        }
    }
    for component in c.products.mixture.components() {
        let e = component.elements();
        if !matches!(
            (e.carbon, e.hydrogen, e.oxygen, e.nitrogen, e.sulfur, e.helium, e.argon),
            (1, 0, 2, 0, 0, 0, 0)
                | (0, 2, 1, 0, 0, 0, 0)
                | (0, 0, 0, 2, 0, 0, 0)
                | (0, 0, 2, 0, 1, 0, 0)
                | (0, 0, 0, 0, 0, 1, 0)
                | (0, 0, 0, 0, 0, 0, 1)
        ) {
            return Err(PhysicalError(
                "products must be CO2/H2O/N2/SO2/He/Ar by composition, without dissociation",
            ));
        }
    }
    let mut formation = [Finite::new(0.0)?; 3];
    let mut cp = [Positive::new(1.0)?; 3];
    for (i, stream) in streams.iter().enumerate() {
        if stream.formation_enthalpy.reference() != reference
            || stream.mixture.state().pressure_pascal != pressure
            || reference.state.pressure_pascal != pressure
        {
            return Err(PhysicalError(
                "mixtures need a common pressure and formation reference",
            ));
        }
        formation[i] = Finite::new(
            stream
                .formation_enthalpy
                .at(Unit::JoulePerKilogram, reference.state)?,
        )?;
        cp[i] = cp_mass(
            &stream.mixture,
            reference,
            tref.get(),
            stream.mixture.state().temperature_kelvin.get(),
        )?;
    }
    let fuel_elements = c.fuel.mixture.element_inventory_moles_per_kg()?;
    let oxidizer_elements = c.oxidizer.mixture.element_inventory_moles_per_kg()?;
    let product_elements = c.products.mixture.element_inventory_moles_per_kg()?;
    for i in 0..fuel_elements.len() {
        let incoming =
            Finite::new(z * fuel_elements[i].get() + (1.0 - z) * oxidizer_elements[i].get())?.get();
        if !controls
            .element_balance_moles_per_kg
            .accepts(incoming, product_elements[i].get())?
        {
            return Err(PhysicalError(
                "supplied products violate elemental mass balance",
            ));
        }
    }
    // Products are a supplied complete-reaction endpoint, not an equilibrium calculation.
    // Require no remaining net oxygen demand; atom balance alone does not ensure that.
    if !controls
        .element_balance_moles_per_kg
        .accepts(c.products.mixture.oxygen_demand_moles_per_kg()?.get(), 0.0)?
    {
        return Err(PhysicalError("products retain net oxygen demand"));
    }
    let chemical_release = Positive::new(
        z * formation[0].get() + (1.0 - z) * formation[1].get() - formation[2].get(),
    )?;
    let temperatures = [
        c.fuel.mixture.state().temperature_kelvin.get(),
        c.oxidizer.mixture.state().temperature_kelvin.get(),
    ];
    let inlet = [
        Finite::new(formation[0].get() + cp[0].get() * (temperatures[0] - tref.get()))?,
        Finite::new(formation[1].get() + cp[1].get() * (temperatures[1] - tref.get()))?,
    ];
    let (sheet_temperature, _, energy_residual) =
        enthalpy_temperature(z, z, tref.get(), inlet, formation, cp)?;
    if !controls
        .energy_balance_joule_per_kg
        .accepts(energy_residual.get(), 0.0)?
    {
        return Err(PhysicalError(
            "energy arithmetic residual exceeds declared tolerance",
        ));
    }
    let lo = temperatures[0]
        .min(temperatures[1])
        .min(sheet_temperature.get());
    let hi = temperatures[0]
        .max(temperatures[1])
        .max(sheet_temperature.get());
    for stream in streams {
        cp_mass(&stream.mixture, reference, lo, hi)?;
    }
    let state = ThermodynamicState {
        temperature_kelvin: sheet_temperature,
        pressure_pascal: pressure,
    };
    for input in [&c.burner_radius, &c.injected_mass_flow, &c.mass_diffusivity] {
        if input.reference() != reference {
            return Err(PhysicalError("input reference mismatch"));
        }
    }
    let radius = Positive::new(c.burner_radius.at(Unit::Metre, state)?)?;
    let mass_flow = Positive::new(c.injected_mass_flow.at(Unit::KilogramPerSecond, state)?)?;
    let gamma = Positive::new(c.mass_diffusivity.at(Unit::KilogramPerMetreSecond, state)?)?;
    for t in [lo, hi] {
        c.mass_diffusivity.at(
            Unit::KilogramPerMetreSecond,
            ThermodynamicState {
                temperature_kelvin: Positive::new(t)?,
                pressure_pascal: pressure,
            },
        )?;
    }
    let length = Positive::new(mass_flow.get() / p.solid_angle.get() / gamma.get())?;
    let injected_st = match c.mode {
        Mode::NormalFuelInjection => z,
        Mode::InverseOxidizerInjection => 1.0 - z,
    };
    let log = Positive::new(-(-injected_st).ln_1p())?;
    let sheet_radius = Positive::new(length.get() / log.get())?;
    if sheet_radius.get() <= radius.get() {
        return Err(PhysicalError(
            "no exterior reaction sheet for supplied flow and radius; not an extinction prediction",
        ));
    }
    let burner_fraction = Finite::new(-(-length.get() / radius.get()).exp_m1())?;
    let result = ConditionalReference {
        mode: c.mode,
        stoichiometric_fuel_mass_fraction: zst,
        reaction_sheet_radius_m: sheet_radius,
        sheet_temperature_kelvin: sheet_temperature,
        chemical_release_joule_per_kg_stoichiometric_mixture: chemical_release,
        burner_injected_stream_mass_fraction: burner_fraction,
        minimum_mass_flow_for_exterior_sheet_kg_per_s: Positive::new(
            p.solid_angle.get() * gamma.get() * radius.get() * log.get(),
        )?,
        energy_residual_joule_per_kg: energy_residual,
        gravity_m_per_s2: c.gravity_m_per_s2.clone(),
        radius,
        length,
        mass_flow,
        gamma,
        parameters: p,
        reference_temperature: tref,
        inlet_enthalpies: inlet,
        formation_enthalpies: formation,
        heat_capacities: cp,
        temperature_min: Positive::new(lo)?,
        temperature_max: Positive::new(hi)?,
        controls,
    };
    result.at_radius(radius)?;
    result.at_radius(sheet_radius)?;
    Ok(result)
}

impl ConditionalReference {
    /// Radial state on the assumed established burning branch, not a transient forecast.
    pub fn at_radius(&self, radius_m: Positive) -> Result<RadialState> {
        let r = radius_m.get();
        if r < self.radius.get() {
            return Err(PhysicalError("radius lies inside solid burner"));
        }
        let a_over_r = Positive::new(self.length.get() / r)?.get();
        let unburned_infinity_fraction = (-a_over_r).exp();
        // expm1 avoids cancellation in the far-field/small-flow limit.
        let w = Finite::new(-(-a_over_r).exp_m1())?.get();
        let dw = Finite::new(-a_over_r / r * unburned_infinity_fraction)?.get();
        let (z, dz) = match self.mode {
            Mode::NormalFuelInjection => (w, dw),
            Mode::InverseOxidizerInjection => (unburned_infinity_fraction, -dw),
        };
        let (temperature, fractions, energy_residual) = enthalpy_temperature(
            z,
            self.stoichiometric_fuel_mass_fraction.get(),
            self.reference_temperature.get(),
            self.inlet_enthalpies,
            self.formation_enthalpies,
            self.heat_capacities,
        )?;
        if !self
            .controls
            .energy_balance_joule_per_kg
            .accepts(energy_residual.get(), 0.0)?
            || temperature.get() < self.temperature_min.get()
            || temperature.get() > self.temperature_max.get()
        {
            return Err(PhysicalError(
                "radial energy calculation outside admitted bounds",
            ));
        }
        let area = Positive::new(self.parameters.solid_angle.get() * r * r)?.get();
        let flux = Positive::new(self.mass_flow.get() / area)?;
        let scalar_residual = Finite::new(
            self.mass_flow.get() * w - area * self.gamma.get() * dw - self.mass_flow.get(),
        )?;
        if !self
            .controls
            .scalar_flux_kg_per_s
            .accepts(scalar_residual.get(), 0.0)?
        {
            return Err(PhysicalError(
                "scalar flux conservation outside arithmetic tolerance",
            ));
        }
        Ok(RadialState {
            radius_m,
            fuel_origin_mass_fraction: Finite::new(z)?,
            fuel_origin_gradient_per_m: Finite::new(dz)?,
            reacted_product_mass_fraction: Finite::new(fractions[2])?,
            unreacted_fuel_stream_mass_fraction: Finite::new(fractions[0])?,
            unreacted_oxidizer_stream_mass_fraction: Finite::new(fractions[1])?,
            temperature_kelvin: temperature,
            outward_mass_flux_kg_per_m2_s: flux,
            injected_scalar_flow_residual_kg_per_s: scalar_residual,
        })
    }
}

/// Finite radial apparatus, with independently sourced constant material properties.
/// Density is frozen: thermal expansion feedback on momentum is not solved.
#[derive(Clone, Debug)]
pub struct RadialApparatus {
    pub face_radii_m: Vec<Positive>,
    pub geometry_provenance: PropertyProvenance,
    pub frozen_density: ScalarInput,
    pub planck_mean_absorption: ScalarInput,
    pub radiation_bath_temperature: ScalarInput,
    pub wall_temperature: ScalarInput,
    pub wall_conductance: ScalarInput,
    /// Required source/assumption justifying negligible reabsorption/scattering.
    /// The returned optical depth is a diagnostic, not an automatic qualification.
    pub optically_thin_support: PropertyProvenance,
}

#[derive(Clone, Debug)]
pub struct RadialInitialState {
    pub fuel_origin_mass_fraction: Vec<Finite>,
    /// Includes formation and sensible enthalpy in the common reference convention.
    pub total_enthalpy_joule_per_kg: Vec<Finite>,
    pub elapsed_seconds: Finite,
    pub time_origin: String,
    pub provenance: PropertyProvenance,
}

#[derive(Clone, Copy, Debug)]
pub struct ExplicitTimeGrid {
    pub step_seconds: Positive,
    pub steps: u32,
    pub energy_budget_joule: NumericalTolerance,
    pub scalar_budget_kg: NumericalTolerance,
}

#[derive(Clone, Debug)]
pub struct RadialEvolution {
    /// Computed values: model outputs, never copies of the input measurement status.
    pub state: EvolvedRadialState,
    pub initial_state_provenance: PropertyProvenance,
    pub temperatures_kelvin: Vec<Positive>,
    pub optical_depth: Finite,
    pub energy_change_joule: Finite,
    pub net_boundary_energy_joule: Finite,
    pub radiation_loss_joule: Finite,
    pub wall_loss_joule: Finite,
    pub energy_budget_residual_joule: Finite,
    pub scalar_budget_residual_kg: Finite,
}

#[derive(Clone, Debug)]
pub struct EvolvedRadialState {
    pub fuel_origin_mass_fraction: Vec<Finite>,
    pub total_enthalpy_joule_per_kg: Vec<Finite>,
    pub elapsed_seconds: Finite,
    pub time_origin: String,
}

/// Conservative explicit finite volumes for Z and total enthalpy, with a prescribed
/// stationary outward mass flow. Outer boundary supplies the ambient stream composition
/// and enthalpy for diffusion; exiting advective flux uses the last cell's state.
/// Every internal face flux cancels from the global budgets. No automatic timestep tuning.
/// Radiation is 4*kappa_P*sigma*(T^4-T_bath^4), conditional on optically thin gas.
/// Wall loss is H_wall*(T_first-T_wall), an explicitly supplied apparatus conductance.
pub fn evolve_radial(
    c: &Conditions,
    p: Parameters,
    controls: Controls,
    apparatus: &RadialApparatus,
    initial: &RadialInitialState,
    time: ExplicitTimeGrid,
) -> Result<RadialEvolution> {
    let reference = forward(c, p, controls)?;
    check_provenance(&apparatus.geometry_provenance)?;
    check_provenance(&apparatus.optically_thin_support)?;
    check_provenance(&initial.provenance)?;
    if initial.time_origin.trim().is_empty()
        || initial.elapsed_seconds.get() < 0.0
        || time.steps == 0
    {
        return Err(PhysicalError(
            "explicit time origin, nonnegative elapsed time and positive step count required",
        ));
    }
    let faces = &apparatus.face_radii_m;
    if faces.len() < 2
        || faces[0] != reference.radius
        || faces.windows(2).any(|w| w[0].get() >= w[1].get())
    {
        return Err(PhysicalError(
            "radial faces must increase from burner radius",
        ));
    }
    let n = faces.len() - 1;
    if initial.fuel_origin_mass_fraction.len() != n
        || initial.total_enthalpy_joule_per_kg.len() != n
    {
        return Err(PhysicalError("radial state and cell count differ"));
    }
    let ref_state = c.fuel.formation_enthalpy.reference().state;
    let check_input = |input: &ScalarInput, unit| -> Result<f64> {
        if input.reference() != c.fuel.formation_enthalpy.reference() {
            return Err(PhysicalError("apparatus property reference mismatch"));
        }
        input.at(unit, ref_state)
    };
    let density = Positive::new(check_input(
        &apparatus.frozen_density,
        Unit::KilogramPerCubicMetre,
    )?)?
    .get();
    let kappa = check_input(&apparatus.planck_mean_absorption, Unit::PerMetre)?;
    let bath = Positive::new(check_input(
        &apparatus.radiation_bath_temperature,
        Unit::Kelvin,
    )?)?
    .get();
    let wall = Positive::new(check_input(&apparatus.wall_temperature, Unit::Kelvin)?)?.get();
    let conductance = check_input(&apparatus.wall_conductance, Unit::WattPerKelvin)?;
    if kappa < 0.0 || conductance < 0.0 {
        return Err(PhysicalError("negative absorption or wall conductance"));
    }
    let optical_depth = Finite::new(kappa * (faces[n].get() - faces[0].get()))?;
    if optical_depth.get() >= 1.0 {
        return Err(PhysicalError(
            "optical depth at least unity contradicts optically thin closure",
        ));
    }
    let mut volume = Vec::with_capacity(n);
    let mut mass = Vec::with_capacity(n);
    let mut centres = Vec::with_capacity(n);
    for bounds in faces.windows(2) {
        let a = bounds[0].get();
        let b = bounds[1].get();
        let v = Positive::new(p.solid_angle.get() / 3.0 * (b - a) * (b * b + a * b + a * a))?.get();
        volume.push(v);
        mass.push(Positive::new(density * v)?.get());
        centres.push(Finite::new(a + (b - a) / 2.0)?.get());
    }
    // K_face = area * rho*D / centre spacing, kg/s.
    let mut diffusion = vec![0.0; n + 1];
    for i in 1..=n {
        let distance = if i == n {
            faces[n].get() - centres[n - 1]
        } else {
            centres[i] - centres[i - 1]
        };
        diffusion[i] = Positive::new(
            p.solid_angle.get() * faces[i].get() * faces[i].get() * reference.gamma.get()
                / distance,
        )?
        .get();
    }
    let dt = time.step_seconds.get();
    let mdot = reference.mass_flow.get();
    let (z_in, z_out, h_in, h_out) = match c.mode {
        Mode::NormalFuelInjection => (
            1.0,
            0.0,
            reference.inlet_enthalpies[0].get(),
            reference.inlet_enthalpies[1].get(),
        ),
        Mode::InverseOxidizerInjection => (
            0.0,
            1.0,
            reference.inlet_enthalpies[1].get(),
            reference.inlet_enthalpies[0].get(),
        ),
    };
    let mut z: Vec<f64> = initial
        .fuel_origin_mass_fraction
        .iter()
        .map(|x| x.get())
        .collect();
    let mut h: Vec<f64> = initial
        .total_enthalpy_joule_per_kg
        .iter()
        .map(|x| x.get())
        .collect();
    let total = |values: &[f64]| -> Result<f64> {
        Ok(Finite::new(values.iter().zip(&mass).map(|(v, m)| v * m).sum())?.get())
    };
    let initial_scalar = total(&z)?;
    let initial_energy = total(&h)?;
    let mut boundary_energy = 0.0;
    let mut boundary_scalar = 0.0;
    let mut radiation_loss = 0.0;
    let mut wall_loss = 0.0;
    let mut temperatures = Vec::with_capacity(n);
    let state_temperature = |z: f64, h: f64| -> Result<(Positive, f64)> {
        let (temperature, fractions, residual) = temperature_from_enthalpy(
            z,
            reference.stoichiometric_fuel_mass_fraction.get(),
            reference.reference_temperature.get(),
            h,
            reference.formation_enthalpies,
            reference.heat_capacities,
        )?;
        if !controls
            .energy_balance_joule_per_kg
            .accepts(residual.get(), 0.0)?
        {
            return Err(PhysicalError("transient enthalpy reconstruction residual"));
        }
        let state = ThermodynamicState {
            temperature_kelvin: temperature,
            pressure_pascal: ref_state.pressure_pascal,
        };
        for (input, unit) in [
            (&apparatus.frozen_density, Unit::KilogramPerCubicMetre),
            (&apparatus.planck_mean_absorption, Unit::PerMetre),
            (&c.mass_diffusivity, Unit::KilogramPerMetreSecond),
        ] {
            input.at(unit, state)?;
        }
        for stream in [&c.fuel, &c.oxidizer, &c.products] {
            cp_mass(
                &stream.mixture,
                c.fuel.formation_enthalpy.reference(),
                temperature.get(),
                temperature.get(),
            )?;
        }
        let cp: f64 = fractions
            .iter()
            .zip(reference.heat_capacities)
            .map(|(y, cp)| y * cp.get())
            .sum();
        Ok((temperature, Positive::new(cp)?.get()))
    };
    for _ in 0..time.steps {
        let mut losses = Vec::with_capacity(n);
        temperatures.clear();
        for i in 0..n {
            let (temperature, cp) = state_temperature(z[i], h[i])?;
            let t = temperature.get();
            temperatures.push(temperature);
            let wall_h = if i == 0 { conductance } else { 0.0 };
            // Monotonic explicit loss bound uses the hotter of cell and radiation bath.
            let radiation_slope = Finite::new(
                16.0 * kappa * p.stefan_boltzmann.get() * t.max(bath).powi(3) * volume[i] / cp,
            )?
            .get();
            let rate = Positive::new(
                (mdot + diffusion[i] + diffusion[i + 1] + radiation_slope + wall_h / cp) / mass[i],
            )?
            .get();
            if Finite::new(dt * rate)?.get() > 1.0 {
                return Err(PhysicalError(
                    "explicit timestep violates transport/thermal monotonicity bound",
                ));
            }
            let radiation = Finite::new(
                4.0 * kappa * p.stefan_boltzmann.get() * (t.powi(4) - bath.powi(4)) * volume[i],
            )?
            .get();
            let wall_power = Finite::new(wall_h * (t - wall))?.get();
            losses.push(Finite::new(radiation + wall_power)?.get());
            radiation_loss = Finite::new(radiation_loss + dt * radiation)?.get();
            wall_loss = Finite::new(wall_loss + dt * wall_power)?.get();
        }
        let mut fz = vec![0.0; n + 1];
        let mut fh = vec![0.0; n + 1];
        fz[0] = mdot * z_in;
        fh[0] = mdot * h_in;
        for i in 1..=n {
            let (zr, hr) = if i == n { (z_out, h_out) } else { (z[i], h[i]) };
            fz[i] = Finite::new(mdot * z[i - 1] - diffusion[i] * (zr - z[i - 1]))?.get();
            fh[i] = Finite::new(mdot * h[i - 1] - diffusion[i] * (hr - h[i - 1]))?.get();
        }
        boundary_energy = Finite::new(boundary_energy + dt * (fh[0] - fh[n]))?.get();
        boundary_scalar = Finite::new(boundary_scalar + dt * (fz[0] - fz[n]))?.get();
        for i in 0..n {
            z[i] = Finite::new(z[i] + dt * (fz[i] - fz[i + 1]) / mass[i])?.get();
            h[i] = Finite::new(h[i] + dt * (fh[i] - fh[i + 1] - losses[i]) / mass[i])?.get();
            // Never clip a failed composition or temperature to rescue a run.
            state_temperature(z[i], h[i])?;
        }
    }
    temperatures.clear();
    for i in 0..n {
        temperatures.push(state_temperature(z[i], h[i])?.0);
    }
    let energy_change = Finite::new(total(&h)? - initial_energy)?;
    let energy_residual =
        Finite::new(energy_change.get() - boundary_energy + radiation_loss + wall_loss)?;
    let scalar_residual = Finite::new(total(&z)? - initial_scalar - boundary_scalar)?;
    if !time
        .energy_budget_joule
        .accepts(energy_residual.get(), 0.0)?
        || !time.scalar_budget_kg.accepts(scalar_residual.get(), 0.0)?
    {
        return Err(PhysicalError(
            "global finite-volume conservation residual exceeds arithmetic budget",
        ));
    }
    Ok(RadialEvolution {
        state: EvolvedRadialState {
            fuel_origin_mass_fraction: z.into_iter().map(Finite::new).collect::<Result<_>>()?,
            total_enthalpy_joule_per_kg: h.into_iter().map(Finite::new).collect::<Result<_>>()?,
            elapsed_seconds: Finite::new(
                initial.elapsed_seconds.get() + dt * f64::from(time.steps),
            )?,
            time_origin: initial.time_origin.clone(),
        },
        initial_state_provenance: initial.provenance.clone(),
        temperatures_kelvin: temperatures,
        optical_depth,
        energy_change_joule: energy_change,
        net_boundary_energy_joule: Finite::new(boundary_energy)?,
        radiation_loss_joule: Finite::new(radiation_loss)?,
        wall_loss_joule: Finite::new(wall_loss)?,
        energy_budget_residual_joule: energy_residual,
        scalar_budget_residual_kg: scalar_residual,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_core::{
        ElementalComposition, FractionBasis, GasComponent, PropertyOrigin, PropertyUnit,
        SuppliedProperty,
    };

    fn f(x: f64) -> Finite {
        Finite::new(x).unwrap()
    }
    fn pos(x: f64) -> Positive {
        Positive::new(x).unwrap()
    }
    fn provenance() -> PropertyProvenance {
        PropertyProvenance {
            origin: PropertyOrigin::SyntheticFixture,
            source: "algebraic unit fixture, not NASA or a material database".into(),
            locator: "property_spherical_w0::tests".into(),
            dependencies: vec![],
            uncertainty: PropertyUncertainty::Unquantified,
        }
    }
    fn state(t: f64) -> ThermodynamicState {
        ThermodynamicState {
            temperature_kelvin: pos(t),
            pressure_pascal: pos(100000.0),
        }
    }
    fn reference() -> ReferenceState {
        ReferenceState {
            state: state(300.0),
            convention: EnthalpyConvention::IncludingFormation {
                reference_identity: "synthetic elemental reference at 300 K".into(),
            },
        }
    }
    fn domain() -> PropertyDomain {
        PropertyDomain {
            phase: Phase::Gas,
            minimum: state(100.0),
            maximum: state(5000.0),
        }
    }
    fn scalar(value: f64, unit: Unit) -> ScalarInput {
        ScalarInput::new(f(value), unit, domain(), reference(), provenance()).unwrap()
    }
    fn mixture(name: &str, h: u32, o: u32, molar_mass: f64) -> GasMixture {
        let elements = ElementalComposition {
            carbon: 0,
            hydrogen: h,
            oxygen: o,
            nitrogen: 0,
            sulfur: 0,
            helium: 0,
            argon: 0,
        };
        let property = |value, unit| {
            SuppliedProperty::new(pos(value), unit, domain(), reference(), provenance()).unwrap()
        };
        let component = GasComponent::new(
            name.into(),
            elements,
            property(molar_mass, PropertyUnit::KilogramPerMole),
            property(1000.0, PropertyUnit::JoulePerKilogramKelvin),
        )
        .unwrap();
        GasMixture::new(
            vec![(component, f(1.0))],
            FractionBasis::Mass,
            state(300.0),
            NumericalTolerance::new(0.0, 0.0).unwrap(),
        )
        .unwrap()
    }
    fn fixture(mode: Mode) -> Conditions {
        let zero = Vector3::new([f(0.0); 3], "laboratory".into()).unwrap();
        Conditions {
            mode,
            closure: BurningClosure::EstablishedFastChemistryAdiabaticEqualDiffusivity,
            burner_radius: scalar(1.0, Unit::Millimetre),
            injected_mass_flow: scalar(0.001, Unit::GramPerSecond),
            mass_diffusivity: scalar(0.00001, Unit::KilogramPerMetreSecond),
            fuel: MixtureThermo {
                mixture: mixture("synthetic hydrogen", 2, 0, 0.002),
                formation_enthalpy: scalar(0.0, Unit::JoulePerKilogram),
            },
            oxidizer: MixtureThermo {
                mixture: mixture("synthetic oxygen", 0, 2, 0.032),
                formation_enthalpy: scalar(0.0, Unit::JoulePerKilogram),
            },
            products: MixtureThermo {
                mixture: mixture("synthetic water vapor", 2, 1, 0.018),
                formation_enthalpy: scalar(-1000.0, Unit::KilojoulePerKilogram),
            },
            gravity_m_per_s2: zero.clone(),
            ambient_velocity_m_per_s: zero,
            gravity_provenance: provenance(),
            ambient_velocity_provenance: provenance(),
        }
    }
    fn parameters() -> Parameters {
        Parameters::from_literals(&[12.566370614359172, 1.0, 0.00000005670374419]).unwrap()
    }
    fn controls() -> Controls {
        Controls {
            element_balance_moles_per_kg: NumericalTolerance::new(1e-10, 1e-12).unwrap(),
            energy_balance_joule_per_kg: NumericalTolerance::new(1e-8, 0.0).unwrap(),
            scalar_flux_kg_per_s: NumericalTolerance::new(1e-18, 0.0).unwrap(),
        }
    }
    fn near(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() <= tol, "{a} differs from {b}");
    }

    #[test]
    fn spherical_flux_energy_and_normal_inverse_mapping() {
        let normal = forward(
            &fixture(Mode::NormalFuelInjection),
            parameters(),
            controls(),
        )
        .unwrap();
        let inverse = forward(
            &fixture(Mode::InverseOxidizerInjection),
            parameters(),
            controls(),
        )
        .unwrap();
        near(
            normal.stoichiometric_fuel_mass_fraction.get(),
            1.0 / 9.0,
            1e-15,
        );
        near(normal.sheet_temperature_kelvin.get(), 1300.0, 1e-10);
        near(
            normal.sheet_temperature_kelvin.get(),
            inverse.sheet_temperature_kelvin.get(),
            1e-10,
        );
        let a = normal.length.get();
        near(
            normal.reaction_sheet_radius_m.get(),
            a / (9.0_f64 / 8.0).ln(),
            1e-15,
        );
        near(
            inverse.reaction_sheet_radius_m.get(),
            a / 9.0_f64.ln(),
            1e-15,
        );
        for r in [0.001, 0.01, 0.1, 10.0] {
            let n = normal.at_radius(pos(r)).unwrap();
            let i = inverse.at_radius(pos(r)).unwrap();
            near(
                n.fuel_origin_mass_fraction.get() + i.fuel_origin_mass_fraction.get(),
                1.0,
                1e-15,
            );
            near(
                n.fuel_origin_gradient_per_m.get(),
                -i.fuel_origin_gradient_per_m.get(),
                1e-14,
            );
            near(n.injected_scalar_flow_residual_kg_per_s.get(), 0.0, 1e-18);
        }
        let sheet = normal.at_radius(normal.reaction_sheet_radius_m).unwrap();
        near(sheet.reacted_product_mass_fraction.get(), 1.0, 1e-14);
        near(sheet.temperature_kelvin.get(), 1300.0, 1e-9);
    }

    #[test]
    fn far_field_analytic_limit_and_flow_scaling() {
        let c = fixture(Mode::NormalFuelInjection);
        let a = forward(&c, parameters(), controls()).unwrap();
        let far = a.at_radius(pos(1e8)).unwrap();
        near(
            far.fuel_origin_mass_fraction.get() * 1e8 / a.length.get(),
            1.0,
            1e-9,
        );
        near(far.temperature_kelvin.get(), 300.0, 1e-5);
        let mut doubled = c;
        doubled.injected_mass_flow = scalar(0.000002, Unit::KilogramPerSecond);
        let b = forward(&doubled, parameters(), controls()).unwrap();
        near(
            b.reaction_sheet_radius_m.get(),
            2.0 * a.reaction_sheet_radius_m.get(),
            1e-14,
        );
        near(
            b.sheet_temperature_kelvin.get(),
            a.sheet_temperature_kelvin.get(),
            1e-10,
        );
    }

    #[test]
    fn units_metadata_reference_and_missing_closures_fail_closed() {
        let c = fixture(Mode::NormalFuelInjection);
        near(c.burner_radius.raw_value().get(), 1.0, 0.0);
        near(c.burner_radius.si_value().get(), 0.001, 0.0);
        let mut uncertainty = provenance();
        uncertainty.uncertainty = PropertyUncertainty::AbsoluteBound(f(2.0));
        let converted = ScalarInput::new(
            f(10.0),
            Unit::KilojoulePerKilogram,
            domain(),
            reference(),
            uncertainty,
        )
        .unwrap();
        assert_eq!(
            converted.si_uncertainty(),
            &PropertyUncertainty::AbsoluteBound(f(2000.0))
        );
        assert_eq!(
            converted.provenance().uncertainty,
            PropertyUncertainty::AbsoluteBound(f(2.0))
        );
        let mut bad = c.clone();
        bad.gravity_m_per_s2 =
            Vector3::new([f(0.0), f(0.000001), f(0.0)], "laboratory".into()).unwrap();
        assert!(forward(&bad, parameters(), controls())
            .unwrap_err()
            .0
            .contains("gravity"));
        let mut bad = c.clone();
        bad.products.formation_enthalpy.reference.convention = EnthalpyConvention::SensibleOnly {
            reference_identity: "wrong".into(),
        };
        assert!(forward(&bad, parameters(), controls()).is_err());
        let mut bad = c.clone();
        bad.mass_diffusivity.domain.maximum = state(1000.0);
        assert!(forward(&bad, parameters(), controls()).is_err());
        let mut bad = c.clone();
        bad.products.mixture = mixture("wrong water mass", 2, 1, 0.020);
        assert!(forward(&bad, parameters(), controls())
            .unwrap_err()
            .0
            .contains("elemental"));
        let mut bad = c.clone();
        bad.injected_mass_flow = scalar(1e-12, Unit::KilogramPerSecond);
        assert!(forward(&bad, parameters(), controls())
            .unwrap_err()
            .0
            .contains("exterior"));
        let mut bad = c;
        bad.mass_diffusivity = scalar(0.00001, Unit::Metre);
        assert!(forward(&bad, parameters(), controls()).is_err());
        assert!(Finite::new(f64::NAN).is_err());
        assert!(
            Parameters::from_literals(&[12.566370614359172, 0.8, 0.00000005670374419]).is_err()
        );
    }

    fn apparatus() -> RadialApparatus {
        RadialApparatus {
            face_radii_m: [0.001, 0.002, 0.003, 0.004, 0.005]
                .into_iter()
                .map(pos)
                .collect(),
            geometry_provenance: provenance(),
            frozen_density: scalar(1.0, Unit::KilogramPerCubicMetre),
            planck_mean_absorption: scalar(0.1, Unit::PerMetre),
            radiation_bath_temperature: scalar(300.0, Unit::Kelvin),
            wall_temperature: scalar(300.0, Unit::Kelvin),
            wall_conductance: scalar(0.001, Unit::WattPerKelvin),
            optically_thin_support: provenance(),
        }
    }
    fn initial() -> RadialInitialState {
        RadialInitialState {
            fuel_origin_mass_fraction: vec![f(1.0 / 9.0); 4],
            total_enthalpy_joule_per_kg: vec![f(0.0); 4],
            elapsed_seconds: f(0.0),
            time_origin: "synthetic established hot field at t=0".into(),
            provenance: provenance(),
        }
    }
    fn time() -> ExplicitTimeGrid {
        ExplicitTimeGrid {
            step_seconds: pos(0.00001),
            steps: 10,
            energy_budget_joule: NumericalTolerance::new(1e-12, 0.0).unwrap(),
            scalar_budget_kg: NumericalTolerance::new(1e-18, 0.0).unwrap(),
        }
    }

    #[test]
    fn transient_global_budgets_and_cooling_are_conservative() {
        for mode in [Mode::NormalFuelInjection, Mode::InverseOxidizerInjection] {
            let result = evolve_radial(
                &fixture(mode),
                parameters(),
                controls(),
                &apparatus(),
                &initial(),
                time(),
            )
            .unwrap();
            assert!(result.radiation_loss_joule.get() > 0.0);
            assert!(result.wall_loss_joule.get() > 0.0);
            assert!(result.temperatures_kelvin.iter().all(|t| t.get() < 1300.0));
            near(result.energy_budget_residual_joule.get(), 0.0, 1e-12);
            near(result.scalar_budget_residual_kg.get(), 0.0, 1e-18);
            near(result.state.elapsed_seconds.get(), 0.0001, 1e-18);
        }
    }

    #[test]
    fn zero_loss_limit_and_explicit_stability_gate() {
        let mut a = apparatus();
        a.planck_mean_absorption = scalar(0.0, Unit::PerMetre);
        a.wall_conductance = scalar(0.0, Unit::WattPerKelvin);
        let c = fixture(Mode::NormalFuelInjection);
        let result = evolve_radial(&c, parameters(), controls(), &a, &initial(), time()).unwrap();
        near(result.radiation_loss_joule.get(), 0.0, 0.0);
        near(result.wall_loss_joule.get(), 0.0, 0.0);
        assert!(result
            .state
            .total_enthalpy_joule_per_kg
            .iter()
            .all(|h| h.get() == 0.0));
        let mut t = time();
        t.step_seconds = pos(100.0);
        assert!(
            evolve_radial(&c, parameters(), controls(), &a, &initial(), t)
                .unwrap_err()
                .0
                .contains("timestep")
        );
        a.planck_mean_absorption = scalar(1000.0, Unit::PerMetre);
        assert!(
            evolve_radial(&c, parameters(), controls(), &a, &initial(), time())
                .unwrap_err()
                .0
                .contains("optical depth")
        );
    }

    #[test]
    fn transient_step_halving_has_first_order_convergence() {
        let c = fixture(Mode::NormalFuelInjection);
        let mut t = time();
        let coarse =
            evolve_radial(&c, parameters(), controls(), &apparatus(), &initial(), t).unwrap();
        t.step_seconds = pos(0.000005);
        t.steps = 20;
        let medium =
            evolve_radial(&c, parameters(), controls(), &apparatus(), &initial(), t).unwrap();
        t.step_seconds = pos(0.0000025);
        t.steps = 40;
        let fine =
            evolve_radial(&c, parameters(), controls(), &apparatus(), &initial(), t).unwrap();
        let distance = |a: &RadialEvolution, b: &RadialEvolution| {
            a.temperatures_kelvin
                .iter()
                .zip(&b.temperatures_kelvin)
                .map(|(x, y)| (x.get() - y.get()).abs())
                .sum::<f64>()
        };
        assert!(distance(&medium, &fine) < distance(&coarse, &medium));
    }
}

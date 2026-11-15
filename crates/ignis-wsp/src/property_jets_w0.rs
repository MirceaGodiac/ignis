//! Outcome-blind Astra W0: conditional, axisymmetric laminar jet/coflow transport.
//!
//! This is a Boussinesq boundary-layer calculation, not a qualified ACME release.
//! The complete-reaction/constant-cp caloric closure and unity scalar diffusivity
//! are explicit inputs. No fuel names, fitted constants, extinction thresholds,
//! optical mappings, or electric-force surrogates occur in this module.
//! Derivation, number inventory and reading extent: docs/wsp/research/astra_jets_w0.md.

use crate::physical_core::{
    AtomicMolarMasses, EnthalpyConvention, Finite, FractionBasis, GasComponent, GasMixture,
    NumericalTolerance, Phase, PhysicalError, Positive, PropertyDomain, PropertyProvenance,
    PropertyUncertainty, ReferenceState, Result, ThermodynamicState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    One,
    Metre,
    Millimetre,
    MetrePerSecond,
    MetrePerSecondSquared,
    SquareMetrePerSecond,
    Kelvin,
    Pascal,
    Kilopascal,
    JoulePerKilogram,
    KilogramPerCubicMetre,
    KilogramPerMole,
    VoltPerMetre,
}

/// Raw and SI are both retained. The two conversions are exact unit definitions.
/// A thermophysical property needs a gas domain; geometry has no material phase.
#[derive(Clone, Debug)]
pub struct Record {
    pub raw_value: Finite,
    pub raw_unit: Unit,
    pub provenance: PropertyProvenance,
    pub domain: Option<PropertyDomain>,
    pub reference_identity: String,
}

impl Record {
    pub fn si(&self, expected: Unit) -> Result<f64> {
        if self.provenance.source.trim().is_empty()
            || self.provenance.locator.trim().is_empty()
            || self.reference_identity.trim().is_empty()
            || self
                .provenance
                .dependencies
                .iter()
                .any(|s| s.trim().is_empty())
        {
            return Err(PhysicalError("missing input provenance/reference"));
        }
        if let PropertyUncertainty::AbsoluteBound(v) = self.provenance.uncertainty {
            if v.get() < 0.0 {
                return Err(PhysicalError("negative input uncertainty"));
            }
        }
        if let Some(d) = self.domain {
            if d.minimum.temperature_kelvin.get() > d.maximum.temperature_kelvin.get()
                || d.minimum.pressure_pascal.get() > d.maximum.pressure_pascal.get()
            {
                return Err(PhysicalError("inverted input property domain"));
            }
        }
        let value = match (self.raw_unit, expected) {
            (Unit::Millimetre, Unit::Metre) => self.raw_value.get() / 1000.0,
            (Unit::Kilopascal, Unit::Pascal) => self.raw_value.get() * 1000.0,
            (a, b) if a == b => self.raw_value.get(),
            _ => return Err(PhysicalError("input unit mismatch")),
        };
        Finite::new(value).map(Finite::get)
    }

    pub fn si_uncertainty_absolute_bound(&self, expected: Unit) -> Result<Option<f64>> {
        self.si(expected)?;
        match self.provenance.uncertainty {
            PropertyUncertainty::Unquantified => Ok(None),
            PropertyUncertainty::AbsoluteBound(v) => {
                let mut bound = self.clone();
                bound.raw_value = v;
                Ok(Some(bound.si(expected)?))
            }
        }
    }

    fn positive(&self, unit: Unit) -> Result<f64> {
        Positive::new(self.si(unit)?).map(Positive::get)
    }

    fn at_gas_state(&self, state: ThermodynamicState) -> Result<()> {
        match self.domain {
            Some(d) if d.contains(state, Phase::Gas) => Ok(()),
            _ => Err(PhysicalError(
                "missing gas property domain or extrapolation",
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Composition {
    /// Fractions retain their supplied basis, raw numbers, units and provenance.
    pub entries: Vec<(GasComponent, Record)>,
    pub basis: FractionBasis,
}

impl Composition {
    fn build(
        &self,
        state: ThermodynamicState,
        tolerance: NumericalTolerance,
    ) -> Result<GasMixture> {
        GasMixture::new(
            self.entries
                .iter()
                .map(|(c, y)| Ok((c.clone(), Finite::new(y.si(Unit::One)?)?)))
                .collect::<Result<_>>()?,
            self.basis,
            state,
            tolerance,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Stream {
    pub composition: Composition,
    pub temperature: Record,
    pub pressure: Record,
    pub axial_speed: Record,
}

impl Stream {
    fn state(&self) -> Result<ThermodynamicState> {
        Ok(ThermodynamicState {
            temperature_kelvin: Positive::new(self.temperature.si(Unit::Kelvin)?)?,
            pressure_pascal: Positive::new(self.pressure.si(Unit::Pascal)?)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct JetInput {
    pub jet: Stream,
    pub coflow: Stream,
    /// Complete oxidation products of the stoichiometric mixture. Verified against
    /// the incoming CHONS/He/Ar element inventory and supplied atomic masses.
    pub stoichiometric_products: Composition,
    /// Positive heat released per kg STOICHIOMETRIC MIXTURE at the common reference.
    /// A sensible-only enthalpy reference is required to prevent double counting.
    pub heat_release_st: Record,
    pub atomic_molar_masses_chons_he_ar: [Record; 7],
    pub inlet_radius: Record,
    /// Finite outer streamline with prescribed ambient state, not a solid wall.
    pub outer_inlet_radius: Record,
    pub common_scalar_diffusivity: Record,
    pub kinematic_viscosity: Record,
    pub reference_density: Record,
    pub axis: [Record; 3],
    pub gravity: [Record; 3],
    pub electric_field: [Record; 3],
    pub coordinate_frame: String,
    /// Caller-declared approximation budget, not a universal physical threshold.
    /// Actual ideal-gas density contrast is checked, reported and never clipped.
    pub maximum_boussinesq_density_change: Record,
    /// Must identify the assumption/source supporting the specified regime.
    pub steady_laminar_parabolic_assumption: String,
    pub common_diffusivity_unity_lewis_assumption: String,
    pub fast_chemistry_adiabatic_assumption: String,
}

/// These are the ONLY literal scientific numbers in the patch. Integers retain
/// their analytic role; pi is rounded to f64, not inferred from data.
#[derive(Clone, Copy, Debug)]
pub struct Coefficients {
    pi: f64,
    two: f64,
    four: f64,
}

impl Coefficients {
    pub fn from_literal_values(values: &[f64]) -> Result<Self> {
        if values != [3.141592653589793, 2.0, 4.0] {
            return Err(PhysicalError(
                "analytic constants differ from frozen literal patch",
            ));
        }
        Ok(Self {
            pi: values[0],
            two: values[1],
            four: values[2],
        })
    }
}

/// Numerical resource/accuracy choices. None is a fitted physical parameter.
#[derive(Clone, Copy, Debug)]
pub struct MarchOptions {
    pub jet_cells: usize,
    pub coflow_cells: usize,
    pub maximum_steps: usize,
    pub maximum_step_m: f64,
    pub cfl_fraction: f64,
    /// Absolute allowance only for dimensionless fractions/unit-axis checks.
    /// Dimensional mass, inventory and flux comparisons use relative tolerance.
    pub arithmetic_absolute_tolerance: f64,
    pub arithmetic_relative_tolerance: f64,
}

impl MarchOptions {
    fn tolerance(self) -> Result<NumericalTolerance> {
        if self.jet_cells == 0
            || self.coflow_cells == 0
            || self.maximum_steps == 0
            || !self.cfl_fraction.is_finite()
            || self.cfl_fraction <= 0.0
            || self.cfl_fraction > 1.0
        {
            return Err(PhysicalError(
                "invalid numerical resolution, budget or CFL fraction",
            ));
        }
        Positive::new(self.maximum_step_m)?;
        NumericalTolerance::new(
            self.arithmetic_absolute_tolerance,
            self.arithmetic_relative_tolerance,
        )
    }
}

#[derive(Clone, Debug)]
pub struct CaloricState {
    pub temperature_kelvin: f64,
    pub reacted_stoichiometric_mass_fraction: f64,
    pub remaining_jet_mass_fraction: f64,
    pub remaining_coflow_mass_fraction: f64,
    pub heat_capacity_j_per_kg_k: f64,
    pub molar_mass_kg_per_mol: f64,
    pub density_over_coflow: f64,
    pub energy_balance_residual_j_per_kg: f64,
}

#[derive(Clone, Debug)]
pub struct Station {
    pub axial_distance_m: f64,
    pub radius_edges_m: Vec<f64>,
    pub axial_speed_m_per_s: Vec<f64>,
    pub jet_origin_mass_fraction: Vec<f64>,
    pub temperature_kelvin: Vec<f64>,
    pub density_over_coflow: Vec<f64>,
    /// 2 D (dZ/dr)^2 only; axial contribution and extinction kinetics are absent.
    pub radial_scalar_dissipation_per_second_at_edges: Vec<f64>,
    /// Radial cell-centre brackets only. No visible/UV quantity is assigned.
    pub reaction_sheet_radius_brackets_m: Vec<[f64; 2]>,
}

#[derive(Clone, Debug)]
pub struct JetForecast {
    /// Original raw inputs/properties and their provenance remain bound to the result.
    /// This is an internal record; it is not a parallel wire contract.
    pub input_snapshot: JetInput,
    pub literal_coefficients: [f64; 3],
    pub station: Station,
    pub stoichiometric_jet_fraction: f64,
    pub gravity_parallel_m_per_s2: f64,
    pub gravity_vector_m_per_s2: [f64; 3],
    pub axis: [f64; 3],
    pub coordinate_frame: String,
    pub steps: usize,
    pub maximum_observed_density_change: f64,
    pub maximum_observed_streamline_slope: f64,
    pub maximum_embedded_temperature_difference_k: f64,
    pub initial_jet_origin_mass_flow_kg_per_s: f64,
    pub jet_origin_mass_flow_kg_per_s: f64,
    pub integrated_outer_scalar_diffusion_kg_per_s: f64,
    pub scalar_mass_balance_residual_kg_per_s: f64,
    pub momentum_balance_residual_newton: f64,
    pub inlet_jet_reynolds_number: f64,
    pub qualification: &'static str,
    pub limitations: Vec<&'static str>,
}

struct Prepared<'a> {
    input: &'a JetInput,
    jet: GasMixture,
    coflow: GasMixture,
    products: GasMixture,
    reference: ReferenceState,
    z_st: f64,
    q_st: f64,
    cp_j: f64,
    cp_c: f64,
    cp_p: f64,
    uj: f64,
    uc: f64,
    a: f64,
    outer: f64,
    diffusion: f64,
    viscosity: f64,
    rho: f64,
    gravity_parallel: f64,
    gravity: [f64; 3],
    axis: [f64; 3],
    density_budget: f64,
}

fn cp_mass(mix: &GasMixture) -> Result<f64> {
    Positive::new(
        mix.components()
            .iter()
            .zip(mix.mass_fractions())
            .map(|(c, y)| {
                Ok(y.get() * c.heat_capacity_molar()?.get() / c.molar_mass().value().get())
            })
            .collect::<Result<Vec<_>>>()?
            .iter()
            .sum(),
    )
    .map(Positive::get)
}

fn validate_product_species(component: &GasComponent) -> Result<()> {
    let e = component.elements();
    let counts = [
        e.carbon, e.hydrogen, e.oxygen, e.nitrogen, e.sulfur, e.helium, e.argon,
    ];
    if ![
        [1, 0, 2, 0, 0, 0, 0], // CO2, exact atom balance
        [0, 2, 1, 0, 0, 0, 0], // gas H2O
        [0, 0, 0, 2, 0, 0, 0], // N2
        [0, 0, 2, 0, 1, 0, 0], // SO2
        [0, 0, 0, 0, 0, 1, 0], // He
        [0, 0, 0, 0, 0, 0, 1], // Ar
    ]
    .contains(&counts)
    {
        return Err(PhysicalError(
            "product is outside complete CHONS/He/Ar oxidation closure",
        ));
    }
    Ok(())
}

impl<'a> Prepared<'a> {
    fn new(input: &'a JetInput, options: MarchOptions) -> Result<Self> {
        let tolerance = options.tolerance()?;
        let dimensioned_tolerance =
            NumericalTolerance::new(0.0, options.arithmetic_relative_tolerance)?;
        for identity in [
            &input.coordinate_frame,
            &input.steady_laminar_parabolic_assumption,
            &input.common_diffusivity_unity_lewis_assumption,
            &input.fast_chemistry_adiabatic_assumption,
        ] {
            if identity.trim().is_empty() {
                return Err(PhysicalError(
                    "missing frame or explicit physical assumption",
                ));
            }
        }
        let js = input.jet.state()?;
        let cs = input.coflow.state()?;
        if js.pressure_pascal != cs.pressure_pascal {
            return Err(PhysicalError("isobaric streams require the same pressure"));
        }
        let jet = input.jet.composition.build(js, tolerance)?;
        let coflow = input.coflow.composition.build(cs, tolerance)?;
        let reference = jet.components()[0].heat_capacity().reference().clone();
        if !matches!(
            reference.convention,
            EnthalpyConvention::SensibleOnly { .. }
        ) {
            return Err(PhysicalError(
                "separate heat release requires sensible-only enthalpy",
            ));
        }
        let products = input
            .stoichiometric_products
            .build(reference.state, tolerance)?;
        let mut atoms = Vec::new();
        for atom in &input.atomic_molar_masses_chons_he_ar {
            atoms.push(Positive::new(atom.si(Unit::KilogramPerMole)?)?);
        }
        let atomic = AtomicMolarMasses {
            carbon_kg_per_mol: atoms[0],
            hydrogen_kg_per_mol: atoms[1],
            oxygen_kg_per_mol: atoms[2],
            nitrogen_kg_per_mol: atoms[3],
            sulfur_kg_per_mol: atoms[4],
            helium_kg_per_mol: atoms[5],
            argon_kg_per_mol: atoms[6],
        };
        for mix in [&jet, &coflow, &products] {
            for c in mix.components() {
                if c.heat_capacity().reference() != &reference
                    || c.molar_mass().reference() != &reference
                {
                    return Err(PhysicalError(
                        "incompatible stream/product thermochemical reference",
                    ));
                }
                c.check_element_mass(atomic, dimensioned_tolerance)?;
                for other in [&jet, &coflow, &products] {
                    if let Some(same) = other
                        .components()
                        .iter()
                        .find(|o| o.identity() == c.identity())
                    {
                        if same != c {
                            return Err(PhysicalError(
                                "same species identity has conflicting stream/product properties",
                            ));
                        }
                    }
                }
            }
        }
        let demand_j = jet.oxygen_demand_moles_per_kg()?.get();
        let demand_c = coflow.oxygen_demand_moles_per_kg()?.get();
        if demand_j == 0.0 || demand_c == 0.0 || demand_j.signum() == demand_c.signum() {
            return Err(PhysicalError("streams lack opposing net oxygen demand"));
        }
        for (stream, signed_demand) in [(&jet, demand_j), (&coflow, demand_c)] {
            for (component, fraction) in stream.components().iter().zip(stream.mass_fractions()) {
                if fraction.get() > 0.0
                    && component.elements().oxygen_demand_moles_per_mole()?.get() * signed_demand
                        < 0.0
                {
                    return Err(PhysicalError(
                        "premixed opposing reactants need an inlet progress/chemistry closure",
                    ));
                }
            }
        }
        // This signed formula also handles an oxidizer jet inside a fuel coflow.
        let z_st = Finite::new(-demand_c / (demand_j - demand_c))?.get();
        if z_st <= 0.0 || z_st >= 1.0 {
            return Err(PhysicalError("degenerate stoichiometry"));
        }
        let ij = jet.element_inventory_moles_per_kg()?;
        let ic = coflow.element_inventory_moles_per_kg()?;
        let ip = products.element_inventory_moles_per_kg()?;
        for k in 0..ij.len() {
            if !dimensioned_tolerance
                .accepts(ip[k].get(), z_st * ij[k].get() + (1.0 - z_st) * ic[k].get())?
            {
                return Err(PhysicalError(
                    "products do not conserve stoichiometric element/mass inventory",
                ));
            }
        }
        for c in products.components() {
            validate_product_species(c)?;
        }
        let q_st = input.heat_release_st.positive(Unit::JoulePerKilogram)?;
        input.heat_release_st.at_gas_state(reference.state)?;
        let EnthalpyConvention::SensibleOnly { reference_identity } = &reference.convention else {
            unreachable!()
        };
        if &input.heat_release_st.reference_identity != reference_identity {
            return Err(PhysicalError(
                "heat release reference differs from sensible enthalpy reference",
            ));
        }
        if input.heat_release_st.provenance.dependencies.is_empty() {
            return Err(PhysicalError(
                "heat release must identify its reaction/composition dependency",
            ));
        }
        let axis = [
            input.axis[0].si(Unit::One)?,
            input.axis[1].si(Unit::One)?,
            input.axis[2].si(Unit::One)?,
        ];
        let norm2: f64 = axis.iter().map(|x| x * x).sum();
        if !tolerance.accepts(norm2, 1.0)? {
            return Err(PhysicalError("axis is not a unit vector"));
        }
        let gravity = [
            input.gravity[0].si(Unit::MetrePerSecondSquared)?,
            input.gravity[1].si(Unit::MetrePerSecondSquared)?,
            input.gravity[2].si(Unit::MetrePerSecondSquared)?,
        ];
        let gravity_parallel =
            Finite::new(axis.iter().zip(gravity).map(|(a, g)| a * g).sum())?.get();
        // Test vector components, rather than subtracting squared magnitudes.
        for k in 0..3 {
            if !dimensioned_tolerance.accepts(gravity[k], gravity_parallel * axis[k])? {
                return Err(PhysicalError(
                    "transverse gravity requires non-axisymmetric momentum closure",
                ));
            }
            if input.electric_field[k].si(Unit::VoltPerMetre)? != 0.0 {
                return Err(PhysicalError(
                    "electric response requires charge, force and energy closure",
                ));
            }
        }
        let density_budget = input.maximum_boussinesq_density_change.si(Unit::One)?;
        if density_budget <= 0.0 || density_budget >= 1.0 {
            return Err(PhysicalError(
                "Boussinesq approximation budget must be explicitly between zero and one",
            ));
        }
        let a = input.inlet_radius.positive(Unit::Metre)?;
        let outer = input.outer_inlet_radius.positive(Unit::Metre)?;
        if outer <= a {
            return Err(PhysicalError("coflow outer radius must exceed jet radius"));
        }
        let prepared = Self {
            cp_j: cp_mass(&jet)?,
            cp_c: cp_mass(&coflow)?,
            cp_p: cp_mass(&products)?,
            uj: input.jet.axial_speed.positive(Unit::MetrePerSecond)?,
            uc: input.coflow.axial_speed.positive(Unit::MetrePerSecond)?,
            diffusion: input
                .common_scalar_diffusivity
                .positive(Unit::SquareMetrePerSecond)?,
            viscosity: input
                .kinematic_viscosity
                .positive(Unit::SquareMetrePerSecond)?,
            rho: input
                .reference_density
                .positive(Unit::KilogramPerCubicMetre)?,
            input,
            jet,
            coflow,
            products,
            reference,
            z_st,
            q_st,
            a,
            outer,
            gravity_parallel,
            gravity,
            axis,
            density_budget,
        };
        input.reference_density.at_gas_state(cs)?;
        for z in [0.0, z_st, 1.0] {
            prepared.caloric(z)?;
        }
        Ok(prepared)
    }

    fn caloric(&self, z: f64) -> Result<CaloricState> {
        if !z.is_finite() || !(0.0..=1.0).contains(&z) {
            return Err(PhysicalError("mixture fraction outside [0,1]"));
        }
        let (reacted, jet_left, coflow_left) = if z <= self.z_st {
            let reacted = z / self.z_st;
            (reacted, 0.0, 1.0 - reacted)
        } else {
            let reacted = (1.0 - z) / (1.0 - self.z_st);
            (reacted, 1.0 - reacted, 0.0)
        };
        let cp =
            Positive::new(reacted * self.cp_p + jet_left * self.cp_j + coflow_left * self.cp_c)?
                .get();
        let tr = self.reference.state.temperature_kelvin.get();
        let incoming_h = z * self.cp_j * (self.jet.state().temperature_kelvin.get() - tr)
            + (1.0 - z) * self.cp_c * (self.coflow.state().temperature_kelvin.get() - tr);
        let outgoing_h = Finite::new(incoming_h + reacted * self.q_st)?.get();
        let temperature = Positive::new(tr + outgoing_h / cp)?.get();
        let state = ThermodynamicState {
            temperature_kelvin: Positive::new(temperature)?,
            pressure_pascal: self.jet.state().pressure_pascal,
        };
        // Even a zero-weight component is not silently extrapolated by this closure.
        for mix in [&self.jet, &self.coflow, &self.products] {
            for c in mix.components() {
                if !c.heat_capacity().domain().contains(state, Phase::Gas)
                    || !c.molar_mass().domain().contains(state, Phase::Gas)
                {
                    return Err(PhysicalError(
                        "caloric state exceeds a supplied gas property domain",
                    ));
                }
            }
        }
        self.input.common_scalar_diffusivity.at_gas_state(state)?;
        self.input.kinematic_viscosity.at_gas_state(state)?;
        let inverse_mass = reacted / self.products.molar_mass_kg_per_mol().get()
            + jet_left / self.jet.molar_mass_kg_per_mol().get()
            + coflow_left / self.coflow.molar_mass_kg_per_mol().get();
        let molar_mass = Positive::new(1.0 / inverse_mass)?.get();
        let density_ratio = Positive::new(
            molar_mass / self.coflow.molar_mass_kg_per_mol().get()
                * self.coflow.state().temperature_kelvin.get()
                / temperature,
        )?
        .get();
        if (density_ratio - 1.0).abs() > self.density_budget {
            return Err(PhysicalError(
                "density change exceeds declared Boussinesq domain; no clipping",
            ));
        }
        Ok(CaloricState {
            temperature_kelvin: temperature,
            reacted_stoichiometric_mass_fraction: reacted,
            remaining_jet_mass_fraction: jet_left,
            remaining_coflow_mass_fraction: coflow_left,
            heat_capacity_j_per_kg_k: cp,
            molar_mass_kg_per_mol: molar_mass,
            density_over_coflow: density_ratio,
            energy_balance_residual_j_per_kg: Finite::new(cp * (temperature - tr) - outgoing_h)?
                .get(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_core::{
        ElementalComposition, PropertyOrigin, PropertyUnit, SuppliedProperty,
    };

    // All numbers below are invented dimensioned fixtures, not material properties
    // or experimental observations. Atomic masses of one deliberately distinguish
    // this conservation exercise from a property database.
    fn f(v: f64) -> Finite {
        Finite::new(v).unwrap()
    }
    fn p(v: f64) -> Positive {
        Positive::new(v).unwrap()
    }
    fn state(t: f64, pressure: f64) -> ThermodynamicState {
        ThermodynamicState {
            temperature_kelvin: p(t),
            pressure_pascal: p(pressure),
        }
    }
    fn reference() -> ReferenceState {
        ReferenceState {
            state: state(10.0, 20.0),
            convention: EnthalpyConvention::SensibleOnly {
                reference_identity: "synthetic-sensible-zero".into(),
            },
        }
    }
    fn domain() -> PropertyDomain {
        PropertyDomain {
            phase: Phase::Gas,
            minimum: state(1.0, 1.0),
            maximum: state(100.0, 100.0),
        }
    }
    fn provenance() -> PropertyProvenance {
        PropertyProvenance {
            origin: PropertyOrigin::SyntheticFixture,
            source: "synthetic_fixture".into(),
            locator: "property_jets_w0::tests".into(),
            dependencies: vec!["invented-carbon-monoxide-oxygen-to-carbon-dioxide".into()],
            uncertainty: PropertyUncertainty::Unquantified,
        }
    }
    fn record(value: f64, unit: Unit) -> Record {
        Record {
            raw_value: f(value),
            raw_unit: unit,
            provenance: provenance(),
            domain: Some(domain()),
            reference_identity: "synthetic-sensible-zero".into(),
        }
    }
    fn component(identity: &str, carbon: u32, oxygen: u32, mass: f64) -> GasComponent {
        let property = |value, unit| {
            SuppliedProperty::new(p(value), unit, domain(), reference(), provenance()).unwrap()
        };
        GasComponent::new(
            identity.into(),
            ElementalComposition {
                carbon,
                hydrogen: 0,
                oxygen,
                nitrogen: 0,
                sulfur: 0,
                helium: 0,
                argon: 0,
            },
            property(mass, PropertyUnit::KilogramPerMole),
            property(1.0, PropertyUnit::JoulePerKilogramKelvin),
        )
        .unwrap()
    }
    fn composition(component: GasComponent) -> Composition {
        Composition {
            entries: vec![(component, record(1.0, Unit::One))],
            basis: FractionBasis::Mass,
        }
    }
    fn fixture() -> JetInput {
        let stream = |mix| Stream {
            composition: mix,
            temperature: record(10.0, Unit::Kelvin),
            pressure: record(20.0, Unit::Pascal),
            axial_speed: record(1.0, Unit::MetrePerSecond),
        };
        JetInput {
            jet: stream(composition(component("synthetic-CO", 1, 1, 2.0))),
            coflow: stream(composition(component("synthetic-O2", 0, 2, 2.0))),
            stoichiometric_products: composition(component("synthetic-CO2", 1, 2, 3.0)),
            heat_release_st: record(5.0, Unit::JoulePerKilogram),
            atomic_molar_masses_chons_he_ar: std::array::from_fn(|_| {
                record(1.0, Unit::KilogramPerMole)
            }),
            inlet_radius: record(1.0, Unit::Metre),
            outer_inlet_radius: record(4.0, Unit::Metre),
            common_scalar_diffusivity: record(0.1, Unit::SquareMetrePerSecond),
            kinematic_viscosity: record(0.1, Unit::SquareMetrePerSecond),
            reference_density: record(1.0, Unit::KilogramPerCubicMetre),
            axis: [
                record(0.0, Unit::One),
                record(0.0, Unit::One),
                record(1.0, Unit::One),
            ],
            gravity: std::array::from_fn(|_| record(0.0, Unit::MetrePerSecondSquared)),
            electric_field: std::array::from_fn(|_| record(0.0, Unit::VoltPerMetre)),
            coordinate_frame: "synthetic-laboratory".into(),
            maximum_boussinesq_density_change: record(0.1, Unit::One),
            steady_laminar_parabolic_assumption: "explicit synthetic boundary-layer scenario"
                .into(),
            common_diffusivity_unity_lewis_assumption:
                "explicit synthetic equal-diffusivity scenario".into(),
            fast_chemistry_adiabatic_assumption: "explicit synthetic complete-reaction scenario"
                .into(),
        }
    }
    fn coefficients() -> Coefficients {
        Coefficients::from_literal_values(&[3.141592653589793, 2.0, 4.0]).unwrap()
    }
    fn options() -> MarchOptions {
        MarchOptions {
            jet_cells: 8,
            coflow_cells: 32,
            maximum_steps: 100_000,
            maximum_step_m: 0.01,
            cfl_fraction: 0.6,
            arithmetic_absolute_tolerance: 1e-10,
            arithmetic_relative_tolerance: 1e-10,
        }
    }
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn caloric_element_mass_and_energy_limits_and_inverse_stream_transform() {
        let input = fixture();
        let prepared = Prepared::new(&input, options()).unwrap();
        close(prepared.z_st, 2.0 / 3.0);
        let peak = caloric_state(&input, prepared.z_st, options()).unwrap();
        close(peak.temperature_kelvin, 15.0);
        close(peak.reacted_stoichiometric_mass_fraction, 1.0);
        close(
            caloric_state(&input, 0.0, options())
                .unwrap()
                .temperature_kelvin,
            10.0,
        );
        close(
            caloric_state(&input, 1.0, options())
                .unwrap()
                .temperature_kelvin,
            10.0,
        );
        let mut inverse = input.clone();
        std::mem::swap(&mut inverse.jet, &mut inverse.coflow);
        let inverse_prepared = Prepared::new(&inverse, options()).unwrap();
        close(inverse_prepared.z_st, 1.0 - prepared.z_st);
        for z in [0.0, 0.25, 0.5, 2.0 / 3.0, 0.875, 1.0] {
            let normal = caloric_state(&input, z, options()).unwrap();
            let reverse = caloric_state(&inverse, 1.0 - z, options()).unwrap();
            close(normal.temperature_kelvin, reverse.temperature_kelvin);
            close(normal.energy_balance_residual_j_per_kg, 0.0);
            close(
                normal.reacted_stoichiometric_mass_fraction
                    + normal.remaining_jet_mass_fraction
                    + normal.remaining_coflow_mass_fraction,
                1.0,
            );
        }
        let mut invalid = input.clone();
        invalid.stoichiometric_products = invalid.jet.composition.clone();
        assert!(caloric_state(&invalid, 0.5, options()).is_err());
        invalid = input.clone();
        invalid.atomic_molar_masses_chons_he_ar[0] = record(2.0, Unit::KilogramPerMole);
        assert!(caloric_state(&invalid, 0.5, options()).is_err());
    }

    #[test]
    fn centerline_exact_tip_and_finite_disk_transport_converge_without_point_source_singularity() {
        let input = fixture();
        let (_, tip) =
            analytic_centerline_reference(&input, 1.0, coefficients(), options()).unwrap();
        let (at_tip, _) =
            analytic_centerline_reference(&input, tip, coefficients(), options()).unwrap();
        close(at_tip, 2.0 / 3.0);
        let (near_inlet, _) =
            analytic_centerline_reference(&input, 1e-6, coefficients(), options()).unwrap();
        close(near_inlet, 1.0);
        let (reference, _) =
            analytic_centerline_reference(&input, 2.0, coefficients(), options()).unwrap();
        let coarse = forward(&input, 2.0, coefficients(), options()).unwrap();
        let fine_options = MarchOptions {
            jet_cells: 16,
            coflow_cells: 64,
            maximum_step_m: 0.0025,
            ..options()
        };
        let fine = forward(&input, 2.0, coefficients(), fine_options).unwrap();
        let coarse_error = (coarse.station.jet_origin_mass_fraction[0] - reference).abs();
        let fine_error = (fine.station.jet_origin_mass_fraction[0] - reference).abs();
        assert!(
            fine_error < coarse_error,
            "mesh refinement failed: coarse={coarse_error}, fine={fine_error}"
        );
        assert!(
            fine_error < 0.02,
            "synthetic discretization check, not experimental accuracy"
        );
        for u in &fine.station.axial_speed_m_per_s {
            close(*u, 1.0);
        }
        close(
            fine.initial_jet_origin_mass_flow_kg_per_s,
            coefficients().pi,
        );
        close(fine.scalar_mass_balance_residual_kg_per_s, 0.0);
        close(fine.momentum_balance_residual_newton, 0.0);
        assert!(fine
            .station
            .jet_origin_mass_fraction
            .iter()
            .all(|z| (0.0..=1.0).contains(z)));
    }

    #[test]
    fn unequal_speeds_diffuse_momentum_and_retain_mass_with_an_open_outer_boundary() {
        let mut input = fixture();
        input.jet.axial_speed = record(2.0, Unit::MetrePerSecond);
        input.outer_inlet_radius = record(1.5, Unit::Metre);
        let result = forward(&input, 2.0, coefficients(), options()).unwrap();
        assert!(result.station.axial_speed_m_per_s[0] < 2.0);
        assert!(result
            .station
            .axial_speed_m_per_s
            .iter()
            .all(|u| *u >= 1.0 && *u <= 2.0));
        assert!(result.integrated_outer_scalar_diffusion_kg_per_s < 0.0);
        assert!(
            result.jet_origin_mass_flow_kg_per_s < result.initial_jet_origin_mass_flow_kg_per_s
        );
        close(result.scalar_mass_balance_residual_kg_per_s, 0.0);
        close(result.momentum_balance_residual_newton, 0.0);
        assert!(analytic_centerline_reference(&input, 1.0, coefficients(), options()).is_err());
    }

    #[test]
    fn axial_gravity_has_correct_sign_and_rotation_while_transverse_force_is_refused() {
        let mut up = fixture();
        up.gravity[2] = record(-1.0, Unit::MetrePerSecondSquared);
        let rising = forward(&up, 1.0, coefficients(), options()).unwrap();
        assert!(rising.station.axial_speed_m_per_s.iter().any(|u| *u > 1.0));
        assert!(rising.station.axial_speed_m_per_s.iter().all(|u| *u >= 1.0));
        let mut down = up.clone();
        down.gravity[2] = record(1.0, Unit::MetrePerSecondSquared);
        let falling = forward(&down, 1.0, coefficients(), options()).unwrap();
        assert!(falling.station.axial_speed_m_per_s.iter().any(|u| *u < 1.0));
        let mut rotated = up.clone();
        rotated.axis = [
            record(1.0, Unit::One),
            record(0.0, Unit::One),
            record(0.0, Unit::One),
        ];
        rotated.gravity = [
            record(-1.0, Unit::MetrePerSecondSquared),
            record(0.0, Unit::MetrePerSecondSquared),
            record(0.0, Unit::MetrePerSecondSquared),
        ];
        let rotation = forward(&rotated, 1.0, coefficients(), options()).unwrap();
        assert_eq!(
            rising.station.axial_speed_m_per_s,
            rotation.station.axial_speed_m_per_s
        );
        close(rising.momentum_balance_residual_newton, 0.0);
        rotated.gravity[2] = record(0.1, Unit::MetrePerSecondSquared);
        assert!(forward(&rotated, 1.0, coefficients(), options()).is_err());
    }

    #[test]
    fn raw_si_uncertainty_reference_domains_and_missing_inputs_fail_closed() {
        let input = fixture();
        let expected = forward(&input, 0.1, coefficients(), options()).unwrap();
        let mut converted = input.clone();
        converted.inlet_radius = record(1000.0, Unit::Millimetre);
        converted.inlet_radius.provenance.uncertainty = PropertyUncertainty::AbsoluteBound(f(2.0));
        converted.jet.pressure = record(0.02, Unit::Kilopascal);
        let actual = forward(&converted, 0.1, coefficients(), options()).unwrap();
        assert_eq!(
            expected.station.jet_origin_mass_fraction,
            actual.station.jet_origin_mass_fraction
        );
        close(
            converted
                .inlet_radius
                .si_uncertainty_absolute_bound(Unit::Metre)
                .unwrap()
                .unwrap(),
            0.002,
        );
        assert_eq!(converted.inlet_radius.raw_value.get(), 1000.0);
        let mut broken = input.clone();
        broken.heat_release_st.reference_identity = "different-reference".into();
        assert!(forward(&broken, 0.1, coefficients(), options()).is_err());
        broken = input.clone();
        broken
            .common_scalar_diffusivity
            .domain
            .as_mut()
            .unwrap()
            .maximum
            .temperature_kelvin = p(12.0);
        assert!(forward(&broken, 0.1, coefficients(), options()).is_err());
        broken = input.clone();
        broken.maximum_boussinesq_density_change = record(0.001, Unit::One);
        assert!(forward(&broken, 1.0, coefficients(), options()).is_err());
        broken = input.clone();
        broken.inlet_radius.provenance.locator.clear();
        assert!(forward(&broken, 0.1, coefficients(), options()).is_err());
        broken = input.clone();
        broken.electric_field[2] = record(1.0, Unit::VoltPerMetre);
        assert!(forward(&broken, 0.1, coefficients(), options()).is_err());
        broken = input.clone();
        broken.jet.axial_speed = record(0.0, Unit::MetrePerSecond);
        assert!(forward(&broken, 0.1, coefficients(), options()).is_err());
        assert!(forward(
            &input,
            1.0,
            coefficients(),
            MarchOptions {
                maximum_steps: 1,
                ..options()
            }
        )
        .is_err());
        assert!(forward(
            &input,
            1.0,
            coefficients(),
            MarchOptions {
                cfl_fraction: 1.1,
                ..options()
            }
        )
        .is_err());
        assert!(Finite::new(f64::NAN).is_err());
    }

    #[test]
    fn literal_spec_meta_and_patch_are_complete_and_valid() {
        use crate::assemble::{make_patch, materialize, PatchMeta};
        use ignis_core::contracts::{LiteralOp, ModelSpec};
        let spec: ModelSpec = serde_json::from_str(include_str!(
            "../../../configs/models/jets/property-w0/cleanroom-astra/model_spec.json"
        ))
        .unwrap();
        let meta: PatchMeta = serde_json::from_str(include_str!(
            "../../../configs/models/jets/property-w0/cleanroom-astra/patch_meta.json"
        ))
        .unwrap();
        let ops: Vec<LiteralOp> = serde_json::from_str(include_str!(
            "../../../configs/models/jets/property-w0/cleanroom-astra/literal_ops.json"
        ))
        .unwrap();
        spec.validate().unwrap();
        assert_eq!(meta.author.model_id, "gpt-6-astra");
        assert!(
            meta.exposure.datasets.is_empty()
                && meta.exposure.development_feedback.is_empty()
                && meta.exposure.prior_candidates.is_empty()
        );
        let patch = make_patch(&spec, meta, ops);
        patch.validate_against(&spec).unwrap();
        let tensors = materialize(&spec, &patch, None).unwrap();
        Coefficients::from_literal_values(&tensors["analytic_constants"].data).unwrap();
    }
}

/// Pure thermochemical closure; does not assert that a flame can stabilize or ignite.
pub fn caloric_state(input: &JetInput, z: f64, options: MarchOptions) -> Result<CaloricState> {
    Prepared::new(input, options)?.caloric(z)
}

/// Unbounded equal-speed zero-gravity analytical reference only. Geometry is the
/// stoichiometric REACTION SHEET, not lift-off, visible diameter or UV diameter.
pub fn analytic_centerline_reference(
    input: &JetInput,
    axial_distance_m: f64,
    coefficients: Coefficients,
    options: MarchOptions,
) -> Result<(f64, f64)> {
    let p = Prepared::new(input, options)?;
    if p.uj != p.uc || p.gravity.iter().any(|&g| g != 0.0) {
        return Err(PhysicalError(
            "analytic reference requires equal speeds and zero gravity",
        ));
    }
    Positive::new(axial_distance_m)?;
    let exponent =
        Positive::new(p.a * (p.a * p.uj / p.diffusion) / coefficients.four / axial_distance_m)?
            .get();
    let center = -(-exponent).exp_m1();
    let tip =
        Positive::new(-p.a * (p.a * p.uj / p.diffusion) / coefficients.four / (-p.z_st).ln_1p())?
            .get();
    Ok((Finite::new(center)?.get(), tip))
}

struct Grid {
    widths: Vec<f64>,
    centers: Vec<f64>,
    edges: Vec<f64>,
}

impl Grid {
    fn inlet(
        p: &Prepared<'_>,
        options: MarchOptions,
        c: Coefficients,
    ) -> Result<(Self, Vec<f64>, Vec<f64>)> {
        let n = options
            .jet_cells
            .checked_add(options.coflow_cells)
            .ok_or(PhysicalError("grid size overflow"))?;
        let mut widths = Vec::new();
        widths
            .try_reserve_exact(n)
            .map_err(|_| PhysicalError("grid allocation exceeds resources"))?;
        let jwidth = Positive::new(p.uj * p.a * p.a / c.two / options.jet_cells as f64)?.get();
        let cwidth = Positive::new(
            p.uc * (p.outer * p.outer - p.a * p.a) / c.two / options.coflow_cells as f64,
        )?
        .get();
        widths.extend(std::iter::repeat_n(jwidth, options.jet_cells));
        widths.extend(std::iter::repeat_n(cwidth, options.coflow_cells));
        let mut edges = vec![0.0];
        for width in &widths {
            edges.push(Positive::new(edges.last().unwrap() + width)?.get());
        }
        let centers = edges.windows(2).map(|e| (e[0] + e[1]) / c.two).collect();
        let mut u = vec![p.uj; options.jet_cells];
        u.extend(std::iter::repeat_n(p.uc, options.coflow_cells));
        let mut z = vec![1.0; options.jet_cells];
        z.extend(std::iter::repeat_n(0.0, options.coflow_cells));
        Ok((
            Self {
                widths,
                centers,
                edges,
            },
            u,
            z,
        ))
    }

    fn radial_edges(&self, u: &[f64], c: Coefficients) -> Result<Vec<f64>> {
        let mut squared = 0.0;
        let mut radius = vec![0.0];
        for (&width, &speed) in self.widths.iter().zip(u) {
            Positive::new(speed)?;
            squared = Positive::new(squared + c.two * width / speed)?.get();
            radius.push(squared.sqrt());
        }
        Ok(radius)
    }

    fn weighted_sum(&self, values: &[f64]) -> Result<f64> {
        Finite::new(self.widths.iter().zip(values).map(|(w, v)| w * v).sum()).map(Finite::get)
    }
}

struct Rates {
    du: Vec<f64>,
    dz: Vec<f64>,
    step_bound: f64,
    outer_scalar: f64,
    outer_momentum: f64,
    integrated_buoyancy: f64,
    max_density_change: f64,
    max_streamline_slope: f64,
}

fn rates(
    p: &Prepared<'_>,
    grid: &Grid,
    u: &[f64],
    z: &[f64],
    c: Coefficients,
    safety: f64,
) -> Result<Rates> {
    let n = u.len();
    let radius = grid.radial_edges(u, c)?;
    let mut conductance = vec![0.0; n + 1];
    for k in 1..n {
        let left = grid.edges[k] - grid.centers[k - 1];
        let right = grid.centers[k] - grid.edges[k];
        let uf = (right * u[k - 1] + left * u[k]) / (left + right);
        conductance[k] = Positive::new(radius[k] * radius[k] * uf / (left + right))?.get();
    }
    conductance[n] =
        Positive::new(radius[n] * radius[n] * p.uc / (grid.edges[n] - grid.centers[n - 1]))?.get();
    let mut fz = vec![0.0; n + 1];
    let mut fu = vec![0.0; n + 1];
    for k in 1..n {
        fz[k] = Finite::new(p.diffusion * conductance[k] * (z[k] - z[k - 1]))?.get();
        fu[k] = Finite::new(p.viscosity * conductance[k] * (u[k] - u[k - 1]))?.get();
    }
    fz[n] = Finite::new(-p.diffusion * conductance[n] * z[n - 1])?.get();
    fu[n] = Finite::new(p.viscosity * conductance[n] * (p.uc - u[n - 1]))?.get();
    let mut du = Vec::with_capacity(n);
    let mut dz = Vec::with_capacity(n);
    let mut bound = f64::INFINITY;
    let mut b_total = 0.0;
    let mut density_change: f64 = 0.0;
    for k in 0..n {
        let thermo = p.caloric(z[k])?;
        density_change = density_change.max((thermo.density_over_coflow - 1.0).abs());
        // Ambient hydrostatic pressure is subtracted: hot/light fluid rises
        // against the gravity vector. The axis dot product retains orientation.
        let buoyancy =
            Finite::new(p.gravity_parallel * (thermo.density_over_coflow - 1.0) / u[k])?.get();
        b_total += grid.widths[k] * buoyancy;
        let diff_rate = Positive::new(
            p.diffusion.max(p.viscosity) * (conductance[k] + conductance[k + 1]) / grid.widths[k],
        )?
        .get();
        // Split the positivity budget between diffusion and an adverse source.
        let adverse_rate = if buoyancy < 0.0 {
            -buoyancy / u[k]
        } else {
            0.0
        };
        bound = bound.min(safety / (diff_rate + adverse_rate));
        du.push(Finite::new((fu[k + 1] - fu[k]) / grid.widths[k] + buoyancy)?.get());
        dz.push(Finite::new((fz[k + 1] - fz[k]) / grid.widths[k])?.get());
    }
    let mut radial_derivative_integral = 0.0;
    let mut slope: f64 = 0.0;
    for k in 0..n {
        radial_derivative_integral -= grid.widths[k] * du[k] / u[k] / u[k];
        slope = slope.max((radial_derivative_integral / radius[k + 1]).abs());
    }
    Ok(Rates {
        du,
        dz,
        step_bound: Positive::new(bound)?.get(),
        outer_scalar: fz[n],
        outer_momentum: fu[n],
        integrated_buoyancy: Finite::new(b_total)?.get(),
        max_density_change: density_change,
        max_streamline_slope: Finite::new(slope)?.get(),
    })
}

fn euler(values: &[f64], derivatives: &[f64], step: f64) -> Result<Vec<f64>> {
    values
        .iter()
        .zip(derivatives)
        .map(|(v, d)| Finite::new(v + step * d).map(Finite::get))
        .collect()
}

/// Spatial forward solve only: no optimization, parameter inference or outcome access.
/// Marching coordinate is axial distance, not experimental or system time.
pub fn forward(
    input: &JetInput,
    axial_distance_m: f64,
    coefficients: Coefficients,
    options: MarchOptions,
) -> Result<JetForecast> {
    options.tolerance()?;
    let tolerance = NumericalTolerance::new(0.0, options.arithmetic_relative_tolerance)?;
    let p = Prepared::new(input, options)?;
    Positive::new(axial_distance_m)?;
    let (grid, mut u, mut z) = Grid::inlet(&p, options, coefficients)?;
    let initial_scalar = grid.weighted_sum(&z)?;
    let initial_momentum = grid.weighted_sum(&u)?;
    let mut outer_scalar = 0.0;
    let mut momentum_change = 0.0;
    let mut axial = 0.0;
    let mut steps = 0;
    let mut attempts = 0;
    let mut max_density: f64 = 0.0;
    let mut max_slope: f64 = 0.0;
    let mut max_temperature_difference: f64 = 0.0;
    while axial < axial_distance_m {
        let first = rates(&p, &grid, &u, &z, coefficients, options.cfl_fraction)?;
        let mut step = first
            .step_bound
            .min(options.maximum_step_m)
            .min(axial_distance_m - axial);
        let (next_u, next_z, second) = loop {
            attempts += 1;
            if attempts > options.maximum_steps {
                return Err(PhysicalError(
                    "forward resource budget exhausted before requested station",
                ));
            }
            if axial + step == axial || step <= 0.0 {
                return Err(PhysicalError("axial step underflow/stagnation"));
            }
            let stage_u = euler(&u, &first.du, step)?;
            let stage_z = euler(&z, &first.dz, step)?;
            let second = rates(
                &p,
                &grid,
                &stage_u,
                &stage_z,
                coefficients,
                options.cfl_fraction,
            )?;
            if step > second.step_bound {
                step = step.min(second.step_bound) / coefficients.two;
                continue;
            }
            let twice_u = euler(&stage_u, &second.du, step)?;
            let twice_z = euler(&stage_z, &second.dz, step)?;
            let next_u = u
                .iter()
                .zip(twice_u)
                .map(|(a, b)| Positive::new((a + b) / coefficients.two).map(Positive::get))
                .collect::<Result<Vec<_>>>()?;
            let next_z = z
                .iter()
                .zip(twice_z)
                .map(|(a, b)| Finite::new((a + b) / coefficients.two).map(Finite::get))
                .collect::<Result<Vec<_>>>()?;
            for (&new_z, &old_z) in next_z.iter().zip(&stage_z) {
                // An embedded diagnostic, not a rigorous error bound or tolerance-driven fit.
                let t_new = p.caloric(new_z)?.temperature_kelvin;
                let t_euler = p.caloric(old_z)?.temperature_kelvin;
                max_temperature_difference =
                    max_temperature_difference.max((t_new - t_euler).abs());
            }
            break (next_u, next_z, second);
        };
        outer_scalar += step * (first.outer_scalar + second.outer_scalar) / coefficients.two;
        momentum_change += step
            * (first.outer_momentum
                + first.integrated_buoyancy
                + second.outer_momentum
                + second.integrated_buoyancy)
            / coefficients.two;
        max_density = max_density
            .max(first.max_density_change)
            .max(second.max_density_change);
        max_slope = max_slope
            .max(first.max_streamline_slope)
            .max(second.max_streamline_slope);
        u = next_u;
        z = next_z;
        axial = Finite::new(axial + step)?.get();
        steps += 1;
    }
    let radius = grid.radial_edges(&u, coefficients)?;
    let mut dissipation = vec![0.0; u.len() + 1];
    for k in 1..u.len() {
        let left = grid.edges[k] - grid.centers[k - 1];
        let right = grid.centers[k] - grid.edges[k];
        let uf = (right * u[k - 1] + left * u[k]) / (left + right);
        let gradient = radius[k] * uf * (z[k] - z[k - 1]) / (left + right);
        dissipation[k] = Finite::new(coefficients.two * p.diffusion * gradient * gradient)?.get();
    }
    let last = u.len();
    let outer_gradient =
        -radius[last] * p.uc * z[last - 1] / (grid.edges[last] - grid.centers[last - 1]);
    dissipation[last] =
        Finite::new(coefficients.two * p.diffusion * outer_gradient * outer_gradient)?.get();
    let centers: Vec<f64> = radius
        .windows(2)
        .map(|r| ((r[0] * r[0] + r[1] * r[1]) / coefficients.two).sqrt())
        .collect();
    let mut brackets = Vec::new();
    for k in 1..z.len() {
        if (z[k - 1] - p.z_st) * (z[k] - p.z_st) <= 0.0 {
            brackets.push([centers[k - 1], centers[k]]);
        }
    }
    let thermo = z
        .iter()
        .map(|&v| p.caloric(v))
        .collect::<Result<Vec<_>>>()?;
    for cell in &thermo {
        max_density = max_density.max((cell.density_over_coflow - 1.0).abs());
    }
    let final_scalar = grid.weighted_sum(&z)?;
    let final_momentum = grid.weighted_sum(&u)?;
    let scale = Positive::new(coefficients.two * coefficients.pi * p.rho)?.get();
    let scalar_residual =
        Finite::new(scale * (final_scalar - initial_scalar - outer_scalar))?.get();
    let momentum_residual =
        Finite::new(scale * (final_momentum - initial_momentum - momentum_change))?.get();
    if !tolerance.accepts(final_scalar, initial_scalar + outer_scalar)?
        || !tolerance.accepts(final_momentum, initial_momentum + momentum_change)?
    {
        return Err(PhysicalError(
            "discrete mass/momentum balance exceeds arithmetic budget",
        ));
    }
    Ok(JetForecast {
        input_snapshot: input.clone(),
        literal_coefficients: [coefficients.pi, coefficients.two, coefficients.four],
        station: Station {
            axial_distance_m: axial, radius_edges_m: radius, axial_speed_m_per_s: u,
            jet_origin_mass_fraction: z, temperature_kelvin: thermo.iter().map(|s| s.temperature_kelvin).collect(),
            density_over_coflow: thermo.iter().map(|s| s.density_over_coflow).collect(),
            radial_scalar_dissipation_per_second_at_edges: dissipation,
            reaction_sheet_radius_brackets_m: brackets,
        },
        stoichiometric_jet_fraction: p.z_st, gravity_parallel_m_per_s2: p.gravity_parallel,
        gravity_vector_m_per_s2: p.gravity, axis: p.axis, coordinate_frame: input.coordinate_frame.clone(),
        steps, maximum_observed_density_change: max_density, maximum_observed_streamline_slope: max_slope,
        maximum_embedded_temperature_difference_k: max_temperature_difference,
        initial_jet_origin_mass_flow_kg_per_s: scale*initial_scalar,
        jet_origin_mass_flow_kg_per_s: scale*final_scalar,
        integrated_outer_scalar_diffusion_kg_per_s: scale*outer_scalar,
        scalar_mass_balance_residual_kg_per_s: scalar_residual,
        momentum_balance_residual_newton: momentum_residual,
        inlet_jet_reynolds_number: Positive::new(p.uj*p.a/p.viscosity)?.get(),
        qualification: "experimental_conditional_limit_unqualified",
        limitations: vec![
            "Steady axisymmetric laminar parabolic model; inlet rim, axial diffusion and pressure feedback unresolved.",
            "Boussinesq approximation: scalar/momentum inertia use reference density; only buoyancy uses density contrast.",
            "Signed axial gravity supported conditionally; transverse gravity and flow reversal unsupported.",
            "Complete oxidation, supplied heat release, constant caloric properties, unity Lewis and equal diffusivities; no dissociation/radiation/soot/finite-rate chemistry.",
            "Finite outer streamline is prescribed coflow; outer-domain and radial-grid errors are unestimated.",
            "SSPRK2 embedded Euler difference is a diagnostic, not a certified global error bound.",
            "Reaction-sheet brackets have no optical equivalence; no lift-off, extinction, ignition or event-time prediction.",
            "Empty reaction-sheet brackets do not prove absence: a near-axis sheet may be below the radial resolution.",
            "No electric response without charge transport, Poisson, force and energy closure.",
            "Input uncertainty is retained but not propagated; no probabilistic coverage or NASA qualification.",
        ],
    })
}

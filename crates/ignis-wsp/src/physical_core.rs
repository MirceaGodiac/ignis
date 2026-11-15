//! Internal, outcome-free physical algebra; not a release, chemistry closure, or shared contract.
//!
//! All dimensional inputs use named SI units. No property database or physical constant is
//! embedded here. Property values are supplied samples/declared constant-domain approximations,
//! never coefficients fitted or interpolated by this module. See `docs/wsp/measure/PHYSICAL_CORE.md`.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalError(pub &'static str);

impl fmt::Display for PhysicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for PhysicalError {}
pub type Result<T> = std::result::Result<T, PhysicalError>;

/// Finite scalar. Its dimensional meaning is provided by the containing named field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Finite(f64);

impl Finite {
    pub fn new(value: f64) -> Result<Self> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(PhysicalError("nonfinite input or arithmetic result"))
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Positive(Finite);

impl Positive {
    pub fn new(value: f64) -> Result<Self> {
        let value = Finite::new(value)?;
        if value.get() > 0.0 {
            Ok(Self(value))
        } else {
            Err(PhysicalError(
                "positive value required (zero/underflow refused)",
            ))
        }
    }

    pub fn get(self) -> f64 {
        self.0.get()
    }
}

/// Caller-declared arithmetic error bounds; never a physical/model parameter.
/// `absolute` has the units of the comparison. No implicit/default tolerance exists.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericalTolerance {
    absolute: Finite,
    relative: Finite,
}

impl NumericalTolerance {
    pub fn new(absolute: f64, relative: f64) -> Result<Self> {
        if absolute < 0.0 || relative < 0.0 {
            return Err(PhysicalError("negative arithmetic error bound"));
        }
        Ok(Self {
            absolute: Finite::new(absolute)?,
            relative: Finite::new(relative)?,
        })
    }

    pub fn accepts(self, a: f64, b: f64) -> Result<bool> {
        Finite::new(a)?;
        Finite::new(b)?;
        let bound = Finite::new(self.absolute.get() + self.relative.get() * a.abs().max(b.abs()))?;
        Ok(Finite::new(a - b)?.get().abs() <= bound.get())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Gas,
    Liquid,
    Solid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyUnit {
    KilogramPerMole,
    JoulePerMoleKelvin,
    JoulePerKilogramKelvin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyOrigin {
    SyntheticFixture,
    ExternalDetermination,
    ExplicitScenarioAssumption,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyUncertainty {
    Unquantified,
    /// Same units as the property; no probabilistic interpretation is inferred.
    AbsoluteBound(Finite),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyProvenance {
    pub origin: PropertyOrigin,
    pub source: String,
    pub locator: String,
    pub dependencies: Vec<String>,
    pub uncertainty: PropertyUncertainty,
}

impl PropertyProvenance {
    fn validate(&self) -> Result<()> {
        if self.source.trim().is_empty()
            || self.locator.trim().is_empty()
            || self.dependencies.iter().any(|id| id.trim().is_empty())
        {
            return Err(PhysicalError(
                "missing property source/locator/dependency identity",
            ));
        }
        if let PropertyUncertainty::AbsoluteBound(bound) = self.uncertainty {
            if bound.get() < 0.0 {
                return Err(PhysicalError("negative property uncertainty bound"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThermodynamicState {
    pub temperature_kelvin: Positive,
    pub pressure_pascal: Positive,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PropertyDomain {
    pub phase: Phase,
    pub minimum: ThermodynamicState,
    pub maximum: ThermodynamicState,
}

impl PropertyDomain {
    fn validate(self) -> Result<()> {
        if self.minimum.temperature_kelvin.get() > self.maximum.temperature_kelvin.get()
            || self.minimum.pressure_pascal.get() > self.maximum.pressure_pascal.get()
        {
            Err(PhysicalError("inverted property domain"))
        } else {
            Ok(())
        }
    }

    pub fn contains(self, state: ThermodynamicState, phase: Phase) -> bool {
        self.phase == phase
            && state.temperature_kelvin.get() >= self.minimum.temperature_kelvin.get()
            && state.temperature_kelvin.get() <= self.maximum.temperature_kelvin.get()
            && state.pressure_pascal.get() >= self.minimum.pressure_pascal.get()
            && state.pressure_pascal.get() <= self.maximum.pressure_pascal.get()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EnthalpyConvention {
    /// Identifier of the explicitly shared elemental reference convention.
    IncludingFormation { reference_identity: String },
    /// Sensible enthalpy zero at ReferenceState.state; chemical heat is separate.
    SensibleOnly { reference_identity: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceState {
    pub state: ThermodynamicState,
    pub convention: EnthalpyConvention,
}

impl ReferenceState {
    fn validate(&self) -> Result<()> {
        let (EnthalpyConvention::IncludingFormation { reference_identity }
        | EnthalpyConvention::SensibleOnly { reference_identity }) = &self.convention;
        if reference_identity.trim().is_empty() {
            Err(PhysicalError("missing thermochemical reference identity"))
        } else {
            Ok(())
        }
    }
}

/// Only these SI units are accepted: raw value/unit remain available without conversion loss.
/// A domain wider than a point is the supplier's explicit constant-property approximation.
#[derive(Clone, Debug, PartialEq)]
pub struct SuppliedProperty {
    value: Positive,
    unit: PropertyUnit,
    domain: PropertyDomain,
    reference: ReferenceState,
    provenance: PropertyProvenance,
}

impl SuppliedProperty {
    pub fn new(
        value: Positive,
        unit: PropertyUnit,
        domain: PropertyDomain,
        reference: ReferenceState,
        provenance: PropertyProvenance,
    ) -> Result<Self> {
        domain.validate()?;
        reference.validate()?;
        provenance.validate()?;
        Ok(Self {
            value,
            unit,
            domain,
            reference,
            provenance,
        })
    }

    pub fn value(&self) -> Positive {
        self.value
    }
    pub fn unit(&self) -> PropertyUnit {
        self.unit
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
}

/// Neutral CHONS/He/Ar species only. Helium and argon are retained as inert atoms,
/// not substituted with nitrogen. Charge and other elements are unsupported.
/// Counts are atoms per molecule/formula unit; properties are always supplied separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElementalComposition {
    pub carbon: u32,
    pub hydrogen: u32,
    pub oxygen: u32,
    pub nitrogen: u32,
    pub sulfur: u32,
    pub helium: u32,
    pub argon: u32,
}

impl ElementalComposition {
    fn counts(self) -> [f64; 7] {
        [
            self.carbon.into(),
            self.hydrogen.into(),
            self.oxygen.into(),
            self.nitrogen.into(),
            self.sulfur.into(),
            self.helium.into(),
            self.argon.into(),
        ]
    }

    fn validate(self) -> Result<()> {
        if self.counts().iter().all(|&v| v == 0.0) {
            Err(PhysicalError("empty elemental composition"))
        } else {
            Ok(())
        }
    }

    /// Signed mol O2 per mol species, for final products CO2/H2O/N2/SO2 with He/Ar
    /// retained unchanged. Neither noble gas contributes oxygen demand.
    /// Negative demand is oxygen excess. Integers 2 and 4 are atom balance, not fitted values.
    pub fn oxygen_demand_moles_per_mole(self) -> Result<Finite> {
        self.validate()?;
        Finite::new(
            f64::from(self.carbon) + f64::from(self.hydrogen) / 4.0 + f64::from(self.sulfur)
                - f64::from(self.oxygen) / 2.0,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GasComponent {
    identity: String,
    elements: ElementalComposition,
    molar_mass: SuppliedProperty,
    heat_capacity: SuppliedProperty,
}

impl GasComponent {
    pub fn new(
        identity: String,
        elements: ElementalComposition,
        molar_mass: SuppliedProperty,
        heat_capacity: SuppliedProperty,
    ) -> Result<Self> {
        if identity.trim().is_empty() {
            return Err(PhysicalError("missing species identity"));
        }
        elements.validate()?;
        if molar_mass.unit != PropertyUnit::KilogramPerMole
            || !matches!(
                heat_capacity.unit,
                PropertyUnit::JoulePerMoleKelvin | PropertyUnit::JoulePerKilogramKelvin
            )
        {
            return Err(PhysicalError("molar mass or heat-capacity unit mismatch"));
        }
        if molar_mass.domain.phase != Phase::Gas || heat_capacity.domain.phase != Phase::Gas {
            return Err(PhysicalError("gas component requires gas-phase properties"));
        }
        if molar_mass.reference != heat_capacity.reference {
            return Err(PhysicalError(
                "incompatible component property reference conventions",
            ));
        }
        Ok(Self {
            identity,
            elements,
            molar_mass,
            heat_capacity,
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn elements(&self) -> ElementalComposition {
        self.elements
    }
    pub fn molar_mass(&self) -> &SuppliedProperty {
        &self.molar_mass
    }
    pub fn heat_capacity(&self) -> &SuppliedProperty {
        &self.heat_capacity
    }

    fn require_state(&self, state: ThermodynamicState) -> Result<()> {
        if !self.molar_mass.domain.contains(state, Phase::Gas)
            || !self.heat_capacity.domain.contains(state, Phase::Gas)
        {
            return Err(PhysicalError("state outside a supplied property domain"));
        }
        Ok(())
    }

    pub fn heat_capacity_molar(&self) -> Result<Positive> {
        Positive::new(match self.heat_capacity.unit {
            PropertyUnit::JoulePerMoleKelvin => self.heat_capacity.value.get(),
            PropertyUnit::JoulePerKilogramKelvin => {
                self.heat_capacity.value.get() * self.molar_mass.value.get()
            }
            PropertyUnit::KilogramPerMole => {
                return Err(PhysicalError("heat-capacity unit mismatch"))
            }
        })
    }

    /// Optional physical consistency check using caller-supplied atomic masses in CHONS/He/Ar order.
    /// Atomic constants are not inferred from the species name or embedded in this module.
    pub fn check_element_mass(
        &self,
        atomic: AtomicMolarMasses,
        tolerance_kg_per_mol: NumericalTolerance,
    ) -> Result<()> {
        let expected = self
            .elements
            .counts()
            .into_iter()
            .zip(atomic.values())
            .map(|(n, m)| n * m.get())
            .sum();
        if tolerance_kg_per_mol.accepts(self.molar_mass.value.get(), expected)? {
            Ok(())
        } else {
            Err(PhysicalError(
                "molar mass inconsistent with supplied atomic masses",
            ))
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AtomicMolarMasses {
    pub carbon_kg_per_mol: Positive,
    pub hydrogen_kg_per_mol: Positive,
    pub oxygen_kg_per_mol: Positive,
    pub nitrogen_kg_per_mol: Positive,
    pub sulfur_kg_per_mol: Positive,
    pub helium_kg_per_mol: Positive,
    pub argon_kg_per_mol: Positive,
}

impl AtomicMolarMasses {
    fn values(self) -> [Positive; 7] {
        [
            self.carbon_kg_per_mol,
            self.hydrogen_kg_per_mol,
            self.oxygen_kg_per_mol,
            self.nitrogen_kg_per_mol,
            self.sulfur_kg_per_mol,
            self.helium_kg_per_mol,
            self.argon_kg_per_mol,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FractionBasis {
    Mole,
    Mass,
}

/// Sorted by species identity so summation order is independent of input permutation.
/// Duplicate identities and unnormalized inputs outside the explicit error budget are refused.
#[derive(Clone, Debug)]
pub struct GasMixture {
    components: Vec<GasComponent>,
    mole_fractions: Vec<Finite>,
    mass_fractions: Vec<Finite>,
    molar_mass_kg_per_mol: Positive,
    state: ThermodynamicState,
}

fn normalize(mut values: Vec<f64>) -> Result<Vec<Finite>> {
    if values.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(PhysicalError("invalid fraction"));
    }
    let sum = Positive::new(values.iter().sum())?.get();
    for v in &mut values {
        let before = *v;
        *v /= sum;
        if before > 0.0 && *v == 0.0 {
            return Err(PhysicalError("fraction underflow"));
        }
    }
    values.into_iter().map(Finite::new).collect()
}

impl GasMixture {
    pub fn new(
        mut entries: Vec<(GasComponent, Finite)>,
        basis: FractionBasis,
        state: ThermodynamicState,
        normalization_tolerance: NumericalTolerance,
    ) -> Result<Self> {
        if entries.is_empty() {
            return Err(PhysicalError("empty gas mixture"));
        }
        entries.sort_by(|a, b| a.0.identity.cmp(&b.0.identity));
        for pair in entries.windows(2) {
            if pair[0].0.identity == pair[1].0.identity {
                return Err(PhysicalError("duplicate species identity"));
            }
        }
        let reference = &entries[0].0.heat_capacity.reference;
        for (component, fraction) in &entries {
            if fraction.get() < 0.0 || fraction.get() > 1.0 {
                return Err(PhysicalError("fraction outside [0,1]"));
            }
            component.require_state(state)?;
            if &component.heat_capacity.reference != reference {
                return Err(PhysicalError("incompatible mixture reference conventions"));
            }
        }
        let sum: f64 = entries.iter().map(|(_, f)| f.get()).sum();
        if !normalization_tolerance.accepts(sum, 1.0)? {
            return Err(PhysicalError(
                "fractions not normalized within declared arithmetic budget",
            ));
        }
        let supplied = normalize(entries.iter().map(|(_, f)| f.get()).collect())?;
        let components: Vec<_> = entries.into_iter().map(|(c, _)| c).collect();
        let converted = normalize(
            components
                .iter()
                .zip(&supplied)
                .map(|(c, f)| {
                    let converted = match basis {
                        FractionBasis::Mole => f.get() * c.molar_mass.value.get(),
                        FractionBasis::Mass => f.get() / c.molar_mass.value.get(),
                    };
                    if f.get() > 0.0 && converted == 0.0 {
                        return Err(PhysicalError("fraction conversion underflow"));
                    }
                    Ok(converted)
                })
                .collect::<Result<_>>()?,
        )?;
        let (mole_fractions, mass_fractions) = match basis {
            FractionBasis::Mole => (supplied, converted),
            FractionBasis::Mass => (converted, supplied),
        };
        let molar_mass_kg_per_mol = Positive::new(
            components
                .iter()
                .zip(&mole_fractions)
                .map(|(c, x)| c.molar_mass.value.get() * x.get())
                .sum(),
        )?;
        Ok(Self {
            components,
            mole_fractions,
            mass_fractions,
            molar_mass_kg_per_mol,
            state,
        })
    }

    pub fn components(&self) -> &[GasComponent] {
        &self.components
    }
    pub fn mole_fractions(&self) -> &[Finite] {
        &self.mole_fractions
    }
    pub fn mass_fractions(&self) -> &[Finite] {
        &self.mass_fractions
    }
    pub fn molar_mass_kg_per_mol(&self) -> Positive {
        self.molar_mass_kg_per_mol
    }
    pub fn state(&self) -> ThermodynamicState {
        self.state
    }

    pub fn ideal_gas_properties(
        &self,
        gas_constant: MolarGasConstant,
    ) -> Result<IdealGasProperties> {
        let cp_molar = Positive::new(
            self.components
                .iter()
                .zip(&self.mole_fractions)
                .map(|(c, x)| {
                    let cp = c.heat_capacity_molar()?.get();
                    if cp <= gas_constant.joule_per_mole_kelvin.get() {
                        return Err(PhysicalError(
                            "ideal-gas component requires positive cv = cp - R",
                        ));
                    }
                    Ok(cp * x.get())
                })
                .collect::<Result<Vec<_>>>()?
                .iter()
                .sum(),
        )?;
        Ok(IdealGasProperties {
            density_kg_per_m3: Positive::new(
                self.state.pressure_pascal.get() * self.molar_mass_kg_per_mol.get()
                    / gas_constant.joule_per_mole_kelvin.get()
                    / self.state.temperature_kelvin.get(),
            )?,
            heat_capacity_joule_per_mole_kelvin: cp_molar,
            heat_capacity_joule_per_kg_kelvin: Positive::new(
                cp_molar.get() / self.molar_mass_kg_per_mol.get(),
            )?,
        })
    }

    pub fn oxygen_demand_moles_per_kg(&self) -> Result<Finite> {
        Finite::new(
            self.components
                .iter()
                .zip(&self.mass_fractions)
                .map(|(c, y)| {
                    Ok(y.get() * c.elements.oxygen_demand_moles_per_mole()?.get()
                        / c.molar_mass.value.get())
                })
                .collect::<Result<Vec<_>>>()?
                .iter()
                .sum(),
        )
    }

    /// Signed kg O2 / kg mixture. Requires component masses to agree with the supplied
    /// atomic masses before converting the molar demand to an oxygen mass balance.
    pub fn oxygen_demand_kg_per_kg(
        &self,
        atomic: AtomicMolarMasses,
        tolerance_kg_per_mol: NumericalTolerance,
    ) -> Result<Finite> {
        for component in &self.components {
            component.check_element_mass(atomic, tolerance_kg_per_mol)?;
        }
        Finite::new(self.oxygen_demand_moles_per_kg()?.get() * 2.0 * atomic.oxygen_kg_per_mol.get())
    }

    /// Inventory in CHONS/He/Ar order, mol atoms / kg mixture. No reaction is performed.
    pub fn element_inventory_moles_per_kg(&self) -> Result<[Finite; 7]> {
        let mut inventory = [0.0; 7];
        for (c, y) in self.components.iter().zip(&self.mass_fractions) {
            for (slot, atoms) in inventory.iter_mut().zip(c.elements.counts()) {
                *slot += y.get() * atoms / c.molar_mass.value.get();
            }
        }
        Ok([
            Finite::new(inventory[0])?,
            Finite::new(inventory[1])?,
            Finite::new(inventory[2])?,
            Finite::new(inventory[3])?,
            Finite::new(inventory[4])?,
            Finite::new(inventory[5])?,
            Finite::new(inventory[6])?,
        ])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MolarGasConstant {
    pub joule_per_mole_kelvin: Positive,
}

#[derive(Clone, Copy, Debug)]
pub struct IdealGasProperties {
    pub density_kg_per_m3: Positive,
    pub heat_capacity_joule_per_mole_kelvin: Positive,
    pub heat_capacity_joule_per_kg_kelvin: Positive,
}

/// Mass fraction of the first (net oxygen-demanding) stream. Requires opposing signed
/// demands and consistent species/reference definitions. Not an equilibrium/flame calculation.
pub fn stoichiometric_mixture_fraction(fuel: &GasMixture, oxidizer: &GasMixture) -> Result<Finite> {
    require_stream_compatibility(fuel, oxidizer)?;
    let a = fuel.oxygen_demand_moles_per_kg()?.get();
    let b = oxidizer.oxygen_demand_moles_per_kg()?.get();
    if a <= 0.0 || b >= 0.0 {
        return Err(PhysicalError(
            "two-stream stoichiometry requires positive fuel demand and negative oxidizer demand",
        ));
    }
    let z = Positive::new((-b) / Finite::new(a - b)?.get())?.get();
    if z >= 1.0 {
        return Err(PhysicalError("degenerate two-stream stoichiometry"));
    }
    Finite::new(z)
}

fn require_stream_compatibility(a: &GasMixture, b: &GasMixture) -> Result<()> {
    if a.components[0].heat_capacity.reference != b.components[0].heat_capacity.reference {
        return Err(PhysicalError("incompatible stream reference conventions"));
    }
    for first in &a.components {
        if let Some(second) = b.components.iter().find(|c| c.identity == first.identity) {
            if first != second {
                return Err(PhysicalError(
                    "same species identity has conflicting properties/provenance",
                ));
            }
        }
    }
    Ok(())
}

/// Composition-only mixing at an explicitly supplied output state. Does not solve enthalpy
/// balance or infer a mixed temperature; the caller must close that separate boundary problem.
pub fn mix_stream_compositions(
    a: &GasMixture,
    b: &GasMixture,
    mass_fraction_a: Finite,
    output_state: ThermodynamicState,
    tolerance: NumericalTolerance,
) -> Result<GasMixture> {
    require_stream_compatibility(a, b)?;
    let z = mass_fraction_a.get();
    if !(0.0..=1.0).contains(&z) {
        return Err(PhysicalError("stream mass fraction outside [0,1]"));
    }
    let mut entries: BTreeMap<String, (GasComponent, f64)> = BTreeMap::new();
    for (stream, weight) in [(a, z), (b, 1.0 - z)] {
        for (component, fraction) in stream.components.iter().zip(&stream.mass_fractions) {
            let contribution = weight * fraction.get();
            if weight > 0.0 && fraction.get() > 0.0 && contribution == 0.0 {
                return Err(PhysicalError(
                    "positive stream species contribution underflow",
                ));
            }
            let entry = entries
                .entry(component.identity.clone())
                .or_insert((component.clone(), 0.0));
            entry.1 += contribution;
        }
    }
    GasMixture::new(
        entries
            .into_values()
            .map(|(c, f)| Ok((c, Finite::new(f)?)))
            .collect::<Result<_>>()?,
        FractionBasis::Mass,
        output_state,
        tolerance,
    )
}

/// Cartesian components; coordinate frame is a required identity, not an inferred orientation.
#[derive(Clone, Debug, PartialEq)]
pub struct Vector3 {
    components: [Finite; 3],
    frame: String,
}

impl Vector3 {
    pub fn new(components: [Finite; 3], frame: String) -> Result<Self> {
        if frame.trim().is_empty() {
            return Err(PhysicalError("missing vector coordinate frame"));
        }
        Ok(Self { components, frame })
    }
    pub fn components(&self) -> [Finite; 3] {
        self.components
    }
    pub fn frame(&self) -> &str {
        &self.frame
    }
    pub fn magnitude(&self) -> Result<Finite> {
        Finite::new(
            self.components[0]
                .get()
                .hypot(self.components[1].get())
                .hypot(self.components[2].get()),
        )
    }
}

#[derive(Clone, Debug)]
pub struct TransportInputs {
    pub length_m: Positive,
    pub species_diffusivity_m2_per_s: Positive,
    pub kinematic_viscosity_m2_per_s: Positive,
    pub thermal_diffusivity_m2_per_s: Positive,
    pub velocity_m_per_s: Vector3,
    pub gravity_m_per_s2: Vector3,
    /// Signed (rho_reference - rho_comparison) / rho_reference; definition is caller-owned.
    pub density_contrast_over_reference: Finite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    Absent,
    Present,
}

#[derive(Clone, Debug)]
pub struct TransportGroups {
    pub diffusion_time_seconds: Positive,
    pub peclet: Finite,
    pub schmidt: Positive,
    pub prandtl: Positive,
    pub lewis: Positive,
    pub gravity_diffusion: Finite,
    /// Undefined at zero imposed velocity; no epsilon-denominator approximation.
    pub richardson: Option<Finite>,
    pub imposed_advection: Presence,
    pub gravity: Presence,
    pub density_contrast: Presence,
    /// Retained in full; scalar G alone does not describe direction-sensitive transport.
    pub gravity_m_per_s2: Vector3,
}

pub fn transport_groups(input: &TransportInputs) -> Result<TransportGroups> {
    if input.velocity_m_per_s.frame != input.gravity_m_per_s2.frame {
        return Err(PhysicalError(
            "velocity and gravity use different coordinate frames",
        ));
    }
    let l = input.length_m.get();
    let d = input.species_diffusivity_m2_per_s.get();
    let nu = input.kinematic_viscosity_m2_per_s.get();
    let alpha = input.thermal_diffusivity_m2_per_s.get();
    let speed = input.velocity_m_per_s.magnitude()?.get();
    let g = input.gravity_m_per_s2.magnitude()?.get();
    let contrast = input.density_contrast_over_reference.get();
    if contrast >= 1.0 {
        return Err(PhysicalError(
            "density contrast implies nonpositive comparison density",
        ));
    }
    let presence = |x: f64| {
        if x == 0.0 {
            Presence::Absent
        } else {
            Presence::Present
        }
    };
    let pe = if speed == 0.0 {
        0.0
    } else {
        Positive::new(speed * l / d)?.get()
    };
    let gravity_group = if g == 0.0 || contrast == 0.0 {
        0.0
    } else {
        Positive::new(g * contrast.abs() * (l / d) * (l / d) * l)?.get() * contrast.signum()
    };
    let richardson = if speed == 0.0 {
        None
    } else {
        // This form avoids dividing by a squared Pe that can overflow at strong advection.
        Some(Finite::new(if gravity_group == 0.0 {
            0.0
        } else {
            Positive::new(g * contrast.abs() * l / speed / speed)?.get() * contrast.signum()
        })?)
    };
    Ok(TransportGroups {
        diffusion_time_seconds: Positive::new(l * (l / d))?,
        peclet: Finite::new(pe)?,
        schmidt: Positive::new(nu / d)?,
        prandtl: Positive::new(nu / alpha)?,
        lewis: Positive::new(alpha / d)?,
        gravity_diffusion: Finite::new(gravity_group)?,
        richardson,
        imposed_advection: presence(speed),
        gravity: presence(g),
        density_contrast: presence(contrast),
        gravity_m_per_s2: input.gravity_m_per_s2.clone(),
    })
}

/// Require the supplied mass-averaged diffusive species fluxes to sum to zero in every
/// Cartesian direction. Fluxes are kg/(m2 s); no balancing species is silently adjusted.
pub fn check_diffusive_mass_fluxes(
    mixture: &GasMixture,
    fluxes: &BTreeMap<String, Vector3>,
    tolerance_kg_per_m2_s: NumericalTolerance,
) -> Result<()> {
    if fluxes.len() != mixture.components.len() {
        return Err(PhysicalError("diffusive flux species inventory mismatch"));
    }
    let mut total = [0.0; 3];
    let mut frame: Option<&str> = None;
    for component in &mixture.components {
        let flux = fluxes
            .get(&component.identity)
            .ok_or(PhysicalError("missing species diffusive flux"))?;
        if let Some(expected) = frame {
            if expected != flux.frame {
                return Err(PhysicalError("diffusive flux coordinate frame mismatch"));
            }
        }
        frame = Some(&flux.frame);
        for (sum, value) in total.iter_mut().zip(flux.components) {
            *sum += value.get();
        }
    }
    for residual in total {
        if !tolerance_kg_per_m2_s.accepts(residual, 0.0)? {
            return Err(PhysicalError(
                "diffusive mass fluxes do not conserve total mass",
            ));
        }
    }
    Ok(())
}

/// Prevent a separate complete-combustion heat source from double-counting formation
/// energy. This is a convention guard, not an energy-equation solver.
pub fn check_chemical_energy_convention(
    reference: &ReferenceState,
    separate_complete_combustion_heat: bool,
) -> Result<()> {
    reference.validate()?;
    if separate_complete_combustion_heat
        && matches!(
            reference.convention,
            EnthalpyConvention::IncludingFormation { .. }
        )
    {
        Err(PhysicalError(
            "formation enthalpy plus complete-combustion heat double-counts chemical energy",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Deliberately synthetic dimensioned properties; no real fuel/property table is used.
    fn p(x: f64) -> Positive {
        Positive::new(x).unwrap()
    }
    fn f(x: f64) -> Finite {
        Finite::new(x).unwrap()
    }
    // Explicit engineering error budget for these short f64 sums/conversions only.
    fn tol() -> NumericalTolerance {
        NumericalTolerance::new(1e-12, 1e-12).unwrap()
    }
    fn state() -> ThermodynamicState {
        ThermodynamicState {
            temperature_kelvin: p(10.0),
            pressure_pascal: p(20.0),
        }
    }
    fn reference() -> ReferenceState {
        ReferenceState {
            state: state(),
            convention: EnthalpyConvention::SensibleOnly {
                reference_identity: "synthetic-zero".into(),
            },
        }
    }
    fn property(value: f64, unit: PropertyUnit) -> SuppliedProperty {
        SuppliedProperty::new(
            p(value),
            unit,
            PropertyDomain {
                phase: Phase::Gas,
                minimum: state(),
                maximum: state(),
            },
            reference(),
            PropertyProvenance {
                origin: PropertyOrigin::SyntheticFixture,
                source: "synthetic_fixture".into(),
                locator: "physical_core::tests".into(),
                dependencies: vec![],
                uncertainty: PropertyUncertainty::Unquantified,
            },
        )
        .unwrap()
    }
    fn component(
        id: &str,
        elements: [u32; 5],
        mass: f64,
        cp: f64,
        unit: PropertyUnit,
    ) -> GasComponent {
        GasComponent::new(
            id.into(),
            ElementalComposition {
                carbon: elements[0],
                hydrogen: elements[1],
                oxygen: elements[2],
                nitrogen: elements[3],
                sulfur: elements[4],
                helium: 0,
                argon: 0,
            },
            property(mass, PropertyUnit::KilogramPerMole),
            property(cp, unit),
        )
        .unwrap()
    }
    fn a() -> GasComponent {
        component(
            "synthetic-a",
            [3, 2, 1, 4, 2],
            7.0,
            14.0,
            PropertyUnit::JoulePerMoleKelvin,
        )
    }
    fn b() -> GasComponent {
        component(
            "synthetic-b",
            [0, 0, 2, 0, 0],
            11.0,
            3.0,
            PropertyUnit::JoulePerKilogramKelvin,
        )
    }
    fn mixture(entries: Vec<(GasComponent, Finite)>, basis: FractionBasis) -> GasMixture {
        GasMixture::new(entries, basis, state(), tol()).unwrap()
    }
    fn close(a: f64, b: f64) {
        assert!(tol().accepts(a, b).unwrap(), "{a} != {b}");
    }

    #[test]
    fn composition_permutation_mass_mole_roundtrip_and_units() {
        let first = mixture(vec![(a(), f(0.25)), (b(), f(0.75))], FractionBasis::Mole);
        let reversed = mixture(vec![(b(), f(0.75)), (a(), f(0.25))], FractionBasis::Mole);
        assert_eq!(first.mole_fractions(), reversed.mole_fractions());
        assert_eq!(first.mass_fractions(), reversed.mass_fractions());
        let roundtrip = mixture(
            first
                .components()
                .iter()
                .cloned()
                .zip(first.mass_fractions().iter().copied())
                .collect(),
            FractionBasis::Mass,
        );
        for (x, recovered) in first
            .mole_fractions()
            .iter()
            .zip(roundtrip.mole_fractions())
        {
            close(x.get(), recovered.get());
        }
        let gas = first
            .ideal_gas_properties(MolarGasConstant {
                joule_per_mole_kelvin: p(2.0),
            })
            .unwrap();
        close(
            gas.heat_capacity_joule_per_mole_kelvin.get(),
            0.25 * 14.0 + 0.75 * 3.0 * 11.0,
        );
        close(
            gas.heat_capacity_joule_per_kg_kelvin.get(),
            first.mass_fractions()[0].get() * 2.0 + first.mass_fractions()[1].get() * 3.0,
        );
        close(
            gas.density_kg_per_m3.get(),
            20.0 * (0.25 * 7.0 + 0.75 * 11.0) / 2.0 / 10.0,
        );
        let mut renamed_a = a();
        renamed_a.identity = "renamed-z".into();
        let mut renamed_b = b();
        renamed_b.identity = "renamed-a".into();
        let renamed = mixture(
            vec![(renamed_a, f(0.25)), (renamed_b, f(0.75))],
            FractionBasis::Mole,
        )
        .ideal_gas_properties(MolarGasConstant {
            joule_per_mole_kelvin: p(2.0),
        })
        .unwrap();
        close(renamed.density_kg_per_m3.get(), gas.density_kg_per_m3.get());
        close(
            renamed.heat_capacity_joule_per_kg_kelvin.get(),
            gas.heat_capacity_joule_per_kg_kelvin.get(),
        );
        let mut wrong = a();
        wrong.heat_capacity.unit = PropertyUnit::KilogramPerMole;
        assert!(GasComponent::new(
            "wrong".into(),
            wrong.elements,
            wrong.molar_mass,
            wrong.heat_capacity
        )
        .is_err());
    }

    #[test]
    fn mixed_fuel_stoichiometry_and_element_conservation() {
        let inert = component(
            "synthetic-c",
            [0, 0, 0, 2, 0],
            13.0,
            5.0,
            PropertyUnit::JoulePerMoleKelvin,
        );
        let other_fuel = component(
            "synthetic-d",
            [1, 2, 0, 3, 1],
            17.0,
            8.0,
            PropertyUnit::JoulePerMoleKelvin,
        );
        let fuel = mixture(
            vec![
                (a(), f(0.5)),
                (other_fuel, f(0.25)),
                (inert.clone(), f(0.25)),
            ],
            FractionBasis::Mass,
        );
        let ox = mixture(vec![(b(), f(0.75)), (inert, f(0.25))], FractionBasis::Mass);
        close(
            a().elements.oxygen_demand_moles_per_mole().unwrap().get(),
            5.0,
        );
        let z = stoichiometric_mixture_fraction(&fuel, &ox).unwrap();
        let mixed = mix_stream_compositions(&fuel, &ox, z, state(), tol()).unwrap();
        close(mixed.oxygen_demand_moles_per_kg().unwrap().get(), 0.0);
        let fa = fuel.element_inventory_moles_per_kg().unwrap();
        let oa = ox.element_inventory_moles_per_kg().unwrap();
        let ma = mixed.element_inventory_moles_per_kg().unwrap();
        for i in 0..ma.len() {
            close(
                ma[i].get(),
                z.get() * fa[i].get() + (1.0 - z.get()) * oa[i].get(),
            );
        }
        assert!(stoichiometric_mixture_fraction(&ox, &fuel).is_err());
        assert!(stoichiometric_mixture_fraction(&fuel, &fuel).is_err());
        assert!(mix_stream_compositions(&fuel, &ox, f(1.1), state(), tol()).is_err());
    }

    #[test]
    fn invalid_compositions_and_nonfinite_values_fail_closed() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(Finite::new(value).is_err());
        }
        assert!(Positive::new(0.0).is_err());
        assert!(NumericalTolerance::new(0.0, f64::NAN).is_err());
        for entries in [
            vec![],
            vec![(a(), f(0.2)), (b(), f(0.3))],
            vec![(a(), f(-0.1)), (b(), f(1.0))],
            vec![(a(), f(0.5)), (a(), f(0.5))],
        ] {
            assert!(GasMixture::new(entries, FractionBasis::Mole, state(), tol()).is_err());
        }
    }

    #[test]
    fn arithmetic_extremes_and_nonphysical_ideal_gas_inputs_are_refused() {
        let one = mixture(vec![(a(), f(1.0))], FractionBasis::Mole);
        assert!(one
            .ideal_gas_properties(MolarGasConstant {
                joule_per_mole_kelvin: p(14.0)
            })
            .is_err());
        let mut tiny_mass = a();
        tiny_mass.molar_mass.value = p(f64::MIN_POSITIVE);
        assert!(GasMixture::new(
            vec![(tiny_mass, f(f64::MIN_POSITIVE)), (b(), f(1.0))],
            FractionBasis::Mole,
            state(),
            tol()
        )
        .is_err());
        let mut huge_state_component = a();
        let mut huge_state = state();
        huge_state.pressure_pascal = p(f64::MAX);
        huge_state_component.molar_mass.domain.maximum = huge_state;
        huge_state_component.heat_capacity.domain.maximum = huge_state;
        let huge = GasMixture::new(
            vec![(huge_state_component, f(1.0))],
            FractionBasis::Mole,
            huge_state,
            tol(),
        )
        .unwrap();
        assert!(huge
            .ideal_gas_properties(MolarGasConstant {
                joule_per_mole_kelvin: p(2.0)
            })
            .is_err());
        let mut input = transport();
        input.density_contrast_over_reference = f(1.0);
        assert!(transport_groups(&input).is_err());
    }

    #[test]
    fn stream_mixing_refuses_lost_positive_contributions_but_accepts_exact_zeros() {
        let trace = component(
            "synthetic-trace",
            [0, 0, 0, 2, 0],
            1.0,
            10.0,
            PropertyUnit::JoulePerMoleKelvin,
        );
        let bulk = component(
            "synthetic-bulk",
            [0, 0, 0, 2, 0],
            1.0,
            10.0,
            PropertyUnit::JoulePerMoleKelvin,
        );
        let trace_stream = |fraction| {
            mixture(
                vec![(trace.clone(), f(fraction)), (bulk.clone(), f(1.0))],
                FractionBasis::Mass,
            )
        };
        let dilute = trace_stream(1e-200);
        let pure = mixture(vec![(bulk.clone(), f(1.0))], FractionBasis::Mass);
        assert!(mix_stream_compositions(&dilute, &pure, f(1e-200), state(), tol()).is_err());
        for endpoint in [0.0, 1.0] {
            assert!(mix_stream_compositions(&dilute, &pure, f(endpoint), state(), tol()).is_ok());
        }
        assert!(mix_stream_compositions(&trace_stream(0.0), &pure, f(0.5), state(), tol()).is_ok());
        let representable =
            mix_stream_compositions(&trace_stream(1e-100), &pure, f(1e-100), state(), tol())
                .unwrap();
        let index = representable
            .components()
            .iter()
            .position(|c| c.identity() == "synthetic-trace")
            .unwrap();
        let actual = representable.mass_fractions()[index].get();
        assert!(actual > 0.0);
        close(actual / 1e-200, 1.0);
    }

    #[test]
    fn richardson_underflow_is_not_reported_as_physical_zero() {
        let mut input = transport();
        input.length_m = p(1.0);
        input.species_diffusivity_m2_per_s = p(1.0);
        input.kinematic_viscosity_m2_per_s = p(1.0);
        input.thermal_diffusivity_m2_per_s = p(1.0);
        input.velocity_m_per_s = vector(1e150, 0.0, 0.0);
        input.gravity_m_per_s2 = vector(1e-100, 0.0, 0.0);
        for contrast in [0.5, -0.5] {
            input.density_contrast_over_reference = f(contrast);
            assert!(transport_groups(&input).is_err());
        }
        input.gravity_m_per_s2 = vector(0.0, 0.0, 0.0);
        assert_eq!(
            transport_groups(&input).unwrap().richardson.unwrap().get(),
            0.0
        );
        input.gravity_m_per_s2 = vector(1e-100, 0.0, 0.0);
        input.density_contrast_over_reference = f(0.0);
        assert_eq!(
            transport_groups(&input).unwrap().richardson.unwrap().get(),
            0.0
        );
        input.density_contrast_over_reference = f(-0.5);
        input.velocity_m_per_s = vector(0.0, 0.0, 0.0);
        assert_eq!(transport_groups(&input).unwrap().richardson, None);
        input.velocity_m_per_s = vector(2.0, 0.0, 0.0);
        input.gravity_m_per_s2 = vector(4.0, 0.0, 0.0);
        close(
            transport_groups(&input).unwrap().richardson.unwrap().get(),
            -0.5,
        );
    }

    #[test]
    fn domains_phases_and_reference_conventions_are_not_silently_combined() {
        let mut out_of_domain = a();
        out_of_domain
            .heat_capacity
            .domain
            .minimum
            .temperature_kelvin = p(11.0);
        out_of_domain
            .heat_capacity
            .domain
            .maximum
            .temperature_kelvin = p(12.0);
        assert!(GasMixture::new(
            vec![(out_of_domain, f(1.0))],
            FractionBasis::Mole,
            state(),
            tol()
        )
        .is_err());
        let mut bad = b();
        bad.heat_capacity.reference.convention = EnthalpyConvention::IncludingFormation {
            reference_identity: "synthetic-elements".into(),
        };
        bad.molar_mass.reference = bad.heat_capacity.reference.clone();
        assert!(GasMixture::new(
            vec![(a(), f(0.5)), (bad.clone(), f(0.5))],
            FractionBasis::Mole,
            state(),
            tol()
        )
        .is_err());
        assert!(check_chemical_energy_convention(&bad.heat_capacity.reference, true).is_err());
        assert!(check_chemical_energy_convention(&bad.heat_capacity.reference, false).is_ok());
        assert!(check_chemical_energy_convention(&reference(), true).is_ok());
        let mut solid = property(1.0, PropertyUnit::KilogramPerMole);
        solid.domain.phase = Phase::Solid;
        assert!(GasComponent::new("solid".into(), a().elements, solid, a().heat_capacity).is_err());
        let mut conflicting = a();
        conflicting.molar_mass.value = p(9.0);
        let one = mixture(vec![(a(), f(1.0))], FractionBasis::Mass);
        let two = mixture(vec![(conflicting, f(1.0))], FractionBasis::Mass);
        assert!(mix_stream_compositions(&one, &two, f(0.5), state(), tol()).is_err());
    }

    fn vector(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3::new([f(x), f(y), f(z)], "synthetic-lab".into()).unwrap()
    }
    fn transport() -> TransportInputs {
        TransportInputs {
            length_m: p(2.0),
            species_diffusivity_m2_per_s: p(3.0),
            kinematic_viscosity_m2_per_s: p(5.0),
            thermal_diffusivity_m2_per_s: p(7.0),
            velocity_m_per_s: vector(0.0, 0.0, 0.0),
            gravity_m_per_s2: vector(0.0, 0.0, 0.0),
            density_contrast_over_reference: f(0.25),
        }
    }

    #[test]
    fn zero_limits_group_identities_and_vector_gravity_are_explicit() {
        let mut input = transport();
        let zero = transport_groups(&input).unwrap();
        assert_eq!(zero.peclet.get(), 0.0);
        assert_eq!(zero.gravity_diffusion.get(), 0.0);
        assert_eq!(zero.richardson, None);
        assert_eq!(zero.imposed_advection, Presence::Absent);
        close(zero.diffusion_time_seconds.get(), 4.0 / 3.0);
        close(zero.lewis.get(), zero.schmidt.get() / zero.prandtl.get());
        input.gravity_m_per_s2 = vector(0.0, -2.0, 0.0);
        let stationary = transport_groups(&input).unwrap();
        assert_eq!(stationary.richardson, None);
        assert_eq!(stationary.gravity, Presence::Present);
        assert_eq!(stationary.gravity_m_per_s2.components()[1].get(), -2.0);
        close(stationary.gravity_diffusion.get(), 2.0 * 0.25 * 8.0 / 9.0);
        input.velocity_m_per_s = vector(0.0, 4.0, 0.0);
        let moving = transport_groups(&input).unwrap();
        close(
            moving.richardson.unwrap().get(),
            moving.gravity_diffusion.get() / moving.peclet.get().powi(2),
        );
        input.gravity_m_per_s2 = vector(0.0, 0.0, 0.0);
        assert_eq!(
            transport_groups(&input).unwrap().richardson.unwrap().get(),
            0.0
        );
        input.gravity_m_per_s2.frame = "another-frame".into();
        assert!(transport_groups(&input).is_err());
    }

    #[test]
    fn conservation_boundaries_require_inventory_and_consistent_atomic_masses() {
        let mix = mixture(vec![(a(), f(0.5)), (b(), f(0.5))], FractionBasis::Mass);
        let mut fluxes = BTreeMap::from([
            ("synthetic-a".into(), vector(1.0, -2.0, 3.0)),
            ("synthetic-b".into(), vector(-1.0, 2.0, -3.0)),
        ]);
        check_diffusive_mass_fluxes(&mix, &fluxes, tol()).unwrap();
        fluxes.insert("synthetic-b".into(), vector(-1.0, 2.0, 0.0));
        assert!(check_diffusive_mass_fluxes(&mix, &fluxes, tol()).is_err());
        fluxes.remove("synthetic-b");
        assert!(check_diffusive_mass_fluxes(&mix, &fluxes, tol()).is_err());
        let atomic = AtomicMolarMasses {
            carbon_kg_per_mol: p(1.0),
            hydrogen_kg_per_mol: p(1.0),
            oxygen_kg_per_mol: p(1.0),
            nitrogen_kg_per_mol: p(1.0),
            sulfur_kg_per_mol: p(1.0),
            helium_kg_per_mol: p(1.0),
            argon_kg_per_mol: p(1.0),
        };
        assert!(a().check_element_mass(atomic, tol()).is_err());
        let consistent = component(
            "synthetic-consistent",
            [3, 2, 1, 4, 2],
            12.0,
            6.0,
            PropertyUnit::JoulePerMoleKelvin,
        );
        consistent.check_element_mass(atomic, tol()).unwrap();
        let consistent_mix = mixture(vec![(consistent, f(1.0))], FractionBasis::Mass);
        close(
            consistent_mix
                .oxygen_demand_kg_per_kg(atomic, tol())
                .unwrap()
                .get(),
            10.0 / 12.0,
        );
        assert!(mix.oxygen_demand_kg_per_kg(atomic, tol()).is_err());
    }

    #[test]
    fn helium_argon_diluents_keep_their_elements_properties_and_zero_oxygen_demand() {
        // These atomic masses and heat capacities are synthetic fixture values, not
        // helium/argon physical-property determinations or constants for production use.
        let atomic = AtomicMolarMasses {
            carbon_kg_per_mol: p(1.0),
            hydrogen_kg_per_mol: p(1.0),
            oxygen_kg_per_mol: p(1.0),
            nitrogen_kg_per_mol: p(1.0),
            sulfur_kg_per_mol: p(1.0),
            helium_kg_per_mol: p(3.0),
            argon_kg_per_mol: p(5.0),
        };
        let inert = |identity: &str, helium, argon, mass, cp| {
            GasComponent::new(
                identity.into(),
                ElementalComposition {
                    carbon: 0,
                    hydrogen: 0,
                    oxygen: 0,
                    nitrogen: 0,
                    sulfur: 0,
                    helium,
                    argon,
                },
                property(mass, PropertyUnit::KilogramPerMole),
                property(cp, PropertyUnit::JoulePerMoleKelvin),
            )
            .unwrap()
        };
        let helium = inert("synthetic-inert-a", 1, 0, 3.0, 12.0);
        let argon = inert("synthetic-inert-b", 0, 1, 5.0, 20.0);
        for component in [&helium, &argon] {
            assert_eq!(
                component
                    .elements()
                    .oxygen_demand_moles_per_mole()
                    .unwrap()
                    .get(),
                0.0
            );
            component.check_element_mass(atomic, tol()).unwrap();
        }
        let mut inconsistent_argon = argon.clone();
        inconsistent_argon.molar_mass.value = p(3.0);
        assert!(inconsistent_argon
            .check_element_mass(atomic, tol())
            .is_err());
        let inert_mix = mixture(
            vec![(helium.clone(), f(0.25)), (argon.clone(), f(0.75))],
            FractionBasis::Mass,
        );
        let inventory = inert_mix.element_inventory_moles_per_kg().unwrap();
        assert!(inventory[..5].iter().all(|count| count.get() == 0.0));
        close(inventory[5].get(), 0.25 / 3.0);
        close(inventory[6].get(), 0.75 / 5.0);
        assert_eq!(
            inert_mix
                .oxygen_demand_kg_per_kg(atomic, tol())
                .unwrap()
                .get(),
            0.0
        );
        let properties = inert_mix
            .ideal_gas_properties(MolarGasConstant {
                joule_per_mole_kelvin: p(2.0),
            })
            .unwrap();
        close(properties.heat_capacity_joule_per_kg_kelvin.get(), 4.0);
        close(
            properties.density_kg_per_m3.get(),
            1.0 / (0.25 / 3.0 + 0.75 / 5.0),
        );
        let roundtrip = mixture(
            inert_mix
                .components()
                .iter()
                .cloned()
                .zip(inert_mix.mole_fractions().iter().copied())
                .collect(),
            FractionBasis::Mole,
        );
        for (before, after) in inert_mix
            .mass_fractions()
            .iter()
            .zip(roundtrip.mass_fractions())
        {
            close(before.get(), after.get());
        }

        let fuel_component = component(
            "synthetic-reactive-a",
            [1, 2, 0, 0, 0],
            3.0,
            12.0,
            PropertyUnit::JoulePerMoleKelvin,
        );
        let oxidizer_component = component(
            "synthetic-reactive-b",
            [0, 0, 2, 0, 0],
            2.0,
            8.0,
            PropertyUnit::JoulePerMoleKelvin,
        );
        let fuel = mixture(
            vec![(fuel_component, f(0.5)), (helium, f(0.5))],
            FractionBasis::Mass,
        );
        let oxidizer = mixture(
            vec![(oxidizer_component, f(0.25)), (argon, f(0.75))],
            FractionBasis::Mass,
        );
        let z = stoichiometric_mixture_fraction(&fuel, &oxidizer).unwrap();
        close(z.get(), 1.0 / 3.0);
        assert!(stoichiometric_mixture_fraction(&fuel, &inert_mix).is_err());
        let mixed = mix_stream_compositions(&fuel, &oxidizer, z, state(), tol()).unwrap();
        close(
            mixed.oxygen_demand_kg_per_kg(atomic, tol()).unwrap().get(),
            0.0,
        );
        let first = fuel.element_inventory_moles_per_kg().unwrap();
        let second = oxidizer.element_inventory_moles_per_kg().unwrap();
        let result = mixed.element_inventory_moles_per_kg().unwrap();
        for i in 0..result.len() {
            close(
                result[i].get(),
                z.get() * first[i].get() + (1.0 - z.get()) * second[i].get(),
            );
        }
        assert!(result[5].get() > 0.0 && result[6].get() > 0.0);
    }
}

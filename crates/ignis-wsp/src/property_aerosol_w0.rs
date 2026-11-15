//! Private W0 aerosol science boundary. Not a wire contract, ModelSpec or release.
//! Standalone offline invariant tests: rustc --edition 2021 --test this-file.rs.
//! Coalescing compact homogeneous spheres; complete derivation/limits are in
//! docs/wsp/research/astra_aerosol_w0.md. No source yield or detector inversion.

use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::PI;

type Result<T> = std::result::Result<T, String>;

fn finite(x: f64) -> Result<f64> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err("nonfinite value or arithmetic result".into())
    }
}
fn nonnegative(x: f64) -> Result<f64> {
    finite(x)?;
    if x >= 0.0 {
        Ok(x)
    } else {
        Err("negative physical value".into())
    }
}
fn positive(x: f64) -> Result<f64> {
    finite(x)?;
    if x > 0.0 {
        Ok(x)
    } else {
        Err("positive value required; zero/underflow refused".into())
    }
}
fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    One,
    Kelvin,
    Pascal,
    KilogramPerCubicMetre,
    PascalSecond,
    Metre,
    SquareMetre,
    CubicMetre,
    CubicMetrePerSecond,
    Second,
    PerSecond,
    NumberPerSecond,
    NumberPerCubicMetre,
    MetrePerSecondSquared,
    Watt,
    Volt,
    VoltPerWatt,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Gas,
    Liquid,
    Solid,
    Apparatus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    SyntheticFixture,
    IndependentDetermination,
    ExplicitScenario,
}
#[derive(Clone, Debug)]
pub enum Uncertainty {
    Unquantified,
    AbsoluteBound(f64),
}

#[derive(Clone, Debug)]
pub struct Evidence {
    pub source: String,
    pub locator: String,
    /// Full transitive dependency closure, including raw measurement identities.
    pub dependencies: BTreeSet<String>,
    pub origin: Origin,
}
impl Evidence {
    fn check(&self, truth: &BTreeSet<String>) -> Result<()> {
        require(
            !self.source.trim().is_empty() && !self.locator.trim().is_empty(),
            "source/locator missing",
        )?;
        require(
            self.dependencies.iter().all(|s| !s.trim().is_empty()),
            "empty dependency",
        )?;
        require(
            !truth.contains(&self.source) && self.dependencies.is_disjoint(truth),
            "input derived from detector validation truth; independent prediction refused",
        )
    }
}
#[derive(Clone, Debug)]
pub struct Domain {
    pub temperature_kelvin: [f64; 2],
    pub pressure_pascal: [f64; 2],
    pub diameter_metres: [f64; 2],
    pub gas_composition_id: String,
    pub particle_composition_id: String,
}
#[derive(Clone, Debug)]
pub struct Sample {
    /// Raw SI value is retained unchanged; non-SI values are rejected, not converted tacitly.
    pub value: f64,
    pub unit: Unit,
    pub phase: Phase,
    pub domain: Domain,
    pub reference: String,
    pub uncertainty: Uncertainty,
    pub evidence: Evidence,
}
impl Sample {
    fn read(
        &self,
        context: &Context,
        unit: Unit,
        phase: Phase,
        diameter: Option<f64>,
    ) -> Result<f64> {
        finite(self.value)?;
        require(
            self.unit == unit && self.phase == phase,
            "property unit/phase mismatch",
        )?;
        require(
            !self.reference.trim().is_empty(),
            "property reference missing",
        )?;
        self.evidence.check(&context.detector_truth_dependencies)?;
        if let Uncertainty::AbsoluteBound(bound) = self.uncertainty {
            nonnegative(bound)?;
        }
        for range in [
            self.domain.temperature_kelvin,
            self.domain.pressure_pascal,
            self.domain.diameter_metres,
        ] {
            positive(range[0])?;
            positive(range[1])?;
            require(range[0] <= range[1], "inverted property domain")?;
        }
        let within = |value: f64, range: [f64; 2]| value >= range[0] && value <= range[1];
        require(
            within(context.temperature.value, self.domain.temperature_kelvin)
                && within(context.pressure.value, self.domain.pressure_pascal),
            "temperature/pressure outside property domain",
        )?;
        require(
            self.domain.gas_composition_id == context.gas_composition.identity
                && self.domain.particle_composition_id == context.particle_composition.identity,
            "property composition domain mismatch",
        )?;
        if let Some(d) = diameter {
            require(
                within(d, self.domain.diameter_metres),
                "diameter outside property domain",
            )?;
        }
        Ok(self.value)
    }
}
#[derive(Clone, Debug)]
pub struct Composition {
    pub identity: String,
    /// Explicit species mass fractions. No identity-based coefficient lookup exists.
    pub mass_fractions: BTreeMap<String, Sample>,
    pub evidence: Evidence,
}
impl Composition {
    fn check(&self, context: &Context, phase: Phase, tolerance: f64) -> Result<()> {
        require(
            !self.identity.trim().is_empty() && !self.mass_fractions.is_empty(),
            "composition missing",
        )?;
        self.evidence.check(&context.detector_truth_dependencies)?;
        let mut sum = 0.0;
        for (species, fraction) in &self.mass_fractions {
            require(!species.trim().is_empty(), "species identity missing")?;
            sum += nonnegative(fraction.read(context, Unit::One, phase, None)?)?;
        }
        require(
            (sum - 1.0).abs() <= tolerance,
            "mass fractions do not sum to one",
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GravityRegime {
    Microgravity,
    PartialGravity,
    TerrestrialOrOther,
}
#[derive(Clone, Debug)]
pub struct Context {
    pub temperature: Sample,
    pub pressure: Sample,
    pub gas_composition: Composition,
    pub particle_composition: Composition,
    pub particle_phase: Phase,
    pub gravity: [Sample; 3],
    pub coordinate_frame: String,
    pub gravity_regime: GravityRegime,
    pub condition_id: String,
    pub time_origin: String,
    pub detector_truth_dependencies: BTreeSet<String>,
}
impl Context {
    fn check(&self, tolerance: f64) -> Result<()> {
        require(
            matches!(self.particle_phase, Phase::Liquid | Phase::Solid),
            "particle phase must be explicit liquid or solid",
        )?;
        positive(
            self.temperature
                .read(self, Unit::Kelvin, Phase::Gas, None)?,
        )?;
        positive(self.pressure.read(self, Unit::Pascal, Phase::Gas, None)?)?;
        require(
            !self.condition_id.trim().is_empty()
                && !self.time_origin.trim().is_empty()
                && !self.coordinate_frame.trim().is_empty(),
            "condition/time origin/frame missing",
        )?;
        self.gas_composition.check(self, Phase::Gas, tolerance)?;
        self.particle_composition
            .check(self, self.particle_phase, tolerance)?;
        for g in &self.gravity {
            g.read(self, Unit::MetrePerSecondSquared, Phase::Apparatus, None)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Parameters {
    values: BTreeMap<String, f64>,
}
impl Parameters {
    /// Strict private scalar assignment loader: no expressions, code, defaults or unknown tensors.
    pub fn from_ops(text: &str) -> Result<Self> {
        let expected = [
            "boltzmann_j_per_k",
            "slip_alpha",
            "slip_beta",
            "slip_gamma",
            "maximum_continuum_kn",
            "maximum_settling_reynolds",
            "maximum_rayleigh_size",
        ];
        let mut values = BTreeMap::new();
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            let words: Vec<_> = line.split_whitespace().collect();
            require(
                words.len() == 4 && words[0] == "set" && words[2] == "[]",
                "only scalar literal set assignments accepted",
            )?;
            require(expected.contains(&words[1]), "unknown scalar")?;
            let value = words[3]
                .parse::<f64>()
                .map_err(|_| "nonliteral scalar".to_string())?;
            positive(value)?;
            require(
                values.insert(words[1].into(), value).is_none(),
                "duplicate assignment",
            )?;
        }
        require(
            values.len() == expected.len(),
            "incomplete literal inventory",
        )?;
        for key in [
            "maximum_continuum_kn",
            "maximum_settling_reynolds",
            "maximum_rayleigh_size",
        ] {
            require(
                values[key] < 1.0,
                "asymptotic domain threshold must be below unity",
            )?;
        }
        Ok(Self { values })
    }
    pub fn authored() -> Result<Self> {
        Self::from_ops(include_str!(
            "../../../configs/models/aerosol/property-w0/cleanroom-astra/constants.ops"
        ))
    }
    fn get(&self, key: &str) -> f64 {
        self.values[key]
    }
}
#[derive(Clone, Debug)]
pub enum DragLaw {
    ContinuumNoSlip,
    /// Published PSL/air coefficients; caller supplies condition-specific independent support.
    /// The supporting record must state any extrapolation; this does not qualify the output.
    PublishedSlipApproximation {
        applicability: Sample,
    },
}
#[derive(Clone, Debug)]
pub struct AerosolProperties {
    pub particle_density: Sample,
    pub gas_density: Sample,
    pub gas_viscosity: Sample,
    pub gas_mean_free_path: Sample,
    pub dilute_neutral_compact_coalescing_sphere_support: Sample,
    pub drag: DragLaw,
}
#[derive(Clone, Copy, Debug)]
struct Particle {
    diameter: f64,
    diffusion: f64,
    thermal_speed: f64,
    jump: f64,
    gravity_velocity: [f64; 3],
}
impl AerosolProperties {
    fn check(&self, context: &Context) -> Result<()> {
        positive(self.particle_density.read(
            context,
            Unit::KilogramPerCubicMetre,
            context.particle_phase,
            None,
        )?)?;
        positive(
            self.gas_density
                .read(context, Unit::KilogramPerCubicMetre, Phase::Gas, None)?,
        )?;
        positive(
            self.gas_viscosity
                .read(context, Unit::PascalSecond, Phase::Gas, None)?,
        )?;
        positive(
            self.gas_mean_free_path
                .read(context, Unit::Metre, Phase::Gas, None)?,
        )?;
        require(
            self.dilute_neutral_compact_coalescing_sphere_support.read(
                context,
                Unit::One,
                context.particle_phase,
                None,
            )? == 1.0,
            "dilute neutral compact coalescing sphere support missing",
        )?;
        if let DragLaw::PublishedSlipApproximation { applicability } = &self.drag {
            require(
                applicability.read(context, Unit::One, Phase::Apparatus, None)? == 1.0,
                "published slip approximation requires explicit condition support",
            )?;
        }
        Ok(())
    }
    fn particle(&self, volume: f64, context: &Context, p: &Parameters) -> Result<Particle> {
        let diameter = positive((6.0 * positive(volume)? / PI).cbrt())?;
        let rho = positive(self.particle_density.read(
            context,
            Unit::KilogramPerCubicMetre,
            context.particle_phase,
            Some(diameter),
        )?)?;
        let gas_rho = positive(self.gas_density.read(
            context,
            Unit::KilogramPerCubicMetre,
            Phase::Gas,
            Some(diameter),
        )?)?;
        let mu = positive(self.gas_viscosity.read(
            context,
            Unit::PascalSecond,
            Phase::Gas,
            Some(diameter),
        )?)?;
        let mean_free_path = positive(self.gas_mean_free_path.read(
            context,
            Unit::Metre,
            Phase::Gas,
            Some(diameter),
        )?)?;
        let kn = positive(2.0 * mean_free_path / diameter)?;
        let slip = match &self.drag {
            DragLaw::ContinuumNoSlip => {
                require(
                    kn <= p.get("maximum_continuum_kn"),
                    "no-slip continuum domain exceeded",
                )?;
                1.0
            }
            DragLaw::PublishedSlipApproximation { applicability } => {
                require(
                    applicability.read(context, Unit::One, Phase::Apparatus, Some(diameter))?
                        == 1.0,
                    "published slip approximation requires explicit condition support",
                )?;
                1.0 + kn
                    * (p.get("slip_alpha") + p.get("slip_beta") * (-p.get("slip_gamma") / kn).exp())
            }
        };
        let kt = positive(p.get("boltzmann_j_per_k") * context.temperature.value)?;
        let diffusion = positive(kt * slip / (3.0 * PI * mu * diameter))?;
        let thermal_speed = positive((8.0 * kt / (PI * rho * volume)).sqrt())?;
        let travel = positive(8.0 * diffusion / (PI * thermal_speed))?;
        let x = travel / diameter;
        let s = x.hypot(1.0);
        let jump = positive(
            diameter * x * (1.0 + 2.0 * s + x * (4.0 - 1.0 / (s + 1.0))) / (3.0 * (1.0 + x + s)),
        )?;
        let mut gravity_velocity = [0.0; 3];
        for (axis, velocity) in gravity_velocity.iter_mut().enumerate() {
            *velocity = finite(
                (rho - gas_rho) * diameter.powi(2) * slip * context.gravity[axis].value
                    / (18.0 * mu),
            )?;
        }
        let speed = gravity_velocity[0]
            .hypot(gravity_velocity[1])
            .hypot(gravity_velocity[2]);
        require(
            gas_rho * speed * diameter / mu <= p.get("maximum_settling_reynolds"),
            "settling creeping-flow domain exceeded",
        )?;
        Ok(Particle {
            diameter,
            diffusion,
            thermal_speed,
            jump,
            gravity_velocity,
        })
    }
}
fn collision_kernel(a: Particle, b: Particle) -> Result<f64> {
    let diameter = a.diameter + b.diameter;
    let diffusion = a.diffusion + b.diffusion;
    positive(
        2.0 * PI * diameter * diffusion
            / (diameter / (diameter + 2.0 * a.jump.hypot(b.jump))
                + 8.0 * diffusion / (a.thermal_speed.hypot(b.thermal_speed) * diameter)),
    )
}

#[derive(Clone, Debug)]
pub struct AbsorbingFace {
    pub area: Sample,
    pub film_thickness: Sample,
    pub outward_normal: [Sample; 3],
    pub coordinate_frame: String,
}
#[derive(Clone, Debug)]
pub struct Chamber {
    pub volume: Sample,
    /// Total balanced carrier inflow/exhaust at chamber conditions, including the source line.
    /// Incoming particles are supplied separately; this is not standard volume flow.
    pub balanced_throughflow: Sample,
    pub absorbing_faces: Vec<AbsorbingFace>,
    pub well_mixed_constant_volume_support: Sample,
}
impl Chamber {
    fn check(&self, context: &Context, tolerance: f64) -> Result<()> {
        positive(
            self.volume
                .read(context, Unit::CubicMetre, Phase::Apparatus, None)?,
        )?;
        nonnegative(self.balanced_throughflow.read(
            context,
            Unit::CubicMetrePerSecond,
            Phase::Gas,
            None,
        )?)?;
        require(
            self.well_mixed_constant_volume_support.read(
                context,
                Unit::One,
                Phase::Apparatus,
                None,
            )? == 1.0,
            "well-mixed constant-volume support missing",
        )?;
        for face in &self.absorbing_faces {
            require(
                face.coordinate_frame == context.coordinate_frame,
                "wall/gravity coordinate frames differ",
            )?;
            positive(
                face.area
                    .read(context, Unit::SquareMetre, Phase::Apparatus, None)?,
            )?;
            positive(
                face.film_thickness
                    .read(context, Unit::Metre, Phase::Apparatus, None)?,
            )?;
            let mut norm2 = 0.0;
            for normal in &face.outward_normal {
                norm2 += normal
                    .read(context, Unit::One, Phase::Apparatus, None)?
                    .powi(2);
            }
            require(
                (norm2 - 1.0).abs() <= tolerance,
                "wall normal must be unit length",
            )?;
        }
        Ok(())
    }
    fn rates(&self, particle: Particle, context: &Context, tolerance: f64) -> Result<(f64, f64)> {
        let volume =
            positive(
                self.volume
                    .read(context, Unit::CubicMetre, Phase::Apparatus, None)?,
            )?;
        let dilution = nonnegative(self.balanced_throughflow.read(
            context,
            Unit::CubicMetrePerSecond,
            Phase::Gas,
            None,
        )?)? / volume;
        require(
            self.well_mixed_constant_volume_support.read(
                context,
                Unit::One,
                Phase::Apparatus,
                None,
            )? == 1.0,
            "well-mixed constant-volume support missing",
        )?;
        let mut deposition = 0.0;
        for face in &self.absorbing_faces {
            require(
                face.coordinate_frame == context.coordinate_frame,
                "wall/gravity coordinate frames differ",
            )?;
            let area =
                positive(
                    face.area
                        .read(context, Unit::SquareMetre, Phase::Apparatus, None)?,
                )?;
            let film = positive(face.film_thickness.read(
                context,
                Unit::Metre,
                Phase::Apparatus,
                Some(particle.diameter),
            )?)?;
            let mut normal = [0.0; 3];
            for (axis, component) in normal.iter_mut().enumerate() {
                *component =
                    face.outward_normal[axis].read(context, Unit::One, Phase::Apparatus, None)?;
            }
            require(
                (normal.iter().map(|x| x * x).sum::<f64>() - 1.0).abs() <= tolerance,
                "wall normal must be unit length",
            )?;
            let projected: f64 = normal
                .iter()
                .zip(particle.gravity_velocity)
                .map(|(a, b)| a * b)
                .sum();
            deposition += area / volume * (particle.diffusion / film + projected.max(0.0));
        }
        Ok((nonnegative(dilution)?, nonnegative(deposition)?))
    }
}
#[derive(Clone, Debug)]
pub struct SourceCohort {
    pub number_rate: Sample,
    pub particle_volume: Sample,
    pub line_loss_rate: Sample,
}
#[derive(Clone, Debug)]
pub enum SourceYield {
    Unknown,
    IndependentlySupplied(Vec<SourceCohort>),
}
#[derive(Clone, Debug)]
pub struct SourceInterval {
    pub start_seconds: Sample,
    pub end_seconds: Sample,
    pub condition_id: String,
    pub time_origin: String,
    pub source: SourceYield,
}
#[derive(Clone, Debug)]
pub struct TransferLine {
    pub volume: Sample,
    pub flow: Sample,
    pub branch_fraction: Sample,
    pub initially_empty_support: Sample,
}
#[derive(Clone, Debug)]
pub struct InitialCohort {
    pub number_concentration: Sample,
    pub particle_volume: Sample,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Section {
    pub number_per_m3: f64,
    pub particle_volume_per_m3: f64,
}
impl Section {
    fn mean_volume(self) -> Result<Option<f64>> {
        nonnegative(self.number_per_m3)?;
        nonnegative(self.particle_volume_per_m3)?;
        if self.number_per_m3 == 0.0 {
            require(
                self.particle_volume_per_m3 == 0.0,
                "volume without particles",
            )?;
            Ok(None)
        } else {
            Ok(Some(positive(
                self.particle_volume_per_m3 / self.number_per_m3,
            )?))
        }
    }
}
#[derive(Clone, Debug)]
pub struct Numerics {
    /// Finite positive strictly increasing upper volume edges; final bin is open.
    pub section_upper_volumes: Vec<f64>,
    pub max_step_seconds: f64,
    pub max_fractional_coagulation_loss: f64,
    pub comparison_tolerance: f64,
    pub max_substeps: usize,
}
impl Numerics {
    fn check(&self) -> Result<()> {
        positive(self.max_step_seconds)?;
        positive(self.max_fractional_coagulation_loss)?;
        positive(self.comparison_tolerance)?;
        require(
            self.max_fractional_coagulation_loss < 1.0
                && self.comparison_tolerance < 1.0
                && self.max_substeps > 0,
            "invalid numerical/resource controls",
        )?;
        let mut previous = 0.0;
        for edge in &self.section_upper_volumes {
            positive(*edge)?;
            require(*edge > previous, "unordered section edges")?;
            previous = *edge;
        }
        Ok(())
    }
    fn destination(&self, volume: f64) -> usize {
        self.section_upper_volumes
            .partition_point(|edge| *edge < volume)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coagulation {
    DisabledScenario,
    UnitStickingNeutralHardSpheres,
}
#[derive(Clone, Debug, Default)]
pub struct Budget {
    pub initial_number_per_m3: f64,
    pub initial_volume_per_m3: f64,
    pub source_emitted_number_per_m3: f64,
    pub source_emitted_volume_per_m3: f64,
    pub line_inventory_number_per_m3: f64,
    pub line_inventory_volume_per_m3: f64,
    pub admitted_number_per_m3: f64,
    pub admitted_volume_per_m3: f64,
    pub line_lost_number_per_m3: f64,
    pub line_lost_volume_per_m3: f64,
    pub exhausted_number_per_m3: f64,
    pub exhausted_volume_per_m3: f64,
    pub deposited_number_per_m3: f64,
    pub deposited_volume_per_m3: f64,
    pub coagulation_events_per_m3: f64,
}
#[derive(Clone, Debug)]
pub struct Prediction {
    pub sections: Vec<Section>,
    pub budget: Budget,
    pub time_seconds: f64,
    pub condition_id: String,
    pub time_origin: String,
    pub gravity_regime: GravityRegime,
    pub experimental_unqualified: bool,
    pub uncertainty: Uncertainty,
    pub sectional_size_approximation: bool,
    pub substeps: usize,
    pub particle_density: Sample,
}
#[derive(Clone, Copy, Debug)]
pub struct Moments {
    pub number_per_m3: f64,
    pub condensed_volume_per_m3: f64,
    pub mass_kg_per_m3: f64,
    pub second_volume_moment_m3: f64,
    pub number_mean_volume_equivalent_diameter_m: Option<f64>,
}
impl Prediction {
    pub fn moments(&self, density: &Sample, context: &Context) -> Result<Moments> {
        require(
            density.value == self.particle_density.value
                && density.unit == self.particle_density.unit
                && density.phase == self.particle_density.phase
                && density.evidence.source == self.particle_density.evidence.source,
            "mass cannot be reinterpreted using a different particle density",
        )?;
        require(
            self.condition_id == context.condition_id && self.time_origin == context.time_origin,
            "prediction context differs",
        )?;
        let rho = positive(density.read(
            context,
            Unit::KilogramPerCubicMetre,
            context.particle_phase,
            None,
        )?)?;
        let mut n = 0.0;
        let mut u = 0.0;
        let mut m2 = 0.0;
        for section in &self.sections {
            if let Some(v) = section.mean_volume()? {
                n += section.number_per_m3;
                u += section.particle_volume_per_m3;
                m2 += section.number_per_m3 * v * v;
            }
        }
        let diameter = if n > 0.0 {
            Some(positive((6.0 * u / (PI * n)).cbrt())?)
        } else {
            None
        };
        Ok(Moments {
            number_per_m3: nonnegative(n)?,
            condensed_volume_per_m3: nonnegative(u)?,
            mass_kg_per_m3: nonnegative(rho * u)?,
            second_volume_moment_m3: nonnegative(m2)?,
            number_mean_volume_equivalent_diameter_m: diameter,
        })
    }
}

/// One fill, one explicit origin, piecewise-constant independent emissions.
/// Source history must cover the complete requested interval, including explicit zeros.
#[allow(clippy::too_many_arguments)]
pub fn propagate(
    context: &Context,
    properties: &AerosolProperties,
    chamber: &Chamber,
    transfer: &TransferLine,
    initial: &[InitialCohort],
    history: &[SourceInterval],
    start_seconds: f64,
    end_seconds: f64,
    coagulation: Coagulation,
    parameters: &Parameters,
    numerics: &Numerics,
) -> Result<Prediction> {
    numerics.check()?;
    context.check(numerics.comparison_tolerance)?;
    properties.check(context)?;
    chamber.check(context, numerics.comparison_tolerance)?;
    finite(start_seconds)?;
    finite(end_seconds)?;
    require(end_seconds > start_seconds, "time interval must advance")?;
    require(!history.is_empty(), "explicit source history required")?;
    let volume = positive(chamber.volume.read(
        context,
        Unit::CubicMetre,
        Phase::Apparatus,
        None,
    )?)?;
    let delay =
        nonnegative(
            transfer
                .volume
                .read(context, Unit::CubicMetre, Phase::Apparatus, None)?,
        )? / positive(
            transfer
                .flow
                .read(context, Unit::CubicMetrePerSecond, Phase::Gas, None)?,
        )?;
    nonnegative(delay)?;
    require(
        transfer
            .initially_empty_support
            .read(context, Unit::One, Phase::Apparatus, None)?
            == 1.0,
        "initial line inventory must be explicitly empty for this transport closure",
    )?;
    let branch = nonnegative(transfer.branch_fraction.read(
        context,
        Unit::One,
        Phase::Apparatus,
        None,
    )?)?;
    require(branch <= 1.0, "branch fraction outside [0,1]")?;
    let source_carrier_flow = nonnegative(transfer.flow.value * branch)?;
    require(
        source_carrier_flow <= chamber.balanced_throughflow.value,
        "source-line gas inflow exceeds balanced chamber throughflow",
    )?;
    // The transfer line is initially empty. Source emission starts at start_seconds.
    let mut history_end = start_seconds;
    for interval in history {
        require(
            interval.condition_id == context.condition_id
                && interval.time_origin == context.time_origin,
            "history mixes fills, condition identities or time origins",
        )?;
        let start = interval
            .start_seconds
            .read(context, Unit::Second, Phase::Apparatus, None)?;
        let end = interval
            .end_seconds
            .read(context, Unit::Second, Phase::Apparatus, None)?;
        require(
            start == history_end && end > start,
            "source history gap/overlap/reversed interval",
        )?;
        history_end = end;
        let SourceYield::IndependentlySupplied(cohorts) = &interval.source else {
            return Err("unknown source yield cannot be replaced by zero".into());
        };
        for cohort in cohorts {
            let v = positive(cohort.particle_volume.read(
                context,
                Unit::CubicMetre,
                context.particle_phase,
                None,
            )?)?;
            let d = properties.particle(v, context, parameters)?.diameter;
            nonnegative(cohort.number_rate.read(
                context,
                Unit::NumberPerSecond,
                context.particle_phase,
                Some(d),
            )?)?;
            nonnegative(cohort.line_loss_rate.read(
                context,
                Unit::PerSecond,
                Phase::Apparatus,
                Some(d),
            )?)?;
        }
    }
    require(
        history_end >= end_seconds,
        "source history does not cover forecast",
    )?;
    let mut sections = vec![Section::default(); numerics.section_upper_volumes.len() + 1];
    let mut budget = Budget::default();
    for cohort in initial {
        let v = positive(cohort.particle_volume.read(
            context,
            Unit::CubicMetre,
            context.particle_phase,
            None,
        )?)?;
        let d = properties.particle(v, context, parameters)?.diameter;
        let n = nonnegative(cohort.number_concentration.read(
            context,
            Unit::NumberPerCubicMetre,
            context.particle_phase,
            Some(d),
        )?)?;
        let slot = numerics.destination(v);
        sections[slot].number_per_m3 += n;
        sections[slot].particle_volume_per_m3 += n * v;
        budget.initial_number_per_m3 += n;
        budget.initial_volume_per_m3 += n * v;
    }
    // Validate chamber assumptions even if there are no particles at all.
    let support = chamber.well_mixed_constant_volume_support.read(
        context,
        Unit::One,
        Phase::Apparatus,
        None,
    )?;
    require(support == 1.0, "well-mixed constant-volume support missing")?;
    nonnegative(chamber.balanced_throughflow.read(
        context,
        Unit::CubicMetrePerSecond,
        Phase::Gas,
        None,
    )?)?;
    let mut time = start_seconds;
    let mut steps = 0;
    while time < end_seconds {
        require(
            steps < numerics.max_substeps,
            "explicit substep resource budget exhausted; no partial result emitted",
        )?;
        let mut volumes = Vec::with_capacity(sections.len());
        let mut particles = Vec::with_capacity(sections.len());
        for section in &sections {
            let v = section.mean_volume()?;
            particles.push(
                v.map(|value| properties.particle(value, context, parameters))
                    .transpose()?,
            );
            volumes.push(v);
        }
        let mut pairs = Vec::new();
        let mut removal_rates = vec![0.0; sections.len()];
        if coagulation == Coagulation::UnitStickingNeutralHardSpheres {
            for i in 0..sections.len() {
                for j in i..sections.len() {
                    if let (Some(a), Some(b)) = (particles[i], particles[j]) {
                        let k = collision_kernel(a, b)?;
                        let rate = nonnegative(
                            k * sections[i].number_per_m3 * sections[j].number_per_m3
                                / if i == j { 2.0 } else { 1.0 },
                        )?;
                        removal_rates[i] += rate;
                        removal_rates[j] += rate;
                        pairs.push((i, j, rate));
                    }
                }
            }
        }
        let mut dt = numerics.max_step_seconds.min(end_seconds - time);
        for (section, rate) in sections.iter().zip(&removal_rates) {
            if *rate > 0.0 {
                dt =
                    dt.min(numerics.max_fractional_coagulation_loss * section.number_per_m3 / rate);
            }
        }
        positive(dt)?;
        require(
            time + dt > time,
            "time step cannot advance floating-point clock",
        )?;
        // Remove each reactant's total loss fraction once; births are accumulated separately.
        let mut next = sections.clone();
        for i in 0..sections.len() {
            if sections[i].number_per_m3 > 0.0 {
                let factor = 1.0 - removal_rates[i] * dt / sections[i].number_per_m3;
                nonnegative(factor)?;
                next[i].number_per_m3 *= factor;
                next[i].particle_volume_per_m3 *= factor;
            }
        }
        for (i, j, rate) in pairs {
            let count = nonnegative(rate * dt)?;
            let v = positive(volumes[i].unwrap() + volumes[j].unwrap())?;
            let target = numerics.destination(v);
            next[target].number_per_m3 += count;
            next[target].particle_volume_per_m3 += count * v;
            budget.coagulation_events_per_m3 += count;
        }
        sections = next;
        // Exact frozen-size linear removal; coagulation/transport/source splitting is first order.
        for section in &mut sections {
            if let Some(v) = section.mean_volume()? {
                let particle = properties.particle(v, context, parameters)?;
                let (dilution, deposition) =
                    chamber.rates(particle, context, numerics.comparison_tolerance)?;
                let total = nonnegative(dilution + deposition)?;
                if total > 0.0 {
                    let removed_fraction = nonnegative(-(-total * dt).exp_m1())?;
                    let removed_n = section.number_per_m3 * removed_fraction;
                    let removed_u = section.particle_volume_per_m3 * removed_fraction;
                    budget.exhausted_number_per_m3 += removed_n * dilution / total;
                    budget.exhausted_volume_per_m3 += removed_u * dilution / total;
                    budget.deposited_number_per_m3 += removed_n * deposition / total;
                    budget.deposited_volume_per_m3 += removed_u * deposition / total;
                    let remaining = positive((-total * dt).exp())?;
                    section.number_per_m3 *= remaining;
                    section.particle_volume_per_m3 *= remaining;
                }
            }
        }
        for interval in history {
            let arrival_start = finite(interval.start_seconds.value + delay)?;
            let arrival_end = finite(interval.end_seconds.value + delay)?;
            let overlap = (time + dt).min(arrival_end) - time.max(arrival_start);
            if overlap <= 0.0 {
                continue;
            }
            let SourceYield::IndependentlySupplied(cohorts) = &interval.source else {
                unreachable!()
            };
            for cohort in cohorts {
                let v = cohort.particle_volume.value;
                let exponent = nonnegative(cohort.line_loss_rate.value * delay)?;
                let fraction = (-exponent).exp();
                let directed_n = nonnegative(cohort.number_rate.value * branch * overlap / volume)?;
                let n = directed_n * fraction;
                budget.admitted_number_per_m3 += n;
                budget.admitted_volume_per_m3 += n * v;
                let target = numerics.destination(v);
                sections[target].number_per_m3 += n;
                sections[target].particle_volume_per_m3 += n * v;
            }
        }
        for section in &sections {
            section.mean_volume()?;
        }
        require(
            sections
                .iter()
                .map(|s| s.particle_volume_per_m3)
                .sum::<f64>()
                < 1.0,
            "condensed particle volume cannot occupy the entire chamber",
        )?;
        time += dt;
        steps += 1;
    }
    // Account for all emitted particles, including cohorts still in the transfer line.
    // A supplied line loss rate acts continuously during the plug-flow residence time.
    for interval in history {
        let emitted_start = interval.start_seconds.value;
        let emitted_end = interval.end_seconds.value.min(end_seconds);
        if emitted_end <= emitted_start {
            continue;
        }
        let SourceYield::IndependentlySupplied(cohorts) = &interval.source else {
            unreachable!()
        };
        for cohort in cohorts {
            let rate = nonnegative(cohort.number_rate.value * branch / volume)?;
            let v = cohort.particle_volume.value;
            let emitted = nonnegative(rate * (emitted_end - emitted_start))?;
            let in_line_start = emitted_start.max(end_seconds - delay);
            let duration = (emitted_end - in_line_start).max(0.0);
            let loss = cohort.line_loss_rate.value;
            let survival_integral = if loss == 0.0 {
                duration
            } else {
                let oldest_exponent = nonnegative(loss * duration)?;
                let youngest_exponent = nonnegative(loss * (end_seconds - emitted_end))?;
                nonnegative((-youngest_exponent).exp() * -(-oldest_exponent).exp_m1() / loss)?
            };
            let inventory = nonnegative(rate * survival_integral)?;
            let departed_duration = (emitted_end.min(end_seconds - delay) - emitted_start).max(0.0);
            let departed_loss = rate * departed_duration * -(-nonnegative(loss * delay)?).exp_m1();
            let transit_loss = rate * nonnegative(duration - survival_integral)?;
            let lost = nonnegative(departed_loss + transit_loss)?;
            budget.source_emitted_number_per_m3 += emitted;
            budget.source_emitted_volume_per_m3 += emitted * v;
            budget.line_inventory_number_per_m3 += inventory;
            budget.line_inventory_volume_per_m3 += inventory * v;
            budget.line_lost_number_per_m3 += lost;
            budget.line_lost_volume_per_m3 += lost * v;
        }
    }
    for value in [
        budget.initial_number_per_m3,
        budget.initial_volume_per_m3,
        budget.source_emitted_number_per_m3,
        budget.source_emitted_volume_per_m3,
        budget.line_inventory_number_per_m3,
        budget.line_inventory_volume_per_m3,
        budget.admitted_number_per_m3,
        budget.admitted_volume_per_m3,
        budget.line_lost_number_per_m3,
        budget.line_lost_volume_per_m3,
        budget.exhausted_number_per_m3,
        budget.exhausted_volume_per_m3,
        budget.deposited_number_per_m3,
        budget.deposited_volume_per_m3,
        budget.coagulation_events_per_m3,
    ] {
        nonnegative(value)?;
    }
    let prediction = Prediction {
        sections,
        budget,
        time_seconds: time,
        condition_id: context.condition_id.clone(),
        time_origin: context.time_origin.clone(),
        gravity_regime: context.gravity_regime,
        experimental_unqualified: true,
        uncertainty: Uncertainty::Unquantified,
        sectional_size_approximation: true,
        substeps: steps,
        particle_density: properties.particle_density.clone(),
    };
    // Conservation is a hard invariant, not just a testing diagnostic.
    let m = prediction.moments(&properties.particle_density, context)?;
    let b = &prediction.budget;
    for (left, right) in [
        (
            b.source_emitted_number_per_m3,
            b.admitted_number_per_m3 + b.line_lost_number_per_m3 + b.line_inventory_number_per_m3,
        ),
        (
            b.source_emitted_volume_per_m3,
            b.admitted_volume_per_m3 + b.line_lost_volume_per_m3 + b.line_inventory_volume_per_m3,
        ),
        (
            b.initial_volume_per_m3 + b.admitted_volume_per_m3,
            m.condensed_volume_per_m3 + b.exhausted_volume_per_m3 + b.deposited_volume_per_m3,
        ),
        (
            b.initial_number_per_m3 + b.admitted_number_per_m3,
            m.number_per_m3
                + b.exhausted_number_per_m3
                + b.deposited_number_per_m3
                + b.coagulation_events_per_m3,
        ),
    ] {
        finite(left)?;
        finite(right)?;
        require(
            (left - right).abs() <= numerics.comparison_tolerance * left.abs().max(right.abs()),
            "number/volume budget failed",
        )?;
    }
    Ok(prediction)
}

#[derive(Clone, Debug)]
pub struct ExtinctionDetector {
    pub wavelength_in_medium: Sample,
    /// Real relative refractive index; explicit nonabsorbing homogeneous spheres only.
    pub real_relative_refractive_index: Sample,
    pub nonabsorbing_sphere_support: Sample,
    pub path_length: Sample,
    pub incident_power: Sample,
    pub responsivity: Sample,
    pub dark_voltage: Sample,
    pub collimated_no_scattered_light_support: Sample,
}
#[derive(Clone, Copy, Debug)]
pub struct DetectorOutput {
    pub scattering_coefficient_per_metre: f64,
    pub transmitted_power_watt: f64,
    pub detector_voltage: f64,
    pub experimental_unqualified: bool,
}
pub fn observe(
    prediction: &Prediction,
    detector: &ExtinctionDetector,
    context: &Context,
    p: &Parameters,
) -> Result<DetectorOutput> {
    require(
        prediction.condition_id == context.condition_id
            && prediction.time_origin == context.time_origin,
        "observation context differs",
    )?;
    let wavelength = positive(detector.wavelength_in_medium.read(
        context,
        Unit::Metre,
        Phase::Apparatus,
        None,
    )?)?;
    let m = positive(detector.real_relative_refractive_index.read(
        context,
        Unit::One,
        context.particle_phase,
        None,
    )?)?;
    require(
        detector.nonabsorbing_sphere_support.read(
            context,
            Unit::One,
            context.particle_phase,
            None,
        )? == 1.0,
        "nonabsorbing compact sphere support missing; absorbing soot is not covered",
    )?;
    require(
        detector.collimated_no_scattered_light_support.read(
            context,
            Unit::One,
            Phase::Apparatus,
            None,
        )? == 1.0,
        "collimated extinction geometry support missing",
    )?;
    let path = positive(detector.path_length.read(
        context,
        Unit::Metre,
        Phase::Apparatus,
        None,
    )?)?;
    let incident = positive(detector.incident_power.read(
        context,
        Unit::Watt,
        Phase::Apparatus,
        None,
    )?)?;
    let response = positive(detector.responsivity.read(
        context,
        Unit::VoltPerWatt,
        Phase::Apparatus,
        None,
    )?)?;
    let dark = detector
        .dark_voltage
        .read(context, Unit::Volt, Phase::Apparatus, None)?;
    let contrast = finite((m * m - 1.0) / (m * m + 2.0))?;
    let mut coefficient = 0.0;
    for section in &prediction.sections {
        if let Some(v) = section.mean_volume()? {
            let d = positive((6.0 * v / PI).cbrt())?;
            // Optical property validity also depends on the representative size.
            detector.real_relative_refractive_index.read(
                context,
                Unit::One,
                context.particle_phase,
                Some(d),
            )?;
            detector.nonabsorbing_sphere_support.read(
                context,
                Unit::One,
                context.particle_phase,
                Some(d),
            )?;
            let x = PI * d / wavelength;
            require(
                x <= p.get("maximum_rayleigh_size") && m * x <= p.get("maximum_rayleigh_size"),
                "Rayleigh size/internal-field domain exceeded; Mie optics required",
            )?;
            let cross_section =
                nonnegative((8.0 * PI / 3.0) * (d / 2.0).powi(2) * x.powi(4) * contrast.powi(2))?;
            coefficient += section.number_per_m3 * cross_section;
        }
    }
    nonnegative(coefficient)?;
    let optical_depth = nonnegative(coefficient * path)?;
    let power = positive(incident * (-optical_depth).exp())?;
    Ok(DetectorOutput {
        scattering_coefficient_per_metre: coefficient,
        transmitted_power_watt: power,
        detector_voltage: finite(dark + response * power)?,
        experimental_unqualified: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(value: f64, unit: Unit, phase: Phase) -> Sample {
        Sample {
            value,
            unit,
            phase,
            domain: Domain {
                temperature_kelvin: [200.0, 400.0],
                pressure_pascal: [50000.0, 200000.0],
                diameter_metres: [1e-12, 1.0],
                gas_composition_id: "synthetic-gas".into(),
                particle_composition_id: "synthetic-particles".into(),
            },
            reference: "explicit synthetic constant-property domain; SI".into(),
            uncertainty: Uncertainty::Unquantified,
            evidence: Evidence {
                source: "synthetic-fixture".into(),
                locator: "inline invariant fixture".into(),
                dependencies: BTreeSet::new(),
                origin: Origin::SyntheticFixture,
            },
        }
    }
    fn context() -> Context {
        let composition = |identity: &str, phase| Composition {
            identity: identity.into(),
            mass_fractions: BTreeMap::from([
                ("synthetic-species-a".into(), sample(0.25, Unit::One, phase)),
                ("synthetic-species-b".into(), sample(0.75, Unit::One, phase)),
            ]),
            evidence: sample(1.0, Unit::One, phase).evidence,
        };
        Context {
            temperature: sample(300.0, Unit::Kelvin, Phase::Gas),
            pressure: sample(100000.0, Unit::Pascal, Phase::Gas),
            gas_composition: composition("synthetic-gas", Phase::Gas),
            particle_composition: composition("synthetic-particles", Phase::Liquid),
            particle_phase: Phase::Liquid,
            gravity: std::array::from_fn(|_| {
                sample(0.0, Unit::MetrePerSecondSquared, Phase::Apparatus)
            }),
            coordinate_frame: "chamber-xyz".into(),
            gravity_regime: GravityRegime::Microgravity,
            condition_id: "one-synthetic-fill".into(),
            time_origin: "synthetic emission clock zero".into(),
            detector_truth_dependencies: BTreeSet::from([
                "independent-detector-validation-trace".into()
            ]),
        }
    }
    fn properties() -> AerosolProperties {
        AerosolProperties {
            particle_density: sample(1000.0, Unit::KilogramPerCubicMetre, Phase::Liquid),
            gas_density: sample(1.0, Unit::KilogramPerCubicMetre, Phase::Gas),
            gas_viscosity: sample(1.8e-5, Unit::PascalSecond, Phase::Gas),
            gas_mean_free_path: sample(1e-12, Unit::Metre, Phase::Gas),
            dilute_neutral_compact_coalescing_sphere_support: sample(1.0, Unit::One, Phase::Liquid),
            drag: DragLaw::ContinuumNoSlip,
        }
    }
    fn chamber() -> Chamber {
        Chamber {
            volume: sample(1.0, Unit::CubicMetre, Phase::Apparatus),
            balanced_throughflow: sample(0.0, Unit::CubicMetrePerSecond, Phase::Gas),
            absorbing_faces: vec![],
            well_mixed_constant_volume_support: sample(1.0, Unit::One, Phase::Apparatus),
        }
    }
    fn transfer() -> TransferLine {
        TransferLine {
            volume: sample(0.0, Unit::CubicMetre, Phase::Apparatus),
            flow: sample(1.0, Unit::CubicMetrePerSecond, Phase::Gas),
            branch_fraction: sample(0.0, Unit::One, Phase::Apparatus),
            initially_empty_support: sample(1.0, Unit::One, Phase::Apparatus),
        }
    }
    fn numerics(step: f64) -> Numerics {
        Numerics {
            section_upper_volumes: vec![],
            max_step_seconds: step,
            max_fractional_coagulation_loss: 0.2,
            comparison_tolerance: 1e-10,
            max_substeps: 100000,
        }
    }
    fn history(end: f64) -> Vec<SourceInterval> {
        vec![SourceInterval {
            start_seconds: sample(0.0, Unit::Second, Phase::Apparatus),
            end_seconds: sample(end, Unit::Second, Phase::Apparatus),
            condition_id: context().condition_id,
            time_origin: context().time_origin,
            source: SourceYield::IndependentlySupplied(vec![]),
        }]
    }
    fn initial(number: f64, diameter: f64) -> Vec<InitialCohort> {
        vec![InitialCohort {
            number_concentration: sample(number, Unit::NumberPerCubicMetre, Phase::Liquid),
            particle_volume: sample(PI / 6.0 * diameter.powi(3), Unit::CubicMetre, Phase::Liquid),
        }]
    }
    fn run(
        initial: &[InitialCohort],
        chamber: &Chamber,
        coagulation: Coagulation,
        step: f64,
        end: f64,
    ) -> Prediction {
        propagate(
            &context(),
            &properties(),
            chamber,
            &transfer(),
            initial,
            &history(end),
            0.0,
            end,
            coagulation,
            &Parameters::authored().unwrap(),
            &numerics(step),
        )
        .unwrap()
    }
    fn close(a: f64, b: f64, tolerance: f64) {
        assert!(
            (a - b).abs() <= tolerance * a.abs().max(b.abs()).max(f64::MIN_POSITIVE),
            "{a} != {b}"
        );
    }
    fn detector() -> ExtinctionDetector {
        ExtinctionDetector {
            wavelength_in_medium: sample(1e-3, Unit::Metre, Phase::Apparatus),
            real_relative_refractive_index: sample(1.5, Unit::One, Phase::Liquid),
            nonabsorbing_sphere_support: sample(1.0, Unit::One, Phase::Liquid),
            path_length: sample(0.1, Unit::Metre, Phase::Apparatus),
            incident_power: sample(0.01, Unit::Watt, Phase::Apparatus),
            responsivity: sample(2.0, Unit::VoltPerWatt, Phase::Apparatus),
            dark_voltage: sample(0.1, Unit::Volt, Phase::Apparatus),
            collimated_no_scattered_light_support: sample(1.0, Unit::One, Phase::Apparatus),
        }
    }

    #[test]
    fn closed_empty_and_no_source_dilution_preserve_interpretable_moments() {
        let empty = run(&[], &chamber(), Coagulation::DisabledScenario, 0.2, 2.0);
        let moments = empty
            .moments(&properties().particle_density, &context())
            .unwrap();
        assert_eq!(moments.number_per_m3, 0.0);
        assert_eq!(moments.number_mean_volume_equivalent_diameter_m, None);
        let cohorts = initial(1e10, 1e-6);
        let closed = run(
            &cohorts,
            &chamber(),
            Coagulation::DisabledScenario,
            0.2,
            2.0,
        );
        let cm = closed
            .moments(&properties().particle_density, &context())
            .unwrap();
        close(cm.number_per_m3, 1e10, 1e-12);
        close(
            cm.condensed_volume_per_m3,
            1e10 * cohorts[0].particle_volume.value,
            1e-12,
        );
        close(
            cm.mass_kg_per_m3,
            1000.0 * cm.condensed_volume_per_m3,
            1e-12,
        );
        close(
            cm.number_mean_volume_equivalent_diameter_m.unwrap(),
            1e-6,
            1e-12,
        );
        close(
            cm.second_volume_moment_m3,
            1e10 * cohorts[0].particle_volume.value.powi(2),
            1e-12,
        );
        let mut vessel = chamber();
        vessel.balanced_throughflow.value = 0.3;
        let diluted = run(&cohorts, &vessel, Coagulation::DisabledScenario, 0.17, 2.0);
        let dm = diluted
            .moments(&properties().particle_density, &context())
            .unwrap();
        close(dm.number_per_m3, 1e10 * (-0.6_f64).exp(), 1e-12);
        close(
            dm.condensed_volume_per_m3 / cm.condensed_volume_per_m3,
            (-0.6_f64).exp(),
            1e-12,
        );
        close(
            dm.number_mean_volume_equivalent_diameter_m.unwrap(),
            1e-6,
            1e-12,
        );
    }

    #[test]
    fn coagulation_conserves_mass_volume_and_removes_one_number_per_event() {
        let input = initial(1e15, 1e-6);
        let mut controls = numerics(0.01);
        let v = input[0].particle_volume.value;
        controls.section_upper_volumes = vec![1.5 * v, 3.5 * v, 7.5 * v];
        let result = propagate(
            &context(),
            &properties(),
            &chamber(),
            &transfer(),
            &input,
            &history(2.0),
            0.0,
            2.0,
            Coagulation::UnitStickingNeutralHardSpheres,
            &Parameters::authored().unwrap(),
            &controls,
        )
        .unwrap();
        let moments = result
            .moments(&properties().particle_density, &context())
            .unwrap();
        close(moments.condensed_volume_per_m3, 1e15 * v, 1e-12);
        close(moments.mass_kg_per_m3, 1000.0 * 1e15 * v, 1e-12);
        close(
            moments.number_per_m3 + result.budget.coagulation_events_per_m3,
            1e15,
            1e-12,
        );
        assert!(moments.number_per_m3 < 1e15);
        assert!(moments.second_volume_moment_m3 > 1e15 * v * v);
        assert!(result.sections.last().unwrap().number_per_m3 > 0.0);
        assert!(result.sections.iter().all(|s| s.mean_volume().is_ok()));
    }

    #[test]
    fn hard_sphere_kernel_is_symmetric_finite_and_has_continuum_limit() {
        let c = context();
        let p = Parameters::authored().unwrap();
        let props = properties();
        let v = initial(1.0, 1e-5)[0].particle_volume.value;
        let a = props.particle(v, &c, &p).unwrap();
        let b = props.particle(8.0 * v, &c, &p).unwrap();
        close(
            collision_kernel(a, b).unwrap(),
            collision_kernel(b, a).unwrap(),
            1e-14,
        );
        // Synthetic very-small particle inertia makes continuum encounter dynamics explicit.
        let mut limit = a;
        limit.jump = 0.0;
        limit.thermal_speed = 1e100;
        close(
            collision_kernel(limit, limit).unwrap(),
            8.0 * p.get("boltzmann_j_per_k") * 300.0 / (3.0 * 1.8e-5),
            1e-12,
        );
    }

    #[test]
    fn delayed_independent_source_has_separate_line_and_chamber_budgets() {
        let mut line = transfer();
        line.volume.value = 1.0;
        line.branch_fraction.value = 1.0;
        let mut vessel = chamber();
        vessel.balanced_throughflow.value = 1.0;
        let mut source = history(4.0);
        source[0].source = SourceYield::IndependentlySupplied(vec![SourceCohort {
            number_rate: sample(2.0, Unit::NumberPerSecond, Phase::Liquid),
            particle_volume: initial(1.0, 1e-6)[0].particle_volume.clone(),
            line_loss_rate: sample(2.0_f64.ln(), Unit::PerSecond, Phase::Apparatus),
        }]);
        let result = propagate(
            &context(),
            &properties(),
            &vessel,
            &line,
            &[],
            &source,
            0.0,
            4.0,
            Coagulation::DisabledScenario,
            &Parameters::authored().unwrap(),
            &numerics(0.001),
        )
        .unwrap();
        let moments = result
            .moments(&properties().particle_density, &context())
            .unwrap();
        close(moments.number_per_m3, 1.0 - (-3.0_f64).exp(), 0.001);
        close(
            moments.number_per_m3 + result.budget.exhausted_number_per_m3,
            3.0,
            1e-10,
        );
        close(result.budget.admitted_number_per_m3, 3.0, 1e-12);
        close(result.budget.source_emitted_number_per_m3, 8.0, 1e-12);
        close(
            result.budget.line_inventory_number_per_m3,
            1.0 / 2.0_f64.ln(),
            1e-12,
        );
        close(
            result.budget.line_lost_number_per_m3,
            5.0 - 1.0 / 2.0_f64.ln(),
            1e-12,
        );
    }

    #[test]
    fn source_dilution_splitting_converges_to_analytic_balance() {
        let mut vessel = chamber();
        vessel.balanced_throughflow.value = 0.5;
        let mut line = transfer();
        line.flow.value = 0.5;
        line.branch_fraction.value = 1.0;
        let mut source = history(2.0);
        source[0].source = SourceYield::IndependentlySupplied(vec![SourceCohort {
            number_rate: sample(10.0, Unit::NumberPerSecond, Phase::Liquid),
            particle_volume: initial(1.0, 1e-6)[0].particle_volume.clone(),
            line_loss_rate: sample(0.0, Unit::PerSecond, Phase::Apparatus),
        }]);
        let solve = |step| {
            propagate(
                &context(),
                &properties(),
                &vessel,
                &line,
                &[],
                &source,
                0.0,
                2.0,
                Coagulation::DisabledScenario,
                &Parameters::authored().unwrap(),
                &numerics(step),
            )
            .unwrap()
            .moments(&properties().particle_density, &context())
            .unwrap()
            .number_per_m3
        };
        let exact = 20.0 * (1.0 - (-1.0_f64).exp());
        let coarse = (solve(0.1) - exact).abs();
        let fine = (solve(0.05) - exact).abs();
        assert!(fine < 0.6 * coarse);
        assert!(fine / exact < 0.02);
    }

    #[test]
    fn coagulation_time_refinement_converges_without_changing_coefficients() {
        let input = initial(1e15, 1e-6);
        let solve = |step| {
            run(
                &input,
                &chamber(),
                Coagulation::UnitStickingNeutralHardSpheres,
                step,
                2.0,
            )
            .moments(&properties().particle_density, &context())
            .unwrap()
            .number_per_m3
        };
        let coarse = solve(0.04);
        let medium = solve(0.02);
        let fine = solve(0.01);
        assert!((medium - fine).abs() < 0.6 * (coarse - medium).abs());
    }

    #[test]
    fn resolved_pair_birth_recovers_second_moment_lost_by_one_section_closure() {
        let input = initial(1e15, 1e-6);
        let v = input[0].particle_volume.value;
        let p = Parameters::authored().unwrap();
        let particle = properties().particle(v, &context(), &p).unwrap();
        let dt = 0.01;
        let expected =
            1e15 * v * v + collision_kernel(particle, particle).unwrap() * 1e30 * v * v * dt;
        let solve = |edges| {
            let mut controls = numerics(dt);
            controls.section_upper_volumes = edges;
            propagate(
                &context(),
                &properties(),
                &chamber(),
                &transfer(),
                &input,
                &history(dt),
                0.0,
                dt,
                Coagulation::UnitStickingNeutralHardSpheres,
                &p,
                &controls,
            )
            .unwrap()
            .moments(&properties().particle_density, &context())
            .unwrap()
            .second_volume_moment_m3
        };
        let unresolved = solve(vec![]);
        let resolved = solve(vec![1.5 * v]);
        let refined = solve(vec![1.5 * v, 2.5 * v, 3.5 * v]);
        assert!(unresolved < expected);
        close(resolved, expected, 1e-12);
        close(refined, expected, 1e-12);
    }

    #[test]
    fn gravity_orientation_changes_deposition_and_partial_gravity_stays_unqualified() {
        let mut c = context();
        c.gravity[2].value = -1.0;
        c.gravity_regime = GravityRegime::PartialGravity;
        let face = |normal| AbsorbingFace {
            area: sample(1.0, Unit::SquareMetre, Phase::Apparatus),
            film_thickness: sample(0.01, Unit::Metre, Phase::Apparatus),
            outward_normal: [
                sample(0.0, Unit::One, Phase::Apparatus),
                sample(0.0, Unit::One, Phase::Apparatus),
                sample(normal, Unit::One, Phase::Apparatus),
            ],
            coordinate_frame: c.coordinate_frame.clone(),
        };
        let mut bottom = chamber();
        bottom.absorbing_faces.push(face(-1.0));
        let mut top = chamber();
        top.absorbing_faces.push(face(1.0));
        let particle = properties()
            .particle(
                initial(1.0, 1e-6)[0].particle_volume.value,
                &c,
                &Parameters::authored().unwrap(),
            )
            .unwrap();
        assert!(
            bottom.rates(particle, &c, 1e-10).unwrap().1
                > top.rates(particle, &c, 1e-10).unwrap().1
        );
        let result = propagate(
            &c,
            &properties(),
            &bottom,
            &transfer(),
            &initial(1e10, 1e-6),
            &history(1.0),
            0.0,
            1.0,
            Coagulation::DisabledScenario,
            &Parameters::authored().unwrap(),
            &numerics(0.1),
        )
        .unwrap();
        assert_eq!(result.gravity_regime, GravityRegime::PartialGravity);
        assert!(result.experimental_unqualified);
        assert!(matches!(result.uncertainty, Uncertainty::Unquantified));
        assert!(result.budget.deposited_volume_per_m3 > 0.0);
        bottom.absorbing_faces[0].coordinate_frame = "other-frame".into();
        assert!(bottom.rates(particle, &c, 1e-10).is_err());
    }

    #[test]
    fn optical_size_moment_and_voltage_are_not_mass_or_number_concentration() {
        let c = context();
        let p = Parameters::authored().unwrap();
        let det = detector();
        let first = run(
            &initial(1e10, 1e-6),
            &chamber(),
            Coagulation::DisabledScenario,
            0.1,
            1.0,
        );
        let second = run(
            &initial(2e10, 1e-6 / 2.0_f64.cbrt()),
            &chamber(),
            Coagulation::DisabledScenario,
            0.1,
            1.0,
        );
        let a = observe(&first, &det, &c, &p).unwrap();
        let b = observe(&second, &det, &c, &p).unwrap();
        close(
            first
                .moments(&properties().particle_density, &c)
                .unwrap()
                .mass_kg_per_m3,
            second
                .moments(&properties().particle_density, &c)
                .unwrap()
                .mass_kg_per_m3,
            1e-12,
        );
        close(
            b.scattering_coefficient_per_metre,
            a.scattering_coefficient_per_metre / 2.0,
            1e-12,
        );
        close(
            a.detector_voltage,
            det.dark_voltage.value + det.responsivity.value * a.transmitted_power_watt,
            1e-12,
        );
        let mut different = det.clone();
        different.responsivity.value *= 2.0;
        let changed = observe(&first, &different, &c, &p).unwrap();
        assert_eq!(
            changed.scattering_coefficient_per_metre,
            a.scattering_coefficient_per_metre
        );
        assert!(changed.detector_voltage > a.detector_voltage);
        different.wavelength_in_medium.value = 1e-7;
        assert!(observe(&first, &different, &c, &p).is_err());
        different = det;
        different.nonabsorbing_sphere_support.value = 0.0;
        assert!(observe(&first, &different, &c, &p).is_err());
    }

    #[test]
    fn missing_yield_same_detector_mixed_fills_units_and_invalid_properties_fail_closed() {
        let c = context();
        let p = Parameters::authored().unwrap();
        let controls = numerics(0.1);
        let check =
            |props: &AerosolProperties, source: &[SourceInterval], input: &[InitialCohort]| {
                propagate(
                    &c,
                    props,
                    &chamber(),
                    &transfer(),
                    input,
                    source,
                    0.0,
                    1.0,
                    Coagulation::DisabledScenario,
                    &p,
                    &controls,
                )
            };
        let mut source = history(1.0);
        source[0].source = SourceYield::Unknown;
        assert!(check(&properties(), &source, &[])
            .unwrap_err()
            .contains("unknown source"));
        source = history(1.0);
        source[0].condition_id = "different-fill".into();
        assert!(check(&properties(), &source, &[]).is_err());
        source = history(1.0);
        source[0].start_seconds.value = 0.1;
        assert!(check(&properties(), &source, &[]).is_err());
        let mut input = initial(1.0, 1e-6);
        input[0]
            .number_concentration
            .evidence
            .dependencies
            .insert("independent-detector-validation-trace".into());
        assert!(check(&properties(), &history(1.0), &input)
            .unwrap_err()
            .contains("validation truth"));
        input = initial(1.0, 1e-6);
        input[0].number_concentration.unit = Unit::One;
        assert!(check(&properties(), &history(1.0), &input).is_err());
        let mut props = properties();
        props.particle_density.value = f64::NAN;
        assert!(check(&props, &history(1.0), &[]).is_err());
        props = properties();
        props.gas_viscosity.domain.temperature_kelvin = [100.0, 200.0];
        assert!(check(&props, &history(1.0), &[]).is_err());
        props = properties();
        props.gas_mean_free_path.value = 1e-6;
        assert!(check(&props, &history(1.0), &initial(1.0, 1e-6)).is_err());
        props.drag = DragLaw::PublishedSlipApproximation {
            applicability: sample(1.0, Unit::One, Phase::Apparatus),
        };
        assert!(check(&props, &history(1.0), &initial(1.0, 1e-6)).is_ok());
        let mut tiny_budget = controls;
        tiny_budget.max_substeps = 1;
        assert!(propagate(
            &c,
            &properties(),
            &chamber(),
            &transfer(),
            &[],
            &history(1.0),
            0.0,
            1.0,
            Coagulation::DisabledScenario,
            &p,
            &tiny_budget
        )
        .unwrap_err()
        .contains("resource budget"));
    }

    #[test]
    fn literal_loader_rejects_missing_duplicate_nonfinite_and_executable_values() {
        let text = include_str!(
            "../../../configs/models/aerosol/property-w0/cleanroom-astra/constants.ops"
        );
        assert!(Parameters::from_ops(text).is_ok());
        assert!(Parameters::from_ops("set boltzmann_j_per_k [] 1.0").is_err());
        assert!(Parameters::from_ops(&format!("{text}\nset slip_alpha [] 1.0")).is_err());
        assert!(Parameters::from_ops(&text.replace("1.165", "NaN")).is_err());
        assert!(Parameters::from_ops(&text.replace("1.165", "1+0.165")).is_err());
    }
}

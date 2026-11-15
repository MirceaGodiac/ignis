//! Clean-room, property-driven solid-phase W0. No gas-flame prediction is implied.
//! Internal kernel types only; adapters must use shared contracts at lane boundaries.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Error(pub &'static str);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

fn require(ok: bool, message: &'static str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error(message))
    }
}
fn finite(x: f64) -> bool {
    x.is_finite()
}
fn positive(x: f64) -> bool {
    finite(x) && x > 0.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    KgPerM3,
    JPerKgK,
    WPerMK,
    PerSecond,
    JPerMol,
    JPerKg,
    WPerM2K,
    WPerM2,
    One,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    ExternalDetermination,
    ExplicitScenarioAssumption,
    SyntheticFixture,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Solid,
    Interface,
}

/// SI-only inputs deliberately preserve raw values: unsupported unit conversion fails closed.
#[derive(Clone, Debug)]
pub struct Property {
    pub raw_value: f64,
    pub raw_unit: Unit,
    pub si_value: f64,
    pub si_unit: Unit,
    pub phase: Phase,
    pub source: String,
    pub locator: String,
    pub origin: Origin,
    pub uncertainty_absolute: Option<f64>,
    pub temperature_domain_k: [f64; 2],
    pub pressure_domain_pa: [f64; 2],
    pub reference_temperature_k: f64,
    pub reference_pressure_pa: f64,
    pub reference_identity: String,
}
impl Property {
    fn validate(&self, unit: Unit, phase: Phase, tr: f64, pressure: f64) -> Result<()> {
        require(
            self.raw_unit == unit && self.si_unit == unit && self.raw_value == self.si_value,
            "property raw/SI unit or value mismatch; convert upstream with provenance",
        )?;
        require(
            finite(self.si_value) && self.si_value >= 0.0 && self.phase == phase,
            "invalid property value or phase",
        )?;
        require(
            !self.source.trim().is_empty()
                && !self.locator.trim().is_empty()
                && !self.reference_identity.trim().is_empty(),
            "missing property provenance/reference",
        )?;
        require(
            self.uncertainty_absolute
                .is_none_or(|x| finite(x) && x >= 0.0),
            "invalid property uncertainty",
        )?;
        let [lo, hi] = self.temperature_domain_k;
        let [pl, ph] = self.pressure_domain_pa;
        require(
            positive(lo)
                && finite(hi)
                && lo <= hi
                && positive(pl)
                && finite(ph)
                && pl <= ph
                && positive(self.reference_temperature_k)
                && positive(self.reference_pressure_pa)
                && self.reference_pressure_pa == pressure
                && self.reference_temperature_k == tr
                && pressure >= pl
                && pressure <= ph,
            "invalid property domain or inconsistent sensible-energy reference",
        )
    }
    fn at(&self, t: f64) -> Result<f64> {
        require(
            finite(t) && t >= self.temperature_domain_k[0] && t <= self.temperature_domain_k[1],
            "temperature outside supplied constant-property domain",
        )?;
        Ok(self.si_value)
    }
}

/// All non-universal coefficients are caller-supplied physical properties, never defaults.
#[derive(Clone, Debug)]
pub struct Reactant {
    pub composition_id: String,
    pub loading_kg_per_m3: Property,
    pub heat_capacity: Property,
    pub preexponential: Property,
    pub activation_energy: Property,
    pub endothermic_reaction_enthalpy: Property,
}
#[derive(Clone, Debug)]
pub struct Material {
    pub inert_loading: Property,
    pub inert_heat_capacity: Property,
    /// Fixed effective conductivity requires a persistent matrix; no swelling or char update.
    pub effective_conductivity: Property,
    pub reactants: Vec<Reactant>,
    pub reference_temperature_k: f64,
    pub pressure_pa: f64,
}
impl Material {
    fn properties(&self) -> Vec<(&Property, Unit)> {
        let mut out = vec![
            (&self.inert_loading, Unit::KgPerM3),
            (&self.inert_heat_capacity, Unit::JPerKgK),
            (&self.effective_conductivity, Unit::WPerMK),
        ];
        for r in &self.reactants {
            out.extend([
                (&r.loading_kg_per_m3, Unit::KgPerM3),
                (&r.heat_capacity, Unit::JPerKgK),
                (&r.preexponential, Unit::PerSecond),
                (&r.activation_energy, Unit::JPerMol),
                (&r.endothermic_reaction_enthalpy, Unit::JPerKg),
            ]);
        }
        out
    }
    fn validate(&self) -> Result<()> {
        require(
            positive(self.reference_temperature_k) && positive(self.pressure_pa),
            "invalid reference state",
        )?;
        for (p, u) in self.properties() {
            p.validate(
                u,
                Phase::Solid,
                self.reference_temperature_k,
                self.pressure_pa,
            )?;
            require(
                p.reference_identity == self.inert_heat_capacity.reference_identity,
                "incompatible energy reference identity",
            )?;
        }
        require(
            positive(self.inert_loading.si_value)
                && positive(self.inert_heat_capacity.si_value)
                && positive(self.effective_conductivity.si_value),
            "persistent inert matrix and conductivity required",
        )?;
        let mut names = std::collections::BTreeSet::new();
        for r in &self.reactants {
            require(
                !r.composition_id.trim().is_empty()
                    && names.insert(r.composition_id.as_str())
                    && positive(r.heat_capacity.si_value),
                "invalid or duplicate composition identity/heat capacity",
            )?;
        }
        Ok(())
    }
    fn at(&self, t: f64) -> Result<()> {
        for (p, _) in self.properties() {
            p.at(t)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Constants {
    pub gas_constant_j_mol_k: f64,
    pub stefan_boltzmann_w_m2_k4: f64,
}
impl Constants {
    fn validate(self) -> Result<()> {
        require(
            positive(self.gas_constant_j_mol_k) && positive(self.stefan_boltzmann_w_m2_k4),
            "invalid authored physical constants",
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub nx: usize,
    pub nz: usize,
    pub length_m: f64,
    pub thickness_m: f64,
    pub width_m: f64,
}
impl Grid {
    fn validate(self) -> Result<()> {
        require(
            self.nx > 0
                && self.nz > 0
                && self.nx.checked_mul(self.nz).is_some()
                && positive(self.length_m)
                && positive(self.thickness_m)
                && positive(self.width_m),
            "invalid finite specimen grid",
        )?;
        require(positive(self.volume()), "cell-volume overflow/underflow")
    }
    fn dx(self) -> f64 {
        self.length_m / self.nx as f64
    }
    fn dz(self) -> f64 {
        self.thickness_m / self.nz as f64
    }
    fn volume(self) -> f64 {
        self.dx() * self.dz() * self.width_m
    }
    fn area(self) -> f64 {
        self.dx() * self.width_m
    }
}

/// Declared conditions for which the supplied boundary coefficients/loads are valid.
/// No empirical flow/gravity correction is manufactured from this context.
#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    pub flow_velocity_m_s: [f64; 3],
    pub gravity_m_s2: [f64; 3],
    pub surface_normal: [f64; 3],
    pub oxygen_mole_fraction: f64,
    pub pressure_pa: f64,
    pub normal_tolerance: f64,
}
impl Environment {
    fn validate(&self) -> Result<()> {
        require(
            self.flow_velocity_m_s
                .iter()
                .chain(&self.gravity_m_s2)
                .chain(&self.surface_normal)
                .all(|x| finite(*x))
                && finite(self.oxygen_mole_fraction)
                && (0.0..=1.0).contains(&self.oxygen_mole_fraction)
                && positive(self.pressure_pa)
                && finite(self.normal_tolerance)
                && (0.0..1.0).contains(&self.normal_tolerance),
            "invalid environmental context",
        )?;
        require(
            (self.surface_normal.iter().map(|v| v * v).sum::<f64>() - 1.0).abs()
                <= self.normal_tolerance,
            "surface normal is not unit length",
        )
    }
}

#[derive(Clone, Debug)]
pub struct FaceBoundary {
    pub gas_temperature_k: f64,
    pub radiant_temperature_k: f64,
    pub heat_transfer: Property,
    pub emissivity: Property,
    pub absorbed_incident_flux: Property,
}
impl FaceBoundary {
    fn validate(&self, m: &Material) -> Result<()> {
        require(
            positive(self.gas_temperature_k) && positive(self.radiant_temperature_k),
            "invalid boundary temperature",
        )?;
        for (p, u) in [
            (&self.heat_transfer, Unit::WPerM2K),
            (&self.emissivity, Unit::One),
            (&self.absorbed_incident_flux, Unit::WPerM2),
        ] {
            p.validate(
                u,
                Phase::Interface,
                m.reference_temperature_k,
                m.pressure_pa,
            )?;
        }
        require(self.emissivity.si_value <= 1.0, "emissivity exceeds unity")
    }
    fn flux(&self, t: f64, c: Constants) -> Result<f64> {
        let q = self.absorbed_incident_flux.at(t)?
            + self.heat_transfer.at(t)? * (self.gas_temperature_k - t)
            + self.emissivity.at(t)?
                * c.stefan_boltzmann_w_m2_k4
                * (self.radiant_temperature_k.powi(4) - t.powi(4));
        require(finite(q), "boundary flux overflow")?;
        Ok(q)
    }
    fn lipschitz(&self, c: Constants) -> f64 {
        self.heat_transfer.si_value
            + 4.0
                * self.emissivity.si_value
                * c.stefan_boltzmann_w_m2_k4
                * self.emissivity.temperature_domain_k[1].powi(3)
    }
}

#[derive(Clone, Debug)]
pub struct Control {
    pub start_s: f64,
    pub end_s: f64,
    pub environment: Environment,
    pub boundary_valid_for: Environment,
    pub boundary_source: String,
    /// Explicit pilot/control history retained as context; pilot does not imply predicted flame.
    pub pilot_on: bool,
    pub view_available: bool,
    pub front: Vec<FaceBoundary>,
    pub back: Vec<FaceBoundary>,
}

#[derive(Clone, Copy, Debug)]
pub struct Numerics {
    pub maximum_step_s: f64,
    pub energy_absolute_tolerance_j: f64,
    pub relative_tolerance: f64,
    pub mass_absolute_tolerance_kg: f64,
}

#[derive(Clone, Debug)]
struct Cell {
    temperature_k: f64,
    masses_kg: Vec<f64>,
}
#[derive(Clone, Debug)]
pub struct State {
    time_origin: String,
    grid: Grid,
    material: Material,
    constants: Constants,
    cells: Vec<Cell>,
    time_s: f64,
    initial_energy_j: f64,
    initial_fuel_kg: f64,
    released_kg: f64,
    boundary_energy_j: f64,
    gas_sensible_energy_j: f64,
    reaction_energy_j: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FlameStatus {
    UnsupportedGasPhaseClosure,
}
#[derive(Clone, Debug)]
pub struct Report {
    pub time_origin: String,
    pub time_s: f64,
    pub temperature_k: Vec<f64>,
    pub conversion: Vec<f64>,
    /// Each interval bounds columns whose through-thickness conversion exceeds the supplied level.
    /// Quantization is one dx; this is not a luminous contour or an interpolated precise front.
    pub solid_conversion_intervals_m: Vec<[f64; 2]>,
    pub remaining_reactant_kg: f64,
    pub released_volatile_kg: f64,
    pub stored_sensible_energy_j: f64,
    pub mass_residual_kg: f64,
    pub energy_residual_j: f64,
    pub sample_exhausted_at_resolution: bool,
    pub no_initial_fuel: bool,
    pub luminous_front: Option<Vec<[f64; 2]>>,
    pub flame_status: FlameStatus,
    pub quench_time_s: Option<f64>,
    pub view_lost: bool,
}

impl State {
    pub fn new(
        time_origin: &str,
        grid: Grid,
        material: Material,
        constants: Constants,
        time_s: f64,
        temperatures_k: Vec<f64>,
    ) -> Result<Self> {
        grid.validate()?;
        material.validate()?;
        constants.validate()?;
        require(
            !time_origin.trim().is_empty()
                && finite(time_s)
                && temperatures_k.len() == grid.nx * grid.nz,
            "invalid time origin/initial time/temperature field",
        )?;
        let mut cells = Vec::with_capacity(temperatures_k.len());
        for t in temperatures_k {
            material.at(t)?;
            cells.push(Cell {
                temperature_k: t,
                masses_kg: material
                    .reactants
                    .iter()
                    .map(|r| r.loading_kg_per_m3.si_value * grid.volume())
                    .collect(),
            });
        }
        let mut state = Self {
            time_origin: time_origin.into(),
            grid,
            material,
            constants,
            cells,
            time_s,
            initial_energy_j: 0.0,
            initial_fuel_kg: 0.0,
            released_kg: 0.0,
            boundary_energy_j: 0.0,
            gas_sensible_energy_j: 0.0,
            reaction_energy_j: 0.0,
        };
        state.initial_energy_j = state.energy();
        state.initial_fuel_kg = state.fuel();
        require(
            finite(state.initial_energy_j) && finite(state.initial_fuel_kg),
            "initial inventory overflow",
        )?;
        Ok(state)
    }
    fn capacity(&self, cell: &Cell) -> f64 {
        self.grid.volume()
            * self.material.inert_loading.si_value
            * self.material.inert_heat_capacity.si_value
            + cell
                .masses_kg
                .iter()
                .zip(&self.material.reactants)
                .map(|(m, r)| m * r.heat_capacity.si_value)
                .sum::<f64>()
    }
    fn energy(&self) -> f64 {
        self.cells
            .iter()
            .map(|c| self.capacity(c) * (c.temperature_k - self.material.reference_temperature_k))
            .sum()
    }
    fn fuel(&self) -> f64 {
        self.cells.iter().flat_map(|c| &c.masses_kg).sum()
    }
    fn validate_control(&self, control: &Control) -> Result<()> {
        control.environment.validate()?;
        control.boundary_valid_for.validate()?;
        require(finite(control.start_s) && finite(control.end_s) && control.start_s < control.end_s
            && control.environment == control.boundary_valid_for
            && control.environment.pressure_pa == self.material.pressure_pa
            && !control.boundary_source.trim().is_empty()
            && control.front.len() == self.grid.nx && control.back.len() == self.grid.nx,
            "invalid history or boundary closure not valid for supplied flow/oxygen/gravity/orientation")?;
        for b in control.front.iter().chain(&control.back) {
            b.validate(&self.material)?;
        }
        Ok(())
    }
    /// A physical positivity/CFL bound, not an accuracy guarantee.
    pub fn stable_step_s(&self, control: &Control) -> Result<f64> {
        self.validate_control(control)?;
        let g = self.grid;
        let gx = self.material.effective_conductivity.si_value * g.dz() * g.width_m / g.dx();
        let gz = self.material.effective_conductivity.si_value * g.area() / g.dz();
        let mut bound = f64::INFINITY;
        for x in 0..g.nx {
            for z in 0..g.nz {
                let i = x * g.nz + z;
                let mut sum = 0.0;
                if x > 0 {
                    sum += gx;
                }
                if x + 1 < g.nx {
                    sum += gx;
                }
                if z > 0 {
                    sum += gz;
                }
                if z + 1 < g.nz {
                    sum += gz;
                }
                if z == 0 {
                    sum += g.area() * control.front[x].lipschitz(self.constants);
                }
                if z + 1 == g.nz {
                    sum += g.area() * control.back[x].lipschitz(self.constants);
                }
                require(
                    finite(sum) && positive(self.capacity(&self.cells[i])),
                    "conductance/capacity overflow",
                )?;
                if sum > 0.0 {
                    bound = bound.min(self.capacity(&self.cells[i]) / sum);
                }
            }
        }
        require(bound > 0.0, "time-step underflow")?;
        Ok(bound)
    }
    /// Transactional step: every rejection leaves the original state unchanged.
    pub fn advance(&mut self, dt_s: f64, control: &Control, n: Numerics) -> Result<()> {
        require(
            positive(n.maximum_step_s)
                && finite(n.energy_absolute_tolerance_j)
                && n.energy_absolute_tolerance_j >= 0.0
                && finite(n.mass_absolute_tolerance_kg)
                && n.mass_absolute_tolerance_kg >= 0.0
                && finite(n.relative_tolerance)
                && (0.0..1.0).contains(&n.relative_tolerance),
            "invalid numerical budget",
        )?;
        require(
            positive(dt_s)
                && dt_s <= n.maximum_step_s
                && self.time_s >= control.start_s
                && self.time_s + dt_s <= control.end_s
                && self.time_s + dt_s > self.time_s,
            "step outside declared control interval or numerical budget",
        )?;
        require(
            dt_s <= self.stable_step_s(control)?,
            "heat step violates positivity bound",
        )?;
        let g = self.grid;
        let k = self.material.effective_conductivity.si_value;
        let gx = k * g.dz() * g.width_m / g.dx();
        let gz = k * g.area() / g.dz();
        let mut heat = vec![0.0; self.cells.len()];
        let mut boundary = 0.0;
        for x in 0..g.nx {
            for z in 0..g.nz {
                let i = x * g.nz + z;
                for (j, conductance) in [
                    (if x + 1 < g.nx { Some(i + g.nz) } else { None }, gx),
                    (if z + 1 < g.nz { Some(i + 1) } else { None }, gz),
                ] {
                    if let Some(j) = j {
                        let q = conductance
                            * (self.cells[j].temperature_k - self.cells[i].temperature_k)
                            * dt_s;
                        heat[i] += q;
                        heat[j] -= q;
                    }
                }
                if z == 0 {
                    let q = g.area()
                        * control.front[x].flux(self.cells[i].temperature_k, self.constants)?
                        * dt_s;
                    heat[i] += q;
                    boundary += q;
                }
                if z + 1 == g.nz {
                    let q = g.area()
                        * control.back[x].flux(self.cells[i].temperature_k, self.constants)?
                        * dt_s;
                    heat[i] += q;
                    boundary += q;
                }
            }
        }
        let mut next = self.clone();
        for (i, cell) in self.cells.iter().enumerate() {
            let cp = self.capacity(cell);
            let heated = cell.temperature_k + heat[i] / cp;
            self.material.at(heated)?;
            let mut released_sensible = 0.0;
            let mut reaction = 0.0;
            for (a, r) in self.material.reactants.iter().enumerate() {
                let rate = r.preexponential.si_value
                    * (-r.activation_energy.si_value
                        / (self.constants.gas_constant_j_mol_k * heated))
                        .exp();
                let extent = -(-rate * dt_s).exp_m1();
                let dm = cell.masses_kg[a] * extent;
                require(
                    finite(dm) && dm >= 0.0 && dm <= cell.masses_kg[a],
                    "reaction overflow/negative mass",
                )?;
                // Preserve the exponential tail when subtraction of dm would round to zero.
                next.cells[i].masses_kg[a] = cell.masses_kg[a] * (-rate * dt_s).exp();
                next.released_kg += dm;
                released_sensible += dm
                    * r.heat_capacity.si_value
                    * (heated - self.material.reference_temperature_k);
                reaction += dm * r.endothermic_reaction_enthalpy.si_value;
            }
            let energy = cp * (heated - self.material.reference_temperature_k)
                - released_sensible
                - reaction;
            let temperature =
                self.material.reference_temperature_k + energy / next.capacity(&next.cells[i]);
            self.material.at(temperature)?;
            next.cells[i].temperature_k = temperature;
            next.gas_sensible_energy_j += released_sensible;
            next.reaction_energy_j += reaction;
        }
        next.boundary_energy_j += boundary;
        next.time_s += dt_s;
        let mass_residual = next.initial_fuel_kg - next.fuel() - next.released_kg;
        let energy_scale = next.initial_energy_j.abs()
            + next.boundary_energy_j.abs()
            + next.gas_sensible_energy_j.abs()
            + next.reaction_energy_j.abs()
            + next.energy().abs();
        let energy_residual = next.energy_residual();
        require(
            finite(mass_residual)
                && mass_residual.abs()
                    <= n.mass_absolute_tolerance_kg + n.relative_tolerance * next.initial_fuel_kg
                && finite(energy_scale)
                && finite(energy_residual)
                && energy_residual.abs()
                    <= n.energy_absolute_tolerance_j + n.relative_tolerance * energy_scale,
            "mass or energy balance exceeds declared arithmetic tolerance",
        )?;
        *self = next;
        Ok(())
    }
    fn energy_residual(&self) -> f64 {
        self.initial_energy_j + self.boundary_energy_j
            - self.energy()
            - self.gas_sensible_energy_j
            - self.reaction_energy_j
    }
    pub fn report(
        &self,
        conversion_level: f64,
        residual_mass_resolution_kg: f64,
        view_available: bool,
    ) -> Result<Report> {
        require(
            finite(conversion_level)
                && conversion_level > 0.0
                && conversion_level <= 1.0
                && finite(residual_mass_resolution_kg)
                && residual_mass_resolution_kg >= 0.0,
            "invalid observation/conversion resolution",
        )?;
        let initial_cell: f64 = self
            .material
            .reactants
            .iter()
            .map(|r| r.loading_kg_per_m3.si_value * self.grid.volume())
            .sum();
        let conversion: Vec<f64> = self
            .cells
            .iter()
            .map(|c| {
                if initial_cell > 0.0 {
                    1.0 - c.masses_kg.iter().sum::<f64>() / initial_cell
                } else {
                    0.0
                }
            })
            .collect();
        let mut intervals = Vec::new();
        let mut start = None;
        for x in 0..self.grid.nx {
            let average = conversion[x * self.grid.nz..(x + 1) * self.grid.nz]
                .iter()
                .sum::<f64>()
                / self.grid.nz as f64;
            let active = initial_cell > 0.0 && average >= conversion_level;
            if active && start.is_none() {
                start = Some(x);
            }
            if !active {
                if let Some(s) = start.take() {
                    intervals.push([s as f64 * self.grid.dx(), x as f64 * self.grid.dx()]);
                }
            }
        }
        if let Some(s) = start {
            intervals.push([s as f64 * self.grid.dx(), self.grid.length_m]);
        }
        let fuel = self.fuel();
        Ok(Report {
            time_origin: self.time_origin.clone(),
            time_s: self.time_s,
            temperature_k: self.cells.iter().map(|c| c.temperature_k).collect(),
            conversion,
            solid_conversion_intervals_m: intervals,
            remaining_reactant_kg: fuel,
            released_volatile_kg: self.released_kg,
            stored_sensible_energy_j: self.energy(),
            mass_residual_kg: self.initial_fuel_kg - fuel - self.released_kg,
            energy_residual_j: self.energy_residual(),
            sample_exhausted_at_resolution: self.initial_fuel_kg > 0.0
                && residual_mass_resolution_kg > 0.0
                && fuel <= residual_mass_resolution_kg,
            no_initial_fuel: self.initial_fuel_kg == 0.0,
            luminous_front: None,
            flame_status: FlameStatus::UnsupportedGasPhaseClosure,
            quench_time_s: None,
            view_lost: !view_available,
        })
    }
}

/// Exact lumped, uniform-temperature, inert, prescribed-net-flux energy limit.
pub fn thermally_thin_temperature(
    t0: f64,
    net_flux_w_m2: f64,
    duration_s: f64,
    areal_capacity_j_m2_k: f64,
) -> Result<f64> {
    require(
        positive(t0)
            && finite(net_flux_w_m2)
            && finite(duration_s)
            && duration_s >= 0.0
            && positive(areal_capacity_j_m2_k),
        "invalid thin-limit input",
    )?;
    let t = t0 + net_flux_w_m2 * duration_s / areal_capacity_j_m2_k;
    require(positive(t), "invalid thin-limit output")?;
    Ok(t)
}

/// Exact semi-infinite homogeneous inert body, constant imposed *net* flux, initially uniform.
pub fn thermally_thick_surface_temperature(
    t0: f64,
    net_flux_w_m2: f64,
    duration_s: f64,
    conductivity_w_m_k: f64,
    capacity_j_m3_k: f64,
) -> Result<f64> {
    require(
        positive(t0)
            && finite(net_flux_w_m2)
            && finite(duration_s)
            && duration_s >= 0.0
            && positive(conductivity_w_m_k)
            && positive(capacity_j_m3_k),
        "invalid thick-limit input",
    )?;
    let t = t0
        + 2.0
            * net_flux_w_m2
            * (duration_s / (std::f64::consts::PI * conductivity_w_m_k * capacity_j_m3_k)).sqrt();
    require(positive(t), "invalid thick-limit output")?;
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    // All material/boundary numbers here are deliberately synthetic dimensions, not NASA data.
    fn p(value: f64, unit: Unit) -> Property {
        Property {
            raw_value: value,
            raw_unit: unit,
            si_value: value,
            si_unit: unit,
            phase: Phase::Solid,
            source: "synthetic_fixture".into(),
            locator: "inline invariant".into(),
            origin: Origin::SyntheticFixture,
            uncertainty_absolute: Some(0.0),
            temperature_domain_k: [200.0, 2000.0],
            pressure_domain_pa: [100000.0, 110000.0],
            reference_temperature_k: 300.0,
            reference_pressure_pa: 100000.0,
            reference_identity: "synthetic sensible-only".into(),
        }
    }
    fn constants() -> Constants {
        Constants {
            gas_constant_j_mol_k: 8.31446261815324,
            stefan_boltzmann_w_m2_k4: 5.670374419e-8,
        }
    }
    fn material(fuel: bool) -> Material {
        Material {
            inert_loading: p(100.0, Unit::KgPerM3),
            inert_heat_capacity: p(1000.0, Unit::JPerKgK),
            effective_conductivity: p(1.0, Unit::WPerMK),
            reference_temperature_k: 300.0,
            pressure_pa: 100000.0,
            reactants: if fuel {
                vec![Reactant {
                    composition_id: "invented constituent".into(),
                    loading_kg_per_m3: p(100.0, Unit::KgPerM3),
                    heat_capacity: p(1000.0, Unit::JPerKgK),
                    preexponential: p(1.0, Unit::PerSecond),
                    activation_energy: p(0.0, Unit::JPerMol),
                    endothermic_reaction_enthalpy: p(0.0, Unit::JPerKg),
                }]
            } else {
                vec![]
            },
        }
    }
    fn grid(nx: usize, nz: usize) -> Grid {
        Grid {
            nx,
            nz,
            length_m: 1.0,
            thickness_m: 1.0,
            width_m: 1.0,
        }
    }
    fn face(q: f64) -> FaceBoundary {
        let interface = |v, u| {
            let mut x = p(v, u);
            x.phase = Phase::Interface;
            x
        };
        FaceBoundary {
            gas_temperature_k: 300.0,
            radiant_temperature_k: 300.0,
            heat_transfer: interface(0.0, Unit::WPerM2K),
            emissivity: interface(0.0, Unit::One),
            absorbed_incident_flux: interface(q, Unit::WPerM2),
        }
    }
    fn control(nx: usize, q: f64) -> Control {
        let e = Environment {
            flow_velocity_m_s: [0.0, 0.0, 0.0],
            gravity_m_s2: [0.0, 0.0, 0.0],
            surface_normal: [0.0, 0.0, 1.0],
            oxygen_mole_fraction: 0.0,
            pressure_pa: 100000.0,
            normal_tolerance: 1e-12,
        };
        Control {
            start_s: 0.0,
            end_s: 100.0,
            environment: e.clone(),
            boundary_valid_for: e,
            boundary_source: "synthetic prescribed boundary".into(),
            pilot_on: false,
            view_available: true,
            front: vec![face(q); nx],
            back: vec![face(0.0); nx],
        }
    }
    fn numerics() -> Numerics {
        Numerics {
            maximum_step_s: 1.0,
            energy_absolute_tolerance_j: 1e-6,
            mass_absolute_tolerance_kg: 1e-10,
            relative_tolerance: 1e-11,
        }
    }
    fn close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() <= tol, "{a} != {b}, tolerance {tol}");
    }

    #[test]
    fn no_flux_no_fuel_and_exact_lumped_limit() {
        let mut s = State::new(
            "synthetic initial-state time zero",
            grid(1, 1),
            material(false),
            constants(),
            0.0,
            vec![300.0],
        )
        .unwrap();
        s.advance(1.0, &control(1, 0.0), numerics()).unwrap();
        let r = s.report(0.5, 0.0, true).unwrap();
        assert_eq!(r.temperature_k, vec![300.0]);
        assert!(r.no_initial_fuel);
        assert!(!r.sample_exhausted_at_resolution);
        assert!(r.solid_conversion_intervals_m.is_empty());
        s.advance(1.0, &control(1, 100.0), numerics()).unwrap();
        let expected = thermally_thin_temperature(300.0, 100.0, 1.0, 100000.0).unwrap();
        close(
            s.report(0.5, 0.0, true).unwrap().temperature_k[0],
            expected,
            1e-12,
        );
        assert_eq!(
            thermally_thin_temperature(300.0, 0.0, 5.0, 1.0).unwrap(),
            300.0
        );
        assert_eq!(
            thermally_thick_surface_temperature(300.0, 0.0, 5.0, 1.0, 1.0).unwrap(),
            300.0
        );
        assert_eq!(
            thermally_thick_surface_temperature(300.0, 100.0, 0.0, 1.0, 1.0).unwrap(),
            300.0
        );
    }

    #[test]
    fn mass_and_energy_conserved_with_reaction_and_gas_sensible_export() {
        let mut m = material(true);
        m.reactants[0].endothermic_reaction_enthalpy = p(1000.0, Unit::JPerKg);
        let mut s = State::new(
            "synthetic initial-state time zero",
            grid(2, 2),
            m,
            constants(),
            0.0,
            vec![400.0, 410.0, 420.0, 430.0],
        )
        .unwrap();
        for _ in 0..100 {
            s.advance(0.01, &control(2, 100.0), numerics()).unwrap();
        }
        let r = s.report(0.5, 1e-9, false).unwrap();
        close(r.remaining_reactant_kg, 100.0 * (-1.0f64).exp(), 1e-10);
        close(r.mass_residual_kg, 0.0, 1e-10);
        close(r.energy_residual_j, 0.0, 1e-6);
        assert!(s.gas_sensible_energy_j > 0.0 && s.reaction_energy_j > 0.0);
        assert!(r.temperature_k.iter().all(|t| *t > 200.0));
        assert!(r.view_lost && r.luminous_front.is_none() && r.quench_time_s.is_none());
    }

    #[test]
    fn complete_finite_reactant_exhaustion_is_not_quench_or_view_loss() {
        let mut m = material(true);
        m.reactants[0].preexponential = p(1000.0, Unit::PerSecond);
        let mut s = State::new(
            "synthetic initial-state time zero",
            grid(1, 1),
            m,
            constants(),
            0.0,
            vec![400.0],
        )
        .unwrap();
        s.advance(1.0, &control(1, 0.0), numerics()).unwrap();
        let r = s.report(1.0, 1e-12, true).unwrap();
        assert!(r.sample_exhausted_at_resolution && !r.view_lost);
        assert_eq!(r.remaining_reactant_kg, 0.0);
        assert_eq!(r.released_volatile_kg, 100.0);
        assert_eq!(r.solid_conversion_intervals_m, vec![[0.0, 1.0]]);
        assert!(r.luminous_front.is_none() && r.quench_time_s.is_none());
        close(r.temperature_k[0], 400.0, 1e-12);
    }

    #[test]
    fn spatial_conversion_can_be_disconnected_and_remains_inside_sample() {
        let mut m = material(true);
        m.reactants[0].activation_energy = p(10000.0, Unit::JPerMol);
        let mut s = State::new(
            "synthetic initial-state time zero",
            grid(3, 1),
            m,
            constants(),
            0.0,
            vec![600.0, 300.0, 600.0],
        )
        .unwrap();
        s.advance(1.0, &control(3, 0.0), numerics()).unwrap();
        let r = s.report(0.05, 0.0, true).unwrap();
        assert_eq!(
            r.solid_conversion_intervals_m,
            vec![[0.0, 1.0 / 3.0], [2.0 / 3.0, 1.0]]
        );
    }

    #[test]
    fn conduction_smooths_without_losing_energy_and_stability_violation_is_atomic() {
        let mut s = State::new(
            "synthetic initial-state time zero",
            grid(2, 1),
            material(false),
            constants(),
            0.0,
            vec![300.0, 500.0],
        )
        .unwrap();
        let ctrl = control(2, 0.0);
        let limit = s.stable_step_s(&ctrl).unwrap();
        let mut n = numerics();
        n.maximum_step_s = limit * 2.0;
        let mut long_ctrl = ctrl.clone();
        long_ctrl.end_s = limit * 3.0;
        assert!(s.advance(limit * 2.0, &long_ctrl, n).is_err());
        assert_eq!(
            s.report(0.5, 0.0, true).unwrap().temperature_k,
            vec![300.0, 500.0]
        );
        s.advance(1.0, &ctrl, numerics()).unwrap();
        let r = s.report(0.5, 0.0, true).unwrap();
        assert!(r.temperature_k[0] > 300.0 && r.temperature_k[1] < 500.0);
        close(r.energy_residual_j, 0.0, 1e-6);
    }

    #[test]
    fn thick_mesh_converges_to_semi_infinite_surface_solution() {
        let run = |nz, steps| {
            let mut g = grid(1, nz);
            g.thickness_m = 0.1;
            let mut s = State::new(
                "synthetic initial-state time zero",
                g,
                material(false),
                constants(),
                0.0,
                vec![300.0; nz],
            )
            .unwrap();
            let c = control(1, 100.0);
            for _ in 0..steps {
                s.advance(1.0 / steps as f64, &c, numerics()).unwrap();
            }
            s.report(0.5, 0.0, true).unwrap().temperature_k[0] + 100.0 * g.dz() / 2.0
        };
        let exact = thermally_thick_surface_temperature(300.0, 100.0, 1.0, 1.0, 100000.0).unwrap();
        let coarse = run(40, 400);
        let fine = run(80, 1600);
        assert!((fine - exact).abs() < (coarse - exact).abs());
        close(fine, exact, 0.01);
    }

    #[test]
    fn reaction_temperature_split_converges_as_step_decreases() {
        let run = |steps| {
            let mut m = material(true);
            m.reactants[0].activation_energy = p(10000.0, Unit::JPerMol);
            m.reactants[0].endothermic_reaction_enthalpy = p(100000.0, Unit::JPerKg);
            let mut s = State::new(
                "synthetic initial-state time zero",
                grid(1, 1),
                m,
                constants(),
                0.0,
                vec![600.0],
            )
            .unwrap();
            for _ in 0..steps {
                s.advance(1.0 / steps as f64, &control(1, 10000.0), numerics())
                    .unwrap();
            }
            s.report(0.5, 0.0, true).unwrap().temperature_k[0]
        };
        let coarse = run(10);
        let medium = run(20);
        let fine = run(40);
        assert!((fine - medium).abs() < (medium - coarse).abs());
    }

    #[test]
    fn provenance_units_domains_and_context_fail_closed() {
        let make = |m| {
            State::new(
                "synthetic initial-state time zero",
                grid(1, 1),
                m,
                constants(),
                0.0,
                vec![300.0],
            )
        };
        let mut m = material(false);
        m.inert_heat_capacity.raw_unit = Unit::JPerMol;
        assert!(make(m).is_err());
        let mut m = material(false);
        m.inert_heat_capacity.reference_identity = "different".into();
        assert!(make(m).is_err());
        let mut m = material(false);
        m.effective_conductivity.reference_identity = "different".into();
        assert!(make(m).is_err());
        let mut m = material(false);
        m.inert_heat_capacity.source.clear();
        assert!(make(m).is_err());
        let mut m = material(false);
        m.inert_heat_capacity.temperature_domain_k = [400.0, 500.0];
        assert!(make(m).is_err());
        let mut m = material(false);
        m.inert_heat_capacity.uncertainty_absolute = Some(f64::NAN);
        assert!(make(m).is_err());
        let mut m = material(false);
        m.inert_heat_capacity.reference_temperature_k = 301.0;
        assert!(make(m).is_err());
        let mut s = make(material(false)).unwrap();
        let mut c = control(1, 0.0);
        c.environment.oxygen_mole_fraction = 0.2;
        assert!(s.advance(1.0, &c, numerics()).is_err());
        let mut c = control(1, 0.0);
        c.front[0].absorbed_incident_flux = {
            let mut q = p(1e100, Unit::WPerM2);
            q.phase = Phase::Interface;
            q
        };
        assert!(s.advance(1.0, &c, numerics()).is_err());
        assert_eq!(s.report(0.5, 0.0, true).unwrap().temperature_k, vec![300.0]);
        assert!(s.advance(f64::NAN, &control(1, 0.0), numerics()).is_err());
        assert!(s.report(0.0, 0.0, true).is_err());
    }

    #[test]
    fn composition_properties_drive_conversion_without_name_branches() {
        let mut m = material(true);
        let mut second = m.reactants[0].clone();
        second.composition_id = "another invented constituent".into();
        second.loading_kg_per_m3 = p(50.0, Unit::KgPerM3);
        second.preexponential = p(2.0, Unit::PerSecond);
        second.heat_capacity = p(2000.0, Unit::JPerKgK);
        m.reactants.push(second);
        let mut renamed = m.clone();
        renamed.reactants[0].composition_id = "arbitrary label x".into();
        renamed.reactants[1].composition_id = "arbitrary label y".into();
        let run = |material| {
            let mut s = State::new(
                "synthetic time zero",
                grid(1, 1),
                material,
                constants(),
                0.0,
                vec![400.0],
            )
            .unwrap();
            s.advance(1.0, &control(1, 0.0), numerics()).unwrap();
            s.report(0.5, 0.0, true).unwrap()
        };
        let a = run(m);
        let b = run(renamed);
        close(
            a.remaining_reactant_kg,
            100.0 * (-1.0f64).exp() + 50.0 * (-2.0f64).exp(),
            1e-10,
        );
        assert_eq!(a.remaining_reactant_kg, b.remaining_reactant_kg);
        assert_eq!(a.temperature_k, b.temperature_k);
        close(a.temperature_k[0], 400.0, 1e-12);
        close(a.energy_residual_j, 0.0, 1e-6);
    }

    #[test]
    fn control_history_changes_heating_and_boundary_cooling_is_conservative() {
        let mut s = State::new(
            "synthetic control time zero",
            grid(1, 1),
            material(false),
            constants(),
            0.0,
            vec![400.0],
        )
        .unwrap();
        let mut c = control(1, 1000.0);
        c.pilot_on = true;
        c.end_s = 1.0;
        s.advance(1.0, &c, numerics()).unwrap();
        let hot = s.report(0.5, 0.0, true).unwrap().temperature_k[0];
        assert!(hot > 400.0);
        c.start_s = 1.0;
        c.end_s = 2.0;
        c.pilot_on = false;
        c.front[0] = face(0.0);
        c.front[0].heat_transfer = {
            let mut v = p(10.0, Unit::WPerM2K);
            v.phase = Phase::Interface;
            v
        };
        c.front[0].emissivity = {
            let mut v = p(0.5, Unit::One);
            v.phase = Phase::Interface;
            v
        };
        s.advance(1.0, &c, numerics()).unwrap();
        let r = s.report(0.5, 0.0, true).unwrap();
        assert!(r.temperature_k[0] < hot && r.temperature_k[0] > 300.0);
        close(r.energy_residual_j, 0.0, 1e-6);
        for e in [
            Environment {
                flow_velocity_m_s: [1.0, 0.0, 0.0],
                ..c.environment.clone()
            },
            Environment {
                gravity_m_s2: [0.0, 0.0, 1.0],
                ..c.environment.clone()
            },
            Environment {
                surface_normal: [1.0, 0.0, 0.0],
                ..c.environment.clone()
            },
        ] {
            c.environment = e;
            assert!(s.stable_step_s(&c).is_err());
        }
    }
}

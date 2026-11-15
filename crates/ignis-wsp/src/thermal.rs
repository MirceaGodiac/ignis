//! Explicit lumped-liquid mass/energy balance with a quasi-steady spherical gas film.
//!
//! This is a numerical law, not a property database. Every material/environment parameter is
//! authored in the release. Constant liquid density/heat capacity, constant gas heat capacity,
//! arithmetic-mean film temperature, reciprocal diffusivity mixing and linear conductivity
//! mixing are declared approximations; O2 is included with the N2 carrier. No gas-film memory,
//! internal liquid gradient, condensation, chemical heating or radiative flux is represented.

use ignis_core::contracts::{TensorRole, TensorSpec};
use ignis_core::expert::Refusal;
use serde::Serialize;

use crate::spec::{authored, sparse, Params};

/// Explicitly authored zero property blocks permit an exact disabled predecessor. Enabling
/// requires every positive property/domain below; material properties are never tacit defaults.
pub fn tensors() -> Vec<TensorSpec> {
    let mut out = Vec::new();
    let mut add = |name: &str, shape: &[u32], unit: &str, description: &str| {
        let role = if matches!(
            name,
            "on" | "nitrogen_only"
                | "cutoff_on"
                | "apply"
                | "dt_s"
                | "vapor_extension"
                | "latent_extrapolation"
        ) {
            TensorRole::Weight
        } else {
            TensorRole::PhysicalParameter
        };
        let mut t = if role == TensorRole::PhysicalParameter {
            authored(
                &format!("thermal.{name}"),
                shape,
                role,
                Some(unit),
                None,
                description,
            )
        } else {
            sparse(&format!("thermal.{name}"), shape, role, None, description)
        };
        t.unit = Some(unit.into());
        out.push(t);
    };
    add(
        "on",
        &[],
        "1",
        "Binary switch. 0 exactly preserves droplet6; 1 enables the thermal block, overriding sc/post and the independent dying-flame mass loss for selected last-flame kinds.",
    );
    add(
        "nitrogen_only",
        &[],
        "1",
        "Binary: 1 limits the entire thermal mechanism to runs with x_CO2=x_He=0 (O2 included with N2 carrier); other carrier compositions explicitly retain all predecessor laws. 0 enables the declared approximate mixture rules with all authored binary properties.",
    );
    add(
        "cutoff_on",
        &[],
        "1",
        "Binary: 0 preserves predecessor no-flame prefix/filter state and its evolution; 1 explicitly restarts the authored hot/unknown liquid and cloud state at cutoff, without pre-cutoff thermal-history inference.",
    );
    add(
        "apply",
        &[3],
        "1",
        "Binary applicability by last flame: hot radiative/unknown, hot diffusive, cool. Zero retains the predecessor law for that kind.",
    );
    add(
        "dt_s",
        &[],
        "s",
        "Explicit midpoint with authored maximum step; final partial step and deterministic q-floor event substeps. Invalid states refuse rather than silently clipping temperature. Convergence must be checked for the authored step.",
    );
    add(
        "gas_constant_j_mol_k",
        &[],
        "J/(mol K)",
        "Molar ideal-gas constant R.",
    );
    add("molar_mass_kg_mol", &[], "kg/mol", "Fuel molecular mass M.");
    add(
        "rho_kg_m3",
        &[],
        "kg/m3",
        "Constant liquid density approximation.",
    );
    add(
        "cp_liquid_j_kg_k",
        &[],
        "J/(kg K)",
        "Constant liquid heat-capacity approximation.",
    );
    add(
        "cp_gas_j_kg_k",
        &[],
        "J/(kg K)",
        "Constant effective gas-film heat capacity in the Stefan heat-transfer correction.",
    );
    add(
        "k_gas_w_m_k",
        &[3],
        "W/(m K)",
        "Reference conductivities in N2/O2-carrier, CO2, He order; linear mole-fraction mixture, no temperature correction.",
    );
    add(
        "diffusivity_ref_m2_s",
        &[3],
        "m2/s",
        "Fuel binary diffusivities in N2/O2-carrier, CO2, He order. Reciprocal mixture rule, each scaled as (Tfilm/Tref)^exponent * pref/p.",
    );
    add(
        "transport_ref_k",
        &[],
        "K",
        "Reference temperature of the diffusivity approximation.",
    );
    add(
        "transport_ref_pa",
        &[],
        "Pa",
        "Reference pressure of binary diffusivities.",
    );
    add(
        "diffusivity_exponent",
        &[],
        "1",
        "Authored common temperature exponent of binary diffusivity.",
    );
    add(
        "antoine",
        &[3],
        "mixed",
        "A,B,C in log10(p_sat/bar)=A-B/(T/K+C); B and C in K.",
    );
    add(
        "antoine_range_k",
        &[2],
        "K",
        "Closed source validity interval of the Antoine correlation.",
    );
    add(
        "latent",
        &[4],
        "mixed",
        "A [kJ/mol], alpha [1], beta [1], Tc [K] in Lmol=A*exp(-alpha*T/Tc)*(1-T/Tc)^beta.",
    );
    add(
        "latent_range_k",
        &[2],
        "K",
        "Closed source validity interval of the latent-heat correlation.",
    );
    add(
        "vapor_extension",
        &[],
        "1",
        "Binary: 0 refuse outside Antoine source interval; 1 explicit boundary Clausius-Clapeyron extension with constant latent heat evaluated at the boundary. Not a silent Antoine extrapolation.",
    );
    add(
        "latent_extrapolation",
        &[],
        "1",
        "Binary: 0 refuse outside latent source interval; 1 explicitly extrapolate the stated analytic correlation within 0<T<Tc, never across the critical point.",
    );
    add("ambient_k", &[], "K", "Constant ambient gas temperature.");
    add(
        "cloud_excess_k",
        &[3],
        "K",
        "Nonnegative initial residual gas-temperature excess by last-flame kind, decaying exponentially; no measured or resolved gas-film history.",
    );
    add(
        "cloud_tau_s",
        &[3],
        "s",
        "Positive gas-cloud decay times by last-flame kind.",
    );
    add(
        "initial_k",
        &[3],
        "K",
        "Authored initial liquid temperatures by last-flame kind. At cutoff the hot/unknown value restarts without thermal-history inference.",
    );
    add(
        "activity",
        &[3],
        "1",
        "Fuel surface activities in [0,1] by last-flame kind; constant-composition approximation, no evolving water uptake.",
    );
    add(
        "x_infinity",
        &[],
        "1",
        "Far-field fuel mole fraction in [0,1). Saturated/supersaturated gas yields zero evaporation; condensation is omitted.",
    );
    add(
        "surface_fraction_max",
        &[],
        "1",
        "Strictly between x_infinity and 1. Initial liquid temperature is capped using analytical vapor-pressure inversion to stay below this fuel surface mole fraction; cap and temperature are reported. Subsequent crossings are refused.",
    );
    out.push(authored("thermal.member_weight", &[], TensorRole::Weight, Some("1"), Some((0.0, 1.0)), "Probability weight of the thermal member against the unchanged predecessor, combined as complete predictive distributions; 1 pure thermal, 0 predecessor with unused thermal block ignored. Must be authored even when thermal.on=0."));
    out
}

#[derive(Clone, Debug, Serialize)]
pub struct Thermal {
    pub nitrogen_only: bool,
    pub cutoff_on: bool,
    pub apply: [bool; 3],
    pub dt_s: f64,
    pub gas_constant_j_mol_k: f64,
    pub molar_mass_kg_mol: f64,
    pub rho_kg_m3: f64,
    pub cp_liquid_j_kg_k: f64,
    pub cp_gas_j_kg_k: f64,
    pub k_gas_w_m_k: [f64; 3],
    pub diffusivity_ref_m2_s: [f64; 3],
    pub transport_ref_k: f64,
    pub transport_ref_pa: f64,
    pub diffusivity_exponent: f64,
    pub antoine: [f64; 3],
    pub antoine_range_k: [f64; 2],
    pub latent: [f64; 4],
    pub latent_range_k: [f64; 2],
    pub vapor_extension: bool,
    pub latent_extrapolation: bool,
    pub ambient_k: f64,
    pub cloud_excess_k: [f64; 3],
    pub cloud_tau_s: [f64; 3],
    pub initial_k: [f64; 3],
    pub activity: [f64; 3],
    pub x_infinity: f64,
    pub surface_fraction_max: f64,
}

fn error(message: impl Into<String>) -> String {
    message.into()
}
fn binary(x: f64) -> Result<bool, String> {
    match x {
        0.0 => Ok(false),
        1.0 => Ok(true),
        _ => Err(error("thermal switch must be exactly 0 or 1")),
    }
}

impl Thermal {
    pub(crate) fn bind(p: &Params<'_>) -> Result<Option<Self>, Refusal> {
        let read = || -> Result<Option<Self>, String> {
            if !binary(p.s("thermal.on"))? {
                return Ok(None);
            }
            let s = |n: &str| p.s(&format!("thermal.{n}"));
            let v3 = |n: &str| -> [f64; 3] {
                p.v(&format!("thermal.{n}")).try_into().expect("spec shape")
            };
            let v2 = |n: &str| -> [f64; 2] {
                p.v(&format!("thermal.{n}")).try_into().expect("spec shape")
            };
            let apply = v3("apply");
            let value = Self {
                nitrogen_only: binary(s("nitrogen_only"))?,
                cutoff_on: binary(s("cutoff_on"))?,
                apply: [binary(apply[0])?, binary(apply[1])?, binary(apply[2])?],
                dt_s: s("dt_s"),
                gas_constant_j_mol_k: s("gas_constant_j_mol_k"),
                molar_mass_kg_mol: s("molar_mass_kg_mol"),
                rho_kg_m3: s("rho_kg_m3"),
                cp_liquid_j_kg_k: s("cp_liquid_j_kg_k"),
                cp_gas_j_kg_k: s("cp_gas_j_kg_k"),
                k_gas_w_m_k: v3("k_gas_w_m_k"),
                diffusivity_ref_m2_s: v3("diffusivity_ref_m2_s"),
                transport_ref_k: s("transport_ref_k"),
                transport_ref_pa: s("transport_ref_pa"),
                diffusivity_exponent: s("diffusivity_exponent"),
                antoine: v3("antoine"),
                antoine_range_k: v2("antoine_range_k"),
                latent: p.v("thermal.latent").try_into().expect("spec shape"),
                latent_range_k: v2("latent_range_k"),
                vapor_extension: binary(s("vapor_extension"))?,
                latent_extrapolation: binary(s("latent_extrapolation"))?,
                ambient_k: s("ambient_k"),
                cloud_excess_k: v3("cloud_excess_k"),
                cloud_tau_s: v3("cloud_tau_s"),
                initial_k: v3("initial_k"),
                activity: v3("activity"),
                x_infinity: s("x_infinity"),
                surface_fraction_max: s("surface_fraction_max"),
            };
            value.validate()?;
            Ok(Some(value))
        };
        read().map_err(|e| Refusal::failed("thermal_parameters", e))
    }

    pub fn supports_gas(&self, gas: Gas) -> bool {
        !self.nitrogen_only || (gas.x_co2 == 0.0 && gas.x_he == 0.0)
    }

    pub fn validate(&self) -> Result<(), String> {
        let positives = [
            self.dt_s,
            self.gas_constant_j_mol_k,
            self.molar_mass_kg_mol,
            self.rho_kg_m3,
            self.cp_liquid_j_kg_k,
            self.cp_gas_j_kg_k,
            self.transport_ref_k,
            self.transport_ref_pa,
            self.ambient_k,
            self.latent[0],
            self.latent[3],
            self.antoine[1],
        ];
        if positives
            .iter()
            .chain(self.k_gas_w_m_k.iter())
            .chain(self.diffusivity_ref_m2_s.iter())
            .chain(self.cloud_tau_s.iter())
            .chain(self.initial_k.iter())
            .any(|x| !x.is_finite() || *x <= 0.0)
        {
            return Err(error(
                "thermal positive property/temperature/timestep is missing or invalid",
            ));
        }
        if self
            .antoine
            .iter()
            .chain(self.latent.iter())
            .any(|x| !x.is_finite())
            || !self.diffusivity_exponent.is_finite()
            || self.latent[2] < 0.0
            || self
                .cloud_excess_k
                .iter()
                .any(|x| !x.is_finite() || *x < 0.0)
            || self
                .activity
                .iter()
                .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
            || !(0.0..1.0).contains(&self.x_infinity)
            || !(self.surface_fraction_max > self.x_infinity && self.surface_fraction_max < 1.0)
        {
            return Err(error(
                "invalid thermal correlation, activity, cloud or surface fraction",
            ));
        }
        for [lo, hi] in [self.antoine_range_k, self.latent_range_k] {
            if !(lo.is_finite() && hi.is_finite() && 0.0 < lo && lo < hi && hi < self.latent[3]) {
                return Err(error(
                    "thermal source intervals must be ordered, positive and below Tc",
                ));
            }
        }
        if self.antoine_range_k[0] + self.antoine[2] <= 0.0 {
            return Err(error(
                "Antoine interval crosses its denominator singularity",
            ));
        }
        Ok(())
    }

    /// Latent heat in J/kg, retaining the authored correlation and declared source interval.
    pub fn latent_heat(&self, t: f64) -> Result<f64, String> {
        if !(t > 0.0 && t < self.latent[3]) {
            return Err(error("liquid temperature outside 0<T<Tc"));
        }
        if !self.latent_extrapolation
            && !(self.latent_range_k[0]..=self.latent_range_k[1]).contains(&t)
        {
            return Err(error(format!(
                "latent heat at {t} K outside authored source interval"
            )));
        }
        let tr = t / self.latent[3];
        Ok(
            1000.0
                * self.latent[0]
                * (-self.latent[1] * tr).exp()
                * (1.0 - tr).powf(self.latent[2])
                / self.molar_mass_kg_mol,
        )
    }

    fn antoine_log_pa(&self, t: f64) -> f64 {
        100_000.0_f64.ln()
            + std::f64::consts::LN_10 * (self.antoine[0] - self.antoine[1] / (t + self.antoine[2]))
    }

    pub fn vapor_pressure_pa(&self, t: f64) -> Result<f64, String> {
        if !(t > 0.0 && t < self.latent[3]) {
            return Err(error("vapor-pressure temperature outside liquid domain"));
        }
        let [lo, hi] = self.antoine_range_k;
        if (lo..=hi).contains(&t) {
            return Ok(self.antoine_log_pa(t).exp());
        }
        if !self.vapor_extension {
            return Err(error(format!(
                "vapor pressure at {t} K outside authored Antoine interval"
            )));
        }
        let edge = if t < lo { lo } else { hi };
        let lm = self.latent_heat(edge)? * self.molar_mass_kg_mol;
        Ok(
            (self.antoine_log_pa(edge) + lm / self.gas_constant_j_mol_k * (1.0 / edge - 1.0 / t))
                .exp(),
        )
    }

    /// Analytical inverse of the same piecewise vapor-pressure law. No parameter solving.
    fn saturation_temperature(&self, pressure: f64) -> Result<f64, String> {
        let lp = pressure.ln();
        let [lo, hi] = self.antoine_range_k;
        if lp >= self.antoine_log_pa(lo) && lp <= self.antoine_log_pa(hi) {
            return Ok(
                self.antoine[1] / (self.antoine[0] - (pressure / 100_000.0).log10())
                    - self.antoine[2],
            );
        }
        if !self.vapor_extension {
            return Err(error(
                "pressure-cap temperature lies outside Antoine source interval",
            ));
        }
        let edge = if lp < self.antoine_log_pa(lo) { lo } else { hi };
        let lm = self.latent_heat(edge)? * self.molar_mass_kg_mol;
        let inverse =
            1.0 / edge - (lp - self.antoine_log_pa(edge)) * self.gas_constant_j_mol_k / lm;
        if inverse <= 0.0 {
            return Err(error(
                "Clausius-Clapeyron inverse has no positive temperature",
            ));
        }
        Ok(1.0 / inverse)
    }

    pub fn initial(
        &self,
        q_m2: f64,
        pressure_pa: f64,
        kind: usize,
    ) -> Result<(State, bool), String> {
        if kind >= 3
            || !(q_m2.is_finite() && q_m2 > 0.0 && pressure_pa.is_finite() && pressure_pa > 0.0)
        {
            return Err(error("invalid thermal initial state/kind/pressure"));
        }
        let authored = self.initial_k[kind];
        let mut temperature_k = authored;
        if self.activity[kind] > 0.0
            && self.activity[kind] * self.vapor_pressure_pa(authored)? / pressure_pa
                > self.surface_fraction_max
        {
            temperature_k = self.saturation_temperature(
                self.surface_fraction_max * pressure_pa / self.activity[kind],
            )?;
        }
        self.latent_heat(temperature_k)?;
        Ok((
            State {
                q_m2,
                temperature_k,
            },
            temperature_k < authored,
        ))
    }

    pub fn flux(&self, state: State, age_s: f64, gas: Gas, kind: usize) -> Result<Flux, String> {
        if kind >= 3
            || !self.supports_gas(gas)
            || !(age_s.is_finite() && age_s >= 0.0 && state.q_m2.is_finite() && state.q_m2 > 0.0)
            || !(gas.pressure_pa.is_finite() && gas.pressure_pa > 0.0)
            || !(gas.x_co2 >= 0.0 && gas.x_he >= 0.0 && gas.x_co2 + gas.x_he <= 1.0)
        {
            return Err(error(
                "invalid thermal state, age or unsupported carrier composition",
            ));
        }
        let t = state.temperature_k;
        let tg =
            self.ambient_k + self.cloud_excess_k[kind] * (-age_s / self.cloud_tau_s[kind]).exp();
        let tf = 0.5 * (t + tg);
        let x = [1.0 - gas.x_co2 - gas.x_he, gas.x_co2, gas.x_he];
        let d = (tf / self.transport_ref_k).powf(self.diffusivity_exponent) * self.transport_ref_pa
            / gas.pressure_pa
            / x.iter()
                .zip(self.diffusivity_ref_m2_s)
                .map(|(x, d)| x / d)
                .sum::<f64>();
        let kg = x
            .iter()
            .zip(self.k_gas_w_m_k)
            .map(|(x, k)| x * k)
            .sum::<f64>();
        let xs = self.activity[kind] * self.vapor_pressure_pa(t)? / gas.pressure_pa;
        // The tiny relative tolerance covers only analytical inversion roundoff, not extrapolation.
        if xs > self.surface_fraction_max * (1.0 + 32.0 * f64::EPSILON) || xs >= 1.0 {
            return Err(error(format!(
                "thermal surface fraction {xs} exceeds authored bound; reduce dt or revise the declared state"
            )));
        }
        let log = ((1.0 - self.x_infinity).ln() - (-xs).ln_1p()).max(0.0);
        let k = 8.0 * self.molar_mass_kg_mol * gas.pressure_pa * d
            / (self.rho_kg_m3 * self.gas_constant_j_mol_k * tf)
            * log;
        let beta = self.rho_kg_m3 * k * self.cp_gas_j_kg_k / (8.0 * kg);
        let correction = if beta == 0.0 {
            1.0
        } else {
            beta / beta.exp_m1()
        };
        let latent = self.latent_heat(t)?;
        let heating = 12.0 * kg * (tg - t) * correction / self.rho_kg_m3;
        let dt = (heating - 1.5 * latent * k) / (self.cp_liquid_j_kg_k * state.q_m2);
        if ![k, dt, heating, latent, xs, beta]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err(error("non-finite thermal flux"));
        }
        Ok(Flux {
            k_m2_s: k,
            dt_k_s: dt,
            heating,
            latent_j_kg: latent,
            surface_fraction: xs,
        })
    }

    /// Explicit midpoint integration, authored maximum dt, with a final partial step and
    /// q-floor event substeps. A first-order predictor can shorten a step but never stop a path. The q floor
    /// is an absorbing numerical terminal event; residual liquid at that floor is not evaporated.
    pub fn evolve(
        &self,
        state: State,
        start_s: f64,
        end_s: f64,
        gas: Gas,
        kind: usize,
        q_floor: f64,
    ) -> Result<State, String> {
        self.advance(
            state,
            start_s,
            end_s,
            gas,
            kind,
            q_floor,
            &mut Trace::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn advance(
        &self,
        mut state: State,
        start_s: f64,
        end_s: f64,
        gas: Gas,
        kind: usize,
        q_floor: f64,
        trace: &mut Trace,
    ) -> Result<State, String> {
        if !(start_s.is_finite()
            && end_s.is_finite()
            && start_s >= 0.0
            && end_s >= start_s
            && q_floor.is_finite()
            && q_floor > 0.0)
        {
            return Err(error("invalid thermal interval or diameter floor"));
        }
        let mut age = start_s;
        while age < end_s && state.q_m2 > q_floor {
            let mut h = self.dt_s.min(end_s - age);
            if age + h <= age {
                return Err(error("thermal timestep cannot advance floating-point time"));
            }
            trace.record(self, state);
            let a = self.flux(state, age, gas, kind)?;
            trace.integrated_steps += 1;
            if a.k_m2_s > 0.0 {
                h = h.min((state.q_m2 - q_floor) / a.k_m2_s);
            }
            if age + h <= age {
                return Err(error(
                    "thermal q-floor event substep cannot advance floating-point time",
                ));
            }
            let mid = State {
                q_m2: state.q_m2 - 0.5 * h * a.k_m2_s,
                temperature_k: state.temperature_k + 0.5 * h * a.dt_k_s,
            };
            trace.record(self, mid);
            let b = self.flux(mid, age + 0.5 * h, gas, kind)?;
            // Only the evaluated midpoint flux can bracket the terminal diameter. If cooling
            // has reduced K since the predictor, keep the residual liquid and advance again.
            let event_h = if b.k_m2_s > 0.0 {
                h.min((state.q_m2 - q_floor) / b.k_m2_s)
            } else {
                h
            };
            let reached_floor = b.k_m2_s > 0.0 && (state.q_m2 - q_floor) / b.k_m2_s <= h;
            let next = State {
                q_m2: if reached_floor {
                    q_floor
                } else {
                    state.q_m2 - event_h * b.k_m2_s
                },
                temperature_k: state.temperature_k + event_h * b.dt_k_s,
            };
            if !next.temperature_k.is_finite()
                || next.temperature_k <= 0.0
                || next.temperature_k >= self.latent[3]
            {
                return Err(error(
                    "thermal integration left positive liquid-temperature domain; reduce authored dt",
                ));
            }
            if next.q_m2 > q_floor {
                trace.record(self, next);
                self.flux(next, age + h, gas, kind)?;
            }
            state = next;
            age += event_h;
        }
        Ok(state)
    }
}

/// Numerical execution facts, distinct from source validity and empirical qualification.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Trace {
    pub min_liquid_temperature_k: Option<f64>,
    pub max_liquid_temperature_k: Option<f64>,
    pub antoine_extension_evaluations: usize,
    pub latent_extrapolation_evaluations: usize,
    pub integrated_steps: usize,
    pub cached_trajectories: usize,
}

impl Trace {
    fn record(&mut self, th: &Thermal, state: State) {
        let t = state.temperature_k;
        self.min_liquid_temperature_k = Some(self.min_liquid_temperature_k.map_or(t, |v| v.min(t)));
        self.max_liquid_temperature_k = Some(self.max_liquid_temperature_k.map_or(t, |v| v.max(t)));
        self.antoine_extension_evaluations +=
            usize::from(!(th.antoine_range_k[0]..=th.antoine_range_k[1]).contains(&t));
        self.latent_extrapolation_evaluations +=
            usize::from(!(th.latent_range_k[0]..=th.latent_range_k[1]).contains(&t));
    }
}

/// Forecast-local cache of deterministic trajectories. Nodes stay on the same age/dt grid
/// regardless of requested horizons or query order; off-grid queries use one uncached partial
/// midpoint step. No state/parameter inference or interpolation across different droplets.
pub struct Trajectories<'a> {
    thermal: &'a Thermal,
    gas: Gas,
    q_floor: f64,
    nodes: std::collections::BTreeMap<(u64, usize), Vec<State>>,
    pub trace: Trace,
}

impl<'a> Trajectories<'a> {
    pub fn new(thermal: &'a Thermal, gas: Gas, q_floor: f64) -> Self {
        Self {
            thermal,
            gas,
            q_floor,
            nodes: Default::default(),
            trace: Trace::default(),
        }
    }

    pub fn at(&mut self, q_initial: f64, age_s: f64, kind: usize) -> Result<State, String> {
        if !age_s.is_finite() || age_s < 0.0 {
            return Err(error("invalid thermal trajectory age"));
        }
        let key = (q_initial.to_bits(), kind);
        if !self.nodes.contains_key(&key) {
            let (initial, _) = self
                .thermal
                .initial(q_initial, self.gas.pressure_pa, kind)?;
            self.trace.record(self.thermal, initial);
            self.nodes.insert(key, vec![initial]);
            self.trace.cached_trajectories += 1;
        }
        let th = self.thermal;
        let nodes = self.nodes.get_mut(&key).expect("inserted");
        let index = (age_s / th.dt_s).floor() as usize;
        while nodes.len() <= index {
            let previous = *nodes.last().expect("initial state");
            if previous.q_m2 <= self.q_floor {
                return Ok(previous);
            }
            let start = (nodes.len() - 1) as f64 * th.dt_s;
            let end = nodes.len() as f64 * th.dt_s;
            let next = th.advance(
                previous,
                start,
                end,
                self.gas,
                kind,
                self.q_floor,
                &mut self.trace,
            )?;
            nodes.push(next);
        }
        let state = nodes[index];
        if state.q_m2 <= self.q_floor {
            return Ok(state);
        }
        th.advance(
            state,
            index as f64 * th.dt_s,
            age_s,
            self.gas,
            kind,
            self.q_floor,
            &mut self.trace,
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Gas {
    pub pressure_pa: f64,
    pub x_co2: f64,
    pub x_he: f64,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct State {
    pub q_m2: f64,
    pub temperature_k: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct Flux {
    pub k_m2_s: f64,
    pub dt_k_s: f64,
    /// Gas heating numerator in the temperature balance [m2 K/s * J/(kg K)].
    pub heating: f64,
    pub latent_j_kg: f64,
    pub surface_fraction: f64,
}

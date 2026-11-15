//! Source-bound, internal property catalog. Pending evidence is inspectable, never evaluable.
//! No material-name branches, interpolation, fitting, or extrapolation are performed.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Dimensionless,
    JoulePerKelvin,
    PerMole,
    JouleSecond,
    MetrePerSecond,
    JoulePerMoleKelvin,
    WattPerSquareMetreKelvin4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quantity {
    BoltzmannConstant,
    AvogadroConstant,
    PlanckConstant,
    LightSpeed,
    CircleConstant,
    RelativeMolecularMass,
    IsobaricMolarHeatCapacity,
}

impl Quantity {
    fn unit(self) -> Unit {
        match self {
            Self::BoltzmannConstant => Unit::JoulePerKelvin,
            Self::AvogadroConstant => Unit::PerMole,
            Self::PlanckConstant => Unit::JouleSecond,
            Self::LightSpeed => Unit::MetrePerSecond,
            Self::CircleConstant | Self::RelativeMolecularMass => Unit::Dimensionless,
            Self::IsobaricMolarHeatCapacity => Unit::JoulePerMoleKelvin,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Gas,
    Liquid,
}

/// A point is deliberately not a range: a measured Cp is not a constant-property closure.
/// None pressure records missing source metadata, not pressure-independent applicability.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Domain {
    Universal,
    MolecularIdentity,
    Point {
        temperature_kelvin: f64,
        pressure_pascal: Option<f64>,
        phase: Phase,
        composition: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Uncertainty {
    ExactDefinition,
    Unquantified {
        reason: String,
    },
    /// Absolute numerical representation error, not experimental uncertainty.
    RepresentationBound {
        absolute_si: f64,
    },
    /// Source-reported standard uncertainty; no coverage factor inferred.
    Standard {
        absolute_si: f64,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    pub url: String,
    pub locator: String,
    /// Underlying determination/definition, not the web host or a mirror URL.
    pub origin: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub raw: String,
    pub raw_value: f64,
    pub raw_unit: Unit,
    pub value_si: f64,
    pub unit_si: Unit,
    pub source: Citation,
    pub uncertainty: Uncertainty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonKind {
    DefinitionalConcordance,
    MathematicalConcordance,
    IndependentDeterminations,
    IndependenceUnverified,
    CommonOrigin,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Comparison {
    pub observation: Observation,
    pub domain: Domain,
    pub reference: String,
    pub kind: ComparisonKind,
    pub rationale: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Admission {
    Definition,
    Mathematical,
    Independent,
    Pending { reason: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyRecord {
    pub id: String,
    pub material: Option<String>,
    pub quantity: Quantity,
    pub domain: Domain,
    pub reference: String,
    pub observation: Observation,
    pub comparison: Option<Comparison>,
    pub admission: Admission,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub id: String,
    pub formula: String,
    /// C,H,O,N,S,He,Ar atoms per neutral molecule. Identity only, not property prediction.
    pub atoms: [u16; 7],
    pub source: Citation,
    pub limitation: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    version: String,
    author: String,
    materials: Vec<Material>,
    records: Vec<PropertyRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvaluatedProperty {
    pub value_si: f64,
    pub unit: Unit,
    /// Includes property identities and underlying source identities. Reuse is shared dependence.
    pub dependencies: Vec<String>,
    pub qualification: String,
}

fn nonempty(s: &str) -> Result<()> {
    if s.trim().is_empty() {
        Err("empty identity or evidence field".into())
    } else {
        Ok(())
    }
}

fn positive(v: f64) -> Result<()> {
    if v.is_finite() && v > 0.0 {
        Ok(())
    } else {
        Err("expected finite positive value".into())
    }
}

impl Citation {
    fn validate(&self) -> Result<()> {
        nonempty(&self.origin)?;
        nonempty(&self.locator)?;
        if !self.url.starts_with("https://") {
            return Err("citation requires HTTPS URL".into());
        }
        Ok(())
    }
}

impl Domain {
    fn validate(&self) -> Result<()> {
        if let Self::Point {
            temperature_kelvin,
            pressure_pascal,
            composition,
            ..
        } = self
        {
            positive(*temperature_kelvin)?;
            if let Some(p) = pressure_pascal {
                positive(*p)?;
            }
            nonempty(composition)?;
        }
        Ok(())
    }
}

impl Observation {
    fn validate(&self, quantity: Quantity) -> Result<()> {
        nonempty(&self.raw)?;
        self.source.validate()?;
        positive(self.raw_value)?;
        positive(self.value_si)?;
        // This minimal catalog accepts only identity conversions, including J Hz^-1 = J s.
        // Adding a unit conversion requires an explicit, tested conversion law, not a multiplier.
        if self.raw_unit != quantity.unit()
            || self.unit_si != quantity.unit()
            || self.raw_value != self.value_si
        {
            return Err("property dimension or raw/SI identity conversion mismatch".into());
        }
        match &self.uncertainty {
            Uncertainty::Unquantified { reason } => nonempty(reason)?,
            Uncertainty::RepresentationBound { absolute_si }
            | Uncertainty::Standard { absolute_si } => positive(*absolute_si)?,
            Uncertainty::ExactDefinition => (),
        }
        Ok(())
    }
}

impl Catalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let catalog: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        catalog.validate()?;
        Ok(catalog)
    }

    /// The adjacent digest pins the complete manually transcribed evidence dataset.
    pub fn bundled() -> Result<Self> {
        let bytes =
            include_bytes!("../../../configs/models/properties/catalog-w0-astra/catalog.json");
        let expected =
            include_str!("../../../configs/models/properties/catalog-w0-astra/catalog.sha256")
                .trim();
        if hex::encode(Sha256::digest(bytes)) != expected {
            return Err("catalog digest mismatch".into());
        }
        Self::from_json(bytes)
    }

    pub fn records(&self) -> &[PropertyRecord] {
        &self.records
    }
    pub fn materials(&self) -> &[Material] {
        &self.materials
    }
    pub fn material(&self, id: &str) -> Result<&Material> {
        self.materials
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| "unknown material identity".into())
    }
    pub fn record(&self, id: &str) -> Result<&PropertyRecord> {
        self.records
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| "unknown property identity".into())
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != "property-catalog-w0-v1" || self.author != "gpt-6-astra" {
            return Err("unsupported catalog version/author provenance".into());
        }
        let mut materials = BTreeSet::new();
        for m in &self.materials {
            nonempty(&m.id)?;
            nonempty(&m.formula)?;
            nonempty(&m.limitation)?;
            m.source.validate()?;
            if !materials.insert(m.id.as_str()) || m.atoms.iter().all(|&n| n == 0) {
                return Err("duplicate or empty material composition".into());
            }
        }
        let mut records = BTreeSet::new();
        for r in &self.records {
            nonempty(&r.id)?;
            nonempty(&r.reference)?;
            if !records.insert(r.id.as_str()) {
                return Err("duplicate property identity".into());
            }
            r.domain.validate()?;
            r.observation.validate(r.quantity)?;
            match r.quantity {
                Quantity::RelativeMolecularMass => {
                    if r.domain != Domain::MolecularIdentity || r.material.is_none() {
                        return Err("molecular mass requires material identity domain".into());
                    }
                }
                Quantity::IsobaricMolarHeatCapacity => {
                    if !matches!(r.domain, Domain::Point { .. }) || r.material.is_none() {
                        return Err("Cp requires explicit point and material".into());
                    }
                }
                _ => {
                    if r.domain != Domain::Universal || r.material.is_some() {
                        return Err("universal constant has a material or state".into());
                    }
                }
            }
            if let Some(m) = &r.material {
                if !materials.contains(m.as_str()) {
                    return Err("unresolved material dependency".into());
                }
                if let Domain::Point { composition, .. } = &r.domain {
                    if composition != m {
                        return Err("pure-component identity mismatch".into());
                    }
                }
            }
            if let Some(c) = &r.comparison {
                c.observation.validate(r.quantity)?;
                c.domain.validate()?;
                nonempty(&c.reference)?;
                nonempty(&c.rationale)?;
            }
            if let Admission::Pending { reason } = &r.admission {
                nonempty(reason)?;
                continue;
            }
            let c = r
                .comparison
                .as_ref()
                .ok_or("admission requires a comparison")?;
            if c.domain != r.domain || c.reference != r.reference {
                return Err("comparison state or reference convention differs".into());
            }
            if let Domain::Point {
                pressure_pascal: None,
                ..
            } = r.domain
            {
                return Err("missing pressure blocks admission".into());
            }
            match r.admission {
                Admission::Definition => {
                    if !matches!(
                        r.quantity,
                        Quantity::BoltzmannConstant
                            | Quantity::AvogadroConstant
                            | Quantity::PlanckConstant
                            | Quantity::LightSpeed
                    ) || c.kind != ComparisonKind::DefinitionalConcordance
                        || r.observation.uncertainty != Uncertainty::ExactDefinition
                        || c.observation.uncertainty != Uncertainty::ExactDefinition
                        || c.observation.value_si != r.observation.value_si
                        || c.observation.source.url == r.observation.source.url
                    {
                        return Err("invalid SI definitional concordance".into());
                    }
                }
                Admission::Mathematical => {
                    if r.quantity != Quantity::CircleConstant
                        || c.kind != ComparisonKind::MathematicalConcordance
                        || c.observation.value_si != r.observation.value_si
                        || !matches!(
                            r.observation.uncertainty,
                            Uncertainty::RepresentationBound { .. }
                        )
                        || !matches!(
                            c.observation.uncertainty,
                            Uncertainty::RepresentationBound { .. }
                        )
                        || c.observation.source.url == r.observation.source.url
                    {
                        return Err("invalid mathematical concordance".into());
                    }
                }
                Admission::Independent => {
                    if r.material.is_none()
                        || c.kind != ComparisonKind::IndependentDeterminations
                        || c.observation.source.origin == r.observation.source.origin
                        || c.observation.source.url == r.observation.source.url
                    {
                        return Err(
                            "independent admission requires distinct underlying determinations"
                                .into(),
                        );
                    }
                }
                Admission::Pending { .. } => unreachable!(),
            }
        }
        Ok(())
    }

    /// `requested` must exactly match the qualified state. No nearby-state inference is made.
    pub fn evaluate(&self, id: &str, requested: &Domain) -> Result<EvaluatedProperty> {
        self.validate()?;
        requested.validate()?;
        let r = self.record(id)?;
        if let Admission::Pending { reason } = &r.admission {
            return Err(format!("non-admitted property {id}: {reason}"));
        }
        if &r.domain != requested {
            return Err("outside source state/phase/composition domain".into());
        }
        let c = r.comparison.as_ref().ok_or("comparison missing")?;
        let mut dependencies = vec![
            r.id.clone(),
            r.observation.source.origin.clone(),
            c.observation.source.origin.clone(),
        ];
        dependencies.sort();
        dependencies.dedup();
        Ok(EvaluatedProperty {
            value_si: r.observation.value_si,
            unit: r.quantity.unit(),
            dependencies,
            qualification: match r.admission {
                Admission::Definition => "SI definition; floating representation",
                Admission::Mathematical => "mathematical constant; floating approximation",
                _ => "independently compared point property",
            }
            .into(),
        })
    }

    fn constant(&self, quantity: Quantity) -> Result<EvaluatedProperty> {
        let mut found = self.records.iter().filter(|r| r.quantity == quantity);
        let r = found.next().ok_or("required universal constant absent")?;
        if found.next().is_some() {
            return Err("ambiguous universal constant".into());
        }
        self.evaluate(&r.id, &Domain::Universal)
    }

    /// Exact defining relation R = N_A k_B; the returned f64 is not exact decimal arithmetic.
    pub fn molar_gas_constant(&self) -> Result<EvaluatedProperty> {
        let k = self.constant(Quantity::BoltzmannConstant)?;
        let na = self.constant(Quantity::AvogadroConstant)?;
        derived(
            k.value_si * na.value_si,
            Unit::JoulePerMoleKelvin,
            &[k, na],
            "law:R=N_A*k_B;NIST-SP961-2024",
        )
    }

    /// Stefan law sigma = 2*pi^5*k_B^4/(15*h^3*c^2). Integers are algebraic, not fitted weights.
    pub fn stefan_boltzmann_constant(&self) -> Result<EvaluatedProperty> {
        let k = self.constant(Quantity::BoltzmannConstant)?;
        let h = self.constant(Quantity::PlanckConstant)?;
        let c = self.constant(Quantity::LightSpeed)?;
        let pi = self.constant(Quantity::CircleConstant)?;
        // Rearrangement avoids fourth powers of tiny dimensional constants.
        let value =
            (2.0 / 15.0) * pi.value_si.powi(5) * k.value_si * (k.value_si / h.value_si).powi(3)
                / c.value_si.powi(2);
        derived(
            value,
            Unit::WattPerSquareMetreKelvin4,
            &[k, h, c, pi],
            "law:sigma=2*pi^5*k_B^4/(15*h^3*c^2);NIST-SP961-2024",
        )
    }
}

fn derived(
    value_si: f64,
    unit: Unit,
    inputs: &[EvaluatedProperty],
    law: &str,
) -> Result<EvaluatedProperty> {
    positive(value_si)?;
    let mut dependencies = inputs
        .iter()
        .flat_map(|p| p.dependencies.clone())
        .collect::<Vec<_>>();
    dependencies.push(law.into());
    dependencies.sort();
    dependencies.dedup();
    Ok(EvaluatedProperty {
        value_si,
        unit,
        dependencies,
        qualification:
            "fixed defining relation evaluated in f64; not a measurement or fitted parameter".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_data_roundtrip_and_shared_constant_dependencies() {
        let c = Catalog::bundled().unwrap();
        assert_eq!(
            c,
            Catalog::from_json(&serde_json::to_vec(&c).unwrap()).unwrap()
        );
        let r = c.molar_gas_constant().unwrap();
        assert!((r.value_si - 8.31446261815324).abs() < 4e-15);
        let sigma = c.stefan_boltzmann_constant().unwrap();
        // CODATA prints a truncated exact decimal; this verifies that printed interval.
        assert!(sigma.value_si >= 5.670374419e-8 && sigma.value_si < 5.670374420e-8);
        assert!(r.dependencies.contains(&"si-2019-definitions".into()));
        assert_eq!(
            r.dependencies
                .iter()
                .filter(|s| *s == "si-2019-definitions")
                .count(),
            1
        );
        assert!(sigma.dependencies.contains(&"constant.k_B".into()));
    }

    #[test]
    fn pending_and_wrong_state_are_refused_without_name_fallbacks() {
        let c = Catalog::bundled().unwrap();
        for r in c.records() {
            if matches!(r.admission, Admission::Pending { .. }) {
                assert!(c.evaluate(&r.id, &r.domain).is_err());
            }
        }
        assert!(c
            .evaluate("constant.k_B", &Domain::MolecularIdentity)
            .is_err());
        assert!(c.evaluate("methane", &Domain::Universal).is_err());
        assert!(c.material("invented-formulation").is_err());
    }

    #[test]
    fn units_duplicates_nonfinite_and_false_corroboration_fail_closed() {
        let c = Catalog::bundled().unwrap();
        let mut bad = c.clone();
        bad.records[0].observation.unit_si = Unit::Dimensionless;
        assert!(bad.validate().is_err());
        let mut bad = c.clone();
        bad.records.push(bad.records[0].clone());
        assert!(bad.validate().is_err());
        let mut bad = c.clone();
        bad.records[0].observation.value_si = f64::NAN;
        assert!(bad.validate().is_err());
        let mut bad = c.clone();
        bad.records[0].comparison.as_mut().unwrap().kind = ComparisonKind::CommonOrigin;
        assert!(bad.validate().is_err());
        let mut bad = c.clone();
        bad.records[0]
            .comparison
            .as_mut()
            .unwrap()
            .observation
            .value_si *= 2.0;
        assert!(bad.validate().is_err());
        let mut raw = serde_json::to_value(c).unwrap();
        raw["hidden_fit"] = true.into();
        assert!(Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    }

    #[test]
    fn missing_pressure_and_reference_mismatch_block_promotion() {
        let mut c = Catalog::bundled().unwrap();
        let r = c
            .records
            .iter_mut()
            .find(|r| r.id == "methanol.cp_liquid.298_15")
            .unwrap();
        r.admission = Admission::Independent;
        r.comparison.as_mut().unwrap().kind = ComparisonKind::IndependentDeterminations;
        assert!(c.validate().is_err());
        let mut c = Catalog::bundled().unwrap();
        c.records[0].comparison.as_mut().unwrap().reference = "other convention".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn synthetic_admitted_point_obeys_phase_pressure_temperature_and_composition() {
        // Synthetic evidence for the admission/domain machinery, never a physical dataset.
        let mut c = Catalog::bundled().unwrap();
        let r = c
            .records
            .iter_mut()
            .find(|r| r.id == "methanol.cp_liquid.298_15")
            .unwrap();
        r.id = "synthetic.point".into();
        r.observation.raw = "7 synthetic J/mol/K".into();
        r.observation.raw_value = 7.0;
        r.observation.value_si = 7.0;
        r.observation.source.url = "https://example.org/synthetic-a".into();
        r.observation.source.origin = "synthetic-a".into();
        r.domain = Domain::Point {
            temperature_kelvin: 10.0,
            pressure_pascal: Some(20.0),
            phase: Phase::Liquid,
            composition: "methanol".into(),
        };
        let comparison = r.comparison.as_mut().unwrap();
        comparison.observation = r.observation.clone();
        comparison.observation.source.url = "https://example.org/synthetic-b".into();
        comparison.observation.source.origin = "synthetic-b".into();
        comparison.domain = r.domain.clone();
        comparison.kind = ComparisonKind::IndependentDeterminations;
        r.admission = Admission::Independent;
        let point = r.domain.clone();
        c.validate().unwrap();
        assert_eq!(c.evaluate("synthetic.point", &point).unwrap().value_si, 7.0);
        for wrong in [
            Domain::Point {
                temperature_kelvin: 11.0,
                pressure_pascal: Some(20.0),
                phase: Phase::Liquid,
                composition: "methanol".into(),
            },
            Domain::Point {
                temperature_kelvin: 10.0,
                pressure_pascal: Some(21.0),
                phase: Phase::Liquid,
                composition: "methanol".into(),
            },
            Domain::Point {
                temperature_kelvin: 10.0,
                pressure_pascal: Some(20.0),
                phase: Phase::Gas,
                composition: "methanol".into(),
            },
            Domain::Point {
                temperature_kelvin: 10.0,
                pressure_pascal: Some(20.0),
                phase: Phase::Liquid,
                composition: "water".into(),
            },
        ] {
            assert!(c.evaluate("synthetic.point", &wrong).is_err());
        }
    }
}

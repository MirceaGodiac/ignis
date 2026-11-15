//! Manually transcribed material property points with explicit thermodynamic paths.
//!
//! This internal reader has no interpolation, fitting, uncertainty pooling, or
//! material-name branches. Published smoothed determinations remain identified as
//! such. C_sat is never substituted for C_p, even where rounded values coincide.

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

const BUNDLED: &str =
    include_str!("../../../configs/models/properties/catalog-materials-astra/materials.json");
const BUNDLED_HASH: &str =
    include_str!("../../../configs/models/properties/catalog-materials-astra/materials.sha256");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialQuantity {
    SaturationPathMolarHeatCapacity,
    IsobaricMolarHeatCapacity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialUnit {
    JoulePerMoleKelvin,
    Kelvin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureScale {
    Its90,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ThermodynamicPath {
    /// Pure liquid on its liquid-vapor coexistence curve; pressure is p_sat(T).
    SaturatedLiquid,
    /// Zero-density ideal-gas limit, not a finite-pressure measurement.
    IdealGasLimit,
    LiquidAtPressure {
        pressure_pascal: f64,
    },
    LiquidPressureUnresolved,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialState {
    pub temperature_kelvin: f64,
    pub temperature_scale: TemperatureScale,
    pub path: ThermodynamicPath,
    /// Identity for joins only; the evaluator does not dispatch physics on it.
    pub composition: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiteralDatum {
    pub raw: String,
    pub raw_value: f64,
    pub raw_unit: MaterialUnit,
    pub value_si: f64,
    pub unit_si: MaterialUnit,
}

impl LiteralDatum {
    fn validate(&self, expected: MaterialUnit) -> Result<()> {
        ensure!(
            self.raw_value.is_finite() && self.value_si.is_finite(),
            "nonfinite literal"
        );
        ensure!(
            self.raw_value > 0.0 && self.value_si > 0.0,
            "nonpositive property or temperature"
        );
        ensure!(
            self.raw_unit == expected && self.unit_si == expected,
            "unit mismatch"
        );
        ensure!(
            self.raw_value == self.value_si,
            "this catalog only admits identity SI conversions"
        );
        let parsed: f64 = self
            .raw
            .parse()
            .context("raw decimal is not a literal number")?;
        ensure!(
            parsed == self.raw_value,
            "raw decimal differs from numeric transcription"
        );
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceRole {
    OriginalExperiment,
    Reanalysis,
    SecondaryTable,
    Documentation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialSource {
    pub id: String,
    pub title: String,
    pub url: String,
    pub role: SourceRole,
    pub sha256: Option<String>,
    pub read_extent: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDependency {
    pub id: String,
    pub description: String,
    pub source_ids: Vec<String>,
}

/// Historical experimental accuracy is not a standard deviation, and is not a
/// quantified uncertainty for the subsequently reprocessed table point.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoricalAccuracy {
    pub relative_percent: f64,
    pub source_id: String,
    pub locator: String,
    pub interpretation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialUncertainty {
    pub unquantified_reason: String,
    pub historical_accuracy: Option<HistoricalAccuracy>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Determination {
    pub experiment_id: String,
    pub tabulation_source_id: String,
    pub locator: String,
    pub original_source_ids: Vec<String>,
    pub dependency_ids: Vec<String>,
    pub processing: String,
    pub uncertainty: MaterialUncertainty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MaterialAdmission {
    /// Distinct underlying experiments, with shared dependencies explicitly kept.
    DistinctExperimentsSharedProcessing {
        rationale: String,
    },
    Pending {
        reason: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialPoint {
    pub temperature: LiteralDatum,
    pub primary: LiteralDatum,
    pub comparison: Option<LiteralDatum>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialSeries {
    pub id: String,
    pub material_id: String,
    pub quantity: MaterialQuantity,
    pub temperature_scale: TemperatureScale,
    pub path: ThermodynamicPath,
    pub reference: String,
    pub primary: Determination,
    pub comparison: Option<Determination>,
    pub shared_dependency_ids: Vec<String>,
    pub admission: MaterialAdmission,
    pub points: Vec<MaterialPoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageGap {
    pub material_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialCatalog {
    version: String,
    author: String,
    sources: Vec<MaterialSource>,
    dependencies: Vec<SourceDependency>,
    series: Vec<MaterialSeries>,
    coverage_gaps: Vec<CoverageGap>,
}

#[derive(Clone, Debug)]
pub struct MaterialEvaluation<'a> {
    pub series: &'a MaterialSeries,
    pub point: &'a MaterialPoint,
    /// The primary column is chosen prospectively by the literal catalog.
    pub primary_si: f64,
    pub comparison_si: f64,
    /// Comparison minus primary; diagnostic arithmetic, not an uncertainty bound.
    pub comparison_minus_primary_si: f64,
    pub source_ids: Vec<&'a str>,
    pub dependency_ids: Vec<&'a str>,
}

fn nonempty(value: &str) -> Result<()> {
    ensure!(!value.trim().is_empty(), "empty metadata");
    Ok(())
}

fn unique(values: impl IntoIterator<Item = impl AsRef<str>>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for value in values {
        nonempty(value.as_ref())?;
        ensure!(
            seen.insert(value.as_ref().to_owned()),
            "duplicate identifier"
        );
    }
    Ok(())
}

impl MaterialCatalog {
    pub fn from_json(json: &str) -> Result<Self> {
        let catalog: Self = serde_json::from_str(json)?;
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn bundled() -> Result<Self> {
        let actual = hex::encode(Sha256::digest(BUNDLED.as_bytes()));
        ensure!(
            actual == BUNDLED_HASH.trim(),
            "material catalog hash mismatch"
        );
        Self::from_json(BUNDLED)
    }

    pub fn series(&self) -> &[MaterialSeries] {
        &self.series
    }
    pub fn sources(&self) -> &[MaterialSource] {
        &self.sources
    }
    pub fn dependencies(&self) -> &[SourceDependency] {
        &self.dependencies
    }
    pub fn coverage_gaps(&self) -> &[CoverageGap] {
        &self.coverage_gaps
    }

    fn source(&self, id: &str) -> Result<&MaterialSource> {
        self.sources
            .iter()
            .find(|s| s.id == id)
            .context("unknown source dependency")
    }

    fn determination(&self, d: &Determination, admitted: bool) -> Result<()> {
        nonempty(&d.experiment_id)?;
        nonempty(&d.locator)?;
        nonempty(&d.processing)?;
        nonempty(&d.uncertainty.unquantified_reason)?;
        let tabulation = self.source(&d.tabulation_source_id)?;
        unique(&d.original_source_ids)?;
        unique(&d.dependency_ids)?;
        for id in &d.original_source_ids {
            let source = self.source(id)?;
            ensure!(
                source.role == SourceRole::OriginalExperiment,
                "original source is not experimental"
            );
            if admitted {
                ensure!(source.sha256.is_some(), "original bytes not pinned");
            }
        }
        for id in &d.dependency_ids {
            ensure!(
                self.dependencies.iter().any(|x| x.id == *id),
                "unknown processing dependency"
            );
        }
        if let Some(a) = &d.uncertainty.historical_accuracy {
            ensure!(
                a.relative_percent.is_finite() && a.relative_percent > 0.0,
                "invalid accuracy"
            );
            self.source(&a.source_id)?;
            nonempty(&a.locator)?;
            nonempty(&a.interpretation)?;
        }
        if admitted {
            ensure!(
                !d.original_source_ids.is_empty(),
                "no reviewed original experiment"
            );
            ensure!(tabulation.sha256.is_some(), "tabulation bytes not pinned");
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == "property-materials-w0-v1",
            "unsupported material catalog version"
        );
        ensure!(self.author == "gpt-6-astra", "unexpected author");
        ensure!(!self.series.is_empty(), "empty property catalog");
        unique(self.sources.iter().map(|x| &x.id))?;
        unique(self.dependencies.iter().map(|x| &x.id))?;
        unique(self.series.iter().map(|x| &x.id))?;
        unique(self.coverage_gaps.iter().map(|x| &x.material_id))?;
        for s in &self.sources {
            nonempty(&s.title)?;
            ensure!(s.url.starts_with("https://"), "source URL must be HTTPS");
            nonempty(&s.read_extent)?;
            if let Some(hash) = &s.sha256 {
                ensure!(
                    hash.len() == 64
                        && hash
                            .bytes()
                            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
                    "invalid SHA-256"
                );
            }
        }
        for d in &self.dependencies {
            nonempty(&d.description)?;
            ensure!(!d.source_ids.is_empty(), "unsourced dependency");
            unique(&d.source_ids)?;
            for id in &d.source_ids {
                self.source(id)?;
            }
        }
        for gap in &self.coverage_gaps {
            nonempty(&gap.reason)?;
        }
        for s in &self.series {
            nonempty(&s.material_id)?;
            nonempty(&s.reference)?;
            ensure!(!s.points.is_empty(), "empty series");
            let admitted = matches!(
                s.admission,
                MaterialAdmission::DistinctExperimentsSharedProcessing { .. }
            );
            match &s.admission {
                MaterialAdmission::Pending { reason } => nonempty(reason)?,
                MaterialAdmission::DistinctExperimentsSharedProcessing { rationale } => {
                    nonempty(rationale)?
                }
            }
            if let ThermodynamicPath::LiquidAtPressure { pressure_pascal } = s.path {
                ensure!(
                    pressure_pascal.is_finite() && pressure_pascal > 0.0,
                    "invalid liquid pressure"
                );
            }
            if s.quantity == MaterialQuantity::SaturationPathMolarHeatCapacity {
                ensure!(
                    s.path == ThermodynamicPath::SaturatedLiquid,
                    "C_sat requires the saturation path"
                );
            }
            if admitted {
                ensure!(
                    s.temperature_scale != TemperatureScale::Unresolved,
                    "unresolved temperature scale"
                );
                ensure!(
                    s.path != ThermodynamicPath::LiquidPressureUnresolved,
                    "unresolved pressure"
                );
            }
            self.determination(&s.primary, admitted)?;
            unique(&s.shared_dependency_ids)?;
            if let Some(c) = &s.comparison {
                self.determination(c, admitted)?;
                let common: BTreeSet<_> = s
                    .primary
                    .dependency_ids
                    .iter()
                    .filter(|id| c.dependency_ids.contains(id))
                    .collect();
                let declared: BTreeSet<_> = s.shared_dependency_ids.iter().collect();
                ensure!(
                    common == declared,
                    "shared dependencies not declared exactly"
                );
                if admitted {
                    ensure!(
                        s.primary.experiment_id != c.experiment_id,
                        "same experiment is not corroboration"
                    );
                    ensure!(
                        s.primary
                            .original_source_ids
                            .iter()
                            .all(|id| !c.original_source_ids.contains(id)),
                        "original experiments overlap"
                    );
                    if s.primary.tabulation_source_id == c.tabulation_source_id {
                        ensure!(
                            s.shared_dependency_ids.iter().any(|id| self
                                .dependencies
                                .iter()
                                .any(|d| d.id == *id
                                    && d.source_ids.contains(&c.tabulation_source_id))),
                            "shared tabulation omitted from dependencies"
                        );
                    }
                }
            } else {
                ensure!(!admitted, "admission needs a comparison");
                ensure!(
                    s.shared_dependency_ids.is_empty(),
                    "shared dependencies without a comparison"
                );
            }
            let mut previous = None;
            for p in &s.points {
                p.temperature.validate(MaterialUnit::Kelvin)?;
                p.primary.validate(MaterialUnit::JoulePerMoleKelvin)?;
                if let Some(t) = previous {
                    ensure!(
                        p.temperature.value_si > t,
                        "temperatures must be unique and increasing"
                    );
                }
                previous = Some(p.temperature.value_si);
                ensure!(
                    p.comparison.is_some() == s.comparison.is_some(),
                    "comparison column incomplete"
                );
                if let Some(c) = &p.comparison {
                    c.validate(MaterialUnit::JoulePerMoleKelvin)?;
                }
            }
        }
        Ok(())
    }

    pub fn evaluate(
        &self,
        series_id: &str,
        state: &MaterialState,
        quantity: MaterialQuantity,
    ) -> Result<MaterialEvaluation<'_>> {
        self.validate()?;
        ensure!(
            state.temperature_kelvin.is_finite() && state.temperature_kelvin > 0.0,
            "invalid requested temperature"
        );
        let s = self
            .series
            .iter()
            .find(|s| s.id == series_id)
            .context("unknown material series")?;
        if let MaterialAdmission::Pending { reason } = &s.admission {
            bail!("pending property: {reason}");
        }
        ensure!(s.quantity == quantity, "heat-capacity quantity mismatch");
        ensure!(
            s.material_id == state.composition
                && s.path == state.path
                && s.temperature_scale == state.temperature_scale,
            "thermodynamic state or composition mismatch"
        );
        let p = s.points.iter().find(|p| p.temperature.value_si == state.temperature_kelvin).context("temperature is not a tabulated point; interpolation and extrapolation are not admitted")?;
        let comparison = s
            .comparison
            .as_ref()
            .context("missing admitted comparison")?;
        let comparison_value = p
            .comparison
            .as_ref()
            .context("missing comparison value")?
            .value_si;
        let mut sources = BTreeSet::new();
        let mut dependencies = BTreeSet::new();
        for d in [&s.primary, comparison] {
            sources.insert(d.tabulation_source_id.as_str());
            for id in &d.original_source_ids {
                sources.insert(id.as_str());
            }
            if let Some(a) = &d.uncertainty.historical_accuracy {
                sources.insert(a.source_id.as_str());
            }
            for id in &d.dependency_ids {
                dependencies.insert(id.as_str());
                let dep = self
                    .dependencies
                    .iter()
                    .find(|dep| dep.id == *id)
                    .context("missing dependency")?;
                for source in &dep.source_ids {
                    sources.insert(source.as_str());
                }
            }
        }
        Ok(MaterialEvaluation {
            series: s,
            point: p,
            primary_si: p.primary.value_si,
            comparison_si: comparison_value,
            comparison_minus_primary_si: comparison_value - p.primary.value_si,
            source_ids: sources.into_iter().collect(),
            dependency_ids: dependencies.into_iter().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> MaterialState {
        MaterialState {
            temperature_kelvin: 298.15,
            temperature_scale: TemperatureScale::Its90,
            path: ThermodynamicPath::SaturatedLiquid,
            composition: "n_heptane".into(),
        }
    }

    #[test]
    fn bundled_roundtrip_retains_comparison_and_dependency_identity() -> Result<()> {
        let c = MaterialCatalog::bundled()?;
        let roundtrip = MaterialCatalog::from_json(&serde_json::to_string(&c)?)?;
        let v = roundtrip.evaluate(
            "n_heptane.csat.its90",
            &state(),
            MaterialQuantity::SaturationPathMolarHeatCapacity,
        )?;
        assert_eq!(v.primary_si, 224.88);
        assert_eq!(v.comparison_si, 225.08);
        assert_eq!(
            v.comparison_minus_primary_si,
            v.comparison_si - v.primary_si
        );
        assert!(v.source_ids.contains(&"douglas_1954"));
        assert!(v.source_ids.contains(&"mccullough_messerly_1961"));
        assert!(v.dependency_ids.contains(&"shared_its90_reprocessing"));
        assert_eq!(v.series.points.len(), 19);
        Ok(())
    }

    #[test]
    fn no_path_quantity_temperature_scale_or_composition_substitution() -> Result<()> {
        let c = MaterialCatalog::bundled()?;
        let q = MaterialQuantity::SaturationPathMolarHeatCapacity;
        assert!(c
            .evaluate(
                "n_heptane.csat.its90",
                &state(),
                MaterialQuantity::IsobaricMolarHeatCapacity
            )
            .is_err());
        for bad in [
            MaterialState {
                temperature_kelvin: 298.16,
                ..state()
            },
            MaterialState {
                temperature_kelvin: 180.0,
                ..state()
            },
            MaterialState {
                temperature_scale: TemperatureScale::Unresolved,
                ..state()
            },
            MaterialState {
                path: ThermodynamicPath::IdealGasLimit,
                ..state()
            },
            MaterialState {
                path: ThermodynamicPath::LiquidAtPressure {
                    pressure_pascal: 100000.0,
                },
                ..state()
            },
            MaterialState {
                composition: "methanol".into(),
                ..state()
            },
            MaterialState {
                temperature_kelvin: f64::NAN,
                ..state()
            },
        ] {
            assert!(c.evaluate("n_heptane.csat.its90", &bad, q).is_err());
        }
        Ok(())
    }

    #[test]
    fn aliases_are_joins_not_physical_branches_and_pending_gas_is_not_pressure_data() -> Result<()>
    {
        let mut c = MaterialCatalog::bundled()?;
        c.series[0].material_id = "synthetic_identity_alias".into();
        c.validate()?;
        let alias = MaterialState {
            composition: "synthetic_identity_alias".into(),
            ..state()
        };
        assert_eq!(
            c.evaluate(
                "n_heptane.csat.its90",
                &alias,
                MaterialQuantity::SaturationPathMolarHeatCapacity
            )?
            .primary_si,
            224.88
        );
        let gas = MaterialState {
            composition: "methane".into(),
            path: ThermodynamicPath::IdealGasLimit,
            temperature_scale: TemperatureScale::Unresolved,
            ..state()
        };
        assert!(c
            .evaluate(
                "methane.cp0.298_15.pending",
                &gas,
                MaterialQuantity::IsobaricMolarHeatCapacity
            )
            .is_err());
        Ok(())
    }

    #[test]
    fn rejects_false_corroboration_lost_dependencies_and_unpinned_originals() -> Result<()> {
        let base = MaterialCatalog::bundled()?;
        let mut c = base.clone();
        c.series[0].comparison.as_mut().unwrap().experiment_id =
            c.series[0].primary.experiment_id.clone();
        assert!(c.validate().is_err());
        let mut c = base.clone();
        c.series[0].shared_dependency_ids.clear();
        assert!(c.validate().is_err());
        let mut c = base.clone();
        c.series[0].comparison.as_mut().unwrap().original_source_ids =
            c.series[0].primary.original_source_ids.clone();
        assert!(c.validate().is_err());
        let mut c = base;
        c.sources
            .iter_mut()
            .find(|s| s.id == "douglas_1954")
            .unwrap()
            .sha256 = None;
        assert!(c.validate().is_err());
        Ok(())
    }

    #[test]
    fn rejects_unit_nonfinite_transcription_duplicate_and_schema_corruption() -> Result<()> {
        let base = MaterialCatalog::bundled()?;
        let mut c = base.clone();
        c.series[0].points[0].primary.unit_si = MaterialUnit::Kelvin;
        assert!(c.validate().is_err());
        let mut c = base.clone();
        c.series[0].points[0].primary.value_si = f64::INFINITY;
        assert!(c.validate().is_err());
        let mut c = base.clone();
        c.series[0].points[0].primary.raw = "0".into();
        assert!(c.validate().is_err());
        let mut c = base.clone();
        let duplicate = c.series[0].points[0].clone();
        c.series[0].points.insert(0, duplicate);
        assert!(c.validate().is_err());
        let mut json = serde_json::to_value(&base)?;
        json["unrecognized"] = true.into();
        assert!(MaterialCatalog::from_json(&serde_json::to_string(&json)?).is_err());
        Ok(())
    }
}

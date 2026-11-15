//! Astra jets v2: synthetic-physics-review-informed admission correction.
//!
//! The immutable W0 numerical law and literal values are reused. This module
//! closes a property-domain hole before any sensible-enthalpy evaluation.
//! See docs/wsp/research/astra_jets_v2.md; this is not a qualified release.

use crate::physical_core::{Phase, PhysicalError, Positive, Result, ThermodynamicState};
use crate::property_jets_w0 as predecessor;

pub use predecessor::{
    CaloricState, Coefficients, Composition, JetForecast, JetInput, MarchOptions, Record, Station,
    Stream, Unit,
};

/// Constant caloric properties are admitted on closed rectangular T/P domains.
/// W0 checks inlet and evaluated output states. Checking the common reference
/// and the reference at operating pressure additionally admits every point on
/// each reference-to-state path; no extrapolation or new property value occurs.
/// Molar mass is included because a molar cp is converted to mass-specific cp.
fn require_enthalpy_reference_paths(input: &JetInput) -> Result<()> {
    let reference = input
        .jet
        .composition
        .entries
        .first()
        .ok_or(PhysicalError("empty jet composition"))?
        .0
        .heat_capacity()
        .reference()
        .state;
    let operating_pressure = Positive::new(input.jet.pressure.si(Unit::Pascal)?)?;
    let operating_reference = ThermodynamicState {
        temperature_kelvin: reference.temperature_kelvin,
        pressure_pascal: operating_pressure,
    };
    for composition in [
        &input.jet.composition,
        &input.coflow.composition,
        &input.stoichiometric_products,
    ] {
        for (component, _) in &composition.entries {
            for property in [component.heat_capacity(), component.molar_mass()] {
                if !property.domain().contains(reference, Phase::Gas)
                    || !property.domain().contains(operating_reference, Phase::Gas)
                {
                    return Err(PhysicalError(
                        "enthalpy reference path exceeds a supplied gas cp or molar-mass domain",
                    ));
                }
            }
        }
    }
    Ok(())
}

pub fn caloric_state(input: &JetInput, z: f64, options: MarchOptions) -> Result<CaloricState> {
    require_enthalpy_reference_paths(input)?;
    predecessor::caloric_state(input, z, options)
}

pub fn analytic_centerline_reference(
    input: &JetInput,
    axial_distance_m: f64,
    coefficients: Coefficients,
    options: MarchOptions,
) -> Result<(f64, f64)> {
    require_enthalpy_reference_paths(input)?;
    predecessor::analytic_centerline_reference(input, axial_distance_m, coefficients, options)
}

pub fn forward(
    input: &JetInput,
    axial_distance_m: f64,
    coefficients: Coefficients,
    options: MarchOptions,
) -> Result<JetForecast> {
    require_enthalpy_reference_paths(input)?;
    predecessor::forward(input, axial_distance_m, coefficients, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_core::{
        ElementalComposition, EnthalpyConvention, Finite, FractionBasis, GasComponent,
        PropertyDomain, PropertyOrigin, PropertyProvenance, PropertyUncertainty, PropertyUnit,
        ReferenceState, SuppliedProperty,
    };

    // Synthetic material values, independent of NASA outcomes or real properties.
    fn positive(v: f64) -> Positive {
        Positive::new(v).unwrap()
    }
    fn state(t: f64, pressure: f64) -> ThermodynamicState {
        ThermodynamicState {
            temperature_kelvin: positive(t),
            pressure_pascal: positive(pressure),
        }
    }
    fn reference() -> ReferenceState {
        ReferenceState {
            state: state(300.0, 20.0),
            convention: EnthalpyConvention::SensibleOnly {
                reference_identity: "synthetic-v2-reference".into(),
            },
        }
    }
    fn domain(min_t: f64, min_p: f64) -> PropertyDomain {
        PropertyDomain {
            phase: Phase::Gas,
            minimum: state(min_t, min_p),
            maximum: state(2000.0, 100.0),
        }
    }
    fn provenance() -> PropertyProvenance {
        PropertyProvenance {
            origin: PropertyOrigin::SyntheticFixture,
            source: "synthetic_fixture".into(),
            locator: "property_jets_v2::tests".into(),
            dependencies: vec!["synthetic-CO-oxidation".into()],
            uncertainty: PropertyUncertainty::Unquantified,
        }
    }
    fn record(v: f64, unit: Unit) -> Record {
        Record {
            raw_value: Finite::new(v).unwrap(),
            raw_unit: unit,
            provenance: provenance(),
            domain: Some(domain(200.0, 1.0)),
            reference_identity: "synthetic-v2-reference".into(),
        }
    }
    fn component(
        id: &str,
        carbon: u32,
        oxygen: u32,
        mass: f64,
        cp_min_t: f64,
        mass_min_t: f64,
        cp_min_p: f64,
    ) -> GasComponent {
        GasComponent::new(
            id.into(),
            ElementalComposition {
                carbon,
                hydrogen: 0,
                oxygen,
                nitrogen: 0,
                sulfur: 0,
                helium: 0,
                argon: 0,
            },
            SuppliedProperty::new(
                positive(mass),
                PropertyUnit::KilogramPerMole,
                domain(mass_min_t, 1.0),
                reference(),
                provenance(),
            )
            .unwrap(),
            SuppliedProperty::new(
                positive(1000.0),
                PropertyUnit::JoulePerKilogramKelvin,
                domain(cp_min_t, cp_min_p),
                reference(),
                provenance(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn composition(c: GasComponent) -> Composition {
        Composition {
            entries: vec![(c, record(1.0, Unit::One))],
            basis: FractionBasis::Mass,
        }
    }
    fn fixture() -> JetInput {
        let stream = |composition| Stream {
            composition,
            temperature: record(320.0, Unit::Kelvin),
            pressure: record(22.0, Unit::Pascal),
            axial_speed: record(1.0, Unit::MetrePerSecond),
        };
        JetInput {
            jet: stream(composition(component(
                "synthetic-CO",
                1,
                1,
                2.0,
                200.0,
                200.0,
                1.0,
            ))),
            coflow: stream(composition(component(
                "synthetic-O2",
                0,
                2,
                2.0,
                200.0,
                200.0,
                1.0,
            ))),
            stoichiometric_products: composition(component(
                "synthetic-CO2",
                1,
                2,
                3.0,
                200.0,
                200.0,
                1.0,
            )),
            heat_release_st: record(150000.0, Unit::JoulePerKilogram),
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
            coordinate_frame: "synthetic-v2-laboratory".into(),
            maximum_boussinesq_density_change: record(0.1, Unit::One),
            steady_laminar_parabolic_assumption: "synthetic laminar boundary-layer scenario".into(),
            common_diffusivity_unity_lewis_assumption: "synthetic common transport scenario".into(),
            fast_chemistry_adiabatic_assumption: "synthetic complete-reaction scenario".into(),
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

    #[test]
    fn reference_temperature_gap_is_reproduced_in_w0_and_refused_by_every_v2_entry() {
        let mut input = fixture();
        input.jet.composition =
            composition(component("synthetic-CO", 1, 1, 2.0, 310.0, 200.0, 1.0));
        let predecessor = predecessor::caloric_state(&input, 1.0, options()).unwrap();
        assert_eq!(predecessor.temperature_kelvin, 320.0);
        assert!(caloric_state(&input, 1.0, options())
            .unwrap_err()
            .0
            .contains("enthalpy reference path"));
        assert!(forward(&input, 0.1, coefficients(), options()).is_err());
        assert!(analytic_centerline_reference(&input, 0.1, coefficients(), options()).is_err());
    }

    #[test]
    fn reference_paths_cover_both_streams_products_mass_conversion_and_pressure() {
        let mut input = fixture();
        input.coflow.composition =
            composition(component("synthetic-O2", 0, 2, 2.0, 310.0, 200.0, 1.0));
        assert!(predecessor::caloric_state(&input, 1.0, options()).is_ok());
        assert!(caloric_state(&input, 1.0, options()).is_err());
        input = fixture();
        input.jet.composition =
            composition(component("synthetic-CO", 1, 1, 2.0, 200.0, 310.0, 1.0));
        assert!(predecessor::caloric_state(&input, 1.0, options()).is_ok());
        assert!(caloric_state(&input, 1.0, options()).is_err());
        input = fixture();
        input.jet.composition =
            composition(component("synthetic-CO", 1, 1, 2.0, 200.0, 200.0, 21.0));
        assert!(predecessor::caloric_state(&input, 1.0, options()).is_ok());
        assert!(caloric_state(&input, 1.0, options()).is_err());
        input = fixture();
        input.stoichiometric_products =
            composition(component("synthetic-CO2", 1, 2, 3.0, 310.0, 200.0, 1.0));
        assert!(caloric_state(&input, 1.0, options()).is_err());
        input = fixture();
        input.jet.composition =
            composition(component("synthetic-CO", 1, 1, 2.0, 300.0, 300.0, 20.0));
        assert!(
            caloric_state(&input, 1.0, options()).is_ok(),
            "closed reference-domain boundary must be admitted"
        );
    }

    #[test]
    fn valid_predecessor_caloric_and_analytic_results_are_exactly_unchanged() {
        let input = fixture();
        for z in [0.0, 0.25, 0.5, 2.0 / 3.0, 0.875, 1.0] {
            let old = predecessor::caloric_state(&input, z, options()).unwrap();
            let new = caloric_state(&input, z, options()).unwrap();
            assert_eq!(format!("{old:?}"), format!("{new:?}"));
        }
        assert_eq!(
            predecessor::analytic_centerline_reference(&input, 0.5, coefficients(), options())
                .unwrap(),
            analytic_centerline_reference(&input, 0.5, coefficients(), options()).unwrap(),
        );
    }

    #[test]
    fn valid_predecessor_coupled_normal_inverse_and_gravity_fields_are_exactly_unchanged() {
        let compare = |input: &JetInput| {
            let old = predecessor::forward(input, 0.5, coefficients(), options()).unwrap();
            let new = forward(input, 0.5, coefficients(), options()).unwrap();
            assert_eq!(format!("{old:?}"), format!("{new:?}"));
        };
        let mut input = fixture();
        compare(&input);
        input.jet.axial_speed = record(2.0, Unit::MetrePerSecond);
        compare(&input);
        input.gravity[2] = record(-1.0, Unit::MetrePerSecondSquared);
        compare(&input);
        std::mem::swap(&mut input.jet, &mut input.coflow);
        compare(&input);
    }

    #[test]
    fn v2_contract_packet_materializes_identical_literal_values_with_review_exposure() {
        use crate::assemble::{make_patch, materialize, PatchMeta};
        use ignis_core::contracts::{LiteralOp, ModelSpec};
        let spec: ModelSpec = serde_json::from_str(include_str!(
            "../../../configs/models/jets/property-v2/law-review-astra/model_spec.json"
        ))
        .unwrap();
        let meta: PatchMeta = serde_json::from_str(include_str!(
            "../../../configs/models/jets/property-v2/law-review-astra/patch_meta.json"
        ))
        .unwrap();
        let old_ops = include_str!(
            "../../../configs/models/jets/property-w0/cleanroom-astra/literal_ops.json"
        );
        let new_ops = include_str!(
            "../../../configs/models/jets/property-v2/law-review-astra/literal_ops.json"
        );
        assert_eq!(old_ops, new_ops);
        assert!(meta.rationale.as_ref().unwrap().contains("data-informed"));
        let ops: Vec<LiteralOp> = serde_json::from_str(new_ops).unwrap();
        spec.validate().unwrap();
        let patch = make_patch(&spec, meta, ops);
        let tensors = materialize(&spec, &patch, None).unwrap();
        Coefficients::from_literal_values(&tensors["analytic_constants"].data).unwrap();
    }
}

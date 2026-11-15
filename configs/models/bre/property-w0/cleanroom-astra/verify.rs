#![allow(dead_code)]

#[path = "../../../../../crates/ignis-wsp/src/assemble.rs"]
mod assemble;
#[path = "../../../../../crates/ignis-wsp/src/physical_core.rs"]
mod physical_core;
#[path = "../../../../../crates/ignis-wsp/src/property_bre_w0.rs"]
mod property_bre_w0;

#[cfg(test)]
mod bre_checks {
    use super::{assemble, physical_core::*, property_bre_w0::*};
    use ignis_core::contracts::{LiteralOp, ModelSpec};

    fn provenance() -> PropertyProvenance {
        PropertyProvenance {
            origin: PropertyOrigin::SyntheticFixture,
            source: "synthetic_fixture:bre-conservation".into(),
            locator: "verify.rs".into(),
            dependencies: vec![],
            uncertainty: PropertyUncertainty::Unquantified,
        }
    }
    fn domain() -> Domain {
        Domain {
            temperature_kelvin: [1.0, 2000.0],
            pressure_pascal: [1.0, 200000.0],
            oxygen_mass_fraction: [0.0, 1.0],
            speed_m_per_s: [0.0, 100.0],
            gravity_m_per_s2: [0.0, 20.0],
        }
    }
    fn p(value: f64, unit: Unit) -> Property {
        Property {
            value,
            unit,
            domain: domain(),
            reference: "synthetic-sensible-reference".into(),
            provenance: provenance(),
        }
    }
    fn vector(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3::new(
            [
                Finite::new(x).unwrap(),
                Finite::new(y).unwrap(),
                Finite::new(z).unwrap(),
            ],
            "synthetic-lab".into(),
        )
        .unwrap()
    }
    fn c() -> Coefficients {
        let ops: Vec<LiteralOp> = serde_json::from_str(include_str!("literal_ops.json")).unwrap();
        let LiteralOp::Set { value, .. } = ops[0] else {
            panic!("scalar literal required")
        };
        Coefficients {
            stefan_boltzmann_watt_per_m2_kelvin4: value.get(),
        }
    }
    fn a() -> Apparatus {
        Apparatus {
            thermal_capacity: p(100.0, Unit::JoulePerKelvin),
            exposed_area: p(1.0, Unit::SquareMetre),
            linear_loss_conductance: p(0.0, Unit::WattPerKelvin),
            emissivity: p(0.0, Unit::Dimensionless),
            heat_of_combustion: p(1000.0, Unit::JoulePerKilogram),
            absorbed_heat_fraction: p(0.5, Unit::Dimensionless),
            conversion_fraction: p(1.0, Unit::Dimensionless),
            oxygen_required_per_fuel: p(2.0, Unit::Dimensionless),
            actuator_lag: p(1.0, Unit::Second),
            actuator_maximum_flow: p(2.0, Unit::KilogramPerSecond),
            thermal_limit: None,
            quasi_steady_combustion_assertion:
                "synthetic instantaneous well-mixed conversion assumption".into(),
        }
    }
    fn m() -> VirtualMaterial {
        VirtualMaterial {
            exposed_area: p(1.0, Unit::SquareMetre),
            heat_capacity: p(10.0, Unit::JoulePerKilogramKelvin),
            phase_change_enthalpy: p(100.0, Unit::JoulePerKilogram),
            phase_temperature: p(310.0, Unit::Kelvin),
            sensible_reference_temperature: p(300.0, Unit::Kelvin),
            linear_loss_conductance: p(0.0, Unit::WattPerKelvin),
            emissivity: p(0.0, Unit::Dimensionless),
            equilibrium_phase_change_assertion: "synthetic equilibrium phase-change body".into(),
        }
    }
    fn s() -> State {
        State {
            seconds: 0.0,
            time_origin: "synthetic power-on".into(),
            virtual_material: None,
            apparatus_temperature_kelvin: 300.0,
            actual_fuel_flow_kg_per_s: 0.0,
            provenance: provenance(),
        }
    }
    fn f(command: f64) -> Forcing {
        Forcing {
            environment: Environment {
                ambient_temperature_kelvin: 300.0,
                pressure_pascal: 100000.0,
                oxygen_mass_fraction: 0.5,
                oxidizer_mass_flow_kg_per_s: 10.0,
                velocity_m_per_s: vector(0.0, 0.0, 0.0),
                gravity_m_per_s2: vector(0.0, 0.0, 0.0),
                provenance: provenance(),
            },
            command: Command::OpenLoop {
                fuel_flow: p(command, Unit::KilogramPerSecond),
            },
            virtual_absorbed_heat_flux: None,
            apparatus_external_power: p(0.0, Unit::Watt),
            provenance: provenance(),
        }
    }
    fn n(dt: f64) -> NumericalControls {
        NumericalControls {
            step_seconds: dt,
            maximum_step_seconds: 10.0,
            energy_absolute_tolerance_joule: 1e-9,
            mass_absolute_tolerance_kg: 1e-12,
            relative_tolerance: 1e-12,
        }
    }
    fn close(a: f64, b: f64) {
        assert!(
            (a - b).abs() <= 1e-9 * (1.0 + a.abs() + b.abs()),
            "{a} != {b}"
        );
    }

    #[test]
    fn actual_contract_spec_and_patch_validate_without_release_assembly() {
        let spec: ModelSpec = serde_json::from_str(include_str!("model_spec.json")).unwrap();
        spec.validate().unwrap();
        let meta: assemble::PatchMeta =
            serde_json::from_str(include_str!("patch_meta.json")).unwrap();
        let ops: Vec<LiteralOp> = serde_json::from_str(include_str!("literal_ops.json")).unwrap();
        let patch = assemble::make_patch(&spec, meta, ops);
        patch.validate_against(&spec).unwrap();
        assert_eq!(patch.author.model_id, "gpt-6-astra");
        assert!(patch.exposure.datasets.is_empty());
        assert!(patch.exposure.development_feedback.is_empty());
        assert!(patch.exposure.prior_candidates.is_empty());
        assert_eq!(
            spec.target.quantity,
            ignis_core::contracts::Quantity::Temperature
        );
        println!("BRE_SPEC_HASH={}", spec.spec_hash());
        println!("BRE_PATCH_HASH={}", patch.patch_hash());
    }

    #[test]
    fn open_loop_lag_and_integrated_response_are_not_the_command() {
        let out = advance(c(), None, &a(), &s(), &f(1.0), n(1.0)).unwrap();
        close(out.requested_command_kg_per_s, 1.0);
        close(out.actual_fuel_flow_end_kg_per_s, 1.0 - (-1.0_f64).exp());
        close(
            out.apparatus_ledger.delivered_fuel_mass_kg,
            (-1.0_f64).exp(),
        );
        close(
            out.predicted_apparatus_temperature_kelvin,
            300.0 + 5.0 * (-1.0_f64).exp(),
        );
        assert_eq!(out.safety.status, "UNKNOWN");
        assert!(out.virtual_ledger.is_none());
        close(out.apparatus_ledger.mass_residual_kg, 0.0);
        close(out.apparatus_ledger.energy_residual_joule, 0.0);
    }

    #[test]
    fn no_command_no_heat_and_no_fictitious_feedback() {
        let mut warm = s();
        warm.apparatus_temperature_kelvin = 500.0;
        let out = advance(c(), None, &a(), &warm, &f(0.0), n(1.0)).unwrap();
        close(out.actual_fuel_flow_end_kg_per_s, 0.0);
        close(out.predicted_apparatus_temperature_kelvin, 500.0);
        let cool = advance(c(), None, &a(), &s(), &f(1.0), n(1.0)).unwrap();
        let hot = advance(c(), None, &a(), &warm, &f(1.0), n(1.0)).unwrap();
        close(
            cool.apparatus_ledger.delivered_fuel_mass_kg,
            hot.apparatus_ledger.delivered_fuel_mass_kg,
        );
    }

    #[test]
    fn virtual_warming_phase_change_and_real_energy_are_separate() {
        let mut initial = s();
        initial.virtual_material = Some(VirtualState {
            remaining_mass_kg: 2.0,
            temperature_kelvin: 300.0,
        });
        let mut input = f(0.0);
        input.virtual_absorbed_heat_flux = Some(p(300.0, Unit::WattPerSquareMetre));
        input.command = Command::VirtualMaterialDemand;
        let out = advance(c(), Some(&m()), &a(), &initial, &input, n(1.0)).unwrap();
        let v = out.virtual_ledger.as_ref().unwrap();
        close(v.gasified_mass_kg, 1.0);
        close(v.energy_residual_joule, 0.0);
        close(v.sensible_storage_change_joule, 100.0);
        close(v.departing_sensible_enthalpy_joule, 100.0);
        close(v.phase_change_energy_joule, 100.0);
        close(
            out.state
                .virtual_material
                .as_ref()
                .unwrap()
                .temperature_kelvin,
            310.0,
        );
        close(out.requested_command_kg_per_s, 1.0);
        close(out.apparatus_ledger.external_heat_joule, 0.0);
        close(
            out.apparatus_ledger.storage_change_joule,
            out.apparatus_ledger.absorbed_chemical_heat_joule,
        );
        input.command = Command::OpenLoop {
            fuel_flow: p(0.0, Unit::KilogramPerSecond),
        };
        let no_real_feed = advance(c(), Some(&m()), &a(), &initial, &input, n(1.0)).unwrap();
        close(no_real_feed.predicted_apparatus_temperature_kelvin, 300.0);
        close(no_real_feed.virtual_ledger.unwrap().gasified_mass_kg, 1.0);
    }

    #[test]
    fn virtual_phase_onset_does_not_start_feed_retroactively() {
        let mut initial = s();
        initial.virtual_material = Some(VirtualState {
            remaining_mass_kg: 2.0,
            temperature_kelvin: 300.0,
        });
        let mut input = f(0.0);
        input.command = Command::VirtualMaterialDemand;
        input.virtual_absorbed_heat_flux = Some(p(300.0, Unit::WattPerSquareMetre));
        let all = advance(c(), Some(&m()), &a(), &initial, &input, n(1.0)).unwrap();
        close(all.command_change_after_seconds.unwrap(), 2.0 / 3.0);
        close(all.requested_command_kg_per_s, 1.0);
        close(all.requested_command_end_kg_per_s, 3.0);
        close(all.saturated_command_kg_per_s, 2.0);
        close(
            all.actual_fuel_flow_end_kg_per_s,
            2.0 * (1.0 - (-1.0_f64 / 3.0).exp()),
        );
        let before = advance(c(), Some(&m()), &a(), &initial, &input, n(0.5)).unwrap();
        close(before.apparatus_ledger.delivered_fuel_mass_kg, 0.0);
        let after = advance(c(), Some(&m()), &a(), &before.state, &input, n(0.5)).unwrap();
        close(
            all.apparatus_ledger.delivered_fuel_mass_kg,
            after.apparatus_ledger.delivered_fuel_mass_kg,
        );
        close(
            all.actual_fuel_flow_end_kg_per_s,
            after.actual_fuel_flow_end_kg_per_s,
        );
        close(
            all.predicted_apparatus_temperature_kelvin,
            after.predicted_apparatus_temperature_kelvin,
        );
    }

    #[test]
    fn oxygen_crossings_and_incomplete_conversion_preserve_mass() {
        for (start, command) in [(0.0, 1.0), (1.0, 0.0)] {
            let mut initial = s();
            initial.actual_fuel_flow_kg_per_s = start;
            let mut input = f(command);
            input.environment.oxidizer_mass_flow_kg_per_s = 2.0;
            let mut apparatus = a();
            apparatus.conversion_fraction.value = 0.5;
            let out = advance(c(), None, &apparatus, &initial, &input, n(1.0)).unwrap();
            let l = out.apparatus_ledger;
            let crossing = 2.0_f64.ln();
            let accessible = if start == 0.0 {
                crossing - 0.5 + 0.5 * (1.0 - crossing)
            } else {
                0.5 * crossing + 0.5 - (-1.0_f64).exp()
            };
            close(l.reacted_fuel_mass_kg, 0.5 * accessible);
            close(l.mass_residual_kg, 0.0);
            assert!(l.unburned_fuel_mass_kg > 0.0);
            assert!(l.oxygen_exhaust_mass_kg >= 0.0);
        }
    }

    #[test]
    fn zero_oxygen_and_zero_conversion_are_not_burning() {
        let mut initial = s();
        initial.actual_fuel_flow_kg_per_s = 1.0;
        let mut input = f(1.0);
        input.environment.oxygen_mass_fraction = 0.0;
        let out = advance(c(), None, &a(), &initial, &input, n(1.0)).unwrap();
        close(out.apparatus_ledger.reacted_fuel_mass_kg, 0.0);
        close(out.safety.unburned_fuel_mass_kg, 1.0);
        close(out.predicted_apparatus_temperature_kelvin, 300.0);
        let mut apparatus = a();
        apparatus.conversion_fraction.value = 0.0;
        let out = advance(c(), None, &apparatus, &initial, &f(1.0), n(1.0)).unwrap();
        close(out.apparatus_ledger.reacted_fuel_mass_kg, 0.0);
        close(out.predicted_apparatus_temperature_kelvin, 300.0);
    }

    #[test]
    fn analytic_linear_cooling_and_steady_limit() {
        let mut apparatus = a();
        apparatus.linear_loss_conductance.value = 10.0;
        let mut state = s();
        state.apparatus_temperature_kelvin = 400.0;
        for _ in 0..1000 {
            state = advance(c(), None, &apparatus, &state, &f(0.0), n(0.001))
                .unwrap()
                .state;
        }
        let analytic = 300.0 + 100.0 * (-0.1_f64).exp();
        assert!((state.apparatus_temperature_kelvin - analytic).abs() < 0.001);
        let mut equilibrium = s();
        equilibrium.apparatus_temperature_kelvin = 350.0;
        equilibrium.actual_fuel_flow_kg_per_s = 1.0;
        let out = advance(c(), None, &apparatus, &equilibrium, &f(1.0), n(1.0)).unwrap();
        close(out.predicted_apparatus_temperature_kelvin, 350.0);
    }

    #[test]
    fn thermal_limit_is_information_not_a_safe_verdict() {
        let mut apparatus = a();
        apparatus.thermal_limit = Some(p(301.0, Unit::Kelvin));
        let out = advance(c(), None, &apparatus, &s(), &f(1.0), n(1.0)).unwrap();
        assert_eq!(out.safety.status, "UNKNOWN");
        assert_eq!(
            out.safety.thermal_limit_assessment,
            ThermalLimitAssessment::Exceeded
        );
        assert!(out.safety.thermal_limit_margin_kelvin.unwrap() < 0.0);
    }

    #[test]
    fn unavailable_closures_units_domains_and_large_steps_fail_closed() {
        let mut input = f(1.0);
        input.command = Command::VirtualMaterialDemand;
        assert!(advance(c(), None, &a(), &s(), &input, n(1.0)).is_err());
        let mut apparatus = a();
        apparatus.actuator_lag.value = 0.0;
        assert!(advance(c(), None, &apparatus, &s(), &f(1.0), n(1.0)).is_err());
        apparatus = a();
        apparatus.heat_of_combustion.unit = Unit::Watt;
        assert!(advance(c(), None, &apparatus, &s(), &f(1.0), n(1.0)).is_err());
        apparatus = a();
        apparatus.thermal_capacity.domain.gravity_m_per_s2 = [1.0, 2.0];
        assert!(advance(c(), None, &apparatus, &s(), &f(1.0), n(1.0)).is_err());
        apparatus = a();
        apparatus.linear_loss_conductance.value = 1000.0;
        assert!(advance(c(), None, &apparatus, &s(), &f(0.0), n(1.0)).is_err());
        input = f(1.0);
        input.environment.oxygen_mass_fraction = f64::NAN;
        assert!(advance(c(), None, &a(), &s(), &input, n(1.0)).is_err());
        apparatus = a();
        apparatus.quasi_steady_combustion_assertion.clear();
        assert!(advance(c(), None, &apparatus, &s(), &f(0.0), n(1.0)).is_err());
    }
}

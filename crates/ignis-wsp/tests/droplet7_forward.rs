//! Synthetic invariant tests. These inputs are not NASA measurements or authored candidates.
//! Neutral tests reuse committed literal source verbatim; enabled end-to-end tests use the
//! parent author's candidate when it exists. The dimensionless thermal fixture below exercises
//! conservation and numerical limits with structural numbers only, not fuel-property estimates.

mod common;

use ignis_core::contracts::{ModelRelease, RawValue};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::thermal::{Gas, State, Thermal};
use ignis_wsp::{assemble, droplet6, droplet7, flex_tools, models, text};
use serde_json::json;

fn loaded(mut release: ModelRelease, params: TensorMap) -> LoadedRelease {
    release.id = content_hash(&release, &["id", "created_at"]);
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

fn root(key: &str, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let source = if key.ends_with('6') && !src.contains("thermal.on") {
        format!("{src}\n{THERMAL_OFF}")
    } else {
        src.to_string()
    };
    let ops = text::compile(&source, &spec).expect("literal source");
    let patch = assemble::make_patch(&spec, common::meta("test-droplet7"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("release");
    loaded(release, tensors)
}

#[test]
fn disabled_thermal_is_exact_predecessor_on_both_fuel_fixtures() {
    for (dir, old_fuel, new_fuel, old_key, new_key) in [
        (
            "methanol",
            droplet6::Fuel::Methanol,
            droplet7::Fuel::Methanol,
            "flex_methanol5",
            "flex_methanol6",
        ),
        (
            "heptane",
            droplet6::Fuel::Heptane,
            droplet7::Fuel::Heptane,
            "flex_heptane5",
            "flex_heptane6",
        ),
    ] {
        let path = format!(
            "{}/../../configs/models/flex/wsp5/{dir}/baseline-d2/edits.wsp",
            env!("CARGO_MANIFEST_DIR")
        );
        let src = std::fs::read_to_string(path).unwrap();
        let old = root(old_key, &src);
        let new = root(new_key, &src);
        let mut p = flex_tools::fixture_prefix(0.003, 0.5e-6, 5.0, 0.21, 0.0, 0.0, 101325.0);
        p.known_conditions.get_mut("fuel").unwrap().raw_value =
            Some(RawValue::Text(new_fuel.label().into()));
        p.input_hash = content_hash(&p, &["input_hash"]);
        let settings = json!({"times_s": [5.0, 6.0, 10.0, 20.0]});
        let a = droplet6::forecast(&droplet6::spec(old_fuel), &old, &p, &settings)
            .unwrap()
            .unwrap();
        let b = droplet7::forecast(&droplet7::spec(new_fuel), &new, &p, &settings)
            .unwrap()
            .unwrap();
        assert_eq!(a.quantiles, b.quantiles);
        assert_eq!(a.latent_quantiles, b.latent_quantiles);
        assert_eq!(a.mean, b.mean);
        assert_eq!(a.state_probability, b.state_probability);
        assert_eq!(a.events, b.events);
        assert!(b.encoded.added.thermal.is_none());
    }
}

/// Artificial unit system, not a physical fuel. The simple latent law is exactly 1 J/kg;
/// all gas species share properties, and Antoine pressure stays in a dilute domain.
fn synthetic() -> Thermal {
    Thermal {
        nitrogen_only: false,
        cutoff_on: true,
        apply: [true; 3],
        dt_s: 0.01,
        gas_constant_j_mol_k: 1.0,
        molar_mass_kg_mol: 1.0,
        rho_kg_m3: 1.0,
        cp_liquid_j_kg_k: 1.0,
        cp_gas_j_kg_k: 1.0,
        k_gas_w_m_k: [0.01; 3],
        diffusivity_ref_m2_s: [0.01; 3],
        transport_ref_k: 1.0,
        transport_ref_pa: 1.0,
        diffusivity_exponent: 0.0,
        antoine: [-5.0, 1.0, 0.0],
        antoine_range_k: [0.5, 5.0],
        latent: [0.001, 0.0, 0.0, 10.0],
        latent_range_k: [0.5, 5.0],
        vapor_extension: false,
        latent_extrapolation: false,
        ambient_k: 2.0,
        cloud_excess_k: [0.0; 3],
        cloud_tau_s: [1.0; 3],
        initial_k: [2.0; 3],
        activity: [1.0; 3],
        x_infinity: 0.0,
        surface_fraction_max: 0.9,
    }
}

/// Explicit physical zeros, never tacit material defaults, for the disabled mechanism.
const THERMAL_OFF: &str = "
thermal.member_weight = 1
thermal.on = 0
thermal.gas_constant_j_mol_k = 0
thermal.molar_mass_kg_mol = 0
thermal.rho_kg_m3 = 0
thermal.cp_liquid_j_kg_k = 0
thermal.cp_gas_j_kg_k = 0
thermal.k_gas_w_m_k[0:3] = 0
thermal.diffusivity_ref_m2_s[0:3] = 0
thermal.transport_ref_k = 0
thermal.transport_ref_pa = 0
thermal.diffusivity_exponent = 0
thermal.antoine[0:3] = 0
thermal.antoine_range_k[0:2] = 0
thermal.latent[0:4] = 0
thermal.latent_range_k[0:2] = 0
thermal.ambient_k = 0
thermal.cloud_excess_k[0:3] = 0
thermal.cloud_tau_s[0:3] = 0
thermal.initial_k[0:3] = 0
thermal.activity[0:3] = 0
thermal.x_infinity = 0
thermal.surface_fraction_max = 0
";

/// The same structural fixture expressed as literal assignments. Every numerical token is
/// typed, not emitted from a program; this block never leaves the synthetic test suite.
const THERMAL_UNUSED: &str = "
thermal.member_weight = 1
thermal.on = 1
thermal.cutoff_on = 0
thermal.apply[0:3] = 0
thermal.dt_s = 0.01
thermal.gas_constant_j_mol_k = 1
thermal.molar_mass_kg_mol = 1
thermal.rho_kg_m3 = 1
thermal.cp_liquid_j_kg_k = 1
thermal.cp_gas_j_kg_k = 1
thermal.k_gas_w_m_k[0:3] = 0.01
thermal.diffusivity_ref_m2_s[0:3] = 0.01
thermal.transport_ref_k = 1
thermal.transport_ref_pa = 1
thermal.diffusivity_exponent = 0
thermal.antoine[0:3] = [-5, 1, 0]
thermal.antoine_range_k[0:2] = [0.5, 5]
thermal.latent[0:4] = [0.001, 0, 0, 10]
thermal.latent_range_k[0:2] = [0.5, 5]
thermal.vapor_extension = 0
thermal.latent_extrapolation = 0
thermal.ambient_k = 2
thermal.cloud_excess_k[0:3] = 0
thermal.cloud_tau_s[0:3] = 1
thermal.initial_k[0:3] = 2
thermal.activity[0:3] = 1
thermal.x_infinity = 0
thermal.surface_fraction_max = 0.9
";

#[test]
fn cutoff_switch_off_preserves_the_inherited_no_flame_history() {
    let src = std::fs::read_to_string(format!(
        "{}/../../configs/models/flex/wsp5/methanol/baseline-d2/edits.wsp",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let src = force_unlit(src);
    let old = root("flex_methanol5", &src);
    let new = root("flex_methanol6", &format!("{src}\n{THERMAL_UNUSED}"));
    let p = flex_tools::fixture_prefix(0.003, 0.05e-6, 5.0, 0.21, 0.0, 0.0, 101325.0);
    let settings = json!({"times_s": [5.0, 6.0, 10.0, 20.0]});
    let a = droplet6::forecast(
        &droplet6::spec(droplet6::Fuel::Methanol),
        &old,
        &p,
        &settings,
    )
    .unwrap()
    .unwrap();
    let b = droplet7::forecast(
        &droplet7::spec(droplet7::Fuel::Methanol),
        &new,
        &p,
        &settings,
    )
    .unwrap()
    .unwrap();
    assert_eq!(a.encoded.state, b.encoded.state);
    assert_eq!(a.quantiles, b.quantiles);
    assert_eq!(a.mean, b.mean);
    assert_eq!(a.unlit_at_cutoff.law, b.unlit_at_cutoff.law);
    assert!(b
        .encoded
        .added
        .thermal
        .unwrap()
        .cutoff_history
        .contains("inherited"));
    assert_eq!(b.thermal_execution.unwrap().integrated_steps, 0);
}

#[test]
fn unsupported_carrier_retains_every_predecessor_branch_explicitly() {
    let src = std::fs::read_to_string(format!(
        "{}/../../configs/models/flex/wsp5/methanol/baseline-d2/edits.wsp",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let src = force_unlit(src);
    // Switch every route on but restrict carrier support. Very small artificial transport
    // coefficients keep the assembly's separate synthetic N2 fixture in this toy domain.
    let thermal = THERMAL_UNUSED
        .replace("thermal.cutoff_on = 0", "thermal.cutoff_on = 1")
        .replace("thermal.apply[0:3] = 0", "thermal.apply[0:3] = 1")
        .replace(
            "thermal.k_gas_w_m_k[0:3] = 0.01",
            "thermal.k_gas_w_m_k[0:3] = 0.000000000001",
        )
        .replace(
            "thermal.diffusivity_ref_m2_s[0:3] = 0.01",
            "thermal.diffusivity_ref_m2_s[0:3] = 0.000000000001",
        );
    let old = root("flex_methanol5", &src);
    let new = root(
        "flex_methanol6",
        &format!("{src}\n{thermal}\nthermal.nitrogen_only = 1\n"),
    );
    let p = flex_tools::fixture_prefix(0.003, 0.05e-6, 5.0, 0.21, 0.1, 0.2, 101325.0);
    let settings = json!({"times_s": [5.0, 6.0, 10.0, 20.0]});
    let a = droplet6::forecast(
        &droplet6::spec(droplet6::Fuel::Methanol),
        &old,
        &p,
        &settings,
    )
    .unwrap()
    .unwrap();
    let b = droplet7::forecast(
        &droplet7::spec(droplet7::Fuel::Methanol),
        &new,
        &p,
        &settings,
    )
    .unwrap()
    .unwrap();
    assert_eq!(a.encoded.state, b.encoded.state);
    assert_eq!(a.quantiles, b.quantiles);
    assert_eq!(a.mean, b.mean);
    assert_eq!(a.events, b.events);
    let diagnostics = b.encoded.added.thermal.unwrap();
    assert!(!diagnostics.active_for_carrier);
    assert!(diagnostics
        .carrier_fallback
        .unwrap()
        .contains("predecessor"));
    assert!(b.thermal_execution.is_none());
}

#[test]
fn impossible_total_gas_fraction_is_refused_before_evolution() {
    let src = std::fs::read_to_string(format!(
        "{}/../../configs/models/flex/wsp5/methanol/baseline-d2/edits.wsp",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let release = root("flex_methanol6", &src);
    let p = flex_tools::fixture_prefix(0.003, 0.5e-6, 5.0, 0.3, 0.4, 0.4, 101325.0);
    let support = droplet7::forecast(
        &droplet7::spec(droplet7::Fuel::Methanol),
        &release,
        &p,
        &json!({"times_s": [5.0]}),
    )
    .unwrap()
    .unwrap_err();
    assert!(support
        .reasons
        .iter()
        .any(|r| r.contains("sum to at most one")));
}

/// Structural test forcing: the hot phase ended before cutoff and the legacy law evaporates.
/// These typed test-only assignments are not a scientific candidate or a material estimate.
fn force_unlit(src: String) -> String {
    src.replace("crit.b0 = 5.0", "crit.b0 = -4.0")
        .replace("rad.b0 = 0.0", "rad.b0 = -3.0")
        .replace("post.k_inf = 0.0", "post.k_inf = 0.0000001")
}

fn toy_active_thermal() -> String {
    THERMAL_UNUSED
        .replace("thermal.cutoff_on = 0", "thermal.cutoff_on = 1")
        .replace("thermal.apply[0:3] = 0", "thermal.apply[0:3] = 1")
        .replace(
            "thermal.k_gas_w_m_k[0:3] = 0.01",
            "thermal.k_gas_w_m_k[0:3] = 0.000000000001",
        )
        .replace(
            "thermal.diffusivity_ref_m2_s[0:3] = 0.01",
            "thermal.diffusivity_ref_m2_s[0:3] = 0.000000000001",
        )
}

#[test]
fn joint_forecast_mixes_complete_cdfs_and_preserves_member_mean_and_events() {
    let src = std::fs::read_to_string(format!(
        "{}/../../configs/models/flex/wsp5/methanol/baseline-d2/edits.wsp",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let src = force_unlit(src);
    let thermal = toy_active_thermal();
    let pure = root("flex_methanol6", &format!("{src}\n{thermal}"));
    let old = root(
        "flex_methanol6",
        &format!(
            "{src}\n{}",
            thermal.replace("thermal.member_weight = 1", "thermal.member_weight = 0")
        ),
    );
    let joint = root(
        "flex_methanol6",
        &format!(
            "{src}\n{}",
            thermal.replace("thermal.member_weight = 1", "thermal.member_weight = 0.5")
        ),
    );
    let p = flex_tools::fixture_prefix(0.003, 0.05e-6, 5.0, 0.21, 0.0, 0.0, 101325.0);
    let settings = json!({"times_s": [5.0, 6.0, 10.0, 20.0]});
    let run = |r: &LoadedRelease| {
        droplet7::forecast(&droplet7::spec(droplet7::Fuel::Methanol), r, &p, &settings)
            .unwrap()
            .unwrap()
    };
    let (a, b, c) = (run(&pure), run(&old), run(&joint));
    assert!(a
        .mean
        .iter()
        .zip(&b.mean)
        .any(|(a, b)| (a - b).abs() > 1.0e-8));
    let diagnostics = c.ensemble.as_ref().unwrap();
    assert_eq!(diagnostics.thermal_weight, 0.5);
    assert_eq!(diagnostics.predecessor_weight, 0.5);
    for j in 0..c.times_s.len() {
        assert_eq!(c.mean[j], 0.5 * a.mean[j] + 0.5 * b.mean[j]);
        for e in 0..4 {
            assert_eq!(c.events[e][j], 0.5 * a.events[e][j] + 0.5 * b.events[e][j]);
        }
        let atoms = &c.atoms_by_time[j];
        let total: f64 = atoms.iter().map(|a| a.1).sum();
        for (l, level) in c.levels.iter().enumerate() {
            let q = c.latent_quantiles[l][j];
            let below: f64 = atoms.iter().filter(|a| a.0 < q).map(|a| a.1).sum::<f64>() / total;
            let at: f64 = atoms.iter().filter(|a| a.0 <= q).map(|a| a.1).sum::<f64>() / total;
            assert!(
                below <= *level + 1.0e-12 && at >= *level - 1.0e-12,
                "mixture quantile must invert the pooled CDF"
            );
            assert!(q >= a.latent_quantiles[l][j].min(b.latent_quantiles[l][j]) - 1.0e-14);
            assert!(q <= a.latent_quantiles[l][j].max(b.latent_quantiles[l][j]) + 1.0e-14);
        }
    }
}

#[test]
fn zero_member_weight_ignores_unused_thermal_properties_and_neutral_mixture_is_exact() {
    let src = std::fs::read_to_string(format!(
        "{}/../../configs/models/flex/wsp5/methanol/baseline-d2/edits.wsp",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let base = root("flex_methanol6", &src);
    let zero = root(
        "flex_methanol6",
        &format!(
            "{src}\n{}",
            THERMAL_OFF
                .replace("thermal.member_weight = 1", "thermal.member_weight = 0")
                .replace("thermal.on = 0", "thermal.on = 1")
        ),
    );
    let half = root(
        "flex_methanol6",
        &format!(
            "{src}\n{}",
            THERMAL_OFF.replace("thermal.member_weight = 1", "thermal.member_weight = 0.5")
        ),
    );
    let p = flex_tools::fixture_prefix(0.003, 0.5e-6, 5.0, 0.21, 0.0, 0.0, 101325.0);
    let settings = json!({"times_s": [5.0, 10.0, 20.0]});
    let run = |r: &LoadedRelease| {
        droplet7::forecast(&droplet7::spec(droplet7::Fuel::Methanol), r, &p, &settings)
            .unwrap()
            .unwrap()
    };
    let a = run(&base);
    for b in [run(&zero), run(&half)] {
        assert_eq!(a.quantiles, b.quantiles);
        assert_eq!(a.mean, b.mean);
        assert_eq!(a.events, b.events);
    }
}

const GAS: Gas = Gas {
    pressure_pa: 1.0,
    x_co2: 0.0,
    x_he: 0.0,
};
const INITIAL: State = State {
    q_m2: 1.0,
    temperature_k: 2.0,
};

#[test]
fn flux_obeys_spherical_mass_and_energy_balances() {
    let th = synthetic();
    th.validate().unwrap();
    let state = State {
        temperature_k: 1.5,
        ..INITIAL
    };
    let f = th.flux(state, 0.0, GAS, 0).unwrap();
    let radius = state.q_m2.sqrt() / 2.0;
    let mass = 4.0 * std::f64::consts::PI / 3.0 * th.rho_kg_m3 * radius.powi(3);
    let mdot = std::f64::consts::PI * th.rho_kg_m3 * radius * f.k_m2_s / 2.0;
    let beta = mdot * th.cp_gas_j_kg_k / (4.0 * std::f64::consts::PI * radius * th.k_gas_w_m_k[0]);
    let heat = 4.0
        * std::f64::consts::PI
        * radius
        * th.k_gas_w_m_k[0]
        * (th.ambient_k - state.temperature_k)
        * beta
        / beta.exp_m1();
    let storage = mass * th.cp_liquid_j_kg_k * f.dt_k_s;
    assert!((storage - (heat - mdot * f.latent_j_kg)).abs() < 1.0e-15);
    let molar_flux = 4.0 * std::f64::consts::PI * radius * GAS.pressure_pa
        / (th.gas_constant_j_mol_k * 0.5 * (th.ambient_k + state.temperature_k))
        * th.diffusivity_ref_m2_s[0]
        * (1.0 / (1.0 - f.surface_fraction)).ln();
    assert!((mdot - th.molar_mass_kg_mol * molar_flux).abs() < 1.0e-15);
}

#[test]
fn explicit_trajectory_converges_and_stays_physical() {
    let mut th = synthetic();
    let run = |th: &Thermal| th.evolve(INITIAL, 0.0, 2.0, GAS, 0, 0.001).unwrap();
    th.dt_s = 0.2;
    let coarse = run(&th);
    th.dt_s = 0.1;
    let fine = run(&th);
    th.dt_s = 0.025;
    let reference = run(&th);
    let err = |s: State| {
        (s.q_m2 - reference.q_m2).abs() + (s.temperature_k - reference.temperature_k).abs()
    };
    assert!(err(fine) < err(coarse) / 3.0);
    assert!(reference.q_m2 > 0.0 && reference.q_m2 < INITIAL.q_m2);
    assert!(reference.temperature_k > 0.0 && reference.temperature_k < INITIAL.temperature_k);
}

#[test]
fn cooling_trajectory_conserves_latent_and_stored_energy_in_the_adiabatic_limit() {
    let mut th = synthetic();
    // With a vanishing conductivity, the Stefan correction drives gas heat input to zero.
    // Constant cp and L then imply T-T0 = (3 L / 2 cp) ln(q/q0).
    th.k_gas_w_m_k = [1.0e-12; 3];
    let final_state = th.evolve(INITIAL, 0.0, 2.0, GAS, 0, 0.001).unwrap();
    let temperature_from_energy = INITIAL.temperature_k
        + 1.5 * th.latent_heat(INITIAL.temperature_k).unwrap() / th.cp_liquid_j_kg_k
            * (final_state.q_m2 / INITIAL.q_m2).ln();
    assert!((final_state.temperature_k - temperature_from_energy).abs() < 1.0e-8);
}

#[test]
fn cached_trajectories_do_not_depend_on_horizon_query_order() {
    use ignis_wsp::thermal::Trajectories;
    let th = synthetic();
    let mut forward = Trajectories::new(&th, GAS, 0.001);
    let early = forward.at(1.0, 0.137, 0).unwrap();
    let late = forward.at(1.0, 1.431, 0).unwrap();
    let mut reverse = Trajectories::new(&th, GAS, 0.001);
    let late_reverse = reverse.at(1.0, 1.431, 0).unwrap();
    let early_reverse = reverse.at(1.0, 0.137, 0).unwrap();
    assert_eq!(early.q_m2, early_reverse.q_m2);
    assert_eq!(early.temperature_k, early_reverse.temperature_k);
    assert_eq!(late.q_m2, late_reverse.q_m2);
    assert_eq!(late.temperature_k, late_reverse.temperature_k);
    assert_eq!(forward.trace.cached_trajectories, 1);
}

#[test]
fn saturated_gas_has_no_evaporation_and_no_latent_heat_sink() {
    let mut th = synthetic();
    th.x_infinity = 0.5;
    let f = th.flux(INITIAL, 0.0, GAS, 0).unwrap();
    assert_eq!(f.k_m2_s, 0.0);
    assert_eq!(f.dt_k_s, 0.0);
    assert_eq!(
        th.evolve(INITIAL, 0.0, 2.0, GAS, 0, 0.001).unwrap().q_m2,
        INITIAL.q_m2
    );
}

#[test]
fn unsupported_property_states_refuse_and_extensions_are_explicit() {
    let mut th = synthetic();
    assert!(th.vapor_pressure_pa(0.4).is_err());
    assert!(th.latent_heat(0.4).is_err());
    th.vapor_extension = true;
    let edge = th.vapor_pressure_pa(0.5).unwrap();
    let below = th.vapor_pressure_pa(0.4).unwrap();
    assert!((below / edge - (1.0_f64 / 0.5 - 1.0 / 0.4).exp()).abs() < 1.0e-14);
    th.latent_extrapolation = true;
    assert_eq!(th.latent_heat(0.4).unwrap(), 1.0);
    assert!(th.latent_heat(10.0).is_err());
    assert!(th.vapor_pressure_pa(10.0).is_err());
}

#[test]
fn initial_pressure_cap_is_analytic_and_last_flame_activity_is_used() {
    let mut th = synthetic();
    th.surface_fraction_max = 0.2;
    let (initial, capped) = th.initial(1.0, 1.0, 0).unwrap();
    assert!(capped);
    assert!((th.vapor_pressure_pa(initial.temperature_k).unwrap() - 0.2).abs() < 1.0e-15);
    th.activity[1] = 0.0;
    let (diffusive, capped) = th.initial(1.0, 1.0, 1).unwrap();
    assert!(!capped);
    assert_eq!(th.flux(diffusive, 0.0, GAS, 1).unwrap().k_m2_s, 0.0);
}

#[test]
fn invalid_steps_and_parameters_fail_closed() {
    let mut th = synthetic();
    th.dt_s = 0.0;
    assert!(th.validate().is_err());
    th = synthetic();
    th.activity[2] = 2.0;
    assert!(th.validate().is_err());
    th = synthetic();
    assert!(th.evolve(INITIAL, 2.0, 1.0, GAS, 0, 0.001).is_err());
    assert!(th
        .flux(
            State {
                temperature_k: -1.0,
                ..INITIAL
            },
            0.0,
            GAS,
            0
        )
        .is_err());
}

#[test]
fn first_order_floor_predictor_cannot_erase_a_strongly_cooling_droplet() {
    let mut th = synthetic();
    th.dt_s = 1.0;
    th.cp_liquid_j_kg_k = 0.0001;
    // The initial rate alone crosses the floor, but its predicted huge cooling cannot be
    // ignored. This coarse step must refuse or resolve a positive residual, never absorb early.
    let result = th.evolve(INITIAL, 0.0, 1.0, GAS, 0, 0.99);
    assert!(result.is_err() || result.unwrap().q_m2 > 0.99);
}

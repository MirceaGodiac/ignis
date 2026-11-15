//! Everything about the `droplet9.rs` models that is not their forward: release declarations and the
//! synthetic fixture of each fuel. Kept out of `droplet9.rs` so that a change here never changes the
//! forward's identity. Same declarations as `droplet_tools.rs` gives the earlier droplet forwards,
//! with this forward's fuel names, domain and horizon.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use ignis_core::contracts::{PrefixBundle, RawValue, TensorFill};

use crate::droplet9::{spec, Fuel};

/// What the registry record of a `droplet9` release declares besides spec, patch and tensors.
pub fn release_notes(fuel: Fuel) -> (BTreeMap<&'static str, Value>, Value, Value) {
    let s = spec(fuel);
    let constant = |name: &str| {
        s.tensors
            .iter()
            .find(|t| t.name == name)
            .and_then(|t| match &t.fill {
                TensorFill::Fixed { values } => values.first().map(|v| v.get()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{name} is a constant of droplet9::spec"))
    };
    let applicability = BTreeMap::from([
        ("fuel", json!(fuel.label().to_lowercase())),
        ("configuration", json!("single free-floating droplet")),
        ("time_origin", json!(ignis_core::views::FLEX_TIME_ORIGIN)),
        ("domain", json!(s.domain)),
        (
            "meaning",
            json!("declared physical domain of the handwritten law, not a training-support region"),
        ),
    ]);
    let observation_model = json!({
        "prefix": "liquid_diameter observations, valid, positive, 0 <= t <= cutoff, first of equal time stamps, after the validity gate (a value later frames exceed by more than gate.growth_tol is dropped)",
        "noise": "independent Gaussian of sd unc.sigma_d on each measurement (assumed)",
        "calibration": "published calibration assumed applied upstream by the data lane",
    });
    let numerics = json!({
        "integrator": "RK4 of the hot phase on a uniform grid of num.dt_s from the cutoff; closed form after a transition",
        "scenarios": "num.z_count rate nodes spaced num.z_step and num.ce_count cool-extinction nodes spaced num.ce_step, normal weights",
        "t_valid_s": constant("num.t_valid_s"),
        "d_min_m": constant("num.d_min_m"),
        "request_time_fitting": "none",
    });
    (applicability, observation_model, numerics)
}

/// A deterministic synthetic prefix of the fuel: an exact D^2-law droplet of 3 mm burning at
/// 0.55 mm2/s in air at 1 atm, cutoff 3 s, free, an even mixture for a two-component fuel, igniter
/// energy 1 J (never NASA data).
pub fn fixture_prefix(fuel: Fuel) -> PrefixBundle {
    let mut p = crate::flex_tools::fixture_prefix(3.0e-3, 0.55e-6, 3.0, 0.21, 0.0, 0.0, 101_325.0);
    if let Some(f) = p.known_conditions.get_mut("fuel") {
        f.raw_value = Some(RawValue::Text(fuel.label().into()));
    }
    // the conditions sections 3.2 and 3.3 read, so that a release with those laws switched on can
    // be checked on the fixture: a free droplet, an even mixture of a two-component fuel, and an
    // igniter energy equal to ig.e_ref (c_ign = 0). Synthetic, like everything in this prefix.
    let template = p.known_conditions["x_o2"].clone();
    let mut put = |key: &str, value: f64, unit: &str| {
        let mut m = template.clone();
        m.raw_value = Some(RawValue::Number(value));
        m.raw_unit = Some(unit.into());
        m.value_si = Some(value);
        m.unit_si = Some(unit.into());
        p.known_conditions.insert(key.into(), m);
    };
    put("fiber", 0.0, "1");
    if fuel.component_b().is_some() {
        put("y_b_mass", 0.5, "kg/kg");
    }
    if fuel.has_cool() {
        put("igniter_power", 1.0, "W");
        put("igniter_time", 1.0, "s");
    }
    p.run_id = format!("wsp-fixture-{}", fuel.label().to_lowercase());
    p.input_hash = ignis_core::hashing::content_hash(&p, &["input_hash"]);
    p
}

//! Everything about the droplet models that is not their forward: release declarations, the
//! synthetic fixture and read-only inspection of a release's laws. Kept out of `droplet.rs` so that
//! adding an inspection helper never changes the forward's identity.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use ignis_core::contracts::{ModelSpec, PrefixBundle, RawValue};
use ignis_core::expert::Refusal;
use ignis_core::registry::LoadedRelease;

use crate::droplet::{spec, Fuel, Laws, Weights, H_RATE};
use crate::spec::Params;

/// The fuel of the third-generation droplet forward as the fuel of the first one (same labels), so
/// both generations share the fixture and the release declarations.
pub fn fuel_of3(fuel: crate::droplet3::Fuel) -> Fuel {
    match fuel {
        crate::droplet3::Fuel::Methanol => Fuel::Methanol,
        crate::droplet3::Fuel::Heptane => Fuel::Heptane,
    }
}

/// What the registry record of a droplet release declares besides spec, patch and tensors.
pub fn release_notes(fuel: Fuel) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(fuel, json!(spec(fuel).domain))
}

/// [`release_notes`] of a `droplet3` release: the same declarations with that spec's domain.
pub fn release_notes3(
    fuel: crate::droplet3::Fuel,
) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(fuel_of3(fuel), json!(crate::droplet3::spec(fuel).domain))
}

/// The fuel of the fourth-generation droplet forward as the fuel of the first one.
pub fn fuel_of4(fuel: crate::droplet4::Fuel) -> Fuel {
    match fuel {
        crate::droplet4::Fuel::Methanol => Fuel::Methanol,
        crate::droplet4::Fuel::Heptane => Fuel::Heptane,
    }
}

/// [`release_notes`] of a `droplet4` release: the same declarations with that spec's domain.
pub fn release_notes4(
    fuel: crate::droplet4::Fuel,
) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(fuel_of4(fuel), json!(crate::droplet4::spec(fuel).domain))
}

/// The fuel of the flame-reading droplet forward as the fuel of the first one.
pub fn fuel_of5(fuel: crate::droplet5::Fuel) -> Fuel {
    match fuel {
        crate::droplet5::Fuel::Methanol => Fuel::Methanol,
        crate::droplet5::Fuel::Heptane => Fuel::Heptane,
    }
}

/// [`release_notes`] of a `droplet5` release: the same declarations with that spec's domain.
pub fn release_notes5(
    fuel: crate::droplet5::Fuel,
) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(fuel_of5(fuel), json!(crate::droplet5::spec(fuel).domain))
}

/// The fuel of the sixth droplet forward as the fuel of the first one.
pub fn fuel_of6(fuel: crate::droplet6::Fuel) -> Fuel {
    match fuel {
        crate::droplet6::Fuel::Methanol => Fuel::Methanol,
        crate::droplet6::Fuel::Heptane => Fuel::Heptane,
    }
}

/// [`release_notes`] of a `droplet6` release: the same declarations with that spec's domain.
pub fn release_notes6(
    fuel: crate::droplet6::Fuel,
) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(fuel_of6(fuel), json!(crate::droplet6::spec(fuel).domain))
}

fn notes(fuel: Fuel, domain: Value) -> (BTreeMap<&'static str, Value>, Value, Value) {
    let applicability = BTreeMap::from([
        ("fuel", json!(fuel.label().to_lowercase())),
        ("configuration", json!("single free-floating droplet")),
        ("time_origin", json!(ignis_core::views::FLEX_TIME_ORIGIN)),
        ("domain", domain),
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
        "t_valid_s": 45.0,
        "d_min_m": 2.5e-4,
        "request_time_fitting": "none",
    });
    (applicability, observation_model, numerics)
}

/// A deterministic synthetic prefix of the fuel: an exact D^2-law droplet in air (never NASA data).
pub fn fixture_prefix(fuel: Fuel) -> PrefixBundle {
    let mut p = crate::flex_tools::fixture_prefix(3.0e-3, 0.55e-6, 3.0, 0.21, 0.0, 0.0, 101_325.0);
    if let Some(f) = p.known_conditions.get_mut("fuel") {
        f.raw_value = Some(RawValue::Text(fuel.label().into()));
    }
    p.run_id = format!("wsp-fixture-{}", fuel.label().to_lowercase());
    p.input_hash = ignis_core::hashing::content_hash(&p, &["input_hash"]);
    p
}

/// The condition-level laws of a release for one droplet, and its hot-flame rate along a burn at
/// the population level (`(t s, D mm, K mm2/s)` rows), for inspection. No data and no gradient.
pub fn inspect(
    spec: &ModelSpec,
    release: &LoadedRelease,
    fuel: Fuel,
    c: [f64; 5],
) -> Result<(Laws, Vec<(f64, f64, f64)>), Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params, fuel)?;
    let mut scratch = [0.0; H_RATE];
    let laws = wt.laws(&c, &mut scratch);
    let d0 = 0.002 * c[4].exp();
    let q0 = d0 * d0;
    let (mut t, mut q) = (0.0, q0);
    let dt = 0.02;
    let mut rows = Vec::new();
    while q > 0.04 * q0 && t < 60.0 {
        let k = wt.k_hot_pop(
            &c,
            (1.0 - q / q0).clamp(0.0, 1.0),
            t * wt.k_ref / q0,
            &mut scratch,
        );
        if ((t / dt).round() as i64) % 25 == 0 {
            rows.push((t, q.sqrt() * 1e3, k * 1e6));
        }
        q -= k * dt;
        t += dt;
    }
    Ok((laws, rows))
}

/// Hot-flame population rate (m^2/s) of a release at one state.
pub fn k_hot(
    spec: &ModelSpec,
    release: &LoadedRelease,
    fuel: Fuel,
    c: [f64; 5],
    w: f64,
    s: f64,
) -> Result<f64, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params, fuel)?;
    let mut scratch = [0.0; H_RATE];
    Ok(wt.k_hot_pop(&c, w, s, &mut scratch))
}

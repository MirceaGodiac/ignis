//! Fixture and release declarations for Astra's additive model identities.
//! These declarations do not participate in numerical forecast computation.

use std::collections::BTreeMap;

use ignis_core::contracts::Family;
use serde_json::{json, Value};

use crate::{droplet, droplet7, droplet_tools, flame_tools, front_tools};

pub fn fuel_of7(fuel: droplet7::Fuel) -> droplet::Fuel {
    match fuel {
        droplet7::Fuel::Methanol => droplet::Fuel::Methanol,
        droplet7::Fuel::Heptane => droplet::Fuel::Heptane,
    }
}

pub fn droplet_notes(fuel: droplet7::Fuel) -> (BTreeMap<&'static str, Value>, Value, Value) {
    let (mut app, obs, mut num) = droplet_tools::release_notes(fuel_of7(fuel));
    app.insert("domain", json!(droplet7::spec(fuel).domain));
    app.insert("thermal_assumptions", json!("optional pure-fuel molar Stefan transport, lumped constant-density liquid; typed initial temperature and gas heat history; activity is a declared approximation to liquid composition"));
    num["integrator"] = json!(
        "inherited RK4 hot phase and closed forms; enabled thermal post-flame branch uses explicit midpoint steps of authored thermal.dt_s with absorbing minimum diameter"
    );
    num["property_domains"] = json!(
        "source intervals and explicit extrapolation switches travel with thermal coefficients; enabled approximation limits appear in forecast diagnostics"
    );
    (app, obs, num)
}

pub fn front_notes() -> (BTreeMap<&'static str, Value>, Value, Value) {
    let (mut app, obs, mut num) = front_tools::release_notes2();
    app.insert("domain", json!(crate::front3::spec().domain));
    app.insert("oxygen_closure", json!("steady oxygen budget with L = v residence_time; total combustible loading and active faces explicit; visible-front transients and quench hazard remain separate closures"));
    num["speed"] = json!(
        "authored mixture of empirical spread and positive quadratic-root oxygen-supply closure; actual width converts SI speed to sample widths/s"
    );
    num["law_committee"] = json!("oxygen.member_weight mixes complete persistent position distributions; active joint applies the new coherent first-event mapping to both speed scenarios, not the old artifact event rows");
    num["event_coordinates"] = json!("active route: p_quenched is first-event quench incidence; p_reached_end is F_end/(1-F_quench), preserving the legacy DTO interface; artifact rows are the actual competing incidences");
    (app, obs, num)
}

pub fn condition_notes(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    let (mut app, obs, mut num) = flame_tools::release_notes5(family);
    app.insert(
        "domain",
        json!(crate::condition_committee::spec(family).domain),
    );
    app.insert("condition_scenarios", json!("five persistent flow/pressure nodes; supplied composition, Tad and Zst unchanged; weights are assumed at the cutoff, not inferred or calibrated"));
    num["outer_mixture"] = json!(
        "whole conditional competing-risk trajectories averaged with the same normalized node masses at every horizon; no hazard averaging or repeated node selection"
    );
    (app, obs, num)
}

pub fn front4_notes() -> (BTreeMap<&'static str, Value>, Value, Value) {
    let (mut app, mut obs, mut num) = front_notes();
    app.insert("domain", json!(crate::front4::spec().domain));
    obs["prefix_operator"] = json!("empirical mean at each channel's admitted still times; tip overhang, law advance and delayed base use matching sample operators; the base delay remains evaluated once at its late-window mean time");
    num["prefix_anchor"] = json!("last-window mean tip minus mean modeled overhang, propagated from mean law position; mean time remains the inherited gain and lead clock");
    num["uncertainty_limits"] = json!("unchanged coefficients and variance formula applied to window-relative increments; not a covariance model for averaged measurement noise or drift");
    (app, obs, num)
}

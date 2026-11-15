//! Everything about the ACME clock models that is not their forward: the release record's
//! declarations. The synthetic fixture is the first generation's (`acme_tools::default_fixture`).

use std::collections::BTreeMap;

use serde_json::{json, Value};

use ignis_core::contracts::Family;

use crate::flame::{spec, INTERPRETATION};

/// What the registry record of an ACME clock release declares besides spec, patch and tensors.
pub fn release_notes(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(
        family,
        json!(spec(family).domain),
        "midpoint hazards, step <= num.dt_s, exact competing-risk split within a step, two latent classes re-weighted by survival to the cutoff",
    )
}

/// [`release_notes`] of a growth-clock (`flame3`) release.
pub fn release_notes3(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(
        family,
        json!(crate::flame3::spec(family).domain),
        "closed-form survival of three independent families (self-extinction: the weak class and num.phi_nodes equally likely values of the reduced flow; burner limit: num.bt_nodes heating nodes times the weighted limits; programmed stops and background), conditioned on the cutoff; step <= num.dt_s, the loss of a step split between causes in proportion to -ln of each family's survival ratio",
    )
}

/// [`release_notes`] of a committee (`committee.rs`) release.
pub fn release_notes5(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(
        family,
        json!(crate::committee::spec(family).domain),
        "weighted mean of up to six members' event tables at the requested times; flame.rs members: two latent classes integrated with hazards constant over each step; flame3.rs members: closed-form survival of three independent families conditioned on the cutoff; each member with its own step and constants",
    )
}

/// [`release_notes`] of a release of the second committee (`committee2.rs`).
pub fn release_notes7(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(
        family,
        json!(crate::committee2::spec(family).domain),
        "weighted mean of the members' event tables at the requested times, over up to ten members and, when the script's planned stop is read, up to three stop times; flame.rs members: two latent classes integrated with hazards constant over each step; flame3.rs members: closed-form survival of three independent families conditioned on the cutoff; flame4.rs members: the two latent classes of flame.rs with the closed-form burner-heating survival of flame3.rs as the burner limit; each member with its own step and constants",
    )
}

/// [`release_notes`] of a hinge-law release with the burner-heating law (`flame4.rs`).
pub fn release_notes8(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(
        family,
        json!(crate::flame4::spec(family).domain),
        "midpoint hazards, step <= num.dt_s, exact competing-risk split within a step, two latent classes re-weighted by survival to the cutoff; the burner limit enters each step as the hazard ln(S_bt(t) / S_bt(t + h)) / h of the closed-form burner-heating survival (num.bt_nodes heating nodes times the weighted limits)",
    )
}

/// [`notes`] of a release of a forward with the fuel table: the fuels are those its table admits.
fn fuel_table_notes(
    family: Family,
    domain: Value,
    integrator: &str,
) -> (BTreeMap<&'static str, Value>, Value, Value) {
    let (mut a, o, n) = notes(family, domain, integrator);
    a.insert(
        "fuels",
        json!("the rows of the release's fuel table with fuel.admit = 1 (rows: ethane, propane, butane, ethylene, methane, hydrogen); the row of a prefix is its fuel_species, else its stored indicators"),
    );
    (a, o, n)
}

/// [`release_notes`] of a hinge-law release with the fuel table (`flame5.rs`).
pub fn release_notes9(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    fuel_table_notes(
        family,
        json!(crate::flame5::spec(family).domain),
        "midpoint hazards, step <= num.dt_s, exact competing-risk split within a step, two latent classes re-weighted by survival to the cutoff; each law network evaluated at the alkane vertices with a non-zero weight of the fuel's coordinates and its outputs weighed",
    )
}

/// [`release_notes`] of a growth-clock release with the fuel table (`flame6.rs`).
pub fn release_notes10(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    fuel_table_notes(
        family,
        json!(crate::flame6::spec(family).domain),
        "closed-form survival of three independent families (self-extinction: the weak class and num.phi_nodes equally likely values of the reduced flow; burner limit: num.bt_nodes heating nodes times the weighted limits; programmed stops and background), conditioned on the cutoff; step <= num.dt_s, the loss of a step split between causes in proportion to -ln of each family's survival ratio; each law network evaluated at the alkane vertices with a non-zero weight of the fuel's coordinates and its outputs weighed",
    )
}

/// [`release_notes`] of a hinge-law release with the burner-heating law and the fuel table
/// (`flame7.rs`).
pub fn release_notes11(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    fuel_table_notes(
        family,
        json!(crate::flame7::spec(family).domain),
        "midpoint hazards, step <= num.dt_s, exact competing-risk split within a step, two latent classes re-weighted by survival to the cutoff; the burner limit enters each step as the hazard ln(S_bt(t) / S_bt(t + h)) / h of the closed-form burner-heating survival (num.bt_nodes heating nodes times the weighted limits); each law network evaluated at the alkane vertices with a non-zero weight of the fuel's coordinates and its outputs weighed",
    )
}

/// [`release_notes`] of a release of the third committee (`committee3.rs`).
pub fn release_notes12(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    fuel_table_notes(
        family,
        json!(crate::committee3::spec(family).domain),
        "weighted mean of the members' event tables at the requested times, over up to ten members and, when the script's planned stop is read, up to three stop times (the rules of committee2.rs); flame5.rs, flame6.rs and flame7.rs members, each the law of flame.rs, flame3.rs or flame4.rs reading its own fuel table, with its own step and constants",
    )
}

/// [`release_notes`] of a blend (`blend.rs`) release.
pub fn release_notes4(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    notes(
        family,
        json!(crate::blend::spec(family).domain),
        "weighted mean of two members' event tables at the requested times; member a (flame.rs): two latent classes integrated with hazards constant over each step; member b (flame3.rs): closed-form survival of three independent families conditioned on the cutoff; each member with its own step and constants",
    )
}

fn notes(
    family: Family,
    domain: Value,
    integrator: &str,
) -> (BTreeMap<&'static str, Value>, Value, Value) {
    let applicability = BTreeMap::from([
        (
            "configuration",
            json!(if family == Family::AcmeInverse {
                "inverse"
            } else {
                "normal"
            }),
        ),
        (
            "population",
            json!("established hot flame of a burner-stabilised gas flame (PSI-159)"),
        ),
        ("fuels", json!(["ethane", "propane", "butane"])),
        ("time_origin", json!(ignis_core::views::ACME_TIME_ORIGIN)),
        ("domain", domain),
        ("interpretation", json!(INTERPRETATION)),
        (
            "meaning",
            json!(
                "declared physical domain of the handwritten clocks, not a training-support region"
            ),
        ),
    ]);
    let observation_model = json!({
        "prefix": "prospective conditions and control stages known at the cutoff; no time series",
        "endpoint": "cause-specific event time; an operator or script stop is its own cause, never an extinction; self-extinction is the end of the hot flame",
        "censoring": "handled by the evaluator; the forward conditions on the flame burning at the cutoff",
    });
    let numerics = json!({
        "integrator": integrator,
        "horizon_s": 600.0,
        "request_time_fitting": "none",
    });
    (applicability, observation_model, numerics)
}

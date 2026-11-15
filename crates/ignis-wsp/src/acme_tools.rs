//! Everything about the ACME literal models that is not their forward: the synthetic fixture
//! prefix, the release record's declarations and read-only inspection of the hazards. Kept out of
//! `acme.rs` so that adding a helper never changes the forward's identity.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use ignis_core::contracts::{
    EvidenceClass, Family, Measured, ModelSpec, PrefixBundle, RawValue, Resolution,
};
use ignis_core::expert::Refusal;
use ignis_core::registry::LoadedRelease;

use crate::acme::{spec, Weights, HIDDEN, INTERPRETATION};
use crate::spec::Params;

/// What the registry record of an ACME literal release declares besides spec, patch and tensors.
pub fn release_notes(family: Family) -> (BTreeMap<&'static str, Value>, Value, Value) {
    let applicability = BTreeMap::from([
        (
            "configuration",
            json!(if family == Family::AcmeInverse { "inverse" } else { "normal" }),
        ),
        ("population", json!("established hot flame of a burner-stabilised gas flame (PSI-159)")),
        ("fuels", json!(["ethane", "propane", "butane"])),
        ("time_origin", json!(ignis_core::views::ACME_TIME_ORIGIN)),
        ("domain", json!(spec(family).domain)),
        ("interpretation", json!(INTERPRETATION)),
        (
            "meaning",
            json!("declared physical domain of the handwritten hazards, not a training-support region"),
        ),
    ]);
    let observation_model = json!({
        "prefix": "prospective conditions and control stages known at the cutoff; no time series",
        "endpoint": "cause-specific event time; an operator stop is its own cause, never an extinction",
        "censoring": "handled by the evaluator; the forward conditions on the phase being established at the cutoff",
    });
    let numerics = json!({
        "integrator": "midpoint hazards, step <= num.dt_s, exact competing-risk split within a step",
        "horizon_s": 600.0,
        "request_time_fitting": "none",
    });
    (applicability, observation_model, numerics)
}

/// A deterministic synthetic ACME prefix (conditions only; every value `evidence_class =
/// synthetic`). For build checks, tests and benchmarks; never NASA data.
#[allow(clippy::too_many_arguments)]
pub fn fixture_prefix(
    family: Family,
    cutoff_s: f64,
    pressure_pa: f64,
    injected_kg_s: f64,
    x_o2_role: f64,
    x_fuel_role: f64,
    tad_k: f64,
    zst: f64,
    fuel: &str,
) -> PrefixBundle {
    let measured = |v: f64, unit: &str| Measured {
        raw_value: Some(RawValue::Number(v)),
        raw_unit: Some(unit.into()),
        value_si: Some(v),
        unit_si: Some(unit.into()),
        uncertainty_si: None,
        uncertainty_reason: None,
        evidence_class: EvidenceClass::Synthetic,
        locator: None,
        derivation: None,
        resolution: Resolution::Resolved,
        flags: vec![],
    };
    let mut species = measured(0.0, "1");
    species.raw_value = Some(RawValue::Text(fuel.into()));
    species.value_si = None;
    let mut p = PrefixBundle {
        family,
        run_id: format!("wsp-fixture-{}", family.as_str()),
        session_id: None,
        cutoff_s,
        time_origin: ignis_core::views::ACME_TIME_ORIGIN.into(),
        known_conditions: BTreeMap::from([
            ("pressure".to_string(), measured(pressure_pa, "Pa")),
            (
                "injected_mass_flow".to_string(),
                measured(injected_kg_s, "kg/s"),
            ),
            ("x_o2_role".to_string(), measured(x_o2_role, "1")),
            ("x_fuel_role".to_string(), measured(x_fuel_role, "1")),
            ("tad".to_string(), measured(tad_k, "K")),
            ("zst".to_string(), measured(zst, "1")),
            ("fuel_species".to_string(), species),
        ]),
        known_controls: vec![],
        observations: vec![],
        contours: vec![],
        phase_established_at_s: Some(0.0),
        support_evidence: vec!["synthetic fixture".into()],
        transform_versions: BTreeMap::new(),
        input_hash: String::new(),
    };
    p.input_hash = ignis_core::hashing::content_hash(&p, &["input_hash"]);
    p
}

/// The default fixture of a configuration: mid-range conditions, cutoff 3 s.
pub fn default_fixture(family: Family) -> PrefixBundle {
    match family {
        Family::AcmeInverse => fixture_prefix(
            family, 3.0, 101_325.0, 5.0e-6, 0.50, 0.10, 2400.0, 0.45, "propane",
        ),
        _ => fixture_prefix(
            family, 3.0, 101_325.0, 1.0e-6, 0.30, 0.30, 2300.0, 0.25, "propane",
        ),
    }
}

/// Both hazards (1/s) of a release along time for one prefix's conditions (inspection only).
pub fn hazard_curves(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    times_s: &[f64],
) -> Result<Vec<(f64, f64, f64)>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params);
    let c = wt
        .features(prefix)
        .map_err(|r| Refusal::unsupported("condition", r.join("; ")))?;
    let mut scratch = [0.0; HIDDEN];
    Ok(times_s
        .iter()
        .map(|&t| {
            let (e, o) = wt.hazards(&c, t, &mut scratch);
            (t, e, o)
        })
        .collect())
}

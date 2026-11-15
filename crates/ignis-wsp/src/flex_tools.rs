//! Everything about `flex.wsp` that is not its forward: the synthetic fixture prefix, the release
//! record's declarations and read-only inspection of a release's population law. Kept out of
//! `flex.rs` so that adding an inspection helper never changes the forward's identity.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use ignis_core::contracts::{Family, ModelSpec, PrefixBundle, Quantity};
use ignis_core::expert::Refusal;
use ignis_core::registry::{LoadedRelease, Tensor, TensorMap};

use crate::flex::{spec, Weights, HIDDEN};
use crate::spec::Params;

/// Named curves of a release for inspection: the rate and extinction networks along burn progress
/// for one condition vector at a fixed clock `s`, hidden activations included. No data and no
/// gradient.
pub fn curves(
    spec: &ModelSpec,
    release: &LoadedRelease,
    c: [f64; 5],
    s: f64,
) -> Result<TensorMap, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params)?;
    let n = 101;
    let (mut w, mut k, mut eta) = (vec![], vec![], vec![]);
    let mut hid_rate = Vec::with_capacity(n * HIDDEN);
    let mut hid_haz = Vec::with_capacity(n * HIDDEN);
    let mut scratch = [0.0; HIDDEN];
    for i in 0..n {
        let wi = i as f64 / (n - 1) as f64;
        let x = [c[0], c[1], c[2], c[3], c[4], wi, s];
        w.push(wi);
        k.push(wt.k_ref * wt.rate.eval(&x, &mut scratch).exp());
        hid_rate.extend(scratch.iter().map(|v| v.max(0.0)));
        eta.push(wt.haz.eval(&x, &mut scratch).min(wt.log_eta_cap).exp());
        hid_haz.extend(scratch.iter().map(|v| v.max(0.0)));
    }
    let mut out = TensorMap::new();
    out.insert("w".into(), Tensor::vector(w));
    out.insert("rate.k_m2_s".into(), Tensor::vector(k));
    out.insert("haz.eta".into(), Tensor::vector(eta));
    out.insert("rate.hidden".into(), Tensor::new(vec![n, HIDDEN], hid_rate));
    out.insert("haz.hidden".into(), Tensor::new(vec![n, HIDDEN], hid_haz));
    Ok(out)
}

/// The population law of a release at one state, for inspection: burning-rate constant (m^2/s)
/// and extinction intensity per unit progress, without run offset or scenario.
pub fn population(
    spec: &ModelSpec,
    release: &LoadedRelease,
    c: [f64; 5],
    w: f64,
    s: f64,
) -> Result<(f64, f64), Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params)?;
    let mut scratch = [0.0; HIDDEN];
    let x = [c[0], c[1], c[2], c[3], c[4], w, s];
    Ok((
        wt.k_ref * wt.rate.eval(&x, &mut scratch).exp(),
        wt.haz.eval(&x, &mut scratch).min(wt.log_eta_cap).exp(),
    ))
}

/// Burn progress at which half of the droplets of a condition have lost their hot flame under the
/// release's extinction network (clock `s` held fixed); `None` when the cumulative intensity never
/// reaches ln 2 before the droplet is gone.
pub fn median_extinction_progress(
    spec: &ModelSpec,
    release: &LoadedRelease,
    c: [f64; 5],
    s: f64,
) -> Result<Option<f64>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params)?;
    let mut scratch = [0.0; HIDDEN];
    let n = 2000;
    let eta = |w: f64, scratch: &mut [f64; HIDDEN]| {
        let x = [c[0], c[1], c[2], c[3], c[4], w, s];
        wt.haz.eval(&x, scratch).min(wt.log_eta_cap).exp()
    };
    let (mut cum, mut prev) = (0.0, eta(0.0, &mut scratch));
    for i in 1..=n {
        let w = i as f64 / n as f64;
        let e = eta(w, &mut scratch);
        cum += 0.5 * (prev + e) / n as f64;
        if cum >= std::f64::consts::LN_2 {
            return Ok(Some(w));
        }
        prev = e;
    }
    Ok(None)
}

/// What the registry record of a `flex.wsp` release declares besides spec, patch and tensors.
pub fn release_notes() -> (BTreeMap<&'static str, Value>, Value, Value) {
    let applicability = BTreeMap::from([
        ("fuel", json!("methanol")),
        ("configuration", json!("single free-floating droplet")),
        ("time_origin", json!(ignis_core::views::FLEX_TIME_ORIGIN)),
        ("domain", json!(spec().domain)),
        (
            "meaning",
            json!("declared physical domain of the handwritten law, not a training-support region"),
        ),
    ]);
    let observation_model = json!({
        "prefix": "liquid_diameter observations, valid, positive, 0 <= t <= cutoff, first of equal time stamps",
        "noise": "independent Gaussian of sd unc.sigma_d on each measurement (assumed)",
        "calibration": "published calibration assumed applied upstream by the data lane",
    });
    let numerics = json!({
        "integrator": "RK4, fixed step <= num.dt_s, steps end on forecast times",
        "scenarios": "num.z_count nodes spaced num.z_step, normal weights",
        "t_valid_s": 30.0,
        "d_min_m": 2.5e-4,
        "request_time_fitting": "none",
    });
    (applicability, observation_model, numerics)
}

/// A deterministic synthetic FLEX prefix: an exact D^2-law droplet sampled at 30 Hz up to the
/// cutoff, every observation `evidence_class = synthetic`. For build checks, tests and benchmarks;
/// never NASA data.
#[allow(clippy::too_many_arguments)]
pub fn fixture_prefix(
    d0_m: f64,
    k_m2_s: f64,
    cutoff_s: f64,
    x_o2: f64,
    x_co2: f64,
    x_he: f64,
    pressure_pa: f64,
) -> PrefixBundle {
    use ignis_core::contracts::{
        Derivation, EvidenceClass, Measured, Method, Observation, RawValue, Resolution,
        SourceLocator,
    };
    let derivation = || Derivation {
        method: Method::Deterministic,
        method_id: "ignis_wsp.flex.fixture_prefix".into(),
        method_version: "1".into(),
        inputs: vec![],
        agent: None,
        notes: Some("synthetic D^2-law fixture".into()),
    };
    let run_id = "wsp-fixture-flex".to_string();
    let n = (cutoff_s * 30.0).floor() as usize;
    let observations = (0..=n)
        .map(|i| {
            let t = i as f64 / 30.0;
            let d = (d0_m * d0_m - k_m2_s * t).max(0.0).sqrt();
            Observation {
                id: format!("{run_id}:d:{i}"),
                run_id: run_id.clone(),
                quantity: Quantity::LiquidDiameter,
                channel: None,
                time_s: t,
                available_at_s: t,
                raw_value: Some(RawValue::Number(d * 1e3)),
                raw_unit: Some("mm".into()),
                value_si: Some(d),
                unit_si: "m".into(),
                uncertainty_si: None,
                calibration_id: None,
                quality_flags: vec![],
                evidence_class: EvidenceClass::Synthetic,
                locator: SourceLocator::default(),
                derivation: derivation(),
            }
        })
        .collect();
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
    let mut fuel = measured(0.0, "1");
    fuel.raw_value = Some(RawValue::Text("Methanol".into()));
    fuel.value_si = None;
    let mut p = PrefixBundle {
        family: Family::Flex,
        run_id,
        session_id: None,
        cutoff_s,
        time_origin: ignis_core::views::FLEX_TIME_ORIGIN.into(),
        known_conditions: BTreeMap::from([
            ("x_o2".to_string(), measured(x_o2, "1")),
            ("x_co2".to_string(), measured(x_co2, "1")),
            ("x_he".to_string(), measured(x_he, "1")),
            ("pressure".to_string(), measured(pressure_pa, "Pa")),
            ("fuel".to_string(), fuel),
        ]),
        known_controls: vec![],
        observations,
        contours: vec![],
        phase_established_at_s: Some(0.0),
        support_evidence: vec!["synthetic fixture".into()],
        transform_versions: BTreeMap::new(),
        input_hash: String::new(),
    };
    p.input_hash = ignis_core::hashing::content_hash(&p, &["input_hash"]);
    p
}

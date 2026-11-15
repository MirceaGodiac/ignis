//! Everything about `bass.wsp` that is not its forward: the synthetic metric fixture and the release
//! record's declarations.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use ignis_core::contracts::{
    ContourObservation, ContourStatus, Derivation, EvidenceClass, Family, Method, PrefixBundle,
    SourceLocator,
};

/// What the registry record of a `bass.wsp` release declares besides spec, patch and tensors.
pub fn release_notes() -> (BTreeMap<&'static str, Value>, Value, Value) {
    let applicability = BTreeMap::from([
        ("observable", json!("luminous front position in a metrically calibrated camera plane")),
        ("time_origin", json!(ignis_core::views::BASS_TIME_ORIGIN)),
        (
            "qualification",
            json!("none possible today: no BASS run of the cohort has a metric calibration, so no metric \
                   forecast has been or can be scored; the release is runnable and unevaluated"),
        ),
        (
            "refuses",
            json!("pixel-only contours, contours without a calibration, fewer than four usable contours"),
        ),
    ]);
    let observation_model = json!({
        "prefix": "single-valued contours on one transverse grid, at or before the cutoff",
        "units": "metres only; a pixel contour is refused, never rescaled",
        "not_inferred": "temperature, species, pyrolysis front, extinction",
    });
    let numerics = json!({
        "projection": "trapezoid integrals against four cosine modes (declared quadrature, no fit)",
        "propagation": "closed form per mode",
        "request_time_fitting": "none",
    });
    (applicability, observation_model, numerics)
}

/// A deterministic synthetic BASS prefix with METRIC contours: a front
/// `f(s, t) = f0 + v t + tilt cos(pi xi)` on a 20 mm wide grid, five frames per second up to the
/// cutoff. `pixel_only` writes the vision lane's uncalibrated marker instead. Never NASA data.
pub fn fixture_prefix(cutoff_s: f64, v_m_s: f64, tilt_m: f64, pixel_only: bool) -> PrefixBundle {
    let n = 21;
    let s: Vec<f64> = (0..n).map(|i| 0.02 * i as f64 / (n - 1) as f64).collect();
    let frames = (cutoff_s * 5.0).floor() as usize;
    let run_id = "wsp-fixture-bass".to_string();
    let contours = (0..=frames)
        .map(|k| {
            let t = k as f64 / 5.0;
            let f = s
                .iter()
                .map(|x| Some(0.01 + v_m_s * t + tilt_m * (std::f64::consts::PI * x / 0.02).cos()))
                .collect();
            ContourObservation {
                id: format!("{run_id}:c:{k}"),
                run_id: run_id.clone(),
                time_s: t,
                available_at_s: t,
                s_m: s.clone(),
                f_m: f,
                status: ContourStatus::SingleValued,
                calibration_id: if pixel_only {
                    "pixel-identity".into()
                } else {
                    "synthetic-metric".into()
                },
                scale_uncertainty_rel: None,
                mask_dependency_frames: vec![k as i64],
                frame: k as i64,
                evidence_class: EvidenceClass::Synthetic,
                locator: SourceLocator::default(),
                derivation: Derivation {
                    method: Method::Deterministic,
                    method_id: "ignis_wsp.bass.fixture_prefix".into(),
                    method_version: "1".into(),
                    inputs: vec![],
                    agent: None,
                    notes: Some(if pixel_only {
                        "UNCALIBRATED: s_m/f_m hold pixel coordinates, not metres".into()
                    } else {
                        "synthetic metric fixture".into()
                    }),
                },
            }
        })
        .collect();
    let mut p = PrefixBundle {
        family: Family::Bass,
        run_id,
        session_id: None,
        cutoff_s,
        time_origin: ignis_core::views::BASS_TIME_ORIGIN.into(),
        known_conditions: BTreeMap::new(),
        known_controls: vec![],
        observations: vec![],
        contours,
        phase_established_at_s: Some(0.0),
        support_evidence: vec!["synthetic fixture".into()],
        transform_versions: BTreeMap::new(),
        input_hash: String::new(),
    };
    p.input_hash = ignis_core::hashing::content_hash(&p, &["input_hash"]);
    p
}

//! The BASS strict forward on synthetic metric contours with test-only coefficients: exact
//! kinematics, mode relaxation, and the unit guard that keeps pixel contours out.

mod common;

use ignis_core::contracts::{ContourStatus, Family, PrefixBundle, SupportGate};
use ignis_core::expert::Expert;
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::bass::{self, Forecast};
use ignis_wsp::{assemble, bass_tools, text};
use serde_json::{json, Value};

fn release(edits: &str) -> LoadedRelease {
    let spec = bass::spec();
    let ops = text::compile(edits, &spec).unwrap_or_else(|e| panic!("{e}\n{edits}"));
    let patch = assemble::make_patch(&spec, common::meta("test-bass"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release =
        ignis_wsp::build_release(Family::Bass, &patch, &tensors, None, &common::binding())
            .expect("release");
    LoadedRelease {
        release,
        params: tensors,
        posterior: TensorMap::new(),
    }
}

/// Pure translation: the mean keeps its velocity for ever, the shape is frozen, no spread.
const TRANSLATE: &str = "
op.gain[0:4] = [1.0, 0.0, 0.0, 0.0]
op.persist_s[0:4] = [1.0e5, 1.0, 1.0, 1.0]
op.damp_per_s[0:4] = 0.0
unc.sigma_m = 0.0
unc.rel_v = 0.0
unc.sigma_v_m_s = 0.0
";

fn run(rel: &LoadedRelease, p: &PrefixBundle, settings: Value) -> Forecast {
    bass::forecast(&bass::spec(), rel, p, &settings)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn rehash(mut p: PrefixBundle) -> PrefixBundle {
    p.input_hash = content_hash(&p, &["input_hash"]);
    p
}

#[test]
fn a_translating_front_is_carried_exactly_and_its_shape_relaxes_at_the_written_rate() {
    // f(s, t) = 0.01 + 0.002 t + 0.001 cos(pi xi): mean 0.01 + 0.002 t, tilt amplitude 0.001 / sqrt(2)
    let p = bass_tools::fixture_prefix(4.0, 0.002, 0.001, false);
    let f = run(
        &release(TRANSLATE),
        &p,
        json!({"times_s": [4.0, 6.5, 14.0]}),
    );
    assert!((f.velocities_m_s[0] - 0.002).abs() < 1e-12);
    assert!(
        (f.amplitudes_m[1] - 0.001 / 2.0_f64.sqrt()).abs() < 1e-6,
        "{:?}",
        f.amplitudes_m
    );
    for (j, t) in f.times_s.iter().enumerate() {
        // the carried drift is v T (1 - exp(-D/T)): with T = 1e5 s it is D to within D^2 / (2 T)
        assert!(
            (f.mean_m[j] - (0.01 + 0.002 * t)).abs() < 2e-6,
            "mean at {t}"
        );
        for (i, s) in f.s_m.iter().enumerate() {
            let want = 0.01 + 0.002 * t + 0.001 * (std::f64::consts::PI * s / 0.02).cos();
            assert!(
                (f.field_m[j][i] - want).abs() < 2e-6,
                "front at s = {s}, t = {t}"
            );
        }
        assert_eq!(f.sd_m[j], 0.0);
    }
    // damping only the tilt mode at 0.5 /s: after 2 s its amplitude is exp(-1) of what it was, and
    // what the modes did not resolve stays put
    let damped = release(&TRANSLATE.replace(
        "op.damp_per_s[0:4] = 0.0",
        "op.damp_per_s[0:4] = [0.0, 0.5, 0.0, 0.0]",
    ));
    let g = run(&damped, &p, json!({"times_s": [6.0]}));
    let tilt_now = g.field_m[0][0] - g.mean_m[0];
    let tilt_then = 0.001;
    assert!(
        (tilt_now / tilt_then - (-1.0_f64).exp()).abs() < 2e-3,
        "tilt ratio {}",
        tilt_now / tilt_then
    );
    // unresolved nodes are bridged along s, and a contour that is mostly missing is not used
    let mut q = p.clone();
    q.contours[10].f_m[7] = None;
    for v in q.contours[12].f_m.iter_mut().take(15) {
        *v = None;
    }
    q.contours[14].status = ContourStatus::Multivalued;
    let h = run(&release(TRANSLATE), &rehash(q), json!({"times_s": [6.5]}));
    assert_eq!(h.frames_used, 21 - 2);
    assert!((h.mean_m[0] - (0.01 + 0.002 * 6.5)).abs() < 1e-6);
}

#[test]
fn pixel_contours_are_refused_and_never_rescaled() {
    let rel = release(TRANSLATE);
    let spec = bass::spec();
    let refused = |p: PrefixBundle, settings: Value| {
        bass::forecast(&spec, &rel, &rehash(p), &settings)
            .expect("runnable")
            .expect_err("should be refused")
    };
    // the vision lane's marker for an identity (pixel) mapping
    let s = refused(
        bass_tools::fixture_prefix(4.0, 2.0, 30.0, true),
        Value::Null,
    );
    assert_eq!(s.gate, SupportGate::Measurement);
    assert!(s.reasons[0].contains("pixel"), "{:?}", s.reasons);
    // one uncalibrated contour among metric ones is enough
    let mut p = bass_tools::fixture_prefix(4.0, 0.002, 0.001, false);
    p.contours[3].derivation.notes =
        Some("UNCALIBRATED: s_m/f_m hold pixel coordinates, not metres".into());
    assert_eq!(refused(p, Value::Null).gate, SupportGate::Measurement);
    // a contour that names no calibration is not metric either
    let mut p = bass_tools::fixture_prefix(4.0, 0.002, 0.001, false);
    p.contours[0].calibration_id = String::new();
    assert_eq!(refused(p, Value::Null).gate, SupportGate::Measurement);
    // too few frames, another family, a lead beyond the declared maximum
    assert_eq!(
        refused(
            bass_tools::fixture_prefix(0.4, 0.002, 0.001, false),
            Value::Null
        )
        .gate,
        SupportGate::Measurement
    );
    let mut p = bass_tools::fixture_prefix(4.0, 0.002, 0.001, false);
    p.family = Family::Flex;
    assert_eq!(refused(p, Value::Null).gate, SupportGate::Semantic);
    let p = bass_tools::fixture_prefix(4.0, 0.002, 0.001, false);
    assert_eq!(
        refused(p.clone(), json!({"times_s": [5.0, 90.0]})).gate,
        SupportGate::Horizon
    );

    // the refusal artifact keeps the metric target and carries no numbers; the admitted one carries
    // the contour field the front scorer reads
    let a = bass::artifact(
        &spec,
        &rel,
        &rehash(bass_tools::fixture_prefix(4.0, 2.0, 30.0, true)),
        &Value::Null,
    )
    .unwrap();
    assert!(!a.support.admitted && a.mean.is_empty() && a.unit.as_deref() == Some("m"));
    let ok = bass::BassWsp::new()
        .forecast(&rel, &p, &Value::Null)
        .unwrap();
    ok.validate().unwrap();
    assert_eq!(
        ok.diagnostics["front_mean_m"].as_array().unwrap().len(),
        ok.times_s.len()
    );
    assert_eq!(ok.diagnostics["front_s_m"].as_array().unwrap().len(), 21);
}

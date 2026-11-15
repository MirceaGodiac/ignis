//! The fifth flame-front forward of `bass_front.wsp5` (`front5.rs`): the tip of `front.rs` unchanged,
//! its quench hazard replaced by the boundary law of `docs/wsp/research/phys_quench_w0.md` section 6.
//!
//! * the spec: the tensors of section 6.2 exactly, the quadrature as a constant of the spec;
//! * neutrality on the tip: the committed carry of D1 forecasts the tip of every synthetic burn and
//!   of the four train tracks bit for bit as D1 does, and with the neutral boundary every sample
//!   ignites and no flame goes out;
//! * the law: with the literal assignments of section 6.4 (a TEST fixture, copied from the dossier
//!   and checked against it), the dossier's own tables of section 4 at their printed precision; the
//!   long-time limits; a flow step down and back up on the same node;
//! * the three new inputs, their defaults and refusals; the opposed-flow burn, whose tip is refused
//!   and whose event probabilities are returned; the bounds of the new tensors.
//!
//! The 6.4 numbers are the dossier's W0 law; nothing here scores them on data.

mod common;

use ignis_core::contracts::{
    CheckStatus, EvidenceClass, Measured, ModelSpec, ReleaseId, Resolution, SupportGate,
};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::front5::{self, Condition, Direction, Outcome};
use ignis_wsp::front_tools::{self, Track};
use ignis_wsp::{assemble, audit, front, front5_tools, models, text};
use serde_json::{json, Value};

const FRONT_BASE: &str =
    include_str!("../../../configs/models/bass/front/baseline-persistence/edits.wsp");
const FRONT_D1: &str = include_str!("../../../configs/models/bass/front/d1-fable/edits.wsp");
const FRONT5_BASE: &str =
    include_str!("../../../configs/models/bass/front5/baseline-persistence/edits.wsp");
const FRONT5_CARRY: &str = include_str!("../../../configs/models/bass/front5/carry-d1/edits.wsp");

/// TEST FIXTURE: the literal assignments of section 6.4 of `docs/wsp/research/phys_quench_w0.md`,
/// copied from the dossier (checked against it by
/// `the_law_fixture_is_the_dossiers_section_6_4_verbatim`). Not a release of this repository.
const LAW_6_4: &str = "# floor of the oxygen limit: D4 (SIBAL), Takahashi 2014 film minimum plus thickness judgement (PMMA)
sus.x0[0:2] = [0.148, 0.136]
# pressure shift of the floor per e-fold of 1/p: judgement, D7
sus.cp[0:2] = [0.020, 0.020]
# blow-off slope per cm/s, rows material, columns [concurrent, opposed]: D5
sus.sb[0:2, 0:2] = [[0.00080, 0.0016], [0.00080, 0.0015]]
# radiative-loss coefficient: 0.055 (SIBAL) and 0.054 (PMMA) for G = 1, times G = 1.36 and 0.73: D3
sus.ar[0:2, 0:2] = [[0.075, 0.040], [0.074, 0.039]]
# lateral coefficient: section 1.6
sus.aw[0:2] = [0.030, 0.030]
# self-spread speed, de Ris thin limit: D6
sus.us[0:2] = [0.15, 0.020]
# standard deviations of the five coefficients
sus.sd_x0[0:2] = [0.010, 0.015]
sus.sd_cp[0:2] = [0.020, 0.020]
sus.sd_sb[0:2, 0:2] = [[0.00030, 0.00070], [0.00040, 0.00070]]
sus.sd_ar[0:2, 0:2] = [[0.026, 0.014], [0.030, 0.016]]
sus.sd_aw[0:2] = [0.030, 0.030]
# ignition by a hot wire at the sample edge: D8
ign.dx[0:2] = [0.0040, 0.0060]
ign.sd_a = 0.0050
ign.t_req[0:2] = [2.0, 10.0]
ign.sd_lnt = 0.50
# time to go out, rows material, columns [radiative quench, blow-off]: D9
ext.tau[0:2, 0:2] = [[12.0, 1.5], [100.0, 3.0]]
ext.width = 0.0020
";

const LEVELS: [f64; 5] = [0.05, 0.25, 0.5, 0.75, 0.95];
const CUTOFFS: [f64; 4] = [5.0, 10.0, 20.0, 40.0];
const HORIZONS: [f64; 5] = [10.0, 20.0, 30.0, 45.0, 60.0];
/// The duct speeds of the dossier's tables, cm/s.
const SPEEDS: [f64; 11] = [0.0, 1.0, 2.0, 3.0, 5.0, 7.5, 10.0, 15.0, 20.0, 30.0, 40.0];

fn repo(path: &str) -> String {
    let path = format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn loaded(mut release: ignis_core::contracts::ModelRelease, params: TensorMap) -> LoadedRelease {
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
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key}: {e}"));
    let patch = assemble::make_patch(&spec, common::meta("test-front5"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("release");
    loaded(release, tensors)
}

fn revise(key: &str, parent: &LoadedRelease, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let mut meta = common::meta("test-front5-revision");
    meta.parent_release_id = Some(ReleaseId::new(parent.release.id.clone()).unwrap());
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key} revision: {e}"));
    let patch = assemble::make_patch(&spec, meta, ops);
    let tensors = assemble::materialize(&spec, &patch, Some(parent)).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, Some(parent), &common::binding())
        .expect("revision release");
    loaded(release, tensors)
}

/// D1 of `bass_front.wsp` with its root, and the committed root and carry of D1 of
/// `bass_front.wsp5`, all built here from their committed edits.
fn committed() -> [LoadedRelease; 4] {
    let base1 = root("bass_front", FRONT_BASE);
    let d1 = revise("bass_front", &base1, FRONT_D1);
    let base5 = root("bass_front5", FRONT5_BASE);
    let carry = revise("bass_front5", &base5, FRONT5_CARRY);
    [base1, d1, base5, carry]
}

/// The carry of D1 with the dossier's law of section 6.4 in place of the neutral boundary.
fn with_law(carry: &LoadedRelease, extra: &str) -> LoadedRelease {
    revise("bass_front5", carry, &format!("{LAW_6_4}{extra}"))
}

fn run1(rel: &LoadedRelease, input: &front::FrontInput) -> front::Forecast {
    front::forecast_track(&front::spec(), rel, input, &LEVELS)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn run5(rel: &LoadedRelease, input: &front5::FrontInput) -> Outcome {
    front5::forecast_track(&front5::spec(), rel, input, &LEVELS)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn full(o: Outcome) -> front5::Forecast {
    match o {
        Outcome::Full(f) => f,
        Outcome::EventsOnly { tip_refusal, .. } => panic!("tip refused: {:?}", tip_refusal),
    }
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// The tip of the two forwards is the same, bit for bit: quantiles, mean, base, reached the end and
/// what the prefix said.
fn assert_same_tip(a: &front::Forecast, b: &front5::Forecast, what: &str) {
    assert_eq!(bits(&a.times_s), bits(&b.times_s), "{what}: times");
    assert_eq!(a.quantiles.len(), b.quantiles.len(), "{what}: levels");
    for (l, (qa, qb)) in a.quantiles.iter().zip(&b.quantiles).enumerate() {
        assert_eq!(bits(qa), bits(qb), "{what}: tip quantile {}", LEVELS[l]);
    }
    assert_eq!(bits(&a.mean), bits(&b.mean), "{what}: mean");
    assert_eq!(bits(&a.base_median), bits(&b.base_median), "{what}: base");
    assert_eq!(
        bits(&a.p_reached_end),
        bits(&b.p_reached_end),
        "{what}: reached the end"
    );
    let (ea, eb) = (&a.encoded, &b.encoded);
    assert_eq!(ea.stills_used, eb.stills_used, "{what}: stills");
    assert_eq!(
        (ea.e_tip, ea.e_base, ea.tip_end_w),
        (eb.e_tip, eb.e_base, eb.tip_end_w),
        "{what}: offsets"
    );
    assert_eq!(
        bits(&[
            ea.t_end_s,
            ea.span_s,
            ea.settled,
            ea.gain,
            ea.e,
            ea.s,
            ea.x_end_w,
            ea.h_end_w,
            ea.sigma_x_w,
            ea.laws_at_cutoff.v,
            ea.laws_at_cutoff.h_ss,
            ea.laws_at_cutoff.tau_b,
        ]),
        bits(&[
            eb.t_end_s,
            eb.span_s,
            eb.settled,
            eb.gain,
            eb.e,
            eb.s,
            eb.x_end_w,
            eb.h_end_w,
            eb.sigma_x_w,
            eb.laws_at_cutoff.v,
            eb.laws_at_cutoff.h_ss,
            eb.laws_at_cutoff.tau_b,
        ]),
        "{what}: encoder"
    );
}

/// A burn of the reference sample, igniter on from -6 s, one still per second from 0 to
/// `last_still_s` (none when `None`).
fn burn(
    cutoff_s: f64,
    x_o2: f64,
    flow: Vec<(f64, f64)>,
    last_still_s: Option<usize>,
    times: Vec<f64>,
) -> front::FrontInput {
    let ts: Vec<f64> = last_still_s
        .map(|n| (0..=n).map(|i| i as f64).collect())
        .unwrap_or_default();
    front::FrontInput {
        cutoff_s,
        igniter_on_s: -6.0,
        x_o2,
        width_ratio: 1.0,
        window_w: Some(4.5),
        flow,
        tip: ts.iter().map(|t| (*t, 0.8 + 0.05 * t)).collect(),
        base: ts.iter().map(|t| (*t, 0.1 + 0.05 * t)).collect(),
        times,
    }
}

fn grid(from: f64, to: f64, step: f64) -> Vec<f64> {
    let n = ((to - from) / step).round() as usize;
    (0..=n).map(|i| from + step * i as f64).collect()
}

fn train_value(name: &str) -> Value {
    serde_json::from_str(&repo(&format!(
        "tools/wsp/bass_optical/tracks/train/{name}.json"
    )))
    .unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn track_of(v: Value) -> Track {
    serde_json::from_value(v).expect("track")
}

/// The SIBAL fabric of the 2 cm class (2.2 cm, the dossier's table width) at 1 atm, concurrent.
fn sibal(x_o2: f64) -> Condition {
    Condition {
        x_o2,
        p_atm: 1.0,
        direction: Direction::Concurrent,
        material: 0,
        width_cm: 2.2,
    }
}

// ------------------------------------------------------------------------------------------- spec

#[test]
fn the_spec_is_the_first_front_spec_without_its_quench_and_with_the_tensors_of_section_6_2() {
    let spec = front5::spec();
    spec.validate().expect("spec valid");
    let on_disk: ModelSpec =
        serde_json::from_str(&repo("configs/models/bass/front5/spec.json")).unwrap();
    assert_eq!(
        on_disk.spec_hash(),
        spec.spec_hash(),
        "configs/models/bass/front5/spec.json drifted from front5::spec(); a changed spec is a new \
         spec id (re-export with `ignis wsp spec --family bass_front5 --out …`)"
    );
    assert_eq!(spec.spec_id.as_str(), "bass_front-wsp-v5");
    assert_eq!(spec.model_name, "bass_front.wsp5");
    assert_eq!(spec.forward.entry_point, "ignis_wsp::front5::forecast");
    assert_eq!(
        models::by_name("bass_front.wsp5").map(|m| m.key),
        Some("bass_front5")
    );
    // every tensor of the first front spec but quench.*, unchanged and in its order
    let first = front::spec();
    let kept: Vec<_> = first
        .tensors
        .iter()
        .filter(|t| !t.name.starts_with("quench."))
        .collect();
    let at = kept.iter().position(|t| t.name == "unc.drift_w").unwrap() + 1;
    let table: [(&str, Vec<u32>, (f64, f64), &str); 17] = [
        ("sus.x0", vec![2], (0.08, 0.30), "mole fraction"),
        (
            "sus.cp",
            vec![2],
            (0.0, 0.10),
            "mole fraction per e-fold of 1/p",
        ),
        ("sus.sb", vec![2, 2], (0.0, 0.01), "mole fraction per cm/s"),
        ("sus.ar", vec![2, 2], (0.0, 0.5), "mole fraction cm/s atm"),
        ("sus.aw", vec![2], (-0.2, 0.3), "mole fraction cm^2/s atm"),
        ("sus.us", vec![2], (0.0, 5.0), "cm/s"),
        ("sus.sd_x0", vec![2], (0.001, 0.05), "mole fraction"),
        (
            "sus.sd_cp",
            vec![2],
            (0.0, 0.10),
            "mole fraction per e-fold of 1/p",
        ),
        (
            "sus.sd_sb",
            vec![2, 2],
            (0.0, 0.01),
            "mole fraction per cm/s",
        ),
        (
            "sus.sd_ar",
            vec![2, 2],
            (0.0, 0.5),
            "mole fraction cm/s atm",
        ),
        ("sus.sd_aw", vec![2], (0.0, 0.3), "mole fraction cm^2/s atm"),
        ("ign.dx", vec![2], (-0.02, 0.05), "mole fraction"),
        ("ign.sd_a", vec![], (0.001, 0.03), "mole fraction"),
        ("ign.t_req", vec![2], (0.1, 120.0), "s"),
        ("ign.sd_lnt", vec![], (0.1, 2.0), "ln"),
        ("ext.tau", vec![2, 2], (0.1, 300.0), "s"),
        ("ext.width", vec![], (0.0005, 0.01), "mole fraction"),
    ];
    assert_eq!(spec.tensors.len(), kept.len() + table.len() + 2);
    for (a, b) in spec.tensors[..at].iter().zip(&kept[..at]) {
        assert_eq!(a, *b);
    }
    for (a, b) in spec.tensors[at + table.len()..].iter().zip(&kept[at..]) {
        assert_eq!(a, *b);
    }
    for (t, (name, shape, (lo, hi), unit)) in spec.tensors[at..at + table.len()].iter().zip(&table)
    {
        assert_eq!(&t.name, name);
        assert_eq!(&t.shape, shape, "{name}");
        let b = t.bounds.as_ref().unwrap();
        assert_eq!((b.min.get(), b.max.get()), (*lo, *hi), "{name}");
        assert_eq!(t.unit.as_deref(), Some(*unit), "{name}");
        assert!(matches!(
            t.fill,
            ignis_core::contracts::TensorFill::AuthorWritesAll
        ));
    }
    // the quadrature is a constant of the spec: 41 equal-probability nodes of a standard normal
    let fixed = |name: &str| -> Vec<f64> {
        match &spec.tensor(name).unwrap().fill {
            ignis_core::contracts::TensorFill::Fixed { values } => {
                values.iter().map(|v| v.get()).collect()
            }
            other => panic!("{name}: {other:?}"),
        }
    };
    assert_eq!(fixed("quad.z"), front5::Q41_Z.to_vec());
    assert_eq!(fixed("quad.w"), front5::Q41_W.to_vec());
    assert_eq!(front5::NODES, 41);
    // each node is the normal quantile at (k - 0.5) / 41: the erfc round-trip returns the level to
    // within two units in the last place, the nodes are increasing and symmetric, the centre is 0
    for k in 0..41 {
        let z = front5::Q41_Z[k];
        let level = (k as f64 + 0.5) / 41.0;
        let back = 0.5 * libm::erfc(-z / std::f64::consts::SQRT_2);
        assert!(
            (back - level).abs() <= 2.0 * f64::EPSILON,
            "node {k}: {back} against {level}"
        );
        assert_eq!(z, -front5::Q41_Z[40 - k], "node {k}: symmetry");
        assert_eq!(front5::Q41_W[k], 1.0 / 41.0);
    }
    assert!(front5::Q41_Z.windows(2).all(|w| w[1] > w[0]));
    assert_eq!(front5::Q41_Z[20], 0.0);
    assert!((front5::Q41_W.iter().sum::<f64>() - 1.0).abs() < 1e-15);
}

#[test]
fn the_forward_identity_is_the_shared_sources_and_front5_alone() {
    let names: Vec<&str> = audit::forward_sources_of("bass_front.wsp5")
        .expect("a literal model")
        .iter()
        .map(|s| s.0)
        .collect();
    assert_eq!(
        names,
        ["nn.rs", "quad.rs", "common.rs", "spec.rs", "front5.rs"]
    );
    assert_eq!(
        audit::banned_hits_of("bass_front.wsp5"),
        Vec::<String>::new()
    );
    let [_, d1, _, carry] = committed();
    let inp = burn(0.0, 0.19, vec![(-60.0, 5.0)], None, vec![20.0]);
    assert!(front::forecast_track(&front::spec(), &carry, &inp, &LEVELS).is_err());
    assert!(front5::forecast_track(
        &front5::spec(),
        &d1,
        &front5::FrontInput::from_first(inp),
        &LEVELS
    )
    .is_err());
}

#[test]
fn the_committed_root_and_carry_hold_the_tip_numbers_of_front_rs_and_the_neutral_boundary() {
    let [base1, d1, base5, carry] = committed();
    let neutral: [(&str, Vec<f64>); 17] = [
        ("ext.tau", vec![300.0; 4]),
        ("ext.width", vec![0.0005]),
        ("ign.dx", vec![-0.02, -0.02]),
        ("ign.sd_a", vec![0.001]),
        ("ign.sd_lnt", vec![0.1]),
        ("ign.t_req", vec![0.1, 0.1]),
        ("sus.ar", vec![0.0; 4]),
        ("sus.aw", vec![0.0; 2]),
        ("sus.cp", vec![0.0; 2]),
        ("sus.sb", vec![0.0; 4]),
        ("sus.sd_ar", vec![0.0; 4]),
        ("sus.sd_aw", vec![0.0; 2]),
        ("sus.sd_cp", vec![0.0; 2]),
        ("sus.sd_sb", vec![0.0; 4]),
        ("sus.sd_x0", vec![0.001, 0.001]),
        ("sus.us", vec![5.0, 5.0]),
        ("sus.x0", vec![0.08, 0.08]),
    ];
    for (first, fifth, dir) in [
        (&base1, &base5, "front5/baseline-persistence"),
        (&d1, &carry, "front5/carry-d1"),
    ] {
        for (name, t) in &first.params {
            if name.starts_with("quench.") {
                assert!(!fifth.params.contains_key(name), "{dir}: {name}");
                continue;
            }
            assert_eq!(
                fifth.params.get(name).map(|x| bits(&x.data)),
                Some(bits(&t.data)),
                "{dir}: {name}"
            );
        }
        let mut extra: Vec<(&str, Vec<f64>)> = fifth
            .params
            .iter()
            .filter(|(n, _)| !first.params.contains_key(*n) && !n.starts_with("quad."))
            .map(|(n, t)| (n.as_str(), t.data.clone()))
            .collect();
        extra.sort_by(|a, b| a.0.cmp(b.0));
        assert_eq!(extra, neutral.to_vec(), "{dir}");
        let on_disk: Value =
            serde_json::from_str(&repo(&format!("configs/models/bass/{dir}/release.json")))
                .unwrap();
        let m = fifth.release.construction.as_ref().unwrap();
        assert_eq!(
            on_disk["construction"]["weights_hash"],
            json!(m.weights_hash),
            "{dir}"
        );
        assert_eq!(
            on_disk["construction"]["forward_hash"],
            json!(m.forward_hash),
            "{dir}: assembled with another forward; rebuild it (python tools/wsp/reassemble.py --only bass_front5)"
        );
        assert!(m.checks.iter().all(|c| c.status == CheckStatus::Passed));
        assert_eq!(m.checks.len(), 5);
    }
}

// ------------------------------------------------------------------------------ neutrality on tip

#[test]
fn the_carry_of_d1_forecasts_the_tip_of_synthetic_burns_as_d1_does_and_every_burn_stays_lit() {
    let [_, d1, _, carry] = committed();
    let flows = [
        vec![(-60.0, 10.0)],
        vec![(-60.0, 10.0), (12.0, 3.0)],
        vec![(-60.0, 5.0), (8.0, 2.0), (20.0, 0.0), (30.0, 6.0)],
        vec![(-60.0, 0.0)],
    ];
    let mut n = 0;
    for flow in &flows {
        for x_o2 in [0.10, 0.164, 0.19, 0.21] {
            for (cutoff, last) in [(0.0, None), (10.0, Some(10)), (40.0, Some(35))] {
                let times = grid(cutoff, cutoff + 120.0, 2.5);
                let inp = burn(cutoff, x_o2, flow.clone(), last, times);
                let a = run1(&d1, &inp);
                let b = full(run5(&carry, &front5::FrontInput::from_first(inp)));
                let what = format!("O2 {x_o2}, flow {flow:?}, cutoff {cutoff}");
                assert_same_tip(&a, &b, &what);
                // certain ignition, no quench: exactly with a flame in the prefix, and to the
                // rounding of the sum of the node weights (1e-15) without one
                let bd = &b.boundary;
                let tol = if last.is_some() { 0.0 } else { 1e-15 };
                assert_eq!(bd.p_ignites, 1.0, "{what}");
                assert!(bd.p_alive.iter().all(|p| 1.0 - p <= tol), "{what}");
                assert!(b.p_quenched.iter().all(|p| *p <= tol), "{what}");
                n += 1;
            }
        }
    }
    assert_eq!(n, 48);
}

#[test]
fn the_carry_of_d1_forecasts_the_train_tracks_as_d1_does_bit_for_bit() {
    let [base1, d1, base5, carry] = committed();
    let mut cases = 0usize;
    for name in ["T5", "T6", "T16", "T10"] {
        let track = track_of(train_value(name));
        assert_eq!(track.partition, "train", "{name}");
        let stills = front_tools::flame_stills(&track);
        for cutoff in CUTOFFS {
            let mut times: Vec<f64> = stills
                .iter()
                .filter(|s| s.0 > cutoff && s.0 <= cutoff + 60.0)
                .map(|s| s.0)
                .collect();
            times.extend(HORIZONS.iter().map(|h| cutoff + h));
            times.extend(grid(cutoff, cutoff + 240.0, 7.5));
            times.sort_by(f64::total_cmp);
            times.dedup();
            let inp = front_tools::input_of(&track, cutoff, times.clone()).expect("input");
            // the fifth reader gives the same burn, plus the defaults of the new inputs
            let inp5 = front5_tools::input_of(&track, cutoff, times).expect("input5");
            assert_eq!(
                (inp5.p_atm, inp5.direction, inp5.material),
                (1.0, Direction::Concurrent, 0)
            );
            for (first, fifth, who) in [(&d1, &carry, "D1"), (&base1, &base5, "persistence")] {
                let b = full(run5(fifth, &inp5));
                assert_same_tip(
                    &run1(first, &inp),
                    &b,
                    &format!("{name} at {cutoff} s, {who}"),
                );
                assert!(b.boundary.p_alive.iter().all(|p| *p == 1.0));
                cases += 1;
            }
        }
        // the ignition forecast of the scorer: cutoff 0, no stills; the neutral boundary lights
        // every sample and keeps it lit
        let ign = front5_tools::ignition_input(&track, 20.0).expect("ignition input");
        let o = run5(&carry, &ign);
        assert!(o.p_quenched()[0] <= 1e-15, "{name}");
        assert_eq!(o.boundary().p_ignites, 1.0, "{name}");
    }
    assert_eq!(cases, 32);
}

// ----------------------------------------------------------------------------------------- the law

#[test]
fn the_law_fixture_is_the_dossiers_section_6_4_verbatim() {
    let dossier = repo("docs/wsp/research/phys_quench_w0.md").replace("\r\n", "\n");
    let section = dossier
        .split("### 6.4 Literal assignments")
        .nth(1)
        .expect("section 6.4");
    assert!(
        section.contains(LAW_6_4),
        "the test fixture is not the dossier's text"
    );
}

/// `(oxygen, speed index, printed value)` of the dossier's table.
type Points<'a> = &'a [(f64, usize, &'a str)];

fn check_table(
    law: &LoadedRelease,
    cond: impl Fn(f64) -> Condition,
    points: Points,
    p_of: impl Fn(&front5::Limit, f64) -> f64,
    table: &str,
) {
    for &(x, j, want) in points {
        let (l, p_ign) = front5::limit_of(&front5::spec(), law, &cond(x), SPEEDS[j], 8.0)
            .unwrap()
            .unwrap();
        let got = format!("{:.2}", p_of(&l, p_ign));
        assert_eq!(got, want, "{table}: X = {x}, U = {} cm/s", SPEEDS[j]);
    }
}

#[test]
fn the_law_reproduces_the_dossiers_tables_at_their_printed_precision() {
    let [_, _, _, carry] = committed();
    let law = with_law(&carry, "");
    // T1, SIBAL 2.2 cm, concurrent, 1 atm, P(sustained): ten grid points
    let t1: Points = &[
        (0.23, 1, "0.42"),
        (0.19, 2, "0.42"),
        (0.19, 3, "0.76"),
        (0.18, 4, "0.81"),
        (0.17, 6, "0.68"),
        (0.16, 7, "0.30"),
        (0.15, 6, "0.09"),
        (0.20, 10, "0.87"),
        (0.18, 9, "0.65"),
        (0.17, 0, "0.00"),
    ];
    check_table(&law, sibal, t1, |l, _| l.p_sus, "T1");
    // and its last two rows, the 50 % limit and its sd in percent of oxygen, at every speed
    let x_lim = [
        "73.9", "23.6", "19.4", "18.0", "17.0", "16.6", "16.5", "16.6", "16.8", "17.5", "18.2",
    ];
    let sigma = [
        "19.6", "3.1", "1.8", "1.4", "1.2", "1.1", "1.1", "1.1", "1.2", "1.3", "1.6",
    ];
    for (j, u) in SPEEDS.iter().enumerate() {
        let (l, _) = front5::limit_of(&front5::spec(), &law, &sibal(0.19), *u, 8.0)
            .unwrap()
            .unwrap();
        assert_eq!(
            format!("{:.1}", 100.0 * l.x_lim),
            x_lim[j],
            "T1 X_lim at {u}"
        );
        assert_eq!(format!("{:.1}", 100.0 * l.s), sigma[j], "T1 sigma at {u}");
    }
    // T2, opposed
    let opposed = |x| Condition {
        direction: Direction::Opposed,
        ..sibal(x)
    };
    let t2: Points = &[
        (0.23, 1, "0.96"),
        (0.19, 2, "0.84"),
        (0.17, 3, "0.50"),
        (0.20, 9, "0.53"),
        (0.18, 10, "0.13"),
        (0.23, 0, "0.02"),
    ];
    check_table(&law, opposed, t2, |l, _| l.p_sus, "T2");
    // T5, PMMA sheet 1 mm, 2.0 cm wide (width ratio 2.0 / 2.2 of the reference), concurrent
    let pmma = |x| Condition {
        material: 1,
        width_cm: 2.0,
        ..sibal(x)
    };
    let t5: Points = &[
        (0.17, 6, "0.86"),
        (0.15, 4, "0.32"),
        (0.19, 1, "0.17"),
        (0.14, 10, "0.08"),
    ];
    check_table(&law, pmma, t5, |l, _| l.p_sus, "T5");
    // T9, SIBAL 2.2 cm, concurrent, P(ignition) with the igniter on 8 s
    let t9: Points = &[
        (0.21, 2, "0.74"),
        (0.18, 6, "0.82"),
        (0.17, 4, "0.38"),
        (0.20, 10, "0.80"),
        (0.19, 3, "0.66"),
        (0.16, 6, "0.23"),
    ];
    check_table(&law, sibal, t9, |_, p| p, "T9");
}

/// The joint of ignition and survival at constant conditions with the boundary error shared
/// (section 6.3 and remark 4 of section 2): the integral over the standard normal e of
/// lit(e) 1[X - X_lim - s e > 0], by a fine midpoint rule on [-8, 8] (test arithmetic, not the
/// forward's nodes).
fn joint_exact(law: &LoadedRelease, x: f64, u: f64, dx: f64, sd_a: f64, tf: f64) -> f64 {
    let (l, _) = front5::limit_of(&front5::spec(), law, &sibal(x), u, 8.0)
        .unwrap()
        .unwrap();
    let n = 20_000;
    let h = 16.0 / n as f64;
    (0..n)
        .map(|i| {
            let e = -8.0 + h * (i as f64 + 0.5);
            if x - l.x_lim - l.s * e > 0.0 {
                let lit = ignis_wsp::quad::normal_cdf((x - l.x_lim - dx - l.s * e) / sd_a) * tf;
                lit * (-0.5 * e * e).exp() / (2.0 * std::f64::consts::PI).sqrt() * h
            } else {
                0.0
            }
        })
        .sum()
}

fn constant_burn(x_o2: f64, u: f64, times: Vec<f64>) -> front5::FrontInput {
    front5::FrontInput {
        cutoff_s: 0.0,
        igniter_on_s: -8.0,
        x_o2,
        width_ratio: 1.0,
        window_w: Some(4.5),
        flow: vec![(-60.0, u)],
        tip: vec![],
        base: vec![],
        times,
        p_atm: 1.0,
        direction: Direction::Concurrent,
        material: 0,
    }
}

/// Ten grid points of the dossier's tables across the boundary of the 2.2 cm fabric in concurrent
/// flow: `(oxygen, duct speed cm/s)`, P_sus from 0.09 to 0.99.
const ACROSS: [(f64, f64); 10] = [
    (0.15, 10.0),
    (0.16, 10.0),
    (0.17, 10.0),
    (0.18, 10.0),
    (0.19, 10.0),
    (0.17, 5.0),
    (0.18, 5.0),
    (0.19, 3.0),
    (0.21, 2.0),
    (0.17, 15.0),
];

#[test]
fn the_nodes_add_up_and_alive_never_rises() {
    let [_, _, _, carry] = committed();
    let law = with_law(&carry, "");
    let times = grid(0.0, 600.0, 10.0);
    for (x, u) in ACROSS {
        let o = run5(&law, &constant_burn(x, u, times.clone()));
        let b = o.boundary();
        // alive and quenched add up to the nodes' ignition at every time; alive never rises
        for j in 0..times.len() {
            assert!((b.p_alive[j] + b.p_quenched_by[j] - b.p_lit_nodes).abs() < 1e-12);
            assert!((o.p_quenched()[j] - (1.0 - b.p_alive[j])).abs() < 1e-15);
        }
        assert!(b.p_alive.windows(2).all(|w| w[1] <= w[0]));
        // the nodes' ignition is the closed form of P_ign to the resolution of 41 nodes
        assert!(
            (b.p_lit_nodes - b.p_ignites).abs() < 1.0 / 82.0,
            "{x} {u}: {} {}",
            b.p_lit_nodes,
            b.p_ignites
        );
    }
}

#[test]
fn at_constant_conditions_and_long_times_the_mixture_is_the_joint_of_ignition_and_survival() {
    let [_, _, _, carry] = committed();
    // the 6.4 fixture with ext.width at its lower bound (0.0005), 600 s after igniter-off
    let law = revise("bass_front5", &with_law(&carry, ""), "ext.width = 0.0005\n");
    let tf = ignis_wsp::quad::normal_cdf((8.0f64 / 2.0).ln() / 0.5);
    println!("  X     U    P_alive(600 s)  joint   P_ign   P_sus   P_ign x P_sus");
    for (x, u) in ACROSS {
        let o = run5(&law, &constant_burn(x, u, vec![600.0]));
        let b = o.boundary();
        let p = b.p_alive[0];
        let joint = joint_exact(&law, x, u, 0.004, 0.005, tf);
        let p_sus = b.at_ignition.p_sus;
        println!(
            "  {x:.2} {u:>5.1}   {p:.4}          {joint:.4}  {:.4}  {p_sus:.4}  {:.4}",
            b.p_ignites,
            b.p_ignites * p_sus
        );
        // ignition and survival share the boundary error: the long-time value is their joint,
        // not the product of the two marginals (the dossier's remark 4 of section 2)
        assert!((p - joint).abs() < 0.02, "{x} {u}: {p} against {joint}");
        assert!(p <= b.p_ignites.min(p_sus) + 0.02, "{x} {u}");
    }
    // with dx = 0 and sd_a at its lower bound, ignition is the same event as survival at the flow
    // of ignition, and the long-time value is P_sus times the igniter's time factor, less the
    // nodes within about three switch widths of the limit, which still go out by 600 s (the
    // smoothing of ext.width, not the nodes: up to 0.051 here, at 17 % and 15 cm/s)
    let pure = revise(
        "bass_front5",
        &law,
        "ign.dx[0:2] = [0.0, 0.0]\nign.sd_a = 0.001\n",
    );
    for (x, u) in ACROSS {
        let o = run5(&pure, &constant_burn(x, u, vec![600.0]));
        let b = o.boundary();
        let want = b.at_ignition.p_sus * tf;
        assert!(
            b.p_alive[0] <= want + 1.0 / 82.0 && b.p_alive[0] > want - 0.06,
            "{x} {u}: {} against P_sus x tf = {want}",
            b.p_alive[0]
        );
    }
    // with the dossier's own switch width (0.002) a node whose margin is under two widths keeps
    // going out after the flame should have settled: at 18 % and 10 cm/s the value at 600 s is
    // below the joint (the smoothing, recorded here; the author types a smaller ext.width)
    let wide = with_law(&carry, "");
    let o = run5(&wide, &constant_burn(0.18, 10.0, vec![60.0, 600.0]));
    let a = &o.boundary().p_alive;
    let joint = joint_exact(&wide, 0.18, 10.0, 0.004, 0.005, tf);
    assert!(a[1] < a[0] && a[1] < joint - 0.03, "{a:?} {joint}");
}

#[test]
fn a_flow_step_down_and_back_up_keeps_the_same_node() {
    let [_, _, _, carry] = committed();
    let law = with_law(&carry, "");
    // 18 % oxygen, 10 cm/s, down to 3 cm/s from 30 to 60 s, back to 10 cm/s
    let step = vec![(-60.0, 10.0), (30.0, 3.0), (60.0, 10.0)];
    let times = vec![0.0, 30.0, 60.0, 90.0];
    let mut inp = constant_burn(0.18, 10.0, times.clone());
    inp.flow = step.clone();
    let o = run5(&law, &inp);
    let a = &o.boundary().p_alive;
    // survival over thirty seconds at 10 cm/s, first and after the low phase: the flames that came
    // through 3 cm/s are those whose boundary lies low, and the same node carries that back to
    // 10 cm/s, where they go out less. A fresh draw of the node at each time, or one hazard of the
    // flow, would give the two windows the same survival.
    let (s1, s3) = (a[1] / a[0], a[3] / a[2]);
    assert!(a[2] < a[1], "the low phase quenches some flames: {a:?}");
    assert!(
        s3 > s1 + 0.01,
        "first window {s1}, after the low phase {s3}"
    );
    assert!(s3 <= 1.0);
    // the node weights a flame seen at 62 s carries forward: the low phase moved them to the
    // nodes where the boundary lies low (negative e_k)
    let mut seen = inp.clone();
    seen.cutoff_s = 62.0;
    seen.times = vec![62.0, 92.0];
    seen.tip = vec![(62.0, 1.0)];
    let o = run5(&law, &seen);
    let b = o.boundary();
    assert_eq!(b.flame_seen_at_s, Some(62.0));
    let mean_e: f64 = (0..front5::NODES)
        .map(|k| b.node_weights[k] * front5::Q41_Z[k])
        .sum();
    assert!(
        (b.node_weights.iter().sum::<f64>() - 1.0).abs() < 1e-12,
        "{:?}",
        b.node_weights
    );
    assert!(mean_e < -0.2, "mean node {mean_e}");
    assert_eq!(b.p_alive[0], 1.0);
    // and over the next thirty seconds at 10 cm/s that flame survives as the mixture says
    assert!(
        (b.p_alive[1] - s3).abs() < 0.02,
        "{} against {s3}",
        b.p_alive[1]
    );
}

// ---------------------------------------------------------------------------- inputs and refusals

#[test]
fn the_three_inputs_default_to_one_atmosphere_concurrent_flow_and_the_fabric() {
    let [_, _, _, carry] = committed();
    let law = with_law(&carry, "");
    let mut v = train_value("T16");
    let plain = front5_tools::input_of(&track_of(v.clone()), 20.0, vec![30.0, 60.0]).unwrap();
    v["conditions"]["pressure_atm"] = json!(1.0);
    v["conditions"]["flow"] = json!("Concurrent");
    v["conditions"]["material_index"] = json!(0);
    let stated = front5_tools::input_of(&track_of(v.clone()), 20.0, vec![30.0, 60.0]).unwrap();
    let (a, b) = (full(run5(&law, &plain)), full(run5(&law, &stated)));
    assert_eq!(bits(&a.p_quenched), bits(&b.p_quenched));
    assert_eq!(bits(&a.quantiles[2]), bits(&b.quantiles[2]));
    // half an atmosphere raises the limit by cp ln 2 and lowers survival
    v["conditions"]["pressure_atm"] = json!(0.5);
    let half = front5_tools::input_of(&track_of(v.clone()), 0.0, vec![30.0]).unwrap();
    let mut one = half.clone();
    one.p_atm = 1.0;
    one.tip.clear();
    let mut half = half;
    half.tip.clear();
    let (bh, b1) = (run5(&law, &half), run5(&law, &one));
    let dx = bh.boundary().at_ignition.x_lim - b1.boundary().at_ignition.x_lim;
    let (l1, _) = front5::limit_of(
        &front5::spec(),
        &law,
        &sibal(0.176),
        b1.boundary().at_ignition.u_cm_s,
        8.0,
    )
    .unwrap()
    .unwrap();
    let a_term = l1.a / l1.u_e_cm_s; // A / (p U_e) at 1 atm, doubled at 0.5 atm
    assert!(
        (dx - (0.020 * 2f64.ln() + a_term)).abs() < 1e-12,
        "{dx} against {}",
        0.020 * 2f64.ln() + a_term
    );
    assert!(bh.boundary().p_alive[0] < b1.boundary().p_alive[0]);
    // the shared prefix: pressure in Pa, flow_direction and material as numbers, absent = default
    let measured = |v: f64| Measured {
        raw_value: None,
        raw_unit: None,
        value_si: Some(v),
        unit_si: None,
        uncertainty_si: None,
        uncertainty_reason: None,
        evidence_class: EvidenceClass::Synthetic,
        locator: None,
        derivation: None,
        resolution: Resolution::Resolved,
        flags: vec![],
    };
    let prefix = front_tools::fixture_prefix(10.0, 0.001);
    let mut stated = prefix.clone();
    for (k, v) in [
        ("pressure", 101_325.0),
        ("flow_direction", 0.0),
        ("material", 0.0),
    ] {
        stated.known_conditions.insert(k.into(), measured(v));
    }
    let model = models::by_key("bass_front5").unwrap();
    let (a, b) = (
        model.artifact(&law, &prefix, &Value::Null).unwrap(),
        model.artifact(&law, &stated, &Value::Null).unwrap(),
    );
    assert!(a.support.admitted && b.support.admitted);
    assert_eq!(a.quantiles, b.quantiles);
    assert_eq!(a.event_cdf, b.event_cdf);
    for (k, v) in [
        ("flow_direction", 2.0),
        ("material", 2.0),
        ("material", 0.5),
        ("pressure", f64::NAN),
    ] {
        let mut bad = prefix.clone();
        bad.known_conditions.insert(k.into(), measured(v));
        let r = model.artifact(&law, &bad, &Value::Null).unwrap();
        assert!(!r.support.admitted, "{k} = {v}");
        assert_eq!(r.support.gate, SupportGate::Condition);
        assert!(r.quantiles.is_empty() && r.event_cdf.is_empty());
    }
}

#[test]
fn the_track_reader_refuses_what_it_cannot_read_and_leaves_the_first_reader_unchanged() {
    for (field, value) in [
        ("pressure_atm", json!(0.0)),
        ("pressure_atm", json!(-1.0)),
        ("pressure_atm", json!("one")),
        ("flow", json!("sideways")),
        ("material_index", json!(2)),
        ("material_index", json!(-1)),
        ("sample_label", json!("2 cm PMMA")),
    ] {
        let mut v = train_value("T5");
        v["conditions"][field] = value.clone();
        assert!(
            front5_tools::input_of(&track_of(v), 10.0, vec![20.0]).is_err(),
            "{field} = {value}"
        );
    }
    // a PMMA track that says so is read; the first reader refuses it, as before
    let mut v = train_value("T5");
    v["conditions"]["sample_label"] = json!("2 cm PMMA");
    v["conditions"]["material_index"] = json!(1);
    let t = track_of(v);
    assert_eq!(front5_tools::extra_of(&t).unwrap().material, 1);
    assert!(front_tools::conditions_of(&t).is_err());
    // the fifth reader reads the train tracks as the first does
    for name in ["T5", "T6", "T16", "T10"] {
        let t = track_of(train_value(name));
        assert_eq!(
            front5_tools::conditions_of(&t).unwrap(),
            front_tools::conditions_of(&t).unwrap(),
            "{name}"
        );
    }
}

#[test]
fn an_opposed_flow_burn_gets_its_event_probabilities_and_a_refused_tip() {
    let [_, _, _, carry] = committed();
    let law = with_law(&carry, "");
    let mut v = train_value("T5");
    v["conditions"]["flow"] = json!("opposed");
    let t = track_of(v);
    // the first reader refuses the track, as it always has
    assert!(front_tools::conditions_of(&t).is_err());
    for (cutoff, times) in [(10.0, vec![20.0, 40.0, 70.0]), (0.0, vec![20.0])] {
        let inp = if cutoff > 0.0 {
            front5_tools::input_of(&t, cutoff, times.clone()).unwrap()
        } else {
            front5_tools::ignition_input(&t, 20.0).unwrap()
        };
        assert_eq!(inp.direction, Direction::Opposed);
        match run5(&law, &inp) {
            Outcome::Full(_) => panic!("the tip of an opposed flame must be refused"),
            Outcome::EventsOnly {
                times_s,
                p_quenched,
                boundary,
                tip_refusal,
            } => {
                assert_eq!(times_s, times);
                assert!(!tip_refusal.admitted);
                assert_eq!(tip_refusal.gate, SupportGate::Condition);
                assert!(tip_refusal.reasons[0].contains("concurrent"));
                assert!(tip_refusal.reasons[0].contains("opposed"));
                assert_eq!(boundary.direction, Direction::Opposed);
                // the opposed row of the law: at 10.9 cm/s the opposed limit is above the
                // concurrent one (T2 against T1)
                assert!(boundary.at_ignition.x_lim > 0.165);
                for (q, a) in p_quenched.iter().zip(&boundary.p_alive) {
                    assert!((0.0..=1.0).contains(q) && (q + a - 1.0).abs() < 1e-15);
                }
                assert!((0.0..=1.0).contains(&boundary.p_ignites));
            }
        }
    }
    // the same on the metric route: an admitted artifact with the event rows, no quantiles, and
    // the tip's refusal in its diagnostics
    let measured = Measured {
        raw_value: None,
        raw_unit: None,
        value_si: Some(1.0),
        unit_si: None,
        uncertainty_si: None,
        uncertainty_reason: None,
        evidence_class: EvidenceClass::Synthetic,
        locator: None,
        derivation: None,
        resolution: Resolution::Resolved,
        flags: vec![],
    };
    let mut prefix = front_tools::fixture_prefix(10.0, 0.001);
    prefix
        .known_conditions
        .insert("flow_direction".into(), measured);
    let a = models::by_key("bass_front5")
        .unwrap()
        .artifact(&law, &prefix, &Value::Null)
        .unwrap();
    a.validate().unwrap();
    assert!(a.support.admitted);
    assert!(a.quantiles.is_empty() && a.mean.is_empty());
    assert_eq!(a.event_categories, ["active_supported", "extinction"]);
    assert!(a.diagnostics["tip_forecast_refused"]["reasons"][0]
        .as_str()
        .unwrap()
        .contains("concurrent"));
}

#[test]
fn a_material_outside_the_law_and_a_zero_effective_speed_are_refused() {
    let [_, _, _, carry] = committed();
    let law = with_law(&carry, "");
    let mut inp = constant_burn(0.19, 5.0, vec![10.0]);
    inp.material = 2;
    let r = front5::forecast_track(&front5::spec(), &law, &inp, &LEVELS).unwrap();
    assert_eq!(r.unwrap_err().gate, SupportGate::Condition);
    inp.material = 1;
    assert!(matches!(run5(&law, &inp), Outcome::EventsOnly { .. }));
    let mut inp = constant_burn(0.19, 5.0, vec![10.0]);
    inp.p_atm = 0.0;
    let r = front5::forecast_track(&front5::spec(), &law, &inp, &LEVELS).unwrap();
    assert_eq!(r.unwrap_err().gate, SupportGate::Condition);
    // no self-spread speed and the fan off: the limit is not defined, and the forward says so
    let still = revise("bass_front5", &law, "sus.us[0:2] = [0.0, 0.0]\n");
    let mut inp = constant_burn(0.19, 5.0, vec![10.0, 40.0]);
    inp.flow = vec![(-60.0, 5.0), (20.0, 0.0)];
    let r = front5::forecast_track(&front5::spec(), &still, &inp, &LEVELS).unwrap();
    let s = r.unwrap_err();
    assert_eq!(s.gate, SupportGate::Condition);
    assert!(s.reasons[0].contains("effective flow speed"));
}

/// Whether the committed root with `extra` replacing its statements of the same tensors is refused
/// by the assembler.
fn refused(extra: &str) -> bool {
    let name_of = |l: &str| {
        l.trim_start()
            .split(['[', ' ', '='])
            .next()
            .unwrap_or_default()
            .to_string()
    };
    let mut src = String::new();
    for line in FRONT5_BASE.lines() {
        let name = name_of(line);
        if name.is_empty() || !extra.lines().any(|l| name_of(l) == name) {
            src.push_str(line);
            src.push('\n');
        }
    }
    src.push_str(extra);
    let spec = front5::spec();
    match text::compile(&src, &spec) {
        Err(_) => true,
        Ok(ops) => {
            let patch = assemble::make_patch(&spec, common::meta("test-front5-bounds"), ops);
            assemble::materialize(&spec, &patch, None).is_err()
        }
    }
}

#[test]
fn the_new_tensors_are_held_to_the_bounds_of_section_6_2() {
    assert!(!refused(""));
    assert!(!refused(LAW_6_4));
    for good in [
        "sus.x0[0:2] = [0.30, 0.08]",
        "sus.aw[0:2] = [-0.2, 0.3]",
        "ign.t_req[0:2] = [0.1, 120.0]",
        "ext.width = 0.01",
    ] {
        assert!(!refused(good), "{good}");
    }
    for bad in [
        "sus.x0[0:2] = [0.07, 0.10]",
        "sus.x0[0:2] = [0.10, 0.31]",
        "sus.cp[0:2] = [-0.01, 0.0]",
        "sus.sb[0:2, 0:2] = [[0.011, 0.0], [0.0, 0.0]]",
        "sus.ar[0:2, 0:2] = [[0.0, 0.0], [0.0, 0.6]]",
        "sus.aw[0:2] = [-0.21, 0.0]",
        "sus.us[0:2] = [5.1, 0.0]",
        "sus.sd_x0[0:2] = [0.0, 0.01]",
        "sus.sd_aw[0:2] = [0.31, 0.0]",
        "ign.dx[0:2] = [0.06, 0.0]",
        "ign.sd_a = 0.0",
        "ign.t_req[0:2] = [0.05, 1.0]",
        "ign.sd_lnt = 2.5",
        "ext.tau[0:2, 0:2] = [[0.05, 1.0], [1.0, 1.0]]",
        "ext.width = 0.0001",
        "quad.w[0] = 0.5",
        "quench.b0 = -40.0",
    ] {
        assert!(refused(bad), "{bad}");
    }
}

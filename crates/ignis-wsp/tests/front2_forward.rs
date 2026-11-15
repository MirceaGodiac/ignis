//! The second flame-front forward of `bass_front.wsp2` (`front2.rs`): with its four additions off it
//! is the first front forward bit for bit, on the committed carry of D1 against D1 itself (the four
//! train tracks at every scoring cutoff, the ignition forecast, fixture burns, the metric route) and
//! on test-only weights; then each addition alone, with test-only weights: two oxygen powers of the
//! spread rate, a slow growth and a fast shrinking of the overhang, a slow-down downstream, and the
//! mixture of flow paths; the committed spec, the construction checks and the bounds of the new
//! entries. Test-only weights are synthetic and are no candidate; nothing here scores a release.

mod common;

use ignis_core::contracts::{CheckStatus, ModelSpec, ReleaseId};
use ignis_core::hashing::content_hash;
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::front::{self, FrontInput};
use ignis_wsp::front_tools::{self, rules, Track};
use ignis_wsp::quad::normal_cdf;
use ignis_wsp::{assemble, audit, front2, models, text};
use serde_json::{json, Value};

const FRONT_BASE: &str =
    include_str!("../../../configs/models/bass/front/baseline-persistence/edits.wsp");
const FRONT_D1: &str = include_str!("../../../configs/models/bass/front/d1-fable/edits.wsp");
const FRONT2_BASE: &str =
    include_str!("../../../configs/models/bass/front2/baseline-persistence/edits.wsp");
const FRONT2_CARRY: &str = include_str!("../../../configs/models/bass/front2/carry-d1/edits.wsp");

/// Test-only weights of the first front forward, not a candidate: a constant spread rate of 0.1
/// sample widths per second, no overhang, a burn-through time of 5 s, no quench, the igniter
/// piloting at the law's rate from the moment it is switched on, an encoder gain of about 2e-9, no
/// spread at all.
const TOY: &str = "
speed.b0 = 0.1
speed.skip[0:3] = 0.0
speed.pow[0:2] = 0.0
over.b0 = 0.0
over.skip[0:3] = 0.0
over.pow[0:2] = 0.0
over.tau_s = 1.0
burn.b0 = 1.6094379124341003
burn.skip[0:3] = 0.0
quench.b0 = -40.0
quench.skip[0:3] = 0.0
ign.gain = 1.0
ign.delay_s = 0.0
end.linger_s = 0.0
enc.gain_w[0:2] = 0.0
enc.gain_b = -20.0
enc.mix[0:2] = [1.0, 0.0]
enc.e_clip[0:2] = 1.0
unc.s_pop = 0.0
unc.s_floor = 0.0
unc.s_far[0:2] = 0.0
unc.sigma_w = 0.0
unc.sigma_x0_w = 0.0
unc.rel_over = 0.0
unc.drift_w = 0.0
";

/// The entries of the second front forward, every addition off.
const OFF: &str = "speed.split = 0.0
speed.pow_i = 0.0
speed.kappa = 0.0
over.tau_up_s = 0.0
unc.flow_abs_cm_s = 0.0
unc.flow_rel = 0.0
unc.flow_w[0:3] = [0.0, 1.0, 0.0]
";

const LEVELS: [f64; 5] = [0.05, 0.25, 0.5, 0.75, 0.95];
const MED: usize = 2;
const CUTOFFS: [f64; 4] = [5.0, 10.0, 20.0, 40.0];
const HORIZONS: [f64; 5] = [10.0, 20.0, 30.0, 45.0, 60.0];

fn repo(path: &str) -> String {
    let path = format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// `base` with `extra` statements replacing the base statements of the tensors they name.
fn source(base: &str, extra: &str) -> String {
    let name_of = |l: &str| {
        l.trim_start()
            .split(['[', ' ', '='])
            .next()
            .unwrap_or_default()
            .to_string()
    };
    let mut src = String::new();
    for line in base.lines() {
        let name = name_of(line);
        if name.is_empty() || !extra.lines().any(|l| name_of(l) == name) {
            src.push_str(line);
            src.push('\n');
        }
    }
    src.push_str(extra);
    src
}

/// An in-memory release, with the content id the registry would give it (a revision names it).
fn loaded(mut release: ignis_core::contracts::ModelRelease, params: TensorMap) -> LoadedRelease {
    release.id = content_hash(&release, &["id", "created_at"]);
    LoadedRelease {
        release,
        params,
        posterior: TensorMap::new(),
    }
}

/// The root release of a model from complete edits, built in memory.
fn root(key: &str, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key}: {e}\n{src}"));
    let patch = assemble::make_patch(&spec, common::meta("test-front2"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("release");
    loaded(release, tensors)
}

/// A literal revision of `parent` by the edits `src`, built in memory.
fn revise(key: &str, parent: &LoadedRelease, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let mut meta = common::meta("test-front2-revision");
    meta.parent_release_id = Some(ReleaseId::new(parent.release.id.clone()).unwrap());
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key} revision: {e}"));
    let patch = assemble::make_patch(&spec, meta, ops);
    let tensors = assemble::materialize(&spec, &patch, Some(parent)).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, Some(parent), &common::binding())
        .expect("revision release");
    loaded(release, tensors)
}

/// The toy weights on the second front forward, the additions off unless `extra` says otherwise.
fn toy2(extra: &str) -> LoadedRelease {
    root("bass_front2", &source(&format!("{TOY}{OFF}"), extra))
}

/// The toy weights on the first front forward.
fn toy1(extra: &str) -> LoadedRelease {
    root("bass_front", &source(TOY, extra))
}

/// The committed D1 of `bass_front.wsp` and the committed carry of D1 on `bass_front.wsp2`, with
/// the baselines they revise.
fn committed() -> [LoadedRelease; 4] {
    let base1 = root("bass_front", FRONT_BASE);
    let d1 = revise("bass_front", &base1, FRONT_D1);
    let base2 = root("bass_front2", FRONT2_BASE);
    let carry = revise("bass_front2", &base2, FRONT2_CARRY);
    [base1, d1, base2, carry]
}

fn run1(rel: &LoadedRelease, input: &FrontInput) -> front::Forecast {
    front::forecast_track(&front::spec(), rel, input, &LEVELS)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn run2(rel: &LoadedRelease, input: &FrontInput) -> front2::Forecast {
    front2::forecast_track(&front2::spec(), rel, &input.clone().into(), &LEVELS)
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

/// A burn of the reference sample, igniter on from -6 s, with one still per second from 0 to
/// `last_still_s` (none when `None`) whose tip and base follow `tip` and `base`.
fn burn(
    cutoff_s: f64,
    x_o2: f64,
    flow: Vec<(f64, f64)>,
    last_still_s: Option<usize>,
    tip: impl Fn(f64) -> f64,
    base: impl Fn(f64) -> f64,
    times: Vec<f64>,
) -> FrontInput {
    let ts: Vec<f64> = last_still_s
        .map(|n| (0..=n).map(|i| i as f64).collect())
        .unwrap_or_default();
    FrontInput {
        cutoff_s,
        igniter_on_s: -6.0,
        x_o2,
        width_ratio: 1.0,
        window_w: None,
        flow,
        tip: ts.iter().map(|t| (*t, tip(*t))).collect(),
        base: ts.iter().map(|t| (*t, base(*t))).collect(),
        times,
    }
}

fn grid(from: f64, to: f64, step: f64) -> Vec<f64> {
    let n = ((to - from) / step).round() as usize;
    (0..=n).map(|i| from + step * i as f64).collect()
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// The two forwards returned the same numbers, bit for bit: tip quantiles and mean, base, event
/// probabilities, and what the prefix said.
fn assert_same(a: &front::Forecast, b: &front2::Forecast, what: &str) {
    assert_eq!(bits(&a.times_s), bits(&b.times_s), "{what}: times");
    assert_eq!(a.quantiles.len(), b.quantiles.len(), "{what}: levels");
    for (l, (qa, qb)) in a.quantiles.iter().zip(&b.quantiles).enumerate() {
        assert_eq!(bits(qa), bits(qb), "{what}: tip quantile {}", LEVELS[l]);
    }
    assert_eq!(bits(&a.mean), bits(&b.mean), "{what}: mean");
    assert_eq!(bits(&a.base_median), bits(&b.base_median), "{what}: base");
    assert_eq!(bits(&a.p_quenched), bits(&b.p_quenched), "{what}: quenched");
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
            ea.laws_at_cutoff.quench
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
            eb.laws_at_cutoff.quench
        ]),
        "{what}: encoder"
    );
    assert!(b.paths.is_empty(), "{what}: one run on the flow as read");
    assert!(
        b.trace.flow_factor.iter().all(|f| *f == 1.0),
        "{what}: no slow-down"
    );
}

fn train_track(name: &str) -> Track {
    serde_json::from_str(&repo(&format!(
        "tools/wsp/bass_optical/tracks/train/{name}.json"
    )))
    .unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn the_committed_spec_is_the_first_front_spec_with_seven_entries_after_its_authored_ones() {
    let spec = front2::spec();
    spec.validate().expect("spec valid");
    assert_eq!(
        models::by_key("bass_front2").unwrap().spec().spec_hash(),
        spec.spec_hash()
    );
    let on_disk: ModelSpec =
        serde_json::from_str(&repo("configs/models/bass/front2/spec.json")).unwrap();
    assert_eq!(
        on_disk.spec_hash(),
        spec.spec_hash(),
        "configs/models/bass/front2/spec.json drifted from front2::spec(); a changed spec is a new \
         spec id (re-export with `ignis wsp spec --family bass_front2 --out …`), never an in-place edit"
    );
    assert_eq!(spec.spec_id.as_str(), "bass_front-wsp-v2");
    assert_eq!(spec.model_name, "bass_front.wsp2");
    assert_eq!(spec.forward.entry_point, "ignis_wsp::front2::forecast");
    assert_eq!(spec.forward.version, "2");
    assert_eq!(
        models::by_name("bass_front.wsp2").map(|m| m.key),
        Some("bass_front2")
    );
    // every tensor of the first front spec, unchanged and in its order, and seven new authored
    // tensors in the stated order after the last authored one
    let first = front::spec();
    let new = [
        ("speed.split", vec![], (0.0, 1.0)),
        ("speed.pow_i", vec![], (0.0, 6.0)),
        ("speed.kappa", vec![], (0.0, 1.0)),
        ("over.tau_up_s", vec![], (0.0, 120.0)),
        ("unc.flow_abs_cm_s", vec![], (0.0, 5.0)),
        ("unc.flow_rel", vec![], (0.0, 1.0)),
        ("unc.flow_w", vec![3u32], (0.0, 1000.0)),
    ];
    let at = first
        .tensors
        .iter()
        .position(|t| t.name == "unc.drift_w")
        .unwrap()
        + 1;
    assert_eq!(spec.tensors.len(), first.tensors.len() + new.len());
    assert_eq!(&spec.tensors[..at], &first.tensors[..at]);
    assert_eq!(&spec.tensors[at + new.len()..], &first.tensors[at..]);
    for (t, (name, shape, (lo, hi))) in spec.tensors[at..].iter().zip(&new) {
        assert_eq!(&t.name, name);
        assert_eq!(&t.shape, shape, "{name}");
        let b = t
            .bounds
            .as_ref()
            .unwrap_or_else(|| panic!("{name}: bounds"));
        assert_eq!((b.min.get(), b.max.get()), (*lo, *hi), "{name}");
        assert!(!t.description.is_empty());
    }
    assert_eq!(spec.features, first.features);
}

#[test]
fn the_forward_identity_is_the_shared_sources_and_front2_alone() {
    let names: Vec<&str> = audit::forward_sources_of("bass_front.wsp2")
        .expect("a literal model")
        .iter()
        .map(|s| s.0)
        .collect();
    assert_eq!(
        names,
        ["nn.rs", "quad.rs", "common.rs", "spec.rs", "front2.rs"]
    );
    assert_ne!(
        audit::forward_hash_of("bass_front.wsp2"),
        audit::forward_hash_of("bass_front.wsp")
    );
    assert_eq!(
        audit::banned_hits_of("bass_front.wsp2"),
        Vec::<String>::new()
    );
    assert_eq!(
        audit::strict_route_check_of("bass_front.wsp2").status,
        CheckStatus::Passed
    );
    // a release of one front forward is not run by the other
    let [_, d1, _, carry] = committed();
    let inp = burn(
        0.0,
        0.19,
        vec![(-60.0, 5.0)],
        None,
        |t| t,
        |t| t,
        vec![20.0],
    );
    assert!(front::forecast_track(&front::spec(), &carry, &inp, &LEVELS).is_err());
    assert!(front2::forecast_track(&front2::spec(), &d1, &inp.clone().into(), &LEVELS).is_err());
}

#[test]
fn the_committed_carry_holds_the_numbers_of_d1_and_the_new_entries_off() {
    let [base1, d1, base2, carry] = committed();
    for (first, second, dir1, dir2) in [
        (
            &base1,
            &base2,
            "front/baseline-persistence",
            "front2/baseline-persistence",
        ),
        (&d1, &carry, "front/d1-fable", "front2/carry-d1"),
    ] {
        // tensor by tensor: the first forward's numbers, unchanged
        for (name, t) in &first.params {
            assert_eq!(
                second.params.get(name).map(|x| bits(&x.data)),
                Some(bits(&t.data)),
                "{dir2}: {name}"
            );
        }
        // and nothing else but the new entries, switched off
        let mut extra: Vec<(&str, &[f64])> = second
            .params
            .iter()
            .filter(|(n, _)| !first.params.contains_key(*n))
            .map(|(n, t)| (n.as_str(), t.data.as_slice()))
            .collect();
        extra.sort_by(|a, b| a.0.cmp(b.0));
        let off: [(&str, &[f64]); 7] = [
            ("over.tau_up_s", &[0.0]),
            ("speed.kappa", &[0.0]),
            ("speed.pow_i", &[0.0]),
            ("speed.split", &[0.0]),
            ("unc.flow_abs_cm_s", &[0.0]),
            ("unc.flow_rel", &[0.0]),
            ("unc.flow_w", &[0.0, 1.0, 0.0]),
        ];
        assert_eq!(extra, off, "{dir2}");
        // the releases built here from the committed edits are the committed releases
        for (rel, dir) in [(first, dir1), (second, dir2)] {
            let on_disk: Value =
                serde_json::from_str(&repo(&format!("configs/models/bass/{dir}/release.json")))
                    .unwrap();
            let m = rel.release.construction.as_ref().unwrap();
            assert_eq!(
                on_disk["construction"]["weights_hash"],
                json!(m.weights_hash),
                "{dir}: weights"
            );
            assert_eq!(
                on_disk["construction"]["spec_hash"],
                json!(m.spec_hash),
                "{dir}: spec"
            );
            assert_eq!(
                on_disk["construction"]["forward_hash"],
                json!(m.forward_hash),
                "{dir}: the committed release was assembled with another forward; rebuild it \
                 (python tools/wsp/reassemble.py)"
            );
            assert_eq!(on_disk["model_name"], json!(rel.release.model_name));
            let checks = on_disk["construction"]["checks"].as_array().unwrap();
            assert_eq!(checks.len(), 5, "{dir}");
            assert!(
                checks.iter().all(|c| c["status"] == "passed"),
                "{dir}: {checks:?}"
            );
        }
    }
}

#[test]
fn with_the_additions_off_the_carry_of_d1_forecasts_the_train_tracks_as_d1_does() {
    let [base1, d1, base2, carry] = committed();
    let mut cases = 0usize;
    for name in ["T5", "T6", "T16", "T10"] {
        let track = train_track(name);
        assert_eq!(track.partition, "train", "{name}");
        let stills = front_tools::flame_stills(&track);
        for cutoff in CUTOFFS {
            // the times a case is scored at: its target stills and the event horizons; and a grid
            let mut times: Vec<f64> = stills
                .iter()
                .filter(|s| s.0 > cutoff && s.0 <= cutoff + 60.0)
                .map(|s| s.0)
                .collect();
            times.extend(HORIZONS.iter().map(|h| cutoff + h));
            times.extend(grid(cutoff, cutoff + 240.0, 7.5));
            times.sort_by(f64::total_cmp);
            times.dedup();
            let inp = front_tools::input_of(&track, cutoff, times).expect("input");
            for (first, second, who) in [(&d1, &carry, "D1"), (&base1, &base2, "persistence")] {
                assert_same(
                    &run1(first, &inp),
                    &run2(second, &inp),
                    &format!("{name} at {cutoff} s, {who}"),
                );
                cases += 1;
            }
        }
        // the ignition outcome: cutoff 0, no stills
        let mut inp = front_tools::input_of(&track, 0.0, vec![20.0]).expect("input");
        inp.tip.clear();
        inp.base.clear();
        assert_same(
            &run1(&d1, &inp),
            &run2(&carry, &inp),
            &format!("{name}, ignition"),
        );
    }
    assert_eq!(cases, 32);
}

#[test]
fn with_the_additions_off_fixture_burns_and_the_metric_route_are_those_of_the_first_forward() {
    let [base1, d1, base2, carry] = committed();
    let law = |t: f64| 0.05 * (t + 6.0) + 0.3;
    let mut fixtures = vec![
        // a step down, a step up, a slow ramp, the fan stopped, a short prefix, no prefix
        burn(
            15.0,
            0.19,
            vec![(-60.0, 10.0), (20.0, 2.0)],
            Some(15),
            law,
            |t| law(t) - 0.6,
            grid(15.0, 75.0, 0.5),
        ),
        burn(
            20.0,
            0.21,
            vec![(-60.0, 3.0), (12.0, 11.0), (40.0, 25.0)],
            Some(20),
            law,
            |t| law(t) - 0.4,
            grid(20.0, 140.0, 1.3),
        ),
        burn(
            10.0,
            0.176,
            std::iter::once((-60.0, 5.0))
                .chain((5..=60).map(|t| (t as f64, 5.0 + 0.3 * (t as f64 - 4.0))))
                .collect(),
            Some(10),
            law,
            |t| law(t) - 0.5,
            grid(10.0, 70.0, 0.7),
        ),
        burn(
            8.0,
            0.19,
            vec![(-60.0, 6.0), (10.0, 0.0)],
            Some(8),
            law,
            |t| law(t) - 0.5,
            vec![8.0, 9.9, 10.1, 12.0, 30.0],
        ),
        burn(
            2.0,
            0.23,
            vec![(-60.0, 45.0)],
            Some(2),
            law,
            law,
            vec![2.0, 600.0],
        ),
        burn(
            7.0,
            0.164,
            vec![(-60.0, 6.0)],
            None,
            law,
            law,
            vec![7.0, 20.0, 60.0],
        ),
    ];
    // the narrow sample, with the end of the sample in reach
    let mut narrow = fixtures[0].clone();
    narrow.width_ratio = rules::NARROW_RATIO;
    narrow.window_w = Some(2.0);
    fixtures.push(narrow);
    let mut windowed = fixtures[1].clone();
    windowed.window_w = Some(4.5);
    windowed.igniter_on_s = -9.67;
    fixtures.push(windowed);
    let toys = (
        toy1("speed.skip[0:3] = [0.05, 0.0, 0.0]\nover.b0 = 0.5\nover.skip[0:3] = [0.3, 0.0, 0.0]\nunc.sigma_w = 0.05\nunc.s_pop = 0.2\nunc.rel_over = 0.5"),
        toy2("speed.skip[0:3] = [0.05, 0.0, 0.0]\nover.b0 = 0.5\nover.skip[0:3] = [0.3, 0.0, 0.0]\nunc.sigma_w = 0.05\nunc.s_pop = 0.2\nunc.rel_over = 0.5"),
    );
    for (k, inp) in fixtures.iter().enumerate() {
        for (first, second, who) in [
            (&d1, &carry, "D1"),
            (&base1, &base2, "persistence"),
            (&toys.0, &toys.1, "toy"),
        ] {
            assert_same(
                &run1(first, inp),
                &run2(second, inp),
                &format!("fixture {k}, {who}"),
            );
        }
    }
    // entries that are not read while their addition is off change nothing
    let unread = toy2("speed.pow_i = 4.0\nunc.flow_abs_cm_s = 3.0\nunc.flow_rel = 0.5");
    assert_same(
        &run1(&toy1(""), &fixtures[0]),
        &run2(&unread, &fixtures[0]),
        "unread entries",
    );

    // the metric route on the synthetic fixture: the same artifact numbers
    for settings in [
        Value::Null,
        json!({"times_s": [20.0, 23.5, 50.0, 110.0, 400.0]}),
    ] {
        let p = front_tools::fixture_prefix(10.0, 0.001);
        let a = front::artifact(&front::spec(), &d1, &p, &settings).unwrap();
        let b = front2::artifact(&front2::spec(), &carry, &p, &settings).unwrap();
        b.validate().unwrap();
        assert!(a.support.admitted && b.support.admitted);
        assert_eq!(bits(&a.times_s), bits(&b.times_s));
        assert_eq!(a.quantile_levels, b.quantile_levels);
        for (qa, qb) in a.quantiles.iter().zip(&b.quantiles) {
            assert_eq!(bits(qa), bits(qb));
        }
        assert_eq!(bits(&a.mean), bits(&b.mean));
        assert_eq!(a.event_categories, b.event_categories);
        for (ra, rb) in a.event_cdf.iter().zip(&b.event_cdf) {
            assert_eq!(bits(ra), bits(rb));
        }
        for key in [
            "tip_quantiles_sample_widths",
            "base_median_m",
            "sample_width_m",
        ] {
            assert_eq!(a.diagnostics[key], b.diagnostics[key], "{key}");
        }
        assert_eq!(
            b.diagnostics["flow_paths"],
            json!("one run on the flow as read")
        );
    }
}

#[test]
fn the_zero_flow_part_of_the_spread_rate_follows_its_own_oxygen_power() {
    // N(u) = 0.1 + 0.05 u, u = U / 5 - 1: 0.15 at 10 cm/s; at zero flow the feature is that of
    // 0.1 cm/s, N0 = 0.1 + 0.05 (0.02 - 1) = 0.051. Oxygen powers: 3 at zero flow, 1.5 for the rest.
    let law = "speed.skip[0:3] = [0.05, 0.0, 0.0]\nspeed.pow[0:2] = [1.5, 0.0]\nspeed.pow_i = 3.0";
    let spec = front2::spec();
    let split = toy2(&format!("{law}\nspeed.split = 1.0"));
    let v = |rel: &LoadedRelease, u: f64, x: f64| front2::laws_of(&spec, rel, u, x, 1.0).unwrap();
    let (n0, n10) = (0.1 + 0.05 * (0.1 / 5.0 - 1.0), 0.15);
    for x in [0.164, 0.176, 0.19, 0.21, 0.23] {
        let o: f64 = x / 0.19;
        // at zero flow the rate is the zero-flow part alone and scales as o^3
        let still = v(&split, 0.0, x);
        assert!((still.v - n0 * o.powf(3.0)).abs() < 1e-15, "{x}");
        assert_eq!(still.v.to_bits(), still.v_zero_flow.to_bits());
        assert!((still.v / v(&split, 0.0, 0.19).v - o.powf(3.0)).abs() < 1e-13);
        // a flow below the smallest the laws are evaluated at is zero flow
        assert_eq!(v(&split, 0.05, x).v.to_bits(), still.v.to_bits());
        // the flow-dependent part scales as o^1.5
        for (u, n) in [(10.0, n10), (5.0, 0.1), (2.0, 0.07), (25.0, 0.3)] {
            let l = v(&split, u, x);
            let flow_part = l.v - l.v_zero_flow;
            assert!(
                (flow_part - (n - n0) * o.powf(1.5)).abs() < 1e-15,
                "{x} {u}"
            );
            assert_eq!(l.v_zero_flow.to_bits(), still.v.to_bits());
            let reference = v(&split, u, 0.19);
            assert!(
                (flow_part / (reference.v - reference.v_zero_flow) - o.powf(1.5)).abs() < 1e-12
            );
        }
    }
    // the switch off: one oxygen power for the whole rate, the number of the first forward
    let whole = toy2(law);
    let first = toy1("speed.skip[0:3] = [0.05, 0.0, 0.0]\nspeed.pow[0:2] = [1.5, 0.0]");
    for (u, x) in [(0.0, 0.21), (10.0, 0.21), (3.0, 0.17)] {
        let want = front::laws_of(&front::spec(), &first, u, x, 1.0).unwrap().v;
        assert_eq!(v(&whole, u, x).v.to_bits(), want.to_bits());
        // between: the weighted mean of the two laws
        let half = toy2(&format!("{law}\nspeed.split = 0.25"));
        let mixed = 0.75 * want + 0.25 * v(&split, u, x).v;
        assert!((v(&half, u, x).v - mixed).abs() < 1e-15);
    }
    // equal powers: the split law is the first law
    let equal = toy2(
        "speed.skip[0:3] = [0.05, 0.0, 0.0]\nspeed.pow[0:2] = [1.5, 0.0]\nspeed.pow_i = 1.5\nspeed.split = 1.0",
    );
    assert!((v(&equal, 10.0, 0.21).v - v(&whole, 10.0, 0.21).v).abs() < 1e-15);
    // the width power multiplies both parts
    let wide = toy2(
        "speed.skip[0:3] = [0.05, 0.0, 0.0]\nspeed.pow[0:2] = [1.5, -1.0]\nspeed.pow_i = 3.0\nspeed.split = 1.0",
    );
    let l = front2::laws_of(&spec, &wide, 10.0, 0.21, 0.5).unwrap();
    let o: f64 = 0.21 / 0.19;
    assert!((l.v_zero_flow - 2.0 * n0 * o.powf(3.0)).abs() < 1e-15);
    assert!((l.v - 2.0 * (n0 * o.powf(3.0) + (n10 - n0) * o.powf(1.5))).abs() < 1e-15);
    // a network that falls with the flow cannot take the rate below the smallest zero-flow rate
    let falling = toy2(
        "speed.skip[0:3] = [-0.05, 0.0, 0.0]\nspeed.pow[0:2] = [3.0, 0.0]\nspeed.pow_i = 0.0\nspeed.split = 1.0",
    );
    assert_eq!(v(&falling, 10.0, 0.23).v, 0.001);

    // the forecast moves at that rate: no stills, a steady flow, the tip is the law path
    for (u, x) in [(0.0, 0.21), (10.0, 0.21), (10.0, 0.17)] {
        let f = run2(
            &split,
            &burn(
                0.0,
                x,
                vec![(-60.0, u)],
                None,
                |t| t,
                |t| t,
                vec![0.0, 10.0, 40.0],
            ),
        );
        let rate = v(&split, u, x).v;
        assert!(((f.quantiles[MED][2] - f.quantiles[MED][0]) / 40.0 - rate).abs() < 1e-12);
        assert!((f.trace.speed_w_s[1] - rate).abs() < 1e-15);
        assert_eq!(
            f.trace.speed_zero_flow_w_s[1].to_bits(),
            v(&split, 0.0, x).v.to_bits()
        );
    }
}

#[test]
fn the_overhang_grows_with_its_slow_time_and_shrinks_with_its_fast_one() {
    // h_ss = 0.5 + 0.3 u: 0.32 at 2 cm/s, 0.8 at 10 cm/s. The fan steps up at 20 s and down at 50 s.
    // A growing overhang takes 4 s, a shrinking one 1 s.
    let over = "over.b0 = 0.5\nover.skip[0:3] = [0.3, 0.0, 0.0]\nover.tau_s = 1.0";
    let rel = toy2(&format!("{over}\nover.tau_up_s = 4.0"));
    let times = grid(0.0, 80.0, 0.25);
    let inp = burn(
        0.0,
        0.19,
        vec![(-60.0, 2.0), (20.0, 10.0), (50.0, 2.0)],
        None,
        |t| t,
        |t| t,
        times.clone(),
    );
    let f = run2(&rel, &inp);
    let at = |t: f64| times.iter().position(|s| *s == t).unwrap();
    let h = &f.trace.overhang_w;
    let (lo, hi) = (0.32, 0.8);
    let (h20, h50) = (h[at(20.0)], h[at(50.0)]);
    for (j, &t) in times.iter().enumerate() {
        let (want, target, tau) = if t <= 20.0 {
            // from nothing when the igniter is switched on, growing
            (lo * (1.0 - (-(t + 6.0) / 4.0).exp()), lo, 4.0)
        } else if t <= 50.0 {
            (hi + (h20 - hi) * (-(t - 20.0) / 4.0).exp(), hi, 4.0)
        } else {
            (lo + (h50 - lo) * (-(t - 50.0) / 1.0).exp(), lo, 1.0)
        };
        assert!(
            (h[j] - want).abs() < 1e-12,
            "overhang at {t}: {} vs {want}",
            h[j]
        );
        // the tip is the front plus the overhang
        assert!((f.quantiles[MED][j] - f.trace.front_w[j] - h[j]).abs() < 1e-12);
        if t != 20.0 && t != 50.0 {
            assert!(
                (f.trace.overhang_target_w[j] - target).abs() < 1e-12,
                "target at {t}"
            );
            assert_eq!(f.trace.overhang_tau_s[j], tau, "time in force at {t}");
        }
    }
    // 4 s after the step up the overhang has covered 1 - 1/e of the way; 1 s after the step down
    assert!(((h[at(24.0)] - h20) / (hi - h20) - (1.0 - (-1.0f64).exp())).abs() < 1e-12);
    assert!(((h[at(51.0)] - h50) / (lo - h50) - (1.0 - (-1.0f64).exp())).abs() < 1e-12);
    assert!(h50 > 0.79 && h20 > 0.319, "nearly settled before each step");

    // with the slow time off, the one time in both directions: the first forward, bit for bit
    let one_time = toy2(over);
    let g = run2(&one_time, &inp);
    let k = at(21.0);
    let h20 = g.trace.overhang_w[at(20.0)];
    assert!((g.trace.overhang_w[k] - (hi + (h20 - hi) * (-1.0f64).exp())).abs() < 1e-12);
    assert!(g.trace.overhang_tau_s.iter().all(|t| *t == 1.0));
    assert_same(&run1(&toy1(over), &inp), &g, "one overhang time");

    // read with a prefix: a step down after the cutoff shrinks with the fast time whatever the slow
    // one is (stills on the settled overhang of 10 cm/s; the law path has had 40 s to grow)
    let v0 = 0.1;
    let x = |t: f64| v0 * (t + 6.0);
    let slow_growth = toy2(&format!("{over}\nover.tau_up_s = 4.0"));
    let times = grid(35.0, 60.0, 0.25);
    let stepped = burn(
        35.0,
        0.19,
        vec![(-60.0, 10.0), (40.0, 2.0)],
        Some(35),
        |t| x(t) + hi,
        |t| x(t - 5.0),
        times.clone(),
    );
    let f = run2(&slow_growth, &stepped);
    let (t_end, tip_end) = (f.encoded.t_end_s, f.encoded.tip_end_w.unwrap());
    for (j, &t) in times.iter().enumerate() {
        let deficit = tip_end + v0 * (t - t_end) - f.quantiles[MED][j];
        let h40 = f.trace.overhang_w[times.iter().position(|s| *s == 40.0).unwrap()];
        let want = (f.encoded.h_end_w - h40) + (h40 - lo) * (1.0 - (-(t - 40.0).max(0.0)).exp());
        if t >= 40.0 {
            assert!(
                (deficit - want).abs() < 1e-9,
                "tip deficit at {t}: {deficit} vs {want}"
            );
        }
    }
}

/// The law path of the slow-down test by the forward's own recurrence: explicit steps of 0.25 s
/// from the igniter being switched on at -6 s, `rate(U_eff)` the spread rate; the factor acts in
/// the steps that start after the one that holds `t_ref`. Returns `(t, X, factor of the step)`.
fn slowed_path(
    rate: impl Fn(f64) -> f64,
    u: f64,
    t_ref: f64,
    k: f64,
    until: f64,
) -> Vec<(f64, f64, f64)> {
    let (t0, dt) = (-6.0, 0.25);
    let first = ((t_ref - t0) / dt).floor() as usize + 1;
    let n = ((until - t0) / dt).round() as usize;
    let mut out = Vec::with_capacity(n + 1);
    let (mut x, mut x_ref) = (0.0, 0.0);
    for i in 0..=n {
        if i + 1 == first {
            x_ref = x;
        }
        let factor = if i >= first {
            (1.0 - k * (x - x_ref)).max(0.0)
        } else {
            1.0
        };
        out.push((t0 + dt * i as f64, x, factor));
        x += rate(u * factor) * dt;
    }
    out
}

#[test]
fn the_front_slows_as_it_advances_by_the_written_share_per_sample_width() {
    // v = 0.1 + 0.05 (U / 5 - 1) = 0.05 + 0.01 U for U >= 0.1 cm/s; U = 10 cm/s: 0.15 w/s.
    // kappa = 0.2 per sample width: U_eff = U max(0, 1 - 0.2 (x - x_ref)).
    let rate = |u: f64| 0.05 + 0.01 * u.max(0.1);
    let law = "speed.skip[0:3] = [0.05, 0.0, 0.0]";
    let rel = toy2(&format!("{law}\nspeed.kappa = 0.2"));
    let x = |t: f64| 0.15 * (t + 6.0);
    let times = grid(10.0, 70.0, 0.25);
    let inp = burn(
        10.0,
        0.19,
        vec![(-60.0, 10.0)],
        Some(10),
        x,
        |t| x(t) - 1.0,
        times.clone(),
    );
    let f = run2(&rel, &inp);
    let e = f.encoded.e.exp();
    assert!((e - 1.0).abs() < 1e-8, "the toy prefix is not believed");
    // the reference is the front at the cutoff
    assert_eq!(f.trace.t_ref_s, 10.0);
    assert!((f.trace.x_ref_w - x(10.0)).abs() < 1e-12);
    let path = slowed_path(rate, 10.0, 10.0, 0.2 * e, 70.0);
    let (t_end, tip_end) = (f.encoded.t_end_s, f.encoded.tip_end_w.unwrap());
    assert_eq!(t_end, 8.0);
    for (j, &t) in times.iter().enumerate() {
        let (tp, xp, factor) = path[((t + 6.0) / 0.25).round() as usize];
        assert_eq!(tp, t);
        // exactly the formula, step by step
        let want = tip_end + e * (xp - x(t_end));
        assert!(
            (f.quantiles[MED][j] - want).abs() < 1e-11,
            "median at {t}: {} vs {want}",
            f.quantiles[MED][j]
        );
        assert!(
            (f.trace.flow_factor[j] - factor).abs() < 1e-11,
            "factor at {t}"
        );
        assert!((f.trace.front_w[j] - want).abs() < 1e-11);
        // the factor is the formula on the forecast front
        if t > 10.25 {
            let direct = (1.0 - 0.2 * (f.trace.front_w[j] - f.trace.x_ref_w)).max(0.0);
            assert!(
                (f.trace.flow_factor[j] - direct).abs() < 1e-9,
                "factor at {t}"
            );
            assert!((f.trace.speed_w_s[j] - rate(10.0 * factor)).abs() < 1e-12);
        }
        // and the solution of dy/dt = 0.15 - 0.02 y for the advance y, while the flow is not yet
        // zero (the explicit step of 0.25 s is within 0.01 widths of it)
        if (10.25..=50.0).contains(&t) {
            let y = 7.5 - (7.5 - 0.0375) * (-0.02 * (t - 10.25)).exp();
            assert!(
                (f.quantiles[MED][j] - (x(10.0) + y)).abs() < 0.01,
                "closed form at {t}"
            );
        }
    }
    // slower than the steady front, by more and more
    let steady = run2(&toy2(law), &inp);
    let lag: Vec<f64> = (0..times.len())
        .map(|j| steady.quantiles[MED][j] - f.quantiles[MED][j])
        .collect();
    assert!(
        lag[0] == 0.0 && lag[1] == 0.0,
        "nothing before the first slowed step"
    );
    assert!(lag.windows(2).skip(2).all(|w| w[1] > w[0]));
    assert!(steady.trace.flow_factor.iter().all(|x| *x == 1.0));

    // the flow never turns: with kappa = 1 it is zero one sample width downstream, and the front
    // keeps the rate of zero flow; the overhang target follows the effective flow; the quench hazard
    // and the burn-through time keep the flow as read (a hazard on below 2 cm/s stays off)
    let stop = toy2(&format!(
        "{law}\nspeed.kappa = 1.0\nover.b0 = 0.5\nover.skip[0:3] = [0.3, 0.0, 0.0]\n\
         quench.W1[0, 0] = -1.0\nquench.b1[0] = -0.6\nquench.w2[0] = 300.0"
    ));
    let tip = |t: f64| x(t) + 0.8;
    let g = run2(
        &stop,
        &burn(
            10.0,
            0.19,
            vec![(-60.0, 10.0)],
            Some(10),
            tip,
            x,
            times.clone(),
        ),
    );
    let spec = front2::spec();
    let j60 = times.iter().position(|t| *t == 60.0).unwrap();
    let last = times.len() - 1;
    assert_eq!(g.trace.flow_factor[j60], 0.0);
    assert_eq!(g.trace.flow_factor[last], 0.0);
    let zero = front2::laws_of(&spec, &stop, 0.0, 0.19, 1.0).unwrap();
    assert!((zero.v - 0.051).abs() < 1e-15);
    let slope = (g.trace.front_w[last] - g.trace.front_w[j60]) / 10.0;
    assert!(
        (slope - g.encoded.e.exp() * zero.v).abs() < 1e-12,
        "{slope}"
    );
    for (j, factor) in g.trace.flow_factor.iter().enumerate() {
        let l = front2::laws_of(&spec, &stop, 10.0 * factor, 0.19, 1.0).unwrap();
        assert!((g.trace.overhang_target_w[j] - l.h_ss).abs() < 1e-12);
    }
    assert!((g.trace.overhang_target_w[last] - zero.h_ss).abs() < 1e-12);
    assert!(
        (g.trace.overhang_w[last] - zero.h_ss).abs() < 1e-6,
        "the overhang follows"
    );
    assert!(g.p_quenched.iter().all(|p| *p < 1e-9));
    assert_eq!(bits(&g.trace.flow_cm_s), bits(&vec![10.0; times.len()]));

    // a run that is 20 % faster than the law, believed: the advance is E times the law path's, so
    // the factor falls 1.2 times as fast along the law path
    let fast = |t: f64| 0.18 * (t + 6.0);
    let trusting = toy2(&format!("{law}\nspeed.kappa = 0.2\nenc.gain_b = 20.0"));
    let h = run2(
        &trusting,
        &burn(
            10.0,
            0.19,
            vec![(-60.0, 10.0)],
            Some(10),
            fast,
            |t| fast(t) - 1.0,
            times.clone(),
        ),
    );
    let e = h.encoded.e.exp();
    assert!((e - 1.2).abs() < 1e-6);
    let path = slowed_path(rate, 10.0, 10.0, 0.2 * e, 70.0);
    for (j, &t) in times.iter().enumerate() {
        let (_, xp, factor) = path[((t + 6.0) / 0.25).round() as usize];
        let want = h.encoded.tip_end_w.unwrap() + e * (xp - x(8.0));
        assert!((h.quantiles[MED][j] - want).abs() < 1e-11, "median at {t}");
        assert!((h.trace.flow_factor[j] - factor).abs() < 1e-11);
    }

    // without stills the reference is the front when the igniter is switched off
    let times = grid(5.0, 45.0, 0.25);
    let blind = run2(
        &rel,
        &burn(5.0, 0.19, vec![(-60.0, 10.0)], None, x, x, times.clone()),
    );
    assert_eq!(blind.trace.t_ref_s, 0.0);
    assert!((blind.trace.x_ref_w - x(0.0)).abs() < 1e-12);
    let path = slowed_path(rate, 10.0, 0.0, 0.2, 45.0);
    for (j, &t) in times.iter().enumerate() {
        let (_, xp, factor) = path[((t + 6.0) / 0.25).round() as usize];
        assert!(
            (blind.quantiles[MED][j] - xp).abs() < 1e-11,
            "median at {t}"
        );
        assert!((blind.trace.flow_factor[j] - factor).abs() < 1e-11);
    }
    assert!(
        blind.trace.flow_factor[0] < 0.9,
        "already slowed at the cutoff"
    );
}

/// A steady burn at `u` cm/s whose stills follow the law of the toy weights at 6 cm/s.
fn steady(u: f64, times: Vec<f64>) -> FrontInput {
    let x = |t: f64| 0.11 * (t + 6.0);
    burn(
        10.0,
        0.19,
        vec![(-60.0, u)],
        Some(10),
        x,
        move |t| x(t) - 0.6,
        times,
    )
}

#[test]
fn the_flow_paths_mix_whole_runs_of_the_forward_with_normalised_weights() {
    // v = 0.05 + 0.01 U: 0.09, 0.11 and 0.13 w/s at 4, 6 and 8 cm/s
    let law = "speed.skip[0:3] = [0.05, 0.0, 0.0]\nunc.sigma_w = 0.05\nunc.s_pop = 0.1\nunc.drift_w = 0.01";
    let times = vec![10.0, 20.0, 40.0, 70.0];
    let single = toy2(&format!("{law}\nunc.flow_abs_cm_s = 2.0"));
    let paths = |w: &str| {
        toy2(&format!(
            "{law}\nunc.flow_abs_cm_s = 2.0\nunc.flow_w[0:3] = {w}"
        ))
    };
    let one = run2(&single, &steady(6.0, times.clone()));

    // [0, 1, 0]: one run, the single forward, whatever the path entries hold
    assert!(one.paths.is_empty());
    assert_same(
        &run1(&toy1(law), &steady(6.0, times.clone())),
        &one,
        "weights [0, 1, 0]",
    );
    // no positive outer weight is one run too
    let none = run2(&paths("[0.0, 0.0, 0.0]"), &steady(6.0, times.clone()));
    assert_eq!(bits(&none.quantiles[MED]), bits(&one.quantiles[MED]));
    assert!(none.paths.is_empty());

    // [1, 1, 1]: the three runs are the single forward on the flow moved by 2 cm/s
    let mix = run2(&paths("[1.0, 1.0, 1.0]"), &steady(6.0, times.clone()));
    assert_eq!(mix.paths.len(), 3);
    let lower = run2(&single, &steady(4.0, times.clone()));
    let upper = run2(&single, &steady(8.0, times.clone()));
    for (path, (alone, sign, u)) in
        mix.paths
            .iter()
            .zip([(&lower, -1.0, 4.0), (&one, 0.0, 6.0), (&upper, 1.0, 8.0)])
    {
        assert_eq!(path.sign, sign);
        assert_eq!(path.weight, 1.0 / 3.0);
        assert_eq!(bits(&path.tip_median_w), bits(&alone.quantiles[MED]));
        assert_eq!(bits(&path.base_median_w), bits(&alone.base_median));
        assert_eq!(bits(&path.p_quenched), bits(&alone.p_quenched));
        assert!(path.trace.flow_cm_s.iter().all(|x| *x == u));
        assert_eq!(path.encoded.e.to_bits(), alone.encoded.e.to_bits());
    }
    assert_eq!(
        mix.encoded.e.to_bits(),
        one.encoded.e.to_bits(),
        "the flow as read"
    );
    for j in 1..times.len() {
        let (lo, mid, hi) = (
            lower.quantiles[MED][j],
            mix.quantiles[MED][j],
            upper.quantiles[MED][j],
        );
        assert!(
            lo < mid && mid < hi,
            "median at {}: {lo} {mid} {hi}",
            times[j]
        );
        let band = |f: &front2::Forecast| f.quantiles[4][j] - f.quantiles[0][j];
        assert!(
            band(&mix) > band(&one) + 0.05,
            "band at {}: {} vs {}",
            times[j],
            band(&mix),
            band(&one)
        );
        assert!(mix.quantiles.windows(2).all(|q| q[1][j] >= q[0][j]));
    }
    // every quantile is where the weighted sum of the paths' distributions reaches its level
    let check_levels = |f: &front2::Forecast| {
        for j in 0..f.times_s.len() {
            for (l, level) in LEVELS.iter().enumerate() {
                let p: f64 = f
                    .paths
                    .iter()
                    .map(|p| {
                        p.weight
                            * normal_cdf((f.quantiles[l][j] - p.tip_median_w[j]) / p.tip_sd_w[j])
                    })
                    .sum();
                assert!((p - level).abs() < 1e-12, "level {level} at {j}: {p}");
            }
            let mean: f64 = f.paths.iter().map(|p| p.weight * p.tip_median_w[j]).sum();
            assert!((f.mean[j] - mean).abs() < 1e-12);
        }
        assert!((f.paths.iter().map(|p| p.weight).sum::<f64>() - 1.0).abs() < 1e-15);
    };
    check_levels(&mix);
    // equal weights: the base is that of the flow as read (the middle of the three)
    for (a, b) in mix.base_median.iter().zip(&one.base_median) {
        assert!((a - b).abs() < 1e-12, "base: {a} vs {b}");
    }

    // weights are normalised by their sum
    for w in [
        "[3.0, 3.0, 3.0]",
        "[1000.0, 1000.0, 1000.0]",
        "[0.5, 0.5, 0.5]",
    ] {
        let scaled = run2(&paths(w), &steady(6.0, times.clone()));
        for (a, b) in scaled.quantiles.iter().zip(&mix.quantiles) {
            assert_eq!(bits(a), bits(b), "weights {w}");
        }
        assert_eq!(bits(&scaled.mean), bits(&mix.mean));
    }
    let uneven = run2(&paths("[1.0, 2.0, 1.0]"), &steady(6.0, times.clone()));
    assert_eq!(
        uneven.paths.iter().map(|p| p.weight).collect::<Vec<_>>(),
        [0.25, 0.5, 0.25]
    );
    check_levels(&uneven);
    for j in 1..times.len() {
        let band = |f: &front2::Forecast| f.quantiles[4][j] - f.quantiles[0][j];
        assert!(band(&one) < band(&uneven) && band(&uneven) < band(&mix));
    }
    // more weight on the higher flow moves the median towards it
    let high = run2(&paths("[1.0, 1.0, 4.0]"), &steady(6.0, times.clone()));
    assert!(high.quantiles[MED][3] > mix.quantiles[MED][3] + 0.1);
    check_levels(&high);

    // a path of weight zero is not run; the reference is then the heavier outer path
    let outer = run2(&paths("[1.0, 0.0, 3.0]"), &steady(6.0, times.clone()));
    assert_eq!(
        outer
            .paths
            .iter()
            .map(|p| (p.sign, p.weight))
            .collect::<Vec<_>>(),
        [(-1.0, 0.25), (1.0, 0.75)]
    );
    assert_eq!(bits(&outer.trace.flow_cm_s), bits(&vec![8.0; times.len()]));
    let only_low = run2(&paths("[1.0, 0.0, 0.0]"), &steady(6.0, times.clone()));
    assert_eq!(only_low.paths.len(), 1);
    for (a, b) in only_low.quantiles.iter().zip(&lower.quantiles) {
        for (x, y) in a.iter().zip(b) {
            assert!((x - y).abs() < 1e-8, "one path is that run: {x} vs {y}");
        }
    }

    // the relative part scales before the shift, and a flow cannot turn negative
    let rel = toy2(&format!(
        "{law}\nunc.flow_abs_cm_s = 1.0\nunc.flow_rel = 0.25\nunc.flow_w[0:3] = [1.0, 1.0, 1.0]"
    ));
    let scaled = run2(&rel, &steady(6.0, times.clone()));
    let flows: Vec<f64> = scaled.paths.iter().map(|p| p.trace.flow_cm_s[0]).collect();
    assert_eq!(flows, [6.0 * 0.75 - 1.0, 6.0, 6.0 * 1.25 + 1.0]);
    let floor = run2(&paths("[1.0, 1.0, 1.0]"), &steady(1.0, times.clone()));
    let flows: Vec<f64> = floor.paths.iter().map(|p| p.trace.flow_cm_s[0]).collect();
    assert_eq!(flows, [0.0, 1.0, 3.0]);

    // paths that do not differ are the single forward (the mixture inverts the exact normal CDF,
    // the single run uses the rational quantile: 1e-9 of an sd apart)
    let same = toy2(&format!("{law}\nunc.flow_w[0:3] = [1.0, 1.0, 1.0]"));
    let f = run2(&same, &steady(6.0, times.clone()));
    for (a, b) in f.quantiles.iter().zip(&one.quantiles) {
        for (x, y) in a.iter().zip(b) {
            assert!((x - y).abs() < 1e-8, "{x} vs {y}");
        }
    }
}

#[test]
fn each_flow_path_reads_the_prefix_under_its_own_flow_and_the_event_rows_are_mixed() {
    // a believed prefix: each path reads its own run offset against its own law, so the three
    // paths carry the speed the stills show and the mixture closes up
    let law = "speed.skip[0:3] = [0.05, 0.0, 0.0]\nunc.sigma_w = 0.05\nunc.flow_abs_cm_s = 2.0\nunc.flow_w[0:3] = [1.0, 1.0, 1.0]";
    let times = vec![10.0, 20.0, 40.0, 70.0];
    let believed = run2(
        &toy2(&format!("{law}\nenc.gain_b = 20.0")),
        &steady(6.0, times.clone()),
    );
    let e: Vec<f64> = believed.paths.iter().map(|p| p.encoded.e).collect();
    for (got, v) in e.iter().zip([0.09, 0.11, 0.13]) {
        assert!((got - (0.11f64 / v).ln()).abs() < 1e-6, "{e:?}");
    }
    for j in 0..times.len() {
        let m: Vec<f64> = believed.paths.iter().map(|p| p.tip_median_w[j]).collect();
        assert!(
            (m[0] - m[1]).abs() < 1e-6 && (m[2] - m[1]).abs() < 1e-6,
            "{m:?}"
        );
    }
    // not believed: the paths part by 0.02 w/s
    let doubted = run2(&toy2(law), &steady(6.0, times.clone()));
    let m: Vec<f64> = doubted.paths.iter().map(|p| p.tip_median_w[3]).collect();
    assert!((m[2] - m[1] - 0.02 * 62.0).abs() < 1e-6, "{m:?}");

    // a quench that takes the flame only below 2 cm/s: at 3 cm/s as read the lower path (1 cm/s) is
    // quenched and the other two are not, so a third of the forecast is
    let gated = "quench.W1[0, 0] = -1.0\nquench.b1[0] = -0.6\nquench.w2[0] = 300.0";
    let rel = toy2(&format!("{law}\n{gated}\nunc.drift_w = 0.02"));
    let times = grid(10.0, 70.0, 2.0);
    let mut inp = steady(3.0, times.clone());
    inp.window_w = Some(4.0);
    let f = run2(&rel, &inp);
    let third = 1.0 / 3.0;
    assert!(f.p_quenched[0] < 1e-12);
    assert!((f.p_quenched[1] - third).abs() < 1e-9);
    assert!((f.p_quenched[times.len() - 1] - third).abs() < 1e-9);
    let mut some_reached = false;
    for j in 0..times.len() {
        // each row of the mixture is the weighted mean of the rows of the paths
        let (q, e) = (f.p_quenched[j], f.p_reached_end[j]);
        let row = |g: &dyn Fn(f64, f64) -> f64| -> f64 {
            f.paths
                .iter()
                .map(|p| p.weight * g(p.p_quenched[j], p.p_reached_end[j]))
                .sum()
        };
        assert!(((1.0 - q) * (1.0 - e) - row(&|q, e| (1.0 - q) * (1.0 - e))).abs() < 1e-12);
        assert!((q - row(&|q, _| q)).abs() < 1e-15);
        assert!(((1.0 - q) * e - row(&|q, e| (1.0 - q) * e)).abs() < 1e-12);
        assert!((0.0..=1.0).contains(&e));
        some_reached |= e > 0.5;
        // the tip is cut at the end of the sample
        assert!(f.quantiles.iter().all(|row| row[j] <= 4.0));
    }
    assert!(some_reached, "the unquenched paths reach the end");
    // the ignition forecast (no stills) is mixed too
    let blind = run2(
        &rel,
        &burn(
            0.0,
            0.19,
            vec![(-60.0, 3.0)],
            None,
            |t| t,
            |t| t,
            vec![20.0],
        ),
    );
    assert!((blind.p_quenched[0] - third).abs() < 1e-9);

    // the metric route carries the mixture and its paths
    let p = front_tools::fixture_prefix(10.0, 0.001);
    let a = front2::artifact(&front2::spec(), &rel, &p, &Value::Null).unwrap();
    a.validate().unwrap();
    assert!(a.support.admitted, "{:?}", a.support.reasons);
    for j in 0..a.times_s.len() {
        let s: f64 = a.event_cdf.iter().map(|r| r[j]).sum();
        assert!((s - 1.0).abs() < 1e-12, "rows sum to {s} at {j}");
    }
    assert_eq!(
        a.diagnostics["flow_paths"].as_array().map(Vec::len),
        Some(3)
    );
    assert!(a.diagnostics["trace_sample_widths"]["flow_factor"].is_array());
}

/// Whether the toy weights with `extra` are refused by the assembler.
fn refused(base: &str, extra: &str) -> bool {
    let spec = front2::spec();
    match text::compile(&source(base, extra), &spec) {
        Err(_) => true,
        Ok(ops) => {
            let patch = assemble::make_patch(&spec, common::meta("test-front2-bounds"), ops);
            assemble::materialize(&spec, &patch, None).is_err()
        }
    }
}

#[test]
fn every_construction_check_passes_and_an_entry_out_of_bounds_is_refused() {
    let complete = format!("{TOY}{OFF}");
    for rel in committed().iter().skip(2).chain([
        &toy2(""),
        &toy2("speed.split = 1.0\nspeed.pow_i = 6.0\nspeed.kappa = 1.0\nover.tau_up_s = 120.0\nunc.flow_abs_cm_s = 5.0\nunc.flow_rel = 1.0\nunc.flow_w[0:3] = [1000.0, 0.0, 1000.0]"),
    ]) {
        assert_eq!(rel.release.model_name, "bass_front.wsp2");
        let m = rel.release.construction.as_ref().unwrap();
        assert_eq!(m.checks.len(), 5);
        assert!(
            m.checks.iter().all(|c| c.status == CheckStatus::Passed),
            "{:?}",
            m.checks
        );
        assert_eq!(m.forward_hash, audit::forward_hash_of("bass_front.wsp2"));
        assert_eq!(m.spec_hash, front2::spec().spec_hash());
    }
    assert!(!refused(&complete, ""));
    for bad in [
        "speed.split = 1.5",
        "speed.split = -0.1",
        "speed.pow_i = 6.5",
        "speed.pow_i = -0.5",
        "speed.kappa = 1.01",
        "speed.kappa = -0.01",
        "over.tau_up_s = 121.0",
        "over.tau_up_s = -1.0",
        "unc.flow_abs_cm_s = 5.5",
        "unc.flow_abs_cm_s = -0.5",
        "unc.flow_rel = 1.2",
        "unc.flow_rel = -0.2",
        "unc.flow_w[0:3] = [0.0, 1.0, 1001.0]",
        "unc.flow_w[0:3] = [-1.0, 1.0, 0.0]",
        "unc.flow_w[0:4] = [0.0, 1.0, 0.0, 0.0]",
        "over.tau_s = 0.01",
    ] {
        assert!(refused(&complete, bad), "{bad} should be refused");
    }
    // a root patch must state every new entry: the numbers of the first forward alone are not a
    // release of the second, and neither is a patch that leaves one path weight unwritten
    assert!(refused(TOY, ""));
    assert!(refused(&complete, "unc.flow_w[0:2] = [0.0, 1.0]"));
    for line in OFF.lines() {
        let without: String = complete
            .lines()
            .filter(|l| *l != line)
            .map(|l| format!("{l}\n"))
            .collect();
        assert!(refused(&without, ""), "a root without `{line}`");
    }
    // a revision may write a new entry over its parent's, once
    let parent = toy2("");
    let on = revise("bass_front2", &parent, "speed.kappa = 0.2\n");
    assert_eq!(on.params["speed.kappa"].data, [0.2]);
    assert_eq!(on.params["speed.split"].data, [0.0]);
    assert!(refused(&complete, "speed.kappa = 0.2\nspeed.kappa = 0.3"));
    let spec = front2::spec();

    // a burn that cannot be forecast is refused before any path is run
    let rel = toy2("unc.flow_abs_cm_s = 2.0\nunc.flow_w[0:3] = [1.0, 1.0, 1.0]");
    let mut inp = steady(6.0, vec![10.0, 20.0]);
    inp.flow[0].1 = f64::NAN;
    let support = front2::forecast_track(&spec, &rel, &inp.into(), &LEVELS)
        .expect("runnable")
        .expect_err("a flow that is not a number");
    assert!(support.reasons[0].contains("flow"), "{:?}", support.reasons);
}

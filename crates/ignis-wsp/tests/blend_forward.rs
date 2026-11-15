//! The blend forward of the ACME burner flames (`blend.rs`): its spec is the two member specs under
//! the prefixes `a.` and `b.` plus two weights; its forecast is the weighted mean of what the two
//! member forwards return for the same numbers, prefix and times; a member with weight zero is not
//! run; no weight, or a member that refuses, is a refusal; the forward identity covers both member
//! sources; the committed candidates hold the member candidates' numbers unchanged.
//!
//! The weights used here are the committed reference baselines (roots, so each file is complete).

mod common;

use ignis_core::contracts::{CheckStatus, Family, ModelRelease, ModelSpec, SupportGate};
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::{acme_tools, assemble, audit, blend, flame, flame3, models, text};
use serde_json::json;

const FAMILIES: [(Family, &str); 2] = [
    (Family::AcmeNormal, "normal"),
    (Family::AcmeInverse, "inverse"),
];

fn config(path: &str) -> String {
    let path = format!(
        "{}/../../configs/models/acme/{path}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn key(family: Family, generation: u8) -> String {
    let name = if family == Family::AcmeInverse {
        "acme_inverse"
    } else {
        "acme_normal"
    };
    format!("{name}{generation}")
}

/// An in-memory root release of `key` from a complete edits text.
fn release(key: &str, src: &str) -> LoadedRelease {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key}: {e}"));
    let patch = assemble::make_patch(&spec, common::meta("test-blend"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    let release = model
        .build_release(&patch, &tensors, None, &common::binding())
        .expect("release");
    LoadedRelease {
        release,
        params: tensors,
        posterior: TensorMap::new(),
    }
}

/// The committed baseline pair with other weights.
fn pair(family: Family, dir: &str, w: [f64; 2]) -> LoadedRelease {
    let src = config(&format!("wsp4/{dir}/baseline-pair/edits.wsp"));
    let line = "mix.w[0:2] = [1.0, 1.0]";
    assert!(src.contains(line), "the baseline pair types its weights");
    release(
        &key(family, 4),
        &src.replace(line, &format!("mix.w[0:2] = [{:?}, {:?}]", w[0], w[1])),
    )
}

fn members(family: Family, dir: &str) -> (LoadedRelease, LoadedRelease) {
    (
        release(
            &key(family, 2),
            &config(&format!("wsp2/{dir}/baseline-clock/edits.wsp")),
        ),
        release(
            &key(family, 3),
            &config(&format!("wsp3/{dir}/baseline-theory/edits.wsp")),
        ),
    )
}

const TIMES: [f64; 7] = [3.0, 6.0, 15.0, 40.0, 90.0, 200.0, 500.0];

#[test]
fn the_committed_specs_are_the_member_specs_under_two_prefixes() {
    for (family, dir) in FAMILIES {
        let spec = blend::spec(family);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(&key(family, 4)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp4/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/acme/wsp4/{dir}/spec.json drifted from blend::spec(); a changed spec is \
             a new spec id (re-export with `ignis wsp spec --family {} --out …`), never an in-place edit",
            key(family, 4)
        );
        let (a, b) = (flame::spec(family), flame3::spec(family));
        assert_eq!(spec.tensors.len(), 1 + a.tensors.len() + b.tensors.len());
        assert_eq!(spec.tensors[0].name, "mix.w");
        let prefixed = a
            .tensors
            .iter()
            .map(|t| ("a.", t))
            .chain(b.tensors.iter().map(|t| ("b.", t)));
        for (have, (prefix, want)) in spec.tensors[1..].iter().zip(prefixed) {
            let mut want = want.clone();
            want.name = format!("{prefix}{}", want.name);
            assert_eq!(have, &want);
        }
        // the committed root assembles with every build check passed
        let r = pair(family, dir, [1.0, 1.0]);
        let checks = &r.release.construction.as_ref().unwrap().checks;
        assert!(
            checks.iter().all(|c| c.status == CheckStatus::Passed),
            "{checks:?}"
        );
    }
}

#[test]
fn the_forecast_is_the_weighted_mean_of_the_members_run_alone() {
    let settings = json!({ "times_s": TIMES });
    for (family, dir) in FAMILIES {
        let prefix = acme_tools::default_fixture(family);
        let (a, b) = members(family, dir);
        let fa = flame::forecast(&flame::spec(family), &a, &prefix, &settings)
            .unwrap()
            .expect("member a admitted");
        let fb = flame3::forecast(&flame3::spec(family), &b, &prefix, &settings)
            .unwrap()
            .expect("member b admitted");
        assert_eq!(fa.times_s, fb.times_s);
        let differ = (0..fa.times_s.len()).any(|j| (fa.table[1][j] - fb.table[1][j]).abs() > 0.05);
        assert!(differ, "the two baselines should not say the same thing");
        for w in [[1.0, 1.0], [3.0, 1.0], [0.2, 5.0]] {
            let r = pair(family, dir, w);
            let f = blend::forecast(&blend::spec(family), &r, &prefix, &settings)
                .unwrap()
                .expect("blend admitted");
            let (wa, wb) = (w[0] / (w[0] + w[1]), w[1] / (w[0] + w[1]));
            assert_eq!(f.weights, [wa, wb]);
            assert_eq!(f.times_s, fa.times_s);
            for j in 0..f.times_s.len() {
                let mut sum = 0.0;
                for k in 0..4 {
                    let want = wa * fa.table[k][j] + wb * fb.table[k][j];
                    assert!(
                        (f.table[k][j] - want).abs() < 1e-15,
                        "row {k} at {} s: {} against {want}",
                        f.times_s[j],
                        f.table[k][j]
                    );
                    sum += f.table[k][j];
                }
                assert!((sum - 1.0).abs() < 1e-12, "rows sum to {sum}");
            }
            let artifact = models::by_key(&key(family, 4))
                .unwrap()
                .artifact(&r, &prefix, &settings)
                .expect("artifact");
            artifact.validate().expect("valid artifact");
            assert!(artifact.support.admitted);
        }
    }
}

#[test]
fn a_member_with_weight_zero_is_not_run_and_the_other_answers_alone() {
    let settings = json!({ "times_s": TIMES });
    for (family, dir) in FAMILIES {
        let prefix = acme_tools::default_fixture(family);
        let (a, b) = members(family, dir);
        let fa = flame::forecast(&flame::spec(family), &a, &prefix, &settings)
            .unwrap()
            .unwrap();
        let fb = flame3::forecast(&flame3::spec(family), &b, &prefix, &settings)
            .unwrap()
            .unwrap();
        let only_a = blend::forecast(
            &blend::spec(family),
            &pair(family, dir, [2.0, 0.0]),
            &prefix,
            &settings,
        )
        .unwrap()
        .unwrap();
        assert!(only_a.b.is_none());
        assert_eq!(only_a.table, fa.table);
        let only_b = blend::forecast(
            &blend::spec(family),
            &pair(family, dir, [0.0, 0.5]),
            &prefix,
            &settings,
        )
        .unwrap()
        .unwrap();
        assert!(only_b.a.is_none());
        assert_eq!(only_b.table, fb.table);
    }
}

#[test]
fn no_weight_is_refused_and_a_member_refusal_names_the_member() {
    let settings = json!({ "times_s": TIMES });
    let family = Family::AcmeNormal;
    let spec = blend::spec(family);
    let prefix = acme_tools::default_fixture(family);
    // No weight cannot become a release: the fixture forecast is refused and the assembler rejects
    // a release with a failed build check.
    let model = models::by_key(&key(family, 4)).unwrap();
    let src = config("wsp4/normal/baseline-pair/edits.wsp")
        .replace("mix.w[0:2] = [1.0, 1.0]", "mix.w[0:2] = [0.0, 0.0]");
    let ops = text::compile(&src, &spec).expect("zero is inside the bounds");
    let patch = assemble::make_patch(&spec, common::meta("test-blend"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    assert!(model
        .build_release(&patch, &tensors, None, &common::binding())
        .is_err());
    // and the forward itself refuses such weights
    let mut zeroed = pair(family, "normal", [1.0, 1.0]);
    zeroed.params.get_mut("mix.w").expect("mix.w").data = vec![0.0, 0.0];
    let none = blend::forecast(&spec, &zeroed, &prefix, &settings)
        .unwrap()
        .expect_err("no weight");
    assert_eq!(none.gate, SupportGate::Semantic);
    assert!(none.reasons[0].contains("mix.w"), "{:?}", none.reasons);

    let mut not_established = prefix.clone();
    not_established.phase_established_at_s = None;
    let r = pair(family, "normal", [1.0, 1.0]);
    let refused = blend::forecast(&spec, &r, &not_established, &settings)
        .unwrap()
        .expect_err("phase not established");
    assert!(
        refused.reasons[0].starts_with("member a (flame.rs): "),
        "{:?}",
        refused.reasons
    );
    // with member a off, the same prefix is refused by member b, under its name
    let r = pair(family, "normal", [0.0, 1.0]);
    let refused = blend::forecast(&spec, &r, &not_established, &settings)
        .unwrap()
        .expect_err("phase not established");
    assert!(
        refused.reasons[0].starts_with("member b (flame3.rs): "),
        "{:?}",
        refused.reasons
    );
    // a blend release is not a member release: a member forward refuses to run it
    let wrong = flame3::forecast(&flame3::spec(family), &r, &prefix, &settings);
    assert!(wrong.is_err());
}

#[test]
fn the_forward_identity_covers_both_member_sources() {
    for name in ["acme_normal.wsp4", "acme_inverse.wsp4"] {
        let files: Vec<&str> = audit::forward_sources_of(name)
            .expect("a literal model")
            .iter()
            .map(|(n, _)| *n)
            .collect();
        for f in ["flame.rs", "flame3.rs", "blend.rs"] {
            assert!(files.contains(&f), "{name} lacks {f}: {files:?}");
        }
    }
    let blend = audit::forward_hash_of("acme_normal.wsp4");
    assert_ne!(blend, audit::forward_hash_of("acme_normal.wsp2"));
    assert_ne!(blend, audit::forward_hash_of("acme_normal.wsp3"));
}

#[test]
fn the_committed_candidates_hold_the_member_candidates_numbers_unchanged() {
    let read = |path: &str| -> ModelRelease {
        serde_json::from_str(&config(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    for (dir, a, b) in [
        ("normal", "wsp2/normal/d3-fable", "wsp3/normal/d1-fable"),
        ("inverse", "wsp2/inverse/d1-fable", "wsp3/inverse/d1-fable"),
    ] {
        let blend = read(&format!("wsp4/{dir}/d1-fable/release.json"));
        let (a, b) = (
            read(&format!("{a}/release.json")),
            read(&format!("{b}/release.json")),
        );
        assert_eq!(blend.tensors.len(), 1 + a.tensors.len() + b.tensors.len());
        for (prefix, member) in [("a.", &a), ("b.", &b)] {
            for t in &member.tensors {
                let name = format!("{prefix}{}", t.name);
                let have = blend
                    .tensors
                    .iter()
                    .find(|e| e.name == name)
                    .unwrap_or_else(|| panic!("{dir}: no tensor {name}"));
                assert_eq!(
                    (&have.shape, &have.sha256),
                    (&t.shape, &t.sha256),
                    "{dir}: {name} differs from {} of {}",
                    t.name,
                    member.id
                );
            }
        }
        let c = blend.construction.as_ref().expect("a literal release");
        assert_eq!(
            c.forward_hash,
            audit::forward_hash_of(&blend.model_name),
            "{dir}: re-assemble (tools/wsp/reassemble.py)"
        );
    }
}

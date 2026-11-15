//! The committee forward of the ACME burner flames (`committee.rs`): its spec is six member specs
//! under six slot prefixes plus six weights; its forecast is the weighted mean of what the member
//! forwards return for the same numbers, prefix and times; a slot with weight zero is not run and
//! one member alone answers bit for bit; no weight, or a member that refuses, is a refusal; the
//! forward identity covers both member sources; the committed candidates hold their member
//! releases' numbers unchanged.
//!
//! The weights used here are the committed reference baselines (roots, so each file is complete).

mod common;

use ignis_core::contracts::{CheckStatus, Family, ModelRelease, ModelSpec, SupportGate};
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::{acme_tools, assemble, audit, committee, flame, flame3, models, text};
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
    let patch = assemble::make_patch(&spec, common::meta("test-committee"), ops);
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

const ROOT_WEIGHTS: &str = "mix.w[0:6] = [1.0, 0.0, 1.0, 0.0, 0.0, 0.0]";

/// The committed root with other weights.
fn set(family: Family, dir: &str, w: [f64; 6]) -> LoadedRelease {
    let src = config(&format!("wsp5/{dir}/baseline-set/edits.wsp"));
    assert!(src.contains(ROOT_WEIGHTS), "the root types its weights");
    let typed: Vec<String> = w.iter().map(|x| format!("{x:?}")).collect();
    release(
        &key(family, 5),
        &src.replace(
            ROOT_WEIGHTS,
            &format!("mix.w[0:6] = [{}]", typed.join(", ")),
        ),
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
fn the_committed_specs_are_the_member_specs_under_six_prefixes() {
    for (family, dir) in FAMILIES {
        let spec = committee::spec(family);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(&key(family, 5)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp5/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/acme/wsp5/{dir}/spec.json drifted from committee::spec(); a changed \
             spec is a new spec id (re-export with `ignis wsp spec --family {} --out …`), never an \
             in-place edit",
            key(family, 5)
        );
        let (a, b) = (flame::spec(family), flame3::spec(family));
        // The committee's own tensors: the two sets of slot weights and the script offset.
        assert_eq!(
            spec.tensors.len(),
            3 + committee::A_SLOTS * a.tensors.len() + committee::B_SLOTS * b.tensors.len()
        );
        assert_eq!(spec.tensors[0].name, "mix.w");
        assert_eq!(spec.tensors[0].shape, vec![committee::SLOTS as u32]);
        assert_eq!(spec.tensors[1].name, "mix.w_script");
        assert_eq!(spec.tensors[1].shape, vec![committee::SLOTS as u32]);
        assert_eq!(spec.tensors[2].name, "script.onset_s");
        let mut at = 3;
        for (i, prefix) in committee::SLOT_PREFIX.iter().enumerate() {
            let member = if i < committee::A_SLOTS { &a } else { &b };
            for want in &member.tensors {
                let mut want = want.clone();
                want.name = format!("{prefix}{}", want.name);
                assert_eq!(&spec.tensors[at], &want);
                at += 1;
            }
        }
        let r = set(family, dir, [1.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
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
        // every slot of the root holds one of the two baselines, so any weights give a mean of two tables
        for w in [
            [1.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            [1.0, 2.0, 0.5, 0.0, 0.0, 0.5],
            [0.0, 0.3, 0.0, 1.0, 2.0, 3.0],
        ] {
            let r = set(family, dir, w);
            let f = committee::forecast(&committee::spec(family), &r, &prefix, &settings)
                .unwrap()
                .expect("committee admitted");
            let total: f64 = w.iter().sum();
            let wa = (w[0] + w[1]) / total;
            let wb = 1.0 - wa;
            assert_eq!(
                f.members.len(),
                w.iter().filter(|x| **x > 0.0).count(),
                "slots with weight zero are not run"
            );
            assert_eq!(f.times_s, fa.times_s);
            for j in 0..f.times_s.len() {
                let mut sum = 0.0;
                for k in 0..4 {
                    let want = wa * fa.table[k][j] + wb * fb.table[k][j];
                    assert!(
                        (f.table[k][j] - want).abs() < 1e-14,
                        "row {k} at {} s: {} against {want}",
                        f.times_s[j],
                        f.table[k][j]
                    );
                    sum += f.table[k][j];
                }
                assert!((sum - 1.0).abs() < 1e-12, "rows sum to {sum}");
            }
            let artifact = models::by_key(&key(family, 5))
                .unwrap()
                .artifact(&r, &prefix, &settings)
                .expect("artifact");
            artifact.validate().expect("valid artifact");
            assert!(artifact.support.admitted);
        }
    }
}

#[test]
fn one_member_alone_answers_bit_for_bit() {
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
        let spec = committee::spec(family);
        let only_a1 = committee::forecast(
            &spec,
            &set(family, dir, [0.0, 2.0, 0.0, 0.0, 0.0, 0.0]),
            &prefix,
            &settings,
        )
        .unwrap()
        .unwrap();
        assert_eq!(only_a1.members.len(), 1);
        assert_eq!(only_a1.members[0].slot, "a1.");
        assert_eq!(only_a1.table, fa.table);
        let only_b3 = committee::forecast(
            &spec,
            &set(family, dir, [0.0, 0.0, 0.0, 0.0, 0.0, 0.5]),
            &prefix,
            &settings,
        )
        .unwrap()
        .unwrap();
        assert_eq!(only_b3.members[0].slot, "b3.");
        assert_eq!(only_b3.table, fb.table);
    }
}

#[test]
fn no_weight_is_refused_and_a_member_refusal_names_the_slot() {
    let settings = json!({ "times_s": TIMES });
    let family = Family::AcmeInverse;
    let spec = committee::spec(family);
    let prefix = acme_tools::default_fixture(family);
    // No weight cannot become a release: the fixture forecast is refused and the assembler rejects
    // a release with a failed build check.
    let model = models::by_key(&key(family, 5)).unwrap();
    let src = config("wsp5/inverse/baseline-set/edits.wsp")
        .replace(ROOT_WEIGHTS, "mix.w[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]");
    let ops = text::compile(&src, &spec).expect("zero is inside the bounds");
    let patch = assemble::make_patch(&spec, common::meta("test-committee"), ops);
    let tensors = assemble::materialize(&spec, &patch, None).expect("materialize");
    assert!(model
        .build_release(&patch, &tensors, None, &common::binding())
        .is_err());
    // and the forward itself refuses such weights
    let mut zeroed = set(family, "inverse", [1.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
    zeroed.params.get_mut("mix.w").expect("mix.w").data = vec![0.0; committee::SLOTS];
    let none = committee::forecast(&spec, &zeroed, &prefix, &settings)
        .unwrap()
        .expect_err("no weight");
    assert_eq!(none.gate, SupportGate::Semantic);
    assert!(none.reasons[0].contains("mix.w"), "{:?}", none.reasons);

    let mut not_established = prefix.clone();
    not_established.phase_established_at_s = None;
    let r = set(family, "inverse", [0.0, 0.0, 0.0, 1.0, 1.0, 0.0]);
    let refused = committee::forecast(&spec, &r, &not_established, &settings)
        .unwrap()
        .expect_err("phase not established");
    assert!(
        refused.reasons[0].starts_with("member b1: "),
        "{:?}",
        refused.reasons
    );
    // a committee release is not a member release: a member forward refuses to run it
    assert!(flame3::forecast(&flame3::spec(family), &r, &prefix, &settings).is_err());
}

#[test]
fn the_forward_identity_covers_both_member_sources() {
    for name in ["acme_normal.wsp5", "acme_inverse.wsp5"] {
        let files: Vec<&str> = audit::forward_sources_of(name)
            .expect("a literal model")
            .iter()
            .map(|(n, _)| *n)
            .collect();
        for f in ["flame.rs", "flame3.rs", "committee.rs"] {
            assert!(files.contains(&f), "{name} lacks {f}: {files:?}");
        }
    }
    let c = audit::forward_hash_of("acme_inverse.wsp5");
    assert_ne!(c, audit::forward_hash_of("acme_inverse.wsp4"));
    assert_ne!(c, audit::forward_hash_of("acme_inverse.wsp3"));
}

#[test]
fn the_committed_candidates_hold_their_member_releases_numbers_unchanged() {
    let read = |path: &str| -> ModelRelease {
        serde_json::from_str(&config(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    // (committee candidate, the release each slot was copied from)
    let cases: [(&str, [&str; 6]); 1] = [(
        "wsp5/inverse/d1-fable",
        [
            "wsp2/inverse/baseline-clock",
            "wsp2/inverse/baseline-clock",
            "wsp3/inverse/d1-fable",
            "wsp3/inverse/d2-fable",
            "wsp3/inverse/baseline-theory",
            "wsp3/inverse/baseline-theory",
        ],
    )];
    for (dir, slots) in cases {
        let committee = read(&format!("{dir}/release.json"));
        // mix.w, mix.w_script and script.onset_s are the committee's own.
        let mut expected = 3;
        for (prefix, member_dir) in committee::SLOT_PREFIX.iter().zip(slots) {
            let member = read(&format!("{member_dir}/release.json"));
            expected += member.tensors.len();
            for t in &member.tensors {
                let name = format!("{prefix}{}", t.name);
                let have = committee
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
        assert_eq!(committee.tensors.len(), expected);
        let c = committee.construction.as_ref().expect("a literal release");
        assert_eq!(
            c.forward_hash,
            audit::forward_hash_of(&committee.model_name),
            "{dir}: re-assemble (tools/wsp/reassemble.py)"
        );
    }
}

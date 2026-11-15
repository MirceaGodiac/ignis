//! The edit language and the literal assembler: what an author writes is exactly what is stored,
//! nothing else is, and anything that is not a literal assignment is refused whole.

mod common;

use ignis_core::contracts::{
    ConstructionMethod, Family, Lineage, LiteralOp, ModelSpec, QualificationState, ReleaseId,
    TensorFill, TrainedOn,
};
use ignis_core::registry::{self, LoadedRelease, TensorMap};
use ignis_wsp::{assemble, flex, text};

fn spec() -> ModelSpec {
    flex::spec()
}

#[test]
fn committed_spec_json_is_the_spec_the_forward_was_compiled_with() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../configs/models/flex/wsp/spec.json"
    );
    let on_disk: ModelSpec = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        on_disk.spec_hash(),
        spec().spec_hash(),
        "configs/models/flex/wsp/spec.json drifted from flex::spec(); a changed spec is a new spec id \
         (re-export with `ignis wsp spec --family flex --out …`), never an in-place edit"
    );
}

#[test]
fn nothing_is_nonzero_before_the_author_writes() {
    let s = spec();
    let t = assemble::zero_init(&s);
    for ts in &s.tensors {
        let data = &t[&ts.name].data;
        match &ts.fill {
            TensorFill::Fixed { values } => {
                assert!(
                    data.iter().zip(values).all(|(a, b)| *a == b.get()),
                    "{}",
                    ts.name
                )
            }
            _ => assert!(
                data.iter().all(|v| *v == 0.0),
                "{} is not zero-initialised",
                ts.name
            ),
        }
    }
}

#[test]
fn statements_compile_to_the_three_literal_ops_and_land_row_major() {
    let s = spec();
    let ops = text::compile(
        "rate.W1[2, 5] = 1.5          # one element
         rate.W1[3, :] = -0.25        # constant over a row
         rate.b1[0:3] = [0.5, -0.5,
                         0.125]       # literal list over two lines
         haz.W1[0:2, 1:3] = [[1, 2], [3, 4]]
         rate.b0 = -0.58",
        &s,
    )
    .unwrap();
    assert!(matches!(&ops[0], LiteralOp::Set { index, .. } if index == &[2, 5]));
    assert!(
        matches!(&ops[1], LiteralOp::FillBox { start, shape, .. } if start == &[3, 0] && shape == &[1, 7])
    );
    assert!(matches!(&ops[2], LiteralOp::SetBox { values, .. } if values.len() == 3));
    assert!(matches!(&ops[4], LiteralOp::Set { index, .. } if index.is_empty()));
    let (_, patch) = common::flex_patch(
        "rate.W1[2, 5] = 1.5
         haz.W1[0:2, 1:3] = [[1, 2], [3, 4]]",
    );
    let t = assemble::materialize(&s, &patch, None).unwrap();
    let w = &t["rate.W1"].data;
    assert_eq!(w[2 * 7 + 5], 1.5);
    assert_eq!(w.iter().filter(|v| **v != 0.0).count(), 1);
    let h = &t["haz.W1"].data;
    assert_eq!([h[1], h[2], h[7 + 1], h[7 + 2]], [1.0, 2.0, 3.0, 4.0]);
    // every op renders back to a statement that compiles to the same op
    for op in &patch.ops {
        assert_eq!(&text::compile(&text::render(op), &s).unwrap()[0], op);
    }
}

#[test]
fn anything_but_a_literal_assignment_rejects_the_whole_file() {
    let s = spec();
    for (src, why) in [
        ("rate.b0 = 1 + 2", "arithmetic"),
        ("rate.b0 = -0.5 * 2", "arithmetic"),
        ("rate.b0 = haz.b0", "a name as a value"),
        ("rate.b0 = exp(1)", "a call"),
        ("rate.b1[0:16] = [0.1] * 16", "a repetition operator"),
        ("rate.b1[0:16:2] = 1.0", "a stride"),
        ("rate.b1[-1] = 1.0", "a negative index"),
        ("rate.b1[16] = 1.0", "an index out of range"),
        ("rate.b1[4:20] = 1.0", "a slice out of range"),
        ("rate.W1[0] = 1.0", "a missing axis"),
        ("rate.b0[0] = 1.0", "an index on a scalar"),
        ("rate.b1 = 1.0", "a tensor written without a selector"),
        ("rate.b1[0:3] = [1.0, 2.0]", "a list of the wrong length"),
        ("haz.W1[0:2, 0:2] = [[1, 2], [3]]", "a ragged list"),
        ("rate.b0 = nan", "nan"),
        ("rate.b0 = 1e999", "an overflowing literal"),
        ("rate.b0 = 0x10", "a hex literal"),
        ("resid.W9[0] = 1.0", "an unknown tensor"),
        ("rate.b0 = 1.0 rate.b1[0] = 2.0", "two statements on a line"),
        ("# only a comment", "an empty file"),
    ] {
        assert!(text::compile(src, &s).is_err(), "{why} was accepted: {src}");
    }
    // one bad statement among good ones: no partial result
    assert!(text::compile("rate.b0 = -0.5\nrate.b1[0] = 2 * 3\n", &s).is_err());
}

#[test]
fn a_root_patch_must_state_every_mandatory_parameter_and_respect_bounds() {
    let s = spec();
    // the toy patch without the disruption hazard: a hazard parameter is never filled tacitly
    let src: String = common::TOY_FLEX
        .lines()
        .filter(|l| !l.starts_with("haz.log_disruption"))
        .chain(["rate.skip[0:7] = 0.0", "haz.skip[0:7] = 0.0"])
        .collect::<Vec<_>>()
        .join("\n");
    let patch = assemble::make_patch(&s, common::meta("p"), text::compile(&src, &s).unwrap());
    let err = assemble::materialize(&s, &patch, None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("haz.log_disruption"), "{err}");
    // a value outside the declared physical range is rejected, not clipped
    let (_, patch) = common::flex_patch("post.k_evap = 1.0e-3");
    assert!(assemble::materialize(&s, &patch, None).is_err());
    // a constant of the spec cannot be written
    let (_, patch) = common::flex_patch("num.dt_s = 0.5");
    assert!(assemble::materialize(&s, &patch, None).is_err());
    // an element is written once
    let (_, patch) = common::flex_patch("rate.b1[3] = 0.1\nrate.b1[0:8] = 0.2");
    assert!(assemble::materialize(&s, &patch, None).is_err());
}

#[test]
fn releases_are_literal_reproducible_and_revisions_keep_their_parent() {
    let dir = std::env::temp_dir().join(format!("ignis-wsp-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let s = spec();
    let (_, patch) = common::flex_patch("rate.W1[0, 5] = 1.0\nrate.w2[0] = -0.2");
    let tensors = assemble::materialize(&s, &patch, None).unwrap();
    let build = || {
        ignis_wsp::build_release(Family::Flex, &patch, &tensors, None, &common::binding()).unwrap()
    };
    let w0 = assemble::save(&dir, build(), &tensors).unwrap();
    // classification: never NASA-trained, never qualified by construction, no posterior
    assert_eq!(w0.trained_on, TrainedOn::NotTrained);
    assert_eq!(w0.state, QualificationState::Experimental);
    assert_eq!(w0.lineage, Lineage::LiteralAuthoring);
    assert!(w0.posterior.is_empty() && w0.parent_id.is_none());
    let m = w0.construction.as_ref().unwrap();
    assert_eq!(m.construction_method, ConstructionMethod::LiteralOneShot);
    assert_eq!(m.patch_hash.as_ref(), Some(&patch.patch_hash()));
    assert_eq!(m.forward_hash, ignis_wsp::audit::forward_hash(Family::Flex));
    assert!(
        m.checks
            .iter()
            .all(|c| c.status == ignis_core::contracts::CheckStatus::Passed),
        "{:?}",
        m.checks
    );
    // the same spec and patch give the same release id, on a clean or a dirty working tree
    assert_eq!(assemble::save(&dir, build(), &tensors).unwrap().id, w0.id);
    // the stored tensors are exactly the assembled ones and nothing is marked trainable
    let loaded = registry::load(&dir, &w0.id).unwrap();
    assert_eq!(loaded.params, tensors);
    assert!(loaded.release.tensors.iter().all(|t| !t.trainable));

    // W1: a literal patch over the parent changes only what it names
    let mut meta = common::meta("test-flex-w1");
    meta.parent_release_id = Some(ReleaseId::new(w0.id.clone()).unwrap());
    let rev = assemble::make_patch(&s, meta, text::compile("rate.w2[0] = -0.3", &s).unwrap());
    let child = assemble::materialize(&s, &rev, Some(&loaded)).unwrap();
    for (name, t) in &child {
        for (i, v) in t.data.iter().enumerate() {
            let before = tensors[name].data[i];
            if name == "rate.w2" && i == 0 {
                assert_eq!((*v, before), (-0.3, -0.2));
            } else {
                assert_eq!(*v, before, "{name}[{i}] changed");
            }
        }
    }
    let w1 = ignis_wsp::build_release(
        Family::Flex,
        &rev,
        &child,
        Some(&loaded),
        &common::binding(),
    )
    .unwrap();
    assert_eq!(w1.lineage, Lineage::WeightEdit);
    assert_eq!(w1.parent_id.as_deref(), Some(w0.id.as_str()));
    assert_eq!(
        w1.construction.as_ref().unwrap().construction_method,
        ConstructionMethod::LiteralRevision
    );
    // a revision without its parent, or over a release of another spec, is refused
    assert!(assemble::materialize(&s, &rev, None).is_err());
    let mut other = LoadedRelease {
        release: loaded.release.clone(),
        params: loaded.params.clone(),
        posterior: TensorMap::new(),
    };
    other.release.construction.as_mut().unwrap().spec_hash =
        ignis_core::contracts::Digest::of_bytes(b"another spec");
    assert!(assemble::materialize(&s, &rev, Some(&other)).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

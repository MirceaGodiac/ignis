//! Essential lab checks: edits create new releases, never mutate the parent, track lineage and
//! posterior validity; test outcomes are unreachable from lab code.

mod common;

use std::collections::BTreeMap;
use std::path::Path;

use common::*;
use ignis_core::contracts::Lineage;
use ignis_core::registry::{load_release, release_dir, Tensor, TensorMap};
use ignis_lab::lab::{edits::apply_edit, guard::assert_lab_partition};
use serde_json::json;

fn bytes(d: &Path) -> BTreeMap<String, Vec<u8>> {
    std::fs::read_dir(d)
        .unwrap()
        .flatten()
        .map(|e| {
            (
                e.file_name().to_string_lossy().to_string(),
                std::fs::read(e.path()).unwrap(),
            )
        })
        .collect()
}

fn parent(env: &Env) -> ignis_core::contracts::ModelRelease {
    let params = TensorMap::from([
        ("theta".to_string(), Tensor::vector(vec![0.5, -1.0])),
        (
            "net.w".to_string(),
            Tensor::new(vec![2, 4], (0..8).map(|x| x as f64).collect()),
        ),
    ]);
    // draws of theta only
    let post = TensorMap::from([(
        "theta".to_string(),
        Tensor::new(vec![5, 2], (0..10).map(|x| 0.1 * x as f64).collect()),
    )]);
    release(env, "s", &["a"], json!({"hidden": 4}), &params, &post)
}

#[test]
fn edit_creates_child_and_leaves_parent_byte_identical() {
    let env = env("edit");
    let rel = parent(&env);
    let dir = release_dir(&env.artifacts(), &rel.id);
    let before = bytes(&dir);
    let op = json!({"op": "low_rank_add", "tensor": "net.w", "rank": 1, "seed": 3, "std": 0.1});
    let out = apply_edit(&rel.id, op.clone(), None, false).unwrap();
    assert_eq!(bytes(&dir), before);
    let (child, p, _) =
        load_release(&env.artifacts(), out["release_id"].as_str().unwrap(), None).unwrap();
    assert_eq!(child.parent_id.as_deref(), Some(rel.id.as_str()));
    assert_eq!(child.lineage, Lineage::WeightEdit);
    assert_eq!(child.lineage_note.as_deref(), out["manifest_hash"].as_str());
    let h = out["manifest_hash"]
        .as_str()
        .unwrap()
        .split(':')
        .nth(1)
        .unwrap()
        .to_string();
    assert!(env
        .artifacts()
        .join("edits")
        .join(format!("{h}.json"))
        .exists());
    assert_ne!(
        p["net.w"],
        load_release(&env.artifacts(), &rel.id, None).unwrap().1["net.w"]
    );
    assert!(child.posterior_valid_for_parameters); // net.w does not underlie the theta draws
    let again = apply_edit(&rel.id, op, None, false).unwrap();
    assert_eq!(again["manifest_hash"], out["manifest_hash"]); // reproducible
}

#[test]
fn edit_of_posterior_parameters_invalidates_draws_and_npy_edits_are_recorded() {
    let env = env("posterior");
    let rel = parent(&env);
    let out = apply_edit(
        &rel.id,
        json!([{"op": "scale", "tensor": "theta", "factor": 2.0}]),
        None,
        false,
    )
    .unwrap();
    let (child, _, _) =
        load_release(&env.artifacts(), out["release_id"].as_str().unwrap(), None).unwrap();
    assert!(!child.posterior_valid_for_parameters);
    assert_eq!(serde_json::to_value(child.state).unwrap(), "fit_only");
    assert_eq!(
        out["manifest"]["posterior"]["touched_basis"],
        json!(["theta"])
    );
    let grand = apply_edit(
        &child.id,
        json!({"op": "scale", "tensor": "net.w", "factor": 0.5}),
        None,
        false,
    )
    .unwrap();
    let (g, _, _) = load_release(
        &env.artifacts(),
        grand["release_id"].as_str().unwrap(),
        None,
    )
    .unwrap();
    assert!(!g.posterior_valid_for_parameters); // never relabelled valid
                                                // a tensor file produced elsewhere (e.g. by a Codex session): widened net.w, hash recorded
    let npy = env.root.join("widened.npy");
    ignis_lab::lab::nd::write_npy(
        &npy,
        &Tensor::new(vec![2, 5], (0..10).map(|x| x as f64).collect()),
    )
    .unwrap();
    let shaped = apply_edit(
        &rel.id,
        json!({"op": "from_npy", "tensor": "net.w", "file": npy.to_string_lossy()}),
        None,
        false,
    )
    .unwrap();
    assert_eq!(shaped["lineage"], "architecture_change");
    assert_eq!(shaped["manifest"]["reshaped"], json!(["net.w"]));
    assert_eq!(
        shaped["manifest"]["ops"][0]["source"]["file_sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    let dry = apply_edit(
        &rel.id,
        json!({"op": "pad", "tensor": "net.w", "pad": [[0, 0], [0, 1]]}),
        None,
        true,
    )
    .unwrap();
    assert_eq!(dry["manifest"]["lineage"], "architecture_change");
    assert!(dry.get("release_id").is_none());
    // a declared lineage may not understate the edit; python code is never executed
    let under = apply_edit(
        &rel.id,
        json!({"ops": [{"op": "pad", "tensor": "net.w", "pad": [[0, 0], [0, 1]]}], "lineage": "weight_edit"}),
        None,
        true,
    );
    assert!(is_refusal(&under.unwrap_err(), "understates"));
    assert!(is_refusal(
        &apply_edit(&rel.id, json!({"op": "python", "ref": "m:f"}), None, true).unwrap_err(),
        "not executed"
    ));
}

#[test]
fn lab_cannot_reach_test_partition_or_the_vault() {
    let env = env("guard");
    assert!(assert_lab_partition("test").is_err());
    assert!(assert_lab_partition("validation").is_ok());
    let _lab = ignis_lab::access::LabSession::enter();
    let err = ignis_lab::eval::reveal("sha256:00", true).err().unwrap();
    assert!(is_refusal(
        &err,
        "lab code cannot read sealed continuations"
    ));
    let log = ignis_lab::eval::vault::reveal_log();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0]["granted"], false);
    drop(env);
}

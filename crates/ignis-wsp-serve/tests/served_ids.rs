//! The served releases keep the ids the authoring lane adopted. A release id is the content hash of
//! its record, which includes the forward identity and the weights digest; this fails if a served
//! release, its forward source or its weights changed.

use std::path::{Path, PathBuf};

use ignis_wsp::MemoryPrefixes;
use ignis_wsp_serve::{ServedList, ServedModels};

/// The adopted releases of `docs/wsp/README.md` (feat/handwritten-science e4477d6), in full.
const PINNED: [(&str, &str); 5] = [
    (
        "flex_methanol.wsp2",
        "sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce",
    ),
    (
        "flex_heptane.wsp2",
        "sha256:43f7871f778040550291789ca97e319729675a9dd0df9c9529525c1805ad588d",
    ),
    (
        "acme_normal.wsp12",
        "sha256:55fcf4cbbf500ca57a4e4207426826d5c3d3b21d84fb279479e29b7e5a65971d",
    ),
    (
        "acme_inverse.wsp12",
        "sha256:dff7da0da8e77785acecc0ffb5acaf222358d4274b36e2c52f39426aaebd594e",
    ),
    (
        "bass_front.wsp",
        "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c",
    ),
];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn served() -> PathBuf {
    repo().join(ignis_wsp_serve::DEFAULT_DIR)
}

#[test]
fn served_ids_are_the_adopted_ids_and_verify() {
    let list = ServedList::read(&served().join("served.json")).unwrap();
    let ids: Vec<(&str, &str)> = list
        .releases
        .iter()
        .map(|e| (e.model_name.as_str(), e.release_id.as_str()))
        .collect();
    assert_eq!(ids, PINNED);
    // The README quotes each adopted id as `sha256:<first 8>…<last 4>`.
    let readme = std::fs::read_to_string(repo().join("docs/wsp/README.md")).unwrap();
    for (_, id) in PINNED {
        let hex = &id[7..];
        let quoted = format!("sha256:{}…{}", &hex[..8], &hex[hex.len() - 4..]);
        assert!(
            readme.contains(&quoted),
            "{quoted} is not in docs/wsp/README.md"
        );
    }
    // Registry checks, content hash of each record and forward identity of this build.
    let m = ServedModels::open(served(), MemoryPrefixes::new()).unwrap();
    assert_eq!(m.count(), PINNED.len());
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), target).unwrap();
        }
    }
}

#[test]
fn a_changed_record_or_an_unlisted_forward_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&served(), dir.path());
    // One edited character of a record (not a number of the model): the id no longer matches.
    let hex = &PINNED[0].1[7..];
    let record = dir.path().join("models").join(hex).join("release.json");
    let text = std::fs::read_to_string(&record).unwrap();
    std::fs::write(
        &record,
        text.replacen("D3, data-informed", "D3, data informed", 1),
    )
    .unwrap();
    let e = ServedModels::open(dir.path(), MemoryPrefixes::new())
        .err()
        .expect("a changed record is refused");
    assert!(format!("{e:#}").contains("hashes to"), "{e:#}");

    // The list names a release with another model name: refused.
    std::fs::write(&record, text).unwrap();
    let list = dir.path().join("served.json");
    let t = std::fs::read_to_string(&list).unwrap();
    std::fs::write(
        &list,
        t.replacen("\"flex_heptane.wsp2\"", "\"flex_heptane.wsp3\"", 1),
    )
    .unwrap();
    assert!(ServedModels::open(dir.path(), MemoryPrefixes::new()).is_err());
}

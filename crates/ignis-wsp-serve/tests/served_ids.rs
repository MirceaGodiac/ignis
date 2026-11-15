//! The served releases keep the ids the authoring lane adopted. A release id is the content hash of
//! its record, which includes the forward identity and the weights digest; this fails if a served
//! release, its forward source or its weights changed.

use std::path::{Path, PathBuf};

use ignis_wsp::MemoryPrefixes;
use ignis_wsp_serve::{ServedList, ServedModels};

/// The adopted releases: the five of `docs/wsp/README.md` (feat/handwritten-science), then
/// the nine generic-law releases of `docs/wsp/COVERAGE_19.md`, in full.
const PINNED: [(&str, &str); 14] = [
    (
        "flex_methanol.wsp2",
        "sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654",
    ),
    (
        "flex_heptane.wsp2",
        "sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef",
    ),
    (
        "acme_normal.wsp12",
        "sha256:2ae770d8d159bca359e2125ef348ce13724abce49e91af4a3d62c5c66af6bc52",
    ),
    (
        "acme_inverse.wsp12",
        "sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7",
    ),
    (
        "bass_front.wsp",
        "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c",
    ),
    (
        "bre.wsp",
        "sha256:aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d",
    ),
    (
        "spice.wsp",
        "sha256:93e750242725a5a1fb9d38f4178b23605cac636f210bf9f3f8bc8e8150e2f2a8",
    ),
    (
        "efield.wsp",
        "sha256:73f91b55c81dc724fe2441c2254e109c65f8e08b253fcbe4c00a841f7a0986a9",
    ),
    (
        "samer.wsp",
        "sha256:05f77ae2f7a666e0da69d04fa29d9ad8b0ac15790645abe1adc0149292327fb0",
    ),
    (
        "same.wsp",
        "sha256:e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062",
    ),
    (
        "daft.wsp",
        "sha256:061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d",
    ),
    (
        "bass1.wsp",
        "sha256:b3bb1869df2fc837f7957e925f55b357ecc6272d312f1ddbb46346da15376ca0",
    ),
    (
        "slice.wsp",
        "sha256:d10c94bc6ea0bcb957f89734393b0cbabbd44fdf0ac9c2e440f051a33cd3006e",
    ),
    (
        "cld.wsp",
        "sha256:a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f",
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
    for (_, id) in &PINNED[..5] {
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

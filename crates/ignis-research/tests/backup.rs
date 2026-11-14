//! State snapshots: taken while the store is open and writing, restored onto an empty volume with
//! every episode and event intact; refused onto a non-empty target, with tampered files, with paths
//! that escape the target; credentials never captured; pushed to and pulled from the bucket.

use std::path::Path;

use ignis_core::contracts::*;
use ignis_research::backup::{pull, push, restore, snapshot, verify};
use ignis_research::s3::S3Client;
use ignis_research::store::Replay;
use ignis_research::ResearchStore;

mod common;
mod s3fake;

fn seed(state: &Path) -> (ExecutionScope, EpisodeId, ResearchStore) {
    let store = ResearchStore::open(state.join("state.sqlite")).unwrap();
    let sc = common::scope("ws_ana", true);
    let (ep, _) = store
        .create_episode(
            &sc,
            "flame spread",
            &ClientRequestId::new("c-1").unwrap(),
            Default::default(),
            Origin::System,
        )
        .unwrap();
    for i in 0..20 {
        store
            .post_message(
                &sc,
                &ep.episode_id,
                &format!("message {i}"),
                &ClientRequestId::new(format!("m-{i}")).unwrap(),
            )
            .unwrap();
    }
    // What else lives in a state directory.
    std::fs::create_dir_all(state.join("codex/sessions")).unwrap();
    std::fs::write(state.join("codex/sessions/t1.jsonl"), b"{\"thread\":1}\n").unwrap();
    std::fs::write(state.join("codex/auth.json"), b"{\"token\":\"secret\"}").unwrap();
    std::fs::create_dir_all(state.join("uploads/up_1")).unwrap();
    std::fs::write(state.join("uploads/up_1/data"), b"partial").unwrap();
    (sc, ep.episode_id, store)
}

fn events(store: &ResearchStore, sc: &ExecutionScope, ep: &EpisodeId) -> Vec<ResearchEvent> {
    match store.replay(sc, ep, 0, 10_000).unwrap() {
        Replay::Events { events, .. } => events,
        _ => panic!("log does not cover the cursor"),
    }
}

#[test]
fn a_live_store_restores_onto_an_empty_volume() {
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("state");
    let (sc, ep, live) = seed(&state);
    let before = events(&live, &sc, &ep);
    assert!(before.len() >= 20);

    // Taken while `live` holds the database open in WAL mode.
    let snap = dir.path().join("snap");
    let m = snapshot(&state, &snap).unwrap();
    let paths: Vec<&str> = m.files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"state.sqlite"), "{paths:?}");
    assert!(paths.contains(&"codex/sessions/t1.jsonl"));
    assert!(!paths
        .iter()
        .any(|p| p.contains("auth.json") || p.contains("uploads") || p.ends_with("-wal")));
    // Writes after the snapshot are not in it.
    live.post_message(&sc, &ep, "after", &ClientRequestId::new("m-after").unwrap())
        .unwrap();

    let target = dir.path().join("restored");
    restore(&snap, &target).unwrap();
    let restored = ResearchStore::open(target.join("state.sqlite")).unwrap();
    assert_eq!(events(&restored, &sc, &ep), before);
    assert_eq!(
        restored.episode(&sc, &ep).unwrap().last_seq,
        before.last().unwrap().seq
    );
    check_event_log(&events(&restored, &sc, &ep)).unwrap();
    // The restored store keeps working (next seq follows the restored log).
    restored
        .post_message(&sc, &ep, "next", &ClientRequestId::new("m-next").unwrap())
        .unwrap();

    // Never over existing state.
    let e = restore(&snap, &target).unwrap_err();
    assert!(e.contains("not empty"), "{e}");
}

#[test]
fn damaged_or_hostile_snapshots_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("state");
    let (_sc, _ep, _live) = seed(&state);
    let snap = dir.path().join("snap");
    snapshot(&state, &snap).unwrap();
    verify(&snap).unwrap();

    // Tampered bytes.
    let f = snap.join("files/codex/sessions/t1.jsonl");
    std::fs::write(&f, b"{\"thread\":2}\n").unwrap();
    assert!(restore(&snap, &dir.path().join("t1")).is_err());
    assert!(
        !dir.path().join("t1").join("state.sqlite").exists(),
        "nothing copied"
    );
    std::fs::write(&f, b"{\"thread\":1}\n").unwrap();
    verify(&snap).unwrap();

    // A manifest path that would leave the target.
    let mpath = snap.join("manifest.json");
    let good = std::fs::read_to_string(&mpath).unwrap();
    for bad in ["../escape", "/etc/passwd", "a/../../b", "C:/x"] {
        std::fs::write(&mpath, good.replacen("codex/sessions/t1.jsonl", bad, 1)).unwrap();
        let e = restore(&snap, &dir.path().join("t2")).unwrap_err();
        assert!(e.contains("unsafe path"), "{bad}: {e}");
    }
}

#[test]
fn snapshots_round_trip_through_the_bucket() {
    let f = s3fake::fake();
    let s3 = S3Client::new(s3fake::cfg(&f)).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("state");
    let (sc, ep, live) = seed(&state);
    let before = events(&live, &sc, &ep);
    let snap = dir.path().join("snap");
    snapshot(&state, &snap).unwrap();
    push(&s3, &snap, "backups/", "state-1").unwrap();
    assert!(
        push(&s3, &snap, "backups/", "state-1").is_err(),
        "a published snapshot is never overwritten"
    );

    let pulled = dir.path().join("pulled");
    pull(&s3, "backups/", "state-1", &pulled).unwrap();
    let target = dir.path().join("restored");
    restore(&pulled, &target).unwrap();
    let restored = ResearchStore::open(target.join("state.sqlite")).unwrap();
    assert_eq!(events(&restored, &sc, &ep), before);
    assert!(f.bucket.lock().unwrap().refused.is_empty());

    assert!(pull(&s3, "backups/", "missing", &dir.path().join("nope")).is_err());
}

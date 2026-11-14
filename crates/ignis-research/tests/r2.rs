//! `R2ArtifactStore` against the in-process S3 stand-in (`s3fake`: verifies signatures and body
//! hashes, refuses chunked uploads, honours conditional writes, ranges and multipart). The live check against a real
//! bucket is `r2_live` (opt-in).

use std::io::Read;

use ignis_core::contracts::*;
use ignis_core::services::{ArtifactDraft, ArtifactStore};
use ignis_research::r2::R2ArtifactStore;
use ignis_research::s3::S3Config;
use sha2::{Digest as _, Sha256};

mod s3fake;
use s3fake::{cfg, fake, KEY_ID, SECRET};

fn scope(ws: &str) -> ExecutionScope {
    let mut s: ExecutionScope = serde_json::from_str(include_str!(
        "../../../fixtures/contracts/common/scope_research.json"
    ))
    .unwrap();
    s.workspace_id = WorkspaceId::new(ws).unwrap();
    s
}

fn draft() -> ArtifactDraft {
    ArtifactDraft {
        media_type: "application/octet-stream".into(),
        contract: None,
        produced_by: Producer::default(),
        inputs: vec![],
        origin: Origin::System,
    }
}

fn read_all(mut r: Box<dyn Read + Send>) -> Vec<u8> {
    let mut v = Vec::new();
    r.read_to_end(&mut v).unwrap();
    v
}

#[test]
fn bytes_round_trip_through_signed_requests_with_ranges_and_isolation() {
    let f = fake();
    let dir = tempfile::tempdir().unwrap();
    let store = R2ArtifactStore::open(cfg(&f), "registry", dir.path().join("a")).unwrap();
    store.probe().unwrap();
    let ana = scope("ws_ana");
    let bytes: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();

    let a = store.put(&ana, draft(), &mut bytes.as_slice()).unwrap();
    assert_eq!(a.byte_length, bytes.len() as u64);
    assert_eq!(a.sha256.hex(), hex::encode(Sha256::digest(&bytes)));
    assert_eq!(
        read_all(store.open(&ana, &a.artifact_id, 0, None).unwrap()),
        bytes
    );
    assert_eq!(
        read_all(store.open(&ana, &a.artifact_id, 99_990, Some(100)).unwrap()),
        bytes[99_990..],
        "range clamped to the end"
    );
    assert_eq!(
        read_all(store.open(&ana, &a.artifact_id, 10, Some(5)).unwrap()),
        bytes[10..15]
    );
    assert!(read_all(
        store
            .open(&ana, &a.artifact_id, bytes.len() as u64, None)
            .unwrap()
    )
    .is_empty());
    assert!(store
        .open(&ana, &a.artifact_id, bytes.len() as u64 + 1, None)
        .is_err());

    // Another workspace cannot see it; a cold store (no local cache) finds it in the bucket.
    let bob = scope("ws_bob");
    assert_eq!(
        store.stat(&bob, &a.artifact_id).unwrap_err().code,
        ErrorCode::NotFound
    );
    let cold = R2ArtifactStore::open(cfg(&f), "registry", dir.path().join("b")).unwrap();
    assert_eq!(cold.stat(&ana, &a.artifact_id).unwrap(), a);
    let other_prefix = R2ArtifactStore::open(cfg(&f), "fixture", dir.path().join("c")).unwrap();
    assert!(
        other_prefix.stat(&ana, &a.artifact_id).is_err(),
        "instances do not share"
    );

    // The same output again: same reference (first created_at), no second object upload.
    let puts_before = f
        .bucket
        .lock()
        .unwrap()
        .log
        .iter()
        .filter(|l| l.0 == "PUT")
        .count();
    let again = cold.put(&ana, draft(), &mut bytes.as_slice()).unwrap();
    assert_eq!(again, a);
    let puts_after = f
        .bucket
        .lock()
        .unwrap()
        .log
        .iter()
        .filter(|l| l.0 == "PUT")
        .count();
    assert_eq!(puts_before, puts_after);

    // Same bytes for bob: a new manifest, the object is reused.
    let b = store.put(&bob, draft(), &mut bytes.as_slice()).unwrap();
    assert_ne!(b.artifact_id, a.artifact_id);
    let bucket = f.bucket.lock().unwrap();
    let objects = bucket
        .objects
        .keys()
        .filter(|k| k.contains("/objects/"))
        .count();
    assert_eq!(objects, 1);
    assert!(bucket.refused.is_empty(), "{:?}", bucket.refused);
    assert!(bucket.objects.keys().all(|k| k.starts_with("registry/")));
    // Nothing left in staging.
    assert_eq!(
        std::fs::read_dir(dir.path().join("a/staging"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn large_outputs_go_up_in_verified_parts() {
    let f = fake();
    let dir = tempfile::tempdir().unwrap();
    // 1 KiB parts so a 5000-byte artifact needs 5 parts (the last one short).
    let store = R2ArtifactStore::with_part_size(cfg(&f), "p", dir.path(), 1024).unwrap();
    let ana = scope("ws_ana");
    let bytes: Vec<u8> = (0..5000u32).map(|i| (i * 7 % 256) as u8).collect();
    let a = store.put(&ana, draft(), &mut bytes.as_slice()).unwrap();
    assert_eq!(
        read_all(store.open(&ana, &a.artifact_id, 0, None).unwrap()),
        bytes
    );
    let exact: Vec<u8> = vec![9; 2048];
    let e = store.put(&ana, draft(), &mut exact.as_slice()).unwrap();
    assert_eq!(
        read_all(store.open(&ana, &e.artifact_id, 0, None).unwrap()),
        exact
    );
    let empty = store.put(&ana, draft(), &mut [].as_slice()).unwrap();
    assert_eq!(empty.byte_length, 0);
    assert!(read_all(store.open(&ana, &empty.artifact_id, 0, None).unwrap()).is_empty());

    let b = f.bucket.lock().unwrap();
    assert!(b.refused.is_empty(), "{:?}", b.refused);
    let parts = |n: usize| {
        b.log
            .iter()
            .filter(|l| l.0 == "PUT" && l.2.contains("partNumber"))
            .count()
            == n
    };
    assert!(parts(5 + 2), "5 parts + 2 parts: {:?}", b.log);
    assert!(b.uploads.is_empty(), "every upload completed");
}

#[test]
fn a_manifest_published_first_by_another_writer_wins() {
    let f = fake();
    let dir = tempfile::tempdir().unwrap();
    let one = R2ArtifactStore::open(cfg(&f), "", dir.path().join("1")).unwrap();
    let two = R2ArtifactStore::open(cfg(&f), "", dir.path().join("2")).unwrap();
    let ana = scope("ws_ana");
    let first = one.put(&ana, draft(), &mut b"same".as_slice()).unwrap();
    // `two` has no cached manifest; its conditional publish gets 412 and adopts the existing one.
    std::thread::sleep(std::time::Duration::from_millis(5));
    let second = two.put(&ana, draft(), &mut b"same".as_slice()).unwrap();
    assert_eq!(second.created_at, first.created_at);
}

#[test]
fn unreachable_or_misconfigured_storage_is_reported_not_hidden() {
    let dead = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = dead.local_addr().unwrap().port();
    drop(dead);
    let dir = tempfile::tempdir().unwrap();
    let mut c = S3Config {
        endpoint: format!("http://127.0.0.1:{port}"),
        bucket: "ignis-test".into(),
        region: "auto".into(),
        access_key_id: KEY_ID.into(),
        secret_access_key: SECRET.into(),
    };
    let store = R2ArtifactStore::open(c.clone(), "", dir.path()).unwrap();
    assert!(store.probe().is_err());
    let e = store
        .put(&scope("ws_ana"), draft(), &mut b"x".as_slice())
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::UpstreamUnavailable);
    assert_eq!(
        std::fs::read_dir(dir.path().join("staging"))
            .unwrap()
            .count(),
        0
    );

    // Wrong secret: the stand-in refuses the signature, and that is not retryable.
    let f = fake();
    c.endpoint = f.endpoint.clone();
    c.secret_access_key = "wrong".into();
    let store = R2ArtifactStore::open(c, "", dir.path()).unwrap();
    let e = store
        .put(&scope("ws_ana"), draft(), &mut b"x".as_slice())
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::Internal, "{e:?}");
}

/// Against a real bucket: `IGNIS_R2_LIVE_ENDPOINT`, `IGNIS_R2_LIVE_BUCKET`,
/// `IGNIS_R2_LIVE_ACCESS_KEY_ID`, `IGNIS_R2_LIVE_SECRET_ACCESS_KEY`. Writes under a fresh prefix
/// `ignis-live-test/<uuid>/` (left in place; R2 has no cheap recursive delete via this client).
#[test]
#[ignore = "needs a real R2/S3 bucket: IGNIS_R2_LIVE_* variables"]
fn r2_live() {
    let var = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("{k} not set"));
    let c = S3Config {
        endpoint: var("IGNIS_R2_LIVE_ENDPOINT"),
        bucket: var("IGNIS_R2_LIVE_BUCKET"),
        region: std::env::var("IGNIS_R2_LIVE_REGION").unwrap_or_else(|_| "auto".into()),
        access_key_id: var("IGNIS_R2_LIVE_ACCESS_KEY_ID"),
        secret_access_key: var("IGNIS_R2_LIVE_SECRET_ACCESS_KEY"),
    };
    let dir = tempfile::tempdir().unwrap();
    let prefix = format!("ignis-live-test/{}", uuid_like());
    // 5 MiB parts (R2's minimum) so 11 MiB exercises multipart.
    let store = R2ArtifactStore::with_part_size(c, &prefix, dir.path(), 5 << 20).unwrap();
    store.probe().unwrap();
    let ana = scope("ws_ana");
    let big: Vec<u8> = (0..(11u32 << 20)).map(|i| (i % 253) as u8).collect();
    let a = store.put(&ana, draft(), &mut big.as_slice()).unwrap();
    assert_eq!(
        read_all(store.open(&ana, &a.artifact_id, 0, None).unwrap()),
        big
    );
    assert_eq!(
        read_all(store.open(&ana, &a.artifact_id, 6 << 20, Some(10)).unwrap()),
        big[(6 << 20)..(6 << 20) + 10]
    );
    let again = store.put(&ana, draft(), &mut big.as_slice()).unwrap();
    assert_eq!(again, a);
}

fn uuid_like() -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{t:x}")
}

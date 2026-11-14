//! M12 source access, offline: route → catalog → resolver → `SourceAccess` on manifest rows copied
//! from `data/manifests` (`tests/data/sources/manifests`), against a loopback stand-in of geode
//! that answers with the measured `redirect=false` body (signature redacted, expiry refreshed). The
//! keyframe index is a synthetic fixture (R11 has not landed). Live geode is opt-in:
//! `IGNIS_LIVE_PSI=1 cargo test -p ignis-server --test sources -- --ignored`.

use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_core::contracts::*;
use ignis_server::v1::io::{research_scope, AuthMode};
use ignis_server::v1::sources::catalog::{Catalog, PsiFile};
use ignis_server::v1::sources::hls::{self, KeyframeIndex};
use ignis_server::v1::sources::resolver::{GeodeSigner, Signed, SignedUrls, Signer};
use ignis_server::v1::sources::{SourceArgs, SourceDelivery};
use ignis_server::v1::ResearchServices;
use ignis_server::{app, fixture, ServeArgs, Services};
use serde_json::Value;
use tower::ServiceExt;

mod support;

const PDF: &str = "sha256:5ffa911fd9b6356f204646003fb9342e341810dcdfdf5cb15756e7f01ec5f770"; // PSI-25 report
const CSV_FLEX2: &str = "sha256:cc57ad1bd9977aa13e7ac6c3aa618303012686e232b36c052c6a9f1a74e5f528"; // PSI-68
const TS: &str = "sha256:a9585e18f0f87f9c8c3c75e8603342ce19b4c72944ebd69dd91e1e89a910cc16"; // PSI-159 .ts (listing id)
const TS_SIZE: u64 = 14_767_400;
const NTRS_PDF: &str = "sha256:d1959a93556199928fb62d6239560568989b8072eb6178b70e10caae6c173c16";
const PERMANENT: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000001";
const ZENODO: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000002";
const PUBLIC: &str = "https://ignis.example.org";

fn data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/sources")
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// geode stand-in: counts calls, records request targets, answers with the measured body (fresh
/// `Expires`); a file named `…Engineering…` gets a URL outside the PSI buckets.
struct Geode {
    url: String,
    calls: Arc<AtomicUsize>,
    targets: Arc<Mutex<Vec<String>>>,
}

fn geode() -> Geode {
    let sample = std::fs::read_to_string(data().join("geode_redirect_false.txt")).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let calls = Arc::new(AtomicUsize::new(0));
    let targets = Arc::new(Mutex::new(Vec::new()));
    let (c, t) = (calls.clone(), targets.clone());
    std::thread::spawn(move || {
        for mut s in listener.incoming().flatten() {
            let mut buf = [0u8; 8192];
            let n = s.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let target = req
                .split_whitespace()
                .nth(1)
                .unwrap_or_default()
                .to_string();
            c.fetch_add(1, Ordering::SeqCst);
            t.lock().unwrap().push(target.clone());
            let fresh = format!("Expires={}", now() + 3600);
            let body = if target.contains("Engineering") {
                format!("https://evil.example.org/x?{fresh}")
            } else {
                sample.replace("Expires=1794698040", &fresh)
            };
            let _ = write!(
                s,
                "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    Geode {
        url: format!("http://127.0.0.1:{port}/geode-py/ws"),
        calls,
        targets,
    }
}

fn source_args(geode: &str, public: Option<&str>) -> SourceArgs {
    SourceArgs {
        manifests: data().join("manifests"),
        media_index: data().join("keyframes"),
        public_url: public.map(String::from),
        geode_url: geode.into(),
        margin_s: 300,
        geode_concurrency: 2,
    }
}

fn router(dir: &Path, sources: SourceArgs) -> axum::Router {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let args = ServeArgs {
        port: 0,
        host: "127.0.0.1".into(),
        root: crate_dir.join("../.."),
        data: dir.join("no-registry"),
        cohorts: dir.join("no-cohorts"),
        splits: dir.join("no-splits"),
        raw_samples: dir.join("no-raw"),
        artifacts: dir.join("artifacts"),
        evidence_db: dir.join("none.sqlite"),
        compute_threads: 1,
        dev: false,
        fixtures: crate_dir.join("fixtures"),
        web_dist: dir.join("no-dist"),
        state_dir: dir.join("state"),
        auth: AuthMode::Local,
        internal_routes: false,
        enable_models: false,
        max_upload_bytes: 1 << 20,
        contract_fixtures: "fixtures/contracts".into(),
        codex: Default::default(),
        mcp_port: 0,
        runner: Default::default(),
        artifact_backend: Default::default(),
        r2: Default::default(),
        access: support::access().args.clone(),
        sources,
    };
    app(
        args,
        Services {
            experts: Vec::new(),
            fixture_experts: fixture::experts(),
            acquisition: None,
            research: ResearchServices::default(),
        },
    )
}

async fn get(r: &axum::Router, path: &str) -> (StatusCode, Vec<(String, String)>, String) {
    let resp = r
        .clone()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let st = resp.status();
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 24)
        .await
        .unwrap();
    (st, headers, String::from_utf8(bytes.to_vec()).unwrap())
}

fn header<'a>(h: &'a [(String, String)], name: &str) -> Option<&'a str> {
    h.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
}

async fn access(r: &axum::Router, id: &str) -> SourceAccess {
    let (st, h, body) = get(r, &format!("/api/v1/sources/{id}/content")).await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert_eq!(header(&h, "cache-control"), Some("private, no-store"));
    let a: SourceAccess = serde_json::from_str(&body).unwrap();
    a.validate().unwrap();
    assert_eq!(a.source_id.as_str(), id);
    a
}

#[tokio::test]
async fn psi_file_gets_a_fresh_signed_url_resolved_once() {
    let g = geode();
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), source_args(&g.url, Some(PUBLIC)));

    let a = access(&r, PDF).await;
    assert_eq!(a.mode, AccessMode::Direct);
    assert!(
        a.url.starts_with("https://psi-repo-prod.s3.amazonaws.com/"),
        "{}",
        a.url
    );
    assert!(a.range && a.cors);
    assert_eq!(
        a.landing_url,
        "https://psi.nasa.gov/physci/repo/data/investigations/PSI-25"
    );
    assert_eq!(a.origin, Origin::System);
    let exp = chrono::DateTime::parse_from_rfc3339(a.expires_at.as_ref().unwrap().as_str())
        .unwrap()
        .timestamp();
    assert!(
        (exp - (now() + 3600)).abs() < 30,
        "expiry comes from the signed URL"
    );
    // geode was asked for exactly this file version, without following the redirect.
    let t = g.targets.lock().unwrap().clone();
    assert_eq!(t.len(), 1);
    assert!(t[0].starts_with("/geode-py/ws/studies/PSI-25/download?file=PSI-25_Reports_BASS-II%20Summary%20Report.pdf&version="));
    assert!(t[0].ends_with("&redirect=false"));

    // Cached until expiry minus the margin: same URL, no second geode call.
    let again = access(&r, PDF).await;
    assert_eq!(again.url, a.url);
    assert_eq!(g.calls.load(Ordering::SeqCst), 1);
    // `fresh` (the client saw a 403) re-resolves once.
    let (st, _, _) = get(&r, &format!("/api/v1/sources/{PDF}/content?fresh=true")).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(g.calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn ts_video_plays_from_a_keyframe_aligned_byte_range_playlist() {
    let g = geode();
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), source_args(&g.url, Some(PUBLIC)));

    let a = access(&r, TS).await;
    assert_eq!(a.mode, AccessMode::Direct);
    assert_eq!(a.url, format!("{PUBLIC}/api/v1/sources/{TS}/playlist.m3u8"));
    assert!(
        a.cors && !a.range,
        "the playlist is fetched whole; its segments use Range"
    );
    assert!(a.expires_at.is_some(), "segments are signed URLs");

    let (st, h, m3u8) = get(&r, &format!("/api/v1/sources/{TS}/playlist.m3u8")).await;
    assert_eq!(st, StatusCode::OK, "{m3u8}");
    assert_eq!(
        header(&h, "content-type"),
        Some("application/vnd.apple.mpegurl")
    );
    assert_eq!(header(&h, "cache-control"), Some("private, no-store"));
    // One geode call served both the access and the playlist (same cached URL).
    assert_eq!(g.calls.load(Ordering::SeqCst), 1);

    let index: KeyframeIndex = serde_json::from_slice(
        &std::fs::read(data().join(format!("keyframes/{}.json", TS.replace(':', "-")))).unwrap(),
    )
    .unwrap();
    check_playlist(&m3u8, &index);
    // Every segment streams from NASA's bucket; nothing points at IGNIS.
    assert!(m3u8.lines().filter(|l| !l.starts_with('#')).all(|l| l
        .starts_with("https://psi-repo-prod.s3.amazonaws.com/")
        && l.contains("Expires=")));
}

/// RFC 8216 media-playlist properties hls.js relies on to seek.
fn check_playlist(m3u8: &str, index: &KeyframeIndex) {
    let lines: Vec<&str> = m3u8.lines().collect();
    assert_eq!(lines[0], "#EXTM3U");
    assert!(lines.contains(&"#EXT-X-VERSION:4"));
    assert!(lines.contains(&"#EXT-X-PLAYLIST-TYPE:VOD"));
    assert_eq!(*lines.last().unwrap(), "#EXT-X-ENDLIST");
    let target: f64 = lines
        .iter()
        .find_map(|l| l.strip_prefix("#EXT-X-TARGETDURATION:"))
        .unwrap()
        .parse()
        .unwrap();
    let offsets: BTreeSet<u64> = index.keyframes.iter().map(|k| k.offset).collect();
    let (mut next_byte, mut total) = (0u64, 0.0);
    let mut segs = 0;
    for w in lines.windows(3) {
        let Some(d) = w[0].strip_prefix("#EXTINF:") else {
            continue;
        };
        let d: f64 = d.trim_end_matches(',').parse().unwrap();
        let (len, off) = w[1]
            .strip_prefix("#EXT-X-BYTERANGE:")
            .unwrap()
            .split_once('@')
            .unwrap();
        let (len, off): (u64, u64) = (len.parse().unwrap(), off.parse().unwrap());
        assert_eq!(off, next_byte, "segments are contiguous");
        assert!(
            off == 0 || offsets.contains(&off),
            "segment {segs} starts on a keyframe"
        );
        assert!(d.round() <= target && d > 0.0);
        next_byte = off + len;
        total += d;
        segs += 1;
    }
    assert_eq!(
        next_byte, index.size_bytes,
        "the playlist covers the whole object"
    );
    let span = index.end_pts_s - index.keyframes[0].pts_s;
    assert!(
        (total - span).abs() < 1e-3,
        "durations add up to the PTS span: {total} vs {span}"
    );
    assert!(segs > 1);
}

#[test]
fn playlist_builder_segments_on_keyframes_and_refuses_bad_indexes() {
    let index: KeyframeIndex = serde_json::from_slice(
        &std::fs::read(data().join(format!("keyframes/{}.json", TS.replace(':', "-")))).unwrap(),
    )
    .unwrap();
    let id = SourceId::new(TS).unwrap();
    index.validate(&id, TS_SIZE).unwrap();
    let url = "https://psi-repo-prod.s3.amazonaws.com/k.ts?Expires=1&Signature=x";
    let m = hls::playlist(&index, url, hls::DEFAULT_TARGET_S).unwrap();
    check_playlist(&m, &index);
    // ~8 s segments from 2.002 s GOPs: four GOPs each, the last one shorter.
    let segs = hls::segments(&index, 8.0);
    assert!(segs[..segs.len() - 1]
        .iter()
        .all(|s| (s.duration_s - 4.0 * 2.002).abs() < 1e-9));
    // A GOP longer than the target is one longer segment, never a split inside it.
    let long = hls::segments(&index, 1.0);
    assert_eq!(long.len(), index.keyframes.len());

    let bad = |f: &dyn Fn(&mut KeyframeIndex)| {
        let mut ix = index.clone();
        f(&mut ix);
        ix.validate(&id, TS_SIZE)
    };
    assert!(
        bad(&|ix| ix.keyframes[3].offset += 1).is_err(),
        "not packet aligned"
    );
    assert!(
        bad(&|ix| ix.keyframes.swap(2, 3)).is_err(),
        "not increasing"
    );
    assert!(bad(&|ix| ix.keyframes[4].pts_s = f64::NAN).is_err());
    assert!(bad(&|ix| ix.keyframes.clear()).is_err());
    assert!(bad(&|ix| ix.end_pts_s = ix.keyframes.last().unwrap().pts_s).is_err());
    assert!(index.validate(&id, TS_SIZE + 188).is_err(), "stale index");
    assert!(index
        .validate(&SourceId::new(PDF).unwrap(), TS_SIZE)
        .is_err());
    assert!(hls::playlist(&index, "https://x/a b", 8.0).is_err());
    assert!(hls::playlist(&index, "https://x/a\n#EXT-X-ENDLIST", 8.0).is_err());
}

#[tokio::test]
async fn ntrs_goes_through_the_worker_proxy_and_permanent_urls_pass_through() {
    let g = geode();
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), source_args(&g.url, Some(PUBLIC)));

    let a = access(&r, NTRS_PDF).await;
    assert_eq!(a.mode, AccessMode::Proxy);
    assert_eq!(
        a.url,
        format!("{PUBLIC}/ntrs/api/citations/20040142357/downloads/20040142357.pdf")
    );
    assert!(a.range && a.cors && a.expires_at.is_none());
    assert_eq!(a.landing_url, "https://ntrs.nasa.gov/citations/20040142357");

    // An NTRS record without a download opens at NTRS.
    let cat = Catalog::load(&data().join("manifests")).unwrap();
    assert!(cat.rejected.is_empty(), "{:?}", cat.rejected);
    let no_pdf = std::fs::read_to_string(data().join("manifests/ntrs/sample.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|v| v["record"]["pdf_url"].is_null())
        .unwrap();
    let a = access(&r, no_pdf["source"]["id"].as_str().unwrap()).await;
    assert_eq!(a.mode, AccessMode::LinkOut);
    assert_eq!(a.url, a.landing_url);

    let a = access(&r, PERMANENT).await;
    assert_eq!(a.mode, AccessMode::Direct);
    assert_eq!(
        a.url,
        "https://images-assets.nasa.gov/video/ignis-test/ignis-test~orig.mp4"
    );
    assert!(a.range && a.cors && a.expires_at.is_none());

    let a = access(&r, ZENODO).await;
    assert_eq!(a.mode, AccessMode::LinkOut);
    assert!(!a.range && !a.cors);
    assert_eq!(
        g.calls.load(Ordering::SeqCst),
        0,
        "no geode call for non-PSI sources"
    );
}

#[tokio::test]
async fn without_a_public_url_ntrs_links_out_and_ts_streams_without_playlist() {
    let g = geode();
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), source_args(&g.url, None));
    let a = access(&r, NTRS_PDF).await;
    assert_eq!(a.mode, AccessMode::LinkOut);
    let a = access(&r, TS).await;
    assert!(a.url.starts_with("https://psi-repo-prod.s3.amazonaws.com/") && a.range);
}

#[tokio::test]
async fn unknown_ids_and_bad_upstream_answers_fail_closed() {
    let g = geode();
    let dir = tempfile::tempdir().unwrap();
    let r = router(dir.path(), source_args(&g.url, Some(PUBLIC)));

    let unknown = "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    let (st, _, body) = get(&r, &format!("/api/v1/sources/{unknown}/content")).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert_eq!(
        serde_json::from_str::<Value>(&body).unwrap()["code"],
        "not_found"
    );
    let (st, _, _) = get(&r, &format!("/api/v1/sources/{unknown}/playlist.m3u8")).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    let (st, _, _) = get(
        &r,
        &format!("/api/v1/sources/{PDF}/content?scope=evaluation"),
    )
    .await;
    assert!(st.is_client_error(), "no request field can carry a scope");
    assert_eq!(g.calls.load(Ordering::SeqCst), 0);

    // A PDF is not a video: no playlist.
    let (st, _, _) = get(&r, &format!("/api/v1/sources/{PDF}/playlist.m3u8")).await;
    assert_eq!(st, StatusCode::NOT_IMPLEMENTED);

    // geode pointing outside the PSI buckets: refused, no URL handed out.
    let listing_pdf = Catalog::psi_listing_id(
        &PsiFile {
            investigation: "PSI-159".into(),
            file_name: "PSI-159_Engineering Documents_ACME Engineering Implementation Rpt..pdf"
                .into(),
            file_version: Some(1),
        },
        Some(7_496_324),
    );
    let (st, _, body) = get(&r, &format!("/api/v1/sources/{listing_pdf}/content")).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE, "{body}");
    let v: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["code"], "upstream_unavailable");
    assert!(!body.contains("evil.example.org/x?"));
}

/// Fake signer: counts calls, takes `delay`, signs for `ttl` past the margin; can fail first.
struct Fake {
    calls: AtomicUsize,
    delay: Duration,
    valid_s: i64,
    fail_first: bool,
}

impl Signer for Fake {
    fn sign(&self, f: &PsiFile) -> Result<Signed, ErrorEnvelope> {
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(self.delay);
        if self.fail_first && n == 0 {
            return Err(ErrorEnvelope::new(
                ErrorCode::UpstreamUnavailable,
                "geode down",
                RetryPolicy::Never,
            ));
        }
        Ok(Signed {
            url: format!(
                "https://psi-repo-prod.s3.amazonaws.com/{}?n={n}",
                f.file_name.replace(' ', "%20")
            ),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(self.valid_s),
        })
    }
}

fn file(name: &str) -> PsiFile {
    PsiFile {
        investigation: "PSI-159".into(),
        file_name: name.into(),
        file_version: Some(1),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn resolver_is_single_flight_with_ttl_below_expiry() {
    let margin = Duration::from_secs(300);
    // Valid for the margin + 1 s: cached for about one second.
    let fake = Arc::new(Fake {
        calls: AtomicUsize::new(0),
        delay: Duration::from_millis(200),
        valid_s: 301,
        fail_first: false,
    });
    let urls = Arc::new(SignedUrls::new(fake.clone(), margin, 100, 2));
    let tasks: Vec<_> = (0..16)
        .map(|_| {
            let u = urls.clone();
            tokio::spawn(async move { u.get(&file("a.ts"), false).await.unwrap() })
        })
        .collect();
    let got: Vec<_> = futures_join(tasks).await;
    assert_eq!(
        fake.calls.load(Ordering::SeqCst),
        1,
        "one call for 16 concurrent readers"
    );
    assert!(got.iter().all(|s| s.url == got[0].url));
    // Another file version is another key.
    urls.get(&file("b.ts"), false).await.unwrap();
    assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
    // Past expiry minus margin the entry is gone and the URL is re-resolved.
    tokio::time::sleep(Duration::from_millis(1300)).await;
    let later = urls.get(&file("a.ts"), false).await.unwrap();
    assert_ne!(later.url, got[0].url);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 3);

    // A URL already inside the margin is handed out but never cached.
    let short = Arc::new(Fake {
        calls: AtomicUsize::new(0),
        delay: Duration::ZERO,
        valid_s: 60,
        fail_first: false,
    });
    let urls = SignedUrls::new(short.clone(), margin, 100, 2);
    urls.get(&file("c.ts"), false).await.unwrap();
    urls.get(&file("c.ts"), false).await.unwrap();
    assert_eq!(short.calls.load(Ordering::SeqCst), 2);

    // Errors are not cached: the next request tries again.
    let flaky = Arc::new(Fake {
        calls: AtomicUsize::new(0),
        delay: Duration::ZERO,
        valid_s: 3600,
        fail_first: true,
    });
    let urls = SignedUrls::new(flaky.clone(), margin, 100, 2);
    let e = urls.get(&file("d.ts"), false).await.unwrap_err();
    assert_eq!(e.code, ErrorCode::UpstreamUnavailable);
    urls.get(&file("d.ts"), false).await.unwrap();
    urls.get(&file("d.ts"), false).await.unwrap();
    assert_eq!(flaky.calls.load(Ordering::SeqCst), 2);
}

async fn futures_join<T>(tasks: Vec<tokio::task::JoinHandle<T>>) -> Vec<T> {
    let mut out = Vec::new();
    for t in tasks {
        out.push(t.await.unwrap());
    }
    out
}

#[tokio::test]
async fn scope_is_host_derived_and_lockbox_originals_stay_closed_to_authoring() {
    let fake = Arc::new(Fake {
        calls: AtomicUsize::new(0),
        delay: Duration::ZERO,
        valid_s: 3600,
        fail_first: false,
    });
    let root = PathBuf::from("/");
    let d = SourceDelivery::with_signer(
        &source_args("http://unused", Some(PUBLIC)),
        &root,
        Origin::System,
        fake.clone(),
    );
    let research = research_scope("alice@example.org", false);
    let pdf = SourceId::new(PDF).unwrap();
    d.access(&research, &pdf, false).await.unwrap();

    // BASS-II (PSI-25) is a lockbox investigation: refused to an authoring scope.
    let authoring = ExecutionScope {
        role: ScopeRole::Authoring,
        ..research.clone()
    };
    let e = d.access(&authoring, &pdf, false).await.unwrap_err();
    assert_eq!(e.code, ErrorCode::ScopeDenied);
    let e = d
        .playlist(&authoring, &SourceId::new(TS).unwrap())
        .await
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::ScopeDenied);
    // FLEX-2 (PSI-68) is not: opened.
    d.access(&authoring, &SourceId::new(CSV_FLEX2).unwrap(), false)
        .await
        .unwrap();

    // No `read_sources`, no access.
    let mut none = research.clone();
    none.capabilities.remove(&Capability::ReadSources);
    let e = d.access(&none, &pdf, false).await.unwrap_err();
    assert_eq!(e.code, ErrorCode::ScopeDenied);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
}

/// The committed R6 manifests read completely: every line has an address or is refused loudly.
#[test]
fn repository_manifests_load_without_rejected_lines() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/manifests");
    let cat = Catalog::load(&root).unwrap();
    assert!(cat.rejected.is_empty(), "{:?}", cat.rejected);
    for id in [TS, PDF, CSV_FLEX2, NTRS_PDF] {
        assert!(cat.get(&SourceId::new(id).unwrap()).is_some(), "{id}");
    }
    assert!(cat.len() > 60_000, "{} sources", cat.len());
}

/// Live geode, opt-in (`IGNIS_LIVE_PSI=1`, `--ignored`): one `redirect=false` call, no bytes.
#[test]
#[ignore = "network: set IGNIS_LIVE_PSI=1"]
fn live_geode_signs_a_psi_file_for_about_an_hour() {
    if std::env::var("IGNIS_LIVE_PSI").as_deref() != Ok("1") {
        eprintln!("IGNIS_LIVE_PSI is not 1: skipped");
        return;
    }
    let g = GeodeSigner::new(ignis_data::sources::psi::BASE, Duration::from_secs(30));
    let s = g
        .sign(&file(
            "PSI-159_Raw Data_21130N1-vlc-record-2021-05-10-23h23m18s-udp___233.1.14.13_50400-.ts",
        ))
        .unwrap();
    let left = (s.expires_at - chrono::Utc::now()).num_minutes();
    assert!((30..=90).contains(&left), "signed for {left} min");
}

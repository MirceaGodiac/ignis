//! `--auth google`: the origin admits a request only with a Google-signed ID token for our client
//! id, a Google-verified e-mail and an allowlisted account, verified against the published keys
//! (a loopback stand-in here). Forged, expired, re-targeted, unverified, unlisted or unsigned
//! identities are refused; the workspace follows the verified e-mail only. CORS admits only the
//! configured UI origin. `--auth cloudflare-access` is covered by `tests/access.rs`, unchanged.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_server::v1::io::AuthMode;
use ignis_server::{app, fixture, ServeArgs, Services};
use serde_json::{json, Value};
use tower::ServiceExt;

mod support;
use support::{access, now, KID};

fn args(dir: &Path) -> ServeArgs {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ServeArgs {
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
        auth: AuthMode::Google,
        internal_routes: false,
        enable_models: true,
        max_upload_bytes: 1 << 20,
        contract_fixtures: "fixtures/contracts".into(),
        codex: Default::default(),
        mcp_port: 0,
        runner: Default::default(),
        artifact_backend: Default::default(),
        r2: Default::default(),
        access: Default::default(),
        google: access().google_args(),
    }
}

fn router(a: ServeArgs) -> axum::Router {
    app(
        a,
        Services {
            experts: Vec::new(),
            fixture_experts: fixture::experts(),
            acquisition: None,
            research: Default::default(),
        },
    )
}

async fn send(r: &axum::Router, req: Request<Body>) -> (StatusCode, Value) {
    let resp = r.clone().oneshot(req).await.unwrap();
    let st = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), 1 << 22)
        .await
        .unwrap();
    (st, serde_json::from_slice(&b).unwrap_or(Value::Null))
}

async fn get(r: &axum::Router, path: &str, headers: &[(&str, String)]) -> (StatusCode, Value) {
    let mut req = Request::get(path);
    for (k, v) in headers {
        req = req.header(*k, v);
    }
    send(r, req.body(Body::empty()).unwrap()).await
}

async fn create(r: &axum::Router, email: &str) -> Value {
    let mut req = Request::post("/api/v1/episodes").header("content-type", "application/json");
    for (k, v) in access().google_headers(email) {
        req = req.header(k, v);
    }
    let body = json!({"goal": "g", "client_request_id": "k"}).to_string();
    let (st, ep) = send(r, req.body(Body::from(body)).unwrap()).await;
    assert!(st.is_success(), "{st} {ep}");
    ep
}

#[tokio::test(flavor = "multi_thread")]
async fn only_verified_allowlisted_tokens_open_a_workspace() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(args(dir.path()));
    let a = access();
    let bearer = |c: Value| ("authorization", format!("Bearer {}", a.sign(&c)));
    let ana = || a.google_claims("ana@example.org");
    let refused = |what: &str, st: StatusCode, e: &Value| {
        assert_eq!(st, StatusCode::UNAUTHORIZED, "{what}: {e}");
        assert_eq!(e["code"], "scope_denied", "{what}");
    };

    // Liveness needs nothing and says nothing.
    let resp = r
        .clone()
        .oneshot(Request::get("/livez").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let with = |f: &dyn Fn(&mut Value)| {
        let mut c = ana();
        f(&mut c);
        vec![bearer(c)]
    };
    let cases: Vec<(&str, Vec<(&str, String)>)> = vec![
        ("nothing", vec![]),
        // Headers a proxy might set are never an identity.
        (
            "unsigned e-mail header",
            vec![("x-forwarded-email", "ana@example.org".into())],
        ),
        (
            "token without the Bearer scheme",
            vec![("authorization", a.sign(&ana()))],
        ),
        ("expired", with(&|c| c["exp"] = json!(now() - 3600))),
        (
            "other issuer",
            with(&|c| c["iss"] = json!("https://evil.example")),
        ),
        (
            "another app's client id",
            with(&|c| c["aud"] = json!("999-other.apps.googleusercontent.com")),
        ),
        (
            "e-mail not verified",
            with(&|c| c["email_verified"] = json!(false)),
        ),
        (
            "no e-mail",
            with(&|c| {
                c.as_object_mut().unwrap().remove("email");
            }),
        ),
        (
            "domain only in the address, not signed as hd",
            with(&|c| {
                c.as_object_mut().unwrap().remove("hd");
            }),
        ),
        (
            "unlisted account",
            vec![bearer(a.google_claims("eve@other.org"))],
        ),
        (
            "unlisted consumer account",
            vec![bearer(a.google_claims("bob@gmail.com"))],
        ),
        (
            "signed with an unpublished key under the real kid",
            vec![(
                "authorization",
                format!("Bearer {}", a.sign_with(&a.other, KID, &ana())),
            )],
        ),
        (
            "unknown kid",
            vec![(
                "authorization",
                format!("Bearer {}", a.sign_with(&a.other, "kid-nobody", &ana())),
            )],
        ),
        ("tampered claims", {
            let good = a.sign(&ana());
            let bob = a.sign(&a.google_claims("bob@example.org"));
            let (g, b): (Vec<&str>, Vec<&str>) =
                (good.split('.').collect(), bob.split('.').collect());
            vec![(
                "authorization",
                format!("Bearer {}.{}.{}", g[0], b[1], g[2]),
            )]
        }),
        ("garbage", vec![("authorization", "Bearer a.b.c".into())]),
    ];
    for (what, headers) in &cases {
        let (st, e) = get(&r, "/api/v1/episodes", headers).await;
        refused(what, st, &e);
    }

    // The legacy API needs a verified user too.
    let (st, e) = get(&r, "/api/health", &[]).await;
    refused("legacy without token", st, &e);
    let (st, _) = get(&r, "/api/health", &a.google_headers("ana@example.org")).await;
    assert_eq!(st, StatusCode::OK);

    // A listed consumer address is admitted without `hd`.
    let (st, e) = get(
        &r,
        "/api/v1/episodes",
        &a.google_headers(support::GOOGLE_SOLO),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{e}");

    // A verified user gets a workspace keyed by the e-mail (case-insensitive), and only that one.
    let ana = create(&r, "ana@example.org").await;
    let ana_again = create(&r, "Ana@Example.org").await;
    assert_eq!(
        ana["episode_id"], ana_again["episode_id"],
        "same principal, idempotent"
    );
    let bob = create(&r, "bob@example.org").await;
    assert_ne!(ana["workspace_id"], bob["workspace_id"]);
    let path = format!("/api/v1/episodes/{}", ana["episode_id"].as_str().unwrap());
    let (st, _) = get(&r, &path, &access().google_headers("ana@example.org")).await;
    assert_eq!(st, StatusCode::OK);
    let (st, _) = get(&r, &path, &access().google_headers("bob@example.org")).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "bob cannot see ana's episode");
}

#[tokio::test(flavor = "multi_thread")]
async fn cors_admits_only_the_configured_ui_origin() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(args(dir.path()));
    let preflight = |origin: &str| {
        Request::builder()
            .method("OPTIONS")
            .uri("/api/v1/episodes")
            .header("origin", origin)
            .header("access-control-request-method", "POST")
            .header(
                "access-control-request-headers",
                "authorization,content-type",
            )
            .body(Body::empty())
            .unwrap()
    };
    // The preflight carries no token and is answered before the identity check.
    let resp = r
        .clone()
        .oneshot(preflight(support::UI_ORIGIN))
        .await
        .unwrap();
    assert!(resp.status().is_success(), "{}", resp.status());
    let h = resp.headers();
    assert_eq!(h["access-control-allow-origin"], support::UI_ORIGIN);
    assert!(h["access-control-allow-headers"]
        .to_str()
        .unwrap()
        .contains("authorization"));
    assert!(h.get("access-control-allow-credentials").is_none());

    let resp = r
        .clone()
        .oneshot(preflight("https://evil.example"))
        .await
        .unwrap();
    assert!(resp.headers().get("access-control-allow-origin").is_none());

    // A refusal is still readable by the UI (so it can ask the user to sign in again).
    let resp = r
        .clone()
        .oneshot(
            Request::get("/api/v1/episodes")
                .header("origin", support::UI_ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp.headers()["access-control-allow-origin"],
        support::UI_ORIGIN
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_access_token_is_not_a_google_identity() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(args(dir.path()));
    let a = access();
    // A valid Cloudflare user assertion, presented as the bearer or in its own header.
    let cf = a.sign(&a.user_claims("ana@example.org"));
    for headers in [
        vec![("authorization", format!("Bearer {cf}"))],
        a.headers("ana@example.org").to_vec(),
    ] {
        let (st, e) = get(&r, "/api/v1/episodes", &headers).await;
        assert_eq!(st, StatusCode::UNAUTHORIZED, "{e}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn unreachable_keys_fail_closed_and_misconfiguration_does_not_start() {
    let dir = tempfile::tempdir().unwrap();
    let mut a = args(dir.path());
    // Nothing listens there: no key can be fetched, so nothing verifies (503, retryable).
    let dead = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    a.google.google_certs_url = Some(format!(
        "http://127.0.0.1:{}/certs",
        dead.local_addr().unwrap().port()
    ));
    drop(dead);
    let r = router(a);
    let (st, e) = get(
        &r,
        "/api/v1/episodes",
        &access().google_headers("ana@example.org"),
    )
    .await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE, "{e}");

    // No allowlist: the server refuses to start, and a router built anyway refuses everything.
    let mut a = args(dir.path());
    a.google.allowed_emails.clear();
    a.google.allowed_domains.clear();
    let err = tokio::task::spawn_blocking({
        let a = a.clone();
        move || {
            ignis_server::run(a, Services::default())
                .unwrap_err()
                .to_string()
        }
    })
    .await
    .unwrap();
    assert!(err.contains("--allowed-emails"), "{err}");
    let r = router(a);
    let headers = access().google_headers("ana@example.org");
    let (st, _) = get(&r, "/api/v1/episodes", &headers).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
    let (st, _) = get(&r, "/api/health", &headers).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
}

//! `--auth cloudflare-access`: the origin admits a request only with both Cloudflare-signed
//! assertions (the Worker's service token for the origin application, the user's own token for the
//! UI application), verified against the published keys. Forged, expired, swapped, re-targeted or
//! unsigned identities are refused; the workspace follows the verified e-mail only.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_server::v1::access::{ORIGIN_ASSERTION, USER_ASSERTION};
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
        auth: AuthMode::CloudflareAccess,
        internal_routes: false,
        max_upload_bytes: 1 << 20,
        contract_fixtures: "fixtures/contracts".into(),
        codex: Default::default(),
        mcp_port: 0,
        runner: Default::default(),
        access: access().args.clone(),
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
    for (k, v) in access().headers(email) {
        req = req.header(k, v);
    }
    let body = json!({"goal": "g", "client_request_id": "k"}).to_string();
    let (st, ep) = send(r, req.body(Body::from(body)).unwrap()).await;
    assert!(st.is_success(), "{st} {ep}");
    ep
}

#[tokio::test(flavor = "multi_thread")]
async fn only_verified_assertions_open_a_workspace() {
    let dir = tempfile::tempdir().unwrap();
    let r = router(args(dir.path()));
    let a = access();
    let origin = (ORIGIN_ASSERTION, a.sign(&a.origin_claims()));
    let user = |c: Value| (USER_ASSERTION, a.sign(&c));
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

    let cases: Vec<(&str, Vec<(&str, String)>)> = vec![
        ("nothing", vec![]),
        // The unsigned header Access also sets is never an identity.
        (
            "unsigned e-mail header",
            vec![(
                "cf-access-authenticated-user-email",
                "ana@example.org".into(),
            )],
        ),
        ("origin only", vec![origin.clone()]),
        ("user only", vec![user(a.user_claims("ana@example.org"))]),
        (
            "user token in the origin slot",
            vec![
                (ORIGIN_ASSERTION, a.sign(&a.user_claims("ana@example.org"))),
                user(a.user_claims("ana@example.org")),
            ],
        ),
        (
            "origin token in the user slot",
            vec![origin.clone(), (USER_ASSERTION, a.sign(&a.origin_claims()))],
        ),
        (
            "another service token",
            vec![
                (ORIGIN_ASSERTION, {
                    let mut c = a.origin_claims();
                    c["common_name"] = json!("someone-else.access");
                    a.sign(&c)
                }),
                user(a.user_claims("ana@example.org")),
            ],
        ),
        (
            "expired user token",
            vec![origin.clone(), {
                let mut c = a.user_claims("ana@example.org");
                c["exp"] = json!(now() - 3600);
                user(c)
            }],
        ),
        (
            "other issuer",
            vec![origin.clone(), {
                let mut c = a.user_claims("ana@example.org");
                c["iss"] = json!("https://evil.cloudflareaccess.com");
                user(c)
            }],
        ),
        (
            "no e-mail",
            vec![origin.clone(), {
                let mut c = a.user_claims("ana@example.org");
                c.as_object_mut().unwrap().remove("email");
                user(c)
            }],
        ),
        (
            "signed with an unpublished key under the real kid",
            vec![
                origin.clone(),
                (
                    USER_ASSERTION,
                    a.sign_with(&a.other, KID, &a.user_claims("ana@example.org")),
                ),
            ],
        ),
        (
            "unknown kid",
            vec![
                origin.clone(),
                (
                    USER_ASSERTION,
                    a.sign_with(&a.other, "kid-nobody", &a.user_claims("ana@example.org")),
                ),
            ],
        ),
        (
            "tampered claims",
            vec![origin.clone(), {
                let good = a.sign(&a.user_claims("ana@example.org"));
                let bob = a.sign(&a.user_claims("bob@example.org"));
                let (g, b): (Vec<&str>, Vec<&str>) =
                    (good.split('.').collect(), bob.split('.').collect());
                (USER_ASSERTION, format!("{}.{}.{}", g[0], b[1], g[2]))
            }],
        ),
        (
            "garbage",
            vec![
                (ORIGIN_ASSERTION, "a.b.c".into()),
                (USER_ASSERTION, "x".into()),
            ],
        ),
    ];
    for (what, headers) in &cases {
        let (st, e) = get(&r, "/api/v1/episodes", headers).await;
        refused(what, st, &e);
    }

    // The legacy API and static routes need the origin assertion too (no user identity there).
    let (st, e) = get(&r, "/api/health", &[]).await;
    refused("legacy without origin", st, &e);
    let (st, _) = get(&r, "/api/health", &[origin.clone()]).await;
    assert_eq!(st, StatusCode::OK);

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
    let (st, _) = get(&r, &path, &access().headers("ana@example.org")).await;
    assert_eq!(st, StatusCode::OK);
    let (st, _) = get(&r, &path, &access().headers("bob@example.org")).await;
    assert_eq!(st, StatusCode::NOT_FOUND, "bob cannot see ana's episode");
}

#[tokio::test(flavor = "multi_thread")]
async fn unreachable_keys_fail_closed_and_misconfiguration_does_not_start() {
    let dir = tempfile::tempdir().unwrap();
    let mut a = args(dir.path());
    // Nothing listens there: no key can be fetched, so nothing verifies (503, retryable).
    let dead = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    a.access.certs_url = Some(format!(
        "http://127.0.0.1:{}/certs",
        dead.local_addr().unwrap().port()
    ));
    drop(dead);
    let r = router(a);
    let (st, e) = get(&r, "/api/v1/episodes", &access().headers("ana@example.org")).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE, "{e}");

    // Incomplete configuration: the server refuses to start, and a router built anyway refuses all.
    let mut a = args(dir.path());
    a.access.service_token_id = None;
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
    assert!(err.contains("--access-service-token-id"), "{err}");
    let r = router(a);
    let (st, _) = get(&r, "/api/v1/episodes", &access().headers("ana@example.org")).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
    let (st, _) = get(&r, "/api/health", &access().headers("ana@example.org")).await;
    assert_eq!(st, StatusCode::SERVICE_UNAVAILABLE);
}

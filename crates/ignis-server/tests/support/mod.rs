//! Cloudflare Access stand-in for the integration tests: a loopback JWKS endpoint serving the public
//! half of a test-only RSA key (`tests/data/access_test_rsa2048.pkcs1.der`, never used anywhere else) and
//! RS256 tokens shaped like Access's (service-token assertion for the origin, user assertion for the
//! UI application).
#![allow(dead_code)]

use std::io::{Read, Write};
use std::sync::OnceLock;

use base64::Engine as _;
use ignis_server::v1::access::{AccessArgs, ORIGIN_ASSERTION, USER_ASSERTION};
use ring::rand::SystemRandom;
use ring::rsa::{KeyPair, PublicKeyComponents};
use serde_json::{json, Value};

pub const TEAM: &str = "https://ignis-test.cloudflareaccess.com";
pub const ORIGIN_AUD: &str = "aud-origin-0000";
pub const USER_AUD: &str = "aud-ui-1111";
pub const SERVICE_TOKEN_ID: &str = "worker-client.access";
pub const KID: &str = "kid-test-1";

pub struct TestAccess {
    pub args: AccessArgs,
    key: KeyPair,
    /// A second key that is not published (forgery attempts sign with it under the real kid).
    pub other: KeyPair,
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// One JWKS endpoint per test binary.
pub fn access() -> &'static TestAccess {
    static A: OnceLock<TestAccess> = OnceLock::new();
    A.get_or_init(|| {
        let key =
            KeyPair::from_der(include_bytes!("../data/access_test_rsa2048.pkcs1.der")).unwrap();
        let other =
            KeyPair::from_der(include_bytes!("../data/access_other_rsa2048.pkcs1.der")).unwrap();
        let pk: PublicKeyComponents<Vec<u8>> = key.public().into();
        let jwks = json!({"keys": [
            {"kty": "EC", "kid": "ec-ignored", "crv": "P-256", "x": "AA", "y": "AA"},
            {"kty": "RSA", "kid": KID, "alg": "RS256", "use": "sig", "n": b64(&pk.n), "e": b64(&pk.e)},
        ]})
        .to_string();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for mut s in listener.incoming().flatten() {
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                let _ = write!(
                    s,
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    jwks.len(),
                    jwks
                );
            }
        });
        TestAccess {
            args: AccessArgs {
                team_domain: Some(TEAM.into()),
                origin_aud: Some(ORIGIN_AUD.into()),
                service_token_id: Some(SERVICE_TOKEN_ID.into()),
                user_aud: Some(USER_AUD.into()),
                certs_url: Some(format!("http://127.0.0.1:{port}/cdn-cgi/access/certs")),
            },
            key,
            other,
        }
    })
}

impl TestAccess {
    pub fn sign_with(&self, key: &KeyPair, kid: &str, claims: &Value) -> String {
        let h = b64(json!({"alg": "RS256", "kid": kid, "typ": "JWT"})
            .to_string()
            .as_bytes());
        let p = b64(claims.to_string().as_bytes());
        let input = format!("{h}.{p}");
        let mut sig = vec![0; key.public().modulus_len()];
        key.sign(
            &ring::signature::RSA_PKCS1_SHA256,
            &SystemRandom::new(),
            input.as_bytes(),
            &mut sig,
        )
        .unwrap();
        format!("{input}.{}", b64(&sig))
    }

    pub fn sign(&self, claims: &Value) -> String {
        self.sign_with(&self.key, KID, claims)
    }

    /// What Access puts on a request from the Worker's service token.
    pub fn origin_claims(&self) -> Value {
        json!({"iss": TEAM, "aud": [ORIGIN_AUD], "common_name": SERVICE_TOKEN_ID, "sub": "",
               "type": "app", "iat": now(), "exp": now() + 300})
    }

    /// What Access puts on a signed-in user's request to the UI.
    pub fn user_claims(&self, email: &str) -> Value {
        json!({"iss": TEAM, "aud": [USER_AUD], "email": email, "sub": "u-1", "type": "app",
               "iat": now(), "exp": now() + 300})
    }

    /// Both headers the Worker forwards for `email`.
    pub fn headers(&self, email: &str) -> [(&'static str, String); 2] {
        [
            (ORIGIN_ASSERTION, self.sign(&self.origin_claims())),
            (USER_ASSERTION, self.sign(&self.user_claims(email))),
        ]
    }
}

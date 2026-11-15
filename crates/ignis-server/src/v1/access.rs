//! Cloudflare Access verification at the origin (`--auth cloudflare-access`).
//!
//! Topology (`infra/README.md`): the browser reaches the Worker through an Access application on the
//! UI hostname; the Worker calls the tunnel hostname with its **service token**, through a second
//! Access application that admits only that token. A request reaching Axum therefore carries two
//! Cloudflare-signed RS256 JWTs, both verified here against the team's published keys:
//!
//! - `Cf-Access-Jwt-Assertion` — set by Access in front of the origin. `aud` = the origin app's AUD
//!   tag, `common_name` = the Worker's service-token client id. It proves the request came through
//!   Access and the Worker; it does not name a user.
//! - `X-Ignis-User-Assertion` — the user's own Access JWT, which the Worker copies from its incoming
//!   `Cf-Access-Jwt-Assertion` (it strips any client-supplied value). `aud` = the UI app's AUD tag,
//!   `email` = the user. Only this token decides the principal, hence the workspace.
//!
//! No header is trusted on its own: the unsigned `Cf-Access-Authenticated-User-Email` is ignored.
//! Keys come from `<team>/cdn-cgi/access/certs`; they are refetched when a token names an unknown
//! `kid` (rotation) or the set is older than [`KEYS_MAX_AGE`], at most once per [`REFETCH_GAP`],
//! and a refetch replaces the whole set so a withdrawn key stops verifying.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use serde::Deserialize;
use serde_json::Value;

/// Header Access sets on requests to the origin (the service-token JWT here).
pub const ORIGIN_ASSERTION: &str = "cf-access-jwt-assertion";
/// Header the Worker forwards the user's Access JWT in.
pub const USER_ASSERTION: &str = "x-ignis-user-assertion";

const KEYS_MAX_AGE: Duration = Duration::from_secs(3600);
const REFETCH_GAP: Duration = Duration::from_secs(30);
/// Clock skew tolerated on `exp` / `nbf`.
const LEEWAY_S: i64 = 30;

#[derive(clap::Args, Debug, Clone, Default)]
pub struct AccessArgs {
    /// Cloudflare Access team domain, e.g. `https://ignis.cloudflareaccess.com` (the JWT issuer).
    #[arg(long = "access-team-domain", env = "IGNIS_ACCESS_TEAM_DOMAIN")]
    pub team_domain: Option<String>,
    /// AUD tag of the Access application in front of the origin (tunnel) hostname.
    #[arg(long = "access-origin-aud", env = "IGNIS_ACCESS_ORIGIN_AUD")]
    pub origin_aud: Option<String>,
    /// Client id of the Worker's service token (`common_name` of the origin JWT).
    #[arg(
        long = "access-service-token-id",
        env = "IGNIS_ACCESS_SERVICE_TOKEN_ID"
    )]
    pub service_token_id: Option<String>,
    /// AUD tag of the Access application in front of the UI hostname (the user's JWT).
    #[arg(long = "access-user-aud", env = "IGNIS_ACCESS_USER_AUD")]
    pub user_aud: Option<String>,
    /// Override of the key endpoint (tests); defaults to `<team>/cdn-cgi/access/certs`.
    #[arg(long = "access-certs-url", env = "IGNIS_ACCESS_CERTS_URL", hide = true)]
    pub certs_url: Option<String>,
}

/// Why a request is not admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessError {
    /// No valid identity: 401, the user signs in again.
    Denied(String),
    /// The keys could not be obtained: 503, retryable.
    Unavailable(String),
}

#[derive(Debug, Clone)]
struct RsaKey {
    n: Vec<u8>,
    e: Vec<u8>,
}

#[derive(Default)]
struct Keys {
    by_kid: HashMap<String, RsaKey>,
    fetched_at: Option<Instant>,
    attempted_at: Option<Instant>,
}

pub struct AccessVerifier {
    issuer: String,
    certs_url: String,
    origin_aud: String,
    service_token_id: String,
    user_aud: String,
    keys: Arc<RwLock<Keys>>,
    /// One key fetch at a time: a request that finds the set missing its `kid` while a fetch is in
    /// flight waits for that fetch instead of being refused (a page fires several requests at once
    /// right after a restart, when the set is still empty).
    fetching: Arc<tokio::sync::Mutex<()>>,
}

impl std::fmt::Debug for AccessVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessVerifier")
            .field("issuer", &self.issuer)
            .finish_non_exhaustive()
    }
}

impl AccessVerifier {
    /// Validates the configuration; every field is required (no default trusts anything).
    pub fn new(a: &AccessArgs) -> Result<Self, String> {
        fn need(v: &Option<String>, flag: &str) -> Result<String, String> {
            v.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .ok_or_else(|| format!("--auth cloudflare-access needs {flag}"))
        }
        let issuer = need(&a.team_domain, "--access-team-domain")?
            .trim_end_matches('/')
            .to_string();
        if !issuer.starts_with("https://") {
            return Err(format!(
                "--access-team-domain must be an https:// URL, got {issuer}"
            ));
        }
        let v = AccessVerifier {
            certs_url: a
                .certs_url
                .clone()
                .unwrap_or_else(|| format!("{issuer}/cdn-cgi/access/certs")),
            origin_aud: need(&a.origin_aud, "--access-origin-aud")?,
            service_token_id: need(&a.service_token_id, "--access-service-token-id")?,
            user_aud: need(&a.user_aud, "--access-user-aud")?,
            issuer,
            keys: Default::default(),
            fetching: Default::default(),
        };
        if v.origin_aud == v.user_aud {
            return Err("origin and UI must be separate Access applications (distinct AUD tags): a user JWT must not open the origin directly".into());
        }
        Ok(v)
    }

    /// Verify the origin assertion (request came through Access with the Worker's service token).
    pub async fn check_origin(&self, origin_jwt: Option<&str>) -> Result<(), AccessError> {
        let jwt = origin_jwt.ok_or_else(|| denied("missing Access assertion"))?;
        let claims = self.verify(jwt, &self.origin_aud).await?;
        match claims.get("common_name").and_then(Value::as_str) {
            Some(cn) if cn == self.service_token_id => Ok(()),
            _ => Err(denied("origin assertion is not the Worker's service token")),
        }
    }

    /// Verify both assertions and return the principal key (the user's e-mail, lowercased).
    pub async fn identify(
        &self,
        origin_jwt: Option<&str>,
        user_jwt: Option<&str>,
    ) -> Result<String, AccessError> {
        self.check_origin(origin_jwt).await?;
        let jwt = user_jwt.ok_or_else(|| denied("missing user assertion"))?;
        let claims = self.verify(jwt, &self.user_aud).await?;
        claims
            .get("email")
            .and_then(Value::as_str)
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty() && s.contains('@'))
            .ok_or_else(|| denied("user assertion has no e-mail identity"))
    }

    async fn verify(
        &self,
        jwt: &str,
        aud: &str,
    ) -> Result<serde_json::Map<String, Value>, AccessError> {
        let t = Token::parse(jwt)?;
        let key = self.key(&t.kid).await?;
        ring::signature::RsaPublicKeyComponents {
            n: &key.n,
            e: &key.e,
        }
        .verify(
            &ring::signature::RSA_PKCS1_2048_8192_SHA256,
            t.signing_input.as_bytes(),
            &t.signature,
        )
        .map_err(|_| denied("bad signature"))?;
        check_claims(&t.claims, &self.issuer, aud, now_s())?;
        Ok(t.claims)
    }

    async fn key(&self, kid: &str) -> Result<RsaKey, AccessError> {
        let look = || {
            let k = self.keys.read().unwrap_or_else(|p| p.into_inner());
            (
                k.by_kid.get(kid).cloned(),
                k.fetched_at.is_none_or(|t| t.elapsed() > KEYS_MAX_AGE),
                k.attempted_at.is_none_or(|t| t.elapsed() > REFETCH_GAP),
            )
        };
        if let (Some(key), false, _) = look() {
            return Ok(key);
        }
        // Missing or stale: take the fetch slot, then look again (a fetch that finished while this
        // request waited may already hold the key, and then no second fetch is made).
        let _slot = self.fetching.lock().await;
        let (hit, stale, may_fetch) = look();
        if let (Some(key), false) = (&hit, stale) {
            return Ok(key.clone());
        }
        if (hit.is_none() || stale) && may_fetch {
            let url = self.certs_url.clone();
            let keys = self.keys.clone();
            {
                keys.write().unwrap_or_else(|p| p.into_inner()).attempted_at = Some(Instant::now());
            }
            let fetched = tokio::task::spawn_blocking(move || fetch_keys(&url))
                .await
                .map_err(|e| AccessError::Unavailable(format!("key fetch: {e}")))?;
            match fetched {
                Ok(set) => {
                    let mut k = keys.write().unwrap_or_else(|p| p.into_inner());
                    k.by_kid = set;
                    k.fetched_at = Some(Instant::now());
                    if let Some(key) = k.by_kid.get(kid) {
                        return Ok(key.clone());
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Cloudflare Access keys not fetched");
                    // A stale but known key still verifies until the endpoint answers again.
                    return hit.ok_or(AccessError::Unavailable(e));
                }
            }
            return Err(denied("unknown signing key"));
        }
        hit.ok_or_else(|| denied("unknown signing key"))
    }
}

fn denied(m: &str) -> AccessError {
    AccessError::Denied(m.to_string())
}

fn now_s() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

struct Token {
    kid: String,
    signing_input: String,
    signature: Vec<u8>,
    claims: serde_json::Map<String, Value>,
}

#[derive(Deserialize)]
struct Header {
    alg: String,
    kid: Option<String>,
}

impl Token {
    fn parse(jwt: &str) -> Result<Token, AccessError> {
        let b64 = &base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let mut parts = jwt.trim().split('.');
        let (Some(h), Some(p), Some(s), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(denied("malformed token"));
        };
        let header: Header = b64
            .decode(h)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .ok_or_else(|| denied("malformed token header"))?;
        // Fixed algorithm: never let the token choose (`none`, HS256 with the public key, …).
        if header.alg != "RS256" {
            return Err(denied("unexpected token algorithm"));
        }
        let claims = b64
            .decode(p)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .ok_or_else(|| denied("malformed token claims"))?;
        Ok(Token {
            kid: header.kid.ok_or_else(|| denied("token has no key id"))?,
            signing_input: format!("{h}.{p}"),
            signature: b64.decode(s).map_err(|_| denied("malformed signature"))?,
            claims,
        })
    }
}

fn check_claims(
    c: &serde_json::Map<String, Value>,
    issuer: &str,
    aud: &str,
    now: i64,
) -> Result<(), AccessError> {
    if c.get("iss").and_then(Value::as_str) != Some(issuer) {
        return Err(denied("wrong issuer"));
    }
    let aud_ok = match c.get("aud") {
        Some(Value::String(s)) => s == aud,
        Some(Value::Array(a)) => a.iter().any(|v| v.as_str() == Some(aud)),
        _ => false,
    };
    if !aud_ok {
        return Err(denied("token is for another application"));
    }
    match c.get("exp").and_then(Value::as_i64) {
        Some(exp) if now <= exp + LEEWAY_S => {}
        Some(_) => return Err(denied("token expired")),
        None => return Err(denied("token has no expiry")),
    }
    if let Some(nbf) = c.get("nbf").and_then(Value::as_i64) {
        if now + LEEWAY_S < nbf {
            return Err(denied("token not yet valid"));
        }
    }
    Ok(())
}

fn fetch_keys(url: &str) -> Result<HashMap<String, RsaKey>, String> {
    let body = ureq::get(url)
        .call()
        .map_err(|e| format!("GET {url}: {e}"))?
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("GET {url}: {e}"))?;
    parse_jwks(&body)
}

/// RSA keys of a JWKS document (`{"keys": [{kty, kid, n, e}, …]}`); other key types are skipped.
fn parse_jwks(body: &str) -> Result<HashMap<String, RsaKey>, String> {
    #[derive(Deserialize)]
    struct Jwk {
        kty: String,
        kid: Option<String>,
        n: Option<String>,
        e: Option<String>,
    }
    #[derive(Deserialize)]
    struct Jwks {
        keys: Vec<Jwk>,
    }
    let set: Jwks = serde_json::from_str(body).map_err(|e| format!("JWKS: {e}"))?;
    let b64 = &base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let mut out = HashMap::new();
    for k in set.keys.into_iter().filter(|k| k.kty == "RSA") {
        let (Some(kid), Some(n), Some(e)) = (k.kid, k.n, k.e) else {
            continue;
        };
        let (Ok(n), Ok(e)) = (
            b64.decode(n.trim_end_matches('=')),
            b64.decode(e.trim_end_matches('=')),
        ) else {
            return Err(format!("JWKS key {kid}: bad base64url"));
        };
        out.insert(kid, RsaKey { n, e });
    }
    if out.is_empty() {
        return Err("JWKS has no RSA keys".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims(v: Value) -> serde_json::Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn claims_are_checked_strictly() {
        let iss = "https://t.cloudflareaccess.com";
        let ok = |c: Value| check_claims(&claims(c), iss, "A", 1000);
        assert!(ok(serde_json::json!({"iss": iss, "aud": ["A"], "exp": 1100})).is_ok());
        assert!(
            ok(serde_json::json!({"iss": iss, "aud": "A", "exp": 990})).is_ok(),
            "leeway"
        );
        assert!(ok(serde_json::json!({"iss": iss, "aud": ["B"], "exp": 1100})).is_err());
        assert!(ok(serde_json::json!({"iss": "https://evil", "aud": ["A"], "exp": 1100})).is_err());
        assert!(ok(serde_json::json!({"iss": iss, "aud": ["A"], "exp": 900})).is_err());
        assert!(ok(serde_json::json!({"iss": iss, "aud": ["A"]})).is_err());
        assert!(
            ok(serde_json::json!({"iss": iss, "aud": ["A"], "exp": 1100, "nbf": 1100})).is_err()
        );
    }

    #[test]
    fn only_rs256_tokens_parse() {
        let b64 = &base64::engine::general_purpose::URL_SAFE_NO_PAD;
        for alg in ["none", "HS256", "RS512"] {
            let h = b64.encode(format!(r#"{{"alg":"{alg}","kid":"k"}}"#));
            let p = b64.encode("{}");
            assert!(Token::parse(&format!("{h}.{p}.")).is_err(), "{alg}");
        }
    }

    #[test]
    fn config_needs_every_field_and_distinct_apps() {
        let full = AccessArgs {
            team_domain: Some("https://t.cloudflareaccess.com/".into()),
            origin_aud: Some("o".into()),
            service_token_id: Some("id.access".into()),
            user_aud: Some("u".into()),
            certs_url: None,
        };
        let v = AccessVerifier::new(&full).unwrap();
        assert_eq!(v.issuer, "https://t.cloudflareaccess.com");
        assert_eq!(
            v.certs_url,
            "https://t.cloudflareaccess.com/cdn-cgi/access/certs"
        );
        for broken in [
            AccessArgs {
                team_domain: None,
                ..full.clone()
            },
            AccessArgs {
                team_domain: Some("http://t".into()),
                ..full.clone()
            },
            AccessArgs {
                service_token_id: Some(" ".into()),
                ..full.clone()
            },
            AccessArgs {
                user_aud: Some("o".into()),
                ..full.clone()
            },
        ] {
            assert!(AccessVerifier::new(&broken).is_err(), "{broken:?}");
        }
    }
}

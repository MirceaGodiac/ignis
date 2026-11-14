//! Google sign-in verification at the origin (`--auth google`), the alternative to Cloudflare
//! Access ([`super::access`]) for a static UI hosted elsewhere (Vercel).
//!
//! Topology (`infra/README.md` §Google sign-in): the browser signs the user in with Google Identity
//! Services and calls `https://<api-hostname>/api/*` directly with the Google ID token as
//! `Authorization: Bearer <jwt>`. A TLS proxy (Caddy) sits in front of Axum and nothing else, so
//! every route except `/livez` goes through this check.
//!
//! The token is an RS256 JWT signed by Google, verified against Google's published keys (same key
//! cache as Access): `iss` is Google, `aud` is our OAuth client id, `exp`/`nbf` hold,
//! `email_verified` is true, and the account is on the allowlist: an exact e-mail, or a Google
//! Workspace domain matched on the signed `hd` claim, never on the address suffix alone (a consumer
//! Google account can carry any address). The verified e-mail (lowercased) is the principal, hence
//! the workspace. No other header is an identity.

use serde_json::Value;

use super::access::{denied, AccessError, KeySet};

/// Google's published signing keys (JWKS).
pub const GOOGLE_CERTS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
/// The two issuer spellings Google uses in ID tokens.
pub const GOOGLE_ISSUERS: [&str; 2] = ["https://accounts.google.com", "accounts.google.com"];

#[derive(clap::Args, Debug, Clone, Default)]
pub struct GoogleArgs {
    /// OAuth 2.0 web client id of the UI (`<n>.apps.googleusercontent.com`): the ID token's `aud`.
    #[arg(long = "google-client-id", env = "IGNIS_GOOGLE_CLIENT_ID")]
    pub google_client_id: Option<String>,
    /// Google accounts admitted by exact e-mail (comma-separated).
    #[arg(
        long = "allowed-emails",
        env = "IGNIS_ALLOWED_EMAILS",
        value_delimiter = ','
    )]
    pub allowed_emails: Vec<String>,
    /// Google Workspace domains admitted, matched on the token's signed `hd` claim (comma-separated).
    #[arg(
        long = "allowed-domains",
        env = "IGNIS_ALLOWED_DOMAINS",
        value_delimiter = ','
    )]
    pub allowed_domains: Vec<String>,
    /// Exact browser origins allowed to call the API cross-origin (the UI's production URL, e.g.
    /// `https://ignis.vercel.app`; comma-separated, no wildcards). Empty = same-origin only.
    #[arg(
        long = "cors-origins",
        env = "IGNIS_CORS_ORIGINS",
        value_delimiter = ','
    )]
    pub cors_origins: Vec<String>,
    /// Override of the key endpoint (tests); defaults to [`GOOGLE_CERTS_URL`].
    #[arg(long = "google-certs-url", env = "IGNIS_GOOGLE_CERTS_URL", hide = true)]
    pub google_certs_url: Option<String>,
}

pub struct GoogleVerifier {
    client_id: String,
    emails: Vec<String>,
    domains: Vec<String>,
    keys: KeySet,
}

impl std::fmt::Debug for GoogleVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleVerifier")
            .field("client_id", &self.client_id)
            .finish_non_exhaustive()
    }
}

/// Trimmed, lowercased, empty entries dropped.
fn normalized(v: &[String]) -> Vec<String> {
    v.iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

impl GoogleVerifier {
    /// Validates the configuration. The client id and a non-empty allowlist are required: without
    /// an allowlist every Google account would get a workspace (and spend the model budget).
    pub fn new(a: &GoogleArgs) -> Result<Self, String> {
        let client_id = a
            .google_client_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or("--auth google needs --google-client-id")?
            .to_string();
        let emails = normalized(&a.allowed_emails);
        let domains = normalized(&a.allowed_domains);
        if emails.is_empty() && domains.is_empty() {
            return Err("--auth google needs --allowed-emails or --allowed-domains (an empty allowlist admits nobody)".into());
        }
        if let Some(bad) = emails.iter().find(|e| !e.contains('@')) {
            return Err(format!("--allowed-emails: {bad} is not an e-mail address"));
        }
        if let Some(bad) = domains.iter().find(|d| d.contains('@') || !d.contains('.')) {
            return Err(format!("--allowed-domains: {bad} is not a domain"));
        }
        Ok(GoogleVerifier {
            client_id,
            emails,
            domains,
            keys: KeySet::new(
                a.google_certs_url
                    .clone()
                    .unwrap_or_else(|| GOOGLE_CERTS_URL.to_string()),
            ),
        })
    }

    /// Verify an `Authorization: Bearer <Google ID token>` header value and return the principal
    /// key (the verified e-mail, lowercased).
    pub async fn identify(&self, authorization: Option<&str>) -> Result<String, AccessError> {
        let jwt = authorization
            .and_then(|h| {
                h.strip_prefix("Bearer ")
                    .or_else(|| h.strip_prefix("bearer "))
            })
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .ok_or_else(|| denied("missing bearer token"))?;
        let claims = self
            .keys
            .verify(jwt, &GOOGLE_ISSUERS, &self.client_id)
            .await?;
        admit(&claims, &self.emails, &self.domains)
    }
}

/// The allowlist decision over verified claims: a Google-verified e-mail that is listed, or whose
/// signed Workspace domain (`hd`) is listed.
fn admit(
    c: &serde_json::Map<String, Value>,
    emails: &[String],
    domains: &[String],
) -> Result<String, AccessError> {
    let email = c
        .get("email")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && s.contains('@'))
        .ok_or_else(|| denied("token has no e-mail identity"))?;
    // Google sends a JSON boolean; older tokens used the string "true".
    let verified = match c.get("email_verified") {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => s == "true",
        _ => false,
    };
    if !verified {
        return Err(denied("e-mail address is not verified by Google"));
    }
    let hd = c
        .get("hd")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_lowercase());
    let listed = emails.contains(&email)
        || hd
            .as_ref()
            .is_some_and(|hd| domains.contains(hd) && email.ends_with(&format!("@{hd}")));
    if !listed {
        return Err(denied("this account is not on the allowlist"));
    }
    Ok(email)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn claims(v: Value) -> serde_json::Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn allowlist_needs_a_verified_listed_email_or_a_signed_workspace_domain() {
        let emails = vec!["ana@gmail.com".to_string()];
        let domains = vec!["uni.ro".to_string()];
        let admit = |c: Value| admit(&claims(c), &emails, &domains);
        assert_eq!(
            admit(json!({"email": "Ana@Gmail.com", "email_verified": true})).unwrap(),
            "ana@gmail.com"
        );
        assert!(admit(json!({"email": "ana@gmail.com", "email_verified": "true"})).is_ok());
        assert!(admit(json!({"email": "ana@gmail.com", "email_verified": false})).is_err());
        assert!(
            admit(json!({"email": "ana@gmail.com"})).is_err(),
            "unverified by omission"
        );
        assert!(admit(json!({"email": "bob@gmail.com", "email_verified": true})).is_err());
        assert!(admit(json!({"email_verified": true, "hd": "uni.ro"})).is_err());
        // Workspace domain: only on the signed `hd`, never on the address suffix alone.
        assert!(
            admit(json!({"email": "x@uni.ro", "email_verified": true, "hd": "uni.ro"})).is_ok()
        );
        assert!(admit(json!({"email": "x@uni.ro", "email_verified": true})).is_err());
        assert!(
            admit(json!({"email": "x@evil.ro", "email_verified": true, "hd": "uni.ro"})).is_err()
        );
        assert!(
            admit(json!({"email": "x@uni.ro", "email_verified": true, "hd": "evil.ro"})).is_err()
        );
    }

    #[test]
    fn config_needs_a_client_id_and_an_allowlist() {
        let full = GoogleArgs {
            google_client_id: Some("123.apps.googleusercontent.com".into()),
            allowed_emails: vec![" Ana@Gmail.com ".into()],
            ..Default::default()
        };
        let v = GoogleVerifier::new(&full).unwrap();
        assert_eq!(v.emails, ["ana@gmail.com"]);
        for broken in [
            GoogleArgs {
                google_client_id: None,
                ..full.clone()
            },
            GoogleArgs {
                google_client_id: Some(" ".into()),
                ..full.clone()
            },
            GoogleArgs {
                allowed_emails: vec!["".into()],
                ..full.clone()
            },
            GoogleArgs {
                allowed_emails: vec!["not-an-email".into()],
                ..full.clone()
            },
            GoogleArgs {
                allowed_emails: vec![],
                allowed_domains: vec!["@uni.ro".into()],
                ..full.clone()
            },
        ] {
            assert!(GoogleVerifier::new(&broken).is_err(), "{broken:?}");
        }
    }
}

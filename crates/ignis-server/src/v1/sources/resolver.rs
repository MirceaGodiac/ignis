//! Signed PSI URLs: geode `redirect=false` behind a single-flight cache.
//!
//! geode (`/studies/<ACC>/download?file=..&version=..&redirect=false`) answers `200` with the
//! signed S3 URL as the whole body (measured 2026-11-15; sample in
//! `tests/data/sources/geode_redirect_false.txt`, signature redacted). The URL carries its own
//! expiry (`Expires=<unix>` for SigV2, `X-Amz-Date` + `X-Amz-Expires` for SigV4), about one hour.
//!
//! The cache is keyed by file version (investigation, file name, file version): concurrent
//! requests for one file share one geode call, an entry lives until its URL's expiry minus a
//! margin, capacity is bounded (LRU-like eviction by moka), and geode calls are throttled because
//! the endpoint is undocumented. Errors are not cached.

use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDateTime, Utc};
use ignis_core::contracts::{ErrorCode, ErrorEnvelope, RetryPolicy};
use moka::future::Cache;
use moka::Expiry;
use tokio::sync::Semaphore;

use super::catalog::PsiFile;

/// A signed delivery URL and when it stops working.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signed {
    pub url: String,
    pub expires_at: DateTime<Utc>,
}

/// Turns a PSI file version into a signed URL (blocking; called off the reactor).
pub trait Signer: Send + Sync + 'static {
    fn sign(&self, file: &PsiFile) -> Result<Signed, ErrorEnvelope>;
}

fn upstream(msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(
        ErrorCode::UpstreamUnavailable,
        msg,
        RetryPolicy::AfterDelay { seconds: 30 },
    )
}

/// The S3 buckets PSI serves from (LANES_V2 §10). A signed URL anywhere else is refused.
const PSI_BUCKETS: [&str; 2] = ["psi-repo-prod", "psi-curation-prod"];

fn allowed_host(host: &str) -> bool {
    let Some(rest) = host.strip_suffix(".amazonaws.com") else {
        return false;
    };
    PSI_BUCKETS.iter().any(|b| {
        rest.strip_prefix(b)
            .and_then(|r| r.strip_prefix(".s3"))
            .is_some_and(|region| {
                // `.s3` or `.s3.<region>` / `.s3-<region>`
                region.is_empty()
                    || region.strip_prefix(['.', '-']).is_some_and(|r| {
                        !r.is_empty() && r.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                    })
            })
    })
}

fn query_param<'a>(url: &'a str, name: &str) -> Option<&'a str> {
    let q = url.split_once('?')?.1;
    q.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        k.eq_ignore_ascii_case(name).then_some(v)
    })
}

/// Check a geode answer and read the URL's expiry. Anything unexpected is an upstream error: a URL
/// is only handed out when it points at a PSI bucket and says when it expires.
pub fn parse_signed(body: &str) -> Result<Signed, ErrorEnvelope> {
    let url = body.trim();
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(upstream("geode answer is not a single URL"));
    }
    let rest = url
        .strip_prefix("https://")
        .ok_or_else(|| upstream("geode answer is not an https URL"))?;
    let host = &rest[..rest.find(['/', '?', '#']).unwrap_or(rest.len())];
    if !allowed_host(host) {
        return Err(upstream(format!(
            "geode pointed at {host:?}, not a PSI bucket"
        )));
    }
    let expires_at = if let Some(e) = query_param(url, "Expires") {
        e.parse::<i64>()
            .ok()
            .and_then(|s| DateTime::from_timestamp(s, 0))
    } else {
        query_param(url, "X-Amz-Date")
            .and_then(|d| NaiveDateTime::parse_from_str(d, "%Y%m%dT%H%M%SZ").ok())
            .zip(query_param(url, "X-Amz-Expires").and_then(|s| s.parse::<i64>().ok()))
            .map(|(d, s)| d.and_utc() + chrono::Duration::seconds(s))
    }
    .ok_or_else(|| upstream("signed URL carries no readable expiry"))?;
    Ok(Signed {
        url: url.to_string(),
        expires_at,
    })
}

/// The real signer: one GET to geode, no redirect followed, nothing downloaded.
pub struct GeodeSigner {
    base: String,
    agent: ureq::Agent,
}

impl GeodeSigner {
    /// `base` is the geode web-service root (`ignis_data::sources::psi::BASE` in production).
    pub fn new(base: impl Into<String>, timeout: Duration) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .user_agent(ignis_data::sources::psi::UA)
            .timeout_global(Some(timeout))
            .max_redirects(0)
            .http_status_as_error(false)
            .build()
            .into();
        GeodeSigner {
            base: base.into().trim_end_matches('/').to_string(),
            agent,
        }
    }

    fn url(&self, f: &PsiFile) -> String {
        // Same address as the stable identity URL, under the configured root, plus `redirect=false`.
        let stable = f.stable_url();
        let path = stable
            .strip_prefix(ignis_data::sources::psi::BASE)
            .unwrap_or(&stable);
        format!("{}{path}&redirect=false", self.base)
    }
}

impl Signer for GeodeSigner {
    fn sign(&self, f: &PsiFile) -> Result<Signed, ErrorEnvelope> {
        let mut resp = self
            .agent
            .get(&self.url(f))
            .call()
            .map_err(|e| upstream(format!("geode unreachable: {e}")))?;
        let status = resp.status().as_u16();
        match status {
            200 => {}
            // The id is in the manifests but NASA no longer has the object: retrying will not help.
            // A corpus-integrity problem (the manifest points at something missing): `source_gone`,
            // distinct from `not_found` (unknown id) and from `upstream_unavailable` (outage).
            404 | 410 => {
                tracing::error!(
                    investigation = %f.investigation,
                    file = %f.file_name,
                    version = ?f.file_version,
                    status,
                    "corpus integrity: manifest entry not served by PSI"
                );
                return Err(ErrorEnvelope::new(
                    ErrorCode::SourceGone,
                    format!(
                        "corpus integrity: the manifest lists {:?} version {:?}, but PSI no longer serves it (geode {status})",
                        f.file_name, f.file_version
                    ),
                    RetryPolicy::Never,
                ));
            }
            429 => {
                return Err(ErrorEnvelope::new(
                    ErrorCode::RateLimited,
                    "geode is rate limiting",
                    RetryPolicy::AfterDelay { seconds: 60 },
                ))
            }
            s => return Err(upstream(format!("geode answered {s}"))),
        }
        // The body is one URL (~2 KB); anything far larger is not the answer we expect.
        let body = resp
            .body_mut()
            .with_config()
            .limit(64 * 1024)
            .read_to_string()
            .map_err(|e| upstream(format!("geode answer unreadable: {e}")))?;
        parse_signed(&body)
    }
}

struct UntilExpiry {
    margin: chrono::Duration,
}

impl Expiry<PsiFile, Arc<Signed>> for UntilExpiry {
    fn expire_after_create(
        &self,
        _key: &PsiFile,
        value: &Arc<Signed>,
        _created_at: Instant,
    ) -> Option<Duration> {
        Some(
            (value.expires_at - self.margin - Utc::now())
                .to_std()
                .unwrap_or(Duration::ZERO),
        )
    }
}

/// A URL past its expiry is never handed out (nor cached): the caller gets `upstream_unavailable`.
fn live(s: Arc<Signed>) -> Result<Arc<Signed>, ErrorEnvelope> {
    if s.expires_at <= Utc::now() {
        return Err(upstream(format!(
            "signed URL already expired at {}",
            s.expires_at.to_rfc3339()
        )));
    }
    Ok(s)
}

/// Single-flight, TTL-bounded cache of signed URLs.
pub struct SignedUrls {
    signer: Arc<dyn Signer>,
    cache: Cache<PsiFile, Arc<Signed>>,
    throttle: Arc<Semaphore>,
}

impl SignedUrls {
    /// `margin`: an entry is dropped this long before its URL expires (5 min in production).
    /// `capacity`: entries kept (memory bound). `concurrency`: geode calls in flight at once.
    pub fn new(
        signer: Arc<dyn Signer>,
        margin: Duration,
        capacity: u64,
        concurrency: usize,
    ) -> Self {
        let margin = chrono::Duration::from_std(margin).unwrap_or(chrono::Duration::zero());
        SignedUrls {
            signer,
            cache: Cache::builder()
                .max_capacity(capacity)
                .expire_after(UntilExpiry { margin })
                .build(),
            throttle: Arc::new(Semaphore::new(concurrency.max(1))),
        }
    }

    /// The cached URL of `file`, or one geode call shared by every concurrent caller.
    /// `fresh` drops the cached entry first (the client saw a 403 on it).
    pub async fn get(&self, file: &PsiFile, fresh: bool) -> Result<Arc<Signed>, ErrorEnvelope> {
        if fresh {
            self.cache.invalidate(file).await;
        }
        let (signer, throttle, f) = (self.signer.clone(), self.throttle.clone(), file.clone());
        self.cache
            .try_get_with(file.clone(), async move {
                let _permit = throttle
                    .acquire_owned()
                    .await
                    .map_err(|_| upstream("resolver closed"))?;
                tokio::task::spawn_blocking(move || signer.sign(&f))
                    .await
                    .map_err(|e| {
                        ErrorEnvelope::new(
                            ErrorCode::Internal,
                            format!("signer task: {e}"),
                            RetryPolicy::Never,
                        )
                    })?
                    .and_then(|s| live(Arc::new(s)))
            })
            .await
            .map_err(|e: Arc<ErrorEnvelope>| (*e).clone())
            .and_then(live)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_measured_answer_and_refuses_foreign_or_undated_urls() {
        let body = include_str!("../../../tests/data/sources/geode_redirect_false.txt");
        let s = parse_signed(body).unwrap();
        assert!(s
            .url
            .starts_with("https://psi-repo-prod.s3.amazonaws.com/psi-data/PSI-159/"));
        assert_eq!(s.expires_at.timestamp(), 1_794_698_040);

        let v4 = "https://psi-curation-prod.s3.us-west-2.amazonaws.com/k?X-Amz-Date=20261115T100000Z&X-Amz-Expires=3600&X-Amz-Signature=x";
        assert_eq!(
            parse_signed(v4).unwrap().expires_at.to_rfc3339(),
            "2026-11-15T11:00:00+00:00"
        );
        for bad in [
            "https://evil.example.org/x?Expires=1794698040",
            "https://psi-repo-prod.s3.amazonaws.com.evil.org/x?Expires=1",
            "https://psi-repo-prod-x.s3.amazonaws.com/x?Expires=1",
            "http://psi-repo-prod.s3.amazonaws.com/x?Expires=1794698040",
            "https://psi-repo-prod.s3.amazonaws.com/x",
            "<html>error</html>",
            "https://psi-repo-prod.s3.amazonaws.com/x?Expires=1\nhttps://evil",
        ] {
            assert!(parse_signed(bad).is_err(), "{bad}");
        }
    }
}

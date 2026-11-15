//! The guarded HTTP fetcher of `web_fetch`: http(s) only, no credentials in the URL, no proxy, every
//! resolved address checked (private, loopback, link-local/metadata, multicast and reserved ranges
//! are refused) by the resolver the connection itself uses, so the address that is checked is the
//! address that is dialled (no DNS-rebinding gap). Redirects are followed by hand, one hop at a time,
//! each hop re-checked, up to [`FetchPolicy::max_redirects`]. The body is read up to
//! [`FetchPolicy::max_bytes`]; a larger one is refused, never truncated into a snapshot.

use std::collections::BTreeSet;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use ignis_core::contracts::{ErrorCode, ErrorEnvelope, RetryPolicy};
use ureq::unversioned::resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};

/// Security and resource budgets of one fetch. These are limits against a hostile or broken
/// server (memory, time, internal-network reach), not caps on what research may read.
#[derive(Debug, Clone)]
pub struct FetchPolicy {
    pub max_redirects: u32,
    pub max_bytes: u64,
    pub connect_timeout: Duration,
    /// Whole request (all hops, body included).
    pub total_timeout: Duration,
    /// Exact `ip:port` pairs exempt from the address check. Empty in every deployment; tests use it
    /// to reach their own loopback fake server while every other loopback address stays refused.
    pub allow_addrs: BTreeSet<SocketAddr>,
}

impl Default for FetchPolicy {
    fn default() -> Self {
        FetchPolicy {
            max_redirects: 5,
            max_bytes: 32 << 20,
            connect_timeout: Duration::from_secs(10),
            total_timeout: Duration::from_secs(60),
            allow_addrs: BTreeSet::new(),
        }
    }
}

/// What one fetch returned (status 2xx only; anything else is an error).
#[derive(Debug, Clone)]
pub struct Fetched {
    pub requested_url: String,
    /// Every hop after the first (`Location` targets, resolved), in order.
    pub redirects: Vec<String>,
    pub final_url: String,
    pub status: u16,
    /// `Content-Type` as sent (parameters included), when sent.
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

fn refuse(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

/// True for addresses a fetch must never reach: unspecified, loopback, private (RFC 1918, ULA),
/// shared (CGNAT), link-local (cloud metadata lives at 169.254.169.254), documentation, benchmarking,
/// multicast, broadcast and reserved ranges; IPv4 embedded in IPv6 (mapped, compatible, NAT64, 6to4)
/// is checked as the IPv4 it carries.
pub fn blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => blocked_v4(v4),
        IpAddr::V6(v6) => blocked_v6(v6),
    }
}

fn blocked_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_multicast()
        || ip.is_documentation()
        || a == 0
        || (a == 100 && (64..128).contains(&b)) // shared address space (CGNAT)
        || (a == 192 && b == 0 && c == 0) // IETF protocol assignments
        || (a == 198 && (b == 18 || b == 19)) // benchmarking
        || a >= 240 // reserved
}

fn blocked_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return blocked_v4(v4);
    }
    let s = ip.segments();
    let embedded_v4 = |hi: u16, lo: u16| Ipv4Addr::from(((hi as u32) << 16) | lo as u32);
    ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_multicast()
        || (s[0] & 0xfe00) == 0xfc00 // unique local
        || (s[0] & 0xffc0) == 0xfe80 // link-local
        || (s[0] & 0xffc0) == 0xfec0 // site-local (deprecated)
        || (s[0] == 0x2001 && s[1] == 0x0db8) // documentation
        || (s[..6] == [0; 6] && blocked_v4(embedded_v4(s[6], s[7]))) // IPv4-compatible
        || (s[0] == 0x0064 && s[1] == 0xff9b && blocked_v4(embedded_v4(s[6], s[7]))) // NAT64
        || (s[0] == 0x2002 && blocked_v4(embedded_v4(s[1], s[2]))) // 6to4
}

/// Resolves like ureq's default resolver, then refuses the whole name when any of its addresses is
/// blocked (fail closed: a name that also points inside is not fetched at all).
#[derive(Debug)]
struct GuardedResolver {
    inner: DefaultResolver,
    allow: Arc<BTreeSet<SocketAddr>>,
}

impl Resolver for GuardedResolver {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let addrs = self.inner.resolve(uri, config, timeout)?;
        if addrs.is_empty() {
            return Err(ureq::Error::HostNotFound);
        }
        for a in addrs.iter() {
            if blocked_ip(a.ip()) && !self.allow.contains(a) {
                return Err(ureq::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!(
                        "ssrf-blocked: {} resolves to {}, a non-public address",
                        host_of(uri),
                        a.ip()
                    ),
                )));
            }
        }
        Ok(addrs)
    }
}

fn host_of(uri: &ureq::http::Uri) -> String {
    uri.host().unwrap_or("?").to_string()
}

/// Scheme and authority checks before anything is resolved.
pub fn check_url(url: &str) -> Result<ureq::http::Uri, ErrorEnvelope> {
    let uri: ureq::http::Uri = url.parse().map_err(|e| {
        refuse(
            ErrorCode::InvalidRequest,
            format!("{url:?} is not a URL: {e}"),
        )
    })?;
    match uri.scheme_str() {
        Some("http") | Some("https") => {}
        _ => {
            return Err(refuse(
                ErrorCode::InvalidRequest,
                format!("{url:?}: only http and https are fetched"),
            ))
        }
    }
    let auth = uri
        .authority()
        .ok_or_else(|| refuse(ErrorCode::InvalidRequest, format!("{url:?} has no host")))?;
    if auth.as_str().contains('@') {
        return Err(refuse(
            ErrorCode::InvalidRequest,
            format!("{url:?}: credentials in a URL are never sent"),
        ));
    }
    if auth.host().is_empty() {
        return Err(refuse(
            ErrorCode::InvalidRequest,
            format!("{url:?} has no host"),
        ));
    }
    Ok(uri)
}

/// `location` relative to `base` (RFC 3986 reference forms that redirects use).
pub fn join(base: &ureq::http::Uri, location: &str) -> String {
    let loc = location.trim();
    if loc.contains("://") {
        return loc.to_string();
    }
    let scheme = base.scheme_str().unwrap_or("https");
    let auth = base.authority().map(|a| a.as_str()).unwrap_or_default();
    if let Some(rest) = loc.strip_prefix("//") {
        return format!("{scheme}://{rest}");
    }
    if loc.starts_with('/') {
        return format!("{scheme}://{auth}{loc}");
    }
    let path = base.path();
    let dir = &path[..path.rfind('/').map_or(0, |i| i + 1)];
    let dir = if dir.is_empty() { "/" } else { dir };
    if loc.starts_with('?') {
        return format!("{scheme}://{auth}{path}{loc}");
    }
    format!("{scheme}://{auth}{dir}{loc}")
}

pub struct Fetcher {
    agent: ureq::Agent,
    policy: FetchPolicy,
}

impl Fetcher {
    pub fn new(policy: FetchPolicy) -> Fetcher {
        let config = ureq::Agent::config_builder()
            .max_redirects(0)
            .http_status_as_error(false)
            // a proxy would be resolved and dialled instead of the target, bypassing the check
            .proxy(None)
            .timeout_connect(Some(policy.connect_timeout))
            .timeout_global(Some(policy.total_timeout))
            .user_agent(concat!("ignis-web-fetch/", env!("CARGO_PKG_VERSION")))
            .build();
        let resolver = GuardedResolver {
            inner: DefaultResolver::default(),
            allow: Arc::new(policy.allow_addrs.clone()),
        };
        Fetcher {
            agent: ureq::Agent::with_parts(config, DefaultConnector::new(), resolver),
            policy,
        }
    }

    pub fn policy(&self) -> &FetchPolicy {
        &self.policy
    }

    /// GET `url` under the policy. Blocking: call it off the async reactor.
    pub fn fetch(&self, url: &str) -> Result<Fetched, ErrorEnvelope> {
        let started = std::time::Instant::now();
        let mut current = url.to_string();
        let mut redirects = Vec::new();
        loop {
            let uri = check_url(&current)?;
            let remaining = self.policy.total_timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(ErrorEnvelope::new(
                    ErrorCode::UpstreamUnavailable,
                    format!("fetching {url:?} exceeded {:?}", self.policy.total_timeout),
                    RetryPolicy::AfterDelay { seconds: 60 },
                ));
            }
            let mut resp = self
                .agent
                .get(&current)
                .header(
                    "Accept",
                    "text/html, application/pdf, text/plain;q=0.9, */*;q=0.5",
                )
                .config()
                .timeout_global(Some(remaining))
                .build()
                .call()
                .map_err(|e| transport_error(&current, e))?;
            let status = resp.status().as_u16();
            if (300..400).contains(&status) && status != 304 {
                let loc = resp
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| {
                        ErrorEnvelope::new(
                            ErrorCode::UpstreamUnavailable,
                            format!("{current:?} answered {status} without a Location"),
                            RetryPolicy::Never,
                        )
                    })?;
                if redirects.len() as u32 >= self.policy.max_redirects {
                    return Err(refuse(
                        ErrorCode::InvalidRequest,
                        format!(
                            "{url:?} redirects more than {} times",
                            self.policy.max_redirects
                        ),
                    ));
                }
                current = join(&uri, loc);
                redirects.push(current.clone());
                continue;
            }
            if !(200..300).contains(&status) {
                let code = match status {
                    404 | 410 => ErrorCode::NotFound,
                    429 => ErrorCode::RateLimited,
                    401 | 403 => ErrorCode::ScopeDenied,
                    _ => ErrorCode::UpstreamUnavailable,
                };
                return Err(ErrorEnvelope::new(
                    code,
                    format!("{current:?} answered HTTP {status}; nothing was stored"),
                    RetryPolicy::Never,
                ));
            }
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            let max = self.policy.max_bytes;
            if let Some(len) = resp
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
            {
                if len > max {
                    return Err(too_large(&current, max));
                }
            }
            let mut bytes = Vec::new();
            resp.body_mut()
                .as_reader()
                .take(max + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| {
                    ErrorEnvelope::new(
                        ErrorCode::UpstreamUnavailable,
                        format!("reading {current:?}: {e}"),
                        RetryPolicy::AfterDelay { seconds: 60 },
                    )
                })?;
            if bytes.len() as u64 > max {
                return Err(too_large(&current, max));
            }
            return Ok(Fetched {
                requested_url: url.to_string(),
                redirects,
                final_url: current,
                status,
                content_type,
                bytes,
            });
        }
    }
}

fn too_large(url: &str, max: u64) -> ErrorEnvelope {
    refuse(
        ErrorCode::Unsupported,
        format!("{url:?} is larger than the fetch budget of {max} bytes; nothing was stored"),
    )
}

fn transport_error(url: &str, e: ureq::Error) -> ErrorEnvelope {
    let msg = e.to_string();
    if msg.contains("ssrf-blocked") {
        return refuse(
            ErrorCode::ScopeDenied,
            format!("{url:?} refused by the SSRF policy: {msg}"),
        );
    }
    match e {
        ureq::Error::HostNotFound => {
            refuse(ErrorCode::NotFound, format!("{url:?}: host not found"))
        }
        ureq::Error::Timeout(_) => ErrorEnvelope::new(
            ErrorCode::UpstreamUnavailable,
            format!("{url:?}: timed out"),
            RetryPolicy::AfterDelay { seconds: 60 },
        ),
        _ => ErrorEnvelope::new(
            ErrorCode::UpstreamUnavailable,
            format!("{url:?}: {msg}"),
            RetryPolicy::AfterDelay { seconds: 60 },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_ranges() {
        for ip in [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "255.255.255.255",
            "224.0.0.1",
            "::1",
            "::",
            "fe80::1",
            "fd00::1",
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254",
            "64:ff9b::a9fe:a9fe",
            "2002:7f00:1::",
        ] {
            assert!(blocked_ip(ip.parse().unwrap()), "{ip} must be blocked");
        }
        for ip in [
            "8.8.8.8",
            "140.211.1.1",
            "2606:4700::1111",
            "::ffff:8.8.8.8",
        ] {
            assert!(!blocked_ip(ip.parse().unwrap()), "{ip} is public");
        }
    }

    #[test]
    fn urls() {
        assert!(check_url("ftp://example.org/x").is_err());
        assert!(check_url("file:///etc/passwd").is_err());
        assert!(check_url("https://user:pw@example.org/").is_err());
        assert!(check_url("https://example.org/a").is_ok());
        let base: ureq::http::Uri = "https://a.org/x/y?q".parse().unwrap();
        assert_eq!(join(&base, "/z"), "https://a.org/z");
        assert_eq!(join(&base, "z"), "https://a.org/x/z");
        assert_eq!(join(&base, "//b.org/w"), "https://b.org/w");
        assert_eq!(join(&base, "http://c.org/"), "http://c.org/");
    }
}

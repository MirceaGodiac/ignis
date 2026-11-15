//! Search providers behind `web_search`. The adapter is kept apart from the contract (master
//! §13.1), so a provider or endpoint change touches only this file.
//!
//! [`ParallelSearch`] calls the Parallel Search API (`POST /v1/search`, `x-api-key`) over REST. The
//! master design names Parallel as the initial provider through its MCP endpoint; the REST API is
//! the same service without an MCP client in the server. The key is read only from
//! `PARALLEL_API_KEY` and never printed.

use std::time::Duration;

use ignis_core::contracts::{ErrorCode, ErrorEnvelope, RetryPolicy, WebProvider};
use serde_json::{json, Value};

/// One provider result, before IGNIS turns it into an `ExternalSourceCandidate`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderHit {
    pub url: String,
    pub title: Option<String>,
    /// The provider's excerpt: not the document.
    pub excerpt: Option<String>,
}

pub trait SearchProvider: Send + Sync {
    fn descriptor(&self) -> WebProvider;
    /// Up to `max_results` hits for `query`, best first. Blocking (network).
    fn search(&self, query: &str, max_results: usize) -> Result<Vec<ProviderHit>, ErrorEnvelope>;
}

pub const PARALLEL_KEY_ENV: &str = "PARALLEL_API_KEY";
pub const PARALLEL_URL_ENV: &str = "IGNIS_WEB_SEARCH_URL";
pub const PARALLEL_DEFAULT_URL: &str = "https://api.parallel.ai/v1/search";
/// Characters of excerpt per result requested from the provider.
const EXCERPT_CHARS_PER_RESULT: usize = 1500;

pub struct ParallelSearch {
    endpoint: String,
    key: String,
    agent: ureq::Agent,
}

impl std::fmt::Debug for ParallelSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelSearch")
            .field("endpoint", &self.endpoint)
            .field("key", &"<redacted>")
            .finish()
    }
}

impl ParallelSearch {
    pub fn new(endpoint: impl Into<String>, key: impl Into<String>) -> ParallelSearch {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(60)))
            .build()
            .new_agent();
        ParallelSearch {
            endpoint: endpoint.into(),
            key: key.into(),
            agent,
        }
    }

    /// From `PARALLEL_API_KEY` (and optionally `IGNIS_WEB_SEARCH_URL`); `None` without a key.
    pub fn from_env() -> Option<ParallelSearch> {
        let key = std::env::var(PARALLEL_KEY_ENV)
            .ok()
            .filter(|k| !k.trim().is_empty())?;
        let url = std::env::var(PARALLEL_URL_ENV)
            .ok()
            .filter(|u| !u.trim().is_empty())
            .unwrap_or_else(|| PARALLEL_DEFAULT_URL.to_string());
        Some(ParallelSearch::new(url, key.trim()))
    }

    /// The request body (also what `--execute`-less CLI runs print).
    pub fn body(query: &str, max_results: usize) -> Value {
        json!({
            "objective": query,
            "search_queries": [query],
            "mode": "basic",
            "max_chars_total": EXCERPT_CHARS_PER_RESULT * max_results.max(1),
            "advanced_settings": { "max_results": max_results },
        })
    }
}

fn upstream(msg: String) -> ErrorEnvelope {
    ErrorEnvelope::new(
        ErrorCode::UpstreamUnavailable,
        msg,
        RetryPolicy::AfterDelay { seconds: 60 },
    )
}

impl SearchProvider for ParallelSearch {
    fn descriptor(&self) -> WebProvider {
        WebProvider {
            name: "parallel".into(),
            tool: "search_api_v1".into(),
            version: None,
        }
    }

    fn search(&self, query: &str, max_results: usize) -> Result<Vec<ProviderHit>, ErrorEnvelope> {
        let mut resp = self
            .agent
            .post(&self.endpoint)
            .header("x-api-key", &self.key)
            .header("content-type", "application/json")
            .send(Self::body(query, max_results).to_string())
            .map_err(|e| upstream(format!("web search provider: {e}")))?;
        let status = resp.status().as_u16();
        let text = resp
            .body_mut()
            .with_config()
            .limit(16 << 20)
            .read_to_string()
            .map_err(|e| upstream(format!("web search provider body: {e}")))?;
        match status {
            200..=299 => {}
            401 | 403 => {
                return Err(ErrorEnvelope::new(
                    ErrorCode::CapabilityUnavailable,
                    format!("web search provider refused the API key (HTTP {status})"),
                    RetryPolicy::AfterUserAction,
                ))
            }
            429 => {
                return Err(ErrorEnvelope::new(
                    ErrorCode::RateLimited,
                    "web search provider rate limit".to_string(),
                    RetryPolicy::AfterDelay { seconds: 60 },
                ))
            }
            _ => {
                return Err(upstream(format!(
                    "web search provider answered HTTP {status}"
                )))
            }
        }
        parse_parallel(&text)
    }
}

/// `{"results": [{"url", "title", "excerpts": [..]}]}` → hits.
pub fn parse_parallel(text: &str) -> Result<Vec<ProviderHit>, ErrorEnvelope> {
    let v: Value = serde_json::from_str(text)
        .map_err(|e| upstream(format!("web search provider answered non-JSON: {e}")))?;
    let results = v
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| upstream("web search provider answer has no `results`".into()))?;
    results
        .iter()
        .map(|r| {
            let url = r
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| upstream("a web search result has no url".into()))?;
            let excerpt = r.get("excerpts").and_then(Value::as_array).map(|xs| {
                xs.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("\n…\n")
            });
            Ok(ProviderHit {
                url: url.to_string(),
                title: r.get("title").and_then(Value::as_str).map(str::to_string),
                excerpt: excerpt.filter(|e| !e.is_empty()),
            })
        })
        .collect()
}

//! Blocking DeepInfra client for Qwen3 embeddings and reranking (V3.1 §18.2).
//!
//! * Embeddings: `POST {base}/openai/embeddings` (OpenAI-compatible): `{"model", "input": [..],
//!   "encoding_format": "float", "dimensions"?}` -> `data[].{index, embedding}`, `usage.prompt_tokens`.
//! * Rerank: `POST {base}/inference/{model}`: `{"queries": [q], "documents": [..], "instruction"?}` ->
//!   `scores[]`, `input_tokens`, `inference_status.cost`. Measured 2026-11-08: the endpoint requires
//!   `len(queries) == len(documents)` or one query, and scores every document against `queries[0]`
//!   (a second, different query is ignored), so one request carries exactly one query.
//!
//! The key is read only from `DEEPINFRA_API_KEY`; it never appears in `Debug` output or in error
//! messages ([`redact`] strips it and any `Bearer` token). Requests are batched by item count and
//! characters, retried with exponential backoff on 429/5xx/transport errors, and bounded by a
//! per-request timeout and an optional overall deadline (error text starts with `timeout`).

use std::ops::Range;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub const PROVIDER: &str = "deepinfra";
pub const KEY_ENV: &str = "DEEPINFRA_API_KEY";

// ------------------------------------------------------------------ key

/// The API key. No `Display`; `Debug` is redacted.
#[derive(Clone)]
pub struct ApiKey(String);

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey([REDACTED])")
    }
}

impl ApiKey {
    pub fn new(s: impl Into<String>) -> Option<ApiKey> {
        let s: String = s.into();
        let s = s.trim().trim_matches('"').to_string();
        (!s.is_empty()).then_some(ApiKey(s))
    }
    /// `DEEPINFRA_API_KEY`, if set and non-empty.
    pub fn from_env() -> Option<ApiKey> {
        std::env::var(KEY_ENV).ok().and_then(ApiKey::new)
    }
}

/// Remove the key and any `Bearer <token>` from a message.
pub fn redact(msg: &str, key: Option<&ApiKey>) -> String {
    static R: OnceLock<Regex> = OnceLock::new();
    let mut s = msg.to_string();
    if let Some(k) = key {
        if k.0.len() >= 4 {
            s = s.replace(&k.0, "[REDACTED]");
        }
    }
    let rx = R.get_or_init(|| Regex::new(r"(?i)(bearer)\s+[A-Za-z0-9._~+/=\-]+").unwrap());
    rx.replace_all(&s, "$1 [REDACTED]").into_owned()
}

// ------------------------------------------------------------------ wire shapes

#[derive(Debug, Serialize)]
pub struct EmbedRequest<'a> {
    pub model: &'a str,
    pub input: &'a [String],
    pub encoding_format: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct EmbedItem {
    pub index: usize,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Default, Deserialize)]
pub struct EmbedUsage {
    #[serde(default)]
    pub prompt_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
}

#[derive(Debug, Deserialize)]
pub struct EmbedResponse {
    pub data: Vec<EmbedItem>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub usage: EmbedUsage,
}

#[derive(Debug, Serialize)]
pub struct RerankRequest<'a> {
    pub queries: [&'a str; 1],
    pub documents: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<&'a str>,
}

#[derive(Debug, Default, Deserialize)]
pub struct InferenceStatus {
    #[serde(default)]
    pub runtime_ms: u64,
    #[serde(default)]
    pub cost: f64,
    #[serde(default)]
    pub tokens_input: u64,
}

#[derive(Debug, Deserialize)]
pub struct RerankResponse {
    pub scores: Vec<f32>,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub inference_status: Option<InferenceStatus>,
    #[serde(default)]
    pub request_id: Option<String>,
}

impl EmbedResponse {
    /// Vectors in input order; checks count, a permutation of indices and one dimension.
    pub fn into_ordered(self, n: usize, dim: Option<usize>) -> Result<Vec<Vec<f32>>> {
        if self.data.len() != n {
            bail!(
                "embeddings: {} vectors returned for {n} inputs",
                self.data.len()
            );
        }
        let mut out: Vec<Option<Vec<f32>>> = vec![None; n];
        for it in self.data {
            if it.index >= n || out[it.index].is_some() {
                bail!("embeddings: bad or duplicate index {}", it.index);
            }
            out[it.index] = Some(it.embedding);
        }
        let out: Vec<Vec<f32>> = out.into_iter().map(|v| v.unwrap()).collect();
        let d = dim.unwrap_or_else(|| out.first().map(|v| v.len()).unwrap_or(0));
        if out.iter().any(|v| v.len() != d || d == 0) {
            bail!("embeddings: expected dimension {d}");
        }
        Ok(out)
    }
}

// ------------------------------------------------------------------ batching and estimates

/// Contiguous batches in input order: at most `max_items` items and `max_chars` characters each
/// (an item longer than `max_chars` goes alone).
pub fn batches(lens: &[usize], max_items: usize, max_chars: usize) -> Vec<Range<usize>> {
    let (max_items, max_chars) = (max_items.max(1), max_chars.max(1));
    let mut out = Vec::new();
    let (mut start, mut chars) = (0, 0);
    for (i, &l) in lens.iter().enumerate() {
        if i > start && (i - start >= max_items || chars + l > max_chars) {
            out.push(start..i);
            start = i;
            chars = 0;
        }
        chars += l;
    }
    if start < lens.len() {
        out.push(start..lens.len());
    }
    out
}

/// Token estimate before a call (~4 chars/token for English technical prose; compared with billed
/// tokens in every usage record).
pub fn estimate_tokens(text: &str) -> u64 {
    (text.chars().count() as u64).div_ceil(4)
}

/// Reranker prompt overhead per (query, document) pair, measured: 82 billed tokens for a 3-token
/// query and a 7-token document with the default instruction.
pub const RERANK_PAIR_OVERHEAD_TOKENS: u64 = 72;

pub fn usd(tokens: u64, usd_per_mtok: f64) -> f64 {
    tokens as f64 * usd_per_mtok / 1e6
}

/// Accounting for one stage: estimated before the call, billed after (tokens/cost reported by the API).
#[derive(Debug, Clone, Default, Serialize)]
pub struct ApiUsage {
    pub provider: &'static str,
    pub model: String,
    pub requests: u64,
    pub attempts: u64,
    pub estimated_tokens: u64,
    pub billed_tokens: u64,
    pub estimated_usd: f64,
    pub billed_usd: f64,
    /// `inference_status.cost` (rerank) or `usage.prompt_tokens x list price` (embeddings: the
    /// OpenAI-compatible response carries no cost field)
    pub billed_source: &'static str,
    pub latency_ms: f64,
}

// ------------------------------------------------------------------ client

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderCfg {
    #[serde(default = "default_base")]
    pub base_url: String,
    #[serde(default = "d30")]
    pub timeout_s: f64,
    #[serde(default = "d4")]
    pub max_retries: u32,
    #[serde(default = "d05")]
    pub backoff_s: f64,
}

fn default_base() -> String {
    "https://api.deepinfra.com/v1".into()
}
fn d30() -> f64 {
    30.0
}
fn d4() -> u32 {
    4
}
fn d05() -> f64 {
    0.5
}

impl Default for ProviderCfg {
    fn default() -> Self {
        ProviderCfg {
            base_url: default_base(),
            timeout_s: 30.0,
            max_retries: 4,
            backoff_s: 0.5,
        }
    }
}

pub struct Client {
    agent: ureq::Agent,
    key: ApiKey,
    pub cfg: ProviderCfg,
}

/// Retry 429 and 5xx; other statuses are final.
pub fn retryable(status: u16) -> bool {
    status == 429 || (500..600).contains(&status)
}

impl Client {
    pub fn new(key: ApiKey, cfg: ProviderCfg) -> Client {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .user_agent(concat!("ignis-evidence/", env!("CARGO_PKG_VERSION")))
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs_f64(cfg.timeout_s.max(0.1))))
            .build()
            .into();
        Client { agent, key, cfg }
    }

    fn err(&self, msg: String) -> anyhow::Error {
        anyhow::anyhow!(redact(&msg, Some(&self.key)))
    }

    /// POST JSON with retries; returns the parsed body and the number of attempts.
    fn post<B: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &B,
        deadline: Option<Instant>,
    ) -> Result<(R, u64)> {
        let url = format!(
            "{}/{}",
            self.cfg.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let payload = serde_json::to_string(body)?;
        let mut attempt: u32 = 0;
        loop {
            attempt += 1;
            let mut timeout = Duration::from_secs_f64(self.cfg.timeout_s.max(0.1));
            if let Some(d) = deadline {
                let left = d.saturating_duration_since(Instant::now());
                if left.is_zero() {
                    bail!("timeout: deadline reached before attempt {attempt} to POST {url}");
                }
                timeout = timeout.min(left);
            }
            let res = self
                .agent
                .post(&url)
                .header("Authorization", &format!("Bearer {}", self.key.0))
                .header("Content-Type", "application/json")
                .config()
                .timeout_global(Some(timeout))
                .build()
                .send(payload.as_str());
            let (retry, msg, after) = match res {
                Ok(mut r) => {
                    let status = r.status().as_u16();
                    let after = r
                        .headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.trim().parse::<f64>().ok());
                    let text = r.body_mut().with_config().limit(256 << 20).read_to_string();
                    match text {
                        Ok(t) if (200..300).contains(&status) => {
                            return serde_json::from_str::<R>(&t)
                                .map(|v| (v, attempt as u64))
                                .map_err(|e| {
                                    self.err(format!(
                                        "POST {url}: response does not parse ({e}): {}",
                                        snippet(&t)
                                    ))
                                });
                        }
                        Ok(t) => (
                            retryable(status),
                            format!("POST {url}: HTTP {status}: {}", snippet(&t)),
                            after,
                        ),
                        Err(e) => (true, format!("POST {url}: reading body: {e}"), after),
                    }
                }
                Err(ureq::Error::Timeout(t)) => {
                    if deadline.is_some() {
                        return Err(self.err(format!(
                            "timeout: POST {url} ({t:?}) after {attempt} attempt(s)"
                        )));
                    }
                    (true, format!("timeout: POST {url} ({t:?})"), None)
                }
                Err(e) => (true, format!("POST {url}: {e}"), None),
            };
            if !retry || attempt > self.cfg.max_retries {
                return Err(self.err(format!("{msg} (attempt {attempt})")));
            }
            let wait = after
                .unwrap_or(self.cfg.backoff_s * 2f64.powi(attempt as i32 - 1))
                .clamp(0.0, 30.0);
            if deadline.is_some_and(|d| Instant::now() + Duration::from_secs_f64(wait) >= d) {
                return Err(self.err(format!("timeout: no time left to retry after: {msg}")));
            }
            std::thread::sleep(Duration::from_secs_f64(wait));
        }
    }

    /// L2-normalized embeddings in input order.
    #[allow(clippy::too_many_arguments)]
    pub fn embed(
        &self,
        model: &str,
        texts: &[String],
        dimensions: Option<usize>,
        max_items: usize,
        max_chars: usize,
        usd_per_mtok: f64,
        deadline: Option<Instant>,
    ) -> Result<(Vec<Vec<f32>>, ApiUsage)> {
        let t = Instant::now();
        let mut u = ApiUsage {
            provider: PROVIDER,
            model: model.into(),
            billed_source: "usage.prompt_tokens x list price",
            ..Default::default()
        };
        u.estimated_tokens = texts.iter().map(|s| estimate_tokens(s)).sum();
        u.estimated_usd = usd(u.estimated_tokens, usd_per_mtok);
        let lens: Vec<usize> = texts.iter().map(|s| s.chars().count()).collect();
        let mut out = Vec::with_capacity(texts.len());
        for r in batches(&lens, max_items, max_chars) {
            let req = EmbedRequest {
                model,
                input: &texts[r.clone()],
                encoding_format: "float",
                dimensions,
            };
            let (resp, attempts): (EmbedResponse, u64) =
                self.post("openai/embeddings", &req, deadline)?;
            u.requests += 1;
            u.attempts += attempts;
            u.billed_tokens += resp.usage.prompt_tokens.max(resp.usage.total_tokens);
            for mut v in resp.into_ordered(r.len(), dimensions)? {
                let n = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
                v.iter_mut().for_each(|x| *x /= n);
                out.push(v);
            }
        }
        u.billed_usd = usd(u.billed_tokens, usd_per_mtok);
        u.latency_ms = ms(t);
        Ok((out, u))
    }

    /// Relevance in [0, 1] per document (input order), one query per request.
    #[allow(clippy::too_many_arguments)]
    pub fn rerank(
        &self,
        model: &str,
        query: &str,
        docs: &[String],
        instruction: Option<&str>,
        max_items: usize,
        max_chars: usize,
        usd_per_mtok: f64,
        deadline: Option<Instant>,
    ) -> Result<(Vec<f32>, ApiUsage)> {
        let t = Instant::now();
        let mut u = ApiUsage {
            provider: PROVIDER,
            model: model.into(),
            billed_source: "inference_status.cost",
            ..Default::default()
        };
        let fixed = estimate_tokens(query)
            + instruction.map(estimate_tokens).unwrap_or(0)
            + RERANK_PAIR_OVERHEAD_TOKENS;
        u.estimated_tokens = docs.iter().map(|d| fixed + estimate_tokens(d)).sum();
        u.estimated_usd = usd(u.estimated_tokens, usd_per_mtok);
        let lens: Vec<usize> = docs.iter().map(|s| s.chars().count()).collect();
        let mut out = Vec::with_capacity(docs.len());
        for r in batches(&lens, max_items, max_chars) {
            let req = RerankRequest {
                queries: [query],
                documents: &docs[r.clone()],
                instruction,
            };
            let (resp, attempts): (RerankResponse, u64) =
                self.post(&format!("inference/{model}"), &req, deadline)?;
            if resp.scores.len() != r.len() {
                bail!(
                    "rerank: {} scores for {} documents",
                    resp.scores.len(),
                    r.len()
                );
            }
            u.requests += 1;
            u.attempts += attempts;
            let st = resp.inference_status.unwrap_or_default();
            u.billed_tokens += resp.input_tokens.max(st.tokens_input);
            u.billed_usd += st.cost;
            out.extend(resp.scores);
        }
        u.latency_ms = ms(t);
        Ok((out, u))
    }
}

fn snippet(t: &str) -> String {
    let s: String = t.chars().take(300).collect();
    s.replace('\n', " ")
}

fn ms(t: Instant) -> f64 {
    (t.elapsed().as_secs_f64() * 1e4).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_request_and_recorded_response() {
        let input = vec!["a".to_string(), "b".to_string()];
        let req = EmbedRequest {
            model: "Qwen/Qwen3-Embedding-8B",
            input: &input,
            encoding_format: "float",
            dimensions: None,
        };
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            serde_json::json!({"model": "Qwen/Qwen3-Embedding-8B", "input": ["a", "b"], "encoding_format": "float"})
        );
        let req = EmbedRequest {
            dimensions: Some(1024),
            ..req
        };
        assert_eq!(serde_json::to_value(&req).unwrap()["dimensions"], 1024);
        // shape recorded from a real call (2026-11-08), vectors shortened, indices out of order
        let rec = r#"{"object":"list","data":[{"object":"embedding","index":1,"embedding":[0.0,1.0]},
            {"object":"embedding","index":0,"embedding":[1.0,0.0]}],"model":"Qwen/Qwen3-Embedding-8B",
            "usage":{"prompt_tokens":6,"total_tokens":6}}"#;
        let r: EmbedResponse = serde_json::from_str(rec).unwrap();
        assert_eq!(r.usage.prompt_tokens, 6);
        let v = r.into_ordered(2, Some(2)).unwrap();
        assert_eq!(v, vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
        let bad: EmbedResponse = serde_json::from_str(
            r#"{"data":[{"index":0,"embedding":[1.0]},{"index":0,"embedding":[1.0]}]}"#,
        )
        .unwrap();
        assert!(bad.into_ordered(2, None).is_err());
        let short: EmbedResponse =
            serde_json::from_str(r#"{"data":[{"index":0,"embedding":[1.0]}]}"#).unwrap();
        assert!(short.into_ordered(1, Some(4096)).is_err());
    }

    #[test]
    fn rerank_request_and_recorded_response() {
        let docs = vec!["d1".to_string(), "d2".to_string()];
        let req = RerankRequest {
            queries: ["q"],
            documents: &docs,
            instruction: Some("judge"),
        };
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            serde_json::json!({"queries": ["q"], "documents": ["d1", "d2"], "instruction": "judge"})
        );
        let rec = r#"{"request_id":"RsxbRoYKXDzudrD9lJuxSxIt","inference_status":{"runtime_ms":152,
            "cost":1.3750000000000002e-05,"tokens_input":275},"scores":[0.9890931844711304,1.8217839397038915e-06,
            0.2355547547340393],"input_tokens":275}"#;
        let r: RerankResponse = serde_json::from_str(rec).unwrap();
        assert_eq!(r.scores.len(), 3);
        assert_eq!(r.input_tokens, 275);
        let st = r.inference_status.unwrap();
        assert!((st.cost - 1.375e-5).abs() < 1e-12);
        assert_eq!(st.tokens_input, 275);
    }

    #[test]
    fn batching_boundaries() {
        assert!(batches(&[], 4, 100).is_empty());
        assert_eq!(batches(&[1; 5], 2, 100), vec![0..2, 2..4, 4..5]);
        assert_eq!(batches(&[1; 4], 4, 100), vec![0..4]);
        // char budget: 40+40 fits 100, a third 40 does not
        assert_eq!(batches(&[40, 40, 40], 10, 100), vec![0..2, 2..3]);
        assert_eq!(batches(&[50, 50, 1], 10, 100), vec![0..2, 2..3]); // exactly at the limit
                                                                      // an oversize item goes alone, neighbours are not merged into it
        assert_eq!(batches(&[10, 500, 10], 10, 100), vec![0..1, 1..2, 2..3]);
        // every index appears once, in order
        let lens: Vec<usize> = (0..97).map(|i| (i * 37) % 300).collect();
        let b = batches(&lens, 8, 900);
        assert_eq!(b.first().unwrap().start, 0);
        assert_eq!(b.last().unwrap().end, 97);
        assert!(b.windows(2).all(|w| w[0].end == w[1].start));
        assert!(b.iter().all(
            |r| r.len() <= 8 && (r.len() == 1 || lens[r.clone()].iter().sum::<usize>() <= 900)
        ));
    }

    #[test]
    fn key_is_redacted() {
        let k = ApiKey::new("sk_TEST0123456789abcdef").unwrap();
        assert_eq!(format!("{k:?}"), "ApiKey([REDACTED])");
        let msg = "POST https://x/v1: HTTP 401 Authorization: Bearer sk_TEST0123456789abcdef; echo sk_TEST0123456789abcdef";
        let r = redact(msg, Some(&k));
        assert!(!r.contains("sk_TEST0123456789abcdef"), "{r}");
        assert!(r.contains("Bearer [REDACTED]"));
        assert!(!redact("authorization: bearer abc.def-ghi", None).contains("abc.def-ghi"));
        assert!(ApiKey::new("  ").is_none());
        // a client error never carries the key
        let c = Client::new(k.clone(), ProviderCfg::default());
        let e = c.err(format!(
            "failed with Bearer {} inside",
            "sk_TEST0123456789abcdef"
        ));
        assert!(!format!("{e:#}").contains("sk_TEST0123456789abcdef"));
    }

    #[test]
    fn retry_policy() {
        assert!(retryable(429) && retryable(500) && retryable(503));
        assert!(!retryable(400) && !retryable(401) && !retryable(422));
    }
}

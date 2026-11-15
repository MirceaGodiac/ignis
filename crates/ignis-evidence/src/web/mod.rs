//! Web discovery (workboard R4, web half): `web_search` and `web_fetch` behind
//! [`ignis_core::services::KnowledgeService`].
//!
//! **Not the corpus.** A web result is an [`ExternalSourceCandidate`]: a source found outside the
//! NASA corpus, never a corpus source, never a NASA measurement, its text data and never
//! instructions. `web_search` returns provider excerpts (`content = excerpt`); `web_fetch` stores
//! what it fetched and returns `content = full_snapshot` with the snapshot's [`ArtifactRef`].
//!
//! **Snapshots.** `web_fetch` puts the fetched bytes (for HTML: the extracted text, `text/plain`)
//! into the deployment's artifact store, in the caller's workspace, so `artifacts_read` opens them
//! and a citation points at a frozen, hashed copy instead of the live page. The store is
//! content-addressed: identical bytes share one artifact, a changed page is a new artifact, and old
//! artifacts keep resolving. Each fetch also appends a [`SnapshotRecord`] (requested and final URL,
//! redirect chain, HTTP status, original content type and its sha256, fetch time, rights) to the
//! append-only fetch log next to the catalog; [`WebDiscovery::snapshot`] reads a snapshot back by
//! artifact id with every hash re-verified.
//!
//! **Rights.** Per snapshot, from a domain rule ([`rights::default_rules`]: nasa.gov public domain,
//! ntrs.nasa.gov per document) or the page's own declaration; otherwise `unverified` with no
//! licence, the explicit unknown.
//!
//! **Security.** [`fetch::Fetcher`] (http(s) only, SSRF checks on every hop's resolved addresses,
//! bounded redirects, size and time budgets). The service refuses evaluation and authoring scopes
//! and scopes without `network_fetch`.
//!
//! **Gating.** Nothing networked runs unless the deployment opts in: `IGNIS_ENABLE_WEB=1` (off by
//! default) wires the tools; `web_search` needs a provider key as well ([`provider`]).

pub mod cli;
pub mod fetch;
pub mod html;
pub mod provider;
pub mod rights;

use std::io::{Read as _, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ignis_core::contracts::{
    ArtifactId, ArtifactRef, Capability, Digest, ErrorCode, ErrorEnvelope, EvidenceSchema,
    ExecutionScope, ExternalSourceCandidate, Origin, PageCursor, RetryPolicy, Rights, ScopeRole,
    Timestamp, WebContent, WebFetchRequest, WebProvider, WebSearchPage, WebSearchRequest,
    WorkspaceId,
};
use ignis_core::services::{ArtifactDraft, ArtifactStore, ServiceResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest as _, Sha256};

use self::fetch::{FetchPolicy, Fetcher};
use self::provider::SearchProvider;
use self::rights::RightsPolicy;

/// The env flag that opts a deployment into web discovery (off by default).
pub const ENABLE_ENV: &str = "IGNIS_ENABLE_WEB";
/// Results per `web_search` page when the request names none.
pub const DEFAULT_PAGE_SIZE: u32 = 10;
/// The fetch log, next to the catalog.
pub const FETCH_LOG: &str = "web/fetches.jsonl";

/// True when `IGNIS_ENABLE_WEB` is `1`/`true`/`yes`/`on`.
pub fn enabled_from_env() -> bool {
    std::env::var(ENABLE_ENV).is_ok_and(|v| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

/// The fetch log of the catalog at `db`.
pub fn fetch_log_for(db: &Path) -> PathBuf {
    db.with_file_name(FETCH_LOG)
}

/// One fetch, as appended to the fetch log. The artifact holds the bytes; this holds how they were
/// obtained. Several records may name one artifact (identical bytes fetched again).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotRecord {
    pub artifact_id: ArtifactId,
    /// Digest of the stored bytes (equals the artifact's).
    pub sha256: Digest,
    pub workspace_id: WorkspaceId,
    pub requested_url: String,
    pub redirects: Vec<String>,
    pub final_url: String,
    pub http_status: u16,
    /// `Content-Type` the server sent.
    pub content_type: Option<String>,
    /// Media type of the stored bytes (`text/plain` when HTML text was extracted).
    pub stored_media_type: String,
    /// True when the stored bytes are text extracted from HTML, not the bytes served.
    pub extracted_text: bool,
    /// Digest and length of the bytes as served (differ from `sha256` when text was extracted).
    pub source_sha256: Digest,
    pub source_bytes: u64,
    pub fetched_at: Timestamp,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub canonical_url: Option<String>,
    pub rights: Rights,
    pub origin: Origin,
}

/// A snapshot read back: its artifact, the bytes (hash verified) and every fetch that produced them.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub artifact: ArtifactRef,
    pub bytes: Vec<u8>,
    pub records: Vec<SnapshotRecord>,
}

fn err(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

fn digest_of(bytes: &[u8]) -> Digest {
    Digest::parse(&format!("sha256:{}", hex::encode(Sha256::digest(bytes)))).expect("sha256 hex")
}

fn http_url(s: &str) -> bool {
    fetch::check_url(s).is_ok() && !s.chars().any(|c| c.is_whitespace() || c.is_control())
}

pub struct WebDiscovery {
    fetcher: Fetcher,
    provider: Option<Arc<dyn SearchProvider>>,
    store: Arc<dyn ArtifactStore>,
    log_path: PathBuf,
    log_lock: Mutex<()>,
    rights: RightsPolicy,
    origin: Origin,
}

impl WebDiscovery {
    /// Snapshots go to `store` (the deployment's artifact store); the fetch log to `log_path`.
    pub fn new(
        store: Arc<dyn ArtifactStore>,
        log_path: PathBuf,
        provider: Option<Arc<dyn SearchProvider>>,
        policy: FetchPolicy,
        origin: Origin,
    ) -> WebDiscovery {
        WebDiscovery {
            fetcher: Fetcher::new(policy),
            provider,
            store,
            log_path,
            log_lock: Mutex::new(()),
            rights: RightsPolicy::default(),
            origin,
        }
    }

    pub fn has_provider(&self) -> bool {
        self.provider.is_some()
    }

    fn check_scope(&self, scope: &ExecutionScope, tool: &str) -> ServiceResult<()> {
        scope
            .validate()
            .map_err(|e| err(ErrorCode::ScopeDenied, e.to_string()))?;
        if matches!(scope.role, ScopeRole::Evaluation | ScopeRole::Authoring) {
            return Err(err(
                ErrorCode::ScopeDenied,
                format!(
                    "{tool} is off for {:?} scopes (frozen evaluation, protected authoring)",
                    scope.role
                ),
            ));
        }
        scope.require(Capability::NetworkFetch)?;
        Ok(())
    }

    fn fetch_provider(&self) -> WebProvider {
        WebProvider {
            name: "ignis".into(),
            tool: "web_fetch".into(),
            version: Some(env!("CARGO_PKG_VERSION").into()),
        }
    }

    pub fn search(
        &self,
        scope: &ExecutionScope,
        req: &WebSearchRequest,
    ) -> ServiceResult<WebSearchPage> {
        self.check_scope(scope, "web_search")?;
        req.validate()?;
        let provider = self.provider.as_ref().ok_or_else(|| {
            ErrorEnvelope::new(
                ErrorCode::CapabilityUnavailable,
                "no web search provider configured (set PARALLEL_API_KEY)",
                RetryPolicy::AfterUserAction,
            )
        })?;
        let desc = provider.descriptor();
        let query = req.query.trim();
        let digest = hex::encode(
            &Sha256::digest(json!({"q": query, "p": desc.name, "t": desc.tool}).to_string())[..12],
        );
        let offset = match &req.cursor {
            None => 0,
            Some(c) => {
                let (d, o) = c
                    .as_str()
                    .strip_prefix("w1.")
                    .and_then(|r| r.split_once('.'))
                    .and_then(|(d, o)| Some((d, o.parse::<usize>().ok()?)))
                    .ok_or_else(|| err(ErrorCode::InvalidRequest, "malformed cursor"))?;
                if d != digest {
                    return Err(err(
                        ErrorCode::InvalidRequest,
                        "the cursor belongs to another web search (query or provider differ)",
                    ));
                }
                o
            }
        };
        let size = req.page_size.unwrap_or(DEFAULT_PAGE_SIZE) as usize;
        // The provider does not page: ask for everything up to this page plus one to know whether
        // more exist. The live web can change between pages; the cursor binds query and provider.
        let hits = provider.search(query, offset + size + 1)?;
        let more = hits.len() > offset + size;
        let accessed_at = Timestamp::now();
        let candidates = hits
            .into_iter()
            .skip(offset)
            .take(size)
            .map(|h| {
                if !http_url(&h.url) {
                    return Err(err(
                        ErrorCode::UpstreamUnavailable,
                        format!(
                            "the web search provider returned a non-http(s) URL {:?}",
                            h.url
                        ),
                    ));
                }
                let c = ExternalSourceCandidate {
                    schema_version: EvidenceSchema::V2,
                    rights: self.rights.rights(&h.url, None),
                    url: h.url,
                    final_url: None,
                    canonical_url: None,
                    title: h.title,
                    excerpt: h.excerpt,
                    content: WebContent::Excerpt,
                    snapshot: None,
                    accessed_at: accessed_at.clone(),
                    provider: desc.clone(),
                    promoted_source_id: None,
                    origin: self.origin,
                };
                c.validate()?;
                Ok(c)
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        let next_cursor = more.then(|| {
            PageCursor::new(format!("w1.{digest}.{}", offset + candidates.len()))
                .expect("cursor has no spaces")
        });
        let page = WebSearchPage {
            schema_version: EvidenceSchema::V2,
            query: query.to_string(),
            provider: desc,
            candidates,
            next_cursor,
            origin: self.origin,
        };
        page.validate()?;
        Ok(page)
    }

    /// Fetch `req.url` under the SSRF policy into a snapshot artifact. Blocking (network).
    pub fn fetch(
        &self,
        scope: &ExecutionScope,
        req: &WebFetchRequest,
    ) -> ServiceResult<ExternalSourceCandidate> {
        self.check_scope(scope, "web_fetch")?;
        req.validate()?;
        let got = self.fetcher.fetch(&req.url)?;
        let fetched_at = Timestamp::now();
        let source_sha256 = digest_of(&got.bytes);
        let source_bytes = got.bytes.len() as u64;
        let html = html::is_html(got.content_type.as_deref(), &got.bytes);
        let (stored, media_type, title, canonical, declared) = if html {
            let page = html::extract(&String::from_utf8_lossy(&got.bytes));
            if page.text.trim().is_empty() {
                return Err(err(
                    ErrorCode::Unsupported,
                    format!(
                        "{:?} has no extractable text (a script-rendered page?); nothing was stored",
                        got.final_url
                    ),
                ));
            }
            let base: ureq::http::Uri = got
                .final_url
                .parse()
                .map_err(|_| err(ErrorCode::Internal, "final URL no longer parses"))?;
            let canonical = page
                .canonical
                .map(|c| fetch::join(&base, &c))
                .filter(|c| http_url(c));
            (
                page.text.into_bytes(),
                "text/plain; charset=utf-8".to_string(),
                page.title,
                canonical,
                page.license,
            )
        } else {
            let mt = got
                .content_type
                .as_deref()
                .map(str::trim)
                .filter(|c| {
                    c.split(';')
                        .next()
                        .and_then(|b| b.split_once('/'))
                        .is_some_and(|(a, b)| !a.trim().is_empty() && !b.trim().is_empty())
                })
                .unwrap_or("application/octet-stream")
                .to_string();
            (got.bytes, mt, None, None, None)
        };
        let rights = self.rights.rights(&got.final_url, declared.as_deref());
        let artifact = self.store.put(
            scope,
            ArtifactDraft {
                media_type: media_type.clone(),
                contract: None,
                produced_by: Default::default(),
                inputs: Vec::new(),
                origin: self.origin,
            },
            &mut stored.as_slice(),
        )?;
        let record = SnapshotRecord {
            artifact_id: artifact.artifact_id.clone(),
            sha256: artifact.sha256.clone(),
            workspace_id: scope.workspace_id.clone(),
            requested_url: got.requested_url.clone(),
            redirects: got.redirects.clone(),
            final_url: got.final_url.clone(),
            http_status: got.status,
            content_type: got.content_type.clone(),
            stored_media_type: media_type,
            extracted_text: html,
            source_sha256,
            source_bytes,
            fetched_at: fetched_at.clone(),
            title: title.clone(),
            canonical_url: canonical.clone(),
            rights: rights.clone(),
            origin: self.origin,
        };
        self.append(&record)?;
        let c = ExternalSourceCandidate {
            schema_version: EvidenceSchema::V2,
            url: req.url.clone(),
            final_url: Some(got.final_url),
            canonical_url: canonical,
            title,
            excerpt: None,
            content: WebContent::FullSnapshot,
            snapshot: Some(artifact),
            accessed_at: fetched_at,
            provider: self.fetch_provider(),
            rights,
            promoted_source_id: None,
            origin: self.origin,
        };
        c.validate()?;
        Ok(c)
    }

    fn append(&self, r: &SnapshotRecord) -> ServiceResult<()> {
        let io = |e: std::io::Error| err(ErrorCode::Internal, format!("fetch log: {e}"));
        let _g = self.log_lock.lock().expect("fetch log lock");
        if let Some(d) = self.log_path.parent() {
            std::fs::create_dir_all(d).map_err(io)?;
        }
        let mut line =
            serde_json::to_vec(r).map_err(|e| err(ErrorCode::Internal, e.to_string()))?;
        line.push(b'\n');
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(io)?;
        f.write_all(&line).map_err(io)?;
        f.sync_data().map_err(io)
    }

    /// Every fetch record of `id` in the scope's workspace.
    pub fn records(
        &self,
        scope: &ExecutionScope,
        id: &ArtifactId,
    ) -> ServiceResult<Vec<SnapshotRecord>> {
        let text = match std::fs::read_to_string(&self.log_path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(err(ErrorCode::Internal, format!("fetch log: {e}"))),
        };
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                serde_json::from_str::<SnapshotRecord>(l)
                    .map_err(|e| err(ErrorCode::Internal, format!("fetch log line: {e}")))
            })
            .filter(|r| {
                r.as_ref().map_or(true, |r| {
                    &r.artifact_id == id && r.workspace_id == scope.workspace_id
                })
            })
            .collect()
    }

    /// Read a snapshot back by artifact id: the bytes are re-hashed and must match the artifact's
    /// digest and every fetch record's. A mismatch is an integrity failure, never served.
    pub fn snapshot(&self, scope: &ExecutionScope, id: &ArtifactId) -> ServiceResult<Snapshot> {
        let records = self.records(scope, id)?;
        if records.is_empty() {
            return Err(err(
                ErrorCode::NotFound,
                format!("no web snapshot {id} in this workspace"),
            ));
        }
        let artifact = self.store.stat(scope, id)?;
        let mut bytes = Vec::new();
        self.store
            .open(scope, id, 0, None)?
            .read_to_end(&mut bytes)
            .map_err(|e| err(ErrorCode::Internal, format!("reading snapshot {id}: {e}")))?;
        let sha = digest_of(&bytes);
        if sha != artifact.sha256 || records.iter().any(|r| r.sha256 != sha) {
            return Err(err(
                ErrorCode::Internal,
                format!("snapshot {id} failed its hash check ({sha} read)"),
            ));
        }
        Ok(Snapshot {
            artifact,
            bytes,
            records,
        })
    }
}

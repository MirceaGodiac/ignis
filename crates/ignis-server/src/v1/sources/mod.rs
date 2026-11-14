//! Source access without copies (task M12, LANES_V2 §10): `GET /sources/{id}/content` →
//! [`SourceAccess`].
//!
//! The originals stay at NASA. Per source version, from the R6 manifests ([`catalog`]):
//! * PSI: a fresh signed S3 URL from geode `redirect=false` ([`resolver`], single-flight, cached
//!   until expiry minus a margin), `direct` (S3 answers Range and CORS `*`);
//! * PSI CFI-G `.ts` with a keyframe index: a byte-range HLS playlist
//!   (`GET /sources/{id}/playlist.m3u8`, [`hls`]) whose segments are that same signed URL, so the
//!   viewer can seek; without an index, the signed URL itself (plays, no seek);
//! * NTRS: the Worker pass-through proxy (`<public-url>/ntrs/api/citations/<id>/downloads/<file>`,
//!   `infra/worker`), `proxy`; nothing is stored;
//! * a permanent URL on a measured host (NASA Image and Video Library): passed through, `direct`;
//! * anything else: `link_out` to the provider's page.
//!
//! The scope is host-derived ([`Caller`]); an id that is not in the manifests is `not_found`; no
//! URL is ever built from an id alone. A signed URL is delivery, never identity: citations keep the
//! `Source`.

pub mod catalog;
pub mod hls;
pub mod resolver;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use aide::axum::routing::get_with;
use aide::axum::ApiRouter;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use ignis_core::contracts::{
    AccessMode, Capability, ErrorCode, ErrorEnvelope, ExecutionScope, Origin, RetryPolicy,
    ScopeRole, SourceAccess, SourceId, Timestamp,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::sync::OnceCell;

use catalog::{Address, Catalog, Entry, PsiFile};
use resolver::{GeodeSigner, Signed, SignedUrls, Signer};

use super::io::{ApiError, ApiResult, Caller};
use super::V1;

/// Where sources are found and how they are delivered (`ignis serve`).
#[derive(Debug, Clone, clap::Args)]
pub struct SourceArgs {
    /// R6 source manifests (`<ACC>.jsonl`, `<ACC>.listing.jsonl`, `ntrs/*.jsonl`).
    #[arg(
        long = "source-manifests",
        env = "IGNIS_SOURCE_MANIFESTS",
        default_value = "data/manifests"
    )]
    pub manifests: PathBuf,
    /// Keyframe indexes of `.ts` sources (`<source id with ':' as '-'>.json`, task R11).
    #[arg(
        long = "media-index",
        env = "IGNIS_MEDIA_INDEX",
        default_value = "data/media/keyframes"
    )]
    pub media_index: PathBuf,
    /// Public origin of the UI (the Worker hostname, `https://…`, no trailing slash). Needed for
    /// the NTRS proxy and HLS playlist URLs; unset = NTRS opens as a link out and `.ts` without
    /// a playlist.
    #[arg(long = "public-url", env = "IGNIS_PUBLIC_URL")]
    pub public_url: Option<String>,
    /// geode web-service root (PSI's undocumented download API).
    #[arg(long = "psi-geode-url", env = "IGNIS_PSI_GEODE_URL", default_value = ignis_data::sources::psi::BASE)]
    pub geode_url: String,
    /// A signed URL leaves the cache this long before it expires.
    #[arg(long = "signed-url-margin-s", default_value_t = 300)]
    pub margin_s: u64,
    /// geode calls in flight at once (politeness to an undocumented endpoint).
    #[arg(long = "geode-concurrency", default_value_t = 2)]
    pub geode_concurrency: usize,
}

impl Default for SourceArgs {
    fn default() -> Self {
        SourceArgs {
            manifests: "data/manifests".into(),
            media_index: "data/media/keyframes".into(),
            public_url: None,
            geode_url: ignis_data::sources::psi::BASE.into(),
            margin_s: 300,
            geode_concurrency: 2,
        }
    }
}

/// Signed URLs kept (one per PSI file version being viewed).
const URL_CACHE_CAPACITY: u64 = 10_000;

/// The source access service of one `/api/v1` instance.
pub struct SourceDelivery {
    manifests: PathBuf,
    media_index: PathBuf,
    public_url: Option<String>,
    catalog: OnceCell<Arc<Catalog>>,
    urls: SignedUrls,
    origin: Origin,
}

fn envelope(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

impl SourceDelivery {
    /// Production wiring: geode signer, paths resolved against `root`.
    pub fn new(args: &SourceArgs, root: &Path, origin: Origin) -> Self {
        let signer = Arc::new(GeodeSigner::new(&args.geode_url, Duration::from_secs(20)));
        Self::with_signer(args, root, origin, signer)
    }

    pub fn with_signer(
        args: &SourceArgs,
        root: &Path,
        origin: Origin,
        signer: Arc<dyn Signer>,
    ) -> Self {
        let public_url = args
            .public_url
            .as_deref()
            .map(|u| u.trim_end_matches('/').to_string())
            .filter(|u| u.starts_with("https://") || u.starts_with("http://"));
        SourceDelivery {
            manifests: root.join(&args.manifests),
            media_index: root.join(&args.media_index),
            public_url,
            catalog: OnceCell::new(),
            urls: SignedUrls::new(
                signer,
                Duration::from_secs(args.margin_s),
                URL_CACHE_CAPACITY,
                args.geode_concurrency,
            ),
            origin,
        }
    }

    /// The catalog, read once on first use (off the reactor).
    async fn catalog(&self) -> Result<Arc<Catalog>, ErrorEnvelope> {
        self.catalog
            .get_or_try_init(|| async {
                let dir = self.manifests.clone();
                let cat = tokio::task::spawn_blocking(move || Catalog::load(&dir))
                    .await
                    .map_err(|e| envelope(ErrorCode::Internal, format!("catalog task: {e}")))?
                    .map_err(|e| {
                        envelope(ErrorCode::Internal, format!("source manifests: {e:#}"))
                    })?;
                for (file, n) in &cat.rejected {
                    tracing::warn!(file, lines = n, "source manifest lines not readable");
                }
                tracing::info!(sources = cat.len(), dir = %self.manifests.display(), "source catalog loaded");
                Ok(Arc::new(cat))
            })
            .await
            .cloned()
    }

    /// The catalog entry of `id`, after the scope checks.
    async fn entry(&self, scope: &ExecutionScope, id: &SourceId) -> Result<Entry, ErrorEnvelope> {
        scope.require(Capability::ReadSources)?;
        let cat = self.catalog().await?;
        let e = cat
            .get(id)
            .cloned()
            .ok_or_else(|| envelope(ErrorCode::NotFound, format!("unknown source {id}")))?;
        // Originals of a lockbox investigation hold outcomes of held-out tests; an authoring scope
        // reads those investigations only through the lockbox (stripped passages), never raw.
        let lockbox = e
            .investigation
            .as_deref()
            .and_then(ignis_data::sources::storage::flight)
            .is_some_and(|f| f.lockbox);
        if lockbox && scope.role == ScopeRole::Authoring {
            return Err(envelope(
                ErrorCode::ScopeDenied,
                format!(
                    "scope {}: originals of {} are not opened in an authoring scope (lockbox)",
                    scope.scope_id,
                    e.investigation.as_deref().unwrap_or_default()
                ),
            ));
        }
        Ok(e)
    }

    /// The keyframe index of a `.ts` source, if one exists and is valid for this version.
    async fn keyframes(&self, id: &SourceId, size: Option<u64>) -> Option<hls::KeyframeIndex> {
        let path = self
            .media_index
            .join(format!("{}.json", id.as_str().replace(':', "-")));
        let size = size?;
        let read = tokio::task::spawn_blocking(move || std::fs::read(&path).ok())
            .await
            .ok()
            .flatten()?;
        let parsed: Result<hls::KeyframeIndex, String> =
            serde_json::from_slice(&read).map_err(|e| e.to_string());
        match parsed.and_then(|ix| ix.validate(id, size).map(|_| ix)) {
            Ok(ix) => Some(ix),
            Err(reason) => {
                tracing::warn!(source = %id, reason, "keyframe index refused; no playlist");
                None
            }
        }
    }

    fn timestamp(s: &Signed) -> Result<Timestamp, ErrorEnvelope> {
        Timestamp::parse(&s.expires_at.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .map_err(|e| envelope(ErrorCode::Internal, e.to_string()))
    }

    fn api_url(&self, public: &str, id: &SourceId, tail: &str) -> String {
        format!(
            "{public}/api/v1/sources/{}/{tail}",
            ignis_data::sources::psi::quote(id.as_str()).replace("%3A", ":")
        )
    }

    /// `SourceAccess` for `id`. `fresh` re-resolves a signed URL the client saw refused.
    pub async fn access(
        &self,
        scope: &ExecutionScope,
        id: &SourceId,
        fresh: bool,
    ) -> Result<SourceAccess, ErrorEnvelope> {
        let e = self.entry(scope, id).await?;
        let mk = |mode, url: String, expires_at, range, cors| SourceAccess {
            schema_version: Default::default(),
            source_id: id.clone(),
            mode,
            url,
            expires_at,
            range,
            cors,
            landing_url: e.landing_url.clone(),
            origin: self.origin,
        };
        let access = match &e.address {
            Address::Psi { file, size_bytes } => {
                let signed = self.urls.get(file, fresh).await?;
                let expires = Some(Self::timestamp(&signed)?);
                let playlist = match (&self.public_url, file.is_ts()) {
                    (Some(p), true) => self
                        .keyframes(id, *size_bytes)
                        .await
                        .map(|_| self.api_url(p, id, "playlist.m3u8")),
                    _ => None,
                };
                match playlist {
                    // The playlist answers no Range itself; its segments do.
                    Some(url) => mk(AccessMode::Direct, url, expires, false, true),
                    None => mk(AccessMode::Direct, signed.url.clone(), expires, true, true),
                }
            }
            Address::Ntrs { pdf_url } => {
                match (&self.public_url, pdf_url.as_deref().and_then(ntrs_path)) {
                    (Some(p), Some(path)) => mk(
                        AccessMode::Proxy,
                        format!("{p}/ntrs{path}"),
                        None,
                        true,
                        true,
                    ),
                    _ => mk(
                        AccessMode::LinkOut,
                        e.landing_url.clone(),
                        None,
                        false,
                        false,
                    ),
                }
            }
            Address::Permanent { url, range, cors } => {
                mk(AccessMode::Direct, url.clone(), None, *range, *cors)
            }
            Address::LinkOut => mk(
                AccessMode::LinkOut,
                e.landing_url.clone(),
                None,
                false,
                false,
            ),
        };
        access
            .validate()
            .map_err(|err| envelope(ErrorCode::Internal, err.to_string()))?;
        Ok(access)
    }

    /// The byte-range HLS playlist of a `.ts` source (fails closed without a valid index).
    pub async fn playlist(
        &self,
        scope: &ExecutionScope,
        id: &SourceId,
    ) -> Result<String, ErrorEnvelope> {
        let e = self.entry(scope, id).await?;
        let Address::Psi { file, size_bytes } = &e.address else {
            return Err(envelope(
                ErrorCode::Unsupported,
                format!("{id} is not a PSI video"),
            ));
        };
        if !file.is_ts() {
            return Err(envelope(
                ErrorCode::Unsupported,
                format!("{id} is not an MPEG-TS file"),
            ));
        }
        let index = self.keyframes(id, *size_bytes).await.ok_or_else(|| {
            envelope(
                ErrorCode::SourceMissing,
                format!("no valid keyframe index for {id} (task R11)"),
            )
        })?;
        let signed = self.urls.get(file, false).await?;
        hls::playlist(&index, &signed.url, hls::DEFAULT_TARGET_S)
            .map_err(|r| envelope(ErrorCode::Internal, r))
    }

    /// Exposed for tests of the cache: the signed URL of one PSI file.
    pub async fn signed(&self, file: &PsiFile, fresh: bool) -> Result<Arc<Signed>, ErrorEnvelope> {
        self.urls.get(file, fresh).await
    }
}

/// `/api/citations/<11 digits>/downloads/<file>` of an NTRS download URL, the only shape the
/// Worker proxy forwards; anything else is not proxied.
pub fn ntrs_path(pdf_url: &str) -> Option<String> {
    let path = pdf_url.strip_prefix("https://ntrs.nasa.gov")?;
    let rest = path.strip_prefix("/api/citations/")?;
    let (id, rest) = rest.split_once('/')?;
    let file = rest.strip_prefix("downloads/")?;
    let ok_id = id.len() == 11 && id.bytes().all(|b| b.is_ascii_digit());
    let ok_file = !file.is_empty()
        && !file.starts_with('.')
        && file
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b));
    (ok_id && ok_file).then(|| path.to_string())
}

// ------------------------------------------------------------------------------ routes

#[derive(Deserialize, JsonSchema)]
struct SourcePath {
    id: SourceId,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ContentQuery {
    /// Re-resolve the signed URL (the previous one was refused with 403).
    #[serde(default)]
    fresh: bool,
}

fn delivery(v: &V1) -> ApiResult<Arc<SourceDelivery>> {
    v.sources
        .clone()
        .ok_or_else(|| ApiError::unavailable("source access"))
}

async fn content(
    State(v): State<Arc<V1>>,
    c: Caller,
    UrlPath(p): UrlPath<SourcePath>,
    Query(q): Query<ContentQuery>,
) -> ApiResult<Response> {
    let d = delivery(&v)?;
    let access = d.access(&c.scope, &p.id, q.fresh).await?;
    // A signed URL is short-lived and per-request: never cached by an intermediary.
    Ok(([(header::CACHE_CONTROL, "private, no-store")], Json(access)).into_response())
}

async fn playlist(
    State(v): State<Arc<V1>>,
    c: Caller,
    UrlPath(p): UrlPath<SourcePath>,
) -> ApiResult<Response> {
    let d = delivery(&v)?;
    let m3u8 = d.playlist(&c.scope, &p.id).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/vnd.apple.mpegurl"),
            (header::CACHE_CONTROL, "private, no-store"),
        ],
        m3u8,
    )
        .into_response())
}

/// The routes of this module, merged into the `/api/v1` command router.
pub fn routes() -> ApiRouter<Arc<V1>> {
    ApiRouter::new()
        .api_route(
            "/sources/{id}/content",
            get_with(content, |o| {
                o.summary("How to open a source version's original at NASA (SourceAccess; no copy)")
                    .response::<200, Json<SourceAccess>>()
            }),
        )
        .api_route(
            "/sources/{id}/playlist.m3u8",
            get_with(playlist, |o| {
                o.summary(
                    "Byte-range HLS playlist of an MPEG-TS source (segments stream from NASA)",
                )
            }),
        )
}

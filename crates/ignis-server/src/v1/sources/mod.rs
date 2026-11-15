//! Source access without copies (task M12, LANES_V2 §10): `GET /sources/{id}/content` →
//! [`SourceAccess`].
//!
//! The originals stay at NASA. Per source version, from the R6 manifests ([`catalog`]):
//! * PSI: a fresh signed S3 URL from geode `redirect=false` ([`resolver`], single-flight, cached
//!   until expiry minus a margin), `direct` (S3 answers Range and CORS `*`);
//! * PSI CFI-G `.ts` with a keyframe index (R11, `ignis sources media keyframes`): a byte-range
//!   HLS playlist (`GET /sources/{id}/playlist.m3u8`, [`hls`]) whose segments are that same signed
//!   URL, so the viewer can seek; an I-frames-only playlist (`iframes.m3u8`) for scrubbing and a
//!   master playlist (`master.m3u8`) that lists both; without an index, the signed URL itself
//!   (plays, no seek);
//! * PSI ZIP archives (R11): `GET /sources/{id}/archive` pages the entries from the archive's
//!   central directory (read by byte range once, cached under `--media-cache`), and
//!   `GET /sources/{id}/archive/entry?name=…` serves one entry read by byte range, TIFF frames
//!   converted to PNG (faithful) or WebP (display) on request and cached by content. These are
//!   the only routes where NASA bytes pass through IGNIS: an entry must be cut out of the archive
//!   (and a TIFF converted) before a browser can show it; the archive is never downloaded;
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
    AccessMode, ArchiveEntry, ArchiveListing, Capability, ErrorCode, ErrorEnvelope, ExecutionScope,
    MediaKeyframeIndex, Origin, RetryPolicy, ScopeRole, SourceAccess, SourceId, Timestamp,
};
use ignis_data::media::frames::{self, FrameFormat};
use ignis_data::media::range::HttpRange;
use ignis_data::media::zipdir::{self, DirCache, ZipDirectory};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::sync::{OnceCell, Semaphore};

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
    /// Cache of ZIP central directories (`zipdir/`) and converted frames (`frames/`), task R11.
    #[arg(
        long = "media-cache",
        env = "IGNIS_MEDIA_CACHE",
        default_value = "data/media"
    )]
    pub media_cache: PathBuf,
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
    /// One geode call fails as `upstream_unavailable` after this long.
    #[arg(long = "geode-timeout-s", default_value_t = 20)]
    pub geode_timeout_s: u64,
}

impl Default for SourceArgs {
    fn default() -> Self {
        SourceArgs {
            manifests: "data/manifests".into(),
            media_index: "data/media/keyframes".into(),
            media_cache: "data/media".into(),
            public_url: None,
            geode_url: ignis_data::sources::psi::BASE.into(),
            margin_s: 300,
            geode_concurrency: 2,
            geode_timeout_s: 20,
        }
    }
}

/// Signed URLs kept (one per PSI file version being viewed).
const URL_CACHE_CAPACITY: u64 = 10_000;

/// Largest archive entry read into memory (a frame is a few MB; a memory limit, not a policy).
const MAX_ENTRY_BYTES: u64 = 256 << 20;

/// Entries per archive page by default and at most (pagination, not a cap on the archive).
const ARCHIVE_PAGE: usize = 500;
const ARCHIVE_PAGE_MAX: usize = 5_000;

/// Byte-range reads of one archive request fail after this long.
const RANGE_TIMEOUT: Duration = Duration::from_secs(60);

/// Which HLS playlist of a `.ts` source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaylistKind {
    /// Keyframe-aligned media segments (what hls.js plays).
    Media,
    /// One range per indexed keyframe, for scrubbing.
    IFrames,
    /// The media playlist as the one variant plus the I-frame stream.
    Master,
}

/// The source access service of one `/api/v1` instance.
pub struct SourceDelivery {
    manifests: PathBuf,
    media_index: PathBuf,
    media_cache: PathBuf,
    /// Archive reads and frame conversions in flight (blocking work, bounded).
    media_permits: Arc<Semaphore>,
    public_url: Option<String>,
    catalog: OnceCell<Arc<Catalog>>,
    urls: SignedUrls,
    origin: Origin,
}

fn envelope(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

fn upstream_err(msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(
        ErrorCode::UpstreamUnavailable,
        msg,
        RetryPolicy::AfterDelay { seconds: 30 },
    )
}

/// Content type of an original archive entry, by extension.
fn content_type_of(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    match n.rsplit('.').next().unwrap_or_default() {
        "tif" | "tiff" => "image/tiff",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "bmp" => "image/bmp",
        "txt" | "csv" | "log" => "text/plain; charset=utf-8",
        "json" => "application/json",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

impl SourceDelivery {
    /// Production wiring: geode signer, paths resolved against `root`.
    pub fn new(args: &SourceArgs, root: &Path, origin: Origin) -> Self {
        let signer = Arc::new(GeodeSigner::new(
            &args.geode_url,
            Duration::from_secs(args.geode_timeout_s),
        ));
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
            media_cache: root.join(&args.media_cache),
            media_permits: Arc::new(Semaphore::new(
                std::thread::available_parallelism().map_or(2, |n| n.get()),
            )),
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
    async fn keyframes(&self, id: &SourceId, size: Option<u64>) -> Option<MediaKeyframeIndex> {
        let path = self
            .media_index
            .join(ignis_data::media::index_file_name(id.as_str()));
        let size = size?;
        let read = tokio::task::spawn_blocking(move || std::fs::read(&path).ok())
            .await
            .ok()
            .flatten()?;
        let parsed: Result<MediaKeyframeIndex, String> =
            serde_json::from_slice(&read).map_err(|e| e.to_string());
        match parsed.and_then(|ix| hls::check(&ix, id, size).map(|_| ix)) {
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
        self.hls(scope, id, PlaylistKind::Media).await
    }

    /// One of the HLS playlists of a `.ts` source (fails closed without a valid index).
    pub async fn hls(
        &self,
        scope: &ExecutionScope,
        id: &SourceId,
        kind: PlaylistKind,
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
        if kind == PlaylistKind::Master {
            // The master names IGNIS URLs only; the signed URL goes into the two playlists.
            let public = self.public_url.as_deref().ok_or_else(|| {
                envelope(
                    ErrorCode::Unsupported,
                    "a master playlist needs the public URL (--public-url)",
                )
            })?;
            return hls::master_playlist(
                &index,
                &self.api_url(public, id, "playlist.m3u8"),
                &self.api_url(public, id, "iframes.m3u8"),
                hls::DEFAULT_TARGET_S,
            )
            .map_err(|r| envelope(ErrorCode::Internal, r));
        }
        // A VOD playlist is loaded once; its ranges must stay valid for the whole video. The cache
        // guarantees at least the margin; when that is shorter than the video, re-sign once.
        let mut signed = self.urls.get(file, false).await?;
        let left = (signed.expires_at - chrono::Utc::now()).num_milliseconds() as f64 / 1000.0;
        if left < index.duration_s() {
            signed = self.urls.get(file, true).await?;
        }
        match kind {
            PlaylistKind::IFrames => hls::iframes_playlist(&index, &signed.url),
            _ => hls::playlist(&index, &signed.url, hls::DEFAULT_TARGET_S),
        }
        .map_err(|r| envelope(ErrorCode::Internal, r))
    }

    /// The PSI ZIP behind `id` and its central directory (cached; read by byte range once).
    async fn archive_directory(
        &self,
        scope: &ExecutionScope,
        id: &SourceId,
    ) -> Result<(PsiFile, Option<u64>, Arc<ZipDirectory>), ErrorEnvelope> {
        let e = self.entry(scope, id).await?;
        let Address::Psi { file, size_bytes } = e.address else {
            return Err(envelope(
                ErrorCode::Unsupported,
                format!("{id} is not a PSI archive"),
            ));
        };
        if !file.file_name.to_ascii_lowercase().ends_with(".zip") {
            return Err(envelope(
                ErrorCode::Unsupported,
                format!("{id} is not a ZIP archive"),
            ));
        }
        let dir = self.media_cache.join("zipdir");
        let key = id.as_str().to_string();
        // A cached directory needs no signed URL.
        if let Some(size) = size_bytes {
            let (k, c) = (key.clone(), DirCache::new(&dir));
            let hit = tokio::task::spawn_blocking(move || c.load(&k, size))
                .await
                .map_err(|e| envelope(ErrorCode::Internal, format!("cache task: {e}")))?
                .ok()
                .flatten();
            if let Some(d) = hit {
                return Ok((file, size_bytes, Arc::new(d)));
            }
        }
        let signed = self.urls.get(&file, false).await?;
        let _permit = self
            .media_permits
            .acquire()
            .await
            .map_err(|_| envelope(ErrorCode::Internal, "media permits closed"))?;
        let url = signed.url.clone();
        let d = tokio::task::spawn_blocking(move || {
            let r = HttpRange::open(&url, size_bytes, RANGE_TIMEOUT)?;
            DirCache::new(dir).get(&key, &r)
        })
        .await
        .map_err(|e| envelope(ErrorCode::Internal, format!("archive task: {e}")))?
        .map_err(|e| upstream_err(format!("archive directory: {e:#}")))?;
        Ok((file, size_bytes, Arc::new(d)))
    }

    /// A page of an archive's entries, in central-directory order.
    pub async fn archive(
        &self,
        scope: &ExecutionScope,
        id: &SourceId,
        after: Option<u64>,
        limit: Option<usize>,
    ) -> Result<ArchiveListing, ErrorEnvelope> {
        let (_, _, d) = self.archive_directory(scope, id).await?;
        let total = d.entries.len() as u64;
        let first = after.unwrap_or(0).min(total);
        let n = limit.unwrap_or(ARCHIVE_PAGE).clamp(1, ARCHIVE_PAGE_MAX) as u64;
        let end = (first + n).min(total);
        let entries = d.entries[first as usize..end as usize]
            .iter()
            .map(|e| ArchiveEntry {
                name: e.name.clone(),
                size_bytes: e.size,
                compressed_bytes: e.compressed_size,
                frame: frames::is_tiff(&e.name),
            })
            .collect();
        let page = ArchiveListing {
            schema_version: Default::default(),
            source_id: id.clone(),
            archive_size_bytes: d.size_bytes,
            total_entries: total,
            first,
            entries,
            next_after: (end < total).then_some(end),
            origin: self.origin,
        };
        page.validate()
            .map_err(|err| envelope(ErrorCode::Internal, err.to_string()))?;
        Ok(page)
    }

    /// One archive entry (`format` png/webp converts a TIFF frame): bytes and content type.
    pub async fn archive_entry(
        &self,
        scope: &ExecutionScope,
        id: &SourceId,
        name: &str,
        format: Option<FrameFormat>,
    ) -> Result<(Vec<u8>, &'static str), ErrorEnvelope> {
        let (file, size_bytes, d) = self.archive_directory(scope, id).await?;
        let entry = d.entry(name).cloned().ok_or_else(|| {
            envelope(
                ErrorCode::NotFound,
                format!("{name:?} is not an entry of {id}"),
            )
        })?;
        if entry.size > MAX_ENTRY_BYTES || entry.compressed_size > MAX_ENTRY_BYTES {
            return Err(envelope(
                ErrorCode::Unsupported,
                format!(
                    "{name}: {} B is more than one request reads into memory",
                    entry.size
                ),
            ));
        }
        if format.is_some() && !frames::is_tiff(&entry.name) {
            return Err(envelope(
                ErrorCode::InvalidRequest,
                format!("{name} is not a TIFF frame; ask for the original"),
            ));
        }
        let signed = self.urls.get(&file, false).await?;
        let _permit = self
            .media_permits
            .acquire()
            .await
            .map_err(|_| envelope(ErrorCode::Internal, "media permits closed"))?;
        let url = signed.url.clone();
        let frames_dir = self.media_cache.join("frames");
        let bytes = tokio::task::spawn_blocking(move || {
            let r = HttpRange::open(&url, size_bytes, RANGE_TIMEOUT)?;
            let raw = zipdir::read_entry(&r, &entry)?;
            match format {
                Some(f) => Ok(frames::FrameCache::new(frames_dir).get(&raw, f)?.0),
                None => anyhow::Ok(raw),
            }
        })
        .await
        .map_err(|e| envelope(ErrorCode::Internal, format!("entry task: {e}")))?
        .map_err(|e| upstream_err(format!("archive entry: {e:#}")))?;
        let ctype = match format {
            Some(f) => f.content_type(),
            None => content_type_of(name),
        };
        Ok((bytes, ctype))
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

async fn m3u8(v: Arc<V1>, c: Caller, id: SourceId, kind: PlaylistKind) -> ApiResult<Response> {
    let d = delivery(&v)?;
    let m3u8 = d.hls(&c.scope, &id, kind).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/vnd.apple.mpegurl"),
            (header::CACHE_CONTROL, "private, no-store"),
        ],
        m3u8,
    )
        .into_response())
}

async fn playlist(
    State(v): State<Arc<V1>>,
    c: Caller,
    UrlPath(p): UrlPath<SourcePath>,
) -> ApiResult<Response> {
    m3u8(v, c, p.id, PlaylistKind::Media).await
}

async fn iframes(
    State(v): State<Arc<V1>>,
    c: Caller,
    UrlPath(p): UrlPath<SourcePath>,
) -> ApiResult<Response> {
    m3u8(v, c, p.id, PlaylistKind::IFrames).await
}

async fn master(
    State(v): State<Arc<V1>>,
    c: Caller,
    UrlPath(p): UrlPath<SourcePath>,
) -> ApiResult<Response> {
    m3u8(v, c, p.id, PlaylistKind::Master).await
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ArchiveQuery {
    /// Index of the first entry (the previous page's `next_after`).
    #[serde(default)]
    after: Option<u64>,
    /// Entries per page (default 500, at most 5000).
    #[serde(default)]
    limit: Option<usize>,
}

async fn archive(
    State(v): State<Arc<V1>>,
    c: Caller,
    UrlPath(p): UrlPath<SourcePath>,
    Query(q): Query<ArchiveQuery>,
) -> ApiResult<Response> {
    let d = delivery(&v)?;
    let page = d.archive(&c.scope, &p.id, q.after, q.limit).await?;
    // A source version's directory never changes.
    Ok((
        [(header::CACHE_CONTROL, "private, max-age=86400")],
        Json(page),
    )
        .into_response())
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct EntryQuery {
    /// Entry path inside the archive.
    name: String,
    /// `png` or `webp` converts a TIFF frame; `original` (default) serves the entry's bytes.
    #[serde(default)]
    format: Option<String>,
}

async fn archive_entry(
    State(v): State<Arc<V1>>,
    c: Caller,
    UrlPath(p): UrlPath<SourcePath>,
    Query(q): Query<EntryQuery>,
) -> ApiResult<Response> {
    let d = delivery(&v)?;
    let format = match q.format.as_deref() {
        None | Some("original") => None,
        Some(f) => Some(FrameFormat::parse(f).ok_or_else(|| {
            envelope(
                ErrorCode::InvalidRequest,
                format!("format {f:?}: png, webp or original"),
            )
        })?),
    };
    let (bytes, ctype) = d.archive_entry(&c.scope, &p.id, &q.name, format).await?;
    Ok((
        [
            (header::CONTENT_TYPE, ctype),
            // Bytes of one immutable source version (and a versioned converter).
            (
                header::CACHE_CONTROL,
                "private, max-age=31536000, immutable",
            ),
        ],
        bytes,
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
        .api_route(
            "/sources/{id}/iframes.m3u8",
            get_with(iframes, |o| {
                o.summary("I-frames-only HLS playlist of an MPEG-TS source (one keyframe per range)")
            }),
        )
        .api_route(
            "/sources/{id}/master.m3u8",
            get_with(master, |o| {
                o.summary("HLS master playlist of an MPEG-TS source (media + I-frame streams)")
            }),
        )
        .api_route(
            "/sources/{id}/archive",
            get_with(archive, |o| {
                o.summary("Entries of a remote ZIP source, paged (central directory read by byte range)")
                    .response::<200, Json<ArchiveListing>>()
            }),
        )
        .api_route(
            "/sources/{id}/archive/entry",
            get_with(archive_entry, |o| {
                o.summary("One entry of a remote ZIP source, read by byte range; TIFF frames as PNG or WebP")
            }),
        )
}

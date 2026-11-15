//! Media indexes for streaming NASA media without copies (R11, LANES_V2 §10).
//!
//! * [`ts`]: keyframe index of an MPEG-TS recording ([`MediaKeyframeIndex`], read by M12's
//!   byte-range HLS playlists), built from byte ranges.
//! * [`zipdir`]: central directory of a remote ZIP read by byte range and cached; one entry read
//!   without the archive.
//! * [`frames`]: TIFF → PNG/WebP on demand, content-addressed cache.
//! * [`range`]: the byte-range readers they share (local file, HTTP `Range`, memory).
//!
//! Everything reads through [`range::RangeRead`]; nothing downloads a whole media object.
//!
//! [`MediaKeyframeIndex`]: ignis_core::contracts::MediaKeyframeIndex

pub mod cli;
pub mod frames;
pub mod range;
pub mod synth;
pub mod ts;
pub mod zipdir;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{ensure, Context};
use serde_json::json;

use crate::paths;
use crate::sources::intl::common::listing_id;
use crate::sources::psi::{self, RemoteFile};

/// Where keyframe indexes are written (the server's `--media-index` default resolves here).
pub fn keyframes_dir() -> PathBuf {
    paths::root().join("media").join("keyframes")
}

/// The cache root of ZIP directories and converted frames (the server's `--media-cache`).
pub fn cache_dir() -> PathBuf {
    paths::root().join("media")
}

/// File name of a source's index: the id with `:` as `-` (what M12 looks up).
pub fn index_file_name(source_id: &str) -> String {
    format!("{}.json", source_id.replace(':', "-"))
}

/// The source id of a PSI file kept only as a manifest entry: the listing id of its stable geode
/// URL and size (the convention of the server's catalog, `v1/sources/catalog.rs`).
pub fn listing_source_id(
    investigation: &str,
    file_name: &str,
    file_version: Option<i64>,
    size_bytes: Option<i64>,
) -> String {
    listing_id(json!({
        "url": psi::stable_url(investigation, file_name, file_version),
        "size": size_bytes,
    }))
}

impl RemoteFile {
    pub fn listing_source_id(&self) -> String {
        listing_source_id(
            &self.investigation,
            &self.file_name,
            self.file_version,
            self.size_bytes,
        )
    }
}

/// A signed, expiring S3 URL for one PSI file version: geode `redirect=false` (one GET, nothing
/// downloaded). Only an `https` URL on a PSI bucket is accepted.
pub fn sign(f: &RemoteFile, timeout: Duration) -> anyhow::Result<String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .user_agent(psi::UA)
        .timeout_global(Some(timeout))
        .max_redirects(0)
        .build()
        .into();
    let url = format!("{}&redirect=false", f.stable_url());
    let mut r = agent
        .get(&url)
        .call()
        .with_context(|| format!("geode {url}"))?;
    let body = r
        .body_mut()
        .with_config()
        .limit(16 * 1024)
        .read_to_string()?;
    let signed = body.trim().to_string();
    let host = signed
        .strip_prefix("https://")
        .and_then(|r| r.split(['/', '?']).next())
        .unwrap_or_default();
    ensure!(
        host.ends_with(".amazonaws.com") && host.starts_with("psi-"),
        "geode answered something other than a PSI bucket URL"
    );
    ensure!(
        !signed.chars().any(|c| c.is_whitespace() || c.is_control()),
        "geode answer is not one URL"
    );
    Ok(signed)
}

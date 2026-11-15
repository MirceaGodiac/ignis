//! Byte-range readers: a local file or one HTTP object read with `Range` requests.
//!
//! Every media indexer reads through [`RangeRead`], so the same code indexes a local test file
//! and a 1 GB recording at NASA while fetching only the ranges it needs. A [`Meter`] counts
//! requests and bytes, so a run reports what it actually read.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{bail, ensure, Context};

/// Requests made and bytes received by one reader.
#[derive(Debug, Default)]
pub struct Meter {
    requests: AtomicU64,
    bytes: AtomicU64,
}

impl Meter {
    fn add(&self, n: u64) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(n, Ordering::Relaxed);
    }
    pub fn requests(&self) -> u64 {
        self.requests.load(Ordering::Relaxed)
    }
    pub fn bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }
}

/// Random access to one immutable object of known size.
pub trait RangeRead: Send + Sync {
    fn size(&self) -> u64;
    /// Bytes `[offset, offset + len)`, clamped to the end of the object. Short reads are errors.
    fn read_range(&self, offset: u64, len: u64) -> anyhow::Result<Vec<u8>>;
    fn meter(&self) -> &Meter;
}

fn clamp(size: u64, offset: u64, len: u64) -> anyhow::Result<u64> {
    ensure!(
        offset <= size,
        "range starts at {offset}, past the end ({size} B)"
    );
    Ok(len.min(size - offset))
}

/// A local file (tests, or a copy that already exists on disk).
pub struct FileRange {
    file: Mutex<File>,
    size: u64,
    meter: Meter,
}

impl FileRange {
    pub fn open(path: &Path) -> anyhow::Result<FileRange> {
        let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
        let size = file.metadata()?.len();
        Ok(FileRange {
            file: Mutex::new(file),
            size,
            meter: Meter::default(),
        })
    }
}

impl RangeRead for FileRange {
    fn size(&self) -> u64 {
        self.size
    }
    fn read_range(&self, offset: u64, len: u64) -> anyhow::Result<Vec<u8>> {
        let len = clamp(self.size, offset, len)?;
        let mut buf = vec![0u8; len as usize];
        let mut f = self.file.lock().expect("file lock");
        f.seek(SeekFrom::Start(offset))?;
        f.read_exact(&mut buf)?;
        self.meter.add(len);
        Ok(buf)
    }
    fn meter(&self) -> &Meter {
        &self.meter
    }
}

/// An object behind an HTTP URL that answers `Range` with `206` (a signed S3 URL).
pub struct HttpRange {
    agent: ureq::Agent,
    url: String,
    size: u64,
    meter: Meter,
}

/// `bytes <first>-<last>/<total>` of a `Content-Range` header.
pub fn parse_content_range(v: &str) -> Option<(u64, u64, u64)> {
    let (range, total) = v.trim().strip_prefix("bytes ")?.split_once('/')?;
    let (a, b) = range.split_once('-')?;
    Some((a.parse().ok()?, b.parse().ok()?, total.parse().ok()?))
}

impl HttpRange {
    /// Opens `url`; with `size = None` one 1-byte request learns the size from `Content-Range`.
    /// A known size is checked against every answer, so a changed object fails instead of mixing
    /// bytes of two versions.
    pub fn open(url: &str, size: Option<u64>, timeout: Duration) -> anyhow::Result<HttpRange> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .user_agent(crate::sources::psi::UA)
            .timeout_global(Some(timeout))
            .max_redirects(0)
            .build()
            .into();
        let mut r = HttpRange {
            agent,
            url: url.to_string(),
            size: size.unwrap_or(u64::MAX),
            meter: Meter::default(),
        };
        if size.is_none() {
            let (_, total) = r.get(0, 1)?;
            r.size = total;
        }
        Ok(r)
    }

    fn get(&self, offset: u64, len: u64) -> anyhow::Result<(Vec<u8>, u64)> {
        let last = offset + len - 1;
        let mut resp = self
            .agent
            .get(&self.url)
            .header("Range", &format!("bytes={offset}-{last}"))
            .call()
            .with_context(|| format!("ranged GET bytes={offset}-{last}"))?;
        if resp.status().as_u16() != 206 {
            bail!(
                "server answered {} to a Range request (needs 206)",
                resp.status()
            );
        }
        let cr = resp
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_content_range)
            .context("206 without a readable Content-Range")?;
        ensure!(
            cr.0 == offset && cr.1 == last,
            "asked bytes={offset}-{last}, got {}-{}",
            cr.0,
            cr.1
        );
        if self.size != u64::MAX {
            ensure!(
                cr.2 == self.size,
                "object is {} B, expected {} B: it changed",
                cr.2,
                self.size
            );
        }
        let body = resp.body_mut().with_config().limit(len + 1).read_to_vec()?;
        ensure!(
            body.len() as u64 == len,
            "short body: {} of {len} B",
            body.len()
        );
        self.meter.add(len);
        Ok((body, cr.2))
    }
}

impl RangeRead for HttpRange {
    fn size(&self) -> u64 {
        self.size
    }
    fn read_range(&self, offset: u64, len: u64) -> anyhow::Result<Vec<u8>> {
        let len = clamp(self.size, offset, len)?;
        if len == 0 {
            return Ok(Vec::new());
        }
        Ok(self.get(offset, len)?.0)
    }
    fn meter(&self) -> &Meter {
        &self.meter
    }
}

/// An in-memory object (tests).
pub struct MemRange {
    pub bytes: Vec<u8>,
    meter: Meter,
}

impl MemRange {
    pub fn new(bytes: Vec<u8>) -> MemRange {
        MemRange {
            bytes,
            meter: Meter::default(),
        }
    }
}

impl RangeRead for MemRange {
    fn size(&self) -> u64 {
        self.bytes.len() as u64
    }
    fn read_range(&self, offset: u64, len: u64) -> anyhow::Result<Vec<u8>> {
        let len = clamp(self.size(), offset, len)?;
        self.meter.add(len);
        Ok(self.bytes[offset as usize..(offset + len) as usize].to_vec())
    }
    fn meter(&self) -> &Meter {
        &self.meter
    }
}

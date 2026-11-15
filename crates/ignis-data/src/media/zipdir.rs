//! Remote ZIP archives read by byte range (R11): the central directory once, cached; one entry
//! served without downloading the archive.
//!
//! The PSI image archives (BASS/SAFFIRE `*_RAW2TIFF.zip`, up to 18 GB) stay at NASA. The central
//! directory of an 18 GB archive is 0.44 MB (*measured*, LANES_V2 §5), so after one tail read and
//! one directory read (cached), a frame costs one ranged request: its local header and data are
//! read together. ZIP64 archives are read through the ZIP64 end-of-central-directory record.
//!
//! Supported entries: stored (method 0) and deflate (method 8), unencrypted. Every entry read is
//! checked against the directory's size and CRC-32, so a wrong range never passes as a frame.

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::range::RangeRead;

const EOCD: [u8; 4] = *b"PK\x05\x06";
const LOC64: [u8; 4] = *b"PK\x06\x07";
const EOCD64: [u8; 4] = *b"PK\x06\x06";
const CDH: [u8; 4] = *b"PK\x01\x02";
const LFH: [u8; 4] = *b"PK\x03\x04";

/// Extra bytes read past the expected local header, so a local extra field longer than the
/// central one rarely costs a second request.
const LOCAL_SLACK: u64 = 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZipEntry {
    pub name: String,
    pub method: u16,
    pub flags: u16,
    pub crc32: u32,
    pub compressed_size: u64,
    pub size: u64,
    pub local_header_offset: u64,
}

impl ZipEntry {
    pub fn is_dir(&self) -> bool {
        self.name.ends_with('/')
    }
}

/// The central directory of one archive version.
#[derive(Debug, Clone)]
pub struct ZipDirectory {
    pub size_bytes: u64,
    pub cd_offset: u64,
    /// Raw central-directory bytes (what the cache stores).
    pub raw: Vec<u8>,
    pub entries: Vec<ZipEntry>,
}

impl ZipDirectory {
    pub fn entry(&self, name: &str) -> Option<&ZipEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}

fn u16le(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}
fn u32le(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
fn u64le(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}

/// Finds the central directory (offset, size, entry count) from the archive's tail.
fn locate(r: &dyn RangeRead) -> anyhow::Result<(u64, u64, u64)> {
    let size = r.size();
    ensure!(size >= 22, "{size} B is too small for a ZIP archive");
    let tail_len = size.min(22 + 0xffff + 20);
    let tail_at = size - tail_len;
    let tail = r.read_range(tail_at, tail_len)?;
    let i = (0..=tail.len() - 22)
        .rev()
        .find(|&i| tail[i..i + 4] == EOCD && i + 22 + u16le(&tail, i + 20) as usize <= tail.len())
        .context("no ZIP end-of-central-directory record")?;
    let entries = u16le(&tail, i + 10) as u64;
    let cd_size = u32le(&tail, i + 12) as u64;
    let cd_offset = u32le(&tail, i + 16) as u64;
    if entries != 0xffff && cd_size != 0xffff_ffff && cd_offset != 0xffff_ffff {
        return Ok((cd_offset, cd_size, entries));
    }
    // ZIP64: the locator sits right before the classic record.
    ensure!(
        i >= 20 && tail[i - 20..i - 16] == LOC64,
        "ZIP64 archive without a ZIP64 locator"
    );
    let at = u64le(&tail, i - 20 + 8);
    let rec = if at >= tail_at && at + 56 <= size && ((at - tail_at) as usize + 56) <= tail.len() {
        tail[(at - tail_at) as usize..(at - tail_at) as usize + 56].to_vec()
    } else {
        r.read_range(at, 56)?
    };
    ensure!(
        rec.len() == 56 && rec[..4] == EOCD64,
        "no ZIP64 end-of-central-directory record at {at}"
    );
    Ok((u64le(&rec, 48), u64le(&rec, 40), u64le(&rec, 32)))
}

/// Parses raw central-directory bytes.
pub fn parse_central(raw: &[u8]) -> anyhow::Result<Vec<ZipEntry>> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 46 <= raw.len() {
        ensure!(raw[i..i + 4] == CDH, "bad central-directory header at +{i}");
        let flags = u16le(raw, i + 8);
        let method = u16le(raw, i + 10);
        let crc32 = u32le(raw, i + 16);
        let mut csize = u32le(raw, i + 20) as u64;
        let mut usize_ = u32le(raw, i + 24) as u64;
        let n = u16le(raw, i + 28) as usize;
        let m = u16le(raw, i + 30) as usize;
        let k = u16le(raw, i + 32) as usize;
        let mut lho = u32le(raw, i + 42) as u64;
        let end = i + 46 + n + m + k;
        ensure!(
            end <= raw.len(),
            "central-directory entry at +{i} runs past the end"
        );
        let name_bytes = &raw[i + 46..i + 46 + n];
        let name = if flags & 0x800 != 0 {
            String::from_utf8(name_bytes.to_vec()).context("entry name is not UTF-8")?
        } else {
            // CP437 names: the archives here are ASCII; anything else is kept lossily.
            String::from_utf8_lossy(name_bytes).into_owned()
        };
        // ZIP64 extra field: the 64-bit values of the 32-bit fields set to 0xFFFFFFFF, in order.
        let mut x = i + 46 + n;
        let xend = x + m;
        while x + 4 <= xend {
            let id = u16le(raw, x);
            let len = u16le(raw, x + 2) as usize;
            if id == 1 {
                let mut p = x + 4;
                for field in [&mut usize_, &mut csize, &mut lho] {
                    if *field == 0xffff_ffff && p + 8 <= x + 4 + len {
                        *field = u64le(raw, p);
                        p += 8;
                    }
                }
            }
            x += 4 + len;
        }
        out.push(ZipEntry {
            name,
            method,
            flags,
            crc32,
            compressed_size: csize,
            size: usize_,
            local_header_offset: lho,
        });
        i = end;
    }
    Ok(out)
}

/// Reads the central directory of a remote archive (2 requests, 3 for ZIP64).
pub fn read_directory(r: &dyn RangeRead) -> anyhow::Result<ZipDirectory> {
    let (cd_offset, cd_size, count) = locate(r)?;
    ensure!(
        cd_offset
            .checked_add(cd_size)
            .is_some_and(|e| e <= r.size()),
        "central directory [{cd_offset}, +{cd_size}) lies outside the archive"
    );
    let raw = r.read_range(cd_offset, cd_size)?;
    let entries = parse_central(&raw)?;
    ensure!(
        entries.len() as u64 == count,
        "central directory lists {} entries, the end record says {count}",
        entries.len()
    );
    Ok(ZipDirectory {
        size_bytes: r.size(),
        cd_offset,
        raw,
        entries,
    })
}

/// The decompressed bytes of one entry, checked against its size and CRC-32.
pub fn read_entry(r: &dyn RangeRead, e: &ZipEntry) -> anyhow::Result<Vec<u8>> {
    ensure!(
        e.flags & 1 == 0,
        "{}: encrypted entries are not read",
        e.name
    );
    ensure!(!e.is_dir(), "{} is a directory", e.name);
    let want = 30 + e.name.len() as u64 + LOCAL_SLACK + e.compressed_size;
    let mut buf = r.read_range(e.local_header_offset, want)?;
    ensure!(
        buf.len() >= 30 && buf[..4] == LFH,
        "{}: no local header at {}",
        e.name,
        e.local_header_offset
    );
    let start = 30 + u16le(&buf, 26) as usize + u16le(&buf, 28) as usize;
    let end = start as u64 + e.compressed_size;
    if (buf.len() as u64) < end {
        let more = r.read_range(
            e.local_header_offset + buf.len() as u64,
            end - buf.len() as u64,
        )?;
        buf.extend_from_slice(&more);
    }
    ensure!(
        buf.len() as u64 >= end,
        "{}: archive ends inside the entry",
        e.name
    );
    let data = &buf[start..end as usize];
    let out = match e.method {
        0 => data.to_vec(),
        8 => {
            let mut out = Vec::with_capacity(e.size as usize);
            flate2::read::DeflateDecoder::new(data)
                .take(e.size + 1)
                .read_to_end(&mut out)
                .with_context(|| format!("{}: deflate", e.name))?;
            out
        }
        m => bail!("{}: compression method {m} is not supported", e.name),
    };
    ensure!(
        out.len() as u64 == e.size,
        "{}: {} B decompressed, the directory says {}",
        e.name,
        out.len(),
        e.size
    );
    let crc = crc32fast::hash(&out);
    ensure!(crc == e.crc32, "{}: CRC-32 mismatch", e.name);
    Ok(out)
}

/// Cached central directories: `<key>.cd` (raw bytes) + `<key>.json` (where they came from).
pub struct DirCache {
    dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CacheMeta {
    key: String,
    size_bytes: u64,
    cd_offset: u64,
    cd_size: u64,
    entries: u64,
    sha256: String,
}

fn safe_key(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "-_.".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect()
}

impl DirCache {
    pub fn new(dir: impl Into<PathBuf>) -> DirCache {
        DirCache { dir: dir.into() }
    }

    fn paths(&self, key: &str) -> (PathBuf, PathBuf) {
        let k = safe_key(key);
        (
            self.dir.join(format!("{k}.json")),
            self.dir.join(format!("{k}.cd")),
        )
    }

    /// The cached directory of `key`, if present and for an archive of `size_bytes`.
    pub fn load(&self, key: &str, size_bytes: u64) -> anyhow::Result<Option<ZipDirectory>> {
        let (meta_p, raw_p) = self.paths(key);
        let Ok(meta) = std::fs::read(&meta_p) else {
            return Ok(None);
        };
        let meta: CacheMeta = serde_json::from_slice(&meta)?;
        if meta.key != key || meta.size_bytes != size_bytes {
            return Ok(None);
        }
        let raw = std::fs::read(&raw_p)?;
        if hex::encode(Sha256::digest(&raw)) != meta.sha256 {
            return Ok(None);
        }
        let entries = parse_central(&raw)?;
        Ok(Some(ZipDirectory {
            size_bytes,
            cd_offset: meta.cd_offset,
            raw,
            entries,
        }))
    }

    pub fn store(&self, key: &str, d: &ZipDirectory) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let (meta_p, raw_p) = self.paths(key);
        write_atomic(&raw_p, &d.raw)?;
        let meta = CacheMeta {
            key: key.to_string(),
            size_bytes: d.size_bytes,
            cd_offset: d.cd_offset,
            cd_size: d.raw.len() as u64,
            entries: d.entries.len() as u64,
            sha256: hex::encode(Sha256::digest(&d.raw)),
        };
        write_atomic(&meta_p, &serde_json::to_vec_pretty(&meta)?)
    }

    /// Cached directory, or read it through `r` and cache it.
    pub fn get(&self, key: &str, r: &dyn RangeRead) -> anyhow::Result<ZipDirectory> {
        if let Some(d) = self.load(key, r.size())? {
            return Ok(d);
        }
        let d = read_directory(r)?;
        self.store(key, &d)?;
        Ok(d)
    }
}

/// Writes through a temporary file and a rename, so a reader never sees half a file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".tmp{}", std::process::id()));
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    Ok(())
}

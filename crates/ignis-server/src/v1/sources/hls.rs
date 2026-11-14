//! Byte-range HLS playlists over one MPEG-TS object at NASA (LANES_V2 §10).
//!
//! CFI-G `.ts` recordings (15–996 MB) play from S3 with mpegts.js but cannot seek (no duration, no
//! index). hls.js on a byte-range playlist seeks, provided every segment starts on a keyframe
//! (measured 2026-11-14: even-byte segments fail on seek with a buffer-append error). The keyframe
//! index (byte offset ↔ PTS of each IDR) is task R11 (Raul); until it lands, [`KeyframeIndex`] is
//! the server-internal shape this builder reads, and the only index files are test fixtures. When
//! R11 ships its contract in `ignis-core`, this module reads that type instead.
//!
//! The playlist is a delivery detail: every segment is the same signed URL with a different
//! `EXT-X-BYTERANGE`, so the bytes stream from S3 to the browser and none pass through IGNIS.

use std::fmt::Write as _;

use ignis_core::contracts::SourceId;
use serde::Deserialize;

/// MPEG-TS packet size: a segment boundary that is not a packet boundary cannot be demuxed.
pub const TS_PACKET: u64 = 188;

/// Segment length hls.js was measured to seek with (8 s segments, LANES_V2 §10).
pub const DEFAULT_TARGET_S: f64 = 8.0;

/// One IDR frame of the recording.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keyframe {
    /// Byte offset of the TS packet that starts the keyframe's PES.
    pub offset: u64,
    /// Presentation time in seconds (33-bit wraps already unwrapped by the indexer).
    pub pts_s: f64,
}

/// Keyframe index of one `.ts` source version.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyframeIndex {
    pub source_id: SourceId,
    /// Size of the indexed object; must equal the catalog's size, or the index is stale.
    pub size_bytes: u64,
    /// Presentation time where the last frame ends (last PTS + one frame duration).
    pub end_pts_s: f64,
    pub keyframes: Vec<Keyframe>,
    /// Where the index came from (`synthetic_fixture` for the test files).
    #[serde(default)]
    pub note: Option<String>,
}

impl KeyframeIndex {
    /// Every rule a playable playlist depends on; a violation refuses the playlist (fail closed).
    pub fn validate(&self, expected: &SourceId, size_bytes: u64) -> Result<(), String> {
        if &self.source_id != expected {
            return Err(format!("index is for {}, not {expected}", self.source_id));
        }
        if self.size_bytes != size_bytes {
            return Err(format!(
                "index covers {} bytes, the source has {size_bytes}: stale index",
                self.size_bytes
            ));
        }
        let Some(last) = self.keyframes.last() else {
            return Err("index has no keyframes".into());
        };
        for (i, k) in self.keyframes.iter().enumerate() {
            if !k.pts_s.is_finite() {
                return Err(format!("keyframe {i}: PTS is not finite"));
            }
            if k.offset % TS_PACKET != 0 {
                return Err(format!(
                    "keyframe {i}: offset {} is not on a {TS_PACKET}-byte packet boundary",
                    k.offset
                ));
            }
            if k.offset >= self.size_bytes {
                return Err(format!("keyframe {i}: offset {} is past the end", k.offset));
            }
            if let Some(p) = i.checked_sub(1).map(|j| self.keyframes[j]) {
                if k.offset <= p.offset || k.pts_s <= p.pts_s {
                    return Err(format!(
                        "keyframe {i}: offsets and PTS must both increase strictly"
                    ));
                }
            }
        }
        if !(self.end_pts_s.is_finite() && self.end_pts_s > last.pts_s) {
            return Err("end_pts_s must be after the last keyframe".into());
        }
        Ok(())
    }
}

/// One playlist segment: `[offset, offset + length)` of the object, `duration_s` long.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub offset: u64,
    pub length: u64,
    pub duration_s: f64,
}

/// Keyframe-aligned segments of about `target_s` seconds covering the whole object. The first
/// segment starts at byte 0 so the PAT/PMT before the first IDR are delivered.
/// The index must have passed [`KeyframeIndex::validate`].
pub fn segments(index: &KeyframeIndex, target_s: f64) -> Vec<Segment> {
    let kf = &index.keyframes;
    // Start keyframes: greedy, a new segment at the first keyframe at least `target_s` after the
    // current start (a GOP longer than the target becomes its own, longer segment).
    let mut starts = vec![0usize];
    for (i, k) in kf.iter().enumerate().skip(1) {
        if k.pts_s - kf[*starts.last().unwrap()].pts_s >= target_s {
            starts.push(i);
        }
    }
    starts
        .iter()
        .enumerate()
        .map(|(n, &i)| {
            let begin = if n == 0 { 0 } else { kf[i].offset };
            let (end, end_pts) = match starts.get(n + 1) {
                Some(&j) => (kf[j].offset, kf[j].pts_s),
                None => (index.size_bytes, index.end_pts_s),
            };
            Segment {
                offset: begin,
                length: end - begin,
                duration_s: end_pts - kf[i].pts_s,
            }
        })
        .collect()
}

/// The VOD media playlist (RFC 8216; `EXT-X-BYTERANGE` needs version 4). `url` is the signed URL
/// of the object; it must be a single line.
pub fn playlist(index: &KeyframeIndex, url: &str, target_s: f64) -> Result<String, String> {
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("segment URL must be one line without whitespace".into());
    }
    let segs = segments(index, target_s);
    let longest = segs.iter().map(|s| s.duration_s).fold(0.0, f64::max);
    let mut m = String::with_capacity(segs.len() * (url.len() + 64) + 200);
    m.push_str("#EXTM3U\n#EXT-X-VERSION:4\n#EXT-X-PLAYLIST-TYPE:VOD\n");
    // Every EXTINF rounded to the nearest integer must not exceed the target duration.
    let _ = writeln!(
        m,
        "#EXT-X-TARGETDURATION:{}",
        longest.ceil().max(1.0) as u64
    );
    m.push_str("#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-INDEPENDENT-SEGMENTS\n");
    for s in &segs {
        let _ = writeln!(
            m,
            "#EXTINF:{:.6},\n#EXT-X-BYTERANGE:{}@{}\n{url}",
            s.duration_s, s.length, s.offset
        );
    }
    m.push_str("#EXT-X-ENDLIST\n");
    Ok(m)
}

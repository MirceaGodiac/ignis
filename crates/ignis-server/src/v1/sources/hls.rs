//! Byte-range HLS playlists over one MPEG-TS object at NASA (LANES_V2 §10, RFC 8216).
//!
//! CFI-G `.ts` recordings (15–996 MB) play from S3 with mpegts.js but cannot seek (no duration, no
//! index). hls.js on a byte-range playlist seeks, provided every segment starts on a keyframe
//! (measured 2026-11-14: even-byte segments fail on seek with a buffer-append error). The keyframe
//! index (byte offset ↔ PTS of each IDR) is task R11 (Raul); until it lands, [`KeyframeIndex`] is
//! the server-internal shape this builder reads, and the only index files are test fixtures. When
//! R11 ships its contract in `ignis-core`, this module reads that type instead.
//!
//! RFC 8216 rules the playlist follows:
//! * §4.3.2.2 `EXT-X-BYTERANGE` always carries its explicit `@offset`, so no segment depends on the
//!   previous entry.
//! * §3.2 a TS segment must contain a PAT and PMT or have an `EXT-X-MAP`. A range cut at a
//!   mid-file keyframe usually has neither, so the playlist declares the file's leading PAT+PMT
//!   packets (`init_bytes` of the index, whole 188-byte packets) as the media initialization
//!   section, `#EXT-X-MAP:URI=…,BYTERANGE="<init_bytes>@0"`. `EXT-X-MAP` in a playlist that is not
//!   I-frames-only needs `#EXT-X-VERSION:6` (§4.3.1.2), so the playlist is version 6. The
//!   alternative, cutting only where a PAT precedes the IDR, depends on the muxer repeating its
//!   PAT and is not used.
//! * §3 media segments start at keyframe-index offsets (an IDR), `#EXTINF` is the PTS difference
//!   between the segment's first keyframe and the next segment's, and `#EXT-X-TARGETDURATION` is
//!   the ceiling of the longest `#EXTINF`.
//! * Segments are contiguous: from the end of the init section to the last whole packet, with
//!   no gap or overlap. Every range is a whole number of 188-byte packets and starts on a packet
//!   (sync byte `0x47`). A partial trailing packet (a truncated recording) is left out.
//!
//! No `EXT-X-I-FRAMES-ONLY` playlist: it needs each IDR's byte length, which the index does not
//! carry (a range up to the next keyframe would be the whole GOP, so scrubbing would not get any
//! cheaper).
//!
//! The playlist is a delivery detail: every range is the same signed URL, so the bytes stream from
//! S3 to the browser and none pass through IGNIS.

use std::fmt::Write as _;

use ignis_core::contracts::SourceId;
use serde::Deserialize;

/// MPEG-TS packet size: a range boundary that is not a packet boundary cannot be demuxed.
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
    /// Length of the leading PAT + PMT packets (whole packets): the initialization section.
    pub init_bytes: u64,
    /// Presentation time where the last frame ends (last PTS + one frame duration).
    pub end_pts_s: f64,
    pub keyframes: Vec<Keyframe>,
    /// Where the index came from (`synthetic_fixture` for the test files).
    #[serde(default)]
    pub note: Option<String>,
}

impl KeyframeIndex {
    /// End of the last whole packet.
    pub fn media_end(&self) -> u64 {
        self.size_bytes - self.size_bytes % TS_PACKET
    }

    /// PTS span the media segments cover.
    pub fn duration_s(&self) -> f64 {
        self.keyframes
            .first()
            .map_or(0.0, |k| self.end_pts_s - k.pts_s)
    }

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
        let (Some(first), Some(last)) = (self.keyframes.first(), self.keyframes.last()) else {
            return Err("index has no keyframes".into());
        };
        if self.init_bytes == 0 || self.init_bytes % TS_PACKET != 0 {
            return Err(format!(
                "init_bytes {} is not a whole number of {TS_PACKET}-byte packets",
                self.init_bytes
            ));
        }
        if first.offset < self.init_bytes {
            return Err("the first keyframe lies inside the PAT/PMT init section".into());
        }
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
            if k.offset >= self.media_end() {
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

/// One media segment: `[offset, offset + length)` of the object, `duration_s` long.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub offset: u64,
    pub length: u64,
    pub duration_s: f64,
}

/// Keyframe-aligned media segments of about `target_s` seconds, contiguous from the end of the
/// init section to the last whole packet. Every segment after the first starts at a keyframe;
/// the first starts right after the init section (at the first keyframe when nothing lies
/// between). The index must have passed [`KeyframeIndex::validate`].
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
            let begin = if n == 0 {
                index.init_bytes
            } else {
                kf[i].offset
            };
            let (end, end_pts) = match starts.get(n + 1) {
                Some(&j) => (kf[j].offset, kf[j].pts_s),
                None => (index.media_end(), index.end_pts_s),
            };
            Segment {
                offset: begin,
                length: end - begin,
                duration_s: end_pts - kf[i].pts_s,
            }
        })
        .collect()
}

/// The VOD media playlist. `url` is the signed URL of the object; it must be one line without
/// whitespace or quotes (it is also the quoted `EXT-X-MAP` URI).
pub fn playlist(index: &KeyframeIndex, url: &str, target_s: f64) -> Result<String, String> {
    if url
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '"')
    {
        return Err("segment URL must be one line without whitespace or quotes".into());
    }
    let segs = segments(index, target_s);
    let longest = segs.iter().map(|s| s.duration_s).fold(0.0, f64::max);
    let mut m = String::with_capacity(segs.len() * (url.len() + 64) + url.len() + 300);
    m.push_str("#EXTM3U\n#EXT-X-VERSION:6\n#EXT-X-PLAYLIST-TYPE:VOD\n");
    let _ = writeln!(
        m,
        "#EXT-X-TARGETDURATION:{}",
        longest.ceil().max(1.0) as u64
    );
    m.push_str("#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-INDEPENDENT-SEGMENTS\n");
    let _ = writeln!(
        m,
        "#EXT-X-MAP:URI=\"{url}\",BYTERANGE=\"{}@0\"",
        index.init_bytes
    );
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

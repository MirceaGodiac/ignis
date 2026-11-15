//! Byte-range HLS playlists over one MPEG-TS object at NASA (LANES_V2 §10, RFC 8216).
//!
//! CFI-G `.ts` recordings (15–996 MB) play from S3 with mpegts.js but cannot seek (no duration, no
//! index). hls.js on a byte-range playlist seeks, provided every segment starts on a keyframe
//! (measured 2026-11-14: even-byte segments fail on seek with a buffer-append error). The keyframe
//! index is R11's [`MediaKeyframeIndex`] (`ignis sources media keyframes`): for each keyframe the
//! byte offset of the TS packet that starts its PES, its length (the access unit) and its PTS.
//!
//! RFC 8216 rules the playlists follow:
//! * §4.3.2.2 `EXT-X-BYTERANGE` always carries its explicit `@offset`, so no segment depends on the
//!   previous entry.
//! * §3.2 a TS segment must contain a PAT and PMT or have an `EXT-X-MAP`. A range cut at a
//!   mid-file keyframe usually has neither, so the playlists declare the file's leading packets up
//!   to the first PMT (`init_bytes`, whole 188-byte packets) as the media initialization section,
//!   `#EXT-X-MAP:URI=…,BYTERANGE="<init_bytes>@0"`. `EXT-X-MAP` needs `#EXT-X-VERSION:6` in a
//!   media playlist and `5` in an I-frames-only one (§4.3.1.2).
//! * §3 media segments start at keyframe offsets, `#EXTINF` is the PTS difference between the
//!   segment's first keyframe and the next segment's, and `#EXT-X-TARGETDURATION` is the ceiling
//!   of the longest `#EXTINF`.
//! * Media segments are contiguous: from the end of the init section to the last whole packet,
//!   with no gap or overlap. Every range is a whole number of 188-byte packets and starts on a
//!   packet. A partial trailing packet (a truncated recording) is left out.
//! * §4.3.3.6 the I-frames-only playlist lists one range per indexed keyframe: exactly its access
//!   unit (`offset`, `length`), never the init section and never the rest of the GOP, with
//!   `#EXTINF` the time to the next keyframe (§4.3.2.1 for I-frame playlists). The master playlist
//!   (§4.3.4.3) references it with `EXT-X-I-FRAME-STREAM-INF`.
//!
//! The playlists are a delivery detail: every range is the same signed URL, so the bytes stream
//! from S3 to the browser and none pass through IGNIS.

use std::fmt::Write as _;

use ignis_core::contracts::{MediaKeyframeIndex, SourceId};

pub use ignis_core::contracts::TS_PACKET_BYTES as TS_PACKET;

/// Segment length hls.js was measured to seek with (8 s segments, LANES_V2 §10).
pub const DEFAULT_TARGET_S: f64 = 8.0;

/// The contract rules plus the binding to this source version.
pub fn check(
    index: &MediaKeyframeIndex,
    expected: &SourceId,
    size_bytes: u64,
) -> Result<(), String> {
    if &index.source_id != expected {
        return Err(format!("index is for {}, not {expected}", index.source_id));
    }
    if index.size_bytes != size_bytes {
        return Err(format!(
            "index covers {} bytes, the source has {size_bytes}: stale index",
            index.size_bytes
        ));
    }
    index.validate().map_err(|e| e.reason)
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
/// between). The index must have passed [`check`].
pub fn segments(index: &MediaKeyframeIndex, target_s: f64) -> Vec<Segment> {
    let kf = &index.keyframes;
    let pts = |i: usize| kf[i].pts_s.get();
    // Start keyframes: greedy, a new segment at the first keyframe at least `target_s` after the
    // current start (a GOP longer than the target becomes its own, longer segment).
    let mut starts = vec![0usize];
    for i in 1..kf.len() {
        if pts(i) - pts(*starts.last().unwrap()) >= target_s {
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
                Some(&j) => (kf[j].offset, pts(j)),
                None => (index.media_end(), index.end_pts_s.get()),
            };
            Segment {
                offset: begin,
                length: end - begin,
                duration_s: end_pts - pts(i),
            }
        })
        .collect()
}

fn check_url(url: &str) -> Result<(), String> {
    if url
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '"')
    {
        return Err("segment URL must be one line without whitespace or quotes".into());
    }
    Ok(())
}

fn header(m: &mut String, version: u8, longest: f64) {
    let _ = writeln!(m, "#EXTM3U\n#EXT-X-VERSION:{version}");
    m.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
    let _ = writeln!(
        m,
        "#EXT-X-TARGETDURATION:{}",
        longest.ceil().max(1.0) as u64
    );
    m.push_str("#EXT-X-MEDIA-SEQUENCE:0\n#EXT-X-INDEPENDENT-SEGMENTS\n");
}

/// The VOD media playlist. `url` is the signed URL of the object; it must be one line without
/// whitespace or quotes (it is also the quoted `EXT-X-MAP` URI).
pub fn playlist(index: &MediaKeyframeIndex, url: &str, target_s: f64) -> Result<String, String> {
    check_url(url)?;
    let segs = segments(index, target_s);
    let longest = segs.iter().map(|s| s.duration_s).fold(0.0, f64::max);
    let mut m = String::with_capacity(segs.len() * (url.len() + 64) + url.len() + 300);
    header(&mut m, 6, longest);
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

/// The I-frames-only playlist: one range per indexed keyframe, exactly its access unit.
pub fn iframes_playlist(index: &MediaKeyframeIndex, url: &str) -> Result<String, String> {
    check_url(url)?;
    let kf = &index.keyframes;
    let dur = |i: usize| {
        kf.get(i + 1)
            .map_or(index.end_pts_s.get(), |n| n.pts_s.get())
            - kf[i].pts_s.get()
    };
    let longest = (0..kf.len()).map(dur).fold(0.0, f64::max);
    let mut m = String::with_capacity(kf.len() * (url.len() + 64) + url.len() + 300);
    header(&mut m, 5, longest);
    m.push_str("#EXT-X-I-FRAMES-ONLY\n");
    let _ = writeln!(
        m,
        "#EXT-X-MAP:URI=\"{url}\",BYTERANGE=\"{}@0\"",
        index.init_bytes
    );
    for (i, k) in kf.iter().enumerate() {
        let _ = writeln!(
            m,
            "#EXTINF:{:.6},\n#EXT-X-BYTERANGE:{}@{}\n{url}",
            dur(i),
            k.length,
            k.offset
        );
    }
    m.push_str("#EXT-X-ENDLIST\n");
    Ok(m)
}

/// Peak bit rate of the media segments and of the I-frame ranges (bits per second), for the
/// master playlist's `BANDWIDTH` (§4.3.4.2: the peak segment bit rate).
pub fn bandwidths(index: &MediaKeyframeIndex, target_s: f64) -> (u64, u64) {
    let peak = |bytes: u64, s: f64| if s > 0.0 { bytes as f64 * 8.0 / s } else { 0.0 };
    let media = segments(index, target_s)
        .iter()
        .map(|s| peak(s.length, s.duration_s))
        .fold(0.0, f64::max);
    let kf = &index.keyframes;
    let iframes = kf
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let next = kf
                .get(i + 1)
                .map_or(index.end_pts_s.get(), |n| n.pts_s.get());
            peak(k.length, next - k.pts_s.get())
        })
        .fold(0.0, f64::max);
    (media.ceil().max(1.0) as u64, iframes.ceil().max(1.0) as u64)
}

/// The master playlist: the media playlist as the one variant and the I-frame playlist for
/// scrubbing. Both URIs are IGNIS API URLs (no whitespace or quotes).
pub fn master_playlist(
    index: &MediaKeyframeIndex,
    media_uri: &str,
    iframes_uri: &str,
    target_s: f64,
) -> Result<String, String> {
    check_url(media_uri)?;
    check_url(iframes_uri)?;
    let (media, iframes) = bandwidths(index, target_s);
    let mut m = String::from("#EXTM3U\n#EXT-X-VERSION:6\n#EXT-X-INDEPENDENT-SEGMENTS\n");
    let _ = writeln!(m, "#EXT-X-STREAM-INF:BANDWIDTH={media}\n{media_uri}");
    let _ = writeln!(
        m,
        "#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH={iframes},URI=\"{iframes_uri}\""
    );
    Ok(m)
}

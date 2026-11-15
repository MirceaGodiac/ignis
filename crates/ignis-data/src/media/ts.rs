//! Keyframe index of an MPEG-TS recording (R11), read through byte ranges only.
//!
//! What M12's byte-range HLS playlists need (`ignis-server` `v1/sources/hls.rs`): the leading
//! PAT + PMT packets (`init_bytes`, the playlist's `EXT-X-MAP`), and for each keyframe the byte
//! offset of the TS packet that starts its PES, its PTS, and its length (to the next video PES
//! start: the keyframe's access unit plus the packets of other streams interleaved with it), so
//! the I-frames-only playlist can address one keyframe without the rest of its GOP.
//!
//! **Sampled coverage (remote recordings).** A CFI-G `.ts` is 15–996 MB at NASA and is never
//! downloaded (LANES_V2 §10). Segments only need to start on *a* keyframe roughly every few
//! seconds, so the indexer reads the head (PAT/PMT, first keyframe), the tail (last PTS, frame
//! duration), and then, every `sample_every_s` seconds of presentation time, jumps to the byte
//! offset the observed bitrate predicts and scans forward to the next keyframe and the end of its
//! access unit. Each probe reads a few hundred kB, so a recording costs a fraction of its size
//! (the run reports `bytes_read`).
//!
//! **All coverage (local files).** `sample_every_s = None` scans the whole file and lists every
//! keyframe.
//!
//! What counts as a keyframe: H.264 IDR (NAL 5) or a non-IDR access unit whose first slice is an
//! I slice starting at macroblock 0 (open-GOP encoders); HEVC IRAP (NAL 16–23); MPEG-2 video
//! I picture. PTS is 33 bits at 90 kHz; values are unwrapped against the first keyframe's PTS
//! (one wrap = 26.5 h, longer than any recording here) and stored in seconds.

use anyhow::{bail, ensure, Context};
use ignis_core::contracts::{
    Finite, KeyframeCoverage, KeyframeEntry, MediaKeyframeIndex, Origin, SourceId, VideoCodec,
};

use super::range::RangeRead;

/// MPEG-TS packet size.
pub const TS: u64 = 188;
/// First read of a scan (348 packets, ~64 kB); each further read doubles, up to [`WINDOW`].
const FIRST_WINDOW: u64 = TS * 348;
/// Largest read of a scan (2048 packets).
const WINDOW: u64 = TS * 2048;
/// Head read for PAT/PMT (retried 16 times larger when they are not in it).
const HEAD: u64 = TS * 1024;
/// Tail read for the last PTS and the frame duration (grows until two video PTS are seen).
const TAIL: u64 = TS * 4096;
/// Elementary-stream bytes of one access unit examined before giving up on classifying it.
const CLASSIFY_CAP: usize = 256 * 1024;
const WRAP: i128 = 1 << 33;
/// Version of this indexer, recorded in every index it writes.
pub const INDEXER: &str = "ignis-data/media.ts/1";

/// Indexing parameters.
#[derive(Debug, Clone, Copy)]
pub struct TsOptions {
    /// Seconds of presentation time between probes; `None` = every keyframe (reads everything).
    pub sample_every_s: Option<f64>,
    /// Bytes a probe may scan looking for a keyframe before it gives up.
    pub max_scan: u64,
}

impl Default for TsOptions {
    fn default() -> Self {
        TsOptions {
            sample_every_s: Some(4.0),
            max_scan: 64 << 20,
        }
    }
}

struct Packet<'a> {
    pid: u16,
    pusi: bool,
    payload: &'a [u8],
}

fn packet(p: &[u8], at: u64) -> anyhow::Result<Packet<'_>> {
    ensure!(
        p.len() == TS as usize && p[0] == 0x47,
        "lost MPEG-TS sync at byte {at}"
    );
    let pid = (((p[1] & 0x1f) as u16) << 8) | p[2] as u16;
    let pusi = p[1] & 0x40 != 0;
    let afc = (p[3] >> 4) & 3;
    let mut start = 4usize;
    if afc & 2 != 0 {
        start += 1 + p[4] as usize;
    }
    let payload = if afc & 1 != 0 && start < p.len() {
        &p[start..]
    } else {
        &[][..]
    };
    Ok(Packet { pid, pusi, payload })
}

/// The section of a PSI packet (after the pointer field), if it starts one.
fn section<'a>(pkt: &Packet<'a>) -> Option<&'a [u8]> {
    if !pkt.pusi || pkt.payload.is_empty() {
        return None;
    }
    let ptr = pkt.payload[0] as usize;
    pkt.payload.get(1 + ptr..)
}

fn section_len(sec: &[u8]) -> Option<usize> {
    (sec.len() >= 3).then(|| (((sec[1] & 0x0f) as usize) << 8) | sec[2] as usize)
}

fn pat_pmt_pid(sec: &[u8]) -> Option<u16> {
    if sec.first() != Some(&0) {
        return None;
    }
    let end = (3 + section_len(sec)?).checked_sub(4)?.min(sec.len());
    let mut i = 8;
    while i + 4 <= end {
        let program = u16::from_be_bytes([sec[i], sec[i + 1]]);
        let pid = (((sec[i + 2] & 0x1f) as u16) << 8) | sec[i + 3] as u16;
        if program != 0 {
            return Some(pid);
        }
        i += 4;
    }
    None
}

fn pmt_video(sec: &[u8]) -> Option<(u16, VideoCodec)> {
    if sec.first() != Some(&2) || sec.len() < 12 {
        return None;
    }
    let end = (3 + section_len(sec)?).checked_sub(4)?.min(sec.len());
    let pil = (((sec[10] & 0x0f) as usize) << 8) | sec[11] as usize;
    let mut i = 12 + pil;
    while i + 5 <= end {
        let st = sec[i];
        let pid = (((sec[i + 1] & 0x1f) as u16) << 8) | sec[i + 2] as u16;
        let esl = (((sec[i + 3] & 0x0f) as usize) << 8) | sec[i + 4] as usize;
        let codec = match st {
            0x1b => Some(VideoCodec::H264),
            0x24 => Some(VideoCodec::Hevc),
            0x01 | 0x02 => Some(VideoCodec::Mpeg2),
            _ => None,
        };
        if let Some(c) = codec {
            return Some((pid, c));
        }
        i += 5 + esl;
    }
    None
}

/// PES header of a payload that starts a PES: (PTS in 90 kHz ticks, elementary-stream bytes).
fn pes(payload: &[u8]) -> Option<(Option<u64>, &[u8])> {
    if payload.len() < 9 || payload[..3] != [0, 0, 1] {
        return None;
    }
    let flags = payload[7];
    let hdr = payload[8] as usize;
    let es = payload.get(9 + hdr..).unwrap_or(&[]);
    let pts = (flags & 0x80 != 0 && payload.len() >= 14).then(|| {
        let p = &payload[9..14];
        (((p[0] >> 1) & 7) as u64) << 30
            | (p[1] as u64) << 22
            | ((p[2] >> 1) as u64) << 15
            | (p[3] as u64) << 7
            | (p[4] >> 1) as u64
    });
    Some((pts, es))
}

/// Exp-Golomb `ue(v)` reader over a few bytes (emulation-prevention bytes ignored: the fields
/// read here sit in the first bytes of a slice header).
struct Bits<'a> {
    b: &'a [u8],
    pos: usize,
}

impl Bits<'_> {
    fn bit(&mut self) -> Option<u32> {
        let byte = *self.b.get(self.pos / 8)?;
        let v = (byte >> (7 - self.pos % 8)) & 1;
        self.pos += 1;
        Some(v as u32)
    }
    fn ue(&mut self) -> Option<u32> {
        let mut zeros = 0;
        while self.bit()? == 0 {
            zeros += 1;
            if zeros > 31 {
                return None;
            }
        }
        let mut v = 0u32;
        for _ in 0..zeros {
            v = (v << 1) | self.bit()?;
        }
        Some((1u32 << zeros) - 1 + v)
    }
}

/// Is the access unit starting with `es` a keyframe? `None` = not decidable yet (no picture data
/// seen); `from` is where the start-code scan resumes.
fn classify(codec: VideoCodec, es: &[u8], from: &mut usize) -> Option<bool> {
    let mut i = from.saturating_sub(3);
    while i + 3 < es.len() {
        if es[i] != 0 || es[i + 1] != 0 || es[i + 2] != 1 {
            i += 1;
            continue;
        }
        let h = es[i + 3];
        match codec {
            VideoCodec::H264 => match h & 0x1f {
                5 => return Some(true),
                1 => {
                    // Need the first bytes of the slice header.
                    let rest = es.get(i + 4..i + 12)?;
                    let mut b = Bits { b: rest, pos: 0 };
                    let first_mb = b.ue()?;
                    let slice_type = b.ue()?;
                    return Some(first_mb == 0 && matches!(slice_type % 5, 2 | 4));
                }
                2..=4 => return Some(false),
                _ => {}
            },
            VideoCodec::Hevc => match (h >> 1) & 0x3f {
                16..=23 => return Some(true),
                0..=15 | 24..=31 => return Some(false),
                _ => {}
            },
            VideoCodec::Mpeg2 => {
                if h == 0x00 {
                    let b = *es.get(i + 5)?;
                    return Some((b >> 3) & 7 == 1);
                }
            }
        }
        i += 3;
    }
    *from = es.len();
    None
}

/// One keyframe found by a scan, PTS still raw (33-bit).
#[derive(Debug, Clone, Copy)]
struct Found {
    offset: u64,
    length: u64,
    pts_raw: u64,
}

struct Acc {
    offset: u64,
    pts: Option<u64>,
    es: Vec<u8>,
    from: usize,
    decided: Option<bool>,
}

struct Stream {
    init_bytes: u64,
    video_pid: u16,
    codec: VideoCodec,
    media_end: u64,
}

fn round_up(v: u64) -> u64 {
    v.div_ceil(TS) * TS
}

/// From packet-aligned `start`, the first keyframe whose PES starts at or after `start`, and its
/// length. `None` at the end of the media or after `max_scan` bytes without one.
fn scan(r: &dyn RangeRead, s: &Stream, start: u64, max_scan: u64) -> anyhow::Result<Option<Found>> {
    let start = round_up(start).max(s.init_bytes);
    let mut pos = start;
    let mut cur: Option<Acc> = None;
    let mut found: Option<(u64, u64)> = None;
    let mut win = FIRST_WINDOW;
    while pos < s.media_end && (pos - start < max_scan || found.is_some()) {
        let chunk = r.read_range(pos, win.min(s.media_end - pos))?;
        win = (win * 2).min(WINDOW);
        for (k, p) in chunk.chunks_exact(TS as usize).enumerate() {
            let at = pos + k as u64 * TS;
            let pkt = packet(p, at)?;
            if pkt.pid != s.video_pid {
                continue;
            }
            if pkt.pusi {
                if let Some((offset, pts_raw)) = found {
                    return Ok(Some(Found {
                        offset,
                        length: at - offset,
                        pts_raw,
                    }));
                }
                cur = pes(pkt.payload).map(|(pts, es)| Acc {
                    offset: at,
                    pts,
                    es: es.to_vec(),
                    from: 0,
                    decided: None,
                });
            } else if let Some(a) = cur.as_mut().filter(|a| a.decided.is_none()) {
                a.es.extend_from_slice(pkt.payload);
            }
            if let Some(a) = cur.as_mut().filter(|a| a.decided.is_none()) {
                a.decided = classify(s.codec, &a.es, &mut a.from)
                    .or((a.es.len() > CLASSIFY_CAP).then_some(false));
                if a.decided == Some(true) {
                    if let Some(p) = a.pts {
                        found = Some((a.offset, p));
                    }
                }
                if a.decided.is_some() {
                    a.es = Vec::new();
                }
            }
        }
        pos += chunk.len() as u64;
    }
    Ok(found.map(|(offset, pts_raw)| Found {
        offset,
        length: s.media_end - offset,
        pts_raw,
    }))
}

/// `raw` (33-bit) as the unwrapped tick count closest to `expect`.
fn unwrap(raw: u64, expect: i128) -> i128 {
    let v = raw as i128;
    v + ((expect - v) as f64 / WRAP as f64).round() as i128 * WRAP
}

fn head(r: &dyn RangeRead) -> anyhow::Result<Stream> {
    let size = r.size();
    let media_end = size - size % TS;
    ensure!(
        media_end >= 3 * TS,
        "{size} B is too small for an MPEG-TS recording"
    );
    let mut bytes = r.read_range(0, HEAD.min(media_end))?;
    for k in 0..3 {
        ensure!(
            bytes[k * TS as usize] == 0x47,
            "not a 188-byte MPEG-TS stream (no sync byte at {})",
            k as u64 * TS
        );
    }
    for attempt in 0..2 {
        if attempt == 1 {
            if bytes.len() as u64 >= media_end {
                break;
            }
            bytes = r.read_range(0, (HEAD * 16).min(media_end))?;
        }
        let mut pmt_pid = None;
        for (k, p) in bytes.chunks_exact(TS as usize).enumerate() {
            let at = k as u64 * TS;
            let pkt = packet(p, at)?;
            if pkt.pid == 0 && pmt_pid.is_none() {
                pmt_pid = section(&pkt).and_then(pat_pmt_pid);
            } else if Some(pkt.pid) == pmt_pid {
                if let Some((video_pid, codec)) = section(&pkt).and_then(pmt_video) {
                    return Ok(Stream {
                        init_bytes: at + TS,
                        video_pid,
                        codec,
                        media_end,
                    });
                }
            }
        }
    }
    bail!(
        "no PAT/PMT with a video stream in the first {} B",
        bytes.len()
    )
}

/// Last video PTS (raw) and the frame duration in ticks, from the tail.
fn tail(r: &dyn RangeRead, s: &Stream) -> anyhow::Result<(u64, u64)> {
    let mut len = TAIL;
    let pts = loop {
        let start = round_up(s.media_end.saturating_sub(len).max(s.init_bytes));
        let bytes = r.read_range(start, s.media_end - start)?;
        let mut pts = Vec::new();
        for (k, p) in bytes.chunks_exact(TS as usize).enumerate() {
            let pkt = packet(p, start + k as u64 * TS)?;
            if pkt.pid == s.video_pid && pkt.pusi {
                if let Some((Some(t), _)) = pes(pkt.payload) {
                    pts.push(t);
                }
            }
        }
        if pts.len() >= 2 || start == s.init_bytes || len >= 64 << 20 {
            break pts;
        }
        len *= 4;
    };
    let last = *pts
        .last()
        .context("no video PTS in the tail of the recording")?;
    let mut ticks: Vec<i128> = pts.iter().map(|&t| unwrap(t, last as i128)).collect();
    ticks.sort_unstable();
    ticks.dedup();
    let frame = ticks
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|d| *d > 0)
        .min()
        .unwrap_or(3003) as u64;
    // 1/240 s .. 1 s; anything else is not a frame interval.
    let frame = if (375..=90_000).contains(&frame) {
        frame
    } else {
        3003
    };
    let max = *ticks.last().unwrap();
    Ok(((max.rem_euclid(WRAP)) as u64, frame))
}

/// Builds the keyframe index of one recording.
pub fn index(
    r: &dyn RangeRead,
    source_id: SourceId,
    opts: TsOptions,
    origin: Origin,
) -> anyhow::Result<MediaKeyframeIndex> {
    let s = head(r)?;
    let first = scan(r, &s, s.init_bytes, opts.max_scan)?
        .with_context(|| {
            format!(
                "no keyframe after the first PAT/PMT (init section ends at byte {}; scanned up to {} B)",
                s.init_bytes, opts.max_scan
            )
        })?;
    let base = first.pts_raw as i128;
    let (last_raw, frame) = tail(r, &s)?;
    // The end lies after the first keyframe: the smallest unwrapped value not below it.
    let mut end_ticks = last_raw as i128;
    while end_ticks < base {
        end_ticks += WRAP;
    }
    let end_ticks = end_ticks + frame as i128;
    let duration = (end_ticks - base) as f64 / 90_000.0;
    ensure!(duration > 0.0, "recording has no positive duration");

    let mut kfs: Vec<(u64, u64, i128)> = vec![(first.offset, first.length, base)];
    loop {
        let &(off, len, t) = kfs.last().unwrap();
        let next_from = match opts.sample_every_s {
            None => off + len,
            Some(every) => {
                let t_next = t as f64 / 90_000.0 + every;
                if t_next >= end_ticks as f64 / 90_000.0 - every * 0.5 {
                    break;
                }
                // Bitrate seen so far (whole recording before the second keyframe).
                let rate = if kfs.len() > 1 && t > base {
                    (off - first.offset) as f64 / ((t - base) as f64 / 90_000.0)
                } else {
                    (s.media_end - first.offset) as f64 / duration
                };
                off + len.max((every * rate) as u64)
            }
        };
        let expect = t + (opts.sample_every_s.unwrap_or(0.0) * 90_000.0) as i128;
        let mut from = next_from;
        let next = loop {
            let Some(f) = scan(r, &s, from, opts.max_scan)? else {
                break None;
            };
            let ticks = unwrap(f.pts_raw, expect);
            if ticks > t && f.offset > off {
                break Some((f.offset, f.length, ticks));
            }
            // PTS not after the previous keyframe (a muxer glitch): not a usable boundary.
            from = f.offset + f.length;
        };
        match next {
            Some(k) => kfs.push(k),
            None => break,
        }
    }
    // A keyframe's length never runs into the next listed keyframe.
    let keyframes = kfs
        .iter()
        .map(|&(offset, length, t)| {
            Ok(KeyframeEntry {
                offset,
                length,
                pts_s: Finite::new(t as f64 / 90_000.0)?,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let ix = MediaKeyframeIndex {
        schema_version: Default::default(),
        source_id,
        size_bytes: r.size(),
        init_bytes: s.init_bytes,
        video_pid: s.video_pid,
        codec: s.codec,
        end_pts_s: Finite::new(end_ticks as f64 / 90_000.0)?,
        coverage: if opts.sample_every_s.is_some() {
            KeyframeCoverage::Sampled
        } else {
            KeyframeCoverage::All
        },
        keyframes,
        bytes_read: r.meter().bytes(),
        indexer: INDEXER.to_string(),
        origin,
    };
    ix.validate()?;
    Ok(ix)
}

//! Synthetic MPEG-TS streams for tests (`origin = synthetic_fixture`, never NASA data).
//!
//! The stream is structurally what the indexer reads: PAT then PMT (H.264 video on PID 0x100, an
//! audio PID 0x101), then one PES per frame on the video PID with a PTS, an access-unit
//! delimiter and an IDR (keyframes) or a P slice, and audio packets interleaved. The truth
//! (offset, length, PTS of every keyframe) is returned with the bytes.

/// What a synthetic stream contains.
#[derive(Debug, Clone)]
pub struct SynthSpec {
    pub fps: u32,
    pub gop_frames: u32,
    pub gops: u32,
    /// Packets of a keyframe / of another frame.
    pub key_packets: usize,
    pub frame_packets: usize,
    /// An audio packet after every `audio_every` video packets (0 = none).
    pub audio_every: usize,
    /// PTS of the first frame (90 kHz, 33-bit; near 2^33 to exercise the wrap).
    pub first_pts: u64,
    /// Keyframes after the first are non-IDR I slices (open-GOP encoders) instead of IDRs.
    pub i_slices: bool,
    /// Bytes of a partial packet appended at the end (a truncated recording).
    pub trailing: usize,
}

impl Default for SynthSpec {
    fn default() -> Self {
        SynthSpec {
            fps: 25,
            gop_frames: 25,
            gops: 60,
            key_packets: 40,
            frame_packets: 4,
            audio_every: 5,
            first_pts: (1u64 << 33) - 90_000 * 10,
            i_slices: false,
            trailing: 0,
        }
    }
}

/// One keyframe as written.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SynthKeyframe {
    pub offset: u64,
    pub length: u64,
    /// Unwrapped PTS in ticks.
    pub pts_ticks: u64,
}

pub struct Synth {
    pub bytes: Vec<u8>,
    pub init_bytes: u64,
    pub keyframes: Vec<SynthKeyframe>,
    /// Unwrapped PTS (ticks) where the last frame ends.
    pub end_ticks: u64,
}

const VIDEO: u16 = 0x100;
const AUDIO: u16 = 0x101;
const PMT: u16 = 0x1000;

fn crc32_mpeg(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in data {
        crc ^= (b as u32) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ 0x04c1_1db7
            } else {
                crc << 1
            };
        }
    }
    crc
}

fn psi_packet(pid: u16, section: &[u8]) -> Vec<u8> {
    let mut p = vec![0x47, 0x40 | (pid >> 8) as u8, pid as u8, 0x10, 0x00];
    p.extend_from_slice(section);
    p.extend_from_slice(&crc32_mpeg(section).to_be_bytes());
    p.resize(188, 0xff);
    p
}

fn pts_bytes(pts: u64) -> [u8; 5] {
    [
        0x21 | (((pts >> 30) & 7) as u8) << 1,
        (pts >> 22) as u8,
        0x01 | (((pts >> 15) & 0x7f) as u8) << 1,
        (pts >> 7) as u8,
        0x01 | ((pts & 0x7f) as u8) << 1,
    ]
}

fn packet(pid: u16, pusi: bool, payload: &[u8]) -> Vec<u8> {
    let mut p = vec![
        0x47,
        if pusi { 0x40 } else { 0 } | (pid >> 8) as u8,
        pid as u8,
        0x10,
    ];
    p.extend_from_slice(&payload[..payload.len().min(184)]);
    p.resize(188, 0xff);
    p
}

/// Builds a stream to `spec`.
pub fn stream(spec: &SynthSpec) -> Synth {
    // PAT: program 1 -> PMT PID.
    let pat = [
        0x00,
        0xb0,
        13,
        0x00,
        0x01,
        0xc1,
        0x00,
        0x00,
        0x00,
        0x01,
        0xe0 | (PMT >> 8) as u8,
        PMT as u8,
    ];
    // PMT: PCR on video; H.264 video, ADTS audio.
    let pmt = [
        0x02,
        0xb0,
        23,
        0x00,
        0x01,
        0xc1,
        0x00,
        0x00,
        0xe0 | (VIDEO >> 8) as u8,
        VIDEO as u8,
        0xf0,
        0x00,
        0x1b,
        0xe0 | (VIDEO >> 8) as u8,
        VIDEO as u8,
        0xf0,
        0x00,
        0x0f,
        0xe0 | (AUDIO >> 8) as u8,
        AUDIO as u8,
        0xf0,
        0x00,
    ];
    let mut bytes = psi_packet(0, &pat);
    bytes.extend(psi_packet(PMT, &pmt));
    let init_bytes = bytes.len() as u64;
    let tick = 90_000 / spec.fps as u64;
    let mut keyframes = Vec::new();
    let mut video_packets = 0usize;
    let mut pending_key: Option<(u64, u64)> = None;
    let frames = spec.gop_frames * spec.gops;
    for f in 0..frames {
        let key = f % spec.gop_frames == 0;
        let unwrapped = spec.first_pts + f as u64 * tick;
        let pts = unwrapped % (1 << 33);
        let start = bytes.len() as u64;
        if let Some((o, t)) = pending_key.take() {
            keyframes.push(SynthKeyframe {
                offset: o,
                length: start - o,
                pts_ticks: t,
            });
        }
        if key {
            pending_key = Some((start, unwrapped));
        }
        let mut pes = vec![0, 0, 1, 0xe0, 0, 0, 0x80, 0x80, 5];
        pes.extend_from_slice(&pts_bytes(pts));
        pes.extend_from_slice(&[0, 0, 0, 1, 0x09, 0xf0]); // AUD
        if key {
            pes.extend_from_slice(&[0, 0, 0, 1, 0x67, 0x42, 0x00, 0x1e]); // SPS
            pes.extend_from_slice(&[0, 0, 0, 1, 0x68, 0xce, 0x38, 0x80]); // PPS
            if spec.i_slices && f > 0 {
                // non-IDR slice, first_mb 0 ('1'), slice_type 7 = I ('0001000')
                pes.extend_from_slice(&[0, 0, 1, 0x41, 0x88, 0x80]);
            } else {
                pes.extend_from_slice(&[0, 0, 1, 0x65, 0x88, 0x80]); // IDR
            }
        } else {
            // P slice: first_mb 0 ('1'), slice_type 0 ('1')
            pes.extend_from_slice(&[0, 0, 1, 0x41, 0xc0]);
        }
        let n = if key {
            spec.key_packets
        } else {
            spec.frame_packets
        };
        for k in 0..n {
            let payload = if k == 0 { pes.clone() } else { vec![0xaa; 184] };
            bytes.extend(packet(VIDEO, k == 0, &payload));
            video_packets += 1;
            if spec.audio_every > 0 && video_packets % spec.audio_every == 0 {
                bytes.extend(packet(AUDIO, true, &[0, 0, 1, 0xc0, 0, 0, 0x80, 0x00, 0]));
            }
        }
    }
    let end = bytes.len() as u64;
    if let Some((o, t)) = pending_key {
        keyframes.push(SynthKeyframe {
            offset: o,
            length: end - o,
            pts_ticks: t,
        });
    }
    bytes.extend(std::iter::repeat_n(0x47u8, spec.trailing));
    Synth {
        bytes,
        init_bytes,
        keyframes,
        end_ticks: spec.first_pts + frames as u64 * tick,
    }
}

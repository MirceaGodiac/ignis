//! Video probing and decoding with real presentation timestamps (V3.1 §4.5).
//!
//! Demux: the `mp4` crate (pure Rust) reads the ISO-BMFF sample table. A sample's presentation time
//! is `dts + ctts − edit_shift` in ticks of the media timescale (`mdhd`): decode times come from
//! `stts` (variable frame rate is just unequal `stts` deltas), composition offsets from `ctts`
//! (B-frame reordering) and the edit list (`elst`) shift is applied the way FFmpeg applies it, so
//! the PTS axis is the one the container declares. Time is never `index / fps`.
//!
//! Decode: OpenH264 (Cisco), compiled from its bundled source by the `openh264` crate. No system
//! FFmpeg, no DLL. The decoder emits pictures in presentation order (it holds a reorder buffer
//! sized from the SPS); each emitted picture is paired with the next PTS of the sorted sample PTS
//! set, and the count of emitted pictures is checked against the sample count.
//!
//! Colour: YUV 4:2:0 → RGB with BT.601 limited-range coefficients (the conversion FFmpeg/PyAV
//! apply to an untagged stream), chroma replicated per 2×2 block.
//!
//! Frames are indexed in presentation order; that index is `SourceLocator.frame` and
//! `ContourObservation.frame`.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context};
use serde::{Deserialize, Serialize};

/// Container metadata and every sample PTS (demux only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoProbe {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub container: String,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    /// Exact rational, e.g. "1/30000".
    pub time_base: String,
    /// Counted from the sample table, not the header.
    pub n_frames: usize,
    pub header_frames: Option<usize>,
    /// Average rate implied by the media duration (informative only).
    pub nominal_rate: Option<String>,
    pub first_pts_s: Option<f64>,
    pub last_pts_s: Option<f64>,
    pub duration_s: Option<f64>,
    /// Distinct presentation-order PTS steps (seconds, 9 significant digits) → count.
    pub pts_step_s: BTreeMap<String, usize>,
    pub variable_frame_rate: bool,
    /// PTS decreases in decode order (B-frame reordering; informative).
    pub decode_order_pts_reorders: usize,
    pub missing_pts: usize,
    /// Edit-list shift subtracted from every composition time (ticks).
    pub edit_shift_ticks: i64,
    pub metadata: BTreeMap<String, String>,
}

impl VideoProbe {
    pub fn timescale(&self) -> u64 {
        self.time_base
            .split_once('/')
            .and_then(|(_, d)| d.parse().ok())
            .unwrap_or(1)
    }
}

/// The demuxed track: decode-order samples with their PTS.
pub struct Demuxed {
    reader: mp4::Mp4Reader<BufReader<File>>,
    pub track_id: u32,
    pub timescale: u32,
    pub width: u32,
    pub height: u32,
    pub length_size: usize,
    pub sps: Vec<Vec<u8>>,
    pub pps: Vec<Vec<u8>>,
    /// PTS (ticks) per sample in decode order.
    pub pts: Vec<i64>,
    pub edit_shift: i64,
    pub sample_count: u32,
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// `pts_s` exactly as `float(Fraction(pts, timescale))` (correctly rounded quotient).
pub fn ticks_to_s(pts: i64, timescale: u64) -> f64 {
    pts as f64 / timescale as f64
}

pub fn demux(path: &Path) -> anyhow::Result<Demuxed> {
    let f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let size = f.metadata()?.len();
    let reader = mp4::Mp4Reader::read_header(BufReader::new(f), size)
        .map_err(|e| anyhow!("mp4 demux: {e}"))?;
    let movie_ts = reader.moov.mvhd.timescale.max(1) as i64;
    let (track_id, track) = reader
        .tracks()
        .iter()
        .find(|(_, t)| matches!(t.media_type(), Ok(mp4::MediaType::H264)))
        .map(|(id, t)| (*id, t))
        .ok_or_else(|| anyhow!("no H.264 video track (only MP4/H.264 is supported)"))?;
    let timescale = track.timescale();
    let avc = track
        .trak
        .mdia
        .minf
        .stbl
        .stsd
        .avc1
        .as_ref()
        .ok_or_else(|| anyhow!("missing avc1 sample entry"))?;
    let sps = avc
        .avcc
        .sequence_parameter_sets
        .iter()
        .map(|n| n.bytes.clone())
        .collect();
    let pps = avc
        .avcc
        .picture_parameter_sets
        .iter()
        .map(|n| n.bytes.clone())
        .collect();
    let length_size = (avc.avcc.length_size_minus_one & 3) as usize + 1;
    // Edit list, as FFmpeg's mov demuxer: a leading empty edit delays the track, the first
    // non-empty edit's media_time is the composition time that maps to presentation 0.
    let mut edit_shift: i64 = 0;
    if let Some(elst) = track.trak.edts.as_ref().and_then(|e| e.elst.as_ref()) {
        let v1 = elst.version == 1;
        let empty = |media_time: u64| {
            (v1 && media_time == u64::MAX) || (!v1 && media_time == u32::MAX as u64)
        };
        let mut delay: i64 = 0;
        for e in &elst.entries {
            if empty(e.media_time) {
                delay += e.segment_duration as i64 * timescale as i64 / movie_ts;
            } else {
                edit_shift = e.media_time as i64 - delay;
                break;
            }
        }
    }
    let sample_count = track.sample_count();
    let (width, height) = (track.width() as u32, track.height() as u32);
    let mut d = Demuxed {
        reader,
        track_id,
        timescale,
        width,
        height,
        length_size,
        sps,
        pps,
        pts: Vec::new(),
        edit_shift,
        sample_count,
    };
    let mut pts = Vec::with_capacity(sample_count as usize);
    for i in 1..=sample_count {
        let s = d
            .reader
            .read_sample(track_id, i)
            .map_err(|e| anyhow!("sample {i}: {e}"))?
            .ok_or_else(|| anyhow!("sample {i} missing"))?;
        pts.push(s.start_time as i64 + s.rendering_offset as i64 - edit_shift);
    }
    d.pts = pts;
    Ok(d)
}

pub fn probe(path: &Path) -> anyhow::Result<VideoProbe> {
    let sha = ignis_core::hashing::file_sha256(path)?;
    probe_with_sha(path, sha)
}

pub fn probe_with_sha(path: &Path, sha256: String) -> anyhow::Result<VideoProbe> {
    let d = demux(path)?;
    let ts = d.timescale as u64;
    let nonmono = d.pts.windows(2).filter(|w| w[1] <= w[0]).count();
    let mut pres = d.pts.clone();
    pres.sort_unstable();
    let mut steps = BTreeMap::new();
    let mut tmin = i64::MAX;
    let mut tmax = i64::MIN;
    for w in pres.windows(2) {
        let dt = w[1] - w[0];
        tmin = tmin.min(dt);
        tmax = tmax.max(dt);
        *steps.entry(fmt_g9(ticks_to_s(dt, ts))).or_insert(0) += 1;
    }
    // VFR when more than one distinct step exists beyond one tick of rounding jitter.
    let vfr = pres.len() > 1 && tmax - tmin > 1;
    let first = pres.first().map(|&p| ticks_to_s(p, ts));
    let last = pres.last().map(|&p| ticks_to_s(p, ts));
    let media_duration = d
        .reader
        .tracks()
        .get(&d.track_id)
        .map(|t| t.trak.mdia.mdhd.duration)
        .unwrap_or(0);
    let nominal_rate = (media_duration > 0 && !pres.is_empty()).then(|| {
        let (n, m) = (pres.len() as u64 * ts, media_duration);
        let g = gcd(n, m);
        format!("{}/{}", n / g, m / g)
    });
    let mut metadata = BTreeMap::new();
    metadata.insert("major_brand".into(), d.reader.ftyp.major_brand.to_string());
    metadata.insert(
        "minor_version".into(),
        d.reader.ftyp.minor_version.to_string(),
    );
    metadata.insert(
        "compatible_brands".into(),
        d.reader
            .ftyp
            .compatible_brands
            .iter()
            .map(|b| b.to_string())
            .collect::<String>(),
    );
    Ok(VideoProbe {
        path: path.display().to_string(),
        sha256,
        size_bytes: std::fs::metadata(path)?.len(),
        container: "mp4".into(),
        codec: "h264".into(),
        width: d.width,
        height: d.height,
        time_base: format!("1/{ts}"),
        n_frames: d.pts.len(),
        header_frames: Some(d.sample_count as usize),
        nominal_rate,
        first_pts_s: first,
        last_pts_s: last,
        duration_s: first.zip(last).map(|(a, b)| b - a),
        pts_step_s: steps,
        variable_frame_rate: vfr,
        decode_order_pts_reorders: nonmono,
        missing_pts: 0,
        edit_shift_ticks: d.edit_shift,
        metadata,
    })
}

/// Python's `f"{x:.9g}"`.
fn fmt_g9(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let e = x.abs().log10().floor() as i32;
    if !(-5..9).contains(&e) {
        let s = format!("{:.8e}", x);
        let (m, ex) = s.split_once('e').unwrap();
        let m = trim_zeros(m);
        let ex: i32 = ex.parse().unwrap();
        return format!("{m}e{}{:02}", if ex < 0 { '-' } else { '+' }, ex.abs());
    }
    let dec = (8 - e).max(0) as usize;
    trim_zeros(&format!("{:.*}", dec, x)).to_string()
}

fn trim_zeros(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}

/// One decoded picture in presentation order.
pub struct Frame {
    pub index: usize,
    pub pts: i64,
    pub pts_s: f64,
    pub width: usize,
    pub height: usize,
    /// H×W×3 RGB, row-major.
    pub rgb: Vec<u8>,
}

/// Streaming decoder: yields frames in presentation order with container PTS.
pub struct FrameReader {
    d: Demuxed,
    dec: openh264::decoder::Decoder,
    next_sample: u32,
    sorted_pts: Vec<i64>,
    emitted: usize,
    pending: std::collections::VecDeque<(usize, usize, Vec<u8>)>,
    flushed: bool,
    annexb: Vec<u8>,
    want_image: bool,
    pub timescale: u64,
}

impl FrameReader {
    pub fn open(path: &Path, want_image: bool) -> anyhow::Result<Self> {
        let d = demux(path)?;
        let mut sorted_pts = d.pts.clone();
        sorted_pts.sort_unstable();
        if sorted_pts.windows(2).any(|w| w[0] == w[1]) {
            bail!("duplicate presentation timestamps in the sample table");
        }
        let cfg = openh264::decoder::DecoderConfig::new()
            .flush_after_decode(openh264::decoder::Flush::NoFlush);
        let dec =
            openh264::decoder::Decoder::with_api_config(openh264::OpenH264API::from_source(), cfg)
                .map_err(|e| anyhow!("openh264 init: {e}"))?;
        let timescale = d.timescale as u64;
        Ok(FrameReader {
            d,
            dec,
            next_sample: 1,
            sorted_pts,
            emitted: 0,
            pending: Default::default(),
            flushed: false,
            annexb: Vec::new(),
            want_image,
            timescale,
        })
    }

    pub fn n_frames(&self) -> usize {
        self.sorted_pts.len()
    }

    fn to_annexb(&mut self, sample: &[u8], with_params: bool) -> anyhow::Result<()> {
        self.annexb.clear();
        if with_params {
            for p in self.d.sps.iter().chain(self.d.pps.iter()) {
                self.annexb.extend_from_slice(&[0, 0, 0, 1]);
                self.annexb.extend_from_slice(p);
            }
        }
        let n = self.d.length_size;
        let mut i = 0;
        while i + n <= sample.len() {
            let len = sample[i..i + n]
                .iter()
                .fold(0usize, |a, &b| (a << 8) | b as usize);
            i += n;
            if i + len > sample.len() {
                bail!("truncated NAL unit in sample");
            }
            self.annexb.extend_from_slice(&[0, 0, 0, 1]);
            self.annexb.extend_from_slice(&sample[i..i + len]);
            i += len;
        }
        Ok(())
    }

    /// Feed samples until at least one picture is pending (or the stream is exhausted).
    fn fill(&mut self) -> anyhow::Result<()> {
        while self.pending.is_empty() && !self.flushed {
            if self.next_sample > self.d.sample_count {
                let rest: Vec<_> = {
                    let frames = self
                        .dec
                        .flush_remaining()
                        .map_err(|e| anyhow!("openh264 flush: {e}"))?;
                    let want = self.want_image;
                    frames
                        .iter()
                        .map(|y| {
                            use openh264::formats::YUVSource;
                            let (w, h) = y.dimensions();
                            (
                                w,
                                h,
                                if want {
                                    yuv420_to_rgb(y, w, h)
                                } else {
                                    Vec::new()
                                },
                            )
                        })
                        .collect()
                };
                self.pending.extend(rest);
                self.flushed = true;
                break;
            }
            let id = self.next_sample;
            self.next_sample += 1;
            let s = self
                .d
                .reader
                .read_sample(self.d.track_id, id)
                .map_err(|e| anyhow!("sample {id}: {e}"))?
                .ok_or_else(|| anyhow!("sample {id} missing"))?;
            let with_params = id == 1 || s.is_sync;
            self.to_annexb(&s.bytes, with_params)?;
            let buf = std::mem::take(&mut self.annexb);
            let out = match self.dec.decode(&buf) {
                Ok(Some(y)) => {
                    use openh264::formats::YUVSource;
                    let (w, h) = y.dimensions();
                    Some((
                        w,
                        h,
                        if self.want_image {
                            yuv420_to_rgb(&y, w, h)
                        } else {
                            Vec::new()
                        },
                    ))
                }
                Ok(None) => None,
                Err(e) => {
                    self.annexb = buf;
                    bail!("openh264 decode error at sample {id}: {e}");
                }
            };
            self.annexb = buf;
            if let Some(p) = out {
                self.pending.push_back(p);
            }
        }
        Ok(())
    }
}

impl Iterator for FrameReader {
    type Item = anyhow::Result<Frame>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Err(e) = self.fill() {
            return Some(Err(e));
        }
        let (w, h, rgb) = self.pending.pop_front()?;
        let index = self.emitted;
        self.emitted += 1;
        let Some(&pts) = self.sorted_pts.get(index) else {
            return Some(Err(anyhow!(
                "decoder emitted more pictures ({}) than the container has samples",
                index + 1
            )));
        };
        Some(Ok(Frame {
            index,
            pts,
            pts_s: ticks_to_s(pts, self.timescale),
            width: w,
            height: h,
            rgb,
        }))
    }
}

/// BT.601 limited-range YUV 4:2:0 → RGB24, bit-identical to FFmpeg's x86 unscaled `yuv420p→rgb24`
/// path (what PyAV's `to_ndarray("rgb24")` runs): 13-bit coefficients, operands promoted by 3 bits,
/// `pmulhw` (product >> 16, floor) per term, saturating pack. Chroma is replicated per 2×2 block.
pub fn yuv420_to_rgb<S: openh264::formats::YUVSource>(src: &S, w: usize, h: usize) -> Vec<u8> {
    let (sy, su, sv) = src.strides();
    let (yp, up, vp) = (src.y(), src.u(), src.v());
    let mut out = vec![0u8; w * h * 3];
    // round(c * 2^13): 1.164383 (255/219), 1.596027, -0.391762, -0.812968, 2.017232.
    const YC: i32 = 9539;
    const VR: i32 = 13075;
    const UG: i32 = -3209;
    const VG: i32 = -6660;
    const UB: i32 = 16525;
    #[inline(always)]
    fn pm(a: i32, b: i32) -> i32 {
        (a * b) >> 16
    }
    for r in 0..h {
        let yrow = &yp[r * sy..r * sy + w];
        let urow = &up[(r / 2) * su..];
        let vrow = &vp[(r / 2) * sv..];
        let o = &mut out[r * w * 3..(r + 1) * w * 3];
        for c in 0..w {
            let y = pm(((yrow[c] as i32) << 3) - 128, YC);
            let u = ((urow[c / 2] as i32) << 3) - 1024;
            let v = ((vrow[c / 2] as i32) << 3) - 1024;
            o[c * 3] = (y + pm(v, VR)).clamp(0, 255) as u8;
            o[c * 3 + 1] = (y + pm(u, UG) + pm(v, VG)).clamp(0, 255) as u8;
            o[c * 3 + 2] = (y + pm(u, UB)).clamp(0, 255) as u8;
        }
    }
    out
}

/// Iterate every `every`-th frame in `[start, stop)`. Unselected frames are decoded (H.264 needs
/// them as references) but never converted to RGB beyond what the decoder does.
pub fn iter_frames(
    path: &Path,
    every: usize,
    start: usize,
    stop: Option<usize>,
    want_image: bool,
) -> anyhow::Result<impl Iterator<Item = anyhow::Result<Frame>>> {
    let every = every.max(1);
    let r = FrameReader::open(path, want_image)?;
    let mut last: Option<i64> = None;
    Ok(r.take_while(move |f| match (f, stop) {
        (Ok(f), Some(s)) => f.index < s,
        _ => true,
    })
    .filter(move |f| match f {
        Ok(f) => f.index >= start && (f.index - start) % every == 0,
        Err(_) => true,
    })
    .map(move |f| {
        let f = f?;
        if let Some(l) = last {
            if f.pts <= l {
                bail!(
                    "non-monotonic presentation timestamps at frame {}: {} <= {}",
                    f.index,
                    f.pts,
                    l
                );
            }
        }
        last = Some(f.pts);
        Ok(f)
    }))
}

/// On-disk frame cache: `<root>/<sha256>/f<index:07>.png` + `index.json` (index → pts_s).
pub struct FrameCache {
    pub dir: PathBuf,
    pub max_frames: usize,
}

impl FrameCache {
    pub fn new(sha256: &str, root: &Path, max_frames: usize) -> Self {
        FrameCache {
            dir: root.join(sha256),
            max_frames,
        }
    }
    pub fn default_root() -> PathBuf {
        std::env::var_os("IGNIS_CACHE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("data/cache"))
            .join("frames")
    }
    fn index_path(&self) -> PathBuf {
        self.dir.join("index.json")
    }
    pub fn index(&self) -> anyhow::Result<BTreeMap<usize, f64>> {
        let p = self.index_path();
        if !p.exists() {
            return Ok(BTreeMap::new());
        }
        let m: BTreeMap<String, f64> = serde_json::from_slice(&std::fs::read(p)?)?;
        Ok(m.into_iter()
            .filter_map(|(k, v)| k.parse().ok().map(|k| (k, v)))
            .collect())
    }
    pub fn path_of(&self, index: usize) -> PathBuf {
        self.dir.join(format!("f{index:07}.png"))
    }
    pub fn extract(
        &self,
        video: &Path,
        every: usize,
        start: usize,
        stop: Option<usize>,
    ) -> anyhow::Result<serde_json::Value> {
        std::fs::create_dir_all(&self.dir)?;
        let mut idx = self.index()?;
        let (mut written, mut skipped, mut refused) = (0, 0, 0);
        for f in iter_frames(video, every, start, stop, true)? {
            let f = f?;
            if idx.contains_key(&f.index) && self.path_of(f.index).exists() {
                skipped += 1;
                continue;
            }
            if idx.len() >= self.max_frames {
                refused += 1;
                continue;
            }
            crate::imageio::write_rgb_png(&self.path_of(f.index), f.width, f.height, &f.rgb)?;
            idx.insert(f.index, f.pts_s);
            written += 1;
        }
        let m: serde_json::Map<String, serde_json::Value> = idx
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
            .collect();
        std::fs::write(self.index_path(), serde_json::to_vec_pretty(&m)?)?;
        Ok(
            serde_json::json!({"written": written, "already_cached": skipped, "refused_over_bound": refused, "cached_total": idx.len()}),
        )
    }
}

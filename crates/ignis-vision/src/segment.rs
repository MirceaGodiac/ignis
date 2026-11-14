//! Segmentation of the luminous flame region (V3.1 §9.2).
//!
//! * [`ClassicalSegmenter`] — the first comparator. Per frame: brightest channel `max(R,G,B)` (or
//!   luma / one channel) → excluded ROIs zeroed → 5×5 Gaussian blur → Otsu threshold with an
//!   absolute floor (a dark frame yields an empty mask instead of thresholded noise) → morphological
//!   open/close with elliptical elements → 8-connected area filter. It depends on the current frame
//!   only, so frames are segmented in parallel.
//!
//!   The primitives reproduce OpenCV's 8-bit results exactly (fixed-point separable Gaussian with
//!   `BORDER_REFLECT_101`, `THRESH_OTSU`, `getStructuringElement(MORPH_ELLIPSE)`, erode/dilate with
//!   the neutral border), which is what makes the Rust port comparable pixel-for-pixel.
//!
//! * SAM2.1 — a pretrained helper, not a calibrated instrument — runs in the Python sidecar
//!   (`sidecar/sam2/`, see [`crate::sam2`]); it is sequential and strictly causal.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// A frame's luminous mask and the frames it was computed from.
#[derive(Debug, Clone)]
pub struct MaskResult {
    pub frame: usize,
    pub width: usize,
    pub height: usize,
    pub mask: Vec<bool>,
    pub dependency_frames: Vec<i64>,
    pub info: Value,
}

/// The prompt/dependency rule of a recurrent forward tracker: the output at `t` depends on every
/// frame in `[min(prompts), t]`. A prompt after `t` is a leak.
pub fn causal_dependencies(prompt_frames: &[i64], t: i64) -> anyhow::Result<Vec<i64>> {
    let Some(&p0) = prompt_frames.iter().min() else {
        anyhow::bail!("a tracker needs at least one prompt")
    };
    let fut: Vec<i64> = prompt_frames.iter().copied().filter(|&p| p > t).collect();
    if !fut.is_empty() {
        anyhow::bail!("leakage: prompt on future frame(s) {fut:?} for frame {t}");
    }
    Ok((p0..=t).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClassicalSegmenter {
    /// "max" | "luma" | "r" | "g" | "b"; BASS flames are red (soot) and blue (CH*): max keeps both.
    pub channel: String,
    pub blur_px: usize,
    /// Absolute 8-bit floor; Otsu below this is noise.
    pub floor: u8,
    pub open_px: usize,
    pub close_px: usize,
    pub min_area_px: usize,
    pub exclude_rois_px: Vec<[f64; 4]>,
}

impl Default for ClassicalSegmenter {
    fn default() -> Self {
        ClassicalSegmenter {
            channel: "max".into(),
            blur_px: 5,
            floor: 40,
            open_px: 3,
            close_px: 7,
            min_area_px: 30,
            exclude_rois_px: vec![],
        }
    }
}

pub const CLASSICAL_METHOD_ID: &str = "vision.segment.classical_otsu";

impl ClassicalSegmenter {
    pub fn segment(
        &self,
        frame: usize,
        w: usize,
        h: usize,
        rgb: &[u8],
    ) -> anyhow::Result<MaskResult> {
        let mut g = channel(rgb, w, h, &self.channel)?;
        for r in &self.exclude_rois_px {
            fill_rect(&mut g, w, h, r, 0u8);
        }
        if self.blur_px > 1 {
            g = gaussian_blur(&g, w, h, self.blur_px | 1);
        }
        let otsu = otsu_threshold(&g);
        let thr = (otsu as f64).max(self.floor as f64);
        let mut m: Vec<bool> = g.iter().map(|&v| v as f64 > thr).collect();
        if self.open_px > 1 {
            let se = ellipse(self.open_px);
            m = dilate(&erode(&m, w, h, &se), w, h, &se);
        }
        if self.close_px > 1 {
            let se = ellipse(self.close_px);
            m = erode(&dilate(&m, w, h, &se), w, h, &se);
        }
        let (labels, areas) = label_components(&m, w, h);
        let keep: Vec<bool> = areas.iter().map(|&a| a >= self.min_area_px).collect();
        let n_keep = keep.iter().skip(1).filter(|&&k| k).count();
        let mask = labels.iter().map(|&l| l > 0 && keep[l as usize]).collect();
        Ok(MaskResult {
            frame,
            width: w,
            height: h,
            mask,
            dependency_frames: vec![frame as i64],
            info: json!({"threshold": thr, "otsu": otsu as f64, "components": n_keep}),
        })
    }

    pub fn manifest(&self) -> Value {
        json!({"method_id": CLASSICAL_METHOD_ID, "causal": true, "dependency": "current frame only", "params": self})
    }
}

pub fn channel(rgb: &[u8], w: usize, h: usize, ch: &str) -> anyhow::Result<Vec<u8>> {
    let px = rgb.chunks_exact(3).take(w * h);
    Ok(match ch {
        "max" => px.map(|p| p[0].max(p[1]).max(p[2])).collect(),
        // cv2.COLOR_RGB2GRAY fixed point (14 bits).
        "luma" => px
            .map(|p| {
                ((p[0] as u32 * 4899 + p[1] as u32 * 9617 + p[2] as u32 * 1868 + 8192) >> 14) as u8
            })
            .collect(),
        "r" => px.map(|p| p[0]).collect(),
        "g" => px.map(|p| p[1]).collect(),
        "b" => px.map(|p| p[2]).collect(),
        other => anyhow::bail!("unknown channel {other:?}"),
    })
}

/// numpy `a[int(y0):int(y1), int(x0):int(x1)] = v` for non-negative bounds (clipped to the image).
pub fn fill_rect<T: Copy>(a: &mut [T], w: usize, h: usize, r: &[f64; 4], v: T) {
    let c = |x: f64, n: usize| (x.trunc().max(0.0) as usize).min(n);
    let (x0, y0, x1, y1) = (c(r[0], w), c(r[1], h), c(r[2], w), c(r[3], h));
    for y in y0..y1.max(y0) {
        for x in x0..x1.max(x0) {
            a[y * w + x] = v;
        }
    }
}

fn reflect101(i: isize, n: usize) -> usize {
    let n = n as isize;
    if n == 1 {
        return 0;
    }
    let mut i = i;
    loop {
        if i < 0 {
            i = -i;
        } else if i >= n {
            i = 2 * n - 2 - i;
        } else {
            return i as usize;
        }
    }
}

/// Integer Gaussian taps for `cv2.GaussianBlur(k, sigma=0)` with 8-bit input (OpenCV's fixed
/// small kernels; these are exact in its bit-exact fixed-point path).
fn gaussian_taps(k: usize) -> Option<(Vec<u32>, u32)> {
    match k {
        3 => Some((vec![1, 2, 1], 2)),
        5 => Some((vec![1, 4, 6, 4, 1], 4)),
        7 => Some((vec![2, 7, 14, 18, 14, 7, 2], 6)),
        _ => None,
    }
}

/// `cv2.GaussianBlur(src, (k, k), 0)` for 8-bit, `BORDER_REFLECT_101`. Exact for k ∈ {3, 5, 7}
/// (OpenCV's bit-exact path: separable dyadic taps, rounding once at the end); other sizes use the
/// OpenCV σ rule with a float kernel.
pub fn gaussian_blur(src: &[u8], w: usize, h: usize, k: usize) -> Vec<u8> {
    let r = (k / 2) as isize;
    if let Some((taps, bits)) = gaussian_taps(k) {
        let mut tmp = vec![0u32; w * h];
        for y in 0..h {
            let row = &src[y * w..(y + 1) * w];
            for x in 0..w {
                let mut s = 0u32;
                for (i, &t) in taps.iter().enumerate() {
                    s += t * row[reflect101(x as isize + i as isize - r, w)] as u32;
                }
                tmp[y * w + x] = s;
            }
        }
        let shift = 2 * bits;
        let half = 1u32 << (shift - 1);
        let mut out = vec![0u8; w * h];
        for y in 0..h {
            let rows: Vec<usize> = (0..k)
                .map(|i| reflect101(y as isize + i as isize - r, h) * w)
                .collect();
            for x in 0..w {
                let mut s = 0u32;
                for (i, &t) in taps.iter().enumerate() {
                    s += t * tmp[rows[i] + x];
                }
                out[y * w + x] = ((s + half) >> shift).min(255) as u8;
            }
        }
        return out;
    }
    let sigma = 0.3 * ((k as f64 - 1.0) * 0.5 - 1.0) + 0.8;
    let mut kern: Vec<f64> = (0..k)
        .map(|i| (-((i as f64 - r as f64).powi(2)) / (2.0 * sigma * sigma)).exp())
        .collect();
    let s: f64 = kern.iter().sum();
    kern.iter_mut().for_each(|x| *x /= s);
    let mut tmp = vec![0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            tmp[y * w + x] = (0..k)
                .map(|i| kern[i] * src[y * w + reflect101(x as isize + i as isize - r, w)] as f64)
                .sum();
        }
    }
    let mut out = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let v: f64 = (0..k)
                .map(|i| kern[i] * tmp[reflect101(y as isize + i as isize - r, h) * w + x])
                .sum();
            out[y * w + x] = v.round().clamp(0.0, 255.0) as u8;
        }
    }
    out
}

/// `cv2.threshold(..., THRESH_OTSU)` threshold value for 8-bit (OpenCV's exact recurrence).
pub fn otsu_threshold(g: &[u8]) -> u8 {
    let mut hist = [0u64; 256];
    for &v in g {
        hist[v as usize] += 1;
    }
    let scale = 1.0 / g.len().max(1) as f64;
    let mut mu = 0.0;
    for (i, &c) in hist.iter().enumerate() {
        mu += i as f64 * c as f64;
    }
    mu *= scale;
    let (mut mu1, mut q1, mut max_sigma, mut max_val) = (0.0f64, 0.0f64, 0.0f64, 0u8);
    let eps = f32::EPSILON as f64;
    for (i, &c) in hist.iter().enumerate() {
        let p_i = c as f64 * scale;
        mu1 *= q1;
        q1 += p_i;
        let q2 = 1.0 - q1;
        if q1.min(q2) < eps || q1.max(q2) > 1.0 - eps {
            continue;
        }
        mu1 = (mu1 + i as f64 * p_i) / q1;
        let mu2 = (mu - q1 * mu1) / q2;
        let sigma = q1 * q2 * (mu1 - mu2) * (mu1 - mu2);
        if sigma > max_sigma {
            max_sigma = sigma;
            max_val = i as u8;
        }
    }
    max_val
}

/// `cv2.getStructuringElement(MORPH_ELLIPSE, (k, k))` as horizontal spans `(dy, dx0, dx1)` relative
/// to the centre anchor (inclusive dx0, exclusive dx1).
pub fn ellipse(k: usize) -> Vec<(isize, isize, isize)> {
    let r = (k / 2) as isize;
    let c = r;
    let inv_r2 = if r > 0 { 1.0 / (r * r) as f64 } else { 0.0 };
    let mut spans = Vec::new();
    for i in 0..k as isize {
        let dy = i - r;
        if dy.abs() <= r {
            let dx = (c as f64 * (((r * r - dy * dy) as f64) * inv_r2).sqrt()).round_ties_even()
                as isize;
            let j1 = (c - dx).max(0);
            let j2 = (c + dx + 1).min(k as isize);
            if j2 > j1 {
                spans.push((dy, j1 - c, j2 - c));
            }
        }
    }
    spans
}

/// Per-row prefix counts of `true`.
fn row_prefix(m: &[bool], w: usize, h: usize) -> Vec<u32> {
    let mut p = vec![0u32; (w + 1) * h];
    for y in 0..h {
        let base = y * (w + 1);
        for x in 0..w {
            p[base + x + 1] = p[base + x] + m[y * w + x] as u32;
        }
    }
    p
}

/// Binary erosion with the neutral border (pixels outside the image do not constrain), as OpenCV.
pub fn erode(m: &[bool], w: usize, h: usize, se: &[(isize, isize, isize)]) -> Vec<bool> {
    let p = row_prefix(m, w, h);
    let mut out = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            out[y * w + x] = se.iter().all(|&(dy, a, b)| {
                let yy = y as isize + dy;
                if yy < 0 || yy >= h as isize {
                    return true;
                }
                let x0 = (x as isize + a).clamp(0, w as isize) as usize;
                let x1 = (x as isize + b).clamp(0, w as isize) as usize;
                let base = yy as usize * (w + 1);
                p[base + x1] - p[base + x0] == (x1 - x0) as u32
            });
        }
    }
    out
}

pub fn dilate(m: &[bool], w: usize, h: usize, se: &[(isize, isize, isize)]) -> Vec<bool> {
    let p = row_prefix(m, w, h);
    let mut out = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            out[y * w + x] = se.iter().any(|&(dy, a, b)| {
                let yy = y as isize + dy;
                if yy < 0 || yy >= h as isize {
                    return false;
                }
                let x0 = (x as isize + a).clamp(0, w as isize) as usize;
                let x1 = (x as isize + b).clamp(0, w as isize) as usize;
                let base = yy as usize * (w + 1);
                p[base + x1] > p[base + x0]
            });
        }
    }
    out
}

/// Erosion where pixels outside the image count as `false` (`borderValue=0`), with a square box.
pub fn erode_box_zero_border(m: &[bool], w: usize, h: usize, k: usize) -> Vec<bool> {
    let r = (k / 2) as isize;
    let p = row_prefix(m, w, h);
    let mut out = vec![false; w * h];
    for y in 0..h as isize {
        if y - r < 0 || y + r >= h as isize {
            continue;
        }
        for x in 0..w as isize {
            if x - r < 0 || x + r >= w as isize {
                continue;
            }
            out[y as usize * w + x as usize] = (-r..=r).all(|dy| {
                let base = (y + dy) as usize * (w + 1);
                p[base + (x + r + 1) as usize] - p[base + (x - r) as usize] == k as u32
            });
        }
    }
    out
}

/// 8-connected component labels (0 = background) and the area of each label.
pub fn label_components(m: &[bool], w: usize, h: usize) -> (Vec<u32>, Vec<usize>) {
    let mut lab = vec![0u32; w * h];
    let mut areas = vec![0usize];
    let mut stack = Vec::new();
    for start in 0..w * h {
        if !m[start] || lab[start] != 0 {
            continue;
        }
        let id = areas.len() as u32;
        areas.push(0);
        lab[start] = id;
        stack.push(start);
        while let Some(i) = stack.pop() {
            areas[id as usize] += 1;
            let (x, y) = ((i % w) as isize, (i / w) as isize);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (xx, yy) = (x + dx, y + dy);
                    if xx < 0 || yy < 0 || xx >= w as isize || yy >= h as isize {
                        continue;
                    }
                    let j = yy as usize * w + xx as usize;
                    if m[j] && lab[j] == 0 {
                        lab[j] = id;
                        stack.push(j);
                    }
                }
            }
        }
    }
    (lab, areas)
}

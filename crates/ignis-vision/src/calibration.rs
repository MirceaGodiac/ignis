//! Pixel → specimen-plane calibration (V3.1 §3.4, §9.1, §9.3).
//!
//! Specimen-plane coordinates are `(f, s)` in metres: `f` longitudinal (the direction of spread is a
//! declared sign), `s` transverse in `[0, W]`. A homography is fitted by normalized DLT (Hartley)
//! from declared reference points (specimen holder corners, fiducials) cited from a source.
//!
//! Uncertainty:
//! * `scale_rel`: relative standard deviation of the local scale `sqrt|det J|` at the specimen
//!   centre, from a seeded Monte Carlo over the declared reference-point pixel uncertainty plus the
//!   fit residual (when over-determined).
//! * `parallax_rel`: the luminous flame stands off the specimen surface. With a declared stand-off
//!   bound `h` and camera distance `D` the projection error is bounded by `h / D`. Without both the
//!   record says the out-of-plane error is unbounded: the contour is projected geometry.
//!
//! Lens distortion (OpenCV model `k1 k2 p1 p2 [k3]`): when the camera matrix and coefficients are
//! declared, image points are undistorted before the homography; otherwise the record says so.
//! A `pixel_only` calibration keeps pixel coordinates with `resolution = unknown`.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context};
use ignis_core::contracts::{
    Calibration, CalibrationKind, Derivation, Method, Resolution, SourceLocator,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::METHOD_VERSION;

pub type Mat3 = [[f64; 3]; 3];

/// Declared calibration inputs (see `configs/vision/calibration.example.json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalibrationSpec {
    pub applies_to: Vec<String>,
    #[serde(default)]
    pub specimen_length_m: Option<f64>,
    #[serde(default)]
    pub specimen_width_m: Option<f64>,
    #[serde(default)]
    pub image_points_px: Vec<[f64; 2]>,
    #[serde(default)]
    pub plane_points_m: Vec<[f64; 2]>,
    #[serde(default = "one")]
    pub point_sigma_px: f64,
    #[serde(default)]
    pub camera_matrix: Option<Mat3>,
    #[serde(default)]
    pub distortion: Option<Vec<f64>>,
    #[serde(default)]
    pub camera_distance_m: Option<f64>,
    #[serde(default)]
    pub flame_standoff_max_m: Option<f64>,
    /// +1: the leading edge is the max-f luminous point; −1: min-f.
    #[serde(default = "one_i")]
    pub propagation_sign: i32,
    /// `[x0, y0, x1, y1]` excluded (overlays, holders).
    #[serde(default)]
    pub occlusion_rois_px: Vec<[f64; 4]>,
    /// Admitted `[x0, y0, x1, y1]`; `None` = the whole image.
    #[serde(default)]
    pub roi_px: Option<[f64; 4]>,
    #[serde(default)]
    pub pixel_only: bool,
    #[serde(default)]
    pub image_size_px: Option<[u32; 2]>,
    #[serde(default)]
    pub locator: Option<SourceLocator>,
    #[serde(default)]
    pub notes: Option<String>,
}

fn one() -> f64 {
    1.0
}
fn one_i() -> i32 {
    1
}

impl CalibrationSpec {
    /// Load a spec; keys starting with `_` are comments.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("calibration spec {}", path.display()))?;
        let mut v: serde_json::Value = serde_json::from_str(&text)?;
        if let Some(m) = v.as_object_mut() {
            m.retain(|k, _| !k.starts_with('_'));
        }
        Ok(serde_json::from_value(v)?)
    }
}

/// Fitted mapping with its contract record.
#[derive(Debug, Clone)]
pub struct PlaneCalibration {
    /// Pixel (undistorted) → plane `(f, s)`.
    pub h: Mat3,
    pub h_inv: Mat3,
    pub spec: CalibrationSpec,
    pub record: Calibration,
    pub scale_rel: Option<f64>,
    pub parallax_rel: Option<f64>,
    pub residual_rms_m: Option<f64>,
}

impl PlaneCalibration {
    pub fn units(&self) -> &'static str {
        if self.spec.pixel_only {
            "px"
        } else {
            "m"
        }
    }
    pub fn out_of_plane_bounded(&self) -> bool {
        self.parallax_rel.is_some()
    }
    pub fn combined_scale_rel(&self) -> Option<f64> {
        self.scale_rel
            .map(|s| s.hypot(self.parallax_rel.unwrap_or(0.0)))
    }
    fn lens(&self) -> Option<(&Mat3, &[f64])> {
        match (&self.spec.camera_matrix, &self.spec.distortion) {
            (Some(k), Some(d)) => Some((k, d.as_slice())),
            _ => None,
        }
    }
    pub fn px_to_plane(&self, uv: [f64; 2]) -> [f64; 2] {
        let p = match self.lens() {
            Some((k, d)) => undistort_point(uv, k, d),
            None => uv,
        };
        apply_h(&self.h, p)
    }
    /// Plane → raw-image pixel (re-distorted when distortion is declared).
    pub fn plane_to_px(&self, fs: [f64; 2]) -> [f64; 2] {
        let uv = apply_h(&self.h_inv, fs);
        match self.lens() {
            Some((k, d)) => distort_point(uv, k, d),
            None => uv,
        }
    }
    /// `(f_min, f_max, s_min, s_max)` of the admitted specimen domain in plane units.
    pub fn domain(&self) -> anyhow::Result<(f64, f64, f64, f64)> {
        if self.spec.pixel_only {
            let [w, h] = self.spec.image_size_px.unwrap_or([0, 0]);
            let [x0, y0, x1, y1] = self.spec.roi_px.unwrap_or([0.0, 0.0, w as f64, h as f64]);
            return Ok((x0, x1, y0, y1));
        }
        match (self.spec.specimen_length_m, self.spec.specimen_width_m) {
            (Some(l), Some(w)) => Ok((0.0, l, 0.0, w)),
            _ => bail!("a metric calibration needs specimen_length_m and specimen_width_m"),
        }
    }
}

pub fn apply_h(h: &Mat3, p: [f64; 2]) -> [f64; 2] {
    let x = h[0][0] * p[0] + h[0][1] * p[1] + h[0][2];
    let y = h[1][0] * p[0] + h[1][1] * p[1] + h[1][2];
    let w = h[2][0] * p[0] + h[2][1] * p[1] + h[2][2];
    [x / w, y / w]
}

pub fn mat_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut c = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            c[i][j] = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    c
}

pub fn mat_inv(m: &Mat3) -> anyhow::Result<Mat3> {
    let c = |r: usize, s: usize| {
        let (r1, r2) = ((r + 1) % 3, (r + 2) % 3);
        let (s1, s2) = ((s + 1) % 3, (s + 2) % 3);
        m[r1][s1] * m[r2][s2] - m[r1][s2] * m[r2][s1]
    };
    let det = m[0][0] * c(0, 0) + m[0][1] * c(0, 1) + m[0][2] * c(0, 2);
    if det.abs() < 1e-300 {
        bail!("singular 3x3 matrix");
    }
    let mut inv = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            inv[i][j] = c(j, i) / det;
        }
    }
    Ok(inv)
}

/// OpenCV distortion model: normalized `(x, y)` → distorted normalized.
fn distort_norm(x: f64, y: f64, d: &[f64]) -> (f64, f64) {
    let g = |i: usize| d.get(i).copied().unwrap_or(0.0);
    let (k1, k2, p1, p2, k3) = (g(0), g(1), g(2), g(3), g(4));
    let r2 = x * x + y * y;
    let radial = 1.0 + r2 * (k1 + r2 * (k2 + r2 * k3));
    (
        x * radial + 2.0 * p1 * x * y + p2 * (r2 + 2.0 * x * x),
        y * radial + p1 * (r2 + 2.0 * y * y) + 2.0 * p2 * x * y,
    )
}

/// `cv2.projectPoints` of an undistorted pixel (through `K⁻¹`, distortion, `K`).
pub fn distort_point(uv: [f64; 2], k: &Mat3, d: &[f64]) -> [f64; 2] {
    let (fx, fy, cx, cy) = (k[0][0], k[1][1], k[0][2], k[1][2]);
    let (x, y) = ((uv[0] - cx) / fx, (uv[1] - cy) / fy);
    let (xd, yd) = distort_norm(x, y, d);
    [fx * xd + cx, fy * yd + cy]
}

/// `cv2.undistortPoints(..., P=K)`: fixed-point iteration as OpenCV does, then refined.
pub fn undistort_point(uv: [f64; 2], k: &Mat3, d: &[f64]) -> [f64; 2] {
    let (fx, fy, cx, cy) = (k[0][0], k[1][1], k[0][2], k[1][2]);
    let (x0, y0) = ((uv[0] - cx) / fx, (uv[1] - cy) / fy);
    let g = |i: usize| d.get(i).copied().unwrap_or(0.0);
    let (k1, k2, p1, p2, k3) = (g(0), g(1), g(2), g(3), g(4));
    let (mut x, mut y) = (x0, y0);
    for _ in 0..50 {
        let r2 = x * x + y * y;
        let icdist = 1.0 / (1.0 + r2 * (k1 + r2 * (k2 + r2 * k3)));
        let dx = 2.0 * p1 * x * y + p2 * (r2 + 2.0 * x * x);
        let dy = p1 * (r2 + 2.0 * y * y) + 2.0 * p2 * x * y;
        let (nx, ny) = ((x0 - dx) * icdist, (y0 - dy) * icdist);
        let done = (nx - x).abs() < 1e-14 && (ny - y).abs() < 1e-14;
        x = nx;
        y = ny;
        if done {
            break;
        }
    }
    [fx * x + cx, fy * y + cy]
}

/// Symmetric eigen-decomposition by cyclic Jacobi; returns (eigenvalues, eigenvectors as columns).
fn jacobi_eigen<const N: usize>(mut a: [[f64; N]; N]) -> ([f64; N], [[f64; N]; N]) {
    let mut v = [[0.0; N]; N];
    for (i, row) in v.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    for _sweep in 0..100 {
        let off: f64 = (0..N)
            .flat_map(|i| (0..N).filter(move |&j| j != i).map(move |j| (i, j)))
            .map(|(i, j)| a[i][j] * a[i][j])
            .sum();
        let norm: f64 = (0..N).map(|i| a[i][i] * a[i][i]).sum::<f64>() + off;
        if off <= 1e-30 * norm.max(1e-300) {
            break;
        }
        for p in 0..N {
            for q in p + 1..N {
                if a[p][q].abs() < 1e-300 {
                    continue;
                }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..N {
                    let (akp, akq) = (a[k][p], a[k][q]);
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                for k in 0..N {
                    let (apk, aqk) = (a[p][k], a[q][k]);
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
                for row in v.iter_mut() {
                    let (vkp, vkq) = (row[p], row[q]);
                    row[p] = c * vkp - s * vkq;
                    row[q] = s * vkp + c * vkq;
                }
            }
        }
    }
    let mut e = [0.0; N];
    for i in 0..N {
        e[i] = a[i][i];
    }
    (e, v)
}

/// Normalized DLT (Hartley). Least squares for > 4 points; exact for 4 in general position.
pub fn fit_homography(src: &[[f64; 2]], dst: &[[f64; 2]]) -> anyhow::Result<Mat3> {
    if src.len() < 4 || src.len() != dst.len() {
        bail!("a homography needs at least 4 corresponding points");
    }
    let norm = |p: &[[f64; 2]]| -> (Vec<[f64; 2]>, Mat3) {
        let n = p.len() as f64;
        let c = [
            p.iter().map(|q| q[0]).sum::<f64>() / n,
            p.iter().map(|q| q[1]).sum::<f64>() / n,
        ];
        let d = p
            .iter()
            .map(|q| ((q[0] - c[0]).powi(2) + (q[1] - c[1]).powi(2)).sqrt())
            .sum::<f64>()
            / n;
        let sc = if d > 0.0 { 2f64.sqrt() / d } else { 1.0 };
        let t = [
            [sc, 0.0, -sc * c[0]],
            [0.0, sc, -sc * c[1]],
            [0.0, 0.0, 1.0],
        ];
        (p.iter().map(|&q| apply_h(&t, q)).collect(), t)
    };
    let (a, ta) = norm(src);
    let (b, tb) = norm(dst);
    let mut ata = [[0.0; 9]; 9];
    for (&[x, y], &[u, v]) in a.iter().zip(&b) {
        let r1 = [-x, -y, -1.0, 0.0, 0.0, 0.0, u * x, u * y, u];
        let r2 = [0.0, 0.0, 0.0, -x, -y, -1.0, v * x, v * y, v];
        for i in 0..9 {
            for j in 0..9 {
                ata[i][j] += r1[i] * r1[j] + r2[i] * r2[j];
            }
        }
    }
    let (e, vecs) = jacobi_eigen(ata);
    let mut order: Vec<usize> = (0..9).collect();
    order.sort_by(|&i, &j| e[i].total_cmp(&e[j]));
    let (lmin2, lmax) = (e[order[1]].max(0.0), e[order[8]].max(0.0));
    // singular values are sqrt(eigenvalues): sv[-2] < 1e-12 sv[0]
    if lmin2.sqrt() < 1e-12 * lmax.sqrt() {
        bail!("degenerate reference points (collinear or repeated)");
    }
    let k = order[0];
    let mut hn = [[0.0; 3]; 3];
    for i in 0..9 {
        hn[i / 3][i % 3] = vecs[i][k];
    }
    let h = mat_mul(&mat_mul(&mat_inv(&tb)?, &hn), &ta);
    let s = h[2][2];
    Ok(h.map(|r| r.map(|x| x / s)))
}

/// `sqrt|det J|` of pixel → plane at `uv` (plane units per pixel).
pub fn local_scale(h: &Mat3, uv: [f64; 2], eps: f64) -> f64 {
    let a = apply_h(h, [uv[0] + eps, uv[1]]);
    let b = apply_h(h, [uv[0] - eps, uv[1]]);
    let c = apply_h(h, [uv[0], uv[1] + eps]);
    let d = apply_h(h, [uv[0], uv[1] - eps]);
    let fx = [(a[0] - b[0]) / (2.0 * eps), (a[1] - b[1]) / (2.0 * eps)];
    let fy = [(c[0] - d[0]) / (2.0 * eps), (c[1] - d[1]) / (2.0 * eps)];
    (fx[0] * fy[1] - fx[1] * fy[0]).abs().sqrt()
}

/// Seeded normal generator (SplitMix64 + Box–Muller): reproducible Monte Carlo.
pub struct Normal {
    state: u64,
    spare: Option<f64>,
}

impl Normal {
    pub fn new(seed: u64) -> Self {
        Normal {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
            spare: None,
        }
    }
    fn uniform(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }
    pub fn sample(&mut self) -> f64 {
        if let Some(s) = self.spare.take() {
            return s;
        }
        let (u1, u2) = (self.uniform(), self.uniform());
        let r = (-2.0 * u1.ln()).sqrt();
        let t = std::f64::consts::TAU * u2;
        self.spare = Some(r * t.sin());
        r * t.cos()
    }
}

pub fn calibrate(
    spec: &CalibrationSpec,
    asset_sha256: Option<&str>,
    n_mc: usize,
    seed: u64,
) -> anyhow::Result<PlaneCalibration> {
    let mut flags: Vec<String> = Vec::new();
    let eye = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let (h, scale_rel, parallax, resid, resolution, params, unc);
    if spec.pixel_only {
        h = eye;
        scale_rel = None;
        parallax = None;
        resid = None;
        resolution = Resolution::Unknown;
        flags.push(
            "pixel_only: no declared reference points; coordinates are pixels, not metres".into(),
        );
        params = json!({"homography": h, "pixel_only": 1.0});
        unc = BTreeMap::new();
    } else {
        let lens = spec.camera_matrix.as_ref().zip(spec.distortion.as_deref());
        let img: Vec<[f64; 2]> = spec
            .image_points_px
            .iter()
            .map(|&p| match lens {
                Some((k, d)) => undistort_point(p, k, d),
                None => p,
            })
            .collect();
        let pl = &spec.plane_points_m;
        h = fit_homography(&img, pl)?;
        let r2: f64 = img
            .iter()
            .zip(pl)
            .map(|(&p, q)| {
                let m = apply_h(&h, p);
                (m[0] - q[0]).powi(2) + (m[1] - q[1]).powi(2)
            })
            .sum::<f64>()
            / img.len() as f64;
        let rr = r2.sqrt();
        resid = Some(rr);
        let (l, w) = (
            spec.specimen_length_m.unwrap_or(0.0),
            spec.specimen_width_m.unwrap_or(0.0),
        );
        let centre_px = apply_h(&mat_inv(&h)?, [l / 2.0, w / 2.0]);
        let k0 = local_scale(&h, centre_px, 0.5);
        // the fit residual in pixels adds to the declared point uncertainty when over-determined
        let resid_px = if img.len() > 4 { rr / k0 } else { 0.0 };
        let sig = spec.point_sigma_px.hypot(resid_px);
        let mut rng = Normal::new(seed);
        let mut ks = Vec::with_capacity(n_mc);
        for _ in 0..n_mc {
            let pert: Vec<[f64; 2]> = img
                .iter()
                .map(|p| [p[0] + sig * rng.sample(), p[1] + sig * rng.sample()])
                .collect();
            if let Ok(hm) = fit_homography(&pert, pl) {
                ks.push(local_scale(&hm, centre_px, 0.5));
            }
        }
        scale_rel = (!ks.is_empty()).then(|| {
            let m = ks.iter().sum::<f64>() / ks.len() as f64;
            (ks.iter().map(|k| (k - m).powi(2)).sum::<f64>() / ks.len() as f64).sqrt() / k0
        });
        parallax = match (spec.camera_distance_m, spec.flame_standoff_max_m) {
            (Some(dd), Some(hh)) if dd > 0.0 => Some(hh / dd),
            _ => {
                flags.push(
                    "out_of_plane_unbounded: luminous flame stand-off/camera distance not declared; \
                     contour is projected geometry on the specimen plane"
                        .into(),
                );
                None
            }
        };
        if lens.is_none() {
            flags.push("lens_distortion_not_corrected".into());
        }
        resolution = if parallax.is_some() {
            Resolution::Resolved
        } else {
            Resolution::Unknown
        };
        params = json!({
            "homography": h, "pixel_only": 0.0, "metres_per_px_centre": k0,
            "specimen_length_m": l, "specimen_width_m": w,
            "propagation_sign": spec.propagation_sign as f64, "n_reference_points": img.len() as f64,
            "lens_distortion": lens.map(|(k, d)| json!({"camera_matrix": k, "coefficients": d})),
        });
        let mut u = BTreeMap::from([
            ("point_sigma_px".to_string(), spec.point_sigma_px),
            ("residual_rms_m".to_string(), rr),
        ]);
        if let Some(s) = scale_rel {
            u.insert("scale_rel".into(), s);
        }
        if let Some(p) = parallax {
            u.insert("parallax_rel".into(), p);
        }
        unc = u;
    }
    let locator = spec
        .locator
        .clone()
        .or_else(|| asset_sha256.map(|s| crate::locator(s, None, None, None)));
    let mut notes = flags;
    notes.extend(spec.notes.clone());
    let derivation = Derivation {
        method: Method::Numerical,
        method_id: "vision.calibration.homography_dlt".into(),
        method_version: METHOD_VERSION.into(),
        inputs: asset_sha256
            .map(|s| vec![s.to_string()])
            .unwrap_or_default(),
        agent: None,
        notes: (!notes.is_empty()).then(|| notes.join("; ")),
    };
    let mut record = Calibration {
        id: String::new(),
        kind: CalibrationKind::Homography,
        applies_to: spec.applies_to.clone(),
        parameters: params.as_object().cloned().unwrap_or_default(),
        uncertainty: unc,
        locator,
        derivation,
        resolution,
    };
    let hash = ignis_core::hashing::content_hash(&record, &["id"]);
    record.id = format!("cal-{}", &ignis_core::hashing::hex_part(&hash)[..16]);
    Ok(PlaneCalibration {
        h_inv: mat_inv(&h)?,
        h,
        spec: spec.clone(),
        record,
        scale_rel,
        parallax_rel: parallax,
        residual_rms_m: resid,
    })
}

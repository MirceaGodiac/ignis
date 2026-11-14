//! Leading-luminous contour on a fixed transverse grid and topology status (V3.1 §3.4, §9.1, §9.3).
//!
//! Definition (fixed before annotation): for each grid node `s_j = s0 + j (s1 − s0) / (K − 1)` the
//! sampling line `{(f, s_j) : f ∈ [f_min, f_max]}` on the specimen plane is mapped to the image and
//! the luminous mask is sampled along it. The leading-luminous position `f_j` is the luminous sample
//! furthest in the declared propagation direction. Nodes without a luminous sample, outside the
//! image or inside an occlusion ROI are `None` — never interpolated.
//!
//! Status (first match wins, nothing is smoothed):
//!   out_of_field   no luminous pixels in the admitted field, the luminous region touches the
//!                  admitted-field edge at its leading point, or it reaches no grid node
//!   occluded       more than `occluded_frac` of the nodes fall in occlusion ROIs / outside the image
//!   detached       no luminous component touches the specimen domain, or a component ahead of the
//!                  specimen-anchored flame is separated from it by more than `gap`
//!   multivalued    on some node the line crosses separate luminous runs separated by more than `gap`
//!   single_valued  otherwise

use ignis_core::contracts::{ContourObservation, ContourStatus, Derivation, EvidenceClass, Method};

use crate::calibration::PlaneCalibration;
use crate::segment::{erode_box_zero_border, fill_rect, label_components};
use crate::METHOD_VERSION;

#[derive(Debug, Clone)]
pub struct ContourGeometry {
    pub s: Vec<f64>,
    pub f: Vec<Option<f64>>,
    pub status: ContourStatus,
    pub reasons: Vec<String>,
    /// Luminous runs crossed per node (0 = none / unresolved).
    pub n_runs: Vec<usize>,
}

/// `numpy.linspace(a, b, n)` bit-for-bit (`i * step + a`, last point exactly `b`).
pub fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![a];
    }
    let step = (b - a) / (n - 1) as f64;
    let mut v: Vec<f64> = (0..n)
        .map(|i| {
            if step == 0.0 {
                (i as f64 / (n - 1) as f64) * (b - a) + a
            } else {
                i as f64 * step + a
            }
        })
        .collect();
    v[n - 1] = b;
    v
}

/// Precomputed sampling geometry for one calibration and image size.
pub struct ContourExtractor {
    pub cal: PlaneCalibration,
    pub k: usize,
    pub samples_per_line: usize,
    pub occluded_frac: f64,
    pub edge_margin_px: usize,
    pub width: usize,
    pub height: usize,
    f_lin: Vec<f64>,
    s: Vec<f64>,
    gap: f64,
    sign: i32,
    /// K×N rounded/clipped pixel index of each sample.
    sample_idx: Vec<usize>,
    /// K×N: sample outside the image or inside an occlusion ROI.
    occ: Vec<bool>,
    node_occ: Vec<bool>,
    field: Vec<bool>,
    inner: Vec<bool>,
    domain: Vec<bool>,
    /// `f` of every pixel (px → plane), for leading-point tests.
    f_of_px: Vec<f64>,
}

impl ContourExtractor {
    pub fn new(
        cal: PlaneCalibration,
        k: usize,
        width: usize,
        height: usize,
    ) -> anyhow::Result<Self> {
        let (samples_per_line, gap_frac, occluded_frac, edge_margin_px) =
            (400usize, 0.02, 0.5, 2usize);
        let (f0, f1, s0, s1) = cal.domain()?;
        let f_lin = linspace(f0, f1, samples_per_line);
        let s = linspace(s0, s1, k);
        let gap = gap_frac * (f1 - f0);
        let (w, h) = (width, height);
        let n = samples_per_line;
        let mut sample_idx = vec![0usize; k * n];
        let mut occ = vec![false; k * n];
        for j in 0..k {
            for i in 0..n {
                let [u, v] = cal.plane_to_px([f_lin[i], s[j]]);
                let (ui, vi) = (u.round_ties_even(), v.round_ties_even());
                let mut o = ui < 0.0
                    || vi < 0.0
                    || ui >= w as f64
                    || vi >= h as f64
                    || !ui.is_finite()
                    || !vi.is_finite();
                for r in &cal.spec.occlusion_rois_px {
                    o |= u >= r[0] && u < r[2] && v >= r[1] && v < r[3];
                }
                occ[j * n + i] = o;
                let uc = if ui.is_finite() {
                    ui.clamp(0.0, (w - 1) as f64) as usize
                } else {
                    0
                };
                let vc = if vi.is_finite() {
                    vi.clamp(0.0, (h - 1) as f64) as usize
                } else {
                    0
                };
                sample_idx[j * n + i] = vc * w + uc;
            }
        }
        let node_occ = (0..k)
            .map(|j| occ[j * n..(j + 1) * n].iter().filter(|&&o| o).count() as f64 / n as f64 > 0.5)
            .collect();
        // admitted field: ROI (python round, half-even) minus occlusion ROIs
        let mut field = vec![true; w * h];
        if let Some(r) = cal.spec.roi_px {
            field.iter_mut().for_each(|x| *x = false);
            let rr = r.map(|x| x.round_ties_even());
            fill_rect(&mut field, w, h, &rr, true);
        }
        for r in &cal.spec.occlusion_rois_px {
            fill_rect(&mut field, w, h, r, false);
        }
        let inner = erode_box_zero_border(&field, w, h, 2 * edge_margin_px + 1);
        let poly: Vec<[f64; 2]> = [[f0, s0], [f1, s0], [f1, s1], [f0, s1]]
            .iter()
            .map(|&p| cal.plane_to_px(p).map(|x| x.round_ties_even()))
            .collect();
        let domain = fill_poly(&poly, w, h);
        let mut f_of_px = vec![0.0; w * h];
        for y in 0..h {
            for x in 0..w {
                f_of_px[y * w + x] = cal.px_to_plane([x as f64, y as f64])[0];
            }
        }
        let sign = if cal.spec.propagation_sign == 0 {
            1
        } else {
            cal.spec.propagation_sign.signum()
        };
        Ok(ContourExtractor {
            cal,
            k,
            samples_per_line,
            occluded_frac,
            edge_margin_px,
            width: w,
            height: h,
            f_lin,
            s,
            gap,
            sign,
            sample_idx,
            occ,
            node_occ,
            field,
            inner,
            domain,
            f_of_px,
        })
    }

    pub fn s_grid(&self) -> &[f64] {
        &self.s
    }

    fn leading(&self, it: impl Iterator<Item = usize>) -> Option<f64> {
        let mut best: Option<f64> = None;
        for i in it {
            let f = self.f_of_px[i];
            best = Some(match best {
                None => f,
                Some(b) => {
                    if self.sign > 0 {
                        b.max(f)
                    } else {
                        b.min(f)
                    }
                }
            });
        }
        best
    }

    pub fn extract(&self, mask: &[bool]) -> ContourGeometry {
        let (w, h, k, n) = (self.width, self.height, self.k, self.samples_per_line);
        assert_eq!(mask.len(), w * h, "mask size does not match the extractor");
        let lum: Vec<bool> = mask
            .iter()
            .zip(&self.field)
            .map(|(&m, &f)| m && f)
            .collect();
        let df = self.f_lin[1] - self.f_lin[0];
        let run_gap = self.gap.max(1.5 * df);
        let mut f_out = Vec::with_capacity(k);
        let mut n_runs = Vec::with_capacity(k);
        let mut multi = false;
        for j in 0..k {
            if self.node_occ[j] {
                f_out.push(None);
                n_runs.push(0);
                continue;
            }
            let idx: Vec<usize> = (0..n)
                .filter(|&i| lum[self.sample_idx[j * n + i]] && !self.occ[j * n + i])
                .collect();
            if idx.is_empty() {
                f_out.push(None);
                n_runs.push(0);
                continue;
            }
            let fv: Vec<f64> = idx.iter().map(|&i| self.f_lin[i]).collect();
            let runs = 1 + fv.windows(2).filter(|p| p[1] - p[0] > run_gap).count();
            n_runs.push(runs);
            multi |= runs > 1;
            let lead = if self.sign > 0 {
                fv.iter().copied().fold(f64::MIN, f64::max)
            } else {
                fv.iter().copied().fold(f64::MAX, f64::min)
            };
            f_out.push(Some(lead));
        }
        let geom = |f: Vec<Option<f64>>, st, why: &str, runs: Vec<usize>| ContourGeometry {
            s: self.s.clone(),
            f,
            status: st,
            reasons: if why.is_empty() {
                vec![]
            } else {
                vec![why.to_string()]
            },
            n_runs: runs,
        };
        if !lum.iter().any(|&x| x) {
            return geom(
                vec![None; k],
                ContourStatus::OutOfField,
                "no luminous pixels in field",
                vec![0; k],
            );
        }
        let lead = self.leading((0..w * h).filter(|&i| lum[i]));
        if self.touches_leading_boundary(&lum, lead) {
            return geom(
                f_out,
                ContourStatus::OutOfField,
                "luminous region touches the field boundary in the propagation direction",
                n_runs,
            );
        }
        if f_out.iter().all(|x| x.is_none()) {
            return geom(
                f_out,
                ContourStatus::OutOfField,
                "luminous region reaches no grid node (front not resolvable at grid resolution)",
                n_runs,
            );
        }
        let occ_nodes = self.node_occ.iter().filter(|&&o| o).count();
        if occ_nodes as f64 > self.occluded_frac * k as f64 {
            return geom(
                f_out,
                ContourStatus::Occluded,
                &format!("{occ_nodes}/{k} nodes occluded or outside the image"),
                n_runs,
            );
        }
        let (lab, areas) = label_components(&lum, w, h);
        let nc = areas.len();
        let mut anchored = vec![false; nc];
        for i in 0..w * h {
            if lab[i] > 0 && self.domain[i] {
                anchored[lab[i] as usize] = true;
            }
        }
        let n_anchored = anchored.iter().filter(|&&a| a).count();
        if n_anchored == 0 {
            return geom(
                f_out,
                ContourStatus::Detached,
                "no luminous component touches the specimen domain",
                n_runs,
            );
        }
        if nc - 1 > n_anchored {
            let lead_anchor =
                self.leading((0..w * h).filter(|&i| lab[i] > 0 && anchored[lab[i] as usize]));
            let mut comp_lead: Vec<Option<f64>> = vec![None; nc];
            for i in 0..w * h {
                let c = lab[i] as usize;
                if c > 0 && !anchored[c] {
                    let f = self.f_of_px[i];
                    comp_lead[c] = Some(match comp_lead[c] {
                        None => f,
                        Some(b) => {
                            if self.sign > 0 {
                                b.max(f)
                            } else {
                                b.min(f)
                            }
                        }
                    });
                }
            }
            for lf in comp_lead.into_iter().flatten() {
                if let Some(la) = lead_anchor {
                    if self.sign as f64 * (lf - la) > self.gap {
                        return geom(
                            f_out,
                            ContourStatus::Detached,
                            "a luminous component ahead of the anchored flame is separated from it",
                            n_runs,
                        );
                    }
                }
            }
        }
        if multi {
            return geom(
                f_out,
                ContourStatus::Multivalued,
                "sampling line crosses separate luminous runs",
                n_runs,
            );
        }
        geom(f_out, ContourStatus::SingleValued, "", n_runs)
    }

    /// True when a luminous pixel within `edge_margin_px` of the field edge is (within two samples)
    /// the leading-most luminous point: the front may continue beyond what is visible.
    fn touches_leading_boundary(&self, lum: &[bool], lead: Option<f64>) -> bool {
        let Some(lead) = lead else { return false };
        let tol = 2.0 * (self.f_lin[1] - self.f_lin[0]);
        (0..lum.len()).any(|i| lum[i] && !self.inner[i] && (self.f_of_px[i] - lead).abs() <= tol)
    }
}

/// `cv2.fillPoly` of one convex/simple polygon with integer vertices: pixels inside or on an edge.
fn fill_poly(poly: &[[f64; 2]], w: usize, h: usize) -> Vec<bool> {
    let mut m = vec![false; w * h];
    let on_seg = |p: [f64; 2], a: [f64; 2], b: [f64; 2]| {
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        cross.abs() < 1e-9
            && p[0] >= a[0].min(b[0]) - 1e-9
            && p[0] <= a[0].max(b[0]) + 1e-9
            && p[1] >= a[1].min(b[1]) - 1e-9
            && p[1] <= a[1].max(b[1]) + 1e-9
    };
    let ymin = poly.iter().map(|p| p[1]).fold(f64::MAX, f64::min);
    let ymax = poly.iter().map(|p| p[1]).fold(f64::MIN, f64::max);
    for y in 0..h {
        if (y as f64) < ymin || (y as f64) > ymax {
            continue;
        }
        for x in 0..w {
            let p = [x as f64, y as f64];
            let mut inside = false;
            let mut edge = false;
            for i in 0..poly.len() {
                let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
                if on_seg(p, a, b) {
                    edge = true;
                    break;
                }
                if (a[1] > p[1]) != (b[1] > p[1])
                    && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
                {
                    inside = !inside;
                }
            }
            m[y * w + x] = inside || edge;
        }
    }
    m
}

pub struct ObservationContext<'a> {
    pub cal: &'a PlaneCalibration,
    pub run_id: &'a str,
    pub asset_sha256: &'a str,
    pub asset_path: Option<&'a str>,
    pub time_origin_s: f64,
    pub method_id: &'a str,
    pub evidence_class: EvidenceClass,
}

pub fn make_observation(
    geom: &ContourGeometry,
    ctx: &ObservationContext<'_>,
    frame: usize,
    pts_s: f64,
    dependency_frames: &[i64],
    extra_notes: &[String],
) -> ContourObservation {
    let cal = ctx.cal;
    let reasons = if geom.reasons.is_empty() {
        "-".to_string()
    } else {
        geom.reasons.join("|")
    };
    let mut notes = vec![
        format!("units={}", cal.units()),
        format!("status_reasons={reasons}"),
        format!(
            "time_origin_s={} (container PTS minus declared origin)",
            ctx.time_origin_s
        ),
    ];
    if cal.spec.pixel_only {
        notes.push("UNCALIBRATED: s_m/f_m hold pixel coordinates, not metres".into());
    }
    if !cal.out_of_plane_bounded() && !cal.spec.pixel_only {
        notes.push("out_of_plane_unbounded".into());
    }
    notes.extend(extra_notes.iter().cloned());
    let t = pts_s - ctx.time_origin_s;
    let mut deps: Vec<i64> = dependency_frames.to_vec();
    deps.sort_unstable();
    deps.dedup();
    let mut obs = ContourObservation {
        id: String::new(),
        run_id: ctx.run_id.to_string(),
        time_s: t,
        available_at_s: t,
        s_m: geom.s.clone(),
        f_m: geom.f.clone(),
        status: geom.status,
        calibration_id: cal.record.id.clone(),
        scale_uncertainty_rel: cal.combined_scale_rel(),
        mask_dependency_frames: deps,
        frame: frame as i64,
        evidence_class: ctx.evidence_class,
        locator: crate::locator(
            ctx.asset_sha256,
            ctx.asset_path,
            Some(frame as i64),
            Some(pts_s),
        ),
        derivation: Derivation {
            method: Method::Numerical,
            method_id: ctx.method_id.to_string(),
            method_version: METHOD_VERSION.into(),
            inputs: vec![
                format!("sha256:{}", ctx.asset_sha256),
                cal.record.id.clone(),
            ],
            agent: None,
            notes: Some(notes.join("; ")),
        },
    };
    let hash = ignis_core::hashing::content_hash(&obs, &["id"]);
    obs.id = format!("contour-{}", &ignis_core::hashing::hex_part(&hash)[..16]);
    obs
}

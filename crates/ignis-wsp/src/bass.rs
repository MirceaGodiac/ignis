//! `bass.wsp` — the strict BASS forward: where a flame front seen by a calibrated camera will be
//! after the cutoff, from fixed handwritten coefficients of a diagonal spectral operator.
//!
//! ```text
//! xi = (s - s_first) / (s_last - s_first)            transverse coordinate, 0..1
//! phi_0 = 1,  phi_m = sqrt(2) cos(m pi xi)           cosine modes, m = 0..K-1
//! a_m   = integral f phi_m d xi                      (trapezoid; a declared projection, no fit)
//! v_m   = finite difference of the window means of a_m over the last two windows of the prefix
//!
//! a_m(t_n + D) = a_m exp(-damp_m D) + gain_m v_m T_m (1 - exp(-D / T_m))
//! f(xi, t)     = r(xi) + sum_m a_m(t) phi_m(xi)      r = what the K modes do not resolve, carried
//! ```
//!
//! The mean front (mode 0) keeps the velocity the prefix shows; the shape modes relax and their
//! drift fades. Twelve coefficients, each one a rate, a time or a fraction with a physical reading.
//!
//! What it predicts is a geometric observable: the position of the luminous front in the units of
//! the admitted contours. It is a metric forecast only when those contours are metric, so a prefix
//! whose contours are pixel coordinates is refused. Nothing here reads RGB intensity, and nothing
//! claims a temperature, a species, a pyrolysis front or an extinction.

use anyhow::bail;
use serde::Serialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    AnalysisArtifact, ContourObservation, ContourStatus, Family, FeatureSpec, ForwardContract,
    ForwardProfile, ModelRelease, ModelSpec, PrefixBundle, QualificationState, Quantity, SpecId,
    Support, SupportGate, TargetSpec, TensorRole, WspSchema,
};
use ignis_core::expert::{Expert, ExpertResult, FitContext, FitData, Refusal};
use ignis_core::prefix::ProspectivePolicy;
use ignis_core::registry::LoadedRelease;
use ignis_core::resources::Profile;
use ignis_core::views::View;

use crate::common::{self, admitted, refuse, Body};
use crate::quad::normal_quantile;
use crate::spec::{authored, constant, Params};

pub const SPEC_ID: &str = "bass-wsp-v1";
pub const MODEL_NAME: &str = "bass.wsp";
pub const FORWARD_VERSION: &str = "1";
/// Cosine modes carried by the operator.
pub const MODES: usize = 4;

/// The frozen specification.
pub fn spec() -> ModelSpec {
    use TensorRole::*;
    let feature = |name: &str, unit: &str, source: &str, transform: &str| FeatureSpec {
        name: name.into(),
        unit: unit.into(),
        source: source.into(),
        transform: transform.into(),
    };
    let k = MODES as u32;
    ModelSpec {
        schema_version: WspSchema::V1,
        spec_id: SpecId::new(SPEC_ID).expect("id"),
        family: Family::Bass,
        model_name: MODEL_NAME.into(),
        target: TargetSpec {
            quantity: Quantity::FrontPosition,
            unit: "m".into(),
            description: "Position of the luminous flame front after the cutoff, in the camera plane \
                          a metric calibration defines: the front contour f(s, t) on the transverse \
                          grid of the admitted contours, and its spatial mean with an assumed spread."
                .into(),
            excludes: vec![
                "any forecast from pixel-only contours (no metric calibration, no metric target)".into(),
                "temperature, heat flux, species or soot inferred from image intensity".into(),
                "the pyrolysis or char front (the camera sees luminous emission)".into(),
                "extinction or flammability limits (a front leaving view is not an extinction)".into(),
                "effects of oxygen, flow, pressure or thickness: no metric run exists to support one".into(),
                "a calibrated posterior: the spread is assumed".into(),
            ],
        },
        features: vec![
            feature("a_m", "m", "observation:contour@prefix_last",
                    "cosine-mode amplitudes of the last usable contour: trapezoid integral of f * phi_m over xi @1"),
            feature("v_m", "m/s", "observation:contour@prefix_windows",
                    "(mean a_m over the last window - mean over the window before it) / (difference of their \
                     mean times); windows span enc.window_frac of the prefix duration each @1"),
            feature("lead", "s", "state:lead_time", "t - time of the last usable contour @1"),
        ],
        domain: vec![],
        tensors: vec![
            authored("op.gain", &[k], Weight, Some("1"), Some((0.0, 1.5)),
                     "Fraction of each mode's observed velocity that the forecast carries."),
            authored("op.persist_s", &[k], PhysicalParameter, Some("s"), Some((0.01, 1.0e5)),
                     "Time over which each mode's carried drift persists (large = steady spread)."),
            authored("op.damp_per_s", &[k], PhysicalParameter, Some("1/s"), Some((0.0, 100.0)),
                     "Relaxation rate of each mode's amplitude (front smoothing)."),
            authored("unc.sigma_m", &[], NoiseParameter, Some("m"), Some((0.0, 0.05)),
                     "Assumed sd of the measured mean front position at zero lead."),
            authored("unc.rel_v", &[], NoiseParameter, Some("1"), Some((0.0, 5.0)),
                     "Assumed relative sd of the carried mean velocity."),
            authored("unc.sigma_v_m_s", &[], NoiseParameter, Some("m/s"), Some((0.0, 0.1)),
                     "Assumed absolute sd of the mean velocity, whatever its size."),
            constant("enc.window_frac", 0.34, "1", "Each velocity window spans this fraction of the prefix duration."),
            constant("enc.min_frames", 4.0, "1", "Fewest usable contours for a forecast."),
            constant("enc.max_missing_frac", 0.3, "1", "Largest fraction of unresolved nodes of a usable contour."),
            constant("num.max_lead_s", 60.0, "s", "Longest lead time after the last usable contour."),
        ],
        forward: ForwardContract {
            entry_point: "ignis_wsp::bass::forecast".into(),
            version: FORWARD_VERSION.into(),
            profile: ForwardProfile::StrictNoFitting,
        },
        notes: Some(
            "Usable contour: status single_valued, the transverse grid of the last admitted contour, \
             unresolved nodes at most enc.max_missing_frac (filled by linear interpolation along s, \
             nearest value at the ends). A contour whose derivation notes say UNCALIBRATED or pixel, \
             or that names no calibration, makes the prefix unusable: the forecast is refused, never \
             rescaled. Spread of the mean front: sqrt(sigma_m^2 + (rel_v |gain_0 v_0| + sigma_v)^2 \
             lead^2), Gaussian. Uncertainty kind: assumed_noise."
                .into(),
        ),
    }
}

pub(crate) struct Weights<'a> {
    gain: &'a [f64],
    persist: &'a [f64],
    damp: &'a [f64],
    sigma: f64,
    rel_v: f64,
    sigma_v: f64,
    window_frac: f64,
    min_frames: usize,
    max_missing: f64,
    max_lead: f64,
}

impl<'a> Weights<'a> {
    pub(crate) fn bind(p: &Params<'a>) -> Self {
        Weights {
            gain: p.v("op.gain"),
            persist: p.v("op.persist_s"),
            damp: p.v("op.damp_per_s"),
            sigma: p.s("unc.sigma_m"),
            rel_v: p.s("unc.rel_v"),
            sigma_v: p.s("unc.sigma_v_m_s"),
            window_frac: p.s("enc.window_frac"),
            min_frames: p.s("enc.min_frames") as usize,
            max_missing: p.s("enc.max_missing_frac"),
            max_lead: p.s("num.max_lead_s"),
        }
    }
}

fn basis(m: usize, xi: f64) -> f64 {
    if m == 0 {
        1.0
    } else {
        std::f64::consts::SQRT_2 * (m as f64 * std::f64::consts::PI * xi).cos()
    }
}

/// Trapezoid integral of `f * phi_m` over `xi`.
fn project(xi: &[f64], f: &[f64], m: usize) -> f64 {
    let mut acc = 0.0;
    for i in 1..xi.len() {
        let (a, b) = (f[i - 1] * basis(m, xi[i - 1]), f[i] * basis(m, xi[i]));
        acc += 0.5 * (a + b) * (xi[i] - xi[i - 1]);
    }
    acc
}

/// A contour with its unresolved nodes filled along `s`, or `None` when too much is missing.
fn filled(c: &ContourObservation, max_missing: f64) -> Option<Vec<f64>> {
    let n = c.f_m.len();
    let known: Vec<usize> = (0..n)
        .filter(|&i| c.f_m[i].is_some_and(f64::is_finite))
        .collect();
    if known.is_empty() || (n - known.len()) as f64 > max_missing * n as f64 {
        return None;
    }
    let value = |i: usize| c.f_m[i].expect("known");
    Some(
        (0..n)
            .map(|i| {
                let hi = known.partition_point(|&k| k < i);
                if hi < known.len() && known[hi] == i {
                    value(i)
                } else if hi == 0 {
                    value(known[0])
                } else if hi == known.len() {
                    value(known[hi - 1])
                } else {
                    let (a, b) = (known[hi - 1], known[hi]);
                    let w = (c.s_m[i] - c.s_m[a]) / (c.s_m[b] - c.s_m[a]);
                    value(a) + w * (value(b) - value(a))
                }
            })
            .collect(),
    )
}

/// Why a contour is not a metric observation, if it is not.
fn not_metric(c: &ContourObservation) -> Option<String> {
    let notes = c
        .derivation
        .notes
        .as_deref()
        .unwrap_or_default()
        .to_lowercase();
    if notes.contains("uncalibrated") || notes.contains("pixel") {
        return Some(format!(
            "contour {} is in pixel coordinates ({})",
            c.id, c.calibration_id
        ));
    }
    if c.calibration_id.trim().is_empty() || c.calibration_id.to_lowercase().contains("pixel") {
        return Some(format!("contour {} names no metric calibration", c.id));
    }
    None
}

/// The numbers of one forecast.
#[derive(Debug, Clone, Serialize)]
pub struct Forecast {
    pub frames_used: usize,
    pub t_last_s: f64,
    pub amplitudes_m: [f64; MODES],
    pub velocities_m_s: [f64; MODES],
    pub s_m: Vec<f64>,
    pub times_s: Vec<f64>,
    /// `[time][s]` front position.
    pub field_m: Vec<Vec<f64>>,
    /// Spatial mean of the front at each time.
    pub mean_m: Vec<f64>,
    pub sd_m: Vec<f64>,
}

fn propagate(wt: &Weights<'_>, a: &[f64; MODES], v: &[f64; MODES], lead: f64) -> [f64; MODES] {
    std::array::from_fn(|m| {
        let t = wt.persist[m];
        a[m] * (-wt.damp[m] * lead).exp() + wt.gain[m] * v[m] * t * -(-lead / t).exp_m1()
    })
}

/// Strict forward of a `bass.wsp` release on an admitted prefix.
pub fn forecast(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<Result<Forecast, Support>, Refusal> {
    let params = Params::bind(spec, &release.release, &release.params)?;
    let wt = Weights::bind(&params);
    let built_with = release
        .release
        .construction
        .as_ref()
        .map(|m| &m.forward_hash);
    let this = crate::audit::forward_hash(Family::Bass);
    if built_with != Some(&this) {
        return Err(Refusal::unsupported(
            "forward_mismatch",
            format!(
                "release {} was assembled with forward {}, this build runs {this}",
                release.release.id,
                built_with.map_or("none".to_string(), |d| d.to_string()),
            ),
        ));
    }
    if matches!(
        release.release.state,
        QualificationState::Failed | QualificationState::Unsupported
    ) {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            format!("release state {:?}", release.release.state),
        )));
    }
    if prefix.family != Family::Bass {
        return Ok(Err(refuse(
            SupportGate::Semantic,
            "prefix is not a BASS run",
        )));
    }
    // the unit guard comes first: a pixel prefix is not a smaller metric prefix
    if let Some(why) = prefix.contours.iter().find_map(not_metric) {
        return Ok(Err(refuse(
            SupportGate::Measurement,
            format!("{why}: pixel-only contours cannot support a metric front forecast"),
        )));
    }
    let tau = prefix.cutoff_s;
    let mut frames: Vec<&ContourObservation> = prefix
        .contours
        .iter()
        .filter(|c| c.time_s <= tau && c.status == ContourStatus::SingleValued && c.s_m.len() >= 3)
        .collect();
    frames.sort_by(|a, b| a.time_s.total_cmp(&b.time_s));
    let Some(last) = frames.last().copied() else {
        return Ok(Err(refuse(
            SupportGate::Measurement,
            "no single-valued contour at or before the cutoff",
        )));
    };
    let s = &last.s_m;
    let span = s[s.len() - 1] - s[0];
    if !(span > 0.0) || s.windows(2).any(|w| !(w[1] > w[0])) {
        return Ok(Err(refuse(
            SupportGate::Measurement,
            "transverse grid is not strictly increasing",
        )));
    }
    let xi: Vec<f64> = s.iter().map(|x| (x - s[0]) / span).collect();
    let same_grid = |c: &ContourObservation| {
        c.s_m.len() == s.len()
            && c.s_m
                .iter()
                .zip(s)
                .all(|(a, b)| (a - b).abs() <= 1e-9 + 1e-6 * b.abs())
    };
    let mut usable: Vec<(f64, [f64; MODES], Vec<f64>)> = Vec::new();
    for c in frames.iter().filter(|c| same_grid(c)) {
        if usable.last().is_some_and(|u| u.0 == c.time_s) {
            continue;
        }
        if let Some(f) = filled(c, wt.max_missing) {
            usable.push((c.time_s, std::array::from_fn(|m| project(&xi, &f, m)), f));
        }
    }
    if usable.len() < wt.min_frames {
        return Ok(Err(refuse(
            SupportGate::Measurement,
            format!(
                "{} usable contours in the prefix (< {})",
                usable.len(),
                wt.min_frames
            ),
        )));
    }
    let (t1, tn) = (usable[0].0, usable[usable.len() - 1].0);
    let width = wt.window_frac * (tn - t1);
    let window = |lo: f64, hi: f64| -> Option<(f64, [f64; MODES])> {
        let sel: Vec<&(f64, [f64; MODES], Vec<f64>)> =
            usable.iter().filter(|u| u.0 >= lo && u.0 <= hi).collect();
        if sel.is_empty() {
            return None;
        }
        let n = sel.len() as f64;
        Some((
            sel.iter().map(|u| u.0).sum::<f64>() / n,
            std::array::from_fn(|m| sel.iter().map(|u| u.1[m]).sum::<f64>() / n),
        ))
    };
    let late = window(tn - width, tn).expect("the last frame is in its own window");
    let early = window(tn - 2.0 * width, tn - width - 1e-12)
        .filter(|e| late.0 > e.0)
        .unwrap_or((t1, usable[0].1));
    if !(late.0 > early.0) {
        return Ok(Err(refuse(
            SupportGate::Measurement,
            "usable contours span no time",
        )));
    }
    let v: [f64; MODES] = std::array::from_fn(|m| (late.1[m] - early.1[m]) / (late.0 - early.0));
    let (_, a, f_last) = &usable[usable.len() - 1];
    // what the K modes do not resolve in the last contour is carried unchanged
    let residual: Vec<f64> = (0..xi.len())
        .map(|i| f_last[i] - (0..MODES).map(|m| a[m] * basis(m, xi[i])).sum::<f64>())
        .collect();
    let times = common::times_of(settings, tau, 10.0, tn + wt.max_lead);
    if times.is_empty() || times.iter().any(|t| !(*t >= tau && *t - tn <= wt.max_lead)) {
        return Ok(Err(refuse(
            SupportGate::Horizon,
            format!(
                "forecast times must lie in [cutoff, last contour + {} s]",
                wt.max_lead
            ),
        )));
    }
    let (mut field, mut mean, mut sd) = (vec![], vec![], vec![]);
    for &t in &times {
        let lead = t - tn;
        let at = propagate(&wt, a, &v, lead);
        let row: Vec<f64> = (0..xi.len())
            .map(|i| residual[i] + (0..MODES).map(|m| at[m] * basis(m, xi[i])).sum::<f64>())
            .collect();
        mean.push(project(&xi, &row, 0));
        let dv = wt.rel_v * (wt.gain[0] * v[0]).abs() + wt.sigma_v;
        sd.push((wt.sigma * wt.sigma + dv * dv * lead * lead).sqrt());
        field.push(row);
    }
    Ok(Ok(Forecast {
        frames_used: usable.len(),
        t_last_s: tn,
        amplitudes_m: *a,
        velocities_m_s: v,
        s_m: s.clone(),
        times_s: times,
        field_m: field,
        mean_m: mean,
        sd_m: sd,
    }))
}

fn body(f: Forecast, settings: &Value) -> Body {
    let levels = common::levels_of(settings);
    let quantiles: Vec<Vec<f64>> = levels
        .iter()
        .map(|p| {
            let z = normal_quantile(*p);
            f.mean_m
                .iter()
                .zip(&f.sd_m)
                .map(|(m, s)| m + z * s)
                .collect()
        })
        .collect();
    let diagnostics = json!({
        "front_mean_m": f.field_m,
        "front_s_m": f.s_m,
        "front_t_s": f.times_s,
        "frames_used": f.frames_used,
        "t_last_contour_s": f.t_last_s,
        "mode_amplitudes_m": f.amplitudes_m,
        "mode_velocities_m_s": f.velocities_m_s,
        "observable": "luminous front position in the calibrated camera plane; not a pyrolysis front, a temperature or a species",
        "uncertainty_kind": "assumed_noise",
    });
    Body {
        kind: None,
        quantity: Some(Quantity::FrontPosition),
        times_s: f.times_s,
        quantile_levels: levels,
        quantiles,
        mean: f.mean_m,
        event_categories: vec![],
        event_cdf: vec![],
        diagnostics,
    }
}

/// The artifact of a forecast or of a scientific refusal (target quantity kept, no numbers).
pub fn artifact(
    spec: &ModelSpec,
    release: &LoadedRelease,
    prefix: &PrefixBundle,
    settings: &Value,
) -> Result<AnalysisArtifact, Refusal> {
    Ok(match forecast(spec, release, prefix, settings)? {
        Ok(f) => common::artifact(release, prefix, settings, admitted(), body(f, settings)),
        Err(support) => common::artifact(
            release,
            prefix,
            settings,
            support,
            Body {
                quantity: Some(Quantity::FrontPosition),
                ..Body::default()
            },
        ),
    })
}

/// `bass.wsp` behind the common expert interface.
pub struct BassWsp {
    spec: ModelSpec,
    policy: ProspectivePolicy,
}

impl Default for BassWsp {
    fn default() -> Self {
        Self::new()
    }
}

impl BassWsp {
    pub fn new() -> Self {
        BassWsp {
            spec: spec(),
            policy: ProspectivePolicy::new(
                Family::Bass,
                &[
                    "x_o2",
                    "pressure",
                    "flow_velocity",
                    "width",
                    "length",
                    "thickness",
                ],
                &[],
            ),
        }
    }
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }
}

impl Expert for BassWsp {
    fn family(&self) -> Family {
        Family::Bass
    }
    fn model_name(&self) -> &str {
        MODEL_NAME
    }
    fn is_baseline(&self) -> bool {
        false
    }
    fn policy(&self) -> &ProspectivePolicy {
        &self.policy
    }
    fn default_config(&self) -> ExpertResult<Value> {
        Ok(json!({"construction": "literal", "spec_id": SPEC_ID}))
    }
    fn estimate_hours(&self, _: &Value, _: usize, _: usize, _: &Profile) -> f64 {
        0.0
    }
    fn fit(&self, _: &FitData<'_>, _: &Value, _: &mut FitContext) -> ExpertResult<ModelRelease> {
        bail!("bass.wsp has no fitting procedure: its release is assembled from a literal patch (`ignis wsp assemble`)")
    }
    fn forecast(
        &self,
        release: &LoadedRelease,
        prefix: &PrefixBundle,
        settings: &Value,
    ) -> ExpertResult<AnalysisArtifact> {
        match forecast(&self.spec, release, prefix, settings)? {
            Ok(f) => Ok(common::artifact(
                release,
                prefix,
                settings,
                admitted(),
                body(f, settings),
            )),
            Err(support) => Err(common::refusal_of(&support).into()),
        }
    }
    fn log_likelihood(&self, _: &LoadedRelease, _: &View) -> ExpertResult<f64> {
        bail!("bass.wsp declares an assumed spread, not a likelihood")
    }
}

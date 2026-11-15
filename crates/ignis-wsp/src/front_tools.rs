//! Everything about `bass_front.wsp` that is not its forward: the synthetic metric fixture, the
//! release record's declarations, and the reader of the front-track files of the measurement study.
//!
//! A track file (`wsp-bass-front-track/1`) holds one burn measured in the top view: tip and base of
//! the flame in sample widths at each still, the fan history read from the video overlay, and the
//! day's fan calibration. It is a lane-local development record, not a shared observation: nothing
//! here writes it into the shared contracts, and its positions are never called metres.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{json, Value};

use ignis_core::contracts::{
    ControlStage, Derivation, EvidenceClass, Family, Measured, Method, Observation, PrefixBundle,
    Quantity, RawValue, Resolution, SourceLocator,
};

use crate::front::FrontInput;

/// What the registry record of a `bass_front.wsp` release declares besides spec, patch and tensors.
pub fn release_notes() -> (BTreeMap<&'static str, Value>, Value, Value) {
    let applicability = BTreeMap::from([
        ("observable", json!("downstream tip of the visible flame along a thin fuel strip, from the upstream end of the sample")),
        ("configuration", json!("concurrent flow, cotton-fibreglass fabric (SIBAL), 1-2 cm wide, low-speed duct")),
        ("time_origin", json!("the prefix's own; the forward shifts it so the igniter is off at zero")),
        ("conditional_on", json!("the flow history up to each forecast time (a known control)")),
        (
            "qualification",
            json!("development only: scored in sample widths on front tracks measured from the public videos; no metric forecast has been scored because no run has a length calibration"),
        ),
        (
            "refuses",
            json!("a prefix without an oxygen fraction, a sample width in metres, an igniter stage or a flow history"),
        ),
    ]);
    let observation_model = json!({
        "prefix": "tip and base positions of the stills at or before the cutoff; flow stages; the igniter stage",
        "units": "the model works in sample widths; metres follow from the width condition of the prefix",
        "not_inferred": "temperature, species, pyrolysis front from intensity",
    });
    let numerics = json!({
        "integrator": "explicit step num.dt_s for the front, exact relaxation of the overhang",
        "request_time_fitting": "none",
    });
    (applicability, observation_model, numerics)
}

/// A deterministic synthetic BASS prefix with METRIC tip and base observations of a 22 mm wide
/// sample: a band moving at `v_m_s` with a constant length, one still per second up to the cutoff,
/// flow stepping from 10 to 3 cm/s at 12 s, igniter on from -6 s to 0. Never NASA data.
pub fn fixture_prefix(cutoff_s: f64, v_m_s: f64) -> PrefixBundle {
    let derivation = || Derivation {
        method: Method::Deterministic,
        method_id: "ignis_wsp.front.fixture_prefix".into(),
        method_version: "1".into(),
        inputs: vec![],
        agent: None,
        notes: Some("synthetic metric front fixture".into()),
    };
    let measured = |v: f64, unit: &str| Measured {
        raw_value: Some(RawValue::Number(v)),
        raw_unit: Some(unit.into()),
        value_si: Some(v),
        unit_si: Some(unit.into()),
        uncertainty_si: None,
        uncertainty_reason: None,
        evidence_class: EvidenceClass::Synthetic,
        locator: None,
        derivation: None,
        resolution: Resolution::Resolved,
        flags: vec![],
    };
    let run_id = "wsp-fixture-bass-front".to_string();
    // the prefix clock starts 10 s before the igniter is switched off
    let t_off = 10.0;
    let n = cutoff_s.floor() as usize;
    let mut observations = Vec::new();
    for i in 1..=n {
        let t = i as f64;
        for (channel, x) in [("tip", 0.020 + v_m_s * t), ("base", 0.004 + v_m_s * t)] {
            observations.push(Observation {
                id: format!("{run_id}:{channel}:{i}"),
                run_id: run_id.clone(),
                quantity: Quantity::FrontPosition,
                channel: Some(channel.into()),
                time_s: t_off + t,
                available_at_s: t_off + t,
                raw_value: Some(RawValue::Number(x * 1e3)),
                raw_unit: Some("mm".into()),
                value_si: Some(x),
                unit_si: "m".into(),
                uncertainty_si: None,
                calibration_id: Some("synthetic-metric".into()),
                quality_flags: vec![],
                evidence_class: EvidenceClass::Synthetic,
                locator: SourceLocator::default(),
                derivation: derivation(),
            });
        }
    }
    let stage = |control: &str, t0: f64, t1: Option<f64>, value: Option<Measured>| ControlStage {
        t_start_s: t0,
        t_end_s: t1,
        control: control.into(),
        value,
        operator_action: false,
    };
    let mut p = PrefixBundle {
        family: Family::Bass,
        run_id,
        session_id: None,
        cutoff_s: t_off + cutoff_s,
        time_origin: "synthetic clock; igniter off at 10 s".into(),
        known_conditions: BTreeMap::from([
            ("x_o2".to_string(), measured(0.19, "1")),
            ("width".to_string(), measured(0.022, "m")),
            ("length".to_string(), measured(0.10, "m")),
        ]),
        known_controls: vec![
            stage("igniter", t_off - 6.0, Some(t_off), None),
            stage(
                "flow_velocity",
                0.0,
                Some(t_off + 12.0),
                Some(measured(0.10, "m/s")),
            ),
            stage(
                "flow_velocity",
                t_off + 12.0,
                None,
                Some(measured(0.03, "m/s")),
            ),
        ],
        observations,
        contours: vec![],
        phase_established_at_s: Some(t_off),
        support_evidence: vec!["synthetic fixture".into()],
        transform_versions: BTreeMap::new(),
        input_hash: String::new(),
    };
    p.input_hash = ignis_core::hashing::content_hash(&p, &["input_hash"]);
    p
}

/// A text field that a track file may give as `null` (the files write `"quality": null` for stills
/// without a flame); `null` and absent both read as the empty string.
fn text_or_null<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(Option::<String>::deserialize(d)?.unwrap_or_default())
}

/// One still of a track file.
#[derive(Debug, Clone, Deserialize)]
pub struct Still {
    pub t_s: f64,
    #[serde(default)]
    pub tip_w: Option<f64>,
    #[serde(default)]
    pub base_w: Option<f64>,
    #[serde(default, deserialize_with = "text_or_null")]
    pub state: String,
    #[serde(default, deserialize_with = "text_or_null")]
    pub quality: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FanEntry {
    pub t_s: f64,
    pub fan: f64,
    #[serde(default)]
    pub u_cm_s: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackEnd {
    #[serde(default, deserialize_with = "text_or_null")]
    pub kind: String,
    #[serde(default)]
    pub t_s: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Igniter {
    #[serde(default)]
    pub on_t_s: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Geometry {
    #[serde(default)]
    pub window_length_w: Option<f64>,
    /// Width of the lit sample band in the top-view stills, pixels of the proof-sheet thumbnail.
    #[serde(default)]
    pub band_width_px: Option<f64>,
    /// Width of the burning sample across the flow, metres, when the record states it (the Saffire
    /// sheets). When present it decides the width ratio, over the band and the label.
    #[serde(default)]
    pub sample_width_m: Option<f64>,
}

/// The fields of a `wsp-bass-front-track/1` file the forward and its scoring read. Unknown fields
/// are ignored: the files carry measurement detail the model has no use for.
#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    pub schema: String,
    pub run_id: String,
    #[serde(default, deserialize_with = "text_or_null")]
    pub partition: String,
    pub position_unit: String,
    pub conditions: Value,
    #[serde(default)]
    pub geometry: Option<Geometry>,
    #[serde(default)]
    pub igniter: Option<Igniter>,
    #[serde(default)]
    pub fan: Vec<FanEntry>,
    #[serde(default)]
    pub fan_calibration: Vec<(f64, f64)>,
    #[serde(default)]
    pub stills: Vec<Still>,
    #[serde(default)]
    pub end: Option<TrackEnd>,
}

/// Declared reading rules of a track file (not model parameters: how a control and a geometry are
/// taken from the record).
pub mod rules {
    /// Fan display at which the fan stalls and the duct flow is zero, used when a day's calibration
    /// block has no zero-flow row (3.8 V in the calibration workbook).
    pub const FAN_ZERO: f64 = 38.0;
    /// Width of the narrow sample over the width of the reference sample (1.2 cm over 2.2 cm, the
    /// widths of the two holders in the experiment's documents).
    pub const NARROW_RATIO: f64 = 1.2 / 2.2;
    /// Exposed length of the sample in reference sample widths (10 cm over 2.2 cm), used when a
    /// file gives no window length.
    pub const WINDOW_REF_W: f64 = 10.0 / 2.2;
    /// Assumed igniter interval when a file does not give it, s.
    pub const IGNITER_S: f64 = 6.0;
    /// Band width in the proof-sheet thumbnails that separates the two sample classes, pixels: the
    /// 2 cm samples measure 52-57 px and the 1 cm samples 30-32 px in every measured test.
    pub const BAND_SPLIT_PX: f64 = 44.0;
    /// Width of the reference sample in metres, the divisor of a width the record states in metres:
    /// the holder width of the 2 cm class, the value the forwards' metric route divides by
    /// (`norm.w_ref_m`), so a track and a shared prefix of the same sheet read the same ratio.
    pub const REF_WIDTH_M: f64 = 0.022;
}

/// Flow speed (cm/s) at a fan display value: the calibrated value of the entry when the file has
/// one, else linear interpolation through the day's calibration rows with a zero-flow row at
/// [`rules::FAN_ZERO`] added when the block has none, extended linearly above the last row.
pub fn flow_speed(calibration: &[(f64, f64)], fan: f64, calibrated: Option<f64>) -> Option<f64> {
    if let Some(u) = calibrated.filter(|u| u.is_finite() && *u >= 0.0) {
        return Some(u);
    }
    let mut rows: Vec<(f64, f64)> = calibration
        .iter()
        .copied()
        .filter(|r| r.0.is_finite() && r.1.is_finite())
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    if rows.is_empty() {
        return None;
    }
    if !rows.iter().any(|r| r.1 == 0.0) && rows[0].0 > rules::FAN_ZERO {
        rows.insert(0, (rules::FAN_ZERO, 0.0));
    }
    // the highest zero-flow row is where the flow starts
    let start = rows.iter().rposition(|r| r.1 == 0.0).unwrap_or(0);
    let rows = &rows[start..];
    if fan <= rows[0].0 {
        return Some(rows[0].1.max(0.0));
    }
    if rows.len() == 1 {
        return None;
    }
    let i = rows
        .partition_point(|r| r.0 <= fan)
        .clamp(1, rows.len() - 1);
    let (a, b) = (rows[i - 1], rows[i]);
    if !(b.0 > a.0) {
        return Some(a.1);
    }
    Some((a.1 + (fan - a.0) * (b.1 - a.1) / (b.0 - a.0)).max(0.0))
}

/// The oxygen mole fraction, width ratio and window length of a track, or why it cannot be read.
pub fn conditions_of(track: &Track) -> Result<(f64, f64, f64), String> {
    let x_o2 = track
        .conditions
        .get("x_o2_initial")
        .and_then(Value::as_f64)
        .ok_or("no x_o2_initial")?;
    let x_o2 = if x_o2 > 1.0 { x_o2 / 100.0 } else { x_o2 };
    let label = track
        .conditions
        .get("sample_label")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_lowercase();
    if !label.contains("sibal") {
        return Err(format!(
            "sample {label:?} is not the cotton-fibreglass fabric"
        ));
    }
    // The width class is what the top view shows when a band was measured (one label of the test
    // matrix says 1 cm over a band as wide as the 2 cm samples); the label decides otherwise.
    let band = track
        .geometry
        .as_ref()
        .and_then(|g| g.band_width_px)
        .filter(|b| b.is_finite() && *b > 0.0);
    let width_m = track
        .geometry
        .as_ref()
        .and_then(|g| g.sample_width_m)
        .filter(|w| w.is_finite() && *w > 0.0);
    let width_ratio = if let Some(w) = width_m {
        w / rules::REF_WIDTH_M
    } else if let Some(b) = band {
        if b < rules::BAND_SPLIT_PX {
            rules::NARROW_RATIO
        } else {
            1.0
        }
    } else if label.trim_start().starts_with("1 cm") || label.trim_start().starts_with("1cm") {
        rules::NARROW_RATIO
    } else if label.trim_start().starts_with("2 cm") || label.trim_start().starts_with("2cm") {
        1.0
    } else {
        return Err(format!(
            "sample width class of {label:?} is not 1 cm or 2 cm"
        ));
    };
    if let Some(flow) = track.conditions.get("flow").and_then(Value::as_str) {
        if !flow.eq_ignore_ascii_case("concurrent") {
            return Err(format!(
                "flow is {flow}, the model is for concurrent spread"
            ));
        }
    }
    let window = track
        .geometry
        .as_ref()
        .and_then(|g| g.window_length_w)
        .filter(|w| w.is_finite() && *w > 0.0)
        .unwrap_or(rules::WINDOW_REF_W / width_ratio);
    Ok((x_o2, width_ratio, window))
}

/// The flow history of a track as a step function in cm/s.
pub fn flow_of(track: &Track) -> Result<Vec<(f64, f64)>, String> {
    let mut out: Vec<(f64, f64)> = Vec::with_capacity(track.fan.len());
    for e in &track.fan {
        let u = flow_speed(&track.fan_calibration, e.fan, e.u_cm_s).ok_or_else(|| {
            format!(
                "no flow speed for fan display {} (no calibration rows)",
                e.fan
            )
        })?;
        if out.last().is_none_or(|l| l.1 != u) {
            out.push((e.t_s, u));
        }
    }
    if out.is_empty() {
        return Err("no fan history".into());
    }
    Ok(out)
}

/// Stills that show a tracked flame after the igniter is off: `(t, tip, base)`, base NaN if absent.
pub fn flame_stills(track: &Track) -> Vec<(f64, f64, f64)> {
    let mut v: Vec<(f64, f64, f64)> = track
        .stills
        .iter()
        .filter(|s| s.state == "flame" && s.quality != "edge" && s.t_s >= 0.0)
        .filter_map(|s| {
            let tip = s.tip_w.filter(|t| t.is_finite())?;
            Some((
                s.t_s,
                tip,
                s.base_w.filter(|b| b.is_finite()).unwrap_or(f64::NAN),
            ))
        })
        .collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v
}

/// The forward input of a track at one cutoff, forecasting at `times` (all after the cutoff).
pub fn input_of(track: &Track, cutoff_s: f64, times: Vec<f64>) -> Result<FrontInput, String> {
    if track.schema != "wsp-bass-front-track/1" {
        return Err(format!("unknown track schema {:?}", track.schema));
    }
    if track.position_unit != "sample_width" {
        return Err(format!(
            "positions are in {}, not sample widths",
            track.position_unit
        ));
    }
    let (x_o2, width_ratio, window) = conditions_of(track)?;
    let stills = flame_stills(track);
    let prefix: Vec<&(f64, f64, f64)> = stills.iter().filter(|s| s.0 <= cutoff_s).collect();
    Ok(FrontInput {
        cutoff_s,
        igniter_on_s: track
            .igniter
            .as_ref()
            .and_then(|i| i.on_t_s)
            .filter(|t| t.is_finite() && *t <= 0.0)
            .unwrap_or(-rules::IGNITER_S),
        x_o2,
        width_ratio,
        window_w: Some(window),
        flow: flow_of(track)?,
        tip: prefix.iter().map(|s| (s.0, s.1)).collect(),
        base: prefix
            .iter()
            .filter(|s| s.2.is_finite())
            .map(|s| (s.0, s.2))
            .collect(),
        times,
    })
}

/// What the registry record of a `bass_front.wsp2` release declares: that of `bass_front.wsp`, with
/// the numerics of the second front forward.
pub fn release_notes2() -> (BTreeMap<&'static str, Value>, Value, Value) {
    let (applicability, observation_model, mut numerics) = release_notes();
    numerics["integrator"] = json!(
        "explicit step num.dt_s for the front, the slow-down downstream evaluated at the start of each step; exact relaxation of the overhang within a step, its time chosen at the start of the step"
    );
    numerics["flow_paths"] = json!(
        "up to three whole runs under shifted and scaled flow histories; quantiles of their mixture by a fixed number of halvings of the monotone mixture CDF"
    );
    (applicability, observation_model, numerics)
}

/// The input of the second front forward: the same burn, field for field.
impl From<FrontInput> for crate::front2::FrontInput {
    fn from(i: FrontInput) -> Self {
        Self {
            cutoff_s: i.cutoff_s,
            igniter_on_s: i.igniter_on_s,
            x_o2: i.x_o2,
            width_ratio: i.width_ratio,
            window_w: i.window_w,
            flow: i.flow,
            tip: i.tip,
            base: i.base,
            times: i.times,
        }
    }
}

/// The input of the second front forward for a track at one cutoff: [`input_of`], unchanged (the
/// same reader and the same reading rules).
pub fn input_of2(
    track: &Track,
    cutoff_s: f64,
    times: Vec<f64>,
) -> Result<crate::front2::FrontInput, String> {
    input_of(track, cutoff_s, times).map(Into::into)
}

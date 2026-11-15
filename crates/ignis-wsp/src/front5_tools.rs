//! Everything about `bass_front.wsp5` that is not its forward: the release record's declarations and
//! the reader that takes a front-track file (`wsp-bass-front-track/1`) to the fifth front forward.
//!
//! The reader of `front_tools` stays as it is, so no existing release reads anything new. This one
//! reads the same fields by the same rules and three more, each optional with a stated default:
//!
//! | Track field | Meaning | Absent |
//! |---|---|---|
//! | `conditions.pressure_atm` | pressure, atm, finite and positive | 1.0 |
//! | `conditions.flow` | `concurrent` or `opposed` (any case) | concurrent |
//! | `conditions.material_index` | 0 SIBAL fabric, 1 PMMA sheet 1 mm | 0, when the sample label is empty or names SIBAL |
//!
//! A field that is present but unusable is a refusal, never the default; a track whose label names
//! another material must state its `material_index`. An opposed-flow track and a PMMA track are read:
//! the forward gives their event probabilities and refuses their tip.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::front5::{Direction, FrontInput, MATERIALS};
use crate::front_tools::{self, rules, Track};

/// What the registry record of a `bass_front.wsp5` release declares besides spec, patch and tensors:
/// that of `bass_front.wsp`, with the boundary law.
pub fn release_notes() -> (BTreeMap<&'static str, Value>, Value, Value) {
    let (mut applicability, mut observation_model, mut numerics) = front_tools::release_notes();
    applicability.insert(
        "configuration",
        json!("tip: concurrent flow over the cotton-fibreglass fabric (SIBAL), 1-2 cm wide, low-speed duct; ignition and quench: the SIBAL fabric and the PMMA sheet of 1 mm, concurrent or opposed flow, pressure in atm"),
    );
    applicability.insert(
        "refuses",
        json!("a prefix without an oxygen fraction, a sample width in metres, an igniter stage or a flow history; a pressure, flow direction or material that is present but unusable; a zero effective flow speed; the tip (not the event probabilities) of an opposed-flow burn or of a material other than the fabric"),
    );
    observation_model["conditions"] = json!(
        "optional: pressure (Pa; 1 atm when absent), flow_direction (0 concurrent, 1 opposed; concurrent when absent), material (0 SIBAL fabric, 1 PMMA sheet 1 mm; 0 when absent)"
    );
    numerics["boundary"] = json!(
        "closed forms of the oxygen limit and its propagated sd; a fixed mixture of 41 equal-probability nodes over the boundary error, the same node at every time; each node's rate of going out integrated exactly on the steps of the flow history"
    );
    (applicability, observation_model, numerics)
}

/// The three conditions the boundary law adds, as a track states them (defaults when absent).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extra {
    pub p_atm: f64,
    pub direction: Direction,
    pub material: usize,
}

/// The added conditions of a track, or why they cannot be read.
pub fn extra_of(track: &Track) -> Result<Extra, String> {
    let c = &track.conditions;
    let p_atm = match c.get("pressure_atm") {
        None | Some(Value::Null) => 1.0,
        Some(v) => {
            let p = v
                .as_f64()
                .ok_or_else(|| format!("pressure_atm {v} is not a number"))?;
            if !(p > 0.0 && p.is_finite()) {
                return Err(format!("pressure_atm {p} is not positive"));
            }
            p
        }
    };
    let direction = match c.get("flow") {
        None | Some(Value::Null) => Direction::Concurrent,
        Some(v) => match v.as_str().map(str::to_ascii_lowercase).as_deref() {
            Some("concurrent") => Direction::Concurrent,
            Some("opposed") => Direction::Opposed,
            _ => return Err(format!("flow {v} is neither concurrent nor opposed")),
        },
    };
    let label = c
        .get("sample_label")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_lowercase();
    let material = match c.get("material_index") {
        None | Some(Value::Null) => {
            if label.trim().is_empty() || label.contains("sibal") {
                0
            } else {
                return Err(format!(
                    "sample {label:?} is not the SIBAL fabric and the track states no material_index"
                ));
            }
        }
        Some(v) => match v.as_u64() {
            Some(m) if (m as usize) < MATERIALS.len() => m as usize,
            _ => {
                return Err(format!(
                    "material_index {v} is neither 0 ({}) nor 1 ({})",
                    MATERIALS[0], MATERIALS[1]
                ))
            }
        },
    };
    Ok(Extra {
        p_atm,
        direction,
        material,
    })
}

/// The oxygen mole fraction, width ratio and window length of a track by the rules of
/// `front_tools::conditions_of`, for any material and either direction.
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
    let window = track
        .geometry
        .as_ref()
        .and_then(|g| g.window_length_w)
        .filter(|w| w.is_finite() && *w > 0.0)
        .unwrap_or(rules::WINDOW_REF_W / width_ratio);
    Ok((x_o2, width_ratio, window))
}

fn igniter_on(track: &Track) -> f64 {
    track
        .igniter
        .as_ref()
        .and_then(|i| i.on_t_s)
        .filter(|t| t.is_finite() && *t <= 0.0)
        .unwrap_or(-rules::IGNITER_S)
}

fn check_schema(track: &Track) -> Result<(), String> {
    if track.schema != "wsp-bass-front-track/1" {
        return Err(format!("unknown track schema {:?}", track.schema));
    }
    if track.position_unit != "sample_width" {
        return Err(format!(
            "positions are in {}, not sample widths",
            track.position_unit
        ));
    }
    Ok(())
}

/// The input of the fifth front forward for a track at one cutoff, forecasting at `times`: the
/// reading of `front_tools::input_of` with the three added conditions.
pub fn input_of(track: &Track, cutoff_s: f64, times: Vec<f64>) -> Result<FrontInput, String> {
    check_schema(track)?;
    let extra = extra_of(track)?;
    let (x_o2, width_ratio, window) = conditions_of(track)?;
    let stills = front_tools::flame_stills(track);
    let prefix: Vec<&(f64, f64, f64)> = stills.iter().filter(|s| s.0 <= cutoff_s).collect();
    Ok(FrontInput {
        cutoff_s,
        igniter_on_s: igniter_on(track),
        x_o2,
        width_ratio,
        window_w: Some(window),
        flow: front_tools::flow_of(track)?,
        tip: prefix.iter().map(|s| (s.0, s.1)).collect(),
        base: prefix
            .iter()
            .filter(|s| s.2.is_finite())
            .map(|s| (s.0, s.2))
            .collect(),
        times,
        p_atm: extra.p_atm,
        direction: extra.direction,
        material: extra.material,
    })
}

/// The input of the ignition forecast of a track: cutoff 0, no stills, one forecast time `t_s`
/// (the conditions alone).
pub fn ignition_input(track: &Track, t_s: f64) -> Result<FrontInput, String> {
    check_schema(track)?;
    let extra = extra_of(track)?;
    let (x_o2, width_ratio, window) = conditions_of(track)?;
    Ok(FrontInput {
        cutoff_s: 0.0,
        igniter_on_s: igniter_on(track),
        x_o2,
        width_ratio,
        window_w: Some(window),
        flow: front_tools::flow_of(track)?,
        tip: vec![],
        base: vec![],
        times: vec![t_s],
        p_atm: extra.p_atm,
        direction: extra.direction,
        material: extra.material,
    })
}

/// The input of `front.rs` for a track, carried to the fifth forward with the track's added
/// conditions (the same burn, field for field, plus three).
pub fn extend(track: &Track, input: crate::front::FrontInput) -> Result<FrontInput, String> {
    let extra = extra_of(track)?;
    let mut i = FrontInput::from_first(input);
    i.p_atm = extra.p_atm;
    i.direction = extra.direction;
    i.material = extra.material;
    Ok(i)
}

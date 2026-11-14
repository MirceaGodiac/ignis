//! Causal prefixes. Information isolation is enforced here, in code, not in prompts.
//!
//! A [`PrefixBundle`] is produced only by [`build_prefix`]. It holds conditions known before the run,
//! observations available at the cutoff, control stages known at the cutoff, and nothing terminal.
//! Families declare which condition keys are prospective; terminal / full-history keys are rejected.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::contracts::{
    ContourObservation, ControlStage, ExperimentRecord, Family, Observation, PrefixBundle,
};
use crate::hashing::content_hash;

/// Names that are terminal or full-history in every family. Families add their own.
pub const ALWAYS_FORBIDDEN: &[&str] = &[
    "k_full",
    "burning_rate_full",
    "final_diameter",
    "extinction_diameter",
    "duration",
    "burn_time",
    "end_time",
    "final_label",
    "endpoint",
    "max_burner_temperature",
    "peak_temperature",
    "post_test_composition",
    "post_test_pressure",
    "outcome",
    "publication_outcome",
];

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("information leak: {0}")]
pub struct LeakError(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectivePolicy {
    pub family: Family,
    pub allowed_conditions: BTreeSet<String>,
    #[serde(default)]
    pub forbidden_conditions: BTreeSet<String>,
}

impl ProspectivePolicy {
    pub fn new(family: Family, allowed: &[&str], forbidden: &[&str]) -> Self {
        ProspectivePolicy {
            family,
            allowed_conditions: allowed.iter().map(|s| s.to_string()).collect(),
            forbidden_conditions: forbidden.iter().map(|s| s.to_string()).collect(),
        }
    }
    fn is_forbidden(&self, key: &str) -> bool {
        ALWAYS_FORBIDDEN.contains(&key) || self.forbidden_conditions.contains(key)
    }
}

/// Structural checks every stored observation must pass.
pub fn validate_observation(o: &Observation) -> Result<(), String> {
    if o.unit_si != o.quantity.unit_si() {
        return Err(format!(
            "{:?} must be stored in {}",
            o.quantity,
            o.quantity.unit_si()
        ));
    }
    if o.available_at_s < o.time_s {
        return Err("a measurement cannot be available before it was taken".into());
    }
    Ok(())
}

pub fn validate_contour(c: &ContourObservation) -> Result<(), String> {
    if c.s_m.len() != c.f_m.len() {
        return Err("s_m and f_m must align".into());
    }
    if c.mask_dependency_frames.iter().any(|&f| f > c.frame) {
        return Err("contour depends on a future frame (non-causal tracking)".into());
    }
    Ok(())
}

#[derive(Default)]
pub struct PrefixInputs<'a> {
    pub contours: &'a [ContourObservation],
    pub known_controls: &'a [ControlStage],
    /// Control names fixed in advance by the protocol (a scripted stop); only these may lie after the cutoff.
    pub protocol_scheduled: &'a [&'a str],
    pub phase_established_at_s: Option<f64>,
    pub support_evidence: Vec<String>,
    pub transform_versions: std::collections::BTreeMap<String, String>,
}

pub fn build_prefix(
    record: &ExperimentRecord,
    observations: &[Observation],
    cutoff_s: f64,
    policy: &ProspectivePolicy,
    time_origin: &str,
    extra: PrefixInputs<'_>,
) -> Result<PrefixBundle, LeakError> {
    if policy.family != record.family {
        return Err(LeakError(format!(
            "policy for {:?} applied to a {:?} run",
            policy.family, record.family
        )));
    }
    let mut conditions = std::collections::BTreeMap::new();
    for (k, v) in &record.conditions {
        if policy.is_forbidden(k) {
            return Err(LeakError(format!(
                "condition {k:?} is terminal/full-history and cannot be prospective"
            )));
        }
        if policy.allowed_conditions.contains(k) {
            conditions.insert(k.clone(), v.clone());
        }
    }
    let mut obs: Vec<Observation> = observations
        .iter()
        .filter(|o| o.run_id == record.id && o.available_at_s <= cutoff_s && o.time_s <= cutoff_s)
        .cloned()
        .collect();
    obs.sort_by(|a, b| a.time_s.total_cmp(&b.time_s).then_with(|| a.id.cmp(&b.id)));
    let cons: Vec<ContourObservation> = extra
        .contours
        .iter()
        .filter(|c| c.run_id == record.id && c.available_at_s <= cutoff_s)
        .cloned()
        .collect();
    if let Some(last) = cons.iter().map(|c| c.frame).max() {
        if let Some(c) = cons
            .iter()
            .find(|c| c.mask_dependency_frames.iter().any(|&f| f > last))
        {
            return Err(LeakError(format!(
                "contour {} was tracked with a frame after the cutoff",
                c.id
            )));
        }
    }
    if matches!(extra.phase_established_at_s, Some(t) if t > cutoff_s) {
        return Err(LeakError(
            "phase establishment must be known at the cutoff".into(),
        ));
    }
    let mut controls = Vec::new();
    for st in extra.known_controls {
        if st.t_start_s > cutoff_s && !extra.protocol_scheduled.contains(&st.control.as_str()) {
            return Err(LeakError(format!(
                "control {:?} at {}s is after the cutoff and not protocol-fixed",
                st.control, st.t_start_s
            )));
        }
        controls.push(st.clone());
    }
    controls.sort_by(|a, b| {
        a.t_start_s
            .total_cmp(&b.t_start_s)
            .then_with(|| a.control.cmp(&b.control))
    });
    let mut bundle = PrefixBundle {
        family: record.family,
        run_id: record.id.clone(),
        session_id: record.session_id.clone(),
        cutoff_s,
        time_origin: time_origin.to_string(),
        known_conditions: conditions,
        known_controls: controls,
        observations: obs,
        contours: cons,
        phase_established_at_s: extra.phase_established_at_s,
        support_evidence: extra.support_evidence,
        transform_versions: extra.transform_versions,
        input_hash: String::new(),
    };
    bundle.input_hash = content_hash(&bundle, &["input_hash"]);
    Ok(bundle)
}

/// Sealed-evaluation check: nothing from the continuation is inside the bundle, nothing is after the
/// cutoff, and the bundle was not modified after construction.
pub fn assert_isolated(
    bundle: &PrefixBundle,
    continuation: &[Observation],
) -> Result<(), LeakError> {
    let inside: BTreeSet<&str> = bundle.observations.iter().map(|o| o.id.as_str()).collect();
    if let Some(o) = continuation.iter().find(|o| inside.contains(o.id.as_str())) {
        return Err(LeakError(format!(
            "continuation observation {} is inside the prefix",
            o.id
        )));
    }
    if let Some(o) = bundle
        .observations
        .iter()
        .find(|o| o.time_s > bundle.cutoff_s || o.available_at_s > bundle.cutoff_s)
    {
        return Err(LeakError(format!(
            "observation {} is after the cutoff",
            o.id
        )));
    }
    if content_hash(bundle, &["input_hash"]) != bundle.input_hash {
        return Err(LeakError(
            "prefix bundle was modified after construction".into(),
        ));
    }
    Ok(())
}

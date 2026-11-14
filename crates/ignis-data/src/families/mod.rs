//! Family ingest registry (FLEX, ACME normal/inverse, BASS).
//!
//! Each entry binds the ingest function, the cohort filter + declared criteria, the prospective
//! policy, the investigations it reads, the grouping keys used by the frozen split and the
//! [`FieldRule`] table of every field it produces.

pub mod acme;
pub mod bass;
pub mod flex;

use ignis_core::contracts::{Family, Method};
use ignis_core::interpret::FieldRule;
use ignis_core::prefix::ProspectivePolicy;

use crate::common::Bundle;

pub type Kept = (Vec<String>, Vec<(String, String)>);

pub struct FamilySpec {
    pub family: Family,
    pub investigations: &'static [&'static str],
    pub ingest: fn(Family) -> anyhow::Result<Bundle>,
    pub cohort_filter: fn(&Bundle) -> Kept,
    pub criteria: &'static [(&'static str, &'static str)],
    pub policy: ProspectivePolicy,
    /// Merged with union-find: runs sharing any key share a group.
    pub group_keys: &'static [&'static str],
    pub rules: fn(Family) -> Vec<FieldRule>,
}

pub const NAMES: [&str; 4] = ["flex", "acme_normal", "acme_inverse", "bass"];

pub fn family_of(name: &str) -> anyhow::Result<Family> {
    Ok(match name {
        "flex" => Family::Flex,
        "acme_normal" => Family::AcmeNormal,
        "acme_inverse" => Family::AcmeInverse,
        "bass" => Family::Bass,
        _ => anyhow::bail!("unknown family {name:?}; expected one of {NAMES:?}"),
    })
}

pub fn get(fam: Family) -> anyhow::Result<FamilySpec> {
    Ok(match fam {
        Family::Flex => FamilySpec {
            family: fam,
            investigations: &["PSI-69"],
            ingest: |_| flex::ingest(),
            cohort_filter: flex::cohort_filter,
            criteria: flex::COHORT_CRITERIA,
            policy: flex::policy(),
            group_keys: &["session"],
            rules: |_| flex::rules(),
        },
        Family::AcmeNormal | Family::AcmeInverse => FamilySpec {
            family: fam,
            investigations: &["PSI-159"],
            ingest: acme::ingest,
            cohort_filter: acme::cohort_filter,
            criteria: acme::COHORT_CRITERIA,
            policy: acme::policy(fam),
            group_keys: &["session"],
            rules: acme::rules,
        },
        Family::Bass => FamilySpec {
            family: fam,
            investigations: &["PSI-25", "PSI-26"],
            ingest: |_| bass::ingest(),
            cohort_filter: bass::cohort_filter,
            criteria: bass::COHORT_CRITERIA,
            policy: bass::policy(),
            group_keys: &["session", "specimen_reuse_group"],
            rules: |_| bass::rules(),
        },
        Family::Other => {
            anyhow::bail!("family 'other' has no NASA ingest (international sources stay separate)")
        }
    })
}

pub(crate) fn r(
    family: &'static str,
    field: &'static str,
    method: Method,
    method_id: &'static str,
    rationale: &'static str,
) -> FieldRule {
    FieldRule {
        family,
        field,
        method,
        method_id,
        version: "1",
        rationale,
    }
}

/// Rules shared by every family for the view fields added in the Rust port.
pub(crate) fn view_rules(family: &'static str) -> Vec<FieldRule> {
    let mut v = vec![r(
        family,
        "view.time_origin",
        Method::Deterministic,
        "views.time_origin",
        "the time origin each view row declares is the one the source data declares for that time base \
         (never an adjudicated phase entry unless review adjudicated one)",
    )];
    if family == "flex" {
        v.push(r(
            family,
            "view.event_before_support_exit",
            Method::HumanReview,
            "review.event_before_support_exit",
            "None unless an adjudication (agent/reviewer) of field 'event_before_support_exit' recorded \
             'yes'/'no' on the run; never inferred from the trajectory",
        ));
    }
    if family == "bass" {
        v.push(r(
            family,
            "view.control_changes_s",
            Method::Deterministic,
            "bass.view.control_changes",
            "flow-control change times known for the run (ControlHistory stages); the master table lists \
             flow settings without times, so this stays empty until the video overlay is adjudicated",
        ));
    }
    v
}

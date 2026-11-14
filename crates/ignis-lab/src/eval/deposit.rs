//! Continuation deposit from a cohort: what `ignis eval deposit-cohort` stores in the vault.
//!
//! A case is (run, cutoff). The payload of a case holds only what lies after that cutoff, so a value
//! a forecast may see is never part of the hidden side. Runs whose phase is not established at the
//! cutoff (for example, already over) are not cases.

use std::collections::BTreeSet;

use ignis_core::analysis::{at_risk, landmarks};
use ignis_core::contracts::{Cohort, Partition, Split};
use ignis_core::views::{measurement_valid, RunArchive, ViewSource};
use serde_json::{json, Value};

use super::vault;
use crate::util::refused;

/// The hidden continuation of one run at one cutoff.
pub fn continuation_payload(a: &RunArchive, cutoff_s: f64) -> Value {
    let observations: Vec<Value> = a
        .observations
        .iter()
        .filter(|o| o.time_s > cutoff_s)
        .map(|o| json!({"id": o.id, "time_s": o.time_s, "value_si": o.value_si, "quantity": o.quantity, "valid": measurement_valid(o)}))
        .collect();
    let mut p = json!({
        "run_id": a.record.id,
        "session_id": a.record.session_id,
        "time_origin": a.time_origin,
        "cutoff_s": cutoff_s,
        "observations": observations,
    });
    if let Some(e) = &a.endpoint {
        p["endpoint"] = json!({"censoring": e.censoring, "t_lower_s": e.last_confirmed_active_s, "t_upper_s": e.first_terminal_s, "allowed_causes": e.allowed_causes});
    }
    let frames: Vec<_> = a.contours.iter().filter(|c| c.time_s > cutoff_s).collect();
    if let Some(first) = frames.first() {
        if frames.iter().all(|c| c.s_m == first.s_m) {
            p["front"] = json!({"t_s": frames.iter().map(|c| c.time_s).collect::<Vec<_>>(), "s_m": first.s_m, "f_m": frames.iter().map(|c| c.f_m.clone()).collect::<Vec<_>>()});
        }
    }
    p
}

/// Deposit every case of a cohort's split partitions. `partition`: `validation`, `test` or `all`.
pub fn deposit_cohort(
    views: &dyn ViewSource,
    cohort_id: &str,
    split_id: &str,
    cutoffs: Option<Vec<f64>>,
    partition: &str,
) -> anyhow::Result<Value> {
    let cohort: Cohort = views.cohort(cohort_id)?;
    let split: Split = views.split(split_id)?;
    if split.cohort_id != cohort.id {
        return Err(refused(format!(
            "split {} belongs to cohort {}, not {}",
            split.id, split.cohort_id, cohort.id
        )));
    }
    let cutoffs = match cutoffs {
        Some(c) => c,
        None => landmarks(cohort.family)?,
    };
    let wanted: Vec<Partition> = match partition {
        "validation" => vec![Partition::Validation],
        "test" => vec![Partition::Test],
        "all" => vec![Partition::Validation, Partition::Test],
        other => {
            return Err(refused(format!(
                "partition {other:?}: validation, test or all"
            )))
        }
    };
    let members: BTreeSet<&str> = cohort.run_ids.iter().map(String::as_str).collect();
    let archives = views.run_archives(cohort.family)?;
    let (mut written, mut not_at_risk, mut per_partition) = (
        0usize,
        0usize,
        std::collections::BTreeMap::<String, usize>::new(),
    );
    for a in archives
        .iter()
        .filter(|a| members.contains(a.record.id.as_str()))
    {
        let Some(part) = split
            .partition_of(&a.record.id)
            .filter(|p| wanted.contains(p))
        else {
            continue;
        };
        for &cutoff in &cutoffs {
            if !at_risk(a, cutoff) {
                not_at_risk += 1;
                continue;
            }
            vault::deposit_landmark(
                &split,
                &a.record.id,
                cutoff,
                continuation_payload(a, cutoff),
            )?;
            written += 1;
            *per_partition
                .entry(format!("{part:?}").to_lowercase())
                .or_default() += 1;
        }
    }
    Ok(
        json!({"cohort": cohort.id, "split": split.id, "family": cohort.family, "cutoffs_s": cutoffs, "deposited_cases": written,
              "not_at_risk_cases": not_at_risk, "per_partition": per_partition}),
    )
}

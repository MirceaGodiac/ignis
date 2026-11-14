//! Semantic review queue: export items, import adjudications, apply them to a bundle.
//!
//! Format (`data/review/<family>/`):
//!
//! * `items.jsonl` — one object per question: `id, family, run_id, field, question, cited_text,
//!   locator{asset_sha256, path, row, column, char_start, char_end}, candidates[], deterministic_screen, context`.
//! * `adjudications.jsonl` — one object per answer: `item_id, value, resolution
//!   (resolved|unresolved_conflict|unknown), agent | reviewer (at least one), rationale, adjudicated_at,
//!   evidence[] (optional extra locators)`. `agent` = a model/agent session (recorded as SEMANTIC),
//!   `reviewer` = a named person (recorded as HUMAN_REVIEW).
//!
//! Nothing here calls a model (`crate::auto` does, and writes the same format). Until an adjudication
//! exists the value stays unresolved/unknown. Fields applied: `free_floating` and
//! `event_before_support_exit` (FLEX record flags), `flex_endpoint` (FLEX endpoint + view fields),
//! `endpoint_cause` and `comment_semantics` (ACME endpoint, flags, cohort), `video_run` (BASS video map).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{Censoring, Derivation, Method, Resolution, SourceLocator};
use serde_json::{json, Value};

use crate::common::{read_jsonl_values, write_jsonl, Bundle};
use crate::paths;

const REQUIRED: [&str; 3] = ["item_id", "value", "resolution"];

pub fn export(b: &Bundle) -> anyhow::Result<PathBuf> {
    let p = paths::review(&b.family).join("items.jsonl");
    let mut items: Vec<_> = b.review.iter().collect();
    items.sort_by(|a, b| a.id.cmp(&b.id));
    write_jsonl(&p, items.iter().map(|r| r.to_json()))?;
    Ok(p)
}

fn parse_resolution(v: &Value) -> anyhow::Result<Resolution> {
    Ok(serde_json::from_value(v.clone())?)
}

fn truthy(v: &Value) -> bool {
    !(v.is_null() || v.as_str() == Some(""))
}

pub fn validate(adj: &Value, item_ids: &HashSet<String>) -> anyhow::Result<()> {
    let missing: Vec<&str> = REQUIRED
        .iter()
        .copied()
        .filter(|k| adj.get(k).is_none())
        .collect();
    anyhow::ensure!(
        missing.is_empty(),
        "adjudication missing {missing:?}: {adj}"
    );
    let id = adj["item_id"].as_str().unwrap_or_default();
    anyhow::ensure!(
        adj.get("agent").is_some_and(truthy) || adj.get("reviewer").is_some_and(truthy),
        "adjudication {id} names neither agent nor reviewer"
    );
    anyhow::ensure!(item_ids.contains(id), "adjudication for unknown item {id}");
    parse_resolution(&adj["resolution"])?;
    Ok(())
}

pub fn import_file(family: &str, src: &Path) -> anyhow::Result<usize> {
    let items: HashSet<String> = read_jsonl_values(&paths::review(family).join("items.jsonl"))?
        .iter()
        .filter_map(|r| r["id"].as_str().map(String::from))
        .collect();
    let mut new = read_jsonl_values(src)?;
    for a in &mut new {
        validate(a, &items)?;
        if a.get("adjudicated_at").is_none() {
            a["adjudicated_at"] =
                json!(chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false));
        }
    }
    let dest = paths::review(family).join("adjudications.jsonl");
    let mut all = read_jsonl_values(&dest)?;
    let n = new.len();
    all.extend(new);
    write_jsonl(&dest, &all)?;
    Ok(n)
}

fn s(a: &Value, k: &str) -> String {
    match a.get(k) {
        Some(Value::String(x)) => x.clone(),
        Some(Value::Null) | None => String::new(),
        Some(v) => v.to_string(),
    }
}

fn derivation(a: &Value) -> Derivation {
    let reviewer = a.get("reviewer").filter(|v| truthy(v));
    Derivation {
        method: if reviewer.is_some() {
            Method::HumanReview
        } else {
            Method::Semantic
        },
        method_id: "review.adjudication".into(),
        method_version: "1".into(),
        inputs: vec![],
        agent: Some(if reviewer.is_some() {
            s(a, "reviewer")
        } else {
            s(a, "agent")
        }),
        notes: Some(format!(
            "{} -> {} ({}): {}",
            s(a, "item_id"),
            s(a, "value"),
            s(a, "resolution"),
            s(a, "rationale")
        )),
    }
}

/// Source locators an adjudication cites (`evidence`), entries that are not locators are ignored.
fn locators(a: &Value) -> Vec<SourceLocator> {
    a["evidence"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|l| serde_json::from_value(l.clone()).ok())
        .collect()
}

fn parse_censoring(v: &Value) -> Censoring {
    serde_json::from_value(v.clone()).unwrap_or(Censoring::Unknown)
}

fn opt_time(v: &Value) -> Option<f64> {
    v.as_f64().filter(|t| t.is_finite())
}

/// Extra cohort criteria, present only when adjudications actually applied them, so a family without
/// these adjudications keeps its cohort id.
pub fn adjudication_criteria(b: &Bundle) -> Vec<(String, String)> {
    let flagged = |p: &str| {
        b.records
            .iter()
            .any(|r| r.quality_flags.iter().any(|f| f.starts_with(p)))
    };
    let mut v = vec![];
    if flagged("flex_endpoint:") {
        v.push(("endpoint_adjudication".to_string(),
            "flex_endpoint adjudicated by two independent LLM passes; a run whose passes disagree or fail validation is excluded".to_string()));
    }
    if flagged("comment_semantics:") {
        v.push(("comment_adjudication".to_string(),
            "comment_semantics: non_spherical_or_irregular, other and unresolved are excluded; intervention on an SE row \
             becomes an unresolved endpoint conflict".to_string()));
    }
    v
}

fn set_flag(flags: &mut Vec<String>, prefix: &str, value: &str) {
    flags.retain(|f| !f.starts_with(prefix));
    flags.push(format!("{prefix}{value}"));
}

/// Apply the latest adjudication per item (later lines win). Returns how many were applied.
pub fn apply(b: &mut Bundle) -> anyhow::Result<usize> {
    let mut adjs: indexmap::IndexMap<String, Value> = indexmap::IndexMap::new();
    for a in read_jsonl_values(&paths::review(&b.family).join("adjudications.jsonl"))? {
        adjs.insert(s(&a, "item_id"), a);
    }
    if adjs.is_empty() {
        return Ok(0);
    }
    let items: HashMap<String, (String, String)> = b
        .review
        .iter()
        .map(|r| (r.id.clone(), (r.field.clone(), r.run_id.clone())))
        .collect();
    let recs: HashMap<String, usize> = b
        .records
        .iter()
        .enumerate()
        .map(|(i, r)| (r.id.clone(), i))
        .collect();
    let eps: HashMap<String, usize> = b
        .endpoints
        .iter()
        .enumerate()
        .map(|(i, e)| (e.run_id.clone(), i))
        .collect();
    let specimens: Vec<Option<String>> = b.records.iter().map(|r| r.specimen_id.clone()).collect();
    let mut n = 0;
    for (item_id, a) in &adjs {
        let Some((field, run_id)) = items.get(item_id) else {
            continue;
        };
        let res = parse_resolution(&a["resolution"])?;
        let der = derivation(a);
        let value = s(a, "value");
        match field.as_str() {
            "free_floating" | "event_before_support_exit" if recs.contains_key(run_id) => {
                let r = &mut b.records[recs[run_id]];
                let val = if res != Resolution::Resolved {
                    "unresolved"
                } else if field == "free_floating" {
                    match value.as_str() {
                        "free_floating" => "yes",
                        "fiber_supported" => "no",
                        _ => "unresolved",
                    }
                } else {
                    match value.as_str() {
                        "yes" | "true" => "yes",
                        "no" | "false" => "no",
                        _ => "unresolved",
                    }
                };
                set_flag(&mut r.quality_flags, &format!("{field}:"), val);
                r.identity_evidence.push(der);
                n += 1;
            }
            "endpoint_cause" if eps.contains_key(run_id) => {
                let e = &mut b.endpoints[eps[run_id]];
                let causes: Vec<String> = match value.as_str() {
                    "extinction" => vec!["extinction".into()],
                    "operator_termination" => vec!["operator_termination".into()],
                    "both_possible" => vec!["extinction".into(), "operator_termination".into()],
                    _ => vec![],
                };
                e.evidence.extend(locators(a));
                e.derivation = der;
                if res != Resolution::Resolved {
                    // the ingest state (causes, bounds) is kept; only the status changes
                    e.resolution = res;
                } else if causes.is_empty() {
                    e.resolution = Resolution::Unknown;
                } else {
                    e.resolution = res;
                    if value == "operator_termination" {
                        e.intervention_s = e.first_terminal_s;
                        e.intervention_kind = Some("flow_termination".into());
                    }
                    e.allowed_causes = causes;
                }
                n += 1;
            }
            "flex_endpoint" if eps.contains_key(run_id) && recs.contains_key(run_id) => {
                let answer = &a["answer"];
                let e = &mut b.endpoints[eps[run_id]];
                let r = &mut b.records[recs[run_id]];
                let mut der = der;
                if let Some(note) = answer["time_origin_note"]
                    .as_str()
                    .filter(|n| !n.is_empty())
                {
                    der.notes = der.notes.map(|x| format!("{x}; time origin: {note}"));
                }
                e.evidence.extend(locators(a));
                e.derivation = der;
                e.resolution = res;
                let status = match res {
                    Resolution::Resolved => {
                        let causes: Vec<String> = match value.as_str() {
                            "hot_phase_end" | "disruption" => vec![value.clone()],
                            _ => ignis_core::views::FLEX_CAUSES
                                .iter()
                                .map(|c| c.to_string())
                                .collect(),
                        };
                        e.allowed_causes = causes;
                        e.censoring = parse_censoring(&answer["censoring"]);
                        e.last_confirmed_active_s = opt_time(&answer["last_confirmed_active_s"]);
                        e.first_terminal_s = opt_time(&answer["first_terminal_s"]);
                        "resolved"
                    }
                    Resolution::UnresolvedConflict => "unresolved",
                    _ => "unknown",
                };
                set_flag(&mut r.quality_flags, "flex_endpoint:", status);
                let before = if res == Resolution::Resolved {
                    match answer["event_before_support_exit"].as_str() {
                        Some("yes") => "yes",
                        Some("no") => "no",
                        _ => "unresolved",
                    }
                } else {
                    "unresolved"
                };
                set_flag(&mut r.quality_flags, "event_before_support_exit:", before);
                n += 1;
            }
            "comment_semantics" if recs.contains_key(run_id) => {
                let r = &mut b.records[recs[run_id]];
                let val = if res == Resolution::Resolved {
                    value.as_str()
                } else {
                    "unresolved"
                };
                set_flag(&mut r.quality_flags, "comment_semantics:", val);
                r.identity_evidence.push(der.clone());
                if let Some(&ei) = eps.get(run_id) {
                    let e = &mut b.endpoints[ei];
                    e.evidence.extend(locators(a));
                    if res == Resolution::Resolved {
                        match value.as_str() {
                            "intervention" => {
                                e.intervention_kind
                                    .get_or_insert_with(|| "documented_in_comment".into());
                                // An SE row whose comment documents an intervention is the same
                                // contradiction as the ingest conflict: both causes stay open.
                                if e.allowed_causes == ["extinction".to_string()] {
                                    e.allowed_causes.push("operator_termination".into());
                                    e.resolution = Resolution::UnresolvedConflict;
                                    e.derivation = der;
                                }
                            }
                            "data_loss" => {
                                e.observation_state = Some("data_loss_documented_in_comment".into())
                            }
                            "non_spherical_or_irregular" => {
                                e.observation_state =
                                    Some("non_spherical_or_irregular_flame".into())
                            }
                            _ => {}
                        }
                    }
                }
                n += 1;
            }
            "video_run" => {
                let res_s = crate::families::flex::res_str(res);
                for v in b.extra.get_mut("video_map").into_iter().flatten() {
                    if format!("video:{}", v["file_name"].as_str().unwrap_or("")) == *item_id {
                        let is_run = res == Resolution::Resolved
                            && !matches!(
                                value.as_str(),
                                "not_a_test_recording" | "unclear" | "unknown"
                            );
                        v["run_id"] = if is_run { json!(value) } else { Value::Null };
                        v["specimen_id"] = recs
                            .get(&value)
                            .filter(|_| is_run)
                            .and_then(|i| specimens[*i].clone())
                            .map_or(Value::Null, Value::String);
                        v["resolution"] = json!(res_s);
                        v["evidence"] = json!(format!(
                            "{}; adjudicated by {}",
                            s(v, "evidence"),
                            der.agent.clone().unwrap_or_default()
                        ));
                        v["method_id"] = json!("review.adjudication");
                        n += 1;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(n)
}

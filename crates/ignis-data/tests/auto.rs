//! Automatic adjudication with a fake `claude` executable (`IGNIS_CLAUDE_BIN`).

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use ignis_core::contracts::{
    Censoring, Derivation, EndpointRecord, ExperimentRecord, Family, Method, Quantity, Resolution,
};
use ignis_data::auto::{self, AutoConfig};
use ignis_data::common::{det, loc, text_value, Bundle, ReviewItem};
use ignis_data::dossier::Dossier;
use ignis_data::families::flex;
use ignis_data::review;
use serde_json::{json, Value};

static LOCK: Mutex<()> = Mutex::new(());

struct Env {
    _g: MutexGuard<'static, ()>,
    dir: PathBuf,
}

/// One temp root per test; the environment is process-global, hence the lock.
fn env(name: &str) -> Env {
    let g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join(format!("ignis-auto-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::env::set_var("IGNIS_DATA", &dir);
    std::env::set_var("IGNIS_FAKE_DIR", &dir);
    std::env::set_var("IGNIS_CLAUDE_BIN", env!("CARGO_BIN_EXE_ignis-fake-claude"));
    std::env::set_var("IGNIS_AUTO_BACKOFF_MS", "1");
    Env { _g: g, dir }
}

fn dossier(id: &str) -> Dossier {
    let mut d = Dossier::new(&ReviewItem {
        id: id.into(),
        family: "acme_normal".into(),
        run_id: "run".into(),
        field: "comment_semantics".into(),
        question: "q".into(),
        cited_text: "alpha beta".into(),
        locator: loc("sha", "table.csv", Some(4), Some("Comments")),
        candidates: vec!["no_effect".into(), "other".into()],
        deterministic_screen: None,
        context: Default::default(),
    });
    d.add(
        "alpha beta gamma",
        Some(loc("sha", "table.csv", Some(4), Some("Comments"))),
    );
    d
}

fn cfg() -> AutoConfig {
    AutoConfig {
        family: "acme_normal".into(),
        shard: (1, 1),
        model: "opus".into(),
        passes: 2,
        concurrency: 2,
        limit: None,
        only: None,
        execute: true,
        dump_dossiers: None,
    }
}

fn out(e: &Env) -> PathBuf {
    e.dir.join("drafts/auto-1of1.jsonl")
}

fn lines(p: &PathBuf) -> Vec<Value> {
    ignis_data::common::read_jsonl_values(p).unwrap()
}

fn calls(e: &Env) -> usize {
    std::fs::read_to_string(e.dir.join("calls.log"))
        .unwrap_or_default()
        .lines()
        .count()
}

#[test]
fn agreement_is_resolved_with_agent_label_and_locators() {
    let e = env("agree");
    let s = auto::execute(&[dossier("item#agree")], &cfg(), &out(&e)).unwrap();
    assert_eq!(s.processed, 1);
    assert_eq!(s.usage["calls"], 2);
    let v = &lines(&out(&e))[0];
    assert_eq!(v["resolution"], "resolved");
    assert_eq!(v["value"], "no_effect");
    assert_eq!(v["agent"], "claude-code:opus:2");
    assert!(v.get("reviewer").is_none());
    assert_eq!(v["evidence"][0]["path"], "table.csv");
    assert_eq!(v["passes"].as_array().unwrap().len(), 2);
}

#[test]
fn disagreement_stays_unresolved_with_both_answers() {
    let e = env("disagree");
    auto::execute(&[dossier("item#disagree")], &cfg(), &out(&e)).unwrap();
    let v = &lines(&out(&e))[0];
    assert_eq!(v["resolution"], "unresolved_conflict");
    assert_eq!(v["value"], "unknown");
    assert_eq!(v["passes"][0]["answer"]["value"], "no_effect");
    assert_eq!(v["passes"][1]["answer"]["value"], "other");
}

#[test]
fn fabricated_locator_is_rejected_after_one_retry() {
    let e = env("fabricate");
    auto::execute(
        &[dossier("item#fabricate"), dossier("item#recover")],
        &cfg(),
        &out(&e),
    )
    .unwrap();
    let rows = lines(&out(&e));
    let by = |id: &str| rows.iter().find(|r| r["item_id"] == id).unwrap();
    let bad = by("item#fabricate");
    assert_eq!(bad["resolution"], "unresolved_conflict");
    assert_eq!(bad["passes"][0]["valid"], false);
    assert_eq!(bad["passes"][0]["attempts"], 2);
    assert!(bad["passes"][0]["errors"][0]
        .as_str()
        .unwrap()
        .contains("L9999"));
    let good = by("item#recover");
    assert_eq!(good["resolution"], "resolved");
    assert_eq!(good["passes"][0]["attempts"], 2);
}

#[test]
fn validator_checks_quotes_and_flex_times() {
    let _e = env("validator");
    let mut d = dossier("v");
    assert!(auto::validate_answer(
        &d,
        &json!({"value": "other", "evidence": [{"id": "L0001", "quote": "not in the line"}]})
    )[0]
    .contains("verbatim"));
    assert!(
        auto::validate_answer(&d, &json!({"value": "other", "evidence": []}))[0].contains("cite")
    );
    d.field = "flex_endpoint".into();
    d.candidates = vec![
        "hot_phase_end".into(),
        "disruption".into(),
        "unknown".into(),
    ];
    d.observed_range = Some((0.0, 10.0));
    let ans = |cause: &str, censoring: &str, last: Value, first: Value| {
        json!({"value": cause, "censoring": censoring, "last_confirmed_active_s": last, "first_terminal_s": first, "event_before_support_exit": "yes",
               "time_origin_note": "DropletData", "evidence": [{"id": "L0001", "quote": "alpha beta"}]})
    };
    assert!(auto::validate_answer(
        &d,
        &ans("hot_phase_end", "interval", json!(3.0), json!(4.0))
    )
    .is_empty());
    assert!(!auto::validate_answer(
        &d,
        &ans("hot_phase_end", "interval", json!(4.0), json!(3.0))
    )
    .is_empty());
    assert!(!auto::validate_answer(
        &d,
        &ans("hot_phase_end", "interval", json!(3.0), json!(40.0))
    )
    .is_empty());
    assert!(
        !auto::validate_answer(&d, &ans("hot_phase_end", "right", json!(3.0), Value::Null))
            .is_empty()
    );
    assert!(
        auto::validate_answer(&d, &ans("unknown", "right", json!(3.0), Value::Null)).is_empty()
    );
    assert!(
        !auto::validate_answer(&d, &ans("unknown", "event", json!(3.0), json!(3.0))).is_empty()
    );
}

#[test]
fn resume_skips_done_items_and_rate_limit_stops_cleanly() {
    let e = env("resume");
    let items = [dossier("item#agree-1"), dossier("item#agree-2")];
    auto::execute(&items, &cfg(), &out(&e)).unwrap();
    assert_eq!(calls(&e), 4);
    auto::execute(&items, &cfg(), &out(&e)).unwrap();
    assert_eq!(calls(&e), 4, "finished items must not call the model again");
    let s = auto::execute(&[dossier("item#ratelimit")], &cfg(), &out(&e)).unwrap();
    assert_eq!(s.processed, 0);
    assert_eq!(s.remaining, 1);
    assert_eq!(s.stopped.as_ref().unwrap().0, "rate_limit");
    assert_eq!(lines(&out(&e)).len(), 2);
}

#[test]
fn shard_partition_is_stable_and_covers_every_item_once() {
    let ids: Vec<String> = (0..500).map(|i| format!("item-{i}")).collect();
    for n in [1, 2, 4, 7] {
        let mut seen = vec![0usize; n + 1];
        for id in &ids {
            let s = auto::shard_of(id, n);
            assert!((1..=n).contains(&s));
            assert_eq!(s, auto::shard_of(id, n));
            seen[s] += 1;
        }
        assert_eq!(seen.iter().sum::<usize>(), ids.len());
        assert!(seen[1..].iter().all(|c| *c > 0));
    }
}

fn record(id: &str, family: Family) -> ExperimentRecord {
    let mut conditions = std::collections::BTreeMap::new();
    conditions.insert(
        "fuel".to_string(),
        text_value("Methanol", None, "t", None, None, &[]),
    );
    ExperimentRecord {
        id: id.into(),
        investigation_id: "X".into(),
        family,
        session_id: None,
        specimen_id: None,
        aliases: vec![],
        fuel_or_material: None,
        configuration: None,
        gravity: None,
        conditions,
        identity_resolution: Resolution::Resolved,
        identity_evidence: vec![],
        quality_flags: vec!["free_floating:yes".into()],
        notes: None,
    }
}

fn endpoint(id: &str, causes: &[&str]) -> EndpointRecord {
    EndpointRecord {
        run_id: id.into(),
        original_label: None,
        original_comment: None,
        allowed_causes: causes.iter().map(|s| s.to_string()).collect(),
        censoring: Censoring::Unknown,
        last_confirmed_active_s: None,
        first_terminal_s: None,
        intervention_s: None,
        intervention_kind: None,
        observation_state: None,
        resolution: Resolution::Unknown,
        evidence: vec![],
        derivation: det("d", "1", vec![], None),
    }
}

fn item(run: &str, field: &str) -> ReviewItem {
    ReviewItem {
        id: format!("{run}#{field}"),
        family: "x".into(),
        run_id: run.into(),
        field: field.into(),
        question: "q".into(),
        cited_text: "c".into(),
        locator: loc("s", "p", None, None),
        candidates: vec![],
        deterministic_screen: None,
        context: Default::default(),
    }
}

fn write_adjudications(e: &Env, family: &str, rows: &[Value]) {
    let dir = e.dir.join("review").join(family);
    std::fs::create_dir_all(&dir).unwrap();
    let text: Vec<String> = rows.iter().map(|r| r.to_string()).collect();
    std::fs::write(dir.join("adjudications.jsonl"), text.join("\n") + "\n").unwrap();
}

fn semantic(der: &Derivation) -> bool {
    der.method == Method::Semantic && der.agent.as_deref() == Some("claude-code:opus:2")
}

#[test]
fn flex_endpoint_applies_to_endpoint_flags_and_cohort() {
    let e = env("apply-flex");
    let mut b = Bundle::new("flex");
    for run in ["R1", "R2"] {
        b.records.push(record(run, Family::Flex));
        b.endpoints.push(endpoint(run, &["hot_phase_end"]));
        b.review.push(item(run, "flex_endpoint"));
        let header = vec!["t".to_string(), "d".to_string()];
        let rows: Vec<flex::SeriesRow> = (0..12)
            .map(|i| (i + 2, vec![Some(i as f64 * 0.1), Some(2.0)]))
            .collect();
        b.observations.extend(flex::series_observations(
            run,
            "sha",
            "f",
            &header,
            &rows,
            "t",
            "d",
            Quantity::LiquidDiameter,
            "mm",
            "DropletData",
        ));
    }
    let (keep, _) = flex::cohort_filter(&b);
    assert_eq!(
        keep.len(),
        2,
        "without adjudications the cohort does not change"
    );
    assert!(review::adjudication_criteria(&b).is_empty());
    write_adjudications(
        &e,
        "flex",
        &[
            json!({"item_id": "R1#flex_endpoint", "value": "disruption", "resolution": "resolved", "agent": "claude-code:opus:2", "rationale": "r",
               "answer": {"censoring": "interval", "last_confirmed_active_s": 3.5, "first_terminal_s": 4.0, "event_before_support_exit": "yes", "time_origin_note": "DropletData"},
               "evidence": [loc("sha", "sheet.txt", Some(9), None)]}),
            json!({"item_id": "R2#flex_endpoint", "value": "unknown", "resolution": "unresolved_conflict", "agent": "claude-code:opus:2", "rationale": "r", "answer": null}),
        ],
    );
    assert_eq!(review::apply(&mut b).unwrap(), 2);
    let ep = &b.endpoints[0];
    assert_eq!(ep.censoring, Censoring::Interval);
    assert_eq!(
        (ep.last_confirmed_active_s, ep.first_terminal_s),
        (Some(3.5), Some(4.0))
    );
    assert_eq!(ep.allowed_causes, ["disruption"]);
    assert_eq!(ep.resolution, Resolution::Resolved);
    assert_eq!(ep.evidence[0].path.as_deref(), Some("sheet.txt"));
    assert!(semantic(&ep.derivation));
    assert!(b.records[0]
        .quality_flags
        .iter()
        .any(|f| f == "event_before_support_exit:yes"));
    assert_eq!(b.endpoints[1].resolution, Resolution::UnresolvedConflict);
    let (keep, out) = flex::cohort_filter(&b);
    assert_eq!(keep, ["R1"]);
    assert!(out[0].1.contains("endpoint unresolved"));
    assert_eq!(
        review::adjudication_criteria(&b).len(),
        1,
        "the cohort records the new criterion"
    );
}

#[test]
fn comment_semantics_changes_flags_and_endpoint() {
    let e = env("apply-acme");
    let mut b = Bundle::new("acme_normal");
    for run in ["A", "B", "C"] {
        b.records.push(record(run, Family::AcmeNormal));
        let mut ep = endpoint(run, &["extinction"]);
        ep.resolution = Resolution::Resolved;
        b.endpoints.push(ep);
        b.review.push(item(run, "comment_semantics"));
    }
    write_adjudications(
        &e,
        "acme_normal",
        &[
            json!({"item_id": "A#comment_semantics", "value": "intervention", "resolution": "resolved", "agent": "claude-code:opus:2", "rationale": "r"}),
            json!({"item_id": "B#comment_semantics", "value": "non_spherical_or_irregular", "resolution": "resolved", "agent": "claude-code:opus:2", "rationale": "r"}),
            json!({"item_id": "C#comment_semantics", "value": "unknown", "resolution": "unresolved_conflict", "agent": "claude-code:opus:2", "rationale": "r"}),
        ],
    );
    assert_eq!(review::apply(&mut b).unwrap(), 3);
    assert_eq!(b.endpoints[0].resolution, Resolution::UnresolvedConflict);
    assert_eq!(
        b.endpoints[0].allowed_causes,
        ["extinction", "operator_termination"]
    );
    assert_eq!(
        b.endpoints[0].intervention_kind.as_deref(),
        Some("documented_in_comment")
    );
    assert!(semantic(&b.endpoints[0].derivation));
    assert_eq!(
        b.endpoints[1].resolution,
        Resolution::Resolved,
        "a non-spherical flame leaves the population, not the endpoint"
    );
    assert!(b.records[1]
        .quality_flags
        .iter()
        .any(|f| f == "comment_semantics:non_spherical_or_irregular"));
    assert!(b.records[2]
        .quality_flags
        .iter()
        .any(|f| f == "comment_semantics:unresolved"));
}

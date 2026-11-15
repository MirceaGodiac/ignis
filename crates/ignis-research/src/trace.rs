//! Per-episode traces (task M10): the `TraceBundle` of an episode, content-addressed and
//! replayable, built from what IGNIS already keeps (the event log, the artifacts it references,
//! the harness pin and the measured usage). Benchmark scorecards read traces; they never re-run an
//! episode to learn what it did or cost.
//!
//! - **Content-addressed.** Every piece is an artifact of the content-addressed store (the event
//!   log as JSONL, each tool call's arguments and `ToolResult`), and `trace_id` is derived from the
//!   bundle's content (never the time of export): exporting the same log twice gives the same
//!   bundle, byte for byte.
//! - **Replayable.** [`replay_events`] reads the bundle's event log back, checks it
//!   (`check_event_log`, the bundle's range), and returns the events; importing them into an empty
//!   store reproduces the episode's log, and [`crate::metrics`] reproduces its numbers.
//! - **Honest.** Usage and cost come from [`crate::metrics`] (measured tokens × the versioned price
//!   list); a cost that cannot be established is absent, never a guess. `partitions_touched` is the
//!   set of partitions the episode's scope granted: an upper bound on what its calls read (the log
//!   does not record per-call partitions yet), which is the safe side for a proposer that must
//!   refuse traces that touched `test`.

use ignis_core::contracts::{
    check_event_log, ArtifactRef, EpisodeId, ErrorCode, ErrorEnvelope, EventBody, EventRange,
    ExecutionScope, Finite, HarnessRun, Producer, ResearchEvent, ResearchSchema, RetryPolicy,
    TraceBundle, TraceId, TraceToolCall,
};
use ignis_core::hashing::content_hash;
use ignis_core::services::{ArtifactDraft, ArtifactStore, ServiceResult};

use crate::metrics::{self, PriceList};
use crate::ResearchStore;

/// Media type of the traced event log (one `ResearchEvent` JSON per line).
pub const EVENT_LOG_MEDIA_TYPE: &str = "application/x-ndjson";

fn refused(msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(ErrorCode::InvalidRequest, msg, RetryPolicy::Never)
}

fn put_json(
    artifacts: &dyn ArtifactStore,
    scope: &ExecutionScope,
    produced_by: Producer,
    contract: Option<&str>,
    media_type: &str,
    bytes: &[u8],
    origin: ignis_core::contracts::Origin,
) -> ServiceResult<ArtifactRef> {
    artifacts.put(
        scope,
        ArtifactDraft {
            media_type: media_type.into(),
            contract: contract.map(str::to_string),
            produced_by,
            inputs: vec![],
            origin,
        },
        &mut &bytes[..],
    )
}

/// The trace of episode `id`'s whole log. `run` names the benchmark run it belongs to, if any.
/// Refused when the episode has no harness pin (a trace without its harness is not comparable).
pub fn build_trace(
    store: &ResearchStore,
    artifacts: &dyn ArtifactStore,
    scope: &ExecutionScope,
    id: &EpisodeId,
    run: Option<HarnessRun>,
    prices: &PriceList,
) -> ServiceResult<TraceBundle> {
    let episode = store.episode(scope, id)?;
    let Some(harness) = episode.pins.harness.clone() else {
        return Err(refused(format!(
            "episode {id} has no harness pin: it never ran a turn, so there is nothing to trace"
        )));
    };
    let events = metrics::read_log(store, scope, id)?;
    let (Some(first), Some(last)) = (events.first(), events.last()) else {
        return Err(refused(format!("episode {id} has no events")));
    };
    let origin = episode.origin;
    let by_episode = Producer {
        episode_id: Some(id.clone()),
        ..Default::default()
    };

    // The event log, one event per line.
    let mut jsonl = Vec::new();
    for e in &events {
        serde_json::to_writer(&mut jsonl, e).expect("events serialize");
        jsonl.push(b'\n');
    }
    let event_log = put_json(
        artifacts,
        scope,
        by_episode.clone(),
        None,
        EVENT_LOG_MEDIA_TYPE,
        &jsonl,
        origin,
    )?;

    // Tool calls: where they sit in the log, their arguments and results as artifacts.
    let mut calls: Vec<TraceToolCall> = vec![];
    let mut answer = None;
    for e in &events {
        match &e.body {
            EventBody::ToolStarted {
                call_id,
                tool,
                arguments,
            } => {
                let by_call = Producer {
                    episode_id: Some(id.clone()),
                    tool_call_id: Some(call_id.clone()),
                    ..Default::default()
                };
                let arguments = match arguments {
                    None => None,
                    // A partial preview points at the full arguments already.
                    Some(a) if a.value.partial => a.value.full.clone(),
                    Some(a) => Some(put_json(
                        artifacts,
                        scope,
                        by_call,
                        None,
                        "application/json",
                        &serde_json::to_vec(&a.value.value).expect("JSON serializes"),
                        origin,
                    )?),
                };
                calls.push(TraceToolCall {
                    call_id: call_id.clone(),
                    tool: tool.clone(),
                    started_seq: e.seq,
                    finished_seq: None,
                    execution_status: None,
                    arguments,
                    result: None,
                });
            }
            EventBody::ToolFinished { result } => {
                let Some(c) = calls
                    .iter_mut()
                    .find(|c| c.call_id == result.call_id && c.finished_seq.is_none())
                else {
                    continue;
                };
                let by_call = Producer {
                    episode_id: Some(id.clone()),
                    tool_call_id: Some(result.call_id.clone()),
                    ..Default::default()
                };
                c.finished_seq = Some(e.seq);
                c.execution_status = Some(result.execution_status);
                c.result = Some(put_json(
                    artifacts,
                    scope,
                    by_call,
                    Some("ToolResult"),
                    "application/json",
                    &serde_json::to_vec(result).expect("ToolResult serializes"),
                    origin,
                )?);
            }
            EventBody::AnswerCompleted { answer: a, .. } => answer = Some(a.clone()),
            _ => {}
        }
    }

    let measure = metrics::measure(&episode, &events, prices);
    let cost = match &measure.metrics.cost {
        ignis_core::contracts::CostStatus::Priced { cost } => Some(cost.clone()),
        _ => None,
    };
    let millis = |t: &ignis_core::contracts::Timestamp| {
        chrono::DateTime::parse_from_rfc3339(t.as_str())
            .map(|d| d.timestamp_micros())
            .unwrap_or(0)
    };
    let wall = (millis(&last.occurred_at) - millis(&first.occurred_at)).max(0) as f64 / 1e6;
    let mut partitions: Vec<_> = scope.partitions.iter().copied().collect();
    partitions.sort_by_key(|p| format!("{p:?}"));

    let mut bundle = TraceBundle {
        schema_version: ResearchSchema::V2,
        trace_id: TraceId::new("trace-pending").expect("literal id"),
        episode_id: id.clone(),
        harness,
        runtime: episode.runtime.clone(),
        run,
        events: EventRange {
            first_seq: first.seq,
            last_seq: last.seq,
        },
        event_log: Some(event_log),
        tool_calls: calls,
        usage: measure.token_usage(),
        cost,
        partitions_touched: partitions,
        answer,
        wall_time_s: Finite::new(wall).expect("finite"),
        started_at: first.occurred_at.clone(),
        finished_at: last.occurred_at.clone(),
        origin,
    };
    let digest = content_hash(&bundle, &["trace_id"]);
    let hex = digest.strip_prefix("sha256:").unwrap_or(&digest);
    bundle.trace_id = TraceId::new(format!("trace-{}", &hex[..32])).expect("generated id");
    bundle
        .validate()
        .map_err(|e| ErrorEnvelope::new(ErrorCode::Internal, e.to_string(), RetryPolicy::Never))?;
    Ok(bundle)
}

/// The events a bundle traced, read back from its event log artifact and checked: a gap-free log
/// of the bundle's episode covering exactly its range, and a trace id that matches its content.
pub fn replay_events(
    artifacts: &dyn ArtifactStore,
    scope: &ExecutionScope,
    bundle: &TraceBundle,
) -> ServiceResult<Vec<ResearchEvent>> {
    let digest = content_hash(bundle, &["trace_id"]);
    let hex = digest.strip_prefix("sha256:").unwrap_or(&digest);
    if bundle.trace_id.as_str() != format!("trace-{}", &hex[..32]) {
        return Err(refused(format!(
            "trace {} does not match its content",
            bundle.trace_id
        )));
    }
    let Some(log) = &bundle.event_log else {
        return Err(refused(format!(
            "trace {} has no event log",
            bundle.trace_id
        )));
    };
    let mut text = String::new();
    std::io::Read::read_to_string(
        &mut artifacts.open(scope, &log.artifact_id, 0, None)?,
        &mut text,
    )
    .map_err(|e| ErrorEnvelope::new(ErrorCode::Internal, e.to_string(), RetryPolicy::Never))?;
    let events = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<ResearchEvent>(l))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| refused(format!("trace event log: {e}")))?;
    check_event_log(&events).map_err(ErrorEnvelope::from)?;
    let range = (events.first().map(|e| e.seq), events.last().map(|e| e.seq));
    if range != (Some(bundle.events.first_seq), Some(bundle.events.last_seq))
        || events.iter().any(|e| e.episode_id != bundle.episode_id)
    {
        return Err(refused(format!(
            "trace {}: the event log does not cover its range",
            bundle.trace_id
        )));
    }
    Ok(events)
}

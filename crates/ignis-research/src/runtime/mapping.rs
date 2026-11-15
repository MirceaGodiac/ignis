//! Codex thread/turn/item → IGNIS event records. Pure functions over the wire JSON of the pinned
//! protocol; the bridge stores what they return.
//!
//! Interpreted (everything else that names a thread is archived as `runtime.opaque`):
//!
//! | Codex | IGNIS |
//! |---|---|
//! | `item/agentMessage/delta` | `message.delta` (never the only copy) |
//! | `item/completed` `agentMessage` | `message.completed` |
//! | `item/started` / `item/completed` `mcpToolCall` | `tool.started` (with its arguments) / `tool.finished` |
//! | `agentMessage` right before an `mcpToolCall` in the same turn | also `step.rationale` of that call ([`Preamble`]) |
//! | `item/*` `userMessage` | nothing (IGNIS recorded `message.user` itself) |
//! | `item/completed` of any other item type | `runtime.opaque` (method `item/completed`) |
//! | `turn/completed` | `episode.status_changed` (completed / waiting_user / paused / cancelled / failed) |
//! | `thread/tokenUsage/updated` | `usage.reported` (`thread_total`; a malformed one stays opaque) |
//!
//! Every record that can be seen twice (live, then again in a thread replayed after a restart)
//! carries a mark, so it is appended once. Deltas are live-only and unmarked.

use ignis_core::contracts::{
    Causation, EpisodeStatus, EventBody, MessageId, RationaleSource, ToolCallId, UsageAccounting,
    UsageReport,
};
use serde_json::Value;

use super::preflight::LIVE;
use crate::store::{RuntimeChange, RuntimeRecord};

/// Name of the IGNIS MCP server in the runtime configuration (M3); its tools return `ToolResult`.
pub const IGNIS_MCP_SERVER: &str = "ignis";

/// Why the host stopped a turn (decides the episode status an interrupted turn ends in).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    Pause,
    Cancel,
    /// The runtime did something the profile forbids (e.g. a shell command in the public profile).
    Policy,
}

/// A record to store, or something the bridge must turn into one first.
#[derive(Debug, Clone, PartialEq)]
pub enum Mapped {
    Record(RuntimeRecord),
    /// A tool call started; the bridge records `tool.started` with its arguments (secret-like
    /// members redacted, large arguments archived as an artifact behind a partial preview).
    ToolStarted {
        call_id: ToolCallId,
        tool: String,
        /// The arguments as the runtime reported them (absent when it reported none).
        arguments: Option<Value>,
        causation: Causation,
        mark: String,
    },
    /// Archive `payload` as an artifact and record `runtime.opaque` pointing at it.
    Opaque {
        method: String,
        payload: Value,
        causation: Causation,
        mark: Option<String>,
    },
    /// A finished tool call; the bridge builds the `ToolResult` (large results become artifacts).
    ToolFinished {
        call_id: ToolCallId,
        tool: String,
        /// `structuredContent` (or `content`) of the call's result, also when it failed.
        value: Option<Value>,
        /// Message of a failed call.
        error: Option<String>,
        causation: Causation,
        mark: String,
    },
}

fn s<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

/// The thread a notification or server request belongs to, when it names one.
pub fn thread_of(params: &Value) -> Option<&str> {
    s(params, "threadId").or_else(|| params.pointer("/thread/id").and_then(Value::as_str))
}

/// The turn a notification belongs to, when it names one.
pub fn turn_of(params: &Value) -> Option<&str> {
    s(params, "turnId").or_else(|| params.pointer("/turn/id").and_then(Value::as_str))
}

fn causation(turn: Option<&str>, item: Option<&str>) -> Causation {
    Causation {
        runtime_turn_id: turn.map(str::to_string),
        runtime_item_id: item.map(str::to_string),
        ..Default::default()
    }
}

fn record(mark: Option<String>, causation: Causation, body: EventBody) -> Mapped {
    Mapped::Record(RuntimeRecord {
        mark,
        causation,
        change: RuntimeChange::Event(body),
    })
}

/// Links the model's own message to the tool call it introduces, as a `step.rationale`.
///
/// What a runtime exposes as the model's explanation of a step is the text it wrote for the user
/// right before calling a tool (Claude's text block before a `tool_use`, Codex's `agentMessage`
/// item before an `mcpToolCall`). That text, unchanged, becomes the rationale of the first call it
/// precedes; a call with no such message has none. Reasoning items (hidden or summarized thinking)
/// are never used, and nothing is generated. One per turn; replays use a fresh one per turn and get
/// the same marks.
#[derive(Debug, Default)]
pub struct Preamble {
    last: Option<String>,
    /// Calls already seen (Codex reports a call again when it completes).
    seen: std::collections::HashSet<ToolCallId>,
    /// Calls started in this turn and not finished yet, in start order.
    open: Vec<(ToolCallId, String, Causation)>,
}

/// Error of a call the runtime abandoned: the turn ended without a result for it (seen live with
/// Codex: two slow `sources_search` calls past its tool timeout never completed, and the log kept
/// them running forever).
pub const ABANDONED: &str = "the runtime ended the turn without returning this call's result (abandoned or timed out by the agent runtime); the call may have kept running on the host";

impl Preamble {
    /// Pass `mapped` through, inserting the rationale record before each call a message precedes.
    pub fn link(&mut self, mapped: Vec<Mapped>) -> Vec<Mapped> {
        let mut out = Vec::with_capacity(mapped.len());
        for m in mapped {
            match &m {
                Mapped::Record(RuntimeRecord {
                    change: RuntimeChange::Event(EventBody::AssistantMessage { text, .. }),
                    ..
                }) => {
                    self.last = Some(text.clone()).filter(|t| !t.trim().is_empty());
                }
                Mapped::ToolStarted {
                    call_id,
                    causation,
                    mark,
                    tool,
                    ..
                } => {
                    if !self.open.iter().any(|(c, _, _)| c == call_id)
                        && !self.seen.contains(call_id)
                    {
                        self.open
                            .push((call_id.clone(), tool.clone(), causation.clone()));
                    }
                    if !self.seen.insert(call_id.clone()) {
                        // Already linked (or not) when first seen.
                    } else if let Some(text) = self.last.take() {
                        out.push(record(
                            Some(format!("{mark}:rationale")),
                            causation.clone(),
                            EventBody::StepRationale {
                                call_id: Some(call_id.clone()),
                                text,
                                source: RationaleSource::ModelExplanation,
                            },
                        ));
                    }
                }
                Mapped::ToolFinished { call_id, .. } => {
                    self.open.retain(|(c, _, _)| c != call_id);
                }
                Mapped::Record(RuntimeRecord {
                    change: RuntimeChange::Status { .. },
                    ..
                }) => {
                    self.last = None;
                    // The turn is over: a call still open never gets a result from the runtime.
                    for (call_id, tool, causation) in std::mem::take(&mut self.open) {
                        let mark = format!("item:{call_id}:abandoned");
                        out.push(Mapped::ToolFinished {
                            call_id,
                            tool,
                            value: None,
                            error: Some(ABANDONED.to_string()),
                            causation,
                            mark,
                        });
                    }
                }
                _ => {}
            }
            out.push(m);
        }
        out
    }
}

fn tool_name(item: &Value) -> String {
    let tool = s(item, "tool").unwrap_or("unknown");
    match s(item, "server") {
        Some(IGNIS_MCP_SERVER) | None => tool.to_string(),
        Some(server) => format!("{server}.{tool}"),
    }
}

fn message_id(id: &str) -> Option<MessageId> {
    MessageId::new(id).ok()
}

/// Map one notification. `stop` is the host's reason if it interrupted the current turn.
pub fn map_notification(method: &str, params: &Value, stop: Option<StopReason>) -> Vec<Mapped> {
    let turn = turn_of(params);
    match method {
        "item/agentMessage/delta" => {
            let (Some(item), Some(delta)) = (s(params, "itemId"), s(params, "delta")) else {
                return opaque(method, params);
            };
            match message_id(item) {
                Some(message_id) => vec![record(
                    None,
                    causation(turn, Some(item)),
                    EventBody::AssistantDelta {
                        message_id,
                        text: delta.to_string(),
                    },
                )],
                None => opaque(method, params),
            }
        }
        "item/started" | "item/completed" => match params.get("item") {
            Some(item) => map_item(turn, item, method == "item/completed"),
            None => opaque(method, params),
        },
        "turn/completed" => match params.get("turn") {
            Some(t) => map_turn_completed(t, stop),
            None => opaque(method, params),
        },
        "thread/tokenUsage/updated" => {
            match params.pointer("/tokenUsage/total").and_then(codex_usage) {
                Some(usage) => vec![record(
                    None,
                    causation(turn, None),
                    EventBody::UsageReported { usage },
                )],
                None => opaque(method, params),
            }
        }
        // Known, but carry no event: the turn start is recorded by IGNIS before `turn/start` is sent,
        // and server-request resolution is handled by the bridge's approval bookkeeping.
        "turn/started" | "thread/started" | "serverRequest/resolved" => vec![],
        _ => opaque(method, params),
    }
}

/// The running thread total of a Codex `TokenUsageBreakdown` (input includes the cached part,
/// output includes reasoning). `None` when a required count is missing or inconsistent.
fn codex_usage(total: &Value) -> Option<UsageReport> {
    let n = |k: &str| total.get(k).and_then(Value::as_u64);
    let usage = UsageReport {
        accounting: UsageAccounting::ThreadTotal,
        model: None,
        input_tokens: n("inputTokens")?,
        cached_input_tokens: n("cachedInputTokens"),
        cache_write_tokens: None,
        output_tokens: n("outputTokens")?,
        reasoning_tokens: n("reasoningOutputTokens"),
        reported_cost: None,
    };
    usage.validate().is_ok().then_some(usage)
}

fn opaque(method: &str, params: &Value) -> Vec<Mapped> {
    vec![Mapped::Opaque {
        method: method.to_string(),
        payload: params.clone(),
        causation: causation(turn_of(params), None),
        mark: None,
    }]
}

/// Map a thread item (from `item/started`, `item/completed`, or a thread replayed by
/// `thread/resume`, where every item counts as completed).
pub fn map_item(turn: Option<&str>, item: &Value, completed: bool) -> Vec<Mapped> {
    let Some(id) = s(item, "id") else {
        return vec![Mapped::Opaque {
            method: if completed {
                "item/completed"
            } else {
                "item/started"
            }
            .into(),
            payload: item.clone(),
            causation: causation(turn, None),
            mark: None,
        }];
    };
    let c = causation(turn, Some(id));
    let started_mark = format!("item:{id}:started");
    let done_mark = format!("item:{id}:completed");
    match (s(item, "type").unwrap_or(""), completed) {
        ("userMessage", _) => vec![],
        ("agentMessage", false) => vec![],
        ("agentMessage", true) => match message_id(id) {
            Some(message_id) => vec![record(
                Some(done_mark),
                c,
                EventBody::AssistantMessage {
                    message_id,
                    text: s(item, "text").unwrap_or_default().to_string(),
                },
            )],
            None => vec![],
        },
        ("mcpToolCall", done) => {
            let Ok(call_id) = ToolCallId::new(id) else {
                return vec![];
            };
            let tool = tool_name(item);
            let tool_call = {
                let id = call_id.clone();
                move |c: Causation| Causation {
                    tool_call_id: Some(id.clone()),
                    ..c
                }
            };
            let mut out = vec![Mapped::ToolStarted {
                call_id: call_id.clone(),
                tool: tool.clone(),
                arguments: item.get("arguments").filter(|a| !a.is_null()).cloned(),
                causation: tool_call(c.clone()),
                mark: started_mark,
            }];
            let status = s(item, "status").unwrap_or("");
            if done && status != "inProgress" {
                let failed = status == "failed" || item.get("error").is_some_and(|e| !e.is_null());
                let value = item.get("result").filter(|r| !r.is_null()).map(|r| {
                    r.get("structuredContent")
                        .filter(|v| !v.is_null())
                        .or_else(|| r.get("content"))
                        .cloned()
                        .unwrap_or(Value::Null)
                });
                // A failed call keeps its result: an IGNIS refusal is a `ToolResult` carried in an
                // MCP error result, and its reason must reach the log (the recorder decides).
                out.push(Mapped::ToolFinished {
                    call_id,
                    tool,
                    value,
                    error: failed.then(|| {
                        item.pointer("/error/message")
                            .and_then(Value::as_str)
                            .unwrap_or("tool call failed")
                            .to_string()
                    }),
                    causation: tool_call(c),
                    mark: done_mark,
                });
            }
            out
        }
        // Other item types are archived once, from their completed snapshot.
        (_, false) => vec![],
        (_, true) => vec![Mapped::Opaque {
            method: "item/completed".into(),
            payload: item.clone(),
            causation: c,
            mark: Some(done_mark),
        }],
    }
}

/// The episode status a finished turn leads to, and the turn outcome stored with it.
pub fn turn_outcome(turn: &Value, stop: Option<StopReason>) -> (EpisodeStatus, String, String) {
    let status = s(turn, "status").unwrap_or("");
    let error = turn
        .pointer("/error/message")
        .and_then(Value::as_str)
        .unwrap_or("no error message");
    match (status, stop) {
        ("completed", _) => (
            EpisodeStatus::Completed,
            "completed".into(),
            "turn completed".into(),
        ),
        ("interrupted", Some(StopReason::Pause)) => (
            EpisodeStatus::Paused,
            "interrupted".into(),
            "turn stopped at a checkpoint for pause".into(),
        ),
        ("interrupted", Some(StopReason::Cancel)) => (
            EpisodeStatus::Cancelled,
            "interrupted".into(),
            "turn cancelled".into(),
        ),
        ("interrupted", Some(StopReason::Policy)) => (
            EpisodeStatus::Failed,
            "interrupted".into(),
            "turn stopped: the runtime tried a shell command, which the public profile forbids"
                .into(),
        ),
        ("interrupted", None) => (
            EpisodeStatus::WaitingUser,
            "interrupted".into(),
            "turn interrupted by the runtime; its messages are not re-sent".into(),
        ),
        ("failed", _) => (
            EpisodeStatus::Failed,
            "failed".into(),
            format!("turn failed: {error}"),
        ),
        (other, _) => (
            EpisodeStatus::Failed,
            format!("unknown:{other}"),
            format!("turn ended with unknown status {other:?}"),
        ),
    }
}

/// Map a finished turn to a status change (applied only while the turn is live: a pause or cancel
/// already recorded by the host wins).
pub fn map_turn_completed(turn: &Value, stop: Option<StopReason>) -> Vec<Mapped> {
    let Some(id) = s(turn, "id") else {
        return vec![];
    };
    if s(turn, "status") == Some("inProgress") {
        return vec![];
    }
    let (to, _, reason) = turn_outcome(turn, stop);
    vec![Mapped::Record(RuntimeRecord {
        mark: Some(format!("turn:{id}:completed")),
        causation: causation(Some(id), None),
        change: RuntimeChange::Status {
            to,
            reason: Some(reason),
            from_any: LIVE.to_vec(),
        },
    })]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn unknown_notifications_are_opaque_and_never_interpreted() {
        let p = json!({"threadId": "t", "turnId": "u", "anything": 1});
        let out = map_notification("x/somethingNew", &p, None);
        assert!(
            matches!(&out[..], [Mapped::Opaque { method, payload, .. }] if method == "x/somethingNew" && payload == &p)
        );
    }

    #[test]
    fn codex_token_usage_is_a_thread_total_and_a_malformed_one_stays_opaque() {
        let p = json!({"threadId": "t", "turnId": "u", "tokenUsage": {
            "total": {"totalTokens": 130, "inputTokens": 100, "cachedInputTokens": 60,
                      "outputTokens": 30, "reasoningOutputTokens": 12},
            "last": {"inputTokens": 1, "outputTokens": 1}}});
        let out = map_notification("thread/tokenUsage/updated", &p, None);
        let [Mapped::Record(r)] = &out[..] else {
            panic!("{out:?}")
        };
        let RuntimeChange::Event(EventBody::UsageReported { usage }) = &r.change else {
            panic!()
        };
        assert_eq!(usage.accounting, UsageAccounting::ThreadTotal);
        assert_eq!(
            (usage.input_tokens, usage.cached_input_tokens),
            (100, Some(60))
        );
        assert_eq!(
            (usage.output_tokens, usage.reasoning_tokens),
            (30, Some(12))
        );
        assert_eq!(r.causation.runtime_turn_id.as_deref(), Some("u"));
        // No output count, or more cached than input: not interpreted.
        for total in [
            json!({"inputTokens": 10}),
            json!({"inputTokens": 1, "cachedInputTokens": 2, "outputTokens": 0}),
        ] {
            let p = json!({"threadId": "t", "tokenUsage": {"total": total}});
            let out = map_notification("thread/tokenUsage/updated", &p, None);
            assert!(matches!(&out[..], [Mapped::Opaque { .. }]), "{out:?}");
        }
    }

    #[test]
    fn a_failed_turn_is_never_reported_as_success() {
        let t = json!({"id": "u", "status": "failed", "error": {"message": "model refused"}, "items": []});
        let out = map_turn_completed(&t, None);
        let Mapped::Record(r) = &out[0] else { panic!() };
        assert!(
            matches!(&r.change, RuntimeChange::Status { to: EpisodeStatus::Failed, reason: Some(m), .. } if m.contains("model refused"))
        );
    }

    #[test]
    fn a_call_the_runtime_abandons_is_closed_when_the_turn_ends() {
        // Two calls start; one completes; the turn ends without the other (Codex past its tool
        // timeout). The log must not keep the second one running forever.
        let started = |id: &str| {
            json!({"type": "mcpToolCall", "id": id, "server": "ignis", "tool": "sources_search",
                   "status": "inProgress", "arguments": {"query": "q"}})
        };
        let done = json!({"type": "mcpToolCall", "id": "call-a", "server": "ignis", "tool": "sources_search",
                          "status": "completed", "arguments": {"query": "q"},
                          "result": {"content": [], "structuredContent": {"x": 1}}});
        let mut p = Preamble::default();
        let mut out = Vec::new();
        out.extend(p.link(map_item(Some("u"), &started("call-a"), false)));
        out.extend(p.link(map_item(Some("u"), &started("call-b"), false)));
        out.extend(p.link(map_item(Some("u"), &done, true)));
        let end = json!({"id": "u", "status": "completed", "items": []});
        out.extend(p.link(map_turn_completed(&end, None)));
        let finished: Vec<_> = out
            .iter()
            .filter_map(|m| match m {
                Mapped::ToolFinished {
                    call_id,
                    error,
                    mark,
                    ..
                } => Some((call_id.as_str().to_string(), error.clone(), mark.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(finished.len(), 2, "{out:?}");
        assert_eq!(finished[0].0, "call-a");
        assert_eq!(finished[0].1, None);
        assert_eq!(finished[1].0, "call-b");
        assert_eq!(finished[1].1.as_deref(), Some(ABANDONED));
        assert_eq!(finished[1].2, "item:call-b:abandoned");
        // The closing record comes before the turn's status change.
        assert!(matches!(
            out.last(),
            Some(Mapped::Record(RuntimeRecord {
                change: RuntimeChange::Status { .. },
                ..
            }))
        ));
    }

    #[test]
    fn a_tool_call_seen_live_and_in_a_replay_has_stable_marks() {
        let item = json!({"type": "mcpToolCall", "id": "call-1", "server": "ignis", "tool": "sources_search",
                          "status": "completed", "arguments": {}, "result": {"content": [], "structuredContent": {"x": 1}}});
        let live: Vec<_> = map_item(Some("u"), &item, true);
        let replay: Vec<_> = map_item(Some("u"), &item, true);
        assert_eq!(live, replay);
        assert!(
            matches!(&live[1], Mapped::ToolFinished { value: Some(v), mark, .. } if v == &json!({"x": 1}) && mark == "item:call-1:completed")
        );
    }
}

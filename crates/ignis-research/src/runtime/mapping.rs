//! Codex thread/turn/item → IGNIS event records. Pure functions over the wire JSON of the pinned
//! protocol; the bridge stores what they return.
//!
//! Interpreted (everything else that names a thread is archived as `runtime.opaque`):
//!
//! | Codex | IGNIS |
//! |---|---|
//! | `item/agentMessage/delta` | `message.delta` (never the only copy) |
//! | `item/completed` `agentMessage` | `message.completed` |
//! | `item/started` / `item/completed` `mcpToolCall` | `tool.started` / `tool.finished` |
//! | `item/*` `userMessage` | nothing (IGNIS recorded `message.user` itself) |
//! | `item/completed` of any other item type | `runtime.opaque` (method `item/completed`) |
//! | `turn/completed` | `episode.status_changed` (completed / waiting_user / paused / cancelled / failed) |
//!
//! Every record that can be seen twice (live, then again in a thread replayed after a restart)
//! carries a mark, so it is appended once. Deltas are live-only and unmarked.

use ignis_core::contracts::{Causation, EpisodeStatus, EventBody, MessageId, ToolCallId};
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
        /// `structuredContent` (or `content`) of a completed call.
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
        // Known, but carry no event: the turn start is recorded by IGNIS before `turn/start` is sent,
        // and server-request resolution is handled by the bridge's approval bookkeeping.
        "turn/started" | "thread/started" | "serverRequest/resolved" => vec![],
        _ => opaque(method, params),
    }
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
            let mut out = vec![record(
                Some(started_mark),
                tool_call(c.clone()),
                EventBody::ToolStarted {
                    call_id: call_id.clone(),
                    tool: tool.clone(),
                },
            )];
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
                out.push(Mapped::ToolFinished {
                    call_id,
                    tool,
                    value: if failed { None } else { value },
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
    fn a_failed_turn_is_never_reported_as_success() {
        let t = json!({"id": "u", "status": "failed", "error": {"message": "model refused"}, "items": []});
        let out = map_turn_completed(&t, None);
        let Mapped::Record(r) = &out[0] else { panic!() };
        assert!(
            matches!(&r.change, RuntimeChange::Status { to: EpisodeStatus::Failed, reason: Some(m), .. } if m.contains("model refused"))
        );
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

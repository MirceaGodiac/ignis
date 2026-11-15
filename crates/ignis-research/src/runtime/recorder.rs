//! Recording runtime output in an episode log, shared by every [`super::AgentRuntime`]
//! implementation: mapped items become records (opaque payloads and large tool results are archived
//! as artifacts) appended in one transaction, then the commit hook wakes the episode's readers.

use std::sync::Arc;

use ignis_core::contracts::{
    EpisodeId, ErrorCode, ErrorEnvelope, EventBody, ExecutionScope, ExecutionStatus, InlinePayload,
    Origin, Producer, RetryPolicy, ScientificStatus, ToolArguments, ToolCallId, ToolResult,
    ToolsSchema,
};
use ignis_core::services::{ArtifactDraft, ArtifactStore, ServiceResult};
use serde_json::{json, Value};

use super::bridge::CommitHook;
use super::mapping::Mapped;
use crate::store::{RuntimeChange, RuntimeRecord};
use crate::ResearchStore;

/// Tool results larger than this (serialized) go to an artifact; the event keeps a reference.
pub(super) const INLINE_MAX: usize = 64 * 1024;

/// Strings longer than this are cut in the preview of archived arguments (the artifact keeps them).
const PREVIEW_STRING_BYTES: usize = 2 * 1024;

/// Member names whose values are credentials, never written to the log. IGNIS tools take no
/// credentials (scope is host-derived), so such a member can only come from another server or a
/// confused model; its value is dropped and its pointer listed as redacted.
fn secret_like(key: &str) -> bool {
    let k = key.to_ascii_lowercase().replace(['-', '_'], "");
    [
        "authorization",
        "password",
        "passwd",
        "secret",
        "token",
        "apikey",
        "cookie",
        "credential",
        "privatekey",
    ]
    .iter()
    .any(|s| k.contains(s))
}

fn pointer_segment(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// Remove secret-like members (at any depth), returning their JSON Pointers.
fn redact(v: &mut Value, at: &str, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            let keys: Vec<String> = o.keys().cloned().collect();
            for k in keys {
                let p = format!("{at}/{}", pointer_segment(&k));
                if secret_like(&k) {
                    o.remove(&k);
                    out.push(p);
                } else if let Some(x) = o.get_mut(&k) {
                    redact(x, &p, out);
                }
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter_mut().enumerate() {
                redact(x, &format!("{at}/{i}"), out);
            }
        }
        _ => {}
    }
}

/// The same shape with long strings cut (marked with their full length); the full value is in the
/// artifact the preview points at.
fn preview(v: &Value) -> Value {
    match v {
        Value::String(s) if s.len() > PREVIEW_STRING_BYTES => {
            let mut end = PREVIEW_STRING_BYTES;
            while !s.is_char_boundary(end) {
                end -= 1;
            }
            Value::String(format!(
                "{}… [{} bytes in the artifact]",
                &s[..end],
                s.len()
            ))
        }
        Value::Object(o) => Value::Object(o.iter().map(|(k, x)| (k.clone(), preview(x))).collect()),
        Value::Array(a) => Value::Array(a.iter().map(preview).collect()),
        other => other.clone(),
    }
}

/// The IGNIS `ToolResult` in an MCP call result: its structured content, or the JSON text of a
/// content block (clients that only keep text content).
fn ignis_result(v: &Value) -> Option<ToolResult> {
    if let Ok(r) = serde_json::from_value::<ToolResult>(v.clone()) {
        return Some(r);
    }
    v.as_array()?.iter().find_map(|block| {
        let text = block.get("text").and_then(Value::as_str)?;
        serde_json::from_str::<ToolResult>(text).ok()
    })
}

pub(super) struct Recorder {
    pub store: Arc<ResearchStore>,
    pub artifacts: Arc<dyn ArtifactStore>,
    pub on_commit: CommitHook,
}

impl Recorder {
    pub(super) fn archive(
        &self,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        origin: Origin,
        value: &Value,
    ) -> ServiceResult<ignis_core::contracts::ArtifactRef> {
        let bytes = serde_json::to_vec(value).expect("JSON serializes");
        self.artifacts.put(
            scope,
            ArtifactDraft {
                media_type: "application/json".into(),
                contract: None,
                produced_by: Producer {
                    episode_id: Some(ep.clone()),
                    ..Default::default()
                },
                inputs: vec![],
                origin,
            },
            &mut bytes.as_slice(),
        )
    }

    /// The arguments of a started call as the log shows them: secret-like members redacted, and
    /// arguments larger than [`INLINE_MAX`] archived whole (after redaction) behind a partial
    /// preview. A runtime that sends arguments as a JSON string gets them parsed.
    pub(super) fn tool_arguments(
        &self,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        origin: Origin,
        arguments: Value,
    ) -> ServiceResult<ToolArguments> {
        let mut v = match arguments {
            Value::String(s) => serde_json::from_str(&s).unwrap_or(Value::String(s)),
            other => other,
        };
        let mut redacted = vec![];
        redact(&mut v, "", &mut redacted);
        let size = serde_json::to_vec(&v).map_or(0, |b| b.len());
        let value = if size > INLINE_MAX {
            let full = self.archive(scope, ep, origin, &v)?;
            InlinePayload {
                value: preview(&v),
                partial: true,
                full: Some(full),
            }
        } else {
            InlinePayload {
                value: v,
                partial: false,
                full: None,
            }
        };
        let a = ToolArguments { value, redacted };
        a.validate().map_err(ErrorEnvelope::from)?;
        Ok(a)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn tool_result(
        &self,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        origin: Origin,
        call_id: ToolCallId,
        tool: String,
        value: Option<Value>,
        error: Option<String>,
    ) -> ToolResult {
        // Results of the IGNIS MCP server are already `ToolResult`s (M3 adapters). The server does
        // not know the runtime's item id, so the result is re-keyed to the call it finishes.
        // A failed call carries the server's refusal or failure (with its reason) this way; it is
        // trusted only if it does not claim success.
        if !tool.contains('.') {
            if let Some(mut r) = value
                .as_ref()
                .and_then(ignis_result)
                .filter(|r| r.validate().is_ok() && r.tool == tool)
                .filter(|r| {
                    error.is_none()
                        || matches!(
                            r.execution_status,
                            ExecutionStatus::Refused
                                | ExecutionStatus::Failed
                                | ExecutionStatus::Cancelled
                        )
                })
            {
                r.call_id = call_id;
                return r;
            }
        }
        let failed = |msg: String| ToolResult {
            schema_version: ToolsSchema::V1,
            call_id: call_id.clone(),
            tool: tool.clone(),
            execution_status: ExecutionStatus::Failed,
            scientific_status: ScientificStatus::NotAssessed,
            result: None,
            job: None,
            artifact_refs: vec![],
            error: Some(ErrorEnvelope::new(
                ErrorCode::Internal,
                msg,
                RetryPolicy::Never,
            )),
            origin,
        };
        if let Some(msg) = error {
            return failed(msg);
        }
        let mut artifact_refs = vec![];
        let result = match value {
            None => None,
            Some(v) if serde_json::to_vec(&v).map_or(0, |b| b.len()) > INLINE_MAX => {
                match self.archive(scope, ep, origin, &v) {
                    Ok(a) => {
                        artifact_refs.push(a.clone());
                        Some(InlinePayload {
                            value: Value::Null,
                            partial: true,
                            full: Some(a),
                        })
                    }
                    Err(e) => return failed(format!("store the tool result: {}", e.message)),
                }
            }
            Some(v) => Some(InlinePayload {
                value: v,
                partial: false,
                full: None,
            }),
        };
        let r = ToolResult {
            schema_version: ToolsSchema::V1,
            call_id: call_id.clone(),
            tool: tool.clone(),
            execution_status: ExecutionStatus::Completed,
            scientific_status: ScientificStatus::NotAssessed,
            result,
            job: None,
            artifact_refs,
            error: None,
            origin,
        };
        match r.validate() {
            Ok(()) => r,
            Err(e) => failed(format!(
                "the runtime's tool result is not a valid ToolResult: {e}"
            )),
        }
    }

    /// Turn mapped items into records (archiving opaque payloads and large tool results) and append
    /// them in one transaction; wake the episode's readers.
    pub(super) fn commit(
        &self,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        origin: Origin,
        mapped: Vec<Mapped>,
    ) {
        let mut records = Vec::with_capacity(mapped.len());
        for m in mapped {
            match m {
                Mapped::Record(r) => records.push(r),
                Mapped::Opaque {
                    method,
                    payload,
                    causation,
                    mark,
                } => {
                    // A replayed thread repeats archived items: do not store their payloads again.
                    if mark
                        .as_deref()
                        .is_some_and(|m| self.store.runtime_marked(ep, m).unwrap_or(false))
                    {
                        continue;
                    }
                    let doc = json!({"method": method, "params": payload});
                    let payload = match self.archive(scope, ep, origin, &doc) {
                        Ok(a) => Some(a),
                        Err(e) => {
                            tracing::warn!(error = %e, %method, "archive opaque runtime payload");
                            None
                        }
                    };
                    records.push(RuntimeRecord {
                        mark,
                        causation,
                        change: RuntimeChange::Event(EventBody::RuntimeOpaque { method, payload }),
                    });
                }
                Mapped::ToolStarted {
                    call_id,
                    tool,
                    arguments,
                    causation,
                    mark,
                } => {
                    // Seen again (item/completed after item/started, or a replay): already logged,
                    // and its arguments already archived.
                    if self.store.runtime_marked(ep, &mark).unwrap_or(false) {
                        continue;
                    }
                    let arguments = arguments.and_then(|a| {
                        self.tool_arguments(scope, ep, origin, a)
                            .map_err(|e| {
                                tracing::warn!(error = %e, %tool, "tool arguments not recorded");
                            })
                            .ok()
                    });
                    records.push(RuntimeRecord {
                        mark: Some(mark),
                        causation,
                        change: RuntimeChange::Event(EventBody::ToolStarted {
                            call_id,
                            tool,
                            arguments,
                        }),
                    });
                }
                Mapped::ToolFinished {
                    call_id,
                    tool,
                    value,
                    error,
                    causation,
                    mark,
                } => {
                    let result = self.tool_result(scope, ep, origin, call_id, tool, value, error);
                    records.push(RuntimeRecord {
                        mark: Some(mark),
                        causation,
                        change: RuntimeChange::Event(EventBody::ToolFinished { result }),
                    });
                }
            }
        }
        if records.is_empty() {
            return;
        }
        match self.store.record_runtime(scope, ep, records) {
            Ok(events) => {
                if let Some(last) = events.last() {
                    (self.on_commit)(ep, last.seq);
                }
                // A turn ended: one structured metrics line for it (M5).
                for e in &events {
                    if let (EventBody::StatusChanged { .. }, Some(turn)) =
                        (&e.body, &e.causation.runtime_turn_id)
                    {
                        let prices = crate::metrics::prices();
                        match crate::metrics::measure_episode(&self.store, scope, ep, &prices) {
                            Ok(m) => crate::metrics::log_turn(&m.metrics, turn),
                            Err(err) => tracing::warn!(episode = %ep, error = %err, "turn metrics"),
                        }
                    }
                }
            }
            Err(e) => tracing::warn!(episode = %ep, error = %e, "record runtime events"),
        }
    }
}

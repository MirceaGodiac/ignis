//! Recording runtime output in an episode log, shared by every [`super::AgentRuntime`]
//! implementation: mapped items become records (opaque payloads and large tool results are archived
//! as artifacts) appended in one transaction, then the commit hook wakes the episode's readers.

use std::sync::Arc;

use ignis_core::contracts::{
    EpisodeId, ErrorCode, ErrorEnvelope, EventBody, ExecutionScope, ExecutionStatus, InlinePayload,
    Origin, Producer, RetryPolicy, ScientificStatus, ToolCallId, ToolResult, ToolsSchema,
};
use ignis_core::services::{ArtifactDraft, ArtifactStore, ServiceResult};
use serde_json::{json, Value};

use super::bridge::CommitHook;
use super::mapping::Mapped;
use crate::store::{RuntimeChange, RuntimeRecord};
use crate::ResearchStore;

/// Tool results larger than this (serialized) go to an artifact; the event keeps a reference.
pub(super) const INLINE_MAX: usize = 64 * 1024;

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
            }
            Err(e) => tracing::warn!(episode = %ep, error = %e, "record runtime events"),
        }
    }
}

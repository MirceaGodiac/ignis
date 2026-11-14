//! [`CodexBridge`]: the [`AgentRuntime`] over pinned `codex app-server` processes.
//!
//! - One app-server per workspace, `CODEX_HOME = <home>/ws-<hash>`, started on first use, private
//!   stdio only. One Codex thread per episode (`ResearchEpisode.runtime`), cwd an empty per-episode
//!   directory.
//! - A dispatcher thread per connection handles inbound messages in wire order: notifications are
//!   mapped ([`super::mapping`]) and recorded through the state store (gap-free `seq`, then the
//!   commit hook wakes SSE); server requests are answered explicitly, approvals through the host.
//! - Turns: [`ResearchStore::begin_turn`] records the intent (and `running`) before `turn/start` is
//!   sent; the store's unique index allows one open turn per episode. Messages that arrive during a
//!   turn are picked up by the next one. A turn this process does not run (crash, lost connection)
//!   is settled from the runtime thread (`thread/resume`) before anything new starts; its messages
//!   are never re-sent.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex, Weak};

use ignis_core::contracts::{
    ApprovalDecision, ApprovalId, Capability, Causation, EpisodeId, EpisodeStatus, ErrorCode,
    ErrorEnvelope, EventBody, ExecutionScope, Origin, ResearchEpisode, RetryPolicy, RuntimeKind,
    RuntimeThreadRef, ToolEffect, WorkspaceId,
};
use ignis_core::hashing::sha256_hex;
use ignis_core::services::{ArtifactStore, ServiceResult};
use serde_json::{json, Value};

use super::grants::{McpLease, MCP_TOKEN_ENV};
use super::harness;
use super::mapping::{self, map_item, map_notification, map_turn_completed, Mapped, StopReason};
use super::preflight::{
    ApprovalKind, ApprovalRule, CodexConfig, ProfilePolicy, RuntimeBuildManifest, LIVE,
};
use super::recorder::Recorder;
use super::rpc::{CallError, Connection, Inbound};
use super::AgentRuntime;
use crate::store::{ActiveTurn, RuntimeChange, RuntimeRecord};
use crate::ResearchStore;

/// Called after events were committed to an episode log (the host wakes its SSE readers).
pub type CommitHook = Arc<dyn Fn(&EpisodeId, u64) + Send + Sync>;

/// High-volume streams whose content is repeated in the `item/completed` snapshot: suppressed at
/// the source (`optOutNotificationMethods`) instead of archived delta by delta.
const OPT_OUT: &[&str] = &[
    "item/reasoning/summaryTextDelta",
    "item/reasoning/summaryPartAdded",
    "item/reasoning/textDelta",
    "item/plan/delta",
    "item/commandExecution/outputDelta",
    "item/commandExecution/terminalInteraction",
    "item/fileChange/outputDelta",
    "item/fileChange/patchUpdated",
    "item/mcpToolCall/progress",
    "command/exec/outputDelta",
    "process/outputDelta",
    "account/rateLimits/updated",
];

pub struct CodexBridge {
    inner: Arc<Inner>,
}

struct Inner {
    cfg: CodexConfig,
    manifest: RuntimeBuildManifest,
    store: Arc<ResearchStore>,
    on_commit: CommitHook,
    rec: Recorder,
    state: Mutex<State>,
    spawn_lock: Mutex<()>,
    me: Weak<Inner>,
}

#[derive(Default)]
struct State {
    conns: HashMap<WorkspaceId, Arc<Live>>,
    slots: HashMap<EpisodeId, Slot>,
    threads: HashMap<String, EpisodeId>,
    /// Keyed by episode too: item ids are only unique within a runtime thread.
    approvals: HashMap<(EpisodeId, ApprovalId), Pending>,
    generation: u64,
    shutdown: bool,
}

/// One running app-server.
struct Live {
    rpc: Arc<Connection>,
    generation: u64,
    workspace: WorkspaceId,
    /// Threads started or resumed on this process.
    loaded: Mutex<HashSet<String>>,
    /// The process's IGNIS MCP token; taking it out revokes it.
    mcp: Mutex<Option<McpLease>>,
    /// Episode grants of the threads loaded on this process (sent per thread as a header, so a
    /// call knows its episode); revoked with the process.
    episode_mcp: Mutex<HashMap<EpisodeId, McpLease>>,
}

impl Live {
    fn revoke_mcp(&self) {
        drop(self.mcp.lock().unwrap_or_else(|p| p.into_inner()).take());
        self.episode_mcp
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
    }
}

/// What this process knows about an episode's turn.
struct Slot {
    scope: ExecutionScope,
    origin: Origin,
    /// A turn is being started or runs in this process.
    busy: bool,
    /// More work was asked for while busy.
    again: bool,
    turn_key: Option<String>,
    runtime_turn_id: Option<String>,
    thread_id: Option<String>,
    generation: u64,
    stop: Option<StopReason>,
    /// Said in the next turn's status change (e.g. that the runtime thread was replaced).
    note: Option<String>,
}

impl Slot {
    fn new(scope: &ExecutionScope) -> Self {
        Slot {
            scope: scope.clone(),
            origin: Origin::System,
            busy: false,
            again: false,
            turn_key: None,
            runtime_turn_id: None,
            thread_id: None,
            generation: 0,
            stop: None,
            note: None,
        }
    }
}

enum Verdict {
    Host(ApprovalDecision),
    /// The runtime withdrew the request (or its connection is gone): nothing to answer.
    Withdrawn,
}

struct Pending {
    episode: EpisodeId,
    workspace: WorkspaceId,
    effect: ToolEffect,
    rpc_id: Value,
    generation: u64,
    tx: mpsc::Sender<Verdict>,
}

fn unavailable(msg: impl std::fmt::Display) -> ErrorEnvelope {
    ErrorEnvelope::new(
        ErrorCode::CapabilityUnavailable,
        format!("agent runtime: {msg}"),
        RetryPolicy::AfterDelay { seconds: 30 },
    )
}

fn call_err(e: CallError) -> ErrorEnvelope {
    let retry = if e.is_ambiguous() {
        RetryPolicy::Reconcile
    } else {
        RetryPolicy::Never
    };
    ErrorEnvelope::new(ErrorCode::UpstreamUnavailable, e.to_string(), retry)
}

fn status(to: EpisodeStatus, from_any: &[EpisodeStatus], reason: String) -> RuntimeChange {
    RuntimeChange::Status {
        to,
        reason: Some(reason),
        from_any: from_any.to_vec(),
    }
}

fn required_capability(effect: ToolEffect) -> Option<Capability> {
    match effect {
        ToolEffect::Read => None,
        ToolEffect::WorkspaceWrite => Some(Capability::WorkspaceWrite),
        ToolEffect::Compute => Some(Capability::ExecuteCode),
        ToolEffect::NetworkFetch => Some(Capability::NetworkFetch),
        ToolEffect::ExternalWrite => Some(Capability::ExternalWrite),
    }
}

impl CodexBridge {
    /// A bridge for a preflighted build. No process starts until an episode needs one.
    pub fn new(
        cfg: CodexConfig,
        manifest: RuntimeBuildManifest,
        store: Arc<ResearchStore>,
        artifacts: Arc<dyn ArtifactStore>,
        on_commit: CommitHook,
    ) -> Self {
        let rec = Recorder {
            store: store.clone(),
            artifacts: artifacts.clone(),
            on_commit: on_commit.clone(),
        };
        let inner = Arc::new_cyclic(|me| Inner {
            cfg,
            manifest,
            store,
            on_commit,
            rec,
            state: Mutex::new(State::default()),
            spawn_lock: Mutex::new(()),
            me: me.clone(),
        });
        CodexBridge { inner }
    }

    /// Stop every app-server without recording anything (what a host crash looks like to the log).
    /// Open turns stay open in the store and are reconciled by the next process ([`AgentRuntime::recover`]).
    pub fn shutdown(&self) {
        let conns: Vec<Arc<Live>> = {
            let mut st = self.inner.lock();
            st.shutdown = true;
            st.approvals.clear();
            st.conns.drain().map(|(_, l)| l).collect()
        };
        for l in conns {
            l.rpc.kill();
            l.revoke_mcp();
        }
    }
}

impl Drop for CodexBridge {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl AgentRuntime for CodexBridge {
    fn manifest(&self) -> &RuntimeBuildManifest {
        &self.inner.manifest
    }

    fn kick(&self, scope: &ExecutionScope, episode: &EpisodeId) -> ServiceResult<()> {
        self.inner.store.episode(scope, episode)?;
        self.inner.drive(scope, episode, false)
    }

    fn interrupt(
        &self,
        scope: &ExecutionScope,
        episode: &EpisodeId,
        why: StopReason,
    ) -> ServiceResult<()> {
        self.inner.store.episode(scope, episode)?;
        self.inner.interrupt(episode, why);
        Ok(())
    }

    fn resume(&self, scope: &ExecutionScope, episode: &EpisodeId) -> ServiceResult<()> {
        self.inner.store.episode(scope, episode)?;
        // Reconciliation may resume a long thread: it runs on the runtime's own thread, and its
        // outcome reaches the episode log like any runtime event.
        self.inner.drive(scope, episode, false)
    }

    fn resolve_approval(
        &self,
        scope: &ExecutionScope,
        episode: &EpisodeId,
        approval: &ApprovalId,
        decision: ApprovalDecision,
    ) -> ServiceResult<()> {
        if !matches!(
            decision,
            ApprovalDecision::Approved | ApprovalDecision::Denied
        ) {
            return Err(ErrorEnvelope::new(
                ErrorCode::InvalidRequest,
                "a caller can only approve or deny",
                RetryPolicy::Never,
            ));
        }
        self.inner.store.episode(scope, episode)?;
        let mut st = self.inner.lock();
        let key = (episode.clone(), approval.clone());
        let effect = match st.approvals.get(&key) {
            Some(p) if p.episode == *episode && p.workspace == scope.workspace_id => p.effect,
            _ => {
                return Err(ErrorEnvelope::new(
                    ErrorCode::NotFound,
                    format!("approval {approval} not found or already resolved"),
                    RetryPolicy::Never,
                ))
            }
        };
        if decision == ApprovalDecision::Approved {
            if let Some(cap) = required_capability(effect) {
                scope.require(cap).map_err(ErrorEnvelope::from)?;
            }
        }
        let p = st.approvals.remove(&key).expect("checked above");
        let _ = p.tx.send(Verdict::Host(decision));
        Ok(())
    }

    fn recover(&self) -> ServiceResult<()> {
        for open in self.inner.store.active_turns()? {
            if let Err(e) = self.inner.drive(&open.scope, &open.episode_id, true) {
                tracing::warn!(episode = %open.episode_id, error = %e, "runtime recovery failed");
            }
        }
        Ok(())
    }
}

impl Inner {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn arc(&self) -> Arc<Inner> {
        self.me.upgrade().expect("bridge alive while in use")
    }

    fn ws_home(&self, ws: &WorkspaceId) -> PathBuf {
        if let Some(shared) = &self.cfg.login_home {
            return shared.clone();
        }
        self.cfg
            .home
            .join(format!("ws-{}", &sha256_hex(ws.as_str().as_bytes())[..16]))
    }

    // ------------------------------------------------------------------------ turns

    /// Claim the episode and run its pending work (here when `sync`, else on a new thread).
    fn drive(&self, scope: &ExecutionScope, ep: &EpisodeId, sync: bool) -> ServiceResult<()> {
        let claimed = {
            let mut st = self.lock();
            if st.shutdown {
                return Err(unavailable("the bridge is shut down"));
            }
            let slot = st
                .slots
                .entry(ep.clone())
                .or_insert_with(|| Slot::new(scope));
            slot.scope = scope.clone();
            if slot.busy {
                slot.again = true;
                false
            } else {
                slot.busy = true;
                slot.again = false;
                true
            }
        };
        if claimed {
            if sync {
                self.run_turns(ep);
            } else {
                let (inner, ep) = (self.arc(), ep.clone());
                std::thread::Builder::new()
                    .name("codex-turn".into())
                    .spawn(move || inner.run_turns(&ep))
                    .map_err(|e| unavailable(format!("spawn turn thread: {e}")))?;
            }
        }
        Ok(())
    }

    fn run_turns(&self, ep: &EpisodeId) {
        loop {
            let scope = match self.lock().slots.get(ep) {
                Some(s) => s.scope.clone(),
                None => return,
            };
            let outcome = self.start_one(&scope, ep);
            let mut st = self.lock();
            let Some(slot) = st.slots.get_mut(ep) else {
                return;
            };
            match outcome {
                Ok(true) => return,
                Ok(false) if slot.again => slot.again = false,
                Ok(false) => {
                    slot.busy = false;
                    return;
                }
                Err(e) => {
                    tracing::warn!(episode = %ep, error = %e, "runtime turn not started");
                    slot.busy = false;
                    slot.again = false;
                    return;
                }
            }
        }
    }

    /// Settle what a previous process left open, then start one turn. `Ok(true)` when a turn runs.
    fn start_one(&self, scope: &ExecutionScope, ep: &EpisodeId) -> ServiceResult<bool> {
        let live = self.connection(scope)?;
        let episode = self.store.episode(scope, ep)?;
        {
            let mut st = self.lock();
            if let Some(slot) = st.slots.get_mut(ep) {
                slot.origin = episode.origin;
                slot.generation = live.generation;
            }
        }
        let thread_id = self.ensure_thread(&live, scope, &episode)?;
        if let Some(open) = self.store.active_turn(scope, ep)? {
            self.settle(&live, scope, ep, &thread_id, &open, None)?;
        }
        let note = self.lock().slots.get_mut(ep).and_then(|s| s.note.take());
        let Some(ts) = self.store.begin_turn(scope, ep, note.as_deref())? else {
            return Ok(false);
        };
        (self.on_commit)(ep, ts.episode.last_seq);
        {
            let mut st = self.lock();
            if let Some(slot) = st.slots.get_mut(ep) {
                slot.turn_key = Some(ts.turn_key.clone());
                slot.runtime_turn_id = None;
                slot.thread_id = Some(thread_id.clone());
                slot.stop = None;
            }
        }
        // The tail of the context: episode-specific input goes here, after the harness prefix
        // (`developerInstructions`), so the prefix stays byte-identical and cached.
        let input: Vec<Value> = ts
            .messages
            .iter()
            .map(|(_, text)| json!({"type": "text", "text": text}))
            .collect();
        let resp = live.rpc.request(
            "turn/start",
            json!({"threadId": thread_id, "input": input}),
            self.cfg.request_timeout,
        );
        match resp {
            Ok(r) => {
                let turn_id = r
                    .pointer("/turn/id")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let stop = {
                    let mut st = self.lock();
                    match st.slots.get_mut(ep) {
                        Some(slot) if slot.turn_key.as_deref() == Some(ts.turn_key.as_str()) => {
                            if slot.runtime_turn_id.is_none() {
                                slot.runtime_turn_id = turn_id.clone();
                            }
                            slot.stop
                        }
                        // Already finished (a fast turn completes before its response is read).
                        _ => return Ok(true),
                    }
                };
                if let Some(t) = &turn_id {
                    self.store.turn_running(&ts.turn_key, t)?;
                    if stop.is_some() {
                        self.send_interrupt(&live, &thread_id, t);
                    }
                }
                Ok(true)
            }
            Err(e) => {
                let (outcome, to, reason) = if e.is_ambiguous() {
                    (
                        "ambiguous",
                        EpisodeStatus::WaitingUser,
                        format!("turn start not confirmed ({e}); its messages are not re-sent"),
                    )
                } else {
                    (
                        "rejected",
                        EpisodeStatus::Failed,
                        format!("the runtime refused the turn: {e}"),
                    )
                };
                self.store.finish_turn(&ts.turn_key, outcome)?;
                self.clear_turn(ep, &ts.turn_key);
                self.commit(
                    scope,
                    ep,
                    episode.origin,
                    vec![Mapped::Record(RuntimeRecord {
                        mark: Some(format!("turn-key:{}:settled", ts.turn_key)),
                        causation: Causation::default(),
                        change: status(to, LIVE, reason),
                    })],
                );
                Ok(false)
            }
        }
    }

    fn clear_turn(&self, ep: &EpisodeId, turn_key: &str) {
        let mut st = self.lock();
        if let Some(slot) = st.slots.get_mut(ep) {
            if slot.turn_key.as_deref() == Some(turn_key) {
                slot.turn_key = None;
                slot.runtime_turn_id = None;
                slot.stop = None;
            }
        }
    }

    fn thread_params(
        &self,
        live: &Live,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        thread: Option<&str>,
    ) -> Value {
        // Always under IGNIS's runtime state, even when a shared login home is the CODEX_HOME.
        let dir = self
            .cfg
            .home
            .join(format!(
                "ws-{}",
                &sha256_hex(scope.workspace_id.as_str().as_bytes())[..16]
            ))
            .join("episodes")
            .join(&sha256_hex(ep.as_str().as_bytes())[..24]);
        let _ = std::fs::create_dir_all(&dir);
        let profile = self.cfg.profile;
        let mut p = json!({
            "cwd": dir.to_string_lossy(),
            "sandbox": profile.sandbox(),
            "approvalPolicy": profile.approval_policy(),
        });
        if let Some(m) = &self.cfg.model {
            p["model"] = json!(m);
        }
        // The same bytes on start and on every resume, so the prefix stays cacheable and a thread
        // reloaded by a new process keeps its instructions.
        if let Some(h) = &self.cfg.harness {
            p["developerInstructions"] = json!(h.text());
        }
        // The thread's IGNIS MCP requests carry its episode grant (one per episode and process).
        if let Some(m) = &self.cfg.mcp {
            let mut leases = live.episode_mcp.lock().unwrap_or_else(|p| p.into_inner());
            let lease = leases
                .entry(ep.clone())
                .or_insert_with(|| m.grants.lease_for_episode(scope, ep));
            p["config"] = m.thread_config(lease.token());
        }
        match thread {
            Some(t) => p["threadId"] = json!(t),
            None => {
                p["serviceName"] = json!("ignis");
                p["ephemeral"] = json!(false);
            }
        }
        p
    }

    /// The episode's thread on this process: started (and recorded) the first time, otherwise
    /// resumed, replaying its finished turns so facts recorded while IGNIS was down are added once.
    fn ensure_thread(
        &self,
        live: &Arc<Live>,
        scope: &ExecutionScope,
        episode: &ResearchEpisode,
    ) -> ServiceResult<String> {
        let ep = &episode.episode_id;
        let tid = match &episode.runtime {
            None => self.start_thread(live, scope, ep, None)?,
            Some(t) => {
                let tid = t.thread_id.clone();
                self.register_thread(ep, &tid);
                if live
                    .loaded
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .contains(&tid)
                {
                    return Ok(tid);
                }
                match self.resume_rpc(live, scope, ep, &tid) {
                    Ok((thread, resp)) => {
                        self.pin_harness(scope, ep, &resp)?;
                        self.replay(scope, ep, &thread);
                        if let Some(open) = self.store.active_turn(scope, ep)? {
                            self.settle(live, scope, ep, &tid, &open, Some(&thread))?;
                        }
                        tid
                    }
                    // The runtime answered that it cannot resume this thread (its state was lost,
                    // or the thread came from elsewhere, e.g. an imported fixture): settle what is
                    // open, continue in a new thread, and say so in the log.
                    Err(CallError::Rpc { message, .. }) if thread_unknown(&message) => {
                        tracing::warn!(episode = %ep, thread = %tid, %message, "runtime cannot resume the episode's thread; starting a new one");
                        if let Some(open) = self.store.active_turn(scope, ep)? {
                            self.settle(live, scope, ep, &tid, &open, Some(&json!({"turns": []})))?;
                        }
                        let new = self.start_thread(live, scope, ep, Some(&tid))?;
                        self.add_note(ep, format!(
                            "runtime thread {tid} could not be resumed ({message}); continuing in new thread {new}, without its earlier conversation"
                        ));
                        new
                    }
                    Err(e) => return Err(call_err(e)),
                }
            }
        };
        live.loaded
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(tid.clone());
        Ok(tid)
    }

    /// `thread/start` and record the mapping (replacing `replacing` when given).
    fn start_thread(
        &self,
        live: &Arc<Live>,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        replacing: Option<&str>,
    ) -> ServiceResult<String> {
        let resp = live
            .rpc
            .request(
                "thread/start",
                self.thread_params(live, scope, ep, None),
                self.cfg.request_timeout,
            )
            .map_err(call_err)?;
        let tid = resp
            .pointer("/thread/id")
            .and_then(Value::as_str)
            .ok_or_else(|| unavailable("thread/start answered without a thread id"))?
            .to_string();
        self.register_thread(ep, &tid);
        self.store.set_runtime_thread(
            scope,
            ep,
            RuntimeThreadRef {
                runtime: RuntimeKind::CodexAppServer,
                runtime_build_id: self.manifest.runtime_build_id.clone(),
                thread_id: tid.clone(),
            },
            replacing,
        )?;
        self.pin_harness(scope, ep, &resp)?;
        Ok(tid)
    }

    /// Pin the harness on the episode with the model and effort the runtime answered with
    /// (`thread/start` / `thread/resume`); a new or changed pin is said in the next turn's status.
    fn pin_harness(
        &self,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        resp: &Value,
    ) -> ServiceResult<()> {
        let Some(h) = &self.cfg.harness else {
            return Ok(());
        };
        let model = resp.get("model").and_then(Value::as_str);
        let effort = resp.get("reasoningEffort").and_then(Value::as_str);
        let build = &self.manifest.runtime_build_id;
        if let Some(note) =
            harness::pin_episode(&self.store, scope, ep, h, build, model, effort, true)?
        {
            self.add_note(ep, note);
        }
        Ok(())
    }

    fn add_note(&self, ep: &EpisodeId, note: String) {
        if let Some(slot) = self.lock().slots.get_mut(ep) {
            slot.note = Some(match slot.note.take() {
                Some(prev) => format!("{prev}; {note}"),
                None => note,
            });
        }
    }

    fn register_thread(&self, ep: &EpisodeId, tid: &str) {
        let mut st = self.lock();
        st.threads.insert(tid.to_string(), ep.clone());
        if let Some(slot) = st.slots.get_mut(ep) {
            slot.thread_id = Some(tid.to_string());
        }
    }

    /// `thread/resume` (re-pinning the harness), then record the items of its finished turns (marks
    /// drop what is known).
    fn resume_thread(
        &self,
        live: &Arc<Live>,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        tid: &str,
    ) -> ServiceResult<Value> {
        let (thread, resp) = self.resume_rpc(live, scope, ep, tid).map_err(call_err)?;
        self.pin_harness(scope, ep, &resp)?;
        self.replay(scope, ep, &thread);
        Ok(thread)
    }

    fn resume_rpc(
        &self,
        live: &Arc<Live>,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        tid: &str,
    ) -> Result<(Value, Value), CallError> {
        let resp = live.rpc.request(
            "thread/resume",
            self.thread_params(live, scope, ep, Some(tid)),
            self.cfg.request_timeout,
        )?;
        let thread = resp.get("thread").cloned().ok_or(CallError::Rpc {
            code: -32603,
            message: "thread/resume answered without a thread".into(),
        })?;
        Ok((thread, resp))
    }

    /// Record the items of a resumed thread's finished turns (marks drop what is already known).
    fn replay(&self, scope: &ExecutionScope, ep: &EpisodeId, thread: &Value) {
        let origin = self.origin_of(ep);
        for turn in thread
            .get("turns")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if turn.get("status").and_then(Value::as_str) == Some("inProgress") {
                continue;
            }
            let turn_id = turn.get("id").and_then(Value::as_str);
            let mapped: Vec<Mapped> = turn
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .flat_map(|item| map_item(turn_id, item, true))
                .collect();
            self.commit(scope, ep, origin, mapped);
        }
    }

    /// Close an open turn this process does not run, from the runtime thread's own record.
    fn settle(
        &self,
        live: &Arc<Live>,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        tid: &str,
        open: &ActiveTurn,
        snapshot: Option<&Value>,
    ) -> ServiceResult<()> {
        if self
            .lock()
            .slots
            .get(ep)
            .is_some_and(|s| s.turn_key.as_deref() == Some(open.turn_key.as_str()))
        {
            return Ok(()); // live here
        }
        let thread = match snapshot {
            Some(t) => t.clone(),
            None => self.resume_thread(live, scope, ep, tid)?,
        };
        let origin = self.origin_of(ep);
        let found = open.runtime_turn_id.as_deref().and_then(|id| {
            thread
                .get("turns")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .find(|t| t.get("id").and_then(Value::as_str) == Some(id))
                .cloned()
        });
        match found {
            Some(t) if t.get("status").and_then(Value::as_str) == Some("inProgress") => {
                Err(ErrorEnvelope::new(
                    ErrorCode::Conflict,
                    format!(
                        "turn {} is still running in the runtime; try again when it ends",
                        open.runtime_turn_id.as_deref().unwrap_or("?")
                    ),
                    RetryPolicy::AfterDelay { seconds: 5 },
                ))
            }
            Some(t) => {
                let (_, outcome, _) = mapping::turn_outcome(&t, None);
                self.commit(scope, ep, origin, map_turn_completed(&t, None));
                self.store.finish_turn(&open.turn_key, &outcome)
            }
            None => {
                self.commit(
                    scope,
                    ep,
                    origin,
                    vec![Mapped::Record(RuntimeRecord {
                        mark: Some(format!("turn-key:{}:settled", open.turn_key)),
                        causation: Causation::default(),
                        change: status(
                            EpisodeStatus::WaitingUser,
                            LIVE,
                            "the runtime has no record of the last turn (interrupted before it was confirmed); its messages are not re-sent".into(),
                        ),
                    })],
                );
                self.store.finish_turn(&open.turn_key, "ambiguous")
            }
        }
    }

    fn origin_of(&self, ep: &EpisodeId) -> Origin {
        self.lock()
            .slots
            .get(ep)
            .map_or(Origin::System, |s| s.origin)
    }

    fn send_interrupt(&self, live: &Arc<Live>, thread: &str, turn: &str) {
        let (rpc, timeout) = (live.rpc.clone(), self.cfg.request_timeout);
        let p = json!({"threadId": thread, "turnId": turn});
        std::thread::spawn(move || {
            if let Err(e) = rpc.request("turn/interrupt", p, timeout) {
                tracing::warn!(error = %e, "turn/interrupt failed");
            }
        });
    }

    fn interrupt(&self, ep: &EpisodeId, why: StopReason) {
        let target = {
            let mut st = self.lock();
            let Some(slot) = st.slots.get_mut(ep) else {
                return;
            };
            slot.stop = Some(why);
            let (thread, turn, ws) = (
                slot.thread_id.clone(),
                slot.runtime_turn_id.clone(),
                slot.scope.workspace_id.clone(),
            );
            let live = st.conns.get(&ws).cloned();
            (live, thread, turn)
        };
        self.cancel_approvals(ep, Verdict::Host(ApprovalDecision::Cancelled));
        if let (Some(live), Some(thread), Some(turn)) = target {
            self.send_interrupt(&live, &thread, &turn);
        }
    }

    fn cancel_approvals(&self, ep: &EpisodeId, how: Verdict) {
        let mut st = self.lock();
        let ids: Vec<(EpisodeId, ApprovalId)> = st
            .approvals
            .keys()
            .filter(|(e, _)| e == ep)
            .cloned()
            .collect();
        for id in ids {
            if let Some(p) = st.approvals.remove(&id) {
                let _ = p.tx.send(match how {
                    Verdict::Host(d) => Verdict::Host(d),
                    Verdict::Withdrawn => Verdict::Withdrawn,
                });
            }
        }
    }

    // ------------------------------------------------------------------------ connections

    fn connection(&self, scope: &ExecutionScope) -> ServiceResult<Arc<Live>> {
        let ws = &scope.workspace_id;
        let live_one = |st: &State| st.conns.get(ws).filter(|l| !l.rpc.is_closed()).cloned();
        {
            let st = self.lock();
            if st.shutdown {
                return Err(unavailable("the bridge is shut down"));
            }
            if let Some(l) = live_one(&st) {
                return Ok(l);
            }
        }
        let _spawning = self.spawn_lock.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(l) = live_one(&self.lock()) {
            return Ok(l);
        }
        let home = self.ws_home(ws);
        std::fs::create_dir_all(&home).map_err(|e| unavailable(format!("runtime home: {e}")))?;
        let (tx, rx) = mpsc::channel();
        let generation = {
            let mut st = self.lock();
            st.generation += 1;
            st.generation
        };
        let label = format!("{}#{generation}", &sha256_hex(ws.as_str().as_bytes())[..8]);
        let mut cmd = self.cfg.command(&home);
        // The IGNIS tools: a loopback MCP server, reached with a token bound to this workspace's
        // host-derived scope (one process per workspace). Revoked when the process goes away.
        let lease = self.cfg.mcp.as_ref().map(|m| {
            for o in m.config_overrides() {
                cmd.arg("-c").arg(o);
            }
            let lease = m.grants.lease(scope);
            cmd.env(MCP_TOKEN_ENV, lease.token().expose());
            lease
        });
        let rpc = Connection::spawn(cmd, label, tx)
            .map_err(|e| unavailable(format!("spawn codex app-server: {e}")))?;
        let live = Arc::new(Live {
            rpc: rpc.clone(),
            generation,
            workspace: ws.clone(),
            loaded: Mutex::new(HashSet::new()),
            mcp: Mutex::new(lease),
            episode_mcp: Mutex::new(HashMap::new()),
        });
        let (weak, l) = (self.me.clone(), live.clone());
        std::thread::Builder::new()
            .name(format!("codex-dispatch-{generation}"))
            .spawn(move || dispatch(weak, l, rx))
            .map_err(|e| unavailable(format!("spawn dispatcher: {e}")))?;
        if let Err(e) = rpc.handshake(OPT_OUT, self.cfg.request_timeout) {
            rpc.kill();
            live.revoke_mcp();
            return Err(unavailable(format!("handshake: {e}")));
        }
        // API-key mode: the app-server only authenticates through an explicit login (it ignores
        // the key in its environment), so every process is logged in right after the handshake.
        if let Some(key) = self.cfg.api_key() {
            if let Err(e) = rpc.request(
                "account/login/start",
                json!({"type": "apiKey", "apiKey": key}),
                self.cfg.request_timeout,
            ) {
                rpc.kill();
                live.revoke_mcp();
                return Err(unavailable(format!(
                    "log the runtime in with the API key: {e}"
                )));
            }
        }
        let mut st = self.lock();
        if st.shutdown {
            rpc.kill();
            live.revoke_mcp();
            return Err(unavailable("the bridge is shut down"));
        }
        st.conns.insert(ws.clone(), live.clone());
        Ok(live)
    }

    fn on_closed(&self, live: &Arc<Live>, reason: &str) {
        live.revoke_mcp();
        let affected: Vec<(EpisodeId, ExecutionScope, Origin)> = {
            let mut st = self.lock();
            if st
                .conns
                .get(&live.workspace)
                .is_some_and(|l| l.generation == live.generation)
            {
                st.conns.remove(&live.workspace);
            }
            if st.shutdown {
                return;
            }
            let ids: Vec<(EpisodeId, ApprovalId)> = st
                .approvals
                .iter()
                .filter(|(_, p)| p.generation == live.generation)
                .map(|(k, _)| k.clone())
                .collect();
            for id in ids {
                if let Some(p) = st.approvals.remove(&id) {
                    let _ = p.tx.send(Verdict::Withdrawn);
                }
            }
            st.slots
                .iter_mut()
                .filter(|(_, s)| s.generation == live.generation && s.turn_key.is_some())
                .map(|(ep, s)| {
                    s.busy = false;
                    s.again = false;
                    s.turn_key = None;
                    s.runtime_turn_id = None;
                    (ep.clone(), s.scope.clone(), s.origin)
                })
                .collect()
        };
        tracing::warn!(runtime = %live.rpc.label(), %reason, episodes = affected.len(), "codex app-server connection closed");
        for (ep, scope, origin) in affected {
            self.commit(
                &scope,
                &ep,
                origin,
                vec![Mapped::Record(RuntimeRecord {
                    mark: None,
                    causation: Causation::default(),
                    change: status(
                        EpisodeStatus::WaitingUser,
                        LIVE,
                        format!("runtime connection lost ({reason}); the turn is reconciled on the next message or resume"),
                    ),
                })],
            );
        }
    }

    // ------------------------------------------------------------------------ inbound

    /// Episode, scope and origin of a thread this bridge hosts.
    fn target(&self, params: &Value) -> Option<(EpisodeId, ExecutionScope, Origin)> {
        let thread = mapping::thread_of(params)?;
        let st = self.lock();
        let ep = st.threads.get(thread)?;
        let slot = st.slots.get(ep)?;
        Some((ep.clone(), slot.scope.clone(), slot.origin))
    }

    fn on_notification(&self, live: &Arc<Live>, method: &str, params: Value) {
        let Some((ep, scope, origin)) = self.target(&params) else {
            tracing::debug!(%method, "runtime notification not attributable to a hosted thread; not recorded");
            return;
        };
        if method == "serverRequest/resolved" {
            self.withdrawn(&ep, params.get("requestId"));
            return;
        }
        let turn = mapping::turn_of(&params).map(str::to_string);
        let adopt = {
            let mut st = self.lock();
            match (st.slots.get_mut(&ep), &turn) {
                (Some(slot), Some(t))
                    if slot.turn_key.is_some() && slot.runtime_turn_id.is_none() =>
                {
                    slot.runtime_turn_id = Some(t.clone());
                    slot.turn_key.clone().map(|k| (k, t.clone()))
                }
                _ => None,
            }
        };
        if let Some((k, t)) = adopt {
            if let Err(e) = self.store.turn_running(&k, &t) {
                tracing::warn!(error = %e, "record runtime turn id");
            }
        }
        let shell = method == "item/started"
            && params.pointer("/item/type").and_then(Value::as_str) == Some("commandExecution");
        if shell && !self.cfg.profile.allows_shell() {
            let target = {
                let mut st = self.lock();
                st.slots.get_mut(&ep).and_then(|s| {
                    s.stop = Some(StopReason::Policy);
                    s.thread_id.clone().zip(s.runtime_turn_id.clone())
                })
            };
            tracing::error!(episode = %ep, "runtime started a shell command in a profile without shell; interrupting the turn");
            if let Some((thread, t)) = target {
                self.send_interrupt(live, &thread, &t);
            }
        }
        let stop = self.lock().slots.get(&ep).and_then(|s| s.stop);
        let mapped = map_notification(method, &params, stop);
        self.commit(&scope, &ep, origin, mapped);
        if method == "turn/completed" {
            if let Some(t) = params.get("turn") {
                self.turn_finished(&scope, &ep, t, stop);
            }
        }
    }

    fn turn_finished(
        &self,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        turn: &Value,
        stop: Option<StopReason>,
    ) {
        let id = turn.get("id").and_then(Value::as_str);
        let key = {
            let mut st = self.lock();
            let Some(slot) = st.slots.get_mut(ep) else {
                return;
            };
            let ours = slot.turn_key.is_some()
                && (slot.runtime_turn_id.is_none() || slot.runtime_turn_id.as_deref() == id);
            if !ours {
                return;
            }
            slot.busy = false;
            slot.runtime_turn_id = None;
            slot.stop = None;
            slot.turn_key.take()
        };
        let (_, outcome, _) = mapping::turn_outcome(turn, stop);
        if let Some(k) = key {
            if let Err(e) = self.store.finish_turn(&k, &outcome) {
                tracing::warn!(error = %e, "close runtime turn");
            }
        }
        self.cancel_approvals(ep, Verdict::Host(ApprovalDecision::Cancelled));
        // Messages that arrived during the turn (begin_turn refuses paused or ended episodes).
        if let Err(e) = self.drive(scope, ep, false) {
            tracing::debug!(error = %e, "no follow-up turn");
        }
    }

    fn on_request(&self, live: &Arc<Live>, id: Value, method: &str, params: Value) {
        let kind = match method {
            "item/commandExecution/requestApproval" => Some(ApprovalKind::Command),
            "item/fileChange/requestApproval" => Some(ApprovalKind::FileChange),
            "item/permissions/requestApproval" => Some(ApprovalKind::Permissions),
            _ => None,
        };
        let target = self.target(&params);
        match (kind, target) {
            (Some(kind), Some((ep, scope, origin))) => {
                self.approval(live, id, kind, params, ep, scope, origin)
            }
            (Some(kind), None) => {
                // Not a thread we host: fail closed.
                let _ = live.rpc.respond(
                    &id,
                    approval_answer(kind, ApprovalDecision::Denied, &params),
                );
            }
            (None, target) => {
                let sent = match method {
                    "mcpServer/elicitation/request" => {
                        live.rpc.respond(&id, json!({"action": "decline"}))
                    }
                    _ => live
                        .rpc
                        .respond_error(&id, -32601, "not supported by the IGNIS host"),
                };
                if let Err(e) = sent {
                    tracing::warn!(%method, error = %e, "answer server request");
                }
                if let Some((ep, scope, origin)) = target {
                    self.commit(
                        &scope,
                        &ep,
                        origin,
                        vec![Mapped::Opaque {
                            method: method.to_string(),
                            payload: params,
                            causation: Causation::default(),
                            mark: None,
                        }],
                    );
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn approval(
        &self,
        live: &Arc<Live>,
        rpc_id: Value,
        kind: ApprovalKind,
        params: Value,
        ep: EpisodeId,
        scope: ExecutionScope,
        origin: Origin,
    ) {
        let item = params
            .get("itemId")
            .and_then(Value::as_str)
            .unwrap_or("item");
        let sub = params.get("approvalId").and_then(Value::as_str);
        let raw = match sub {
            Some(s) => format!("apr_{item}_{s}"),
            None => format!("apr_{item}"),
        };
        let approval_id = ApprovalId::new(
            raw.chars()
                .map(|c| {
                    if c.is_whitespace() || c.is_control() {
                        '_'
                    } else {
                        c
                    }
                })
                .collect::<String>(),
        )
        .expect("non-empty id without whitespace");
        let causation = Causation {
            runtime_turn_id: mapping::turn_of(&params).map(str::to_string),
            runtime_item_id: Some(item.to_string()),
            ..Default::default()
        };
        let text = |k: &str| params.get(k).and_then(Value::as_str);
        let action = match kind {
            ApprovalKind::Command => format!(
                "run a shell command: {}",
                text("command").unwrap_or("(not shown)")
            ),
            ApprovalKind::FileChange => format!(
                "write files{}",
                text("reason")
                    .or(text("grantRoot"))
                    .map(|r| format!(": {r}"))
                    .unwrap_or_default()
            ),
            ApprovalKind::Permissions => "widen the runtime's permissions".to_string(),
        };
        let effect = kind.effect();
        let requested = Mapped::Record(RuntimeRecord {
            mark: Some(format!("approval:{approval_id}:requested")),
            causation: causation.clone(),
            change: RuntimeChange::Event(EventBody::ApprovalRequested {
                approval_id: approval_id.clone(),
                action,
                effect,
            }),
        });
        let resolved = |d: ApprovalDecision| {
            Mapped::Record(RuntimeRecord {
                mark: Some(format!("approval:{approval_id}:resolved")),
                causation: causation.clone(),
                change: RuntimeChange::Event(EventBody::ApprovalResolved {
                    approval_id: approval_id.clone(),
                    decision: d,
                }),
            })
        };
        match self.cfg.profile.approval_rule(kind) {
            ApprovalRule::Deny => {
                self.commit(
                    &scope,
                    &ep,
                    origin,
                    vec![requested, resolved(ApprovalDecision::Denied)],
                );
                if let Err(e) = live.rpc.respond(
                    &rpc_id,
                    approval_answer(kind, ApprovalDecision::Denied, &params),
                ) {
                    tracing::warn!(error = %e, "answer approval");
                }
            }
            ApprovalRule::AskHost => {
                // Registered before the request is visible, so an interrupt or a withdrawal that
                // follows the event always finds it.
                let (tx, rx) = mpsc::channel();
                self.lock().approvals.insert(
                    (ep.clone(), approval_id.clone()),
                    Pending {
                        episode: ep.clone(),
                        workspace: scope.workspace_id.clone(),
                        effect,
                        rpc_id: rpc_id.clone(),
                        generation: live.generation,
                        tx,
                    },
                );
                self.commit(
                    &scope,
                    &ep,
                    origin,
                    vec![
                        requested,
                        Mapped::Record(RuntimeRecord {
                            mark: None,
                            causation: causation.clone(),
                            change: status(
                                EpisodeStatus::WaitingApproval,
                                &[EpisodeStatus::Running, EpisodeStatus::WaitingTools],
                                format!("waiting for the host to resolve {approval_id}"),
                            ),
                        }),
                    ],
                );
                let (inner, rpc) = (self.arc(), live.rpc.clone());
                std::thread::spawn(move || {
                    inner.await_approval(
                        rx,
                        rpc,
                        rpc_id,
                        kind,
                        params,
                        approval_id,
                        causation,
                        ep,
                        scope,
                        origin,
                    )
                });
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn await_approval(
        &self,
        rx: mpsc::Receiver<Verdict>,
        rpc: Arc<Connection>,
        rpc_id: Value,
        kind: ApprovalKind,
        params: Value,
        approval_id: ApprovalId,
        causation: Causation,
        ep: EpisodeId,
        scope: ExecutionScope,
        origin: Origin,
    ) {
        let verdict = match rx.recv_timeout(self.cfg.approval_timeout) {
            Ok(v) => v,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Expired, unless a decision raced the deadline.
                let taken = self
                    .lock()
                    .approvals
                    .remove(&(ep.clone(), approval_id.clone()))
                    .is_some();
                if taken {
                    Verdict::Host(ApprovalDecision::Expired)
                } else {
                    rx.try_recv().unwrap_or(Verdict::Withdrawn)
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if self.lock().shutdown {
                    return;
                }
                Verdict::Withdrawn
            }
        };
        let (decision, answer) = match verdict {
            Verdict::Host(d) => (d, true),
            Verdict::Withdrawn => (ApprovalDecision::Cancelled, false),
        };
        let others = self.lock().approvals.values().any(|p| p.episode == ep);
        let mut records = vec![Mapped::Record(RuntimeRecord {
            mark: Some(format!("approval:{approval_id}:resolved")),
            causation: causation.clone(),
            change: RuntimeChange::Event(EventBody::ApprovalResolved {
                approval_id: approval_id.clone(),
                decision,
            }),
        })];
        if !others {
            records.push(Mapped::Record(RuntimeRecord {
                mark: None,
                causation,
                change: status(
                    EpisodeStatus::Running,
                    &[EpisodeStatus::WaitingApproval],
                    format!("{approval_id} resolved"),
                ),
            }));
        }
        self.commit(&scope, &ep, origin, records);
        if answer {
            if let Err(e) = rpc.respond(&rpc_id, approval_answer(kind, decision, &params)) {
                tracing::warn!(error = %e, "answer approval");
            }
        }
    }

    /// The runtime resolved one of its own requests (e.g. on interrupt): stop waiting for the host.
    fn withdrawn(&self, ep: &EpisodeId, request: Option<&Value>) {
        let Some(request) = request else { return };
        let mut st = self.lock();
        let id = st
            .approvals
            .iter()
            .find(|(_, p)| p.episode == *ep && &p.rpc_id == request)
            .map(|(k, _)| k.clone());
        if let Some(p) = id.and_then(|id| st.approvals.remove(&id)) {
            let _ = p.tx.send(Verdict::Withdrawn);
        }
    }

    // ------------------------------------------------------------------------ recording

    /// Record mapped items (see [`Recorder::commit`]).
    fn commit(&self, scope: &ExecutionScope, ep: &EpisodeId, origin: Origin, mapped: Vec<Mapped>) {
        self.rec.commit(scope, ep, origin, mapped);
    }
}

/// The runtime's answer to `thread/resume` says it has no such thread (lost state, or an id it
/// never issued). Any other error is not a reason to abandon the thread's history.
fn thread_unknown(message: &str) -> bool {
    // Codex 0.141.0 and 0.169.0: "no rollout found for thread id …" / "invalid session id: …".
    ["no rollout found", "invalid session id", "no such thread"]
        .iter()
        .any(|m| message.contains(m))
}

fn approval_answer(kind: ApprovalKind, decision: ApprovalDecision, params: &Value) -> Value {
    match kind {
        ApprovalKind::Command | ApprovalKind::FileChange => json!({"decision": match decision {
            ApprovalDecision::Approved => "accept",
            ApprovalDecision::Cancelled => "cancel",
            ApprovalDecision::Denied | ApprovalDecision::Expired => "decline",
        }}),
        ApprovalKind::Permissions => {
            let granted = if decision == ApprovalDecision::Approved {
                params
                    .get("permissions")
                    .cloned()
                    .unwrap_or_else(|| json!({}))
            } else {
                json!({})
            };
            json!({"permissions": granted, "scope": "turn"})
        }
    }
}

fn dispatch(weak: Weak<Inner>, live: Arc<Live>, rx: mpsc::Receiver<Inbound>) {
    let mut stash: Option<Inbound> = None;
    loop {
        let msg = match stash.take() {
            Some(m) => m,
            None => match rx.recv() {
                Ok(m) => m,
                Err(_) => return,
            },
        };
        let Some(inner) = weak.upgrade() else { return };
        match msg {
            Inbound::Notification { method, mut params } => {
                if method == "item/agentMessage/delta" {
                    // Coalesce deltas of the same item that are already queued (one event, not N).
                    let key = |p: &Value| (p.get("threadId").cloned(), p.get("itemId").cloned());
                    let k = key(&params);
                    while let Ok(next) = rx.try_recv() {
                        match next {
                            Inbound::Notification {
                                method: m2,
                                params: p2,
                            } if m2 == method && key(&p2) == k => {
                                let more = p2.get("delta").and_then(Value::as_str).unwrap_or("");
                                let joined = format!(
                                    "{}{more}",
                                    params.get("delta").and_then(Value::as_str).unwrap_or("")
                                );
                                params["delta"] = json!(joined);
                            }
                            other => {
                                stash = Some(other);
                                break;
                            }
                        }
                    }
                }
                inner.on_notification(&live, &method, params);
            }
            Inbound::Request { id, method, params } => inner.on_request(&live, id, &method, params),
            Inbound::Closed { reason } => {
                inner.on_closed(&live, &reason);
                return;
            }
        }
    }
}

//! [`ClaudeBridge`]: the [`AgentRuntime`] over Claude Code in headless mode (ADR 17).
//!
//! - One `claude -p` process per turn, `--output-format stream-json`. The Claude session id (a UUID
//!   IGNIS chooses with `--session-id`, then `--resume`s) is the episode's runtime thread.
//! - The harness bundle (M6) is appended to Claude Code's system prompt from a file named by its
//!   digest, identical for every turn, and pinned on the episode.
//! - Claude Code owns the model/tool loop. Its built-in tools are off (`--tools ""`), user/project
//!   settings and other MCP servers are ignored; the only tools are the IGNIS MCP server's read and
//!   compute tools, pre-allowed, so no approval is ever requested (`--permission-mode dontAsk`
//!   denies anything else).
//! - Each workspace has its own config directory (sessions, credentials) unless a shared login home
//!   is configured. The child gets a cleared environment: the forwarded credential variables, the
//!   MCP token of this turn, and the few variables the OS needs to start a process.
//! - stream-json lines are translated into the item shapes of [`super::mapping`] and recorded through
//!   the shared [`Recorder`], so both runtimes produce the same events.
//! - Interrupt kills the turn's process. A turn left open by a previous host process cannot be
//!   reconciled from the outside and is settled as interrupted; its messages are not re-sent.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use ignis_core::contracts::{
    ApprovalDecision, ApprovalId, Causation, CostBasis, CostFigure, EpisodeId, EpisodeStatus,
    ErrorCode, ErrorEnvelope, EventBody, ExecutionScope, Finite, Origin, RetryPolicy,
    RuntimeBuildId, RuntimeKind, RuntimeThreadRef, Timestamp, UsageAccounting, UsageReport,
};
use ignis_core::hashing::sha256_hex;
use ignis_core::services::{ArtifactStore, ServiceResult};
use serde_json::{json, Value};

use super::bridge::CommitHook;
use super::grants::{McpAttachment, MCP_TOKEN_ENV};
use super::harness::{self, HarnessBundle};
use super::mapping::{
    self, map_item, map_turn_completed, Mapped, Preamble, StopReason, IGNIS_MCP_SERVER,
};
use super::preflight::{
    sha256_file, RuntimeBuildManifest, RuntimeCredentials, RuntimeProfile, LIVE,
};
use super::recorder::Recorder;
use super::AgentRuntime;
use crate::store::{RuntimeChange, RuntimeRecord};
use crate::ResearchStore;

/// The fixed part of every `claude` invocation; part of the build id, so a change is visible.
const FIXED_ARGS: &[&str] = &[
    "-p",
    "--output-format",
    "stream-json",
    "--verbose",
    "--tools",
    "",
    "--strict-mcp-config",
    "--setting-sources",
    "",
    "--disable-slash-commands",
    "--permission-mode",
    "dontAsk",
    // cwd, environment and similar per-machine sections go to the first user message, so the
    // system prompt (default + harness) is the same bytes for every episode and cacheable across them.
    "--exclude-dynamic-system-prompt-sections",
];

/// Claude Code's native subagent tool. With delegation on it is the one built-in tool enabled.
pub const DELEGATION_TOOL: &str = "Agent";
/// Names of the subagent tool across Claude Code versions (`Task` before the rename).
const DELEGATION_TOOLS: &[&str] = &["Agent", "Task"];
/// The one subagent IGNIS defines (`--agents`).
pub const SUBAGENT_NAME: &str = "ignis-subinvestigator";
/// IGNIS tools a subagent never gets: the answer is the investigator's alone.
const PARENT_ONLY_TOOLS: &[&str] = &["answer_submit"];
const SUBAGENT_DESCRIPTION: &str = "Hand ONE bounded, self-contained subtask to a \
sub-investigator with the same IGNIS tools and scope, e.g. an aggregation over many runs computed \
with analysis_execute, or a source survey of one sub-question. It returns a report whose findings \
cite the ids its tools returned. Not for a single lookup or a single tool call: call that tool \
directly.";
const SUBAGENT_PROMPT: &str = "You are an IGNIS sub-investigator. The investigator gave you one \
bounded subtask; do only that. Use only the IGNIS tools. Every finding cites the ids a tool \
returned to you (evidence ids, artifact ids with a JSON Pointer, job ids, locators) exactly as \
returned; a value without such an id is not a finding. Mount data into analysis_execute by \
reference instead of copying it. You cannot delegate further and you never submit the answer. End \
with one report: (1) what you did, (2) findings, each with its supporting ids, (3) what you could \
not establish and why (refusals, missing, conflicting or not-applicable data).";

/// Whether this configuration runs with depth-1 delegation (it needs the IGNIS tools).
fn delegates(cfg: &ClaudeConfig) -> bool {
    cfg.delegation && cfg.mcp.is_some()
}

/// The fixed arguments of this configuration: [`FIXED_ARGS`] with the subagent tool enabled when
/// delegation is on.
fn fixed_args(cfg: &ClaudeConfig) -> Vec<String> {
    let mut a: Vec<String> = FIXED_ARGS.iter().map(|s| s.to_string()).collect();
    if delegates(cfg) {
        let i = a.iter().position(|x| x == "--tools").expect("--tools") + 1;
        a[i] = DELEGATION_TOOL.into();
    }
    a
}

/// What the build id covers: the fixed arguments and, with delegation, the subagent tool and its
/// definition. It follows the configured switch (preflight runs before the MCP server is attached).
fn build_args(cfg: &ClaudeConfig) -> Vec<String> {
    let mut a: Vec<String> = FIXED_ARGS.iter().map(|s| s.to_string()).collect();
    if cfg.delegation {
        let i = a.iter().position(|x| x == "--tools").expect("--tools") + 1;
        a[i] = DELEGATION_TOOL.into();
        a.push(format!("agents:{SUBAGENT_NAME}"));
        a.push(sha256_hex(
            format!("{SUBAGENT_DESCRIPTION}\u{1f}{SUBAGENT_PROMPT}").as_bytes(),
        ));
    }
    a
}

/// The `--agents` definition: one subagent with the IGNIS tools the investigator may call without
/// asking, minus the parent-only ones. Its tool list has no subagent tool, so it cannot delegate
/// (Claude Code gives subagents none either, and the translator checks it).
fn subagents(approved_tools: &[String]) -> Value {
    let tools: Vec<String> = approved_tools
        .iter()
        .filter(|t| !PARENT_ONLY_TOOLS.contains(&t.as_str()))
        .map(|t| format!("mcp__{IGNIS_MCP_SERVER}__{t}"))
        .collect();
    json!({SUBAGENT_NAME: {
        "description": SUBAGENT_DESCRIPTION,
        "prompt": SUBAGENT_PROMPT,
        "tools": tools,
    }})
}

/// Variables a process needs to start on the host OS (no secrets); everything else is cleared.
const OS_ENV: &[&str] = &[
    "PATH",
    "LANG",
    "LC_ALL",
    "TZ",
    "SYSTEMROOT",
    "TEMP",
    "TMP",
    "COMSPEC",
];

/// stderr kept for the failure reason of a turn.
const STDERR_TAIL: usize = 4096;
/// Token cap Claude Code applies to one MCP tool result. The IGNIS host inlines at most 64 KiB of
/// JSON, carried twice (structured content and its text form), so even counted twice an inline
/// result stays below this; the default (25,000) would cut a full `sources_read` page.
const MAX_MCP_OUTPUT_TOKENS: &str = "80000";

/// Configuration of the Claude Code runtime.
#[derive(Clone)]
pub struct ClaudeConfig {
    /// Native `claude` executable.
    pub binary: PathBuf,
    /// Runtime state root: `<home>/ws-<hash>/` is one workspace's config directory and episode cwds.
    pub home: PathBuf,
    pub model: Option<String>,
    /// Environment variables forwarded to the child: the credentials (a plan token from
    /// `claude setup-token`, or an Anthropic API key).
    pub pass_env: Vec<String>,
    /// One config directory signed in with `claude /login`, shared by every workspace instead of an
    /// empty one each (their sessions then share that directory).
    pub login_home: Option<PathBuf>,
    pub mcp: Option<McpAttachment>,
    /// Longest a turn may run before it is stopped (a resource budget, not a semantic limit).
    pub turn_timeout: Duration,
    /// The harness bundle appended to Claude Code's system prompt (`--append-system-prompt-file`).
    /// `None`: no IGNIS instructions, and no harness pin on the episode.
    pub harness: Option<super::harness::HarnessBundle>,
    /// Depth-1 delegation (M9): the investigator may hand one bounded subtask to Claude Code's
    /// native subagent tool. The subagent has the same IGNIS tools over the same MCP token (so the
    /// same host-derived scope) minus `answer_submit`, cannot delegate further, and runs inside
    /// this turn's budget; its tool calls are recorded in the parent episode. Off by default (an
    /// ablation arm); the build id says which.
    pub delegation: bool,
}

impl ClaudeConfig {
    pub fn new(binary: impl Into<PathBuf>, home: impl Into<PathBuf>) -> Self {
        ClaudeConfig {
            binary: binary.into(),
            home: home.into(),
            model: None,
            pass_env: vec!["CLAUDE_CODE_OAUTH_TOKEN".into(), "ANTHROPIC_API_KEY".into()],
            login_home: None,
            mcp: None,
            turn_timeout: Duration::from_secs(60 * 60),
            harness: None,
            delegation: false,
        }
    }

    fn credentials(&self) -> RuntimeCredentials {
        if self.login_home.is_some() {
            RuntimeCredentials::SharedLoginHome
        } else {
            RuntimeCredentials::Environment
        }
    }
}

/// Check the binary and that some credential is configured. `Err` is a reason for `/capabilities`.
pub fn claude_preflight(cfg: &ClaudeConfig) -> Result<RuntimeBuildManifest, String> {
    let bin = &cfg.binary;
    let has_env = cfg
        .pass_env
        .iter()
        .any(|k| std::env::var_os(k).is_some_and(|v| !v.is_empty()));
    match &cfg.login_home {
        Some(h) if !h.is_dir() => {
            return Err(format!("no Claude login directory at {}", h.display()));
        }
        None if !has_env => {
            return Err(format!(
                "no Claude credential: set one of {} (a plan token from `claude setup-token`, or an API key), or IGNIS_CLAUDE_LOGIN_HOME",
                cfg.pass_env.join(", ")
            ));
        }
        _ => {}
    }
    let out = Command::new(bin)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("run {} --version: {e}", bin.display()))?;
    if !out.status.success() {
        return Err(format!(
            "{} --version failed: {}",
            bin.display(),
            out.status
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let version = text
        .split_whitespace()
        .next()
        .filter(|v| v.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .ok_or_else(|| format!("unexpected `claude --version` output: {}", text.trim()))?
        .to_string();
    let binary_sha256 = sha256_file(bin).map_err(|e| format!("hash {}: {e}", bin.display()))?;
    let fixed = build_args(cfg);
    let args_sha256 = sha256_hex(fixed.join("\u{1f}").as_bytes());
    let id = format!(
        "claude-code-{version}+{}.{}",
        &binary_sha256[..12],
        &args_sha256[..12]
    );
    Ok(RuntimeBuildManifest {
        runtime_build_id: RuntimeBuildId::new(id).map_err(|e| e.to_string())?,
        runtime: RuntimeKind::ClaudeCode,
        user_agent: format!("claude-code/{version}"),
        version,
        binary_sha256,
        binary_hash_pinned: false,
        protocol_schema_sha256: args_sha256,
        transport: "stdio stream-json (one process per turn)".into(),
        profile: RuntimeProfile::Public,
        config_overrides: fixed,
        model: cfg.model.clone(),
        forwarded_env: cfg.pass_env.clone(),
        credentials: cfg.credentials(),
        preflight_at: Timestamp::now(),
    })
}

// ------------------------------------------------------------------------------ translation

/// Claude Code stream-json → the item shapes [`super::mapping`] records. One per turn.
pub struct Translator {
    turn: String,
    /// tool_use id → (server, tool)
    tools: HashMap<String, (String, String)>,
    /// Text the model wrote right before a `tool_use` becomes that call's `step.rationale`.
    preamble: Preamble,
    /// Set when a subagent tried to delegate further (depth 1 is the limit): the bridge stops the
    /// turn with this reason.
    depth_violation: Option<String>,
    /// The model the `system`/`init` line named (for a `result` without `modelUsage`).
    model: Option<String>,
}

/// `mcp__<server>__<tool>` → (server, tool); anything else is reported under server `claude`.
fn split_tool(name: &str) -> (String, String) {
    name.strip_prefix("mcp__")
        .and_then(|rest| rest.split_once("__"))
        .map(|(s, t)| (s.to_string(), t.to_string()))
        .unwrap_or_else(|| ("claude".to_string(), name.to_string()))
}

fn result_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|p| p.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

impl Translator {
    pub fn new(turn: impl Into<String>) -> Self {
        Translator {
            turn: turn.into(),
            tools: HashMap::new(),
            preamble: Preamble::default(),
            depth_violation: None,
            model: None,
        }
    }

    /// The `usage.reported` records of a `result` line: one increment per model of `modelUsage`
    /// (with the cost Claude Code reports), else one from `usage` for the model `init` named.
    /// Anthropic counts cache reads and writes apart from `input_tokens`; the report folds them in.
    fn usage(&self, v: &Value) -> Vec<Mapped> {
        fn n(o: &Value, k: &str) -> Option<u64> {
            o.get(k).and_then(Value::as_u64)
        }
        fn report(
            model: Option<String>,
            o: &Value,
            camel: bool,
            cost: Option<f64>,
        ) -> Option<UsageReport> {
            let key =
                |snake: &'static str, camel_k: &'static str| if camel { camel_k } else { snake };
            let read = n(o, key("cache_read_input_tokens", "cacheReadInputTokens"));
            let write = n(
                o,
                key("cache_creation_input_tokens", "cacheCreationInputTokens"),
            );
            let usage = UsageReport {
                accounting: UsageAccounting::Increment,
                model,
                input_tokens: n(o, key("input_tokens", "inputTokens"))?
                    .checked_add(read.unwrap_or(0))?
                    .checked_add(write.unwrap_or(0))?,
                cached_input_tokens: read,
                cache_write_tokens: write,
                output_tokens: n(o, key("output_tokens", "outputTokens"))?,
                reasoning_tokens: None,
                reported_cost: cost
                    .and_then(|a| Finite::new(a).ok())
                    .map(|amount| CostFigure {
                        amount,
                        currency: "USD".into(),
                        basis: CostBasis::ProviderReported,
                        price_list: None,
                    }),
            };
            usage.validate().is_ok().then_some(usage)
        }
        let mut reports = vec![];
        if let Some(per_model) = v.get("modelUsage").and_then(Value::as_object) {
            for (model, o) in per_model {
                let cost = o.get("costUSD").and_then(Value::as_f64);
                reports.extend(report(Some(model.clone()), o, true, cost));
            }
        } else if let Some(o) = v.get("usage") {
            let cost = v.get("total_cost_usd").and_then(Value::as_f64);
            reports.extend(report(self.model.clone(), o, false, cost));
        }
        reports
            .into_iter()
            .map(|usage| {
                let mark = format!(
                    "usage:{}:{}",
                    self.turn,
                    usage.model.as_deref().unwrap_or("unreported")
                );
                Mapped::Record(RuntimeRecord {
                    mark: Some(mark),
                    causation: Causation {
                        runtime_turn_id: Some(self.turn.clone()),
                        ..Default::default()
                    },
                    change: RuntimeChange::Event(EventBody::UsageReported { usage }),
                })
            })
            .collect()
    }

    /// Why the turn must stop because a subagent tried to delegate, once that happened.
    pub fn depth_violation(&self) -> Option<&str> {
        self.depth_violation.as_deref()
    }

    /// Map one stream-json line. The `result` line is returned separately; it ends the turn.
    pub fn line(&mut self, v: &Value) -> (Vec<Mapped>, Option<Value>) {
        let (mapped, result) = self.translate(v);
        (self.preamble.link(mapped), result)
    }

    fn translate(&mut self, v: &Value) -> (Vec<Mapped>, Option<Value>) {
        let turn = Some(self.turn.as_str());
        match v.get("type").and_then(Value::as_str).unwrap_or("") {
            "assistant" => {
                let msg = v.get("message").cloned().unwrap_or(Value::Null);
                let mid = msg
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("msg")
                    .to_string();
                // A subagent's line names the subagent tool call it runs under. Its tool calls are
                // recorded like the investigator's (same episode, same scope); its text is archived
                // under that call, never as an assistant message of the episode.
                let parent = v.get("parent_tool_use_id").and_then(Value::as_str);
                let mut out = vec![];
                for (i, block) in msg
                    .get("content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    match block.get("type").and_then(Value::as_str) {
                        Some("text") => {
                            let text = block.get("text").and_then(Value::as_str).unwrap_or("");
                            if text.is_empty() {
                                continue;
                            }
                            if let Some(p) = parent {
                                out.push(Mapped::Opaque {
                                    method: "claude/subagent_message".into(),
                                    payload: json!({"parent_tool_use_id": p, "text": text}),
                                    causation: Causation {
                                        runtime_turn_id: Some(self.turn.clone()),
                                        runtime_item_id: Some(format!("{mid}-{i}")),
                                        ..Default::default()
                                    },
                                    mark: Some(format!("subagent:{mid}-{i}")),
                                });
                                continue;
                            }
                            out.extend(map_item(
                                turn,
                                &json!({"id": format!("{mid}-{i}"), "type": "agentMessage", "text": text}),
                                true,
                            ));
                        }
                        Some("tool_use") => {
                            let Some(id) = block.get("id").and_then(Value::as_str) else {
                                continue;
                            };
                            let name = block.get("name").and_then(Value::as_str).unwrap_or("");
                            if parent.is_some() && DELEGATION_TOOLS.contains(&name) {
                                self.depth_violation.get_or_insert_with(|| {
                                    format!(
                                        "a subagent tried to delegate further ({name}); delegation is depth 1"
                                    )
                                });
                            }
                            let (server, tool) = split_tool(name);
                            self.tools
                                .insert(id.to_string(), (server.clone(), tool.clone()));
                            out.extend(map_item(
                                turn,
                                &json!({"id": id, "type": "mcpToolCall", "server": server,
                                        "tool": tool, "status": "inProgress",
                                        "arguments": block.get("input").cloned().unwrap_or(Value::Null)}),
                                false,
                            ));
                        }
                        // Thinking and other blocks carry no event (and are never a rationale).
                        _ => {}
                    }
                }
                (out, None)
            }
            "user" => {
                let mut out = vec![];
                for block in v
                    .pointer("/message/content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter(|b| b.get("type").and_then(Value::as_str) == Some("tool_result"))
                {
                    let Some(id) = block.get("tool_use_id").and_then(Value::as_str) else {
                        continue;
                    };
                    let (server, tool) = self
                        .tools
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| ("claude".into(), "unknown".into()));
                    let text = result_text(block.get("content").unwrap_or(&Value::Null));
                    let failed = block
                        .get("is_error")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    // The IGNIS server's results are ToolResult JSON; keep anything else as text.
                    let value = serde_json::from_str::<Value>(&text)
                        .unwrap_or_else(|_| json!({ "text": text }));
                    let mut item = json!({"id": id, "type": "mcpToolCall", "server": server,
                                          "tool": tool,
                                          "status": if failed { "failed" } else { "completed" },
                                          "result": {"structuredContent": value}});
                    if failed {
                        item["error"] = json!({ "message": text });
                    }
                    out.extend(map_item(turn, &item, true));
                }
                (out, None)
            }
            "result" => (self.usage(v), Some(v.clone())),
            "system" => {
                if v.get("subtype").and_then(Value::as_str) == Some("init") {
                    if let Some(m) = v.get("model").and_then(Value::as_str) {
                        self.model = Some(m.to_string());
                    }
                }
                // Say so when the IGNIS server did not connect: the turn then has no tools.
                let ignis_down = v
                    .get("mcp_servers")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .any(|s| {
                        s.get("name").and_then(Value::as_str) == Some(IGNIS_MCP_SERVER)
                            && s.get("status").and_then(Value::as_str) != Some("connected")
                    });
                if ignis_down {
                    (
                        vec![Mapped::Opaque {
                            method: "claude/system".into(),
                            payload: v.clone(),
                            causation: Causation {
                                runtime_turn_id: Some(self.turn.clone()),
                                ..Default::default()
                            },
                            mark: None,
                        }],
                        None,
                    )
                } else {
                    (vec![], None)
                }
            }
            // Partial messages and rate-limit notices are repeated in the lines above or the result.
            "stream_event" | "rate_limit_event" => (vec![], None),
            other => (
                vec![Mapped::Opaque {
                    method: format!("claude/{other}"),
                    payload: v.clone(),
                    causation: Causation {
                        runtime_turn_id: Some(self.turn.clone()),
                        ..Default::default()
                    },
                    mark: None,
                }],
                None,
            ),
        }
    }
}

/// The Codex-shaped turn snapshot of a finished turn, for [`map_turn_completed`].
pub fn turn_snapshot(turn: &str, result: Option<&Value>, exit: &str, stderr: &str) -> Value {
    match result {
        Some(r) => {
            let ok = r.get("subtype").and_then(Value::as_str) == Some("success")
                && !r.get("is_error").and_then(Value::as_bool).unwrap_or(false);
            if ok {
                json!({"id": turn, "status": "completed"})
            } else {
                let msg = r
                    .get("result")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .or_else(|| r.get("subtype").and_then(Value::as_str))
                    .unwrap_or("error");
                json!({"id": turn, "status": "failed", "error": {"message": msg}})
            }
        }
        None => json!({"id": turn, "status": "failed", "error": {"message":
            format!("Claude Code exited ({exit}) without a result: {}", stderr.trim())}}),
    }
}

// ------------------------------------------------------------------------------ bridge

pub struct ClaudeBridge {
    inner: Arc<Inner>,
}

struct Inner {
    cfg: ClaudeConfig,
    manifest: RuntimeBuildManifest,
    store: Arc<ResearchStore>,
    rec: Recorder,
    on_commit: CommitHook,
    state: Mutex<State>,
    me: Weak<Inner>,
}

#[derive(Default)]
struct State {
    slots: HashMap<EpisodeId, Slot>,
    shutdown: bool,
}

#[derive(Default)]
struct Slot {
    /// A driver thread is running this episode's turns.
    busy: bool,
    /// Work was requested since the driver last looked.
    dirty: bool,
    /// Turn key of the turn this process runs.
    turn_key: Option<String>,
    child: Option<Arc<Mutex<Child>>>,
    stop: Option<StopReason>,
    origin: Option<Origin>,
}

fn unavailable(msg: impl std::fmt::Display) -> ErrorEnvelope {
    ErrorEnvelope::new(
        ErrorCode::CapabilityUnavailable,
        format!("agent runtime unavailable: {msg}"),
        RetryPolicy::AfterDelay { seconds: 5 },
    )
}

impl ClaudeBridge {
    pub fn new(
        cfg: ClaudeConfig,
        manifest: RuntimeBuildManifest,
        store: Arc<ResearchStore>,
        artifacts: Arc<dyn ArtifactStore>,
        on_commit: CommitHook,
    ) -> Self {
        let rec = Recorder {
            store: store.clone(),
            artifacts,
            on_commit: on_commit.clone(),
        };
        let inner = Arc::new_cyclic(|me| Inner {
            cfg,
            manifest,
            store,
            rec,
            on_commit,
            state: Mutex::new(State::default()),
            me: me.clone(),
        });
        ClaudeBridge { inner }
    }

    /// Stop every running turn without recording anything (open turns are settled on the next start).
    pub fn shutdown(&self) {
        let children: Vec<_> = {
            let mut st = self.inner.lock();
            st.shutdown = true;
            st.slots.values().filter_map(|s| s.child.clone()).collect()
        };
        for c in children {
            let _ = c.lock().unwrap_or_else(|p| p.into_inner()).kill();
        }
    }
}

impl Drop for ClaudeBridge {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl AgentRuntime for ClaudeBridge {
    fn manifest(&self) -> &RuntimeBuildManifest {
        &self.inner.manifest
    }

    fn kick(&self, scope: &ExecutionScope, episode: &EpisodeId) -> ServiceResult<()> {
        self.inner.drive(scope, episode)
    }

    fn interrupt(
        &self,
        _scope: &ExecutionScope,
        episode: &EpisodeId,
        why: StopReason,
    ) -> ServiceResult<()> {
        let child = {
            let mut st = self.inner.lock();
            let Some(slot) = st.slots.get_mut(episode) else {
                return Ok(());
            };
            if slot.turn_key.is_none() {
                return Ok(());
            }
            slot.stop = Some(why);
            slot.child.clone()
        };
        if let Some(c) = child {
            let _ = c.lock().unwrap_or_else(|p| p.into_inner()).kill();
        }
        Ok(())
    }

    fn resume(&self, scope: &ExecutionScope, episode: &EpisodeId) -> ServiceResult<()> {
        self.inner.drive(scope, episode)
    }

    fn resolve_approval(
        &self,
        _scope: &ExecutionScope,
        _episode: &EpisodeId,
        approval: &ApprovalId,
        _decision: ApprovalDecision,
    ) -> ServiceResult<()> {
        Err(ErrorEnvelope::new(
            ErrorCode::NotFound,
            format!(
                "no pending approval {approval}: the Claude Code runtime is offered only tools that need none"
            ),
            RetryPolicy::Never,
        ))
    }

    fn recover(&self) -> ServiceResult<()> {
        for open in self.inner.store.active_turns()? {
            self.inner.settle_foreign(&open.scope, &open.episode_id)?;
            self.inner.drive(&open.scope, &open.episode_id)?;
        }
        Ok(())
    }
}

impl Inner {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn ws_dir(&self, scope: &ExecutionScope) -> PathBuf {
        self.cfg.home.join(format!(
            "ws-{}",
            &sha256_hex(scope.workspace_id.as_str().as_bytes())[..16]
        ))
    }

    /// Start a driver for the episode unless one runs. Every call marks the episode dirty, and the
    /// driver runs turns until a pass finds it clean, so a message posted while the last turn ends
    /// is never left waiting.
    fn drive(&self, scope: &ExecutionScope, ep: &EpisodeId) -> ServiceResult<()> {
        let episode = self.store.episode(scope, ep)?;
        {
            let mut st = self.lock();
            if st.shutdown {
                return Err(unavailable("the runtime is shut down"));
            }
            let slot = st.slots.entry(ep.clone()).or_default();
            slot.origin = Some(episode.origin);
            slot.dirty = true;
            if slot.busy {
                return Ok(());
            }
            slot.busy = true;
        }
        let me = self.me.upgrade().expect("bridge alive while in use");
        let (scope, ep) = (scope.clone(), ep.clone());
        std::thread::Builder::new()
            .name("ignis-claude-turns".into())
            .spawn(move || loop {
                {
                    let mut st = me.lock();
                    let shutdown = st.shutdown;
                    let slot = st.slots.entry(ep.clone()).or_default();
                    if !slot.dirty || shutdown {
                        slot.busy = false;
                        return;
                    }
                    slot.dirty = false;
                }
                loop {
                    match me.start_one(&scope, &ep) {
                        Ok(true) => continue,
                        Ok(false) => break,
                        Err(e) => {
                            tracing::warn!(episode = %ep, error = %e, "claude turn");
                            break;
                        }
                    }
                }
            })
            .map_err(|e| unavailable(format!("spawn turn thread: {e}")))?;
        Ok(())
    }

    /// A turn opened by another host process: settle it as interrupted (never re-sent).
    fn settle_foreign(&self, scope: &ExecutionScope, ep: &EpisodeId) -> ServiceResult<()> {
        let Some(open) = self.store.active_turn(scope, ep)? else {
            return Ok(());
        };
        if self
            .lock()
            .slots
            .get(ep)
            .is_some_and(|s| s.turn_key.as_deref() == Some(open.turn_key.as_str()))
        {
            return Ok(()); // live here
        }
        let origin = self.store.episode(scope, ep)?.origin;
        self.rec.commit(
            scope,
            ep,
            origin,
            vec![Mapped::Record(RuntimeRecord {
                mark: Some(format!("turn-key:{}:settled", open.turn_key)),
                causation: Causation::default(),
                change: RuntimeChange::Status {
                    to: EpisodeStatus::WaitingUser,
                    reason: Some(
                        "the previous host process ended during this turn; its messages are not re-sent"
                            .into(),
                    ),
                    from_any: LIVE.to_vec(),
                },
            })],
        );
        self.store.finish_turn(&open.turn_key, "ambiguous")
    }

    /// The episode's Claude session: the recorded one, or a new UUID recorded now (also when the
    /// episode was started on another runtime: its earlier conversation is not carried over).
    fn session(&self, scope: &ExecutionScope, ep: &EpisodeId) -> ServiceResult<(String, bool)> {
        let episode = self.store.episode(scope, ep)?;
        match &episode.runtime {
            Some(t) if t.runtime == RuntimeKind::ClaudeCode => Ok((t.thread_id.clone(), false)),
            other => {
                let sid = uuid::Uuid::new_v4().to_string();
                self.store.set_runtime_thread(
                    scope,
                    ep,
                    RuntimeThreadRef {
                        runtime: RuntimeKind::ClaudeCode,
                        runtime_build_id: self.manifest.runtime_build_id.clone(),
                        thread_id: sid.clone(),
                    },
                    other.as_ref().map(|t| t.thread_id.as_str()),
                )?;
                Ok((sid, true))
            }
        }
    }

    /// The bundle as a file named by its digest (written once, atomically): the same bytes for
    /// every turn, and no command-line length limit.
    fn harness_file(&self, ws: &std::path::Path, h: &HarnessBundle) -> ServiceResult<PathBuf> {
        let dir = ws.join("harness");
        let path = dir.join(format!("{}.md", h.digest().hex()));
        if path.is_file() {
            return Ok(path);
        }
        let tmp = dir.join(format!(".{}.tmp", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&dir)
            .and_then(|()| std::fs::write(&tmp, h.text().as_bytes()))
            .and_then(|()| std::fs::rename(&tmp, &path))
            .or_else(|e| if path.is_file() { Ok(()) } else { Err(e) })
            .map_err(|e| unavailable(format!("write harness {}: {e}", path.display())))?;
        let _ = std::fs::remove_file(&tmp);
        Ok(path)
    }

    fn command(
        &self,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        session: &str,
        new: bool,
        mcp_token: Option<&str>,
    ) -> ServiceResult<Command> {
        let ws = self.ws_dir(scope);
        let config_dir = self
            .cfg
            .login_home
            .clone()
            .unwrap_or_else(|| ws.join("config"));
        let cwd = ws
            .join("episodes")
            .join(&sha256_hex(ep.as_str().as_bytes())[..24]);
        for d in [&config_dir, &cwd] {
            std::fs::create_dir_all(d)
                .map_err(|e| unavailable(format!("create {}: {e}", d.display())))?;
        }
        let mut cmd = Command::new(&self.cfg.binary);
        cmd.args(fixed_args(&self.cfg));
        if new {
            cmd.args(["--session-id", session]);
        } else {
            cmd.args(["--resume", session]);
        }
        if let Some(m) = &self.cfg.model {
            cmd.args(["--model", m]);
        }
        if let Some(h) = &self.cfg.harness {
            cmd.arg("--append-system-prompt-file")
                .arg(self.harness_file(&ws, h)?);
        }
        cmd.env_clear();
        for k in OS_ENV {
            if let Some(v) = std::env::var_os(k) {
                cmd.env(k, v);
            }
        }
        // Claude Code truncates an MCP tool result above MAX_MCP_OUTPUT_TOKENS (25,000 by
        // default), and its built-in tools are off, so the agent could not read a truncated
        // remainder any other way. Results up to the host's 64 KiB inline limit (sent as both
        // structured content and text) fit well within this; larger ones arrive as an artifact
        // reference. An operator may still override it through `pass_env`.
        cmd.env("MAX_MCP_OUTPUT_TOKENS", MAX_MCP_OUTPUT_TOKENS);
        for k in &self.cfg.pass_env {
            if let Some(v) = std::env::var_os(k).filter(|v| !v.is_empty()) {
                cmd.env(k, v);
            }
        }
        cmd.env("HOME", &config_dir)
            .env("USERPROFILE", &config_dir)
            .env("CLAUDE_CONFIG_DIR", &config_dir)
            .env("DISABLE_AUTOUPDATER", "1")
            .env("DISABLE_TELEMETRY", "1");
        if let (Some(m), Some(token)) = (&self.cfg.mcp, mcp_token) {
            // The token stays in the environment; the config only names the variable.
            let config = json!({"mcpServers": {IGNIS_MCP_SERVER: {
                "type": "http",
                "url": m.url,
                "headers": {"Authorization": format!("Bearer ${{{MCP_TOKEN_ENV}}}")},
            }}});
            let mut allowed: Vec<String> = m
                .approved_tools
                .iter()
                .map(|t| format!("mcp__{IGNIS_MCP_SERVER}__{t}"))
                .collect();
            cmd.arg("--mcp-config").arg(config.to_string());
            if delegates(&self.cfg) {
                cmd.arg("--agents")
                    .arg(subagents(&m.approved_tools).to_string());
                allowed.push(DELEGATION_TOOL.into());
            }
            if !allowed.is_empty() {
                cmd.arg("--allowedTools").arg(allowed.join(","));
            }
            cmd.env(MCP_TOKEN_ENV, token);
        }
        cmd.current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        Ok(cmd)
    }

    /// Run one turn for the pending messages. `Ok(false)`: nothing to do.
    fn start_one(&self, scope: &ExecutionScope, ep: &EpisodeId) -> ServiceResult<bool> {
        self.settle_foreign(scope, ep)?;
        let origin = self.store.episode(scope, ep)?.origin;
        let (session, new) = self.session(scope, ep)?;
        // A resumed session keeps the system prompt it started with (Claude Code's prompt
        // snapshot), so a different bundle is not pinned on it.
        let note = match &self.cfg.harness {
            Some(h) => harness::pin_episode(
                &self.store,
                scope,
                ep,
                h,
                &self.manifest.runtime_build_id,
                self.cfg.model.as_deref(),
                None,
                new,
            )?,
            None => None,
        };
        let Some(ts) = self.store.begin_turn(scope, ep, note.as_deref())? else {
            return Ok(false);
        };
        (self.on_commit)(ep, ts.episode.last_seq);
        let turn = format!("claude-turn-{}", uuid::Uuid::new_v4().simple());
        self.store.turn_running(&ts.turn_key, &turn)?;
        // One process per turn of one episode: its token is bound to the episode.
        let lease = self
            .cfg
            .mcp
            .as_ref()
            .map(|m| m.grants.lease_for_episode(scope, ep));
        let spawned = self
            .command(
                scope,
                ep,
                &session,
                new,
                lease.as_ref().map(|l| l.token().expose()),
            )
            .and_then(|mut c| {
                c.spawn()
                    .map_err(|e| unavailable(format!("start {}: {e}", self.cfg.binary.display())))
            });
        let mut child = match spawned {
            Ok(c) => c,
            Err(e) => {
                self.finish(
                    scope,
                    ep,
                    origin,
                    &ts.turn_key,
                    &turn,
                    None,
                    "not started",
                    &e.message,
                );
                return Ok(true);
            }
        };
        // The tail of the context: episode-specific input goes here, after the harness prefix
        // (the appended system prompt), so the prefix stays byte-identical and cached.
        let prompt = ts
            .messages
            .iter()
            .map(|(_, t)| t.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        let mut stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let mut stderr = child.stderr.take().expect("piped stderr");
        let child = Arc::new(Mutex::new(child));
        {
            let mut st = self.lock();
            let slot = st.slots.entry(ep.clone()).or_default();
            slot.turn_key = Some(ts.turn_key.clone());
            slot.child = Some(child.clone());
            if slot.stop.is_some() {
                let _ = child.lock().unwrap_or_else(|p| p.into_inner()).kill();
            }
        }
        std::thread::spawn(move || {
            let _ = stdin.write_all(prompt.as_bytes());
        });
        let err_tail = std::thread::spawn(move || {
            let mut b = vec![];
            let _ = stderr.read_to_end(&mut b);
            let s = String::from_utf8_lossy(&b).to_string();
            let mut cut = s.len().saturating_sub(STDERR_TAIL);
            while !s.is_char_boundary(cut) {
                cut += 1;
            }
            s[cut..].to_string()
        });
        // Turn budget: kill the process when it runs too long.
        let watchdog = {
            let child = child.clone();
            let limit = self.cfg.turn_timeout;
            let done = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let d = done.clone();
            std::thread::spawn(move || {
                let t0 = std::time::Instant::now();
                while !d.load(std::sync::atomic::Ordering::Relaxed) {
                    if t0.elapsed() > limit {
                        let _ = child.lock().unwrap_or_else(|p| p.into_inner()).kill();
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }
            });
            done
        };
        let mut tr = Translator::new(turn.clone());
        let mut result = None;
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            let Ok(v) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            self.pin_reported_model(scope, ep, &v);
            let (mapped, r) = tr.line(&v);
            if !mapped.is_empty() {
                self.rec.commit(scope, ep, origin, mapped);
            }
            if r.is_some() {
                result = r;
            }
            if let Some(why) = tr.depth_violation() {
                // Depth 1 is enforced by the host too: stop the turn, say why.
                let _ = child.lock().unwrap_or_else(|p| p.into_inner()).kill();
                result = Some(
                    json!({"type": "result", "subtype": "error_delegation_depth",
                                     "is_error": true, "result": why}),
                );
                break;
            }
        }
        let exit = child
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .wait()
            .map(|s| s.to_string())
            .unwrap_or_else(|e| e.to_string());
        watchdog.store(true, std::sync::atomic::Ordering::Relaxed);
        let stderr = err_tail.join().unwrap_or_default();
        drop(lease);
        self.finish(
            scope,
            ep,
            origin,
            &ts.turn_key,
            &turn,
            result.as_ref(),
            &exit,
            &stderr,
        );
        Ok(true)
    }

    /// The model Claude Code reports in its `system`/`init` line completes the harness pin.
    fn pin_reported_model(&self, scope: &ExecutionScope, ep: &EpisodeId, v: &Value) {
        let Some(h) = &self.cfg.harness else { return };
        if v.get("type").and_then(Value::as_str) != Some("system")
            || v.get("subtype").and_then(Value::as_str) != Some("init")
        {
            return;
        }
        let Some(model) = v.get("model").and_then(Value::as_str) else {
            return;
        };
        let pinned = self
            .store
            .episode(scope, ep)
            .ok()
            .and_then(|e| e.pins.harness);
        if pinned.is_some_and(|p| p.digest == *h.digest()) {
            let build = &self.manifest.runtime_build_id;
            if let Err(e) =
                harness::pin_episode(&self.store, scope, ep, h, build, Some(model), None, false)
            {
                tracing::warn!(episode = %ep, error = %e, "harness pin");
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish(
        &self,
        scope: &ExecutionScope,
        ep: &EpisodeId,
        origin: Origin,
        turn_key: &str,
        turn: &str,
        result: Option<&Value>,
        exit: &str,
        stderr: &str,
    ) {
        let stop = {
            let mut st = self.lock();
            st.slots.get_mut(ep).and_then(|s| {
                s.child = None;
                s.turn_key = None;
                s.stop.take()
            })
        };
        let snapshot = match (result, stop) {
            (None, Some(_)) => json!({"id": turn, "status": "interrupted"}),
            _ => turn_snapshot(turn, result, exit, stderr),
        };
        let (_, outcome, _) = mapping::turn_outcome(&snapshot, stop);
        self.rec
            .commit(scope, ep, origin, map_turn_completed(&snapshot, stop));
        if let Err(e) = self.store.finish_turn(turn_key, &outcome) {
            tracing::warn!(episode = %ep, error = %e, "finish claude turn");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::RuntimeChange;
    use ignis_core::contracts::EventBody;

    fn events(m: &[Mapped]) -> Vec<String> {
        m.iter()
            .map(|x| match x {
                Mapped::Record(r) => match &r.change {
                    RuntimeChange::Event(EventBody::AssistantMessage { text, .. }) => {
                        format!("message:{text}")
                    }
                    RuntimeChange::Event(EventBody::StepRationale { call_id, text, .. }) => {
                        format!("rationale:{}:{text}", call_id.as_ref().unwrap())
                    }
                    RuntimeChange::Status { to, .. } => format!("status:{to:?}"),
                    other => format!("{other:?}"),
                },
                Mapped::ToolStarted {
                    tool, arguments, ..
                } => match arguments {
                    Some(a) => format!("tool.started:{tool}:{a}"),
                    None => format!("tool.started:{tool}"),
                },
                Mapped::ToolFinished { tool, error, .. } => {
                    format!("tool.finished:{tool}:{}", error.is_some())
                }
                Mapped::Opaque { method, .. } => format!("opaque:{method}"),
            })
            .collect()
    }

    #[test]
    fn a_stream_maps_to_the_same_events_as_codex_items() {
        let mut t = Translator::new("t1");
        let lines = [
            json!({"type":"system","subtype":"init","session_id":"s","mcp_servers":[{"name":"ignis","status":"connected"}]}),
            json!({"type":"assistant","message":{"id":"m1","content":[
                {"type":"text","text":"Searching."},
                {"type":"tool_use","id":"tu1","name":"mcp__ignis__sources_search","input":{"query":"x"}}]}}),
            json!({"type":"user","message":{"content":[
                {"type":"tool_result","tool_use_id":"tu1","content":[{"type":"text","text":"{\"hits\":[]}"}]}]}}),
            json!({"type":"assistant","message":{"id":"m2","content":[{"type":"text","text":"Done."}]}}),
        ];
        let mut all = vec![];
        for l in &lines {
            let (m, r) = t.line(l);
            assert!(r.is_none());
            all.extend(m);
        }
        assert_eq!(
            events(&all),
            [
                "message:Searching.",
                "rationale:tu1:Searching.",
                r#"tool.started:sources_search:{"query":"x"}"#,
                // Seen again with the result; the recorder keeps the first (its mark).
                "tool.started:sources_search",
                "tool.finished:sources_search:false",
                "message:Done."
            ]
        );
        let (_, r) = t.line(&json!({"type":"result","subtype":"success","is_error":false}));
        let snap = turn_snapshot("t1", r.as_ref(), "exit 0", "");
        assert_eq!(snap["status"], "completed");
    }

    #[test]
    fn hidden_thinking_is_never_a_rationale() {
        let mut t = Translator::new("t4");
        let (m, _) = t.line(&json!({"type":"assistant","message":{"id":"m","content":[
            {"type":"thinking","thinking":"private reasoning about the query"},
            {"type":"tool_use","id":"tu","name":"mcp__ignis__sources_search","input":{"query":"q"}}]}}));
        assert_eq!(events(&m), [r#"tool.started:sources_search:{"query":"q"}"#]);
        // A message linked once is not reused for the next call.
        let (m, _) = t.line(&json!({"type":"assistant","message":{"id":"m2","content":[
            {"type":"text","text":"Reading the top hit."},
            {"type":"tool_use","id":"a","name":"mcp__ignis__sources_read","input":{}},
            {"type":"tool_use","id":"b","name":"mcp__ignis__sources_read","input":{}}]}}));
        assert_eq!(
            events(&m),
            [
                "message:Reading the top hit.",
                "rationale:a:Reading the top hit.",
                "tool.started:sources_read:{}",
                "tool.started:sources_read:{}"
            ]
        );
    }

    #[test]
    fn failures_keep_their_reason_and_never_read_as_success() {
        let mut t = Translator::new("t2");
        t.line(&json!({"type":"assistant","message":{"id":"m","content":[
            {"type":"tool_use","id":"tu","name":"mcp__ignis__datasets_inspect","input":{}}]}}));
        let (m, _) = t.line(&json!({"type":"user","message":{"content":[
            {"type":"tool_result","tool_use_id":"tu","is_error":true,"content":"denied"}]}}));
        assert_eq!(
            events(&m),
            [
                "tool.started:datasets_inspect",
                "tool.finished:datasets_inspect:true"
            ]
        );
        let limit = json!({"type":"result","subtype":"success","is_error":true,
                           "result":"Claude usage limit reached"});
        let snap = turn_snapshot("t2", Some(&limit), "exit 1", "");
        assert_eq!(snap["status"], "failed");
        assert_eq!(snap["error"]["message"], "Claude usage limit reached");
        let crashed = turn_snapshot("t2", None, "exit status: 1", "Invalid API key\n");
        assert_eq!(crashed["status"], "failed");
        assert!(crashed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Invalid API key"));
    }

    #[test]
    fn an_unconnected_ignis_server_is_recorded() {
        let mut t = Translator::new("t3");
        let (m, _) = t.line(&json!({"type":"system","subtype":"init",
            "mcp_servers":[{"name":"ignis","status":"failed"}]}));
        assert_eq!(events(&m), ["opaque:claude/system"]);
        assert_eq!(
            split_tool("mcp__ignis__jobs_get"),
            ("ignis".into(), "jobs_get".into())
        );
        assert_eq!(split_tool("Bash"), ("claude".into(), "Bash".into()));
    }
}

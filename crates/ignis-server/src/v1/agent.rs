//! Composition of the agent runtime behind `/api/v1`: one preflight per process, then one runtime
//! per `/api/v1` instance (registry and fixture mode have separate state stores, so separate
//! runtimes and state directories). Two runtimes exist (ADR 17), chosen with `--agent-runtime`:
//!
//! - `codex`: the pinned Codex App Server. Credentials: `CODEX_API_KEY` / `OPENAI_API_KEY` in the
//!   environment, or a ChatGPT plan through a signed-in login home (`--codex-login-home`, made with
//!   `codex login --device-auth` on a headless host).
//! - `claude`: Claude Code in headless mode. Credentials: `CLAUDE_CODE_OAUTH_TOKEN` (a Claude plan,
//!   from `claude setup-token`) or `ANTHROPIC_API_KEY`, or a login home (`--claude-login-home`).
//!
//! A missing binary or credential is reported in `GET /capabilities`; the server still starts and
//! records messages.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ignis_core::services::ArtifactStore;
use ignis_research::runtime::{
    claude_preflight, preflight, AgentRuntime, ClaudeBridge, ClaudeConfig, CodexBridge,
    CodexConfig, HarnessBundle, HarnessShelf, HarnessSource, McpAttachment, RuntimeBuildManifest,
    ToolDescriptions, PINNED_CODEX_VERSION,
};
use ignis_research::ResearchStore;

use super::Hub;

/// Which agent runtime hosts the episodes.
#[derive(clap::ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentRuntimeKind {
    #[default]
    Codex,
    Claude,
}

/// `ignis serve` flags of the agent runtime. Without the selected runtime's binary it is not
/// configured.
#[derive(clap::Args, Debug, Clone, Default)]
pub struct CodexArgs {
    /// Agent runtime: `codex` (Codex App Server) or `claude` (Claude Code, headless).
    #[arg(
        long = "agent-runtime",
        env = "IGNIS_AGENT_RUNTIME",
        value_enum,
        default_value_t
    )]
    pub agent_runtime: AgentRuntimeKind,
    /// Native `codex` executable of the pinned release (the file that is hashed; not an npm shim).
    #[arg(long = "codex-bin", env = "IGNIS_CODEX_BIN")]
    pub bin: Option<PathBuf>,
    /// Expected sha256 (hex) of that executable. Unset: the hash is recorded but not enforced.
    #[arg(long = "codex-sha256", env = "IGNIS_CODEX_SHA256")]
    pub sha256: Option<String>,
    /// Model the Codex runtime uses (recorded in the build manifest). Unset: Codex's default.
    #[arg(long = "codex-model", env = "IGNIS_CODEX_MODEL")]
    pub model: Option<String>,
    /// Reasoning effort of every Codex turn (`none`, `minimal`, `low`, `medium`, `high`, `xhigh`;
    /// recorded in the build manifest and pinned per episode). Unset: the runtime's default for
    /// the model (`medium` for GPT-6 with Codex 0.169).
    #[arg(long = "codex-reasoning-effort", env = "IGNIS_CODEX_REASONING_EFFORT")]
    pub reasoning_effort: Option<String>,
    /// A CODEX_HOME signed in with a ChatGPT account (`codex login`, or `codex login --device-auth`
    /// on a headless host), used instead of an API key. Every workspace shares it, and its plan
    /// limits. Must be writable: Codex refreshes the token in place.
    #[arg(long = "codex-login-home", env = "IGNIS_CODEX_LOGIN_HOME")]
    pub login_home: Option<PathBuf>,
    /// Native `claude` executable (Claude Code).
    #[arg(long = "claude-bin", env = "IGNIS_CLAUDE_BIN")]
    pub claude_bin: Option<PathBuf>,
    /// Model the Claude runtime uses. Unset: Claude Code's default for the account.
    #[arg(long = "claude-model", env = "IGNIS_CLAUDE_MODEL")]
    pub claude_model: Option<String>,
    /// A Claude config directory signed in with `claude /login`, shared by every workspace, used
    /// when no token or API key is set. Must be writable.
    #[arg(long = "claude-login-home", env = "IGNIS_CLAUDE_LOGIN_HOME")]
    pub claude_login_home: Option<PathBuf>,
    /// Let the Claude investigator hand one bounded subtask to Claude Code's native subagent
    /// (depth 1, same IGNIS tools and scope, no `answer_submit`). Off by default; the runtime
    /// build id records it.
    #[arg(long = "claude-delegation", env = "IGNIS_CLAUDE_DELEGATION")]
    pub claude_delegation: bool,
    /// A harness directory (`investigator.md`, `skills/<name>/SKILL.md`) to start episodes with
    /// instead of the bundle compiled from `harness/`; its directory name is the label. A malformed
    /// bundle leaves the runtime unavailable.
    #[arg(long = "harness-dir", env = "IGNIS_HARNESS_DIR")]
    pub harness_dir: Option<PathBuf>,
}

#[derive(Clone)]
enum Prepared {
    Codex(CodexConfig),
    Claude(ClaudeConfig),
}

/// Result of the one preflight of this process.
#[derive(Clone)]
pub struct Preflighted {
    cfg: Option<Prepared>,
    outcome: Result<RuntimeBuildManifest, String>,
    harness_dir: Option<PathBuf>,
}

impl Preflighted {
    /// The runtime passed preflight (so a tool endpoint has a consumer).
    pub fn ready(&self) -> bool {
        self.cfg.is_some() && self.outcome.is_ok()
    }

    /// Run the preflight (blocking; startup only). `home` holds the runtime's preflight state.
    pub fn run(args: &CodexArgs, home: &Path, _auth: super::io::AuthMode) -> Self {
        let (cfg, outcome) = match args.agent_runtime {
            AgentRuntimeKind::Codex => match &args.bin {
                None => (
                    None,
                    Err(format!(
                        "not configured: set --codex-bin / IGNIS_CODEX_BIN to the codex {PINNED_CODEX_VERSION} executable; messages are recorded"
                    )),
                ),
                Some(bin) => {
                    let mut cfg = CodexConfig::new(bin, home);
                    cfg.expected_sha256 = args.sha256.clone();
                    cfg.model = args.model.clone();
                    cfg.reasoning_effort = args
                        .reasoning_effort
                        .as_deref()
                        .map(str::trim)
                        .filter(|e| !e.is_empty())
                        .map(str::to_ascii_lowercase);
                    cfg.login_home = args.login_home.clone();
                    let outcome = preflight(&cfg);
                    (Some(Prepared::Codex(cfg)), outcome)
                }
            },
            AgentRuntimeKind::Claude => match &args.claude_bin {
                None => (
                    None,
                    Err("not configured: set --claude-bin / IGNIS_CLAUDE_BIN to the claude executable; messages are recorded".into()),
                ),
                Some(bin) => {
                    let mut cfg = ClaudeConfig::new(bin, home.with_file_name("claude"));
                    cfg.model = args.claude_model.clone();
                    cfg.login_home = args.claude_login_home.clone();
                    cfg.delegation = args.claude_delegation;
                    let outcome = claude_preflight(&cfg);
                    (Some(Prepared::Claude(cfg)), outcome)
                }
            },
        };
        match &outcome {
            Ok(m) => {
                tracing::info!(build = %m.runtime_build_id, credentials = %m.credentials, "agent runtime preflight passed")
            }
            Err(e) => {
                tracing::error!(reason = %e, "agent runtime preflight failed; the agent runtime is unavailable")
            }
        }
        Preflighted {
            cfg,
            outcome,
            harness_dir: args.harness_dir.clone(),
        }
    }
}

/// What `/capabilities` says about the agent runtime.
#[derive(Clone)]
pub struct RuntimeReport {
    pub available: bool,
    pub detail: String,
    pub manifest: Option<RuntimeBuildManifest>,
}

/// A runtime over `store`, its state under `home`, waking `hub` after each commit, with the IGNIS
/// tools of `mcp` attached to every runtime process. The harness bundle it renders goes on `shelf`
/// (the tool host's), where `skills_read` serves its skills.
#[allow(clippy::too_many_arguments)]
pub fn runtime(
    pre: &Preflighted,
    home: PathBuf,
    store: Arc<ResearchStore>,
    artifacts: Arc<dyn ArtifactStore>,
    hub: Arc<Hub>,
    mcp: Option<McpAttachment>,
    shelf: &HarnessShelf,
) -> (Option<Arc<dyn AgentRuntime>>, RuntimeReport) {
    let manifest = match (&pre.cfg, &pre.outcome) {
        (Some(_), Ok(m)) => m.clone(),
        (_, Err(reason)) => {
            return (
                None,
                RuntimeReport {
                    available: false,
                    detail: format!("agent runtime unavailable: {reason}"),
                    manifest: None,
                },
            )
        }
        (None, Ok(_)) => unreachable!("a manifest needs a configuration"),
    };
    // The bundle names the tools this runtime is attached to, so it is rendered per attachment.
    let attached = mcp
        .as_ref()
        .map(|m| m.enabled_tools.clone())
        .unwrap_or_default();
    // Codex defers MCP tool definitions, so its prefix carries each tool's description; Claude Code
    // sends every definition with each request, so its prefix only lists the tools.
    let descriptions = match pre.cfg {
        Some(Prepared::Claude(_)) => ToolDescriptions::Index,
        _ => ToolDescriptions::Full,
    };
    let bundle = match &pre.harness_dir {
        None => Ok(HarnessBundle::embedded_with(&attached, descriptions)),
        Some(dir) => HarnessSource::load(dir).and_then(|src| {
            let label = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "custom".into());
            HarnessBundle::render_with(&label, &src, &attached, descriptions)
        }),
    };
    let bundle = match bundle {
        Ok(b) => b,
        Err(reason) => {
            tracing::error!(%reason, "harness bundle refused; the agent runtime is unavailable");
            return (
                None,
                RuntimeReport {
                    available: false,
                    detail: format!("agent runtime unavailable: {reason}"),
                    manifest: None,
                },
            );
        }
    };
    shelf.add(bundle.clone());
    let harness = format!("harness {}", bundle.short());
    let tools = if mcp.is_some() {
        "IGNIS tools over loopback MCP"
    } else {
        "no IGNIS tools"
    };
    let hook: ignis_research::runtime::CommitHook = Arc::new(move |ep, seq| hub.notify(ep, seq));
    let (rt, what): (Arc<dyn AgentRuntime>, String) = match pre.cfg.clone().expect("checked") {
        Prepared::Codex(mut cfg) => {
            cfg.home = home;
            cfg.mcp = mcp;
            cfg.harness = Some(bundle);
            (
                Arc::new(CodexBridge::new(
                    cfg,
                    manifest.clone(),
                    store,
                    artifacts,
                    hook,
                )),
                format!("Codex App Server {} over private stdio", manifest.version),
            )
        }
        Prepared::Claude(mut cfg) => {
            cfg.home = home.with_file_name("claude");
            cfg.mcp = mcp;
            cfg.harness = Some(bundle);
            (
                Arc::new(ClaudeBridge::new(
                    cfg,
                    manifest.clone(),
                    store,
                    artifacts,
                    hook,
                )),
                format!("Claude Code {} headless (stream-json)", manifest.version),
            )
        }
    };
    (
        Some(rt),
        RuntimeReport {
            available: true,
            detail: format!(
                "{what} ({:?} profile: no shell; {tools}; {harness}; credentials: {})",
                manifest.profile, manifest.credentials
            ),
            manifest: Some(manifest),
        },
    )
}

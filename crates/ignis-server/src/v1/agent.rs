//! Composition of the agent runtime behind `/api/v1` (task M1): preflight of the pinned Codex App
//! Server once per process, then one [`CodexBridge`] per `/api/v1` instance (registry and fixture
//! mode have separate state stores, so separate bridges and `CODEX_HOME`s). A missing or wrong
//! binary is reported in `GET /capabilities`; the server still starts and records messages.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ignis_core::services::ArtifactStore;
use ignis_research::runtime::{
    preflight, AgentRuntime, CodexBridge, CodexConfig, McpAttachment, RuntimeBuildManifest,
    PINNED_CODEX_VERSION,
};
use ignis_research::ResearchStore;

use super::Hub;

/// `ignis serve` flags of the agent runtime. Without `--codex-bin` the runtime is not configured.
#[derive(clap::Args, Debug, Clone, Default)]
pub struct CodexArgs {
    /// Native `codex` executable of the pinned release (the file that is hashed; not an npm shim).
    #[arg(long = "codex-bin", env = "IGNIS_CODEX_BIN")]
    pub bin: Option<PathBuf>,
    /// Expected sha256 (hex) of that executable. Unset: the hash is recorded but not enforced.
    #[arg(long = "codex-sha256", env = "IGNIS_CODEX_SHA256")]
    pub sha256: Option<String>,
    /// Model the runtime uses (recorded in the build manifest). Unset: Codex's default.
    #[arg(long = "codex-model", env = "IGNIS_CODEX_MODEL")]
    pub model: Option<String>,
    /// Local development only (`--auth local`): a CODEX_HOME signed in with a Codex account
    /// (`tools/codex/login.*`), used instead of an API key. Every workspace shares it.
    #[arg(long = "codex-login-home", env = "IGNIS_CODEX_LOGIN_HOME")]
    pub login_home: Option<PathBuf>,
}

/// Result of the one preflight of this process.
#[derive(Clone)]
pub struct Preflighted {
    cfg: Option<CodexConfig>,
    outcome: Result<RuntimeBuildManifest, String>,
}

impl Preflighted {
    /// The runtime passed preflight (so a tool endpoint has a consumer).
    pub fn ready(&self) -> bool {
        self.cfg.is_some() && self.outcome.is_ok()
    }

    /// Run the preflight (blocking; startup only). `home` holds the generated protocol schema and
    /// the probe's throwaway `CODEX_HOME`.
    pub fn run(args: &CodexArgs, home: &Path, auth: super::io::AuthMode) -> Self {
        let Some(bin) = &args.bin else {
            return Preflighted {
                cfg: None,
                outcome: Err(format!(
                    "not configured: set --codex-bin / IGNIS_CODEX_BIN to the codex {PINNED_CODEX_VERSION} executable; messages are recorded"
                )),
            };
        };
        let mut cfg = CodexConfig::new(bin, home);
        cfg.expected_sha256 = args.sha256.clone();
        cfg.model = args.model.clone();
        cfg.login_home = args.login_home.clone();
        let outcome = if cfg.login_home.is_some() && auth != super::io::AuthMode::Local {
            // A signed-in personal account shared by every workspace is not multi-user isolation.
            Err("--codex-login-home is for local development only (--auth local); deployments use an API key".into())
        } else {
            preflight(&cfg)
        };
        match &outcome {
            Ok(m) => {
                tracing::info!(build = %m.runtime_build_id, "codex app-server preflight passed")
            }
            Err(e) => {
                tracing::error!(reason = %e, "codex app-server preflight failed; the agent runtime is unavailable")
            }
        }
        Preflighted {
            cfg: Some(cfg),
            outcome,
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

/// A bridge over `store`, its `CODEX_HOME`s under `home`, waking `hub` after each commit, with the
/// IGNIS tools of `mcp` registered in every runtime process.
pub fn runtime(
    pre: &Preflighted,
    home: PathBuf,
    store: Arc<ResearchStore>,
    artifacts: Arc<dyn ArtifactStore>,
    hub: Arc<Hub>,
    mcp: Option<McpAttachment>,
) -> (Option<Arc<dyn AgentRuntime>>, RuntimeReport) {
    match (&pre.cfg, &pre.outcome) {
        (Some(cfg), Ok(manifest)) => {
            let mut cfg = cfg.clone();
            cfg.home = home;
            let tools = if mcp.is_some() {
                "IGNIS tools over loopback MCP"
            } else {
                "no IGNIS tools"
            };
            cfg.mcp = mcp;
            let bridge = CodexBridge::new(
                cfg,
                manifest.clone(),
                store,
                artifacts,
                Arc::new(move |ep, seq| hub.notify(ep, seq)),
            );
            (
                Some(Arc::new(bridge)),
                RuntimeReport {
                    available: true,
                    detail: format!(
                        "Codex App Server {} over private stdio ({:?} profile: no shell; {tools})",
                        manifest.version, manifest.profile
                    ),
                    manifest: Some(manifest.clone()),
                },
            )
        }
        (_, Err(reason)) => (
            None,
            RuntimeReport {
                available: false,
                detail: format!("agent runtime unavailable: {reason}"),
                manifest: None,
            },
        ),
        (None, Ok(_)) => unreachable!("a manifest needs a configuration"),
    }
}

//! `ignis serve` flags and startup preflight of the isolated analysis runner (`analysis_execute`).
//! Without `--runner-image` the runner is not configured; a runner that fails preflight is absent.
//! Either way the tool is not registered and `/capabilities` says why.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ignis_core::contracts::{Digest, RunnerEnvironment, RunnerLimits};
use ignis_research::sandbox::{Sandbox, SandboxConfig};

use super::io::AuthMode;

const MIB: u64 = 1 << 20;

#[derive(clap::Args, Debug, Clone)]
pub struct RunnerArgs {
    /// Image id of the runner (`sha256:…`, printed by `tools/runner/build.sh`).
    #[arg(long = "runner-image", env = "IGNIS_RUNNER_IMAGE")]
    pub image: Option<String>,
    /// OCI runtime the engine starts the container with (`runsc` = gVisor).
    #[arg(
        long = "runner-runtime",
        env = "IGNIS_RUNNER_RUNTIME",
        default_value = "runsc"
    )]
    pub runtime: String,
    /// Allow a runtime other than gVisor (development machines without `runsc`; `--auth local`
    /// only).
    #[arg(long = "runner-allow-container", env = "IGNIS_RUNNER_ALLOW_CONTAINER")]
    pub allow_container: bool,
    /// Container engine CLI.
    #[arg(
        long = "runner-engine",
        env = "IGNIS_RUNNER_ENGINE",
        default_value = "docker"
    )]
    pub engine: PathBuf,
    /// Directory of the per-run mounts (absolute, shareable with the engine). Default:
    /// `<state-dir>/runner`.
    #[arg(long = "runner-dir", env = "IGNIS_RUNNER_DIR")]
    pub dir: Option<PathBuf>,
    #[arg(long = "runner-wall-seconds", default_value_t = 600)]
    pub wall_seconds: u64,
    #[arg(long = "runner-memory-mib", default_value_t = 2048)]
    pub memory_mib: u64,
    /// CPU quota in thousandths of a core.
    #[arg(long = "runner-cpu-millis", default_value_t = 2000)]
    pub cpu_millis: u64,
    #[arg(long = "runner-pids", default_value_t = 256)]
    pub pids: u64,
    #[arg(long = "runner-tmp-mib", default_value_t = 256)]
    pub tmp_mib: u64,
    #[arg(long = "runner-output-mib", default_value_t = 1024)]
    pub output_mib: u64,
    #[arg(long = "runner-stream-mib", default_value_t = 16)]
    pub stream_mib: u64,
}

impl Default for RunnerArgs {
    fn default() -> Self {
        RunnerArgs {
            image: None,
            runtime: "runsc".into(),
            allow_container: false,
            engine: "docker".into(),
            dir: None,
            wall_seconds: 600,
            memory_mib: 2048,
            cpu_millis: 2000,
            pids: 256,
            tmp_mib: 256,
            output_mib: 1024,
            stream_mib: 16,
        }
    }
}

/// What `/capabilities` reports about the runner.
#[derive(Debug, Clone)]
pub struct RunnerReport {
    pub available: bool,
    pub detail: String,
    pub environment: Option<RunnerEnvironment>,
}

/// Preflight the runner once per process (blocking; startup only).
pub fn preflight(
    args: &RunnerArgs,
    state_dir: &Path,
    auth: AuthMode,
) -> (Option<Arc<Sandbox>>, RunnerReport) {
    let unavailable = |detail: String| {
        (
            None,
            RunnerReport {
                available: false,
                detail,
                environment: None,
            },
        )
    };
    let Some(image) = &args.image else {
        return unavailable("not configured (set --runner-image)".into());
    };
    let image = match Digest::parse(image) {
        Ok(d) => d,
        Err(e) => return unavailable(format!("--runner-image must be an image id: {e}")),
    };
    if args.allow_container && auth != AuthMode::Local {
        return unavailable(
            "--runner-allow-container is for --auth local only; a shared deployment runs gVisor"
                .into(),
        );
    }
    let cfg = SandboxConfig {
        engine: args.engine.clone(),
        image,
        oci_runtime: args.runtime.clone(),
        require_gvisor: !args.allow_container,
        work_dir: args.dir.clone().unwrap_or_else(|| state_dir.join("runner")),
        limits: RunnerLimits {
            wall_seconds: args.wall_seconds,
            memory_bytes: args.memory_mib * MIB,
            cpu_millis: args.cpu_millis,
            pids: args.pids,
            tmp_bytes: args.tmp_mib * MIB,
            output_bytes: args.output_mib * MIB,
            stream_bytes: args.stream_mib * MIB,
        },
    };
    match Sandbox::preflight(cfg) {
        Ok(s) => {
            let env = s.environment().clone();
            tracing::info!(image = %env.image_id, runtime = %env.oci_runtime, isolation = ?env.isolation, "analysis runner passed preflight");
            let detail = format!(
                "isolated runner: {} under {} ({:?}), {}; probe passed",
                env.image_id, env.oci_runtime, env.isolation, env.interpreter
            );
            (
                Some(Arc::new(s)),
                RunnerReport {
                    available: true,
                    detail,
                    environment: Some(env),
                },
            )
        }
        Err(e) => {
            tracing::error!(error = %e, "analysis runner failed preflight; analysis_execute is not registered");
            unavailable(format!("failed preflight: {e}"))
        }
    }
}

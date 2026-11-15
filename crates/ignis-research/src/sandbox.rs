//! The isolated analysis runner behind `analysis_execute` (M3).
//!
//! A program runs once in a fresh OCI container started by the host's container engine CLI, never
//! as a host process. Isolation comes from how the container is started, not from the image:
//!
//! - no network (`--network none`), read-only root, `/tmp` in memory, uid/gid 65534, every
//!   capability dropped, `no-new-privileges`, budgets on memory (no swap), CPU, pids, wall time and
//!   file size;
//! - exactly three bind mounts, all under a per-run directory: the program (`/job`, read-only, with
//!   `/job/inputs.json` listing the mounted inputs), the inputs (`/in`, read-only: copies of the data
//!   the request names by reference — workspace artifacts, dataset views, complete conditions
//!   tables — resolved under the caller's scope, see [`check_inputs`]) and the output directory
//!   (`/out`). No host path, socket, home, key or other workspace is mounted, and no
//!   host environment variable is passed (only the fixed ones in [`FIXED_ENV`]);
//! - the image is pinned by content address (image id) and the OCI runtime is named explicitly; with
//!   `runsc` the program runs under gVisor, which is what the public deployment requires.
//!
//! [`Sandbox::preflight`] proves this on the configured engine before the tool is registered: it
//! checks the image id and the runtime, then runs a probe program through the same code path as a
//! real execution and refuses the sandbox unless the probe observes the isolation from inside. A
//! sandbox that fails preflight is absent: the tool is not registered and capability status says
//! why. There is no fallback to a host process.
//!
//! After the container exits, stdout/stderr (up to their budget, truncation recorded) and the
//! regular files under `/out` become artifacts; symbolic links and special files are never followed
//! and are reported as rejected. The [`ExecutionRecord`] ties program, environment, inputs, budget
//! and outcome together.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use ignis_core::contracts::{
    AnalysisExecuteRequest, AnalysisInput, AnalysisLanguage, AnalysisOutput, ArtifactRef,
    CapturedStream, ConditionsQuery, ConditionsTable, DatasetBundle, DatasetId, DatasetView,
    Digest, EntityRef, ErrorCode, ErrorEnvelope, ExecutionOutcome, ExecutionRecord, ExecutionScope,
    MountedInput, Origin, Producer, RejectedOutput, ResolvedInput, RetryPolicy, RunnerEnvironment,
    RunnerIsolation, RunnerLimits, Timestamp, ToolsSchema,
};
use ignis_core::services::{ArtifactDraft, ArtifactStore, DatasetService, ServiceResult};
use serde::Deserialize;
use sha2::{Digest as _, Sha256};

/// Container label of every run; preflight removes leftovers of a crashed process carrying the label
/// of the same run directory.
const LABEL: &str = "org.ignis.runner";
/// The uid/gid the program runs as (`nobody`): never the host service's uid.
pub const RUN_AS: &str = "65534:65534";
/// The only environment the program receives besides what the pinned image declares.
pub const FIXED_ENV: [(&str, &str); 4] = [
    ("HOME", "/tmp"),
    ("PYTHONDONTWRITEBYTECODE", "1"),
    ("PYTHONUNBUFFERED", "1"),
    ("MPLCONFIGDIR", "/tmp"),
];

/// Host configuration of the runner. Budgets are the operator's, never the request's.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Container engine CLI (`docker`, or a compatible one).
    pub engine: PathBuf,
    /// Image id to run (`sha256:…`); any other image is refused.
    pub image: Digest,
    /// OCI runtime (`runsc` for gVisor, `runc` for plain containers).
    pub oci_runtime: String,
    /// Refuse a runtime that is not gVisor (public deployment).
    pub require_gvisor: bool,
    /// Per-run directories are created here (absolute; must be shareable with the engine).
    pub work_dir: PathBuf,
    pub limits: RunnerLimits,
}

/// A runner that passed preflight.
#[derive(Debug)]
pub struct Sandbox {
    cfg: SandboxConfig,
    env: RunnerEnvironment,
    /// Label value scoping leftover cleanup to this run directory.
    owner: String,
}

fn err(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

fn internal(context: &str, e: impl std::fmt::Display) -> ErrorEnvelope {
    err(ErrorCode::Internal, format!("{context}: {e}"))
}

fn sha256_hex(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}

/// Whether the OCI runtime name means gVisor.
pub fn isolation_of(oci_runtime: &str) -> RunnerIsolation {
    if oci_runtime == "runsc" || oci_runtime.starts_with("runsc-") {
        RunnerIsolation::Gvisor
    } else {
        RunnerIsolation::Container
    }
}

/// The paths of one run.
struct RunDir {
    root: PathBuf,
}

impl RunDir {
    fn create(work_dir: &Path) -> ServiceResult<Self> {
        let root = work_dir.join(format!("run_{}", uuid::Uuid::new_v4().simple()));
        for d in ["", "job", "in", "out"] {
            std::fs::create_dir_all(root.join(d)).map_err(|e| internal("run directory", e))?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let set = |p: PathBuf, m: u32| {
                std::fs::set_permissions(&p, std::fs::Permissions::from_mode(m))
                    .map_err(|e| internal("run directory permissions", e))
            };
            // Only the host service traverses the run directory; the container sees the mounts.
            set(root.clone(), 0o700)?;
            set(root.join("job"), 0o755)?;
            set(root.join("in"), 0o755)?;
            // The program runs as 65534 and must be able to write its outputs.
            set(root.join("out"), 0o777)?;
        }
        Ok(RunDir { root })
    }
    fn job(&self) -> PathBuf {
        self.root.join("job")
    }
    fn inputs(&self) -> PathBuf {
        self.root.join("in")
    }
    fn out(&self) -> PathBuf {
        self.root.join("out")
    }
}

impl Drop for RunDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// A mount source as the engine's `--mount` syntax accepts it (comma-separated key=value pairs).
/// Windows verbatim prefixes (`\\?\D:\…`, from `canonicalize`) are dropped: the engine refuses them.
fn mount_source(p: &Path) -> ServiceResult<String> {
    let s = p
        .to_str()
        .ok_or_else(|| internal("run directory", "path is not UTF-8"))?;
    let s = match s.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC") => rest,
        _ => s,
    };
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        return Err(internal(
            "run directory",
            format!("{s:?} cannot be passed to --mount"),
        ));
    }
    Ok(s.to_string())
}

/// The full `run` command line of one execution. Pure, so the isolation flags are unit-tested.
pub fn run_args(
    cfg: &SandboxConfig,
    owner: &str,
    name: &str,
    job: &str,
    inputs: &str,
    out: &str,
) -> Vec<String> {
    let l = &cfg.limits;
    let mut a: Vec<String> = vec![
        "run".into(),
        "--name".into(),
        name.into(),
        "--label".into(),
        format!("{LABEL}={owner}"),
        "--runtime".into(),
        cfg.oci_runtime.clone(),
        "--network".into(),
        "none".into(),
        "--read-only".into(),
        "--cap-drop".into(),
        "ALL".into(),
        "--security-opt".into(),
        "no-new-privileges".into(),
        "--user".into(),
        RUN_AS.into(),
        "--pids-limit".into(),
        l.pids.to_string(),
        "--memory".into(),
        l.memory_bytes.to_string(),
        "--memory-swap".into(),
        l.memory_bytes.to_string(),
        "--cpus".into(),
        format!("{}.{:03}", l.cpu_millis / 1000, l.cpu_millis % 1000),
        "--ulimit".into(),
        format!("fsize={0}:{0}", l.output_bytes.max(l.stream_bytes)),
        "--ulimit".into(),
        "core=0:0".into(),
        "--tmpfs".into(),
        format!("/tmp:rw,nosuid,nodev,noexec,size={}", l.tmp_bytes),
        "--mount".into(),
        format!("type=bind,source={job},target=/job,readonly"),
        "--mount".into(),
        format!("type=bind,source={inputs},target=/in,readonly"),
        "--mount".into(),
        format!("type=bind,source={out},target=/out"),
        "--workdir".into(),
        "/out".into(),
        "--log-driver".into(),
        "none".into(),
    ];
    for (k, v) in FIXED_ENV {
        a.push("--env".into());
        a.push(format!("{k}={v}"));
    }
    a.extend([
        "--entrypoint".into(),
        "python".into(),
        cfg.image.to_string(),
        "-I".into(),
        "/job/main.py".into(),
    ]);
    a
}

/// What the container engine reports about an exited container.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContainerState {
    exit_code: i64,
    #[serde(rename = "OOMKilled")]
    oom_killed: bool,
    #[serde(default)]
    error: String,
}

/// A stream captured to a file, with the kept prefix bounded by the budget.
struct Capture {
    path: PathBuf,
    total: u64,
}

fn capture(mut from: impl Read, to: PathBuf, budget: u64) -> std::io::Result<Capture> {
    let mut f = File::create(&to)?;
    let mut buf = vec![0u8; 64 << 10];
    let mut total = 0u64;
    loop {
        let n = match from.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        let keep = budget.saturating_sub(total).min(n as u64) as usize;
        f.write_all(&buf[..keep])?;
        total += n as u64;
    }
    f.sync_all()?;
    Ok(Capture { path: to, total })
}

struct RawRun {
    outcome: ExecutionOutcome,
    stdout: Capture,
    stderr: Capture,
    started_at: Timestamp,
    finished_at: Timestamp,
}

impl Sandbox {
    /// Check the engine, the runtime and the pinned image, remove leftovers of a previous process,
    /// then run the isolation probe. `Err` explains why the runner is unavailable.
    pub fn preflight(cfg: SandboxConfig) -> Result<Sandbox, String> {
        let isolation = isolation_of(&cfg.oci_runtime);
        if cfg.require_gvisor && isolation != RunnerIsolation::Gvisor {
            return Err(format!(
                "OCI runtime {:?} is not gVisor and this deployment requires gVisor",
                cfg.oci_runtime
            ));
        }
        if !cfg.work_dir.is_absolute() {
            return Err(format!(
                "runner directory {} is not absolute",
                cfg.work_dir.display()
            ));
        }
        let l = &cfg.limits;
        if [
            l.wall_seconds,
            l.memory_bytes,
            l.cpu_millis,
            l.pids,
            l.tmp_bytes,
        ]
        .contains(&0)
        {
            return Err("a runner budget is zero".into());
        }
        std::fs::create_dir_all(&cfg.work_dir)
            .map_err(|e| format!("runner directory {}: {e}", cfg.work_dir.display()))?;
        let engine = |args: &[&str]| -> Result<String, String> {
            let o = Command::new(&cfg.engine)
                .args(args)
                .stdin(Stdio::null())
                .output()
                .map_err(|e| format!("{} {}: {e}", cfg.engine.display(), args[0]))?;
            if !o.status.success() {
                return Err(format!(
                    "{} {}: {}",
                    cfg.engine.display(),
                    args.join(" "),
                    String::from_utf8_lossy(&o.stderr).trim()
                ));
            }
            Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
        };
        let os = engine(&["version", "--format", "{{.Server.Os}}"])?;
        if os != "linux" {
            return Err(format!("container engine runs {os} containers, not linux"));
        }
        let runtimes: serde_json::Value =
            serde_json::from_str(&engine(&["info", "--format", "{{json .Runtimes}}"])?)
                .map_err(|e| format!("engine runtimes: {e}"))?;
        if runtimes.get(&cfg.oci_runtime).is_none() {
            let known: Vec<&String> = runtimes
                .as_object()
                .map(|o| o.keys().collect())
                .unwrap_or_default();
            return Err(format!(
                "OCI runtime {:?} is not installed in the engine (has {known:?})",
                cfg.oci_runtime
            ));
        }
        let id = engine(&[
            "image",
            "inspect",
            "--format",
            "{{.Id}}",
            cfg.image.as_str(),
        ])?;
        if id != cfg.image.as_str() {
            return Err(format!(
                "image {} resolves to {id}; the runner only runs the pinned id",
                cfg.image
            ));
        }
        let image_env: Vec<String> = serde_json::from_str(&engine(&[
            "image",
            "inspect",
            "--format",
            "{{json .Config.Env}}",
            cfg.image.as_str(),
        ])?)
        .map_err(|e| format!("image environment: {e}"))?;
        let owner = sha256_hex(cfg.work_dir.to_string_lossy().as_bytes())[..16].to_string();
        let leftovers = engine(&["ps", "-aq", "--filter", &format!("label={LABEL}={owner}")])?;
        for c in leftovers.split_whitespace() {
            let _ = engine(&["rm", "-f", c]);
        }
        let mut sandbox = Sandbox {
            env: RunnerEnvironment {
                image_id: cfg.image.clone(),
                oci_runtime: cfg.oci_runtime.clone(),
                isolation,
                interpreter: String::new(),
            },
            cfg,
            owner,
        };
        let allowed_env: BTreeSet<String> = image_env
            .iter()
            .filter_map(|kv| kv.split_once('=').map(|(k, _)| k.to_string()))
            .chain(FIXED_ENV.iter().map(|(k, _)| k.to_string()))
            .chain(["HOSTNAME".to_string()])
            .collect();
        sandbox.env.interpreter = sandbox.probe(&allowed_env)?;
        Ok(sandbox)
    }

    pub fn environment(&self) -> &RunnerEnvironment {
        &self.env
    }

    pub fn limits(&self) -> &RunnerLimits {
        &self.cfg.limits
    }

    /// Run the probe through the real execution path and check what it observes from inside.
    /// Returns the interpreter version.
    fn probe(&self, allowed_env: &BTreeSet<String>) -> Result<String, String> {
        let dir = RunDir::create(&self.cfg.work_dir).map_err(|e| e.message)?;
        std::fs::write(dir.job().join("main.py"), PROBE)
            .and_then(|()| std::fs::write(dir.inputs().join("probe.txt"), b"ignis-probe"))
            .map_err(|e| format!("probe files: {e}"))?;
        let never = AtomicBool::new(false);
        let run = self.run(&dir, &never).map_err(|e| e.message)?;
        let stdout = std::fs::read_to_string(&run.stdout.path).unwrap_or_default();
        let stderr = std::fs::read_to_string(&run.stderr.path).unwrap_or_default();
        if run.outcome != (ExecutionOutcome::Exited { exit_code: 0 }) {
            return Err(format!(
                "isolation probe did not complete ({:?}): {}",
                run.outcome,
                stderr.trim()
            ));
        }
        let r: ProbeReport = serde_json::from_str(stdout.trim())
            .map_err(|e| format!("isolation probe output: {e}: {stdout:?}"))?;
        r.check(allowed_env)?;
        Ok(format!("Python {}", r.python))
    }

    /// Start the container for a prepared run directory and wait for it, killing it at the wall
    /// budget or on `cancel`. The container is always removed.
    fn run(&self, dir: &RunDir, cancel: &AtomicBool) -> ServiceResult<RawRun> {
        let name = format!("ignis-run-{}", uuid::Uuid::new_v4().simple());
        let args = run_args(
            &self.cfg,
            &self.owner,
            &name,
            &mount_source(&dir.job())?,
            &mount_source(&dir.inputs())?,
            &mount_source(&dir.out())?,
        );
        let started_at = Timestamp::now();
        let mut child: Child = Command::new(&self.cfg.engine)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| internal("starting the container engine", e))?;
        let budget = self.cfg.limits.stream_bytes;
        let (so, se) = (
            child.stdout.take().expect("piped"),
            child.stderr.take().expect("piped"),
        );
        let (po, pe) = (dir.root.join("stdout"), dir.root.join("stderr"));
        let to = std::thread::spawn(move || capture(so, po, budget));
        let te = std::thread::spawn(move || capture(se, pe, budget));
        let deadline = Instant::now() + Duration::from_secs(self.cfg.limits.wall_seconds);
        let mut timed_out = false;
        let mut cancelled = false;
        let kill = || {
            let _ = Command::new(&self.cfg.engine)
                .args(["kill", &name])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        };
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {}
                Err(e) => {
                    kill();
                    let _ = child.wait();
                    self.remove(&name);
                    return Err(internal("waiting for the container", e));
                }
            }
            if !timed_out && !cancelled {
                if cancel.load(Ordering::SeqCst) {
                    cancelled = true;
                    kill();
                } else if Instant::now() >= deadline {
                    timed_out = true;
                    kill();
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let joined = |t: std::thread::JoinHandle<std::io::Result<Capture>>| {
            t.join()
                .map_err(|_| internal("capturing output", "capture thread panicked"))?
                .map_err(|e| internal("capturing output", e))
        };
        let (stdout, stderr) = (joined(to), joined(te));
        let state = self.inspect(&name);
        self.remove(&name);
        let finished_at = Timestamp::now();
        let (stdout, stderr) = (stdout?, stderr?);
        if cancelled {
            return Err(err(ErrorCode::Cancelled, "cancelled on request"));
        }
        let state = state.map_err(|e| {
            let tail = std::fs::read_to_string(&stderr.path).unwrap_or_default();
            internal(
                "the container did not run",
                format!("{e}; engine said: {}", tail.trim()),
            )
        })?;
        if !state.error.is_empty() {
            return Err(internal("the container did not run", state.error));
        }
        let outcome = if timed_out {
            ExecutionOutcome::TimedOut
        } else if state.oom_killed {
            ExecutionOutcome::OutOfMemory
        } else {
            ExecutionOutcome::Exited {
                exit_code: state.exit_code,
            }
        };
        Ok(RawRun {
            outcome,
            stdout,
            stderr,
            started_at,
            finished_at,
        })
    }

    fn inspect(&self, name: &str) -> Result<ContainerState, String> {
        let o = Command::new(&self.cfg.engine)
            .args(["inspect", "--format", "{{json .State}}", name])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        if !o.status.success() {
            return Err(String::from_utf8_lossy(&o.stderr).trim().to_string());
        }
        serde_json::from_slice(&o.stdout).map_err(|e| e.to_string())
    }

    fn remove(&self, name: &str) {
        let _ = Command::new(&self.cfg.engine)
            .args(["rm", "-f", name])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    /// Execute one request with the caller's `scope`. Returns the job outputs: the
    /// [`ExecutionRecord`] first, then the program, the captured streams and the kept outputs.
    /// A program that fails, times out or runs out of memory is a completed execution (the record
    /// says how it ended); only host problems are errors.
    pub fn execute(
        &self,
        scope: &ExecutionScope,
        req: &AnalysisExecuteRequest,
        artifacts: &dyn ArtifactStore,
        datasets: Option<&dyn DatasetService>,
        producer: Producer,
        cancel: &AtomicBool,
    ) -> ServiceResult<Vec<ArtifactRef>> {
        req.validate().map_err(ErrorEnvelope::from)?;
        let AnalysisLanguage::Python = req.language;
        let dir = RunDir::create(&self.cfg.work_dir)?;

        // Inputs: resolved by reference under the caller's scope, copied and verified against
        // their digest. The program finds the list in /job/inputs.json.
        let mounted = stage_inputs(scope, req, artifacts, datasets, &producer, &dir.inputs())?;
        let manifest = serde_json::to_vec_pretty(&mounted).map_err(|e| internal("inputs", e))?;
        std::fs::write(dir.job().join("inputs.json"), manifest)
            .map_err(|e| internal("staging the input list", e))?;
        let origin = Origin::combine(mounted.iter().map(|m| m.artifact.origin));
        // A dataset view is not a workspace artifact: its lineage is the dataset, whose partitions
        // a later reader of these outputs must hold (see `check_lineage`).
        let lineage: Vec<EntityRef> = mounted
            .iter()
            .map(|m| match &m.resolved {
                Some(ResolvedInput::DatasetView { dataset_id, .. }) => {
                    EntityRef::Dataset(dataset_id.clone())
                }
                _ => EntityRef::Artifact(m.artifact.artifact_id.clone()),
            })
            .collect();
        let put = |media_type: &str,
                   contract: Option<&str>,
                   inputs: Vec<EntityRef>,
                   bytes: &mut dyn Read|
         -> ServiceResult<ArtifactRef> {
            artifacts.put(
                scope,
                ArtifactDraft {
                    media_type: media_type.into(),
                    contract: contract.map(str::to_string),
                    produced_by: producer.clone(),
                    inputs,
                    origin,
                },
                bytes,
            )
        };

        std::fs::write(dir.job().join("main.py"), req.source.as_bytes())
            .map_err(|e| internal("staging the program", e))?;
        let program = put("text/x-python", None, vec![], &mut req.source.as_bytes())?;

        let run = self.run(&dir, cancel)?;
        let l = &self.cfg.limits;
        let stream = |c: &Capture| -> ServiceResult<CapturedStream> {
            let artifact = if c.total == 0 {
                None
            } else {
                let mut f = File::open(&c.path).map_err(|e| internal("captured output", e))?;
                let mut lineage = lineage.clone();
                lineage.push(EntityRef::Artifact(program.artifact_id.clone()));
                Some(put("text/plain", None, lineage, &mut f)?)
            };
            Ok(CapturedStream {
                total_bytes: c.total,
                truncated: c.total > l.stream_bytes,
                artifact,
            })
        };
        let stdout = stream(&run.stdout)?;
        let stderr = stream(&run.stderr)?;

        let mut kept = Vec::new();
        let mut rejected = Vec::new();
        let mut used = 0u64;
        for (rel, path, kind) in walk_outputs(&dir.out()) {
            match kind {
                OutKind::File(len) if used + len <= l.output_bytes => {
                    let mut f = match open_regular(&path) {
                        Ok(f) => f,
                        Err(reason) => {
                            rejected.push(RejectedOutput { path: rel, reason });
                            continue;
                        }
                    };
                    let mut lineage = lineage.clone();
                    lineage.push(EntityRef::Artifact(program.artifact_id.clone()));
                    let a = put(media_type_of(&rel), None, lineage, &mut f)?;
                    used += a.byte_length;
                    kept.push(AnalysisOutput {
                        path: rel,
                        artifact: a,
                    });
                }
                OutKind::File(len) => rejected.push(RejectedOutput {
                    path: rel,
                    reason: format!(
                        "output budget exhausted ({len} bytes; {} of {} already kept)",
                        used, l.output_bytes
                    ),
                }),
                OutKind::Rejected(reason) => rejected.push(RejectedOutput { path: rel, reason }),
            }
        }

        kept.sort_by(|a, b| a.path.cmp(&b.path));
        rejected.sort_by(|a, b| a.path.cmp(&b.path));
        let record = ExecutionRecord {
            schema_version: ToolsSchema::V1,
            language: req.language,
            program: program.clone(),
            environment: self.env.clone(),
            inputs: mounted,
            limits: l.clone(),
            outcome: run.outcome,
            stdout,
            stderr,
            outputs: kept,
            rejected_outputs: rejected,
            started_at: run.started_at,
            finished_at: run.finished_at,
            origin,
        };
        record.validate().map_err(ErrorEnvelope::from)?;
        let bytes = serde_json::to_vec_pretty(&record).map_err(|e| internal("record", e))?;
        let mut lineage = lineage;
        lineage.push(EntityRef::Artifact(program.artifact_id.clone()));
        let rec = put(
            "application/json",
            Some("ExecutionRecord"),
            lineage,
            &mut bytes.as_slice(),
        )?;
        let mut out = vec![rec, program];
        out.extend(record.stdout.artifact);
        out.extend(record.stderr.artifact);
        out.extend(record.outputs.into_iter().map(|o| o.artifact));
        Ok(out)
    }
}

// ------------------------------------------------------------------------------ inputs

fn datasets_of<'a>(
    datasets: Option<&'a dyn DatasetService>,
    input: &str,
) -> ServiceResult<&'a dyn DatasetService> {
    datasets.ok_or_else(|| {
        err(
            ErrorCode::CapabilityUnavailable,
            format!(
                "input {input:?} names a dataset or conditions reference, but no DatasetService is configured"
            ),
        )
    })
}

/// A bundle the scope may read: the service's own checks, then every partition of the bundle
/// against the scope (`ExecutionScope::require_partition`), so a service that missed a check still
/// cannot hand the runner a partition the scope does not hold.
fn readable_bundle(
    scope: &ExecutionScope,
    ds: &dyn DatasetService,
    id: &DatasetId,
) -> ServiceResult<DatasetBundle> {
    let b = ds.dataset(scope, id)?;
    for p in &b.partitions {
        scope.require_partition(*p)?;
    }
    Ok(b)
}

fn view_of<'a>(b: &'a DatasetBundle, view: &str) -> ServiceResult<&'a DatasetView> {
    b.views.iter().find(|v| v.name == view).ok_or_else(|| {
        let names: Vec<&str> = b.views.iter().map(|v| v.name.as_str()).collect();
        err(
            ErrorCode::NotFound,
            format!(
                "{} has no view {view:?}; its views are {}",
                b.dataset_id,
                names.join(", ")
            ),
        )
    })
}

/// Everything partitioned in the transitive lineage of `root` must be readable by `scope`: an
/// artifact computed from a partition the scope does not hold (an evaluation run's output in the
/// same workspace) is refused like the partition itself. Fails closed on anything it cannot place:
/// - a dataset is checked bundle by bundle; one the scope cannot see at all (the evaluator root
///   outside an evaluation scope) is refused, not skipped;
/// - a run is checked against every partition the frozen splits give it
///   ([`DatasetService::run_partitions`]); a run with no known partition, or no service to ask,
///   is refused;
/// - an observation is refused: its run cannot be resolved here;
/// - a lineage artifact that is not in the workspace store is refused: what it was computed from
///   is unknown.
///
/// Sources, evidence, releases, wiki pages and episodes carry no partition.
pub fn check_lineage(
    scope: &ExecutionScope,
    artifacts: &dyn ArtifactStore,
    datasets: Option<&dyn DatasetService>,
    root: &ArtifactRef,
) -> ServiceResult<()> {
    let mut seen_art = std::collections::HashSet::new();
    let mut seen_ds = std::collections::HashSet::new();
    let mut seen_run = std::collections::HashSet::new();
    seen_art.insert(root.artifact_id.as_str().to_string());
    let mut todo: Vec<EntityRef> = root.inputs.clone();
    while let Some(e) = todo.pop() {
        match e {
            EntityRef::Dataset(id) => {
                if !seen_ds.insert(id.as_str().to_string()) {
                    continue;
                }
                let refuse = |why: String| {
                    err(
                        ErrorCode::ScopeDenied,
                        format!(
                            "artifact {} was computed from dataset {id}, which this scope cannot read: {why}",
                            root.artifact_id
                        ),
                    )
                };
                let ds = datasets
                    .ok_or_else(|| refuse("no DatasetService to check its partitions".into()))?;
                readable_bundle(scope, ds, &id).map_err(|e| refuse(e.message))?;
            }
            EntityRef::Artifact(id) => {
                if !seen_art.insert(id.as_str().to_string()) {
                    continue;
                }
                match artifacts.stat(scope, &id) {
                    Ok(a) => todo.extend(a.inputs),
                    Err(e) if e.code == ErrorCode::NotFound => {
                        return Err(err(
                            ErrorCode::ScopeDenied,
                            format!(
                                "artifact {} was computed from artifact {id}, which is not in this workspace; its partitions are unknown",
                                root.artifact_id
                            ),
                        ));
                    }
                    Err(e) => return Err(e),
                }
            }
            EntityRef::Run(id) => {
                if !seen_run.insert(id.as_str().to_string()) {
                    continue;
                }
                let refuse = |why: String| {
                    err(
                        ErrorCode::ScopeDenied,
                        format!(
                            "artifact {} was computed from run {id}, which this scope cannot read: {why}",
                            root.artifact_id
                        ),
                    )
                };
                let ds = datasets
                    .ok_or_else(|| refuse("no DatasetService to check its partition".into()))?;
                let parts = ds
                    .run_partitions(scope, &id)
                    .map_err(|e| refuse(e.message))?;
                if parts.is_empty() {
                    return Err(refuse(
                        "no frozen split assigns it a partition (unknown partition)".into(),
                    ));
                }
                for p in parts {
                    scope
                        .require_partition(p)
                        .map_err(|e| refuse(e.to_string()))?;
                }
            }
            EntityRef::Observation(id) => {
                return Err(err(
                    ErrorCode::ScopeDenied,
                    format!(
                        "artifact {} was computed from observation {id}, whose run and partition cannot be resolved here",
                        root.artifact_id
                    ),
                ));
            }
            EntityRef::Source(_)
            | EntityRef::Evidence(_)
            | EntityRef::Release(_)
            | EntityRef::WikiPage(_)
            | EntityRef::Episode(_) => {}
        }
    }
    Ok(())
}

/// Resolve every by-reference input of `req` under `scope` without copying anything: the checks a
/// tool runs before it queues the job, so a refusal (scope, partition, lineage, missing view) is
/// immediate. The job repeats them when it stages the inputs.
pub fn check_inputs(
    scope: &ExecutionScope,
    req: &AnalysisExecuteRequest,
    artifacts: &dyn ArtifactStore,
    datasets: Option<&dyn DatasetService>,
) -> ServiceResult<()> {
    req.validate().map_err(ErrorEnvelope::from)?;
    for i in &req.inputs {
        if let Some(id) = &i.artifact_id {
            let art = artifacts.stat(scope, id)?;
            check_lineage(scope, artifacts, datasets, &art)?;
        } else if let Some(v) = &i.dataset_view {
            let b = readable_bundle(scope, datasets_of(datasets, &i.name)?, &v.dataset_id)?;
            view_of(&b, &v.view)?;
        } else if i.conditions.is_some() {
            datasets_of(datasets, &i.name)?.conditions_snapshot(scope)?;
        }
    }
    Ok(())
}

/// Copy `src` to `path`, failing unless the bytes hash to `expected`.
fn copy_verified(
    mut src: impl Read,
    path: &Path,
    expected: &Digest,
    what: &str,
) -> ServiceResult<()> {
    let mut dst = File::create(path).map_err(|e| internal("staging an input", e))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 64 << 10];
    loop {
        let n = match src.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(internal("reading an input", e)),
        };
        h.update(&buf[..n]);
        dst.write_all(&buf[..n])
            .map_err(|e| internal("staging an input", e))?;
    }
    dst.sync_all()
        .map_err(|e| internal("staging an input", e))?;
    if format!("sha256:{}", hex::encode(h.finalize())) != expected.as_str() {
        return Err(internal(
            "staging an input",
            format!("{what} does not match its digest"),
        ));
    }
    Ok(())
}

/// Every page of a conditions query as one table. The snapshot is read before and after: a table
/// assembled across a change of the underlying data is refused (retry), never mixed.
fn complete_conditions(
    scope: &ExecutionScope,
    ds: &dyn DatasetService,
    q: &ConditionsQuery,
) -> ServiceResult<(ConditionsTable, Digest)> {
    let before = ds.conditions_snapshot(scope)?;
    let mut page_q = q.clone();
    let mut table = ds.conditions(scope, &page_q)?;
    while let Some(c) = table.next_cursor.take() {
        if page_q.cursor.as_ref() == Some(&c) {
            return Err(internal(
                "conditions",
                "the service returned the same cursor twice",
            ));
        }
        page_q.cursor = Some(c);
        let page = ds.conditions(scope, &page_q)?;
        table.rows.extend(page.rows);
        table.next_cursor = page.next_cursor;
    }
    table.total_count = Some(table.rows.len() as u64);
    let after = ds.conditions_snapshot(scope)?;
    if before != after {
        return Err(ErrorEnvelope::new(
            ErrorCode::Conflict,
            "the conditions data changed while the table was read; retry",
            RetryPolicy::AfterDelay { seconds: 1 },
        ));
    }
    Ok((table, after))
}

fn stage_one(
    scope: &ExecutionScope,
    i: &AnalysisInput,
    artifacts: &dyn ArtifactStore,
    datasets: Option<&dyn DatasetService>,
    producer: &Producer,
    path: &Path,
) -> ServiceResult<MountedInput> {
    if let Some(id) = &i.artifact_id {
        let art = artifacts.stat(scope, id)?;
        check_lineage(scope, artifacts, datasets, &art)?;
        let src = artifacts.open(scope, id, 0, None)?;
        copy_verified(src, path, &art.sha256, &format!("artifact {id}"))?;
        return Ok(MountedInput {
            name: i.name.clone(),
            artifact: art,
            resolved: None,
        });
    }
    if let Some(v) = &i.dataset_view {
        let ds = datasets_of(datasets, &i.name)?;
        let b = readable_bundle(scope, ds, &v.dataset_id)?;
        let view = view_of(&b, &v.view)?;
        let (art, src) = ds.open_view(scope, &v.dataset_id, &v.view, 0, None)?;
        if art.sha256 != view.artifact.sha256 {
            return Err(internal(
                "staging an input",
                format!(
                    "view {} of {} is served with digest {}; the bundle says {}",
                    v.view, v.dataset_id, art.sha256, view.artifact.sha256
                ),
            ));
        }
        let what = format!("view {} of {}", v.view, v.dataset_id);
        copy_verified(src, path, &art.sha256, &what)?;
        return Ok(MountedInput {
            name: i.name.clone(),
            artifact: art,
            resolved: Some(ResolvedInput::DatasetView {
                dataset_id: v.dataset_id.clone(),
                view: v.view.clone(),
                bundle_digest: b.content_digest(),
                partitions: b.partitions.clone(),
                time_origin: b.time_origin.clone(),
                columns: view.columns.clone(),
                row_count: view.row_count,
            }),
        });
    }
    if let Some(q) = &i.conditions {
        let ds = datasets_of(datasets, &i.name)?;
        let (table, snapshot) = complete_conditions(scope, ds, q)?;
        let bytes = serde_json::to_vec_pretty(&table).map_err(|e| internal("conditions", e))?;
        let art = artifacts.put(
            scope,
            ArtifactDraft {
                media_type: "application/json".into(),
                contract: Some("ConditionsTable".into()),
                produced_by: producer.clone(),
                inputs: vec![],
                origin: table.origin,
            },
            &mut bytes.as_slice(),
        )?;
        copy_verified(bytes.as_slice(), path, &art.sha256, "the conditions table")?;
        return Ok(MountedInput {
            name: i.name.clone(),
            artifact: art,
            resolved: Some(ResolvedInput::Conditions {
                query: q.clone(),
                snapshot,
                row_count: table.rows.len() as u64,
            }),
        });
    }
    Err(err(
        ErrorCode::InvalidRequest,
        format!("input {:?} names no reference", i.name),
    ))
}

/// Resolve, copy and verify every input into `dir` (read-only for the program).
fn stage_inputs(
    scope: &ExecutionScope,
    req: &AnalysisExecuteRequest,
    artifacts: &dyn ArtifactStore,
    datasets: Option<&dyn DatasetService>,
    producer: &Producer,
    dir: &Path,
) -> ServiceResult<Vec<MountedInput>> {
    let mut mounted = Vec::with_capacity(req.inputs.len());
    for i in &req.inputs {
        let path = dir.join(&i.name);
        mounted.push(stage_one(scope, i, artifacts, datasets, producer, &path)?);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444))
                .map_err(|e| internal("staging an input", e))?;
        }
    }
    Ok(mounted)
}

enum OutKind {
    File(u64),
    Rejected(String),
}

/// Every entry under `/out`, depth first in name order, without following symbolic links.
fn walk_outputs(root: &Path) -> Vec<(String, PathBuf, OutKind)> {
    let mut found = Vec::new();
    let mut stack = vec![(String::new(), root.to_path_buf())];
    while let Some((prefix, dir)) = stack.pop() {
        let mut entries: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd.filter_map(Result::ok).collect(),
            Err(e) => {
                found.push((
                    prefix.trim_end_matches('/').to_string(),
                    dir,
                    OutKind::Rejected(format!("unreadable directory: {e}")),
                ));
                continue;
            }
        };
        entries.sort_by_key(|e| e.file_name());
        let mut subdirs = Vec::new();
        for e in entries {
            let name = e.file_name();
            let Some(name) = name.to_str().filter(|n| !n.contains('\\')) else {
                found.push((
                    format!("{prefix}{}", name.to_string_lossy()),
                    e.path(),
                    OutKind::Rejected("name is not portable UTF-8".into()),
                ));
                continue;
            };
            let rel = format!("{prefix}{name}");
            let path = e.path();
            match std::fs::symlink_metadata(&path) {
                Err(err) => found.push((rel, path, OutKind::Rejected(err.to_string()))),
                Ok(m) if m.file_type().is_symlink() => found.push((
                    rel,
                    path,
                    OutKind::Rejected("symbolic link (only regular files are kept)".into()),
                )),
                Ok(m) if m.is_dir() => subdirs.push((format!("{rel}/"), path)),
                Ok(m) if m.is_file() => found.push((rel, path, OutKind::File(m.len()))),
                Ok(_) => found.push((
                    rel,
                    path,
                    OutKind::Rejected("special file (only regular files are kept)".into()),
                )),
            }
        }
        stack.extend(subdirs.into_iter().rev());
    }
    found
}

/// Open a file only if it is still a regular file (never through a link).
fn open_regular(path: &Path) -> Result<File, String> {
    #[cfg(unix)]
    let f = {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc_o_nofollow())
            .open(path)
    };
    #[cfg(not(unix))]
    let f = File::open(path);
    let f = f.map_err(|e| format!("not opened: {e}"))?;
    match f.metadata() {
        Ok(m) if m.is_file() => Ok(f),
        _ => Err("not a regular file".into()),
    }
}

#[cfg(unix)]
fn libc_o_nofollow() -> i32 {
    // O_NOFOLLOW: 0o400000 on Linux (x86_64, aarch64).
    0o400000
}

/// Media type of an output by extension (`application/octet-stream` when unknown).
pub fn media_type_of(path: &str) -> &'static str {
    let ext = path
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "json" => "application/json",
        "jsonl" => "application/jsonl",
        "csv" => "text/csv",
        "txt" | "log" => "text/plain",
        "md" => "text/markdown",
        "parquet" => "application/vnd.apache.parquet",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "py" => "text/x-python",
        _ => "application/octet-stream",
    }
}

/// What the probe sees from inside the container.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProbeReport {
    uid: u32,
    gid: u32,
    cap_eff: Option<String>,
    no_new_privs: Option<String>,
    root_writable: bool,
    in_writable: bool,
    out_writable: bool,
    input: String,
    network: bool,
    ifaces: Option<Vec<String>>,
    docker_sock: bool,
    env: Vec<String>,
    python: String,
}

impl ProbeReport {
    fn check(&self, allowed_env: &BTreeSet<String>) -> Result<(), String> {
        let mut fail = Vec::new();
        if (self.uid, self.gid) != (65534, 65534) {
            fail.push(format!(
                "runs as {}:{}, not 65534:65534",
                self.uid, self.gid
            ));
        }
        if self
            .cap_eff
            .as_deref()
            .is_some_and(|c| !c.trim_start_matches('0').is_empty())
        {
            fail.push(format!("effective capabilities {:?}", self.cap_eff));
        }
        if self.no_new_privs.as_deref().is_some_and(|v| v != "1") {
            fail.push("no_new_privs is not set".into());
        }
        if self.root_writable {
            fail.push("the root filesystem is writable".into());
        }
        if self.in_writable {
            fail.push("/in is writable".into());
        }
        if !self.out_writable {
            fail.push("/out is not writable".into());
        }
        if self.input != "ignis-probe" {
            fail.push("the probe input was not mounted".into());
        }
        if self.network {
            fail.push("the network is reachable".into());
        }
        if let Some(i) = &self.ifaces {
            if i.iter().any(|n| n != "lo") {
                fail.push(format!("network interfaces {i:?}"));
            }
        }
        if self.docker_sock {
            fail.push("a container engine socket is visible".into());
        }
        let extra: Vec<&String> = self
            .env
            .iter()
            .filter(|k| !allowed_env.contains(*k))
            .collect();
        if !extra.is_empty() {
            fail.push(format!("unexpected environment variables {extra:?}"));
        }
        if fail.is_empty() {
            Ok(())
        } else {
            Err(format!("isolation probe failed: {}", fail.join("; ")))
        }
    }
}

/// The isolation probe (runs as the program of a normal execution).
const PROBE: &str = r#"
import json, os, socket, sys
def status(key):
    try:
        for line in open("/proc/self/status"):
            if line.startswith(key + ":"):
                return line.split()[1]
    except OSError:
        return None
def writable(path):
    try:
        with open(path, "w") as f:
            f.write("x")
        os.remove(path)
        return True
    except OSError:
        return False
def network():
    for host in ("1.1.1.1", "8.8.8.8"):
        try:
            socket.create_connection((host, 53), timeout=2).close()
            return True
        except OSError:
            pass
    try:
        socket.getaddrinfo("example.com", 443)
        return True
    except OSError:
        return False
print(json.dumps({
    "uid": os.getuid(),
    "gid": os.getgid(),
    "cap_eff": status("CapEff"),
    "no_new_privs": status("NoNewPrivs"),
    "root_writable": writable("/ignis_probe"),
    "in_writable": writable("/in/ignis_probe"),
    "out_writable": writable("/out/ignis_probe"),
    "input": open("/in/probe.txt").read(),
    "network": network(),
    "ifaces": sorted(os.listdir("/sys/class/net")) if os.path.isdir("/sys/class/net") else None,
    "docker_sock": any(os.path.exists(p) for p in ("/var/run/docker.sock", "/run/docker.sock")),
    "env": sorted(os.environ),
    "python": sys.version.split()[0],
}))
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> SandboxConfig {
        SandboxConfig {
            engine: "docker".into(),
            image: Digest::parse(&format!("sha256:{}", "a".repeat(64))).unwrap(),
            oci_runtime: "runsc".into(),
            require_gvisor: true,
            work_dir: "/srv/ignis/runner".into(),
            limits: RunnerLimits {
                wall_seconds: 600,
                memory_bytes: 1 << 31,
                cpu_millis: 1500,
                pids: 256,
                tmp_bytes: 1 << 28,
                output_bytes: 1 << 30,
                stream_bytes: 1 << 24,
            },
        }
    }

    #[test]
    fn the_container_is_started_closed_and_sees_only_its_three_mounts() {
        let a = run_args(&cfg(), "own", "n", "/r/job", "/r/in", "/r/out");
        let pair = |flag: &str, value: &str| a.windows(2).any(|w| w[0] == flag && w[1] == value);
        assert!(pair("--network", "none"));
        assert!(pair("--cap-drop", "ALL"));
        assert!(pair("--security-opt", "no-new-privileges"));
        assert!(pair("--user", RUN_AS));
        assert!(pair("--runtime", "runsc"));
        assert!(pair("--memory-swap", &(1u64 << 31).to_string()), "no swap");
        assert!(pair("--cpus", "1.500"));
        assert!(a.contains(&"--read-only".to_string()));
        let mounts: Vec<&String> = a
            .windows(2)
            .filter(|w| w[0] == "--mount")
            .map(|w| &w[1])
            .collect();
        assert_eq!(
            mounts,
            [
                "type=bind,source=/r/job,target=/job,readonly",
                "type=bind,source=/r/in,target=/in,readonly",
                "type=bind,source=/r/out,target=/out",
            ]
        );
        for forbidden in [
            "-v",
            "--volume",
            "--privileged",
            "--env-file",
            "--network=host",
            "--pid",
            "--ipc",
            "--device",
            "--cap-add",
            "--volumes-from",
        ] {
            assert!(!a.iter().any(|x| x == forbidden), "{forbidden}");
        }
        let env: Vec<&String> = a
            .windows(2)
            .filter(|w| w[0] == "--env")
            .map(|w| &w[1])
            .collect();
        assert_eq!(env.len(), FIXED_ENV.len(), "only fixed variables: {env:?}");
        // The image is addressed by id and the program cannot choose the interpreter flags.
        let tail = &a[a.len() - 3..];
        assert_eq!(tail[0], cfg().image.to_string());
        assert_eq!(tail[1..], ["-I", "/job/main.py"]);
    }

    #[test]
    fn mount_sources_that_would_change_the_mount_spec_are_refused() {
        assert!(mount_source(Path::new("/srv/a,readonly=false")).is_err());
        assert!(mount_source(Path::new("/srv/ok")).is_ok());
        assert_eq!(
            mount_source(Path::new(r"\\?\E:\runs\job")).unwrap(),
            r"E:\runs\job"
        );
    }

    #[test]
    fn gvisor_is_recognised_only_by_its_runtime() {
        assert_eq!(isolation_of("runsc"), RunnerIsolation::Gvisor);
        assert_eq!(isolation_of("runsc-kvm"), RunnerIsolation::Gvisor);
        assert_eq!(isolation_of("runc"), RunnerIsolation::Container);
        let mut c = cfg();
        c.oci_runtime = "runc".into();
        let e = Sandbox::preflight(c).unwrap_err();
        assert!(e.contains("requires gVisor"), "{e}");
    }

    #[test]
    fn outputs_never_follow_links_and_keep_only_regular_files() {
        let d = tempfile::tempdir().unwrap();
        let out = d.path();
        std::fs::write(out.join("b.json"), b"{}").unwrap();
        std::fs::create_dir(out.join("sub")).unwrap();
        std::fs::write(out.join("sub").join("a.csv"), b"x,y\n").unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("secret"), b"host").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(outside.path().join("secret"), out.join("link")).unwrap();
        let found = walk_outputs(out);
        let files: Vec<&str> = found
            .iter()
            .filter(|(_, _, k)| matches!(k, OutKind::File(_)))
            .map(|(r, _, _)| r.as_str())
            .collect();
        assert_eq!(files, ["b.json", "sub/a.csv"]);
        #[cfg(unix)]
        assert!(found
            .iter()
            .any(|(r, _, k)| r == "link" && matches!(k, OutKind::Rejected(_))));
    }

    #[test]
    fn probe_failures_name_every_broken_property() {
        let r = ProbeReport {
            uid: 0,
            gid: 0,
            cap_eff: Some("00000000a80425fb".into()),
            no_new_privs: Some("0".into()),
            root_writable: true,
            in_writable: true,
            out_writable: true,
            input: "ignis-probe".into(),
            network: true,
            ifaces: Some(vec!["eth0".into(), "lo".into()]),
            docker_sock: true,
            env: vec!["PATH".into(), "IGNIS_MCP_TOKEN".into()],
            python: "3.12".into(),
        };
        let e = r.check(&BTreeSet::from(["PATH".to_string()])).unwrap_err();
        for part in [
            "65534",
            "capabilities",
            "no_new_privs",
            "root filesystem",
            "/in",
            "network is reachable",
            "eth0",
            "socket",
            "IGNIS_MCP_TOKEN",
        ] {
            assert!(e.contains(part), "{part}: {e}");
        }
    }

    #[test]
    fn a_kept_stream_prefix_never_exceeds_its_budget() {
        let d = tempfile::tempdir().unwrap();
        let c = capture(&b"0123456789"[..], d.path().join("s"), 4).unwrap();
        assert_eq!(c.total, 10);
        assert_eq!(std::fs::read(&c.path).unwrap(), b"0123");
    }
}

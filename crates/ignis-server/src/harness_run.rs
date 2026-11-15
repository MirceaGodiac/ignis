//! `ignis harness run` (task M10): run a set of dev questions under a named harness variant and
//! write one `TraceBundle` per (task, repetition) plus a summary.
//!
//! It drives an in-process `/api/v1` instance (the same routes, runtime bridge, MCP tools and state
//! store `ignis serve` uses) over its own state directory `<out>/state`, so a benchmark run is the
//! product path, not a second agent loop. Each task is an episode: create (idempotent on a key
//! derived from variant, task and repetition, so an interrupted run resumes instead of repeating
//! work), post the question, wait for the turn to end, export its trace
//! ([`ignis_research::trace::build_trace`]).
//!
//! Guards (fail closed, before anything starts):
//! - no path (task file, variant directory, output) under a `holdout/` directory;
//! - the task set must declare a development partition, never `test`;
//! - the run's scope is the host-derived research scope (train only); an evaluation scope or one
//!   that grants `test` is refused ([`check_scope`]);
//! - nothing runs without `--execute` (a real runtime spends money); without it the command prints
//!   the plan. Offline tests pass `--execute` with the fake runtimes.
//!
//! **Seams.** Task sets load through [`TaskLoader`] (`--tasks-format`): today the dev-only fixture
//! format ([`JsonTaskFile`], `origin = synthetic_fixture`); Tudor's T10 dev set plugs in as another
//! loader when its `GoldTask` contract lands on `main` (the trace never carries a task's answer).
//! Comparisons of variants (T11: paired statistics, ablations, seeds) and offline harness search
//! (M10b: GEPA / Meta-Harness) read the traces and `summary.json`; neither is implemented here.

use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ignis_core::contracts::{
    CostStatus, EpisodeId, EpisodeStatus, ExecutionScope, HarnessRun, Partition, ScopeRole,
    TokenTotals, WorkspaceMetrics,
};
use ignis_research::metrics::{self, PriceList};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::v1::agent::CodexArgs;
use crate::v1::io::AuthMode;
use crate::{ServeArgs, Services};

// ------------------------------------------------------------------------------ CLI

#[derive(clap::Subcommand, Debug)]
pub enum HarnessCmd {
    /// Run a dev task set under a harness variant; write TraceBundles and a summary.
    Run(HarnessRunArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct HarnessRunArgs {
    /// The variant: a harness directory (`investigator.md`, `skills/`), or the `sha256:` digest
    /// of the bundle built into this binary (checked against the pin of the first episode).
    #[arg(long)]
    pub variant: String,
    /// The task set (dev only).
    #[arg(long)]
    pub tasks: PathBuf,
    #[arg(long, value_enum, default_value_t)]
    pub tasks_format: TaskFormat,
    /// Output directory: `state/`, `traces/<task>.r<n>.json`, `summary.json`.
    #[arg(long)]
    pub out: PathBuf,
    /// Repetitions of every task under the variant.
    #[arg(long, default_value_t = 1)]
    pub repeat: u32,
    /// Seconds a task may take before its episode is cancelled and recorded as timed out.
    #[arg(long, default_value_t = 900)]
    pub task_timeout_s: u64,
    /// Versioned price list; unset = the one built into the binary.
    #[arg(long, env = "IGNIS_PRICE_LIST")]
    pub price_list: Option<PathBuf>,
    /// Serve the model families, as `ignis serve --enable-models` (served releases from
    /// `IGNIS_WSP_SERVED`, default `configs/models/served`).
    #[arg(long, env = "IGNIS_ENABLE_MODELS")]
    pub enable_models: bool,
    /// Actually run (starts the agent runtime: real runtimes spend money). Without it, print the
    /// plan and exit.
    #[arg(long)]
    pub execute: bool,
    /// Agent runtime (same flags as `ignis serve`; `--harness-dir` is replaced by `--variant`).
    #[command(flatten)]
    pub runtime: CodexArgs,
}

// ------------------------------------------------------------------------------ task sets

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum TaskFormat {
    /// The dev fixture format (`ignis.harness.tasks.v0`).
    #[default]
    Json,
    /// Tudor's T10 gold set (`GoldTask`); not available until its contract is on `main`.
    Gold,
}

/// One question of a task set. Never carries the expected answer: scoring is the evaluator's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessTask {
    pub task_id: String,
    pub question: String,
}

/// A loaded task set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskSet {
    pub schema: String,
    pub task_set: String,
    /// `synthetic_fixture` for the fixture; the T10 set says what it is.
    pub origin: ignis_core::contracts::Origin,
    /// Must be a development partition (`dev`, `train`, `validation`).
    pub partition: String,
    pub tasks: Vec<HarnessTask>,
}

/// Where task sets come from. The T10 dev set is one more implementation.
pub trait TaskLoader {
    fn load(&self, path: &Path) -> Result<TaskSet, String>;
}

/// `ignis.harness.tasks.v0` JSON files (`fixtures/harness/dev_tasks.json`).
pub struct JsonTaskFile;

impl TaskLoader for JsonTaskFile {
    fn load(&self, path: &Path) -> Result<TaskSet, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("task set {}: {e}", path.display()))?;
        let set: TaskSet =
            serde_json::from_str(&text).map_err(|e| format!("task set {}: {e}", path.display()))?;
        if set.schema != "ignis.harness.tasks.v0" {
            return Err(format!(
                "task set schema {:?} is not ignis.harness.tasks.v0",
                set.schema
            ));
        }
        Ok(set)
    }
}

/// Placeholder for the T10 loader: refuses until the `GoldTask` contract is on `main`.
pub struct GoldTaskSet;

impl TaskLoader for GoldTaskSet {
    fn load(&self, _: &Path) -> Result<TaskSet, String> {
        Err(
            "the T10 gold-set loader is not available yet: its GoldTask contract (K1-evaluation) \
             is not on main; use --tasks-format json with a dev fixture"
                .into(),
        )
    }
}

pub fn loader(format: TaskFormat) -> Box<dyn TaskLoader> {
    match format {
        TaskFormat::Json => Box::new(JsonTaskFile),
        TaskFormat::Gold => Box::new(GoldTaskSet),
    }
}

// ------------------------------------------------------------------------------ guards

/// Refuse a path under a `holdout/` directory (the private hold-out of the gold set), however it
/// is spelled (relative, `..`, case).
pub fn check_not_holdout(what: &str, path: &Path) -> Result<(), String> {
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let full = std::fs::canonicalize(&abs).unwrap_or(abs);
    let named = |p: &Path| {
        p.components().any(|c| {
            matches!(c, Component::Normal(s) if s.to_string_lossy().eq_ignore_ascii_case("holdout"))
        })
    };
    if named(path) || named(&full) {
        return Err(format!(
            "{what} {} is under holdout/: the hold-out is read only by the evaluator",
            path.display()
        ));
    }
    Ok(())
}

/// A harness run never reads the test partition: an evaluation scope or one granting `test` is
/// refused.
pub fn check_scope(scope: &ExecutionScope) -> Result<(), String> {
    if scope.role == ScopeRole::Evaluation || scope.partitions.contains(&Partition::Test) {
        return Err(format!(
            "scope {} grants the test partition (or is an evaluation scope): harness runs are dev only",
            scope.scope_id
        ));
    }
    Ok(())
}

fn check_task_set(set: &TaskSet) -> Result<(), String> {
    let p = set.partition.to_ascii_lowercase();
    if !matches!(p.as_str(), "dev" | "train" | "validation") {
        return Err(format!(
            "task set {} declares partition {:?}: harness runs take development tasks only",
            set.task_set, set.partition
        ));
    }
    if set.tasks.is_empty() {
        return Err(format!("task set {} has no tasks", set.task_set));
    }
    let mut ids = std::collections::BTreeSet::new();
    for t in &set.tasks {
        if t.task_id.trim().is_empty() || t.question.trim().is_empty() || !ids.insert(&t.task_id) {
            return Err(format!(
                "task {:?}: empty or duplicate id, or empty question",
                t.task_id
            ));
        }
        if !t
            .task_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(format!(
                "task id {:?}: letters, digits, - and _ only",
                t.task_id
            ));
        }
    }
    Ok(())
}

// ------------------------------------------------------------------------------ variant

/// How the variant reaches the runtime.
#[derive(Debug, Clone, PartialEq)]
enum Variant {
    /// A harness directory, labelled by its name.
    Dir(PathBuf),
    /// The built-in bundle, expected to have this digest.
    Digest(String),
}

impl Variant {
    fn parse(s: &str) -> Result<Self, String> {
        if let Some(hex) = s.strip_prefix("sha256:") {
            if hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Ok(Variant::Digest(s.to_ascii_lowercase()));
            }
            return Err(format!("{s:?} is not a sha256:<64 hex> digest"));
        }
        let dir = PathBuf::from(s);
        check_not_holdout("variant", &dir)?;
        if !dir.join("investigator.md").is_file() {
            return Err(format!(
                "variant {s:?} is neither a sha256 digest nor a harness directory (no investigator.md)"
            ));
        }
        Ok(Variant::Dir(dir))
    }
}

// ------------------------------------------------------------------------------ summary

/// One (task, repetition) of the run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunEntry {
    pub task_id: String,
    pub repetition: u32,
    pub episode_id: String,
    /// Episode status when the turn ended (`completed`, `failed`, …) or `timed_out`.
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_file: Option<String>,
    /// Why no trace was written, when none was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_error: Option<String>,
    /// Status of the last `answer.completed` (the host's validation outcome), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_status: Option<String>,
    pub tool_calls: u64,
    pub tokens: TokenTotals,
    pub cost: CostStatus,
}

/// `summary.json`. T11 and M10b read the traces it lists; it carries no comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunSummary {
    pub schema: String,
    pub variant: String,
    /// Digest and label of the harness the episodes were pinned to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<ignis_core::contracts::HarnessRef>,
    pub task_set: String,
    pub task_origin: ignis_core::contracts::Origin,
    pub tasks_file: String,
    pub repeat: u32,
    pub runs: Vec<RunEntry>,
    pub aggregate: WorkspaceMetrics,
    /// What this summary deliberately does not contain.
    pub not_included: String,
}

// ------------------------------------------------------------------------------ run

fn json_body(v: Value) -> Body {
    Body::from(v.to_string())
}

async fn call(
    r: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let req = Request::builder().method(method).uri(path);
    let req = match body {
        Some(b) => req
            .header("content-type", "application/json")
            .body(json_body(b)),
        None => req.body(Body::empty()),
    }
    .expect("request");
    match r.clone().oneshot(req).await {
        Ok(resp) => {
            let st = resp.status();
            let bytes = axum::body::to_bytes(resp.into_body(), 1 << 26)
                .await
                .unwrap_or_default();
            (st, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"message": e.to_string()}),
        ),
    }
}

/// A stable idempotency key (letters, digits, `-`).
fn key(parts: &[&str]) -> String {
    let h = ignis_core::hashing::sha256_hex(parts.join("\u{1f}").as_bytes());
    format!("harness-{}", &h[..32])
}

/// The `ignis serve` arguments of a harness run: the same data roots, served models, runner and
/// sources as `ignis serve` (its defaults and `IGNIS_*` environment), with the harness's own
/// state directory, loopback ports, local auth and local artifacts, and the variant's runtime.
pub fn serve_args(run: &HarnessRunArgs) -> anyhow::Result<ServeArgs> {
    #[derive(clap::Parser)]
    struct Serve {
        #[command(flatten)]
        args: ServeArgs,
    }
    let mut s = <Serve as clap::Parser>::try_parse_from(["ignis-serve"])
        .map_err(|e| anyhow::anyhow!("serve settings of the harness run: {e}"))?
        .args;
    let mut runtime = run.runtime.clone();
    runtime.harness_dir = match Variant::parse(&run.variant).map_err(|e| anyhow::anyhow!(e))? {
        Variant::Dir(d) => Some(std::path::absolute(&d).unwrap_or(d)),
        Variant::Digest(_) => None,
    };
    let none = run.out.join("none");
    s.port = 0;
    s.host = "127.0.0.1".into();
    s.mcp_port = 0;
    s.dev = false;
    s.internal_routes = false;
    s.web_dist = none.join("web");
    s.state_dir = std::path::absolute(run.out.join("state"))?;
    s.auth = AuthMode::Local;
    s.artifact_backend = Default::default();
    s.enable_models = run.enable_models;
    s.price_list = None;
    s.codex = runtime;
    Ok(s)
}

/// Run the command. `services` are the lane services the episodes' tools run on (as for
/// `ignis serve`).
pub fn run(cmd: HarnessCmd, services: Services) -> anyhow::Result<()> {
    let HarnessCmd::Run(args) = cmd;
    let summary = run_tasks(&args, services)?;
    match summary {
        None => {}
        Some(s) => println!(
            "{} runs of {} under {}: summary in {}",
            s.runs.len(),
            s.task_set,
            s.variant,
            args.out.join("summary.json").display()
        ),
    }
    Ok(())
}

/// Validate, then (with `--execute`) run every task; returns the summary written, or `None` for a
/// dry run.
pub fn run_tasks(args: &HarnessRunArgs, services: Services) -> anyhow::Result<Option<RunSummary>> {
    let fail = |e: String| anyhow::anyhow!(e);
    check_not_holdout("task set", &args.tasks).map_err(fail)?;
    check_not_holdout("output", &args.out).map_err(fail)?;
    if args.runtime.harness_dir.is_some() {
        anyhow::bail!("--harness-dir / IGNIS_HARNESS_DIR is not used by harness runs: name the variant with --variant");
    }
    anyhow::ensure!(args.repeat >= 1, "--repeat is at least 1");
    let variant = Variant::parse(&args.variant).map_err(fail)?;
    let set = loader(args.tasks_format).load(&args.tasks).map_err(fail)?;
    check_task_set(&set).map_err(fail)?;
    let scope = crate::v1::io::research_scope("local", false);
    check_scope(&scope).map_err(fail)?;
    let prices = match &args.price_list {
        Some(p) => PriceList::load(p).map_err(fail)?,
        None => PriceList::builtin(),
    };

    if !args.execute {
        println!(
            "dry run (nothing started; add --execute to run, which spends money with a real runtime):"
        );
        println!(
            "  variant {} · task set {} ({:?}, partition {}) · {} tasks × {} repetitions · runtime {:?} · out {}",
            args.variant,
            set.task_set,
            set.origin,
            set.partition,
            set.tasks.len(),
            args.repeat,
            args.runtime.agent_runtime,
            args.out.display()
        );
        for t in &set.tasks {
            println!("  - {}: {}", t.task_id, t.question);
        }
        return Ok(None);
    }

    std::fs::create_dir_all(args.out.join("traces"))?;
    // The runtime first: the app binds the IGNIS MCP listener only inside a Tokio runtime, and
    // without it the agent runs with no IGNIS tools at all.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let serve = serve_args(args)?;
    let router = rt.block_on(async move { crate::app(serve, services) });
    let state = args.out.join("state");
    let store = crate::observe::open_store(&state)?;
    let artifacts = ignis_research::LocalArtifactStore::open(state.join("artifacts"))
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let (_, caps) = rt.block_on(call(&router, "GET", "/api/v1/capabilities", None));
    if caps["agent_runtime"]["available"] != json!(true) {
        anyhow::bail!(
            "the agent runtime is unavailable: {}",
            caps["agent_runtime"]["detail"]
                .as_str()
                .unwrap_or("unknown")
        );
    }

    check_tools(&caps)?;

    let mut runs = vec![];
    let mut episodes = vec![];
    let mut harness = None;
    for t in &set.tasks {
        for rep in 0..args.repeat {
            let ep_key = key(&[
                &args.variant,
                &set.task_set,
                &t.task_id,
                &rep.to_string(),
                "episode",
            ]);
            let (st, ep) = rt.block_on(call(
                &router,
                "POST",
                "/api/v1/episodes",
                Some(json!({"goal": format!("{} / {}", set.task_set, t.task_id), "client_request_id": ep_key})),
            ));
            anyhow::ensure!(
                st.is_success(),
                "create the episode of {}: {st} {ep}",
                t.task_id
            );
            let id = ep["episode_id"].as_str().unwrap_or_default().to_string();
            let msg_key = key(&[
                &args.variant,
                &set.task_set,
                &t.task_id,
                &rep.to_string(),
                "message",
            ]);
            let (st, acc) = rt.block_on(call(
                &router,
                "POST",
                &format!("/api/v1/episodes/{id}/messages"),
                Some(json!({"text": t.question, "client_request_id": msg_key})),
            ));
            anyhow::ensure!(
                st.is_success(),
                "post the question of {}: {st} {acc}",
                t.task_id
            );
            let outcome = rt.block_on(wait_turn(
                &router,
                &id,
                Duration::from_secs(args.task_timeout_s),
            ));
            let ep_id = EpisodeId::new(&id)?;
            let rec = store
                .episode(&scope, &ep_id)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if let (Variant::Digest(want), Some(pin)) = (&variant, &rec.pins.harness) {
                if pin.digest.as_str() != want {
                    anyhow::bail!(
                        "variant {want} is not the bundle this binary runs ({}); pass a harness directory instead",
                        pin.digest.as_str()
                    );
                }
            }
            if harness.is_none() {
                harness = rec.pins.harness.clone();
            }
            let run_ref = HarnessRun {
                variant: args.variant.clone(),
                task_id: t.task_id.clone(),
                repetition: rep,
            };
            let measure = metrics::measure_episode(&store, &scope, &ep_id, &prices)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let events =
                metrics::read_log(&store, &scope, &ep_id).map_err(|e| anyhow::anyhow!("{e}"))?;
            let answer_status = events.iter().rev().find_map(|e| match &e.body {
                ignis_core::contracts::EventBody::AnswerCompleted { status, .. } => {
                    serde_json::to_value(status)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_string))
                }
                _ => None,
            });
            let mut entry = RunEntry {
                task_id: t.task_id.clone(),
                repetition: rep,
                episode_id: id.clone(),
                outcome,
                trace_id: None,
                trace_file: None,
                trace_error: None,
                answer_status,
                tool_calls: measure.metrics.tools.iter().map(|x| x.calls).sum(),
                tokens: measure.metrics.tokens.clone(),
                cost: measure.metrics.cost.clone(),
            };
            match ignis_research::trace::build_trace(
                &store,
                &artifacts,
                &scope,
                &ep_id,
                Some(run_ref),
                &prices,
            ) {
                Ok(bundle) => {
                    let name = format!("{}.r{rep}.json", t.task_id);
                    let path = args.out.join("traces").join(&name);
                    std::fs::write(&path, serde_json::to_vec_pretty(&bundle)?)?;
                    entry.trace_id = Some(bundle.trace_id.to_string());
                    entry.trace_file = Some(format!("traces/{name}"));
                }
                Err(e) => entry.trace_error = Some(e.to_string()),
            }
            tracing::info!(
                target: "ignis::harness",
                task = %t.task_id, repetition = rep, episode = %id, outcome = %entry.outcome,
                trace = entry.trace_id.as_deref().unwrap_or("none"),
                "harness task done"
            );
            runs.push(entry);
            episodes.push(measure);
        }
    }
    let summary = RunSummary {
        schema: "ignis.harness.summary.v0".into(),
        variant: args.variant.clone(),
        harness,
        task_set: set.task_set.clone(),
        task_origin: set.origin,
        tasks_file: args.tasks.display().to_string(),
        repeat: args.repeat,
        runs,
        aggregate: metrics::aggregate(&episodes, &prices),
        not_included: "no scores, expected answers or comparisons: scoring is the evaluator's \
                       (T10/T11), variant search is M10b"
            .into(),
    };
    std::fs::write(
        args.out.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    drop(router);
    rt.shutdown_timeout(Duration::from_secs(5));
    Ok(Some(summary))
}

/// Fail closed when the agent would run without IGNIS tools: no MCP endpoint, or no registered
/// tool available. Unconfigured data services are reported (the run is still the product path).
pub fn check_tools(caps: &Value) -> anyhow::Result<()> {
    if caps["mcp"]["available"] != json!(true) {
        anyhow::bail!(
            "the agent would run without IGNIS tools: MCP {}",
            caps["mcp"]["detail"].as_str().unwrap_or("unavailable")
        );
    }
    let tools = caps["tools"].as_array().cloned().unwrap_or_default();
    let available: Vec<&str> = tools
        .iter()
        .filter(|t| t["available"] == json!(true))
        .filter_map(|t| t["name"].as_str())
        .collect();
    anyhow::ensure!(
        !available.is_empty(),
        "the agent would run without IGNIS tools: none of {} is available",
        tools.len()
    );
    for svc in ["knowledge", "datasets", "models"] {
        if caps[svc]["available"] != json!(true) {
            eprintln!(
                "harness run: {svc}: {}",
                caps[svc]["detail"].as_str().unwrap_or("unavailable")
            );
        }
    }
    eprintln!(
        "harness run: {} IGNIS tools available over MCP: {}",
        available.len(),
        available.join(", ")
    );
    Ok(())
}

/// Wait until the episode leaves the running states; cancel it when `timeout` passes.
async fn wait_turn(r: &axum::Router, id: &str, timeout: Duration) -> String {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let (_, ep) = call(r, "GET", &format!("/api/v1/episodes/{id}"), None).await;
        let status: Option<EpisodeStatus> = serde_json::from_value(ep["status"].clone()).ok();
        match status {
            Some(
                EpisodeStatus::Completed
                | EpisodeStatus::Failed
                | EpisodeStatus::Cancelled
                | EpisodeStatus::WaitingUser
                | EpisodeStatus::Paused,
            ) => {
                return ep["status"].as_str().unwrap_or("unknown").to_string();
            }
            _ if tokio::time::Instant::now() >= deadline => {
                let _ = call(r, "POST", &format!("/api/v1/episodes/{id}/cancel"), None).await;
                return "timed_out".into();
            }
            _ => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
}

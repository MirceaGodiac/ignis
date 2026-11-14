//! `ignis-server`: Axum + Tokio + Tower runtime API, started by `ignis serve`.
//!
//! Tokio serves I/O. Forecasts run **in process**: the binary injects the experts
//! (`Vec<Box<dyn Expert>>`) and, optionally, an acquisition scoring function ([`Services`]); the
//! server builds the causal [`PrefixBundle`] in Rust with the expert's own prospective policy and
//! runs `Expert::forecast` on one bounded Rayon pool through `spawn_blocking`. With `--dev`, a
//! second, fully separate origin serves the synthetic development fixture under `/fixtures/*` with
//! its own (synthetic) experts.
//!
//! `/api/v1` ([`v1`]) is the research runtime API: episodes, resumable SSE, artifacts and durable
//! jobs over `ignis-research`, with the lanes' services injected through [`Services::research`].
//! Transport limits are per router: the legacy `/api/*` and the v1 commands keep 30 s / 1 MiB, the
//! v1 event stream has no request timeout.

mod api;
mod evidence;
pub mod fixture;
pub mod ops;
mod refusal;
mod state;
pub mod v1;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use axum::Router;
use ignis_core::contracts::{AcquisitionAction, DecisionTrace, PrefixBundle};
use ignis_core::expert::Expert;
use ignis_core::registry::LoadedRelease;
use serde_json::Value;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use ignis_core::records::{Registry, RegistryPaths};

use crate::state::{DataOrigin, Origin};

/// Acquisition scoring hook for `POST /acquisition`.
///
/// Called with the verified release, the stored (isolation-checked) prefix of the analysis, the
/// candidate actions (observations recorded at or before the cutoff but not yet admitted; values
/// are not included) and the analysis' forecast settings. Returns the [`DecisionTrace`] of the
/// decision layer. Runs on the bounded Rayon pool, never on a Tokio worker. An `Err` is reported as
/// a typed `failed` refusal inside `scoring`; when no function is injected, `scoring` is a typed
/// `unsupported` refusal (`decision_unavailable`).
pub type AcquisitionScorer = Arc<
    dyn Fn(
            &LoadedRelease,
            &PrefixBundle,
            &[AcquisitionAction],
            &Value,
        ) -> anyhow::Result<DecisionTrace>
        + Send
        + Sync,
>;

/// What the binary injects into the server.
#[derive(Default)]
pub struct Services {
    /// Experts serving the registry origin (`/api/*`), e.g. collected from the model crates.
    pub experts: Vec<Box<dyn Expert>>,
    /// Experts serving the synthetic `/fixtures` origin only (dev mode). Never consulted for the
    /// registry; see [`fixture::experts`].
    pub fixture_experts: Vec<Box<dyn Expert>>,
    /// Decision function for `POST /acquisition` (registry origin and fixture origin alike).
    pub acquisition: Option<AcquisitionScorer>,
    /// Lane services behind `/api/v1` (knowledge, datasets, models); absent ones are reported as
    /// unavailable capabilities.
    pub research: v1::ResearchServices,
}

#[derive(clap::Args, Debug, Clone)]
pub struct ServeArgs {
    #[arg(long, env = "IGNIS_PORT", default_value_t = 8787)]
    pub port: u16,
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// Repository root (relative paths below resolve against it).
    #[arg(long, env = "IGNIS_ROOT", default_value = ".")]
    pub root: PathBuf,
    /// Registry directory (JSON lines, see `ignis_core::records`).
    #[arg(long, env = "IGNIS_RECORDS", default_value = "data/work/records")]
    pub data: PathBuf,
    /// Cohort files (`<hex>.json`) of the registry origin.
    #[arg(long, default_value = "data/work/cohorts")]
    pub cohorts: PathBuf,
    /// Frozen split files of the registry origin.
    #[arg(long, default_value = "data/splits")]
    pub splits: PathBuf,
    /// Raw samples root (BASS recordings for the overlay).
    #[arg(long, default_value = "data/raw/samples")]
    pub raw_samples: PathBuf,
    /// Artifacts root: models/<hex>/ (read) and analysis/, sealed/, reveals/ (write-once).
    #[arg(long, env = "IGNIS_ARTIFACTS", default_value = "artifacts")]
    pub artifacts: PathBuf,
    /// Evidence catalog (SQLite + FTS5).
    #[arg(
        long,
        env = "IGNIS_EVIDENCE_DB",
        default_value = "artifacts/evidence/evidence.sqlite"
    )]
    pub evidence_db: PathBuf,
    /// Threads of the bounded Rayon pool (forecasts, acquisition scoring, kernels).
    #[arg(long, default_value_t = 2)]
    pub compute_threads: usize,
    /// Serve the synthetic development fixture under /fixtures (never mixed with the registry).
    #[arg(long)]
    pub dev: bool,
    #[arg(long, default_value = "crates/ignis-server/fixtures")]
    pub fixtures: PathBuf,
    /// Serve a built web client (web/dist) at / if present.
    #[arg(long, default_value = "web/dist")]
    pub web_dist: PathBuf,
    /// Runtime state: `state.sqlite` (episodes, events, jobs) and `artifacts/` (content-addressed).
    #[arg(long, env = "IGNIS_STATE_DIR", default_value = "runs/server")]
    pub state_dir: PathBuf,
    /// How callers are identified (the host derives each request's ExecutionScope from it).
    #[arg(long, value_enum, env = "IGNIS_AUTH", default_value = "local")]
    pub auth: v1::io::AuthMode,
    /// Where `/api/v1` artifacts live.
    #[arg(
        long,
        value_enum,
        env = "IGNIS_ARTIFACT_BACKEND",
        default_value = "local"
    )]
    pub artifact_backend: ops::ArtifactBackend,
    /// R2 bucket (with `--artifact-backend r2`).
    #[command(flatten)]
    pub r2: ops::R2Args,
    /// Cloudflare Access verification (required by `--auth cloudflare-access`).
    #[command(flatten)]
    pub access: v1::access::AccessArgs,
    /// Also serve the internal registry routes (`/seal`, `/reveal`, `/admin/reload`). Off by
    /// default: they are development/evaluation tools, not part of the public runtime.
    #[arg(long)]
    pub internal_routes: bool,
    /// Serve the model families (FLEX/ACME/BASS) through `/api/v1` and the IGNIS tools. Off by
    /// default until their releases are qualified: the contracts and tool descriptors stay
    /// published, `models_inspect`/`models_forecast` are not registered and `/api/v1/capabilities`
    /// says why. The legacy `/api` lab routes (fitted baselines) are not affected.
    #[arg(long, env = "IGNIS_ENABLE_MODELS")]
    pub enable_models: bool,
    /// Largest single upload accepted by `/api/v1/uploads` (a storage budget, not a result limit).
    #[arg(long, env = "IGNIS_MAX_UPLOAD_BYTES", default_value_t = 2 << 30)]
    pub max_upload_bytes: u64,
    /// C0 contract fixtures served by `/api/fixtures/v1` in dev mode.
    #[arg(long, default_value = "fixtures/contracts")]
    pub contract_fixtures: PathBuf,
    /// Agent runtime (pinned Codex App Server); unset = not configured.
    #[command(flatten)]
    pub codex: v1::agent::CodexArgs,
    /// Port of the IGNIS MCP endpoint the runtime reaches the tools through. Always bound to
    /// 127.0.0.1; 0 picks a free port.
    #[arg(long, env = "IGNIS_MCP_PORT", default_value_t = 0)]
    pub mcp_port: u16,
    /// Isolated analysis runner (`analysis_execute`); unset image = not configured.
    #[command(flatten)]
    pub runner: v1::sandbox::RunnerArgs,
}

#[allow(clippy::too_many_arguments)]
fn origin(
    args: &ServeArgs,
    kind: DataOrigin,
    data: PathBuf,
    models_root: PathBuf,
    store_root: PathBuf,
    pool: Arc<rayon::ThreadPool>,
    experts: Vec<Box<dyn Expert>>,
    acquisition: Option<AcquisitionScorer>,
) -> Arc<Origin> {
    let runs_root = std::env::var_os("IGNIS_RUNS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("runs"));
    let reveal_log = match kind {
        DataOrigin::Registry => args
            .root
            .join(runs_root)
            .join("evaluation")
            .join("reveal_log.jsonl"),
        DataOrigin::DevelopmentFixture => store_root.join("reveal_log.jsonl"),
    };
    let evidence_db = match kind {
        DataOrigin::Registry => args.root.join(&args.evidence_db),
        DataOrigin::DevelopmentFixture => args.root.join(&args.fixtures).join("evidence.sqlite"),
    };
    let registry_paths = match kind {
        DataOrigin::Registry => RegistryPaths {
            cohorts: Some(args.root.join(&args.cohorts)),
            splits: Some(args.root.join(&args.splits)),
            vision: Some(store_root.join("vision")),
        },
        DataOrigin::DevelopmentFixture => RegistryPaths::default(),
    };
    Arc::new(Origin {
        origin: kind,
        registry: RwLock::new(Arc::new(Registry::load_with(&data, &registry_paths))),
        registry_paths,
        models_root,
        store_root,
        raw_samples_root: args.root.join(&args.raw_samples),
        experts,
        acquisition,
        releases: Mutex::new(Default::default()),
        evidence: evidence::Evidence::new(evidence_db),
        cache: moka::future::Cache::builder()
            .max_capacity(512)
            .time_to_idle(Duration::from_secs(3600))
            .build(),
        jobs: RwLock::new(Default::default()),
        pool,
        public_scope: v1::io::research_scope("local", false),
        internal_routes: args.internal_routes || kind == DataOrigin::DevelopmentFixture,
        reveal_log,
    })
}

/// Apply `--enable-models`: when off, no `ModelService` reaches `/api/v1` or the tools, and the
/// catalog reports the model tools as disabled rather than unconfigured.
fn model_gate(s: v1::ResearchServices, enabled: bool) -> v1::ResearchServices {
    if enabled {
        return s;
    }
    v1::ResearchServices {
        models: None,
        models_disabled: true,
        ..s
    }
}

/// Windows `canonicalize` returns verbatim paths (`\\?\D:\x`); logs prefer `D:\x`.
fn strip_verbatim(p: PathBuf) -> PathBuf {
    let s = p.display().to_string();
    match s.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => p,
    }
}

/// Build the router for both origins (registry at `/api`, fixture at `/api/fixtures` in dev mode).
pub fn app(mut args: ServeArgs, services: Services) -> Router {
    args.root = std::fs::canonicalize(&args.root)
        .map(strip_verbatim)
        .unwrap_or(args.root.clone());
    let pool = Arc::new(ignis_core::batch::bounded_pool(args.compute_threads));
    let access = access_verifier(&args);
    let artifacts = args.root.join(&args.artifacts);
    let Services {
        experts,
        fixture_experts,
        acquisition,
        research,
    } = services;
    let main_origin = origin(
        &args,
        DataOrigin::Registry,
        args.root.join(&args.data),
        artifacts.clone(),
        artifacts.clone(),
        pool.clone(),
        experts,
        acquisition.clone(),
    );
    let reg = main_origin.registry();
    tracing::info!(experiments = reg.experiments.len(), errors = reg.errors.len(), dir = %reg.dir.display(),
                   experts = main_origin.experts.len(), "registry loaded");

    let mut app = Router::new().merge(legacy_limits(api::router(main_origin.clone())));
    if args.dev {
        let fx = args.root.join(&args.fixtures);
        let fixture = origin(
            &args,
            DataOrigin::DevelopmentFixture,
            fx.join("registry"),
            fx.join("store"),
            artifacts.join("dev-fixture-runs"),
            pool.clone(),
            fixture_experts,
            acquisition,
        );
        tracing::warn!(
            experiments = fixture.registry().experiments.len(),
            "serving DEVELOPMENT FIXTURE (synthetic) under /fixtures"
        );
        app = app.nest("/fixtures", legacy_limits(api::router(fixture)));
    }
    let state_dir = args.root.join(&args.state_dir);
    // One preflight per process; one bridge (and CODEX_HOME tree) per state store.
    let codex = blocking_startup(|| {
        v1::agent::Preflighted::run(&args.codex, &state_dir.join("codex"), args.auth)
    });
    // One runner per process, shared by the instances (each run is scoped by its caller).
    let (sandbox, runner_report) =
        blocking_startup(|| v1::sandbox::preflight(&args.runner, &state_dir, args.auth));
    let gate = |s: v1::ResearchServices| model_gate(s, args.enable_models);
    let research = gate(v1::ResearchServices {
        sandbox: sandbox.clone(),
        ..research
    });
    // The IGNIS tools reach the runtime over a loopback-only MCP listener, never through `/api`.
    let mut mcp = McpListener::bind(&args, &codex);
    let registry_v1 = v1_instance(
        &args,
        &codex,
        &state_dir,
        research,
        ignis_core::contracts::Origin::System,
        None,
        "registry",
        mcp.as_mut(),
        runner_report.clone(),
        access.clone(),
    );
    app = app.nest("/v1", v1::build(registry_v1));
    if args.dev {
        // Same routes, backed by the C0 contract fixtures, in its own state directory.
        match v1::fixture::Fixture::load(&args.root.join(&args.contract_fixtures)) {
            Ok(fx) => {
                let fixture_v1 = v1_instance(
                    &args,
                    &codex,
                    &state_dir.join("dev-fixture"),
                    gate(v1::ResearchServices {
                        sandbox: sandbox.clone(),
                        ..fx.services.clone()
                    }),
                    ignis_core::contracts::Origin::SyntheticFixture,
                    Some(fx),
                    "fixture",
                    mcp.as_mut(),
                    runner_report.clone(),
                    access.clone(),
                );
                tracing::warn!("serving C0 CONTRACT FIXTURES (synthetic) under /api/fixtures/v1");
                app = app.nest("/fixtures/v1", v1::build(fixture_v1));
            }
            Err(e) => {
                tracing::error!(error = %e, "contract fixtures not loaded; /api/fixtures/v1 is off")
            }
        }
    }
    if let Some(m) = mcp {
        m.serve();
    }
    let api = Router::new().nest("/api", app);
    let dist = args.root.join(&args.web_dist);
    let app = if dist.join("index.html").exists() {
        api.fallback_service(ServeDir::new(&dist).fallback(ServeFile::new(dist.join("index.html"))))
    } else {
        api
    };
    // Behind Access every route (legacy `/api/*`, static files) needs the origin assertion; `/api/v1`
    // routes additionally verify the user's. `/livez` is the process-level liveness probe for the
    // local supervisor (no data, no identity).
    let app = match args.auth {
        v1::io::AuthMode::Local => app,
        v1::io::AuthMode::CloudflareAccess => app.layer(axum::middleware::from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                let access = access.clone();
                async move { require_origin(access, req, next).await }
            },
        )),
    };
    app.route("/livez", axum::routing::get(|| async { "ok" }))
        .layer(TraceLayer::new_for_http())
        .layer(tower::limit::ConcurrencyLimitLayer::new(256))
}

/// The Access verifier for `--auth cloudflare-access` (`None` in local mode, or when the
/// configuration is invalid: then every request is refused, see [`run`] which refuses to start).
fn access_verifier(args: &ServeArgs) -> Option<Arc<v1::access::AccessVerifier>> {
    if args.auth != v1::io::AuthMode::CloudflareAccess {
        return None;
    }
    match v1::access::AccessVerifier::new(&args.access) {
        Ok(v) => Some(Arc::new(v)),
        Err(e) => {
            tracing::error!(error = %e, "Cloudflare Access is not configured; every request is refused");
            None
        }
    }
}

async fn require_origin(
    access: Option<Arc<v1::access::AccessVerifier>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let checked = match &access {
        None => Err(v1::access::AccessError::Unavailable(
            "access verification is not configured".into(),
        )),
        Some(v) => {
            let jwt = req
                .headers()
                .get(v1::access::ORIGIN_ASSERTION)
                .and_then(|h| h.to_str().ok());
            v.check_origin(jwt).await
        }
    };
    match checked {
        Ok(()) => next.run(req).await,
        Err(e) => v1::io::ApiError::from(e).into_response(),
    }
}

/// One `/api/v1` instance over the state in `dir` (registry, or the C0 fixture in dev mode), with
/// its tool host, its MCP endpoint (`/<name>/mcp` on the loopback listener, with its own grants, so
/// a fixture token never reaches the registry) and its agent runtime.
#[allow(clippy::too_many_arguments)]
fn v1_instance(
    args: &ServeArgs,
    codex: &v1::agent::Preflighted,
    dir: &std::path::Path,
    services: v1::ResearchServices,
    origin: ignis_core::contracts::Origin,
    fixture: Option<v1::fixture::Fixture>,
    name: &str,
    mcp: Option<&mut McpListener>,
    runner_report: v1::sandbox::RunnerReport,
    access: Option<Arc<v1::access::AccessVerifier>>,
) -> Arc<v1::V1> {
    let (store, inner) = open_state(args, dir, name);
    let (artifacts, seed): (Arc<dyn ignis_core::services::ArtifactStore>, _) = match fixture {
        Some(fx) => (Arc::new(fx.artifacts(inner)), Some(Arc::new(fx.seed))),
        None => (inner, None),
    };
    let runner_wake = Arc::new(tokio::sync::Notify::new());
    let tools = {
        let wake = runner_wake.clone();
        Arc::new(ignis_mcp::ToolHost::new(
            services.clone(),
            store.clone(),
            artifacts.clone(),
            origin,
            Arc::new(move || wake.notify_one()),
        ))
    };
    let attachment = mcp.map(|m| m.mount(name, tools.clone()));
    let mcp_url = attachment.as_ref().map(|a| a.url.clone());
    let hub: Arc<v1::Hub> = Default::default();
    let (runtime, runtime_report) = v1::agent::runtime(
        codex,
        dir.join("codex"),
        store.clone(),
        artifacts.clone(),
        hub.clone(),
        attachment,
    );
    Arc::new(v1::V1 {
        store,
        artifacts,
        services,
        tools,
        hub,
        auth: args.auth,
        access,
        runner_wake,
        origin,
        uploads_dir: dir.join("uploads"),
        max_upload_bytes: args.max_upload_bytes,
        fixture: seed,
        runtime,
        runtime_report,
        runner_report,
        mcp_url,
        artifact_backend: args.artifact_backend,
    })
}

/// The loopback-only listener of the IGNIS MCP endpoints. Bound only when the agent runtime passed
/// preflight (the endpoint has no other consumer) and a Tokio runtime is there to serve it.
struct McpListener {
    listener: std::net::TcpListener,
    port: u16,
    routes: Router,
}

impl McpListener {
    fn bind(args: &ServeArgs, codex: &v1::agent::Preflighted) -> Option<Self> {
        if !codex.ready() || tokio::runtime::Handle::try_current().is_err() {
            return None;
        }
        // Always 127.0.0.1: the endpoint is for the runtime child process, never for the network.
        let listener = std::net::TcpListener::bind(("127.0.0.1", args.mcp_port))
            .and_then(|l| l.set_nonblocking(true).map(|()| l))
            .map_err(|e| tracing::error!(error = %e, port = args.mcp_port, "MCP listener not bound; the runtime has no IGNIS tools"))
            .ok()?;
        let port = listener.local_addr().ok()?.port();
        Some(McpListener {
            listener,
            port,
            routes: Router::new(),
        })
    }

    fn mount(
        &mut self,
        name: &str,
        tools: Arc<ignis_mcp::ToolHost>,
    ) -> ignis_research::runtime::McpAttachment {
        let grants = ignis_research::runtime::McpGrants::new();
        let path = format!("/{name}/mcp");
        let registered = tools.registered();
        let enabled_tools = registered.iter().map(|d| d.name.clone()).collect();
        let approved_tools = registered
            .iter()
            .filter(|d| {
                matches!(
                    d.effect,
                    ignis_core::contracts::ToolEffect::Read
                        | ignis_core::contracts::ToolEffect::Compute
                )
            })
            .map(|d| d.name.clone())
            .collect();
        let routes = std::mem::take(&mut self.routes);
        self.routes = routes.nest_service(&path, ignis_mcp::router(tools, grants.clone()));
        ignis_research::runtime::McpAttachment {
            url: format!("http://127.0.0.1:{}{path}", self.port),
            grants,
            enabled_tools,
            approved_tools,
        }
    }

    fn serve(self) {
        let McpListener {
            listener,
            port,
            routes,
        } = self;
        let Ok(listener) = tokio::net::TcpListener::from_std(listener) else {
            tracing::error!("MCP listener could not join the runtime");
            return;
        };
        tracing::info!(port, "IGNIS MCP listening on loopback");
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, routes.layer(TraceLayer::new_for_http())).await {
                tracing::error!(error = %e, "IGNIS MCP listener stopped");
            }
        });
    }
}

/// Run blocking startup work (child processes) without stalling a Tokio worker: on a
/// multi-threaded runtime the worker hands its other tasks off first.
fn blocking_startup<T>(f: impl FnOnce() -> T) -> T {
    match tokio::runtime::Handle::try_current() {
        Ok(h) if h.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(f)
        }
        _ => f(),
    }
}

fn open_state(
    args: &ServeArgs,
    dir: &std::path::Path,
    name: &str,
) -> (
    Arc<ignis_research::ResearchStore>,
    Arc<dyn ignis_core::services::ArtifactStore>,
) {
    let store = ignis_research::ResearchStore::open(dir.join("state.sqlite"))
        .unwrap_or_else(|e| panic!("state store in {}: {e}", dir.display()));
    let artifacts =
        ops::artifact_store(args.artifact_backend, &args.r2, dir.join("artifacts"), name)
            .unwrap_or_else(|e| panic!("artifact store for {}: {e}", dir.display()));
    (Arc::new(store), artifacts)
}

/// The short-command limits of the legacy routes (30 s, 1 MiB).
fn legacy_limits(r: Router) -> Router {
    r.layer(TimeoutLayer::with_status_code(
        axum::http::StatusCode::REQUEST_TIMEOUT,
        Duration::from_secs(30),
    ))
    .layer(RequestBodyLimitLayer::new(1 << 20))
}

/// `ignis serve`: run the API until Ctrl-C. Builds its own Tokio runtime.
pub fn run(args: ServeArgs, services: Services) -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .try_init();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    if args.auth == v1::io::AuthMode::CloudflareAccess {
        v1::access::AccessVerifier::new(&args.access).map_err(|e| anyhow::anyhow!(e))?;
    }
    if args.artifact_backend == ops::ArtifactBackend::R2 {
        // Fail at start (the supervisor retries) rather than accept work it cannot store.
        ops::probe_r2(&args.r2).map_err(|e| anyhow::anyhow!(e))?;
    }
    rt.block_on(async move {
        let addr = format!("{}:{}", args.host, args.port);
        let app = app(args, services);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| anyhow::anyhow!("bind {addr}: {e}"))?;
        tracing::info!("ignis-server listening on http://{addr}/api");
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    })
}

/// Ctrl-C, or SIGTERM from the supervisor (systemd, `docker stop`).
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut term) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = term.recv() => {}
                }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
    tracing::info!("shutting down");
}

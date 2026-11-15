//! In-process job runner (single host). Claims queued jobs from the state store with a lease,
//! heartbeats while the blocking work runs on the blocking pool, and finishes with its fencing token.
//! Outputs go to the artifact store before the job is marked succeeded, so a succeeded job always
//! references complete bytes. On startup every lease of the previous process is settled
//! (`reap_expired(all)`): idempotent jobs re-queue, effectful ones go to `reconciling`.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ignis_core::contracts::{
    AnalysisExecuteRequest, ArtifactRef, EntityRef, ErrorCode, ErrorEnvelope, ForecastRequest,
    Origin, Producer, RetryPolicy,
};
use ignis_core::services::ArtifactDraft;
use ignis_research::store::{ClaimedJob, JobOutcome, JobPriority};
use tokio::sync::Semaphore;

use super::V1;

pub const FORECAST: &str = ignis_mcp::FORECAST_JOB;
pub const ANALYSIS: &str = ignis_mcp::ANALYSIS_JOB;

const LEASE: Duration = Duration::from_secs(30);
const HEARTBEAT: Duration = Duration::from_secs(10);
const REAP_EVERY: Duration = Duration::from_secs(15);
/// Concurrent jobs (forecasts are CPU-bound; the services use their own pools).
const SLOTS: usize = 3;

/// Scheduling class of a job kind: interactive work (a forecast a researcher waits for) is claimed
/// first; requested evaluations and background ingestion are also capped so they never take every
/// slot (master design §17: interactive > evaluations > ingestion).
pub fn priority_of(kind: &str) -> JobPriority {
    match kind {
        FORECAST | ANALYSIS => JobPriority::Interactive,
        k if k.starts_with("evaluation_") => JobPriority::Evaluation,
        _ => JobPriority::Ingestion,
    }
}

fn cap(p: JobPriority) -> usize {
    match p {
        JobPriority::Interactive => SLOTS,
        JobPriority::Evaluation | JobPriority::Ingestion => 1,
    }
}

pub fn spawn(v: Arc<V1>) {
    match tokio::runtime::Handle::try_current() {
        Ok(h) => {
            h.spawn(run(v));
        }
        Err(_) => tracing::warn!("no Tokio runtime: the /api/v1 job runner is not started"),
    }
}

fn kinds(v: &V1) -> Vec<&'static str> {
    let mut k = Vec::new();
    if v.services.models.is_some() {
        k.push(FORECAST);
    }
    if v.services.sandbox.is_some() {
        k.push(ANALYSIS);
    }
    k
}

async fn run(v: Arc<V1>) {
    let owner = format!("runner-{}-{}", std::process::id(), super::uuid_like());
    let store = v.store.clone();
    match tokio::task::spawn_blocking(move || store.reap_expired(true)).await {
        Ok(Ok(settled)) if !settled.is_empty() => {
            tracing::warn!(
                jobs = settled.len(),
                "settled job leases of a previous process"
            )
        }
        Ok(Err(e)) => tracing::error!(error = %e, "reaping job leases at startup"),
        _ => {}
    }
    let kinds = kinds(&v);
    if kinds.is_empty() {
        tracing::info!("job runner idle: no job kinds have a configured service");
        return;
    }
    let slots = Arc::new(Semaphore::new(SLOTS));
    let running: Arc<[AtomicUsize; 3]> = Arc::new(Default::default());
    let mut last_reap = tokio::time::Instant::now();
    loop {
        if last_reap.elapsed() >= REAP_EVERY {
            let store = v.store.clone();
            let _ = tokio::task::spawn_blocking(move || store.reap_expired(false)).await;
            last_reap = tokio::time::Instant::now();
        }
        let Ok(permit) = slots.clone().acquire_owned().await else {
            return;
        };
        // Kinds whose class still has room; the store claims the highest priority among them.
        let allowed: Vec<&'static str> = kinds
            .iter()
            .copied()
            .filter(|k| {
                let p = priority_of(k);
                running[p as usize].load(Ordering::SeqCst) < cap(p)
            })
            .collect();
        let (store, owner2, kinds2) = (v.store.clone(), owner.clone(), allowed);
        let claimed =
            tokio::task::spawn_blocking(move || store.claim_job(&owner2, &kinds2, LEASE)).await;
        match claimed {
            Ok(Ok(Some(job))) => {
                let v2 = v.clone();
                let class = priority_of(&job.kind) as usize;
                running[class].fetch_add(1, Ordering::SeqCst);
                let running = running.clone();
                tokio::spawn(async move {
                    execute(v2, job).await;
                    running[class].fetch_sub(1, Ordering::SeqCst);
                    drop(permit);
                });
            }
            Ok(Ok(None)) => {
                drop(permit);
                let _ =
                    tokio::time::timeout(Duration::from_secs(1), v.runner_wake.notified()).await;
            }
            Ok(Err(e)) => {
                drop(permit);
                tracing::error!(error = %e, "claiming a job");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(_) => return,
        }
    }
}

async fn wake_episode(v: &Arc<V1>, job: &ClaimedJob) {
    if let Some(ep) = &job.episode_id {
        super::notify_episode(v, &job.scope, ep).await;
    }
}

async fn execute(v: Arc<V1>, job: ClaimedJob) {
    wake_episode(&v, &job).await;
    let cancel = Arc::new(AtomicBool::new(false));
    let lost = Arc::new(AtomicBool::new(false));
    let hb = {
        let (store, id, fence) = (v.store.clone(), job.job_id.clone(), job.fence);
        let (cancel, lost) = (cancel.clone(), lost.clone());
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(HEARTBEAT);
            tick.tick().await;
            loop {
                tick.tick().await;
                let (s, i) = (store.clone(), id.clone());
                match tokio::task::spawn_blocking(move || s.heartbeat(&i, fence, LEASE, None)).await
                {
                    Ok(Ok(c)) => cancel.store(c, Ordering::SeqCst),
                    _ => {
                        lost.store(true, Ordering::SeqCst);
                        return;
                    }
                }
            }
        })
    };
    let (v2, job2, cancel2) = (v.clone(), job.clone(), cancel.clone());
    let result = tokio::task::spawn_blocking(move || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handle(&v2, &job2, &cancel2)
        }))
    })
    .await;
    hb.abort();
    if lost.load(Ordering::SeqCst) {
        tracing::warn!(job = %job.job_id, "lease lost; the outcome of this attempt is discarded");
        return;
    }
    let outcome = if cancel.load(Ordering::SeqCst) {
        JobOutcome::Cancelled
    } else {
        match result {
            Ok(Ok(Ok(outputs))) => JobOutcome::Succeeded { outputs },
            Ok(Ok(Err(error))) => JobOutcome::Failed { error },
            Ok(Err(panic)) => JobOutcome::Failed {
                error: ErrorEnvelope::new(
                    ErrorCode::Internal,
                    format!(
                        "job panicked: {}",
                        panic
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_default()
                    ),
                    RetryPolicy::Never,
                ),
            },
            Err(e) => JobOutcome::Failed {
                error: ErrorEnvelope::new(ErrorCode::Internal, e.to_string(), RetryPolicy::Never),
            },
        }
    };
    let (store, id, fence) = (v.store.clone(), job.job_id.clone(), job.fence);
    match tokio::task::spawn_blocking(move || store.finish_job(&id, fence, outcome)).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => tracing::warn!(job = %job.job_id, error = %e, "finishing a job"),
        Err(e) => tracing::error!(job = %job.job_id, error = %e, "finishing a job"),
    }
    wake_episode(&v, &job).await;
}

fn handle(
    v: &V1,
    job: &ClaimedJob,
    cancel: &AtomicBool,
) -> Result<Vec<ArtifactRef>, ErrorEnvelope> {
    match job.kind.as_str() {
        ANALYSIS => {
            let sandbox = v.services.sandbox.clone().ok_or_else(|| {
                ErrorEnvelope::new(
                    ErrorCode::CapabilityUnavailable,
                    "isolated runner not configured",
                    RetryPolicy::Never,
                )
            })?;
            let req: AnalysisExecuteRequest =
                serde_json::from_value(job.input.clone()).map_err(|e| {
                    ErrorEnvelope::new(ErrorCode::InvalidRequest, e.to_string(), RetryPolicy::Never)
                })?;
            let producer = Producer {
                episode_id: job.episode_id.clone(),
                job_id: Some(job.job_id.clone()),
                tool_call_id: None,
                release_id: None,
            };
            sandbox.execute(
                &job.scope,
                &req,
                v.artifacts.as_ref(),
                v.services.datasets.as_deref(),
                producer,
                cancel,
            )
        }
        FORECAST => {
            let models = v.services.models.clone().ok_or_else(|| {
                ErrorEnvelope::new(
                    ErrorCode::CapabilityUnavailable,
                    "ModelService not configured",
                    RetryPolicy::Never,
                )
            })?;
            let req: ForecastRequest = serde_json::from_value(job.input.clone()).map_err(|e| {
                ErrorEnvelope::new(ErrorCode::InvalidRequest, e.to_string(), RetryPolicy::Never)
            })?;
            let artifact = models.forecast(&job.scope, &req)?;
            let bytes = serde_json::to_vec_pretty(&artifact).map_err(|e| {
                ErrorEnvelope::new(ErrorCode::Internal, e.to_string(), RetryPolicy::Never)
            })?;
            let draft = ArtifactDraft {
                media_type: "application/json".into(),
                contract: Some("AnalysisArtifact".into()),
                produced_by: Producer {
                    episode_id: job.episode_id.clone(),
                    job_id: Some(job.job_id.clone()),
                    tool_call_id: None,
                    release_id: Some(req.release_id.clone()),
                },
                inputs: vec![EntityRef::Release(req.release_id.clone())],
                origin: Origin::System,
            };
            Ok(vec![v.artifacts.put(
                &job.scope,
                draft,
                &mut bytes.as_slice(),
            )?])
        }
        other => Err(ErrorEnvelope::new(
            ErrorCode::Unsupported,
            format!("no handler for job kind {other}"),
            RetryPolicy::Never,
        )),
    }
}

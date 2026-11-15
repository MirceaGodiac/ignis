//! Opt-in: `analysis_execute` end to end through the tool host, the job store and a REAL runner
//! (container engine + pinned image). Ignored by default.
//!
//! ```bash
//! IGNIS_RUNNER_IMAGE=sha256:… IGNIS_RUNNER_RUNTIME=runc \
//!   cargo test -p ignis-mcp --test analysis_live -- --ignored
//! ```

mod common;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use common::*;
use ignis_core::contracts::*;
use ignis_research::sandbox::{Sandbox, SandboxConfig};
use ignis_research::store::JobOutcome;
use serde_json::json;

fn sandbox(dir: &std::path::Path) -> Option<Arc<Sandbox>> {
    let image = std::env::var("IGNIS_RUNNER_IMAGE").ok()?;
    let runtime = std::env::var("IGNIS_RUNNER_RUNTIME").unwrap_or_else(|_| "runsc".into());
    let dir = std::fs::canonicalize(dir).unwrap();
    let dir = match dir.to_str().and_then(|s| s.strip_prefix(r"\?\")) {
        Some(s) => s.into(),
        None => dir,
    };
    Some(Arc::new(
        Sandbox::preflight(SandboxConfig {
            engine: "docker".into(),
            image: Digest::parse(&image).unwrap(),
            require_gvisor: runtime.starts_with("runsc"),
            oci_runtime: runtime,
            work_dir: dir.join("runner"),
            limits: RunnerLimits {
                wall_seconds: 120,
                memory_bytes: 512 << 20,
                cpu_millis: 1000,
                pids: 128,
                tmp_bytes: 64 << 20,
                output_bytes: 1 << 20,
                stream_bytes: 1 << 16,
            },
        })
        .unwrap_or_else(|e| panic!("preflight: {e}")),
    ))
}

#[test]
#[ignore = "needs a container engine and IGNIS_RUNNER_IMAGE"]
fn analysis_execute_runs_in_the_runner_as_a_job_of_the_callers_workspace() {
    let runner_dir = tempfile::tempdir().unwrap();
    let Some(sb) = sandbox(runner_dir.path()) else {
        eprintln!("IGNIS_RUNNER_IMAGE not set; skipped");
        return;
    };
    let h = host_with(Some(sb.clone()));
    assert!(h
        .host
        .registered()
        .iter()
        .any(|d| d.name == "analysis_execute"));
    let args = json!({
        "language": "python",
        "source": "import json\njson.dump({'sum': sum(range(10))}, open('/out/r.json', 'w'))",
        "client_request_id": "crid-analysis-1",
    });

    // The research fixture scope has no ExecuteCode: refused before anything is queued.
    let plain = scope("ws_a");
    let r = h.host.call(&plain, "analysis_execute", args.clone());
    assert_eq!(r.execution_status, ExecutionStatus::Refused);
    assert_eq!(r.error.unwrap().code, ErrorCode::ScopeDenied);

    let mut sc = scope("ws_a");
    sc.capabilities.insert(Capability::ExecuteCode);
    let r = h.host.call(&sc, "analysis_execute", args.clone());
    assert_eq!(r.execution_status, ExecutionStatus::Accepted, "{r:?}");
    let job = r.job.unwrap();
    assert_eq!(job.kind, "analysis_execute");
    // Same idempotency key: same job.
    let again = h.host.call(&sc, "analysis_execute", args);
    assert_eq!(again.job.unwrap().job_id, job.job_id);

    // What the server's job runner does with the claimed job.
    let claimed = h
        .store
        .claim_job("test", &[ignis_mcp::ANALYSIS_JOB], Duration::from_secs(60))
        .unwrap()
        .unwrap();
    assert_eq!(claimed.scope, sc, "the job runs with the scope of the call");
    let req: AnalysisExecuteRequest = serde_json::from_value(claimed.input.clone()).unwrap();
    let outputs = sb
        .execute(
            &claimed.scope,
            &req,
            h.artifacts.as_ref(),
            None,
            Producer {
                job_id: Some(claimed.job_id.clone()),
                ..Default::default()
            },
            &AtomicBool::new(false),
        )
        .unwrap();
    h.store
        .finish_job(
            &claimed.job_id,
            claimed.fence,
            JobOutcome::Succeeded { outputs },
        )
        .unwrap();

    let r = h
        .host
        .call(&sc, "jobs_get", json!({"job_id": job.job_id.as_str()}));
    let done: JobRef = serde_json::from_value(r.result.unwrap().value).unwrap();
    assert_eq!(done.status, JobStatus::Succeeded);
    let record = &done.outputs[0];
    assert_eq!(record.contract.as_deref(), Some("ExecutionRecord"));
    let read = |pointer: &str| {
        h.host
            .call(
                &sc,
                "artifacts_read",
                json!({"artifact_id": record.artifact_id.as_str(), "json_pointer": pointer}),
            )
            .result
            .unwrap()
            .value
    };
    assert_eq!(read("/outcome"), json!({"kind": "exited", "exit_code": 0}));
    assert_eq!(read("/outputs/0/path"), json!("r.json"));
    // Another workspace cannot see the job or its record.
    let other = {
        let mut s = scope("ws_b");
        s.capabilities.insert(Capability::ExecuteCode);
        s
    };
    let r = h
        .host
        .call(&other, "jobs_get", json!({"job_id": job.job_id.as_str()}));
    assert_eq!(r.execution_status, ExecutionStatus::Refused);
}

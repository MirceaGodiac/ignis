//! Opt-in checks of the analysis runner against a REAL container engine and the pinned runner image.
//! Ignored by default; they start containers (no network, no host mounts besides a temp directory).
//!
//! ```bash
//! eval "$(tools/runner/build.sh)"            # prints IGNIS_RUNNER_IMAGE=sha256:…
//! IGNIS_RUNNER_IMAGE=sha256:… IGNIS_RUNNER_RUNTIME=runc \
//!   cargo test -p ignis-research --test sandbox_live -- --ignored --test-threads 1
//! ```
//! `IGNIS_RUNNER_RUNTIME` defaults to `runsc` (gVisor); `runc` checks the same isolation flags under
//! the host kernel (development machines without gVisor).

use std::collections::BTreeSet;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ignis_core::contracts::*;
use ignis_core::services::{ArtifactDraft, ArtifactStore};
use ignis_research::sandbox::{Sandbox, SandboxConfig};
use ignis_research::LocalArtifactStore;
use serde_json::Value;

fn scope(ws: &str) -> ExecutionScope {
    ExecutionScope {
        scope_id: ScopeId::new(format!("scope-{ws}")).unwrap(),
        workspace_id: WorkspaceId::new(ws).unwrap(),
        principal: PrincipalId::new(format!("user-{ws}")).unwrap(),
        role: ScopeRole::Research,
        partitions: BTreeSet::from([Partition::Train]),
        capabilities: BTreeSet::from([Capability::ExecuteCode]),
        corpus_snapshot: None,
        parent_scope_id: None,
    }
}

fn limits() -> RunnerLimits {
    RunnerLimits {
        wall_seconds: 120,
        memory_bytes: 512 << 20,
        cpu_millis: 1000,
        pids: 128,
        tmp_bytes: 64 << 20,
        output_bytes: 1 << 20,
        stream_bytes: 1 << 16,
    }
}

struct Live {
    _dir: tempfile::TempDir,
    sandbox: Sandbox,
    store: LocalArtifactStore,
}

fn live(limits: RunnerLimits) -> Option<Live> {
    let image = std::env::var("IGNIS_RUNNER_IMAGE").ok()?;
    let runtime = std::env::var("IGNIS_RUNNER_RUNTIME").unwrap_or_else(|_| "runsc".into());
    let dir = tempfile::tempdir().unwrap();
    let work_dir = std::fs::canonicalize(dir.path()).unwrap();
    let work_dir = match work_dir.to_str().and_then(|s| s.strip_prefix(r"\\?\")) {
        Some(s) => s.into(),
        None => work_dir,
    };
    let sandbox = Sandbox::preflight(SandboxConfig {
        engine: std::env::var("IGNIS_RUNNER_DOCKER")
            .unwrap_or_else(|_| "docker".into())
            .into(),
        image: Digest::parse(&image).unwrap(),
        require_gvisor: runtime.starts_with("runsc"),
        oci_runtime: runtime,
        work_dir: work_dir.join("runner"),
        limits,
    })
    .unwrap_or_else(|e| panic!("preflight: {e}"));
    let store = LocalArtifactStore::open(work_dir.join("artifacts")).unwrap();
    Some(Live {
        _dir: dir,
        sandbox,
        store,
    })
}

impl Live {
    fn put(&self, sc: &ExecutionScope, bytes: &[u8]) -> ArtifactId {
        self.store
            .put(
                sc,
                ArtifactDraft {
                    media_type: "text/csv".into(),
                    contract: None,
                    produced_by: Producer::default(),
                    inputs: vec![],
                    origin: Origin::System,
                },
                &mut &bytes[..],
            )
            .unwrap()
            .artifact_id
    }

    fn exec(
        &self,
        sc: &ExecutionScope,
        source: &str,
        inputs: Vec<AnalysisInput>,
        cancel: &AtomicBool,
    ) -> Result<(ExecutionRecord, Vec<ArtifactRef>), ErrorEnvelope> {
        let req = AnalysisExecuteRequest {
            language: AnalysisLanguage::Python,
            source: source.into(),
            inputs,
            client_request_id: None,
        };
        let out = self
            .sandbox
            .execute(sc, &req, &self.store, None, Producer::default(), cancel)?;
        let rec: ExecutionRecord = serde_json::from_slice(&self.bytes(sc, &out[0])).unwrap();
        rec.validate().unwrap();
        Ok((rec, out))
    }

    fn bytes(&self, sc: &ExecutionScope, a: &ArtifactRef) -> Vec<u8> {
        let mut v = Vec::new();
        self.store
            .open(sc, &a.artifact_id, 0, None)
            .unwrap()
            .read_to_end(&mut v)
            .unwrap();
        v
    }

    fn stdout_json(&self, sc: &ExecutionScope, r: &ExecutionRecord) -> Value {
        let a = r.stdout.artifact.as_ref().unwrap_or_else(|| {
            let err = r
                .stderr
                .artifact
                .as_ref()
                .map(|a| String::from_utf8_lossy(&self.bytes(sc, a)).to_string());
            panic!("no stdout; outcome {:?}; stderr {err:?}", r.outcome)
        });
        serde_json::from_slice(&self.bytes(sc, a)).unwrap()
    }
}

macro_rules! live_or_skip {
    ($limits:expr) => {
        match live($limits) {
            Some(l) => l,
            None => {
                eprintln!("IGNIS_RUNNER_IMAGE not set; skipped");
                return;
            }
        }
    };
}

#[test]
#[ignore = "needs a container engine and IGNIS_RUNNER_IMAGE"]
fn a_scientific_program_reads_its_input_and_its_outputs_become_artifacts() {
    let l = live_or_skip!(limits());
    let sc = scope("ws-a");
    eprintln!("{:?}", l.sandbox.environment());
    let csv = l.put(&sc, b"t,x\n0,0\n1,2\n2,4\n3,6\n");
    let src = r#"
import json, numpy as np, pandas as pd, scipy.stats as st
d = pd.read_csv("/in/series.csv")
fit = st.linregress(d["t"], d["x"])
json.dump({"slope": fit.slope, "n": int(len(d))}, open("/out/rate.json", "w"))
import os; os.makedirs("/out/plots", exist_ok=True); open("/out/plots/a.txt", "w").write("ok")
print(json.dumps({"numpy": np.__version__}))
"#;
    let never = AtomicBool::new(false);
    let (rec, out) = l
        .exec(
            &sc,
            src,
            vec![AnalysisInput {
                name: "series.csv".into(),
                artifact_id: Some(csv.clone()),
                dataset_view: None,
                conditions: None,
            }],
            &never,
        )
        .unwrap();
    assert_eq!(rec.outcome, ExecutionOutcome::Exited { exit_code: 0 });
    assert_eq!(rec.inputs[0].artifact.artifact_id, csv);
    let paths: Vec<&str> = rec.outputs.iter().map(|o| o.path.as_str()).collect();
    assert_eq!(paths, ["plots/a.txt", "rate.json"]);
    let rate: Value = serde_json::from_slice(&l.bytes(&sc, &rec.outputs[1].artifact)).unwrap();
    assert_eq!(rate["slope"], 2.0);
    assert_eq!(rate["n"], 4);
    // Lineage: every output names the input and the program it came from.
    let o = &rec.outputs[1].artifact;
    assert!(o.inputs.contains(&EntityRef::Artifact(csv)));
    assert!(o
        .inputs
        .contains(&EntityRef::Artifact(rec.program.artifact_id.clone())));
    assert_eq!(out[0].contract.as_deref(), Some("ExecutionRecord"));
    assert_eq!(l.bytes(&sc, &rec.program), src.as_bytes());
    assert!(l.stdout_json(&sc, &rec)["numpy"].is_string());
}

#[test]
#[ignore = "needs a container engine and IGNIS_RUNNER_IMAGE"]
fn a_hostile_program_cannot_reach_the_host_the_network_or_its_secrets() {
    // A secret in the host environment (as the MCP token is for Codex) must not reach the program.
    std::env::set_var("IGNIS_TEST_HOST_SECRET", "do-not-leak");
    let l = live_or_skip!(limits());
    let sc = scope("ws-h");
    let input = l.put(&sc, b"original");
    let src = r#"
import json, os, socket, subprocess
r = {}
def attempt(name, f):
    try:
        f(); r[name] = "allowed"
    except Exception as e:
        r[name] = type(e).__name__
attempt("connect", lambda: socket.create_connection(("1.1.1.1", 443), timeout=3).close())
attempt("dns", lambda: socket.getaddrinfo("pypi.org", 443))
attempt("write_root", lambda: open("/usr/lib/evil", "w").write("x"))
attempt("write_input", lambda: open("/in/data.txt", "w").write("x"))
attempt("docker_sock", lambda: socket.socket(socket.AF_UNIX).connect("/var/run/docker.sock"))
attempt("setuid", lambda: os.setuid(0))
attempt("mount", lambda: subprocess.run(["mount", "-t", "tmpfs", "x", "/mnt"], check=True, capture_output=True))
r["secret_in_env"] = any("do-not-leak" in v for v in os.environ.values()) or "IGNIS_TEST_HOST_SECRET" in os.environ
r["uid"] = os.getuid()
r["input"] = open("/in/data.txt").read()
os.symlink("/etc/passwd", "/out/passwd_link")
os.symlink("/in/data.txt", "/out/input_link")
open("/out/ok.txt", "w").write("fine")
print(json.dumps(r))
"#;
    let never = AtomicBool::new(false);
    let (rec, _) = l
        .exec(
            &sc,
            src,
            vec![AnalysisInput {
                name: "data.txt".into(),
                artifact_id: Some(input.clone()),
                dataset_view: None,
                conditions: None,
            }],
            &never,
        )
        .unwrap();
    std::env::remove_var("IGNIS_TEST_HOST_SECRET");
    let r = l.stdout_json(&sc, &rec);
    eprintln!("{r}");
    for k in [
        "connect",
        "dns",
        "write_root",
        "write_input",
        "docker_sock",
        "setuid",
        "mount",
    ] {
        assert_ne!(
            r[k], "allowed",
            "{k} must be refused inside the runner: {r}"
        );
    }
    assert_eq!(r["secret_in_env"], false);
    assert_eq!(r["uid"], 65534);
    assert_eq!(r["input"], "original");
    // Links are reported, never followed into host or input bytes.
    let kept: Vec<&str> = rec.outputs.iter().map(|o| o.path.as_str()).collect();
    assert_eq!(kept, ["ok.txt"]);
    let rejected: Vec<&str> = rec
        .rejected_outputs
        .iter()
        .map(|o| o.path.as_str())
        .collect();
    assert_eq!(rejected, ["input_link", "passwd_link"]);
    // The input artifact is unchanged.
    assert_eq!(
        l.bytes(&sc, &l.store.stat(&sc, &input).unwrap()),
        b"original"
    );
}

#[test]
#[ignore = "needs a container engine and IGNIS_RUNNER_IMAGE"]
fn budgets_end_runaway_programs_and_say_how() {
    let l = live_or_skip!(RunnerLimits {
        wall_seconds: 3,
        memory_bytes: 96 << 20,
        stream_bytes: 1000,
        ..limits()
    });
    let sc = scope("ws-b");
    let never = AtomicBool::new(false);

    let (rec, _) = l.exec(&sc, "while True: pass", vec![], &never).unwrap();
    assert_eq!(rec.outcome, ExecutionOutcome::TimedOut);

    let (rec, _) = l
        .exec(
            &sc,
            "x = bytearray(512 * 1024 * 1024)\nprint(len(x))",
            vec![],
            &never,
        )
        .unwrap();
    assert_eq!(rec.outcome, ExecutionOutcome::OutOfMemory);

    let (rec, _) = l
        .exec(
            &sc,
            "print('y' * 5000)\nraise SystemExit(3)",
            vec![],
            &never,
        )
        .unwrap();
    assert_eq!(rec.outcome, ExecutionOutcome::Exited { exit_code: 3 });
    assert!(rec.stdout.truncated);
    assert_eq!(rec.stdout.total_bytes, 5001);
    assert_eq!(rec.stdout.artifact.as_ref().unwrap().byte_length, 1000);

    // Fork bomb: stopped by the pids budget, or by the memory budget first (gVisor accounts the
    // children's memory to the sandbox and reaches 96 MiB before the pids limit); the host is
    // untouched either way.
    let (rec, _) = l
        .exec(
            &sc,
            "import os\nn=0\ntry:\n    while True:\n        if os.fork() == 0:\n            import time; time.sleep(30)\n        n += 1\nexcept OSError:\n    print(n)",
            vec![],
            &never,
        )
        .unwrap();
    assert!(
        matches!(
            rec.outcome,
            ExecutionOutcome::Exited { .. }
                | ExecutionOutcome::TimedOut
                | ExecutionOutcome::OutOfMemory
        ),
        "{:?}",
        rec.outcome
    );
}

#[test]
#[ignore = "needs a container engine and IGNIS_RUNNER_IMAGE"]
fn cancel_stops_the_container_and_another_workspace_cannot_mount_my_artifacts() {
    let l = live_or_skip!(limits());
    let mine = scope("ws-c");
    let cancel = Arc::new(AtomicBool::new(false));
    let c2 = cancel.clone();
    let t = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2));
        c2.store(true, Ordering::SeqCst);
    });
    let started = std::time::Instant::now();
    let e = l
        .exec(&mine, "import time\ntime.sleep(300)", vec![], &cancel)
        .unwrap_err();
    t.join().unwrap();
    assert_eq!(e.code, ErrorCode::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(60));
    let left = std::process::Command::new("docker")
        .args(["ps", "-aq", "--filter", "label=org.ignis.runner"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&left.stdout).trim().is_empty(),
        "no container is left behind"
    );

    let secret = l.put(&mine, b"workspace c only");
    let other = scope("ws-d");
    let never = AtomicBool::new(false);
    let e = l
        .exec(
            &other,
            "print(open('/in/x').read())",
            vec![AnalysisInput {
                name: "x".into(),
                artifact_id: Some(secret),
                dataset_view: None,
                conditions: None,
            }],
            &never,
        )
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::NotFound);
}

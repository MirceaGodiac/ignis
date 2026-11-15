//! M9, offline: `analysis_execute` with inputs named by reference (workspace artifact, dataset
//! view, complete conditions table) through the tool host, the job store and the real `Sandbox`
//! code path, over the fake container engine (`ignis-mcp-fake-engine`, which reports what it found
//! under `/in` instead of running Python). Proves that the host resolves and mounts the data, that
//! the record names exactly the bytes the program saw, and that a test-partition reference — direct
//! or through an artifact's lineage — is refused outside an evaluation scope.

mod common;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use common::*;
use ignis_core::contracts::*;
use ignis_core::hashing::sha256_hex;
use ignis_core::services::{ArtifactDraft, ArtifactStore, DatasetService, ServiceResult};
use ignis_research::sandbox::{Sandbox, SandboxConfig};
use serde_json::{json, Value};

const BUILDER: &str = "fixture-dataset-flex";
const HOLDOUT: &str = "fixture-dataset-flex-test";
const VIEW: &str = "observations";
const TEST_RUN: &str = "run-holdout-01";
const TRAIN_RUN: &str = "run-train-01";

fn view_bytes(id: &str) -> Vec<u8> {
    format!("PAR1 fake parquet of {id} PAR1").into_bytes()
}

fn snapshot() -> Digest {
    Digest::of_bytes(b"fixture conditions snapshot")
}

/// Two bundles: the builder fixture (train + validation) and a test-partition bundle. The fake is
/// deliberately lax: it hands either bundle to any scope, so the refusal the tests observe comes
/// from the host's own partition check, not from the service. Conditions come in two pages.
struct RefDatasets;

impl RefDatasets {
    fn bundle(id: &str) -> DatasetBundle {
        let mut b: DatasetBundle = fixture("data/dataset_bundle.json");
        b.dataset_id = DatasetId::new(id).unwrap();
        if id == HOLDOUT {
            b.partitions = [Partition::Test].into();
        }
        let bytes = view_bytes(id);
        let v = b.views.iter_mut().find(|v| v.name == VIEW).unwrap();
        v.artifact.artifact_id = ArtifactId::new(format!("view-{id}")).unwrap();
        v.artifact.sha256 = Digest::parse(&format!("sha256:{}", sha256_hex(&bytes))).unwrap();
        v.artifact.byte_length = bytes.len() as u64;
        b
    }
}

impl DatasetService for RefDatasets {
    fn dataset(&self, _: &ExecutionScope, id: &DatasetId) -> ServiceResult<DatasetBundle> {
        Ok(Self::bundle(id.as_str()))
    }
    fn open_view(
        &self,
        _: &ExecutionScope,
        id: &DatasetId,
        view: &str,
        _offset: u64,
        _len: Option<u64>,
    ) -> ServiceResult<(ArtifactRef, Box<dyn std::io::Read + Send>)> {
        let b = Self::bundle(id.as_str());
        let v = b.views.into_iter().find(|v| v.name == view).unwrap();
        Ok((
            v.artifact,
            Box::new(std::io::Cursor::new(view_bytes(id.as_str()))),
        ))
    }
    fn conditions(
        &self,
        _: &ExecutionScope,
        q: &ConditionsQuery,
    ) -> ServiceResult<ConditionsTable> {
        let mut t: ConditionsTable = fixture("data/conditions_table.json");
        t.next_cursor = match q.cursor {
            None => Some(PageCursor::new("page-2").unwrap()),
            Some(_) => None,
        };
        Ok(t)
    }
    fn conditions_snapshot(&self, _: &ExecutionScope) -> ServiceResult<Digest> {
        Ok(snapshot())
    }
    fn run_partitions(
        &self,
        _: &ExecutionScope,
        run: &RunId,
    ) -> ServiceResult<std::collections::BTreeSet<Partition>> {
        Ok(match run.as_str() {
            TEST_RUN => [Partition::Test].into(),
            TRAIN_RUN => [Partition::Train].into(),
            _ => Default::default(),
        })
    }
}

fn sandbox(dir: &std::path::Path) -> Arc<Sandbox> {
    Arc::new(
        Sandbox::preflight(SandboxConfig {
            engine: env!("CARGO_BIN_EXE_ignis-mcp-fake-engine").into(),
            image: Digest::parse(&format!("sha256:{}", "e".repeat(64))).unwrap(),
            oci_runtime: "runsc".into(),
            require_gvisor: true,
            work_dir: dir.join("runner"),
            limits: RunnerLimits {
                wall_seconds: 60,
                memory_bytes: 256 << 20,
                cpu_millis: 1000,
                pids: 64,
                tmp_bytes: 16 << 20,
                output_bytes: 1 << 20,
                stream_bytes: 1 << 20,
            },
        })
        .unwrap_or_else(|e| panic!("preflight over the fake engine: {e}")),
    )
}

fn research() -> ExecutionScope {
    let mut sc = scope("ws_a");
    sc.capabilities.insert(Capability::ExecuteCode);
    sc
}

fn evaluation() -> ExecutionScope {
    let mut sc = research();
    sc.scope_id = ScopeId::new("scope_ws_a_eval").unwrap();
    sc.role = ScopeRole::Evaluation;
    sc.partitions = [Partition::Test].into();
    sc
}

struct T {
    h: Host,
    sb: Arc<Sandbox>,
    ds: Arc<RefDatasets>,
    _runner: tempfile::TempDir,
}

fn setup() -> T {
    let runner = tempfile::tempdir().unwrap();
    let sb = sandbox(runner.path());
    let ds = Arc::new(RefDatasets);
    let h = host_full(
        Some(sb.clone()),
        Some(ds.clone() as Arc<dyn DatasetService>),
    );
    T {
        h,
        sb,
        ds,
        _runner: runner,
    }
}

impl T {
    /// What the server's job runner does with the next queued analysis job.
    fn run_next(&self) -> ServiceResult<Vec<ArtifactRef>> {
        let claimed = self
            .h
            .store
            .claim_job("test", &[ignis_mcp::ANALYSIS_JOB], Duration::from_secs(60))
            .unwrap()
            .expect("a queued analysis job");
        let req: AnalysisExecuteRequest = serde_json::from_value(claimed.input.clone()).unwrap();
        self.sb.execute(
            &claimed.scope,
            &req,
            self.h.artifacts.as_ref(),
            Some(self.ds.as_ref()),
            Producer {
                job_id: Some(claimed.job_id.clone()),
                ..Default::default()
            },
            &AtomicBool::new(false),
        )
    }

    fn queued(&self) -> bool {
        self.h
            .store
            .claim_job("probe", &[ignis_mcp::ANALYSIS_JOB], Duration::from_secs(60))
            .unwrap()
            .is_some()
    }

    fn bytes(&self, sc: &ExecutionScope, a: &ArtifactRef) -> Vec<u8> {
        let mut b = vec![];
        std::io::Read::read_to_end(
            &mut self.h.artifacts.open(sc, &a.artifact_id, 0, None).unwrap(),
            &mut b,
        )
        .unwrap();
        b
    }
}

fn call(inputs: Value, crid: &str) -> Value {
    json!({"language": "python", "source": "print('fake engine runs nothing')", "inputs": inputs,
           "client_request_id": crid})
}

#[test]
fn references_are_resolved_by_the_host_and_the_record_names_what_the_program_read() {
    let t = setup();
    let sc = research();
    let csv =
        t.h.artifacts
            .put(
                &sc,
                ArtifactDraft {
                    media_type: "text/csv".into(),
                    contract: None,
                    produced_by: Producer::default(),
                    inputs: vec![],
                    origin: Origin::SyntheticFixture,
                },
                &mut b"run,value\nr1,1.5\n".as_slice(),
            )
            .unwrap();
    let inputs = json!([
        {"name": "notes.csv", "artifact_id": csv.artifact_id.as_str()},
        {"name": "obs.parquet", "dataset_view": {"dataset_id": BUILDER, "view": VIEW}},
        {"name": "conditions.json", "conditions": {"schema_version": "dataset.v2", "page_size": 1}},
    ]);
    let r =
        t.h.host
            .call(&sc, "analysis_execute", call(inputs, "m9-refs"));
    assert_eq!(r.execution_status, ExecutionStatus::Accepted, "{r:?}");

    let out = t.run_next().unwrap();
    let record: ExecutionRecord = serde_json::from_slice(&t.bytes(&sc, &out[0])).unwrap();
    record.validate().unwrap();
    let names: Vec<&str> = record.inputs.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["notes.csv", "obs.parquet", "conditions.json"]);

    // The artifact: the workspace bytes, nothing resolved.
    assert_eq!(record.inputs[0].artifact, csv);
    assert!(record.inputs[0].resolved.is_none());
    // The view: its Parquet artifact, the bundle identity, partitions and units.
    let bundle = RefDatasets::bundle(BUILDER);
    let view = bundle.views.iter().find(|v| v.name == VIEW).unwrap();
    assert_eq!(record.inputs[1].artifact, view.artifact);
    match &record.inputs[1].resolved {
        Some(ResolvedInput::DatasetView {
            dataset_id,
            bundle_digest,
            partitions,
            columns,
            time_origin,
            row_count,
            ..
        }) => {
            assert_eq!(dataset_id.as_str(), BUILDER);
            assert_eq!(*bundle_digest, bundle.content_digest());
            assert_eq!(*partitions, bundle.partitions);
            assert_eq!(*columns, view.columns);
            assert_eq!(*time_origin, bundle.time_origin);
            assert_eq!(*row_count, view.row_count);
        }
        other => panic!("view resolved to {other:?}"),
    }
    // The conditions: every page, stored as a ConditionsTable artifact, with the snapshot.
    let page: ConditionsTable = fixture("data/conditions_table.json");
    let cond = &record.inputs[2];
    assert_eq!(cond.artifact.contract.as_deref(), Some("ConditionsTable"));
    match &cond.resolved {
        Some(ResolvedInput::Conditions {
            snapshot: s,
            row_count,
            ..
        }) => {
            assert_eq!(*s, snapshot());
            assert_eq!(*row_count, 2 * page.rows.len() as u64, "both pages mounted");
        }
        other => panic!("conditions resolved to {other:?}"),
    }
    let table: ConditionsTable = serde_json::from_slice(&t.bytes(&sc, &cond.artifact)).unwrap();
    assert_eq!(table.rows.len(), 2 * page.rows.len());
    assert!(table.next_cursor.is_none());

    // What the program saw under /in is byte for byte what the record names, and
    // /job/inputs.json is the record's input list.
    let report = record
        .outputs
        .iter()
        .find(|o| o.path == "report.json")
        .expect("the fake engine's report");
    let report: Value = serde_json::from_slice(&t.bytes(&sc, &report.artifact)).unwrap();
    let seen: Vec<(String, String)> = report["in"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["name"].as_str().unwrap().into(),
                f["sha256"].as_str().unwrap().into(),
            )
        })
        .collect();
    let mut recorded: Vec<(String, String)> = record
        .inputs
        .iter()
        .map(|m| (m.name.clone(), m.artifact.sha256.to_string()))
        .collect();
    recorded.sort();
    assert_eq!(seen, recorded);
    assert_eq!(
        report["inputs_json"],
        serde_json::to_value(&record.inputs).unwrap()
    );

    // Outputs carry the lineage: the artifact, the dataset (not its view artifact, which is not a
    // workspace artifact) and the stored conditions table.
    let lineage = &record.outputs[0].artifact.inputs;
    assert!(lineage.contains(&EntityRef::Artifact(csv.artifact_id.clone())));
    assert!(lineage.contains(&EntityRef::Dataset(DatasetId::new(BUILDER).unwrap())));
    assert!(lineage.contains(&EntityRef::Artifact(cond.artifact.artifact_id.clone())));
    assert_eq!(record.origin, Origin::SyntheticFixture);
}

#[test]
fn a_test_partition_reference_is_refused_outside_an_evaluation_scope() {
    let t = setup();
    let holdout =
        json!([{"name": "obs.parquet", "dataset_view": {"dataset_id": HOLDOUT, "view": VIEW}}]);

    // A research scope: refused before anything is queued, by the host's partition check.
    let r = t.h.host.call(
        &research(),
        "analysis_execute",
        call(holdout.clone(), "m9-deny"),
    );
    assert_eq!(r.execution_status, ExecutionStatus::Refused, "{r:?}");
    let e = r.error.unwrap();
    assert_eq!(e.code, ErrorCode::ScopeDenied);
    assert!(e.message.contains("Test"), "{}", e.message);
    assert!(!t.queued(), "a refused reference queues nothing");

    // The evaluation scope may mount it; its partitions are on the record.
    let ev = evaluation();
    let r =
        t.h.host
            .call(&ev, "analysis_execute", call(holdout, "m9-eval"));
    assert_eq!(r.execution_status, ExecutionStatus::Accepted, "{r:?}");
    let out = t.run_next().unwrap();
    let record: ExecutionRecord = serde_json::from_slice(&t.bytes(&ev, &out[0])).unwrap();
    assert!(matches!(
        &record.inputs[0].resolved,
        Some(ResolvedInput::DatasetView { partitions, .. }) if partitions.contains(&Partition::Test)
    ));

    // An output of that run lives in the same workspace. A research scope naming it is refused
    // through its lineage, at the tool and again when a job stages it.
    let derived = record.outputs[0].artifact.clone();
    let via_lineage = json!([{"name": "r.json", "artifact_id": derived.artifact_id.as_str()}]);
    let r = t.h.host.call(
        &research(),
        "analysis_execute",
        call(via_lineage.clone(), "m9-lineage"),
    );
    assert_eq!(r.execution_status, ExecutionStatus::Refused, "{r:?}");
    let e = r.error.unwrap();
    assert_eq!(e.code, ErrorCode::ScopeDenied);
    assert!(e.message.contains(HOLDOUT), "{}", e.message);
    assert!(!t.queued());

    let req: AnalysisExecuteRequest =
        serde_json::from_value(call(via_lineage, "m9-direct")).unwrap();
    let e =
        t.sb.execute(
            &research(),
            &req,
            t.h.artifacts.as_ref(),
            Some(t.ds.as_ref()),
            Producer::default(),
            &AtomicBool::new(false),
        )
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::ScopeDenied);
}

/// An artifact of this workspace whose lineage names `inputs` (any producer may write lineage).
fn derived_from(t: &T, sc: &ExecutionScope, inputs: Vec<EntityRef>) -> ArtifactRef {
    t.h.artifacts
        .put(
            sc,
            ArtifactDraft {
                media_type: "application/json".into(),
                contract: None,
                produced_by: Producer::default(),
                inputs,
                origin: Origin::SyntheticFixture,
            },
            &mut b"{\"x\": 1}".as_slice(),
        )
        .unwrap()
}

#[test]
fn run_lineage_reaching_the_test_partition_is_refused_and_an_unknown_partition_fails_closed() {
    let t = setup();
    let sc = research();
    let run = |id: &str| EntityRef::Run(RunId::new(id).unwrap());
    let mount = |a: &ArtifactRef, crid: &str| {
        t.h.host.call(
            &sc,
            "analysis_execute",
            call(
                json!([{"name": "x.json", "artifact_id": a.artifact_id.as_str()}]),
                crid,
            ),
        )
    };

    // Directly and through an intermediate artifact: a test-partition run is refused.
    let direct = derived_from(&t, &sc, vec![run(TEST_RUN)]);
    let via = derived_from(
        &t,
        &sc,
        vec![EntityRef::Artifact(direct.artifact_id.clone())],
    );
    for (a, crid) in [(&direct, "lin-test"), (&via, "lin-test-via")] {
        let r = mount(a, crid);
        assert_eq!(
            r.execution_status,
            ExecutionStatus::Refused,
            "{crid}: {r:?}"
        );
        let e = r.error.unwrap();
        assert_eq!(e.code, ErrorCode::ScopeDenied, "{crid}");
        assert!(e.message.contains(TEST_RUN), "{crid}: {}", e.message);
    }
    // A run in no frozen split, an observation, an artifact not in the store: the partition is
    // unknown, so refused.
    for (inputs, crid) in [
        (vec![run("run-nowhere")], "lin-unknown"),
        (
            vec![EntityRef::Observation(ObservationId::new("obs-1").unwrap())],
            "lin-obs",
        ),
        (
            vec![EntityRef::Artifact(
                ArtifactId::new("art-not-in-this-store").unwrap(),
            )],
            "lin-missing",
        ),
    ] {
        let r = mount(&derived_from(&t, &sc, inputs), crid);
        assert_eq!(
            r.execution_status,
            ExecutionStatus::Refused,
            "{crid}: {r:?}"
        );
        assert_eq!(r.error.unwrap().code, ErrorCode::ScopeDenied, "{crid}");
    }
    assert!(!t.queued(), "nothing refused was queued");

    // The job re-checks at staging (a request that bypassed the tool is refused too).
    let req: AnalysisExecuteRequest = serde_json::from_value(call(
        json!([{"name": "x.json", "artifact_id": direct.artifact_id.as_str()}]),
        "lin-direct-job",
    ))
    .unwrap();
    let e =
        t.sb.execute(
            &sc,
            &req,
            t.h.artifacts.as_ref(),
            Some(t.ds.as_ref()),
            Producer::default(),
            &AtomicBool::new(false),
        )
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::ScopeDenied);

    // A train run is readable by the research scope; the evaluation scope may read the test run.
    let r = mount(&derived_from(&t, &sc, vec![run(TRAIN_RUN)]), "lin-train");
    assert_eq!(r.execution_status, ExecutionStatus::Accepted, "{r:?}");
    let ev = evaluation();
    let a = derived_from(&t, &ev, vec![run(TEST_RUN)]);
    let r = t.h.host.call(
        &ev,
        "analysis_execute",
        call(
            json!([{"name": "x.json", "artifact_id": a.artifact_id.as_str()}]),
            "lin-eval",
        ),
    );
    assert_eq!(r.execution_status, ExecutionStatus::Accepted, "{r:?}");
}

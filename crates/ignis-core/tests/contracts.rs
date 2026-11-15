//! C0 contract tests. Each test guards a failure mode that would be expensive to find later:
//! schema/serde drift seen by TypeScript consumers, fixtures that silently stop matching their
//! contract, a handwritten release labelled NASA-trained, malformed literal patches, leaked test
//! partitions, previews or predictions passed off as results or measurements, broken event replay.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ignis_core::contracts::*;
use ignis_core::registry::{self, Tensor, TensorMap};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json(rel: &str) -> Value {
    let p = repo().join(rel);
    serde_json::from_str(
        &std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display())),
    )
    .unwrap()
}

fn fixture(rel: &str) -> Value {
    read_json(&format!("fixtures/contracts/{rel}"))
}

fn typed<T: DeserializeOwned>(v: &Value) -> T {
    serde_json::from_value(v.clone()).unwrap_or_else(|e| panic!("{e}\n{v:#}"))
}

/// Parse, check the invariants, and require the fixture to be the exact wire form (no field dropped,
/// renamed or defaulted on the way through).
fn check<T: DeserializeOwned + serde::Serialize>(
    v: &Value,
    validate: impl Fn(&T) -> Result<(), ContractError>,
) -> T {
    let t: T = typed(v);
    validate(&t).unwrap_or_else(|e| panic!("{e}\n{v:#}"));
    assert_eq!(
        &serde_json::to_value(&t).unwrap(),
        v,
        "fixture is not the canonical wire form"
    );
    t
}

fn origins(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                if k == "origin" && x.is_string() {
                    out.push(x.as_str().unwrap().to_string());
                }
                origins(x, out);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| origins(x, out)),
        _ => {}
    }
}

#[test]
fn fixtures_match_schemas_contracts_and_wire_form() {
    let manifest = fixture("manifest.json");
    assert_eq!(manifest["origin"], "synthetic_fixture");
    let exported: BTreeSet<String> = ignis_core::schema::all()
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    for f in manifest["fixtures"].as_array().unwrap() {
        let path = f["path"].as_str().unwrap();
        let contract = f["contract"].as_str().unwrap();
        let v = fixture(path);
        let base = contract.trim_end_matches("[]");
        assert!(
            exported.contains(base),
            "{path}: {base} is not an exported contract"
        );

        // 1. The committed JSON Schema (what TypeScript is generated from) accepts the fixture.
        let schema = read_json(&format!("schemas/{base}.json"));
        let validator = jsonschema::validator_for(&schema).unwrap();
        let items: Vec<&Value> = if contract.ends_with("[]") {
            v.as_array().unwrap().iter().collect()
        } else {
            vec![&v]
        };
        for item in &items {
            let errors: Vec<String> = validator.iter_errors(item).map(|e| e.to_string()).collect();
            assert!(
                errors.is_empty(),
                "{path}: schema {base} rejects the fixture: {errors:?}"
            );
        }

        // 2. Every payload in a fixture is labelled synthetic.
        let mut o = Vec::new();
        origins(&v, &mut o);
        assert!(
            o.iter().all(|x| x == "synthetic_fixture"),
            "{path}: origins {o:?}"
        );
        assert!(
            !o.is_empty()
                || base == "ExecutionScope"
                || base.ends_with("Request")
                || base == "ModelSpec"
                || base == "WikiPatch"
                || base == "AnswerSubmission"
                || base == "Capabilities"
                || base == "ErrorEnvelope",
            "{path}: no origin label"
        );

        // 3. Rust parses it, its invariants hold, and it round-trips exactly.
        match contract {
            "ExecutionScope" => drop(check::<ExecutionScope>(&v, ExecutionScope::validate)),
            "ResearchEpisode" => drop(check::<ResearchEpisode>(&v, ResearchEpisode::validate)),
            "ResearchEvent[]" => drop(check::<Vec<ResearchEvent>>(&v, |e| check_event_log(e))),
            "ToolResult" => drop(check::<ToolResult>(&v, ToolResult::validate)),
            "ErrorEnvelope" => drop(check::<ErrorEnvelope>(&v, ErrorEnvelope::validate)),
            "ExecutionRecord" => drop(check::<ExecutionRecord>(&v, ExecutionRecord::validate)),
            "ResearchAnswer" => drop(check::<ResearchAnswer>(&v, ResearchAnswer::validate)),
            "AnswerSubmission" => drop(check::<AnswerSubmission>(&v, AnswerSubmission::validate)),
            "AnswerRefusal" => drop(check::<AnswerRefusal>(&v, AnswerRefusal::validate)),
            "TraceBundle" => drop(check::<TraceBundle>(&v, TraceBundle::validate)),
            "Capabilities" => drop(check::<Capabilities>(&v, Capabilities::validate)),
            "SearchRequest" => drop(check::<SearchRequest>(&v, SearchRequest::validate)),
            "SearchPage" => drop(check::<SearchPage>(&v, SearchPage::validate)),
            "ReadResult" => drop(check::<ReadResult>(&v, ReadResult::validate)),
            "SourceOutline" => drop(check::<SourceOutline>(&v, SourceOutline::validate)),
            "SourceAccess" => drop(check::<SourceAccess>(&v, SourceAccess::validate)),
            "MediaKeyframeIndex" => drop(check::<MediaKeyframeIndex>(
                &v,
                MediaKeyframeIndex::validate,
            )),
            "ArchiveListing" => drop(check::<ArchiveListing>(&v, ArchiveListing::validate)),
            "SourceLabel[]" => drop(check::<Vec<SourceLabel>>(&v, |l| {
                l.iter().try_for_each(SourceLabel::validate)
            })),
            "LabelSet" => drop(check::<LabelSet>(&v, LabelSet::validate)),
            "GraphNeighborsPage" => drop(check::<GraphNeighborsPage>(
                &v,
                GraphNeighborsPage::validate,
            )),
            "WikiPatch" => drop(check::<WikiPatch>(&v, WikiPatch::validate)),
            "WikiRevision" => drop(check::<WikiRevision>(&v, WikiRevision::validate)),
            "WikiReadResult" => drop(check::<WikiReadResult>(&v, WikiReadResult::validate)),
            "WebSearchPage" => drop(check::<WebSearchPage>(&v, WebSearchPage::validate)),
            "ExternalSourceCandidate" => drop(check::<ExternalSourceCandidate>(
                &v,
                ExternalSourceCandidate::validate,
            )),
            "ConditionsTable" => drop(check::<ConditionsTable>(&v, ConditionsTable::validate)),
            "DatasetBundle" => drop(check::<DatasetBundle>(&v, DatasetBundle::validate)),
            "VisualObservationBundle" => drop(check::<VisualObservationBundle>(
                &v,
                VisualObservationBundle::validate,
            )),
            "ImageContourObservation[]" => drop(check::<Vec<ImageContourObservation>>(&v, |r| {
                r.iter().try_for_each(ImageContourObservation::validate)
            })),
            "ModelSpec" => drop(check::<ModelSpec>(&v, ModelSpec::validate)),
            "HandwrittenWeightPatch" => drop(check::<HandwrittenWeightPatch>(&v, |p| {
                p.exposure.validate()
            })),
            "ModelRelease" => drop(check::<ModelRelease>(&v, ModelRelease::check_construction)),
            "EvaluationRequest" => {
                drop(check::<EvaluationRequest>(&v, EvaluationRequest::validate))
            }
            "EvaluationResult" => drop(check::<EvaluationResult>(&v, EvaluationResult::validate)),
            other => panic!("{path}: no check registered for {other}"),
        }
    }
}

/// Test-only literal materialization (zeros, spec-fixed values, then the patch's assignments). The
/// real assembler is `ignis-wsp`; this proves the contracts are sufficient to rebuild the tensors.
fn materialize(spec: &ModelSpec, patch: &HandwrittenWeightPatch) -> TensorMap {
    let mut m = TensorMap::new();
    for t in &spec.tensors {
        let data = match &t.fill {
            TensorFill::Fixed { values } => values.iter().map(|v| v.get()).collect(),
            _ => vec![0.0; t.numel()],
        };
        m.insert(
            t.name.clone(),
            Tensor::new(t.shape.iter().map(|&d| d as usize).collect(), data),
        );
    }
    for op in &patch.ops {
        let shape = &spec.tensor(op.tensor()).unwrap().shape;
        for (i, v) in op.assignments(shape).unwrap() {
            m.get_mut(op.tensor()).unwrap().data[i] = v.get();
        }
    }
    m
}

struct Models {
    spec: ModelSpec,
    patch: HandwrittenWeightPatch,
    release: ModelRelease,
}

fn models() -> Models {
    Models {
        spec: typed(&fixture("models/model_spec.json")),
        patch: typed(&fixture("models/patch_w0.json")),
        release: typed(&fixture("models/release_w0.json")),
    }
}

#[test]
fn fixtures_are_mutually_consistent() {
    let Models {
        spec,
        patch,
        release,
    } = models();
    let mut wrong = Vec::new();
    let mut expect = |what: &str, got: String, want: String| {
        if got != want {
            wrong.push(format!("{what}: is {got}, should be {want}"));
        }
    };
    // The release is rebuildable from spec + literal patch alone.
    let tensors = materialize(&spec, &patch);
    let names: Vec<String> = tensors.keys().cloned().collect();
    let entries = registry::tensor_entries(&tensors, &names);
    for e in &entries {
        let f = release
            .tensors
            .iter()
            .find(|t| t.name == e.name)
            .map(|t| t.sha256.clone())
            .unwrap_or_default();
        expect(&format!("release tensor {}", e.name), f, e.sha256.clone());
    }
    let m = release.construction.as_ref().unwrap();
    expect(
        "patch.spec_hash",
        patch.spec_hash.to_string(),
        spec.spec_hash().to_string(),
    );
    expect(
        "manifest.spec_hash",
        m.spec_hash.to_string(),
        spec.spec_hash().to_string(),
    );
    expect(
        "manifest.patch_hash",
        m.patch_hash.as_ref().unwrap().to_string(),
        patch.patch_hash().to_string(),
    );
    expect(
        "manifest.weights_hash",
        m.weights_hash.to_string(),
        weights_digest(&entries).to_string(),
    );
    let release_id = ignis_core::hashing::content_hash(&release, &["id", "created_at"]);
    expect("release.id", release.id.clone(), release_id.clone());

    let bundle: DatasetBundle = typed(&fixture("data/dataset_bundle.json"));
    let request: EvaluationRequest = typed(&fixture("evaluation/request_dev.json"));
    let result: EvaluationResult = typed(&fixture("evaluation/result_dev.json"));
    expect(
        "request.dataset_hash",
        request.dataset_hash.to_string(),
        bundle.content_digest().to_string(),
    );
    expect(
        "request.candidate.release_id",
        request.candidate.release_id.to_string(),
        release.id.clone(),
    );
    expect(
        "result.request_hash",
        result.request_hash.to_string(),
        request.request_hash().to_string(),
    );
    if result.request != request {
        wrong.push("result.request differs from evaluation/request_dev.json".into());
    }
    // Every reference to the release across fixtures names the real release id.
    for path in [
        "research/episode.json",
        "research/events.json",
        "research/answer.json",
        "evaluation/result_dev.json",
    ] {
        let text = std::fs::read_to_string(repo().join("fixtures/contracts").join(path)).unwrap();
        if text.contains("fixture-release-placeholder") {
            wrong.push(format!(
                "{path}: replace fixture-release-placeholder with {}",
                release.id
            ));
        }
    }
    assert!(wrong.is_empty(), "fixtures drifted:\n{}", wrong.join("\n"));

    patch.validate_against(&spec).unwrap();
    spec.check_release(&release).unwrap();
    assert_eq!(bundle.family, spec.family);
}

#[test]
fn a_handwritten_release_is_never_labelled_nasa_trained() {
    let Models {
        spec,
        patch,
        release,
    } = models();
    let mut v = serde_json::to_value(&release).unwrap();

    // The legacy reader keeps the 2.0.0 meaning only for legacy files (no construction record).
    let mut legacy = v.clone();
    let m = legacy.as_object_mut().unwrap();
    m.remove("construction");
    m.remove("trained_on");
    m.insert("state".into(), json!("fit_only"));
    m.insert("lineage".into(), json!("initial_fit"));
    assert_eq!(
        registry::parse_release(&legacy.to_string())
            .unwrap()
            .trained_on,
        TrainedOn::Nasa
    );

    // A new-format file must state trained_on; it is never defaulted.
    v.as_object_mut().unwrap().remove("trained_on");
    assert!(registry::parse_release(&v.to_string()).is_err());
    assert!(serde_json::from_value::<ModelRelease>(v.clone()).is_err());

    // Literal construction and NASA training contradict each other, in both directions.
    let mut nasa = release.clone();
    nasa.trained_on = TrainedOn::Nasa;
    assert!(nasa.check_construction().is_err());
    let mut bare = release.clone();
    bare.construction = None;
    assert!(
        bare.check_construction().is_err(),
        "not_trained without a manifest"
    );
    let mut fitted_state = release.clone();
    fitted_state.state = QualificationState::ForecastQualified;
    assert!(
        fitted_state.check_construction().is_err(),
        "a literal release is created experimental"
    );

    // The registry enforces it on save and on load; a lab edit that keeps the manifest but changes a
    // tensor no longer matches weights_hash and is refused.
    let root = std::env::temp_dir().join(format!("ignis-c0-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let tensors = materialize(&spec, &patch);
    let frozen: Vec<String> = tensors.keys().cloned().collect();
    assert!(registry::save_release(&root, nasa, &tensors, &TensorMap::new(), &frozen).is_err());
    let saved =
        registry::save_release(&root, release.clone(), &tensors, &TensorMap::new(), &frozen)
            .unwrap();
    assert_eq!(saved.id, release.id);
    let loaded = registry::load(&root, &saved.id).unwrap();
    assert_eq!(loaded.release.trained_on, TrainedOn::NotTrained);
    let mut edited = tensors.clone();
    edited.get_mut("residual.W1").unwrap().data[0] = 0.5;
    assert!(registry::save_release(&root, release, &edited, &TensorMap::new(), &frozen).is_err());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn literal_patches_reject_anything_but_explicit_in_bounds_values() {
    let Models { spec, patch, .. } = models();
    patch.validate_against(&spec).unwrap();
    let with_ops = |ops: Value| {
        let mut v = serde_json::to_value(&patch).unwrap();
        v["ops"] = ops;
        typed::<HandwrittenWeightPatch>(&v)
    };
    let w1 = json!({"op": "set_box", "tensor": "residual.W1", "start": [0, 0], "shape": [2, 2], "values": [0.1, 0.2, 0.3, 0.4]});
    let k0 = json!({"op": "set", "tensor": "physics.k0", "index": [], "value": 1e-6});
    let cases = [
        (
            "index out of bounds",
            json!([w1, k0, {"op": "set", "tensor": "residual.a1", "index": [2], "value": 0.1}]),
        ),
        (
            "box outside the tensor",
            json!([w1, k0, {"op": "fill_box", "tensor": "residual.a1", "start": [1], "shape": [2], "value": 0.0}]),
        ),
        (
            "value count != box size",
            json!([{"op": "set_box", "tensor": "residual.W1", "start": [0, 0], "shape": [2, 2], "values": [0.1]}, k0]),
        ),
        (
            "element written twice",
            json!([w1, k0, {"op": "set", "tensor": "residual.W1", "index": [1, 1], "value": 0.0}]),
        ),
        (
            "spec-fixed constant",
            json!([w1, k0, {"op": "set", "tensor": "physics.time_scale", "index": [], "value": 2.0}]),
        ),
        (
            "unknown tensor",
            json!([w1, k0, {"op": "set", "tensor": "residual.W9", "index": [0], "value": 0.0}]),
        ),
        ("physical parameter left unwritten", json!([w1])),
        (
            "physical parameter outside bounds",
            json!([w1, {"op": "set", "tensor": "physics.k0", "index": [], "value": 1.0}]),
        ),
        (
            "partially written author_writes_all",
            json!([{"op": "set", "tensor": "residual.W1", "index": [0, 0], "value": 0.1}, k0]),
        ),
    ];
    for (why, ops) in cases {
        assert!(
            with_ops(ops).validate_against(&spec).is_err(),
            "accepted: {why}"
        );
    }
    // Code, expressions and NaN are not representable at all.
    for bad in [
        json!({"op": "set", "tensor": "physics.k0", "index": [], "value": "8e-7*2"}),
        json!({"op": "python", "code": "w = fit()"}),
        json!({"op": "set", "tensor": "physics.k0", "index": [], "value": null}),
    ] {
        assert!(
            serde_json::from_value::<LiteralOp>(bad.clone()).is_err(),
            "parsed: {bad}"
        );
    }
    // Wrong spec, and test exposure.
    let mut other = patch.clone();
    other.spec_hash = Digest::of_bytes(b"another spec");
    assert!(other.validate_against(&spec).is_err());
    let mut leaked = patch.clone();
    leaked.exposure.datasets.push(ExposedDataset {
        dataset_id: DatasetId::new("d").unwrap(),
        partitions: [Partition::Test].into(),
    });
    assert!(leaked.validate_against(&spec).is_err());
}

#[test]
fn wire_primitives_reject_ambiguous_values() {
    assert!(Finite::new(f64::NAN).is_err() && Finite::new(f64::INFINITY).is_err());
    assert!(
        serde_json::from_str::<Finite>("null").is_err(),
        "null is not a number"
    );
    assert!(serde_json::from_str::<Digest>("\"sha256:ABC\"").is_err());
    assert!(
        serde_json::from_str::<Digest>(&format!("\"{}\"", "a".repeat(64))).is_err(),
        "bare hex is not a Digest"
    );
    assert!(
        serde_json::from_str::<Timestamp>("\"2026-11-14T18:00:00+02:00\"").is_err(),
        "UTC only"
    );
    assert!(
        serde_json::from_str::<EpisodeId>("\"\"").is_err()
            && serde_json::from_str::<EpisodeId>("\"a b\"").is_err()
    );
    // Another family's or version's payload is not misread.
    let mut ev = fixture("research/tool_accepted.json");
    ev["schema_version"] = json!("tools.v2");
    assert!(serde_json::from_value::<ToolResult>(ev).is_err());
}

#[test]
fn locators_added_fields_do_not_change_legacy_wire_form() {
    // Pre-2.1 locators (embedded in observations, hence in prefix input hashes) serialize unchanged.
    let legacy = json!({"asset_sha256": "0".repeat(64), "path": "run.csv", "row": 3, "column": "d_mm", "page": null, "frame": null, "pts_s": null, "char_start": null, "char_end": null});
    let l: SourceLocator = typed(&legacy);
    assert_eq!(serde_json::to_value(&l).unwrap(), legacy);
    // Citable offsets need their parse view and unit; bboxes need a page and lie in [0, 1].
    let mut cite = l.clone();
    cite.char_start = Some(1);
    cite.char_end = Some(5);
    assert!(cite.validate().is_err());
    cite.parse_view_id = Some("pv".into());
    cite.offset_unit = Some(OffsetUnit::Utf8Byte);
    cite.validate().unwrap();
    let mut boxed = l.clone();
    boxed.bbox = Some([0.1, 0.1, 0.9, 0.9]);
    assert!(boxed.validate().is_err(), "bbox without page");
    boxed.page = Some(1);
    boxed.validate().unwrap();
    boxed.bbox = Some([0.9, 0.1, 0.1, 1.2]);
    assert!(boxed.validate().is_err());
}

#[test]
fn scopes_keep_the_lockbox_and_cannot_widen() {
    let scope: ExecutionScope = typed(&fixture("common/scope_research.json"));
    assert!(scope.require_partition(Partition::Validation).is_ok());
    assert!(scope.require_partition(Partition::Test).is_err());
    let mut forged = scope.clone();
    forged.partitions.insert(Partition::Test);
    assert!(forged.validate().is_err());
    assert!(
        forged.require_partition(Partition::Test).is_err(),
        "a research scope never grants the lockbox"
    );
    let mut evaluator = forged.clone();
    evaluator.role = ScopeRole::Evaluation;
    assert!(evaluator.require_partition(Partition::Test).is_ok());
    let mut author = scope.clone();
    author.role = ScopeRole::Authoring;
    author.capabilities.insert(Capability::ExecuteCode);
    assert!(author.validate().is_err(), "authoring cannot execute code");

    let child = scope
        .derive_child(
            ScopeId::new("child").unwrap(),
            [Partition::Train].into(),
            [Capability::ReadSources].into(),
        )
        .unwrap();
    assert_eq!(child.parent_scope_id, Some(scope.scope_id.clone()));
    assert!(scope
        .derive_child(
            ScopeId::new("c2").unwrap(),
            [Partition::Test].into(),
            BTreeSet::new()
        )
        .is_err());
    assert!(scope
        .derive_child(
            ScopeId::new("c3").unwrap(),
            BTreeSet::new(),
            [Capability::ExecuteCode].into()
        )
        .is_err());
}

#[test]
fn tool_results_keep_execution_and_science_apart() {
    let accepted = fixture("research/tool_accepted.json");
    let failed = fixture("research/tool_failed.json");
    let insufficient = fixture("research/tool_insufficient_evidence.json");
    let broken = |base: &Value, patch: Value| {
        let mut v = base.clone();
        for (k, x) in patch.as_object().unwrap() {
            if x.is_null() {
                v.as_object_mut().unwrap().remove(k);
            } else {
                v[k] = x.clone();
            }
        }
        typed::<ToolResult>(&v).validate().is_err()
    };
    assert!(
        broken(&accepted, json!({"job": null})),
        "accepted without a job handle"
    );
    assert!(
        broken(&accepted, json!({"scientific_status": "conclusive"})),
        "assessed before the job ran"
    );
    assert!(
        broken(&failed, json!({"error": null})),
        "failure without an error"
    );
    assert!(
        broken(&failed, json!({"scientific_status": "inconclusive"})),
        "a technical failure is not a scientific outcome"
    );
    assert!(
        broken(
            &insufficient,
            json!({"error": {"code": "internal", "message": "x", "retry": {"kind": "never"}}})
        ),
        "completed with an error"
    );
    assert!(
        broken(
            &insufficient,
            json!({"result": {"value": {}, "partial": true}})
        ),
        "a preview without the full artifact"
    );
}

#[test]
fn event_logs_replay_without_gaps_or_contradictions() {
    let events: Vec<ResearchEvent> = typed(&fixture("research/events.json"));
    check_event_log(&events).unwrap();
    check_event_log(&events[7..]).unwrap(); // replay after cursor seq=7
    let mut gap = events.clone();
    gap.remove(4);
    assert!(check_event_log(&gap).is_err());
    let mut dup = events.clone();
    dup[3].event_id = dup[2].event_id.clone();
    assert!(check_event_log(&dup).is_err());
    let mut other = events.clone();
    other[5].episode_id = EpisodeId::new("another").unwrap();
    assert!(check_event_log(&other).is_err());
    let mut jump = events.clone();
    if let EventBody::StatusChanged { from, .. } = &mut jump[14].body {
        *from = EpisodeStatus::Paused;
    }
    assert!(
        check_event_log(&jump).is_err(),
        "status changes start where the last one ended"
    );
    let mut revived = events.clone();
    revived.push(ResearchEvent {
        seq: 16,
        event_id: EventId::new("e16").unwrap(),
        ..events[6].clone()
    });
    assert!(
        check_event_log(&revived).is_err(),
        "a succeeded job does not run again"
    );
}

#[test]
fn answers_cannot_report_predictions_as_observations() {
    let base = fixture("research/answer.json");
    typed::<ResearchAnswer>(&base).validate().unwrap();
    let with_support = |claim: usize, support: Value| {
        let mut v = base.clone();
        v["claims"][claim]["support"] = support;
        typed::<ResearchAnswer>(&v).validate().is_err()
    };
    let prediction = json!([{"kind": "artifact_value", "artifact_id": "fixture-artifact-forecast-01", "pointer": "/mean/3"}]);
    assert!(
        with_support(1, prediction.clone()),
        "observed claim backed by a model output"
    );
    assert!(
        with_support(
            2,
            json!([{"kind": "evidence", "evidence_id": "fixture-evidence-02"}])
        ),
        "prediction without the model artifact"
    );
    assert!(
        with_support(
            0,
            json!([{"kind": "evidence", "evidence_id": "no-such-evidence"}])
        ),
        "dangling evidence"
    );
    let mut v = base.clone();
    v["evidence"][1]["evidence_class"] = json!("model_output");
    assert!(
        typed::<ResearchAnswer>(&v).validate().is_err(),
        "an observed claim cites model output"
    );
}

/// M7b: a number in a claim's value slot always carries the cell it was read from (digest, row
/// keys, column), never a position, and its tier fixes the artifact it may come from.
#[test]
fn claim_values_bind_numbers_to_cells_never_to_positions() {
    let sub = fixture("research/answer_submission.json");
    let bound = &sub["claims"][2]["values"][0];
    typed::<ClaimValue>(bound).validate().unwrap();
    assert!(
        typed::<AnswerSubmission>(&sub).problems().is_empty(),
        "a bound value supports a predicted claim"
    );

    // A bare numeral (no source) does not parse; a malformed binding is refused by validate().
    let mut bare = bound.clone();
    bare.as_object_mut().unwrap().remove("source");
    assert!(serde_json::from_value::<ClaimValue>(bare).is_err());
    let refused = |f: &dyn Fn(&mut Value)| {
        let mut v = bound.clone();
        f(&mut v);
        typed::<ClaimValue>(&v).validate().is_err()
    };
    assert!(refused(&|v| v["unit"] = json!(" ")), "no unit");
    assert!(refused(&|v| v["significant_digits"] = json!(0)));
    assert!(refused(&|v| v["significant_digits"] = json!(18)));
    assert!(
        refused(&|v| v["source"]["rows"][0]["where"] = json!([])),
        "a row is selected by key fields, never by position"
    );
    assert!(refused(
        &|v| v["source"]["rows"][0]["where"] =
            json!([{"field": "/t", "equals": 1}, {"field": "/t", "equals": 2}])
    ));
    assert!(refused(&|v| v["source"]["column"] = json!("mean")));
    assert!(refused(
        &|v| v["derivation"] = json!({"computation": "a-rec", "inputs": ["a-in", "a-in"]})
    ));
    assert!(refused(
        &|v| v["derivation"] = json!({"computation": "a-rec", "inputs": ["a-rec"]})
    ));
    // The reference is the source of truth: a value may be left to the host, but a stated value
    // carries its unit, and a quoted number is always stated (it is searched for in the quote).
    let mut unstated = bound.clone();
    unstated.as_object_mut().unwrap().remove("value");
    unstated.as_object_mut().unwrap().remove("unit");
    typed::<ClaimValue>(&unstated).validate().unwrap();
    assert!(refused(&|v| {
        v.as_object_mut().unwrap().remove("unit");
    }));
    assert!(!refused(
        &|v| v["source"] = json!({"kind": "evidence_quote", "evidence_id": "e-1"})
    ));
    let mut quote = unstated.clone();
    quote["source"] = json!({"kind": "evidence_quote", "evidence_id": "e-1"});
    assert!(typed::<ClaimValue>(&quote).validate().is_err());
    // A condition is read under a snapshot; a row without a run id is selected by locator.
    let mut cond = bound.clone();
    cond["source"] = json!({"kind": "conditions_cell", "snapshot": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "investigation_id": "inv-a", "run_id": "run-1", "key": "pressure"});
    typed::<ClaimValue>(&cond).validate().unwrap();
    cond["source"].as_object_mut().unwrap().remove("run_id");
    assert!(typed::<ClaimValue>(&cond).validate().is_err());
    cond["source"]["locator"] = json!({"asset_sha256": "2e", "path": "t.csv", "row": 5});
    typed::<ClaimValue>(&cond).validate().unwrap();
    for (path, v) in [
        ("unit", json!("")),
        (
            "resolved",
            json!({"level": "cell", "value": 0.69, "unit": "m"}),
        ),
    ] {
        let mut bad = sub.clone();
        bad["claims"][2]["values"][0][path] = v;
        assert_eq!(
            typed::<AnswerSubmission>(&bad)
                .problems()
                .into_iter()
                .map(|p| p.kind)
                .collect::<Vec<_>>(),
            vec![AnswerProblemKind::InvalidStructure],
            "{path}: malformed, or written by the agent instead of the host"
        );
    }

    // A grounding report lists grounding problems only, and an enforced answer stores none.
    let report = |mode: &str, kind: &str| {
        typed::<GroundingReport>(&json!({"mode": mode, "grounded": 0,
            "problems": [{"kind": kind, "detail": "x"}]}))
        .validate()
        .is_ok()
    };
    assert!(report("log", "value_mismatch"));
    assert!(!report("enforce", "value_mismatch"));
    assert!(!report("log", "unknown_evidence"));

    // In the stored answer, a value's artifact fits the claim's tier.
    let answer = fixture("research/answer_v2.json");
    let value_from = |artifact: &str| {
        let mut v = bound.clone();
        v["source"]["artifact_id"] = json!(artifact);
        v
    };
    let kinds = |claim: usize, value: Value| -> Vec<AnswerProblemKind> {
        let mut v = answer.clone();
        v["claims"][claim]["values"] = json!([value]);
        typed::<ResearchAnswer>(&v)
            .problems()
            .into_iter()
            .map(|p| p.kind)
            .collect()
    };
    use AnswerProblemKind as K;
    assert!(kinds(1, value_from("fixture-artifact-conditions-01")).is_empty());
    assert_eq!(
        kinds(1, value_from("fixture-artifact-forecast-01")),
        vec![K::KindSupportMismatch],
        "an observed number read from a model output"
    );
    assert_eq!(
        kinds(4, value_from("fixture-artifact-forecast-01")),
        vec![K::KindSupportMismatch],
        "a computed number read from a model output"
    );
    let mut derived = value_from("fixture-artifact-observed-01");
    derived["derivation"] = json!({"computation": "no-such-record", "inputs": []});
    assert_eq!(kinds(4, derived), vec![K::UnknownArtifact]);
}

#[test]
fn evaluation_counts_every_case_and_only_a_real_final_test_qualifies() {
    let base = fixture("evaluation/result_dev.json");
    let res = |f: &dyn Fn(&mut Value)| {
        let mut v = base.clone();
        f(&mut v);
        typed::<EvaluationResult>(&v).validate()
    };
    res(&|_| {}).unwrap();
    assert!(
        res(&|v| v["verdict"] = json!("qualified")).is_err(),
        "development feedback qualified a model"
    );
    assert!(
        res(&|v| v["cases"]["refused"] = json!(0)).is_err(),
        "a refused case dropped from the denominator"
    );
    assert!(res(&|v| v["coverage"] = json!(1.0)).is_err());
    assert!(
        res(&|v| v["failures"] = json!([])).is_err(),
        "failures hidden without per-case output"
    );
    assert!(
        res(&|v| v["metrics"][0]["n"] = json!(4)).is_err(),
        "scored-case metric over more than the scored cases"
    );
    let req = |f: &dyn Fn(&mut Value)| {
        let mut v = fixture("evaluation/request_dev.json");
        f(&mut v);
        typed::<EvaluationRequest>(&v).validate()
    };
    assert!(
        req(&|v| v["partition"] = json!("test")).is_err(),
        "development feedback on the test partition"
    );
    assert!(
        req(&|v| {
            v["purpose"] = json!("final_test");
            v["partition"] = json!("test");
        })
        .is_err(),
        "final test without a frozen protocol"
    );
    assert!(req(&|v| v["unit"] = json!("mm")).is_err());
}

#[test]
fn data_bundles_keep_test_outcomes_and_pixels_apart() {
    let bundle = fixture("data/dataset_bundle.json");
    let b = |f: &dyn Fn(&mut Value)| {
        let mut v = bundle.clone();
        f(&mut v);
        typed::<DatasetBundle>(&v).validate()
    };
    assert!(
        b(&|v| v["partitions"] = json!(["train", "test"])).is_err(),
        "builder bundle with test outcomes"
    );
    assert!(
        b(&|v| v["views"][0]["columns"][3]["unit"] = json!("mm")).is_err(),
        "display unit stored as data"
    );
    assert!(b(&|v| v["views"][0]["columns"][1]["unit"] = json!("ms")).is_err());

    let pixels: VisualObservationBundle = typed(&fixture("data/visual_bundle_pixel_only.json"));
    assert!(pixels.require_metric().is_err());
    let mut relabelled = pixels.clone();
    relabelled.contours.contract = Some("ContourObservation".into());
    assert!(
        relabelled.validate().is_err(),
        "pixel contours passed off as metres"
    );
    let mut undeclared = pixels.clone();
    undeclared.contours.contract = None;
    assert!(
        undeclared.validate().is_err(),
        "pixel contours must declare ImageContourObservation"
    );
    let records: Vec<ImageContourObservation> =
        typed(&fixture("data/image_contours_pixel_only.json"));
    let mut as_metres = records[0].clone();
    as_metres.units = UnitsStatus::Metric;
    assert!(
        as_metres.validate().is_err(),
        "image records claiming metres"
    );
    let mut future = records[0].clone();
    future.mask_dependency_frames.push(future.frame + 1);
    assert!(future.validate().is_err(), "non-causal mask");
    let mut metric = relabelled.clone();
    metric.units_status = UnitsStatus::Metric;
    assert!(metric.validate().is_err(), "metric without calibration");
    metric.calibration_id = Some("cal".into());
    metric.validate().unwrap();
    assert!(metric.require_metric().is_err(), "unresolved run identity");
}

#[test]
fn analysis_runs_cannot_escape_their_mounts_or_hide_truncation() {
    let req = |names: &[&str]| AnalysisExecuteRequest {
        language: AnalysisLanguage::Python,
        source: "print(1)".into(),
        inputs: names
            .iter()
            .map(|n| AnalysisInput {
                name: n.to_string(),
                artifact_id: ArtifactId::new("art_1").unwrap(),
            })
            .collect(),
        client_request_id: None,
    };
    assert!(req(&["series.parquet", "notes_v2.txt"]).validate().is_ok());
    for bad in ["../etc/passwd", "a/b", "..", ".env", "", r"a\b", "x y"] {
        assert!(req(&[bad]).validate().is_err(), "input name {bad:?}");
    }
    assert!(
        req(&["a", "a"]).validate().is_err(),
        "one name mounted twice"
    );

    let record = fixture("research/execution_record.json");
    let broken = |pointer: &str, x: Value| {
        let mut v = record.clone();
        *v.pointer_mut(pointer).unwrap() = x;
        typed::<ExecutionRecord>(&v).validate().is_err()
    };
    assert!(
        broken("/stdout/total_bytes", json!(1u64 << 40)),
        "a stream beyond the budget must say it was truncated"
    );
    assert!(
        broken("/stdout/truncated", json!(true)),
        "truncated without exceeding the budget"
    );
    assert!(
        broken("/outputs/0/path", json!("../escape.json")),
        "output outside /out"
    );
    assert!(
        broken("/outputs/0/path", json!("/abs.json")),
        "absolute output path"
    );
    assert!(
        broken("/limits/output_bytes", json!(1)),
        "outputs over budget"
    );
}

#[test]
fn tool_catalog_uses_exported_contracts() {
    let exported: BTreeSet<String> = ignis_core::schema::all()
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    let mut names = BTreeMap::new();
    for t in tool_catalog() {
        assert!(
            t.name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase())
                && t.name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "{}",
            t.name
        );
        assert!(
            names.insert(t.name.clone(), ()).is_none(),
            "duplicate tool {}",
            t.name
        );
        assert!(
            exported.contains(&t.input_contract) && exported.contains(&t.output_contract),
            "{}: unexported contract",
            t.name
        );
    }
}

#[test]
fn k1_knowledge_contracts_keep_identity_gaps_and_versions_honest() {
    // A v2 addition under the v1 tag is refused; the v1 document itself still reads.
    let mut req = fixture("evidence/search_request.json");
    typed::<SearchRequest>(&req).validate().unwrap();
    req["response_format"] = json!("concise");
    assert!(typed::<SearchRequest>(&req).validate().is_err());

    let page = fixture("evidence/graph_neighbors_page.json");
    let link = |i: usize| page["neighbors"][i]["link"].clone();
    let mut v1 = link(0);
    v1["schema_version"] = json!("evidence.v1");
    assert!(
        typed::<ProvenanceLink>(&v1).validate().is_err(),
        "uses_fuel is v2"
    );

    // Similarity never becomes identity.
    let mut mention = link(2);
    mention["resolution"] = json!("resolved");
    assert!(typed::<ProvenanceLink>(&mention).validate().is_err());
    let mut candidate = link(0);
    candidate["relation"] = json!("same_as_candidate");
    assert!(typed::<ProvenanceLink>(&candidate).validate().is_err());
    let mut wrong_end = link(0);
    wrong_end["to"] = json!({"concept": "hardware", "id": "x"});
    assert!(typed::<ProvenanceLink>(&wrong_end).validate().is_err());

    // A model label is resolved only by two agreeing passes.
    let mut label = fixture("evidence/source_labels.json")[0].clone();
    label["agreement"]["passes"] = json!(1);
    assert!(typed::<SourceLabel>(&label).validate().is_err());

    // Every row has one cell per key; numeric values keep SI units and the original's locator.
    let table = fixture("data/conditions_table.json");
    let mut gap = table.clone();
    gap["rows"][0]["cells"].as_array_mut().unwrap().remove(2);
    assert!(
        typed::<ConditionsTable>(&gap).validate().is_err(),
        "absent cell"
    );
    let mut unit = table.clone();
    unit["rows"][0]["cells"][1]["value"]["unit_si"] = json!("psi");
    assert!(typed::<ConditionsTable>(&unit).validate().is_err());
    let mut unlocated = table.clone();
    unlocated["rows"][0]["cells"][0]["value"]["locator"] = Value::Null;
    assert!(typed::<ConditionsTable>(&unlocated).validate().is_err());

    // Wiki deltas: a new page only adds; one item per op.
    let mut create = fixture("evidence/wiki_patch.json");
    create.as_object_mut().unwrap().remove("base_revision");
    create["title"] = json!("t");
    assert!(typed::<WikiPatch>(&create).validate().is_err());
    let mut twice = fixture("evidence/wiki_patch.json");
    let first = twice["ops"][0].clone();
    twice["ops"].as_array_mut().unwrap().push(first);
    assert!(typed::<WikiPatch>(&twice).validate().is_err());

    // A web excerpt is never promoted; only http(s) is fetched.
    let mut excerpt = fixture("evidence/web_search_page.json")["candidates"][0].clone();
    excerpt["promoted_source_id"] = json!("fixture-source-web-v1");
    assert!(typed::<ExternalSourceCandidate>(&excerpt)
        .validate()
        .is_err());
    let fetch: WebFetchRequest =
        typed(&json!({"schema_version": "evidence.v2", "url": "file:///etc/passwd"}));
    assert!(fetch.validate().is_err());

    // A section sits inside its parent's pages; a link out streams nothing.
    let mut outline = fixture("evidence/outline.json");
    outline["nodes"][1]["children"][1]["page_end"] = json!(41);
    assert!(typed::<SourceOutline>(&outline).validate().is_err());
    let mut link_out = fixture("evidence/source_access_link_out.json");
    link_out["range"] = json!(true);
    assert!(typed::<SourceAccess>(&link_out).validate().is_err());
}

#[test]
fn k1_research_contracts_bind_views_and_keep_versions_honest() {
    // v1 documents still read; a v2 addition under the v1 tag is refused.
    let v1_log: Vec<ResearchEvent> = typed(&fixture("research/events.json"));
    check_event_log(&v1_log).unwrap();
    let mut rationale = fixture("research/events_v2.json")[2].clone();
    typed::<ResearchEvent>(&rationale).validate().unwrap();
    rationale["schema_version"] = json!("research.v1");
    assert!(typed::<ResearchEvent>(&rationale).validate().is_err());
    let mut episode = fixture("research/episode_v2.json");
    episode["schema_version"] = json!("research.v1");
    assert!(typed::<ResearchEpisode>(&episode).validate().is_err());
    let mut skills = fixture("research/episode.json");
    skills["pins"]["skills_version"] = json!("x");
    assert!(
        serde_json::from_value::<ResearchEpisode>(skills).is_err(),
        "skills_version is replaced by harness"
    );

    // A view shows only what the answer cites, and a series keeps prediction and observation apart.
    let base = fixture("research/answer_v2.json");
    typed::<ResearchAnswer>(&base).validate().unwrap();
    let refused = |f: &dyn Fn(&mut Value)| {
        let mut v = base.clone();
        f(&mut v);
        typed::<ResearchAnswer>(&v).validate().is_err()
    };
    assert!(refused(&|v| v["schema_version"] = json!("research.v1")));
    assert!(
        refused(&|v| v["blocks"][2]["view"]["series"][0]["role"] = json!("predicted")),
        "a predicted series read from a non-model artifact"
    );
    assert!(
        refused(&|v| v["blocks"][2]["view"]["series"][1]["role"] = json!("observed")),
        "a model output plotted as an observation"
    );
    assert!(
        refused(&|v| v["blocks"][1]["view"]["evidence_id"] = json!("not-cited")),
        "an excerpt of evidence the answer does not list"
    );
    assert!(
        refused(&|v| v["blocks"][3]["view"]["table"] = json!("fixture-artifact-observed-01")),
        "a conditions matrix over a non-ConditionsTable artifact"
    );
    assert!(
        refused(&|v| v["model_releases"] = json!([])),
        "a release card and a prediction from an unlisted release"
    );
    assert!(refused(
        &|v| v["blocks"][2]["view"]["series"][0]["unit_pointer"] = json!("unit")
    ));
    assert!(
        refused(&|v| v["claims"][0]["checked_by"]["model_id"] = Value::Null),
        "a model check names its model"
    );

    // The submission names ids only; the agent cannot check its own claims.
    let sub: AnswerSubmission = typed(&fixture("research/answer_submission.json"));
    assert!(sub.problems().is_empty());
    assert!(sub.cited_evidence().len() == 2 && sub.cited_artifacts().len() == 3);
    let mut own_check = fixture("research/answer_submission.json");
    own_check["claims"][0]["checked_by"] = json!({"method": "human", "verdict": "supported"});
    let mut no_limit = fixture("research/answer_submission.json");
    no_limit["limitations"] = json!([]);
    let mut bad = fixture("research/answer_submission.json");
    bad["claims"][1]["support"] = json!([{"kind": "artifact_value", "artifact_id": "fixture-artifact-forecast-01", "pointer": "/mean/3"}]);
    bad["blocks"][0]["claim_ids"] = json!(["no-such-claim"]);
    for v in [own_check, no_limit] {
        assert!(typed::<AnswerSubmission>(&v).validate().is_err());
    }
    let kinds: BTreeSet<AnswerProblemKind> = typed::<AnswerSubmission>(&bad)
        .problems()
        .into_iter()
        .map(|p| p.kind)
        .collect();
    assert_eq!(
        kinds,
        BTreeSet::from([
            AnswerProblemKind::KindSupportMismatch,
            AnswerProblemKind::UnknownClaim
        ]),
        "every problem is reported at once"
    );

    // Tool arguments are redacted by pointer; traces stay inside their range and never guess cost.
    let mut args = fixture("research/events_v2.json")[3].clone();
    args["body"]["payload"]["arguments"]["redacted"] = json!(["filters"]);
    assert!(typed::<ResearchEvent>(&args).validate().is_err());
    let trace = fixture("research/trace_bundle.json");
    let trace_refused = |f: &dyn Fn(&mut Value)| {
        let mut v = trace.clone();
        f(&mut v);
        typed::<TraceBundle>(&v).validate().is_err()
    };
    assert!(trace_refused(
        &|v| v["tool_calls"][0]["finished_seq"] = json!(11)
    ));
    assert!(trace_refused(&|v| v["cost"]["price_list"] = Value::Null));
    assert!(trace_refused(
        &|v| v["partitions_touched"] = json!(["train", "train"])
    ));

    // Capabilities give a reason exactly for what is unavailable.
    let mut caps = fixture("research/capabilities.json");
    caps["tools"][0]["reason"] = json!("x");
    assert!(typed::<Capabilities>(&caps).validate().is_err());

    // The new catalog entries name exported contracts (also checked for every tool above).
    let names: BTreeSet<String> = tool_catalog().into_iter().map(|t| t.name).collect();
    for n in [
        "answer_submit",
        "sources_outline",
        "conditions_query",
        "graph_neighbors",
        "wiki_read",
        "web_search",
        "web_fetch",
    ] {
        assert!(names.contains(n), "{n}");
    }
}

/// A source gone upstream is a corpus-integrity problem: no retry policy may promise it comes back.
#[test]
fn source_gone_is_never_retryable() {
    let gone = |retry| ErrorEnvelope::new(ErrorCode::SourceGone, "gone", retry);
    gone(RetryPolicy::Never).validate().unwrap();
    for retry in [
        RetryPolicy::AfterDelay { seconds: 60 },
        RetryPolicy::AfterUserAction,
        RetryPolicy::Reconcile,
    ] {
        assert!(gone(retry).validate().is_err());
    }
    // Other codes keep whatever policy their producer chose.
    ErrorEnvelope::new(
        ErrorCode::UpstreamUnavailable,
        "outage",
        RetryPolicy::AfterDelay { seconds: 30 },
    )
    .validate()
    .unwrap();
}

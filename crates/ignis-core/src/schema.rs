//! JSON Schema export of the shared contracts (`ignis schemas export`).

use std::path::Path;

use schemars::{schema_for, JsonSchema};
use serde_json::json;

use crate::contracts::*;

fn one<T: JsonSchema>(name: &str) -> (String, serde_json::Value) {
    let mut v = serde_json::to_value(schema_for!(T)).expect("schema serializes");
    v["$id"] = json!(format!("ignis/{SCHEMA_VERSION}/{name}"));
    (name.to_string(), v)
}

/// Every exported contract as (type name, JSON Schema).
pub fn all() -> Vec<(String, serde_json::Value)> {
    vec![
        one::<Source>("Source"),
        one::<Investigation>("Investigation"),
        one::<Campaign>("Campaign"),
        one::<Session>("Session"),
        one::<Specimen>("Specimen"),
        one::<ExperimentRecord>("ExperimentRecord"),
        one::<ControlHistory>("ControlHistory"),
        one::<Observation>("Observation"),
        one::<ContourObservation>("ContourObservation"),
        one::<Calibration>("Calibration"),
        one::<EndpointRecord>("EndpointRecord"),
        one::<Cohort>("Cohort"),
        one::<Split>("Split"),
        one::<PrefixBundle>("PrefixBundle"),
        one::<ModelRelease>("ModelRelease"),
        one::<AnalysisArtifact>("AnalysisArtifact"),
        one::<AcquisitionAction>("AcquisitionAction"),
        one::<DecisionTrace>("DecisionTrace"),
        one::<SealedPrediction>("SealedPrediction"),
        // C0: common
        one::<ExecutionScope>("ExecutionScope"),
        one::<ArtifactRef>("ArtifactRef"),
        one::<ArtifactReadRequest>("ArtifactReadRequest"),
        one::<InlinePayload>("InlinePayload"),
        one::<ErrorEnvelope>("ErrorEnvelope"),
        // C0: research (research.v1, tools.v1; read alongside research.v2)
        one::<ResearchEpisode>("ResearchEpisode"),
        one::<ResearchEvent>("ResearchEvent"),
        one::<JobRef>("JobRef"),
        one::<ToolResult>("ToolResult"),
        one::<ToolDescriptor>("ToolDescriptor"),
        one::<ResearchAnswer>("ResearchAnswer"),
        one::<DatasetRef>("DatasetRef"),
        one::<ReleaseRef>("ReleaseRef"),
        one::<JobLookup>("JobLookup"),
        one::<AnalysisExecuteRequest>("AnalysisExecuteRequest"),
        one::<ExecutionRecord>("ExecutionRecord"),
        // K1: research.v2
        one::<AnswerSubmission>("AnswerSubmission"),
        one::<AnswerRefusal>("AnswerRefusal"),
        one::<TraceBundle>("TraceBundle"),
        one::<CreateEpisode>("CreateEpisode"),
        one::<PostMessage>("PostMessage"),
        one::<MessageAccepted>("MessageAccepted"),
        one::<ResolveApproval>("ResolveApproval"),
        one::<ApprovalAccepted>("ApprovalAccepted"),
        one::<EpisodeAction>("EpisodeAction"),
        one::<Capabilities>("Capabilities"),
        // evidence (C0 evidence.v1, K1 evidence.v2)
        one::<EvidenceRef>("EvidenceRef"),
        one::<SearchRequest>("SearchRequest"),
        one::<SearchPage>("SearchPage"),
        one::<ReadRequest>("ReadRequest"),
        one::<ReadResult>("ReadResult"),
        one::<OutlineRequest>("OutlineRequest"),
        one::<SourceOutline>("SourceOutline"),
        one::<SourceAccess>("SourceAccess"),
        one::<ProvenanceLink>("ProvenanceLink"),
        one::<GraphNeighborsRequest>("GraphNeighborsRequest"),
        one::<GraphNeighborsPage>("GraphNeighborsPage"),
        one::<SourceLabel>("SourceLabel"),
        one::<LabelSet>("LabelSet"),
        one::<WikiPatch>("WikiPatch"),
        one::<WikiRevision>("WikiRevision"),
        one::<WikiReadRequest>("WikiReadRequest"),
        one::<WikiReadResult>("WikiReadResult"),
        one::<WebSearchRequest>("WebSearchRequest"),
        one::<WebSearchPage>("WebSearchPage"),
        one::<WebFetchRequest>("WebFetchRequest"),
        one::<ExternalSourceCandidate>("ExternalSourceCandidate"),
        // dataset (C0 dataset.v1, K1 dataset.v2)
        one::<DatasetBundle>("DatasetBundle"),
        one::<VisualObservationBundle>("VisualObservationBundle"),
        one::<ImageContourObservation>("ImageContourObservation"),
        one::<ConditionsQuery>("ConditionsQuery"),
        one::<ConditionsTable>("ConditionsTable"),
        // C0: wsp (wsp.v1)
        one::<ModelSpec>("ModelSpec"),
        one::<HandwrittenWeightPatch>("HandwrittenWeightPatch"),
        one::<ModelBuildManifest>("ModelBuildManifest"),
        one::<ForecastRequest>("ForecastRequest"),
        // C0: evaluation (evaluation.v1)
        one::<EvaluationRequest>("EvaluationRequest"),
        one::<EvaluationResult>("EvaluationResult"),
        // K1: evaluation (evaluation.v2)
        one::<GoldTask>("GoldTask"),
        one::<GoldSet>("GoldSet"),
        one::<GoldPromptSet>("GoldPromptSet"),
    ]
}

/// Writes every schema and `index.json`, and removes `*.json` schemas no longer exported (stale files
/// would otherwise keep feeding the TypeScript generator).
pub fn export(dir: &Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(dir)?;
    let schemas = all();
    for e in std::fs::read_dir(dir)?.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if let Some(stem) = name.strip_suffix(".json") {
            let ours =
                std::fs::read_to_string(e.path()).is_ok_and(|t| t.contains("\"$id\": \"ignis/"));
            if ours && stem != "index" && !schemas.iter().any(|(n, _)| n == stem) {
                std::fs::remove_file(e.path())?;
            }
        }
    }
    let mut names = Vec::new();
    for (name, s) in &schemas {
        let mut text = serde_json::to_string_pretty(&crate::hashing::sorted(s)).unwrap();
        text.push('\n');
        std::fs::write(dir.join(format!("{name}.json")), text)?;
        names.push(name.clone());
    }
    let index = json!({"schema_version": SCHEMA_VERSION, "types": names});
    std::fs::write(
        dir.join("index.json"),
        serde_json::to_string_pretty(&index).unwrap() + "\n",
    )?;
    Ok(schemas.len())
}

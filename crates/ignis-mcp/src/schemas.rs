//! Input schemas of the tools, generated from the contract types the catalog names. Subschemas are
//! inlined (no `$ref`/`$defs`): agent runtimes flatten MCP schemas into provider function
//! definitions, and a dangling reference would reach the model as an untyped field.

use ignis_core::contracts::{
    AnalysisExecuteRequest, AnswerSubmission, ArtifactReadRequest, ConditionsQuery, DatasetRef,
    ForecastRequest, GraphNeighborsRequest, JobLookup, OutlineRequest, ReadRequest, ReleaseRef,
    SearchRequest, WebFetchRequest, WebSearchRequest, WikiPatch, WikiReadRequest,
};
use schemars::generate::SchemaSettings;
use schemars::JsonSchema;
use serde_json::Value;

fn inline<T: JsonSchema>() -> Value {
    let mut schema = SchemaSettings::draft2020_12()
        .with(|s| s.inline_subschemas = true)
        .into_generator()
        .into_root_schema_for::<T>();
    schema.remove("$schema");
    schema.to_value()
}

/// The JSON Schema of an input contract named in [`ignis_core::contracts::tool_catalog`].
pub fn input_schema(contract: &str) -> Option<Value> {
    Some(match contract {
        "SearchRequest" => inline::<SearchRequest>(),
        "ReadRequest" => inline::<ReadRequest>(),
        "DatasetRef" => inline::<DatasetRef>(),
        "ReleaseRef" => inline::<ReleaseRef>(),
        "ForecastRequest" => inline::<ForecastRequest>(),
        "ArtifactReadRequest" => inline::<ArtifactReadRequest>(),
        "JobLookup" => inline::<JobLookup>(),
        "AnalysisExecuteRequest" => inline::<AnalysisExecuteRequest>(),
        "WikiPatch" => inline::<WikiPatch>(),
        "AnswerSubmission" => inline::<AnswerSubmission>(),
        "OutlineRequest" => inline::<OutlineRequest>(),
        "ConditionsQuery" => inline::<ConditionsQuery>(),
        "GraphNeighborsRequest" => inline::<GraphNeighborsRequest>(),
        "WikiReadRequest" => inline::<WikiReadRequest>(),
        "WebSearchRequest" => inline::<WebSearchRequest>(),
        "WebFetchRequest" => inline::<WebFetchRequest>(),
        _ => return None,
    })
}

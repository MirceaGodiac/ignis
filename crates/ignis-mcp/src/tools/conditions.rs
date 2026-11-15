//! The result of `conditions_query` for the model: the table is always stored as a
//! `ConditionsTable` artifact, so every cell (and each row's `nearness`) is readable and citable
//! later, and the call returns the artifact's id and digest and the conditions snapshot it was
//! answered under, with the table inline when it is small or a preview of it otherwise.

use ignis_core::contracts::{
    ConditionsQuery, ConditionsTable, Digest, EntityRef, EpisodeId, ErrorCode, ExecutionScope,
    InlinePayload, Producer, ToolCallId,
};
use ignis_core::services::{ArtifactDraft, ServiceResult};
use serde_json::{json, Value};

use super::{envelope, preview_of, ToolHost, INLINE_RESULT_BYTES, PREVIEW_BYTES};

const HOW_TO_CITE: &str = "cite a run's condition (oxygen_mole_fraction, pressure, …) on an observed claim as a value whose source is {\"kind\": \"conditions_cell\", \"snapshot\": <conditions_snapshot>, \"investigation_id\", \"run_id\", \"key\"} (add `locator` to pick a candidate of a conflicting cell, `reading: raw` for the number as the source wrote it); any other cell of this table is an artifact_cell of artifact_id at sha256 (rows: {\"table\": \"/rows\", \"where\": [{\"field\": \"/run/run_id\", \"equals\": …}]}, then {\"table\": \"/cells\", \"where\": [{\"field\": \"/key\", \"equals\": …}]}, column \"/value\")";

impl ToolHost {
    pub(super) fn conditions_result(
        &self,
        scope: &ExecutionScope,
        episode: Option<&EpisodeId>,
        call_id: &ToolCallId,
        q: &ConditionsQuery,
    ) -> ServiceResult<InlinePayload> {
        let ds = self.datasets()?;
        // The snapshot is read before and after the query, so a table that changed while it was
        // answered is never bound to either version. A service without snapshots still answers;
        // its cells are then citable through the artifact only.
        let before = ds.conditions_snapshot(scope);
        let table: ConditionsTable = ds.conditions(scope, q)?;
        let snapshot: Result<Digest, String> = match (before, ds.conditions_snapshot(scope)) {
            (Ok(a), Ok(b)) if a == b => Ok(b),
            (Ok(a), Ok(b)) => {
                return Err(envelope(
                    ErrorCode::Conflict,
                    format!(
                        "the conditions changed while they were read ({a} then {b}); call again"
                    ),
                ))
            }
            (Err(e), _) | (_, Err(e)) => Err(e.message),
        };
        let bytes =
            serde_json::to_vec(&table).map_err(|e| envelope(ErrorCode::Internal, e.to_string()))?;
        let artifact = self.artifacts.put(
            scope,
            ArtifactDraft {
                media_type: "application/json".into(),
                contract: Some("ConditionsTable".into()),
                produced_by: Producer {
                    episode_id: episode.cloned(),
                    job_id: None,
                    tool_call_id: Some(call_id.clone()),
                    release_id: None,
                },
                inputs: Vec::<EntityRef>::new(),
                origin: self.origin,
            },
            &mut bytes.as_slice(),
        )?;
        let mut value = json!({
            "artifact_id": artifact.artifact_id,
            "sha256": artifact.sha256,
            "view_fields": view_fields(&table),
        });
        match snapshot {
            Ok(s) => {
                value["conditions_snapshot"] = json!(s);
                value["how_to_cite"] = json!(HOW_TO_CITE);
            }
            Err(why) => {
                value["conditions_snapshot"] = Value::Null;
                value["how_to_cite"] = json!(format!(
                    "this deployment reports no conditions snapshot ({why}), so conditions_cell values cannot be bound; cite cells as artifact_cell of artifact_id at sha256 (rows: {{\"table\": \"/rows\", \"where\": [{{\"field\": \"/run/run_id\", \"equals\": …}}]}}, then {{\"table\": \"/cells\", \"where\": [{{\"field\": \"/key\", \"equals\": …}}]}}, column \"/value\")"
                ));
            }
        }
        let full = serde_json::to_value(&table)
            .map_err(|e| envelope(ErrorCode::Internal, e.to_string()))?;
        if bytes.len() + 4096 <= INLINE_RESULT_BYTES {
            value["table"] = full;
            return Ok(InlinePayload {
                value,
                partial: false,
                full: Some(artifact),
            });
        }
        let (head, cut) = preview_of(&full, PREVIEW_BYTES);
        value["total_bytes"] = json!(bytes.len());
        value["preview"] = head;
        value["cut"] = cut;
        value["how_to_read"] = json!("the preview holds every small field whole and the first rows (`cut` says how many of how many); read the rest with artifacts_read on artifact_id and a json_pointer (e.g. \"/rows/40\"), or narrow the call (keys, filters, near, a page_size) and call again");
        Ok(InlinePayload {
            value,
            partial: true,
            full: Some(artifact),
        })
    }
}

/// Row fields of this table for an answer view (`chart` x/y/group/label over `/rows`): a cell's
/// position in `cells` is its key's position in `keys`, so each numeric key's SI value is
/// `/cells/<i>/value/value_si` (absent in a row whose cell is a gap, which a chart leaves out).
fn view_fields(table: &ConditionsTable) -> Value {
    let mut f = serde_json::Map::new();
    f.insert("rows".into(), json!("/rows"));
    f.insert("run_id".into(), json!("/run/run_id"));
    f.insert("investigation_id".into(), json!("/investigation_id"));
    for (i, k) in table.keys.iter().enumerate() {
        let Some(name) = serde_json::to_value(k)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
        else {
            continue;
        };
        let field = match k.quantity() {
            Some(q) => json!({"field": format!("/cells/{i}/value/value_si"), "unit": q.unit_si()}),
            None => json!({"field": format!("/cells/{i}/value/raw_value")}),
        };
        f.insert(name, field);
    }
    if table.rows.iter().any(|r| r.nearness.is_some()) {
        f.insert("nearness_distance".into(), json!("/nearness/distance"));
        f.insert("oxygen_gap".into(), json!("/nearness/oxygen_gap"));
        f.insert("pressure_gap_pa".into(), json!("/nearness/pressure_gap_pa"));
    }
    Value::Object(f)
}

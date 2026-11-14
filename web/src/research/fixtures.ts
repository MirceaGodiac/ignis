// The C0 contract fixtures (fixtures/contracts/, owned by each family's lane), imported as data for
// mock mode. This is the ONLY place they are cast to contract types: JSON imports widen string
// literals. The fixtures are checked against the Rust contracts by `cargo test -p ignis-core --test
// contracts` (fixtures/contracts/manifest.json `checked_by`), not by this cast. Never edit fixtures from web/.

import type {
  ArtifactRef,
  DatasetBundle,
  EvaluationResult,
  ExecutionRecord,
  ModelRelease,
  ModelSpec,
  ReadResult,
  ResearchAnswer,
  ResearchEpisode,
  ResearchEvent,
  SearchPage,
  ToolResult,
  VisualObservationBundle,
} from "../contracts/generated";
import episode from "../../../fixtures/contracts/research/episode.json";
import events from "../../../fixtures/contracts/research/events.json";
import answer from "../../../fixtures/contracts/research/answer.json";
import toolAccepted from "../../../fixtures/contracts/research/tool_accepted.json";
import toolFailed from "../../../fixtures/contracts/research/tool_failed.json";
import toolInsufficient from "../../../fixtures/contracts/research/tool_insufficient_evidence.json";
import searchPage from "../../../fixtures/contracts/evidence/search_page.json";
import readResult from "../../../fixtures/contracts/evidence/read_result.json";
import releaseW0 from "../../../fixtures/contracts/models/release_w0.json";
import modelSpec from "../../../fixtures/contracts/models/model_spec.json";
import datasetBundle from "../../../fixtures/contracts/data/dataset_bundle.json";
import visualBundle from "../../../fixtures/contracts/data/visual_bundle_pixel_only.json";
import executionRecord from "../../../fixtures/contracts/research/execution_record.json";
import evaluationResult from "../../../fixtures/contracts/evaluation/result_dev.json";
import manifest from "../../../fixtures/contracts/manifest.json";

const as = <T>(v: unknown) => v as T;

export const FIXTURES = {
  notice: manifest.notice,
  episode: as<ResearchEpisode>(episode),
  events: as<ResearchEvent[]>(events),
  answer: as<ResearchAnswer>(answer),
  /** ArtifactRef id under which `answer.completed` stores the ResearchAnswer in events.json. */
  answerArtifactId: "fixture-artifact-answer-01",
  tools: {
    accepted: as<ToolResult>(toolAccepted),
    failed: as<ToolResult>(toolFailed),
    insufficient: as<ToolResult>(toolInsufficient),
  },
  searchPage: as<SearchPage>(searchPage),
  readResult: as<ReadResult>(readResult),
  releases: [as<ModelRelease>(releaseW0)],
  modelSpec: as<ModelSpec>(modelSpec),
  datasetBundle: as<DatasetBundle>(datasetBundle),
  visualBundle: as<VisualObservationBundle>(visualBundle),
  /** Mock-mode artifact refs for the typed viewers (T7). The refs are made here; the payloads are the C0 fixtures. */
  typedArtifacts: [
    typedRef("fixture-artifact-exec-record", "ExecutionRecord", as<ExecutionRecord>(executionRecord)),
    typedRef("fixture-artifact-dataset-bundle", "DatasetBundle", as<DatasetBundle>(datasetBundle)),
    typedRef("fixture-artifact-eval-result", "EvaluationResult", as<EvaluationResult>(evaluationResult)),
  ],
};

function typedRef(artifact_id: string, contract: string, payload: { created_at?: string; finished_at?: string }): { ref: ArtifactRef; payload: unknown } {
  const body = JSON.stringify(payload);
  return {
    ref: {
      artifact_id,
      contract,
      media_type: "application/json",
      byte_length: body.length,
      sha256: `sha256:${"0".repeat(64)}`,
      origin: "synthetic_fixture",
      created_at: payload.created_at ?? payload.finished_at ?? "2026-11-12T18:00:00Z",
    },
    payload,
  };
}

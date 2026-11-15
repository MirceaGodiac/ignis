# Research harness ↔ UI contract

Status of `main` at 531de71 (2026-11-14), read from `crates/ignis-core/src/contracts/research.rs`,
`crates/ignis-server/src/v1/mod.rs` and `web/src/research/*`. This file says what the harness gives
the UI today, how the UI uses it, and what the UI needs next. It changes no contract: proposals in
§4 go to the contract owner (Mircea, `research.rs`; Raul, `evidence.rs`) as normal contract PRs.

## 1. What the harness does today

- **Runtime.** Codex App Server owns the conversation and the model/tool loop. IGNIS maps one
  `ResearchEpisode` to one runtime thread and keeps its own append-only `ResearchEvent` log.
- **Tools (C0 catalog).** `sources_search`, `sources_read`, `datasets_inspect`, `models_inspect`,
  `models_forecast` (may return a job), `artifacts_read`, `jobs_get`, `wiki_patch`. Every call
  returns a `ToolResult` with two independent axes: `execution_status` (accepted, completed,
  refused, failed, cancelled) and `scientific_status` (not assessed, conclusive, inconclusive,
  insufficient evidence, out of domain). Large outputs are `ArtifactRef`s, never inline.
- **Routes (`/api/v1`, fixtures at `/api/fixtures/v1`).** capabilities; episodes (create, list,
  get, messages, approvals, pause/resume/cancel); `GET /episodes/{id}/events` (resumable SSE,
  `Last-Event-ID` or `?after=`); artifacts (bytes, JSON Pointer, meta); search; source pages;
  models; analyses (forecast as job); jobs; uploads.
- **Answer.** `ResearchAnswer` = text blocks bound to claim ids, artifact blocks, typed claims
  (`source_reports`, `observed`, `computed`, `predicted`, `interpretation`), evidence refs,
  artifacts, model releases used, limitations. `validate()` enforces that each claim kind has the
  support it needs (an `observed` claim cannot cite a model; a `predicted` claim must point into
  an `AnalysisArtifact`).
- **Evidence.** `EvidenceRef` points into one source version: `SourceLocator` (page + page label +
  bbox, or file path + row + column, or video frame + PTS, or metadata only), evidence class,
  optional exact `quote`.

## 2. How the UI consumes it

| Contract | UI surface |
|---|---|
| `ResearchEvent` log (seq, dedup, 410 resync) | `research/eventLog.ts` + `stream.ts` (unchanged, tested) → thread, route, status |
| `tool.started` / `tool.finished` / `job.updated` / `artifact.created` / `approval.*` | **Research route**: one stop per tool call, job and approval, under each answer; a stop opens in the side panel |
| `ToolResult` two axes | Each stop shows execution and science status separately; `insufficient_evidence` is a result, not an error |
| `ResearchAnswer` blocks + claims | Answer set as text with **margin keys**; each key = one claim, glyph = claim kind |
| `ClaimSupport::Evidence` | Key → hover card (quote, locator) → side panel (page text, bbox, row/column) |
| `ClaimSupport::ArtifactValue` | Value read by JSON Pointer, never transcribed; unit read from `/unit` |
| `AnswerBlock::Artifact` | Numbered figure with caption; opens in the side panel |
| `limitations` | "Limits that change the interpretation" under every answer |
| `Origin::SyntheticFixture` | Hatched synthetic marking on the item and in the shell |
| Capabilities | Service status in the sidebar footer; disabled actions say which service is missing |

## 3. Gaps the UI shows instead of inventing

1. **Tool arguments.** `tool.started` carries only `call_id` and `tool`. The route can say "Search
   sources" but not *what* was searched. Shown as: "arguments not recorded by the harness".
2. **Model rationale ("why").** No event or field carries it. The side panel has a "Why this step"
   section that says the rationale is not yet in the contract.
3. **Quote may be absent.** `EvidenceRef.quote` is optional; the UI then reads the page
   (`sources_read`) and slices the span by `char_start/char_end/offset_unit`, or says no quote is
   carried.
4. **Run links across origins.** Artifact inputs name runs (`EntityRef`); a run absent from the
   current data source opens a "not in this data source" state.

## 4. Proposed additions (for the contract owners)

| # | Proposal | Where | Why |
|---|---|---|---|
| P1 | `tool.started { call_id, tool, arguments: ArtifactRef \| InlinePayload }` (redacted per scope) | `research.rs` `EventBody::ToolStarted` | Route shows the query, page range or run that a step used |
| P2 | `step.rationale { call_id?, text, source: "model_explanation" }`, asked after the step, never chain-of-thought | `research.rs` new `EventBody` | "Why" card; labelled as the model's own explanation, may be unfaithful |
| P3 | `Claim.confidence?` + `Claim.checked_by?` (relevance/support judge result, e.g. Jev) | `research.rs` `Claim` | Shows which citations passed the support check |
| P4 | `SourceLabel { family, doc_type, investigation, runs[], conditions[], method: semantic, model_id, confidence }` | `evidence.rs` | Classifier output (e.g. Muse Spark) as typed, reviewable labels |
| P5 | Entity graph read (`graph_neighbors(entity) → EntityRef[] + relation`) | new tool, `evidence.rs` | PPR / multi-hop retrieval and "related" navigation in the side panel |
| P6 | Move `CreateEpisode`, `PostMessage`, `Capabilities`, `ToolStatus` into `ignis-core` | `research.rs` | **Done:** K1.7 (#31) moved them into `ignis-core`; the UI reads them from `generated.ts` and `serverTypes.ts` is deleted (T6) |
| P7 | Original bytes of a source, without IGNIS storing them | **Planned:** `LANES_V2.md` M12 (`GET /api/v1/sources/{id}/content`) returning K1.18 `SourceAccess` (`direct` / `proxy` / `link_out`) | T7 PDF.js at the `SourceLocator` bbox and BASS video stream PSI originals from NASA S3 and NTRS through the Worker proxy. Until M12 lands, the viewer shows parsed page text with a CSS box |

T7 contract coverage (which view shows each contract, or why none does) is checked in code:
`web/src/research/artifactViews.ts` `CONTRACT_COVERAGE`, tested by `web/tests/artifacts.test.ts`.

## 5. Navigation model the UI implements

- **Sidebar** (Slack): investigations (episodes, with status) above a library (atmosphere map,
  runs, compare, forecast lab, sources, models); one data-source switch; service status at the
  foot.
- **Centre** (Linear): one view at a time, addressable by URL hash (`#/runs`, `#/lab/<run>`,
  `#/research/<episode>` …) so every state can be linked and reloaded.
- **Side panel** (Linear peek / Slack thread): the selected run, claim, source, artifact or step,
  with its own back stack, so a scientist can follow claim → evidence → source → run and return.
- **Command menu** (`Ctrl/⌘ K`): every view, run and investigation by name.

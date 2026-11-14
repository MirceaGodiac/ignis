# Phase 2 lanes: harness, knowledge, workspace and evaluation

Iancu's lane (FLEX/ACME/BASS, literal WSP, I1–I5) is unchanged and is not part of this split. The
remaining work goes to three parallel lanes:

| Lane | Owner | One line | Branch |
|---|---|---|---|
| **A. Harness & platform** | Mircea | what the agent is told, which tools it holds, how its answer is bound and checked, how its runs are traced and compared; plus the platform underneath | `feat/agent-backend` |
| **B. Knowledge & NASA data** | Raul | every PSI combustion investigation and NTRS document catalogued, parsed with page coordinates, classified, and joined in a typed combustion graph the agent can query | `feat/scientific-data` |
| **C. Research workspace & evaluation** | Tudor | the researcher's surfaces (viewers, charts, catalog-driven answer views, challenge views) and the independent benchmark that measures the harness | `feat/research-product` |

The split follows the existing ownership map, so no file changes owner. Lanes meet only through the
contracts in §5, which land first as fixtures; until a dependency ships, each lane works against those
fixtures (`/api/fixtures/v1`, `fixtures/contracts/`).

Revision (same day): storage rule from Iancu's atlas and stream-from-source access (§7, measured
against NASA's servers), section-tree retrieval and Jev in R7/R8, the three K1 contract PRs written in
parallel, `answer_submit` classified as `compute`.

---

## 1. Where we are (facts from `main`, b234f89)

**Platform.** Deployed on `ignis-ardealu.app` (Cloudflare Access + Worker + Tunnel + R2, GCE VM
`ignis-origin`, gVisor runner verified). `/api/v1` with resumable SSE, durable jobs, artifacts,
uploads, generated OpenAPI. Codex 0.169.0 bridge and a Claude Code bridge (ADR 17,
`--agent-runtime claude`, verified only against the fake binary). Google sign-in mode (ADR 18) is in
progress on `feat/google-auth`.

**Harness: the gaps.**
- No system prompt, developer instructions or skills reach either runtime. Codex `thread/start`
  sends cwd/sandbox/approval/model only (`ignis-research/src/runtime/bridge.rs` ~502–531); Claude
  receives only the user text (`runtime/claude.rs` ~749–770). The per-episode cwd is empty, so no
  AGENTS.md reaches the agent. `EpisodePins.skills_version` exists and is always `None`. The only
  agent-facing text is the MCP `INSTRUCTIONS` string (`ignis-mcp/src/http.rs` ~37–42) and the tool
  descriptions.
- **No live turn produces a `ResearchAnswer`.** The runtime emits `message.completed` (free text),
  tool events and approvals. `answer.completed` with typed claims exists only in the fixture server
  (`ignis-server/src/v1/fixture.rs`). The product promise (every claim bound to source, artifact or
  release) is therefore not yet produced by the live agent.
- `tool.started` carries no arguments; there is no rationale event; `wiki_patch` is always
  unavailable; there is no delegation, no web tool, no trace bundle, and Codex `multi_agent` is
  disabled in the public profile (`runtime/preflight.rs` ~86–124).
- Master §25 already contains drafts: investigator prompt (§25.1), weights author, literal revision,
  evidence reviewer, and eight skill bodies.

**Knowledge.** Active corpus = 6 indexed documents (FLEX TP, ACME, BASS, Saffire VI, BASS-II summary,
BASS-II SRD; snapshot `corpus-d0f2d2c5a4e1381c`). Manifests cover PSI-69/159/25/26 (674 PDFs listed,
not 674 papers) plus JAXA/ZARM/ESA/Mendeley candidates. Retrieval = FTS5 + exact dense (Qwen3-Embedding-8B)
+ RRF + Qwen3-Reranker-8B via DeepInfra, lexical-only without the key. PDF parsing via MuPDF. No
wiki, no `ProvenanceLink` producer, no graph, no document classification (the `ignis data review auto`
queue adjudicates run-level semantics, not documents).

**Workspace.** PR #16 shipped the shell (sidebar, centre view, side panel, command menu), Answer view
with margin claim keys, Research route, Atmosphere map, Runs, Compare, Lab, Sources, Models.
`web/HARNESS_CONTRACT.md` lists the UI's gaps and contract proposals P1–P6. Runs/Lab/Compare/Atmosphere
still read the legacy `/api/experiments*` routes through hand-written DTOs in `api.ts`. No PDF
renderer (text + CSS bbox box), no video playback, artifacts mostly as raw JSON, no renderer for
`EvaluationResult`, `ExecutionRecord`, `DatasetBundle`, `ModelSpec`, wiki, provenance.

**Evaluation.** Fixture evaluator, protocol freeze and scorecards are in. Missing: the benchmark
(gold tasks), the H0/H1 runner, WSP scoring (after I3).

**The challenge** ("Flame in Freefall", Space Apps 2026) asks for an SI-powered dashboard that finds,
summarizes, ranks and interprets combustion investigations for habitat fire safety. It names the
attributes to show (objectives, fuels and sample geometry, O2, airflow, pressure, how tests varied,
sensors, cross-investigation similarities) and the habitat atmospheres (Earth/ISS 21 % at 14.7 psi;
suit 100 % at 4.3 psi; Moon base and rovers ≥ 30 % at ≤ 10.2 psi; no partial-gravity data in PSI
yet). PSI lists 26 combustion items. **19 are flight investigations**: FLEX, FLEX-2, CFI, ACME CFI-G,
ACME Flame Design, ACME s-Flame, ACME BRE, ACME CLD, ACME E-FIELD, SPICE, SLICE, BASS, BASS-II,
Saffire I–III, SAME, SAME-R, DAFT/DAFT-2 (Iancu's atlas, §7). The rest are ground and modelling studies.
**Today the corpus holds 6 documents and the datasets of 4 investigations.** Judging (2025 guide;
2026 *unverified*): impact, creativity, validity, relevance, presentation.

---

## 2. Lane A — Mircea: Harness & platform

Goal: the live agent produces validated, bound `ResearchAnswer`s under a versioned, measurable
harness. Everything here is configuration of the runtime plus host-side checks. The runtime keeps the
loop (AGENTS.md; no second loop, no compactor, no gateway).

| ID | Deliverable | Depends on | Done when |
|---|---|---|---|
| **M6** | **Harness bundle v1.** New `harness/` directory: `investigator.md` (operational developer instructions trimmed from master §25.1 to the invariants: claim kinds, citation and locator duty, statuses, model output ≠ measurement, test isolation), `skills/<name>/SKILL.md` (agentskills.io format), `agents/*.md` (subagent definitions, M9). The tool section of the prompt is **generated from the registered tools** so it never names a missing tool. Injected into Codex (`thread/start` developer/base instructions, plus skills under the episode cwd `.agents/skills/`; confirm field names against the pinned 0.169.0 protocol schema the preflight already exports) and Claude Code (`--append-system-prompt`, `.claude/skills/`; check that `--tools ""` and `--setting-sources ""` still allow the Skill tool, and adjust the fixed args minimally). The bundle digest becomes `EpisodePins.harness` (K1) and is shown in `/capabilities`. | K1-research | a live turn in both runtimes shows the instructions took effect (skill invoked, tool section matches `/capabilities`); digest pinned on the episode |
| **M7** | **Bound answer path.** Tool `answer_submit` (`AnswerSubmission` → host validation → `ResearchAnswer` artifact + `answer.completed`). The host checks `ResearchAnswer::validate()` **and** that each `EvidenceRef` was returned by a tool result in this episode, each `ArtifactValue` pointer resolves with a unit, each release is pinned. A failure is a `refused` `ToolResult` with an actionable reason, so the agent can fix and resubmit. Effect class `compute`: the tool writes only this episode's answer, through host validation. It cannot be `write`, because the bridge refuses elicitations and the MCP layer fails closed on new `write` tools (M3); the classification is fixed in K1.3. Supports `AnswerBlock::View` (catalog co-designed with Tudor, K1). `message.completed` stays for free text; the UI treats `answer.completed` as the result. | K1-research, M6 | the end-to-end path 1 of `WORKBOARD.md` runs live: question → MCP search/read → computation → submitted answer → UI opens the cited page |
| **M8** | **Tool ergonomics.** P1 (`tool.started.arguments`, redacted by scope), P2 (`step.rationale`, labelled as the model's own explanation, never chain-of-thought), P6 (server-only types moved to `ignis-core`, deletes `web/src/research/serverTypes.ts` mirrors). `response_format: concise\|detailed` plus cursor pagination on search/read/artifacts (Raul implements the service side). Audit every tool description against the MCP description-smell rubric *(lit.)* and A/B test changes on the dev split. Register Raul's new tools (`conditions_query`, `graph_neighbors`, `wiki_read`, `web_search`/`web_fetch`) as they land, each behind its capability. | K1-research, K1-evidence | descriptions reviewed; new tools appear in `/capabilities` only when configured; UI shows arguments |
| **M9** | **Context in the environment, depth-1 delegation.** RLM's useful half *(lit.)*: large tables and corpora stay as `ArtifactRef` handles mounted into `analysis_execute` (a small `ignis` Python helper in the pinned runner image reads artifacts and `DatasetBundle` Parquet by ref), so the agent computes on them instead of loading them into context. Delegation only through the runtime's own subagents (Claude Code agents; Codex `multi_agent` once enabled), depth 1, with a derived child scope, a budget and a result contract (master §8.3). Open design point: a child that shares the process MCP token shares its scope; child scopes need either a per-child token or a host-side check. No hand-written recursive LLM loop (it would be a second agent loop). Simple lookups stay off the REPL path, since RLM-style recursion hurts simple retrieval *(lit.)*. | M6, runner (M3) | a multi-run aggregation question answered through the runner on handles; a delegated literature subtask returns a condensed, cited summary; ablation available to Tudor |
| **M10** | **Traces and harness experiments.** Each episode can export a `TraceBundle` (K1): harness digest, runtime build, model, effort, event range, tool I/O as artifacts, measured token usage and cost when the runtime reports them, partitions touched. `ignis harness run --variant <name> --tasks <dev set>` drives episodes for Tudor's runner (Tudor owns the protocol and scoring). Offline, dev-only harness search (GEPA or Meta-Harness style, *(lit.)*) is a stretch: the proposer reads traces from the filesystem and **refuses any trace that touched `test` or the private hold-out**. | K1-research, K1-evaluation, T10 | H0 and H1 variants run unattended on the dev set with complete traces |
| M5 | (existing) live integration, cancel/resume/reconciliation, observability; `ModelService` once I2 lands | R3, I2 | — |
| M4 | (remaining) restore drill with measured RPO/RTO; denied approval exercised live; merge `feat/google-auth` (ADR 18) | — | — |
| M11 | **v1 routes for the workspace**: datasets/runs/conditions endpoints over `DatasetService` and the new conditions service, so Tudor can retire the legacy `/api/experiments*` routes (T6) | K1-dataset, R9 | legacy DTOs deleted from `api.ts` |
| **M12** | **Source access without mirroring** (§7). `GET /api/v1/sources/{id}/content` returns a `SourceAccess` (K1.18): for PSI, a fresh signed S3 URL resolved through geode (`redirect=false`), cached for less than its ~1 h lifetime and re-resolved on 403; for NTRS, a URL on a Worker pass-through proxy (allow-listed host, `Range` forwarded, no storage); for the NASA Image and Video Library, the permanent asset URL; otherwise `link_out`. No NASA media bytes pass through or stay on the origin. Rate limits on the geode calls, which are undocumented endpoints. | K1-evidence, R6 | Tudor's viewer plays a PSI video and opens a PSI and an NTRS PDF with zero bytes stored on our side |

Skills (M6), with content reviewers. Start with the four that SkillsBench-style evidence favours
(focused, 2–3 modules each, hand-written, never auto-generated) *(lit.)*:
`source-grounding` (Raul reviews), `experimental-semantics` (Raul), `quantitative-analysis`
(Mircea/Tudor), `model-qualification` (Iancu). `research-memory` arrives with the wiki (R4).
`benchmark-integrity` belongs to the evaluator, not the investigator. Every skill gets a paired
with/without measurement inside H1 (T11); a skill that does not help is removed.

Mircea still has no presentation tasks.

---

## 3. Lane B — Raul: Knowledge & NASA data

Goal: cover the challenge's whole investigation set with openable originals, page coordinates,
typed labels and a combustion graph whose values keep raw value, unit, SI value and locator.

| ID | Deliverable | Depends on | Done when |
|---|---|---|---|
| **R6** | **Catalog and stored corpus for the 19 flight investigations, plus NTRS.** Start from Iancu's atlas (§7): store the 636 documents (1.63 GB) and the 25,293 sensor/settings text files (12.47 GB); keep the 40,336 image/video/archive files (13.47 TB) and 236 other archives as manifest entries only (name, size, version, date, stable geode URL), reached through M12. Store the hash of every fetched file; NASA versions and dates per file are kept. Licence per investigation from NASA metadata (CC0-1.0 for 18; **PSI-10 Flame Design has an empty licence field: verify before use**). The geode endpoints are undocumented: throttle, cache, save response samples (as `sources/psi.rs` does). NTRS via its documented API (500 requests per 15 min, 10 000 records per query) and bulk metadata; rights per document; NTRS `.txt` full text is used for indexing where present; deduplicate PSI presentations that reuse NTRS ids. Journal papers: metadata + DOI only unless open access. Record a `SourceAccess` mode per source (direct, proxy, link out). **Lockbox:** documents of FLEX (PSI-69), ACME CFI-G (PSI-159), BASS-II (PSI-25) and BASS (PSI-26) contain outcomes of held-out tests; `strip_for_sealed` must hold for every added document, and the same applies to the other 15 investigations once their evaluation groups are frozen. Users of the app see them normally. | — | manifests for all 19; ~14.1 GB stored with hashes; `CorpusSnapshotId` reproducible; coverage table per investigation; a test that an authoring scope receives no held-out passage from a newly added document |
| **R7** | **Parsing with coordinates, then a section tree.** Docling (MIT; page, bbox, char span) for body chunks, figures and tables; GROBID (Apache) for header metadata and references; MuPDF stays as a baseline. Producer rule: bboxes normalized to `[0,1]`, top-left origin, physical 1-based page (Docling is bottom-left; convert at the producer, never in the UI). Versioned parse views; old citations stay resolvable. olmOCR/Nougat are unsuitable (no bboxes) *(lit.)*. On top of the parse view, a **PageIndex-style section tree** *(lit.)*: the document's own heading hierarchy as nodes, each with page range, child nodes and a generated summary (summaries by a cheap text model, e.g. Muse Spark 1.3 Contributor; labelled as generated, never cited as the source). Exposed as `sources_outline` (K1.19) so the agent walks long reports (FLEX TP 314 pages, BASS-II summary 144) node by node and then reads the passage it needs. The agent is the navigator, so no extra LLM call per tree step. Hybrid search stays alongside for exact names and numbers; T10 measures both. | R6 | a passage, a table cell and a figure each open at the right place in Tudor's viewer; the outline of the FLEX TP is navigable |
| **R8** | **Document classification and extraction.** `SourceLabel` (K1-evidence): family, document type (closed enum), investigation, runs, conditions mentioned, method `semantic`, model id, prompt hash, confidence, two-pass agreement, status. Labels never establish identity (`resolution = unknown` stays unknown). **Jev** (TypeSafe's decision model: typed answers from a developer-defined answer space, with confidence; it does not write text) is the natural fit for the closed-enum labels; the same model can serve the claim support check (`Claim.checked_by`, K1.5) at answer time. Jev's prices and current version differ between third-party listings (*unverified*). Text generation (summaries, R7) uses a cheap generative model; the data is public (PSI CC0, NTRS public), so Muse Spark Contributor is acceptable. Two independent passes (the agreement rule of `ignis data review auto`), escalation of disagreements, a hand-labelled gold set of about 100 documents to measure accuracy before the labels are used. Live calls only with explicit `--execute`. | R7, K1-evidence | labels for the whole catalog with a measured accuracy; conflicts visible |
| **R9** | **Typed combustion graph and conditions service.** SQLite edge tables (master §11.3; no Neo4j, no "universal KG from every sentence"). Nodes: investigation, run, fuel and sample geometry, hardware (CIR/MDCA, MSG/SPICE duct, ACME modules), condition, diagnostic, observable, source, passage, figure, table, analysis, model release, wiki revision. Edges reuse and extend `ProvenanceLink`. **Seeded from the PSI experimental-table CSVs** (ground truth), then filled from R8 extraction with locators. Two services: `conditions_query` (per-run O2 mole fraction, pressure, flow velocity, fuel, geometry, gravity level, diagnostics, each a measured value with locator or an explicit `missing`/`conflicting`/`not_applicable` status, plus the tested envelope per investigation) and `graph_neighbors` (typed neighbours with evidence, cursor-paginated). The graph exists for joins and filters ("all runs with O2 ≥ 30 %", "investigations sharing MDCA"), not for community summaries: plain hybrid RAG matches or beats GraphRAG on single-hop questions *(lit.)*. | R6, R8, K1-evidence, K1-dataset | the conditions matrix for all 19 flight investigations, with gaps explicit; both services behind `KnowledgeService`/`DatasetService` with fixtures |
| R4 | (existing) **wiki** with CAS and ACE-style itemized deltas (no whole-page LLM rewrites) *(lit.)*, `wiki_read`, `ProvenanceLink` producer; **web discovery** via Parallel MCP into `ExternalSourceCandidate` with snapshot, rights and SSRF protections (master §13) | R3 | `wiki_patch` registered; a web-found source enters the corpus with a snapshot |
| **R10** | **Evaluation-driven retrieval upgrades**, each only if Tudor's retrieval benchmark shows the gap: Tantivy migration (measured recall/latency/memory), a ColQwen page/figure index for figure queries, HippoRAG-2-style personalized PageRank over entity–passage links for multi-hop *(lit.)*. Plus the R5 remainder (BASS metric calibration) and the evaluator dataset bundle (`audience = evaluator`) for Tudor. | T10 results | each upgrade has a before/after number on the dev split |

Raul keeps the presentation role he has (NASA evidence, bibliography, claim checking on slides), and
reviews the gold answers' citations for Tudor (T10) without seeing the private hold-out answers that
his own retrieval will be scored on.

---

## 4. Lane C — Tudor: Research workspace & evaluation

Goal: every contract the system produces has a faithful view, the agent can compose answers from a
trusted component catalog, the challenge's questions have direct views, and the harness is measured
independently.

| ID | Deliverable | Depends on | Done when |
|---|---|---|---|
| **T6** | **Retire legacy routes.** Runs, Lab, Compare and Atmosphere move to v1 routes (M11) and `generated.ts` types; delete hand-written DTOs in `api.ts`; one error model (`ErrorEnvelope`). Lab's seal/reveal goes through the vault only. | M11 | `api.ts` has no cross-lane DTO |
| **T7** | **Faithful viewers, streamed from NASA** (§7). Every original opens through `SourceAccess` (M12): PSI PDFs and videos load **directly from NASA's S3** (Range and CORS verified), NTRS PDFs through the Worker proxy, library media from its permanent URL; "Open at NASA" is always offered. PDF.js with a highlight layer at the `SourceLocator` bbox (S3 exposes only `ETag` to scripts, so PDF.js will likely load whole files rather than ranges; fine at these sizes, to confirm in the browser). Our own artifacts still go through blob URLs, because the Google-auth fetch wrapper cannot authorize `<img>`/`<video>` sources; signed NASA URLs need no token. BASS video with the contour drawn from our `VisualObservationBundle` as an SVG/canvas layer synced via `requestVideoFrameCallback` (no pixel readback needed; `pixel_only` never on a metric axis). CFI-G `.ts` video through a transmuxing player (hls.js/mpegts.js; *untested*). Never link a raw signed URL as a plain `<a>`: PSI sets `Content-Disposition: attachment`, so it would download instead of display. Typed artifact viewers: `AnalysisArtifact` (observed vs predicted, units, bands, explicit `time_origin`), `ExecutionRecord` (program, inputs, stdout/stderr, rejected outputs), `DatasetBundle` (columns, units, quality), `EvaluationResult` (coverage and scored quality side by side). Model release + qualification view. Wiki view (R4) and graph "related" navigation (R9) in the side panel. | R7 fixtures, K1 | every contract in `generated.ts` that a user can meet has a view or an explicit "not shown" reason |
| **T8** | **Catalog-driven answer views (static generative UI).** A component registry keyed on the closed `ViewSpec` enum (K1): `source_excerpt`, `series_comparison`, `conditions_matrix`, `atmosphere_map`, `video_overlay`, `release_card`, `artifact_table`. The agent places and configures components from this catalog through `AnswerBlock::View`; it never emits HTML or code. This is A2UI's security model (agent picks from an app-registered catalog) without adopting A2UI's transport *(lit.)*. Optional: a generated A2UI 1.0 catalog definition and MCP Apps exports for use inside Claude/ChatGPT; not the main UI (an iframe per card breaks linked views). OpenUI Lang is skipped (a DSL for a token saving we do not need). Linked views: one shared time/selection store (seconds since `time_origin`) syncing chart, video frame and PDF highlight. | K1-research, M7 | a live answer renders a chart and a source excerpt that the agent chose, both linked |
| **T9** | **Challenge views.** Conditions matrix (fuel × geometry × O2 × pressure × flow × diagnostics per investigation, gaps explicit) and habitat relevance ranking (distance of each investigation's tested envelope from the Moon/rover, suit and ISS atmospheres; "outside tested envelope" and "no partial-gravity data" as explicit statuses, never extrapolated silently). The Atmosphere map moves onto `conditions_query`. | R9, M8 | the three challenge objectives each have one view a judge can operate |
| **T10** | **Benchmark (T3 part 2).** `GoldTask` set (K1-evaluation), piloted at about 30 tasks before the 120 proposed in master §24: documentary tasks in LitQA2 format with an "insufficient evidence" option, scored on accuracy **and** precision so abstention counts *(lit.)*; citation tasks scored on whether the cited locator supports the statement (ALCE/FACT-style) *(lit.)*; analytical tasks with executable checks (ScienceAgentBench/BixBench style) *(lit.)*; predictive tasks only through the frozen evaluator protocol. Split into a **dev set** shared with Mircea for harness iteration and a **private hold-out of about 20 %** kept in the evaluator vault, never in a builder bundle, used once *(lit.: LAB-Bench)*. Retrieval sub-benchmark (single-hop, multi-hop, figure questions) that decides R10. | K1-evaluation | pilot set frozen with protocol hash; retrieval numbers published to Raul |
| **T11** | **H0/H1 and ablations (part of T4).** Same runtime build, model and effort; H0 = minimal workspace, H1 = IGNIS harness. Ablations: each skill, wiki, graph tools, subagents. Three or more seeds, paired statistics with cluster bootstrap, cost/tokens/tool calls from `TraceBundle`, full harness disclosure with every number (harness variance can exceed model variance *(lit.)*). | M10, T10 | a scorecard H0 vs H1 with intervals and costs, on the dev set; the hold-out run once with the team |
| T4 | (existing) WSP scoring and paired results | T2, I3, M5 | — |
| T5 | (existing) demo bundle, deck, video, verifiable export | T3, admitted results | — |

---

## 5. Shared contracts: what crosses lanes and how it merges

This section is the merge plan. It applies `CONTRACTS.md` to the Phase 2 additions.

### 5.1 Principles

1. **Contract wave first, implementation second.** All new cross-lane types land in one small
   contract PR per family (wave **K1**) before the lanes build on them: Rust type + `validate()` +
   fixture in `fixtures/contracts/manifest.json` + `ignis schemas export` + `npm run gen:types` +
   consumer stubs. No implementation in a contract PR.
2. **One version bump per family per wave.** With `deny_unknown_fields`, a new enum variant or field
   is breaking for strict readers (`CONTRACTS.md` §"Cum schimbi un contract", rule 3). So each family
   gathers all its K1 changes into a single bump (`research.v1 → v2`, `evidence.v1 → v2`,
   `dataset.v1 → v2`, `evaluation.v1 → v2`), with a reader that still accepts durable v1 data (event
   logs in SQLite, stored answers, manifests). No second bump until the next wave.
3. **Cross-family references only through ids and refs.** A type in one family points at another
   family's object only through `ArtifactRef`, `EvidenceRef`, `EntityRef`, typed ids or `Digest`, never
   by embedding the other family's struct. Example: `ViewSpec::ConditionsMatrix` holds an `ArtifactRef`
   to a `ConditionsTable`, not the table. This keeps the families' PRs independent and their bumps
   uncoupled.
4. **Fixture-first.** Every new type ships with a fixture that the producer's tests and the
   consumer's tests both read, and that `/api/fixtures/v1` serves. Consumers build against the fixture
   while the producer works; the fixture is the merge point.
5. **Capability-gated.** A new tool or service is registered only when configured, and
   `/capabilities` says why when it is not. Lanes merge disabled-but-published code without breaking
   the deployed system.
6. **Trait additions do not break other implementors.** New methods on service traits in
   `services.rs` get a default body returning `capability_unavailable`, so fixture and existing
   implementations keep compiling when the trait grows.
7. **Generated files are never merged by hand.** On conflict in `schemas/` or `generated.ts`, take
   `main` and regenerate from the integrated Rust. Contract PRs merge one at a time through the
   integrator, in the order of §5.3.
8. **Closed catalogs change by contract PR only.** Adding a `ViewSpec` kind, a `SourceLabel`
   document type or a graph relation is a contract change, reviewed by the renderer/consumer.

### 5.2 Wave K1: the new and changed types

| # | Type / change | Family file (owner) | Producer | Consumers (required reviewers) | Fixture |
|---|---|---|---|---|---|
| K1.1 | `ToolStarted.arguments: Option<ToolArguments>` (`InlinePayload` or `ArtifactRef`, redacted by scope) — P1 | `research.rs` (Mircea) | Mircea (runtime mapping) | Tudor | `research/events.json` gains arguments |
| K1.2 | `EventBody::StepRationale { call_id?, text, source: model_explanation }` — P2 | `research.rs` | Mircea | Tudor | `research/events.json` |
| K1.3 | `AnswerSubmission` (input of `answer_submit`) + refusal reasons (`unknown_evidence`, `unresolved_pointer`, `kind_support_mismatch`, `unpinned_release`) | `research.rs` | agent via MCP; validated by Mircea | Tudor (renders refusals), Raul (evidence semantics) | `research/answer_submission{,_refused}.json` |
| K1.4 | `AnswerBlock::View { view: ViewSpec, caption?, claim_ids }` and closed `ViewSpec` catalog v1: `source_excerpt{evidence}`, `series_comparison{series: [SeriesRef{artifact_id, pointer, role: observed\|derived\|predicted, unit_pointer}], time_origin_pointer}`, `conditions_matrix{table: ArtifactRef}`, `atmosphere_map{table: ArtifactRef}`, `video_overlay{source_id, bundle: ArtifactRef}`, `release_card{release_id}`, `artifact_table{artifact_id, pointer}` | `research.rs` (Mircea writes; **Tudor co-designs**) | agent via `answer_submit` | Tudor (renderer), Iancu (series semantics for predictions) | `research/answer_views.json` with one block per kind |
| K1.5 | `Claim.checked_by: Option<SupportCheck{method, model_id?, verdict: supported\|partial\|unsupported, confidence?}>` — P3 (a Jev-style typed judge fits) | `research.rs` | Mircea (host-side check at `answer_submit`) or Tudor's evaluator offline | Tudor | `research/answer.json` |
| K1.6 | `EpisodePins.harness: Option<HarnessRef{digest, label, runtime_build, model, effort?}>`, replacing the unused `skills_version` | `research.rs` | Mircea | Tudor (shows it; T11 groups by it) | `research/episode.json` |
| K1.7 | Server-only types moved in: `CreateEpisode`, `PostMessage`, `MessageAccepted`, `HostDecision`, `Capabilities`, `ServiceStatus`, `ToolStatus`, `EpisodeAction` — P6 | `research.rs` | Mircea | Tudor (deletes `serverTypes.ts`) | `research/capabilities.json` |
| K1.8 | `TraceBundle` manifest: harness ref, event range, tool I/O artifacts, measured usage, partitions touched | `research.rs` | Mircea | Tudor (T11 runner) | `research/trace_bundle.json` |
| K1.9 | `tool_catalog()` entries: `answer_submit` (effect `compute`), `sources_outline`, `conditions_query`, `graph_neighbors`, `wiki_read`, `web_search`, `web_fetch` (descriptors only; request/response types live in their families) | `research.rs` | Mircea | Raul (services), Tudor | catalog test |
| K1.10 | `SourceLabel` + `LabelSet` (closed `DocType`, investigation id, runs as `EntityRef`, method `semantic`, model id, prompt hash, confidence, agreement, status) — P4 | `evidence.rs` (Raul) | Raul (R8) | Mircea (transport), Tudor (viewer) | `evidence/source_labels.json` |
| K1.11 | `ProvenanceLink` relation enum extended for the graph (`describes_run`, `uses_fuel`, `has_condition`, `measured_by`, `uses_hardware`, `supports`, `disputes`, `derived_from`, `same_as_candidate`) + `GraphNeighborsRequest/Page` (cursor) — P5. No parallel edge type. | `evidence.rs` | Raul (R9) | Mircea, Tudor | `evidence/graph_neighbors_page.json` |
| K1.12 | `SearchRequest`/`ReadRequest.response_format: concise\|detailed` | `evidence.rs` | Raul | Mircea | existing search fixtures |
| K1.13 | `ConditionsQuery` / `ConditionsTable`: per-run conditions as measured values (raw, unit, SI, locator) or explicit status; per-investigation tested envelope | `dataset.rs` (Raul) | Raul (R9) | Tudor (T9), Mircea (tool, M11), Iancu (review: units) | `data/conditions_table.json` (incl. missing and conflicting cells) |
| K1.14 | `GoldTask` (class, question, answer key with accepted locators or executable check or protocol ref, `split: dev\|holdout`; hold-out items never serialized into builder bundles) | `evaluation.rs` (Tudor) | Tudor | Mircea (M10 runner reads dev only), Raul (citation review) | `evaluation/gold_task_{documentary,analytical}.json` |
| K1.15 | `EvaluationRequest.purpose = harness_benchmark` and a `HarnessComparison` section in `EvaluationResult` (per variant, seed and task; paired deltas with intervals; cost from `TraceBundle`) rather than a parallel result type | `evaluation.rs` | Tudor | Mircea, team review before freeze | `evaluation/harness_comparison.json` |
| K1.16 | (only if figure citations need it) `SourceLocator.figure_id`, additive and omitted when absent | `contracts.rs` (Iancu, semantic) | Raul (R7) | Tudor, Raul | `evidence/` locator fixture |
| K1.17 | Service traits: `KnowledgeService::{graph_neighbors, labels, outline, access}`, `DatasetService::conditions` with default `capability_unavailable` bodies | `services.rs` (common) | Raul implements | Mircea (composition root) | — |
| K1.18 | `SourceAccess { source_id, mode: direct\|proxy\|link_out, url, expires_at?, range: bool, cors: bool, landing_url }`: how the UI opens an original without IGNIS storing it. The stable identity stays the `Source` (geode URL + file version + hash); `url` is a short-lived delivery detail, never stored in citations | `evidence.rs` (Raul) | Raul (mode per source, R6); Mircea resolves the URL (M12) | Tudor (viewers) | `evidence/source_access_{direct,proxy,link_out}.json` |
| K1.19 | `OutlineRequest` / `OutlineNode { node_id, title, page_start, page_end, summary?: GeneratedText{model_id}, children }` for the section tree (R7); a summary is labelled generated and is never evidence | `evidence.rs` | Raul | Mircea (tool), Tudor (outline in the source panel) | `evidence/outline.json` |

`wsp.rs` is not touched by this wave. `common.rs` is not expected to change; if it must, it follows
the common-file rule (one reviewer per consuming lane).

### 5.3 Merge order

```text
K1-evidence + K1-dataset (Raul)  ──►  K1-research (Mircea)  ──►  K1-evaluation (Tudor)  ──►  lanes build
     K1.10–13, K1.17                   K1.1–9 (tool descriptors       K1.14–15 (reads TraceBundle
                                       name Raul's request types)     and HarnessRef by id)
```

The three PRs are **written in parallel** from the same `main` (principle 3 keeps them independent:
`research.rs` names Raul's types only through ids/refs and tool names, `evaluation.rs` reads
`TraceBundle` and `HarnessRef` by id). They **merge back to back** in the order above, each rebased on
the previous with `schemas/` and `generated.ts` regenerated, never hand-merged. Nobody waits for
another lane to start writing. K1.16 rides with K1-evidence if needed. Any type discovered later
waits for wave K2, which follows the same rules.

### 5.4 Interfaces after K1

| From → to | Contract | Path |
|---|---|---|
| Raul → Mircea | `SourceLabel`, graph page, `ConditionsTable`, wiki, web candidates | `KnowledgeService` / `DatasetService` → MCP tools, `/api/v1` |
| Raul → Tudor | the same, plus parse views with bboxes and figures | `/api/v1` (via Mircea), fixtures until then |
| Mircea → Tudor | `answer.completed` with `View` blocks, `tool.started.arguments`, `step.rationale`, `Capabilities`, `HarnessRef`, `TraceBundle` | SSE + `/api/v1` |
| Tudor → Mircea | dev `GoldTask`s and the scoring protocol; never the hold-out | `benchmarks/` dev split + evaluator API |
| Tudor → Raul | retrieval benchmark results | `benchmarks/` |
| Iancu → all | `ModelRelease`, `AnalysisArtifact` (unchanged) | `ModelService` |

---

## 6. End-to-end milestones (dependency order, not calendar)

1. **Bound answer, live**: M6 + M7 + M12 + T7 (PDF at bbox, streamed from NASA). A real question in the deployed app gives a
   submitted, validated answer whose citations open the PDF page.
2. **Whole challenge corpus**: R6 + R7 + R8. All 19 flight investigations catalogued (~14.1 GB stored, media at NASA); classification accuracy
   measured.
3. **Conditions and habitats**: R9 + M8 + T9. The agent answers "which investigations tested
   O2 ≥ 30 % below 10.2 psi, and what is missing" with a conditions matrix it placed itself (T8).
4. **Measured harness**: T10 pilot + M10 + T11. H0 vs H1 on the dev set, with costs and intervals.
5. **WSP joins**: Iancu's I2/I3 → M5 `ModelService` → T4.

---

## 7. NASA data: what the server stores, and how the UI reaches the rest

**Storage rule** (Iancu's atlas: `docs/wsp/atlas/SERVER_STORAGE.md`, `server_documents.csv`,
`server_sensor_text.csv.gz`, built from NASA's own listings of the 19 flight investigations on
2026-11-14, no file content read; in Iancu's checkout, to be pushed):

| What | Files | Size | On the server? |
|---|---|---|---|
| Documents, tables, analysed data (categories Experimental table, Science Documents, Reports, Presentations, Engineering Documents, Analyzed Data, Reference Data, Calibration, Investigation Metadata Files; types pdf, doc(x), ppt(x), xls(x), csv, txt, rtf, pxp, log, json, xml, md) | 636 | 1.63 GB | yes |
| Sensor and settings text in Raw Data (same types) | 25,293 | 12.47 GB | yes (per investigation as it is taken up) |
| Images, video and archives of them in Raw Data | 40,336 | 13.47 TB | **no**: manifest entry only, opened at NASA |
| Other archives outside Raw Data | 236 | 12.66 GB | no |

So the server keeps about 14.1 GB of NASA's ~13.5 TB, plus the listings (~30 MB) and what IGNIS derives
(indexes, parse views, labels, graph, Parquet tables, contours, releases, forecasts, artifacts).

**Access, measured 2026-11-14** (requests sent from the dev machine with `Origin: https://ignis-ardealu.app`):

| Host | Range | CORS | Notes | IGNIS access mode |
|---|---|---|---|---|
| PSI download `psi.nasa.gov/geode-py/ws/studies/<PSI>/download` | n/a | only `https://psi.nasa.gov` | 302 to a signed S3 URL (or the URL as text with `redirect=false`); the signed URL expired ~1 h after issue | backend resolves (M12) |
| PSI bytes `psi-curation-prod.s3.amazonaws.com` (MP4, PDF, ZIP) | 206 | `*`, preflight allows `range` | exposes only `ETag` to scripts; `Content-Disposition: attachment` | `direct`: browser streams from NASA |
| NTRS `ntrs.nasa.gov/api/citations/<id>/downloads/<file>` | 206 | none | `X-Frame-Options: SAMEORIGIN` (no iframe); `.txt` full text offered | `proxy` (Worker pass-through) + link to `ntrs.nasa.gov/citations/<id>` |
| NASA Image and Video Library `images-assets.nasa.gov` (+ `images-api.nasa.gov` search) | 206 | `*` | permanent unsigned URLs; has CIR/ACME hardware videos, not run videos | `direct` |
| YouTube (ScienceCasts) | — | — | — | embed |

**Consequences**

- The viewers stream NASA originals; no media mirror, no R2 copy of NASA bytes. The Worker proxy for
  NTRS forwards `Range` and stores nothing (NASA asks for links rather than rehosting).
- The image archives (most of the 13.47 TB) are never sent to the browser. When a single frame is
  needed (BASS calibration, a figure), the backend reads the ZIP's central directory with one `Range`
  request on the tail and then fetches just that member; verified on a PSI-26 archive (one 64 KiB
  read listed its 108 entries). Archives over 4 GiB use ZIP64, same technique, not yet tested. The
  extracted member becomes a small artifact with lineage to the NASA file.
- Identity never depends on a signed URL: a citation names the `Source` (geode URL + file version +
  hash); `SourceAccess.url` is resolved at view time.
- The geode endpoints are the public site's, not a documented API: they can change, so samples are
  saved and calls are throttled. PSI-10's licence is blank in NASA's metadata and must be checked.

---

## 8. Sources

Verified against arXiv or primary pages on 2026-11-14 unless marked. "Secondary" = figure from a
third-party summary only.

**Harness**
- Recursive Language Models: Zhang, Kraska, Khattab, arXiv 2512.24601 (blog Oct 2025). Context held
  in a REPL, recursive sub-calls; strong on long-context aggregation. Reproduction arXiv 2603.02615:
  depth-1 helps hard aggregation, hurts simple retrieval, depth 2 worse, 25–100× latency.
- Meta-Harness: Lee, Nair, Zhang, Lee, Khattab, Finn, arXiv 2603.28052. Outer loop over harness code
  with a proposer that reads full traces; beats hand-built harnesses. Lesson: keep full traces.
- ACE (Agentic Context Engineering), arXiv 2510.04618: itemized playbook deltas avoid context
  collapse. GEPA, arXiv 2507.19457: reflective prompt evolution. DSPy arXiv 2310.03714. ADAS
  2408.08435, AFlow 2410.10762, Darwin Gödel Machine 2505.22954 (too costly for us).
- Agent Skills (Anthropic, Oct 2025; agentskills.io open standard, Dec 2025; adopted by Codex).
  SkillsBench, arXiv 2602.12670: curated skills +16.6 points pass rate; focused skills beat exhaustive
  documentation.
- Anthropic, "Effective context engineering for AI agents" (Sep 2025) and "Writing effective tools for
  agents" (Sep 2025). MCP description smells: arXiv 2602.14878, 2602.18914. "Evaluating AGENTS.md",
  arXiv 2602.11988: LLM-generated context files slightly hurt; minimal human-written ones help a little.
  Harness disclosure: arXiv 2605.23950.
- Scientific agents/benchmarks: PaperQA2/LitQA2 2409.13740; ALCE 2305.14627; DeepResearch Bench
  2506.11763; ScienceAgentBench 2410.05080; BixBench 2503.00096; DiscoveryBench 2407.01725; SciCode
  2407.13168; LAB-Bench 2407.10362; Robin 2505.13400.

**Knowledge**
- GraphRAG 2404.16130; LightRAG 2410.05779; HippoRAG 2 2502.14802; RAPTOR 2401.18059; KAG 2409.13731.
  When graphs help: GraphRAG-Bench 2506.05690; RAG vs GraphRAG 2502.11371 (secondary cost figures).
- ColPali 2407.01449; ColBERTv2 2112.01488; PLAID 2205.09707.
- PageIndex (VectifyAI, open source): table-of-contents tree with node summaries, LLM tree search
  instead of vector search; 98.7 % FinanceBench is a vendor claim (*unverified*); returns pages and
  sections, not bboxes, so it sits on top of a parser.
- Jev (TypeSafe): decision model answering typed questions over a text with confidence; listed on
  OpenRouter, Vercel AI Gateway, Pydantic AI; versions and prices differ between listings (*unverified*).
- Parsers: Docling (bbox, MIT), GROBID (TEI coords, Apache), MinerU (AGPL), Marker (GPL),
  PaddleOCR-VL 2510.14528, dots.ocr; olmOCR 2 2510.19817 and Nougat have no bboxes.
- Muse Spark (Meta Superintelligence Labs): 1.3 released 2026-09-02, 1M context, PDF input; standard
  $1.25/$4.25 per M tokens; Contributor tier reported $0.10/$0.20 with training use of prompts
  (aggregators; official pricing page unreachable: *unverified*).

**Workspace**
- A2UI 1.0 (Google, 2026-11; React renderer `@a2ui/react`), declarative catalog model. MCP Apps
  (SEP-1865, sandboxed `ui://` iframes). AG-UI (CopilotKit, event transport). OpenUI (thesys, DSL).
  Vercel AI SDK typed tool parts.
- NASA access: PSI repository (DOIs, CC0, e.g. FLEX DOI 10.60555/mbq8-0451); NTRS OpenAPI
  (`sti.nasa.gov` documentation, rate limits); PSI/OSDR consolidated front door (Oct 2025).
- Challenge: spaceappschallenge.org/2026/challenges/flame-in-freefall (read 2026-11-14); judging guide
  (2025 edition).

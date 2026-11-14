# Phase 2 lanes: harness, knowledge, workspace and evaluation

---

## 1. From first principles: what has to be true, and who makes it true

**What we must demonstrate.** The challenge asks for an SI dashboard that finds, summarizes, ranks,
compares and interprets NASA's microgravity combustion investigations for habitat fire safety. A
researcher (and a judge) must be able to (a) ask a question about any of the 19 flight investigations,
(b) get an answer whose every claim opens at its source, (c) compare tested conditions against habitat
atmospheres, (d) see a forecast with its qualification state, and (e) trust that the system was measured,
not just demoed.

**The value chain**, each link with one owner:

| Link | Question it answers | Owner | Why this person |
|---|---|---|---|
| Data reachable and citable | is every claim's source in the system, openable at the exact place? | **Raul** | owns the corpus, parsers, datasets and retrieval already (R1–R5) |
| Agent that uses it and binds answers | does the live agent produce checked, bound answers, cheaply and repeatably? | **Mircea** | owns the runtime bridges, MCP, API, runner and deployment (M1–M5); the harness is configuration of exactly those |
| Researcher can see and verify | can a person follow claim → evidence → source → run, and operate the challenge views? | **Tudor** | owns the workspace (T1, T3, PR #16, #24) |
| Forecasts with honest state | is there a literal WSP release with a real score? | **Iancu** | WSP lane, unchanged (I1–I5) |
| Proof that it works | does the harness beat a plain agent on frozen tasks, at what cost? | **Tudor** | the evaluator must not be a builder of what is scored; Tudor builds neither harness nor retrieval |

**Independence constraints that shape the split.** The person who tunes the harness (Mircea) and the
person who tunes retrieval (Raul) never see the private hold-out; the evaluator (Tudor) owns it and runs
it once, with the team, on a frozen protocol (the "Protect the test" rule of `AGENTS.md`, applied to the
benchmark). Model authors never see test outcomes (Iancu's lane, unchanged).

**Critical path** (everything else is secondary until these hold):

1. **Bound answer, live**: M7 `answer_submit` + M6 minimal harness + M12 source access + T7 PDF at bbox.
2. **Conditions across all 19 investigations**: R6 corpus + R9 conditions table (seeded from PSI's
   experimental tables) + T9 conditions matrix and habitat ranking.
3. **Measured harness**: T10 pilot gold set + M10 traces + T11 H0/H1.

**Priorities used below.** **P0** = on the critical path; start now. **P1** = needed for the full
demo; start when P0 is unblocked or waiting. **P2** = improvement; only after P0/P1 or when measurement
says it pays.

---

## 2. Where we are (main at 062d2cd, 2026-11-14)

- **Platform:** deployed on `ignis-ardealu.app` (Access + Worker + Tunnel + R2, GCE VM, gVisor runner
  verified). `/api/v1` with resumable SSE, durable jobs, artifacts, uploads, OpenAPI. Codex 0.169.0 and
  Claude Code bridges (Claude verified only against the fake binary). Google sign-in (ADR 18) in
  progress locally on `feat/google-auth`.
- **Harness gaps:** no system prompt, developer instructions or skills reach either runtime; the
  per-episode cwd is empty; `EpisodePins.skills_version` is always `None`. **No live turn produces a
  `ResearchAnswer`** (only `message.completed` text; `answer.completed` exists only in the fixture
  server). No tool arguments in events, no wiki service, no delegation, no traces.
- **Knowledge:** 6 indexed documents, datasets of 4 investigations, FTS5 + dense + RRF + rerank, MuPDF
  parsing. Iancu's atlas (`docs/wsp/atlas/`) lists every NASA file of the 19 flight investigations and
  the storage rule (§10). **Raul's K1 contract wave is merged** (PR #26:
  `evidence.v2` + `dataset.v2`: `SourceAccess`, `SourceLabel`, `SourceOutline`, wiki read/patch, web
  search/fetch, `ConditionsQuery`/`ConditionsTable`, `ResponseFormat`).
- **Workspace:** shell, Answer view with claim keys, Research route, Atmosphere map, Runs, Compare,
  Lab, Sources, Models (PR #16). **PR #24 (T7 part 1, open):** typed views for `ExecutionRecord`,
  `DatasetBundle`, `EvaluationResult`, `AnalysisArtifact`, release construction, and a compile-time
  contract coverage table. Runs/Lab/Compare/Atmosphere still use legacy `/api/experiments*`.
- **Evaluation:** fixture evaluator, protocol freeze, scorecards. No gold tasks, no H0/H1 runner.
- **GitHub:** 23 merged branches deleted on 2026-11-14 (SHAs recorded by the integrator). Remaining:
  `main` and `feat/research-product-t7` (PR #24). CI is disabled for now (see `AGENTS.md`).

---

## 3. How we work

**Access first:** repo, local setup, the staging app and VM, where secrets and data live:
[ACCESS.md](ACCESS.md).

**Branches.** One short-lived branch per task from current `main`, named `<lane>-<task>`
(`feat/agent-backend-m7`, `feat/scientific-data-r6`, `feat/research-product-t10`), one PR each, deleted
on merge. No long-lived lane branches. A worktree per person: `git worktree add -b <branch>
../ignis-<name> origin/main`.

**Contracts first** (§9): the three K1 PRs are written in parallel and merged back to back; consumers
build on fixtures meanwhile.

**Research method: how to choose the best option yourself.** Do not adopt a library, model or
technique from a blog post or from this document. For every real choice:

1. Write the decision as a question with a metric ("which parser gives the best bbox-accurate text on
   PSI reports, per second of CPU?").
2. Build a small evaluation from **our** data first: 10–50 items, labelled by hand, committed under
   `benchmarks/` (Tudor's dev set when it exists). Include the hard cases (scanned 1990s reports, tables,
   long reports, missing values).
3. Shortlist at most three candidates from primary sources: arXiv (search via Semantic Scholar or
   alphaXiv; read the method and the ablations, not the abstract), the project's GitHub (releases, open
   issues about your exact use, licence), official docs and pricing pages. Treat vendor benchmarks and
   aggregator prices as *unverified* until reproduced.
4. Measure on the evaluation: quality, p50/p95 latency, peak memory, cost per 1,000 items, licence.
   Same inputs, same machine, numbers in the PR.
5. Record the decision in the PR description (or an ADR when it changes architecture), with the
   numbers and the rejected options. A choice without numbers is provisional.
6. Re-measure when inputs change (new model version, more documents).

**Shared performance rules** (from `AGENTS.md`, made concrete): move references (`ArtifactRef`,
`SourceAccess`), never payloads; stream bytes; Parquet for numeric tables; no CPU work on Tokio reactor
threads; every cache key includes every version that changes the answer; no whole-corpus loads; measure
before optimizing and report p50/p95, bytes moved and cost.

---

## 4. Lane A — Mircea: harness & platform

**Mission:** the live agent produces validated, bound answers under a versioned harness whose effect is
measured, at a known cost per question; NASA originals reach the browser without copies. The runtime
keeps the loop (no second loop, no compactor, no gateway).

| P | ID | Deliverable | Depends on | Done when |
|---|---|---|---|---|
| P0 | **M7** | **Bound answer path.** `answer_submit` (`AnswerSubmission` → host validation → `ResearchAnswer` artifact + `answer.completed`). The host checks `ResearchAnswer::validate()` and that each `EvidenceRef` was returned by a tool result in this episode, each `ArtifactValue` pointer resolves with a unit, each release is pinned. A failure is a `refused` `ToolResult` with an actionable reason so the agent fixes and resubmits. Effect class `compute` (writes only this episode's answer through host checks; a `write` tool would fail closed because the bridge refuses elicitations). Supports `AnswerBlock::View` (catalog co-designed with Tudor). | K1-research | a live question on the deployed app ends in `answer.completed`; a deliberately bad citation is refused and corrected |
| P0 | **M6** | **Harness bundle v1, minimal first.** `harness/investigator.md` (operational instructions trimmed from master §25.1 to the invariants: claim kinds, citation duty, statuses, model output ≠ measurement, test isolation, "submit through `answer_submit`"), `harness/skills/<name>/SKILL.md` (agentskills.io), later `harness/agents/*.md`. Tool section generated from the registered tools. Injected into Codex (developer/base instructions on `thread/start`; skills under the episode cwd `.agents/skills/`; confirm field names in the protocol schema the preflight exports) and Claude Code (`--append-system-prompt`; `.claude/skills/`; check whether `--tools ""` and `--setting-sources ""` still allow the Skill tool). Digest pinned in `EpisodePins.harness`. Start with instructions + 2 skills; add only what an observed failure justifies *(lit.: AGENTS.md eval, context engineering)*. | K1-research | both runtimes show the instructions took effect; digest on the episode |
| P0 | **M12** | **Source access without copies** (§10). `GET /api/v1/sources/{id}/content` → `SourceAccess`: PSI = fresh signed S3 URL via geode `redirect=false`; NTRS = Worker pass-through proxy (allow-listed host, `Range` forwarded, nothing stored); NASA media library = permanent URL; otherwise `link_out`. For CFI-G `.ts` video, a byte-range HLS playlist built from Raul's keyframe index (§10). | K1-evidence, R6 | Tudor's viewer opens a PSI PDF at a page, plays a PSI MP4 and a CFI-G `.ts` with seeking, and opens an NTRS PDF, with zero NASA bytes stored |
| P1 | **M10** | **Traces.** `TraceBundle` per episode: harness digest, runtime build, model, effort, event range, tool I/O as artifacts, measured usage and cost, partitions touched. `ignis harness run --variant <v> --tasks <dev set>` for Tudor's runner. | K1-research, T10 | H0 and H1 run unattended on the dev set with complete traces |
| P1 | **M8** | **Tool ergonomics.** Tool arguments in `tool.started` (redacted by scope), `step.rationale` (the model's own explanation, never chain-of-thought), server-only types moved into `ignis-core`, `response_format` and cursors wired, descriptions audited against the MCP smell rubric *(lit.)* and A/B tested on the dev set. Register Raul's tools (`sources_outline`, `conditions_query`, `graph_neighbors`, `wiki_read`, `wiki_patch`, `web_search`, `web_fetch`) behind capabilities as they land. | K1 | UI shows arguments; new tools appear only when configured |
| P1 | **M11** | v1 routes for runs, datasets and conditions so the UI retires `/api/experiments*` | K1-dataset, R9 | Tudor's T6 deletes the legacy DTOs |
| P2 | **M9** | **Context in the environment, depth-1 delegation.** The useful half of RLM *(lit.)*: datasets and corpus slices mounted into `analysis_execute` by reference (a small `ignis` Python helper in the runner image), so the agent computes instead of reading tables into context. Delegation only through runtime-native subagents, depth 1, derived child scope (per-child token or host check), budget, result contract. Simple lookups stay off this path (recursion hurts simple retrieval *(lit.)*). | M6 | a multi-run aggregation answered on handles; ablation available to T11 |
| P2 | M10b | Offline, dev-only harness search (GEPA / Meta-Harness style *(lit.)*); the proposer refuses traces that touched `test` or the hold-out | M10, T11 | a proposed harness change beats the current one on dev |
| P2 | M4, M5 | restore drill (RPO/RTO measured), denied approval exercised live, merge Google auth; observability; `ModelService` after I2 | — | — |

**Skills** (content reviewers): `source-grounding` (Raul), `experimental-semantics` (Raul),
`quantitative-analysis` (Tudor), `model-qualification` (Iancu); `research-memory` with the wiki.
Hand-written, focused, 2–3 modules each *(lit.: SkillsBench)*; each one ablated in T11 and removed if
it does not help.

**Research tips (Mircea)**
- Read the pinned Codex protocol schema (exported by the preflight) for instruction fields, and Claude
  Code's headless docs for `--append-system-prompt`, skills and subagents; test each flag with the fake
  binaries first, then one live turn.
- Prompt-cache behaviour differs by provider: read each provider's caching docs and confirm from the
  usage fields in real turns (cache read tokens), not from assumptions.
- For tool design, read Anthropic's "Writing effective tools for agents" and the two MCP description
  studies *(lit.)*; then measure on the dev set, because description changes shift tool selection.
- For harness search, read Meta-Harness and GEPA for what they log; copy the logging, not the loop.

**Efficiency and performance (Mircea)**
- **Prompt caching:** keep the instruction prefix byte-stable (instructions → generated tool section in
  sorted order → skill metadata); no timestamps, episode ids or user data in it. Track cache-read share
  per turn in `TraceBundle`; it is the biggest cost lever.
- **Token budget per question:** tools answer `concise` by default; large results become `ArtifactRef`
  with a preview (existing 64 KiB rule); the agent reads more only on demand.
- **Signed URL resolver:** single-flight per `(source, file version)`, TTL = expiry minus 5 min (URLs
  live ~1 h, *measured*), bounded LRU, one re-resolve on 403; geode calls throttled (undocumented API).
- **NTRS proxy:** stream with `Range` passthrough, no buffering, no storage.
- **`answer_submit`:** keep a per-episode index of returned evidence ids so validation is O(claims).
- **Traces:** written after the turn, content-addressed, off the request path.
- **Report** per release: p50/p95 tool latency per tool, tokens and dollars per answered question,
  cache-read share.

Mircea has no presentation tasks.

---

## 5. Lane B — Raul: knowledge & NASA data

**Mission:** every one of the 19 flight investigations is in the system: documents openable at the
cited place, sensor tables as typed datasets, conditions comparable across investigations, labels and a
graph the agent can query, with NASA's media left at NASA.

| P | ID | Deliverable | Depends on | Done when |
|---|---|---|---|---|
| done | **K1-evidence/dataset** | `evidence.v2` + `dataset.v2`, merged in PR #26 | — | merged |
| P0 | **R6** | **Corpus for the 19 flight investigations** by the atlas rule (§10), downloaded **on the VM** into `/srv/ignis/data/raw/<PSI-id>/` by a scripted, incremental step in `infra/scripts/build-data.sh` (Raul supplies the selection from the atlas CSVs, Mircea wires and runs it); laptops keep only a development subset ([ACCESS.md](ACCESS.md) §5). Store the 636 documents (1.63 GB) and the 24,482 sensor/settings text files (12.26 GB) with hashes; the 35,446 image/video/archive files (13.44 TB) and 236 other archives stay as manifest entries. Licence per investigation (PSI-10 blank: verify). NTRS via its API, `.txt` full text where present, rights per document, dedup with PSI. A `SourceAccess` mode per source. **Lockbox:** documents of FLEX, CFI-G, BASS-II and BASS contain held-out outcomes; `strip_for_sealed` must hold for every added document (test), and for the other 15 once their groups are frozen. | — | manifests for all 19; ~13.9 GB stored and hashed; snapshot reproducible; lockbox test on a newly added document |
| P0 | **R9a** | **Conditions table first.** `ConditionsTable` for all 19 investigations, seeded from the 21 PSI experimental-table CSVs (ground truth, tiny), each cell a measured value with raw value, unit, SI and locator, or `missing`/`conflicting`/`not_applicable`; per-investigation tested envelope. This alone powers the challenge's comparison and habitat views. | K1-dataset, R6 | the conditions matrix renders for 19 investigations with gaps explicit |
| P0 | **R7a** | **Parse views with coordinates** for the stored PDFs: page, bbox normalized `[0,1]` top-left, char spans, tables, figures; versioned; old citations still resolve. Candidates: Docling, GROBID (headers/references), MuPDF baseline; choose by the research method (several PSI reports are scanned: OCR quality matters). | R6 | a passage, a table cell and a figure open at the right place in T7 |
| P1 | **R7b** | **Section tree** (`SourceOutline`): the document's heading hierarchy with page ranges and generated summaries (labelled generated, never evidence), exposed as `sources_outline` so the agent walks long reports and then reads the passage. PageIndex-style *(lit.)*, but the agent is the navigator, so no extra LLM call per step. | R7a | the FLEX TP (314 pp) and BASS-II summary (144 pp) are navigable |
| P1 | **R8** | **Labels** (`SourceLabel`): closed-enum document type, investigation, runs, conditions mentioned; two passes, escalation on disagreement, gold set of ~100 hand-labelled documents before use. A typed decision model (Jev) or a small model with structured output; choose by the research method. | R7a | labels with measured accuracy; conflicts visible |
| P1 | **R2b** | **Sensor text → datasets** for the investigations taken up beyond the first four: Parquet with `ColumnSpec`, units, `time_origin`, quality report | R6 | `datasets_inspect` works for a fifth investigation |
| P1 | **R9b** | **Graph** (`ProvenanceLink` edges in SQLite): investigation, run, fuel, hardware, condition, diagnostic, observable, source, passage, figure, table; `graph_neighbors`. For joins and filters, not community summaries *(lit.: GraphRAG-Bench)*. | R8, R9a | "investigations sharing MDCA", "runs with O2 ≥ 30 %" answered with evidence |
| P1 | **R11** | **Media indexes for streaming** (§10): keyframe index per CFI-G `.ts` file (byte offset ↔ PTS of each IDR) and cached ZIP central directories for archives that viewers need frames from; frames converted TIFF → PNG/WebP on demand and cached as artifacts | R6 | M12 serves seekable `.ts` playback; a BASS frame opens from its archive |
| P2 | R4 | wiki (CAS, itemized deltas, never whole-page rewrites *(lit.: ACE)*), `wiki_read`, web discovery with snapshot and rights | R3 | `wiki_patch` registered |
| P2 | R10 | retrieval upgrades only where T10 shows a gap: Tantivy, ColQwen figure index, PPR multi-hop *(lit.)*; BASS metric calibration; evaluator dataset bundle | T10 | before/after numbers |

**Research tips (Raul)**
- **Parsers:** take 10 PSI PDFs (2 scanned, 2 table-heavy, 2 long, 2 slide decks, 2 engineering docs)
  and compare Docling, GROBID and the MuPDF baseline on: bbox accuracy against a hand-marked passage,
  table cell accuracy, CPU seconds per page, licence. Check the parsers' GitHub issues for scanned-PDF and
  table behaviour before running.
- **Labels:** compare a typed decision model (Jev) with a small model using structured output on the
  100-document gold set: accuracy per label, agreement between two passes, cost per 1,000 documents.
  Jev's versions and prices differ between listings: confirm on TypeSafe's own docs.
- **Graph or not:** read GraphRAG-Bench and "RAG vs GraphRAG" *(lit.)* for when graphs help; build the
  table and edges, then let T10's multi-hop questions decide whether PPR is worth it.
- **Retrieval:** for any backend change (Tantivy, ColQwen), measure recall@k, p95 latency and memory on
  the T10 retrieval questions against the current FTS5 + dense + rerank.

**Efficiency and performance (Raul)**
- **Incremental ingest:** skip a file when `(investigation, file name, file version, size)` is unchanged;
  hash while streaming; bounded concurrency (start at 4) and a polite rate to geode and S3.
- **Parse once:** key parse views by `(file hash, parser, parser version)`; store as content-addressed
  artifacts; build index generations separately and switch by atomic pointer.
- **Embeddings:** batch; cache by `(chunk hash, model id)`; rerank only the top k.
- **Sensor text:** convert once to Parquet (zstd), read with column pruning; never load an investigation
  whole.
- **Classification cost:** static prompt prefix (instructions, schema, examples) then the document, so
  provider prompt caching applies; first pages + outline instead of full text when accuracy holds
  (measure); batch APIs where offered.
- **Remote ZIP:** cache each archive's central directory (0.44 MB for an 18 GB archive, *measured*), so a
  later frame costs 2 requests.
- **Report:** documents per hour, CPU seconds per page, cost per 1,000 labels, retrieval recall and p95.

Raul keeps his presentation role (NASA evidence, bibliography, claim checking) and reviews gold-answer
citations for Tudor without seeing the private hold-out.

---

## 6. Lane C — Tudor: research workspace & evaluation

**Mission:** every contract a user can meet has a faithful view, NASA originals open at the cited place
straight from NASA, the agent composes answers from a trusted component catalog, the challenge's
questions have direct views, and the harness is measured independently.

| P | ID | Deliverable | Depends on | Done when |
|---|---|---|---|---|
| P0 | **T7** | **Faithful viewers.** Part 1 in PR #24 (typed artifact views, contract coverage). Part 2: originals through `SourceAccess`: PDF.js with a **custom range transport** (size from the manifest; *measured*: page 100 of a 49 MB report rendered from 1.26 MB in 6.2 s) and a highlight layer at the `SourceLocator` bbox; MP4 via `<video>` direct from S3; CFI-G `.ts` via hls.js on M12's byte-range playlist; BASS contour drawn from our `VisualObservationBundle` as an SVG/canvas layer synced by `requestVideoFrameCallback`; NTRS through the proxy; "Open at NASA" always. Never a raw signed URL in a plain `<a>` (PSI forces download). | K1-evidence, M12, R7a | a citation opens the PDF at the bbox; video plays with seek and overlay |
| P0 | **K1-evaluation** | `GoldTask` and the harness-comparison section of `EvaluationResult` (§9) | — | merged third in the K1 order |
| P0 | **T10** | **Pilot gold set (~30 tasks), then grow toward master §24's 120.** Documentary tasks in LitQA2 format with an "insufficient evidence" option, scored on accuracy **and** precision *(lit.)*; citation tasks scored on whether the cited locator supports the statement (ALCE/FACT-style) *(lit.)*; analytical tasks with executable checks *(lit.: ScienceAgentBench, BixBench)*; predictive tasks only through the frozen protocol. Dev set shared with Mircea and Raul; private hold-out of ~20 % in the evaluator vault, never in a builder bundle, run once *(lit.: LAB-Bench)*. Retrieval sub-set (single-hop, multi-hop, figure questions) for R10. Cover all 19 investigations, not only the modelled four. | K1-evaluation | pilot frozen with protocol hash |
| P1 | **T9** | **Challenge views:** conditions matrix (fuel × geometry × O2 × pressure × flow × diagnostics, gaps explicit) and habitat relevance ranking (Moon/rover ≥ 30 % O2 at ≤ 10.2 psi, suit 100 % at 4.3 psi, ISS 21 % at 14.7 psi; "outside tested envelope" and "no partial-gravity data" as explicit statuses). The Atmosphere map moves onto `ConditionsTable`. | R9a | each challenge objective has a view a judge can operate |
| P1 | **T8** | **Catalog-driven answer views** (static generative UI): a registry keyed on the closed `ViewSpec` enum (`source_excerpt`, `series_comparison`, `conditions_matrix`, `atmosphere_map`, `video_overlay`, `release_card`, `artifact_table`); the agent places components through `AnswerBlock::View` and never emits HTML (A2UI's security model *(lit.)*). Linked views: one time/selection store (seconds since `time_origin`) syncing chart, video and PDF. Optional later: A2UI catalog export, MCP Apps cards. | K1-research, M7 | a live answer renders a chart and an excerpt the agent chose, linked |
| P1 | **T11** | **H0/H1 and ablations:** same runtime build, model, effort; ablate each skill, wiki, graph tools, subagents; ≥ 3 seeds, paired statistics with cluster bootstrap, cost and tokens from `TraceBundle`, harness disclosed with every number *(lit.)* | M10, T10 | scorecard with intervals and costs; hold-out run once with the team |
| P1 | **T6** | retire legacy routes: Runs, Lab, Compare, Atmosphere on v1 (M11) and `generated.ts`; one error model | M11 | `api.ts` has no cross-lane DTO |
| P1 | T4, T5 | WSP scoring after I3; demo bundle, deck, video (timed to the event, not to code) | I3, results | — |

**Research tips (Tudor)**
- **Viewers:** read PDF.js's `PDFDataRangeTransport` and text-layer docs, hls.js's byte-range support,
  and `requestVideoFrameCallback` browser support; prototype each against a real signed NASA URL in the
  browser (as in §10) before building UI around it.
- **Generative UI:** read the A2UI 1.0 spec and MCP Apps (SEP-1865) *(lit.)* for the catalog and
  security models; keep our closed `ViewSpec` and adopt their ideas, not their transport, unless a
  concrete need appears.
- **Benchmark design:** read LitQA2 (abstention scoring), ALCE (citation metrics), LAB-Bench (private
  split), and the harness disclosure paper *(lit.)* before writing tasks; pilot 30 tasks, look at
  failures, then scale.
- **Statistics:** paired comparisons on the same tasks need fewer runs; read up on cluster bootstrap for
  tasks grouped by investigation.

**Efficiency and performance (Tudor)**
- **PDF:** custom range transport, `rangeChunkSize` 256 KiB, `disableAutoFetch`, render and text-layer
  only visible pages; lazy-load the PDF.js chunk.
- **Video:** MP4 straight from S3; `.ts` through hls.js on keyframe-aligned segments (even-byte segments
  broke on seek at 2 s, *measured*); load players only when opened.
- **Charts:** uPlot; decimate long series to ~2 points per pixel (min/max) in a worker or server-side.
- **Lists:** virtualize runs and sources (tens of thousands of files across 19 investigations).
- **Caching:** content-addressed artifacts get `staleTime: Infinity`; query keys include data mode,
  ids and versions.
- **Benchmark:** run tasks concurrently up to provider rate limits; reuse traces; report cost per task.
- **Report:** bundle size per route, time to first page of a PDF, seek latency, chart frame time.

---

## 7. Iancu's lane (unchanged) and its touchpoints

Iancu continues I1–I5 (literal WSP for FLEX/ACME/BASS). Touchpoints in this phase: review `ViewSpec`
series semantics for predictions (K1.4) and the units in `ConditionsTable`; write the
`model-qualification` skill content; keep the lockbox for the other 15 investigations when their
evaluation groups are frozen; the atlas (`docs/wsp/atlas/`) is the storage rule Raul implements.

---

## 8. End-to-end milestones (dependency order, not calendar)

1. **Bound answer, live:** M6 + M7 + M12 + T7. A real question on the deployed app gives a submitted,
   validated answer whose citations open the NASA PDF at the cited place.
2. **Conditions across 19 investigations:** R6 + R9a + T9. The comparison matrix and habitat ranking
   work for every flight investigation, gaps explicit.
3. **Agent uses the conditions:** M8 registers `conditions_query`; T8 renders the matrix the agent
   placed. "Which investigations tested O2 ≥ 30 % below 10.2 psi, and what is missing?"
4. **Measured harness:** T10 pilot + M10 + T11. H0 vs H1 on dev, with costs and intervals.
5. **Whole corpus navigable:** R7a/b + R8 + R9b. Every stored document parsed, outlined, labelled.
6. **WSP joins:** I2/I3 → M5 `ModelService` → T4.

---

## 9. Shared contracts: what crosses lanes and how it merges

### 9.1 Principles

1. **Contract wave first, implementation second.** All new cross-lane types land in one small PR per
   family (wave **K1**): Rust type + `validate()` + fixture in `fixtures/contracts/manifest.json` +
   `ignis schemas export` + `npm run gen:types` + consumer stubs. No implementation in a contract PR.
2. **One version bump per family per wave** (`deny_unknown_fields` makes new variants breaking):
   `evidence.v2`, `dataset.v2`, `research.v2`, `evaluation.v2`, each with a reader that still accepts
   durable v1 data.
3. **Cross-family references only through ids and refs** (`ArtifactRef`, `EvidenceRef`, `EntityRef`,
   typed ids, `Digest`); never embed another family's struct. This keeps the three PRs independent.
4. **Fixture-first:** producer and consumer tests read the same fixture; `/api/fixtures/v1` serves it.
5. **Capability-gated:** a tool or service is registered only when configured; `/capabilities` says why.
6. **Trait growth does not break implementors:** new methods in `services.rs` get a default body
   returning `capability_unavailable`.
7. **Generated files are regenerated, never hand-merged.**
8. **Closed catalogs change by contract PR only** (`ViewSpec` kinds, `DocType`, graph relations,
   `SourceAccess` modes).

### 9.2 Wave K1

| # | Type / change | Family (owner) | Producer | Consumers (reviewers) | Status |
|---|---|---|---|---|---|
| K1.1 | `ToolStarted.arguments` (redacted by scope) | `research.rs` (Mircea) | Mircea | Tudor | to write |
| K1.2 | `EventBody::StepRationale { call_id?, text, source: model_explanation }` | `research.rs` | Mircea | Tudor | to write |
| K1.3 | `AnswerSubmission` + refusal reasons (`unknown_evidence`, `unresolved_pointer`, `kind_support_mismatch`, `unpinned_release`); tool effect `compute` | `research.rs` | agent via MCP; Mircea validates | Tudor, Raul | to write |
| K1.4 | `AnswerBlock::View { view: ViewSpec, caption?, claim_ids }`; `ViewSpec` v1: `source_excerpt{evidence}`, `series_comparison{series: [SeriesRef{artifact_id, pointer, role, unit_pointer}], time_origin_pointer}`, `conditions_matrix{table: ArtifactRef}`, `atmosphere_map{table: ArtifactRef}`, `video_overlay{source_id, bundle: ArtifactRef}`, `release_card{release_id}`, `artifact_table{artifact_id, pointer}` | `research.rs` (Mircea writes, **Tudor co-designs**) | agent | Tudor, Iancu | to write |
| K1.5 | `Claim.checked_by: Option<SupportCheck{method, model_id?, verdict, confidence?}>` | `research.rs` | Mircea or Tudor's evaluator | Tudor | to write |
| K1.6 | `EpisodePins.harness: Option<HarnessRef{digest, label, runtime_build, model, effort?}>` replacing `skills_version` | `research.rs` | Mircea | Tudor | to write |
| K1.7 | server-only types into core: `CreateEpisode`, `PostMessage`, `MessageAccepted`, `HostDecision`, `Capabilities`, `ServiceStatus`, `ToolStatus`, `EpisodeAction` | `research.rs` | Mircea | Tudor | to write |
| K1.8 | `TraceBundle` | `research.rs` | Mircea | Tudor | to write |
| K1.9 | `tool_catalog()` entries: `answer_submit`, `sources_outline`, `conditions_query`, `graph_neighbors`, `wiki_read`, `web_search`, `web_fetch` (request/response types stay in their families) | `research.rs` | Mircea | Raul, Tudor | to write |
| K1.10 | `SourceLabel` | `evidence.rs` (Raul) | Raul | Mircea, Tudor | **merged** (#26) |
| K1.11 | `ProvenanceLink` relations for the graph + `GraphNeighborsRequest/Page` | `evidence.rs` | Raul | Mircea, Tudor | **merged** (#26) |
| K1.12 | `ResponseFormat` on search/read | `evidence.rs` | Raul | Mircea | **merged** (#26) |
| K1.13 | `ConditionsQuery` / `ConditionsTable` (cells, envelope) | `dataset.rs` (Raul) | Raul | Tudor, Mircea, Iancu (units) | **merged** (#26) |
| K1.14 | `GoldTask` (`split: dev\|holdout`; hold-out never in builder bundles) | `evaluation.rs` (Tudor) | Tudor | Mircea, Raul | to write |
| K1.15 | `EvaluationRequest.purpose = harness_benchmark` + `HarnessComparison` in `EvaluationResult` | `evaluation.rs` | Tudor | Mircea, team | to write |
| K1.16 | `SourceLocator.figure_id` (only if needed) | `contracts.rs` (Iancu) | Raul | Tudor | optional |
| K1.17 | trait methods with default bodies (outline, labels, access, graph, conditions, wiki, web) | `services.rs` (common) | Raul | Mircea | **merged** (#26) |
| K1.18 | `SourceAccess` (mode, url, expiry, range, cors, landing url); identity stays the `Source` | `evidence.rs` | Raul (mode), Mircea (URL, M12) | Tudor | **merged** (#26) |
| K1.19 | `SourceOutline` (section tree; summaries labelled generated) | `evidence.rs` | Raul | Mircea, Tudor | **merged** (#26) |
| K1.20 | wiki read/patch and web search/fetch request/response types | `evidence.rs` | Raul | Mircea | **merged** (#26) |

**K2 candidates** (not in this wave): a `SourceAccess` mode for byte-range playlists and the media
keyframe index (R11/M12), if the playlist cannot stay server-internal.

### 9.3 Merge order

```text
K1-evidence + K1-dataset (Raul, merged #26)  ──►  K1-research (Mircea)  ──►  K1-evaluation (Tudor)
```

Written in parallel from the same `main`; merged back to back in this order, each rebased on the
previous with `schemas/` and `generated.ts` regenerated. Nobody waits to start writing.

### 9.4 Interfaces after K1

| From → to | Contract | Path |
|---|---|---|
| Raul → Mircea | labels, outline, graph, conditions, wiki, web, source access modes | services → MCP tools, `/api/v1` |
| Raul → Tudor | the same, parse views with bboxes | `/api/v1` (via Mircea), fixtures until then |
| Mircea → Tudor | `answer.completed` with views, tool arguments, rationale, capabilities, harness ref, traces, `SourceAccess` URLs | SSE + `/api/v1` |
| Tudor → Mircea, Raul | dev gold tasks, protocol, retrieval results; never the hold-out | `benchmarks/` |
| Iancu → all | `ModelRelease`, `AnalysisArtifact` | `ModelService` |

---

## 10. NASA data: what the server stores, and how the UI reaches the rest

**Storage rule** (Iancu's atlas, `docs/wsp/atlas/`, NASA's listings of the 19 flight investigations,
each file counted once; NASA's listings repeat 5,701 rows):

| What | Files | Size | On the server? |
|---|---|---|---|
| Documents, tables, analysed data | 636 | 1.63 GB | yes |
| Sensor and settings text in Raw Data | 24,482 | 12.26 GB | yes (per investigation as taken up) |
| Images, video and their archives | 35,446 | 13.44 TB | **no**: manifest entry, opened at NASA |
| Other archives outside Raw Data | 236 | 12.66 GB | no |

The server keeps ~13.89 GB of NASA's ~13.47 TB, plus listings (~30 MB) and derived data. CLD Flame,
FLEX-2 and SLICE file documents under Raw Data, so some documents sit in the sensor-text list; both lists
are stored. PSI-10's licence field is blank: verify.

**Access, measured 2026-11-14** (from the dev machine and the built-in browser):

| Host | Range | CORS | Notes | Mode |
|---|---|---|---|---|
| PSI geode download endpoint | n/a | only `psi.nasa.gov` | 302 / `redirect=false` → signed S3 URL, ~1 h lifetime | backend resolves (M12) |
| PSI bytes, `psi-curation-prod` and `psi-repo-prod` S3 buckets | 206 | `*`, preflight allows `range` | only `ETag` exposed to scripts; curation bucket forces download | `direct` |
| NTRS downloads | 206 | none | `X-Frame-Options: SAMEORIGIN`; `.txt` full text | `proxy` + link out |
| NASA Image and Video Library assets | 206 | `*` | permanent unsigned URLs; hardware videos, not run videos | `direct` |
| YouTube | — | — | — | embed |

**Browser and range tests**

| Test | Result |
|---|---|
| PDF.js default loader on a 49 MB PSI PDF | one full GET (status 200); `Accept-Ranges` invisible to scripts, so no range mode |
| PDF.js with a custom `PDFDataRangeTransport` (size from manifest) | 144-page report opened; page 100 rendered with text in 6.2 s from **1.26 MB (2.6 %)**, 5 range requests |
| ZIP64 member read, 18.13 GB CFI-G archive (3,330 entries) | member past 4 GiB read and decompressed with **5 requests, 0.54 MB**; frames inside are `.tif` (convert server-side) |
| CFI-G `.ts` codecs (4 random files) | H.264 video + a private data stream; 15–996 MB per file |
| `.ts` with mpegts.js, direct from S3 | plays (640×480, frames decoding, no errors); duration reported as `Infinity`, **seek fails** |
| `.ts` with hls.js on a byte-range playlist (duration from first/last PTS) | duration known, seek to 120 s works with 8 s segments; with 2 s even-byte segments first frames in 0.9 s but a fatal buffer-append error on seek → segments must be **keyframe-aligned** (R11 index) |

**Consequences:** no media mirror and no R2 copy of NASA bytes; identity never depends on a signed URL
(a citation names the `Source`: geode URL + file version + hash); archives are read member by member;
geode endpoints are undocumented, so calls are throttled and response samples kept.

---

## 11. Sources

Verified against arXiv or primary pages on 2026-11-14 unless marked; "secondary" = third-party
summary only.

**Harness**
- Recursive Language Models, arXiv 2512.24601; reproduction 2603.02615 (depth 1 helps aggregation,
  hurts simple retrieval, depth 2 worse, 25–100× latency).
- Meta-Harness, arXiv 2603.28052 (proposer reads full traces). ACE 2510.04618 (itemized deltas). GEPA
  2507.19457. DSPy 2310.03714. ADAS 2408.08435, AFlow 2410.10762, Darwin Gödel Machine 2505.22954.
- Agent Skills (Anthropic, Oct 2025; agentskills.io, Dec 2025; adopted by Codex). SkillsBench
  2602.12670 (+16.6 points with curated skills; focused beats exhaustive).
- Anthropic, "Effective context engineering" and "Writing effective tools for agents" (Sep 2025). MCP
  description studies 2602.14878, 2602.18914. "Evaluating AGENTS.md" 2602.11988. Harness disclosure
  2605.23950.
- PaperQA2/LitQA2 2409.13740; ALCE 2305.14627; DeepResearch Bench 2506.11763; ScienceAgentBench
  2410.05080; BixBench 2503.00096; DiscoveryBench 2407.01725; SciCode 2407.13168; LAB-Bench 2407.10362;
  Robin 2505.13400.

**Knowledge**
- GraphRAG 2404.16130; LightRAG 2410.05779; HippoRAG 2 2502.14802; RAPTOR 2401.18059; KAG 2409.13731;
  GraphRAG-Bench 2506.05690; RAG vs GraphRAG 2502.11371 (secondary cost figures).
- ColPali 2407.01449; ColBERTv2 2112.01488; PLAID 2205.09707.
- PageIndex (VectifyAI): section tree + LLM tree search; 98.7 % FinanceBench is a vendor claim
  (*unverified*); pages/sections, not bboxes.
- Jev (TypeSafe): typed decision model with confidence; versions and prices differ across listings
  (*unverified*).
- Parsers: Docling (MIT, bbox), GROBID (Apache, TEI coords), MinerU (AGPL), Marker (GPL), PaddleOCR-VL
  2510.14528, dots.ocr; olmOCR 2 2510.19817 and Nougat have no bboxes.
- Muse Spark 1.3 (Meta): 1M context, PDF input; Contributor tier reported $0.10/$0.20 per M tokens
  (aggregators, *unverified*).

**Workspace**
- A2UI 1.0 (Google, Nov 2026); MCP Apps SEP-1865; AG-UI; OpenUI (thesys); Vercel AI SDK tool parts.
- PDF.js `PDFDataRangeTransport`; hls.js (EXT-X-BYTERANGE); mpegts.js.
- NASA: PSI repository (DOIs, CC0); NTRS OpenAPI (sti.nasa.gov); NASA Image and Video Library API;
  challenge page spaceappschallenge.org/2026/challenges/flame-in-freefall; judging guide (2025).

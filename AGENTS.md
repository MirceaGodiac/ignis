# AGENTS.md — operating guide for coding agents in IGNIS

IGNIS is a scientific research workspace for NASA microgravity-combustion data (FLEX, ACME, BASS):
a researcher asks a documentary, analytical or predictive question and gets an answer bound to
openable sources, executed computations, model releases with their qualification state, and the
limits that change the interpretation. Read this file fully; follow the links only for your task.

## Architecture in one screen

| Layer | What | Where |
|---|---|---|
| Agent runtime | **Codex App Server** (pinned build, JSON-RPC over private stdio) or **Claude Code** (headless, stream-json; ADR 17) owns the conversation and the model/tool loop, chosen with `IGNIS_AGENT_RUNTIME` | `ignis_research::runtime` (`AgentRuntime`, `CodexBridge`, `ClaudeBridge`; Mircea) — offline tests plus an opt-in real turn |
| Backend | Rust + Axum + Tokio: API, SSE events, jobs, artifacts, scope, MCP transport (`rmcp`) | `crates/ignis-server` (`/api/v1`), `crates/ignis-research` (state store, jobs, artifacts, isolated runner), `crates/ignis-mcp` (IGNIS tools over MCP) |
| Scientific services | knowledge (search/read/wiki), datasets, models (FLEX/ACME/BASS, literal WSP), evaluation | `ignis-evidence`, `ignis-data`, `ignis-vision`, `ignis-flex/acme/bass`, planned `ignis-wsp`, `ignis-lab/src/eval` |
| Shared contracts | the single source of truth for every cross-lane type | `crates/ignis-core` (`contracts.rs` + `contracts/*.rs`) |
| Frontend / edge | React + TypeScript + Vite; Cloudflare static assets, a small Worker, R2, Access, Tunnel | `web/`, `infra/worker` |
| Hosting | Google Compute Engine VM as the Linux origin (Compose + systemd, `cloudflared` → Axum; no Nginx); domain from the Space Apps offer (registrar Porkbun), served by Cloudflare | `infra/` (runbook: `infra/README.md`; provisioned 2026-11-14 as `ignis-ardealu.app`) |

The agent runtime (Codex or Claude Code) owns the agent loop. IGNIS owns episodes, scope, jobs, artifacts, data, models and evaluation.
**Do not write a second agent loop, a compactor, a universal LLM gateway, or fork Codex** without an
ADR that demonstrates a limitation configuration/MCP/adapters cannot solve. Do not move the backend
to Go or another framework. Decisions and their rationale: `docs/implementation/ARCHITECTURE.md`.
Full design (Romanian, authoritative for direction): `docs/design/IGNIS_MASTER_V2.md`.

**Authority order:** explicit current instructions → the master design → implemented contracts and
their tests → historical documents. Code is the authority for what works *now*; never claim a planned
component exists. `docs/IGNIS_V3.1.md` is historical (equations, early infrastructure); it does not
govern WSP, ownership or the runtime.

## Where does my change go?

1. Find your lane in `docs/implementation/WORKBOARD.md` and read your `HANDOFF_<NAME>.md`.
2. Ownership of every directory and shared file: `docs/implementation/INTEGRATION.md`. Edit only what
   your lane owns. Shared files (`ignis-core`, root `Cargo.toml`, `Cargo.lock`, `ignis-cli/src/main.rs`,
   `schemas/`, `web/src/contracts/generated.ts`) change only by the rules there.
3. A type that crosses a lane boundary lives in `ignis-core/src/contracts/` — never a local copy, a
   hand-written TS interface or a parallel JSON Schema. Contract rules: `docs/implementation/CONTRACTS.md`
   and `crates/ignis-core/AGENTS.md`.
4. Domain logic lives in a library crate behind a service trait (`ignis_core::services`); HTTP, MCP and
   CLI are thin adapters over the same call. Never re-implement search, physics or scoring in a wrapper.

## Non-negotiable rules

**Data, results and interpretation stay separate.** An original value keeps its raw value, raw unit,
SI value and locator. IGNIS computations are artifacts with inputs and code identity. Model outputs
are `model_output`, never measurements. LLM interpretation is a claim of kind `interpretation`.
Missing, conflicting and not-applicable are explicit statuses or documented absent fields, never zero.
Experimental times are seconds since an explicit `time_origin`; system times are RFC 3339 UTC.

**Literal weight-space programming (WSP).** For FLEX/ACME/BASS WSP releases the author model
(Fable/Astra) writes the numbers: a `HandwrittenWeightPatch` of literal `set` / `set_box` / `fill_box`
assignments against a `ModelSpec`. No code, expressions, seeds, random init, solvers, `.npy` without a
literal patch, or optimizer hidden in preprocessing or the forward pass (no gradient/MAP/NUTS/least
squares at request time). W0 (one-shot, no feedback) and W1 (revision with development feedback) are
separate experiments. A literal release is `trained_on = not_trained`, `state = experimental`, carries a
`construction` manifest; the registry refuses anything else (`ModelRelease::check_construction`).
Releases never need a NASA-trained parent. Until FLEX/ACME/BASS releases exist, `ignis serve` keeps the model families off
(`--enable-models` / `IGNIS_ENABLE_MODELS`, default off): contracts and tool descriptors stay
published, the model tools are not registered and `/api/v1/capabilities` says why. The legacy `ignis lab` loop (fit → probe → edit → resume)
still works for fitted baselines, but it is not WSP and its results are never presented as WSP.

**Protect the test.** Never pass `--final-test`, read `holdout/`, score or compare on the test
partition, or show final-test outcomes to an author. Only an evaluation scope may read the test
partition (`ExecutionScope::require_partition`). Builder bundles never contain test outcomes. A change
guided by test results turns the test into development; renaming files or reseeding does not undo it.
Every reveal is logged in `runs/evaluation/reveal_log.jsonl`. The final test runs once, on a frozen
protocol, by the evaluator (Tudor's lane), with the team.

**Provenance and honesty.** Fixtures are `origin = synthetic_fixture` and are never presented as NASA
results; nothing computed from them qualifies a model. A preview is `partial = true` and points at the
full artifact. A degraded retrieval says so. Do not claim "validated", "superior harness" or "complete
corpus" because CI passed. Do not state latency/throughput you did not measure. Never edit
`artifacts/models/` (releases are immutable and content-addressed; changes are new releases).

**Scope is host-derived.** Services receive `&ExecutionScope` as a separate argument. Never accept a
scope, workspace id or partition grant from a request body, tool argument, prompt or document.

## Engineering standards

- **Correctness by construction:** typed ids, `Finite` numbers, closed enums, `deny_unknown_fields`,
  invariants in `validate()` functions next to the type. Fail closed; never skip invalid input silently.
- **Performance by design:** move references (`ArtifactRef`) not payloads; stream large bytes; Parquet
  for numeric tables; no CPU work on Tokio reactor threads (`spawn_blocking`/rayon, bounded); cache keys
  include every version that changes the answer (scope, corpus snapshot, release id, prefix
  `input_hash`, settings hash); no whole-corpus loads into RAM.
- **No arbitrary limits** on documents, tokens, steps or result size. Use pagination, streaming,
  checkpoints and real resource budgets. Security and physical limits stay.
- **Small surface, no speculation:** no framework, microservice or abstraction without a concrete
  consumer. Do not rewrite mature components for novelty.
- **Tests guard invariants, boundaries and expensive failure modes** — schema compatibility, payload
  validation, release meaning, test isolation, citation/unit correctness, replay. No getter tests,
  line-by-line mirrors, valueless snapshots, heavy mocks or coverage padding. Keep existing valuable tests.

## Commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p ignis-core                 # contracts, schema drift, fixtures, isolation, registry
cargo test -p <crate-you-changed>
cargo run -p ignis-cli -- schemas export # after any contract change (writes schemas/)
npm ci --prefix web                      # once
npm --prefix web run gen:types           # after schemas export
npm --prefix web run check:types         # fails if generated.ts is stale
npm --prefix web run typecheck
npm --prefix web run build
```

Nothing expensive or networked runs without an explicit `--execute`; live-API tests are separate
from offline tests and need explicit authorization.

## Delivering a PR

One task id from the workboard per PR, on a short-lived branch from `main` named `feat/<lane>-<task>` (deleted on merge). Phase 2 lanes, priorities and shared contracts: `docs/implementation/LANES_V2.md`. The description lists: task id,
contracts and versions touched, paths changed, behaviour added, fixtures/artifacts proving it, the
commands you actually ran with their results, limitations, and the reproduce command. Never write
PASS for something you did not run. A contract change follows `docs/implementation/CONTRACTS.md`
(producer + fixture + consumer + regenerated schemas/TS in the same PR). Do not merge to `main` without
the integrator's decision (`docs/implementation/INTEGRATION.md`).

## Legacy lab and evaluation CLI

The existing commands (`ignis lab …`, `ignis infer …`, `ignis eval …`, `ignis fit …`) are documented
in `docs/GUIDE.md` §5–§8 and `crates/ignis-lab/src/lab/edits.rs`. They operate on fitted releases and
the sealed validation chain; the test-partition rules above apply to them unchanged.

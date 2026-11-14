# Integrare, ownership și flux de lucru

Scop: patru agenți lucrează în paralel fără să editeze aceleași fișiere de nucleu, iar munca lor se
unește prin contractele C0, nu prin conversații. Regula de bază: **fiecare fișier are un singur owner
de scriere**; ceilalți propun prin PR mic sau prin funcția de înregistrare a lane-ului lor.

## Worktree-uri și branch-uri

După merge-ul C0 în `main`, fiecare pornește din același commit:

```bash
git fetch origin
git worktree add -b feat/handwritten-science ../ignis-iancu  origin/main
git worktree add -b feat/agent-backend      ../ignis-mircea origin/main
git worktree add -b feat/scientific-data    ../ignis-raul   origin/main
git worktree add -b feat/research-product   ../ignis-tudor  origin/main
```

PR-uri mici și dese către `main` (un task din `WORKBOARD.md` per PR). Rebase pe `main` înainte de
merge. Nu se țin patru versiuni de `main` până la final. Niciun agent nu lucrează în checkout-ul altuia.

## Harta ownership-ului

| Cale | Owner de scriere | Note |
|---|---|---|
| `crates/ignis-core/src/contracts/common.rs`, `services.rs`, `schema.rs`, `registry.rs`, `hashing.rs` | comun — PR de contract cu review din fiecare lane consumator | vezi `CONTRACTS.md` |
| `crates/ignis-core/src/contracts.rs` (modelul științific), `prefix.rs`, `views.rs`, `analysis.rs`, `expert.rs`, `npz.rs`, `kernels.rs`, `units.rs` | Iancu (semantic) | Raul review pentru `Observation`/`Source`/locatori |
| `crates/ignis-core/src/contracts/research.rs` | Mircea | Tudor review |
| `crates/ignis-core/src/contracts/evidence.rs`, `dataset.rs` | Raul | Iancu/Tudor review numeric |
| `crates/ignis-core/src/contracts/wsp.rs` | Iancu | Tudor review evaluabilitate |
| `crates/ignis-core/src/contracts/evaluation.rs` | Tudor | review științific înainte de freeze |
| `crates/ignis-core/src/resources.rs` | Mircea | job ledger / profile de resurse |
| `crates/ignis-wsp/` (nou) | Iancu | assembler literal, forward strict, specificații |
| `crates/ignis-flex/`, `ignis-acme/`, `ignis-bass/`, `ignis-science/`, `ignis-train/`, `ignis-accel/` | Iancu | baselines și numerica existentă |
| `crates/ignis-lab/src/lab/`, `crates/ignis-lab/src/infer.rs` | Iancu | authoring, probe, lineage, forecast de cohortă |
| `crates/ignis-lab/src/eval/`, `access.rs` | Tudor | vault, seal/reveal, scoring, compare, report, lockbox |
| `crates/ignis-lab/src/util.rs`, `lib.rs` | owner-ul funcției atinse; lib.rs: Iancu | modificări mici |
| `crates/ignis-data/` (tot: `sources/` = discovery, `families/`, `cohort.rs`, `views.rs`, `review.rs` = date numerice) | Raul | împărțirea discovery/numeric e internă lane-ului |
| `crates/ignis-evidence/`, `crates/ignis-knowledge/` (nou, doar dacă separarea o justifică) | Raul | un singur serviciu de căutare/citire |
| `crates/ignis-vision/`, `sidecar/sam2/` | Raul | extragerea observației; modelul BASS e al lui Iancu |
| `crates/ignis-server/`, `crates/ignis-research/` (nou), `crates/ignis-mcp/` (nou) | Mircea | API, SSE, joburi, artefacte, Codex bridge, MCP |
| `crates/ignis-cli/src/main.rs`, root `Cargo.toml` (`[workspace.dependencies]`), `Cargo.lock` | Mircea (integrator) | colegii expun funcții/enum-uri de subcomandă; patch ≤ 10 linii |
| `infra/` (nou), `configs/resources/`, `tools/runpod/` | Mircea | Tudor contribuie doar build-ul frontendului |
| `configs/models/` | Iancu | |
| `configs/evidence/`, `configs/vision/`, `data/` | Raul | |
| `configs/evaluation/`, `benchmarks/` (nou), `docs/demo/` (nou) | Tudor | |
| `web/` (fără `src/contracts/generated.ts`) | Tudor | ceilalți livrează fixture-uri, nu ecrane |
| `schemas/*.json`, `web/src/contracts/generated.ts` | **generate** | doar `ignis schemas export` + `npm run gen:types` |
| `fixtures/contracts/<familie>/` | ownerul familiei | `manifest.json`: PR cu review comun |
| `docs/implementation/HANDOFF_<NUME>.md` | persoana respectivă | |
| `AGENTS.md`, `docs/implementation/{ARCHITECTURE,CONTRACTS,INTEGRATION,WORKBOARD}.md`, `docs/TEAM.md`, `docs/REPOMAP.md` | Mircea (integrator), review Iancu | se actualizează în PR-ul care schimbă realitatea descrisă |

## Zonele care se suprapun — rezolvate

- **`ignis-core`**: fiecare familie are fișierul ei; nimeni nu atinge fișierul altei familii. Un PR de
  contract nu conține implementare de lane.
- **`ignis-cli/src/main.rs`**: fiecare crate expune `pub enum XCmd` (clap `Subcommand`) și
  `pub fn run(cmd) -> anyhow::Result<()>`, ca azi. Owner-ul crate-ului cere integratorului o variantă nouă
  în `enum Cmd` + un braț în `match` (patch ≤ 10 linii). Serviciile pentru HTTP/MCP se construiesc
  într-o singură funcție de composition root (Mircea) din constructorii publici ai lane-urilor.
- **Workspace `Cargo.toml`**: `members = ["crates/*"]` — un crate nou nu cere editare root. Dependențele
  noi se pun în `Cargo.toml`-ul crate-ului cu versiune explicită și motiv în PR; promovarea în
  `[workspace.dependencies]` o face integratorul.
- **`Cargo.lock`**: la conflict se ia versiunea din `main`, apoi `cargo check --workspace` regenerează
  intrările lipsă; nu se editează manual.
- **`ignis-data`**: tot crate-ul e al lui Raul. Iancu/Tudor consumă `DatasetBundle`, nu modulele interne.
- **`ignis-lab`**: `src/lab/` + `infer.rs` = Iancu (authoring); `src/eval/` + `access.rs` = Tudor
  (scoring, vault). Nu se mută funcții peste graniță fără review de ambele părți. Iancu nu schimbă
  labels/splits/metrici; Tudor nu schimbă modelele.
- **`ignis-vision` vs `ignis-bass`**: Raul produce `VisualObservationBundle` (calibrare, contur,
  `units_status`); Iancu consumă exact acea observabilă (`require_metric()` pentru forecast metric).
- **`schemas/` și TS**: generate; un conflict de merge se rezolvă regenerând din Rust-ul integrat.

## Interfețe între lane-uri (cine produce ce)

| De la → la | Contract | Unde |
|---|---|---|
| Raul → Iancu, Tudor | `DatasetBundle` (Parquet + unități + `time_origin` + partiții), `VisualObservationBundle` | `DatasetService` |
| Raul → Mircea, Tudor | `SearchPage`, `ReadResult`, `EvidenceRef`, `WikiRevision`, `ProvenanceLink` | `KnowledgeService` |
| Iancu → Mircea, Tudor | `ModelRelease` (+ `construction`), `ModelSpec`, `AnalysisArtifact` | `ModelService` |
| Mircea → Tudor | `ResearchEpisode`, `ResearchEvent` (SSE), `JobRef`, `ToolResult`, `ArtifactRef`, `ResearchAnswer` | HTTP `/api/v1` |
| Tudor → Iancu | `EvaluationResult` cu `purpose = development_feedback` | `EvaluationService` (proces separat) |
| Tudor → toți | scorecards, raport de integrare | `benchmarks/`, `docs/demo/` |

Endpointuri `/api/v1` (implementate în M2; OpenAPI generat la `GET /api/v1/openapi.json`) → contracte:

| Endpoint | Request | Response |
|---|---|---|
| `GET /api/v1/capabilities` | — | serviciile configurate, tool-urile din `tool_catalog()` care pot rula, runtime-ul agentic (`agent_runtime`, manifestul `runtime_build`) |
| `POST /api/v1/episodes` | goal + `client_request_id` (+ `model_releases`, verificate; corpusul îl pin-uiește hostul) | `ResearchEpisode` (202, și la retry) |
| `GET /api/v1/episodes` | — | `ResearchEpisode[]` din workspace-ul apelantului |
| `POST /api/v1/episodes/{id}/messages` | text + `client_request_id` | 202 (`seq`, `duplicate`); cu runtime-ul configurat pornește un turn Codex (M1) |
| `POST /api/v1/episodes/{id}/approvals/{approval_id}` | `decision`: `approved` / `denied` | 202; aprobarea cere capabilitatea efectului; 404 după rezolvare (M1) |
| `GET /api/v1/episodes/{id}` | — | `ResearchEpisode` |
| `GET /api/v1/episodes/{id}/events?after=<seq>` | — (sau `Last-Event-ID`) | SSE de `ResearchEvent`; 410 dacă jurnalul nu acoperă cursorul |
| `POST /api/v1/episodes/{id}/{pause,cancel,resume}` | — | `ResearchEpisode` |
| `GET /api/v1/artifacts/{id}` (+ `?offset&len`, `?pointer`), `/artifacts/{id}/meta` | `ArtifactReadRequest` | bytes / `InlinePayload`; `ArtifactRef` |
| `POST /api/v1/uploads/initiate` | media type, lungime exactă, sha256 (+ `client_request_id`) | bilet: unde se face `PUT` |
| `PUT /api/v1/uploads/{id}/content` | bytes brute (verificate la lungime și sha256) | bilet `received` |
| `POST /api/v1/uploads/complete` | `upload_id` | `ArtifactRef` (idempotent) |
| `POST /api/v1/search` | `SearchRequest` | `SearchPage` |
| `GET /api/v1/sources/{id}/pages/{page}` | `ReadRequest` | `ReadResult` |
| `GET /api/v1/models`, `/models/{id}` | — | `ModelRelease[]`, `ModelRelease` |
| `POST /api/v1/analyses` | `ForecastRequest` (+ `Idempotency-Key`, `?episode_id`) | `ToolResult` (`accepted` + `JobRef` sau `completed`) |
| `GET /api/v1/jobs/{id}`, `POST /api/v1/jobs/{id}/cancel` | — | `JobRef` |
| `GET /api/v1/openapi.json` | — | OpenAPI generat din handler-e (`aide` peste schemars) |

Cu `--dev`, aceleași rute sunt servite și la `/api/fixtures/v1`, din fixture-urile C0
(`fixtures/contracts/`, `origin = synthetic_fixture`): episodul fixture în fiecare workspace, căutare și
citire din fixture, release-ul W0, forecast refuzat (job eșuat). Serviciul neconfigurat răspunde
`503 capability_unavailable`; joburile se revendică interactive > evaluare > ingestie.

Rutele vechi `/api/...` rămân adaptoare peste aceleași servicii până la migrare; rutele admin,
reload, seal/reveal și authoring nu sunt publice implicit.

## Probleme găsite în audit, de rezolvat în lane-uri

| Problemă (cod actual) | Owner | Task |
|---|---|---|
| `GET /experiments/{id}/observations` și `/series` acceptă orice `cutoff_s` și întorc observațiile run-urilor de test (`crates/ignis-server/src/api.rs` ~285–337); `/overlay` întoarce contururi (~363–397) | Mircea (cu scope) + Tudor (politica lockbox) | M2 — **rezolvat**: scope public = doar `train`; run-urile validation/test → 403 `scope_denied`, sumarele ascund numărul/durata (`tests/leaks.rs`) |
| `/seal` ia partiția de la client dacă split-ul lipsește din registry; `/reveal` pe validare ocolește `reveal_log`/`VaultAccess` (`api.rs` ~829–875) | Mircea + Tudor: un singur vault (`ignis-lab/src/eval/vault.rs`) | M2/T2 — **parțial**: partiția nu mai vine de la client; `/seal`, `/reveal`, `/admin/reload` doar cu `--internal-routes`; fiecare reveal (acceptat sau refuzat) intră în `runs/evaluation/reveal_log.jsonl`. Unificarea cu `VaultAccess` rămâne în T2 |
| `/evidence/search` și `/brief` nu aplică `firewall::strip_for_sealed`; serverul are un al doilea motor FTS (`ignis-server/src/evidence.rs`) | Raul (`KnowledgeService`), Mircea șterge duplicatul | R3/M3 — filtrarea aplicată din M2 (aceeași regulă conservatoare); duplicatul se șterge în M3 |
| `ignis-evidence` întoarce `serde_json::Value` neschimbat, fără paginare | Raul | R3 |
| `ignis-vision` scrie contururi `pixel_only` ca `ContourObservation` (semantică de metri) cu unitatea doar în `derivation.notes` | Raul | R5 |
| Forecast FLEX folosește Newton/Laplace (`ignis-flex/src/inference.rs` `newton_mode`, `importance_locals`) și least squares (`prefix_d2_estimate`); BASS operator face MAP L-BFGS local (`ignis-bass/src/model.rs` `local_inference`) | Iancu: forward WSP separat, fără aceste căi | I2 |
| Middleware 30 s / 1 MiB aplicat global (`ignis-server/src/lib.rs`) | Mircea: limitele de transport nu sunt limite ale cercetării | M2 — **rezolvat**: limitele sunt per router; SSE fără timeout |
| Test intermitent `ignis-data --test auto::fabricated_locator_is_rejected_after_one_retry` (`attempts` 1 în loc de 2, ~1 din 3–6 rulări; reprodus și pe `main` 90afacf, deci preexistent) | Raul | R1 |
| `cargo fmt --all -- --check` nu trecea pe `main` (stil vechi); repo-ul e formatat o singură dată în C0, înainte de împărțire | Mircea menține gate-ul | — |

## Gate-uri de merge

Rulate local (nu există CI configurat încă; Mircea îl adaugă în M4):

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p ignis-core
cargo test -p <crate-urile atinse>
npm --prefix web run check:types
npm --prefix web run typecheck      # și build, dacă web/ e atins
```

Testele de izolare a testului, citare, unități, WSP strict și replay sunt criterii de acceptanță, nu
opționale. Testele live (API, credite) sunt separate și cer autorizare explicită. Merge în `main`:
integratorul (Mircea) după review-ul ownerului; schimbările de contract au nevoie și de consumator.

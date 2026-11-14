# Workboard

Unități de integrare, nu estimări de ore. Fiecare task are un owner și un rezultat testabil. După C0,
liniile pornesc simultan; dependențele de mai jos sunt reale (de date/contract), nu de calendar —
până la livrarea dependenței se lucrează pe fixture-urile din `fixtures/contracts/`.

| ID | Owner | Livrabil | Dependență | Stare |
|---|---|---|---|---|
| C0 | Iancu | contracte, servicii, scheme + TS, fixture-uri, teste de contract, AGENTS.md, handoff-uri | — | **integrat** în `main` |
| I1 | Iancu | `ModelSpec` complet FLEX și ACME (normal/inverse): parametri fizici, noise/hazard, stări locale, layout, fără defaulturi ascunse | C0 | de făcut. Contractele (`ModelSpec`, `HandwrittenWeightPatch`, `ModelRelease`) sunt publicate; în harness familiile sunt oprite (`--enable-models` / `IGNIS_ENABLE_MODELS`, implicit off) |
| I2 | Iancu | `ignis-wsp`: assembler literal (spec + patch [+ parent] → tensori → release prin registry), forward strict, audit al rutei reale de forecast | I1 | de făcut; până atunci `ignis serve` nu injectează `ModelService`, iar `models_inspect`/`models_forecast` raportează „disabled” în `/api/v1/capabilities` |
| I3 | Iancu | candidați W0 independenți Fable / Astra, manifeste, model cards | I2 | de făcut |
| I4 | Iancu | W1 cu feedback de development; candidații nereușiți păstrați | I3 + T2 + R2 | de făcut |
| I5 | Iancu | BASS WSP pe observabila calificată, prin `ModelService` | I2 + R5 | de făcut (după I2 + R5); oprit în harness ca I1–I4 |
| M1 | Mircea | Codex pin + preflight, bridge stdio, handshake, approvals, mapping thread/turn/item, turn fixture | C0 | **integrat** în `main`: bridge Codex 0.169.0, preflight, approvals, mapping, recovery; turn real verificat (login ChatGPT, opt-in) |
| M2 | Mircea | `/api/v1` HTTP + SSE reluabil, state store (SQLite WAL), `ArtifactStore`, joburi cu lease; închiderea scurgerilor de test din API-ul vechi; OpenAPI generat | C0 | **integrat** în `main`: `/api/v1`, SSE reluabil, store, joburi, artefacte, uploads, OpenAPI, fixture mode `/api/fixtures/v1` |
| M3 | Mircea | MCP (`rmcp`) peste serviciile colegilor; runner izolat; fără shell pe host | M1 + M2 | **integrat** în `main`: `ignis-mcp` (dispatcher unic + rmcp pe loopback), `jobs_get`, runner izolat + `analysis_execute`. Verificat sub gVisor pe VM-ul GCE (2026-11-14) |
| M4 | Mircea | Cloudflare/GCE (origin pe VM Google Compute Engine), auth, secrets, backup/restore, CI | M2 + M3 | **integrat** în `main`: CI, JWT Access, R2 `ArtifactStore`, `ignis state` backup/restore, `infra/`. **Provizionat** 2026-11-14: `ignis-ardealu.app` (Access, Worker, Tunnel, R2), VM GCE `ignis-origin`, gVisor verificat, verificări live în `infra/README.md` §Status. Rămân: exercițiu de restore (RPO/RTO măsurate), aprobare refuzată exersată live |
| M5 | Mircea | integrare live, cancel/resume/reconciliation, observabilitate | R3 + I2 + T1 | pornit: `ignis serve` injectează `KnowledgeService` (R3) și `DatasetService` (R2) când datele lor există; rămân integrarea live, observabilitatea și `ModelService` (după I2) |
| R1 | Raul | manifest de corpus; un document primar + parser + locator verificat | C0 | **integrat** în `main` (cu R3) |
| R2 | Raul | `DatasetBundle` FLEX/ACME: identitate, unități, timp, calitate | C0 | **integrat** în `main`: `DatasetBundle` FLEX/ACME + `DatasetService`. Bundle-ul de evaluator e pasul lui Tudor |
| R3 | Raul | `KnowledgeService`: hybrid retrieval + reader, aceeași logică pentru CLI/HTTP/MCP | R1 | **integrat** în `main`: `KnowledgeService` (lexical-only fără `DEEPINFRA_API_KEY`, declarat). Ruta veche `/api/evidence/search` folosește încă motorul FTS din `ignis-server` |
| R4 | Raul | web discovery cu snapshot, wiki (CAS), `ProvenanceLink`, corpus snapshot | R3 | de făcut |
| R5 | Raul | BASS: clip→run, calibrare, contur, `VisualObservationBundle` | R2 (nu blochează R3) | **integrat** în `main`: bundle-uri vizuale BASS (`dataset.v1` `ImageContourObservation`), fără calibrare metrică |
| T1 | Tudor | UI pe fixture-urile C0: chat/SSE, evidence viewer, artefacte, stări pending/failed/inconclusive/experimental | C0 | **integrat** în `main` |
| T2 | Tudor | evaluator pe fixture → `EvaluationResult` determinist; protocol freeze; scorecards | C0 | **integrat** în `main`: evaluator pe fixture, protocol freeze, scorecards. Rămâne `CaseSource` din vault (R2) |
| T3 | Tudor | UI live + benchmark retrieval/răspuns/calcul | M2 + R3 | partea 1 **integrată** (UI live pe `/api/v1`); benchmark-ul (partea 2) rămâne |
| T4 | Tudor | scoring WSP; harness H0/H1, rezultate pereche | T2 + I3 + M5 | de făcut (după I3) |
| T5 | Tudor | demo bundle, deck/video, export verificabil | T3 + rezultate admise | pornit: animația Stageshow pentru deck |

### Faza 2 — trei lane-uri paralele (detalii, motivare și surse: [LANES_V2](LANES_V2.md))

Lane-ul lui Iancu (I1–I5) nu se schimbă. Contractele noi intră întâi, ca val **K1** (un PR per
familie, cu fixture, fără implementare), în ordinea din `LANES_V2.md` §5.3.

| ID | Owner | Livrabil | Dependență | Stare |
|---|---|---|---|---|
| K1 | Raul → Mircea → Tudor | val de contracte: `evidence.v2` + `dataset.v2` (Raul), `research.v2` (Mircea), `evaluation.v2` (Tudor); o singură creștere de versiune per familie, reader pentru datele v1 durabile | — | de făcut |
| M6 | Mircea | harness bundle v1 (`harness/`): instrucțiuni operaționale, skills (agentskills.io), secțiunea de tool-uri generată din tool-urile înregistrate; injectat în Codex și Claude Code; digest în `EpisodePins.harness` | K1 | de făcut |
| M7 | Mircea | `answer_submit`: `AnswerSubmission` → validare pe host (evidence întoarsă în episod, pointeri rezolvați, release-uri pin-uite) → `answer.completed`; blocuri `View` | K1, M6 | de făcut |
| M8 | Mircea | ergonomia tool-urilor: argumente în `tool.started`, `step.rationale`, tipuri server mutate în core, `response_format` + paginare, audit descrieri; înregistrarea tool-urilor noi ale lui Raul | K1 | de făcut |
| M9 | Mircea | context în mediu (artefacte montate în `analysis_execute` prin ref), delegare depth-1 prin subagenții runtime-ului, scope derivat per copil | M6 | de făcut |
| M10 | Mircea | `TraceBundle` + `ignis harness run --variant`; optimizare offline doar pe dev (stretch) | K1, T10 | de făcut |
| M11 | Mircea | rute v1 pentru runs/datasets/conditions, ca UI-ul să renunțe la `/api/experiments*` | K1, R9 | de făcut |
| R6 | Raul | catalog PSI pentru toate cele 26 de investigații de combustie + NTRS, drepturi per document, media leneșă | — | de făcut |
| R7 | Raul | parsare cu coordonate: Docling + GROBID, bbox normalizat top-left, parse views versionate | R6 | de făcut |
| R8 | Raul | `SourceLabel`: clasificare în batch, două treceri, gold set de ~100 documente; Muse Spark Contributor doar pe documente publice | R7, K1 | de făcut |
| R9 | Raul | graf tipizat de combustie (SQLite, `ProvenanceLink`), seed din CSV-urile PSI; servicii `conditions_query` și `graph_neighbors` | R6, R8, K1 | de făcut |
| R10 | Raul | upgrade-uri de retrieval doar pe baza benchmark-ului (Tantivy, ColQwen, PPR); calibrare BASS; bundle-ul de evaluator | T10 | de făcut |
| T6 | Tudor | Runs/Lab/Compare/Atmosphere pe rute v1; fără DTO-uri scrise de mână | M11 | de făcut |
| T7 | Tudor | viewere: PDF.js la bbox, video BASS cu contur, artefacte tipizate, release + calificare, wiki, graf | K1, R7 | de făcut |
| T8 | Tudor | UI generativ din catalog închis (`ViewSpec`), vederi legate pe timp/selecție | K1, M7 | de făcut |
| T9 | Tudor | matricea de condiții și clasamentul de relevanță pentru habitate (Lună, costum, ISS) | R9, M8 | de făcut |
| T10 | Tudor | benchmark (T3 partea 2): `GoldTask`, pilot ~30, set dev + hold-out privat ~20 %, sub-benchmark de retrieval | K1 | de făcut |
| T11 | Tudor | H0/H1 + ablații (skills, wiki, graf, subagenți), ≥ 3 seed-uri, costuri, harness declarat | M10, T10 | de făcut |

R5 și I5 se pot dezvolta pe fixture înaintea calibrării reale, dar nu se declară `metric` / calificate
până nu există. Un WSP slab nu blochează documente/calcul; schimbă doar verdictul și partea de demo.

## Primele trasee cap-coadă

1. Publicație reală + passage id + calcul simplu: Codex cere sursa prin MCP → `EvidenceRef` → UI
   deschide originalul → un calcul produce un artefact → răspunsul îl citează.
2. Un WSP real cu `state = experimental` până la scor.
3. Revizie W1 și comparația părinte–copil.
4. Reluarea investigației după restart și o sursă descoperită pe web.

## Prezentare (separat de cod)

| Cine | Ce |
|---|---|
| Iancu | povestea tehnică, scenariul, model cards; prezentarea orală cu rezultate și limite reale |
| Raul | dovezile NASA: bibliografie, explicația surselor și a proveniența, verificarea fiecărei afirmații de pe slide-uri |
| Tudor | deck, capturi, screen recording, video de rezervă etichetat ca înregistrare, operarea demo-ului |
| Mircea | **nimic** — exclusiv backend, infrastructură și runbook |

Repetiția sistemului este o verificare comună de funcționare, nu redistribuirea slide-urilor.

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

### Faza 2 — trei lane-uri paralele (detalii, motivare, sfaturi de cercetare și surse: [LANES_V2](LANES_V2.md))

Lane-ul lui Iancu (I1–I5) nu se schimbă. **P0** = drumul critic, se începe acum; **P1** = necesar
pentru demo-ul complet; **P2** = îmbunătățire, doar după P0/P1 sau când măsurătorile o justifică.
Contractele noi intră întâi, ca val **K1** (un PR per familie, cu fixture, fără implementare): scrise
în paralel, merge unul după altul, Raul → Mircea → Tudor (`LANES_V2.md` §9.3). Branch scurt per task
din `main` (`feat/<lane>-<task>`), șters la merge.

| P | ID | Owner | Livrabil | Dependență | Stare |
|---|---|---|---|---|---|
| P0 | K1-evidence/dataset | Raul | `evidence.v2` + `dataset.v2`: `SourceAccess`, `SourceLabel`, `SourceOutline`, graf, wiki, web, `ConditionsTable`, `ResponseFormat` | — | **integrat** în `main` (PR #26) |
| P0 | K1-research | Mircea | `research.v2`: `AnswerSubmission`, `ViewSpec` (co-design Tudor), argumente tool, `StepRationale`, `HarnessRef`, `TraceBundle`, tipuri server în core, intrări noi în `tool_catalog()` | — | de făcut |
| P0 | K1-evaluation | Tudor | `evaluation.v2`: `GoldTask`, `HarnessComparison` | — | de făcut |
| P0 | M7 | Mircea | `answer_submit` (efect `compute`): validare pe host → `answer.completed`; refuz cu motiv acționabil | K1-research | de făcut |
| P0 | M6 | Mircea | harness bundle minimal (`harness/`): instrucțiuni operaționale + 2 skills, secțiune de tool-uri generată, injectat în Codex și Claude Code, digest pe episod | K1-research | de făcut |
| P0 | M12 | Mircea | acces la surse fără copie: `GET /api/v1/sources/{id}/content` → `SourceAccess` (URL S3 semnat, proxy NTRS, URL permanent, playlist HLS pe byte-range pentru `.ts`) | K1-evidence, R6 | de făcut |
| P0 | R6 | Raul | corpusul celor 19 investigații de zbor după atlas, descărcat **pe VM** în `/srv/ignis/data/raw/` printr-un pas din `infra/scripts/build-data.sh` (Raul dă selecția, Mircea îl rulează; pe laptop doar un subset, `ACCESS.md` §5): 636 documente (1,63 GB) + 24.482 fișiere text (12,26 GB) cu hash; 35.446 imagini/filme (13,44 TB) rămân la NASA; licența PSI-10; test de lockbox pe documente noi | — | de făcut |
| P0 | R9a | Raul | `ConditionsTable` pentru toate cele 19, din cele 21 de CSV-uri „Experimental table”, cu golurile explicite | K1-dataset, R6 | de făcut |
| P0 | R7a | Raul | parse views cu coordonate (bbox normalizat top-left, tabele, figuri), parser ales prin măsurători pe PDF-uri PSI | R6 | de făcut |
| P0 | R12a | Raul | retrieval SIRA, partea de query: **embeddings, reranker și DeepInfra eliminate**; `sources_search` primește schița agentului (`expected_terms`), filtru DF, `BM25(query) + w·BM25(expansiune)` într-un singur apel; fără reranker | K2-evidence | de făcut |
| P0 | T7 | Tudor | viewere: partea 1 (artefacte tipizate, acoperirea contractelor) în PR #24; partea 2: PDF.js cu transport de range propriu la bbox, MP4 direct din S3, `.ts` prin hls.js, contur BASS, proxy NTRS | K1-evidence, M12, R7a | partea 1 **în review** (PR #24) |
| P0 | T10 | Tudor | pilot gold set ~30 (documentar cu abținere, citare, analitic, predictiv prin protocol), set dev + hold-out privat ~20 %, toate cele 19 investigații | K1-evaluation | de făcut |
| P1 | M10 | Mircea | `TraceBundle` + `ignis harness run --variant` | K1-research, T10 | de făcut |
| P1 | M8 | Mircea | argumente în `tool.started`, `step.rationale`, `response_format`, audit descrieri, înregistrarea tool-urilor lui Raul | K1 | de făcut |
| P1 | M11 | Mircea | rute v1 pentru runs/datasets/conditions | K1-dataset, R9a | de făcut |
| P1 | R7b | Raul | arbore de secțiuni (`SourceOutline`, `sources_outline`), rezumate etichetate ca generate | R7a | de făcut |
| P1 | R8 | Raul | `SourceLabel` cu două treceri și gold set de ~100 documente; model ales prin măsurători (Jev sau model mic cu output structurat) | R7a | de făcut |
| P1 | R2b | Raul | fișierele text de senzori → `DatasetBundle` Parquet pentru investigațiile noi | R6 | de făcut |
| P1 | R9b | Raul | graf (`ProvenanceLink` în SQLite) + `graph_neighbors` | R8, R9a | de făcut |
| P1 | R12b | Raul | retrieval SIRA, partea de index: vocabular de expansiune per chunk (coloană FTS5 separată, niciodată citat), generat în aceeași trecere LLM cu rezumatele R7b și etichetele R8; τ și w alese pe setul dev T10 | R7a, R12a | de făcut |
| P1 | R11 | Raul | indexuri media: cadre-cheie pentru `.ts`, directoare centrale ZIP în cache, conversie TIFF → PNG/WebP la cerere | R6 | de făcut |
| P1 | T9 | Tudor | matricea de condiții + clasamentul pentru habitate (Lună/rover, costum, ISS), statusuri explicite | R9a | de făcut |
| P1 | T8 | Tudor | UI generativ din catalogul închis `ViewSpec`, vederi legate | K1-research, M7 | de făcut |
| P1 | T11 | Tudor | H0/H1 + ablații, ≥ 3 seed-uri, costuri, harness declarat | M10, T10 | de făcut |
| P1 | T6 | Tudor | Runs/Lab/Compare/Atmosphere pe rute v1 | M11 | de făcut |
| P2 | M9 | Mircea | context în mediu (date montate prin ref în `analysis_execute`), delegare depth-1 | M6 | de făcut |
| P2 | M10b | Mircea | căutare offline de harness doar pe dev (GEPA / Meta-Harness) | M10, T11 | de făcut |
| P2 | R4 | Raul | wiki (CAS, delte itemizate) + web discovery cu snapshot | R3 | de făcut |
| P2 | R10 | Raul | upgrade-uri de retrieval doar unde T10 arată un gol (Tantivy, PPR; fără indexuri de embeddings); calibrare BASS; bundle evaluator | T10 | de făcut |

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

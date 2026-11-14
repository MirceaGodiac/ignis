# Workboard

Unități de integrare, nu estimări de ore. Fiecare task are un owner și un rezultat testabil. După C0,
liniile pornesc simultan; dependențele de mai jos sunt reale (de date/contract), nu de calendar —
până la livrarea dependenței se lucrează pe fixture-urile din `fixtures/contracts/`.

| ID | Owner | Livrabil | Dependență | Stare |
|---|---|---|---|---|
| C0 | Iancu | contracte, servicii, scheme + TS, fixture-uri, teste de contract, AGENTS.md, handoff-uri | — | **în acest PR** |
| I1 | Iancu | `ModelSpec` complet FLEX și ACME (normal/inverse): parametri fizici, noise/hazard, stări locale, layout, fără defaulturi ascunse | C0 | |
| I2 | Iancu | `ignis-wsp`: assembler literal (spec + patch [+ parent] → tensori → release prin registry), forward strict, audit al rutei reale de forecast | I1 | |
| I3 | Iancu | candidați W0 independenți Fable / Astra, manifeste, model cards | I2 | |
| I4 | Iancu | W1 cu feedback de development; candidații nereușiți păstrați | I3 + T2 + R2 | |
| I5 | Iancu | BASS WSP pe observabila calificată, prin `ModelService` | I2 + R5 | |
| M1 | Mircea | Codex pin + preflight, bridge stdio, handshake, approvals, mapping thread/turn/item, turn fixture | C0 | **în lucru** (`claude/agent-runtime-m1-b40a11`, peste M2): `ignis_research::runtime` (pin 0.141.0 → 0.169.0, preflight, bridge stdio, approvals prin host, mapping, pause/cancel/resume, recovery fără duplicare), endpoint de aprobare, teste offline cu app-server fals; turn real cu model verificat pe 0.169.0 (login ChatGPT, opt-in), tool-uri prin MCP în M3 |
| M2 | Mircea | `/api/v1` HTTP + SSE reluabil, state store (SQLite WAL), `ArtifactStore`, joburi cu lease; închiderea scurgerilor de test din API-ul vechi; OpenAPI generat | C0 | **în lucru** (`feat/agent-backend`): store, SSE, joburi, artefacte locale, OpenAPI, scurgeri închise, uploads, priorități, fixture mode `/api/fixtures/v1`. Rămân: backend R2 (M4), un singur vault cu Tudor (T2) |
| M3 | Mircea | MCP (`rmcp`) peste serviciile colegilor; runner izolat; fără shell pe host | M1 + M2 | **în lucru** (`feat/m3-mcp`): `crates/ignis-mcp` (dispatcher unic `ToolHost` + server rmcp streamable HTTP pe loopback), token per proces Codex → scope-ul workspace-ului, tool nou `jobs_get`, job MCP atașat episodului; pin Codex 0.169.0; turn real cu model (login ChatGPT) cheamă `sources_search` prin MCP. **Livrat și** (`feat/m3-runner`): runner izolat (`ignis_research::sandbox`, container OCI fără rețea/root read-only/uid 65534/fără capabilități, imagine pin-uită prin id, probă de izolare la preflight) + tool `analysis_execute` → `ExecutionRecord`. Rămâne: verificarea sub gVisor (`runsc`) pe Netcup (M4) |
| M4 | Mircea | Cloudflare/Netcup, auth, secrets, backup/restore, CI | M2 + M3 | **în lucru** (`feat/m4-infra`): CI, verificarea JWT Access în Rust (două assertion-uri semnate), backend R2 pentru `ArtifactStore`, snapshot/restore de stare (`ignis state`), `infra/` (imagine, Compose, systemd, Worker, runbook). Rămân, cu autorizare: provizionarea Cloudflare/Netcup, bucket R2 real, gVisor pe Netcup, RPO/RTO măsurate |
| M5 | Mircea | integrare live, cancel/resume/reconciliation, observabilitate | R3 + I2 + T1 | |
| R1 | Raul | manifest de corpus; un document primar + parser + locator verificat | C0 | |
| R2 | Raul | `DatasetBundle` FLEX/ACME: identitate, unități, timp, calitate | C0 | |
| R3 | Raul | `KnowledgeService`: hybrid retrieval + reader, aceeași logică pentru CLI/HTTP/MCP | R1 | |
| R4 | Raul | web discovery cu snapshot, wiki (CAS), `ProvenanceLink`, corpus snapshot | R3 | |
| R5 | Raul | BASS: clip→run, calibrare, contur, `VisualObservationBundle` | R2 (nu blochează R3) | |
| T1 | Tudor | UI pe fixture-urile C0: chat/SSE, evidence viewer, artefacte, stări pending/failed/inconclusive/experimental | C0 | |
| T2 | Tudor | evaluator pe fixture → `EvaluationResult` determinist; protocol freeze; scorecards | C0 | |
| T3 | Tudor | UI live + benchmark retrieval/răspuns/calcul | M2 + R3 | |
| T4 | Tudor | scoring WSP; harness H0/H1, rezultate pereche | T2 + I3 + M5 | |
| T5 | Tudor | demo bundle, deck/video, export verificabil | T3 + rezultate admise | |

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

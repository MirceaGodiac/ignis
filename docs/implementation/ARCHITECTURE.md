# Arhitectura IGNIS — rezumat operațional

## Componente și autorități

```text
Browser: React / TypeScript / Vite                         web/  (Tudor)
   │  HTTPS, SSE
Cloudflare: Static Assets + Worker mic + Access + R2       infra/ (Mircea, planificat)
   │  Tunnel privat
Netcup: IGNIS Rust / Axum                                   ignis-server + ignis-research (store, joburi, artefacte)
   API · auth · scope · episoade · evenimente · joburi · artefacte
   │
CodexBridge (Rust)  ── JSON-RPC pe stdio privat ──▶ Codex App Server (versiune pin-uită)
   │                                                  investigatorul (model verificat la preflight)
IGNIS MCP (rmcp, loopback) + runner izolat           ignis-mcp (MCP livrat; runner planificat)
   │            │                 │
Knowledge     Datasets          Models (FLEX/ACME/BASS, WSP)      Evaluator izolat (proces separat)
Raul          Raul              Iancu                             Tudor
   └────────── artefacte imutabile + proveniență comune (ignis-core) ──────────┘
```

| Cine deține | Ce |
|---|---|
| Codex App Server | conversația nativă, loop-ul model–tool, delegarea nativă |
| IGNIS (Rust) | episodul științific, scope-ul, joburile, identitățile, artefactele, evenimentele proprii |
| Sursa originală | ce bytes și valori au fost publicate |
| Serviciul numeric | ce operație a fost executată |
| LLM | interpretarea și strategia (claims `interpretation`) |
| Evaluatorul | scorul pe protocolul înghețat |
| Wiki | memorie cu referințe; nu e sursă primară |

## Decizii (ADR) — nu se redeschid fără blocaj demonstrat

| # | Decizie | Înlocuiește |
|---|---|---|
| 01 | Codex App Server ca runtime agentic, adaptor Rust subțire (`AgentRuntime` intern în `ignis-research`) | runtime IGNIS propriu, gateway universal |
| 02 | Rust (Axum/Tokio/Tower, serde/schemars) pentru backend și servicii; TS pentru UI/edge; Python doar în workeri utili (SAM2, parsare) | rescriere în Go |
| 03 | C0 (acest PR) apoi patru lane-uri | contracte comune lăsate la final |
| 04 | Iancu WSP/modele · Mircea backend/Codex/infra · Raul date/corpus/retrieval/wiki/vision · Tudor UI/evaluare | `docs/TEAM.md` istoric |
| 05 | Mircea fără sarcini de prezentare | deck/video la Mircea |
| 06 | WSP literal: autorul scrie numerele; assemblerul aplică și serializează | generator de coeficienți, training ascuns, „start from NASA-trained” |
| 07 | Forward strict separat (fără optimizer la request); evaluator independent | local MAP/Laplace în serving pentru releases WSP |
| 08 | Catalog relevant + materializare selectivă, buget de stocare | mirror 3,45 TB sau plafon arbitrar de PDF-uri |
| 09 | Un singur serviciu hybrid de retrieval (lexical + dense + rerank) pentru CLI/HTTP/MCP; query numeric separat | trei implementări, embeddings ca dovadă |
| 10 | Proveniență comună, artefacte imutabile content-addressed | citări doar cu URL |
| 11 | Cloudflare + Netcup, Compose + systemd, fără Nginx | stack distribuit „de prestigiu” |
| 12 | Limite reale de resurse + continuare (paginare, checkpoint); fără `max_steps/sources/tokens` semantice | trunchiere ireversibilă |
| 13 | Benchmark H0/H1 cu același runtime/model/date/tools | comparație cu un LLM fără acces |
| 14 | Fork/rewrite doar pe beneficiu măsurat | rescriere presupusă superioară |
| 15 (C0) | Serviciile de domeniu sunt trait-uri **sincrone** în `ignis-core`; transporturile async le cheamă pe pool blocant | `async-trait`/Tokio în nucleu |
| 16 (C0) | `TrainedOn` fără default; construcția WSP în `ModelRelease.construction`; reader legacy pentru 2.0.0 | default `nasa` implicit |

Un ADR nou (în acest fișier, rând nou + paragraf): motivul, alternativa respinsă, testul care ar
infirma alegerea, impactul pe contracte, ownerul.

## Traseul unei întrebări (țintă, nu cod existent)

1. Browserul trimite întrebarea (`client_request_id`). API validează utilizatorul și derivă
   `ExecutionScope`; creează `ResearchEpisode` pin-uit pe corpus/model; răspunde 202; browserul
   deschide SSE.
2. Bridge-ul pornește un turn Codex. Investigatorul vede catalogul de tool-uri (`tool_catalog()`),
   nu documentele.
3. `sources_search` → `SearchPage` (hit = `EvidenceRef` + snippet); `sources_read` pentru pasaj/pagină.
4. `datasets_inspect` / `models_inspect`: domeniu, input schema, stare de calificare.
5. `models_forecast` (`ForecastRequest` cu run + cutoff): serviciul numeric construiește `PrefixBundle`
   (`build_prefix`, izolare causală), rulează forwardul cu weights fixe, salvează `AnalysisArtifact`.
   Lung ⇒ `accepted` + `JobRef`; conexiunea pierdută nu omoară jobul.
6. Răspunsul: `ResearchAnswer` cu claims legate de evidență și de valori de artefact (JSON Pointer),
   limitări și release-urile folosite; `answer.completed` în stream.
7. Evaluarea finală e alt proces (Tudor), alt scope, alt mount; nu rulează din episoade.

## Performanță și memorie — decizii structurale

- **Referințe, nu payloaduri:** `ArtifactRef` în evenimente și tool results; previews `partial` cu
  `full`. SSE nu transportă bytes mari.
- **Streaming:** `ArtifactStore::put/open` lucrează cu `Read`; nici un serviciu nu încarcă artefacte
  întregi ca să le paseze mai departe; corpusul nu se încarcă în RAM.
- **Columnar:** datele numerice circulă ca Parquet (`DatasetView`), nu ca JSON array.
- **CPU în afara reactorului:** serviciile sunt sincrone; Axum le cheamă pe `spawn_blocking`/rayon cu
  concurență bugetată (interactive > evaluări cerute > ingestie). Calculele lungi sunt joburi cu lease.
- **Cache keys complete:** scope/workspace + corpus snapshot + release id + prefix `input_hash` +
  hash-ul setărilor (exemplul existent: `analysis::cache_key`). Niciodată doar după run id.
- **Idempotență:** `client_request_id` la creare; job-urile au id durabil; efectele ambigue merg în
  `reconciling`, nu se reexecută orb.
- **Fără limite arbitrare:** paginare (`PageCursor`), `ReadResult.next`, `JobProgress.total` opțional.
  Limitele de transport (chunk, body) nu limitează rezultatul total.
- Nu există măsurători de latență/throughput în acest PR; nu se publică cifre până nu se măsoară.

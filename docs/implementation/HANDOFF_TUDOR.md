# Handoff — Tudor: Research Workspace & Independent Evaluation

Dă agentului tău: acest fișier, `docs/implementation/LANES_V2.md` (§6, §9, §10), `AGENTS.md`, `docs/implementation/{CONTRACTS,INTEGRATION}.md` și
`docs/design/IGNIS_MASTER_V2.md` §16–§18 și §22–§26. Branch: unul scurt per task din `main` (`feat/research-product-<task>`).

## 0. Faza 2 (citește întâi) — workspace & evaluare independentă

Detalii, motivare, surse: [LANES_V2](LANES_V2.md) §6 (lane-ul tău), §9 (contracte), §10 (testele de
acces și de browser, cu cifre).

**Misiune:** orice contract pe care îl întâlnește un utilizator are o vedere fidelă, originalele NASA se
deschid la locul citat direct de la NASA, agentul compune răspunsuri dintr-un catalog închis de
componente, întrebările challenge-ului au vederi directe, iar harness-ul e măsurat independent.

**P0, în ordine:** (1) PR-ul `evaluation.v2` (`GoldTask`, `HarnessComparison`), scris în paralel, intră
al treilea; co-design `ViewSpec` cu Mircea; (2) **T7** partea 1 e în PR #24; partea 2: PDF.js cu
**transport de range propriu** (mărimea din manifest; măsurat: pagina 100 dintr-un PDF de 49 MB din
1,26 MB în 6,2 s), highlight la bbox, MP4 direct din S3, `.ts` prin hls.js pe playlistul lui Mircea,
contur BASS ca strat sincronizat; (3) **T10** pilot de ~30 de task-uri pe toate cele 19 investigații,
set dev partajat + hold-out privat ~20 % în vault, rulat o singură dată cu echipa. **P1:** T9 matricea
de condiții și clasamentul pentru habitate, T8 catalogul `ViewSpec`, T11 H0/H1, T6 rute v1, T4/T5.

**Cum alegi singur:** metoda din LANES_V2 §3. Prototipezi fiecare viewer pe un URL NASA semnat real în
browser înainte de UI (ca în §10). Citești LitQA2 (abținere), ALCE (citări), LAB-Bench (split privat) și
lucrarea despre declararea harness-ului înainte de a scrie task-uri; pilot, analiza eșecurilor, apoi
scalare.

**Eficiență:** PDF.js cu `rangeChunkSize` 256 KiB, `disableAutoFetch`, doar paginile vizibile,
încărcat leneș; `.ts` doar pe segmente aliniate la cadre-cheie (segmentele egale pe bytes au picat la
seek); uPlot cu decimare min/max la ~2 puncte/pixel; liste virtualizate; artefactele content-addressed
cu `staleTime: Infinity`; benchmark concurent până la limitele providerului; raportezi mărimea
bundle-ului per rută, timpul până la prima pagină, latența de seek.

## 1. Obiectiv

Două livrabile independente: (a) Research Workspace — UI-ul în care cercetătorul conduce episodul,
deschide sursele, vede grafice observație vs predicție, modele și starea lor de calificare; (b)
evaluatorul independent — scorecards reproductibile, protocol înghețat, lockbox separat, feedback de
development pentru Iancu și benchmark-ul H0/H1.

## 2. Ce există și se reutilizează

- `web/`: React 18 + Vite + `@tanstack/react-query` + uPlot (`components/ForecastChart.tsx`), workspaces
  Explore / Compare / Lab, `api.ts` (fetch spre `/api` sau `/api/fixtures`, tipuri de envelope scrise de
  mână pentru API-ul vechi), `src/contracts/generated.ts` (generat, 48 de tipuri, inclusiv C0).
- `crates/ignis-lab/src/eval/`: vault (`holdout/`, seal/reveal, `reveal_log.jsonl`, `VaultAccess`
  emis doar de `eval`), scorere (CRPS, pinball, intervale, LPD, Brier KM/IPCW, erori de front,
  refusal accounting), `compare` (refuză cazuri/metrici/unități diferite), bootstrap pe clustere, report.
- Contracte: `EvaluationRequest/Result` (`purpose`, `CaseAccounting`, `Population`, `Verdict`),
  `ResearchEvent` (+ `check_event_log`), `ToolResult`, `ResearchAnswer` (claims kind-aware),
  `EvidenceRef` + `SourceLocator` (bbox, page label), `ArtifactRef`, `InlinePayload.partial`.
- Fixture-uri: tot `fixtures/contracts/` — în special `research/events.json` (stream complet cu job,
  failure, answer), `research/answer.json`, `evidence/search_page.json` (degradat),
  `data/visual_bundle_pixel_only.json`, `evaluation/*`.

## 3. Ce lipsește

- Client SSE cu cursor + deduplicare `(episode_id, seq)` + resync la 410; chat/steering; evidence/PDF
  viewer cu bbox; ecranele de artefacte, modele (stări `experimental`/`fit_only`/calificat), wiki;
  toate stările pending/failed/inconclusive/insufficient_evidence/degraded.
- `EvaluationService` care produce `EvaluationResult` din scorerele existente; protocol freeze
  (`ProtocolRef.frozen_hash`); scorecards; benchmark (`benchmarks/`).

## 4. Fișiere deținute

`web/` (fără `generated.ts`), `crates/ignis-lab/src/eval/`, `crates/ignis-lab/src/access.rs`,
`configs/evaluation/`, `benchmarks/`, `docs/demo/`, `crates/ignis-core/src/contracts/evaluation.rs`,
`fixtures/contracts/evaluation/`.

## 5. Contracte

Consumi: tot ce e afișabil (research, evidence, dataset, wsp, `AnalysisArtifact`). Produci:
`EvaluationRequest/Result`, implementarea `EvaluationService`, scorecards.

## 6. Interfețe

UI ↔ Mircea: `/api/v1` + SSE (`CONTRACTS.md` §Streaming; tabelul din `INTEGRATION.md`).
Evaluator ↔ Iancu: rulezi release-urile prin `ModelService` în procesul tău; îi dai doar
`EvaluationResult` cu `purpose = development_feedback`. Evaluator ↔ Raul: `DatasetBundle`
(`audience = evaluator` pentru test, montat doar la tine).

## 7. Dependențe

T1/T2 depind doar de C0: UI pe fixture-uri (import JSON sau servite static), evaluator pe fixture.
T3 după M2 + R3; T4 după T2 + I3 + M5.

## 8. Sugestii tehnice

- Tipurile vin **doar** din `generated.ts`; un DTO scris de mână e un bug. `api.ts` vechi rămâne pentru
  rutele vechi până la migrare.
- Selector mock/live: aceleași contracte și rute; mock = fișierele din `fixtures/contracts`.
- Randează discriminat pe `body.kind`; `runtime.opaque` ca element neutru; `message.delta` doar pentru
  afișare progresivă, `message.completed`/`answer.completed` sunt sursa finală.
- Afișează `origin = synthetic_fixture` vizibil; `InlinePayload.partial` ca preview cu link la artefact;
  `SearchPage` degradat ca atare; `units_status = pixel_only` niciodată pe o axă în metri.
- Unitățile: datele sunt SI; conversia de afișare doar în componenta de grafic.
- Evaluator: rulează în proces separat, cu scope `ScopeRole::Evaluation`; lockbox-ul nu e montat în
  worktree-urile builderilor; refuzurile rămân în numitor; raportezi atât calitatea condiționată
  (`scored_cases`) cât și coverage pe toate cazurile.
- Harness H0/H1: același Codex/model/date/tools, cu și fără extensiile IGNIS.

## 9. Riscuri

- Scurgerile de test din API-ul vechi (vezi `INTEGRATION.md`) — Lab-ul UI nu trebuie să ceară
  observații dincolo de cutoff pentru run-uri de validare/test; cere fix de la Mircea, nu ocoli în UI.
- Două store-uri de seal/reveal (server vs vault) — vault-ul tău e singurul autoritativ.
- Nu remedia un scor prin eliminarea cazurilor eșuate; schimbarea protocolului = versiune nouă.

## 10. Criterii de finalizare

Citarea deschide sursa la locator; graficul arată corect observație vs predicție cu unități; streamul
se reia după reconectare fără dubluri (test pe `events.json` tăiat la un cursor); evaluatorul rulează
independent și produce `EvaluationResult` determinist care trece `validate()`; toate failure-urile
sunt reprezentate în UI.

## 11. Ce NU faci

Nu copiezi formule în frontend; nu inventezi tipuri pentru inputurile modelelor; nu repari în
lane-urile celorlalți; nu dai lui Iancu rezultate de final-test; nu schimbi modelele; nu operezi
infrastructura (contribui doar build-ul frontendului).

## 12. Predare

PR per task (T1…T5): UI live, scorecards, gold tasks verificate, teste de reconnect/citare/unități,
comenzi rulate. Raport de integrare către toți (nu doar capturi).

## Prezentare

Deck, capturi, screen recording, video de rezervă etichetat ca înregistrare, operarea demo-ului.
Iancu dă povestea, Raul sursele și verificarea; Mircea nu primește taskuri de prezentare.

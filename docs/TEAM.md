# Echipa și împărțirea muncii

Document intern pentru Iancu, Mircea, Raul și Tudor. Înlocuiește împărțirea anterioară (felii RAG /
corpus / experimente / „Iancu minim”), coada de clasificare pe trei shard-uri și calendarul
fit-peste-noapte → test sigilat. Decizia curentă: `docs/design/IGNIS_MASTER_V2.md` §1–§3.

## Cine ce face

| Cine | Lane | Branch | Handoff |
|---|---|---|---|
| **Iancu** | C0 (contracte comune), apoi Scientific Models & literal WSP: FLEX/ACME/BASS, `ignis-wsp`, specificații, assembler, forward strict, releases fără parent NASA-trained | `feat/handwritten-science` | [HANDOFF_IANCU](implementation/HANDOFF_IANCU.md) |
| **Mircea** | Codex App Server, backend Rust/Axum, SSE, joburi, artefacte, MCP, runner, Cloudflare/GCE; integratorul composition root-ului | `feat/agent-backend` | [HANDOFF_MIRCEA](implementation/HANDOFF_MIRCEA.md) |
| **Raul** | date NASA, corpus, retrieval hibrid, reader, web, wiki, `DatasetBundle`, observații BASS | `feat/scientific-data` | [HANDOFF_RAUL](implementation/HANDOFF_RAUL.md) |
| **Tudor** | Research Workspace (frontend) și evaluarea independentă (vault, scorecards, benchmark) | `feat/research-product` | [HANDOFF_TUDOR](implementation/HANDOFF_TUDOR.md) |

Ownership pe fișiere, zonele comune și regulile de merge: [INTEGRATION](implementation/INTEGRATION.md).
Task-urile și dependențele: [WORKBOARD](implementation/WORKBOARD.md). Contractele:
[CONTRACTS](implementation/CONTRACTS.md).

## Prezentarea

| Cine | Ce |
|---|---|
| Iancu | povestea tehnică, scenariul, model cards, prezentarea orală |
| Raul | dovezile NASA, bibliografia, explicația surselor, verificarea afirmațiilor de pe slide-uri |
| Tudor | deck, capturi, screen recording, video de rezervă etichetat, operarea demo-ului |
| Mircea | **fără sarcini de prezentare** — exclusiv backend, infrastructură și runbook |

## Cum pornește fiecare

1. După merge-ul C0: `git worktree add -b <branch> ../ignis-<nume> origin/main` (vezi INTEGRATION).
2. Pornești sesiunea agentului în worktree-ul tău, cu `AGENTS.md` + handoff-ul tău.
3. Lucrezi pe fixture-urile din `fixtures/contracts/` până livrează colegul de care depinzi.
4. PR mic per task; schimbările de contract în PR separat, cu review de la consumator.

## Partiția de test

Split-urile din `data/splits/` sunt înghețate (train / validation / test pe grupuri). Partiția de test
se deschide **o singură dată**, de evaluator (lane-ul lui Tudor), cu toată echipa, după ce cohorta,
split-ul, modelele, protocolul și metricile sunt înghețate. Până atunci nimeni, nici om, nici agent:

- nu rulează `ignis eval reveal`, `score` sau `compare` cu `--final-test` / `--partition test`;
- nu citește `holdout/` și nu deschide continuările run-urilor de test;
- nu dă unui autor de weights rezultate de test și nu alege exemple, praguri sau setări după ele.

Orice reveal, acceptat sau refuzat, intră în `runs/evaluation/reveal_log.jsonl`. O corectură ghidată
de test transformă testul în dezvoltare; redenumirea fișierelor sau schimbarea seed-ului nu repară asta.

## Istoric

Coada de clasificare automată a etichetelor (`ignis data review auto …`, documentată în
[GUIDE](GUIDE.md) §3) rămâne disponibilă ca unealtă a lane-ului lui Raul; nu mai e împărțită pe
shard-uri între colegi.

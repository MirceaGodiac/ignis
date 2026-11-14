# Echipa și împărțirea muncii

Document intern pentru Iancu, Mircea, Raul și Tudor. Înlocuiește împărțirea anterioară (felii RAG /
corpus / experimente / „Iancu minim”), coada de clasificare pe trei shard-uri și calendarul
fit-peste-noapte → test sigilat. Decizia curentă: `docs/design/IGNIS_MASTER_V2.md` §1–§3.

## Cine ce face

| Cine | Lane | Prefix de branch (unul scurt per task) | Handoff |
|---|---|---|---|
| **Iancu** | C0 (contracte comune), apoi Scientific Models & literal WSP: FLEX/ACME/BASS, `ignis-wsp`, specificații, assembler, forward strict, releases fără parent NASA-trained | `feat/handwritten-science-<task>` | [HANDOFF_IANCU](implementation/HANDOFF_IANCU.md) |
| **Mircea** | harness & platformă: runtime-uri (Codex, Claude Code), instrucțiuni, skills, `answer_submit`, trace-uri, backend Rust/Axum, SSE, joburi, artefacte, MCP, runner, acces la surse fără copie, Cloudflare/GCE; integratorul | `feat/agent-backend-<task>` | [HANDOFF_MIRCEA](implementation/HANDOFF_MIRCEA.md) |
| **Raul** | cunoaștere & date NASA: corpusul celor 19 investigații, parsare cu coordonate, arbore de secțiuni, etichete, `ConditionsTable`, graf, retrieval hibrid, wiki, web, `DatasetBundle`, observații BASS, indexuri media | `feat/scientific-data-<task>` | [HANDOFF_RAUL](implementation/HANDOFF_RAUL.md) |
| **Tudor** | Research Workspace (viewere direct de la NASA, UI generativ din catalog, vederile challenge-ului) și evaluarea independentă (gold set, H0/H1, vault, scorecards) | `feat/research-product-<task>` | [HANDOFF_TUDOR](implementation/HANDOFF_TUDOR.md) |

Ownership pe fișiere, zonele comune și regulile de merge: [INTEGRATION](implementation/INTEGRATION.md).
Task-urile și dependențele: [WORKBOARD](implementation/WORKBOARD.md). Contractele:
[CONTRACTS](implementation/CONTRACTS.md).

**Faza 2** ([LANES_V2](implementation/LANES_V2.md)): împărțirea pornește de la ce trebuie demonstrat
(§1 acolo): date citabile (Raul), agent care leagă răspunsurile (Mircea), cercetător care verifică
(Tudor), prognoze cu stare onestă (Iancu), dovada măsurată (Tudor, evaluator independent). Drumul
critic: răspuns legat live, condițiile pentru toate cele 19 investigații, harness măsurat. Serverul
ține ~13,9 GB; media NASA (13,44 TB) se citește direct de la NASA. Fiecare lane are în `LANES_V2.md`
prioritățile P0/P1/P2, metoda de a-și alege singur opțiunile (măsurat pe datele noastre) și regulile
de eficiență. Contractele comune (val K1) și ordinea de merge: §9.

## Prezentarea

| Cine | Ce |
|---|---|
| Iancu | povestea tehnică, scenariul, model cards, prezentarea orală |
| Raul | dovezile NASA, bibliografia, explicația surselor, verificarea afirmațiilor de pe slide-uri |
| Tudor | deck, capturi, screen recording, video de rezervă etichetat, operarea demo-ului |
| Mircea | **fără sarcini de prezentare** — exclusiv backend, infrastructură și runbook |

## Cum pornește fiecare

0. Citești [ACCESS](implementation/ACCESS.md): repo, branch-uri, setup local, aplicația de staging și
   VM-ul, unde stau secretele și datele.
1. Pentru fiecare task: `git worktree add -b feat/<lane>-<task> ../ignis-<nume> origin/main` (sau un
   branch nou în worktree-ul tău, din `origin/main` proaspăt); branch-ul se șterge la merge.
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

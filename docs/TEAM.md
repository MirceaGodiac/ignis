# Echipa și împărțirea muncii

## Cine ce face

Ownership pe fișiere, zonele comune și regulile de merge: [INTEGRATION](implementation/INTEGRATION.md).
Task-urile și dependențele: [WORKBOARD](implementation/WORKBOARD.md). Contractele:
[CONTRACTS](implementation/CONTRACTS.md).

**Faza 2** ([LANES_V2](implementation/LANES_V2.md)): Mircea preia și harness-ul (instrucțiuni, skills,
subagenți, `answer_submit`, trace-uri, H0/H1 de partea runtime-ului, accesul la surse fără copie); Raul
acoperă cele 19 investigații PSI de zbor (~14,1 GB pe server, media rămâne la NASA), clasificarea
documentelor și graful de combustie; Tudor face viewerele (originalele citite direct de la NASA),
UI-ul generativ din catalog, vederile cerute de challenge și benchmark-ul independent. Contractele
comune ale fazei (val K1) și ordinea de merge sunt în `LANES_V2.md` §5.

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

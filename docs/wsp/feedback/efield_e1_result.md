# efield: candidate E1 (PROTOCOL 13.6), development, in-sample

- Study: `docs/wsp/research/phys_efield_study.md` (sha256 `573d8dbc5abe440fc8cb097270b4b61908d69fd057aa3ab9c79e491c7458da6e`). Edits: `runs/dev/gen7/study/efield_e1.wsp` (sha256 `9c4344adf3598741e0d769d77f2e94acd3de4eceb760e91cde485f5dab6ed39b`), verbatim.
- Release: `sha256:dbb4bd6453a2870ddbc6c34125b38068dd6b83da87025b2ae170aa0f25b873b0` (`configs/models/law/efield/e1-fable`), a revision of efield W0 `sha256:73f91b55c81dc724fe2441c2254e109c65f8e08b253fcbe4c00a841f7a0986a9`.
- Element check: the release equals its parent except exactly the 31 elements the file writes (8 of them already carried the same value in the parent); 23 elements differ from the parent; mismatches: 0.
- Scores: the Rust forward (`tools/wsp/law_run.py`, `target-merged/release/ignis.exe`) through the lane scorer (`tools/wsp/ext/law_score.py`, the lane's own `score()`), by `tools/wsp/ext/law_revision.py dev`; train and validation days together (PROTOCOL 13.4: in-sample), on the cases parent and candidate both answer; the other releases and the baselines on the same cases. Lower is better. Lane case files (sha256): `cases_train.jsonl` `707a005faeec…`, `cases_validation_inputs.jsonl` `76bc35d2501d…`, `cases_validation_outcomes.jsonl` `f591f4a10082…`.

## target t1

| metric | partition | efield W0 | efield E1 | efield2 s1 | constant (simple) | space-charge law up to saturation, train sigma (T1) | stratified incidence (T2; a stratum absent from train takes the pooled incidence) |
|---|---|---|---|---|---|---|---|
| CRPS uA, case mean | pooled | 0.6592 | 0.2530 | 0.7206 | 0.5625 | 0.4256 | - |
| CRPS uA, case mean | train | 0.7202 | 0.2681 | 0.7857 | 0.5313 | 0.4533 | - |
| CRPS uA, case mean | validation | 0.5463 | 0.2252 | 0.6002 | 0.6202 | 0.3742 | - |
| mean of day means | pooled | 0.6324 | 0.2428 | 0.6733 | 0.5612 | 0.4267 | - |
| mean of day means | train | 0.6690 | 0.2487 | 0.7224 | 0.4968 | 0.4225 | - |
| mean of day means | validation | 0.5320 | 0.2263 | 0.5384 | 0.7383 | 0.4382 | - |

## target t2

| metric | partition | efield W0 | efield E1 | efield2 s1 | constant (simple) | space-charge law up to saturation, train sigma (T1) | stratified incidence (T2; a stratum absent from train takes the pooled incidence) |
|---|---|---|---|---|---|---|---|
| integrated Brier | pooled | 0.1478 | 0.1478 | 0.1001 | 0.1252 | - | 0.0697 |
| integrated Brier | train | 0.1761 | 0.1761 | 0.1015 | 0.1397 | - | 0.0544 |
| integrated Brier | validation | 0.0753 | 0.0753 | 0.0975 | 0.0884 | - | 0.1124 |
| Brier of ends in window | pooled | 0.1484 | 0.1484 | 0.1303 | 0.1533 | - | 0.1237 |
| Brier of ends in window | train | 0.1689 | 0.1689 | 0.1220 | 0.1649 | - | 0.1175 |
| Brier of ends in window | validation | 0.0762 | 0.0762 | 0.1596 | 0.1120 | - | 0.1456 |

## by day: T1 ion current (CRPS uA per level; error = ln median forecast - ln outcome)

Cases: efield W0 3235, efield E1 3235, efield2 s1 3235.

| day | efield W0 | efield E1 | efield2 s1 |
|---|---|---|---|
| 18073 | 0.6537 | 0.2355 | 0.8134 |
| 18078 | 0.3519 | 0.1860 | 0.3604 |
| 18080 | 0.1490 | 0.0934 | 0.1521 |
| 18092 | 1.1302 | 0.3502 | 1.0904 |
| 18095 | 0.4016 | 0.1634 | 0.3590 |
| 18102 | 0.1500 | 0.0674 | 0.1829 |
| 18106 | 0.2080 | 0.1523 | 0.2176 |
| 18113 | 0.8922 | 0.0877 | 1.1239 |
| 18130 | 0.5264 | 0.1399 | 0.5701 |
| 18137 | 1.9050 | 0.7956 | 2.4087 |
| 18282 | 0.4073 | 0.2005 | 0.3542 |
| 18285 | 0.9231 | 0.3648 | 0.7998 |
| 18298 | 0.1993 | 0.1189 | 0.1800 |
| 18312 | 0.9369 | 0.1229 | 0.8360 |
| 18319 | 0.6521 | 0.5628 | 0.6517 |

Candidate lower than the parent on **15 of 15** days whose forecasts change (15 days).

## by day: T2 flame out under the field (per test: mean Brier over the eligible horizons; forecast and observed = out by 60 s, tests observed to 60 s)

Cases: efield W0 86, efield E1 86, efield2 s1 86.

| day | efield W0 | efield E1 | efield2 s1 |
|---|---|---|---|
| 18073 | 0.0004 | 0.0004 | 0.0106 |
| 18078 | 0.0695 | 0.0695 | 0.0627 |
| 18080 | 0.0022 | 0.0022 | 0.0345 |
| 18092 | 0.0023 | 0.0023 | 0.0023 |
| 18095 | 0.0027 | 0.0027 | 0.0027 |
| 18102 | 0.1654 | 0.1654 | 0.1034 |
| 18106 | 0.1288 | 0.1288 | 0.0752 |
| 18113 | 0.3355 | 0.3355 | 0.2252 |
| 18130 | 0.2401 | 0.2401 | 0.1571 |
| 18137 | 0.0003 | 0.0003 | 0.0003 |
| 18282 | 0.0879 | 0.0879 | 0.0594 |
| 18285 | 0.0006 | 0.0006 | 0.0391 |
| 18298 | 0.0389 | 0.0389 | 0.0912 |
| 18312 | 0.1403 | 0.1403 | 0.0754 |
| 18319 | 0.0014 | 0.0014 | 0.0619 |

Candidate lower than the parent on **0 of 0** days whose forecasts change (15 days).

Agreement with the logged what-if run of E1 (2026-11-15T07:59:14+00:00, the Python copy of the forward): largest pooled difference over every metric 1.7e-16 (round-off); days lower identical: 15 of 15; 0 of 0.
**Reading (13.6):** ion current (t1, CRPS uA, case mean, pooled) 0.2530 against efield W0 0.6592 (efield2 0.7206; best baseline: space-charge law up to saturation 0.4256), lower on 15 of 15 days whose forecasts change; flame-out (t2) identical to the parent (0.1478; 0 days change): **kept**. E1 replaces W0 as the efield model for the final run. On t2 it carries W0's forecasts, which stay above the stratified-incidence baseline (0.0697) and above efield2 (0.1001, not kept in 13.4 because of its ion current). Development numbers, in-sample; nothing is validated.

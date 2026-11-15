# samer: candidate S1 (PROTOCOL 13.6), development, in-sample

- Study: `docs/wsp/research/phys_samer_study.md` (sha256 `e9687b7b782354b0b0ecaa96af88983971190a76543ea48ded07a50eafaa884f`). Edits: `runs/dev/gen7/study/samer_s1.wsp` (sha256 `b9a15a814a4ca024dfc61045d4503adedb3ce2acffc80b57749f564f0d5e4872`), verbatim.
- Release: `sha256:e33ee7665b599e7e6c8c978f61ebf3bf106b2b6a94c3a0821dbdf42cf47a1b5f` (`configs/models/law/samer/s1-fable`), a revision of samer W0 `sha256:05f77ae2f7a666e0da69d04fa29d9ad8b0ac15790645abe1adc0149292327fb0`.
- Element check: the release equals its parent except exactly the 103 elements the file writes (16 of them already carried the same value in the parent); 87 elements differ from the parent; mismatches: 0.
- Scores: the Rust forward (`tools/wsp/law_run.py`, `target-merged/release/ignis.exe`) through the lane scorer (`tools/wsp/ext/law_score.py`, the lane's own `score()`), by `tools/wsp/ext/law_revision.py dev`; train and validation days together (PROTOCOL 13.4: in-sample), on the cases parent and candidate both answer; the other releases and the baselines on the same cases. Lower is better. Lane case files (sha256): `cases_train.jsonl` `b04f002d8bba…`, `cases_validation_inputs.jsonl` `c746ab7205ec…`, `cases_validation_outcomes.jsonl` `186c084fe085…`.

## target d10

| metric | partition | samer W0 | samer S1 | samer D1 | B0 / I0 constant (simple, SAME-R train) | B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) |
|---|---|---|---|---|---|---|
| CRPS of ln d10, case mean | pooled | 0.2377 | 0.1717 | 0.2016 | 0.3458 | 0.2762 |
| CRPS of ln d10, case mean | train | 0.2722 | 0.1907 | 0.2158 | 0.3229 | 0.2454 |
| CRPS of ln d10, case mean | validation | 0.1672 | 0.1327 | 0.1725 | 0.3925 | 0.3392 |
| day-equal | pooled | 0.2112 | 0.1509 | 0.1790 | 0.3487 | 0.2773 |
| day-equal | train | 0.2292 | 0.1594 | 0.1859 | 0.2807 | 0.2369 |
| day-equal | validation | 0.1512 | 0.1225 | 0.1561 | 0.5756 | 0.4120 |

## target iss

| metric | partition | samer W0 | samer S1 | samer D1 | B0 / I0 constant (simple, SAME-R train) | B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) |
|---|---|---|---|---|---|---|
| Brier, case mean | pooled | 0.1753 | 0.0562 | 0.1468 | 0.2729 | 0.1012 |
| Brier, case mean | train | 0.1501 | 0.0491 | 0.1005 | 0.2066 | 0.0312 |
| Brier, case mean | validation | 0.2279 | 0.0711 | 0.2436 | 0.4112 | 0.2473 |
| day-equal | pooled | 0.1516 | 0.0512 | 0.1074 | 0.2579 | 0.0734 |
| day-equal | train | 0.1490 | 0.0499 | 0.0882 | 0.2032 | 0.0336 |
| day-equal | validation | 0.1605 | 0.0552 | 0.1713 | 0.4400 | 0.2060 |

## by day: d10 (CRPS of ln d10; error = ln median forecast - ln outcome)

Cases: samer W0 70, samer S1 70, samer D1 70.

| day | samer W0 | samer S1 | samer D1 |
|---|---|---|---|
| PSI-101:day:2010-07-16 | 0.2733 | 0.1662 | 0.2033 |
| PSI-101:day:2010-07-20 | 0.1640 | 0.1640 | 0.1640 |
| PSI-101:day:2010-07-21 | 0.1119 | 0.0964 | 0.1119 |
| PSI-101:day:2010-07-22 | 0.1506 | 0.1161 | 0.1256 |
| PSI-101:day:2010-07-29 | 0.1438 | 0.1093 | 0.1438 |
| PSI-101:day:2010-07-30 | 0.2825 | 0.2014 | 0.2114 |
| PSI-101:day:2010-08-02 | 0.2235 | 0.1602 | 0.2235 |
| PSI-101:day:2010-08-23 | 0.3432 | 0.1045 | 0.1963 |
| PSI-101:day:2010-08-24 | 0.3247 | 0.2106 | 0.2478 |
| PSI-101:day:2010-08-25 | 0.1004 | 0.0961 | 0.0923 |
| PSI-101:day:2010-09-07 | 0.1603 | 0.1315 | 0.1779 |
| PSI-101:day:2010-09-08 | 0.1815 | 0.1397 | 0.1786 |
| PSI-101:day:2010-09-09 | 0.2857 | 0.2654 | 0.2507 |

Candidate lower than the parent on **12 of 12** days whose forecasts change (13 days).

## by day: ISS detector at full scale (Brier)

Cases: samer W0 71, samer S1 71, samer D1 71.

| day | samer W0 | samer S1 | samer D1 |
|---|---|---|---|
| PSI-101:day:2010-07-16 | 0.2511 | 0.0335 | 0.0967 |
| PSI-101:day:2010-07-20 | 0.0131 | 0.0131 | 0.0131 |
| PSI-101:day:2010-07-21 | 0.0011 | 0.0011 | 0.0011 |
| PSI-101:day:2010-07-22 | 0.1185 | 0.0110 | 0.0364 |
| PSI-101:day:2010-07-29 | 0.0725 | 0.0084 | 0.0725 |
| PSI-101:day:2010-07-30 | 0.1036 | 0.0148 | 0.0371 |
| PSI-101:day:2010-08-02 | 0.3164 | 0.3164 | 0.3164 |
| PSI-101:day:2010-08-23 | 0.1972 | 0.0244 | 0.0824 |
| PSI-101:day:2010-08-24 | 0.0925 | 0.0235 | 0.0521 |
| PSI-101:day:2010-08-25 | 0.1821 | 0.0203 | 0.0628 |
| PSI-101:day:2010-09-07 | 0.1755 | 0.1134 | 0.1851 |
| PSI-101:day:2010-09-08 | 0.3049 | 0.0511 | 0.3278 |
| PSI-101:day:2010-09-09 | 0.1426 | 0.0341 | 0.1128 |

Candidate lower than the parent on **10 of 10** days whose forecasts change (13 days).

Agreement with the logged what-if run of S1 (2026-11-15T08:54:20+00:00, the Python copy of the forward): largest pooled difference over every metric 8.3e-17 (round-off); days lower identical: 12 of 12; 10 of 10.
**Reading (13.6):** both targets change and both are lower than the current model (W0). Particle diameter (d10, CRPS of ln d10, case mean, pooled) 0.1717 against W0 0.2377 (D1 0.2016; best baseline: per-material B1 0.2762), lower on 12 of 12 days whose forecasts change; detector at full scale (iss, Brier, case mean) 0.0562 against 0.1753 (D1 0.1468; best baseline: per-material I1 0.1012), lower on 10 of 10 days: **kept**. S1 replaces W0 as the SAME-R model for the final run. Against D1 as well: lower pooled on both targets, and on 10 of 12 (d10) and 10 of 10 (iss) days. On iss, train alone, the per-material baseline stays lower (0.0312 against 0.0491); on validation S1 is far lower (0.0711 against 0.2473). The edits file is the logged `s1.wsp` (copied byte for byte into `runs/dev/gen7/study/samer_s1.wsp` from the scratchpad path the log names; sha256 equal to the log and to the dossier); its 103 elements equal those of the dossier's Edits block (checked). The two hinge units change role as the dossier says: `d10.median` unit 7 (all zero in W0) now carries W1[7,3] = W1[7,6] = 1, b1 -1, w2 -0.5; `iss_full_scale.logit` unit 1 has W1[1,0] and W1[1,1] 1 to 0, W1[1,2] 0 to 1, bias -1 and the no-flow entry 1 kept, and its output weight 0.8 to 1.5 (a line of the block). Development numbers, in-sample; nothing is validated.

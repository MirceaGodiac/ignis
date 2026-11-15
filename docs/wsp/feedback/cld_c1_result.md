# cld: candidate C1 (PROTOCOL 13.6), development, in-sample

- Study: `docs/wsp/research/phys_cld_study.md` (sha256 `fd966ea8a5c2fb2e73ca0d29c4f5128268a0942e20709956f961479e88754bf6`). Edits: `runs/dev/gen7/study/cld_c1.wsp` (sha256 `f7f10d4b52ad04486bd22a1cb3f5f616e16ae00f047366f39870b23bbd091979`), verbatim.
- Release: `sha256:541e9b2a232e0d33f1638650d69198a4153e2ba8d8d6257632e96285c7d99ab6` (`configs/models/law/cld/c1-fable`), a revision of cld D1 `sha256:a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f`.
- Element check: the release equals its parent except exactly the 20 elements the file writes (3 of them already carried the same value in the parent); 17 elements differ from the parent; mismatches: 0.
- Scores: the Rust forward (`tools/wsp/law_run.py`, `target-merged/release/ignis.exe`) through the lane scorer (`tools/wsp/ext/law_score.py`, the lane's own `score()`), by `tools/wsp/ext/law_revision.py dev`; train and validation days together (PROTOCOL 13.4: in-sample), on the cases parent and candidate both answer; the other releases and the baselines on the same cases. Lower is better. Lane case files (sha256): `cases_train.jsonl` `6dd959837ff9…`, `cases_validation_inputs.jsonl` `415af9416dda…`, `cases_validation_outcomes.jsonl` `654b55be36c0…`.

## target loss

| metric | partition | cld D1 | cld C1 | cld W0 | cld2 s1 | incidence (simple) | by fuel | loss by dilution ramp | loss, classical LFL at the stoichiometric surface | radiometer, Roper length plus fuel offset |
|---|---|---|---|---|---|---|---|---|---|---|
| integrated Brier | pooled | 0.3174 | 0.0431 | 0.3862 | 0.1428 | 0.3277 | 0.2745 | 0.2282 | 0.4552 | - |
| integrated Brier | train | 0.1253 | 0.0195 | 0.1569 | 0.0702 | 0.1493 | 0.1405 | 0.0705 | 0.1892 | - |
| integrated Brier | validation | 0.5139 | 0.0935 | 0.5925 | 0.2420 | 0.4782 | 0.3884 | 0.3282 | 0.6762 | - |
| Brier of lost before fuel-off | pooled | 0.0805 | 0.0415 | 0.0988 | 0.0647 | 0.1139 | 0.1066 | 0.0815 | 0.1289 | - |
| Brier of lost before fuel-off | train | 0.0535 | 0.0296 | 0.0660 | 0.0485 | 0.0855 | 0.0811 | 0.0587 | 0.0859 | - |
| Brier of lost before fuel-off | validation | 0.1728 | 0.0821 | 0.2107 | 0.1201 | 0.2111 | 0.1933 | 0.1592 | 0.2759 | - |
| integrated Brier, mean of day means | pooled | 0.1555 | 0.0787 | 0.1699 | 0.1130 | 0.1568 | 0.1457 | 0.1117 | 0.1943 | - |
| integrated Brier, mean of day means | train | 0.1179 | 0.0664 | 0.1302 | 0.0925 | 0.1284 | 0.1233 | 0.0893 | 0.1536 | - |
| integrated Brier, mean of day means | validation | 0.2747 | 0.1176 | 0.2957 | 0.1781 | 0.2470 | 0.2163 | 0.1826 | 0.3233 | - |

## target radiometer

| metric | partition | cld D1 | cld C1 | cld W0 | cld2 s1 | incidence (simple) | by fuel | loss by dilution ramp | loss, classical LFL at the stoichiometric surface | radiometer, Roper length plus fuel offset |
|---|---|---|---|---|---|---|---|---|---|---|
| CRPS of log10 V | pooled | 0.1186 | 0.1186 | 0.1414 | 0.1186 | 0.2390 | 0.2201 | - | - | 0.1284 |
| CRPS of log10 V | train | 0.1234 | 0.1234 | 0.1500 | 0.1234 | 0.2328 | 0.2096 | - | - | 0.1166 |
| CRPS of log10 V | validation | 0.1003 | 0.1003 | 0.1083 | 0.1003 | 0.2627 | 0.2605 | - | - | 0.1743 |
| mean of day means | pooled | 0.1167 | 0.1167 | 0.1416 | 0.1167 | 0.2523 | 0.2080 | - | - | 0.1474 |
| mean of day means | train | 0.1205 | 0.1205 | 0.1491 | 0.1205 | 0.2452 | 0.1959 | - | - | 0.1323 |
| mean of day means | validation | 0.1043 | 0.1043 | 0.1176 | 0.1043 | 0.2747 | 0.2469 | - | - | 0.1956 |

## by day: loss (per case: mean Brier over the horizons at risk; calibration of lost before fuel-off)

Cases: cld D1 256, cld C1 256, cld W0 256, cld2 s1 256.

| day | cld D1 | cld C1 | cld W0 | cld2 s1 |
|---|---|---|---|---|
| 17319 | 0.0004 | 0.0002 | 0.0002 | 0.0009 |
| 17324 | 0.1149 | 0.0533 | 0.1402 | 0.0528 |
| 17342 | 0.0379 | 0.0355 | 0.0444 | 0.0379 |
| 18003 | 0.2739 | 0.0765 | 0.2897 | 0.1220 |
| 18005 | 0.0678 | 0.0715 | 0.0713 | 0.0678 |
| 18008 | 0.3846 | 0.1821 | 0.4031 | 0.3174 |
| 18011 | 0.0586 | 0.0045 | 0.0641 | 0.0258 |
| 18022 | 0.2900 | 0.2463 | 0.2934 | 0.2900 |
| 18026 | 0.1240 | 0.0674 | 0.1314 | 0.1240 |
| 18032 | 0.0209 | 0.0043 | 0.0218 | 0.0209 |
| 18043 | 0.0003 | 0.0002 | 0.0002 | 0.0013 |
| 18053 | 0.0001 | 0.0001 | 0.0001 | 0.0001 |
| 18145 | 0.4510 | 0.4511 | 0.4510 | 0.4510 |
| 18177 | 0.2202 | 0.2202 | 0.2202 | 0.2202 |
| 18192 | 0.0001 | 0.0001 | 0.0001 | 0.0001 |
| 18214 | 0.0001 | 0.0001 | 0.0001 | 0.0001 |
| 18218 | 0.0001 | 0.0001 | 0.0001 | 0.0001 |
| 18235 | 0.0001 | 0.0001 | 0.0001 | 0.0001 |
| 18239 | 0.0001 | 0.0001 | 0.0001 | 0.0001 |
| 18241 | 0.0356 | 0.0334 | 0.0358 | 0.0190 |
| 18249 | 0.0101 | 0.0300 | 0.0101 | 0.0221 |
| 18253 | 0.0003 | 0.0001 | 0.0003 | 0.0003 |
| 18257 | 0.0002 | 0.0002 | 0.0002 | 0.0002 |
| 21046 | 0.0002 | 0.0001 | 0.0002 | 0.0002 |
| 21062 | 0.0005 | 0.0002 | 0.0003 | 0.0005 |

Candidate lower than the parent on **21 of 25** days whose forecasts change (25 days).

## by day: radiometer (CRPS of log10 V; error = median forecast - outcome, in log10 V)

Cases: cld D1 195, cld C1 195, cld W0 195, cld2 s1 195.

| day | cld D1 | cld C1 | cld W0 | cld2 s1 |
|---|---|---|---|---|
| 17324 | 0.1273 | 0.1273 | 0.1922 | 0.1273 |
| 17342 | 0.1204 | 0.1204 | 0.1716 | 0.1204 |
| 18003 | 0.1164 | 0.1164 | 0.1596 | 0.1164 |
| 18011 | 0.1002 | 0.1002 | 0.1279 | 0.1002 |
| 18022 | 0.1061 | 0.1061 | 0.1287 | 0.1061 |
| 18026 | 0.1018 | 0.1018 | 0.1120 | 0.1018 |
| 18032 | 0.0909 | 0.0909 | 0.0947 | 0.0909 |
| 18043 | 0.1623 | 0.1623 | 0.2238 | 0.1623 |
| 18053 | 0.1521 | 0.1521 | 0.2097 | 0.1521 |
| 18145 | 0.0943 | 0.0943 | 0.0859 | 0.0943 |
| 18177 | 0.1112 | 0.1112 | 0.1044 | 0.1112 |
| 18192 | 0.1101 | 0.1101 | 0.1427 | 0.1101 |
| 18214 | 0.1947 | 0.1947 | 0.2445 | 0.1947 |
| 18218 | 0.1304 | 0.1304 | 0.1691 | 0.1304 |
| 18235 | 0.1061 | 0.1061 | 0.1217 | 0.1061 |
| 18239 | 0.1172 | 0.1172 | 0.1457 | 0.1172 |
| 18249 | 0.0985 | 0.0985 | 0.1162 | 0.0985 |
| 18253 | 0.1046 | 0.1046 | 0.1129 | 0.1046 |
| 18257 | 0.1108 | 0.1108 | 0.1030 | 0.1108 |
| 21046 | 0.0968 | 0.0968 | 0.1048 | 0.0968 |
| 21062 | 0.0978 | 0.0978 | 0.1022 | 0.0978 |

Candidate lower than the parent on **0 of 0** days whose forecasts change (21 days).

Agreement with the logged what-if run of C1 (2026-11-15T07:53:57+00:00, the Python copy of the forward): largest pooled difference over every metric 5.6e-17 (round-off); days lower identical: 21 of 25; 0 of 0.
**Reading (13.6):** loss (integrated Brier, pooled) 0.0431 against cld D1 0.3174 (cld W0 0.3862, cld2 0.1428; best baseline: incidence by dilution ramp 0.2282), lower on 21 of 25 days whose forecasts change (84 %, at least two thirds); radiometer identical to the parent (0.1186; 0 days change), so not worse on the other target: **kept**. C1 replaces D1 as the cld model for the final run; it is also below every baseline pooled and on train (0.0195) and validation (0.0935) apart. Development numbers, in-sample; nothing is validated.

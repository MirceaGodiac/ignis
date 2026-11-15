# spice: candidate P3 (PROTOCOL 13.6), development, in-sample

- Study: `docs/wsp/research/phys_spice_study.md` (sha256 `33f9808be1fe798355b9d5df29dd0695b63467050240dfe6f8ee92473fd4bb05`). Edits: `runs/dev/gen7/study/spice_p3.wsp` (sha256 `4338268390c3d6c3c6fa6c55cf6eb2f839ce871dfcc1dfe2b6ad37e6a4a41b51`), verbatim.
- Release: `sha256:000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0` (`configs/models/law/spice/p3-fable`), a revision of spice P2 `sha256:f1fe69f0956eddbbc4333187b1668d0e3b8e6ddc8af649872408e99581e3258a`.
- Element check: the release equals its parent except exactly the 35 elements the file writes (0 of them already carried the same value in the parent); 35 elements differ from the parent; mismatches: 0.
- Scores: the Rust forward (`tools/wsp/law_run.py`, `target-merged/release/ignis.exe`) through the lane scorer (`tools/wsp/ext/law_score.py`, the lane's own `score()`), by `tools/wsp/ext/law_revision.py dev`; train and validation days together (PROTOCOL 13.4: in-sample), on the cases parent and candidate both answer; the other releases and the baselines on the same cases. Lower is better. Lane case files (sha256): `cases_train.jsonl` `79c407c20b16…`, `cases_validation_inputs.jsonl` `079c908493c3…`, `cases_validation_inputs_given_flow.jsonl` `0222124e6109…`, `cases_validation_outcomes.jsonl` `0dfc9d7e5055…`.

## target T1

| metric | partition | spice P2 | spice P3 | spice W0 (variant B) | spice lit | B0 pooled climatology (simple) | B1 per-fuel climatology | Lin and Faeth coflow law, train sigma (T2 only) |
|---|---|---|---|---|---|---|---|---|
| quantile score x2, mm, mean of day means | pooled | 2.1292 | 2.1292 | 6.5136 | 3.1075 | 10.1567 | 6.6258 | - |
| quantile score x2, mm, mean of day means | train | 2.2162 | 2.2162 | 6.6998 | 3.1628 | 11.3160 | 7.0783 | - |
| quantile score x2, mm, mean of day means | validation | 1.8245 | 1.8245 | 5.8621 | 2.9140 | 6.0990 | 5.0419 | - |
| pooled mean | pooled | 2.1520 | 2.1520 | 6.9134 | 3.3739 | 9.9446 | 7.4851 | - |
| pooled mean | train | 2.2175 | 2.2175 | 7.0751 | 3.4494 | 10.5812 | 7.9425 | - |
| pooled mean | validation | 1.7590 | 1.7590 | 5.9432 | 2.9210 | 6.1253 | 4.7406 | - |

## target T2

| metric | partition | spice P2 | spice P3 | spice W0 (variant B) | spice lit | B0 pooled climatology (simple) | B1 per-fuel climatology | Lin and Faeth coflow law, train sigma (T2 only) |
|---|---|---|---|---|---|---|---|---|
| quantile score x2, mm, mean of day means | pooled | 1.9370 | 1.2519 | 3.0715 | 3.0715 | 10.1567 | 6.6258 | 1.8416 |
| quantile score x2, mm, mean of day means | train | 2.0864 | 1.3723 | 3.2254 | 3.2254 | 11.3160 | 7.0783 | 2.0076 |
| quantile score x2, mm, mean of day means | validation | 1.4139 | 0.8305 | 2.5329 | 2.5329 | 6.0990 | 5.0419 | 1.2606 |
| pooled mean | pooled | 1.8952 | 1.1240 | 3.0328 | 3.0328 | 9.9446 | 7.4851 | 2.1114 |
| pooled mean | train | 1.9839 | 1.1746 | 3.1144 | 3.1144 | 10.5812 | 7.9425 | 2.2575 |
| pooled mean | validation | 1.3630 | 0.8206 | 2.5431 | 2.5431 | 6.1253 | 4.7406 | 1.2349 |

## by day: T1 (quantile score x2, mm; error = ln median forecast - ln outcome)

Cases: spice P2 49, spice P3 49, spice W0 (variant B) 49, spice lit 49.

| day | spice P2 | spice P3 | spice W0 (variant B) | spice lit |
|---|---|---|---|---|
| 045 | 3.3082 | 3.3082 | 8.6482 | 4.9832 |
| 052 | 2.6435 | 2.6435 | 11.7902 | 3.7975 |
| 058 | 2.0861 | 2.0861 | 6.2318 | 3.2220 |
| 119 | 1.2819 | 1.2819 | 3.0536 | 0.8988 |
| 136 | 1.3659 | 1.3659 | 6.4304 | 2.9629 |
| 167 | 1.8443 | 1.8443 | 4.3789 | 2.0350 |
| 168 | 2.5288 | 2.5288 | 7.3336 | 3.5039 |
| 173 | 2.2831 | 2.2831 | 5.2937 | 2.8650 |
| 174 | 1.8207 | 1.8207 | 5.4623 | 3.6995 |

Candidate lower than the parent on **0 of 0** days whose forecasts change (9 days).

## by day: T2 (quantile score x2, mm; error = ln median forecast - ln outcome)

Cases: spice P2 49, spice P3 49, spice W0 (variant B) 49, spice lit 49.

| day | spice P2 | spice P3 | spice W0 (variant B) | spice lit |
|---|---|---|---|---|
| 045 | 3.8990 | 1.8339 | 4.6410 | 4.6410 |
| 052 | 2.2321 | 1.5077 | 3.3094 | 3.3094 |
| 058 | 1.6669 | 0.8759 | 2.9983 | 2.9983 |
| 119 | 0.7038 | 0.8197 | 2.5682 | 2.5682 |
| 136 | 1.0579 | 0.7617 | 2.6046 | 2.6046 |
| 167 | 1.9597 | 1.2270 | 2.5159 | 2.5159 |
| 168 | 2.7188 | 2.4845 | 3.9052 | 3.9052 |
| 173 | 1.7699 | 0.8992 | 2.4612 | 2.4612 |
| 174 | 1.4246 | 0.8576 | 2.6397 | 2.6397 |

Candidate lower than the parent on **8 of 9** days whose forecasts change (9 days).

Agreement with the logged what-if run of P3 (2026-11-15T08:46:33+00:00, the Python copy of the forward): largest pooled difference over every metric 8.9e-16 (round-off); days lower identical: 0 of 0; 8 of 9.
**Reading (13.6):** flame length given the flow (T2, quantile score x2 mm, mean of day means, pooled) 1.2519 against spice P2 1.9370 (W0 and lit 3.0715; best baseline: Lin and Faeth coflow law 1.8416), lower on 8 of 9 days whose forecasts change (89 %, at least two thirds; higher on day 119, 0.704 to 0.820); lower on train (1.3723 against 2.0864) and validation (0.8305 against 1.4139) apart, and now below the Lin and Faeth baseline pooled and on both partitions; smoke-point length (T1) identical to P2 (2.1292; 0 days change): **kept**. P3 replaces P2 as the spice model for the final run. Nine days in all. Development numbers, in-sample; nothing is validated.

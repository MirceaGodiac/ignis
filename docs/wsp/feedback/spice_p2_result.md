# spice: candidate P2 (PROTOCOL 13.6), development, in-sample

- Study: `docs/wsp/research/phys_spice_study.md` (sha256 `df53f43e936b41877b3ed74c0d611d4ff2975174a78396fe3ca5a10027185bcb`). Edits: `runs/dev/gen7/study/spice_p2.wsp` (sha256 `63530856c22440ae2096655f03d1e9343538799161cd504ff6e92b065d98105c`), verbatim.
- Release: `sha256:f1fe69f0956eddbbc4333187b1668d0e3b8e6ddc8af649872408e99581e3258a` (`configs/models/law/spice/p2-fable`), a revision of spice W0 (variant B) `sha256:93e750242725a5a1fb9d38f4178b23605cac636f210bf9f3f8bc8e8150e2f2a8`.
- Element check: the release equals its parent except exactly the 55 elements the file writes (15 of them already carried the same value in the parent); 40 elements differ from the parent; mismatches: 0.
- Scores: the Rust forward (`tools/wsp/law_run.py`, `target-merged/release/ignis.exe`) through the lane scorer (`tools/wsp/ext/law_score.py`, the lane's own `score()`), by `tools/wsp/ext/law_revision.py dev`; train and validation days together (PROTOCOL 13.4: in-sample), on the cases parent and candidate both answer; the other releases and the baselines on the same cases. Lower is better. Lane case files (sha256): `cases_train.jsonl` `79c407c20b16…`, `cases_validation_inputs.jsonl` `079c908493c3…`, `cases_validation_inputs_given_flow.jsonl` `0222124e6109…`, `cases_validation_outcomes.jsonl` `0dfc9d7e5055…`.

## target T1

| metric | partition | spice W0 (variant B) | spice P2 | spice lit | B0 pooled climatology (simple) | B1 per-fuel climatology | Lin and Faeth coflow law, train sigma (T2 only) |
|---|---|---|---|---|---|---|---|
| quantile score x2, mm, mean of day means | pooled | 6.5136 | 2.1292 | 3.1075 | 10.1567 | 6.6258 | - |
| quantile score x2, mm, mean of day means | train | 6.6998 | 2.2162 | 3.1628 | 11.3160 | 7.0783 | - |
| quantile score x2, mm, mean of day means | validation | 5.8621 | 1.8245 | 2.9140 | 6.0990 | 5.0419 | - |
| pooled mean | pooled | 6.9134 | 2.1520 | 3.3739 | 9.9446 | 7.4851 | - |
| pooled mean | train | 7.0751 | 2.2175 | 3.4494 | 10.5812 | 7.9425 | - |
| pooled mean | validation | 5.9432 | 1.7590 | 2.9210 | 6.1253 | 4.7406 | - |

## target T2

| metric | partition | spice W0 (variant B) | spice P2 | spice lit | B0 pooled climatology (simple) | B1 per-fuel climatology | Lin and Faeth coflow law, train sigma (T2 only) |
|---|---|---|---|---|---|---|---|
| quantile score x2, mm, mean of day means | pooled | 3.0715 | 1.9370 | 3.0715 | 10.1567 | 6.6258 | 1.8416 |
| quantile score x2, mm, mean of day means | train | 3.2254 | 2.0864 | 3.2254 | 11.3160 | 7.0783 | 2.0076 |
| quantile score x2, mm, mean of day means | validation | 2.5329 | 1.4139 | 2.5329 | 6.0990 | 5.0419 | 1.2606 |
| pooled mean | pooled | 3.0328 | 1.8952 | 3.0328 | 9.9446 | 7.4851 | 2.1114 |
| pooled mean | train | 3.1144 | 1.9839 | 3.1144 | 10.5812 | 7.9425 | 2.2575 |
| pooled mean | validation | 2.5431 | 1.3630 | 2.5431 | 6.1253 | 4.7406 | 1.2349 |

## by day: T1 (quantile score x2, mm; error = ln median forecast - ln outcome)

Cases: spice W0 (variant B) 49, spice P2 49, spice lit 49.

| day | spice W0 (variant B) | spice P2 | spice lit |
|---|---|---|---|
| 045 | 8.6482 | 3.3082 | 4.9832 |
| 052 | 11.7902 | 2.6435 | 3.7975 |
| 058 | 6.2318 | 2.0861 | 3.2220 |
| 119 | 3.0536 | 1.2819 | 0.8988 |
| 136 | 6.4304 | 1.3659 | 2.9629 |
| 167 | 4.3789 | 1.8443 | 2.0350 |
| 168 | 7.3336 | 2.5288 | 3.5039 |
| 173 | 5.2937 | 2.2831 | 2.8650 |
| 174 | 5.4623 | 1.8207 | 3.6995 |

Candidate lower than the parent on **9 of 9** days whose forecasts change (9 days).

## by day: T2 (quantile score x2, mm; error = ln median forecast - ln outcome)

Cases: spice W0 (variant B) 49, spice P2 49, spice lit 49.

| day | spice W0 (variant B) | spice P2 | spice lit |
|---|---|---|---|
| 045 | 4.6410 | 3.8990 | 4.6410 |
| 052 | 3.3094 | 2.2321 | 3.3094 |
| 058 | 2.9983 | 1.6669 | 2.9983 |
| 119 | 2.5682 | 0.7038 | 2.5682 |
| 136 | 2.6046 | 1.0579 | 2.6046 |
| 167 | 2.5159 | 1.9597 | 2.5159 |
| 168 | 3.9052 | 2.7188 | 3.9052 |
| 173 | 2.4612 | 1.7699 | 2.4612 |
| 174 | 2.6397 | 1.4246 | 2.6397 |

Candidate lower than the parent on **9 of 9** days whose forecasts change (9 days).

Agreement with the logged what-if run of P2 (2026-11-15T08:13:23+00:00, the Python copy of the forward): largest pooled difference over every metric 1.8e-15 (round-off); days lower identical: 9 of 9; 9 of 9.
**Reading (13.6):** both targets change and both are lower. Smoke-point length (T1, quantile score x2 mm, mean of day means, pooled) 2.1292 against spice W0 6.5136 (spice lit 3.1075; best baseline: per-fuel climatology 6.6258), lower on 9 of 9 days; flame length given the flow (T2) 1.9370 against 3.0715 (spice lit 3.0715; best baseline: Lin and Faeth coflow law 1.8416), lower on 9 of 9 days: **kept**. P2 replaces W0 (variant B) as the spice model for the final run. On T2 it remains above the Lin and Faeth baseline pooled (1.937 against 1.842; pooled mean 1.895 against 2.111 is lower). Fewer than ten days in all (9). Development numbers, in-sample; nothing is validated.

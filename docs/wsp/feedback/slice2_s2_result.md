# slice2: candidate S2 (PROTOCOL 13.6), development, in-sample

- Study: `docs/wsp/research/phys_slice_study.md` (sha256 `a3bd91b2bad5dac175fae201526822ab557163f92abea418ff490546704d6384`). Edits: `runs/dev/gen7/study/slice2_s2.wsp` (sha256 `06e2f3b5c1d66d446127efa463fe9c60960e66b0b1c660e50362233c861974c6`), verbatim.
- Release: `sha256:0f5763f5749bb669b6ed9b70ac2629c8923838b3308f6094b2efcd7d8313388a` (`configs/models/law/slice2/s2-fable`), a revision of slice2 s1 `sha256:d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61`.
- Element check: the release equals its parent except exactly the 50 elements the file writes (16 of them already carried the same value in the parent); 34 elements differ from the parent; mismatches: 0.
- Scores: the Rust forward (`tools/wsp/law_run.py`, `target-merged/release/ignis.exe`) through the lane scorer (`tools/wsp/ext/law_score.py`, the lane's own `score()`), by `tools/wsp/ext/law_revision.py dev`; train and validation days together (PROTOCOL 13.4: in-sample), on the cases parent and candidate both answer; the other releases and the baselines on the same cases. Lower is better. Lane case files (sha256): `cases_train.jsonl` `fa4b72e3b9c6…`, `cases_validation_inputs.jsonl` `11aadddd5f16…`, `cases_validation_outcomes.jsonl` `843d4fe1f717…`.

## target blow_off

| metric | partition | slice2 s1 | slice2 S2 | slice W0 | slice D1 | incidence pooled (simple) | incidence by nominal tube | never (persistence) |
|---|---|---|---|---|---|---|---|---|
| Brier, mean of day means | pooled | 0.1089 | 0.0583 | 0.1089 | 0.0730 | 0.0509 | 0.0480 | 0.0538 |
| Brier, mean of day means | train | 0.1023 | 0.0500 | 0.1023 | 0.0601 | 0.0352 | 0.0335 | 0.0362 |
| Brier, mean of day means | validation | 0.1548 | 0.1159 | 0.1548 | 0.1631 | 0.1605 | 0.1493 | 0.1765 |
| pooled | pooled | 0.1151 | 0.0641 | 0.1151 | 0.0779 | 0.0561 | 0.0532 | 0.0596 |
| pooled | train | 0.1101 | 0.0575 | 0.1101 | 0.0671 | 0.0428 | 0.0410 | 0.0448 |
| pooled | validation | 0.1548 | 0.1159 | 0.1548 | 0.1631 | 0.1605 | 0.1493 | 0.1765 |

## target lifted

| metric | partition | slice2 s1 | slice2 S2 | slice W0 | slice D1 | incidence pooled (simple) | incidence by nominal tube | never (persistence) |
|---|---|---|---|---|---|---|---|---|
| Brier, mean of day means | pooled | 0.0333 | 0.0333 | 0.1474 | 0.1373 | 0.0297 | 0.0293 | 0.0306 |
| Brier, mean of day means | train | 0.0305 | 0.0305 | 0.1559 | 0.1490 | 0.0258 | 0.0249 | 0.0265 |
| Brier, mean of day means | validation | 0.0536 | 0.0536 | 0.0886 | 0.0555 | 0.0563 | 0.0596 | 0.0588 |
| pooled | pooled | 0.0368 | 0.0368 | 0.1695 | 0.1597 | 0.0320 | 0.0316 | 0.0331 |
| pooled | train | 0.0346 | 0.0346 | 0.1798 | 0.1730 | 0.0290 | 0.0280 | 0.0299 |
| pooled | validation | 0.0536 | 0.0536 | 0.0886 | 0.0555 | 0.0563 | 0.0596 | 0.0588 |

## by day: blow_off (Brier per line)

Cases: slice2 s1 302, slice2 S2 302, slice W0 302, slice D1 302.

| day | slice2 s1 | slice2 S2 | slice W0 | slice D1 |
|---|---|---|---|---|
| 040 | 0.0012 | 0.0021 | 0.0012 | 0.0012 |
| 048 | 0.0003 | 0.0009 | 0.0003 | 0.0003 |
| 058 | 0.1000 | 0.0927 | 0.1000 | 0.1000 |
| 061 | 0.5749 | 0.2085 | 0.5749 | 0.2796 |
| 065 | 0.1548 | 0.1159 | 0.1548 | 0.1631 |
| 069 | 0.0181 | 0.0254 | 0.0181 | 0.0181 |
| 072 | 0.0212 | 0.0203 | 0.0212 | 0.0212 |
| 079 | 0.0006 | 0.0001 | 0.0006 | 0.0006 |

Candidate lower than the parent on **5 of 8** days whose forecasts change (8 days).

## by day: lifted (Brier per line)

Cases: slice2 s1 302, slice2 S2 302, slice W0 302, slice D1 302.

| day | slice2 s1 | slice2 S2 | slice W0 | slice D1 |
|---|---|---|---|---|
| 040 | 0.0017 | 0.0017 | 0.0199 | 0.0199 |
| 048 | 0.0003 | 0.0003 | 0.0037 | 0.0027 |
| 058 | 0.0703 | 0.0703 | 0.2002 | 0.2002 |
| 061 | 0.1087 | 0.1087 | 0.7116 | 0.6648 |
| 065 | 0.0536 | 0.0536 | 0.0886 | 0.0555 |
| 069 | 0.0095 | 0.0095 | 0.1090 | 0.1090 |
| 072 | 0.0219 | 0.0219 | 0.0385 | 0.0385 |
| 079 | 0.0007 | 0.0007 | 0.0080 | 0.0080 |

Candidate lower than the parent on **0 of 0** days whose forecasts change (8 days).

Agreement with the logged what-if run of S2 (2026-11-15T08:08:03+00:00, the Python copy of the forward): largest pooled difference over every metric 3.5e-17 (round-off); days lower identical: 5 of 8; 0 of 0.
**Reading (13.6):** blow-off (Brier, mean of day means, pooled) 0.0583 against slice2 s1 0.1089 (slice D1 0.0730), lower on 5 of 8 days whose forecasts change: 8 days change (six or more), and 5 of 8 is short of two thirds, so by 13.6 S2 is **not kept** as the model; lifted identical to the parent (0.0333). It also stays above all three baselines pooled (incidence by nominal tube 0.0480, pooled incidence 0.0509, never 0.0538). As its author proposes, it is carried as a reported comparator; slice2 s1 stays the SLICE model. Development numbers, in-sample; nothing is validated.

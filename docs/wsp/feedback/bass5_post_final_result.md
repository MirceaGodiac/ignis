# Plastics quench speed: bass5, the post-final revision (PROTOCOL 13.8)

**Typed after the final run with every partition seen; in-sample on all data; no held-out score exists.**

- Dossier: `docs/wsp/research/phys_bass_post_final.md` (sha256 `7135fe26bf992b15a3c7d47cbc1a9a5b547c4979e59d9ab7415a1fb3ff0352ce`), "For the engineer" and "Edits". The Edits block (lines 130-142) is copied verbatim to `runs/dev/gen7/study/bass5_block.wsp`.
- Declaration `configs/models/law/bass5/decl.json`: the eleven bass4 features plus `flat_width` (index 11; m, ln). Targets, coverage and scoring rule as bass4. Rebuilt once (`slug!("bass5")` in `law_slugs.rs`).
- Input (`tools/wsp/ext/law_inputs.py`, `bass5_row`), quoting the dossier: "PMMA films and slabs: `width_m` as installed. Everything else (rods, spheres, sections, fabrics, Ultem), and a flat PMMA sample with no width recorded: 0.02".
- Root `sha256:1558a1c15d26b900b459ba473bc0f0c42ed332c22c896777b3af3d636643cf52`. Release **bass5 s1-fable `sha256:050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d`**. Edits: every line of bass4 s1-fable, then the block. Lineage: `tools/wsp/reassemble.py` (root and s1-fable, weights unchanged).
- Exposure: the exposure contract (`crates/ignis-core/src/contracts/wsp.rs`) refuses a dataset entry with the test partition ("an author never sees the test partition"). The meta therefore lists train and validation as datasets, and records the test-day cases in `sources`: read after the final run as study data, by PROTOCOL 13.8. The contract has no entry for that case; this is for the coordinator to settle.
- Element check: every element of bass5 equals bass4's except the 9 the block writes (4 domain lines and 5 weights), which carry its numbers (`runs/dev/gen7/bass5_parent_check.json`).
- Forecast check, on both codings:
  - "Kept burning": identical to bass4 on every case of every partition (0 differ).
  - Quench speed: different from bass4 on 6 cases, all 1 cm wide PMMA slabs. Three are the ones the author names: train M11 and M8, test M10.
  - The other three also change, because the law reaches every 1 cm slab: train M9 (out after a flow increase, not scored in the low-flow target), and validation 2013-04-26:72 and :73 (one-sided 2 mm slabs, not sustained, no quench outcome).
  - None of those three carries a scored quench outcome, so the scores move only through M11, M8 and M10.
- The author's logged run is reproduced through the Rust forward on version 2. Quench day means 0.1281, 0.1082, 2.4726 (train, validation, test), against the author's 0.128, 0.108, 2.473. The constant's are 0.8334, 0.6482, 2.7049 (the author's 0.833, 0.648, 2.705).
- Baselines on version 3 are rebuilt from the version 3 train file, the same way version 2's were built from version 2 (`tools/wsp/ext/bass1_baselines_v3.py`, written to `transfer_bass1/baselines_v3.json`):
  - constant quantiles of the quench speed 0.66, 1.20, 1.46, 1.66, 9.06 cm/s (version 2: 0.66, 1.22, 1.46, 1.80, 14.08);
  - on the display 40.40, 40.55, 40.80, 41.20, 58.00 (version 2: upper quantile 74.35);
  - "kept burning" train incidence 0.888, as on version 2.
  - Bhattacharjee et al. keeps its formula and its version 1 train log width, and is not in the table.
- Scores: Rust forward through the lane scorer (`tools/wsp/ext/law_score.py`, `BASS_CODING` = v3 or v2; the test partition is study data under 13.8).
  - Each cell is the mean of day means, with the case mean in brackets; quench in cm/s, quench_display in fan-display units, kept burning as Brier; lower is better.
  - Cases are those bass5 answers that carry a scored outcome; the baselines are scored on the same cases.
  - Version 3 scores 55 low-flow burns on train, 14 on validation and 17 on test (the data lane's counts). bass5 answers 43, 12 and 17 of them; the rest are geometries it refuses, as bass4 does.
- Producer: `runs/dev/gen7/bass5_dev.json`. Partitions: train, validation, test, and pooled (all three together).


## Version 3 (made after the final run; the coding that counts from here)

| target | partition | cases | bass4 | bass5 | constant (simple) | train incidence | always lit |
|---|---|---|---|---|---|---|---|
| quench | train | 43 | 0.1611 (0.2413) | 0.1143 (0.1585) | 0.6709 (0.9407) |  |  |
| quench | validation | 12 | 0.1082 (0.1057) | 0.1082 (0.1057) | 0.5466 (0.5480) |  |  |
| quench | test | 17 | 0.5633 (0.5130) | 0.1255 (0.1525) | 0.9157 (1.1706) |  |  |
| quench | pooled | 72 | 0.2392 (0.2828) | 0.1156 (0.1483) | 0.7011 (0.9295) |  |  |
| quench_display | train | 43 | 0.3675 (0.5687) | 0.2310 (0.3274) | 1.4437 (2.1235) |  |  |
| quench_display | validation | 12 | 0.2778 (0.2730) | 0.2778 (0.2730) | 1.1099 (1.1244) |  |  |
| quench_display | test | 17 | 1.6911 (1.5117) | 0.3476 (0.4053) | 2.6716 (3.3056) |  |  |
| quench_display | pooled | 72 | 0.6402 (0.7421) | 0.2652 (0.3367) | 1.6497 (2.2361) |  |  |
| sustain | train | 92 | 0.0220 (0.0189) | 0.0220 (0.0189) |  | 0.1036 (0.1053) | 0.1173 (0.1196) |
| sustain | validation | 27 | 0.1221 (0.0945) | 0.1221 (0.0945) |  | 0.1160 (0.0988) | 0.1333 (0.1111) |
| sustain | test | 40 | 0.0532 (0.0250) | 0.0532 (0.0250) |  | 0.1120 (0.0707) | 0.1282 (0.0750) |
| sustain | pooled | 159 | 0.0513 (0.0333) | 0.0513 (0.0333) |  | 0.1085 (0.0955) | 0.1236 (0.1069) |

## Version 2 (the coding of the final run)

| target | partition | cases | bass4 | bass5 | constant (simple) | train incidence | always lit |
|---|---|---|---|---|---|---|---|
| quench | train | 45 | 0.1661 (0.2868) | 0.1281 (0.2067) | 0.8334 (1.2660) |  |  |
| quench | validation | 12 | 0.1082 (0.1057) | 0.1082 (0.1057) | 0.6482 (0.6508) |  |  |
| quench | test | 17 | 2.9103 (1.5438) | 2.4726 (1.1833) | 2.7049 (2.1082) |  |  |
| quench | pooled | 74 | 0.7555 (0.5462) | 0.6372 (0.4147) | 1.2081 (1.3597) |  |  |
| quench_display | train | 45 | 0.6812 (1.1866) | 0.5705 (0.9529) | 2.0618 (3.2240) |  |  |
| quench_display | validation | 12 | 0.2778 (0.2730) | 0.2778 (0.2730) | 1.4301 (1.4446) |  |  |
| quench_display | test | 17 | 7.4058 (4.0160) | 6.0623 (2.9097) | 6.7190 (5.3626) |  |  |
| quench_display | pooled | 74 | 2.0765 (1.6884) | 1.7169 (1.2922) | 2.9621 (3.4268) |  |  |
| sustain | train | 92 | 0.0220 (0.0189) | 0.0220 (0.0189) |  | 0.1036 (0.1053) | 0.1173 (0.1196) |
| sustain | validation | 27 | 0.1221 (0.0945) | 0.1221 (0.0945) |  | 0.1160 (0.0988) | 0.1333 (0.1111) |
| sustain | test | 40 | 0.0765 (0.0477) | 0.0765 (0.0477) |  | 0.1120 (0.0707) | 0.1282 (0.0750) |
| sustain | pooled | 159 | 0.0576 (0.0390) | 0.0576 (0.0390) |  | 0.1085 (0.0955) | 0.1236 (0.1069) |

## Reading (PROTOCOL 13.8)

The rule: the revision is served if bass5 is lower than the constant on each of the three partitions for the quench speed, and "kept burning" is unchanged.

**Version 3** (the corrected coding, made after the final run; the one that counts from here):

| partition | bass5, quench (cm/s) | constant | lower? |
|---|---|---|---|
| train | 0.1143 (0.1585) | 0.6709 (0.9407) | yes |
| validation | 0.1082 (0.1057) | 0.5466 (0.5480) | yes |
| test | 0.1255 (0.1525) | 0.9157 (1.1706) | yes |

- On the display scale, bass5 is also lower than the constant on all three partitions.
- "Kept burning" is identical to bass4 on every case.
- **By 13.8, the bass5 quench speed is served.** Labelled "typed after the final run with every partition seen; in-sample on all data; no held-out score exists"; its scores describe fit and are not test results.

**Version 2**, for the record:
- bass5 is lower than the constant on train (0.1281 against 0.8334), validation (0.1082 against 0.6482) and test (2.4726 against 2.7049).
- The test gap is small. It was dominated by O13a, a burn mis-coded in version 2 that version 3 corrects.

**What version 3 changes for bass4 too.**
- bass4's test quench score falls from 2.9103 to 0.5633 (day mean) on version 3, mostly through O13a. That burn becomes a floor quench, which bass4 already forecast well.
- bass5 then removes the M10 slab loss: test 0.5633 to 0.1255.
- The validation partition is the same in both versions, and bass5 does not change any scored validation quench burn.


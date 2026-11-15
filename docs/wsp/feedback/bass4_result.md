# Plastics and fabrics: bass4 (PROTOCOL 13.6), development, in-sample

- Study: `docs/wsp/research/phys_bass_study2.md`, section E ("For the engineer" and "Edits"). The Edits block (lines 253-271) is copied verbatim to `runs/dev/gen7/study/bass4_block.wsp`.
  - The SIBAL numbers are zero-shot by their author: the expectation was written before any fabric case was opened (section A).
  - The single Nomex and Ultem floor is data-informed.
  - The comparators S1 and Q1 of section E are not assembled: the study says "not part of the model unless the coordinator decides so".
- Declaration `configs/models/law/bass4/decl.json`: the nine bass3 features plus `is_sibal` and `is_nomex_ultem` (indices 9, 10, identity), targets as bass3. A second rebuild, after the one for efield3 and cld3: bass4 arrived once those were built. efield3 and cld3 reassemble unchanged on the new binary.
- Inputs (`tools/wsp/ext/law_inputs.py`, `bass4_row`), from condition fields only:
  - `is_sibal`: 1 if `material_class` is `fabric_sibal`;
  - `is_nomex_ultem`: 1 if it is `fabric_nomex` or `ultem_sheet`;
  - fabric and Ultem rows: kept burning only, with the study's values (`nose_radius` 0.01, `heat_time` 200, `is_rod` 0, `flat_thickness` 0.00032 / 0.0003 / 0.00025, `u_pot1` as any case, `is_one_sided` 0);
  - `is_concurrent`: 1 concurrent, 0 opposed, 0.5 where no direction is recorded (15 of the 19 burns);
  - their quench speed stays refused;
  - PMMA rows are bass3's, with both indicators 0.
- Root `sha256:27898a87318921a4ba165a65a18be41379873e3c7305bff6ed16d51891c4ce8b`. Release **bass4 s1-fable `sha256:ceefc15d4cb8cf658772ec3603c9dabd6565fb3db6528bb35c2205f0d3bdab86`**. Edits: every line of bass3 s1-fable, then the block (no overlap).
- Checks:
  - Every element of bass4 equals bass3's except the 14 the block writes, which carry its numbers (`runs/dev/gen7/third_laws_parent_check.json`).
  - **Every PMMA forecast is identical to bass3**, on both codings: 0 quench and 0 non-fabric kept-burning forecasts differ, on train and validation.
  - Fabric kept-burning answered: bass3 0 of 19, bass4 19 of 19 (18 train, 1 validation).
  - Reassembly: weights unchanged.
- What-if tool: `bass4` added, with the flag `--coding v1|v2` (alias `--cases`). It sets `BASS_CODING` for the plastics and the SLICE case version for slice. It is printed in the header and logged as `coding` in each line, so no wrapper is needed any more.
- Scores: Brier of kept burning, mean of day means (case mean in brackets); lower is better. Producer: `runs/dev/gen7/bass4_dev.json`.

| coding | burns | bass4 | train incidence | always lit |
|---|---|---|---|---|
| v1 | all (19) | 0.0509 (0.0529) | 0.3954 (0.3801) | 0.4933 (0.4737) |
| v1 | SIBAL (9) | 0.0002 (0.0003) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| v1 | Nomex (9) | 0.1230 (0.1002) | 0.6777 (0.7023) | 0.8571 (0.8889) |
| v1 | Ultem (1) | 0.1015 (0.1015) | 0.7885 (0.7885) | 1.0000 (1.0000) |
| v1 | Nomex and Ultem (10) | 0.1266 (0.1003) | 0.6777 (0.7109) | 0.8571 (0.9000) |
| v2 | all (19) | 0.0509 (0.0529) | 0.3954 (0.3801) | 0.4933 (0.4737) |
| v2 | SIBAL (9) | 0.0002 (0.0003) | 0.0125 (0.0125) | 0.0000 (0.0000) |
| v2 | Nomex (9) | 0.1230 (0.1002) | 0.6777 (0.7023) | 0.8571 (0.8889) |
| v2 | Ultem (1) | 0.1015 (0.1015) | 0.7885 (0.7885) | 1.0000 (1.0000) |
| v2 | Nomex and Ultem (10) | 0.1266 (0.1003) | 0.6777 (0.7109) | 0.8571 (0.9000) |

Kept burning on the fabric and Ultem burns, by day (both codings give the same numbers; v2 shown):

| day | bass4 | train incidence | always lit |
|---|---|---|---|
| 2012-03-30 | 0.0000 | 0.0125 | 0.0000 |
| 2012-04-05 | 0.0000 | 0.0125 | 0.0000 |
| 2012-05-10 | 0.0051 | 0.2712 | 0.3333 |
| 2012-06-05 | 0.0141 | 0.7885 | 1.0000 |
| 2012-06-28 | 0.3489 | 0.0125 | 0.0000 |
| 2012-07-03 | 0.0001 | 0.0125 | 0.0000 |
| 2012-07-19 | 0.0308 | 0.4781 | 0.6000 |
| 2014-02-19 | 0.0666 | 0.7885 | 1.0000 |
| 2014-07-28 | 0.0106 | 0.7885 | 1.0000 |
| 2014-08-01 | 0.0327 | 0.7885 | 1.0000 |

Each burn (inputs, forecast, outcome):

| burn | material | direction | O2 | u cm/s | P(kept burning) | kept burning |
|---|---|---|---|---|---|---|
| PSI-25:F1 | Nomex | concurrent | 0.222 | 9.56 | 0.258 | 0 |
| PSI-25:F2 | Nomex | concurrent | 0.21 | 15.10 | 0.103 | 0 |
| PSI-25:F3 | Nomex | concurrent | 0.217 | 8.05 | 0.181 | 0 |
| PSI-26:2012-03-30:1 | SIBAL | not stated (0.5) | 0.221 | 10.07 | 0.995 | 1 |
| PSI-26:2012-04-05:7 | SIBAL | not stated (0.5) | 0.219 | 10.07 | 0.994 | 1 |
| PSI-26:2012-04-05:8 | SIBAL | not stated (0.5) | 0.219 | 5.03 | 0.994 | 1 |
| PSI-26:2012-05-10:10 | SIBAL | not stated (0.5) | 0.2163 | 11.07 | 0.991 | 1 |
| PSI-26:2012-05-10:12 | Nomex | not stated (0.5) | 0.2163 | 15.10 | 0.115 | 0 |
| PSI-26:2012-05-10:4 | SIBAL | not stated (0.5) | 0.2163 | 22.14 | 0.954 | 1 |
| PSI-26:2012-06-05:12-R | Nomex | not stated (0.5) | 0.2117 | 9.06 | 0.119 | 0 |
| PSI-26:2012-06-05:12-R2 | Nomex | not stated (0.5) | 0.2117 | 9.06 | 0.119 | 0 |
| PSI-26:2012-06-28:14 | Nomex | not stated (0.5) | 0.2198 | 14.60 | 0.165 | 1 |
| PSI-26:2012-06-28:5 | SIBAL | not stated (0.5) | 0.2198 | 5.49 | 0.994 | 1 |
| PSI-26:2012-07-03:6 | SIBAL | opposed | 0.2222 | 9.53 | 0.993 | 1 |
| PSI-26:2012-07-19:13 | Nomex | not stated (0.5) | 0.2253 | 15.10 | 0.227 | 0 |
| PSI-26:2012-07-19:15 | Nomex | not stated (0.5) | 0.2253 | 30.20 | 0.034 | 0 |
| PSI-26:2012-07-19:17 | Ultem | not stated (0.5) | 0.2253 | 8.05 | 0.319 | 0 |
| PSI-26:2012-07-19:2 | SIBAL | not stated (0.5) | 0.2272 | 13.26 | 0.995 | 1 |
| PSI-26:2012-07-19:3 | SIBAL | not stated (0.5) | 0.2267 | 13.26 | 0.995 | 1 |

All plastics targets, pooled, each release on its own answered cases (mean of day means, case mean):

| coding | target | bass3 | bass4 | bass4 without the fabric burns |
|---|---|---|---|---|
| v1 | quench | 0.1310 (0.2020), 52 | 0.1310 (0.2020), 52 | 0.1310 (0.2020), 52 |
| v1 | sustain | 0.0233 (0.0254), 101 | 0.0312 (0.0297), 120 | 0.0233 (0.0254), 101 |
| v2 | quench | 0.1522 (0.2487), 57 | 0.1522 (0.2487), 57 | 0.1522 (0.2487), 57 |
| v2 | sustain | 0.0467 (0.0328), 100 | 0.0506 (0.0360), 119 | 0.0467 (0.0328), 100 |

**Against the expectation.** The study expected about 0.054 against 0.38 (incidence) and 0.47 (always lit). Realised, case mean:

| | expected | realised |
|---|---|---|
| bass4, all 19 burns | about 0.054 | 0.0529 |
| incidence | 0.38 | 0.3801 |
| always lit | 0.47 | 0.4737 |
| SIBAL | about 0.0005 | 0.0003 |
| Nomex and Ultem | about 0.10 | 0.1003 |

The one Nomex burn that held (2012-06-28:14) costs 0.70. The fabric outcomes are the same in both codings, so their scores are too.

**All plastics targets unchanged.**
- Quench: identical (bass4 answers no fabric quench).
- Kept burning without the fabric burns: identical to bass3 on both codings.
- With the fabrics, the pooled kept-burning number changes only because 19 burns are added. Their mean Brier (0.053) is above the PMMA mean (0.025 on v1, 0.033 on v2).
- All numbers are development, in-sample. On SIBAL the law is judged here only in cabin air at 5 cm/s or more (the study's section F).

**By partition, each release on its own answered cases** (as the final-run path scores it; `by_partition` in `runs/dev/gen7/bass4_dev.json`):

| coding | release | partition | answered (quench, kept burning) | quench, mean of day means | quench, case mean | kept burning, mean of day means | kept burning, case mean | quench_display, mean of day means |
|---|---|---|---|---|---|---|---|---|
| v1 | bass3 | train | 78, 78 | 0.1611 | 0.2413 | 0.0114 | 0.0144 | - |
| v1 | bass3 | validation | 28, 28 | 0.0169 | 0.0146 | 0.0459 | 0.0555 | - |
| v1 | bass4 | train | 78, 96 | 0.1611 | 0.2413 | 0.0254 | 0.0225 | - |
| v1 | bass4 | validation | 28, 29 | 0.0169 | 0.0146 | 0.0459 | 0.0535 | - |
| v2 | bass3 | train | 78, 78 | 0.1661 | 0.2868 | 0.0070 | 0.0099 | 0.6812 |
| v2 | bass3 | validation | 28, 28 | 0.1082 | 0.1057 | 0.1221 | 0.0981 | 0.2778 |
| v2 | bass4 | train | 78, 96 | 0.1661 | 0.2868 | 0.0220 | 0.0189 | 0.6812 |
| v2 | bass4 | validation | 28, 29 | 0.1082 | 0.1057 | 0.1221 | 0.0945 | 0.2778 |

# E-FIELD (PSI-22): efield3, the third law (PROTOCOL 13.6), development, in-sample

- Study: `docs/wsp/research/phys_efield_study.md` (sha256 `573d8dbc5abe440fc8cb097270b4b61908d69fd057aa3ab9c79e491c7458da6e`), section 3 "For the engineer" and section 4. Its typed block (lines 200-248) is copied verbatim to `runs/dev/gen7/study/efield3_block.wsp` (sha256 `3810450a…03ec`).
- Declaration `configs/models/law/efield3/decl.json`: the declaration of efield with `is_gas_jet`, `still_coflow_burner` and `strong_coflow` appended (indices 7, 8, 9, identity), and `flame_end` with causes `blowoff`, `press`, `other` (no scheduled stop, as in efield).
- One rebuild for both new slugs: `slug!("efield3")` and `slug!("cld3")` added to `crates/ignis-wsp/src/law_slugs.rs`, then `cargo build --release -p ignis-cli -p ignis-wsp-dev` into target-merged.
- Inputs (`tools/wsp/ext/law_inputs.py`, `efield3_extra`), quoting the study:
  - `is_gas_jet`: "`burner` = `gas_jet_2.1mm`";
  - `still_coflow_burner`: "`burner` = `coflow` and `u_coflow_m_s` = 0";
  - `strong_coflow`: "`u_coflow_m_s` >= 0.1".
- Where the inputs come from: `inputs.burner` and `inputs.u_coflow_m_s`, both known before the test (`docs/wsp/feedback/efield_planned_fields.md`). `sweep_v_max` is the planned maximum, as the law assumes: that file finds it equal to `sweep_v_max_planned` on every train and validation test.
- Two input rules:
  - A gas-jet test gets `still_coflow_burner` = 0, whatever its coflow field says.
  - A missing coflow leaves `strong_coflow` out. This refuses one train test, 18319U1 (its script has no coflow key); E1 already refuses it (no carbon flow, no levels).
- Answered: train 2735 levels / 122 tests, validation 1289 / 36, the same as E1.
- Root `sha256:f78c2ece35c2e4808e00edb7aca5c797fe668edb11d9f6250081ef1e9c38efd8`. Release **efield3 s1-fable `sha256:8575550b310950efde7da2f00a1964c9a4d0c393fd3fbcaf4e4f2f1f8f046f04`**.
- How the edits are built (`configs/models/law/efield3/s1-fable/edits.wsp`):
  - every line of the chain W0 to E1, with each later change written in place, then the block. E1 is `configs/models/law/efield/e1-fable`, its lines taken from `runs/dev/gen7/study/efield_e1.wsp`;
  - where the block writes ten columns and E1 writes seven, columns 0 to 6 go inside E1's lines (the same numbers) and columns 7 to 9 one element per line. This is form only, done by `tools/wsp/ext/law_merge_edits.py`, which I extended for it.
  - The meta says efield3 = E1's ion current plus the new flame-out law.
- Parent check. The study says "efield3 = E1 with these lines added or replaced", checked with `ignis wsp inspect --edits` on both releases:
  - The block writes 150 elements; all carry the block's numbers, and 58 of them differ from E1.
  - No other element of efield3 differs from E1 (`runs/dev/gen7/third_laws_parent_check.json`).
  - The Python forward of the what-if tool equals the Rust forward on every metric (largest difference 1.1e-16).
  - `tools/wsp/reassemble.py` rebuilds root and s1-fable with the weights unchanged.
- What-if tool: `efield3` added (`--base configs/models/law/efield3/s1-fable`). I logged no run.
- Nothing the study left open stopped this assembly.
- One scorer change: the end probability of T2 is now the sum of every declared cause, so `press` is included. Before, the scorer named `blowoff` and `other`. For two causes the sum is the same, so the registered numbers of efield and cld do not move.
- Scores are on the levels and tests all four releases answer, with the lane metrics of `tools/wsp/ext/law_score.py`. Pooled means train and validation cases scored together. Producer: `runs/dev/gen7/third_laws_dev.json`. Lower is better.

| release | partition | t1: CRPS uA, case mean | t1: mean of day means | t2: integrated Brier | t2: Brier of ends in window |
|---|---|---|---|---|---|
| W0 | train | 0.7202 | 0.6690 | 0.1761 | 0.1689 |
| W0 | validation | 0.5463 | 0.5320 | 0.0753 | 0.0762 |
| W0 | pooled | 0.6592 | 0.6324 | 0.1478 | 0.1484 |
| E1 | train | 0.2681 | 0.2487 | 0.1761 | 0.1689 |
| E1 | validation | 0.2252 | 0.2263 | 0.0753 | 0.0762 |
| E1 | pooled | 0.2530 | 0.2428 | 0.1478 | 0.1484 |
| efield2 s1 | train | 0.7857 | 0.7224 | 0.1015 | 0.1220 |
| efield2 s1 | validation | 0.6002 | 0.5384 | 0.0975 | 0.1596 |
| efield2 s1 | pooled | 0.7206 | 0.6733 | 0.1001 | 0.1303 |
| efield3 | train | 0.1913 | 0.1772 | 0.0555 | 0.0429 |
| efield3 | validation | 0.2455 | 0.2628 | 0.0850 | 0.0951 |
| efield3 | pooled | 0.2103 | 0.2000 | 0.0630 | 0.0545 |
| baseline: constant (simple) | train | 0.5313 | 0.4968 | 0.1397 | 0.1649 |
| baseline: constant (simple) | validation | 0.6202 | 0.7383 | 0.0884 | 0.1120 |
| baseline: constant (simple) | pooled | 0.5625 | 0.5612 | 0.1252 | 0.1533 |
| baseline: space-charge law up to saturation, train sigma (T1) | train | 0.4533 | 0.4225 |  |  |
| baseline: space-charge law up to saturation, train sigma (T1) | validation | 0.3742 | 0.4382 |  |  |
| baseline: space-charge law up to saturation, train sigma (T1) | pooled | 0.4256 | 0.4267 |  |  |
| baseline: stratified incidence (T2; a stratum absent from train takes the pooled incidence) | train |  |  | 0.0544 | 0.1175 |
| baseline: stratified incidence (T2; a stratum absent from train takes the pooled incidence) | validation |  |  | 0.1124 | 0.1456 |
| baseline: stratified incidence (T2; a stratum absent from train takes the pooled incidence) | pooled |  |  | 0.0697 | 0.1237 |

t1, by day (the lane's day value; cases in brackets for efield3):

| day | W0 | E1 | efield2 s1 | efield3 | constant (simple) | space-charge law up to saturation, train |
|---|---|---|---|---|---|---|
| 18073 | 0.6537 | 0.2355 | 0.8134 | 0.2355 (497) | 0.5435 | 0.3295 |
| 18078 | 0.3519 | 0.1860 | 0.3604 | 0.1860 (366) | 0.3314 | 0.2386 |
| 18080 | 0.1490 | 0.0934 | 0.1521 | 0.0934 (264) | 0.2165 | 0.1274 |
| 18092 | 1.1302 | 0.3502 | 1.0904 | 0.3502 (288) | 0.9554 | 0.8386 |
| 18095 | 0.4016 | 0.1634 | 0.3590 | 0.1634 (155) | 0.3402 | 0.2521 |
| 18102 | 0.1500 | 0.0674 | 0.1829 | 0.0674 (186) | 0.2105 | 0.1788 |
| 18106 | 0.2080 | 0.1523 | 0.2176 | 0.0383 (132) | 0.2984 | 0.2462 |
| 18113 | 0.8922 | 0.0877 | 1.1239 | 0.0628 (159) | 0.2201 | 0.3775 |
| 18130 | 0.5264 | 0.1399 | 0.5701 | 0.0506 (187) | 0.2699 | 0.3140 |
| 18137 | 1.9050 | 0.7956 | 2.4087 | 0.3873 (252) | 1.0139 | 0.9530 |
| 18282 | 0.4073 | 0.2005 | 0.3542 | 0.2000 (178) | 0.7015 | 0.3389 |
| 18285 | 0.9231 | 0.3648 | 0.7998 | 0.5223 (155) | 1.8243 | 0.9217 |
| 18298 | 0.1993 | 0.1189 | 0.1800 | 0.1073 (117) | 0.2541 | 0.2629 |
| 18312 | 0.9369 | 0.1229 | 0.8360 | 0.0841 (148) | 0.2461 | 0.4685 |
| 18319 | 0.6521 | 0.5628 | 0.6517 | 0.4513 (151) | 0.9924 | 0.5526 |

efield3 against E1 on t1: lower on **8 of 9** days whose scores differ; higher on 18285 (0.3648 to 0.5223).

t2, by day (the lane's day value; cases in brackets for efield3):

| day | W0 | E1 | efield2 s1 | efield3 | constant (simple) | stratified incidence (T2; a stratum abse |
|---|---|---|---|---|---|---|
| 18073 | 0.0012 | 0.0012 | 0.0290 | 0.0004 (7) | 0.0424 | 0.0000 |
| 18078 | 0.2129 | 0.2129 | 0.1127 | 0.2286 (9) | 0.1554 | 0.2494 |
| 18080 | 0.0078 | 0.0078 | 0.0590 | 0.0001 (7) | 0.0424 | 0.0000 |
| 18092 | 0.0032 | 0.0032 | 0.0032 | 0.0000 (11) | 0.0150 | 0.0000 |
| 18095 | 0.0038 | 0.0038 | 0.0038 | 0.0000 (8) | 0.0150 | 0.0000 |
| 18102 | 0.1494 | 0.1494 | 0.0685 | 0.0486 (8) | 0.1465 | 0.0486 |
| 18106 | 0.2503 | 0.2503 | 0.1252 | 0.1216 (11) | 0.2028 | 0.1100 |
| 18113 | 0.3999 | 0.3999 | 0.2277 | 0.0998 (14) | 0.2893 | 0.0981 |
| 18130 | 0.2941 | 0.2941 | 0.1432 | 0.1116 (14) | 0.2126 | 0.1004 |
| 18137 | 0.0010 | 0.0010 | 0.0010 | 0.0000 (12) | 0.0424 | 0.0000 |
| 18282 | 0.0911 | 0.0911 | 0.0707 | 0.0309 (13) | 0.0596 | 0.0263 |
| 18285 | 0.0019 | 0.0019 | 0.1141 | 0.0000 (9) | 0.0595 | 0.2327 |
| 18298 | 0.1931 | 0.1931 | 0.1687 | 0.2193 (9) | 0.1838 | 0.1180 |
| 18312 | 0.3390 | 0.3390 | 0.1632 | 0.1703 (13) | 0.2098 | 0.1435 |
| 18319 | 0.0019 | 0.0019 | 0.0878 | 0.0000 (9) | 0.0150 | 0.0164 |

efield3 against E1 on t2: lower on **13 of 15** days whose scores differ; higher on 18078 (0.2129 to 0.2286), 18298 (0.1931 to 0.2193).

t2 end in window, by day (the lane's day value; cases in brackets for efield3):

| day | W0 | E1 | efield2 s1 | efield3 | constant (simple) | stratified incidence (T2; a stratum abse |
|---|---|---|---|---|---|---|
| 18073 | 0.0044 | 0.0044 | 0.0633 | 0.0010 (7) | 0.0434 | 0.0000 |
| 18078 | 0.0938 | 0.0938 | 0.1516 | 0.0991 (9) | 0.1082 | 0.1111 |
| 18080 | 0.0303 | 0.0303 | 0.1951 | 0.0008 (7) | 0.0434 | 0.0000 |
| 18092 | 0.0086 | 0.0086 | 0.0086 | 0.0000 (11) | 0.0434 | 0.0000 |
| 18095 | 0.0135 | 0.0135 | 0.0135 | 0.0000 (8) | 0.0434 | 0.0000 |
| 18102 | 0.1320 | 0.1320 | 0.0772 | 0.0054 (8) | 0.1163 | 0.0312 |
| 18106 | 0.2031 | 0.2031 | 0.1674 | 0.1457 (11) | 0.2555 | 0.2273 |
| 18113 | 0.2399 | 0.2399 | 0.1077 | 0.0557 (14) | 0.2101 | 0.1250 |
| 18130 | 0.2116 | 0.2116 | 0.0724 | 0.0618 (14) | 0.2101 | 0.1429 |
| 18137 | 0.0011 | 0.0011 | 0.0011 | 0.0000 (12) | 0.0434 | 0.0000 |
| 18282 | 0.4074 | 0.4074 | 0.2555 | 0.0141 (13) | 0.3126 | 0.2485 |
| 18285 | 0.0018 | 0.0018 | 0.1088 | 0.0000 (9) | 0.0434 | 0.2130 |
| 18298 | 0.1888 | 0.1888 | 0.2935 | 0.2595 (9) | 0.2378 | 0.2259 |
| 18312 | 0.3736 | 0.3736 | 0.1964 | 0.1285 (13) | 0.3126 | 0.2842 |
| 18319 | 0.0080 | 0.0080 | 0.2449 | 0.0000 (9) | 0.0434 | 0.0744 |

efield3 against E1 on t2 end in window: lower on **13 of 15** days whose scores differ; higher on 18078 (0.0938 to 0.0991), 18298 (0.1888 to 0.2595).

**Against the study's expectation**

- Ion current: about 0.19 expected; 0.2103 realised (E1 0.2530).
  - As expected, three days are lower: 18137 (0.796 to 0.387), 18319 (0.563 to 0.451) and 18106 (0.152 to 0.038).
  - The study said the other days would stay within 0.01. That did not hold: 18113, 18130, 18312 and 18298 are lower by 0.01 to 0.09, and 18285 (a validation day) is **higher**, 0.365 to 0.522.
- Flame-out: 0.06 to 0.07 expected; 0.0630 realised (stratified incidence 0.0697).
  - As expected, it is lower on 18106, 18113, 18130, 18282 and 18312.
  - It is **higher** on 18298 (0.193 to 0.219), one of the days the study named as lower, and on 18078 (0.213 to 0.229). The study expected no day to go up.
- The three days that went up (two on T2, one on T1) are validation days. On the four validation days alone, efield3 is worse than E1 on both targets: T1 0.2455 against 0.2252, T2 0.0850 against 0.0753.

**Reading (13.6 keep rule, development, in-sample)**

- Ion current: pooled 0.2103 against E1 0.2530, lower on 8 of 9 days whose scores differ.
- Flame-out: pooled 0.0630 against 0.1478, lower on 13 of 15.
- **efield3 passes the keep rule against E1 on both targets**: each target is lower pooled and on more than two thirds of the days that change, and neither target is worse elsewhere.
- Stated beside the result: the gain is on the train days. On the validation days both targets are worse than E1.

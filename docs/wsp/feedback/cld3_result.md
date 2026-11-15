# CLD Flame (PSI-21): cld3, the third law (PROTOCOL 13.6), development, in-sample

- Study: `docs/wsp/research/phys_cld_study.md` (sha256 `fd966ea8a5c2fb2e73ca0d29c4f5128268a0942e20709956f961479e88754bf6`), section 5: the feature table and the typed block. The block (lines 198-209) is copied verbatim to `runs/dev/gen7/study/cld3_block.wsp` (sha256 `be59cbbb…bd0f`).
- Declaration `configs/models/law/cld3/decl.json`: the declaration of cld with `t_severe` (s, ln) appended (index 6), and `loss` with causes `blowoff`, `other`, `crossing`. The planned fuel-off stays the scheduled stop. Built in the same rebuild as efield3.
- Input (`tools/wsp/ext/law_inputs.py`, `cld3_row`), quoting the study: `t_severe` = "seconds from flame onset at which the schedule first holds both its lowest `x_fuel` and its highest `v_coflow_m_s` (the later of the two times, less `at_cutoff.onset_from_ignition_command_s`), and not less than 2 s".
- How it is computed:
  - The schedule is linear inside each segment, so a value is first held at a segment boundary: the start of the first segment, or the end of a segment.
  - The lowest fraction and the highest coflow are the ones cld already uses (`x_fuel_final`, `v_coflow_final`).
  - This is how the study's own table computed it (`runs/dev/gen7/study/cld_t3.py`), plus the 2 s floor, which that table did not apply.
- Refusals: only where cld already refuses (no window or no schedule). Answered: train 198, validation 61, the same as C1.
- Root `sha256:1dc481742265689ecef0a738023c1cd3631a4bb606bd0ec7a795873cd1a8e2f4`. Release **cld3 s1-fable `sha256:b9df52397944078288b5dc5363bd19d914afc930326b0d2cbe2acac72d7a9e4b`**.
- Edits: every line of the chain W0, D1, C1, with each later change written in place, then the block's lines. C1 is `configs/models/law/cld/c1-fable`, its lines taken from `runs/dev/gen7/study/cld_c1.wsp`. The block does not overlap C1.
- Parent check. The study says "cld3 = C1 with these lines added":
  - The block writes 18 elements; all carry its numbers.
  - No other element of cld3 differs from C1 (`runs/dev/gen7/third_laws_parent_check.json`).
  - The Python forward equals Rust on every metric, and reassembly gives the weights unchanged.
- What-if tool: `cld3` added. I logged no run.
- Nothing the study left open stopped this assembly.
- Scores are on the cases all four releases answer, with the lane metrics. Pooled means train and validation scored together. Producer: `runs/dev/gen7/third_laws_dev.json`. Lower is better.

| release | partition | loss: integrated Brier | loss: Brier of lost before fuel-off | loss: integrated Brier, mean of day means | radiometer: CRPS of log10 V | radiometer: mean of day means |
|---|---|---|---|---|---|---|
| D1 | train | 0.1253 | 0.0535 | 0.1179 | 0.1234 | 0.1205 |
| D1 | validation | 0.5139 | 0.1728 | 0.2747 | 0.1003 | 0.1043 |
| D1 | pooled | 0.3174 | 0.0805 | 0.1555 | 0.1186 | 0.1167 |
| C1 | train | 0.0195 | 0.0296 | 0.0664 | 0.1234 | 0.1205 |
| C1 | validation | 0.0935 | 0.0821 | 0.1176 | 0.1003 | 0.1043 |
| C1 | pooled | 0.0431 | 0.0415 | 0.0787 | 0.1186 | 0.1167 |
| cld2 s1 | train | 0.0702 | 0.0485 | 0.0925 | 0.1234 | 0.1205 |
| cld2 s1 | validation | 0.2420 | 0.1201 | 0.1781 | 0.1003 | 0.1043 |
| cld2 s1 | pooled | 0.1428 | 0.0647 | 0.1130 | 0.1186 | 0.1167 |
| cld3 | train | 0.0180 | 0.0281 | 0.0605 | 0.1234 | 0.1205 |
| cld3 | validation | 0.0610 | 0.0793 | 0.0866 | 0.1003 | 0.1043 |
| cld3 | pooled | 0.0334 | 0.0397 | 0.0668 | 0.1186 | 0.1167 |
| baseline: incidence (simple) | train | 0.1493 | 0.0855 | 0.1284 | 0.2328 | 0.2452 |
| baseline: incidence (simple) | validation | 0.4782 | 0.2111 | 0.2470 | 0.2627 | 0.2747 |
| baseline: incidence (simple) | pooled | 0.3277 | 0.1139 | 0.1568 | 0.2390 | 0.2523 |
| baseline: by fuel | train | 0.1405 | 0.0811 | 0.1233 | 0.2096 | 0.1959 |
| baseline: by fuel | validation | 0.3884 | 0.1933 | 0.2163 | 0.2605 | 0.2469 |
| baseline: by fuel | pooled | 0.2745 | 0.1066 | 0.1457 | 0.2201 | 0.2080 |
| baseline: loss by dilution ramp | train | 0.0705 | 0.0587 | 0.0893 |  |  |
| baseline: loss by dilution ramp | validation | 0.3282 | 0.1592 | 0.1826 |  |  |
| baseline: loss by dilution ramp | pooled | 0.2282 | 0.0815 | 0.1117 |  |  |
| baseline: loss, classical LFL at the stoichiometric surface | train | 0.1892 | 0.0859 | 0.1536 |  |  |
| baseline: loss, classical LFL at the stoichiometric surface | validation | 0.6762 | 0.2759 | 0.3233 |  |  |
| baseline: loss, classical LFL at the stoichiometric surface | pooled | 0.4552 | 0.1289 | 0.1943 |  |  |
| baseline: radiometer, Roper length plus fuel offset | train |  |  |  | 0.1166 | 0.1323 |
| baseline: radiometer, Roper length plus fuel offset | validation |  |  |  | 0.1743 | 0.1956 |
| baseline: radiometer, Roper length plus fuel offset | pooled |  |  |  | 0.1284 | 0.1474 |

loss, by day (the lane's day value; cases in brackets for cld3):

| day | D1 | C1 | cld2 s1 | cld3 | incidence (simple) | by fuel | loss by dilution ramp | loss, classical LFL at the stoichiometri |
|---|---|---|---|---|---|---|---|---|
| 17319 | 0.0025 | 0.0009 | 0.0046 | 0.0009 (9) | 0.0121 | 0.0296 | 0.0506 | 0.0000 |
| 17324 | 0.3136 | 0.0563 | 0.0853 | 0.0680 (4) | 0.3908 | 0.3216 | 0.0508 | 0.5403 |
| 17342 | 0.0648 | 0.0608 | 0.0648 | 0.0608 (2) | 0.0684 | 0.0701 | 0.0712 | 0.0852 |
| 18003 | 0.5276 | 0.0981 | 0.1837 | 0.0119 (4) | 0.4404 | 0.3636 | 0.0260 | 0.6110 |
| 18005 | 0.1013 | 0.1072 | 0.1013 | 0.1072 (9) | 0.1062 | 0.1048 | 0.1060 | 0.1111 |
| 18008 | 0.7226 | 0.2086 | 0.4576 | 0.1203 (10) | 0.6530 | 0.5310 | 0.5552 | 0.9083 |
| 18011 | 0.1906 | 0.0058 | 0.0515 | 0.0083 (13) | 0.1933 | 0.1781 | 0.0176 | 0.2528 |
| 18022 | 0.5481 | 0.3481 | 0.5481 | 0.2700 (4) | 0.4955 | 0.4495 | 0.5374 | 0.5862 |
| 18026 | 0.4007 | 0.1125 | 0.4007 | 0.1012 (12) | 0.3686 | 0.3046 | 0.4226 | 0.5099 |
| 18032 | 0.0606 | 0.0082 | 0.0606 | 0.0029 (13) | 0.0611 | 0.0674 | 0.0617 | 0.0691 |
| 18043 | 0.0014 | 0.0006 | 0.0067 | 0.0006 (10) | 0.0205 | 0.0495 | 0.0072 | 0.0000 |
| 18053 | 0.0004 | 0.0002 | 0.0004 | 0.0002 (14) | 0.0063 | 0.0155 | 0.0016 | 0.0000 |
| 18145 | 0.3943 | 0.3944 | 0.3943 | 0.3944 (4) | 0.3836 | 0.3878 | 0.3824 | 0.4018 |
| 18177 | 0.2598 | 0.2598 | 0.2598 | 0.2598 (7) | 0.2515 | 0.2561 | 0.2518 | 0.2653 |
| 18192 | 0.0001 | 0.0001 | 0.0001 | 0.0001 (10) | 0.0008 | 0.0003 | 0.0007 | 0.0000 |
| 18214 | 0.0001 | 0.0001 | 0.0001 | 0.0001 (13) | 0.0008 | 0.0003 | 0.0007 | 0.0000 |
| 18218 | 0.0002 | 0.0002 | 0.0002 | 0.0002 (17) | 0.0063 | 0.0015 | 0.0016 | 0.0000 |
| 18235 | 0.0001 | 0.0001 | 0.0001 | 0.0001 (16) | 0.0008 | 0.0003 | 0.0007 | 0.0000 |
| 18239 | 0.0002 | 0.0002 | 0.0002 | 0.0002 (14) | 0.0063 | 0.0015 | 0.0016 | 0.0000 |
| 18241 | 0.2178 | 0.1721 | 0.0952 | 0.1363 (6) | 0.3382 | 0.3850 | 0.0307 | 0.4310 |
| 18249 | 0.0771 | 0.1309 | 0.1068 | 0.1243 (11) | 0.0695 | 0.0782 | 0.1995 | 0.0862 |
| 18253 | 0.0004 | 0.0002 | 0.0004 | 0.0002 (13) | 0.0063 | 0.0015 | 0.0016 | 0.0000 |
| 18257 | 0.0004 | 0.0003 | 0.0004 | 0.0003 (15) | 0.0121 | 0.0020 | 0.0036 | 0.0000 |
| 21046 | 0.0009 | 0.0003 | 0.0009 | 0.0003 (16) | 0.0121 | 0.0020 | 0.0036 | 0.0000 |
| 21062 | 0.0016 | 0.0006 | 0.0016 | 0.0006 (10) | 0.0163 | 0.0396 | 0.0054 | 0.0000 |

cld3 against C1 on loss: lower on **7 of 9** days whose scores differ; higher on 17324 (0.0563 to 0.0680), 18011 (0.0058 to 0.0083).

radiometer, by day (the lane's day value; cases in brackets for cld3):

| day | D1 | C1 | cld2 s1 | cld3 | incidence (simple) | by fuel | radiometer, Roper length plus fuel offse |
|---|---|---|---|---|---|---|---|
| 17324 | 0.1273 | 0.1273 | 0.1273 | 0.1273 (1) | 0.1871 | 0.0769 | 0.1610 |
| 17342 | 0.1204 | 0.1204 | 0.1204 | 0.1204 (1) | 0.1670 | 0.0700 | 0.1329 |
| 18003 | 0.1164 | 0.1164 | 0.1164 | 0.1164 (1) | 0.1793 | 0.0743 | 0.1209 |
| 18011 | 0.1002 | 0.1002 | 0.1002 | 0.1002 (8) | 0.1651 | 0.0688 | 0.0821 |
| 18022 | 0.1061 | 0.1061 | 0.1061 | 0.1061 (1) | 0.3290 | 0.2078 | 0.0900 |
| 18026 | 0.1018 | 0.1018 | 0.1018 | 0.1018 (8) | 0.2554 | 0.1503 | 0.0699 |
| 18032 | 0.0909 | 0.0909 | 0.0909 | 0.0909 (12) | 0.1998 | 0.1034 | 0.0541 |
| 18043 | 0.1623 | 0.1623 | 0.1623 | 0.1623 (9) | 0.2486 | 0.2035 | 0.0753 |
| 18053 | 0.1521 | 0.1521 | 0.1521 | 0.1521 (14) | 0.1462 | 0.1857 | 0.0672 |
| 18145 | 0.0943 | 0.0943 | 0.0943 | 0.0943 (2) | 0.5044 | 0.4085 | 0.4706 |
| 18177 | 0.1112 | 0.1112 | 0.1112 | 0.1112 (5) | 0.4872 | 0.4001 | 0.4223 |
| 18192 | 0.1101 | 0.1101 | 0.1101 | 0.1101 (10) | 0.2234 | 0.1594 | 0.0789 |
| 18214 | 0.1947 | 0.1947 | 0.1947 | 0.1947 (13) | 0.4546 | 0.3740 | 0.1761 |
| 18218 | 0.1304 | 0.1304 | 0.1304 | 0.1304 (17) | 0.2610 | 0.1908 | 0.1316 |
| 18235 | 0.1061 | 0.1061 | 0.1061 | 0.1061 (16) | 0.2124 | 0.1759 | 0.0916 |
| 18239 | 0.1172 | 0.1172 | 0.1172 | 0.1172 (14) | 0.1858 | 0.2038 | 0.1165 |
| 18249 | 0.0985 | 0.0985 | 0.0985 | 0.0985 (9) | 0.2739 | 0.3582 | 0.2149 |
| 18253 | 0.1046 | 0.1046 | 0.1046 | 0.1046 (13) | 0.2331 | 0.2984 | 0.1659 |
| 18257 | 0.1108 | 0.1108 | 0.1108 | 0.1108 (15) | 0.1822 | 0.2499 | 0.1274 |
| 21046 | 0.0968 | 0.0968 | 0.0968 | 0.0968 (16) | 0.2363 | 0.3218 | 0.1885 |
| 21062 | 0.0978 | 0.0978 | 0.0978 | 0.0978 (10) | 0.1654 | 0.0874 | 0.0566 |

cld3 against C1 on radiometer: lower on **0 of 0** days whose scores differ; higher on none.

**Against the study's expectation**

- Integrated Brier: from 0.043 to about 0.03 expected; 0.0334 realised.
- Days 18003 (0.098 to 0.012) and 18008 (0.209 to 0.120) are lower, as expected.
- Day 18249 was to come back below D1: **not met**. C1 0.1309, cld3 0.1243, D1 0.0771.
- No change on days without a loss: met. All nine days that change have losses.
- Two of those nine days go up: 17324 (0.056 to 0.068) and 18011 (0.006 to 0.008).
- Radiometer: unchanged (0.1186).

**Reading (13.6 keep rule, development, in-sample)**

- Loss: pooled 0.0334 against C1 0.0431 (train 0.0180 against 0.0195; validation 0.0610 against 0.0935), lower on 7 of 9 days whose forecasts change.
- Radiometer: identical.
- **cld3 passes the keep rule against C1.**

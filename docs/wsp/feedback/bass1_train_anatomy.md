# bass1: train anatomy of the W0 law (w0-fable, `sha256:97c0daf9de84482ed7b125b3d41302521f5dd7281c0667241807d93387e239a2`)

TRAIN days only, written by `tools/wsp/ext/law_anatomy.py` after the registered look of PROTOCOL 13.2. For the revision the physicist will type; no number is changed here. Per-case file: `runs/dev/data_all/work/transfer_bass1/law_study_97c0daf9de84482e.jsonl` (baselines beside it as `law_study_baseline_*.jsonl`).

## 1. Score against the baselines (train, on the cases the release answers)

### target quench

- Asked of the release: 131; answered: 72; refused: 59; refusal reasons: {"not covered: fabric_sibal|None": 7, "not covered: fabric_nomex|None": 6, "not covered: pmma_sphere|wake": 17, "not covered: pmma_slab|None": 6, "not covered: fabric_sibal|opposed": 1, "not covered: candle|forward stagnation": 7, "not covered: ultem_sheet|None": 1, "not covered: wax_slab|None": 3, "not covered: fabric_nomex|concurrent": 3, "not covered: pmma_rod|None": 2, "not covered: pmma_rod|concurrent": 6}
| source | interval CRPS cm/s, mean of day means | case mean | on |
|---|---|---|---|
| **release** | 1.3296 | 1.5856 | cases the release answers |
| constant (simple) | 0.8406 | 1.0189 | cases the release answers |
| constant (simple) | 0.5518 | 0.7714 | all lane cases |
| Bhattacharjee et al. 2016, train log width | 1.0263 | 1.2261 | cases the release answers |
| Bhattacharjee et al. 2016, train log width | 0.6134 | 0.8947 | all lane cases |
**Reading (interval CRPS cm/s, mean of day means):** release 1.3296; simple baseline (constant (simple)) 0.8406; best baseline (constant (simple)) 0.8406: **loses (higher than the simple baseline)**.

### target sustain

- Asked of the release: 131; answered: 63; refused: 68; refusal reasons: {"not covered: fabric_sibal|None": 7, "not covered: fabric_nomex|None": 6, "not covered: pmma_sphere|wake": 17, "not covered: pmma_slab|None": 6, "not covered: fabric_sibal|opposed": 1, "not covered: candle|forward stagnation": 7, "not covered: ultem_sheet|None": 1, "not covered: wax_slab|None": 3, "not covered: fabric_nomex|concurrent": 3, "not covered: pmma_rod|None": 2, "not covered: pmma_rod|opposed": 15}
| source | Brier, mean of day means | case mean | on |
|---|---|---|---|
| **release** | 0.0147 | 0.0177 | cases the release answers |
| train incidence (simple) | 0.0368 | 0.0384 | cases the release answers |
| train incidence (simple) | 0.0881 | 0.0995 | all lane cases |
| always lit | 0.0312 | 0.0333 | cases the release answers |
| always lit | 0.0973 | 0.1120 | all lane cases |
**Reading (Brier, mean of day means):** release 0.0147; simple baseline (train incidence (simple)) 0.0368; best baseline (always lit) 0.0312: **beats the best baseline**.

## 2. Refusals on train (admission, by reason)

- holds_low_flow: 72 answered, 59 refused; reasons: {"feature flow_speed: condition \"flow_speed\" missing from the prefix": 59, "feature nose_radius: condition \"nose_radius\" missing from the prefix": 59, "feature heat_time: condition \"heat_time\" missing from the prefix": 59, "feature is_concurrent: condition \"is_concurrent\" missing from the prefix": 59}; by coverage: {"not_covered:fabric_sibal|None|refused": 7, "nose|admitted": 49, "not_covered:fabric_nomex|None|refused": 6, "not_covered:pmma_sphere|wake|refused": 17, "not_covered:pmma_slab|None|refused": 6, "not_covered:fabric_sibal|opposed|refused": 1, "not_covered:candle|forward stagnation|refused": 7, "not_covered:ultem_sheet|None|refused": 1, "not_covered:wax_slab|None|refused": 3, "flat|admitted": 23, "not_covered:fabric_nomex|concurrent|refused": 3, "not_covered:pmma_rod|None|refused": 2, "not_covered:pmma_rod|concurrent|refused": 6}
- holds_high_flow: 72 answered, 59 refused; reasons: {"feature flow_speed: condition \"flow_speed\" missing from the prefix": 59, "feature nose_radius: condition \"nose_radius\" missing from the prefix": 59, "feature heat_time: condition \"heat_time\" missing from the prefix": 59, "feature is_concurrent: condition \"is_concurrent\" missing from the prefix": 59}; by coverage: {"not_covered:fabric_sibal|None|refused": 7, "nose|admitted": 49, "not_covered:fabric_nomex|None|refused": 6, "not_covered:pmma_sphere|wake|refused": 17, "not_covered:pmma_slab|None|refused": 6, "not_covered:fabric_sibal|opposed|refused": 1, "not_covered:candle|forward stagnation|refused": 7, "not_covered:ultem_sheet|None|refused": 1, "not_covered:wax_slab|None|refused": 3, "flat|admitted": 23, "not_covered:fabric_nomex|concurrent|refused": 3, "not_covered:pmma_rod|None|refused": 2, "not_covered:pmma_rod|concurrent|refused": 6}
- oxygen_limit: 72 answered, 59 refused; reasons: {"feature flow_speed: condition \"flow_speed\" missing from the prefix": 59, "feature nose_radius: condition \"nose_radius\" missing from the prefix": 59, "feature heat_time: condition \"heat_time\" missing from the prefix": 59, "feature is_concurrent: condition \"is_concurrent\" missing from the prefix": 59}; by coverage: {"not_covered:fabric_sibal|None|refused": 7, "nose|admitted": 49, "not_covered:fabric_nomex|None|refused": 6, "not_covered:pmma_sphere|wake|refused": 17, "not_covered:pmma_slab|None|refused": 6, "not_covered:fabric_sibal|opposed|refused": 1, "not_covered:candle|forward stagnation|refused": 7, "not_covered:ultem_sheet|None|refused": 1, "not_covered:wax_slab|None|refused": 3, "flat|admitted": 23, "not_covered:fabric_nomex|concurrent|refused": 3, "not_covered:pmma_rod|None|refused": 2, "not_covered:pmma_rod|concurrent|refused": 6}
- quench_speed: 72 answered, 59 refused; reasons: {"feature flow_speed: condition \"flow_speed\" missing from the prefix": 59, "feature nose_radius: condition \"nose_radius\" missing from the prefix": 59, "feature heat_time: condition \"heat_time\" missing from the prefix": 59, "feature is_concurrent: condition \"is_concurrent\" missing from the prefix": 59}; by coverage: {"not_covered:fabric_sibal|None|refused": 7, "nose|admitted": 49, "not_covered:fabric_nomex|None|refused": 6, "not_covered:pmma_sphere|wake|refused": 17, "not_covered:pmma_slab|None|refused": 6, "not_covered:fabric_sibal|opposed|refused": 1, "not_covered:candle|forward stagnation|refused": 7, "not_covered:ultem_sheet|None|refused": 1, "not_covered:wax_slab|None|refused": 3, "flat|admitted": 23, "not_covered:fabric_nomex|concurrent|refused": 3, "not_covered:pmma_rod|None|refused": 2, "not_covered:pmma_rod|concurrent|refused": 6}
- blowoff_speed: 72 answered, 59 refused; reasons: {"feature flow_speed: condition \"flow_speed\" missing from the prefix": 59, "feature nose_radius: condition \"nose_radius\" missing from the prefix": 59, "feature heat_time: condition \"heat_time\" missing from the prefix": 59, "feature is_concurrent: condition \"is_concurrent\" missing from the prefix": 59}; by coverage: {"not_covered:fabric_sibal|None|refused": 7, "nose|admitted": 49, "not_covered:fabric_nomex|None|refused": 6, "not_covered:pmma_sphere|wake|refused": 17, "not_covered:pmma_slab|None|refused": 6, "not_covered:fabric_sibal|opposed|refused": 1, "not_covered:candle|forward stagnation|refused": 7, "not_covered:ultem_sheet|None|refused": 1, "not_covered:wax_slab|None|refused": 3, "flat|admitted": 23, "not_covered:fabric_nomex|concurrent|refused": 3, "not_covered:pmma_rod|None|refused": 2, "not_covered:pmma_rod|concurrent|refused": 6}
- flat_holds_low_flow: 72 answered, 59 refused; reasons: {"feature flow_speed: condition \"flow_speed\" missing from the prefix": 59, "feature nose_radius: condition \"nose_radius\" missing from the prefix": 59, "feature heat_time: condition \"heat_time\" missing from the prefix": 59, "feature is_concurrent: condition \"is_concurrent\" missing from the prefix": 59}; by coverage: {"not_covered:fabric_sibal|None|refused": 7, "nose|admitted": 49, "not_covered:fabric_nomex|None|refused": 6, "not_covered:pmma_sphere|wake|refused": 17, "not_covered:pmma_slab|None|refused": 6, "not_covered:fabric_sibal|opposed|refused": 1, "not_covered:candle|forward stagnation|refused": 7, "not_covered:ultem_sheet|None|refused": 1, "not_covered:wax_slab|None|refused": 3, "flat|admitted": 23, "not_covered:fabric_nomex|concurrent|refused": 3, "not_covered:pmma_rod|None|refused": 2, "not_covered:pmma_rod|concurrent|refused": 6}
- flat_holds_high_flow: 72 answered, 59 refused; reasons: {"feature flow_speed: condition \"flow_speed\" missing from the prefix": 59, "feature nose_radius: condition \"nose_radius\" missing from the prefix": 59, "feature heat_time: condition \"heat_time\" missing from the prefix": 59, "feature is_concurrent: condition \"is_concurrent\" missing from the prefix": 59}; by coverage: {"not_covered:fabric_sibal|None|refused": 7, "nose|admitted": 49, "not_covered:fabric_nomex|None|refused": 6, "not_covered:pmma_sphere|wake|refused": 17, "not_covered:pmma_slab|None|refused": 6, "not_covered:fabric_sibal|opposed|refused": 1, "not_covered:candle|forward stagnation|refused": 7, "not_covered:ultem_sheet|None|refused": 1, "not_covered:wax_slab|None|refused": 3, "flat|admitted": 23, "not_covered:fabric_nomex|concurrent|refused": 3, "not_covered:pmma_rod|None|refused": 2, "not_covered:pmma_rod|concurrent|refused": 6}
- flat_quench_speed: 72 answered, 59 refused; reasons: {"feature flow_speed: condition \"flow_speed\" missing from the prefix": 59, "feature nose_radius: condition \"nose_radius\" missing from the prefix": 59, "feature heat_time: condition \"heat_time\" missing from the prefix": 59, "feature is_concurrent: condition \"is_concurrent\" missing from the prefix": 59}; by coverage: {"not_covered:fabric_sibal|None|refused": 7, "nose|admitted": 49, "not_covered:fabric_nomex|None|refused": 6, "not_covered:pmma_sphere|wake|refused": 17, "not_covered:pmma_slab|None|refused": 6, "not_covered:fabric_sibal|opposed|refused": 1, "not_covered:candle|forward stagnation|refused": 7, "not_covered:ultem_sheet|None|refused": 1, "not_covered:wax_slab|None|refused": 3, "flat|admitted": 23, "not_covered:fabric_nomex|concurrent|refused": 3, "not_covered:pmma_rod|None|refused": 2, "not_covered:pmma_rod|concurrent|refused": 6}

## 3. Where the loss sits, and the sign of the error

### quench_u (interval CRPS, cm/s; error = median forecast minus the nearer end of the outcome interval, 0 inside it)

39 scored train cases.

By sample:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| pmma_slab opposed | 8 | 5.665 | 73 % | -5.651 | 2/8 | 3/8 |
| pmma_rod opposed | 11 | 0.938 | 17 % | -1.174 | 1/11 | 1/11 |
| pmma_rod stagnation | 5 | 0.844 | 7 % | -1.042 | 0/5 | 0/5 |
| pmma_sphere forward stagnation | 11 | 0.111 | 2 % | 0.000 | 10/11 | 11/11 |
| pmma_film opposed | 4 | 0.192 | 1 % | -0.373 | 2/4 | 4/4 |

By diameter, m:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| None | 12 | 3.841 | 75 % | -0.861 | 4/12 | 7/12 |
| 0.00635 | 8 | 0.974 | 13 % | -1.108 | 1/8 | 1/8 |
| 0.009524999999999999 | 8 | 0.842 | 11 % | -0.946 | 0/8 | 0/8 |
| 0.02 | 6 | 0.203 | 2 % | -0.117 | 5/6 | 6/6 |
| 0.01 | 5 | 0.000 | 0 % | 0.000 | 5/5 | 5/5 |

By O2:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 0.19 to 0.21 | 12 | 2.746 | 53 % | -0.797 | 3/12 | 4/12 |
| 0.17 to 0.19 | 16 | 0.829 | 21 % | -0.512 | 8/16 | 10/16 |
| >= 0.21 | 7 | 1.704 | 19 % | 0.000 | 4/7 | 5/7 |
| 0.15 to 0.17 | 3 | 0.868 | 4 % | -1.042 | 0/3 | 0/3 |
| < 0.15 | 1 | 1.090 | 2 % | -1.339 | 0/1 | 0/1 |

By ignition speed, cm/s:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 10 to 20 | 8 | 4.290 | 56 % | -0.597 | 4/8 | 5/8 |
| 3 to 6 | 23 | 1.012 | 38 % | -0.747 | 5/23 | 7/23 |
| >= 20 | 1 | 3.663 | 6 % | -4.252 | 0/1 | 0/1 |
| 6 to 10 | 7 | 0.083 | 1 % | 0.000 | 6/7 | 7/7 |

By investigation:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| PSI-25 | 28 | 2.165 | 98 % | -1.001 | 5/28 | 8/28 |
| PSI-26 | 11 | 0.111 | 2 % | 0.000 | 10/11 | 11/11 |

By day:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 2014-07-11 | 4 | 6.919 | 45 % | -6.077 | 1/4 | 2/4 |
| 2014-02-20 | 3 | 3.784 | 18 % | -4.252 | 1/3 | 1/3 |
| 2014-08-01 | 1 | 6.291 | 10 % | -7.049 | 0/1 | 0/1 |
| 2014-07-08 | 4 | 1.150 | 7 % | -1.194 | 0/4 | 0/4 |
| 2014-06-16 | 3 | 0.967 | 5 % | -1.421 | 1/3 | 1/3 |
| 2014-07-30 | 4 | 0.610 | 4 % | -0.620 | 0/4 | 0/4 |
| 2014-06-26 | 2 | 1.096 | 4 % | -1.108 | 0/2 | 0/2 |
| 2014-06-23 | 1 | 1.464 | 2 % | -1.468 | 0/1 | 0/1 |
| 2014-07-28 | 2 | 0.468 | 2 % | -0.554 | 0/2 | 0/2 |
| 2014-07-02 | 3 | 0.256 | 1 % | -0.747 | 1/3 | 3/3 |
| 2013-04-09 | 2 | 0.290 | 1 % | -0.521 | 1/2 | 2/2 |
| 2013-04-23 | 1 | 0.279 | 0 % | -0.423 | 1/1 | 1/1 |
| 2013-05-17 | 1 | 0.238 | 0 % | -0.234 | 1/1 | 1/1 |
| 2013-04-29 | 1 | 0.122 | 0 % | 0.000 | 1/1 | 1/1 |
| 2012-04-09 | 1 | 0.000 | 0 % | 0.000 | 1/1 | 1/1 |
| 2012-06-14 | 3 | 0.000 | 0 % | 0.000 | 3/3 | 3/3 |
| 2013-04-16 | 2 | 0.000 | 0 % | 0.000 | 2/2 | 2/2 |
| 2014-02-19 | 1 | 0.000 | 0 % | 0.000 | 1/1 | 1/1 |

All cases: median error -0.635; in the 50 % band 15/39; in the 90 % band 19/39.

### sustain (Brier)

60 scored train cases.

By sample:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| pmma_film opposed | 11 | 0.045 | 47 % | 0.951 | 0.909 | 0.042 |
| pmma_slab opposed | 10 | 0.034 | 32 % | 0.930 | 1.000 | -0.070 |
| pmma_rod stagnation | 11 | 0.017 | 17 % | 0.859 | 0.909 | -0.050 |
| pmma_rod concurrent | 6 | 0.006 | 3 % | 0.951 | 1.000 | -0.049 |
| pmma_sphere forward stagnation | 22 | 5.36e-04 | 1 % | 0.984 | 1.000 | -0.016 |

By diameter, m:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| None | 21 | 0.040 | 78 % | 0.941 | 0.952 | -0.011 |
| 0.009524999999999999 | 4 | 0.027 | 10 % | 0.764 | 0.750 | 0.014 |
| 0.00635 | 13 | 0.008 | 10 % | 0.931 | 1.000 | -0.069 |
| 0.02 | 15 | 7.70e-04 | 1 % | 0.979 | 1.000 | -0.021 |
| 0.01 | 7 | 3.59e-05 | 0 % | 0.996 | 1.000 | -0.004 |

By O2:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 0.15 to 0.17 | 13 | 0.046 | 56 % | 0.907 | 0.923 | -0.016 |
| >= 0.21 | 16 | 0.021 | 31 % | 0.955 | 1.000 | -0.045 |
| < 0.15 | 2 | 0.054 | 10 % | 0.535 | 0.500 | 0.035 |
| 0.17 to 0.19 | 21 | 9.25e-04 | 2 % | 0.975 | 1.000 | -0.025 |
| 0.19 to 0.21 | 8 | 1.25e-04 | 0 % | 0.993 | 1.000 | -0.007 |

By ignition speed, cm/s:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 3 to 6 | 27 | 0.026 | 67 % | 0.914 | 0.926 | -0.012 |
| >= 20 | 2 | 0.166 | 31 % | 0.710 | 1.000 | -0.290 |
| 6 to 10 | 14 | 6.15e-04 | 1 % | 0.982 | 1.000 | -0.018 |
| 10 to 20 | 14 | 3.70e-04 | 0 % | 0.986 | 1.000 | -0.014 |
| < 3 | 3 | 7.84e-04 | 0 % | 0.972 | 1.000 | -0.028 |

By investigation:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| PSI-25 | 38 | 0.028 | 99 % | 0.919 | 0.947 | -0.029 |
| PSI-26 | 22 | 5.36e-04 | 1 % | 0.984 | 1.000 | -0.016 |

By day:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 2014-07-02 | 6 | 0.083 | 47 % | 0.924 | 0.833 | 0.090 |
| 2014-02-20 | 4 | 0.083 | 31 % | 0.855 | 1.000 | -0.145 |
| 2014-07-08 | 3 | 0.040 | 11 % | 0.656 | 0.667 | -0.010 |
| 2014-06-26 | 5 | 0.010 | 5 % | 0.923 | 1.000 | -0.077 |
| 2014-07-22 | 4 | 0.005 | 2 % | 0.962 | 1.000 | -0.038 |
| 2014-08-01 | 3 | 0.005 | 1 % | 0.942 | 1.000 | -0.058 |
| 2014-07-28 | 3 | 0.005 | 1 % | 0.954 | 1.000 | -0.046 |
| 2013-04-23 | 3 | 0.003 | 1 % | 0.951 | 1.000 | -0.049 |
| 2014-02-19 | 5 | 4.71e-04 | 0 % | 0.983 | 1.000 | -0.017 |
| 2014-07-11 | 5 | 4.67e-04 | 0 % | 0.984 | 1.000 | -0.016 |
| 2013-04-29 | 2 | 9.88e-04 | 0 % | 0.969 | 1.000 | -0.031 |
| 2013-05-17 | 3 | 3.27e-04 | 0 % | 0.982 | 1.000 | -0.018 |
| 2013-04-09 | 4 | 9.93e-05 | 0 % | 0.990 | 1.000 | -0.010 |
| 2013-04-16 | 3 | 8.07e-05 | 0 % | 0.991 | 1.000 | -0.009 |
| 2012-04-09 | 3 | 3.45e-05 | 0 % | 0.994 | 1.000 | -0.006 |
| 2012-06-14 | 4 | 2.23e-06 | 0 % | 0.999 | 1.000 | -0.001 |

Calibration (forecast probability bin):

| bin | n | mean forecast | observed rate |
|---|---|---|---|
| 0.15 to 0.3 | 1 | 0.265 | 0.000 |
| 0.3 to 0.5 | 1 | 0.424 | 1.000 |
| 0.7 to 0.85 | 3 | 0.772 | 0.667 |
| 0.85 to 0.95 | 8 | 0.906 | 1.000 |
| >= 0.95 | 47 | 0.985 | 1.000 |

## 4. Sensitivity of the train score to each typed number

Each typed number carrying weight moved by +1 % of its value (forward difference, Python copy of law.rs checked against the Rust forward); change of the train score (negative = better). 176 numbers; base: {"quench: interval CRPS cm/s, mean of day means": 1.3296, "sustain: Brier, mean of day means": 0.0147}. Full list: `runs/dev/gen7/sensitivity/bass1_97c0daf9de84482e.json`.

Largest for quench: interval CRPS cm/s, mean of day means:

| number | value | change of the score for +1 % |
|---|---|---|
| `flat_quench_speed.median.b0` | -4.601 | -0.01058 |
| `x.x_o2.centre` | 0.18 | -0.00375 |
| `flat_quench_speed.spread.b0` | 0.5 | -0.00133 |
| `x.nose_radius.centre` | 0.01 | +0.00099 |
| `x.nose_radius.scale` | 1 | -0.00056 |
| `quench_speed.median.W1[3, 2]` | -2 | -0.00041 |
| `flat_quench_speed.tail_width` | 2.5 | -0.00033 |
| `flat_quench_speed.tail_weight` | 0.15 | -0.00028 |
| `quench_speed.spread.b0` | 0.75 | +0.00026 |
| `quench_speed.median.w2[3]` | -2.15 | -0.00026 |
| `x.heat_time.centre` | 200 | -0.00026 |
| `quench_speed.median.W1[2, 2]` | -2 | -0.00021 |

Largest for sustain: Brier, mean of day means:

| number | value | change of the score for +1 % |
|---|---|---|
| `x.x_o2.centre` | 0.18 | +0.00136 |
| `x.flow_speed.scale` | 1 | -0.00121 |
| `flat_holds_high_flow.logit.W1[4, 1]` | 1 | +0.00041 |
| `flat_holds_high_flow.logit.b1[4]` | -3.689 | +0.00038 |
| `flat_holds_high_flow.logit.W1[3, 1]` | 1 | +0.00038 |
| `x.flow_speed.centre` | 0.01 | -0.00031 |
| `flat_holds_high_flow.logit.skip[0]` | 20 | -0.00029 |
| `flat_holds_high_flow.logit.b1[3]` | -2.996 | +0.00028 |
| `x.x_o2.scale` | 1 | +0.00027 |
| `flat_holds_high_flow.logit.W1[2, 1]` | 1 | +0.00024 |
| `flat_holds_high_flow.logit.W1[1, 1]` | 1 | +0.00018 |
| `holds_high_flow.logit.b0` | 8.26 | -0.00015 |


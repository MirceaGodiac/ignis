# spice: train anatomy of the W0 law (lit-fable, `sha256:40c345b0453da40b38e85bd7c7441e9dc1e272ce097cf204f7bb3f4c3e35d3bf`)

TRAIN days only, written by `tools/wsp/ext/law_anatomy.py` after the registered look of PROTOCOL 13.2. For the revision the physicist will type; no number is changed here. Per-case file: `runs/dev/data_all/work/transfer_spice/law_study_40c345b0453da40b.jsonl` (baselines beside it as `law_study_baseline_*.jsonl`).

## 1. Score against the baselines (train, on the cases the release answers)

### target T1

- Asked of the release: 42; answered: 42; refused: 0; refusal reasons: {}
| source | quantile score x2, mm, mean of day means | pooled mean | on |
|---|---|---|---|
| **release** | 3.1628 | 3.4494 | cases the release answers |
| B0 pooled climatology (simple) | 11.3160 | 10.5812 | cases the release answers |
| B0 pooled climatology (simple) | 11.3160 | 10.5812 | all lane cases |
| B1 per-fuel climatology | 7.0783 | 7.9425 | cases the release answers |
| B1 per-fuel climatology | 7.0783 | 7.9425 | all lane cases |
**Reading (quantile score x2, mm, mean of day means):** release 3.1628; simple baseline (B0 pooled climatology (simple)) 11.3160; best baseline (B1 per-fuel climatology) 7.0783: **beats the best baseline**.

### target T2

- Asked of the release: 42; answered: 42; refused: 0; refusal reasons: {}
| source | quantile score x2, mm, mean of day means | pooled mean | on |
|---|---|---|---|
| **release** | 3.2254 | 3.1144 | cases the release answers |
| B0 pooled climatology (simple) | 11.3160 | 10.5812 | cases the release answers |
| B0 pooled climatology (simple) | 11.3160 | 10.5812 | all lane cases |
| B1 per-fuel climatology | 7.0783 | 7.9425 | cases the release answers |
| B1 per-fuel climatology | 7.0783 | 7.9425 | all lane cases |
| Lin and Faeth coflow law, train sigma (T2 only) | 2.0076 | 2.2575 | cases the release answers |
| Lin and Faeth coflow law, train sigma (T2 only) | 2.0076 | 2.2575 | all lane cases |
**Reading (quantile score x2, mm, mean of day means):** release 3.2254; simple baseline (B0 pooled climatology (simple)) 11.3160; best baseline (Lin and Faeth coflow law, train sigma (T2 only)) 2.0076: **beats the simple baseline**.

## 2. Refusals on train (admission, by reason)

- smoke_point_length: 42 answered, 0 refused; reasons: {}
- smoke_point_fuel_flow: 42 answered, 0 refused; reasons: {}
- flame_length: 42 answered, 0 refused; reasons: {}
- emits_soot: 42 answered, 0 refused; reasons: {}

## 3. Where the loss sits, and the sign of the error

### T1 (quantile score x2, mm; error = ln median forecast - ln outcome)

42 scored train cases.

By fuel:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| Propylene | 22 | 3.461 | 53 % | -0.095 | 6/22 | 21/22 |
| Ethylene | 6 | 4.737 | 20 % | 0.077 | 4/6 | 6/6 |
| Propane | 3 | 5.714 | 12 % | 0.107 | 2/3 | 3/3 |
| 75 % Propylene | 7 | 2.148 | 10 % | -0.041 | 5/7 | 7/7 |
| 50 % Propylene | 4 | 2.035 | 6 % | -0.015 | 4/4 | 4/4 |

By nozzle, mm:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 0.762 | 13 | 4.623 | 41 % | 0.211 | 3/13 | 13/13 |
| 1.6002 | 25 | 2.305 | 40 % | -0.064 | 16/25 | 25/25 |
| 0.4064 | 4 | 6.787 | 19 % | -0.215 | 2/4 | 3/4 |

By coflow, cm/s:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 20 to 40 | 18 | 2.949 | 37 % | 0.054 | 13/18 | 18/18 |
| 10 to 20 | 10 | 4.672 | 32 % | -0.011 | 4/10 | 9/10 |
| >= 40 | 11 | 3.211 | 24 % | -0.090 | 4/11 | 11/11 |
| < 10 | 3 | 3.248 | 7 % | -0.315 | 0/3 | 3/3 |

By day:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 174 | 11 | 3.699 | 28 % | -0.088 | 3/11 | 10/11 |
| 058 | 11 | 3.222 | 24 % | -0.102 | 3/11 | 11/11 |
| 052 | 8 | 3.797 | 21 % | -0.022 | 5/8 | 8/8 |
| 045 | 5 | 4.983 | 17 % | 0.092 | 3/5 | 5/5 |
| 167 | 4 | 2.035 | 6 % | -0.015 | 4/4 | 4/4 |
| 168 | 1 | 3.504 | 2 % | 0.063 | 1/1 | 1/1 |
| 119 | 2 | 0.899 | 1 % | -0.013 | 2/2 | 2/2 |

All cases: median error -0.015; in the 50 % band 21/42; in the 90 % band 41/42.

### T2 (quantile score x2, mm; error = ln median forecast - ln outcome)

42 scored train cases.

By fuel:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| Propylene | 22 | 2.819 | 47 % | -0.264 | 5/22 | 20/22 |
| Ethylene | 6 | 4.518 | 21 % | -0.080 | 6/6 | 6/6 |
| 75 % Propylene | 7 | 2.610 | 14 % | -0.240 | 2/7 | 7/7 |
| Propane | 3 | 4.448 | 10 % | -0.034 | 3/3 | 3/3 |
| 50 % Propylene | 4 | 2.516 | 8 % | -0.124 | 4/4 | 4/4 |

By nozzle, mm:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 1.6002 | 25 | 2.742 | 52 % | -0.203 | 11/25 | 23/25 |
| 0.762 | 13 | 3.789 | 38 % | -0.224 | 6/13 | 13/13 |
| 0.4064 | 4 | 3.248 | 10 % | -0.116 | 3/4 | 4/4 |

By coflow, cm/s:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 20 to 40 | 18 | 3.216 | 44 % | -0.221 | 7/18 | 18/18 |
| 10 to 20 | 10 | 3.480 | 27 % | -0.141 | 6/10 | 9/10 |
| >= 40 | 11 | 2.840 | 24 % | -0.160 | 6/11 | 11/11 |
| < 10 | 3 | 2.292 | 5 % | -0.224 | 1/3 | 2/3 |

By day:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 058 | 11 | 2.998 | 25 % | -0.286 | 2/11 | 10/11 |
| 174 | 11 | 2.640 | 22 % | -0.246 | 3/11 | 10/11 |
| 052 | 8 | 3.309 | 20 % | -0.114 | 5/8 | 8/8 |
| 045 | 5 | 4.641 | 18 % | -0.065 | 5/5 | 5/5 |
| 167 | 4 | 2.516 | 8 % | -0.124 | 4/4 | 4/4 |
| 119 | 2 | 2.568 | 4 % | -0.321 | 0/2 | 2/2 |
| 168 | 1 | 3.905 | 3 % | -0.129 | 1/1 | 1/1 |

All cases: median error -0.164; in the 50 % band 20/42; in the 90 % band 40/42.

## 4. Sensitivity of the train score to each typed number

Each typed number carrying weight moved by +1 % of its value (forward difference, Python copy of law.rs checked against the Rust forward); change of the train score (negative = better). 150 numbers; base: {"T1: quantile score x2, mm, mean of day means": 3.1628, "T2: quantile score x2, mm, mean of day means": 3.2254}. Full list: `runs/dev/gen7/sensitivity/spice_40c345b0453da40b.json`.

Largest for T1: quantile score x2, mm, mean of day means:

| number | value | change of the score for +1 % |
|---|---|---|
| `smoke_point_length.median.b0` | -3.863 | +0.17195 |
| `smoke_point_length.median.b1[0]` | 1.342 | +0.03178 |
| `smoke_point_length.median.w2[0]` | 1 | +0.02187 |
| `x.nozzle_diameter.centre` | 0.001 | +0.02017 |
| `smoke_point_length.spread.b0` | 0.3 | +0.01621 |
| `x.x_fuel.centre` | 1 | +0.01505 |
| `smoke_point_length.spread.b1[0]` | 1.342 | -0.01449 |
| `smoke_point_length.spread.w2[0]` | -0.2 | +0.01034 |
| `smoke_point_length.median.W1[0, 0]` | -0.91 | +0.00914 |
| `x.coflow_speed.centre` | 0.25 | -0.00876 |
| `x.nozzle_diameter.scale` | 1 | +0.00863 |
| `smoke_point_length.spread.b1[1]` | 0.842 | +0.00828 |

Largest for T2: quantile score x2, mm, mean of day means:

| number | value | change of the score for +1 % |
|---|---|---|
| `flame_length.median.b0` | -3.895 | -0.19923 |
| `x.fuel_mass_flow.centre` | 1e-06 | +0.08034 |
| `x.x_fuel.centre` | 1 | +0.05268 |
| `x.fuel_mass_flow.scale` | 1 | +0.01295 |
| `flame_length.spread.b0` | 0.22 | +0.01290 |
| `flame_length.median.b1[2]` | 1.374 | +0.00993 |
| `x.x_fuel.scale` | 1 | -0.00767 |
| `x.coflow_speed.centre` | 0.25 | -0.00763 |
| `flame_length.median.W1[2, 5]` | -1.254 | +0.00733 |
| `flame_length.median.skip[3]` | 0.66 | +0.00728 |
| `x.is_propylene.scale` | 1 | +0.00595 |
| `flame_length.median.b1[0]` | 0.784 | -0.00389 |


# spice: train anatomy of the W0 law (w0-fable, `sha256:93e750242725a5a1fb9d38f4178b23605cac636f210bf9f3f8bc8e8150e2f2a8`)

TRAIN days only, written by `tools/wsp/ext/law_anatomy.py` after the registered look of PROTOCOL 13.2. For the revision the physicist will type; no number is changed here. Per-case file: `runs/dev/data_all/work/transfer_spice/law_study_93e750242725a5a1.jsonl` (baselines beside it as `law_study_baseline_*.jsonl`).

## 1. Score against the baselines (train, on the cases the release answers)

### target T1

- Asked of the release: 42; answered: 42; refused: 0; refusal reasons: {}
| source | quantile score x2, mm, mean of day means | pooled mean | on |
|---|---|---|---|
| **release** | 6.6998 | 7.0751 | cases the release answers |
| B0 pooled climatology (simple) | 11.3160 | 10.5812 | cases the release answers |
| B0 pooled climatology (simple) | 11.3160 | 10.5812 | all lane cases |
| B1 per-fuel climatology | 7.0783 | 7.9425 | cases the release answers |
| B1 per-fuel climatology | 7.0783 | 7.9425 | all lane cases |
**Reading (quantile score x2, mm, mean of day means):** release 6.6998; simple baseline (B0 pooled climatology (simple)) 11.3160; best baseline (B1 per-fuel climatology) 7.0783: **beats the best baseline**.

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
| Propylene | 22 | 5.847 | 43 % | 0.159 | 14/22 | 22/22 |
| Propane | 3 | 23.855 | 24 % | 0.443 | 1/3 | 3/3 |
| Ethylene | 6 | 8.429 | 17 % | 0.082 | 6/6 | 6/6 |
| 75 % Propylene | 7 | 4.123 | 10 % | 0.184 | 6/7 | 7/7 |
| 50 % Propylene | 4 | 4.379 | 6 % | -0.093 | 4/4 | 4/4 |

By nozzle, mm:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 0.762 | 13 | 12.835 | 56 % | 0.467 | 2/13 | 13/13 |
| 1.6002 | 25 | 4.036 | 34 % | 0.079 | 25/25 | 25/25 |
| 0.4064 | 4 | 7.350 | 10 % | 0.039 | 4/4 | 4/4 |

By coflow, cm/s:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 20 to 40 | 18 | 8.277 | 50 % | 0.179 | 12/18 | 18/18 |
| 10 to 20 | 10 | 7.600 | 26 % | 0.179 | 8/10 | 10/10 |
| >= 40 | 11 | 5.582 | 21 % | 0.010 | 9/11 | 11/11 |
| < 10 | 3 | 3.588 | 4 % | -0.061 | 2/3 | 3/3 |

By day:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 052 | 8 | 11.790 | 32 % | 0.259 | 5/8 | 8/8 |
| 058 | 11 | 6.232 | 23 % | 0.152 | 7/11 | 11/11 |
| 174 | 11 | 5.462 | 20 % | 0.166 | 7/11 | 11/11 |
| 045 | 5 | 8.648 | 15 % | 0.097 | 5/5 | 5/5 |
| 167 | 4 | 4.379 | 6 % | -0.093 | 4/4 | 4/4 |
| 168 | 1 | 7.334 | 2 % | 0.068 | 1/1 | 1/1 |
| 119 | 2 | 3.054 | 2 % | 0.213 | 2/2 | 2/2 |

All cases: median error 0.145; in the 50 % band 31/42; in the 90 % band 42/42.

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

Each typed number carrying weight moved by +1 % of its value (forward difference, Python copy of law.rs checked against the Rust forward); change of the train score (negative = better). 147 numbers; base: {"T1: quantile score x2, mm, mean of day means": 6.6998, "T2: quantile score x2, mm, mean of day means": 3.2254}. Full list: `runs/dev/gen7/sensitivity/spice_93e750242725a5a1.json`.

Largest for T1: quantile score x2, mm, mean of day means:

| number | value | change of the score for +1 % |
|---|---|---|
| `smoke_point_length.median.b0` | -3.863 | +0.49781 |
| `smoke_point_length.median.w2[0]` | 1 | +0.17314 |
| `smoke_point_length.median.b1[0]` | 1.347 | +0.16239 |
| `x.nozzle_diameter.centre` | 0.001 | +0.10676 |
| `x.x_fuel.centre` | 1 | +0.08439 |
| `smoke_point_length.median.skip[5]` | -1.304 | +0.06367 |
| `x.is_propylene.scale` | 1 | +0.04851 |
| `x.coflow_speed.centre` | 0.25 | -0.04775 |
| `smoke_point_length.spread.b0` | 0.45 | +0.04504 |
| `smoke_point_length.median.W1[0, 5]` | 0.299 | +0.01453 |
| `smoke_point_length.median.skip[4]` | 0.287 | +0.00906 |
| `x.is_propane.scale` | 1 | -0.00608 |

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


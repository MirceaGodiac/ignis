# samer: train anatomy of the W0 law (w0-fable, `sha256:05f77ae2f7a666e0da69d04fa29d9ad8b0ac15790645abe1adc0149292327fb0`)

TRAIN days only, written by `tools/wsp/ext/law_anatomy.py` after the registered look of PROTOCOL 13.2. For the revision the physicist will type; no number is changed here. Per-case file: `runs/dev/data_all/work/transfer_samer/law_study_05f77ae2f7a666e0.jsonl` (baselines beside it as `law_study_baseline_*.jsonl`).

## 1. Score against the baselines (train, on the cases the release answers)

### target d10

- Asked of the release: 50; answered: 48; refused: 2; refusal reasons: {"feature net_aging_time": 2, "feature sample_temperature": 1}
| source | CRPS of ln d10, case mean | day-equal | on |
|---|---|---|---|
| **release** | 0.2722 | 0.2292 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.3229 | 0.2807 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.3229 | 0.2807 | all lane cases |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.2454 | 0.2369 | cases the release answers |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.2454 | 0.2369 | all lane cases |
**Reading (CRPS of ln d10, case mean):** release 0.2722; simple baseline (B0 / I0 constant (simple, SAME-R train)) 0.3229; best baseline (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) 0.2454: **beats the simple baseline**.

### target iss

- Asked of the release: 50; answered: 48; refused: 2; refusal reasons: {"feature net_aging_time": 2, "feature sample_temperature": 1}
| source | Brier, case mean | day-equal | on |
|---|---|---|---|
| **release** | 0.1501 | 0.1490 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.2066 | 0.2032 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.2066 | 0.2032 | all lane cases |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.0312 | 0.0336 | cases the release answers |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.0312 | 0.0336 | all lane cases |
**Reading (Brier, case mean):** release 0.1501; simple baseline (B0 / I0 constant (simple, SAME-R train)) 0.2066; best baseline (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) 0.0312: **beats the simple baseline**.

## 2. Refusals on train (admission, by reason)

- d10: 48 answered, 2 refused; reasons: {"feature net_aging_time: condition \"net_aging_time\" missing from the prefix": 2, "feature sample_temperature: condition \"sample_temperature\" missing from the prefix": 1}
- d30: 48 answered, 2 refused; reasons: {"feature net_aging_time: condition \"net_aging_time\" missing from the prefix": 2, "feature sample_temperature: condition \"sample_temperature\" missing from the prefix": 1}
- iss_full_scale: 48 answered, 2 refused; reasons: {"feature net_aging_time: condition \"net_aging_time\" missing from the prefix": 2, "feature sample_temperature: condition \"sample_temperature\" missing from the prefix": 1}

## 3. Where the loss sits, and the sign of the error

### d10 (CRPS of ln d10; error = ln median forecast - ln outcome)

47 scored train cases.

By material:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| silicone | 17 | 0.310 | 41 % | -0.355 | 7/17 | 15/17 |
| pyrell | 8 | 0.330 | 21 % | -0.537 | 2/8 | 7/8 |
| teflon | 8 | 0.320 | 20 % | 0.015 | 2/8 | 7/8 |
| kapton | 7 | 0.191 | 10 % | -0.105 | 3/7 | 7/7 |
| lampwick | 7 | 0.141 | 8 % | -0.007 | 5/7 | 7/7 |

By measurement:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| primary | 31 | 0.230 | 56 % | -0.227 | 16/31 | 29/31 |
| aged | 16 | 0.354 | 44 % | -0.475 | 3/16 | 14/16 |

By inlet velocity, m/s:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 0.081 | 30 | 0.265 | 62 % | -0.284 | 12/30 | 28/30 |
| 0.0 | 6 | 0.441 | 21 % | -0.256 | 2/6 | 4/6 |
| 0.01 | 6 | 0.166 | 8 % | -0.152 | 3/6 | 6/6 |
| 0.02 | 4 | 0.232 | 7 % | -0.370 | 2/4 | 4/4 |
| 0.08 | 1 | 0.273 | 2 % | -0.457 | 0/1 | 1/1 |

By sample temperature, K:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 600 to 700 | 18 | 0.303 | 43 % | -0.354 | 8/18 | 16/18 |
| < 600 | 16 | 0.244 | 30 % | -0.267 | 7/16 | 15/16 |
| >= 800 | 10 | 0.264 | 21 % | -0.145 | 3/10 | 9/10 |
| 700 to 800 | 3 | 0.269 | 6 % | 0.410 | 1/3 | 3/3 |

By day:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| PSI-101:day:2010-09-09 | 16 | 0.286 | 36 % | -0.169 | 7/16 | 14/16 |
| PSI-101:day:2010-08-23 | 8 | 0.343 | 21 % | -0.559 | 2/8 | 7/8 |
| PSI-101:day:2010-07-30 | 6 | 0.282 | 13 % | -0.171 | 2/6 | 6/6 |
| PSI-101:day:2010-08-24 | 5 | 0.325 | 13 % | -0.539 | 2/5 | 4/5 |
| PSI-101:day:2010-08-02 | 4 | 0.223 | 7 % | -0.390 | 1/4 | 4/4 |
| PSI-101:day:2010-07-29 | 4 | 0.144 | 4 % | 0.164 | 3/4 | 4/4 |
| PSI-101:day:2010-07-16 | 1 | 0.273 | 2 % | -0.457 | 0/1 | 1/1 |
| PSI-101:day:2010-07-20 | 1 | 0.164 | 1 % | -0.300 | 0/1 | 1/1 |
| PSI-101:day:2010-07-22 | 1 | 0.151 | 1 % | -0.267 | 1/1 | 1/1 |
| PSI-101:day:2010-08-25 | 1 | 0.100 | 1 % | -0.066 | 1/1 | 1/1 |

All cases: median error -0.267; in the 50 % band 19/47; in the 90 % band 43/47.

### ISS detector at full scale (Brier)

48 scored train cases.

By material:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| silicone | 17 | 0.136 | 32 % | 0.642 | 1.000 | -0.358 |
| kapton | 8 | 0.265 | 29 % | 0.275 | 0.250 | 0.025 |
| pyrell | 8 | 0.189 | 21 % | 0.575 | 1.000 | -0.425 |
| teflon | 8 | 0.105 | 12 % | 0.260 | 0.000 | 0.260 |
| lampwick | 7 | 0.061 | 6 % | 0.762 | 1.000 | -0.238 |

By measurement:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| primary | 32 | 0.135 | 60 % | 0.514 | 0.688 | -0.174 |
| aged | 16 | 0.179 | 40 % | 0.543 | 0.750 | -0.207 |

By inlet velocity, m/s:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 0.081 | 30 | 0.151 | 63 % | 0.452 | 0.733 | -0.281 |
| 0.0 | 6 | 0.176 | 15 % | 0.649 | 0.667 | -0.017 |
| 0.01 | 6 | 0.164 | 14 % | 0.699 | 0.667 | 0.032 |
| 0.02 | 5 | 0.076 | 5 % | 0.595 | 0.600 | -0.005 |
| 0.08 | 1 | 0.251 | 3 % | 0.499 | 1.000 | -0.501 |

By sample temperature, K:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| >= 800 | 10 | 0.284 | 39 % | 0.359 | 0.200 | 0.159 |
| < 600 | 16 | 0.137 | 30 % | 0.652 | 1.000 | -0.348 |
| 600 to 700 | 18 | 0.115 | 29 % | 0.582 | 0.889 | -0.307 |
| 700 to 800 | 4 | 0.029 | 2 % | 0.153 | 0.000 | 0.153 |

By day:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| PSI-101:day:2010-09-09 | 17 | 0.143 | 34 % | 0.651 | 0.647 | 0.004 |
| PSI-101:day:2010-08-23 | 8 | 0.197 | 22 % | 0.563 | 1.000 | -0.437 |
| PSI-101:day:2010-08-02 | 4 | 0.316 | 18 % | 0.122 | 0.500 | -0.378 |
| PSI-101:day:2010-07-30 | 6 | 0.104 | 9 % | 0.450 | 0.667 | -0.217 |
| PSI-101:day:2010-08-24 | 5 | 0.093 | 6 % | 0.469 | 0.600 | -0.131 |
| PSI-101:day:2010-07-29 | 4 | 0.073 | 4 % | 0.545 | 0.750 | -0.205 |
| PSI-101:day:2010-07-16 | 1 | 0.251 | 3 % | 0.499 | 1.000 | -0.501 |
| PSI-101:day:2010-08-25 | 1 | 0.182 | 3 % | 0.573 | 1.000 | -0.427 |
| PSI-101:day:2010-07-22 | 1 | 0.118 | 2 % | 0.656 | 1.000 | -0.344 |
| PSI-101:day:2010-07-20 | 1 | 0.013 | 0 % | 0.114 | 0.000 | 0.114 |

Calibration (forecast probability bin):

| bin | n | mean forecast | observed rate |
|---|---|---|---|
| < 0.05 | 2 | 0.038 | 0.000 |
| 0.05 to 0.15 | 5 | 0.118 | 0.000 |
| 0.15 to 0.3 | 4 | 0.232 | 0.500 |
| 0.3 to 0.5 | 4 | 0.453 | 0.750 |
| 0.5 to 0.7 | 26 | 0.619 | 0.846 |
| 0.7 to 0.85 | 6 | 0.795 | 1.000 |
| 0.85 to 0.95 | 1 | 0.865 | 1.000 |

## 4. Sensitivity of the train score to each typed number

Each typed number carrying weight moved by +1 % of its value (forward difference, Python copy of law.rs checked against the Rust forward); change of the train score (negative = better). 163 numbers; base: {"d10: CRPS of ln d10, case mean": 0.2722, "iss: Brier, case mean": 0.1501}. Full list: `runs/dev/gen7/sensitivity/samer_05f77ae2f7a666e0.json`.

Largest for d10: CRPS of ln d10, case mean:

| number | value | change of the score for +1 % |
|---|---|---|
| `d10.median.b0` | -15.162 | -0.03711 |
| `x.sample_temperature.centre` | 673 | +0.01088 |
| `x.sample_temperature.scale` | 50 | -0.00104 |
| `d10.median.skip[4]` | 0.2 | +0.00088 |
| `x.inlet_velocity.centre` | 0.081 | -0.00075 |
| `x.is_silicone.scale` | 1 | -0.00065 |
| `x.net_aging_time.scale` | 1 | +0.00052 |
| `x.is_kapton.scale` | 1 | -0.00045 |
| `d10.median.b1[1]` | -6.1 | +0.00042 |
| `d10.median.skip[2]` | -0.225 | -0.00042 |
| `d10.median.skip[0]` | -1.666 | -0.00041 |
| `d10.median.W1[1, 5]` | -1 | -0.00036 |

Largest for iss: Brier, case mean:

| number | value | change of the score for +1 % |
|---|---|---|
| `x.sample_temperature.centre` | 673 | +0.00695 |
| `iss_full_scale.logit.b0` | 2.456 | -0.00198 |
| `iss_full_scale.logit.skip[4]` | 0.6 | +0.00151 |
| `x.sample_temperature.scale` | 50 | -0.00148 |
| `iss_full_scale.logit.skip[1]` | -6.06 | +0.00109 |
| `iss_full_scale.logit.skip[2]` | -1.57 | -0.00091 |
| `x.is_silicone.scale` | 1 | -0.00090 |
| `x.is_teflon.scale` | 1 | +0.00067 |
| `iss_full_scale.logit.b1[0]` | -2 | +0.00042 |
| `x.heating_duration.centre` | 60 | +0.00041 |
| `x.inlet_velocity.centre` | 0.081 | -0.00038 |
| `x.inlet_velocity.scale` | 0.01 | -0.00035 |


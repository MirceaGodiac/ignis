# cld: train anatomy of the W0 law (w0-fable, `sha256:0995df666bdabfa546607fcba565da166a28839ed984f2f1226bc62274eefe4e`)

TRAIN days only, written by `tools/wsp/ext/law_anatomy.py` after the registered look of PROTOCOL 13.2. For the revision the physicist will type; no number is changed here. Per-case file: `runs/dev/data_all/work/transfer_cld/law_study_0995df666bdabfa5.jsonl` (baselines beside it as `law_study_baseline_*.jsonl`).

## 1. Score against the baselines (train, on the cases the release answers)

### target loss

- Asked of the release: 218; answered: 198; refused: 20; refusal reasons: {"feature x_fuel_stage": 19, "feature v_fuel_stage": 19, "feature x_fuel_final": 10, "feature v_coflow_final": 10, "feature window": 11}
| source | integrated Brier | Brier of lost before fuel-off | integrated Brier, mean of day means | on |
|---|---|---|---|---|
| **release** | 0.1569 | 0.0660 | 0.1302 | cases the release answers |
| incidence (simple) | 0.1493 | 0.0855 | 0.1284 | cases the release answers |
| incidence (simple) | 0.1506 | 0.0926 | 0.1290 | all lane cases |
| by fuel | 0.1405 | 0.0811 | 0.1233 | cases the release answers |
| by fuel | 0.1417 | 0.0875 | 0.1239 | all lane cases |
| loss by dilution ramp | 0.0705 | 0.0587 | 0.0893 | cases the release answers |
| loss by dilution ramp | 0.0718 | 0.0602 | 0.0907 | all lane cases |
| loss, classical LFL at the stoichiometric surface | 0.1892 | 0.0859 | 0.1536 | cases the release answers |
| loss, classical LFL at the stoichiometric surface | 0.1909 | 0.0966 | 0.1545 | all lane cases |
**Reading (integrated Brier):** release 0.1569; simple baseline (incidence (simple)) 0.1493; best baseline (loss by dilution ramp) 0.0705: **loses (higher than the simple baseline)**.

### target radiometer

- Asked of the release: 218; answered: 198; refused: 20; refusal reasons: {"feature x_fuel_stage": 19, "feature v_fuel_stage": 19, "feature x_fuel_final": 10, "feature v_coflow_final": 10, "feature window": 11}
| source | CRPS of log10 V | mean of day means | on |
|---|---|---|---|
| **release** | 0.1500 | 0.1491 | cases the release answers |
| incidence (simple) | 0.2328 | 0.2452 | cases the release answers |
| incidence (simple) | 0.2328 | 0.2452 | all lane cases |
| by fuel | 0.2096 | 0.1959 | cases the release answers |
| by fuel | 0.2096 | 0.1959 | all lane cases |
| radiometer, Roper length plus fuel offset | 0.1166 | 0.1323 | cases the release answers |
| radiometer, Roper length plus fuel offset | 0.1166 | 0.1323 | all lane cases |
**Reading (CRPS of log10 V):** release 0.1500; simple baseline (incidence (simple)) 0.2328; best baseline (radiometer, Roper length plus fuel offset) 0.1166: **beats the simple baseline**.

## 2. Refusals on train (admission, by reason)

- loss: 198 answered, 20 refused; reasons: {"feature x_fuel_stage: condition \"x_fuel_stage\" missing from the prefix": 19, "feature v_fuel_stage: condition \"v_fuel_stage\" missing from the prefix": 19, "feature window: condition \"window\" missing from the prefix": 11, "feature x_fuel_final: condition \"x_fuel_final\" missing from the prefix": 10, "feature v_coflow_final: condition \"v_coflow_final\" missing from the prefix": 10}
- radiometer: 198 answered, 20 refused; reasons: {"feature x_fuel_stage: condition \"x_fuel_stage\" missing from the prefix": 19, "feature v_fuel_stage: condition \"v_fuel_stage\" missing from the prefix": 19, "feature window: condition \"window\" missing from the prefix": 11, "feature x_fuel_final: condition \"x_fuel_final\" missing from the prefix": 10, "feature v_coflow_final: condition \"v_coflow_final\" missing from the prefix": 10}

## 3. Where the loss sits, and the sign of the error

### loss (per case: mean Brier over the horizons at risk; calibration of lost before fuel-off)

198 scored train cases.

By fuel:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| methane | 87 | 0.056 | 71 % | 0.043 | 0.149 | -0.106 |
| ethylene | 111 | 0.018 | 29 % | 0.032 | 0.036 | -0.004 |

By dilution during the test:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| no dilution | 191 | 0.028 | 77 % | 0.029 | 0.058 | -0.029 |
| dilutes | 7 | 0.229 | 23 % | 0.262 | 0.857 | -0.596 |

By lowest fuel fraction of the schedule:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 0.4 to 0.7 | 93 | 0.037 | 49 % | 0.029 | 0.065 | -0.036 |
| 0.25 to 0.4 | 15 | 0.219 | 48 % | 0.092 | 0.600 | -0.508 |
| < 0.15 | 2 | 0.106 | 3 % | 0.475 | 1.000 | -0.525 |
| 0.15 to 0.25 | 35 | 2.39e-04 | 0 % | 0.039 | 0.000 | 0.039 |
| >= 0.7 | 53 | 1.03e-04 | 0 % | 0.018 | 0.000 | 0.018 |

By highest coflow of the schedule, m/s:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 0.3 to 0.45 | 77 | 0.056 | 62 % | 0.047 | 0.130 | -0.083 |
| >= 0.45 | 44 | 0.055 | 35 % | 0.049 | 0.136 | -0.088 |
| 0.15 to 0.3 | 67 | 0.003 | 3 % | 0.020 | 0.015 | 0.006 |
| < 0.15 | 10 | 9.24e-05 | 0 % | 0.016 | 0.000 | 0.016 |

By day:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 18145 | 4 | 0.451 | 26 % | 0.015 | 0.500 | -0.485 |
| 18026 | 12 | 0.131 | 23 % | 0.045 | 0.333 | -0.289 |
| 18022 | 4 | 0.293 | 17 % | 0.035 | 0.500 | -0.465 |
| 18011 | 13 | 0.064 | 12 % | 0.077 | 0.154 | -0.077 |
| 18005 | 9 | 0.071 | 9 % | 0.021 | 0.111 | -0.091 |
| 17324 | 4 | 0.140 | 8 % | 0.133 | 0.750 | -0.617 |
| 18241 | 6 | 0.036 | 3 % | 0.185 | 0.333 | -0.148 |
| 17342 | 2 | 0.044 | 1 % | 0.069 | 0.500 | -0.431 |
| 21046 | 16 | 2.11e-04 | 0 % | 0.039 | 0.000 | 0.039 |
| 18257 | 15 | 2.07e-04 | 0 % | 0.038 | 0.000 | 0.038 |
| 21062 | 10 | 2.61e-04 | 0 % | 0.042 | 0.000 | 0.042 |
| 18218 | 17 | 1.26e-04 | 0 % | 0.019 | 0.000 | 0.019 |
| 18043 | 10 | 2.08e-04 | 0 % | 0.030 | 0.000 | 0.030 |
| 17319 | 9 | 1.66e-04 | 0 % | 0.030 | 0.000 | 0.030 |
| 18239 | 14 | 1.06e-04 | 0 % | 0.018 | 0.000 | 0.018 |
| 18235 | 16 | 8.96e-05 | 0 % | 0.015 | 0.000 | 0.015 |
| 18053 | 14 | 8.84e-05 | 0 % | 0.017 | 0.000 | 0.017 |
| 18214 | 13 | 8.33e-05 | 0 % | 0.015 | 0.000 | 0.015 |
| 18192 | 10 | 7.18e-05 | 0 % | 0.015 | 0.000 | 0.015 |

Calibration (forecast probability bin):

| bin | n | mean forecast | observed rate |
|---|---|---|---|
| < 0.05 | 164 | 0.021 | 0.030 |
| 0.05 to 0.15 | 28 | 0.072 | 0.214 |
| 0.15 to 0.3 | 5 | 0.225 | 1.000 |
| 0.5 to 0.7 | 1 | 0.674 | 1.000 |

### radiometer (CRPS of log10 V; error = median forecast - outcome, in log10 V)

155 scored train cases.

By fuel:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| ethylene | 103 | 0.144 | 64 % | 0.203 | 67/103 | 99/103 |
| methane | 52 | 0.161 | 36 % | 0.285 | 21/52 | 52/52 |

By day:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 18214 | 13 | 0.245 | 14 % | 0.294 | 2/13 | 10/13 |
| 18053 | 14 | 0.210 | 13 % | 0.434 | 0/14 | 14/14 |
| 18218 | 17 | 0.169 | 12 % | 0.282 | 3/17 | 16/17 |
| 18239 | 14 | 0.146 | 9 % | 0.171 | 10/14 | 14/14 |
| 18043 | 9 | 0.224 | 9 % | 0.442 | 0/9 | 9/9 |
| 18235 | 16 | 0.122 | 8 % | 0.160 | 15/16 | 16/16 |
| 21046 | 16 | 0.105 | 7 % | 0.140 | 15/16 | 16/16 |
| 18257 | 15 | 0.103 | 7 % | -0.083 | 14/15 | 15/15 |
| 18192 | 10 | 0.143 | 6 % | 0.217 | 6/10 | 10/10 |
| 18011 | 8 | 0.128 | 4 % | 0.110 | 5/8 | 8/8 |
| 21062 | 10 | 0.102 | 4 % | 0.132 | 10/10 | 10/10 |
| 18026 | 8 | 0.112 | 4 % | 0.149 | 6/8 | 8/8 |
| 17324 | 1 | 0.192 | 1 % | 0.360 | 0/1 | 1/1 |
| 18145 | 2 | 0.086 | 1 % | 0.027 | 2/2 | 2/2 |
| 17342 | 1 | 0.172 | 1 % | 0.326 | 0/1 | 1/1 |
| 18022 | 1 | 0.129 | 1 % | 0.254 | 0/1 | 1/1 |

By fuel fraction of the stage:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 0.4 to 0.7 | 68 | 0.132 | 39 % | 0.184 | 48/68 | 68/68 |
| >= 0.99 | 28 | 0.234 | 28 % | 0.441 | 4/28 | 25/28 |
| 0.7 to 0.99 | 24 | 0.171 | 18 % | 0.286 | 3/24 | 23/24 |
| 0.2 to 0.4 | 27 | 0.099 | 12 % | -0.013 | 27/27 | 27/27 |
| < 0.2 | 8 | 0.117 | 4 % | 0.158 | 6/8 | 8/8 |

All cases: median error 0.218; in the 50 % band 88/155; in the 90 % band 151/155.

## 4. Sensitivity of the train score to each typed number

Each typed number carrying weight moved by +1 % of its value (forward difference, Python copy of law.rs checked against the Rust forward); change of the train score (negative = better). 54 numbers; base: {"loss: integrated Brier": 0.1569, "radiometer: CRPS of log10 V": 0.15}. Full list: `runs/dev/gen7/sensitivity/cld_0995df666bdabfa5.json`.

Largest for loss: integrated Brier:

| number | value | change of the score for +1 % |
|---|---|---|
| `loss.blowoff.median.b0` | 6.403 | +0.00365 |
| `x.x_fuel_final.scale` | 1 | +0.00117 |
| `x.x_fuel_final.centre` | 1 | -0.00106 |
| `x.window.centre` | 60 | -0.00062 |
| `x.v_coflow_final.centre` | 0.2 | +0.00060 |
| `loss.blowoff.median.skip[0]` | 0.55 | -0.00046 |
| `loss.blowoff.median.skip[1]` | -1 | +0.00045 |
| `x.v_coflow_final.scale` | 1 | +0.00045 |
| `loss.blowoff.median.b1[2]` | -1.204 | -0.00045 |
| `loss.blowoff.median.W1[1, 0]` | -1 | +0.00041 |
| `loss.blowoff.shape` | 3.4 | +0.00041 |
| `loss.other.median.b0` | 11.156 | +0.00039 |

Largest for radiometer: CRPS of log10 V:

| number | value | change of the score for +1 % |
|---|---|---|
| `x.x_fuel_stage.centre` | 1 | -0.00332 |
| `x.v_fuel_stage.centre` | 0.2 | -0.00139 |
| `radiometer.median.b1[0]` | 3.299 | +0.00138 |
| `radiometer.median.w2[0]` | 0.31 | +0.00121 |
| `x.x_fuel_stage.scale` | 1 | +0.00115 |
| `x.is_ethylene.scale` | 1 | -0.00082 |
| `radiometer.median.skip[4]` | 1 | -0.00058 |
| `radiometer.median.b0` | -0.404 | +0.00057 |
| `radiometer.median.w2[1]` | 0.38 | +0.00046 |
| `radiometer.median.b1[1]` | 1.099 | +0.00042 |
| `radiometer.median.skip[3]` | 0.5 | +0.00041 |
| `radiometer.median.W1[0, 4]` | 2.5 | -0.00035 |


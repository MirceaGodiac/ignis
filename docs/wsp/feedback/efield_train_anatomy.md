# efield: train anatomy of the W0 law (w0-fable, `sha256:73f91b55c81dc724fe2441c2254e109c65f8e08b253fcbe4c00a841f7a0986a9`)

TRAIN days only, written by `tools/wsp/ext/law_anatomy.py` after the registered look of PROTOCOL 13.2. For the revision the physicist will type; no number is changed here. Per-case file: `runs/dev/data_all/work/transfer_efield/law_study_73f91b55c81dc724.jsonl` (baselines beside it as `law_study_baseline_*.jsonl`).

## 1. Score against the baselines (train, on the cases the release answers)

### target t1

- Asked of the release: 2760; answered: 2735; refused: 25; refusal reasons: {"feature voltage": 25}
| source | CRPS uA, case mean | mean of day means | on |
|---|---|---|---|
| **release** | 0.7202 | 0.6690 | cases the release answers |
| constant (simple) | 0.5313 | 0.4968 | cases the release answers |
| constant (simple) | 0.5303 | 0.4991 | all lane cases |
| space-charge law up to saturation, train sigma (T1) | 0.4533 | 0.4225 | cases the release answers |
| space-charge law up to saturation, train sigma (T1) | 0.4482 | 0.4188 | all lane cases |
**Reading (CRPS uA, case mean):** release 0.7202; simple baseline (constant (simple)) 0.5313; best baseline (space-charge law up to saturation, train sigma (T1)) 0.4533: **loses (higher than the simple baseline)**.

### target t2

- Asked of the release: 123; answered: 122; refused: 1; refusal reasons: {"feature carbon_flow": 1, "feature voltage": 1, "feature sweep_v_max": 1, "feature window": 1}
| source | integrated Brier | Brier of ends in window | on |
|---|---|---|---|
| **release** | 0.1761 | 0.1689 | cases the release answers |
| constant (simple) | 0.1397 | 0.1649 | cases the release answers |
| constant (simple) | 0.1397 | 0.1649 | all lane cases |
| stratified incidence (T2; a stratum absent from train takes the pooled incidence) | 0.0544 | 0.1175 | cases the release answers |
| stratified incidence (T2; a stratum absent from train takes the pooled incidence) | 0.0544 | 0.1175 | all lane cases |
**Reading (integrated Brier):** release 0.1761; simple baseline (constant (simple)) 0.1397; best baseline (stratified incidence (T2; a stratum absent from train takes the pooled incidence)) 0.0544: **loses (higher than the simple baseline)**.

## 2. Refusals on train (admission, by reason)

- ion_current: 2735 answered, 25 refused; reasons: {"feature voltage = # V is outside its declared domain [#, #]": 25}
- flame_end: 122 answered, 1 refused; reasons: {"feature carbon_flow: condition \"carbon_flow\" missing from the prefix": 1, "feature voltage: condition \"voltage\" missing from the prefix": 1, "feature sweep_v_max: condition \"sweep_v_max\" missing from the prefix": 1, "feature window: condition \"window\" missing from the prefix": 1}
- peak_ion_current: 122 answered, 1 refused; reasons: {"feature carbon_flow: condition \"carbon_flow\" missing from the prefix": 1, "feature voltage: condition \"voltage\" missing from the prefix": 1, "feature sweep_v_max: condition \"sweep_v_max\" missing from the prefix": 1, "feature window: condition \"window\" missing from the prefix": 1}
- peak_voltage: 122 answered, 1 refused; reasons: {"feature carbon_flow: condition \"carbon_flow\" missing from the prefix": 1, "feature voltage: condition \"voltage\" missing from the prefix": 1, "feature sweep_v_max: condition \"sweep_v_max\" missing from the prefix": 1, "feature window: condition \"window\" missing from the prefix": 1}

## 3. Where the loss sits, and the sign of the error

### T1 ion current (CRPS uA per level; error = ln median forecast - ln outcome)

2100 scored train cases.

By fuel:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| ethylene 1.0 | 224 | 1.972 | 29 % | 0.225 | 67/224 | 133/224 |
| ethylene 0.3 | 443 | 0.875 | 26 % | -1.363 | 108/443 | 429/443 |
| methane 1.0 | 355 | 0.835 | 20 % | 0.371 | 107/355 | 238/355 |
| methane 0.7 | 317 | 0.489 | 10 % | 0.474 | 135/317 | 229/317 |
| methane 0.4 | 582 | 0.163 | 6 % | -0.158 | 419/582 | 558/582 |
| ethylene 0.8 | 81 | 0.987 | 5 % | 0.008 | 25/81 | 62/81 |
| ethylene 0.5 | 62 | 0.491 | 2 % | -0.553 | 32/62 | 54/62 |
| ethylene 0.25 | 36 | 0.736 | 2 % | -1.609 | 5/36 | 35/36 |

By burner and polarity:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| coflow positive | 1071 | 0.782 | 55 % | 0.166 | 397/1071 | 812/1071 |
| coflow negative | 552 | 0.661 | 24 % | -0.700 | 273/552 | 544/552 |
| gas_jet_2.1mm positive | 299 | 0.793 | 16 % | -0.177 | 93/299 | 216/299 |
| gas_jet_2.1mm negative | 178 | 0.407 | 5 % | 0.004 | 135/178 | 166/178 |

By coflow:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| coflow on | 1327 | 0.842 | 74 % | -0.395 | 553/1327 | 1145/1327 |
| no coflow | 773 | 0.511 | 26 % | 0.076 | 345/773 | 593/773 |

By day:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 18137 | 252 | 1.905 | 32 % | 0.345 | 61/252 | 148/252 |
| 18092 | 288 | 1.130 | 22 % | -1.995 | 65/288 | 280/288 |
| 18113 | 159 | 0.892 | 9 % | 1.134 | 21/159 | 100/159 |
| 18312 | 148 | 0.937 | 9 % | 1.013 | 25/148 | 80/148 |
| 18319 | 151 | 0.652 | 7 % | -0.580 | 68/151 | 136/151 |
| 18130 | 187 | 0.526 | 7 % | 0.831 | 61/187 | 121/187 |
| 18282 | 178 | 0.407 | 5 % | 0.004 | 135/178 | 166/178 |
| 18095 | 155 | 0.402 | 4 % | -1.067 | 43/155 | 149/155 |
| 18080 | 264 | 0.149 | 3 % | -0.370 | 208/264 | 264/264 |
| 18102 | 186 | 0.150 | 2 % | -0.252 | 165/186 | 186/186 |
| 18106 | 132 | 0.208 | 2 % | 0.707 | 46/132 | 108/132 |

By level |V|, kV:

| stratum | n | mean loss | share of the loss | median error of the median | in 50 % band | in 90 % band |
|---|---|---|---|---|---|---|
| 6 to 8 | 307 | 1.370 | 28 % | -0.540 | 117/307 | 268/307 |
| 4 to 6 | 403 | 0.955 | 25 % | -0.052 | 199/403 | 323/403 |
| >= 8 | 244 | 1.323 | 21 % | -0.916 | 84/244 | 233/244 |
| 2 to 4 | 560 | 0.495 | 18 % | 0.151 | 285/560 | 451/560 |
| 1 to 2 | 311 | 0.250 | 5 % | -0.126 | 137/311 | 273/311 |
| < 1 | 275 | 0.106 | 2 % | -0.818 | 76/275 | 190/275 |

All cases: median error -0.148; in the 50 % band 898/2100; in the 90 % band 1738/2100.

### T2 flame out under the field (per test: mean Brier over the eligible horizons; forecast and observed = out by 60 s, tests observed to 60 s)

61 scored train cases.

By fuel:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| methane 1.0 | 10 | 0.230 | 38 % | 0.045 | 0.700 | -0.655 |
| methane 0.7 | 11 | 0.178 | 32 % | 0.076 | 0.636 | -0.561 |
| methane 0.4 | 19 | 0.092 | 29 % | 0.117 | 0.263 | -0.146 |
| ethylene 0.3 | 9 | 0.002 | 0 % | 0.111 | 0.000 | 0.111 |
| ethylene 0.25 | 2 | 0.004 | 0 % | 0.142 | 0.000 | 0.142 |
| ethylene 0.5 | 3 | 8.43e-04 | 0 % | 0.065 | 0.000 | 0.065 |
| ethylene 1.0 | 5 | 2.72e-04 | 0 % | 0.032 | 0.000 | 0.032 |
| ethylene 0.8 | 2 | 3.40e-04 | 0 % | 0.041 | 0.000 | 0.041 |

By burner and polarity:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| coflow positive | 31 | 0.157 | 80 % | 0.088 | 0.355 | -0.267 |
| gas_jet_2.1mm positive | 13 | 0.055 | 12 % | 0.075 | 0.308 | -0.233 |
| gas_jet_2.1mm negative | 5 | 0.088 | 7 % | 0.063 | 0.800 | -0.737 |
| coflow negative | 12 | 0.002 | 0 % | 0.101 | 0.000 | 0.101 |

By coflow:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| no coflow | 35 | 0.170 | 99 % | 0.083 | 0.543 | -0.460 |
| coflow on | 26 | 0.003 | 1 % | 0.089 | 0.000 | 0.089 |

By day:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 18113 | 5 | 0.335 | 28 % | 0.038 | 0.600 | -0.562 |
| 18130 | 6 | 0.240 | 24 % | 0.065 | 0.500 | -0.435 |
| 18106 | 7 | 0.129 | 15 % | 0.142 | 0.571 | -0.430 |
| 18102 | 5 | 0.165 | 14 % | 0.108 | 0.200 | -0.092 |
| 18312 | 5 | 0.140 | 12 % | 0.077 | 0.800 | -0.723 |
| 18282 | 5 | 0.088 | 7 % | 0.063 | 0.800 | -0.737 |
| 18080 | 7 | 0.002 | 0 % | 0.098 | 0.000 | 0.098 |
| 18092 | 5 | 0.002 | 0 % | 0.106 | 0.000 | 0.106 |
| 18319 | 8 | 0.001 | 0 % | 0.073 | 0.000 | 0.073 |
| 18095 | 4 | 0.003 | 0 % | 0.117 | 0.000 | 0.117 |
| 18137 | 4 | 3.19e-04 | 0 % | 0.034 | 0.000 | 0.034 |

Calibration (forecast probability bin):

| bin | n | mean forecast | observed rate |
|---|---|---|---|
| < 0.05 | 15 | 0.037 | 0.333 |
| 0.05 to 0.15 | 44 | 0.098 | 0.318 |
| 0.15 to 0.3 | 2 | 0.175 | 0.000 |

## 4. Sensitivity of the train score to each typed number

Each typed number carrying weight moved by +1 % of its value (forward difference, Python copy of law.rs checked against the Rust forward); change of the train score (negative = better). 68 numbers; base: {"t1: CRPS uA, case mean": 0.7202, "t2: integrated Brier": 0.1761}. Full list: `runs/dev/gen7/sensitivity/efield_73f91b55c81dc724.json`.

Largest for t1: CRPS uA, case mean:

| number | value | change of the score for +1 % |
|---|---|---|
| `ion_current.median.b0` | -12.939 | +0.07718 |
| `ion_current.median.skip[0]` | 1 | +0.00472 |
| `x.carbon_flow.centre` | 1e+19 | -0.00410 |
| `x.x_fuel.centre` | 1 | -0.00410 |
| `x.carbon_flow.scale` | 1 | -0.00394 |
| `x.voltage.centre` | 3000 | -0.00208 |
| `ion_current.median.W1[1, 4]` | 2 | -0.00187 |
| `ion_current.median.W1[0, 0]` | 1 | -0.00154 |
| `ion_current.median.W1[0, 4]` | -2 | -0.00133 |
| `ion_current.median.W1[1, 0]` | -1 | -0.00108 |
| `ion_current.median.b1[0]` | 0.524 | -0.00100 |
| `ion_current.median.b1[1]` | -0.824 | -0.00092 |

Largest for t2: integrated Brier:

| number | value | change of the score for +1 % |
|---|---|---|
| `flame_end.blowoff.median.b0` | 5.991 | +0.00182 |
| `flame_end.other.median.b0` | 6.397 | +0.00131 |
| `x.x_fuel.centre` | 1 | -0.00047 |
| `flame_end.other.shape` | 1.5 | +0.00038 |
| `flame_end.blowoff.median.w2[0]` | 1 | +0.00036 |
| `x.window.centre` | 100 | -0.00032 |
| `flame_end.blowoff.shape` | 3 | +0.00029 |
| `flame_end.blowoff.median.skip[1]` | 1.5 | -0.00024 |
| `x.x_fuel.scale` | 1 | +0.00021 |
| `flame_end.blowoff.median.W1[0, 5]` | 1 | +0.00018 |
| `flame_end.blowoff.median.skip[5]` | -1 | +0.00018 |
| `flame_end.blowoff.median.skip[3]` | -0.511 | +0.00016 |


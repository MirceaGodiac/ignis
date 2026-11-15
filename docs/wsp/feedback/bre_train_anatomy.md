# bre: train anatomy of the W0 law (w0-fable, `sha256:aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d`)

TRAIN days only, written by `tools/wsp/ext/law_anatomy.py` after the registered look of PROTOCOL 13.2. For the revision the physicist will type; no number is changed here. Per-case file: `runs/dev/data_all/work/transfer_bre/law_study_aa0eef5bc7653922.jsonl` (baselines beside it as `law_study_baseline_*.jsonl`).

## 1. Score against the baselines (train, on the cases the release answers)

### target t1

- Asked of the release: 279; answered: 240; refused: 39; refusal reasons: {"feature burner_mass_flux": 21, "feature x_fuel": 24, "feature pressure": 3}
| source | extinction IBS | operator termination IBS | mean of the two causes | on |
|---|---|---|---|---|
| **release** | 0.2511 | 0.0494 | 0.1502 | cases the release answers |
| incidence (simple) | 0.2300 | 0.0646 | 0.1473 | cases the release answers |
| incidence (simple) | 0.2293 | 0.0766 | 0.1530 | all lane cases |
| planned stop | 0.2320 | 0.0618 | 0.1469 | cases the release answers |
| planned stop | 0.2311 | 0.0688 | 0.1500 | all lane cases |
**Reading (extinction IBS):** release 0.2511; simple baseline (incidence (simple)) 0.2300; best baseline (incidence (simple)) 0.2300: **loses (higher than the simple baseline)**.

**Reading (operator termination IBS):** release 0.0494; simple baseline (incidence (simple)) 0.0646; best baseline (planned stop) 0.0618: **beats the best baseline**.

## 2. Refusals on train (admission, by reason)

- flame_end: 93 answered, 23 refused; reasons: {"feature burner_mass_flux: condition \"burner_mass_flux\" missing from the prefix": 10, "feature x_fuel: condition \"x_fuel\" missing from the prefix": 10, "feature x_fuel = # # is outside its declared domain [#, #]": 7, "feature pressure: condition \"pressure\" missing from the prefix": 6, "feature burner_mass_flux = # kg/m#/s is outside its declared domain [#, #]": 6, "feature x_o#: condition \"x_o#\" missing from the prefix": 5}
- lasts: 93 answered, 23 refused; reasons: {"feature burner_mass_flux: condition \"burner_mass_flux\" missing from the prefix": 10, "feature x_fuel: condition \"x_fuel\" missing from the prefix": 10, "feature x_fuel = # # is outside its declared domain [#, #]": 7, "feature pressure: condition \"pressure\" missing from the prefix": 6, "feature burner_mass_flux = # kg/m#/s is outside its declared domain [#, #]": 6, "feature x_o#: condition \"x_o#\" missing from the prefix": 5}
- height_at_end: 93 answered, 23 refused; reasons: {"feature burner_mass_flux: condition \"burner_mass_flux\" missing from the prefix": 10, "feature x_fuel: condition \"x_fuel\" missing from the prefix": 10, "feature x_fuel = # # is outside its declared domain [#, #]": 7, "feature pressure: condition \"pressure\" missing from the prefix": 6, "feature burner_mass_flux = # kg/m#/s is outside its declared domain [#, #]": 6, "feature x_o#: condition \"x_o#\" missing from the prefix": 5}

## 3. Where the loss sits, and the sign of the error

### T1 extinction (per case: IPCW integrated Brier; forecast and observed = incidence by 60 s after onset)

240 scored train cases.

By fuel:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| ethylene | 210 | 0.251 | 88 % | 0.614 | 0.400 | 0.214 |
| methane | 30 | 0.249 | 12 % | 0.411 | 1.000 | -0.589 |

By bottle fuel fraction:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 1.0 | 174 | 0.223 | 65 % | 0.582 | 0.534 | 0.047 |
| 0.5 | 66 | 0.324 | 35 % | 0.608 | 0.318 | 0.290 |

By burner diameter, m:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 0.025 | 189 | 0.268 | 84 % | 0.547 | 0.476 | 0.071 |
| 0.05 | 51 | 0.189 | 16 % | 0.742 | 0.471 | 0.272 |

By ambient O2:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 0.3 to 0.36 | 76 | 0.303 | 38 % | 0.521 | 0.329 | 0.192 |
| 0.25 to 0.3 | 58 | 0.363 | 35 % | 0.729 | 0.328 | 0.401 |
| >= 0.36 | 61 | 0.230 | 23 % | 0.322 | 0.410 | -0.088 |
| < 0.25 | 45 | 0.047 | 4 % | 0.886 | 1.000 | -0.114 |

By pressure, Pa:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 65000 to 85000 | 46 | 0.446 | 34 % | 0.726 | 0.152 | 0.574 |
| >= 85000 | 129 | 0.156 | 33 % | 0.631 | 0.698 | -0.067 |
| < 65000 | 65 | 0.301 | 32 % | 0.409 | 0.262 | 0.148 |

By day:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 19066 | 23 | 0.455 | 17 % | 0.814 | 0.348 | 0.466 |
| 19087 | 21 | 0.479 | 17 % | 0.510 | 0.000 | 0.510 |
| 19070 | 25 | 0.370 | 15 % | 0.559 | 0.160 | 0.399 |
| 20339 | 30 | 0.249 | 12 % | 0.411 | 1.000 | -0.589 |
| 19094 | 27 | 0.246 | 11 % | 0.570 | 0.667 | -0.097 |
| 20247 | 18 | 0.288 | 9 % | 0.776 | 0.167 | 0.609 |
| 21028 | 33 | 0.134 | 7 % | 0.724 | 0.636 | 0.088 |
| 20261 | 18 | 0.195 | 6 % | 0.410 | 0.000 | 0.410 |
| 19056 | 18 | 0.092 | 3 % | 0.414 | 0.500 | -0.086 |
| 19039 | 27 | 0.058 | 3 % | 0.649 | 0.778 | -0.129 |

By cutoff s:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 1.0 | 82 | 0.246 | 33 % | 0.600 | 0.488 | 0.112 |
| 3.0 | 82 | 0.245 | 33 % | 0.599 | 0.488 | 0.111 |
| 5.0 | 76 | 0.263 | 33 % | 0.566 | 0.447 | 0.119 |

Calibration (forecast probability bin):

| bin | n | mean forecast | observed rate |
|---|---|---|---|
| 0.05 to 0.15 | 45 | 0.117 | 0.133 |
| 0.15 to 0.3 | 27 | 0.218 | 0.889 |
| 0.3 to 0.5 | 30 | 0.418 | 0.000 |
| 0.5 to 0.7 | 30 | 0.626 | 0.700 |
| 0.7 to 0.85 | 22 | 0.819 | 0.727 |
| 0.85 to 0.95 | 44 | 0.896 | 0.455 |
| >= 0.95 | 42 | 0.986 | 0.643 |

### T1 operator_termination (per case: IPCW integrated Brier; forecast and observed = incidence by 60 s after onset)

240 scored train cases.

By fuel:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| ethylene | 210 | 0.050 | 89 % | 3.47e-04 | 0.000 | 3.47e-04 |
| methane | 30 | 0.044 | 11 % | 0.000 | 0.000 | 0.000 |

By bottle fuel fraction:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 1.0 | 174 | 0.049 | 73 % | 4.18e-04 | 0.000 | 4.18e-04 |
| 0.5 | 66 | 0.049 | 27 % | 0.000 | 0.000 | 0.000 |

By burner diameter, m:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 0.025 | 189 | 0.040 | 64 % | 0.000 | 0.000 | 0.000 |
| 0.05 | 51 | 0.084 | 36 % | 0.001 | 0.000 | 0.001 |

By ambient O2:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| >= 0.36 | 61 | 0.082 | 42 % | 0.000 | 0.000 | 0.000 |
| 0.3 to 0.36 | 76 | 0.058 | 37 % | 0.000 | 0.000 | 0.000 |
| 0.25 to 0.3 | 58 | 0.042 | 21 % | 0.001 | 0.000 | 0.001 |
| < 0.25 | 45 | 3.95e-04 | 0 % | 0.000 | 0.000 | 0.000 |

By pressure, Pa:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| >= 85000 | 129 | 0.046 | 50 % | 5.64e-04 | 0.000 | 5.64e-04 |
| < 65000 | 65 | 0.053 | 29 % | 0.000 | 0.000 | 0.000 |
| 65000 to 85000 | 46 | 0.053 | 21 % | 0.000 | 0.000 | 0.000 |

By day:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 21028 | 33 | 0.076 | 21 % | 0.000 | 0.000 | 0.000 |
| 19070 | 25 | 0.082 | 17 % | 0.000 | 0.000 | 0.000 |
| 20247 | 18 | 0.097 | 15 % | 0.004 | 0.000 | 0.004 |
| 20261 | 18 | 0.086 | 13 % | 0.000 | 0.000 | 0.000 |
| 20339 | 30 | 0.044 | 11 % | 0.000 | 0.000 | 0.000 |
| 19066 | 23 | 0.042 | 8 % | 0.000 | 0.000 | 0.000 |
| 19087 | 21 | 0.045 | 8 % | 0.000 | 0.000 | 0.000 |
| 19094 | 27 | 0.010 | 2 % | 0.000 | 0.000 | 0.000 |
| 19039 | 27 | 0.009 | 2 % | 0.000 | 0.000 | 0.000 |
| 19056 | 18 | 0.013 | 2 % | 0.000 | 0.000 | 0.000 |

By cutoff s:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 1.0 | 82 | 0.048 | 33 % | 2.71e-04 | 0.000 | 2.71e-04 |
| 3.0 | 82 | 0.048 | 33 % | 2.89e-04 | 0.000 | 2.89e-04 |
| 5.0 | 76 | 0.052 | 33 % | 3.54e-04 | 0.000 | 3.54e-04 |

Calibration (forecast probability bin):

| bin | n | mean forecast | observed rate |
|---|---|---|---|
| < 0.05 | 240 | 3.03e-04 | 0.000 |

## 4. Sensitivity of the train score to each typed number

Each typed number carrying weight moved by +1 % of its value (forward difference, Python copy of law.rs checked against the Rust forward); change of the train score (negative = better). 164 numbers; base: {"t1: extinction IBS": 0.2511}. Full list: `runs/dev/gen7/sensitivity/bre_aa0eef5bc7653922.json`.

Largest for t1: extinction IBS:

| number | value | change of the score for +1 % |
|---|---|---|
| `x.burner_mass_flux.centre` | 0.004 | +0.00206 |
| `x.x_fuel.centre` | 1 | +0.00186 |
| `flame_end.extinction.median.b1[5]` | -1.7 | +0.00157 |
| `x.burner_mass_flux.scale` | 1 | -0.00153 |
| `flame_end.extinction.median.W1[5, 1]` | -2.17 | -0.00147 |
| `flame_end.extinction.median.b0` | 2.82 | -0.00103 |
| `flame_end.extinction.median.skip[1]` | 1.05 | +0.00085 |
| `x.x_fuel.scale` | 1 | -0.00084 |
| `x.x_o2.centre` | 0.265 | +0.00080 |
| `flame_end.extinction.median.W1[5, 4]` | -1.97 | -0.00077 |
| `x.pressure.centre` | 101325 | -0.00071 |
| `x.burner_diameter.centre` | 0.025 | +0.00064 |


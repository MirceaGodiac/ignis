# slice: train anatomy of the W0 law (w0-fable, `sha256:d10c94bc6ea0bcb957f89734393b0cbabbd44fdf0ac9c2e440f051a33cd3006e`)

TRAIN days only, written by `tools/wsp/ext/law_anatomy.py` after the registered look of PROTOCOL 13.2. For the revision the physicist will type; no number is changed here. Per-case file: `runs/dev/data_all/work/transfer_slice/law_study_d10c94bc6ea0bcb9.jsonl` (baselines beside it as `law_study_baseline_*.jsonl`).

## 1. Score against the baselines (train, on the cases the release answers)

### target blow_off

- Asked of the release: 289; answered: 273; refused: 16; refusal reasons: {"feature x_o2": 14, "feature u_coflow": 2}
| source | Brier, mean of day means | pooled | on |
|---|---|---|---|
| **release** | 0.1023 | 0.1101 | cases the release answers |
| incidence pooled (simple) | 0.0352 | 0.0428 | cases the release answers |
| incidence pooled (simple) | 0.0535 | 0.0504 | all lane cases |
| incidence by nominal tube | 0.0335 | 0.0410 | cases the release answers |
| incidence by nominal tube | 0.0516 | 0.0484 | all lane cases |
| never (persistence) | 0.0362 | 0.0448 | cases the release answers |
| never (persistence) | 0.0567 | 0.0532 | all lane cases |
**Reading (Brier, mean of day means):** release 0.1023; simple baseline (incidence pooled (simple)) 0.0352; best baseline (incidence by nominal tube) 0.0335: **loses (higher than the simple baseline)**.

### target lifted

- Asked of the release: 289; answered: 273; refused: 16; refusal reasons: {"feature x_o2": 14, "feature u_coflow": 2}
| source | Brier, mean of day means | pooled | on |
|---|---|---|---|
| **release** | 0.1559 | 0.1798 | cases the release answers |
| incidence pooled (simple) | 0.0258 | 0.0290 | cases the release answers |
| incidence pooled (simple) | 0.0258 | 0.0276 | all lane cases |
| incidence by nominal tube | 0.0249 | 0.0280 | cases the release answers |
| incidence by nominal tube | 0.0249 | 0.0267 | all lane cases |
| never (persistence) | 0.0265 | 0.0299 | cases the release answers |
| never (persistence) | 0.0265 | 0.0284 | all lane cases |
**Reading (Brier, mean of day means):** release 0.1559; simple baseline (incidence pooled (simple)) 0.0258; best baseline (incidence by nominal tube) 0.0249: **loses (higher than the simple baseline)**.

## 2. Refusals on train (admission, by reason)

- blow_off: 273 answered, 16 refused; reasons: {"feature x_o#: condition \"x_o#\" missing from the prefix": 14, "feature u_coflow: condition \"u_coflow\" missing from the prefix": 2}
- lifted: 273 answered, 16 refused; reasons: {"feature x_o#: condition \"x_o#\" missing from the prefix": 14, "feature u_coflow: condition \"u_coflow\" missing from the prefix": 2}
- flame_length: 273 answered, 16 refused; reasons: {"feature x_o#: condition \"x_o#\" missing from the prefix": 14, "feature u_coflow: condition \"u_coflow\" missing from the prefix": 2}

## 3. Where the loss sits, and the sign of the error

### blow_off (Brier per line)

268 scored train cases.

By fuel:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| methane 0.4 | 39 | 0.575 | 76 % | 0.670 | 0.077 | 0.593 |
| methane 1.0 | 117 | 0.048 | 19 % | 0.055 | 0.060 | -0.005 |
| ethane 1.0 | 79 | 0.018 | 5 % | 0.056 | 0.025 | 0.030 |
| methane 0.7 | 28 | 6.45e-04 | 0 % | 0.021 | 0.000 | 0.021 |
| propane 1.0 | 5 | 1.45e-04 | 0 % | 0.009 | 0.000 | 0.009 |

By tube, mm:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 0.4 | 81 | 0.331 | 91 % | 0.404 | 0.111 | 0.293 |
| 0.8 | 66 | 0.026 | 6 % | 0.042 | 0.030 | 0.011 |
| 1.6 | 40 | 0.023 | 3 % | 0.021 | 0.025 | -0.004 |
| 3.2 | 60 | 4.70e-04 | 0 % | 0.016 | 0.000 | 0.016 |
| 2.1 | 21 | 4.98e-04 | 0 % | 0.015 | 0.000 | 0.015 |

By coflow, cm/s:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 20 to 30 | 48 | 0.191 | 31 % | 0.184 | 0.083 | 0.101 |
| < 10 | 77 | 0.107 | 28 % | 0.147 | 0.026 | 0.121 |
| 30 to 45 | 88 | 0.080 | 24 % | 0.123 | 0.068 | 0.055 |
| 10 to 20 | 36 | 0.123 | 15 % | 0.140 | 0.000 | 0.140 |
| >= 45 | 19 | 0.033 | 2 % | 0.085 | 0.000 | 0.085 |

By jet, m/s:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| >= 15 | 22 | 0.650 | 48 % | 0.754 | 0.136 | 0.618 |
| 5 to 15 | 61 | 0.231 | 48 % | 0.270 | 0.131 | 0.139 |
| < 1 | 103 | 0.009 | 3 % | 0.017 | 0.010 | 0.007 |
| 1 to 5 | 82 | 0.002 | 0 % | 0.034 | 0.000 | 0.034 |

By day:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 061 | 39 | 0.575 | 76 % | 0.670 | 0.077 | 0.593 |
| 058 | 47 | 0.100 | 16 % | 0.096 | 0.128 | -0.032 |
| 069 | 79 | 0.018 | 5 % | 0.056 | 0.025 | 0.030 |
| 072 | 42 | 0.021 | 3 % | 0.027 | 0.024 | 0.003 |
| 040 | 28 | 0.001 | 0 % | 0.029 | 0.000 | 0.029 |
| 079 | 26 | 6.30e-04 | 0 % | 0.020 | 0.000 | 0.020 |
| 048 | 7 | 3.45e-04 | 0 % | 0.014 | 0.000 | 0.014 |

Calibration (forecast probability bin):

| bin | n | mean forecast | observed rate |
|---|---|---|---|
| < 0.05 | 172 | 0.020 | 0.000 |
| 0.05 to 0.15 | 48 | 0.079 | 0.083 |
| 0.15 to 0.3 | 16 | 0.208 | 0.312 |
| 0.3 to 0.5 | 3 | 0.397 | 0.333 |
| 0.5 to 0.7 | 2 | 0.699 | 0.000 |
| 0.7 to 0.85 | 5 | 0.773 | 0.000 |
| 0.85 to 0.95 | 13 | 0.903 | 0.077 |
| >= 0.95 | 9 | 0.979 | 0.111 |

### lifted (Brier per line)

268 scored train cases.

By fuel:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| methane 0.4 | 39 | 0.712 | 58 % | 0.773 | 0.077 | 0.696 |
| methane 1.0 | 117 | 0.099 | 24 % | 0.227 | 0.043 | 0.185 |
| ethane 1.0 | 79 | 0.109 | 18 % | 0.195 | 0.000 | 0.195 |
| methane 0.7 | 28 | 0.008 | 0 % | 0.069 | 0.000 | 0.069 |
| propane 1.0 | 5 | 8.76e-04 | 0 % | 0.020 | 0.000 | 0.020 |

By tube, mm:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 0.4 | 81 | 0.533 | 90 % | 0.689 | 0.074 | 0.615 |
| 0.8 | 66 | 0.053 | 7 % | 0.175 | 0.015 | 0.160 |
| 1.6 | 40 | 0.025 | 2 % | 0.071 | 0.025 | 0.046 |
| 3.2 | 60 | 0.006 | 1 % | 0.050 | 0.000 | 0.050 |
| 2.1 | 21 | 0.007 | 0 % | 0.050 | 0.000 | 0.050 |

By coflow, cm/s:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| < 10 | 77 | 0.224 | 36 % | 0.290 | 0.013 | 0.277 |
| 30 to 45 | 88 | 0.150 | 27 % | 0.300 | 0.057 | 0.243 |
| 20 to 30 | 48 | 0.222 | 22 % | 0.313 | 0.042 | 0.271 |
| 10 to 20 | 36 | 0.146 | 11 % | 0.165 | 0.000 | 0.165 |
| >= 45 | 19 | 0.094 | 4 % | 0.241 | 0.000 | 0.241 |

By jet, m/s:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 5 to 15 | 61 | 0.391 | 50 % | 0.590 | 0.082 | 0.508 |
| >= 15 | 22 | 0.905 | 41 % | 0.964 | 0.045 | 0.919 |
| 1 to 5 | 82 | 0.037 | 6 % | 0.138 | 0.012 | 0.125 |
| < 1 | 103 | 0.013 | 3 % | 0.056 | 0.010 | 0.046 |

By day:

| stratum | n | Brier | share of the loss | mean forecast | observed rate | forecast - observed |
|---|---|---|---|---|---|---|
| 061 | 39 | 0.712 | 58 % | 0.773 | 0.077 | 0.696 |
| 058 | 47 | 0.200 | 20 % | 0.400 | 0.085 | 0.315 |
| 069 | 79 | 0.109 | 18 % | 0.195 | 0.000 | 0.195 |
| 072 | 42 | 0.039 | 3 % | 0.113 | 0.024 | 0.089 |
| 040 | 28 | 0.020 | 1 % | 0.110 | 0.000 | 0.110 |
| 079 | 26 | 0.008 | 0 % | 0.067 | 0.000 | 0.067 |
| 048 | 7 | 0.004 | 0 % | 0.039 | 0.000 | 0.039 |

Calibration (forecast probability bin):

| bin | n | mean forecast | observed rate |
|---|---|---|---|
| < 0.05 | 91 | 0.012 | 0.000 |
| 0.05 to 0.15 | 57 | 0.107 | 0.000 |
| 0.15 to 0.3 | 40 | 0.210 | 0.025 |
| 0.3 to 0.5 | 23 | 0.386 | 0.043 |
| 0.5 to 0.7 | 9 | 0.572 | 0.111 |
| 0.7 to 0.85 | 11 | 0.760 | 0.273 |
| 0.85 to 0.95 | 7 | 0.892 | 0.000 |
| >= 0.95 | 30 | 0.998 | 0.067 |

## 4. Sensitivity of the train score to each typed number

Each typed number carrying weight moved by +1 % of its value (forward difference, Python copy of law.rs checked against the Rust forward); change of the train score (negative = better). 96 numbers; base: {"blow_off: Brier, mean of day means": 0.1023, "lifted: Brier, mean of day means": 0.1559}. Full list: `runs/dev/gen7/sensitivity/slice_d10c94bc6ea0bcb9.json`.

Largest for blow_off: Brier, mean of day means:

| number | value | change of the score for +1 % |
|---|---|---|
| `x.x_o2.centre` | 0.21 | +0.00082 |
| `x.u_jet.scale` | 1 | -0.00066 |
| `x.x_fuel.centre` | 1 | +0.00063 |
| `x.x_fuel.scale` | 1 | -0.00061 |
| `blow_off.logit.b0` | -4.158 | +0.00057 |
| `blow_off.logit.w2[0]` | 0.62 | +0.00046 |
| `blow_off.logit.b1[2]` | -4.685 | +0.00037 |
| `x.burner_id.scale` | 1 | -0.00036 |
| `blow_off.logit.w2[1]` | 0.76 | +0.00034 |
| `x.u_jet.centre` | 1 | -0.00026 |
| `x.burner_id.centre` | 0.0016 | +0.00026 |
| `blow_off.logit.W1[1, 1]` | 1 | +0.00024 |

Largest for lifted: Brier, mean of day means:

| number | value | change of the score for +1 % |
|---|---|---|
| `x.x_o2.centre` | 0.21 | +0.00396 |
| `x.x_fuel.centre` | 1 | +0.00166 |
| `x.u_jet.scale` | 1 | -0.00152 |
| `lifted.logit.b0` | -3.356 | +0.00127 |
| `lifted.logit.w2[0]` | 1.054 | +0.00109 |
| `x.burner_id.scale` | 1 | -0.00085 |
| `x.burner_id.centre` | 0.0016 | +0.00073 |
| `x.u_jet.centre` | 1 | -0.00072 |
| `lifted.logit.b1[1]` | -2.485 | +0.00068 |
| `lifted.logit.W1[0, 1]` | 1 | +0.00066 |
| `lifted.logit.W1[1, 1]` | 1 | +0.00062 |
| `x.u_coflow.centre` | 0.3 | -0.00053 |


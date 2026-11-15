# bass1: seventh-generation W0 law, scores beside the baselines

## bass1: release `sha256:97c0daf9de84482ed7b125b3d41302521f5dd7281c0667241807d93387e239a2`, partition validation, look 13.2

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.

- sustain: the law answers whether a flame, given that it was established, is still there 60 s after the igniter is switched off (phys_bass1_w0.md section 9); it does not forecast ignition. P = the smaller of holds_low_flow and holds_high_flow (rod tips, spheres, sections) or of the flat_ pair (films, slabs up to 5 mm), at the initial oxygen, the duct speed at ignition, nose radius diameter / 2 (0.01 flat); heat_time 200 s for every case (no case carries a planned igniter on-time).
- heat_time = 200 s (no planned igniter on-time among the inputs) for every covered case: {"quench": 28, "sustain": 22}.

### target quench

- Asked of the release: 41; answered: 28; refused: 13; refusal reasons: {"not covered: fabric_sibal|None": 1, "not covered: candle|wake": 8, "not covered: pmma_sphere|wake": 3, "not covered: candle|stagnation": 1}
- Lane cases without a release forecast (not scored): 2

| source | interval CRPS cm/s, mean of day means | case mean | on |
|---|---|---|---|
| **release** | 0.5169 | 0.3936 | cases the release answers |
| constant (simple) | 0.2299 | 0.2409 | cases the release answers |
| constant (simple) | 0.2258 | 0.2345 | all lane cases |
| Bhattacharjee et al. 2016, train log width | 0.1797 | 0.1869 | cases the release answers |
| Bhattacharjee et al. 2016, train log width | 0.1647 | 0.1692 | all lane cases |

**Reading (interval CRPS cm/s, mean of day means):** release 0.5169; simple baseline (constant (simple)) 0.2299; best baseline (Bhattacharjee et al. 2016, train log width) 0.1797: **loses (higher than the simple baseline)**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "target": "quench",
  "n_cases": 9,
  "n_days": 5,
  "refused": 2,
  "case_mean": 0.39359780844968273,
  "day_mean": 0.5169331688458707,
  "per_day": {
   "2012-06-11": 0.018476839378789127,
   "2014-07-03": 1.4130297651465364,
   "2014-07-29": 0.6247640682830535,
   "2014-07-31": 0.2959966847771588,
   "2014-08-04": 0.23239848664381593
  }
 },
 "baselines_on_cases_answered": {
  "constant (simple)": {
   "target": "quench",
   "n_cases": 9,
   "n_days": 5,
   "refused": 2,
   "case_mean": 0.24089555555555567,
   "day_mean": 0.22990800000000008,
   "per_day": {
    "2012-06-11": 0.16072000000000014,
    "2014-07-03": 0.18300000000000013,
    "2014-07-29": 0.5490200000000001,
    "2014-07-31": 0.14806000000000014,
    "2014-08-04": 0.10874000000000009
   }
  },
  "Bhattacharjee et al. 2016, train log width": {
   "target": "quench",
   "n_cases": 9,
   "n_days": 5,
   "refused": 2,
   "case_mean": 0.1868760163058704,
   "day_mean": 0.1797397532546717,
   "per_day": {
    "2012-06-11": 0.03551449995035004,
    "2014-07-03": 0.09908143103595739,
    "2014-07-29": 0.6251841460373833,
    "2014-07-31": 0.08697223454139175,
    "2014-08-04": 0.051946454708276114
   }
  }
 }
}
```

</details>

### target sustain

- Asked of the release: 41; answered: 22; refused: 19; refusal reasons: {"not covered: fabric_sibal|None": 1, "not covered: candle|wake": 8, "not covered: pmma_sphere|wake": 3, "not covered: candle|stagnation": 1, "not covered: pmma_rod|opposed": 6}
- Lane cases without a release forecast (not scored): 18

| source | Brier, mean of day means | case mean | on |
|---|---|---|---|
| **release** | 0.0862 | 0.0681 | cases the release answers |
| train incidence (simple) | 0.1203 | 0.0831 | cases the release answers |
| train incidence (simple) | 0.0583 | 0.0707 | all lane cases |
| always lit | 0.1389 | 0.0909 | cases the release answers |
| always lit | 0.0590 | 0.0750 | all lane cases |

**Reading (Brier, mean of day means):** release 0.0862; simple baseline (train incidence (simple)) 0.1203; best baseline (train incidence (simple)) 0.1203: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "target": "sustain",
  "n_cases": 22,
  "n_days": 9,
  "refused": 18,
  "case_mean": 0.0681285745916907,
  "day_mean": 0.08623725372098842,
  "per_day": {
   "2012-03-30": 1.6332249804134134e-05,
   "2012-06-11": 8.366581585446132e-05,
   "2013-04-26": 1.4702929820431785e-05,
   "2013-05-15": 0.23940397569418947,
   "2014-07-09": 0.05767875286127229,
   "2014-07-15": 0.0024254697259846144,
   "2014-07-29": 0.4759201797927145,
   "2014-07-31": 0.0005908618366513302,
   "2014-08-04": 1.34258260460819e-06
  }
 },
 "baselines_on_cases_answered": {
  "train incidence (simple)": {
   "target": "sustain",
   "n_cases": 22,
   "n_days": 9,
   "refused": 18,
   "case_mean": 0.08308945454545455,
   "day_mean": 0.12032177777777779,
   "per_day": {
    "2012-03-30": 0.012543999999999998,
    "2012-06-11": 0.012543999999999998,
    "2013-04-26": 0.012543999999999998,
    "2013-05-15": 0.206544,
    "2014-07-09": 0.012543999999999998,
    "2014-07-15": 0.012543999999999998,
    "2014-07-29": 0.788544,
    "2014-07-31": 0.012543999999999998,
    "2014-08-04": 0.012543999999999998
   }
  },
  "always lit": {
   "target": "sustain",
   "n_cases": 22,
   "n_days": 9,
   "refused": 18,
   "case_mean": 0.09090909090909091,
   "day_mean": 0.1388888888888889,
   "per_day": {
    "2012-03-30": 0.0,
    "2012-06-11": 0.0,
    "2013-04-26": 0.0,
    "2013-05-15": 0.25,
    "2014-07-09": 0.0,
    "2014-07-15": 0.0,
    "2014-07-29": 1.0,
    "2014-07-31": 0.0,
    "2014-08-04": 0.0
   }
  }
 }
}
```

</details>


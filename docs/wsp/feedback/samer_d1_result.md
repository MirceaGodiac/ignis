# samer: D1 revision (PROTOCOL 13.3), one look beside its W0 parent

- Dossier: `docs\wsp\research\phys_samer_d1.md`, sha256 `8b610f1d584b01ed4cfb781fd4a372a40d6682e17bd2e05fbd6fa7cc22ac3786`.
- D1 release: `sha256:f55c40440da6f9d759bf54db8ac7b732c21832ee08446ab47c1071e49141ce7a` (revision of the W0 release `sha256:05f77ae2f7a666e0da69d04fa29d9ad8b0ac15790645abe1adc0149292327fb0`; changed: {"('d10.median.skip', (3,))": [0.104, 0.324], "('d10.median.W1', (3, 3))": [-0.282, 0.268], "('d10.median.W1', (4, 3))": [-0.282, 0.268], "('d10.median.W1', (5, 3))": [-0.282, 0.268], "('d10.median.W1', (6, 3))": [-0.282, 0.268], "('d30.median.skip', (3,))": [0.037, 0.257], "('d30.median.W1', (3, 3))": [-0.282, 0.268], "('d30.median.W1', (4, 3))": [-0.282, 0.268], "('d30.median.W1', (5, 3))": [-0.282, 0.268], "('d30.median.W1', (6, 3))": [-0.282, 0.268], "('iss_full_scale.logit.skip', (3,))": [-0.47, 0.19], "('d10.median.W1', (3, 2))": [-1.436, -0.536], "('d10.median.W1', (4, 2))": [-1.436, -0.536], "('d10.median.W1', (5, 2))": [-1.436, -0.536], "('d10.median.W1', (6, 2))": [-1.436, -0.536], "('d10.median.skip', (2,))": [-0.225, -0.125], "('d30.median.W1', (3, 2))": [-1.436, -0.536], "('d30.median.W1', (4, 2))": [-1.436, -0.536], "('d30.median.W1', (5, 2))": [-1.436, -0.536], "('d30.median.W1', (6, 2))": [-1.436, -0.536], "('d30.median.skip', (2,))": [-0.225, -0.125], "('iss_full_scale.logit.skip', (2,))": [-1.57, -0.77]}; assembly check: every other number equals the parent).

## samer: release `sha256:f55c40440da6f9d759bf54db8ac7b732c21832ee08446ab47c1071e49141ce7a`, partition validation, look 13.3

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.


### target d10

- Asked of the release: 24; answered: 23; refused: 1; refusal reasons: {"feature net_aging_time": 1}
- Lane cases without a release forecast (not scored): 0

| source | CRPS of ln d10, case mean | day-equal | on |
|---|---|---|---|
| **release** | 0.1725 | 0.1561 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.3925 | 0.5756 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.3925 | 0.5756 | all lane cases |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.3392 | 0.4120 | cases the release answers |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.3392 | 0.4120 | all lane cases |

**Reading (CRPS of ln d10, case mean):** release 0.1725; simple baseline (B0 / I0 constant (simple, SAME-R train)) 0.3925; best baseline (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) 0.3392: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "aggregate": 0.17251108010539892,
  "day_equal": 0.15612034024074525,
  "n": 23,
  "per_day": {
   "PSI-101:day:2010-07-21": [
    0.11188639537479966,
    2
   ],
   "PSI-101:day:2010-09-07": [
    0.1779044841648857,
    9
   ],
   "PSI-101:day:2010-09-08": [
    0.17857014118255035,
    12
   ]
  }
 },
 "baselines_on_cases_answered": {
  "B0 / I0 constant (simple, SAME-R train)": {
   "aggregate": 0.39247917537690485,
   "day_equal": 0.5755557850303804,
   "n": 23,
   "per_day": {
    "PSI-101:day:2010-07-21": [
     1.0194220957084834,
     2
    ],
    "PSI-101:day:2010-09-07": [
     0.4995887567800154,
     9
    ],
    "PSI-101:day:2010-09-08": [
     0.2076565026026422,
     12
    ]
   }
  },
  "B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)": {
   "aggregate": 0.33920967243632566,
   "day_equal": 0.41202325888956537,
   "n": 23,
   "per_day": {
    "PSI-101:day:2010-07-21": [
     0.5966691465513361,
     2
    ],
    "PSI-101:day:2010-09-07": [
     0.35477446282516706,
     9
    ],
    "PSI-101:day:2010-09-08": [
     0.284626167292193,
     12
    ]
   }
  }
 }
}
```

</details>

### target iss

- Asked of the release: 24; answered: 23; refused: 1; refusal reasons: {"feature net_aging_time": 1}
- Lane cases without a release forecast (not scored): 0

| source | Brier, case mean | day-equal | on |
|---|---|---|---|
| **release** | 0.2436 | 0.1713 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.4112 | 0.4400 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.4112 | 0.4400 | all lane cases |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.2473 | 0.2060 | cases the release answers |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.2473 | 0.2060 | all lane cases |

**Reading (Brier, case mean):** release 0.2436; simple baseline (B0 / I0 constant (simple, SAME-R train)) 0.4112; best baseline (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) 0.2473: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "aggregate": 0.2435661530055562,
  "day_equal": 0.17134151807495504,
  "n": 23,
  "per_day": {
   "PSI-101:day:2010-07-21": [
    0.0010987941655917926,
    2
   ],
   "PSI-101:day:2010-09-07": [
    0.18509506330489048,
    9
   ],
   "PSI-101:day:2010-09-08": [
    0.32783069675438287,
    12
   ]
  }
 },
 "baselines_on_cases_answered": {
  "B0 / I0 constant (simple, SAME-R train)": {
   "aggregate": 0.4111564009661836,
   "day_equal": 0.44000771604938277,
   "n": 23,
   "per_day": {
    "PSI-101:day:2010-07-21": [
     0.5017361111111112,
     2
    ],
    "PSI-101:day:2010-09-07": [
     0.4554398148148149,
     9
    ],
    "PSI-101:day:2010-09-08": [
     0.36284722222222227,
     12
    ]
   }
  },
  "B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)": {
   "aggregate": 0.24728260869565216,
   "day_equal": 0.20601851851851852,
   "n": 23,
   "per_day": {
    "PSI-101:day:2010-07-21": [
     0.0625,
     2
    ],
    "PSI-101:day:2010-09-07": [
     0.3680555555555556,
     9
    ],
    "PSI-101:day:2010-09-08": [
     0.1875,
     12
    ]
   }
  }
 }
}
```

</details>


## Summary: train (study data) and validation, W0 against D1

| target, metric | W0 train | D1 train | W0 validation | D1 validation | simple baseline | best baseline (validation) |
|---|---|---|---|---|---|---|
| d10: CRPS of ln d10, case mean | 0.2722 | 0.2158 | 0.1672 | 0.1725 | 0.3925 | 0.3392 (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) |
| iss: Brier, case mean | 0.1501 | 0.1005 | 0.2279 | 0.2436 | 0.4112 | 0.2473 (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) |

**Reading (13.3):** target that lost: d10: CRPS of ln d10, case mean: W0 0.1672, D1 0.1725 (not lower); other scored target(s): iss: Brier, case mean: W0 0.2279, D1 0.2436 (more than 5 % of the parent). **the parent (W0) stays.**


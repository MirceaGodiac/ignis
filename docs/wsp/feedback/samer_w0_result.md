# samer: seventh-generation W0 law, scores beside the baselines

## samer: release `sha256:05f77ae2f7a666e0da69d04fa29d9ad8b0ac15790645abe1adc0149292327fb0`, partition validation, look 13.2

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.


### target d10

- Asked of the release: 24; answered: 23; refused: 1; refusal reasons: {"feature net_aging_time": 1}
- Lane cases without a release forecast (not scored): 0

| source | CRPS of ln d10, case mean | day-equal | on |
|---|---|---|---|
| **release** | 0.1672 | 0.1512 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.3925 | 0.5756 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.3925 | 0.5756 | all lane cases |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.3392 | 0.4120 | cases the release answers |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.3392 | 0.4120 | all lane cases |

**Reading (CRPS of ln d10, case mean):** release 0.1672; simple baseline (B0 / I0 constant (simple, SAME-R train)) 0.3925; best baseline (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) 0.3392: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "aggregate": 0.16715630602130985,
  "day_equal": 0.15122921259331193,
  "n": 23,
  "per_day": {
   "PSI-101:day:2010-07-21": [
    0.11188639537479966,
    2
   ],
   "PSI-101:day:2010-09-07": [
    0.1602642203737023,
    9
   ],
   "PSI-101:day:2010-09-08": [
    0.18153702203143385,
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
| **release** | 0.2279 | 0.1605 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.4112 | 0.4400 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.4112 | 0.4400 | all lane cases |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.2473 | 0.2060 | cases the release answers |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.2473 | 0.2060 | all lane cases |

**Reading (Brier, case mean):** release 0.2279; simple baseline (B0 / I0 constant (simple, SAME-R train)) 0.4112; best baseline (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) 0.2473: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "aggregate": 0.22785549464487015,
  "day_equal": 0.16050633443058288,
  "n": 23,
  "per_day": {
   "PSI-101:day:2010-07-21": [
    0.0010987941655917926,
    2
   ],
   "PSI-101:day:2010-09-07": [
    0.17552124033768407,
    9
   ],
   "PSI-101:day:2010-09-08": [
    0.3048989687884728,
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


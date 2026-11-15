# same: seventh-generation W0 law, scores beside the baselines

## same: release `sha256:e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062`, partition held, look 13.2

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.


### target d10

- Asked of the release: 45; answered: 45; refused: 0; refusal reasons: {}
- Lane cases without a release forecast (not scored): 0

| source | CRPS of ln d10, case mean | day-equal | on |
|---|---|---|---|
| **release** | 0.1844 | 0.1844 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.2197 | 0.2197 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.2197 | 0.2197 | all lane cases |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.1921 | 0.1921 | cases the release answers |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.1921 | 0.1921 | all lane cases |

**Reading (CRPS of ln d10, case mean):** release 0.1844; simple baseline (B0 / I0 constant (simple, SAME-R train)) 0.2197; best baseline (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) 0.1921: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "aggregate": 0.1843918566009345,
  "day_equal": 0.1843918566009345,
  "n": 25,
  "per_day": {
   "PSI-102:matrix": [
    0.1843918566009345,
    25
   ]
  }
 },
 "baselines_on_cases_answered": {
  "B0 / I0 constant (simple, SAME-R train)": {
   "aggregate": 0.2196873530144762,
   "day_equal": 0.2196873530144762,
   "n": 25,
   "per_day": {
    "PSI-102:matrix": [
     0.2196873530144762,
     25
    ]
   }
  },
  "B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)": {
   "aggregate": 0.1921383353231933,
   "day_equal": 0.1921383353231933,
   "n": 25,
   "per_day": {
    "PSI-102:matrix": [
     0.1921383353231933,
     25
    ]
   }
  }
 }
}
```

</details>

### target iss

- Asked of the release: 45; answered: 45; refused: 0; refusal reasons: {}
- Lane cases without a release forecast (not scored): 0

| source | Brier, case mean | day-equal | on |
|---|---|---|---|
| **release** | 0.0591 | 0.0591 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.2054 | 0.2054 | cases the release answers |
| B0 / I0 constant (simple, SAME-R train) | 0.2054 | 0.2054 | all lane cases |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.0554 | 0.0554 | cases the release answers |
| B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | 0.0554 | 0.0554 | all lane cases |

**Reading (Brier, case mean):** release 0.0591; simple baseline (B0 / I0 constant (simple, SAME-R train)) 0.2054; best baseline (B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)) 0.0554: **beats the simple baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "aggregate": 0.059085374108460986,
  "day_equal": 0.059085374108460986,
  "n": 45,
  "per_day": {
   "PSI-102:matrix": [
    0.059085374108460986,
    45
   ]
  }
 },
 "baselines_on_cases_answered": {
  "B0 / I0 constant (simple, SAME-R train)": {
   "aggregate": 0.2054398148148148,
   "day_equal": 0.2054398148148148,
   "n": 45,
   "per_day": {
    "PSI-102:matrix": [
     0.2054398148148148,
     45
    ]
   }
  },
  "B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0)": {
   "aggregate": 0.055401234567901234,
   "day_equal": 0.055401234567901234,
   "n": 45,
   "per_day": {
    "PSI-102:matrix": [
     0.055401234567901234,
     45
    ]
   }
  }
 }
}
```

</details>


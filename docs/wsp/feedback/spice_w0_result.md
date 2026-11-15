# spice: seventh-generation W0 law, scores beside the baselines

## spice: release `sha256:93e750242725a5a1fb9d38f4178b23605cac636f210bf9f3f8bc8e8150e2f2a8`, partition validation, look 13.2

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.


### target T1

- Asked of the release: 7; answered: 7; refused: 0; refusal reasons: {}
- Lane cases without a release forecast (not scored): 0

| source | quantile score x2, mm, mean of day means | pooled mean | on |
|---|---|---|---|
| **release** | 5.8621 | 5.9432 | cases the release answers |
| B0 pooled climatology (simple) | 6.0990 | 6.1253 | cases the release answers |
| B0 pooled climatology (simple) | 6.0990 | 6.1253 | all lane cases |
| B1 per-fuel climatology | 5.0419 | 4.7406 | cases the release answers |
| B1 per-fuel climatology | 5.0419 | 4.7406 | all lane cases |

**Reading (quantile score x2, mm, mean of day means):** release 5.8621; simple baseline (B0 pooled climatology (simple)) 6.0990; best baseline (B1 per-fuel climatology) 5.0419: **beats the simple baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "target": "L_sp_mm",
  "aggregate_day_mean": 5.862052611602033,
  "pooled_mean": 5.943248626616011,
  "n": 7,
  "per_day": {
   "136": 6.430424716699881,
   "173": 5.293680506504185
  }
 },
 "baselines_on_cases_answered": {
  "B0 pooled climatology (simple)": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 6.099042651700975,
   "pooled_mean": 6.125321056592536,
   "n": 7,
   "per_day": {
    "136": 6.282991485941896,
    "173": 5.915093817460055
   }
  },
  "B1 per-fuel climatology": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 5.04189419424366,
   "pooled_mean": 4.740555022245409,
   "n": 7,
   "per_day": {
    "136": 2.9325199902558934,
    "173": 7.151268398231427
   }
  }
 }
}
```

</details>

### target T2

- Asked of the release: 7; answered: 7; refused: 0; refusal reasons: {}
- Lane cases without a release forecast (not scored): 0

| source | quantile score x2, mm, mean of day means | pooled mean | on |
|---|---|---|---|
| **release** | 2.5329 | 2.5431 | cases the release answers |
| B0 pooled climatology (simple) | 6.0990 | 6.1253 | cases the release answers |
| B0 pooled climatology (simple) | 6.0990 | 6.1253 | all lane cases |
| B1 per-fuel climatology | 5.0419 | 4.7406 | cases the release answers |
| B1 per-fuel climatology | 5.0419 | 4.7406 | all lane cases |
| Lin and Faeth coflow law, train sigma (T2 only) | 1.2606 | 1.2349 | cases the release answers |
| Lin and Faeth coflow law, train sigma (T2 only) | 1.2606 | 1.2349 | all lane cases |

**Reading (quantile score x2, mm, mean of day means):** release 2.5329; simple baseline (B0 pooled climatology (simple)) 6.0990; best baseline (Lin and Faeth coflow law, train sigma (T2 only)) 1.2606: **beats the simple baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "target": "L_sp_mm",
  "aggregate_day_mean": 2.532900177944975,
  "pooled_mean": 2.543140056253393,
  "n": 7,
  "per_day": {
   "136": 2.6045793261039005,
   "173": 2.4612210297860493
  }
 },
 "baselines_on_cases_answered": {
  "B0 pooled climatology (simple)": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 6.099042651700975,
   "pooled_mean": 6.125321056592536,
   "n": 7,
   "per_day": {
    "136": 6.282991485941896,
    "173": 5.915093817460055
   }
  },
  "B1 per-fuel climatology": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 5.04189419424366,
   "pooled_mean": 4.740555022245409,
   "n": 7,
   "per_day": {
    "136": 2.9325199902558934,
    "173": 7.151268398231427
   }
  },
  "Lin and Faeth coflow law, train sigma (T2 only)": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 1.260557014856125,
   "pooled_mean": 1.23493941131053,
   "n": 7,
   "per_day": {
    "136": 1.0812337900369628,
    "173": 1.439880239675287
   }
  }
 }
}
```

</details>

## spice: release `sha256:40c345b0453da40b38e85bd7c7441e9dc1e272ce097cf204f7bb3f4c3e35d3bf`, partition validation, look 13.2

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.


### target T1

- Asked of the release: 7; answered: 7; refused: 0; refusal reasons: {}
- Lane cases without a release forecast (not scored): 0

| source | quantile score x2, mm, mean of day means | pooled mean | on |
|---|---|---|---|
| **release** | 2.9140 | 2.9210 | cases the release answers |
| B0 pooled climatology (simple) | 6.0990 | 6.1253 | cases the release answers |
| B0 pooled climatology (simple) | 6.0990 | 6.1253 | all lane cases |
| B1 per-fuel climatology | 5.0419 | 4.7406 | cases the release answers |
| B1 per-fuel climatology | 5.0419 | 4.7406 | all lane cases |

**Reading (quantile score x2, mm, mean of day means):** release 2.9140; simple baseline (B0 pooled climatology (simple)) 6.0990; best baseline (B1 per-fuel climatology) 5.0419: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "target": "L_sp_mm",
  "aggregate_day_mean": 2.9139855316574157,
  "pooled_mean": 2.920978875220832,
  "n": 7,
  "per_day": {
   "136": 2.962938936601332,
   "173": 2.8650321267134995
  }
 },
 "baselines_on_cases_answered": {
  "B0 pooled climatology (simple)": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 6.099042651700975,
   "pooled_mean": 6.125321056592536,
   "n": 7,
   "per_day": {
    "136": 6.282991485941896,
    "173": 5.915093817460055
   }
  },
  "B1 per-fuel climatology": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 5.04189419424366,
   "pooled_mean": 4.740555022245409,
   "n": 7,
   "per_day": {
    "136": 2.9325199902558934,
    "173": 7.151268398231427
   }
  }
 }
}
```

</details>

### target T2

- Asked of the release: 7; answered: 7; refused: 0; refusal reasons: {}
- Lane cases without a release forecast (not scored): 0

| source | quantile score x2, mm, mean of day means | pooled mean | on |
|---|---|---|---|
| **release** | 2.5329 | 2.5431 | cases the release answers |
| B0 pooled climatology (simple) | 6.0990 | 6.1253 | cases the release answers |
| B0 pooled climatology (simple) | 6.0990 | 6.1253 | all lane cases |
| B1 per-fuel climatology | 5.0419 | 4.7406 | cases the release answers |
| B1 per-fuel climatology | 5.0419 | 4.7406 | all lane cases |
| Lin and Faeth coflow law, train sigma (T2 only) | 1.2606 | 1.2349 | cases the release answers |
| Lin and Faeth coflow law, train sigma (T2 only) | 1.2606 | 1.2349 | all lane cases |

**Reading (quantile score x2, mm, mean of day means):** release 2.5329; simple baseline (B0 pooled climatology (simple)) 6.0990; best baseline (Lin and Faeth coflow law, train sigma (T2 only)) 1.2606: **beats the simple baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "target": "L_sp_mm",
  "aggregate_day_mean": 2.532900177944975,
  "pooled_mean": 2.543140056253393,
  "n": 7,
  "per_day": {
   "136": 2.6045793261039005,
   "173": 2.4612210297860493
  }
 },
 "baselines_on_cases_answered": {
  "B0 pooled climatology (simple)": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 6.099042651700975,
   "pooled_mean": 6.125321056592536,
   "n": 7,
   "per_day": {
    "136": 6.282991485941896,
    "173": 5.915093817460055
   }
  },
  "B1 per-fuel climatology": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 5.04189419424366,
   "pooled_mean": 4.740555022245409,
   "n": 7,
   "per_day": {
    "136": 2.9325199902558934,
    "173": 7.151268398231427
   }
  },
  "Lin and Faeth coflow law, train sigma (T2 only)": {
   "target": "L_sp_mm",
   "aggregate_day_mean": 1.260557014856125,
   "pooled_mean": 1.23493941131053,
   "n": 7,
   "per_day": {
    "136": 1.0812337900369628,
    "173": 1.439880239675287
   }
  }
 }
}
```

</details>


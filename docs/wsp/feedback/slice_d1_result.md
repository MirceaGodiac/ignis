# slice: D1 revision (PROTOCOL 13.3), one look beside its W0 parent

- Dossier: `docs\wsp\research\phys_slice_d1.md`, sha256 `adad2fbff877db6e6a5eaf6881ba34e1622d44446afb0d4721dfa57a619d648a`.
- D1 release: `sha256:c40cd91b76476c153c7b677adf9adca3feb244ae582f4b361307212d23f6b0d8` (revision of the W0 release `sha256:d10c94bc6ea0bcb957f89734393b0cbabbd44fdf0ac9c2e440f051a33cd3006e`; changed: {"('lifted.logit.W1', (0, 3))": [-1.3, 0.0], "('lifted.logit.W1', (1, 3))": [-1.3, 0.0], "('lifted.logit.W1', (2, 3))": [-1.3, 0.0], "('blow_off.logit.W1', (0, 3))": [-1.3, 0.0], "('blow_off.logit.W1', (1, 3))": [-1.3, 0.0], "('blow_off.logit.W1', (2, 3))": [-1.3, 0.0]}; assembly check: every other number equals the parent).

## slice: release `sha256:c40cd91b76476c153c7b677adf9adca3feb244ae582f4b361307212d23f6b0d8`, partition validation, look 13.3

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.

- Cabin oxygen supplied to validation lines without one (PROTOCOL 13.2): the median of the 275 train lines that record it, x_o2 = 0.219122.
- Lines given the supplied oxygen: 75.

### target blow_off

- Asked of the release: 75; answered: 34; refused: 41; refusal reasons: {"feature x_fuel": 41, "feature is_ethane": 41, "feature is_propane": 41}
- Lane cases without a release forecast (not scored): 40

| source | Brier, mean of day means | pooled | on |
|---|---|---|---|
| **release** | 0.1631 | 0.1631 | cases the release answers |
| incidence pooled (simple) | 0.1605 | 0.1605 | cases the release answers |
| incidence pooled (simple) | 0.1152 | 0.1115 | all lane cases |
| incidence by nominal tube | 0.1493 | 0.1493 | cases the release answers |
| incidence by nominal tube | 0.1083 | 0.1050 | all lane cases |
| never (persistence) | 0.1765 | 0.1765 | cases the release answers |
| never (persistence) | 0.1257 | 0.1216 | all lane cases |

**Reading (Brier, mean of day means):** release 0.1631; simple baseline (incidence pooled (simple)) 0.1605; best baseline (incidence by nominal tube) 0.1493: **loses (higher than the simple baseline)**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "target": "blow_off",
  "aggregate_day_mean": 0.16312588189743407,
  "pooled_mean": 0.16312588189743407,
  "n": 34,
  "per_day": {
   "065": 0.16312588189743407
  }
 },
 "baselines_on_cases_answered": {
  "incidence pooled (simple)": {
   "target": "blow_off",
   "aggregate_day_mean": 0.16052645594226825,
   "pooled_mean": 0.16052645594226825,
   "n": 34,
   "per_day": {
    "065": 0.16052645594226825
   }
  },
  "incidence by nominal tube": {
   "target": "blow_off",
   "aggregate_day_mean": 0.14927986380271396,
   "pooled_mean": 0.14927986380271396,
   "n": 34,
   "per_day": {
    "065": 0.14927986380271396
   }
  },
  "never (persistence)": {
   "target": "blow_off",
   "aggregate_day_mean": 0.17647058823529413,
   "pooled_mean": 0.17647058823529413,
   "n": 34,
   "per_day": {
    "065": 0.17647058823529413
   }
  }
 }
}
```

</details>

### target lifted

- Asked of the release: 75; answered: 34; refused: 41; refusal reasons: {"feature x_fuel": 41, "feature is_ethane": 41, "feature is_propane": 41}
- Lane cases without a release forecast (not scored): 40

| source | Brier, mean of day means | pooled | on |
|---|---|---|---|
| **release** | 0.0555 | 0.0555 | cases the release answers |
| incidence pooled (simple) | 0.0563 | 0.0563 | cases the release answers |
| incidence pooled (simple) | 0.0285 | 0.0263 | all lane cases |
| incidence by nominal tube | 0.0596 | 0.0596 | cases the release answers |
| incidence by nominal tube | 0.0299 | 0.0274 | all lane cases |
| never (persistence) | 0.0588 | 0.0588 | cases the release answers |
| never (persistence) | 0.0294 | 0.0270 | all lane cases |

**Reading (Brier, mean of day means):** release 0.0555; simple baseline (incidence pooled (simple)) 0.0563; best baseline (incidence pooled (simple)) 0.0563: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "target": "lifted",
  "aggregate_day_mean": 0.05545871241574941,
  "pooled_mean": 0.05545871241574941,
  "n": 34,
  "per_day": {
   "065": 0.05545871241574941
  }
 },
 "baselines_on_cases_answered": {
  "incidence pooled (simple)": {
   "target": "lifted",
   "aggregate_day_mean": 0.05629081268843737,
   "pooled_mean": 0.05629081268843737,
   "n": 34,
   "per_day": {
    "065": 0.05629081268843737
   }
  },
  "incidence by nominal tube": {
   "target": "lifted",
   "aggregate_day_mean": 0.05964060817576504,
   "pooled_mean": 0.05964060817576504,
   "n": 34,
   "per_day": {
    "065": 0.05964060817576504
   }
  },
  "never (persistence)": {
   "target": "lifted",
   "aggregate_day_mean": 0.058823529411764705,
   "pooled_mean": 0.058823529411764705,
   "n": 34,
   "per_day": {
    "065": 0.058823529411764705
   }
  }
 }
}
```

</details>


## Summary: train (study data) and validation, W0 against D1

| target, metric | W0 train | D1 train | W0 validation | D1 validation | simple baseline | best baseline (validation) |
|---|---|---|---|---|---|---|
| blow_off: Brier, mean of day means | 0.1023 | 0.0601 | 0.1548 | 0.1631 | 0.1605 | 0.1493 (incidence by nominal tube) |
| lifted: Brier, mean of day means | 0.1559 | 0.1490 | 0.0886 | 0.0555 | 0.0563 | 0.0563 (incidence pooled (simple)) |

**Reading (13.3):** target that lost: lifted: Brier, mean of day means: W0 0.0886, D1 0.0555 (lower); other scored target(s): blow_off: Brier, mean of day means: W0 0.1548, D1 0.1631 (more than 5 % of the parent). **the parent (W0) stays.**


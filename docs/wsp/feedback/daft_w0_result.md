# daft: seventh-generation W0 law, scores beside the baselines

## daft: release `sha256:061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d`, partition held, look 13.2

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.


### target number_concentration

- Asked of the release: 53; answered: 53; refused: 0; refusal reasons: {}
- Lane cases without a release forecast (not scored): 0

| source | CRPS of ln(1 + N per cm3), case mean | group-equal | on |
|---|---|---|---|
| **release** | 0.7772 | 1.3725 | cases the release answers |
| smoke number per mass, SAME-R train (simple) | 2.6778 | 2.2342 | cases the release answers |
| smoke number per mass, SAME-R train (simple) | 2.6778 | 2.2342 | all lane cases |

**Reading (CRPS of ln(1 + N per cm3), case mean):** release 0.7772; simple baseline (smoke number per mass, SAME-R train (simple)) 2.6778; best baseline (smoke number per mass, SAME-R train (simple)) 2.6778: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "aggregate": 0.7771944189011255,
  "group_equal": 1.372499627853698,
  "n": 53,
  "per_group": {
   "PSI-47:session:D2": [
    2.3902795012242253,
    11
   ],
   "PSI-47:2006-08-21:bags": [
    0.35471975448317067,
    42
   ]
  }
 },
 "baselines_on_cases_answered": {
  "smoke number per mass, SAME-R train (simple)": {
   "aggregate": 2.677845933855966,
   "group_equal": 2.234151924921865,
   "n": 53,
   "per_group": {
    "PSI-47:session:D2": [
     1.4755782967442082,
     11
    ],
    "PSI-47:2006-08-21:bags": [
     2.992725553099522,
     42
    ]
   }
  }
 }
}
```

</details>


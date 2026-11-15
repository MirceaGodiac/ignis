# bre: D1 revision (PROTOCOL 13.3), one look beside its W0 parent

- Dossier: `docs\wsp\research\phys_bre_d1.md`, sha256 `abca2b07b070fa9f58051e96b3ad12b8449d7d2cc76f538dacfb92bb7838e648`.
- D1 release: `sha256:3cffa05ad9dcd29e1fc5b422474e30e2c07c6d5909ea944b77a12d9c695c0f76` (revision of the W0 release `sha256:aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d`; changed: {"('flame_end.extinction.median.skip', (3,))": [0.28, -0.15], "('flame_end.extinction.median.W1', (0, 3))": [-0.27, -0.43], "('flame_end.extinction.median.W1', (1, 3))": [-0.27, -0.43], "('flame_end.extinction.median.W1', (2, 3))": [-0.27, -0.43], "('flame_end.extinction.median.W1', (3, 3))": [-0.27, -0.43], "('flame_end.extinction.median.W1', (4, 3))": [-0.27, -0.43], "('flame_end.extinction.median.W1', (5, 3))": [0.5, 0.8], "('flame_end.extinction.median.W1', (6, 3))": [0.5, 0.8], "('lasts.logit.skip', (3,))": [-1.08, -1.72], "('lasts.logit.W1', (0, 3))": [0.46, 0.13], "('lasts.logit.W1', (1, 3))": [0.46, 0.13], "('lasts.logit.W1', (2, 3))": [0.46, 0.13], "('lasts.logit.W1', (3, 3))": [0.46, 0.13], "('lasts.logit.W1', (4, 3))": [0.46, 0.13], "('lasts.logit.W1', (5, 3))": [0.46, 0.13], "('lasts.logit.W1', (6, 3))": [0.5, 0.8], "('lasts.logit.W1', (7, 3))": [0.5, 0.8], "('height_at_end.median.skip', (3,))": [-0.27, -0.43]}; assembly check: every other number equals the parent).

## bre: release `sha256:3cffa05ad9dcd29e1fc5b422474e30e2c07c6d5909ea944b77a12d9c695c0f76`, partition validation, look 13.3

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.


### target t1

- Asked of the release: 90; answered: 87; refused: 3; refusal reasons: {"feature x_fuel": 3}
- Lane cases without a release forecast (not scored): 0

| source | extinction IBS | operator termination IBS | mean of the two causes | on |
|---|---|---|---|---|
| **release** | 0.2866 | 0.1403 | 0.2134 | cases the release answers |
| incidence (simple) | 0.2274 | 0.1799 | 0.2036 | cases the release answers |
| incidence (simple) | 0.2290 | 0.1746 | 0.2018 | all lane cases |
| planned stop | 0.2129 | 0.0924 | 0.1526 | cases the release answers |
| planned stop | 0.2204 | 0.1002 | 0.1603 | all lane cases |

**Reading (extinction IBS):** release 0.2866; simple baseline (incidence (simple)) 0.2274; best baseline (planned stop) 0.2129: **loses (higher than the simple baseline)**.

**Reading (operator termination IBS):** release 0.1403; simple baseline (incidence (simple)) 0.1799; best baseline (planned stop) 0.0924: **beats the simple baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "n_cases": 87,
  "horizons_s": [
   6.0,
   8.0,
   10.0,
   12.0,
   15.0,
   20.0,
   25.0,
   30.0,
   40.0,
   50.0,
   60.0,
   80.0,
   100.0,
   120.0,
   150.0,
   180.0,
   210.0
  ],
  "causes": {
   "extinction": {
    "integrated_brier": 0.2866001411602688,
    "session_mean": 0.2543979750920223,
    "brier_per_horizon": [
     0.043859935701805755,
     0.06824636748991078,
     0.08653266893088299,
     0.10002362382810677,
     0.11500430736406637,
     0.13423091946429203,
     0.18078071055478692,
     0.21658291806080604,
     0.2607200827283483,
     0.24323270476852146,
     0.2160372487935901,
     0.2910517832582166,
     0.3208152544004029,
     0.33035220785692765,
     0.3437221127751105,
     0.33604279352141325,
     0.3356216478806419
    ],
    "per_day": {
     "19051": 0.19437976613160407,
     "19063": 0.08258045910464089,
     "20346": 0.2609178912799738,
     "20352": 0.47971378385187036
    }
   },
   "operator_termination": {
    "integrated_brier": 0.14026458142467668,
    "session_mean": 0.11011162855026453,
    "brier_per_horizon": [
     0.0,
     0.0,
     0.0,
     0.0,
     0.0,
     0.0,
     0.0,
     0.0,
     0.0,
     1.29501545192071e-06,
     0.03410144268977069,
     0.05235223871239125,
     0.0738405854036122,
     0.0738405854036122,
     0.30829305083657516,
     0.30829305083657516,
     0.34899905683520543
    ],
    "per_day": {
     "19051": 0.017223502403463563,
     "19063": 0.00013641191948168805,
     "20346": 0.17309389079788187,
     "20352": 0.249992709080231
    }
   }
  },
  "integrated_brier_mean_of_causes": 0.21343236129247273
 },
 "baselines_on_cases_answered": {
  "incidence (simple)": {
   "n_cases": 87,
   "horizons_s": [
    6.0,
    8.0,
    10.0,
    12.0,
    15.0,
    20.0,
    25.0,
    30.0,
    40.0,
    50.0,
    60.0,
    80.0,
    100.0,
    120.0,
    150.0,
    180.0,
    210.0
   ],
   "causes": {
    "extinction": {
     "integrated_brier": 0.2273617843002479,
     "session_mean": 0.21960124797952751,
     "brier_per_horizon": [
      0.002659279778393352,
      0.004895937409620837,
      0.008060203932112155,
      0.008060203932112155,
      0.014545870386968826,
      0.04204467873715545,
      0.08840003217849432,
      0.11825046766082424,
      0.1708519041718072,
      0.20913006964888214,
      0.22299977971647594,
      0.25641644952956194,
      0.2717796546368693,
      0.2717796546368693,
      0.2678595962718048,
      0.27103952466705766,
      0.27103952466705766
     ],
     "per_day": {
      "19051": 0.19956675001581084,
      "19063": 0.18778059261967678,
      "20346": 0.2372361879861498,
      "20352": 0.25382146129647265
     }
    },
    "operator_termination": {
     "integrated_brier": 0.17988576570675763,
     "session_mean": 0.1413883319582152,
     "brier_per_horizon": [
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.03385644337266942,
      0.09639629733145812,
      0.19093737688038856,
      0.18752733872649113,
      0.3614733288530573,
      0.3614733288530573,
      0.27103952466705766
     ],
     "per_day": {
      "19051": 0.02524858017735304,
      "19063": 0.022255224348918766,
      "20346": 0.31672247310531076,
      "20352": 0.20132705020127825
     }
    }
   },
   "integrated_brier_mean_of_causes": 0.20362377500350276
  },
  "planned stop": {
   "n_cases": 87,
   "horizons_s": [
    6.0,
    8.0,
    10.0,
    12.0,
    15.0,
    20.0,
    25.0,
    30.0,
    40.0,
    50.0,
    60.0,
    80.0,
    100.0,
    120.0,
    150.0,
    180.0,
    210.0
   ],
   "causes": {
    "extinction": {
     "integrated_brier": 0.21286799008545218,
     "session_mean": 0.20889312027362975,
     "brier_per_horizon": [
      0.002659279778393352,
      0.004895937409620837,
      0.008060203932112155,
      0.008060203932112155,
      0.014545870386968826,
      0.04204467873715545,
      0.08840003217849432,
      0.11825046766082424,
      0.1708519041718072,
      0.20736333862562883,
      0.22133042831295563,
      0.2500189259085345,
      0.25670840846703447,
      0.25670840846703447,
      0.24284547995320444,
      0.24087404913568292,
      0.24087404913568292
     ],
     "per_day": {
      "19051": 0.19956675001581084,
      "19063": 0.18778059261967678,
      "20346": 0.20240875316939283,
      "20352": 0.24581638528963853
     }
    },
    "operator_termination": {
     "integrated_brier": 0.09236084689816874,
     "session_mean": 0.07296312502655283,
     "brier_per_horizon": [
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.0,
      0.013918131683143996,
      0.015854432904143238,
      0.029487030736672512,
      0.06511569298643766,
      0.06525963229715101,
      0.19153616169242382,
      0.19153616169242382,
      0.2299266648930951
     ],
     "per_day": {
      "19051": 0.014665199322369834,
      "19063": 0.007003106096361337,
      "20346": 0.1384652807659166,
      "20352": 0.13171891392156354
     }
    }
   },
   "integrated_brier_mean_of_causes": 0.15261441849181046
  }
 }
}
```

</details>


## Summary: train (study data) and validation, W0 against D1

| target, metric | W0 train | D1 train | W0 validation | D1 validation | simple baseline | best baseline (validation) |
|---|---|---|---|---|---|---|
| t1: extinction IBS | 0.2511 | 0.2174 | 0.2755 | 0.2866 | 0.2274 | 0.2129 (planned stop) |
| t1: operator termination IBS | 0.0494 | 0.0430 | 0.1383 | 0.1403 | 0.1799 | 0.0924 (planned stop) |

**Reading (13.3):** target that lost: t1: extinction IBS: W0 0.2755, D1 0.2866 (not lower); other scored target(s): t1: operator termination IBS: W0 0.1383, D1 0.1403 (within 5 % of the parent). **the parent (W0) stays.**


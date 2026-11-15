# efield: seventh-generation W0 law, scores beside the baselines

## efield: release `sha256:73f91b55c81dc724fe2441c2254e109c65f8e08b253fcbe4c00a841f7a0986a9`, partition validation, look 13.2

Written by `tools/wsp/ext/law_score.py look`; every score is the lane's own `score()`. Lower is better.


### target t1

- Asked of the release: 1289; answered: 1289; refused: 0; refusal reasons: {}
- Lane cases without a release forecast (not scored): 1

| source | CRPS uA, case mean | mean of day means | on |
|---|---|---|---|
| **release** | 0.5463 | 0.5320 | cases the release answers |
| constant (simple) | 0.6202 | 0.7383 | cases the release answers |
| constant (simple) | 0.6202 | 0.7383 | all lane cases |
| space-charge law up to saturation, train sigma (T1) | 0.3742 | 0.4382 | cases the release answers |
| space-charge law up to saturation, train sigma (T1) | 0.3742 | 0.4382 | all lane cases |

**Reading (CRPS uA, case mean):** release 0.5463; simple baseline (constant (simple)) 0.6202; best baseline (space-charge law up to saturation, train sigma (T1)) 0.3742: **beats the simple baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "t1_crps_ua": 0.5463200474652506,
  "t1_cases": 1135,
  "t1_crps_ua_mean_of_days": 0.531987377614351,
  "days": 4,
  "per_day": {
   "18073": {
    "t1_crps_ua": 0.6536798985390585,
    "t1_cases": 497,
    "t2_integrated_brier": NaN,
    "t2_end_in_window_brier": NaN,
    "t2_tests": 0
   },
   "18078": {
    "t1_crps_ua": 0.3519209716521634,
    "t1_cases": 366,
    "t2_integrated_brier": NaN,
    "t2_end_in_window_brier": NaN,
    "t2_tests": 0
   },
   "18285": {
    "t1_crps_ua": 0.9230652042976911,
    "t1_cases": 155,
    "t2_integrated_brier": NaN,
    "t2_end_in_window_brier": NaN,
    "t2_tests": 0
   },
   "18298": {
    "t1_crps_ua": 0.19928343596849085,
    "t1_cases": 117,
    "t2_integrated_brier": NaN,
    "t2_end_in_window_brier": NaN,
    "t2_tests": 0
   }
  }
 },
 "baselines_on_cases_answered": {
  "constant (simple)": {
   "t1_crps_ua": 0.6202038340969503,
   "t1_cases": 1135,
   "t1_crps_ua_mean_of_days": 0.7383269487821775,
   "days": 4,
   "per_day": {
    "18073": {
     "t1_crps_ua": 0.5435418944959107,
     "t1_cases": 497,
     "t2_integrated_brier": NaN,
     "t2_end_in_window_brier": NaN,
     "t2_tests": 0
    },
    "18078": {
     "t1_crps_ua": 0.33143290171081263,
     "t1_cases": 366,
     "t2_integrated_brier": NaN,
     "t2_end_in_window_brier": NaN,
     "t2_tests": 0
    },
    "18285": {
     "t1_crps_ua": 1.8242533483037136,
     "t1_cases": 155,
     "t2_integrated_brier": NaN,
     "t2_end_in_window_brier": NaN,
     "t2_tests": 0
    },
    "18298": {
     "t1_crps_ua": 0.2540796506182732,
     "t1_cases": 117,
     "t2_integrated_brier": NaN,
     "t2_end_in_window_brier": NaN,
     "t2_tests": 0
    }
   }
  },
  "space-charge law up to saturation, train sigma (T1)": {
   "t1_crps_ua": 0.3741927495004688,
   "t1_cases": 1135,
   "t1_crps_ua_mean_of_days": 0.43817904566883425,
   "days": 4,
   "per_day": {
    "18073": {
     "t1_crps_ua": 0.3294590435537601,
     "t1_cases": 497,
     "t2_integrated_brier": NaN,
     "t2_end_in_window_brier": NaN,
     "t2_tests": 0
    },
    "18078": {
     "t1_crps_ua": 0.23864234089363875,
     "t1_cases": 366,
     "t2_integrated_brier": NaN,
     "t2_end_in_window_brier": NaN,
     "t2_tests": 0
    },
    "18285": {
     "t1_crps_ua": 0.9216999441334942,
     "t1_cases": 155,
     "t2_integrated_brier": NaN,
     "t2_end_in_window_brier": NaN,
     "t2_tests": 0
    },
    "18298": {
     "t1_crps_ua": 0.2629148540944439,
     "t1_cases": 117,
     "t2_integrated_brier": NaN,
     "t2_end_in_window_brier": NaN,
     "t2_tests": 0
    }
   }
  }
 }
}
```

</details>

### target t2

- Asked of the release: 36; answered: 36; refused: 0; refusal reasons: {}
- Lane cases without a release forecast (not scored): 1

| source | integrated Brier | Brier of ends in window | on |
|---|---|---|---|
| **release** | 0.0753 | 0.0762 | cases the release answers |
| constant (simple) | 0.0884 | 0.1120 | cases the release answers |
| constant (simple) | 0.0884 | 0.1120 | all lane cases |
| stratified incidence (T2; a stratum absent from train takes the pooled incidence) | 0.1124 | 0.1456 | cases the release answers |
| stratified incidence (T2; a stratum absent from train takes the pooled incidence) | 0.1124 | 0.1456 | all lane cases |

**Reading (integrated Brier):** release 0.0753; simple baseline (constant (simple)) 0.0884; best baseline (constant (simple)) 0.0884: **beats the best baseline**.

<details><summary>lane results (aggregates and per-day means)</summary>

```json
{
 "release": {
  "t2_integrated_brier": 0.07530488654968162,
  "t2_horizons_used": [
   2,
   5,
   10,
   15,
   20,
   30,
   45,
   60,
   90
  ],
  "t2_brier_by_horizon": {
   "2": 1.0700696623448264e-07,
   "5": 1.8636542332243292e-06,
   "10": 1.9759723228341427e-05,
   "15": 0.02833291124658807,
   "20": 0.0300918375478844,
   "30": 0.05790431300984171,
   "45": 0.054312253022505454,
   "60": 0.04243069940335644,
   "90": 0.25107945468335086
  },
  "t2_n_by_horizon": {
   "2": 35,
   "5": 35,
   "10": 35,
   "15": 35,
   "20": 33,
   "30": 32,
   "45": 32,
   "60": 25,
   "90": 8
  },
  "t2_end_in_window_brier": 0.07620343757307164,
  "t2_end_in_window_tests": 34,
  "days": 4,
  "per_day": {
   "18073": {
    "t1_crps_ua": NaN,
    "t1_cases": 0,
    "t2_integrated_brier": 0.0011690698149121879,
    "t2_end_in_window_brier": 0.004396904545208238,
    "t2_tests": 7
   },
   "18078": {
    "t1_crps_ua": NaN,
    "t1_cases": 0,
    "t2_integrated_brier": 0.21293521502054832,
    "t2_end_in_window_brier": 0.0938488824339651,
    "t2_tests": 9
   },
   "18285": {
    "t1_crps_ua": NaN,
    "t1_cases": 0,
    "t2_integrated_brier": 0.00188408708810723,
    "t2_end_in_window_brier": 0.001783729480040524,
    "t2_tests": 9
   },
   "18298": {
    "t1_crps_ua": NaN,
    "t1_cases": 0,
    "t2_integrated_brier": 0.19311170623109747,
    "t2_end_in_window_brier": 0.1888272264935475,
    "t2_tests": 9
   }
  }
 },
 "baselines_on_cases_answered": {
  "constant (simple)": {
   "t2_integrated_brier": 0.08842065086764149,
   "t2_horizons_used": [
    2,
    5,
    10,
    15,
    20,
    30,
    45,
    60,
    90
   ],
   "t2_brier_by_horizon": {
    "2": 6.830134553650662e-05,
    "5": 0.00027320538214602646,
    "10": 0.0006147121098285596,
    "15": 0.02791768712129343,
    "20": 0.029506181271771053,
    "30": 0.05873030856841887,
    "45": 0.06158673114254548,
    "60": 0.0983672621179019,
    "90": 0.2363113616810305
   },
   "t2_n_by_horizon": {
    "2": 35,
    "5": 35,
    "10": 35,
    "15": 35,
    "20": 33,
    "30": 32,
    "45": 32,
    "60": 25,
    "90": 8
   },
   "t2_end_in_window_brier": 0.11203022875816993,
   "t2_end_in_window_tests": 34,
   "days": 4,
   "per_day": {
    "18073": {
     "t1_crps_ua": NaN,
     "t1_cases": 0,
     "t2_integrated_brier": 0.04242478049097085,
     "t2_end_in_window_brier": 0.04340277777777778,
     "t2_tests": 7
    },
    "18078": {
     "t1_crps_ua": NaN,
     "t1_cases": 0,
     "t2_integrated_brier": 0.15541942300687855,
     "t2_end_in_window_brier": 0.10821759259259259,
     "t2_tests": 9
    },
    "18285": {
     "t1_crps_ua": NaN,
     "t1_cases": 0,
     "t2_integrated_brier": 0.0594926191753516,
     "t2_end_in_window_brier": 0.04340277777777778,
     "t2_tests": 9
    },
    "18298": {
     "t1_crps_ua": NaN,
     "t1_cases": 0,
     "t2_integrated_brier": 0.18381010099451855,
     "t2_end_in_window_brier": 0.2378472222222222,
     "t2_tests": 9
    }
   }
  },
  "stratified incidence (T2; a stratum absent from train takes the pooled incidence)": {
   "t2_integrated_brier": 0.11240988747795969,
   "t2_horizons_used": [
    2,
    5,
    10,
    15,
    20,
    30,
    45,
    60,
    90
   ],
   "t2_brier_by_horizon": {
    "2": 0.0,
    "5": 0.0,
    "10": 0.0,
    "15": 0.02857142857142857,
    "20": 0.030303030303030304,
    "30": 0.0625,
    "45": 0.06072909531505111,
    "60": 0.14159751988995534,
    "90": 0.30958730671205476
   },
   "t2_n_by_horizon": {
    "2": 35,
    "5": 35,
    "10": 35,
    "15": 35,
    "20": 33,
    "30": 32,
    "45": 32,
    "60": 25,
    "90": 8
   },
   "t2_end_in_window_brier": 0.14559463572215525,
   "t2_end_in_window_tests": 34,
   "days": 4,
   "per_day": {
    "18073": {
     "t1_crps_ua": NaN,
     "t1_cases": 0,
     "t2_integrated_brier": 0.0,
     "t2_end_in_window_brier": 0.0,
     "t2_tests": 7
    },
    "18078": {
     "t1_crps_ua": NaN,
     "t1_cases": 0,
     "t2_integrated_brier": 0.24936868686868688,
     "t2_end_in_window_brier": 0.1111111111111111,
     "t2_tests": 9
    },
    "18285": {
     "t1_crps_ua": NaN,
     "t1_cases": 0,
     "t2_integrated_brier": 0.23270065835551204,
     "t2_end_in_window_brier": 0.21301775147928997,
     "t2_tests": 9
    },
    "18298": {
     "t1_crps_ua": NaN,
     "t1_cases": 0,
     "t2_integrated_brier": 0.11798508526546708,
     "t2_end_in_window_brier": 0.2258953168044077,
     "t2_tests": 9
    }
   }
  }
 }
}
```

</details>


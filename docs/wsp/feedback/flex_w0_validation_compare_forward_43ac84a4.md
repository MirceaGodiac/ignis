# Comparison on validation (56 common cases)

- explicitly allowed mismatch: feature order differs: ["1", "c_o2", "c_co2", "c_he", "c_logp", "c_logd0", "c_o2^2", "c_o2*c_co2", "c_o2*c_logp"] vs ["c_o2", "c_co2", "c_he", "c_logp", "c_logd0", "w", "s", "e_late", "e_early", "g_span", "g_count", "g_progress"] (sha256:34e29dba6f9fbf479d39b35131ae6844f0e591bd36ecd4aa3c0aef5cb1ec2637)
- explicitly allowed mismatch: units differ: {"K_ref": "m^2/s", "d": "m", "pressure": "Pa", "q": "m^2", "t": "s", "x_co2": "1", "x_he": "1", "x_o2": "1"} vs {"c_co2": "1", "c_he": "1", "c_logd0": "1", "c_logp": "1", "c_o2": "1", "e_early": "1", "e_late": "1", "g_count": "1", "g_progress": "1", "g_span": "1", "s": "1", "w": "1"} (sha256:34e29dba6f9fbf479d39b35131ae6844f0e591bd36ecd4aa3c0aef5cb1ec2637)
- excluded (not sealed by all releases): 3

## sha256:34e29dba6f9fbf479d39b35131ae6844f0e591bd36ecd4aa3c0aef5cb1ec2637
- presented: 56, refused: 0 (rate 0), admitted denominator: 56, scored: 56

| metric | group-equal mean | 95% CI | cases | clusters |
|---|---|---|---|---|
| coverage@0.5 | 0.581 | [0.4224, 0.7413] | 56 | 9 |
| coverage@0.9 | 0.8502 | [0.7317, 0.9519] | 56 | 9 |
| crps | 5.041e-05 | [2.601e-05, 9.131e-05] | 56 | 9 |
| lpd | 8.226 | [7.701, 8.655] | 56 | 9 |
| mae_mean | 6.979e-05 | [3.627e-05, 0.0001255] | 56 | 9 |
| mae_median | 6.871e-05 | [3.473e-05, 0.0001249] | 56 | 9 |
| width@0.5 | 9.609e-05 | [7.203e-05, 0.0001219] | 56 | 9 |
| width@0.9 | 0.0002517 | [0.0001897, 0.0003193] | 56 | 9 |


## sha256:4bfddff5060f096d8f6a18cb382058ebd231dffd76b9b7f4b52f48bbbbef4894
- presented: 56, refused: 0 (rate 0), admitted denominator: 56, scored: 56

| metric | group-equal mean | 95% CI | cases | clusters |
|---|---|---|---|---|
| coverage@0.5 | 0.04924 | [0.02206, 0.08264] | 56 | 9 |
| coverage@0.9 | 0.1069 | [0.05191, 0.171] | 56 | 9 |
| crps | 0.0001375 | [7.962e-05, 0.0001999] | 56 | 9 |
| lpd | -6.093 | [-11.05, -1.756] | 50 | 9 |
| mae_mean | 0.0001498 | [8.792e-05, 0.0002162] | 56 | 9 |
| mae_median | 0.0001499 | [8.798e-05, 0.0002163] | 56 | 9 |
| width@0.5 | 1.254e-05 | [1.016e-05, 1.534e-05] | 56 | 9 |
| width@0.9 | 2.983e-05 | [2.429e-05, 3.644e-05] | 56 | 9 |


## sha256:d99ac9b6a533c6ba878ae9d9694d435edcd33996881a4454489f28eebcc9ea7e
- presented: 56, refused: 0 (rate 0), admitted denominator: 56, scored: 56

| metric | group-equal mean | 95% CI | cases | clusters |
|---|---|---|---|---|
| coverage@0.5 | 0.5019 | [0.4047, 0.614] | 56 | 9 |
| coverage@0.9 | 0.7856 | [0.6458, 0.9082] | 56 | 9 |
| crps | 0.0001034 | [6.485e-05, 0.0001483] | 56 | 9 |
| lpd | 6.804 | [5.742, 7.689] | 56 | 9 |
| mae_mean | 0.0001313 | [8.449e-05, 0.000184] | 56 | 9 |
| mae_median | 0.000145 | [9.923e-05, 0.0001967] | 56 | 9 |
| width@0.5 | 0.0001748 | [0.000152, 0.0001967] | 56 | 9 |
| width@0.9 | 0.0004176 | [0.0003634, 0.00047] | 56 | 9 |


## Paired differences: sha256:34e29dba6f9fbf479d39b35131ae6844f0e591bd36ecd4aa3c0aef5cb1ec2637 − sha256:4bfddff5060f096d8f6a18cb382058ebd231dffd76b9b7f4b52f48bbbbef4894

| metric | group-equal mean | 95% CI | cases | clusters |
|---|---|---|---|---|
| coverage@0.5 | 0.5317 | [0.3789, 0.6927] | 56 | 9 |
| coverage@0.9 | 0.7433 | [0.6301, 0.8471] | 56 | 9 |
| crps | -8.708e-05 | [-0.0001533, -2.601e-05] | 56 | 9 |
| lpd | 14.41 | [10.17, 19.19] | 50 | 9 |
| mae_mean | -8.003e-05 | [-0.0001559, -9.079e-06] | 56 | 9 |
| mae_median | -8.116e-05 | [-0.0001562, -9.889e-06] | 56 | 9 |
| width@0.5 | 8.355e-05 | [6.025e-05, 0.0001087] | 56 | 9 |
| width@0.9 | 0.0002219 | [0.0001621, 0.0002895] | 56 | 9 |

## Paired differences: sha256:d99ac9b6a533c6ba878ae9d9694d435edcd33996881a4454489f28eebcc9ea7e − sha256:4bfddff5060f096d8f6a18cb382058ebd231dffd76b9b7f4b52f48bbbbef4894

| metric | group-equal mean | 95% CI | cases | clusters |
|---|---|---|---|---|
| coverage@0.5 | 0.4527 | [0.3757, 0.5372] | 56 | 9 |
| coverage@0.9 | 0.6787 | [0.566, 0.7891] | 56 | 9 |
| crps | -3.405e-05 | [-8.839e-05, 1.116e-05] | 56 | 9 |
| lpd | 13.43 | [9.249, 18.23] | 50 | 9 |
| mae_mean | -1.85e-05 | [-7.148e-05, 2.795e-05] | 56 | 9 |
| mae_median | -4.891e-06 | [-6.288e-05, 4.521e-05] | 56 | 9 |
| width@0.5 | 0.0001623 | [0.0001401, 0.0001837] | 56 | 9 |
| width@0.9 | 0.0003877 | [0.0003347, 0.0004412] | 56 | 9 |

Lower is better except lpd. Intervals: cluster bootstrap over independent groups (sessions merged); few clusters ⇒ wide, unstable intervals. Read with the per-case table.

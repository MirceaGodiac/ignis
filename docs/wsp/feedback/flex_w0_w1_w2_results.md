# FLEX — development results of W0, W1 and W2 against the fitted D² baselines

Cutoffs 1, 3, 5 s; horizon 10 s; metrics in µm except coverage and lpd; group-equal means from `ignis eval compare` (validation, sealed) and case means from `wsp-dev feedback` (train). Validation became development data after W1 (its case detail was read before W2). The test partition was not read.

## Validation (sealed, paired on common cases)

```text
partition validation, common cases 56, excluded incomplete 3, reference d2_population(fitted)

release                             crps    mae_median      mae_mean  coverage@0.5  coverage@0.9           lpd     width@0.9
flex.wsp_W1                        56.30         76.99         76.94          0.49          0.81          8.09        218.88
d2_population(fitted)             137.49        149.86        149.82          0.05          0.11         -6.09         29.83
flex.wsp_W0                        50.41         68.71         69.79          0.58          0.85          8.23        251.71
d2_prefix(fitted)                 103.43        144.97        131.32          0.50          0.79          6.80        417.56
flex.wsp_W2                        41.71         59.78         58.88          0.56          0.93          8.37        239.91

paired differences (other - reference), group-equal mean [95% CI]:
```

## Train (in-process, evaluator scoring functions)

```text
flex.wsp sha256:fbd3d6d4 (not_trained)
  crps          case mean    45.40   group-equal    63.78  ci [28.01, 107.16]  cases 182
  mae_median    case mean    56.55   group-equal    76.73  ci [36.68, 123.34]  cases 182
  coverage@0.9  case mean     0.91   group-equal     0.88  ci [0.80, 0.95]  cases 182
  refused       3
flex.wsp sha256:2b168d0a (not_trained)
  crps          case mean    44.10   group-equal    61.75  ci [26.25, 105.76]  cases 182
  mae_median    case mean    54.18   group-equal    72.97  ci [34.26, 120.24]  cases 182
  coverage@0.9  case mean     0.92   group-equal     0.88  ci [0.81, 0.95]  cases 182
  refused       3
flex.wsp sha256:9a830c7d (not_trained)
  crps          case mean    62.38   group-equal    79.07  ci [42.05, 122.03]  cases 182
  mae_median    case mean    85.56   group-equal   103.42  ci [58.31, 149.69]  cases 182
  coverage@0.9  case mean     0.85   group-equal     0.83  ci [0.75, 0.91]  cases 182
  refused       3
flex.d2_prefix sha256:d99ac9b6 (nasa)
  crps          case mean    81.12   group-equal    99.61  ci [71.03, 146.86]  cases 164
  mae_median    case mean   124.25   group-equal   144.96  ci [110.41, 199.66]  cases 164
  coverage@0.9  case mean     0.93   group-equal     0.90  ci [0.81, 0.96]  cases 164
  refused       21
flex.d2_population sha256:4bfddff5 (nasa)
  crps          case mean   149.95   group-equal   175.61  ci [134.27, 231.20]  cases 164
  mae_median    case mean   165.49   group-equal   193.79  ci [148.48, 252.95]  cases 164
  coverage@0.9  case mean     0.15   group-equal     0.13  ci [0.07, 0.20]  cases 164
  refused       21
```

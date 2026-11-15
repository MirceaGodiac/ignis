# Final run of the seventh generation (PROTOCOL 12.5)

## PSI-20 BRE (`bre`)

Release list sha256 `f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`. Outcome coding v1. Test days only; aggregates; lower is better.

| release | role | rule | answered | t1: extinction IBS | t1: mean of the two causes | t1: operator termination IBS |
|---|---|---|---|---|---|---|
| bre d3 by the lasts rule | model | lasts | {"t1": 95} | 0.1389 | 0.0962 | 0.0535 |
| bre W0 by flame_end (registered zero-shot reading) | comparator 1 | flame_end | {"t1": 83} | 0.2448 | 0.1759 | 0.1070 |
| bre W0 by the lasts rule (phys_bre2.md section 2) | comparator 2 | lasts | {"t1": 83} | 0.2047 | 0.1473 | 0.0899 |
| bre D1 by flame_end | comparator 3 | flame_end | {"t1": 83} | 0.2223 | 0.1603 | 0.0983 |
| bre d2 by the lasts rule | comparator 4 | lasts | {"t1": 83} | 0.1984 | 0.1436 | 0.0887 |
| bre d3m by the lasts rule | comparator 5 | lasts | {"t1": 95} | 0.0982 | 0.0701 | 0.0419 |
| baseline: incidence (simple) | on the model's cases | | | 0.2220 | 0.1626 | 0.1031 |
| baseline: planned stop | on the model's cases | | | 0.2250 | 0.1625 | 0.0999 |

## PSI-21 CLD Flame (`cld`)

Release list sha256 `f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`. Outcome coding v1. Test days only; aggregates; lower is better.

| release | role | rule | answered | loss: Brier of lost before fuel-off | loss: integrated Brier | loss: integrated Brier, mean of day means | radiometer: CRPS of log10 V | radiometer: mean of day means |
|---|---|---|---|---|---|---|---|---|
| cld3 (third law) | model | lane | {"loss": 63, "radiometer": 63} | 0.0175 | 0.1239 | 0.0732 | 0.1145 | 0.1134 |
| cld W0 | comparator 1 | lane | {"loss": 63, "radiometer": 63} | 0.0545 | 0.0786 | 0.0342 | 0.1411 | 0.1384 |
| cld D1 | comparator 2 | lane | {"loss": 63, "radiometer": 63} | 0.0443 | 0.0726 | 0.0320 | 0.1145 | 0.1134 |
| cld C1 | comparator 3 | lane | {"loss": 63, "radiometer": 63} | 0.0173 | 0.1052 | 0.0574 | 0.1145 | 0.1134 |
| cld2 (second law) | comparator 4 | lane | {"loss": 63, "radiometer": 63} | 0.0372 | 0.1094 | 0.0476 | 0.1145 | 0.1134 |
| baseline: incidence (simple) | on the model's cases | | | 0.1022 | 0.1020 | 0.0559 | 0.3145 | 0.3128 |
| baseline: by fuel | on the model's cases | | | 0.1017 | 0.1199 | 0.0688 | 0.2547 | 0.2527 |
| baseline: loss by dilution ramp | on the model's cases | | | 0.0568 | 0.1166 | 0.0565 | - | - |
| baseline: loss, classical LFL at the stoichiometric surface | on the model's cases | | | 0.1111 | 0.1201 | 0.0659 | - | - |
| baseline: radiometer, Roper length plus fuel offset | on the model's cases | | | - | - | - | 0.1439 | 0.1750 |

## PSI-22 E-FIELD Flames (`efield`)

Release list sha256 `f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`. Outcome coding v1. Test days only; aggregates; lower is better.

| release | role | rule | answered | t1: CRPS uA, case mean | t1: mean of day means | t2: Brier of ends in window | t2: integrated Brier |
|---|---|---|---|---|---|---|---|
| efield3 (third law) | model | lane | {"t1": 788, "t2": 39} | 0.3190 | 0.3135 | 0.0547 | 0.0348 |
| efield W0 | comparator 1 | lane | {"t1": 788, "t2": 39} | 0.7039 | 0.7085 | 0.1072 | 0.0282 |
| efield E1 | comparator 2 | lane | {"t1": 788, "t2": 39} | 0.3238 | 0.3115 | 0.1072 | 0.0282 |
| efield2 (second law) | comparator 3 | lane | {"t1": 788, "t2": 39} | 0.7719 | 0.8066 | 0.1964 | 0.0898 |
| baseline: constant (simple) | on the model's cases | | | 0.9864 | 0.9847 | 0.1331 | 0.0394 |
| baseline: space-charge law up to saturation, train sigma (T1) | on the model's cases | | | 0.5746 | 0.5625 | - | - |
| baseline: stratified incidence (T2; a stratum absent from train takes the pooled incidence) | on the model's cases | | | - | - | 0.1456 | 0.0551 |

## PSI-107 SPICE (`spice`)

Release list sha256 `f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`. Outcome coding v1. Test days only; aggregates; lower is better.

| release | role | rule | answered | T1: pooled mean | T1: quantile score x2, mm, mean of day means | T2: pooled mean | T2: quantile score x2, mm, mean of day means |
|---|---|---|---|---|---|---|---|
| spice P3 | model | lane | {"T1": 6, "T2": 6} | 4.1757 | 4.1757 | 1.1804 | 1.1804 |
| spice P2 | comparator 1 | lane | {"T1": 6, "T2": 6} | 4.1757 | 4.1757 | 2.4458 | 2.4458 |
| spice W0 (variant B) | comparator 2 | lane | {"T1": 6, "T2": 6} | 6.9358 | 6.9358 | 2.9823 | 2.9823 |
| spice lit (published fit; not blind) | comparator 3 | lane | {"T1": 6, "T2": 6} | 3.1598 | 3.1598 | 2.9823 | 2.9823 |
| baseline: B0 pooled climatology (simple) | on the model's cases | | | 10.5782 | 10.5782 | 10.5782 | 10.5782 |
| baseline: B1 per-fuel climatology | on the model's cases | | | 10.3261 | 10.3261 | 10.3261 | 10.3261 |
| baseline: Lin and Faeth coflow law, train sigma (T2 only) | on the model's cases | | | - | - | 2.5627 | 2.5627 |

## PSI-101 SAME-R (`samer`)

Release list sha256 `f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`. Outcome coding v1. Test days only; aggregates; lower is better.

| release | role | rule | answered | d10: CRPS of ln d10, case mean | d10: day-equal | iss: Brier, case mean | iss: day-equal |
|---|---|---|---|---|---|---|---|
| samer S1 | model | lane | {"d10": 33, "iss": 33} | 0.1672 | 0.1508 | 0.0144 | 0.0123 |
| samer W0 | comparator 1 | lane | {"d10": 33, "iss": 33} | 0.2595 | 0.2050 | 0.0807 | 0.0858 |
| samer D1 | comparator 2 | lane | {"d10": 33, "iss": 33} | 0.2165 | 0.1702 | 0.0477 | 0.0590 |
| baseline: B0 / I0 constant (simple, SAME-R train) | on the model's cases | | | 0.3609 | 0.3483 | 0.2997 | 0.2303 |
| baseline: B1 / I1 per material (SAME-R train; a material absent from it takes B0 / I0) | on the model's cases | | | 0.2559 | 0.2532 | 0.0189 | 0.0140 |

## PSI-26 BASS and PSI-25 BASS-II plastics and fabrics (`bass`)

Release list sha256 `f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`. Outcome coding v2. Test days only; aggregates; lower is better.

| release | role | rule | answered | quench: case mean | quench: interval CRPS cm/s, mean of day means | quench_display: case mean | quench_display: interval CRPS, fan display units, mean of day means | sustain: Brier, mean of day means | sustain: case mean |
|---|---|---|---|---|---|---|---|---|---|
| bass4 (fabrics in kept burning) | model | lane | {"quench": 38, "sustain": 42} | 1.5438 | 2.9103 | 4.0160 | 7.4058 | 0.0765 | 0.0477 |
| bass3 | comparator 1 | lane | {"quench": 38, "sustain": 38} | 1.5438 | 2.9103 | 4.0160 | 7.4058 | 0.0758 | 0.0523 |
| bass3 Q1 | comparator 2 | lane | {"quench": 38, "sustain": 38} | 1.5284 | 2.8917 | 3.9983 | 7.3842 | 0.0758 | 0.0523 |
| bass3 S1 (os1) | comparator 3 | lane | {"quench": 38, "sustain": 38} | 1.5438 | 2.9103 | 4.0160 | 7.4058 | 0.0780 | 0.0585 |
| bass2 K1 | comparator 4 | lane | {"quench": 38, "sustain": 38} | 1.7832 | 2.8620 | 4.5270 | 7.1415 | 0.0758 | 0.0523 |
| bass2 s1 | comparator 5 | lane | {"quench": 38, "sustain": 38} | 1.6604 | 2.4582 | 4.2728 | 6.2803 | 0.0758 | 0.0523 |
| bass1 W0 | comparator 6 | lane | {"quench": 32, "sustain": 34} | 1.9780 | 1.6069 | 5.2226 | 4.3848 | 0.0821 | 0.0588 |
| bass1 D1 | comparator 7 | lane | {"quench": 32, "sustain": 34} | 1.3364 | 0.9847 | 3.4729 | 2.6428 | 0.0821 | 0.0588 |
| baseline: constant (simple) | on the model's cases | | | 2.1082 | 2.7049 | 5.3626 | 6.7190 | - | - |
| baseline: Bhattacharjee et al. 2016, v1 train log width | on the model's cases | | | 2.5790 | 3.4074 | 6.7476 | 8.8344 | - | - |
| baseline: train incidence (simple) | on the model's cases | | | - | - | - | - | 0.1120 | 0.0707 |
| baseline: always lit | on the model's cases | | | - | - | - | - | 0.1282 | 0.0750 |

## PSI-106 SLICE (`slice`)

Release list sha256 `f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`. Outcome coding v2. Test days only; aggregates; lower is better.

| release | role | rule | answered | blow_off: Brier, mean of day means | blow_off: pooled | lifted: Brier, mean of day means | lifted: pooled |
|---|---|---|---|---|---|---|---|
| slice2 s1 (second law) | model | lane | {"blow_off": 31, "lifted": 31} | 0.0021 | 0.0021 | 0.0041 | 0.0041 |
| slice W0 | comparator 1 | lane | {"blow_off": 31, "lifted": 31} | 0.0021 | 0.0021 | 0.0466 | 0.0466 |
| slice D1 | comparator 2 | lane | {"blow_off": 31, "lifted": 31} | 0.0021 | 0.0021 | 0.0464 | 0.0464 |
| slice2 S2 | comparator 3 | lane | {"blow_off": 31, "lifted": 31} | 0.0565 | 0.0565 | 0.0041 | 0.0041 |
| baseline: incidence pooled (simple) | on the model's cases | | | 0.0028 | 0.0028 | 0.0008 | 0.0008 |
| baseline: incidence by nominal tube | on the model's cases | | | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| baseline: never (persistence) | on the model's cases | | | 0.0000 | 0.0000 | 0.0000 | 0.0000 |


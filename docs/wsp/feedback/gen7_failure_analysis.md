# Seventh generation: where the hand-written laws lose, and why (PROTOCOL 13.4)

Development, in-sample: train and validation cases together. The validation days have been used twice for
selection (13.2, 13.3) and judge nothing more (13.4); every validation number here is development, in-sample.
Test days stay unread. Adopted releases: cld D1, bass1 D1, bre W0, slice W0, efield W0, spice W0 (variant B),
samer W0; same and daft keep W0. Per-case files: `runs/dev/data_all/work/transfer_<slug>/law_study_<id16>[_dev_validation].jsonl`;
tables made by `tools/wsp/ext/law_failure.py` (full output under `runs/dev/gen7/failure/`). Per-case losses below
are per-case contributions (cld: mean Brier over the horizons at risk; bre: IPCW integrated Brier per case), so
stratum means are comparable between the law and the baseline but are not the lane's aggregate.

## 1. The four targets that lose

### cld loss (D1 against the best baseline, incidence by dilution ramp)

| stratum | cases | observed loss rate | forecast P(lost before fuel-off) | law | best baseline | law minus baseline |
|---|---|---|---|---|---|---|
| validation | 58 | 0.276 | 0.103 | 0.118 | 0.090 | +0.029 |
| train | 198 | 0.086 | 0.055 | 0.033 | 0.029 | +0.005 |
| **procedure: script dilutes during the test** | 16 | 0.812 | 0.387 | 0.221 | 0.060 | **+0.160** |
| script does not dilute | 240 | 0.083 | 0.044 | 0.041 | 0.041 | 0.000 |
| **procedure: velocity ramps (ramp_1..3)** | 26 | 0.577 | 0.254 | 0.191 | 0.086 | **+0.106** |
| no velocity ramps | 230 | 0.078 | 0.044 | 0.037 | 0.038 | -0.001 |
| procedure: coflow matching ramp | 50 | 0.360 | 0.163 | 0.132 | 0.083 | +0.049 |
| methane | 114 | 0.219 | 0.102 | 0.086 | 0.063 | +0.023 |
| ethylene | 142 | 0.056 | 0.037 | 0.026 | 0.026 | 0.000 |
| final fuel fraction 0.25-0.4 | 42 | 0.500 | 0.162 | 0.196 | 0.133 | +0.063 |
| round 1 | 104 | 0.240 | 0.104 | 0.094 | 0.069 | +0.025 |

The whole deficit is in tests whose script ramps something (fuel fraction or velocities) towards a harder
condition. On the 230 to 240 tests without a ramp the law equals the best baseline.

### bass1 quench_u (D1 against Bhattacharjee 2016)

| stratum | cases | observed interval midpoint, cm/s | forecast median, cm/s | law | best baseline | law minus baseline |
|---|---|---|---|---|---|---|
| validation | 9 | 1.49 | 0.68 | 0.382 | 0.187 | +0.195 |
| train | 39 | 2.74 | 0.68 | 1.104 | 1.226 | -0.123 |
| **PMMA rod, opposed** | 14 | 1.24 | 0.0006 | 0.927 | 0.161 | **+0.766** |
| PMMA rod, stagnation | 5 | 0.98 | 0.007 | 0.844 | 0.514 | +0.329 |
| PMMA film, opposed | 5 | 2.38 | 0.95 | 0.200 | 0.058 | +0.141 |
| ignition speed below 5 cm/s | 9 | 1.33 | 0.12 | 0.580 | 0.207 | +0.373 |
| O2 below 0.16 | 2 | 1.32 | 0.013 | 1.125 | 0.345 | +0.779 |

The rods go out at about 1 cm/s where the law says "no quench while the fan turns" (median below 0.01 cm/s).
On train, D1 now beats the baseline on the slabs and loses on the rods.

### bre extinction (W0 against the planned-stop baseline)

| stratum | cases (test x cutoff) | observed extinction by 60 s | forecast | law | best baseline | law minus baseline |
|---|---|---|---|---|---|---|
| validation | 87 | 0.241 | 0.479 | 0.276 | 0.213 | +0.063 |
| train | 240 | 0.475 | 0.589 | 0.251 | 0.232 | +0.019 |
| **pressure 65-85 kPa** | 49 | 0.204 | 0.717 | 0.425 | 0.225 | **+0.199** |
| ambient O2 0.30-0.36 | 97 | 0.258 | 0.508 | 0.323 | 0.228 | +0.095 |
| ambient O2 0.25-0.30 | 76 | 0.487 | 0.718 | 0.295 | 0.218 | +0.077 |
| procedure: planned stop 150-250 s | 112 | 0.357 | 0.530 | 0.305 | 0.233 | +0.072 |
| ethylene 50 % bottle | 111 | 0.351 | 0.575 | 0.273 | 0.232 | +0.042 |
| pressure >= 85 kPa | 207 | 0.522 | 0.580 | 0.209 | 0.221 | -0.012 |
| 50 mm burner | 51 | 0.471 | 0.742 | 0.189 | 0.210 | -0.022 |

At 1 atm the law is at least as good as the baseline. It loses at reduced pressure and in enriched oxygen, where it
forecasts extinction far more often than it happens: the held class (flames that never go out) is missing from the
event target.

### slice lifted (W0 against the pooled incidence)

| stratum | lines | observed lifted | forecast | law | best baseline | law minus baseline |
|---|---|---|---|---|---|---|
| train | 268 | 0.030 | 0.277 | 0.180 | 0.029 | +0.151 |
| validation | 34 | 0.059 | 0.206 | 0.089 | 0.056 | +0.032 |
| **0.4 mm tube** | 94 | 0.064 | 0.637 | 0.477 | 0.061 | **+0.416** |
| 40 % methane (day 061) | 39 | 0.077 | 0.773 | 0.712 | 0.073 | +0.638 |
| ethane | 79 | 0.000 | 0.195 | 0.109 | 0.001 | +0.108 |
| procedure: jet stepped up from the previous line | 48 | 0.062 | 0.437 | 0.288 | 0.060 | +0.228 |
| procedure: coflow stepped up | 132 | 0.038 | 0.311 | 0.192 | 0.037 | +0.156 |
| procedure: first line of the flame | 82 | 0.012 | 0.205 | 0.135 | 0.012 | +0.123 |
| 3.2 mm tube | 60 | 0.000 | 0.050 | 0.006 | 0.001 | +0.005 |

The law over-forecasts lifting everywhere, by a factor of about 7. The label "lifted" is rare (3 to 6 % of lines),
and it is written only when the crew noted it.

## 2. What the best baseline knows that the law does not, and the fields that would separate the cases

| target | what the best baseline uses | field that separates the cases (no outcome) | coverage |
|---|---|---|---|
| cld loss | whether the script dilutes during the test (`x_fuel_final < x_fuel_initial`) | dilution flag; velocity-ramp flag (stages `ramp_1..3` in `schedule_from_ignition_command`); the time spent at the most severe condition before fuel-off (from the schedule) | 256 of 256 scored cases (train and validation) |
| bass1 quench_u | the oxygen level alone (Bhattacharjee: a critical speed that rises as oxygen falls, about 1.3 to 2 cm/s), with no geometry | geometry class (`material_class`, `configuration`, `diameter_m`); for rods, the radiative-loss floor that the law's `D_c / R` term cancels | all covered cases |
| bre extinction | the planned stop; incidence by cutoff | `pressure_pa` and `x_o2_ambient` (in the law already); the held class needs a target of its own | 327 of 327 test-cutoff cases |
| slice lifted | the rarity of the label (pooled rate 0.03) | `step_in_test`, the step from the previous line (derived from the conditions of earlier lines), `burner_nominal_mm` | 302 of 302 lines |

## 3. How validation differs from train

| investigation | condition | train | validation |
|---|---|---|---|
| cld | tests whose script dilutes | 11 of 218 (5 %) | 11 of 72 (15 %) |
| cld | median final fuel fraction | 0.4 | 0.3 |
| cld | campaign round | rounds 1-3 | rounds 1-2 only (round 3 never lost a flame) |
| bre | fuel | ethylene 98, methane 18 | ethylene 35, no methane |
| bre | bottle | median 1.0 (pure) | median 0.5 (50 % in N2) |
| bre | burner | 25 and 50 mm | 25 mm only |
| samer | inlet velocity | 32 of 50 at 8.1 cm/s, 6 at 0 | 16 of 24 at 0 (no flow) |
| samer | material | silicone 17, Kapton 8 | Kapton 11, silicone 3 |

Per-day validation scores (development, in-sample), W0 against D1:

| slug | day | W0 | D1 | best baseline |
|---|---|---|---|---|
| bre (extinction IBS) | 19051 | 0.139 | **0.194** | 0.200 |
| bre | 19063 | 0.083 | 0.083 | 0.188 |
| bre | 20346 | 0.258 | 0.261 | 0.202 |
| bre | 20352 | 0.478 | 0.480 | 0.246 |
| samer (d10 CRPS) | 2010-07-21 | 0.112 | 0.112 | 0.597 |
| samer | 2010-09-07 | 0.160 | **0.178** | 0.355 |
| samer | 2010-09-08 | 0.182 | 0.179 | 0.285 |
| cld (loss IBS) | 18003 | 0.563 | 0.528 | 0.026 (dilution ramp) |
| cld | 18008 | 0.810 | 0.723 | 0.555 |

- **cld:** validation is harder for every forecaster because three times as many of its tests ramp the script towards a
  harder condition. The two worst days, 18003 and 18008, hold 13 of the validation ramp tests (8 dilute, 5 more ramp
  velocities); 3 more dilute on 18249. The dilution-ramp baseline gives a diluting test the train rate of those
  tests (most are lost) and scores 0.026 on 18003. The law gives such tests a mean 0.39, because it reads only the
  lowest fraction and the window, not that the script runs on to extinction.
- **BRE:** the D1 step changed the pressure exponent. That fits the train strata at reduced pressure, but the
  validation days are almost all at 1 atm with 50 % bottles. It hurt one 1 atm, 34 % O2 day (19051: 0.139 to
  0.194) and left the others unchanged.
- **SAME-R:** the D1 step moved Pyrell and silicone, the train materials. Validation is mostly Kapton and two
  thirds no-flow, so the step hit only the few silicone and Pyrell rows (day 2010-09-07 worse).

## 4. One change per target

| target | the single change most likely to help | can `law.rs` express it |
|---|---|---|
| cld loss | add the procedure inputs: a dilution-ramp flag, and the coflow at the end of the schedule beside its maximum (velocity ramps go downwards); the loss clock then reads the condition the flame actually ends in | yes: two features of a new declaration (new slug) |
| bass1 quench_u | rods: a quench floor that does not vanish with small radius (a radiative-loss term on the rod tip, a hinge on `nose_radius`) | yes, as a hinge on z_r in `quench_speed.median` |
| bre extinction | score the held class through `lasts` (yes/no at the planned stop), or recombine `flame_end` with `lasts` as a cure fraction | the recombination is not expressible (no mass at infinity on a clock); scoring `lasts` at the planned stop is |
| slice lifted | a window between the lifting and blow-off limits (lifted and not yet blown off), and a label prior for an unmarked line | the window needs two thresholds on one output: not one network; expressible as two targets combined at scoring time |
| efield ion current | carry the coflow: with coflow the median is low by 0.40 in ln, without it high (train anatomy) | yes, through `u_coflow` as a feature (new declaration) |
| spice T1 (variant B) | nozzle-dependent dilution of the coflow term (0.76 mm median high by 0.47 in ln) | yes |
| bre termination | none: it is the planned-stop construction, shared with the baseline | - |
| slice blow-off | the 0.4 mm, 40 % methane day (61): the fuel label is in doubt (log "100 %" against the bottle table "40 %") | an input check, not a law change |
| same detector | DBP and silicone saturate the detector; I1 knows the material incidence | the material terms exist; a revision may use SAME-R train |

Detail tables: `runs/dev/gen7/failure/{cld,bass1,bre,slice,shift}.md`.

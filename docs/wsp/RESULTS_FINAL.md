# Final run of 15 November (PROTOCOL 12.5, registered as 12.11)

This run used data that no release of this generation was written or chosen from. It was run once by
the scoring agent (claude-opus-5-5) with `tools/wsp/final/run_final.py`. Each step was rehearsed first on
the validation partition or validation days, and every known number reproduced. Every table below holds
aggregates only. Per-case and per-test rows stay in `runs/final/20261115/local/` and are not in git.

**Nothing is adopted or refused here.** These are the adopted releases measured once on held data. Some
sets are very small (one session, one day, one droplet, two burns). Read each table with its counts.

## Identity

- **Release list:** `tools/wsp/final/releases_final_20261115.json`, sha256
  `a46df540bb69f8b573738255f981b8435f063580a44b429ae7b1352f97108415`. The hash was verified before any step.
- **Binaries:**
  - `target-merged/release/ignis.exe` `b03a35c820d08347afb15fa232e6d10deef7d288463b574a415a918df4248ffb`
  - `target-merged/release/wsp-dev.exe` `bd6055e0958751aa4a28aca7160d0b8348bb5a8aecddba1f20c28e6345ab4d73`
- **Steps run:** each step ran once, in order 1, 2, 3, 4, 5, and each completed. The markers are in
  `runs/final/20261115/markers/`. Step 5 ran after its separate go, once the scale was pinned.
- **Reveal-log lines added for the final run** (`runs/evaluation/reveal_log.jsonl`, `final_test: true`):
  3,792 in all, every one granted.

  | Step | Writer | Lines | Purpose |
  |---|---|---|---|
  | 1 | the evaluator | 2,900 | `compare` |
  | 2 | this script | 882 | `final:flame_design` |
  | 3 | this script | 6 | `final:cfi` |
  | 4 | `wsp-dev` | 2 | `final:bass_front` |
  | 5 | this script | 2 | `final:flex2` |

**Metrics.**
- **Droplets:** CRPS and MAE of the median in µm, and coverage of the 50 % and 90 % bands.
- **Burner flames:** integrated Brier score from 6 to 120 s, for extinction and for operator termination
  ("termination"), and their sum. Lower is better.

**Paired differences** are the adopted release minus the comparator.
- Step 1: the evaluator's cluster bootstrap over sessions (95 %). Where a comparison holds both data
  inputs, the half for each input is recomputed with `tools/wsp/analysis/cmp_split.py`, which reproduces
  the evaluator exactly (checked).
- Steps 2, 3 and 5: the chains' session or day bootstrap.
- Step 4: wsp-dev's bootstrap over burns.

## Step 1. Sealed test partitions of the project

### FLEX methanol (PSI-69): `flex_methanol.wsp2` D3

The test partition holds 65 cases of 22 runs in 9 sessions, and 1 case not at risk. No release refused a
case. 64 cases scored, because one has no target point in the window.

| Release | CRPS | MAE of median | Cov. 50 % | Cov. 90 % |
|---|---|---|---|---|
| **D3 (adopted)** | **18.5** | 22.4 | 0.83 | 0.99 |
| D-squared law of the spec | 31.6 | 43.8 | 0.65 | 0.93 |
| `flex.v3` (MAP, fitted) | 42.7 | 65.1 | 0.40 | 0.96 |
| `flex.d2_prefix` (fitted) | 55.7 | 89.7 | 0.49 | 0.98 |
| `flex.v1` (MAP, fitted) | 58.9 | 75.7 | 0.68 | 0.97 |
| `flex.d2_population` (fitted) | 75.4 | 82.9 | 0.07 | 0.18 |

Paired CRPS difference, D3 minus comparator (µm; 65 cases, 9 sessions):

| Comparator | Difference | 95 % interval |
|---|---|---|
| D-squared law | −13.1 | [−19.0, −7.7] |
| W0 | −18.7 | [−35.2, −5.8] |
| v3 | −24.2 | [−32.7, −15.9] |
| d2_prefix | −37.2 | [−45.4, −28.8] |
| v1 | −40.4 | [−56.2, −26.5] |
| d2_population | −56.9 | [−86.5, −30.7] |

### FLEX n-heptane (PSI-69): `flex_heptane.wsp2` D2

The test partition holds 43 cases of 16 runs in 7 sessions, and 5 cases not at risk. W0 refused 1 case
(horizon), so the comparison with W0 is on 42 cases. There is no fitted heptane baseline.

| Release | Cases | CRPS | MAE of median | Cov. 50 % | Cov. 90 % |
|---|---|---|---|---|---|
| **D2 (adopted)** | 43 | **32.2** | 43.9 | 0.68 | 0.91 |
| D-squared law of the spec | 43 | 54.0 | 72.4 | 0.56 | 0.77 |
| D2, on the 42 cases W0 answers | 42 | 33.3 | 46.0 | 0.68 | 0.92 |

Paired CRPS difference, D2 minus comparator (µm; 7 sessions):

| Comparator | Difference | 95 % interval |
|---|---|---|
| D-squared law | −21.8 | [−47.1, −4.9] |
| W0 | −47.0 | [−124.9, −3.2] |

### Normal burner flames (CFI-G, PSI-159): `acme_normal.wsp12` F2

The test partition holds 60 cases of 20 runs in **2 sessions**. No case was not at risk, and no release
refused a case on either input. F2 was scored on both inputs: the shared records, and the planned stop of
`runs/dev/data_ext2`. The comparators were scored on the shared records, and the baseline set of the spec
on both inputs.

| Release | Input | Extinction | Termination | Sum |
|---|---|---|---|---|
| **F2 (adopted)** | shared records | **0.0414** | 0.0494 | 0.0908 |
| **F2 (adopted)** | with the planned stop | **0.0411** | 0.0466 | 0.0877 |
| `acme_normal.wsp5` P1 (the model of 14 November) | shared | 0.0456 | 0.0558 | 0.1014 |
| baseline set of the spec | either | 0.1200 | 0.1750 | 0.2950 |
| linear baseline (fitted) | shared | 0.0933 | 0.1199 | 0.2132 |
| hazard (fitted) | shared | 0.1068 | 0.1180 | 0.2247 |
| piecewise baseline (fitted) | shared | 0.2388 | 0.1717 | 0.4105 |
| constant baseline (fitted) | shared | 0.2420 | 0.1686 | 0.4106 |

Paired difference, F2 minus comparator (60 cases, 2 sessions; with 2 sessions an interval is little more
than the range of the two):

| Comparator | Input | Extinction | Termination |
|---|---|---|---|
| P1 | shared | −0.0036 [−0.0056, −0.0016] | −0.0109 [−0.0260, +0.0042] |
| baseline set | shared | −0.0779 [−0.0803, −0.0754] | −0.1366 [−0.1733, −0.0998] |
| baseline set | with stop | −0.0774 [−0.0823, −0.0725] | −0.1372 [−0.1667, −0.1077] |
| linear | shared | −0.0498 [−0.0566, −0.0430] | −0.0699 [−0.0719, −0.0678] |
| hazard | shared | −0.0621 [−0.0728, −0.0514] | −0.0701 [−0.0751, −0.0650] |
| piecewise | shared | −0.1991 [−0.2050, −0.1933] | −0.1325 [−0.1666, −0.0984] |
| constant | shared | −0.2005 [−0.2007, −0.2002] | −0.1292 [−0.1623, −0.0960] |

### Inverse burner flames (CFI-G, PSI-159): `acme_inverse.wsp12` F2

The test partition holds 98 cases of 34 runs, and 4 cases not at risk.
- F2, P1 and the baseline set refused 5 cases (condition outside the domain). They answered 93 cases of
  31 runs in **3 sessions**.
- The fitted baselines refused 29 (condition support). Comparisons with them are on 69 cases of 23 runs in
  3 sessions.

| Release | Input | Cases | Extinction | Termination | Sum |
|---|---|---|---|---|---|
| **F2 (adopted)** | shared records | 93 | **0.0998** | 0.1103 | 0.2100 |
| **F2 (adopted)** | with the planned stop | 93 | **0.1045** | 0.0793 | 0.1838 |
| `acme_inverse.wsp5` P1 | shared | 93 | 0.0998 | 0.1103 | 0.2100 (identical forecasts to F2 on these alkane flames without the stop) |
| baseline set of the spec | either | 93 | 0.2222 | 0.2325 | 0.4547 |
| F2 on the 69 cases the fitted baselines answer | shared | 69 | 0.1045 | 0.0752 | 0.1797 |
| hazard (fitted) | shared | 69 | 0.1645 | 0.0817 | 0.2462 |
| linear baseline (fitted) | shared | 69 | 0.1744 | 0.0809 | 0.2553 |
| piecewise baseline (fitted) | shared | 69 | 0.2270 | 0.1385 | 0.3655 |
| constant baseline (fitted) | shared | 69 | 0.2233 | 0.1562 | 0.3796 |

Paired difference, F2 minus comparator (3 sessions):

| Comparator | Input | Cases | Extinction | Termination |
|---|---|---|---|---|
| P1 | shared | 93 | 0 (identical) | 0 (identical) |
| baseline set | shared | 93 | −0.1260 [−0.1782, −0.0265] | −0.1140 [−0.3175, +0.0011] |
| baseline set | with stop | 93 | −0.1224 [−0.1876, −0.0034] | −0.1445 [−0.3809, −0.0186] |
| hazard | shared | 69 | −0.0553 [−0.1181, −0.0086] | −0.0044 [−0.0149, +0.0093] |
| linear | shared | 69 | −0.0628 [−0.1290, −0.0038] | −0.0017 [−0.0149, +0.0222] |
| piecewise | shared | 69 | −0.1415 [−0.2416, −0.0728] | −0.0727 [−0.1253, −0.0435] |
| constant | shared | 69 | −0.1286 [−0.1849, −0.1003] | −0.0865 [−0.1168, −0.0685] |

## Step 2. Flame Design test days (PSI-10)

**Normal flames.** 29 tests on 3 test days, 87 cases.
- Every test is ethylene.
- 54 cases ended by extinction and 33 by operator termination.

**Inverse flames.** 20 tests on 1 test day, 60 cases.
- Every test is ethylene.
- All 60 cases ended by operator termination.

The endpoints were measured with `flame_design.py`, thresholds unchanged (status ok for all 49 tests).
Without the planned stop, every release answered every case. With the planned stop, F1 and ethylene as
ethane each refused 4 normal-flame cases, and F2 refused none. The table for that path is on the 83 cases
that all releases answer. All values are case means; columns are extinction / termination / sum.

| Family, path | Cases / tests / days | F2 (adopted) | F1 | Ethylene as ethane | Incidence baseline |
|---|---|---|---|---|---|
| normal, no stop | 87 / 29 / 3 | **0.0681** / 0.1451 / 0.2133 | 0.1147 / 0.1472 / 0.2620 | 0.0803 / 0.1412 / 0.2216 | 0.2547 / 0.1698 / 0.4244 |
| normal, stop | 83 / 29 / 3 | **0.0444** / 0.0168 / 0.0612 | 0.1111 / 0.0482 / 0.1593 | 0.0773 / 0.0358 / 0.1131 | 0.2544 / 0.1775 / 0.4320 |
| inverse, no stop | 60 / 20 / 1 | **0.0003** / 0.2743 / 0.2746 | 0.0003 / 0.2743 / 0.2746 | 0.0004 / 0.2616 / 0.2620 | 0.2532 / 0.4234 / 0.6766 |
| inverse, stop | 60 / 20 / 1 | **0.0001** / 0.0020 / 0.0020 | 0.0000 / 0.0398 / 0.0399 | 0.0000 / 0.0398 / 0.0399 | 0.2532 / 0.4234 / 0.6766 |

Paired difference, F2 minus the other release (session bootstrap, 95 %; 3 normal days, 1 inverse day):

| Family, path | F2 − F1: extinction / termination / sum | F2 − ethylene as ethane: extinction / termination / sum |
|---|---|---|
| normal, no stop | −0.0466 [−0.1076, −0.0026] / −0.0021 [−0.0052, 0.0000] / −0.0487 [−0.1128, −0.0026] | −0.0122 [−0.0290, +0.0002] / +0.0039 [−0.0000, +0.0094] / −0.0083 [−0.0196, +0.0002] |
| normal, stop | −0.0667 [−0.0978, −0.0078] / −0.0314 [−0.0497, 0.0000] / −0.0981 [−0.1177, −0.0078] | −0.0329 [−0.0477, −0.0029] / −0.0190 [−0.0497, +0.0087] / −0.0519 [−0.0974, −0.0029] |
| inverse, no stop | 0 / 0 / 0 (identical forecasts) | −0.0002 / +0.0127 / +0.0125 (one day, no interval) |
| inverse, stop | 0.0000 / −0.0378 / −0.0378 (one day, no interval) | 0.0000 / −0.0378 / −0.0378 (one day, no interval) |

**Inverse extinction scores.** All inverse test-day cases ended by operator termination, so the inverse
extinction scores measure only how little false extinction each release forecasts.

## Step 3. CFI test day (PSI-39): n-dodecane, one droplet

**This set is 1 test, 3 cases, 1 day.** No case was not at risk, and no release refused a case. A single
droplet is an anecdote, not a test of a fuel. These numbers are its score and carry no interval.

| Release | CRPS | MAE of median | Cov. 50 % | Cov. 90 % |
|---|---|---|---|---|
| `flex_dodecane.wsp7` W0 (zero-shot candidate) | 14.8 | 19.5 | 0.91 | 1.00 |
| root (the heptane law) | 39.7 | 54.8 | 0.54 | 0.74 |
| persistence | 106.5 | | | |
| classical D² law | 60.8 | | | |

**Paired, candidate minus comparator (µm, one day, no interval):**
- root −24.9
- persistence −91.7
- D² law −46.0

## Step 4. BASS-II test partition (PSI-25): fabric burns

**This set is 2 burns, 7 tip cases.** The tracks are the measurer's test tracks rated usable, converted to
the scorer's format (`runs/dev/media/fabric/CONVERSION.md`). The conversion left 2 held tracks out: 1 for
`quality failed`, 1 for a missing `conditions.fan_calibration`. No case or ignition track was refused.

| Release | Tip CRPS (sample widths) | Abs. error of median | Cov. 50 % | Cov. 90 % | Track ended, Brier | Ignition outcome, Brier (2 burns) |
|---|---|---|---|---|---|---|
| `bass_front.wsp` D1 (adopted) | 0.0839 | 0.1217 | 0.62 | 0.95 | 0.2066 | 0.0274 |
| `bass_front.wsp5` Q0 | 0.0839 | 0.1217 | 0.62 | 0.95 | 0.0291 | 0.5347 |
| `bass_front.wsp5` carry of D1 | 0.0839 | 0.1217 | 0.62 | 0.95 | 0.0110 | 0.0000 |

- **Tip.** The tip forecast is the same law in all three, so the paired tip-CRPS differences are 0 on both
  burns.
- **Ignition outcome.** With two burns, the Brier score of the ignition outcome is close to a description
  of those two burns. It is not an estimate of skill.

PSI-26 was not scored: it holds no fabric burn with conditions. Its test days stay unread.

## Step 5. FLEX-2 test days (PSI-68): arm F, rescaled series

The scale was pinned before the run (`runs/dev/media/droplet/SCALE_PINNED.txt`: 33.85 px/mm, 1.5 %
1-sigma). The held series read are the rescaled ones the marker names,
`runs/dev/media/droplet/flex2_series_test_v2/`.
- Of the 11 held series, the measurer's rule accepts 2, both n-decane.
- **n-octane has no usable series** (6 held: 4 not usable, 2 failed).
- n-decane: 2 usable, 1 not usable, 2 failed.

**The n-decane set is 2 tests, 6 cases, 1 day.** No case was not at risk, and no release refused a case.

| Release | CRPS | MAE of median | Cov. 50 % | Cov. 90 % |
|---|---|---|---|---|
| `flex_decane.wsp7` W0 (zero-shot candidate) | 80.6 | 101.3 | 0.55 | 0.62 |
| root (the heptane law) | 65.4 | 83.9 | 0.53 | 0.71 |
| persistence | 82.4 | | | |
| classical D² law | 25.4 | | | |

**Paired, candidate minus comparator (µm, one day, no interval):**
- root +15.2
- persistence −1.9
- classical D² law +55.1

Root minus persistence: −17.1.

## Rehearsals before the run (validation, already-known numbers)

- **Step 1.** D3 40.0 µm and D2 46.1 µm reproduced. F2 normal 0.1537 / 0.0961 and 0.1473 / 0.0798
  reproduced; F2 inverse 0.1288 / 0.0880 and 0.1349 / 0.0480 reproduced. The split by input matched the
  evaluator to 6e-17. No new validation seal was created.
- **Step 2.** The decision-input table of `flame_design_f2_result.md` reproduced, with a largest difference
  of 0.0.
- **Step 3.** The validation-day line of `cfi_transfer_result.md` reproduced, with a largest difference of
  0.0.
- **Step 4.** 12.9's train tracks reproduced: tip CRPS 0.0761, ignition Brier 0.0033 (D1) and 0.1469 (Q0).
  - The final code path was also run on the converted train tracks (`tracks_scorer/psi25/train`). It gave
    tip CRPS 0.0843 for D1, the carry and Q0.
  - That run had 4 burns with cases, not the 3 expected. The fifth track is a second video of one burn
    (T16), which the scorer counts as its own run. One track was unscoreable (fewer than 6 flame stills).
- **Step 5.** The n-decane validation-day line of `flex2_transfer_result_v2.md` reproduced on the rescaled
  series: candidate 72.4 and root 57.3 µm, 6 cases on 2 days.

## What did not go as planned

- **Fabric.** The first fabric plan could not have run: the measurer's raw tracks are in frame-width units
  that the scorer does not read. The run used the converted tracks. Step 4 then rests on 2 burns.
- **Data_ext.** P1 with the old planned stop (`runs/dev/data_ext`) was not scored. That root holds the stop
  for train and validation runs only.
- **Case sets differ by comparator.** The intersection of seals sets the case set of each comparison:
  - FLEX methanol: 65 cases in every comparison.
  - Heptane against W0: 42 of 43.
  - Inverse flames against the fitted baselines: 69 of 93.
- **Step 1, normal flames:** 2 sessions. **Step 2, inverse flames:** 1 day. **Steps 3 to 5:** 1 to 2
  tests or burns each.
- **Code changes during the run.** Two script changes were made after the go and before step 1, each
  rehearsed:
  - a comparator's failed forecast or comparison is recorded instead of stopping the step (no comparator
    failed);
  - step 4 reads the converted folder the list names.

  For step 5, the folder named by the scale marker replaces the list's older `test_series` key, as the
  list's note says.
- **Validation looks during preparation.** The rehearsals repeated validation looks that had already been
  taken (the evaluator logged those reveals as development, `final_test: false`). Step 5's rehearsal on the
  rescaled validation series repeated a score already in `flex2_transfer_result_v2.md`.

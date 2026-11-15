# CFI (PSI-39), arm F of PROTOCOL 12.7: n-dodecane, zero-shot, scored once

> Scored on 15 November 2026 by the delegated scoring and data agent (claude-opus-5-5), as registered in
> PROTOCOL 12.7 and its amendment (commit `a573d81`). Nothing fitted or changed. Aggregates only;
> the per-test rows below are of **train days** and are the study material of PROTOCOL 12.3. No test-day
> file was opened. Paths under `runs/dev/` are local working folders, not in git.

## The chain, re-checked with the merged binaries

`target-merged/release/ignis.exe` (sha256 `b03a35c8…`) and `wsp-dev.exe` (`16d0d381…`).
`tools/wsp/ext/transfer_droplet.py flex-check --tag merged` against `wsp-dev feedback --partition
train` on the FLEX n-heptane train split: `flex_heptane.wsp7` carry of P1
(`sha256:d87166c2…6f`), the wsp7 heptane root, `flex_heptane.wsp2` D2 and its D-squared law. 125 cases,
9 refused, 13 not at risk, for each. **Largest difference on any per-case metric: 0.0**, on both
prefix paths (the stored records, and the same runs rebuilt from the generic series format used for
CFI). Report: `runs/dev/data_all/work/transfer_droplet/flex/chain_check_merged.json`.

## What was scored

- **Tests:** the 7 registered n-dodecane free droplets with a processed series, inside the domain, off the
  test days: FLEX-1206, 1239, 1240, 1250, 1264 (train days 2017-03-09, 05-01, 05-05, 05-19) and two tests
  of validation day 2017-03-31. 21 cases (cutoffs 1, 3, 5 s; horizon 10 s); every case at risk.
- **Releases:** the candidate `flex_dodecane.wsp7` W0 (`sha256:70042961…7e74`) and its root
  `configs/models/flex/wsp7/dodecane/baseline-d2` (`sha256:fb5395d1…fd67`, "treat the fuel as heptane").
  Baselines computed by the chain: persistence (last diameter carried forward) and the classical
  D-squared law (D² fitted by least squares over the prefix diameters in [0, cutoff], extrapolated at the
  fitted rate; a rising D² gives rate 0). Both baselines are point forecasts, so their CRPS is their
  absolute error and they have no bands.
- **Inputs, fixed before scoring** (`tools/wsp/ext/cfi_cases.py`): NASA's `DropletData.txt` liquid
  diameter only, with the FLEX ingest's quality flags, under the FLEX time origin; O2 and He from the
  campaign spreadsheet, no CO2, the nominal pressure; fuel "n-Dodecane". No flame-camera series.
- **Refusals: none.** Both releases answered all 21 cases; none was unscored.
- Scored with `tools/wsp/ext/transfer_droplet_arms.py`; output `runs/dev/data_all/work/transfer_droplet/cfi/aggregate.json`.

## Scores

CRPS and MAE of the median in µm, case means; coverage = fraction of target points inside the band.

| Scope | Cases / tests / days | Release | CRPS | MAE of median | Coverage 50 % | Coverage 90 % |
|---|---|---|---|---|---|---|
| Pooled | 21 / 7 / 5 | candidate W0 | 239.9 | 270.0 | 0.37 | 0.63 |
| | | root (heptane law) | 259.1 | 302.8 | 0.29 | 0.58 |
| | | persistence | 431.5 | 431.5 | | |
| | | classical D² law | 707.6 | 707.6 | | |
| Train days | 15 / 5 / 4 | candidate W0 | 283.8 | 315.9 | 0.39 | 0.67 |
| | | root | 278.0 | 321.3 | 0.32 | 0.66 |
| | | persistence | 478.6 | 478.6 | | |
| | | classical D² law | 464.3 | 464.3 | | |
| Validation day | 6 / 2 / 1 | candidate W0 | 130.1 | 155.3 | 0.33 | 0.54 |
| | | root | 212.0 | 256.6 | 0.24 | 0.38 |
| | | persistence | 313.6 | 313.6 | | |
| | | classical D² law | 1315.6 | 1315.6 | | |

**Paired difference, candidate minus root (CRPS, µm, 95 % day bootstrap):**

| Scope | Cases / days | Mean difference | Interval | Days where the candidate is lower |
|---|---|---|---|---|
| Pooled | 21 / 5 | −19.3 | [−85.6, +58.5] | 3 of 5 |
| Train days | 15 / 4 | +5.8 | [−87.1, +89.6] | 2 of 4 |
| Validation day | 6 / 1 | −81.9 | one day, no interval | 1 of 1 |

Against the baselines, pooled: candidate minus persistence −191.6 [−212.0, −179.4], candidate minus
classical D² law −467.7 [−946.5, −129.9], root minus classical D² law −448.4 [−887.7, −116.1]; both
releases beat both baselines on all 5 days.

**Per train day, CRPS (µm):**

| Day | Tests | Candidate | Root | Persistence | Classical D² |
|---|---|---|---|---|---|
| 2017-03-09 | 1 | 255.2 | 194.7 | 456.5 | 588.4 |
| 2017-05-01 | 2 | 25.3 | 29.0 | 202.7 | 122.8 |
| 2017-05-05 | 1 | 1082.1 | 963.4 | 1261.4 | 1227.1 |
| 2017-05-19 | 1 | 31.1 | 173.8 | 269.7 | 260.6 |

As registered, CFI (7 tests) is reported and **no conclusion is drawn**. The pooled interval of
candidate minus root includes zero, and so does the train-day one. The bands are too narrow for both
releases: the 90 % band covers about 0.6 of the targets.

## The registered expectations (phys_droplet_fuels.md section 4), train-day tests only

What the five train-day tests show is taken from NASA's analysed values in the campaign spreadsheet
(sheet `DATA`, rows of those five tests only, read after the score): hot-flame rate `khot`, cool-flame
rate `kcool`, hot-flame end diameter, cool-flame end diameter, hot-flame burn time. All five are
n-dodecane, free, in air (21 % O2), D0 4.0 to 4.4 mm: FLEX-1206 and 1250 at 1 atm, 1239 and 1240 at
0.5 atm, 1264 at 1 atm with 29 % helium.

| Registered expectation | Expected | Train-day tests show |
|---|---|---|
| 4.2: in air at 1 atm, droplets below about 4.7 mm keep the hot flame and burn down with **no cool flame**; "if the 4 mm droplets in air lose their flame too, D_crit is below 4 mm and the compact-flame argument is wrong for this fuel" | hot flame kept, no cool flame | **all five lost the hot flame** at 3.5 to 3.9 mm (7 to 14 s) and **a cool flame followed in all five**. By the dossier's own test, D_crit is below 4 mm and the argument fails for n-dodecane |
| 4.1, CFI as flown: hot flame lost within 4 to 13 s, apparent rate 0.17 to 0.29 mm²/s at 1 atm (± 0.08), at most the cool-flame rate | lost early, slow | lost after 7.2 to 14 s (one just beyond the range). Apparent hot rate 0.38 at 1 atm in air (two tests), 0.32 and 0.35 at 0.5 atm, 0.43 with 29 % He: **above the range**, even with the one-sigma margin (0.37). It is close to the plateau value of the table (0.44 ± 0.07 at 3 mm) |
| 4.1: hot rate ≤ the cool rate that follows | k_hot ≤ k_cool | roughly equal: 0.379 / 0.385, 0.383 / 0.387 at 1 atm; 0.318 / 0.310, 0.351 / 0.311 at 0.5 atm; 0.426 / 0.464 with helium |
| 4.3: K_c of n-dodecane 0.24 ± 0.06 (3 mm), 0.26 (5 mm) at 1 atm in N2; no pressure dependence between 0.5 and 1 atm; +22 % per 20 % He | 0.25; 0.25; about 0.33 with 29 % He | 0.385 and 0.387 at 1 atm (+2 sigma); 0.31 at 0.5 atm, **lower than at 1 atm**; 0.464 with helium. **All above** |
| 4.3: D_ce/D0 0.37 in air at 1 atm, 0.57 at 21 % and 0.5 atm, × (1 + x_He) | 0.37; 0.57; about 0.48 | 0.12 and 0.10 at 1 atm (the cool flame burns much further down); 0.44 and 0.47 at 0.5 atm; 0.23 with helium |
| Reignition (no state in the forward) | | no reignition in any of the five |

In short: the hot flame of 4 mm n-dodecane droplets in air goes out, as the dossier says it should not
below 4.7 mm. The cool flame always follows. It burns about 1.5 times faster than typed, and at 1 atm
it burns down to about a tenth of D0. The typed hot rate for CFI as flown is too low.

## Anatomy of the train-day cases (for the study step, PROTOCOL 12.3)

Per-case file: `runs/dev/data_all/work/transfer_droplet/cfi/train_days_cases.jsonl` (15 rows: one per
train-day case, with its conditions, both releases' forecast quantiles, the target points and the
per-case metrics). Per test, from `anatomy_train.json`: median minus observed in µm by lead after the
cutoff (all three cutoffs together); mean absolute error against the half-width of the 90 % band; 90 %
coverage.

| Test (conditions) | Release | Bias 0–2 s | 2–5 s | 5–10 s | Mean abs. error | Half-width 90 % | Coverage 90 % |
|---|---|---|---|---|---|---|---|
| FLEX-1206 (1 atm, air) | candidate | +5 | +145 | +313 | 296 | 99 | 0.37 |
| | root | −26 | +37 | +81 | 244 | 157 | 0.59 |
| FLEX-1239 (0.5 atm) | candidate | +2 | +17 | +56 | 34 | 88 | 1.00 |
| | root | −8 | −15 | −9 | 48 | 83 | 0.96 |
| FLEX-1240 (0.5 atm) | candidate | +5 | +25 | +74 | 46 | 94 | 1.00 |
| | root | −4 | −3 | +22 | 44 | 84 | 0.92 |
| FLEX-1250 (1 atm, air) | candidate | +4 | +310 | +1208 | 1169 | 86 | 0.02 |
| | root | −27 | +206 | +984 | 1060 | 140 | 0.18 |
| FLEX-1264 (1 atm, 29 % He) | candidate | −46 | +20 | +39 | 36 | 111 | 0.96 |
| | root | −87 | −128 | −311 | 211 | 179 | 0.64 |

- **Centre against spread.** Where the error is large (1206, 1250) it is a centre error, many times the
  band's half-width. Where the error is small (1239, 1240, 1264) the candidate's band is wide enough.
- **Bias against lead.** The candidate's median runs high at every test, and the gap grows with lead
  time: it shrinks the droplet too slowly. That fits a hot rate typed too low and a cool-flame rate
  typed too low (above).
- **1250 dominates the train mean.** The forecast stays above the observed diameter by 1 mm at 5 to
  10 s. The train table has its cool flame ending at 0.4 mm. This test alone carries most of the
  train-day mean.
- **The root** (heptane rates) has a smaller bias at 1 atm in air. It is badly low with helium
  (1264), where the candidate is close.

## Not verified

- The CFI time base equals FLEX's (`DropletTime_sec` read as is under the FLEX time origin); not
  adjudicated against the igniter stamps.
- The expectations comparison uses NASA's analysed rates and diameters for the five tests, not values
  measured here.
- The classical D² law is weak at a 1 s cutoff, where the prefix is still in heat-up and its fitted rate
  is 0; it is a reference, not a competitor.

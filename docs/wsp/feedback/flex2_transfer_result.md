> **VOID (15 November 2026):** every diameter of the series scored here was 1.653 times too large (pixel scale assumed, not measured); the scale was pinned and the arms rescored once under the third amendment of PROTOCOL 12.7: see [flex2_transfer_result_v2.md](flex2_transfer_result_v2.md).

# FLEX-2 (PSI-68), arms H and F of PROTOCOL 12.7: scored once on batch 1

> Scored on 15 November 2026 by the delegated scoring and data agent (claude-opus-5-5), as registered in
> PROTOCOL 12.7 and its amendment (`a573d81`). Nothing fitted or changed. Aggregates only; the per-test
> rows below are of **train days** (study material for PROTOCOL 12.3). No validation-day test is
> reported alone, and no test-day series was opened (`flex2_series_test/` was not read). Paths under
> `runs/dev/` are local working folders, not in git.

## What entered the score

- **Series:** batch 1 (48 tests, `docs/wsp/atlas/splits/PSI-68-batch1.json`), measured blind from the
  backlit frames by the frozen tool (`runs/dev/media/droplet/FREEZE.json`;
  `FLEX2_MEASUREMENT_REPORT.md` exists, so the measurement was complete). **Only the 15 series the
  measurer's rule accepts (`quality = usable`) were scored:**
  - n-heptane: 8 of 10, all train, on 2 days (2013-03-08 and 2013-03-14);
  - n-octane: 1 of 12 (train, 2012-02-22);
  - iso-octane: 0 of 2;
  - n-decane: 6 of 24 (3 train on 2 days, 3 validation on 2 days).

  The other 33 series were rejected by the measurer, as not usable (19) or failed (14, mostly the
  tracker following a soot clump). The rejections fall heavily on the sooty fuels (n-octane, n-decane).
- **Conditions:** as written in each series from `flex2_tests.jsonl`: the gas reading of the day,
  chamber pressure before ignition, and the fuel from NASA's data record ("Heptane", "n-octane",
  "n-decane").
- **Time base:** t = 0 at the timeline's "AS Igniter Off/Retract Start", handed over under the FLEX
  time origin by declared equivalence, uncertain by about ±0.4 s.
- **Chain:** `tools/wsp/ext/transfer_droplet_arms.py` with `target-merged/release/ignis.exe`, the binary
  re-checked against `wsp-dev feedback` on FLEX train (largest difference 0.0; see
  `cfi_transfer_result.md`). Cutoffs 1, 3, 5 s, horizon 10 s, the evaluator's metrics.
- **Baselines:** persistence, and the classical D-squared law (least squares on the prefix; both are
  point forecasts, CRPS = absolute error, no bands).
- **Refusals:** none, by any release, on any case.
- **Not at risk:** 1 case of arm H and 3 of n-decane, where the series ended before a cutoff.
- Outputs: `runs/dev/data_all/work/transfer_droplet/{flex2_H,flex2_F_decane,flex2_F_octane}/aggregate.json`.

## A threat that decides the reading: the scale is not measured

The measurement report says the pixel scale is the literature's assumed field of view (50 mm). The
ruler check reads the needle-arm rods about 30 % wider than on FLEX. If that is the same rod, every
FLEX-2 diameter is about 30 % too large and every burning rate about 70 % too large. The rates seen in
the prefixes fit that: the classical D² fit gives 0.7 to 1.4 mm²/s for n-heptane at 1 atm in air,
against about 0.6 on FLEX. A forecast built for FLEX-sized rates then shrinks the droplet too slowly.
**Absolute diameters and rates of these series, and therefore every CRPS below, carry this unresolved
error.** It acts on every release, but not equally:
- the D² laws of the releases and the classical fit read the rate from the prefix;
- the adopted and new-fuel candidates type their rates.

## Arm H: n-heptane, the adopted model unchanged

8 tests, 23 cases, 2 days, all train. CRPS and MAE of the median in µm; coverage of the bands.

| Release | CRPS | MAE of median | Coverage 50 % | Coverage 90 % |
|---|---|---|---|---|
| `flex_heptane.wsp2` D2 (adopted) | 460.5 | 507.8 | 0.02 | 0.06 |
| `flex_heptane.wsp` W0 (zero-shot) | 247.7 | 304.8 | 0.05 | 0.22 |
| D-squared law of the wsp2 spec (`baseline-d2`) | 177.6 | 227.5 | 0.17 | 0.50 |
| persistence | 735.2 | 735.2 | | |
| classical D² law (chain) | 206.1 | 206.1 | | |
| `flex_heptane.wsp7` carry of P1 (reported only) | 472.7 | 518.8 | 0.02 | 0.05 |

**Per day (the rule of arm H), CRPS:**

| Day | Tests / cases | D2 | D-squared law | Persistence | W0 | carry of P1 |
|---|---|---|---|---|---|---|
| 2013-03-08 | 4 / 12 | 514.2 | 196.0 | 810.0 | 272.0 | 524.8 |
| 2013-03-14 | 4 / 11 | 401.8 | 157.5 | 653.5 | 221.1 | 415.7 |

**Verdict inputs.** The registration names 3 days; the accepted series cover 2. On both days:
- D2 is **higher than the D-squared law** (+282.9 µm pooled, day bootstrap [+244.4, +318.2]);
- D2 is lower than persistence (−274.7);
- D2 is higher than W0 (+212.8).

By the registered rule ("does not carry over if higher than the D-squared law on at least two days"),
**`flex_heptane.wsp2` D2 does not carry over** to FLEX-2 batch 1 as measured. The carry of P1 is 12.2
µm worse than D2 ([+10.6, +13.9]). The scale threat above applies in full.

## Arm F: new fuels, zero-shot, against the heptane law ("root")

| Fuel | Scope | Cases / tests / days | Candidate W0: CRPS / MAE / cov50 / cov90 | Root: CRPS / MAE / cov50 / cov90 | Persistence | Classical D² |
|---|---|---|---|---|---|---|
| n-decane | pooled | 15 / 5 / 4 | 202.1 / 241.5 / 0.09 / 0.23 | 66.8 / 91.5 / 0.37 / 0.67 | 356.8 | 107.3 |
| | train days | 9 / 3 / 2 | 261.3 / 307.4 / 0.06 / 0.13 | 74.0 / 102.2 / 0.35 / 0.72 | 477.6 | 122.3 |
| | validation days | 6 / 2 / 2 | 113.3 / 142.6 / 0.15 / 0.37 | 56.1 / 75.4 / 0.40 / 0.60 | 175.5 | 84.7 |
| n-octane | train (only) | 3 / 1 / 1 | 91.2 / 121.8 / 0.14 / 0.34 | 71.1 / 92.7 / 0.30 / 0.60 | 49.1 | 60.5 |
| iso-octane | | 0 | no usable series | | | |

**Paired difference, candidate minus root (CRPS, µm, day bootstrap):**

| Fuel | Scope | Cases / days | Mean | 95 % interval | Days where the candidate is lower |
|---|---|---|---|---|---|
| n-decane | pooled | 15 / 4 | +135.3 | [+57.2, +203.6] | 0 of 4 |
| | train | 9 / 2 | +187.3 | [+163.0, +236.0] | 0 of 2 |
| | validation | 6 / 2 | +57.2 | [+8.0, +106.4] | 0 of 2 |
| n-octane | train | 3 / 1 | +20.2 | one day | 0 of 1 |

**Verdict inputs.**
- **n-decane** has 4 days, so the rule applies. The candidate's CRPS is higher than the root's and the
  day-bootstrap interval excludes zero, so by the registered rule the typed ratios **do not add
  something** for n-decane on batch 1 as measured.
- The root also beats the classical D² law (66.8 against 107.3) and persistence (−290.0,
  [−432.6, −98.3]).
- **n-octane** (1 day) and **iso-octane** (no series) do not meet the rule's three days: reported, no
  conclusion.
- Same caveat: the candidate types a slower n-decane rate (0.49 mm²/s at 3 mm against heptane's 0.58).
  If the series' rates are inflated by the scale, that difference is exactly the one the error
  penalises.

## Anatomy of the train-day cases (PROTOCOL 12.3)

File: `runs/dev/data_all/work/transfer_droplet/flex2/train_days_cases.jsonl`. It holds 35 rows, one
per train-day case and arm (H 23, n-decane 9, n-octane 3). Each row carries its conditions, every
release's forecast quantiles, the target points and the per-case metrics. Per test below:
- the bias of the median (median minus observed, µm) by lead after the cutoff;
- the mean absolute error against the half-width of the 90 % band;
- the coverage of the 90 % band.

| Test | Release | Bias 0–2 s | 2–5 s | 5–10 s | Abs. error | Half-width 90 % | Cov. 90 % |
|---|---|---|---|---|---|---|---|
| E066C01 (heptane) | D2 | +96 | +374 | +778 | 443 | 62 | 0.06 |
| | D² law | +31 | +140 | +470 | 228 | 119 | 0.46 |
| E066C02 | D2 | +74 | +286 | +648 | 424 | 73 | 0.06 |
| | D² law | +24 | +111 | +268 | 172 | 120 | 0.52 |
| E066C03 | D2 | +200 | +789 | +1620 | 780 | 73 | 0.03 |
| | D² law | +75 | +341 | +1189 | 443 | 180 | 0.34 |
| E066C04 | D2 | +124 | +484 | +1133 | 670 | 75 | 0.04 |
| | D² law | +35 | +149 | +538 | 284 | 182 | 0.64 |
| E073C02 | D2 | +108 | +426 | +881 | 482 | 62 | 0.06 |
| | D² law | +36 | +162 | +565 | 254 | 126 | 0.43 |
| E073C03 | D2 | +69 | +268 | +609 | 399 | 72 | 0.06 |
| | D² law | +23 | +105 | +261 | 166 | 115 | 0.51 |
| E073C04 | D2 | +82 | +323 | +782 | 504 | 74 | 0.05 |
| | D² law | +27 | +129 | +349 | 219 | 129 | 0.44 |
| E073C09 | D2 | +253 | +776 | | 425 | 54 | 0.07 |
| | D² law | +112 | +571 | | 263 | 117 | 0.24 |
| A118D01 (decane) | candidate | +70 | +250 | +552 | 365 | 106 | 0.10 |
| | root | +19 | +72 | +151 | 108 | 122 | 0.74 |
| A118D02 | candidate | +54 | +177 | +344 | 183 | 83 | 0.20 |
| | root | +19 | +63 | +180 | 80 | 83 | 0.66 |
| A221D03 | candidate | +74 | +261 | +582 | 385 | 104 | 0.09 |
| | root | +23 | +89 | +195 | 129 | 118 | 0.70 |
| E053A01 (octane) | candidate | −21 | −80 | −187 | 122 | 95 | 0.34 |
| | root | −16 | −61 | −142 | 93 | 79 | 0.60 |

W0 and the carry of P1 are in `flex2_H/anatomy_train.json`. W0 lies between D2 and the D² law on every
test. The carry of P1 lies on D2.

- **Centre, not spread.** On every n-heptane and n-decane test the median runs high, and the gap grows
  with lead time: the observed droplet shrinks faster than any release forecasts. D2's error is 6 to
  10 times its band's half-width, so its band almost never covers. The D² laws, which take their rate
  from the prefix, err less, and their bands are about twice as wide.
- **The one n-octane test runs the other way** (median low), and persistence beats every forecast on it.
- **The sign and growth of the heptane and decane bias** are what a diameter scale about 30 % too large
  would produce: the rate is inflated by the square of the scale error. A study step should pin the
  FLEX-2 scale before it reads these errors as model errors.

## Not done or not verified

- The FLEX-2 pixel scale (assumed 50 mm field of view); the batch should be rescored once it is pinned.
- The 33 series rejected by the measurer, and iso-octane entirely; the third registered n-heptane day.
- The time-origin equivalence (±0.4 s).

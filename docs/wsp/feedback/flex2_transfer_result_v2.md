# FLEX-2 (PSI-68), arms H and F of PROTOCOL 12.7, rescored once on the pinned scale (third amendment)

> Scored on 15 November 2026 by the delegated scoring and data agent (claude-opus-5-5), as the one rescore
> of the third amendment of 12.7. Nothing fitted or changed; the same chain, releases, cutoffs and rules
> as the first report (`flex2_transfer_result.md`, now void). Aggregates only; per-test rows are of
> **train days** only (study material, PROTOCOL 12.3). `flex2_series_test_v2/` was not opened. Paths under
> `runs/dev/` are local working folders, not in git.

## What changed, and what entered

- **The scale.** The measurer pinned it at 33.85 px/mm, the FLEX optical scale
  (`runs/dev/media/droplet/SCALE_FLEX2.md`, `SCALE_PINNED.txt`), from the FLEX-2 requirements document
  and the needle arms in first frames. No droplet size, rate or forecast was involved. Every diameter of
  the first series was 1.653 times too large, so the v2 series are the first ones × 0.605. The stated
  uncertainty is 1.5 % (one sigma), worst case −7 %.
- **The series.** `runs/dev/media/droplet/flex2_series_v2/`, with the acceptance rule applied unchanged:
  the same 15 usable series as before.
  - n-heptane: 8 (train, 2 days)
  - n-decane: 6 (3 train on 2 days, 3 validation on 2 days)
  - n-octane: 1 (train)
  - iso-octane: 0
- **The chain.** Cases built by `tools/wsp/ext/flex2_cases.py`, scored by
  `tools/wsp/ext/transfer_droplet_arms.py` with `target-merged/release/ignis.exe`. That binary reproduces
  `wsp-dev feedback` on FLEX train with a largest difference of 0.0.
- **Not at risk:** 1 n-heptane case and 3 n-decane cases.
- **Refusals:** none, by any release, on any case.
- Outputs: `runs/dev/data_all/work/transfer_droplet/flex2_v2/{H,F_decane,F_octane}/aggregate.json`.

## Arm H: n-heptane, the adopted model unchanged

8 tests, 23 cases, 2 days, all train. CRPS and MAE of the median in µm.

| Release | CRPS | MAE of median | Coverage 50 % | Coverage 90 % |
|---|---|---|---|---|
| `flex_heptane.wsp2` D2 (adopted) | **20.9** | 25.0 | 0.88 | 1.00 |
| D-squared law of the wsp2 spec (`baseline-d2`) | 32.6 | 45.8 | 0.63 | 0.99 |
| persistence | 444.8 | 444.8 | | |
| classical D² law (chain) | 124.7 | 124.7 | | |
| `flex_heptane.wsp7` carry of P1 (reported only) | 20.9 | 24.3 | 0.90 | 1.00 |

**Per day (the rule of arm H), CRPS:**

| Day | Tests / cases | D2 | D-squared law of its spec | Persistence | W0 | carry of P1 |
|---|---|---|---|---|---|---|
| 2013-03-08 | 4 / 12 | 25.5 | 35.5 | 490.1 | 39.2 | 25.5 |
| 2013-03-14 | 4 / 11 | 15.9 | 29.5 | 395.4 | 31.6 | 16.0 |

**Verdict inputs.**
- D2 is **lower than both baselines on each of the two days scored**:
  - D2 minus the D-squared law: −11.7 µm pooled, day bootstrap [−13.6, −10.0];
  - D2 minus persistence: −423.9 µm.
- D2 is also lower than W0 (−14.6). The carry of P1 equals D2 (+0.0).
- The registered rule asks for this on each of three days. The third n-heptane day has no usable
  series, so the "carries over" condition holds on every day that exists, but on 2 days rather than 3.
- The coordinator decides how the rule reads with two days. The "does not carry over" condition (higher
  than the D-squared law on at least two days) is not met.

## Arm F: new fuels, zero-shot, against the heptane law ("root")

| Fuel | Scope | Cases / tests / days | Candidate W0: CRPS / MAE / cov50 / cov90 | Root: CRPS / MAE / cov50 / cov90 | Persistence | Classical D² |
|---|---|---|---|---|---|---|
| n-decane | pooled | 15 / 5 / 4 | 45.8 / 61.8 / 0.60 / 0.85 | 38.8 / 54.1 / 0.59 / 0.88 | 215.9 | 64.9 |
| | train days | 9 / 3 / 2 | 28.1 / 41.2 / 0.71 / 1.00 | 26.4 / 38.4 / 0.76 / 1.00 | 289.0 | 74.0 |
| | validation days | 6 / 2 / 2 | 72.4 / 92.9 / 0.45 / 0.62 | 57.3 / 77.6 / 0.33 / 0.71 | 106.2 | 51.3 |
| n-octane | train (only) | 3 / 1 / 1 | 324.5 / 395.2 / 0.05 / 0.11 | 110.2 / 136.8 / 0.40 / 0.67 | 29.7 | 36.6 |
| iso-octane | | 0 | no usable series | | | |

**Paired difference, candidate minus root (CRPS, µm, day bootstrap):**

| Fuel | Scope | Cases / days | Mean | 95 % interval | Days where the candidate is lower |
|---|---|---|---|---|---|
| n-decane | pooled | 15 / 4 | +7.0 | [−11.5, +34.8] | 2 of 4 |
| | train | 9 / 2 | +1.7 | [−6.8, +18.5] | 1 of 2 |
| | validation | 6 / 2 | +15.1 | [−21.0, +51.1] | 1 of 2 |
| n-octane | train | 3 / 1 | +214.3 | one day | 0 of 1 |

**Verdict inputs.**
- **n-decane** (4 days, so the rule applies): the interval includes zero. By the registered rule the
  typed ratios are **undecided** for n-decane.
- The candidate and the root both beat persistence (root −177.1, [−291.0, −31.4]).
- **n-octane** (1 day) and **iso-octane** (no series) do not reach the rule's three days: reported, no
  conclusion.

## What the stated uncertainties could do (arm H, run once each, a stated sensitivity, not a search)

| Variant | D2 | D-squared law | Persistence | D2 − D² law | Day 03-08: D2 / D² law | Day 03-14: D2 / D² law |
|---|---|---|---|---|---|---|
| as measured | 20.9 | 32.6 | 444.8 | −11.7 [−13.6, −10.0] | 25.5 / 35.5 | 15.9 / 29.5 |
| diameters × 0.985 | 23.3 | 31.7 | 438.1 | −8.4 [−10.4, −6.5] | 27.6 / 34.2 | 18.6 / 29.0 |
| diameters × 1.015 | 21.4 | 33.8 | 451.5 | −12.4 [−14.5, −10.4] | 26.6 / 37.0 | 15.7 / 30.2 |
| times − 0.4 s | 20.3 | 27.9 | 437.6 | −7.6 [−10.1, −5.3] | 24.4 / 29.8 | 15.9 / 25.9 |
| times + 0.4 s (24 cases) | 21.0 | 37.0 | 434.4 | −16.0 [−19.1, −13.0] | 25.9 / 38.9 | 16.1 / 35.1 |

- Within ±1.5 % of scale and ±0.4 s of origin, D2 stays below the D-squared law and persistence on both
  days. The margin narrows to about 8 µm in the least favourable variant.
- The time shift moves the classical D² fit most (89 to 236 µm), because its rate is read from a prefix
  partly in heat-up.
- The worst-case scale error (−7 %) was not run. It is about five times the one-sigma step; the D2 margin
  shrinks by about 3 µm per 1.5 % toward smaller diameters, so a −7 % error could plausibly close it.

## Registered expectations (phys_droplet_fuels.md section 4), train-day tests only

Rates are measured here (`tools/wsp/ext/flex2_rates.py`, `flex2_v2/rates_train.json`) from the usable
frames of the v2 series:
- K_hot: −d(D²)/dt by least squares from 1.5 s over the first 40 % of the record;
- K_late: the same over the last 40 %;
- D0: the median over 0.5 to 1 s.

| Fuel (train-day tests) | Expected (section 4) | Measured |
|---|---|---|
| n-heptane, air, 1 atm, D0 1.85–3.6 mm (8 tests) | K 0.62 at 2 mm, 0.58 at 3 mm | K_hot 0.61 (1.85 mm), 0.57 (2.25 mm), 0.50–0.54 (2.6–3.1 mm), 0.46–0.52 (3.3–3.6 mm). Close at 2 mm, 10–20 % below at 3 to 3.6 mm |
| n-heptane, radiative loss and cool flame (D_crit 3.7 mm in air; K_c 0.345) | below 3.7 mm the droplet burns down | 5 of 8 (D0 ≤ 3.1 mm) burn at a steady rate to the end of the record. The 3 largest (3.3–3.6 mm) slow late: K_late 0.37 on one (≈ K_c, a cool flame), 0.12 on two (≈ the flameless evaporation of 4.3). Loss just below the expected D_crit, within its uncertainty |
| n-decane, 16.8–17 % O2, 1 atm, D0 3.1–3.8 mm (3 tests) | K 0.49 at 3 mm in air, −25 % for −5 % O2: about 0.37 | K_hot 0.35–0.38: **as expected** |
| n-decane, cool flame (D_crit 3.0 mm at 18 % O2; K_c 0.28 ± 0.06) | hot flame lost, cool flame follows | K_late 0.29–0.34, a mild drop from K_hot; no distinct break. Consistent with a cool flame at the upper end of K_c, not shown |
| n-octane, air, 1 atm, D0 2.8 mm (1 test) | K about 0.55 | K 0.035: the series barely shrinks (D/D0 0.975 after 21 s). The droplet does not burn like a flamed droplet over the record; the expectation cannot be compared |

## Anatomy of the train-day cases (PROTOCOL 12.3)

File: `runs/dev/data_all/work/transfer_droplet/flex2_v2/train_days_cases.jsonl`. It holds 35 rows, one
per train-day case and arm (H 23, n-decane 9, n-octane 3). Each row carries its conditions, every
release's quantiles, the target points and the per-case metrics. Per-test anatomy:
`flex2_v2/{H,F_decane,F_octane}/anatomy_train.json`.

Median minus observed (µm) by lead after the cutoff; mean absolute error against the 90 % half-width:

| Test | Release | Bias 0–2 s | 2–5 s | 5–10 s | Abs. error | Half-width 90 % | Cov. 90 % |
|---|---|---|---|---|---|---|---|
| E066C01 (heptane, 3.1 mm) | D2 | −2 | −5 | −9 | 6 | 96 | 1.00 |
| E066C02 (3.5 mm) | D2 | −2 | −6 | −31 | 18 | 104 | 1.00 |
| E066C03 (2.25 mm) | D2 | +13 | +62 | +177 | 72 | 178 | 1.00 |
| E066C04 (2.6 mm) | D2 | −7 | −32 | −60 | 38 | 172 | 1.00 |
| E073C02 (2.9 mm) | D2 | −4 | −12 | −16 | 11 | 104 | 1.00 |
| E073C03 (3.6 mm) | D2 | −3 | −7 | −29 | 17 | 99 | 1.00 |
| E073C04 (3.3 mm) | D2 | −3 | −7 | −14 | 10 | 114 | 1.00 |
| E073C09 (1.85 mm) | D2 | +16 | +39 | | 24 | 94 | 1.00 |
| A118D01 (decane) | candidate | +9 | +29 | +55 | 38 | 139 | 1.00 |
| | root | −6 | −20 | −57 | 51 | 126 | 1.00 |
| A118D02 | candidate | +10 | +28 | +57 | 30 | 88 | 1.00 |
| | root | −4 | −19 | −97 | 50 | 95 | 1.00 |
| A221D03 | candidate | +12 | +41 | +85 | 58 | 135 | 1.00 |
| | root | −2 | −3 | −14 | 23 | 117 | 1.00 |
| E053A01 (octane) | candidate | −66 | −253 | −612 | 395 | 188 | 0.11 |
| | root | −24 | −89 | −210 | 137 | 100 | 0.67 |

The three largest structured errors on the train days:

1. **n-octane E053A01: a droplet that barely shrinks.** Every forecast burns it down: the candidate is
   612 µm low at 5–10 s, the root 210 µm. Persistence and the classical D² fit are best. Neither the
   candidate nor the root has a state for a droplet that is not burning at the cutoff.
2. **n-decane, opposite signs of candidate and root.** On all three tests the candidate runs high (+55
   to +85 µm at 5–10 s: its typed rate is too slow) and the root runs low (−14 to −97 µm: the heptane
   rate is too fast). The measured rate (0.35–0.38 at 17 % O2) lies between them. This is a centre error
   of each, about half of the band's half-width.
3. **n-heptane, small droplets fast and large ones slow, and bands too wide.** D2 runs high on the two
   smallest droplets (+177 µm at 5–10 s on E066C03 at 2.25 mm; +39 on E073C09 at 1.85 mm). It runs
   slightly low on the larger ones, so the rate's size dependence is too weak. And D2's 90 % band half-width is 4 to
   16 times its absolute error on 7 of 8 tests (2.5 on E066C03): coverage 1.00 at 90 % and 0.88 at 50 %, so the band is
   much too wide for FLEX-2.

## Not done or not verified

- The worst-case scale error (−7 %) was not run.
- The 33 series the measurer rejected, iso-octane, and the third registered n-heptane day.
- The time-origin equivalence itself (±0.4 s is the measurer's statement; the sensitivity above uses it).
- The cool-flame readings in the expectations table are inferred from changes of the D² slope alone.

# The earlier BASS campaign (PSI-26): fabric burns measured from the public stills, verified by a second reader

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. PSI-26 is outside every partition and was read in full; of PSI-25 only train material and the fan calibration workbook were opened.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# PSI-26 (BASS-I) fabric burns: verification, completion and report

The interrupted agent's eight tracks hold up: I re-derived seven of them independently and the positions, times and assigned flows agree. One result was wrong (opposed leading-edge speeds in test 3) and is corrected. I tracked the two burns it had left, and rebuilt the results, quench and inventory files.

One thing differs from the brief: **PSI-26 has no video of any fabric test.** All 2012 raw data are zips of Nikon NEF stills; the 114 mp4 files are all from April–May 2013 (PMMA, wax, candles). So the tracks come from top-view stills, there is no overlay, and **the fan history is not recorded anywhere**. Fan-change times in the tracks are inferred from the flame's response.

## 1. Inventory

All tests ran in cabin air (O2 21.2–22.7 %, not vitiated). "Flow" is the as-run AIR reading for the fan setting, which equals the flow in the BASS-I team's Table I (Ferkul et al. 2013). R = number of flow restrictors.

| Test (as-run) | GMT 2012 | Material | O2 % | Direction | FAN → flow (cm/s), R | As-run outcome | Stills (zip, NEFs) | Tracked |
|---|---|---|---|---|---|---|---|---|
| 1 (1) | 090 | SIBAL 2 cm | 22.1 | concurrent | 66 → 52: 10, 5; R2 | steady spread, shrinks at 5, out at burnout | none in PSI-26 | no |
| 7 (4) | 096 | SIBAL 2 cm | 21.9 | concurrent | 66: 10; R2 | ~40 s burn | none | no |
| 8 (5) | 096 | SIBAL 2 cm | 21.9 | concurrent | 52: 5; R2 | ~60 s burn | none | no |
| 4 (12) | 131 | SIBAL 2 cm | 21.6 | concurrent | 95: 22; R2 | long flame, not steady, 26 s burn | 05_11, 38 (+16 second camera) | yes |
| 10 (13) | 131 | SIBAL 1 cm | 21.6 | concurrent | 62: 11; R2 | huge ignition kernel, then dim | 05_11, 64 (+11) | yes |
| 12 (14) | 131 | Nomex blend 2 cm | 21.6 | concurrent | 75: 15; R2 | burned 2 s, died at igniter-off | 05_11, 62 (+4) | no spread |
| 12-R, 12-R2 (24) | 157 | Nomex blend 2 cm | 21.2 | concurrent | 60: 9; R2 | no ignition | 06_06, 52 + 45, not downloaded | no |
| 5 (37) | 180 | SIBAL 2 cm | 22.0 | concurrent | 50, 46, 42, 41: 5, 3, 1.5, 1; R2 | extinguished at fan 41, ~2.5 cm left | 06_29, 83 (+23) | yes |
| 14 (38) | 180 | Nomex III | 22.0 | concurrent | 74: 14.5; R2 | spread in centre only, out ~1 cm from end | 06_29, 63 (+21) | yes |
| 5b (44) | 181 | SIBAL 2 cm remnant of test 5 | 21.9 | opposed | 62: 10; R2 | top side only, quenched 5 mm from end | 06_30, 46 | yes (new) |
| 6 (45) | 185 | SIBAL 2 cm | 22.2 | opposed | 62 (→50 at the end): 9; R2 | steady spread, burned out whole sample | 07_04, 83 | yes (new) |
| 2 (54) | 201 | SIBAL 2 cm | 22.7 | concurrent | 51 → max: 9, 44; R0 | did not blow out | 07_20, 62 | yes |
| 3 (55) | 201 | SIBAL 2 cm | 22.7 | opposed | 51, 71, max: 10, 20, 44; R0 | did not blow out, smaller at max | 07_20, 76 | yes |
| 15 (56) | 201 | Nomex III | 22.5 | concurrent | 91: 30; R0 | no ignition, igniter on throughout | 07_20, 76 (19 downloaded) | no |
| 13 (57) | 201 | Nomex blend 2 cm | 22.5 | concurrent | 61: 15; R0 | brief ignition, out in ~3 s | 07_20, 41 | no spread |
| 11 (65) | 222 | SIBAL 1 cm | 21.5 | concurrent | 98: 20; R2 | nice burn, igniter on twice | 08_09, 76 | yes |
| 9 (66) | 222 | SIBAL 2 cm | 21.6 | opposed | 65, 56, 48, 43, 41, 40: 10, 6, 4, 2, 1.5, 1; R2 | kept going to end of sample | 08_09, 104 | yes |

- **Views:** one top view (Nikon, 4288 × 2848, about 1.1–1.25 s per still). The day zips are 2.8–5.9 GB with roughly 12 MB per NEF; only the embedded JPEG previews were fetched.
- **Second camera:** zips `2012_05_10` and `2012_06_28` hold a hand-held oblique view on a camera set to GMT. It is unusable for positions but good for timing.

## 2. Comparability with the BASS-II train burns

- **Duct and fan:** the same SPICE duct (76 × 76 mm), fan and restrictor plates, according to both campaigns' documents. This cannot be checked from images.
- **Fabric and holder:** the same SIBAL fabric (75/25 cotton/fibreglass), and a woven Kanthal igniter. The lit band is 0.173 of the frame width in both campaigns (743–748 px of 4288 here, 55.6–56.7 of 324 in train), so framing matches.
- **Window length differs:** 4.30 w here (4.22 in test 2) against 4.52–4.65 w in the train tracks. Whether that is a longer holder or a different origin convention cannot be established.
- **Millimetres are not established.** Thread pitch gives about 21.5 mm per width (first agent, unchecked), with window width on the frame drawing reported as 22 mm (also unchecked). The published mm values for test 10 match only if 1 w ≈ 10.0–10.6 mm for the 1-cm sample, a 13–20 % disagreement. Widths are robust; mm are not.
- **Views and overlay:** train used 324-px thumbnails plus duct video with overlay. Here there are full-resolution stills (1 px ≈ 0.0013 w) and no duct view, overlay, AIR or RAD.
- **Time origin:** igniter-off is judged from coil colour between two stills, ±0.5–0.7 s, not from a video frame.
- **Oxygen:** 21.2–22.7 % here against 16.4–18.8 % in train. D1's oxygen factor (1.20–1.31 here) is an extrapolation.
- **Igniter interval:** 2.3–21 s here, and twice in test 11.
- **Fan-to-flow calibration:** PSI-26 contains none for 2012. The only numbers are the as-run AIR readings, which are whole cm/s, with 1 cm/s the lowest measurable.
  - They are not monotonic across days (FAN 62 → 9, 10 or 11; 95 → 22 but 98 → 20), so I estimate ±1–2 cm/s.
  - Against the BASS-II GMT 100 block at the same FAN number they read 0–30 % lower (50: 5 vs 6.5; 95: 22 vs 25.3; 98: 20 vs 26.8; no restrictor, max: 44 vs 55).
  - That is inside BASS-II's own day-to-day spread (2 restrictors, FAN 72 → 7 to 16 across days).
  - Both flows are carried in the files (`u_cm_s`, `u_bass2_cal_cm_s`).

## 3. Verification of the interrupted agent's tracks

I wrote an independent measurement (`verify/remeasure.py`): gradient-based geometry, column projection instead of connected components, own background.

| Check | Result |
|---|---|
| Flame positions, blue tip / base, compared in pixels | medians ≤ 0.005 w; 90 % within 0.025 w / 0.015 w on tests 5 (n = 44), 10 (43), 11 (20), 3 (61), 9 (66) |
| Sooty flames (tests 4, 2) | tip agrees within 0.03 w; base is definition-dependent, up to 0.24 w in a few stills |
| Phase speeds | test 5: 0.0587 vs 0.0591 and 0.048–0.051 vs 0.0485 w/s; test 10: 0.1727 / 0.1691 vs 0.1715 / 0.169; test 11 within 2 %; test 9 within 5 % |
| External check, test 10 | published tip 1.71 t, base 1.68 t (mm, s) equals the measured 0.1715 and 0.169 w/s at 10 mm per width |
| Band width | within 1 % on seven burns |
| Origin (`x_up`) | lies 0.00–0.05 w inside the coil's outer edge on tests 5, 10, 4, 2, 3, 9; test 11 may be 0.06–0.10 w too far out |
| Igniter-off | confirmed by eye on tests 5, 10, 3, 11, 2, 4 and by coil colour on 9, 14, all within the stated ±0.5–0.7 s; test 5 could be up to about 1 s earlier |
| Second camera | burn duration of test 5 is 57.1 s there against 57.8–59.0 s here; end times of tests 4, 10, 14 and igniter-off of 14 agree within its 2-s cadence |
| Flows and directions | match the as-run matrix and Table I; BASS-II comparison values recomputed from the PSI-25 calibration workbook and correct |

**Corrected:**
- **Test 3, opposed leading-edge speeds.** The first agent used the any-colour edge, which includes a glow up to 0.25 w ahead of the flame. Its 0.069 / 0.064 / 0.041 w/s become 0.060 / 0.065 / 0.050 w/s by the blue edge. `results.json` now carries both definitions for every phase.
- **Test 13.** Its quench entry was "as-run only"; the stills show a flame for about 3 s with the igniter on.
- **Inventory.** The `video` field was null and is now explicit.

**Taken over unchecked:**
- Test 14's flame extents and geometry (borrowed from test 5; my automatic geometry failed on it).
- The thread-pitch scale and the 22-mm window figure.
- The EXIF parsing, checked only against file-name seconds.
- The contents of the nine 2012 zips for non-fabric days.

## 4. Results per burn

Speeds are in widths per second, as differences between 3-still medians at the window ends. D1 is evaluated only, at the as-run flow and (in brackets) at the BASS-II-calibrated flow, with the card's ×1.83 factor for 1-cm samples.

**Concurrent**

| Test | Flow phase (window, s) | Tip | Base | Length (w) | D1 | Tip / D1 | Base / D1 |
|---|---|---|---|---|---|---|---|
| 5 | 5 cm/s (17.0–34.5) | 0.0572 | 0.0591 | 0.86 | 0.0666 | 0.86 (0.73) | 0.89 (0.75) |
| 5 | 3 cm/s (38.0–49.6) | 0.0387 | 0.0485 | 0.61 | 0.0506 | 0.76 (0.60) | 0.96 (0.75) |
| 5 | ~1 cm/s (50.8–54.6), not settled | −0.007 | 0.0535 | 0.59 → 0.31 | 0.0347 | — | 1.54 (1.31) |
| 4 | 22 cm/s (base 3.6–19.4; tip 1.5–9.9) | 0.2555, accelerating | 0.2042 | 1.6 → 3.2, growing | 0.1971 | 1.30 (1.15) | 1.04 (0.92) |
| 2 | 9 cm/s (1.6–9.9), still growing | 0.114 | 0.078 | 1.04 | 0.1034 | 1.10 | 0.75 |
| 2 | ramp to 44 (14.1–26.7) | off picture | 0.167 | up to 3.1 | flow unknown | — | — |
| 10 (1 cm) | 11 cm/s (18.5–37.7) | 0.1715 | 0.169 | 1.52 | 0.2043 | 0.84 | 0.83 |
| 11 (1 cm) | 20 cm/s (2.7–13.7) | 0.2265 | 0.2253 | 1.93 | 0.3298 | 0.69 (0.53) | 0.68 (0.53) |
| 14 (Nomex III) | 14.5 cm/s (1.7–32.1) | 0.071 | 0.086 | 0.93 → 0.39 | SIBAL law, scale only | 0.50 | 0.60 |

**Opposed** (blue leading / trailing edge; D1 not applicable)

| Test | Flow (cm/s) | Leading | Trailing | Blue length (w) |
|---|---|---|---|---|
| 3 | 10 / 20 / 44 | 0.059 / 0.066 / 0.052 | 0.056 / 0.073 / 0.055 | 0.66 / 0.56 / 0.34 |
| 9 | 10 / 6 / 4 / 2 / 1.5–1 | 0.055 / 0.051 / 0.043 / 0.038 / 0.027 | 0.053 / 0.048 / 0.043 / 0.037 / 0.032 | 0.87 / 0.81 / 0.69 / 0.59 / 0.43 |
| 6 | 9 (two halves) | 0.054 / 0.062 | 0.058 / 0.060 | 0.88 / 0.83 |
| 5b | 10, remnant | 0.016 | 0.085 | shrinking |

**Fan changes** (all times inferred from the flame):
- **Test 5, 35.1 s (5 → 3 cm/s):** tip retreats 0.065 w, stalls 3–4 s, passes its old position at +4.4 s. Length goes 0.88 → 0.68 w in 2 s and reaches 0.58 by +9 s. The base never slows.
- **Test 5, 50.2 s (3 → 1.5/1 cm/s):** tip retreats 0.06 w and never advances again. Base continues at 0.053 w/s. Quench follows 4.4–5.7 s later.
- **Test 2, ~10.5 s (ramp up):** no retreat. Length goes 1.12 → 1.40 w in 1.5 s and 2.4 w by +8 s.
- **Opposed (tests 3, 9):** never a retreat or stall at 1.2-s resolution. Length resettles within 2–6 s.

**First ten seconds after igniter-off** (tip / base, w):
- Test 5: 0.79 → 1.38 / 0.01 → 0.42. Length overshoots to 1.05 at 8 s and settles to 0.85–0.89 by about 15 s.
- Test 4: 1.89 → 4.38 / 0.06 → 1.44.
- Test 2: 0.91 → 2.06 / 0.22 → 0.95.
- Test 10: 0.87 → 2.38 / −0.03 → 1.08; steady only after about 18 s.
- Test 11: the flame is already at 4.4 / 2.5 w at t = 0 because of the long igniter interval.
- Test 3: 0.79 → 1.28 / −0.09 → 0.47.
- Test 9: 0.97 → 1.38 / 0.12 → 0.55.
- Test 6: 1.63 → 2.14 / 0.74 → 1.23, having spread during 21 s of igniter.

**Ends:**
- Test 5 quenched at 55.2 ± 0.6 s, at about 1 cm/s and 22.0 % O2, with its base at 2.88 w, 1.43 w short of the window end.
- Test 14 went out at 32.6–33.7 s, at 14.5 cm/s, 0.57–0.69 w short.
- Test 5b went out at 13–15 s, 0.30 w short of the closed end.
- All others reached the end: test 4 at 21.0 s, 2 at 28.3 s (at or near 44 cm/s), 10 at 48.3 s, 11 at 22.0 s, 3 at 68.6 s, 9 at 82.8 s, 6 at about 46 s (leading edge).

**Published BASS-I fits, for context** (ASGSR 2012 slides, base speed in mm/s, U in cm/s): 2 cm: V = 0.56 + 0.20 U; 1 cm: V = 0.07 + 0.15 U. At about 10 cm/s the 2-cm concurrent speed is 2.55 mm/s against 1.21 mm/s opposed.

## 5. Quench, blow-off and failure-to-spread events

| Test | Sample, direction | Flow (cm/s) | O2 % | Event | Evidence |
|---|---|---|---|---|---|
| 5 | SIBAL 2 cm, concurrent | ~1 (FAN 42 → 41); settled flames at 5 and 3 | 22.0 | quench mid-sample, about 5 s after the last reduction | stills (verified) + as-run |
| 9 | SIBAL 2 cm, opposed | down to 1 (FAN 41, 40) | 21.6 | no quench; reached the end | stills (verified) |
| 2 | SIBAL 2 cm, concurrent | 9 → 44 | 22.7 | no blow-off | stills (verified) |
| 3 | SIBAL 2 cm, opposed | 44 | 22.7 | no blow-off; flame smaller | stills (verified) |
| 5b | SIBAL remnant, opposed | 10 | 21.9 | went out 0.30 w short of the closed end; confounded (pre-burned, one-sided, frame heat loss) | stills (new) |
| 14 | Nomex III, concurrent | 14.5 | 22.0 | narrowed and went out before the end | stills (first agent) + second camera timing |
| 12 | Nomex blend, concurrent | 15 | 21.6 | no self-sustained flame; glow gone within 1–3.5 s of each igniter-off | stills (checked) |
| 13 | Nomex blend, concurrent | 15 | 22.5 | flame for about 3 s with igniter on, then none | stills (checked) |
| 12-R, 12-R2 | Nomex blend, concurrent | 9 | 21.2 | no ignition | as-run only |
| 15 | Nomex III, concurrent | 30 | 22.5 | no ignition | as-run only |
| 4, 10, 11, 6 | SIBAL, concurrent and opposed | 22 / 11 / 20 / 9 | 21.5–22.2 | burned to the end | stills |
| 1, 7, 8 | SIBAL 2 cm, concurrent | 10 → 5 / 10 / 5 | 21.9–22.1 | burned to the end | as-run only |

The team's paper puts concurrent extinction "less than 1 cm/s"; its Table I lists the last setting of test 5 as 1 cm/s.

## 6. Review aid: what to re-check first

Scripts are in `runs/dev/research5/bass1/verify/` unless stated; run them with `python -I`.

1. **Test 5 fan-change times and FAN assignment** (35.1 and 50.2 s; three reductions, two responses). Look at stills 48–50 and 60–62 of `S180_T5` (camera 16:15:24.9–27.4 and 16:15:40.0–42.5), and `area_px` in `tracks/BASS1_GMT180_T5.json`.
2. **Test 5 igniter-off.** Stills 19–21 (16:14:48.57, 49.75, 51.08) in `work/T5_ign.png`.
3. **Test 5 quench.** Stills 61–65; `remeasure.py S180_T5`.
4. **Flow at each phase.** As-run AIR against the BASS-II calibration; the D1 ratio moves by up to 25 % on this choice.
5. **Test 3 leading edge.** `remeasure.py S201_T3 --mirror`, then `compare.py S201_T3`; compare `tip_w` with `blue_tip_w`.
6. **Origin per burn.** `coil.py LABEL IDX [--mirror]`; test 11 with still 3, test 14 unverified.
7. **Test 6 (new).** Coil and igniter-off in `T6_coil.jpg` (stills 9, 26, 27); `x_up` = 3740 px mirrored.
8. **Test 2 ramp start** (10.5 s) and its sooty base.
9. **Test 10 mm scale** against the published 1.71 / 1.68 mm/s.
10. **Test 5b origin**, set at the first igniter glow: `T5b_frames.jpg`.

## What could not be measured

- Fan, AIR and RAD time series, and any duct or edge view: no 2012 video exists in PSI-26.
- Tests 1, 7 and 8: no images of GMT 90 or 96.
- Millimetre scale.
- Final O2 (only one value per test).
- The flow during test 2's ramp.
- Which of test 5's reductions each response belongs to.
- Test 14's geometry.
- The stills of 12-R, 12-R2 (not downloaded) and test 15 (19 of 76 downloaded) were not examined.

## Files written

Everything is in `D:\dev\SpaceApps-Ardealu\runs\dev\research5\bass1\`.

- `results.json`, `quench_table.json`, `inventory.json` — rebuilt; the first agent's originals are kept as `verify\first_agent_*.json`.
- `tracks\BASS1_GMT185_T6_opposed.json` and `tracks\BASS1_GMT181_T5b_opposed_remnant.json` — new.
- A `verification` block added to all ten track files; no measurement in the first agent's eight was changed.
- `scripts\config.json` and `scripts\meta.json` — two entries added.
- `work\results_table.txt`, plus `work\meas_*`, `results_*` and `measlog_*` for the two new burns.
- `verify\` — scripts (`remeasure.py`, `compare.py`, `coil.py`, `igniter.py`, `speeds_check.py`, `build_results.py`, `add_tests.py`, `run_fetch_second.sh`), `re_*.json`, `results_extra.json` and check images.

Downloads went to `D:\dev\SpaceApps-Ardealu\runs\dev\nasa\PSI-26\stills\` (`C2_*` folders and `S201_T15`). No git-tracked file was modified.

## Bytes downloaded

I downloaded 308,028,080 bytes (216 sequential range requests on three PSI-26 zips), against the 3 GB cap. The first agent's log shows 772,449,958 bytes before that. I stopped my fetch job before it reached tests 12-R and 12-R2.

## PSI-25 material opened

- The two train documents in `docs\wsp\feedback\` (tests T5, T6, T16, T10).
- The train track file `T5.json`; the other three were listed by name only.
- The dataset-level fan calibration workbook, read in full (calibration blocks by day, no test rows).
- `data\work\records\bass\experiments.jsonl` was loaded by a script that printed one PSI-26 record only; no PSI-25 record was displayed or used.

Not opened: the PSI-25 ReadMe, test matrices and experimental table, any other PSI-25 test, the model card, `holdout`, `val_sealed`, `data_ext` and `dev_eval.py`.

The PSI-26 as-run workbook has a sheet listing 2014 data-folder dates and data types; it holds no results. The PSI-26 document texts I searched contain no BASS-II results.

No helper agents were launched. Nothing was fitted.

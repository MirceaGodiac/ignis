# Plastics quench speed: the one revision after the final run (PROTOCOL 13.8)

Author: `claude-fable-5-2`, 15 November 2026. **Typed after the final run with every partition
seen; in-sample on all data; no held-out score exists.** Parent: `bass4` (`sha256:ceefc15d…`). Read:
`runs/dev/gen7/study/bass_test_quench_cases.md`, the test-day cases of
`runs/final/gen7/final/local/transfer_bass1/` with the operators' words from the workbook and the
report appendix, the development slabs of `phys_bass_study.md`. One candidate run, once.

## 1. What the test days show

17 scored quench burns, total loss of bass4 26.2 cm/s.

| Burn | What it is | Outcome as coded | bass4 median | Loss | Share |
|---|---|---|---:|---:|---:|
| O13a | 12.7 mm rod, concurrent, 15.9 % O2 | 17.9 to 40.8 cm/s | 1.47 | 16.20 | 62 % |
| M10 | slab 4 mm, two sides, **1 cm wide**, 20.1 % | 8.1 to 10.1 | 1.21 | 6.66 | 25 % |
| M15 | slab 4 mm, one side, 2 cm wide, 21.0 % | 20.1 and above | 14.7 | 2.56 | 10 % |
| M19 | slab 4 mm, two sides, 2 cm wide, 19.0 % | 5.0 to 7.0 | 5.2 | 0.50 | 2 % |
| 4 rods, 4 films, 5 spheres | | 0 to 2.1 | 1.2 to 1.4 | 0.33 together | 1 % |

**The floor held.** Thirteen rods, films and spheres of days never seen cost 0.025 per case, as on
the development days (0.01 to 0.03).

**O13a is not a blow-off and not a quench at 18 to 41 cm/s. It is a floor quench, miscoded.** The
row of the workbook reads: fan cell `1.3, 0.7, 0.45, 0.36, 0.32, 0.31` (pots); air cell
`49,44,42,41,41,40.5 (5, 2, 0)`; words: "Ignited and burned at 5 pot settings, flame
self-extinguished after 3+ min at pot 0.31". On that day the operators wrote the fan displays in
the air cell and the air readings in the bracket (the rows O5a, O5b and O13b of the same day do the
same: `5,10,5, 2, 0 (49, 62, 49, 44, 42)`, `49, 5 cm/s`, `7, 4? (49, ?)`). The coding read the
displays 49 to 40.5 as speeds in cm/s and "5 pot settings" as "pot 5". The burn went out at pot
0.31, display 40.5, after holding 0.32: the same setting as the 22 development rods. On that day's
curve this is about 1.47 cm/s, which is what bass4 forecast. Nothing is typed for this burn. It is
for the data lane to recode; with it at the floor, my hand estimate of the test day mean is about
0.6 for bass4 against about 1.1 for the constant (2.91 against 2.70 as coded).

**M15** is in the right class (forecast 14.7 cm/s, went out at 20 with no higher setting held); its
loss is the open interval and the band.

**M10 is the slab class again, and it is the same fault as M11 on the train days.**

## 2. The slab class, with M10 and the development slabs together

Two-sided slabs of 3 to 5 mm, all partitions:

| Width | Burn | Thickness | Starting O2 | Went out at | Class |
|---|---|---:|---:|---|---|
| 1 cm | M9 (train) | 3 mm | 18.0 % | 14 cm/s | limit |
| 1 cm | M11 (train) | 5 mm | 20.0 % | 12 to 15 | limit |
| 1 cm | **M10 (test)** | 4 mm | 20.1 % | 8 to 10 | limit |
| 2 cm | M18 (validation) | 3 mm | 17.9 % | 8 to 10 | limit |
| 2 cm | M20 (train) | 5 mm | 18.0 % | 8 to 10 | limit |
| 2 cm | M19 (test) | 4 mm | 19.0 % | 5 to 7 | between |
| 2 cm | M13 (validation) | 4 mm | 20.8 % | pot 0.3 | floor |
| 2 cm | M4 (validation) | 5 mm | 21.8 % | pot 0.3 | floor |
| 2 cm | M5 (train) | 3 mm | 22.2 % | 5 to 8, no restrictor | floor |

No threshold in thickness and oxygen alone separates these. M10 (4 mm, 20.1 %, limit) and M13
(4 mm, 20.8 %, floor) are 0.7 points apart, less than the 1.2 points the change of class takes;
and any 4 mm threshold raised far enough to move M10 moves M19 the wrong way by as much (by hand:
threshold at 20.45 %, M10 from 6.7 to about 4.5, M19 from 0.5 to about 2.6). What separates them is
the width: every 1 cm slab of 3 mm or more reached its limit, at up to 20.1 %; no 2 cm slab did
above 19 %. The 1 cm slabs of 1 and 2 mm (M7, M8) stayed at the floor, like the 2 cm ones.

**Physics.** The slab is held along both long edges in a metal frame. Heat conducted sideways from
the burning zone into the frame is, per unit of burning area, `4 lambda_s (T_s - T_inf) d / W^2`:
11 kW/m2 for 4 mm in a 1 cm frame, 2.8 kW/m2 in a 2 cm frame, against 7.2 kW/m2 radiated by the
surface. It is a steady loss, it grows with thickness and it is four times larger in the narrow
frame. A thick narrow slab therefore needs more oxygen to keep spreading, so its threshold `X_c`
is higher. (The operators said as much of the 1 cm PSI-26 slab: "heat losses are different though,
due to the metal holder".) The size is not derived: it is set between what M10 and M11 need (a
threshold of at least 21 % at 4 mm, 1 cm) and what M8 allows (2 mm, 1 cm, 19.1 %, at the floor: at
most 19.6 %): a factor 1.079 for half the width, `c_w = 0.11` per e-fold.

## 3. The change, the expectation and the result

One number, in five places: `z_c` of the slab law gets `- 0.11 ln(width / 2 cm)`. It needs the
width of flat samples as an input.

Checked once with the logged tool on `bass4`, all three partitions, second coding
(`runs/dev/gen7/study/bass4_w1.wsp` through `runs/dev/gen7/study/whatif_width.py`, which supplies
half the width of a PMMA film or slab in the idle `nose_radius` input, 0.01 for the 2 cm samples
as before; the note in the log says so). Interval CRPS, cm/s.

| | Expected (in the log before the run) | Realised | Constant |
|---|---|---|---:|
| train, day mean | 0.166 to about 0.125 | 0.166 to **0.128** | 0.833 |
| validation, day mean | unchanged, 0.108 | **0.108** | 0.648 |
| test, day mean | 2.910 to about 2.47 | 2.910 to **2.473** | 2.705 |
| train, case mean | 0.287 to 0.20 | 0.207 | 1.266 |
| validation, case mean | 0.106 | 0.106 | 0.651 |
| test, case mean | 1.544 to 1.18 | 1.183 | 2.108 |
| slabs, all partitions, case mean | M11 4.28 to 0.6, M10 6.66 to 0.45, M8 0.06 to 0.05 | 1.187 to 0.581 over 16 | |
| display scale, test day mean | not stated | 7.406 to 6.062 | 6.719 |
| days whose forecasts differ | two | 2 of 2 lower | |
| kept burning | identical | identical on every partition | |

Lower than the constant on each of the three partitions, on both scales; "kept burning" unchanged.
The condition of 13.8 is met. What this is and is not:

- It is a description of fit. The width term was typed after seeing M10; the test score of 2.473
  is not a test.
- On the test days it passes the constant by 0.23 with O13a still counted at 16.2. If the lane
  recodes O13a, both the law and the constant fall and the margin widens (hand estimate: about 0.16
  against about 1.1).
- It moves three slabs (M11, M10, M8). Everything else is bass4 to the last digit.
- Left as it was: one-sided slabs, the band of 0.35 at the limit level, M15's open interval.

## For the engineer

New slug `bass5`: the eleven features of `bass4` in the same order, one appended. Targets, kinds,
coverage and scoring rule as `bass4`.

| # | `name` | SI unit | transform | centre | scale | domain | Derivation from the condition fields |
|---|---|---|---|---|---|---|---|
| 0 to 10 | as `bass4` | | | | | | unchanged (flat samples keep `nose_radius` = 0.01) |
| 11 | `flat_width` | m | ln | 0.02 | 1 | 0.002 to 0.1 | PMMA films and slabs: `width_m` as installed. Everything else (rods, spheres, sections, fabrics, Ultem), and a flat PMMA sample with no width recorded: 0.02 |

`z = ln(flat_width / 0.02)` is the same number as the `ln(nose_radius / 0.01)` of the tool check
(half the width against 1 cm), so the release must reproduce the run above: quench day means
0.128, 0.108, 2.473 on train, validation and test (second coding), kept burning identical to bass4.

Label, wherever it appears: "typed after the final run with every partition seen; in-sample on all
data; no held-out score exists".

## Edits

All lines of the `bass4` release unchanged. New lines (column 11 is zero wherever not written):

```text
# bass5: bass4 plus the width of a slab in its frame. Typed after the final run with every partition seen;
# in-sample on all data; no held-out score exists.
x.flat_width.centre = 0.02
x.flat_width.scale = 1.0
x.flat_width.lo = 0.002
x.flat_width.hi = 0.1
# a thick slab in a narrow frame loses heat sideways into the holder (4 lambda dT d / W^2): its oxygen
# threshold is 1.079 times higher at half the width. z_c - 0.11 ln(width / 2 cm); times 30 in g.
flat_quench_speed.median.W1[3, 11] = -3.3
flat_quench_speed.median.W1[4, 11] = -3.3
flat_quench_speed.spread.W1[3, 11] = -3.3
flat_quench_speed.spread.W1[4, 11] = -3.3
flat_quench_speed.spread.W1[5, 11] = -3.3
```

For the data lane: O13a (section 1), and the three other rows of 2014-07-21 whose air cell holds
fan displays.

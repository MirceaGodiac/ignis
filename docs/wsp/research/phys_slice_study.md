# SLICE (PSI-106): study of the development lines, the day-061 question, and the blow-off law

Author: `claude-fable-5-2`, 15 November 2026, under PROTOCOL 13.6. **Data-informed, in-sample**:
written from the 364 logged lines of 98 flames on the development days (train and validation),
read flame by flame, and from the log pages of those days in
`Raw Data/SLICE-RPT-002_Post Flight Report-.pdf` (pages 40 to 46 only; no page of a test day).
Throwaway tables: `runs/dev/gen7/study/sl_*.py`. Two candidates were run with the logged tool on
`slice2` (2 of 5; the engineer's own tool test on `lift_noted` is not one of them and I did not
use its direction).

## 1. What the data show

### 1.1 The blow-off boundary is a straight line in the two speeds

On one tube with one fuel the crew found the limit several times, at different fan settings. Pure
methane on the 0.41 mm tube (day 058), blow-off or lifting at:

| coflow, cm/s | 8.7 | 21 | 26 | 31 | 43 | 5.4 |
|---|---|---|---|---|---|---|
| jet speed, m/s | 18.8 | 13.5 to 14.4 | 10.8 to 11.1 | 9.7 | 5.9 (lifted) | 22.5 survived |

that is `U_j / 22.4 m/s + U_co / 0.57 m/s = 1`. The same form holds for the other fuel and tube
pairs with other intercepts:

| fuel, tube | jet intercept `U_j0` | coflow intercept `U_c0` | from |
|---|---|---|---|
| methane 100 %, 0.41 mm | 22 m/s | 0.57 m/s | day 058, seven limit points |
| methane 100 %, 0.77 mm | 25 to 40 m/s | about 0.6 m/s | days 058, 072: (12.8 m/s, 39 cm/s), (8.3, 37), lifted (4.7, 44) |
| methane 70 %, 0.41 mm | 4.6 m/s | 0.36 m/s | day 065: (3.9, 11), (2.5, 17), (4.2, 5) twice |
| methane 70 %, 0.77 mm | 7 to 10 m/s | 0.5 to 0.75 m/s | days 048, 065 |
| methane 40 %, 1.6 mm | (jet slow) | 0.30 to 0.34 m/s | day 061: blow-out at 29 cm/s, lifted at 34 cm/s |
| ethane, 0.41 mm | about 20 m/s | about 0.6 m/s | day 069: (6.1, 42), (19.0, 6.5) |

How the intercepts move:

- **with dilution**, the coflow intercept as the square of the burning speed (70 %: 0.71 of pure
  predicted, 0.63 seen), which is what CLD Flame shows independently (`phys_cld_study.md`); the jet
  intercept much faster (70 %: 0.21 of pure on 0.41 mm, 0.3 on 0.77 mm): as `(S_L / Z_st)^2`, the
  burning speed squared and the stoichiometric mixture fraction squared (0.25 predicted). The W0
  law had `S_L` to the first power and `Z_st` to the first power in the jet term; the D1 step
  removed the `Z_st` factor and was wrong to (see 1.3);
- **with the tube**, the jet intercept grows less than in proportion to the diameter (exponent
  about 0.7; W0 had 1), and the coflow intercept grows with it (exponent about 0.45; W0 had 0): a
  thicker rim shelters the flame base better.

One typed boundary, with these exponents and the two anchors of day 058:

```text
Psi = U_j / U_j0 + U_co / U_c0
U_c0 = 0.57 m/s (S_L / 0.38)^2 (d / 0.41 mm)^0.45 (x_O2 / 0.228)^3
U_j0 = 22.4 m/s (S_L / 0.38)^2 (0.0552 / Z_st)^2 (d / 0.41 mm)^0.7 (x_O2 / 0.228)^3
```

and the lines of every development day except the first block of day 061 fall like this:

| ln Psi | lines | blow-off | lifted (not blown off) |
|---|---:|---:|---:|
| below -0.2 (Psi below 0.82) | 215 | 0 | 0 |
| -0.2 to 0 | 41 | 10 | 4 |
| 0 to 0.2 | 13 | 3 | 3 |
| 0.2 to 0.5 | 18 | 6 | 1 |

No flame was lost or lifted below `Psi` = 0.82; above it about one line in four is a blow-off,
whatever the distance beyond. The exponents were read from these lines, so the table is a
description, not a test.

Oxygen is the weak point of the formula: ethane on day 069 (cabin oxygen 0.195) blows off at
`Psi` = 1.45 with the cube of the oxygen fraction and at 0.9 without it, while methane on days 058
and 072 (0.228 and 0.217) wants a positive exponent. The data cannot fix it; the cube is the
physical prior (burning speed).

### 1.2 Day 061: the fuel label can be settled, the tube label cannot stand

The crew log writes "100 % Methane, 0.4 mm" for tests 63 to 67 and 63A of GMT 061; the bottle
table of the final report says the day burned 40 % methane, and the case builder took the table.

**Fuel: 40 %, the table is right.** The log records the radiometer reading beside each line. Per
unit of fuel display, at radiometer gain 250 and the lowest fan setting:

| day, label | reading / fuel display | relative to pure methane |
|---|---|---|
| 058, methane 100 %, 0.4 mm | 0.0092 to 0.0097 (displays 26 to 98) | 1 |
| 065, methane 70 %, 0.4 mm | 0.0065 to 0.0073 (displays 22 to 43) | 0.72 |
| 061, tests 63 to 67 | 0.0041 to 0.0053 (displays 70 to 240) | 0.43 to 0.55 |

A flame's radiant output goes with the methane it burns. Per display unit the methane flow is
`K_mix x`: 0.72 for pure methane, 0.55 at 70 % (0.76 of pure), 0.346 at 40 % (0.48 of pure). The
readings give 0.72 and 0.43 to 0.55. Day 061 burned 40 % methane. (The radiometer is a
measurement of the flame; it is used here to settle a label, not as an input.)

**Tube: not 0.4 mm.** As recorded, 40 % methane on the 0.41 mm tube holds at jet speeds of 7.5 to
53 m/s and coflows to 46 cm/s. On the same tube 70 % methane blows off at 2.5 to 4.2 m/s (day 065,
"blow off immediately at fuel = 70"), and pure methane at 9 to 19 m/s. A more diluted fuel cannot
be ten times more robust. The same day's 40 % flames on the 1.6 mm tube (tests 168, 169) blow off
at 29 to 34 cm/s, as expected of 40 %. With the boundary of 1.1, the three event lines of the first
block and its 26 other lines sit at:

| tube assumed for tests 63 to 67A | Psi of the three event lines | other lines above Psi = 1 |
|---|---|---|
| 0.41 mm (as logged) | 22, 23, 56 | 26 of 26 |
| 0.77 mm | 5.5, 5.6, 11 | 26 of 26 |
| 1.59 mm | 1.8, 2.0, 2.4 | 20 of 26 |
| 2.14 mm | 1.4, 1.5, 1.5 | 12 of 26 |
| **3.23 mm** | **0.93, 1.03, 1.13** | **1 of 26** |

Only the 3.2 mm tube puts the block where every other day puts its limit. Its jet speeds would
then be 0.12 to 0.86 m/s, and the investigators' published series of 40 % methane on the 3.2 mm
burner runs from 0.24 to 0.89 m/s (Cao et al. 2015). I hold the burner entry of these six tests to
be wrong in the log, or in its transcription. It cannot be proved from the conditions alone: the
image file names or EXIF of that day, or the test matrix of the report, would settle it.

### 1.3 What this says about the earlier steps

W0 lost on blow-off almost entirely on these 29 lines (62 % of its loss). D1 took out the `Z_st`
factor of the jet term to soften them. The genuine dilution data (day 065, 70 % methane on 0.41
and 0.77 mm) say the opposite: dilution weakens the jet limit more than W0 had it. D1's step was a
fit to a mislabelled block. I had flagged the fuel label in D1 and not looked at the tube.

## 2. Candidates

### S1 (run; not proposed as it stands): the boundary of 1.1

`logit P(blow-off on a line) = -1.2 + 4 ln Psi`, `Psi` as a three-piece sum of the two parts in
the form of the W0 network; 40 elements of `blow_off.logit`; `lifted` untouched.

| day (mean Brier per line) | slice2 = W0 | Expected before the run | Realised |
|---|---:|---|---:|
| 058 | 0.100 | lower, about 0.07 | 0.093 |
| 065 | 0.155 | lower, about 0.10 | 0.121 |
| 072, 069 | 0.021, 0.018 | about equal (within 0.01) | 0.020, 0.022 |
| 040, 048, 079 | near zero | about equal | 0.002, 0.001, 0.0002 |
| 061 | 0.575 | **higher**, about 0.7 (the 29 lines at Psi 20 to 80 survive) | 0.749 |
| pooled, mean of day means | 0.109 | 0.11 to 0.12, not passing the keep rule | 0.126 (4 of 8 days lower) |

Every sign as predicted. The law is better on the days whose inputs are sound and worse on the
block whose tube label is wrong, by construction: a law that knows the limit says those flames
cannot be there.

### S2 (run; proposed as the comparator): S1, with the jet part saturating

A flame that is seen burning with its jet part alone already beyond the limit says the jet limit
is wrong for that flame; the jet part then stops adding and the coflow part still counts. One
hinge added (`-4 relu(ln(U_j / U_j0))`). This is the same logic as the lifted window of `slice2`:
the forecast is conditional on the flame being there.

| | slice2 = W0 | Expected before the run | Realised |
|---|---:|---|---:|
| 061 | 0.575 | lower, about 0.12 | 0.209 |
| 058, 065 | 0.100, 0.155 | as S1 (0.093, 0.121) | 0.093, 0.116 |
| 069 | 0.018 | slightly higher, about 0.024 (one event line with jet part 1.32) | 0.025 |
| 040, 048, 072, 079 | | as S1 | 0.002, 0.001, 0.020, 0.0002 |
| pooled, mean of day means | 0.109 | about 0.048 | **0.058** (incidence baselines 0.048 to 0.054) |
| train / validation | 0.102 / 0.155 | | 0.050 / 0.116 (baselines 0.033-0.036 / 0.149-0.176) |
| days lower | | | 5 of 8 (higher: 040 and 048 by 0.001 or less, 069 by 0.007) |
| lifted | 0.033 | unchanged | 0.033 |

Signs as predicted on every day; day 061 improved less than I wrote (0.21, not 0.12: its lines
keep a coflow part that carries them to 0.3 to 0.5 per line). By the keep rule S2 does not replace
the model: it is lower pooled (0.058 against 0.109) but on 5 of 8 days, not two thirds; the three
"higher" days differ by 0.007 at most. It is a reported comparator.

Not run: a steeper slope below the boundary (the 57 lines forecast at 0.05 to 0.15 hold no
event). It would lower the no-event days and raise the event lines just inside the boundary by
about as much; I expect no net gain and did not spend a candidate on it.

## 3. Proposal

1. **An input check before any further score**: the burner of GMT 061 tests 63, 64, 65, 66, 67
   and 63A. If it is the 3.2 mm tube, rebuild the cases (`burner_id_mm` 3.23, `u_jet_m_s`
   divided by 62) and run **S1** on them: I expect day 061 near 0.05 and the pooled score near
   0.04, below the incidence baselines. That run is a new candidate and must be logged as one.
2. Until then, **S2 as the comparator** on `slice2`; the model stays `slice2` s1.

### For the engineer

- The check above (log pdf page 43, first data column "Burner tube ID"; image names of GMT 061).
- Day 061 keeps `x_fuel` = 0.4: confirmed by the radiometer (1.2).
- The 41 ethylene lines (day 075, 20 % ethylene) are refused because the declaration has no
  ethylene indicator. With `S_L` = 0.295 m/s and `Z_st` = 0.254 for 20 % ethylene and the factor
  1.9 by which ethylene exceeds methane at equal burning speed (CLD Flame), the boundary of 1.1
  puts two of their three blow-off lines at `Psi` about 1.0 to 1.1 (the third blew off before its
  setting was reached). An indicator `is_ethylene` would carry it: about `+2.8` on `ln U_c0` and
  `+4.0` on `ln U_j0` relative to what the methane terms give at this dilution (valid for 20 %
  only; not run).

### Edits (changed lines; parent `configs/models/law/slice2/s1-fable`). S1 is the block without its last three lines; S2 is the whole block.

```text
blow_off.logit.b0 = -5.232
blow_off.logit.skip[0:8] = [4.0, 0.0, -1.8, -4.4, -12.0, -0.608, -0.608, 0.0]
blow_off.logit.W1[0, 0:8] = [-1.0, 1.0, -0.25, -2.6, 0.0, 0.130, 0.177, 0.0]
blow_off.logit.b1[0] = -0.608
blow_off.logit.w2[0] = 1.24
blow_off.logit.W1[1, 0:8] = [-1.0, 1.0, -0.25, -2.6, 0.0, 0.130, 0.177, 0.0]
blow_off.logit.b1[1] = -2.808
blow_off.logit.w2[1] = 1.52
blow_off.logit.W1[2, 0:8] = [-1.0, 1.0, -0.25, -2.6, 0.0, 0.130, 0.177, 0.0]
blow_off.logit.b1[2] = -5.008
blow_off.logit.w2[2] = 1.24
blow_off.logit.w2[3] = 4.0
blow_off.logit.W1[4, 0:8] = [0.0, 1.0, -0.7, -3.7, -3.0, -0.022, 0.025, 0.0]
blow_off.logit.b1[4] = -3.816
blow_off.logit.w2[4] = -4.0
```

How the numbers follow from 1.1, with `z_co = ln(U_co / 0.3)`, `z_j = ln(U_j / 1)`, `z_d = ln(d /
1.6 mm)`, `z_x = ln x`, `z_o = ln(x_O2 / 0.21)`: the coflow part is `A = -1.008 + z_co - 0.45 z_d
- 1.1 z_x - 3 z_o - 0.152 (ethane, propane) + relu(-0.693 - z_x)`; the jet part is `B = -3.816 +
z_j - 0.7 z_d - 3.7 z_x - 3 z_o - 0.022 ethane + 0.025 propane`; `ln Psi = A + ln(1 + e^(B - A))`
in three pieces; logit `= -1.2 + 4 ln Psi` (S1), less `4 relu(B)` (S2). (Files run:
`runs/dev/gen7/study/slice2_s1.wsp`, `slice2_s2.wsp`.)

## 4. What remains structural

- **The per-line probability needs the previous line.** A blow-off on a line is the limit lying
  between the last setting and this one. The data show it: the limit points of day 058 were
  reached after jumps of 12 to 49 % in fuel display, and a line repeated at the same setting
  cannot blow off for the law's reason. With the size of the step as an input the forecast would
  be the hazard times the step; as it is, a step of 10 % is built into the level -1.2. The
  forward reads one line; this needs a feature computed from two lines and its weight depends on
  which part (jet or coflow) was stepped.
- **Above the boundary the probability is flat** (one line in four); a logit linear in `ln Psi`
  cannot be flat on one side and steep on the other without a hinge on `ln Psi`, which already
  holds hinges.
- **Lifted** is seen in the same band of `Psi` as blow-off (8 lines of 72 between 0.82 and 1.65,
  none below). The window targets of `slice2` still read the W0 stability number; retyping them
  on `Psi` is a further candidate, expected gain 0.004 on a score of 0.033. Not done.
- **Cabin oxygen**: one exponent cannot satisfy days 058, 069 and 072 together (1.1).

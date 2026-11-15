# SPICE (PSI-107): study of the development smoke points, the nozzle dependence, and a third law

Author: `claude-fable-5-2`, 15 November 2026, under PROTOCOL 13.6. **Data-informed, in-sample**:
written from the 49 smoke points of the development days (train and validation), read one by
one. The six points of the test days were not opened. Throwaway tables in
`runs/dev/gen7/study/`. Two candidates were run with the logged tool on `spice`, base `w0-fable`
(the clean variant B of `phys_spice_w0.md` 5.1), 2 of 5.

## 1. What the data show

### 1.1 The nozzle does not act as a power

Smoke-point length of pure propylene near 27 cm/s of coflow: 53 mm on the 0.41 mm nozzle, 22 mm on
0.76 mm, 17 mm on 1.6 mm. From 0.41 to 0.76 mm the length falls by 2.4 (an exponent of -1.4); from
0.76 to 1.6 mm by 1.25 (an exponent of -0.3). Ethylene near 18 cm/s: 105 mm on 0.76 mm, 37 mm on
1.6 mm (an exponent of -1.4 already between these two). One exponent for all (the published
-0.91, which both my W0 laws carried) must then be too high on the middle nozzle and too low on
the ends:

| nozzle | cases | variant B, median ln(forecast / observed) | published fit |
|---|---:|---:|---:|
| 0.41 mm | 4 | +0.04 | -0.22 |
| 0.76 mm | 16 | **+0.48** | +0.21 |
| 1.6 mm | 29 | +0.10 | -0.06 |

This is what the residence-time argument of W0 section 1.2 said and what I then did not use: the
jet's own residence time `t_jet` goes as `d^2`; while it is several times the fuel's soot time
`t_c` the nozzle hardly matters, and as it comes down to `t_c` the smoke point runs away. With
the W0 numbers:

| fuel, nozzle | `t_jet / t_c` | what the data do |
|---|---:|---|
| propylene, 1.6 mm | 16 | weak nozzle effect |
| propylene, 0.76 mm | 3.6 | weak nozzle effect (1.25 over 1.6 mm) |
| propylene, 0.41 mm | 1.06 | long smoke points, 38 to 59 mm, almost flat in coflow |
| 75 % propylene, 0.76 mm | 1.9 | 1.4 to 1.65 over 1.6 mm |
| ethylene, 0.76 mm | 1.25 | 2.8 over 1.6 mm |
| propane, 0.76 mm | 1.8 | 64 to 106 mm (no 1.6 mm point to compare) |

The steep part starts where `t_jet` is about twice `t_c`: at a nozzle `d_c` of 0.96 mm for
ethylene, 0.80 mm for propane, 0.56 mm for propylene, and larger for a diluted fuel (as
`x^-1.1`: 0.74 mm at 75 %, 1.27 mm at 50 %). These follow from the W0 property numbers; only the
factor two is read from the table.

### 1.2 A floor, then a rise with the coflow

On the 1.6 mm and 0.76 mm nozzles the propylene smoke point does not fall below about 14 mm
however slow the coflow (13.4 to 14.2 mm at 7 to 10 cm/s on both nozzles), then rises: on 1.6 mm
15.5 mm at 20 cm/s, 19.5 at 40, 26.5 at 57. The rise is convex, not the concave `u^0.41` of the
published fit; between 17 and 57 cm/s it goes as `u^0.5` to `u^0.65` for every fuel on 1.6 mm.
This is the two-term picture of W0 section 1.4 (a radiative floor and a coflow term). Two numbers
of that section hold against the data without adjustment:

- ethylene on 1.6 mm: floor 21 mm (shuttle) plus coflow speed times the normal-gravity soot time
  92 ms gives 38, 52, 54, 57, 74 mm at 18.5, 33.6, 36.4, 39.2, 58 cm/s; measured 37, 54, 57.5, 56,
  75 mm;
- propylene on 1.6 mm at the fastest coflow: 26.5 mm at 57 cm/s is 46 ms, against 48 ms.

They do not hold together for propylene at lower coflow (a sum of 14 mm and 48 ms times the speed
is 40 % too long at 57 cm/s), so the form typed below is a floor and a power, not a sum.

The floor of propylene is 14 mm, not the 5.7 mm (variant B) or 7.5 mm (main law) I had guessed.

### 1.3 The length at the smoke-point flow

Given the flow, the luminous length is `K mdot F / Z_st` to within 0.13 in ln, but short by a
median 0.17 (0.20 on 1.6 mm). Two causes. The soot term of the W0 network is switched on at some
smoke points (W0 section 2 already said to set it to zero for this target). And the shortfall is
an offset, not a factor: on 1.6 mm it is 4.6 mm for a 14 mm flame and 2 mm for a 26 mm flame,
about two nozzle diameters. The far-field law treats the nozzle as a point; the fuel leaves a
tube and the jet core adds its length (the virtual origin of Lin and Faeth, which W0 neglected,
noting that the shortest flames on the widest nozzle would carry the error).

## 2. Candidates

### P1 (run): the smoke-point length

```text
L_sp = max( L0 ,  c u^0.55 (d / 1.6 mm)^-0.33 / x_fuel ) * (d_c / d)^3 for d < d_c
L0 = 21, 28, 14 mm;   c = 8.08, 10.3, 2.87 mm at 1 cm/s;   d_c = 0.96, 0.80, 0.56 mm x_fuel^-1.1      (ethylene, propane, propylene)
ln-sd 0.18
```

`L0` of ethylene and propane are the shuttle values of W0; that of propylene and the three `c`
are read from the 1.6 mm points (propane from its three 0.76 mm points); `d_c` is 1.1; the
exponent 3 of the jet factor is read from the two pairs of 1.1; dilution as `1/x` (the mixing
rule) reproduces the 75 and 50 % series.

| | variant B | Expected before the run | Realised |
|---|---:|---|---:|
| T1 score, mm, mean of day means | 6.51 | 2 to 2.6, below the per-fuel baseline | **2.13** (per-fuel baseline 6.63; pooled climatology 10.16) |
| train / validation | 6.70 / 5.86 | | 2.22 / 1.83 (baseline 7.08 / 5.04) |
| 0.76 mm, median ln error | +0.48 | within 0.1 | -0.003 |
| 1.6 mm | +0.10 | within 0.05 | +0.03 |
| 0.41 mm | +0.04 | within 0.15, scatter stays | +0.035 (2 of 4 inside the 50 % band) |
| propane | +0.44 | about +0.05 | +0.08 |
| days lower | | all, or all but day 174 | **9 of 9** |
| T2 | 3.07 | unchanged | 3.07 |

### P2 (run; proposed): P1 and the flame length

`L = K mdot F / Z_st + 2 d`, no soot term, ln-sd 0.14.

| | variant B | Expected before the run | Realised |
|---|---:|---|---:|
| T2 score, mm, mean of day means | 3.07 | 1.5 to 1.8, at or below the Lin and Faeth baseline | **1.94** (Lin and Faeth with a train-fitted width: 1.84) |
| train / validation | 3.23 / 2.53 | | 2.09 / 1.41 (baseline 2.01 / 1.26) |
| median ln error | -0.17 | about -0.04 | -0.07 (1.6 mm), -0.10 (0.76 mm), -0.07 (0.41 mm) |
| days lower | | all 9 | **9 of 9** |
| T1 | | as P1 | 2.13 |

Signs as predicted; the T2 gain is smaller than I wrote, and the law is still short by 0.07 to
0.10 and just above the baseline (which is the same far-field law with its width fitted on the
train days). I did not chase the remainder: it could be a third diameter of offset or four per
cent on `C_f`, and the data do not say which.

## 3. Proposal

**P2 as the model on slug `spice`** (a revision of `w0-fable`; numbers only). Lower pooled on both
targets and on 9 of 9 days for each.

What it is: a law read off 49 points with seven fuel numbers, one coflow exponent and three
nozzle numbers. The six test points are its only test. Its physical content, which W0 had and
did not use: a floor, a coflow term, and a nozzle that matters only when the jet's residence time
comes down to the fuel's soot time.

### For the engineer

Nothing new: same declaration, same inputs. The secondary targets `smoke_point_fuel_flow` and
`emits_soot` still carry the W0 smoke-point numbers inside them and are not consistent with P2;
they are not scored. If they are ever used, they must be retyped from P2.

### Edits (changed lines; parent `configs/models/law/spice/w0-fable`)

```text
smoke_point_length.median.b0 = -3.863
smoke_point_length.median.skip[0:7] = [0.0, 0.0, 0.0, 0.0, 0.288, -0.405, 2.0]
smoke_point_length.median.W1[0, 0:7] = [-0.33, 0.55, 0.0, -1.0, -0.045, -0.630, 0.0]
smoke_point_length.median.b1[0] = 0.970
smoke_point_length.median.w2[0] = 1.0
smoke_point_length.median.W1[1, 0:7] = [-1.0, 0.0, 0.0, -1.1, -0.182, -0.539, 0.0]
smoke_point_length.median.b1[1] = -0.041
smoke_point_length.median.w2[1] = 3.0
smoke_point_length.spread.b0 = 0.18
flame_length.median.W1[2, 0:7] = [1.0, 0.0, -1.0, -0.66, -0.055, 0.0, -0.143]
flame_length.median.b1[2] = -0.120
flame_length.median.w2[2] = 0.31
flame_length.median.W1[3, 0:7] = [1.0, 0.0, -1.0, -0.66, -0.055, 0.0, -0.143]
flame_length.median.b1[3] = -2.320
flame_length.median.w2[3] = 0.38
flame_length.median.W1[4, 0:7] = [1.0, 0.0, -1.0, -0.66, -0.055, 0.0, -0.143]
flame_length.median.b1[4] = -4.520
flame_length.median.w2[4] = 0.31
flame_length.spread.b0 = 0.14
```

Reading of the numbers, with `z_d = ln(d / 1 mm)`, `z_u = ln(u / 25 cm/s)`, `z_x = ln x`:
`b0` is the floor of ethylene (21 mm), the skips the floors of propane (28 mm) and propylene
(14 mm); unit 0 is the coflow term above the floor, `0.970 = ln(8.08 * 25^0.55 * 1.6^0.33 / 21)`
with the fuel offsets `ln((c / L0) / (c / L0)_ethylene)`; unit 1 is the jet factor,
`3 relu(ln d_c - ln d)` with `ln 0.96 = -0.041` and the fuel offsets `ln(0.80 / 0.96)`,
`ln(0.56 / 0.96)`. In the flame length, units 2 to 4 are `ln(1 + 2 d / L_far)` in three pieces
(`-2.320 = ln(2 mm / 20.3 mm)`). (Files run: `runs/dev/gen7/study/spice_p1.wsp`, `spice_p2.wsp`.)

## 4. What remains structural

- **The 0.41 mm nozzle**: four points, 38 to 59 mm, with no order in the coflow (38 mm at 5 cm/s,
  59 mm at 10, 47 at 15, 53 at 27). Where the jet's residence time is at the fuel's soot time the
  smoke point is set by second-order effects, and a few per cent of fuel flow move it by tens of
  millimetres. The law gives a level there and no more; a width that grows as `d` falls below
  `d_c` would say so, and a spread hinge can carry it (not typed: four points).
- **Propane** rests on three points on one nozzle on one day; its `c` for 1.6 mm is inferred
  through the nozzle law. Ethylene on 0.76 mm is one point.
- **Why the rise is `u^0.55`** and not the sum of a floor and a residence term: unexplained. The
  sum works for ethylene with no adjusted number and fails for propylene.
- **Cabin oxygen** (20 to 23 %) is not recorded per smoke point; it is in the width.

## 5. Addendum (15 November, later): the flame length given the flow, against the Lin and Faeth law

Asked by the coordinator after P2 became the model: P2 was still 0.07 to 0.10 short on the flame
length and just behind the baseline (1.94 against 1.84 mm). One more candidate was run (3 of 5 on
`spice`), on base `p2-fable`.

### 5.1 What the baseline has

The baseline is `L = mdot / (4 pi mu Z_st)`: Lin and Faeth's coflow law with both coefficients set
to one (`C_f Sc` = 1 where the published values give 0.80), the viscosity of air at each fuel's
mean flame temperature, and one width fitted on the train days. It has no dependence on the
velocity ratio, the nozzle or the fuel beyond `Z_st`. It is not a better law than P2; its constant
happens to sit in the middle of the data. With `G` = observed length over the baseline's:

| nozzle, fuel | cases | `r = u_air / u_fuel` | L / d | median G |
|---|---:|---|---|---:|
| 0.41 mm, propylene | 4 | 0.01 to 0.04 | 93 to 146 | 1.37 |
| 0.76 mm, ethylene | 1 | 0.02 | 138 | 1.08 |
| 0.76 mm, propane | 3 | 0.06 to 0.07 | 84 to 138 | 1.05 |
| 0.76 mm, propylene | 8 | 0.12 to 0.33 | 19 to 42 | 1.12 |
| 0.76 mm, propylene 75 % | 4 | 0.13 to 0.16 | 26 to 58 | 1.08 |
| 1.6 mm, ethylene | 5 | 0.27 to 0.41 | 23 to 47 | 0.91 |
| 1.6 mm, propylene 50 % | 4 | 0.28 to 0.52 | 17 to 31 | 0.95 |
| 1.6 mm, propylene 75 % | 10 | 0.71 to 1.20 | 9 to 24 | 1.03 |
| 1.6 mm, propylene | 10 | 0.73 to 1.85 | 8 to 17 | 1.05 |
| all | 49 | | | 1.055 (sd of ln G 0.117) |

Three things in this table that P2 had wrong, and that a constant averages over:

1. **The fuel.** At the same velocity ratio propylene is longer than ethylene by about 1.2
   (1.37 against 1.08 at r near 0.02; 1.0 to 1.1 against 0.91 near r = 0.3). The luminous tip lies
   beyond the flame sheet by the length the soot needs to burn out, and a sootier fuel carries
   more of it: the factor `C_f` is not one number. Diluting propylene brings it back (75 %: 1.02;
   50 %: 0.92, level with ethylene).
2. **The velocity ratio acts slowly.** Theory gives two limits, 3/2 for a jet in still air and 1
   for strong coflow. P2 (from W0) put the whole passage between r = 0.02 and r = 0.5. The data
   spread it over the whole range flown: for one fuel, `G` falls by about 0.08 per e-fold of `r`
   from 0.01 to 2 (1.5 over two decades and a half, which is the theoretical ratio). So P2 was
   too long for ethylene and propane on the 0.76 mm nozzle (r 0.02 to 0.07) and too short for
   propylene where the coflow is faster than the jet (r above 0.7).
3. **The offset** is nearer 1.3 nozzle diameters than 2 once the two points above are in.

### 5.2 Candidate P3 (run; proposed)

```text
L = c_f[fuel] K mdot / Z_st (r / 0.3)^-0.08 + 1.3 d,     K as W0 (1.30 mm per mg/s)
c_f = 1.079 (ethylene), 1.169 (propane), 1.324 x_fuel^0.22 (propylene);   ln-sd 0.10
```

`c_f` is the value at r = 0.3 relative to the W0 constant; for ethylene it is what P2 already
gave there, so the ethylene flames on 1.6 mm keep their level.

| | P2 | Expected before the run | Realised |
|---|---:|---|---:|
| T2 score, mm, mean of day means | 1.94 | 1.3 to 1.6, below the Lin and Faeth baseline (1.84) | **1.25** |
| train / validation | 2.09 / 1.41 | | 1.37 / 0.83 (baseline 2.01 / 1.26) |
| median ln error, 1.6 mm | -0.07 | within 0.03 | -0.056 |
| 0.76 mm | -0.10 | within 0.03 | -0.009 |
| 0.41 mm | -0.07 | within 0.03 | -0.011 |
| days lower | | at least 7 of 9; day 045 possibly higher | **8 of 9**; day 045 lower (3.90 to 1.83); day 119 higher (0.70 to 0.82) |
| T1 | 2.13 | unchanged | unchanged |

The pooled sign and the size were predicted; the day I named as the risk improved and another
(119, two flames of 75 % propylene on 1.6 mm) got slightly worse; the 1.6 mm nozzle is still
0.06 short, outside the 0.03 I wrote. By the keep rule P3 passes (lower pooled, 8 of 9 days, the
other target unchanged).

### 5.3 Edits (changed lines; parent `configs/models/law/spice/p2-fable`)

```text
flame_length.median.b0 = -3.797
flame_length.median.skip[0:7] = [-0.16, -0.08, 1.08, 0.858, 0.099, 0.173, 0.188]
flame_length.median.w2[0] = 0.0
flame_length.median.w2[1] = 0.0
flame_length.median.W1[2, 0:7] = [1.16, 0.08, -1.08, -0.858, -0.099, -0.173, -0.188]
flame_length.median.b1[2] = -0.648
flame_length.median.W1[3, 0:7] = [1.16, 0.08, -1.08, -0.858, -0.099, -0.173, -0.188]
flame_length.median.b1[3] = -2.848
flame_length.median.W1[4, 0:7] = [1.16, 0.08, -1.08, -0.858, -0.099, -0.173, -0.188]
flame_length.median.b1[4] = -5.048
flame_length.spread.b0 = 0.10
```

Reading: `ln r = -1.477 + z_u + 2 z_d - z_m + 0.27 z_x + 0.452 propane + 0.405 propylene - 0.559
methane` (the W0 expression), so `-0.08 ln(r / 0.3)` adds `0.022 - 0.08 z_u - 0.16 z_d + 0.08 z_m
- 0.022 z_x - 0.036 propane - 0.032 propylene + 0.045 methane` to the W0 line; `ln c_f` adds
0.076, and 0.080 more for propane, 0.205 more for propylene, with `0.22 z_x`; the two hinges of
the old velocity-ratio step are switched off; units 2 to 4 are `ln(1 + 1.3 d / L_far)` in three
pieces, `-2.848 = ln(1.3 mm) - (-3.797)`. (File run: `runs/dev/gen7/study/spice_p3.wsp`.)

Left as it is: the 0.06 on the 1.6 mm nozzle (an offset of 2 d with these factors, or 5 % on
`c_f`; the cases do not say which), and the dilution exponent 0.22, which rests on propylene
alone. The numbers are read from the 49 development points; the six test points are the test.

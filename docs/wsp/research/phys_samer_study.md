# SAME-R (PSI-101): a study of the development rows, and one checked revision of the W0 law

Author: claude-fable-5-2, 15 November 2026, under `docs/wsp/PROTOCOL.md` 13.6. **Data-informed.** Read, row by
row: the 74 development rows of SAME-R (50 on train days, 24 on validation days: inputs, `d10`, and from the
outcome files also `d30`, `sigma_g`, number and mass concentration, the detector), and the 45 rows of SAME as
study data for the physics the two flights share (coagulation coefficient, number concentration, narrowing; SAME
is not revised). The three SAME-R test days were not opened. Parent of the proposal: `samer` W0
(`sha256:05f77ae2…7fb0`), not D1. Four evaluations were logged in `runs/dev/gen7/whatif_log.jsonl`
(three candidates and the proposal as one file); each prediction was written in the note before its run.

## 0. What changed in one screen

The W0 law had the right mechanism and the wrong amounts. The data confirm the coagulation law to 10 %
(`K` = 5.0 to 5.7e-16 m3/s measured on the large smokes against 6.0e-16 typed) and show that:

1. **every smoke starts aging with about the same number of particles**, 3.4e6 per cm3 at 8.1 cm/s, whatever
   the material and the temperature. W0 gave each material its own number (1.2 to 4.2e6) and let it rise with
   temperature; silicone and Pyrell therefore aged far too slowly in the law;
2. **the number falls with the flow** (1.25e6 at 2 cm/s, 0.3e6 at 1 cm/s, 0.2e6 with no flow), so low-flow and
   no-flow smokes hardly grow afterwards;
3. **the size is set by the mass of smoke**, which spans 1 to 350 mg/m3: with one number for all, the diameter
   of average mass is the cube root of the mass concentration. The smokes are also broader than typed
   (`sigma_g` 2.1 to 2.5, not 1.6 to 1.9), so the count mean grows faster with aging (power 0.55 of the number
   ratio, not 0.41);
4. **the detector follows the mass of smoke**: at full scale on 29 of 29 rows of lamp wick, Pyrell and silicone
   with flow, never for Teflon, for Kapton only on its densest smoke; with no flow the smoke that reaches the
   detector is 3 to 10 times less (not half), except for silicone.

| Target | W0 | Proposal S1 | Best baseline | Days lower / days that change |
|---|---|---|---|---|
| `d10`, CRPS of ln, pooled (train / validation) | 0.238 (0.272 / 0.167) | 0.172 (0.191 / 0.133) | 0.276 | 12 / 12 |
| Detector at full scale, Brier, pooled (train / validation) | 0.175 (0.150 / 0.228) | 0.056 (0.049 / 0.071) | 0.101 | 10 / 10 |

Development numbers, in-sample. The test days are the judge.

## 1. What the data showed

### 1.1 The coagulation law holds, with the typed coefficient

For each run measured twice (primary, then aged), `K = (1/N_2 - 1/N_1) / (t_2 - t_1)` from the two number
concentrations:

| Smoke | Runs | `K`, 1e-16 m3/s | W0 |
|---|---|---|---|
| Silicone, 190 to 250 nm | 59, 60, 65, 66 | 5.5, 5.3, 4.8, 5.6 | 6.0 typed (5.7 derived for 0.31 um) |
| Pyrell, 210 to 250 nm | 63, 64 | 5.7, 5.6 | 6.0 |
| Kapton at 847 K, 148 nm | 62 | 5.0 | 6.0 |
| Kapton at 690 to 800 K, 56 to 92 nm | 51, 61, 93, 94 | 18, 12, 13, 10 | theory for 60 nm with slip and `sigma_g` 2.1: 14 |
| Teflon, 53 nm | 58 | 17 | as Kapton |
| Lamp wick, 116 nm, 171 s | 55 | 11 | 6.1 |
| SAME (study only): silicone; lamp wick; Kapton; Teflon | 23, 35; 32, 46; 29, 47; 26 | 6.2, 5.1; 7.4, 5.6; 9.3, 7.9; 10.8 | |

The first falsifier of W0 (growth proportional to number times time, coefficient within a factor 1.5 of
6e-16 for the liquid smokes) is passed. The diameter of average mass follows the cube root of the number
ratio (silicone 59: number down 2.65 times, `d30` up 1.35 = 2.65^0.31).

### 1.2 The number at the start is the same for every smoke, and falls with the flow

Number concentration of primary rows aged under 100 s, 1e6 per cm3:

| Flow | Kapton | Lamp wick | Pyrell | Silicone | Teflon | Median | W0 implied (`1/(K t_c)`) |
|---|---|---|---|---|---|---|---|
| 8.1 cm/s | 1.7 to 5.9 | 3.2, 3.4 | 3.1 to 3.9 | 2.5 to 5.7 | 2.1 to 3.7 | 3.4 | Kapton 1.0 (690 K) to 5.1 (850 K), Teflon 3.7, lamp wick 1.3 to 1.8, Pyrell 0.8, silicone 0.8 to 1.1 |
| 2 cm/s | - | 0.92 | 1.18 | 1.99 | 1.33 | 1.25 | as at 8.1 |
| 1 cm/s | 0.50 | - | 0.21 | 0.31 | - | 0.31 | as at 8.1 |
| 0 | 0.26, 0.28 | 0.11 to 0.44 | 0.04, 0.07 | 0.16, 0.66 | 0.20, 0.55 | 0.21 | halved (the 720 s) |

SAME at 8 cm/s: 1.3 to 3.0, median 1.9: the first flight had about half the number, which is the published
"number down 1.7 to 2.7 times in 800 s" that I used to temper W0. It did not transfer to SAME-R.

Mass concentration at 8.1 cm/s, mg/m3: Kapton 1.2 to 3.5 at 690 to 800 K, 69 at 847 K (and 7.5 in run 70 at
850 K: a ninefold difference between two runs at one set point); lamp wick 27 to 37; Pyrell 58 to 179;
silicone 104 to 349; Teflon 8 to 86. I had assumed 5 to 15.

With the number fixed, `d30 = (6 c / (pi rho N))^(1/3)`: the "condensation size" of W0 is in fact the smoke
mass. Why the number is universal I can only suggest: 3.4e6 per cm3 is what coagulation leaves of any
denser smoke after about 500 s at `K`, or after a few seconds in the undiluted plume followed by a
hundredfold dilution. The second reading also gives a number that falls with the flow, as observed.

### 1.3 The smokes are broader than typed, and narrow faster

`sigma_g` of fresh smoke at 8.1 cm/s: Kapton 1.6 to 2.5, Teflon 1.8 to 2.8, Pyrell 2.1 to 2.3, silicone 2.2
to 2.6, lamp wick 2.5 (typed 1.6, 1.6, 1.8, 1.9, 1.9). Exponent `e` in `d10 ~ (N_1/N_2)^e` between the two
measurements of a run: silicone 0.59, 0.64, 0.59, 0.59; Pyrell 0.54, 0.57; Teflon 0.51; lamp wick 0.51;
Kapton 0.33 to 0.48 (SAME: 0.22 to 0.55). W0 typed 0.413. Lamp wick's count mean was over-forecast for this
reason alone: its `d30` (253 to 275 nm at 536 K) is what W0 said, its width is 2.5, not 1.9.

### 1.4 Size against material, flow, temperature and aging (W0 errors, ln forecast over outcome)

| Material | 8.1 cm/s, fresh | 8.1 cm/s, aged 200 s or more | 1 to 2 cm/s | No flow, fresh | No flow, aged |
|---|---|---|---|---|---|
| Kapton | +0.10, +0.11, -0.11, -0.06, -0.38, -0.12 | -0.17, -0.17, -0.12, -0.55, -0.40, +0.01 | +0.07, +0.42 (aged) | -0.30, -0.07 | +0.18, -0.09 |
| Lamp wick | +0.26, +0.32 | +0.07, -0.01 | -0.18 | -0.36, +0.37, +0.60 | -0.15, +0.20, +0.62 |
| Pyrell | -0.50, -0.37 | -0.60, -0.80, -0.94 | -0.57, -0.16, -0.14 (aged) | +0.65, +0.14 | -0.18 |
| Silicone | -0.05 to -0.27 (7 rows), -0.46 | -0.54, -0.61, -0.55, -0.71, -0.72 | +0.07, -0.35, -0.36 (aged) | -0.17, -1.18 | -0.33, -1.31 |
| Teflon | -0.30, +0.67, -0.25 | -1.05, +0.41 | -0.56 | +0.28, +0.13 | +0.56, +0.19 |

- **Aging** is the largest fault: every silicone and Pyrell row aged with flow is 0.5 to 0.9 low, Kapton at
  690 K 0.55 low; at 1 cm/s the aged Kapton row is 0.42 high (the law let a smoke of 0.5e6 per cm3 age like
  one of 5e6).
- **Material level**, fresh rows at 8.1 cm/s: Pyrell 0.4 low (it was run at 498 to 519 K, 55 K below my
  reference, and is the densest smoke after silicone); silicone 0.15 low; lamp wick 0.3 high (width);
  Kapton right; Teflon scattered (53, 132, 148 nm at one condition).
- **Flow**: 1 to 2 cm/s over 8.1 cm/s at the same temperature: 0.9, 1.0, 1.06, 1.5, 1.6, 1.9, 2.1; the typed
  1.42 stands.
- **Temperature**: steeper than typed where it can be seen (Kapton 59 nm at 690 K, 92 at 800, 148 at 847;
  silicone 185 at 624 K, 242 at 653 to 665; Pyrell 207 at 498, 254 at 515): 0.3 to 0.6 per 50 K against
  0.20. Not changed (section 5).
- **Heating time**: one pair (Kapton, no flow): 120 s gives 304 nm against 223 nm at 60 s, a power 0.45.

### 1.5 The no-flow rows: does the data support the 720 s?

In kind, yes; as a number, no. With no flow the smoke does coagulate in confinement: its number is 16 times
lower than at 8.1 cm/s, and afterwards it barely grows (aged over primary: 1.05, 1.09 Kapton; 0.91, 1.28,
1.11 lamp wick; 1.45 Pyrell; 1.26, 1.17 silicone; 0.94, 1.17 Teflon; median 1.15 in about 530 s, where W0
gave 1.05 to 1.33). But "720 s of extra aging at half the mass" is not what happens. The mass that reaches
the chamber is 8 to 17 mg/m3 for Kapton, lamp wick, Pyrell and Teflon alike, 3 to 10 times less than with
flow (silicone keeps its 120 to 146), and the number is 16 times less. The two nearly cancel in
`d30^3 = c/N`: `d30` is 1.4 to 2.1 times its value with flow, as W0 said, and `d10` 1.2 to 2.0 times for
Kapton, Teflon, lamp wick and silicone (W0: 1.8 to 2.1; median W0 error on the 21 no-flow rows +0.13).
Pyrell is the exception: 0.6 and 0.95 times, because its distribution becomes extremely broad at 1 cm/s
and below (`sigma_g` 3.1 to 3.5), so its count mean stays small while its `d30` doubles. The no-flow rows
scatter by 0.5 in ln whatever the law; two runs stand far out (below).

### 1.6 The counter and large smoke

Three runs give a count mean far above their neighbours: silicone 104 with no flow (940 nm, aged 1103;
the other no-flow silicone run 422), lamp wick 106 with no flow (442 nm; the other two 168 and 185), Teflon
69 (473 nm, with `d30` below `d10`). In each the reported width is the lowest of its material (1.66, 1.74,
none) and `d30` is 0.45 to 1.2 um. That is what an undercount of the counter does (fewer counts raise
M1/M0 and lower the width), and the readings (0.1 to 0.8e6 per cm3) are far below coincidence, so the
loss is of particles near and above 1 um, the counter's upper size. Five rows of 72. No input announces
them (Pyrell with no flow has the same `d30` and a normal count), so the law keeps them in its tail; they
are always on the large side, which a symmetric tail cannot say.

### 1.7 The detector

Rows at full scale over rows scored:

| Material | 8.1 cm/s | 1 to 2 cm/s | No flow | Mass concentration, mg/m3 |
|---|---|---|---|---|
| Silicone | 13 / 13 | 3 / 3 | 4 / 4 | 93 to 349 |
| Pyrell | 5 / 5 | 3 / 3 | 0 / 3 | 58 to 179 with flow; 11 to 17 with no flow |
| Lamp wick | 4 / 4 | 1 / 1 | 4 / 6 | 27 to 53 with flow; no flow: 11 to 15 at 570 K (yes), 8 at 533 K (no) |
| Kapton | 2 / 12 | 0 / 3 | 0 / 4 | yes at 69 to 75; no at 17 and below |
| Teflon | 0 / 5 | 0 / 1 | 0 / 4 | no even at 68 to 86 |

W0 forecasts: large smokes with flow 0.47 to 0.86 (observed 29 of 29); Kapton and Teflon with low or no flow
0.37 to 0.79 (observed 0 of 12). The cube-of-size gain I gave Kapton and Teflon at low and no flow is real
in size (their `d10` doubles) and absent at the detector, because their smoke mass falls 4 to 8 times at
the same temperature (Kapton at 850 K: 69 mg/m3 at 8.1 cm/s, 15 at 1 cm/s, 8 to 17 with no flow; mass
loss 3.75 mg against 1.3 to 1.4: its decomposition needs the air). Teflon never reaches full scale: weak
index, dense particle, and most of its lost mass is gas.

### 1.8 The band widths

W0: 38 of 70 rows in the 50 % band, 66 of 70 in the 90 % band. The 90 % band was right; the shortfall of
the 50 % band was the centre (material and aging), not the width. With the centre repaired (S1) the bands
hold 54 and 67 of 70: now wide for rows with flow (32 of 38 in the 50 % band), right with no flow (14 and
19 of 21). I leave the spread: the gain from narrowing is small (for a true error of 0.25 the expected
score is 0.151 at a width of 0.40 and 0.144 at 0.33), the residuals are in-sample, and Teflon alone still
scatters by 0.5.

## 2. Candidates: expectation, then result

Edits files and their sha256 are in the log; the lines are those of the block "Edits", cumulated.

| | Lines changed | Physical reason | Expected (written before the run) | Result |
|---|---|---|---|---|
| **C1** `5a74b1d5…` | aging hinges of `d10` and `d30`: no material and no temperature entry, `t_c` = 450 s, +0.30 per cm/s and +0.30 with no flow in `ln(t/t_c)`; `d10` exponent 0.55 | 1.1 to 1.3: one number for every smoke at 8.1 cm/s, falling with the flow; broader smokes | aged rows with flow gain 0.2 to 0.6 (silicone, Pyrell, cool Kapton), lamp wick 55a and 68 and Teflon 58a worse; Kapton 108a better by 0.42; no-flow aged Kapton and Teflon lower; `d10` 0.24 to about 0.19; lower on at least 9 days, worse on 2010-07-29 | `d10` 0.238 to 0.198 (train 0.272 to 0.215, validation 0.167 to 0.161); aged rows 0.282 to 0.195; lower on 11 of 12 days, worse on 2010-07-29 (0.144 to 0.163). Sign as predicted in every stratum. **Kept** |
| **C2** `8c7495a8…` | C1 plus `d10` levels: lamp wick -0.25, silicone +0.10, Pyrell +0.35 with -0.5 at no flow (unit 7), heating exponent 0.30 | 1.3 (lamp wick's width), 1.4 (Pyrell run 55 K below the reference, silicone the densest smoke), 1.5 (Pyrell's count mean does not rise at no flow), accumulation with heating time | lamp wick median error to about 0, three rows of 2010-09-09 worse; silicone median -0.185 to -0.085; Pyrell loss 0.213 to 0.14; `d10` to about 0.17, validation to about 0.13; lower than W0 on at least 11 of 12 days | `d10` 0.172 (train 0.191, validation 0.133); lamp wick median +0.026, silicone -0.085, Pyrell loss 0.140; lower on 12 of 12 days. As predicted. **Kept** |
| **C3** `1d30e80f…` | detector: +1.5 for the three large smokes (b0, with Kapton and Teflon held), no flow -0.6 to -2.6, silicone exempt (unit 1 re-typed, +1.5), low-flow gain of Kapton and Teflon removed | 1.7: denser smokes; no flow costs 3 to 10 times the mass except for silicone; the size gain of Kapton and Teflon is cancelled by their loss of mass | large smokes with flow to 0.80 to 0.97; Kapton and Teflon no flow to 0.02 to 0.05; Pyrell no flow 0.45; lamp wick no flow at 570 K worse; Brier 0.175 to about 0.06, validation to about 0.06; lower on all of about 10 days | Brier 0.056 (train 0.049, validation 0.071); by material Kapton 0.228 to 0.086, Teflon 0.145 to 0.011, silicone 0.131 to 0.017, Pyrell 0.226 to 0.071, lamp wick 0.142 to 0.102; lower on 10 of 10 days. As predicted (the logged note counts 34 large-smoke rows with flow; there are 29). **Kept** |
| **S1** `b9a15a81…` | C2 and C3 in one file (different networks) | - | reproduces C2 and C3 exactly | `d10` 0.172, 12 of 12; detector 0.056, 10 of 10 |

No candidate was dropped; no number was changed after a run. The fifth evaluation was not used.

By day, W0 against S1:

| Day | Rows | `d10` W0 | `d10` S1 | Detector W0 | Detector S1 |
|---|---|---|---|---|---|
| 2010-07-16 | 1 | 0.273 | 0.166 | 0.251 | 0.034 |
| 2010-07-20 | 1 | 0.164 | 0.164 | 0.013 | 0.013 |
| 2010-07-21 (validation) | 2 | 0.112 | 0.096 | 0.001 | 0.001 |
| 2010-07-22 | 1 | 0.151 | 0.116 | 0.118 | 0.011 |
| 2010-07-29 | 4 | 0.144 | 0.109 | 0.073 | 0.008 |
| 2010-07-30 | 6 | 0.282 | 0.201 | 0.104 | 0.015 |
| 2010-08-02 | 4 | 0.223 | 0.160 | 0.316 | 0.316 |
| 2010-08-23 | 8 | 0.343 | 0.105 | 0.197 | 0.024 |
| 2010-08-24 | 5 | 0.325 | 0.211 | 0.093 | 0.023 |
| 2010-08-25 | 1 | 0.100 | 0.096 | 0.182 | 0.020 |
| 2010-09-07 (validation) | 9 | 0.160 | 0.131 | 0.176 | 0.113 |
| 2010-09-08 (validation) | 12 | 0.182 | 0.140 | 0.305 | 0.051 |
| 2010-09-09 | 16 | 0.286 | 0.265 | 0.143 | 0.034 |

## 3. The proposal: S1, a revision of `samer` W0

```text
ln d10 = level[m] + 0.20 (T - T_ref)/50 K + f(U) + no-flow terms + 0.30 ln(t_h/60 s) + 0.55 softplus(y)
y      = ln(t / 450 s) + 0.30 (U - 8.1 cm/s)/(1 cm/s) + 0.30 [U = 0]          (no material, no temperature)
```

- One coagulation time for every smoke at 8.1 cm/s: 450 s (3.4e6 per cm3 at `K` of 5.5 to 6e-16 is 490 to
  530 s; 450 s leaves room for the larger `K` of the small smokes). It lengthens 1.35 times per cm/s below
  8.1 cm/s: 6 times at 2 cm/s, 8 times at 1 cm/s and with no flow.
- `d10` grows as the power 0.55 of `1 + t/t_c`; `d30` keeps 1/3.
- Levels of `d10` at the W0 reference temperatures, 8.1 cm/s, unaged: lamp wick 129 nm (was 166), Kapton 94
  and Teflon 104 (unchanged), silicone 227 (was 205), Pyrell 271 (was 191); Pyrell with no flow 0.61 of
  what the others' no-flow term gives.
- Heating time: power 0.30 (was 0.10): with no flow the smoke accumulates near the sample, a cube root.
- Detector: logit +1.5 for lamp wick, Pyrell and silicone; no flow -2.6 (was -0.6), silicone exempt; no gain
  for Kapton and Teflon at low or no flow. The aging term and the temperature slope are W0's.
- Spread, tails, flow law, temperature slope, Kapton and Teflon levels, `d30` levels: W0's.

### For the engineer

- A revision of the W0 release of slug `samer`: same declaration, same inputs, no new feature. Apply the block
  "Edits" (it is the logged file `s1.wsp`, sha256 `b9a15a81…4872`, 103 elements).
- Two hinge units change role. `d10.median` unit 7 was unused in W0 and now carries Pyrell with no flow.
  `iss_full_scale.logit` unit 1 was "Kapton or Teflon with no flow" and is now "silicone with no flow": its
  entries for features 0 and 1 are written to 0 and feature 2 to 1; its bias and its no-flow entry stay.
- Keep rule (13.6): `d10` lower pooled and on 12 of 12 days that change; detector lower pooled and on 10 of
  10; neither target is worse. More than six days change for both.
- `d30` gets the aging change only and is not scored. Its level is known to be 0.3 to 0.65 low (the smokes
  are broader than typed, so for the same `d10` the `d30` is larger); I did not type an unchecked fix.
- If the slug for aged rows given their primary measurement is ever built (W0 section 7.5), the data support
  `ln d10(t2) = ln d10(t1) + 0.55 ln(1 + K N_1 (t2 - t1))` with `K` = 5.5e-16 m3/s for a count mean above
  150 nm and 1.3e-15 below 100 nm, in place of 0.413 and 7e-16. The aged smoke is not diluted (mass
  concentration of aged rows is that of their primary rows to 10 %).
- A missing input, named as 13.6 asks: nothing known before a test tells run 62 (69 mg/m3) from run 70
  (7.5 mg/m3) at the same set point. The mass loss does, and it is weighed after the flight.

## 4. What stays unknown

- **Why the number is the same for every smoke.** Plume coagulation followed by dilution, a property of
  the counter and its diluter, or both. SAME had half the number, so 450 s is a SAME-R constant.
- **The temperature slope.** Steeper than 0.20 per 50 K for Kapton, silicone and Pyrell. Changing it means
  re-centring every material at the temperature it was run at; that is a second step for new data. S1
  leaves Kapton at 847 K 0.3 to 0.4 low.
- **Lamp wick at 570 K** (runs 106, 110: 0.4 to 0.6 low after S1) against lamp wick at 533 to 569 K with
  no flow (0.1 to 0.35 high): one of them is the undercount of 1.6.
- **Teflon's size**: 53 to 148 nm at one condition, 473 nm after 745 s. Left at W0.
- **Flow between 8.1 and 2 cm/s for the number.** The typed slope gives a sixth of the number at 2 cm/s; the
  four fresh rows show a third. No aged row at 2 cm/s exists in development; one on a test day would be
  forecast about 0.1 low in ln for it.
- **Thin support**: the heating exponent rests on one pair of runs and a cube-root argument; Pyrell with no
  flow on two runs; silicone's exemption at the detector on two runs. Each is typed because the physics
  agrees, and each is where a test day could contradict S1.
- **The detector for Kapton at about 850 K with flow**: full scale on one run of two; the law says 0.2.

## Edits

Changed lines only; a revision of the `samer` W0 release.

```text
# samer S1: study revision of W0 (docs/wsp/research/phys_samer_study.md). Author claude-fable-5-2. Data-informed.
# aging: one number for every smoke at 8.1 cm/s (t_c 450 s), falling with the flow; d10 as the power 0.55
d10.median.W1[3, 0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.30, 0.30, 1.0, 0.0]
d10.median.W1[4, 0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.30, 0.30, 1.0, 0.0]
d10.median.W1[5, 0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.30, 0.30, 1.0, 0.0]
d10.median.W1[6, 0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.30, 0.30, 1.0, 0.0]
d10.median.b1[3:7] = [0.996, -0.604, -2.404, -4.004]
d10.median.w2[3:7] = [0.11, 0.165, 0.165, 0.11]
d30.median.W1[3, 0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.30, 0.30, 1.0, 0.0]
d30.median.W1[4, 0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.30, 0.30, 1.0, 0.0]
d30.median.W1[5, 0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.30, 0.30, 1.0, 0.0]
d30.median.W1[6, 0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.30, 0.30, 1.0, 0.0]
d30.median.b1[3:7] = [0.996, -0.604, -2.404, -4.004]
# levels of d10: lamp wick 22 % smaller (broader than typed), Kapton and Teflon kept, silicone +0.10, Pyrell +0.35
d10.median.b0 = -15.412
d10.median.skip[0:4] = [-1.416, -1.133, 0.125, 0.704]
# Pyrell with no flow: the count mean does not rise (sigma_g 3.2 to 3.5)
d10.median.W1[7, 3] = 1.0
d10.median.W1[7, 6] = 1.0
d10.median.b1[7] = -1.0
d10.median.w2[7] = -0.5
# heating time: the smoke accumulates, cube root
d10.median.skip[8] = 0.30
# detector: the three large smokes are denser than assumed (+1.5), Kapton and Teflon held with flow;
# no flow costs the smoke that reaches the detector, except silicone; no low-flow gain for Kapton and Teflon
iss_full_scale.logit.b0 = 3.956
iss_full_scale.logit.skip[0] = -7.57
iss_full_scale.logit.skip[1] = -7.56
iss_full_scale.logit.skip[6] = -2.6
iss_full_scale.logit.w2[0] = 0.0
iss_full_scale.logit.W1[1, 0] = 0.0
iss_full_scale.logit.W1[1, 1] = 0.0
iss_full_scale.logit.W1[1, 2] = 1.0
iss_full_scale.logit.w2[1] = 1.5
```

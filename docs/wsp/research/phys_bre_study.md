# ACME BRE (PSI-20): a study of the development tests, and the third law (`d3-fable`)

Author: `claude-fable-5-2`, 15 November 2026, under PROTOCOL 13.6. **Data-informed, in-sample**: the
train and validation days were studied test by test (111 answered tests of 14 days, 151 with the
refused and unmeasured ones). No test day was opened. No git command. Throwaway tables:
`runs/dev/gen7/study/bre_*.py`; the five logged candidates: `runs/dev/gen7/study/bre_c1.wsp` to
`bre_c4.wsp` and `bre_d3.wsp`, outputs `bre_c*_out.txt` beside them, log
`runs/dev/gen7/whatif_log.jsonl`. Every number below was typed by me with its reason; the programs
made tables of the data and of my own closed form, and the logged tool scored the candidates.

## 0. The result

One number orders the outcomes: the size ratio `b = ln(xi_ss / xi_e)` of the W0 law, **once two
things I had added to it are taken out**: the burner's share of the heat at its adiabatic value, and
a pressure exponent. With them out, of 32 ethylene flames on the 25 mm burner with `b < 0` all 32
were still burning at the planned stop, and of 24 with `b > 0.2` all 24 went out. Pressure does
nothing that the data can show. The 65 to 85 kPa tests were held because they were run at low fuel
flux (median 1.8 g/m2/s) in 25 to 28 % oxygen, which puts half of them below the boundary; W0
charged low-flux flames a burner loss they do not pay. Both my pressure steps (D1, d2) pushed a
number that should have been zero.

| read through `lasts`, development, in-sample | extinction, pooled | train | validation | operator termination | days lower than W0 by `lasts` |
|---|---|---|---|---|---|
| W0 by `flame_end` (registered) | 0.2576 | 0.2511 | 0.2755 | 0.0730 | |
| W0 by `lasts` | 0.1955 | 0.1909 | 0.2083 | 0.0577 | |
| planned-stop baseline | 0.2269 | 0.2320 | 0.2129 | 0.0699 | |
| **third law (C3, proposed)** | **0.1121** | 0.123 | 0.083 | 0.0398 | 14 of 14 |
| third law with methane (C4, comparator) | 0.0980 | 0.103 | 0.083 | 0.0358 | 14 of 14 (one day differs from C3) |

## 1. What the data showed

### 1.1 The size ratio orders the outcomes

`b` here is the closed form of section 2 (no pressure, burner share at the extinction temperature).
Tests above the stand-off limit (`Lambda0 d / 25 mm` of 2.3 or more); ends in seconds after onset.

| `b` | ethylene, 25 mm: went out (at) | burning at the stop (stops) |
|---|---|---|
| below -0.3 | 0 of 8 | 8 (65 to 184 s) |
| -0.3 to -0.1 | 0 of 16 | 16 (65 to 184 s) |
| -0.1 to 0 | 0 of 8 | 8 (52 to 193 s) |
| 0 to 0.1 | 3 of 10 (50, 132, 158) | 7 (65 to 184 s) |
| 0.1 to 0.2 | 9 of 11 (36, 58, 58, 77, 79, 86, 88, 99, 130) | 2 (97, 123 s) |
| 0.2 to 0.35 | 8 of 8 (30, 37, 40, 43, 66, 74, 80, 149) | 0 |
| 0.35 to 0.5 | 7 of 7 (19, 21, 27, 33, 36, 45, 91) | 0 |
| 0.5 to 0.7 | 5 of 5 (16, 18, 25, 32, 36) | 0 |
| above 0.7 | 4 of 4 (14, 14, 19, 27) | 0 |

The boundary sits at `b` between 0 and 0.15 on every 25 mm ethylene day that has tests near it, with the
constants W0 took from the literature (`rhoD` 6.0e-5 kg/m/s, critical flux 9.4 kW/m2) untouched.

### 1.2 Pressure

Pairs at the same oxygen, fuel and flux:

| tests | X_O2 | pressure kPa | fuel flux g/m2/s | life s |
|---|---|---|---|---|
| 19051B1 / 19051F1 | 0.331 / 0.326 | 101.8 / 57.0 | 3.09 | 99 / 130 |
| 19039C1 / 19051K1 | 0.265 / 0.259 | 99.0 / 69.0 | 3.86 | 26.7 / 24.9 |
| 19056B1 / 19056F1 (50 % bottle) | 0.257 / 0.252 | 100.9 / 68.9 | 2.25 | 36.8 / 40.0 |

Flames at 2.0 to 2.6 g/m2/s in 34 to 36 % oxygen are held at 57 kPa and at 102 kPa alike. Neither the boundary nor the life moves with
pressure by more than the scatter (ratios 1.31, 0.93, 1.09). This is what the published abstract
said ("independent of pressure") and what W0 section 6 listed as a way the law could be wrong.

Why the reduced-pressure strata looked "held":

| pressure band (25 mm, ethylene, above the stand-off) | tests | fuel flux, median (range) | X_O2 | `b` below 0.1 | out by 60 s | went out at all |
|---|---|---|---|---|---|---|
| 65 to 85 kPa | 15 | 1.80 (0.99 to 3.86) | 0.25 to 0.29 | 8 | 2 | 9 |
| below 65 kPa | 20 | 2.01 (1.04 to 7.01) | 0.31 to 0.36 | 15 | 3 | 5 |
| 85 kPa and above | 42 | 2.25 (0.95 to 6.09) | 0.20 to 0.40 | 19 | 18 | 22 |

The experimenters ran the reduced-pressure atmospheres near the holding flux. At 70 kPa and 27 %
oxygen the flame is held up to 1.5 g/m2/s, lives 130 to 160 s at 1.7 to 1.8, 75 to 90 s at 1.8 to
2.5 and 25 s at 3.9. W0 forecast 45 % out by 60 s there because its burner term raised `b` by 0.2
to 0.4 at 1 to 2 g/m2/s.

### 1.3 The life of flames that go out

For flames with `b > 0.12` the life is 0.42 of the W0 clock taken without its pressure factor
(`R^2 xi_e^2 / D0` times `exp(G(b))`): the mean of `ln(observed / law)` is -0.87 + (0.07, -0.17,
-0.15, 0.00, +0.27) in the five groups (26 % at 1 atm, 26 % at 70 kPa, 34 % at 57 kPa, air, 39 %),
sd 0.14 to 0.37 inside a group (computed with the far-field `b`, which differs from 2.4 by under 0.1). The shape `G(b)` holds: the life falls steeply just above the
boundary and flattens (14 to 19 s in air from 3 to 5 g/m2/s). It does not grow with the flux in
the range flown, as W0 said it would far from the boundary; no test is that far.

### 1.4 The smallest flames

Below a fuel heat-release flux of about 41 kW/m2 on the 25 mm burner (`Lambda0 d / 25 mm` below
2.2, 0.88 g/m2/s of ethylene) the flame does not stand: it dies 3 to 9 s after onset.

| `Lambda0 d / 25 mm` | tests | outcome |
|---|---|---|
| 1.0 to 1.6 | 19066A2, A3, 20261A2, 20346H5 | all out at 3 to 9 s |
| 1.8 to 2.05 | 20261H1, 20261C1, 19070J1, 20247J1 (50 mm), 19070J2 | out at 84 s, held 130 s, out at 4 s, out at 22 s, out at 4 s |
| 2.21 | 19066D1, D2 | out at 4 s |
| 2.25 to 2.30 | 19066D5, 19070L1, 19066D4 | held to 184 s |

The limit is sharp and noisy: 0.88 g/m2/s died twice and 0.90 to 0.91 held three times on the same
day. Above it the burner costs nothing visible (see 2.1). The W0 small-flame hinge began far too
high (it penalised pure ethylene at 1.1 g/m2/s, held for 184 s).

### 1.5 The 50 mm burner, methane

- **50 mm** (two days): the same ordering by `b`, but flames live 1.3 to 2.3 times longer than the
  law at `b` 0.35 to 0.7 (four held to stops of 74 to 136 s where the law's median is 35 to 60 s). On one
  day the flame was lit at 1.5 to 3.8 times the test flow; on the other it was not, and both show
  it. I found no physical reason and typed nothing for it (section 6).
- **Methane** (one day, 12 flames): every flame went out, at 7 to 9 s in 25 % oxygen and at 14 to
  30 s in 38 %, where ethylene at the same `b` is held or lives 80 s. It behaves as ethylene would
  with `b` larger by 0.3 and a clock 0.55 as long: a critical flux 1.7 to 1.8 times that of ethylene.

### 1.6 The heat flux to the burner (the lane's second target)

Centre gauge, mean over 8 to 12 s, against half the face-mean flux of the Burke-Schumann solution
the law assumes, `q_BS = (4 rhoD (dh_c / S) Y_O / (pi R)) x / (e^x - 1)`, `x = pi Pe / 2`:

| group (ethylene) | tests | measured / theory, median (10 to 90 %) |
|---|---|---|
| 25 mm, burning at the stop | 45 | 0.51 (0.36 to 0.63) |
| 25 mm, went out within 50 s | 21 | 0.36 (0.27 to 0.52) |
| 25 mm, went out later | 16 | 0.43 (0.32 to 0.51) |
| 50 mm, all | 22 | 0.93 (0.74 to 1.02) |

The measured flux falls with the fuel flux, as the theory says (5.2, 4.5, 3.6, 3.0, 2.0 kW/m2 at
1.2, 1.5, 1.9, 3.1, 3.9 g/m2/s in air). The ratio is the flame's temperature excess over its
adiabatic value: about one half in a flame that will last, less in one that will not, and 0.26 to
0.29 in the pure-ethylene flames within 15 to 55 s of going out (19094H1, J1, K1, 19051G1, K1,
19039F1). The law's `theta_c` (the excess at which the sheet goes out, 1130 K) is 0.2 to 0.32. The
gauges therefore confirm the extinction temperature and show what W0 got wrong (2.1).

### 1.7 Stops, script, date

Every operator termination is the planned stop (one came 11 s early). No flame with `b < 0` gives
any sign of what it would have done after the stop. The planned stop carries no information beyond
truncation: tests planned for 50 to 75 s and for 180 s split by `b` in the same way. The 2019 and
2020-21 days lie on the same line. The 39 refused train cases are bottles diluted below 30 % fuel at
the burner and fluxes below 0.8 g/m2/s; they follow the same ordering (table of 1.4, and 20261H2,
G2) and the domain is widened for them.

## 2. Mechanism: what W0 had right and what it had wrong

Right, and kept with their published constants: the steady height `xi_ss = cot(L / Pe)`, the
extinction height from a critical release per sheet area, the growth curve `G`, the extinction
temperature.

### 2.1 The burner's share was charged at the adiabatic flame temperature

W0 wrote the sheet's balance as `theta = 1 - phi_b - chi_R` with `phi_b` the Burke-Schumann share
of the heat conducted to the burner. That share is for an adiabatic flame. Conduction to the
burner follows the actual temperature excess, so the share is `phi_b theta`, and at extinction
`theta = theta_c`:

```text
theta_c = 1 - phi_b theta_c - chi_R      =>      q_x = q_c0 (1 - theta_c) / (1 - theta_c (1 + phi_b))
```

instead of `(1 - theta_c) / (1 - theta_c - phi_b)`. For `phi_b = 0.44` (1.1 g/m2/s) the factor is
1.18 instead of 2.5; `b` rises by 0.08 instead of 0.45. The gauges of 1.6 measure exactly this
reduced share.

### 2.2 No pressure exponent

`q_x ~ p^a` with `a = 0.5` (W0), 0.8 (D1, d2): the data give 0 within about 0.2 (1.2). The clock
loses its pressure factor too: the lives of 1.2 do not shorten at low pressure as a diffusion
time `R^2 p / D0` would. I do not have a derivation of why the two cancel so completely; a critical
flux and a diffusivity that both scale with density would do it.

### 2.3 The clock

`D_eff = 5.1e-5 / 0.42 = 1.2e-4 m2/s`, without pressure. W0 read 5.1e-5 from five seconds of drop
tests; the flame grows through gas at 1000 K and more, where the diffusivity is 1.6e-4.

### 2.4 The third law, closed form

```text
Pe, L, Z_st, phi_b, theta_c as in W0 section 3
Lambda0 = m_fuel dh_c / (2 q_c0)                      no pressure
A = (1 - theta_c) / (1 - theta_c (1 + phi_b))
xi_e = sqrt(Lambda0 / A - 2/3)          xi_ss = cot(L / Pe)          b = ln(xi_ss / xi_e)
flame does not stand if Lambda0 d / 25 mm < about 2.2
held if b < 0;  else  t_ext = 0.42 (R^2 xi_e^2 / D0) exp(G(b))
```

Spread: 0.31 in `b` (was 0.425). In-sample the boundary is located to 0.07; I keep four times that
for days not seen.

## 3. Candidates: expectation, then result

All on W0, read through `lasts` unless said; each is cumulative. Scores: extinction, pooled.

| | change | expected (written in the log before the run) | realised |
|---|---|---|---|
| C1 | `b` of 2.4 (burner share cut, no pressure), base pressure-free, one stand-off hinge | 0.1955 to about 0.165; days 19070, 19066, 19087, 20352 lower; 65 to 85 kPa forecast toward 0.3; 19094 lower; air and methane about unchanged; 20247 and 21028 worse; at least 8 of 14 days lower | **0.1444**; the four days lower (0.24 to 0.09, 0.25 to 0.13, 0.25 to 0.08, 0.31 to 0.05); 65 to 85 kPa 0.57 to 0.33; 19094 lower; 21028 worse; 9 of 14 days. Not predicted: methane worse (0.26 to 0.34), 20247 better, 19056 and 19063 slightly worse |
| C2 | C1 + clock 0.42 (base - 0.87) | 0.144 to about 0.125; air forecast 0.585 to about 0.9; 19039, 19056, 19063, 19094, 21028, 20339, 19051 lower than C1; 20261, 20247, 19087, 19070 slightly higher; 20352, 19066 unchanged | **0.1271**; air 0.585 to 0.71; 19039, 19063, 19094, 21028, 20339, 19051 lower; 20247, 19087, 19070 higher; 13 of 14 days below W0. Not predicted: 19056 unchanged, 20261 slightly lower, 20352 higher (0.05 to 0.07) |
| C3 | C2 + spread 0.425 to 0.31 (`c` 4.0 to 5.5) | 0.127 to about 0.112; held days, air, methane, 19063, 19094, 21028, 19051, 19056 lower; 19066, 20346 about unchanged; 20247 unchanged or worse | **0.1121**; every named day lower; 19066 0.136 to 0.146; 20346 lower; 20247 unchanged; 14 of 14 days below W0 |
| C4 | C3 + methane critical flux 1.7 times | only 20339 changes: 0.192 to about 0.08; methane forecast 0.53 to about 0.9; pooled to about 0.102 | **0.0980**; 20339 0.192 to 0.039; methane forecast 0.83 |
| C5 | the complete file of section 5, read through `flame_end` (not the scoring rule) | 0.2576 to about 0.13; at least 12 of 14 days below W0's clock; worse than `lasts`; worst days 19087, 20261, 19051, 20247; 20339 about 0.2 | **0.120**; 13 of 14 days; train 0.138, validation 0.069. Worst days were 19066, 20339 (0.30, above W0's clock), 20247; 19087 and 20261 were among the best |

C1 alone fails the two-thirds rule and several of its day predictions; it is not proposed alone.
C3 is lower than W0 by `lasts` pooled and on 14 of 14 days, with operator termination lower (0.0398
against 0.0577, lower on 11 of 14 days). C4 changes one day, so by 13.6 it is a comparator.

After C3 the largest remaining losses by day are methane (0.19), 20247 (0.20, 50 mm), 19066 (0.15,
the 0.88 g/m2/s deaths) and the near-boundary days where nothing went out by 60 s and the forecast
is 0.26 to 0.32 (19087, 20261).

## 4. What is proposed

- **Model: `d3-fable`**, a revision of W0 on the same slug and declaration, scored by the `lasts`
  rule of `phys_bre2.md` section 2. Its `lasts` lines are C3. Its `flame_end` and `height_at_end`
  are brought to the same law so that the release does not carry two physics; they are not used by
  the score.
- **Comparator: `d3m-fable`** = d3 plus the methane lines (C4). One day of evidence; the test days
  hold a methane day, which will say whether it carries.

## 5. For the engineer

- Slug `bre`, declaration unchanged, no new input. Parent: W0 `sha256:aa0eef5b...2b4d`
  (`configs/models/law/bre/w0-fable`).
- Scoring-time rule: unchanged from `phys_bre2.md` section 2 (extinction by `h` =
  `1 - lasts(min(h, stop)) / lasts(c)`; operator termination `lasts(stop) / lasts(c)` from the stop).
- The file is `runs/dev/gen7/study/bre_d3.wsp`, the fifth logged candidate (sha256
  `defb7d19fc3730baa53011fe4c4840488f2b7af3a54a7d0c895ad05c9dbe3e04`); the block below repeats its
  assignment lines (checked identical), and its `lasts` lines are those of `bre_c3.wsp`.
- The domain is widened (`x_fuel` from 0.2, flux from 0.3 g/m2/s). The newly admitted cases were not
  scored by the what-if (the base refuses them). Expected for them: the 20261 flames at 26 to 28 %
  fuel at the burner follow table 1.1; the 50 mm flames at 0.4 to 0.7 g/m2/s get lives of 20 to
  100 s; anything with `Lambda0 d / 25 mm` below 1.6 is forecast out within seconds.

### Edits (`d3-fable`)

Parent W0, all its lines unchanged except:

```text
# bre d3-fable: the third law (parent W0). z = (z_d, z_m, z_o, z_p, z_f, z_c, z_t)
# size ratio  b    = 0.514 + 1.01 z_d + 0.47 z_m - 0.93 z_o + 0.43 z_f + 0.11 z_c          (no pressure; burner share at the extinction temperature)
# clock       base = 2.451 + 2.05 z_d + 1.12 z_m - 0.03 z_o + 1.12 z_f + 0.08 z_c          (ln s; 0.42 of the W0 diffusion time; no pressure)
# stand-off   s    = 2.307 + z_d + z_m + z_f = ln(Lambda0 d / 25 mm); penalty below 0.90
x.x_fuel.lo = 0.2
x.burner_mass_flux.lo = 0.0003

# lasts: logit = 5.5 (H(v) - b) - 9 relu(0.90 - s), v = 0.950 + z_t - base(z)
lasts.logit.b0 = -2.83
lasts.logit.skip[0:7] = [-5.56, -2.59, 5.12, 0.0, -2.37, -0.61, 0.0]
lasts.logit.W1[0:6, 0] = 2.05
lasts.logit.W1[0:6, 1] = 1.12
lasts.logit.W1[0:6, 2] = -0.03
lasts.logit.W1[0:6, 3] = 0.0
lasts.logit.W1[0:6, 4] = 1.12
lasts.logit.b1[0:6] = [2.51, 0.26, -0.35, -0.86, -1.30, -1.72]
lasts.logit.w2[0:6] = [0.97, 0.56, 0.67, 0.87, 1.14, 1.62]
lasts.logit.W1[6, 0:7] = [-1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0]
lasts.logit.b1[6] = -1.41
lasts.logit.w2[6] = -9.0
lasts.logit.w2[7] = 0.0

# flame_end: ln tau = base + G(b) - 6 relu(0.90 - s); G on knots 1.8, 1.3, 1.0, 0.77, 0.57, 0.40, 0.25; 2.09 = 2.451 - 0.03 - 0.65 * 0.514
flame_end.extinction.median.b0 = 2.09
flame_end.extinction.median.skip[0:7] = [1.39, 0.81, 0.57, 0.0, 0.84, 0.01, 0.0]
flame_end.extinction.median.W1[0:7, 0] = -1.01
flame_end.extinction.median.W1[0:7, 1] = -0.47
flame_end.extinction.median.W1[0:7, 2] = 0.93
flame_end.extinction.median.W1[0:7, 3] = 0.0
flame_end.extinction.median.W1[0:7, 4] = -0.43
flame_end.extinction.median.W1[0:7, 5] = -0.11
flame_end.extinction.median.b1[0:7] = [1.286, 0.786, 0.486, 0.256, 0.056, -0.114, -0.264]
flame_end.extinction.median.w2[0:7] = [0.29, 0.36, 0.49, 0.70, 1.09, 2.09, 5.03]
flame_end.extinction.median.W1[7, 0:7] = [-1.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0]
flame_end.extinction.median.b1[7] = -1.41
flame_end.extinction.median.w2[7] = -6.0
flame_end.extinction.shape = 3.0

# height_at_end: ln(R xi_e / 1 m); -3.28 = ln(0.0125 * 3.01)
height_at_end.median.b0 = -3.28
height_at_end.median.skip[0:7] = [1.03, 0.56, -0.02, 0.0, 0.56, 0.04, 0.0]
```

Where the numbers come from (logarithmic derivatives of 2.4 at 25 mm, 4 g/m2/s, 26.5 % oxygen,
pure ethylene: `Pe` 0.417, `phi_b` 0.092, `A` 1.034, `xi_e` 3.01, `xi_ss` 5.03, `b` 0.514):
`d ln xi_ss / d ln Pe = 1.03`, `d ln xi_e / d ln Lambda0 = 0.536`, the burner factor adds -0.02 to
the flux and diameter coefficients; `base = ln(3.064 s x 9.04) - 0.87`; `lasts` skip is `-5.5 b`,
its hinge weights `5.5 x (0.176, 0.103, 0.122, 0.159, 0.207, 0.294)`, its hinge biases the knots
of `H` (3.46, 1.21, 0.60, 0.09, -0.35, -0.77) minus `ln 30 - 2.451 = 0.950`. Against the closed
form on the development inputs the linear `b` is within 0.05 where it matters and 0.1 to 0.3 too
high (towards extinction) for flames deep in the held region; the linear `base` is 0.2 to 0.4 too
long below 1.3 g/m2/s.

### Edits (`d3m-fable`, comparator)

`d3-fable` and, in addition:

```text
# methane: critical flux 1.7 x ethylene. b: 0.11 -> 0.30 on is_methane; base: 0.08 -> -0.60
lasts.logit.skip[5] = -1.65
lasts.logit.W1[0:6, 5] = -0.60
flame_end.extinction.median.skip[5] = -0.80
flame_end.extinction.median.W1[0:7, 5] = -0.30
height_at_end.median.skip[5] = -0.24
```

(In a file these replace the `skip[0:7]` and `W1[..., 5]` lines above rather than follow them; the
compiler refuses an element written twice.) The two `lasts` lines are C4 as logged; the
`flame_end` and `height_at_end` lines are the same step (`-0.60 - 0.65 x 0.30`, `-0.30`,
`-0.27 + 0.03`) and were not run. Physical reason: methane needs a hotter sheet than ethylene to
stay lit, so at its limit the gas radiates more; 1.7 corresponds to about 1350 K against 1130 K. The
coefficients also absorb that a methane-nitrogen mixture carries less fuel per mole fraction than
`ln x_fuel` assumes (the mean over the bottle fractions flown).

## 6. What remains, and what is structural

- **The 50 mm burner lives 1.3 to 2.3 times longer than the law.** No reason found, two days; a
  number typed for it would be a fit. If the test days confirm it, it wants a mechanism first.
- **Whether a small flame stands** is close to a coin toss at the limit (0.88 against 0.90 g/m2/s).
  It is a separate yes/no about ignition, and the inputs that might decide it (the igniter, the
  flow at ignition) are not in the declaration. A target `stands` with those inputs would be a new
  slug; three or four tests per day are at stake.
- **Flow history.** On 20247 the flame was lit at up to 3.8 times the test flow and held there for
  8 s; from 2020 nitrogen is added at the burner. The law reads only the test flow. A new input
  (ratio of ignition to test flow) would be a new slug; the data here do not show that it matters.
- **Beyond the stop nothing is known.** Seven of ten flames with `b` between 0 and 0.1 were stopped
  at 65 to 184 s. The growth curve says such flames live 200 to 1000 s; the three that went out did
  so at 50 to 158 s. The steep part of `G` is unverified and the forecast there stays diffuse
  (0.26 to 0.32 out by 60 s on days where nothing went out).
- **Methane** rests on one day.
- **The heat flux (T2)** is not a target of this slug. The study gives a law for it: the centre flux
  at 8 to 12 s is `theta` times half the Burke-Schumann face mean of 1.6, `theta` about 0.5 for a
  25 mm flame that lasts, 0.36 for one that goes out within 50 s, 0.9 on the 50 mm burner. As a
  positive target it needs a new slug; it could be typed in an hour.
- **My earlier steps.** D1 and d2 raised the pressure exponent from 0.5 to 0.8. The data put it
  at 0. They were typed from stratum summaries in which pressure stood in for the fuel flux.

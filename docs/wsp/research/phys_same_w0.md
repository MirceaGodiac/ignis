# SAME (PSI-102): smoke size and the ISS detector on the first flight, with a liquid reference. A W0 law

Author: claude-fable-5-2, physicist-author of the seventh generation, 15 November 2026. Zero-shot. SAME is
held whole: no SAME outcome was opened, and none is in the engineer's note. This file applies the law of
[phys_samer_w0.md](phys_samer_w0.md) (same hardware, same instruments, flown three years later) to the
conditions of the first flight and adds its fifth material, dibutyl phthalate. The derivations are there
(sections 1.3 to 1.7); here they are restated in short, with what is specific to SAME derived in full.

## 0. The result in one screen

```text
ln D_n  = ln D_ref[m] + 0.25 ln(8.1 cm/s / max(U, 2 cm/s))
ln d10  = ln D_n - ln^2 sigma_0[m] + 0.413 ln(1 + t / t_c[m])           (count mean diameter, M1/M0)
ln D30  = ln D_n + (1/3) ln(1 + t / t_c[m])                             (diameter of average mass)
```

| Material | `D_ref` nm | `sigma_0` | `t_c` s | d10 at 8.1 cm/s, 40 s | after 1800 s | P(ISS scatter at full scale), 40 s |
|---|---|---|---|---|---|---|
| Kapton | 115 | 1.6 | 400 | 94 nm | 186 nm | 0.15 |
| Teflon | 130 | 1.6 | 500 | 104 nm | 196 nm | 0.10 |
| Lamp wick | 250 | 1.9 | 1000 | 166 nm | 257 nm | 0.75 |
| Silicone rubber | 310 | 1.9 | 1400 | 205 nm | 297 nm | 0.70 |
| Dibutyl phthalate | 400 | 1.5 | 2500 | 339 nm | 436 nm | 0.90 |

One standard deviation is a factor 1.5 (ln-sd 0.42), 1.65 for dibutyl phthalate. Sizes are 1.13 times
larger at 5 cm/s and 1.42 times at 2 cm/s.

## 1. Mechanism

### 1.1 What is common with SAME-R

A wire heats the sample below ignition for about a minute. The vapour condenses in the first millimetre
(the condensation size `D_n`, set by the material and by the flow that cools and dilutes the vapour); a
piston draws the smoke into the aging chamber; there Brownian coagulation at conserved mass makes the mean
particle volume grow linearly in time, `v = v_0 + K phi t` with `K` = 6.0e-16 m3/s, so that
`D30 = D_n (1 + t/t_c)^(1/3)` with `t_c = 1/(K N_0)`, and narrows the distribution, so that the count mean
diameter grows as the power 0.413. The groups are the coagulation number `Co = t/t_c`, the Knudsen number
of the particle (slip in `K`), the Peclet number of the vapour at the sample (flow law, flat below about
2 cm/s), and the optical size `pi d / lambda` (the scattering detector).

### 1.2 Dibutyl phthalate: evaporation and recondensation of an intact liquid

The four polymers decompose; dibutyl phthalate (DBP, boiling point 613 K, liquid density 1050 kg/m3, held
in a cotton wick) only evaporates. Three consequences.

1. **Nearly all the vapour comes back as aerosol.** Its saturation vapour pressure at 296 K is of order
   1e-3 Pa, that is 0.1 mg/m3 of vapour; anything above that condenses. A polymer gives a few per cent of
   its lost mass as particles, DBP nearly 100 %. For the same heating the chamber smoke is several times
   denser: I take 30 mg/m3 (factor 3).
2. **Few nuclei, large drops.** At the wick (400 to 450 K is enough for a milligram per minute) the
   saturation ratio after cooling is high but far below that of a polymer fragment, and the vapour is
   plentiful: the first droplets take up vapour all along the boundary layer. Condensation aerosols of
   such liquids made by cooling a hot vapour come out at 0.3 to 1 um and fairly uniform. I type
   `D_ref` = 400 nm (factor 1.6) and `sigma_0` = 1.5.
3. **Slow coagulation, no evaporation.** 30 mg/m3 in 0.4 um drops is 8.5e11 per m3; with `K` = 4.6e-16 m3/s
   at that size, `t_c` = 2500 s: the count mean diameter grows by 25 % in 1800 s, against 50 to 100 % for
   the polymers. Loss by evaporation to the walls is limited by the diffusion of 0.1 mg/m3 of vapour
   through the stirred boundary layer: about 2 mg/m3 in 1800 s, 2 % of the diameter. So DBP is the nearly
   inert reference: if its `d10` moves with aging by more than a factor 1.5, something other than
   coagulation acts in the chamber (dilution, wall loss), and the polymer results must be read with it.

For the scattering detector DBP sits near the top of the response (`d_sc = D30 exp(3 ln^2 sigma)` = 0.65 um,
index 1.49) and is dense: P(full scale) = 0.90.

### 1.3 Gravity (the question SAME was flown for)

On the ground the hot sample makes a plume of about `sqrt(g beta dT d_s)` = 20 cm/s; the speed at the
sample is then `sqrt(U^2 + 20^2)` cm/s whatever the duct flow. By the flow law the low-gravity particles
are larger by `(21.5/8.1)^0.25` = 1.28 at 8.1 cm/s, `(20.6/5)^0.25` = 1.42 at 5 cm/s and
`(20.1/2)^0.25` = 1.78 at 2 cm/s. The card quotes about 50 % for the count mean diameter (class L). Every
SAME flight row is in microgravity, so gravity is not an input here; the statement is for the ground runs.

## 2. Targets (from the engineer's note)

`docs/wsp/feedback/same_feasibility.md`, sections "For the physicist", 2 and 3: as SAME-R.

| Target | Definition | Unit | Score | A forecast may know |
|---|---|---|---|---|
| `d10` | count mean diameter M1/M0 of the row at its `net_aging_time` | m | quantile CRPS of ln(d10), levels 5/25/50/75/95 % | material (Kapton, Teflon, silicone, lamp wick, dibutyl phthalate), primary or aged row, inlet velocity, heater set point, durations, net aging time (the horizon), sample temperature (41 of 45 rows) |
| `iss_full_scale` (optional) | ISS detector scatter channel at full scale, \|V\| >= 6.0 V | yes / no | Brier | the same |
| `d30` (mine, unscored) | diameter of average mass | m | | the same |

I do not use the sample temperature: four rows lack it and would be refused, and I have no reference
temperature for the DBP wick. The law is evaluated at each material's reference condition, and the spread
is widened for it (0.42 instead of 0.40).

## 3. The law

```text
inputs: material m, inlet velocity U (m/s), net aging time t (s)

f(U)    = 0.25 ln(8.1 cm/s / max(U, 2 cm/s))        typed as a slope and two hinges in U
ln d10  = ln D_ref[m] - ln^2 sigma_0[m] + f(U) + 0.413 softplus(ln(t / t_c[m]))
ln D30  = ln D_ref[m] + f(U) + (1/3) softplus(ln(t / t_c[m]))
ln-sd   = 0.42; + 0.10 for dibutyl phthalate; + 0.03 per e-fold of t beyond 270 s
tails   = 10 % of the weight in a component 2.5 times wider

logit P(full scale) = 1.10 (lamp wick), 0.85 (silicone), -1.73 (Kapton), -2.20 (Teflon), 2.20 (DBP)
                      + 0.10 per cm/s below 8.1
                      + 0.9 at 2 cm/s for Kapton and Teflon (linear in U from 8.1 cm/s)
                      + 0.3 per e-fold of t beyond 270 s for Kapton and Teflon
```

The width: condensation size of the material 0.33, unknown heater set point 0.20 (40 K at 0.20 per 50 K,
and its effect on the smoke mass), `sigma_0` 0.10, repeatability 0.10. The error is mostly common to the
rows of one material.

## 4. Constants

Class: P property table, T theory, L published about this experiment, J judgement. Those shared with
SAME-R are listed there (section 4) with their derivation; values repeated here.

| Symbol | Value | Class | Uncertainty | Origin |
|---|---|---|---|---|
| `K` | 6.0e-16 m3/s | T | 25 % | Brownian coagulation with slip, 0.2 to 0.3 um |
| Exponents on `1 + Co` | 1/3 (D30), 0.413 (d10) | T | 0.06 on the second | conserved mass; Lee's narrowing |
| `D_ref` Kapton, Teflon, lamp wick, silicone | 115, 130, 250, 310 nm | J | factor 1.4 | SAME-R 1.5 |
| `D_ref` DBP | 400 nm | J | factor 1.6 | 1.2 |
| `sigma_0` | 1.6, 1.6, 1.9, 1.9; DBP 1.5 | J, with L | 0.15 | narrow Kapton and Teflon, broad silicone and lamp wick: the card (L) |
| `t_c` | 400, 500, 1000, 1400 s; DBP 2500 s | T, with L | factor 2.2; DBP 3 | theory, tempered by the card's "number down 1.7 to 2.7 in 800 s" (L); DBP theory only |
| `p` (flow) | 0.25, flat below 2 cm/s | T/J | 0.15 | SAME-R 1.5 |
| DBP vapour pressure, boiling point, density, index | 1e-3 Pa at 296 K, 613 K, 1050 kg/m3, 1.49 | P | factor 3 on the pressure | property tables, from memory |
| DBP chamber smoke | 30 mg/m3 | J | factor 3 | 1.2 |
| Buoyant plume speed on the ground | 20 cm/s | T | factor 1.5 | `sqrt(g beta dT d_s)`, 3 mm sample, 300 K excess |
| Logit values | section 3 | J | 1.0 | SAME-R 1.7 |

## 5. Registered expectations

Medians; `d10` and `D30` in nm. Flows: 8.1 cm/s (the SAME-R standard), 5 and 10 cm/s (the plan of SAME),
2 cm/s (the low end flown). Written before any comparison; no SAME outcome exists in anything I read.

| Material | Flow cm/s | d10 at 40 s | d10 at 750 s | d10 at 1800 s | D30 at 40 s | D30 at 1800 s | P full scale, 40 s | P full scale, 1800 s |
|---|---|---|---|---|---|---|---|---|
| Lamp wick | 10 | 154 | 199 | 239 | 232 | 331 | 0.71 | 0.71 |
| Lamp wick | 8.1 | 166 | 214 | 257 | 250 | 356 | 0.75 | 0.75 |
| Lamp wick | 5 | 187 | 242 | 290 | 282 | 402 | 0.80 | 0.80 |
| Lamp wick | 2 | 235 | 304 | 364 | 355 | 506 | 0.85 | 0.85 |
| Kapton | 10 | 87 | 134 | 173 | 108 | 188 | 0.13 | 0.21 |
| Kapton | 8.1 | 94 | 144 | 186 | 116 | 203 | 0.15 | 0.24 |
| Kapton | 5 | 106 | 163 | 210 | 131 | 229 | 0.28 | 0.40 |
| Kapton | 2 | 133 | 205 | 264 | 165 | 288 | 0.45 | 0.59 |
| Teflon | 10 | 97 | 145 | 182 | 121 | 201 | 0.08 | 0.14 |
| Teflon | 8.1 | 104 | 156 | 196 | 130 | 216 | 0.10 | 0.16 |
| Teflon | 5 | 118 | 176 | 221 | 147 | 244 | 0.19 | 0.30 |
| Teflon | 2 | 148 | 221 | 277 | 184 | 306 | 0.33 | 0.47 |
| Silicone | 10 | 191 | 230 | 276 | 288 | 388 | 0.66 | 0.66 |
| Silicone | 8.1 | 205 | 248 | 297 | 310 | 418 | 0.70 | 0.70 |
| Silicone | 5 | 232 | 280 | 335 | 350 | 471 | 0.76 | 0.76 |
| Silicone | 2 | 291 | 352 | 422 | 440 | 593 | 0.81 | 0.81 |
| DBP | 10 | 315 | 351 | 405 | 371 | 454 | 0.88 | 0.88 |
| DBP | 8.1 | 339 | 378 | 436 | 400 | 489 | 0.90 | 0.90 |
| DBP | 5 | 383 | 426 | 492 | 451 | 552 | 0.92 | 0.92 |
| DBP | 2 | 482 | 536 | 618 | 567 | 694 | 0.94 | 0.94 |

Ratios:

| Statement | Forecast |
|---|---|
| Number after 800 s over number at the start | Kapton 0.33, Teflon 0.38, lamp wick 0.56, silicone 0.64, DBP 0.76 |
| `d10` at 1800 s over `d10` at 40 s | Kapton 2.0, Teflon 1.9, lamp wick 1.55, silicone 1.45, DBP 1.29 |
| Order of `d10` at equal conditions | Kapton < Teflon < lamp wick < silicone < DBP |
| Low gravity over normal gravity, same duct flow | 1.3 at 8 cm/s, 1.4 at 5 cm/s, 1.8 at 2 cm/s |
| Repeated DBP points | within a factor 1.2 of each other (a liquid reference; if not, the source is not repeatable and neither are the polymers) |

## 6. What would show the law is wrong

1. **DBP is not the largest and most stable smoke.** If DBP's `d10` is below lamp wick's, or grows with
   aging as fast as Kapton's, the picture of 1.2 (few nuclei, large inert drops) is wrong, and with it the
   reading of the material constants as "volatility of the condensing vapour".
2. **Aging growth does not order by initial size.** The law says the smallest smokes (Kapton, Teflon) grow
   most with aging, because at equal mass they are the most numerous. Equal growth factors for all
   materials, or the largest growth for the largest particles, would say the growth is not coagulation.
3. **Flow has no effect, or the wrong sign.** Between the lowest and the highest duct flow flown (2 to
   8 cm/s) the law gives a factor 1.4 in `d10`. A factor below 1.1 or above 2, across materials, would
   put the flow exponent outside 0.07 to 0.5.

## 7. Interface for the strict forward (`docs/wsp/research/law_forward.md`)

### 7.1 Declaration for the slug `same`

Six features. The indicators are derived by the engineer from `material` (lamp wick is the reference).

| # | Name | Unit | Transform | Centre | Scale | Domain | Meaning |
|---|---|---|---|---|---|---|---|
| 0 | `is_kapton` | 1 | identity | 0 | 1 | 0 to 1 | material indicator |
| 1 | `is_teflon` | 1 | identity | 0 | 1 | 0 to 1 | |
| 2 | `is_silicone` | 1 | identity | 0 | 1 | 0 to 1 | |
| 3 | `is_dbp` | 1 | identity | 0 | 1 | 0 to 1 | dibutyl phthalate |
| 4 | `inlet_velocity` | m/s | identity | 0.081 | 0.01 | 0 to 0.12 | `z` = cm/s below or above 8.1 |
| 5 | `net_aging_time` | s | ln | 100 | 1 | 5 to 5000 | the horizon |

Targets: `d10` (positive, m, lane-local), `d30` (positive, m, lane-local), `iss_full_scale` (binary).
The velocity is read with the identity transform so that a row at zero flow, if SAME has one, is forecast
(as the 2 cm/s value) and not refused. The network is the one of SAME-R section 7.2 without the
temperature, no-flow and heating-time terms: units 0 and 1 make the flow law, units 2 to 5 the soft-plus
of `ln(t/t_c)`.

### 7.2 Literal assignments

```text
# same.wsp W0: zero-shot law of docs/wsp/research/phys_same_w0.md. Author claude-fable-5-2.
# features: 0 is_kapton, 1 is_teflon, 2 is_silicone, 3 is_dbp, 4 inlet_velocity, 5 net_aging_time.
x.is_kapton.centre = 0.0
x.is_kapton.scale = 1.0
x.is_kapton.lo = 0.0
x.is_kapton.hi = 1.0
x.is_teflon.centre = 0.0
x.is_teflon.scale = 1.0
x.is_teflon.lo = 0.0
x.is_teflon.hi = 1.0
x.is_silicone.centre = 0.0
x.is_silicone.scale = 1.0
x.is_silicone.lo = 0.0
x.is_silicone.hi = 1.0
x.is_dbp.centre = 0.0
x.is_dbp.scale = 1.0
x.is_dbp.lo = 0.0
x.is_dbp.hi = 1.0
x.inlet_velocity.centre = 0.081
x.inlet_velocity.scale = 0.01
x.inlet_velocity.lo = 0.0
x.inlet_velocity.hi = 0.12
x.net_aging_time.centre = 100.0
x.net_aging_time.scale = 1.0
x.net_aging_time.lo = 5.0
x.net_aging_time.hi = 5000.0

# d10, count mean diameter, ln(m). b0: lamp wick, D_ref 250 nm, sigma_0 1.9.
d10.median.b0 = -15.614
d10.median.skip[0:6] = [-0.586, -0.463, 0.215, 0.718, -0.039, 0.0]
# units 0, 1: flow law, steeper below 5 cm/s, flat below 2 cm/s
d10.median.W1[0, 4] = -1.0
d10.median.b1[0] = -3.1
d10.median.w2[0] = 0.0373
d10.median.W1[1, 4] = -1.0
d10.median.b1[1] = -6.1
d10.median.w2[1] = -0.0763
# units 2 to 5: 0.413 ln(1 + t/t_c); t_c 1000 s lamp wick, 400 Kapton, 500 Teflon, 1400 silicone, 2500 DBP
d10.median.W1[2:6, 0:6] = [[0.917, 0.693, -0.336, -0.916, 0.0, 1.0],
                           [0.917, 0.693, -0.336, -0.916, 0.0, 1.0],
                           [0.917, 0.693, -0.336, -0.916, 0.0, 1.0],
                           [0.917, 0.693, -0.336, -0.916, 0.0, 1.0]]
d10.median.b1[2:6] = [0.197, -1.403, -3.203, -4.803]
d10.median.w2[2:6] = [0.0826, 0.124, 0.124, 0.0826]
d10.spread.b0 = 0.42
d10.spread.skip[0:6] = [0.0, 0.0, 0.0, 0.10, 0.0, 0.0]
d10.spread.W1[0, 5] = 1.0
d10.spread.b1[0] = -1.0
d10.spread.w2[0] = 0.03
d10.tail_weight = 0.10
d10.tail_width = 2.5

# d30, diameter of average mass, ln(m)
d30.median.b0 = -15.202
d30.median.skip[0:6] = [-0.777, -0.654, 0.215, 0.470, -0.039, 0.0]
d30.median.W1[0, 4] = -1.0
d30.median.b1[0] = -3.1
d30.median.w2[0] = 0.0373
d30.median.W1[1, 4] = -1.0
d30.median.b1[1] = -6.1
d30.median.w2[1] = -0.0763
d30.median.W1[2:6, 0:6] = [[0.917, 0.693, -0.336, -0.916, 0.0, 1.0],
                           [0.917, 0.693, -0.336, -0.916, 0.0, 1.0],
                           [0.917, 0.693, -0.336, -0.916, 0.0, 1.0],
                           [0.917, 0.693, -0.336, -0.916, 0.0, 1.0]]
d30.median.b1[2:6] = [0.197, -1.403, -3.203, -4.803]
d30.median.w2[2:6] = [0.0667, 0.100, 0.100, 0.0667]
d30.spread.b0 = 0.40
d30.spread.skip[0:6] = [0.0, 0.0, 0.0, 0.10, 0.0, 0.0]
d30.spread.W1[0, 5] = 1.0
d30.spread.b1[0] = -1.0
d30.spread.w2[0] = 0.03
d30.tail_weight = 0.10
d30.tail_width = 2.5

# ISS scatter channel at full scale
iss_full_scale.logit.b0 = 1.10
iss_full_scale.logit.skip[0:6] = [-2.83, -3.30, -0.25, 1.10, -0.10, 0.0]
# unit 0: Kapton and Teflon gain the cube of the size at low flow (1 at 2 cm/s)
iss_full_scale.logit.W1[0, 0] = 2.0
iss_full_scale.logit.W1[0, 1] = 2.0
iss_full_scale.logit.W1[0, 4] = -0.164
iss_full_scale.logit.b1[0] = -2.0
iss_full_scale.logit.w2[0] = 0.9
# unit 1: and with aging beyond 270 s
iss_full_scale.logit.W1[1, 0] = 2.0
iss_full_scale.logit.W1[1, 1] = 2.0
iss_full_scale.logit.W1[1, 5] = 1.0
iss_full_scale.logit.b1[1] = -3.0
iss_full_scale.logit.w2[1] = 0.3
```

Check values: lamp wick, 8.1 cm/s, 40 s: `z` = (0, 0, 0, 0, 0, -0.916), the aging units are off,
`d10` = exp(-15.614) = 166 nm, logit 1.10. DBP, 5 cm/s, 1800 s: `z` = (0, 0, 0, 1, -3.1, 2.890);
`ln(t/t_c)` = -0.329; `d10` = 492 nm; P = 0.92.

## 8. Sources, limits, exposure

**Sources.** Cards PSI-102 and PSI-101; `docs/wsp/atlas/INVESTIGATIONS.md`, section PSI-102;
`docs/wsp/feedback/same_feasibility.md`, sections "For the physicist", 2 and 3; and the sources of
[phys_samer_w0.md](phys_samer_w0.md) section 8.1 (Hinds; Friedlander; Lee 1983; the abstracts of the
SAME papers of 2009, 2015 and 2018, class L). DBP properties from memory of standard tables.

**Could not find or check.** The temperature at which the DBP wick was run and the mass of DBP smoke per
test; a measured size of a DBP condensation aerosol made this way (my 400 nm is a judgement between the
0.3 and 1 um of laboratory condensation generators); whether SAME's duct flows were set values or
measured; the geometry (as SAME-R 8.2).

**Exposure.** Everything listed in SAME-R 8.3 applies: the cards' findings (class L, some of them from
SAME itself: count mean diameter below 300 nm and about 50 % above the ground value; number down by 1.7
to 2.7 and count mean diameter up about 1.5 times in 800 s; narrow Kapton and Teflon, broad silicone and
lamp wick), the abstracts, and the SAME-R train facts in the engineer's note, read after the constants
were typed and not used. The SAME note itself holds no outcome. Of the card's SAME findings I used the
1.7 to 2.7 to temper `t_c` and the narrow or broad grouping; the 50 % and the 300 nm were known to me
when I typed the flow exponent and `D_ref`, and the law's 1.3 to 1.8 and its 94 to 205 nm at 8.1 cm/s
are consistent with them, which is therefore not a test. The DBP row owes nothing to any SAME result.

**The earlier author's kernel.** As in SAME-R 8.4: it evolves a supplied distribution and has no law for
the source, no agglomerates, no scattering or ionization detector and no width. For DBP it would be
within its own support (coalescing spheres); it still needs the initial size it does not provide.

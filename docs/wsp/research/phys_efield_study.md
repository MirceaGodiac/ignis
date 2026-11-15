# E-FIELD Flames (PSI-22): study of the development cases, and a third law for the ion current

Author: `claude-fable-5-2`, 15 November 2026, under PROTOCOL 13.6. **Data-informed, in-sample**:
written from the 3259 scored voltage levels of 152 tests and the 156 flame-out records of the
development days (train and validation), read test by test. No test day was opened. Throwaway
tables: `runs/dev/gen7/study/ef_*.py`. One candidate was run with the logged tool (1 of 5).

## 1. What the data show

### 1.1 Current against voltage, by flame type

Median current (uA) of the levels in each voltage band; in brackets the median of
`ln(observed / W0 median)`. "3 cm/s" is the weak coflow most coflow-burner tests ran with.

| mesh, fuel, burner | 0.4-0.8 kV | 0.8-1.3 | 1.3-1.8 | 1.8-2.6 | 2.6-3.6 | 3.6-5.2 | 5.2-7.2 | 7.2-10 kV |
|---|---|---|---|---|---|---|---|---|
| negative, methane, coflow burner 3 cm/s (670) | 0.10 | 0.22 | 0.39 | 0.62 | 0.96 | 1.53 (-0.36) | 1.89 (-0.50) | 1.65 (+0.46) |
| negative, methane, coflow burner still (457) | 0.10 | 0.18 | 0.29 | 0.50 | 0.81 | 1.20 | 0.87 | 0.82 |
| negative, methane, gas-jet (178) | 0.08 | 0.30 | 0.51 | 0.84 | 1.40 | 2.32 | 2.66 | 2.16 |
| negative, ethylene 30 %, coflow burner 3 cm/s (288) | 0.08 | 0.16 | 0.29 | 0.53 | 0.86 | 1.44 (+1.16) | 2.27 (+2.09) | 3.36 (+2.66) |
| negative, ethylene, gas-jet (155) | 0.15 | 0.28 | 0.50 | 0.72 | 1.23 | 2.14 | 3.55 | 6.09 |
| positive, methane, coflow burner 3 cm/s (463) | 0.16 | 0.50 | 0.56 | 0.54 (-0.52) | 0.50 (-0.91) | 0.50 | 0.49 | 0.51 |
| positive, methane, coflow burner still (201) | 0.07 | 0.16 | 0.17 (-1.2) | 0.13 (-1.8) | 0.09 (-2.7) | 0.07 (-2.9) | | |
| positive, methane, gas-jet (265) | 0.19 | 0.45 | 0.54 | 0.51 | 0.40 (-1.5) | 0.33 (-1.7) | 0.21 | |
| positive, ethylene, coflow burner 3 cm/s (357) | 0.28 | 0.78 | 1.09 | 1.29 | 1.41 | 1.33 (-0.82) | 1.26 (-1.44) | 1.27 (-1.34) |
| positive, ethylene, coflow burner 19-31 cm/s (50) | 0.54 | 2.55 | 3.39 | 3.54 | 4.14 | 6.53 | 7.57 | 7.26 |
| positive, ethylene, gas-jet (151) | 0.20 | 0.74 | 1.21 | 1.68 | 2.35 | 2.59 | 2.62 | |

Four things W0 did not have.

1. **The polarity decides the ceiling.** With the mesh negative the current climbs for several
   kilovolts and its ceiling scales with the carbon flow: methane levels off at 0.8e-6 ion pairs
   per carbon atom (0.78 to 0.86 in the three methane rows), ethylene is at 1.2 to 1.9e-6 and still
   rising at 10 kV. With the mesh positive the current stops rising between 1 and 2.5 kV at a value
   that hardly depends on the fuel flow: 0.5 uA for methane (0.33 to 0.74 uA over a fivefold range
   of carbon flow), 1.3 uA for ethylene (1.33 to 1.67 uA whatever the flow or the dilution). W0
   gave both polarities the same ceiling and the positive mesh a higher rising branch: at 4 to
   10 kV it forecast 3 to 5 uA where 0.5 and 1.3 uA were measured. This alone is 40 % of the W0
   loss.
2. **The rise is not quadratic above 1 kV.** `I / V^2` falls from 0.25 uA/kV2 at 1 kV to 0.13 at
   3 kV and 0.08 at 4.4 kV (mesh negative): between 1 and 5 kV the current goes as `V^1.4`. Below
   1 kV it is quadratic with `G` = 0.25 uA/kV2 (W0: 0.158), and 2.5 times that with the mesh
   positive (W0: 1.5 times).
3. **The yield of methane is half of what I took** (0.8e-6 against 1.5e-6 per carbon), dilution
   costs little (30 % ethylene yields more per carbon than pure ethylene; 40 % methane about 0.6
   of pure), and ethylene yields about three times methane per carbon. W0's `x^1.0` put the
   ceiling of 30 % ethylene at 0.8 uA; it reaches 3.4 uA.
4. **There is no fall after the ceiling** in most flame types (W0's `1/V`); where the current
   falls (still gas, positive mesh; coflow burner in still gas above 5 kV) it is the flame
   weakening, which the law cannot see without the coflow.

Why the plain space-charge baseline beat W0: its rising branch is almost the same as W0's (Thomson
planar over the mesh disc gives 0.18 and 0.24 uA/kV2), its cap is `5e-7` per carbon, one third of
W0's, so it sat lower at high voltage where the positive-mesh flames sit, and it has no `x^1.0`
penalty and no fall. The level of `G` was not the difference; the cap and the fall were.

### 1.2 The physics of the positive ceiling

With the mesh positive the positive ions are driven to the burner and their wind blows against the
fuel jet. The flame is pressed on to the tube (the report: compressed, then oscillating) and burns
as a small flat flame whose reaction zone no longer grows with the fuel supplied; fuel passes it
unburnt or burns downstream, cold of the ion-making zone. The ceiling is then a property of the
pressed flame, not of the carbon flow. A real coflow (19 to 31 cm/s) lifts the ceiling back to the
carbon-limited value (7.5 uA at 3.7e19 atoms/s, 1.25e-6 per carbon): the air stream wins against
the wind. With no air at all (coflow burner in still gas) the pressed flame starves: 0.17 uA and
falling.

The exponent 1.4: as the voltage rises the flame becomes compact and withdraws from the mesh, so
the gap of the space-charge law grows with the voltage; the card says the peak coincides with the
most compact flame. I cannot derive 1.4; it is read from the table.

### 1.3 Flame-out under the field

| class | tests | flame out | when |
|---|---:|---:|---|
| coflow burner with coflow (3 to 31 cm/s), any fuel, either polarity | 65 | 0 | |
| ethylene, any burner | 50 | 0 | |
| coflow burner in still gas, methane, mesh positive | 26 | 13 | 0 to 60 s, at 0.2 to 5.4 kV |
| coflow burner in still gas, methane, mesh negative | 11 | 1 | 14 s |
| gas-jet, methane, mesh negative | 13 | 6 | at 6.8 to 7.9 kV; all 6 programmes that went past 6.4 kV, none of 7 that stopped at 5.5 kV or below |
| gas-jet, methane, mesh positive | 22 | 9 | at 3.8 to 8.0 kV; 6 of 9 programmes past 6 kV, 3 of 13 below |

W0's blow-off voltage (20 kV times `x^1.5`, only below saturation) is wrong on each count: the
gas-jet methane flames go out near 7 kV whatever their dilution (40, 70, 100 %), the coflow-burner
flames with 3 cm/s of air stand 10 kV at 40 % methane, and half of the still-gas flames with a
positive mesh die at any voltage. The events are decided by burner, coflow and polarity, none of
which the present declaration carries except polarity.

## 2. Candidates

### E1 (run; proposed): the ion current, on the existing declaration

```text
z_c = ln(N_C / 1e19),  z_x = ln(x_fuel),  z_v = ln(V / 3 kV),  e ethylene,  p mesh positive
rising branch   ln a = ln(1.164 uA) + 1.4 z_v + 0.9 p - 0.6 relu(ln(1 kV / V))      (0.25 uA/kV2 below 1 kV; 2.46 times, mesh positive)
ceiling, mesh negative   ln A = ln(1.28 uA) + z_c + 0.4 z_x + 1.10 e                 (0.8e-6 per carbon, x^0.4, ethylene 3 times)
ceiling, mesh positive   ln B = ln(0.5 uA) + 0.25 z_c + 0.95 e                       (0.5 uA methane, 1.3 uA ethylene)
ln I = ln a - relu(ln a - ln A - 20 p) - relu(ln a - ln B - 20 (1 - p));   ln-sd 0.45
```

The two ceilings are switched by the polarity indicator inside the hinge, so no hinge sits inside
another. 31 elements changed; `flame_end`, `peak_ion_current`, `peak_voltage` untouched.

Expectation as logged with the run (my strata), and what the tool reports (its strata):

| | W0 | Expected before the run | Realised |
|---|---:|---|---:|
| pooled CRPS, uA | 0.659 | 0.35 to 0.45, below the space-charge baseline if at the low end | **0.253** (space-charge baseline 0.426, constant 0.562) |
| train / validation | 0.720 / 0.546 | lower | 0.268 / 0.225 |
| days lower | | most | **15 of 15** |
| flame-out (T2) | 0.148 | unchanged | 0.148 |
| mesh positive, ethylene, coflow burner 3 cm/s | 1.32 | 0.25 | |
| mesh negative, ethylene 30 %, coflow burner | 1.13 | 0.45 | ethylene 30 % (both polarities): 0.875 to 0.285 |
| mesh negative, methane, coflow burner 3 cm/s / still | 0.41 / 0.48 | 0.30 / 0.50 (no better in still gas) | coflow burner, mesh negative: 0.578 to 0.220 |
| mesh positive, methane, coflow burner 3 cm/s / still | 0.48 / 0.37 | 0.12 / 0.30 | coflow burner, mesh positive: 0.782 to 0.279 |
| gas-jet, mesh negative (ethylene / methane) | 0.92 / 0.41 | 0.6 / 0.35 | 0.647 to 0.277 |
| gas-jet, mesh positive (ethylene / methane) | 0.65 / 0.61 | 0.6 / 0.20 | 0.626 to 0.281 |
| mesh positive, ethylene, real coflow (50 levels) | 1.43 | **worse**, about 4 | not reported separately; its day, 18137, goes 1.905 to 0.796 and holds the largest remaining error |

The tool does not print my strata, so the stratum expectations can be checked only through the
strata it does print; none of those moved against the expectation.

The pooled gain is larger than I wrote. What is left, by day: 18137
(0.80: the two tests with a real coflow, forecast 1.7 uA, measured 4 to 7.6), 18319 (0.56:
gas-jet ethylene with the mesh positive, measured twice the coflow-burner ceiling), 18106 (the
still-gas flames with the mesh positive, forecast 0.5 uA, measured 0.07 to 0.17). All three need
the burner and the coflow.

## 3. Proposal

**E1 as the model on slug `efield` for the ion current** (a revision of W0; numbers only). By the
keep rule: lower pooled, lower on 15 of 15 days, the other target unchanged.

### For the engineer

Nothing new for E1. For the next slug (section 4) three indicators are needed, all from fields
already in the cases:

| # | `name` | unit | transform | domain | Derived from |
|---|---|---|---|---|---|
| 7 | `is_gas_jet` | 1 | identity (centre 0, scale 1) | 0 to 1 | `burner` = `gas_jet_2.1mm` |
| 8 | `still_coflow_burner` | 1 | identity | 0 to 1 | `burner` = `coflow` and `u_coflow_m_s` = 0 |
| 9 | `strong_coflow` | 1 | identity | 0 to 1 | `u_coflow_m_s` >= 0.1 |

and `sweep_v_max` must be the largest voltage the programme *plans*, not the largest level reached
before the flame went out (please confirm which it is; the law of section 4 assumes the planned
one).

### Edits (changed lines; parent W0, `configs/models/law/efield/w0-fable`)

```text
ion_current.median.b0 = -13.664
ion_current.median.skip[0:7] = [0.0, 0.0, 0.0, 0.9, 1.4, 0.0, 0.0]
ion_current.median.W1[0, 0:7] = [-1.0, -0.4, -1.10, -20.0, 1.4, 0.0, 0.0]
ion_current.median.b1[0] = -0.095
ion_current.median.w2[0] = -1.0
ion_current.median.W1[1, 0:7] = [-0.25, 0.0, -0.95, 20.0, 1.4, 0.0, 0.0]
ion_current.median.b1[1] = -18.255
ion_current.median.w2[1] = -1.0
ion_current.median.W1[2, 4] = -1.0
ion_current.median.b1[2] = -1.0986
ion_current.median.w2[2] = -0.6
ion_current.spread.b0 = 0.45
ion_current.spread.w2[0] = 0.0
```

(The file run is `runs/dev/gen7/study/efield_e1.wsp`.)

## 4. What remains structural: burner and coflow (`efield3`)

A new slug: the declaration of `efield` with the three indicators of section 3 appended (indices
7, 8, 9), and `flame_end` with causes `blowoff`, `press`, `other`. Typed below and **not run** (the
tool cannot run a declaration that does not exist); it will count as a candidate when it is.

Ion current, on top of E1:

- gas-jet: the rising branch 1.4 times higher (0.30 against 0.22 uA at 1 kV, 2.3 against 1.5 uA at
  4.4 kV, mesh negative; the tip of the gas-jet tube stands closer to the mesh);
- mesh positive, gas-jet: ceiling 0.45 uA for methane, 2.6 uA for ethylene (no dependence on flow);
- mesh positive, coflow burner in still gas: ceiling 0.15 uA (0.3 of the weak-coflow value);
- mesh positive with a real coflow: the carbon-limited ceiling at 0.52 of the negative-mesh one.

Flame-out:

- `blowoff` (gas-jet, methane, mesh negative): the flame goes out when the voltage passes 7.3 kV;
  clock `tau = window * 7.3 kV / V_max`, shape 12 (0.03 for a programme to 5.5 kV, 0.98 to 10 kV);
- `press` (gas-jet, methane, mesh positive): 7.0 kV, shape 4 (0.2 at 5 kV, 0.5 at 7 kV, 0.8 at
  10 kV): the pressed flame fails over a wide range of voltage;
- `other`: a flame on the coflow burner in still gas starves: median 45 s with the mesh positive
  (13 of 26 out, half of them within 35 s), 600 s with the mesh negative (1 of 11); every other
  flame 3000 s (none of 115 went out).

Expected when run: ion current from 0.253 to about 0.19 (days 18137, 18319, 18106 lower; the
others within 0.01); flame-out from 0.148 to 0.06-0.07, level with the stratified incidence
(0.070), lower on the days with still-gas or gas-jet methane flames (18106, 18113, 18130, 18282,
18298, 18312) and slightly higher on none.

```text
# efield3 = E1 with these lines added or replaced. Features z = (z_c, z_x, e, p, z_v, z_m, z_w, gas, scb, strong).
x.is_gas_jet.centre = 0.0
x.is_gas_jet.scale = 1.0
x.is_gas_jet.lo = 0.0
x.is_gas_jet.hi = 1.0
x.still_coflow_burner.centre = 0.0
x.still_coflow_burner.scale = 1.0
x.still_coflow_burner.lo = 0.0
x.still_coflow_burner.hi = 1.0
x.strong_coflow.centre = 0.0
x.strong_coflow.scale = 1.0
x.strong_coflow.lo = 0.0
x.strong_coflow.hi = 1.0
# ion current: gas-jet rising branch 1.4 times (ln 1.4 = 0.35).
ion_current.median.skip[0:10] = [0.0, 0.0, 0.0, 0.9, 1.4, 0.0, 0.0, 0.35, 0.0, 0.0]
ion_current.median.W1[0, 0:10] = [-1.0, -0.4, -1.10, -20.0, 1.4, 0.0, 0.0, 0.35, 0.0, 0.0]
# positive ceiling of the coflow burner: off for the gas-jet and for a real coflow; 0.3 times in still gas.
ion_current.median.W1[1, 0:10] = [-0.25, 0.0, -0.95, 20.0, 1.4, 0.0, 0.0, -20.0, 1.2, -20.0]
# unit 3: positive ceiling of the gas-jet, 0.45 uA methane, 2.6 uA ethylene.
ion_current.median.W1[3, 0:10] = [0.0, 0.0, -1.754, 20.0, 1.4, 0.0, 0.0, 20.0, 0.0, 0.0]
ion_current.median.b1[3] = -37.800
ion_current.median.w2[3] = -1.0
# unit 4: positive mesh with a real coflow: the negative ceiling times 0.52.
ion_current.median.W1[4, 0:10] = [-1.0, -0.4, -1.10, 20.0, 1.4, 0.0, 0.0, 0.0, 0.0, 20.0]
ion_current.median.b1[4] = -38.545
ion_current.median.w2[4] = -1.0
# flame out. blowoff: gas-jet methane, mesh negative, 7.3 kV. b0 = ln(100 s) + ln(60 kV / 5 kV).
flame_end.blowoff.median.b0 = 7.090
flame_end.blowoff.median.skip[0:10] = [0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 1.0, 0.0, 0.0, 0.0]
flame_end.blowoff.median.W1[0, 0:10] = [0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
flame_end.blowoff.median.b1[0] = 0.0
flame_end.blowoff.median.w2[0] = -2.106
flame_end.blowoff.shape = 12.0
# press: gas-jet methane, mesh positive, 7.0 kV.
flame_end.press.median.b0 = 7.090
flame_end.press.median.skip[0:10] = [0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 1.0, 0.0, 0.0, 0.0]
flame_end.press.median.W1[0, 0:10] = [0.0, 0.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
flame_end.press.median.b1[0] = -1.0
flame_end.press.median.w2[0] = -2.148
flame_end.press.shape = 4.0
# other: starving in still gas on the coflow burner; 45 s mesh positive, 600 s negative, else 3000 s.
flame_end.other.median.b0 = 8.006
flame_end.other.median.skip[0:10] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
flame_end.other.median.W1[0, 0:10] = [0.0, 0.0, -1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]
flame_end.other.median.b1[0] = -1.0
flame_end.other.median.w2[0] = -4.2
flame_end.other.median.W1[1, 0:10] = [0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]
flame_end.other.median.b1[1] = 0.0
flame_end.other.median.w2[1] = -1.6
flame_end.other.shape = 1.5
```

Also structural, and not typed:

- the fall of the current when the flame weakens (still gas above 5 kV with the mesh negative;
  the gas-jet with the mesh positive above 3 kV): it needs the flame state, which is the other
  target;
- the strong-coflow rising branch (2.5 uA at 1 kV against 0.6): two tests, no number typed;
- why the rise goes as `V^1.4`: the gap grows as the flame withdraws; a flame-height input (from
  images) would say.

## 5. What the W0 physics got right and wrong

Right: space charge limits the rise and the carbon flow limits the ceiling with the mesh
negative; the ion wind decides what happens to the flame. Wrong: the yield (the detector's value
is an upper bound that these flames do not reach), the dilution penalty, the symmetry between the
polarities, and the idea that a saturated flame is safe: with the mesh positive the wind presses
the flame on the burner at any voltage above a kilovolt, and without air around it the flame dies.

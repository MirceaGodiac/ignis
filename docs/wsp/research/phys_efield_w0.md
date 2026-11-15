# ACME E-FIELD Flames (PSI-22): the ion current a flame gives to a field. A W0 law

Author: `claude-fable-5-2`, 15 November 2026. Kind: **W0, literal authoring**. No file under `runs/`
(other than the brief), `data/` or `holdout/` was opened. I found no paper with numbers from this
experiment; what I know of its outcome is the wording of the card and of two public NASA summaries,
and two facts in the engineer's feasibility note (section 8.3). Nothing was fitted. The only program
run was a forward evaluation of the typed law to fill section 5.

## 0. The result in one screen

A hydrocarbon flame makes about one or two ion pairs per million carbon atoms burnt. A field between
the burner and a mesh above the flame pulls them apart and a current flows. Two things limit it:
how fast the ions can cross the gap against their own space charge (low voltage) and how many the
flame makes (high voltage).

```text
saturation current   I_sat = eps * e * N_C * x_fuel^1.0        eps = 1.5e-6 ion pairs per fuel carbon atom
rising branch        I(V)  = G * V^2,  G = 2 eps0 K / h = 0.158 uA/kV^2   (mesh negative; 1.5 times that, mesh positive)
current at a level   I     = min(I_sat, G V^2), falling as 1/V beyond 1.16 V_sat,   V_sat = sqrt(I_sat / G)
flame put out by the field:  only if the voltage it cannot stand, V_ext = 20 kV * x_fuel^1.5 (methane),
                     lies below both V_sat and the highest voltage of the programme
```

`N_C` is the flow of fuel carbon atoms per second, `e` the elementary charge, `x_fuel` the fuel mole
fraction of the burner gas. For 1e19 carbon atoms per second (22 standard cm3/min of methane) of
undiluted fuel: 2.4 uA, reached near 3.9 kV with the mesh negative. Widths (one standard deviation
in ln): 0.65 on the current of the rising branch; 0.55 on the saturation current of a pure fuel,
rising to 1.15 at 30 % fuel; 0.45 on the saturation voltage. They are wide because the two
constants that carry the law, the ion yield and the mobility times the geometry, are each known to
a factor of about 1.7.

## 1. Mechanism

### 1.1 Where the ions come from

In the reaction zone `CH + O -> CHO+ + e-`, followed at once by proton transfer to water
(`H3O+`). Thermal ionisation is negligible below 2500 K. The source is tied to `CH`, hence to fuel
carbon passing through a hot reaction zone, and not to the amount of oxidiser or of product. This is
the flame ionisation detector: its response is proportional to the carbon mass flow, about 0.015
coulomb per gram of carbon whatever the hydrocarbon (alkanes 1.0 per carbon, olefins 0.95).
Dimensionless form, the **ion yield**:

```text
eps = I_sat / (e * N_C)        ion pairs collected per fuel carbon atom
```

The detector value is 1.9e-6; premixed hydrocarbon flames give "about one per million" (Calcote;
Lawton and Weinberg); small methane diffusion flames in the laboratory of the investigators
saturate near 0.1 uA per cm3/min of methane, which is 1.5e-6 (my memory of their papers, not
re-read). I take **1.5e-6**: 0.145 C per mole of carbon, 0.24 uA per 1e18 carbon atoms per second.

**Saturation** means every ion pair is pulled apart before it recombines. Recombination is fast
(`alpha` about 2e-13 m3/s; with 1e16 ions/m3 a lifetime of 0.5 ms) but the field wins once the
drift time across the reaction zone, `delta / (K E)` with `delta` 1 mm, is shorter: `E` above about
1e4 V/m, a few hundred volts over this gap. So recombination is not what delays saturation; space
charge is (1.2).

**Dilution.** A detector flame is hot hydrogen; a diluted hydrocarbon flame is cooler and makes less
`CH` per carbon. With an activation temperature of 25,000 K for the `CH` pool, the stoichiometric
adiabatic temperatures 2226, 2080 and 1900 K of 100, 50 and 30 % methane in nitrogen give factors
1, 0.45 and 0.15, a power of the fuel fraction between 1.2 and 1.6. Against that, the detector
shows no loss when the sample is carried in nitrogen. I take the yield proportional to
`x_fuel^1.0`, with an uncertainty of 0.5 on the exponent.

### 1.2 The rising branch: space charge

With the mesh negative, positive ions leave the flame and cross the cold gap to the mesh; electrons
go to the burner a few millimetres away. Ions of one sign in transit shield the flame from the
field. For a small source facing a plane at distance `h` the largest current the gap can carry is
(Sigmond's unipolar limit; the planar Mott-Gurney law `9 eps0 K V^2 A / (8 h^3)` over the disc of
radius `0.75 h` that the ions reach gives the same number)

```text
I_sc = 2 eps0 K V^2 / h          Pi = I h / (2 eps0 K V^2) <= 1
```

`K` is the ion mobility. The current follows `I_sc` until it meets the source: quadratic in the
voltage, independent of fuel and flow, inversely proportional to the gap. With `K` = 2.5e-4 m2/V/s
and `h` = 28 mm (mesh at 35 mm, flame centre at 7 mm), `G` = 1.58e-13 A/V2. A threshold exists in
principle (the field must beat the gas speed, `E > u / K`, about 30 V here, and the sheath
potential of the flame, about 1 V); it is below 0.1 kV and I leave it out. With the mesh positive
the carriers in the gap are electrons that attach to oxygen within millimetres in cold air but stay
free in the hot plume above the flame, so the effective mobility is higher: I take 1.5 times,
uncertain between 1 and 4.

### 1.3 The peak, and why microgravity has no plateau

The ions drag the gas: a body force `j / K` per unit volume. At 2 uA through a plume 2 cm across
that is 27 N/m3, three times the buoyancy of flame gas on Earth (`rho g (1 - T0/T)` = 9 N/m3), and
the thrust `I h / K` = 2e-4 N is ten thousand times the momentum flux of the fuel jet. The wind
speed scale is `u_w = E sqrt(eps0 / rho)`: 0.4 to 0.8 m/s at 4 kV. This is why NASA says the field
mimics partial gravity. Dimensionless: `N_E = j / (K rho g_0)`, the electric body force in units of
Earth buoyancy, equal to 1 near 0.7 uA.

Without a field a microgravity jet flame in still gas is large, cool and radiating. As the voltage
rises the ion wind feeds it air, the flame shrinks and brightens, the current climbs along the
space-charge law. On Earth buoyancy has already done this and stays in charge, so the current
levels at saturation. In microgravity the wind is the only flow: the card records a distinct peak
that coincides with the most compact flame, then a sharp drop. The law takes the peak as the
saturation current of 1.1 and lets the median current fall as `1/V` beyond 1.16 times the peak
voltage, with a width that grows quickly there. I cannot derive the drop; it is a hedge.

### 1.4 When the field puts the flame out

On the rising branch the wind grows with the voltage, `u_w = (V / h) sqrt(eps0 / rho)`: 0.49 m/s
at 5 kV, a strain `u_w / l` of about 100 1/s on a flame base of `l` = 5 mm. Once the current
saturates the body force `j / K` cannot grow any more, and neither can the wind. So the largest
strain a flame ever feels is the one at its own saturation voltage, and the field blows a flame
off only if the voltage it cannot stand lies below both its saturation voltage and the highest
voltage applied:

```text
V_ext = h l a_ext / sqrt(eps0 / rho),      event if  V_ext < min(V_sat, V_max)
Da_E  = a_ext l h / (V sqrt(eps0 / rho)):  flame time over wind time; blow-off at Da_E = 1
```

`a_ext` is the extinction strain rate of the diffusion flame: about 400 1/s for methane in air and
1400 1/s for ethylene, falling with nitrogen dilution roughly as `x_fuel^1.5` (the Arrhenius
estimate with the temperatures of 1.1 gives 160 1/s at 40 % and 93 1/s at 30 % methane). That is
`V_ext` = 20 kV for pure methane, 5.1 kV at 40 %, 3.3 kV at 30 %; 3.5 times higher for ethylene.
A small or pure flame saturates first and survives any voltage; a large dilute flame is blown off
on the way up. With the mesh positive the wind blows towards the burner and presses the flame on
it (NASA reports oscillation, then extinction): I lower `V_ext` by a factor 0.6.

A weak field also helps: in still gas a dilute flame starves, and NASA reports that very weak
fields stabilise highly diluted flames. The law has no term for a flame that dies for lack of a
field; such ends are in the second clock of section 3.

### 1.5 What each condition does

| Condition | Enters | Direction |
|---|---|---|
| Carbon flow | the source | saturation current proportional; saturation voltage as its square root; a larger flame is easier to blow off |
| Fuel | carbon number (already in the carbon flow); `a_ext` | ethylene twice the current of methane at equal volume flow, and 3.5 times harder to blow off |
| Dilution | yield per carbon; `a_ext` | saturation current proportional to `x_fuel` at fixed carbon flow (between `x^0.5` and `x^1.5`); blow-off voltage as `x^1.5` |
| Voltage | `V^2` on the rising branch | fuel plays no part there |
| Polarity | carrier mobility in the gap; direction of the wind | mesh positive: more current at a given voltage, saturation earlier, blow-off easier |
| Burner, coflow | flame position and temperature | not in the law; in its width |
| Gap | `1/h` | a mesh twice as far halves the rising branch |

### 1.6 The earlier kernel (`property_jets_v2.rs`), as a colleague's work

Its input carries an electric field vector that no equation reads; the kernel it wraps refuses any
nonzero field ("electric response requires charge, force and energy closure"). The companion note
asks for electrode geometry, mobilities, a charge-conserving mechanism and a Poisson closure before
any current is forecast. That is the right list for a field calculation and it is why no forecast
came out. The integral quantities that were measured need less: the saturation current is the
volume integral of the ion source (a yield times a carbon flow, no field solution); the rising
branch is the capacity of the gap for charge of one sign (geometry and one mobility, no
chemistry); the blow-off is a comparison of a wind speed with a flame's extinction strain.

## 2. Targets

Taken from the engineer's note `docs/wsp/feedback/efield_feasibility.md` (its paragraph for the
physicist and its sections 1 and 2; it appeared after sections 0 to 1.3 of this document were first
written). Two of my own are kept as secondary.

| Target | Kind, unit | A forecast may know |
|---|---|---|
| T1 `ion_current`: the flame current at each planned voltage level of the programme, positive in the direction the field drives it, given that the flame still burns | positive, A (scored in uA) | fuel, fuel fraction, carbon flow, burner, polarity, mesh position, the commanded voltage of the level |
| T2 `flame_end`: the flame goes out under the field; probability by 2 to 150 s after the field is switched on, and before the programme ends | event, s since field-on | the same and the programme (highest voltage, window) |
| `peak_ion_current` (secondary): largest current of the sweep | positive, A | as T1 without the level |
| `peak_voltage` (secondary): voltage at which it is reached | positive, V | as T1 without the level |

Inputs named in the note and used: `carbon_flow_atoms_s` (1/s), `x_fuel`, `fuel`, `polarity`,
`levels[].v_cmd` (V), and from `sweep` the largest commanded voltage and `window_s`. Not used:
`u_coflow_m_s`, `burner`, the optional `pre_field` block. The mesh is 35 mm above the burner (37 mm
in the final report, says the note), which is the value I had assumed from memory.

## 3. The law

```text
z_c = ln(N_C / 1e19 1/s),  z_x = ln(x_fuel),  z_v = ln(|V| / 3 kV),  z_m = ln(V_max / 5 kV),  z_w = ln(window / 100 s)
e = 1 for ethylene,  p = 1 for mesh positive

ln I_sat = ln(2.40 uA) + z_c + 1.0 z_x
ln I_sc  = ln(1.42 uA) + 2 z_v + 0.405 p
a        = ln I_sat - ln I_sc = 0.524 + z_c + z_x - 2 z_v - 0.405 p
T1:  ln I(V)  = ln I_sat - relu(a) - 0.5 relu(-a - 0.3)
     ln V_sat = ln(3.90 kV) + 0.5 z_c + 0.5 z_x - 0.203 p
T2:  ln V_ext = ln(20 kV) + 1.5 z_x + 1.253 e - 0.511 p
     blow-off clock:  ln tau_b = ln(window) + ln V_ext - min(ln V_sat, ln V_max),  shape 3
     other ends:      ln tau_o = ln(600 s) + 0.7 z_x,                              shape 1.5
     P(out by t) = 1 - 1 / ((1 + (t / tau_b)^3) (1 + (t / tau_o)^1.5))
```

The blow-off clock is a log-logistic whose median is the window stretched by the ratio of the
voltage the flame cannot stand to the highest voltage it will feel; its shape 3 stands for an
uncertainty of 0.55 in ln on that ratio. The second clock covers flames that end without the wind
(a dilute flame starving in still gas, a fuel flow that never arrived): 6 % of pure-fuel tests and
20 % of 30 % tests in 100 s.

| Target | ln-sd | Why |
|---|---|---|
| `ion_current` | 0.65 on the rising branch, + 0.35 per e-fold of `I_sc / I_sat` beyond 1.35 | mobility 0.5, gap 0.3, geometry 0.3; past the peak the flame may be lifted or weak |
| `peak_ion_current` | 0.55 - 0.5 z_x (+0.10 mesh positive) | yield 0.5; dilution exponent 0.5 per e-fold of dilution; a positive mesh presses the flame on the burner |
| `peak_voltage` | 0.45 - 0.25 z_x | half the combined width of source and gap |
| the three | 10 % of cases three times wider | a flame that oscillates, touches the mesh, or a corona |

Limits. `V -> 0`: no current. `N_C -> 0`: no current at any voltage, and no blow-off (no wind).
`h` large: the rising branch vanishes and saturation needs a voltage growing as `sqrt(h)`.
`x_fuel -> 0`: the yield vanishes and any field blows the flame off. Not covered: above about 8 kV
the gap itself conducts (corona, then arc; NASA reports both), which adds a current that has
nothing to do with the flame; the recording channel clips at 9.3 uA. Range: 1e18 to 1e20 carbon
atoms per second, fuel fraction 0.2 to 1, 0.1 to 10 kV, about 1 atm, mesh 35 mm above the burner.

## 4. Constants

Classes: **P** property, **T** theory, **L** published experiment, **J** judgement.

| Constant | Value | Uncertainty | Class | Origin |
|---|---|---|---|---|
| Elementary charge | 1.602e-19 C | | P | |
| Ion yield per fuel carbon, `eps` | 1.5e-6 | factor 1.7 | L, J | detector 0.015 C per g C (1.9e-6); premixed flames about 1e-6; small methane diffusion flames about 0.1 uA per cm3/min (from memory) |
| Dilution exponent of the yield | 1.0 | 0.5 | T, J | 1.1: between 0 (detector) and 1.6 (`CH` Arrhenius estimate) |
| Permittivity of free space | 8.854e-12 F/m | | P | |
| Ion mobility in the gap, mesh negative | 2.5e-4 m2/V/s | 1.5e-4 to 6e-4 | P, J | `H3O+` clusters in air, 2 to 3 cm2/V/s at 300 K, rising with temperature; flame models use 1e-4 (Fialkov) |
| Polarity factor on `G`, mesh positive | 1.5 | 1 to 4 | J | 1.2 |
| Gap from flame to mesh, `h` | 28 mm | 20 % | L, J | mesh 35 mm above the burner (engineer's note; 37 mm in the final report) less 7 mm of flame |
| Geometry factor | 2 | 1.3 to 3 | T | Sigmond's unipolar limit; planar law over the disc reached |
| `G = 2 eps0 K / h` | 1.58e-13 A/V2 | factor 1.9 | T | |
| Post-peak slope | -0.5 per e-fold of `I_sc / I_sat` | 0 to -2 | J | 1.3 |
| Threshold voltage | below 100 V, left out | 0 to 300 V | T | 1.2 |
| Extinction strain rate, methane in air | 400 1/s | 30 % | L | counterflow diffusion flames (from memory) |
| Extinction strain rate, ethylene in air | 1400 1/s | 30 % | L | same; ratio 3.5 |
| Dilution exponent of `a_ext` | 1.5 | 1.2 to 2 | T | 1.4 |
| Flame base length `l` | 5 mm | 3 to 10 mm | J | the standoff of a small jet flame |
| Air density in `u_w` | 1.18 kg/m3 | | P | the wind is mostly cold air |
| `V_ext`, pure methane, mesh negative | 20 kV | 8 to 40 kV | T | `h l a_ext / sqrt(eps0 / rho)` with `h` = 28 mm: 20.4 kV |
| Polarity factor on `V_ext`, mesh positive | 0.6 | 0.3 to 1 | J | 1.4 |
| Shape of the blow-off clock | 3 | 2 to 5 | J | ln-sd 0.55 on `V_ext / V_felt` |
| Other ends: median, shape, dilution exponent | 600 s, 1.5, 0.7 | factor 2 on the median | J | a few per cent of established flames end for other reasons within a programme; more when dilute |

## 5. Registered expectations

Written before any comparison with project data. The carbon flow is in units of 1e19 atoms per
second: 1 unit is 22.3 standard cm3/min of methane or 11.2 of ethylene, a methane velocity of
0.10 m/s in the fuel tube (3.66 mm2) if the script velocity is at 0 C.

Saturation current, and the voltage at which it is reached with the mesh negative / positive:

| Fuel fraction | 0.5e19 | 1e19 | 2e19 | 4e19 |
|---:|---|---|---|---|
| 1.0 | 1.2 uA at 2.8 / 2.3 kV | 2.4 uA at 3.9 / 3.2 kV | 4.8 uA at 5.5 / 4.5 kV | 9.6 uA at 7.8 / 6.4 kV |
| 0.7 | 0.84 uA at 2.3 / 1.9 kV | 1.7 uA at 3.3 / 2.7 kV | 3.4 uA at 4.6 / 3.8 kV | 6.7 uA at 6.5 / 5.3 kV |
| 0.5 | 0.60 uA at 1.9 / 1.6 kV | 1.2 uA at 2.8 / 2.3 kV | 2.4 uA at 3.9 / 3.2 kV | 4.8 uA at 5.5 / 4.5 kV |
| 0.3 | 0.36 uA at 1.5 / 1.2 kV | 0.72 uA at 2.1 / 1.7 kV | 1.4 uA at 3.0 / 2.5 kV | 2.9 uA at 4.3 / 3.5 kV |

T1, median current in uA at a level (mesh negative / positive), pure fuel at 2e19 atoms/s:

| 0.5 kV | 1 kV | 2 kV | 3 kV | 4 kV | 5 kV | 6 kV | 8 kV | 10 kV |
|---|---|---|---|---|---|---|---|---|
| 0.04 / 0.06 | 0.16 / 0.24 | 0.63 / 0.95 | 1.4 / 2.1 | 2.5 / 3.8 | 4.0 / 4.8 | 4.8 / 4.2 | 3.9 / 3.1 | 3.1 / 2.5 |

T1, 40 % fuel at 1e19 atoms/s, mesh negative (ln-sd in brackets):

| 0.5 kV | 1 kV | 2 kV | 3 kV | 4 kV | 5 kV | 6 kV | 8 kV | 10 kV |
|---|---|---|---|---|---|---|---|---|
| 0.04 (0.65) | 0.16 (0.65) | 0.63 (0.65) | 0.92 (0.68) | 0.69 (0.88) | 0.55 (1.04) | 0.46 (1.17) | 0.34 (1.37) | 0.28 (1.53) |

T2, probability that the flame is out before the end of a 100 s programme (mesh negative /
positive), for a small flame (1e19 atoms/s, any highest voltage from 3 kV up) and for a large one
(4e19 atoms/s) at highest voltages of 3 and 10 kV:

| Fuel | Fuel fraction | 1e19 | 4e19, to 3 kV | 4e19, to 10 kV |
|---|---:|---|---|---|
| Methane | 1.0 | 0.07 / 0.08 | 0.07 / 0.08 | 0.12 / 0.19 |
| Methane | 0.7 | 0.11 / 0.14 | 0.11 / 0.16 | 0.22 / 0.37 |
| Methane | 0.5 | 0.17 / 0.24 | 0.19 / 0.35 | 0.41 / 0.60 |
| Methane | 0.4 | 0.24 / 0.34 | 0.30 / 0.57 | 0.56 / 0.75 |
| Methane | 0.3 | 0.37 / 0.52 | 0.54 / 0.82 | 0.75 / 0.88 |
| Ethylene | 1.0 | 0.06 / 0.06 | 0.06 / 0.06 | 0.06 / 0.07 |
| Ethylene | 0.5 | 0.12 / 0.13 | 0.13 / 0.13 | 0.13 / 0.15 |
| Ethylene | 0.3 | 0.20 / 0.21 | 0.21 / 0.26 | 0.23 / 0.29 |

Against time, methane at 30 %, 4e19 atoms/s, to 10 kV in 100 s, mesh negative: 0.00 at 5 s, 0.01
at 10 s, 0.04 at 20 s, 0.09 at 30 s, 0.22 at 45 s, 0.39 at 60 s, 0.68 at 90 s. Pure methane at
1e19: 0.002 at 10 s, 0.011 at 30 s, 0.032 at 60 s, 0.060 at 90 s.

In words:

1. On the rising branch every flame of either fuel, at any flow and dilution, carries the same
   current at the same voltage and polarity: 0.16 uA at 1 kV, 0.63 uA at 2 kV, 1.4 uA at 3 kV
   (mesh negative).
2. Doubling the carbon flow doubles the saturation current and raises its voltage by 41 %.
3. Halving the fuel fraction at fixed carbon flow halves the saturation current.
4. A positive mesh reaches the same saturation at a voltage 18 % lower.
5. Ethylene flames are almost never put out by the field; pure methane flames rarely; methane at
   30 to 40 % in about half the programmes when the flame is large, a quarter to a third when it
   is small, and more often with the mesh positive.

## 6. What would show the law is wrong

1. **The largest current does not scale with the carbon flow.** If it is the same at 1e19 and
   4e19 atoms/s, the peak is not a saturation of the source: it is set by the wind destroying the
   flame, and the right variable is then a field strength. A peak that scales with carbon but sits
   below one third of the table (below 0.08 uA per 1e18 atoms/s) would mean the microgravity flame
   never gets hot enough to reach the detector yield: `eps` moves, the form stays.
2. **The rising branch is not one curve for all flames, or is not quadratic.** If below the peak
   the current at a given voltage grows with the fuel flow, the flame is not space-charge limited
   (recombination or flame size matters) and `G` must depend on the flame. A linear rise, or no
   current until more than 1 kV, would mean a different limit (a resistive plume, or a sheath
   that needs a threshold field).
3. **Flames go out where the law says the wind has stopped growing.** The law says a flame that
   has saturated is safe at any higher voltage, so that small pure flames survive 10 kV, and that
   blow-off, when it happens, happens on the rising branch. If pure methane or ethylene flames are
   regularly lost at high voltage after a plateau, or if dilute flames are lost at the lowest
   fields rather than on the way up, the strain picture of 1.4 is wrong (the first would point to
   corona or to the flame reaching the mesh; the second to starvation, which a field cures).

A fourth, weaker: with the mesh positive the law gives more current at the same voltage. If the
order is reversed, the carriers in the gap are heavier than assumed (attached electrons, or soot).

## 7. Interface for the law forward (`docs/wsp/research/law_forward.md`)

Slug `efield`, seven features, four targets. Targets that do not use a feature carry weight zero
on it; the condition must still be present (for the peak targets give the highest voltage of the
sweep as `voltage`).

### 7.1 Features (declaration order)

| # | `name` | SI unit | transform | centre | scale | domain (raw) | Case-file input |
|---|---|---|---|---|---|---|---|
| 0 | `carbon_flow` | 1/s | ln | 1e19 | 1 | 1e18 to 1e20 | `carbon_flow_atoms_s` |
| 1 | `x_fuel` | 1 | ln | 1 | 1 | 0.2 to 1 | `x_fuel` |
| 2 | `is_ethylene` | 1 | identity | 0 | 1 | 0 to 1 | `fuel` = ethylene |
| 3 | `mesh_positive` | 1 | identity | 0 | 1 | 0 to 1 | `polarity` = positive |
| 4 | `voltage` | V | ln | 3000 | 1 | 100 to 10000 | magnitude of the level's `v_cmd` |
| 5 | `sweep_v_max` | V | ln | 5000 | 1 | 100 to 10000 | largest magnitude of `v_cmd` over the programme |
| 6 | `window` | s | ln | 100 | 1 | 5 to 2000 | `window_s` |

A level at less than 100 V is outside the domain; the law's statement there is a current below
0.002 uA.

### 7.2 Targets (declaration)

| `name` | kind | unit | causes | shared quantity |
|---|---|---|---|---|
| `ion_current` | positive | A | | none (lane-local) |
| `flame_end` | event | s | `blowoff`, `other` | none |
| `peak_ion_current` | positive | A | | none (lane-local) |
| `peak_voltage` | positive | V | | none (lane-local) |

The time origin of `flame_end` is the moment the field is switched on (cutoff 0). T2's `cif` is
the sum of the two cause rows; `p_end_in_window` is that sum at `window_s`. No scheduled stop is
declared.

### 7.3 Tensors

Those of section 3 of the forward's document for 7 features: `x.<f>.centre|scale|lo|hi`; per network
`b0`, `skip` [7], `W1` [8, 7], `b1` [8], `w2` [8]. Positive targets: `<t>.median` (ln A or ln V),
`<t>.spread` (ln), `<t>.tail_weight`, `<t>.tail_width`. Event target: `flame_end.blowoff.median`,
`flame_end.other.median` (ln s) and the two `shape` scalars.

### 7.4 Literal assignments

```text
# efield.wsp: W0, zero-shot. Author claude-fable-5-2. Source: docs/wsp/research/phys_efield_w0.md
# features z = (z_c, z_x, e, p, z_v, z_m, z_w), columns 0..6
x.carbon_flow.centre = 10000000000000000000
x.carbon_flow.scale = 1.0
x.carbon_flow.lo = 1000000000000000000
x.carbon_flow.hi = 100000000000000000000
x.x_fuel.centre = 1.0
x.x_fuel.scale = 1.0
x.x_fuel.lo = 0.2
x.x_fuel.hi = 1.0
x.is_ethylene.centre = 0.0
x.is_ethylene.scale = 1.0
x.is_ethylene.lo = 0.0
x.is_ethylene.hi = 1.0
x.mesh_positive.centre = 0.0
x.mesh_positive.scale = 1.0
x.mesh_positive.lo = 0.0
x.mesh_positive.hi = 1.0
x.voltage.centre = 3000.0
x.voltage.scale = 1.0
x.voltage.lo = 100.0
x.voltage.hi = 10000.0
x.sweep_v_max.centre = 5000.0
x.sweep_v_max.scale = 1.0
x.sweep_v_max.lo = 100.0
x.sweep_v_max.hi = 10000.0
x.window.centre = 100.0
x.window.scale = 1.0
x.window.lo = 5.0
x.window.hi = 2000.0

# --- T1, current at a level = min(I_sat, G V^2), falling as 1/V beyond 1.16 V_sat.
# b0 = ln(1.5e-6 * 1.602e-19 C * 1e19 1/s) = ln(2.403e-6 A). Yield proportional to fuel fraction.
ion_current.median.b0 = -12.939
ion_current.median.skip[0:7] = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]
# unit 0: a = ln I_sat - ln I_sc; I_sc = 1.58e-13 A/V2 * (3000 V)^2 = 1.423e-6 A at the centre.
ion_current.median.W1[0, 0:7] = [1.0, 1.0, 0.0, -0.405, -2.0, 0.0, 0.0]
ion_current.median.b1[0] = 0.524
ion_current.median.w2[0] = -1.0
# unit 1: past the peak.
ion_current.median.W1[1, 0:7] = [-1.0, -1.0, 0.0, 0.405, 2.0, 0.0, 0.0]
ion_current.median.b1[1] = -0.824
ion_current.median.w2[1] = -0.5
ion_current.spread.b0 = 0.65
ion_current.spread.skip[0:7] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
ion_current.spread.W1[0, 0:7] = [-1.0, -1.0, 0.0, 0.405, 2.0, 0.0, 0.0]
ion_current.spread.b1[0] = -0.824
ion_current.spread.w2[0] = 0.35
ion_current.tail_weight = 0.1
ion_current.tail_width = 3.0

# --- T2, blow-off by the ion wind. ln tau = ln window + ln V_ext - min(ln V_sat, ln V_max).
# b0 = ln(100 s) + ln(20000 V) - ln(5000 V); V_ext as x^1.5, 3.5 times for ethylene, 0.6 mesh positive.
flame_end.blowoff.median.b0 = 5.991
flame_end.blowoff.median.skip[0:7] = [0.0, 1.5, 1.253, -0.511, 0.0, -1.0, 1.0]
# unit 0: the flame saturates below the highest voltage: ln V_max - ln V_sat, V_sat = 3.90 kV at the centre.
flame_end.blowoff.median.W1[0, 0:7] = [-0.5, -0.5, 0.0, 0.203, 0.0, 1.0, 0.0]
flame_end.blowoff.median.b1[0] = 0.249
flame_end.blowoff.median.w2[0] = 1.0
flame_end.blowoff.shape = 3.0
# other ends: median 600 s for a pure fuel, shorter when dilute.
flame_end.other.median.b0 = 6.397
flame_end.other.median.skip[0:7] = [0.0, 0.7, 0.0, 0.0, 0.0, 0.0, 0.0]
flame_end.other.shape = 1.5

# --- secondary: saturation current and its voltage.
peak_ion_current.median.b0 = -12.939
peak_ion_current.median.skip[0:7] = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]
peak_ion_current.spread.b0 = 0.55
peak_ion_current.spread.skip[0:7] = [0.0, -0.5, 0.0, 0.10, 0.0, 0.0, 0.0]
peak_ion_current.tail_weight = 0.1
peak_ion_current.tail_width = 3.0
# b0 = ln(3000 V) + 0.524 / 2: 3.90 kV.
peak_voltage.median.b0 = 8.268
peak_voltage.median.skip[0:7] = [0.5, 0.5, 0.0, -0.203, 0.0, 0.0, 0.0]
peak_voltage.spread.b0 = 0.45
peak_voltage.spread.skip[0:7] = [0.0, -0.25, 0.0, 0.0, 0.0, 0.0, 0.0]
peak_voltage.tail_weight = 0.1
peak_voltage.tail_width = 3.0
```

What the forward cannot express here, and what that costs. One clock per cause sets both the
probability of blow-off in the window and its timing: the probability is the law's, the time
profile inside the window is cruder (a flame that will be blown off at 3 kV of a 10 kV sweep is
placed later than it should be). A sweep that is not a single rise from near zero (several cycles,
a downward staircase) is treated as one rise over the window. A level of 0 V, the corona current
above about 8 kV, and the burner type are not expressed.

## 8. Sources and limits

### 8.1 Sources

- J. Lawton and F. J. Weinberg, Electrical Aspects of Combustion (1969); H. F. Calcote, ion
  production in hydrocarbon flames (from memory): yield of order 1e-6 per carbon, saturation
  currents, the mechanical effect of ion drift, the limit on the ion wind.
- T. Holm, "Aspects of the mechanism of the flame ionization detector", J. Chromatogr. A 842 (1999)
  221 (from memory): 0.015 C per gram of carbon; equal response per carbon.
- R. S. Sigmond, "Simple approximate treatment of unipolar space-charge-dominated coronas",
  J. Appl. Phys. 53 (1982) 891 (from memory): `I = 2 mu eps0 V^2 / d`.
- M. Di Renzo et al., "An efficient flamelet progress-variable method for modeling non-premixed
  flames in weak electric fields", arXiv:1708.03480 (read in part): heavy-ion mobility 1e-4 m2/V/s
  after Fialkov; the flame as a second electrode.
- S. Karnani and D. Dunn-Rankin, Combust. Flame 162 (2015) 2865; Borgatelli and Dunn-Rankin (2012)
  (from memory only; not retrieved): quadratic rise and saturation of small methane flames, the
  order 0.1 uA per cm3/min.
- Extinction strain rates of methane and ethylene counterflow diffusion flames: textbook values
  (from memory; Seshadri and co-workers for methane).
- D. P. Stocker, "Laminar non-premixed flames of gaseous fuel aboard the International Space
  Station", Central States Section meeting 2022, and the JASMAC-33 slides (2021): the set-up in
  words and the summary findings quoted in 8.3.
- The card `docs/wsp/atlas/cards/PSI-22.json`, the section of `INVESTIGATIONS.md`, and the
  engineer's note (paragraph for the physicist, sections 1 and 2).

### 8.2 Not found or not checked

- Any measured current of the flight, the fuel flows used, the shape of the programmes.
- The papers of the investigators with the laboratory currents; the 0.1 uA per cm3/min is memory.
- The mobility of the negative carriers in the hot plume; the polarity factor is a guess.
- The extinction strain rate of nitrogen-diluted methane and ethylene; the exponent 1.5 is an
  estimate.
- Whether the script velocity behind `carbon_flow_atoms_s` is at 0 C or at room temperature (the
  note leaves it open): an 8 % question, inside the width.

### 8.3 Exposure

The card states, from the final report: the current "peaks then drops sharply, unlike 1-g
plateau"; the gas-jet burner "shows clear parabolic current rise with oscillations, not seen with
coflow"; a positive field compresses the flame and led to oscillation before extinction, a
negative one keeps it near spherical; the field stabilises very dilute flames. The public
summaries add that the peak coincides with the most compact flame and that corona and arcs occur
at high field. One slide shows the labels "~1.0", "~2.5", "~1.8" beside flame pictures without a
unit I could read; I did not use them.

The engineer's note, in the sections I was told to read, contains two facts that come from train
outcomes: 25 flame-end events among about 121 train tests, and the recording range of 9.3 uA. I
saw both. The first told me that most flames survive the field; I wrote section 1.4 (blow-off only
below saturation) after seeing it, and chose `V_ext` and the second clock inside their theoretical
ranges knowing it. T2 is therefore **not blind to the train base rate**. The second only bounds the
currents. The note's header also mentions a validation video named "extinguished early"; it names
no condition and I drew nothing from it. Its sections 3 to 6 were not read.

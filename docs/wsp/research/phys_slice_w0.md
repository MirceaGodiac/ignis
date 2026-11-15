# SLICE (PSI-106): when a microgravity jet flame in coflow lifts and blows off. A W0 law

Author: `claude-fable-5-2`, 15 November 2026. Kind: **W0, literal authoring**. No file under `runs/`
(other than the brief), `data/` or `holdout/` was opened, and of the feedback files only the target
and input sections of the engineer's note. One published paper on this experiment was read (Cao et
al. 2015); what it showed is marked **L\*** and listed in section 8.3. Nothing was fitted. The only
program run was a forward evaluation of the typed law to fill section 5.

## 0. The result in one screen

The base of a jet diffusion flame sits in the slow gas just outside the burner rim and holds there
as long as the gas speed it feels is not much more than the speed at which a flame edge can burn
its way upstream. Two things raise the speed at the base: the coflow, and, on small tubes, the
shear of the fuel jet. One thing lowers what the flame can stand: nitrogen in the fuel.

```text
speed at the flame base     v_k  = 0.66 U_co + 8 l_j (Z_st / Z_st,CH4) U_j / d        l_j = 3.3 um
stability number            Phi  = v_k / (1.4 S_L)
lifted when Phi > 1;  blown off when Phi > 1.3;   ln Phi known to 0.5 (one standard deviation)
```

`U_co` coflow speed, `U_j` mean jet speed, `d` tube inner diameter, `S_L` the laminar burning speed
of the stoichiometric mixture of the jet gas with the cabin air, `Z_st` its stoichiometric mixture
fraction. `S_L` = 0.38 m/s for methane, 0.32 at 70 %, 0.21 at 40 % in nitrogen, and it rises as the
cube of the oxygen fraction.

On the tubes of 1.6 mm and more the jet term is negligible and the law is a coflow speed: lifting
at 0.81, 0.68 and 0.45 m/s for 100, 70 and 40 % methane, blow-off at 1.05, 0.89 and 0.58 m/s. On
the 0.41 mm tube the jet does it: pure methane blows off near 9 m/s of jet speed at any coflow.
Because the crew stepped the flow up until the flame blew off, the probability asked for (blow-off
on a given line of the log) is the chance that the limit lies inside one step: 0.16 at the law's
median limit, 0.03 at half of it, 0.4 at twice.

## 1. Mechanism

### 1.1 The flame base

A diffusion flame does not end at the burner: it ends in a small premixed region, the reaction
kernel, standing about a millimetre off the rim where fuel and air have mixed by diffusion across
the wake of the tube wall. The kernel burns towards the fresh mixture at a speed of the order of
the laminar burning speed `S_L`; the flow carries it away at the local gas speed `v_k`. The
dimensionless group is a Damkohler number of the base,

```text
Da_k = S_L / v_k        (flow time l / v_k over flame time l / S_L)
```

and the flame leaves the rim when `Da_k` falls below a critical value. The detailed computation of
Takahashi and co-workers for a butane jet in air (0.8 mm tube, normal gravity) puts numbers on it:
as the coflow rises from 0.01 to 0.374 m/s the speed at the kernel rises linearly from 0.30 to
0.54 m/s (slope 0.66) while its temperature stays at 1555 K; at 0.54 m/s it lifts. The burning
speed of butane in air is 0.39 m/s, so the critical `v_k / S_L` is **1.4**. The 0.30 m/s at zero
coflow is the buoyant draught of the flame itself; the same authors find 0.005 m/s at the kernel in
microgravity. Hence, without gravity,

```text
v_k = 0.66 U_co + (jet term),      lifted when v_k > 1.4 S_L
```

The factor 0.66 is the shelter of the rim: the kernel sits inside the boundary layer of the coflow
on the tube.

### 1.2 Why microgravity flames hold better, and a check against the sister paper

In normal gravity the buoyant 0.30 m/s is always there. For the SLICE burner of 3.2 mm with a
coflow of about 0.15 m/s the law gives `v_k` = 0.40 m/s on the ground and 0.10 m/s in orbit. The
threshold `1.4 S_L` is 0.53 m/s for pure methane, 0.45 for 70 % and 0.30 for 40 %. So on the ground
the 40 % flame is lifted and the other two attached; in orbit all three are attached. This is what
the investigators report for their eighteen flames (**L\***, read after the law was written down in
this form): "only the 40 % CH4 flame in 1g is lifted and all other flames are attached".

### 1.3 The jet term

Fuel leaving a thin tube drags the gas of the wake with it. What the kernel feels is the wall shear
of the pipe flow, `8 U_j / d`, times a short length: `v_j = l_j * 8 U_j / d`. The length is set by
the classical blow-off gradient of rim-held diffusion flames: pure methane flames in still air
leave small tubes at wall gradients of the order of 1.6e5 1/s (20 m/s in a 1 mm tube; from memory,
uncertain by a factor of three), which with `v_k = 1.4 S_L = 0.53 m/s` gives `l_j` = 3.3 um. For a
diluted fuel the stoichiometric surface lies closer to the jet (`Z_st` = 0.055, 0.093, 0.175 for
100, 70, 40 % methane), where the gas is faster; I scale `l_j` with `Z_st`.

For the tubes of 1.6 mm and more at jet speeds below a few m/s the term is below 0.02 m/s: the
coflow alone decides. For the 0.41 mm tube at 10 m/s it is 0.64 m/s: the jet alone decides.

### 1.4 Dilution, oxygen, fuel

The burning speed carries all three. With a one-step activation temperature of 24,000 K and the
adiabatic temperature of the stoichiometric mixture (2226 K for methane in air, lowered by the
extra nitrogen),

```text
S_L(x) = S_L0 exp( -12000 K (1 / T_ad(x) - 1 / T_ad(1)) ),   T_ad(x) = 298 K + 1928 K / (1 + 0.085 (1 - x) / x)
```

gives 0.32 m/s at 70 %, 0.26 at 50 %, 0.21 at 40 %, 0.15 at 30 %: about `x^0.55` down to half and
steeper below. One per cent more oxygen in the cabin raises `T_ad` by about 60 K and `S_L` by 16 %:
`S_L` goes as the cube of the oxygen fraction near 21 %. Ethane and propane burn at 0.41 m/s.

### 1.5 From lifting to blow-off

Once off the rim the base moves up to where the gas is slow enough. In microgravity the jet decays
to the coflow speed and the coflow is uniform, so a flame that the rim could not hold at
`0.66 U_co = 1.4 S_L`, that is `U_co = 2.1 S_L`, finds little shelter downstream: a flame edge in a
weak mixture-fraction gradient propagates at most at about `2 S_L` (the density ratio to the power
one half). The lifted range is therefore narrow. I put blow-off at `Phi = 1.3`, with an uncertainty
that covers 1.0 to 2.0. Methane has a Schmidt number below one, for which a free jet has no stable
lifted flame at all (Chung and Lee): jet-driven lifting on the small tubes should pass to blow-off
within a step or two.

### 1.6 What the log asks, and why it changes the probability

The crew raised the fan or the fuel flow line by line until the flame blew off. A flame that is
burning on a given line has survived the line before, which was only a little gentler. The chance
of blow-off on a line is therefore not the chance that the condition is beyond the limit, but the
chance that the limit lies between the last line and this one. With my uncertainty `s` = 0.5 on
`ln Phi` and steps of about 10 % in the controlling speed, the hazard of a normal distribution
gives per line 0.011, 0.058, 0.16, 0.30 at two and one standard deviations below, at, and one above
the median limit. In logit form: `-1.66 + 2.0 (ln Phi - ln 1.3)`. The lifted state persists once
reached, so for it the plain probability `P(Phi > 1)` is used.

### 1.7 Length (secondary)

As for SPICE (`phys_spice_w0.md`, section 1.1): `L = K mdot F(r) / Z_st`, `K` = 1.30 mm per mg/s,
`F` between 1 (strong coflow) and 1.61 (fast jet). For the 3.2 mm burner at 2.5 mg/s of methane it
gives 62 mm; the published normal-gravity length law times the 1.28 that the investigators measured
between orbit and ground (**L\***) gives 65 mm.

### 1.8 The earlier kernel (`property_jets_v2.rs`), as a colleague's work

It computes a conserved scalar and the axial momentum of a jet in coflow and reports the
stoichiometric surface. It states that it predicts no lift-off and no blow-off, and its companion
note explains why: stabilisation is a balance of flow speed against edge propagation, which a
scalar field does not contain. That diagnosis is right. What is missing is the next step: the
balance can be written with one flame property, `S_L`, and two transport numbers (the shelter
factor 0.66 and the length `l_j`), without solving the field. The kernel's rim is "idealised, no
accuracy claim near it", and the rim is where this question lives.

## 2. Targets

From the engineer's note `docs/wsp/feedback/slice_feasibility.md` (its paragraph for the physicist
and its sections 1 and 2).

| Target | Kind | A forecast may know |
|---|---|---|
| `blow_off` (primary): the flame blows off at this logged setting, or during the adjustment noted on its line | binary | fuel, fuel fraction, tube diameter, jet flow and speed, coflow speed, cabin oxygen and pressure, and the earlier settings of the same flame |
| `lifted` (secondary): the flame base has left the rim at this setting | binary | the same |
| `flame_length` (mine, not scorable from text): luminous length | positive, m | the same |

Lift-off height and flame length exist only in images. The law uses the setting of the line alone;
of the earlier settings it uses only the fact that the flame survived them (1.6).

## 3. The law

```text
z_co = ln(U_co / 0.3 m/s),  z_j = ln(U_j / 1 m/s),  z_d = ln(d / 1.6 mm),  z_x = ln(x_fuel),  z_o = ln(x_O2 / 0.21)
e2 = 1 for ethane,  e3 = 1 for propane

A = ln(0.66 U_co)                                  coflow part of the base speed
B = ln(8 l_j U_j / d) + ln(Z_st / 0.0552)          jet part
t = B - A = -2.485 - z_co + z_j - z_d - 1.3 z_x + 0.065 e2 + 0.088 e3
ln v_k  = A + sp(t),     sp(t) = 0.31 relu(t + 2.2) + 0.38 relu(t) + 0.31 relu(t - 2.2)     (ln(1 + e^t) in three pieces)
ln S_L  = ln(0.38 m/s) + 0.55 z_x - 0.5 relu(-0.693 - z_x) + 3.0 z_o + 0.076 (e2 + e3)
ln Phi  = ln v_k - ln 1.4 - ln S_L
        = -0.987 + z_co - 0.55 z_x - 3.0 z_o - 0.076 (e2 + e3) + sp(t) + 0.5 relu(-0.693 - z_x)

logit P(lifted)   = 3.4 ln Phi
logit P(blow_off) = -1.66 + 2.0 (ln Phi - ln 1.3) = -2.184 + 2.0 ln Phi
ln L = ln(K mdot / Z_st) + 0.148 clip( ln(0.5 U_j / U_co), 0, 3.22 )
```

The slope 3.4 is `1.7 / 0.5`: a logistic of the same width as a normal uncertainty of 0.5 on
`ln Phi`. The three-piece `sp` is exact at `t` = -2.2, 0, 2.2 and within 0.1 elsewhere.

Limits. `U_co` and `U_j` to zero: `Phi` to zero, the flame holds (in still gas the real danger is
radiative quenching of a slow flame, which is not blow-off and not in this law). `d` large: no jet
term. `x_fuel` small: `S_L` falls faster than any power and every flame is blown off. More oxygen:
more stable. Normal gravity: add 0.30 m/s to `v_k` (not a term of the network: the experiment is
in orbit). Range: 0.4 to 3.3 mm tubes, coflow 2 to 150 cm/s, jet 0.02 to 100 m/s, methane 0.3 to
1, ethane, propane, cabin oxygen 0.18 to 0.25.

## 4. Constants

Classes: **P** property, **T** theory, **L** published experiment or computation, **L\*** paper on
this experiment, **J** judgement.

| Constant | Value | Uncertainty | Class | Origin |
|---|---|---|---|---|
| Shelter factor, kernel speed over coflow speed | 0.66 | 0.4 to 1.0 | L, J | slope of `v_k` against coflow in Takahashi et al. 2022 (butane, 0.8 mm tube, normal gravity); carried to microgravity by judgement |
| Critical `v_k / S_L` at lifting | 1.4 | 1.1 to 1.8 | L | 0.54 m/s over 0.39 m/s, same computation; agrees with the 1g and 0g states of Cao et al. 2015 (L\*) |
| Blow-off over lifting, in `Phi` | 1.3 | 1.0 to 2.0 | T, J | 1.5 |
| `l_j` | 3.3e-6 m | factor 3 | L, J | 0.53 m/s over a blow-off wall gradient of 1.6e5 1/s for pure methane (from memory) |
| Scaling of `l_j` with dilution | `Z_st / 0.0552` | exponent 0 to 1.5 | T | 1.3 |
| `S_L` methane, ethane, propane in air | 0.38, 0.41, 0.41 m/s | 5 % | P | stoichiometric, 298 K, 1 atm |
| `S_L` of 70, 40 % methane in nitrogen | 0.32, 0.21 m/s | 15 % | T | 1.4 |
| One-step activation temperature | 24,000 K | 20 % | P | methane-air overall (from memory) |
| Adiabatic temperature, methane in air | 2226 K | 1 % | P | |
| Oxygen exponent of `S_L` | 3.0 | 2 to 4 | T | 1.4 |
| `Z_st` methane 100, 70, 40 %; ethane; propane | 0.0552, 0.0927, 0.1745; 0.0589; 0.0603 | 1 % | P | stoichiometry; `d ln Z_st / d ln x` = -1.3 |
| Width of `ln Phi` | 0.5 | | J | combined: shelter 0.25, critical ratio 0.2, `S_L` 0.15, coflow reading 0.1, and 0.35 for carrying a 1g computation to orbit |
| Step of the crew in the controlling speed | 10 % | 5 to 25 % | J | 1.6; sets the per-line level -1.66 |
| `K`, `F`, `r0` of the length law | as SPICE | | T, L | `phys_spice_w0.md` section 4 |

## 5. Registered expectations

Written before any comparison with project data. Cabin oxygen 0.21.

Coflow-driven, 3.2 mm tube, jet 0.5 m/s. Entries: P(lifted) / P(blow-off on that line).

| Fuel | 10 cm/s | 20 cm/s | 30 cm/s | 45 cm/s | 60 cm/s | 80 cm/s |
|---|---|---|---|---|---|---|
| Methane 100 % | 0.00 / 0.00 | 0.01 / 0.01 | 0.03 / 0.02 | 0.12 / 0.03 | 0.27 / 0.06 | 0.49 / 0.10 |
| Methane 70 % | 0.00 / 0.00 | 0.02 / 0.01 | 0.06 / 0.02 | 0.21 / 0.05 | 0.42 / 0.08 | 0.66 / 0.14 |
| Methane 40 % | 0.01 / 0.01 | 0.07 / 0.02 | 0.22 / 0.05 | 0.53 / 0.11 | 0.75 / 0.18 | 0.89 / 0.28 |
| Ethane, propane | 0.00 / 0.00 | 0.01 / 0.01 | 0.03 / 0.01 | 0.10 / 0.03 | 0.22 / 0.05 | 0.43 / 0.09 |

Jet-driven, coflow 20 cm/s. Entries as above, against the mean jet speed.

| Fuel | Tube | 2 m/s | 5 m/s | 10 m/s | 20 m/s | 40 m/s |
|---|---:|---|---|---|---|---|
| Methane 100 % | 0.41 mm | 0.08 / 0.03 | 0.42 / 0.09 | 0.79 / 0.19 | 0.95 / 0.40 | 1.00 / 0.73 |
| Methane 100 % | 0.76 mm | 0.04 / 0.02 | 0.15 / 0.04 | 0.46 / 0.09 | 0.81 / 0.21 | 0.96 / 0.44 |
| Methane 100 % | 1.6 mm | 0.02 / 0.01 | 0.05 / 0.02 | 0.13 / 0.04 | 0.43 / 0.09 | 0.80 / 0.20 |
| Methane 70 % | 0.41 mm | 0.33 / 0.07 | 0.81 / 0.21 | 0.96 / 0.40 | 0.99 / 0.71 | 1.00 / 0.91 |
| Methane 70 % | 0.76 mm | 0.13 / 0.03 | 0.50 / 0.10 | 0.83 / 0.22 | 0.96 / 0.43 | 1.00 / 0.74 |
| Methane 40 % | 0.41 mm | 0.92 / 0.32 | 0.99 / 0.62 | 1.00 / 0.86 | 1.00 / 0.96 | 1.00 / 0.99 |
| Methane 40 % | 0.76 mm | 0.72 / 0.17 | 0.96 / 0.41 | 0.99 / 0.65 | 1.00 / 0.88 | 1.00 / 0.97 |
| Methane 40 % | 1.6 mm | 0.37 / 0.08 | 0.80 / 0.20 | 0.95 / 0.40 | 0.99 / 0.63 | 1.00 / 0.87 |

Median limits (where the law puts lifting and blow-off; one standard deviation is a factor 1.65):

| | Methane 100 % | Methane 70 % | Methane 40 % | Ethane, propane |
|---|---:|---:|---:|---:|
| Coflow at lifting, large tube | 0.81 m/s | 0.68 m/s | 0.45 m/s | 0.87 m/s |
| Coflow at blow-off, large tube | 1.05 m/s | 0.89 m/s | 0.58 m/s | 1.13 m/s |
| Jet speed at blow-off, 0.41 mm tube, coflow 20 cm/s | 8.7 m/s | 4.2 m/s | 1.3 m/s | about 9 m/s |
| Jet speed at blow-off, 0.76 mm tube, coflow 20 cm/s | 16 m/s | 7.8 m/s | 2.4 m/s | about 17 m/s |

Oxygen: 40 % methane at 45 cm/s on the large tube: P(lifted) 0.65, 0.53, 0.41, 0.31 at 20, 21, 22,
23 % oxygen.

Length (not scorable now): methane on the 3.2 mm tube at 20 cm/s coflow: 23 mm at 1 mg/s, 103 mm
at 4 mg/s; 70 % methane at 2 mg/s of jet gas 30 mm; 40 % at 4 mg/s 29 mm. Width 0.22 in ln.

In words:

1. On the tubes of 1.6 mm and more, pure methane, ethane and propane do not blow off by coflow
   below about 60 cm/s; if the fan cannot give more than that, their flames end for other reasons
   (fuel exhausted, the ground).
2. The coflow a flame stands scales with the burning speed of its fuel mixture: 1 : 0.84 : 0.55
   for 100, 70 and 40 % methane.
3. On the 0.41 mm tube blow-off comes from the fuel flow, at a jet speed proportional to the tube
   diameter and, for a diluted fuel, several times lower.
4. Lifted flames are seen over a narrow range before blow-off (a factor 1.3 in speed).
5. On any single line of the log the probability of blow-off stays below about 0.3 except for
   strongly diluted fuel on the smallest tubes.

## 6. What would show the law is wrong

1. **Dilution does not move the limit as the burning speed.** The law says 40 % methane blows off
   at 0.55 of the coflow (or, on a small tube, at about one seventh of the jet speed) that pure
   methane stands. If the three methane mixtures blow off at the same speeds, the base is not held
   by a propagating edge; if 40 % is far weaker still (one fifth of the coflow), the activation
   temperature or the role of `Z_st` is wrong.
2. **Large tubes blow off by fuel flow at low coflow, or small tubes do not.** The law says the
   jet matters as `U_j / d`: a 3.2 mm tube would need 70 m/s, a 0.41 mm tube 9 m/s. A blow-off of
   pure methane on the 1.6 or 3.2 mm tube at a jet speed of a few m/s with the fan low would mean
   the jet reaches the base through something other than wall shear (for instance the velocity on
   the stoichiometric surface far above the rim), and `l_j / d` would have to be replaced by a
   constant fraction of the jet speed.
3. **Microgravity flames lift at the coflow speeds of normal gravity.** The law says that in orbit
   a pure methane flame on a large tube holds to about 0.8 m/s of coflow, against about 0.35 m/s
   on the ground, and 40 % methane to 0.45 m/s where on the ground it is lifted at any coflow. If
   pure methane is lifted or lost in orbit at 0.3 to 0.4 m/s, the shelter factor is near 1.5
   instead of 0.66, or the kernel feels the jet's entrainment.

Calibration, separately: over all lines given a blow-off probability between 0.1 and 0.3, about
one in five should be a blow-off; if it is one in two, the crew's steps were coarser than 10 %.

## 7. Interface for the law forward (`docs/wsp/research/law_forward.md`)

Slug `slice`, eight features, three targets.

### 7.1 Features (declaration order)

| # | `name` | SI unit | transform | centre | scale | domain (raw) | Case-file input |
|---|---|---|---|---|---|---|---|
| 0 | `u_coflow` | m/s | ln | 0.3 | 1 | 0.02 to 1.5 | `u_coflow_cm_s` / 100 |
| 1 | `u_jet` | m/s | ln | 1 | 1 | 0.02 to 100 | `u_jet_m_s` |
| 2 | `burner_id` | m | ln | 0.0016 | 1 | 0.0003 to 0.004 | `burner_id_mm` / 1000 |
| 3 | `x_fuel` | 1 | ln | 1 | 1 | 0.3 to 1 | `x_fuel` |
| 4 | `x_o2` | 1 | ln | 0.21 | 1 | 0.18 to 0.25 | `x_o2_cabin` |
| 5 | `is_ethane` | 1 | identity | 0 | 1 | 0 to 1 | `fuel_hydrocarbon` |
| 6 | `is_propane` | 1 | identity | 0 | 1 | 0 to 1 | `fuel_hydrocarbon` |
| 7 | `mdot_jet` | kg/s | ln | 0.000001 | 1 | 0.00000002 to 0.00005 | `mdot_jet_mg_s` / 1e6 (length only) |

Where the log has no cabin oxygen (the note counts some lines), the forward refuses the case. If
the engineer prefers to carry the day's value or 0.21, that is an assumption to record; a change of
one per cent of the oxygen value (0.210 to 0.212) moves the blow-off logit by 0.06. `step_in_test`
and the cabin pressure are not used.

### 7.2 Targets (declaration)

| `name` | kind | unit |
|---|---|---|
| `blow_off` | binary | |
| `lifted` | binary | |
| `flame_length` | positive | m (lane-local) |

### 7.3 Tensors

Those of section 3 of the forward's document for 8 features: `x.<f>.centre|scale|lo|hi`; networks
`blow_off.logit`, `lifted.logit`, `flame_length.median`, `flame_length.spread`, each with `b0`,
`skip` [8], `W1` [8, 8], `b1` [8], `w2` [8]; `flame_length.tail_weight`, `flame_length.tail_width`.

### 7.4 Literal assignments

```text
# slice.wsp: W0, zero-shot. Author claude-fable-5-2. Source: docs/wsp/research/phys_slice_w0.md
# features z = (z_co, z_j, z_d, z_x, z_o, e2, e3, z_m), columns 0..7
x.u_coflow.centre = 0.3
x.u_coflow.scale = 1.0
x.u_coflow.lo = 0.02
x.u_coflow.hi = 1.5
x.u_jet.centre = 1.0
x.u_jet.scale = 1.0
x.u_jet.lo = 0.02
x.u_jet.hi = 100.0
x.burner_id.centre = 0.0016
x.burner_id.scale = 1.0
x.burner_id.lo = 0.0003
x.burner_id.hi = 0.004
x.x_fuel.centre = 1.0
x.x_fuel.scale = 1.0
x.x_fuel.lo = 0.3
x.x_fuel.hi = 1.0
x.x_o2.centre = 0.21
x.x_o2.scale = 1.0
x.x_o2.lo = 0.18
x.x_o2.hi = 0.25
x.is_ethane.centre = 0.0
x.is_ethane.scale = 1.0
x.is_ethane.lo = 0.0
x.is_ethane.hi = 1.0
x.is_propane.centre = 0.0
x.is_propane.scale = 1.0
x.is_propane.lo = 0.0
x.is_propane.hi = 1.0
x.mdot_jet.centre = 0.000001
x.mdot_jet.scale = 1.0
x.mdot_jet.lo = 0.00000002
x.mdot_jet.hi = 0.00005

# --- lifted: logit = 3.4 ln Phi. ln Phi = -0.987 at the centre (pure methane, coflow 0.3 m/s):
# 0.66 * 0.3 / (1.4 * 0.38). Coflow +1; dilution lowers S_L (-0.55 on ln Phi per e-fold of x);
# oxygen raises it as the cube; ethane and propane burn 8 % faster.
lifted.logit.b0 = -3.356
lifted.logit.skip[0:8] = [3.4, 0.0, 0.0, -1.87, -10.2, -0.258, -0.258, 0.0]
# units 0 to 2: the jet term, ln(1 + e^t) in three pieces, t = ln(jet part / coflow part);
# t = -2.485 at the centre: 8 * 3.3e-6 * 1 / 0.0016 = 0.0165 m/s against 0.198 m/s.
lifted.logit.W1[0, 0:8] = [-1.0, 1.0, -1.0, -1.3, 0.0, 0.065, 0.088, 0.0]
lifted.logit.b1[0] = -0.285
lifted.logit.w2[0] = 1.054
lifted.logit.W1[1, 0:8] = [-1.0, 1.0, -1.0, -1.3, 0.0, 0.065, 0.088, 0.0]
lifted.logit.b1[1] = -2.485
lifted.logit.w2[1] = 1.292
lifted.logit.W1[2, 0:8] = [-1.0, 1.0, -1.0, -1.3, 0.0, 0.065, 0.088, 0.0]
lifted.logit.b1[2] = -4.685
lifted.logit.w2[2] = 1.054
# unit 3: below 50 % fuel the burning speed falls faster.
lifted.logit.W1[3, 3] = -1.0
lifted.logit.b1[3] = -0.693
lifted.logit.w2[3] = 1.7

# --- blow-off on a line: logit = -2.184 + 2.0 ln Phi (the limit lies inside a 10 % step).
blow_off.logit.b0 = -4.158
blow_off.logit.skip[0:8] = [2.0, 0.0, 0.0, -1.10, -6.0, -0.152, -0.152, 0.0]
blow_off.logit.W1[0, 0:8] = [-1.0, 1.0, -1.0, -1.3, 0.0, 0.065, 0.088, 0.0]
blow_off.logit.b1[0] = -0.285
blow_off.logit.w2[0] = 0.62
blow_off.logit.W1[1, 0:8] = [-1.0, 1.0, -1.0, -1.3, 0.0, 0.065, 0.088, 0.0]
blow_off.logit.b1[1] = -2.485
blow_off.logit.w2[1] = 0.76
blow_off.logit.W1[2, 0:8] = [-1.0, 1.0, -1.0, -1.3, 0.0, 0.065, 0.088, 0.0]
blow_off.logit.b1[2] = -4.685
blow_off.logit.w2[2] = 0.62
blow_off.logit.W1[3, 3] = -1.0
blow_off.logit.b1[3] = -0.693
blow_off.logit.w2[3] = 1.0

# --- luminous length = K mdot F / Z_st. b0 = ln(1296) + ln(1e-6) - ln(0.0552): 23.5 mm per mg/s
# of methane in strong coflow; Z_st rises with dilution (1.3 per e-fold), ethane, propane.
flame_length.median.b0 = -3.752
flame_length.median.skip[0:8] = [0.0, 0.0, 0.0, 1.3, 0.0, -0.065, -0.088, 1.0]
# units 0, 1: ln F = 0.148 clip(ln(0.5 U_j / U_co), 0, 3.219); 0.511 at the centre.
flame_length.median.W1[0, 0] = -1.0
flame_length.median.W1[0, 1] = 1.0
flame_length.median.b1[0] = 0.511
flame_length.median.w2[0] = 0.148
flame_length.median.W1[1, 0] = -1.0
flame_length.median.W1[1, 1] = 1.0
flame_length.median.b1[1] = -2.708
flame_length.median.w2[1] = -0.148
flame_length.spread.b0 = 0.22
flame_length.spread.skip[0:8] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
flame_length.tail_weight = 0.1
flame_length.tail_width = 3.0
```

What the forward cannot express here. The exact conditional probability of 1.6 needs the previous
line of the same flame (the size of the step actually taken); the forward sees one line, so a
fixed step of 10 % is built into the level. A line on which the crew made a large jump is given
too low a probability, a line repeated at the same setting too high. `mdot_jet` must be present
for every case though only the length reads it.

## 8. Sources and limits

### 8.1 Sources

- F. Takahashi, L. Smith, D. T. Souza, V. R. Katta, "Critical conditions at liftoff limit of a
  laminar n-butane-air jet diffusion flame", Proc. Combust. Inst. 39 (2022), author's manuscript:
  the reaction kernel, its speed against coflow (0.30 to 0.54 m/s between 0.01 and 0.374 m/s),
  lifting at 0.38 to 0.41 m/s for a 2 m/s jet from a 0.8 mm tube, kernel speeds of 0.005 m/s in
  microgravity, `Da = S_L / v_k`. A ground study that extends SLICE; no SLICE outcome in it.
- S. Cao, B. Ma, B. A. V. Bennett, D. Giassi, D. P. Stocker, F. Takahashi, M. B. Long,
  M. D. Smooke, "A computational and experimental study of coflow laminar methane/air diffusion
  flames: effects of fuel dilution, inlet velocity, and gravity", Proc. Combust. Inst. 35 (2015)
  897-903 (NASA preprint). **This experiment.** Taken: burner 3.24 mm inside a 76.2 mm duct, the
  eighteen conditions (40, 70, 100 % methane, 24 to 89 cm/s), which flames were lifted, length
  proportional to methane flow and 28 % longer in orbit.
- S. H. Chung and B. J. Lee, Combust. Flame 86 (1991) 62 (from memory): no stable lifted laminar
  flame in a free jet for a Schmidt number below one.
- Blow-off of methane diffusion flames from small tubes; burning speeds; activation temperature:
  textbook values (from memory).
- `phys_spice_w0.md` for the length law and its sources.
- The card `docs/wsp/atlas/cards/PSI-106.json`, the section of `INVESTIGATIONS.md`, the engineer's
  note (paragraph for the physicist, sections 1 and 2).

### 8.2 Not found or not checked

- Any measured lifting or blow-off limit of SLICE; the range of the fan in cm/s.
- A blow-off wall gradient for laminar methane diffusion flames from a paper I could open; the
  1.6e5 1/s is memory and carries the whole jet term.
- Whether the shelter factor 0.66, computed with buoyancy, holds without it.
- The burning speed of nitrogen-diluted methane from a table; it is computed here.
- Ethylene (the card lists two ethylene days; the note's fuels are methane, ethane and propane):
  for ethylene `S_L` = 0.68 m/s would replace 0.38.

### 8.3 Exposure

From the paper on this experiment I saw that, of eighteen microgravity flames on the 3.2 mm burner
at low coflow, none was lifted, and that on the ground only the 40 % flame was. I had written
`v_k = 0.66 U_co` and the threshold 1.4 from the butane computation before reading it; the check
of 1.2 was made after. I saw no blow-off or lifting limit of the flight. From the engineer's note
I read how the log was taken (stepping until blow-off), the tube sizes and the input names; its
sections 3 to 7 were not read. The per-line level of 1.6 uses the stepping procedure, not a count
of outcomes.

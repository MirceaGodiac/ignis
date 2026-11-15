# Quenching and blow-off of PMMA rods, spheres, films and slabs in a slow duct flow: a W0 law from first principles

Author: `claude-fable-5-2`, 15 November 2026. Kind: **W0, literal authoring, zero-shot**. Slug `bass1`:
BASS (PSI-26) and the non-fabric burns of BASS-II (PSI-25). No outcome file of this project was opened
(section 8.3 lists what was read; it includes published papers about these very burns, whose numbers
are marked **L\***). Every number was typed from a property table, a derivation shown here, a cited
publication or a stated judgement. Nothing was fitted. The only programs run were evaluations of the
typed formulas to fill the tables.

## 0. The result in one screen

A flame on a solid in a slow stream lives between two limits in oxygen mole fraction `X`. On the
slow side the hot surface radiates and the cold interior of a thick solid soaks up heat, while the
flame's supply of heat falls as the square root of the stretch rate; on the fast side chemistry has
no time. For a rod tip or a sphere of nose radius `R` in a duct flow `U`, `t` seconds after ignition:

```text
X_th = x_s + (q_r + b_c / sqrt(t)) / (k_s sqrt(a_e))        thermal (radiative quench) limit
X_k  = x_b + s_b a_n                                        kinetic limit (floor, then blow-off)
flame holds when  X > max(X_th, X_k)

a_e = 1.5 (U + D_c / R) / R        stretch rate the supply sees, 1/s (forced flow plus diffusion)
a_n = 1.5 (U / R) (R / r0)^n       stretch rate of the blow-off correlation, 1/s
```

`x_s = 0.030`, `q_r = 7.2 kW/m2` (surface radiation), `b_c = 110 kW m-2 s^0.5` (conduction into the
cold solid), `k_s = 62 kW m-2 s^0.5` (heat the flame can deliver per unit oxygen fraction),
`D_c = 0.88 cm2/s`, `x_b = 0.134`, `s_b = 0.00035 s`, `r0 = 0.318 cm`, `n = 0.25`.
For a flat sample (film, slab) whose flame edge faces the flow, `sqrt(a_e)` becomes
`U / sqrt(D_h)` with `D_h = 1.7 cm2/s`, the floor is 0.138 and blow-off is linear in `U`.

What follows for fire safety, all at 1 atm:

- **The floor.** No PMMA flame below about 13.4 % oxygen at any speed (one standard deviation
  0.8 % O2); the most flammable flow is 0.5 to 3 cm/s.
- **The quench speed of a thick sample is a clock, not a constant.** The interior takes as much heat
  as the surface radiates at `t_c = (b_c / q_r)^2 = 230 s` after ignition. The stretch rate at
  quench is `a_q = [(q_r + b_c / sqrt(t)) / (k_s (X - x_s))]^2`, and the duct speed at quench is
  `U_q = 2 R a_q / 3 - D_c / R`. A 2 cm sphere in air goes out below about 1.2 cm/s one minute after
  ignition and below about 0.1 cm/s after five minutes. The law is known to a factor of about 3 in
  speed.
- **Small bodies have no quench speed.** For `R` below about 0.6 cm (the three rod sizes, the 1 cm
  sphere) diffusion alone (`D_c / R`) feeds a heated flame; while the fan turns at all, only the
  floor and blow-off limit them.
- **Blow-off** of the stagnation flame: `U_b = (X - x_b) R^0.75 r0^0.25 / (1.5 s_b)`; 22 cm/s for a
  1/4 inch rod at 17 %, 46 cm/s in air.
- **Films** (0.1 to 0.4 mm, opposed flow): quench at `U_q = 0.15 cm/s / (X - 0.03)`, 0.8 cm/s in air
  and 1.2 cm/s at 16 %, whatever the thickness.

## 1. Mechanism from first principles

### 1.1 What the flame can give the surface

Take the forward stagnation point of a burning sphere or rod tip. Outside the flame sheet the gas
carries oxygen, inside it fuel vapour. With unit Lewis number the combination
`c_p T + Y_O dh_O` (`dh_O` heat released per kg of oxygen) has no source, so its flux to the wall is
the transfer conductance `g` times its difference across the layer. Oxygen does not reach the wall,
so that flux is heat:

```text
q_w = g (Y_O dh_O - c_p (T_s - T_inf))          for weak blowing
g   = 0.763 Sc^-0.6 sqrt(rho_inf mu_inf a)      axisymmetric stagnation point, a = du_e/dx
```

For a sphere in a uniform stream `a = 3U / 2R`. This is the classical stagnation-point result (the
Fay and Riddell form with the product `rho mu` taken at the edge). With air at 300 K,
`g = 4.29e-3 sqrt(a)` kg m-2 s-1 (`a` in 1/s), `Y_O = 1.107 X`, `dh_O = 13.1 MJ/kg`:

```text
q_w = k_s sqrt(a) (X - x_s0),   k_s = 62.2 kW m-2 s^0.5,   x_s0 = c_p (T_s - T_inf) / (1.107 dh_O) = 0.025
```

In air at `a = 1 /s` the flame can hand the surface 11.5 kW/m2. That is the whole budget.

### 1.2 What the surface does with it: the thermal limit

```text
q_w = m'' L_v + q_r + q_c
```

`m'' L_v` gasifies fuel, `q_r = eps sigma (T_s^4 - T_inf^4)` is radiated, `q_c` is conducted into
solid that is still cold. PMMA holds its surface near 630 K whatever the flame does, so the losses
do not fall when the supply does. As `a` falls, `m''` falls; the flame sheet moves towards the wall
(its position in the mixture-fraction coordinate is `xi_f = nu Y_F,w / (Y_O + nu Y_F,w)`) and its
temperature `T_f = T_s + (Y_O dh_O / c_p - (T_s - T_inf)) xi_f` falls with it. At `a = 2 /s` in air a
flame at 1100 K still burns (measured: Olson and T'ien 2000), which needs `xi_f = 0.2` and only 3 %
of the budget for gasification. So the flame goes out, to within a few per cent, when

```text
k_s sqrt(a) (X - x_s) = q_r + q_c,      x_s = 0.030
```

`x_s` is `x_s0` plus 0.005 for the fuel a 1100 K flame still needs. This is T'ien's radiative
extinction at low stretch (T'ien 1986; Foutch and T'ien 1987), written so that the oxygen limit is
explicit. The **loss number** is

```text
Lambda = (q_r + q_c) / (k_s sqrt(a_e) (X - x_s))        quench at Lambda = 1
```

Check with no microgravity input: flames under PMMA cylinders of very large radius in air (1 g,
buoyant stretch) lost 7.7 kW/m2 by radiation and 5 to 9 kW/m2 into the solid (8.4 near the limit),
stayed uniform down to 3 /s and went out at 1.5 to 2 /s. The formula gives
`a_q = (15.6 / (62.2 * 0.18))^2 = 1.9 /s`. Their surface balance had 90 % of the heat reaching the
surface lost at the limit; the derivation above says 97 %.

### 1.3 The kinetic limit: a floor, then blow-off

A flame also dies when reaction cannot keep up with the flow through it. At the stagnation point of
PMMA rods the limit is linear in stretch rate in 1 g (Marcum, Olson and Ferkul 2016: five radii,
15 to 18 % O2), extrapolating to 13.4 % at zero stretch, and the flight boundary of BASS-II is
reported as the same line with a lowest burning test at 13.6 % (Olson and Ferkul 2015, **L\***). A
4 cm PMMA sphere in a different rig went out at 13.5 % at 7 to 9 /s (Li et al. 2024, SoFIE). I take
the floor from these: `x_b = 0.134`.

An honest remark. A one-step Arrhenius flame with the 1100 K anchor above would give a floor near
10 to 11 % at 5 to 10 /s (I computed it). The measured floor is 2.5 % O2 higher. Gas radiation at
long residence times and incomplete combustion (BASS-II measured CO/CO2 far above efficient
burning) both act in that direction; I do not derive them. The floor is therefore class L, not T.

The slope: Marcum et al. align their 1 g line with the flight boundary by a buoyant stretch of
78 /s for the 0.318 cm rod, whose 1 g limit is 16 %: `s_b = (0.16 - 0.134) / 78 = 0.00033 s`. They
find the blow-off stretch falling as `R^-0.5`. The SoFIE sphere (R = 2 cm) blew off at 15.0 % at
25.7 /s and at 14.4 % at 37 /s, which asks for an exponent between -0.1 and 0.3 in the form
`a_n = a (R / r0)^n`. I take `s_b = 0.00035 s` and `n = 0.25` with a standard deviation as large
as the value.

### 1.4 The thick solid: a heat-up clock

A solid whose surface is suddenly held at `T_s` draws `q_c = lambda_s (T_s - T_inf) / sqrt(pi alpha_s t)`.
For PMMA (`lambda_s = 0.21 W/m/K`, `alpha_s = 1.2e-7 m2/s`) this is `b_c / sqrt(t)` with
`b_c = 112 kW m-2 s^0.5`; I type 110. It equals the surface radiation at

```text
t_c = (b_c / q_r)^2 = 230 s
```

Before `t_c` conduction into the solid decides the limit; long after it, radiation alone. The
steady conduction of a slowly regressing surface is `m'' c_s (T_s - T_inf)`, 0.5 kW/m2 near the
limit, and is already inside the heat of gasification. Measured in SoFIE (times of the telemetry,
read as time since ignition): 3.6 kW/m2 at 757 s (the formula: 4.0), 1.7 at 1238 s (3.1; the 4 cm
sphere is warm through by then), 9.5 at 257 s (6.9). I use the semi-infinite form for every thick body and give the term a 50 % uncertainty.

Three consequences. (a) A flame that has just been lit needs far more flow or oxygen than one that
has burned for minutes: ignition limits lie above flammability limits, and an igniter must stay on
for `t_min = [b_c / (k_s sqrt(a_e) (X - x_s) - q_r)]^2`, minutes near the limit. BASS-II reports
exactly this for rods (ignition delays up to ten times the standard igniter time, **L\***). (b) The
quench speed found by stepping the flow down depends on how long the steps were held. (c) A thin
film has no interior: `q_c = 0` after about a second, and its thickness does not enter.

### 1.5 Radius: stretch, and diffusion that needs no flow

Forced stretch is `a_f = 3U / 2R`: at the same duct speed a small body is fed better. And a small
body is fed by diffusion even without flow, at a rate `rho D Y_O / R` per unit area. In the
stretch-rate language this is an extra speed `u_d = D_c / R`, with `D_c` the diffusivity of oxygen
at the mean temperature of the layer outside a near-limit flame (700 K): `0.21 (700/300)^1.7 = 0.88`
cm2/s. So `a_e = 1.5 (U + D_c / R) / R`. Its Peclet number `U R / D_c` is 1 at 0.9 cm/s for a 2 cm
sphere and at 2.8 cm/s for a 1/4 inch rod: at BASS speeds diffusion is never negligible.

With the fan off the term does not last: the duct is 7.6 cm wide, and the oxygen within reach is
used in `(3.8 cm)^2 / 0.2 cm2/s`, about a minute. The law applies from 0.3 cm/s up.

### 1.6 Flat samples: an edge instead of a stagnation point

The flame on a film or slab is held by its upstream edge. Its length scale is `D_h / U_e`, with
`D_h` the diffusivity of the hot gas at the edge (air near 1100 K: 1.7 cm2/s) and `U_e` the gas
speed relative to the edge. The supply per unit area is then `k_s U_e / sqrt(D_h)`, which is the
stagnation result with `a = U_e^2 / D_h`. The limit:

```text
X_th = x_s + A_e / U_e,     A_e = (q_r / k_s) sqrt(D_h) = 0.151 cm/s  (opposed flow)
```

The same criterion written with the gas properties at the flame is Bhattacharjee's radiative
extinction speed `q_r / (rho_g c_g (T_f - T_v))`, 0.8 to 1.6 cm/s in air depending on where the gas
properties are taken, independent of thickness (Bhattacharjee et al. 2003). For a flame spreading
with the flow I multiply `A_e` by 1.5 (judgement: the whole burning length radiates; my colleague
uses 1.9 for fabric). The kinetic limit is linear in `U`, `X_k = 0.138 + 0.0016 U` (opposed; half
the slope concurrent), the floor being the published minimum of a thin PMMA film in parabolic
flight (14.7 % at 6 cm/s, taken from `phys_quench_w0.md`) less the blow-off term at that speed.

A slab of 1 to 5 mm is warm through under its flame within 3 to 60 s (`0.3 d^2 / alpha_s`), so after
the first minute it behaves as a film that does not burn out: its flame can sit still, where a
film's flame must keep moving onto cold fuel.

### 1.7 Oxygen in the sealed glovebox, and burning to completion

The duct exhausts into a working volume of 250 L (BASS-II's own mixing model, **L\***) holding
10.3 mol of gas. Burning PMMA uses 1.92 g of oxygen per gram:

```text
X(t) = X_0 - 0.0058 * (grams of PMMA burned)
```

The burning rate at the stagnation point is `m'' = g ln(1 + B)`,
`B = (k_s sqrt(a_e) (X - x_s) - q_r - q_c) / (g L_v)`, `L_v = 1.6 MJ/kg`: 13 g m-2 s-1 for a 2 cm
sphere at 10 cm/s in air, about 5 mg/s over the burning face, 125 W, and 0.17 % O2 per minute. A
rod gives 0.03 % per minute. Whole samples: a 2 cm sphere is 5.0 g (2.9 % O2), a 1 cm sphere 0.62 g,
a 1/4 inch rod 2.2 g, a film 0.24 g, a slab 2 x 10 cm 2.4 g per mm of thickness (1.4 % O2 per mm).

So: a film burns to completion in 30 to 100 s whenever it holds. A 2 cm sphere needs
`rho_s R / m''`, about 25 minutes at 10 cm/s (as `R^1.5 U^-0.5`), and ends 3 % lower in oxygen: it
completes only if the crew lets it. A slab thicker than `(X_0 - 0.14) / 0.014` mm, 3.5 mm from
19 %, cannot complete: it brings the oxygen to the floor first.

### 1.8 Limits and orders of magnitude

| Limit | What the law says | Physical? |
|---|---|---|
| `U -> 0`, large body | `a_e -> 1.5 D_c / R^2`; `X_th` above 21 % for `R > 1 cm` within 5 min | yes while the duct has oxygen; then everything goes out |
| `U -> 0`, small body | heated rods and 1 cm spheres hold down to the floor | yes for about a minute (1.5); candles and droplets burn in still air |
| `t -> 0` | `X_th` unbounded | yes: a flame cannot hold on cold thick fuel without the igniter |
| `t` large | `X_th -> x_s + q_r / (k_s sqrt(a_e))` | yes; a preheated sample is the most flammable |
| `R` large, forced | `a_q` fixed, `U_q = 2 R a_q / 3` grows with size | yes: 1 g needed radii of 1 to 2 m to reach 2 /s |
| high oxygen | both limits passed at any speed above 0.3 cm/s | yes |
| `U` large | linear blow-off of the stagnation flame | the flame may hold on the shoulder or in the wake at high oxygen: not covered |
| pressure | supply as `sqrt(p)`: `X_th - x_s` as `p^-0.5` (stagnation), `p^-1` (edge) | scaling only; every burn here is at 1 atm |

Checks against published observations (not fits, except where marked):

| Published observation | Law |
|---|---|
| Thick PMMA, air, 1 g low stretch: quench at 1.5 to 3 /s (Olson and T'ien 2000) | 1.9 /s with their measured losses |
| 4 cm PMMA sphere, SoFIE: goes out at 20.8 % at 0.44 cm/s (1238 s) | 20.6 % with the measured conduction, 23.4 % with `b_c / sqrt(t)` |
| same: 14.2 % at 4.8 cm/s; 13.5 % at 11.5 cm/s; 15.0 % at 34 cm/s; 14.4 % at 50 cm/s | 13.6; 13.9; 14.8; 15.5 % (`x_b` and `s_b` lean on the middle three) |
| BASS-II rods: lowest burning test 13.6 %; near-limit flames at 0.5 to 1.4 cm/s (**L\***) | floor 13.4 %; at 13.6 % and 0.5 to 2 cm/s P = 0.4 to 0.6 |
| BASS: 15 mm sphere at 17 % burning below 1 cm/s, out when the flow is stopped (**L\***) | P = 0.75 at 0.8 cm/s after 5 min, 0.88 after 10 min; fan off is outside the law |
| BASS: 2 cm sphere burning at 17 % and 12 cm/s (**L\***) | P = 0.96 two minutes after ignition |
| BASS: thick acrylic, 1 cm wide, air, 9 cm/s: small flame for minutes (**L\***) | P = 1.00 |
| BASS-II film 0.1 mm: radiative quench as flow goes below 2 cm/s, near blow-off at 40 cm/s, oxygen not stated (**L\***) | in air: quench 0.8 cm/s (0.4 to 1.9); P = 0.73 of holding at 40 cm/s |
| BASS-II rods: heat release mostly below 100 W (**L\***) | 30 to 125 W |

### 1.9 The earlier kernel (`property_solids_v2.rs`), as a colleague's work

It is a careful finite-volume solver for conduction and first-order pyrolysis in a flat slab, with
mass and energy ledgers, refusal on any input outside its declared domain, and the two exact
heating limits as checks. As a forecaster of this experiment it stops before the question starts:

- It has no gas phase. `FlameStatus` has one value, `UnsupportedGasPhaseClosure`; `quench_time_s`
  and `luminous_front` are always `None`. Whether a flame exists is not something it can say.
- The quantities that decide the outcome (heat transfer coefficient, gas temperature, absorbed flux)
  are inputs the caller must supply as valid for a declared flow and oxygen; the file says no flow
  or gravity correction is made from that context. The dependence on flow and oxygen is the whole
  of BASS, and it sits outside the kernel.
- It requires a persistent inert matrix with fixed conductivity. PMMA has none: it regresses and
  leaves nothing. Rods and spheres are not flat slabs.
- What it resolves in detail, the temperature field inside the solid, enters the limit through one
  number, the surface heat flux `lambda_s dT / sqrt(pi alpha_s t)`. Its own exact thick limit
  (surface temperature under constant flux) is the dual of that closed form.

I keep its discipline (units, refusal outside the domain) and replace the solver by sections 1.1 to
1.6: the outcome is decided where the gas meets the surface.

## 2. Targets

No feasibility note for `bass1` existed when this was written (`docs/wsp/feedback/bass1_feasibility.md`
absent), so the targets are defined here from the cards. The experiments held a flow and oxygen
level, changed the flow in steps, and recorded whether and how the flame went out.

| Target | Kind | Unit | Applies to | What a forecast may know |
|---|---|---|---|---|
| `holds_low_flow` | probability | | rod tips, spheres, sphere sections | oxygen fraction at the step, duct speed of the step, nose radius, seconds from ignition to the end of the step |
| `holds_high_flow` | probability | | same | oxygen, speed, radius |
| `oxygen_limit` | positive | mole fraction | same | speed, radius, time since ignition |
| `quench_speed` | positive | m/s | same | oxygen, radius, time since ignition at the quench |
| `blowoff_speed` | positive | m/s | same | oxygen, radius |
| `flat_holds_low_flow` | probability | | films and slabs | oxygen, speed, direction of spread |
| `flat_holds_high_flow` | probability | | same | oxygen, speed, direction |
| `flat_quench_speed` | positive | m/s | same | oxygen, direction |

Definitions. *Holds*: a flame is still there at the end of a step held for at least 60 s (thick
samples cool slowly; a step shorter than that proves nothing), or the sample was consumed.
`holds_low_flow` is the probability of not being quenched by heat loss (it tends to its floor value
at high speed); `holds_high_flow` of not being blown off (it tends to the floor value at low
speed). They describe two different ways of going out, which the films show apart (a flame that
shrinks and turns blue against one that is swept downstream). **The law's probability that a flame
holds is the smaller of the two.** *Oxygen limit*: the oxygen fraction at which the flame goes out
when the flow is held and the oxygen falls. *Quench speed*: the duct speed below which the flame
goes out when the flow is lowered at constant oxygen. *Blow-off speed*: the duct speed at which the
stagnation-region flame opens and is swept downstream when the flow is raised.

Not targets of the forward, stated in section 1.7: the oxygen fall and whether a sample burns to
completion. Not covered at all: wax candles (see 3.4), wake flames behind a sphere or hemisphere,
nitrogen-jet suppression, Nomex and Ultem, fabric (that is `phys_quench_w0.md`).

## 3. The law

Units in this section: `U` in cm/s, `R` in cm, `t` in s, `X` mole fraction, 1 atm.

### 3.1 Rod tips and spheres

```text
a_e  = 1.5 (U + D_c / R) / R
X_th = x_s + (q_r + b_c / sqrt(t)) / (k_s sqrt(a_e))          sd: sqrt(0.010^2 + (0.33 (X_th - x_s))^2)
a_n  = 1.5 (U / R) (R / r0)^n
X_k  = x_b + s_b a_n                                          sd: sqrt(0.008^2 + (0.30 (X_k - x_b))^2)

oxygen limit       X_lim = max(X_th, X_k)
quench speed       U_q = (2R/3) [(q_r + b_c / sqrt(t)) / (k_s (X - x_s))]^2 - D_c / R      (X > x_b + 0.01)
                   "no quench while the fan turns" when this is below 0.3 cm/s
blow-off speed     U_b = (X - x_b) R^(1-n) r0^n / (1.5 s_b)
P(not quenched)    = Phi( ln(X / max(X_th, x_b)) / s ),  s = 0.27 on the thermal branch, 0.06 at the floor
P(not blown off)   = Phi( ln(X / X_k) / 0.06 )
P(holds)           = the smaller of the two
```

Spreads. The quench speed is the difference of two uncertain terms. Its logarithm has a standard
deviation of 0.75 when the forced part dominates (a factor of 2.1; from 25 % on `k_s`, 15 % on
`q_r`, 50 % on `b_c`, doubled by the square) and grows without bound as `U_q` approaches zero: there
the statement is "below any fan setting", not a number. The blow-off speed has 0.36 in the
logarithm (a factor of 1.4) well above the floor and 0.6 within 1 % O2 of it. The oxygen limit has
0.06 in the logarithm (0.8 % O2) on the floor and blow-off side and up to 0.27 (4 to 6 % O2) where
heat loss sets it.

### 3.2 Films and slabs

```text
X_th = x_s + c_dir A_e / U          c_dir = 1 opposed, 1.5 concurrent;  sd of the excess 45 %
X_k  = x_bf + s_bf[dir] U           sd: sqrt(0.010^2 + (0.44 s_bf U)^2)
quench speed    U_q = c_dir A_e / (X - x_s)             ln-sd 0.50
blow-off speed  U_b = (X - x_bf) / s_bf[dir]
```

`U` should be the speed relative to the flame edge; a film's own spread adds 0.02 cm/s divided by
its thickness in mm (0.2 cm/s for 0.1 mm), which I neglect. For a slab the formula holds once it is
warm through (60 s for 5 mm); for the thick BASS slabs add `b_c / sqrt(t)` to `q_r` as in 3.1.

### 3.3 In dimensionless form

With `Lambda = (q_r + q_c) / (k_s sqrt(a_e) (X - x_s))` (losses over what the flame can deliver),
`K = (x_b + s_b a_n) / X` (oxygen that chemistry needs over oxygen present),
`Fo_c = t / t_c` (how far the solid has warmed, `q_c / q_r = Fo_c^-0.5`) and
`Pe = U R / D_c` (forced over diffusive supply, `a_e = a_f (1 + 1/Pe)`):

```text
a flame holds on a solid when   Lambda < 1   and   K < 1
```

Another material changes `q_r` (its surface temperature and emissivity), `b_c` (its
`lambda_s (T_s - T_inf) / sqrt(pi alpha_s)`), `x_b` and `s_b`; another apparatus changes how `U`
maps to `a`.

### 3.4 Wax candles (judgement only, no forward target)

A candle flame is held beside and behind its wick, not at a stagnation point, and the wick is a
few millimetres across, so diffusion feeds it at any speed and the blow-off law of 3.1 does not
apply (it would forbid a candle in 1 g). I state only: a lit candle in the duct holds from 0.3 to
40 cm/s at 17 % oxygen and above (P = 0.9), does not hold below 15 % (P = 0.8 that it goes out),
and with the fan off dims and goes out within one to three minutes as the duct's litre of air is
used. Thick flat wax: as a PMMA slab with a lower surface temperature; no number offered.

## 4. Constants

Classes: **P** property table, **T** derived here, **L** published experiment on another apparatus,
**L\*** published statement about BASS or BASS-II themselves, **J** judgement. "sd" is one standard
deviation of my uncertainty.

| Constant | Value | sd | Class | Origin |
|---|---|---|---|---|
| `dh_O`, heat per kg of oxygen | 13.1 MJ/kg | 5 % | P | Huggett 1980 (memory) |
| Air at 300 K: `rho`, `mu`, `Sc` of oxygen | 1.177 kg/m3, 1.85e-5 Pa s, 0.73 | | P | |
| Stagnation transfer coefficient | 0.763 Sc^-0.6 = 0.92 | | T | boundary-layer theory (memory) |
| `k_s` | 62 kW m-2 s^0.5 | 25 % | T | 1.1: `0.92 sqrt(rho mu) * 1.107 dh_O`; low-Reynolds flow and gas radiation not in it |
| PMMA surface temperature, emissivity | 630 K, 0.85 | 20 K, 0.05 | L | Olson and T'ien 2000, measured at low stretch |
| `q_r` | 7.2 kW/m2 | 15 % | T | `0.85 * 5.67e-8 * (630^4 - 300^4)` |
| `x_s` | 0.030 | 0.010 | T | 1.2: 0.025 sensible heat of the layer, 0.005 fuel for a 1100 K flame |
| PMMA `lambda_s`, `alpha_s`, `rho_s` | 0.21 W/m/K, 1.2e-7 m2/s, 1190 kg/m3 | 10 % | P | values used by Olson and Ferkul 2015 |
| `b_c` | 110 kW m-2 s^0.5 | 50 % | T | 1.4; checked on three SoFIE values (L) |
| `t_c = (b_c / q_r)^2` | 230 s | factor 2.5 | T | |
| `D_c` | 0.88 cm2/s | factor 1.6 | T | 1.5: oxygen in nitrogen at 700 K; SoFIE's quench at 0.44 cm/s asks for 0.86 with the measured conduction (L) |
| `x_b`, floor, rods and spheres | 0.134 | 0.008 | L + L\* | Marcum et al. 2016 (1 g extrapolation 13.4 %); BASS-II lowest burning test 13.6 % and fit intercept 13.3 % (L\*); SoFIE sphere 13.5 % |
| `s_b` | 0.00035 s | 30 % | L | Marcum et al.: 16 % at 78 /s; SoFIE: 15.0 % at 25.7 /s, R = 2 cm |
| `r0`, `n` | 0.318 cm, 0.25 | n: 0.25 | L + J | Marcum et al. find 0.5 in 1 g; SoFIE fits 0 to 0.3 |
| `L_v`, PMMA | 1.6 MJ/kg | 15 % | P | only in the burning rate of 1.7 |
| `D_h` | 1.7 cm2/s | factor 1.6 | T | 1.6: air at 1100 K |
| `A_e = (q_r / k_s) sqrt(D_h)` | 0.151 cm/s | 45 % | T | 1.6 |
| `c_dir` concurrent | 1.5 | 0.4 | J | 1.6 |
| `x_bf`, floor, flat | 0.138 | 0.010 | L | thin PMMA film in parabolic flight, 14.7 % at 6 cm/s (Takahashi et al. 2014, via `phys_quench_w0.md`), less `0.0016 * 6` |
| `s_bf` opposed, concurrent | 0.0016, 0.0008 per cm/s | 44 % | T + J | taken from `phys_quench_w0.md` D5 (Arrhenius chord; thin-cellulose check) |
| Working volume | 250 L, 10.3 mol | 10 % | L\* | Olson and Ferkul 2015: mixing model of the glovebox |
| Oxygen per gram of PMMA | 1.92 g, 0.0058 mole fraction per g burned | 5 % | P / T | C5H8O2 + 6 O2 |

Derived, for orientation: quench stretch of a sample that has burned for 200 s, `a_q = 1.8 /s` in
air, 2.6 /s at 18 %, 4.0 /s at 15 %; after a long time 0.41, 0.59 and 0.93 /s.

## 5. Registered expectations

Written before any comparison. These are the outputs of the restated networks of section 7 (what
will run), 1 atm. Where the exact formulas of section 3 differ by more than rounding they are in
brackets. Intervals are 5 % to 95 % of the core log-normal; the wide component (one case in ten,
twice the width) is not in them.

**E1. Oxygen limit, %, median of `oxygen_limit`** (duct speed in cm/s across)

| Sample, time since ignition | 0.5 | 1 | 2 | 3 | 5 | 10 | 20 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| rod R = 0.318 cm, 300 s | 13.4 | 13.4 | 13.7 | 14.0 | 14.2 | 15.1 | 17.0 | 20.1 |
| rod R = 0.476 cm, 300 s | 13.8 | 13.5 | 13.6 | 13.8 | 14.1 | 14.7 | 16.0 | 18.4 |
| rod R = 0.635 cm, 300 s | 15.2 [13.4] | 13.8 | 13.5 | 13.7 | 13.9 | 14.4 | 15.4 | 17.6 |
| rod R = 0.635 cm, 1200 s | 13.6 | 13.4 | 13.5 | 13.7 | 13.9 | 14.4 | 15.4 | 17.6 |
| sphere R = 0.5 cm, 120 s | 16.0 [13.6] | 13.9 | 13.7 | 13.8 | 14.0 | 14.7 | 15.8 | 18.2 |
| sphere R = 0.75 cm, 120 s | 20.3 [18.1] | 17.1 | 14.4 | 14.1 | 14.1 | 14.3 | 15.2 | 17.1 |
| sphere R = 1.0 cm, 60 s | 27.8 | 23.3 | 19.6 | 17.9 | 16.0 [14.6] | 14.6 | 15.0 | 16.4 |
| sphere R = 1.0 cm, 120 s | 24.0 | 20.2 | 17.0 | 15.5 | 14.3 | 14.3 | 14.9 | 16.4 |
| sphere R = 1.0 cm, 600 s | 17.2 | 14.5 | 13.7 | 13.6 | 13.8 | 14.1 | 14.9 | 16.4 |

Width (ln-sd): 0.06 where the entry is below 15 % or on the rising right side; 0.10 to 0.28 where
heat loss sets it (the left entries of the spheres). Example: 2 cm sphere, 120 s, 1 cm/s: 20.2 %,
90 % interval 13.6 to 30 %. Where a bracket is lower than the entry, the restatement is conservative
at speeds well below `D_c / R`.

**E2. P(not quenched), `holds_low_flow`**

2 cm sphere (R = 1.0 cm), 120 s after ignition:

| X_O2 \ U (cm/s) | 0.3 | 0.5 | 1 | 2 | 3 | 5 | 10 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0.21 | 0.20 | 0.36 | 0.63 | 0.83 | 0.91 | 0.96 | 0.98 |
| 0.19 | 0.12 | 0.23 | 0.47 | 0.73 | 0.84 | 0.92 | 0.97 |
| 0.17 | 0.06 | 0.13 | 0.31 | 0.57 | 0.72 | 0.85 | 0.94 |
| 0.15 | 0.03 | 0.06 | 0.17 | 0.38 | 0.53 | 0.72 | 0.88 |
| 0.14 | 0.02 | 0.04 | 0.12 | 0.28 | 0.43 | 0.62 | 0.77 |
| 0.13 | 0.01 | 0.03 | 0.08 | 0.20 | 0.30 | 0.30 | 0.30 |

The same sphere 600 s after ignition:

| X_O2 \ U (cm/s) | 0.3 | 0.5 | 1 | 2 | 3 | 5 | 10 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0.21 | 0.67 | 0.82 | 0.93 | 0.98 | 0.99 | 0.99 | 1.00 |
| 0.19 | 0.52 | 0.71 | 0.88 | 0.96 | 0.98 | 0.99 | 1.00 |
| 0.17 | 0.35 | 0.55 | 0.79 | 0.92 | 0.95 | 0.98 | 0.99 |
| 0.15 | 0.20 | 0.36 | 0.62 | 0.83 | 0.90 | 0.95 | 0.96 |
| 0.14 | 0.14 | 0.27 | 0.52 | 0.76 | 0.77 | 0.77 | 0.77 |
| 0.13 | 0.09 | 0.18 | 0.30 | 0.30 | 0.30 | 0.30 | 0.30 |

1/4 inch rod (R = 0.318 cm), 300 s after ignition:

| X_O2 \ U (cm/s) | 0.3 | 0.5 | 1 | 2 | 5 | 10 |
|---|---:|---:|---:|---:|---:|---:|
| 0.17 to 0.22 | 0.94 to 0.99 | 0.97 to 0.99 | 0.99 | 1.00 | 1.00 | 1.00 |
| 0.16 | 0.91 | 0.96 | 0.99 | 0.99 | 0.99 | 0.99 |
| 0.15 | 0.87 | 0.94 | 0.96 | 0.96 | 0.96 | 0.96 |
| 0.145 | 0.84 | 0.90 | 0.90 | 0.90 | 0.90 | 0.90 |
| 0.14 | 0.77 | 0.77 | 0.77 | 0.77 | 0.77 | 0.77 |
| 0.135 | 0.55 | 0.55 | 0.55 | 0.55 | 0.55 | 0.55 |
| 0.13 | 0.30 | 0.30 | 0.30 | 0.30 | 0.30 | 0.30 |

Other bodies at 17 %: 1 cm sphere, 120 s: 0.46 / 0.65 / 0.85 / 0.94 at 0.3 / 0.5 / 1 / 2 cm/s.
1.5 cm sphere, 300 s: 0.39 / 0.59 / 0.81 / 0.93. 1/2 inch rod, 300 s: 0.54 / 0.72 / 0.89 / 0.96.

**E3. P(not blown off), `holds_high_flow`**

| X_O2 \ U (cm/s) | 5 | 10 | 15 | 20 | 30 | 40 | 55 |
|---|---:|---:|---:|---:|---:|---:|---:|
| rod R = 0.318: 0.21 | 1.00 | 1.00 | 1.00 | 1.00 | 0.98 | 0.77 | 0.14 |
| 0.19 | 1.00 | 1.00 | 0.99 | 0.96 | 0.71 | 0.17 | 0.01 |
| 0.17 | 0.99 | 0.96 | 0.84 | 0.51 | 0.10 | 0.01 | 0.00 |
| 0.16 | 0.96 | 0.82 | 0.49 | 0.16 | 0.02 | 0.00 | 0.00 |
| 0.15 | 0.81 | 0.43 | 0.14 | 0.03 | 0.00 | 0.00 | 0.00 |
| 0.14 | 0.39 | 0.10 | 0.02 | 0.00 | 0.00 | 0.00 | 0.00 |
| rod R = 0.635: 0.17 | 1.00 | 0.99 | 0.97 | 0.94 | 0.67 | 0.28 | 0.06 |
| 0.15 | 0.88 | 0.75 | 0.51 | 0.33 | 0.06 | 0.01 | 0.00 |
| sphere R = 1.0: 0.17 | 1.00 | 0.99 | 0.99 | 0.98 | 0.93 | 0.73 | 0.31 |
| 0.15 | 0.92 | 0.84 | 0.72 | 0.54 | 0.29 | 0.07 | 0.01 |

**E4. Quench speed, cm/s, median of `quench_speed`** with the 5 % to 95 % interval of its core

| Sphere, time since ignition | 21 % | 19 % | 17 % | 15 % |
|---|---|---|---|---|
| R = 1.0 cm, 60 s | 1.2 (0.3 to 4.8) | 1.8 (0.5 to 6.5) | 2.6 (0.8 to 9.0) | 3.7 (1.1 to 13) |
| R = 1.0 cm, 300 s | 0.11 (0.01 to 1.1) | 0.34 (0.05 to 2.3) | 0.72 (0.16 to 3.3) | 1.2 (0.3 to 4.8) |
| R = 1.0 cm, 900 s | below 0.3 (95 %: 0.2) | below 0.3 (0.4) | 0.09 (to 0.9) | 0.36 (0.05 to 2.4) |
| R = 0.75 cm, 60 s | 0.44 (0.06 to 3.0) | 0.87 (0.2 to 4.2) | 1.4 (0.35 to 5.9) | 2.4 (0.7 to 8.6) |
| R = 0.75 cm, 300 s | below 0.3 (0.2) | below 0.3 (0.4) | 0.10 (to 1.1) | 0.44 (0.06 to 3.1) |
| R = 0.5 cm, 60 s | below 0.3 (0.3) | below 0.3 (0.6) | 0.15 (to 1.6) | 0.65 (0.09 to 4.5) |
| R = 0.5 cm, 300 s or more | below 0.3 (0.0) | below 0.3 (0.0) | below 0.3 (0.1) | below 0.3 (0.3) |
| R = 2.0 cm, 300 s (not flown in BASS) | 1.4 (0.4 to 4.9) | 1.9 (0.6 to 6.5) | 2.6 (0.8 to 9.0) | 3.8 (1.1 to 13) |

Rods: R = 0.318 cm, below 0.3 cm/s at every oxygen level and time (95 %: 0.6 cm/s at 60 s and
14 %, 0.03 cm/s from 300 s). R = 0.635 cm: 0.8 / 1.6 / 2.1 cm/s at 17 / 15 / 14 % after 60 s; below
0.3 cm/s from 300 s (95 %: 1.2 cm/s at 15 %).

**E5. Blow-off speed, cm/s, median of `blowoff_speed`** (5 % to 95 %)

| Body | 14.5 % | 15 % | 16 % | 17 % | 19 % | 21 % |
|---|---|---|---|---|---|---|
| rod R = 0.318 cm | 7 (2 to 19) | 10 (4 to 25) | 16 (8 to 32) | 22 (12 to 40) | 35 (19 to 63) | 46 (25 to 83) |
| rod R = 0.476 cm | 9 (3 to 26) | 13 (5 to 33) | 22 (11 to 43) | 30 (17 to 55) | 47 (26 to 85) | 62 (34 to 113) |
| rod R = 0.635 cm | 11 (4 to 33) | 17 (7 to 42) | 27 (14 to 53) | 38 (21 to 68) | 58 (32 to 106) | 77 (43 to 140) |
| sphere R = 0.5 cm | 10 (3 to 27) | 14 (6 to 35) | 23 (12 to 44) | 31 (17 to 57) | 49 (27 to 88) | 65 (36 to 117) |
| sphere R = 1.0 cm | 16 (6 to 46) | 23 (9 to 58) | 38 (19 to 75) | 53 (29 to 95) | 82 (45 to 149) | 109 (60 to 197) |

**E6. Films and slabs**

Quench speed, median of `flat_quench_speed`, cm/s (5 % to 95 %): opposed 0.83 (0.37 to 1.9) at 21 %,
0.94 at 19 %, 1.08 at 17 %, 1.25 (0.55 to 2.8) at 15 %; concurrent 1.25, 1.41, 1.61, 1.87. Oxygen
limit, opposed (section 3.2): 33 % at 0.5 cm/s, 18 % at 1, 14.0 % at 1.5, 14.1 % at 2, 14.6 % at 5,
15.4 % at 10, 17.0 % at 20, 20.2 % at 40 cm/s.

`flat_holds_low_flow`, opposed:

| X_O2 \ U (cm/s) | 0.3 | 0.5 | 1 | 1.5 | 2 | 3 | 5 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0.21 | 0.02 | 0.13 | 0.66 | 0.90 | 0.96 | 0.99 | 1.00 |
| 0.19 | 0.01 | 0.09 | 0.56 | 0.85 | 0.94 | 0.99 | 1.00 |
| 0.17 | 0.01 | 0.05 | 0.43 | 0.78 | 0.91 | 0.98 | 0.98 |
| 0.15 | 0.00 | 0.03 | 0.30 | 0.66 | 0.84 | 0.84 | 0.84 |
| 0.14 | 0.00 | 0.02 | 0.24 | 0.57 | 0.57 | 0.57 | 0.57 |

Concurrent: the same numbers at 1.5 times the speed (0.30 at 1 cm/s and 0.85 at 2 cm/s in air).

`flat_holds_high_flow`, opposed:

| X_O2 \ U (cm/s) | 5 | 10 | 20 | 30 | 40 | 55 |
|---|---:|---:|---:|---:|---:|---:|
| 0.21 | 1.00 | 1.00 | 0.99 | 0.92 | 0.73 | 0.21 |
| 0.19 | 1.00 | 0.99 | 0.92 | 0.61 | 0.27 | 0.04 |
| 0.17 | 0.96 | 0.90 | 0.55 | 0.14 | 0.04 | 0.00 |
| 0.15 | 0.68 | 0.42 | 0.09 | 0.01 | 0.00 | 0.00 |

Concurrent: the same numbers at twice the speed.

**E7. Sealed volume.** Oxygen falls by 0.0058 per gram of PMMA burned: 0.1 to 0.2 % per minute for
a 2 cm sphere at 10 cm/s, 0.03 % per minute for a rod, 0.14 % for a whole film. No slab thicker than
3.5 mm started at 19 % burns to completion; no sphere of 2 cm does in less than 15 minutes.

## 6. What would show the law is wrong

On steps held at least 60 s, oxygen known to 0.3 %, speed known to 0.3 cm/s.

1. **There is no clock.** The law says the lowest speed at which a thick body holds falls about
   tenfold between one and five minutes after ignition (2 cm sphere in air: 1.2 then 0.11 cm/s).
   It is wrong if a 2 cm sphere that has burned five minutes or more at 17 % oxygen or above goes
   out, other than by blow-off, at a steady speed above 3.5 cm/s (the 95 % point at 17 % is 3.3);
   or if a sphere lit less than a minute before holds for a minute at 0.3 cm/s or less at 19 % or
   below. The first would cut `k_s` or raise `q_r`; the second would cut `b_c`, the conduction term,
   by more than its 50 %.
2. **The floor is elsewhere, or depends on the body.** The law says no steady flame on any PMMA
   shape at 12.5 % or below (P at most 0.13), and that a heated rod at 15 % holds at 0.5 to 2 cm/s
   (P 0.9 or more), with the same floor for all three rod sizes once they have burned 20 minutes.
   A flame lasting two minutes at 12.5 %, or heated rods failing at 15.5 %, moves `x_b`. A floor
   that is 1 % O2 higher for the 1/2 inch rod than for the 1/4 inch rod after long burning (the
   flight paper suggests it) means conduction along a rod never dies away as `1 / sqrt(t)`: a
   steady term is missing.
3. **Blow-off does not scale as `(X - x_b) R^0.75`.** The law says a 1/4 inch rod blows off at
   16 cm/s at 16 % (8 to 32) and a 1/2 inch rod at 1.7 times that. It is wrong if the blow-off
   speed at fixed oxygen does not rise with radius, if a 1/4 inch rod at 16 % holds its stagnation
   flame at 40 cm/s, or if one at 17 % loses it below 8 cm/s. The first changes `n`, the others
   `s_b`.
4. **Small bodies do quench.** The law says a 1 cm sphere or a 1/4 inch rod, burning three minutes
   or more at 16 % or above, holds at any speed from 0.3 cm/s. If such a flame goes out at a steady
   1 cm/s, diffusion does not feed it as `D_c / R` and the term should be cut.

For films: a quench speed in air above 2.5 cm/s (95 % point 1.9), or one that does not rise as
oxygen falls, or that doubles from 0.1 to 0.4 mm. The published "below 2 cm/s" (oxygen not stated)
already sits at the upper edge of the band; I left the coefficient where the derivation put it.

Calibration is checked apart from these: of the steps given a probability between 0.3 and 0.7,
about half should have held.

## 7. Interface for the law forward (`docs/wsp/research/law_forward.md`)

One slug, `bass1`, five features, eight targets (the forward's maximum). The forward takes
`ln`, `identity` or `reciprocal` of each input and a network of at most eight hinge units, with no
product of features and no nesting. The law of section 3 is restated for it with four
approximations, each checked on the grid of section 5:

- `ln(X - 0.03) = ln 0.15 + 1.2 ln(X / 0.18)`, within 0.015 from 13 to 23 % oxygen;
- `1 + b_c / (q_r sqrt(t)) = 2.08 (t / 200 s)^-0.25`, within 12 % from 30 to 1500 s;
- `U + D_c / R = 1.96 U^0.6 (D_c / R)^0.4`, within 20 % for `U` between 0.4 and 5 times `D_c / R`,
  low by up to 40 % outside (this makes `X_th` too high at the far ends, where it rarely decides);
  used only in `holds_low_flow` and `oxygen_limit`; `quench_speed` keeps the exact difference;
- logistic for the normal distribution function, slope 1.7 / sd; "the smaller of two" is one hinge
  unit where both are linear, and two separate targets where they are not.

Candidate as a revision of the slug's neutral root: what is not written below stays zero.

### 7.1 Features (declaration order)

| # | `name` | SI unit | transform | centre | scale | domain (raw) | Meaning |
|---|---|---|---|---|---|---|---|
| 0 | `x_o2` | 1 | ln | 0.18 | 1 | 0.12 to 0.25 | oxygen mole fraction of the gas entering the duct during the step (if only the pre-burn reading exists: that value less 0.0058 per gram already burned, or the pre-burn value itself) |
| 1 | `flow_speed` | m/s | ln | 0.01 | 1 | 0.003 to 0.6 | duct speed of the step (anemometer, or the fan calibration) |
| 2 | `nose_radius` | m | ln | 0.01 | 1 | 0.002 to 0.03 | radius of the rod, sphere or spherical section as installed; for a flat sample supply 0.01 (weight zero in the flat targets) |
| 3 | `heat_time` | s | ln | 200 | 1 | 20 to 4000 | time from ignition of this burn to the end of the step (for quench and blow-off speeds: to the event). If it is not recorded supply 200 |
| 4 | `is_concurrent` | 1 | identity | 0 | 1 | 0 to 1 | flat samples: 1 if the flame spreads with the flow, 0 against it; 0 for rods and spheres |

`z0 = ln(X / 0.18)`, `z1 = ln(U / 1 cm/s)`, `z2 = ln(R / 1 cm)`, `z3 = ln(t / 200 s)`, `z4` the
indicator. A step below 0.3 cm/s (fan off) is outside the domain and refused; for it the statement
is section 1.5: a flame on PMMA goes out within one to two minutes. A sample re-lit while still warm
is more flammable than its `heat_time` says.

### 7.2 Targets (declaration)

| `name` | kind | unit | shared quantity | Description |
|---|---|---|---|---|
| `holds_low_flow` | binary | | | rod tip or sphere: flame not quenched by heat loss at the end of the step |
| `holds_high_flow` | binary | | | rod tip or sphere: stagnation flame not blown off |
| `oxygen_limit` | positive | 1 | `mole_fraction` | rod tip or sphere: oxygen fraction at which the flame goes out at this speed and time |
| `quench_speed` | positive | m/s | `flow_velocity` | rod tip or sphere: duct speed below which the flame goes out |
| `blowoff_speed` | positive | m/s | `flow_velocity` | rod tip or sphere: duct speed at which the stagnation flame blows off |
| `flat_holds_low_flow` | binary | | | film or slab: flame not quenched |
| `flat_holds_high_flow` | binary | | | film or slab: flame not blown off |
| `flat_quench_speed` | positive | m/s | `flow_velocity` | film or slab: duct speed below which the flame goes out |

A probability that the flame holds is the smaller of the low-flow and the high-flow target of its
geometry (taken by whoever scores; the forward gives each). A quench speed forecast below 0.003 m/s
means "holds down to the lowest fan setting"; such a case is better scored on `holds_low_flow`.

### 7.3 Tensors

Those of section 3 of the forward's document for 5 features:

| Tensor | Shape | Bounds | Unit | Meaning |
|---|---|---|---|---|
| `x.<f>.centre`, `x.<f>.scale` | scalar | as the forward | unit of the feature; 1 | centre and scale of the transform |
| `x.<f>.lo`, `x.<f>.hi` | scalar | as the forward | unit of the feature | declared domain; outside it the forward refuses |
| `<t>.logit.b0` (4 binary targets) | scalar | -40 to 40 | logit | value at the centre: 18 % O2, 1 cm/s, 1 cm, 200 s |
| `<t>.median.b0` (4 positive targets) | scalar | -80 to 80 | ln(m/s), ln(mole fraction) | ln of the median at the centre |
| `<t>.spread.b0` | scalar | 0.001 to 10 | ln | ln-sd of the core at the centre |
| `<P>.skip` | [5] | as the forward | per unit of z | linear weights (power-law exponents) |
| `<P>.W1`, `<P>.b1`, `<P>.w2` | [8, 5], [8], [8] | as the forward | | hinge units: where a limit takes over, and chords of a curved term |
| `<t>.tail_weight`, `<t>.tail_width` | scalar | 0 to 1; 1 to 100 | | share and relative width of the wide component |

Every value below is inside the forward's bounds. `oxygen_limit` names the shared quantity
`mole_fraction` on the assumption that its SI unit is `1`; otherwise it is lane-local.

How the numbers arise (cm, s):

```text
ln X_th  = -1.750 - 0.25 z1 + 0.583 z2 - 0.208 z3          (3.1 with the three approximations)
ln a_n   =  0.692 + z1 - 0.75 z2
ln(X_k / x_b) = 0.039 relu(ln a_n - 1.609) + 0.055 relu(ln a_n - 3.219)
              + 0.107 relu(ln a_n - 4.094) + 0.138 relu(ln a_n - 5.011)      (chords of ln(1 + s_b a_n / x_b))
l_f  = 28 ln(X / 0.134)            floor, sd 0.06 in ln
l_th = 6.3 ln(X / X_th)            thermal limit, sd 0.27 in ln
ln U_q* = 0.538 - 2.4 z0 + z2 - 0.5 z3                      (2R a_q / 3, cm/s)
v    = ln(U_q* R / D_c) = 0.666 - 2.4 z0 + 2 z2 - 0.5 z3
ln U_q  = ln U_q* + ln(1 - exp(-v)),   the last term as four chords
ln U_b  = 7.266 + 0.75 z2 + ln(X - 0.134),   the last term as a line and five chords in z0
flat: ln X_th = -1.711 - 0.833 z1 + 0.338 z4;   X_k = 0.138 + 0.0016 U exp(-0.693 z4)
```

### 7.4 Literal assignments

```text
# bass1.wsp: W0, zero-shot. Author claude-fable-5-2. Source: docs/wsp/research/phys_bass1_w0.md
# features z = (z_x, z_u, z_r, z_t, concurrent), columns 0..4
x.x_o2.centre = 0.18
x.x_o2.scale = 1.0
x.x_o2.lo = 0.12
x.x_o2.hi = 0.25
x.flow_speed.centre = 0.01
x.flow_speed.scale = 1.0
x.flow_speed.lo = 0.003
x.flow_speed.hi = 0.6
x.nose_radius.centre = 0.01
x.nose_radius.scale = 1.0
x.nose_radius.lo = 0.002
x.nose_radius.hi = 0.03
x.heat_time.centre = 200.0
x.heat_time.scale = 1.0
x.heat_time.lo = 20.0
x.heat_time.hi = 4000.0
x.is_concurrent.centre = 0.0
x.is_concurrent.scale = 1.0
x.is_concurrent.lo = 0.0
x.is_concurrent.hi = 1.0

# --- rod tips and spheres: not quenched. logit = min(l_f, l_th) = l_f - relu(l_f - l_th).
# l_f = 28 ln(X / 0.134) = 8.26 + 28 z_x: floor 13.4 % O2, sd 0.06 in ln (0.8 % O2).
holds_low_flow.logit.b0 = 8.26
holds_low_flow.logit.skip[0:5] = [28.0, 0.0, 0.0, 0.0, 0.0]
# unit 0: l_f - l_th, l_th = 6.3 (ln X - ln X_th), ln X_th = -1.750 - 0.25 z_u + 0.583 z_r - 0.208 z_t
# (surface radiation 7.2 kW/m2 plus conduction 110 / sqrt(t) against a supply 62 sqrt(a_e) (X - 0.03)).
holds_low_flow.logit.W1[0, 0:5] = [21.7, -1.58, 3.67, -1.31, 0.0]
holds_low_flow.logit.b1[0] = 8.04
holds_low_flow.logit.w2[0] = -1.0

# --- rod tips and spheres: not blown off. logit = 28 (ln(X / 0.134) - ln(X_k / 0.134)),
# X_k = 0.134 + 0.00035 a_n, ln a_n = 0.692 + z_u - 0.75 z_r; chords at a_n = 5, 25, 60, 150 /s.
holds_high_flow.logit.b0 = 8.26
holds_high_flow.logit.skip[0:5] = [28.0, 0.0, 0.0, 0.0, 0.0]
holds_high_flow.logit.W1[0, 1] = 1.0
holds_high_flow.logit.W1[0, 2] = -0.75
holds_high_flow.logit.b1[0] = -0.917
holds_high_flow.logit.w2[0] = -1.09
holds_high_flow.logit.W1[1, 1] = 1.0
holds_high_flow.logit.W1[1, 2] = -0.75
holds_high_flow.logit.b1[1] = -2.527
holds_high_flow.logit.w2[1] = -1.54
holds_high_flow.logit.W1[2, 1] = 1.0
holds_high_flow.logit.W1[2, 2] = -0.75
holds_high_flow.logit.b1[2] = -3.402
holds_high_flow.logit.w2[2] = -3.00
holds_high_flow.logit.W1[3, 1] = 1.0
holds_high_flow.logit.W1[3, 2] = -0.75
holds_high_flow.logit.b1[3] = -4.319
holds_high_flow.logit.w2[3] = -3.86

# --- oxygen limit = floor, plus the excess of the thermal limit over it, plus the blow-off rise.
# b0 = ln 0.134.
oxygen_limit.median.b0 = -2.010
oxygen_limit.median.skip[0:5] = [0.0, 0.0, 0.0, 0.0, 0.0]
# units 0, 1: relu(ln X_th - ln 0.134), rounded at the corner (0.15 of it starts 0.3 earlier).
oxygen_limit.median.W1[0, 0:5] = [0.0, -0.25, 0.583, -0.208, 0.0]
oxygen_limit.median.b1[0] = 0.260
oxygen_limit.median.w2[0] = 0.85
oxygen_limit.median.W1[1, 0:5] = [0.0, -0.25, 0.583, -0.208, 0.0]
oxygen_limit.median.b1[1] = 0.560
oxygen_limit.median.w2[1] = 0.15
# units 2..5: ln(1 + 0.00035 a_n / 0.134).
oxygen_limit.median.W1[2, 1] = 1.0
oxygen_limit.median.W1[2, 2] = -0.75
oxygen_limit.median.b1[2] = -0.917
oxygen_limit.median.w2[2] = 0.039
oxygen_limit.median.W1[3, 1] = 1.0
oxygen_limit.median.W1[3, 2] = -0.75
oxygen_limit.median.b1[3] = -2.527
oxygen_limit.median.w2[3] = 0.055
oxygen_limit.median.W1[4, 1] = 1.0
oxygen_limit.median.W1[4, 2] = -0.75
oxygen_limit.median.b1[4] = -3.402
oxygen_limit.median.w2[4] = 0.107
oxygen_limit.median.W1[5, 1] = 1.0
oxygen_limit.median.W1[5, 2] = -0.75
oxygen_limit.median.b1[5] = -4.319
oxygen_limit.median.w2[5] = 0.138
# spread: 0.06 on the floor; rises to 0.28 once the thermal limit is 0.44 (in ln) above the floor;
# plus 0.02 per e-fold of stretch beyond 25 /s (the slope s_b is known to 30 %).
oxygen_limit.spread.b0 = 0.06
oxygen_limit.spread.skip[0:5] = [0.0, 0.0, 0.0, 0.0, 0.0]
oxygen_limit.spread.W1[0, 0:5] = [0.0, -0.25, 0.583, -0.208, 0.0]
oxygen_limit.spread.b1[0] = 0.260
oxygen_limit.spread.w2[0] = 0.5
oxygen_limit.spread.W1[1, 0:5] = [0.0, -0.25, 0.583, -0.208, 0.0]
oxygen_limit.spread.b1[1] = -0.180
oxygen_limit.spread.w2[1] = -0.5
oxygen_limit.spread.W1[2, 1] = 1.0
oxygen_limit.spread.W1[2, 2] = -0.75
oxygen_limit.spread.b1[2] = -2.527
oxygen_limit.spread.w2[2] = 0.02
oxygen_limit.tail_weight = 0.1
oxygen_limit.tail_width = 2.0

# --- quench speed = 2 R a_q / 3 - D_c / R. b0 = ln(1.71 cm/s) - ln(100): R = 1 cm, 18 % O2, 200 s.
# skip: a_q ~ (X - 0.03)^-2 -> -2.4 per e-fold of X; R; (1 + 15.3 / sqrt(t))^2 -> -0.5 per e-fold of t.
quench_speed.median.b0 = -4.067
quench_speed.median.skip[0:5] = [-2.4, 0.0, 1.0, -0.5, 0.0]
# units 0..3: ln(1 - exp(-v)), v = 0.666 - 2.4 z_x + 2 z_r - 0.5 z_t = ln(forced part / diffusive part);
# chords at v = 3.0, 1.2, 0.6, 0.3. Below v = 0 the median runs to zero: no quench while the fan turns.
quench_speed.median.W1[0, 0:5] = [2.4, 0.0, -2.0, 0.5, 0.0]
quench_speed.median.b1[0] = 2.334
quench_speed.median.w2[0] = -0.20
quench_speed.median.W1[1, 0:5] = [2.4, 0.0, -2.0, 0.5, 0.0]
quench_speed.median.b1[1] = 0.534
quench_speed.median.w2[1] = -0.53
quench_speed.median.W1[2, 0:5] = [2.4, 0.0, -2.0, 0.5, 0.0]
quench_speed.median.b1[2] = -0.066
quench_speed.median.w2[2] = -1.12
quench_speed.median.W1[3, 0:5] = [2.4, 0.0, -2.0, 0.5, 0.0]
quench_speed.median.b1[3] = -0.366
quench_speed.median.w2[3] = -2.15
# spread: 0.75 (k_s 25 %, q_r 15 %, b_c 50 %, squared); wider as the two parts cancel.
quench_speed.spread.b0 = 0.75
quench_speed.spread.skip[0:5] = [0.0, 0.0, 0.0, 0.0, 0.0]
quench_speed.spread.W1[0, 0:5] = [2.4, 0.0, -2.0, 0.5, 0.0]
quench_speed.spread.b1[0] = 0.534
quench_speed.spread.w2[0] = 0.3
quench_speed.spread.W1[1, 0:5] = [2.4, 0.0, -2.0, 0.5, 0.0]
quench_speed.spread.b1[1] = -0.066
quench_speed.spread.w2[1] = 0.6
quench_speed.tail_weight = 0.1
quench_speed.tail_width = 2.0

# --- blow-off speed = (X - 0.134) R^0.75 r0^0.25 / (1.5 * 0.00035). b0: R = 1 cm, X = 0.18 on the
# line through X = 0.21 with slope 2.8; units 0..4 bend it down to ln(X - 0.134) (knots at 19, 18,
# 17, 16, 15 % O2).
blowoff_speed.median.b0 = -0.347
blowoff_speed.median.skip[0:5] = [2.8, 0.0, 0.75, 0.0, 0.0]
blowoff_speed.median.W1[0, 0] = -1.0
blowoff_speed.median.b1[0] = 0.054
blowoff_speed.median.w2[0] = -0.85
blowoff_speed.median.W1[1, 0] = -1.0
blowoff_speed.median.b1[1] = 0.0
blowoff_speed.median.w2[1] = -0.65
blowoff_speed.median.W1[2, 0] = -1.0
blowoff_speed.median.b1[2] = -0.057
blowoff_speed.median.w2[2] = -1.06
blowoff_speed.median.W1[3, 0] = -1.0
blowoff_speed.median.b1[3] = -0.118
blowoff_speed.median.w2[3] = -2.22
blowoff_speed.median.W1[4, 0] = -1.0
blowoff_speed.median.b1[4] = -0.182
blowoff_speed.median.w2[4] = -3.4
# spread: 0.36 (s_b 30 %, radius exponent); grows below 17 % as the floor's 0.8 % O2 comes to matter.
blowoff_speed.spread.b0 = 0.36
blowoff_speed.spread.skip[0:5] = [0.0, 0.0, 0.0, 0.0, 0.0]
blowoff_speed.spread.W1[0, 0] = -1.0
blowoff_speed.spread.b1[0] = -0.057
blowoff_speed.spread.w2[0] = 0.8
blowoff_speed.spread.W1[1, 0] = -1.0
blowoff_speed.spread.b1[1] = -0.118
blowoff_speed.spread.w2[1] = 1.5
blowoff_speed.tail_weight = 0.1
blowoff_speed.tail_width = 2.0

# --- films and slabs: not quenched. l_f = 20 ln(X / 0.138) (sd 0.085 in ln);
# l_th = 4.5 (ln X - ln X_th), ln X_th = -1.711 - 0.833 z_u + 0.338 concurrent (sd 0.38 in ln).
flat_holds_low_flow.logit.b0 = 5.31
flat_holds_low_flow.logit.skip[0:5] = [20.0, 0.0, 0.0, 0.0, 0.0]
flat_holds_low_flow.logit.W1[0, 0:5] = [15.5, -3.75, 0.0, 0.0, 1.52]
flat_holds_low_flow.logit.b1[0] = 5.33
flat_holds_low_flow.logit.w2[0] = -1.0

# --- films and slabs: not blown off. X_k = 0.138 + 0.0016 U (cm/s), half the slope concurrent;
# chords of ln(X_k / 0.138) at U = 1, 5, 10, 20, 40 cm/s.
flat_holds_high_flow.logit.b0 = 5.31
flat_holds_high_flow.logit.skip[0:5] = [20.0, 0.0, 0.0, 0.0, 0.0]
flat_holds_high_flow.logit.W1[0, 1] = 1.0
flat_holds_high_flow.logit.W1[0, 4] = -0.693
flat_holds_high_flow.logit.b1[0] = 0.0
flat_holds_high_flow.logit.w2[0] = -0.56
flat_holds_high_flow.logit.W1[1, 1] = 1.0
flat_holds_high_flow.logit.W1[1, 4] = -0.693
flat_holds_high_flow.logit.b1[1] = -1.609
flat_holds_high_flow.logit.w2[1] = -0.98
flat_holds_high_flow.logit.W1[2, 1] = 1.0
flat_holds_high_flow.logit.W1[2, 4] = -0.693
flat_holds_high_flow.logit.b1[2] = -2.303
flat_holds_high_flow.logit.w2[2] = -1.32
flat_holds_high_flow.logit.W1[3, 1] = 1.0
flat_holds_high_flow.logit.W1[3, 4] = -0.693
flat_holds_high_flow.logit.b1[3] = -2.996
flat_holds_high_flow.logit.w2[3] = -2.10
flat_holds_high_flow.logit.W1[4, 1] = 1.0
flat_holds_high_flow.logit.W1[4, 4] = -0.693
flat_holds_high_flow.logit.b1[4] = -3.689
flat_holds_high_flow.logit.w2[4] = -2.30

# --- films and slabs: quench speed = c_dir * 0.151 cm/s / (X - 0.03): 1.00 cm/s at 18 % opposed.
flat_quench_speed.median.b0 = -4.601
flat_quench_speed.median.skip[0:5] = [-1.2, 0.0, 0.0, 0.0, 0.405]
flat_quench_speed.spread.b0 = 0.50
flat_quench_speed.spread.skip[0:5] = [0.0, 0.0, 0.0, 0.0, 0.0]
flat_quench_speed.tail_weight = 0.15
flat_quench_speed.tail_width = 2.5
```

### 7.5 What the restatement leaves out

The forward cannot hold these; they stay in sections 1 and 3 as formulas: the fall of oxygen during
a burn and burning to completion (1.7); the igniter time `t_min` (1.4); a flow history (the forward
sees one step; `heat_time` carries what the earlier steps did to the solid, not what they did to
the flame); the conduction term of a thick flat slab (3.2); pressure (every burn is at cabin
pressure; a declared pressure feature would refuse the records that lack it); candles (3.4).

## 8. Sources and limits

### 8.1 Sources

Read in this session (text of the public copy):

| Source | Apparatus | Taken |
|---|---|---|
| S. L. Olson, P. V. Ferkul, "Microgravity flammability of PMMA rods in concurrent flow", 9th US National Combustion Meeting, 2015 (NTRS 20150008961) | **BASS-II** (these burns) | duct 7.6 x 7.6 x 17 cm; rods of 1/4, 3/8, 1/2 inch; stretch defined as 3U/2R; lowest burning test 13.6 %, fit intercept 13.3 %; near-limit flames at forced stretch 2.4 to 3.5 /s; blow-off tests at 16.1 % and below; boundary linear on the blow-off side; solid response time `0.3 R^2 / alpha`; PMMA properties; working-volume mixing model; quench burns up to 28 min. The figure of the boundary is an image and was not read |
| J. W. Marcum, S. L. Olson, P. V. Ferkul, "Correlation of normal gravity mixed convection blowoff limits with microgravity forced flow blowoff limits", CSSCI 2016 (NTRS 20160012691) | 1 g flow tube, PMMA rods of five radii | linear blow-off line, intercept 13.4 %; buoyant stretch 78 /s for R = 0.318 cm; blow-off stretch as `R^-0.5` |
| P. V. Ferkul et al., "Thickness and fuel preheating effects on material flammability in microgravity from the BASS experiment", ASGSR 2013 (NTRS 20140011099) | **BASS** (these burns) | samples flown (17 thin flat, 12 spheres, 4 thick flat, 8 candles; 41 samples, 115 burns); statements about spheres at 17 %, the thick flat sample in air, flames dim and stable below 1 cm/s, the solid storing heat |
| P. V. Ferkul et al., "Combustion of solids in microgravity: results from the BASS-II experiment", ASGSR 2014 poster (NTRS 20160000593) | **BASS-II** | films 0.1 to 0.4 mm (22 tests), slabs 1 to 5 mm (20 tests, opposed, 18 to 20 %), 24 rods, a spherical section (2 tests); film quench below 2 cm/s and near blow-off at 40 cm/s; rods steady at 13.6 % against 16 % in 1 g |
| C. Li, J. S. T'ien, P. V. Ferkul, S. L. Olson, M. C. Johnston, "Extinction of solid diffusion flame in microgravity: details of quenching and blowoff processes", 2024 (NTRS 20240007628) | SoFIE-GEL, 4 cm PMMA sphere, 13.3 cm duct | Table 1: five extinction points with speed, oxygen and measured conduction into the solid. Used to check `k_s`, `b_c`, `D_c` and to set the range of `n`; `x_b` and `s_b` lean on it |
| S. L. Olson, J. S. T'ien, "Buoyant low stretch diffusion flames beneath cylindrical PMMA samples" (NTRS 20000004509; Combust. Flame 121, 2000) | 1 g, PMMA cylinders of large radius | surface 630 K, emissivity 0.85, radiative loss 7.7 kW/m2, conduction 5 to 8.4 kW/m2, uniform flame down to 3 /s, quench 1.5 to 2 /s, flame near 1100 K at the limit, 90 % of the surface heat lost |
| S. Bhattacharjee, C. Paolini, K. Wakai, S. Takahashi, "Extinction criteria for opposed-flow flame spread in a microgravity environment", NASA/CP-2003-212376 | theory | radiative extinction speed independent of thickness |
| `docs/wsp/research/phys_quench_w0.md` (colleague) | | the film minimum of Takahashi et al. 2014 and the blow-off slope 0.0016 per cm/s |

Known only from an abstract or a search summary: Olson and Ferkul, Combust. Flame 180 (2017) 217
(the final rod paper); Marcum, Olson, Ferkul, Proc. Combust. Inst. 37 (2019); Bhattacharjee, Simsek,
Olson, Ferkul 2016 (critical speed of radiative extinction in BASS-II films; I have only the card's
sentence); James, Li, Ferkul, Olson, Johnston, Combust. Flame 2025 (SoFIE sphere boundaries, 0.15
to 1.1 atm); Olson et al., Fire Safety J. 150 (2024) (rods in partial gravity).

From memory, not re-read: T'ien 1986; Foutch and T'ien 1987; the stagnation-point transfer
coefficient; Huggett 1980; the diffusivity of oxygen and its temperature exponent; air properties.

### 8.2 Not found or not checked

- The radius of the BASS-II spherical section, the thickness of the BASS thick flat samples, the wax
  samples and candle wicks, the igniter on-times.
- The rod flammability boundary itself (points of quench and blow-off against oxygen and stretch).
  I have its description, three near-limit stretch rates and the two intercepts.
- Numbers for the film quench speed against oxygen. Mine is derived; the one published number
  ("below 2 cm/s") is at the upper edge of my band.
- Whether the oxygen recorded for a burn is the reading before or after it. For a sphere at high
  flow the difference is 0.5 to 1 % O2, as large as the width of the floor.
- How well the duct speed is known below 1 cm/s (the anemometer takes a minute to respond; flow
  restrictors were used). Most of the quench physics of thick PMMA lies below 1 cm/s.
- The floor from first principles. A one-step flame gives 10 to 11 %; the measured 13.4 to 13.6 %
  is typed.
- Low-Reynolds corrections to the stagnation flow (the layer is as thick as the body at these
  speeds), gas radiation, soot. All inside the 25 % on `k_s`.
- Flames held on the shoulder or in the wake after the stagnation region opens. `blowoff_speed`
  and `holds_high_flow` mean the stagnation flame; at high oxygen the sample may go on burning.

### 8.3 Exposure

- **Repository.** `runs/dev/gen7/BRIEF_PHYSICS.md`; `AGENTS.md`; `docs/wsp/PROTOCOL.md` sections 1 to
  5 and 12.1 to 12.5; the cards `PSI-26.json` and `PSI-25.json` (the latter carries one aggregate
  score of the fabric model, not of these samples); the PSI-25 and PSI-26 paragraphs of
  `docs/wsp/atlas/INVESTIGATIONS.md`; `crates/ignis-wsp/src/property_solids_v2.rs`; two lines of
  `docs/wsp/dossier/ASTRA_COVERAGE_19.md`; `docs/wsp/research/phys_quench_w0.md`;
  `docs/wsp/research/law_forward.md`; section 7 of `docs/wsp/research/phys_spice_w0.md` (format
  only); the first 120 lines of `configs/models/acme/wsp3/normal/d1-fable/edits.wsp` (syntax);
  `configs/models/law/bass1/decl.json` (placeholder). Of `docs/wsp/feedback/bass1_psi26_tracks.md`
  only the section headings, to see whether it was the feasibility note; it reports outcomes of
  fabric burns and was not opened. Nothing under `runs/`, `data/`, `holdout/`.
- **Published summaries of the same burns.** The four BASS and BASS-II documents above describe
  burns this law will be run on. Their statements set two numbers, `x_b` (with two independent
  sources agreeing) and the working volume, and are otherwise used as the checks of section 1.8. I
  read them before settling the derivations and cannot exclude that they steered choices such as
  the size of the diffusion term for rods. This document is blind to the project's files and not
  blind to the literature on its experiment.
- **Pretraining.** The author's training may include these papers; that cannot be measured.

## 9. How the law, as typed, answers `sustain`

Added on 15 November at the data lane's question, from this document and physics alone. No number
of section 7.4 is changed; no outcome was seen. `sustain` is: the sample ignited and kept burning
after the igniter was switched off, yes or no, from inputs known before ignition.

**What the typed law can say.** It has no igniter. Its probability is that of a flame which exists
and still exists at the end of a first step held 60 s. With the igniter off the solid goes on
warming, so at fixed flow and oxygen the thermal limit only falls: the moment just after
switch-off is the hardest one, and a flame that passes it stays. The typed answer is therefore

```text
P(sustain) = the smaller of holds_low_flow and holds_high_flow            rod tips, spheres, sections
           = the smaller of flat_holds_low_flow and flat_holds_high_flow  films and slabs
```

**Inputs.**

| Feature | Value for `sustain` |
|---|---|
| `x_o2` | the initial oxygen reading of the burn |
| `flow_speed` | the duct speed at ignition (the speed at which the igniter is switched off; a later change of flow is another step, not this target) |
| `nose_radius` | diameter / 2 as installed; 0.01 for a flat sample |
| `heat_time` | igniter on-time + 60 s if the on-time is among the inputs known before ignition (a planned duration); otherwise 200, the convention already typed in 7.1 and the one used for `quench_u` |
| `is_concurrent` | as in 7.1 |

**Refused, or not answered.**

- Duct speed at ignition below 0.3 cm/s (fan off): outside the domain. The text says such a flame
  goes out within one to two minutes; no probability for a 60 s criterion is typed.
- Oxygen outside 12 to 25 %, radius outside 2 to 30 mm, `heat_time` below 20 s.
- Samples the targets do not describe (section 2): candles and wax, a sphere or section lit on its
  downstream side (wake flame), a rod lit at its downstream end with the flame spreading against
  the flow along it, Nomex, Ultem, fabric. Running a rod-tip or flat target on these would be a
  number without a law behind it.
- Flat PMMA thicker than 5 mm: the flat targets carry no conduction term, so just after ignition
  they describe a slab already warm through. For these the pair is an upper bound, not a forecast.
- Earlier burns of the same sample: no typed input carries them. A sample re-lit while warm is more
  flammable than the forecast says; the bias is towards "no" and is not corrected.

**Where this is honest and where it is not.** The typed answer is conditional on two things the
forward does not see: that a flame was established at all, and how long the igniter had been on.

- Films, slabs up to 5 mm, 1/4 inch rods, the 1 cm sphere: `heat_time` hardly matters (a rod of
  R = 0.318 cm at 17 % and 1 cm/s: 0.95, 0.97, 0.98 at 80, 120, 200 s). The answer stands as typed.
- 1/2 inch rods, 1.5 and 2 cm spheres and sections, at 2 cm/s and below: the answer moves with the
  on-time. 2 cm sphere in air at 1 cm/s: 0.50, 0.63, 0.77 at 80, 120, 200 s; at 2 cm/s: 0.75, 0.83,
  0.91; at 17 % and 2 cm/s: 0.44, 0.57, 0.72. 1/2 inch rod at 17 % and 1 cm/s: 0.58, 0.70, 0.82.
  With the on-time unknown, 200 s is the optimistic end if the igniter was on for less than about
  two minutes. From 5 cm/s up the three values differ by 0.05 at most in air; at 17 % the 2 cm
  sphere still moves from 0.77 to 0.92 at 5 cm/s and from 0.91 to 0.97 at 10 cm/s.
- The ignition half of `sustain` is not in the forward. Section 1.4 gives the time an igniter must
  stay on, `t_min = [b_c / (k_s sqrt(a_e) (X - x_s) - q_r)]^2`, seconds in air at 10 cm/s and
  minutes near the limit; an igniter withdrawn earlier leaves no flame however flammable the
  condition. Where the crew held the igniter until a flame stood, the typed answer applies; where
  the on-time was fixed and short, it is too high for thick samples near the limit.
- "Kept burning" here means 60 s. Thick PMMA cools slowly, so a flame that will go out can last
  20 to 60 s; scored on a shorter criterion the typed answer is too low near the boundary.

**What one more typed target would settle.** The missing quantity is the igniter on-time. A binary
target `ignites`, with logit proportional to `ln(t_on / t_min)` and `t_min` from the formula above,
would answer the first half, and `sustain` would be the smallest of the three. It needs `t_on` as a
sixth feature, which the declaration of the registered slug does not have; that is a new slug, not
an edit of this release. Until then `sustain` is answered as above and should be reported as
"given that a flame was established, on-time taken as 140 s".

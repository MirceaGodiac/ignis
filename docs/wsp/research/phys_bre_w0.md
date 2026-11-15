# ACME BRE (PSI-20): does a flame over a burning surface go out by itself in a still atmosphere? A W0 law

Author: `claude-fable-5-2`, 15 November 2026. Kind: **W0, literal authoring, zero-shot**. Slug `bre`.
No file under `runs/` (other than the brief), `data/` or `holdout/` was opened, and no feasibility note
for BRE existed when this was written, so the targets of section 2 are defined here. Section 8.3 lists
exactly what was read, including aggregate statements about BRE outcomes that are printed on the
investigation's card and in published abstracts. Every number below was typed from a property table,
a derivation shown here, a cited publication or a stated judgement. Nothing was fitted. The only
programs run were forward evaluations of the typed law and of its network restatement, to fill the
tables of sections 5 and 7.

## 0. The result in one screen

A porous disc of radius `R` feeds gas at a mass flux `m` into still gas. Without buoyancy the flame is
a dome that grows by diffusion towards a steady size, and its heat release is spread over an ever
larger sheet. Three outcomes follow from two sizes, both in burner radii:

```text
xi_ss = cot(L / Pe)                 steady flame height / R     (diffusion alone)
xi_e  = sqrt(Lambda - 2/3)          flame height / R at which radiation puts the flame out

Pe = m R / (2 rhoD)                 L = ln(1 + Y_O / (S Y_F))
Lambda = m Y_F dh_c (1 - phi_b / (1 - theta_c)) / (2 q_c0 p^a)

Lambda <= 2/3     the flame cannot stand off the burner: it does not establish, or dies in seconds
xi_ss <= xi_e     the flame reaches its steady size first: it is held (burns as long as it is fed)
xi_ss >  xi_e     the flame goes out by itself at
                  t_ext = (R^2 p / D0) xi_e^2 / (4 u^2),   erfc(u) = Z_st / (1 - exp(-Pe arccot(xi_e)))
```

`q_c0 = 9.4 kW/m2` is the chemical heat release per unit flame area below which a non-buoyant
ethylene flame is put out by its own radiation (published for spherical flames in the same chamber);
`phi_b` is the share of the heat that returns to the cold burner. The boundary variable is
`b = ln(xi_ss / xi_e)`: held for `b < 0`, a life that shortens steeply as `b` grows from 0 to 0.5 and
then flattens. The theory is unsure of `b` by 0.30 and of the clock by 0.45 (both ln, one sd).

For the 25 mm burner in air at 1 atm the law gives a life of 19 to 43 s for 2 to 12 g/m2/s and no
held flame at any flux. In 34 % oxygen at 0.57 atm it holds the flame between 1.3 and 2.2 g/m2/s of
ethylene and gives 120 s at 6 g/m2/s. The 50 mm burner never holds a flame; its flames live 20 to
75 s in air and 60 to 140 s in 34 to 40 % oxygen.

## 1. Mechanism

### 1.1 The steady flame over a disc

Take unit Lewis number, constant `rhoD` and one-step fast chemistry. The mixture fraction `Z` (1 in
the burner gas, 0 far away) obeys one linear equation. In oblate spheroidal coordinates
(`z = R xi eta`, `r = R sqrt((1 + xi^2)(1 - eta^2))`; the disc is `xi = 0`, the axis above its centre is
`z = R xi`) a field that depends on `xi` alone carries the total flow `M = pi R^2 m` through every
surface `xi = const`:

```text
M Z - 2 pi R rhoD (1 + xi^2) dZ/dxi = M        (all of it burner gas)
=>  Z_ss(xi) = 1 - exp(-Pe arccot(xi)),         Pe = M / (2 pi R rhoD) = m R / (2 rhoD)
```

Far away `arccot(xi) -> R/z` and this is the half-space point source `1 - exp(-M / (2 pi rhoD z))`.
At the burner face `Z_s = 1 - exp(-pi Pe / 2) < 1`: products diffuse back to the face. The flame is the
spheroid where `Z = Z_st = Y_O / (S Y_F + Y_O)`:

```text
xi_ss = cot(L / Pe),   L = -ln(1 - Z_st) = ln(1 + Y_O / (S Y_F))      (needs L / Pe < pi/2)
```

`Y_O` ambient oxygen mass fraction, `Y_F` fuel mass fraction of the burner gas, `S` kg of oxygen per
kg of fuel. For `Pe >> L` the height is `M / (2 pi rhoD L)`: proportional to the total flow, to
`m R^2`, and inversely to `L`, which is nearly proportional to oxygen and inversely to the fuel
fraction. It does not depend on pressure. This is the published BRE standoff scaling (Markan et al.
2020, the same solution with `Z` normalised at the face).

The coupling variable `c_p (T - T_inf) + (dh_c / S)(Y_O - Y_O,inf)` obeys the same equation with fixed
values at the face (cold, no oxygen) and far away, so the flame temperature without radiation is

```text
c_p (T_f - T_inf) = Z_st Y_F dh_c (1 - phi_b),     phi_b = Z_st (1 - Z_s) / ((1 - Z_st) Z_s)
```

`phi_b` is exactly the share of the heat release conducted into the burner (it is 1 when the flame
sits on the face, 0 for a large `Pe`). With `dh_c` it is the "heat of gasification" that BRE reads
from its heat-flux gauges: `L_g = phi_b Y_F dh_c`, 1 to 6 kJ/g over the conditions below.

### 1.2 Growth

A source switched on at `t = 0` in a half space fills it by diffusion; far from the burner the exact
transient is the steady field times `erfc(z / (2 sqrt(D t)))`. Markan et al. multiply the exact steady
solution by this factor (it is exact far away and at long times):

```text
Z(xi, t) = Z_ss(xi) erfc(xi / (2 sqrt(tau))),    tau = D t / R^2
```

The flame at time `t` is the spheroid where this equals `Z_st`. The time to reach a height `xi` is
explicit: `t = (R^2 / D) xi^2 / (4 u^2)` with `erfc(u) = Z_st / Z_ss(xi)`. The approach to the steady
size is slow (`1 - xi/xi_ss ~ xi_ss / sqrt(pi tau)`), so a flame near its steady size has been burning
for many diffusion times `R^2 / D` (3 s for the 25 mm burner, 12 s for the 50 mm one at 1 atm).

### 1.3 Why it goes out

The heat release `Q = M Y_F dh_c` is fixed by the flow, while the sheet area `A_f` grows; the release
per unit area `Q / A_f` falls as the flame grows. The hot gas on both sides of the sheet radiates
(CO2 and H2O bands; these flames are blue). Per unit sheet area the loss at a given flame
temperature does not fall as the flame grows, so the flame temperature falls:

```text
(T_f - T_inf) / (T_ad - T_inf) = 1 - phi_b - q_R'' / (Q / A_f)
```

Below a temperature `T_c` near 1100 to 1200 K chain branching no longer outruns recombination,
however long the residence time, and the sheet goes out (first at the top, where the flux is lowest,
then by oscillation everywhere). With `theta_c = c_p (T_c - T_inf) / (Z_st Y_F dh_c)` the flame is lost
when

```text
Q / A_f  <  q_R''(T_c) / (1 - phi_b - theta_c)
```

**The loss `q_R''`.** Two estimates bracket it for a flame of a few centimetres at `T_c`. The
optically thin one, `4 kappa_P sigma T_c^4` times a radiating thickness of 0.4 flame radii with
`kappa_P = 3.4 /m` (13 % CO2 and 13 % H2O at 1150 K), gives 16 kW/m2 at 30 mm. The band-saturated one
(total emissivity 0.02 to 0.03 for a path of 0.003 to 0.005 m atm) gives 1.4 to 2 kW/m2. The strong
4.3 micron CO2 band is neither thin nor black at these paths, so the truth lies between, and the
theory alone is unsure by a factor of three each way. I therefore take the measured value from the
neighbouring experiment in the same chamber: Irace et al. (2023) find that ethylene spherical
diffusion flames at 1 bar, in 20 to 38 % oxygen, with 13 to 100 % fuel in the burner gas and flows of
0.6 to 12 mg/s, all go out when the fuel flux through the sheet falls to 0.2 g/m2/s, at 1130 K. That
is `q_c0 = 0.2 g/m2/s x 47.2 MJ/kg = 9.4 kW/m2` of chemical release per sheet area, which lies inside
the bracket once multiplied by `1 - theta_c` (6 to 8 kW/m2 of loss). Their flames lose nothing to the
burner, so for BRE the critical flux is raised by the burner's share:

```text
q_x = q_c0 p^a (1 - theta_c) / (1 - phi_b - theta_c)
```

Pressure: emission follows `p` in the thin limit and about `p^0.6` to `p^0.9` for saturated bands with
line broadening at a fixed path; the hot layer outside a growing flame thins as `p^-1/2`. I take
`a = 0.5 +/- 0.3`.

**The sheet area.** Half an oblate spheroid of height `R xi` over a disc of radius `R` has area
`pi R^2 [(1 + xi^2) + xi^2 sqrt(1 + xi^2) arcsinh(1/xi)]`, which is `2 pi R^2 (xi^2 + 2/3)` to 3 % for
`xi >= 1`. Setting `Q / A_f = q_x`:

```text
xi_e^2 = Lambda - 2/3,      Lambda = m Y_F dh_c / (2 q_x)
```

The height at which the flame goes out, in burner radii, depends on the fuel heat-release flux at
the burner face and on nothing else (apart from the burner's share): 2 to 5 radii for 2 to 12 g/m2/s
of ethylene.

### 1.4 The groups and the three regimes

| Group | Meaning | Range flown (25 mm; 50 mm) |
|---|---|---|
| `Pe = m R / (2 rhoD)` | blowing against diffusion over the burner radius | 0.1 to 1.3; 0.2 to 2.5 |
| `L = ln(1 + Y_O / (S Y_F))` | oxygen the fuel needs, as a transfer number | 0.066 (air) to 0.12 (40 %); 0.13 to 0.23 at 50 % fuel |
| `Lambda = m Y_F dh_c / (2 q_x)` | heat release at the burner face over twice the critical flux | 1 to 30 |
| `phi_b` | share of the heat returned to the burner | 0.01 to 0.6 |
| `b = ln(xi_ss / xi_e)` | steady size over extinction size | -0.25 to 2 |
| `R^2 p / D0` | diffusion time of the burner | 3 s; 12 s at 1 atm |

- `Lambda <= 2/3` (or `phi_b + theta_c >= 1`): the release at the face cannot pay the burner and the
  radiation even for a flame flat on the burner. Small-flame, heat-loss limit.
- `b <= 0`: held. In the far field `xi_ss ~ m R^2 / (2 rhoD L)` and `xi_e ~ sqrt(m Y_F dh_c / (2 q_x))`,
  so the flame is held when `m R^4 < 2 (rhoD L)^2 Y_F dh_c / q_x`: small burners, low flux, high
  oxygen (through `L^2`), low pressure (through `q_x`). The total flow `m R^2` matters more than the
  flux.
- `b > 0`: self-extinction at `t_ext`. Near `b = 0` the life diverges; for `b > 1` it grows slowly
  with the flow, because a stronger flame must grow further before its flux falls to `q_x`.

`t_ext` is weakly dependent on pressure away from the boundary (diffusion is faster at low pressure
but the flame must grow larger): 24 s at 1 atm against 28 s at 0.57 atm for 3 g/m2/s in air. Near the
boundary low pressure lengthens the life strongly.

### 1.5 Limits and orders of magnitude (checks, not fits)

- `m -> infinity`: `phi_b -> 0`, the burner no longer matters, the law is the spherical-flame law in a
  half space. `Y_O -> 0`: `L -> 0`, the steady flame is infinitely large, every flame goes out.
- Markan et al. (2020), 5 s drops of this burner: with their `rhoD` the law's steady heights for their
  three 25 mm tests are 49, 55 and 91 mm and the composite solution gives 44, 36 and 26 % of that at
  5 s; they report "about 30 to 40 %". (For their 50 mm tests I get 12 to 16 %; see 8.2.)
- Heat to the burner: `phi_b` = 0.06 to 0.12 for those tests, 3 to 6 kJ/g of ethylene, 13 to 20 kW/m2
  on the face. The card gives 1 to 10 kJ/g as the range of BRE's heat of gasification.
- A mass bound on the clock: the flame encloses gas with `Z >= Z_st`, so its mass cannot exceed the
  mass injected divided by `Z_st`. For 3 g/m2/s in air this needs at least 1 s to reach the extinction
  size; the diffusion clock says 24 s. No conflict.
- Published aggregate statements about BRE itself, read on the card and in abstracts (class L, this
  experiment; not used to set a number): every flame in 21 % oxygen went out within 40 s; quasi-steady
  flames at 26.5 % and above; little effect of pressure; lives correlated as `X_O2^3 Re^-0.5`,
  shorter on the larger burner. The law gives 19 to 43 s in air and a life that rises as `X^3.6`
  between 21 and 40 % oxygen for the 25 mm burner at 4 g/m2/s and 1 atm (27, 45, 106, 273 s). It does
  **not** reproduce a shorter life on the larger burner in air (section 6), and at 1 atm it holds no
  flame at 26.5 %.

### 1.6 The earlier author's kernel (`property_bre_w0.rs`)

It is a correct ledger and not a forecast. It advances a lumped "virtual material" and a lumped
burner temperature from heat fluxes, a conversion fraction and an "absorbed heat fraction" that the
caller must supply, and it reports safety as unknown. Three things keep it from the question here:
(1) nothing in it decides whether a flame exists, so extinction is outside it by construction;
(2) it limits burning by an *oxidiser mass flow* supplied to the apparatus, which does not exist in a
quiescent chamber where oxygen arrives by diffusion; (3) its closed-loop mode (feed commanded by the
emulated mass loss) is not how the flight tests ran: the flow was set and held (card, approach). What
it asks for as an input is what the theory gives: the absorbed fraction is `phi_b` of 1.1. Its burner
heating balance is worth keeping for one thing this law leaves out, the operator's stop when the
burner or the heat-flux gauge nears its temperature limit, which censors long burns.

## 2. Targets

No feasibility note existed; these are defined from the card (video and image archives for 207 tests,
sensor series for 187).

| Target | Kind | Unit | Definition | A forecast may know |
|---|---|---|---|---|
| `flame_end` | event, cause `extinction` | s since ignition | the flame goes out by itself while the fuel flow is still at its first setpoint; a test ended or re-set by the operator is a stop (censored) | conditions set before ignition, the planned stop if one is known |
| `lasts` | yes or no | - | the flame is still burning `stated_time` seconds after ignition, flow unchanged; tests stopped earlier are not scored | the same, and `stated_time` |
| `height_at_end` (optional) | positive | m | height of the flame above the burner centre in the last second before self-extinction | the same |

Inputs known before ignition: burner diameter; fuel and its mole fraction in the burner gas; mass
flux of the burner gas (fuel plus its nitrogen, per unit face area); ambient oxygen mole fraction and
chamber pressure just before ignition. Not known: anything after ignition.

## 3. The law

```text
Y_O = 32 X / (32 X + 28.01 (1 - X))            Y_F = x_F W_F / (x_F W_F + 28.01 (1 - x_F))
Z_st = Y_O / (S Y_F + Y_O)                     L = -ln(1 - Z_st)
Pe = m R / (2 rhoD)                            Z_s = 1 - exp(-pi Pe / 2)
phi_b = Z_st (1 - Z_s) / ((1 - Z_st) Z_s)      theta_c = c_p (T_c - T_inf) / (Z_st Y_F dh_c)
q_x = q_c0 (p / 1 atm)^a (1 - theta_c) / (1 - phi_b - theta_c)
Lambda = m Y_F dh_c / (2 q_x)                  xi_e = sqrt(Lambda - 2/3) e^(-eps)
xi_ss = cot(L / Pe)                            x = Z_st / (1 - exp(-Pe arccot(xi_e)))

no flame   if phi_b + theta_c >= 1 or Lambda <= 2/3
held       if x >= 1
else       t_ext(eps) = (R^2 p / (D0 atm)) xi_e^2 / (4 erfcinv(x)^2)
```

Median: `eps = 0`. Spread: `eps ~ N(0, s_b)` with `s_b = 0.30` is the theory's uncertainty about the
ratio of the two sizes (`q_c0` and its transfer to this geometry 0.15, `rhoD` 0.10, the sheet shape
and where on it extinction starts 0.10, the model 0.20, in ln of size), and the clock carries a
further log-normal factor with `s_t = 0.45` (`D0` 0.35, ignition transient and the definition of "out"
0.25). So

```text
P(lasts t) = integral phi(e) [ held(s_b e)  or  Phi((ln t_ext(s_b e) - ln t) / s_t) ] de
P(held)    = Phi(-b / s_b) to first order
height at the end = R sqrt(Lambda - 2/3),  ln-sd 0.30
```

The spread of the life is not a constant: `d ln t_ext / d b` is -1 far from the boundary and below
-10 near it, so the same 0.30 is 0.5 in ln of time at `b = 1` and more than 2 at `b = 0.2`.

Methane uses the same `q_c0` (per joule it makes 17 % less CO2 and more H2O; its lower adiabatic
temperature works the other way) and its own `S`, `dh_c`, `W_F`.

## 4. Constants

| Symbol | Value | Class | Origin | Uncertainty |
|---|---|---|---|---|
| `rhoD` | 6.0e-5 kg/m/s | L, P | Markan et al. 2020 (drop tests of this burner); equals `rho D` of nitrogen at about 1100 K (1.18 x 2.1e-5 x (1100/300)^0.7 = 6.2e-5) | 15 % |
| `D0` | 5.1e-5 m2/s at 1 atm, as 1/p | L, T | `rhoD` over the ambient density 1.18 kg/m3, the reading of Markan's `tau` that reproduces their 25 mm growth | 35 % |
| `q_c0` | 9.4 kW/m2 | L (other experiment) | Irace et al. 2023: 0.2 g/m2/s of ethylene at the sheet, spherical flames, 1 bar; theory bracket 1.4 to 16 kW/m2 of loss | 30 % as transferred |
| `a` | 0.5 | J | between thin (1), saturated bands (0.6 to 0.9) and a hot layer thinning as p^-1/2 | 0.3 |
| `T_c` | 1130 K | L | Irace et al. 2023; 1100 to 1200 K is also stated for BRE on the card | 70 K |
| `T_inf` | 300 K | P | chamber gas | 5 K |
| `c_p` | 1.17 kJ/kg/K | P | mean of nitrogen-rich products, 300 to 1130 K | 5 % |
| `dh_c` | 47.2 (C2H4), 50.0 (CH4) MJ/kg | P | lower heating values | 0.5 % |
| `S` | 3.422 (C2H4), 3.989 (CH4) | P | stoichiometry | exact |
| `W` | 28.05, 16.04; N2 28.01, O2 32.00 g/mol | P | | exact |
| 2/3 | 0.667 | T | area of half an oblate spheroid, large-`xi` expansion | 3 % of area for `xi >= 1` |
| `s_b` | 0.30 | J | section 3 | - |
| `s_t` | 0.45 | J | section 3 | - |
| `kappa_P` | 3.4 /m (CO2 21.8, H2O 4.2 /m/atm at 1150 K, 13 % each) | P, from memory | RADCAL curve fits; used only for the bracket of 1.3 | 25 % |

## 5. Registered expectations

Written before any comparison. "closed form" is section 3 with `eps = 0` (held = no self-extinction;
its two probability columns are the integral of section 3). "network" is what the forward of
section 7 returns from the numbers of 7.4 and is the registered forecast: `tau` the median of the
extinction clock, `P10 ... P120` the probability that the flame lasts 10 to 120 s, `P_inf` that it
never goes out by itself, `h` the flame height at the end. Flux in g/m2/s of burner gas, pressure
in atm.

| d mm | fuel | X_O2 | p | flux | closed form t_ext s | P30 | P120 | network tau s | P10 | P30 | P60 | P120 | P_inf | h mm |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 25 | C2H4 | 0.210 | 1.00 | 2 | 19.1 | 0.39 | 0.17 | 22.5 | 0.79 | 0.43 | 0.30 | 0.21 | 0.11 | 23 |
| 25 | C2H4 | 0.210 | 1.00 | 3 | 23.8 | 0.44 | 0.16 | 24.4 | 0.89 | 0.43 | 0.25 | 0.17 | 0.06 | 30 |
| 25 | C2H4 | 0.210 | 1.00 | 6 | 31.3 | 0.52 | 0.15 | 34.9 | 0.99 | 0.60 | 0.27 | 0.13 | 0.02 | 47 |
| 25 | C2H4 | 0.210 | 1.00 | 12 | 43.2 | 0.64 | 0.18 | 58.9 | 1.00 | 0.94 | 0.49 | 0.15 | 0.01 | 73 |
| 25 | C2H4 | 0.265 | 0.69 | 2 | 69.0 | 0.64 | 0.43 | 56.2 | 0.90 | 0.68 | 0.56 | 0.44 | 0.31 | 25 |
| 25 | C2H4 | 0.265 | 0.69 | 3 | 57.3 | 0.64 | 0.38 | 54.8 | 0.94 | 0.67 | 0.50 | 0.38 | 0.19 | 32 |
| 25 | C2H4 | 0.265 | 0.69 | 6 | 49.6 | 0.65 | 0.29 | 55.5 | 0.99 | 0.77 | 0.48 | 0.29 | 0.08 | 51 |
| 25 | C2H4 | 0.265 | 0.69 | 12 | 55.7 | 0.71 | 0.27 | 76.7 | 1.00 | 0.96 | 0.65 | 0.31 | 0.03 | 80 |
| 25 | C2H4 | 0.340 | 0.57 | 2 | held | 0.84 | 0.72 | 135 | 0.96 | 0.86 | 0.78 | 0.69 | 0.57 | 26 |
| 25 | C2H4 | 0.340 | 0.57 | 3 | 674 | 0.82 | 0.65 | 131 | 0.97 | 0.84 | 0.74 | 0.63 | 0.42 | 34 |
| 25 | C2H4 | 0.340 | 0.57 | 6 | 119 | 0.79 | 0.50 | 126 | 1.00 | 0.88 | 0.70 | 0.53 | 0.20 | 53 |
| 25 | C2H4 | 0.340 | 0.57 | 12 | 88.5 | 0.81 | 0.42 | 126 | 1.00 | 0.98 | 0.81 | 0.53 | 0.08 | 83 |
| 25 | C2H4 | 0.400 | 0.57 | 2 | held | 0.91 | 0.84 | 224 | 0.98 | 0.91 | 0.86 | 0.80 | 0.71 | 26 |
| 25 | C2H4 | 0.400 | 0.57 | 3 | held | 0.90 | 0.78 | 218 | 0.98 | 0.90 | 0.83 | 0.75 | 0.56 | 34 |
| 25 | C2H4 | 0.400 | 0.57 | 6 | 297 | 0.86 | 0.64 | 208 | 1.00 | 0.93 | 0.81 | 0.67 | 0.31 | 53 |
| 25 | C2H4 | 0.400 | 0.57 | 12 | 134 | 0.86 | 0.53 | 200 | 1.00 | 0.99 | 0.88 | 0.66 | 0.14 | 83 |
| 25 | C2H4 | 0.340 | 1.00 | 2 | 211 | 0.75 | 0.56 | 101 | 0.95 | 0.80 | 0.70 | 0.59 | 0.42 | 22 |
| 25 | C2H4 | 0.340 | 1.00 | 3 | 137 | 0.76 | 0.52 | 98.5 | 0.98 | 0.80 | 0.65 | 0.53 | 0.28 | 29 |
| 25 | C2H4 | 0.340 | 1.00 | 6 | 87.5 | 0.77 | 0.43 | 94.3 | 1.00 | 0.88 | 0.66 | 0.44 | 0.12 | 46 |
| 25 | C2H4 | 0.340 | 1.00 | 12 | 86.3 | 0.82 | 0.40 | 111 | 1.00 | 0.99 | 0.82 | 0.48 | 0.05 | 71 |
| 50 | C2H4 | 0.210 | 1.00 | 1.5 | 21.2 | 0.38 | 0.07 | 22.3 | 0.96 | 0.35 | 0.13 | 0.07 | 0.02 | 41 |
| 50 | C2H4 | 0.210 | 1.00 | 3 | 37.2 | 0.59 | 0.14 | 39.1 | 1.00 | 0.76 | 0.23 | 0.07 | 0.01 | 64 |
| 50 | C2H4 | 0.210 | 1.00 | 6 | 60.1 | 0.77 | 0.24 | 75.0 | 1.00 | 0.98 | 0.72 | 0.15 | 0.00 | 101 |
| 50 | C2H4 | 0.265 | 0.69 | 1.5 | 31.5 | 0.52 | 0.18 | 32.2 | 0.97 | 0.54 | 0.30 | 0.18 | 0.05 | 45 |
| 50 | C2H4 | 0.265 | 0.69 | 3 | 47.5 | 0.67 | 0.22 | 48.1 | 1.00 | 0.82 | 0.39 | 0.16 | 0.02 | 70 |
| 50 | C2H4 | 0.265 | 0.69 | 6 | 69.2 | 0.80 | 0.30 | 83.8 | 1.00 | 0.99 | 0.79 | 0.26 | 0.01 | 110 |
| 50 | C2H4 | 0.340 | 0.57 | 1.5 | 61.1 | 0.69 | 0.36 | 64.5 | 0.98 | 0.75 | 0.53 | 0.38 | 0.14 | 46 |
| 50 | C2H4 | 0.340 | 0.57 | 3 | 72.8 | 0.77 | 0.36 | 72.8 | 1.00 | 0.91 | 0.60 | 0.33 | 0.05 | 73 |
| 50 | C2H4 | 0.340 | 0.57 | 6 | 91.4 | 0.86 | 0.41 | 108 | 1.00 | 0.99 | 0.88 | 0.45 | 0.02 | 114 |
| 50 | C2H4 | 0.400 | 0.57 | 1.5 | 109 | 0.78 | 0.49 | 107 | 0.99 | 0.84 | 0.67 | 0.52 | 0.22 | 46 |
| 50 | C2H4 | 0.400 | 0.57 | 3 | 105 | 0.83 | 0.47 | 103 | 1.00 | 0.94 | 0.72 | 0.47 | 0.09 | 72 |
| 50 | C2H4 | 0.400 | 0.57 | 6 | 117 | 0.89 | 0.50 | 138 | 1.00 | 1.00 | 0.92 | 0.59 | 0.03 | 114 |
| 25 | C2H4 50 % | 0.210 | 1.00 | 3 | 15.6 | 0.36 | 0.16 | 18.2 | 0.71 | 0.40 | 0.29 | 0.20 | 0.13 | 19 |
| 25 | C2H4 50 % | 0.210 | 1.00 | 6 | 26.0 | 0.46 | 0.17 | 25.1 | 0.90 | 0.44 | 0.25 | 0.17 | 0.06 | 30 |
| 25 | C2H4 50 % | 0.265 | 0.69 | 3 | 69.9 | 0.63 | 0.44 | 48.4 | 0.87 | 0.68 | 0.56 | 0.44 | 0.36 | 21 |
| 25 | C2H4 50 % | 0.265 | 0.69 | 6 | 60.8 | 0.66 | 0.38 | 55.9 | 0.94 | 0.67 | 0.50 | 0.38 | 0.19 | 33 |
| 25 | C2H4 50 % | 0.340 | 0.57 | 3 | held | 0.84 | 0.73 | 103 | 0.94 | 0.84 | 0.76 | 0.66 | 0.60 | 22 |
| 25 | C2H4 50 % | 0.340 | 0.57 | 6 | 543 | 0.83 | 0.65 | 134 | 0.97 | 0.85 | 0.74 | 0.63 | 0.41 | 34 |
| 25 | C2H4 50 % | 0.400 | 0.57 | 3 | held | 0.91 | 0.84 | 141 | 0.95 | 0.88 | 0.82 | 0.74 | 0.69 | 22 |
| 25 | C2H4 50 % | 0.400 | 0.57 | 6 | held | 0.90 | 0.78 | 222 | 0.98 | 0.91 | 0.83 | 0.75 | 0.56 | 34 |
| 50 | C2H4 50 % | 0.210 | 1.00 | 3 | 23.8 | 0.42 | 0.09 | 23.2 | 0.96 | 0.36 | 0.14 | 0.07 | 0.01 | 42 |
| 50 | C2H4 50 % | 0.265 | 0.69 | 3 | 35.4 | 0.56 | 0.19 | 33.1 | 0.97 | 0.56 | 0.30 | 0.18 | 0.05 | 46 |
| 50 | C2H4 50 % | 0.340 | 0.57 | 3 | 67.9 | 0.72 | 0.38 | 65.8 | 0.99 | 0.76 | 0.54 | 0.38 | 0.14 | 47 |
| 50 | C2H4 50 % | 0.400 | 0.57 | 3 | 119 | 0.80 | 0.51 | 109 | 0.99 | 0.84 | 0.67 | 0.52 | 0.22 | 47 |
| 25 | CH4 | 0.210 | 1.00 | 3 | 19.8 | 0.38 | 0.12 | 20.3 | 0.87 | 0.36 | 0.19 | 0.12 | 0.04 | 31 |
| 25 | CH4 | 0.210 | 1.00 | 6 | 27.7 | 0.48 | 0.11 | 31.7 | 0.99 | 0.55 | 0.21 | 0.09 | 0.02 | 49 |
| 25 | CH4 | 0.265 | 0.69 | 3 | 38.7 | 0.57 | 0.29 | 40.1 | 0.93 | 0.59 | 0.41 | 0.30 | 0.13 | 34 |
| 25 | CH4 | 0.265 | 0.69 | 6 | 40.0 | 0.59 | 0.23 | 45.8 | 0.99 | 0.72 | 0.40 | 0.22 | 0.05 | 53 |
| 25 | CH4 | 0.340 | 0.57 | 3 | 166 | 0.76 | 0.55 | 96.1 | 0.96 | 0.79 | 0.66 | 0.54 | 0.32 | 35 |
| 25 | CH4 | 0.340 | 0.57 | 6 | 78.2 | 0.74 | 0.41 | 92.0 | 1.00 | 0.86 | 0.63 | 0.44 | 0.14 | 55 |
| 25 | CH4 | 0.400 | 0.57 | 3 | 1806 | 0.85 | 0.69 | 159 | 0.98 | 0.87 | 0.77 | 0.68 | 0.45 | 35 |
| 25 | CH4 | 0.400 | 0.57 | 6 | 148 | 0.82 | 0.54 | 153 | 1.00 | 0.91 | 0.75 | 0.58 | 0.22 | 55 |

Where the closed form holds a flame (flux of burner gas, g/m2/s; "stands" = lowest flux at which a
flame can stand off the burner):

| Burner, fuel | 21 %, 1.00 atm | 26.5 %, 0.69 atm | 34 %, 0.57 atm | 40 %, 0.57 atm | 34 or 40 %, 1.00 atm |
|---|---|---|---|---|---|
| 25 mm, C2H4 | stands above 0.84; never held | above 0.88; never | held 1.26 to 2.21 | held 1.22 to 3.45 | never (lives 100 to 700 s) |
| 25 mm, C2H4 50 % | above 1.64; never | above 1.70; never | held 2.44 to 4.19 | held 2.34 to 6.45 | never |
| 25 mm, CH4 | above 0.77; never | above 0.80; never | never (166 s at 3) | held 1.16 to 2.52 | never |
| 50 mm, C2H4 | above 0.55; never | never | never | never | never |
| 50 mm, C2H4 50 % | above 1.07; never | never | never | never | never |

In words: (a) in air the closed form holds no flame; half of the 25 mm flames are out before 20 to
45 s and about one in six lasts two minutes; (b) held flames exist only on the 25 mm burner, in 34 % oxygen
or more at reduced pressure and at low flux, and an emulated material of half the heat of combustion
is held at twice the gas flux; (c) the 50 mm burner holds nothing; (d) at equal flux the larger
burner's flame lives longer in air; (e) the flame at the end stands 1.6 to 6.6 burner radii high and
that height follows the square root of the fuel heat-release flux.

## 6. What would show the law is wrong

1. **The air lives.** If the median self-extinction time of 25 mm ethylene flames in 21 % oxygen at
   1 atm and 2 to 6 g/m2/s lies outside 10 to 60 s, or more than a third of them are still burning at
   120 s, the pair (`q_c0`, `D0`) is wrong; too short points at the clock or at the neglected
   ignition cloud, too long at the transferred critical flux.
2. **The size at the end.** The law says the flame goes out at a height `R sqrt(Lambda - 2/3)`: 30 mm
   at 3 g/m2/s and 47 mm at 6 g/m2/s on the 25 mm burner, about twice that on the 50 mm one, nearly
   whatever the oxygen (10 % more at 0.57 atm). If the measured heights at extinction do not follow the square root of the fuel
   heat-release flux, or differ from these by more than a factor 1.5, the constant critical flux
   borrowed from spherical flames does not hold for a flat burner.
3. **The ordering by burner and pressure.** The law says that in air at equal flux the 50 mm flame
   outlives the 25 mm one by a factor of 1.6 to 1.9, and that near the boundary (34 to 40 % oxygen,
   25 mm, 2 to 4 g/m2/s) lowering the pressure from 1 to 0.57 atm turns lives of 100 to 650 s into
   held flames or lives beyond 600 s. The published correlation (`X^3 Re^-0.5`, shorter on the larger burner, no pressure
   effect) suggests otherwise for part of this. If the larger burner's flames go out sooner in air,
   diffusion from the burner is not the clock; if pressure does nothing near the boundary, `a` is 0.

## 7. Interface for the strict forward (`law.rs`, `docs/wsp/research/law_forward.md`)

### 7.1 Declaration (names; for `configs/models/law/bre/decl.json`)

Features, in this order (the edits below depend on the order):

| # | name | unit of `value_si` | transform | meaning |
|---|---|---|---|---|
| 0 | `burner_diameter` | m | ln | diameter of the porous face (0.025 or 0.050) |
| 1 | `burner_mass_flux` | kg/m2/s | ln | mass flow of the burner gas (fuel plus its nitrogen) at the first setpoint, per unit face area |
| 2 | `x_o2` | 1 | ln | oxygen mole fraction of the chamber gas just before ignition |
| 3 | `pressure` | Pa | ln | chamber pressure just before ignition |
| 4 | `x_fuel` | 1 | ln | fuel mole fraction of the burner gas (1 for a pure fuel, 0.5 for the 50 % bottle) |
| 5 | `is_methane` | 1 | identity | 1 for methane, 0 for ethylene |
| 6 | `stated_time` | s | ln | the time asked about by `lasts`; any admissible value for the other targets, which give it no weight |

Targets:

| name | kind | detail |
|---|---|---|
| `flame_end` | event | causes `["extinction"]`; no `scheduled_stop.condition` (a stop comes from `known_controls` when the engineer knows one) |
| `lasts` | binary | flame still burning at `stated_time` |
| `height_at_end` | positive | unit `m`, `shared_quantity` null (lane-local) |

`family`: `other`. Time origin: ignition.

### 7.2 How the network restates the law

Centre: 25 mm, 4 g/m2/s, 26.5 % oxygen, 1 atm, pure ethylene, where `Pe = 0.417`, `phi_b = 0.092`,
`Lambda = 8.79`, `xi_e = 2.85`, `xi_ss = 5.03`, `b = 0.568`, `t_ext = 45 s`. With
`z = (z_d, z_m, z_o, z_p, z_f, z_c, z_t)` the normalised features, the law is three linear forms and
one curve. The coefficients are the law's own logarithmic derivatives at the centre, taken by hand
(for example `d ln xi_ss / d ln Pe = 2 phi / sin 2 phi = 1.03` at `phi = L / Pe = 0.197`;
`d ln xi_e / d ln Lambda = Lambda / (2 (Lambda - 2/3)) = 0.541`; `d ln phi_b / d ln Pe = -1.36`;
`d ln L / d ln X = 0.925`):

```text
b(z)    = 0.568 + 0.921 z_d + 0.380 z_m - 0.899 z_o + 0.27 z_p + 0.369 z_f + 0.11 z_c
base(z) = 3.214 + 2.21 z_d + 1.29 z_m - 0.10 z_o + 0.46 z_p + 1.23 z_f + 0.08 z_c     ln(R^2 p xi_e^2 / D0)
ln h    = -3.34 + 1.11 z_d + 0.65 z_m - 0.05 z_o - 0.27 z_p + 0.62 z_f + 0.04 z_c     ln(R xi_e / 1 m)
ln t_ext = base + G(b),   G(b) = -ln(4 erfcinv(e^(-0.9 b))^2)
```

(the factor 0.9 is the centre value of `ln x / (-b)`, because `x = Z_st / Z_ss(xi_e)` is a little
larger than `xi_e / xi_ss`). `G` from tables of the error function:

| b | 0.06 | 0.12 | 0.25 | 0.40 | 0.57 | 0.77 | 1.02 | 1.34 | 1.79 | 2.56 |
|---|---|---|---|---|---|---|---|---|---|---|
| G | 4.84 | 3.45 | 2.05 | 1.21 | 0.60 | 0.09 | -0.35 | -0.77 | -1.19 | -1.69 |

Checked against the closed form by evaluation over the table of section 5: `b` (with the small-flame
hinge below) is within 0.07 of the exact `b`; the height is within 10 %, except the smallest flames
(25 % too tall at 22 mm).

**`flame_end`.** `ln tau = base + G_s(b) + small-flame terms`, with `G_s` the chord approximation of
`G` on knots 1.8, 1.3, 1.0, 0.77, 0.57 and the slope below 0.57 held at -3.58 instead of following
`G` to infinity: `G_s(b) = -0.03 - 0.65 b + 0.29 r(1.8 - b) + 0.36 r(1.3 - b) + 0.49 r(1.0 - b)
+ 0.70 r(0.77 - b) + 1.09 r(0.57 - b)`, `r` = relu. The cap is deliberate: a log-logistic clock of
one shape cannot say "held with probability P, otherwise about t", so near the boundary the clock's
median is set where its 25 % point matches the law's (at `b = 0` the law gives 21 % of flames out
by 200 s; the capped clock gives 24 %). Shape 2.2 (ln-sd 0.82), the law's spread at `b` about 0.7.

**`lasts`.** `logit = c (H(v) - b - small-flame terms)`, `v = ln t* - base(z)`, `H = G^-1` (the `b`
whose life is `t*`), `c = 1.7 / 0.425 = 4.0`. `H` on knots 3.46, 1.21, 0.60, 0.09, -0.35, -0.77 with
slope increments 0.176, 0.103, 0.122, 0.159, 0.207, 0.294. For long times `H -> 0` and the logit
tends to `-c b`: the probability that the flame is held. This target carries the law near the
boundary and is the one to score there.

**Small flames.** Near the heat-loss limit `Lambda` falls faster than its centre slope. One margin
collapses the departures of the exact `b` from the linear one:
`h = 2.17 z_m + 1.17 z_d - 0.65 z_o - 0.5 z_p + 1.97 z_f + 0.18 z_c` (ln of the release at the face
times `(Pe / r)^1.2`). The exact `b` exceeds the linear one by 0.05 at `h = -1.5`, 0.15 at -2.1, 0.5
at -2.8 and without bound below -3.1: a hinge of slope 0.45 from -1.7 and a steep one from -2.8.

### 7.3 Tensors (as generated by `ignis wsp spec --family bre` once declared; n = 7, H = 8)

| Tensor | Shape | Unit | Bounds | Meaning here |
|---|---|---|---|---|
| `x.<f>.centre`, `.scale`, `.lo`, `.hi` | [] each, 7 features | unit of f; 1 | spec | centre and domain of each input |
| `flame_end.extinction.median.b0` | [] | ln(s) | [-20, 25] | ln of the median life at the centre, far-slope form |
| `flame_end.extinction.median.skip` | [7] | ln(s) per unit z | spec | `base - 0.65 b` |
| `flame_end.extinction.median.W1`, `.b1`, `.w2` | [8, 7], [8], [8] | - | spec | rows 0 to 4: `G_s` on `b`; rows 5, 6: small flames |
| `flame_end.extinction.shape` | [] | 1 | [0.1, 50] | log-logistic shape |
| `lasts.logit.b0`, `.skip` | [], [7] | logit | [-40, 40] | `-c b` |
| `lasts.logit.W1`, `.b1`, `.w2` | [8, 7], [8], [8] | - | spec | rows 0 to 5: `c H(v)`; rows 6, 7: small flames |
| `height_at_end.median.b0`, `.skip` | [], [7] | ln(m) | [-80, 80] | `ln(R xi_e)` |
| `height_at_end.spread.b0`, `.skip` | [], [7] | ln-sd | [0.001, 10] | 0.30, constant |
| `height_at_end.tail_weight`, `.tail_width` | [] | 1 | [0, 1], [1, 100] | one flame in ten off by up to three sd |

### 7.4 Literal assignments (a revision of the neutral root)

```text
# bre W0, zero-shot. z = (z_d, z_m, z_o, z_p, z_f, z_c, z_t), all ln except z_c.
x.burner_diameter.centre = 0.025
x.burner_diameter.scale = 1.0
x.burner_diameter.lo = 0.020
x.burner_diameter.hi = 0.060
x.burner_mass_flux.centre = 0.004
x.burner_mass_flux.scale = 1.0
x.burner_mass_flux.lo = 0.0008
x.burner_mass_flux.hi = 0.015
x.x_o2.centre = 0.265
x.x_o2.scale = 1.0
x.x_o2.lo = 0.18
x.x_o2.hi = 0.45
x.pressure.centre = 101325
x.pressure.scale = 1.0
x.pressure.lo = 50000
x.pressure.hi = 105000
x.x_fuel.centre = 1.0
x.x_fuel.scale = 1.0
x.x_fuel.lo = 0.3
x.x_fuel.hi = 1.0
x.is_methane.centre = 0.0
x.is_methane.scale = 1.0
x.is_methane.lo = 0.0
x.is_methane.hi = 1.0
x.stated_time.centre = 30.0
x.stated_time.scale = 1.0
x.stated_time.lo = 1.0
x.stated_time.hi = 3000.0

# flame_end: ln tau = base + G_s(b) + small flames. 2.82 = 3.214 - 0.03 - 0.65 * 0.568.
flame_end.extinction.median.b0 = 2.82
flame_end.extinction.median.skip[0:7] = [1.61, 1.05, 0.48, 0.28, 0.99, 0.01, 0.0]
# rows 0-4: relu(knot - b), knots 1.8, 1.3, 1.0, 0.77, 0.57; row weights are -b's coefficients
flame_end.extinction.median.W1[0:5, 0:7] = [[-0.921, -0.380, 0.899, -0.27, -0.369, -0.11, 0.0],
                                            [-0.921, -0.380, 0.899, -0.27, -0.369, -0.11, 0.0],
                                            [-0.921, -0.380, 0.899, -0.27, -0.369, -0.11, 0.0],
                                            [-0.921, -0.380, 0.899, -0.27, -0.369, -0.11, 0.0],
                                            [-0.921, -0.380, 0.899, -0.27, -0.369, -0.11, 0.0]]
flame_end.extinction.median.b1[0:5] = [1.232, 0.732, 0.432, 0.202, 0.002]
flame_end.extinction.median.w2[0:5] = [0.29, 0.36, 0.49, 0.70, 1.09]
# rows 5-6: small flames, relu(-1.7 - h) and relu(-2.8 - h)
flame_end.extinction.median.W1[5:7, 0:7] = [[-1.17, -2.17, 0.65, 0.5, -1.97, -0.18, 0.0],
                                            [-1.17, -2.17, 0.65, 0.5, -1.97, -0.18, 0.0]]
flame_end.extinction.median.b1[5:7] = [-1.7, -2.8]
flame_end.extinction.median.w2[5:7] = [-1.8, -6.0]
flame_end.extinction.shape = 2.2

# lasts: logit = 4.0 * (H(v) - b), v = 0.187 + z_t - base(z); 0.187 = ln 30 - 3.214
lasts.logit.b0 = -2.27
lasts.logit.skip[0:7] = [-3.68, -1.52, 3.60, -1.08, -1.48, -0.44, 0.0]
# rows 0-5: relu(knot - v), knots 3.46, 1.21, 0.60, 0.09, -0.35, -0.77; b1 = knot - 0.187
lasts.logit.W1[0:6, 0:7] = [[2.21, 1.29, -0.10, 0.46, 1.23, 0.08, -1.0],
                            [2.21, 1.29, -0.10, 0.46, 1.23, 0.08, -1.0],
                            [2.21, 1.29, -0.10, 0.46, 1.23, 0.08, -1.0],
                            [2.21, 1.29, -0.10, 0.46, 1.23, 0.08, -1.0],
                            [2.21, 1.29, -0.10, 0.46, 1.23, 0.08, -1.0],
                            [2.21, 1.29, -0.10, 0.46, 1.23, 0.08, -1.0]]
lasts.logit.b1[0:6] = [3.273, 1.023, 0.413, -0.097, -0.537, -0.957]
lasts.logit.w2[0:6] = [0.71, 0.41, 0.49, 0.64, 0.83, 1.18]
# rows 6-7: small flames
lasts.logit.W1[6:8, 0:7] = [[-1.17, -2.17, 0.65, 0.5, -1.97, -0.18, 0.0],
                            [-1.17, -2.17, 0.65, 0.5, -1.97, -0.18, 0.0]]
lasts.logit.b1[6:8] = [-1.7, -2.8]
lasts.logit.w2[6:8] = [-1.8, -8.0]

# height_at_end: ln(R xi_e / 1 m); -3.34 = ln(0.0125 * 2.85)
height_at_end.median.b0 = -3.34
height_at_end.median.skip[0:7] = [1.11, 0.65, -0.05, -0.27, 0.62, 0.04, 0.0]
height_at_end.spread.b0 = 0.30
height_at_end.spread.skip[0:7] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
height_at_end.tail_weight = 0.1
height_at_end.tail_width = 3.0
```

Hand check at the centre, `stated_time` 45 s: `v = 0.187 + ln 1.5 = 0.592`,
`4.0 H = 0.71 x 2.868 + 0.41 x 0.618 + 0.49 x 0.008 = 2.29`, logit `= -2.27 + 2.29 = 0.02`: one half at
the median life, as it must be. As `stated_time` grows the logit tends to -2.27: 9 % held.

### 7.5 What the forward cannot express, and what was done about it

- **A held class.** The event clock has one shape and no mass at infinity. `flame_end` therefore
  understates late survival near the boundary (at 34 %, 0.57 atm, 3 g/m2/s it gives 0.55 at 120 s
  where `lasts` gives 0.63, and it tends to 0 where `lasts` tends to 0.42). Score `lasts` for the
  yes/no question; use `flame_end` for the time of flames that did go out.
- **A spread that depends on the features** for the event. The law's ln-sd runs from 0.5 far from
  the boundary to above 2 near it; the shape 2.2 is a middle value.
- **Nested nonlinearity.** `G(b)` with `b` itself hinged cannot be written in one layer, so the
  small-flame departure enters as additive hinges. In the corner of high oxygen, dilute fuel and
  the lowest flux, where the closed form says no flame at all, the network still gives
  probabilities of 0.2 to 0.6 at 30 s. Those are conditions at or below the lowest flux flown.
- **A flow changed during the test** (BRE's stepwise search for the limit). The law is for the first
  constant-flow segment; later segments start from a grown flame and need a state the forward
  does not carry.

## 8. Sources and limits of this document

### 8.1 Sources, and what was taken from each

| Source | Taken |
|---|---|
| A. Markan, H. R. Baum, P. B. Sunderland, J. G. Quintiere, J. L. de Ris, "Transient ellipsoidal combustion model for a porous burner in microgravity", Combust. Flame 212 (2020) 93-106 (read: text of sections 5 to 7 and Table 1) | the oblate-spheroid solution, the composite transient, `rhoD = 6e-5 kg/m/s`, `k = 0.07 W/m/K`, `c_p = 1.167 kJ/kg/K`, the conditions of six drop tests, "30 to 40 % of steady height at 5 s". Pre-flight; no ISS outcome. |
| P. H. Irace, K. A. Waddell, D. Constales, P. B. Sunderland, R. L. Axelbaum, "Critical temperature and reactant mass flux for radiative extinction of ethylene microgravity spherical diffusion flames at 1 bar", Proc. Combust. Inst. 39 (2023) 1905-1913 (abstract) | 1130 K and 0.2 g/m2/s of fuel at the sheet, over 20 to 38 % oxygen and 13 to 100 % fuel. Another experiment (ACME spherical flames), same chamber. |
| A. E. Wright, "Using a Burning Rate Emulator to Analyze Flame Extinction Time on the International Space Station", M.S. thesis, Univ. of Maryland, 2021 (abstract only) | class L, **this experiment**: lives scale as `X_O2^3 Re^-0.5`, shorter on the 50 mm burner, no pressure effect; ranges flown. Used as a comparison in 1.5 and 6, not for a number. |
| P. Dehghani, "Burning Emulations of Condensed Phase Fuels Aboard the International Space Station", Ph.D. dissertation, Univ. of Maryland, 2022 (abstract only) | class L, **this experiment**: flames in 21 % oxygen out within 40 s; steady flames at 26.5, 34 and 40 %. Comparison only. |
| E. A. Kuznetsov, A. Yu. Snegirev, E. S. Markus, Combust. Explos. Shock Waves 56 (2020) 394-411 (abstract) | mechanism only: radiative cooling during growth, local extinction, oscillation. |
| The card `docs/wsp/atlas/cards/PSI-20.json` and `INVESTIGATIONS.md`, PSI-20 | apparatus, ranges flown, what was recorded; its "NASA states" findings (see 8.3). |

### 8.2 What I could not find or check

- The proportionality constant and the scatter of the published BRE correlation, the definition of
  its Reynolds number, and any per-test time: not sought (they are outcomes of this investigation).
- The full text of Irace et al. (2023): whether 0.2 g/m2/s is at the onset of extinction or at its
  end, and how constant it is across flame sizes of 10 to 50 mm. The law leans on it being constant.
- Markan et al. state that 5 s flames reach 30 to 40 % of their steady height on both burners. My
  reading of their `tau` (`D = rhoD / rho_inf`) gives that for the 25 mm tests and only 12 to 16 % for
  the 50 mm tests, where a mass bound also forbids 30 %. Either the statement is loose for the large
  burner or the clock of large burners is faster than `D0` says; the 50 mm lives carry that doubt.
- Planck-mean coefficients and total emissivities of CO2 and H2O were written from memory of the
  RADCAL fits and of Hottel's charts and were not checked on line; they enter only the bracket of
  1.3.
- The pressure of each oxygen level flown (the card says 0.565 to 1 bar and names 14.7, 10.2 and
  8.2 psia with 21, 26.5 and 34 %): the grid of section 5 assumes those pairs.
- How the flight tests were ignited (how much fuel had left the burner before the igniter fired) and
  how long a test was allowed to run.

### 8.3 Exposure: what this author saw

- The brief; `AGENTS.md`; `docs/wsp/PROTOCOL.md` sections 1 to 5 and 12.1 to 12.5;
  `docs/wsp/research/law_forward.md`.
- The card PSI-20 in full. Its narrative states aggregate outcomes: quasi-steady burning at oxygen
  0.265 and above with a weak pressure effect; flames in 21 % oxygen self-extinguished in a short
  time; extinction near 1100 to 1200 K with a radiative and a small-flame limit; radiant fractions up
  to about 60 %; emulated PE, PP, nylon and PMMA burning near 2.5 to 3 g/m2/s. I could not un-read
  these. None set a number; where the law disagrees with them (no held flame at 26.5 %, held PMMA-like
  flames only at reduced pressure) it was left as derived.
- `configs/models/acme/wsp3/normal/d1-fable/edits.wsp`, read as the example the brief names. Its
  comments quote train-partition facts of the spherical-flame investigation PSI-159 (lives at three
  pressures, a holding flow that does not move with pressure). Not BRE; they agree with a weak
  pressure effect and were one reason for taking `a` below the radiation-theory value.
- One line of `docs/wsp/research/exp_acme.md` returned by a search: BRE flames "self-extinguish within
  1.5 minutes" when large, and below about 25 % oxygen.
- The earlier author's `docs/wsp/research/astra_bre_w0.md`, the first half of `astra_bre_all_nasa.md`
  and the kernel `crates/ignis-wsp/src/property_bre_w0.rs`; `phys_quench_w0.md` for layout only.
- The author's pretraining may include published BRE results; that cannot be measured.
- Not opened: anything under `runs/` except the brief, `data/`, `holdout/`, any feedback file, the
  BRE experiment table, its Readme transcription, any raw file.

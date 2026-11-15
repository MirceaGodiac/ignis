# SPICE (PSI-107): the smoke point of a microgravity jet flame in coflow. A W0 law

Author: `claude-fable-5-2`, 15 November 2026. Kind: **W0, literal authoring**. No file under `runs/`
(other than the brief), `data/` or `holdout/` was opened, and of the feedback files only the
target and input sections of the engineer's feasibility note.
The published paper on this very experiment (Dotson et al. 2011) **was read**, as the brief allows;
every number taken from it is marked **L\*** and section 8.3 says exactly what it showed me. Nothing
was fitted. The only program run was a forward evaluation of the typed law to fill section 5.

## 0. The result in one screen

A jet flame emits soot when the soot made on the fuel side cannot be burnt before the tip cools.
In microgravity with a coflow, three things fix that point: the fuel (how fast it makes soot), the
coflow speed (it carries the soot through the flame faster) and the nozzle diameter (a small nozzle
gives a fast, thin jet). The law:

```text
smoke-point length   L_sp = max( L0[fuel] ,  A[fuel] (d / 1 mm)^-0.91 (u_air / 1 cm/s)^0.41 ) * dil(x_fuel)
luminous length      L    = K * mdot * F(r) / Z_st            K = C_f Sc / (4 pi mu) = 1.30 mm per mg/s
                     F(r) = 1 + 0.61 / (1 + r / 0.10)         r = u_air / u_fuel
smoke-point flow     mdot_sp = L_sp Z_st / (K F)
P(soot leaves the tip at a setting) = Phi( ln(L / L_sp) / 0.28 )
```

`A` = 21.2 (ethylene), 18.5 (propane), 6.05 mm (propylene) **L\***; `L0` = 21, 28, 7.5 mm (the
still-air smoke point: the first two read from the earlier shuttle experiment, the third estimated);
`dil` = 1, 1.26, 2.30 for 100, 75, 50 % propylene in nitrogen **L\***; methane has no smoke point
inside the duct. Width: 0.20 in ln (22 %) where the coflow term rules, 0.30 at the still-air floor.

What is mine and what is the paper's. The exponents and the three `A` are the published fit of the
flight data. From first principles I derive: the length law and its constant `K` (no flight number
in it); the still-air floor, which the published power law lacks (it gives zero length at zero
coflow); why the diameter exponent is negative and the coflow exponent positive, and inside which
brackets they must lie (section 1.4: theory alone gives the right signs, a diameter exponent between
-0.5 and -3 and a coflow exponent between 0 and 1, and cannot do better); which fuel and nozzle
pairs can have a smoke point at all inside a 120 mm duct; and the conversion of the smoke point into
a fuel flow, which is what the test table records.

## 1. Mechanism

### 1.1 Length: fuel flow over diffusion

A laminar jet flame ends where enough air has diffused to the axis. For a round jet the length is
proportional to the fuel flow and independent of the nozzle diameter (Burke and Schumann; Roper 1977):
the flow carries the fuel up as fast as diffusion brings air in, and a wider nozzle is slower by
exactly the factor by which it needs longer to mix. With equal diffusivities, constant properties
and a flame sheet (Spalding; Mahalingam et al.; Lin and Faeth 1999):

```text
strong coflow (uniform velocity):   L / d = C_f Re Sc / (16 Z_st)
still air (momentum jet):           L / d = 3 C_f Re Sc / (32 Z_st)        (1.5 times longer)
Re = 4 mdot / (pi d mu):            L = C_f Sc mdot / (4 pi mu Z_st)  times 1 or 1.5
```

`Z_st` is the stoichiometric mixture fraction of the nozzle gas, `mu` and `Sc` are taken for air at
the mean flame temperature (Lin and Faeth), `C_f` is the ratio of the luminous length to the
flame-sheet length: about 0.5 for a soot-free flame and 1.05 (coflow) or 1.13 (still air) for a
luminous flame near its smoke point, because soot keeps glowing past the flame sheet. The jet is
1.5 times longer in still air because a momentum jet stays narrow; the coflow takes over once the
jet's excess velocity on the axis, `u_f x0 / x` with `x0 = 3 Re d / 32`, has fallen to the coflow
speed. At the flame tip that excess is `0.12 u_f / F`, so the change happens around
`r = u_air / u_f = 0.1`:

```text
F(r) = 1 + 0.61 / (1 + r / r0),   r0 = 0.10        (0.61 = 1.5 * 1.13 / 1.05 - 1)
```

Dimensionless statement: `L Z_st mu / (C_f Sc mdot) = F(r) / (4 pi)`. Pressure, gravity and nozzle
diameter do not appear; that is why the same constant serves the shuttle flames in still air.

### 1.2 Soot: a residence time

Soot forms in the fuel-rich core over the time a parcel spends there and must burn out in the tip
before the gas, cooled by the soot's own radiation, drops below about 1300 K (1000 K is quoted for
microgravity). Both the soot load and the radiative cooling grow with the time on the axis,

```text
t_res = integral_0^L dx / u_cl(x),      u_cl = u_air + u_f min(1, x0 / x)
t_res = t_jet(d) * G(Lam),   t_jet = (3/64) (C_f Sc / Z_st)^2 rho_f d^2 / mu,   Lam = u_air L / (u_f x0)
G(Lam) = (2 / Lam) (1 - ln(1 + Lam) / Lam):   G -> 1 for a pure jet,  G -> 2 / Lam for strong coflow
```

Two limits. **Pure jet** (still air): `t_res = t_jet(d)`, proportional to `d^2` and independent of
the fuel flow. The time does not grow with the flame, which once led to the prediction that
microgravity jet flames have no smoke point. **Strong coflow**: `t_res = L / u_air`, so
`L_sp = u_air t_c`, with `t_c` a time that belongs to the fuel.

Orders of magnitude for ethylene (rho_f 1.16 kg/m3, mu 4.9e-5 Pa s, Z_st 0.0637):
`t_jet` = 0.51 s for the 1.6 mm nozzle, 0.115 s for 0.76 mm, 0.034 s for 0.41 mm. The soot time of
ethylene from its normal-gravity smoke point (106 mm, buoyant acceleration about 25 m/s2,
`t = sqrt(2 L / a)`) is 0.09 s; propane 0.11 s; propylene 0.05 s. So:

- on the 1.6 mm nozzle every fuel has far more time than it needs unless the coflow removes it:
  the smoke point is short and coflow-controlled;
- on the 0.41 mm nozzle ethylene and propane do not accumulate their soot time in the jet
  (0.034 s against 0.09 to 0.11 s): no smoke point, or one so long that the duct ends first;
  propylene (0.05 s needed, 0.05 s available with its higher density) is marginal: a long smoke
  point;
- the smaller the nozzle the longer the smoke point, steeply.

### 1.3 The floor: radiative quenching in still air

The shuttle experiment LSP (Urban et al. 2000) did find smoke points in still air, 15 to 63 mm,
with residence times 40 to 300 ms that were not constant and with almost no effect of the nozzle
diameter. Long, slow flames lose 40 to 60 % of their heat by radiation near the smoke point, the
tip opens, and soot escapes through the quenched tip. This sets a length `L0` below which the
smoke point does not fall when the coflow goes to zero. Read from the figures of that paper at
100 kPa and 1.6 mm: ethylene 21 mm, propane 28 mm; inversely proportional to pressure.

### 1.4 What theory gives and what it cannot

Setting `t_res = t_c` in 1.2 gives `L_sp = u_air t_c H(d)` with `H` falling from infinity (where
`t_jet(d) = t_c`) to 1 for large nozzles: a coflow exponent of exactly 1 and a diameter exponent
between -3 (near the divergence) and -0.5. Adding the floor, `L_sp = L0 + u_air t_c H(d)`, the
local coflow exponent is `u t_c H / (L0 + u t_c H)`: 0.2 at 5 cm/s rising to 0.7 at 65 cm/s for
ethylene on 1.6 mm, 0.45 on average. The flight fit found -0.91 and 0.41 **L\***. Both are inside
the brackets and the coflow exponent is the average of the two-term form; I take the published
numbers, because the residence-time closure cannot fix `H(d)` or `t_c` to better than a factor of
two (the authors of the experiment say the same: their residence times do not collapse the smoke
points). A check made after reading the fit: the slope `(L_sp(65 cm/s) - L0) / 65 cm/s` on the
1.6 mm nozzle is 87 ms for ethylene, 61 ms for propane, about 23 ms for propylene, against the
normal-gravity soot times 92, 114 and 48 ms: the same order and, but for propane, the same ranking.

### 1.5 Dilution and fuel

Nitrogen in the fuel lowers the soot rate twice: fewer fuel molecules per unit of gas (the mixing
rule `1 / L_sp = sum Y_i / L_sp,i` gives 1.22 and 1.67 for 75 and 50 % propylene) and a cooler core
(soot inception is steeply temperature dependent). The flight factors are 1.26 and 2.30 **L\***:
the first is the mixing rule, the second is 1.4 times above it. Fuel order by tendency to soot in
the flight fit: propylene, 75 %, 50 %, propane, ethylene; propane and ethylene are exchanged
against normal gravity, which the paper attributes to propane's lower exit velocity at the same
length (more time in the core). Methane: no smoke point before the flame reached the outlet plate
120 mm downstream.

### 1.6 The earlier kernel (`property_jets_v2.rs`), as a colleague's work

It marches a conserved scalar and axial momentum for a jet in coflow with a complete-reaction
energy balance and refuses any property outside its stated domain. As transport it is sound, and
its analytic check `L_s = -a^2 U / (4 D ln(1 - Z_s))` is the strong-coflow limit of 1.1. For this
investigation it has three gaps. It predicts none of the recorded quantities: its own text says no
lift-off, extinction, smoke point or luminous length is predicted, and its three typed constants
are pi, 2 and 4. The one number that decides the length, the diffusivity at flame temperature, is
left to the caller, so the kernel cannot be wrong and cannot forecast. And a stoichiometric tip is
not a luminous tip: the ratio is `C_f`, 0.5 to 1.1, and it is the soot that sets it. What I keep
from it: equal-speed transport as the reference limit, and the refusal of inputs outside a domain.

## 2. Targets

No feasibility note existed when the law was written; the targets were defined from the card. The
engineer's note `docs/wsp/feedback/spice_feasibility.md` appeared afterwards (I read its paragraph
for the physicist and its sections 1 and 2). It asks for two things, both on the 55 smoke points of
the investigators' own review sheet:

- **T1** `L_sp_mm`: the smoke-point length from the pre-test conditions. This is
  `smoke_point_length` below.
- **T2**: the same length when the nozzle gas flow at the smoke point is given. This is
  `flame_length` below, evaluated at that flow. At a smoke point the soot term should be zero; the
  network switches it on wherever the length law falls short of the fitted smoke-point length (up
  to 14 % off). For T2 alone, set `flame_length.median.w2[2]` and `w2[3]` to 0.

These 55 points are the ones the published fit was made on. **For T1 the law's centre is therefore
the investigators' own fit of the same data, not a blind forecast**; section 5.1 gives the numbers
I would have typed without the paper, so that a clean variant can be run. The other two targets
are mine and are kept as secondary (the note says the flame lengths at other flows and the soot
marks are not yet scorable).

| Target | Kind, unit | A forecast may know |
|---|---|---|
| `smoke_point_length`: luminous length, nozzle to tip on the axis, at incipient soot emission | positive, m | fuel, fuel fraction, nozzle diameter, coflow speed |
| `smoke_point_fuel_flow`: nozzle gas mass flow at that point | positive, kg/s | the same |
| `flame_length`: luminous length of the flame at a setting | positive, m | the same and the nozzle gas mass flow |
| `emits_soot`: at this setting soot leaves the tip (at or past the smoke point) | binary | the same and the nozzle gas mass flow |

For fire safety the first two matter (how large a gas-jet fire can be before it smokes). The
coflow speed is the calibrated duct anemometer reading; the fuel flow is the mass flow of
everything through the nozzle (fuel and nitrogen).

## 3. The law

```text
z_d = ln(d / 1 mm),  z_u = ln(u_air / 25 cm/s),  z_m = ln(mdot / 1 mg/s),  z_x = ln(x_fuel)

ln L_sp   = ln L0[f] + dil(z_x) + relu( ln(A[f] / L0[f]) + 1.333 - 0.91 z_d + 0.414 z_u )
dil(z_x)  = -0.82 z_x + 0.65 relu(-0.288 - z_x)                 (1.26 at 75 %, 2.30 at 50 %)
ln L      = ln K + ln mdot - ln Z_st[f, x] + ln F(r) + soot(q)
ln F(r)   = 0.148 * clip( ln(0.5 / r), 0, 3.22 )                (piecewise form of F in 1.1: 0 to 0.476)
soot(q)   = -0.14 * clip( q, 0, 1.10 ),  q = ln(L_sp / L)       (C_f falls to 0.90 far below the smoke point)
ln mdot_sp = ln L_sp + ln Z_st - ln K - ln F(r_sp)
logit P(emits_soot) = 6.1 * ( ln L - ln L_sp )                  (logistic form of Phi(. / 0.28); L without soot term)
```

Spread (one standard deviation in ln):

| Target | ln-sd | Why |
|---|---|---|
| `smoke_point_length` | 0.20 where the coflow term exceeds the floor by half an e-fold, rising to 0.30 at the floor | scatter of the published fit (R2 0.90 over lengths of roughly 8 to 110 mm), 5 to 10 % repeatability, cabin oxygen 20 to 23 % not recorded per test; the floor of propylene is my estimate |
| `smoke_point_fuel_flow` | 0.30 | the above and the length law (`C_f` 15 %, `mu` 10 %, `F` 10 %) |
| `flame_length` | 0.22 | `C_f` between soot-free and smoke-point values; the tip is a 2 to 5 mm fade |
| all three | 10 % of cases three times wider | a smoke point judged by eye in video; unsteady coflow at high fan settings |

`emits_soot` has no spread of its own: its slope 6.1 per e-fold is `1.7 / 0.28`, the combined
ln-sd of the smoke-point length (0.20) and of the length law (0.20).

Limits. `u_air -> 0`: the floor `L0`, not zero. `d` large: the coflow term falls below the floor
and the smoke point no longer depends on `d`, as in still air and in normal gravity. `x_fuel -> 0`:
`L_sp` grows without bound (no soot). `mdot -> 0`: `L -> 0`, never sooting. `r` large: `F = 1`,
Burke-Schumann. Range: 0.3 to 4 mm nozzles, 3 to 90 cm/s, 0.05 to 20 mg/s, fuel fraction 0.4 to 1,
air at 1 atm. Above about 115 mm the flame meets the outlet plate: a predicted smoke point longer
than that means "none observable".

## 4. Constants

Classes: **P** property table, **T** theory or derivation here, **L** published experiment,
**L\*** published paper on this experiment (Dotson et al. 2011), **J** judgement.

| Constant | Value | Uncertainty | Class | Origin |
|---|---|---|---|---|
| Viscosity of air at 1300 K, `mu` | 4.9e-5 Pa s | 10 % (choice of temperature) | P | Sutherland; mean of 300 K and the adiabatic 2300 K, as Lin and Faeth prescribe |
| Schmidt number | 0.76 | 5 % | L | Lin and Faeth 1999, 2000 |
| `C_f`, luminous flame near its smoke point, coflow | 1.05 | 15 % | L | Lin and Faeth 2000, low-pressure coflow flames of acetylene, propylene, butadiene |
| `C_f` in still air | 1.13 | 15 % | L | same, on the shuttle flames of Urban et al. 1998 |
| `C_f` far below the smoke point | 0.90 | 0.5 to 1.05 | J | soot-free flames were half as long in drop tests (Sunderland et al. 1999), but in the sister experiment SLICE blue 40 % methane flames and sooting pure methane flames fall on one line of length against methane flow (Cao et al. 2015); reached at one third of the smoke-point length |
| `K = C_f Sc / (4 pi mu)` | 1296 m per kg/s (1.30 mm per mg/s) | 20 % | T | 1.1; gives 20.3 mm per mg/s for ethylene in strong coflow |
| Jet factor over coflow | 1.61 | 1.4 to 1.7 | T, L | 3/2 (theory) times 1.13 / 1.05 |
| `r0`, velocity ratio at the change | 0.10 | 0.05 to 0.5 | T | 1.1; Lin and Faeth saw the still-air law hold "crudely" up to 0.5 |
| `Z_st` ethylene, propane, propylene, methane | 0.0637, 0.0603, 0.0637, 0.0552 | 1 % | P | stoichiometry in air |
| `Z_st` of 75 %, 50 % propylene in N2 | 0.0768, 0.1018 | 1 % | P | `d ln Z_st / d ln x = -0.66` |
| Gas density at 294 K, 1 atm: ethylene, propane, propylene, methane, nitrogen | 1.163, 1.828, 1.744, 0.665, 1.161 kg/m3 | 1 % | P | ideal gas |
| Diameter exponent `a` | -0.91 | theory bracket -3 to -0.5 | L\* | fit of 55 flight smoke points; sign and bracket T (1.4) |
| Coflow exponent `b` | 0.41 | theory bracket 0 to 1 | L\* | same; average of the two-term form T (1.4) |
| `A` ethylene, propane, propylene | 21.2, 18.5, 6.05 mm at 1 mm and 1 cm/s | 20 % | L\* | Table 2 of the paper |
| Dilution factor 75 %, 50 % propylene | 1.26, 2.30 | 15 % | L\* | ratios of `A` in Table 2; mixing rule gives 1.22, 1.67 (T) |
| `L0` ethylene, propane (still air, 1 atm) | 21, 28 mm | 3 mm (read from a figure) | L | Urban et al. 2000, 1.6 mm nozzle, shuttle |
| `L0` propylene | 7.5 mm | 5 to 11 mm | J | 1.9 times its coflow term at 1.6 mm and 1 cm/s; the ratio is 1.5 for ethylene, 2.3 for propane |
| Methane | `A`, `L0` 7.4 times ethylene's | at least 3 times | J | no smoke point in normal gravity nor in the duct (L\*: none before 120 mm) |
| Soot time from normal-gravity smoke points 106, 162, 29 mm | 92, 114, 48 ms | 30 % | L, T | Schug et al. 1980; `sqrt(2 L / a)`, `a` = 25 m/s2; used for 1.2 and 1.4 only |

## 5. Registered expectations

Written before any comparison with project data. Network form of section 7 evaluated on a grid.
Entries: smoke-point length in mm / nozzle gas flow at the smoke point in mg/s. One standard
deviation is 22 % on the length and 35 % on the flow. Lengths above 115 mm are beyond the duct.

| Fuel | d (mm) | 5 cm/s | 10 cm/s | 20 cm/s | 40 cm/s | 65 cm/s |
|---|---:|---:|---:|---:|---:|---:|
| Ethylene | 0.41 | 93 / 2.8 | 124 / 3.8 | 165 / 5.0 | 220 / 6.7 | 269 / 8.2 |
| Ethylene | 0.76 | 53 / 1.6 | 71 / 2.2 | 94 / 3.1 | 125 / 4.4 | 153 / 5.5 |
| Ethylene | 1.6 | 27 / 1.1 | 36 / 1.5 | 48 / 2.1 | 64 / 2.9 | 78 / 3.7 |
| Propane | 0.41 | 81 / 2.3 | 108 / 3.1 | 144 / 4.2 | 192 / 5.5 | 235 / 6.9 |
| Propane | 0.76 | 46 / 1.4 | 62 / 2.0 | 82 / 2.8 | 109 / 3.9 | 134 / 5.0 |
| Propane | 1.6 | 28 / 1.1 (floor) | 31 / 1.3 | 42 / 1.9 | 56 / 2.6 | 68 / 3.2 |
| Propylene | 0.41 | 27 / 0.81 | 35 / 1.1 | 47 / 1.5 | 63 / 2.1 | 77 / 2.7 |
| Propylene | 0.76 | 15 / 0.56 | 20 / 0.78 | 27 / 1.1 | 36 / 1.5 | 44 / 2.0 |
| Propylene | 1.6 | 7.7 / 0.37 | 10 / 0.50 | 14 / 0.67 | 18 / 0.89 | 22 / 1.1 |
| 75 % propylene | 0.41 | 34 / 1.2 | 45 / 1.7 | 60 / 2.2 | 79 / 3.1 | 97 / 3.9 |
| 75 % propylene | 0.76 | 19 / 0.80 | 26 / 1.1 | 34 / 1.6 | 45 / 2.2 | 55 / 2.8 |
| 75 % propylene | 1.6 | 9.7 / 0.54 | 13 / 0.75 | 17 / 1.0 | 23 / 1.4 | 28 / 1.7 |
| 50 % propylene | 0.41 | 61 / 2.9 | 81 / 3.9 | 108 / 5.2 | 144 / 7.0 | 176 / 8.5 |
| 50 % propylene | 0.76 | 35 / 1.7 | 46 / 2.4 | 62 / 3.4 | 82 / 4.8 | 101 / 6.1 |
| 50 % propylene | 1.6 | 18 / 1.2 | 24 / 1.6 | 31 / 2.3 | 42 / 3.2 | 51 / 4.0 |
| Methane | any | above 190 | | | | none inside the duct |

In words:

1. Ethylene and propane have no observable smoke point on the 0.41 mm nozzle above about 8 to
   12 cm/s coflow (the entry exceeds the duct), and none on 0.76 mm above about 30 to 45 cm/s.
   The 0.41 mm rows of these two fuels are an extrapolation of the fit: the published ranges of
   exit velocity show no smoke point of ethylene or propane on that nozzle.
2. Pure propylene is the only fuel with smoke points on all three nozzles over the whole range of
   coflow.
3. In still or reversed air (rows with zero or negative air speed) the smoke point is the floor:
   21 mm ethylene, 28 mm propane, 7.5 mm propylene, on any nozzle.
4. Flame length: 20 mm per mg/s of ethylene or propylene and 21.5 mm per mg/s of propane when
   `u_air / u_f` is above 0.5; up to 1.6 times that when the jet is fast (below 0.02); a flame at
   one third of its smoke-point flow is a further 14 % shorter. Methane: 22 mm at 1 mg/s and 78 mm
   at 3 mg/s on the 1.6 mm nozzle at 20 cm/s; it reaches the plate near 4.2 mg/s.
5. At a setting, soot leaves the tip with probability 0.5 when the predicted length equals the
   smoke-point length, 0.1 at 0.70 of it and 0.9 at 1.43 of it. Examples at 20 cm/s on 1.6 mm:
   ethylene 1, 2, 3 mg/s: 0.01, 0.43, 0.93; propylene 0.4, 0.8 mg/s: 0.04, 0.75.

### 5.1 Variant B: the smoke point without any number from the paper

What theory, the shuttle experiment and normal-gravity smoke points give by themselves, with the
exponents as quoted in the card. The two-term form of 1.4 at the centre of the flown range (1.6 mm,
25 cm/s), `L_sp = L0 + u t_c H`, with `H` from `G(Lam)` of 1.2 (1.38, 1.26, 1.13) and the soot
times 92, 114, 48 ms; propylene's floor scaled from ethylene's by the normal-gravity ratio
(21 mm x 29 / 106 = 5.7 mm); dilution by the mixing rule (1.22, 1.67):

| | Ethylene | Propane | Propylene |
|---|---:|---:|---:|
| `L_sp` at 1.6 mm, 25 cm/s, variant B | 52.7 mm | 63.9 mm | 19.3 mm |
| the same, main law (published fit) | 52.3 mm | 45.7 mm | 15.0 mm |
| `A` of variant B (mm at 1 mm, 1 cm/s) | 21.3 | 25.9 | 7.8 |

Variant B is the main law with these lines changed, and a flat width of 0.45:

```text
smoke_point_length.median.skip[0:7] = [0.0, 0.0, 0.0, -0.72, 0.287, -1.304, 2.0]
smoke_point_length.median.W1[0, 4] = -0.092
smoke_point_length.median.W1[0, 5] = 0.299
smoke_point_length.median.b1[0] = 1.347
smoke_point_length.median.w2[1] = 0.0
smoke_point_length.spread.b0 = 0.45
smoke_point_length.spread.w2[0] = 0.0
smoke_point_length.spread.w2[1] = 0.0
```

It says propane soots less readily than ethylene, as in normal gravity; the flight found the
opposite. That is the one place where the paper changed the physics and not just a number.

## 6. What would show the law is wrong

1. **The smoke point goes to zero with the coflow, or the 1.6 mm ethylene and propane flames at
   5 to 8 cm/s have smoke points well below 20 mm.** Then there is no radiative floor in this duct
   (the fan-off flow or the walls matter) and `L0` must go. Conversely, if `L_sp` against `u_air`
   at fixed fuel and nozzle is a straight line with a positive intercept and no curvature, the
   two-term form of 1.4 is the law and the exponent 0.41 is an average: the forecasts at 5 and at
   65 cm/s are then both too high by 10 to 20 % and those near 25 cm/s too low.
2. **Flame length is not proportional to the nozzle gas flow divided by `Z_st`, or the factor
   between a fast jet and a slow one is not there.** The law says 20 mm per mg/s (ethylene,
   propylene) at `u_air / u_f` above 0.5 and 28 to 33 at 0.02 to 0.05. A single constant for all
   velocity ratios would remove `F`; a constant outside 15 to 40 mm per mg/s would mean the property
   temperature (the 1300 K in `mu`) is wrong.
3. **Ethylene or propane smoke on the 0.41 mm nozzle at a length below 65 mm (the law gives 81 mm
   or more at any coflow from 5 cm/s up), or propylene has no smoke point on it.** That breaks the
   residence-time reading of the diameter effect (1.2): the nozzle would then act through something
   other than the time on the axis, and the exponent -0.91 could not be trusted outside the three
   diameters flown.

Calibration, separately: of the settings given a probability between 0.3 and 0.7 about half should
be sooting; if nearly all are, the smoke points marked in the table sit at lower flow than the
published ones (a different criterion: first reddening of the tip rather than soot leaving it).

## 7. Interface for the law forward (`docs/wsp/research/law_forward.md`)

One slug, `spice`, seven features, four targets. The two smoke-point targets carry weight zero on
`fuel_mass_flow` (typed below), so the flow of a row cannot inform them; the forward still needs
the condition to be present.

### 7.1 Features (declaration order)

| # | `name` | SI unit | transform | centre | scale | domain (raw) | Meaning |
|---|---|---|---|---|---|---|---|
| 0 | `nozzle_diameter` | m | ln | 0.001 | 1 | 0.0003 to 0.004 | inner diameter of the fuel nozzle |
| 1 | `coflow_speed` | m/s | ln | 0.25 | 1 | 0.03 to 0.9 | duct air speed (anemometer) |
| 2 | `fuel_mass_flow` | kg/s | ln | 0.000001 | 1 | 0.00000005 to 0.00002 | mass flow of all gas through the nozzle |
| 3 | `x_fuel` | 1 | ln | 1 | 1 | 0.4 to 1 | fuel mole fraction of the nozzle gas (balance nitrogen) |
| 4 | `is_propane` | 1 | identity | 0 | 1 | 0 to 1 | indicator |
| 5 | `is_propylene` | 1 | identity | 0 | 1 | 0 to 1 | indicator |
| 6 | `is_methane` | 1 | identity | 0 | 1 | 0 to 1 | indicator (ethylene is the reference fuel) |

From the case files: `nozzle_diameter` = `nozzle_id_mm` / 1000; `coflow_speed` = `u_air_cm_s` / 100;
`fuel_mass_flow` = `inputs_given_flow.mdot_jet_mg_s` / 1e6 (T2; for T1 any value inside the domain,
its weight is zero); `x_fuel`; the indicators from `hydrocarbon`. The note's `zst_jet`,
`rho_jet_g_m3` and `mu_air_mean_g_m_s` are not needed: the law carries its own property values
(section 4), which agree with the investigators' conventions.

A row with air speed below 3 cm/s (zero, negative, missing) is outside the domain and refused by
the forward; for such rows the statement of this document is the floor `L0` of section 4.

### 7.2 Targets (declaration)

| `name` | kind | unit | shared quantity |
|---|---|---|---|
| `smoke_point_length` | positive | m | none (lane-local) |
| `smoke_point_fuel_flow` | positive | kg/s | `mass_flow` |
| `flame_length` | positive | m | none (lane-local) |
| `emits_soot` | binary | | |

### 7.3 Tensors

Those of section 3 of the forward's document for 7 features: `x.<f>.centre|scale|lo|hi` (scalars),
and per network `b0` (scalar), `skip` [7], `W1` [8, 7], `b1` [8], `w2` [8]; networks
`<t>.median`, `<t>.spread` for the three positive targets with `<t>.tail_weight`, `<t>.tail_width`,
and `emits_soot.logit`. Units: medians in ln(m) or ln(kg/s), spreads in ln, logit dimensionless.
The values typed below are inside the forward's bounds (median b0 in [-80, 80], spread in
[0.001, 10], logit in [-40, 40]).

### 7.4 Literal assignments

```text
# spice.wsp: W0, zero-shot. Author claude-fable-5-2. Source: docs/wsp/research/phys_spice_w0.md
# features z = (z_d, z_u, z_m, z_x, f_propane, f_propylene, f_methane), columns 0..6
x.nozzle_diameter.centre = 0.001
x.nozzle_diameter.scale = 1.0
x.nozzle_diameter.lo = 0.0003
x.nozzle_diameter.hi = 0.004
x.coflow_speed.centre = 0.25
x.coflow_speed.scale = 1.0
x.coflow_speed.lo = 0.03
x.coflow_speed.hi = 0.9
x.fuel_mass_flow.centre = 0.000001
x.fuel_mass_flow.scale = 1.0
x.fuel_mass_flow.lo = 0.00000005
x.fuel_mass_flow.hi = 0.00002
x.x_fuel.centre = 1.0
x.x_fuel.scale = 1.0
x.x_fuel.lo = 0.4
x.x_fuel.hi = 1.0
x.is_propane.centre = 0.0
x.is_propane.scale = 1.0
x.is_propane.lo = 0.0
x.is_propane.hi = 1.0
x.is_propylene.centre = 0.0
x.is_propylene.scale = 1.0
x.is_propylene.lo = 0.0
x.is_propylene.hi = 1.0
x.is_methane.centre = 0.0
x.is_methane.scale = 1.0
x.is_methane.lo = 0.0
x.is_methane.hi = 1.0

# --- smoke-point length. b0 = ln(0.021 m), the still-air floor of ethylene (Urban 2000).
# skip: dilution -0.82 per e-fold of fuel fraction; floors of propane (28 mm), propylene (7.5 mm),
# methane (7.4 times ethylene) relative to ethylene.
smoke_point_length.median.b0 = -3.863
smoke_point_length.median.skip[0:7] = [0.0, 0.0, 0.0, -0.82, 0.287, -1.030, 2.0]
# unit 0: the coflow term above the floor. 1.342 = ln(21.2 mm * 25^0.414 / 21 mm); exponents
# -0.91 (diameter) and 0.414 (coflow), Dotson 2011; fuel offsets ln(A/L0) relative to ethylene.
smoke_point_length.median.W1[0, 0] = -0.91
smoke_point_length.median.W1[0, 1] = 0.414
smoke_point_length.median.W1[0, 4] = -0.423
smoke_point_length.median.W1[0, 5] = -0.224
smoke_point_length.median.b1[0] = 1.342
smoke_point_length.median.w2[0] = 1.0
# unit 1: dilution below 75 % fuel is steeper (cooler core): 2.30 at 50 % against 1.26 at 75 %.
smoke_point_length.median.W1[1, 3] = -1.0
smoke_point_length.median.b1[1] = -0.288
smoke_point_length.median.w2[1] = 0.65
# spread: 0.30 at the floor, 0.20 once the coflow term is half an e-fold above it.
smoke_point_length.spread.b0 = 0.30
smoke_point_length.spread.skip[0:7] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
smoke_point_length.spread.W1[0, 0] = -0.91
smoke_point_length.spread.W1[0, 1] = 0.414
smoke_point_length.spread.W1[0, 4] = -0.423
smoke_point_length.spread.W1[0, 5] = -0.224
smoke_point_length.spread.b1[0] = 1.342
smoke_point_length.spread.w2[0] = -0.2
smoke_point_length.spread.W1[1, 0] = -0.91
smoke_point_length.spread.W1[1, 1] = 0.414
smoke_point_length.spread.W1[1, 4] = -0.423
smoke_point_length.spread.W1[1, 5] = -0.224
smoke_point_length.spread.b1[1] = 0.842
smoke_point_length.spread.w2[1] = 0.2
smoke_point_length.tail_weight = 0.1
smoke_point_length.tail_width = 3.0

# --- smoke-point fuel flow = L_sp Z_st / (K F). b0 = -3.863 + ln(0.0637) - ln(1296).
# skip: dilution of L_sp (-0.82) and of Z_st (-0.66); Z_st of propane and methane.
smoke_point_fuel_flow.median.b0 = -13.784
smoke_point_fuel_flow.median.skip[0:7] = [0.0, 0.0, 0.0, -1.48, 0.232, -1.030, 1.857]
smoke_point_fuel_flow.median.W1[0, 0] = -0.91
smoke_point_fuel_flow.median.W1[0, 1] = 0.414
smoke_point_fuel_flow.median.W1[0, 4] = -0.423
smoke_point_fuel_flow.median.W1[0, 5] = -0.224
smoke_point_fuel_flow.median.b1[0] = 1.342
smoke_point_fuel_flow.median.w2[0] = 1.0
smoke_point_fuel_flow.median.W1[1, 3] = -1.0
smoke_point_fuel_flow.median.b1[1] = -0.288
smoke_point_fuel_flow.median.w2[1] = 0.65
# units 2, 3: the jet factor F at the smoke point. y = ln(0.5 / r_sp) for F = 1; ln F =
# 0.1289 clip(y, 0, 3.695) once the effect of F on r is solved (0.148 / 1.148).
smoke_point_fuel_flow.median.W1[2, 0:7] = [-2.91, -0.586, 0.0, -1.75, -0.643, -1.659, 2.416]
smoke_point_fuel_flow.median.b1[2] = 2.158
smoke_point_fuel_flow.median.w2[2] = -0.1289
smoke_point_fuel_flow.median.W1[3, 0:7] = [-2.91, -0.586, 0.0, -1.75, -0.643, -1.659, 2.416]
smoke_point_fuel_flow.median.b1[3] = -1.537
smoke_point_fuel_flow.median.w2[3] = 0.1289
smoke_point_fuel_flow.spread.b0 = 0.30
smoke_point_fuel_flow.spread.skip[0:7] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
smoke_point_fuel_flow.tail_weight = 0.1
smoke_point_fuel_flow.tail_width = 3.0

# --- luminous flame length at a setting = K mdot F / Z_st, shorter far below the smoke point.
# b0 = ln(1296) + ln(1e-6) - ln(0.0637): 20.3 mm at 1 mg/s of ethylene in strong coflow.
flame_length.median.b0 = -3.895
flame_length.median.skip[0:7] = [0.0, 0.0, 1.0, 0.66, 0.055, 0.0, 0.143]
# units 0, 1: ln F = 0.148 clip(ln(0.5 / r), 0, 3.219); ln(0.5 / r) = 0.784 at the centre.
flame_length.median.W1[0, 0:7] = [-2.0, -1.0, 1.0, -0.27, -0.452, -0.405, 0.559]
flame_length.median.b1[0] = 0.784
flame_length.median.w2[0] = 0.148
flame_length.median.W1[1, 0:7] = [-2.0, -1.0, 1.0, -0.27, -0.452, -0.405, 0.559]
flame_length.median.b1[1] = -2.435
flame_length.median.w2[1] = -0.148
# units 2, 3: less soot, shorter glow: -0.14 per e-fold below the smoke point, at most -0.15.
flame_length.median.W1[2, 0:7] = [-0.91, 0.414, -1.0, -1.48, -0.191, -1.254, 1.857]
flame_length.median.b1[2] = 1.374
flame_length.median.w2[2] = -0.14
flame_length.median.W1[3, 0:7] = [-0.91, 0.414, -1.0, -1.48, -0.191, -1.254, 1.857]
flame_length.median.b1[3] = 0.274
flame_length.median.w2[3] = 0.14
flame_length.spread.b0 = 0.22
flame_length.spread.skip[0:7] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
flame_length.tail_weight = 0.1
flame_length.tail_width = 3.0

# --- soot leaves the tip: logit = 6.1 (ln L - ln L_sp), L without the soot term.
emits_soot.logit.b0 = -0.195
emits_soot.logit.skip[0:7] = [0.0, 0.0, 6.1, 9.03, -1.415, 6.283, -11.33]
emits_soot.logit.W1[0, 0:7] = [-2.0, -1.0, 1.0, -0.27, -0.452, -0.405, 0.559]
emits_soot.logit.b1[0] = 0.784
emits_soot.logit.w2[0] = 0.903
emits_soot.logit.W1[1, 0:7] = [-2.0, -1.0, 1.0, -0.27, -0.452, -0.405, 0.559]
emits_soot.logit.b1[1] = -2.435
emits_soot.logit.w2[1] = -0.903
emits_soot.logit.W1[2, 0] = -0.91
emits_soot.logit.W1[2, 1] = 0.414
emits_soot.logit.W1[2, 4] = -0.423
emits_soot.logit.W1[2, 5] = -0.224
emits_soot.logit.b1[2] = 1.342
emits_soot.logit.w2[2] = -6.1
emits_soot.logit.W1[3, 3] = -1.0
emits_soot.logit.b1[3] = -0.288
emits_soot.logit.w2[3] = -3.965
```

Approximations made to fit the form (each below 8 % on the median, checked against the implicit
law on the grid of section 5): `F` is piecewise linear in `ln r`; inside the hinges that need the
smoke-point length (the jet factor at the smoke point, the soot term) the coflow line is used
without its floor and without the extra dilution hinge, because a hinge cannot hold another hinge.

What the forward cannot express here: the still-air case (no logarithm of zero; refused), and the
duct limit of 115 mm (a predicted smoke point beyond it should be scored as "none observed", which
is a rule for the scorer, not a tensor).

## 8. Sources and limits

### 8.1 Sources

- K. T. Dotson, P. B. Sunderland, Z.-G. Yuan, D. L. Urban, "Laminar smoke points of coflowing
  flames in microgravity", Fire Safety Journal 46 (2011) 550-555. **This experiment.** Taken: the
  exponents -0.910 and 0.414, `A_f` for five fuel mixtures, the fuel ranking, duct geometry (76 mm
  square, outlet plate 120 mm downstream), smoke-point criterion, cabin oxygen 20 to 23 %,
  measurement uncertainties, the normal-gravity smoke points of Schug et al.
- D. L. Urban et al., "Smoke-point properties of nonbuoyant round laminar jet diffusion flames",
  Proc. Combust. Inst. 28 (2000) 1965-1972 (NASA reprint, scanned). Taken: still-air smoke points
  of ethylene and propane at 100 kPa read from its figures 3 and 4; residence times 40 to 302 ms,
  lengths 15 to 63 mm; inverse proportionality to pressure; tip opening.
- K.-C. Lin and G. M. Faeth, "Flame shapes of luminous nonbuoyant laminar coflowing jet diffusion
  flames" (NASA workshop paper, 2000; journal version AIAA J. 37 (1999) 759-765). Taken: the length
  laws, `C_f` 1.05 and 1.13, `Sc` 0.76, properties of air at the mean flame temperature, the 3/2
  ratio, the validity ranges in velocity ratio.
- F. G. Roper, Combust. Flame 29 (1977) 219-226 and 227-234 (from memory): length proportional to
  flow, the buoyant velocity `sqrt(2 a z)`.
- P. B. Sunderland et al., Combust. Flame 116 (1999) 376-386 (from memory): soot-free lengths about
  half the luminous ones.
- S. Cao et al., Proc. Combust. Inst. 35 (2015) 897-903 (SLICE, the sister experiment in the same
  duct): luminous length proportional to methane mass flow whatever the dilution; used for `C_f`
  far below the smoke point only.
- K. P. Schug et al., Combust. Sci. Technol. 22 (1980) 235-250: 106, 162, 29 mm (quoted in Dotson).
- The card `docs/wsp/atlas/cards/PSI-107.json` and the section of `INVESTIGATIONS.md`.

### 8.2 Not found or not checked

- The individual smoke points of the flight (only the fit and the ranges per fuel).
- The oxygen fraction of each test; the sensitivity of the smoke point to it.
- A still-air smoke point of propylene in microgravity.
- Whether the "air rate" column of the table is the anemometer reading used in the paper's fit.
- The virtual origin of the length law (neglected; a few nozzle diameters, so the shortest flames
  on the 1.6 mm nozzle carry the largest relative error).
- Roper's and Sunderland's numbers were not re-read.

### 8.3 Exposure

Before deriving anything I had read the card, whose findings quote the exponents -0.91 and 0.41,
the fuel order and "length proportional to fuel mass flow". The derivations of 1.1 and 1.2 (length
law, `t_jet`, `G`) were written before the paper was opened; 1.4, the choice of the exponents and
of `A` came after. From the paper I saw the fit, the per-fuel ranges of fuel flow, exit velocity
and Reynolds number at the smoke points (used only to check that the length law and the fit agree:
they do within 10 %), and two figures as text fragments. I saw no per-test value. This forecast of
the smoke-point length is therefore **not blind to the published summary of the outcome**; the
length law, the floor, the fuel-flow conversion and the probability of soot at a setting are mine.
From the engineer's note I read the paragraph for the physicist and sections 1, 2 and 4 (targets,
inputs, file list); they hold no outcome. Its sections 3 and 5 to 7 were not read.

# SAME-R (PSI-101): how big is the smoke of an overheated material, and does the ISS detector see it. A W0 law

Author: claude-fable-5-2, physicist-author of the seventh generation, 15 November 2026. Zero-shot: written
from the card, the engineer's note on targets and inputs, and the literature. No file under `runs/`, `data/`
or `holdout/` was opened. What I saw of outcomes is listed in section 8.3, with what it changed (nothing typed).

Siblings: [phys_same_w0.md](phys_same_w0.md) (SAME, the first flight, with dibutyl phthalate),
[phys_daft_w0.md](phys_daft_w0.md) (DAFT, the particle counter on cabin air and test dust).

## 0. The result in one screen

A wire heats a few milligrams of a spacecraft material below ignition. The vapour of the decomposition
products condenses into particles within millimetres of the sample; a piston draws the smoke into an aging
chamber, where it is stirred, held, and pushed through the instruments. Two things set the size that is
measured:

1. **the condensation size `D_n`**, a property of the material's vapour and of how fast it is cooled and
   diluted: about 0.12 um for Kapton and Teflon (solid, barely volatile fragments: many nuclei, small
   particles, narrow distribution) and 0.25 to 0.31 um for lamp wick, Pyrell and silicone (semi-volatile
   tars and oils that keep condensing on the particles already there: fewer, larger, broad);
2. **Brownian coagulation**, which at conserved mass makes the mean particle volume grow linearly in time:

```text
D30(t)^3 = D_n^3 + (6 K / pi) * phi * t          or          D30(t) = D_n (1 + t / t_c)^(1/3),   t_c = 1 / (K N_0)
```

   with `K` = 6.0e-16 m3/s the coagulation coefficient of 0.1 to 0.5 um particles in cabin air, `phi` the
   volume of smoke per volume of air and `N_0` the number concentration when aging starts. `t_c` is 400 s
   (Kapton) to 1400 s (silicone) for the smoke these tests made.

The count mean diameter that the engineer scores is `d10 = D30 exp(-ln^2 sigma_g)`; coagulation also
narrows the distribution, so `d10` grows a little faster than `D30`: as `(1 + t/t_c)^0.41`.

Forecast medians of `d10` (section 5): at 8.1 cm/s and 40 s of aging, Kapton 94 nm, Teflon 104 nm, lamp
wick 166 nm, Pyrell 191 nm, silicone 205 nm; times 1.5 (lamp wick) to 2.0 (Kapton) after 1900 s; times
1.42 at 2 cm/s; times 1.8 to 2.1 with no flow during heating. One standard deviation is a factor 1.5
(ln-sd 0.40), a factor 1.6 to 1.7 for long aging and no flow.

The ISS detector measures light scattered forward at 30 degrees from a near-infrared beam. Per unit mass
of smoke that signal goes as the cube of the particle size below about 0.7 um and is flat above. Kapton
and Teflon smokes give about 1/10 and 1/40 of the signal of the same mass of lamp wick smoke. Probability
that the scatter channel is at full scale, at 8.1 cm/s and short aging: lamp wick 0.75, silicone 0.70,
Pyrell 0.68, Kapton 0.15, Teflon 0.10; higher for Kapton and Teflon at low flow, no flow and long aging.

## 1. Mechanism from first principles

### 1.1 The chain of events and its time scales

| Stage | Where | Time | What happens to the particles |
|---|---|---|---|
| Pyrolysis | sample on the wire, 290 to 560 C | 60 s of heating | vapour of decomposition products leaves the surface at a rate that is Arrhenius in the sample temperature |
| Nucleation and condensation | thermal boundary layer, about 1 mm thick at 8 cm/s | 10 to 100 ms | the vapour is cooled by hundreds of kelvin; its saturation ratio becomes enormous; particles nucleate and take up the vapour |
| Plume | duct, a few cm | about 1 s | too short and too dilute for coagulation (1.3) |
| Aging chamber | litres, stirred | 20 to 1900 s | Brownian coagulation at conserved mass |
| Measurement | counter (M0), ionization chamber (M1), DustTrak (M3), ISS and Shuttle detectors | seconds | the moments of the size distribution are read |

### 1.2 Dimensionless groups

| Group | Definition | Values here | Decides |
|---|---|---|---|
| Coagulation number | `Co = K N_0 t = t / t_c` | 0.03 (40 s) to 5 (1900 s, Kapton) | growth by aging: `D30 ~ (1 + Co)^(1/3)`, number `~ 1/(1 + Co)` |
| Knudsen number | `Kn = 2 lambda / d`, `lambda` = 67 nm | 1.3 at 0.1 um, 0.27 at 0.5 um | slip: `K` is 2.4 times its continuum value at 0.1 um, 1.3 times at 0.5 um |
| Peclet number of the vapour at the sample | `Pe = U d_s / D_v` | 25 at 8.1 cm/s, 6 at 2 cm/s, 0 with no flow (sample 3 mm, `D_v` about 0.1 cm2/s) | whether the flow or diffusion carries the vapour away: below `Pe` about 3 the flow stops mattering |
| Richardson number of the hot sample | `Ri = g beta dT d_s / U^2` | 0 in orbit; about 5 on the ground at 8 cm/s | on the ground the buoyant plume (about 20 cm/s) replaces the duct flow near the sample |
| Optical size | `x = pi d / lambda_light` | 0.4 (0.1 um) to 2.6 (0.7 um) for a near-infrared beam | Rayleigh scattering (`~ d^6` per particle) below `x` about 1, saturating above |
| Stokes settling number | `v_s t / H` | 0 in orbit; 0.7 on the ground for 1 um, 1900 s, 10 cm | no loss of the large particles in orbit |

### 1.3 Coagulation: the part that is theory

For particles that stick on contact, `dN/dt = -K N^2` and the volume of smoke per volume of air `phi` is
conserved. The mean particle volume `v = phi / N` therefore obeys `d(1/N)/dt = K`, that is

```text
v(t) = v_0 + K phi t        D30^3 = D_n^3 + (6 K / pi) phi t        N(t) = N_0 / (1 + K N_0 t)
```

`D30 = (6 phi / (pi N))^(1/3)` is the diameter of average mass. The law holds across a dilution (both
`phi` and `N` scale, `v` does not), so stages with different concentrations add: the growth of `D30^3`
is `6K/pi` times the time integral of `phi`.

The coefficient. For equal spheres in the continuum `K = 4 k T C_c / (3 mu)`: with `T` = 296 K and
`mu` = 1.83e-5 Pa s this is 2.98e-16 m3/s times the slip factor `C_c` (1.89 at 0.2 um, 1.58 at 0.3 um,
2.9 at 0.1 um), times the Fuchs transition factor (0.88 at 0.1 um, 0.96 at 0.3 um), times
`(1 + exp(ln^2 sigma_g)) / 2` for a lognormal distribution (Lee 1983: 1.12 at `sigma_g` 1.6, 1.26 at 1.9).
That gives 7.8e-16 (Kapton, 0.115 um), 6.1e-16 (lamp wick), 5.7e-16 m3/s (silicone); fractal agglomerates
(Teflon, Pyrell) collide about 1.5 times faster than spheres of the same mass. I type one value,
`K = 6.0e-16 m3/s`, and carry the material differences in `t_c`.

Orders of magnitude. A smoke of 10 mg/m3 of unit density has `phi` = 1e-8. Then `(6K/pi) phi t` =
1.15e-23 m3 per second: after 800 s it equals (210 nm)^3. A smoke born at 200 nm reaches 260 nm; one born
at 115 nm reaches 220 nm. So aging matters most for the small-particle smokes, and after long aging all
smokes of the same mass concentration converge to the same size, whatever they started from:
`D30 -> ((6K/pi) phi t)^(1/3)`. This is the main prediction that owes nothing to a material constant.

In the plume, by contrast: the aerosol mass rate is of order 1e-9 kg/s (10 mg/m3 in a few litres over
60 s), the plume carries about 4 cm3/s, so `phi` is about 2e-7 for about 1 s: `(6K/pi) phi t` =
(6 nm)^3. Coagulation in the plume is negligible with flow, and the size at the chamber inlet is the
condensation size.

With no flow during heating the smoke stays in about 64 cm3 around the sample (class L, the card) at about
50 times the concentration of a flow test, for the 60 s of heating, and half of it is lost through the
inlet (class L, the card). The dose `phi t` of that stage equals that of `50 * 0.5 * 60 s / 2` = 750 s in
the chamber (the concentration rises linearly, hence the half). I type `T_c` = 720 s: a no-flow test is,
for coagulation, a flow test aged 12 minutes longer.

### 1.4 The distribution narrows

Lee's moment solution for a lognormal distribution under Brownian coagulation:
`9 ln^2 sigma = ln[2 + (exp(9 ln^2 sigma_0) - 2) / (1 + Co)]`, tending to `sigma_g` = 1.32. Over
`Co` = 0.1 to 10 the decrease of `ln^2 sigma` is close to `a ln(1 + Co)` with `a` = 0.10 for a broad smoke
(`sigma_0` = 1.9) and 0.06 for a narrow one (1.6); I type 0.08. With the Hatch-Choate relations for a
lognormal distribution of geometric mean `d_g`:

```text
count median   d_g  = D30 exp(-1.5 ln^2 sigma)        count mean   d10 = D30 exp(-ln^2 sigma)
mass median    MMD  = D30 exp(+1.5 ln^2 sigma)        scattering   d_sc = (M6/M3)^(1/3) = D30 exp(+3 ln^2 sigma)
```

so that `ln d10 = ln D_n - ln^2 sigma_0 + (1/3 + 0.08) ln(1 + t/t_c)`. The count median grows as the
power 0.45, the mass median as 0.21: aging "removes the small particles" more than it makes large ones.

### 1.5 The condensation size: the part that is judgement

Theory gives its direction and its scaling, not its value. The vapour flux `J` from the surface fills a
boundary layer of thickness `delta ~ U^(-1/2)`, so the vapour concentration there goes as `J U^(-1/2)` and
the cooling rate as `U`. A nucleation burst fed at a rate `R` makes a number of particles that rises less
than linearly in `R` (about `R^0.6` for growth by condensation in the free-molecule regime); the mass per
volume divided by that number gives

```text
D_n ~ J^(0.15 to 0.3) * U^(-0.25)
```

I type `p` = 0.25 for the flow exponent between 8.1 and 2 cm/s and let it saturate below 2 cm/s (at
`Pe` of a few, diffusion takes over; the no-flow tests then add the coagulation of 1.3). For the
temperature: 50 K raise the pyrolysis rate by a factor of about 3 (activation energies of 120 to 200
kJ/mol give 6 to 10; the sample lags the wire and runs out of volatile matter), hence `D_n` by
3^0.2 = 1.25: `s_T` = 0.20 per 50 K, and the chamber concentration by 3, which shortens `t_c`.

Why the materials differ. Teflon and Kapton decompose above 500 C into gases (tetrafluoroethylene; CO,
CO2) and a few per cent of fragments of very low volatility: the saturation ratio at the first millimetre
is astronomical, nucleation is violent, the condensable is spent on a great number of nuclei, and the
particles are solid, so collisions build agglomerates (Teflon) and the distribution is narrow. Lamp wick
(cellulose tar), silicone rubber (siloxane oils) and the polyurethane foam release semi-volatile liquids
over a range of boiling points: fewer nuclei form, the vapour keeps condensing on them along the plume, and
plume regions of different history mix: larger particles, broad distribution. The typed values:

| Material | `D_n` (D30, nm) | `sigma_g` at birth | `t_c` (s) | Reference temperature `T_ref` (K) | Why |
|---|---|---|---|---|---|
| Kapton | 115 | 1.6 | 400 | 830 | polyimide, decomposes at 550 to 600 C; solid nuclei |
| Teflon | 130 | 1.6 | 500 | 790 | PTFE fume: primaries of tens of nm, agglomerated within the first minute |
| Lamp wick | 250 | 1.9 | 1000 | 560 | smouldering cotton: count median 0.1 to 0.2 um, mass median 0.3 to 0.5 um in ground tests |
| Pyrell | 270 | 1.8 | 1200 | 570 | polyurethane: isocyanate and polyol fragments |
| Silicone | 310 | 1.9 | 1400 | 670 | siloxane oils; the lightest cyclics stay in the gas |

`t_c` from 1.3 with a chamber smoke of 5 mg/m3 (Kapton, Teflon), 10 (Pyrell) and 15 mg/m3 (lamp wick,
silicone) gives 290, 455, 1070, 1450 and 1790 s. The SAME card reports that the number fell by a factor
1.7 to 2.7 in 800 s for the spacecraft materials (class L), that is `t_c` = 470 to 1140 s. I typed values
between the two: the theory's ordering, pulled a third of the way towards the published range.

### 1.6 Gravity

On the ground the hot sample drives a plume of about `sqrt(g beta dT d_s)` = 20 cm/s. The effective speed
at the sample is then about 21 cm/s instead of 8.1, and by the flow law the particles are smaller by
`(21/8.1)^0.25` = 1.27 at 8.1 cm/s, 1.4 at 5 cm/s, 1.8 at 2 cm/s. Without buoyancy the smoke also stays
near the source and coagulates there (the no-flow tests are the extreme). This is a transfer statement:
every SAME-R flight test is in microgravity and gravity is not an input of the law.

### 1.7 The detectors

**Forward scattering (ISS).** A particle much smaller than the wavelength scatters as `d^6`; per unit mass
of smoke the signal is `M6/M3 = d_sc^3`. Above `x` about 3 the scattering per unit mass falls as `1/d`. A
form with both limits:

```text
S_ISS / mass  ~  opt * u^3 / (1 + u^4),     u = d_sc / d*,    d* = 0.70 um  (peak at d_sc = 0.92 um)
opt = ((m^2 - 1)/(m^2 + 2))^2 / rho
```

`d*` from a peak of the mass scattering efficiency at `d` about 1.1 wavelengths for forward angles and a
beam of 0.85 um (the wavelength is my recollection of a near-infrared laser diode, class J).
`d_sc = D30 exp(3 ln^2 sigma)` is 0.9 um for lamp wick (broad) and 0.23 um for Kapton (small and narrow):

| Material | `d_sc` (um) | `u^3/(1+u^4)` relative to lamp wick | `opt` relative (index, density) | Signal per mass, relative |
|---|---|---|---|---|
| Lamp wick | 0.89 | 1.00 | 1.00 (1.50, 1.2) | 1.00 |
| Silicone | 1.10 | 0.96 | 0.84 (1.40, 0.97) | 0.81 |
| Pyrell | 0.79 | 0.96 | 1.17 (1.55, 1.2) | 1.12 |
| Kapton | 0.24 | 0.067 | 1.33 (1.70, 1.42) | 0.09 |
| Teflon | 0.27 | 0.098 | 0.27 (1.35, 2.2) | 0.026 |

For the three large-particle smokes the signal per mass sits on the flat top: nothing but the mass of
smoke changes it. For Kapton and Teflon it goes as the cube of the size, so everything that makes the
particles larger (aging, low flow, no flow, a hotter sample) raises it. Coagulation raises `d_sc` slowly:
exactly, for a constant kernel, `d_sc^3 = d_sc0^3 + 2 (6K/pi) phi t`, a gain of 14 % for Kapton in 800 s
and of 1 % for lamp wick.

**Ionization (Shuttle, and the ionization chamber that gives M1).** The current drop goes as the first
moment `M1 = N d10`; per unit mass that is `1 / (rho D30^2 exp(ln^2 sigma))`. Small particles are
favoured as the square of the size: Kapton smoke gives about 4 times the signal of the same mass of lamp
wick smoke, and aging lowers the signal as `(1 + t/t_c)^(-0.59)`. The two detectors therefore move in
opposite directions with aging. The engineer found the Shuttle detector in line on a minority of tests
and without a conversion of its volts, so it is not a target; the statement stands for a later check.

**The event.** Target 2 is "the scatter channel is at full scale". With the chamber mass concentration
`c` and a full-scale level `c_fs` for well-sized smoke, `P = Phi((ln c + ln(signal per mass) - ln c_fs) / s)`.
I typed the probabilities for `c` of 5 to 15 mg/m3 (section 1.5) against a level of about 4 mg/m3, with
`s` = 2.0 in ln (a factor 3 on `c`, 3 on the level, 3.3 on the size term). They were first typed as
"reaches its alarm level" with an assumed level of 5 mg/m3; the engineer's note then defined the event as
full scale and quoted the alarm specification (1 % per foot, of order 0.5 mg/m3). Full scale some ten
times above an alarm is the same level as I had assumed, so the numbers stand unchanged (section 8.3).

## 2. The forecast targets (from the engineer's note)

`docs/wsp/feedback/samer_feasibility.md`, sections "For the physicist", 3 and 4.

| Target | Definition | Unit | Score | A forecast may know |
|---|---|---|---|---|
| `d10` (primary) | count mean diameter M1/M0 of the row, at its `net_aging_time` | m | quantile CRPS of ln(d10), levels 5/25/50/75/95 % | material, primary or aged row, inlet velocity, Rt/R0 set point, heating duration, aging and mixing durations, net aging time (the horizon), sample temperature during heating, assumed particle density; nothing measured on the smoke of the row |
| `iss_full_scale` (optional) | ISS detector scatter channel at full scale, \|V\| >= 6.0 V | yes / no | Brier | the same |
| `d30` (mine, not scored by the note) | diameter of average mass (6 M3 / (pi rho M0))^(1/3), in the outcome files as `d30_m` | m | as `d10` if the engineer wishes | the same |

`mass_loss` is weighed after the flight and is not used. `net_aging_time` is the time the forecast is for.
The optional `prior` of aged rows (the primary measurement of the same run) is used only by the separate
law of section 7.5.

## 3. The law

```text
inputs: material m, sample temperature T (K), inlet velocity U (m/s), net aging time t (s), heating time t_h (s)

f(U)     = 0.25 ln(8.1 cm/s / max(U, 2 cm/s))                    0 at 8.1, 0.35 at 2, 1 and 0 cm/s
ln D_n   = ln D_ref[m] + 0.20 (T - T_ref[m]) / 50 K + f(U) + 0.10 ln(t_h / 60 s)
t_c      = t_c[m] * exp(-0.5 (T - T_ref[m]) / 50 K)
Co       = (t + 720 s * [U = 0]) / t_c

ln D30   = ln D_n + (1/3)  ln(1 + Co)
ln d10   = ln D_n - ln^2 sigma_0[m] + 0.413 ln(1 + Co)
ln-sd    = 0.40 (d10), 0.38 (D30);  + 0.06 with no flow;  + 0.03 per e-fold of t beyond 270 s
tails    = 10 % of the weight in a component 2.5 times wider
```

`P(full scale) = logistic(L)`, with `L` at 8.1 cm/s, `T_ref`, short aging: lamp wick +1.10, silicone
+0.85, Pyrell +0.75, Kapton -1.73, Teflon -2.20; `+0.6` per 50 K (three times the smoke; the cube of the
size for the small smokes is inside the uncertainty of `T_ref`); `+0.10` per cm/s below 8.1 (more
concentrated) and a further `+0.9` at 2 cm/s for Kapton and Teflon (the cube of 1.42); no flow: `-0.6`
(half the smoke lost) and `+0.8` for Kapton and Teflon (coagulated); long aging: `+0.3` per e-fold of
`t` beyond 270 s for Kapton and Teflon; `+0.5` per e-fold of heating time.

How unsure the theory is. The ln-sd of 0.40 is made of: the condensation size of the material 0.33
(a factor 1.4, judgement), the reference temperature 0.15 (40 K), the width `sigma_0` 0.10, test-to-test
repeatability 0.10. Long aging adds the uncertainty of `t_c` (a factor 2.2) times `(1/3) Co/(1+Co)`: 0.18
at `Co` = 2. Most of this error is common to all tests of one material: the law may be right in every
ratio (aging, flow, temperature) and wrong by a constant factor per material. The first revision should
look at that before anything else.

## 4. Constants

Class: P property table, T theory, L published about this experiment, J judgement.

| Symbol | Value | Class | Uncertainty | Origin |
|---|---|---|---|---|
| `k T` | 4.09e-21 J at 296 K | P | 1 % | cabin temperature 22 to 24 C |
| `mu` air | 1.83e-5 Pa s | P | 1 % | at 296 K |
| `lambda` air | 67 nm | P | 3 % | at 101 kPa, 296 K |
| Slip factor | `1 + Kn (1.257 + 0.4 exp(-1.1/Kn))` | P | 2 % | Cunningham, Davies constants |
| `K` | 6.0e-16 m3/s | T | 25 % | 1.3; equal spheres 0.2 to 0.3 um, slip, transition, `sigma_g` 1.6 to 1.9 |
| Exponent of `D30` on `1 + Co` | 1/3 | T | exact at conserved mass | 1.3 |
| Narrowing coefficient `a` | 0.08 | T | 0.06 to 0.10 | Lee's solution, 1.4 |
| `D_ref` | 115, 130, 250, 270, 310 nm | J | factor 1.4 each | 1.5; ground tests of smouldering cotton and PTFE fume for two of them |
| `sigma_0` | 1.6, 1.6, 1.9, 1.8, 1.9 | J, with L | 0.15 | narrow Kapton and Teflon, broad silicone and lamp wick: SAME card (L); values mine |
| `t_c` | 400, 500, 1000, 1200, 1400 s | T, with L | factor 2.2 | 1.5: theory 290 to 1790 s, published SAME range 470 to 1140 s |
| `T_ref` | 830, 790, 560, 570, 670 K | P/J | 40 K | decomposition temperatures of polyimide, PTFE, cellulose, polyurethane, silicone rubber |
| `s_T` | 0.20 per 50 K | T/J | 0.15 | 1.5 |
| Shift of `ln t_c` with temperature | -0.5 per 50 K | T/J | 0.4 | three times the mass, 1.8 times the particle volume |
| `p` (flow) | 0.25 | T/J | 0.15 | 1.5 |
| Flow floor | 2 cm/s | J | 1 to 3 cm/s | `Pe` of a few |
| `T_c` (no flow) | 720 s | T, with L | factor 2 | 64 cm3 and half lost (card, L); 50 times, 60 s |
| Heating-time exponent | 0.10 | J | 0.10 | more smoke; one validation run only differs from 60 s |
| `d*` | 0.70 um | T/J | 0.15 um | 1.7; wavelength from memory |
| Refractive indices, densities | 1.70/1420 (Kapton), 1.35/2200 (Teflon), 1.50/1200 (cellulose tar), 1.55/1200 (polyurethane), 1.40/970 (silicone oil) kg/m3 | P | 0.05 / 10 % | polymer tables |
| Chamber smoke `c` | 5, 5, 15, 10, 15 mg/m3 | J | factor 3 | 1 to 2 mg/min lost (plan of SAME, the card), yield of a few per cent, litres of air |
| Full-scale level | 4 mg/m3 of well-sized smoke | J | factor 3 | 1.7 |
| Logit values | section 3 | J | 1.0 each | from the three lines above |

## 5. Registered expectations

Medians. `d10` and `D30` in nm; ln-sd is the core width (add the tail of section 3). `T = T_ref` of the
material, heating 60 s. Written before any comparison; computed by hand calculator from the literals of 7.4.

| Material | Flow cm/s | d10 at 40 s | d10 at 750 s | d10 at 1900 s | D30 at 40 s | D30 at 1900 s | ln-sd 40 / 1900 s | P full scale, 40 s | P full scale, 1900 s |
|---|---|---|---|---|---|---|---|---|---|
| Lamp wick | 8.1 | 166 | 214 | 260 | 250 | 360 | 0.40 / 0.46 | 0.75 | 0.75 |
| Lamp wick | 2 or 1 | 235 | 304 | 369 | 355 | 510 | 0.40 / 0.46 | 0.85 | 0.85 |
| Lamp wick | 0 | 297 | 337 | 404 | 429 | 549 | 0.46 / 0.52 | 0.79 | 0.79 |
| Kapton | 8.1 | 94 | 144 | 190 | 116 | 206 | 0.40 / 0.46 | 0.15 | 0.24 |
| Kapton | 2 | 133 | 205 | 269 | 165 | 292 | 0.40 / 0.46 | 0.45 | 0.59 |
| Kapton | 0 | 192 | 260 | 315 | 222 | 332 | 0.46 / 0.52 | 0.62 | 0.74 |
| Teflon | 8.1 | 104 | 156 | 199 | 130 | 219 | 0.40 / 0.46 | 0.10 | 0.17 |
| Teflon | 2 | 148 | 221 | 282 | 184 | 311 | 0.40 / 0.46 | 0.33 | 0.47 |
| Teflon | 0 | 217 | 281 | 340 | 251 | 361 | 0.46 / 0.52 | 0.50 | 0.64 |
| Silicone | 8.1 | 205 | 248 | 300 | 310 | 421 | 0.40 / 0.46 | 0.70 | 0.70 |
| Silicone | 2 or 1 | 291 | 352 | 426 | 440 | 598 | 0.40 / 0.46 | 0.81 | 0.81 |
| Silicone | 0 | 369 | 406 | 467 | 532 | 644 | 0.46 / 0.52 | 0.74 | 0.74 |
| Pyrell | 8.1 | 191 | 238 | 289 | 270 | 377 | 0.40 / 0.46 | 0.68 | 0.68 |
| Pyrell | 2 or 1 | 271 | 338 | 410 | 383 | 534 | 0.40 / 0.46 | 0.80 | 0.80 |
| Pyrell | 0 | 343 | 383 | 449 | 463 | 575 | 0.46 / 0.52 | 0.72 | 0.72 |

Kapton and Teflon at 1 cm/s: sizes as at 2 cm/s, P 0.51 and 0.39 (40 s), 0.65 and 0.54 (1900 s).
Core quantiles of one case (Kapton, 8.1 cm/s, 40 s): 49, 72, 94, 123, 181 nm at 5, 25, 50, 75, 95 %.

Temperature, at 8.1 cm/s and 40 s, `d10` in nm / P: 40 K below `T_ref`, at it, 40 K above:
lamp wick 141 / 0.65, 166 / 0.75, 194 / 0.83; Kapton 79 / 0.10, 94 / 0.15, 114 / 0.22; Teflon 89 / 0.06,
104 / 0.10, 126 / 0.15; silicone 175 / 0.59, 205 / 0.70, 241 / 0.79; Pyrell 163 / 0.57, 191 / 0.68,
224 / 0.77. Heating for 120 s instead of 60 s: `d10` times 1.07.

Ratios that do not depend on the material constants being right:

| Statement | Forecast |
|---|---|
| Number concentration after 800 s of aging over that at the start | 1/(1 + 800/t_c): 0.33 (Kapton) to 0.64 (silicone) |
| `d10` after 1900 s over `d10` at 40 s, 8.1 cm/s | 2.0 Kapton, 1.9 Teflon, 1.57 lamp wick, 1.5 Pyrell and silicone |
| `d10` with no flow over `d10` at 8.1 cm/s, short aging | 2.0 to 2.1 Kapton and Teflon, 1.8 lamp wick, Pyrell, silicone |
| The same after 1900 s | 1.65 to 1.7 Kapton and Teflon, 1.55 the others: the gap closes |
| `sigma_g` after 1900 s | Kapton and Teflon 1.4, lamp wick and silicone 1.75 |
| Scatter signal per mass, Kapton over lamp wick | 0.09 (0.03 to 0.3) |
| Ionization signal per mass, Kapton over lamp wick | 4 (2 to 8) |

## 6. What would show the law is wrong

1. **Aging does not follow the mean-volume law.** For runs with a primary and an aged measurement,
   `(d10 aged / d10 primary)^(1/0.413) - 1` should be proportional to the measured number concentration
   times the elapsed time, with a coefficient within a factor 1.5 of 6e-16 m3/s, the same for the liquid
   smokes. A growth that does not scale with number times time, or a coefficient three times off, means
   the growth is not coagulation (condensation of vapour from the walls, loss of the small particles to
   the walls and the fan, or a dilution I do not know of).
2. **The no-flow tests are not the largest, or the materials do not converge.** The law says the smoke
   made with no flow is larger than at any flow for every material, and that Kapton and Teflon gain more
   than the liquid smokes. No-flow particles equal to or smaller than the 2 cm/s ones, or a gain of the
   same factor for all materials, would say that the confined stage is not a coagulation stage.
3. **The scatter detector does not rank the smokes by `d_sc`.** If Kapton or Teflon tests reach full scale
   as often as lamp wick and silicone at 8.1 cm/s and short aging, or if for Kapton the event is not more
   frequent at low flow, no flow and long aging than at 8.1 cm/s and short aging, the cube law of 1.7 is
   wrong for this instrument (or the smoke mass differs between conditions by more than I allowed).

Also telling, but weaker: a median `d10` of a material outside a factor 2 of its line in section 5 (the
judged constant is wrong, the structure may still hold); `d10` falling with temperature.

## 7. Interface for the strict forward (`docs/wsp/research/law_forward.md`)

### 7.1 Declaration for the slug `samer`

Nine features, in this order. The four material indicators and `no_flow` are derived by the engineer from
`material` and `inlet_velocity` (the forward reads no text; 1 when true, 0 otherwise; lamp wick is the
reference with all four at 0). The others are case inputs under their own names.

| # | Name | Unit | Transform | Centre | Scale | Domain | Meaning |
|---|---|---|---|---|---|---|---|
| 0 | `is_kapton` | 1 | identity | 0 | 1 | 0 to 1 | material indicator |
| 1 | `is_teflon` | 1 | identity | 0 | 1 | 0 to 1 | |
| 2 | `is_silicone` | 1 | identity | 0 | 1 | 0 to 1 | |
| 3 | `is_pyrell` | 1 | identity | 0 | 1 | 0 to 1 | |
| 4 | `sample_temperature` | K | identity | 673 | 50 | 450 to 950 | sample temperature during heating; `z = (T - 673)/50` |
| 5 | `inlet_velocity` | m/s | identity | 0.081 | 0.01 | 0 to 0.12 | `z` = cm/s below or above 8.1 |
| 6 | `no_flow` | 1 | identity | 0 | 1 | 0 to 1 | 1 when the inlet velocity is 0 |
| 7 | `net_aging_time` | s | ln | 100 | 1 | 5 to 5000 | the horizon; `z = ln(t / 100 s)` |
| 8 | `heating_duration` | s | ln | 60 | 1 | 20 to 300 | `z = ln(t_h / 60 s)` |

Targets: `d10` (positive, m, lane-local), `d30` (positive, m, lane-local), `iss_full_scale` (binary).
A row without a sample temperature or a net aging time is refused by the forward; that is the honest
answer for it.

### 7.2 How the closed form becomes the network

`N(z) = b0 + skip . z + sum_j w2[j] relu(b1[j] + W1[j,:] . z)`.

- The material constants and the reference temperatures fold into `b0` and the material weights:
  `b0 = ln D_ref[lamp wick] - ln^2 sigma_0 - 0.20 z_Tref[lamp wick]`, and so on.
- `f(U)` is piecewise linear in the velocity through its values at 8.1, 5, 2 and 0 cm/s: a slope and two
  hinges (units 0 and 1). It equals the power law at 8.1, 5 and 2 cm/s and is flat below 2.
- `ln(1 + Co)` is a soft-plus of `y = ln Co`, which is linear in the features. It is typed as four hinges
  (units 3 to 6): `0.2 relu(y + 2.5) + 0.3 relu(y + 0.9) + 0.3 relu(y - 0.9) + 0.2 relu(y - 2.5)`, within
  0.08 of the soft-plus everywhere (0.03 in ln of the diameter).
- No flow: `1 + (t + T_c)/t_c = (1 + T_c/t_c)(1 + t/(t_c + T_c))` exactly. The first factor is a constant
  per material: 0.19 in ln D30 for the three large smokes and 0.31 for Kapton and Teflon (unit 2 adds the
  difference); the second is a shift of `y` by -0.7 (it is -0.5 to -1.0 by material).

### 7.3 Tensors

Those of `law_forward.md` section 3 for nine features: `x.<f>.centre`, `.scale`, `.lo`, `.hi` (scalars);
per network `b0` (scalar), `skip` [9], `W1` [8, 9], `b1` [8], `w2` [8]; `d10.median` and `d30.median` in
ln(m), `d10.spread` and `d30.spread` in ln units, `tail_weight`, `tail_width`; `iss_full_scale.logit`.

### 7.4 Literal assignments

```text
# samer.wsp W0: zero-shot law of docs/wsp/research/phys_samer_w0.md. Author claude-fable-5-2.
# features: 0 is_kapton, 1 is_teflon, 2 is_silicone, 3 is_pyrell, 4 sample_temperature, 5 inlet_velocity,
#           6 no_flow, 7 net_aging_time, 8 heating_duration. Lamp wick is the reference material.
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
x.is_pyrell.centre = 0.0
x.is_pyrell.scale = 1.0
x.is_pyrell.lo = 0.0
x.is_pyrell.hi = 1.0
x.sample_temperature.centre = 673.0
x.sample_temperature.scale = 50.0
x.sample_temperature.lo = 450.0
x.sample_temperature.hi = 950.0
x.inlet_velocity.centre = 0.081
x.inlet_velocity.scale = 0.01
x.inlet_velocity.lo = 0.0
x.inlet_velocity.hi = 0.12
x.no_flow.centre = 0.0
x.no_flow.scale = 1.0
x.no_flow.lo = 0.0
x.no_flow.hi = 1.0
x.net_aging_time.centre = 100.0
x.net_aging_time.scale = 1.0
x.net_aging_time.lo = 5.0
x.net_aging_time.hi = 5000.0
x.heating_duration.centre = 60.0
x.heating_duration.scale = 1.0
x.heating_duration.lo = 20.0
x.heating_duration.hi = 300.0

# d10, count mean diameter, ln(m). b0: lamp wick, D_n 250 nm, sigma_0 1.9, T_ref 560 K.
d10.median.b0 = -15.162
# material (condensation size, width, reference temperature), 0.20 per 50 K, flow slope per cm/s,
# no flow (coagulation in the confined smoke), aging enters through the hinges, heating time
d10.median.skip[0:9] = [-1.666, -1.383, -0.225, 0.104, 0.20, -0.039, 0.236, 0.0, 0.10]
# units 0, 1: the flow law steepens below 5 cm/s and is flat below 2 cm/s
d10.median.W1[0, 5] = -1.0
d10.median.b1[0] = -3.1
d10.median.w2[0] = 0.0373
d10.median.W1[1, 5] = -1.0
d10.median.b1[1] = -6.1
d10.median.w2[1] = -0.0763
# unit 2: with no flow Kapton and Teflon gain more (shorter coagulation time)
d10.median.W1[2, 0] = 1.0
d10.median.W1[2, 1] = 1.0
d10.median.W1[2, 6] = 1.0
d10.median.b1[2] = -1.0
d10.median.w2[2] = 0.149
# units 3 to 6: 0.413 ln(1 + t/t_c); row = ln(t/t_c): t_c 1000 s lamp wick, 400 Kapton, 500 Teflon,
# 1400 silicone, 1200 Pyrell at T_ref, shorter by exp(-0.5) per 50 K, no flow shifts by -0.7
d10.median.W1[3:7, 0:9] = [[-1.783, -1.607, -1.436, -0.282, 0.5, 0.0, -0.7, 1.0, 0.0],
                           [-1.783, -1.607, -1.436, -0.282, 0.5, 0.0, -0.7, 1.0, 0.0],
                           [-1.783, -1.607, -1.436, -0.282, 0.5, 0.0, -0.7, 1.0, 0.0],
                           [-1.783, -1.607, -1.436, -0.282, 0.5, 0.0, -0.7, 1.0, 0.0]]
d10.median.b1[3:7] = [1.327, -0.273, -2.073, -3.673]
d10.median.w2[3:7] = [0.0826, 0.124, 0.124, 0.0826]
# spread: factor 1.5; wider with no flow and for long aging (beyond 270 s)
d10.spread.b0 = 0.40
d10.spread.skip[0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.06, 0.0, 0.0]
d10.spread.W1[0, 7] = 1.0
d10.spread.b1[0] = -1.0
d10.spread.w2[0] = 0.03
d10.tail_weight = 0.10
d10.tail_width = 2.5

# d30, diameter of average mass, ln(m): the same law without the width term, exponent 1/3
d30.median.b0 = -14.750
d30.median.skip[0:9] = [-1.857, -1.574, -0.225, 0.037, 0.20, -0.039, 0.19, 0.0, 0.10]
d30.median.W1[0, 5] = -1.0
d30.median.b1[0] = -3.1
d30.median.w2[0] = 0.0373
d30.median.W1[1, 5] = -1.0
d30.median.b1[1] = -6.1
d30.median.w2[1] = -0.0763
d30.median.W1[2, 0] = 1.0
d30.median.W1[2, 1] = 1.0
d30.median.W1[2, 6] = 1.0
d30.median.b1[2] = -1.0
d30.median.w2[2] = 0.12
d30.median.W1[3:7, 0:9] = [[-1.783, -1.607, -1.436, -0.282, 0.5, 0.0, -0.7, 1.0, 0.0],
                           [-1.783, -1.607, -1.436, -0.282, 0.5, 0.0, -0.7, 1.0, 0.0],
                           [-1.783, -1.607, -1.436, -0.282, 0.5, 0.0, -0.7, 1.0, 0.0],
                           [-1.783, -1.607, -1.436, -0.282, 0.5, 0.0, -0.7, 1.0, 0.0]]
d30.median.b1[3:7] = [1.327, -0.273, -2.073, -3.673]
d30.median.w2[3:7] = [0.0667, 0.100, 0.100, 0.0667]
d30.spread.b0 = 0.38
d30.spread.skip[0:9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.06, 0.0, 0.0]
d30.spread.W1[0, 7] = 1.0
d30.spread.b1[0] = -1.0
d30.spread.w2[0] = 0.03
d30.tail_weight = 0.10
d30.tail_width = 2.5

# ISS scatter channel at full scale. b0: lamp wick at T_ref (logit 1.10 there).
iss_full_scale.logit.b0 = 2.456
iss_full_scale.logit.skip[0:9] = [-6.07, -6.06, -1.57, -0.47, 0.6, -0.10, -0.6, 0.0, 0.5]
# unit 0: Kapton and Teflon gain the cube of the size at low flow (1 at 2 cm/s)
iss_full_scale.logit.W1[0, 0] = 2.0
iss_full_scale.logit.W1[0, 1] = 2.0
iss_full_scale.logit.W1[0, 5] = -0.164
iss_full_scale.logit.b1[0] = -2.0
iss_full_scale.logit.w2[0] = 0.9
# unit 1: and with no flow
iss_full_scale.logit.W1[1, 0] = 1.0
iss_full_scale.logit.W1[1, 1] = 1.0
iss_full_scale.logit.W1[1, 6] = 1.0
iss_full_scale.logit.b1[1] = -1.0
iss_full_scale.logit.w2[1] = 0.8
# unit 2: and with aging beyond 270 s
iss_full_scale.logit.W1[2, 0] = 2.0
iss_full_scale.logit.W1[2, 1] = 2.0
iss_full_scale.logit.W1[2, 7] = 1.0
iss_full_scale.logit.b1[2] = -3.0
iss_full_scale.logit.w2[2] = 0.3
```

Check values for whoever wires it (lamp wick, 560 K, 8.1 cm/s, 40 s, 60 s): `z` = (0, 0, 0, 0, -2.26, 0,
0, -0.916, 0); `ln Co` = -3.219, so the four aging units are off; `ln d10` = -15.162 - 0.452 = -15.614,
`d10` = 166 nm; logit = 2.456 - 1.356 = 1.10. Kapton, 830 K, 0 cm/s, 1900 s: `z` = (1, 0, 0, 0, 3.14,
-8.1, 1, 2.944, 0); `d10` = 315 nm, P = 0.74.

### 7.5 An optional second law for aged rows that are given their primary measurement

When the evaluator hands a frozen law the `prior` of an aged row, the forecast needs no material constant:

```text
ln d10(t2) = ln d10(t1) + 0.413 ln(1 + K N_1 (t2 - t1)),     K = 7e-16 m3/s
```

`N_1` is the number concentration measured on the primary row (chamber value, 1/m3). `K` is a little
above the generic 6e-16 because half the aged rows are agglomerates or small particles. This needs its own
slug (the forward refuses a case that lacks a declared feature, and primary rows have no prior), say
`samer_aged`, with three features: `prior_d10` (m, ln, centre 2e-7, scale 1, domain 2e-8 to 2e-6),
`prior_number_concentration` (1/m3, ln, centre 1e12, scale 1, domain 1e9 to 1e15), `net_aging_time`
(s, ln, centre 1000, scale 1, domain 100 to 5000); one target `d10` (positive, m).

```text
# ln d10 = ln(2e-7) + z0 + 0.413 softplus(y),  y = ln(7e-16 * 1e12 * 1000 * 0.96) + z1 + z2 = -0.40 + z1 + z2
d10.median.b0 = -15.425
d10.median.skip[0:3] = [1.0, 0.0, 0.0]
d10.median.W1[0:4, 0:3] = [[0.0, 1.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 1.0]]
d10.median.b1[0:4] = [2.10, 0.50, -1.30, -2.90]
d10.median.w2[0:4] = [0.0826, 0.124, 0.124, 0.0826]
# 0.12: K to 35 %, repeatability of M1/M0; the tail is for a dilution between the two measurements
d10.spread.b0 = 0.12
d10.spread.skip[0:3] = [0.0, 0.0, 0.0]
d10.tail_weight = 0.20
d10.tail_width = 3.0
```

The factor 0.96 stands for `(t2 - t1)/t2`. If the smoke kept for the secondary aging was diluted (the
input `secondary_volume_percent` may mean that; I could not tell), the growth is smaller than this law
says: that is what the wide tail is for, and it is the first thing to check on the train days.

## 8. Sources, limits, exposure

### 8.1 Sources

- Card `docs/wsp/atlas/cards/PSI-101.json` and PSI-102, PSI-47; `docs/wsp/atlas/INVESTIGATIONS.md`,
  sections of the three investigations; `docs/wsp/feedback/samer_feasibility.md`, the sections "For the
  physicist", 3 and 4 only.
- Coagulation, slip, transition: W. C. Hinds, *Aerosol Technology*, 2nd ed., chapters 3 and 12;
  S. K. Friedlander, *Smoke, Dust and Haze*, 2nd ed., chapter 7; N. A. Fuchs, *The Mechanics of Aerosols*.
- Lognormal moment solution: K. W. Lee, J. Colloid Interface Sci. 92 (1983) 315; K. W. Lee and H. Chen,
  Aerosol Sci. Technol. 3 (1984) 327. Quoted from memory; the closed form of 1.4 was re-derived for its
  limits (`sigma` unchanged at `Co` = 0, 1.32 at infinity), not checked against the paper.
- Hatch-Choate relations: Hinds, chapter 4.
- Sizes of smouldering-cotton smoke and of PTFE fume in ground tests: from memory of G. W. Mulholland's
  chapter on smoke in the SFPE Handbook and of the polymer-fume literature; not re-read for this note.
- About this experiment (class L, abstracts and summaries only; the full texts could not be opened):
  G. W. Mulholland et al., "Pyrolysis smoke generated under low-gravity conditions", Aerosol Sci. Technol.
  49 (2015) 310 (abstract: diameter of average mass larger by a factor 1.7, or 1.9 in the NASA copy, with
  no flow; agglomerate and spherical smokes); M. E. Meyer et al., "Smoke characterization and feasibility
  of the moment method for spacecraft fire detection", Aerosol Sci. Technol. 49 (2015) 299 (abstract);
  M. E. Meyer et al., "Evaluation of spacecraft smoke detector performance in the low-gravity environment",
  Fire Safety J. 98 (2018) 74 (through a press summary: the ISS detector saw the large particles of
  cellulose, silicone and Pyrell and often missed Teflon and Kapton; the ionization detector did slightly
  better on Teflon); D. L. Urban et al., AIAA 2009, "Properties of smoke from overheated materials in
  low gravity" (abstract).

### 8.2 What I could not find or check

- The geometry: duct cross-section, chamber volume, where the smoke goes between heating and
  measurement, whether the aged smoke is diluted. I assumed a piston-drawn flow, a few litres, no
  dilution. The engineering drawings are in the archive; I did not open it.
- The mass of smoke per test. It sets `t_c` and the detector event; my 5 to 15 mg/m3 is an estimate.
- The ISS detector's wavelength, its scattering angle and the level of full scale in mass terms.
- The wire alloy. From the range of Rt/R0 (1.22 to 1.48) and decomposition temperatures I infer a
  temperature coefficient of about 9e-4 per K (a stainless steel); the law uses the sample temperature
  NASA derived, not the ratio, and my `T_ref` assume that this temperature is the sample's.
- Ground-test sizes for Kapton, Pyrell and silicone smokes outside this programme: none found.
- The full texts of the three papers (publisher and repository refused the fetch).

### 8.3 Exposure: what I saw of outcomes, and what it changed

- **The cards** (allowed) carry NASA's summary findings: sizes mostly 100 to 600 nm; larger with aging
  and lower flow; no flow: 64 cm3, 50 times the concentration, diameter of average mass 30 to 40 % larger
  in 11 tests, half the smoke lost; liquid and agglomerate smokes; from SAME: count mean diameter below
  300 nm and about 50 % larger than on the ground; number down by 1.7 to 2.7 and count mean diameter up
  about 1.5 times in 800 s; narrow Kapton and Teflon, broad silicone and lamp wick. Used, as class L: the
  64 cm3, the 50 times and the half lost (`T_c`); the 1.7 to 2.7 (to temper `t_c`); narrow and broad (the
  grouping of `sigma_0`). The size range was known to me when I typed `D_ref`.
- **Abstracts and one press summary** (section 8.1): the factor 1.7 to 1.9 with no flow, and which
  materials the ISS detector missed. They confirmed the ordering I had derived (small Kapton and Teflon);
  they did not set a number.
- **The engineer's note**, read after every constant above had been typed and the table of section 5
  computed. Its target section states, for the train days: full scale "Teflon never, silicone, Pyrell and
  lamp wick always, Kapton 2 of 8"; mass concentrations of 2 to 350 mg/m3; one run with `d30` below `d10`;
  and the horizons. These are outcomes of train tests. I changed no number after reading them: the
  probabilities of section 3 are those typed before, moved from "alarm" to "full scale" by the argument
  of 1.7, and `t_c` keeps my 5 to 15 mg/m3 although the note's range says the smoke was often denser
  (which would mean faster coagulation than I forecast). Whoever revises this law should treat those two
  facts as already seen by the author.
- **Pretraining.** I may have read the published SAME tables in training. I cannot quote one of their
  numbers, but the values of `D_ref` are not provably independent of them.

### 8.4 The earlier author's kernel (`crates/ignis-wsp/src/property_aerosol_v2.rs`)

What it is: a sectional solver for coalescing spheres in a stirred chamber: slip-corrected diffusion, the
Fuchs collision kernel, settling, wall loss through a film, a source line, and an extinction detector for
Rayleigh-size spheres. Its seven literals are universal constants (Boltzmann's constant, three slip
coefficients, three domain limits).

What is right: the collision physics is the textbook one and agrees with the `K` used here; the
drift-diffusion wall flux is the correct boundary-value solution; number and volume budgets close.

Why it cannot forecast SAME-R:

1. **It has no law for where the smoke comes from.** The source is `IndependentlySupplied` (a number rate
   and a particle volume) or `Unknown`. The initial size and number are the whole difficulty of this
   experiment; with them supplied, the rest is section 1.3 in one line. The engineer's note confirms the
   files hold no source yield.
2. **Coagulation of a measured distribution does not need sections.** At conserved mass the mean volume
   is linear in `phi t` for any distribution; the kernel integrates in time what has a closed form, and
   asks for bin edges, step limits and wall faces that no file provides.
3. **Two of five materials are outside its support.** It requires compact coalescing spheres; Teflon and
   Pyrell smokes are agglomerates.
4. **Its detector is not one that flew.** It computes collimated extinction with `x <= 0.1`, that is
   particles below 27 nm for a 0.85 um beam. The ISS detector measures forward scattering from particles
   of `x` = 0.4 to 3; the Shuttle detector and the M1 instrument are ionization chambers, absent from the
   kernel.
5. **It states no uncertainty.** Evidence gates admit or refuse an input; the output is a point value.
   A forecast scored by CRPS needs a width, and here the width is the science (a factor 1.5 from the
   condensation size alone).
6. **It is not a typed model.** No spec, no patch of physical numbers an author could be wrong about;
   nothing in it could be revised by the typed step of `PROTOCOL.md` 12.3.

What I would keep from it for a later forward: the transition-regime kernel, if a test ever needs the
size dependence of `K` (smokes far below 0.1 um), and the film flux for wall loss on the ground.

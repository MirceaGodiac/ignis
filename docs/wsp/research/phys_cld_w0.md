# ACME CLD Flame (PSI-21): when a dilute coflow flame is lost, and how much it radiates. A W0 law

Author: `claude-fable-5-2`, 15 November 2026. Kind: **W0, literal authoring, with two stated
exceptions**. No file under `runs/` (other than the brief), `data/` or `holdout/` was opened, and
of the feedback files only the target and input sections of the engineer's note. Those sections
state three aggregates of train outcomes (how many flames were lost and in which stage, and the
range of the radiometer reading in volts). I saw them; section 8.3 says what each was used for. In
short: the mechanism and every flame number are from outside the experiment; the level of the
base hazard, the choice of one constant inside its theoretical range, and the volt scale of the
radiometer are **not blind**. Nothing was fitted. The only program run was a forward evaluation of
the typed law to fill section 5.

## 0. The result in one screen

A dilute flame on this burner is held or lost by one comparison: the coflow speed against the
laminar burning speed of the fuel mixture with air. Dilution lowers the burning speed steeply, so a
flame that is diluted during a test is lost when its burning speed has fallen to about a fifth of
the coflow speed.

```text
stability number   Phi = 0.66 U_co / (1.4 S_L(x_fuel))          lifted above 1, lost above 2.5
loss clock         median time = window * 2.5 / Phi(final conditions), log-logistic, shape 3.4
other losses       4 % of flames in 120 s, front-loaded
radiometer         V = c_V * Q_fuel * chi,   chi = 0.05 + chi_soot x_fuel^2.5   (chi_soot 0.15 methane, 0.35 ethylene)
```

`S_L` = 0.38 m/s for methane and 0.68 m/s for ethylene, falling to 0.21, 0.15, 0.08, 0.04 m/s at
40, 30, 20, 15 % methane and to 0.30, 0.21, 0.11 m/s at 20, 15, 10 % ethylene. `Q_fuel` is the
heat release of the fuel flow: 134 W per m/s of fuel-stream speed for pure methane on this tube,
220 W for ethylene, times the fuel fraction. A flame is lost with probability one half when the
coflow reaches 5.3 times the burning speed at its final dilution: 0.80 m/s at 30 % methane,
0.41 m/s at 20 %, 0.19 m/s at 15 %.

## 1. Mechanism

### 1.1 The base of the flame, as in SLICE

The flame ends at its base in a small premixed kernel that burns towards the fresh mixture at
about the laminar burning speed `S_L` and is carried away at the local gas speed `v_k`. The group
is `Da_k = S_L / v_k`. From the detailed computation of a butane jet flame in coflow (Takahashi et
al. 2022): `v_k = 0.66 U_co` plus, on the ground, a buoyant draught of 0.30 m/s; the flame leaves
the rim at `v_k = 1.4 S_L`. The derivation, the jet term (negligible on this 2.18 mm tube at the
speeds of this experiment: below a tenth of the coflow term) and the checks are in
`phys_slice_w0.md`, sections 1.1 to 1.4. In orbit:

```text
Phi = 0.66 U_co / (1.4 S_L)        lifted when Phi > 1, that is U_co > 2.1 S_L
```

Without the buoyant 0.30 m/s a flame holds at dilutions where on the ground it is already lifted
or gone: 40 % methane has `1.4 S_L` = 0.29 m/s, below the ground draught alone. This is the
published summary of the experiment: microgravity extends the dilution limits and allows lifted
flames at lower fuel concentrations.

### 1.2 Lifted is not lost: the coflow here is a jet

In the SLICE duct the air stream is uniform, and a flame that leaves the rim finds no slower gas
downstream; I put blow-off there at `Phi` = 1.3. Here the air comes from a 25 mm annulus into a
chamber 0.4 m across: the coflow is itself a jet that slows and spreads, and the mixing layer
between fuel and air thickens with height. A lifted base moves up until the local speed has
fallen to its propagation speed, and it can do so over a wide range. NASA shows lifted dilute
methane flames at matched fuel and air speeds of 10 to 40 cm/s, a factor of four. I put the loss
at **`Phi` = 2.5**, uncertain between 1.5 and 4.

### 1.3 Dilution is the lever

```text
S_L(x) = S_L0 exp( -(T_a / 2) (1 / T_ad(x) - 1 / T_ad(1)) )
methane:   T_ad(x) = 298 K + 1928 K / (1 + 0.085 (1 - x) / x),   T_a / 2 = 12000 K
ethylene:  T_ad(x) = 298 K + 2072 K / (1 + 0.058 (1 - x) / x),   T_a / 2 = 10000 K
```

(the extra nitrogen per mole of fuel adds to the heat capacity of the stoichiometric products).

| Fuel fraction | 1.0 | 0.5 | 0.4 | 0.3 | 0.25 | 0.2 | 0.15 | 0.1 | 0.07 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Methane `S_L`, m/s | 0.38 | 0.26 | 0.21 | 0.15 | 0.12 | 0.083 | 0.046 | 0.015 | |
| Ethylene `S_L`, m/s | 0.68 | 0.55 | | 0.42 | | 0.30 | 0.21 | 0.11 | 0.050 |

Ethylene at a fuel fraction `x` loses as much as methane at about `1.6 x`. Below an adiabatic
temperature of 1500 to 1600 K (15 % methane, 8 % ethylene) no flame exists at any speed; the law
reaches that limit through `S_L` going to zero, not through a separate term. A test that dilutes
the fuel while the flame burns walks down this table at fixed speed and must lose the flame when
`Phi` passes 2.5, unless the script stops first.

### 1.4 When

The script ramps speeds and dilution over the planned window. If the final condition is beyond
the limit, the flame is lost on the way; the further beyond, the earlier. With the ramp taken as
uniform in `ln Phi` over the window, the loss time is the window times `2.5 / Phi_final` for
`Phi_final` above 2.5. The uncertainty of `ln Phi` (0.5, as in SLICE) is carried by the shape of
the clock: `P(lost before fuel off) = 1 / (1 + (2.5 / Phi_final)^3.4)`.

A second, small clock stands for everything else: the moment the igniter retracts from a marginal
flame, a controller that jumps instead of ramping (the note says the first coflow controller did),
a fuel flow that sags. Its hazard falls with time.

### 1.5 Radiation

The five radiometers see the radiant power of the flame, `chi Q_fuel`. Two sources:

- **Gas.** Hot carbon dioxide and water, optically thin: `4 kappa sigma T^4` is about 3 MW/m3 at
  1700 K, and a flame of a few watts to tens of watts with its plume has a hot volume of 0.1 to
  1 cm3 per 10 W: a radiant fraction of 2 to 10 %. I take 0.05 for every flame.
- **Soot.** Pure ethylene on this tube at 0.2 to 0.4 m/s burns 1 to 2 mg/s, which is at its
  microgravity smoke point (the SPICE law gives 1.8 mg/s for a 2.2 mm tube at 30 cm/s): radiant
  fractions of 0.4 to 0.6 are reported there. Soot loading falls steeply with dilution, as the
  fuel fraction to a power of 2 to 3. I take a soot fraction of 0.35 (what five detectors facing
  one side are likely to register of it) times `x^2.5` for ethylene, 0.15 times `x^2.5` for
  methane.

The reading is then `V = c_V chi Q_fuel`. The constant `c_V`, volts per radiant watt, depends on
the detectors, their distance and their amplifiers, which I do not have. It is the one number of
this document taken from the project's own data: see 8.3.

### 1.6 The earlier kernel (`property_jets_v2.rs`), as a colleague's work

It would give the stoichiometric surface and an adiabatic temperature field for these flames and
says itself that it predicts no lift-off, no extinction and no radiometer signal: no finite-rate
chemistry, no radiation, an idealised rim. For this experiment that leaves nothing to score. The
omission that matters most is not the missing chemistry mechanism but the missing flame property:
one number, `S_L` of the diluted mixture, is enough to place the limit to within the uncertainty
of the coflow reading.

## 2. Targets

From the engineer's note `docs/wsp/feedback/cld_feasibility.md` (its part for the physicist and
its sections 1 and 2).

| Target | Kind, unit | A forecast may know |
|---|---|---|
| `loss`: the flame goes out while the fuel still flows; probability by 1, 2, 3, 5, 7, 10, 15, 20, 30, 45, 60, 90, 120 s after flame onset, and before the script turns the fuel off | event, s since onset | fuel, initial and final fuel fraction, fuel-stream and coflow speeds at ignition and at the end, the commanded schedule, the planned fuel-off time, tube and annulus diameters, coflow oxygen, chamber pressure, campaign round |
| `radiometer`: excess of the radiometer sum over its pre-ignition level in the first constant-flow stage | positive, V (scored as log10) | the same |

Flame length, lift-off height and soot are in images only. What the law says about lifting is
given in section 5 for the record.

## 3. The law

```text
z_x = ln(x_fuel_final),  z_u = ln(U_co,final / 0.2 m/s),  z_w = ln(window / 60 s),  e = 1 for ethylene
z_s = ln(x_fuel in the first constant-flow stage),  z_v = ln(fuel-stream speed in that stage / 0.2 m/s)

ln(S_L / 0.38 m/s) = 0.582 e + f(z_x + 0.47 e)
f(z) = 0.55 z - 0.55 relu(z) - 0.5 relu(-0.693 - z) - 0.6 relu(-1.204 - z) - 1.0 relu(-1.609 - z)

loss, blow-off clock:  ln tau_b = ln(window) + ln 2.5 - ln Phi_final
                                = 6.403 + z_w - z_u + ln(S_L / 0.38),         shape 3.4
loss, other clock:     tau_o = 70000 s,                                        shape 0.5
P(lost by t) = 1 - 1 / ((1 + (t / tau_b)^3.4) (1 + (t / tau_o)^0.5))

radiometer:  ln V = ln(0.668 V) + z_v + z_s + 0.5 e + sp(1.099 + 2.5 z_s + 0.847 e)
             sp(t) = 0.31 relu(t + 2.2) + 0.38 relu(t) + 0.31 relu(t - 2.2)          (ln(1 + e^t) in three pieces)
             ln-sd 0.8 (0.35 in log10), 10 % of cases three times wider
```

`f` is the table of 1.3 in four straight pieces (within 8 % down to 20 % methane and 15 %
ethylene, 20 to 30 % low below). 0.668 V is `c_V` times the gas radiation of 26.7 W of pure
methane (0.2 m/s on this tube): `0.5 V/W * 0.05 * 26.7 W`.

Limits. Coflow to zero: no blow-off (a real flame in still gas can starve; not in the law).
Dilution to the limit: certain loss, at once. Pure fuels: only the small clock. Window short: the
ramp is steeper and the same final condition is reached sooner. Radiometer: proportional to the
fuel flow at fixed dilution; proportional to the fuel fraction at fixed speed for dilute flames,
and eight times more per watt for pure ethylene than for a dilute flame. Range: coflow 2 to
150 cm/s, fuel fraction 0.04 to 1, fuel-stream speed 2 to 150 cm/s, windows of 5 to 600 s, air
coflow of 21 % oxygen near 1 atm, this burner.

## 4. Constants

Classes: **P** property, **T** theory, **L** published experiment or computation, **J** judgement,
**N** taken from the engineer's note (a train aggregate; not blind).

| Constant | Value | Uncertainty | Class | Origin |
|---|---|---|---|---|
| Shelter factor, kernel speed over coflow speed | 0.66 | 0.4 to 1.0 | L, J | Takahashi et al. 2022; see `phys_slice_w0.md` |
| Critical `v_k / S_L` at lifting | 1.4 | 1.1 to 1.8 | L | same |
| Loss over lifting, in `Phi` | 2.5 | 1.5 to 4 | T, J, N | 1.2; chosen in this range knowing that few flames were lost (8.3) |
| `S_L` methane, ethylene in air | 0.38, 0.68 m/s | 5 % | P | stoichiometric, 298 K, 1 atm |
| Adiabatic temperatures in air | 2226 K, 2370 K | 1 % | P | |
| Half activation temperatures | 12000 K, 10000 K | 20 % | P | overall one-step values (from memory) |
| Heat-capacity ratio of the extra nitrogen | 0.085 (methane), 0.058 (ethylene) per mole of N2 per mole of fuel | 10 % | P | 33 J/mol/K over 388 and 566 J/K of stoichiometric products |
| Ethylene shift on the dilution axis | 1.6 (0.47 in ln) | 1.4 to 1.8 | T | 1.3: equal reduction of `S_L` |
| Shape of the blow-off clock | 3.4 | 2.5 to 5 | T | `1.7 / 0.5`, the width of `ln Phi` |
| Other losses: median, shape | 70000 s, 0.5 | 2 to 8 % in 120 s | J, N | 4 % in 120 s, 0.5 % in 2 s; level chosen knowing the train count (8.3) |
| Fuel tube inner diameter | 2.18 mm | 2.1 to 2.2 | N | engineer's note (final report) |
| Heat release per m/s of fuel-stream speed, pure fuel | 133.6 W (methane), 220.3 W (ethylene) | 5 % | P, T | tube area 3.733 mm2, 44.6 mol/m3 at 0 C, lower heating values 802.3 and 1323 kJ/mol |
| Gas radiant fraction | 0.05 | 0.02 to 0.10 | T | 1.5 |
| Soot radiant fraction of the pure fuel, as seen | 0.15 (methane), 0.35 (ethylene) | factor 2 | L, J | 0.4 to 0.6 radiated at microgravity smoke points (Urban et al. 2000, via Dotson et al. 2011); methane by judgement |
| Dilution exponent of the soot fraction | 2.5 | 2 to 3 | J | soot volume fraction against fuel fraction |
| `c_V`, volts of radiometer sum per radiant watt | 0.5 V/W | factor 2 | N | set so that dilute methane flames of 5 to 25 W fall inside the range of excess the note states for methane and pure ethylene of 40 to 90 W near the top it states for ethylene |

## 5. Registered expectations

Written before any comparison with project data (with the exceptions of 8.3).

Probability that the flame is lost before a planned fuel-off 60 s after onset, against the final
fuel fraction and the final coflow speed. The 0.03 floor is the small clock.

| Fuel | Final fuel fraction | 10 cm/s | 20 cm/s | 30 cm/s | 40 cm/s | 60 cm/s |
|---|---:|---:|---:|---:|---:|---:|
| Methane | 1.0 | 0.03 | 0.03 | 0.03 | 0.03 | 0.04 |
| Methane | 0.4 | 0.03 | 0.03 | 0.04 | 0.06 | 0.14 |
| Methane | 0.3 | 0.03 | 0.04 | 0.06 | 0.11 | 0.29 |
| Methane | 0.25 | 0.03 | 0.05 | 0.11 | 0.23 | 0.52 |
| Methane | 0.2 | 0.04 | 0.11 | 0.28 | 0.49 | 0.79 |
| Methane | 0.15 | 0.12 | 0.55 | 0.83 | 0.93 | 0.98 |
| Methane | 0.1 | 0.81 | 0.98 | 0.99 | 1.00 | 1.00 |
| Ethylene | 1.0 | 0.03 | 0.03 | 0.03 | 0.03 | 0.03 |
| Ethylene | 0.3 | 0.03 | 0.03 | 0.03 | 0.03 | 0.04 |
| Ethylene | 0.2 | 0.03 | 0.03 | 0.03 | 0.04 | 0.07 |
| Ethylene | 0.15 | 0.03 | 0.03 | 0.04 | 0.07 | 0.18 |
| Ethylene | 0.1 | 0.04 | 0.11 | 0.28 | 0.50 | 0.79 |
| Ethylene | 0.07 | 0.20 | 0.70 | 0.90 | 0.96 | 0.99 |
| Ethylene | 0.05 | 0.82 | 0.98 | 0.99 | 1.00 | 1.00 |

Against time, methane diluted to 15 % at 30 cm/s, window 60 s: 0.01 at 5 s, 0.02 at 10 s, 0.11 at
20 s, 0.32 at 30 s, 0.64 at 45 s, 0.83 at 60 s. Methane held at 35 % and 30 cm/s, window 90 s:
0.007 at 3 s, 0.012 at 10 s, 0.021 at 30 s, 0.033 at 60 s, 0.054 at 90 s.

Radiometer excess, volts (median; one standard deviation is a factor 2.2), against the fuel
fraction and the fuel-stream speed of the first constant-flow stage:

| Fuel | Fuel fraction | 10 cm/s | 20 cm/s | 30 cm/s | 50 cm/s |
|---|---:|---:|---:|---:|---:|
| Methane | 1.0 | 1.4 | 2.8 | 4.2 | 7.1 |
| Methane | 0.7 | 0.53 | 1.1 | 1.6 | 2.7 |
| Methane | 0.4 | 0.18 | 0.37 | 0.55 | 0.91 |
| Methane | 0.3 | 0.11 | 0.22 | 0.33 | 0.55 |
| Methane | 0.2 | 0.07 | 0.13 | 0.20 | 0.33 |
| Ethylene | 1.0 | 4.2 | 8.3 | 12.5 | 21 |
| Ethylene | 0.5 | 0.63 | 1.3 | 1.9 | 3.2 |
| Ethylene | 0.3 | 0.23 | 0.47 | 0.70 | 1.2 |
| Ethylene | 0.2 | 0.11 | 0.23 | 0.34 | 0.57 |
| Ethylene | 0.1 | 0.06 | 0.11 | 0.17 | 0.28 |

For the record, not scorable from text: the coflow speed above which the law says the flame is
lifted (`Phi` = 1): 0.81 m/s for pure methane, 0.44 at 40 %, 0.32 at 30 %, 0.24 at 25 %, 0.16 at
20 %; ethylene 0.94 at 30 %, 0.62 at 20 %, 0.40 at 15 %, 0.16 at 10 %.

In words:

1. A flame that is not diluted below 30 % methane or 15 % ethylene and whose coflow stays at or
   below 30 cm/s is lost in about one test in fifteen to thirty, whatever else the script does.
2. A flame diluted during the test to 15 % methane or 7 % ethylene at 20 cm/s or more is lost in
   most tests, in the second half of the dilution.
3. At equal dilution ethylene holds at about three times the coflow of methane.
4. The radiometer reading of a dilute flame is proportional to its fuel flow (speed times fuel
   fraction); pure ethylene reads 15 to 20 times a 30 % ethylene flame at the same speed.

## 6. What would show the law is wrong

1. **Losses do not follow the coflow speed over the burning speed.** If flames diluted to 15 %
   methane at 30 cm/s usually survive (the law: lost in 83 %), or flames at 30 % methane and
   30 cm/s are often lost in the ramps (the law: 6 %), the limit is not where `S_L` puts it. A
   loss rate that does not rise with the coflow speed at fixed dilution would mean extinction is
   not a blow-off at all (heat loss to the burner, or radiative quenching of a slow flame), and
   `Phi` is the wrong variable.
2. **Methane and ethylene do not line up on the dilution axis as the law says.** Ethylene at 10 %
   should behave like methane at 16 to 20 %. If ethylene flames are lost at the same fuel
   fractions as methane, or survive to 5 %, the activation temperatures or the adiabatic
   temperatures used here are wrong by more than their stated uncertainty.
3. **The radiometer does not scale with the fuel's heat release.** For dilute flames of one fuel
   the excess should double when the fuel flow doubles. If it barely moves with flow (the
   detectors see the warming burner more than the flame), or if pure ethylene reads less than
   five times a 30 % flame at the same speed (little soot, or the soot outside the field of
   view), the two-term radiant fraction is wrong. The absolute level is not a test of the law:
   it was set from the note.

A fourth, on timing: the law puts ramp losses in the second half of the window. If they come
within three seconds of onset, they are ignition losses, which the law treats only through the
small clock.

## 7. Interface for the law forward (`docs/wsp/research/law_forward.md`)

Slug `cld`, six features, two targets.

### 7.1 Features (declaration order)

| # | `name` | SI unit | transform | centre | scale | domain (raw) | Case-file input |
|---|---|---|---|---|---|---|---|
| 0 | `x_fuel_final` | 1 | ln | 1 | 1 | 0.04 to 1 | `x_fuel_final` |
| 1 | `v_coflow_final` | m/s | ln | 0.2 | 1 | 0.02 to 1.5 | `v_coflow_final_m_s` |
| 2 | `window` | s | ln | 60 | 1 | 5 to 600 | `at_cutoff.planned_fuel_off_from_onset_s` |
| 3 | `is_ethylene` | 1 | identity | 0 | 1 | 0 to 1 | `fuel` = ethylene |
| 4 | `x_fuel_stage` | 1 | ln | 1 | 1 | 0.04 to 1 | `x_fuel` of the first constant-flow segment of `schedule_from_ignition_command` |
| 5 | `v_fuel_stage` | m/s | ln | 0.2 | 1 | 0.02 to 1.5 | `v_fuel_m_s` of that segment |

If the lowest fuel fraction or the highest coflow speed of the schedule is not the final one, give
the lowest fraction and the highest speed in features 0 and 1: the law wants the most severe
condition the script will reach.

### 7.2 Targets (declaration)

| `name` | kind | unit | causes |
|---|---|---|---|
| `loss` | event | s since flame onset | `blowoff`, `other` |
| `radiometer` | positive | V (lane-local) | |

`p_loss_by` is the sum of the two cause rows at each horizon; `p_loss_before_fuel_off` is that sum
at the planned fuel-off. If the planned fuel-off is declared as the scheduled stop
(`scheduled_stop.condition`), the stop tensors are the two lines given below; without it they do
not exist.

### 7.3 Tensors

Those of section 3 of the forward's document for 6 features: `x.<f>.centre|scale|lo|hi`;
`loss.blowoff.median` and `loss.other.median` (ln s) with `b0`, `skip` [6], `W1` [8, 6], `b1` [8],
`w2` [8]; `loss.blowoff.shape`, `loss.other.shape`; `radiometer.median` (ln V),
`radiometer.spread`, `radiometer.tail_weight`, `radiometer.tail_width`.

### 7.4 Literal assignments

```text
# cld.wsp: W0 with stated exceptions. Author claude-fable-5-2. Source: docs/wsp/research/phys_cld_w0.md
# features z = (z_x, z_u, z_w, e, z_s, z_v), columns 0..5
x.x_fuel_final.centre = 1.0
x.x_fuel_final.scale = 1.0
x.x_fuel_final.lo = 0.04
x.x_fuel_final.hi = 1.0
x.v_coflow_final.centre = 0.2
x.v_coflow_final.scale = 1.0
x.v_coflow_final.lo = 0.02
x.v_coflow_final.hi = 1.5
x.window.centre = 60.0
x.window.scale = 1.0
x.window.lo = 5.0
x.window.hi = 600.0
x.is_ethylene.centre = 0.0
x.is_ethylene.scale = 1.0
x.is_ethylene.lo = 0.0
x.is_ethylene.hi = 1.0
x.x_fuel_stage.centre = 1.0
x.x_fuel_stage.scale = 1.0
x.x_fuel_stage.lo = 0.04
x.x_fuel_stage.hi = 1.0
x.v_fuel_stage.centre = 0.2
x.v_fuel_stage.scale = 1.0
x.v_fuel_stage.lo = 0.02
x.v_fuel_stage.hi = 1.5

# --- loss by blow-off: ln tau = ln window + ln 2.5 - ln Phi_final.
# b0 = ln(60 s) + ln(2.5) - ln(0.66 * 0.2 / (1.4 * 0.38)): 604 s for pure methane at 0.2 m/s.
# skip: S_L falls 0.55 per e-fold of dilution; more coflow, sooner; longer window, later;
# ethylene: ln(0.68 / 0.38) + 0.55 * 0.47.
loss.blowoff.median.b0 = 6.403
loss.blowoff.median.skip[0:6] = [0.55, -1.0, 1.0, 0.84, 0.0, 0.0]
# unit 0: no gain above the pure fuel (ethylene above 62 %).
loss.blowoff.median.W1[0, 0] = 1.0
loss.blowoff.median.W1[0, 3] = 0.47
loss.blowoff.median.b1[0] = 0.0
loss.blowoff.median.w2[0] = -0.55
# units 1 to 3: S_L falls faster below 50, 30 and 20 % methane (31, 19 and 12.5 % ethylene).
loss.blowoff.median.W1[1, 0] = -1.0
loss.blowoff.median.W1[1, 3] = -0.47
loss.blowoff.median.b1[1] = -0.693
loss.blowoff.median.w2[1] = -0.5
loss.blowoff.median.W1[2, 0] = -1.0
loss.blowoff.median.W1[2, 3] = -0.47
loss.blowoff.median.b1[2] = -1.204
loss.blowoff.median.w2[2] = -0.6
loss.blowoff.median.W1[3, 0] = -1.0
loss.blowoff.median.W1[3, 3] = -0.47
loss.blowoff.median.b1[3] = -1.609
loss.blowoff.median.w2[3] = -1.0
loss.blowoff.shape = 3.4
# other losses: 4 % in 120 s, hazard falling with time.
loss.other.median.b0 = 11.156
loss.other.median.skip[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
loss.other.shape = 0.5
# only if the planned fuel-off is declared as the scheduled stop:
loss.stop.offset_s[0:3] = [0.0, 0.0, 0.0]
loss.stop.offset_w[0:3] = [1.0, 0.0, 0.0]

# --- radiometer excess = c_V * chi * Q. b0 = ln(0.5 V/W * 0.05 * 26.72 W) = ln(0.668 V).
# skip: proportional to fuel-stream speed and to fuel fraction; ethylene releases 1.65 times
# the heat per volume of fuel.
radiometer.median.b0 = -0.404
radiometer.median.skip[0:6] = [0.0, 0.0, 0.0, 0.5, 1.0, 1.0]
# units 0 to 2: soot, ln(1 + chi_soot x^2.5 / 0.05) in three pieces; 1.099 = ln(0.15 / 0.05),
# ethylene 0.847 = ln(0.35 / 0.15) more.
radiometer.median.W1[0, 3] = 0.847
radiometer.median.W1[0, 4] = 2.5
radiometer.median.b1[0] = 3.299
radiometer.median.w2[0] = 0.31
radiometer.median.W1[1, 3] = 0.847
radiometer.median.W1[1, 4] = 2.5
radiometer.median.b1[1] = 1.099
radiometer.median.w2[1] = 0.38
radiometer.median.W1[2, 3] = 0.847
radiometer.median.W1[2, 4] = 2.5
radiometer.median.b1[2] = -1.101
radiometer.median.w2[2] = 0.31
radiometer.spread.b0 = 0.8
radiometer.spread.skip[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
radiometer.tail_weight = 0.1
radiometer.tail_width = 3.0
```

What the forward cannot express here. The loss clock reads the most severe condition of the
script and the length of the window, not the schedule itself: a script that reaches its final
dilution early and then holds it is timed too late, one that dilutes only in its last seconds too
early. A loss at the retraction of the igniter belongs to a class of its own (a marginal flame
either survives the first two seconds or does not); one clock per cause cannot hold a fraction of
flames that never fail, so it is folded into the small clock. The target is asked as log10 of
volts: the forward gives quantiles of volts, and the logarithm of a quantile is the quantile of
the logarithm.

## 8. Sources and limits

### 8.1 Sources

- F. Takahashi, L. Smith, D. T. Souza, V. R. Katta, "Critical conditions at liftoff limit of a
  laminar n-butane-air jet diffusion flame", Proc. Combust. Inst. 39 (2022): the kernel speed
  against coflow and the critical ratio, as used in `phys_slice_w0.md`.
- D. P. Stocker, "Laminar non-premixed flames of gaseous fuel aboard the International Space
  Station", Central States Section meeting 2022, and the JASMAC-33 slides (2021). **Summaries of
  this experiment.** Taken: fuel and air speeds typically matched; lifted flames are the focus and
  are shown at 10 to 40 cm/s; dilution limits extended in microgravity; soot several times higher
  than on the ground.
- D. L. Urban et al. 2000 and K. T. Dotson et al. 2011 (see `phys_spice_w0.md`): radiant fractions
  of 0.4 to 0.6 at microgravity smoke points; the smoke point of ethylene on a 2 mm tube.
- Burning speeds, adiabatic temperatures, heating values, overall activation temperatures, the
  Planck-mean absorption of combustion products: textbook values (from memory).
- The card `docs/wsp/atlas/cards/PSI-21.json`, the section of `INVESTIGATIONS.md`, the engineer's
  note (its part for the physicist and sections 1 and 2).

### 8.2 Not found or not checked

- The papers of the investigators on these flames (lift-off heights, extinction limits).
- The burning speed of nitrogen-diluted methane and ethylene from a table; computed here.
- The radiometers: type, distance, field of view, gain. `c_V` could not be derived.
- Whether the first constant-flow stage is at the ignition or the final condition (the features
  ask for the stage's own values).
- Whether the shelter factor 0.66 holds when fuel and air speeds are matched and without gravity.
- The oxygen of the chamber gas around the coflow (the note says nitrogen-filled): a flame longer
  than the air column would starve, which the law does not consider.

### 8.3 Exposure

The sections of the engineer's note that define the targets also report, from train days: that 20
of 207 flames were lost, in which stage of the script (dilution ramp 8, igniter retraction 5,
velocity ramp 3, coflow ramp 4), that 9 of 11 tests which dilute the fuel during the test lost the
flame against 11 of 196 that do not, that losses cluster at 0.9 to 2.4 s and at 10 to 38 s, and
the range of the radiometer excess (methane 0.14 to 1.8 V, ethylene 0.16 to 12.4 V). I read them.
What they changed:

- The mechanism (section 1.1 to 1.3) was carried over from SLICE, written before I read the note.
- `Phi` = 2.5 for loss: the argument of 1.2 allows 1.5 to 4; I chose inside it knowing that few
  undiluted flames are lost.
- The small clock (4 % in 120 s): its level was set knowing the count 11 of 196.
- `c_V` = 0.5 V/W: set from the stated ranges. The radiometer forecast is a scaling law with a
  level taken from train data; its absolute values are not a zero-shot result.
- That dilution tests lose the flame is what the law says by itself; I did not need the count,
  but I had seen it.

No file of outcomes was opened, and the note's sections 3 to 6 were not read.

# Ignition and sustained flame of a thin fuel in low-speed flow: a W0 law from first principles

Author: `claude-fable-5-2`, 15 November 2026. Kind: **W0, literal authoring**. No outcome file of
this project was opened (section 7.3 lists exactly what was read, including one published table of
per-test outcomes that passed in front of me and was not used). Every number below was typed from a
property table, a derivation shown here, a cited publication, or a stated judgement. Nothing was
fitted; the only program run was a forward evaluation of the typed law to fill the tables of
section 4.

## 0. The result in one screen

A flame on a thin fuel in a slow stream survives when the oxygen enthalpy the stream carries to it
covers two demands: what chemistry needs to stay lit at that flow time, and what the hot surface
radiates away. Written in oxygen mole fraction, the limit is a floor, plus a term that grows with
speed (blow-off), plus a term that grows as speed falls (radiative quenching):

```text
X_lim(U) = x0 + cp ln(1/p) + sb[dir] U_e + (ar[dir] + aw / W) / (p U_e)

U_e = U + u_s  (opposed)        U_e = sqrt(U^2 + u_s^2)  (concurrent)

P(sustained)  = Phi( (X - X_lim) / s )
s^2 = sd_x0^2 + (sd_cp ln(1/p))^2 + (sd_sb[dir] U_e)^2 + (sd_ar[dir]^2 + (sd_aw / W)^2) / (p U_e)^2

P(ignition)   = Phi( (X - X_lim(U_ign) - dx) / sqrt(s^2 + sd_a^2) ) * Phi( ln(t_on / t_req) / sd_lnt )
```

`X` oxygen mole fraction, `p` pressure in atm, `U` duct flow speed in cm/s, `W` sample width in cm,
`dir` concurrent or opposed, `Phi` the standard normal distribution function. The boundary variable
is the **oxygen margin `X - X_lim(U)`**; its width `s` is the propagated uncertainty of the five
coefficients, so it widens by itself where the theory is weakest (very low and high speed).

For SIBAL fabric 2.2 cm wide in concurrent flow at 1 atm the 50 % boundary is 17.0 % O2 at 5 cm/s,
16.5 % at 10 cm/s, 16.8 % at 20 cm/s, with a width of 1.1 to 1.2 % O2 (one standard deviation). In
air it puts the low-speed quench at 1.5 cm/s (10 % to 90 % band: 0.8 to 2.3 cm/s).

## 1. Mechanism

### 1.1 The energy balance of a near-limit flame

Take the part of the flame that anchors it: the leading edge of an opposed-flow flame, or the base
of a concurrent-flow flame at the burnout edge. Its only length scale is the distance over which
diffusion balances convection, `l = D / U_e`, where `U_e` is the gas speed the flame sees. Per unit
width of sample:

| Term | Scaling | Comment |
|---|---|---|
| Oxygen enthalpy supplied | `rho_inf Y_O dh_O U_e l` | convection through a height `l`; diffusion from above adds the same order |
| Heat radiated by the surface | `eps sigma (T_s^4 - T_inf^4) l` | the surface stays near the pyrolysis temperature whatever the flow |
| Heat radiated by the gas | `4 kappa_P sigma T_f^4 l^2` | grows as `l^2`: matters only at the lowest speeds |
| Side loss to the holder | `2 lambda (T_f - T_inf) l / W` relative share | the flame edge touches cold metal over a height `l` |
| Heat chemistry must retain | enough to keep the flame above its extinction temperature `T_c(U_e)` | rises with speed: less residence time |

The supply is proportional to `U_e l = D`, independent of speed, while it is spread over a flame
whose size `l` grows as the flow slows. Surface radiation is a fixed flux, so its share of the
budget is `l` times a constant: it grows as `1 / U_e`. This is T'ien's low-stretch radiative
extinction (T'ien 1986) and the low-speed branch found by Olson, Ferkul and T'ien (1988) for thin
paper. Measured surface energy balances say the same: the losses stay roughly constant while the
feedback from the flame falls with the stretch rate (Olson and T'ien 2000, PMMA cylinders in 1 g
at stretch rates 1.5 to 100 1/s).

### 1.2 Two dimensionless groups

**Radiative-loss number** (heat radiated over oxygen enthalpy supplied):

```text
Rad = G eps sigma (T_s^4 - T_inf^4) / (rho_inf Y_O dh_O U_e)
```

`G` is a geometry factor of order one (radiating length over oxygen-capture height, plus the gas
radiation share, plus how the duct speed maps to the speed at the flame). The side loss multiplies
it by `(1 + l_w / W)`, with `l_w` a lateral length of a few millimetres.

**Damkohler number** (flow time over chemical time). A flame goes out when its temperature falls
to an extinction temperature `T_c` that rises with the strain it sees, `1/T_c = 1/T_c0 - (R/E) ln(a/a0)`.
Since the adiabatic temperature rise is nearly proportional to the oxygen fraction (derivation
D2), the chemistry demands an oxygen fraction `X_k(U_e)` that rises with speed. Over 0 to 40 cm/s a
straight line is as good as the logarithm the Arrhenius law gives, and it is what 1 g blow-off
tests on PMMA rods show (Marcum, Olson and Ferkul 2016: oxygen limit proportional to flow speed):

```text
X_k(U_e) = x0 (1 + 1/Da),   Da = U_b / U_e,   U_b = x0 / sb
```

`U_b` is the speed at which chemistry asks for twice the floor oxygen (185 cm/s concurrent, 92 cm/s
opposed for SIBAL with the numbers of section 3).

**The statement that carries to another apparatus:** with `K = (x0 / X)(1 + 1/Da)` and
`R = Rad (1 + l_w / W)`, both evaluated at the ambient oxygen fraction `X`,

```text
a flame is sustained when   K + R < 1
```

`K` is the share of the available temperature rise that chemistry needs, `R` the share radiated
and lost sideways. Because `Rad` is proportional to `1 / X`, multiplying through by `X` gives the
additive oxygen limit of section 0: `ar = G eps sigma (T_s^4 - T_inf^4) / (C_O dh_O)` with `C_O`
the oxygen mass per unit volume per unit mole fraction. This has the same structure as the
published scale model of Takahashi et al. (2014), `eta + R_rad + 1/Da = 1` with a flame at
`eta > 0`, which was checked against parabolic-flight limits of thin PMMA. Two differences, both
deliberate: I normalise the radiative loss by the oxygen enthalpy flux instead of by
`rho c (T_f - T_v) U`, which makes the limit explicit in `X`; and I use a linear blow-off term
with a floor, because their intersection of the two pure limits put the minimum oxygen near 11 %
where the flights measured 14.7 %, and their coupled form still gave about 13 %.

### 1.3 Why two branches, and where the minimum is

`X_lim(U_e) = x0 + sb U_e + A / (p U_e)` with `A = ar + aw / W` is a line plus a hyperbola.

- **Quenching branch** (low speed): `A / (p U_e)` dominates. The flame shrinks, turns blue, and
  goes out slowly, in three dimensions. Quench speed at a given oxygen fraction:
  `U_q = A / (p (X - x0 - sb U_q))`, so the quench speed rises as oxygen falls, and
  `U_q (X - x0)` is roughly constant.
- **Blow-off branch** (high speed): `sb U_e` dominates. The flame base is swept downstream; this is
  fast (a gas transit time).
- **Minimum**: at `U* = sqrt(A / (p sb))`, `X_min = x0 + cp ln(1/p) + 2 sqrt(A sb / p)`. The
  material is most flammable at a speed of the order of 5 to 10 cm/s, the range of spacecraft
  ventilation, and `X_min` lies below any 1 g limit because buoyant flow is already on the
  blow-off side (Olson 2004: thin cellulose burns at 14 % O2 at 5 to 20 cm/s against a 1 g
  opposed-flow limit of 16.5 %).

### 1.4 How each condition enters

| Condition | Enters through | Direction |
|---|---|---|
| Oxygen fraction `X` | the available enthalpy per unit of gas, linearly | more oxygen, more margin |
| Pressure `p` | `rho_inf` in `Rad`: `A / (p U_e)`; a small shift of the floor `cp ln(1/p)` | lower pressure acts like lower speed on the quench side |
| Flow speed `U` | `1 / U_e` (quench) and `sb U_e` (blow-off) | optimum in between |
| Flow direction | `ar[dir]`, `sb[dir]`, and the sign of the self-spread term in `U_e` | opposed survives lower speed; concurrent survives higher speed (1.5) |
| Fuel area density | the self-spread speed `u_s`, inversely; the thermal time of quenching | a heavy fuel cannot make its own relative wind: no flame in still gas |
| Sample width `W` | `aw / W` in the quench term | narrower strip in a metal holder loses more; weak effect, uncertain sign (1.6) |
| Inert content | the floor `x0` (and the heat capacity in `u_s`) | glass raises the limit (D4) |

### 1.5 Direction

In still gas there is no direction: a flame can only spread into fresh oxidiser. The law respects
this through `U_e`: at `U = 0` use the opposed entry, where `U_e = u_s`, the speed the flame makes
by spreading.

With flow, the two configurations differ in two places.

- **Quench side.** A concurrent flame lies over its whole pyrolysis and burnout zone, which
  radiates, and the oxygen has to reach it past the base; an opposed flame radiates only from a
  leading edge that is moving into fresh gas. So `ar[concurrent] > ar[opposed]`. The one direct
  observation I found: thin cellulose in cabin air quenched at 0.5 cm/s opposed and 2 cm/s
  concurrent (Olson 2004, reporting the glovebox experiment of Olson et al. 2001). After the
  self-spread correction the ratio is about 2; I use 1.9.
- **Blow-off side.** The base of a concurrent flame is fed by the pyrolysis zone behind it and can
  retreat over fuel that is still gasifying; the leading edge of an opposed flame has nothing
  behind it. So `sb[concurrent] < sb[opposed]`. Thin cellulose in partial gravity spreads upward
  at 12 % O2 and downward at 14 % (T'ien et al. 1993), and SIBAL fabric does not spread downward
  in air in 1 g while it does spread upward (Ferkul et al. 2012; Johnston et al. 2015).

The two differences cross: below about 7 cm/s the opposed flame has the lower limit, above it the
concurrent flame.

### 1.6 Width

Two effects of opposite sign scale with the same group `l / W = D / (U_e W)`: heat lost sideways
to the cold holder, and oxygen diffusing in from the sides. The model of Shih and T'ien (2003)
finds that near the quench limit side diffusion of oxygen wins and narrow flames spread faster;
the BASS-II fabric paper (Zhao et al. 2015) states that the narrow sample has the smaller flammable
domain and that near quenching the burnt region narrows away from the holder edges. For a strip
clamped in a steel holder I take a small net loss, `aw = 0.03`, with an uncertainty as large as
the value, so the law does not claim to know the sign.

### 1.7 Derivations of the numbers

**D1. Oxygen enthalpy flux.** `C_O = 32 g/mol * p / (R T) = 1.30 kg/m^3` of oxygen per unit mole
fraction at 300 K and 1 atm. With `dh_O = 13.1 MJ` per kg of oxygen consumed (Huggett 1980), the
flux is `1.70e5 X U` W/m^2 with `U` in cm/s: 35.8 kW/m^2 in air at 1 cm/s.

**D2. Flame temperature against oxygen.** Burning 1 kg of fuel with `nu` kg of oxygen carried by
`nu / Y_O` kg of gas gives `T_ad - T_inf = (dh_c / c_p) Y_O / (Y_O + nu)`. For cellulose
(`nu = 1.19`) scaled to 2250 K in air, this is 1860 K at 16 % and 1690 K at 14 %: close to linear,
`kappa = 79 K` per percent of oxygen between 14 % and 23 %.

**D3. Radiative coefficient `ar`.** SIBAL pyrolysis kinetics (`A_s = 1.63e11 1/s`,
`E_s = 161 kJ/mol`, Olson et al. Saffire paper) give a surface temperature
`T_s = E_s / (R ln(A_s t))` of 665 to 690 K for heating times of 10 to 30 s; I use 680 K. With
`eps = 0.8` (same source) the loss is `0.8 * 5.67e-8 * (680^4 - 300^4) = 9.3 kW/m^2`. Divided by
D1: `ar = 0.055` (mole fraction times cm/s) for `G = 1`. In words: in air at 1 cm/s the surface
radiates a quarter of all the enthalpy the stream brings.

What `G` should be, from estimates I can only make to a factor of two:
- gas radiation: `4 kappa_P sigma T^4` is about 4 MW/m^3 for combustion products at 1500 K
  (`kappa_P` about 3.6 1/m, from memory of Planck-mean tables, unchecked); over a hot layer a third
  of `l` thick at 2 cm/s, half escaping, it adds 30 to 60 % to the surface loss, less at higher
  speed;
- duct boundary layer: the flame sits in the shear layer of the holder; with a wall gradient
  `a = 0.332 U^1.5 / sqrt(nu x)` and a flame speed scale `sqrt(a D)`, a 2 cm/s stream 3 cm from the
  holder's leading edge gives 1.2 cm/s at the flame (and `a = 1.4 1/s`, the stretch rate at which
  1 g PMMA flames quench, 1.5 to 3 1/s, Olson and T'ien 2000): a factor 1.4 to 2 on `G`;
- diffusion from above and from the sides adds supply: a factor 0.5 to 1.

Net `G` between 0.7 and 2.6. I take a geometric centre of 1.0 and split it by direction with the
ratio 1.9 of section 1.5: **`G = 1.36` concurrent, `0.73` opposed**, giving `ar = 0.075` and
`0.040`, each with a relative standard deviation of 35 %. The concurrent value was chosen inside
the theoretical range knowing the published BASS statements quoted in section 3; see 7.3.

**D4. The floor `x0`.** First principles give only a bracket. Diffusion flames of solids go out
when the adiabatic flame temperature of the atmosphere falls to about 1700 K, give or take 100 K:
with D2 this is what the microgravity minima of thin cellulose (14 %, 1700 K) and thin PMMA
(14.7 %, 1800 K) and the PMMA rod limit (13.6 %, 1715 K) amount to. That fixes the observable
minimum of a clean thin fuel at 14 % with a standard deviation of 1.3 % O2. For SIBAL I add what
1 g tests show about heavier and inert-loaded cellulose: cotton fabric has a downward limit near
18.4 % against 16.5 % for tissue (plus 1.9 %), and SIBAL does not burn downward in air at all, so
the glass adds more; I take plus 0.6 % for 25 % glass, by judgement (range 0.3 to 1.5 %). Sum:
`X_min = 16.5 %` for the concurrent minimum. Since the law gives `X_min = x0 + 2 sqrt(A sb)` and
the second term is 1.7 % with the numbers of section 3, **`x0 = 0.148`**. The BASS-II paper
reports steady spread at 16.9 % and tests "down to 16 %" with sporadic ignition near the boundary;
that brackets the same value and is why I state 1.0 % and not 1.5 % as its standard deviation.

For another inert fraction `phi`, by the same judgement: `x0 = x0(phi = 0) + 0.024 phi / (1 - phi)`.

**D5. Blow-off slope `sb`.** With `T_c = 1550 K` and a one-step activation energy of 125 kJ/mol
(30 kcal/mol, the usual value in thin-fuel models; exact value in the cited models not re-read),
`R T_c^2 / E = 159 K` per e-fold of strain, or 2.0 % O2 by D2. Strain goes as `U^1.5` in a boundary
layer and as `U^2` for a free leading edge, so 3.0 to 4.0 % O2 per e-fold of speed. The chord of
that logarithm between 10 and 40 cm/s is 0.0014 to 0.0018 per cm/s: **`sb[opposed] = 0.0016`**.
Check: thin cellulose, 14 % at about 12 cm/s equivalent (0.1 g) and 16.5 % at about 25 cm/s (1 g),
is 0.0019 per cm/s. For concurrent flow I take half, **`sb[concurrent] = 0.0008`**, by judgement
supported by two observations: two-sided SIBAL flames burning upward in 1 g air survive until
they are 15 to 30 cm long (Johnston et al. 2015), and concurrent flames in air were run at 40 cm/s
in the BASS duct (Zhao et al. 2015, abstract).

**D6. Self-spread speed `u_s`.** de Ris thin-fuel limit, `u_s = (pi/4) lambda_g F / (m c_s)`, with
`F = (T_f - T_s) / (T_s - T_inf) = 4.1`, `lambda_g = 0.048 W/m/K`, half-thickness area density
`m = 90 g/m^2`, `c_s = 0.75 * 1.26 + 0.25 * 0.70 = 1.12 J/g/K`: 1.5 mm/s. Published opposed spread
rates for this fabric: 1.21 mm/s at 10 cm/s (BASS), 1.3 mm/s at 20 cm/s (Saffire I).

**D7. Pressure.** `rho_inf` in D1 is proportional to `p`, so the quench term is `A / (p U_e)`:
exact within the scaling. The blow-off term is independent of pressure for a second-order
reaction against `U^2` strain and goes as `p^(-1/3)` against boundary-layer strain; I leave it
independent. The floor shift `cp = 0.02` per e-fold of `1/p` is a judgement (low-pressure limits
exist: T'ien et al. 1993; threshold oxygen concentrations rise as pressure falls in the White
Sands data of Hirsch et al., which I could not open) with an uncertainty equal to its value. All
tables below are at 1 atm, where both pressure terms vanish.

**D8. Ignition.** A Kanthal hot wire of the Saffire kind delivers 4.5 to 16 W per cm of sample
width for 8 to 9 s (36 to 150 J/cm). Heating a 0.5 cm strip of SIBAL to pyrolysis and gasifying
its cotton takes about 7 J/cm. The igniter is therefore never short of energy in a flammable
atmosphere; what decides ignition is whether the kernel survives once the wire is off, which is
the same boundary plus a small extra margin (a small three-dimensional kernel loses more). The
paper's own words are that ignition is "sporadic" very near the boundary: hence a 50 % point just
inside it (`dx = 0.004`) and a run-to-run scatter `sd_a = 0.005`. The on-time enters only when it
is short: the fabric under the wire needs `m c_s (T_s - T_inf) / q_wire`, about 1 s at 75 kW/m^2,
and I set the 50 % on-time at 2 s. My estimate of wire cooling at 40 cm/s is half the electrical
power, so ignition is not flow-limited in this duct.

**D9. Time to go out.** Radiative quenching is slow because the solid has to cool:
`m c_s (T_s - T_inf) / q_rad = 4 s` per side for SIBAL, and the flame shrinks over a few of these.
I set `tau_q = 12 s` (range 5 to 30). Blow-off takes a gas transit time: `tau_b = 1.5 s`.

### 1.8 Limits

| Limit | What the law says | Physical? |
|---|---|---|
| `U = 0`, heavy fuel | `U_e = u_s = 0.15 cm/s`, `X_lim = 51 %`; P = 0.01 in air | yes: SIBAL cannot burn in still air; the 1 % is the stated ignorance of `ar` |
| `U = 0`, very thin fuel | `u_s` grows as `1 / m`; tissue (`m = 10 g/m^2`, floor 12.4 %) has a thermal-limit `u_s` near 1 cm/s and a still-gas limit of 16 to 18 % | partly: the published still-gas limit of tissue is about 21 % (Olson 1991, from memory). With the measured near-limit spread rate, about 0.5 cm/s, the law gives 20 to 21 %. `u_s` is an upper bound on the wind a flame makes for itself, so the zero-flow column is optimistic for very thin fuels |
| `m -> 0` | `u_s` unbounded: the flame blows itself off through `sb u_s` | right direction: the thermal spread rate is capped by chemistry |
| high oxygen | P goes to 1 for any speed above about 1 cm/s | yes |
| `U` large | linear blow-off; do not use above about 60 cm/s, where the logarithm bends | stated range |
| `W` large | `aw / W` vanishes: two-dimensional flame | yes |
| `p -> 0` | quench term unbounded: a low-pressure limit at any speed | yes |
| thick fuel | not covered: in-depth conduction is a further steady loss of the size of the radiative one (Olson and T'ien 2000) | outside the law |

### 1.9 Orders of magnitude against published data (checks, not fits)

| Published observation | Law |
|---|---|
| SIBAL, concurrent, air: quenching at "around 1 cm/s" (Ferkul et al. 2012) | 50 % at 1.5 cm/s; P = 0.19 at 1 cm/s |
| SIBAL, concurrent, 16 to 21 % O2: quench speeds "between 1 and 5 cm/s", higher at lower oxygen (Zhao et al. 2015) | 1.5 cm/s at 21 %, 3.0 at 18 %, 4.9 at 17 % |
| SIBAL, opposed, air: flame alive as flow is stepped from 10 down to about 1 cm/s (Ferkul et al. 2012) | P = 0.75 at 1 cm/s, 0.99 at 2 cm/s |
| SIBAL, opposed, air, Saffire I, 20 cm/s: spreads | P = 0.94 (narrow strip; wider is safer) |
| SIBAL, 1 g, downward in air: no spread (Ferkul et al. 2012) | opposed limit reaches 21 % at 38 cm/s duct speed; buoyant flow at an unshielded leading edge is at least that severe |
| Thin PMMA, opposed, parabolic flight: minimum 14.7 % at 6 cm/s (Takahashi et al. 2014) | PMMA film constants (section 3.3): 14.7 % at 6.0 cm/s; the speed is a prediction, the level was set to it |
| PMMA film 0.1 mm, opposed, BASS-II: radiative quenching below 2 cm/s, near blow-off at 40 cm/s (Ferkul et al. 2014, oxygen not stated) | film constants at 18 %: quench 0.9 cm/s, blow-off 33 cm/s; at 21 %: 0.5 and 53 cm/s |
| PMMA in air, 1 g low-stretch: quench at 1.5 to 3 1/s (Olson and T'ien 2000) | air quench at 0.7 to 1.5 cm/s is a wall strain of 0.3 to 0.9 1/s: lower, as it should be for a thin fuel without in-depth loss |

## 2. The law

Two probabilities, each a normal distribution function of the oxygen margin over a limit that
depends on flow speed, direction, width, pressure and material:

```text
sustained:  P_sus = Phi( (X - X_lim(U)) / s(U) )
ignition:   P_ign = Phi( (X - X_lim(U_ign) - dx) / sqrt(s(U_ign)^2 + sd_a^2) ) * Phi( ln(t_on / t_req) / sd_lnt )

X_lim(U) = x0 + cp ln(1/p) + sb[dir] U_e + (ar[dir] + aw / W) / (p U_e)
s(U)^2   = sd_x0^2 + (sd_cp ln(1/p))^2 + (sd_sb[dir] U_e)^2 + (sd_ar[dir]^2 + (sd_aw / W)^2) / (p U_e)^2
U_e      = U + u_s (opposed),   sqrt(U^2 + u_s^2) (concurrent)
```

The variable that carries the boundary is the oxygen margin `X - X_lim(U)`: a floor `x0`, raised
at high speed by the chemistry's need (`sb U_e`) and at low speed by radiation and side loss
(`A / (p U_e)`). The equivalent dimensionless statement is `K + R < 1` (section 1.2). There is
no sharp line anywhere: the width `s` is 1.1 % of oxygen at the optimum speed and grows to 3 % at
1 cm/s and to 1.6 % (concurrent) or 3 % (opposed) at 40 cm/s. Remarks that matter for use.

1. **What "sustained" means.** A gas flame that still exists after three quench times (about 40 s
   for SIBAL) at steady conditions, or reaches the end of the sample. Smoulder spots left in the
   glass matrix are not a flame.
2. **The width is epistemic.** `s` is the standard deviation of `X_lim` under independent normal
   uncertainty of `x0`, `cp`, `sb`, `ar`, `aw`; since `X_lim` is linear in all five, `P(sustained)`
   is exact under that assumption, not an approximation. It says how unsure the theory is about
   where the line is, not how random a burn is. If the oxygen reading has a known error, add its
   variance to `s^2`.
3. **The law never says zero.** As `U_e -> 0` the margin and the width diverge together and
   P tends to `Phi(-A / sd_A)`, 0.002 to 0.03. That is the honest content of a 35 % coefficient.
4. **Ignition and survival share the same unknown.** Do not multiply `P(ignition)` and
   `P(sustained)` at the same condition as if independent; with a shared boundary error the joint
   is close to the smaller of the two. Section 6.3 gives the exact composition.
5. **`U` is the duct speed** as commanded, in a 7.6 cm square duct with the sample in a holder.
   The constants `ar` and `sb` contain that mapping (D3, D5). Another apparatus needs its own `G`.
6. **Range.** 0 to 60 cm/s, 0.12 to 0.30 oxygen, 0.3 to 1.5 atm, thermally thin samples (half
   thickness below about 0.5 mm for PMMA), widths 1 to 40 cm, fresh samples. A re-used, partly
   burnt sample is a different fuel.

## 3. Constants

Origin classes: **P** property table, **T** derived here, **L** read from a published experiment,
**J** judgement. "sd" is one standard deviation of my uncertainty. Units: oxygen in mole fraction,
speed in cm/s, width in cm, pressure in atm.

### 3.1 Physical inputs to the derivations

| Quantity | Value | Class | Source |
|---|---|---|---|
| Stefan-Boltzmann constant | 5.67e-8 W/m^2/K^4 | P | |
| Heat released per kg of oxygen, `dh_O` | 13.1 MJ/kg | P | Huggett 1980 (from memory) |
| Oxygen per unit volume per unit mole fraction, `C_O` | 1.30 kg/m^3 at 300 K, 1 atm | T | ideal gas, D1 |
| Ambient temperature | 300 K | J | cabin |
| SIBAL area density | 18.05 mg/cm^2, of which cotton 13.54 | P | Olson et al. (Saffire duct paper); 18.2 mg/cm^2 and 0.32 mm thick in Zhao et al. 2015 |
| SIBAL heat capacity | cotton 1.26, glass 0.70 J/g/K; blend 1.12 | P | Olson et al. (Saffire duct paper) |
| SIBAL emissivity | 0.80, sd 0.08 | P | same |
| SIBAL surface temperature | 680 K, sd 20 K | T | D3, from the pyrolysis kinetics of the same source |
| Adiabatic flame temperature of cellulose in air | 2250 K, sd 80 K | P/T | from memory; slope 79 K per % O2 by D2 |
| Extinction temperature of the flame | 1550 K, sd 100 K | P | usual range for diffusion flames (from memory) |
| One-step activation energy | 125 kJ/mol, sd 30 % | P | thin-fuel models of the T'ien group (from memory) |
| Air: kinematic viscosity, conductivity at 650 K | 0.16 cm^2/s (300 K); 0.048 W/m/K | P | |
| PMMA: density, heat capacity, surface temperature, emissivity | 1190 kg/m^3; 1.47 J/g/K; 660 K; 0.9 | P | from memory |

### 3.2 SIBAL fabric (75 % cotton, 25 % glass)

| Constant | Value | sd | Class | Origin |
|---|---|---|---|---|
| `x0` floor oxygen | 0.148 | 0.010 | T + L + J | D4: 1700 K rule (T) gives 14 % for the minimum of clean thin cellulose, confirmed by Olson 2004 (L, drop tower and glovebox, tissue); plus 1.9 % heavy cotton (L, 1 g limits, from memory); plus 0.6 % glass (J); minus 1.7 % (T, `2 sqrt(A sb)`). Bracketed by Zhao et al. 2015 (L, BASS-II, same apparatus as this project) |
| `cp` floor shift per e-fold of `1/p` | 0.020 | 0.020 | J | D7 |
| `sb[concurrent]` | 0.00080 per cm/s | 0.00030 | T x J | D5: half the opposed value |
| `sb[opposed]` | 0.0016 per cm/s | 0.0007 | T | D5; check 0.0019 from thin cellulose in partial gravity (L, T'ien et al. 1993, aircraft) |
| `ar[concurrent]` | 0.075 cm/s | 0.026 | T x J | D3: 0.055 (T) times `G = 1.36` |
| `ar[opposed]` | 0.040 cm/s | 0.014 | T x J | D3: 0.055 (T) times `G = 0.73`; ratio 1.9 from Olson 2004 (L, thin cellulose, cabin air, glovebox) |
| `aw` | 0.030 cm^2/s | 0.030 | T (scaling) + J (size) | 1.6: lateral share `c X D / (U_e W)` with `c X D` of 0.02 to 0.06; sign not established |
| `u_s` | 0.15 cm/s | 0.05 | T | D6; published opposed spread 1.2 to 1.3 mm/s (L, BASS and Saffire I) |
| `dx` ignition margin | 0.0040 | inside `sd_a` | J | D8 |
| `sd_a` run-to-run scatter of ignition | 0.0050 | | J | D8: "sporadic" near the boundary (Zhao et al. 2015) |
| `t_req` igniter on-time for 50 % | 2.0 s | factor 1.6 (`sd_lnt = 0.5`) | T + J | D8 |
| `tau_q`, `tau_b` | 12 s, 1.5 s | range 5 to 30 s; 0.5 to 3 s | T + J | D9 |

Derived, for orientation (2.2 cm, concurrent, 1 atm): `A = 0.075 + 0.030 / 2.2 = 0.089`;
`U* = 10.5 cm/s`; `X_min = 16.5 %`.

### 3.3 PMMA sheet, 1 mm (half thickness 0.5 mm)

Covered, with wider uncertainty. The sheet is thermally thin on the quench branch (heat penetrates
0.9 to 1.8 mm during the preheat time at 5 to 20 cm/s, against a half thickness of 0.5 mm) and
marginal above 30 cm/s.

| Constant | Value | sd | Class | Origin |
|---|---|---|---|---|
| `x0` | 0.136 | 0.015 | L + J | PMMA film 0.125 mm, opposed, parabolic flights: minimum 14.7 % (Takahashi et al. 2014); the law's `2 sqrt(A sb)` is 1.8 %, so the film floor is 0.129. Plus 0.007 for the thicker sheet (J: limits rise with thickness in 1 g; the PMMA rod limit of 13.6 % in BASS-II, Ferkul et al. 2014, says the rise is not large) |
| `cp` | 0.020 | 0.020 | J | D7 |
| `sb[concurrent]` | 0.00080 per cm/s | 0.00040 | J | same ratio to opposed as SIBAL |
| `sb[opposed]` | 0.0015 per cm/s | 0.0007 | L + T | the minimum of Takahashi et al. 2014 lies at 6 cm/s: `sb = A / U*^2` with `A = 0.054`; D5 gives the same size |
| `ar[concurrent]` | 0.074 cm/s | 0.030 | T x J | `0.9 * sigma * (660^4 - 300^4) = 9.3 kW/m^2`, 0.054 for `G = 1`, times 1.36 |
| `ar[opposed]` | 0.039 cm/s | 0.016 | T x J | times 0.73 |
| `aw` | 0.030 cm^2/s | 0.030 | J | as SIBAL |
| `u_s` | 0.020 cm/s | 0.010 | T | D6 with 595 g/m^2 per side |
| `dx` | 0.0060 | | J | a slab kernel needs more margin than a fabric kernel |
| `t_req` | 10 s | factor 1.6 | T + J | heating half the sheet at 75 kW/m^2 takes 4 s; with gasification about 10 s |
| `tau_q`, `tau_b` | 100 s, 3 s | range 40 to 250 s; 1 to 6 s | T + J | D9 with the sheet's thermal mass (34 s per side, times three) |

## 4. Predictions (registered before any comparison)

P at 1 atm on the grid asked for. Columns are duct flow speed in cm/s; the last two rows of each
table give the 50 % limit and its standard deviation in percent of oxygen. The `U = 0` column uses
the row's own direction entry; in still gas the opposed tables are the ones to read (1.5). PMMA
tables use the sheet widths flown in BASS-II (2.0 and 1.0 cm; Ferkul et al. 2014); at 2.2 cm the
entries differ from the 2.0 cm table by at most 0.02, at 1.2 cm from the 1.0 cm table by at most
0.07, the largest differences being at 1 to 3 cm/s. The tables were produced by
evaluating the formulas of section 0 with the constants of section 3; no other input.

**T1. SIBAL 2.2 cm, concurrent, P(sustained)**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.00 | 0.42 | 0.98 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| 0.22 | 0.00 | 0.30 | 0.93 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 |
| 0.21 | 0.00 | 0.19 | 0.82 | 0.98 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.96 |
| 0.20 | 0.00 | 0.12 | 0.64 | 0.92 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.97 | 0.87 |
| 0.19 | 0.00 | 0.07 | 0.42 | 0.76 | 0.96 | 0.99 | 0.99 | 0.98 | 0.97 | 0.87 | 0.69 |
| 0.18 | 0.00 | 0.03 | 0.22 | 0.50 | 0.81 | 0.90 | 0.92 | 0.90 | 0.84 | 0.65 | 0.44 |
| 0.17 | 0.00 | 0.02 | 0.09 | 0.24 | 0.51 | 0.65 | 0.68 | 0.64 | 0.55 | 0.36 | 0.22 |
| 0.16 | 0.00 | 0.01 | 0.03 | 0.08 | 0.20 | 0.30 | 0.33 | 0.30 | 0.24 | 0.13 | 0.08 |
| 0.15 | 0.00 | 0.00 | 0.01 | 0.02 | 0.05 | 0.07 | 0.09 | 0.08 | 0.06 | 0.03 | 0.02 |
| 0.14 | 0.00 | 0.00 | 0.00 | 0.00 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.00 | 0.00 |
| *X_lim, %* | 73.9 | 23.6 | 19.4 | 18.0 | 17.0 | 16.6 | 16.5 | 16.6 | 16.8 | 17.5 | 18.2 |
| *sigma, %* | 19.6 | 3.1 | 1.8 | 1.4 | 1.2 | 1.1 | 1.1 | 1.1 | 1.2 | 1.3 | 1.6 |

**T2. SIBAL 2.2 cm, opposed, P(sustained)**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.02 | 0.96 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.91 | 0.71 |
| 0.22 | 0.01 | 0.88 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.98 | 0.83 | 0.59 |
| 0.21 | 0.01 | 0.75 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 | 0.94 | 0.70 | 0.45 |
| 0.20 | 0.01 | 0.57 | 0.96 | 0.99 | 1.00 | 1.00 | 0.99 | 0.95 | 0.84 | 0.53 | 0.32 |
| 0.19 | 0.01 | 0.37 | 0.84 | 0.95 | 0.98 | 0.97 | 0.95 | 0.83 | 0.66 | 0.37 | 0.21 |
| 0.18 | 0.01 | 0.20 | 0.60 | 0.80 | 0.88 | 0.86 | 0.80 | 0.61 | 0.43 | 0.22 | 0.13 |
| 0.17 | 0.01 | 0.09 | 0.32 | 0.50 | 0.62 | 0.59 | 0.52 | 0.35 | 0.23 | 0.12 | 0.07 |
| 0.16 | 0.00 | 0.03 | 0.11 | 0.20 | 0.28 | 0.27 | 0.22 | 0.14 | 0.09 | 0.05 | 0.04 |
| 0.15 | 0.00 | 0.01 | 0.03 | 0.05 | 0.07 | 0.07 | 0.06 | 0.04 | 0.03 | 0.02 | 0.02 |
| 0.14 | 0.00 | 0.00 | 0.00 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 |
| *X_lim, %* | 50.6 | 19.6 | 17.6 | 17.0 | 16.7 | 16.7 | 17.0 | 17.6 | 18.3 | 19.8 | 21.4 |
| *sigma, %* | 13.1 | 2.0 | 1.4 | 1.2 | 1.1 | 1.2 | 1.2 | 1.5 | 1.7 | 2.3 | 3.0 |

**T3. SIBAL 1.2 cm, concurrent, P(sustained)**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.01 | 0.32 | 0.93 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| 0.22 | 0.01 | 0.23 | 0.84 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 |
| 0.21 | 0.01 | 0.15 | 0.70 | 0.95 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 | 0.96 |
| 0.20 | 0.01 | 0.10 | 0.51 | 0.85 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 0.97 | 0.87 |
| 0.19 | 0.00 | 0.06 | 0.32 | 0.66 | 0.93 | 0.98 | 0.99 | 0.98 | 0.96 | 0.86 | 0.68 |
| 0.18 | 0.00 | 0.03 | 0.17 | 0.41 | 0.74 | 0.87 | 0.90 | 0.88 | 0.82 | 0.64 | 0.44 |
| 0.17 | 0.00 | 0.02 | 0.08 | 0.19 | 0.44 | 0.59 | 0.64 | 0.62 | 0.53 | 0.35 | 0.21 |
| 0.16 | 0.00 | 0.01 | 0.03 | 0.07 | 0.17 | 0.26 | 0.29 | 0.28 | 0.22 | 0.13 | 0.08 |
| 0.15 | 0.00 | 0.00 | 0.01 | 0.02 | 0.04 | 0.06 | 0.07 | 0.07 | 0.05 | 0.03 | 0.02 |
| 0.14 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.01 | 0.01 | 0.01 | 0.01 | 0.00 | 0.00 |
| *X_lim, %* | 81.5 | 24.8 | 19.9 | 18.4 | 17.2 | 16.7 | 16.6 | 16.7 | 16.9 | 17.5 | 18.3 |
| *sigma, %* | 24.1 | 3.7 | 2.1 | 1.6 | 1.2 | 1.1 | 1.1 | 1.1 | 1.2 | 1.4 | 1.6 |

**T4. SIBAL 1.2 cm, opposed, P(sustained)**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.03 | 0.81 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.91 | 0.71 |
| 0.22 | 0.03 | 0.69 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.98 | 0.82 | 0.58 |
| 0.21 | 0.03 | 0.55 | 0.95 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 | 0.94 | 0.69 | 0.45 |
| 0.20 | 0.02 | 0.41 | 0.86 | 0.97 | 1.00 | 1.00 | 0.99 | 0.94 | 0.83 | 0.53 | 0.32 |
| 0.19 | 0.02 | 0.27 | 0.69 | 0.88 | 0.96 | 0.96 | 0.94 | 0.82 | 0.65 | 0.36 | 0.21 |
| 0.18 | 0.02 | 0.16 | 0.46 | 0.68 | 0.82 | 0.83 | 0.77 | 0.59 | 0.42 | 0.22 | 0.13 |
| 0.17 | 0.02 | 0.09 | 0.24 | 0.39 | 0.54 | 0.54 | 0.48 | 0.33 | 0.22 | 0.11 | 0.07 |
| 0.16 | 0.01 | 0.04 | 0.10 | 0.16 | 0.23 | 0.23 | 0.20 | 0.13 | 0.09 | 0.05 | 0.04 |
| 0.15 | 0.01 | 0.02 | 0.03 | 0.04 | 0.06 | 0.06 | 0.05 | 0.04 | 0.03 | 0.02 | 0.02 |
| 0.14 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 | 0.01 |
| *X_lim, %* | 58.2 | 20.6 | 18.2 | 17.4 | 16.9 | 16.9 | 17.1 | 17.7 | 18.3 | 19.8 | 21.4 |
| *sigma, %* | 19.1 | 2.7 | 1.7 | 1.4 | 1.2 | 1.2 | 1.3 | 1.5 | 1.7 | 2.3 | 3.0 |

**T5. PMMA sheet 1 mm, 2.0 cm wide, concurrent, P(sustained)**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.00 | 0.55 | 0.98 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| 0.22 | 0.00 | 0.44 | 0.95 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 |
| 0.21 | 0.00 | 0.33 | 0.89 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 | 0.97 |
| 0.20 | 0.00 | 0.24 | 0.79 | 0.96 | 0.99 | 1.00 | 1.00 | 1.00 | 0.99 | 0.97 | 0.91 |
| 0.19 | 0.00 | 0.17 | 0.64 | 0.88 | 0.97 | 0.99 | 0.99 | 0.99 | 0.98 | 0.92 | 0.82 |
| 0.18 | 0.00 | 0.11 | 0.46 | 0.74 | 0.91 | 0.95 | 0.96 | 0.95 | 0.92 | 0.81 | 0.67 |
| 0.17 | 0.00 | 0.06 | 0.30 | 0.54 | 0.77 | 0.84 | 0.86 | 0.84 | 0.79 | 0.64 | 0.50 |
| 0.16 | 0.00 | 0.04 | 0.16 | 0.33 | 0.55 | 0.65 | 0.67 | 0.65 | 0.58 | 0.44 | 0.32 |
| 0.15 | 0.00 | 0.02 | 0.08 | 0.17 | 0.32 | 0.40 | 0.43 | 0.40 | 0.35 | 0.25 | 0.18 |
| 0.14 | 0.00 | 0.01 | 0.03 | 0.07 | 0.14 | 0.19 | 0.21 | 0.20 | 0.17 | 0.12 | 0.08 |
| *X_lim, %* | >1 | 22.6 | 18.2 | 16.8 | 15.8 | 15.4 | 15.3 | 15.4 | 15.6 | 16.3 | 17.0 |
| *sigma, %* | >1 | 3.7 | 2.3 | 1.9 | 1.7 | 1.6 | 1.6 | 1.6 | 1.7 | 1.9 | 2.2 |

**T6. PMMA sheet 1 mm, 2.0 cm wide, opposed, P(sustained)**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.01 | 0.93 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.97 | 0.85 |
| 0.22 | 0.01 | 0.87 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 | 0.92 | 0.76 |
| 0.21 | 0.01 | 0.77 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.98 | 0.85 | 0.65 |
| 0.20 | 0.01 | 0.64 | 0.97 | 0.99 | 1.00 | 1.00 | 1.00 | 0.98 | 0.94 | 0.75 | 0.53 |
| 0.19 | 0.01 | 0.49 | 0.90 | 0.97 | 0.99 | 0.99 | 0.98 | 0.94 | 0.85 | 0.61 | 0.41 |
| 0.18 | 0.01 | 0.34 | 0.78 | 0.90 | 0.95 | 0.94 | 0.92 | 0.83 | 0.71 | 0.46 | 0.29 |
| 0.17 | 0.01 | 0.22 | 0.59 | 0.75 | 0.84 | 0.83 | 0.79 | 0.67 | 0.52 | 0.31 | 0.19 |
| 0.16 | 0.01 | 0.12 | 0.38 | 0.54 | 0.64 | 0.63 | 0.58 | 0.45 | 0.34 | 0.19 | 0.12 |
| 0.15 | 0.01 | 0.06 | 0.20 | 0.31 | 0.39 | 0.39 | 0.35 | 0.25 | 0.18 | 0.10 | 0.07 |
| 0.14 | 0.01 | 0.03 | 0.08 | 0.14 | 0.19 | 0.19 | 0.16 | 0.11 | 0.08 | 0.05 | 0.04 |
| *X_lim, %* | >1 | 19.0 | 16.6 | 15.8 | 15.4 | 15.4 | 15.6 | 16.2 | 16.9 | 18.3 | 19.7 |
| *sigma, %* | >1 | 2.6 | 1.9 | 1.7 | 1.6 | 1.6 | 1.7 | 1.8 | 2.1 | 2.6 | 3.2 |

**T7. PMMA sheet 1 mm, 1.0 cm wide, concurrent, P(sustained)**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.01 | 0.41 | 0.94 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| 0.22 | 0.01 | 0.32 | 0.88 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 |
| 0.21 | 0.01 | 0.25 | 0.78 | 0.96 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 | 0.96 |
| 0.20 | 0.01 | 0.18 | 0.66 | 0.90 | 0.99 | 1.00 | 1.00 | 1.00 | 0.99 | 0.97 | 0.91 |
| 0.19 | 0.01 | 0.13 | 0.51 | 0.79 | 0.95 | 0.98 | 0.99 | 0.98 | 0.97 | 0.92 | 0.81 |
| 0.18 | 0.01 | 0.09 | 0.36 | 0.63 | 0.87 | 0.93 | 0.94 | 0.94 | 0.91 | 0.80 | 0.67 |
| 0.17 | 0.01 | 0.06 | 0.23 | 0.44 | 0.70 | 0.81 | 0.83 | 0.82 | 0.77 | 0.63 | 0.49 |
| 0.16 | 0.01 | 0.04 | 0.13 | 0.26 | 0.48 | 0.60 | 0.64 | 0.62 | 0.56 | 0.43 | 0.31 |
| 0.15 | 0.01 | 0.02 | 0.06 | 0.13 | 0.27 | 0.36 | 0.39 | 0.38 | 0.34 | 0.24 | 0.17 |
| 0.14 | 0.01 | 0.01 | 0.03 | 0.05 | 0.12 | 0.17 | 0.19 | 0.18 | 0.16 | 0.11 | 0.08 |
| *X_lim, %* | >1 | 24.1 | 19.0 | 17.3 | 16.1 | 15.6 | 15.4 | 15.5 | 15.7 | 16.3 | 17.1 |
| *sigma, %* | >1 | 4.5 | 2.6 | 2.1 | 1.7 | 1.6 | 1.6 | 1.6 | 1.7 | 1.9 | 2.2 |

**T8. PMMA sheet 1 mm, 1.0 cm wide, opposed, P(sustained)**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.02 | 0.75 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.96 | 0.84 |
| 0.22 | 0.02 | 0.66 | 0.98 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 | 0.92 | 0.76 |
| 0.21 | 0.02 | 0.55 | 0.95 | 0.99 | 1.00 | 1.00 | 1.00 | 0.99 | 0.98 | 0.85 | 0.65 |
| 0.20 | 0.02 | 0.44 | 0.88 | 0.97 | 0.99 | 1.00 | 0.99 | 0.98 | 0.93 | 0.74 | 0.53 |
| 0.19 | 0.02 | 0.34 | 0.77 | 0.92 | 0.97 | 0.98 | 0.97 | 0.93 | 0.84 | 0.60 | 0.40 |
| 0.18 | 0.02 | 0.25 | 0.62 | 0.81 | 0.91 | 0.92 | 0.90 | 0.82 | 0.70 | 0.45 | 0.29 |
| 0.17 | 0.02 | 0.17 | 0.44 | 0.64 | 0.78 | 0.79 | 0.76 | 0.65 | 0.51 | 0.30 | 0.19 |
| 0.16 | 0.02 | 0.11 | 0.28 | 0.43 | 0.56 | 0.58 | 0.55 | 0.43 | 0.32 | 0.18 | 0.12 |
| 0.15 | 0.02 | 0.07 | 0.15 | 0.24 | 0.33 | 0.35 | 0.32 | 0.24 | 0.17 | 0.10 | 0.07 |
| 0.14 | 0.02 | 0.04 | 0.07 | 0.11 | 0.15 | 0.16 | 0.14 | 0.11 | 0.08 | 0.05 | 0.03 |
| *X_lim, %* | >1 | 20.5 | 17.3 | 16.3 | 15.7 | 15.6 | 15.8 | 16.3 | 16.9 | 18.3 | 19.8 |
| *sigma, %* | >1 | 3.7 | 2.3 | 1.9 | 1.7 | 1.7 | 1.7 | 1.8 | 2.1 | 2.6 | 3.2 |

**T9. SIBAL 2.2 cm, concurrent, P(ignition), igniter on 8 s**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.00 | 0.37 | 0.96 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 |
| 0.22 | 0.00 | 0.25 | 0.88 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.98 |
| 0.21 | 0.00 | 0.16 | 0.74 | 0.96 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.98 | 0.92 |
| 0.20 | 0.00 | 0.10 | 0.55 | 0.86 | 0.98 | 0.99 | 0.99 | 0.99 | 0.98 | 0.93 | 0.80 |
| 0.19 | 0.00 | 0.05 | 0.34 | 0.66 | 0.90 | 0.95 | 0.96 | 0.95 | 0.91 | 0.78 | 0.59 |
| 0.18 | 0.00 | 0.03 | 0.17 | 0.40 | 0.69 | 0.80 | 0.82 | 0.79 | 0.72 | 0.53 | 0.35 |
| 0.17 | 0.00 | 0.01 | 0.07 | 0.17 | 0.38 | 0.50 | 0.54 | 0.50 | 0.42 | 0.27 | 0.16 |
| 0.16 | 0.00 | 0.00 | 0.02 | 0.05 | 0.14 | 0.21 | 0.23 | 0.21 | 0.16 | 0.09 | 0.05 |
| 0.15 | 0.00 | 0.00 | 0.00 | 0.01 | 0.03 | 0.05 | 0.06 | 0.05 | 0.04 | 0.02 | 0.01 |
| 0.14 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.01 | 0.01 | 0.01 | 0.01 | 0.00 | 0.00 |
| *X_lim, %* | 73.9 | 23.6 | 19.4 | 18.0 | 17.0 | 16.6 | 16.5 | 16.6 | 16.8 | 17.5 | 18.2 |
| *sigma, %* | 19.6 | 3.1 | 1.8 | 1.4 | 1.2 | 1.1 | 1.1 | 1.1 | 1.2 | 1.3 | 1.6 |

**T10. SIBAL 2.2 cm, opposed, P(ignition), igniter on 8 s**

| X_O2 \ U (cm/s) | 0 | 1 | 2 | 3 | 5 | 7.5 | 10 | 15 | 20 | 30 | 40 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0.23 | 0.02 | 0.92 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 | 0.88 | 0.66 |
| 0.22 | 0.01 | 0.83 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 0.99 | 0.96 | 0.77 | 0.53 |
| 0.21 | 0.01 | 0.68 | 0.98 | 0.99 | 1.00 | 1.00 | 0.99 | 0.97 | 0.90 | 0.63 | 0.40 |
| 0.20 | 0.01 | 0.49 | 0.91 | 0.97 | 0.99 | 0.99 | 0.97 | 0.90 | 0.76 | 0.47 | 0.28 |
| 0.19 | 0.01 | 0.30 | 0.74 | 0.89 | 0.94 | 0.93 | 0.89 | 0.74 | 0.57 | 0.31 | 0.18 |
| 0.18 | 0.01 | 0.16 | 0.49 | 0.67 | 0.77 | 0.75 | 0.68 | 0.50 | 0.35 | 0.18 | 0.11 |
| 0.17 | 0.00 | 0.07 | 0.24 | 0.38 | 0.48 | 0.46 | 0.40 | 0.26 | 0.17 | 0.09 | 0.06 |
| 0.16 | 0.00 | 0.02 | 0.08 | 0.14 | 0.19 | 0.19 | 0.16 | 0.10 | 0.07 | 0.04 | 0.03 |
| 0.15 | 0.00 | 0.01 | 0.02 | 0.03 | 0.05 | 0.05 | 0.04 | 0.03 | 0.02 | 0.01 | 0.01 |
| 0.14 | 0.00 | 0.00 | 0.00 | 0.00 | 0.01 | 0.01 | 0.01 | 0.01 | 0.00 | 0.00 | 0.01 |
| *X_lim, %* | 50.6 | 19.6 | 17.6 | 17.0 | 16.7 | 16.7 | 17.0 | 17.6 | 18.3 | 19.8 | 21.4 |
| *sigma, %* | 13.1 | 2.0 | 1.4 | 1.2 | 1.1 | 1.2 | 1.2 | 1.5 | 1.7 | 2.3 | 3.0 |

### 4.1 The same predictions as boundary numbers

50 % points, 1 atm. "Quench" and "blow-off" are the two duct speeds at which `X = X_lim`.

| Case | `X_min` at `U*` | Quench speed at 21 / 19 / 18 / 17 % | Blow-off speed at 21 / 19 / 18 / 17 % |
|---|---|---|---|
| SIBAL 2.2 cm, concurrent | 16.5 % at 10.5 cm/s | 1.5 / 2.2 / 3.0 / 4.9 cm/s | 76 / 50 / 37 / 23 cm/s |
| SIBAL 2.2 cm, opposed | 16.7 % at 5.6 cm/s | 0.7 / 1.2 / 1.7 / 3.0 cm/s | 38 / 25 / 18 / 10 cm/s |
| SIBAL 1.2 cm, concurrent | 16.6 % at 11.2 cm/s | 1.7 / 2.5 / 3.4 / 5.8 cm/s | 76 / 50 / 37 / 22 cm/s |
| SIBAL 1.2 cm, opposed | 16.8 % at 6.2 cm/s | 0.9 / 1.5 / 2.2 / 4.2 cm/s | 38 / 24 / 18 / 9 cm/s |
| PMMA 1 mm, 2.0 cm, concurrent | 15.3 % at 10.6 cm/s | 1.2 / 1.7 / 2.1 / 2.8 cm/s | 91 / 66 / 53 / 40 cm/s |
| PMMA 1 mm, 2.0 cm, opposed | 15.4 % at 6.0 cm/s | 0.7 / 1.0 / 1.3 / 1.7 cm/s | 49 / 35 / 28 / 21 cm/s |

Blow-off speeds above 60 cm/s are outside the stated range and are given only to show where the
line goes.

Width of the boundary for SIBAL 2.2 cm, concurrent:

| Duct speed | 50 % limit | sd | 10 % to 90 % band | Largest contributor to the width |
|---|---|---|---|---|
| 2 cm/s | 19.4 % | 1.8 % | 17.1 to 21.7 % | radiative coefficient (1.5 %) |
| 5 cm/s | 17.0 % | 1.2 % | 15.5 to 18.5 % | floor (1.0 %), radiative (0.6 %) |
| 10 cm/s | 16.5 % | 1.1 % | 15.1 to 17.9 % | floor (1.0 %) |
| 20 cm/s | 16.8 % | 1.2 % | 15.3 to 18.4 % | floor (1.0 %), blow-off slope (0.6 %) |
| 40 cm/s | 18.2 % | 1.6 % | 16.2 to 20.2 % | blow-off slope (1.2 %) |

In the flow direction, in air: P = 0.1 at 0.8 cm/s, 0.5 at 1.5 cm/s, 0.9 at 2.3 cm/s. At 18 %:
0.1 at 1.5 cm/s, 0.5 at 3.0 cm/s, 0.9 at 7.4 cm/s.

Ignition and survival together, shared boundary error (section 6.3), SIBAL 2.2 cm concurrent,
igniter on 8 s: (21 %, 10 cm/s) 1.00; (18 %, 10 cm/s) 0.82; (17 %, 5 cm/s) 0.36;
(21 %, 2 cm/s) 0.74.

## 5. What would falsify it

Each is an observation on burns at steady conditions (flow held for at least 40 s, fresh sample,
oxygen read to 0.3 %).

1. **The quench branch is not a hyperbola in the oxygen margin.** The law says the quench speed
   times the oxygen excess over the floor is a constant, `U_q (X - x0 - sb U_q) = A / p`: for the
   2.2 cm fabric in concurrent flow 1.5 cm/s in air, 3.0 cm/s at 18 %, 4.9 cm/s at 17 %. It is
   wrong if flames last at less than half these speeds or go out at more than twice them, and
   wrong in form if the quench speed does not rise as oxygen falls (a vertical quench line, as in
   a pure `R_rad = 1` criterion). The first would change `ar` (the factor `G`); the second would
   replace the oxygen-enthalpy normalisation by a temperature-difference one and move the oxygen
   dependence into the floor.
2. **The direction crossover is absent or reversed.** The law says that at the same oxygen the
   opposed flame survives to about half the flow speed of the concurrent flame (0.7 against
   1.5 cm/s in air) and blows off at about half the speed (38 against 76 cm/s in air; 18 against
   37 cm/s at 18 %). It is wrong if a concurrent flame in air holds steadily below the opposed
   quench speed, or if an opposed flame survives 40 cm/s at 19 % or less. Either would remove the
   direction split of `ar` or of `sb` and leave one value for both.
3. **The floor is elsewhere, or is not flat.** The law says no steady flame of the 2.2 cm fabric
   at or below 15 % oxygen at any speed (P at most 0.09), a broad minimum of 16.5 % between 7 and
   15 cm/s, and reliable burning at or above 19 % between 5 and 20 cm/s (P at least 0.96). A
   steady flame at 15 %, or a failure to sustain at 19.5 % after a clean ignition at 5 to
   20 cm/s, moves `x0` by the amount seen. Both together would mean the width is too small or a
   variable is missing (moisture, sample history, oxygen reading). A minimum that sits at 3 cm/s
   or at 25 cm/s instead of near 10 cm/s would change `sb`: `U* = sqrt(A / sb)`.

A weaker fourth: the law gives the 1.2 cm strip quench speeds about 15 % above those of the
2.2 cm strip. If the narrow strip quenches at clearly lower speed, oxygen side diffusion wins
over side loss (Shih and T'ien 2003) and `aw` is negative.

Calibration is checked separately from these: of all burns the law gives a P between 0.3 and 0.7,
about half should have sustained.

## 6. Interface

### 6.1 Inputs

| Input | Symbol | Unit | Already in `FrontInput` |
|---|---|---|---|
| Oxygen mole fraction | `X` | 1 | `x_o2` |
| Pressure | `p` | atm | no (new; 1.0 for every cabin-pressure burn) |
| Flow speed and its history | `U(t)` | cm/s | `flow` |
| Flow direction relative to spread | `dir` | 0 concurrent, 1 opposed | no (the present forward is concurrent only) |
| Sample width | `W` | cm | as `width_ratio` times the 2 cm class reference |
| Material | `mat` | index | no (new: 0 SIBAL fabric, 1 PMMA sheet 1 mm) |
| Igniter on-time, flow during ignition | `t_on`, `U_ign` | s, cm/s | `igniter_on_s`, first entry of `flow` |

Thickness and area density do not appear as live inputs: they are folded into the per-material
constants `u_s`, `tau` and `t_req`. A new material or thickness is a new row, typed with its
derivation (D3, D4, D6, D9).

### 6.2 Tensors

Axis `M` = material (2), axis `D` = direction (2: concurrent, opposed), axis `B` = branch (2:
radiative quench, blow-off).

| Name | Shape | Bounds | Unit | Meaning |
|---|---|---|---|---|
| `sus.x0` | `[M]` | 0.08 to 0.30 | mole fraction | floor of the oxygen limit |
| `sus.cp` | `[M]` | 0 to 0.10 | mole fraction per e-fold of `1/p` | pressure shift of the floor |
| `sus.sb` | `[M, D]` | 0 to 0.01 | mole fraction per cm/s | blow-off slope |
| `sus.ar` | `[M, D]` | 0 to 0.5 | mole fraction cm/s atm | radiative-loss coefficient |
| `sus.aw` | `[M]` | -0.2 to 0.3 | mole fraction cm^2/s atm | lateral coefficient (loss positive) |
| `sus.us` | `[M]` | 0 to 5 | cm/s | self-spread speed |
| `sus.sd_x0` | `[M]` | 0.001 to 0.05 | mole fraction | sd of `x0` |
| `sus.sd_cp` | `[M]` | 0 to 0.10 | as `cp` | sd of `cp` |
| `sus.sd_sb` | `[M, D]` | 0 to 0.01 | as `sb` | sd of `sb` |
| `sus.sd_ar` | `[M, D]` | 0 to 0.5 | as `ar` | sd of `ar` |
| `sus.sd_aw` | `[M]` | 0 to 0.3 | as `aw` | sd of `aw` |
| `ign.dx` | `[M]` | -0.02 to 0.05 | mole fraction | extra oxygen needed for a kernel to survive |
| `ign.sd_a` | scalar | 0.001 to 0.03 | mole fraction | run-to-run scatter of ignition |
| `ign.t_req` | `[M]` | 0.1 to 120 | s | igniter on-time at which half the attempts light |
| `ign.sd_lnt` | scalar | 0.1 to 2 | ln | spread of that on-time |
| `ext.tau` | `[M, B]` | 0.1 to 300 | s | time for a flame outside the boundary to go out |
| `ext.width` | scalar | 0.0005 to 0.01 | mole fraction | smoothing of the inside/outside switch |

### 6.3 Strict forward

No fitting, no search, no sampling: closed forms and one fixed quadrature.

```text
static (one condition):
  U_e    = U + us[m]                      if opposed
         = sqrt(U^2 + us[m]^2)            if concurrent
  A      = ar[m,d] + aw[m] / W
  X_lim  = x0[m] + cp[m] ln(1/p) + sb[m,d] U_e + A / (p U_e)
  s^2    = sd_x0[m]^2 + (sd_cp[m] ln(1/p))^2 + (sd_sb[m,d] U_e)^2
           + (sd_ar[m,d]^2 + (sd_aw[m] / W)^2) / (p U_e)^2
  P_sus  = Phi((X - X_lim) / s)
  P_ign  = Phi((X - X_lim(U_ign) - dx[m]) / sqrt(s(U_ign)^2 + sd_a^2)) * Phi(ln(t_on / t_req[m]) / sd_lnt)

with a flow history (K-point Gauss-Hermite rule for a standard normal, nodes e_k, weights w_k;
K = 9 is enough; the rule is a constant of the spec, not an authored tensor):
  for each node k:
    lit_k     = Phi((X - X_lim(U_ign) - dx[m] - s(U_ign) e_k) / sd_a) * Phi(ln(t_on / t_req[m]) / sd_lnt)
    margin_k(t) = X - X_lim(U(t)) - s(U(t)) e_k
    w_r(t)    = (A / (p U_e)) / (A / (p U_e) + sb[m,d] U_e)
    rate_k(t) = (w_r / tau[m,0] + (1 - w_r) / tau[m,1]) * Phi(-margin_k(t) / ext.width)
    alive_k(t) = lit_k * exp(-integral of rate_k from igniter-off to t)     (explicit steps of the flow history)
  P_flame(t) = sum_k w_k alive_k(t)
  P_quenched(t) = sum_k w_k lit_k (1 - exp(-integral of rate_k))
```

The node value `e_k` stands for "where the true boundary is, in units of my uncertainty", and it
is the same at every time of a burn: that is what makes a flow step down and back up behave
correctly (a flame that survived 3 cm/s for a minute is evidence about the boundary, and the same
node carries it). For constant conditions and long times `P_flame` tends to the joint probability
of ignition and survival, and with `dx = 0` and `sd_a -> 0` to `P_sus`.

A note for whoever wires this in: the present forward writes quenching as one hazard,
`exp(N_quench(u, c_o2, c_w))`. A single positive hazard drives survival to zero at every
condition given time, so it cannot represent a boundary whose position is uncertain; the mixture
over `e_k` above is the smallest structure that can. It needs a new forward version.

### 6.4 Literal assignments

In the edit language of `docs/wsp/PROTOCOL.md` section 2. Material index 0 is SIBAL fabric, 1 is
the PMMA sheet of 1 mm; direction index 0 is concurrent, 1 is opposed; branch index 0 is radiative
quench, 1 is blow-off.

```text
# floor of the oxygen limit: D4 (SIBAL), Takahashi 2014 film minimum plus thickness judgement (PMMA)
sus.x0[0:2] = [0.148, 0.136]
# pressure shift of the floor per e-fold of 1/p: judgement, D7
sus.cp[0:2] = [0.020, 0.020]
# blow-off slope per cm/s, rows material, columns [concurrent, opposed]: D5
sus.sb[0:2, 0:2] = [[0.00080, 0.0016], [0.00080, 0.0015]]
# radiative-loss coefficient: 0.055 (SIBAL) and 0.054 (PMMA) for G = 1, times G = 1.36 and 0.73: D3
sus.ar[0:2, 0:2] = [[0.075, 0.040], [0.074, 0.039]]
# lateral coefficient: section 1.6
sus.aw[0:2] = [0.030, 0.030]
# self-spread speed, de Ris thin limit: D6
sus.us[0:2] = [0.15, 0.020]
# standard deviations of the five coefficients
sus.sd_x0[0:2] = [0.010, 0.015]
sus.sd_cp[0:2] = [0.020, 0.020]
sus.sd_sb[0:2, 0:2] = [[0.00030, 0.00070], [0.00040, 0.00070]]
sus.sd_ar[0:2, 0:2] = [[0.026, 0.014], [0.030, 0.016]]
sus.sd_aw[0:2] = [0.030, 0.030]
# ignition by a hot wire at the sample edge: D8
ign.dx[0:2] = [0.0040, 0.0060]
ign.sd_a = 0.0050
ign.t_req[0:2] = [2.0, 10.0]
ign.sd_lnt = 0.50
# time to go out, rows material, columns [radiative quench, blow-off]: D9
ext.tau[0:2, 0:2] = [[12.0, 1.5], [100.0, 3.0]]
ext.width = 0.0020
```

## 7. Sources and limits of this document

### 7.1 Sources, and what was taken from each

Read in this session (text extracted from the public copy):

| Source | Apparatus | Taken |
|---|---|---|
| X. Zhao, J. S. T'ien, P. V. Ferkul, S. L. Olson, "Concurrent flame growth, spread and extinction over composite fabric samples in low speed purely forced flow in microgravity", 9th US National Combustion Meeting, 2015 (NTRS 20150008962) | BASS and BASS-II, ISS glovebox duct 7.6 cm square | fabric 75/25, 0.32 mm, 18.2 mg/cm^2; widths 1.2 and 2.2 cm, length 10 cm; flow 1 to 55 cm/s; 16 to 21 % O2, 1 atm; Kanthal hot wire; statements: quench speeds 1 to 5 cm/s rising as oxygen falls, flat bottom not resolved, ignition sporadic near the boundary, narrow sample has the smaller domain, three-dimensional shrinking at quench; steady spread labelled down to 16.9 % |
| P. Ferkul, S. Olson, M. Johnston, J. T'ien, "Flammability aspects of a cotton-fiberglass fabric in opposed and concurrent airflow in microgravity", ASGSR 2012 slides (NTRS 20130012832) | BASS | 18 mg/cm^2; opposed spread 1.21 mm/s and concurrent 2.55 mm/s at 10 cm/s; opposed flame followed from 10 down to about 1 cm/s; concurrent quench around 1 cm/s; no downward spread in air in 1 g; the sketch of the U-shaped boundary |
| P. V. Ferkul et al., "Combustion of solids in microgravity: results from the BASS-II experiment", ASGSR 2014 poster (NTRS 20160000593) | BASS-II | flow 0 to 55 cm/s, oxygen 21 % and below by nitrogen; PMMA film 0.1 mm: radiative quench below 2 cm/s, near blow-off at 40 cm/s; PMMA rods steady at 13.6 % against a 1 g blow-off limit of 16 %; fabric: 6 quenching cases, 1 blow-off case (1.2 cm), sawtooth hot wire |
| P. V. Ferkul et al., "Thickness and fuel preheating effects on material flammability in microgravity from the BASS experiment", ASGSR 2013 slides (NTRS 20140011099) | BASS | oxygen 17 to 21.5 %; concurrent quench around 1 cm/s |
| S. L. Olson et al., "The effect of duct size, sample size, and fuel composition on concurrent flame spread over large cellulose samples in microgravity" (accepted manuscript, UCL Discovery 10164340) | Saffire, 40.6 cm wide samples | SIBAL 18.05 mg/cm^2 (13.54 cotton); heat capacities; emissivity 0.8; pyrolysis kinetics; igniter 183 W for 8 s over 40.6 cm |
| P. Ferkul et al., "Results of large-scale spacecraft flammability tests", 2017 slides (NTRS 20170008805) | Saffire I and II | spread 1.8 mm/s concurrent and 1.3 mm/s opposed at 20 cm/s in air; igniter 80 W for 9.2 s on 5 cm |
| Y. Li, Y.-T. T. Liao, P. V. Ferkul, "Effect of flow duct height on concurrent-flow flame spread and the near-limit oscillation", CSSCI 2018 (NSF PAR 10082431) | model of the BASS duct | duct 7.6 cm x 7.6 cm x 20 cm; wall heat loss and oxygen starvation as the duct narrows |
| S. Takahashi, K. Tsuboi, T. Ihara, S. Bhattacharjee, "Modeling of flammability limits of thin materials in microgravity environment", Int. J. Microgravity Sci. Appl. 31 (2014) 179 | parabolic flights and drop shaft, PMMA 0.125 mm, opposed flow | the form `eta + R_rad + 1/Da = 1`; definitions of `R_rad` and `Da`; minimum oxygen 14.7 % at 6 cm/s against 17 % in 1 g; the underestimate of the uncoupled model (11 %) and of the coupled one (13 %) |
| S. Bhattacharjee, C. Paolini, K. Wakai, S. Takahashi, "Extinction criteria for opposed-flow flame spread in a microgravity environment", NASA/CP-2003-212376 | theory, MGLAB data | the leading-edge energy balance with surface re-radiation; relative speed `V_g + V_f`; thickness enters only through the self-spread speed |
| J. S. T'ien, K. R. Sacksteder, P. V. Ferkul, G. D. Grayson, "Combustion of solid fuel in very low speed oxygen streams", NASA CP (N93-20207), 1993 | model; 5 s drop tower; KC-135 partial gravity, tissue | no low-speed limit without surface radiation; quench by shrinking, blow-off by sweeping; downward spread at 14 % and upward at 12 % in partial gravity; low-pressure limits exist |
| S. L. Olson, "Most probable fire scenarios in spacecraft and extraterrestrial habitats", NASA/TM-2004-213114 p. 123 | summary of drop tower and glovebox work on thin cellulose | 1 g opposed limit 16.5 %, flames at 14 % at 5 to 20 cm/s; in cabin air quench at 0.5 cm/s opposed and 2 cm/s concurrent |
| S. L. Olson, J. S. T'ien, "Buoyant low stretch diffusion flames beneath cylindrical PMMA samples" (NTRS 20000004509; Combust. Flame 121, 2000) | 1 g, large PMMA cylinders | quench at 1.5 to 3 1/s in air; losses constant while feedback falls; in-depth loss comparable to surface radiation for a thick fuel |
| S. L. Olson et al., DARTFire manuscript (NTRS 20030111833) | sounding rocket, thick PMMA, 50 % O2 | surface radiation of 1 W/cm^2 scale; total relative losses above 80 % at extinction; ventilation above 5 cm/s is normal in a cabin |

Known only from an abstract or a search summary in this session: Zhao, Liao, Johnston, T'ien,
Ferkul, Olson, Proc. Combust. Inst. 36 (2017) 2971 (the final fabric paper); Olson and Ferkul,
Combust. Flame 180 (2017) 217 (PMMA rods, 13.6 to 22.2 % O2); Marcum, Olson, Ferkul 2016 (linear
1 g blow-off boundary); Shih and T'ien, Combust. Theory Model. 7 (2003) (side diffusion of oxygen
near quench); Johnston et al., Fire Safety J. 71 (2015) 279 (self blow-off of upward SIBAL flames
in 1 g); Link et al., Sci. Rep. 2018 (PMMA cylinders: 17 % in microgravity against 18 % in 1 g);
Olson et al., ICES 2019 (BASS-II limits against 1 g); Hirsch, Williams, Beeson 2006 (pressure
and oxygen thresholds).

From memory, not re-read: T'ien, Combust. Flame 65 (1986) 31; Olson, Ferkul, T'ien, 22nd Symp.
(1988) 1213; Olson, Combust. Sci. Technol. 76 (1991) 233; Ferkul and T'ien, Combust. Sci. Technol.
99 (1994) 345; de Ris 1969; Huggett 1980; the limiting oxygen index of cotton fabric; Planck-mean
absorption coefficients; the one-step activation energy.

### 7.2 What I could not find or could not check

- The full text of the 2017 fabric paper, with the final quench boundary and the model's
  comparison. I have only the 2015 conference version.
- The numbers of the 1988 and 1991 thin-cellulose maps (still-gas limit, position of the minimum).
  I rely on Olson's 2004 summary and on memory.
- A measured oxygen limit for SIBAL fabric in 1 g, upward or downward. Only "does not spread
  downward in air".
- Whether the opposed SIBAL flame in air blew off at 44 cm/s. The slides say blow-off "is
  attempted". The law gives P = 0.4 there, which is what not knowing looks like.
- The BASS igniter power and on-time, and the distance from the holder's leading edge to the
  sample. Saffire igniter values were used for the first; 3 cm was assumed for the second, in an
  estimate only (D3).
- How well the duct speed is known at 1 to 2 cm/s. The quench branch is steep there: a 0.5 cm/s
  error in the speed is worth 2 to 4 % oxygen.
- The size and direction of the pressure effect on the floor (`cp`). Irrelevant at 1 atm.
- Any limit for a PMMA sheet of exactly 1 mm. The row is an interpolation between a 0.125 mm film
  and rods.
- Gas-phase radiation is lumped into `G`, not modelled; its share grows at the lowest speeds, so
  the true quench branch may be steeper than `1 / U` below 1.5 cm/s.
- The mapping from duct speed to the speed at the flame (boundary layer on the holder, thermal
  expansion in a small duct) is inside `ar` and `sb`. In a larger duct or on a wide sample the
  same constants will not hold without a new `G`.

### 7.3 Exposure: what this author saw

- **Repository.** `crates/ignis-wsp/src/front.rs` (header and input structures),
  `docs/wsp/PROTOCOL.md` sections 1 to 4, `AGENTS.md`, and the file names (not contents) of
  `docs/wsp/research/`. Nothing under `runs/`, `data/`, `holdout/`, `docs/wsp/feedback/`,
  `docs/wsp/RESULTS*`, `docs/wsp/cards/`, `configs/models/bass/`,
  `tools/wsp/bass_optical/tracks/`, and not `docs/wsp/research/phys_bass.md`.
- **Published summaries of the same burns.** The BASS and BASS-II publications above describe
  burns this project holds. Their stated conclusions were read and are cited where used. They
  bear on three choices: the standard deviation of `x0` (1.0 % instead of the 1.5 % the
  independent chain of D4 supports); the upper side of `sb[concurrent]` (flames in air at
  40 cm/s); and my confidence in the centre of `G`. The values of `x0`, `ar` and `sb` themselves
  follow from the derivations and from sources on other fuels and apparatus, and were not moved
  after the comparison of section 1.9; but I had read the statements before I settled the
  derivations and cannot exclude that they steered them. This document is therefore blind to the
  project's files and not blind to the literature on its experiment.
- **One table of per-test outcomes.** The appendix of Zhao et al. 2015 lists the fabric tests
  with flow, oxygen and an outcome word. It was in the text I extracted. Its columns came out
  misaligned and I did not reconstruct it; no constant was set from it. One row, if I read it
  right, is a 1.2 cm sample near 17 % oxygen that blew off only above 40 cm/s. The law gives
  such a flame P = 0.2 to 0.35 of lasting at 30 to 40 cm/s (table T3), so that row would argue
  for a smaller `sb[concurrent]`, nearer 0.0005. I left the constant where D5 put it and record
  the disagreement here instead.
- **Pretraining.** The author's training may include these papers; that cannot be measured.

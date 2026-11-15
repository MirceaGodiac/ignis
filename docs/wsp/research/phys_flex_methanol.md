# Physics of a methanol droplet burning in microgravity, for a mechanistic droplet model

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. Outcomes of train runs only; of validation runs the conditions only.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# FLEX methanol dossier: physics, train confrontation, audit of D3, blueprint

**Bottom line.** A mechanistic model built only from literature constants reproduces, on the 68 train runs, the rate trends, the flame/droplet ratio and its growth, the heat-up, the N2 and He critical sizes and the unlit evaporation. It does not reproduce the water extinction as a thermal limit, the timing of an early flame loss, radiation in CO2, or the rate decline with burn progress. Three closures therefore stay empirical, each in a physical variable: the rate level, the liquid water fraction at flame-out, and the radiative strength with a saturating CO2 term.

- **Largest audit finding:** D3's critical diameter in air (9 mm) is about twice what physics (4.5 mm) and the Shuttle-era literature (4–4.5 mm critical, 6 mm fully transient) give.
- **Train-read numbers I typed** (medians and brackets, no search): c_I = 0.61, Y_w* = 0.30, Ψ* = 0.45, flame-size factor 0.9, helium rate bias −0.20.
- **Not re-checked by me:** the CO2 band factor 0.44 and the radiant fractions "10 % at 1 mm, 70 % at 5 mm"; both come second-hand from a helper.

All scripts and tables are in `D:\dev\SpaceApps-Ardealu\runs\dev\research6\flex_methanol\` (the earlier agent left that folder empty). Units below: mm, s, mm²/s unless stated.

## 1. Physics, mechanism by mechanism

### 1.1 Quasi-steady burning (two-zone flame sheet, my derivation from the classical theory)

```
mdot/(4 pi r_s) = c_I I_i + G_o          K = 8 (c_I I_i + G_o) / rho_l
I_i = Int_{T_s}^{T_f} lambda_i(T) dT / (L_eff + h_i(T) - h_i(T_s))   ~ g_i ln(1 + B_f),  B_f = cp_i (T_f - T_s)/L_eff
G_o = <rho D_O> ln(1 + nu),   nu = Y_O,inf / (sigma eps_F),  sigma = 1.498 kg O2/kg fuel
r_f,QS / r_s = 1 + c_I I_i / G_o = rho_l K / (8 G_o)
T_f0 = T_inf + [(eps_F Q - eps_w L_w)/cp_prod] [1 - (1 + nu)^(-1/Le_O)]
```

- I_i is the conductance between droplet and flame; G_o is the oxygen conductance outside the flame. ⟨ρD_O⟩ is the harmonic mean between T∞ and T_f, equivalent to about 930 K.
- Le_O = λ/(c_p ρ D_O) of the outer gas: 0.98 in N2, 0.91–0.96 in CO2, 1.3–1.7 at 20–45 % He.
- Q = 19.92 MJ/kg (liquid fuel to water vapour). T_s = boiling point: 328.9 / 337.7 / 368–371 K at 0.7 / 1 / 3 atm. L = 1118 / 1101 / 1034 kJ/kg.
- Air, 1 atm: T_f0 = 2259 K (no dissociation), I_i = 7.3e-5, G_o = 0.83e-5 kg m⁻¹ s⁻¹.
- The classical recipe (Law–Williams as quoted by the FLEX papers) is K = 8λ ln(1+B)/(ρ_l c_pF), with λ = 0.4 λ_F + 0.6 λ_ambient at (T_s + T_f)/2.

How it moves:

| Input | Mechanism |
|---|---|
| Oxygen | through ν and T_f |
| CO2 | lower Y_O at equal x_O2 (heavier gas) and higher product heat capacity |
| He | conductivity up, but Le_O > 1 cools the flame; the two nearly cancel |
| Pressure | only through T_s and L |
| Size | not at all in this limit; it enters through radiation and heat-up |

### 1.2 Heat-up and the early transient

The liquid is a sphere whose surface is held at T_s.

```
q_l = (k_l dT / r_s) f(Fo),   Fo = alpha_l t / r_s^2 = 0.376 s      (s = clock, alpha_l = 0.094 mm2/s)
f = 1/sqrt(pi Fo) - 1  (Fo < 0.03),  else 2 Sum exp(-n^2 pi^2 Fo)
L_eff = L + q_l / (mass flux);   mean liquid temperature fraction = 6 sqrt(Fo/pi) - 3 Fo  (short time)
D_apparent ~ rho_l(T_mean)^(-1/3),  d rho_l/dT = -0.96 kg/m3/K
```

- Fo depends only on the clock, which is why heat-up scales with D0².
- 3 mm droplet in air: the sink lowers the true rate to 0.78 / 0.86 / 0.91 / 0.95 / 0.98 of plateau at 0.25 / 0.5 / 1 / 2 / 4 s.
- Thermal swelling raises D by 15 / 21 / 28 / 36 / 49 µm at 0.25 / 0.5 / 1 / 2 / 8 s. Between 0.5 and 1 s that hides 0.08 mm²/s of regression.
- In the simulation most of the early apparent deficit is swelling, not slower vaporisation. I could not switch the sink off cleanly, so the split is approximate.

### 1.3 Flame standoff and its growth

The flame sits where the oxygen field lets it, and that field is still building:

```
r_f(t) = r_QS erfc( r_f / sqrt(4 D_ff t) )      late: r_f/r_QS ~ 1 - r_f / sqrt(pi D_ff t)
```

- D_ff = D_O in the ambient mixture at about 600 K: 0.69 cm²/s in N2 at 1 atm, ∝ 1/p, about ×1.3 at 30 % He, ×0.87 at 50 % CO2.
- The approach is algebraic. A 3–4 mm droplet reaches about 0.75 of r_QS at 5 s and never reaches it.
- Classical theory's "standoff several times too large" comes from using the inner conductance outside the flame.

### 1.4 Water uptake

Flux fractions through the surface (ε_F + ε_w = 1, ε_w < 0 is absorption), from the Stefan-flow solution:

```
R_F = (1+B_f)^Le_F,  R_w = (1+B_f)^Le_w,   Y_Fs = eps_F (1 - 1/R_F),   Y_ws = rho_v(x) Y_Fs   (VLE ratio)
Y_wf = (1.1245 eps_F + eps_w) phi_w,   phi_w = 1 - (1+nu)^(-D_O/D_w)
eps_w = (R_w c1 - 1.1245 phi_w) / (R_w - 1 + R_w c1 - 0.1245 phi_w),   c1 = rho_v (1 - 1/R_F)
dm_F/dt = -eps_F mdot,   dm_W/dt = -eps_w mdot      (well-mixed liquid)
```

- Le_w = 0.64–0.66 (N2, CO2), 0.77–0.97 (He); Le_F = 0.92–1.15.
- Absorption starts at ε_w ≈ −0.10 and reverses when the liquid holds 14–17 % water. After that the liquid enriches by distillation.
- **Why about half the initial diameter:** the model puts the liquid at about 30 % water when D/D0 ≈ 0.47–0.51. That is the published FLEX statement, and the measured flame-out sits there (section 2).
- **What moves it:**
  - the absorption rate, through Y_wf/(R_w − 1), hence ν and Le_w;
  - flame strength: above T_f0 ≈ 2400 K the flame tolerates more water, and at 30 % O2 and above it burns to the water-only limit, De/D0 = 0.26;
  - nearness to the radiative limit;
  - liquid mixing.
- A diffusion-limited liquid makes De smaller, not larger (published De/D0 falls from 0.34 to 0.06 as liquid Péclet number goes from below 1 to 20; my frozen-liquid variant never extinguished).

### 1.5 Radiative loss and radiative extinction

```
chi = c_V 4 sigma_SB kappa (T^4 - T_inf^4) (4/3) pi r_f^3 / (mdot_F Q)
kappa = p [x_H2O a_H2O(T) + f_CO2 x_CO2 a_CO2(T)]
T_f = T_inf + (T_f0 - T_inf)(1 - chi);   flame lost when T_f < T_ext   <=>   chi > chi_c = (T_f0 - T_ext)/(T_f0 - T_inf)
D_crit ~ sqrt( chi_c rho_l K Q / (c_V kappa sigma_SB T_ext^4 S^3) )
```

- T_ext ≈ 1200 K: burner-flame radiative extinction is published at 1100–1200 K.
- c_V is 1–3 from the published radiant fractions (second-hand).
- CO2 lowers T_f0 and adds an emitter, but its band saturates.
- He lowers T_f0 through Le_O; the helium limit is a heat-loss limit, not a radiative one.
- Optically thin emission gives D_crit ∝ p^(−1/2); band saturation weakens that.

### 1.6 The unlit droplet

The liquid cools by its own evaporation while the gas relaxes to ambient.

```
mdot = 4 pi r rho D_F ln(1 + B_M),  B_M = Y_Fs/(1 - Y_Fs),  x_Fs = x_m p_sat(T_l)/p
m c_l dT_l/dt = 4 pi r lambda (T_g - T_l) ln(1+B_M)/B_M - mdot L
T_g = T_inf + (T_f - T_inf) theta(t)      (centre of a hot sphere of radius r_f, conduction)
D from m_F/rho_l(T_l) + m_W/rho_w        (thermal contraction included)
```

- At 1 atm in N2 the evaporation rate is about 0.3 / 0.10 / 0.064 / 0.035 / 0.011 mm²/s at T_l = 335 / 310 / 300 / 290 / 280 K (hand estimate).
- dT_l/dt = −1.5 K_ev (L/c_l)/D², so the decay time goes as D².
- Stored heat plus contraction is worth up to 12 % of D_e².

### 1.7 Property tables

Gas conductivity (W/m/K) and molar heat capacity (J/mol/K), NIST:

| Species | λ at 300 / 900 / 1500 K | c_p at 300 / 900 / 1500 K |
|---|---|---|
| N2 | 0.0260 / 0.0605 / 0.0880 | 29.1 / 32.1 / 34.8 |
| O2 | 0.0265 / 0.0659 / 0.0982 | 29.4 / 34.4 / 36.5 |
| CO2 | 0.0168 / 0.0638 / 0.1021 | 37.2 / 53.0 / 58.4 |
| He | 0.156 / 0.335 / 0.479 | 20.8 |
| H2O | 0.0186 / 0.0826 / 0.165 (extrapolated) | 33.6 / 39.9 / 47.1 |
| CH3OH | 0.015 / 0.10 / 0.20 (extrapolated above 620 K, ±25 %) | 44.1 / 84.9 / 105.4 |

Mixture conductivity: half the sum of the mole-weighted mean and the mole-weighted harmonic mean.

Binary diffusion at 300 K and 1 atm, cm²/s (exponent of T), Chapman–Enskog; water pairs raised 8 % toward the measured value:

| Pair | D (n) | Pair | D (n) | Pair | D (n) |
|---|---|---|---|---|---|
| O2–N2 | 0.208 (1.70) | H2O–N2 | 0.25 (1.79) | CH3OH–N2 | 0.153 (1.78) |
| O2–CO2 | 0.156 (1.74) | H2O–CO2 | 0.18 (1.87) | CH3OH–CO2 | 0.107 (1.86) |
| O2–He | 0.745 (1.66) | H2O–He | 0.89 (1.68) | CH3OH–He | 0.61 (1.68) |

- **Liquid methanol:** ρ = 784.5 kg/m³ at 300 K, 746 at 340 K; c_l = 2.55–2.83 kJ/kg/K; k_l = 0.196 W/m/K; α_l = 9.4e-8 m²/s. Antoine: log10 p[bar] = 5.20409 − 1581.341/(T − 33.50).
- **Water in methanol:** diffusivity about 2.2e-9 m²/s (inferred from a published liquid Lewis number of 45, not measured). Water latent heat 2.36 MJ/kg.
- **Methanol–water equilibrium:** DECHEMA table at 1 atm, typed in `physics.py`.
- **Planck-mean absorption** (TNF/RADCAL fits, m⁻¹ atm⁻¹): a = Σ c_k (1000/T)^k.
  - H2O: −0.23093, −1.12390, 9.41530, −2.99880, 0.51382, −1.8684e-5
  - CO2: 18.741, −121.310, 273.500, −194.050, 56.310, −5.8169
  - Values at 1200 / 1500 K: H2O 3.9 / 2.4, CO2 20 / 12.3.

## 2. Confrontation with train (direct evaluation)

### 2.1 Plateau rate (clock 0.25–0.6), no constant adjusted

| Class | n | K measured | Law–Williams, ln(meas/LW) | Two-zone, ln(meas/2z) |
|---|---|---|---|---|
| N2, O2 ≥ 30 % | 5 | 0.658 | 0.717, −0.09 | 1.133, −0.54 |
| N2, O2 16–22 % | 7 | 0.517 | 0.530, −0.02 | 0.864, −0.51 |
| N2, O2 < 16 % | 9 | 0.375 | 0.366, +0.06 | 0.612, −0.45 |
| CO2, O2 22–27 % | 6 | 0.536 | 0.506, 0.00 | 0.872, −0.52 |
| CO2, O2 16–22 % | 9 | 0.455 | 0.430, +0.05 | 0.741, −0.49 |
| CO2, O2 < 16 % | 3 | 0.379 | 0.305, +0.22 | 0.579, −0.42 |
| He, O2 16–22 % | 6 | 0.541 | 0.625, −0.18 | 0.745, −0.28 |
| He, O2 22–27 % | 2 | 0.676 | 0.824, −0.20 | 0.917, −0.30 |

- **N2 and CO2 (39 runs):** Law–Williams is unbiased (median +0.02, sd 0.11). The two-zone model is right in every trend but 1.65 times too high in level (median −0.50, sd 0.09).
- **Helium:** Law–Williams is 22 % too high because it has no Lewis-number flame cooling. The two-zone model gives no helium enhancement at all. The truth is in between.
- **Independent check on helium:** at the published limiting atmospheres the flame temperature is 1590 K (N2, 12 % O2) and 1661 K (CO2), but 1890 K with Le = 1 and 1382 K with the flame-sheet Le_O in helium. The effective exponent on Le_O is about half.
- **Implication:** the absolute level needs one typed constant (c_I ≈ 0.61). The helium flame temperature is the least certain property in the model.

### 2.2 Flame/droplet ratio

- Predicted from the run's rate, ρ_l K/(8 G_o): measured UV ratio at 5 s over prediction has median 0.92 (range 0.72–1.03, 32 runs). At 3 s it is 0.87.
- Oxygen trend, measured at 5 s against predicted quasi-steady: 4.6 against 4.7 (≥ 30 % O2), 5.6 against 5.9 (air), 5.9 against 7.1 (below 16 %).
- With the growth law (D_ff at 600 K, chosen beforehand): ln(simulated/measured) = −0.18 / −0.07 / −0.06 at 1 / 3 / 5 s (sd 0.09 / 0.14 / 0.08). The simulated flame starts too small, because the igniter kernel is not modelled.

### 2.3 Heat-up, K(0.3–1 s)/K(plateau)

| D0 (mm) | n | Measured | Simulated |
|---|---|---|---|
| 1.5–2.2 | 6 | 1.01 | 0.83 |
| 2.2–2.9 | 17 | 0.80 | 0.75 |
| 2.9–3.6 | 15 | 0.66 | 0.71 |
| 3.6–4.4 | 6 | 0.61 | 0.63 |

The small droplets (all from the 2009 calibration sessions) burn faster early than conduction allows; igniter heat is missing from the model.

### 2.4 Water extinction

State of the water model (no extinction imposed) at each measured flame-out:

| Class | n | De/D0 measured | Liquid water (model) | ε_w | T_f (K) |
|---|---|---|---|---|---|
| N2, O2 ≥ 30 % | 6 | 0.26 | 0.95 (0.37–1.0) | +0.71 | 1184 |
| N2, 21 % | 9 | 0.51 | 0.27 (0.17–0.45) | +0.05 | 2102 |
| N2, 18 % | 2 | 0.46 | 0.34 | +0.08 | 1874 |
| N2, 12–15 % | 4 | 0.53 | 0.22 (0.20–0.37) | +0.04 | 1670 |
| CO2, O2 23–25 % | 6 | 0.48 | 0.33 (0.16–0.73) | +0.07 | 2032 |
| CO2, O2 18–20 % | 7 | 0.43 | 0.41 (0.17–0.84) | +0.11 | 1803 |
| He, O2 21–24 % | 3 | 0.39 | 0.48 | +0.12 | 1849 |
| He, O2 17–20 % | 4 | 0.42 | 0.42 | +0.11 | 1661 |

- **Reproduced:** about 30 % water in the liquid at flame-out for every diluent at 25 % O2 and below, and the burn-to-water limit at 30 % O2 and above (simulated De/D0 0.26, measured 0.26).
- **Not reproduced:** extinction itself. The flame-sheet temperature there is still 1650–2100 K. A thermal criterion gives De/D0 = 0.25 everywhere at T_ext = 1200 K and 0.26 at 1500 K, with the liquid at 94–97 % water. This is the classical failure (published theory 0.30 against 0.45–0.52 in Spacelab).
- **Initial water:** 20 % initial water, the published reconciliation, gives 0.44 but no oxygen trend.
- **Rate decline with progress:** K at progress 0.5–0.65 over K at 0.05–0.2 is 0.86 measured and 1.05 simulated. The decline is missing from the model.
- **No precursor:** the run's rate residual does not order De/D0 (Spearman 0.07, 22 runs).
- **Implication:** carry the liquid water fraction as the state and flame-out as a threshold Y_w* ≈ 0.30, rising with T_f0 above about 2350 K. With that typed threshold the simulation gives De/D0 = 0.49 against 0.48 (N2, 18–25 % O2) and 0.48 against 0.53 (below 16 %).

### 2.5 Early flame loss

Radiant ratio Ψ = χ/χ_c at the measured peak flame size, with c_V = 1 and T_ext = 1200 K:

| Group | Lost early, Ψ | Burns on, Ψ | Runs inside the overlap |
|---|---|---|---|
| N2, optically thin | 0.63–0.84 (n = 3) | 0.07–0.68 (n = 23) | 3 |
| CO2 ≤ 1 atm, thin | 0.97–3.39 (n = 8) | 0.23–1.32 (n = 13) | 3 |
| CO2 ≤ 1 atm, CO2 term × 0.44 | 0.46–1.61 | 0.13–0.69 | 3 |
| N2, CO2 term × 0.44 | 0.38–0.50 | 0.04–0.40 | 3 |
| He, thin | 0.43–0.95 (n = 2) | 0.15–0.53 (n = 7) | 2 |

- **N2:** the boundary implies c_V ≈ 1.5 thin, or 2–2.5 with the band factor; both are inside the literature range.
- **CO2:** thin emission is about twice too strong. Even with 0.44 three CO2 runs sit on the wrong side.
- **3 atm, 70 % CO2:** Ψ is 4–5 thin, yet one droplet burned 13.5 s and the other was still burning at 12.3 s when it left the views.
- **Helium:** the 45 % He run at 0.7 atm lost its flame at Ψ = 0.43. Radiation does not explain it.

From inputs only (`crit_theory.py`; Ψ* = 0.45, flame size 0.9 of quasi-steady), D0/D_crit:

| Group | Physics: lost early / burns on | D3: lost early / burns on |
|---|---|---|
| N2 | 1.03–1.21 / up to 1.02 | 1.03–1.10 / up to 0.90 |
| CO2 ≤ 1 atm | 1.03–2.20 / up to 1.31 | 0.96–2.05 / up to 0.75 |
| He | 1.00–3.36 / up to 0.95 | 0.99–1.85 / up to 0.69 |

- Physics D_crit in N2 at 1 atm: 2.52 / 3.05 / 3.28 / 3.94 / 4.52 mm at 12 / 14 / 15 / 18 / 21 % O2. Train brackets: 1.9–2.6, 2.7–3.3, 3.4–4.0, above 3.7, above 3.9.
- In CO2 the physics D_crit is 0–30 % too small. The train implies CO2 emission saturating near 2 m⁻¹ (±50 %) for x_CO2 from 0.15 to 0.57.
- **Timing:** poor. Among runs both measured and simulated radiative (8), ln(t_sim/t_meas) has median −0.61, range −1.6 to +1.0. Near-critical N2 runs never extinguish in the simulation.
- The growth law alone gives a clock falling steeply with D0/D_crit (my estimate −4 to −6 per unit of ln; measured −3.8), but it diverges at D_crit where train shows about 1.
- It also predicts a clock ∝ p. The two near-critical 0.7 atm runs (clocks 0.99 and 1.16) do not show that.

### 2.6 After the flame, D² rate in mm²/s

| Ending | Window after flame-out (s) | Measured | Simulated |
|---|---|---|---|
| Lost early | 0.3–2 | 0.143 | 0.129 |
| | 2–5 | 0.085 | 0.064 |
| | 5–10 | 0.052 | 0.033 |
| | 10–20 | 0.035 | 0.012 |
| Water extinction (with Y_w* = 0.30) | 0.3–2 | 0.057 | 0.049 |
| | 2–5 | 0.029 | 0.015 |
| | 5–10 | 0.010 | 0.006 |
| | 10–20 | 0.006 | 0.005 |

Self-cooling explains the first 5 s. Later rates are 1.5–3 times too low; the warm product cloud and conduction inside the liquid are not modelled.

### 2.7 Forecast of D from the simulated shape, anchored at the measured D(cutoff)

Median absolute error in µm; five artefact runs left out. Persistence is the D² law with the last second's measured rate.

| Cutoff, horizon | Run's rate known | Rate from c_I = 0.61 | Persistence |
|---|---|---|---|
| 1 s, +5 s | 28 | 78 | 119 |
| 1 s, +10 s | 52 | 94 | 294 |
| 3 s, +5 s | 23 | 69 | 36 |
| 3 s, +10 s | 91 | 102 | 258 |
| 5 s, +5 s | 32 | 53 | 42 |
| 5 s, +10 s | 66 | 175 | 272 |

The "rate known" column uses the plateau rate, which a 1 s prefix does not yet hold. It is an upper bound on what the shape is worth, not a forecast score, and it is not comparable with D3's CRPS.

## 3. Audit of the carried forward (`d3-fable/edits.wsp`)

| Typed term | Mechanism | Verdict |
|---|---|---|
| `rate.b0` = −0.53 | level of the inner conductance | empirical; Law–Williams gives 0.530 in air against 0.589 typed and 0.52–0.57 measured |
| `rate.skip[0]` = 0.14, units 0–1 | transfer number and flame temperature | consistent: typed −0.31 at 15 % against theory −0.31; the flattening below 18 % (unit 1) has no mechanism but train supports it |
| `rate.skip[1]` = −0.04 per 20 % CO2 | lower Y_O, higher heat capacity | sign right; theory gives −0.07 to −0.09 |
| `rate.skip[2]` = +0.18 per 20 % He | conductivity against Lewis cooling | between theory's 0 and +0.33; physics cannot do better |
| `rate.skip[3]` = +0.10 ln p | boiling point and latent heat | sign right; theory about +0.05 |
| `rate.skip[5]` = −0.35 w | water co-vaporising | mechanism known, magnitude not derivable (model gives none) |
| rate unit 2 (large droplets slower early) | radiative loss, not flame development | sign right; physics makes it scale with χ, so stronger in CO2 and at low O2, not with D0 alone |
| rate units 3–4 (heat-up) | liquid sink and swelling | clock scaling right (Fo = 0.376 s); physics shape is a 1/√s tail, not a hinge ending at 0.18; slower at low O2 is consistent |
| `crit.b0` = 1.51 (9.05 mm in air) | radiative threshold | **contradicted**: physics 4.5 mm, published 4–4.5 mm critical and 6 mm fully transient; train only says above 3.9 |
| `crit.skip[0]` = 0.75 per 5 % O2 | flame-temperature margin | right between 12 and 15 % (the brackets); physics slope is 0.44 there and 0.27 above 15 %, so the extrapolation is over-steep |
| `crit.skip[1]` = −0.16 per 20 % CO2 | CO2 emission and cooler flame | sign right, size matches train; a linear term cannot saturate |
| `crit.skip[2]` = −0.18 per 20 % He | Lewis cooling | comparable with physics |
| `crit.skip[3]` = +0.7 ln p | none | **sign opposite** to radiation (κ ∝ p). Train evidence is confounded: every near-critical 0.7 atm run has 40–50 % CO2 or 45 % He. At 3 atm it gives 11.4 mm, yet a 3.03 mm train droplet lost its flame at clock 1.47 with De/D0 ≈ 0.64 |
| `class.kappa`, `class.bias` | bistable near threshold | empirical, physically reasonable |
| `diff.b0` = 1.35 (w = 0.79) | water threshold | equals about 30 % liquid water in the model: consistent |
| `diff.skip[0]`, diff unit 0 | hotter flame tolerates more water | consistent with the burn-to-water limit |
| `diff.skip[3]` = −0.6 ln p | none | typed from the 3 atm CO2 run, whose early flame-out is radiative; at 0.7 atm it moves De/D0 from 0.455 to 0.425 with no support (published theory: within ±7 %, larger De at lower p for pure fuel) |
| diff unit 1 (earlier near D_crit) | radiation weakens the flame | plausible; the nearest train run (189R001) went the other way |
| `rad.W1/w2` (−3.8 from 0.05) | flame growth toward quasi-steady radius | sign and order right |
| `rad.skip[2]` = −0.22 per 20 % He | faster oxygen diffusion | agrees: physics −0.17 |
| rad: no CO2 or pressure term | slower diffusion in CO2, clock ∝ p | physics expects both; train does not show the pressure one |
| `post.k_hot`, `hot_exp`, `tau0_s`, `tau_exp` | self-cooling of the hot liquid, contraction | window means match; the exponents (0.45, 0.2) fit window means, where the mechanism gives a size-independent start near 0.3 mm²/s and a decay time ∝ D_e² |
| `post.k_inf`, `size_exp` = 0.72 | steady evaporation | steady rate cannot depend on size (about 0.011 mm²/s pure fuel); the exponent patches the cooling tail |
| `enc.gain_w` (span only) | heat-up state | physics says clock: 3 s is s = 0.75 for 2 mm and 0.19 for 4 mm |
| `filt.sd_evap`, half-decayed mean | unlit rate | right after flame-out the unlit rate is 0.25–0.5 of burning; physics gives it as a function of liquid temperature |
| `unc.*`, `haz.log_disruption` | session spread | keep |

## 4. Blueprint for an extended forward

**State per scenario node:**
- liquid fuel mass and liquid water mass;
- liquid mean temperature (closed form in Fo while lit, one ODE when unlit);
- flame radius (algebraic);
- radiant fraction;
- lit flag and time of flame-out.

**Per step while lit (explicit Euler, 0.02–0.05 s):**
1. Y_w, bubble temperature and vapour ratio from the equilibrium table.
2. Fo, sink, swelling.
3. ε_w from the closed form, using the previous step's B_f.
4. T_f from ε_w and the previous χ.
5. Rate from g_i ln(1 + B_f) and G_o.
6. Flame radius from the erfc relation (bisection, or one fixed-point pass).
7. χ.
8. Flame-out tests.

**When unlit:** the two ODEs of section 1.6. The only stiff term is liquid cooling (up to 25 K/s), which 0.02 s handles.

**Precomputed per atmosphere, so no integrals at request time:** T_f0 and its slopes in ε_w and χ, g_i, mean c_p of the inner gas, G_o, Le_F, Le_w, φ_w, D_ff, κ coefficients, T_s, L.

**Parameters:**

| Group | Parameter | Value | Uncertainty | Kind |
|---|---|---|---|---|
| Rate | c_I, inner conductance scale | 0.61 (N2, CO2) | ±0.10 run, ±0.05 session | varies by session |
| | helium exponent ζ on Le_O | 0.5 | 0.25–1 | constant, poorly known |
| | film recipe (0.4 fuel, 0.6 outer) | fixed | absorbed in c_I | constant |
| | water decline, −0.35 w | keep typed | ±0.1 | empirical |
| Heat-up | α_l | 9.4e-8 m²/s | ±10 % | constant |
| | c_l(T_s − T0)/L | 0.09 | ±0.01 | constant |
| | expansion, −0.96 kg/m³/K | fixed | ±5 % | constant |
| | ignition delay | 0.25 s | 0.15–0.4 | varies |
| | igniter heat (not modelled) | 0 | adds rate in the first 0.5 s | varies |
| Flame | D_ff reference temperature | 600 K | 450–900 K | constant |
| | measured/quasi-steady ratio | follows the law | ±10 % | — |
| Water | Le_w | 0.65 (N2, CO2), 0.8–0.95 (He) | ±0.1 | constant |
| | D_O/D_w outside | 0.83 | ±0.05 | constant |
| | Y_w*, liquid water at flame-out | 0.30 | sd 0.08 | varies by session |
| | rise of Y_w* with T_f0 | from 2350 K to burn-out at 2750 K | ±150 K | empirical, four sessions |
| | initial water in the fuel | 0 | 0–0.1, unknown | varies |
| Radiation | T_ext | 1200 K | 1100–1400 | constant |
| | c_V | 2.2 with band factor | 1.5–3 | constant |
| | CO2 saturation level | about 2 m⁻¹ | ×1.5 | empirical, four sessions |
| | pressure exponent of D_crit | −0.25 | −0.5 to 0 | unknown |
| | flame-size factor | 0.9 | 0.8–1.0 | constant |
| | clock law at and above D_crit | keep D3's | sd 0.14, more off-train | empirical |
| | helium term in the clock | −0.17 to −0.22 per 20 % | ±0.05 | constant |
| | helium limit margin | T_f0 − T_ext small | dominant for 14–15 % O2 | constant, poorly known |
| Unlit | gas relaxation radius | r_f at flame-out | product cloud ×2–5 larger | constant |
| | liquid mixing | lumped | conduction slows the tail | constant |
| | water activity | Raoult | ±20 % | constant |
| Measurement | band, rate spreads | D3's (20 µm, 0.10, 0.06) | do not narrow | — |

**What a prefix identifies:**
- **Level** at the cutoff: directly.
- **Rate level c_I:** from the end rate, read against the physically predicted transient at that clock. A 1 s prefix sits in heat-up (s = 0.06–0.4), where swelling and sink are known functions of Fo, so the innovation should be taken against them. The gain should rise with clock, not with seconds. On train the rate offset at 1 s has no persistence, so do not claim more than D3's gain there without testing.
- **Lit or not:** from the end rate against the unlit rate at the modelled liquid temperature.
- **Not identifiable:** Y_w*, the side taken near D_crit, the CO2 saturation, initial water.

**Carrying uncertainty:** scenario nodes over c_I, Y_w* and a log factor on D_crit, with mixture quantiles as now. Property data are constants. The helium exponent, CO2 saturation and pressure exponent are constants that are poorly known: carry them as wide fixed priors, not as session variables.

**Keep from D3:** the validity gate; the level carried along the end rate; the state filter; the scenario-node mixture and event rows; the plateau rate law as prior for c_I (median residual +0.04 on N2 and CO2, sd 0.07, in my re-evaluation); the class softness; the clock law above D_crit; all spreads.

**Change:**
- D_crit as the physical scaling normalised on the train brackets, with saturating CO2 and no positive pressure term.
- Post-extinction as the cooling ODE.
- Heat-up as Fo with swelling.
- The diffusive law as a water threshold with no pressure term.

## 5. Unseen validation classes, stated before any outcome

Conditions only. D_crit below combines the physics scaling with the train anchors.

| Class (runs) | Expected regime | Relative to D3 and nearest train | Confidence |
|---|---|---|---|
| CO2 0.25, O2 0.16, 1 atm (2.71, 2.77, 3.70, 4.24 mm) | D_crit 3.1 ± 0.4 mm (D3: 3.55). The two large ones lose the flame early, near 8 s and 7 s; the two small ones are sub-critical by only 10–20 % | large ones 4–5 s earlier than D3; small ones P(early loss) 0.3–0.4 against 0.12–0.14 | moderate |
| He 0.35, O2 0.14, 1 atm (1.88, 2.44 mm); He 0.30, O2 0.15 (2.52 mm) | at or past the published helium limiting oxygen; T_f0 1355–1446 K; no sustained flame at 14 %, marginal at 15 % | P(early loss) 0.8–0.9 (2.44), 0.6–0.7 (1.88, 2.52) against D3's 0.76, 0.19, 0.39; flame gone in 1–4 s | moderate, leans on the published limit |
| He 0.40, O2 0.18, 0.7 atm (2.70, 3.50 mm) | D_crit about 2.8 mm; nearest train (17 % O2, 45 % He, 2.79 mm) lost its flame at 4.1 s | 3.50: early loss near 5–7 s (as D3); 2.70: P about 0.45 against 0.29 | low to moderate |
| He 0.35, O2 0.20, 0.7 atm (2.95, 3.87 mm) | 2.95 burns to water extinction; 3.87 is at the boundary. Rate 0.63 ± 0.05 | close to D3 | moderate |
| N2, O2 0.20, 72 kPa (2.40 mm) | far sub-critical; rate 0.48–0.50; De/D0 0.47 ± 0.06, flame-out near 9.4 s | D3 puts flame-out 0.3–0.5 s later (De/D0 0.42) through its pressure term | high for regime, moderate for the shift |
| CO2 0.45, O2 0.17, 0.7 atm (2.80, 3.02, 3.63 mm) | between two train sessions; D_crit about 2.7 mm; all three near or above | as D3 | moderate |
| CO2 0.35, O2 0.20, 0.7 atm (4.20 mm) | boundary: D_crit 3.8–4.3 mm | P(early loss) 0.5–0.6 against 0.42; if lost, near clock 1 (about 17 s) | low |
| CO2 0.05, O2 0.20, 1 atm (3.33, 3.53, 4.35 mm) | 4.35 mm is near the air critical size (4.5 by physics, above 3.9 by train) | P(early loss) about 0.35 for 4.35 mm against D3's 0.01; smaller ones burn on | low to moderate |
| CO2 0.70, 2.7 atm (1.66, 3.70, 4.42 mm) | radiation-assisted and size-dependent: 1.66 burns to about 0.47; 3.70 and 4.42 lose the flame near clock 1 and 0.5 | D3 gives all three De/D0 0.57 and flame-out at 20–31 s for the large ones; expect 10–17 s. P(lost before 15 s) 0.6 (4.42), 0.4 (3.70) | low: two train runs, one censored |
| Small droplets (1.66, 1.76, 1.88 mm) | on plateau by 0.6 s; flame-out inside every horizon (1.76 mm: about 5.5 ± 0.7 s); fast unlit decay | heat-up correction near zero | moderate |

## 6. What cannot be predicted from these inputs

- Which side a droplet within about 15 % of D_crit takes. The measured flame size separates N2 runs, but the diameter does not.
- The water threshold of a session (progress sd 0.055, about ±0.06 in De/D0, ±0.6 s), since no prefix quantity orders it.
- Igniter energy, initial water or contamination in the fuel, disruption, drift out of view, tracker artefacts, a mis-filed atmosphere.

**The one extra signal that would change this:** the UV flame diameter over the prefix. It gives the flame radius and its growth, hence the radiant fraction directly. On train Ψ from the peak flame size separates early loss from burn-on in N2 with one near-tie. Droplet position would add the drift-out risk.

## 7. Review aid: claims to try hardest to break

All commands run from `D:\dev\SpaceApps-Ardealu`.

| Claim | Re-check |
|---|---|
| D_crit in air is 4–5 mm, not 9 | `python runs/dev/research6/flex_methanol/crit_theory.py`; depends on Ψ* = 0.45 typed after seeing the N2 table, on T_ext = 1200 K, and on the literature figures (repo dossier and helper) |
| D3's pressure terms (`crit.skip[3]`, `diff.skip[3]`) have unsupported sign | check which 0.7 atm and 3 atm train runs constrain them: `out_crit_theory.txt`, test sheets FLEX-010 and FLEX-011 |
| About 30 % liquid water at flame-out, all diluents | `python runs/dev/research6/flex_methanol/eval_water.py`; sensitive to Le_w (0.65) and the well-mixed assumption |
| Thermal extinction criterion fails (De/D0 0.25) | `python runs/dev/research6/flex_methanol/sim.py base` and `sim.py t_ext=1500` |
| Flame ratio follows ρ_l K/(8 G_o) at 0.92 | `python runs/dev/research6/flex_methanol/eval_qs.py`; check the harmonic-mean ρD_O in `physics.py` |
| Early deficit is mostly swelling | not cleanly isolated: my sink-off switch did not take effect; redo with `KL` changed inside `physics.py` |
| Helium Lewis exponent about half | rests on the published limiting atmospheres and my Le_O (outer gas half products) |
| CO2 emission saturates near 2 m⁻¹ | `eval_rad.py 1.0 1.0` against `eval_rad.py 1.0 0.44`; four sessions, hand arithmetic |
| Unlit law is self-cooling | `sim.py c_v=2,f_co2=0.44,yw_ext=0.3`; late windows under-predicted |
| Train-only reading | `measure_train.py` asserts every run id against `lib.partition_ids("flex","train")` |

## Sources

**Read by me (repo only; I ran no web search myself):**
- Full: `docs/wsp/EXPERIMENTAL_SCIENCE.md`, `docs/wsp/research/lit_droplets.md`, `docs/wsp/cards/flex_droplet3.md`, `docs/wsp/cards/flex_droplet4.md`, `docs/wsp/feedback/flex_methanol_d3_train_anatomy.md`, `d3-fable/edits.wsp`, `summary_tables.md`.
- Parts: `exp_flex.md` sections D–E, the `droplet3.rs` module header, the methanol rows of `flame_end_train.csv`, the flex section of `shift/tables.txt`.
- The repo dossiers contain aggregate FLEX statements that include validation and test runs (limiting oxygen values, a standoff ratio near 5, rate ranges). I used the published limiting atmospheres. I transcribed no per-test value.

**Helper A (water, heat-up; ran on opus/medium, launched before the coordinator's note):**
- Full: Williams NASA CP 1995 (NTRS 19960008402); UCSD Williams-group pages with figures.
- Sections: Marchese, Lee, Held & Dryer (NTRS 19960008395); FSDC-2 (NTRS 19990019814); Law 1982 review; the 2005 FLEX requirements document.
- Garbled OCR: Marchese, Dryer & Choi 1994.
- Theory only, results skipped: Nayagam et al. 2013 (NTRS 20130014061); Hicks 2012 slides.
- Abstracts only: Zhang, Card & Williams 1996; Zhang & Williams 1998; Lee & Law 1992; Hicks 2010; Nayagam 2010 and 2013; Law, Chung & Srinivasan 1980; Law & Law 1976; Crespo & Liñán 1975.
- Taken: ε_w ≈ −0.03 and De/D0 0.28–0.32 from theory; the liquid Péclet table; liquid Lewis number 45; Spacelab De/D0 0.45–0.52; the 20 % initial-water reconciliation; heat-up about 10 % of lifetime.

**Helper B (radiation, properties; opus/medium):**
- Full: TNF radiation page; NIST WebBook; Santa et al. 2007; Waddell et al. 2026; the DECHEMA table via Duncan & Reimer.
- Sections or scans: FSDC-2; DCE (NTRS 19990019805); Easton decane; Ackerman TM.
- Abstracts only: Marchese & Dryer 1997; Farouk & Dryer 2012 (theory sentences only); Nayagam 2018 and 2019; Chao, Law & T'ien.
- Taken: Planck fits, property tables, the 6 mm fully-transient size, extinction near 1100–1200 K, the 0.44 band factor, radiant fractions.
- It did not open the FLEX methanol records.

**Both helpers** saw a search-result abstract of Dietrich et al. 2014 and report using nothing from it.

**Re-checked by me:**
- heat capacities, against values I recalled independently;
- the Planck coefficients, which I had written down before the helper returned;
- the boiling point from Antoine (337.7 K);
- the heating value arithmetic;
- the diffusion values, against textbook ones.

## Files written

All in `D:\dev\SpaceApps-Ardealu\runs\dev\research6\flex_methanol\`:
- Scripts: `physics.py`, `measure_train.py`, `eval_qs.py`, `eval_rad.py`, `eval_water.py`, `crit_theory.py`, `d3_laws.py`, `sim.py`
- Tables: `train_measure.csv`, `qs_by_run.csv`, `sim_*.csv`
- Outputs: `out_eval_qs.txt`, `out_eval_rad_thin.txt`, `out_eval_rad_co2_044.txt`, `out_eval_water.txt`, `out_crit_theory.txt`, `out_d3_laws.txt`, `out_sim_*.txt`

One stale file remains: `sim_c_v2_f_co20p44_yw_ext0p3_inner1.csv`, from a variant I dropped because it erased the heat-up deficit.

No git-tracked file was modified by me. `tools/wsp/reassemble.py` shows as modified at 13:38 today; I did not touch it.

## Partition statement

- **Validation:** I read conditions only (x_O2, x_CO2, x_He, pressure, sheet initial diameter, test-point name), through `lib.experiments` and the shift tables. No series, test sheet, flame trace or outcome.
- **Test:** no test run was read. Nothing under `runs/dev/data_ext/`, `holdout`, or `tools/wsp/dev_eval.py` was opened. The PSI-69 summary table was not opened.
- **Published per-test FLEX methanol results:** none read, by me or, by their reports, by the helpers.
- **Method:** no fit, regression or search. Typed values compared were c_V 1 and 2, T_ext 1200 and 1500 K, CO2 factor 1 and 0.44, initial water 0 and 0.2, helium exponent 1 and 0.5.

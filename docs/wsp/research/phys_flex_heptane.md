# Physics of an n-heptane droplet burning in microgravity, for a mechanistic droplet model

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. Outcomes of train runs only; of validation runs the conditions only.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# FLEX n-heptane: physics dossier for a mechanistic forward

Three things change the picture most:

- **Radiative loss of the hot flame.** A physical critical diameter (flame temperature with the oxygen Lewis number, band-saturated gas emissivity) reproduces the typed `crit` law within about 10% in N2 and in CO2 up to 30%. It leaves three 0.7 atm runs on the wrong side and does not rescue the 2 atm CO2 run.
- **Cool-flame end diameter.** It scales with D0, not with an absolute size, and its oxygen dependence follows the published form in O2 partial pressure with a limit near 0.10–0.12 atm.
- **Unlit droplet.** A droplet left hot by a flame evaporates on its own stored heat. That closed form explains the post-cool-flame transient with no typed amplitude, and half or more of the helium after-burn.

Nothing was fitted. Three constants were read from train and are named where used: D_crit,air = 3.7 mm, a model-scale extinction temperature of 1650 K, and the cool-flame normalisation at run 301F001. Property values marked [mem] are handbook numbers recalled from memory and not re-downloaded; re-check them before freezing.

## 1. The physics, mechanism by mechanism

### 1.1 Property tables

| Quantity | Value | Source |
|---|---|---|
| Fuel: M, stoichiometric O2/fuel mass ratio σ | 100.2 g/mol, 3.513 | stoichiometry |
| Heat of combustion from liquid | 44.56 MJ/kg | [mem] NIST |
| Latent heat L | 317 kJ/kg at boiling, about 365 at 298 K | [mem] |
| Liquid c_l, ρ_l, k_l, α_l | 2.24 kJ/kg/K, 680 (300 K) / 614 (boiling) kg/m³, 0.123 W/m/K, 8.1e-8 m²/s | [mem] |
| Liquid expansion β_l | 1.25e-3 /K | from the two densities |
| Boiling point (Antoine, NIST) | 359.9 K at 0.7 atm, 371.6 K at 1 atm, 397.9 K at 2.04 atm | [mem], extrapolated above 372 K |
| Vapour c_p | 1.66 / 2.47 / 3.26 / 3.70 kJ/kg/K at 300 / 500 / 800 / 1000 K | [mem] |
| Vapour conductivity | −0.00172 + 1.6565e-5·T + 1.0525e-7·T² W/m/K | [mem] Yaws |
| O2 diffusivity at 300 K, 1 atm | 0.208 (N2), 0.160 (CO2), 0.74 (He) cm²/s; T^1.7/p; Blanc's law for mixtures | [mem] Marrero–Mason |
| Heptane diffusivity | 0.072 (N2), 0.058 (CO2), 0.26 (He) cm²/s | [mem]; He and CO2 are scaled estimates |
| Sensible enthalpies | JANAF tables for N2, O2, CO2, H2O at 298–3000 K | [mem], in `physics.py` |
| Planck-mean absorption of CO2, H2O | polynomial in 1000/T, 300–2500 K | TNF workshop page, verified |

Derived mixture numbers at 300 K (Wassiljewa–Wilke conductivity; Le_O is thermal diffusivity over O2 diffusivity and does not depend on pressure):

| Atmosphere | λ W/m/K | Le_O | Adiabatic T, Le = 1 (K) | Flame-sheet T with Le_O (K) |
|---|---|---|---|---|
| Air | 0.0261 | 1.05 | 2391 | 2315 |
| 21% O2, 15% He | 0.0351 | 1.27 | 2486 | 2105 |
| 20% O2, 35% He | 0.0502 | 1.54 | 2545 | 1884 |
| 21% O2, 30% CO2 | 0.0226 | 0.90 | 2133 | 2285 |
| 21% O2, 70% CO2 | 0.0189 | 0.75 | 1881 | 2282 |

Temperatures exclude dissociation, so values above about 2300 K are 100–150 K high.

### 1.2 Quasi-steady hot burning

- **Flame temperature.** Heat conducted outward per kg of fuel is c_p(T_f − T∞)/((1 + Y_O/σ)^(1/Le_O) − 1). So each unit of oxygen arrives with Λ ≈ Le_O times its cold ballast. T_f solves: (1 − χ)·q_c = enthalpy of 7 CO2 + 8 H2O + Λ·(diluents) + (Λ − 1)·11 O2, all heated to T_f. χ is the radiated fraction.
- **Burning rate.** K = 8λ_g/(ρ_l c_p)·[ln(1 + c_p(T_f − T_s)/L) + ln(1 + Y_O/σ)/Le_O]. Inner-zone properties follow the Law–Williams recipe as usually quoted (λ_g = 0.4 λ_fuel + 0.6 λ_ambient, c_p of fuel vapour, both at (T_s + T_f)/2); I could not re-verify the recipe from the 1972 paper.
- **What each variable does.**
  - Oxygen acts through T_f and Y_O.
  - Helium raises λ_g by 18–22% at 30–35% but lowers T_f by 300–400 K.
  - CO2 lowers λ_g slightly but raises T_f above adiabatic because Le_O < 1.
  - Pressure enters only through the boiling point.
  - Size enters through radiation (−0.15 per ln D0 is the right sign and size).
- **Level.** The formula gives 0.96 mm²/s in air at χ = 0.3 against 0.54 measured. The absolute level is not predictable to better than a factor 1.5 to 2; type it and use the theory for ratios.

### 1.3 Liquid heat-up and thermal expansion

- The liquid heats with an e-fold time R²/(π²α_l) = 0.31·D0² s/mm²: 1.9 s at 2.5 mm, 3.6 s at 3.4 mm. This is 0.31 clock units, set by liquid conductivity alone, independent of gas, pressure and diluent.
- While it heats, L is effectively up to L + c_l·70 K, which lowers K by about 15–17% at the start.
- The liquid also expands: apparent D² grows by (2/3)·D²·β_l·dT/dt. For 3.4 mm that hides 0.19 mm²/s over 0.3–1 s, 0.10 over 1–2 s, 0.07 over 2–3 s.
- After a flame is lost the droplet cools and contracts, adding 0.03–0.04 mm²/s of apparent shrinking.

### 1.4 Soot and what the tracker sees

From the literature, not measured here:

- Soot collects in a shell at 2–4 droplet radii, where thermophoresis balances the outward vapour flow.
- It grows with flame temperature, pressure and residence time, and fades once radiative cooling drops the flame below about 1350–1600 K (within about 1.5 s for large droplets in air).
- A dense shell is thresholded as droplet: apparent D is high, then D² collapses at several mm²/s. Train shows this at 29–34% O2.
- Expect it at O2 ≥ 0.27 and possibly at 2 atm. Expect it to be negligible below 19% O2 and in He or CO2 dilution.

### 1.5 Flame standoff

- From the oxygen side alone: D_f/D = ρ_l·K / (8·(ρD_O)·ln(1 + Y_O/σ)), with (ρD_O) at (T_f + T∞)/2. It is pressure-independent.
- The far field never becomes steady. Oxygen supply is enhanced by r_f/√(π D_O t), which gives r_f = (δ/2)(−1 + √(1 + 4 r_qs/δ)), δ = √(π D_O t). That is closed form, no ODE.
- Near burn-out the flame lags the droplet, so the ratio rises to 15–19.

### 1.6 Radiative extinction of the hot flame

- **Loss.** χ = ε·σ·T_r⁴·π·D_f² / ((π/4)·ρ_l·D·K·q_c). ε is the gas emissivity over a mean beam length 0.65·D_f.
- **Emissivity.** The CO2 bands are saturated at these sizes, so ε grows about as √(p_CO2·L), not linearly. I used thin-limit Planck mean combined with a strong-line limit anchored on Hottel chart values [mem]. Ambient CO2 adds to p_CO2 everywhere.
- **Criterion.** The flame is lost when its temperature falls to the kinetic limit. Published spherical-flame work puts that at about 1100 K and attributes the loss mainly to products outside the flame (Santa et al. 2007, abstract).
- **Critical diameter.** D_crit = D_crit,air·[(K/K_air)·(χ_c/χ_c,air) / ((FSR/FSR_air)²·(ε/ε_air))]^(1/(1+n)), with χ_c = (T_f0 − T_x)/(T_f0 − T∞) and n ≈ 0.7 the size exponent of ε.
- **CO2** adds emitter, sub-linearly. **Helium** lowers T_f0 through Le_O, which outweighs its higher K. **Oxygen** acts through T_f0. **Pressure** raises emission but also speeds the chemistry (lower T_x); the net is uncertain.
- At high p·x_CO2 the cold CO2 around the flame re-absorbs the band centres, so part of the emission returns. I did not quantify this.

### 1.7 The cool flame

From Williams and Nayagam 2019 (theory sections):

- **Structure.** Partial burning at T_f = 600–800 K, where the temperature slopes on both sides of the flame are equal in mixture-fraction space.
- **Rate.** K_c = 8(λ/c_p)/ρ_l·ln[(2T_f − T_0 − T∞)/T_L], with T_L = L/c_p and T_0 = T_s − T_L. It does not depend on oxygen or pressure. Helium raises it through λ. Standoff is about 3.
- **End diameter.** The flame ends where the inverse residence time reaches its chemical maximum: d_ce² ∝ (ρ_l/ρ∞)²·(K_c²/D∞)/(FSR⁴·χ_max), with χ_max ∝ oxygen concentration at the flame.
  - Scaling: d_ce ∝ ([O2] − β)^(−1/2), β proportional to pressure, diverging at a cool-flame oxygen limit ("of order 0.10" mole fraction in N2 at 1 atm).
  - At fixed oxygen mass fraction, d_ce falls with pressure.
- **Why it ends as it does.** It is an instability at the low end of the NTC range, not a heat-loss extinction. The maximum is flat, so the end point scatters.
- **Oxygen limit generalised (my derivation).** Oxygen left at the flame is 1 − 2·Le_O·c_p,out·(T_f − T∞)·W_O/(q_O·Y_O), with q_O = 336 kJ per mol O2 (heptane to CO + H2O). Helium raises both Le_O and c_p per mass, so it moves the limit up.
- **High pressure.** The NTC window moves up in temperature; a "warm" flame near 900–1000 K burns close to the hot rate (published numerics, quoted in `lit_droplets.md`).

### 1.8 After a flame is lost, including helium

- **Self-cooling evaporation.** A droplet at surface temperature T_s evaporates at K_ev = 8(ρD_F)/ρ_l·ln(1 + B_M), with B_M from the vapour pressure. At 1 atm in air this is 0.40 mm²/s at 369 K, 0.17 at 352 K, 0.08 at 330 K, 0.022 at 300 K and 0.0097 at the cold wet bulb (285 K).
- **Closed form.** The latent heat comes from the liquid, and K_ev falls by e per 26 K of cooling. So K(t) = K0/(1 + t/t0), with t0 = 26 K·c_l·D²/(1.5·K0·L).
  - After a hot flame, T_s0 is the boiling point minus about 3 K (K0 ≈ 0.40).
  - After a cool flame, T_s0 is the wet bulb under 700 K gas (351–353 K at 1 atm, K0 ≈ 0.17).
- **Warm gas.** The gas left behind stays warm for tens of seconds (my estimate: 100–300 K above ambient over a 4 cm ball), which holds a floor of a few hundredths mm²/s before the cold value.
- **Helium.** ρD_F is about 1.9 times higher (cold evaporation 0.018 against 0.010). In 30–35% He at 0.7 atm the oxygen partial pressure is 0.14 atm, the same as 14% O2 at 1 atm. The cool flame is born at or below its end diameter and decays instead of holding.

## 2. Confrontation with train (direct evaluation)

Scripts and full tables are in `runs/dev/research6/flex_heptane/` (`tables.txt` holds every output).

### 2.1 Hot burning rate relative to air (measured air reference 0.544 mm²/s)

| Class | Theory with Le_O | Theory Le = 1 | Measured |
|---|---|---|---|
| N2 18%, small / large droplet | 0.88 | 0.87 | 0.94 / 0.77 |
| N2 16%, small / large | 0.80 | 0.79 | 0.77 / 0.62 |
| N2 15% | 0.75 | 0.75 | 0.71 |
| N2 20%, 0.7 atm | 0.95 | 0.95 | 0.88 |
| CO2 20–25%, 23–24% O2, 0.7 atm | 1.08–1.12 | 0.97–1.03 | 0.96–1.03 |
| CO2 5–10%, 1 atm | 0.92–0.96 | 0.87–0.94 | 0.86–0.95 |
| CO2 70%, 2 atm | 1.03 | 0.76 | 1.14 |
| He 30–35%, 0.7 atm | 1.08–1.09 | 1.43–1.46 | 1.15–1.32 |
| He 10–15%, 1 atm | 0.92–1.01 | 1.03–1.19 | 1.06–1.22 |

- The oxygen dependence is right for small droplets. Large droplets at low oxygen burn 12–22% slower than theory (radiative weakening near the limit).
- The Le_O correction is needed at 70% CO2 and overstated for helium, where measured rates sit about two thirds of the way from the Le_O value to the Le = 1 value.
- **Heat-up and expansion.** Predicted apparent deficit for 3.4 mm over 0.3–1 s is 0.19 mm²/s; measured 0.13–0.30 (three runs). For 2.5 mm, predicted 0.13 and measured 0–0.08. Something compensates early for small droplets; igniter heat and the thin early flame are my candidates, not tested.

### 2.2 Flame/droplet ratio at 3 s

Oxygen-side law with the measured K, over the UV camera ratio:

| Class | n | Law / measured, median (range) |
|---|---|---|
| N2 1 atm | 11 | 0.84 (0.79–1.06) |
| N2 0.7 atm | 1 | 0.86 |
| CO2 0.7 atm | 7 | 0.83 (0.67–0.92) |
| CO2 1 atm | 7 | 0.83 (0.52–0.90) |
| CO2 2 atm | 1 | 1.12 |
| He 0.7 atm | 4 | 0.84 (0.80–0.89) |
| He 1 atm | 1 (H02H101) | 0.89 |

A uniform 16% low with no constant tuned. The steady value (10–13) matches the ratio before radiative loss (7.5–12).

### 2.3 When and at what size the hot flame is lost

- **Loss fraction at the moment of loss** (measured flame and droplet diameters): 0.14–0.45. Model flame temperature there: median about 1640 K, range 1430–1880 K. Burn-downs at their worst moment stay at 1730–2000 K.
- That is about 500 K above the 1100 K kinetic limit. My gas-only loss is too small by roughly a factor 2. Missing: soot, fuel-vapour bands, products outside the OH* edge.
- **Physical D_crit against the typed law and the outcome:**

| Atmosphere | Physical D_crit (mm) | Carried D_crit (mm) | Train outcome |
|---|---|---|---|
| N2 18 / 16 / 15 / 14%, 1 atm | 2.80 / 2.10 / 1.69 / 1.23 | 2.55 / 1.99 / 1.76 / 1.55 | all on the right side in both |
| CO2 5 / 10 / 15 / 20% (20–17% O2), 1 atm | 2.79 / 2.22 / 1.75 / 1.39 | 2.92 / 2.31 / 1.82 / 1.44 | right side in both |
| CO2 15–30%, 0.7 atm | 3.75 / 3.36 / 3.00 / 2.41 | 4.33 / 3.42 / 2.70 / 1.88 | right side in both |
| N2 20%, 0.7 atm | 4.17 | 3.27 | 3.56 mm went out radiatively: physics wrong |
| He 30% / 35%, 0.7 atm | 4.15 / 3.49 | 3.28 / 2.84 | 3.23 and 3.08 mm went out: physics wrong on both |
| CO2 70%, 2 atm | 0.84 | 0.77 | 2.74 mm held its flame 8.0 s: both wrong |

  - The three physics misses are all at 0.7 atm and all come from my pressure factor in the emissivity (D_crit up 23%).
  - As a hand estimate (dividing the helium values by that factor, not re-run in a script), dropping the pressure effect gives about 3.4 and 2.84 mm in helium; the observed boundary is 2.86–3.23 mm and below 3.08 mm.
  - So train says D_crit depends on pressure with an exponent between 0 and −0.3, weaker than emission alone gives. Slower chemistry at low pressure is my explanation, not tested.
- **Clock at loss** (t/D0² in s/mm²) against D0/D_crit (carried):

| D0/D_crit | Clock |
|---|---|
| 1.0–1.1 | 0.6–1.1 |
| 1.2–1.35 | 0.41–0.80 |
| 1.45–1.7 | 0.31–0.53 |
| 2.0–2.1 | 0.20–0.25 |
| 2.6–2.7 | 0.09–0.12 |

  - Helium runs sit at the low end of each band (0.72, 0.57, 0.32), not outside it.
  - Rescaling time by the gas diffusivity does not tighten this; it moves the helium runs the wrong way.

### 2.4 Which runs get a cool flame, its rate and end diameter

- **Rate.** Theory at 700 K gives 0.39 mm²/s in air (0.46 at 800 K). With the paper's own ρD it gives 0.81. Measured is 0.31–0.36 across 15–24% O2, 0–25% CO2 and 0.7–1 atm, with no oxygen or pressure trend, as theory says.
  - Helium: theory 1.21 times at 15% and 1.46–1.55 times at 30–35%. Measured 0.48 (H19H101) and 0.45–0.56 in the first second (30–35%).
  - 2 atm, 70% CO2: measured 0.51, theory at 700 K 0.36. It matches the formula at about 900 K (0.44 at 800 K, 0.56 at 950 K), the warm flame.
- **End diameter scales with D0.** Three same-atmosphere pairs give exponents 1.07, 1.23 and 0.90. Quasi-steady theory says zero. My reading: the cool flame burns inside the oxygen-depleted ball the hot flame left, whose size scales with D0. This is not tested.
- **D_ce/D0 against oxygen partial pressure**, with the published form normalised at 301F001:

| pO2 (atm) | Runs | Measured D_ce/D0 | Form, β = 0.10 | Form, β = 0.12 | Carried |
|---|---|---|---|---|---|
| 0.140 | N04H101 | 0.95 (barely held) | 0.67 | 0.82 | 0.56 |
| 0.151 | N03H101 | 0.66 | 0.59 | 0.66 | 0.58 |
| 0.160–0.161 | N02H101, N02H203, C09H501 (0.7 atm) | 0.57–0.59 | 0.54–0.55 | 0.57–0.58 | 0.53–0.57 |
| 0.168 | C04H101, C04H201 | 0.57–0.59 | 0.51 | 0.53 | 0.52–0.55 |
| 0.180 | 301F001, C03H601 | 0.47, 0.51 | 0.47 | 0.47 | 0.48, 0.53 |
| 0.190 | C02H101, C02H201 | 0.42, 0.39 | 0.45 | 0.44 | 0.47, 0.50 |
| 0.211, 15% He | H19H101 | 0.42 | 0.40 | 0.38 | 0.47 |

  - The measured oxygen dependence is steeper than β = 0.10 gives. Solving the 18% and 16% points for β gives 0.119; that is algebra on two train points, for the author to type or not.
  - The 0.7 atm run falls on the same curve in pO2.
- **Plateau definition.** The first time, at least 1.5 s after loss, that the 1 s rate stays below 0.15 mm²/s for 2 s. It reads slightly high where the roll-off is slow.

### 2.5 The helium after-burn and evaporation

Rates in mm²/s, measured / self-cooling law:

| Case | Window 1 | Window 2 | Window 3 | Later |
|---|---|---|---|---|
| After the cool flame, 10 runs, 0.97–2.15 mm | +0–2 s: 0.09–0.12 / 0.08–0.13 | +2–5 s: 0.02–0.07 / 0.03–0.08 | +5–10 s: 0.045–0.055 / 0.014–0.045 | — |
| H12H101 (He, 2.33 mm at loss) | 0–1 s: 0.45 / 0.31 | 1–2 s: 0.40 / 0.20 | 3–5 s: 0.21 / 0.11 | 8–12 s: 0.08 / 0.05 |
| H13H201 (He, 2.43 mm) | 0.55 / 0.31 | 0.43 / 0.21 | 0.20 / 0.12 | 0.09 / 0.06 |
| H13H101 (He, 3.31 mm) | 0.56 / 0.35 | 0.50 / 0.27 | 0.24 / 0.18 | 0.14 / 0.10 |
| N04H101 (14% O2, 4.15 mm) | 0.10 / 0.36 | 0.23 / 0.30 | 0.19 / 0.22 | 0.12 / 0.13 |

- After the cool flame the amplitude is right with no typed number. The law decays too fast for droplets below 1.3 mm; the warm gas floor is the likely missing term.
- In helium the law gives 55–70% of the first two seconds and nearly all of it from 5 s on. The rest, 0.15–0.25 mm²/s decaying with an e-fold of about 3 s, is a dying flame (one session).
- N04H101 matches from 1 s on. But a 4 mm droplet cannot have stored that heat in 1.5 s of burning (my estimate: about a third of what is needed), so a weak gas-phase source is still required. Both readings give the same D(t).

## 3. Audit of the carried forward

| Tensor and number | Mechanism | Verdict |
|---|---|---|
| `rate.b0` −0.48; `rate.skip[4]` −0.15 | level; size through radiation | typed level is necessary; sign and size right |
| `rate.skip[0]` 0.13, units 0–1 (−0.12, −0.05) | oxygen through T_f and Y_O | agrees with theory (0.80 at 16% against typed 0.77) |
| `rate.skip[1]` −0.08 per 20% CO2 | conductivity, Le_O | fine to 30%; wrong sign at 70% (carried 0.46 mm²/s, measured 0.62) |
| `rate.skip[2]` 0.15, unit 2 | helium conductivity | right; lies between the two theories as the data do |
| `rate.skip[3]` 0.05 | boiling-point shift | right (about zero) |
| `rate` units 5 and 4 (−0.3 over 0.6 clock units, +0.11) | liquid heat-up plus expansion | physical: e-fold 0.31 clock units, start deficit about −0.19; correct to keep on the D0² clock |
| `crit.b0`, `crit.skip` [0.62, −0.45, −0.08, 0.0] | radiative critical diameter | oxygen and CO2 slopes are reproduced by physics up to 30% CO2; helium slope plus zero pressure fits train; log-linear CO2 has no saturation and oxygen has no flammability limit |
| `rad.*` (clock, −2 per unit excess, −1 per ln D0) | time to loss against margin | empirical; holds on 17 N2/CO2 runs; no pressure or chemistry in it |
| `cool.p` (−7.0 on helium above 15%) | cool-flame onset | patch: forces "no cool flame" at 30–35% He, where the model's own K_c (0.49–0.51) and D_ce (2.6–3.2 mm) already describe a sub-critical cool flame |
| `cool.k.skip[0]` 0.2 per 5% O2, `[3]` 0.1 | cool-flame rate | physics and train contradict both: no oxygen or pressure dependence. Carried 0.40–0.44 at 0.7 atm against 0.32–0.35 measured |
| `cool.k.skip[2]` 0.14 | helium conductivity | right (0.47 against 0.48 at 15%) |
| `cool.d.skip` [−0.25, 0.05, 0.2, −1.0, 0.8] | end diameter | pressure −1.0 is the theory's value; D0 exponent 0.8 against 1.0 ± 0.2 measured; no divergence near the oxygen limit (0.56 against 0.95 at 14%) |
| `post.*` (0.03, 0.12, 3.7 s, exponents 0.3 / 0.3 / 0.2) | unlit droplet | patch to self-cooling evaporation: amplitude should depend on what flame preceded (0.17 after cool, 0.40 after hot), decay time on D²/K0, gas on ρD_F |
| `diff.*`, `haz.log_disruption` | burn-down end, disruption | empirical; no first-principles replacement |

**Failure 1: 2 atm, 70% CO2.** The carried D_crit is 0.77 mm, so the clock gives 0.6 s against 8.0 s measured.

- The −0.45 slope was typed from 5–30% CO2 and extrapolated exponentially. Emission grows about as the square root of CO2 path, not exponentially.
- My physical D_crit is also only 0.84 mm, so saturation alone does not fix it.
- What is absent from both laws: re-absorption of band emission by the cold dense CO2, faster chemistry at 2 atm, and Le_O = 0.75 strengthening the flame (measured K is 1.14 times air).
- If the clock pattern of the other runs applied with time scaled by gas diffusivity (0.39 of air), this run's D_crit would be about 1.8–2.2 mm. That is a one-run, after-the-fact inference; the diffusivity scaling is not supported by the helium runs.

**Failure 2: helium after the flame is lost.**

- `cool.p` sends the run to the no-flame law, whose amplitude (0.12) was typed from droplets that had just left a cool flame at about 1.7 mm.
- After a hot flame the droplet is at its boiling point (K0 about 0.40, not 0.17) and is larger, so it cools more slowly. In helium vapour also diffuses about 1.9 times faster.
- On top of that sits a dying cool flame.
- Carried gives 0.15–0.19 in the first second; self-cooling alone 0.31–0.35; measured 0.45–0.56.

## 4. Blueprint for an extended forward

**State:** q = D², stage (hot, cool, dying, none), time since stage change, D and T_s at the last change. All else is algebraic.

**Hot stage (explicit step 1/30 s, or RK4 as now):**

1. dq/dt = −K_h·h(s) + expansion term. K_h = K_ref·(atmosphere ratio)·(D0/2.9 mm)^(−0.15).
2. h(s) = 1 − a_h·exp(−s/0.31); the expansion term uses the liquid mean-temperature series (three terms).
3. Flame radius from the oxygen-side closed form.
4. Loss hazard from either the margin form (keep `rad.*` on D0/D_crit) or T_f,rad(t) against T_x. I recommend the margin form with a physical D_crit; the temperature form did not separate runs within an atmosphere.

**After loss (closed form):**

- If D > D_ce: K = K_c·(1 − a_dip·exp(−t/τ_dip)) until D = D_ce.
- If D ≤ D_ce, or pO2 is below the limit: K = (K_c − K0')·exp(−t/τ_die) + self-cooling law.
- After any flame: K = K0/(1 + t/t0) + warm-gas floor decaying to cold evaporation.

**Parameters:**

| Group | Parameter | Proposed | Uncertainty |
|---|---|---|---|
| Nature | all of 1.1 | as tabled | fixed |
| Hot | K_ref, air, 1 atm | 0.54 mm²/s | ±0.03; run-level sd 0.06 |
| Hot | Lewis weight w in Λ_eff = 1 + w(Λ − 1) for the rate | 0.5 | 0.2–1.0 (He data about 0.3, 70% CO2 about 1) |
| Hot | heat-up amplitude a_h, e-fold | 0.19, 0.31 clock units | ±0.06; e-fold fixed by α_l |
| Hot | expansion on/off scale | 1.0 for D0 ≥ 3 mm, 0.5 below | 0.3–1.0 |
| Radiative | D_crit,air | 3.7 mm | ±0.2 |
| Radiative | T_x (model scale) | 1650 K | ±100 K; dominates helium |
| Radiative | pressure exponent of D_crit | −0.1 | −0.3 to 0 |
| Radiative | CO2 path beyond 0.25 atm | square-root growth, re-absorption factor 0.5 | 0.3–1.0; widen class above 0.3 atm |
| Radiative | class sharpness, clock law | carried values | keep |
| Cool | K_c at 1 atm, N2 | 0.34 mm²/s | ±0.02 |
| Cool | helium factor | λ_g ratio (1.21 at 15%, 1.5 at 35%) | ±0.1 |
| Cool | oxygen and pressure terms 0.7–1 atm | 0 | ±0.05 per 5% O2 |
| Cool | warm branch above about 1.5 atm | 0.50 mm²/s | 0.36–0.56; one run |
| Cool | D_ce/D0 = g0·√((0.18 − β)/(pO2 − β))·(1 + c_He·x_He) | g0 0.47, β 0.11 atm, c_He 1.0 | g0 ±0.04; β 0.10–0.125; c_He 0.5–2; log sd 0.12 |
| Cool | D0 exponent | 1.0 | ±0.2 |
| Cool | dip a_dip, τ_dip (loss before 0.25 clock units) | 0.4, 1.5 s | 0.2–0.6; four runs |
| Dying | τ_die | 3 s | 2–5 s; one session |
| Unlit | T* | 26 K | ±3 |
| Unlit | T_s0 after hot / after cool | boiling − 3 K / boiling − 19 K | ±5 K |
| Unlit | warm-gas floor, decay | 0.045 mm²/s, 15 s | 0.03–0.06 |
| Unlit | cold floor | 0.010 (N2, CO2), 0.018 (He 35%) | ±30% |
| Other | diffusive end, disruption, spreads | carried values | keep |

**What a 1–5 s prefix can identify:**

- The level and whether the tracker disagrees with the sheet D0.
- The run's rate offset: weakly before 2 s (heat-up and expansion confound it), usefully by 5 s.
- Whether the hot flame is already lost: the rate falls to 0.65–0.8 of the hot rate in N2/CO2; in helium it falls and keeps falling.

**What it cannot identify:** the side of D_crit if the flame is still lit, the cool-flame end diameter, whether a cool flame near the oxygen limit will hold, disruption, drift out of view.

**Carrying uncertainty:**

- Constants of nature are fixed.
- Law-level constants (D_crit,air, T_x, β, g0, w) are shared scenario nodes. Their spread is the error of the law on a new atmosphere, not run scatter; it should be wide where train is empty (helium above 35%, CO2 path above 0.25 atm, above 1.2 atm).
- Per run: rate offset, class near D_crit, loss time, D_ce, disruption.
- Per session: the stated oxygen fraction (1% absolute moves D_crit by about 12%) and igniter energy.

**Keep from the carried forward:** the three-state structure and mixture quantiles, the end-rate state filter, the validity gate, the typed rate level and oxygen slopes, the class law inside N2 and CO2 up to 30%, the margin clock, the burn-down law, the spreads.

## 5. Unseen validation atmospheres (stated before any outcome)

Conditions only. Physical D0/D_crit includes my 0.7 atm factor; for the 0.7 atm helium runs I also give the value without it, as a hand estimate.

| Class (runs) | Expected regime | Direction against nearest train | Confidence |
|---|---|---|---|
| 34% O2, 0.71 atm, 3.01 mm (196F002) | hot flame throughout, K 0.7–0.9, heavy soot, tracker artefacts likely | as the 30% runs; the 34% train twin lost its droplet track at 0.6 s | high on regime, low on the diameter record |
| Air, 1 atm, 3.85 mm (181F001) | at the edge (1.04–1.05): flame lost at 9–18 s or held; cool flame after, ending near 0.35–0.4 D0 | larger than any train air run | medium |
| 15% O2, 1.13 mm (193R001) | burns for 1–2 s and goes out at 0.4–0.8 D0; no cool flame; then self-cooling for well under 1 s and a nearly flat diameter | smaller than any train droplet; at the hot oxygen limit | low on the end diameter, medium on "flat after" |
| 20% O2, 5% CO2: 3.06, 3.04 mm (C01H501, C01H601) | at the edge (1.04–1.09): loss around 8–10 s then cool flame to about 0.40 D0, or burn-down | same atmosphere as train C01H302 | medium |
| 20% O2, 5% CO2: 2.12 mm (C01H701) | burn-down to 0.3–0.37 D0, disruption possible | as train C01H201 | high |
| 17% O2, 45% CO2, 0.71 atm, 3.33 mm (C13H102) | flame lost within about 0.5–2 s (D0/D_crit 2.7 physical, 4.1 carried); pO2 0.12 atm is at the cool-flame limit, so no sustained cool flame: 0.15–0.30 mm²/s decaying over 5–10 s, diameter stays above about 0.9 D0 | weaker than train C10H101 in both stages | medium-high on early loss; medium on no cool flame (alternative: 0.33 mm²/s to a plateau above 0.8 D0) |
| He 15%, 18% O2, 1 atm: 3.52 mm (H03H101) | radiative loss (1.5), then a cool flame at 0.40–0.44 mm²/s to 0.55–0.70 D0 | faster cool flame and larger end diameter than N2 at 18% | medium-high |
| He 15%, 18% O2: 2.30, 2.27 mm (H03H201, H03H202) | exactly at D_crit in both laws (0.95–1.01): two-sided | — | low on side |
| He 20%, 17% O2: 2.85 mm (H04H101) | radiative loss (1.4–1.8), cool flame as above with D_ce/D0 0.6–0.75 | flame temperature like 14% O2 in N2 | medium-high |
| He 20%, 17% O2: 2.12 mm (H04H201) | physics says radiative (1.34), carried a coin flip (1.02) | physics lower D_crit (1.58 against 2.08 mm) | medium-low |
| He 40%, 18% O2, 0.7 atm: 2.27, 1.86 mm (H14H101, H14H201) | weak hot flame at K about 0.65–0.75; physics 1.29 and 1.06 (about 1.57 and 1.28 without the pressure factor), carried 1.04 and 0.86. pO2 0.125 atm: no sustained cool flame; after loss a decaying after-burn from about 0.5, shorter-lived than train because droplets are smaller | loss earlier than the 30–35% train runs, less total after-burn | medium on loss (about 0.7 and 0.5), medium-high on no plateau burn |
| He 45%, 17% O2, 0.7 atm: 2.22 mm (H15H101) | flame-sheet temperature (1634 K) is at the model extinction temperature: flame barely sustained, lost within 1–3 s; then decaying after-burn | carried says 1.18 with a 6 s clock | medium; sensitive to T_x ± 100 K |

## 6. What cannot be predicted from these inputs

- **The side of D_crit within about ±10%.** Igniter energy and drift decide it. A flame radiometer or the UV flame diameter trend in the prefix would not settle it either: within one atmosphere the model flame temperature of runs that went out and runs that did not differs by only 50–150 K.
- **Whether the visible flame is out at the cutoff when it went out less than a second earlier.** The UV trace up to the cutoff would settle it.
- **The cool-flame end to better than about 12% in diameter.** The extinction maximum is flat. A formaldehyde-band or radiometer signal would give the end directly.
- **Disruption, bubbles, soot-shell collapse.** Not in any input.
- **Drift out of view.** The position columns exist in NASA's files and are not ingested.
- **The true atmosphere of a late burn in a fill**, beyond the tabulated value.

## 7. Review aid: claims to break

| Claim | How to re-check |
|---|---|
| The Le_O ballast is real and sizes the helium penalty | `python runs/dev/research6/flex_heptane/confront2.py`, table C: helium rates fit Le = 1 better. Only D_crit supports the full penalty, and only if pressure has no effect |
| Physical D_crit reproduces the typed law | `confront5.py`. Two constants are train-read; T_x = 1650 K is a median I chose after seeing table B; three 0.7 atm runs are on the wrong side |
| D_ce scales with D0 and with ([O2] − β)^(−1/2) | `confront6.py`, table K. Twelve plateaus; the threshold is in `measure.py` (`t_plateau`); β came from two points; C02H201 is 12% off |
| Self-cooling evaporation explains the post-flame transient | `confront4.py`, G1 and G2. It decays too fast below 1.3 mm; the lumped assumption fails for N04H101; T* = 26 K is read off my own `k_evap` |
| H19H101 is a 15% He run | Its atmosphere is an unresolved conflict (sheet says air). I used it because its hot rate (0.58) matches H19H201. If it is air, the helium cool-flame evidence at 1 atm disappears. See `EXPERIMENTAL_SCIENCE.md` 1.4 |
| The 2 atm explanation | One 2009 run, assembled after the fact. `confront5.py` shows my own D_crit is as wrong as the carried one |
| Emissivity model | The Hottel anchors and the pressure-broadening factor in `physics.py::emissivity` are from memory; the factor causes the 0.7 atm misses |
| Memory-sourced properties | Yaws vapour conductivity (extrapolated above about 1000 K), heptane–He diffusivity, JANAF tables, Law–Williams weights: all in `physics.py`, tagged [mem] |
| Standoff law | `confront.py` table A and `confront6.py` table L: 16% low everywhere, 0.52–0.67 for two near-limit CO2 runs |
| Expansion estimate | Section 1.3 arithmetic was done by hand, not scripted; it over-predicts for 2.5 mm droplets |

## Sources

| Source | Read | Taken |
|---|---|---|
| Williams and Nayagam, Combust. Theory Model. 23 (2019) 748–770 | abstract, sections 1, 3, 4, 8–12, 14, 15 and parts of 2 and 6, as extracted text. Section 13 (comparison with ISS data) and all figures skipped; only its opening sentence was seen | cool-flame equations 33, 40–42, constants, scaling, oxygen limit. It contains one aggregate remark that measured cool-flame standoff is about 3 and rate about 0.5 mm²/s |
| Santa et al., Combust. Flame 151 (2007) 665–675 (other fuels, drop tower) | abstract and the lines matching "1100" | extinction near 1100 K; loss dominated by products outside the flame |
| TNF workshop radiation page | full, via fetch summary | Planck-mean coefficients (match `physics.py`) |
| Search results on Chao, Law and T'ien 1990 and on Law and Williams 1972 | search summaries only | mechanism of radiative extinction; the 0.4/0.6 recipe was not verified |
| `docs/wsp/research/lit_droplets.md` | full | published trends and its source list; it holds aggregate FLEX statements only |
| Repo: `EXPERIMENTAL_SCIENCE.md`, cards 3 and 4, D2 train anatomy, D2 `edits.wsp`, `droplet3.rs` (module documentation and spec to line 330), `research5/flex_flame/summary_tables.md` and `flame_end_train.csv` (heptane train rows), `research5/shift/tables.txt` (heptane section) | as listed | — |
| `docs/wsp/research/exp_flex.md` | not read | — |

No helper agent was launched.

## Files written

All in `runs\dev\research6\flex_heptane\` (git-ignored; no tracked file was touched):

- `tables.txt` — every script output
- `physics.py`, `load.py`, `measure.py`
- `confront.py`, `confront2.py`, `confront3.py`, `confront4.py`, `confront5.py`, `confront6.py`, `audit.py`
- `train_measure.csv`, `A_hot.csv`, `B_rad.csv`

## Statement on partitions

- **Validation:** I read conditions only (O2, CO2, He fractions, pressure, initial diameter), through `lib.experiments` and the shift tables. No validation series, test sheet, flame trace or outcome was opened.
- **Test:** no run of the test partition, no `holdout` folder, `runs/dev/data_ext/` or `tools/wsp/dev_eval.py` was touched.
- **Train:** every series and flame file was opened by an alias taken from the train list.
- **Literature:** no per-test published result of the FLEX ISS n-heptane tests was read.

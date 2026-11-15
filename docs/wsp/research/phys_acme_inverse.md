# Physics of an inverse spherical burner flame, for a mechanistic model of how it ends

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. Outcomes and raw channels of train tests only; of validation tests the conditions only.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# ACME CFI-G inverse flames: physics dossier for a mechanistic forward

**Bottom line.** A lean inverse flame of propane or n-butane ends by an oscillatory instability that sets in when the oxygen flux at the flame sheet has fallen to about 1.0 g m⁻² s⁻¹. The time to that point is set by diffusion of ambient fuel, so it barely depends on flow. The stoichiometric mixture fraction Z_st (an input you already have) separates "goes out" from "holds" far better than total burner flow. The D2 law rests on one flame that was itself marginal.

Method limits you should know first:
- The 1-D model I wrote gives usable flame radius and sheet flux, but its temperatures are wrong (too cold for every flame). I use only its transport output.
- Radiometer signals are uncalibrated volts; I use them only relatively.
- The literature helper opened Kim et al. 2023 by mistake and saw some per-test content; it says it discarded it, and none reached me (details in the closing statements).

## 1. The physics, mechanism by mechanism

### 1.1 Where the flame sits and how it grows

- The burner is inside the flame. Z is the mass fraction of ambient gas (Z = 1 in the chamber, 0 in the injected stream).
- Z_st = (Y_O,in/σ) / (Y_O,in/σ + Y_F,amb), with σ the O2/fuel stoichiometric mass ratio. I recomputed it for every train row and it matches the table.
- High Z_st means the fuel is the deficient reactant and must diffuse inward against the outflow. Low Z_st (air injected, or rich ambient) means oxygen is deficient.
- Quasi-steady radius: r_ss = m_tot / (4π ρD_F ln(1/Z_st)). The diffusivity that belongs here is the ambient fuel's, so r_ss is Le_F times the value the carried code computes with ρD = 0.0334 g m⁻¹ s⁻¹.
- No lean train flame comes near r_ss. In the 1-D model the radius at 8 s is 17–26 mm where r_ss is 34–63 mm (60–110 mm with the fuel's diffusivity).
- What the flame actually does: the fuel reaches the young flame by transient diffusion, so the flux per unit area is independent of burner flow and the area adjusts to consume the oxygen.

Closed form (O2-equivalent flux at the sheet):

```text
m''(t) = K(Z_st) · σ · Y_F,amb · ρ_amb · sqrt(D_F,amb / (π t))  +  m_O2 / (4π r_ss²)
r_f(t) = sqrt( m_O2 / (4π m''(t)) )
```

- K(Z_st) ≈ 0.8 (1 − Z_st)^−0.8, read off the 1-D model: about 1.0–1.15 at Z_st 0.3, 1.4–1.6 at 0.5, 1.6–1.95 at 0.65.
- The model gives r_f ∝ m_O2^0.5 · t^0.19 and r_f ∝ p^−0.25.
- Consequence: the time to reach any fixed flux scales as Y_F,amb² · p and does not depend on flow.

### 1.2 Flame temperature: the Lewis-number deficit

For a steady inverse flame (I re-derived this; the structure agrees with Santa et al., CTM 2007):

```text
T_f − T_amb = (T_ad − T_amb) · (1 − Z_st^(1/Le_F)) / (1 − Z_st)
```

| Case | T_ad (K) | Steady T_f before radiation (K) |
|---|---|---|
| 85 % O2 into 8.9 % propane | 2587 | about 1700 |
| 85 % O2 into 7 % n-butane | 2586 | about 1510 |
| 85 % O2 into 14 % n-butane | 2823 | about 1725 |
| 85 % O2 into 24 % ethane | 2820 | about 2290 |

- The 1-D model without radiation gives 1890 K falling to 1840 K over 12 s for the first row, so the young flame is hotter than the steady limit and cools as its structure thickens.
- This is why T_ad does not order the train outcomes: the hottest nominal flames (85 % O2 into heavy lean fuel) carry the largest deficit.

### 1.3 Radiation

- Optically thin emission with the TNF Planck-mean fits is too strong here: at 2.2 atm the hot ball inside the flame has κL ≈ 0.1–0.3 and the CO2 bands self-absorb.
- With a self-absorption factor (0.37–0.48, from the Smith–Shen–Friedman grey-gas fit) my model still puts every lean flame below 1130 K within 1–4 s, including air-injected flames that burned 65 s. So the radiation is still at least twice too strong.
- Measured, relative only: radiometer signal per mg/s of O2 at 3–6 s is about 0.40 for 85 % lean flames at 1.1–3.3 atm, 0.22–0.30 for air-injected flames (rising to 0.35–0.40 by 60 s), 0.45–0.65 for rich flames.

### 1.4 What ends a lean flame: oscillation

The radiometers show it directly (`bursts.csv`):

- Every propane or butane flame that goes out first shows bursts of radiant output (1.1–2.2 times the running level, period 1.2–1.8 s) and dies during or just after one, within 0.3–0.5 s.
- Example, 21228A2: signal falls 7 % over 0.3 s, doubles in 0.25 s, flame gone 0.9 s later. The photomultipliers are saturated until the last half second.
- The thin-filament tests of the same condition survive 3–7 bursts where the others survive 1–2 (21228B1: seven bursts, 16 s, against 21228A1: one burst, 7.9 s). A hot fibre acts as a re-igniter. This is the strongest evidence that death is a failed re-ignition, not a smooth radiative fade.
- Air-injected flames that survive ignition do not oscillate at all (21228F1, 21228L2, 21256F1).
- Ethane flames do not oscillate at any Z_st up to 0.53, even after 137 s.

Published theory agrees in direction (Matalon, NASA CP 2003, lines I extracted myself): oscillations need a large Lewis number of the reactant diffusing against the stream, are more likely when the stream reactant is in excess, are promoted by heat loss, and occur only near extinction. Mapping that to this geometry (ambient fuel is the counter-diffusing reactant, so excess means Z_st above about 0.5) is my inference.

**Onset criterion.** Evaluating the 1-D model's own sheet flux at the observed onset time of each simulated train flame (`onset_flux.csv`):

| Group | Flames | Flux at onset (g O2 m⁻² s⁻¹) |
|---|---|---|
| Propane and butane, O2 ≥ 5.4 mg/s, Z_st 0.33–0.67, 1.1–3.3 atm | 12 | 0.82–1.21, mean 1.01, sd 0.11 |
| 21270H2 (2.68 mg/s, never oscillated in 55 s) | 1 | never below 1.04 |
| Ethane, 85 % O2, 0.55 atm (no oscillation) | 1 | down to 0.72 |
| Air into lean propane or butane (no oscillation) | 3 | 0.28–0.38 at the stop |

- The published ethylene extinction flux is 0.68 g m⁻² s⁻¹ (Irace et al. 2023, abstract only). Propane and butane oscillate at about 1.5 times that.
- Pressure: 0.94–1.03 at 1.1 atm, 0.94–1.21 at 2.2 atm, 1.18 at 3.3 atm (one flame). A weak rise, roughly p^0.15.
- Systematic uncertainty about ±20 %: without radiation the model's flux is 18 % higher at the same time.

**After onset.**

| Class | Train evidence | Outcome |
|---|---|---|
| Z_st ≥ 0.59 | all flames, including the four at 9 mg/s O2 | dead in 1–3 cycles (0.5–6 s) |
| Z_st 0.47–0.55, O2 ≤ 5.4 mg/s, lean ambient | 5 of 5 (plus 21222C1/C2 at 0.551) | dead in 1–2 cycles |
| Z_st 0.47–0.53, 14 % butane | 21270H1, H5 died after 18–21 cycles; H3, H4 still cycling at the stop | mixed |
| Z_st 0.47–0.53, O2 ≈ 9 mg/s | 21228C2 (19 cycles to the 66 s stop), 21270H4 (23 cycles) | held in a limit cycle, 2 of 2 |
| Z_st 0.33–0.45, T_ad ≥ 2680 K | 21270A1, A2, C1 | weak oscillation, held |
| Z_st ≤ 0.33, T_ad ≤ 1960 K | three air flames | no oscillation, held |

### 1.5 Why a rich ambient holds

- Lower Z_st means a smaller Lewis deficit and a higher flux floor (the steady term), so the flux reaches 1.0 late or never.
- 17.5 % butane at 5.37 mg/s: onset at 53 s. At 4.03 mg/s: none in 96 s.
- The test then ends on the script or the burner limit.

### 1.6 The role of total burner flow

Three real roles, none of them the D2 law:

1. **Burner screening.** Conduction to the sphere falls as Pe/(e^Pe − 1), with Pe = m_tot c_p / (4π k r_b) ≈ 0.38 per mg/s. Measured initial heating, taking the sphere at 0.31 J/K: 24 % of the heat release at m_tot 2.1 mg/s, 9 % at 4.2, 4.5 % at 6.3, 1.5 % at 10.4, 0.5 % at 17–23. That loss is why the lowest flows die at 3–5 s.
2. **The steady floor**, through r_ss. A larger flow lowers it, which makes a flame oscillate sooner, not later (14 % butane: onset 12.5 s at 8.95 mg/s, 22 s at 5.37, 34 s at 4.03).
3. **Size in the marginal band.** What orders survival there is the O2 flow, not m_tot: 21270H4 held at m_tot 10.3 mg/s while 21228C1, 21228K1 and 21256C1 died at 10.1–10.2.

### 1.7 What air injection changes

- Z_st drops to 0.28–0.33, oxygen (Le ≈ 1) becomes the deficient reactant, and nothing drives the pulsation.
- The flame then lives to the static radiative limit. The critical flux there is unobserved, below 0.28. If it is about 0.22, the end comes near 100 s (range 60–250 s), after the 66 s scripts.
- The flash at ignition is a separate failure. In train it is ordered by ambient fuel and flow, not T_ad: at 5.4 mg/s, ambient ≥ 8.7 % held (3 of 3) and ≤ 8.0 % flashed (2 of 2); 3.63 mg/s at 8.7 % flashed. T_ad was 1835–1879 K for flashes and 1867–1956 K for holds.

### 1.8 Burner heating and limit trips

```text
C_b dT_b/dt = 4π k r_b (T_g − T_b) · Pe/(e^Pe − 1) / (1 − r_b/r_f)  +  a_b (r_b / 2 r_f)² Q_rad  −  losses
```

- The conduction term reproduces the measured initial heating within a factor 1.5–2 for m_tot ≤ 6 mg/s. Above 10 mg/s it is 10–50 times too small; the residual 0.3–0.6 W fits absorbed radiation.
- The carried empirical law checks out where I evaluated it: 97 °C at 10 s for 85 % lean at 5.44 mg/s (measured 97.5); 137 °C at 66 s for air at 23 mg/s (measured 117–131).

### 1.9 Cool flames at 3.3 atm with n-butane

- Unlikely, and outside the target either way.
- In all 33 train self-extinctions that outlived 5 s (butane at 2.2 atm included) the radiometers decay smoothly after the flame, with the same curve for butane and propane, and no plateau, while flow continued 15–30 s.
- Burner peaks were 55–260 °C at extinction; the published threshold for cool flames in normal flames is 515 K (242 °C).

### 1.10 Property data

| Quantity | Ethane | Propane | n-Butane | Source |
|---|---|---|---|---|
| σ (g O2 per g fuel) | 3.725 | 3.628 | 3.579 | stoichiometry |
| Heat per g O2 (kJ) | 12.76 | 12.78 | 12.78 | LHV / σ, checked |
| D in N2 at 300 K, 1 atm (cm²/s), ∝ T^1.7/p | 0.160 | 0.125 | 0.106 | McGivern and Manion, via helper; not independently verified |
| Le_F (α_N2 = 0.222 cm²/s) | 1.39 | 1.78 | 2.10 | helper's calculation; Santa gives 1.98 for propane |

- O2: Le ≈ 1.05. N2 at 300 K: ρα = 2.59e-5 kg m⁻¹ s⁻¹, taken ∝ T^0.7.
- Planck-mean fits (TNF), a_p = Σ c_n (1000/T)^n in m⁻¹ atm⁻¹: CO2 18.741, −121.310, 273.500, −194.050, 56.310, −5.8169; H2O −0.23093, −1.12390, 9.41530, −2.99880, 0.51382, −1.86840e-5.
- Grey-gas fit (Smith et al. 1982, p_w/p_c = 1), verified against a reproduction: k = 0.4303, 7.055, 178.1 (atm m)⁻¹.
- Burner: 6.35 mm sphere, 0.617 g, 0.31 J/K (sphere only; tube and thermocouple add an unknown amount).

## 2. Confrontation with train

**Which variable separates the 65 flames alive at 5 s (33 self-extinguished, 32 flow-terminated).** Overlap means the range where both outcomes occur (`variables.py`):

| Variable | Self-extinguished | Flow-terminated | In overlap (SE / FT) |
|---|---|---|---|
| Z_st | 0.472–0.665 | 0.127–0.527 | 5 / 9 |
| Ambient fuel fraction | 0.070–0.140 | 0.087–0.429 | 22 / 7 |
| Exit velocity (cm/s) | 0.4–6.4 | 1.9–29.9 | 18 / 11 |
| Total flow (mg/s) | 2.1–10.5 | 3.1–33.3 | 25 / 17 |
| Injected O2 fraction | 0.50–0.85 | 0.15–0.85 | 33 / 16 |
| T_ad (K) | 2359–2823 | 1783–2978 | 33 / 14 |
| Steady radius r_ss (mm) | 9.4–61 | 9.1–63 | 33 / 30 |

Inside the Z_st overlap: all five ethane flames held; propane and butane split 5 died, 4 held, ordered by O2 flow as in 1.6.

**Lives in seconds, closed-form flux law with K = 2 and critical flux 1.2** (`fluxlaw.py`), against observed onset:

| Cell | Predicted onset (s) | Observed onset (s) | Observed end (s) |
|---|---|---|---|
| Propane 8.0 %, 3.3 atm, 85 %, 3.6–9 mg/s | 10.5–12.7 | 9.3–12.5 | 10–17 |
| Propane 8.9 %, 2.2 atm, 85 % and 50 %, 3.6–9 mg/s | 8.5–11.1 | 6.0–9.8 | 6.5–12 (16 with fibre) |
| Propane 8.7 %, 1.1 atm, 5.44 mg/s | 4.4–4.6 | 6.6–7.0 | 7.3–9.2 |
| Butane 7.0 %, 2.2 atm, 3.6–9 mg/s | 7.1–8.4 | 6.1–6.8 | 6.7–7.3 |
| Butane 8.8 %, 2.2 atm, 3.6–9 mg/s | 11.4–14.3 | 7.8–8.7 | 8.5–12.9 |
| Butane 14 %, 1.1 atm: 8.95 / 5.37 / 4.03 / 2.68 mg/s | 14 / 18 / 21 / 33 | 12.5 / 22 / 34 / none | held / 49 / held / limit |
| Butane 17.5 %, 1.1 atm: 8.95 / 5.37 / 4.03 mg/s | 23 / 31 / 38 | 18.6 / 53 / none | held |
| Ethane 20–24 %, 0.55 atm | 17–31 | none | held |
| Any, O2 ≤ 1.8 mg/s, lean | 16–22 | 8–14 | 8.5–17 |

Mismatches and what they imply:
- **Pressure.** The formula goes as p; the data go roughly as p^0.4 between 1.1 and 2.2 atm and steeper between 2.2 and 3.3 atm (one session). The critical flux is not pressure-free, or sessions differ.
- **Flow dependence in a rich ambient** is too weak: the steady floor is uncertain by about a factor 2 through ρD_F.
- **Ethane** needs a critical flux of 0.7 or less. With the published 0.68 the formula gives 114–126 s for 9.07 mg/s; 21209B1 burned 137 s without oscillating.
- **Low flows** die earlier than the flux law says: burner heat loss, a separate mechanism.
- **With the published 0.68 for all fuels** the lean lives come out at 16–62 s, three to five times too long. This is the quantitative form of "the published law gives minutes".

**Candidates for the end of a lean flame.**

| Candidate | Verdict | Evidence |
|---|---|---|
| Oscillation near a critical flux | supported | bursts in every dying flame; onset at one flux across 12 flames; fibre effect |
| Radiative loss alone | necessary, not sufficient | air flames survive at a third of the flux |
| Burner heat loss | only below about 2.5 mg/s total flow | 15–25 % of heat release there, under 5 % above 6 mg/s |
| Quench hole at the supply tube | plausible seat of the oscillation | not separable without images |
| Ignition hand-over failure | the flash class only (under 3 s) | 21228L1, 21256M1, 21222F1 |

For 50 % O2 lean flames the radiant output per unit O2 halves between 2 and 6 s before the first burst; 85 % flames stay flat. I do not have a mechanism for that decline.

**Table problems that affect targets and inputs.**
- 21228C4, 21228K2 and 21256C2 are tabulated at 50 % O2 but the flow readbacks show the 85 % bottle alone. Their Z_st and T_ad inputs are wrong, and they behave like 85 % flames.
- 21196M2_1 is tabulated at 5.59 mg/s; the readback is 3.02.
- Session 2021-09-27: table burn times disagree with the photomultiplier end (H2 43 vs 55.5 s; H3 43 vs 93.3 s; A1 60 vs 65.9; A2 90 vs 96; A3 35 vs 29.5).

## 3. Audit of the carried model (D1) and the new scenario (D2)

**D1**

| Tensor | Value | Mechanism | Verdict |
|---|---|---|---|
| `se.osc.skip[3]` (ambient fuel) | +3.0 per e-fold | flux law gives 2 in the transient regime, steeper near holding | sign right, size defensible; the true variable is Z_st with Le_F |
| `se.osc.skip[2]` (O2 fraction) | +0.45 | none | patch; partly an artefact of three mis-tabulated "50 %" tests |
| `se.osc.skip[0]` (pressure) | 0 | flux law and the 3.3 atm session say positive | missing; D1 gives about 10 s where propane at 3.3 atm lasted 10–18 s |
| `se.osc` unit 0 | held above 13.5 % fuel | holding is Z_st below about 0.45 plus the floor | right for 85 % O2 only |
| `se.osc` unit 1 | −8 below about 1.8 mg/s O2 | burner conduction | real; should be written in total flow |
| `se.osc` unit 2 | +8 below 35 % O2 | no pulsation at Z_st ≤ 0.33 | right outcome, wrong variable; says nothing at 40 % |
| `se.flux.b0`, `se.diff.b0` | growth clock off | critical flux with transient growth | mechanism is real; the erfc growth to r_ss with cold D and the 0.68 constant are what fail |
| `se.flash` | logistic at 1880 K, lean only | ignition hand-over | T_ad does not discriminate in train; ambient fuel and flow do |
| `se.floor_s` | 2.0 | establishment | consistent (earliest non-flash deaths 3.5–5 s) |
| `bt.rise` | 2012 K · m_tot^−0.4 · r_ss^−0.4, 30 s | conduction is exponential in flow; r_ss is not the radius | empirical surrogate that works on train; keep |
| `bt.limit_c` | 455 / 510 °C | session setting | keep |

**D2, unit 3** (`se.osc.W1[3,1] = 1`, `W1[3,2] = −0.5`, `b1[3] = −0.245`, `w2[3] = 9`)

- Evidence is one flame (21228C2). The air flames it also claims are already explained by Z_st ≤ 0.33.
- Total flow is not the variable: 21270H4 held at 10.3 mg/s while three flames died at 10.1–10.2. The stated reason (flame further from the burner and tube) is contradicted by 21228A2, which has the same model radius as 21228C2 (25 mm at 8 s) and died at its first burst.
- Its −0.5·c_o2 term does keep it off for 70–85 % O2, which matches train.
- Size: 21228C2 survived 19 near-death cycles. A +9 per e-fold (0.08 of going out by 66 s) is too confident.
- Physics supports a weight of about one third on this scenario at 40–50 % O2 and highest flow, better expressed as "Z_st 0.47–0.56 and O2 flow ≥ about 8 mg/s".

## 4. Blueprint for a mechanistic forward

**Derived quantities** (from inputs only): Y_O,in, Y_F,amb, m_tot = m_O2 / Y_O,in, Z_st, Le_F, σ, D_F,amb(p), ρ_amb(p), Pe = 0.38 · m_tot [mg/s].

**State and equations**, explicit steps of 0.1 s:

1. m''(t) from the closed form in 1.1, multiplied by (1 − φ_b) with φ_b = c_b · Pe/(e^Pe − 1) for burner loss.
2. Onset: draw ln m''_osc ~ Normal; onset is the first time m''(t) < m''_osc. If the floor exceeds it, no onset.
3. After onset: per-cycle death probability q by class, period P_osc.
4. No-oscillation class: static limit at m''_rad.
5. Flash class at ignition; burner ODE (or the carried law); stop atoms. Keep all three as they are.

| Group | Parameter | Proposed | Uncertainty | Varies by |
|---|---|---|---|---|
| Supply | K = k0 (1 − Z_st)^−k1 | k0 0.8, k1 0.8 | ±15 % | constant |
| Supply | ρD_F for the floor | 0.0334 · 1.05 / Le_F g m⁻¹ s⁻¹ | factor 1.5 | constant |
| Onset | m''_osc, propane and butane | 1.0 g m⁻² s⁻¹ at 2.2 atm | sd 0.11 between flames; ±20 % systematic | flame to flame |
| Onset | pressure exponent | 0.15 | 0–0.4 | constant |
| Onset | ethane critical flux | 0.6 | 0.45–0.7, unobserved | constant |
| Static limit | m''_rad at Z_st ≤ 0.35, T_ad ≤ 2100 K | 0.22 | 0.12–0.28, unobserved | constant |
| After onset | q, Z_st ≥ 0.57 | 0.5 per cycle | 0.3–0.8 | flame to flame; fibre lowers it |
| After onset | q, Z_st 0.45–0.57, O2 ≤ 6 mg/s, lean | 0.5 | 0.3–0.8 | — |
| After onset | q, Z_st 0.45–0.57, 14 % ambient | 0.04 | 0.01–0.1 | — |
| After onset | probability of a limit cycle, Z_st 0.45–0.57, O2 ≥ 8 mg/s | 0.5 | 0.25–0.8 (2 of 2 in train) | — |
| After onset | P_osc | 1.5 s | 1.2–1.8, lengthening | — |
| Burner loss | c_b | 0.3 | 0.2–0.45 | constant |
| Burner | C_b | 0.31 J/K | +0 to +60 % | constant |
| Session | limit, script time-out | as carried | — | test day |

**How to carry uncertainty.** Constants of nature (σ, D, Le, heat per g O2) fixed. One log-normal draw on m''_osc per flame, and a second, session-level one of about 0.15: the 3.3 atm session sits 15 % above the others. Class probabilities are the main uncertainty and should be written as mixtures, as the carried forward already does with nodes.

**What to keep.** The heating law, the two limits, the stop atoms, the 2 s floor, the weak class as a class. Replace the ambient-fuel clock by the flux clock with Z_st classes, or as a smaller step add a pressure term and re-express units 0, 2 and 3 in Z_st.

## 5. Unseen validation regimes (stated before any outcome)

Conditions only. "Hold" means burning at the script stop, most likely 63–67 s.

**n-Butane 5.7 %, 3.3 atm** (leaner than any train cell; Y_F 0.11)

| Injected O2 | O2 flow (mg/s) | Z_st | Forecast | Against nearest train | Confidence |
|---|---|---|---|---|---|
| 85 % | 3.6–8.9 | 0.68 | goes out at 7–13 s, median about 9 | shorter than propane 8 % at 3.3 atm (10–18 s); longer than D1's 5 s | medium; the pressure effect rests on one session |
| 85 % | 1.8, 2.7 | 0.68 | 4–8 s and 6–11 s | burner loss | medium |
| 70 % | 8.9 | 0.65 | goes out at 5–10 s, median 7 | as the 85 % flames | medium-high |
| 50 % | 3.6, 5.4 | 0.57 | goes out at 4–8 s | as 21256K1, 21222C1/C2 | medium-high |
| 50 % | 8.9 | 0.57 | goes out at 5–9 s with 0.75; else holds | beyond the band where 21228C2 held | low |
| 45 % | 8.9 | 0.55 | holds with 0.35 | between | low |
| 40 % | 8.9 | 0.52 | holds with 0.40; else out at 5–10 s | same Z_st and flow as 21228C2, but leaner, heavier fuel, T_ad 2209 K | low |
| 40 % | 5.4 | 0.52 | goes out at 4–9 s with 0.85 | as 21228C1 | medium |
| 21 % | 5.4, 8.9 | 0.37 | flash before 3 s with 0.8 and 0.6; if alive at 5 s, holds with 0.75 | leaner and colder (1807 K) than every train flash | medium |

Why 40–45 % is genuinely open: T_ad there (2203–2261 K) lies between the train's non-oscillating flames (≤ 1956 K for propane and butane) and its oscillating ones (≥ 2359 K). The critical flux in between is unknown, anywhere from 0.3 to 1.0. D1 says about 0.03 of holding, D2 about 0.9; the physics says about 0.4.

**Propane about 20 %, 0.5 and 1.1 atm**

| Condition | Z_st | Forecast | Confidence |
|---|---|---|---|
| 85 %, 1.1 atm, 5.4–9.1 mg/s | 0.45–0.46 | holds (0.9); weak oscillation from about 26 s at 9.07, about 50 s at 5.44; burner about 300–350 °C at 66 s, no trip | high |
| 85 %, 0.5 atm | 0.46–0.47 | holds (0.85); burner 290–420 °C at 66 s; trip only if the script runs past 90 s | medium-high |
| 50 %, 3.6 mg/s, 1.1 atm | 0.34 | holds | high |
| 18–21 %, 1.4–1.8 mg/s, 1.1 atm | 0.16–0.19 | holds if lit; small flame near the burner; ignition risk at 1.36 mg/s | medium |

The nearest train cell is butane 17.5 % (all held). Propane has the smaller Lewis number, so it should be at least as stable.

## 6. What the conditions cannot tell

- **Which side a flame at Z_st 0.47–0.57 falls on.** Train shows both outcomes at one condition.
- **The thin-filament fibre.** It adds 5–8 s; it is a set-up fact but not an input.
- **The actual flows.** Three train tests ran without their nitrogen.
- **Script time-out and limit setting** of the day.
- **Ignition success** for air into a lean ambient.

Signals in the first 3–5 s that would change the forecast:
- **Burner temperature slope.** It gives the true total flow: about 17–19 K/s at 2 mg/s against 2.4 K/s at 10 mg/s, which exposes a missing nitrogen flow at once.
- **Flow readbacks.**
- **Radiometer signal per unit O2 between 2 and 5 s.** Falling by 25–40 % marks the failing diluted flames; flat marks an 85 % flame.
- **The first burst** after 3 s: death follows within 1–3 cycles for Z_st ≥ 0.57.
- **Photomultipliers** are saturated and only mark the end.

## 7. Claims a reviewer should try to break

| Claim | How to re-check |
|---|---|
| Bursts are flame oscillations, not igniter, fibre or camera events | `bursts.py`; inspect 21228C2, 21270H4 at 125 Hz. The burst at 1.6 s in nearly every test is an ignition artefact and is excluded |
| Onset at 1.0 ± 0.11 g m⁻² s⁻¹ | `python onset_flux.py`; rerun `sim1d.py` with rad 0 and 1 to see the ±20 % shift; grid and the 1 ms chemistry relaxation were not tested |
| K(Z_st) | last block of `python fluxlaw.py` |
| Z_st beats total flow | `python variables.py`; note the three corrected "50 %" tests |
| Size, not total flow, in the marginal band | rests on six flames; `variables.csv` rows with zst 0.47–0.53 |
| Ethane never oscillates | `bursts.csv`, 2021-07-15 and 07-28 rows; 21196F1 has two isolated events |
| Pressure lengthens lean lives | one session at 3.3 atm; could be a session effect |
| Air flames would end near 100 s | pure extrapolation; no train flame tests it |
| Lewis numbers and D values | helper's values, not independently verified; the temperature formula in 1.2 I derived myself |
| My mapping of Matalon's "rich" to high Z_st | by symmetry, not from a paper on inverse burner flames |
| Burner heat capacity 0.31 J/K | sphere only; loss fractions in 1.6 scale with it |

## Sources

**Read by me**
- Repo documents named in the brief (full): `EXPERIMENTAL_SCIENCE.md`, `lit_spherical_flames.md`, `exp_acme.md`, `acme_growth.md`, `flame3.rs`, both `edits.wsp`, `acme_inverse_train.md`, `summary.txt`, the inverse section of `shift/tables.txt`. `acme_clock.md` and `acme_blend.md` were not read.
- Matalon, NASA/CP-2003-212376 (ntrs 20040053535): matched lines only, for the Lewis-number, mixture-strength and heat-loss statements.
- arXiv 2411.18467: one table, for the Smith–Shen–Friedman coefficients.

**Read by the helper** (Fable 5.2, high effort; web only, no local files)
- Full text: Matalon 2003; Robert and Monkewitz 2025; Dietrich et al. (candle flames); Law, Yoo, Christiansen 2003; Irace et al. CNF 2021; Santa et al. CNF 2007, CTM 2007, AIAA 2007; Sunderland et al. 2003; Frolov et al. 2023 (two papers); TNF radiation page; RadLib page; Berlad NACA report; NIST SP 958.
- Abstract or snippet only: Cheatham and Matalon 2000; Metzener and Matalon 2006; Sohn et al. 1999 and 2000; Wang and Law; Christiansen, Tse, Law 2003; Yoo et al.; Wang, Bechtold, Law; Daou et al.; Short et al.; Thatcher et al.; Irace et al. PCI 2023; Nayagam et al. 2019; Mills and Matalon (both); Alfazazi et al.
- Parts: Waddell thesis (burner model, ethylene results); Ju PCI 2021 (first 70 %); Kim et al. PCI 2023 (general statements only, see below).
- Taken: oscillation conditions and frequencies, ethylene critical flux and temperature, soot against Z_st, property data, cool-flame regimes, burner model.

## Files written

All under `runs\dev\research6\acme_inverse\`:
- `traces.py`, `traces_0p25s.json`
- `bursts.py`, `bursts.csv`
- `variables.py`, `variables.csv`
- `sim1d.py`, `sim_batch.py`, `sim_batch.log`, `sim\*.json` (41 runs)
- `fluxlaw.py`, `onset_flux.py`, `onset_flux.csv`

A throwaway PDF text extractor is in the session scratchpad. No tracked file was modified.

## Statements

- **Validation:** I read conditions only (fuel, pressure, O2 flow, O2 fraction, ambient fuel fraction, T_ad, Z_st) through `lib.experiments`, plus the inverse section of `shift/tables.txt`. No validation outcome or raw file was opened. The 16 validation-condition runs of the 1-D model use those inputs only.
- **Test partition:** nothing read. `holdout`, `data_ext` and `dev_eval.py` were not touched.
- **Raw files:** only the 73 train inverse tests, selected by exact name from `lib.partition_ids`.
- **Published per-test CFI-G results:** I read none. The helper reports that a search result mislabelled Kim et al. 2023 and its reader returned one representative flame's oscillation and extinction times, a range of cool-flame durations and the test-matrix ranges. It says it discarded them; its report to me contains none, and nothing here uses them. From that paper only model equations, the fitted ρD and qualitative statements were kept. It did not open the Kim thesis, the Final Report or the PSI-159 tables.
- **No fitting:** every constant is published, derived, or read off a direct evaluation. The two critical-flux values compared (0.68 and 1.2) and the pressure exponent are stated as such.

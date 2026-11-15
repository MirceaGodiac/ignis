# Published science of burner-supported spherical diffusion flames in microgravity

> Literature review by a delegated research agent (claude-opus-5-5), 14 November 2026. Its rule: mechanisms,
> theory, numerical-model predictions and published correlations, with the kind of every number stated; no
> per-test results of the experiment series being modelled. The author used it to choose laws and typed every
> number himself. Sources are listed at the end; claims marked as the agent's own derivation are not published.

I've covered the eleven questions below. Four main sources were read in full: Santa et al. 2007, Kim et al. 2023 (the CFI‑G cool‑flame paper), Waddell's 2023 thesis and the NASA ACME/CFI‑G reports. Several key papers were seen as abstracts only: Irace 2021 and both Irace 2023 papers, Tse 2001, Mills & Matalon 1997/1998, Nayagam 2019 and Christiansen 2003. Any number from those is marked "(abstract)".

**What each number is:**
- **[T]** theory
- **[N]** numerical model
- **[ISS]** measured or correlated from ISS ACME campaigns (Flame Design is ethylene; CFI‑G is ethane, propane and n‑butane)
- **[DT]** drop tower (2.2 s)
- **[D]** derived here by combining published pieces; not published anywhere

No per‑test CFI‑G results are given. Some files already in the shared scratch folder looked like per‑test data (`normal.csv`, `inverse.csv`, `psi159.json`, `flowtraces.pdf`, `tpnotes.pdf`). I did not use them.

---

## 1. Transient flame growth

- **Quasi‑steady radius, normal flame** (Spalding droplet model applied to the porous burner; unit Lewis number, constant ρD, no radiation):
  - r_f = ṁ / [4π ρD ln(1/(1−Z_st))], where ṁ is the **total** burner mass flow.
  - Mixture fraction at the burner surface: Z_s = 1 − exp(−ṁ / (4π r_s ρD)).
  - [T] Kim et al. 2023, eqs. 7–8.
- **Inverse flame:** r_f = ṁ / [4π ρD ln(1/Z_st)], with Z = 1 in the fuel‑bearing ambient. [T] Kim et al. 2023, eq. 13.
- **Fitted ρD = 0.0334 g m⁻¹ s⁻¹.**
  - It was fitted to ISS n‑butane hot and cool flames (X_O2 0.37–0.40, 0.5–2 bar, 0.75–2 mg/s), with R² = 0.92 between model and measurement. [ISS]
  - For comparison, an O2/N2 mixture at 298 K has ρD = 0.0248. The authors say measured hot flames were smaller than their steady counterparts, so the "true" value is higher.
  - [Kim et al. 2023; Kim MS thesis 2022]
- **No pressure dependence:** ρD does not depend on pressure, so r_f at fixed ṁ and Z_st does not either. [T]
- **Low‑convection form:** for small ṁ, ln(1/(1−Z_st)) ≈ Z_st, so r_f ≈ ṁ/(4πρD·Z_st). [T]
- **Earlier statements of the scaling:** flame radius ∝ flow in quasi‑steady flames. [T] Mills & Matalon 1997; Santa 2007.
- **Approach to steady state is slow.**
  - Radial velocity falls as 1/r², so a few burner diameters out the transport is pure diffusion. The time scale is r²/D: with r = 25 mm and D = 20 mm²/s this is about 30 s. About 30 s of jitter‑free microgravity is needed to approach quasi‑steady state. [T/N] CFI‑G proposal, quoted in the ACME/CFI‑G Final Report 2022.
  - Flames were still growing at 5 s in the 5 s drop facility. [DT, same report]
  - In 2.2 s drops, steady state was never reached, especially with pure fuel or pure O2 from the burner. [DT/N] Sunderland 2003; Tse 2001 (abstract).
- **Closed form for the transient [D].** In the far field, the burner acts like a continuous point source in a still medium. This gives Z(r,t) ≈ (ṁ/4πρDr)·erfc(r/2√(Dt)), so r_f(t) solves r_f = r_ss·erfc(r_f/2√(Dt)).
  - Late in time, r_f ≈ r_ss·[1 − r_ss/√(πDt)]. The approach is algebraic (t^−½), not exponential.
  - Reaching 90 % of r_ss takes about 32·r_ss²/D, i.e. minutes for r_ss ≈ 20 mm.
  - This explains why "flames keep growing".
- **Radiation‑induced growth.** Radiative cooling lowers transport properties, which adds to the growth. [ISS/N] Irace et al. PCI 39 (2023) (abstract).
- **Measured growth exponents in drop towers:** at extinction, r_ext ∝ ṁ^0.44 (ethylene inverse) and ṁ^0.36 (propane normal); Tse 2001 gave about ṁ^0.5. [DT/N] Santa 2007.
- **Qualitative ISS trends:** flame size rises with flow and falls with ambient O2. Large flames grow until radiative extinction; very small flames approach steady state asymptotically. [ISS] Stocker JASMAC 2021, Flame Design.
- **Geometry:** flames sit offset toward the supply tube and are quenched around it. Measured radius uncertainty is ±4 %. [ISS] Waddell thesis 2023.

## 2. Radiative extinction

- **Mechanism.**
  - Products (CO2, H2O, CO) build up outside the flame, so radiative loss Q_R scales with **flame area**, not volume. The outer product layer has a diffusion thickness δ ~ (Dt)^½, independent of flow; the fitted exponent was t^0.32.
  - Extinction comes from a slow rise of Q_R **plus** a fall of heat release Q_C as the flame moves away from the burner and receives less reactant.
  - Extinction occurs at Q_R/Q_C ≈ 0.7.
  - Reactant leakage is negligible until extinction, so this is not a low‑Damköhler event.
  - [N/DT] Santa 2007
- **Critical temperature.**
  - About 1100 K (ethylene and propane). [N/DT] Santa 2007
  - It barely changes with flow: 1100 K at low flow to 1055 K at 104 mg/s. [N] Santa 2007
  - **1130 K for ethylene at 1 bar** across normal flames (X_O2 0.2–0.38, fuel mole fraction 0.13–1, 0.6–12.2 mg/s, T_ad 2000–2800 K) and inverse flames (ambient fuel 0.08–0.12, burner X_O2 0.4–0.85, 2.3–11.3 mg/s, T_ad 2080–2590 K). [ISS + N] Irace et al. PCI 39 (2023) 1905 (abstract).
  - Check on 17 ISS flames: 1130 ± 23 K measured vs 1126 ± 6 K modelled. [ISS/N] Waddell thesis.
  - Kinetic‑extinction temperatures from 1‑g literature are much higher: 1500 ± 50 K (Williams) and 1600 K (Macek). [quoted in Santa 2007]
- **Critical reactant mass flux at extinction (ethylene, 1 bar):**
  - 0.2 g m⁻² s⁻¹, fuel‑based, normal flames.
  - 0.68 g m⁻² s⁻¹, O2‑based, inverse flames.
  - A simple equation gives the extinction radius from reactant flow alone. [ISS/N] Irace PCI 2023 (abstract).
  - My reading [D]: r_ext = √(ṁ_R / (4π m″_cr)), where ṁ_R is the burner‑side reactant mass flow. This matches the ~ṁ^0.4–0.5 exponents above. Please check the exact definition against the full text.
  - **Converted to heat‑release flux [D]**, both values give the same number:
    - 0.2 g m⁻² s⁻¹ × 47.2 kJ/g (ethylene) ≈ 9.4 kW m⁻²
    - 0.68 × 13.8 kJ/g O2 ≈ 9.4 kW m⁻²
    - This suggests a **critical heat‑release flux q″_cr ≈ 9.4 kW m⁻²**, independent of configuration.
    - For alkanes (ΔH_c ≈ 45.7–47.5 kJ/g, about 13 kJ/g O2), the same flux gives m″_F,cr ≈ 0.20 g m⁻² s⁻¹ and m″_O2,cr ≈ 0.7 g m⁻² s⁻¹. This is a hypothesis, not a measurement.
- **Alternative scaling:** r_ext ∝ (ṁ·Y_R / κ_P)^{1/3}, with κ_P the Planck‑mean absorption coefficient. [T] Nayagam, Dietrich & Williams 2019 (abstract).
- **Is there a flow a flame can hold indefinitely?** Yes.
  - Steady flames exist if the steady flame is smaller than the radiative extinction size **and** forms before radiation cools it to 1130 K. Steady behaviour was seen for up to 870 s. [ISS/N] Irace et al. PCI 39 (2023), "On the existence of steady‑state…" (abstract).
  - Very small flames approach steady state. [ISS] JASMAC 2021
- **Threshold flow [D].** Setting r_ss = r_ext gives:
  - Normal: **ṁ\*_F = 4π(ρD)² L² Y_F² / m″_F,cr**, with L = ln(1/(1−Z_st)).
  - Inverse: ṁ\*_O2 uses L = ln(1/Z_st), Y_O2 and m″_O2,cr.
  - With ρD = 0.0334 and m″ = 0.2:
    - pure propane or butane in air (Z_st ≈ 0.06): ṁ\*_F ≈ 0.27 mg/s
    - pure butane in 40 % O2 (Z_st ≈ 0.11): ≈ 0.9 mg/s
    - propane at Y_F = 0.3 in 40 % O2 (Z_st ≈ 0.28): ≈ 0.7 mg/s
  - This reproduces "longer life with more O2" (higher Z_st gives a smaller r_ss).
- **Burner as heat sink.**
  - At low flow the flame is close to the burner and loses heat to it, so it starts cooler and reaches T_cr sooner.
  - Burner loss cannot itself trigger extinction, because it decays as the flame grows.
  - Above about 8–10 mg/s (drop‑tower conditions) temperature histories no longer depend on flow.
  - [N/DT] Santa 2007
- **Why a larger flow can go out sooner.**
  - A larger flow gives a larger r_ss and a larger initial flame, hence more radiating volume early on. A model with a larger initial ignition kernel went out sooner (1.50 s vs 1.64 s). [N] Santa 2007
  - In [D] terms, the threshold ratio r_ext/r_ss falls as ṁ^−½.
  - In the drop tower, extinction time first rises with flow (burner loss) and then levels off (about 1 s measured, 1.65 s modelled). [DT/N]
- **Optically thin models overpredict loss.**
  - Re‑absorption is significant. [N] Tse 2001; Santa 2007
  - An optically thin model "radically decreased combustion duration". [N] Waddell, Dietrich & Nayagam 2024
- **Oscillatory partial extinction precedes total extinction.** The oscillations grow until the flame goes out. [ISS] JASMAC 2021; CFI‑G Final Report.
  - For Lewis number > 1, numerical oscillations always end in extinction (no limit cycle), but can last more than 450 s. [N] Christiansen, Tse & Law 2003 (abstract)
- **Burning Rate Emulator (BRE) burner, a different burner:** flames self‑extinguish below about 25 % O2; larger flames go out within about 1.5 min; methane within 1 min, while ethylene burns for minutes. [ISS] JASMAC 2021.

## 3. Kinetic (small‑flame, low‑Damköhler) extinction and the regime map

- **Two limits.** Radiation extinguishes at high flow; short residence time extinguishes at low flow. Together they bound a window of flows where steady burning is possible.
  - The controls are a reduced mass flow (injection relative to diffusion) and a Damköhler number set by burner area, feed concentration and pressure.
  - Whether a steady flame exists depends on reactant concentrations and Lewis numbers. Stronger radiation or a smaller Damköhler number shrinks the window to nothing: this is the "island".
  - [T] Mills & Matalon 1998 (abstract)
- **Low‑flow failure mode:** steady flames can sit close to the sphere even at low injection velocity, and extinction comes from reactant leakage. [T] Mills & Matalon 1997 (abstract)
- **ISS observations (s‑Flame):**
  - Both modes appear: radiative at high flow, kinetic at low flow, because a small flame sits where velocities are highest.
  - Helium dilution shifts the limits and blurs the kinetic mode.
  - [ISS] JASMAC 2021
- **Weakest flames:** hydrogen micro‑flames at quenching release 0.46 W in air and 0.25 W in O2; leakage is high near the limit. [N/exp] Lecoustre thesis 2009 (abstract)
- **Your "weak flame" class (T_ad 1600–1700 K) [D].**
  - These sit at the classical limiting adiabatic temperature of about 1500–1600 K.
  - Peak temperatures are well below T_ad even early: equal‑T_ad flames had very different peaks, mainly from gas radiation. [DT/N] Sunderland 2003
  - So the ignition kernel never reaches a self‑sustaining structure. Expect death within the ignition transient (1–2 s), independent of the radiative clock.

## 4. Fuel effects

- **Fuel matters little for transient radiative extinction.** Propane‑normal and ethylene‑inverse flames behaved alike: fuel and configuration are "of secondary importance", because the outer diffusion layer dominates. [N/DT] Santa 2007
- **Soot.**
  - Soot appears at low Z_st; flames are soot‑free at high Z_st whatever the convection direction. Convection toward the oxidizer suppresses soot inception. [DT] Sunderland 2003
  - Soot formed early is trapped by thermophoresis and fades as the flame cools. [DT/ISS] Santa 2007; CFI‑G Final Report
  - Soot adds early radiation (methane < ethylene < acetylene). [DT] Berhan et al. 2004
- **Cool flames:** only n‑butane among ethane, propane and n‑butane, consistent with higher alkanes favouring low‑temperature chemistry. [ISS] Kim et al. 2023.
- **Butane vs ethane in enriched O2: hypotheses, not shown in the sources [D].**
  - Heavier fuel has lower D_F (fuel Lewis number roughly 1.4 for ethane, 1.9–2 for n‑butane; general literature values). That gives a larger r_ss and lower actual flame temperature at the same mass flow and Z_st.
  - Soot loading is higher.
  - Warm/low‑temperature chemistry could act in either direction.
  - Mass‑based Z_st and ΔH_c per gram barely differ between the three alkanes (s = 3.73, 3.64, 3.58 g O2/g fuel).
- **Warm‑flame chemistry (ethylene, 1 atm)** emerges near extinction and delays it, but is not exothermic enough to prevent it. [N] Waddell et al. CTM 2024.

## 5. Pressure

- **Extinction temperature:** ethylene inverse flames at 3 atm went out about 50–60 K lower than at 1 atm; 0.5 atm was similar to 1 atm. Peak temperature also falls with pressure. [ISS] Waddell thesis, ch. 4.
- **At 50 atm (model):** warm chemistry extended burning by more than 500 s, then a cool flame took over. [N] Waddell thesis.
- **Radiance:** total radiance does not depend on pressure in droplet experiments from 0.5–5 atm. H2O's share rose from 30 % to 55 % (measured) and 45 % to 65 % (model), because the CO2 4.3 µm band saturates. CO is never more than 15 %. [ISS droplets/N] Waddell et al. 2024; 2026.
- **Net effect on lifetime [D].**
  - r_ss does not depend on pressure.
  - D ∝ 1/p slows growth, which lengthens life.
  - A lower T_cr lengthens life.
  - Higher emitter density shrinks r_ext (Nayagam's κ_P ∝ p gives p^−1/3), which shortens life. Re‑absorption partly offsets this.
  - The sign must be fitted. Your observed shortening with pressure is consistent with the r_ext effect winning.
- **Cool flames:** none below 2 bar for butane. [ISS] Kim et al. 2023.

## 6. Inverse flames

- **Geometry:** the flame sits in the fuel ambient. Z_st is high (about 0.5–0.75 for the conditions used), and soot forms on the outside, fuel side. [DT/ISS] Sunderland 2003; Waddell thesis.
- **Radius:** r_f = ṁ/(4πρD ln(1/Z_st)). A leaner ambient means higher Z_st and a smaller ln term, so a **larger** flame that is also lower in T_ad. [T]
- **Critical O2 flux:** 0.68 g m⁻² s⁻¹ (ethylene). [ISS/N] Irace 2023
- **Threshold O2 flow [D]** for burner X_O2 = 0.85 into propane/N2, with Z_st and ṁ\*_O2 from section 2:

  | Ambient propane | Z_st | ṁ\*_O2 |
  |---|---|---|
  | 7 % | ≈ 0.69 | ≈ 2.1 mg/s |
  | 14 % | ≈ 0.54 | ≈ 5.9 mg/s |
  | 17 % | ≈ 0.49 | ≈ 7.7 mg/s |

  Against CFI‑G inverse O2 flows of 0.9–9 mg/s [ISS, Kim Table 1 ranges], this reproduces your step from "seconds" to "never" with ambient fuel fraction.
- **Supply‑tube effect.** Heat loss to the supply tube opens a quench hole; the hole oscillates and grows until extinction. This can be abrupt or last tens of seconds, and some flames stay quasi‑steady despite the hole. Dim (leaner) flames oscillate more and extinguish; brighter, richer flames oscillate weakly and survive (butane, about 14 % vs about 17.5 % ambient). [ISS] CFI‑G Final Report weekly log.
- **No inverse cool flames:** residence times were short and burners stayed below 500 K. [ISS] Kim 2023
- **Immediate "flash" failure [D, mechanism].** Flow starts before ignition, so a premixed pocket forms. Ignition gives a brief premixed flame that must hand over to a diffusion flame. [DT] Santa 2007. In a lean ambient with low T_ad, that hand‑over can fail.

## 7. Cool flames (CFI‑G)

- **Window.**
  - Only normal n‑butane flames, at 2 bar, X_O2 ≈ 0.39–0.40, X_HC = 0.3, 0.75–1.5 mg/s, T_ad 2650 K, Z_st = 0.20.
  - Not below X_O2 ≈ 0.4, not at other fuel fractions, and not with the burner below **515 K**.
  - Hot flames were about 2.6× the cool‑flame radius (model 3.2). Cool flames sit at Z ≈ 0.53, fitted T_cf ≈ 869 K.
  - [ISS] Kim et al. 2023
- *(An aggregate count over the whole CFI-G campaign was reported here and is withheld: it overlaps the protected partitions.)*
- **Sequence.**
  - Hot radiative extinction, then a dark gap of 0.7–1.1 s (about 1 s), then a quasi‑steady cool flame lasting until the flow stops.
  - Hot‑flame reignition occurred only with a catalytic type‑B thermocouple (720–820 K) in the rich premixture.
  - Lower flow means a longer hot burn, hence a hotter burner at extinction, which favours a cool flame.
  - [ISS] Kim 2023
- **How the hot flame's end is detected.**
  - The radiometer drops to a **non‑zero plateau** if a cool flame follows, otherwise to zero.
  - PMT‑2 (230–700 nm) is used.
  - An intensified camera with a 390–490 nm (CH2O*) filter, frames averaged and brightened ×50.
  - Flow was continued at least 10 s after radiative extinction.
  - Early cool flames were not seen in real time and their flows were cut short.
  - [ISS] Kim 2023; Final Report

## 8. Burner heating

- **Limit:** tests were terminated at a burner temperature of **723 K (450 °C)**. [ISS] Kim 2023; Kim thesis.
- **Burner description:** 6.4 mm sintered 316 stainless steel, 10 µm media; disk porosity about 41 %, the sphere's probably higher. Type‑K thermocouple embedded about 1 mm deep. [ISS] Waddell thesis.
- **Lumped burner model** [N, Waddell eq. 2.9–2.10]:
  - ρ_b V_b (1−φ) c_b dT_b/dt = k_g A_b (∂T/∂r)_b − ṁ c_p (T_b − T_in) + Q_rad,net
  - Parameters: c_b = 500 J kg⁻¹ K⁻¹; solid 4.6 g cm⁻³; assumed porosity 79 %; ε = 0.6.
  - To match measurements at high flow, the gas‑heating term had to be dropped, so the gas is heated before or at the surface.
- **Measured heating (Flame Design ethylene, read from figures).**
  - Normal flames of about 0.7–2.5 mg/s brought the burner to roughly 550–650 K in 40–60 s; about 5 mg/s stayed near or below 400 K.
  - The model allows inverse flames to approach about 700–750 K.
  - Burner temperature matters for small flames and is negligible for large ones (−18 K flame temperature, +0.9 % radius for a small flame at 30 s).
  - [ISS/N] Waddell thesis
  - CFI‑G inverse burners stayed below 500 K. [ISS] Kim 2023
- **Order of magnitude [D].**
  - Burner heat capacity is about 0.07–0.2 J/K, plus the tube.
  - Conduction from a small flame (r_f ≈ 8 mm) is about 4πk(T_f−T_b)/(1/r_b−1/r_f) × blowing factor Pe/(e^Pe−1), roughly 2–3 W.
  - Burner radiation at 723 K is about 1.2 W.
  - So reaching 700 K takes tens of seconds to minutes. Heating falls as r_f grows (fixed ΔT, larger shell) and as blowing (Pe ∝ ṁ) rises.
  - The flow at which heating is fastest probably lies in the middle, because tiny flames release little heat.

## 9. Ignition and establishment

- **Drop tower:** flow started about 1–2 s before ignition; ignition about 30 ms into microgravity. A brief premixed flame transitions to a diffusion flame. The modelled initial temperature overshoot lasts under 0.1 s. [DT/N] Santa 2007
- **ACME hardware.**
  - Hot‑wire tips: Kanthal loop (0.25 mm) or coil (0.32 mm, used in CFI‑G), with programmable voltage and duration, including pre‑heat.
  - PMTs detect ignition and automatically switch off and retract the igniter.
  - Retraction from the burner's free area took 0.6 s (requirement 0.5 s).
  - [ISS] ACME Engineering Implementation 2022
- **Establishment.**
  - "Following the ignition transient" flames grow. [ISS] Kim 2023
  - Cool flames "typically require 20 s to establish". [proposal, Final Report]
  - Your 4–6 s establishment window is consistent with base/hole extinction at 1–2 s in strongly diluted drop‑tower flames [DT] and with the ~r²/D clock.
- **Failures:**
  - Igniter tips failed during CFI‑G and were replaced. [ISS] Final Report
  - Unsustained ignitions are recognised operationally (ACME Engineering doc).

## 10. Variability and operations

- **Chamber:** 83–90 L free volume at constant volume. Tests typically 0.5–2 min, run from scripts with a manual mode (step/ramp changes). [ISS] ACME Engineering 2022
- **Valve safety timer:** 999 s on the limiting reactant, reset only by venting. In effect this caps total flow time per chamber fill. [ISS] Final Report
- **Extinction detection:** radiometer or PMT can end the flow automatically. [ISS] ACME Engineering
- **Composition drift within a fill [D].**
  - Normal flames: 2 mg/s propane for 100 s uses about 0.73 g O2, about 3 % of the O2 in 90 L of air at 1 atm, and adds CO2 and H2O, which are extra radiators. A whole fill can consume tens of percent of the O2 at 1 atm.
  - Inverse flames: ambient fuel depletion is under 1 % per test.
- **Other sources of scatter:**
  - Burner start temperature carried over from the previous test (it changes small‑flame temperature).
  - Supply‑tube quenching and downward offset; possible leakage at the tube joint.
  - Ignition‑kernel size (a larger kernel means earlier extinction [N]).
  - MFC accuracy of ±5 %.
  - Residual gas motion after filling (crew procedure waits for equilibrium).
  - g‑jitter (not evaluated).
  - [ISS] Waddell thesis; Kim 2023; Final Report
- **Sensitivity near the threshold [D].** Near ṁ\*, life diverges as (1 − √(ṁ\*/ṁ))^−2 (section 11). Because ṁ\* ∝ (ρD)², an uncertainty of ±5 % in flow and ±20 % in ρD makes outcomes near the threshold essentially random.

## 11. What a forecaster should do

Carry these prospectively known quantities:
- Z_st (as L = ln(1/(1−Z_st)) for normal flames, ln(1/Z_st) for inverse)
- burner‑side reactant mass flow ṁ_R = Y_R·ṁ
- T_ad
- pressure
- fuel
- position within the fill

A natural reduced flow is **Φ = ṁ_R/ṁ\***, the flow divided by the threshold flow from section 2.

---

## Laws a hand‑written model should contain (most to least established)

1. **Critical temperature** T_cr ≈ 1130 K at 1 bar (ethylene; ±23 K), about 50–60 K lower at 3 atm. [ISS/N]
2. **Flame radius** r_ss = ṁ/(4πρD·L), with ρD ≈ 0.0334 g m⁻¹ s⁻¹ (R² = 0.92) and no pressure dependence. [ISS‑correlation/T]
3. **Radiative loss and heat release both scale with flame area.** Extinction happens at a fixed burner‑side reactant flux: 0.2 g m⁻² s⁻¹ fuel (normal), 0.68 g m⁻² s⁻¹ O2 (inverse), both equal to about 9.4 kW m⁻² of heat release [D]. So r_ext = √(ṁ_R/(4π m″_cr)). [ISS/N; alkane transfer is D]
4. **Steady flames exist when r_ss < r_ext and steady state comes quickly.** This gives ṁ\* = 4π(ρD)²L²Y_R²/m″_cr [D]. Life lengthens as flow falls toward ṁ\*, as Z_st rises (more O2, richer inverse ambient) and as T_ad rises.
5. **Time to self‑extinction [D].**
   - t_ext ≈ t₀ + r_ext² / (4 D_eff [erfc⁻¹(√(1/Φ))]²) for Φ > 1. Near the threshold this becomes ≈ r_ext²/(π D_eff (1 − Φ^−½)²).
   - With D_eff ≈ ρD/ρ_∞ (about 2.8×10⁻⁵ m²/s at 1 atm, ∝ 1/p), this gives tens of seconds for Φ ≈ 8. That matches the ISS ethylene burn times of 28–336 s (abstract, Irace 2021).
   - D_eff and m″_cr should be fitted. Use log‑normal scatter with σ ≈ 0.3–0.5, plus an oscillatory "tail" of a few to tens of seconds.
   - For Φ ≤ 1, set a low residual hazard (oscillation via supply‑tube quenching, larger at low T_ad).
6. **Early death** within about 2 s: a logistic in T_ad − 1550 K (width about 50–100 K) and in a low reduced flow (kinetic limit). [T/N, magnitude D]
7. **Burner clock:** the lumped ODE of section 8, ending at 723 K. Fastest at small to intermediate flows and small r_f; scale it by the burner start temperature.
8. **Pressure:** the sign of the effect on life must be fitted. Ingredients: T_cr −25 K/atm, D ∝ 1/p, r_ext ∝ p^−1/3 if optically thin (re‑absorption weakens this).
9. **Cool flame after hot extinction:** n‑butane, 2 bar, X_O2 ≥ 0.4, burner ≥ 515 K, delay about 1 s; it then persists until the flow stops.

## Open or contested

- The exact definition of m″_cr and whether it carries over to alkanes and other pressures (only ethylene, 1 bar is published).
- Whether radiation‑induced growth or the diffusion approach dominates the growth law. The erfc solution [D] ignores convection near the burner and the drop in D as the flame cools.
- Pressure: total radiance looks pressure‑independent (re‑absorption), yet observed lives shorten with pressure.
- Lewis number and soot as reasons butane is shorter‑lived than ethane: plausible, not demonstrated.
- The kinetic limit on a 6.4 mm sphere for ACME alkanes: s‑Flame reports it qualitatively; no published critical numbers.
- Drop‑tower propane results (extinction time falling with flow) contradict the ethylene trend; Santa et al. left this open.
- Supply‑tube quench‑hole dynamics, which are outside every 1‑D model.

## References

1. Santa, Chao, Sunderland, Urban, Stocker, Axelbaum, *Combust. Flame* 151 (2007) 665–675. https://www.grace.umd.edu/~pbs/2007%20Santa%20et%20al.%20CNF.pdf
2. Kim, Waddell, Sunderland, Nayagam, Stocker, Dietrich, Ju, Williams, Irace, Axelbaum, "Spherical gas‑fueled cool diffusion flames", *PCI* 39 (2023) 1647–1656. https://par.nsf.gov/servlets/purl/10426596
3. Irace, Waddell, Constales, Sunderland, Axelbaum, "Critical temperature and reactant mass flux…", *PCI* 39 (2023) 1905–1913, doi:10.1016/j.proci.2022.07.043. https://profiles.wustl.edu/en/publications/critical-temperature-and-reactant-mass-flux-for-radiative-extinct/
4. Irace, Waddell, Constales, Kim, Yablonsky, Sunderland, Axelbaum, "On the existence of steady‑state gaseous microgravity spherical diffusion flames…", *PCI* 39 (2023) (abstract via https://www.bohrium.com/scholar/i11379286/Minhyeng_Kim)
5. Irace, Lee, Waddell, Tan, Stocker, Sunderland, Axelbaum, *Combust. Flame* 229 (2021) 111373, doi:10.1016/j.combustflame.2021.02.019. https://www.aminer.org/pub/609a6ccbe4510cd7c88dc43d
6. Waddell, PhD thesis, UMD 2023, "Near‑limit spherical diffusion flames and cool diffusion flames". https://drum.lib.umd.edu/items/1d83d9b4-fe61-441b-ba38-df8b7a815e0f
7. Kim, MS thesis, UMD 2022, "Spherical gas‑fueled cool diffusion flames". https://drum.lib.umd.edu/handle/1903/29540
8. Waddell, Irace, Yablonsky, Constales, Sunderland, Axelbaum, *Combust. Theory Model.* (2024), doi:10.1080/13647830.2024.2326830
9. Waddell, Dietrich, Nayagam, CSSCI Spring Meeting 2024; and "Effect of pressure on radiative losses in hot, warm, and cool…" (2026 preprint)
10. Nayagam, Dietrich, Williams, *Combust. Flame* 205 (2019) 368–370. https://scinapse.io/papers/2943308926
11. Sunderland, Axelbaum, Urban, Chao, Liu, *Combust. Flame* 132 (2003) 25–33. https://www.grace.umd.edu/~pbs/2003%20Sunderland%20et%20al.%20CNF.pdf
12. Tse, Zhu, Sung, Ju, Law, *Combust. Flame* 125 (2001) 1265–1278. https://collaborate.princeton.edu/en/publications/microgravity-burner-generated-spherical-diffusion-flames-experime/
13. Mills & Matalon, *Combust. Sci. Technol.* 129 (1997) 295–319; *PCI* 27 (1998) 2535–2541. https://experts.illinois.edu/en/publications/extinction-of-spherical-diffusion-flames-in-the-presence-of-radia/
14. Christiansen, Tse, Law, *Combust. Flame* 134 (2003) 327–337. https://collaborate.princeton.edu/en/publications/a-computational-study-of-oscillatory-extinction-of-spherical-diff/
15. Chao, Law, T'ien, *PCI* 23 (1990) 523–531 (cited in Santa 2007)
16. Lecoustre, PhD thesis UMD 2009. https://drum.lib.umd.edu/handle/1903/9966
17. Berhan, Chernovsky, Atreya, Baum, Sacksteder, "Radiant extinction of gaseous diffusion flames" (NASA workshop paper; local copy only, no URL)
18. Stocker, "Recent research on flames of gaseous fuel aboard the ISS", JASMAC‑33 (2021). https://ntrs.nasa.gov/api/citations/20210022542/downloads/ACME%20JASMAC-33%2020211007.pdf
19. Stocker, "ACME Engineering Implementation" (NASA GRC, 19 May 2022), and Stocker, Sunderland, Axelbaum, Williams, "ACME/CFI‑G Final Report" (NASA GRC, 21 Nov 2022). Both NASA public reports read from local copies; NTRS URLs not recorded.

# Published science of concurrent flame spread over thin fuels in low-speed flow

> Literature review by a delegated research agent (claude-opus-5-5), 14 November 2026. Its rule: mechanisms,
> theory, numerical-model predictions and published correlations, with the kind of every number stated; no
> per-test results of the experiment series being modelled. The author used it to choose laws and typed every
> number himself. Sources are listed at the end; claims marked as the agent's own derivation are not published.

> For this topic the agent did not read the measured results of the BASS-II concurrent SIBAL tests in any
> source (they are what the front model is tested on); the sources it stopped at are listed.

# Concurrent flame spread over thin fuels in low-speed microgravity flow: published laws for a SIBAL front model

Each number is tagged by kind: **[T]** theory or scaling, **[M]** numerical model, **[E-xxx]** experiment, naming which one. The numbered sources are in the reference list at the end.

Much of the classic model work used Kimwipes tissue, which has an area density of about 2.0 mg/cm². That comes from the model's solid density of 0.263 g/cm³ and half-thickness of 0.0038 cm [2]. SIBAL is 18.05 mg/cm² in total, of which 13.54 mg/cm² is cotton [14]. Kimwipes numbers therefore carry over to SIBAL only through the thin-fuel scaling Vf ∝ 1/(ρτ).

## 1. Steady concurrent spread over thin fuels

- **A steady state exists.** For thin fuels the flame length, the pyrolysis length and the spread rate all become constant. The reason is that the fuel burns out at the base at the same rate as the tip advances [T/M: 1, 2, 19]. For thick fuels the end state is a flame that stops growing and stands still [M: 19].
- **The steady problem has one speed.** The model is solved in a frame moving with the flame, and the spread rate comes out as an eigenvalue. The relevant velocity is relative to the burnout point: U_rel = U − Vf [M: 2; T/E: 13].
- **Spread rate is close to linear in U and in O₂.** The Ferkul model for Kimwipes gives eq. (84):

  Vf = 1.12·X·U − 0.073·U + 0.194·X − 0.121, with Vf and U in cm/s, X the O₂ mole fraction, and U the relative velocity [M: 2].

  At X = 0.21 this reduces to Vf ≈ 0.162·U − 0.080. Flame length, pyrolysis length and preheat length are also roughly linear in both U and X [M: 2].
- **2-D model with gas radiation (Kimwipes, 15 % O₂):** spread rate and flame length are about linear in U; at 10 cm/s the flame is about 7 cm long; at 5 cm/s the spread rate is 0.503 cm/s [M: 6]. Spread rate rises linearly with free-stream velocity in concurrent flow; in opposed flow the dependence is not monotonic [M: 7].
- **Area density.** Drop-tower data for cellulose, 0.2 to 13 mg/cm², at 21 % O₂ and 30 cm/s fit Vf ∝ U_rel/ρs and L_f ∝ U_rel. Flame length does not depend directly on area density [E-drop tower, 13].
  - Theory behind it: the mean burning rate is ρs·Vf/L_f ∝ k_g(T_f − T_p)·√(U_rel/L_f)/L [T: 13].
  - Example: cheesecloth at 13.14 mg/cm² gave Vf = 0.20 cm/s and L_f = 7.8 cm [E-drop, 13].
  - Classic thin-fuel energy balance: Vf = q̇′_pre / (ρs·τ·c_s·(T_p − T₀)) [T: 18].
- **Burnout front and burnout time.** Define t_b = L_p/Vf.
  - For SIBAL the burnout-time histories of all the wide Saffire samples collapse onto one curve, with a steady t_b ≈ 22 s (air, about 1 atm) [E-Saffire, 14].
  - The SIBAL flame length equals its pyrolysis length [E-Saffire, 14]. The model gives L_f/L_p = 1.15–1.2 when the duct is taller than the optimum [M: 17].
  - Pyrolysis length is linearly proportional to spread rate at a given duct height [M: 17].
  - Preheat length is about 2.5·L_p, roughly 10 cm when L_p is 4 cm [E/analysis-Saffire, 14].
- **Saffire SIBAL steady values** (Vf, L_f = L_p):

  | Test | Sample | Duct height | Flow | O₂ | Vf | L_f |
  |---|---|---|---|---|---|---|
  | Saffire I | 94 × 40.6 cm | 51 cm | 20 cm/s | 21.7 % | 0.18 cm/s | 4 cm |
  | Saffire II | 29 × 5 cm | 51 cm | 20 cm/s | 22.1 % | 0.21 cm/s | 4 cm |
  | Saffire II | 29 × 5 cm | 51 cm | 25 cm/s | 22.1 % | 0.26 cm/s | 5 cm |
  | Saffire III | 94 × 40.6 cm | 51 cm | 25 cm/s | 21.2 % | 0.23 cm/s | 5 cm |
  | Saffire IV | 50 × 40.6 cm | 30.5 cm | 20 cm/s | 23.1 % | 0.36 cm/s | 8.5 cm |

  Halving the duct roughly doubled both Vf and L [E-Saffire, 14]. These give Vf/U ≈ 0.009–0.018 and L_f/U ≈ 0.2–0.43 s.
- **SIBAL in a 7.6 cm duct (Confined Combustion experiment, about 22 % O₂, 2.2 × 10 cm sample)** [E-Confined Combustion, 16]:
  - Below about 17 cm/s the flame reached a steady state: limiting length and constant spread rate. In one test the tip and base speeds differed by about 2.2 %.
  - At 17 cm/s and above the flame kept growing over the 10 cm sample. At 28 cm/s the tip moved about twice as fast as the base.
  - The authors say this boundary may be specific to the sample length.
  - Near the downstream end of the sample the tip accelerates and the two side flames merge (the no-slip condition ends there).
- **Pressure.**
  - Experiments give Vf ∝ X_O₂·√P for Kimwipes over 1–30 cm/s, 24–85 % O₂ and 3.6–14.7 psia. The correlation is linear in a combined parameter (y = 0.2498x + 0.4038, R² = 0.97) and falls below the line near the limit [E-drop tower, 11].
  - A 3-D model reports Vf ∝ P·U·X_O₂² [M: Li & Liao 2021, as reported in 14].

## 2. Low-speed quenching limit

- **Mechanism.** Radiative loss from the solid surface. Without it the model predicts no low-speed limit [M: 1, 2].
  - At low U the surface radiation becomes a large fraction of the heat release. The shorter flame also sends relatively more heat downstream instead of into the fuel.
  - The limit is a U-shaped boundary: a quench branch at low speed and a blowoff branch at high speed [M: 1, 2, 6].
  - Gas radiation further shortens the flames [M: 1, 5].
- **Ferkul–T'ien boundary (Kimwipes, surface radiation only, read from Fig. 1 of [1])** [M]:
  - Quench velocity: about 1.0 cm/s at 21 % O₂, 1.5 at 19 %, 1.8 at 17 %, 2.7 at 15 %, 4 at 14 %.
  - Minimum O₂ about 13.5 % at about 5 cm/s.
  - Interpolated: about 2.1–2.3 cm/s at 16.4 %, about 1.7 at 17.6 %, about 1.5 at 18.8 %.
- **With gas radiation (Kimwipes, 2-D):** quench at 2.1 cm/s for 15 % O₂; at 5 cm/s the low-oxygen limit is 11.7 % [M: 6]. A tunnel moves the boundary to lower velocities [M: 5].
- **How the flame shrinks near the limit** [M: 1, 2]:
  - It shortens and stays anchored at the burnout point.
  - Peak temperature falls slowly at first, then steeply. In Fig. 2 of [1] it goes from about 0.9 to about 0.5 of the adiabatic value.
  - Spread rate falls monotonically. Standoff distance first grows, then shrinks very close to the limit.
  - More fuel vapour escapes unburned.
  - Extinction comes from the flame cooling, not from it being pushed off.
- **Shrinking flames in the drop tower** [E-drop tower, 1]. At low O₂ and low U the flames shrank during the 5 s of microgravity; at higher O₂ and U they grew. Their final fate was unresolved.
- **Approach time.**
  - Concurrent steady state on Kimwipes needs about 16 s, from an expression based on pyrolysis length and spread rate [T/M: 5].
  - Near a geometric quench limit the 3-D model shows oscillations of flame length and temperature with a period of about 2 s (thermodiffusive), when the local flow is about 3–5 cm/s [M: 15].
  - For PMMA in the same duct, with stepped flows, the flame "quenched within 10 s" after a step to the lowest speed (1.6–2.5 cm/s). This is a thick fuel, so it gives the order of magnitude only [E-Confined Combustion PMMA, 18].
- **Effect of residual products in the inlet:** 0.1 % fuel + 1 % CO₂ + 1 % H₂O moved the 15 % O₂ quench point from 2.1 to 2.0 cm/s [M: 6].

## 3. Blowoff

- **Kimwipes model:** the blowoff branch sits at about 14 % O₂ near 10 cm/s and about 15 % near 16 cm/s [M: 1]. The flame cannot hold at the burnout point and is carried downstream [M: 2].
- **Higher speeds, normal gravity, horizontal tunnel:** spread rate rises up to about 1 m/s and levels off above it [E, cited in 2].
- **In this regime (16–21 % O₂, 0–55 cm/s):** blowoff is not expected. SIBAL at 28 cm/s and about 22 % O₂ kept growing without blowing off [E-Confined Combustion, 16].

## 4. Transient response

- **Growth after ignition.**
  - Wide Saffire SIBAL: burnout time rises nearly 1:1 with time after ignition and peaks at about 30–35 s, so the length overshoots, then decays. L_p is steady by about 100 s [E-Saffire, 14].
  - Mechanism of the overshoot: heat flux starts high while preheat length is short; later the preheat length is long but the flux is lower; their product peaks during growth [M: 19, quoted in 14].
  - Narrow samples (5 cm or less) grow smoothly to steady with no overshoot, helped by lateral expansion and lateral oxygen diffusion [E-Saffire, 14; M: Li et al. 2019 in 14].
  - 2.2 cm SIBAL at 6 cm/s: steady about 10 s after ignition [E/M: 17]. Elsewhere it reached a limiting length quickly once the igniter was switched off [E: 16].
  - Ultra-thin fuel at 30 cm/s: smooth, monotonic approach to steady in under 1 s [E-drop, 13].
- **Time scales to keep separate** (my inference from [13, 14, 17]):
  - Gas phase: τ_g ≈ L_f/U, under a second.
  - Pyrolysis zone: t_b = L_p/Vf ≈ 22 s for SIBAL.
  - Preheat zone: about 2.5·L_p/Vf ≈ 50 s.
  - The tip (gas phase) should respond within τ_g. The base and the pyrolysis zone respond on t_b.
- **Published step-change data.** No published thin-fuel step-response data were found outside the protected tests. PMMA in the same duct with stepped flows reached a new steady state at every step, holding each step at least 200 s [E: 18].
- **Camera artefact.** Visible flame length is a contour of the reaction rate, about 10⁻³–10⁻⁴ g/cm³/s. The high-temperature region (above 1200 K) is about twice as long as the visible flame [M/E: 17].
  - The cameras ran on auto-exposure and auto-gain [16, 17].
  - So a dimming flame loses apparent tip length even when the reaction zone has barely moved.

## 5. Sample width and 3-D effects

- **3-D steady model** [M: 8, 9, from the abstracts]:
  - At high O₂ or higher speed the flames are long, and heat loss at the downstream end and side walls dominates, so wider samples spread faster.
  - At low speed and low O₂ the flames are short and limited by oxygen supply. Oxygen diffusing in from the sides then dominates, narrower samples spread faster, and the trend of the limits reverses.
- **Most flammable width** at low flow is about 4 cm, not an infinitely wide sample [M: 6].
- **Narrow SIBAL in Saffire:** 5 cm samples spread faster than full-width ones. The 1.2 cm sample flame was all blue because of heat loss to the metal holder [E-Saffire, 14].
- **The flame front is centre-leading,** attributed to heat loss to the side holders [M: 17].
- **Upward (buoyant) flow, 1–4 cm widths:** wider samples gave faster and longer flames [E-KC-135, 21].
- **Honda–Ronney width law** (normal-gravity upward spread, via 14):
  - Below about 1 cm width, spread rate scales as width³ (stabilised by convection).
  - Above about 1 cm, it scales as √width (stabilised by radiation).

## 6. Confinement in a small duct

- **3-D model, 10 cm/s air, 2.2 cm SIBAL** [M: 15]:
  - Optimum duct height is 3–4 cm; the quench height is 0.5–0.8 cm.
  - Above the optimum, Vf ∝ 1/h: thermal expansion accelerates the flow ("flow channeling").
  - Below the optimum, Vf ∝ h: oxygen supply limits it.
  - The holder boundary layer can "lock" the flow over the sample and send it into side gaps.
- **Same model at 6 cm/s** [M: 17]:
  - Above the optimum, black walls: L_f = 18.2/H + 0.7 (cm, with H in cm). This gives about 3.1 cm at H = 7.6 cm.
  - Below the optimum: L_f = 3.2·H − 3.9.
  - At H = 7.6 cm the model's spread rate is within 3.7 % of the Confined Combustion measurement.
  - Wall reflectance from 0 to 1 at H = 3 cm raised Vf by 52 % and L_p by 13 %.
- **Oxygen-limited formula:** Vf = ρ_air·Y_O₂·H·U₀ / (r·ρs·τ), with r = 1.92 for PMMA [T: 16, 18].
- **Confined Combustion SIBAL, 6 cm/s** [E-Confined Combustion, 16]:
  - Going from the bare 7.6 cm duct to 4 cm between baffles multiplied Vf by about 1.62.
  - At D = 1 cm the flame never spread past the igniter.
  - Cold flow develops along the duct by less than 10 %.
  - Entrance length is about 0.02·Re_H, about 5 cm.
- **Core-flow acceleration estimate** (method in [14], eqs. 2–8):
  - Thermal-expansion volume: V_e = (L_f·w·2H_f)·(T_mean/T₀ − 1), with T_mean ≈ 1050 K.
  - Boundary-layer displacement: δ*/3 summed over the walls and the sample, iterated four times.
  - The local velocity at the flame base drops sharply over the first 10 cm of holder.

## 7. Material: SIBAL

- **Composition:** 75 % unbleached cotton, 25 % fibreglass; 18.05 mg/cm² in total (13.54 mg/cm² cotton); half-thickness area density 0.009 g/cm² [E: 14].
  - Another paper quotes about 24 mg/cm² [13].
- **After burnout:** the fibreglass matrix stays in place with no curling, so the burnout front is a char front over a surface that still has no-slip flow [16, 14]. Smouldering spots stay behind the flame base, which can confuse base tracking.
- **Properties:**
  - Pyrolysis onset about 400 °C, where the thermocouple slope changes [E: 14; 5].
  - Specific heat: fibreglass 0.7 J/g·K, cotton 1.26 J/g·K. Emissivity 0.8.
  - Latent heat L_v = 300 + (c_p,f − c_s)(T_s − 300) J/g.
  - First-order kinetics: A = 1.626 × 10¹¹ s⁻¹, E = 161.44 kJ/mol (from TGA, attributed to [Zhao 2017]; only the property values were taken, from [14]).
  - Two-step in-depth pyrolysis model [15, 17].
  - Heat of combustion of cotton 17.44 kJ/g, so a 10 × 2.2 cm sample releases about 5.24 kJ [16].
- **Surface heat flux:** near the pyrolysis front it reaches about the critical flux for igniting cellulose [14].
- **Spread rates of SIBAL in other experiments:** see the Saffire table in §1. Pure cotton at the same area density (26.9 % O₂, 76 kPa) spread at 0.34 cm/s; it burns away completely [E-Saffire V, 14].

## 8. Oxygen dependence, 16–21 %

- **Linear in X_O₂** away from the limit [M: 2; E: 11]. A quadratic form, Vf ∝ X², has also been proposed [M: in 14].
- **Kimwipes model, eq. (84) at U = 10 cm/s** [M]:
  - Vf = 1.54 cm/s at 21 %, 1.29 at 18.8 %, 1.16 at 17.6 %, 1.02 at 16.4 %.
  - Ratio 17.6 %/18.8 %: 0.90 (linear law) or 0.88 (quadratic law).
- **Near quench, dependence on O₂ steepens** [M: 1]:
  - Peak temperature falls fastest there.
  - Vf drops below the linear fit [E: 11; M: 1].
  - Separately, near-limit flames limited by oxygen supply become controlled by the supply rate [M: 8].

## 9. What a forecaster can and cannot predict

**Predictable (established or strongly supported):**
- The steady state and its time scale. With an L_p (or L_f) and Vf measured in the prefix, t_b = L_p/Vf; for SIBAL expect about 20 s.
- The direction and rough size of change across a flow step: Vf and L scale with the relative effective velocity (U_eff − Vf).
- Which end responds first: the tip within τ_g, the base on about t_b (my inference).
- That a step to below about 1–2.5 cm/s at 16–19 % O₂ falls in the quench region of the thin-fuel models.
- No blowoff at 0–55 cm/s.
- Tip acceleration as it nears the end of the sample.

**Not reliably predictable:**
- The absolute quench velocity for SIBAL on a narrow holder. Kimwipes model values need a material and holder shift, sign unknown.
- The exact exponent at very low U. Eq. (84) predicts about a 10-fold fall in Vf from 11 to 1.5 cm/s, against your measured 2.5-fold. So the velocity at the flame does not follow the fan velocity at low U.
- The time to extinction after a step.
- How much of an apparent tip retreat is optical rather than real.
- Ignition success near the limit.

## Laws a hand-written front model should contain

Ranked from best established to least.

1. **Kinematic coupling of tip and base** [M: 2, 19; E: 14, 16]. Tip position x_t and base position x_b satisfy x_t − x_b = L(t), and at steady state dx_t/dt = dx_b/dt = Vf.
2. **Burnout-time closure** [E-Saffire: 14; M: 17]. Vf = L_p/t_b, with L_f ≈ L_p (factor 1.0–1.2) and t_b ≈ 22 s for SIBAL in air. Fit t_b from the prefix; widen its prior for narrow samples and low O₂.
3. **Flow law** [M: 2, 6; E: 13]. Vf = a(X)·(U_eff − Vf), so Vf = a·U_eff/(1 + a), and L_f = τ_g(X)·U_eff. For SIBAL in air, a ≈ 0.01–0.02 and τ_g ≈ 0.2–0.4 s. Use U_eff = c_duct·U_fan + U₀, with c_duct > 1 because of thermal expansion in the 7.6 cm duct [M: 15, 17], and a floor U₀ as a free parameter. That floor is my hypothesis, motivated by your 2.5-fold measurement.
4. **Oxygen law** [M: 2; E: 11]. a(X) ∝ X^n with n between 1 and 2, plus a near-limit falloff as U → U_q(X).
5. **Quench boundary** [M: 1, 6]. U_q(X) follows the Kimwipes shape: about 1.0, 1.5, 1.8 and 2.7 cm/s at 21, 19, 17 and 15 % O₂. Treat the shift for SIBAL on a narrow holder as an uncertain additive offset. If U_eff < U_q after a step, the flame decays: L shrinks, the base keeps advancing within t_b, then the flame goes out within about one burnout time (about 10 s for PMMA [18]).
6. **Two-time-scale response to a step** (my inference). L relaxes toward its new steady value with a fast part (τ_g, under 1 s) and a slow part (around t_b). The base speed relaxes on t_b. Add a term for optical dimming.
7. **End and wall effects.** The tip accelerates near the end of the sample [16]. Narrow-width corrections [8, 9, 14]: faster at low U through side oxygen, more holder loss on a 1.2 cm sample.

## Open or contested

- Whether the O₂ exponent is 1 or 2.
- Whether Vf ∝ U holds below about 3 cm/s. Diffusion-limited regimes may set a velocity floor, since oxygen diffusion velocity is about 1 cm/s [13].
- Overshoot in narrow samples. The 2.2 cm model shows a quick monotonic approach.
- How confinement and holder boundary layers change the velocity at the flame base [14, 15].
- Kimwipes-to-SIBAL transfer of the quench limit (radiating fibreglass matrix, holder loss).
- No published thin-fuel step-response time constants were found outside the protected tests.

## Sources deliberately not read for results (BASS-II rule)

- **Zhao, Liao, Johnston, T'ien, Ferkul, Olson,** Proc. Combust. Inst. 36 (2017) 2971–2978. Not opened. Only the pyrolysis constants were taken, second-hand via [14].
- **2013 US National Combustion Meeting paper 2G03-070HE-0218** (sutherland.che.utah.edu). Not opened.
- **Olson et al., ICES-2019,** "Flammability limits from BASS-II testing". Not opened.
- **Ferkul et al., ASGSR 2013 (BASS).** Not opened.
- **Link et al., ICES-2015 (BASS-II sensors).** Not opened.
- **[14]:** I skipped its BASS table rows, Fig. 6 and the BASS paragraphs.
- **[16]:** I skipped the paragraph comparing to "previous BASS" and the BASS sentences in the introduction.
- **[15]:** I did not use the experimental points in Fig. 4.
- **[17]:** Its model validation against BASS was not read.

Two disclosures. Search-engine summaries twice surfaced one-line qualitative BASS-type outcomes, one of them naming a quench flow speed. While I was locating sections of [14], two BASS table rows also appeared in my search output. None of this was used in this report.

## References

1. T'ien, Sacksteder, Ferkul, Grayson, "Combustion of solid fuel in very low speed oxygen streams," NASA CP (1993). https://ntrs.nasa.gov/citations/19930011018
2. Ferkul, "A Model of Concurrent Flow Flame Spread Over a Thin Solid Fuel," NASA CR-191111 (1993). This is the thesis behind Ferkul & T'ien, Combust. Sci. Technol. 99 (1994) 345–370. https://ntrs.nasa.gov/citations/19930015757
3. Grayson, Sacksteder, Ferkul, T'ien, Microgravity Sci. Technol. 7 (1994) 187–195. Record only: https://ntrs.nasa.gov/citations/19950029962
4. Pettegrew, NASA CR-198537 (1996). https://ntrs.nasa.gov/citations/19970003005
5. T'ien et al., "Solid Inflammability Boundary at Low Speed (SIBAL)" (1997). https://ntrs.nasa.gov/citations/19970020611
6. Shih & T'ien, PICAST4 (2001). https://ntrs.nasa.gov/api/citations/20020027088/downloads/20020027088.pdf
7. T'ien et al., SIBAL overview, NASA/CP-2003-212376. https://ntrs.nasa.gov/api/citations/20040053507/downloads/20040053507.pdf
8. Shih & T'ien, Proc. Combust. Inst. 28 (2000) 2777–2784. Abstract only: https://pure.lib.cgu.edu.tw/en/publications/modeling-concurrent-flame-spread-over-a-thin-solid-in-a-low-speed-3/
9. Shih & T'ien, Combust. Theory Model. 7 (2003) 677–704, doi:10.1088/1364-7830/7/4/005. Abstract snippets only.
10. (Not used.)
11. Olson, Ruff, Miller, NASA/TM-2008-215260. https://ntrs.nasa.gov/citations/20080034883
12. Olson & Miller, Proc. Combust. Inst. 32 (2009) 2445–2452. Used via [11] and [14].
13. Vetturini, Cui, Liao, Olson, Ferkul, Fire Technol. 56 (2020) 91–111. https://par.nsf.gov/biblio/10121773
14. Olson et al., "The effect of duct size, sample size, and fuel composition on concurrent flame spread over large cellulose samples in microgravity," accepted manuscript. https://discovery.ucl.ac.uk/id/eprint/10164340/
15. Li, Liao, Ferkul, J. Heat Transfer 142 (2020) 111301. https://par.nsf.gov/biblio/10206129
16. Li, Liao, Ferkul, Johnston, Bunnell, Combust. Flame 227 (2021) 39–51. https://par.nsf.gov/servlets/purl/10313745
17. Li & Liao, Combust. Flame 235 (2022) 111714. https://par.nsf.gov/servlets/purl/10313743
18. Li et al., Combust. Flame 234 (2021) 111637 (PMMA). https://par.nsf.gov/servlets/purl/10313744
19. Tseng & T'ien, J. Heat Transfer 132 (2010) 091201, doi:10.1115/1.4001645. Abstract snippets, plus the quote in [14].
20. Li, Liao, Ferkul, Central States Section meeting paper (2018), abstract. https://par.nsf.gov/biblio/10082431
21. Feier, Shih, Sacksteder, T'ien, AIAA 2001-0466. https://ntrs.nasa.gov/citations/20010059376

Not accessible as full text, used only where cited second-hand: the T'ien et al. 2001 chapter, the Fernandez-Pello and Bhattacharjee reviews, Kleinhenz et al. 2008, Zhao & T'ien 2015, Jiang & T'ien, Honda & Ronney 2000, and Li et al. Proc. Combust. Inst. 37 and 38.

Some downloads briefly landed in . by mistake. I moved them to the scratchpad, and `git status` there is clean.

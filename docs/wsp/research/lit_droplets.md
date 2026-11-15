# Published science of methanol and n-heptane droplet combustion in microgravity

> Literature review by a delegated research agent (claude-opus-5-5), 14 November 2026. Its rule: mechanisms,
> theory, numerical-model predictions and published correlations, with the kind of every number stated; no
> per-test results of the experiment series being modelled. The author used it to choose laws and typed every
> number himself. Sources are listed at the end; claims marked as the agent's own derivation are not published.

I've finished the report below. I read the full text of 13 of the sources, mostly NASA NTRS PDFs plus the Williams & Nayagam 2019 paper, the Microgravity Sci. Technol. 2014 overview and the FLEX technical publication. The rest I saw only as abstracts or citations, and those are marked. Every number carries a tag for what kind it is:

- **[T]** theory
- **[N]** numerical model
- **[F]** correlation or trend from the FLEX campaign
- **[O]** other experiments (drop towers, earlier flights)
- **[E]** my own estimate from standard properties, not published

I have left out test identifiers and per-test tables. Where FLEX numbers appear, they are given as ratios or trends.

---

## 1. Methanol burning rate

- **Quasi-steady law** [T]: K = (8λ_g / ρ_l c_pg) · ln(1+B), with B = [q·f·Y_O∞ + c_pg(T∞ − T_b)] / L. Here q is the heat of combustion per kg of fuel, f the stoichiometric fuel/oxygen mass ratio (0.667 for methanol) and Y_O∞ the ambient oxygen mass fraction. FLEX evaluates properties by the Law–Williams recipe [S1, S19]. That recipe is usually quoted as λ = 0.4λ_F + 0.6λ_N2 and c_p = c_p,F, both at T̄ = (T_b + T_f)/2; I did not re-read the 1972 paper, so treat the quoted form as provisional.
- **Numbers** [E]: q ≈ 19.9 MJ/kg, L ≈ 1.10 MJ/kg and T_b = 338 K at 1 atm give B ≈ 12.1·Y_O∞ − 0.05. That is B ≈ 2.8 in air and ln(1+B) ≈ 1.32. Going from 21% to 15% O2 (in N2) lowers ln(1+B), and hence the predicted K, by about 17%.
- **Measured K** [F]: about 0.2–0.8 mm²/s across FLEX (oxygen 12–30%, CO2 or He 0–70%, 0.7 and 1 atm). It is about 0.5 mm²/s in air for D0 ≈ 2.75 mm [S1, S2].
- **Diluent effect** [F]: at 21% O2, replacing 30% of the N2 with CO2 lowers K by about 15%. Replacing it with He raises K by about 35% (that example was at 0.7 atm), because helium's conductivity is about 10 times that of N2 [S2, S3]. More CO2 at the expense of N2 lowers K slightly [S9].
- **Deviation at low oxygen** [F]: at low O2 the measured K falls well below the quasi-steady prediction [S9]. This deficit is itself an informative feature (see section 10).
- **Water absorption and droplet size** [F, T]: small droplets burn faster than quasi-steady theory, because condensing water releases heat at the surface. Large droplets lose that gain once the absorbed water starts to re-evaporate [S1]. Absorption runs at a roughly constant, gas-diffusion-controlled rate. Diffusion reverses when the liquid is about 30% water by mass [S1, S16].
- **Shape of the D² curve** [F]: nearly linear through most of the burn. The instantaneous −d(D²)/dt drops slightly just before diffusive extinction [S1, S2]. FLEX fits use the middle 80% of the burn, or t = 0.1–0.9 of the burn time τ_b, to exclude heat-up and the end [S1, S2].
- **Initial-diameter dependence** [O]: Shuttle data for 3–5 mm methanol show K falling as D0 grows, attributed to radiative loss [S15]. Numerically, K without radiation rises only mildly and linearly with D0, from transient heating [N, S10].
- **Pressure**: quasi-steady theory gives no explicit pressure dependence, since λ and ρD do not depend on pressure. Only T_b and L move [T]. FLEX at 0.7 versus 1 atm shows no separate pressure trend once diluent is accounted for [S1].
- **Ar and Xe**: no methanol data found. A numerical study cited in the search results says Xe instead of N2 lowers the burning rate, raises the extinction diameter and lengthens the burn [N, abstract only]. For cool flames, Xe lowers K strongly [S6].

## 2. Methanol extinction

- **Diffusive extinction mechanism** [T]: in the simplified picture, extinction happens at about the diameter where water accumulation ends (about 30% water). The adiabatic flame temperature of the methanol–water mixture then falls below the 1200–1700 K range where the flame chemistry fails [S1, S16].
- **Asymptotic models** [T, N]: activation-energy asymptotics (AEA), rate-ratio asymptotics and detailed numerics all under-predict D_e in air. Raising c_p by 60%, so that T_ad ≈ 1700 K, brings AEA close to the data [S1].
- **D_e grows with D0** [F, O]: diffusive D_e rises roughly linearly with D0, more or less regardless of diluent, consistent with USML-2 [S1, S2]. The explanation is that larger droplets burn longer and absorb more water, which lowers the effective flame temperature. The slope of D_e against D0 therefore exceeds the AEA prediction [S1].
- **Contrast with heptane** [T]: for heptane, theory makes the diffusive D_e independent of D0 [S2].
- **Damköhler correlation** [T]: in CO2-enriched air at 1 atm, D_e is well correlated by a critical Damköhler number: residence time D²/K against a chemical time, including fibre conduction [S16].
- **Oxygen effect** [F]: lowering O2 raises the diffusive D_e and lowers the radiative-extinction diameter. The two branches meet at the limiting oxygen index (LOI) [S1, S2].
- **LOI** [F]:
  - about 0.12 in O2/N2 (overall range 12–13% by volume);
  - about 0.14 with CO2 (O2/N2/CO2 = 0.14/0.56/0.30);
  - about the same with He [S1, S2];
  - drop towers give 0.15 ± 0.02 for both He- and CO2-diluted air [O, S2, S20].
- **Radiative extinction**:
  - Critical D0 in air at 1 atm is about 4 mm [O/N, S15]. Above about 4.5 mm, D_e climbs fast and burn time drops sharply [N, S15].
  - Radiative extinction happens with D_e very close to D0. The combustion is transient and D_e probably depends on the ignition energy [F, S9]. In the D_e–D0 plot, large droplets at low O2 sit near the 45° line [S1].
  - The transition is bimodal, not gradual. At intermediate O2, points jump from the small-D_e group to the D_e ≈ D0 group as D0 crosses a critical value. That critical value is larger at higher O2 [F, S9].
  - CO2 promotes radiative extinction (infrared emission). CO2 has little effect on diffusive extinction [F, S9].
  - Below a certain oxygen fraction, every methanol droplet extinguished radiatively. That threshold was lost from the corrupted text of [S9].
- **Scaling** [T]: radiative loss over heat release grows as D². Heat release goes as D (surface × gradient), radiation as flame volume, about D³ [S9]. With pressure: emission per volume goes roughly as p at fixed composition and heat release is about p⁰. So the loss fraction goes as p·D²·FSR³, giving D_crit ∝ p^(−1/2)·FSR^(−3/2) [E]. That form follows the stated assumptions of [S14], whose fitted law I could not read.
- **Flame-standoff ratio (FSR)** [F]: about 5 during quasi-steady burning in all three diluents, falling as O2 rises. Measured FSR in He mixtures is consistently above the theory [S1, S2]. The flame typically grows for 2 s or less, then shrinks until diffusive extinction. Large low-O2 droplets never reach a quasi-steady FSR [S2].
- **Precursors in the diameter record** [F]: diffusive extinction is abrupt: flame present in one frame, gone the next [S2]. For methanol, the only diameter-history precursors are:
  - a K below the quasi-steady value at low O2 [S9];
  - a slight slope drop just before diffusive extinction [S1, S2].
  
  Pre-extinction flame oscillations near 1 Hz occur, but only in the flame, and are rarer for methanol than heptane [S2].

## 3. n-Heptane hot flame

- **Transfer number** [E]: q = 44.6 MJ/kg, f = 0.284, L = 0.317 MJ/kg and T_b = 371 K give B ≈ 40·Y_O∞ − 0.46, which is about 8.9 in air.
- **Measured K** [F]: in air plus 5% CO2 at 1 atm, about 0.5 mm²/s, slightly lower for D0 ≈ 3 mm than for 2 mm [S2]. Hot-flame K for other n-alkanes falls with O2. For decane at 0.5 atm it is about 0.3–0.4 mm²/s at 14–21% O2 [S3].
- **K against D0** [O, N]: in drop towers, K falls as D0 grows (0.8–1.8 mm) because soot and radiation increase [S10]. For larger, fibre-supported 2.6–2.9 mm droplets, K was 0.58–0.68 mm²/s, higher than that trend predicts [O, S10].
- **Soot load** [O, S10]: peak soot volume fraction is about 35–38 ppm at 2.6–2.9 mm, about 40% below the value at 1.8 mm. Luminosity peaks just after ignition and becomes undetectable by about 1.5 s.
- **Radiation in the model** [N, S10]: non-luminous gas radiation alone lowers peak flame temperature by more than 600 K at D0 = 2.9 mm and lowers K from 0.71 to 0.60 mm²/s.
- **Soot and the diameter measurement**: a soot shell can hide the droplet edge [S2]. Bright soot early in the burn distorted sizing until the shutter was shortened [S8].
- **Radiative extinction** [F]:
  - Common for heptane; diffusive extinction is rarely seen because droplets disrupt first [S2].
  - In the standard atmosphere, flames often disappear for D0 above about 2 mm [O/F, S13].
  - LOI: about 0.15 in O2/N2, about 0.17 with CO2 added, about 0.15 with He [S2].
  - The extinction is slow: it starts on the side opposite the drift and the flame retreats over 5–10 frames, sometimes oscillating near 1 Hz [S2].
  - Lower O2 gives earlier radiative extinction. 25% O2 at 0.5 atm avoids it; the flame then dies small, by micro-explosion or diffusive extinction [F, S3].
  - Peak radiative loss comes well before the extinction itself [F, S3].
- **FSR** [F]: grows continuously and nearly linearly through the hot-flame life, more slowly for larger droplets [S2]. Extinction is therefore governed by time and flame growth, not by droplet diameter alone.

## 4. n-Heptane cool flames

- **When a cool flame follows** [F]: only after radiative extinction of the hot flame. Smaller droplets burn to completion or disrupt [S2, S3]. Within the same size range a droplet can burn either way, depending on initial conditions [S2].
- **Numerical predictions** [N, S12]: CO2 and He let low-temperature burning start at smaller D0 than with N2, and small amounts are enough. Radiative loss is what triggers the cool flame. The second-stage peak gas temperature is about 700 K [N, S2].
- **At 75% He, 1 atm** [F, S3]: the hot flame dies within a fraction of a second, a brief cool flame follows, then evaporation. It is fully transient.
- **Cool-flame K** [F]: in every published example, K_cool/K_hot ≈ 0.8–0.87, for decane at 14–21% O2 and octane at 0–50% He [S3].
- **Cool-flame K, theory** [T]: K_c = (8ρD/ρ_l)·ln[(2T_f − T0 − T∞)/T_L] and r_f/r_s = ln[(2T_f − T0 − T∞)/T_L] / ln[(2T_f − T0 − T∞)/(T_f − T0)]. Here T_f ≈ 600–800 K is the flame temperature and T_L = L/c_p is the heat of vaporisation in temperature units.
  - For heptane in air: about 0.5 mm²/s and an FSR of about 3, close to measurements [S5].
  - Partial-burning theory predicts K within ±20% and FSR ≈ 3.1 (one measurement: 3.2). FSR depends only on O2, not on D0 or CO2 [S7].
- **Dependence on D0 and pressure** [F]: K_c depends systematically on both and rises with D0. The hot flame's depletion of oxygen explains only part of this [S7].
- **The transition from hot to cool** [N]: the regression rate is insensitive to the regime change. The flame moves closer as it cools, so the temperature gradient at the surface hardly changes [S4]. FLEX shows a step down of about 15–20% [F, S3]. No published dip or delay is reported.
- **Cool-flame extinction law, O2/N2 only** [F + T]: D_ce ∝ X_O2^(−3/4) [S3]. The basis: the NTC crossover oxygen level varies with temperature with an activation energy E ≈ 120 kJ/mol; ketohydroperoxide branching has E ≈ 180 kJ/mol; so the chemical time goes as [O2]^(−3/2) and D_ce ∝ [O2]^(−3/4).
- **Across diluents and pressures** [F + T]: the residence time D²_ce/K_c correlates with C_O2^(−3/2) [S3]. C_O2 is the ambient oxygen concentration in mol/cm³, covering about 2×10⁻⁶ to 2×10⁻⁵, so it carries pressure and temperature. Heptane, octane and decane share one trend.
- **Later theory** [T, S5]:
  - d²_le ∝ (ρ_l/ρ)²·(K²/ρD)/(FSR_e·χ_fe), where χ_fe is the maximum inverse residence time. So D_ce scales with K.
  - D_ce ∝ ([O2] − β)^(−1/2), where β is proportional to pressure. The extinction diameter therefore steepens without bound as the cool-flame LOI is approached. That LOI is X_O2 of order 0.10 in N2, below the hot-flame LOI.
  - D_ce falls as pressure rises at fixed oxygen mass fraction. The measured pressure dependence is stronger than predicted, and for dodecane the prediction is about 4 times too high at low pressure [S5, S22].
- **Scatter in D_ce** [T]: the χ_f curve is flat near its maximum. Small perturbations therefore move the extinction point, and the scatter probably reflects D0 and ignition conditions [S5].
- **How the cool flame ends** [F]: abruptly. The D² curve plateaus, and a vapour cloud forms just before or after [S2, S3].
- **Elevated pressure and three stages** [N, S4]: at 3 atm with X_He ≥ 20% (21% O2, D0 = 4 mm), the sequence is hot (~1500 K) → warm (~970 K) → cool (~765 K).
  - At 20% He: warm stage about 23 s, cool stage 0.5 s.
  - At 60% He: warm stage about 1 s, cool stage about 18 s.
  - Above 60% He: hot then cool only.
  - The hot flame extinguishes at about 3 s in every case.
  - At 5 atm, three stages need X_He ≥ 40%.
  - In the dodecane experiment, the warm stage was 2 times shorter than predicted, and the hot flame reignited and then extinguished radiatively again [F/N].
- **Reignition causes** [N]: heat fed back through the tether fibre can reignite the hot flame, and so can other perturbations [S4].
- **Reduced models**: Paczko et al. 2014 (CTM 18:515) and Seshadri et al. (Combust. Theory Model., submitted 2015) give the −1/2 power [cited in S5, S7]. Cuoci et al. 2013 and Farouk & Dryer 2014 are detailed transient numerics [S3].

## 5. Helium after the visible flame is lost

- **What it is**: the evidence points to an invisible cool or warm flame with a higher K, not to enhanced evaporation.
  - **Detected, not imaged** [F]: radiometers detect continued heat release while the colour and UV cameras see nothing [S3]. FLEX-2 needed an intensified camera with a formaldehyde filter, plus median filtering and averaging over 500 ms, to image cool flames at all [S4].
  - **Helium raises the cool-flame K**: octane at 1 atm with 15% and 50% He had cool-flame K about 13% and 40% above the N2 case. K_cool/K_hot stays about 0.8 [F, S3].
  - **Burns are shorter, not absent**: helium shortens both stages [F, S3]. Diluents act through heat transfer and diffusion, not through the kinetics [T/F, S6].
  - **Evaporation alone cannot explain it** [E]: in a 300 K ambient, heat-driven evaporation stalls once the droplet cools (B_T = c_p(T∞ − T_b)/L < 0). Mass-transfer evaporation of heptane at room temperature gives K_evap of order 0.01 mm²/s, about 1/50 of the rates observed. Sustained linear shrinkage over many seconds needs a heat source.
- **Rate law to use**: K_post ≈ 0.8–0.87 × K_hot(same atmosphere) [F]. K_hot already includes helium's increase.
- **Elevated pressure with 20–60% He** [N]: expect a warm flame whose rate is about the hot-flame rate [S4].
- **Where it breaks down** [F/T]: at high helium dilution of air, cool-flame burning becomes highly transient and quasi-steady theory fails [S6].
- **Methanol**: no NTC chemistry and no cool flame is known. After extinction it simply evaporates [S2].

## 6. After extinction

- **Methanol** [F]: after diffusive extinction the vaporisation rate falls rapidly and substantially, then continues much more slowly in the hot gas [S1, S2]. The droplet holds about 30% water and sits in water-rich products, which slows it further.
- **Heptane after the cool flame** [F]: a plateau. Radiant signals decay to zero with no further abrupt change [S3].
- **After hot-flame extinction without a cool flame** [O]: in standard-atmosphere data, the rate gradually approaches pure evaporation. In some heptane and octane cases the post-extinction rate was higher, which is probably an unrecognised cool flame [S13].
- **Time constants** [E]:
  - Gas relaxation: τ_g ≈ (FSR·D/2)²/α_g. With FSR·D ≈ 10–30 mm and α ≈ 1–2×10⁻⁴ m²/s, τ_g ≈ 0.1–2 s. It grows as (FSR·D)², so roughly as D² at extinction.
  - Liquid sensible heat: c_p,l(T_s − T∞)/L ≈ 0.5 for heptane and about 0.1 for methanol. At most that fraction of the remaining mass can evaporate on stored heat, and less if the vapour cloud saturates the surroundings.
  - Liquid thermal diffusion time: D²/(4α_l). That is about 7 s for heptane at 1.5 mm, so cooling is conduction-limited and slow.
  - A decaying exponential with τ ∝ D² at the extinction diameter is a defensible form.

## 7. Heat-up and ignition transient

- **Size of the heat-up deficit** [E/T]: it scales with c_p,l(T_b − T0)/L, about 0.09 for methanol and about 0.5 for heptane. The heptane deficit is therefore several times the methanol one.
- **Duration** [F]: FLEX excludes about the first 10% of the burn from K fits [S1, S2]. A heat-up duration of about 0.1·τ_b is the empirical convention, not a measured law.
- **Swelling** [E]: thermal expansion of heated liquid can enlarge D by up to about 3% (heptane, about 615 versus 684 kg/m³) or about 2% (methanol). That is about 100 µm at 3 mm, comparable to the ±50 µm noise.
- **Early shape artefacts** [F]: the droplet is misshapen after the needles retract and regains its shape within about 1 s. Igniter glow makes the size ambiguous during ignition [S8].
- **Igniter energy**:
  - It sets the size reached in transient burns near the LOI and the radiative D_e of methanol [F, S9].
  - It can sustain combustion below the quasi-steady LOI [F, S2].
  - Controlling the rate and duration of energy deposition can start a cool flame directly [F/N, S4].
  - FLEX-2 cut igniter power automatically when the radiometer detected hot-flame ignition [S4].
- **Early methanol water condensation** [T]: it raises the early K of small droplets [S1].

## 8. Variability

- **Drift** [F]: the residual velocity from deployment and ignition is typically 1–2 mm/s, with g-jitter below 10⁻⁵ g0 [S2]. Ignition can impart a strong drift, for example when an air bubble bursts. Droplets sometimes hit an igniter [S8]. The Péclet number is about 0.2 [E], a mild asymmetry. Radiative extinction starts on the side away from the drift [S2].
- **Support fibre** (80 µm SiC) [F/N]: conduction along the fibre alters K and extinction [S16]. It can drive hot-flame reignition [S4].
- **Atmosphere reuse** [F]: 4–10 droplets per fill, in a chamber of about 95 L. The stated atmosphere is corrected for oxygen consumed and CO2 and H2O produced [S8]. One 3 mm heptane droplet uses about 0.1% of the chamber's oxygen [E], so per-burn vitiation is small but cumulative.
- **Needle contamination** in early tests [F]: it probably caused the methanol disruptive extinctions. It had minimal effect on K and diffusive D_e, and none on radiative D_e [S8].
- **Soot shells, soot tails, micro-explosion and disruption** [O/F]: these end heptane burns before diffusive extinction [S2, S10].
- **Cool-flame D_ce scatter** [T/F]: intrinsically large, because the extinction maximum is flat [S5].

## 9. Measurement

- **Optics** [F, S2, S8]: backlit at 650–660 nm, 1024² pixels over about 30 mm, so about 29.5 µm per pixel. Size is the equivalent circle from the projected area. The threshold was calibrated on precision spheres. Estimated uncertainty ±50 µm.
- **Known artefacts** [F, S8]:
  - Igniter glow and soot luminosity, which affected the early long-shutter tests.
  - Soot hiding the edge.
  - A droplet partly out of the field of view: an apparent sharp drop on either side of a gap.
  - Records ending abruptly on drift-out; flame data often continue.
  - Non-uniform background.
  - Vapour cloud at the end of the cool flame.
- **FLEX-2** [F, S4]: field of view enlarged to about 5×5 cm, giving coarser resolution.

## 10. What a forecaster ought to do

**Informative features of the prefix:**
1. **K after heat-up, compared with K_QS(atmosphere)**. A K below quasi-steady at low O2 marks a flame near its limit [S9].
2. **Early curvature**. Its magnitude should match the fuel's c_p,l·ΔT/L. Excess curvature suggests igniter or drift effects.
3. **D0 against a critical diameter**, together with O2, CO2 fraction and pressure. Radiative risk goes as p·D²·FSR³.
4. **For heptane in a long prefix**: whether a step of about 15–20% in K has already happened, which signals a cool flame.

**Unpredictable in principle:**
- the side of the bimodal radiative/diffusive split for D0 near critical;
- whether a cool flame follows a hot-flame extinction near threshold;
- the exact cool-flame extinction point;
- disruption or micro-explosion;
- reignition at high pressure;
- drift out of the field of view.

These need mixture or branching distributions, not a single path.

---

## Laws a hand-written model should contain (best established first)

1. **D²-law with K = (8λ/ρ_l c_p)·ln(1+B), Law–Williams properties.** For trends use B_methanol ≈ 12.1·Y_O − 0.05 and B_heptane ≈ 40·Y_O − 0.46. Apply empirical diluent factors: He about +35% and CO2 about −15% at 30% replacement [T + F].
2. **Methanol diffusive D_e rising linearly with D0**, upward shift as O2 falls, LOI ≈ 0.12–0.14 [F].
3. **Radiative extinction as a sharp, bimodal switch.** For methanol above D_crit (about 4 mm in air at 1 atm), D_e ≈ D0, so the flame is lost early [O/F]. D_crit falls with lower O2, with CO2 and with higher pressure, roughly as p^(−1/2) [E/T].
4. **Heptane cool flame: K_c ≈ 0.8–0.87·K_hot**, in every diluent, helium included [F].
5. **D²_ce/K_c ∝ C_O2^(−3/2)** (mol/cm³) [F + T]. Close to the LOI, the form ([O2] − β)^(−1/2) is better [T].
6. **Post-extinction decay** with τ ∝ (FSR·D_e)²/α plus slow mass-transfer evaporation at about 0.01 mm²/s [E].
7. **Pressure plus helium**: a warm stage at about the hot-flame rate [N].

## Open or contested

- **Radiative-extinction law**: the fitted scaling of D_e(D0, gas) in [S14] was not accessible.
- **Cool-flame D_ce pressure dependence**: stronger than any model predicts [S5, S7].
- **Exponent of the D_ce oxygen law**: −3/4 against −1/2 [S3, S5].
- **Methanol under He at the radiative branch**: no FLEX analysis found.
- **K_c rising with D0**: only partly explained [S7].

## References

- [S1] Nayagam, Dietrich, Hicks, Williams, "Methanol droplet combustion in oxygen-inert environments in microgravity", 8th US Natl. Combust. Mtg. (2013), paper 070HE-0303. https://ntrs.nasa.gov/citations/20130014061
- [S2] Dietrich et al., "Droplet combustion experiments aboard the International Space Station", Microgravity Sci. Technol. 26 (2014) 65–76. https://bpb-us-w2.wpmucdn.com/sites.coecis.cornell.edu/dist/9/231/files/2018/09/2014-MST-Droplet-combustion-experiments-aboard-the-2je8ess-12toi72.pdf
- [S3] Nayagam, Dietrich, Hicks, Williams, "Cool-flame extinction during n-alkane droplet combustion in microgravity", Central States meeting (2014); journal version Combust. Flame 162 (2015) 2140–2147. https://ntrs.nasa.gov/citations/20140006750
- [S4] Farouk, Dietrich, Dryer, "Three stage cool flame droplet burning … hot, warm and cool flame", Proc. Combust. Inst. 37 (2019) 3353–3361. https://ntrs.nasa.gov/citations/20190002838
- [S5] Williams, Nayagam, Combust. Theory Model. 23 (2019) 748–770. https://par.nsf.gov/servlets/purl/10111319
- [S6] Nayagam, Dietrich, Williams, ASGSR 2020 slides. https://ntrs.nasa.gov/citations/20205008449
- [S7] Nayagam, Dietrich, Williams, ASGSR 2015 slides; see also AIAA J. 54 (2016) 1235–1239. https://ntrs.nasa.gov/citations/20160000344
- [S8] Dietrich et al., NASA/TP-2015-216046 (methods sections only were used). https://ntrs.nasa.gov/citations/20150023456
- [S9] Nayagam et al., Eastern States meeting (2012); the PDF text is partly corrupted, so its numbers were lost. https://ntrs.nasa.gov/citations/20120016855
- [S10] Manzello, Choi, Kazakov, Dryer, Dobashi, Hirano, "The burning of large n-heptane droplets in microgravity" (2000). https://ntrs.nasa.gov/citations/20020028056
- [S11] Nayagam et al., Combust. Flame 159 (2012) 3583–3588 (abstract only). https://doi.org/10.1016/j.combustflame.2012.07.012
- [S12] Farouk, Dryer, Combust. Flame 161 (2014) 565–581 (abstract only). https://doi.org/10.1016/j.combustflame.2013.09.011
- [S13] Liu, Xu, Hicks, Avedisian, Combust. Flame 171 (2016) 27–41 (abstract only).
- [S14] Nayagam, Dietrich, Hicks, Williams, Combust. Flame 194 (2018) 107–114 (abstract only).
- [S15] Marchese, Dryer, Colantonio, Proc. Combust. Inst. 27 (1998) 2627–2634 (via S1 and abstracts).
- [S16] Hicks, Nayagam, Williams, Combust. Flame 157 (2010) 1439–1445 (via S1). https://ntrs.nasa.gov/citations/20110020695
- [S17] Easton, T'ien, Dietrich, decane radiative extinction (1999). https://ntrs.nasa.gov/citations/20000007141
- [S18] Chao, Law, T'ien, Proc. Combust. Inst. 23 (1990) 523–531 (cited, not read).
- [S19] Law, Williams, Combust. Flame 19 (1972) 393–405 (cited, not read).
- [S20] Shaw, Wei, drop-tower LOI (as cited in S2).
- [S22] Williams, Nayagam, Combust. Flame 212 (2020) 242–244. https://par.nsf.gov/servlets/purl/10176660

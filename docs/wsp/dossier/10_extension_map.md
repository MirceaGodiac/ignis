# 10. Where the models can grow

**Astra checkpoint:** [ASTRA_RESULTS.md](ASTRA_RESULTS.md) records what became of the
ranked first-principles proposals: persistent condition uncertainty, scoped droplet cooling,
and oxygen-supply spread were built once, scored on train and retained without promotion.
All are data-informed. No unseen improvement follows; the all-NASA instruction now expands
the work under frozen-group and blind-measurement rules.

The owner's instruction of 14 November: the models may have "many, many parameters if it is added
physical logic", must generalise, and must come from understanding what each experiment does. This
page is the author's position on how a hand-written model can honestly become large, and a map,
family by family, of the mechanisms the physics dossiers found, the parameters each would add and
how far each is checked. It is a starting position for a second author to attack and extend.

## 10.1 What a parameter is in a literal model

A release is a forward (code that computes, with explicit integration and no fitting) and tensors
(numbers the author types, each with a reason). Expressive power can sit in either. The question
"how many parameters" is therefore the wrong one; the useful question is where each number's
justification comes from, because that decides what it costs in generalisation.

| Kind | What it is | Where its value comes from | Can it grow freely? |
|---|---|---|---|
| **N** nature | Constants and property data: stoichiometry, heats, vapour-pressure coefficients, diffusivities, conductivities, emission coefficients, enthalpy tables | Handbooks and standard fits, transcribed | Yes. It carries no information about the runs. The risk is transcription and unit errors, which tests can catch |
| **A** apparatus | Sphere mass, igniter timing, frame rates, script delays, calibration rows, field of view | Engineering documents, or instrument readings on train | Yes, when a document gives it; with care when train gives it |
| **L** law | The few constants of a mechanism that theory leaves open: a critical flux, an extinction temperature, an oxygen limit | Literature on other fuels or campaigns; else train | Slowly: one or two per mechanism, each carried with a stated range |
| **S** structure | Class gates, mixture weights, how much a prefix is trusted | Judgment, or train | Slowly, and each needs a reason that holds in another session |
| **U** uncertainty | Band widths, spreads of scenario nodes | Train residuals plus a margin that is a judgment | As little as possible; three refused candidates came from sharpening these |

Three consequences.

1. **A model can hold thousands of typed numbers and still have few degrees of freedom towards the
   train data.** A property table of 300 entries and 20 law constants is a "big" model whose risk of
   overfitting is that of the 20. Every card should count the two separately: numbers typed in
   total, and numbers whose value was read from train.
2. **The carried models are small in the wrong way.** They fold nature, apparatus and law into a few
   coefficients read off train tables (a burning rate "0.54 mm²/s in air" contains the conductivity,
   the heat of vaporisation and the flame temperature at once). That is why they cannot say what
   happens at 45 % helium or at 3.3 atm: the physics that would carry them there is not in them.
3. **Every N or A number added removes a degree of freedom from L, S and U**, because it replaces a
   coefficient that had to be read from train with one that could not have been.

## 10.2 Ways to grow, and where each stands

**Astra qualification (F-006):** section 10.1's three consequences are proposals, not
complexity theorems. A property table does not establish that only the remaining law
constants count as effective degrees of freedom, and adding a nature/apparatus constant
does not necessarily remove one adjustable choice. Source selection, closure choice and
code changes after feedback also consume scientific discretion. [ASTRA_DESIGN.md](ASTRA_DESIGN.md)
keeps the N/A/L/S/U roles and adds an independent evidence-origin and selection-history
ledger. Counts support auditing, not a generalisation guarantee.

| Pattern | What it adds | Status here | Limits known |
|---|---|---|---|
| More states and mechanisms, integrated explicitly | L numbers with meaning | Partly: three droplet states, growth clock, heating law, band-shaped flame | Each new mechanism needs an evaluation against train at literature constants before a number is typed |
| Property data as tensors | Many N numbers | **Not done.** The forwards have no property tables | Values recalled from memory must be looked up; the heptane dossier marks its own as such (X-15) |
| A resolved transport solver as the forward (spherical diffusion with a flame sheet; a fuel-bed heat balance) | Structure in code, N numbers for transport | Not done. Two physics dossiers ran such models outside a forward | The inverse-flame model's temperatures were wrong and its radiation twice too strong: a solver is not automatically right, and it is slow |
| Closed forms from theory in place of hinge units | Fewer S numbers, more N | Partly (growth law) | The closed forms found so far reproduce ratios and trends well and absolute levels badly: type the level, use theory for the dependence |
| Scenario committees | S numbers that say "I do not know which law holds" | Done (`blend.rs`, `committee.rs`) | Gains only when members differ in kind (C-27) |
| Instrument models | A numbers | Partly: validity gate, flame cameras | View censoring (C-12) |
| Session-level latent quantities integrated over typed nodes | U numbers with a physical name (true oxygen fraction, true flow) | Partly | Needs a statement of what varies by session; the dossiers each give one |
| External data to fix L numbers | No exposure of a partition | Started: the earlier BASS campaign (M-11) | Comparability must be argued each time |
| Deterministic copying of stored values between releases | Size without new numbers | Done (`blend_edits.py`, `committee_edits.py`) | The copier may choose and compute nothing |
| Generated tables (a script evaluates a formula on a grid) | — | **Not allowed** by the protocol: a program would write weights | If a function is needed, it belongs in the forward as code, with its constants typed |

## 10.3 The map, family by family

Status words: **rechecked** (the author recomputed it on train), **arithmetic** (the author
re-evaluated the stated formula and it gives the stated numbers), **on trust** (as the report says).
Reports: [phys_bass.md](../research/phys_bass.md), [phys_flex_heptane.md](../research/phys_flex_heptane.md),
[phys_acme_inverse.md](../research/phys_acme_inverse.md). Each has an independent check with its own
code on train ([feedback/verify_phys_*.md](../feedback/)), except methanol.

### n-Heptane droplets

| Mechanism | Computable form | Numbers it needs | Status | Against the carried model |
|---|---|---|---|---|
| Flame temperature with the oxygen Lewis number; burning rate from the transfer number | algebraic; property mixing rules | N: property tables (conductivities, diffusivities, enthalpies); L: one weight for the Lewis correction (0.5, range 0.2 to 1) | on trust; ratios to air within 10 % for N2 and CO2, helium between the two limits | The typed oxygen, CO2 and helium slopes agree up to 30 % CO2; the CO2 slope has the wrong sign at 70 % |
| Liquid heat-up and thermal expansion | series in the liquid's own diffusion time, 0.31 clock units | N: liquid properties; S: an amplitude (0.19) | on trust; over-predicts for 2.5 mm droplets | Supports keeping the heat-up term on the D0² clock |
| Radiative critical diameter from emission and an extinction temperature | algebraic | L: `D_crit` in air (3.7 mm, train), an extinction temperature (1650 K, train-read), a pressure exponent (−0.1, range −0.3 to 0) | on trust; wrong side for three runs at 0.7 atm and for the 2 atm CO2 run | Reproduces the typed law within 10 % in N2 and CO2 up to 30 %; neither law explains 2 atm |
| Cool-flame rate | algebraic, no oxygen or pressure dependence | N; L: a level (0.34 mm²/s) | **confirmed** by an independent re-derivation (0.32 to 0.36 mm²/s on twelve runs, no trend in oxygen, CO2 or pressure) | **Contradicts** the typed oxygen and pressure terms of the cool rate |
| Cool-flame end diameter | `D_ce/D0 = g0 √((0.18 − β)/(pO2 − β))`, pO2 in atm | L: g0 (0.47), β (0.11 atm, range 0.10 to 0.125) | **confirmed** by an independent re-measurement of twelve plateaus, with β at 0.12 to 0.125 rather than 0.10 and a D0 exponent of 0.95 (0.8 to 1.25) | The carried law has no divergence near the oxygen limit and a D0 exponent of 0.8 where three pairs give about 1 |
| Evaporation of a droplet left hot by a flame | `K0 / (1 + t/t0)`, `t0 = 26 K · c_l · D² / (1.5 · K0 · L)` | N; A: the surface temperature left by a hot flame (boiling point − 3 K) or a cool one (− 19 K) | **weakened** by the independent check: the amplitude over the first two seconds is right, but after a cool flame the decay does not scale with D² (what orders the runs is how long the cool flame lasted), and after a hot flame in helium the scaling holds only from 8 s on. Not used for heptane | Replaces three typed numbers by none; explains half or more of the helium after-burn |
| A dying flame below the cool-flame oxygen limit (helium, low pO2) | exponential decay over about 3 s | L: one time (2 to 5 s) | on trust; one session | Replaces the patch that forces "no cool flame" in helium |

Not predictable, by the report: the side of the critical diameter within about 10 %, the end of a
cool flame to better than 12 %, disruption, drift.

### Inverse burner flames

| Mechanism | Computable form | Numbers it needs | Status | Against the carried model |
|---|---|---|---|---|
| Fuel reaches the young flame by transient diffusion | `m''(t) = K(Z_st) σ Y_F ρ √(D_F/(π t)) + m_O2/(4π r_ss²)` | N: fuel diffusivities, stoichiometry; L: `K = 0.8 (1 − Z_st)^−0.8` (read off a transport model) | **weakened**: the closed form predicts the onset within a factor 0.6 to 1.8, misses the pressure dependence between 1.1 and 2.2 bar entirely, and on the one clean pair the fuel-fraction exponent is 1.1 to 1.6, not 2 | Explains why lives barely depend on flow in a lean ambient |
| Oscillatory extinction at a critical oxygen flux | onset when `m''` falls to about 1.0 g m⁻² s⁻¹; then a chance of dying per cycle | L: the flux (1.0, sd 0.11, ±20 % systematic), a pressure exponent (0.15, range 0 to 0.4), an ethane value (unobserved); S: per-cycle death probabilities by class | **confirmed** for the bursts by an independent detector (33 of 34 dying propane and butane flames; none in ethane or in surviving air flames); the flux value stays on trust | Replaces the empirical early-quench clock |
| Classes by stoichiometric mixture fraction | bands at about 0.45 and 0.57 | S: two boundaries, class probabilities | **rechecked** for the separation (M-17, C-38) | Replaces the ambient-fuel hinge, the oxygen-fraction patch and the total-flow scenario (D-26) |
| Burner screening by the outflow | conduction ∝ `Pe/(e^Pe − 1)`, `Pe = 0.38` per mg/s | N, A: sphere heat capacity 0.31 J/K (sphere only) | on trust | The low-flow unit is real and should be written in total flow |
| Flash at ignition | a class by ambient fuel and flow | S | on trust; five flames | The typed temperature threshold does not discriminate on train |

Not predictable: which side a flame at a mixture fraction of 0.47 to 0.57 falls on; the fibre; the
true flows when a controller did not deliver.

### Flame spread over the fabric

| Mechanism | Computable form | Numbers it needs | Status | Against the carried model |
|---|---|---|---|---|
| Oxygen delivered equals fuel consumed | `½(v_tip + v_base) m_F r = ρ Y_O U δ`, `δ² = ν L/U + (α/U)²` | N: gas properties at 1250 K, stoichiometric ratio, cotton per face; no adjusted constant | **rechecked** by hand on two steady train conditions: 1.00, 0.96 and 0.96 of the measured speeds. A 20 mm width in place of 22 mm moves it by 8 %. The independent check confirms it for settled flames (0.87 to 1.09 on twelve windows) and adds that it mostly tests one diffusivity, fails in transients, and that the flow at the lowest fan setting is not known | Derives the typed speed line (intercept 0.029, slope 0.0061 at 18.8 % oxygen against the typed 0.0215 and 0.0064) |
| Oxygen dependence | intercept ∝ Y_O, slope ∝ Y_O² · t_res | none new | on trust | The single power 1.5 is the wrong form; it overstates low-flow speed in air by 10 to 14 % |
| Residence time `t_res = L/v` | one typed time per regime | L: 10 s near the limit, 14 s for a robust wide flame, 8.7 s narrow | on trust; confounded between campaign, oxygen and flow | New |
| Asymmetric response of the flame length | relax down in about 2 s, up in about 10 s | L: two times | shrinking **confirmed** (63 % within 1.5 s); growth is 63 % in 2 to 7 s and steady at 15 to 18 s, so the asymmetry is a factor 2 to 5, not 10; no step up exists in the data | The single 2 s time is the cause of the overshoot at ignition |
| Narrow sample | length power about +0.5 in own widths, speed factor 1.25 to 1.6 | L | **confirmed** as numbers on two burns in air: speed 1.54 and 1.26 times the wide law in own widths, length 0.86 and 0.62 | **Contradicts** the typed width power of the overhang (−1) and weakens that of the speed |
| Flow uncertainty near fan stall | three flow paths (U, U ± 1 cm/s) | U with a physical name | on trust (fan calibration across days) | Explains why the quench law did not carry to another day |
| "Carries a flame" as a class | a prior in oxygen, updated to certainty by one flame still | S | proposal | Replaces the quench hazard. The report's suggestion to floor the prior at a validation aggregate is not to be typed (X-14) |
| Downstream slow-down | `U_eff = U (1 − κ (x − x_ref))` | L: κ 0.05 per width (0.03 to 0.12) | **weakened**: progressive in one train burn of three (0 to −7 % per width, centre −3 %) | New |

The report's own caution: two mechanistic closures for the transients scored worse on train than the
carried law. The skeleton of the carried forward stays.

### Normal burner flames

Report: [phys_acme_normal.md](../research/phys_acme_normal.md).

| Mechanism | Computable form | Numbers it needs | Status | Against the carried model |
|---|---|---|---|---|
| Energy balance of the flame sheet with a radiative length | `(T_ad − T_f)/(T_f − T∞) = δ_O/ℓ*`, `δ_O = r_f²/r_ss`; extinction radius `r_ext² = Θ ℓ* r_ss`, `Θ = (T_ad − T_cr)/(T_cr − T∞)`; held when `r_ss ≤ Θ ℓ*` | L: a radiative length per fuel (7.4 to 11 mm), a critical temperature (1130 K published for ethylene; 1130 to 1400 K carried), a pressure exponent of the length (0.45, range 0.3 to 0.7) | on trust. One corner of the constants reproduces 30 % ethane at 1 atm within 15 %; flames in air come out 2 to 4 times too long at every corner; no single pressure exponent fits both 0.5 and 2 atm. The independent check reproduces the table within 3 % and adds the cells it left out: the ten weak flames come out 7 to 13 times too long and the corrected butane-in-air rows several times too long | Explains why the published critical flux is nearly constant; makes the weak-flame class a limit of the same law (Θ → 0) instead of a separate logistic |
| Pressure acts through the radiative length, and the diffusivity goes as 1/p | as above | none new | on trust | **Contradicts** the typed `se.diff.skip[0] = +0.45` (a diffusivity that rises with pressure). The two placements differ on the holding flow at 0.5 and 2 atm |
| Head start from the ignition flash | a time offset of about 2 s (0 to 5), larger at a low mixture fraction | L: one time | on trust (from the chamber pressure jump at ignition) | New. A candidate, not a demonstrated cause, for flames in air ending twice as fast as the law says |
| Burner energy balance | conduction reduced by the outflow, radiation from the flame, the burner's own radiation, tube loss, gas heating; explicit Euler | N, A: heat capacity 0.308 J/K, emissivity 0.6, area; L: a radiative input of about 1.2 W, a gas-heating efficiency of 0.25 | on trust | Replaces the single 30 s time constant; explains why the traces have more than one |
| Burner limit by era | about 500 °C with 4 s of dwell before late August 2021, about 450 °C after | A: two settings with dates | agrees with M-08; normal-flame trips at 502 to 522 °C and 451 to 455 °C | The typed pair of limits (455 °C at 0.4, 510 °C at 0.6) is wrong for any session before late August |
| Cool flame after a hot extinction (n-butane at pressure) | a rule on the burner temperature at extinction (above about 515 K) | A | on trust (published rule; train radiometer plateaus) | Two train flames labelled as burning for 167 and 246 s lost their first hot flame at 41 and 89 s; both forwards learned from them that such flames last |
| Six train rows with wrong inputs | — | — | agrees with M-08 | The boundary of the weak-flame class rests on one real test and three mislabelled ones |

Still unexplained after this report: why flames in air end twice as fast as the law gives, and why
n-butane in enriched oxygen lives 0.5 to 0.7 of ethane (soot or radiation from the interior are
candidates, not shown).

### Methanol droplets

Report: [phys_flex_methanol.md](../research/phys_flex_methanol.md). Its working folder holds about
ten variants of one simulation: they are pairs of named values (radiation factor 1 and 2, extinction
temperature 1200 and 1500 K, CO2 band factor 1 and 0.44, initial water 0 and 0.2, helium exponent 1
and 0.5), a comparison and not a scan. Five of its constants are read from train and say so.

| Mechanism | Computable form | Numbers it needs | Status | Against the carried model |
|---|---|---|---|---|
| Burning rate from an inner conductance and the oxygen conductance outside the flame | `K = 8 (c_I I_i + G_o)/ρ_l` | N: property tables; L: one level `c_I = 0.61`, read from train | on trust; every trend right, the level 1.65 times too high without the typed constant | The typed oxygen, CO2, helium and pressure terms agree in sign; helium lies between the two theory limits |
| Heat-up as a liquid heat sink plus thermal swelling | closed form in the Fourier number, 0.376 per clock unit | N | on trust; small droplets burn faster early than conduction allows | The clock scaling is right; the shape is a slow tail, not a hinge ending at 0.18 |
| Water uptake; flame-out when the liquid holds about 30 % water | closed form for the water flux; a threshold | L: the threshold (0.30, train-read), its rise for flames hotter than about 2350 K | on trust; a thermal extinction criterion fails outright (0.25 of the initial diameter everywhere) | The typed diffusive law equals that threshold; its pressure term has no support |
| Radiative loss with a CO2 band that saturates | critical diameter from the radiant fraction | L: a threshold ratio (0.45, typed after seeing the N2 table), an extinction temperature, a saturation level | on trust; the timing of an early loss is poor | **The typed critical diameter in air (9 mm) is twice what physics (4.5 mm) and the older literature (4 to 4.5 mm) give; train only says above 3.9 mm. The typed pressure term has the sign opposite to radiation** |
| Unlit droplet evaporating on its stored heat | `K0/(1 + t/t0)`, `t0 ∝ D²` | N | arithmetic by the author: with a start rate of 0.30 mm²/s and 22 K per e-fold it gives 0.150, 0.072, 0.038, 0.020 mm²/s over the four windows after an early loss at 3 mm (measured 0.143, 0.085, 0.052, 0.035) and 0.051, 0.017, 0.008, 0.004 after a water extinction at 1.3 mm (measured 0.057, 0.029, 0.010, 0.006): two sizes, one constant | Replaces the typed size exponents of the after-flame law. For methanol the size scaling holds, unlike after a heptane cool flame |

## 10.4 What I would do first, and what I would not

1. **Property tensors.** A checked list now exists: `research/properties/properties.csv`, 374 values with source and
   verdict, and a proposed block at the end of `research/properties_verified.md`. One shared block of N numbers per fuel and diluent, typed from handbooks
   with the source beside each, with tests for units and for limiting cases (a burning rate in air
   within a factor two of the measured one; a boiling point at 1 atm). It serves all droplet and
   burner forwards and costs no exposure.
2. **The closed forms that were confirmed by re-evaluation**: self-cooling evaporation (droplets),
   the oxygen balance (fabric), the mixture-fraction classes (inverse flames). Each replaces typed
   train coefficients by nature constants.
3. **Session-level nodes with physical names** in place of anonymous spreads: true oxygen fraction,
   true flow near stall, the burner limit of the day.
4. **Not yet:** a resolved solver inside a forward. The one transport model run so far had wrong
   temperatures; until a solver reproduces train flame temperatures or radiant output it would add
   size and confidence without adding truth.
5. **Not at all:** widening a class boundary or narrowing a band because train looks that way; a
   number recalled from memory; a constant derived from two train points and called physics.

## 10.5 Questions I have not answered

1. Is there a principled count of a hand-written model's freedom towards its train data, so that
   two models of different size can be compared before validation? Description length of the typed
   reasons is one candidate.
2. When theory gives the dependence and train gives the level, where does the level's uncertainty
   for an unseen condition come from? Today it is a judgment.
3. How should a forward use a law that is right in ratios and wrong in level by a factor of two (the
   burning rate, the flame standoff 16 % low everywhere)?
4. Can the constants of one family be tied to another's (the same gas properties serve droplets and
   burner flames; the same radiation model serves both)? Tied numbers are a strong test: one wrong
   value shows in two experiments.
5. What is the right size? The owner asks for ambition; three refused candidates say that added
   detail read from train hurts. The table of 10.1 is my answer, and it has not been tested.

## 10.6 What the fifth-generation look said about this map

One look per family ([RESULTS_GEN5.md](../RESULTS_GEN5.md)); two sessions or seven to nine days
each, so every line below is weak evidence.

| Way to grow (10.2) | What was tried | What validation said |
|---|---|---|
| Numbers of kind N in place of numbers read from train | Methanol: the after-flame evaporation as self-cooling with four property-table constants, replacing three train-read numbers | Equal on train, not distinguishable on validation. The model grew in typed numbers without growing in freedom towards train, which is what this map argues for; it earned no score |
| A law in place of a patch (kind L) | n-Heptane: cool-flame rate without the oxygen and pressure terms; cool-flame end in oxygen partial pressure; a dying flame in place of the helium patch | Better on train by 2 µm, worse on validation by 1 µm, inside the interval |
| A second scenario as a committee member (kind S, with a typed probability, kind U) | Inverse flames: a held flame in the marginal band, one third | Carried over: extinction 0.161 to 0.129 |
| A new prospective input | The planned stop of a burner test | The largest effect of the generation: flow termination 0.094 to 0.048 (inverse), 0.098 to 0.075 (normal) |
| A new observation in the prefix | The flame cameras of the droplet runs | No gain: 1.2 µm worse than the same laws without them (n-heptane) |
| Data from outside the partitions (kind A, another campaign) | The narrow-sample powers of the fabric flame from two burns of the earlier campaign | Tip level, track-ended score worse; refused |

What I take from it. Knowing more about the test (its plan) beat knowing more about the flame. A
stated second scenario beat a sharper single law. A law that is better founded did not score better
on a held-out set this small, so the case for writing models this way has to be made on what a
reader can check (10.1), and the question of 10.5, how to compare two models of different size at an
equal score, is now the one that decides what gets carried.


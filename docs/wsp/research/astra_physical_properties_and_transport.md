# Astra: physical properties, transport, and cross-family extensions

Research note, 14 November 2026. Scope: class A science, narrowly used class B campaign context, and one class C mixed source whose abstract-level aggregate summary was visible and read. No per-test outcomes, partition files, model weights, or parameter selection were used. The class C exposure is disclosed below and did not inform any coefficient or proposed law. This is a design proposal, not a candidate or release. It introduces no model numbers chosen from data.

## Main recommendation

The highest-value extension is a versioned shared property block, first covering the two FLEX fuels and the shared gas species used by the burner and fabric families. Store source-native quantities, units, validity intervals, uncertainty or evaluation status, and an equation/source locator. Derived values should be explicitly marked as derived and reproducible from the listed source values. A model should consume this block by property identity and requested state, rather than copy approximate constants into family-specific weights.

Start with the quantities that are both common and defensible: molecular mass; stoichiometric oxygen requirement derived from the balanced formula; NIST vapor-pressure equations with their stated domains; liquid density and heat capacity as temperature-indexed data; and reference gas heat capacities with their equation and temperature interval. Add binary diffusivities only for pairs and temperature ranges supported by direct measurement or critically evaluated tables. Keep a missing pair missing. Do not silently fill it with a Fuller estimate.

This shared block makes a wrong constant visible across every forward that uses it. It also reduces free parameters: the model specifies a mechanism and geometry, while a canonical source supplies the substance property. This is a proposal for a shared source-data layer, not a new cross-family prediction law.

## Directly checked property examples

I independently opened the NIST WebBook equations and queried its fluid-property CGI tables for saturated-liquid points. The values below are a deliberately small demonstrator, not a complete fuel database.

| Fuel | Quantity | Source value / representation | Applicability and note |
|---|---|---|---|
| n-Heptane | Antoine vapor pressure | `log10(P/bar) = 4.02832 − 1268.636/(T − 56.199)` | NIST lists 299.07–372.43 K, citing Willingham, Taylor et al. (1945). Use only in that interval. |
| Methanol | Antoine vapor pressure | `log10(P/bar) = 5.20409 − 1581.341/(T − 33.50)` | NIST lists 288.1–356.83 K, citing Ambrose & Sprake (1970). NIST also lists a distinct high-temperature fit; do not extrapolate the low-temperature fit to 3 atm boiling conditions above its range. |
| n-Heptane | At saturated liquid, 300 K | `p_sat = 6.68452 kPa`; molar density `6.76572 mol/L`; `Cp = 225.829 J/(mol K)`; `k = 0.121592 W/(m K)` | NIST WebBook fluid table. With NIST molar mass 100.2019 g/mol, density and mass heat capacity are derived as about 678.0 kg/m³ and 2.254 kJ/(kg K). |
| Methanol | At saturated liquid, 300 K | `p_sat = 18.6824 kPa`; molar density `24.4836 mol/L`; `Cp = 81.5836 J/(mol K)`; `k = 0.199817 W/(m K)` | NIST WebBook fluid table. With molar mass 32.0419 g/mol, derived density and mass heat capacity are about 784.5 kg/m³ and 2.546 kJ/(kg K). |
| n-Heptane / methanol | Stoichiometric oxygen-to-fuel mass ratio | `σ = 32(x + y/4 − z/2)/M` for `C_x H_y O_z`; about 3.514 kg O₂/kg heptane and 1.498 kg O₂/kg methanol | Formula-derived from balanced complete combustion, using source molecular masses. These are stoichiometric bookkeeping ratios, not measured flame-consumption rates. |

The last two rows' derived mass properties use the NIST saturated-liquid molar density and heat capacity at one point; they are not temperature-independent constants. For production use, retain the tabulated molar quantities and source state, then derive SI mass quantities with a checked conversion. At a requested state, use the NIST fluid reference correlation or a clearly declared interpolation within the source's tabulated range. Do not extrapolate across the critical point or a phase boundary.

The NIST values independently reproduce the corresponding 300 K rows in `docs/wsp/research/properties_verified.md`. That agreement is a cross-check, not a substitute for the source links below.

## Molar versus mass evaporation: a consequential distinction

For a stationary, isothermal, quasi-steady spherical gas film with a single evaporating species `F`, one inert carrier, ideal-gas concentration `c = p/(RT)`, negligible far-field fuel vapor, and constant film `D`, integrate the species molar flux with Stefan outflow. At the droplet surface let `x_s = p_sat(T_s)/p`. The interfacial molar flux is proportional to

`N_F = c D / r · ln(1/(1 − x_s))`.

Multiplying by fuel molar mass and equating loss of liquid mass gives the exact constant-property square-diameter coefficient for these assumptions:

`K_molar = 8 M_F c D / ρ_l · ln(1/(1 − x_s))`, where `d(d²)/dt = −K_molar`.

The common mass-fraction Spalding form writes `K_mass = 8 ρ D / ρ_l · ln(1 + B_M)`, with `B_M = Y_s/(1−Y_s)` when far-field fuel is absent. The conversion `Y_s = M_F x_s/[M_F x_s + M_A(1−x_s)]` matters. Treating the two logarithms as equal, or substituting an ambient density into a molar-flux derivation, is not generally valid when `M_F` and `M_A` differ. The exact derivation assumes one inert carrier and ideal-gas film closure; mixtures with multiple carriers, appreciable ambient fuel vapor, non-ideal fugacity, internal liquid gradients, convection, finite-rate chemistry, and temperature-varying properties require additional equations.

This distinction is much larger for heptane in air near boiling than for methanol in air because heptane is much heavier than the carrier gas. The prior property audit calculated roughly a factor 2.4 near heptane's normal boiling point at `T_b−3 K`; for methanol the discrepancy is under 10% in the analogous condition. These are equation comparisons using the reference properties, not measurements or selected model weights. The molar form is the cleaner baseline when the stated stagnant-carrier assumptions apply; the mass-fraction form remains a useful model convention only when its assumptions and mixture-property definition are stated.

Pressure and mixture effects should be computed through the state and composition: `x_s = p_sat/p`, gas molar concentration `p/(RT)`, and pair/mixture diffusivity. For a dilute gas mixture, a declared Blanc/Wilke-style rule may combine measured binary diffusivities; that mixing rule is an approximation and needs its own source and validity note. He/CO₂ substitutions cannot be assigned a universal multiplier without species-pair data.

## Similarity variables are architecture, not fitted features

Build state and mechanism descriptions around dimensionless groups whose definitions are fixed by physics:

| Process | Candidate group | Use |
|---|---|---|
| Forced-flow transport | `Re = ρ U L/μ`, `Pe_m = U L/D`, `Pe_T = U L/α`, `Le = α/D` | Separates advection, species diffusion, heat diffusion, and relative thermal/species transport. |
| Transient conduction | `Fo = α t/L²`, `Bi = hL/k_s` | Expresses whether solid/liquid temperature can be lumped and the relation between boundary exchange and internal conduction. |
| Chemistry / transport competition | `Da = τ_flow/τ_chem` | Organizes kinetic versus transport-limited behavior when chemistry timescales and mechanisms are available. |
| Radiation | `τ_rad = κ L` (with a stated spectral or grey model) | Indicates optically thin versus thick behavior; a grey coefficient is not transferable without its source/range. |
| Residual buoyancy | `Gr = g β ΔT L³/ν²` or an equivalent buoyancy-to-forced-flow ratio | A check on when the microgravity/forced-flow reduction applies; set `g` from apparatus conditions rather than presuming exactly zero. |
| Oxidizer sufficiency | `Φ_O2 = available O₂ mass flux/(σ · fuel mass flux)` | A control-volume supply/demand ratio for solid-fuel propagation; it cannot be computed from nominal flow alone without the capture geometry and fuel flux. |

Similarity does not reveal an unknown function or certify that two facilities share a regime. It gives a compact, dimensionally valid interface and tells a reviewer which physical competition a law expresses. Regime transitions, radiation, confinement, and chemistry can introduce additional independent groups. Do not let the data choose exponents on these groups while calling the result first-principles.

## Reduced transport and exact time updates

The useful next step is a reduced transport model with interpretable states, rather than a full detailed-chemistry solver at once:

1. **Droplets:** couple a liquid energy balance to quasi-steady spherical gas species/heat transport. Use NIST liquid properties, vapor pressure, latent heat, and supported binary diffusion values. Separate a thermal state from the diameter state. The limiting cases should recover the constant-property `d²` law and a no-evaporation state when `p_sat ≤ p_F,∞`.
2. **Burner flames:** use a radial reduced species/energy balance with a labeled oxygen/fuel supply, radiative loss, heat exchange to the porous burner, and finite chemistry/strain limits. Preserve separate flags or probabilities for kinetic extinction, radiative extinction, oscillatory partial extinction, and apparatus cutoff; these are not one mechanism.
3. **Fabric spread:** use an oxygen control-volume budget coupled to a solid preheat/pyrolysis budget. A candidate conservation check is `ṁ_O2,available ≥ σ ṁ_fuel,consumed`. Estimate the available oxygen flux across a stated control surface from the flow and mixture state; expose the control-volume/capture-width assumption and its uncertainty instead of hiding it inside a spread-rate coefficient.

Where a subsystem has a closed-form evolution, use its exact finite-step map. For example, `dX/dt = −kX` has `X(t+Δt) = X(t) exp(−kΔt)`; a constant ambient first-order thermal relaxation has `T(t+Δt)=T∞+(T(t)−T∞)exp(−Δt/τ)`. Constant-`K` droplet evaporation gives `d²(t+Δt)=max(0,d²(t)−KΔt)` up to the physical disappearance event. These maps preserve their intended limiting behavior and avoid Euler overshoot. They are exact only for their specified constant-coefficient laws.

For coupled equations without a closed map, explicit RK methods are implementable and reviewable, but step size and stability are part of the numerical method. A solver must preserve nonnegative concentrations, bounded composition, positive diameter/temperature where required, and conservation residuals; document its integration error and timestep behavior. Use an exact linear substep plus an explicit nonlinear substep (an integrating-factor/operator-split method) only after the split terms and error order are declared. Never call a solver's state estimate a newly typed weight: laws and constants remain literal, while the numerical map is deterministic code.

## Multimechanism burner interpretation

General spherical diffusion-flame literature supports at least two distinct extinction routes. At sufficiently high flow/large flames, radiative loss can grow relative to heat release; as the flame expands, reactant supply to the flame can fall as well, so radiation and supply decline interact. At low flow near the burner, high local strain can produce kinetic/convective extinction. Flame oscillations and partial extinction may precede complete extinction. Burner heating or a hardware temperature ceiling can end an experiment without being flame extinction. Thus a single monotone “critical size” law or a single event label is structurally incomplete; retain a competing-mechanism classification and separate flame state from experiment termination.

For fabric, oxygen transport limitation is a physically motivated candidate control near quenching, but oxygen availability alone does not set propagation. Gas-side oxygen delivery competes with surface fuel production, solid preheating, conduction/radiation loss, and the geometry of the flame/flow passage. A control-volume ratio `Φ_O2` gives a conservation-based diagnostic and a reusable dimensionless coordinate; it does not by itself supply a validated spread law. Do not encode any BASS-II per-run result from the fabric papers into that ratio or choose a capture width by matching it.

## Recommended shared block shape

Use a compact, versioned source record per substance/property family:

```text
species: stable chemical identity
quantity: typed property name
source_value: raw number and source unit, or coefficient vector
validity: closed temperature/pressure/composition domain
equation: source equation identifier and exact parameter convention
uncertainty/status: source uncertainty, evaluated/estimated/unverified
locator: direct source URL plus table/page/row
derived: optional target unit and explicit conversion derivation
```

Suggested first increments:

- **Fuel thermochemistry:** methanol and n-heptane molar masses, formula-derived stoichiometry, vapor-pressure coefficients with domains, saturation density/`Cp`/conductivity tables, latent heat and gas-phase heat capacity with phase and validity explicit.
- **Common gases:** N₂, O₂, H₂O, CO₂, He molecular masses and vetted `Cp(T)`/transport correlations; avoid mixing dry-air and nitrogen values under a generic “gas” name.
- **Binary diffusion:** explicit ordered species-pair keys, reference pressure, temperature domain, correlation/measurement type, uncertainty, and source. Separate “measured/evaluated” from Fuller-only estimates. The NIST-hosted McGivern–Manion paper is a good direct block for ethane/propane/n-butane in N₂ or He over 300–723 K; it does not cover methanol or heptane.
- **Fabric materials:** cotton/fiberglass mass fractions, fabric thickness/areal density, solid heat capacity, emissivity, and pyrolysis parameters only when a source supports the actual blend and domain. Cotton-only cellulose constants are not automatically blend constants.

Keep apparatus properties in a separate, named apparatus block. Keep model-level uncertainty or priors outside the property table; uncertainty must not be represented as a fabricated “property.” If two canonical sources disagree, preserve both records and make the selection rule explicit.

## Sources consulted and extent

Classes use the dossier convention: A = free science/property/other-campaign experiment; B = campaign-general apparatus/method source; C = mixed mechanism and modeled-campaign outcome source; D = per-test outcomes. One class C abstract-level aggregate summary was visible as disclosed below; no class D per-test outcomes or archive-partition data were consulted.

| Class | Source / direct URL | Extent read and use | Not verified |
|---|---|---|---|
| A | [NIST Chemistry WebBook, n-heptane Antoine parameters](https://webbook.nist.gov/cgi/cbook.cgi?ID=C142825&Mask=24) | Whole Antoine table and references; checked coefficient convention, pressure unit, and the 299.07–372.43 K range. | Underlying 1945 measurements and source uncertainty were not reopened. |
| A | [NIST Chemistry WebBook, methanol Antoine parameters](https://webbook.nist.gov/cgi/cbook.cgi?ID=C67561&Mask=4&Type=ANTOINE) | Whole Antoine table and references; checked 288.1–356.83 K low-temperature equation and saw separate higher-temperature parameterization. | The underlying Ambrose–Sprake measurements were not reopened; no extrapolation was made. |
| A | [NIST Chemistry WebBook, n-heptane identity and molar mass](https://webbook.nist.gov/cgi/cbook.cgi?Name=n-heptane) and [methanol identity and molar mass](https://webbook.nist.gov/cgi/cbook.cgi?Name=Methanol) | Read formula and molecular-mass entries: 100.2019 and 32.0419 g/mol. | Atomic-weight uncertainty was not propagated into derived stoichiometric ratios. |
| A | [NIST WebBook fluid table, n-heptane saturated states](https://webbook.nist.gov/cgi/fluid.cgi?Action=Load&ID=C142825&Type=SatP&TLow=300&THigh=370&TInc=10&Digits=6&Units=SI) | Queried 300–370 K saturation table; read the 300 K liquid row and its pressure, molar density, molar heat capacity, and conductivity. | No independent EOS fit reconstruction; no uncertainty propagation. |
| A | [NIST WebBook fluid table, methanol saturated states](https://webbook.nist.gov/cgi/fluid.cgi?Action=Load&ID=C67561&Type=SatP&TLow=300&THigh=340&TInc=10&Digits=6&Units=SI) | Queried 300–340 K table; read the 300 K liquid row and corresponding state columns. | No independent EOS fit reconstruction; no uncertainty propagation. |
| A | [NIST Chemistry WebBook guide](https://webbook.nist.gov/chemistry/guide/) | Read the portions describing gas-phase heat capacity and Shomate coefficients; used to define a source-faithful property-block convention. | Not every species coefficient was transcribed in this note. |
| A | [McGivern & Manion, “Hydrocarbon binary diffusion coefficient measurements for use in combustion modeling” (2012), NIST copy](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=909700) | Read article: abstract, methods, Tables 1–2, pressure/temperature discussion, and conclusions. Confirmed pair coverage (C₂–C₄ in He/N₂), `D=aT^m`, and the stated 300–723 K span; did not use their tables to fit any IGNIS model number. | No heptane or methanol pair in this paper. The authors note temperature extremes and analyte-specific limitations. |
| A | [Marrero & Mason, “Gaseous Diffusion Coefficients” (1972), JPCRD reprint](https://srd.nist.gov/jpcrdreprint/1.3253094.pdf) | Bibliographic/source-level check; used as the evaluated-data route for gas-pair properties in the proposed block. | I did not re-transcribe any coefficient table here; specific pairs need a separate table-level review before use. |
| A | [A Heat Transfer Textbook, §11.5 excerpt](https://bpb-us-e1.wpmucdn.com/sites.mit.edu/dist/6/1810/files/2024/04/AHTTv600.pdf) | Read the Stefan evaporation tube derivation shown in §11.5, including `N_A = cD/L ln[p/(p−p_sat)]` and conversion from molar flux to liquid mass flux. Used as an independent check of the molar logarithm and molecular-mass factor. | I did not read the whole textbook; its planar-tube geometry is not itself the spherical-droplet solution. |
| A | [“Detailed simulations of droplet evaporation,” §2.1 (open thesis)](https://www.diva-portal.org/smash/get/diva2%3A1157150/FULLTEXT01.pdf) | Read pp. 7–8, Eqs. 2.10–2.15: Maxwell dilute limit, Stefan-flow form, Spalding mass-fraction definition, reference state, and limits of spherical quasi-steady assumptions. | Did not review the thesis's experimental comparisons or use its fitting results. |
| A | [Santa et al., “Radiative extinction of gaseous spherical diffusion flames in microgravity” (2007)](https://terpconnect.umd.edu/~pbs/2007%20Santa%20et%20al.%20CNF.pdf) | Read abstract and paper summary/discussion returned by the source; used only the mechanism distinction: radiation grows while reactant supply/heat release can fall with flame expansion, and low-flow kinetic/strain extinction is distinct. | No per-test outcome transcribed; numerical predictions, individual figures, and any applicability to the current ACME setup were not checked. |
| A | [Olson, “Mechanisms of Microgravity Flame Spread Over a Thin Solid Fuel: Oxygen and Opposed Flow Effects” (1991)](https://www.tandfonline.com/doi/abs/10.1080/00102209108951711) | Abstract only; used as a mechanism pointer that oxygen-transport-limited reaction is proposed near quenching. | Full paper and derivation were not accessible in this pass; no numerical law adopted. |
| B | [NASA ACME on the ISS poster, NASA NTRS](https://ntrs.nasa.gov/api/citations/20210015913/downloads/PS-03403%20ACME%20on%20the%20ISS.pdf) | Read the s-Flame goals and “selected findings” section only; supports separating radiative from kinetic extinction and recognizing oscillations and burner thermal limits. | I did not inspect cohort data, per-test figures, or use its outcomes as coefficients. |
| C | [Zhao et al., “Concurrent flame growth, spread and extinction over composite fabric samples in low speed purely forced flow in microgravity,” NASA NTRS PDF](https://ntrs.nasa.gov/api/citations/20150008962/downloads/20150008962.pdf) | This is a mixed BASS/BASS-II cohort source, not a campaign-general B source. Read the metadata/title, full abstract, and opening introduction/setup pages (PDF pp. 1–2, lines 0–76); the abstract includes an aggregate cohort summary, which I saw and disclose, but did not record or use. Stopped before detailed results. The setup description is relevant only to identifying a composite fabric, not to predicting it. | No per-test result, run outcome, result table, or result figure was consulted. No cohort outcome supports the proposed oxygen-balance law. |
| A | [Hairer & Wanner, “Stability of Runge–Kutta Methods,” Chapter 10](https://webspace.science.uu.nl/~frank011/Articles/CMDS.pdf) | Read chapter sections 10.3–10.3.2 around the test equation and absolute-stability function; used to support step-size/stability requirements for explicit integration. | No implementation or stability-region calculation for an IGNIS forward was performed. |

## Not verified / explicit limits

- The NIST fluid values above were spot-queried, not exhaustively checked against every handbook or source equation. The values are NIST's standard reference-data output; their primary experimental basis was not independently repeated.
- No direct methanol–N₂ or heptane–N₂ diffusion table was re-read here. `properties_verified.md` reports Tang et al. measurements for methanol–N₂ and heptane–N₂; treat them as prior-audit evidence until the exact paper tables and uncertainty are independently reopened before a model consumes them.
- Fuller-only CO₂/He pairs, extrapolation above source temperature ranges, mixture thermal conductivity, high-temperature species transport, cotton/fiberglass effective properties, and cellulose pyrolysis constants remain unsupported for a shared “verified” block.
- The fabric oxygen budget needs its control-volume geometry, local mixture state, oxygen capture area, and fuel-consumption definition. Nominal fan speed by itself is not an oxygen flux.
- A full resolved chemistry/transport solver is not defended by these sources alone. Start with closed-form submodels and conservation tests; introduce a PDE or coupled reduced solver only when its boundary conditions, properties, and numerical error budget are documented.
- No model candidate, typed weight, partition file, or run-derived value was created or inspected for this note. The abstract-level aggregate disclosure above is not a per-test result and was not used in the analysis.

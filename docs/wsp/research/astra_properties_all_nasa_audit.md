# Astra property audit for the all-investigation expansion

Research only, 2026-11-14. Author role: GPT-6 Astra property researcher; no model values, patches, weights, or qualification decisions authored. This audit covers the existing 374-entry property reference table and additional pure-substance, solid, and particle requirements. It does **not** establish experimental coverage of all 19 investigations.

## Finding and scope

All **374 rows replay exactly, including byte identity**. They contain **352 distinct property/value/source tuples**, so 22 row instances repeat such tuples. This is reproducibility of the previous computation, not 374 independent empirical confirmations. Some entries are missing-value statements, apparatus assumptions, derived dimensionless groups, or evaporation/pyrolysis closures. The original claim “property check is complete” needs that qualification.

The strongest additional checks are separate calorimetry lineages for heptane heat capacity, primary transport formulations, direct hydrocarbon diffusion measurements, and a primary-paper trace that exposes a serious conflict in a newer diffusion compilation. There is no defensible blanket statement that the 150 fluid-database rows have been independently validated.

**Exposure declaration:** the expressly permitted `docs/wsp/dossier/ASTRA_PROPERTIES.md` contained prior train-informed/model-choice discussion. This researcher is therefore not clean room with respect to historical development knowledge. No linked feedback was followed, and no outcomes are reproduced here. The permitted previous property report and its reference CSV also contain comparisons to dossier constants; these are treated as provenance to audit, not as authority for new physical properties. No holdout, sealed validation, atlas rows, model artifacts, or investigation outcome paper was opened. Searches used substance/property names, not investigation-result queries.

## Reproducible evidence

The coordinator specifically authorized the property-only input `runs/dev/research7/properties/properties.csv`, its producer `runs/dev/research7/properties/compute_properties.py`, and the producer's exact 20 NIST fluid/ITS-90 reference inputs. Producer inspection established that these inputs are thermophysical/reference-function tables, not experimental investigation observations. The producer's derived closure rows are still computations rather than measurements.

The comparison script is `tools/wsp/astra/property_audit_all.py`. It writes only below `runs/dev/data_astra/research/property_audit/`:

- `row_ledger.csv`: every P001–P374, preserving property, substance, conditions, value, unit, source URL, and original computation description; adds an explicitly inferred phase classification and source-family audit. Every entry defaults to **independent empirical provenance not established**; the scoped checks below are the only exceptions/qualifications.
- `audit_receipt.json`: SHA-256 and byte count of the auditor, producer, original CSV, all 20 inputs, replay outputs, and ledger; executed arithmetic and replay comparison.
- `replay/`: isolated original-producer output; `replay_stdout.txt` preserves execution output.
- `sources/` and `source_manifest.json`: permitted property reference bytes, exact requested/final URLs, retrieval UTC, content type, SHA-256, and explicit download errors. A failed download is not silently counted as archived evidence.

Original CSV SHA-256: `e1ed76906ec39a6da8491c61134c05d8bf3aeec4872c64deb0fe1a95c9f98b9d` (132977 bytes). Producer SHA-256: `52ab91733c4c91a10f1987d24feb7aa74659e09076e9d7ba74f097195e6e299a` (76596 bytes). The receipt, rather than a mutable branch name, identifies the exact audit code and input versions.

Twenty-one reference responses were archived successfully. The Vogel publisher HTML/PDF downloads returned challenge content to the script; they are explicitly unsuccessful in the final manifest, although the paper text and PDF tables were accessible through the web reader. An earlier saved challenge HTML is not scientific evidence. The web-read paper is therefore URL/version identified but has no successfully archived publisher PDF in this run.

```powershell
# Offline replay and comparison; no property fitting or model authoring:
python tools/wsp/astra/property_audit_all.py
# Explicit network execution, fixed property-reference URL allowlist:
python tools/wsp/astra/property_audit_all.py --execute
```

The authorized input names are `heptane_satT1.tsv`, `heptane_satT2.tsv`, `C142825_satP.tsv`, `heptane_sat204.tsv`, `heptane_sat298.tsv`, `heptane_isobar.tsv`, `methanol_satT1.tsv`, `C67561_satP.tsv`, `methanol_sat298.tsv`, `methanol_isobar.tsv`, `water_satT.tsv`, `C7732185_satP.tsv`, `water_sat298.tsv`, `h2o_isobar.tsv`, `n2_isobar.tsv`, `o2_isobar.tsv`, `co2_isobar.tsv`, `he_isobar.tsv`, `type_k.tab.txt`, and `type_b.tab.txt`, all under the original producer's `sources/` directory. No directory-wide raw-data discovery was needed.

## Complete entry census and unchecked coverage

The machine-readable ledger is the authoritative full value/unit/condition transcription. This index accounts for all 374 entries, including entries not independently checked. Adjacent rows are not assumed to share identical conditions.

| Entries | Content | What this audit establishes; what remains unchecked |
|---|---|---|
| P001–P018 | Heptane boiling and vapor pressure | Replay; Antoine domain inspected; no independent pointwise vapor-pressure dataset audit |
| P019–P038 | Heptane liquid density, Cp, conductivity, diffusivity, expansion, heat-up arithmetic | Separate calorimetry/formulation checks below; density and derivatives remain same-source replay; heat-up time is a closure |
| P039–P058 | Heptane latent/combustion heat, stoichiometry, gas Cp/k | Stoichiometry independently recalculated; phase definitions retained; latent/combustion/gas-property entries not independently remeasured or fully traced; P058 polynomial unsupported |
| P059–P095 | Small-molecule binary diffusion and temperature exponents | Replay; N2–CO2 primary trace and water-pair comparison below; all other Marrero coefficients remain independently unchecked |
| P096–P112 | Heptane/methanol/water diffusion estimates | Tang/Fuller lineage distinguished; no new direct fuel–He/CO2 measurements established |
| P113–P141 | Ethane/propane/butane diffusion and Lewis numbers | McGivern full-paper read, measured-versus-fit distinction; Lewis numbers remain compound calculations |
| P142–P155 | Oxygen Lewis number, nitrogen thermal transport, burner Peclet calculations | Replay; no apparatus validation; room-temperature transport formulation check only |
| P156–P193 | N2/O2/CO2/He/water/methanol gas Cp and k | Source branches/phases checked selectively below; high-temperature missing values remain missing |
| P194–P203 | Gas-mixture conductivity and oxygen Lewis number | Mixing-rule arithmetic only; not independent mixture measurements |
| P204–P215 | C2–C4 LHV, stoichiometry, heat per oxygen | Exact reaction arithmetic rechecked; calorimetric provenance not independently fully audited |
| P216–P226 | Radiation fits | TNF coefficient provenance checked; Smith entries remain secondary reproduction |
| P227–P235 | Hot-air transport and cellulose stoichiometry | Unit/arithmetic checks; no claim that dry air represents a reacting environment |
| P236–P251 | Cellulose gaps and fabric/burner calculations | Explicitly unsupported specimen/kinetic inputs; arithmetic replay cannot qualify these as material properties |
| P252–P279 | Type K/B thermocouple reference tables/functions | Replay against authorized ITS-90 bytes; no independent calibration, bead response, radiation-loss, or installation verification |
| P280–P285 | Missing recipe, model/apparatus quantities, liquid diffusion/VLE gaps | Remain unchecked/unsupported; no substitute numbers supplied |
| P286–P288 | Alternative conductivity mixing rules | Arithmetic comparison, not independent mixture-property validation |
| P289–P325 | Methanol phase/liquid/combustion properties | Antoine domain conflict, conductivity check, stoichiometry below; remaining EOS/calorimetric values only replayed |
| P326–P340 | Water phase/liquid/vapor properties | Reference-table replay only; independent IAPWS formulation/measurement audit not completed |
| P341–P374 | Wet-bulb, evaporation, temperature sensitivities, mixture ratios | Reproduced closure outputs; no empirical property or investigation validation implied |

Source-family census: 150 NIST-fluid; 64 WebBook compilation; 43 Marrero–Mason evaluation; 28 ITS-90; 24 without a source URL; 20 McGivern–Manion; 18 Fuller estimates via Tang 2014; 10 Tang 2015; 9 TNF/RADCAL; 4 Kadoya; 2 secondary Smith; 2 other. These sum to 374. The 150 fluid rows use a generic landing URL rather than a state-specific reproducible request; the archived input bytes repair replayability, not missing empirical independence. Empty URLs are explicitly enumerated in the receipt.

## Matched-state property checks

### Heptane liquid and vapor pressure

The [NIST phase table](https://webbook.nist.gov/cgi/cbook.cgi?ID=C142825&Mask=4) lists Antoine `log10(P/bar)=A−B/(T/K+C)` with `(4.02832,1268.636,−56.199)` valid 299.07–372.43 K. P010 at 285 K and P005 near 398 K extrapolate it. A separate low-temperature branch covers 185.29–295.60 K; it does not authorize filling the intervening gap. The evaluated normal boiling point is 371.5 ± 0.3 K. Read extent: phase/Antoine tables and associated references, not every underlying experiment.

The [WebBook liquid Cp table](https://webbook.nist.gov/cgi/cbook.cgi?ID=C142825&Mask=2) attributes 300 K calorimetry of 225.14 J mol−1 K−1 to Van Miltenburg and 225.33 to Tan. With molar mass 0.1002019 kg mol−1 these become 2246.86 and 2248.76 J kg−1 K−1, compared with P021's 2254. Distinct named measurement series are visible; original calorimetry papers were not fully read and overlap with the fluid EOS fit is unresolved.

[Zábranský and Růžička 1994](https://www.nist.gov/system/files/documents/srd/jpcrd469.pdf), Tables 2–4 and sections 3–4, independently documents the evaluation lineage: NBS and Bureau of Mines calorimetry, temperature-scale conversion, and weighted spline fitting. Table 4 isobaric Cp at 300/320/340/360/370 K is 225.58/233.38/241.85/250.94/255.71 J mol−1 K−1; converted values are 2251.25/2329.10/2413.63/2504.34/2551.95 J kg−1 K−1. These broadly corroborate P021–P025's temperature dependence. Saturation heat capacity and isobaric Cp are separate columns: at 370 K they are 255.41 and 255.71 J mol−1 K−1. This is an externally fitted property reference, not fit-free first principles; shared EOS training lineage remains unresolved.

[Assael et al. 2013](https://srd.nist.gov/jpcrdreprint/1.4794091.pdf), abstract, sections 3.3–5, Table 4 and Eq. 12 read, supplies an older saturation conductivity correlation valid 191–365 K: λ = `[212.475−89.635 T/298.15]` mW m−1 K−1. It gives 0.122284 at 300 K and 0.111762 W m−1 K−1 at 335 K versus P028/P030 0.1216/0.1112. The differences are within its stated 1.5% uncertainty. It cannot check P032 at 370 K. The broader reference formulation covers triple point–600 K and pressures to 250 MPa; its measurement lineage overlaps earlier formulations, so this is formulation corroboration, not a newly independent experiment.

### Methanol branches and transport

The [NIST methanol phase table](https://webbook.nist.gov/cgi/cbook.cgi?ID=C67561&Mask=4), phase and Antoine sections read, has overlapping fits that must not be silently spliced. Inverting its bar-based equation gives:

| Published coefficient set; domain K | Tb at 2 atm, K | Tb at 3 atm, K |
|---|---:|---:|
| Ambrose/Sprake 1970; 288.1–356.83 | 356.3977, in range | 368.4410, extrapolated |
| Ambrose/Sprake 1975; 353.5–512.63 | 358.3586 | 370.5424 |
| Hirata/Suda 1967; 353–483 | 356.6223 | 368.8314 |
| Existing fluid-reference table | 356.34 | 368.40 |

P302 wrongly labels the first set's 2-atm result as extrapolation: its resulting temperature is within the stated interval. P301 at 3 atm is extrapolation. P295/P298 correctly evaluate a different in-range branch but disagree with the fluid table by about 2 K. Underlying vapor-pressure papers were not fully inspected; no branch is declared universally correct merely because it agrees with the EOS.

[Sykioti et al. 2013](https://www.nist.gov/system/files/documents/2017/01/09/jpcrdv422013_043101.pdf), abstract, sections 3.4–5 and Eq. 15 read: at 0.101 MPa, λ = `(265.67−0.2191 T)` mW m−1 K−1, valid 260–330 K, stated expanded uncertainty 2%. At 300/320 K it yields 0.199940/0.195558 W m−1 K−1 versus P316/P318 0.1998/0.1960. The pressure states are close but not identical to saturation. Do not apply this simple equation at 337 K. The full formulation extends to 660 K and 245 MPa; its simple and full fits share primary data and are not independent confirmations. The PDF's 660 K limit takes precedence over a differing landing-page title.

### Diffusion: measurements, fits, and a conflicting compilation

[McGivern and Manion 2012](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=909700), complete main text and Tables 1–2 read, measures dilute hydrocarbons in N2/He at 3.5–4 bar and normalizes to 1.013 bar. At 300 K measured ethane–N2 and propane–N2 are 0.156 ± 0.001 and 0.121 ± 0.001 cm² s−1 (one-sigma reported uncertainty); corresponding fitted `a T^m` values differ slightly. Multiply cm² s−1 by 1e−4 for SI. P133/P134's butane–N2 value at 300 K is a fit extrapolation: the 300 K measurement was omitted because of column adhesion, and its measured domain begins at 350 K. Table 1 and Table 2 from this paper are one experimental family, not two independent sources. The fits explicitly depend on measured data.

[NIST TN 2279, February 2024](https://nvlpubs.nist.gov/nistpubs/TechnicalNotes/NIST.TN.2279.pdf), introduction/equation, relevant Table 1 entries and references read, uses `ln D = A+B/T+C ln T`, D in cm² s−1 at 101325 Pa. Its heptane–air 0.072 and methanol–air 0.166 at 298.15 K reuse Tang 2015: no new independent confirmation, and air is not nitrogen. The N2–CO2 fitted coefficients yield 0.222727 at 298.15 K and 0.224902 at 300 K, materially above P066's 0.1671 at 300 K. Water–CO2 gives 0.167084 at 300 K and 1.73955 at 1000 K, compared with the older table's 0.172 and 2.148. That high-temperature disagreement remains unresolved; the newer theoretical source has a corrigendum not fully inspected.

The primary [Vogel, Bich and Hellmann 2023 paper](https://link.springer.com/content/pdf/10.1007/s10765-023-03233-y.pdf), methods, Tables 3/5/7/8, discussion and conclusions read, distinguishes two procedures. Its preferred Table 5 gives N2–CO2 zero-density `ρ_m D = 6.824e−4 mol m−1 s−1` at 298.20 K. Ideal-gas conversion `D=(ρ_m D)RT/p` yields **0.166980 cm² s−1 at 1 atm**. Table 7 gives 0.227150 after the same conversion, but the authors reject that second procedure as unsuitable. Thus TN 2279's much higher value is not safe corroboration. Similarity to the rejected value suggests a possible lineage problem; it does **not** prove a transcription error. The primary result is viscosity-derived with kinetic-theory inputs, not a direct diffusion measurement. Its 298.20 K state also differs from P066's 300 K.

Water–CO2 theory trace: [Hellmann 2019 author copy](https://www.hsu-hh.de/thermodynamik/wp-content/uploads/sites/741/2019/06/Cross-second-virial-coefficient-and-dilute-gas-transport-properties-of-the-H2O-CO2-system-from-first-principles-calculations.pdf), abstract only read; a fitted intermolecular potential underlies the calculation. [2020 corrigendum](https://doi.org/10.1016/j.fluid.2020.112624) found, full corrected content not obtained. These limitations prevent promoting its temperature fit as settled. H2O–N2 and H2O–air must remain separate: P086/P094 explicitly substitute air for N2.

All 18 Fuller estimates remain estimates, not new measured properties. The [Marrero–Mason 1972 evaluation](https://srd.nist.gov/jpcrdreprint/1.3253094.pdf) is archived, but its full 117-page contents were not read in this audit; coefficient replay cannot be described as a complete independent source audit.

### Gas transport, radiation, and exact arithmetic

[Lemmon and Jacobsen 2004](https://trc.nist.gov/refprop/FAQ/NAO.PDF), introduction, correlation discussion and Table V read, gives zero-density 300 K `(μ in μPa s, λ in mW m−1 K−1)`: N2 `(17.8771,25.9361)`, O2 `(20.6307,26.4403)`, Ar `(22.7241,17.8042)`, air `(18.5230,26.3529)`. Existing N2/O2 values at finite pressure are close, but states differ. This widely reused correlation is not an independent family merely because another library implements it. Its collision-integral/residual correlations are externally fitted. No confined-gas/slip-domain validation follows.

[TNF radiation reference](https://tnfworkshop.org/radiation/), full page read, identifies a 2003 update and a correction to earlier coefficients. P216/P217 match its CO2/H2O coefficients, including the small H2O highest-order term. These are RADCAL fits, valid 300–2500 K; absorption is m−1 atm−1 before multiplying by partial pressure in atm. The optically thin closure is an additional assumption. P225/P226's Smith coefficients remain independently unchecked because the cited source is a secondary reproduction.

Arithmetic checks, independently expressed in the audit script, recover liquid α at 300 K as 7.95818e−8 m² s−1 for heptane and 1.00033e−7 for methanol, and 1250 K dry-air ν = 1.77797e−4 and α = 2.41189e−4 m² s−1 from the rounded tabulated k, ρ, Cp, and μ. These are algebraic checks, not new measurements.

Balanced complete-oxidation reactions independently give O2/fuel mass ratios (kg/kg): heptane 3.512776, methanol 1.497982, ethane 3.724627, propane 3.628344, n-butane 3.578533, ethylene 3.421941, propene 3.421949, n-dodecane 3.475378, C15H32 identity 3.464792, cellulose repeat C6H10O5 1.184113. This does not establish mixture composition, combustion completeness, or a specimen's heating value.

## Additional pure substances: bounded property facts

These are reference facts for future author review, not a complete property vector or proposed model constants. Read extent for each WebBook page: relevant identity and thermochemistry/phase tables plus citations; original cited papers were not fully read unless stated above.

| Substance and phase | Property, condition, raw value and SI interpretation | Domain/provenance limit |
|---|---|---|
| Ethylene, ideal gas | M = 28.0532 g/mol; Cp = 43.08/93.88/109.96 J mol−1 K−1 at 300/1000/1500 K | [NIST/TRC](https://webbook.nist.gov/cgi/cbook.cgi?ID=C74851&Mask=1), evaluated temperature table; not constant Cp |
| Ethylene, gaseous reactant | ΔcH = −1411.20 ± 0.30 kJ/mol, water-liquid combustion convention; converting with 2 × 44.004 kJ/mol water phase correction gives LHV 47.1672 MJ/kg at 298 K | Same [NIST Rossini entry](https://webbook.nist.gov/cgi/cbook.cgi?ID=C74851&Mask=1); calorimetry and water-reference uncertainty not propagated here |
| Propene, ideal gas | M = 42.0797 g/mol; Cp = 64.61/144.44/169.18 J mol−1 K−1 at 300/1000/1500 K | [NIST/TRC](https://webbook.nist.gov/cgi/cbook.cgi?ID=C115071&Mask=1); separately listed Bier value 64.71 at 300 K; no full original-paper audit |
| Propene, gaseous reactant | Rossini and Wiberg combustion values −2057.7 ± 0.6 and −2057.8 ± 1.1 kJ/mol imply LHV 45.7629 and 45.7653 MJ/kg with three water-vapor corrections | [NIST calorimetry entries](https://webbook.nist.gov/cgi/cbook.cgi?ID=C115071&Mask=1); distinct named experiments, not proof of every shared auxiliary datum's independence |
| Methane, gaseous reactant | ΔcH = −890.7 ± 0.4 and −890.35 ± 0.30 kJ/mol from Pittam and Prosen entries | [NIST methane](https://webbook.nist.gov/cgi/cbook.cgi?ID=C74828&Mask=1); retain water-product phase convention; methane transport and full Cp audit remain incomplete |
| Hydrogen, ideal gas | Cp(300 K) = 28.84948 J mol−1 K−1 from Shomate | [NIST hydrogen](https://webbook.nist.gov/cgi/cbook.cgi?ID=C1333740&Mask=1); this coefficient branch covers 298–1000 K; higher-temperature branches differ |
| Nitrogen, ideal gas | Cp(300 K) = 29.12479 J mol−1 K−1 from Shomate | [NIST nitrogen](https://webbook.nist.gov/cgi/cbook.cgi?ID=C7727379&Mask=1); branch 100–500 K; reviewed/refitted thermochemical function, not new measurement |
| Helium, ideal gas | Cp = 20.78603 J mol−1 K−1 | [NIST helium](https://webbook.nist.gov/cgi/cbook.cgi?ID=C7440597&Mask=1), 298–6000 K branch; monatomic ideal-gas behavior does not supply mixture transport |
| n-Dodecane, liquid/vapor | M = 170.3348 g/mol; normal Tb = 489 ± 2 K; ΔvapH = 61.4 kJ/mol at 299 K (≈360.47 kJ/kg); 58.1 at 334 K and 57.4 at 344 K | [NIST phase table](https://webbook.nist.gov/cgi/cbook.cgi?ID=C112403&Mask=4), Viton calorimetry entries; Antoine `(4.10549,1625.928,−92.839)` valid 399.53–490.49 K, cannot supply room-temperature pressure |
| Farnesane identity, liquid/vapor | C15H32, M = 212.4146 g/mol; Tb = 525.1 K, TRC assigned uncertainty 0.5 K | [NIST CAS 3891-98-3](https://webbook.nist.gov/cgi/cbook.cgi?ID=C3891983&Mask=4), API 1962 lineage; no audited density/Cp/k/μ/pressure curve; cannot transfer to commercial blends or unspecified isomers |

Ethane/propane/n-butane already have the measured diffusion checks above; their remaining gas Cp, viscosity, and mixture transport need separate state-matched provenance. Oxygen and carbon dioxide high-temperature conductivity gaps P166/P172 are not filled by ideal-gas heat capacities. Water-vapor P182 and methanol-vapor P193 range gaps remain explicit. Surface tension, temperature-dependent viscosity, mixture diffusion, and nonideal liquid activity are additional droplet requirements not supplied by this audit.

## Solids, apparatus, and particles

[PMMA WebBook thermochemistry](https://webbook.nist.gov/cgi/cbook.cgi?ID=C87210320&Mask=2), table and citations read: 131.8 J mol-repeat−1 K−1 at 300 K (Lebedev 1971), with repeat mass 100.1158 g/mol, gives 1316.48 J kg−1 K−1. Other entries are 125.6 at 298 K and 130.8 at 298.15 K. These are specimen-dependent polymer observations, not a universal PMMA-grade constant. Molecular weight, tacticity, additives, thermal history, glass transition, and measurement protocol matter; density/k/emissivity/decomposition remain unchecked.

[Blokhin et al. cellulose thermodynamics](https://doi.org/10.1021/je200270t), publisher abstract only read, explicitly concerns crystallinity dependence. Full text/supporting tables were inaccessible. Therefore P236 stays missing; no Cp was transcribed from an abstract or third-party repost. P237's activation energy without a pre-exponential factor is not a complete kinetic law. P238–P245's arithmetic does not validate the supplied kinetic, emissivity, or areal-mass assumptions.

For cotton, composite fabric, insulation, or trade-name material, require specimen identity/composition, moisture, areal mass, thickness, porosity, weave/orientation, backing, temperature-dependent Cp and anisotropic k, absorptivity/emissivity versus wavelength, decomposition enthalpy/kinetics and residue fraction. A pure cellulose repeat-unit stoichiometry cannot establish those quantities. P282/P283 are apparatus/model-specific, not transferable material-reference entries. No invented trade-name properties are supplied.

[Kim et al. 2005 particle slip measurements](https://nvlpubs.nist.gov/nistpubs/jres/110/1/j110-1kim.pdf), abstract/introduction read, concern spherical PSL particles measured with a nano-DMA. Their empirical Cunningham expression uses Kn = 2λ/d and `Cc = 1+Kn[1.165+0.483 exp(−0.997/Kn)]`, for the tested 19.90/100.7/269 nm particles and Kn about 0.5–83. This external fit is not a soot optical or morphology model. Particle investigations need material density, complex refractive index versus wavelength, morphology/aggregation, effective versus primary diameter, mobility/aerodynamic definition, charge distribution, hygroscopicity, gas mean free path, and instrument calibration. None can be replaced by a generic “smoke property.” No investigation-specific particle outcomes were consulted.

## Interpretation limits and author handoff

Externally fitted properties remain externally fitted: Antoine/Shomate coefficients, fluid EOS and transport correlations, calorimetry splines, empirical diffusion exponents/Fuller volumes, RADCAL fits, ITS-90 reference functions, particle-slip fits, and fitted intermolecular potentials all have calibration lineage. Their permitted use as fundamental reference data does not make them first-principles or independent of each other. The script merely evaluates and compares published facts; it selects no weights.

The original molar Stefan evaporation expression is exact only within its stated idealized transport problem: binary quasisteady diffusion, stagnant inert carrier, isothermal ideal gas with suitable constant cD, equilibrium surface vapor composition, and the specified far-field composition. It is not a measured universal evaporation property. Nonisothermal transport, reacting multicomponent gas, nonideal methanol–water activity, dissolved water and transient liquid heating need separate assumptions/evidence. P284/P285 remain unresolved, and none of P341–P374 is independent experimental validation.

The property block is ready for use as **scoped evidence with explicit gaps**, not as a completed qualification claim. Before transferring a value, retain substance identity, phase, T, p, unit basis, source branch/domain, uncertainty/read extent, and shared-source lineage. Keep the methanol branch disagreement and the N2–CO2 compilation conflict visible. Specimen-dependent solids and particles, missing high-temperature transport, and unverified mixture properties remain unsupported rather than being assigned plausible defaults.

Verification actually executed: isolated 374-row producer replay and byte comparison; source downloads under explicit `--execute`; independent conversion/equation arithmetic captured in the receipt. No model, NASA experiment, holdout, or validation test was run. The report and arithmetic helper are research artifacts; original tables and model files were not changed.

The Python helper parsed successfully with `ast.parse`. `git diff --check` returned zero for the owned paths (both new files were untracked at inspection; this does not substitute for content review). No Rust/application tests were appropriate for these research-only files.

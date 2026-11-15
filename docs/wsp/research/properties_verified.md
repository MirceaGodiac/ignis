# Property values of the physics dossiers, looked up in standard sources

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. No experimental data were opened: only the property tables of the physics dossiers and standard reference sources.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

Property check is complete: 374 rows in `properties.csv`, with 49 memory-marked values each given a verdict. Most typed values hold; the main problems are temperature dependence of liquid properties, CO2-pair diffusivities at flame temperature, the Ferkul gas properties, and the form of the evaporation law itself.

The largest finding concerns the evaporation law. `K_ev = 8 ρD_F/ρ_l · ln(1+B_M)` with ρ of the ambient gas is not exact when fuel and ambient molar masses differ. The exact isothermal result is the molar form `8 M_F c D_F/ρ_l · ln(1/(1−x_s))`, which is 2.4 times higher for heptane near boiling (1.07 against 0.44 mm²/s at Tb−3) and within 9 % for methanol.

## (a) Values that differ by more than 5 %

| Dossier | Quantity | Dossier | Source value | Difference | Source |
|---|---|---|---|---|---|
| heptane | liquid c_l, used over 300–370 K | 2.24 kJ/kg/K | 2.254 at 300 K, 2.551 at 370 K, mean 2.398 | −6.6 % vs mean, −12 % at 370 K | NIST WebBook fluid |
| heptane | k_l at 370 K | 0.123 W/m/K | 0.1216 at 300 K, 0.1014 at 370 K | +21 % at 370 K | NIST fluid |
| heptane | α_l | 8.1e-8 m²/s | 7.96e-8 (300 K), 7.17e-8 (335 K), 6.46e-8 (370 K) | +13 % at 335 K, +25 % at 370 K | NIST fluid |
| heptane | heat-up e-fold coefficient | 0.31 s/mm² | 0.318 (300 K), 0.353 (335 K) | −12 % at 335 K | computed |
| heptane | β_l "from the two densities" | 1.25e-3 /K | 1.254e-3 local at 300 K, 1.395e-3 mean to boiling, 1.505e-3 at 360 K | −10 % vs mean | NIST fluid |
| heptane | O2–CO2 D at 1000 K (0.160·T^1.7) | 1.24 cm²/s | 1.41 | −12 % (−3.3 % at 300 K) | Marrero & Mason Table 13 |
| heptane | K_ev at 352 / 369 K | 0.17 / 0.40 mm²/s | 0.190 / 0.463 (ambient-density form) | −10 % / −14 % | computed |
| heptane | "ρD_F about 1.9 times higher" in helium | 1.9 | 0.93 (35 % He) | wrong quantity | computed |
| inverse | N2 ρα = 2.59e-5·(T/300)^0.7 | 6.02e-5 / 7.99e-5 at 1000 / 1500 K | 5.60e-5 / 7.08e-5 | +7.4 % / +12.9 % (+3.9 % at 300 K) | NIST fluid |
| inverse | D(fuel–N2) vs older evaluated data | 0.160 / 0.125 / 0.106 | 0.152 / 0.116 / 0.100 | +5.4 / +7.9 / +6.2 % | Tang et al. 2015 |
| bass | μ at 1250 K | 4.1e-5 Pa·s | 5.02e-5 (dry air) | −18 % | Kadoya et al. 1985 |
| bass | c_p at 1250 K | 1381 J/kg/K | 1181 (dry air) | +17 % | NIST Shomate |
| bass | ν, α at 1250 K | 1.49e-4, 2.13e-4 m²/s | 1.78e-4, 2.41e-4 | −16 %, −12 % | computed |
| methanol | O2–CO2 D | 0.156 cm²/s | 0.1655 (1.41 at 1000 K) | −5.8 % (−10 % at 1000 K) | Marrero & Mason |
| methanol | H2O–CO2 D at 1000 K (0.18·T^1.87) | 1.71 | 2.15 | −20 % (+4.6 % at 300 K) | Marrero & Mason |
| methanol | O2–He D at 1000 K (0.745·T^1.66) | 5.50 | 5.90 | −6.7 % | Marrero & Mason |
| methanol | CH3OH–N2 D | 0.153 | 0.168 ± 0.014 at 300 K | −8.8 % | Tang et al. 2015 |
| methanol | α_l at 300 K | 9.4e-8 | 1.00e-7 (9.1e-8 at 337 K) | −6 % at 300 K | NIST fluid |
| methanol | conductivity mixing rule, 35 % He | 0.0542 W/m/K | 0.0509 (Wassiljewa) | +6.6 % | computed |
| methanol | K_ev hand estimates | see (c) | see (c) | −43 % to +43 % | computed |

Notes on this table:
- The heptane dossier's two quoted densities (680, 614) give β_l = 1.43e-3, not 1.25e-3; 1.25e-3 is the local 300 K value.
- The helium "1.9" is the K_ev ratio (D ratio 1.33 times 1/0.7 from pressure), not ρD_F; the cold K_ev ratio itself recomputes to 1.82.
- The inverse and normal dossiers both agree with McGivern & Manion (fit 0.1573 / 0.1238 / 0.1056; measured 300 K points 0.156 / 0.121). The inverse dossier quotes the fit, the normal dossier the table points. The n-butane 300 K point was not reported in the paper, so 0.105–0.106 is a fit extrapolated below 350 K.
- The CH3OH–N2 value 0.153 equals the single Lugg (1968) measurement and sits inside the evaluated uncertainty.
- Ferkul's k (0.0808) and ρ (0.275) agree with dry air within 3 %; the other four are model-gas values, not air.
- Pe = 0.38 per mg/s holds only with N2 properties near 1250–1300 K (1.01 / 0.60 / 0.45 at 300 / 600 / 1000 K).

Everything else agreed within 3–5 %: all boiling points, latent heats, densities, heats of combustion, stoichiometric ratios, alkane heating values, Lewis numbers, N2 conductivity, TNF and Smith coefficients, mixture conductivities, and the methanol liquid properties.

## (b) Could not be verified

- **Heptane vapour conductivity polynomial (Yaws):** coefficients not accessible. Evaluated values are +4.8 % at 300 K and within 1.3 % at 400–600 K against NIST; no check above 600 K.
- **Law–Williams 0.4/0.6 recipe:** not found in an open source.
- **Diffusivities with no measurement found:** heptane–CO2 (0.058; Fuller 0.0568), heptane–He (0.26; Fuller 0.2465), CH3OH–CO2 (0.107; Fuller 0.133), CH3OH–He (0.61; Fuller 0.556), H2O–He (0.89; Fuller 0.850). Fuller is 22 % high for the measured H2O–CO2 pair and 2–6 % low for measured alkane–He pairs, so it cannot settle the CO2 pairs.
- **Conductivities above the tabulated range:** O2, CO2, H2O at 1500 K; methanol vapour above 620 K.
- **Cotton specific heat 1.26 J/g/K:** no standard table could be opened.
- **Pyrolysis A = 1.626e11 1/s, E = 161.44 kJ/mol:** not a standard cellulose constant; the one standard value I reached is E = 228 kJ/mol (Antal, Várhegyi & Jakab 1998, abstract only).
- **Apparatus values:** SIBAL data, burner mass and heat capacity, emissivities, 1130 K critical temperature (sources excluded by the rules).
- **Others:** water diffusivity in liquid methanol, the DECHEMA table, JANAF enthalpies in `physics.py` (no numbers quoted).
- **Smith 1982 grey-gas coefficients:** confirmed only against a second reproduction (arXiv:2411.18467), not the paper.
- **Type K / B decoder:** I did not read the dossier that wrote it; the reference EMF values and coefficients are in the CSV for comparison.

## (c) Evaporation constants and e-fold temperature

Reference rule: T_ref = T_l + (300 K − T_l)/3 (one-third rule, gas at 300 K), D_F ∝ T^1.75. Vapour pressure and ρ_l come from the NIST reference equation of state. Form A uses N2 density at T_ref; the molar form is the exact one above. For warmer gas, K scales as T_ref^0.75.

**n-Heptane, 1 atm, N2** (mm²/s; D_F–N2 = 0.0732 cm²/s at 300 K)

| T_l (K) | 280 | 285 | 300 | 320 | 330 | 340 | 352 | 360 | 368.6 | 369 |
|---|---|---|---|---|---|---|---|---|---|---|
| Form A | 0.0074 | 0.0099 | 0.0221 | 0.0553 | 0.0828 | 0.121 | 0.190 | 0.264 | 0.444 | 0.463 |
| Molar form | 0.0076 | 0.0103 | 0.0240 | 0.0666 | 0.108 | 0.175 | 0.320 | 0.510 | 1.067 | 1.129 |
| Dossier | | 0.0097 | 0.022 | | 0.08 | | 0.17 | | | 0.40 |

**Methanol, 1 atm, N2** (D_F–N2 = 0.168 cm²/s at 300 K)

| T_l (K) | 280 | 290 | 300 | 310 | 320 | 330 | 334.6 | 335 |
|---|---|---|---|---|---|---|---|---|
| Form A | 0.0132 | 0.0246 | 0.0448 | 0.0806 | 0.148 | 0.303 | 0.493 | 0.523 |
| Molar form | 0.0132 | 0.0248 | 0.0454 | 0.0824 | 0.154 | 0.322 | 0.535 | 0.568 |
| Dossier | 0.011 | 0.035 | 0.064 | 0.10 | | | | 0.30 |

**Kelvin of cooling per e-fold**

| | Heptane 1 atm | Heptane 0.7 atm | Methanol 1 atm | Methanol 0.7 atm |
|---|---|---|---|---|
| ln(1+B_M), local at Tb−3 | 10.7 | 10.5 | 7.2 | 7.0 |
| ln(1+B_M), local at Tb−10 | 22.1 | 21.2 | 13.6 | 13.1 |
| ln(1+B_M), local at Tb−20 to Tb−40 | 28–29 | 26–27 | 17–18 | 16–17 |
| ln(1+B_M), first full e-fold from Tb−3 | 22.4 | 21.2 | 11.9 | 11.5 |
| K_ev form A, first e-fold from Tb−3 (then second) | 20.6 (26.1) | 19.6 (24.4) | 11.4 (16.3) | 11.0 (15.6) |
| K_ev molar form, first e-fold (then second) | 13.0 (19.7) | 12.4 (18.5) | 10.9 (15.8) | 10.5 (15.1) |

The heptane "26 K" is right between about Tb−15 and Tb−50. It is not right at the starting point after a hot flame: the local value at Tb−3 is 11 K and the first e-fold takes 22 K (13 K in the molar form). A constant T* in `K0/(1 + t/t0)` therefore overstates how long the early rate persists.

**Ambient mixtures: K_ev as a ratio to N2 at 1 atm** (Blanc's law; helium and CO2 fuel diffusivities are Fuller estimates)

| Ambient | p (atm) | Heptane, Tb(p)−3: A / molar | Heptane, 300 K: A / molar | Methanol, Tb(p)−3: A / molar | Methanol, 300 K: A / molar |
|---|---|---|---|---|---|
| 30 % He / 70 % N2 | 1 | 1.02 / 1.27 | 1.22 / 1.27 | 1.05 / 1.27 | 1.22 / 1.27 |
| 45 % He / 55 % N2 | 1 | 1.02 / 1.46 | 1.37 / 1.46 | 1.07 / 1.46 | 1.37 / 1.46 |
| 30 % He / 70 % N2 | 0.7 | 0.96 / 1.19 | 1.70 / 1.84 | 1.00 / 1.20 | 1.79 / 1.90 |
| 45 % He / 55 % N2 | 0.7 | 0.96 / 1.37 | 1.89 / 2.12 | 1.02 / 1.39 | 1.99 / 2.19 |
| 50 % CO2 / 50 % N2 | 1 | 1.05 / 0.87 | 0.90 / 0.87 | 1.03 / 0.88 | 0.91 / 0.88 |
| N2 | 0.7 | 0.95 / 0.94 | 1.41 / 1.45 | 0.95 / 0.95 | 1.49 / 1.50 |
| N2 | 2 | 1.13 / 1.15 | 0.51 / 0.49 | 1.11 / 1.11 | 0.48 / 0.48 |
| N2 | 3 | 1.22 / 1.25 | 0.34 / 0.33 | 1.18 / 1.19 | 0.31 / 0.31 |

The two forms disagree on helium and CO2 for a hot droplet. With ambient density, helium appears to do nothing near boiling (ρ falls as fast as D rises); the molar form gives +27 % and +46 %, the plain D_mix ratio. For CO2 they disagree in sign. Pressure at fixed Tb−3 changes little; at fixed 300 K, K goes as 1/p. Absolute values are in `kev_mixtures.csv`.

## (d) Property block proposal

| Constant | Value | Unit | Source |
|---|---|---|---|
| **n-Heptane** M | 100.202 | g/mol | NIST WebBook |
| σ (O2/fuel) | 3.513 | kg/kg | stoichiometry |
| heat of combustion, liquid → H2O(g) | 44.56 | MJ/kg | NIST ΔfH (Prosen & Rossini) |
| Antoine, log10 p[bar] = A − B/(T+C), 299–372 K | 4.02832, 1268.636, −56.199 | K | NIST WebBook |
| Tb at 0.7 / 1 / 2.04 / 3 atm | 359.85 / 371.55 / 397.85 / 413.97 | K | NIST fluid |
| p_sat at 285 / 300 / 330 / 352 / 369 K | 0.0301 / 0.0660 / 0.2452 / 0.5423 / 0.9273 | atm | NIST fluid |
| L at 298.15 K / Tb(0.7 atm) / Tb(1 atm) / Tb(2.04 atm) | 365.0 / 324.9 / 316.7 / 296.8 | kJ/kg | NIST fluid |
| ρ_l at 300 / 320 / 340 / 360 K / Tb | 677.9 / 660.7 / 642.9 / 624.5 / 613.5 | kg/m³ | NIST fluid |
| c_l at 300 / 320 / 340 / 360 / 370 K | 2.254 / 2.333 / 2.417 / 2.505 / 2.551 | kJ/kg/K | NIST fluid |
| k_l at 300 / 335 / 370 K | 0.1216 / 0.1112 / 0.1014 | W/m/K | NIST fluid |
| β_l mean 300 K → Tb | 1.395e-3 | 1/K | NIST fluid |
| vapour c_p at 300 / 500 / 800 / 1000 K | 1.656 / 2.516 / 3.416 / 3.808 | kJ/kg/K | NIST (Scott 1974) |
| vapour k at 300 / 400 / 500 / 600 K | 0.0121 / 0.0220 / 0.0332 / 0.0456 | W/m/K | NIST fluid |
| D heptane–N2, 300 K, 1 atm, ∝ T^1.75 | 0.0732 | cm²/s | Tang et al. 2015 (55 ± 2 Torr cm²/s at 298 K) |
| **Methanol** σ | 1.498 | kg/kg | stoichiometry |
| heat of combustion, liquid → H2O(g) | 19.92 | MJ/kg | NIST ΔcH (Rossini) |
| Antoine, 288–357 K | 5.20409, 1581.341, −33.50 | bar, K | NIST WebBook |
| Tb at 0.7 / 1 / 2 / 3 atm | 328.83 / 337.63 / 356.34 / 368.40 | K | NIST fluid |
| L at those Tb; at 298.15 K | 1118 / 1101 / 1062 / 1034; 1169 | kJ/kg | NIST fluid |
| ρ_l at 300 / 340 K | 784.5 / 746.0 | kg/m³ | NIST fluid |
| c_l at 300 / 320 / 337 K | 2.546 / 2.684 / 2.820 | kJ/kg/K | NIST fluid |
| k_l at 300 / 337 K | 0.1998 / 0.1928 | W/m/K | NIST fluid |
| vapour c_p (ideal gas) 300 / 900 / 1500 K | 44.17 / 84.95 / 105.40 | J/mol/K | NIST (TRC) |
| D methanol–N2, 300 K | 0.168 ± 0.014 | cm²/s | Tang et al. 2015 |
| **Water** L at 298.15 / 334 / 373.12 K | 2.442 / 2.356 / 2.256 | MJ/kg | NIST fluid (IAPWS) |
| **Diffusion** O2–N2 | 1.13e-5·T^1.724 (0.2107 at 300 K) | cm²/s, atm | Marrero & Mason |
| O2–CO2 | 1.56e-5·T^1.661·exp(−61.3/T) (0.1655) | cm²/s | Marrero & Mason, 287–1083 K |
| O2–He | 4.37e-5·T^1.710 (0.752) | cm²/s | Marrero & Mason |
| H2O–N2 | 0.187e-5·T^2.072 (0.254), 282–373 K; 2.75e-5·T^1.632 above 450 K (air) | cm²/s | Marrero & Mason |
| H2O–CO2 | 9.24e-5·T^1.5·exp(−307.9/T) (0.172) | cm²/s | Marrero & Mason, 296–1640 K |
| ethane / propane / n-butane–N2 | 8.63e-6·T^1.720 / 6.68e-6·T^1.723 / 5.76e-6·T^1.721 | cm²/s | McGivern & Manion 2012, 300–723 K |
| **N2** k at 300 / 600 / 1000 / 1500 K | 0.02597 / 0.04484 / 0.06536 / 0.08801 | W/m/K | NIST fluid |
| N2 k/c_p at 300 / 1000 / 1500 K | 2.494e-5 / 5.60e-5 / 7.08e-5 | kg/m/s | NIST fluid |
| N2 α at 300 K, 1 atm | 0.2191 | cm²/s | NIST fluid |
| **Alkanes** LHV ethane / propane / n-butane | 47.51 / 46.33 / 45.72 | kJ/g | NIST ΔcH |
| σ ethane / propane / n-butane | 3.725 / 3.628 / 3.579 | g/g | stoichiometry |
| **Radiation** TNF CO2 c0..c5 | 18.741, −121.310, 273.500, −194.050, 56.310, −5.8169 | m⁻¹ atm⁻¹ | TNF page |
| TNF H2O c0..c5 | −0.23093, −1.12390, 9.41530, −2.99880, 0.51382, −1.86840e-5 | m⁻¹ atm⁻¹ | TNF page |
| **Dry air at 1250 K** k, μ, c_p, ρ | 0.0804, 5.02e-5, 1181, 0.2824 | W/m/K, Pa·s, J/kg/K, kg/m³ | Kadoya 1985; Shomate; ideal gas |
| **Type K** EMF at 100 / 500 / 1000 °C | 4.096 / 20.644 / 41.276 | mV | NIST ITS-90 |
| **Type B** EMF at 600 / 1000 / 1400 / 1800 °C | 1.792 / 4.834 / 8.956 / 13.591 | mV | NIST ITS-90 |

Not safe to type as they stand: the Fuller-only pairs with He and CO2, and any conductivity above the tabulated ranges. For methanol vapour, use the ideal-gas c_p: the NIST saturated-vapour value at 300 K is 3.66 kJ/kg/K, 2.7 times the ideal-gas 1.38. The ITS-90 reference-function coefficients are in the CSV and reproduce the table to 0.0005 mV.

Two standard sources disagree on methanol's boiling point at 3 atm: the wide-range NIST Antoine set (353.5–512.6 K) gives 370.5 K, the reference equation of state 368.4 K. The dossier's own Antoine set, extrapolated, gives 368.44 K, so keep it.

## (e) Sources and how much of each I read

- **NIST Chemistry WebBook:** thermochemistry and phase-change pages for heptane, methanol, water in full; gas-phase pages for ethane, propane, n-butane, CO2, N2, O2, CO. Fluid tables downloaded for heptane, methanol, water (saturation), and N2, O2, CO2, He, H2O, heptane and methanol (1 atm isobars).
- **Marrero & Mason 1972:** Tables 12 and 13 read from the page image; equations 4.3-1 and 4.3-2 and the Table 16 listing from the text layer. No heptane or methanol correlations exist there.
- **McGivern & Manion 2012:** full text; Tables 1 and 2 and sections 4.2–4.3 used.
- **Tang et al. 2015 supplement:** sections 1.1, 1.2, 1.5, 4.1. **Tang et al. 2014:** section 2.1.1 and Table 1 (Fuller equation and volumes, reproducing Reid/Poling Table 11-1). The CO2 volume 26.9 was seen only in a search snippet.
- **Kadoya et al. 1985:** Tables 8 and 12, 0.1 MPa rows for 700–2000 K, from page images.
- **TNF radiation page:** whole page.
- **NIST ITS-90 database:** type K and type B table files, including coefficient blocks.
- **arXiv:2411.18467:** Table 7 only (Smith coefficients).
- **Antal, Várhegyi & Jakab 1998:** abstract only.
- **Not reached:** Poling directly, Yaws, CRC, Turns/Law/Glassman, Lugg 1968 (via Tang only), Smith 1982 primary, Blokhin 2011, Bradbury 1979.

No file under `data/` or `holdout` was opened, no model or evaluation tool was run, and no existing file was modified.

## (f) Files written

All under `D:\dev\SpaceApps-Ardealu\runs\dev\research7\properties\`:

- `properties.csv`
- `memory_marked_values.csv`
- `kev_tables.csv`
- `efold_tables.csv`
- `kev_mixtures.csv`
- `compute_properties.py` (regenerates the four CSVs above from `sources\`: `python -I compute_properties.py sources .`)
- `make_memory_list.py` (writes `memory_marked_values.csv`)
- `sources\` (20 files: the NIST fluid tables and the two ITS-90 tables)

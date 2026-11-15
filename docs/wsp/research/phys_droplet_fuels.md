# The droplet law carried to the fuels of CFI and FLEX-2

> Written by claude-fable-5-2 on 15 November 2026, as physicist and as author of the candidates
> under `configs/models/flex/wsp5/<fuel>/w0-fable/`. Nothing here is fitted and no program computed a
> number: every derivation below is arithmetic on property values, done by hand to two or three
> figures, so that a reader can redo it.
>
> **What was read.** The droplet forwards `droplet6.rs` and `droplet3.rs`, their cards, the edits of
> the candidates under `configs/models/flex/wsp5/` and `wsp2/`, the physics dossiers
> `phys_flex_heptane.md` and `phys_flex_methanol.md`, `lit_droplets.md`, `properties/properties.csv`,
> `PROTOCOL.md` sections 1 to 5, 10, 11.1 and 11.2, Astra's `ASTRA_FLEX_SOURCE_BOUND_RESULTS.md` and
> `droplet-liquid-pending.json`. **These carry numbers typed after reading FLEX train data, and
> aggregate validation tables of the FLEX candidates; every number taken from them is marked D
> below.** The module headers of `droplet7.rs` and `thermal.rs` (Astra's seventh forward) were
> skimmed to avoid a name clash; nothing was taken from them.
>
> **What was read of CFI (PSI-39) and FLEX-2 (PSI-68).** Conditions only: the `nasa_text`, `tests`
> and `how_tests_differ` blocks of the two atlas cards, their sections of `atlas/INVESTIGATIONS.md`,
> the list of instruments, and for PSI-68 the statements of its narrative, which come from pre-flight
> documents. The `findings` of the PSI-39 narrative were **not** read (they draw on analysed data and
> the final report). One thing seen is outcome-adjacent and is disclosed: the cards list the *names*
> of the CFI outcome columns and how many cells each holds (hot rate 148, cool rate 127, cool
> extinction diameter 122, a column called `reignitions`). That says cool flames were common in CFI
> and that hot flames reignited often enough to have a column. No value was seen.
>
> **Not opened.** `runs/dev/data_all/`, `holdout/`, `runs/dev/holdout_ext/`, any processed or
> analysed file or test sheet of PSI-39 or PSI-68, `docs/wsp/feedback/cfi_flex2_transfer_feasibility.md`.
>
> **Literature.** `lit_droplets.md` quotes published aggregate results for n-decane and n-octane
> droplets on the ISS (fuels of FLEX-2) and one remark on n-dodecane. Each one used here is cited and
> marked L. The author's pretraining may hold more published CFI and FLEX-2 results; that cannot be
> measured, and where a recollection was used it says "recalled, not re-read".
>
> **Looked up today.** NIST Chemistry WebBook pages (phase change and condensed-phase
> thermochemistry) for n-octane, 2,2,4-trimethylpentane, n-decane, n-dodecane, ethanol and
> 1-propanol, through a fetch tool that returns a summary of the page; one web search for farnesane.
> Values marked [mem] are handbook numbers recalled and not re-downloaded.

Origin classes: **P** property table, **T** theory, **L** published experiment, **J** judgement,
**D** carried from a number typed after FLEX train data.

## 0. The result in one page

| Fuel | Flown in | Existing forward | Candidate written | Why not, or what is lost |
|---|---|---|---|---|
| n-heptane | FLEX-2 | yes | none needed: `flex_heptane.wsp2` D2 (carried) and `flex_heptane.wsp5` apply as they are | helium as the only diluent (79 % He) is outside the declared domain |
| n-octane | FLEX-2 | yes | `wsp5/octane/w0-fable` | |
| iso-octane | FLEX-2 | yes | `wsp5/isooctane/w0-fable` | |
| n-decane | FLEX-2 | yes | `wsp5/decane/w0-fable` | |
| ethanol | FLEX-2 | yes (the methanol shape, no cool flame) | `wsp5/ethanol/w0-fable` | |
| n-dodecane | CFI | the main path, yes | `wsp5/dodecane/w0-fable` | no reignition of the hot flame, no separate warm-flame stage, no cool flame lit directly by the igniter, no fibre; 45 s horizon; refuses above 4.4 atm |
| farnesane | CFI | the main path, yes | `wsp5/farnesane/w0-fable` | as n-dodecane; half of its property values are estimates |
| n-heptane + iso-octane | FLEX-2 | no | | a composition input is needed (section 3.2) |
| n-dodecane + iso-dodecane | CFI | no | | composition input, mild staging (section 3.2) |
| 1-propanol + glycerol | FLEX-2 | no | | staged burning, flame contraction, disruption (section 3.2) |

The method. The absolute burning rate of a FLEX-size droplet is not predictable from property
tables: classical theory gives 0.95 mm²/s for n-heptane in air where 3 mm droplets burn at 0.55 to
0.60 (heptane dossier, section 1.2), and it misses the methanol-to-heptane ratio by a factor 1.6
(section 2.3 below). What theory does give is the **ratio between two fuels of one chemical
family**. So every fuel-specific number is the heptane number (for alkanes) or the methanol number
(for alcohols), both class D, multiplied by a ratio computed from property tables with one formula
per tensor. Gas-side coefficients (helium, CO2, the form of the laws, the encoder, the filter) are
carried unchanged. Each new fuel then has about twenty numbers of its own.

Registered burning-rate constants, air, 1 atm, D0 = 3 mm, after heat-up (section 4 has the rest):

| | n-heptane (D) | n-octane | iso-octane | n-decane | n-dodecane | farnesane | methanol (D) | ethanol |
|---|---|---|---|---|---|---|---|---|
| K hot, mm²/s | 0.58 | 0.54 ± 0.05 | 0.61 (+0.04, −0.09) | 0.49 ± 0.06 | 0.44 ± 0.07 | 0.41 ± 0.08 | 0.51 | 0.51 ± 0.05 |
| K cool, mm²/s | 0.345 | 0.32 ± 0.06 | none sustained | 0.28 ± 0.06 | 0.24 ± 0.06 | 0.23 ± 0.07 | none | none |
| D_crit, mm | 3.7 | 3.9 ± 1.0 | 3.5 ± 1.0 | 4.3 ± 1.1 | 4.7 ± 1.4 | 5.0 ± 1.5 | 4 to 9 | 4.0 ± 1.4 |

## 1. What is universal and what belongs to a fuel

### 1.1 The statements

**Quasi-steady burning.** A droplet inside a spherical flame loses D² linearly:
`K = 8 (λ/c_p) ln(1 + B) / ρ_l`. The form is universal. `λ/c_p` is a property of the hot gas between
droplet and flame, a mixture of fuel vapour and ambient gas: within the alkanes it is the same to a
few per cent (the vapours have the same heat capacity per kilogram and nearly the same conductivity),
within the alcohols it falls slowly with chain length. `ρ_l` is the fuel's.

**The transfer number.** `B = [Q Y_O/ν − c_pg (T_b − T∞)] / L_eff`. Q/ν, the heat released per
kilogram of oxygen, is 12.7 MJ/kg for every alkane here and 12.8 to 13.3 for the alcohols: nearly
universal. What separates fuels is the denominator. For a droplet of millimetres the liquid is still
being heated while it burns, so the heat each kilogram costs is not the latent heat alone but
`L_eff = L(T_b) + c_l (T_b − T_0)` (every kilogram must be brought to the boiling point before it
leaves; exact as a lifetime average, an underestimate early). This is the one idea that carries the
law from n-heptane (c_l ΔT / L = 0.56) to n-dodecane (1.92): the heavy alkanes burn slower although
their latent heat is lower.

**Radiative extinction of a large droplet.** Heat release goes as ρ_l K D, the radiating volume as
the cube of the flame diameter, and the flame stands off by a ratio proportional to ρ_l K (the fuel
flux against the oxygen supply, which is the same for all alkanes). So the radiated fraction goes as
(ρ_l K D)² κ/Q-terms and the critical diameter as `D_crit ∝ sqrt(χ_c Q G_o³/κ) / (ρ_l K)`, with
`G_o = ρD_O ln(1 + Y_O/ν)` the oxygen conductance, κ the Planck-mean absorption of the products and
χ_c the fraction of its heat a flame can lose before its chemistry fails. The form, the oxygen, CO2
and helium slopes of D_crit and the clock on which the flame is lost are universal; inside a family
only `ρ_l K` moves. Soot is not in this statement and is fuel-specific (section 5).

**Three states.** Hot flame, cool flame, no flame. The hot flame and the unlit droplet exist for
every fuel. The cool flame needs low-temperature chain branching: strong for normal alkanes from C7
up, weak for highly branched alkanes (iso-octane, iso-dodecane), absent for methanol and ethanol.
Whether a fuel has it is chemistry and cannot be read from a property table; the cetane number is
the nearest tabulated proxy. Given that a cool flame exists, its rate is again a transfer-number law
(`K_c = 8(λ/c_p) ln(1 + c_p(2T_f − T_s − T∞)/L_eff)/ρ_l` with T_f = 700 K, Williams and Nayagam
2019), independent of oxygen, and its end is a residence-time limit.

**Helium and CO2.** Both act on the gas: helium raises the conductivity and the Lewis number of
oxygen, CO2 lowers the oxygen mass fraction at equal mole fraction and adds an emitter. None of that
knows the fuel. The coefficients are carried (D).

**What is not universal and not in any table:** the absolute level of K (soot, radiation, igniter),
the oxygen limit of the cool flame, the water a miscible alcohol takes up, disruption.

### 1.2 Every tensor of `droplet6.rs`

"Universal" means the candidate carries the anchor's number (D) or a number that holds for every
fuel (instrument, numerics). "Family" means universal within alkanes or within alcohols, different
between them.

| Tensor | Scope | Property or reason |
|---|---|---|
| `rate.b0` | **fuel** | level of K: ρ_l, L(T_b), c_l, T_b, Q/ν through ln(1 + B′) |
| `rate.skip[0]`, rate units 0 and 1 (oxygen) | **fuel** | the logarithmic oxygen sensitivity of ln(1 + B′); larger for a smaller B′ |
| `rate.skip[1]` (CO2), `rate.skip[2]` and unit 2 (helium) | universal | gas side |
| `rate.skip[3]` (pressure) | **fuel**, weakly | boiling point shift dT_b/d ln p = R T_b²/ΔH_vap on top of the carried value |
| `rate.skip[4]`, rate unit 3 (size) | family | radiation and soot of a large flame |
| `rate.skip[5]` (progress) | **fuel** for alcohols | water taken up by the liquid (activity of water over the alcohol) |
| rate heat-up units (columns `s`) | **fuel** | amplitude: c_l (T_b − T_0)/L; duration: liquid thermal diffusivity and the heat to be delivered |
| `crit.b0` | **fuel** | 1/(ρ_l K); between families also Q, ν and the products |
| `crit.skip`, crit units | universal / family | gas side; the pressure term differs between the two anchors |
| `class.kappa`, `class.bias` | universal | widened for a carried-over law |
| `diff.*` | **fuel** for alcohols, family for alkanes | water fraction at flame-out; burn-down and disruption of alkanes |
| `rad.*` | family | clock of flame growth, gas side |
| `cool.p.*` | **fuel** | existence of low-temperature chemistry |
| `cool.k.b0` | **fuel** | ρ_l, L, c_l, wet-bulb temperature under a 700 K flame |
| `cool.k.skip`, cool.k units | universal | helium (conductivity), size, pressure (warm flame) |
| `cool.d.*` | not read when `ce.on = 1`, except `cool.d.sd` (universal, widened) | |
| `ce.g0` | **fuel** | sqrt(K_c): equal residence time at the cool-flame end (published for C7, C8, C10) |
| `ce.beta_atm` | **fuel** | oxygen limit of the cool flame: chemistry, no table |
| `ce.p_ref_atm`, `ce.eps_atm`, `ce.c_he`, `ce.d0_exp` | universal | |
| `dy.frac` | **fuel** | 1 for a fuel with a cool flame, 0 without |
| `dy.tau_s` | universal | |
| `sc.k0_hot_mm2_s`, `sc.k0_cool_mm2_s` | **fuel** | ρD of the vapour, vapour pressure, ρ_l at the surface temperature |
| `sc.tstar_k` | **fuel** | R T²/ΔH_vap about 40 K below boiling |
| `sc.cl_j_kg_k`, `sc.lat_j_kg` | **fuel** | liquid heat capacity and latent heat over the cooling range |
| `sc.k_cold_mm2_s` | **fuel** | vapour pressure at the cold wet bulb |
| `sc.gas`, `sc.k_warm_mm2_s`, `sc.tau_warm_s` | family | gas side |
| `sc.diff_scale` | **fuel** for alcohols | water left in the liquid |
| `post.*`, `post.cool_scale` | not read when `sc.on = 1` | |
| `enc.*`, `filt.*`, `rob.eps_r` | universal | how far a prefix is trusted; moved once for all carried-over candidates |
| `fl.*` | universal | camera frame rates |
| `unc.s_pop`, `unc.s_floor` | universal | widened for a carried-over law |
| `unc.sigma_d`, `unc.sigma_frame` | instrument | field of view of the campaign |
| `haz.log_disruption` | family | |
| `norm.*`, `num.*`, `gate.*`, `enc.window_frac` and the other constants | spec | |

Fuel-specific numbers per alkane candidate: 21 (b0, three oxygen numbers, pressure, four heat-up
numbers, D_crit, cool-flame existence, cool rate, g0, β, two K0, T*, c_l, L, k_cold). Of these, β
and the existence of the cool flame are chemistry (L or J); the rest are P and T on a D anchor.

## 2. Properties, and the fuel-specific tensors derived from them

### 2.1 Property values

Y_O = 0.232 in air, T_0 = T∞ = 298 K, pressure 1 atm unless stated.

| Fuel | M g/mol | T_b K | ρ_l 298 K | ρ_l at T_b | L(T_b) kJ/kg | c_l 298 K | c̄_l 298 K to T_b | Q (lower) MJ/kg | ν kg O2/kg | D_F in air 298 K, cm²/s | α_l 298 K, m²/s |
|---|---|---|---|---|---|---|---|---|---|---|---|
| n-heptane | 100.20 | 371.6 | 679.6 | 613.5 | 316.7 | 2.243 | 2.39 | 44.56 | 3.513 | 0.0705 | 8.0e-8 |
| n-octane | 114.23 | 398.8 | 698.6 | 611 | 301.2 | 2.224 | 2.44 | 44.42 | 3.502 | 0.0657 | 8.2e-8 |
| iso-octane | 114.23 | 372.4 | 687.8 | 623 | 269.5 | 2.09 | 2.25 | 44.34 | 3.502 | 0.0657 | 6.9e-8 |
| n-decane | 142.28 | 447.3 | 726.4 | 601 | 272.3 | 2.210 | 2.51 | 44.24 | 3.486 | 0.0583 | 8.2e-8 |
| n-dodecane | 170.33 | 489.5 | 745.8 | 591 | 256 | 2.207 | 2.57 | 44.11 | 3.476 | 0.0528 | 8.3e-8 |
| farnesane | 212.41 | 522 | 768 | 590 | 218 | 2.15 | 2.55 | 43.95 | 3.465 | 0.0470 | 7.3e-8 |
| methanol | 32.04 | 337.7 | 786.6 | 751 | 1101 | 2.53 | 2.68 | 19.92 | 1.498 | 0.162 | 1.0e-7 |
| ethanol | 46.07 | 351.5 | 785.1 | 737 | 837.0 | 2.438 | 2.72 | 26.82 | 2.084 | 0.124 | 8.7e-8 |
| 1-propanol | 60.10 | 370.3 | 799.6 | 732 | 689.5 | 2.39 | 2.75 | 30.70 | 2.396 | 0.103 | |
| glycerol | 92.09 | 563 | 1258 | | 663 | 2.38 | | 16.1 | 1.216 | | |
| iso-dodecane | 170.33 | 450.6 | 746 (293 K) | | 231 | 2.1 | | 44.0 | 3.476 | 0.0528 | |

Sources and classes.

- **T_b, L(T_b), ΔH_vap(298 K)** (P): NIST Chemistry WebBook, phase-change data, read 15 November
  2026. T_b: n-octane 398.7 ± 0.5, iso-octane 372.4 ± 0.2, n-decane 447.2 ± 0.3, n-dodecane 489 ± 2,
  ethanol 351.5 ± 0.2, 1-propanol 370.3 ± 0.5 K. ΔH_vap at T_b (Majer and Svoboda 1985, as listed by
  NIST): n-octane 34.41, iso-octane 30.79, n-decane 38.75, ethanol 38.56, 1-propanol 41.44 kJ/mol.
  ΔH_vap at 298 K: 41.5, 35.1, 51.3, 61.4 (n-dodecane), 42.3, 47.5 kJ/mol. For n-dodecane NIST lists
  no value at T_b; 43.6 kJ/mol (256 kJ/kg) is recalled from the same compilation [mem], and the
  slope of NIST's Antoine line (47.4 kJ/mol for an ideal vapour) times the vapour non-ideality that
  the same slope shows for n-octane and n-decane (0.94, 0.92) gives 43 to 44. n-Heptane and methanol:
  `properties/properties.csv` (verified there).
- **Q** (P): from NIST enthalpies of combustion of the liquid to liquid water (n-octane 5470.5 [mem;
  NIST's average is 5430 ± 100], iso-octane 5461.3, n-decane 6778.3, n-dodecane 8086.0, ethanol
  1367.6 kJ/mol), less 44.01 kJ per mole of water formed. 1-Propanol 2021 and glycerol 1655 kJ/mol
  [mem].
- **c_l at 298 K** (P): NIST condensed-phase data, 254.1, 239 to 242, 314.5, 376.0 and 112.3 J/mol/K.
  The mean to T_b (T, ± 4 %) assumes the slope of n-heptane's saturated liquid, 0.004 kJ/kg/K per K,
  for the alkanes, and 2.44 to 3.0 kJ/kg/K for ethanol between 298 and 351 K [mem].
- **ρ_l** (P, [mem]): 298 K values from standard tables; at T_b from the saturated-liquid density
  tables as recalled, ± 1 %. n-Heptane 677.9 at 300 K and 613.5 at T_b are verified in
  `properties.csv`.
- **D_F** (T): Fuller's correlation with its tabulated diffusion volumes, evaluated by hand
  (`D = 1.0e-3 T^1.75 sqrt(1/M_F + 1/M_air) / [p (V_F^(1/3) + V_air^(1/3))²]`). Check: n-heptane
  0.0705 against 0.0724 verified; methanol 0.162 against 0.153 to 0.166 verified.
- **α_l = k_l/(ρ_l c_l)** (P, [mem] for k_l: 0.128, 0.0995, 0.132, 0.136, 0.167 W/m/K).
- **Farnesane** (2,6,10-trimethyldodecane, C15H32): density 0.768 g/cm³ at 25 °C and a boiling
  point of 249 °C calculated at 1 atm (measured: 120 °C at 11 Torr), from commercial data sheets
  found by web search: secondary sources (P, weak). L(T_b) from Kistiakowsky's rule,
  ΔS = 36.6 + 8.31 ln T_b J/mol/K, which gives 43.1 kJ/mol for n-dodecane against 43.6 (T). Q, c_l,
  k_l and ρ_l(T_b) are estimates by analogy with n-dodecane and the aviation-fuel specification of
  this compound (J). Derived cetane number 59.1 (Won et al., Combust. Flame 2014, from the search
  summary; L).
- **Iso-dodecane** (2,2,4,6,6-pentamethylheptane): T_b and density [mem]; L from Kistiakowsky (T).
- **Glycerol**: [mem]; it decomposes near its boiling point.

Astra's attempt at a property-based kernel (`ASTRA_FLEX_SOURCE_BOUND_RESULTS.md`) refused every case
because no complete liquid state and corroborated property bundle was admitted: the liquid
temperature is not measured, a saturated-path heat capacity is not an isobaric one, and constant gas
properties need a temperature domain. The route here avoids all three by never computing an absolute
rate: the liquid state and the gas properties cancel in a ratio between two fuels of one family,
and what does not cancel is in the table above.

### 2.2 The transfer number with the liquid heated on the way

`N = Q Y_O/ν − c̄_pg (T_b − 298)`, with c̄_pg the vapour heat capacity at the mean of 298 K and T_b
(alkane vapours 1.66 / 2.09 / 2.47 kJ/kg/K at 300 / 400 / 500 K [mem]);
`Den = L(T_b) + c̄_l (T_b − 298)`; `B′ = N/Den`. `S = [Q Y_O/(ν Den)] / [(1 + B′) ln(1 + B′)]` is
d ln K / d ln Y_O. ρ̄ is the mean of the liquid densities at 298 K and at T_b.

| Fuel | Q Y_O/ν | c̄_pg ΔT | N | c̄_l ΔT | Den | B′ | ln(1+B′) | ρ̄ | S |
|---|---|---|---|---|---|---|---|---|---|
| n-heptane | 2943 | 133 | 2810 | 176 | 493 | 5.70 | 1.903 | 646.6 | 0.468 |
| n-octane | 2943 | 188 | 2755 | 246 | 547 | 5.04 | 1.798 | 654.8 | 0.496 |
| iso-octane | 2937 | 136 | 2802 | 167 | 437 | 6.42 | 2.004 | 655.4 | 0.453 |
| n-decane | 2944 | 295 | 2649 | 374 | 647 | 4.10 | 1.629 | 663.7 | 0.549 |
| n-dodecane | 2944 | 396 | 2548 | 492 | 748 | 3.41 | 1.483 | 668.4 | 0.602 |
| farnesane | 2943 | 477 | 2466 | 571 | 789 | 3.13 | 1.417 | 679 | 0.638 |
| methanol | 3085 | 57 | 3028 | 106 | 1207 | 2.51 | 1.255 | 769 | 0.580 |
| ethanol | 2986 | 85 | 2900 | 145 | 982 | 2.95 | 1.375 | 761 | 0.559 |

Without the liquid heating the classical numbers come out: n-heptane B = 2810/317 = 8.9, methanol
B = 3028/1101 = 2.75, as in every textbook.

### 2.3 Checks against the two well-known fuels

These were worked before any number of a new fuel was typed.

1. **Oxygen slope.** d ln K per 5 % of oxygen at air is S × (0.05/0.21) × 0.97: n-heptane 0.108,
   methanol 0.134. The laws typed from FLEX train have 0.13 and 0.14. The theory is right for
   methanol and 17 % low for n-heptane.
2. **Heat-up.** Leaving the liquid heating out, ln(1 + B) = 2.289 for n-heptane; with it 1.903: the
   rate is 17 % lower while the liquid is being heated. The carried n-heptane law starts 0.19 below
   its plateau.
3. **Level across families: the theory fails.** With the Law–Williams gas properties the
   methanol-to-heptane ratio is (1.255/1.903) × (646.6/769) × 1.15 = 0.64, or 0.55 with the
   classical transfer numbers. Measured on FLEX-size droplets: 0.51/0.58 = 0.88. The heptane flame
   loses to soot and radiation what the methanol flame does not. Hence two anchors and no bridge
   between the families.
4. **Level inside a family: a published check.** For n-decane at 0.5 atm the same formulas (T_b 418 K
   at 0.5 atm, Den 580, ln(1 + B′) 1.74 against 2.01 for n-heptane at 0.5 atm) give 0.84 of the
   n-heptane rate, 0.41 mm²/s for a 4 mm droplet in 21 % oxygen with the carried n-heptane law.
   Published: about 0.3 to 0.4 mm²/s at 14 to 21 % oxygen (Nayagam et al. 2015, as quoted in
   `lit_droplets.md`, section 3; L). The cool-flame rate by section 2.6 is 0.29 against a published
   0.33 at 21 % (same source, as quoted in the heptane W0 edits). The ratio was computed first and
   not adjusted afterwards. This is the only quantitative check a new fuel has, and it is a
   published FLEX-2 fuel: for n-decane the test is not fully blind.
5. **Unlit droplet.** The evaporation constant 8 ρD_F ln(1 + B_M)/ρ_l by hand gives 0.024 mm²/s for
   n-heptane at 300 K (verified table: 0.0222) and 0.16 at the cool-flame wet bulb (verified: 0.19
   at 352 K).
6. **Critical diameter across families.** The form of section 1.1 applied from n-heptane (3.7 mm, D)
   to methanol gives 6.7 mm; the literature has 4 to 4.5 mm, the methanol P0 candidate caps at 5.8
   and the carried law extrapolates to 9. Across families the form is good to a factor 1.5; inside a
   family every factor but ρ_l K cancels.

### 2.4 Hot-flame rate, its oxygen and pressure dependence, heat-up

**Level.** `rate.b0 = b0(anchor) + ln[(ln(1+B′)/ρ̄)_fuel / (ln(1+B′)/ρ̄)_anchor]` (T on D).

| Fuel | ln(1+B′) ratio | ρ̄ ratio | K ratio | ln | `rate.b0` | K at 2 mm, air |
|---|---|---|---|---|---|---|
| n-heptane (D) | 1 | 1 | 1 | 0 | −0.48 | 0.619 |
| n-octane | 0.945 | 0.988 | 0.933 | −0.069 | **−0.55** | 0.577 |
| iso-octane | 1.053 | 0.987 | 1.039 | +0.038 | **−0.44** | 0.644 |
| n-decane | 0.856 | 0.974 | 0.834 | −0.181 | **−0.66** | 0.517 |
| n-dodecane | 0.780 | 0.967 | 0.754 | −0.282 | **−0.76** | 0.468 |
| farnesane | 0.745 | 0.952 | 0.709 | −0.343 | **−0.82** | 0.440 |

Ethanol against methanol: ln(1+B′) ratio 1.095, ρ̄ ratio 1.011, and λ/c_p in the ratio of the vapour
heat capacities at 1250 K (3.05 against 3.36 kJ/kg/K [mem]; the vapour conductivities taken equal),
0.908: K ratio 1.005. The methanol law is exp(−0.53 − 0.35 w): 0.512 mm²/s at half-burn (w = 0.4).
The −0.35 is water taken up by the liquid. Ethanol takes up about a third as much (section 2.8), so
its slope is typed −0.12 and its level set so that the rate at w = 0.4 is 1.005 × 0.512:
`rate.b0 = −0.62`, `rate.skip[5] = −0.12` (T on D, the slope J).

**Oxygen.** The three oxygen numbers of the anchor times S_fuel/S_anchor (T on D):

| Fuel | S ratio | `rate.skip[0]` | unit 0 `w2` | unit 1 `w2` |
|---|---|---|---|---|
| n-heptane (D) | 1 | 0.13 | −0.12 | −0.05 |
| n-octane | 1.058 | 0.14 | −0.13 | −0.053 |
| iso-octane | 0.967 | 0.125 | −0.115 | −0.048 |
| n-decane | 1.171 | 0.15 | −0.14 | −0.059 |
| n-dodecane | 1.286 | 0.17 | −0.15 | −0.064 |
| farnesane | 1.362 | 0.18 | −0.16 | −0.068 |
| methanol (D) | 1 | 0.14 | −0.19 | +0.14 |
| ethanol | 0.964 | 0.135 | −0.18 | +0.13 |

**Pressure.** Theory has pressure only in the boiling point: dT_b/d ln p = R T_b²/ΔH_vap is 36 K
(n-heptane), 38 (n-octane), 37 (iso-octane), 43 (n-decane), 46 (n-dodecane), 49 (farnesane), 27
(methanol, ethanol) per e-fold. That moves ln(1+B′) by −0.073, −0.079, −0.081, −0.090, −0.098,
−0.110, −0.026, −0.034 and the density by +0.03. The carried n-heptane value is +0.05 and the
methanol one +0.10 (D): something else acts, and it is taken to be the same for every fuel. Typed:
the anchor's value plus the difference of the theory terms: 0.045, 0.045, 0.03, 0.03, 0.015 for the
alkanes in table order, 0.09 for ethanol (T on D).

**Heat-up.** Early in the burn the liquid takes more than its lifetime share of heat. With twice the
sensible heat in the denominator, ln(1 + N/(L + 2 c̄_l ΔT)) is lower than ln(1+B′) by 0.143 in the
logarithm for n-heptane, where the carried law (D) starts 0.19 below the plateau (unit 5: −0.3 until
s = 0.6; unit 4: +0.11 until s = 0.3). Both output weights are scaled by the theory ratio. The
duration is scaled by the sum of two times, in clock units: conduction into the liquid,
1/(4π² α_l) with α_l in mm²/s (0.31 for the n-alkanes), and the time the gas needs to deliver the
sensible heat, ρ̄ c̄_l ΔT /(12 (λ/c_p) N), 0.128 for n-heptane with λ/c_p from the anchor's own rate.

| Fuel | ln deficit, theory | scale | unit 5 `w2` | unit 4 `w2` | supply + conduction | stretch | `W1[5,6]` | `W1[4,6]` |
|---|---|---|---|---|---|---|---|---|
| n-heptane (D) | 0.143 | 1 | −0.30 | 0.11 | 0.128 + 0.313 | 1 | −1.667 | −3.33 |
| n-octane | 0.182 | 1.27 | −0.38 | 0.14 | 0.185 + 0.313 | 1.13 | −1.48 | −2.95 |
| iso-octane | 0.147 | 1.03 | −0.31 | 0.11 | 0.124 + 0.366 | 1.11 | −1.50 | −3.0 |
| n-decane | 0.241 | 1.69 | −0.51 | 0.19 | 0.296 + 0.313 | 1.38 | −1.21 | −2.41 |
| n-dodecane | 0.283 | 1.98 | −0.60 | 0.22 | 0.408 + 0.313 | 1.63 | −1.02 | −2.04 |
| farnesane | 0.315 | 2.20 | −0.66 | 0.24 | 0.497 + 0.347 | 1.91 | −0.87 | −1.74 |

A consequence worth stating because CFI will test it. With a constant heat flux from the flame the
surface of a cold droplet reaches its boiling point at a clock of about 0.06 (n-heptane), 0.12
(n-octane), 0.3 (n-decane), 0.6 (n-dodecane) and 0.7 (farnesane), by
ΔT_s = 2 q sqrt(t/π)/sqrt(k_l ρ_l c_l) with q four fifths of the largest flux the flame can give,
2 (λ/c_p) N / D. A 5 mm
n-dodecane droplet in 17 % oxygen is predicted to lose its hot flame at a clock near 0.2: its hot
flame burns over a surface some 70 to 80 K below boiling for its whole life, and its apparent rate
is 0.19 to 0.24 mm²/s, **below the rate of the cool flame that follows** (0.26). Thermal swelling of
the liquid (25 % in volume from 298 K to T_b for n-dodecane, 10 % for n-heptane) hides a further
part of the early regression and is inside the scaled deficit only as far as it is inside the
anchor's.

For ethanol the methanol heat-up unit (−0.6 until s = 0.18, D) is kept in amplitude: it is three
times what liquid heating explains and is mostly flame establishment. Its duration is stretched by
the same two times (0.089 + 0.290 against 0.057 + 0.253): 1.22, `W1[3,6] = −4.6` (T on D).

### 2.5 Critical diameter of radiative extinction

Inside the alkanes, `D_crit = 3.70 mm × [ln(1+B′)]_heptane / [ln(1+B′)]_fuel` (ρ̄ K is proportional
to ln(1+B′); T on D):

| Fuel | ratio | D_crit in air, mm | `crit.b0` = ln(D_crit/2 mm) |
|---|---|---|---|
| n-heptane (D) | 1 | 3.70 | 0.615 |
| n-octane | 1.058 | 3.91 | **0.67** |
| iso-octane | 0.950 | 3.50 | **0.56** |
| n-decane | 1.168 | 4.32 | **0.77** |
| n-dodecane | 1.283 | 4.73 | **0.86** |
| farnesane | 1.343 | 4.97 | **0.91** |

A slower-burning fuel has a more compact flame, and a compact flame radiates a smaller share of its
heat: the law says the heavy alkanes tolerate *larger* droplets. Two effects it leaves out push the
other way: heavier and branched alkanes soot more, and a flame over a cold-cored droplet has no
reserve when its heat feedback weakens. The uncertainty is ± 25 to 30 % and one-sided low for
iso-octane, n-dodecane and farnesane.

The oxygen, CO2 and helium slopes are the n-heptane ones (0.62, −0.45, −0.08; D). The pressure
exponent, zero in the carried law from runs at 0.7 and 1 atm, is typed −0.1: the value the heptane
dossier proposes (its range 0 to −0.3; D), because FLEX-2 reaches 3 atm and CFI 5 atm, where
optically thin emission alone would give −0.5.

**Ethanol.** Across to ethanol from methanol all the factors of section 1.1 move:
Q 26.82/19.92 = 1.35; G_o by ln(1 + 0.232/2.084)/ln(1 + 0.232/1.498) = 0.1055/0.144 = 0.73, cubed
0.39; κ equal within 1 % (products 2 CO2 + 3 H2O against CO2 + 2 H2O in their nitrogen); ρ_l K
equal. D_crit ratio sqrt(1.35 × 0.39) = 0.73. The methanol value is the difficulty: 4 to 4.5 mm in
the literature (L, `lit_droplets.md` section 2) and in the methanol dossier's physics, 5.8 mm as
the P0 cap, 9 mm in the carried law by extrapolation (D). 0.73 of those is 3.1, 4.2 and 6.6 mm;
from n-heptane the same form gives 4.9 mm. Typed: **4.0 mm, `crit.b0 = 0.69`** (T on L; ± 35 %).
The oxygen slope below air is methanol's 0.75 (D); above air 0.30 (a hinge of −0.45; J, after the
P0 cap); CO2 and helium as methanol (D). Pressure: the methanol value +0.7 below 1 atm was typed
from CO2- and helium-diluted runs at 0.7 atm and contradicts the n-heptane value 0; typed 0 below
1 atm (J) and −0.25 above (the methanol P0 value, T).

### 2.6 The cool flame

**Which fuels.** n-Octane, n-decane, n-dodecane: yes (published for C8 and C10: Nayagam et al.
2015, L; C12 was flown for it). Farnesane: yes, with a derived cetane number of 59 against 54 to 56
for n-heptane (L, J). Iso-octane: research octane number 100; a published FLEX-2 study of
n-heptane/iso-octane droplets reports no sustained cool flame for pure iso-octane (Farouk, Xu, Lin,
Avedisian and Dryer, Proc. Combust. Inst. 36, 2017: recalled, not re-read; L, weak). Ethanol: no
low-temperature branching (T). `cool.p.b0` = 3.0 (95 % after a radiative loss) for the first four,
with the oxygen hinge of n-heptane removed because the end-diameter law now decides whether the
flame has any length; −4.0 for iso-octane with +2.0 per e-fold of pressure (J: the low-temperature
chemistry of iso-octane wakes at pressure).

**Rate.** Under a cool flame the droplet sits at a wet-bulb temperature below boiling: 351.6 K for
n-heptane (20 K below; heptane dossier). The depression scales as (R T_b²/ΔH_vap) ln(1/x_s), with
x_s the surface mole fraction at equal vapour mass fraction 0.745 (0.458, 0.426, 0.373, 0.332, 0.285
from C7 to farnesane): 23, 23, 30, 36, 44 K. `B_c = c_p (2T_f − T_s − T∞)/L_eff`, c_p = 2.0 kJ/kg/K,
T_f = 700 K, `L_eff = L(T_s) + c̄_l (T_s − 298)`:

| Fuel | T_s K | L_eff kJ/kg | B_c | ln(1+B_c) | ρ̄ (298 K, T_s) | K_c ratio | `cool.k.b0` | K_c at 3 mm | `ce.g0` |
|---|---|---|---|---|---|---|---|---|---|
| n-heptane (D) | 351.6 | 454 | 3.31 | 1.461 | 656.8 | 1 | −1.125 | 0.345 | 0.47 |
| n-octane | 375.6 | 499 | 2.91 | 1.364 | 665.3 | 0.922 | **−1.21** | 0.317 | **0.45** |
| iso-octane | 349.8 | 395 | 3.81 | 1.570 | 665.9 | 1.060 | **−1.07** | (0.37) | 0.48 |
| n-decane | 417.3 | 581 | 2.36 | 1.211 | 677.2 | 0.804 | **−1.34** | 0.278 | **0.42** |
| n-dodecane | 453.6 | 668 | 1.94 | 1.079 | 682.9 | 0.711 | **−1.47** | 0.244 | **0.40** |
| farnesane | 478.4 | 695 | 1.80 | 1.028 | 696.5 | 0.664 | **−1.53** | 0.230 | **0.38** |

No oxygen, CO2 or pressure term below 1 atm, +0.2 per 20 % helium, size exponent 0.15: the
n-heptane P0 values (D). Above 1 atm a warm flame near 950 K takes over (published numerics, Farouk
et al. 2019, as quoted in `lit_droplets.md`). The same formula at T_f = 950 K gives +0.29 in the
logarithm; the one n-heptane train run at 2 atm gave +0.35 (D). Typed: +0.4 per e-fold of pressure
above 1e5 Pa, held at +0.3 from 2.1e5 Pa on (T with D), in place of the uncapped +0.5 of the
n-heptane candidate, which would give a cool flame twice as fast at 5 atm.

**Where it ends.** The closed form of `droplet6.rs`,
`D_ce/D0 = g0 sqrt((p_ref − β)/(pO2 − β)) (1 + x_He)`, p_ref = 0.18 atm, with the diameter
proportional to D0 (n-heptane train: exponent 0.95 on four pairs; D).

- `ce.g0`: the residence time at the end, D_ce²/K_c, is the same for n-heptane, n-octane and
  n-decane at one oxygen concentration (Nayagam et al. 2015, as quoted in `lit_droplets.md`,
  section 4; L). So g0 = 0.47 × sqrt(K_c ratio) (T on L and D): last column above.
- `ce.beta_atm`: n-heptane 0.12 atm (D: no sustained cool flame at 0.14 atm of oxygen on train). It
  does not carry over. Published cool-flame rates exist for n-decane at 0.5 atm down to 14 % oxygen,
  0.07 atm (same source; L): with g0 = 0.42 a cool flame of any length there needs β below 0.035 atm.
  n-Dodecane cool-flame end diameters are published at low pressure (Williams and Nayagam 2020, as
  quoted; L). Typed: **0.03 atm for n-decane and n-dodecane** (J under an L bound), **0.09 for
  n-octane** (J: between C7 and C10), **0.08 for farnesane** (J: a cetane number like n-heptane's on
  a C15 chain), 0.15 for iso-octane (J; hardly read, since its cool flame is improbable).

With β = 0.03 the law gives for n-decane D_ce/D0 = 0.38 in air at 1 atm, 0.59 at 0.5 atm and 21 %
oxygen, 0.81 at 0.5 atm and 14 %. The published pooled correlation (residence time proportional to
the oxygen concentration to the power −3/2, no dependence on D0) anchored on the same n-heptane
point gives 1.2 mm, 2.3 mm and 2.7 mm, which for a 4 mm droplet are 0.31, 0.57 and 0.67. The two
agree within a quarter at 4 mm and part company with size: at 6 mm the typed law says nearly twice
the diameter of the pooled correlation (2.3 mm against 1.3 mm in air). That is the widest open question of this dossier (section 5).

`cool.d.sd`, the spread of ln D_ce, is typed 0.25 against the 0.12 of n-heptane (J).

### 2.7 The droplet without a flame

The self-cooling law of `droplet6.rs` (`sc.on = 1`) is used for every new fuel, because its constants
are properties; the `post.*` law of the carried n-heptane model has an amplitude and a long-time
rate typed from n-heptane train runs (0.03 mm²/s for ever) that are plainly wrong for a fuel with
no vapour pressure at room temperature. The price is known: on n-heptane train the law's decay time
grows as D² and the runs after a cool flame did not show that (heptane P0 edits; D).

`K_ev = 8 ρD_F(T_s) ln(1 + B_M)/ρ_l(T_s)`, with ρD ∝ T^0.75. Anchors, from `properties.csv`:
n-heptane 0.46 mm²/s at 369 K (3 K below boiling) and 0.19 at 352 K (the cool-flame wet bulb).

| Fuel | `sc.k0_hot` (surface at T_b − 3 K) | typed | `sc.k0_cool` | `sc.tstar_k` | `sc.cl_j_kg_k` | `sc.lat_j_kg` | `sc.k_cold` |
|---|---|---|---|---|---|---|---|
| n-heptane (ref.) | 0.46 | | 0.19 | 26 | | | 0.0097 |
| n-octane | 0.47 | 0.47 | 0.19 | 29 | 2500 | 316000 | 0.0046 |
| iso-octane | 0.44 | 0.44 | 0.17 | 28 | 2300 | 283000 | 0.011 |
| n-decane | 0.49 | **0.25** | 0.18 | 33 | 2650 | 289000 | 0.0006 |
| n-dodecane | 0.50 | **0.15** | 0.17 | 36 | 2800 | 272000 | 0.00007 |
| farnesane | 0.49 | **0.15** | 0.16 | 38 | 2800 | 234000 | 0.00001 |
| ethanol | 0.29 | 0.29 | | 16 | 2720 | 875000 | 0.008 |

- K0 after a hot flame (T): the diffusivity ratio, (T_b ratio)^0.75, the ratio of ln(1 + B_M) at a
  surface mole fraction of 0.92 (3.71 for C7 to 4.45 for farnesane) and the density ratio nearly
  cancel: 0.44 to 0.50 for every alkane. But a large n-decane, n-dodecane or farnesane droplet that
  loses its hot flame early is not at its boiling point (section 2.4). Typed half for n-decane and
  0.3 for the two heaviest (J). It matters little: a cool flame nearly always follows.
- K0 after a cool flame (T): the wet-bulb state of section 2.6 has the same ln(1 + B_M) for every
  fuel, so K0 goes as D_F × (T_s ratio)^0.75: 0.98, 0.95, 0.92, 0.85 of n-heptane.
- T* (T): R T²/ΔH_vap at 40 K below boiling, which gives 26.6 K for n-heptane where the heptane
  dossier read 26 K off the evaporation table. For ethanol the same expression (19.3 K) times the
  factor that takes methanol's 19.7 K to the 16 K of the methanol candidate.
- c_l and L (P): means over the 60 K below boiling.
- k_cold (T): vapour pressure at the cold wet bulb [mem: 1.9, 6.5, 0.17, 0.018 kPa at 298 K for
  n-octane, iso-octane, n-decane, n-dodecane]. An unlit n-dodecane droplet does not shrink at all
  once it is cold; the n-heptane law would take 0.03 mm²/s from it for ever.
- `sc.gas` = (1.7, 0, 0.1) for alkanes: the vapour leaves 1.8 times faster through 35 % helium
  (heptane dossier, section 1.8; T). (0.8, 0, 0.15) for ethanol, the methanol values (D).
- `sc.k_warm` 0.045 mm²/s fading over 15 s for alkanes (heptane dossier's proposal; D), 0.025 for
  ethanol (methanol P0; D).

### 2.8 Ethanol: water, and the end of a small flame

A methanol flame goes out when its droplet has shrunk to 0.46 of its initial diameter because the
liquid has taken up 30 % of water (methanol dossier). Ethanol makes as much water per kilogram
(1.17 against 1.12) but holds less of it: over an ethanol-rich liquid at its boiling point the
vapour pressure of water per unit mole fraction is about three times that over methanol (activity
coefficient 2.7 against 1.55, saturation pressure 44 kPa at 351 K against 25 kPa at 338 K; [mem]),
so absorption reverses at about a third of the water content, and beyond the azeotrope (4.4 % water
by mass) distillation no longer enriches the liquid. Typed (J on T): the rate falls by 0.12 over
the burn instead of 0.35; the flame goes out at D/D0 = 0.26 (burn progress 0.93,
`diff.b0 = 2.6`), between the water limit of methanol (0.46) and the pure-fuel limit (about 0.1);
after it the liquid is 0.8 as volatile (`sc.diff_scale`). Range of the extinction ratio: 0.15 to
0.38.

### 2.9 What is carried for every candidate, and what was moved for all of them at once

Carried (D): CO2 and helium coefficients of every law, the size terms of the rate, the diffusive
law of the alkanes, the radiative clock with its helium term, the dying flame (full cool rate
decaying over 3.5 s), the state filter, the camera entries.

Moved for all, because a carried-over law deserves less trust than one typed from its own fuel (J):

| Entry | Anchor | Candidates | Reason |
|---|---|---|---|
| `unc.s_pop`, `unc.s_floor` | 0.10, 0.06 | 0.20, 0.10 (farnesane 0.25, 0.12) | the level of K is uncertain by 10 to 20 % |
| `enc.gain_b` | −1.8 (alkanes), −1.6 | −1.0 | a prefix is believed more: gain 0.19 / 0.40 / 0.51 at 1 / 3 / 5 s for alkanes |
| `enc.e_clip` below | 0.3 | 0.5 (alkanes) | a law that is 30 % high must be correctable |
| `enc.cool_gain` | 0.5 | 0.7 | |
| `class.kappa`, `class.bias` | 14, 0.5 / 10, 0.7 | 6, 0 | D_crit uncertain by a quarter |
| `rob.eps_r` | 0 or 0.1 | 0.2 | one flame in five may not follow the carried clock |
| `cool.d.sd` | 0.12 | 0.25 | |
| `filt.sd_hot` | 0.2 / 0.18 | 0.25 / 0.22 | |
| `diff.sharp`, `rad.sharp` (ethanol) | 14, 5 | 8, 4 | the end of an ethanol flame is the least certain law of that candidate |
| `unc.sigma_d`, `unc.sigma_frame` | 2.5e-5, 8e-6 m (heptane) | 4.2e-5, 1.3e-5 m (3.3e-5, 1.0e-5 for ethanol) | FLEX-2 enlarged the field of view from 30 to about 50 mm (`lit_droplets.md`, section 9; L): 1.67 times coarser pixels. Assumed for CFI too |

## 3. Can the existing forward represent each fuel?

### 3.1 Pure fuels

`droplet6.rs` takes the fuel as a name that selects one of two tensor sets, with and without the
cool-flame laws. To assemble the candidates an engineer adds the fuels to its `Fuel` enum (name,
spec id, label, `has_cool`), registers the models, and writes a binding; no scientific number is
involved. The candidates assume spec ids `flex_<fuel>-wsp-v1`, model names `flex_<fuel>.wsp5`, the
heptane tensor set for octane, isooctane, decane, dodecane and farnesane, and the methanol tensor
set for ethanol.

Each candidate is a **literal revision, not a root**. The contract lets a root patch be written
only by an author who has seen no other candidate (`ModelRelease::check_construction`: "W0 sees no
evaluation feedback and no other candidate"), and this author has read the candidates of n-heptane
and methanol and says so in each `meta.json`. So each fuel needs a lineage root, as every
data-informed first candidate of a spec has had: the D²-law reference of the anchor fuel
(`wsp5/heptane/baseline-d2`, or `wsp5/methanol/baseline-d2` for ethanol), copied without change
under the new spec id, and a chain `["baseline-d2", "w0-fable"]` in `tools/wsp/reassemble.py`,
which then writes the parent id into `meta.json` and into the first line of `edits.wsp` (both hold
a placeholder of zeros now). Each candidate writes every authored element and the roots have no
hinge unit, so no number of a root survives in a candidate: the roots are provenance only. The
folders are called `w0-fable` because the candidates are one shot for their fuel; they are not
zero-shot.

The six `edits.wsp` were linted against the tensor lists of the two existing wsp5 specs (names,
shapes, bounds, every authored element written once): clean. Nothing was assembled and no forward
was run.

What the forward cannot do for these fuels, in order of weight:

1. **Reignition** (CFI). The table of CFI has a column for it. In the forward a cool flame can only
   end; a hot flame that returns from a cool or warm flame, burns for a second or two and is lost
   again has no state. Section 3.3.
2. **Warm flame as a stage of its own** (CFI above about 2 atm, more with helium). The candidates
   give the cool flame a faster rate above 1 atm, which is the rate of a warm flame, but one stage
   with one end diameter.
3. **A cool flame lit directly.** CFI varied igniter power and time; a weak igniter can light a cool
   flame without a hot one. The forward starts every droplet in a hot flame.
4. **Fibre-supported droplets** (103 of the 227 CFI rows, and part of FLEX-2): excluded by the spec.
5. **Domain.** `c_logp` ends at 4.4 atm (CFI reaches 5); `c_he` at 60 % helium (FLEX-2 lists
   oxygen in helium, 70 to 83 % helium); the horizon `num.t_valid_s` is 45 s where a 5 mm
   n-dodecane droplet burns for 80 to 100 s.
6. **Moving droplets and droplet pairs** (FLEX-2): outside the target.
7. **Heat-up as a state.** The candidates write the heat-up as a function of the clock, which is
   right for a droplet ignited cold. Astra's seventh forward (`droplet7.rs`, `thermal.*`) carries a
   liquid temperature; it was not read and may be the better place for the heavy fuels.

### 3.2 Mixtures: the smallest forward that carries them

Three different things happen in a two-component droplet, and the three mixtures flown sit at
three corners.

- **Equal volatility, unequal chemistry** (n-heptane + iso-octane, boiling points 371.6 and
  372.4 K). The liquid keeps its composition. The hot flame does not notice (the two pure levels are
  0.62 and 0.64 mm²/s). The cool flame burns on the n-heptane alone: its probability, its oxygen
  limit and its end diameter depend on the n-heptane fraction.
- **Modest volatility difference** (n-dodecane + iso-dodecane, 489.5 and 450.6 K: relative
  volatility about 2.4). The light, unreactive component leaves first from a thin surface layer; the
  vapour is poorer in n-dodecane than the liquid early and richer late. Rate nearly unchanged;
  cool-flame chemistry as above, drifting with burn progress.
- **Large volatility difference** (1-propanol + glycerol, 370 and 563 K). Stage 1 burns propanol
  while glycerol piles up in a surface layer whose thickness is the liquid diffusivity over the
  regression speed. When the layer is saturated the surface temperature must rise by nearly 200 K:
  vaporisation stalls, the flame contracts onto the droplet (the SRD's "flame contraction"), and
  either the flame is lost in that instant (flash extinction: the fuel flux drops below what a flame
  of that size needs) or a second stage burns the remainder with propanol superheated inside
  (above 0.9 of its critical temperature, 483 K, it nucleates: disruption).

Preferential vaporisation in the diffusion-limited liquid gives the diameter at which stage 1 ends
by a mass balance: the involatile rejected by the vaporised shell, y (D0³ − D_c³), fills a surface
layer of thickness 4 D_c D_l/K on a sphere of diameter D_c:

```text
D_c = D0 [1 + 3 eps (1 - y)/y]^(-1/3),      eps = 8 D_l / K      (Shaw and Williams' diffusion-time ratio)
```

For 1-propanol + glycerol with D_l of order 1e-9 m²/s (J; not looked up) and K = 0.55 mm²/s,
eps = 0.015: D_c/D0 = 0.82 at 5 % of glycerol by mass and 0.95 at 20 %.

**Forward `droplet8.rs` (proposal): `droplet6.rs` with the following, every block off by default so
that it then equals `droplet6.rs` number for number.**

New condition, from the data lane: `y_b`, the mass fraction of component B (the less volatile, or
for equal volatility the less reactive) in the liquid as deployed, 0 to 1. The catalogue records
mixture fractions with their basis unstated; the lane has to state mass, mole or volume, and
convert with the two liquid densities at 298 K, which are properties and must come from the author
as the two authored scalars below, not from the engineer.

| Tensor | Shape | Bounds | Unit | Meaning |
|---|---|---|---|---|
| `mix.on` | [] | 0 or 1 | 1 | Switch. 0: `y_b` is not read. |
| `mix.rho_a`, `mix.rho_b` | [], [] | (300, 2000) | kg/m³ | Liquid densities of the two components at 298 K, for converting a volume fraction to `y_b`. |
| every law network: `rate`, `crit`, `diff`, `rad`, `cool.p`, `cool.k`, `cool.d` | one more input column, last | as the network | 1 | The feature `c_y = y_b`. `W1` gains a column, `skip` an element; the forward appends `y_b` to the input vector of each network. Composition dependence is then written like any other condition. |
| `ce.beta_y` | [] | (−1, 1) | atm | The oxygen limit of the cool flame becomes `ce.beta_atm + ce.beta_y * y_s`. |
| `ce.g0_y` | [] | (−5, 5) | 1 | `ce.g0` is multiplied by `exp(ce.g0_y * y_s)`. |
| `st.on` | [] | 0 or 1 | 1 | Switch of staged burning. |
| `st.eps` | [] | (0, 5) | 1 | Liquid diffusion number 8 D_l/K of the mixture in stage 1. |
| `st.dc_sd` | [] | (0, 1) | 1 | Assumed sd of ln D_c between droplets. |
| `st.drift` | [] | (−1, 1) | 1 | Surface composition seen by the cool-flame laws: `y_s = clamp(y_b + st.drift * w, 0, 1)` with w the burn progress (0: the liquid keeps its composition). |
| `st.k2` | [] | (−3, 1) | 1 | ln of the stage-2 rate over the stage-1 law at the same state. |
| `st.dip` | [2] | (0, 1), (0.001, 2) | 1, clock units | At D_c the rate is multiplied by `1 − dip[0] exp(−(s − s_c)/dip[1])`: the surface heating to its new temperature. |
| `st.crit2` | [] | (−3, 3) | 1 | ln(D_crit in stage 2 / D_crit of the law). |
| `st.flash.W1`, `.b1`, `.w2`, `.skip`, `.b0` | a law network of width 8 on (c_o2, c_co2, c_he, c_logp, c_logd0, x_exc, c_y) | b0 (−30, 30) | logit | Probability that the hot flame is lost at D_c (flash extinction). |
| `st.log_disruption2` | [] | (−20, 3) | ln(1/s) | Disruption hazard in stage 2, in place of `haz.log_disruption`. |

Computation.

1. `y_b` from the condition; `c_y = y_b` appended to every network input. With `mix.on = 0` the
   column is not read.
2. Stage 1 is the hot-flame law as now. With `st.on = 1`, for each scenario node u of a new
   five-point grid (as for ln D_ce): `D_c = D0 [1 + 3 st.eps (1 − y_b)/y_b]^(−1/3) exp(st.dc_sd u)`;
   for `y_b = 0` there is no stage 2.
3. When a hot flame reaches D_c: with probability `sigmoid(N_flash)` the flame is lost; the droplet
   continues under the unlit law as after a diffusive extinction. Otherwise stage 2:
   `K = K_h exp(st.k2) (1 − dip[0] exp(−(s − s_c)/dip[1]))`, the radiative class is re-drawn with
   D_crit multiplied by `exp(st.crit2)`, the diffusive law continues in burn progress, and the
   disruption hazard is `exp(st.log_disruption2)`.
4. A cool flame that reaches D_c has its rate multiplied by the same stage-2 factor; nothing else.
5. The cool-flame laws read `y_s` where they read composition (`ce.beta_y`, `ce.g0_y`, and the
   `c_y` column of `cool.p`, `cool.k`).
6. State at the cutoff: stage 2 has begun if the level at the cutoff is below D_c of the node; no
   new evidence is read.

What the author would then type for the three mixtures, as far as it can be said now: for
n-heptane + iso-octane only the `c_y` columns of `cool.p` and `ce.beta_y` (β from 0.12 atm toward
no cool flame as the iso-octane fraction goes to 1); for n-dodecane + iso-dodecane the same with
`st.drift` negative and a small `st.eps`-driven stage; for 1-propanol + glycerol the whole staging
block on a propanol root that itself has to be written as ethanol was here (alcohol family,
anchor methanol).

### 3.3 Large alkanes: reignition, warm flame, direct cool ignition, fibre

Also for `droplet8.rs`, each behind a switch.

| Tensor | Shape | Bounds | Unit | Meaning |
|---|---|---|---|---|
| `warm.on` | [] | 0 or 1 | 1 | Switch. |
| `warm.p.*` | law network on (c_o2, c_co2, c_he, c_logp, c_logd0, x_exc) | b0 (−30, 30) | logit | Probability that the stage after a radiative loss begins as a warm flame. |
| `warm.k` | [] | (−2, 2) | 1 | ln(K_warm/K_c): the warm flame burns at `K_c exp(warm.k)`. |
| `warm.s.*`, `warm.sharp` | law network; [] (0.5, 40) | b0 (−8, 4) | ln(clock) | Median clock length of the warm stage and its Gompertz sharpness, as `rad.*`; then the cool flame. |
| `re.on` | [] | 0 or 1 | 1 | Switch of reignition. |
| `re.h.*` | law network | b0 (−20, 5) | ln(1/clock) | ln of the hazard, per unit of clock, that a warm or cool flame reignites the hot flame. |
| `re.s2` | [] | (−5, 2) | 1 | ln(clock of the reignited hot flame / s_R): a reignited flame is lost again after `s_R exp(re.s2)`. |
| `re.max` | constant | | 1 | Largest number of reignitions followed (3). |
| `ig.on` | [] | 0 or 1 | 1 | Switch. |
| `ig.cool.*` | law network on the law inputs and `c_ign = ln(igniter power x time / ig.e_ref)` | b0 (−30, 30) | logit | Probability that ignition gives a cool flame and no hot flame. Needs igniter power and time as conditions (the CFI table has both). |
| `ig.e_ref` | constant | | J | Reference igniter energy. |
| `fib.on` | [] | 0 or 1 | 1 | Switch; reads the condition "supported on a fibre" (0 or 1). |
| `fib.rate`, `fib.crit`, `fib.re` | [], [], [] | (−2, 2) | 1 | Added to ln K_h, to ln D_crit and to the reignition ln-hazard for a supported droplet (heat conducted along the fibre). |

Computation. After a radiative loss: warm flame with probability `sigmoid(N_warm_p)`, burning at
`K_c exp(warm.k)` for a clock length drawn as the radiative clock is; then the cool flame as now.
During a warm or cool flame the hot flame returns with hazard `exp(N_re)` per unit clock (competing
with the stage's own end); the returned hot flame burns at K_h and is lost at `s_R exp(re.s2)`
after its return, then the sequence restarts, at most `re.max` times. At ignition, with probability
`sigmoid(N_ig_cool)` the droplet starts in the cool flame at s = 0. The state filter gains no state:
warm counts as cool for the evidence (their rates differ by `warm.k`), and the camera sees a
reignited hot flame as it sees any hot flame.

Nothing in this section has a number yet. The only published guide to them in the repository is
numerical: at 3 atm and 21 % oxygen a warm stage of 23 s at 20 % helium and 1 s at 60 %
(`lit_droplets.md`, section 4).

## 4. Registered expectations

Written before any outcome of CFI or FLEX-2 was opened. "±" is the author's one-sigma. Rates are
−d(D²)/dt in mm²/s on the plateau after heat-up unless said otherwise.

### 4.1 Hot flame

| Fuel | K, air, 1 atm, 2 mm | K, 3 mm | per +5 % O2 above air | per −5 % O2 below air (to 17.5 %) | per e-fold of pressure | per +20 % helium | early deficit (first 0.3 clock units) |
|---|---|---|---|---|---|---|---|
| n-heptane (D) | 0.62 | 0.58 | +14 % | −22 % | +5 % | +16 % | 14 to 17 % |
| n-octane | 0.58 ± 0.05 | 0.54 ± 0.05 | +15 % | −24 % | +5 % | +16 % | 18 to 21 % |
| iso-octane | 0.64 (+0.04, −0.09) | 0.61 (+0.04, −0.09) | +13 % | −21 % | +5 % | +16 % | 15 to 18 % |
| n-decane | 0.52 ± 0.06 | 0.49 ± 0.06 | +16 % | −25 % | +3 % | +16 % | 24 to 27 % |
| n-dodecane | 0.47 ± 0.07 | 0.44 ± 0.07 | +19 % | −27 % | +3 % | +16 % | 28 to 32 % |
| farnesane | 0.44 ± 0.08 | 0.41 ± 0.08 | +20 % | −29 % | +2 % | +16 % | 31 to 34 % |
| methanol (D) | 0.59 at ignition, 0.51 at half-burn | the same | +15 % | −28 % | +11 % | +20 % | |
| ethanol | 0.54 at ignition, 0.51 ± 0.05 at half-burn | the same | +14 % | −27 % | +9 % | +20 % | |

- Order of the plateau rates at equal size and atmosphere: iso-octane ≥ n-heptane > n-octane >
  n-decane > n-dodecane > farnesane; n-dodecane 0.75 ± 0.10 of n-heptane.
- The oxygen sensitivity grows with the chain: n-dodecane 1.29 times n-heptane's.
- CO2 at the expense of nitrogen: −8 % per 20 % (alkanes), −4 % (ethanol); carried.
- Helium beyond 30 %: +10 % per further 20 % (alkanes).
- **CFI as flown** (n-dodecane, 4 to 6.5 mm, 17 to 21 % oxygen): the hot flame is lost to radiation
  within 0.1 to 0.5 clock units (4 to 13 s) and its apparent rate over that time is 0.17 to 0.29
  mm²/s at 1 atm, less than or equal to the cool-flame rate that follows. ± 0.08. Farnesane: 0.15 to
  0.26.

### 4.2 Radiative extinction

| Fuel | D_crit, air, 1 atm | at 18 % O2 | at 25 % O2 | per e-fold of pressure | with 20 % CO2 | with 20 % He |
|---|---|---|---|---|---|---|
| n-heptane (D) | 3.7 mm | 2.55 | 6.1 | 0 (carried) | × 0.64 | × 0.92 |
| n-octane | 3.9 ± 1.0 | 2.7 | 6.4 | −10 % | × 0.64 | × 0.92 |
| iso-octane | 3.5 ± 1.0 | 2.4 | 5.8 | −10 % | × 0.64 | × 0.92 |
| n-decane | 4.3 ± 1.1 | 3.0 | 7.1 | −10 % | × 0.64 | × 0.92 |
| n-dodecane | 4.7 ± 1.4 | 3.3 | 7.8 | −10 % | × 0.64 | × 0.92 |
| farnesane | 5.0 ± 1.5 | 3.4 | 8.2 | −10 % | × 0.64 | × 0.92 |
| ethanol | 4.0 ± 1.4 | 2.55 | 5.1 | 0 below 1 atm, −22 % above | × 0.85 | × 0.84 |

An alkane droplet at its D_crit loses its flame at a clock of 0.6 to 0.8; the clock falls by e² per
e-fold of D0/D_crit and by e per e-fold of D0 above 2.3 mm, and is 20 % shorter per 20 % of helium
(carried).
For CFI this means: in air at 1 atm n-dodecane droplets below about 4.7 mm are expected to keep
their hot flame and burn down with **no cool flame**, and all droplets at 17 to 18 % oxygen to lose
it. If the 4 mm droplets in air lose their flame too, D_crit is below 4 mm and the compact-flame
argument is wrong for this fuel.

### 4.3 Cool flame

| Fuel | follows a radiative loss | K_c, 1 atm, N2, 3 mm | 5 mm | at 3 atm | per +20 % He | D_ce/D0: air 1 atm | 18 %, 1 atm | 21 %, 0.5 atm | 21 %, 3 atm | no sustained cool flame below |
|---|---|---|---|---|---|---|---|---|---|---|
| n-heptane (D) | 95 % (less below 16 % O2) | 0.345 | 0.37 | 0.60 (the uncapped P0 term, one train run at 2 atm) | +22 % | 0.38 | 0.47 | none | 0.16 | 0.12 atm O2 |
| n-octane | 95 % | 0.32 ± 0.06 | 0.34 | 0.43 | +22 % | 0.39 | 0.45 | none (0.105 atm: 1.1) | 0.18 | 0.09 atm |
| n-decane | 95 % | 0.28 ± 0.06 | 0.30 | 0.38 | +22 % | 0.38 | 0.42 | 0.59 | 0.21 | 0.03 atm |
| n-dodecane | 95 % | 0.24 ± 0.06 | 0.26 | 0.33 | +22 % | 0.37 | 0.40 | 0.57 | 0.20 | 0.03 atm |
| farnesane | 95 % | 0.23 ± 0.07 | 0.25 | 0.31 | +22 % | 0.33 | 0.38 | 0.76 | 0.16 | 0.08 atm |
| iso-octane | 2 % at 1 atm, 14 % at 3 atm | (0.37) | | | | (0.34) | | | | |
| ethanol | never | | | | | | | | | |

- K_c does not depend on oxygen or on CO2, and not on pressure between 0.5 and 1 atm.
- D_ce is proportional to D0 and grows by (1 + x_He) with helium. One sigma of ln D_ce: 0.25, and
  0.4 for n-octane and farnesane at 0.5 atm, where the typed β decides.
- After a cool flame the droplet evaporates on its stored heat, 0.16 to 0.19 mm²/s falling as
  1/(1 + t/t0) with t0 = 5 to 13 s for a 2 to 3 mm remnant, and an n-decane, n-dodecane or farnesane
  droplet then stops shrinking altogether.
- A flame born at or above its end diameter dies away over 3.5 s (carried).

### 4.4 Diffusive end

n-Alkanes: the n-heptane law, the flame gone at D/D0 = 0.27 to 0.39 or the droplet disrupted
(carried). Ethanol: D/D0 = 0.26 (0.15 to 0.38), against 0.46 for methanol.

## 5. What would show the carried-over law is wrong

Each line names the observation and the assumption it kills.

1. **n-Dodecane burns as fast as n-heptane at equal size and atmosphere** (ratio above 0.9), or
   below 0.55 of it. Kills: the liquid heating in the transfer number as written (the whole sensible
   heat in the denominator, the lifetime share). The published n-decane value is at the low edge of
   the prediction already.
2. **No early deficit growing with the boiling point**, or a hot rate of large CFI droplets above
   0.4 mm²/s. Kills: the heat-up scaling of section 2.4 and the statement that these hot flames live
   on a cold surface.
3. **The oxygen sensitivity does not grow with chain length.** Kills: the use of B′ for the slopes.
4. **4 mm n-dodecane droplets in air at 1 atm lose the hot flame to radiation**, or iso-octane
   droplets of 3 mm do. Kills: D_crit ∝ 1/(ρ_l K) without soot. The opposite failure, 5.5 mm
   droplets in air burning down, would say the carried clock and class do not hold at these sizes.
5. **Cool flames of n-decane or n-dodecane at 0.5 atm end above 0.9 D0, or are absent.** Kills:
   β = 0.03 atm; the limit would be n-heptane's. Conversely n-octane cool flames at 0.5 atm in
   21 % oxygen kill β = 0.09.
6. **D_ce does not scale with D0** across the 3.8 to 6.5 mm of CFI (the same end diameter in
   millimetres at one atmosphere). Kills: the carried exponent 1; the published pooled correlation
   (D_ce² ∝ K_c C_O2^(−3/2)) would then be the law, and the candidates over-predict D_ce at 6 mm by
   a factor near 2.
7. **K_c of the heavy alkanes equals n-heptane's 0.34**, or depends on oxygen. Kills: the wet-bulb
   transfer number of section 2.6.
8. **Iso-octane sustains a cool flame at 1 atm**; **ethanol shows a second stage.** Kills: the
   reading of low-temperature chemistry from octane and cetane numbers.
9. **Ethanol flames go out near 0.45 D0** as methanol's do. Kills: the water argument of 2.8.
   **Ethanol droplets of 3 mm in air lose the flame early.** Kills: 4.0 mm.
10. **Helium or CO2 act differently on a heavy alkane than on n-heptane** by more than 10 % in rate
    at 30 % of diluent. Kills: the gas-side coefficients as universal.
11. **Rates at 3 atm more than 15 % off the 1 atm rate** for any pure fuel. Kills: the carried
    pressure term, typed from 0.7 to 1 atm; soot at pressure would be the first suspect.
12. **Hot flames that return.** Not a failure of a number but of the forward: section 3.3.

A blind score will mix all of these. They can be separated by scoring, in this order: the plateau
rate at cutoffs where the flame still burns (lines 1 to 3, 10, 11), the hot-flame end time from the
cameras (line 4), the rate after it (lines 7, 8) and the plateau diameter (lines 5, 6).

## 6. The three weakest assumptions

1. **The end of the cool flame for C8 and above**: β typed by judgement under one published bound,
   and proportionality to D0 extrapolated from 2.4 to 4.2 mm n-heptane droplets to 6.5 mm
   n-dodecane droplets against a published correlation that has no size in it.
2. **The critical diameter as 1/(ρ_l K)** with no term for soot and none for the cold core: it makes
   heavy alkanes more robust to radiative loss, and CFI's small droplets in air test exactly that.
3. **The level and the early deficit of the heavy alkanes**: a lifetime-average energy argument
   applied to droplets whose hot flame lasts a tenth of their lifetime, with one published n-decane
   number as its only check.

Next in line: ethanol's water uptake and critical diameter (no check at all), every farnesane
property, and the carried pressure terms beyond 1 atm.

## Sources

| Source | Read | Taken |
|---|---|---|
| `configs/models/flex/wsp5/heptane/{carry-d2,p0-fable,p1-fable}`, `wsp5/methanol/{carry-d3,p0-fable,p1-fable}`, baselines | in full | every number marked D |
| `docs/wsp/research/phys_flex_heptane.md`, `phys_flex_methanol.md` (first 230 lines) | as listed | wet-bulb state, T*, warm floor, helium factor of evaporation, the statement that the level is not predictable |
| `docs/wsp/research/lit_droplets.md` | in full | published n-decane rates, the shared residence time of C7/C8/C10, cool-flame formula, warm flame, FLEX-2 field of view |
| `docs/wsp/research/properties/properties.csv` | n-heptane and methanol rows | verified anchors |
| NIST Chemistry WebBook, SRD 69: phase-change and condensed-phase pages of C8H18 (111-65-9, 540-84-1), C10H22 (124-18-5), C12H26 (112-40-3), C2H6O (64-17-5), C3H8O (71-23-8) | summaries returned by a fetch tool, 15 November 2026 | T_b, ΔH_vap, enthalpies of combustion, c_l |
| Web search, farnesane | result summary | density, calculated boiling point, derived cetane number |
| Williams and Nayagam, Combust. Theory Model. 23 (2019); Nayagam, Dietrich, Hicks and Williams, Combust. Flame 162 (2015); Farouk, Dietrich and Dryer, Proc. Combust. Inst. 37 (2019); Williams and Nayagam, Combust. Flame 212 (2020) | only as quoted in `lit_droplets.md` and the heptane dossier | as cited in the text |
| Farouk, Xu, Lin, Avedisian and Dryer, Proc. Combust. Inst. 36 (2017) | recalled, not re-read | iso-octane without a sustained cool flame |
| `docs/wsp/atlas/cards/PSI-39.json`, `PSI-68.json`, `atlas/INVESTIGATIONS.md` | condition blocks only, as stated at the top | fuels, ranges of oxygen, pressure, size, diluents, support |

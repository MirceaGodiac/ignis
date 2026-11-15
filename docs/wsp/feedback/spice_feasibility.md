# SPICE (PSI-107): what can be forecast and scored without opening an image

> Written by a delegated data agent (claude-opus-5-5) on 15 November 2026 for the seventh generation (brief
> `runs/dev/gen7/BRIEF_DATA.md`); nothing fitted except the constants of the named baselines on train days.
> Cases script: `tools/wsp/ext/spice_cases.py`. Case files: `runs/dev/data_all/work/transfer_spice/` (local, not in git).
> **For the physicist:** read only the paragraph below and sections 1 to 3. Section 5 holds train scores.

**For the physicist.** A forecast is asked for **`L_sp_mm`**: the luminous length (mm, from the burner exit to
the flame tip) of a laminar gas-jet diffusion flame in coflowing air in microgravity at its **smoke point**, the
fuel flow at which soot first escapes the tip. It is asked from these inputs, as named in the case files
(`inputs.*`): `fuel` and `hydrocarbon` (ethylene, propane, propylene; propylene also diluted with N2),
`x_fuel` (hydrocarbon mole fraction of the jet, the rest N2), `mw_jet_g_mol`, `nozzle_id_mm` (0.4064, 0.762 or
1.6002 mm, inner diameter of a round tube), `u_air_cm_s` (coflow air velocity, cm/s), `pressure_pa` (101325,
nominal cabin), `zst_jet` (stoichiometric mixture fraction of the jet gas in air), `tad_k` (adiabatic flame
temperature, K), `rho_jet_g_m3` (jet gas density at 294 K), `mu_air_mean_g_m_s` (air viscosity at the mean of
the adiabatic and ambient temperatures, g/(m s)), `air_per_jet_mol` (stoichiometric moles of air per mole of
jet gas). The fuel flow at the smoke point is **not** an input: it is found by the same search and is an outcome.
Form: **quantiles at 5, 25, 50, 75 and 95 %** of `L_sp_mm`, keys `q05 q25 q50 q75 q95`. A second, optional
target T2 asks for the same length when the smoke-point flow is given (`inputs_given_flow.mdot_jet_mg_s`, total
jet mass flow in mg/s, with `q_jet_sccm` and `u_jet_cm_s`), in the same form; it tests a flame-length law.

## 1. Forecast targets

| | T1 (primary) | T2 (secondary) |
|---|---|---|
| Quantity | smoke-point flame length `L_sp_mm` | the same length, given the smoke-point fuel flow |
| Unit | mm | mm |
| Why it matters | the smoke point is the standard measure of a fuel's sooting tendency; a longer smoke-point flame means a fuel that can burn larger flames before releasing soot (smoke, radiation, toxic products in a cabin) | the flame length at a given fuel flow sets the reach of a jet fire |
| Where recorded | `Raw Data/PSI-107_Raw Data_SmokePointReview-0g-3.xlsx`, sheet "Smoke Point Datasheet", column "smk pt (mm)" (index 18): the investigators' own review of the video record, pixel length converted by their ruler calibration (sheet "Reference", cells B23/B24, day 045) | same row |
| A forecast may know | the pre-test conditions above; no sensor record, no cutoff | the T1 inputs plus `inputs_given_flow` |
| Cutoffs, horizons | none: each smoke point is a steady state | none |
| Score | five-level quantile score x 2 (the five-point approximation of CRPS, mm); per day and day-equal aggregate | same |

One case is one smoke point (one row of the datasheet). A flame (ignition) can give several smoke points at
different coflows. Secondary outcome fields kept in the case files: `mdot_sp_mg_s` (total jet mass flow at the
smoke point) and `flame_width_mm`.

Candidates weighed and not taken:
- **Flame length at any flow** (`Spice Flight Test Data II modified.xls`, column "Length (mm)"): 134 cells over
  8 days, many on half-flow (non-sooting) flames, from two merged transcriptions of the record with different time
  separators. It could add about 55 train flames to T2 later, after the duplicates are resolved.
- **Lift-off** (the same workbook, column "LO ?"): only "y" is written, on some days; a blank does not say
  "attached". Not scorable as a yes or no without the video.
- **The smoke-point flow itself** as the target: equivalent information to T1 (the two are tied by the
  flame-length law, see T2); T1 is the quantity the smoke point is defined by.

## 2. Inputs

All are pre-test conditions except `inputs_given_flow` (T2 only). The investigators computed most of them in the
datasheet; the script copies their values and records the row.

| Name in case file | Unit | Source (datasheet column, or derived) | Trust |
|---|---|---|---|
| `fuel`, `hydrocarbon`, `x_fuel`, `mw_jet_g_mol` | -, -, mol/mol, g/mol | "Fuel", "Xf", "MWmix" | good; bottles of fixed composition |
| `nozzle_id_mm` | mm | "Nozzle (mm)"; measured IDs in sheet "Reference" (0.4064, 0.762, 1.6002) | good (gauge measurement) |
| `pressure_pa` | Pa | "Pressure (Pa)": 101325 on every row, nominal | nominal; cabin about 99 to 102 kPa |
| `air_display`, `fan_display` | display units | video overlay at the smoke point | raw readings |
| `u_air_cm_s` | cm/s | "uair" = 0.9402 x AIR display - 5.918 (sheet "Reference" F10/F11: correlation of the module anemometer with a probe 13 mm above the burner tip, Dotson thesis section 3.1.1) | moderate, about +-10 %: the duct flow drifts with soot build-up; read at the smoke point, the fan having been set before the fuel search |
| `zst_jet` | - | "mdotburner" / "mf/Zst" (the investigators' Zst of the jet mixture, sheet "Reference" column R) | good (property) |
| `tad_k` | K | "Tad" | good (property table of the investigators) |
| `rho_jet_g_m3` | g/m^3 | "HC Density" (294 K, 1 atm) | good |
| `mu_air_mean_g_m_s` | g/(m s) | "Viscosity" (air at (Tad + Tamb)/2, sheet "Reference" column W) | property, convention of the investigators |
| `air_per_jet_mol` | mol/mol | derived: `x_fuel` x (O2 for complete combustion / 0.21) | exact stoichiometry |
| T2: `mdot_jet_mg_s`, `mdot_hc_mg_s` | mg/s | "mdotburner", "mdotHC" = FUEL display x K x density / 60000 | moderate, about +-8 %: the MFC reads N2-equivalent flow and K is the gas correction factor (ethylene 0.60, propane 0.36, propylene 0.41, mixtures by 1/sum(x_i/K_i)); a bench check at Yale (`PSI-106/Raw Data/SLICE_MFC_Calib.xlsx`) found 0.775 against 0.72 for methane |
| T2: `q_jet_sccm`, `u_jet_cm_s` | sccm, cm/s | "Total Fuel Rate", "ufuel" (mean exit velocity) | as above |

Not inputs: GMT times, the radiometer reading and gain at the smoke point (they measure the flame), the
datasheet's residence-time columns (computed from the outcome length).

## 3. Counts by partition

Group = GMT day of 2009 (`docs/wsp/atlas/splits/PSI-107.json`).

| Partition | Days | Smoke points with inputs and outcome | Source of the count |
|---|---|---|---|
| train | 045, 052, 058, 119, 167, 168, 174 | **42** (045: 5, 052: 8, 058: 11, 119: 2, 167: 4, 168: 1, 174: 11) | datasheet, by script |
| validation | 136, 173 | **7** (136: 4, 173: 3); outcomes in `cases_validation_outcomes.jsonl`, never printed | datasheet, by script |
| test | 048, 055 | catalogue: 90 flow-setting rows, 25 flames (`tests.jsonl`); the script drops their datasheet rows unread | catalogue |

Train by fuel: propylene 22, 75 % propylene 7, ethylene 6, 50 % propylene 4, propane 3. Nozzles: 0.4064 mm 4,
0.762 mm 13, 1.6002 mm 25. Coflow 5 to 65 cm/s.

Not in any partition: GMT 062 (57 rows of the flight workbook, absent from the catalogue and the split, so held by
the split's rule; no smoke point of it is in the datasheet) and the KC-135 and STS-83/94 rows at the foot of the
datasheet (earlier experiments). The 2012 SPICE tests run during SLICE are in PSI-106's summary log, not here.

## 4. Files and formats

- `SmokePointReview-0g-3.xlsx` (0.3 MB, openpyxl): sheets "Smoke Point Datasheet" (55 smoke points on the 11
  days plus older experiments), "Reference" (tube IDs, K factors, properties, ruler calibrations per day),
  "Smoke Point Datasheet (2)" and "Reference (2)" (another version, with a length on the same 55 rows; not used).
- `Spice Flight Test Data II modified.xls` (0.4 MB, xlrd 2.0.1 reads it): the flight record, 526 rows on 12
  days, with SH?/LO? flags, flame length, Nikon heights, smoke height.
- `Spice_Flight_Flamelength.xls`: a ruler calibration and 12 test-point lengths (one day).
- The experiment table `..._Spice.csv` of the catalogue is the flight record without GMT 062.

## 5. Baselines on train days (not for the physicist)

From `python tools/wsp/ext/spice_cases.py` (written to `transfer_spice/baselines_train.json`). Score = quantile
score x 2 in mm, aggregate = mean of the 7 day means (pooled case mean in brackets). In-sample baselines are fitted
and scored on the same train days; leave-one-day-out (LODO) refits without the scored day.

| Baseline | T1 in sample | T1 LODO |
|---|---|---|
| B0 pooled climatology (train quantiles of `L_sp_mm`) | 11.32 (10.58) | 12.72 (12.20) |
| B1 per-fuel climatology (pooled below 4 smoke points of the fuel) | 7.08 (7.94) | 10.21 (10.39) |

Persistence does not apply (no time series; each smoke point is a separate steady state).

T2, classical flame-length laws at their published constants (point forecast: the score is the absolute error),
and with one log-normal spread sigma_ln from the train residuals (the only fitted constant):

| Law | Point | With train sigma_ln | median ln(obs/law) |
|---|---|---|---|
| Spalding / Lin and Faeth nonbuoyant jet **in coflow**, unity coefficients: L/d = Re/(16 Zst), Re from air viscosity at the mean temperature, i.e. L = mdot/(4 pi mu Zst) (Lin and Faeth 1998-99, as derived in the Dotson thesis, eq. 2.9) | **3.13 (3.42)** | **2.01 (2.26)**, sigma_ln 0.122 | +0.053 |
| Same, still-air form L/d = 3 Re/(32 Zst) (eq. 2.4) | 19.67 (16.08) | 15.04 (12.09) | -0.352 |
| Roper (1977), circular port, empirical: L = 1330 Q (T_inf/T_F) / ln(1 + 1/S), T_inf/T_F = 1 (as quoted in Turns, *An Introduction to Combustion*) | 13.38 (13.18) | 10.77 (10.88) | +0.468 |

The coflow similarity law with no fitted number is within 5 % in the median and 12 % in spread on the 42 train
smoke points; Roper's normal-gravity constant is 37 % short. A T2 law has little room to gain; T1 is where a
physical law can show its worth (per-fuel climatology: 7 to 10 mm).

## 6. What needs images or video

Nothing for T1 and T2: the outcome is in the investigators' table. Images or video would be needed for:
- smoke points the review did not tabulate, and flame lengths of non-smoke-point flames beyond the flight
  workbook: stills `SPICE_GMT#_#_NEF.zip` (2.1 GB), picture books (0.9 GB), frame grabs (0.57 GB), videos (2.4 GB);
- lift-off, from the frame grabs (`Downlink Frame Grabs - TP 31_Liftoff.zip`, day 052, 16 MB, is one).
A first batch to extend T2 on train days: the frame grabs of days 052 and 058 (45 zips, 0.34 GB).

## 7. Missing, untrustworthy, and what was seen

- Pressure and oxygen are not measured per test (cabin air; the report gives a range).
- The smoke point is a judgement on the video by the reviewing student; the length uses one ruler calibration
  (day 045) for all days although the per-day slopes differ by up to 4 %. One pixel is about 0.23 mm.
- Two train smoke points have no readable ignition time ("Ku LOS", "no ku"); their conditions are complete.
- Fuel flow and coflow carry the calibration uncertainties of section 2.
- Disclosure: while finding the structure of the two workbooks this agent printed per-day counts of filled cells
  (smoke points, lengths, SH and LO flags) for every day, test days 048 and 055 included; no value of a test-day
  or validation-day outcome was printed. The fan calibration workbook of SLICE, read for SLICE, contains no SPICE
  outcome.

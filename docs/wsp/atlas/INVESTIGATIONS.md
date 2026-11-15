# Test-level atlas: the 19 investigations, one by one

What each investigation's table is, which fields of `SCHEMA.md` it fills and from which column, what is missing
and where it could come from, and the defects found in the table. Written with `tools/wsp/atlas/normalize_tests.py`
on 14 November 2026; the numbers are those of `runs/dev/data_all/work/atlas/tests_summary.json`.

Outcome fields are described here by name, count of filled cells and unit only. Their values are in
`tests_outcomes.jsonl` and nowhere else: the evaluation groups of the 15 new investigations are not frozen.

## Overview

| PSI | Acronym | Table is | Rows emitted | Level | Groups | Oxygen and pressure, one value each |
|---|---|---|---|---|---|---|
| 10 | ACME Flame Design | two per-test tables | 327 | test | 22 | 316 (207 ambient, 109 injected oxidiser) |
| 20 | ACME BRE | log of test days | 18 | test day | 18 | 0 (oxygen as a set per day, pressure not tabulated) |
| 21 | ACME CLD Flame | log of test days | 32 | test day | 32 | 0 |
| 22 | ACME E-FIELD Flames | log of test days | 20 | test day | 20 | 0 |
| 23 | ACME s-Flame | log of test days | 18 | test day | 18 | 13 (nominal, about 1 atm) |
| 25 | BASS-II | project records | 21 | test | 3 | 0 (oxygen for all 21, no pressure in the records) |
| 26 | BASS | not reachable | 0 | | | |
| 39 | CFI | per-test spreadsheet | 227 | test | 51 | 227 |
| 47 | DAFT / DAFT-2 | two sessions | 2 | test day | 2 | 0 |
| 68 | FLEX-2 | planned matrix | 8 | matrix row | 1 | 0 (sets) |
| 69 | FLEX | project records | 156 | test | 60 | 156 |
| 98 | Saffire-I | per-sample table | 2 | test | 1 | 0 (oxygen for 1, no pressure) |
| 99 | Saffire-II | per-sample table | 9 | test | 1 | 0 (oxygen for 7, no pressure) |
| 100 | Saffire-III | per-sample table | 2 | test | 1 | 0 |
| 101 | SAME-R | per-run table | 109 | 67 tests, 42 test points | 16 | 0 |
| 102 | SAME | planned matrix | 8 | matrix row | 1 | 0 |
| 106 | SLICE | list of test days | 12 | test day | 12 | 0 |
| 107 | SPICE | flow settings inside flames | 435 | test point | 11 | 0 |
| 159 | ACME CFI-G | project records | 206 | test | 18 | 206 (109 ambient, 97 injected oxidiser) |

1,612 rows: 1,017 tests, 477 test points, 102 test days, 16 matrix rows. 918 rows have one oxygen value and one
pressure value; 712 of them are ambient oxygen and can be compared with crew atmospheres.

Two things are true of every table and are not repeated below.

- **Sequential tests share a chamber.** In the ACME and droplet tables the initial state of a test is the final
  state of the test before it. In the Flame Design tables 249 of 307 final compositions are, digit for digit,
  the initial composition of another test of the same day. An initial condition is therefore a legitimate
  condition and also carries outcome information about the previous test. This is why `group_id` is the day:
  a split must keep a day whole.
- **Basis.** Where a column's numbers are unrounded and the table says nothing about how they were obtained,
  the basis is `unstated`.

---

## PSI-10 ACME Flame Design

**Table.** Two per-test tables, one row per flame: `..._Flame Design_Normal Flame.csv` (207 rows, 13 days of
2019 and one of 2021) and `..._Inverse Flame.csv` (120 rows, 8 days from November 2021 to February 2022).

**Filled.**

| Field | Column | Basis |
|---|---|---|
| `test_id` | Test | |
| `date`, `group_id` | first five characters of Test (`D1`); the Day column where they disagree (`DAYCOL`) | derived |
| `geometry.flame_orientation` | Flame Type | nominal |
| `fuel.name` | Fuel (normal); the header "XC2H4 initial" (inverse) | nominal |
| `atmosphere.x_o2` | XO2 (init), normal only | unstated |
| `atmosphere.x_fuel_ambient` | XC2H4 initial, inverse only | unstated |
| `oxidizer_stream.x_o2` | XO2, inverse (injected stream) | unstated |
| `atmosphere.pressure` | pinit (atm) | unstated |
| `injection.*` | Xfuel, mfuel, mN2, mtot (normal); mO2, mN2, mtot (inverse), mg/s | unstated |
| `geometry.configuration`, burner diameter 6.35 mm | final report, p. 10 (investigation scope) | nominal |

60 normal rows read "RAMP" or "RAMPED" in the flow columns and 9 inverse rows in the oxygen and flow columns:
status `varying`. 2 inverse rows read "LOW O2": `unknown`, the label kept as raw value.

**Outcome fields (held).** `xo2_final` (187 cells), `xc2h4_final` (120), `pfinal_atm` (307, atm), `pmean_atm`
(307, atm), `mean_zst` (327), `tad_k` (327, K). `mean Zst` and `Tad` are held because they are computed with the
final composition (see `docs/wsp/feedback/flame_design_transfer_feasibility.md`).

**Missing.** Balance gas of the chamber; ambient temperature; time of day; the commanded flows of the ramp
tests. Source: Raw Data, the small per-test text files (`as run<T>.txt`, `<T>_mfcSetpoints.csv`, `<T>_TP.txt`).

**Defects.**
- Inverse table row 87: test `22042M1` sits in Day 6 between `22024H2` and `22024M2`. The listing has no file
  named 22042M1 and has files named 22024M1, which the table lacks: a transposed day code. The row keeps the
  table's name, gets the alias `22024M1`, and its date and group follow the Day column.

## PSI-20 ACME BRE (Burning Rate Emulator)

**Table.** `..._ACME BRE.csv`: 18 rows, one per test day, 2019 to 2021. No per-test row.

**Filled.** Date from Year, Month, Date; burner diameter from Burner Size (mm); `fuel.name` and
`injection.x_fuel_injected` from Fuel Bottle (nominal, balance nitrogen from the Readme text);
`atmosphere.x_o2` from Oxygen Conc (percent), as a list per day for 16 rows and one value for 2, nominal, flagged
`approximate` and `conditioned_on_ignition` because the table's note says the concentrations are approximate
and only those of tests that ignited are listed. Configuration `flat_porous_burner` from the Flame Design final
report, p. 10. The test-day code (`test_id`) comes from Table 1 of the Readme, which exists only as a picture and
was transcribed into `tools/wsp/atlas/transcribed/psi20_bre_readme_table1.csv` without its outcome column.

**Outcome fields (held).** `flames_ignited` (18 cells, a count).

**Missing.** Pressure: the table's note says the chamber pressures were varied and are not in the table. Flows.
Any per-test row. One test day has no raw-data file in the listing.

**Defects.**
- Rows 12 and 13 are dated "2020 Oct 17" and "2020 Dec 5". The Readme table gives 20261 (17 September) and
  20279 (5 October), and the listing has raw files for 20261 and 20279 and none for 20291 or 20340: the month
  labels of merged cells slipped when the table was flattened. The atlas uses the Readme's day and keeps the CSV
  date as an alternative.
- Rows 6 and 9: oxygen cells "3400%" and "2100%" (a percent format applied twice). Read from the Readme (34 %, 21 %).

**Next source.** Raw Data: `Final ACME Chamber Calculator - 2019-01-31 to 2019-04-09.xlsx` and
`... 2020-07-22 to 2021-02-03.xlsx` (about 0.5 MB each). The same workbook of CLD Flame, opened to check, has an
event sheet with the test point id, chamber pressure before and after (psia) and the gas sources, which gives
oxygen and pressure per test. Then the per-test `as run<T>.txt` and `<T>_mfcSetpoints.csv` for flows.

## PSI-21 ACME CLD Flame (Coflow Laminar Diffusion Flame)

**Table.** `..._CLD Flame.csv`: 32 rows, one per test day, November 2017 to March 2021.

**Filled.** Date from Test Day (`D1`); `fuel.name` from Fuel Bottle; `injection.x_fuel_injected` from Fuel conc.
Range (a range for 26 days, one value for 6; nominal; the note says it was varied between and sometimes during
tests); `extra_conditions.test_type`, `fuel_mfc_full_scale`, `n2_mfc_full_scale` (slpm, nitrogen basis).
Configuration `coflow_burner` from the title.

**Outcome fields (held).** `flames_ignited` (32 cells, a count).

**Missing.** Everything about the atmosphere and the coflow: oxygen, pressure, coflow speed, flows. The
investigation has no document on PSI besides the table.

**Defects.**
- 26 Fuel Bottle cells are spreadsheet fill-series artefacts: "40% CH4" is followed by "40% CH5" up to
  "40% CH14", "100% C2H4" by "100% C2H5" up to "100% C2H11", and so on. Each run is read as its first cell
  (rule `FILL`). On the 28 methane-based and ethylene-based days the Test Type column confirms the reading; for
  the two artefacts among the last four rows ("30% C2H5", "40% CH5") only the rule does.

**Next source.** Raw Data: `Final ACME Chamber Calculator - 2018-05-22 to 2018-09-19.xlsx` and
`... 2021-02-03 to 2021-03-09.xlsx`, which cover the ethylene and the 2021 days but not the first 14 days; and
the per-test text files (`<T>.txt`, `<T>_TP.txt`), 1,327 small files.

## PSI-22 ACME E-FIELD Flames

**Table.** `..._ACME E-FIELD Flames.csv`: 20 test days of 2018, each with one to four fuel concentrations on
continuation rows.

**Filled.** Date from Year, Month, Date (agrees with Test Day on every row); configuration from Test Type
(`coflow_burner` for 11 days, `gas_jet_burner` for 9); `fuel.name` from Fuel Bottle; `injection.x_fuel_injected`
from Fuel Conc (one value for 14 days, a list for 6; flagged `conditioned_on_ignition`, the note says only
flames ignited are listed); `extra_conditions.mesh_polarity`, `test_type`.

**Outcome fields (held).** `flames_ignited` (20 cells, a count).

**Missing.** Oxygen, pressure, coflow speed, flows, field voltage. The Data Organization Document announces a
test-conditions table on its page 2, which is empty in the PDF's text layer.

**Defects.**
- The block of test day 18319 (rows 30 to 33) repeats rows 26 to 29 exactly. Emitted once.

**Next source.** Raw Data: per-test `<T>_TP.txt` and `<T>_Comments.txt`, and
`ACMEFlowAndPressuresRawDataConversion.xlsx` (4.5 MB). Science Documents: the data interpretation document
(fetched, 0.9 MB) explains the per-test csv headers.

## PSI-23 ACME s-Flame

**Table.** `..._ACME s-Flame.csv`: 18 test days, September 2019 to July 2020.

**Filled.** Date from Year, Month, Date (agrees with Test Day*); `atmosphere.x_o2` and the diluent from
Atmosphere* ("~21% O2 with N2": one value for 13 days, two for 5; nominal, `approximate`);
`atmosphere.pressure` = 1 atm, nominal and `approximate`, from the table's note "Atmosphere is ~1 atm but
pressure increases and O2% drops with vitiation" (investigation scope); `fuel.components` from Fuel Bottle,
with the balance gas by rule `BAL`; `injection.x_fuel_injected` by rule `XF`. Configuration and burner diameter
(6.35 mm) from the s-Flame final report, p. 10; orientation `normal` from the PSI metadata.

**Outcome fields (held).** `flames_ignited` (18 cells, a count).

**Missing.** Per-test rows, flows, measured pressure and oxygen. The 13 rows on the oxygen-pressure map are
day-level nominal points, not tests.

**Defects.**
- 8 Atmosphere and Fuel Bottle cells are fill-series artefacts ("~21% O2 with N3", "50% C2H4 with N3").
- Row 11 (day 20132) is labelled "First round of tests with nitrogen-diluted fuel and oxygen" while its
  atmosphere and fuel are helium-diluted: the round label changes one row late.

**Next source.** Raw Data: per-test `as run<T>.txt` and `<T>_mfcSetpoints.csv` (576 csv files). The two
`ACME Daily Test Point Log` workbooks hold only image counts per test point, not conditions (one was opened).

## PSI-25 BASS-II (project records)

**Source.** Conditions of the 4 train and 17 validation runs of the `bass` family, read through
`tools/wsp/analysis/lib.py`. The family holds 25 runs of one principal investigator's fabric tests; 4 are in the
test partition and were not read. The rest of BASS-II is not in the project's records.

**Filled.** `atmosphere.x_o2` (calibrated initial oxygen, measured), carbon dioxide as diluent, `flow.speed`
(resolved for 8, `varying` for 13 where the record holds an untimed sequence of settings), `fan_display`,
`flow_restrictor`, `x_co`; material and configuration from the record's top-level fields; date from the session
id; group from the frozen split.

**Missing.** Pressure, flow direction, sample size (the material label says "1 cm" or "2 cm" without naming the
dimension, so no size is read). Outcomes are not read: `not_read_protected`.

## PSI-26 BASS

No row. No record of PSI-26 is reachable through `lib.py`, and the brief allows the four project investigations
to be read only through it. `docs/wsp/feedback/bass1_psi26_tracks.md` says PSI-26 is outside every partition; if
the owner confirms, its experiment table can be ingested like the others.

## PSI-39 CFI (Cool Flames Investigation)

**Table.** `..._CFI_DataSummary_Spreadsheet_Jan2021.xlsx` (Science Documents): 227 tests on the Master Sheet,
with sheets DATA, Igniter, Camera Settings and Comments. The public `..._Experimental table_CFI.csv` has 87 rows
for 79 of those tests and agrees with the Master Sheet in every compared cell; its rows are listed in
`source_rows`.

**Filled.**

| Field | Sheet, column | Basis |
|---|---|---|
| `test_id`, aliases | Master Sheet: Flex Number, CIR | |
| `date` | Master Sheet: Test Date | nominal |
| `group_id` | day of year in the CIR test-point name (`GR-CIRDAY`) | |
| `start_time` | Master Sheet: Local Time (zone label "est" kept; 223 of 227) | nominal |
| `atmosphere.pressure` | Master Sheet: Pressure ("1 atm") | nominal |
| `atmosphere.x_o2`, nitrogen, helium | Master Sheet: Oxygen, Nitrogen, Helium | nominal |
| `fuel` | Master Sheet: Fuel | nominal; mixture fractions `basis_of_fraction_unstated` |
| `geometry.configuration` | Master Sheet: Fiber (free 55, fibre 103, blank 69) | nominal |
| initial droplet diameter | DATA: D0 ( mm ), 148 tests | measured, `from_analysis_sheet` |
| `igniter_power`, `igniter_time` | Igniter sheet, 224 tests | nominal |

**Outcome fields (held).** From the DATA sheet: `dext_hot_mm` (144 cells, mm), `khot_mm_2_s` (148, mm^2/s),
`kcool_mm_2_s` (127, mm^2/s), `dext_cool_mm` (122, mm), `reignitions` (151), `extrap_cf` (149),
`hf_burn_time` (147), `peak_hf_rad` (145), `peak_water_vapor_rad` (145), `rad_ratio` (145), `2_5mm_rad` (88),
`2_5mm_narrow` (88), `cf_rad_ratio` (88), `2_5mm_cf` (65), `2_5mm_time` (96), `actual_d` (96),
`completed_analysis` (227); from the Comments sheet: `comments` (227, free text). Units are stated only where
the header gives them.

**Missing.** Carbon dioxide fraction, ambient temperature, measured pressure, flow. Source: Raw Data, the test
sheets `FLEX-<n>_TestSheet.rtf` (22 tests) and `<CIR>_Chamber_Readings.txt`. The Camera Settings sheet (which
cameras ran) is not read; it belongs to `measurements`.

**Defects.**
- CSV: 8 rows are exact repeats (87 rows, 79 tests).
- Master Sheet: oxygen + nitrogen + helium is 0.97 for Flex 1219 to 1222.
- Master Sheet: Flex 1237 is dated 2 May 2017 with CIR test point f121a02 (day 121, 1 May), between two tests of
  1 May; Flex 1301 is dated 29 July with f180d02 (day 180, 29 June), between two tests of 29 June. Both dates
  are typing errors; the date cell is kept, flagged `conflict`, and the group follows the CIR day.
- Local Time: four cells are not complete times ("011:11:09 am est", "04:42:4 pm est", "02:22:xx pm est", "n/a"),
  and the am/pm labels of some neighbouring tests are out of order (Flex 1300 to 1302). Times are copied as written.
- Not rechecked here: the earlier report found Flex 1213 with oxygen 0.17 in the table and 0.21 on its summary PDF.

## PSI-47 DAFT / DAFT-2

**Table.** `..._DAFT.csv`: 2 rows, one per session of 2005. Not a combustion test: an aerosol instrument check.

**Filled.** Date from Actual Day ("GMT 54 (2005)", rule `D2`); sample material; `hardware_description`,
`sampled_location`; configuration `aerosol_bag` from the PSI metadata; phase and flow `not_applicable`.

**Missing.** Rack (the hardware string names only the instrument), cabin atmosphere. No outcome column.

**Defects.** None in the table. The 30-day report heads session 1 "GMT 54, 2005" and says in its text that it
was performed on 22 February 2005; day 54 is 23 February. The atlas follows the table.

**Next source.** Analyzed Data (12 small xls) and the reports mention later sessions of 2006 that the table lacks.

## PSI-68 FLEX-2

**Table.** `..._Flex2.csv`: an 8-row planned matrix, one row per fuel. The campaign has 722 test points in the
listing; none is in the table.

**Filled, as sets.** Fuel and mixture fractions from Sample (Fuel Type); droplet diameter range; oxygen list
(0.17, 0.21, 0.25, 0.3); pressure list (0.5 to 3.0 atm); diluent "O2/N2 or O2/He" (status `multiple`); ambient
temperature 25 C; flow velocity range. All nominal.

**Missing.** Every per-test value. `docs/wsp/feedback/cfi_flex2_transfer_feasibility.md` found that the small
per-test files (`<TP>_MDCA_Testpoint_Settings.txt`, `<TP>_Chamber_Readings.txt`; 7,925 text files, 0.53 GB) give
pressure, temperature, the fibre flag and the flow speeds, but neither the fuel nor the gas composition.

**Defects.** None; all eight rows carry identical condition cells, so the table distinguishes the rows only by fuel.

**Next source.** Raw Data per-test text files for the test list, pressure and temperature; the fuel and gas per
test point must come from the Experiment Data record names described in the Data Organization Document
(`FLEX-2 DoD.docx`: records by test matrix, fuel composition and year) or from the Investigation Metadata zip.

## PSI-69 FLEX (project records)

**Source.** Conditions of 117 train and 39 validation runs of the `flex` (methanol) and `flex_heptane` families,
through `lib.py`. 38 runs of the test partition were not read. FLEX tests with other fuels are not in the
project's records.

**Filled.** Oxygen, nitrogen, carbon dioxide and helium fractions, pressure, ambient temperature, initial
droplet diameter, fibre flag, fuel; date from the session id; group from the frozen split. Basis `unstated`
except the diameter (`derived` in the record). 13 values carry `conflict` where the record says the table and
the test sheet disagree.

**Missing.** Flow (not a condition of the records), time of day. Outcomes are not read.

## PSI-98 Saffire-I, PSI-99 Saffire-II, PSI-100 Saffire-III

**Tables.** One row per sample: 2, 9 and 2 samples. Rows that describe flow-visualisation records and table
notes are not tests and are skipped (2 and 6 rows). The three tables have different columns and write units
differently (in the header, in the cell, or both).

**Filled.** Material; sample thickness, length, width; air flow (nominal); flow direction (concurrent or
opposed); igniter power and time (I and II); camera record names as aliases; `atmosphere.x_o2` from Percent O2
for the samples whose cell is one value (1 of 2, 7 of 9; `approximate`; basis `derived` for Saffire-II, whose
Note 1 says the value is derived from measured CO2 production and the initial oxygen reported by the station;
`unstated` for Saffire-I). Saffire-III has no oxygen column. One group per flight.

**Outcome fields (held).** Saffire-I: `burn_time_s` (2 cells, s), `percent_o2_over_burn` (1). Saffire-II:
`burn_time_s` (9, s), `ug_burn_length` (9), `ug_spread_length` (9), `1_g_burn_length` (9), `1_g_spread_rate`
(9), `percent_o2_over_burn` (2). The three oxygen cells that give a range over the burn are held
(`withheld_outcome`): the first value of each can be promoted to a condition once the groups are frozen.

**Missing.** Pressure, date, time. Source: Raw Data, the sensor workbooks per sample
(`Saffire Sensor Data Sample <n>.xlsx`, 12 to 33 MB each; pressure, oxygen, carbon dioxide and flow against
time) and the telemetry workbooks.

**Defects.**
- Saffire-I and -III give the SIBAL fabric thickness as 0.37 and ".33" in centimetres; the Saffire-II Readme
  gives 0.33 mm for the same fabric. The unit is suspect; the values are copied and flagged `defect_noted`.
- Saffire-I has length 40.6 and width 94; the metadata describes the sample as about 0.94 m long and 0.4 m wide
  and Saffire-III has 94 and 40.6: the two columns are exchanged.
- Saffire-II against Table 1 of its Readme (a picture, transcribed into
  `tools/wsp/atlas/transcribed/psi99_saffire2_readme_table1.csv`): thickness 0.27 against 0.25 mm (sample 1) and
  1.03 against 1.02 mm (sample 3); three cells read "037 mm" (decimal point lost; the Readme gives 0.36 and
  0.33 mm); air flow 20 cm/s in the table against 30 cm/s in the Readme for samples 6 and 9 (the metadata says
  20 or 30); Material blank for samples 6 and 9 (merged cells lost; filled from the Readme); sample 7 thickness
  "0.85 and 0.37 mm" against 0.33 mm; sample 8 thickness "See Fig. 5".
- Saffire-II headers: "Samle Thickness"; "?-g" for micro-g; the column named "?-g Spread Length" holds rates;
  camera record "S56C2" for sample 6; sample 4 has the same record name in both camera columns.
- Metadata: the investigation start and end dates of PSI-98 (2001 to 2008) are those of PSI-102 SAME, not Saffire's.

## PSI-101 SAME-R

**Table.** `..._SAME-R.csv`: 109 rows, the test matrix of Appendix A of the final report: 67 runs and 42
secondary aging runs (test numbers ending in "a"), which the atlas emits as test points of their run.

**Filled.** Sample Type; Date with the year 2010 from the final report (`D3`); `flow.speed` from Inlet velocity
(cm/s), with direction `quiescent` or `forced` (`F2`); `heater_resistance_ratio` from Rt/R0 (the temperature set
point: the report, p. 26, says the software holds the wire resistance at this multiple of its room-temperature
value); `heating_duration`, `primary_aging_time`, `primary_mixing_time`; `description`. Rack `MSG` from the final
report. A secondary aging run takes the group of its run (`GR-PARENT`); four are dated a day later than their run.

**Outcome fields (held).** `net_aging_time_sec` (109 cells, s). Held because it is known only after the run.

**Missing.** Sample heating temperature (the report derives it from logged resistance), cabin atmosphere.
Source: Analyzed Data, `SAME, SAME-R_Results.xlsx` (4 MB) and the per-run workbooks
`2010_<day>_<hour>_<material>_run<n>.xls`; Raw Data `HKPData_gmt<day>_<n>.csv`.

**Defects.**
- Test number 57 appears twice: row 57 (Lampwick) and row 75 (Teflon) with the same description, date,
  velocity and Rt/R0. Analysed-data file names have run 57 only as lampwick. The Teflon row is emitted as
  `57#2`, its material flagged `defect_noted`.
- For 15 runs the Date differs by one day from the GMT day of year in the names of the analysed-data files:
  the Date column is not uniformly a GMT date.

## PSI-102 SAME

**Table.** `..._SAME.csv`: an 8-row planned matrix. Rows 1 to 4 give Teflon with an air flow and an aging
duration; rows 5 to 8 read "<material>: same conditions as 1-4" and take the sets of rows 1 to 4 (rule `REF`).

**Filled.** Material; `flow.speed` (5 or 10 cm/s); `planned_aging_duration`; the period of the flight tests
(9 September to 5 October 2007, from the SAME-R report) as a date range; dibutyl phthalate as `liquid_other`
(the SAME report, p. 3: retained in a cotton wick).

**Outcome fields (held).** `weight_loss_rate_mg_min` (4 cells, mg/min). In this table the column reads as a
planned target of the matrix; it is held all the same because a weight-loss rate is an outcome by the rule.

**Missing.** Every actual run. **Next source:** the final science report (fetched, 3.7 MB) has a run table in
its appendix (run number, GMT day, material, initial or reheat, flow, time window), which would give test-level
rows; Analyzed Data has 59 workbooks by run and GMT day.

## PSI-106 SLICE

**Table.** `..._Slice.csv`: 12 rows, one per test day of February and March 2012, with the fuel of the day.

**Filled.** Date; fuel and its fraction in the bottle ("40% CH4 (balance N2)"); configuration `coflow_burner`
and direction `coflow` from the PSI metadata (flow duct, burner tubes, coflow air). One day names two fuels.

**Missing.** Everything else: nozzle, coflow speed, fuel flow, oxygen, pressure, rack (the hardware string of the
metadata is empty). No outcome column.

**Defects.**
- Three rows carry words of a sentence in the Test Date cell ("2012-FEB-13 SLICE testing with"), and the last
  row has the first of its two fuels there ("... SPICE testing with 100% C3H8 and then 100%" | "100% C2H4"):
  a sentence was split across the two columns.

**Next source.** Raw Data: `pic_info_<day>*.csv` (a few KB per day, per-picture settings) and
`SLICE_BoundaryCondition.xlsx`, `CoflowBC.xlsx`, `Fan_cal.xlsx`.

## PSI-107 SPICE

**Table.** `..._Spice.csv`: 469 rows on 11 GMT days of 2009. A row is a fuel and air flow setting held inside
one flame; a flame is a day and an ignition time, and has from one to twelve rows. The table does not say which
rows are smoke points and gives no flame length. 435 distinct rows are emitted as test points; rows with a
readable ignition time name their flame in `parent_test_id` (135 distinct day-and-time pairs; a flame written
once with ":" and once with "." a second apart counts twice).

**Filled.**

| Field | Column | Basis |
|---|---|---|
| `date`, `group_id` | GMT date (day of year); year 2009 from the final report, p. 19 | derived (`D2`) |
| `start_time` | GMT ignition (423 of 435) | measured |
| `fuel` | Fuel ("75 % Propylene": balance nitrogen from the report) | nominal |
| nozzle inner diameter | Nozzle (mm) | nominal |
| `flow.speed`, direction `coflow` | Air Rate (cm/s), 415 of 435 | measured |
| `injection.total_mass_flow` | Fuel Rate (mg/s) | measured |
| `injection.fuel_mass_flow` | HC Fuel Rate (mg/s) | measured |
| `fuel_volume_flow_std` | Fuel Rate (sccm) | measured |

The flow columns are treated as conditions: the report (p. 19) says the flow conditions were read from the
video record, and a row without a smoke-point mark does not reveal a result. This is a judgement; if the rows
turn out to be smoke points, the fuel rate is an outcome and moves to the held file.

**Outcome fields (held).** `gmt_flame_off` (424 cells, a time of day).

**Missing.** Oxygen and pressure: the glovebox was flushed with cabin air, whose oxygen the report gives only as
a range for the station, not per test. Flame length and the smoke-point mark.

**Defects.**
- 34 rows are exact repeats of another row.
- Times are written with two separators ("14:37:08" and "14.37.08"), often both for the same flame a second
  apart, which suggests two transcriptions of one record merged into one table. 12 distinct rows have no
  readable ignition time ("no ku", "Ku LOS", "LOS", "1744", "n/a").
- 7 distinct rows have blank fuel-rate cells and a hydrocarbon rate of 0 (a formula over empty cells); 20 have
  no air rate; 8 air rates are negative.
- The table has 11 days; the final report speaks of twelve sessions.

**Next source.** Raw Data: `Spice Flight Test Data II modified.xls` (0.4 MB), the report's Appendix A with the
measured flame data, and `SmokePointReview-0g-3.xlsx` (0.3 MB). The first was fetched and not parsed: it is a
legacy `.xls` and no reader for that format is installed.

## PSI-159 ACME CFI-G (project records)

**Source.** Conditions of 154 train and 52 validation runs of the `acme_normal` and `acme_inverse` families,
through `lib.py`. 54 runs of the test partition were not read.

**Filled.** Fuel species; orientation; `atmosphere.x_o2` (normal) or `x_fuel_ambient` and
`oxidizer_stream.x_o2` (inverse); pressure (`approximate`, the header says so); injected fuel or oxygen mass
flow; `zst`, `tad`; date and group from the session and the frozen split.

**Missing.** `geometry.configuration` and therefore `labels.flame_type`: the records hold only the orientation
and do not name the burner. The balance gas of the chamber. Source for the burner: the investigation's report,
which was not opened because it holds outcomes of a protected investigation. Outcomes are not read.

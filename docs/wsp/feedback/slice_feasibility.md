# SLICE (PSI-106): what can be forecast and scored without opening an image

> Written by a delegated data agent (claude-opus-5-5) on 15 November 2026 for the seventh generation (brief
> `runs/dev/gen7/BRIEF_DATA.md`); nothing fitted except the constants of the named baselines on train days.
> Cases script: `tools/wsp/ext/slice_cases.py`. Case files: `runs/dev/data_all/work/transfer_slice/` (local, not in git).
> **For the physicist:** read only the paragraph below and sections 1 to 3. Section 5 holds train scores.

**For the physicist.** A forecast is asked for **`blow_off`**: the probability that a laminar coflow jet
diffusion flame in microgravity (round burner tube in a 76 mm fan-driven air duct, MSG, cabin air) **blows off**
at a logged flow condition, that is at the recorded setting or during the adjustment noted on that line of the
crew log. It is asked from these inputs, as named in the case files (`inputs.*`): `fuel_hydrocarbon` (methane,
ethane, propane) and `x_fuel` (hydrocarbon mole fraction, the rest N2: methane at 1.0, 0.7 and 0.4), `mw_jet_g_mol`,
`burner_id_mm` (measured inner diameter, 0.41 to 3.23 mm; `burner_nominal_mm` 0.4, 0.8, 1.6, 2.1, 3.2),
`q_jet_sccm` and `mdot_jet_mg_s` (jet gas flow), `u_jet_m_s` (mean jet exit velocity, m/s), `u_coflow_cm_s`
(duct air velocity, cm/s), `x_o2_cabin` (cabin O2 mole fraction, when logged), `cabin_p_mmhg`, plus
`step_in_test` (1 for the first condition of a flame). Form: **one probability per case**, in [0, 1]. A forecast
for a condition may use that condition and the earlier conditions of the same flame, never the later ones nor
how many conditions the flame has (the last logged line of a flame is often its end). A second target
**`lifted`** (the flame is lifting or lifted at that condition) is asked in the same form; its labels are weaker.

## 1. Forecast targets

| | blow_off (primary) | lifted (secondary) |
|---|---|---|
| Definition | the flame blows off (or blows out) at this logged condition | the flame base has left the burner rim at this condition |
| Unit | yes / no | yes / no |
| Why it matters | the blow-off limit bounds the jet and coflow speeds at which a leak flame can survive; inside it a jet fire stays lit | a lifted flame is the precursor of blow-off and changes where heat is released |
| Where recorded | Appendix B "SLICE Summary Log" of `Raw Data/SLICE-RPT-002_Post Flight Report-.pdf`: X under "Blow Off", or the NOTES on the line say blow off / blowout | X under "Lifting" or "Lifted", or the NOTES say lifted |
| A forecast may know | the condition's inputs and the earlier conditions of the same flame | same |
| Cutoffs, horizons | none (steady conditions, no sensor time series) | none |
| Score | Brier, per day and day-equal aggregate | Brier |

A case is one logged line: one (fuel flow, fan) setting of one flame. The crew stepped the fan (coflow) or the
fuel flow up, line by line, until the flame blew off, the fuel ran out, or the ground ended the flame. An unmarked
line is read as an attached, burning flame, because the next setting of the same flame was logged. Lines ending
because the fuel ran out, and two lines whose FAN display cannot be read, have `eligible = false` and are not
scored. When the note gives the blow-off point as a FUEL display ("Blow off @ Fuel = 160"), the event is put on
that line and the value is kept in `outcome.event_fuel_display`.

Candidates weighed:
- **Lift-off height** and **flame length**: measured by the investigators from the stills ("lift-off height
  measurements for lifted flames", final report 2017, section C), but no table of them is in any text, table or
  document file. Only in images (section 6).
- **Lift-off and blowout velocity limits per fuel and tube**: plotted in `SLICE EP-02 2012 ASGSR.pdf` (figures,
  no numbers in its text).
- **Attached / lifted / blown off as three classes**: the "Attached" column is never marked, so the three-way
  label reduces to the two yes/no targets above.

## 2. Inputs

All are set before or at the condition; none is measured on the flame.

| Name in case file | Unit | Source | Trust |
|---|---|---|---|
| `fuel_hydrocarbon`, `x_fuel`, `mw_jet_g_mol` | -, mol/mol, g/mol | log columns "Fuel (bottle)"; Table 4 of the final report 2017 (bottle compositions) | good, except GMT 061: the log says "100% Methane", Table 6 and the bottle count say 40 % CH4; the table is taken (`fuel_source`, `fuel_log_text` keep both) |
| `burner_nominal_mm`, `burner_id_mm` | mm | log "Burner tube ID"; measured ranges of Table 5 (midpoint) | good (+-0.006 mm) |
| `fuel_display` | MFC display | log "FUEL display" | raw |
| `k_mix`, `q_jet_sccm` | -, sccm | display x K, K = 1/sum(x_i/K_i) with K = 0.72 (CH4), 0.50 (C2H6, standard MKS factor, not in an on-disk file), 0.36 (C3H8), 1.00 (N2), the convention of the SPICE datasheet (Dotson thesis Table 3.2) | moderate, +-10 to 15 %: the Yale bench check (`SLICE_MFC_Calib.xlsx`) gives K = 0.775 for CH4 and an N2 flow 9 to 19 % above the display |
| `mdot_jet_mg_s` | mg/s | q x M p/(R T) at 21 C, 1 atm | as above |
| `u_jet_m_s` | m/s | q / (pi d^2/4), MFC standard conditions | as above; cabin pressure about 2 % below 1 atm is not applied |
| `fan_display` | display | log "FAN display" (55 to 139) | raw |
| `u_coflow_cm_s` | cm/s | the day's end-of-session fan calibration, FAN display -> u2, linear interpolation (`SLICE fan calibration 20140828.xls`, sheet "SLICE summary"; GMT 040 its own block; GMT 048 the mean of start and end) | moderate, +-10 %; soot build-up lowered the duct flow within sessions (the workbook's own notes) |
| `cabin_o2_mmhg`, `cabin_co2_mmhg`, `cabin_p_mmhg`, `x_o2_cabin` | mmHg, - | log "ISS O2", "ISS CO2", "ISS Pressure", written once or twice a day and carried forward | moderate; missing on the first lines of some days (14 train lines) |
| `step_in_test`, `test`, `gmt_time` | - | log | bookkeeping |

Not inputs: the radiometer display and gain (they measure the flame), the NOTES, the X marks.

## 3. Counts by partition

Group = test day (`docs/wsp/atlas/splits/PSI-106.json`; GMT day of 2012 in the case files).

| Partition | Days (GMT 2012) | Logged conditions | Source |
|---|---|---|---|
| train | 040, 048, 058, 061, 069, 072, 079 | **289** lines of 78 flames (040: 28, 048: 22, 058: 49, 061: 40, 069: 80, 072: 43, 079: 27); 282 eligible, with 15 blow-offs and 8 lifted | log, by script |
| validation | 065, 075 | **75** lines (065: 34, 075: 41); outcomes in `cases_validation_outcomes.jsonl`, never printed | log, by script |
| test | 044, 051, 083 | 3 days in the catalogue; their log pages (pdf pages 32-33, 36-37, 58-61) are never opened, so no per-line count | catalogue |

Train by fuel and tube (lines): methane 100 % on 0.4 / 0.8 / 1.6 / 2.1 / 3.2 mm: 35 / 55 / 14 / 5 / 11; methane
70 % on 0.8 / 1.6 / 3.2: 13 / 8 / 22; methane 40 % on 0.4 / 1.6: 29 / 11; ethane on 0.4 / 0.8 / 1.6 / 2.1 / 3.2:
20 / 12 / 9 / 16 / 23; propane on 2.1 / 3.2: 1 / 5. Jet velocity 0.08 to 53 m/s, coflow 4 to 58 cm/s.

Held: GMT 055 (24 February, SPICE testing with propane, pdf pages 38-39) is missing from the experiment table
and therefore from the split; by the split's rule it is held and was not parsed. The 2012 SPICE flames of GMT
069 and 072 are in the train lines (same apparatus, same log).

## 4. Files and formats

- The log is a 1827 or 1883 pt wide table drawn in the pdf; the observation headers and the X marks are rotated
  characters. `slice_cases.py` reads them with pdfplumber by position (column x from the header letters, rows
  from the FUEL display column, X within 14 pt of a row, notes within 30 pt below). The lines of pdf pages 30-31,
  34-35 and 40-44 were compared by eye with the extracted pdf words; the rest is unaudited.
- Calibrations: `SLICE fan calibration 20140828.xls` (per-day fan blocks), `Fan_cal.xlsx` (GMT 040 only),
  `CoflowBC.xlsx` and `SLICE_BoundaryCondition.xlsx` (velocity profiles across the duct), `SLICE_MFC_Calib.xlsx`.
- `pic_info_ISS.txt.docx` lists, per still, test point, burner, fuel, FUEL/AIR/FAN displays, time and exposure:
  the link between a log line and its pictures, needed for any image measurement.
- 13,917 `.dat` files on disk are analysed arrays per picture (colour planes `color_#_#.dat`, temperature
  `T_#_*.dat`, soot volume fraction `fv_#.dat`, CH* `CH_sig_#.dat`), in folders named by day
  (`Legacy/OMA Format/<day>_..._data`; e.g. `059_UT058_...` holds GMT 058). The raw picture arrays `#.dat`
  (34 GB) are not on disk.

## 5. Baselines on train days (not for the physicist)

From `python tools/wsp/ext/slice_cases.py` (`transfer_slice/baselines_train.json`). Brier, aggregate = mean of
the 7 day means (pooled line mean in brackets), 282 eligible lines.

| Baseline | blow_off | lifted |
|---|---|---|
| incidence, pooled (p = 0.053 / 0.028), in sample | 0.0535 (0.0504) | 0.0258 (0.0276) |
| incidence, leave one day out | 0.0544 (0.0513) | 0.0263 (0.0282) |
| incidence by nominal tube (blow-off p: 0.4 mm 0.111, 0.8 0.063, 1.6 0.025, 2.1 and 3.2 0) | 0.0516 (0.0484) | 0.0249 (0.0267) |
| "never" (persistence: the previous condition of the flame burned) | 0.0567 (0.0532) | 0.0265 (0.0284) |

Blow-offs are concentrated on GMT 048 (70 % CH4, 0.8 mm), 058 and 061 (0.4 mm methane): day Brier 0.13, 0.12,
0.07 against under 0.03 elsewhere.

No classical correlation is used at published constants. The common blow-out correlations (Kalghatgi 1981, and
its successors) are for turbulent lifted jets in still air and do not apply to these laminar jets
(jet Re below 1000 on 288 of 289 train lines, median about 170). For laminar flames, the lift-off criterion of Chung and Lee (1991) is qualitative (a
free laminar jet flame lifts only for fuel Schmidt number above 1, which methane and ethane do not have; in coflow
they lift). The investigators' own conclusions (EP-02: microgravity limits independent of tube diameter, lower
jet velocity at the limit with more coflow and more dilution) are statements about this data and are class L.

## 6. What needs images

- **Lift-off height (mm) of lifted flames and flame length**, the quantities SLICE was built to measure. They
  can be read either from the analysed colour-plane arrays already on disk (no download; train-day folders for
  040, 058, 069, 072 and 048; their format is in `read_oma.m.docx` and `Calib_const_ISS.txt.docx`) or from the
  stills. A first batch of stills for lifted flames on train days: `GMT 2012_058_D300s_NEF.zip` (0.73 GB) and
  `GMT 2012_061_D300s_NEF.zip` (1.94 GB), **2.7 GB**, with the scale proof sheets (under 1 MB each); all train
  days' NEF archives are 5.5 GB (040 1.20, 048 0.35, 058 0.73, 061 1.94, 069 0.54, 072 0.37, 079 0.35). The
  link from a picture to a log line is `pic_info_ISS.txt.docx` (time and displays).
- The whole SLICE image set is 73 GB; video (`UT_*.zip`, `DIMSG*.mp4`) another 12 GB and not needed for either
  target.

## 7. Missing, untrustworthy, and what was seen

- Labels come from X marks and free-text notes placed by the console operator; a blow-off that was not written
  down is read as "burning". The lifted label is weaker still: only 8 lines, and the log never marks "Attached".
- Rows were associated with notes and marks by position; a note written one line low would move an event by one
  condition. A spot check is advised before a registered score.
- GMT 061 fuel conflict (section 2). GMT 079 holds two propane flames (tests 112 and 117) whose numbers repeat
  those of the held GMT 055; they carry GMT 079 stamps and are kept on 079.
- No per-line pressure; O2 and CO2 are cabin values once a day.
- The coflow at the flame is the duct mean from a calibration done once or twice a day; the duct flow fell with
  soot within some sessions.
- Disclosure: to place the log pages by day, this agent printed, for every log page, the GMT day stamps and a
  count of characters in the notes area, test-day pages included, and the text of the two GMT 055 pages' fuel
  column; no mark or note of a test-day page was printed. The fan calibration workbook it read has blocks for
  every day, test days included (input calibrations, not outcomes).

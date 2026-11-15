# BASS (PSI-26) and the non-fabric burns of BASS-II (PSI-25): what can be forecast from the text files

> Written by a delegated data and scoring agent (Opus 5.5) on 15 November 2026 for brief
> `runs/dev/gen7/BRIEF_DATA.md`, slugs `bass1` (PSI-26) and `bass2_solids` (PSI-25 PMMA films, slabs, rods,
> spherical sections, wax candles and slabs). Nothing is fitted except the constants of the named baselines on
> train days. No video, image or track of a held burn was opened. Paths under `runs/dev/` are local, not in git.

## For the physicist (sections 1 to 4 hold no outcome value; stop at the marked line)

A forecast is asked for **`quench_u`: the duct air speed, in cm/s, below which the flame on this sample goes out
when the forced flow is lowered**, the low-speed quenching branch of the flammability map, one number per burn.
It is to be forecast from these inputs, with the names and units of `cases_train.jsonl` and
`cases_validation_inputs.jsonl`: `material_class`, `fuel`, `shape`, `diameter_m`, `width_m`, `thickness_m`,
`sides_burning`, `rod_colour`, `configuration` (opposed, concurrent, stagnation, forward stagnation, wake, or
null), `x_o2_initial` (mole fraction), `pressure_atm` (null on PSI-25: take 1 atm), `co2_mmhg`,
`flow_restrictors`, `u_cm_s_at_ignition` (cm/s), `prior_burns_of_sample` (count), and `fan_curve`, the table
that turns the fan display (volts x 10) into cm/s for that burn. The form is a list of **quantiles at 5, 25, 50,
75 and 95 %, in cm/s**, keyed by `case_id`. The outcome is an interval (the flame held one setting and was out
at the next lower one), so a quantile inside the interval costs nothing. A second, smaller target is
**`sustain`**: the probability that the sample ignites and keeps burning after the igniter is switched off,
from the same inputs. Its form is one probability per `case_id`. There is no time series and no cutoff: both are
forecast before the burn starts.

## 1. What the text files record per burn

There is no sensor file. The fan, anemometer and radiometer readings were written down by the operators, and
the outcome is in their words.

| Source | Burns | What it holds per burn |
|---|---|---|
| `PSI-26/Science Documents/PSI-26_Science Documents_As Run_Test Matrix.xlsx`, sheet `as run matrix` (the sheets `2012` and `2013` repeat it without the observations) | about 120 burn rows, 2012-03-30 to 2013-06-01 (the experiment table `PSI-26_Experimental table_BASS.csv` lists the same 122 with date, test, sample and material only) | date, as-run and test number, sample number, material words, igniter, flow restrictors, sample distance, **fan display** (one value or the sequence of settings: `66, 47, 45, 43, 42`, or pots `0.5, 0.4, 0.35`), **air display** (cm/s), GN2 pot, O2 fraction, pressure (atm), CO2 (mmHg), camera settings, and **"Test Observations and Anomalies"**: free text |
| `PSI-25/Science Documents/PSI-25_Science Documents_BASS-II_Test.Matrices.xlsx`, one sheet per investigator | 129 rows (B, F, M, O, P and T tests), 2014-02-14 to 2018-06-24 (the 2016 and 2018 rows are three rods and two fabric burns) | date, test, sample, material, restrictors, fan display and air display (as in PSI-26, sometimes with words such as `slow decr to ext` or `blowoff`), calibrated initial and final O2 (vol %), CO2, CO. **No observation column.** |
| `PSI-25/Reports/PSI-25_Reports_BASS-II Summary Report.pdf` (NASA/TM-20210011385), Appendix A, pdf pages 111 to 121 | 126 of the 129 tests (not O20 to O22) | the full as-run matrices, with the flow direction (Opposed, Concurrent, Stagnation), the uncorrected O2 reading beside the calibrated one, and **"Test observations and anomalies"**. Parsed with `pdfplumber`; one row per test id. |
| `PSI-25/Reference Data/PSI-25_Reference Data_Fan calibration data.xlsx` | 25 calibration sessions, 14 Feb to 4 Aug 2014 | pot, fan display and air display, per restrictor count (0, 1, 2). No calibration of the 2012-2013 BASS days is on disk. |
| `PSI-25/Science Documents/PSI-25_Science Documents_BASS-II_ReadMe.docx` | | the meaning of the columns: fan display = fan volts x 10 (max 122), pot 0 to 10 on the control box, air display = duct speed in cm/s with a response of about one minute, O2 corrected by a constant offset from the ISS MCA, video date = actual date + 1 |
| SRD, ERD, planned test matrix (PSI-26) | | sample geometry by test number: spheres 1 and 2 cm, wake spheres behind inert hemispheres, thick flat PMMA 1 cm wide 2 mm thick one-sided (test points 66 to 73), wax on fiberglass 1 cm x 2 mm (74 to 77), Japan wax and paraffin candles in tubes |

**What the words give, per burn.** Whether it ignited and kept burning (almost every row says it). How it ended,
for about half of the burns that burned: "went out at pot 0.31", "extinguished when fan set to 40", "flame out
at 0.32", "extinguished at 12 cm/s", "out when fan off", "burned entire sample", "blowoff at pot 6.0", or
nothing. Burn time and spread are mentioned in a few rows only ("~40 s burn", "spread maybe 1 cm"). **No regression
rate, spread rate or flame length is recorded in any text file as a number per burn.** The PSI-26 image sidecars
(`Raw Data/*_txt.zip`, `*_xmp.zip`) are camera metadata (exposure, time stamp) and carry no outcome.

**The flow history is part of the outcome.** The operators lowered the flow step by step while watching the flame
and stopped when it went out. The sequence of settings after ignition is therefore not an input. Only the
setting at ignition, chosen before the burn, is.

## 2. Targets

### 2.1 `quench_u`: the low-flow extinction speed (primary)

**Definition.** For a burn whose flame was sustained and whose forced flow was lowered (or held), the duct air
speed U at which the flame went out, as an interval `[quench_u_lo_cm_s, quench_u_hi_cm_s]`. The lower end is the
setting at which the words say it went out. The upper end is the setting it held just before. A burn that held
every setting down to the lowest and went out only when the fan was stopped, or burned to completion, gives
`[0, lowest setting held]`. The unit is cm/s.

**Where it is recorded.** The observation words and the fan and air cells (section 1), coded by fixed rules in
`tools/wsp/ext/bass1_cases.py` (`outcome_of`, `extinction_value`, `steps_of`). The class and the rule that fired
are kept with each outcome (`quench_class`, `quench_rule`).

**Settings to speed.** A setting written as a pot is turned into a fan display by the workbook's own pot-to-fan
table, and a fan display into U by a calibration curve:

- **PSI-25:** the latest BASS-II calibration on or before the burn date, for the burn's restrictor count.
- **PSI-26:** the median of the 2014 calibrations.
- **Low settings:** below pot 1 (fan display 46.5) the anemometer reads 0 to 4 cm/s with a one-minute lag, so U
  there is taken linear in the fan display between the stopped fan (38, 0) and the reading at pot 1.
- **Air readings:** where the operators wrote an air reading of at least 2 cm/s for the same step, that reading
  is used.

Every case carries its curve (`fan_curve`, `fan_curve_source`), and `fan_curve.json` holds them all.

**Which burns are scored.** Classes `out_at_finite_flow`, `out_at_fan_off` and `burned_to_completion`, without
nitrogen applied during the burn (`n2_applied`). These are left out:

- burns with nitrogen applied during the burn;
- blowoff (the high-flow branch);
- burns that did not sustain;
- burns whose words do not say how the flame ended (`end_not_stated`);
- burns that went out by themselves at a held flow with no value written (`self_extinguished`);
- extinction with no higher setting held (`out_no_upper`).

**What a forecast may know.** The inputs of section 3, all set before ignition. There is no cutoff or horizon.

**Score.** Interval CRPS from the five quantiles: 2 x the mean over the levels of the pinball loss, where the
loss is 0 inside the interval and measured to the nearer end outside it. On a point outcome it is the usual
quantile score. The primary aggregate is the mean over test days of the day mean, as in the project. The case
mean is reported beside it. `score()` in the script returns both, the per-day means and the number of refusals.

### 2.2 `sustain`: ignited and kept burning (secondary)

**Definition.** 1 if the words say the sample burned after the igniter was off, 0 if they say it did not ignite
or did not sustain ("no ignition", "would not sustain", "brief ignition ... extinguished", "died when igniter
turned off", "went out when igniter was retracted"). No code is given where the igniter failed before any flame
or the words say nothing about the flame. **Score:** Brier. It is recorded for 165 of the 172 train and
validation burns, but it is unbalanced by material (section 5).

**Candidates weighed and not taken.**

- **Burned to completion:** written for 7 train burns only, and decided by the operators, who usually turned
  the flow down or off first.
- **Regression or spread rate, flame length:** not in any text file.
- **The high-flow blowoff speed:** 10 train burns, almost all PMMA rods and films on two days.
- **The fabric ignition outcome:** already the target of PROTOCOL 12.9, from video.

## 3. Inputs

All are known before the burn starts.

| Name in the case files | Unit | Source | Trust |
|---|---|---|---|
| `material_class`, `fuel`, `shape` | | material words. PSI-26 "PMMA 1 cm" / "PMMA 2 cm" are spheres (sample numbers and the SRD's test points); PSI-26 "PMMA slab" is the SRD's thick flat sample | good for PSI-25, where the words name the geometry; PSI-26 slab sizes are the planned ones |
| `diameter_m` | m | spheres 0.01 / 0.02, rods 1/4" 3/8" 1/2" (6.35, 9.525, 12.7 mm), spherical section 0.04 | nominal |
| `width_m`, `thickness_m`, `sides_burning` | m, m, count | films "100 micron ... 2 cm wide", slabs "3 mm thick pmma, 1 sided, 2 cm"; PSI-26 flat PMMA and wax 0.01 x 0.002, one-sided (SRD) | nominal. A PSI-25 slab with no "1 sided" is taken as two-sided |
| `rod_colour` | | clear or black (black rods absorb in-depth radiation differently) | good |
| `configuration` | | PSI-25: the appendix column (O1 and O2 are written "?"); PSI-26: "wake" or "opposed" in the words, else forward stagnation for spheres and candles, null for flat samples | PSI-26 flat fabric and slabs: null, not guessed |
| `x_o2_initial` | mole fraction | PSI-26: as-run "O2 % by volume", read before the burn (0.159 to 0.227; 2013 days were vitiated with N2 to 0.16 to 0.18); PSI-25: "calibrated initial O2" / 100 (0.136 to 0.222) | PSI-25: the sensor was out of calibration and was offset to the ISS MCA. The appendix's uncorrected reading is 1.3 to 1.6 points lower (22.2 against 20.6), so take +-0.015 |
| `pressure_atm` | atm | PSI-26 only (0.97 to 1.01) | null on PSI-25 (cabin, about 1 atm) |
| `co2_mmhg` | mmHg | PSI-26 as run; PSI-25 initial CO2 vol % x 7.6 | rises during a day |
| `flow_restrictors` | count | as run; 14 cells empty, taken as 2 (`flow_restrictors_source` says so) | good |
| `ignition_setting`, `u_cm_s_at_ignition` | words, cm/s | the first setting of the fan or air cell | the ignition setting is chosen before the burn |
| `prior_burns_of_sample` | count | earlier burns of the same sample number in the train and validation rows (re-burns are preheated or partly consumed) | test-day rows are not read, so a burn after a test-day burn of the same sample is undercounted |
| `fan_curve`, `fan_curve_source` | [[fan display, cm/s], ...] | section 2.1 | campaign curves for 1 and 2 restrictors come out nearly equal (the workbook's later days fill the 1-restrictor block sparsely); only 10 burns use 0 or 1 restrictor |

**Not inputs:**

- the later fan and air settings (the operators' response to the flame);
- the GN2 pot during the burn;
- the final O2, CO2 and CO;
- the frame counts and radiometer readings.

**What the earlier solids kernel wanted.** `crates/ignis-wsp/src/property_solids_v2.rs` takes, per material:

- reactant loading, heat capacity, pre-exponential factor, activation energy and reaction enthalpy;
- inert loading, heat capacity and conductivity;
- the grid length, thickness and width;
- the flow velocity vector, gravity and O2 fraction;
- the pressure, gas and radiant temperatures, heat transfer coefficient, emissivity and absorbed flux.

The case files give the geometry, oxygen, pressure and flow. Every material property and every boundary value
has to come from the literature, and none is in these files.

## 4. Day groups and counts

Three frozen files. Each was written from dates (and, for PSI-25, material words) before any outcome of its held
days was read, with the project's rule and seed (`tools/wsp/atlas/freeze_groups.py`, `split_groups`, seed
20261107).

| Stratum | Split file | Train days | Validation days | Test days |
|---|---|---|---|---|
| PSI-26 2013 (video days) | `docs/wsp/atlas/splits/PSI-26-days.json` (frozen earlier; as-run date + 1 = video date, the 1 June burns are in videos of 31 May) | 7 | 2 | 3 |
| PSI-26 2012 (no video) | `docs/wsp/feedback/bass1_PSI-26-2012-days.json` (new, a proposal beside the frozen file) | 13 | 4 | 4 |
| PSI-25 solids | `docs/wsp/atlas/splits/PSI-25-solids-days.json` (new). The rule alone, then three days that the project's fabric split already holds (27 June test; 9 July and 4 August validation) keep the stricter partition | 13 | 7 | 6 |

Burns per partition. Test counts come from the date (and the material cell) only; no other cell of a test-day row
was kept or read.

| Stratum | Train burns | with `sustain` | with `quench_u` | Validation burns | with `sustain` | with `quench_u` | Test rows |
|---|---:|---:|---:|---:|---:|---:|---:|
| PSI-26 2012 | 50 | 48 | 14 | 15 | 15 | 5 | 15 |
| PSI-26 2013 | 21 | 20 | 9 | 9 | 9 | 0 | 11 |
| PSI-25 solids | 60 | 57 | 32 | 17 | 16 | 6 | 22 |
| **All** | **131** | **125** | **55** (25 days) | **41** | **40** | **11** | **48** |

Three PSI-25 rows (O20 to O22, 26 February 2016) fall on no frozen day and stay held. Every case has an oxygen
value.

---

**Below this line: outcome aggregates of train days. A zero-shot author does not read further.**

## 5. Baselines on train days

Run through `score()` by `python tools/wsp/ext/bass1_cases.py baselines`, which writes
`runs/dev/data_all/work/transfer_bass1/baselines_train.json`. All baselines are in-sample.

| Target | Baseline | Constants fitted on train | Day mean | Case mean | Cases / days |
|---|---|---|---:|---:|---:|
| `quench_u` | constant quantiles | q05..q95 = 0.66, 1.20, 1.44, 1.66, 9.06 cm/s | 0.552 | 0.771 | 55 / 25 |
| `quench_u` | **classical correlation**: critical opposed-flow velocity for radiative extinction | log-normal width 0.40 around it | 0.613 | 0.895 | 55 / 25 |
| `sustain` | incidence | p = 0.888 | 0.0881 | 0.0995 | 125 / 31 |
| `sustain` | always sustained (the 12.9 carry's assumption) | none | 0.0973 | 0.1120 | 125 / 31 |

**The classical correlation.** Bhattacharjee et al., Combust. Flame 163 (2016) 472-477, as restated in the
summary report (section 2.2.7, eq. 2.5):

V_cr = eps sigma (T_v^4 - T_inf^4) / (rho_g(T_v) c_g (T_f - T_v))

It is used at the published T_v = 673 K and T_f = 2199 K at 21 % O2. Not found on the pages read, and set
without fitting:

- eps = 1;
- T_inf = 300 K;
- c_g = 1070 J/kg/K;
- T_f - T_inf proportional to the oxygen fraction;
- rho_g = p / (R T_v).

It gives 1.30 cm/s at 21 % O2 and 1.71 at 17 %.

**By material.** On the 47 scored burns that are not thick PMMA slabs it is better than the constant (case mean
0.132 against 0.202). On the 8 thick slabs, which went out at 5 to 18 cm/s, it is far worse (5.38 against 4.12).
The slabs carry most of the loss of both baselines (days 2014-02-20 and 2014-07-11). A law that does not
distinguish thermally thick slabs will be judged mostly on them.

**Persistence** does not apply: there is no series within a burn, and re-burns of one sample are rare.

## 6. How well the text rules code the outcome (train audit)

`python tools/wsp/ext/bass1_cases.py show-train` prints, for every train burn, the words beside the coded
outcome. I read all 131.

**No miscoding found among the 55 scored burns.** Each interval matches a hand reading of the words and cells.

The rules lose about ten burns a hand reader could use:

- 117a (2013-04-09): "off" only in the fan cell.
- 26 (2013-04-16): survived pot 0.33; the end is not stated.
- 105 (2013-04-23) and 118 (2013-04-29): "out at finite flow" with no value.
- Slabs and wax 66, 68 and 74 (2012): "extinguished by itself" at a held setting that appears only in the fan
  cell.
- M2 ("self- extinguished", split by the pdf) and M9 (died after the flow was raised).

Settled by choice:

- "N2 extinguishment" on 2012-04-09 test 45 is ambiguous; it is not flagged.
- Vitiation before the burn (2013) is not counted as nitrogen during the burn.
- Five burns held a lower setting during ignition and went out after the flow was raised and lowered again
  (M3, M11, M14, O15b, wax 78). There the upper end is the setting held just before, flagged in
  `quench_history`.

**Validation outcomes** were coded by the same rules, written to `cases_validation_outcomes.jsonl` by the script,
and not looked at.

## 7. What needs video or images, and the cost

**Quench setting where the words are silent** (36 train burns `end_not_stated`, more on held days):

- The FAN and RAD (radiometer) boxes of the video overlay give it. The radiometer falls to its background when
  the flame goes out, and FAN at that second is the quench setting.
- The once-per-second overlay strips are kept: `runs/dev/media/fabric/ovl/`, 136 PSI-25 and 114 PSI-26 files,
  6.4 GB on disk.
- The glyph reader exists (`tools/wsp/media/fabric/overlay.py`; RAD's last digit is cut in most recordings).
- **Cost: no download, CPU only**, plus a rule frozen on train burns.
- This works for PSI-25 and the 2013 PSI-26 burns. **The 2012 PSI-26 burns have no video**, only top-view stills
  without an overlay, so for them the text is all there is.

**Spread or regression rate and flame length:**

- Re-fetch the main-view videos from `runs/dev/media/fabric/manifest.json`: PSI-25 8.21 GB and PSI-26 22.79 GB,
  31.0 GB in all.
- A geometry-specific scale is needed: the rod or sphere diameter as an in-frame ruler. The measurer's 2-cm
  fabric scale and sample-end rule do not transfer (MEASUREMENT_REPORT.md section 6: igniter-like bursts from
  long bright PMMA burns, 1-cm samples).
- A method frozen on train burns is needed.
- The second view (PSI-25, 9.0 GB) and the still sequences (PSI-25 about 231 GB, PSI-26 140.7 GB) would add a
  top view. The 2012 spheres and candles could be sized only from those stills.

## 8. Threats, and what I saw

**What I saw before or while freezing:**

- Before writing `PSI-25-solids-days.json` I read, from `BURNS.jsonl`, only investigation, burn id, date, role,
  material and partition. Its first rows also show the measurer's `quality` of three held fabric burns, which is
  not an outcome.
- Before writing the 2012 split I printed the condition cells (date, test, sample, material, igniter,
  restrictors, distance) of every 2013 row, test days included. I did not print the observation, fan or air cells
  of any test-day or validation row.
- While locating the correlation's constants I read summary-report lines that state published extinction
  speeds for thin PMMA. The physicist has not been shown them.

**Data threats:**

- The low-speed end of `quench_u` rests on an extrapolation below pot 1. The ordering of settings is reliable;
  the absolute speed below about 4 cm/s is not.
- The fan-to-speed relation varies by about +-40 % between calibration days.
- No PSI-26 calibration exists on disk.
- The PSI-25 oxygen offset is +-0.015 (section 3).
- The operators chose when to lower the flow, and the published literature on these burns is extensive.

## 9. Files and commands

- `tools/wsp/ext/bass1_cases.py`: `build`, `baselines`, `show-train`,
  `score --target quench|sustain --forecasts F.json --split train|validation` (validation prints aggregates
  only), `selftest`.
- `runs/dev/data_all/work/transfer_bass1/`: `cases_train.jsonl`, `cases_validation_inputs.jsonl`,
  `cases_validation_outcomes.jsonl`, `fan_curve.json`, `baselines_train.json`.
- Split files: `docs/wsp/atlas/splits/PSI-25-solids-days.json`, `docs/wsp/feedback/bass1_PSI-26-2012-days.json`.

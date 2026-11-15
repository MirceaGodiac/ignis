# SAME-R (PSI-101): what can be forecast, from what, and how it is scored

> Written by a delegated data agent (claude-opus-5-5) on 15 November 2026 for the seventh generation, from
> `runs/dev/gen7/BRIEF_DATA.md`. Nothing fitted except the constants of the named baselines on train days.
> No shared contract was changed. Paths under `runs/dev/` are local working folders that are not in git.
> Siblings: [same_feasibility.md](same_feasibility.md) (SAME, PSI-102, held), [daft_feasibility.md](daft_feasibility.md)
> (DAFT / DAFT-2, PSI-47, held).

## For the physicist

A forecast is asked for **the count mean diameter `d10` of the smoke (M1/M0, in m) at the age `net_aging_time`
(s) at which the row was measured**, from the inputs of `runs/dev/data_all/work/transfer_samer/cases_*.jsonl`,
field `inputs`, with these names and SI units: `material` (kapton, teflon, silicone, lampwick, pyrell; SAME also
has dibutyl_phthalate), `measurement` (`primary`: after the primary aging; `aged`: the smoke kept for a secondary
aging), `inlet_velocity` (m/s), `heater_resistance_ratio` (1, Rt/R0 set point), `heating_duration` (s),
`primary_aging_duration` (s), `primary_mixing_duration` (s), `secondary_aging_duration` (s, aged rows),
`secondary_volume_percent` (%, aged rows), `net_aging_time` (s, the horizon), `sample_temperature` (K, during
heating), `mass_loss` (kg, post-flight weighing, missing on 12 of 50 train rows), `particle_density` (kg/m3, NASA's
assumed value), `sts_detector_in_line`, `gmt_day`. In this form: **quantiles at 5, 25, 50, 75 and 95 % of `d10` in
m**; the score is the quantile CRPS of ln(d10). A second, optional target: **the probability that the ISS smoke
detector's forward-scatter channel is at full scale (|V| >= 6.0 V)** for the same row (Brier). An aged row of train
also carries `prior`, what was measured on its run's primary row (d10, d30, sigma_g, number and mass
concentration, net aging time); a law may use it, but must also forecast without it, because for validation
`prior` sits in the outcomes file and only the evaluator hands it to a frozen law.

## 1. What was read

- The brief, PROTOCOL 11.1 to 11.3, the atlas sections and card of PSI-101, the frozen groups
  `docs/wsp/atlas/splits/PSI-101.json`, the row of `ASTRA_COVERAGE_19.md`, the sibling Flame Design report and
  scripts, and the header of `crates/ignis-wsp/src/property_aerosol_v2.rs`.
- The final report `PSI-101_Reports_SAME-R_Final Report.pdf` (introduction, flight operation, the abstract of the
  results; not the appendix run sheets), and the methods part of the SAME final report (moment method,
  instruments, detector alarm criteria).
- The results workbook of both campaigns, `Analyzed Data/PSI-101_Analyzed Data_SAME, SAME-R_Results.xlsx`: its 43
  sheet names and headers; the cells of train-day rows. Validation-day rows were read by script and written to the
  outcomes file unseen; test-day rows were skipped once their run number was read (no cell stored or written).
- For format only: the run workbook of train run 61 (sheet names, headers) and the minimum and maximum of the
  ISS and STS detector channels in four train-run workbooks (61, 62, 66, 58).
- **What was seen that should not have been** (while printing the first data rows of sheets to learn their
  layout): the condition cells, including the net aging time, of SAME rows 18, 19 and 24 and of SAME-R
  validation rows 95, 96, 97 and 99. No smoke outcome of a validation or test day was displayed. The report's
  abstract shows per-material baseline-run diameters that pool all days, test days among them; I did not use it.

## 2. The data

One run heated a sample on a wire for about 60 s in a duct flow; the smoke was drawn into an aging chamber, held
for the primary aging (11 s, or 720 s on day 2010-08-24), pushed into the diagnostics duct and measured by a
condensation particle counter (P-Trak, M0), an ionization chamber (M1), DustTraks (photometric mass, about M3),
MPASS, the ISS (forward scatter, laser) and Shuttle (ionization) smoke detectors. On most runs part of the
smoke was kept and measured again after a secondary aging (480 to 1800 s): the "a" rows. NASA reduced each
measurement to one row of the results workbook (sheet "Combined Rslts Copy this sheet"; its later copies
"Moment Comparisons Jan 14 (2)" and "detection June 14" agree with it on every compared train cell; "Material Sort
02122013" and "SAME R Results" differ only in text placeholders and in mass-loss cells).

Per-run 1 Hz series (88 run workbooks, 44 housekeeping CSV files) are on disk; the case files do not use them.

## 3. Targets

| | Target 1 (primary) | Target 2 |
|---|---|---|
| Definition | count mean diameter `d10` = M1/M0 of the row | ISS detector scatter channel at full scale: \|V\| >= 6.0 V |
| Unit | m (sheet: um) | yes / no |
| Recorded in | column "M1/M0 (micro-m)--AMD" | column "ISS Detector Scatter (volts)" |
| Forecast may know | the inputs below; nothing measured on the smoke of that row | the same |
| Horizon | `net_aging_time` of the row (train: 21 to 62 s for primary rows, 745 to 751 s for the 720 s primary agings of 2010-08-24, 558 to 1923 s for aged rows) | the same row |
| Optional cutoff | aged rows: the primary row of the run (`prior`) | the same |
| Score | quantile CRPS of ln(d10 / 1 m), levels 5/25/50/75/95 % (trapezoid weights of `transfer_droplet.py`) | Brier |
| Per-day | mean over the cases of a day group; aggregate = mean over cases; day-equal mean reported beside | the same |

Why these. The size of the smoke is the investigation's own question, and it decides whether a light-scattering
detector sees the smoke (the report: Kapton and Teflon particles are so small that a scattering detector sensitive
enough to alarm on them would also alarm on dust). M1/M0 is one of the two diameters that NASA states are
independent of the lognormal assumption; it needs only the counter and the ionization chamber, both calibrated at
NIST, and it is present on 47 of 50 train rows. The diameter of average mass (M3/M0)^(1/3) is in the outcome files
as `d30_m`; it depends on the DustTrak calibration and the assumed density, and on train run 69 it is smaller
than d10 (an impossible pair). The detector target is what a crew would care about.

What target 2 is not: **full scale is not an alarm.** The ISS detector was specified to alarm at 1 % obscuration
per foot (UL smoke box, white light), "on the order of 0.5 mg/m3" (SAME final report, p. 2 and a model study it
quotes); the alarm level in volts of the channel recorded here is not in any file read. Full scale (about 6.25 V,
a clipped plateau) means a strong scattered signal. On train it is decided mostly by the material (Teflon never,
silicone, Pyrell and lamp wick always, Kapton 2 of 8), so it carries less information than target 1. The obscuration
channel of the results sheet is 0 on every row (not reduced); the 1 Hz series has it (beam drop, e.g. run 66).
Other candidates and why they were not chosen: mass concentration (2 to 350 mg/m3 on train, above both detectors'
nominal alarm levels on nearly every row, so not a discriminating safety quantity; it is in the outcome file);
the STS detector (in line on 11 of 50 train rows only; its volts have no stated conversion to mg/m3).

## 4. Inputs

| Name in the case files | Unit | Source (sheet column) | Known | Trust |
|---|---|---|---|---|
| material | - | Sample Type | before | high |
| measurement (primary / aged) | - | SAME # (an "a" suffix) | before | high |
| inlet_velocity | m/s | Inlet Velocity (cm/sec) | before | set point (8.1, 2, 1 or 0 cm/s); a "Fan problem" column exists |
| heater_resistance_ratio | 1 | Rt/R0 | before | set point; the report warns that equal Rt/R0 can give different sample temperatures |
| heating_duration | s | Duration (sec) | before | 60 s except validation run 102 (120 s) |
| primary_aging_duration, primary_mixing_duration | s | Primary Aging / Mixing Duration (sec) | before | set points |
| secondary_aging_duration, secondary_volume_percent | s, % | Secondary Aging Duration, Secondary Diagnostics Volume % | before | written on either the primary or the aged row; the script takes the larger; 0 on some aged rows |
| net_aging_time | s | Net Aging Duration | the horizon | the age at measurement; the catalogue holds it as an outcome field because it is known after the run; it reveals nothing about the smoke and is the time the forecast is for. "n/a" on train runs 52 and 57 |
| sample_temperature | K | Sample Temperature C (+273.15) | during heating | NASA derived it from the logged wire resistance; missing for reburn run 57 |
| mass_loss | kg | Mass Loss mg | after the test (post-flight weighing) | 0 in the sheet read as not weighed: 12 train rows (runs 48 to 54, 57, 67 to 72) |
| particle_density | kg/m3 | Particle density g/cc | before | NASA's assumption per material (0.9 to 2.2 g/cm3), not a measurement |
| sts_detector_in_line | - | Use STS Detector | before | high |
| gmt_day | - | GMT | before | day of year 2010 |

Not in the case files but on disk for train days: gas plume temperature, duct pressures and the sample resistance
at 1 Hz (run workbooks), the aging chamber, smoke tube and fluids drawings (Engineering Documents, PDF). The
aerosol kernel `property_aerosol_v2.rs` wants a chamber volume, a balanced throughflow, wall faces, a transfer
line and an independently supplied source yield (number rate and particle volume); the files give geometry only
in drawings, the flows in the per-run series, and no source yield (`mass_loss / heating_duration` is the vapour
mass rate, not a particle yield; the sheet's "Yield" column is computed from the DustTrak, an outcome).

**Shared quantity list.** `crates/ignis-core/src/contracts.rs`, `Quantity`: `inlet_velocity` is a
`FlowVelocity`, `sample_temperature` a `Temperature`, and `mass_loss / heating_duration` could be a `MassFlow`.
Nothing else is representable: no quantity holds an aerosol size moment or diameter (`LiquidDiameter` is the
diameter of a liquid droplet; using it for a smoke moment would be an alias), a number or mass concentration, a
geometric standard deviation, a detector voltage, a full-scale or alarm event, a resistance ratio or a duration.
Both targets therefore need a lane-local score (the `score` function of `tools/wsp/ext/samer_cases.py`); no
contract was changed.

## 5. Counts by partition (text files only)

Frozen groups: 10 train days, 3 validation days, 3 test days. Rows are runs plus their aged rows.

| | Train | Validation | Test |
|---|---|---|---|
| Catalogue rows (runs + aged) | 51 (35 + 16) | 24 (13 + 11) | 34 (19 + 15), catalogue only |
| Rows in the results workbook | 50 (34 + 16): the duplicate Teflon "57#2" has none | 24 (13 + 11) | not opened |
| Scored for d10 (horizon and d10 present) | 47 (31 + 16) | 23 (13 + 10) | - |
| Scored for ISS full scale | 48 (32 + 16) | 23 (13 + 10) | - |
| With sample temperature / mass loss | 49 / 38 | 24 / 22 | - |

Run 112 (Kapton, GMT 253) is in the workbook and the run files but not in NASA's experiment table: it has no
frozen group and is left out. For 15 runs the table date and the GMT day of the files differ by one; the group is
the catalogue's (table date), as frozen.

## 6. Baselines on train days

`python -I tools/wsp/ext/samer_cases.py baselines`; every number below comes through `score()`. "In-sample" uses
constants from all train days on the same train days; "leave-one-day-out" from the other train days.

Target 1, CRPS of ln(d10), all 47 train rows:

| Baseline | Aggregate | Day-equal |
|---|---|---|
| B0 constant (train quantiles of ln d10), in-sample | 0.3229 | 0.2807 |
| B0 constant, leave-one-day-out | 0.3437 | 0.2976 |
| B1 per-material quantiles, in-sample | 0.2454 | 0.2369 |
| B1 per-material quantiles, leave-one-day-out | 0.3627 | 0.3251 |

Aged rows only (16): B0 0.3543, B1 0.2195, **P persistence** (the primary row's d10, a point) 0.3849, **C
classical coagulation** 0.2121 (day-equal 0.2210). C is Smoluchowski coagulation of a monodisperse aerosol at
fixed mass, d = d_prior (1 + K N0 dt)^(1/3), K = K0 Cc(d30_prior), K0 = 3.0e-16 m3/s, Cunningham Cc with
lambda = 0.066 um and the constants 1.142, 0.558, 0.999 (Hinds, *Aerosol Technology*, 2nd ed., 1999, eqs.
3.18-3.20 and 12.11-12.13), N0 the primary row's count, dt the difference of the net aging times. Nothing of it is
fitted. Persistence is the worst: the smoke grows between the two measurements, and the classical law carries
about the right amount of growth. There is no classical correlation of smoke size against heater temperature,
flow and material at published constants that I know of; the primary rows have only B0 and B1.

Train constants of B0 (d10, um, quantiles 5/25/50/75/95): 0.106, 0.155, 0.244, 0.387, 0.472. B1 medians: Kapton
0.145, Teflon 0.155, lamp wick 0.230, Pyrell 0.301, silicone 0.338.

Leave-one-day-out B1 is worse than B0: day 2010-09-09 holds all the train low-flow and no-flow runs, whose smoke
is much larger, so a per-material constant without that day misses them. Flow is the input a law must carry.

Target 2, Brier, 48 train rows: incidence 0.708; I0 constant 0.2066 (leave-one-day-out 0.2165); I1 per-material
incidence 0.0312 (leave-one-day-out 0.0534); on the 16 aged rows the primary row's state (persistence) scores 0.

## 7. Images and video

Not needed for either target. The archive has 157 image files (1.50 GB: TEM grid images as zip archives, slides
and three TIFFs) and 16 video files (4.17 GB, the downlinked operations video and zips of it). The TEM images
would give a third, direct size measure and the morphology of about 70 grids; reading them needs particle sizing
on images, which is not built. The video would give nothing the logs do not.

## 8. What is missing or untrustworthy

- d10 rests on the counter's dilution correction and the ionization chamber's calibration; M1 is the largest
  contributor to the size uncertainty (SAME final report, eq. 12). No uncertainty per row is in the sheet.
- The P-Trak counts 20 nm to 1 um; large silicone and lamp-wick smoke at low flow may be undercounted, which
  inflates d10.
- Run 69 gives d30 < d10; run 114 has no P-Trak; runs 52 and 57 have no net aging time.
- Mass loss is missing on a quarter of the train rows and was weighed after the flight for the whole sample.
- The reduced value is one number per row; how NASA chose its averaging window is not stated in the sheet.
- The ISS full-scale target is a saturation, not an alarm, and mostly a function of the material.

## 9. Files

- `tools/wsp/ext/samer_cases.py` (build, score, baselines; also builds SAME).
- `runs/dev/data_all/work/transfer_samer/cases_train.jsonl` (50), `cases_validation_inputs.jsonl` (24),
  `cases_validation_outcomes.jsonl` (24; never printed).

# ACME CLD Flame (PSI-21): what can be forecast, from what, and how it is scored

> Written by a delegated data and scoring agent (claude-opus-5-5) on 15 November 2026 for the seventh
> generation (brief: `runs/dev/gen7/BRIEF_DATA.md`). Nothing fitted except the constants of the named baselines,
> on train days. Tool: `tools/wsp/ext/cld_cases.py`. Case files: `runs/dev/data_all/work/transfer_cld/`
> (local, not in git). Day groups: `docs/wsp/atlas/splits/PSI-21.json` (19 train, 6 validation, 7 test days).

## For the physicist (read this first)

A forecast is asked for two quantities on each ignited coflow flame (methane or ethylene fuel diluted with
nitrogen, in a 21 % O2 air coflow, microgravity, about 1 atm).

1. **`loss`: is the flame lost while the fuel still flows, and when.** Give `p_loss_by`, the probability that
   the flame has gone out by each of the times **1, 2, 3, 5, 7, 10, 15, 20, 30, 45, 60, 90, 120 s after flame
   onset** (non-decreasing), plus `p_loss_before_fuel_off`, the probability that it goes out at any time before
   the script turns the fuel off. Scored by the integrated Brier score over those times, and by the Brier score
   of the yes/no.
2. **`radiometer`: how much the flame radiates in its first constant-flow stage.** Give `quantiles` at 5, 25,
   50, 75 and 95 % of **log10 of the radiometer-sum excess in volts** (sum of the five radiometers, its level in
   the constant-flow stage minus its level before ignition). Scored by CRPS. It is asked only of flames that
   burn through at least 3 s of that stage; forecast it for every ignited flame.

Inputs, by their names in `inputs` of the case files: `fuel` (methane or ethylene); `x_fuel_initial`,
`x_fuel_final` (fuel mole fraction of the fuel stream, rest N2, 1); `v_fuel_ignition_m_s`, `v_fuel_final_m_s`
(mean exit velocity of the fuel stream, m/s); `v_coflow_ignition_m_s`, `v_coflow_final_m_s` (air coflow, m/s);
`schedule_from_ignition_command` (the commanded history: segments with `t0_s`, `t1_s` in s after the ignition
command, `v_fuel_m_s` and `v_coflow_m_s` and `x_fuel` at segment start and end, and `stage`);
`planned_fuel_off_from_ignition_command_s` (s); `fuel_tube_inner_diameter_m` (2.18e-3 m),
`coflow_inner_diameter_m` (25e-3 m), `x_o2_coflow` (0.21); `chamber_pressure_reading_pa` (Pa, an uncalibrated
transducer, about 1.02 to 1.13 atm); `zst_initial`, `zst_final` (stoichiometric mixture fraction, 1);
`round` (1, 2 or 3 test campaign; round 3 has the replacement coflow controller). At the forecast time (flame
onset) the record also gives `at_cutoff.onset_from_ignition_command_s` and `at_cutoff.planned_fuel_off_from_onset_s`
(s), which put the schedule on the forecast clock: t_since_onset = t_since_ignition_command − onset delay. The raw
script parameters are in `inputs.script` under their own names. Do not read the CLD Flame final report section
4.C, its Table 1, or the "Flames ignited" column of the experiment table: they summarise outcomes of every day,
including held days.

## 1. What is recorded, and the two targets

The archive has 359 test-point names (352 with a sensor file; 104 more names are auxiliary runs). There is no processed or analysed series for any
test. For each test point the text files are: the script parameters (`<T>.txt` in 2017-18, `as run<T>.txt` in
2021; written before the test, key:value), the 125 Hz channel file `<T>_data.csv` (three MFC readbacks, PMT1
at 310 nm for OH\*, PMT2 broadband, PMT3 at 430 nm for CH\*, five radiometers with cold junctions,
thermocouples including the burner, three pressure transducers), the MFC command log `<T>_mfcSetpoints.csv`
(volts, millisecond times), the stage-stamp file `<T>_TP.txt` (1 s stamps, "Extinction detected"), and camera
notes. The experiment table has one row per test day (fuel bottle, MFC sizes, concentration range, flames
ignited). The burner is a 2.18 mm fuel tube inside a 25 mm air annulus (final report: 2.1 mm design, 2.2 mm
"only"); the coflow is 21/79 O2/N2.

Candidates weighed:

| Candidate | Recorded for most tests in text? | Verdict |
|---|---|---|
| Flame held, lifted or blown off at the commanded velocities | Held vs lost: yes (PMT1 against the fuel command). Lifted vs attached: images only | **Target 1** as "lost while fuel flows"; lift-off is not separable from the channels |
| When the flame ends and why | Yes: flame end (PMT1), fuel-off command (setpoint log); cause = lost under flow or ended by the script | Folded into target 1 (time of loss; the scripted stop ends the case) |
| Radiometer level, a proxy of flame size and soot | Yes, every test with a data file; 0.14 to 12 V excess, ethylene flames 1 to 2 orders above dilute methane | **Target 2** |
| Ignition yes/no | Only 1 of 208 train test points with a complete script did not light; the attempts that failed mostly left no script file (9 train test points have a data file and no script; by the table's day counts, about 5 of them did not light) | Not forecastable from pre-test inputs |
| Flame length, lift-off height, soot volume fraction | Images only | Section 6 |

### Target 1: `loss`

- **Definition.** On the PMT1 (OH\*) channel: dark level = median over the 5.5 s before the ignition command
  (the PMTs are armed when the MFCs open). Onset = first time PMT1 exceeds dark + 0.05 V for 0.2 s, searched from
  1 s before to 12 s after the ignition command. Flame end = first time after onset that PMT1 is below dark + 0.02 V
  for 0.5 s. Fuel off = first command of the fuel MFC to zero after the ignition command (setpoint log; readback
  as fallback). **Loss** = flame end more than 0.5 s before fuel off. Loss time = flame end − onset (s).
  Ignition command = fuel setpoint on (priming start) + `primingAndPreIgnitionDurationSec`; it agrees with the TP
  "Ignition and retraction" stamp to 0.1 to 1.0 s (the stamp is truncated to the second).
- **Unit.** Event in time, s since onset; yes/no at fuel off.
- **Where recorded.** `<T>_data.csv` (PMT1), `<T>_mfcSetpoints.csv` (fuel command). Cross-check: the TP
  "Extinction detected" stamp. On train the stamp and the measurement agree on 161 of the 171 cases that have a TP file.
  The 10 disagreements are understood: 5 losses the on-board detection missed (dilute flames that faded during
  a dilution ramp, or tests with detection off), and 5 tests where the fuel was cut 1.3 s after onset with the flame
  still lit (the detection fired at the igniter retraction). The measurement calls these "not lost; censored at the
  cut".
- **What a forecast may know.** The pre-test inputs (section 2) and, at the cutoff (flame onset), that the flame lit
  and the onset delay. One cutoff, at onset. Optionally a later cutoff could add the first seconds of PMT1 and the
  radiometers. It is not built: the losses cluster at the igniter retraction (0.9 to 2.4 s) and in the dilution
  and coflow ramps (10 to 38 s), so a 3 s cutoff would remove a quarter of the events.
- **Horizons.** 1, 2, 3, 5, 7, 10, 15, 20, 30, 45, 60, 90, 120 s after onset.
- **Score.** Integrated Brier (trapezoid over the horizons divided by their span, as `transfer_acme.py`). A case
  is scored at a horizon only inside its window. The window ends at the planned fuel-off time from the script
  (an input). When the fuel was cut earlier with the flame burning, it ends at that cut. The window does not depend
  on whether the flame was lost. Per horizon, the Brier is the mean over the cases in their window; a horizon
  that no case reaches drops out of the integral. Secondary: Brier of the yes/no with `p_loss_before_fuel_off`.
- **Stage at loss on train** (20 losses in 207 cases, 8 of 19 days): dilution ramp 8, ignition stabilisation
  (igniter retracting) 5, ramp to final velocity 3, coflow matching ramp 4. Nine of the 11 train tests that dilute
  the fuel during the test lost the flame; 11 of 196 that do not dilute it did.

### Target 2: `radiometer`

- **Definition.** Sum of `Radiometer+60`, `+30`, `Center`, `-30`, `-60` (V). Baseline = median over 8 to 1 s
  before the ignition command. Level = median over the planned first constant-flow stage (schedule from the
  script), from 2 s after its start to its end, cut at the flame end or fuel off. At least 3 s must remain.
  Target = log10(max(level − baseline, 0.005 V)).
- **Unit.** log10(V). Train range: methane 0.14 to 1.8 V, ethylene 0.16 to 12.4 V of excess.
- **Where recorded.** `<T>_data.csv`. The radiometers have no gain setting, unlike the PMTs, whose gains change
  from test to test. So the radiometer, not the PMT, carries a comparable level.
- **What a forecast may know.** Pre-test inputs; cutoff at onset.
- **Score.** CRPS from the five quantiles: 2 × mean pinball loss over 5, 25, 50, 75, 95 %.
- **Caveats.** The sum also sees the burner and chamber warming (the excess keeps rising slowly over a stage).
  The radiometer sum is a radiant-power reading, not a calibrated flux (ACME-RPT-011 gives the
  characterisation; it is not on disk). The case requires the flame to burn through the window: 126 of the 155
  train cases cover the full stage, and 29 are cut by the flame end or the fuel-off.

## 2. Inputs

All are known before the test starts, except the two `at_cutoff` values, which are known at onset.

| Input (case-file name) | Unit | Source | Trust |
|---|---|---|---|
| `fuel` | – | experiment table, Fuel Bottle (rule FILL for the fill-series cells) | high; Test Type confirms 28 of 32 days |
| `bottle_fuel_fraction` | 1 | experiment table | high |
| `x_fuel_initial`, `x_fuel_final` | 1 (fuel mole fraction of the fuel+N2 stream) | script `concentrationInitial/Final` | medium: the data-interpretation document warns that parameter files "were not always saved" and "sometimes had missing entries"; 1 train file is incomplete and excluded |
| `v_fuel_ignition_m_s`, `v_fuel_final_m_s` | m/s, mean exit velocity of the fuel stream at the script's standard state (MFCs are calibrated at 0 °C, 760 torr) | script `velocityMix*` | medium; the script converts to N2-basis slpm |
| `v_coflow_ignition_m_s`, `v_coflow_final_m_s` | m/s | script `velocityAir*` | low in rounds 1 and 2: the final report says the original coflow MFC "could not provide a smooth ramped increase"; better in round 3 |
| `schedule_from_ignition_command` | s, m/s, 1 | derived from the script by the stage order of `CoflowTest.js` (CLD Flame Experimental Procedure) | planned fuel-off agrees with the commanded fuel-off within 1 s on 164 of 187 train tests that kept the flame, and within 3 s on 179. Exceptions: early cuts by the on-board detection, and 2 tests whose stage was longer than the file says (the operators can change parameters). Velocity-ramp tests run up to 3 s long (the ramp loop is slow); a matching coflow ramp adds 1.87 s (typed from train command logs) |
| `planned_fuel_off_from_ignition_command_s`, `planned_constant_flow_from_ignition_command_s` | s | as above, capped by `flowTimeoutSec` | as above |
| `script.*` (all raw parameters, e.g. `rampCount`, `rampSizeMPerSec`, `dilutionTimeSec`, `extinctionDetectionStopsTest`, igniter settings) | as named | script | as above |
| `zst_initial`, `zst_final` | 1 | computed: fuel/N2 stream against 21 % O2 air, complete combustion | exact given the inputs |
| `x_fuel_at_stoichiometric_surface_final` | 1 | computed (used by the classical baseline) | exact given the inputs |
| `roper_length_final_m` | m | Roper's circular-port correlation at the final condition (published constant) | the correlation was written for normal-gravity flames in still air; here it is a scale, not a claim |
| `fuel_tube_inner_diameter_m`, `coflow_inner_diameter_m`, `x_o2_coflow` | m, m, 1 | final report, data-interpretation document | 2.1 vs 2.2 mm in the sources |
| `chamber_pressure_reading_pa` | Pa | PTA3 over 8 to 1 s before ignition, psia = 36.0 × (V − 0.508) (two-point reading from CFI-G) | low: the ACME transducers "had not been calibrated in years"; the chamber calculator xlsx (rounds 2 and 3) is better and is not read here |
| `burner_tc_pre_ignition_v`, `radiometer_sum_pre_ignition_v` | V | data file before ignition | the burner may still be warm from the previous test of the day |
| `round` | – | day code (1: Nov 2017 to Feb 2018; 2: May to Sep 2018; 3: Feb to Mar 2021) | high |
| `pmt_gains`, `mfc_full_scale_slpm` | V, slpm | data-file header | high |
| `at_cutoff.onset_from_ignition_command_s` | s | measured at onset (0.22 to 1.06 s, 5 to 95 %) | as the measurement |
| `at_cutoff.planned_fuel_off_from_onset_s` | s | planned fuel-off minus onset delay | as the schedule |

Not in any text file of a test: the flow temperature (ambient chamber gas, not recorded at the burner exit), the
measured coflow velocity profile (hot-wire report `ACME CLD burner hot wire anemometry measurements - REVISED.pdf`,
a burner characterisation, not per test), the actual flow rates in engineering units (the readbacks are volts;
conversion in ACME-DOC-010, whose copy on disk does not open), and the chamber composition (nitrogen-filled;
the chamber calculator xlsx covers rounds 2 and 3 only).

## 3. Counts by partition

| | Test points named | Script + data file | Usable (script complete, ignition reference) | `loss` cases | `radiometer` cases | Days |
|---|---:|---:|---:|---:|---:|---:|
| Train (19 days) | 219 (catalogue), 218 with a data or script file | 216 (catalogue), 208 with a complete script | 208 | 207 | 155 | 19 (loss), 16 (radiometer) |
| Validation (6 days) | 73 (catalogue), 72 with a data or script file | 69 (catalogue) | 69 | written by script, not counted here | written by script, not counted here | 6 |
| Test (7 days) | 67 (catalogue only; no file opened) | 67 (catalogue) | – | – | – | 7 |

Validation outcomes are in `cases_validation_outcomes.jsonl` and were neither printed nor summarised.
`cases_validation_inputs.jsonl` holds every validation test point. A `usable` flag marks the 69 with a complete
script, a data file and an ignition reference; `at_cutoff` reveals whether the flame lit (the condition of a case).

## 4. Baselines on train days

Each baseline runs through `score()` (`python tools/wsp/ext/cld_cases.py baselines`;
`baselines_train.json` holds the per-day scores and the constants). The scores are on the same train cases
whose outcomes set the constants, so they are in-sample.

**`loss`** (207 cases, 19 days; aggregate over cases, then the mean over days of day scores):

| Baseline | Integrated Brier | Brier yes/no | Day-mean IBS |
|---|---:|---:|---:|
| Incidence: one Kaplan-Meier curve of loss time (censored at the window end) | 0.1506 | 0.0926 | 0.1290 |
| Incidence by fuel | 0.1417 | 0.0875 | 0.1239 |
| Incidence by "the script dilutes the fuel during the test" (`x_fuel_final < x_fuel_initial`) | 0.0718 | 0.0602 | 0.0907 |
| Persistence: incidence conditioned on whether the previous lit test of the same day lost its flame | 0.1020 | 0.0643 | 0.1082 |
| Classical: Coward and Jones lower flammability limit (5.0 % CH4, 2.7 % C2H4 in air) applied at the stoichiometric surface. Loss is forecast with P = 1 if the stoichiometric mixture of the fuel stream with air is below the limit, else 0 | 0.1909 | 0.0966 | 0.1545 |

The classical criterion never forecasts a loss on train: the leanest stoichiometric mixture is 4.1 % fuel, and
that test burns ethylene, whose limit is 2.7 %. It is therefore the "never" forecast, and it is worse than the
incidence curve. No published correlation predicts lift-off or blow-off of laminar coflow flames in
microgravity at 0.1 to 0.55 m/s. Blow-out correlations (Kalghatgi) are for turbulent jets in normal gravity.
The long horizons weigh heavily in the integrated score: at 45, 60 and 90 s only 36, 16 and 9 cases are still in
their window, and most of the late ones are velocity-ramp or dilution tests.

**`radiometer`** (155 cases, 16 days; CRPS in log10 V):

| Baseline | CRPS | Day-mean |
|---|---:|---:|
| Constant (train quantiles) | 0.2328 | 0.2452 |
| Constant by fuel | 0.2096 | 0.1959 |
| Persistence: previous test of the same day with a level, plus the train quantiles of the change between consecutive tests | 0.1324 | 0.1381 |
| Classical: log10 of Roper's flame length (L = 1330 s/m² · Q_F / ln(1 + 1/S), Roper 1977, published constant) plus one offset per fuel and its residual quantiles (fitted on train) | 0.1166 | 0.1323 |

## 5. What is missing or not to be trusted

- **Script files.** 9 train test points have a data file and no script, and one script is truncated
  (21062D1); all 10 are excluded. Radiometer and MFC auxiliary runs (`RADn`, `MFCA…`) are not test points and
  are not read. The ACME.log files
  hold the as-run parameters for some days and are the more reliable source. They are per date and span
  several test days, and some dates are held days (for example 2017-12-22 = 17356, 2018-02-08 = 18039 and
  2021-02-19 = 21050 are test days), so they are not read here.
- **The coflow** in rounds 1 and 2 followed its command poorly. The commanded value is the input, and the
  readback (`MFCA1StaticMixerFlow`, volts) is outcome-side.
- **Early cuts.** On 6 train tests the fuel was cut about 1.3 s after onset with the flame lit. These are
  scored only up to the cut.
- **Censoring is mostly by the script.** 187 of 207 cases end with the fuel commanded off and the flame lit. The loss is rare (20 of 207), clustered in round 1
  methane days and in dilution tests. The 2021 round, which was run to find extinction limits with the replacement
  coflow controller, has no loss on its two train days (21046, 21062) under this definition; its other two
  days, 21041 and 21050, are test days.
- **Lifted flames stay lit.** A flame that lifts and holds is "not lost" here. Only images can tell lifted from
  attached.
- **Sequential tests.** The pre-ignition burner temperature and radiometer level carry the previous test's heat.
- **What the agent saw beyond train.** This agent printed the experiment table in full, including the per-day
  "Flames ignited" column of validation and test days. It also read the final report and the data-interpretation
  document, which give dilution-limit ranges from all rounds. Per-test files of validation days were read only by
  the tool, which wrote their outcomes to the separate file, and no agent viewed them. No file of a test day was
  opened; their counts come from the file catalogue and from file names.

## 6. What needs images or video, and what it would cost

| Quantity | Medium | Size (catalogue, distinct files) |
|---|---|---|
| Lift-off height, flame length, lifted vs attached | operations video (NTSC, 30 fps, radiation-damaged), or DataCam / intensified-camera frames | ops video: 36.9 GB for 350 tests (median 0.09 GB per test); train 21.3 GB, validation 6.6 GB, test 8.2 GB |
| Soot volume fraction, soot temperature | DataCam colour frames (HOBJ, 12-bit; converted TIFF without dark-frame subtraction), with Abel inversion and the PI's calibration (green channel adjusted by 10 %) | colour camera 1,446 GB, of which 313 GB as downlinked; HiBMs 121 GB (red filter, judged of limited value by the project) |
| CH\* / OH\* flame shape | intensified camera | 391 GB |
| All images | | 1,959 GB for 324 tests (median 5.6 GB per test); by partition 1,225 GB train, 366 GB validation, 370 GB test |

Flame length and lift-off from the ops video are the cheap route: about 21 GB for the train days. Each frame
still needs a calibration of the burner's pixel scale and a threshold that is robust to the hot-pixel "star
field". Soot volume fraction needs the 1.4 TB colour archive and a pipeline the PI built (OMA2, HDR
bracketing), so it is not a per-test target within reach.

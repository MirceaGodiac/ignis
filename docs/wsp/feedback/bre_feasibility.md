# ACME BRE, the Burning Rate Emulator (PSI-20): what can be forecast, and how it is scored

> Written by a data and scoring agent (claude-opus-5-5) on 15 November 2026, for the brief
> `runs/dev/gen7/BRIEF_DATA.md`. Nothing here is fitted except the constants of the named baselines, on
> TRAIN days. Train days were read in full. Validation days: inputs only; their outcomes were measured by
> the script into a separate file and were never printed or summarised. Test days (20279, 20358, 21020):
> no file opened; counted from the file catalogue. Paths under `runs/dev/` are local and not in git.

## For the physicist

A forecast is asked for two quantities of each BRE test.

**T1: when the flame ends, and why.** The clock starts at the measured flame onset. The causes are
`extinction` (the flame went out while fuel still flowed) and `operator_termination` (the script's planned
stop cut the flow).
- Asked at cutoffs 1, 3 and 5 s after onset, with the flame still burning.
- Give the cumulative incidence of each cause at the horizons 6, 8, 10, 12, 15, 20, 25, 30, 40, 50, 60, 80,
  100, 120, 150, 180 and 210 s after onset. The two must sum to at most 1 at every horizon.
- Write one forecast for every (test, cutoff) of every test in the case file; the scorer keeps the cases
  that are at risk.

**T2: the heat flux to the burner surface.** This is `q_center_8_12s_kw_m2` in kW/m2: the centre gauge in
the porous burner surface, mean over 8 to 12 s after onset.
- Give the quantiles at 5, 25, 50, 75 and 95 %.
- It is defined only for flames still lit at 12 s, so forecast it conditional on that.

**Inputs**, as named in `inputs` of the case files, all known before ignition:
- `fuel_species` (ethylene, methane), `x_fuel_bottle` (mole fraction, balance N2), `burner_diameter_m`
  (0.025 or 0.050).
- Flows: `fuel_mass_flow_kg_s`, `fuel_mass_flux_kg_m2_s`, `x_fuel_burner_stream`,
  `burner_exit_velocity_stp_m_s`, `heat_release_rate_w`. Also the raw commands `fuel_flow_test_n2eq_slpm`,
  `fuel_flow_ignition_n2eq_slpm` and `nitrogen_test_flow_slpm`.
- Chamber: `pressure_pa`, `x_o2_ambient`, `x_n2_ambient`, `x_co2_ambient`, `x_he_ambient`,
  `x_fuel_residual_ambient`.
- Script: `planned_stop_s` (seconds from the start of the ignition step, which is about 0.8 s before onset),
  `constant_flow_time_s`, `stabilization_s`, `ignition_to_test_ramp_s`, `flow_timeout_s`,
  `extinction_detection_stops_test`, `burner_temperature_limit_c`.
- Igniter: `igniter_power_v`, `igniter_duration_ms`.
- `era` (2019, or 2020-21).

**Optional sensor prefix for T2** (cutoff 3 s): `prefix_3s.prefix_q_center_2_3s_kw_m2` and
`prefix_q_offset_2_3s_kw_m2`, the heat flux 2 to 3 s after onset, while the igniter is still in place.

**Scores.**
- T1: IPCW integrated Brier per cause (the scorer of `transfer_acme.py`).
- T2: CRPS from the five quantiles.

The file formats are in the docstring of `tools/wsp/ext/bre_cases.py`.

## 1. What BRE recorded

**Tests.** 208 test points over 18 test days (2019-02-08 to 2021-01-28).
- 17 days have raw text files. Day 19091 (train) has none.
- The file catalogue gives 116 tests on train days, 35 on validation days and 56 on test days. Test 20128H1
  is on a day outside the split.

**Burners and fuels.**
- 25 mm burner on 16 days; 50 mm on 20247 and 21028.
- Ethylene (100 % or 50 % in N2) on most days; methane on 20279 (test) and 20339 (train).

**Per-test text files.** These are the same kinds as Flame Design.

| File | Content |
|---|---|
| `<T>_data.csv` | 125 Hz analog channels |
| `<T>_mfcSetpoints.csv` | flow command log |
| `<T>_TP.txt` | stage stamps, "Extinction detected", "Reason" |
| `as run<T>.txt` | script parameters; named `<T>.txt` on 19056 and `<T>_BRE.txt` on 19039 |
| `<T>_Comments.txt` | image downlink notes |

**Two channel layouts**, the same 35 channels:
- **2019:** the BRE header names `BRECenter180`, `BREOffset180`, `BRECenter` and `BREOffset` (type K
  thermocouples on the burner and its gauges), `BREHeatFluxCenter` and `BREHeatFluxOffset`, and the
  radiometers `RadiometerBREOffset+60/+30/-10`.
- **2020-21:** the generic header names the same physical inputs `TCA`–`TCD` and `Ch20`/`Ch21`.
- The mapping is Table 2 of the ACME non-imaging data file description (2017-02-17, in
  `Science Documents_ACME BRE Data interpretation.zip`). AIN-20 is BRE_FLUX_CTR and AIN-21 is
  BRE_FLUX_OFF, which are columns 33 and 34 in both headers.
- The gauge scale given there is "1 mV = 1 Btu/ft2". I read it as Btu/(ft2 s), so 11.357 kW/m2 per mV.
  The time unit is my reading.

**Chamber Calculator workbooks.** There are two, one for 2019 and one for 2020-21. The EventData sheet has
one row per test point.
- Read: the start pressure (PT28, psia), the start mixture (O2, N2, CO2, He, residual fuel, in standard
  litres), the start chamber temperatures and the fuel type.
- Also on the sheet, and never read as inputs: "Ignite" (yes/no), the delivered MFC volumes (these grow
  with the burn duration), Pend, the end temperatures and the finish mixture.
- Rows of test days are dropped unread by the script, which looks only at the TPID cell.

**Script facts on train and validation days** (140 parameter files):
- Every file has `numberOfRampHolds` = 0, so there are no stepped ramps: a constant flow for
  `constantFlowTimeSec`, then a stop. The constant flow is 35–360 s (120 s on 58 files, 180 s on 44, 300 or
  360 s on 5).
- `extinctionDetectionStopsTest` is true in 131 files and absent in 9. A detected extinction cuts the flow
  and writes "Reason: EXTINCTION DETECTED"; that is the only Reason on train.
- The burner limit is `temperatureLimitC` = 250 °C (critical 300 °C) in every file.

## 2. Do the CFI-G tools work on BRE files as they are?

Checked on train tests: 19039A1, 19039C1, 19056A1, 20247A1, 20247D1, 20339A2 and 21028B1, then all train
tests.

**`flame_design.py`** runs once its folder and name prefix are overridden, but six things are wrong for BRE:

1. **Script file not found.** It looks only for `as run<T>.txt` and `<T>.txt`, so it misses `<T>_BRE.txt` on
   day 19039.
2. **2019 onset 3 s early.** `radiometer_channels()` drops every 2019 radiometer, because their names
   contain "Offset". Onset then falls back to the PMTs, measured against a dark level. Before arming, the
   PMTs sit at a 4.5 V rail. After the flame, one channel idles 0.09 V lower than before ignition. The
   result: onset 3 s before the stamp on 19039A1.
3. **Every extinction counts as a disagreement.** `tp_class` calls "Reason: EXTINCTION DETECTED" an abort,
   so every extinction is scored as a sensor/script disagreement.
4. **No burner limit.** It reads `burnerTemperatureLimitC`; BRE writes `temperatureLimitC`.
5. **Broken records unhandled.** It does not notice data files without a header line (8 train tests), or a
   record with a gap at ignition (21028G3).
6. **TP files of other tests used.** 19066A2 holds the TP of 19063G1, a validation test, and 21028B1 holds
   that of 21028A1.

**`acme_script2.py`:**
- `planned()` works unchanged (branch v1, `constantFlowTimeSec`). Over 33 scripted ends on train with a TP
  stamp, the measured flow cut came 0.15 to 1.1 s after D + S + F from the ignition stamp; the median is
  0.57 s and one outlier is 2.7 s.
- `for_test()` does not find `<T>_BRE.txt` on day 19039.
- `burner_limit()` returns `not_in_parameter_file`, because the key is `temperatureLimitC`.

**What was done instead.** `tools/wsp/ext/bre_cases.py` keeps the readers and helpers of `flame_design.py`
(`read_data`, `read_tp`, `read_setpoints`, `smooth`, `first_sustained`, `dark_level`), `acme_script2.planned`
and the scorer of `transfer_acme.py`, all unchanged. It re-implements the measurement for BRE:

- **Onset.**
  - Rule: the PMT sum rises 0.15 V above its armed pre-ignition median and holds 0.3 s.
  - Search window: from arming to 25 s after the "Ignition and retraction" stamp.
  - Train: onset 0.3–1.7 s after the stamp (median 0.80 s).
  - Cross-check: the radiometer-sum onset agrees within 0.6 s on every train test.
- **Flame end.** The PMT sum comes back within 0.05 V of its dark level and holds 0.5 s, as in
  `flame_design.py`.
- **Flow change.** The first commanded departure from the test flow after the constant stage began.
- **Cause.**
  - The script's own account is used when there is one: "Extinction detected" means extinction; "MFCs off"
    without it means operator termination.
  - The sensors decide otherwise. Extinction when the PMTs went dark more than 0.5 s before the cut;
    operator termination when the cut came first.
  - On train, the sensor label and the TP account agree on 81 of 84 tests. In the other 3 (19066D2,
    20261H1, 21028C1) the script detected extinction while the PMTs were still lit. They are labelled
    extinction, ended at the cut, and flagged.
- **End time.** Always from the sensors.
  - The 2019 script detector lags the PMT darkening by 5 to 35 s on 16 of 17 train extinctions (19039C1:
    dark at 26.7 s, detected at 61.4 s), so a TP stamp alone would misplace the end.
  - In 2020-21 the lag is under 2.3 s on 30 of 32, and 19 and 26 s on the other two.
- **Day 19094, which has no TP files.** The ignition reference is the fuel readback switching on plus the
  script's `fuelFlowIgnitionDeltaMs`, and the cause comes from the sensors.

## 3. Targets

### T1, primary: flame end and its cause (event in time, integrated Brier)

**Definition.** `end_s`, in s after the measured onset, with `cause` set to `extinction` or
`operator_termination`, as in section 2. A flame burning at the end of the file would be right-censored;
none on train.

**Why this target.**
- It is the fire-safety question: does a burning surface go out by itself, and when.
- It is recorded for almost every test with a data file.
- Self-extinction is common, and shows up in the train breakdown below by atmosphere and fuel.

**Train breakdown.**
- 95 lit tests: 58 extinctions and 37 operator terminations.
- Extinction times range from 3.2 to 158 s; the quartiles are 17, 28 and 65 s. 10 extinctions come before
  12 s.
- 1 atm, 21 % O2, 25 mm, pure ethylene: 8 of 8 went out.
- Methane: 10 of 10 went out.
- At 0.55–0.7 atm and O2 of 0.33 or more, about a third went out.

**Where it is recorded.**
- `_data.csv`: the PMT channels.
- `_mfcSetpoints.csv`: the flow cut.
- `_TP.txt`: the cause stamp.

No image is needed.

**What a forecast may know.** The pre-test inputs, the planned stop from the script, and that the flame
is still lit at the cutoff.

**Cutoffs and horizons.**
- Cutoffs: 1, 3 and 5 s.
- Horizons: those of CFI-G (6 to 120 s), extended with 150, 180 and 210 s. Most planned stops are
  56–202 s; five train scripts plan 300–364 s.

**Score.** IPCW Brier per cause, integrated over the horizons (`integrated_brier`), and the session-mean
over test days (PROTOCOL 12.4).

### T2, secondary: heat flux to the burner surface (number, CRPS)

**Definition.** `q_center_8_12s_kw_m2`.
- The centre Medtherm gauge in the burner surface (`BREHeatFluxCenter`, or `Ch20` in 2020-21).
- Its pre-ignition offset is removed, and it is converted at 1 mV = 11.357 kW/m2.
- Mean over 8 to 12 s after onset, for flames lit at 12 s.
- The 8–12 s window starts after the igniter has retracted (3–4 s after the ignition step) and its
  transient has passed.

**Why this target.**
- It is the quantity the BRE was built to measure: the heat fed back from the flame to a fuel surface,
  which sets a condensed fuel's burning rate.
- It is recorded in both eras.

**Behaviour.** The flux is not steady in microgravity. It falls as the flame grows: on 19039A1 it goes
from about 16 kW/m2 at 1 s to 5 kW/m2 at 13 s. A fixed window keeps it a single number.

**Train range.** 85 tests, 1.7 to 9.3 kW/m2, median 5.1. The offset gauge has a median of 4.1.

**What a forecast may know.** The inputs, plus optionally the flux 2–3 s after onset (persistence).

**Score.** CRPS from five quantiles (`scores::crps_quantiles`).

**Not chosen:**
- **Survival past a stated time** (Brier). It is a cut of T1 and is scored inside it; for example, the
  integrated Brier at the 60 s horizon is the Brier for "still burning at 60 s".
- **Flame standoff.** No sensor gives it; images only.
- **Burner thermal response.** This is the target of the earlier kernel, `property_bre_w0.rs`: "real
  apparatus thermal node" temperature, driven by an absorbed heat flux and a commanded fuel flow. The
  thermocouples `BRECenter`/`BREOffset` (or `TCC`/`TCD`) record it in mV type K. It would need the type-K
  conversion and the cold junction (`TCCJ`), and the kernel needs closures (absorbed fraction, apparatus
  heat capacity) that no file gives. It is left as a possible third target.

## 4. Inputs

Everything below is known before ignition.

| Input (case-file name) | Unit | Source | Trust |
|---|---|---|---|
| `fuel_species`, `x_fuel_bottle` | –, mol/mol | test catalogue (day table "Fuel Bottle"); cross-checked against the calculator fuel type on every row read | nominal; balance N2 |
| `burner_diameter_m` | m | test catalogue; data header `Burner:` 6 = 25 mm, 7 = 50 mm, agrees on every day read | good |
| `fuel_flow_test_n2eq_slpm`, `fuel_flow_ignition_n2eq_slpm`, `nitrogen_test_flow_slpm` | standard l/min, N2-equivalent | script file | commanded. The readback equals the command to about 1 % (`MFCA3FuelFlow`, V/5 × full scale) |
| `fuel_mass_flow_kg_s`, `fuel_mass_flux_kg_m2_s`, `x_fuel_burner_stream`, `burner_exit_velocity_stp_m_s` | kg/s, kg/(m2 s), –, m/s | derived; see the note below | train range 0.20–3.4 mg/s and 0.4–7.0 g/(m2 s) |
| `heat_release_rate_w` | W | ṁ × textbook LHV (ethylene 47.16, methane 50.03 MJ/kg) | upper bound: complete combustion |
| `pressure_pa` | Pa | calculator Pstart (PT28) | the data channel PTA3 before ignition reads 2.5–4.4 % higher (median 2.8 %) on all 94 train tests checked: an offset between gauges |
| `x_o2_ambient`, `x_n2_ambient`, `x_co2_ambient`, `x_he_ambient`, `x_fuel_residual_ambient` | mol/mol | calculator start mixture | bookkeeping, not a measurement: it assumes every ignited test burned all its fuel (Burn Fraction = 1). O2 drifts down through a day as products accumulate. The day GC files (`<day>_GC.zip`) could check it; not opened |
| `chamber_temperature_start_c` | °C | calculator TM1–4 start | often empty |
| `planned_stop_s` | s from the start of the ignition step | `acme_script2.planned` (v1) | within 0.15–1.1 s of the measured scripted cut (median +0.57 s) on train |
| `constant_flow_time_s`, `stabilization_s`, `ignition_to_test_ramp_s`, `flow_timeout_s`, `extinction_detection_stops_test`, `burner_temperature_limit_c`, `igniter_*` | s, °C, V, ms | script file | as written |
| `prefix_3s.prefix_q_*_2_3s_kw_m2` | kW/m2 | data file, 2–3 s after onset | a sensor prefix, allowed only at cutoff 3 s; the igniter is still in place |

**How the flows are derived.** Gas-correction factor of the bottle mixture
GCF = 1/(x/GCF_fuel + (1−x)/1). The factors are the Hastings values of the calculator's SourceTable:
ethylene 0.6062, methane 0.7787. The calculator confirms the rule on train: unburned ethylene equals the
MFC volume × 0.606 for pure ethylene, and × 0.377 for 50/50. The MFC standard is 0 °C and 760 Torr.

**Inputs that are not known**, so a forecast cannot use them:
- the flow of the ambient (quiescent; none);
- the burner surface temperature at the start (the thermocouples give it in the record only);
- per-test soot or radiation properties.

## 5. Counts by partition

Counts of tests. Test days are counted from the file catalogue only.

| | Train (11 days, 10 with files) | Validation (4 days) | Test (3 days) |
|---|---:|---:|---:|
| tests in the file catalogue | 116 | 35 | 56 |
| with data, script and TP files in the catalogue | 93 | 33 | 49 |
| with data and script files in the catalogue | 102 | 33 | 49 |
| inputs complete (ṁ, p, x_O2, d, planned stop) | 100 | 29 | not opened |
| T1 outcome measured (lit, endpoint resolved) | 95 | 30 | not opened |
| T1 inputs and outcome | 87 | 26 | not opened |
| T1 cases (test × cutoff at risk) | 279 | not summarised | – |
| T2 outcome (lit at 12 s, gauge present) | 85 | not summarised | – |
| T2 inputs and outcome | 78 | not summarised | – |

**Lost on train.**
- 12 tests have no data file.
- 8 data files start without their header line. They look like continuation pieces; their first line is
  a data row.
- 1 record has a gap at ignition (21028G3).
- 10 tests have no script file.
- 5 have no calculator row; these are the Z-numbered tests, which have no files at all.

**Partition boundary.** Validation outcomes are in `cases_validation_outcomes.jsonl`. No count derived
from them other than "T1 outcome measured" is given.

## 6. Baselines on train days

Constants were computed on train and scored in-sample on train. Command: `bre_cases.py baselines`. Output:
`runs/dev/data_all/work/transfer_bre/baselines_train.json`, which also has the per-day scores.

**T1** (279 cases; IPCW integrated Brier, lower is better):

| Baseline | Constants | Extinction IBS (session mean) | Operator termination IBS (session mean) |
|---|---|---:|---:|
| Incidence | for each cutoff, the fraction of train cases with each cause by each horizon | 0.2293 (0.2316) | 0.0766 (0.0712) |
| Planned stop | the extinction incidence above, cut at the script's stop. The remaining mass is an operator termination at `planned_stop_s` + 0.57 s − 0.80 s (two train medians: cut after D+S+F, onset after the stamp) | 0.2311 (0.2333) | 0.0688 (0.0635) |

- The script explains termination, as expected. The extinction part is the whole difficulty: the incidence
  curve scores 0.23.
- Persistence does not apply to T1: the cases carry no sensor prefix.
- **No classical correlation.** I know of none, at published constants, that gives a self-extinction time
  for a quiescent microgravity flame over a porous burner.

**T2** (85 tests; CRPS in kW/m2):

| Baseline | Constants | CRPS (session mean) |
|---|---|---:|
| Constant | train quantiles 2.34, 3.58, 5.06, 6.40, 8.16 kW/m2 | 0.985 (1.031) |
| Persistence | q(2–3 s) × the train quantiles of the ratio q(8–12 s)/q(2–3 s): 0.332, 0.553, 0.672, 0.804, 1.286 | 0.966 (0.918) |

- **No classical correlation.** The classical surface-flux relation is the stagnant-film / Spalding
  B-number balance, ṁ'' = (k/(c_p δ)) ln(1+B) with q'' = ṁ''·L. It needs a film thickness δ, and in a
  quiescent microgravity ambient δ grows with time and has no published constant. I did not use it.

## 7. What needs images or video

Neither target needs images; both come from the text files.

**What does need images or video:**
- flame standoff, height and shape;
- soot and luminosity structure;
- whether extinction starts at the edge or the centre;
- ignition tests that have no data file. Twelve train tests have only images, or nothing.

**Size in the file catalogue** (image archives and videos):
- train days 978.7 GB;
- validation 511.7 GB;
- test 542.2 GB;
- the held test 0.7 GB;
- about 2.03 TB in all. None of it is on disk.

## 8. Missing, untrustworthy, not verified

**Gauge and chamber values.**
- The heat-flux time unit (Btu/ft2 per second) is my reading of "1 mV = 1 Btu/ft2". Gauge calibration
  sheets and drift between 2019 and 2020-21 were not checked; the instrument reports
  (`ACME Instruments.zip`) were not opened.
- Chamber O2 and pressure are calculator bookkeeping. O2 depends on the assumed burn fraction of earlier
  tests, and the pressure is 2.8 % below the PTA3 channel.

**TP files.**
- The TP files of 19066A2 (holds 19063G1) and 21028B1 (holds 21028A1) were not used for those tests.
- The 19063G1 TP content was read by the script only to detect the name mismatch; nothing of it was
  printed.

**Labels and records.**
- Three train labels rest on the script's detector while the PMTs were still lit (section 2).
- Four records have a time gap after onset. Their ends agree with the TP stamps.

**The flow-timeout rule.** On every train script, `flowTimeoutSec` (80–380 s) is later than the planned
stop, and it was never seen to end a test. Planned stops reach 364 s (19070J1, a 360 s constant flow), beyond
the last horizon of 210 s. Such a case is scored on its extinction incidence up to 210 s.

**Not done.**
- Test-day files: not opened.
- The PDF day logs and test sheets of five days: not read.
- The GC files: not opened.

## Reproduce

```
python tools/wsp/ext/bre_cases.py selftest
python tools/wsp/ext/bre_cases.py build        # cases_train / cases_validation_inputs / cases_validation_outcomes / counts.json
python tools/wsp/ext/bre_cases.py baselines    # baselines_train.json
python tools/wsp/ext/bre_cases.py score --target t1 --forecasts F.jsonl            # train
python tools/wsp/ext/bre_cases.py score --target t1 --forecasts F.jsonl --partition validation --look <PROTOCOL section>
```

The `--partition validation` scoring prints aggregates only.

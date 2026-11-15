# E-FIELD Flames (PSI-22): what can be forecast, from what, and how it is scored

> Written by a data and scoring agent (claude-opus-5-5) on 15 November 2026, for the brief
> `runs/dev/gen7/BRIEF_DATA.md`. Tool: `tools/wsp/ext/efield_cases.py` (new). Case files and baseline numbers:
> `runs/dev/data_all/work/transfer_efield/` (local, not in git).
>
> **What was read.** Train days: everything a test has in text (`_data.csv`, `_VoltageSweep.txt`, `_TP.txt`,
> `_mfcSetpoints.csv`). Validation days: the same files were read **by the script only**. Their outcomes went
> to `cases_validation_outcomes.jsonl`, which was not opened. Nothing about them is printed here except file
> counts and condition columns. Test days: no file opened; counts come from the file catalogue.
> Also read: the experiment table's condition columns, and NASA's documents (conversions, procedures, final
> report).
>
> **One exposure, recorded so it can be weighed.** While I counted image and video bytes from the file
> catalogue, a validation-day video name was printed: `..._18298D1 - extinguished early.mp4`. That is an outcome
> hint for one validation test, seen by this agent and by no author. Nothing was changed because of it.

## For the physicist

**Two forecasts are asked of the law. Both start from the inputs in `cases_validation_inputs.jsonl`, whose
field names and units are those under `inputs`.**

**T1 (the number):** the ion current `ion_current_ua`, in µA, for every planned level in `inputs.levels`
(`level_id`, `v_cmd` in V, `kind` is init, step or ramp, `dwell_s`, `t_planned_s`). The sign convention is that
current in the direction the field drives it is positive, for both polarities. The forecast is conditional on
the flame still burning through that level.

**T2 (the event):** the probability that the flame has gone out under the field by each horizon of
2, 5, 10, 15, 20, 30, 45, 60, 90, 120 and 150 s after the field is switched on, up to `window_s`. Give it as
`t2.cif`, plus one probability `t2.p_end_in_window` that it goes out before the field programme ends.

**The inputs:**
- `fuel` (methane or ethylene), `x_fuel` (fuel mole fraction of the fuel stream, N2 balance).
- `u_fuel_m_s` and `u_coflow_m_s` (m/s, the script's velocities).
- `carbon_flow_atoms_s` (1/s, computed from those), `burner` (coflow or gas_jet_2.1mm), `polarity` (negative or
  positive mesh), `mesh_position` (35 mm), and `sweep` (the script's E-field parameters).
- Optionally, `pre_field`: what the sensors showed in the 2 s before the field came on. That block holds
  `flows_slpm`, `carbon_flow_atoms_s_measured`, `chamber_pressure_atm`, `pmt_sum_above_dark_v`,
  `radiometer_sum_v`, `ion_current_ua` and `seconds_since_flame_onset`.

**The form, one JSON line per test:** `{"test": ..., "t1": {"<level_id>": [q05, q25, q50, q75, q95]}, "t2":
{"cif": {"2": p, ...}, "p_end_in_window": p}}`. T1 is scored by CRPS from the five quantiles, in µA. T2 is
scored by the integrated Brier score over the observed horizons, and by the Brier score of `p_end_in_window`.
Two constraints on any law: `crates/ignis-wsp/src/property_jets_w0.rs` refuses any nonzero electric field
(`"electric response requires charge, force and energy closure"`), and `property_jets_v2.rs` reuses that law.

## 1. Targets

Three candidates were weighed against what is recorded for most tests in text:

- **(a) The ion current at a commanded voltage.** Chosen as T1. It is recorded on every test, at 125 Hz, in
  `IonCurrent`. The commanded voltage is recorded in `EfieldSetpointOut` and the applied one in `EfieldVoltage`.
- **(b) Whether and when the flame goes out under the field.** Chosen as T2. It comes from the PMT channels on
  every test, with the TP file's `Extinction detected` stamp as a cross-check.
- **(c) A radiometer change when the field is switched on.** Rejected. The field comes on 2.0 to 10.9 s after the
  flame lights (median 5.8 s), while the thermopiles are still settling from ignition. Over the 2 s either side
  of switch-on the change in the radiometer sum has an interquartile range of −0.14 to +0.31 V around +0.04 V,
  so the response is buried in the ignition transient.
  - A clear response exists later, at the start of the sweep (median −0.42 V over 4 s). It is a raw sum of
    thermopiles with different gains and no calibration was applied, so it was not made a target. It can be
    added from the same files if wanted.

**Why T1 and T2 matter for fire safety.** This investigation was not designed for fire safety; NASA says so
(card, `research_impacts`). Two links are indirect but real:
- the ion current is a flame-state signal, read the way a flame-ionisation detector reads one;
- whether a field holds a near-limit flame or blows it out is a flame-stability question that spacecraft
  hardware with high voltages could meet.

### T1: ion current at a commanded mesh voltage

- **Case:** one (test, planned level). The levels come from `<T>_VoltageSweep.txt`, the script parameter file,
  which is written before ignition. Only the first pass is used.
  - The initial voltage `efieldInitializeVolts`. It is held for `efieldInitializeTimeSec + constantFlowTimeSec`,
    which agrees within 0.7 s with all 121 train records.
  - Stepped sweeps (`efieldStepVoltage ≠ 0`): `start + k·step` for k = 0 … floor(span/|step|). The start is the
    minimum for positive steps and the maximum for negative steps, as ACME-DOC-010 3.1.8.3 says. Train records
    show the staircase stops at span/|step| dwells.
  - Ramps (`efieldRampRatePerSec ≠ 0`): the ramp's start, every multiple of 500 V inside it, and its end.
- **Outcome:** the median of `IonCurrent` × 1000/162 µA (ACME-DOC-010 Rev D 10.2.5), with the sign set by
  polarity. It is taken over the second half of the step's plateau on the commanded-voltage channel, or ±0.25 s
  around the ramp's crossing of the level. The commanded voltage is `EfieldSetpointOut` converted with the
  document's quadratic for each supply (10.2.36). The applied voltage (`EfieldVoltage`, 10.2.9) is stored beside
  it.
- **Scored only when** the flame was lit through the window and the raw reading stayed below 1.45 V; the
  channel range is ±1.5 V, about ±9.3 µA. Other statuses: `not_reached`, `flame_out` and `adc_clipped`.
- **May know:** everything in `inputs` (script, header, table), and optionally `pre_field`, the record up to
  the field-on cutoff. There is no later cutoff in the primary task. Persistence below shows how much a cutoff
  at the previous step would give.
- **Score:** CRPS in µA from the 5/25/50/75/95 % quantiles, using the evaluator's formula (as
  `transfer_droplet.py`). It is averaged over cases, and also reported as the mean of day means. Dense sweeps
  weigh more in the case mean: a test has 0 to 49 scored levels, median 12.

### T2: the flame ends under the field

- **Clock:** seconds since the field is switched on, meaning the first change of `EfieldSetpointOut` after the
  flame is lit.
- **Window:** `window_s`, the init hold plus the planned sweep length × `efieldCycles`. It is cut by:
  - the command returning to rest (`field_off_s`), which happened 1–2.5 s early on 12 ramp tests;
  - an unplanned flow command (censored, 1 train test);
  - the end of the record.
- **Event:** the PMT sum goes back to its dark level for 0.5 s, by the `flame_design.py` rule and its
  thresholds unchanged. A test whose flame is not lit for the second before switch-on has no case.
- **Cross-check against the TP stamp:** the TP file's `Extinction detected` stamp exists for 19 of 25 train
  events and comes 6.2 to 28.7 s after the PMT end (median 9.8 s). The stamp marks when the script reacted, so it
  is not used as the time.
  - The 6 events without a stamp are 18113D1, 18130A1, 18130A3, 18282A1, 18282B1 and 18312G1. Of these, 18130A1
    and 18130A3 were inspected on the traces, and on 18130A3 PMT, radiometers and ion current fall together.
  - 18130A1 lit only the ignition flash: the fuel flow was absent at switch-on (measured 0.03 of plan). It
    counts as an event at 0.1 s, and a forecast can see the missing flow in `pre_field.flows_slpm`.
- **Score:**
  - Brier score at each horizon of {2, 5, 10, 15, 20, 30, 45, 60, 90, 120, 150} s, pooled over the tests
    observed to that horizon, then integrated by the trapezoid over horizons with at least 5 tests. On train
    that is 2 to 90 s.
  - The Brier score of "ends before the field programme ends", on the 120 train tests not censored.

## 2. Inputs

| Input (case-file name) | Unit | Source | Known before the test | Trust |
|---|---|---|---|---|
| `fuel`, `fuel_bottle_fraction` | – | header `MFCA3 Gas` code (ACME-DOC-010 p. 37: 0 CH4, 2 CH4/N2 40/60, 4 C2H4, 5 C2H4/N2 30/70) | yes | high; agrees with the table's Fuel Bottle |
| `x_fuel` | mol/mol | script `concentrationFinal` (N2 bypass dilutes the bottle) | yes | high as a setpoint |
| `u_fuel_m_s`, `u_coflow_m_s` | m/s | script `velocityMixFinalMPerSec`, `velocityAirFinalMPerSec` | yes | setpoint. The measured fuel-stream flow / plan has median 1.08 and differs by more than 10 % on 34 of 121 train tests. Whether the script's velocity is at 0 °C or room temperature was not settled |
| `fuel_flow_mol_s`, `carbon_flow_atoms_s` | mol/s, 1/s | `u_fuel·A·x_fuel/0.022414`, A = 3.66e-6 m² (data interpretation doc p. 8; the gas-jet uses the same tube) | yes | carries the velocity question above |
| `burner` | – | header `Burner` (3 coflow, 5 gas jet 2.1 mm) | yes | high |
| `polarity` | – | header `EFieldPol` (1 negative, 2 positive) | yes | high; agrees with the table |
| `mesh_position` | – | header `EField` (1 = 35 mm). The final report gives 37 mm above the coflow tip; the gas-jet tip height was not found | yes | gap ±2 mm for coflow, unknown for the gas jet |
| `sweep`, `levels`, `window_s`, `cycles` | V, s | script `efield*` keys | yes | levels and window rules checked on train (above); `not_reached` levels are not scored |
| `constant_flow_s`, `flow_timeout_s`, `extinction_stops_test` | s | script | yes | `extinctionDetectionStopsTest` is true on every test |
| `pmt_gains`, `mfc_full_scale_slpm` | V, slpm | header | yes | high |
| `table` | – | experiment table (Test Type, Mesh Polarity, Fuel Bottle; `Flames Ignited` not read) | yes | day-level only |
| `pre_field.flows_slpm`, `carbon_flow_atoms_s_measured` | slpm, 1/s | MFC readbacks, 2 s before switch-on, slpm at 0 °C and 760 torr | at the cutoff | readback; no gas correction factor applied |
| `pre_field.chamber_pressure_atm` | atm | PTA3 (10.2.8) | at the cutoff | 1.035–1.057 atm on train |
| `pre_field.pmt_sum_above_dark_v`, `radiometer_sum_v`, `ion_current_ua`, `burner_tc_v`, `seconds_since_flame_onset` | V, µA, s | sensor channels | at the cutoff | raw volts. Thermopiles are still rising from ignition. TCBurner is meaningful on the gas jet only (data interpretation doc p. 7) |

Not available in text: soot, flame size, standoff from the mesh, and CH* or OH* images. The tests are mostly
at 1 atm in synthetic air. Oxygen is not in the table; the coflow and the chamber hold "21/79" air (final
report).

## 3. Counts by partition

| | Frozen days | Days with files | Test ids with sensor/script files | data + script | all four files | with T1/T2 outcome measured |
|---|---|---|---|---|---|---|
| Train | 12 | 11 | 129 | 123 | 121 | **121** (2,124 scored levels in 117 tests; 25 events) |
| Validation | 4 | 4 | 36 | 36 | 36 | 36 written (outcome statuses not read) |
| Test (catalogue only) | 4 | 4 | 41 | 40 | 40 | not opened |

Day-level details:
- **Train day 18274** (gas jet, 30 % ethylene, negative) has no `data.csv`, `VoltageSweep.txt` or `TP.txt` in
  the archive, only Comments and image archives.
- **Train day 18106:** 18106A1's sensor file exists only as `18106A1_data.zip` (1.7 MB, text) and was not
  unpacked.
- **Two tests with `data.csv.ERRORS` side files:** 18282M1 (train) and 18285K1 (validation).
- **Train exclusions:** 18319L1 shows no flame by the PMT rule, and 18319U1's script has no E-field keys. Four
  days in the file names (18103, 18281, 19312, 19319) are not in the split file; they are held and have no
  sensor files.

Train T1 level statuses: ok 2,124 (step 1,681, ramp 326, init 117), not reached 432, flame out 128, ADC-clipped
67 (on 9 tests, 7 of them on day 18319).

Train tests by stratum:

| Stratum | Tests | Days | T2 events |
|---|---|---|---|
| coflow, negative, with coflow | 15 | 2 | 0 |
| coflow, negative, no coflow | 3 | 1 | 0 |
| coflow, positive, with coflow | 42 | 6 | 0 |
| coflow, positive, no coflow | 26 | 4 | 13 |
| gas jet, negative | 13 | 1 | 6 (blowout at −6.8 to −7.9 kV) |
| gas jet, positive | 22 | 2 | 6 |

Strata the validation days add, from the table's condition columns only:
- **gas jet, negative, ethylene (18285):** no train data, because 18274 has no files;
- **gas jet, positive, 40 % CH4 (18298);**
- **coflow, negative, CH4 at 100 % and 70 % (18073, 18078).**

## 4. Baselines (train days only; their constants fitted on train; scored through `score()`)

| Baseline | T1 CRPS µA (2,124 cases) | T1 mean of day means | T2 integrated Brier | T2 Brier, ends in window (120 tests) |
|---|---|---|---|---|
| Constant: train-pool quantiles of the current (0.033, 0.320, 0.550, 1.359, 3.467 µA); T2 Kaplan–Meier incidence over all train tests (P(end in window) = 0.208) | **0.530** | 0.499 | **0.140** | **0.165** |
| Classical saturation law I_sat = e·η·Ṅ_C, η = 5×10⁻⁷ e⁻ per fuel carbon (hydrocarbon diffusion flames), log-normal band σ = 1.42 fitted on train | 0.747 | 0.735 | (as constant) | (as constant) |
| Same, η = 0.25 C per g-atom C = 2.6×10⁻⁶ (flame-ionisation detector value) | 4.469 | 4.503 | | |
| **Space-charge law up to saturation:** min(Thomson planar (9/8)ε₀μV²A/d³, I_sat with η = 5×10⁻⁷), σ = 0.93 fitted on train | **0.448** | 0.419 | | |
| Stratified incidence (polarity × coflow × burner, Kaplan–Meier per stratum) | (as constant) | | **0.054** | **0.118** |
| Persistence, a cutoff variant: the previous level's measured current plus train quantiles of the step change (2,007 cases with a previous level) | 0.068 | 0.069 | not applicable (a burning flame persists, p = 0) | |

- **Classical constants and sources.**
  - The proportionality of saturation current to fuel-carbon influx, J_s/Q_f, is the one named in NASA's SRD
    (2010, p. 30: "standard gas analysis by flame ionization detection relies on the fact that Js/Qf is nearly
    constant"). It is objective A.6 of the SCR and RDR. No numeric constant is given in any PSI-22 document I
    searched.
  - η = 5×10⁻⁷ is the order-of-magnitude yield for hydrocarbon diffusion flames quoted in the flame-ionisation
    literature; the Boothman thesis (Imperial College, 1974) is the source the SRD's citation of Boothman and
    Weinberg points to.
  - 0.25 C/g-atom C is the aliphatic FID figure.
  - I found both through a web search, read them only in search snippets, and did not verify them in the full
    texts. The exponent of 5×10⁻⁷ should be confirmed before the number is quoted as published.
  - Thomson's law: ε₀; μ = 1.4×10⁻⁴ (positive) and 1.9×10⁻⁴ (negative) m²/(V s), the standard small-ion
    mobilities of room-temperature air; d = the header mesh position (35 mm); A = the 8.4 cm mesh disc (final
    report p. 13).
- **Reading.**
  - The saturation law with η = 5×10⁻⁷ lands at the right order (observed/law: mean log ratio −0.70 over all
    levels, because low-voltage levels sit below saturation). Combined with the space-charge rise it is unbiased
    in log (mean log ratio +0.008) and beats the constant on 8 of 11 days. It is the reference a zero-shot law
    should beat.
  - Peak/saturation-law ratios vary strongly by stratum: 1.2–4.2 for coflow negative with coflow; 0.01–0.36
    for coflow positive without coflow; 0.25–8.1 for gas jet positive. The polarity and the coflow are
    first-order inputs.
  - Persistence is not comparable with the zero-shot task. It shows that the V–I curve is smooth step to step,
    so the information is in the conditions, not in noise.
- **T2.** The stratified incidence is in-sample on 6 strata; its score is optimistic. All 25 events are in
  three strata: positive field without coflow on either burner, and the negative gas jet. With coflow no flame
  went out (0 of 57).

Reproduce:
```
python -I tools/wsp/ext/efield_cases.py build
python -I tools/wsp/ext/efield_cases.py counts
python -I tools/wsp/ext/efield_cases.py baselines        # writes baselines_train.json and forecasts_train_constant.jsonl
python -I tools/wsp/ext/efield_cases.py score --forecasts runs/dev/data_all/work/transfer_efield/forecasts_train_constant.jsonl
```
The last command reproduces the constant row (0.530 / 0.140 / 0.165). `score` refuses any partition but train.
Validation is scored only in a look registered in PROTOCOL.md beforehand, by calling `score()` on
`cases_validation_outcomes.jsonl` joined to the inputs.

## 5. What needs images or video

Neither T1 nor T2 needs images. The outcomes are in the 125 Hz text channels. These need images or video:

| Quantity | Medium | Size (train + validation) |
|---|---|---|
| Flame shape, height, lift-off, standoff from the mesh, and the oscillation before blowout the final report describes | ACME DataCam colour archives (`<T>_ACME.zip`) | 141 + 39 files, 135 + 35 GB, about 0.9–1 GB per test |
| | CFI intensified CH* camera (`<T>_Intensifier*.zip`) | 179 + 58 files, 256 + 103 GB |
| Soot by extinction | HiBMS (`<T>A_HiBMS.zip`), largely blocked by the mesh mount (engineering doc p. 18) | 94 + 28 files, 133 + 45 GB |
| A coarse visual check of flame presence and size | analog video downlink (`vlc-record-…mp4`) | 138 + 35 files, 8.0 + 1.7 GB, about 50 MB per test |

The archive's images and video total 677 files, 923 GB and 436 files, 25 GB. A flame-shape target, for example
flame height against voltage, would cost about 170 GB of DataCam archives for train and validation. The video is
the only affordable visual source, at about 10 GB, but it is a compressed analog downlink and unlikely to carry
a calibrated shape.

## 6. Missing or untrustworthy

- **Planned against measured fuel flow** (median +8 %, a third of tests beyond 10 %). Unresolved: the reference
  temperature of the script's velocity, and whether the MFC readback needs a gas correction factor. Both
  `carbon_flow_atoms_s` (plan) and `carbon_flow_atoms_s_measured` are given.
- **Electrode gap for the gas-jet burner** is not documented in what I read. The header gives one code
  (35 mm) for both burners.
- **ADC range:** levels above about 9 µA are excluded. Strong ethylene flames at high positive field
  (day 18319) lose up to 10 levels each, so T1 under-represents the most ionised flames.
- **Leakage:** the post-flame duplicate sweep NASA runs reads a median 0.006–0.008 µA per test, which is
  negligible. The spikes at voltage steps are larger.
- **T2 event time** rests on the PMT rule from Flame Design, unchanged. A flame that the field pushes out of the
  PMTs' view, without going out, would be read as an event. The six events without a TP stamp were not all
  inspected.
- **Initial voltage equal to the first step level** (for example −200 V on some validation tests) merges the
  two plateaus; the first step is then `not_reached` and not scored.
- **The constant-stage rule for the sweep start** (init + constant-flow time) is read from train records and
  holds within 0.7 s there. It is a rule of this tool, not a documented script behaviour.
- **No flame-shape, soot or temperature field.** No oxygen or pressure varied: every test is 1 atm air.
- **Train covers only 11 of 12 frozen days, and 3 of its 4 gas-jet days.** One validation stratum (negative gas
  jet, ethylene) has no train counterpart.

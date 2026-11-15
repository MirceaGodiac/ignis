# E-FIELD (PSI-22): planned sweep maximum, burner, coflow and polarity fields

> Answer to the coordinator's question of 15 November about `docs/wsp/research/phys_efield_study.md`.
> Written by the data agent (claude-opus-5-5).
> - **Builder.** `tools/wsp/ext/efield_cases.py` gained `planned_v_max()` and `held_v_max()`, and was re-run on
>   train and validation.
> - **Read.** Train and validation files only. No test-day file was opened. The derivations below read only
>   `<T>_VoltageSweep.txt`, plus the data file's first header line or the experiment table. They work on test
>   days unchanged, but were not run there.

## 1. `sweep_v_max` is the planned maximum, not the level reached

- **Where it is made.** The case builder writes no field of that name; `tools/wsp/ext/law_inputs.py` builds it
  (line 185) as `max |v_cmd| over inputs.levels`.
- **What it is made from.** `inputs.levels` is written by `efield_cases.plan(script, polarity)`, which reads
  the script parameter file `<T>_VoltageSweep.txt` and nothing else.
- **What that file is.** It is written before ignition. Its `efield*` keys are the planned programme.
- **The level list is never filtered by outcome.** Levels the flame did not live to see stay in
  `inputs.levels`; only `outcomes.t1[*].status` marks them `flame_out` or `not_reached`.
- **So `sweep_v_max` carries no outcome**, on train, validation and test alike.

### New condition fields

All three are in `cases_train.jsonl` and `cases_validation_inputs.jsonl`, and all are read from the script only.

| Field | Definition |
|---|---|
| `sweep_v_max_planned` | max(\|efieldInitializeVolts\|, \|start\|, \|end\|). A stepped sweep starts at the minimum bound when the step is positive and at the maximum when it is negative, and ends at start + floor(span/\|step\|)·step. A ramp's end is start + rate·efieldRampDurationSecs, clipped to the bounds. Cycles only go back and forth, so they add nothing. Computed directly from the keys, independently of `plan()`. |
| `sweep_v_bound_script` | max(\|efieldMinVoltage\|, \|efieldMaxVoltage\|): the bounds as typed. |
| `sweep_v_max_held` | The largest level held for at least half a dwell under the staircase's duration rule: k = ceil(span/\|step\| − 0.5) − 1. Ramps as planned. |

### Counts

| | Train | Validation |
|---|---|---|
| Tests with a script | 123 | 36 |
| `sweep_v_max_planned` missing | 1 (18319U1: the script has no `efield*` or velocity keys) | 0 |
| `sweep_v_max` (law_inputs) ≠ `sweep_v_max_planned` | **0** | **0** |
| `sweep_v_bound_script` ≠ planned | 26 (all step sweeps; e.g. 500→10000 by 350 ends at 9950) | 12 |
| `sweep_v_max_held` ≠ planned | 67 (by one step; 2000–2800 V on the square-wave programmes) | 30 |
| For contrast: the highest level reached with the flame burning ≠ planned | 66 of 121 (23 of the 25 flame-outs) | not computed |

### Check against what was commanded (train)

The quantity checked is the largest |`EfieldSetpointOut`| commanded while fuel flowed, converted to volts.
- **Tests that kept the flame to the end of the programme (95 tests).**
  - It equals `sweep_v_max_planned` within 60 V on all 45 step sweeps and on 49 of 50 ramps.
  - The exception, 18137J1, commanded 10 kV against a planned 3 kV. That needs a look before this test is used.
  - On step sweeps the top level is often commanded for less than half a dwell. That is why the builder marks
    it `not_reached` for T1, and why `sweep_v_max_held` is lower.
- **Tests whose flame went out.** The commanded maximum is below the plan on 17 of 26, because the script
  stopped the test. That is the outcome-carrying quantity, and no case field holds it.

### For the law

- **Which field.** `sweep_v_max` (= `sweep_v_max_planned`) is admissible as it stands. If the law means the
  highest voltage actually held, `sweep_v_max_held` is the pre-test alternative.
- **The split in the study is unaffected.** On the gas-jet negative methane tests, the 6 flame-outs have planned
  maxima of 7–10 kV. The 7 tests that stayed lit plan 5.5 kV or less: five ramps to 5.0–5.5 kV, and the step
  sweeps 18282G1 (5.0 kV; held 3.0 kV) and 18282M1 (4.5 kV; held 2.5 kV).

## 2. Burner, coflow and polarity

| Quantity | Case field | Source | Train | Validation | Before the test? |
|---|---|---|---|---|---|
| Burner | `inputs.burner` (`coflow`, `gas_jet_2.1mm`) | first header line of `<T>_data.csv`, code `Burner` (3 coflow, 5 gas jet 2.1 mm; ACME-DOC-010 p. 127). The header is written by SystemDefineSetup at script start, before ignition | 123/123 (86 coflow, 37 gas jet) | 36/36 (17, 19) | yes |
| | `inputs.table.test_type` | experiment table "First round … coflow burner" / "Second round … gas-jet burner" (the test matrix, day level) | agrees on 123/123 | agrees on 36/36 | yes; gives the burner for test days without opening a file |
| Coflow on | `inputs.u_coflow_m_s` (on when > 0) | script `velocityAirFinalMPerSec` | 122/123 (missing 18319U1): 57 on (0.031 m/s on 55, 0.19 and 0.31 once each), 65 off | 36/36: 8 on (0.031), 28 off | yes |
| | check: `inputs.pre_field.flows_slpm[0]` | measured MFCA1 in the 2 s before the field. Not needed, and only known at the cutoff | on ↔ 0.73–0.79 slpm, off ↔ ~0: agrees 123/123 | agrees 36/36 | at the cutoff |
| Polarity | `inputs.polarity` | header code `EFieldPol` (1 negative, 2 positive) | 123/123 (92 positive, 31 negative) | 36/36 (9, 27) | yes |
| | `inputs.table.mesh_polarity` | experiment table, day level | agrees 123/123 | agrees 36/36 | yes |

Burner and coflow together:
- gas-jet tests never have coflow;
- the "coflow burner in still gas" group is `burner == coflow and u_coflow_m_s == 0`: 29 train tests and 9
  validation tests;
- the burner with coflow on covers 57 train and 8 validation tests, which is the study's 65;
- `strong_coflow` (≥ 0.1 m/s) occurs on 2 train tests only (18137C2 at 0.31, 18137M1 at 0.19) and on no
  validation test.

All three quantities are known before ignition, from the script and the test matrix. The header's
burner and polarity codes are setup parameters written when the script starts. On test days, the burner and
polarity come from the experiment table and the coflow from the script. Nothing from a sensor is needed.

# s-Flame (PSI-23): inputs, endpoints and case files for the burner-flame target

> Written by a data and scoring agent (claude-opus-5-5) on 15 November 2026. It replaces a generic
> `sflame_feasibility.md`. The feasibility note `sflame_transfer_feasibility.md` stands; this file does not redo it.
> **No model was run on a validation day.** Validation-day endpoints went by script into
> `cases_validation_outcomes.jsonl`; that file was not opened, printed or summarised. Nothing of the three test
> days (20142, 20170, 20176) or of the unlisted day 19245 was opened.
> Tool: `tools/wsp/ext/sflame_cases.py` (new). Outputs: `runs/dev/data_all/work/transfer_sflame/` (local).
> Binary: `target-merged/release/ignis.exe`, sha256 `b03a35c8…48ffb`, the binary of the F2 chain check in
> `transfer_acme/fd_f2/cfig/chain_check.json` (reproduced).

## For the physicist

- **The quantity.** The forecast is for when the flame of a constant-flow s-Flame test ends, and why: extinction, or
  operator termination (the scripted flow stop, or the 250 °C burner-limit stop). The clock is seconds since the
  flame onset that `flame_design.py` measures. This is the target of CFI-G, Flame Design and the adopted
  `acme_normal.wsp12` F2.
- **When it is asked.** At cutoffs 1, 3 and 5 s, for each test still burning at the cutoff.
- **The form.** Two cumulative incidence curves, P(extinction by t) and P(operator termination by t), at
  t = 6, 8, 10, 12, 15, 20, 25, 30, 40, 50, 60, 80, 100, 120 s. Each curve is non-decreasing, and their sum is at
  most 1.
- **The score.** The IPCW integrated Brier score over 6–120 s, for each cause and for their sum
  (`sflame_cases.score`).
- **The inputs.** All are known before ignition. Names and units are as they appear in `cases_*.jsonl`:

| Field | Unit | Meaning |
|---|---|---|
| `inputs.pressure` | Pa | Chamber pressure before the flow is on |
| `inputs.injected_mass_flow` | kg/s | Mass flow of the fuel species through the burner |
| `inputs.x_fuel_role` | 1 | Fuel mole fraction of the burner stream |
| `inputs.x_o2_role` | 1 | Ambient O2 mole fraction, nominal; `null` on 49 tests |
| `inputs.tad` | K | Stoichiometric adiabatic flame temperature |
| `inputs.zst` | 1 | Stoichiometric mixture fraction |
| `inputs.fuel_species` | text | `ethylene`, `methane`, `hydrogen` or `hydrogen+methane` |
| `inputs.programmed_stop` | s | Planned flow stop on the forecast clock |

- **Beside them:**
  - `flows.burner_stream`: mole fractions of every species in the burner stream.
  - `diluent` (of the burner stream) and `ambient_diluent`: `n2` or `he`.
  - `burner_limit_c`, in °C: 250 on every script that gives it.
  - `burner_t_start_c`, in °C.
  - `inputs_by_o2_candidate`: `tad` and `zst` at each O2 value the table names for the day.
  - `x_o2.pressure_block`.

## 1. The adiabatic flame temperature

**How the tabulated values were made.** They come from the HP equilibrium of the stoichiometric mixture of the fuel
stream and the oxidizer stream, both at 298.15 K.
- **Stoichiometric** means a·Σ x_i ν_i = X_O2,ox moles of fuel stream per mole of oxidizer stream, where ν_i is the
  number of moles of O2 needed per mole of fuel species i.
- **The two tables differ only in pressure and composition:**
  - **CFI-G:** every row is computed at 1 atm, whatever the test pressure, from the initial composition.
  - **Flame Design:** each row is computed at the test's mean pressure, from the mean of the initial and final
    composition.
- **My implementation:** `tad_eq` in `sflame_cases.py`. It uses Cantera 3.2.0, installed here with
  `pip install cantera`, with NASA 7-coefficient species from `nasa_gas.yaml`. The species are the fuels, O2, N2,
  He, CO2, H2O, CO, H2, OH, H, O, NO, NO2, N2O, HO2 and H2O2.

**Agreement with the table** (my value minus the table, in K):

| Rows | n | Initial composition at the test pressure: median / max abs | Initial composition at 1 atm: median / max abs |
|---|---|---|---|
| CFI-G train, normal, ethane | 39 | +0.9 / 42 | **+0.8 / 4.1** |
| CFI-G train, normal, propane | 15 | −0.9 / 23 | **−1.5 / 3.0** |
| CFI-G train, normal, butane | 27 | −0.6 / 51 | **+2.6 / 3.7** |
| CFI-G train, inverse, ethane / propane / butane | 18 / 28 / 27 | −13 to +37 / 69 | **−1.6 to +1.3 / 4.1** |
| Flame Design train days, normal, ethylene | 76 | **+3.9 / 8.9** | +2.7 / 17 |
| Flame Design train days, inverse, ethylene | 64 | **+2.4 / 13.6** | +2.6 / 105 |

- **The table's own convention reproduces Flame Design almost exactly.** With the mean of the initial and final
  composition at the mean pressure, the difference is +2.0 / 3.5 K (normal) and +2.1 / 2.9 K (inverse). That check
  reads the after-test columns of train days only.
- **The method used for s-Flame:** the initial composition, at the measured initial pressure.
  - **Against Flame Design**, it carries a bias of +2 to +4 K and stays within 14 K. No final composition exists
    for s-Flame.
  - **Pressure convention.** On s-Flame, with pressures of 1.00–1.05 atm, the value at the test pressure and the
    value at 1 atm (the CFI-G convention) differ by at most 1.6 K. The pressure convention therefore does not
    matter here.
- **The larger error is the O2 input, not the calculation** (section 2).
  - The tables used the chamber calculator's per-test O2, which falls through a day.
  - s-Flame has only the nominal value, about 21 % or about 30 %.
  - Tad changes by **40–47 K per 0.01 of O2** (19254A1 and 19304A1).
  - On Flame Design's 21 % days the per-test O2 fell by up to 0.006–0.010 within a day.
  - **So the s-Flame Tad is biased upward by up to about 40 K late in a day.** The bias depends on the test and
    is not corrected.

**s-Flame values, constant-flow tests:**

| Day (partition) | Fuel, dilution | Tad, K |
|---|---|---|
| 19254, 20029 (train) | ethylene, N2 | 1829–2157 |
| 19304 (train) | methane, N2 | 1966–2101 |
| 19352, 20024 (train) | H2, N2 | 1188–2220 |
| 20006, 20013 (train) | H2/CH4, N2 | 1615–2014 |
| 20132, 20136 (train) | H2/CH4, He | 2419–2426 |
| 20155, 20190 (train) | H2, He | 2360 at 21 %, 2527 at 30 % |
| 19256 (validation) | ethylene, N2 | 1984–2086 |
| 19308 (validation) | methane, N2, ~30 % | 1846–2396 |
| 20160 (validation) | methane, He | 2441 at 21 %, 2629 at 30 % |
| 20164 (validation) | ethylene, He, ~30 % | 2657–2658 |

## 2. The other inputs (train and validation days, 229 tests)

**Pressure.** The reading is PTA3, the chamber transducer, as a median over the first 0.5 s of the record. The record
starts about 24 s before ignition, with the flow off. One correction to the feasibility note: PTA2 is the burner
line.
- **The documented conversion** (ACME data-file description, AIN-6) is psia = (V − 0.5) × 36.26.
- **It reads high against the tables.** On Flame Design train days, the conversion is 0.027 atm above the table's
  pinit, with an sd of 0.004 over 187 tests. The offset is stable from 2019 to 2022 (0.023–0.029 near 1 atm). The
  relation is linear to an sd of 0.0016 atm over 0.42–3.0 atm.
  - It follows the line through the chamber calculator's start pressures already recorded in
    `docs/wsp/measure/DISCREPANCIES.md` (psia = −18.30 + 36.03 V).
  - The NASA data-interpretation document says the ACME transducers were not recalibrated, and that the chamber
    calculator's pressures are the reliable ones.
- **`pressure` therefore uses that line**, which puts it on the tables' scale. The documented value is kept in
  `pressure.pa_doc_formula`, about 0.026 atm higher, and moves c_logp by only 0.026.
- **Range:** 1.000–1.048 atm.
- **No chamber calculator covers s-Flame.** The archive has calculators for 2019-04 to 2019-08 and from 2020-07-22
  on, but not the "2018-03-08 to 2020-07-08" file the NASA document names.

**Flows.** The MFCs are Hastings HFC-D-302, calibrated for N2 at 0 °C and 760 torr. The script values are in
N2-equivalent slpm.
- **Conversion:** actual slpm = N2-slpm × GCF of the mixture, with the mixture rule 1/Σ(x/GCF).
- **Component GCFs** are those of `ACMEFlowCalculator.xlsx` (PSI-22): C2H4 0.6062, CH4 0.7787, H2 1.0038, He 1.4005,
  N2 1.
- **Bottle factors:**

| Bottle | GCF |
|---|---|
| 50 % C2H4 / N2 | 0.7548 |
| 45 % CH4 / N2 | 0.8866 |
| H2 | 1.0038 |
| 44.4 / 66.7 % H2 in CH4 | 0.8648 / 0.9157 |
| 25/20/55 and 20/25/55 H2/CH4/He | 1.1128 / 1.0953 |
| 45 % H2 / He | 1.1890 |
| 45 % CH4 / He | 1.0303 |
| 20 % C2H4 / He | 1.1097 |

- **One workbook error was not followed.** The workbook gives 99 ("n/a") as the helium factor of its H2/CH4/He
  bottles, which yields 1.90–2.01; 1.4005 is used here instead.
- **The diluent MFC** (MFCA2) is N2 on every day (header code 0).
- **The feasibility note's factors differ.** It used 0.50 for C2H4 and 1.45 for He, so its mass flows were about
  20 % off for ethylene.
- **Ethylene with N2 now reads 0.32–2.67 mg/s**, at x_fuel 0.10–0.23.

**Bottle composition** is taken from the header's MFCA3 gas code (ACME-DOC-010 Rev D; same table in the workbook).
- **It is checked against the experiment table** after reading "N3", "N4" … as N2.
- **Two days disagree.** Both are settled by Table 1 of the s-Flame data-interpretation document:

| Day | Header code | Experiment table | Table 1 | Used |
|---|---|---|---|---|
| 20190 | 14: 45 % H2 / He | 35 % H2 | 45 % H2 | header |
| 20136 | 12: 25/20/55 | 20 % H2, 25 % CH4 | same as the experiment table | table (the header keeps 20132's code) |

**Fuel species.** Ethylene and methane are as the forwards name them; `hydrogen` is pure H2. The H2/CH4 bottles carry
`hydrogen+methane`, which no release knows.

**O2: the 49 tests on 20155, 20160 and 20190 cannot be recovered per test from text files.**
- **No per-test source exists:**
  - No chamber calculator covers the period.
  - No sensor channel carries O2.
  - The session logs record only the CIR manifold states.
  - The daily test-point logs hold downlink data volumes only. I read their header rows; they hold no O2.
- **The gas chromatograph does not help.** Its files (`moduleDataACD_*.asc`) are raw chromatograms that NASA itself
  has not reduced (data-interpretation document, 4.A.3).
  - **20155 and 20160** have no GC file of their own.
  - **20190** has two, taken after its last block. They could at best give the final atmosphere.
- **What the pressure does show: when the chamber was vented and refilled.** The pre-ignition PTA3 rises
  monotonically with vitiation, then falls after a long gap.

| Day | Blocks | Tests in each block (refill gap) | O2 recovered? |
|---|---|---|---|
| 20155 (train) | 2 | A1–C2, then G1–H1 (57 min gap) | no |
| 20190 (train) | 3 | A1–F1, G1–M3 (73 min), N1–W1 (82 min) | no; three blocks for two nominal values |
| 20160 (validation) | 1 | no pressure fall; a 174 min gap before H7 | no |

- **On 20155 and 20190 the same flow series repeats in each block.**
- **The order 30 % then 21 % is plausible** (a partial vent and a He top-up) **but nowhere recorded**.
- **The case files carry:**
  - `x_o2_role: null`;
  - `x_o2.candidates [0.21, 0.30]` and `x_o2.pressure_block`;
  - Tad and zst at both candidates.
- **What it costs:**
  - Every train test of the three days is hydrogen, which F2 refuses anyway.
  - The 9 constant-flow methane/He tests of 20160 (validation) cannot be forecast without a declared O2 value.
- **Vitiation also lowers O2 on single-value days.**
  - The pressure rises 1–4.4 % within a day (20013, 20024), which by dilution alone takes 21 % down by up to 0.009.
  - Consumption lowers it further.
  - Nothing here corrects for it.

**Zst.** The mixture fraction uses the molar masses of the gases actually present, so He where He is the diluent.
- On N2 days it equals `flame_design.zst_initial`, which assumes N2.
- On He days the N2 convention would differ; it is kept in `inputs.jsonl` as `zst_n2_convention`.
- **Not an input, but a limit of the model:** the forward computes m_tot with `phys.mw_n2` for the diluent, so it
  cannot see helium.

**Planned stop** (`acme_script2.py`, PSI-23). Every s-Flame script is the 2019 build (v1) with
`fiberMoveDuringTest: false`.
- **Rule:** stop = igniterDelayBeforeRetractMs/1000 + constantFlowTimeSec (45 or 60 s on constant-flow tests).
  - igniterDelayBeforeRetractMs is 0.5, 1, 1.5, 2 or 3 s.
  - Files without it fall back on the registered stage sum (20 train-day and 6 validation-day files, spread over 12 days).
- **Check against the script's own stamps** (train days, 14 constant-flow tests ended by the script):

| Quantity | Median | Range |
|---|---|---|
| "MFCs off" stamp minus the planned stop | +1.27 s | −1.0 to +2.4 s |

  - The stamps are truncated to the second, which accounts for about +0.5 s.
  - **So the rule stops about 0.8 s early on s-Flame.** The s-Flame build waits about 2 s between "Flame
    stabilization #2" and "Constant flow".
  - This is inside the committee's ±1 s offset nodes. It was not changed.

**Burner limit.** `burnerTemperatureLimitC` is 250 °C, with a 3 s dwell.

| Partition | Scripts at 250 °C | Without the key | Without a script |
|---|---|---|---|
| train | 162 | 0 | 6 |
| validation | 59 | 0 | 2 |

- **The early-era nodes of F2** (all s-Flame dates are early era, p_late = 0) weigh one limit of **510 °C**; 455 °C
  is the late one. They never see 250 °C.
- **From inputs alone, with F2's own burner-heating law**, for the 22 N2-diluted ethylene/methane train tests F2
  answers:
  - The equilibrium burner temperature is 138–512 °C.
  - It is above 250 °C on **11 tests**. They reach 250 °C after 18–71 s, and **10 of them before their planned
    stop**.
  - F2 treats all of them as able to burn to the stop.
- **Validation days:** 59 scripts carry 250 °C. Of the 16 N2 constant-flow ethylene/methane tests F2 could answer,
  all do. Their burner clock was not computed, because no model is run on validation days.
- **The burner temperature at the start** (`burner_t_start_c`, TCBurner plus the cold junction, about 41 µV/K) is
  24–27 °C on every test. F2 assumes 30 °C.

## 3. The endpoint method on s-Flame train days

`flame_design.py measure` ran unchanged; only its folder was set. Results are in
`endpoints_train.jsonl` / `endpoints_check.json`.

**Channels.**

| Item | Result |
|---|---|
| Status | 162 of 168 train tests ok; 1 has no data file; 5 have no ignition reference (20190 tests without a TP file) |
| Onset | radiometer on 159; PMT on 2; stamp + 2.5 s on 1 |
| Onset minus ignition stamp | median +1.0 s, 5–95 % from −0.4 to +1.7 s (as on CFI-G) |
| Ignition reference | TP stamp 144; flow-on rule 14; stabilization stamp 4 |
| Flow change | setpoint log on all 162 |

**Labels against the script's own account** (the TP `Reason` line):
- On **constant-flow tests the labels agree on 70 of 86** that have a script account, by test:

| Fuel | Labels agree | Note |
|---|---|---|
| ethylene | 16 / 17 | one unreadable: 20029J1's TP file is named for 20029H1, the PMT is not lit at onset, and no onset is seen |
| methane | 9 / 9 | |
| ethylene, ended by the burner limit | 1 / 1 | |

  - **Hydrogen carries most of the disagreements: 17 of 28 agree.**
    - 4 tests where the script stopped on the burner limit read as extinction (the PMTs went dark first).
    - 5 tests where the script detected extinction read as operator termination: the script cut the flow, and
      the dim H2 flame crossed the PMT dark level up to 1.5 s later.
    - 2 tests where the script ended normally read as extinction.
  - **H2/CH4: 27 of 31 agree.** Three scripted ends read as extinction, and one script-detected extinction reads
    as operator termination.
- **Across all train tests**, 53 of the 110 script-detected extinctions read as operator termination. 47 of the 53
  are ramped tests. This is one reason the target set is constant flow only.
- **Timing against the stamps** (constant flow, n = 52 extinctions, 24 terminations):

| Comparison | Median | Range |
|---|---|---|
| Measured extinction minus the script's "Extinction detected" stamp | −1.2 s | −2.8 to +0.1 s |
| Measured termination minus the "MFCs off" stamp | +0.55 s | +0.06 to +0.98 s (the stamp's truncation) |

**Thresholds.**
- **Flags:** `pmt_end_threshold_sensitive` 13, `ended_before_constant_stage` 5 (flames out in under 1 s),
  `end_just_before_flow_change` 5.
- **The dark-level rule works on N2-diluted ethylene and methane.** On H2 it is marginal, as above.
- **Helium days have only H2 and H2/CH4 on train**, so there is no train check of He-diluted ethylene or methane.
- **Flow change:**
  - It begins at the "Flame stabilization #1" stamp + 0.5 s. The s-Flame "Constant flow" stamp comes about 2 s
    later, which is harmless when the ignition flow equals the test flow (all constant-flow tests but 19308H4,
    a validation test).
  - On ramped tests the first ramp step is taken as the flow change (10 within 2 s of the stage).

**Validation days:** 61 rows were written to `cases_validation_outcomes.jsonl` and not read.

## 4. Case files and counts

The target set is constant flow (one constant-flow stage, no ramp keys) with a data file.

| | Train (11 days) | Validation (4 days) | Test (3 days) |
|---|---|---|---|
| Test names (files) | 168 | 61 | 60 (names only) |
| Constant flow, in target set | 91 | 34 | not opened |
| … with every input, x_o2 known | 72 | 25 | |
| … by fuel | ethylene 18, methane 9, H2 33, H2/CH4 31 | ethylene 11, methane 23 | |
| … ambient N2 / He | 56 / 35 | 17 / 17 | |
| … with an endpoint (train) | 87 | (in the outcome file) | |
| … at risk at 1 / 3 / 5 s (train) | 78 / 59 / 49 | | |

- **What the adopted F2 could answer on validation, from inputs only:**
  - **N2-diluted:** 17 tests (19256 ethylene 3, 19308 methane 14). One 19308 test has Tad 1846 K, below the 1900 K
    bound for methane.
  - **He-diluted:** 17 tests. These are 20164 ethylene 8 (one above the c_logm bound, 4.6 mg/s) and 20160 methane 9,
    which need an O2 value.
- **Endpoints on train, target set:**
  - 113 extinction and 73 operator-termination cases; no case is right-censored.
  - Median flame end: ethylene 12 s, methane 2.3 s (all ≤ 4.4 s), H2 12 s, H2/CH4 3.4 s.
- **Files:**
  - `cases_train.jsonl`: inputs and `outcome.endpoint`.
  - `cases_validation_inputs.jsonl`: inputs only.
  - `cases_validation_outcomes.jsonl`: kept apart.

## 5. Scores on train days

All scores are IPCW integrated Brier, 6–120 s, through `sflame_cases.score`. Columns are extinction / operator
termination / sum.

**Incidence baselines, full train target set** (186 cases, 78 tests, 11 days):

| Baseline | Score | Mean of day means (sum) |
|---|---|---|
| CFI-G train incidence (frozen 11.1 file, sha256 `c526a6f1…`), nothing from s-Flame | 0.2529 / 0.2674 / 0.5203 | 0.503 |
| s-Flame train incidence, Aalen–Johansen, in sample | 0.2337 / 0.2022 / 0.4359 | 0.418 |
| The same, leave one train day out | 0.2671 / 0.2315 / 0.4987 | 0.473 |

**What the per-day sums show.**
- **CFI-G incidence** ranges from 0.22 (20029) to 1.02 (20190): it forecasts almost no early terminations, and the H2
  days end by burner-limit stops.
- **Leave-one-day-out** ranges from 0.28 to 0.88.

**F2 (`acme_normal.wsp12`, `sha256:2ae770d8…bc52`) on the train days.**
- **Read this with care:** these days are study data for the author of a law, so this is a forecast on development
  days, not a held-out score.
- **Coverage.** Of 186 cases (78 tests at risk), F2 answered 53 cases on 22 tests (ethylene 15, methane 7; days 19254,
  20029, 19304). It refused 133 cases on 56 tests, at the condition gate:

| Reason | Cases |
|---|---|
| `hydrogen+methane` is not in the fuel table | 60 |
| Hydrogen not admitted (`fuel.admit` = 0) | 36 |
| Hydrogen, and x_o2 / tad / zst missing (20155, 20190) | 34 |
| Ethylene Tad 1829 K below `fuel.tad_min_k` 1900 K (one test of 20029) | 3 |

- The answered cases end by 50 extinctions and 3 operator terminations. The date is read from the run id;
  p_late = 0 on every case.

| On the 53 answered cases | No planned stop | With planned stop |
|---|---|---|
| F2 | **0.0591 / 0.0323 / 0.0914** | **0.0596 / 0.0295 / 0.0891** |
| CFI-G train incidence | 0.2347 / 0.0378 / 0.2725 | same |
| s-Flame train incidence (in sample) | 0.1798 / 0.1210 / 0.3008 | same |
| F2 − CFI-G incidence, sum [95 % day bootstrap] | −0.181 [−0.246, −0.146] | −0.183 [−0.246, −0.117] |

- **Per-day sums for F2:**

| Path | 19254 (28 cases) | 19304 (10) | 20029 (15) |
|---|---|---|---|
| no stop | 0.128 | 0.033 | 0.062 |
| stop | 0.108 | 0.033 | 0.091 |

- **The bootstrap rests on only three days.**

## What is missing or cannot be trusted

- **O2:** unknown per test on 20155, 20160 and 20190, and nominal elsewhere, with an unmodelled vitiation drift.
  This is the largest input error: up to about 0.01 of O2, or about 40 K of Tad.
- **Helium:** F2 has no diluent input, and its m_tot assumes N2. He-diluted ethylene and methane occur only on
  validation days, so there is no train check of either the endpoint method or a model on them.
- **Burner limit:** 250 °C in the scripts against the model's 510 °C. By F2's own heating law, half the answerable
  train tests would reach 250 °C before their planned stop.
- **Hydrogen:** the endpoint labels disagree with the script's account on 11 of 28 H2 constant-flow tests. For an H2
  law, the script's `Reason` line (in `outcome.tp_class`) would be the safer cause.
- **Pressure:** taken on the calculator line, not on a calibration of PSI-23 itself.
- **Bottles:** the 20136 bottle rests on table plus document against the header.
- **Images:** not needed. The outcome comes from the sensor channels and the script's text files.
- **Zips and videos:** the 1,974 zip files (771 GB) and 286 videos (9.3 GB) that the feasibility note lists are not
  needed for this target.

## Reproduce

```bash
python -m pip install cantera            # 3.2.0 was installed here
python tools/wsp/ext/sflame_cases.py tad-check
python tools/wsp/ext/sflame_cases.py inputs
python tools/wsp/ext/sflame_cases.py endpoints     # validation outcomes go to their file, unprinted
python tools/wsp/ext/sflame_cases.py build
python tools/wsp/ext/sflame_cases.py baseline
python tools/wsp/ext/sflame_cases.py f2-train --ignis target-merged/release/ignis.exe   # TRAIN days only
```

# s-Flame endpoints, second version (v2): a fixed rule for the final run

> Written by a data and scoring agent (claude-opus-5-5) on 15 November 2026, at the coordinator's request.
> It concerns the adjudications of `docs/wsp/research/phys_sflame_f4.md` §2.3 and replaces none of them.
> **Read:** the train and validation days only (development data since PROTOCOL 13.6). That means the sensor
> files, the setpoint logs, the TP files and the session logs of those days. **No file of a test day was
> opened.** The days 20142, 20170, 20176 and 19245 were excluded by name, both from the session-log greps and
> from `inputs.jsonl`.
> **The first outcome version is untouched:** `endpoints_train.jsonl`, `cases_validation_outcomes.jsonl` and
> `cases_train.jsonl`, on which the registered looks were scored.
> **Code:** `tools/wsp/ext/sflame_cases.py`, commands `endpoints-v2` and `final-run-build`.
> **Outputs** (local, under `runs/dev/data_all/work/transfer_sflame/`):
> - `endpoints_train_v2.jsonl` and `cases_validation_outcomes_v2.jsonl`: v2 endpoints, with every witness value.
> - `cases_train_v2.jsonl` and `cases_validation_v2.jsonl`: the v1 case rows with the v2 outcome.
> - `endpoints_v2_report.json`.

## In short

- **v2 changes 13 labels of the target set** (constant-flow tests): 12 on train days and 19308H4 on a validation
  day. It **reproduces 15 of the 16 adjudicated labels.** The failure is **20006C1**, where my reading of the
  traces differs from the adjudication.
- **No other constant-flow label changes**, and no same-label end time moves by more than 0.05 s.
- **Outside the target set, 31 ramped tests change.** Ramped tests are not scored.
- **Disputed criterion.** The rule that separates the four adjudicated "extinctions" from the two "detector stops"
  among hydrogen detector stops rests on one threshold, placed between two adjudicated cases. It is the weakest part
  of v2, and it is stated as such below.

## 1. The 16 adjudications against the traces and the logs

**What "EXTINCTION DETECTED" means.**
- Every train and validation session log configures the script's detector identically:
  `ExtinctionDetectionConfigure(averagingDurationSecs:1, numberOfInstruments:1, PMT1Limit:0.1, every other instrument -1)`.
- So the script calls extinction when the **1 s mean of PMT1 falls below 0.1 V**. That is the same photomultiplier
  family the v1 sensor method reads. On a dim hydrogen flame the script's stamp is therefore not an independent
  witness. It is a second reading of the photomultiplier.
- **The independent witnesses are:**
  - the radiometer sum, a thermopile with a response of about 0.3 s;
  - the burner thermocouple (type K, cold-junction compensated);
  - the commanded flow (setpoint log) and the fuel readback.

| Group | Tests | Adjudicated | My reading of the traces | Agree? |
|---|---|---|---|---|
| "Extinctions" that were scripted stops | 20155B1, 20190G1, 19352D5, 19352D6 (burner limit); 20024M1, 20024N3, 20013C3, 20013C4, 20013C5 (scripted end) | operator termination | At the commanded cut, the radiometer is on and at its maximum over the preceding 2 s, on all nine. The burner is 63–296 K above its onset value and still heating or level. The photomultipliers lost the flame early. | **yes, 9 of 9** |
| "Stops" that were extinctions | 20155A2, 20190A1, 19352D4 | extinction | The radiometer is still at its maximum at the cut, and so is it on H1 and N2 below; the photomultiplier sits just above the detector limit. What differs is the burner: it peaks with the fuel off (−0.4 to +1.2 s after the fuel readback falls to 5 %), not later. That fits a flame that went out at the detector's call. | **yes**, on the burner witness only |
| | 20006C1 (H2/CH4 blend) | extinction | **Alive at the cut.** The radiometer is at its maximum of the whole test (14.1 V above base). The photomultipliers flare to 9 V in the 0.5 s before the cut. The burner keeps heating for 14 s after the fuel is off. The detector fired at 26.47 s, during a flicker minimum: the PMT sum was below 0.1 V for about 1 s at 25.0–25.8 s, and then the flame flared back. | **no** |
| Detector stops of a live flame | 20155H1, 20024N2 | operator termination | The radiometer is on and rising to the cut. On H1 the burner keeps heating until 1.8 s after the fuel is off. On N2 the fuel readback never reaches 5 % of its level, so that witness cannot be read. | **yes** |
| Methane | 19308H4 | extinction, 8.2 s | v1's "stop" at 1.75 s is the ignition-to-test flow change: the only constant-flow test whose ignition flow differs from its test flow. The photomultipliers end at 8.28 s; the radiometer is falling at the cut (1.1 V against 7.6 V in the 2 s before); the cut comes at 10.26 s, after the detector. | **yes** |

**The times disagree systematically.** For the 11 operator terminations, v2 times the stop at the commanded cut in
the setpoint log. That is 0.3–1.0 s later than the adjudicated stop times, which the physicist's note does not
derive. I kept the commanded cut, which is the v1 definition of a flow change.

## 2. The rule (v2, `sflame.endpoint.v2`)

All times are seconds since the v1 onset. `flame_design.measure` is run unchanged first. Its onset, PMT end and
status are reused.

**Definitions.**

1. **Test flow and cut.**
   - **Reference time:** the TP `Constant flow` stamp + 0.999 s (the stamp's resolution), or v1's constant-stage
     time when there is no TP file. If the first all-zero command comes earlier, the reference is just before it.
   - **Test flow:** the commanded setpoint row in force at the reference time.
   - **t_reach:** the first command equal to the test flow (within 2 % or 0.005 V).
   - **t_c, the cut:** the first command after t_reach that departs from the test flow.
   - **The difference from v1:** v1 starts looking 0.5 s after `Flame stabilization #1`, so it mistakes an
     ignition-to-test flow switch for the end of the test.
   - **No departure in the log:** v1 stands.
2. **W_rad (radiometer on and not falling).** Take the radiometer sum of `flame_design.radiometer_channels`,
   smoothed over 0.3 s, minus its pre-ignition median. W_rad holds when its value at t_c − 0.25 s is both:
   - **≥ 0.15 V** (flame_design's onset rise); and
   - **≥ 0.85 × its maximum over [t_c − 2, t_c − 0.25] s.**
3. **W_tc (burner hot and not cooling).** W_tc holds when both:
   - the burner temperature at t_c is at least **20 K** above its value at the onset; and
   - over [t_c − 2, t_c], it falls by **no more than 0.5 K/s**.
4. **Burner lag.**
   - **Fuel off:** the first time after t_c that the fuel readback falls below 5 % of its level over
     [t_c − 1.5, t_c − 0.2].
   - **Lag:** the time of the burner maximum in [t_c − 2, t_c + 15] minus the fuel-off time. The lag is undefined
     when the readback never falls that low.
5. **Hydrogen-bearing:** the bottle contains H2 (`hydrogen` or `hydrogen+methane`). For these flames the
   photomultipliers are not a flame sensor.

**Labels, by the script's own stop reason** (the TP `Reason` line; no reason with `MFCs off` = scripted end):

| Branch | Script account | Label |
|---|---|---|
| **R1** | Burner limit, or scripted end | **Hydrogen-bearing:** operator termination at t_c if W_rad or W_tc. Otherwise extinction at the PMT end. **Other fuels:** extinction at the PMT end if the PMTs went dark more than 0.5 s before t_c; otherwise operator termination at t_c. This is v1 with the corrected cut. |
| **R2h** | Extinction detected, hydrogen-bearing, W_rad true, lag defined | Operator termination at t_c if the lag is **≥ 1.5 s** (the burner kept heating after the fuel was off). Otherwise extinction at min(PMT end, t_c). |
| **R2** | Extinction detected, every other case | Operator termination at t_c if W_rad. Otherwise extinction at min(PMT end, t_c). |
| **R3** | No script account | As R1, for both fuel classes. |
| — | v1 status not ok, no command log, no cut | v1 stands. |

**How firm the thresholds are.**
- **Taken from existing definitions:** 0.15 V is flame_design's onset rise; 0.5 s is its label margin.
- **Chosen as round values:** 0.85, 20 K and 0.5 K/s were set before looking at the 16.
- **Placed between two cases:** **1.5 s is placed between 20155A2 (+1.18 s, adjudicated extinction) and 20155H1
  (+1.84 s, adjudicated stop).** It is the one number of the rule set against adjudicated labels, from two points,
  and nothing else supports it.
  - **Without R2h**, using W_rad alone on all detector stops (the `variant_a_label` stored in every row), the rule
    turns A2, A1, D4 and the H2 tests 19352D1 and 20155A3 into operator terminations as well.
  - **The burner lag alone is not a witness.** Without the W_rad condition it relabels about 30 blend extinctions on
    20006, 20132 and 20136 as stops, because the burner soaks heat after a flame has gone out. That is why it is
    read only where the radiometer still sees a flame.

## 3. Applied to every train and validation test (229)

**The 16.** Fifteen labels are reproduced; 20006C1 fails. The end times of the 13 label changes in the target set
are within −0.06 to +1.01 s of the adjudicated times. The stops are 0.3–1.0 s later (section 1). The extinctions
are within 0.1 s, and 20006C1 is 0.27 s from its adjudicated time.

**Labels changed beyond the 16.**
- **Constant-flow (target set):** none. Every other target-set label and end time equals v1. The v2 target labels
  are:

| Partition | Extinction | Operator termination | Other |
|---|---|---|---|
| train | 55 | 32 | 3 without endpoint, 1 unreadable (as v1) |
| validation | 33 | 1 | |

- **Ramped tests (outside the target set): 31.**

| Change | Tests | Days | Why |
|---|---|---|---|
| Operator termination → extinction | 22 | 21 on train (19352, 20006, 20013, 20024, 20136, 20155, 20190) and 19256J1 (validation, ethylene) | The script detected extinction and R2 or R2h lets it stand. The radiometer is off or falling, or the burner peaked with the fuel off. v1 had read the first ramp step as the end. |
| Extinction → operator termination | 1 | 20190P1 (no TP file) | The radiometer and burner were on at the cut. |
| Same label, time moved | 8 | 20006F1, 20006K1, 20013D1, 20013G2, 20013K1, 20013Q1, 20013R1, 20136M1 | The cut is now the first departure from the test flow after it is reached, not v1's first command change after the stabilization stamp. |

- **Per test:** the full list, with each test's witness values, is in `endpoints_v2_report.json`,
  `changed_beyond_adjudicated`.

## 4. The final run

- **Build command:**
  `python tools/wsp/ext/sflame_cases.py final-run-build --final-run`.
  - It builds the test-day inputs with the same `inputs_of`, `assign_o2` and `finish_inputs` code.
  - It writes `final_run/cases_test_inputs.jsonl`.
  - It writes the v2 outcomes, kept apart, to `final_run/cases_test_outcomes_v2.jsonl`.
  - Without `--final-run` it refuses. It has not been run.
- **Same rule, same thresholds:** `measure_v2` and the module constants `RAD_ON_V`, `RAD_HELD_FRAC`, `BURNER_HOT_K`,
  `BURNER_COOL_K_S` and `BURNER_FED_LAG_S`.
- **To fix before the final run** (each is a choice for the lead):
  - **(a)** whether R2h, with its two-point 1.5 s threshold, is kept, or W_rad alone is used (`variant_a_label`);
  - **(b)** whether 20006C1 is scored by the rule (operator termination) or by the adjudication;
  - **(c)** whether stops are timed at the commanded cut, as here, or by the physicist's earlier times.
- **Test-day fuels that the hydrogen-bearing branches touch:** the H2/CH4/He bottle of 20142.
- **Other test days:** the ethylene/He days 20170 and 20176 run through R1 and R2 as hydrocarbons.

## Reproduce

```bash
python tools/wsp/ext/sflame_cases.py endpoints-v2
```

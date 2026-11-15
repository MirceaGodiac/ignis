# ACME planned flow stop from the script parameter file (v2), and the burner temperature limit

> Written by a delegated data-engineering agent (claude-opus-5-5) on 15 November 2026. Nothing is fitted.
> Anything involving a recorded stop, a TP stamp, a command log or a sensor channel uses TRAIN tests and
> train days only. For validation and test days, only the pre-ignition script parameter file was opened
> (and, for the burner limit, the one line the flight script prints when a test point starts), and only
> counts or setting values are reported. Code: `tools/wsp/ext/acme_script2.py`. Scratch and outputs:
> `runs/dev/data_all/work/planned_stop/` (`extract.py`, `measure_train.py`, `check.py`, `burner_limit.py`,
> `burner_trips.py`, and their `.json`/`.csv` outputs). `acme_script.py` is unchanged.

## The rule

The forecast clock is seconds since the flame onset (the radiometer rise measured by `flame_design.py`). The
old `programmed_stop` is the stage sum **S** alone. What the script actually does, from its own "Ignition and
retraction" step onwards:

```
planned stop from the ignition step  = D + S + F
planned stop on the forecast clock   = D + S + F - G
```

| Term | Meaning | v1 build (Flame Design normal, 2019) | v2 build (CFI-G 2021; Flame Design inverse 2021-22, and normal day 21301) |
|---|---|---|---|
| S | stage sum | stabilizationTime1Sec + stabilizationTime2Sec + ignitionToTestRampTimeSec + constantFlowTimeSec | stabilizationTime1Sec + stabilizationTime2Sec + ignitionToTestRampTimeSec + constantFlowTime1/2/3Sec + testFlowRamp1To2/2To3TimeSec |
| D | stabilization #1 starts when the igniter is retracted | igniterDelayBeforeRetractMs / 1000 (2.0 s on train) | igniterDelayBeforeRetractMs / 1000 (1.8 s on almost all tests) |
| F | fiber step before the constant-flow stage | 8 s when fiberMoveDuringTest is true, else 0 (a script behaviour, see below) | \|fiberTestPosition - fiberIgnitionPosition\| / fiberVelocity (1.8 to 2.4 s when the positions differ) |
| G | igniter travel inside the ignition step; the flame lights when it arrives | 0 (the igniter is brought to igniterIgnitionPosition in the preheat step, before the ignition step) | (igniterEndPosition - igniterStartPosition) / igniterVelocityCountsPerSec (120/80 = 1.5 s on almost all tests) |

Branches reported by the tool: `v1|fiber_move_wait`, `v1|no_fiber_move`, `v2|fiber_traverse`,
`v2|no_fiber_traverse`, `<gen>|incomplete_file` (a needed key is missing; no value is given),
`no_constant_flow_stage`, `no_script_file`. It also reports `planned_first_flow_change_s` (end of constant
stage 1, for v2 scripts with stage 2 or 3) and `planned_flow_zero_s` (v1 `useRampFlowToZero`: end of the ramp
to zero, capped by `flowTimeoutSec` counted from the ignition step).

For a standard v2 test D - G = 0.3 s, so the planned stop is S + 0.3 s, or S + 0.3 + 1.9 s with the fiber
traverse. For v1 it is S + 2 s, or S + 10 s with the fiber move.

## How it was derived

All from train TP stage stamps (1 s resolution, truncated) and command logs (ms). Stamp differences are
averaged over many tests: with truncated stamps, the mean of a difference equals the true difference.

- **D.** Across igniter settings, the stabilization #1 stamp minus the ignition stamp averages 1.80 to 1.91 s
  for igniterDelayBeforeRetractMs = 1800 (about 140 v2 tests), 2.00 to 2.18 for 2000 (v1, 95 tests), 3 to 4
  for 3000, 9 for 8200 and 28 for 28000. The session log of a train test prints the script's own ignition
  timing ("Start of ignition: 2000 Ms / Start of igniter retraction: 5000 Ms / Start of stabilization time 1:
  5000 Ms", with fuelFlowIgnitionDeltaMs 2000 and igniterDelayBeforeRetractMs 3000). The old sum left D out.
- **G.** In v2 the flame onset comes 1.9 to 2.5 s after the truncated ignition stamp, about 1.4 to 2.0 s after
  the true start. That matches the igniter travel of 1.5 s; with velocity 40 (travel 3 s) it is about 3.1 s.
  In v1 the onset comes 0.3 to 1.0 s after the stamp. There the "Preheat igniter" stamp falls exactly
  igniterPreheatTimeMs = 1 s before the ignition stamp, so the igniter is already in place when the
  ignition step starts.
- **F, v2.** In the v2 build the "Constant flow" stamp is overwritten at the end of the stage. On
  extinction-detected tests it still marks the start, and there the gap between stabilization and constant
  flow is 1.87 s on average for a 1.80 s traverse, 2.24 s for 2.33 s, 2.2 s for 2.40 s, and 0.1 to 0.2 s with
  no traverse.
- **F, v1.** With fiberMoveDuringTest = true, the constant-flow stage starts 8 s after the stabilization
  stages end in 34 of 40 train tests (6 to 10 in the rest; mean 8.10). With the flag false the gap is 0
  (mean 0.33, n = 46). The 8 s does not depend on the travel distance (2300 to 5600 counts) or the velocity
  (100 to 500 counts/s), and no parameter holds it. **It is the one constant in the rule that is not a
  parameter value.** It is a stated behaviour of the 2019 script build, read from that build's own stamps,
  not from stop times.
- **Bracketing is not the cause.** The earlier attribution (`flame_design_fuel_table_attribution.md`)
  linked the lateness to `useBracketedExposures`. On the train days that flag travels with the TFP fiber
  settings, and these tests separate the two:
  - Bracketed but no fiber move: 19156K1 and 19150A1 are as late as unbracketed tests (old lateness 2.01 and
    1.88 s; new -0.01 and -0.13 s).
  - Unbracketed with a fiber traverse: 22035J7 is as late as the bracketed ones (old 2.36 s, new 0.26 s).
  - Unbracketed with a fiber move: 19150F1, 19150G2 and 19156J7 show the 8 s gap in their stamps.
  - Camera logs show the bracketing cycle runs at frame rate inside the stages and adds no time.

## Check on train tests that ran to the scripted end

**Sample.** Train tests whose TP file classes the end as scripted: a "MFCs off" stamp, no extinction stamp,
no abort reason.

**Recorded stop.** The start of the final descent of the reactant setpoint to zero, from the command log.
The reactant is the fuel (MFCA3) for normal flames and O2 (MFCA1) for inverse flames. Times are seconds after
the measured onset.

**Lateness.** Recorded stop minus planned stop. For the single v1 ramp-to-zero test (19189P1), the planned
value is `planned_flow_zero_s`.

Each cell gives n | median [quartiles] | range | share within 1 s | share within 2 s.

| Investigation, family | Old (stage sum) | New rule |
|---|---|---|
| Flame Design normal | 44 \| +9.86 [+2.05, +10.06] \| +1.48 to +38.82 \| 0 % \| 14 % | 43 \| +0.04 [-0.12, +0.09] \| -1.93 to +3.08 \| 86 % \| 95 % |
| Flame Design inverse | 44 \| +2.30 [+1.13, +2.46] \| +0.06 to +4.45 \| 25 % \| 32 % | 43 \| +0.21 [+0.09, +0.59] \| -0.35 to +4.14 \| 77 % \| 86 % |
| CFI-G normal | 11 \| +0.27 [-0.36, +1.67] \| -0.43 to +1.72 \| 64 % \| 100 % | 11 \| -0.70 [-0.94, -0.07] \| -0.97 to +0.05 \| 100 % \| 100 % |
| CFI-G inverse | 19 \| +0.28 [-0.07, +0.39] \| -0.37 to +0.48 \| 100 % \| 100 % | 19 \| -0.02 [-0.37, +0.09] \| -0.67 to +0.18 \| 100 % \| 100 % |

By branch:

| Branch | Old | New |
|---|---|---|
| FD normal v1\|fiber_move_wait | 24 \| +10.05 [+9.88, +10.10] \| 0 % \| 0 % | 24 \| +0.05 [-0.12, +0.10] \| 92 % \| 100 % |
| FD normal v1\|no_fiber_move | 19 \| +2.03 [+1.88, +2.08] \| 0 % \| 32 % | 19 \| +0.03 [-0.12, +0.08] \| 79 % \| 90 % |
| FD inverse v2\|fiber_traverse | 24 \| +2.39 [+2.28, +2.49] \| 0 % \| 4 % | 24 \| +0.20 [+0.10, +0.29] \| 92 % \| 96 % |
| FD inverse v2\|no_fiber_traverse | 19 \| +0.56 [+0.31, +2.45] \| 58 % \| 68 % | 19 \| +0.26 [+0.01, +2.15] \| 58 % \| 74 % |
| CFI-G normal v2\|fiber_traverse (21088) | 4 \| +1.68 \| 0 % \| 100 % | 4 \| -0.95 \| 100 % \| 100 % |
| CFI-G normal v2\|no_fiber_traverse | 7 \| +0.03 \| 100 % \| 100 % | 7 \| -0.27 \| 100 % \| 100 % |
| CFI-G inverse v2\|no_fiber_traverse | 19 \| +0.28 \| 100 % \| 100 % | 19 \| -0.02 \| 100 % \| 100 % |
| incomplete files (19175B8, 22024L2) | +10.01, +2.08 | no value |

Without ramps vs with ramps (`ignitionToTestRampTimeSec` or `testFlowRamp*` > 0), new rule:

| Group | Without ramps | With ramps |
|---|---|---|
| FD normal | 29 \| +0.03 [-0.10, +0.07] \| 93 % \| 100 % | 14 \| +0.06 [-0.14, +1.22] \| 71 % \| 86 % |
| FD inverse | 33 \| +0.13 [+0.02, +0.25] \| 97 % \| 100 % | 10 \| +2.20 [+1.62, +3.14] \| 10 % \| 40 % |

By session, new rule (median, within 1 s); old median in brackets:

| Session | n | New median | Within 1 s | Old median |
|---|---|---|---|---|
| FD normal 19150 | 8 | -0.10 | 100 % | +9.90 |
| FD normal 19156 | 4 | +0.05 | 75 % | +2.53 |
| FD normal 19171 | 5 | +0.03 | 100 % | +2.03 |
| FD normal 19175 | 8 | +0.04 | 88 % | +8.07 |
| FD normal 19182 | 14 | +0.06 | 79 % | +9.81 |
| FD normal 19189 | 4 | +0.37 | 75 % | +10.85 |
| FD inverse 21328 | 3 | +0.24 | 100 % | +2.44 |
| FD inverse 21349 | 12 | +1.06 | 50 % | +1.36 |
| FD inverse 22024 | 16 | +0.24 | 75 % | +2.39 |
| FD inverse 22035 | 12 | +0.14 | 100 % | +2.33 |
| CFI-G normal 21088 / 21106 / 21125 / 21153 / 21291 | 4 / 2 / 1 / 3 / 1 | -0.95 / -0.70 / -0.27 / -0.07 / +0.05 | 100 % | +1.68 / -0.40 / +0.03 / +0.23 / +0.35 |
| CFI-G inverse 21196 / 21209 / 21228 / 21270 | 4 / 8 / 3 / 4 | -0.12 / -0.23 / -0.37 / +0.02 | 100 % | +0.18 / +0.07 / -0.07 / +0.32 |

**Splitting the residual.** The residual splits into a script part (recorded stop minus the ignition stamp
plus 0.5 s, minus D + S + F) and an onset part (onset minus the ignition stamp plus 0.5 s, minus G). Both
carry the ±0.5 s stamp quantization; their sum does not.

| Group | Script part, median [IQR] | Onset part, median [IQR], range |
|---|---|---|
| FD normal | +0.18 [0.00, +0.53] | +0.25 [+0.04, +0.40], -0.42 to +0.83 |
| FD inverse | +0.25 [-0.16, +1.31] | +0.01 [-0.28, +0.30], -0.57 to +0.72 |
| CFI-G normal | +0.05 [-0.17, +0.33] | +0.63 [+0.23, +1.10], -0.22 to +1.26 |
| CFI-G inverse | +0.12 [-0.11, +0.31] | +0.18 [-0.02, +0.59], -0.60 to +0.88 |

**Against the endpoints the forecasts are scored on.** Two endpoints are in use:

- Flame Design scores against `flow_change_s` of `flame_design.py`, compared here with
  `planned_first_flow_change_s`. Normal: median +0.03, 77 % within 1 s. Inverse: median +0.13, 79 % within
  1 s. The large negative cases (down to -37 s) are measurement artefacts. On tests with an ignition-to-test
  ramp the ramp runs past the reference time the measurement starts from, so the "first departure" is found
  inside the ramp: 21349D2, D3, M5, M6, 19171J1, J3 and 19189D3 have endpoints 2 to 40 s before the actual
  stop. I did not change `flame_design.py`.
- CFI-G scores against the table burn time (integer, "hot ignition" origin). Inverse: old median +1.0, new
  +0.7 (74 % within 1 s both ways). Normal: old +1.0 (55 %), new -0.63 (64 %). These are the "about 1 s late"
  nine normal tests. The table clock itself is the limit:
  - most inverse rows sit 0.7 s after the onset-based stop;
  - day 21270 sits 5 s early;
  - 21270H3 sits 52 s early (a table entry, not the script).

## What remains unexplained (remainders, not absorbed)

1. **Onset delay after the igniter is in place** (physical, not in the file). Median +0.25 s for FD normal,
   about 0 for FD inverse, +0.18 for CFI-G inverse and +0.63 (up to 1.26) for CFI-G normal, longest on
   21088. It is why CFI-G normal comes out 0.3 to 1.0 s early under the new rule.
2. **Ramps run long.** A ramp of R seconds is 20·R setpoint steps, each taking 52 to 56 ms instead of 50
   (+5 to 10 %).
   - In v2 a ramp also costs about +1 s more per ramp: 3 s ramps +1.5 to 1.9 s, 25 s +2.1, 50 s +3.1,
     60 s +4.1, three 5 s ramps +3.0.
   - In v1 the excess varies from -0.5 to +3 s by day.

   This is the main reason FD inverse session 21349 is late.
3. **The v1 fiber wait is not always 8 s.** It is 6 s on 19175B4 (-1.93) and 9 s on 19150A5 (+0.90).
   19189H1 is +1.78, with no stabilization stamps to check.
4. **One v2 fiber-traverse test is late**: 22024J1, +1.16.
5. **Incomplete parameter files.**
   - 19175B8: no igniter or fiber keys; recorded +10 s, so the fiber was moved.
   - 22024L2: no igniter keys.

   The tool gives no value for these; the caller may fall back to `stage_sum_s`.

## Input coverage (script file only; counts)

| Investigation, family | Train | Validation | Test days |
|---|---|---|---|
| CFI-G normal | 80 of 81 (1 no file) | 28 of 28 | 20 of 20 |
| CFI-G inverse | 73 of 73 | 24 of 24 | 34 of 34 |
| Flame Design normal | 106 of 114 (6 incomplete, 1 no file, 1 no constant stage) | 38 of 41 (18 of them v2, day 21301) | 35 of 52 (16 no file, 1 incomplete) |
| Flame Design inverse | 73 of 74 (1 incomplete) | 25 of 25 | 20 of 20 |

One Flame Design inverse table row falls on a day that is in no partition. The v2 rule on Flame Design
normal day 21301 (validation) rests on CFI-G normal train evidence for the same build.

## The burner temperature limit

**Parameters.**

- `burnerTemperatureLimitC` (°C).
- `burnerTemperatureLimitDurationSecs`: the time the burner thermocouple must stay above the limit before
  the script stops the flow. Its abort reason reads "Burner temperature limit exceeded for specified
  duration".

`acme_script2.py` reports both per test as `burner_limit` from the parameter file alone. The file holds them
in these cases:

- **Flame Design:** from day 19156 on, on the 2019 normal days and the 2021-22 inverse days. Not on the first
  2019 days (19115 to 19150) and not on normal day 21301.
- **s-Flame:** on most tests.
- **CFI-G:** never.

**Where the CFI-G limit comes from.** The CFI-G flight script has the limit built in. When each test point
starts, it prints the limit to the session log (`<date>_ACME.log`): "temperature limit of N C is ... mV".
`acme_script2.burner_limit_from_log` (CLI `--session-log`) reads only the "test point <x> STARTED" line and
the limit line after it. The dwell is printed nowhere for CFI-G. Where both a file value and a log line exist
(Flame Design 214 tests, s-Flame 227), they agree every time.

**How many tests have the setting.** Counts are test points with a script file or a log start line.

| Investigation | Test points | Limit in parameter file | Dwell in file | Log line at test start | Either |
|---|---|---|---|---|---|
| PSI-159 CFI-G | 346 | 0 | 0 | 318 | 318 |
| PSI-10 Flame Design | 332 | 219 | 219 | 248 | 253 |
| PSI-23 s-Flame | 338 | 277 (3 partial) | 278 | 288 | 338 |

**By session.** Values of the setting only, every partition. "file" is the parameter file; "log" is the
session-log line.

| Inv. | Sessions | Limit (°C) | Dwell (s) | Source |
|---|---|---|---|---|
| Flame Design | 19115, 19123, 19137, 19142, 19150 | none | none | no file key, no log |
| Flame Design | 19156 | 500 (9), 400 (4 in file, 8 in log) | 3 | file + log |
| Flame Design | 19171, 19175, 19182, 19189, 19196, 19200, 19206 | 500 | 3 | file (+ log, except 19206) |
| Flame Design | 21301 (normal) | 450 | not stated | log only |
| Flame Design | 21328, 21340 | 500 | 1 | file + log |
| Flame Design | 21349 | 500 (4), 475 (7), 450 (4-5) | 1 | file + log |
| Flame Design | 21361, 22035 | 450 | 1 | file + log |
| Flame Design | 22018 | 450 (10-11), 500 (2-3) | 1 | file + log |
| Flame Design | 22024 | 450 (16), 500 (4) | 1 | file + log |
| Flame Design | 22027 | 500 (10-11), 450 (6) | 3 at 500, 1 at 450 | file + log |
| CFI-G | 21088 to 21235 (all sessions to 23 Aug; no log for 21174, 21242, 4 of 21112, 1 of 21196) | 500 | not stated | log only |
| CFI-G | 21256, 21265, 21270, 21277, 21291 (13 Sep to 18-19 Oct) | 450 | not stated | log only |
| s-Flame | every session 19254 to 20190 | 250 | 3 | file (+ log except 20136, 20142, 20190) |

The full per-session counts are in `burner_limit_by_session.json`, and per test in `burner_limit_tests.csv`.
On CFI-G the switch from 500 to 450 °C falls between 23 August and 13 September 2021, not 27 September.

**Check against recorded trips (train only).** Tests whose TP file gives the burner-limit reason. The burner
temperature uses type K with an assumed 23.5 °C cold junction (±1.5 °C, as in `acme_raw_signals_train.md`),
smoothed over 0.2 s. "Time above" runs from the last upward crossing of the setting to the commanded flow
stop.

| Inv., session | Setting (°C / s) | Trips | T at stop (°C) | Time above setting before stop (s) |
|---|---|---|---|---|
| CFI-G 21088, 21106, 21153 | 500 / ? (log) | 3 | 529, 505, 508 | 2.99, 4.36, 3.54 |
| CFI-G 21196M2_1 | no log line (name with suffix) | 1 | 512 | — |
| CFI-G 21270, 21291 | 450 / ? (log) | 3 | 456, 452, 450 | 0.76, 1.02, 0.29 |
| FD 19156 (J2, J4-J7) | 500 / 3 | 5 | 488-498 | never above by my reading |
| FD 19156C1 | 400 / 3 | 1 | 212 | never above: unexplained |
| FD 19171S1, 19175N3 | 500 / 3 | 2 | 507, 490 | 0.82, — |
| FD 21349A1, 21349B1 | 500 / 1, 475 / 1 | 2 | 561, 588 | 0.87, 1.92 (heating fast) |
| FD 22024K3 | 450 / 1 | 1 | 450 | 0.14 |
| s-Flame (19254, 19352, 20013, 20024, 20029, 20155, 20190) | 250 / 3 | 22 | 255-331 | 1.27-3.47, median about 2.7 |

**What the check says.**

- **CFI-G.** The limit in force is the logged value, 500 or 450 °C.
- **CFI-G dwell.** By analogy with the files of the same build era (Flame Design 2021-22: 1 s at 450-500 °C;
  2019 and s-Flame: 3 s), the time above suggests about 3 s at 500 °C and about 1 s at 450 °C. That is an
  inference from the trips, not a stated setting. (The earlier "about 4 s / 1.5 s" was measured from the 500
  and 450 °C crossings of my temperature conversion.)
- **2019 Flame Design.** The trips stop at 488-498 °C against a 500 °C setting. My absolute temperature
  probably reads about 10 °C low on those days (unknown cold junction), so the time above cannot be read
  there.
- **s-Flame.** Times above are mostly 2.3 to 3.4 s against a 3 s setting, consistent within the conversion
  uncertainty.

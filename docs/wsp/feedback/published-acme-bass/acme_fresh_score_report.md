# Agent report (Opus 5.5 medium, 15 Nov 2026, extracted from transcript): ACME fresh batch scored

I scored all 26 tier-A tests on both paths, in the order you set. None was refused at the input or measurement stage, and every endpoint measured `ok`. One deviation: the final run's ignis binary (`b03a35c8…`) is no longer on disk, because `target-merged/release/ignis.exe` was rebuilt at 11:01 today (it is now `7a409b32…`). Before making any forecast I checked that the current binary gives exactly the same results:
- **CFI-G train chain check:** it reproduces the final run's numbers exactly (normal 0.052162 / 0.023997, 210 cases; inverse 0.027091 / 0.060816, 204 cases; difference 0).
- **Validation-day forwards:** I re-ran all 966 F2, F1 and surrogate forwards from the validation days. All match bit for bit (status, support, event curves).

Forecasts were only made after both checks passed.

## Results (integrated Brier, case means, extinction / termination / sum)

**Without the planned stop.** No refusals.

| Block | Cases (tests, days) | F2 | F1 | Ethylene as ethane | Incidence | Irace (R + erfc) | Irace (R only) |
|---|---|---|---|---|---|---|---|
| A1 | 60 (20, 5) | .810 / .933 / 1.743 | 1.816 | 1.754 | 1.090 | 1.618 | 1.961 |
| A2 | 9 (3, 3) | .058 / .198 / .256 | .288 | .265 | .400 | .409 | .781 |
| A3 | 8 (3, 2) | .225 / .540 / .764 | .764 | 1.007 | .819 | .962 | 1.587 |
| Pooled | 77 (26, 10) | .661 / .806 / 1.467 | 1.528 | 1.503 | .981 | 1.409 | 1.784 |

F2 minus each comparator, sum, with 95 % intervals from resampling test days:

| Block | − F1 | − surrogate | − incidence | − Irace | − Irace (R only) |
|---|---|---|---|---|---|
| A1 | −.073 [−.193, −.033] | −.012 [−.052, −.001] | +.653 [+.301, +.770] | +.125 [+.056, +.221] | −.219 [−.569, −.102] |
| A2 | −.031 [−.049, .000] | −.009 [−.017, .000] | −.144 [−.275, −.058] | −.153 [−.342, +.022] | −.524 [−1.14, −.145] |
| A3 | 0 | −.242 [−.413, +.042] | −.054 [−.422, +.166] | −.198 [−.569, +.025] | −.823 [−.975, −.569] |
| Pooled | −.061 [−.127, −.027] | −.035 [−.128, −.000] | +.486 [+.032, +.670] | +.059 [−.102, +.136] | −.317 [−.669, −.162] |

**With the planned stop.** Only cases that F2, F1 and the surrogate all answered are scored:
- F1 and the surrogate read the old stage sum (3 s on most A1 tests), so they refuse at the 3 s and 5 s cutoffs.
- F2 refuses fewer cases (11 on A1, 2 on A3, against 29 and 4 for F1 and the surrogate).
- The refusal reason is always "a stop at or before the cutoff: the fed phase has ended".

| Block | Cases (at risk) | F2 sum | F1 | Surrogate | Incidence | Irace | Irace (R only) | F2 − F1 | F2 − surrogate | F2 − incidence | F2 − Irace |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A1 | 31 (60) | .273 | .088 | .060 | 1.062 | .020 | 1.283 | +.185 [−.025, +.605] | +.213 [−.005, +.635] | −.789 | +.254 [+.019, +.656] |
| A2 | 9 (9) | .054 | .087 | .063 | .400 | .161 | .781 | −.033 [−.053, .000] | −.009 | −.346 | −.107 [−.345, +.023] |
| A3 | 4 (8) | .000 | .026 | .026 | .786 | .000 | .000 | −.026 | −.026 | −.785 | 0 |
| Pooled | 44 (77) | .204 | .082 | .058 | .902 | .047 | 1.064 | +.122 [−.031, +.383] | +.146 [−.011, +.406] | −.698 | +.157 [−.035, +.414] |

The secondary A3 path, which uses the chain's stop verbatim (end of the last stage), has 8 cases: F2 .919, F1 .914, surrogate 1.115, incidence .819, Irace .551, Irace (R only) 1.341. The "pooled" block on that path is A3 alone.

## Labels
- **A1:** all 20 labels are operator termination. 14 of them have the measured flow change within 2 s of the scripted ramp start.
  - The four 19123 L4–L7 tests change flow about 6 s before the planned 13 s. The planner adds an 8 s fiber-move wait there that the observed flow change does not show.
  - 19115B2 and 19156J5 change about 2.9 s and 2.4 s after the planned start.
  - On 18 of the 20, the script's own stage file disagrees with the sensor label: it records "extinction detected".
- **A2:** 6 cases are extinction (19150H1 and 19206L6) and 3 are operator termination (21301C2). All three ran on the ignition fallback with the `no_tp` flag.
- **A3:** all operator termination, at the end of stage 1. 22027B4 ends at 4.4 s, so it is at risk only at the 1 s and 3 s cutoffs.
- **Not scored:** none.

## Code choices (written in the script header before any forecast)
1. **Conditions:** the batch's values verbatim, written with the experiment table's rounding to `input/conditions.csv`, with the stoichiometric mixture fraction recomputed. No test falls outside the F2 domain.
   - For A3 the batch uses the sibling's ambient fuel fraction although the table has each test's own (22024B2: 0.186 in the table vs 0.167 used). I followed the batch; the values are recorded in `inputs.json`.
2. **Prefixes:** built the same way as `transfer_f2.build`, for all three cutoffs, before any endpoint existed. Cases were filtered by at-risk only at scoring. The planned-stop field points to `input/stops.csv`, which holds only values derived from the script files.
3. **Stops:** F2 reads the new stop from `acme_script2`, and F1 and the surrogate the old stage sum. For A3 the primary stop is the end of stage 1, because the conditions are stage-1 flows and the label rule ends the test at the first flow change. The chain's verbatim stop is kept as the secondary path.
4. **A2 without `_TP.txt`:** `flame_design.measure` already handles a missing file through its flow-on fallback. Only the inclusion flag needed it, so no measurement code was changed.
5. **Irace comparator:** applied unchanged, including its ethylene molar mass, on the one propane test (21301C2).

No release, forward, threshold, holdout folder or vault was touched, and nothing is committed. The outputs are only on your disk: `runs/` is gitignored.

Files are in .:
- tools\wsp\fresh\score_acme_fresh.py
- runs\fresh\acme\aggregates.json
- runs\fresh\acme\cases.jsonl
- runs\fresh\acme\endpoints.csv
- runs\fresh\acme\forecasts.json
- runs\fresh\acme\equivalence.json
- runs\fresh\acme\input\
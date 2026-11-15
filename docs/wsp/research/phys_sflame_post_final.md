# The helium row after the final run (PROTOCOL 13.8): no revision is proposed

Author: `claude-fable-5-2`, 15 November 2026. **Written after the final run with every partition seen; in-sample on
all data; no held-out score exists.** Read: `docs/wsp/RESULTS_FINAL_SFLAME.md`, `docs/wsp/feedback/sflame_testday_helium.md`,
the test-day cases under `runs/final/gen7/final/transfer_sflame/` (endpoints version 2), their script keys in
`inputs.jsonl`, and the raw traces of the 25 ethylene tests of days 20170 and 20176 that have an ignition reference
(fuel flow readback, burner thermocouple, radiometer, photomultiplier). **No candidate was typed and none was run.**
Every number below is a re-scoring of the F4 helium release as it was run in the final step (`sha256:7fb9f31f…fbef`;
my copy of the same tensors reproduces 0.7080 and 0.7094 on the 49 cases), on subsets of its cases or with the
oxygen fraction declared instead of mixed. They describe fit; none is a test result.

## 1. In short

- **The largest fault is the flow program, not the row.** 11 of the 26 ethylene-in-helium tests of the test days are
  not constant-flow tests. Their scripts ignite at one flow and ramp to another over 1, 5 or 10 s
  (`ignitionToTestRampTimeSec`), which the constant-flow rule of the case builder does not look at. Eight of them
  are scored, as 18 cases, and they carry **78 % of the loss** (27.2 of 34.7). The development days have no such test.
- **On the 12 scored tests that are constant-flow (31 cases) the row, unchanged, is below both baselines:** 0.242
  against 0.414 (CFI-G train incidence) and 0.377 (s-Flame train incidence).
- **The oxygen fraction can be told apart before ignition.** Pressure block 0 of each day is 30 %, block 1 (after
  the vent and refill) is 21 %. Declaring it takes the 49 cases from 0.708 to 0.654, and the 31 constant-flow cases
  from 0.242 to 0.166. It is the second fault, not the first.
- **No change to the helium row brings the test-day cases, as built, below the incidence.** The 18 ramped cases
  alone put the mean at 0.555 if every other case were forecast perfectly; the incidence is 0.481. So, as asked:
  I propose nothing.

## 2. What the test-day flames did

Days 20170 (25 % ethylene in helium, 18 June) and 20176 (30 %, 24 and 25 June); limit 250 °C held 3 s on every test,
as on the validation day. Loss is per case, without the planned stop; "mix" is the release as run (both oxygen
candidates, weights 0.5), "block" is the same release with the block's oxygen declared.

| test | block | script | fuel mg/s | what happened (flame's own end from the traces) | endpoint v2 | cases | mix | block |
|---|---|---|---|---|---|---|---|---|
| 20170A2 | 0 | constant | 1.74 | limit stop; burner over 250 °C from 10.5 s | stop 13.8 | 3 | 0.896 | 0.497 |
| 20170S1 | 0 | constant | 1.92 | out at 13.4 s, one second before the limit stop | extinction 13.4 | 3 | 0.124 | 0.296 |
| 20170T1 | 0 | constant | 2.44 | out | extinction 11.4 | 3 | 0.029 | 0.062 |
| 20170B1 | 0 | constant | 3.48 | out | extinction 12.7 | 3 | 0.033 | 0.022 |
| 20170C1 | 0 | constant | 5.22 | no flame | – | 0 | – | – |
| 20170N2 | 0 | **1 s step** 1.74 → 3.48 | 3.48 | out at 11.6 s | stop 1.7 (the step) | 1 | 1.910 | 1.858 |
| 20170P1 | 0 | **10 s ramp** 1.74 → 3.48 | 3.48 | full flow at 11.2 s, out at 14.3 s | stop 11.0 (end of the ramp) | 3 | 1.871 | 1.819 |
| 20170R1 | 0 | **5 s ramp down** 1.74 → 0.17 | 0.17 | out during the turn-down | extinction 3.7 | 2 | 1.999 | 2.000 |
| 20170R2 | 0 | **10 s ramp down** 1.74 → 0.17 | 0.17 | out during the turn-down | extinction 6.4 | 3 | 1.989 | 1.989 |
| 20170G2 | 1 | constant | 1.74 | flash | extinction 3.0 | 1 | 0.206 | 0.047 |
| 20170H1 | 1 | constant | 3.48 | flash | extinction 4.2 | 2 | 0.011 | 0.005 |
| 20170J1 | 1 | constant | 5.22 | flash | extinction 4.4 | refused (flow) | – | – |
| 20170U1 | 1 | **10 s ramp down** 1.74 → 0.17 | 0.17 | out during the turn-down | extinction 3.6 | 2 | 1.999 | 1.998 |
| 20176A1 | 0 | constant | 1.90 | limit stop | stop 13.8 | 3 | 1.048 | 0.640 |
| 20176T1 | 0 | constant | 2.28 | out | extinction 14.4 | 3 | 0.058 | 0.120 |
| 20176B1 | 0 | constant | 3.79 | out | extinction 12.1 | 3 | 0.034 | 0.023 |
| 20176C1 | 0 | constant | 5.69 | out | extinction 9.7 | refused (flow) | – | – |
| 20176U1 | 0 | **1 s step** 1.90 → 2.66 | 2.66 | out at 12.9 s | extinction 1.2 (the step) | 1 | 0.021 | 0.053 |
| 20176G1 | 0 | **10 s ramp** 1.90 → 3.79 | 3.79 | full flow at 11.6 s, out at 15.4 s | stop 10.7 (end of the ramp) | 3 | 1.886 | 1.847 |
| 20176H1 | 0 | **10 s ramp** 1.90 → 5.69 | 5.69 | full flow at 11.7 s, out at 14.1 s | stop 10.6 | refused (flow) | – | – |
| 20176J1, X1 | 1 | constant | 1.90 | flash | extinction 4.7, 4.0 | 2, 2 | 0.143 | 0.027 |
| 20176K1 | 1 | constant | 3.79 | flash | extinction 5.1 | 3 | 0.011 | 0.005 |
| 20176P1 | 1 | **10 s ramp** 1.90 → 3.79 | 3.79 | out while the flow was still rising | extinction 5.2 | 3 | 0.010 | 0.005 |
| 20176Q1 | 1 | **10 s ramp** 1.90 → 5.69 | 5.69 | out while the flow was still rising | extinction 5.7 | refused (flow) | – | – |
| 20176V1 | 1 | **1 s step down** 1.90 → 0.19 | 0.19 | no ignition reference | – | 0 | – | – |

Against the eight validation flames (day 20164, 20 % ethylene, 30 % oxygen, all constant flow, ended at 10.3 to 13.0 s):

- **Oxygen fraction.** Both values occur, by block. At the same bottle and flow the block-0 flames last 9.7 to 14.4
  s, as the 30 % validation day; every block-1 flame ended at 3.0 to 5.7 s, on both days (nine tests with an
  endpoint, no exception). The experiment table names the two values in that order ("~30% and ~21% O2 with He"), the pressure
  falls between the blocks (a vent and refill), and on 20176 four of the six block-1 tests carry the next day's
  date. The block is already in the case file (`x_o2.pressure_block`) and is known before ignition. The adiabatic
  temperature inputs cannot tell, since they are computed for both candidates.
- **Flow range.** The constant-flow tests span 1.74 to 5.69 mg/s, as on the validation day (1.54 to 4.62). The 0.17
  and 0.19 mg/s "test flows" are the lower ends of turn-down ramps; no flame was ever held at them.
- **Burner limit.** The same 250 °C. Two limit stops at the lowest constant flow, at 13.8 s on both days, as on the
  validation day (its lowest flow, 12.8 s). The forecast limit at 30 % is 12.6 and 13.5 s.
- **End cause.** Of the 14 constant-flow tests with a flame: 2 limit stops, 12 extinctions. The other "stops" are
  the ends of ramps.
- **What carried over.** At 30 % oxygen the lives are flat in flow and near 12 s at 25 % and 30 % ethylene too,
  which is what the row says. At 21 % oxygen ethylene in helium only flashes; the release calls it a 6.5 s flame,
  and the measure, which starts at 6 s, hardly sees the difference (0.005 to 0.05 per case once the oxygen is
  declared).

## 3. Where the loss is

| test-day ethylene cases | cases (tests) | F4 helium as run | same, oxygen by block | CFI-G train inc. | s-Flame train inc. | share of the loss |
|---|---|---|---|---|---|---|
| all, as built | 49 (20) | 0.708 | 0.654 | 0.481 | 0.416 | 100 % |
| constant flow (ramp time 0) | 31 (12) | 0.242 | 0.166 | 0.414 | 0.377 | 22 % |
| ramped, all | 18 (8) | 1.511 | 1.494 | 0.596 | 0.484 | 78 % |
| – turn-downs to a tenth (20170 R1, R2, U1) | 7 (3) | 1.994 | 1.995 | 0.279 | 0.290 | 40 % |
| – 10 s ramps up (20170P1, 20176G1, 20176P1) | 9 (3) | 1.256 | 1.224 | 0.819 | 0.613 | 33 % |
| – 1 s steps (20170N2, 20176U1) | 2 (2) | 0.966 | 0.956 | 0.708 | 0.583 | 6 % |
| day 20170, constant flow | 15 (6) | 0.232 | 0.179 | 0.416 | 0.376 | |
| day 20176, constant flow | 16 (6) | 0.251 | 0.154 | 0.413 | 0.377 | |
| validation days, helium (for reference) | 35 (14) | 0.110 | 0.110 | 0.330 | 0.324 | |

With the planned stop the figures differ in the third decimal (0.709, 0.242, 1.514).

Why the ramped tests cost what they cost:

- **Turn-downs.** The release is told "0.17 mg/s, constant". At that flow the growth law holds the flame (reduced
  flow 0.14 to 0.18) and the heating law, far outside the flows it was typed on, gives the burner a rise of 3700 K
  and the limit at 1.9 s. So the answer is a certain limit stop, and the flame went out while it was being turned
  down: the maximum loss, 2.0 per case. An 8 W flame cannot heat that burner by 120 K/s; the law needs a cap there.
- **10 s ramps up.** The label ends the test at the end of the ramp as an operator termination, with the flame
  alive; it went out 3 to 5 s later. The release, told "3.5 mg/s from ignition", has it out at 6.5 to 9.4 s.
- **1 s steps.** The endpoint rule takes the step itself as the end (1.7 and 1.2 s), the case of 19308H4 again. With
  the flame's own end (11.6 and 12.9 s) the two tests would score 0.027 on six cases.

## 4. Can one change to the helium row pass?

No. The condition is to be below the CFI-G train incidence on the helium cases of the validation days and of the test
days. On the test days as built, the 18 ramped cases carry 27.2 of the summed loss; with every other case forecast
perfectly the mean is still 0.555 against 0.481. The row cannot reach those cases. A flame said to be held at 0.17
mg/s is never forecast to go out, whatever the helium numbers that keep the validation day right, and a label that
stops a live flame at the end of a ramp is missed by any forecast of extinction. Declaring the oxygen by block, the
one change outside the row that the forward can already carry, gives 0.654.

On the constant-flow cases the row needs no change to be below the incidence on both sets (test days 0.242 against
0.414, validation days 0.110 against 0.330). Its own largest residual there is the one stated in
`phys_sflame_f4.md`: the hinge clock is 2 to 5 s early at 30 % oxygen (9.4 s against 11.4 to 14.4 s), which costs
on the two limit stops at the lowest flow (0.50 and 0.64 with the oxygen declared). The data of all three days
would choose `dil.dlnr` near −0.45 instead of −0.75. I do not propose it: it is a fit to data the row has now seen
in full, it does not move the verdict on the cases as built, and with the oxygen mixed it lengthens the forecast of
the 21 % flashes as well.

**So nothing is typed. The helium row stays as in F4 stage 2, and the hydrogen row as in F4 stage 3.**

## 5. For the engineer

Typed lines: **none.** The helium row is unchanged:

```text
dil.dflux[1] = 1.15    dil.dlogm[1] = 0.6    dil.dlnr[1] = -0.75    (d_far 3.5, rho_d 1.15, le_o2 1.37, k_rel 5.8)
```

Facts for the data lane, each known before ignition; whether the cases are rebuilt, and what a score on rebuilt
cases may be called, is the coordinator's decision, and any such score is post hoc.

1. **The constant-flow rule lets ramps through.** `sflame_cases.py` reads `ignitionToTestRampTimeSec` and does not use
   it in `constant_flow`. With it, these tests leave the target set: 20170 N2, P1, R1, R2, U1; 20176 G1, H1, P1, Q1,
   U1, V1. No target test of the train or validation days has a non-zero value (19308H4 differs in ignition flow
   with a ramp time of 0 and stays). On development days nothing moves.
2. **Endpoints of the two 1 s steps** (if they are kept as constant-flow tests from the step on): 20170N2 extinction
   11.6 s, 20176U1 extinction 12.9 s. Rule v2 still takes the ignition-to-test change as the cut when the change is a
   ramp.
3. **Oxygen by pressure block** on days whose table names "~30% and ~21%": block 0 is 30 %, later blocks 21 %.
   Shown on 20170 and 20176 only (ethylene). Day 20160 has one block and stays a mixture; the hydrogen days 20155
   and 20190 were not examined, and 20190 has three blocks.
4. **The heating law below its range.** At 0.17 to 0.19 mg/s the release answers "limit at 1.9 s" with certainty and
   is inside its declared flow range. A bound on the rise by the heat release (`m_F · LHV` over the burner's 0.31
   J/K), or a lower flow bound for helium, is structure; it touches the shared member scalars, so it is not typed
   here.

What the release scores under each reading (per case, without the planned stop; descriptions of fit):

| test-day ethylene cases | cases | F4 helium | CFI-G inc. | s-Flame inc. |
|---|---|---|---|---|
| as built | 49 | 0.708 | 0.481 | 0.416 |
| constant flow only (item 1) | 31 | 0.242 | 0.414 | 0.377 |
| constant flow, plus the two steps with their own ends (items 1, 2) | 37 | 0.207 | 0.387 | 0.364 |
| constant flow, oxygen by block (items 1, 3) | 31 | 0.166 | 0.414 | 0.377 |
| constant flow and steps, oxygen by block (items 1, 2, 3) | 37 | 0.144 | 0.387 | 0.364 |

## 6. What I said in F4 and what the test days did with it

F4 section 8 said the row came from one composition per fuel and that helium ethylene at 21 % oxygen had never been
seen. Both were on the test days. The 30 % flames at two new fuel fractions behaved as the row says; the 21 %
flames flash, which the row does not say and the measure does not charge. What sank the score was neither: it was
tests of a kind (flow ramps after ignition) that no development day contains and that the cases present as
constant flow.

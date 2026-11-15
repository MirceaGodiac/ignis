# Sixth generation: the one look of 15 November

Registered in [PROTOCOL.md](PROTOCOL.md) section 12.6 and committed (`2750ab3`) before either part
was run. All candidates are **data-informed** (typed after reading CFI-G train data). Integrated
Brier score over 6 to 120 s, lower is better; case means, as every table of this branch quotes.

## Normal burner flames: `acme_normal.wsp7` P3 replaces `acme_normal.wsp5` P1

| | Carried model (P1) | P2 | P3 |
|---|---|---|---|
| **(a) CFI-G validation**, 84 cases, 28 tests, 2 sessions | | | |
| without the planned stop: extinction / flow termination | 0.1552 / 0.0968 | 0.1554 / 0.0958 | 0.1537 / 0.0961 |
| with the planned stop | 0.1452 / 0.0747 | 0.1462 / 0.0748 | 0.1442 / 0.0749 |
| **(b) Flame Design**, 308 cases, 103 tests, 10 sessions (ethylene as ethane, and the propane session) | | | |
| without the planned stop | 0.1092 / 0.3465 | 0.1091 / 0.3403 | 0.1074 / 0.3260 |
| with the planned stop | 0.0703 / 0.0902 | 0.0713 / 0.0822 | 0.0707 / 0.0827 |
| **Decision score** (mean over the two investigations of extinction + flow termination) | | | |
| without the planned stop | 0.3538 | 0.3503 | **0.3416** |
| with the planned stop | 0.1902 | 0.1872 | **0.1862** |
| **Guard** (mean over the two investigations of extinction alone) | | | |
| without the planned stop | 0.1322 | 0.1323 | **0.1306** |
| with the planned stop | 0.1078 | 0.1088 | **0.1075** |

**By the rule of 12.6.** P3 passes on both paths: its decision score and its extinction mean are
not higher than those of the carried model. P2 lowers the decision score on both paths and fails the
guard on both (extinction mean higher by 0.00005 without the planned stop and by 0.0010 with it), so
it is not adopted. P3 is the release to serve for normal burner flames, with and without the planned
stop. **By the older rule** (CFI-G validation extinction alone, section 10) the verdict is the same:
P3 passes on both paths, P2 fails on both.

**How much.** The gain is in flow termination: on Flame Design without the planned stop 0.3465 to
0.3260; on CFI-G validation it is level (0.0968 to 0.0961). Extinction moves by 0.001 to 0.002: on
Flame Design without the planned stop the paired difference to the carried model is -0.0018
[-0.0030, -0.0006] over 10 sessions; with the planned stop the extinction score of P3 on Flame
Design is 0.0004 higher than that of the carried model, and the CFI-G half carries the guard. This
is a small, consistent improvement, not a new level of accuracy.

**What P3 is.** The pair of laws of P1 with (1) the burner temperature limit chosen by the date of
the test, (2) the planned stop placed 0, 1 and 2 s after the stage sum of the script, (3) three
statements corrected that had been typed from mis-tabulated rows of the NASA table, and (4) the
burner-limit clock of the hinge member replaced by the physical heating law of the burner.

**Reports.** Part (a): `runs/dev/gen6/acme_normal/cmp_split_gen6.json` (local). Part (b):
[feedback/flame_design_gen6_result.md](feedback/flame_design_gen6_result.md); its carry of P1
reproduces the Flame Design numbers of 14 November to four decimals on every set. The four test days
of Flame Design were neither measured nor scored.

**Exposure.** CFI-G validation has now been read as aggregate tables once more; the Flame Design set
has been scored twice (14 and 15 November). Both are development numbers. The sealed test partitions
and the test days of Flame Design are unread.

## Burner flames on a second fuel: the fuel table (12.8) and its one revision (12.10)

**12.8, the blind run of the zero-shot fuel table on Flame Design** (ethylene as itself against
ethylene borrowed as ethane, identical cases, extinction without the planned stop;
[feedback/flame_design_fuel_table_result.md](feedback/flame_design_fuel_table_result.md)):

| Family | Fuel table F1 | As ethane | Difference [95 %, sessions] | Registered reading |
|---|---|---|---|---|
| inverse flames, 236 cases, 7 sessions | 0.1077 | 0.1466 | -0.0388 [-0.0658, -0.0101] | adds something |
| normal flames, 240 cases, 9 sessions | 0.1853 | 0.1128 | +0.0725 [+0.0402, +0.1022] | loses something |

The zero-shot expectation for lean inverse ethylene flames (twice the adopted clock) held on the
train days (1.7). The one for normal flames in enriched oxygen was wrong in direction.

**12.10, F2: one revision from the train days, judged once.** What F2 changes is in PROTOCOL 12.10.
Extinction / flow termination / their sum, case means:

| | F1 | F2 |
|---|---|---|
| **Normal flames** | | |
| (a) CFI-G validation, 84 cases, without the planned stop | 0.1537 / 0.0961 / 0.2498 | 0.1537 / 0.0961 / 0.2498 (identical forecasts) |
| (a) CFI-G validation, with the planned stop (F1 old input, F2 new input) | 0.1442 / 0.0749 / 0.2191 | 0.1473 / 0.0798 / 0.2271 |
| (b) Flame Design validation days, 86 cases, without the planned stop | 0.0558 / 0.4028 / 0.4586 | 0.0317 / 0.3933 / 0.4250 |
| (b) Flame Design validation days, with the planned stop | 0.0141 / 0.0826 / 0.0966 | 0.0068 / 0.0724 / 0.0792 |
| decision score (mean of the sums), without / with the planned stop | 0.3542 / 0.1579 | **0.3374 / 0.1532** |
| guard (mean of extinction), without / with | 0.1048 / 0.0792 | **0.0927 / 0.0771** |
| **Inverse flames** | | |
| (a) CFI-G validation, 54 cases, without the planned stop | 0.1288 / 0.0880 / 0.2168 | identical forecasts |
| (a) CFI-G validation, with the planned stop | 0.1348 / 0.0478 / 0.1826 | 0.1349 / 0.0480 / 0.1829 |
| (b) Flame Design validation days, 75 cases, without the planned stop | 0.0871 / 0.1959 / 0.2831 | identical forecasts |
| (b) Flame Design validation days, with the planned stop | 0.0921 / 0.0240 / 0.1160 | 0.0920 / 0.0018 / 0.0938 |
| decision score, without / with | 0.2499 / 0.1493 | **0.2499 / 0.1384** |
| guard, without / with | 0.1080 / 0.1134 | 0.1080 / 0.1134 |

**By the rule of 12.10** F2 replaces F1 in both families and on both paths: its decision score and
its extinction mean are not higher anywhere. `acme_normal.wsp12` F2 and `acme_inverse.wsp12` F2 are
the releases to serve, with the planned stop computed by `tools/wsp/ext/acme_script2.py` as their
input.

**Read it with these in mind.**

- On CFI-G validation with the planned stop, F2 is *worse* than the model before it for normal
  flames (sum 0.2271 against 0.2191; two sessions) and level for inverse flames. The new planned
  stop helped on CFI-G train and did not on those two validation sessions. The rule passes because of
  Flame Design, where the stop was 2 to 10 s late before and is on time now (inverse termination
  0.0240 to 0.0018).
- On the Flame Design validation days every normal-flame case ended by a flow stop, so the extinction
  scores there measure only how little false extinction each release forecasts; the gain of the
  revised ethylene row is on the 12 ethylene tests of one day. Two or three validation days per
  family are few.
- On ethane, propane and butane without the planned stop, F2 returns the forecasts of
  `acme_normal.wsp7` P3 and `acme_inverse.wsp5` P1 exactly.
- Still wrong and known: a nitrogen-diluted ethylene flame lives longer than the growth law says;
  ethylene below 1900 K is refused; the endpoint method misplaces the flow change on tests with an
  ignition-to-test ramp.

Reports: [feedback/flame_design_f2_result.md](feedback/flame_design_f2_result.md); part (a) in
`runs/dev/gen6/f2look/` (local). Flame Design's four test days are unread.

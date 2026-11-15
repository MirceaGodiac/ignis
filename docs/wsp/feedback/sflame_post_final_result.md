# s-Flame after the final run: case version 3

**PROTOCOL 13.8: after the final run with every partition seen; in-sample.** Nothing here is a held-out score. The
test days (20142, 20170, 20176) and the validation days are study data since the final run (13.6, 13.8).

No weight, no forward and no release changes. The physicist proposes no typed change
(`docs/wsp/research/phys_sflame_post_final.md`). What changes is the cases: a third case version sits beside the
other two. Versions 1 and 2 are untouched, and the registered numbers still reproduce on them (table (a) below
reproduces the final run's 0.708).

## The two rules (`tools/wsp/ext/sflame_cases.py`, `case_v3`, command `cases-v3`)

Both rules use only what is known before ignition. Endpoints stay at version 2 (`sflame_endpoints_v2.md`).

1. **Ramp.** A test whose script has a non-zero ignition-to-test ramp time
   (`script.ramp.ignitionToTestRampTimeSec`) is not in the constant-flow set.
2. **Oxygen by pressure block.** On a day whose experiment table names two oxygen fractions, a test takes the
   fraction of its pressure block. Block *i* takes the *i*-th fraction in the order the table names them
   ("~30% and ~21% O2 with He": block 0 is 30 %, block 1 is 21 %), and the inputs `x_o2_role`, `tad` and `zst` become
   that candidate's. The block counts as determined only when the day has as many pressure blocks as named fractions.
   Otherwise the 0.5 / 0.5 mixture stays. This is the literal reading of "where the block cannot be determined" (open
   point 1).

The files sit beside the other versions: `runs/dev/data_all/work/transfer_sflame/cases_{train,validation}_v3.jsonl`,
`runs/final/gen7/final/transfer_sflame/cases_test_v3.jsonl` and `cases_v3_report.json`.

| partition | constant-flow tests | removed by rule 1 | two-oxygen tests | oxygen by block (rule 2) | mixture stays |
|---|---|---|---|---|---|
| train | 91 | 0 | 19 | 8 (20155: 6 at 30 %, 2 at 21 %) | 11 (20190: three blocks for two fractions) |
| validation | 34 | 0 | 9 | 0 | 9 (20160: one block) |
| test | 36 | 11 | 26 | 15 among the remaining (20170: 5 at 30 %, 3 at 21 %; 20176: 4 at 30 %, 3 at 21 %) | 0 |

- **Rule 1.** It removes 0 train and 0 validation tests. The physicist is right that no development test has a
  non-zero ramp time. On the test days it removes 11 tests: 20170 N2, P1, R1, R2, U1 and 20176 G1, H1, P1, Q1, U1,
  V1. Of these, 8 are among the 20 answered helium tests of table (a), which leaves 12 tests and 31 cases.
- **Rule 2 check.** No day in any partition has its oxygen known per test. All five two-oxygen days (20155, 20160,
  20190, 20170, 20176) are "unknown per test: the table gives two values for the day". So the rule cannot be checked
  against a validation day. The only validation day with two fractions, 20160, has one pressure block, and its tests
  keep the mixture.

## Helium cases, per set (`sflame_whatif.py --outcomes v2,v3ramp,v3`, without the planned stop)

The run is a no-op child of `f4-hydrogen`. The work directory is `runs/dev/gen7/whatif/20261115T100731Z-19f4ca45`,
and the run is logged in `runs/dev/gen7/whatif_log.jsonl`. Scores are integrated Brier, 6–120 s, case means. On
helium hydrocarbons `f4-hydrogen` and `f4-helium` give identical forecasts.

| case version | set | cases / tests | F3 stage 2 | f4-helium | f4-hydrogen | CFI-G train incidence | s-Flame train incidence |
|---|---|---|---|---|---|---|---|
| (a) as built for the final run | validation days | 35 / 14 | 0.9990 | 0.1105 | 0.1105 | 0.3299 | 0.3244 |
| (a) | test days | 49 / 20 | 1.0566 | 0.7080 | 0.7080 | 0.4812 | 0.4163 |
| (b) ramped excluded | validation days | 35 / 14 | 0.9990 | 0.1105 | 0.1105 | 0.3299 | 0.3244 |
| (b) | test days | 31 / 12 | 1.0226 | 0.2419 | 0.2419 | 0.4143 | 0.3769 |
| (c) ramped excluded, oxygen by block | validation days | 35 / 14 | 0.9990 | 0.1105 | 0.1105 | 0.3299 | 0.3244 |
| (c) | test days | 31 / 12 | 1.1535 | 0.1664 | 0.1664 | 0.4143 | 0.3769 |

With the planned stop the numbers move by at most 0.003. For f4-hydrogen they are: validation 0.1099 for every
version, and test days 0.7094 (a), 0.2420 (b) and 0.1662 (c). The physicist's figures reproduce: 0.708, 0.242 on 31
cases against 0.414 and 0.377, and 0.166. Per day on the test days, f4-hydrogen scores 0.960 and 0.423 in (a),
0.232 and 0.252 in (b), and 0.179 and 0.154 in (c), for 20170 and 20176.

### Hydrogen, train days (f4-hydrogen; 76 cases, 28 tests)

| case version | without stop | with stop | CFI-G train incidence | s-Flame train incidence |
|---|---|---|---|---|
| (a) v2 as built | 0.3140 | 0.2094 | 0.9087 | 0.6443 |
| (b) ramped excluded | 0.3140 | 0.2094 | 0.9087 | 0.6443 |
| (c) + oxygen by block | 0.3144 | 0.2097 | 0.9087 | 0.6443 |

Rule 2 touches hydrogen tests in helium on 20155, which is why (c) differs.

### Neutrality

- **CFI-G and Flame Design.** No forecast moves: 0 of 210 CFI-G train cases and 0 table changes over 444 Flame
  Design train-day prefixes. The rules apply only to s-Flame cases.
- **Nitrogen ethylene.** f4-hydrogen equals F3 stage 1 in every version. On train it scores 0.0887 (0.0928 with the
  stop) on 43 cases; on validation 0.0400 (0.0401) on 9 cases. The pooled nitrogen rows differ from F3 stage 1 only
  through methane, which is f4-methane's change.

## Reading by 13.8

The helium answer is served if `f4-hydrogen` is lower than the CFI-G train incidence on the helium cases of both the
validation days and the test days under rule (b). The validation days give 0.1105 against 0.3299, and the test days
give 0.2419 against 0.4143. Both are lower, so **the helium answer is served under (b).** Under (c), reported beside,
the test days give 0.1664 against 0.4143. On the cases as built (a), the test days are not served: 0.708 against
0.481. This is in-sample. The rules were written after every partition had been seen.

## Open points

1. Rule 2 has a determination condition: the number of blocks must equal the number of named fractions. This keeps
   the mixture on 20190, with three blocks for two fractions, and on 20160, with one block. It is the literal reading
   and is pinned in `o2_by_block`. If the physicist means "block 0 is the first fraction, every later block the
   second" (section 5 of the note), 20190 would change too. That affects train only: the 11 two-oxygen tests there.
2. Rule 2 leaves untouched any case row that has no candidates' equilibria. There are 3 such rows on train (20155)
   and 16 on the test days (20170 and 20176). None of them is in the constant-flow set, so no score depends on this.

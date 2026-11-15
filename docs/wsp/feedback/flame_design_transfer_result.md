# Flame Design (PSI-10): the frozen burner-flame models scored as registered

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. It carries out the test registered in PROTOCOL section 11.1 as amended: the scoring chain is first reproduced on CFI-G train, the endpoint method is checked on CFI-G train, and the frozen releases are scored on the Flame Design tests outside its test days. Aggregates only; the author has not opened a per-test outcome file.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

**Outcome.** The registered test ran end to end on the amended test set. Arm B verdict, in the protocol's words: `acme_normal.wsp5` P1 **carries over to ethylene as a surrogate**; `acme_inverse.wsp5` P1 is **undecided**.

Two things happened mid-run that you need to know first:

- **Amendment applied before any Flame Design endpoint was measured or scored.** The tests of days 19123, 19200, 19206 (normal) and 21361 (inverse) were dropped: 29 normal and 20 inverse tests. Their endpoints were never measured, there is no row, prefix or artifact for them, and nothing had been scored with them, so nothing was discarded. Their raw text files are on disk only because the fetch had already started; they were not opened.
- **`target/` was deleted by someone else at about 18:07–18:11**, during my download. I did not build. Step 1 and the two `wsp-dev` reference reports were made with `target/release`; everything on Flame Design used `runs/dev/bin/ignis.exe` (sha256 `63f85569…7ddb`, dated 14:24). I re-proved Step 1 with that copy: 1242 of 1242 CFI-G train artifact ids identical to the first run, and `runs/dev/bin/wsp-dev.exe` gives a JSON-identical feedback report. `target/` has since reappeared from someone's rebuild; I did not use it.

## What I built

`tools/wsp/ext/transfer_acme.py` (new; no existing file changed). It builds the prefix files, runs `ignis wsp forward`, and scores with a line-for-line port of the evaluator's IPCW Brier and integrated Brier. It also runs the Step 2 check, the fetch, and the session bootstrap. All outputs are under `runs/dev/data_all/work/transfer_acme/`.

The prefix the development tool builds for a CFI-G case holds the record's allowed conditions, the control stages starting at or before the cutoff, no observations, and `phase_established_at_s = 0`. My prefix for 21088A1 at cutoff 3 is JSON-equal to one the Rust path had stored, with the same `input_hash`.

## Step 1: reproduction on CFI-G train (no planned stop)

| Release | Cases | Extinction | Operator termination | Diff. to `wsp-dev` |
|---|---|---|---|---|
| normal P1 | 210 | 0.051752 | 0.027399 | below 1e-16 |
| normal carry | 210 | 0.051668 | 0.027461 | below 1e-16 |
| normal reference | 210 | 0.150686 | 0.068103 | below 1e-16 |
| inverse P1 | 204 | 0.027091 | 0.060816 | below 1e-16 |
| inverse carry | 204 | 0.035654 | 0.063873 | below 1e-16 |
| inverse reference | 204 | 0.314836 | 0.197526 | below 1e-16 |

- Registered numbers 0.0518 / 0.0274 and 0.0271 / 0.0608 are reproduced to within 2e-5.
- Per case, the event table at 120 s and the endpoints match `wsp-dev` exactly; counts match (81 runs, 33 not at risk; 73 runs, 15 not at risk).
- The incidence baseline is frozen in `incidence_baseline_cfig_train.json` (sha256 `c526a6f1…908b`).

## Step 2: endpoint script on CFI-G train raw files (thresholds unchanged)

- **Coverage:** 153 of 154 train tests measured (one has no data file); onset from the radiometers in all.
- **Labels:** 148 of 153 agree (normal 76/80, inverse 72/73). All 5 disagreements are table operator termination against sensor extinction; three are flames where the photomultipliers went dark 34 to 157 s before the flow stopped.
- **Flame end minus table burn time:** median −0.24 s, quartiles −0.71 / +0.33 s, 5–95 % −1.50 / +1.48 s.

| Within | 0.5 s | 1 s | 2 s | 5 s | beyond 5 s |
|---|---|---|---|---|---|
| Tests (of 153) | 67 | 115 | 140 | 144 | 9 (−156.9 to +50.5 s) |

- **Cases at cutoffs 1 / 3 / 5 s:** 142 / 139 / 133 by the table as development counts them, 153 / 143 / 133 by the measured end. Development leaves out a flame whose table burn time is 1 s; the measured rule keeps one that ends at 1.2 s.

Not bad enough to stop; nothing was tuned.

## Step 3: Flame Design

**Fetch.** All 928 text files of the 232 included tests are on disk (943 MB), sizes equal to the listing, no zip, image or video. The server cut four batches short, so about 1.2 GB was downloaded in total: files the manifest did not know were fetched again.

**Endpoints.** 183 remaining tests, all status ok, all resolved, onset from the radiometers in all. The sensor label and the script's own account agree on 158 of 176 and disagree on 18; I did not adjudicate.

**What the prefix carries.**
- Conditions: `pressure`, `injected_mass_flow`, `x_o2_role`, `x_fuel_role`, `tad`, `zst` from `conditions.csv`, and `fuel_species` as recorded text. No controls, no observations.
- Arm A: nothing more.
- Arm B: two stored conditions, which `flame.rs` and `flame3.rs` read before the species text. Ethane is `fuel_is_ethane = 1`, `fuel_is_butane = 0`; propane is 0 / 0; butane is 0 / 1.
- Planned stop: the condition `programmed_stop` = the script's stage sum. Only the P1 releases read it (carry and reference have `mix.w_script` all zero), so their rows are the same in both variants.

**Counts after the amendment.** No case was refused in any scored block.

| Arm | Tests | Sessions | Cases (cutoff 1 / 3 / 5) |
|---|---|---|---|
| A, propane, day 21301 (validation) | 17 | 1 | 50 (17 / 17 / 16) |
| A, ethylene as recorded, normal | 86 | 9 | 258, all refused |
| A, ethylene as recorded, inverse | 80 | 7 | 236, all refused |
| B, normal, all remaining | 86 | 9 | 258 (86 / 86 / 86) |
| B, normal, train days | 74 | 8 | 222 |
| B, normal, validation days | 12 | 1 (19196) | 36 |
| B, inverse, all remaining | 80 | 7 | 236 (80 / 80 / 76) |
| B, inverse, train days | 55 | 5 | 161 |
| B, inverse, validation days | 25 | 2 | 75 |

Arm A refusals: P1, carry and reference each refuse every ethylene case at the condition gate, with "fuel_is_ethane missing and fuel_species is not ethane, propane or butane" (and the same for butane).

**Arm A, propane (no conclusion drawn).** All 50 cases ended in operator termination. Integrated Brier, extinction / operator termination:

| Variant | P1 | Carry | Reference | Baseline |
|---|---|---|---|---|
| No stop | 0.0522 / 0.5007 | 0.0517 / 0.4862 | 0.1145 / 0.5541 | 0.2704 / 0.5802 |
| With stop (50 of 50 carry it) | 0.0081 / 0.0980 | same as no stop | same as no stop | same |

**Arm B, primary (ethane indicators, no planned stop).** Integrated Brier, extinction / operator termination. Differences are paired, with 95 % intervals from 10,000 session resamples, seed 20261114.

| Set | P1 | Carry | Reference | Baseline | P1 − baseline, extinction | P1 − carry, extinction |
|---|---|---|---|---|---|---|
| Normal, all | 0.1203 / 0.3166 | 0.1201 / 0.3138 | 0.1591 / 0.3232 | 0.2473 / 0.3336 | −0.1270 [−0.1816, −0.0722] | +0.0002 [+0.0001, +0.0003] |
| Normal, train days | 0.1345 / 0.2998 | 0.1344 / 0.2976 | 0.1717 / 0.3015 | 0.2436 / 0.3114 | −0.1091 [−0.1630, −0.0606] | +0.0002 [+0.0000, +0.0003] |
| Normal, validation days | 0.0326 / 0.4205 | 0.0323 / 0.4136 | 0.0813 / 0.4575 | 0.2702 / 0.4699 | −0.2376 (one session) | +0.0003 |
| Inverse, all | 0.1466 / 0.2599 | 0.1466 / 0.2599 | 0.1453 / 0.4681 | 0.2460 / 0.3413 | −0.0994 [−0.2089, +0.0116] | 0.0000 |
| Inverse, train days | 0.1612 / 0.2980 | 0.1612 / 0.2980 | 0.1499 / 0.4875 | 0.2468 / 0.3528 | −0.0855 [−0.2509, +0.0572] | 0.0000 |
| Inverse, validation days | 0.1150 / 0.1783 | 0.1150 / 0.1783 | 0.1355 / 0.4263 | 0.2442 / 0.3168 | −0.1292 (two sessions; no usable interval) | 0.0000 |

P1 − baseline for operator termination, all remaining: normal −0.0169 [−0.0296, −0.0059]; inverse −0.0814 [−0.1356, −0.0099].

**Arm B, with the planned stop (ethane, all remaining).** 255 of 258 normal and 236 of 236 inverse cases carry it.

| Family | P1 | P1 − baseline, extinction | P1 − carry, extinction | P1 − carry, operator termination |
|---|---|---|---|---|
| Normal | 0.0823 / 0.0887 | −0.1650 [−0.2158, −0.1103] | −0.0378 [−0.0634, −0.0163] | −0.2251 [−0.3095, −0.1226] |
| Inverse | 0.1594 / 0.1162 | −0.0866 [−0.2057, +0.0343] | +0.0128 [+0.0031, +0.0227] | −0.1437 [−0.2118, −0.0644] |

**Arm B, sensitivity (no stop, all remaining).**

| Indicators | Family | P1 | Reference | P1 − baseline, extinction |
|---|---|---|---|---|
| Propane | Normal | 0.1039 / 0.3023 | 0.1071 extinction | −0.1434 [−0.1916, −0.0877] |
| Butane | Normal | 0.1289 / 0.2870 | 0.1162 extinction | −0.1184 [−0.1707, −0.0560] |
| Propane or butane (identical) | Inverse | 0.1490 / 0.2692 | 0.1453 extinction | −0.0970 [−0.2086, +0.0171] |

The rule gives the same answer in every other variant: normal carries over, inverse is undecided. The per-day tables for the stop and sensitivity variants are in `fd/report_tables.txt`.

## Verdict for arm B

- **`acme_normal.wsp5` P1: carry over to ethylene as a surrogate.** Extinction 0.1203 against 0.2473, interval [−0.1816, −0.0722].
- **`acme_inverse.wsp5` P1: undecided.** Extinction 0.1466 against 0.2460, interval [−0.2089, +0.0116] includes zero.
- The protocol states one rule for "the models"; the condition holds for one of the two.

Facts that bear on what the verdict means:

- P1 is not better than the model it replaced on ethylene: normal is worse by 0.0002 with an interval that excludes zero, inverse is equal.
- For inverse flames, the reference set that holds no train number scores 0.1453, as good as P1.
- For normal flames with butane indicators, the reference set (0.1162) beats P1 (0.1289).
- The baseline is weak here: extinction is much rarer than in CFI-G train (84 of 258 normal cases, 57 of 236 inverse).
- The inverse verdict depends on my choice of interval. The evaluator's own statistic (mean of session means) gives −0.1210 [−0.2136, −0.0124], which would read "carry over". I fixed the pooled interval as primary before any outcome was computed, so undecided stands.

## Choices where the protocol was silent (all fixed before outcomes)

1. **Endpoint.** A measured flame end is stored as an exact event at `end_s`, which is how the development ingest stores a non-integer burn time. All 183 tests resolved, so nothing was censored.
2. **Cases.** Flame end later than the cutoff. This is the at-risk asymmetry reported in Step 2.
3. **Conditions.** As rounded in `conditions.csv`.
4. **Baseline.** "Ended by the horizon" means the upper end of the table interval is at or before the horizon, as the scorer counts an event. One curve per family and cutoff.
5. **Paired difference and interval.** Per-case contributions to the integrated Brier; the primary interval resamples sessions and recomputes the pooled case mean; the evaluator's session-equal mean is given beside it in the report file.
6. **Verdict.** Applied per model, families never pooled.
7. **Tests without a script stage sum.** Run without the stop (one normal test).
8. **Binary.** Continued with `runs/dev/bin/ignis.exe` after `target/` vanished, on the evidence above.
9. **Fetch.** Batches of 10 tests with retries; files on disk but missing from the manifest were re-fetched so the PSI-10 manifest is complete.

## Not verified

- That `runs/dev/bin/ignis.exe` is byte-identical to the deleted `target/release` binary; only identical outputs were shown.
- The Flame Design labels: 18 sensor/script disagreements; on CFI-G the method's 5 errors all turned operator termination into extinction.
- The threats the protocol already lists: table Tad, table flows against setpoints, the burner limit per session, the onset delay, and whether the 0.4 s offset places the scripted stop correctly on Flame Design.
- Interval stability: 9 and 7 sessions.
- The amendment is not written into `PROTOCOL.md`; I may not edit existing files.
- `flame_design/endpoints.csv` and `transfer_acme/fd/cases/*.csv` hold per-test outcomes, including validation days. I did not open them and printed aggregates only. `endpoints_trial.csv` was not opened either.

## Commands to rerun (repository root, Git Bash)

```bash
T=6,8,10,12,15,20,25,30,40,50,60,80,100,120
O=runs/dev/data_all/work/transfer_acme/cfig

runs/dev/bin/wsp-dev.exe feedback \
  --release sha256:1336c6307fc9023f2434994f49e56cada9ff844000b3ebda29e23bc6bb9a77f4 \
  --release sha256:1ac0af0b7756b9d62ca243fb624e355837d68d2ca34228b93f2860ba0131f5c7 \
  --release sha256:67aea1d70bc802554ae33a5855421123d4c130622c10407789ad31a1ca875fc4 \
  --partition train --times $T --out $O/wspdev_fb_train_normal.json

runs/dev/bin/wsp-dev.exe feedback \
  --release sha256:aa05ddcbe976dbb1a5537987d05270b28066551c39ab24db671be65692dcea0e \
  --release sha256:68c840f36971680d7b46fa16de4c62e79190215cfa82930787c6b951af3a80db \
  --release sha256:d7260f5962954904503a756a8f529b40988697aeae2ea57d18c6b6bfa1e0ea5d \
  --partition train --times $T --out $O/wspdev_fb_train_inverse.json

python tools/wsp/ext/transfer_acme.py cfig --ignis runs/dev/bin/ignis.exe --tag devbin \
  --rust-prefix artifacts/analysis/2f4322612f901c0e0d2b334c25305d5d0eeb7684fd59f3aef2f8784281a3e636/prefix.json
python tools/wsp/ext/transfer_acme.py cfig-endpoints
python tools/wsp/ext/transfer_acme.py fd-fetch --ignis runs/dev/bin/ignis.exe            # dry run
python tools/wsp/ext/transfer_acme.py fd-fetch --ignis runs/dev/bin/ignis.exe --execute
python tools/wsp/ext/transfer_acme.py fd-tests-file
python tools/wsp/ext/flame_design.py endpoints \
  --tests-file runs/dev/data_all/work/transfer_acme/fd/tests_remaining.txt
python tools/wsp/ext/transfer_acme.py fd --ignis runs/dev/bin/ignis.exe
python tools/wsp/ext/transfer_acme.py fd-tables
```

Without `--tag`, `cfig` writes to the untagged folder, which holds the first run's artifacts. `fd` refuses to run unless a chain check passed with the same binary hash.

## Files

- `tools\wsp\ext\transfer_acme.py`
- `runs\dev\data_all\work\transfer_acme\incidence_baseline_cfig_train.json`
- `runs\dev\data_all\work\transfer_acme\cfig\chain_check_devbin.json`
- `runs\dev\data_all\work\transfer_acme\cfig\endpoints_check.json`
- `runs\dev\data_all\work\transfer_acme\fd\report.json`
- `runs\dev\data_all\work\transfer_acme\fd\report_tables.txt`

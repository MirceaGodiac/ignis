Diagnostic only. A perturbation is a question about the model, not a proposed value.

# The planned stop of `acme_script2.py` against the stage sum, train only (PROTOCOL 12.3, step 3)

> Written by a scoring agent (claude-opus-5-5) on 15 November 2026. **TRAIN only:** CFI-G train partition and the Flame Design train days (normal 19115–19189, inverse 21328, 21349, 22024, 22027, 22035).
> No validation-partition or validation-day outcome was read or scored.
> Binary `target-merged/release/` (ignis sha256 in `runs/dev/data_all/work/transfer_acme/stop_v2/report.json`).
> Tools (new): `tools/wsp/ext/make_data_ext2.py` and `tools/wsp/ext/stop_v2_attribution.py`.
> The input rule is that of [acme_planned_stop_v2.md](acme_planned_stop_v2.md) (`acme_script2.py`, unchanged).

The diagnostic committees zs1 (`runs/dev/gen6/fuel/diag/stop_variants.json`):

| Family | zs1 release | What it is |
|---|---|---|
| normal | `sha256:e509224c…afe9` | F1's members; planned stop read at −1, 0 and +1 s (weights 0.25 / 0.5 / 0.25); burner-limit era days early through 23 Aug 2021, late from 13 Sep |
| inverse | `sha256:f3fc9eac…de2` | F1's members; the same three readings |

## 1. Inputs: `runs/dev/data_ext2`

**What was built.** A full copy of `runs/dev/data_ext`, which was not modified.

- In `work/records/acme_{normal,inverse}/experiments.jsonl`, `programmed_stop` of every CFI-G run with a script file is replaced by `acme_script2.py`'s `planned_stop_s`.
- The replacement carries method `wsp.ext.acme.cond.programmed_stop` version 2, with the branch noted and the sha256 of the parameter file.
- Nothing else is changed.
- Every run with a file got a value, so no condition had to be removed.

Counts of runs per partition (from the split file; "not in split" are record rows the split does not list):

| Family | Train | Validation | Test | Not in split | No script file (left unchanged) |
|---|---|---|---|---|---|
| normal | 80 replaced (had the old value) | 28 replaced (had old) | 20 new (had none) | 47 new | 1 train, 3 not in split |
| inverse | 73 replaced | 24 replaced | 34 new | 22 new | 1 not in split |

**Flame Design.** The with-stop path of the chain takes `acme_script2.py`'s value per test.

- Normal train days: 37 tests in branch v1|fiber_move_wait and 28 in v1|no_fiber_move. 3 have incomplete files (19115K1, 19150J1, 19175B8) and run without a planned stop under the new rule.
- Inverse train days: 40 in v2|fiber_traverse and 14 in v2|no_fiber_traverse. 1 has an incomplete file and runs without a stop.

## 2. With the planned stop: before (old input, typed releases) and after (new input, zs1)

**Releases compared:**
- CFI-G: P3 (normal) or P1 (inverse) on `data_ext`, against zs1 on `data_ext2`.
- Flame Design: F1 with the stage sum, against zs1 with the new stop, ethylene as itself.

**Coverage.** The six normal tests with table T_ad below 1900 K are outside both (F1 and zs1 refuse them). Every other case is answered by both.

**Check against `wsp-dev feedback`.** Each CFI-G side equals a `wsp-dev feedback --partition train` report with the matching `IGNIS_DATA` to below 1e-17. P3 on `data_ext` gives 0.053834 / 0.014010, the 12.6 train numbers with the stop.

Integrated Brier 6–120 s, extinction / operator termination / sum, case means:

| Train set | Cases (tests, sessions) | Before | After | After − before, sum |
|---|---|---|---|---|
| CFI-G normal | 210 (71, 7) | 0.0538 / 0.0140 / 0.0678 | 0.0537 / 0.0135 / 0.0672 | −0.0006 |
| CFI-G inverse | 204 (71, 7) | 0.0281 / 0.0341 / 0.0622 | 0.0281 / 0.0331 / 0.0612 | −0.0010 |
| Flame Design normal, train days | 204 (68, 8) | 0.1429 / 0.0790 / 0.2219 | 0.1556 / 0.0859 / 0.2416 | +0.0197 |
| Flame Design inverse, train days | 161 (55, 5) | 0.1333 / 0.1499 / 0.2833 | 0.1337 / 0.1379 / 0.2716 | −0.0117 |

Change per session (after − before). Counts are improve / worsen / unchanged:

| Train set | Extinction | Termination | Sum | Sessions that move most (sum) |
|---|---|---|---|---|
| CFI-G normal | 2 / 1 / 4 | 2 / 1 / 4 | 2 / 1 / 4 | 2021-03-29 −0.0040; 2021-04-16 +0.0008 |
| CFI-G inverse | 1 / 6 / 0 (all within 0.0002) | 4 / 3 / 0 | 4 / 3 / 0 | 2021-08-30 −0.0080; 2021-07-28 −0.0024 |
| FD normal | 1 / 7 / 0 | 3 / 5 / 0 | 3 / 5 / 0 | 19175 +0.1042; 19182 −0.0288 |
| FD inverse | 1 / 4 / 0 | 5 / 0 / 0 | 4 / 1 / 0 | 21349 −0.0306; 21328 −0.0217 |

Tests that ended by operator termination, one row per test at its earliest cutoff. Centre is median ln(t50 / t_end) with IQR; width is median ln(t90 / t10); "inside" counts ends in [t10, t90]. CFI-G uses the midpoint of the table interval as t_end.

| Train set | Tests | Before | After |
|---|---|---|---|
| CFI-G normal | 17 | −0.15 [−0.44, −0.04], w 1.18, inside 13 | −0.15 [−0.44, −0.01], w 1.18, inside 14 |
| CFI-G inverse | 32 | −0.03 [−0.04, +0.27], w 0.06, inside 14 | −0.03 [−0.04, +0.27], w 0.11, inside 18 |
| FD normal | 40 | −0.21 [−0.27, −0.06], w 0.56, inside 15 | −0.03 [−0.10, +0.00], w 0.72, inside 35 |
| FD inverse | 44 | −0.11 [−0.16, −0.01], w 0.06, inside 8 | −0.00 [−0.02, +0.02], w 0.11, inside 31 |

**Why Flame Design normal gets worse although its stop is now placed right.** Per test, the sum changes by +4.02 in total over the 204 cases.

- **One incomplete file accounts for most of it.** 19175B8 carries +2.39 of the +4.02 (extinction +0.88, termination +1.51). It has no igniter or fibre keys, so it loses its planned stop altogether under the new rule. Without it the change is about +0.008 per case.
- **The rest comes from held flames given more time.** The next-largest increases are held flames at 36–40 % O2 in 19175 and 19150, all in branch fiber_move_wait. There the new stop is 10 s later than the stage sum, so F1's members keep their over-forecast extinction hazard (error 1 of `flame_design_train_days_anatomy.md`) running 10 s longer before the stop.
- **So the stop is placed right but the forecast gets worse.** Better stop placement exposes the extinction over-forecast at enriched oxygen rather than fixing it.

## 3. Without the planned stop: CFI-G train normal, zs1 against P3 (the era days alone)

- **Era of the train tests.** No train test changes era. The train sessions are dated 29 March to 11 June and 18 October 2021, so none falls between 24 August and 26 September, the only days where the two era rules differ. p_late is 0 on 174 cases and 1 on 36, for both releases.
- **Scores.** zs1 and P3 give the same score to every printed digit: 0.0522 / 0.0240 / 0.0762 (`wsp-dev feedback` gives 0.052162 / 0.023997 for both). The changed era days have no effect on CFI-G train.

## 4. The seven tests whose measured flow change falls inside an ignition-to-test ramp

None of the seven is in a scored train-day set of this run, so they carry 0 % of its termination loss, before and after:

- **19171J1, 19171J3, 21349M5, 21349M6:** ramped rows of the experiment tables, never included by 11.1's rule.
- **21349D2 and 21349D3:** constant-flow, but outside the inverse release's declared domain, so not included.
- **19189D3:** included, but its table T_ad is 1729 K, so F1 and zs1 refuse it.

In the surrogate run of 12.8 (P3 with ethane's indicators), 19189D3 is scored. There it carries 3.6 % of the train-day termination loss of normal flames without the stop (0.788 per case, 3 cases) and 5.9 % with it (0.366 per case). It ended by operator termination. The endpoint method was not changed.

## Not verified

- **Incomplete files.** The fallback for them is not decided here: the tool gives no value, and a caller may use the stage sum.
- **CFI-G inverse era.** For P1 (wsp5) the forward reports no era. zs1 inverse reports p_late 0.5 on every case, and its scores change only through the stop.
- **Endpoint timing.** The ramp-length and onset remainders listed in acme_planned_stop_v2.md are untouched; they are visible in the 13 of 44 FD inverse terminations that still fall outside the band.

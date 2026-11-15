# s-Flame validation days: F2 against the incidence baselines (PROTOCOL 13.1)

> Written by a data and scoring agent (claude-opus-5-5) on 15 November 2026. This runs the look registered in
> PROTOCOL 13.1 once. Nothing was fitted, and no model, input or threshold was changed.
> **Part A is aggregates only.** The per-case validation file is `runs/dev/data_all/work/transfer_sflame/validation_13_1.json`
> (local; it is aggregates too). The test days 20142, 20170 and 20176, and the unlisted day 19245, were not opened.
> Tool: `tools/wsp/ext/sflame_cases.py validation-f2` (run once) and `train-study`.

## The run

**Release.** `acme_normal.wsp12` F2, `sha256:55fcf4cbbf500ca57a4e4207426826d5c3d3b21d84fb279479e29b7e5a65971d`.

**Binary.**
- `target-merged/release/ignis.exe` was rebuilt during the day. Its sha256 is now `98e3a74f…f60b`.
- The chain check of `fd_f2` was done on `b03a35c8…48ffb`.
- Before the look, I re-ran all 372 s-Flame train-day F2 prefixes with the new binary (`binary_check.json`).
  Admission was the same on all of them, and every event table was identical: the maximum difference is 0.

**Inputs and endpoints.** These are the case files of `docs/wsp/feedback/sflame_inputs.md`, built before the
look.
- The F2 forecast reads the inputs only.
- The endpoints come from `cases_validation_outcomes.jsonl`, written by `flame_design.py` unchanged. It was read
  only by the scorer.

**Baselines.** Both baselines answer every case:
- **CFI-G train incidence:** the frozen 11.1 file, sha256 `c526a6f1…908b`. It contains nothing from s-Flame.
- **s-Flame train incidence:** the Aalen–Johansen incidence of the s-Flame train target set, by cutoff.

**Score.** IPCW integrated Brier score over 6–120 s, at cutoffs 1, 3 and 5 s. Every endpoint is an exact event, so
every IPCW weight is 1.

## A. Validation days (13.1)

### Tests, cases and refusals

**Set.** There are 34 constant-flow tests on the four validation days.

| Tests | Days | Status |
|---|---|---|
| 9 | 20160, methane in He | **Outside the set:** no per-test O2 fraction (18 cases at risk) |
| 8 | 20164, ethylene in He | **Refused by 13.1:** helium dilution, since F2 has no diluent input (24 cases at risk) |
| 17 | 19256 ethylene (3), 19308 methane (14), both in N2 | **Run** |

**The 17 tests run.**
- **Cases at risk:** 47 on each path.
- **Refused by the binary:** 1 case of 1 methane test. The reason is "tad below the bound of methane in this
  release (`fuel.tad_min_k` = 1900 K)".
- **Answered: 46 cases, 16 tests, 2 days.**
  - 3 ethylene tests (9 cases) on 19256.
  - 13 methane tests (37 cases) on 19308.
- **Endpoints of the answered cases:** 45 extinctions and 1 operator termination.
- **Era:** p_late = 0 on every case.

### Scores on the cases F2 answers (46 cases)

Columns are extinction / operator termination / sum.

| | Without planned stop, case mean | Without, mean of day means | With planned stop, case mean | With, mean of day means |
|---|---|---|---|---|
| **F2** | **0.2195 / 0.0424 / 0.2619** | 0.1519 / 0.0264 / 0.1783 | **0.2428 / 0.1270 / 0.3698** | 0.1665 / 0.0789 / 0.2454 |
| CFI-G train incidence | 0.2523 / 0.0269 / 0.2793 | 0.2380 / 0.0199 / 0.2579 | same | same |
| s-Flame train incidence | 0.1936 / 0.1282 / 0.3218 | 0.1853 / 0.1247 / 0.3100 | same | same |

**F2 minus the CFI-G incidence (sum, case mean), with a day bootstrap of 10,000 resamples (seed 20261114):**

| Path | Difference | Interval | Mean of day means |
|---|---|---|---|
| Without the stop | −0.0174 | [−0.1821, +0.0227] | −0.0797 |
| With the stop | **+0.0905** | [−0.1819, +0.1568] | −0.0126 |

**The interval is not a stable interval.** With two days there are three distinct resamples, so it spans the two
days' own values. The two days disagree, as the fuel split below shows.

### Ethylene and methane apart

| Fuel | Path | F2 | CFI-G incidence | F2 − CFI-G (sum) |
|---|---|---|---|---|
| Ethylene (19256): 9 cases, 3 tests, 1 day, all extinctions | no stop | 0.0408 / 0.0000 / 0.0409 | 0.2145 / 0.0084 / 0.2229 | −0.1821 |
| | stop | 0.0410 / 0.0000 / 0.0410 | same | −0.1819 |
| **Methane** (19308): 37 cases, 13 tests, 1 day; 36 extinctions, 1 termination | no stop | 0.2629 / 0.0527 / 0.3157 | 0.2615 / 0.0314 / 0.2930 | +0.0227 |
| | stop | 0.2919 / 0.1578 / 0.4498 | same | **+0.1568** |

- **The methane row of F2's fuel table meets data here for the first time.** On this day of ~30 % O2 it does no
  better than the CFI-G incidence on extinction.
- **The planned stop makes methane worse.** Operator termination goes from 0.053 to 0.158, against 0.031 for the
  baseline.
- The s-Flame train incidence scores 0.3293 on methane, and 0.2907 on ethylene.
- Each fuel sits on one day, so neither row has an interval.

### The 250 °C limit, by F2's own heating law (inputs only)

- **F2 assumes a 510 °C limit.** The scripts set 250 °C.
- **Answered tests:** 9 of the 16 have an equilibrium burner temperature above 250 °C. **7 reach 250 °C before
  their planned stop.** All 7 are methane tests on one day.

| Subset | Path | F2 | CFI-G incidence | s-Flame train incidence |
|---|---|---|---|---|
| Reach 250 °C before the stop: 19 cases, 7 tests, 1 day | no stop | 0.4360 / 0.1026 / 0.5386 | 0.2540 / 0.0532 / 0.3073 | 0.1997 / 0.1411 / 0.3407 |
| | stop | 0.4920 / 0.3061 / 0.7981 | same | same |
| Do not reach it: 27 cases, 9 tests, 2 days | no stop | 0.0671 / 0.0001 / 0.0672 | 0.2511 / 0.0084 / 0.2596 | 0.1894 / 0.1191 / 0.3084 |
| | stop | 0.0675 / 0.0009 / 0.0684 | same | same |

- **On the tests the law keeps below 250 °C, F2 leads the CFI-G incidence by about 0.19 in sum on both paths.**
  The F2 − CFI-G interval there is [−0.198, −0.182] (2 days).
- **On the 7 tests the law sends past 250 °C, F2 loses by 0.23 (no stop) and 0.49 (stop).** These flames went out
  by extinction, 18 of 19 cases. F2 forecast them to burn on, and with the stop it forecast their end by the
  scripted stop.

### What 13.1 says about these numbers

The rule compares F2's sum with the CFI-G incidence on the cases F2 answers, on both paths.

| | Without the stop | With the stop | Outcome |
|---|---|---|---|
| **Case means** (the convention of 11.1 and 12.10) | lower (0.2619 < 0.2793) | higher (0.3698 > 0.2793) | **undecided** |
| Means of day means | lower (0.1783 < 0.2579) | lower (0.2454 < 0.2579) | would read "carries over" |

The protocol does not name the mean. Two days do not make a stable interval on either reading.

## B. Train days: the study step

The cases are in `runs/dev/data_all/work/transfer_sflame/train_days_cases.jsonl`: **186 rows**, one per train-day
case (test, cutoff) of the target set. Each row holds:
- the inputs, diluents, burner limit and burner start temperature;
- the endpoint and the script's own stop reason (`script_account`);
- `hit_250C_limit`, from the script's "Burner temperature limit exceeded";
- the contributions of both incidence baselines;
- for each path, either F2's two incidence curves at the 14 horizons with its contributions and its burner clock,
  or F2's refusal reason.

The aggregates are in `train_study.json`. Like the rest of part B, these train days are study data.

**Where F2's loss sits.** F2 answers 53 of the 186 cases: 22 tests, all in N2. Its sum is 0.091 without the stop
and 0.089 with it, against 0.273 for the CFI-G incidence on the same cases.

| Train subset (F2-answered cases) | Cases | F2 sum, no stop / stop | CFI-G incidence sum | Share of F2's total loss, no stop / stop |
|---|---|---|---|---|
| Ethylene | 43 | 0.105 / 0.102 | 0.271 | 0.93 / 0.93 |
| Methane (19304, ~21 % O2, all extinctions within 4.4 s) | 10 | 0.033 / 0.033 | 0.279 | 0.07 / 0.07 |
| Hit the 250 °C limit (1 ethylene test) | 3 | 1.028 / 0.830 | 0.799 | **0.64 / 0.53** |
| Did not hit the limit | 50 | 0.035 / 0.045 | 0.241 | 0.36 / 0.47 |
| F2's heating law reaches 250 °C before the stop | 23 | 0.179 / 0.174 | 0.301 | 0.85 / 0.85 |
| It does not | 30 | 0.024 / 0.024 | 0.251 | 0.15 / 0.15 |
| Endpoint extinction | 50 | 0.035 / 0.045 | 0.241 | 0.36 / 0.47 |
| Endpoint operator termination | 3 | 1.028 / 0.830 | 0.799 | 0.64 / 0.53 |

**What the train days show.**
- **Most of F2's loss sits on the burner limit.** More than half of it comes from the single answered test the
  250 °C limit stopped. F2 forecast little operator termination for it, scoring 0.56 on termination against 0.53
  for the baseline.
- **On the extinction cases, which are everything else, F2 is far better than both baselines.**
- **The cases F2's own heating law flags as reaching 250 °C carry 85 % of its loss**, and only 3 of their 23 cases
  were actually stopped by the limit.
- **Methane on train is one day at ~21 % O2.** Every flame there went out within 4.4 s, and F2 forecast them well.
- **Fuel and dilution outside F2:** the train target set has 186 cases. F2 refuses the 70 hydrogen cases, the 60
  H2/CH4 cases and the 50 helium cases. The baselines score:

| Train subset | CFI-G incidence sum | s-Flame train incidence sum | Note |
|---|---|---|---|
| Hydrogen | 0.833 | 0.612 | 55 of 70 cases end by operator termination, mostly the 250 °C stop |
| H2/CH4 | 0.386 | 0.356 | |
| Helium ambient | 0.739 | 0.545 | |
| Cases that hit the 250 °C limit (33 cases, 13 tests, all fuels) | 0.887 | 0.633 | |

## Reproduce

```bash
python tools/wsp/ext/sflame_cases.py validation-f2 --ignis target-merged/release/ignis.exe   # 13.1, refuses to run twice
python tools/wsp/ext/sflame_cases.py train-study --ignis target-merged/release/ignis.exe
```

## Refusals made by the lane's adapter, not by a release (added with F4)

As the helium dilution of 13.1 was a refusal of the registration (F2 had no diluent input), hydrogen/methane
blends are now refused by the lane's adapter: `tools/wsp/ext/sflame_cases.py` (`forward_cases`, used by every F3/F4
path and by `sflame_whatif.py`) does not hand a prefix whose `fuel_species` is a blend to a release that carries no
blend law (`BLEND_LAW_RELEASES`, empty), and counts the case as a condition-gate refusal with the reason "the fuel is
a blend and this release carries no blend law". Reason: `phys_sflame_f4.md` types no blend law, and the
`acme_normal.wsp13` forward, once hydrogen is admitted, would answer a blend through its oxygen-share rule, which fails
on the hydrogen-rich bottle (day 20013). The forward is not changed.

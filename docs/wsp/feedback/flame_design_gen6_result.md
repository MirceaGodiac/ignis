# Flame Design (PSI-10): the sixth-generation normal-flame releases, part (b) of the look of PROTOCOL 12.6

> Written by a scoring agent (claude-opus-5-5) on 15 November 2026. Nothing fitted, no model or threshold changed.
> It runs part (b) of the look registered in PROTOCOL 12.6 (commit `2750ab3`): the Flame Design transfer set of
> 11.1 as amended, normal flames only, without and with the planned stop. **Aggregates only.** Per-test rows are in
> `runs/dev/data_all/work/transfer_acme/fd_gen6/cases/` (local, not in git) and are not quoted here.
> Part (a) (CFI-G validation) was not run by this agent, so the rule of 12.6 is not applied here.

## Releases

| Role | Release (`acme_normal.wsp7`) |
|---|---|
| carry of P1 | `sha256:2a898731e18dcefefff39146810fb4736392fe24aab02b40079202bc316b0963` |
| P2 | `sha256:ec1aa02e991f8e3feec469edbb374c54dd135faa1c58a4e8ce6bb7d03fae9ba0` |
| P3 | `sha256:96a91f15ab94ff6a0b32e51f34e5c1f71d8377ae618b4cdd331bdb1b20fdaa34` |
| baseline set | `sha256:dd05f4f8b6157cdbbccc0f9b0168284f6425df77c01e1a16db5c79cae406a513` |
| incidence baseline | 11.1, recomputed once from CFI-G train normal flames (210 cases); `fd_gen6/incidence_baseline_cfig_train.json`, sha256 `9211db23…c51b` |

Binary: `target/release/ignis.exe`, sha256 `73232afc…bfb3`, the same file for the chain check and every Flame Design forward.

## Step 1: the chain on CFI-G train

The chain (prefix files, `ignis wsp forward`, the IPCW integrated Brier of `transfer_acme.py`) against `wsp-dev feedback --partition train` with the same four releases and the 14 development horizons:

| Release | Cases | Extinction | Operator termination | Registered (12.6) | Chain minus `wsp-dev` |
|---|---|---|---|---|---|
| carry | 210 | 0.051752 | 0.027399 | 0.0518 / 0.0274 | below 1e-17 |
| P2 | 210 | 0.052579 | 0.028060 | 0.0526 / 0.0281 | below 1e-17 |
| P3 | 210 | 0.052162 | 0.023997 | 0.0522 / 0.0240 | below 1e-17 |
| baseline set | 210 | 0.150686 | 0.068103 | n/a | below 1e-17 |

- 81 runs, 210 cases, 33 not at risk, as `wsp-dev` counts them. Per case, the event table at 120 s and the endpoint match exactly (0 mismatches).
- **Reproduced.** Every registered number agrees to the four decimals it is printed with.
- Supplementary, not registered: the planned-stop path on train. I took the stage sum from each test's "as run" file (`acme_script.parse`, 210 of 210 cases carry it) and passed it as `programmed_stop`. The chain gives carry 0.0536 / 0.0215, P2 0.0542 / 0.0166 and P3 0.0538 / 0.0140, which are the 12.6 train numbers with the stop.

## Step 2: endpoints and the test set

- **Conditions.** `flame_design.py conditions` was run unchanged on the re-downloaded tables. It gives 132 normal tests in 13 sessions (17 propane), as in 11.1.
- **Test days.** The 29 normal tests on the test days 19123, 19200 and 19206 were dropped before measurement. They were not measured or scored, and no file of theirs was opened.
- **Endpoints.** `flame_design.py endpoints` was run unchanged, thresholds included, on the remaining 103 tests (86 ethylene, 17 propane, 10 sessions).
  - All 103 have status ok and a resolved endpoint. Onset came from the radiometers and the flow change from the setpoint log in every test.
  - Sensor and script labels agree on 89 of the 99 tests that have a script account.
  - Every case has an exact event time, so the IPCW weights are all 1. The case mean of the per-case contributions is therefore the integrated Brier score.
- **Cases.** 308 in all (cutoffs 1 / 3 / 5 s: 103 / 103 / 102): 258 ethylene and 50 propane.
- **Planned stop.** 305 of the 308 cases carry `programmed_stop` on the with-stop path. One ethylene test has no constant-flow stage sum, so it runs without the stop, as registered.

**How the releases date a Flame Design test.** `transfer_acme.fd_prefix` writes `run_id = PSI-10:acme_normal:<test>` and `session_id = PSI-10:session:<day>`.

- The session id holds no `YYYY-MM-DD`, so `committee2.rs` reads the run id instead. The first five characters of the test name are NASA's `YYDDD` day stamp: 19115A1 is day 115 of 2019, and 21301A1 is 28 October 2021.
- Every forward reports `test_date_source = run_id`. P2 and P3 give p_late = 0 (early era) to the 2019 tests and p_late = 1 to the propane session. The carry and the baseline set give 0.5 everywhere; their weights do not depend on the era.
- On 20 tests the sensor-file header is one day later than the stamp, which looks like a session that ran past midnight GMT. Both dates fall in the same era on every test, so this cannot change a forecast.

## Step 3 and 4: scores

Integrated Brier score over 6–120 s, as case means (the quoted figure) and as means of session means (in brackets). Columns are extinction / operator termination / sum. No case of a scored block was refused by any release.

### Decision set: arm B (ethylene with ethane's indicators) plus the propane session, 103 tests, 10 sessions, 308 cases

| Release | Without planned stop | With planned stop |
|---|---|---|
| carry | 0.1092 / 0.3465 / 0.4557 (0.1143 / 0.3110 / 0.4253) | 0.0703 / 0.0902 / 0.1605 (0.0770 / 0.0875 / 0.1645) |
| P2 | 0.1091 / 0.3403 / 0.4494 (0.1142 / 0.3079 / 0.4221) | 0.0713 / 0.0822 / 0.1535 (0.0781 / 0.0811 / 0.1592) |
| P3 | 0.1074 / 0.3260 / 0.4334 (0.1127 / 0.2973 / 0.4100) | 0.0707 / 0.0827 / 0.1534 (0.0776 / 0.0813 / 0.1589) |
| baseline set | 0.1518 / 0.3607 / 0.5125 (0.1517 / 0.3233 / 0.4750) | same as without (it does not read the stop) |
| incidence baseline | 0.2511 / 0.3736 / 0.6247 (0.2454 / 0.3344 / 0.5798) | same |

### Arm B alone: ethylene, ethane's indicators, 86 tests, 9 sessions, 258 cases

| Release | Without planned stop | With planned stop (255 of 258 cases carry it) |
|---|---|---|
| carry | 0.1203 / 0.3166 / 0.4369 (0.1212 / 0.2899 / 0.4111) | 0.0823 / 0.0887 / 0.1710 (0.0846 / 0.0864 / 0.1710) |
| P2 | 0.1204 / 0.3184 / 0.4388 (0.1213 / 0.2917 / 0.4130) | 0.0834 / 0.0798 / 0.1632 (0.0858 / 0.0796 / 0.1654) |
| P3 | 0.1190 / 0.3138 / 0.4329 (0.1200 / 0.2871 / 0.4071) | 0.0827 / 0.0784 / 0.1611 (0.0852 / 0.0787 / 0.1639) |
| baseline set | 0.1591 / 0.3232 / 0.4823 (0.1558 / 0.2977 / 0.4535) | same |
| incidence baseline | 0.2473 / 0.3336 / 0.5809 (0.2426 / 0.3071 / 0.5497) | same |

### Arm A: unchanged models

- **Propane session 21301** (17 tests, 1 session, 50 cases), extinction / operator termination / sum. One session gives no interval, and no conclusion is drawn, as registered:

| Release | Without planned stop | With planned stop (50 of 50) |
|---|---|---|
| carry | 0.0522 / 0.5007 / 0.5528 | 0.0081 / 0.0980 / 0.1061 |
| P2 | 0.0506 / 0.4538 / 0.5044 | 0.0091 / 0.0945 / 0.1035 |
| P3 | 0.0474 / 0.3889 / 0.4363 | 0.0090 / 0.1046 / 0.1136 |
| baseline set | 0.1145 / 0.5541 / 0.6687 | same |
| incidence baseline | 0.2704 / 0.5802 / 0.8506 | same |

- **Ethylene as recorded:** all four releases refuse all 258 cases (86 tests, 9 sessions) on both paths, at the condition gate. The reason is "fuel_is_ethane missing and fuel_species is not ethane, propane or butane", and the same for butane.

### Paired differences to the carry (candidate minus carry; 95 % intervals from 10,000 session resamples, seed 20261114)

The first figure is the pooled case mean, the second the mean of session means.

| Set, path | Candidate | Extinction | Operator termination | Sum |
|---|---|---|---|---|
| decision set, no stop | P2 | −0.0002 [−0.0006, +0.0002]; −0.0001 [−0.0004, +0.0002] | −0.0061 [−0.0200, +0.0027]; −0.0031 [−0.0133, +0.0027] | −0.0063 [−0.0207, +0.0028]; −0.0031 [−0.0137, +0.0029] |
| decision set, no stop | P3 | −0.0018 [−0.0030, −0.0006]; −0.0016 [−0.0027, −0.0005] | −0.0205 [−0.0517, +0.0029]; −0.0137 [−0.0382, +0.0033] | −0.0223 [−0.0545, +0.0019]; −0.0153 [−0.0407, +0.0026] |
| decision set, stop | P2 | +0.0011 [+0.0006, +0.0017]; +0.0011 [+0.0005, +0.0018] | −0.0080 [−0.0147, −0.0019]; −0.0064 [−0.0130, −0.0010] | −0.0070 [−0.0134, −0.0008]; −0.0053 [−0.0118, +0.0002] |
| decision set, stop | P3 | +0.0004 [−0.0007, +0.0014]; +0.0006 [−0.0004, +0.0015] | −0.0075 [−0.0161, +0.0008]; −0.0063 [−0.0140, +0.0006] | −0.0071 [−0.0158, +0.0017]; −0.0056 [−0.0135, +0.0015] |
| arm B, no stop | P2 | +0.0001 [+0.0000, +0.0002]; +0.0001 [+0.0000, +0.0002] | +0.0017 [+0.0006, +0.0031]; +0.0018 [+0.0006, +0.0032] | +0.0019 [+0.0006, +0.0033]; +0.0019 [+0.0006, +0.0034] |
| arm B, no stop | P3 | −0.0013 [−0.0022, −0.0004]; −0.0012 [−0.0022, −0.0004] | −0.0028 [−0.0099, +0.0053]; −0.0028 [−0.0107, +0.0060] | −0.0040 [−0.0119, +0.0046]; −0.0040 [−0.0127, +0.0054] |
| arm B, stop | P2 | +0.0011 [+0.0005, +0.0018]; +0.0011 [+0.0005, +0.0019] | −0.0089 [−0.0161, −0.0014]; −0.0067 [−0.0140, −0.0007] | −0.0078 [−0.0148, −0.0004]; −0.0056 [−0.0127, +0.0005] |
| arm B, stop | P3 | +0.0004 [−0.0009, +0.0015]; +0.0006 [−0.0005, +0.0016] | −0.0102 [−0.0179, −0.0017]; −0.0077 [−0.0156, −0.0006] | −0.0099 [−0.0178, −0.0009]; −0.0071 [−0.0153, +0.0002] |
| propane, no stop | P2 / P3 | −0.0016 / −0.0047 | −0.0468 / −0.1118 | −0.0484 / −0.1165 (one session) |
| propane, stop | P2 / P3 | +0.0009 / +0.0009 | −0.0035 / +0.0066 | −0.0026 / +0.0075 (one session) |

### By the days frozen as train and as validation (docs/wsp/atlas/splits/PSI-10.json)

**Train days:** 74 ethylene tests in 8 sessions, 222 cases; there is no propane test on a train day. So the decision set and arm B are the same here.

| Release | Without planned stop | With planned stop (219 of 222) |
|---|---|---|
| carry | 0.1345 / 0.2998 / 0.4343 (0.1323 / 0.2736 / 0.4058) | 0.0944 / 0.0929 / 0.1873 (0.0942 / 0.0894 / 0.1836) |
| P2 | 0.1346 / 0.3011 / 0.4357 (0.1324 / 0.2750 / 0.4074) | 0.0956 / 0.0845 / 0.1800 (0.0954 / 0.0833 / 0.1787) |
| P3 | 0.1333 / 0.2988 / 0.4321 (0.1311 / 0.2722 / 0.4033) | 0.0948 / 0.0832 / 0.1780 (0.0949 / 0.0824 / 0.1773) |
| baseline set | 0.1717 / 0.3015 / 0.4731 | same |
| incidence baseline | 0.2436 / 0.3114 / 0.5551 | same |

Paired differences to the carry, train days, pooled case mean:

| Path | Candidate | Extinction | Operator termination | Sum |
|---|---|---|---|---|
| no stop | P2 | +0.0001 [+0.0000, +0.0002] | +0.0013 [+0.0003, +0.0027] | +0.0014 [+0.0003, +0.0029] |
| no stop | P3 | −0.0012 [−0.0024, −0.0002] | −0.0010 [−0.0087, +0.0073] | −0.0022 [−0.0107, +0.0068] |
| stop | P2 | +0.0012 [+0.0005, +0.0019] | −0.0084 [−0.0163, +0.0002] | −0.0073 [−0.0151, +0.0012] |
| stop | P3 | +0.0004 [−0.0011, +0.0016] | −0.0097 [−0.0185, +0.0004] | −0.0093 [−0.0184, +0.0014] |

**Validation days, decision set:** 29 tests in 2 sessions (12 ethylene on 19196, 17 propane on 21301), 86 cases. Day 19142 has no included test.

| Release | Without planned stop | With planned stop (86 of 86) |
|---|---|---|
| carry | 0.0440 / 0.4671 / 0.5111 (0.0424 / 0.4606 / 0.5030) | 0.0081 / 0.0831 / 0.0912 (0.0081 / 0.0802 / 0.0883) |
| P2 | 0.0431 / 0.4417 / 0.4848 (0.0417 / 0.4393 / 0.4810) | 0.0089 / 0.0761 / 0.0850 (0.0088 / 0.0726 / 0.0814) |
| P3 | 0.0406 / 0.3962 / 0.4369 (0.0393 / 0.3977 / 0.4370) | 0.0087 / 0.0812 / 0.0899 (0.0086 / 0.0767 / 0.0853) |
| baseline set | 0.1006 / 0.5137 / 0.6143 | same |
| incidence baseline | 0.2703 / 0.5340 / 0.8043 | same |

- **Arm B validation days** (19196 only: 12 tests, 36 cases). Without the stop: carry 0.0326 / 0.4205 / 0.4531, P2 0.0328 / 0.4248 / 0.4576, P3 0.0311 / 0.4065 / 0.4376. With it: carry 0.0080 / 0.0624 / 0.0704, P2 0.0086 / 0.0507 / 0.0593, P3 0.0082 / 0.0488 / 0.0570.
- **Intervals.** Every validation-day case ended in operator termination. With two sessions, the intervals amount to the two sessions' values and are not informative.

## Decision inputs for PROTOCOL 12.6

The Flame Design part of the decision score is the case mean on the set 11.3 scored: arm B plus the propane session, 103 tests, 10 sessions, 308 cases. Columns are extinction / operator termination / sum.

| Release | Without planned stop | With planned stop |
|---|---|---|
| carry | 0.1092 / 0.3465 / 0.4557 | 0.0703 / 0.0902 / 0.1605 |
| P2 | 0.1091 / 0.3403 / 0.4494 | 0.0713 / 0.0822 / 0.1535 |
| P3 | 0.1074 / 0.3260 / 0.4334 | 0.0707 / 0.0827 / 0.1534 |

The rule of 12.6 averages these with the CFI-G validation numbers of part (a), which this agent did not compute, so no verdict is given here.

## Does the carry reproduce the P1 numbers of 14 November?

Yes, on every set reported in 11.3, to the four printed decimals:

| Set | P1, 14 November (11.3) | carry, today |
|---|---|---|
| arm B, normal, no stop | 0.1203 / 0.3166 | 0.1203 / 0.3166 |
| arm B, normal, stop | 0.0823 / 0.0887 | 0.0823 / 0.0887 |
| arm B, train days / validation days, no stop | 0.1345 / 0.2998; 0.0326 / 0.4205 | 0.1345 / 0.2998; 0.0326 / 0.4205 |
| propane 21301, no stop | 0.0522 / 0.5007 | 0.0522 / 0.5007 |
| propane 21301, stop | 0.0081 / 0.0980 | 0.0081 / 0.0980 |

- **Baselines.** The old reference set (`sha256:6044b69b…`) and today's baseline set give the same figures on these tests (arm B 0.1591 / 0.3232, propane 0.1145 / 0.5541). The incidence baseline gives the same figures as in 11.3 (0.2473 / 0.3336 and 0.2704 / 0.5802).
- **Counts.** Cases, tests, sessions, refusals and the mix of endpoints (84 of 258 ethylene cases end in extinction) are the same.
- **What this implies.** The re-downloaded raw files gave the same endpoints in aggregate. The carry on the wsp7 spec answers these tests as `acme_normal.wsp5` P1 did. No difference needs explaining.
- **Not checked.** The 14 November working files are deleted, so per-case equality and the old file hashes could not be compared.

## What was adapted, and how to rerun

- **New file:** `tools/wsp/ext/transfer_gen6.py`. `flame_design.py` and `transfer_acme.py` are unchanged; the new file imports them and changes only the following:
  - read paths: `raw/PSI-10/Raw Data/` and `raw/PSI-10/Experimental table/` instead of `raw/samples/PSI-10/`;
  - output folder: `work/transfer_acme/fd_gen6/`;
  - the four releases and the comparisons to the carry, with the sum added;
  - the locator paths inside the prefixes, which the forward does not read;
  - normal flames only;
  - the binary: `target/release`, because the 14 November copy in `runs/dev/bin` predates wsp7.
- **No sensitivity runs.** The propane and n-butane indicator runs of 11.1 were not repeated, because 12.6 names ethane only.

```bash
T=6,8,10,12,15,20,25,30,40,50,60,80,100,120
target/release/wsp-dev.exe feedback --release sha256:2a898731e18dcefefff39146810fb4736392fe24aab02b40079202bc316b0963 \
  --release sha256:ec1aa02e991f8e3feec469edbb374c54dd135faa1c58a4e8ce6bb7d03fae9ba0 \
  --release sha256:96a91f15ab94ff6a0b32e51f34e5c1f71d8377ae618b4cdd331bdb1b20fdaa34 \
  --release sha256:dd05f4f8b6157cdbbccc0f9b0168284f6425df77c01e1a16db5c79cae406a513 \
  --partition train --times $T --out runs/dev/data_all/work/transfer_acme/fd_gen6/cfig/wspdev_fb_train_normal.json
python tools/wsp/ext/transfer_gen6.py chain --with-stop
python tools/wsp/ext/transfer_gen6.py conditions
python tools/wsp/ext/transfer_gen6.py tests-file
python tools/wsp/ext/transfer_gen6.py endpoints
python tools/wsp/ext/transfer_gen6.py score
python tools/wsp/ext/transfer_gen6.py tables
```

- **Outputs.** `fd_gen6/report.json` and `fd_gen6/report_tables.txt` hold aggregates; `fd_gen6/cases/*.csv` and `fd_gen6/flame_design/endpoints.csv` hold per-test outcomes.
- **Input hashes.** `conditions.csv` sha256 `28485d7b…7b2f`, `endpoints.csv` sha256 `dcecef6c…24d7`.

## Not verified

- **Labels.** The Flame Design labels were not adjudicated: sensor and script disagree on 10 of 99 tests.
- **Threats.** The threats listed in 11.1 and 12.6 still stand:
  - the burner limit per session is not known, and the committee treats every 2019 test as early era;
  - the planned-stop offsets were typed on CFI-G;
  - the table Tad is used;
  - ethylene runs under ethane's indicators.
- **Validation days.** They are two sessions, one of them propane. Their intervals carry little information.

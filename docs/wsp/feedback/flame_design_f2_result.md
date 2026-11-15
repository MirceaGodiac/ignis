# Flame Design validation days: F2 against F1, part (b) of the look of PROTOCOL 12.10

> Written by a scoring agent (claude-opus-5-5) on 15 November 2026. It runs part (b) of the look registered in
> PROTOCOL 12.10, once. Nothing was fitted, and no model or threshold changed.
> **Aggregates only.** Per-test rows are in `runs/dev/data_all/work/transfer_acme/fd_f2/` (local).
> The test days (normal 19123, 19200, 19206; inverse 21361) were not read.
> Part (a), CFI-G validation, was not run here, so the rule of 12.10 is not applied in this file.

## Releases, inputs, chain

| Family | F2 | F1 (as in 12.8) | Ethylene as ethane |
|---|---|---|---|
| normal | `acme_normal.wsp12` F2 `sha256:2ae770d8…bc52` | `sha256:4d0acb8e…4b8b` | P3 `sha256:51f4bbc0…3de9` with ethane's indicators |
| inverse | `acme_inverse.wsp12` F2 `sha256:7404f3ad…a7f7` | `sha256:d59251cf…3c42` | P1 `sha256:aa05ddcb…ea0e` with ethane's indicators |

**Binary.** `target-merged/release/ignis.exe` (sha256 `b03a35c8…`, in `fd_f2/report.json`), for the chain check and every forward.

**Inputs, as registered:**
- **Fuel.** Ethylene goes to F2 and F1 as itself (`fuel_species`, no indicators). The propane session 21301 goes to every release as propane.
- **Without the planned stop,** all releases read the same prefix.
- **With the planned stop,** F2 reads the planned stop of `tools/wsp/ext/acme_script2.py`, and F1 and the surrogate read the old stage sum.
  - Branches of the F2 stop on these tests: normal v1|no_fiber_move 6, v1|fiber_move_wait 5, v2|fiber_traverse 14, v2|no_fiber_traverse 3; inverse v2|fiber_traverse 17, v2|no_fiber_traverse 8.
  - One normal test has an incomplete file and gets the stage sum, the registered fallback.
  - Every case carries a planned stop on every release.

**Endpoints and baseline.** Endpoints are those of the 12.8 run (`flame_design.py`, thresholds unchanged), reused. The incidence baseline of 11.1 is the 12.8 file, sha256 `c526a6f1…908b`.

**Chain check, CFI-G TRAIN, no planned stop.**

| Release | Cases | Extinction | Operator termination | Required |
|---|---|---|---|---|
| F2 normal | 210 | 0.052162 | 0.023997 | 0.052162 / 0.023997 |
| F2 inverse | 204 | 0.027091 | 0.060816 | 0.027091 / 0.060816 |

- F2 equals `wsp-dev feedback` to below 1e-16, with the same counts (81 / 210 / 33 and 73 / 204 / 15). F1 gives the same numbers.
- **Reproduced.**

## Tests and refusals

| Family | Validation days with a test | Tests | Cases (all endpoints) | Refusals |
|---|---|---|---|---|
| normal | 19196 (12 ethylene), 21301 (17 propane); 19142 has no included test | 29 | 86, all operator termination | none, by any release, on either path |
| inverse | 21340, 22018 | 25 | 75 (24 extinction, 51 operator termination) | none |

No case is answered by only one of F2 and F1.

## Scores

Integrated Brier score over 6–120 s. Columns are extinction / operator termination / sum, as case means (means of session means in brackets). All endpoints are exact events, so the IPCW weights are 1.

**Intervals.** The paired differences are F2 minus the other release, with percentile intervals from 10,000 session resamples (seed 20261114).
- **With two sessions** there are only three distinct resamples, so the interval amounts to the two sessions' own values and is not a stable interval. Read it as the range of the two sessions, not as an uncertainty.
- **With one session** there is no interval at all.

### Normal flames, all validation-day tests (86 cases, 29 tests, 2 sessions)

| Release | Without planned stop | With planned stop |
|---|---|---|
| F2 | 0.0317 / 0.3933 / 0.4250 (0.0286 / 0.3942 / 0.4228) | 0.0068 / 0.0724 / 0.0792 (0.0062 / 0.0664 / 0.0726) |
| F1 | 0.0558 / 0.4028 / 0.4586 (0.0574 / 0.4055 / 0.4629) | 0.0141 / 0.0826 / 0.0966 (0.0151 / 0.0783 / 0.0933) |
| ethylene as ethane (P3) | 0.0406 / 0.3962 / 0.4369 (0.0393 / 0.3977 / 0.4370) | 0.0087 / 0.0812 / 0.0899 (0.0086 / 0.0767 / 0.0853) |
| incidence baseline | 0.2703 / 0.5340 / 0.8043 (0.2703 / 0.5250 / 0.7953) | same |

| Path | F2 − F1, extinction / termination / sum | F2 − surrogate, extinction / termination / sum |
|---|---|---|
| no stop | −0.0241 [−0.0576, +0.0000] / −0.0095 [−0.0227, +0.0000] / −0.0336 [−0.0803, +0.0000] | −0.0090 [−0.0214, +0.0000] / −0.0029 [−0.0070, +0.0000] / −0.0119 [−0.0284, +0.0000] |
| stop | −0.0073 [−0.0187, +0.0009] / −0.0101 [−0.0224, −0.0013] / −0.0174 [−0.0410, −0.0004] | −0.0019 [−0.0057, +0.0009] / −0.0088 [−0.0191, −0.0013] / −0.0106 [−0.0249, −0.0004] |

Means of session means for the same differences: no stop, F2 − F1 −0.0288 / −0.0114 / −0.0401; stop, −0.0089 / −0.0118 / −0.0207.

**Normal, ethylene tests only** (19196: 36 cases, 12 tests, 1 session; all ended by operator termination):

| Release | Without planned stop | With planned stop |
|---|---|---|
| F2 | 0.0097 / 0.3995 / 0.4092 | 0.0025 / 0.0296 / 0.0321 |
| F1 | 0.0673 / 0.4222 / 0.4895 | 0.0211 / 0.0520 / 0.0731 |
| ethylene as ethane | 0.0311 / 0.4065 / 0.4376 | 0.0082 / 0.0488 / 0.0570 |
| incidence baseline | 0.2702 / 0.4699 / 0.7401 | same |

F2 − F1: −0.0576 / −0.0227 / −0.0803 without the stop, −0.0187 / −0.0224 / −0.0410 with it (one session, no interval).

**Normal, propane session 21301** (50 cases, 17 tests, 1 session; all ended by operator termination):

| Release | Without planned stop | With planned stop |
|---|---|---|
| F2 | 0.0474 / 0.3889 / 0.4363 | 0.0099 / 0.1033 / 0.1132 |
| F1 = P3 | 0.0474 / 0.3889 / 0.4363 | 0.0090 / 0.1046 / 0.1136 |
| incidence baseline | 0.2704 / 0.5802 / 0.8506 | same |

- **Without the stop,** F2 equals F1 on propane, since the alkane rows are unchanged.
- **With the stop,** only the planned-stop input differs: F2 − F1 is +0.0009 / −0.0013 / −0.0004 (one session).

### Inverse flames (75 cases, 25 tests, 2 sessions)

| Release | Without planned stop | With planned stop |
|---|---|---|
| F2 | 0.0871 / 0.1959 / 0.2831 (0.0886 / 0.1935 / 0.2820) | 0.0920 / 0.0018 / 0.0938 (0.0935 / 0.0017 / 0.0952) |
| F1 | 0.0871 / 0.1959 / 0.2831 (0.0886 / 0.1935 / 0.2820) | 0.0921 / 0.0240 / 0.1160 (0.0935 / 0.0236 / 0.1172) |
| ethylene as ethane (P1) | 0.1150 / 0.1783 / 0.2933 (0.1172 / 0.1758 / 0.2930) | 0.1243 / 0.0233 / 0.1476 (0.1266 / 0.0230 / 0.1496) |
| incidence baseline | 0.2442 / 0.3168 / 0.5610 (0.2440 / 0.3137 / 0.5577) | same |

| Path | F2 − F1, extinction / termination / sum | F2 − surrogate, extinction / termination / sum |
|---|---|---|
| no stop | 0 / 0 / 0 (identical forecasts) | −0.0279 [−0.0470, −0.0103] / +0.0177 [+0.0174, +0.0179] / −0.0102 [−0.0295, +0.0077] |
| stop | −0.0000 [−0.0001, +0.0000] / −0.0222 [−0.0298, −0.0140] / −0.0222 [−0.0299, −0.0140] | −0.0323 [−0.0539, −0.0123] / −0.0216 [−0.0290, −0.0135] / −0.0538 [−0.0674, −0.0413] |

**F2 against F1 for inverse flames.**
- Without the stop, inverse F2 is F1 on these tests: the inverse fuel table and the laws are unchanged.
- With the stop, the difference comes from the planned-stop input and its three readings.

**Consistency with 12.8.** F1 reproduces its 12.8 validation-day numbers exactly: normal ethylene 0.0673 / 0.4222 and 0.0211 / 0.0520; inverse 0.0871 / 0.1959 and 0.0921 / 0.0240. The surrogate reproduces them as well.

## Decision inputs for 12.10

Flame Design validation days, on the cases both F2 and F1 answer (all cases here), case means. Columns are extinction / operator termination / sum.

| Family | Path | Cases both answer | Only one answers | F2 | F1 |
|---|---|---|---|---|---|
| normal | without planned stop | 86 | 0 | 0.0317 / 0.3933 / 0.4250 | 0.0558 / 0.4028 / 0.4586 |
| normal | with planned stop | 86 | 0 | 0.0068 / 0.0724 / 0.0792 | 0.0141 / 0.0826 / 0.0966 |
| inverse | without planned stop | 75 | 0 | 0.0871 / 0.1959 / 0.2831 | 0.0871 / 0.1959 / 0.2831 |
| inverse | with planned stop | 75 | 0 | 0.0920 / 0.0018 / 0.0938 | 0.0921 / 0.0240 / 0.1160 |

- **Not applied here.** The rule of 12.10 averages these with part (a) on CFI-G validation, which this agent did not compute.
- **For the record (outside the rule):** ethylene as ethane gives 0.0406 / 0.3962 / 0.4369 and 0.0087 / 0.0812 / 0.0899 (normal), and 0.1150 / 0.1783 / 0.2933 and 0.1243 / 0.0233 / 0.1476 (inverse).

## Reproduce

```bash
T=6,8,10,12,15,20,25,30,40,50,60,80,100,120; W=target-merged/release/wsp-dev.exe; O=runs/dev/data_all/work/transfer_acme/fd_f2/cfig
$W feedback --release sha256:2ae770d8d159bca359e2125ef348ce13724abce49e91af4a3d62c5c66af6bc52 --release sha256:4d0acb8e517871879fa0dea5230c7b75c93232466f8b101a791e752a1deb4b8b --partition train --times $T --out $O/wspdev_fb_train_normal.json
$W feedback --release sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7 --release sha256:e2fb1fccf8c688f4ac357db7ad0abdc283dfed894a6a3f66f23e2b75ca1b2b2f --partition train --times $T --out $O/wspdev_fb_train_inverse.json
python tools/wsp/ext/transfer_f2.py chain --ignis target-merged/release/ignis.exe
python tools/wsp/ext/transfer_f2.py score --ignis target-merged/release/ignis.exe
python tools/wsp/ext/transfer_f2.py tables
```

## Not verified

- **Validation-day endings.** Every normal validation-day case ended by operator termination. The normal extinction scores here therefore measure only how little extinction each release forecasts for flames that were held.
- **Threats.** The threats of 12.10 stand: few validation days, the hinge shift read from the train days, and endpoints inside ignition-to-test ramps.

# Flame Design (PSI-10): ethylene through the fuel table (F1) against ethylene as ethane, PROTOCOL 12.8

> Written by a scoring agent (claude-opus-5-5) on 15 November 2026. It runs the test registered in PROTOCOL 12.8
>. Nothing was fitted, and no model or threshold changed. **Aggregates only.** Per-test rows are
> in `runs/dev/data_all/work/transfer_acme/fd_fuel/` (local, not in git). The four test days were not measured,
> scored or opened.

**Outcome on the registered path (without the planned stop):**
- Normal flames: the fuel table **loses something**.
- Inverse flames: the fuel table **adds something**.

## Releases and chain

| Family | As itself (fuel as recorded) | As ethane (ethane's indicators) |
|---|---|---|
| normal | `acme_normal.wsp12` F1 `sha256:131060f2…7ffdf` | `acme_normal.wsp7` P3 `sha256:96a91f15…a34` |
| inverse | `acme_inverse.wsp12` F1 `sha256:e2fb1fcc…2b2f` | `acme_inverse.wsp5` P1 `sha256:d7292e89…1fcc2` |

- **Incidence baseline.** I recomputed the 11.1 baseline once from CFI-G train, both families. Its sha256 is `c526a6f1…908b`, the hash of the file frozen on 14 November, so it is byte-identical to it.
- **Binary.** `target/release/ignis.exe` (rebuilt), sha256 `f3139eea…9c68`, used for every forward.

**Chain check, CFI-G TRAIN, no planned stop:**

| Release | Cases | Extinction | Operator termination | Required |
|---|---|---|---|---|
| normal F1 | 210 | 0.052162 | 0.023997 | 0.052162 / 0.023997 |
| inverse F1 | 204 | 0.027091 | 0.060816 | 0.027091 / 0.060816 |

- **Against `wsp-dev feedback`.** The chain equals `wsp-dev feedback` to below 1e-17 for F1 and for both surrogates, with the same runs, cases and not-at-risk counts (81 / 210 / 33 and 73 / 204 / 15).
- **F1 against the adopted release.** On CFI-G, F1's forecast table equals the adopted release's bit for bit on 210 of 210 and 204 of 204 cases.
- **Reproduced.**

## How the fuel reaches each release, and the cases

- **As itself.** The prefix carries `fuel_species` = "ethylene" and no `fuel_is_*` condition; the script asserts this for every F1 prefix.
- **As ethane.** It is the prefix of 11.1 arm B: the same text plus `fuel_is_ethane = 1` and `fuel_is_butane = 0`.
- **Everything else is identical,** including conditions, cutoffs, endpoints and the planned stop. Run ids carry the date as in the gen6 run.
- **Normal endpoints.** I reused those of the gen6 run, the same file byte for byte.
- **Inverse endpoints.** I measured them by `flame_design.py` with thresholds unchanged, on the 80 included tests outside test day 21361:
  - all have status ok and a resolved endpoint;
  - sensor and script labels agree on 69 of 77;
  - together with the normal flames, 18 of 176 disagree, as in 11.3.

| Family | Ethylene tests | Sessions | Cases (cutoffs 1 / 3 / 5 s) | Train days | Validation days |
|---|---|---|---|---|---|
| normal | 86 | 9 | 258 (86 / 86 / 86) | 74 tests, 8 sessions | 12 tests, 1 session (19196) |
| inverse | 80 | 7 | 236 (80 / 80 / 76) | 55 tests, 5 sessions | 25 tests, 2 sessions (21340, 22018) |

**Refusals.**
- **Normal F1 refuses 18 cases (6 tests, sessions 19156, 19182 and 19189, all train days), on both paths.** The reason is that the table adiabatic temperature is below the 1900 K bound of ethylene (1683 to 1829 K), in the release's words "the weak-flame law is an alkane law". The lowest temperature F1 answers is 2003.5 K.
- **Inverse F1 refuses nothing** (the lowest temperature is 2074 K).
- **The surrogates refuse nothing.**

## Scores

Integrated Brier score over 6–120 s on the ethylene cases F1 answers. Columns are extinction / operator termination / sum, as case means (means of session means in brackets). All endpoints are exact events, so the IPCW weights are 1.

### Normal flames: 240 cases, 80 tests, 9 sessions (84 extinction, 156 operator termination)

| Path | F1 (as itself) | P3 as ethane | Incidence baseline |
|---|---|---|---|
| no stop | 0.1853 / 0.3103 / 0.4955 (0.1826 / 0.2784 / 0.4609) | 0.1128 / 0.2958 / 0.4086 (0.1137 / 0.2657 / 0.3794) | 0.2456 / 0.3171 / 0.5627 (0.2390 / 0.2867 / 0.5257) |
| stop (240 of 240 carry it) | 0.1246 / 0.0749 / 0.1996 (0.1298 / 0.0759 / 0.2057) | 0.0768 / 0.0626 / 0.1394 (0.0808 / 0.0644 / 0.1452) | same as no stop |

F1 minus surrogate, with 95 % intervals from 10,000 session resamples (seed 20261114). The pooled case mean comes first, the mean of session means second:

| Path | Extinction | Operator termination | Sum |
|---|---|---|---|
| no stop | +0.0725 [+0.0402, +0.1022]; +0.0689 [+0.0412, +0.0962] | +0.0144 [+0.0075, +0.0199]; +0.0127 [+0.0069, +0.0184] | +0.0869 [+0.0491, +0.1216]; +0.0815 [+0.0491, +0.1135] |
| stop | +0.0479 [+0.0250, +0.0709]; +0.0490 [+0.0265, +0.0713] | +0.0123 [+0.0039, +0.0204]; +0.0115 [+0.0036, +0.0200] | +0.0601 [+0.0310, +0.0884]; +0.0605 [+0.0320, +0.0885] |

F1 minus incidence baseline, extinction: no stop −0.0603 [−0.1403, +0.0188]; stop −0.1210 [−0.1904, −0.0451].

On the 18 cases F1 refuses (6 tests, 3 sessions, all operator termination), the surrogate scores 0.2021 / 0.5539 / 0.7560 without the stop and 0.1613 / 0.2890 / 0.4503 with it. The incidence baseline scores 0.2702 / 0.5530 / 0.8232 there.

### Inverse flames: 236 cases, 80 tests, 7 sessions (57 extinction, 179 operator termination)

| Path | F1 (as itself) | P1 as ethane | Incidence baseline |
|---|---|---|---|
| no stop | 0.1077 / 0.2579 / 0.3657 (0.0920 / 0.2544 / 0.3463) | 0.1466 / 0.2599 / 0.4065 (0.1251 / 0.2560 / 0.3812) | 0.2460 / 0.3413 / 0.5873 (0.2461 / 0.3342 / 0.5803) |
| stop (236 of 236) | 0.1202 / 0.1099 / 0.2301 (0.1027 / 0.1019 / 0.2046) | 0.1594 / 0.1162 / 0.2756 (0.1359 / 0.1063 / 0.2422) | same as no stop |

F1 minus surrogate:

| Path | Extinction | Operator termination | Sum |
|---|---|---|---|
| no stop | −0.0388 [−0.0658, −0.0101]; −0.0332 [−0.0608, −0.0094] | −0.0020 [−0.0227, +0.0133]; −0.0016 [−0.0197, +0.0115] | −0.0408 [−0.0855, +0.0011]; −0.0348 [−0.0759, −0.0008] |
| stop | −0.0391 [−0.0682, −0.0110]; −0.0332 [−0.0603, −0.0103] | −0.0063 [−0.0186, +0.0024]; −0.0045 [−0.0154, +0.0031] | −0.0455 [−0.0859, −0.0094]; −0.0376 [−0.0747, −0.0072] |

F1 minus incidence baseline, extinction: no stop −0.1382 [−0.2190, −0.0551]; stop −0.1257 [−0.2175, −0.0257].

### By the days frozen as train and as validation (docs/wsp/atlas/splits/PSI-10.json)

| Family, days, path | Cases (tests, sessions) | F1 | Surrogate | Incidence | F1 − surrogate, extinction |
|---|---|---|---|---|---|
| normal, train, no stop | 204 (68, 8) | 0.2061 / 0.2905 / 0.4966 | 0.1272 / 0.2763 / 0.4035 | 0.2413 / 0.2901 / 0.5314 | +0.0789 [+0.0407, +0.1069] |
| normal, train, stop | 204 (68, 8) | 0.1429 / 0.0790 / 0.2219 | 0.0889 / 0.0651 / 0.1540 | same | +0.0540 [+0.0296, +0.0755] |
| normal, validation, no stop | 36 (12, 1) | 0.0673 / 0.4222 / 0.4895 | 0.0311 / 0.4065 / 0.4376 | 0.2702 / 0.4699 / 0.7401 | +0.0362 (one session) |
| normal, validation, stop | 36 (12, 1) | 0.0211 / 0.0520 / 0.0731 | 0.0082 / 0.0488 / 0.0570 | same | +0.0129 (one session) |
| inverse, train, no stop | 161 (55, 5) | 0.1173 / 0.2868 / 0.4042 | 0.1612 / 0.2980 / 0.4592 | 0.2468 / 0.3528 / 0.5996 | −0.0439 [−0.0780, −0.0007] |
| inverse, train, stop | 161 (55, 5) | 0.1333 / 0.1499 / 0.2833 | 0.1757 / 0.1595 / 0.3352 | same | −0.0423 [−0.0785, −0.0008] |
| inverse, validation, no stop | 75 (25, 2) | 0.0871 / 0.1959 / 0.2831 | 0.1150 / 0.1783 / 0.2933 | 0.2442 / 0.3168 / 0.5610 | −0.0279 [−0.0470, −0.0103] (two sessions) |
| inverse, validation, stop | 75 (25, 2) | 0.0921 / 0.0240 / 0.1160 | 0.1243 / 0.0233 / 0.1476 | same | −0.0322 [−0.0539, −0.0122] (two sessions) |

- **Where the refusals fall.** All 18 normal refusals are on train days.
- **Validation days.** Every normal validation-day case ended by operator termination. Two-session intervals carry little information.

## Verdict inputs for PROTOCOL 12.8

The rule is applied per family on the path without the planned stop: extinction, case mean, F1 minus surrogate, session bootstrap.

| Family | Cases F1 answers (tests, sessions) | F1 refuses | F1 extinction | Surrogate extinction | F1 − surrogate [95 %] | By the rule |
|---|---|---|---|---|---|---|
| normal | 240 (80, 9) | 18 cases, 6 tests (T_ad < 1900 K) | 0.1853 | 0.1128 | +0.0725 [+0.0402, +0.1022] | **loses something** |
| inverse | 236 (80, 7) | 0 | 0.1077 | 0.1466 | −0.0388 [−0.0658, −0.0101] | **adds something** |

- **With the planned stop (not the registered path), the answers are the same.** Normal is +0.0479 [+0.0250, +0.0709]; inverse is −0.0391 [−0.0682, −0.0110].
- **Normal: next step.** By 12.8, F1 answers for ethylene from now on in both families. The normal F1 ethylene row is the first thing the 12.3 revision looks at.
- **Inverse: scope.** "Adds something" is measured against `acme_inverse.wsp5` P1 as ethane. 11.3 found that release undecided against the incidence baseline.

## Reproduce

```bash
T=6,8,10,12,15,20,25,30,40,50,60,80,100,120; O=runs/dev/data_all/work/transfer_acme/fd_fuel/cfig
target/release/wsp-dev.exe feedback --release sha256:4d0acb8e517871879fa0dea5230c7b75c93232466f8b101a791e752a1deb4b8b \
  --release sha256:51f4bbc01faf7ccf2c9070b8f62c213aaab442c195113a70919023bc0cc83de9 --partition train --times $T --out $O/wspdev_fb_train_normal.json
target/release/wsp-dev.exe feedback --release sha256:e2fb1fccf8c688f4ac357db7ad0abdc283dfed894a6a3f66f23e2b75ca1b2b2f \
  --release sha256:aa05ddcbe976dbb1a5537987d05270b28066551c39ab24db671be65692dcea0e --partition train --times $T --out $O/wspdev_fb_train_inverse.json
python tools/wsp/ext/transfer_fuel.py chain
python tools/wsp/ext/transfer_fuel.py endpoints
python tools/wsp/ext/transfer_fuel.py score
python tools/wsp/ext/transfer_fuel.py tables
python tools/wsp/ext/transfer_fuel.py train-days
```

- **New scripts.** `tools/wsp/ext/transfer_fuel.py` is a wrapper over the unchanged `flame_design.py`, `transfer_acme.py` and `transfer_gen6.py`; it changes only releases, fuel handling, comparisons and folders. `tools/wsp/ext/fd_train_anatomy.py` produces the train-day study.
- **Input hashes.** `conditions.csv` (copied from fd_gen6) and the merged `endpoints.csv` are hashed in `fd_fuel/report.json`.

## Not verified

- **Labels.** The Flame Design labels were not adjudicated.
- **Earlier threats.** The threats of 11.1, 12.6 and 12.8 stand: the table Tad, the burner limit per session, and the planned-stop offsets typed on CFI-G.
- **The 1900 K bound.** It acts on the table Tad, which 11.1 noted may carry after-test information.

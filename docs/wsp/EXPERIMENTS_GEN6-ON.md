# IGNIS hand-written models: experiments from the sixth generation onward (14 to 15 November 2026)

Generations 1 to 5 are in [EXPERIMENTS_GEN1-5.md](EXPERIMENTS_GEN1-5.md). This file picks up on the
evening of 14 November and runs to the last commit of 15 November.

**What this covers.** IGNIS forecasts NASA microgravity combustion experiments with hand-written models
(weight-space programming, WSP). An author model types every parameter of a release as a literal number,
with a physical reason. Nothing is fitted (`trained_on = not_trained`, `state = experimental`). Fable
(`claude-fable-5-2`) is the lead author. Astra (`gpt-6-astra`) is the second author and reviewer.

**Where the record lives.** The research repository is `D:\dev\SpaceApps-Ardealu`, branch
`feat/handwritten-science` (written "lane" below), and its last commit read for this file is `542dad8`
(15 Nov 13:29). The product repository is this one (`ignis-web`). Paths without a repository prefix are in
the lane under `docs/wsp/`. Commit hashes are lane commits unless marked *ignis-web*. Paths under `runs/`
are local working folders and are not in git. This file was built only from aggregate documents, commits
and the served list. Nothing under `holdout/`, the evaluator vaults or any per-case test file was opened.

**Conventions.**
- Every score is lower-is-better.
- Droplets: CRPS and MAE of the median, in µm.
- Burner flames: integrated Brier score (IBS) from 6 to 120 s, given as extinction / operator
  termination ("termination") / their sum, as case means.
- Fabric: tip CRPS in sample widths.
- Intervals are 95 % cluster bootstraps (over sessions, days or burns) unless stated otherwise.
- "Zero-shot" means typed before any outcome of that data was seen. "Data-informed" means revised after
  reading train groups.

---

## 1. Headline results

| What | Result | Source |
|---|---|---|
| **Sealed FLEX methanol test** (65 cases, 9 sessions), `flex_methanol.wsp2` D3 | CRPS **18.5 µm** vs 31.6 for the d-squared law, difference −13.1 [−19.0, −7.7]; lower than every fitted baseline (v3 −24.2 [−32.7, −15.9]) | RESULTS_FINAL.md, step 1 (`5686866`) |
| **Sealed FLEX n-heptane test** (43 cases, 7 sessions), `flex_heptane.wsp2` D2 | CRPS **32.2 µm** vs 54.0, difference −21.8 [−47.1, −4.9]; vs zero-shot W0 −47.0 [−124.9, −3.2] | RESULTS_FINAL.md, step 1 |
| **Sealed CFI-G normal-flame test** (60 cases, 2 sessions), `acme_normal.wsp12` F2 | **0.0414 / 0.0494** vs 0.0456 / 0.0558 for the 14 Nov model (extinction −0.0036 [−0.0056, −0.0016]); about one third of the best fitted baseline's error | RESULTS_FINAL.md, step 1 |
| **Sealed CFI-G inverse-flame test** (93 cases, 3 sessions), `acme_inverse.wsp12` F2 | **0.0998 / 0.1103**; vs hazard model −0.0553 [−0.1181, −0.0086] on extinction | RESULTS_FINAL.md, step 1 |
| **Flame Design test days**, normal ethylene flames (87 cases, 3 days) | F2 **0.0681 / 0.1451 / 0.2133** vs F1 0.2620 (sum −0.0487 [−0.1128, −0.0026]); with the planned stop, **0.0612** vs 0.1593 | RESULTS_FINAL.md, step 2 |
| **Head-to-head with Xi, Torero & Jahn (2021)**, their droplet rebuilt from their equations | On their own droplet (FLEX-094): D2 **0.19 %** nRMSE vs their 2.58 %. On the sealed heptane test (their task): D2 − Xi −12.84 points [−36.44, −0.58]; on methanol −1.87 [−2.84, −1.10] | RESULTS_POINT_METRICS.md (`d513ce0`) |
| **Snegirev (2023)** on Flame Design test days | F2 − (Snegirev + clock): normal flames −0.088 [−0.149, −0.041] without the stop, −0.095 [−0.130, −0.058] with it | RESULTS_PUBLISHED_ACME_BASS.md §5 (`09c22d3`) |
| **Irace et al. (2023)** on Flame Design test days | F2 − (Irace R + clock), normal, no stop: sum −0.124 [−0.147, −0.080] | RESULTS_PUBLISHED_ACME_BASS.md §1 (`740b3df`) |
| **Frolov et al. (2023)** on Flame Design test flames | F2 0.014 vs 0.124 (no stop) and 0.027 vs 0.330 (stop), 1 day; median extinction-time error 3.9 s vs 15.8 s | RESULTS_PUBLISHED_ACME_BASS.md §5 |
| **Saffire transfer**: the 2 cm fabric model on 40.6 cm sheets (2 flights) | D1 tip CRPS **0.058** vs 0.250 for persistence; lower on 2 of 2 burns | feedback/saffire-transfer/results/README.md (`e08c79c`) |
| **All 19 PSI investigations** have a hand-written release with a held-data score | 10 new investigations modelled on 15 Nov; nine blind looks taken | PROTOCOL 13.2 (`734ccb2`), COVERAGE_19.md |
| **Seventh-generation final run on unread test days** | Holds against every baseline: BRE extinction 0.139 vs 0.222; E-FIELD ion current 0.319 vs 0.575; SPICE smoke point 4.18 vs 10.33 mm; SAME-R diameter 0.167 vs 0.256; kept-burning 0.048 vs 0.071 | RESULTS_FINAL_GEN7.md, PROTOCOL 13.7 (`cccdf3d`) |
| **Served in the product** | 14 releases covering all 19 investigations | ignis-web `configs/models/served/served.json` (PR 113, merge `89571e2`) |

---

## 2. Timeline

All times are local on 15 November unless marked 14 Nov. Times are commit times.

| Time | Event | Commit |
|---|---|---|
| 14 Nov 23:55 | Astra frozen TRAIN comparison of ten candidates registered | `39f39df` |
| 00:00 | Its result: no Astra candidate beats Fable overall | `84f24c7` |
| 00:18 / 00:26 | Astra R2 frozen and scored: removes the R1 ACME regression, no win over Fable | `570fffd`, `c7a192d` |
| 00:53 / 00:57 | Astra BASS matched-window operator (`bass_front.wsp4`) frozen and scored | `482b368`, `e1a7bad` |
| 01:20 / 01:21 / 02:03 | Saffire transfer registered, scorer frozen, run once: D1 transfers | `ba5c44c`, `a754fd8`, `e08c79c` |
| 03:05 | Owner asks for an unsupervised overnight run | dossier 11.1 |
| 03:23 | PROTOCOL 12 committed; seven lanes start | `8bfcb5f` |
| 03:34 / 04:04 | 12.6 registered and judged: P3 replaces P1 | `2750ab3`, `e63e5ab` |
| 04:45 | Text tier of all 19 investigations downloaded (22.14 GB) | `7e4d3c6` |
| 04:48 | 12.7 droplet transfers registered; FLEX-2 day groups frozen | `1c60180` |
| 04:54 / 05:16 | 12.8 fuel table F1 registered and judged | `787f5ce`, `3b9bc3d` |
| 05:03 | 12.9 zero-shot quench candidate Q0 registered | `2b19980` |
| 05:31 | CFI dodecane transfer (7 tests) | `53bf873` |
| 05:44 | Planned stop and burner limit read from script files (`acme_script2.py`) | `ece9f45` |
| 05:56 / 06:02 | 12.10 F2 registered and judged: F2 replaces F1 | `7a37ab5`, `bda16d4` |
| 06:24 | Final-run script rehearsed on validation only | `ad0e698` |
| 06:59 | First FLEX-2 scoring declared void (pixel scale assumed) | `2f3b4df` |
| 07:14 | 12.9 outcome: Q0 not adopted; BASS (PSI-26) holds no fabric burn | `c506154` |
| 07:32 | Final run registered (12.11); release list frozen | `73c662a` |
| 07:35 / 07:39 | FLEX-2 scale pinned (33.85 px/mm); rescored once | `8329336`, `eb80f6d` |
| 07:42 | **Final run 12.11 completed** | `5686866` |
| 08:04 | Seventh generation opened (PROTOCOL 13); s-Flame test 13.1 registered | `3cb261b` |
| 08:13 | Generic forward `law.rs`; 13.1 outcome: undecided | `e4477d6` |
| 08:54 / 09:15 | 13.2: nine blind looks registered, then taken | `98685fe`, `734ccb2` |
| 09:18 / 09:20 | 13.3: one revision per investigation registered, then scored | `79143e1`, `6d3c8e9` |
| 09:22 to 09:31 | 13.5 F3 of the burner flames: F3 stage 1 replaces F2 | `de62db2` … `fbe2a07` |
| 09:32 | ignis-web: nine generic-law releases served (all 19 investigations) | *ignis-web* `247f43a` |
| 09:34 | 13.5 stage 2, zero-shot helium: does not answer helium | `3631640` |
| 09:47 | 13.6: rules for studying the development data | `f683ee3` |
| 10:02 | Xi et al. head-to-head | `d513ce0` |
| 10:37 | Farouk & Dryer on FLEX-035 | `1b0df6d` |
| 10:54 | Irace, Kim and BASS published comparators | `740b3df` |
| 10:08 to 11:10 | 13.6 studies and candidates (BRE d3, cld3, efield3, bass3/4, SPICE P3, SAME-R S1, SLICE tubes) | `315f683` … `ef2aa44` |
| 11:16 / 11:19 | Fresh BASS and ACME batches frozen | `e036039`, `5e00e34` |
| 11:19 / 11:20 | n-Heptane D3 registered (13.7), then judged: not adopted | `e9f8137`, `861541a` |
| 11:24 | Final run of the seventh generation registered; lists frozen | `34a4d09` |
| 11:30 / 11:34 | Fresh ACME batch scored; planned-stop reader v3 | `6e657bb`, `ec9b75b` |
| 11:35 | **Seventh-generation final run completed** | `cccdf3d` |
| 11:41 | Frolov 2023 and Snegirev 2023 comparators | `09c22d3` |
| 11:42 | ignis-web: final model of each investigation served | *ignis-web* `532ebef` |
| 11:43 | Fresh BASS batch scored | `2524633` |
| 11:50 | 13.8: post-final revisions, by the owner's decision | `80046d3` |
| 12:06 / 12:13 | 13.8 outcomes: bass5 and f4-hydrogen served; fabric boundary not served | `3e076f6`, `4ae3ccb` |
| 12:06 | Fresh heptane batch frozen | `7103c6e` |
| 12:18 | n-Heptane D4 registered (13.9) | `9823afd` |
| 12:19 | Generation closed; served list and registry on the VM staging folder | `ee4653e` |
| 12:33 | ignis-web: PR 113 merged (14 served releases) | *ignis-web* `89571e2` |
| 13:18 to 13:29 | Fresh heptane day measured; 13.9 stage A: D4 not better than D2 | `b0bba02` … `542dad8` |

---

## 3. Before the night: Astra's frozen comparisons and the Saffire transfer (14 Nov 23:55 to 15 Nov 02:03)

### 3.1 Astra's ten candidates against adopted Fable, on TRAIN

- **How it was run.** Registered in `39f39df` before scoring, as `dossier/ASTRA_FROZEN_TRAIN_20261114.md`.
  `wsp-dev feedback` ran on identical paired TRAIN cases. All ten candidates are data-informed, authored by
  GPT-6 Astra.
- **Frozen rule.** Report paired differences, with no automatic adoption.
- **Result.** Taken from `feedback/astra-frozen-train-20261114/results/README.md`. Means are equal-weight
  cluster means.

| Family | Fable | Astra R1 | Joint (Fable + Astra) |
|---|---|---|---|
| FLEX methanol CRPS µm | 48.4466 | 48.4767 | 48.4621 |
| FLEX heptane CRPS µm | 51.0843 | 51.2117 | 51.1403 |
| ACME normal, extinction IBS contribution | 0.063131 | 0.064069 | 0.063592 |
| ACME inverse, extinction IBS contribution | 0.021342 | 0.022030 | 0.021674 |
| BASS tip CRPS | 0.076101 | 0.080217 | **0.073439** |

- Nine of ten have a worse primary TRAIN mean.
- Joint BASS is 3.50 % lower than Fable, on 3 tracks from 1 day, so no interval is possible.
- **Adopted:** none.

### 3.2 Astra R2 (four new data-informed releases)

- **How it was run.** Three GPT-6 Astra high-effort authors read the loss diagnostics. They wrote the
  two vapour-Cp corrections and the two ACME nominal-input rollbacks. Frozen in `570fffd`, scored in
  `c7a192d`.
- **Result.** From `feedback/astra-r2-20261115/results/README.md`:
  - ACME R2 reproduces Fable's case records exactly, which removes the R1 regression.
  - Methanol: R2 − R1 is −0.0000472 µm.
  - Heptane: R2 − R1 is +0.0037 µm.
- **Adopted:** none.
- **Lesson.** The missing information is thermal and flame history, not a more precise constant.

### 3.3 Astra BASS observation operator (`bass_front.wsp4`)

- **How it was run.** A new forward, `front4.rs`, uses the joint weights frozen (all 191 scalar values
  byte-identical). Frozen in `482b368`, scored in `e1a7bad`.
- **Result.** From `feedback/astra-bass-operator-20261115/results/README.md`:

| | Fable | Joint | wsp4 |
|---|---|---|---|
| tip CRPS | 0.07610 | 0.07344 | 0.07396 |

  wsp4 is 2.82 % below Fable and 0.71 % above the joint, on 12 cases from 1 day.
- **Adopted:** none. It is kept as a correctness candidate.

### 3.4 Saffire transfer (PSI-98, -99, -100): the narrow-strip fabric model on wide sheets

- **How it was run.**
  - **Protocol.** `SAFFIRE_TRANSFER.md`, registered in `ba5c44c` before any measurement. The scorer
    snapshot and release ids were frozen in `a754fd8`.
  - **Measurement.** A blind measuring agent (`claude-opus-5-5`) measured NASA's geometrically mapped
    JPEG frames. A Sonnet agent checked overlays on a fixed stride and accepted 30 of 30 sheet images.
  - **Scoring.** One run of `tools/wsp/saffire/score_transfer.py --execute` (`e08c79c`). It first
    reproduced the committed TRAIN aggregate to 1e-12.
  - **Clean-up.** The raw frames (5.7 GB) were deleted at 03:15 (dossier 11.5).
- **Frozen rule (§7).** D1 transfers if its tip CRPS is below persistence on every scored wide burn,
  with at least two burns scored.
- **Result.** From `feedback/saffire-transfer/results/README.md`. Wide sheets: 2 scoreable burns,
  8 cases. The 2 opposed sheets were refused at the flow condition.

| Release | Tip CRPS | Coverage 50 / 90 % | Minus persistence [interval over the 2 burns] | Burns lower |
|---|---|---|---|---|
| persistence | 0.250 | 0.26 / 0.58 | | |
| **`bass_front.wsp` D1** | **0.058** | 0.93 / 1.00 | −0.192 [−0.202, −0.182] | 2 of 2 |
| joint Astra (`wsp3`) | 0.061 | 0.93 / 1.00 | −0.189 | 2 of 2 |
| `wsp4` | 0.061 | 0.93 / 1.00 | −0.189 | 2 of 2 |

- **Verdict:** "transfers", across a width ratio of 18.5.
- The Saffire II 5 cm strips (0.654 vs 0.852) are reported only. The visual check accepted 6 of 31
  strip images there.
- No sheet quenched, so the test says nothing about the quench boundary.

---

## 4. The overnight run (PROTOCOL 12; from 03:23)

### 4.1 Instructions and rules

The owner's request (dossier `11_overnight_20261115.md` §11.1):
- work unsupervised to the end and build the best models possible, without overfitting;
- use Opus 5.5 agents for code and Fable 5.2 for physical reasoning and for typing weights;
- run an "LLM-driven back-propagation" over the NASA data;
- measure 200 to 300 GB of NASA images and video in sequence, deleting media as it piles up.

The separate evaluation lane was removed. From here the author's lane scores (PROTOCOL 12.1, `8bfcb5f`).

These rules stayed (dossier §11.2):
- the author types every weight;
- a candidate and its rule are committed before its score on unseen data exists;
- new data is scored blind once, and only then studied;
- media is measured by an agent that sees no forecast;
- `holdout/` stays closed until one final run;
- every number is named zero-shot or data-informed.

**"Typed back-propagation" (PROTOCOL 12.3).** One step runs in this order:
1. Forward on train.
2. Decompose the loss by session, regime and horizon.
3. Attribute: a script perturbs each block of numbers and reports how the loss moves. It writes no
   weight.
4. The author types a damped step, about half of what the attribution suggests, to 2 or 3 significant
   figures, with a written physical reason. The error pattern must hold across several sessions.
5. One step per new batch of data.

An error that no allowed value explains is a missing mechanism, which means a new forward. Band widths
are never set from the batch being scored.

### 4.2 Lanes, agents and roles (dossier §11.4, §11.5)

| Lane | Model | Task |
|---|---|---|
| ENG-ACME | Opus 5.5 | verify `flame4.rs`, `committee2.rs`; 288 tests pass; 80 releases reassembled with unchanged ids |
| DATA-TEXT | Opus 5.5 | every non-media file of the 19 investigations: 41,004 of 41,031 files (22.14 GB), with a manifest of sha256 values |
| MEDIA-FABRIC | Opus 5.5 | blind tracks of the BASS-II and BASS burn videos |
| MEDIA-DROPLET | Opus 5.5 | droplet-diameter measurer, checked against NASA's processed FLEX series |
| MEDIA-ACME | Opus 5.5 | pilot: flame radius from the colour camera (1 to 2 %, about 0.55 GB per test) |
| PHYS-QUENCH | Fable 5.2 | zero-shot ignition and quench boundary of a thin fuel (`research/phys_quench_w0.md`) |
| PHYS-DROPLET | Fable 5.2 | droplet laws for the CFI and FLEX-2 fuels from property ratios (six zero-shot candidates) |
| PHYS-SPHERICAL (03:30) | Fable 5.2 | fuel table for the burner flames (ethylene, methane) |
| ENG-ATTRIB (03:30) | Opus 5.5 | `tools/wsp/attrib/attrib.py` (`decompose`, `sensitivity`) |
| ENG-FUEL, ENG-QUENCH, ENG-DROPLET | Opus 5.5 | forwards `flame5/6/7.rs` + `committee3.rs`; `front5.rs`; `droplet8.rs`, `droplet9.rs` |
| PLAN-STOP / PHYS-HELD (05:22) | Opus 5.5 / Fable 5.2 | planned stop from the script file; why enriched-oxygen ethylene flames hold |
| TRANSFER-ACME, TRANSFER-DROPLET | Opus 5.5 | scoring chains for Flame Design, s-Flame, CFI and FLEX-2 |

---

## 5. Sixth generation (PROTOCOL 12.6 to 12.10)

### 5.1 12.6: normal burner flames P2 and P3

- **How it was run.**
  - **Origin.** A train-only analysis of `acme_normal.wsp5` P1 by Fable at high effort.
  - **Forwards.** Two new forwards: `flame4.rs` (wsp8) and `committee2.rs` (wsp7).
  - **Registration.** Registered in `2750ab3` before the look.
  - **Look.** Two parts read together: (a) CFI-G validation, 84 cases in 2 sessions, and (b) Flame
    Design, 103 tests in 10 sessions.
- **Frozen rule.** Two conditions:
  - the decision score (mean over the two investigations of extinction + termination) is not higher
    than the carry's;
  - the mean of extinction alone is not higher.
- **Result** (RESULTS_GEN6.md):

| Decision score, without / with the planned stop | Carry (P1) | P2 | **P3** |
|---|---|---|---|
| decision | 0.3538 / 0.1902 | 0.3503 / 0.1872 | **0.3416 / 0.1862** |
| guard (extinction mean) | 0.1322 / 0.1078 | 0.1323 / 0.1088 | **0.1306 / 0.1075** |

- **Adopted: P3** (`acme_normal.wsp7`) replaces P1 on both paths.
- **Refused: P2**, which failed the guard.
- Flame Design extinction, paired P3 − P1: −0.0018 [−0.0030, −0.0006].

### 5.2 12.8: fuel table F1, ethylene as itself

- **How it was run.**
  - **Forwards.** `flame5/6/7.rs` and `committee3.rs` (`ea3c531`). They are bit-for-bit neutral on
    every CFI-G train prefix (310 tests).
  - **Registration.** F1 registered in `787f5ce`.
  - **Comparison.** Ethylene as itself against ethylene borrowed as ethane, on identical cases.
- **Result** (`feedback/flame_design_fuel_table_result.md`). Extinction without the planned stop:

| Family | F1 | as ethane | Difference |
|---|---|---|---|
| inverse (236 cases, 7 sessions) | **0.1077** | 0.1466 | −0.0388 [−0.0658, −0.0101]: adds something |
| normal (240 cases, 9 sessions) | 0.1853 | 0.1128 | +0.0725 [+0.0402, +0.1022]: loses something |

- The registered expectation for inverse flames held: twice the clock was expected and 1.7 was
  observed.
- The normal-flame expectation was refuted in direction. That row went to the next revision.

### 5.3 Attribution and the planned stop from the script file

- **Fuel table.** Seven variants of the ethylene row were scored on train days only
  (`feedback/flame_design_fuel_table_attribution.md`). A position beyond ethane scored 0.1065 against
  0.1272 for "as ethane", and was better in 8 of 8 sessions.
- **`tools/wsp/ext/acme_script2.py`** (`ece9f45`). The planned stop is the stage sum, plus the igniter
  delay before retract, plus the fibre step. Recorded minus planned stop, median, before → after:

| Set | Before | After | Share within 1 s after |
|---|---|---|---|
| Flame Design normal | +9.86 s | +0.04 s | 86 % |
| Flame Design inverse | +2.30 s | +0.21 s | 77 % |
| CFI-G normal | +0.27 s | −0.70 s | 100 % |
| CFI-G inverse | +0.28 s | −0.02 s | 100 % |

- The burner limit is also a script setting: 500 C through 23 Aug 2021 and 450 C from 13 Sep.
- PHYS-HELD (`research/phys_ethylene_held.md`): fuel mass flow alone separates the enriched-oxygen
  ethylene flames that went out from those that held. Ethylene burns as ethane at about 0.58 of its
  flow.

### 5.4 12.10: F2, one revision from the train days

- **Typed.**
  - ethylene coordinates (1.0, 0.0);
  - hinge flow shift −0.31, the half-step of the −0.54 the data asked for;
  - growth critical-flux shift −0.02;
  - the new planned stop, read at −1, 0 and +1 s with weights 0.25, 0.5, 0.25;
  - the burner-limit switch days 2021235 and 2021256.
- **Registration.** `7a37ab5`, before the look.
- **Rule.** As 12.6, per family and path.
- **Result** (RESULTS_GEN6.md):

| Family | Decision score F1 → F2, without / with stop | Guard F1 → F2, without / with |
|---|---|---|
| normal | 0.3542 / 0.1579 → **0.3374 / 0.1532** | 0.1048 / 0.0792 → **0.0927 / 0.0771** |
| inverse | 0.2499 / 0.1493 → **0.2499 / 0.1384** | 0.1080 / 0.1134 → 0.1080 / 0.1134 |

- **Adopted: F2** (`acme_normal.wsp12` `sha256:55fcf4cb…971d`, `acme_inverse.wsp12`
  `sha256:dff7da0d…594e`) in both families on both paths (`bda16d4`).
- With the stop on CFI-G validation, F2 normal is above F1 (0.2271 vs 0.2191). The rule passes on the
  Flame Design half.

### 5.5 12.9: fabric ignition and quench, zero-shot Q0

- **How it was run.**
  - **Forward.** `front5.rs` (`bass_front.wsp5`), merged in `eeb2003`.
  - **Quadrature.** It uses 41 equal-probability nodes. The first 9-point Gauss-Hermite rule missed the
    closed form (0.70 vs 0.82); the author required the change before merge.
  - **Release.** Q0 registered in `2b19980`.
  - **Scoring.** On the BASS-II validation burns: 17 tracks, 12 scoreable, 40 cases.
- **Result** (PROTOCOL 12.9 outcome):

| | D1 | carry ("always lit") | Q0 |
|---|---|---|---|
| tip CRPS | 0.1168 | 0.1168 | 0.1168 |
| track-ended Brier | 0.1780 | 0.1902 | **0.1356** |
| ignition-outcome Brier (primary) | 0.1703 | **0.1333** | 0.1928 |

- **Refused: Q0.** Its primary score is worse than the constant. D1 stays.
- BASS (PSI-26) turned out to hold no fabric burn: all 114 videos were checked (`c506154`).

### 5.6 12.7: droplet transfers (CFI and FLEX-2)

- **Registration.** `1c60180`, with amendments in `a573d81`, `953c3be` and `2f3b4df`.
- **Forwards.** `droplet8.rs` and `droplet9.rs` (`298c3ee`).
- **Zero-shot candidates.** n-octane, iso-octane, n-decane, n-dodecane, farnesane and ethanol, each
  typed from property ratios.

**CFI arm F** (`53bf873`, `feedback/cfi_transfer_result.md`):
- 7 n-dodecane tests on 5 days, 21 cases.
- Dodecane W0 scored 239.9 µm, against 259.1 for the heptane law, 431.5 for persistence and 707.6 for
  the d-squared law.
- Difference to the heptane law: −19.3 [−85.6, +58.5]. As registered, no conclusion from 7 tests.
- One expectation was refuted: 4.0 to 4.4 mm droplets lost the hot flame and went on in a cool flame.

**FLEX-2 arm H** (adopted heptane D2, unchanged), rescored on the pinned scale (`eb80f6d`,
`feedback/flex2_transfer_result_v2.md`). 8 tests, 23 cases, 2 train days:

| | D2 | d-squared law | W0 | persistence |
|---|---|---|---|---|
| CRPS µm | **20.9** | 32.6 | 35.6 | 444.8 |

- D2 − d-squared law: −11.7 [−13.6, −10.0]. D2 is lower on both days.
- The rule named 3 days and one had no usable series. The model carries over on the two that exist.

**FLEX-2 arm F, n-decane** (6 tests, 4 days): candidate 45.8 vs root 38.8, +7.0 [−11.5, +34.8].
Undecided. n-Octane had 1 test and iso-octane none. No droplet candidate was adopted.

---

## 6. Media measurement batches

| Batch | Data | Size | Measured / usable | Result | Deleted |
|---|---|---|---|---|---|
| Saffire sheets and strips | mapped JPEG frames, PSI-98/99/100 | 5.7 GB | 4 sheets (2 scoreable), 2 strips | §3.4 | yes, 03:15 |
| MEDIA-FABRIC | BASS-II (142 burn files) and BASS (114 videos) | about 31 GB | 12 usable fabric tracks (5 train, 4 validation, 3 test); PSI-26 holds no fabric | fed 12.9 and step 4 of the final run, after a converter (`67eeeb4`) | — |
| FLEX-2 batch 1 | 48 free droplets, backlit archives | 70.7 GB | 15 of 48 usable (heptane 8, decane 6, octane 1) | first scoring void; rescored | — |
| FLEX-2 test days | 11 droplets (octane 6, decane 5) | 14.9 GB | 2 usable (both n-decane) | step 5 of the final run | — |
| FLEX-2 batch 2 (40 min cap) | 47 tests | 44.2 GB | 15 usable after visual check | no revision judged (ambient unknown on most) | yes |
| FLEX-2 batch 3 | 4 archives | — | 2 usable n-decane | closed for time | — |
| MEDIA-ACME pilot | colour camera, CFI-G train | about 0.55 GB per test | radius to 1 to 2 %, scale to 4 % | method only | — |
| Fresh BASS T13 / T20 | 2 SIBAL burns | 1.24 GB | 3 tracks | §11.2 | yes |
| Fresh heptane | FLEX-2 day 2013-03-28 and 2013 PSI-69 points | 30.3 GB planned (24 tests) | E087 day: 5 of 8 usable | §11.3 | — |

Sources: dossier 11.5; PROTOCOL 12.7, 12.9, 12.11; `atlas/splits/heptane-fresh-blind.json`;
`atlas/splits/BASS-batch-fresh.json`.

**Scale pinning and the voided FLEX-2 scoring.**
- **The mistake.** The first FLEX-2 scoring (06:59, `2f3b4df`) used an assumed 50 mm field of view.
  The measurer's own rod check had already warned of about 30 % oversize. The scores ran anyway, a
  sequencing mistake recorded by the lead author.
- **The void numbers.** D2 460.5 µm vs 177.6 for the d-squared law (`feedback/flex2_transfer_result.md`).
  No verdict was drawn.
- **The pinned scale.** 33.85 px/mm, 1.5 % (1 sigma), from two pieces of evidence that involve no
  droplet size, rate or forecast (`8329336`, PROTOCOL 12.11 outcome):
  - the FLEX-2 requirements document (3.0 cm view at 30 µm/px);
  - the needle-arm widths (53.9 px on 47 FLEX-2 tests, 54.4 px on FLEX).
- **The correction.** Every first-series diameter was 1.653 times too large.
- **Sensitivity of the rescore.** D2 stays below both baselines on both days for diameters ±1.5 % and
  for a time origin shifted by ±0.4 s.

---

## 7. The final run of the sixth generation (PROTOCOL 12.5, registered as 12.11)

**How it was run.**
- **Who and with what.** Once, by the scoring agent (`claude-opus-5-5`), with
  `tools/wsp/final/run_final.py`.
- **Rehearsal.** On validation only: `ad0e698`, `5349d98`, `31a2906`. Every known number reproduced,
  e.g. D3 40.0 µm and D2 46.1 µm; the input split matched the evaluator to 6e-17.
- **Release list.** `releases_final_20261115.json`, sha256 `fe6ccec0…f686`, frozen in `73c662a`.
- **Binaries.** `ignis.exe` `b03a35c8…8ffb`, `wsp-dev.exe` `bd6055e0…4d73`.
- **Reveals.** 3,792 reveal-log lines (`final_test: true`).
- **Output.** Aggregates only, in `5686866` (RESULTS_FINAL.md). Nothing was adopted or refused here.

**Step 1: the sealed test partitions.**

| Family | Release | Cases / sessions | Score | Best comparator | Difference |
|---|---|---|---|---|---|
| FLEX methanol | D3 | 65 / 9 | CRPS **18.5**, coverage 0.83 / 0.99 | d-squared law 31.6 | −13.1 [−19.0, −7.7] |
| | | | | W0 37.2; v3 42.7; v1 58.9 | −18.7 [−35.2, −5.8]; −24.2 [−32.7, −15.9]; −40.4 [−56.2, −26.5] |
| FLEX n-heptane | D2 | 43 / 7 | CRPS **32.2**, coverage 0.68 / 0.91 | d-squared law 54.0 | −21.8 [−47.1, −4.9] |
| | | 42 | 33.3 | W0 80.4 | −47.0 [−124.9, −3.2] |
| CFI-G normal | F2 | 60 / 2 | **0.0414 / 0.0494** (with stop 0.0411 / 0.0466) | P1 0.0456 / 0.0558; linear 0.0933 / 0.1199 | extinction vs P1 −0.0036 [−0.0056, −0.0016]; vs linear −0.0498 [−0.0566, −0.0430] |
| CFI-G inverse | F2 | 93 / 3 | **0.0998 / 0.1103** (with stop 0.1045 / 0.0793) | baseline set 0.2222 / 0.2325 | −0.1260 [−0.1782, −0.0265] |
| | | 69 / 3 | 0.1045 / 0.0752 | hazard 0.1645 / 0.0817 | extinction −0.0553 [−0.1181, −0.0086] |

F2 inverse equals P1 exactly on these alkane flames without the stop.

**Step 2: Flame Design test days, all ethylene.** Normal flames: 29 tests, 87 cases, 3 days. Inverse
flames: 20 tests, 60 cases, 1 day.

| Family, path | F2 | F1 | as ethane | incidence | F2 − F1, sum |
|---|---|---|---|---|---|
| normal, no stop | **0.0681 / 0.1451 / 0.2133** | 0.2620 | 0.2216 | 0.4244 | −0.0487 [−0.1128, −0.0026] |
| normal, stop (83 cases) | **0.0444 / 0.0168 / 0.0612** | 0.1593 | 0.1131 | 0.4320 | −0.0981 [−0.1177, −0.0078] |
| inverse, stop | **0.0001 / 0.0020 / 0.0020** | 0.0399 | 0.0399 | 0.6766 | −0.0378 (1 day) |

**Steps 3 to 5: very small sets, reported as anecdotes.**
- **CFI dodecane test day** (1 droplet, 3 cases): W0 14.8 vs 39.7 for the heptane law.
- **BASS-II test** (2 burns, 7 cases): D1 tip CRPS 0.0839, and ignition Brier 0.0274 vs 0.5347 for Q0.
- **FLEX-2 test days** (2 n-decane tests, 6 cases): the zero-shot decane W0 scored 80.6, against 65.4
  for the heptane law and 25.4 for the classical d-squared law. This is a loss on one day.

**Changes during the run.** Two script changes were made after the go and before step 1, each
rehearsed:
- a failed comparator is recorded instead of stopping the step;
- step 4 reads the converted fabric folder.

---

## 8. Seventh generation: a model for every investigation (PROTOCOL 13)

### 8.1 Set-up

At 07:42 the owner asked for all 19 investigations to be modelled. Each of the ten remaining
investigations got its own lanes (dossier 11.5):

- **A data and scoring engineer (Opus 5.5)** per investigation, briefed by `runs/dev/gen7/BRIEF_DATA.md`.
  - **Delivers:** a feasibility file, case builders and baselines on train days.
  - **Reads:** train days in full and validation inputs only. Validation outcomes went to a separate
    file by script, and test days stayed unopened.
- **A physicist-author (Fable 5.2)** per family, briefed by `runs/dev/gen7/BRIEF_PHYSICS.md`.
  - **Delivers:** a zero-shot law `research/phys_<slug>_w0.md`, with every constant literal and
    origin-classed, registered expectations, and the law restated as a typed network.
  - **Barred from:** `runs/`, `data/`, `holdout/`, and every outcome feedback file.
- **One engineer** for the generic forward `law.rs` (`e4477d6`).
  - **What it hosts:** up to 16 features and 8 targets (number, yes/no, or event), with hinge networks.
  - **Regression check:** 14 new tests pass, and all 182 existing ids rebuild unchanged.
- **One engineer** transcribed the dossiers into releases without choosing any number
  (`runs/dev/gen7/transcription_checks.md`).
- **One engineer** served the releases in the product (§14).

### 8.2 13.1: s-Flame with F2 unchanged

- **Inputs.** The adiabatic temperature was computed as NASA's tables compute theirs (within 4.1 K on
  154 CFI-G rows).
- **Result** (46 cases, 2 days; `feedback/sflame_transfer_result.md`):
  - F2 scored 0.2619 without the stop and 0.3698 with it, against 0.2793 for the CFI-G incidence.
  - **Undecided.**
- **Cause.** 7 low-flow methane tests reach the 250 C burner limit before their planned stop.

### 8.3 13.2: the nine blind looks

- **How it was run.** Ten zero-shot releases on `law.rs`, registered in `98685fe` and scored once in
  `734ccb2`. SPICE has two variants; the published-fit one is not blind.

| Investigation | Target | Model | Simple baseline | Best baseline | Reading |
|---|---|---|---|---|---|
| SAME-R | particle diameter | **0.167** | 0.393 | 0.339 | beats every baseline |
| SAME-R | detector at full scale | **0.228** | 0.411 | 0.247 | beats every baseline |
| SAME (held whole) | particle diameter | **0.184** | 0.220 | 0.192 | beats every baseline |
| DAFT (held whole) | number concentration | **0.777** | 2.678 | (only baseline) | beats it |
| E-FIELD | flame out under the field | **0.0753** | 0.0884 | 0.0884 | beats every baseline |
| CLD Flame | radiometer | **0.108** | 0.263 | 0.174 | beats every baseline |
| Plastics | kept burning | **0.086** | 0.120 | 0.120 | beats every baseline |
| E-FIELD | ion current µA | 0.546 | 0.620 | 0.374 | beats simple |
| SPICE | smoke-point length mm | 5.862 | 6.099 | 5.042 | beats simple |
| SLICE | blow-off | 0.155 | 0.161 | 0.149 | beats simple |
| BRE | operator termination | 0.1383 | 0.1799 | 0.0924 | beats simple |
| BRE / plastics quench / SLICE lifting / CLD loss | — | 0.2755 / 0.517 / 0.089 / 0.593 | 0.2274 / 0.230 / 0.056 / 0.478 | | lose |

Of 18 targets, 8 beat every baseline, 6 beat the simple one, and 4 lose.

### 8.4 13.3: one damped step per investigation

Registered in `79143e1` and scored in `6d3c8e9`.

| Revision | Validation W0 → D1 | Reading |
|---|---|---|
| CLD Flame, flame lost | 0.5925 → **0.5139** | replaces W0 |
| Plastics, quench speed | 0.5169 → **0.5067** | replaces W0 |
| SLICE | lifting 0.0886 → 0.0555, but blow-off +5.4 % | refused (5 % limit) |
| BRE | 0.2755 → 0.2866 | refused |
| SAME-R | 0.1672 → 0.1725 | refused |

### 8.5 13.4 to 13.6: study of the development data and second laws

- **Validation opened.** After two selection uses, validation days became study data (13.4, `9bd3017`).
  Every number from then on is in-sample and labelled so.
- **13.6 rules (`f683ee3`).**
  - The author studies the cases and writes the expected effect first.
  - At most five candidates per law, run through the logged what-if tool `tools/wsp/ext/law_whatif.py`.
  - A candidate whose effect was not predicted in sign is dropped.
  - Kept only if lower pooled and on two thirds of the days that change. With fewer than six changing
    days, it becomes a comparator.

Development results (in-sample, train and validation days together; PROTOCOL 13.4 and 13.6 outcome
tables):

| Investigation | Kept candidate | Target | Before → after | Best baseline | Days lower |
|---|---|---|---|---|---|
| BRE | d3 (burner heat share cut, no pressure term, clock 0.42) | extinction | 0.2576 → **0.1121** | 0.2269 | 13 / 14 |
| CLD Flame | C1 then cld3 | flame lost | 0.3174 → 0.0431 → **0.0334** | 0.2282 | 21 / 25; 7 / 9 |
| E-FIELD | E1 then efield3 | ion current | 0.6592 → 0.2530 → **0.2103** | 0.4256 | 15 / 15; 8 / 9 |
| SPICE | P2 then P3 | smoke point; flame length | 6.51 → **2.13** mm; 1.94 → **1.25** mm | 6.63; 1.84 | 9 / 9; 8 / 9 |
| SAME-R | S1 | diameter; detector | 0.2377 → **0.1717**; 0.1753 → **0.0562** | 0.2762; 0.1012 | 12 / 12; 10 / 10 |
| Plastics | bass2 → bass3 → bass4 | quench speed; fabric kept burning | 0.8135 → 0.4505 → **0.1310**; refused → **0.0529** | 0.5786; 0.3801 | 12 / 14; 20 / 22 |
| SLICE | slice2 s1 | lifting | 0.1474 → **0.0333** | 0.0293 | 8 / 8 |
| SLICE | documented tubes (data fix) | blow-off | 0.1089 → **0.0456** | 0.0473 | — |

Not kept, one line each:
- cld2: 5 of 8 days.
- efield2: worse on the ion current.
- BRE d2: 9 of 14 days.
- SLICE S2: dropped on the corrected tubes.
- s-Flame F4 stages: comparators, because fewer than six days change.

### 8.6 13.5: F3 of the normal burner flames

- **Forward.** `acme_normal.wsp13` (`de62db2`). A neutral carry reproduces F2 on 1,248 prefixes.
- **Stage 1 (`sha256:d1660331…45aa`):**
  - the burner limit as an input;
  - a heating law with flame-temperature and conductivity terms;
  - a methane step;
  - an ethylene dilution term.

| | F3 stage 1 | F2 | CFI-G incidence |
|---|---|---|---|
| s-Flame validation, 46 cases, sum without / with stop | **0.2048 / 0.2051** | 0.2619 / 0.3698 | 0.2793 |
| CFI-G validation, extinction / termination | **0.1536 / 0.0956** | 0.1537 / 0.0961 | |
| Flame Design validation, no stop | **0.0299 / 0.3737** | 0.0317 / 0.3933 | |

- **Adopted: F3 stage 1** as the normal-flame model on CFI-G, Flame Design and s-Flame (`c401b4e`,
  `fbe2a07`).
- **Refused: stage 2**, the zero-shot helium row: 0.999 vs 0.330 (`3631640`).

---

## 9. Final run of the seventh generation (PROTOCOL 13.7)

**How it was run.**
- **Lists and tools.** Frozen lists `releases_final_gen7.json` (sha256 `f3556071…70e2`, 7 models and
  27 comparators) and `releases_final_sflame.json` (`ba34f8f7…e9c9`), in `34a4d09`. Tools:
  `run_final_gen7.py` and `run_final_sflame.py`.
- **Rehearsal.** 466 validation rows reproduced, with a largest difference of 0.
- **Guards.** One marker per investigation.
- **Results.** `cccdf3d` (RESULTS_FINAL_GEN7.md, RESULTS_FINAL_SFLAME.md).
- **Tooling error on s-Flame.** The step first stopped before any aggregate was written: the default
  binary predated wsp13. It was run once with the merged binary.
- **Rule.** The model *holds* if it is lower than every baseline.

| Investigation | Target | Cases | Model | Zero-shot parent | Best baseline | Reading |
|---|---|---|---|---|---|---|
| BRE | extinction | 95 | **0.1389** (d3) | 0.2448 | 0.2220 | holds |
| BRE | operator termination | 95 | **0.0535** | 0.1070 | 0.0999 | holds |
| E-FIELD | ion current µA | 788 levels | **0.3190** (efield3) | 0.7039 | 0.5746 | holds |
| E-FIELD | flame out in window | 39 | **0.0547** | 0.1072 | 0.1331 | holds |
| SPICE | smoke-point length mm | 6 | **4.18** (P3) | 6.94 | 10.33 | holds |
| SPICE | flame length given the flow mm | 6 | **1.18** | 2.98 | 2.56 | holds |
| SAME-R | particle diameter | 33 | **0.1672** (S1) | 0.2595 | 0.2559 | holds |
| SAME-R | detector at full scale | 33 | **0.0144** | 0.0807 | 0.0189 | holds |
| CLD Flame | lost before fuel-off | 63 | **0.0175** (cld3) | 0.0545 | 0.0568 | holds |
| CLD Flame | radiometer | 63 | **0.1145** | 0.1411 | 0.1439 | holds |
| Plastics and fabrics | kept burning | 42 | **0.0477** (bass4) | 0.0588 | 0.0707 | holds |
| CLD Flame | flame lost, time course | 63 | 0.1239 (cld3) | 0.0786 | 0.1020 | does not hold; D1 0.0726 does |
| Plastics | quench speed, day mean | 38 | 2.9103 | 1.6069 | 2.7049 | does not hold (case mean 1.54 vs 2.11) |
| s-Flame helium | sum | 49 | 0.708 (F4 helium) | 1.057 | 0.416 | does not answer helium |
| SLICE | blow-off; lifting | 31 lines | 0.0021; 0.0041 | | 0.0000 | no event on the answered lines |

Chosen after the run, and labelled so: CLD Flame is served by D1.

---

## 10. Revisions after the final run (PROTOCOL 13.8; owner's decision)

All three are labelled "typed after the final run with every partition seen; in-sample; no held-out
score" (`80046d3`). The serving rule: lower than the constant on train, validation and test days, for
the repaired target.

| Answer | Finding | Release | Train / validation / test vs constant | Served |
|---|---|---|---|---|
| Plastics quench speed | O13a mis-coded (pot settings and fan displays in swapped cells); coding rules R11 to R15; one slab separated by width | `bass5` `sha256:050a4ceb…b82d` | **0.1143 / 0.1082 / 0.1255** vs 0.6709 / 0.5466 / 0.9157 | yes (`3e076f6`) |
| s-Flame helium and hydrogen | 11 of 26 test-day ethylene tests ramp their flow and had been scored as constant-flow | `f4-hydrogen` `sha256:4cc2fa60…9436` | helium validation **0.1105** vs 0.3299; test without ramped tests **0.2419** vs 0.4143; hydrogen train 0.3140 vs 0.9087 | yes (`55b1440`, `4ae3ccb`) |
| BASS-II fabric quench boundary | the burns bracket the boundary without placing it | one number (floor 14.8 → 14.1 %) | 0.1612 vs 0.1333 on validation | no; refusal stands |

---

## 11. n-Heptane D3 and D4, and the fresh blind batches (test, then train)

### 11.1 D3 (PROTOCOL 13.7, heptane section; RESULTS_HEPTANE_D3.md)

- **What changed.** The cool-flame rate law only, five statements. Twelve train cool flames burn at 0.31
  to 0.36 mm²/s, with no oxygen, CO2 or pressure trend.
- **Registration.** `e9f8137`.
- **Look.** One look on validation without FLEX-035 (36 cases, 6 sessions), taken in `861541a`.

| Criterion | D3 | D2 |
|---|---|---|
| CRPS at 10 s | 51.3 | 49.9 |
| MAE at 39 s | 101.6 | 98.6 |

**Refused.** D2 stays.

### 11.2 Fresh ACME and BASS batches

**ACME** (frozen in `5e00e34` before any endpoint was read; scored in `6e657bb`):
- 26 Flame Design tests that no release had scored.
- They are fresh tests on days already used, not fresh sessions.
- Endpoints were measured after the forecasts.

| Block (cases) | F2 | F1 | Incidence | Irace + clock |
|---|---|---|---|---|
| A2, real extinctions, no stop (9) | **0.256** | 0.288 | 0.400 | 0.409 |
| A2, stop (9) | **0.054** | 0.087 | 0.400 | 0.161 |
| A3, inverse, stop (4) | **0.000** | 0.026 | 0.786 | 0.000 |
| A1, scripted ramp, no stop (60) | 1.743 | 1.816 | 1.090 | 1.618 |
| A1, stop, reader v2 → v3 (31) | 0.273 → **0.120** | 0.088 | 1.062 | 0.020 → 0.013 |

- **Planned-stop reader v3** (`ec9b75b`): no fibre wait without a traverse.
  - It changes 6 tests (19123L3 to L8), all on a spent day.
  - It is data-informed and has no independent check.

**BASS** (frozen in `e036039`; measured blind with the new top-view measurer; scored in `2524633`). D1 is
lower than every published-constant forecast on both burns:
- T13 top view: 0.158 vs 0.184 (steady prefix) and 0.222 (Ferkul).
- T20: 0.270 vs 0.368 or more.

### 11.3 Fresh heptane batch and D4 (PROTOCOL 13.9)

**The batch** (`atlas/splits/heptane-fresh-blind.json`):
- 31 tests, frozen in `7103c6e` before any frame was opened.
- It holds the 8 tests of FLEX-2 day 2013-03-28 and 23 PSI-69 points of 2013.
- 24 tests are in the measured batch; 7 fibre-supported tests are excluded.
- The tool is frozen: `droplet_diameter.py` sha256 `66bcb462…eb3`, at the pinned scale.

**The candidate.** D4 (`flex_heptane.wsp10`, `sha256:f3ccc122…b164`, `9823afd`):
- D2 plus the D3 cool rate;
- a transition stage with one new number, `trans.depth` 0.25.

**Stage A** (`542dad8`): 5 usable tests of 2013-03-28, 9 cases.

| | D4 | D2 |
|---|---|---|
| MAE at 10 s | **18.9 µm** | 20.7 µm |
| nRMSE over the burn | 3.84 % | **3.68 %** |

- Criterion 2 fails, so D4 is not adopted. D2 stays.

---

## 12. Published-model head-to-heads (post-hoc, after the final run)

Common recipe (RESULTS_POINT_METRICS.md, RESULTS_PUBLISHED_ACME_BASS.md):
1. Read the full text.
2. Rebuild the published predictor from its equations.
3. Find the paper's own NASA runs in our data.
4. Reproduce the paper's numbers.
5. Score on the same cases.

Research was done by Opus 5.5 medium agents, and every script was re-run by the lead. Nothing was
adopted, and no weight was tuned.

**Point metrics on the sealed FLEX test** (`tools/wsp/final/posthoc_point_metrics.py`):
- 108 reveal-log lines.
- The check reproduces the final-run MAE.
- nRMSE, session-weighted:
  - methanol D3 **1.52 %** vs 2.52 % for the d-squared law;
  - heptane D2 **5.41 %** vs 6.78 %.
- Paired MAE:
  - methanol −21.4 µm [−29.4, −12.8];
  - heptane −28.5 µm [−60.1, −5.2].

**Xi, Torero & Jahn (2021)** (`d513ce0`, `posthoc_xi_comparator.py`):
- Their droplet was identified as FLEX-094 (train), by tracing their Fig. 2. The trace gives 2.42 %
  against their reported 2.46 %.
- The rebuild gives 2.58 %.

| Task | Cases | Xi rebuilt | IGNIS adopted | d-squared | IGNIS − Xi |
|---|---|---|---|---|---|
| FLEX-094, cutoff 5 s, 5–10 s | 1 | 2.58 % | **0.19 %** (D2) | 0.92 % | — |
| heptane, their task | 12 | 16.05 % | **3.20 %** | 3.22 % | −12.84 [−36.44, −0.58] |
| same, without H05H103 | 11 | 1.89 % | **0.82 %** | 1.02 % | −1.08 [−1.76, −0.35] |
| heptane, evaluator window | 43 | 17.16 % | **5.41 %** | 6.78 % | −11.75 [−29.76, −1.62] |
| methanol, their task | 20 | 3.27 % | **1.40 %** | 1.72 % | −1.87 [−2.84, −1.10] |
| methanol, evaluator window | 64 | 6.95 % | **1.52 %** | 2.52 % | −5.43 [−7.62, −3.77] |

**Farouk & Dryer (2014)** on FLEX-035 (exposed validation run; `1b0df6d`, `f56b4da`, `961394b`). The
detailed simulation wins on every window:

| Window | Farouk & Dryer | D2 | D3 | D4 |
|---|---|---|---|---|
| 1–11 s | 0.62 % | 0.86 % | 0.76 % | 0.70 % |
| whole burn from 1 s | 5.18 % | 13.05 % | 11.48 % | 11.36 % |

**Irace et al. (2023)** (`740b3df`):
- Fidelity: steady radii within 1.7 %; the R rule agrees on 6 of 6 cases.
- Flame Design test days, F2 − (Irace R + clock), sum:
  - normal, no stop: **−0.124 [−0.147, −0.080]**;
  - normal, stop: −0.084 [−0.223, +0.062].
- Irace R alone is far worse than either (0.540 vs F2's 0.213, normal, no stop).
- CFI-G train, inverse flames: F2 AUC **0.995** vs 0.60 for R.

**Frolov et al. (2023), *Fire* 6, 285; Snegirev (2023), *CNF* 248, 112499** (`09c22d3`), on Flame Design
test days:

| Comparator | Path | IGNIS − comparator |
|---|---|---|
| Frolov | normal, no stop (1 day) | 0.014 vs 0.124: −0.110 |
| Frolov | normal, stop (1 day) | 0.027 vs 0.330: −0.303 |
| Snegirev rule | normal, no stop / stop | −0.327 [−0.744, −0.031] / −0.432 [−0.959, −0.027] |
| Snegirev + clock | normal, no stop / stop | **−0.088 [−0.149, −0.041] / −0.095 [−0.130, −0.058]** |

On inverse validation flames without the stop, Frolov is ahead: +0.059 [+0.013, +0.200].

**Kim et al. (2023).** A flame-radius model, rebuilt to within 0.68 %. IGNIS forecasts no radius, so
there is no head-to-head.

**BASS published spread rates** (validation, 3 burns):
- D1 **0.121**;
- steady-rate prefix 0.228;
- Ferkul 2012 0.449.

---

## 13. Astra: results and review findings

**Results.**
- §3.1 to §3.3 hold the scores. No Astra candidate was adopted.
- Astra's releases (joint `wsp3`, operator `wsp4`) scored with D1 on the Saffire transfer: 0.061 vs
  D1's 0.058.
- In `ASTRA_COVERAGE_19.md` (14 Nov), Astra's all-19 programme has six clean-room family kernels. They
  have no registered all-19 releases. The FLEX source-bound adapter produced zero forecasts on 351 TRAIN
  cases (`ASTRA_FLEX_SOURCE_BOUND_RESULTS.md`).

**Review findings** (`dossier/review/README.md`):

| Finding | About | Verdict | Severity |
|---|---|---|---|
| F-001 | programmed stop as a prospective input | weakened; train recount confirmed | integrity |
| F-002 | gen-5 validation execution boundary | partly confirmed | integrity |
| F-003 | methanol self-cooling | narrow arithmetic check | model |
| F-004 | alias-based raw-file access | potential exposure, none confirmed; access stopped | integrity |
| F-005 / F-006 | mixture identity; generalisation claims | weakened | record |
| F-007 | BASS reached-end row can decrease | refuted as a first-event CIF; fixed only in a new forward | model |
| F-008 | train feedback filters after loading all records | confirmed; new train-only loader | integrity |
| F-009 / F-010 | paired-key collision; event case export | confirmed; comparisons rebuilt by release id | record |
| F-011 | split count (nine split, six held) | refuted count; prose corrected | record |
| F-012 | Astra's own frozen droplet W0 spec | invalid, reproduced by the Rust validator | model |
| F-013 | construction-check preconditions | counterexamples confirmed on synthetic inputs | integrity |

---

## 14. What is served now (ignis-web)

**Served list.** `configs/models/served/served.json`, with 14 releases:
- *ignis-web* `247f43a` (09:32) added the nine generic-law releases;
- `532ebef` (11:42) served the final model of each investigation;
- `e2cb7a8` added bass5;
- `1885d50` added f4-hydrogen and the blend refusal;
- PR 113 merged in `89571e2` (12:33).

| Release | Id | Covers |
|---|---|---|
| `flex_methanol.wsp2` D3 | `55e0cb12…43ce` | FLEX methanol |
| `flex_heptane.wsp2` D2 | `43f7871f…588d` | FLEX n-heptane; transfer to FLEX-2 and CFI |
| `acme_normal.wsp13` F4 (f4-hydrogen) | `4cc2fa60…9436` | CFI-G, Flame Design, s-Flame normal flames; blends refused |
| `acme_inverse.wsp12` F2 | `dff7da0d…594e` | CFI-G and Flame Design inverse flames |
| `bass_front.wsp` D1 | `ac15e49f…855c` | BASS-II fabric; Saffire I to III transfer |
| `bass5` | `050a4ceb…b82d` | BASS and BASS-II plastics and fabrics (post-final label) |
| `bre` D3 (through `lasts`) | `27e2af51…b955` | BRE |
| `cld` D1 | `a03ec603…2d4f` | CLD Flame (chosen after the final run) |
| `efield3` | `8575550b…6f04` | E-FIELD |
| `slice2` s1 | `d586d0d3…0b61` | SLICE |
| `spice` P3 | `000b88b8…98a0` | SPICE |
| `samer` S1 | `e33ee766…1b5f` | SAME-R |
| `same` W0 | `e1362b55…2062` | SAME |
| `daft` W0 | `061f93de…5e5d` | DAFT |

**Serving behaviour.**
- Model families are off unless `IGNIS_ENABLE_MODELS=true` (`docs/CONFIGURATION.md`).
- Law releases answer typed scenarios only, and refuse stored runs with the reason.
- The lane ledger (dossier 11.5, 12:19) says the served list, the full registry (485 releases) and a
  bundle are on the VM staging folder.

The live deployment state was not checked for this file.

---

## 15. Lessons

- **Rules written before the look did the selecting.**
  - 13.3 refused three of five steps that improved train.
  - The 13.6 predict-then-check discipline produced the kept second laws: BRE d3, cld3, efield3, SPICE
    P3, SAME-R S1 and bass3, which then held on the test days.
- **Many failures were in the inputs or the coding, not the physics.** Examples:
  - the planned stop read from the script file;
  - the burner limit as a script setting;
  - copied SLICE tube cells;
  - swapped plastics workbook cells;
  - flow-ramped s-Flame tests.
- **Pin a measurement before scoring it.** The FLEX-2 pixel scale is the case.
- **A measured law is not always a better forecast.** The D3 cool-flame rate case. The next heptane
  judge has to be unread data.
- **Published comparators are worth rebuilding.** Fidelity checked against each paper's own numbers
  made the Xi, Irace and Snegirev results like-for-like.

---

## 16. Inconsistencies in the record

- COVERAGE_19.md, "What is not done", still says the test days of the seventh-generation
  investigations are unread. Its own table and PROTOCOL 13.7 show they were spent at 11:35.
- Some rows of COVERAGE_19.md still name earlier models (e.g. BRE W0, `bass1` D1). The served list and
  PROTOCOL 13.6 to 13.8 are the current authority.
- The `note` field of `served.json` still cites lane commit `e4477d6` for its texts. The served
  releases were updated later (`532ebef`, `e2cb7a8`, `1885d50`).

# Burner flames and fabric burns beside the published models

15 November 2026, post-hoc, after the final run (PROTOCOL 12.11). The same recipe as the Xi et al. droplet
head-to-head (`RESULTS_POINT_METRICS.md`): full text, rebuild the published predictor from its equations, find
the paper's own NASA runs in our data, reproduce the paper's numbers, then score against the adopted releases on
the same cases. **Nothing is adopted or refused, and no IGNIS weight was read, fitted or tuned.**

Research: three Opus 5.5 medium agents; their reports are in `docs/wsp/feedback/published-acme-bass/`, extracted
from the task transcripts. I re-ran every script named below and checked the numbers quoted here against
the outputs.

## What the literature actually publishes

| Family | Paper | What it predicts | Can it forecast our target? |
|---|---|---|---|
| Flame Design (PSI-10, ethylene) | Irace et al., PCI 39 (2023) 1905 and "On the existence of steady-state…" PCI 39 (2023); Irace et al., CNF 229 (2021); read as chapters 4 to 6 of Irace's 2022 WashU dissertation | Extinction radius r_ext = √(ṁ_R / 4πJ), J = 0.20 (fuel) / 0.68 (O₂) g m⁻² s⁻¹; steady radius r_ss; **R = r_ss / r_ext > 1 ⇒ the flame goes out** | *Whether*, yes. *When*: only from their transient CFD driven by each test's measured burner temperature, which is not a forecast |
| ACME CFI-G (PSI-159, alkanes) | Kim et al., PCI 39 (2023) 1647; Kim MS thesis 2022; Santa et al., CNF 151 (2007) | Kim: quasi-steady flame radius. Santa: extinction from a detailed numerical code | No published *when*. *Whether*: only by carrying Irace's ethylene R rule to alkanes |
| BASS (PSI-25/26, SIBAL fabric) | Zhao & T'ien (CNF 2015, PCI 2017; open as NASA/TM-20210011385 §7.1), Ferkul et al. 2012, Li & Liao 2021, Budzinski & DesJardin 2023, NASA PSI-62 | Spread rates measured on 10 BASS-II burns; Ferkul's linear fit Vf = 0.20·U + 0.56 mm/s (2 cm sample); others are CFD, PMMA, or carry no constants | A constant-rate front from a published rate, yes |

NASA PSI-62 holds one 10 KB metadata archive: no reports, tables or predictions.

## Fidelity of the rebuilds (against the papers' own numbers)

- **Irace** (`tools/wsp/final/posthoc_irace_comparator.py literature`):
  - Steady radii of their CNF 2021 Table 2: within 0.13 mm (1.7 %).
  - Eq. 5.2 against Waddell's 17 measured extinction radii: critical flux 0.199 against their 0.20, mean error 2.2–2.6 %.
  - R rule: 6 of 6 cases in their steady-state Table 1 agree.
- **Kim** (`posthoc_kim_comparator.py`): rebuilt radius against their plotted model, max 0.68 % over 15 flames. Their model and the rebuild are both 11.8 % MAE against measured radius.
- **BASS** (`posthoc_bass_published.py`): our tracks' steady slopes match the published Vf of their burns within about 7 % (one burn the crew called not steady: +18.6 %).

Their runs in our data: every identified Kim/Irace/Waddell flame is a train or validation run. Of the 10 BASS-II burns with a published Vf, 3 are train and 2 are test; the one in validation failed track QC.

## 1. Flame Design test days, Irace's law against F2 (the out-of-sample comparison)

`tools/wsp/final/posthoc_irace_testdays.py` → `runs/final/20261115/posthoc_irace_testdays.json`.
- **Inputs:** the final run's own test-day endpoints and sealed forecasts, read from disk. No new reveal.
- **Check:** F2, F1 and the surrogate re-scored here reproduce `flame-design.json` exactly (max difference 0.0).
- **The comparator** is a step forecast:
  - Irace's R decides whether the flame goes out.
  - The erfc growth clock decides when. **The clock is ours, not theirs** (`lit_spherical_flames.md`).
  - On the stop path, the planned stop F2 read comes first if it is earlier.
- **"R only"** replaces the clock with "goes out at the first horizon", so their criterion can be read without our clock.
- All test flames are ethylene inside their 1 bar domain.

Integrated Brier, case means (extinction / termination / sum):

| Family, path | Cases / tests / days | **F2 (adopted)** | Irace R + clock | Irace R only | F1 |
|---|---|---|---|---|---|
| normal, no stop | 87 / 29 / 3 | **0.068 / 0.145 / 0.213** | 0.137 / 0.200 / 0.337 | 0.340 / 0.200 / 0.540 | 0.115 / 0.147 / 0.262 |
| normal, stop | 83 / 29 / 3 | **0.044 / 0.017 / 0.061** | 0.117 / 0.029 / 0.145 | 0.355 / 0.139 / 0.493 | 0.111 / 0.048 / 0.159 |
| inverse, no stop | 60 / 20 / 1 | **0.000 / 0.274 / 0.275** | 0.000 / 0.815 / 0.815 | 0.750 / 0.815 / 1.565 | 0.000 / 0.274 / 0.275 |
| inverse, stop | 60 / 20 / 1 | 0.000 / 0.002 / 0.002 | **0.000 / 0.000 / 0.000** | 0.750 / 0.620 / 1.370 | 0.000 / 0.040 / 0.040 |

Paired F2 minus Irace R + clock, sum, 95 % bootstrap over days:
- **Normal, no stop:** −0.124 [−0.147, −0.080].
- **Normal, stop:** −0.084 [−0.223, +0.062]. The interval includes zero.
- **Inverse:** one day, no interval.

Per test:
- **Normal flames:** every flame that went out has R > 1 (18 of 18), but so do 8 of the 11 stopped by the operator.
- **Inverse flames:** no test flame went out; 15 of 20 have R > 1.
- **The clock runs late:** median predicted/observed extinction time is 2.25× (e^0.81).

**Reading.**
- **Normal flames, no stop:** on the test days, F2 beats their published law with our clock, and the interval excludes zero over 3 days.
- **Normal flames, stop:** F2 is lower by 0.084, but with 3 days the interval includes zero.
- **Inverse flames, stop:** the comparator scores a perfect 0.000. Every inverse test flame was stopped as planned, and our late clock never fires before the stop. That is not skill at forecasting extinction.
- **Their criterion on its own** (R only) is far worse than either. R > 1 is necessary for extinction on these flames, but it does not separate them from the flames the operator stopped first.

**Limits.**
- This is not their full model. Their burn times (about 20 % accuracy) need the measured burner-temperature history.
- It is post-hoc, on a test that is already spent.

## 2. CFI-G: whether the flame goes out, on train runs only

`tools/wsp/final/posthoc_irace_cfig_occurrence.py` → `runs/final/20261115/posthoc_irace_cfig_occurrence.json`.
- **F2's score per run:** P(extinction first) at 120 s, from the 1 s cutoff.
- **The comparator:** Irace's R with its ethylene constants carried to alkanes.
- **Only CFI-G train forecasts are on disk.** F2 was typed by an author who had read train, so this shows how the published law does on these flames. It does not show that F2 generalises better.

| Family | Runs (went out / stopped) | Always "goes out" | F2 AUC / accuracy / Brier | Irace R AUC / accuracy |
|---|---|---|---|---|
| normal | 71 (54 / 17) | 0.76 | 0.94 / 0.89 / 0.083 | 0.92 / 0.87 |
| inverse | 71 (39 / 32) | 0.55 | 0.995 / 0.93 / 0.033 | 0.60 / 0.65 |

On validation the R rule alone (the Kim agent's dry run) scores:
- **Normal:** accuracy 0.786, exactly the always-"goes out" rate.
- **Inverse:** accuracy 0.79 against a base rate of 0.54.

## 3. BASS: validation only, no test claim

`tools/wsp/final/posthoc_bass_published.py --partition both`; I re-ran it. Tip CRPS in sample widths, 3 validation burns, 11 cases:

| Forecast | Validation error |
|---|---|
| **`bass_front.wsp` D1 (adopted)** | **0.121** |
| Steady rate fitted over the 10 s before the cutoff (their way of measuring Vf) | 0.228 |
| Ferkul 2012 fit with an O₂ ratio scaling (the scaling is ours) | 0.352 |
| Ferkul 2012 fit as published | 0.449 |

- **D1 is lower on all 3 burns** against each forecast: mean −0.108, −0.232 and −0.328.
- **Test:** the test holds 2 burns and 7 cases, so a test comparison has no power and was not run.
- **The oracle on train:** a forecast using each burn's own published steady rate reads the future. It roughly ties D1 on train (0.093 against 0.084).
- **Flow readings:** the investigators' flows (2.4–3.0 cm/s) differ from our fan calibration (1.78–3.55 cm/s over the same steady phases). This is unresolved.

## What this supports, and what it does not

- **Supported (Flame Design):** "On NASA's Flame Design test days, IGNIS forecasts when and why normal ethylene flames end better than the published radiative-extinction law of Irace et al. (2023), paired with a growth clock." Without the planned stop, the 3-day interval excludes zero. With the stop it does not.
- **Supported, weaker (CFI-G, BASS):**
  - CFI-G: on train, IGNIS separates self-extinguishing from stopped flames better than the published R rule. For inverse flames the gap is large: AUC 0.995 against 0.60.
  - BASS: on validation burns, D1 beats every spread forecast that can be rebuilt from published constants.
- **Not supported:**
  - That we beat Irace's CFD. Their burn-time model needs measured burner temperatures and was not rebuilt.
  - Any claim on CFI-G or BASS test data.
  - Kim et al. is a radius model, and IGNIS forecasts no radius: there is nothing to beat.

## 4. Fresh ACME batch: 26 Flame Design tests never scored (frozen, scored 15 Nov)

`tools/wsp/fresh/score_acme_fresh.py` → `runs/fresh/acme/`, report `feedback/published-acme-bass/acme_fresh_score_report.md`.

**About the batch.**
- **Not new sessions.** NASA has no unused ACME session that F2 can score. CFI-G and Flame Design days are all already spent, and s-Flame is helium-diluted, which F2 has no input for. These 26 tests were excluded on days we had already used, so they are fresh *tests*, not fresh *sessions*.
- **Order.** Every forecast was made before any endpoint was measured.
- **Binary check.** The final run's binary had been rebuilt. Before any forecast, the current one was shown to reproduce the CFI-G train chain and all 966 validation-day forwards, bit for bit.

**The three tiers.**
- **A1, 20 tests:** a short constant stage, then a scripted ramp to zero. All 20 ended by operator termination.
- **A2, 3 tests:** constant flow, with the stage file missing. 2 went out (extinction), 1 was stopped.
- **A3, 3 tests:** inverse flames with several flow stages, scored at the end of stage 1.

Integrated Brier sum, case means, F2 minus comparator [95 % over days]:

| Block | Cases | F2 | F1 | Surrogate | Incidence | Irace R + clock | F2 − F1 | F2 − Irace |
|---|---|---|---|---|---|---|---|---|
| A1, no stop | 60 | 1.743 | 1.816 | 1.754 | **1.090** | 1.618 | −0.073 [−0.193, −0.033] | +0.125 [+0.056, +0.221] |
| A2, no stop | 9 | **0.256** | 0.288 | 0.265 | 0.400 | 0.409 | −0.031 [−0.049, 0.000] | −0.153 [−0.342, +0.022] |
| A3, no stop | 8 | **0.764** | 0.764 | 1.007 | 0.819 | 0.962 | 0 | −0.198 [−0.569, +0.025] |
| A1, stop | 31 | 0.273 | 0.088 | 0.060 | 1.062 | **0.020** | +0.185 [−0.025, +0.605] | +0.254 [+0.019, +0.656] |
| A2, stop | 9 | **0.054** | 0.087 | 0.063 | 0.400 | 0.161 | −0.033 [−0.053, 0.000] | −0.107 [−0.345, +0.023] |
| A3, stop | 4 | **0.000** | 0.026 | 0.026 | 0.786 | 0.000 | −0.026 | 0 |

**Reading.**
- **Where flames really went out (A2), F2 is best on both paths,** though the intervals touch zero on 3 tests.
- **A1 shows two real weaknesses.**
  - **Without the stop,** F2 forecasts extinction for flames that the operator's ramp ends. It does worse than the incidence baseline.
  - **With the stop,** F2 loses to F1, the surrogate and Irace. Almost all of the loss is in 5 tests:
    - **19123 L4–L7:** F2's stop reader (`acme_script2.py`) gives 13 s, but the flames ended at about 7 s. The planner adds an 8 s fibre-move wait that the observed flow change does not show.
    - **19115B2:** F2 put extinction before its 20 s stop, and the flame lasted to 22.9 s.
  - On the other 26 A1 stop cases, F2 is as good as or better than F1.
- **Why Irace scores near zero on A1:** with the stop, our late clock simply never fires before the stop.

**What follows.**
- **For the next forward (not tuned here):** the planned-stop reader needs the fibre-move wait checked against the setpoint log of train tests.
- **What F2 lacks:** a mechanism for a commanded ramp. This test was spent once, so it is not to be used for selection.

### 4.1 Planned-stop reader v3 (15 Nov 2026; data-informed, post-hoc)

**The change.** `tools/wsp/ext/acme_script2.py` is now METHOD_VERSION 3.
- **v1 fibre wait:** the 8 s wait is the traverse from the ignition position to the test position. When the two positions are equal, there is no traverse, so no wait (`v1|fiber_move_no_traverse`). This is the same rule as F in v2.
- **Evidence from train:** every train test with the 8 s gap has unequal positions. Its planned stops are within ±0.2 s of the measured flow change, except 19175B7/B4 (−1.9 s) and 19189H1 (+1.8 s).

**Scope.** Checked over all 655 PSI-10 and PSI-159 script files, v3 differs from v2 on **6 tests only: 19123L3–L8**.
- All six are on a spent test day.
- No train, validation or final-run test is affected. The final-run scores stand as they are.

**How it was found.** The rule was found on the fresh-batch tests 19123L4–L7, so it is data-informed. It has **no independent check**: the only tests it changes are the ones that revealed it.

**Fresh A1 batch re-scored** with v3 (`runs/fresh/acme_v3_reader/`; v2 kept in `runs/fresh/acme_v2_reader/`):

| A1, stop path | Cases | F2 | F1 | Surrogate | Irace + clock | F2 − F1 |
|---|---|---|---|---|---|---|
| reader v2 | 31 | 0.273 | 0.088 | 0.060 | 0.020 | +0.185 [−0.025, +0.605] |
| reader v3 | 31 | **0.120** | 0.088 | 0.060 | 0.013 | +0.032 [−0.031, +0.138] |

- **Timing:** v3 puts the stop 1.7–2.0 s before the measured flow change on L4–L7. v2 put it about 6 s after.
- **What remains:** the gap left is mostly 19115B2. There F2's flame law gives 70 % extinction before its 20 s stop, and the flame burned to 22.9 s. That is a flame-law miss on one spent test, and it is not fixed here.

**A ramp mechanism is not added.**
- **Stop path:** when the prefix carries the script, a scripted ramp to zero *is* the planned stop, and F2 already ends the fed phase there. The label rule calls operator termination at the first flow change.
- **No-stop path:** without the script, no forward can know a ramp is coming.
- **Why not a new forward:** it would change nothing that can be scored on unspent data.

## 5. Frolov et al. (2023) and Snegirev (2023) on Flame Design (post-hoc)

`tools/wsp/final/posthoc_frolov_snegirev.py` → `runs/final/20261115/posthoc_frolov_snegirev.json`; report
`feedback/published-acme-bass/frolov_snegirev_report.md`; per-flame transcription `docs/wsp/research/frolov2023_flames.json`.
I re-ran the script; F2, F1 and the surrogate reproduce the stored scores on train, validation and test days
(max difference 0.0).

**Frolov, Ivanov, Frolov, Semenov, *Fire* 6 (2023) 285** is a transient 1-D model that computes burner heating
itself, so it is a real forecast from test inputs.
- **What we read:** their per-flame extinction time windows, from Fig. 6, for flames that go out within 50 s.
- **Their published comparisons:** reproduced for 22018G3, where the model is about 10 s late. For 19189K1, their "41 s against 41 s" does not hold against our endpoint, 52.0 s.
- **Their two flow tables:** the *Fire* flows match NASA's; the *Mathematics* companion's do not and look mis-copied.

| Block | Cases / tests / days | F2 | Frolov (window midpoint) | F2 − Frolov |
|---|---|---|---|---|
| test, normal, no stop | 12 / 4 / 1 | **0.014** | 0.124 | −0.110 (1 day) |
| test, normal, stop | 10 / 4 / 1 | **0.027** | 0.330 | −0.303 (1 day) |
| test incl. flames their model never puts out, no stop | 15 / 5 / 2 | **0.032** | 0.099 | −0.067 [−0.110, +0.103] |
| validation, inverse, no stop | 12 / 4 / 2 | 0.180 | **0.121** | **+0.059 [+0.013, +0.200]** |
| validation, inverse, stop | 12 / 4 / 2 | **0.109** | 0.121 | −0.011 [−0.021, +0.019] |

Median absolute error in extinction time, on flames that went out:
- **Test (4 flames):** F2 3.9 s, Frolov 15.8 s.
- **Validation inverse (4 flames):** F2 22.2 s, Frolov 9.9 s.

**Snegirev, *Combust. Flame* 248 (2023) 112499** is a closed-form critical radius plus a steady-burning test.
- **Only partly reproduced:** his constant C is reproduced, but his radiative fraction is not.
- **Against measurement:** our Eq. 20 is 24 % low against Frolov's measured extinction radii.
- **Assumptions that are ours:** the burner radius of 3.2 mm, the inverse-flame extension, and the clock.

On the test days, F2 minus each Snegirev variant, integrated Brier sum [95 % over days]:

| Path | Cases | F2 − Snegirev, rule only | F2 − Snegirev + clock |
|---|---|---|---|
| Normal, no stop | 87 | −0.327 [−0.744, −0.031] | −0.088 [−0.149, −0.041] |
| Normal, stop | 83 | −0.432 [−0.959, −0.027] | −0.095 [−0.130, −0.058] |

- **Normal flames:** his rule says "goes out" for exactly the same flames as Irace's R.
- **Inverse flames:** his rule says 19 of 20 go out; none did.

**Reading.**
- **Test days:** F2 beats both papers on the normal flames.
- **Against Snegirev + clock:** the intervals exclude zero over 3 days on both paths.
- **Against Frolov:** the comparison rests on 4 flames and 1 day.
- **Inverse validation flames:** Frolov's CFD forecasts these better than F2 when the planned stop is unknown (+0.059 [+0.013, +0.200]). F2's extinction times there are off by a median 22 s. That is a weakness of F2 on inverse flames, which the inverse test day, where no flame went out, could not show.
- **Frolov's lateness:** on normal flames their model goes out about 2.1 times late, like the erfc clock.

## 6. Fresh BASS batch: T13 and T20 (frozen, measured blind, 15 Nov)

Report: `feedback/published-acme-bass/bass_fresh_score_report.md`. Tracks and scores: `runs/dev/media/fabric/fresh/`
(local).

**How the burns were measured.**
- **Top view:** with a new top-view measurer, `tools/wsp/media/fabric/topview.py`. It re-implements the train method, whose scripts were never committed, from the NASA proof sheets.
  - **Checked against train:** on T5, T6 and T16 it agrees with the committed train tracks to within 0.03–0.07 sample widths in the tip median.
  - **Rules set while building it:** three of its rules were set after looking at T5 and T6 stills, before any T13 or T20 track existed.
- **T13 duct view:** now passes QC from the original `.ts` capture. The mp4 still fails QC, and its converted track gives 2 cases.
- **T20 duct view:** refused under QC rule R6, as the 1 cm sample it is.
- **Raw data:** 1.24 GB was downloaded and deleted after measuring.

**Tip CRPS** in sample widths. Each track is 1 burn with 2–3 cutoffs:

| Track | Cases | **D1** | Steady prefix | Ferkul pos12 | Ferkul pos12 + O₂ |
|---|---|---|---|---|---|
| T13 top view (train day) | 3 | **0.158** | 0.184 | 0.222 | 0.193 |
| T13 duct | 2 | **0.186** | 0.211 | 0.274 | 0.226 |
| T20 top view (validation day, 1 cm) | 3 | **0.270** | 0.697 | 0.467 | 0.368 |

**Reading.**
- **Against the published-constant forecasts:** D1 is lower than every one on both burns.
- **By cutoff:** D1 is weakest at the 20 s cutoff. On T13 it scores 0.251 against pos12 at 0.159.
- **Against D1's train level:** both burns score well above it (0.08). T20 also rests on D1's narrow-sample rule, which no train burn supports.
- **Power:** two burns on used days. This is a consistency check, not a skill estimate.
- **What it still needs:** BASS remains unqualified. NASA has no unused SIBAL concurrent burn left.

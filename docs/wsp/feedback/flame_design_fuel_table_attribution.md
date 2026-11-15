Diagnostic only. A perturbation is a question about the model, not a proposed value.

# Flame Design normal flames, train days: seven questions about F1's ethylene row (PROTOCOL 12.3, step 3)

> Written by a scoring agent (claude-opus-5-5) on 15 November 2026.
> **Train days only:** 19115, 19137, 19150, 19156, 19171, 19175, 19182, 19189; 74 ethylene tests, 222 cases. No validation-day test was run or read.
> The conditions and endpoints files were first copied, keeping only train-day rows; the test name was the only field read to decide.
> Chain and endpoints are those of the 12.8 run, unchanged; ethylene runs as itself (`fuel_species`, no indicators).
> Tool: `tools/wsp/ext/fd_fuel_attribution.py`; aggregates in `runs/dev/data_all/work/transfer_acme/fd_fuel/attribution/report.json` and `stop_timing.json` (local).
> Nothing was fitted or searched beyond the seven variants named by the author (ids in `runs/dev/gen6/fuel/diag/variants.json`).

## The variants

| Name | Self-extinction coordinates of ethylene | fuel.s (shift) | T_ad bound |
|---|---|---|---|
| F1 | (0.5, 0.5) | −0.085 | 1900 K |
| zqa | (1, 0), ethane's position | −0.085 | 1900 K |
| zqb | (0.5, 0.5) | 0 | 1900 K |
| zqc | (1, 0) | 0 | 1900 K |
| zqd | (1, 0) | +0.25 | 1900 K |
| zqe | (1.5, −0.5), beyond ethane, away from butane | −0.085 | 1900 K |
| zqf | (1, 0) | +0.5 | 1900 K |
| zqg | (1, 0) | 0 | none |

The surrogate is P3 with ethane's indicators.

- **Refusals.** F1 and zqa–zqf refuse the same 18 cases (6 tests, T_ad 1683–1829 K). zqg and the surrogate refuse none.
- **F1 reproduces 12.8.** On these cases F1 gives the 12.8 train-day numbers exactly: 0.2061 / 0.2905 without the stop, 0.1429 / 0.0790 with it.

## 1. Scores (integrated Brier 6–120 s, case means)

On the 204 cases (68 tests, 8 sessions) that F1 answers. Columns are extinction / operator termination / sum.

| Release | Without planned stop | With planned stop |
|---|---|---|
| F1 | 0.2061 / 0.2905 / 0.4966 | 0.1429 / 0.0790 / 0.2219 |
| surrogate | 0.1272 / 0.2763 / 0.4035 | 0.0889 / 0.0651 / 0.1540 |
| zqa | 0.1519 / 0.2822 / 0.4341 | 0.1040 / 0.0664 / 0.1704 |
| zqb | 0.1973 / 0.2890 / 0.4863 | 0.1315 / 0.0768 / 0.2082 |
| zqc | 0.1431 / 0.2801 / 0.4233 | 0.0959 / 0.0661 / 0.1620 |
| zqd | 0.1180 / 0.2792 / 0.3973 | 0.0806 / 0.0694 / 0.1501 |
| zqe | 0.1065 / 0.2744 / 0.3809 | 0.0750 / 0.0606 / 0.1357 |
| zqf | 0.1282 / 0.2847 / 0.4129 | 0.0939 / 0.0775 / 0.1714 |
| zqg | 0.1431 / 0.2801 / 0.4233 | 0.0959 / 0.0661 / 0.1620 |

**zqg against the surrogate.**

| Cases | zqg | surrogate |
|---|---|---|
| all 222 train-day cases, without stop | 0.1511 / 0.3030 / 0.4541 | 0.1333 / 0.2988 / 0.4321 |
| all 222, with stop | 0.1042 / 0.0856 / 0.1897 | 0.0948 / 0.0832 / 0.1780 |
| the 18 cases F1 refuses, without stop | 0.2407 / 0.5627 / 0.8034 | 0.2021 / 0.5539 / 0.7560 |
| the 18 cases F1 refuses, with stop | 0.1976 / 0.3064 / 0.5041 | 0.1613 / 0.2890 / 0.4503 |

- **On the 18 refused cases,** all held to the flow stop, zqg forecasts extinction by 120 s with mean probability 0.58 without the stop; the surrogate gives 0.54.
- **zqc and zqg** give the same scores on the cases F1 answers; the bound acts only on the refused tests.

## 2. Extinction against F1, by session (without the planned stop)

Each entry is the variant minus F1, case mean within the session; a negative value means the variant is better.

| Release | 19115 | 19137 | 19150 | 19156 | 19171 | 19175 | 19182 | 19189 | Improve / worsen |
|---|---|---|---|---|---|---|---|---|---|
| surrogate | −0.010 | −0.002 | −0.121 | −0.080 | −0.087 | −0.125 | −0.065 | −0.093 | 8 / 0 |
| zqa | −0.005 | −0.002 | −0.071 | −0.061 | −0.069 | −0.099 | −0.040 | −0.058 | 8 / 0 |
| zqb | −0.004 | −0.002 | −0.012 | −0.014 | −0.023 | −0.000 | −0.003 | −0.023 | 8 / 0 |
| zqc | −0.009 | −0.003 | −0.088 | −0.073 | −0.087 | −0.096 | −0.048 | −0.077 | 8 / 0 |
| zqd | −0.021 | +0.001 | −0.147 | −0.105 | −0.118 | −0.133 | −0.023 | −0.113 | 7 / 1 |
| zqe | −0.010 | −0.002 | −0.136 | −0.117 | −0.129 | −0.177 | −0.065 | −0.107 | 8 / 0 |
| zqf | +0.003 | +0.018 | −0.163 | −0.105 | −0.109 | −0.123 | +0.017 | −0.091 | 5 / 3 |
| zqg | as zqc | | | | | | | | 8 / 0 |

Sessions 19115 and 19137 are the air sessions; the other six are at 29–40 % oxygen.

## 3. Centre and spread (one row per test at cutoff 1 s, without the planned stop)

- **Centre** is the median of ln(t50 / t_end), with the interquartile range in brackets.
- **Width** is the median of ln(t90 / t10).
- **"in"** counts the ends inside [t10, t90].

| Release | Extinction, air (16 tests) | Extinction, 36–40 % O2 (11) | Operator termination (40) |
|---|---|---|---|
| F1 | −0.39 [−0.76, −0.07], w 0.78, in 9 | −0.72 [−0.74, −0.68], w 1.34, in 6 | +0.26 [−0.16, +0.55], w 1.76, in 33 |
| surrogate | −0.25 [−0.60, +0.10], w 0.93, in 12 | −0.46 [−0.47, −0.40], w 1.61, in 11 | +0.51 [+0.09, +0.89], w 1.76, in 26 |
| zqa | −0.32 [−0.68, +0.02], w 0.82, in 10 | −0.50 [−0.51, −0.45], w 1.38, in 10 | +0.39 [+0.05, +0.74], w 1.75, in 33 |
| zqb | −0.34 [−0.73, −0.03], w 0.84, in 10 | −0.69 [−0.70, −0.64], w 1.50, in 10 | +0.26 [−0.13, +0.58], w 1.80, in 34 |
| zqc / zqg | −0.27 [−0.65, +0.07], w 0.88, in 11 | −0.47 [−0.48, −0.41], w 1.60, in 11 | +0.47 [+0.08, +0.79], w 1.75, in 30 |
| zqd | −0.09 [−0.51, +0.23], w 1.18, in 13 | −0.24 [−0.26, −0.19], w 2.27, in 11 | +0.46 [+0.06, +0.73], w 2.11, in 36 |
| zqe | −0.25 [−0.60, +0.13], w 0.86, in 10 | −0.27 [−0.28, −0.24], w 1.47, in 11 | +0.57 [+0.27, +0.97], w 1.76, in 28 |
| zqf | +0.11 [−0.26, +0.40], w 1.64, in 14 | −0.03 [−0.06, +0.03], w 2.80, in 11 | +0.31 [+0.03, +0.60], w 2.77, in 35 |

- **Path without the stop.** The termination column has no planned stop, so its centre measures where the flame would have gone out, not when the flow was stopped.
- **With the planned stop,** the two extinction columns move by at most 0.03, except zqf at 36–40 % (+0.06).
- **The termination column with the stop** is −0.19 to −0.21 for every release. Its width ranges from 0.17 (surrogate, zqe) to 0.56 (F1), and 15 to 19 of 40 ends fall inside.

## 4. Tests at 36–40 % oxygen held to the flow stop

There are 39 such tests (117 cases). The table gives the mean forecast probability of extinction by 120 s over those cases.

| Path | F1 | surrogate | zqa | zqb | zqc / zqg | zqd | zqe | zqf |
|---|---|---|---|---|---|---|---|---|
| without stop | 0.664 | 0.530 | 0.580 | 0.666 | 0.576 | 0.529 | 0.490 | 0.498 |
| with stop | 0.369 | 0.248 | 0.291 | 0.348 | 0.271 | 0.265 | 0.226 | 0.301 |

## What the questions answer (about the model, not about values)

- **Coordinates against shift.** Moving ethylene's self-extinction coordinates from (0.5, 0.5) to ethane's (zqa) removes about two thirds of F1's excess extinction loss over the surrogate. Removing the −0.085 shift at F1's position (zqb) removes about a tenth. Doing both (zqc) leaves F1's row about 0.016 above the surrogate.
  - So most of what F1 loses against ethane on these days comes from the butane half of the coordinates, not from the shift.
- **Beyond ethane.** zqe is the only variant below the surrogate on all three scores and on both paths. It improves on F1 in all eight sessions.
- **Positive shifts.**
  - They lengthen the forecast life and widen the band, and help most at 36–40 % O2.
  - At +0.5 (zqf), extinction gets worse in both air sessions and in 19182 (5 better, 3 worse). The air flames are then called late (+0.11).
- **What no variant fixes.** The 36–40 % flames that were held are still given probabilities of about 0.5 of going out by 120 s by every variant, the surrogate included.

## Flow-stop time against the script's stage sum (train days, both families)

The sample is the tests labelled operator termination whose script account is a scripted end. Lateness is flow change minus stage sum, in seconds after the measured onset (the time origin the forecast uses).

| Family, session | Tests | Stage sum (s) | Lateness, median [range] (s) | As fraction of the stage sum, median |
|---|---|---|---|---|
| normal 19150 | 8 | 15–90 | 9.90 [1.88, 10.90] | 0.27 |
| normal 19156 | 3 | 25–69 | 2.01 [1.95, 10.09] | 0.08 |
| normal 19171 | 2 | 35 | 1.75 [1.48, 2.03] | 0.05 |
| normal 19175 | 9 | 25–60 | 8.07 [2.02, 10.06] | 0.14 |
| normal 19182 | 10 | 31–100 | 9.96 [1.76, 10.22] | 0.21 |
| normal 19189 | 3 | 42–45 | 9.85 [−27.49, 11.78] | 0.22 |
| inverse 21328 | 3 | 20–865 | 2.44 [2.33, 2.59] | 0.12 |
| inverse 21349 | 7 | 15–124 | 0.43 [0.06, 2.18] | 0.01 |
| inverse 22024 | 13 | 15–30 | 2.14 [0.39, 3.36] | 0.12 |
| inverse 22035 | 12 | 15–120 | 2.33 [0.27, 2.57] | 0.09 |

**Constant in seconds, not growing with the test.**
- **Normal:** a line through lateness against stage sum has slope 0.057 s per s, and lateness at 30 s and at 90–100 s is the same (about 10 s).
- **Inverse:** the slope is 0.0007 s per s over 15–865 s.

**What the lateness follows.** It takes two values in each family, and which one follows a script setting fixed before ignition, `useBracketedExposures`:

| Family | `useBracketedExposures` | Tests | Lateness (s) |
|---|---|---|---|
| normal | false | 9 | 1.5 to 2.1 |
| normal | true | 26 | 23 at 8.1 to 11.8; 2 at about 2; one at −27.5 |
| inverse | false | 12 | 9 at 0.06 to 0.56; 3 at about 2 |
| inverse | true | 23 | 1.85 to 3.36, median about 2.4 |

- **The outlier.** The −27.5 s case is a flow change well before the stage sum on a test the script calls a scripted end. It was not examined further.
- **Origin.** These times run from the measured onset. The onset falls about 0.8 s (normal) and about 2 s (inverse) after the script's ignition stamp, so lateness from the stamp is that much larger.
- **Against the committee's nodes.** The committee's stop at stage sum plus 0, 1 or 2 s covers the "no bracketing" tests of normal flames. It sits about 8 s early on the bracketed ones, which are the majority on these train days.

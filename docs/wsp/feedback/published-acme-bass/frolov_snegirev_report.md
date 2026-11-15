# Agent report (Opus 5.5 medium, 15 Nov 2026, extracted from transcript): Frolov 2023 and Snegirev 2023 vs Flame Design

I built both comparators, and the check that the scoring reproduces the stored scores passes. On the Flame Design test days, F2 beats both comparators, but most of those comparisons rest on 1 day (test) or 2 days (validation). On the validation days, for the 4 inverse flames Frolov's model puts out within 50 s, the comparator scores better than F2 on the no-stop path. I committed nothing, pushed nothing and switched no branch. No IGNIS weight, holdout file, evaluator vault, `--final-test` or reveal was used.

## Reproduction checks

**Scoring harness.** F2, F1 and the surrogate re-scored here match the stored scores with a maximum difference of 0.0 in every block:
- test days: `flame-design.json`;
- validation days: `fd_f2/report.json`;
- train days: the `ibs_contrib` values in `train_days_cases.jsonl`.

The train-day surrogate is read from its own forecast files in `fd_fuel/artifact`. The `*_dense` copy in the jsonl is on a different time grid and does not reproduce.

**Frolov extraction**
- I re-read Tables 1 and 2 from the MDPI page: they agree with `lit_round2` cell for cell. The grey shading that marks extinction is lost in the HTML.
- **Which model time window each flame falls in (read from Fig. 6):** the 13 flames that go out by 50 s in Fig. 6 are exactly the 13 points of Fig. 7, and each has a calculated extinction radius in the tables. Five more flames (3, 9, 10, 17, 27) have a calculated radius but are still burning at 50 s, so their model time is after 50 s and not published.
- **Fig. 7:** the radius-over-flow values from the table lie on the printed fit, 8 % off on average (worst 27 %, 19206A5).
- **Their published comparisons:**

| Flame | Model goes out | Their last measured point | Our endpoint |
|---|---|---|---|
| 22018G3 | 28.5 ± 0.5 s | 17.7 s | 18.54 s |
| 19189K1 | 41.0 ± 0.3 s | 40.5 s (figure axis ends near 43 s) | 52.03 s (planned stop 55 s) |

  - 22018G3 is reproduced: their model is about 10 s late.
  - 19189K1 is not: the "41 s against 41 s" agreement does not hold against our endpoint, and the 9–12 s gap is unresolved.
  - Fig. 6 also leaves 19189K1 out of the 40 s panel, against Fig. 4's 41 s, so I widened its window to 30–41 s.

**Flow conflict (Fire against the Mathematics companion): resolved in favour of Fire.**
- The Fire flows match our NASA rows: total flow within 3.2 %, ambient oxygen within 0.009 (except 19150G3, 0.338 against 0.358), burner fuel fraction within 0.002.
- The Mathematics flows match nothing:

| Flame | Mathematics flow against ours |
|---|---|
| 19206L6 | −66.7 % |
| 19171D4 | −45.5 % |
| 19189K1 | −72.7 % |
| 19150N1 | ambient oxygen 0.351 against 0.300 |

- 0.660 mg/s is 19115B1's flow, and 0.193 / 0.476 / 1.380 is the 19115M4 row, so the companion table looks mis-copied.
- 19142J3 is a ramped row in the NASA table and cannot be checked.

**Snegirev reproduction: only partly reproduced.**
- C = 0.48^(1/3) = 0.783; he publishes 0.78. Reproduced.
- Specific heat by his Eq. 34: propane 1421 against his 1422. Ethylene gives 1453 against his 1528; 1528 needs an adiabatic flame temperature of 2268 K, not the tabulated 2370 K. That looks like a typo in his table.
- His radiative fraction of 0.59–0.61 is **not reproduced**: I get 0.46–0.73 with his constant specific heat, 0.58–0.68 with Eq. 34.
- Against the 12 measured extinction radii in Frolov's tables, my Eq. 20 is **24 % low** on average. Frolov's own 1-D model is −3 % (7 % mean absolute).

## Mapping to the splits

| Partition | Flames with a model time window | Forecasts on disk |
|---|---|---|
| Test (day 19123) | 19123A2, C1, F1, L1 | F2, F1, surrogate from the final run's local caches |
| Validation inverse (21340, 22018) | 21340M1, 22018J1, G3, G2 | F2, F1, surrogate in `fd_f2` |
| Train (19150, 19189) | 19150G3, 19189K1, 19189J3 | F1 and surrogate only: no F2 forecast of a train day is on disk |

Not scorable:
- 19206A5, G1 and G4 have no data.
- 19206L6, 19142J3 and 21328N5 are not in the transfer set.
- No Frolov flame falls on the validation normal days (19196, 21301) or on the inverse test day (21361).

## Frolov: integrated Brier sum over the same cases

"Windowed" are the flames their model puts out within 50 s; the forecast is extinction at the window midpoint (or the plotted time). "+ never" adds the flames their model never puts out, forecast as no extinction. The early and late window ends are in the JSON: the early end is better on test and validation-stop, worse on train and validation-no-stop.

| Block | Cases / tests / days | F2 | F1 | Surrogate | Frolov | F2 minus Frolov, 95 % day bootstrap |
|---|---|---|---|---|---|---|
| test, normal, no stop | 12 / 4 / 1 | 0.014 | 0.019 | 0.013 | 0.124 | −0.110 (1 day, no interval) |
| test, normal, stop | 10 / 4 / 1 | 0.027 | 0.219 | 0.211 | 0.330 | −0.303 (1 day) |
| test + never, no stop | 15 / 5 / 2 | 0.032 | 0.076 | 0.043 | 0.099 | −0.067 [−0.110, +0.103] |
| validation inverse, no stop | 12 / 4 / 2 | 0.180 | 0.180 | 0.140 | **0.121** | **+0.059 [+0.013, +0.200]** |
| validation inverse, stop | 12 / 4 / 2 | 0.109 | 0.110 | 0.138 | 0.121 | −0.011 [−0.021, +0.019] |
| validation + never, no stop | 18 / 6 / 2 | 0.185 | 0.185 | 0.167 | 0.311 | −0.126 [−0.191, +0.200] |
| train, normal, no stop (F1 is the reference) | 9 / 3 / 2 | – | 0.148 | 0.091 | 0.088 | F1 minus Frolov +0.060 [+0.036, +0.073] |

On the validation and train days the late window end gives the same score as the midpoint, because the forecast horizons are 30 s and 40 s apart.

**Error in extinction time** for flames that went out (median absolute error):

| Partition | Frolov | IGNIS (median time from its forecast) | Frolov's lateness |
|---|---|---|---|
| Test, 4 flames | 15.8 s | F2 3.9 s | about 2.1 times late |
| Validation, 4 flames | 9.9 s | F2 22.2 s | |
| Train, 3 flames | 9.9 s | F1 26.3 s | |

## Snegirev: all test-day cases (sum; extinction / termination in the JSON)

| Block (cases, days) | F2 | Snegirev R only | Snegirev + clock (ours) | Irace + clock | F2 minus Snegirev R only | F2 minus Snegirev + clock |
|---|---|---|---|---|---|---|
| normal, no stop (87, 3) | 0.213 | 0.540 | 0.301 | 0.337 | −0.327 [−0.744, −0.031] | −0.088 [−0.149, −0.041] |
| normal, stop (83, 3) | 0.061 | 0.493 | 0.156 | 0.145 | −0.432 [−0.959, −0.027] | −0.095 [−0.130, −0.058] |
| inverse, no stop (60, 1) | 0.275 | 1.765 | 0.837 | 0.815 | −1.491 (1 day) | −0.563 (1 day) |
| inverse, stop (60, 1) | 0.002 | 1.720 | 0.000 | 0.000 | −1.718 (1 day) | +0.002 (1 day) |

- **Normal test flames:** Snegirev's rule says "goes out" for all 18 that went out and for 8 of the 11 that were stopped. That is identical to Irace's R, so "R only" scores are identical too.
- **Variants:** κ = 4.1 and burner radius 0 change nothing on normal flames.
- **Inverse test flames:** his rule says 19 of 20 go out (15 of 20 with κ = 4.1), but none did.
- **Inverse, stop path:** the 0.000 is the same artefact as Irace's. Every flame was stopped as planned before the late clock fired.

## Assumptions (all flagged in the code)

**Frolov**
- The bracket midpoint is my reading convention, not a time they publish.
- Their model clock starts at ignition; ours starts at flame onset from the radiometer.
- Flames their model puts out after 50 s are left out.
- "–" in their table is read as "the model never puts the flame out".

**Snegirev**
- **Burner radius (`R_s`) = 3.2 mm (ours).** He sets `R_s` = half the burner radius only for the BRE flat burner.
- κ is computed per test from Eq. 35 with the HITEMP values at 1100 K. His 4.1 m⁻¹ BRE average is run as a variant.
- Inverse flames are handled by swapping the burner and ambient streams. **That extension is ours**; he treats burner fuel only.
- The time clock is the erfc clock from `posthoc_irace_comparator.py`, also ours.

**Validation outcomes:** the per-flame table includes individual validation-day outcomes, not only aggregates.

Files are in `D:\dev\SpaceApps-Ardealu`:
- `docs/wsp/research/frolov2023_flames.json`
- `tools/wsp/final/posthoc_frolov_snegirev.py` (run as `python tools/wsp/final/posthoc_frolov_snegirev.py`)
- `runs/final/20261115/posthoc_frolov_snegirev.json`
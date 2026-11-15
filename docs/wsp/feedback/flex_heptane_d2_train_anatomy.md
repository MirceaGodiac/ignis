# Heptane droplet model D2 (third generation): error anatomy on the FLEX train data

> Train partition only; nothing fitted. Written by a delegated analysis agent (claude-opus-5-5) on 14 November
> 2026; the author rechecked the claims it acted on against the train tables. Paths under `scratchpad` are the
> agent's working folder and are not part of the repository.

I've finished the D2 train analysis. Everything is train only: I opened no validation, test or holdout data and changed no repository file. Nothing was fitted; every number comes from finite differences of ±0.25 s window means of D², counts, medians or terciles.

- **Forecasts.** I rebuilt the forecasts by running the D2 forward on the 125 train prefixes. They reproduce the feedback CRPS of every case to about 1e-13 µm.
- **Stage times.** Hand-read stage times come from the D1 audit. I added one: H13H201 loses its visible flame at about 5.5 s, read from where the rate falls (0.69–0.71 → 0.51).

**Headline.** D2's mean train CRPS is 46.2 µm (5770 µm over 125 cases).
- **Problem runs carry 58 % of it.** These are the 2009 soot and glare runs, 248F001 at 2 atm, and C01H101 with its wrong time zero.
- **Measurement artefacts carry 57 % of all CRPS:** the soot and glare runs plus the cases with a tracker artefact in the scored window. Among the 105 non-problem cases, the artefact cases hold 22.5 %.
- **Helium is the largest physics error.** It is 7 % of the total and 17 % of non-problem error, all from one session.
- **The 90 % band is too wide.** On clean cases it covers 1.00 in every lead band, and the bands are 2.5–4 times wider than the realised errors.

## 1. Error anatomy

How I sorted the cases:
- **Problem runs** form their own class.
- **Artefact in window** means the scored window holds frames no droplet can produce (D² falling faster than 2 mm²/s over 0.2 s, apparent growth, or a value frozen for 4 or more frames) and those frames carry at least 20 % of the case's CRPS.
- **Every other case** is classed by the hand-read stage times.

Share of total CRPS:

| class | n at cutoff 1/3/5 s | sessions | 1 s | 3 s | 5 s | all | mean µm | band moved onto the data, µm |
|---|---|---|---|---|---|---|---|---|
| hot through the horizon | 11/10/7 | 9 | .029 | .017 | .014 | .060 | 12.4 | 9.0 |
| hot ends → cool flame | 14/6/3 | 9 | .049 | .017 | .008 | .074 | 18.6 | 10.9 |
| hot ends → cool, cool also ends | 0/4/1 | 4 | – | .016 | .009 | .025 | 28.4 | 18.0 |
| hot ends → helium stage | 3/3/3 | 1 | .017 | .022 | .033 | .072 | 46.2 | 29.9 |
| hot ends → nothing (burn-down) | 2/3/3 | 3 | .012 | .017 | .016 | .044 | 31.7 | 22.9 |
| cool flame at the cutoff | 0/3/6 | 5 | – | .013 | .020 | .033 | 21.2 | 8.5 |
| cool flame ends in the horizon | 0/1/4 | 3 | – | .003 | .017 | .020 | 23.1 | 14.0 |
| artefact in the scored window | 7/6/5 | 6 | .025 | .025 | .045 | .095 | 30.5 | 11.1 |
| problem run | 8/7/5 | 5 | .370 | .182 | .024 | .577 | 166.4 | 12.6 |
| all | 45/43/37 | 20 | .502 | .313 | .185 | 1 | 46.2 | 13.1 |

- The last column is the CRPS left after moving the whole band so its median sits on the observation. What remains is the part due to width alone.
- **"Already out" is empty.** The one candidate, H13H101@5 (0.7 s after the flame went out), is counted under helium.
- **By lead band:** 0–2 s holds 26 % of the total, 2–5 s 27 %, 5–10 s 47 %. For non-problem cases the per-point CRPS is 8 / 17 / 35 µm at a 1 s cutoff and 10 / 24 / 47 µm at 5 s.
- **Artefact frames themselves** (±0.25 s around each flagged frame) carry 18.5 % of all CRPS.

**The 20 worst cases (65 % of the total):**

| # | case | CRPS µm | cause |
|---|---|---|---|
| 1 | 195F002@1 | 1023 | soot shell read as droplet; median 0.7–1.2 mm high |
| 2 | 195F001@3 | 744 | soot; apparent rate 8 mm²/s; median 0.6–0.9 mm low |
| 3 | 20CAL13@1 | 689 | luminous flame hides the droplet; prefix D0 1.15 against 1.82 mm |
| 4, 5, 9 | 248F001@1, @3, @5 | 269, 184, 65 | 2 atm, 70 % CO2: cool-like phase runs faster than the law; outside what was written |
| 6 | C02H202@5 | 108 | one scored frame, a frozen tracker value of 0.493 mm; level carried down to 0.36 |
| 7, 14 | H12H101@5, @3 | 91, 48 | helium: flame out at 7.53 s, D2's median puts it at 11.4 s; then the helium stage; bias −303 µm at 5–10 s |
| 8, 12, 20 | H13H201@5, @3, @1 | 76, 53, 36 | helium: flame out at 5.5 s, D2 says 8.1–8.5 s |
| 10 | 195F001@1 | 62 | soot |
| 11 | C10H101@5 | 55 | droplet leaves the view (3.39 → 3.14 mm in 0.25 s) |
| 13 | C02H201@5 | 49 | cool flame ends 2.1 s early in D2: plateau observed at 1.00 mm, D2's d_ce 1.24 mm |
| 15 | C01H302@5 | 46 | leaves the view (2.50 → 1.75 mm in 0.5 s) |
| 16 | C04H201@3 | 46 | cool-flame offset read in the dip after the flame went out: D2 uses 0.30 mm²/s, observed 0.36 |
| 17–19 | H12H201@1, @3, @5 | 43, 42, 41 | burn-down in 30 % He: the extinction mixture holds the median 0.2–0.3 mm above a droplet still burning |

## 2. Stage timing

D2's implied time is when its probability of a hot flame (or of no flame, for the cool-flame end) crosses 0.5. Δ is D2 minus the hand-read time. Clock units are s = t·(1 mm²/s)/D0².

| group | rows (runs, sessions) | Δt median [IQR] s | Δs median [IQR] |
|---|---|---|---|
| hot-flame end → cool flame | 25 (13 runs, 10 sessions) | −0.06 to +0.04 [−0.29, +0.41] | +0.004 [−0.03, +0.04] |
| hot-flame end, helium | 8 (3 runs, 1 session) | +2.7 [+2.4, +3.4]; H12H101@1 is past the horizon | +0.28 [+0.22, +0.34] |
| hot-flame end, burn-down (H12H201) | 2 | −0.66 | −0.08 |
| cool-flame end | 11 (5 runs, 5 sessions) | −0.61 [−1.97, −0.36] | −0.10 [−0.32, −0.06] |

**Clock law.** Across 16 non-helium radiative runs, ln(s_ext/s_R) has median +0.03 and robust sd 0.15. By D0 class it is +0.06 / +0.03 / −0.01, so D2's size term removed D1's size ordering. The outliers:
- C09H501 (+0.37) and C09H101 (+0.29): 23 % O2 and 25 % CO2 at 0.7 atm. D2 puts C09H501's extinction 2.5 s early.
- The helium runs: −0.21, −0.40, −0.32.

Cool-flame end by run:
- C02H201 −2.0 s, from the plateau diameter.
- C03H601 −0.4 / −1.0 s.
- N02H203 −0.6 / −0.4 s.
- N03H101 +0.8 / +2.1 s.
- N04H101 is undefined: D2 reads "no flame" at 3 s.

**Timing or rate?** I split the median's D² error at the end of the horizon exactly into intervals (level, hot rate before, the window between the observed and implied times, rate after) and shared each case's CRPS out by those shares. Over the 40 cases with a stage end in the horizon (1134 µm):

| class | timing | rate in the next stage | hot rate before | level |
|---|---|---|---|---|
| hot → cool | 75 µm (19 %) | **216 (54 %)** | 82 | 27 |
| hot → helium | **243 (62 %)** | 85 | 56 | 10 |
| cool ends (both classes) | **123 (49 %)** | 72 | 33 | 18 |
| hot → nothing | 40 | 3 | 40 | 1 |

Outside helium, the hot-flame end is timed well, and the error is the rate of the next stage. After a flame goes out inside the horizon, the cool-flame rate D2 applies is within median 5 % of observed (23 cases). The misses are:
- **0.7 atm:** C08H101 −0.25 and C09H501 −0.17 in ln (D2 too fast).
- **14 % O2:** N04H101 −0.61.
- **The dip when the flame goes out early (before about 3 s):** C03H501, C04H101, C04H201, N04H101 burn at 0.40–0.63 of the cool rate in the first second. Late extinctions show no dip (0.87–1.17).

## 3. Helium

Rate of D² in 1 s windows after the visible flame is lost, against D2's no-flame law at the same diameter (observed / D2, mm²/s):

| s after flame loss | H12H101 (30 % He, 21 % O2, D_e 2.33 mm) | H13H101 (35 %, 20 %, 3.32 mm) | H13H201 (35 %, 20 %, 2.41 mm) | median, as share of the hot rate just before |
|---|---|---|---|---|
| 0–1 | .50/.15 | .58/.19 | .57/.15 | 0.83 |
| 1–2 | .39/.12 | .53/.16 | .44/.13 | 0.64 |
| 2–3 | .30/.10 | .37/.13 | .33/.11 | 0.48 |
| 3–4 | .24/.09 | .28/.12 | .26/.09 | 0.38 |
| 4–5 | .19/.08 | .23/.10 | .16/.08 | 0.26 |
| 6–7 | .13/.06 | .14/.08 | .12/.06 | 0.18 |
| 10–11 | .07/.04 | .14/.06 | .08/.04 | 0.12 |
| 13–14 | .04/.04 | .09/.05 | .02/.04 | 0.06 |

- **Shape.** The excess over the long-time rate falls by e roughly every 3.3–3.5 s, close to D2's own decay time at these sizes (3.9–4.4 s). Its amplitude is 3.3–3.8 times D2's: about 0.8 of the hot rate before the loss, against D2's 0.15–0.19. The larger droplet (3.32 mm) runs higher late, 0.13–0.20 against 0.06–0.13.
- **Simplest law in the forward's terms.** Keep k_inf·r^m + k_hot·r^h·exp(−t/τ), and after a radiative extinction with no cool flame scale the k_hot term by a written helium factor. In `emit` that branch currently passes hot_scale = 1.0. An alternative is to tie the amplitude to the hot rate at extinction, written as a share. Add a helium term to the radiative clock (ln −0.2 to −0.4) together with this law, not alone; D2's edits note that the clock alone made these forecasts worse.
- **Weaker helium runs.** There is no train evidence at 5–25 % He. H01H102 (5 %) and H02H101 (10 %) leave the view while still burning, and H19H101 and H19H201 are refused as atmosphere conflicts.
- **Harm to other runs.** None if the law is scoped to helium. Unscoped, it would also change N04H101 (p_cool 0.5) and the state filter's no-flame mean, since rho_e uses the same amplitude. The filter must keep its own no-flame mean, or the no-flame state becomes hard to tell from a cool flame.

## 4. Hot flame → cool flame or → nothing

Every case with an observed outcome is on the right side in D2.

| what followed | runs (atmosphere, p, D0) | D2 P(cool flame after the end) |
|---|---|---|
| cool flame | N02H101 (.16, 1 atm, 3.34 mm), N02H203 (.16, 2.52), N03H101 (.15, 2.55), C04H101/C04H201 (.17/.20 CO2, 3.76/2.90), 301F001 (.18, 3.43), C03H501/C03H601 (.18/.15, 3.74/2.40), C02H101/C02H201 (.19/.10, 3.40/2.48), C01H302 (.20/.05, 3.18), C08H101, C09H501, C09H101, C10H101 (0.7 atm, CO2), 194F001 (0.72 atm, "not clear") | 0.70–0.95 (C08H101 0.70, C02H201 0.79, C01H302 and 194F001 0.81, N03H101 0.82) |
| weak cool flame | N04H101 (.14, 4.16) | 0.50, on the edge |
| helium stage, no cool flame | H12H101, H13H101, H13H201 | 0.02–0.05: correct, but the stage that follows has no law |
| burned to nothing | C01H201, C02H202, C03H701, C09H601, N01H502, H12H201 (D0 1.35–2.86 mm, x_exc −0.14 to −0.38) | 0.01–0.04 |
| unseen (left the view) | 061F002, 301F002, C08H601, H01H102, H02H101, N01H101, N01H102, N01H201, N01H202, N04H201, 209F001 | 0.01–0.49 |

**Where D2 goes wrong** is the rate after the change, not the class:
- the helium stage;
- H12H101's class, pi_r 0.57, so 43 % of its mass sits on a diffusive burn-down;
- N04H101 at 0.50.

Observed extinction diameter of the burn-downs, De/D0 against D2:
- C01H201 0.37 against 0.30
- C02H202 0.30 against 0.35
- C03H701 0.35 against 0.39
- C09H601 0.25 against 0.34
- H12H201 0.24 against 0.28
- N01H502 0.24 against 0.27

## 5. Precursors in the prefix

**Hot-flame end.** Over 34 non-helium cases in 11 sessions I took ln(time left observed / time left at D2's radiative median given survival to the cutoff); the robust sd is 0.26.
- **By r_end tercile:** low +0.08, mid +0.10, high −0.13.
- **By the end/mid trend:** +0.08, +0.05, −0.13.
- **By r_late:** +0.03, −0.15, +0.07, so the order is not kept.

Each tercile spans −0.6 to +0.75. Nothing usable beyond D2's clock. The helium runs sit in the high-r_end tercile (r_end 1.02–1.16) with residuals −0.25 to −1.8, but that is a regime effect in one session.

**Cool-flame end.** Over 10 cases in 5 sessions, a low end-rate/law tercile goes with a plateau far earlier than D2 predicts (N04H101 −32 s, C04H101 −6 s). This is D2's own offset applied in the dip, not new information. An end/mid trend above 1.4 (C04H101@3 1.46, N04H101@3 1.75) marks a prefix coming out of the dip. That is 2 cases in 2 sessions; too few.

**Cool-flame offset.** For a cool flame at the cutoff, the law alone predicts the next 5 s better than k_cool·e^b_cool:
- median |ln| error 0.075 for the law against 0.105 for the offset version, over 13 cases in 6 sessions;
- the offset helps in only 4 of 13: C03H501@5, C04H101@3, N04H101@3 and N04H101@5;
- it hurts when the cutoff is ≤ 0.6 s after the flame went out (301F001, C02H101, C03H601: too fast) and in the dip (C04H201@3: +0.22).

## 6. Artefacts seen from the prefix

Thirty-two cases are artefacts: the soot and glare runs plus the artefact-in-window class. Together they carry 3279 µm.

| prefix signature | fires on artefact cases (µm) | fires on clean cases (µm) |
|---|---|---|
| frames gated by the validity gate > 0 | 14/32 (2730) | 6/93 (117): 301F002 ×3, N01H502 ×3 |
| artefact frames in the last 1 s of the prefix | 11/32 (2692) | 5/93 (103): C03H501@1, C09H101@1 and @3, N01H502@1, N04H101@1 |
| prefix D0 / sheet D0 off by more than 5 % in ln | 4/32 (1763) | 1/93 |
| union of gated, last-1-s artefact, r_end outside (0, 3) | 16/32 (2857) | 12/93 (223) |

The 16 artefact cases that no signature catches carry 422 µm: droplets leaving the view (194F001, C01H302, C10H101) and frozen tracker values after disruption (C01H201, C02H202@1 and @3, C03H701). The prefix diameter cannot show these, and the bundle carries no position track. Frame-to-frame jitter is useless here (median 0 µm, since the series are smooth).

## 7. Calibration

Coverage of the 50 % / 90 % bands, with mean 90 % width and median |median error| in µm:

| class | 0–2 s | 2–5 s | 5–10 s |
|---|---|---|---|
| hot through | .96/1.00 (93; 5) | .92/1.00 (168; 14) | .85/1.00 (351; 20) |
| hot → cool | .88/1.00 (88; 7) | .67/1.00 (137; 20) | .66/1.00 (287; 38) |
| hot → helium | .97/1.00 (104; 11) | .70/1.00 (297; 29) | .94/1.00 (827; 78) |
| hot → nothing | .98/1.00 (128; 6) | .98/1.00 (430; 22) | .87/1.00 (961; 102) |
| cool at cutoff | .74/1.00 (86; 9) | **.46/.98** (114; 36) | **.44/.89** (201; 39) |
| cool ends | .85/1.00 | .64/1.00 | .84/1.00 |
| problem runs | .46/.71 | .54/.68 | .57/.67 |
| clean (no problem, no artefact) | .91/1.00 | .77/1.00 | .75/.99 |

- **Too wide almost everywhere on clean cases.** 13 of 14 clean sessions have 50 % coverage above 0.6 and 90 % coverage at least 0.97. The 90 % band is 2.5–4 times what the realised errors need for hot-through and burn-down cases.
- **Hot rate spread.** In hot-through cases the realised ln-rate error of the median over the horizon has robust sd 0.026 (20 cases), against D2's s_z of 0.11.
- **Mixture width.** The burn-down and helium bands (430–961 µm) come from the mixture over extinction time and diameter, not from the rate spread.
- **Too narrow only for a cool flame at the cutoff,** and that is the offset's bias, not width.
- Problem runs are under-covered.

## Proposals, ranked

The effect sizes are estimates from the location-versus-width split and simple arithmetic on the bands; no modified release was built.

1. **Widen the band when the prefix shows an artefact.** The signature is gated frames > 0, artefact frames in the last 1 s, or r_end outside (0, 3): mix in a broad component.
   - Support: 195F001 and 195F002 (2009-11-09), 182F001 and 182F002 (2009-11-13), 20CAL13 and 30CAL06 (2009-04-10), C02H202@5 (2011-04-20), N01H101@1 (2011-08-29). Five sessions.
   - Contradicted by: 12 clean cases (223 µm), mainly 301F002 and N01H502.
   - Effect: about 200–400 µm, almost all from three 2009 soot and glare cases. It is measurement handling, not physics.
2. **Narrow the hot-flame rate spread** (unc.s_pop and s_floor) toward the realised level, with a margin.
   - Support: 13 of 14 clean sessions over-covered; hot-through ln-rate sd 0.026 against 0.11; the 90 % band also covered 0.97 on unseen sessions.
   - Contradicted by: cool-at-cutoff cases, which are already narrow. Do proposal 4 first.
   - Effect: about 100–250 µm.
3. **Write a law for the helium stage, scoped to helium,** and add the helium clock term at the same time.
   - Support: H12H101, H13H101, H13H201. **One session.**
   - Contradicted by: no train run.
   - Effect: up to 147 µm of median error plus part of the 269 µm width; about 100–250 µm.
4. **Start of a cool flame:** no end-rate offset within about 2 s of D2's implied flame-out time (or set enc.cool_gain toward 0), and a written dip of about half the cool rate for 1–2 s after an early (s ≲ 0.25) extinction.
   - Support: 301F001, C02H101, C03H601, C04H201, C03H501, C04H101, N02H101; six sessions.
   - Contradicted by: N04H101 and C04H101@3, where the offset helped.
   - Effect: about 40–80 µm.
5. **Sharpen the diffusive-extinction hazard** (diff.sharp). Observed De/D0 spreads by about ±0.05 in progress; the hazard spreads by about ±0.07.
   - Support: C09H601, H12H201, N01H502, C02H202, C03H701, C01H201; five sessions.
   - Contradicted by: C01H201 and C09H601, which sit 0.07–0.09 off on opposite sides, so sharpen by no more than about 2 times.
   - Effect: about 40–70 µm.
6. **At 0.7 atm in 23–25 % O2 with CO2:** the radiative clock runs later and the cool flame burns slower.
   - Support: C09H501, C09H101, C08H101. **Two sessions for the clock and two for the rate, both fewer than three.**
   - Contradicted by: 194F001 (0.72 atm, no CO2, clock −0.08) and C08H101's clock (+0.07).
   - Effect: about 30–60 µm.

Leave alone: the class law (no case on the wrong side), the hot-flame end time outside helium, and the cool rate at 1 atm and 15–19 % O2.

## What I could not determine

- H13H201's exact flame-out time: 5.5 s from the rate drop, against "about 5 s" in the narrative; the sheet's Dext contradicts both.
- What the helium stage is physically. The narratives say "no rapid vaporization", but the diameter clearly keeps falling.
- Whether lower helium fractions show the stage; no observable run exists.
- Signatures for droplets leaving the view or a tracker freezing after disruption (422 µm), with only the diameter in the prefix.
- The actual CRPS gain of any proposal; no modified release was assembled.
- Cool-flame end timing rests on only 5 runs.
- The timing split uses the median of a mixture as if it were one path, so it is approximate.

Scripts and outputs are in `D:\Caches\Temp\claude\23fcab7049aa850b\bd40257b-5cbd-4ff9-b44f-69568de1670b\scratchpad\analysis4\heptane\`. The per-case table is `cases.json`; `tables_out.md` and `timing_out.md` hold the tables for items 1 and 2.

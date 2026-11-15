# Heptane droplet model D1: audit against the FLEX train data

> Train partition only; nothing fitted. Written by a delegated analysis agent (claude-opus-5-5) on 14 November 2026;
> the author rechecked the claims it acted on against the train tables. Paths under `scratchpad` are the agent's
> working folder and are not part of the repository.

## Summary (≤400 words)

The basis: the train partition has 125 scored cases in 20 sessions; without the 12 problem runs that leaves 105 cases from 37 runs in 16 sessions. I rebuilt every prefix from the train records and ran the read-only `ignis wsp forward`; the result reproduces the feedback CRPS for all 125 D1 and W0 cases. My Python version of D1's laws matches the forward's diagnostics on all 125 cases to 5e-16.

**Error budget.** D1's mean CRPS is 49.3 µm over all cases and 29.9 µm without problem runs (W0: 40.2, D²-law: 58.1).
- Measurement artefacts (soot, glare, droplet leaving the view) carry 42 % of all D1 error.
- Without problem runs the error is spread out; the 25 worst cases hold 45 % of it. The largest cause is the cool-flame rate (12 %), then H12H101's wrong class (helium, one session), droplet gone, radiative timing, and hot rate too fast.
- Hot-flame cases make up 71 % of non-problem error at about 27 µm each. Their medians are good; their bands are wide.

**Best-supported proposals:**
1. **Cool-flame offset (P1; 7 sessions, 17 cases).** The cool-rate law is right to a median 6 %. The offset b_cool is taken from a prefix window that holds the hot flame or the hot-to-cool dip, so the rate actually used misses by 21 %. Six such cases carry 11 % of non-problem CRPS. Take b_cool only from a window inside the cool phase, or shrink enc.e_clip to about ±0.1.
2. **Early hot rate (P2; 13 of 16 sessions).** The law is about 15 % too slow in the first 1–2 s (ln residual +0.16 at 0.3–1 s, +0.07 at 1–2 s) and right after that. Dropping the fast heat-up term (rate.w2[4] from −0.25 to about 0) fixes most of it. It also removes the positive run offset b, now positive in 24 of 28 cases at the 1 s cutoff, which makes forecasts too fast.
3. **Radiative clock (P3; 6 same-atmosphere pairs, ≥5 sessions).** At a fixed atmosphere, ln s_ext falls 2.9–3.6 per e-fold of D0, against the law's 2.0. Droplets of 2.4–2.6 mm lose the flame +0.28 to +0.45 (ln) later than the law. Add about −1 per e-fold of D0 to ln s_R, with b0 re-anchored so 3.4 mm droplets keep their current value.

Two further proposals: P4, lower s_pop from 0.20 toward 0.10–0.12 (bands too wide at the 1 s cutoff in 16 of 16 sessions); and P5, slower decay of evaporation after the cool flame.

The class law puts no non-problem run on the wrong side, and no alternative variable is supported. The state filter is right in 82 of 82 hot and 11 of 12 cool cases.

**Not determined:**
- The time of visible-flame extinction for C10H101 (it is "reached" only while the droplet leaves the view) and for H13H201 (the sheet contradicts the narrative).
- Burn-down extinction diameters for runs that left the view.
- Whether 209F001 and H12H101 lost the flame to radiation.
- The CRPS gain of any proposal: that needs assembling a new release, which this audit may not do.

---

# Full report: D1 against the train partition

## Method
- **Forecasts.** The feedback file holds only metrics and diagnostics, so I rebuilt each case's causal prefix from `data/work/records/flex/observations.jsonl` (train ids only, time and availability ≤ cutoff) and ran `ignis wsp forward` for D1 and W0 with `{"horizon_s":10}`. Recomputed CRPS and lead-time bias match the feedback file for all 250 cases (largest relative CRPS difference 2e-15; largest bias difference 0.5 µm, rounding).
- **Laws.** `wsp.py` reads the W0 edits file, then applies only the statements D1 names. It matches the forward's `k_model_late` and all laws on 125 of 125 cases.
- **Rates.** K is a finite difference of ±0.25 s window means of D². No fitting anywhere.
- **Stage times.** t_ext is when the 5-frame running mean of D first reaches the sheet's Dext. Plateau t_pl is the first time after t_ext + 1 s at which the 1-s forward K drops below 0.15 mm²/s twice in a row.
- **Artifact store.** It holds no train heptane forecasts, only 117 validation-run artifacts. I read only their `context.json` run ids and opened none.

## A. Error budget

CRPS by cutoff, all cases (µm, mean / median):

| cutoff | n | D1 | W0 | D²-law |
|---|---|---|---|---|
| 1 | 45 | 68.9 / 25.1 | 78.3 / 38.7 | 92.2 / 40.4 |
| 3 | 43 | 39.9 / 26.8 | 52.5 / 26.6 | 59.9 / 29.2 |
| 5 | 37 | 36.3 / 30.3 | 40.6 / 31.1 | 71.4 / 35.9 |

CRPS by cutoff, non-problem runs (16 sessions):

| cutoff | n | D1 | W0 | D²-law |
|---|---|---|---|---|
| 1 | 37 | 26.3 / 23.4 | 39.9 / 38.7 | 55.1 / 39.9 |
| 3 | 36 | 26.5 / 24.6 | 37.6 / 24.0 | 43.5 / 27.9 |
| 5 | 32 | 37.7 / 31.0 | 43.6 / 32.6 | 78.0 / 36.3 |
| all | 105 | 29.9 / 26.5 | 40.2 / 32.0 | 58.1 / 34.0 |

The 20 problem cases average 151 µm (median 33).

CRPS by session (D1 mean, W0 mean, D²-law mean; n cases):

| session | n | D1 | W0 | D²-law |
|---|---|---|---|---|
| 09-04-10 | 4 | 190.4 | 187.3 | 209.4 |
| 09-07-01 | 3 | 99.4 | 85.1 | 63.9 |
| 09-11-09 | 6 | 289.5 | 297.5 | 323.6 |
| 09-11-13 | 4 | 29.6 | 28.5 | 46.4 |
| 10-01-12 | 3 | 32.1 | 20.6 | 19.5 |
| 10-01-29 | 3 | 21.8 | 22.2 | 22.1 |
| 10-03-11 | 6 | 18.3 | 23.6 | 24.8 |
| 10-06-14 | 3 | 18.1 | 22.9 | 25.9 |
| 10-06-15 | 3 | 25.7 | 25.8 | 23.7 |
| 10-06-29 | 6 | 17.2 | 18.2 | 19.2 |
| 10-11-18 | 9 | 37.2 | 49.7 | 40.8 |
| 11-02-21 | 11 | 28.7 | 41.6 | 80.8 |
| 11-04-20 | 15 | 37.8 | 45.6 | 58.4 |
| 11-08-17 | 6 | 42.7 | 51.7 | 76.0 |
| 11-08-29 | 13 | 15.1 | 17.5 | 25.2 |
| 11-08-30 | 3 | 30.6 | 46.8 | 129.6 |
| 11-09-01 | 7 | 25.6 | 31.2 | 71.7 |
| 11-09-13 | 2 | 13.3 | 18.8 | 10.7 |
| 11-09-24 | 6 | 25.8 | 46.2 | 44.8 |
| 11-11-22 | 12 | 51.5 | 88.0 | 131.5 |

**25 worst cases, all runs (62 % of D1 CRPS).** Cause totals: measurement artefact 9 cases / 2584 µm (42 % of all D1 CRPS); cool-flame rate 6 / 525 (9 %); wrong class 2 / 161; droplet gone 1 / 153; radiative timing 2 / 123; hot rate too fast 2 / 101; band width 2 / 94; cool-flame extinction diameter 1 / 56. The four problem cases in the top 4 are artefacts, except 248F001@1 (2 atm, cool-flame rate out of support).

**25 worst non-problem cases (45 % of non-problem CRPS):**

| # | case | CRPS µm | cause | evidence |
|---|---|---|---|---|
| 1 | C02H202@5 | 153 | droplet gone | flame out at 4.7 s; tracker holds 0.493 mm; level carried to 0.33 |
| 2, 5, 25 | H12H101@5, @3, @1 | 102, 58, 39 | wrong class | pi_r 0.55; out at 7.5 s at 2.33 mm, no cool flame |
| 3, 16 | C04H201@3, @5 | 81, 47 | cool-flame rate | b_cool from the dip: used 0.26–0.31 against observed 0.35 |
| 4, 6 | H13H201@5, @3 | 65, 58 | radiative timing | He 35 %: t_R 6.9 s, out at ~5 s |
| 7 | C02H101@5 | 58 | cool-flame rate | b_cool from a hot window: used 0.46 against 0.37 |
| 8 | C03H601@5 | 57 | cool-flame rate | used 0.46 against 0.33 |
| 9 | N01H502@1 | 56 | artefact | 22 early frames gated, D0 5 % high |
| 10 | C02H201@5 | 56 | cool-flame extinction diameter | plateau 1.00 against d_ce 1.23 |
| 11, 17 | C10H101@5, C01H302@5 | 54, 46 | artefact | droplet leaving the view |
| 12, 14 | H12H201@3, C01H201@1 | 53, 48 | hot rate too fast | b about +0.1 learned from the early law deficit |
| 13, 18 | H12H201@1, H13H201@1 | 49, 45 | band width | median right; extinction-branch mixture widens the band |
| 15 | N02H203@5 | 47 | cool-flame rate | used 0.41 against 0.31 |
| 19 | H12H201@5 | 44 | droplet gone | end of a burn-down |
| 20 | C04H101@5 | 44 | cool-flame rate | used 0.26 against 0.34 |
| 21 | 209F001@5 | 43 | hot rate too fast | near-limit oscillating flame, 0.47 against law 0.56 |
| 22 | N04H101@5 | 41 | cool-flame rate | weak cool flame at 14 % O2 |
| 23 | N04H101@3 | 39 | cool flame absent wrongly | filter put 0.94 on no flame |
| 24 | C03H601@1 | 39 | radiative timing | 2.4 mm: t_R 2.8 s, out at 4.4 s |

Cause shares of non-problem CRPS:

| cause | cases | sessions | share |
|---|---|---|---|
| cool-flame rate | 7 | 5 | 12 % |
| wrong class | 3 | 1 | 6 % |
| droplet gone | 2 | 2 | 6 % |
| radiative timing | 3 | 2 | 5 % |
| artefact | 3 | 3 | 5 % |
| hot rate too fast | 3 | 3 | 5 % |
| band width | 2 | 1 | 3 % |
| cool-flame extinction diameter | 1 | 1 | 2 % |
| cool flame absent wrongly | 1 | 1 | 1 % |

All 105 non-problem cases by the true state at the cutoff:

| true state at cutoff | run class | cases | share of CRPS | mean µm |
|---|---|---|---|---|
| hot | burn-down | 37 | 32 % | 26.9 |
| hot | radiative later | 35 | 30 % | 26.7 |
| cool | radiative | 12 | 17 % | 43.4 |
| hot | unclear | 6 | 9 % | 49.2 |
| transition | radiative | 6 | 7 % | 35.6 |

## B. Stage table (37 non-problem runs, 16 sessions)

Rates in mm²/s; columns are explained under the table.

| run | atm O2/CO2/He/atm | D0 | cl | Dext (how) | t_ext | K hot before | K 0–1 / 1–2 s after | cool K med [range], n | t_pl | D_pl | K post 0–2/2–5/5–10/10–20 | CF/cloud | pi_r | d_crit | t_R | p_cool | k_cool | d_ce |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| N04H101 | .14/0/0/1 | 4.16 | R | 4.15 m | 1.3 | . | .08/.23 | .16 [.16–.21] 3 | 9.6* | 3.98 | .13/.10/.07/.06 | W/Y | 1.00 | 1.55 | 2.0 | .50 | .33 | 2.34 |
| N03H101 | .15/0/0/1 | 2.55 | R | 2.31 m | 3.5 | .37 | .28/.32 | .32 [.27–.32] 3 | 11.5 | 1.71 | .15/.06/.05/.03 | Y/Y | 1.00 | 1.76 | 2.6 | .82 | .31 | 1.49 |
| N02H203 | .16/0/0/1 | 2.52 | R | 2.13 m | 4.5 | .43 | .32/.35 | .33 [.32–.33] 2 | 12.2 | 1.46 | .10/.07/.05/.03 | Y/Y | .98 | 1.99 | 3.4 | .95 | .33 | 1.41 |
| N02H101 | .16/0/0/1 | 3.34 | R | 3.18 m | 3.7 | .33 | .29/.28 | .34 [.33–.36] 8 | 22.4 | 1.94 | .10/.06/.01/. | Y/Y | 1.00 | 1.99 | 3.4 | .95 | .34 | 1.77 |
| C04H201 | .17/.20/0/1 | 2.90 | R | 2.79 m | 1.9 | .35 | .22/.32 | .35 [.33–.38] 6 | 16.1 | 1.72 | .12/.01/.05/. | L/Y | 1.00 | 1.44 | 1.8 | .95 | .33 | 1.60 |
| C04H101 | .17/.20/0/1 | 3.76 | R | 3.71 m | 1.4 | . | .13/.23 | .34 [.28–.45] 12 | 28.1 | 2.17 | .09/−.05/./. | Y/Y | 1.00 | 1.44 | 1.8 | .95 | .35 | 1.97 |
| C03H601 | .18/.15/0/1 | 2.40 | R | 1.96 m | 4.4 | .47 | .35/.33 | .34 2 | 11.1 | 1.26 | .13/.07/.05/.03 | Y/Y | .99 | 1.82 | 2.8 | .95 | .34 | 1.28 |
| 301F001 | .18/0/0/1 | 3.43 | R | 3.17 m | 4.7 | .40 | .28/.34 | .36 [.34–.36] 6 | 25.7 | 1.65 | .11/.03/./. | Y/Y | .99 | 2.55 | 5.5 | .95 | .38 | 1.63 |
| C03H501 | .18/.15/0/1 | 3.74 | R | 3.68 m | 2.7 | .33 | .18/.25 | .35 [.29–.40] 14 | none to 33.5 s | . | . | Y/− | 1.00 | 1.82 | 2.8 | .95 | .37 | 1.84 |
| C02H201 | .19/.10/0/1 | 2.48 | R | 1.67 m | 7.1 | .51 | .36/.33 | .32 1 | 12.6 | 1.00 | .12/.07/.05/.01 | Y/Y | .82 | 2.31 | 4.5 | .95 | .36 | 1.23 |
| C02H101 | .19/.10/0/1 | 3.40 | R | 3.13 m | 4.5 | .46 | .34/.36 | .37 [.33–.39] 9 | 26.0 | 1.45 | .11/.03/./. | Y/Y | 1.00 | 2.31 | 4.5 | .95 | .38 | 1.59 |
| H13H201 | .20/0/.35/.7 | 3.08 | R | 1.69 m† | ~5† | . | . | . | . | . | . | N/− | .83 | 2.84 | 6.9 | .02 | .49 | 2.74 |
| C01H302 | .20/.05/0/1 | 3.18 | R | 2.54 m | 8.0 | .50 | 1.73‡/. | . | . | . | . | Y/Y | .84 | 2.92 | 7.2 | .95 | .40 | 1.39 |
| 194F001 | .20/0/0/.72 | 3.56 | R | 3.04 m | 7.9 | .74‡ | .39/.34 | . (left view) | . | . | . | ?/Y | .85 | 3.27 | 9.1 | .95 | .40 | 2.13 |
| H13H101 | .20/0/.35/.7 | 3.69 | R | 3.32 m | 4.3 | .63 | .58/.53 | (no CF) .21–.32 | 10.3 | 2.97 | .13/.16/.09/.04 | N/− | .98 | 2.84 | 6.9 | .02 | .51 | 3.17 |
| C10H101 | .21/.30/0/.7 | 3.57 | R | 3.14 m | unobs.‡ | . | . | . | . | . | . | Y/Y | 1.00 | 1.88 | 3.0 | .95 | .38 | 2.25 |
| C09H501 | .23/.25/0/.7 | 3.32 | R | 2.61 m | 9.0 | .52 | .41/.37 | .35 [.29–.37] 3 | 17.5 | 1.98 | .12/.06/.05/. | L/− | .97 | 2.70 | 6.2 | .95 | .41 | 1.89 |
| C09H101 | .23/.25/0/.7 | 4.14 | R | 3.98 m | 6.4 | .36 | ends | . | . | . | . | Y/Y | 1.00 | 2.70 | 6.2 | .95 | .43 | 2.27 |
| C08H101 | .24/.20/0/.7 | 3.53 | R | 2.76 m | 10.1 | .53 | .37/.34 | .33 1 (ends 14.2) | . | . | . | Y/Y | .73 | 3.42 | 9.9 | .95 | .44 | 1.86 |
| N04H201 | .14/0/0/1 | 1.83 | R | 1.35 m? | ends 1.2 s | . | . | . | . | . | . | ?/− | .94 | 1.55 | 2.0 | .50 | .28 | 1.20 |
| H12H101 | .21/0/.30/.7 | 3.23 | U | 2.33 m | 7.5 | .73 | .50/.39 | (no CF) .27 | 13.0 | 1.93 | .13/.09/.05/.03 | N/− | .56 | 3.28 | 8.8 | .10 | .50 | 2.59 |
| 209F001 | .21/0/0/1 | 3.36 | U | 1.75 x | left view | . | . | . | . | . | . | ?/− | .32 | 3.70 | 9.7 | .95 | .42 | 1.37 |

Burn-down runs (narrative ending is in G): C03H701, C02H202, C09H601, 301F002, C01H201 (t_ext 7.7), N01H502, H02H101, H12H201 (t_ext 11.0), C08H601, N01H201, 061F002, N01H101, N01H102; unclear end: H01H102, N01H202. All have pi_r ≤ 0.49.

Column notes:
- cl is the class: R lost the hot flame to radiation, B burned down, U unclear in the narrative. Dext how: m measured, x extrapolated per the narrative.
- "K hot before" is K over [max(0.5, t_ext − 2.5), t_ext − 0.25]. Cool K is over consecutive 2-s windows from t_ext + 2 s to t_pl.
- CF/cloud is the narrative's cool flame (Y yes, L likely, W weak, N none, ? not stated) and vapour cloud.
- The last six columns are D1's laws for the run (at the encoder's D0 at the 3 s cutoff); t_R = s_r·D0²/(1 mm²/s).

Marks:
- \* For N04H101, the "plateau" is only where K drops below 0.15; the series keeps falling at 0.06–0.13.
- † H13H201's sheet Dext (1.69) is reached only at 33 s. The narrative says ~5 s, where D = 2.5 mm.
- ‡ Artefacts. C01H302 and C10H101 leave the view (C10H101's Dext is "reached" only during the 0.25-s collapse). 194F001 drifts out of the view.

Readings:
- **Transition dip.** In the first second after VFE, K is 0.55–0.88 of the hot rate just before. Against the later cool rate it is ≈1 (0.85–1.17) when VFE happens at 3.5–10 s, and 0.38–0.63 when VFE happens at 1.3–2.7 s (C03H501, C04H101, C04H201, N04H101; 3 sessions). The cool rate is reached 1–2 s after VFE in every run.

## C. Class law

**No run on the wrong side.** No non-problem radiative run has pi_r < 0.3 (every radiative run has 0.73–1.00), and no burn-down has pi_r > 0.7 (0.00–0.49).

Runs with |x_exc| < 0.25 (20 runs, 14 sessions). Y_O2 is the oxygen mass fraction; Z_cp is oxygen's share of the mixture's molar heat capacity.

| run | atm O2/CO2/He/atm | D0 | x_exc | pi_r | class | Y_O2 | Z_cp |
|---|---|---|---|---|---|---|---|
| 061F002 | .25/.15/0/.71 | 3.41 | −.240 | .05 | B | .255 | .239 |
| 30CAL06 (problem) | .21/0/0/1 | 2.95 | −.226 | .06 | problem | .233 | .221 |
| N01H201 | .21/0/0/1 | 2.96 | −.223 | .07 | B | .233 | .221 |
| C08H601 | .24/.20/0/.7 | 2.88 | −.172 | .13 | B | .239 | .223 |
| 301F002 | .18/0/0/1 | 2.15 | −.171 | .13 | B | .201 | .190 |
| H12H201 | .21/0/.30/.7 | 2.86 | −.137 | .19 | B | .311 | .248 |
| C01H101 (problem) | .20/.05/0/1 | 2.58 | −.124 | .23 | problem | .216 | .204 |
| 209F001 | .21/0/0/1 | 3.36 | −.096 | .30 | U | .233 | .221 |
| H02H101 | .19/0/.10/1 | 2.59 | −.068 | .39 | B | .231 | .208 |
| N01H101 | .21/0/0/1 | 3.56 | −.038 | .49 | B (edge) | .233 | .221 |
| N01H102 | .21/0/0/1 | 3.58 | −.033 | .51 | B (edge) | .233 | .221 |
| H12H101 | .21/0/.30/.7 | 3.23 | −.016 | .57 | U | .311 | .248 |
| C08H101 | .24/.20/0/.7 | 3.53 | +.031 | .72 | R | .239 | .223 |
| C02H201 | .19/.10/0/1 | 2.48 | +.073 | .82 | R | .200 | .188 |
| H13H201 | .20/0/.35/.7 | 3.08 | +.081 | .84 | R | .314 | .241 |
| C01H302 | .20/.05/0/1 | 3.18 | +.085 | .84 | R | .216 | .204 |
| 194F001 | .20/0/0/.72 | 3.56 | +.086 | .85 | R | .222 | .211 |
| N04H201 | .14/0/0/1 | 1.83 | +.164 | .94 | R | .157 | .148 |
| C09H501 | .23/.25/0/.7 | 3.32 | +.206 | .97 | R | .224 | .208 |
| N02H203 | .16/0/0/1 | 2.52 | +.236 | .98 | R | .179 | .169 |

**Conclusion C: no alternative variable is supported.**
- **Oxygen mass fraction** would move helium the wrong way. H13H201 loses its flame to radiation at Y_O2 0.314, the highest in the table, while air at 0.233 is on the edge at 3.56 mm.
- **Z_cp** brings nothing. It orders the CO2 runs like the present CO2 term, and it puts helium above air, which H13H201 contradicts.
- **Pressure in d_crit** (194F001 against N01H101 and N01H102) is not resolved.
- The ambiguous band (pi_r 0.3–0.7) holds 209F001, H02H101, N01H101, N01H102 and H12H101. The narratives back these values.

**Radiative clock (related to C).** Observed clock of VFE, s_ext = t_ext·1 mm²/s / D0², against D1's s_R. 18 radiative runs, 13 sessions.

ln(s_ext/s_R) by run, in order of D0:

| run | D0 | ln(s_ext/s_R) |
|---|---|---|
| C03H601 | 2.40 | +.45 |
| C02H201 | 2.48 | +.45 |
| N02H203 | 2.52 | +.28 |
| N03H101 | 2.55 | +.28 |
| C04H201 | 2.90 | +.06 |
| H13H201 | 3.08 | −.32 |
| C01H302 | 3.18 | +.10 |
| C09H501 | 3.32 | +.37 |
| N02H101 | 3.34 | +.09 |
| C02H101 | 3.40 | .00 |
| 301F001 | 3.43 | −.17 |
| C08H101 | 3.53 | +.01 |
| 194F001 | 3.56 | −.13 |
| H13H101 | 3.69 | −.47 |
| C03H501 | 3.74 | −.03 |
| C04H101 | 3.76 | −.25 |
| C09H101 | 4.14 | +.04 |
| N04H101 | 4.16 | −.43 |

Medians by D0 class:

| D0 | runs | sessions | median |
|---|---|---|---|
| < 2.6 mm | 4 | 4 | +0.37 |
| 2.6–3.45 mm | 7 | 6 | +0.06 |
| > 3.45 mm | 7 | 7 | −0.13 |

By x_exc class the residual is not ordered (+.01, +.16, −.03). Same-atmosphere pairs, ln s ratio per e-fold of D0 (law −2.0 in every pair):

| pair | sessions | observed per e-fold of D0 |
|---|---|---|
| C03H501 vs C03H601 | 11-02-21 | −3.1 |
| C02H101 vs C02H201 | 11-04-20 | −3.4 |
| N02H101 vs N02H203 | 11-08-29 / 11-08-30 | −2.7 |
| C04H101 vs C04H201 | 11-08-17 | −3.2 |
| H13H101 vs H13H201 | 11-11-22 | −2.9 |
| C09H101 vs C09H501 | 10-06-29 / 11-04-20 | −3.5 |

## D. Hot-rate law audit

Residual ln(K_obs/K_law) by window, hot phase only. 117 windows, 34 runs, 16 sessions. C10H101 and N01H101 are excluded (VFE time unknown; tracking noise). Overall median +0.053 (W0 −0.016).

| grouping | group | windows | median [min, max] | sessions >+0.05 / <−0.05 |
|---|---|---|---|---|
| window | 0.3–1 s | 34 | **+0.160** [−.46, +.48] | **13/2 of 16** |
| | 1–2 s | 31 | **+0.070** [−.03, +.34] | **12/0 of 15** |
| | 2–3 s | 28 | −0.006 | 3/3 of 14 |
| | 3–5 s | 17 | +0.004 | 1/2 of 12 |
| | 5–8 s | 7 | −0.027 | 0/1 of 7 |
| clock | s < 0.15 | 46 | **+0.143** | **14/2 of 16** |
| | 0.15–0.6 | 61 | +0.025 | 3/1 of 15 |
| | > 0.6 | 10 | −0.023 | 0/0 of 6 |
| oxygen | ≤ 0.16 / 0.17–0.19 / 0.20–0.21 / ≥ 0.23 | | +.050 / +.046 / +.070 / +.027 | 1/0, 2/0, 4/0, 2/1 |
| diluent | N2 / CO2 / He | | +.039 / +.032 / +.083 | 2/0 of 7, 3/1 of 7, 2/0 of 3 |
| pressure | 0.7 / 1 atm | | +.063 / +.040 | 4/1 of 7, 4/0 of 11 |
| D0 class | <2.3 / 2.3–3.0 / 3.0–3.5 / ≥3.5 mm | | +.066 / +.036 / +.039 / +.072 | (≥3.5: 7/1 of 8, from early windows only) |
| order in session | 1st / 2nd / 3rd and later | | +.063 / +.066 / +.028 | |
| session | 2011-11-22 (He) | | +0.125 | single session |

Clock class against D0 class: at s < 0.15 every D0 class is between +0.11 and +0.20; at s 0.15–0.6, between +0.02 and +0.07.

D1's early-time terms at the window centre (median):

| window | residual | unit-4 term (fast heat-up) | unit-5 term (flame development) | residual if unit 4 were removed |
|---|---|---|---|---|
| 0.3–1 s | +0.160 | −0.131 | −0.264 | +0.084 |
| 1–2 s | +0.070 | 0 | −0.214 | +0.066 |
| 2–3 s | −0.006 | 0 | −0.163 | −0.006 |

Does the run offset b help the hot forecast? Observed K over the next ≤ 6 s against the law alone and law·e^b:

| cutoff | cases | sessions | law alone: median, median abs | with b: median, median abs | b helped |
|---|---|---|---|---|---|
| 1 s | 28 | 14 | +.023, .038 | −.043, .048 | 12 |
| 3 s | 17 | 12 | −.023, .028 | −.056, .056 | 6 |
| 5 s | 12 | 9 | −.035, .035 | −.064, .064 | 4 |

b is positive in 24 of 28 cases at 1 s.

Realised spread of the residual at s ≥ 0.15: robust sd 0.066 (71 windows). Run-level spread: robust sd 0.059 over 30 runs.

**Conclusion D.**
- **The only offset backed by at least three sessions is early time.** The law is too slow by about 15 % for s < 0.15, in every D0, oxygen, diluent and pressure class.
- **The early deficit leaks into the forecast** through a positive b, which makes forecasts too fast.
- **Helium** shows +0.08 to +0.23, but in one session.

## E. Cool flame

Observed cool rate against D1's k_cool, ln(observed/law):

| run | ln ratio |
|---|---|
| 301F001 | −.05 |
| C02H101 | −.04 |
| C02H201 | −.10 |
| C03H501 | −.04 |
| C03H601 | +.01 |
| C04H101 | −.02 |
| C04H201 | +.05 |
| N02H101 | .00 |
| N02H203 | .00 |
| N03H101 | +.01 |
| C09H501 (0.7 atm) | −.17 |
| C08H101 (0.7 atm, 1 window) | −.30 |
| N04H101 (14 % O2) | −.73 |
| H13H101 (no cool flame) | −.64 |

The first ten are 1 atm, 15–19 % O2, 7 sessions: median −0.01, observed 0.32–0.37 mm²/s.

Plateau diameter against d_ce, ln(D_pl/d_ce):

| run | ln ratio |
|---|---|
| 301F001 | +.01 |
| C02H101 | −.09 |
| C02H201 | −.21 |
| C03H601 | −.01 |
| C04H101 | +.10 |
| C04H201 | +.08 |
| C09H501 | +.05 |
| N02H101 | +.09 |
| N02H203 | +.03 |
| N03H101 | +.14 |
| 194F001 | +.12 (unreliable) |
| N04H101 | +.53 (weak cool flame) |

Duration from VFE to plateau: 5.5 s (C02H201) to 26.8 s (C04H101).

**Residuals against conditions.**
- The cool rate does not follow O2 (15–19 %), CO2 (0–20 %), D0 (2.4–3.8 mm) or the diameter at the start of the cool phase (1.45–3.66 mm).
- The plateau residual falls with oxygen: +0.09 at 15–17 % O2 (5 runs, 4 sessions), −0.05 at 18–19 % (4 runs, 3 sessions). C09H501 contradicts a steeper slope.
- At 14 % O2, D1's k_cool (0.33) is above its own hot law (0.25).

**Approach to the plateau.** The rate holds at 0.86–1.04 of the cool rate until about 1 s before the plateau, then falls to 0.47–0.99 (median 0.83) in the last second. Over the last 3 s it averages about 0.9. The model's constant rate up to D_ce is the right shape.

**After the plateau** (13 runs, 8 sessions), observed against D1's no-flame law:

| window after plateau | observed median | law median | runs where observed > law |
|---|---|---|---|
| 0–2 s | 0.121 | 0.071 | 13 of 13 (includes the tail of the cool phase) |
| 2–5 s | 0.062 | 0.047 | 9 of 13 |
| 5–10 s | 0.049 | 0.035 | 8 of 10 |
| 10–20 s | 0.033 | 0.028 | 4 of 7 |

In helium without a cool flame (H12H101, H13H101) the droplet loses 0.50–0.58 mm²/s after VFE, decaying to 0.13 within 5 s.

## F. State at the cutoff

Non-problem cases, true state against the encoder's argmax:
- **1 s cutoff:** hot read as hot 33 of 33; transitions read as hot 2; unknown read as hot 2.
- **3 s cutoff:** hot 30 of 30; transitions read as cool 2; cool read as cool 2 and as no flame 1 (N04H101).
- **5 s cutoff:** hot 19 of 19; transitions read as hot 2; cool 9 of 9; no flame read as hot 1 (H13H101); unknown read as cool 1.

Where the filter is wrong:

| case | truth | state [hot, cool, none] | prior | r_late | gain | why |
|---|---|---|---|---|---|---|
| N04H101@3 | weak cool flame | 0, .06, .94 | .01, .49, .49 | 0.42 | 0.78 | k_cool 0.33 is above the hot law 0.25, so the observed 0.11 lands nearest evaporation |
| H13H101@5 | no flame | .99, 0, .01 | .80, 0, .19 | 1.15 | 0.98 | post-extinction evaporation in helium runs at 0.58 mm²/s, as fast as the hot flame |

Transition cases at 3 and 5 s get split probabilities (0.36–0.63), which is correct.

**Cool-rate offset.** Over 17 cool or transition cases in 7 sessions, the law alone misses the observed cool rate by median |ln| **0.06**; the rate actually used, k_cool·e^b_cool, misses by **0.21**.
- When the late window holds the hot flame, the used rate is too fast: C02H101@5 0.46 against 0.37; C03H601@5 0.46 against 0.33; N02H203@5 0.41 against 0.31.
- When it holds the dip, the used rate is too slow: C04H201@3 0.26 against 0.35; C04H101@5 0.26 against 0.34; C03H501@5 0.27 against 0.35.

## G. Small droplets

| run | D/D0 at end | sheet Dext/D0 | D1 De/D0 | end |
|---|---|---|---|---|
| C03H701 | .35 | .35 | .39 | disruption (tracker held) |
| C02H202 | .30 | .30 | .35 | disruption (tracker held) |
| C09H601 | .25 | .23 | .34 | disruption |
| C01H201 | .37 | .37 | .30 | disruption |
| N01H502 | .24 | ? | .27 | disruption |
| H12H201 | .23 | .25 | .28 | diffusive extinction, then disruption |
| 301F002, C08H601, H02H101, N01H201, 061F002, N01H101, N01H102 | .59–.93 | | | left the view: not an extinction diameter |

D1's w_d gives De/D0 0.27–0.39 for the six runs with a visible end (observed median about 0.27; 6 runs, 5 sessions).

**After the droplet is gone** the forecast keeps a slowly evaporating droplet: median 0.25–0.62 mm at cutoff + 10 s, P(no flame) 0.7–0.9, never scored. The last observed second carries 20–100 % of the case CRPS. In C02H202 and C03H701 it scores frozen tracker values (0.493 and 0.474 mm). For C02H202@5 the level was carried to 0.33 mm, past an extinction inside the tail window.

## H. Spread (non-problem runs, D1)

| cutoff | cases | sessions | cov50 | cov90 | width50 / width90 µm | sessions with cov50 > 0.6 | sessions with cov90 ≥ 0.97 | sessions with cov90 < 0.8 |
|---|---|---|---|---|---|---|---|---|
| 1 s | 37 | 16 | 0.90 | 1.00 | 128 / 328 | 15 | 16 | 0 |
| 3 s | 36 | 16 | 0.73 | 0.97 | 91 / 244 | 11 | 12 | 1 (11-08-17) |
| 5 s | 32 | 15 | 0.63 | 0.82 | 97 / 247 | 8 | 6 | 4 (10-01-12, 11-04-20, 11-08-17, 11-09-01) |

By state:
- **Hot at the cutoff:** cov50 0.92 / 0.80 / 0.79 and cov90 1.00 / 0.99 / 0.91 at 1 / 3 / 5 s; mean absolute median error 28–44 µm against width90 250–350 µm. **Too wide.**
- **Cool at the cutoff:** cov50 0.16–0.31, cov90 0.61–0.75. Too narrow and biased, because of b_cool.

Widest in helium (2011-11-22: width90 621–715 µm, cov90 1.00). The per-session-and-cutoff table is in `small_out.md`.

## Proposals

The D1 laws were typed from these same train runs, so on train they look better than they will on unseen sessions. Each risk statement accounts for that.

**P1. A cool flame's run offset must come from the cool flame, or not at all.**
- **Statement.** Set b_cool to zero unless the late window lies inside the cool phase. The structural form is a forward change. The weight-level fallback is enc.e_clip from [0.3, 0.6] to about [0.1, 0.1].
- **Size.** The used cool rate misses by median |ln| 0.21; the law alone by 0.06. Six cases carry 334 µm, 11 % of non-problem CRPS.
- **Support.** 17 cases, 7 sessions.
- **Would contradict it.** Runs whose true cool rate departs from the law: N04H101 at 14 % O2, 248F001 at 2 atm.
- **Risk.** Low for the mechanism. Higher for the e_clip route, which also limits the hot offset.

**P2. The flame starts faster than the law says.**
- **Statement.** Raise ln K by about 0.10–0.15 for s < 0.1, fading out by s ≈ 0.15–0.2: rate.w2[4] from −0.25 to about 0. A residual +0.07 at 1–2 s sits on the earliest part of unit 5, whose depth from s = 0.2 onward is right.
- **Support.** 34 runs, 13 of 16 sessions. Also removes the spurious positive b that makes forecasts too fast.
- **Would contradict it.** C09H101 and 209F001 (−0.45 at 0.3–1 s).
- **Risk.** Moderate: igniter timing may differ in other sessions.

**P3. The radiative clock shortens with D0 faster than x_exc alone gives.**
- **Statement.** Add about −1 per e-fold of D0 to ln s_R (pairs: −0.7 to −1.6 beyond the x_exc slope), with rad.b0 re-anchored by about +0.5 so 3.4 mm droplets keep their current s_R.
- **Support.** 6 same-atmosphere pairs in at least 5 sessions; 18 runs in 13 sessions.
- **Would contradict it.** C09H101 (on the law now, would move to −0.3) and the two helium runs.
- **Risk.** Train D0 range is 2.4–4.2 mm. Extrapolating below 2.3 mm would delay radiative extinction strongly, and N04H201 (1.83 mm) cannot check it.

**P4. The hot-rate spread before prefix evidence is about twice too large.**
- **Statement.** Lower unc.s_pop from 0.20 toward 0.10–0.12. The realised run-level spread is 0.06, but the law came from these runs, so 0.06 understates unseen sessions.
- **Support.** At the 1 s cutoff, cov50 0.90 and cov90 1.00, in 16 of 16 sessions.
- **Would contradict it.** A session whose rate departs from the law by more than ±0.15; none on train after s ≥ 0.15.
- **Risk.** Under-coverage on unseen atmospheres. Do it after P1 and P2, which remove biases the present width hides.

**P5. Evaporation after the cool flame decays over about 10 s, not about 2 s.**
- **Statement.** About +30 % in rate over the first 10 s, through a longer decay (post.tau0_s larger, or k_inf slightly larger).
- **Support.** 13 runs, 8 sessions.
- **Would contradict it.** N02H101, and C04H101/C04H201, where the vapour cloud darkens the backlight and the droplet appears to grow.
- **Risk.** Low, and the CRPS effect is small: plateaus fall inside the 10-s horizon only for 2.4–2.6 mm droplets at the 5 s cutoff.

## Leave alone

- **Helium** (one session for 30–35 % He): hot rate, radiative timing, post-extinction evaporation, d_crit.
- **Cool rate at 0.7 atm and 23–24 % O2:** two sessions, conditions confounded.
- **14 % O2 cool flame:** one session. A cap of k_cool at the hot rate would be a structural guard worth considering, not a number to type.
- **Steeper oxygen slope of d_ce:** the effect is the size of cool.d.sd (0.12), and C09H501 contradicts it.
- **Any change to the class law's variables:** no run is on the wrong side, and helium contradicts a mass-fraction variable.
- **2 atm (248F001), soot and glare runs, leaving-view and frozen-tracker cases:** problem runs or measurement effects, not laws.
- **Order of burn in the session:** no consistent offset.

## Could not determine
- VFE times for C10H101 and H13H201.
- Burn-down end diameters for runs that left the view.
- The class of 209F001 and H12H101.
- The CRPS gain of any proposal (no modified release was assembled).
- Whether part of the early-rate excess comes from igniter timing; it was not checked across sessions.

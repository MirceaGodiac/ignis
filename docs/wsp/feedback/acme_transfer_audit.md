# ACME clocks: what transfers between train sessions (normal D3, inverse D1 and D3)

> Train partition only; nothing fitted. Written by a delegated analysis agent (claude-opus-5-5) on 14 November
> 2026; the author rechecked the claims it acted on against the train tables. Paths under `scratchpad` are the
> agent's working folder and are not part of the repository.

I ran every item on the train partition only. No validation or test files were opened and nothing in the repository was changed. One thing to know first: the card (`docs/wsp/cards/acme_clock.md`) states which fuel/pressure combinations the validation sessions hold, so I saw that line while reading it. I did not use it: the sweep grid is the one you gave me.

The short answer is that the train score is carried by a handful of runs the laws get badly wrong, and that most of what the laws assert outside the train points is unsupported. A shift of the whole law by a session-sized offset barely changes the score, so session offsets do not explain the gap.

**Checks on my re-implementation.** `fl.py` (Python copy of `flame.rs` plus the IPCW Brier of `scores.rs`) matches the stored train feedback (`fb_train_acme_{normal,inverse}.json`) to ≤1e-15 at 120 s on all 210/204 cases. Its train case-mean scores match the forward exactly (nD1 0.05618/0.04144, nD3 0.05159/0.04125, iD1 0.05724/0.06682, iD3 0.04053/0.06176). Its clocks match `ignis wsp forward` diagnostics to <1e-15 on 12 train conditions per release, 48 in all.

## 1. Where the train score comes from

Integrated Brier score (IBS) by session, case means over cutoffs 1/3/5 s:

| Normal | cases | SE/FT | D1 ext | D1 op | D3 ext | D3 op |
|---|---|---|---|---|---|---|
| 03-29 | 39 | 8/5 | .033 | .070 | .023 | .070 |
| 04-16 | 27 | 7/2 | .033 | .025 | .034 | .020 |
| 05-05 | 12 | 5/1 | **.246** | **.212** | **.246** | **.224** |
| 05-10 | 18 | 15/0 | .052 | .003 | .023 | .005 |
| 06-02 | 42 | 9/5 | .032 | .038 | .039 | .041 |
| 06-11 | 36 | 12/0 | .033 | .008 | .018 | .006 |
| 10-18 | 36 | 8/4 | .089 | .023 | .093 | .018 |

| Inverse | cases | SE/FT | D1 ext | D1 op | D3 ext | D3 op |
|---|---|---|---|---|---|---|
| 07-15 | 21 | 0/7 | **.196** | **.171** | .000 | .083 |
| 07-28 | 33 | 0/11 | .003 | .080 | .000 | .078 |
| 08-10 | 22 | 9/0 | .021 | .002 | .025 | .007 |
| 08-16 | 49 | 14/3 | .092 | .067 | .101 | .077 |
| 08-30 | 6 | 0/2 | .000 | .171 | .000 | .181 |
| 09-13 | 43 | 16/1 | .013 | .004 | .019 | .009 |
| 09-27 | 30 | 2/8 | .064 | .095 | .065 | .095 |

Residual ln(observed life) − ln(floor + median R), for self-extinguished (SE) flames lasting more than 2 s:

| Session | nD1 med [min, max] | nD3 med [min, max] | n |
|---|---|---|---|
| 03-29 | −0.04 [−0.48, +0.23] | −0.02 [−0.15, +0.33] | 8 |
| 04-16 | +0.12 [−0.08, +0.47] | +0.06 [−0.12, +0.38] | 7 |
| 05-05 | −0.18 [−4.34, −0.15] | −0.00 [**−5.08** (21125A2), +0.11] | 3 |
| 05-10 | −0.40 [−1.56, +0.04] | **−0.34** [−0.89, −0.18] | 7 |
| 06-02 | −0.17 [−0.36, +0.60] | +0.08 [−0.04, +0.15] | 9 |
| 06-11 | −0.03 [−0.52, +0.40] | −0.01 [−0.07, +0.26] | 12 |
| 10-18 | −0.32 [−0.50, −0.02] | −0.06 [−0.17, +0.27] | 8 |

| Session | iD1 = iD3 med [min, max] | n |
|---|---|---|
| 08-10 | +0.27 [−0.07, +0.55] | 8 |
| 08-16 | −0.03 [**−4.61** (21228L1), +0.52] | 14 |
| 09-13 | −0.08 [−0.39, +0.55] | 15 |
| 09-27 | −0.14 [−0.27, −0.02] | 2 |

Inverse D3 changed no self-extinction law, so its residuals equal D1's. Sessions 07-15, 07-28 and 08-30 have no SE flames.

**A few runs carry most of the score:**
- **Normal D3:** 21125A2 (3 % butane, 0.5 mg/s, out at 8 s; the law says about 1280 s) is 26 % of the summed extinction score and 26 % of the termination score. 21291A1 and C1 (cool-flame cycling, labelled FT) add another 20 % of extinction. Five runs make 52 % of extinction and 64 % of termination.
- **Inverse D1:** 21228C2 (50 % O2 at 9.07 mg/s, stopped at 66 s) is 24 % of extinction. 21196F1/F2/F6 (15 % O2 into 29 % ethane, burned 36–66 s) are 35 %. Five runs make 64 %.
- **Session offsets don't explain the gap.** Moving every radiative median by ±0.4 in ln (the size of D1's own session misses) only takes extinction IBS from 0.052 to 0.067–0.077 (nD3) and from 0.057 to 0.062–0.070 (iD1). The rise to 0.15–0.16 on unseen sessions needs regime-level misses (a flame put in the wrong class, or lives off by ×10–1000), each of which costs about 0.9 per case.
- **What the laws miss:** normal 05-05, 05-10 (propane in air at 2–3 atm), and 10-18 (cycling flames); inverse 07-15 (D1), 08-16 (21228C2, 21228L1) and 09-27.

## 2. Support of every nonzero typed number

Each row is a group of numbers. "Active" means the term changes ln(life) by ≥0.05 in an informative run; "contrast" means two runs within one session differ in the term by ≥0.1.

**Normal D3 (54 nonzero numbers):**

| Numbers | Sessions that test it | Support |
|---|---|---|
| se.shape 3; se.tau.b0 2.43 | all 7 | ≥3 |
| skip c_o2 +0.84, plus 0.65 c_o2 inside units 0–1 | contrast in 03-29, 04-16, 06-02, 06-11, 10-18 | ≥3 |
| skip c_logp −0.28, plus −0.24 hinge shift | contrast in 06-02, 10-18, 05-10 (05-10 contradicts: −0.34) | ≥3 |
| unit 0 (b1 0.50, w2 1.4) | active in all 7; in air the lengthening rests on 06-11 C1/C2/D1 and 03-29 M4 | ≥3 |
| unit 1 (b1 −0.22, w2 2.0) | 04-16, 06-02, 06-11, 10-18; 05-05 A2 contradicts | ≥3 |
| unit 2 dilute butane (−1, 5, −5, −0.35) | 10-18 supports (0.15 vs 0.30); 04-16 contradicts (0.125/0.25/0.5: 60–63 s, flat) | 2 |
| unit 3 butane leveling below 12 % (−5.9, +0.35) | 05-05 only | **1** |
| unit 4 butane in enriched O2 (1, 5, −4.1, −0.25) | 04-16, 10-18 against ethane sessions; fuel is confounded with session | 2 |
| unit 5 propane in enriched O2 (5 numbers) | none | **0** |
| se.floor_s 4 | 05-10 decides it (7 flames at 5–7 s); 03-29 and 05-05 merely consistent | **1** |
| flash b0 −18, c_tad −12, tau 1, shape 4 | flashes in 05-05 and 05-10; the cut between 1701 and 1785 K rests on 05-05 only | 2 (cut 1) |
| ft.tau b0 6.5, ethane +0.54 | ethane limits in 03-29 F1, 06-02 A4/C5 | 2 |
| ft.tau c_logm 4.5 | 06-02 (0.5 against 0.75 mg/s) | **1** |
| ft.tau butane −0.56 | 04-16 F1, 05-05 K3, 10-18 J1/H2 | ≥3 |
| ft.shape 2.5 | 4 sessions with limits | ≥3 |
| atom 61.5 s, logit −2, atom_w −2·c_logm | 03-29 D1/H1, 06-02 C7 | 2 |
| atoms 98 s (−2.9) and 126.5 s (−2.2) | 03-29 only | **1** |
| atom 275.5 s (0.0) | 06-02 C4 stopped; 06-11 A4 passed | **1** |
| log_base −7.6 | 04-16 F3, 10-18 A1/C1 (pooled ln rate −7.4) | 2 |

The four atom logits at −40 are switch-offs: nonzero numbers with no effect.

**Inverse D1 (42 nonzero numbers):**

| Numbers | Sessions that test it | Support |
|---|---|---|
| floor 2, shape 3, b0 2.17 | 08-10, 08-16, 09-13, 09-27 | ≥3 |
| skip c_o2 +0.45 | 50 % against 85 % contrast in 08-10, 08-16, 09-13 | ≥3 |
| skip c_xf +3.0 | contrast in 09-13 only (7.0 against 8.8 %); 08-10, 08-16, 09-27 across sessions, confounded with fuel and pressure | 1 within / 4 across |
| unit 0, fuel above 13.5 % (1, −0.3, 10) | SE only in 09-27 (H1, H5); FT lower bounds in 07-15, 07-28, 08-30, 09-27 | **1** (SE) |
| unit 1, low flow (−1, −1, −8) | collapse in 08-10 A5 and 09-13 A4/H4; edge at 1.8 mg/s in 08-16 | 2 |
| unit 2, air (−1, −0.6, 8) | only SE is 21228L1 at 3 s, which contradicts it (law says ~300 s); otherwise lower bounds | **1, contradicting** |
| flash b0 −11, c_tad −10, tau 1.5, shape 3 | 08-10 F1 and 09-13 M1 flash; 07-15 contradicts | 2 / 1 contra |
| ft.tau b0 6.28, ft.shape 4 | limits in 07-28, 08-30, 09-27 | ≥3 |
| ft.tau c_logm 1.6, c_o2 −1.2, fuel hinge above 18 % (1, −0.6, −2.6) | 07-28 and 09-27; 07-28 and 08-30 | 2 each |
| atom 66 s | 4 of 5 sessions | ≥3 |
| atom 35.5 s | 07-15 and 09-27, with different causes | 2 |
| atoms 26, 43, 60, 63.5, 90, 136 s | one session each | **1** |
| log_base −7.6 | 9 unscripted stops in 2309 flame-s give ln −5.55 (5 sessions): contradicted | contra |

What would not survive losing one session:
- **Normal:** the floor (05-10); burner slope 4.5 (06-02); the 98, 126.5 and 275.5 s atoms; units 3 and 5; the flash cut.
- **Inverse:** six of the eight atoms; the fuel-above-13.5 % hinge as an SE law (09-27); the air unit.

## 3. Do T_ad or Z_st order the lives?

Concordance of each variable with self-extinction life, over pairs of runs whose lives differ by at least ×1.25. Pairs are matched on flow.

| Normal, matched flow | T_ad | lower Z_st | O2 | lower p | reduced flow z (lower) | D3 |
|---|---|---|---|---|---|---|
| enriched, all (75) | 0.60 | 0.72 | 0.44 | 0.87 | 0.77 | 1.00 |
| enriched, across sessions (63) | 0.60 | 0.70 | 0.43 | 0.86 | 0.73 | 1.00 |
| air (22) | 0.55 | 0.55 | 0.27 | 1.00 | 0.95 | 0.95 |

Group medians at 1.5 mg/s, 1 atm, about 39 % O2:

| Fuel | x_f | Life | T_ad | Z_st |
|---|---|---|---|---|
| ethane | 0.30 | 71 s | 2582 | 0.267 |
| butane | 0.30 | 47 s | 2657 | 0.203 |
| butane | 0.15 | 27 s | 2532 | 0.302 |
| butane | 0.031 | 35 s | 1795 | 0.657 |

Neither T_ad nor Z_st puts those four lives in order. The 860 K drop in T_ad at 3 % butane costs only ×0.75. **No: neither variable makes the fuel or dilution terms unnecessary.** Butane has the higher T_ad and the lower Z_st of the first two rows, yet the shorter life (soot). In air, pressure alone orders the pairs (17/17).

One variable computed from the conditions with no typed number comes close to D3's reduced flow: the flame size of a point source, r_f = m_total / (−ln(1−Z_st)). Concordance at the same pressure is 0.80 against D3's z at 0.81, and it absorbs the ethane dilution in 03-29 and part of the butane dilution in 04-16 and 10-18. Using it would need a new forward feature.

**Inverse:** at the same O2, across sessions, ambient fuel fraction orders 0.73, T_ad 0.50, Z_st 0.47. Propane at 3.3 atm, 8 %, 2531 K lives 16 s (median); butane at 2.2 atm, 8.8 %, 2678 K lives 10 s. T_ad adds nothing to ambient fuel fraction.

## 4. Extrapolation sweep

Grid: 2940 normal and 2880 inverse points. Distance d is Euclidean in (ln p, ln m, c_o2, ln x_f) to the nearest train run; 0.3 is near train, above 1 is far. T_ad for flash placement comes from a complete-combustion estimate: within ±30 K of the table below 1900 K, too high by up to ~700 K above, where the normal flash law is already near 0. Inverse D1's flash, read in the 1900–2200 K band, is understated by the sweep; its median lives are unaffected.

| Law | Region | Behaviour | d (median, nearest run) |
|---|---|---|---|
| nD3 | O2 ≥ 0.35, m ≤ 0.5 mg/s (0.3 mg/s at 2–3 atm), all fuels | median life 2000 to 40000 s | 0.8–1.1 (21088F1, 21153C5, 21106F3: all ended at the burner limit in 13–56 s; the only SE nearby is 21125A2 at 8 s) |
| nD3 | O2 between 0.25 and 0.35, m ≤ 1 | life rises up to ×20 per 0.1 O2 (slope above 3 per 0.1) | 0.8–1.1; **no train run between 0.22 and 0.33 O2** |
| nD3 | x_f falling from 0.10 to 0.05 (ethane, propane) and from 0.05 to 0.03 (butane), any O2 | weak-class probability jumps 0 → 1 (ethane at 5 % in 40 % O2: 0.88 at about 1700 K) | 0.6–1.1; train flashes exist only in air |
| nD3 | m = 0.3 mg/s, all 412 points | burner-limit median 2–5 s (slope 4.5 carried below 0.5 mg/s) | 0.5–0.7; shortest observed limit is 28 s |
| nD3 | Z_st above 0.75 | 56 points refused | – |
| nD3 | direction (flow, O2, p, x_f) | no violations | – |
| iD1/iD3 | O2 0.21–0.30, any x_f | life **rises** as O2 falls (unit 2, slope about −3 per 0.1); at 0.30 O2, 10 % fuel the life is 32 s against 11 s at 0.50 | 0.6–0.9; no train run at 30 % O2 into a lean ambient |
| iD1/iD3 | air into 5–10 % fuel | main-class life 60–460 s | 0.2–0.9; 21228L1 lived 3 s |
| iD1/iD3 | x_f between 0.14 and 0.30 | life 50 s → 3e3–3.6e7 s, slope up to 13 per e-fold | 0.6–0.9; SE only at 14 % (09-27) |
| iD1/iD3 | m ≤ 1.8 mg/s | life falls 8 per e-fold, below floor + 1 s at m ≤ 1.4; the collapse is carried down to 1.0 mg/s | 0.7–1.1; lowest train flow 1.34 mg/s |
| iD1/iD3 | any pressure | no pressure term at all (flat) | – |
| iD1/iD3 | m ≤ 1.8 mg/s at 85 % O2, **lean** ambient | burner-limit median 10–19 s (iD1), 8–17 s (iD3) | 0.1–0.7; lean sessions never exceeded 200 °C |
| iD1 / iD3 | air into 5–7 % fuel; 30 % O2 into 5 % fuel | weak-class probability flips by 0.7–0.98 between grid steps | 0.2–0.9 |
| iD1/iD3 | Z_st out of range | 128 points refused | – |

## 5. Flow terminations

| Normal session | Programmed stop | Burner limit | Other |
|---|---|---|---|
| 03-29 | D1 61 (operator flow cut, 459 °C), H1 62, K1 98 (340 °C, unexplained), H2 127 | F1 28 | – |
| 04-16 | – | F1 72 | F3 13 (313 °C) |
| 05-05 | K3 125 (481 °C: limit or script) | – | – |
| 06-02 | C7 61, A5 274, C4 276 | A4 56, C5 49 | – |
| 10-18 | – | J1 80, H2 158 | A1 167, C1 246 (cycling) |

| Inverse session | Programmed stop | Burner limit | Other |
|---|---|---|---|
| 07-15 | L2 63, M2_1 64, F2 66, F6 66, M2 66 (M2 and M2_1 at 515 °C) | – | A2 35 (auto-detect), F1 36 (operator) |
| 07-28 | A4, F1, F2, F3, H1, J1 at 66; A3 67; B1 136 | A1 26, B2 48 | K1 26 (time-out) |
| 08-16 | F1 65, C2 66 | – | L2 16 |
| 08-30 | – | A1 12 | P1 21 |
| 09-13 | F1 66 | – | – |
| 09-27 | A2 90, A4 90 | A3 35, H2 43 | A1 60, H3 43, C1 70, H4 83 |

**How each candidate places its stop probability:**
- **Normal D1 and D3:** atoms of ±0.5 s at 61.5 / 98 / 126.5 / 275.5 s with stop probability 0.12 (0.35 at 0.5 mg/s), 0.05, 0.10 and 0.70 (D3: 0.50), over a background of e^−7.6 (6 % by 120 s). 21153A5 (274 s) falls outside the 275.5 s atom window.
- **Inverse D1:** eight atoms, with 66 s at 0.80, 90 s at 0.60 and 136 s at 0.90, over a background of e^−7.6.
- **Inverse D3:** atoms at 66 / 90 / 136 s with 0.75 / 0.40 / 0.50, over a background of e^−6 (26 % by 120 s). Stops at 63, 64, 65 and 67 s fall outside the 66 s window and are paid for by the background.

**A statement that doesn't depend on having seen the session:**
- **Inverse:** of the 5 sessions with a flame alive at 60 s, 4 stopped flames between 63 and 67 s (15 of 18 flames alive then). The fifth stopped at 90 s (2) or ended its flames at 60, 70 and 83 s. Unscripted stops: 9 in 2309 flame-s (ln −5.5), spread over 5 sessions.
- **Normal:** of 6 sessions with a flame alive at 60 s, 2 stopped at 61–62 s. The 98 s and 125–127 s stops come from one session each; 274–276 s from one of the two sessions that reached it. Programmed rate: 3 stops per 219 flame-s in 60–70 s, and 4 per 1878 flame-s in 70–300 s. Other stops: ln −7.4.

## 6. Inverse D1 against D3, one change at a time

Each column is D1 with one of D3's three changes, scored on train with the verified copy (extinction/termination):

| Session | D1 | + flash | + stops | + burner | D3 |
|---|---|---|---|---|---|
| 07-15 | .196/.171 | **.000/.075** | .195/.180 | .196/.172 | .000/.083 |
| 08-10 | .021/.002 | .021/.002 | .022/.003 | .025/.006 | .025/.007 |
| 08-16 | .092/.067 | .092/.066 | .092/.069 | **.101/.076** | .101/.077 |
| 09-13 | .013/.004 | .014/.004 | .014/.005 | .017/.007 | .019/.009 |
| all | .057/.067 | .037/.056 | .058/.068 | .061/.071 | .041/.062 |

07-28, 08-30 and 09-27 change by less than 0.01.

- **Flash decided by lean ambient.** All of D3's train gain comes from this change, and almost all of that from one session (07-15: F1, F2, F6), plus small gains in 08-16 (L2, F1) and 07-28 (K1). It hurts 21228L1 (3-s flame) and 21256F1 (burned to 66 s). Most different from D1 at 30 % O2 into 5–7 % fuel (+0.20 to +0.27) and air into 7 % (−0.17).
- **Single stops as a background rate.** Net zero on train: it helps 21209B1, 21228C2 and 09-27, and hurts 21196A2, 21209K1, 21270H3/H4 and 21228A4.
- **Steeper, wider burner clock.** Worse on train in 08-16, 09-13 and 08-10 (21228A4, H2, J1; 21256A3; 21222A4: lean ambient, 1.8 mg/s, SE at 11–18 s). It gives those flames a spurious limit at about 17 s. Relative to D1 it moves the median by ln −0.39 at 1 mg/s up to +0.60 at 12 mg/s, everywhere. Neither clock has any fuel dependence below 18 % ambient fuel. This is the likeliest train-side reason D3 lost extinction skill.

## 7. The prefix

A train prefix holds the allowed conditions (pressure, flow, O2, fuel fraction, T_ad, Z_st, fuel species), the time the phase was established, and the control stages that start at or before the cutoff. `observations.jsonl` is empty for both configurations. Control histories exist for 3 normal (21088A1, B1, D1) and 2 inverse (21196A2, F1) train runs. Before 5 s they hold only pre-ignition script steps, ignition at 0 s, and "Flame stabilization" at 2–4 s.

So the forward receives nothing but "still burning". It uses that only to remove the weak-class mass and to condition on survival. Raw `data.csv` files (radiometer, PMT, burner thermocouple) exist for those 5 train runs only and are not ingested. Whether they carry an early signal could not be tested.

## Proposals (ranked)

1. **Give the inverse burner clock a gate on ambient fuel richness.** The limit was reached only with 85 % O2 into at least 13.9 % fuel (07-28, 08-30, 09-27). Lean sessions (08-10, 08-16, 09-13) peaked at or below 200 °C.
   - Supported by: 6 sessions.
   - Contradicted by: none in train.
   - Effect: sharpens; removes a false competing cause; depends on fewer session facts.
2. **Write stops without session-specific atoms.**
   - Inverse: one smooth window covering 63–67 s, weighted by the 4/5-session and 15/18-flame counts; unscripted stops at the pooled rate (ln about −5.5).
   - Normal: a broad programmed hazard over 60–300 s at the pooled count rate; drop the 98, 126.5 and 275.5 s atoms.
   - Contradicted by: 09-27 (no 66 s stop), 06-11 A4 (passed 275 s).
   - Effect: widens.
3. **Bound the normal laws where train is silent.**
   - The flow-hinge lengthening at O2 ≥ 0.35 and ≤ 0.5 mg/s (lives above 2000 s, and 21125A2 contradicts).
   - The O2 coupling across the unsampled 0.25–0.33 band.
   - The burner slope below 0.5 mg/s, where medians of 2–5 s are below any observed heating time.
   - Effect: widens.
4. **Soften the normal flash cut.** Two sessions, with the cut position resting on 05-05 alone. At present every ≤5 % ethane or propane flame in enriched O2 is a sure flash, which train never tested.
   - Effect: widens.
5. **Inverse lean air.** Keep the lean-ambient flash: 3 flashes in 3 sessions, against 07-15's contradiction of the T_ad flash. But restrict unit 2's long main life to rich ambients; in lean air its only SE test (21228L1, 3 s) contradicts it. Lean-air survivors are mixed (3 s, 16 s, 65 s, 66 s), so widen there.
6. **Universal variable (needs a new forward).** The computed flame size r_f could replace D3's typed O2 coefficient in the reduced flow and the dilution units 2–3, with equal ordering on train.
   - Supported by: 03-29, 04-16, part of 10-18.
   - Contradicted by: 05-05.
   - Does not remove the butane soot term.

## What I could not determine

- Whether the validation loss comes from these regions: I did not look.
- T_ad above 1900 K on the grid: my estimate is crude there.
- Early raw signals.
- The cause of several stops: 21088K1, 21125K3, 21196M2/M2_1, and 09-27 A1/C1/H4.
- Inverse pressure.

All scripts and outputs are in `D:\Caches\Temp\claude\23fcab7049aa850b\bd40257b-5cbd-4ff9-b44f-69568de1670b\scratchpad\analysis4\acme\`; the main ones are `fl.py`, `verify.py`, `diag_check.py`, `item1.py`–`item7b.py`, `sweep.py`, `sweep_out.txt`, `shift.py` and `cases.json`.

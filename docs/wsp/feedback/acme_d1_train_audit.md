# ACME clocks D1/D2: session-transfer audit against the PSI-159 train rows

> Train partition only; nothing fitted. Written by a delegated analysis agent (claude-opus-5-5) on 14 November 2026;
> the author rechecked the claims it acted on against the train tables. Paths under `scratchpad` are the agent's
> working folder and are not part of the repository.

## Summary (≤400 words)

**What transfers across sessions:**
- **Normal flames, flow at enriched O2 (33–40 %).** The flow exponent d ln t / d ln m is −0.89 above 1.4 mg/s (5 pairs, 3 sessions) and −2.33 between 1.0 and 1.4 mg/s (15 pairs, 4 sessions). It holds for ethane and butane at 0.5, 1 and 2 atm. D1 gives −0.79 and −2.21.
- **Normal, pressure:** −0.4 to −0.5 per e-fold, in 3 sessions and both fuels. D1 has −0.33.
- **Normal, butane against ethane:** ×0.65–0.75 (D1 ×0.74).
- **Inverse:** life is flat in flow above 1.8 mg/s and collapses to 3–5 s at 1.34–1.36 mg/s. 85 % O2 lives about ×1.44 longer than 50 %. Propane and butane behave the same. The 66 s stop was used in 4 of 5 sessions.

**What does not transfer:**
- **Normal flames in air:** flow barely matters (exponent about 0; D1 −0.41 and −1.22). The 2–3 atm propane flames end at 5 s, which D1's 6 s floor forbids.
- **Normal fuel dilution:** its sign differs between sessions.
- **Normal programmed stops** at 61, 98, 125–127 and 275 s: each comes from 1–2 sessions.
- **Burner-limit setting:** about 455 °C on some days and ≥505 °C on others.
- **Inverse pressure:** the evidence points both ways.
- **Inverse flash (weak class):** T_ad does not separate it. Ethane-rich flames at 1785 K held for 36–66 s, while lean-ambient flames at 1835–1879 K flashed out.

No single variable orders self-extinction times across sessions as well as D1 does (0.86). For normal flames pressure (0.79) beats T_ad (0.72) and O2 (0.64). For inverse flames ambient fuel fraction is best (0.69).

**Best-supported proposals:**
- Normal floor 6 s → about 4 s.
- Normal low-flow lengthening should depend on O2 or T_ad: the hinge is at about 1.5 mg/s in enriched O2 and about 0.6 mg/s in air.
- Normal burner clock steeper in flow (≥4 instead of 3), keeping 0.5 mg/s at about 50 s.
- Inverse flash should depend on ambient fuel fraction (about −5 to −10 per e-fold).
- Inverse background stop rate: ln about −6 instead of −7.6.

**Could not be determined:**
- Propane in enriched O2 (no train run).
- Inverse pressure, and inverse behaviour below 7 % ambient fuel.
- Whether O2 or T_ad gates the low-flow lengthening.
- Whether the inverse TFP effect (×1.0–2.0) comes from the fibre or from the run's place in the day.
- A new session's burner-limit setting and starting burner temperature.

Outside the model's inputs: about 15 % of normal outcomes and about 40 % of inverse outcomes.

---

## Report

### A. The D1 clocks on every run

**Normal (81 runs, 7 sessions).**
- Self-extinction: median P(label) 0.97; median P(time within ×1.5) 0.59 (D2 0.47).
- Flow termination: median P(label) 0.72; median P(×1.5) 0.19 (D2 0.17).

Flagged (6 runs, 4 sessions):

| Run | Session | What happened | D1 |
|---|---|---|---|
| 21125A2 | 05-05 | Butane x_f 0.031, 0.5 mg/s, 39 % O2: SE at 8 s | floor + median 611 s; P(SE) 0.14 |
| 21130M3, N3, N4 | 05-10 | Propane in air, 2–3 atm, 0.5 mg/s: SE at 5 s | 24 s; the 6 s floor forbids it |
| 21106F3 | 04-16 | FT at 13 s with the burner at 313 °C | unknown stop |
| 21291A1 | 10-18 | FT at 167 s with hot/cool cycling | P(FT) 0.06 |

Weak but not flagged: the FTs at 1 mg/s at 61–127 s (P(FT) 0.28–0.50), and 21153A5/C4 at 274/276 s (P(×1.5) 0.04/0.20).

Median miss of D1 per session, ln(t / floor + median life):

| Session | Median miss |
|---|---|
| 03-29 | −0.04 |
| 04-16 | +0.12 |
| 05-05 | −0.18 |
| 05-10 | −0.40 |
| 06-02 | −0.17 |
| 06-11 | −0.03 |
| 10-18 | −0.32 |

**Inverse (73 runs, 7 sessions).**
- Self-extinction: median P(label) 0.98; median P(×1.5) 0.56 (D2 0.46).
- Flow termination: median P(label) 1.00; median P(×1.5) 0.68 (D2 0.50).

Flagged (10 runs, 5 sessions):

| Run(s) | What happened | D1 |
|---|---|---|
| 21196F1, F2, F6 | 15 % O2 into 29 % ethane, 1785 K: burned 36–66 s | p_weak 0.94, P(FT) 0.05–0.06 |
| 21256M1 | Flashed out at 1 s | p_weak 0.42 |
| 21228C2 | 50 % O2 at 9.07 mg/s burned to the 66 s stop | P(FT) 0.01 |
| 21209K1 | FT at 26 s, comment "Timed out" | — |
| 21228L2, 21242P1, 21270C1, 21270H4 | FT at 16, 21, 70, 83 s, burner not at the limit | unknown stops |

Median miss per session: 08-10 +0.27, 08-16 −0.02, 09-13 −0.08, 09-27 −0.14.

### B. Session transfer

**B.0 Leave one session out.** For each run, I took the nearest run from the other sessions. The distance scales were set beforehand, not tuned. Full table in `b_loso_out.md`.

| Configuration | Label agrees | Time within ×1.5 (nearest run) | After carrying along flow | D1, written on all sessions |
|---|---|---|---|---|
| Normal (81) | 67 | 37 | 38 | 57 |
| Inverse (73) | 68 | 41 | 41 | 53 |

The misses come from pressure, fuel and regime, not from flow:
- 06-02 against the 1 atm analogs: 0.5 atm runs came out longer (×1.24–1.37), 2 atm runs shorter (×0.62–0.79).
- 10-18 butane against ethane analogs: ×0.53–0.73.
- 05-10 at 0.5 mg/s: ×0.26 against 21162C1. The air low-flow lengthening seen at 1 atm does not carry to 2–3 atm.
- Inverse 09-13 (7 % fuel) against 08-10 (8 %, 3.3 atm): ×0.38–0.44.
- Inverse 09-27 and 08-30 do not use the 66 s stop.

**(i) Flow.** Pairs are in `b_out.md`.
- Normal, enriched O2: the steepening between 1.5 and 1.0 mg/s is the same in every session that tests it (exponents above). The named pairs include 21153C3/C2 −2.17, 21162A3/A2 −2.92, 21291G1/D1 −3.04 and 21106D1/B1 −2.30.
- Normal, air:
  - 06-11 stays at 9–10 s from 2.36 down to 1.0 mg/s, then lengthens: 11, 19 and 16 s at 0.75–0.5 mg/s (C2, C1, D1).
  - 05-10 gets shorter at low flow: 7 → 6 → 5 s.
  - Exponent below 1 mg/s −0.40 (20 pairs, 2 sessions; D1 −1.22). Between 1.0 and 1.4 mg/s 0.00 (7 pairs, 3 sessions; D1 −0.41).
- Inverse:
  - Exponent 0.00 above 3 mg/s (21 pairs, 4 sessions).
  - +0.08 at 1.5–3 mg/s (18 pairs, 3 sessions).
  - Collapse at 1.34–1.36 mg/s in 08-10 and 09-13, with both fuels.

**(ii) Oxygen.**
- Normal, within 06-02: 21153F1 (34.1 %) 44 s against A2 (37.6 %) 88 s, +1.98 per 10 % O2. D1 gives a ratio of 1.96, which matches.
- Normal, 06-11 air against 39–40 %: the effect per 10 % depends on flow.
  - 2.36 mg/s: +0.82
  - 1.5 mg/s: +1.04 to +1.11
  - 1.0 mg/s: +1.69 to +1.80
- D1 treats O2 and flow separately. It overstates the air-to-39 % ratio at high flow (8.4 against 4.8; 11.1 against 7.1–7.9). It is right at 1.0 mg/s.
- Inverse, 85 % against 50 %: ratio 0.89–2.83, median 1.44 (15 pairs, 3 sessions). D1 gives 1.5–1.7.

**(iii) Pressure.**
- Normal, 06-02 (×4 in pressure): 21153C1/A1 −0.40 and C6/A2 −0.50 per e-fold. C3 and G1 against A3 is about −0.5 even though C3/G1 had 3 % more O2.
- Normal, across sessions:
  - At 2.36 mg/s: 59 / 43 / 34 s at 0.5 / 1 / 2 atm.
  - At 1.5 mg/s: 88 / 71 / 51 s.
  - Butane 1 → 2 atm: −0.34 to −0.6.
- Inverse, within 08-16: a little shorter at higher pressure (non-TFP pairs 0.67–1.0).
- Inverse, across sessions: 08-10 (3.3 atm) runs longer than 08-16 (2.2 atm), the opposite. Not determined.

**(iv) Fuel and dilution.**
- Butane/ethane at the same conditions: 0.66 (21291D1/21162A2), 0.69 (G1/A3); at 2 atm 0.61–0.84 after carrying the flow.
- Propane in enriched O2 is not in train.
- Dilution, ethane 03-29 (x_f 1 → 0.25: 50/43, 42, 37 s): about zero once the 2.1 % O2 drift is accounted for.
- Dilution, butane 10-18 (x_f 0.3 → 0.15: 47 → 27 s and 161 → 83 s): about +0.4 per e-fold.
- Dilution, butane 04-16 (x_f 0.25 / 0.5: 60 / 62 s): none.
- Inverse propane against butane (08-16 against 09-13): 0.82–1.12, so no fuel effect.
- Inverse ambient fuel, 09-13 (7.0 % against 8.8 %): ratio 1.29–2.25 (D1 1.18–1.74).
- Inverse at 14 % fuel (09-27, 46–51 s) against 8.8 % (9–13 s): ×4–5, which D1 reproduces.

**(v) Which variable orders times across sessions.** I took pairs of runs from different sessions at matched flow and counted how often each variable orders them the same way as their burn times.

| Group (pairs) | Best single variables | Others | D1 |
|---|---|---|---|
| Normal, all (173) | pressure 0.79, T_ad 0.72, size proxy 0.70 | O2 0.64, Z_st 0.40 | 0.86 |
| Normal, enriched O2 only | pressure 0.83 | O2 0.42, T_ad 0.57 | — |
| Inverse, all (126) | ambient fuel 0.69 | T_ad 0.66 | 0.81 |

T_ad misorders 05-05: those flames are at 1785–1795 K yet last 17–35 s, against 7–10 s for air flames at 2110–2260 K.

### C. Flow terminations

**Normal: 17 FTs in 5 sessions.** No self-extinguished flame went above 428 °C, so a stop at ≥450 °C reads as the burner limit.

| Session | Run | m (mg/s) | FT (s) | Peak (°C) | Cause |
|---|---|---|---|---|---|
| 03-29 | 21088F1 | 0.5 | 28 | 532 | limit (inferred) |
| 03-29 | 21088D1 | 1 | 61 | 459 | operator: "FC=flow cut", MFCs off [op] at 64 s |
| 03-29 | 21088H1 | 1 | 62 | 417 | script |
| 03-29 | 21088K1 | 1 | 98 | 340 | unknown |
| 03-29 | 21088H2 | 1 | 127 | 422 | script |
| 04-16 | 21106F3 | 0.5 | 13 | 313 | unknown |
| 04-16 | 21106F1 | 0.8 | 72 | 505 | limit (stated) |
| 05-05 | 21125K3 | 0.75 | 125 | 481 | limit or script, ambiguous |
| 06-02 | 21153C5 | 0.5 | 49 | 512 | limit (inferred) |
| 06-02 | 21153A4 | 0.5 | 56 | 509 | limit (inferred) |
| 06-02 | 21153C7 | 1 | 61 | 335 | script |
| 06-02 | 21153A5 | 0.75 | 274 | 421 | script |
| 06-02 | 21153C4 | 0.75 | 276 | 436 | script |
| 10-18 | 21291J1 | 0.75 | 80 | 454 | limit (stated) |
| 10-18 | 21291H2 | 0.75 | 158 | 456 | limit (stated) |
| 10-18 | 21291A1 | 1.5 | 167 | 364 | cool-flame cycling, flow ended |
| 10-18 | 21291C1 | 1 | 246 | 435 | cool-flame cycling, flow ended |

**Which sessions used each stop time** (sessions that stopped a flame there / sessions with a flame still burning):

| Stop | Sessions |
|---|---|
| 61–62 s | 2 of 6 (03-29, 06-02) |
| 98 s | 1 of 6 |
| 125–127 s | 2 of 5 |
| 274–276 s | 1 of 2. 06-02 stopped both of its 0.75 mg/s flames; 06-11's 21162A4 burned past 275 s to SE at 304 s |

For a new session the defensible statement is only that a stop is likely at one of a few round durations. No specific time is supported.

**Burner limit, normal:**
- The limit setting varies: about 455 °C on 10-18, ≥505 °C on 03-29, 04-16 and 06-02.
- Ethane at 0.5 mg/s reached it in 49–56 s; ethane at 0.75 mg/s had not reached it after 275 s (421–436 °C). That is an exponent of at least 4.
- Butane at 0.75–0.8 mg/s reached it at 72, 80 and 158 s.
- O2 and pressure cannot be separated here.
- 21291J1 (80 s) against H2 (158 s) is confounded with their order in the day.

**Inverse: 32 FTs** (full list in the C table of `cdef_out.md`).

| Cause | Count | Runs |
|---|---|---|
| Script 66 s | 13 | includes 21196L2 at 63 s; 21196M2/M2_1 at 515 °C could also be the limit |
| Script 90 s | 2 | 21270A2, A4 (09-27) |
| Script/operator 136 s | 1 | 21209B1 |
| Burner limit | 5 | 21209A1 (544 °C), B2, 21242A1 (652 °C), 21270A3, H2 (455–460 °C) |
| Time-out | 1 | 21209K1 |
| Operator | 1 | 21196F1 |
| Automatic extinction detection | 1 | 21196A2: detected at 34 s, MFCs off at 37 s |
| Unknown | 6 | 21228L2, 21242P1, 21270H3, A1, C1, H4 |

- The 66 s stop was used in 4 of 5 sessions with a flame still burning at 65 s; in those sessions it stopped 13 of 14 such flames. 09-27 did not use it (0 of 4).
- 90 s and 136 s each come from one session.
- Burner-limit time shortens with ambient fuel: 21209A1 at 24 % took 26 s, B2 at 19.75 % took 48 s, 21242A1 at 43 % took 12 s.
- It lengthens with O2 flow: in 09-27, 2.68 mg/s reached it at 35–43 s, while 4.03 mg/s was still below it at 90 s (433 °C). The exponent is at least 1.8; D1 has 1.6.
- 50 % O2 never came near the limit.

### D. The first seconds

- **Normal:** all 10 one-second flashes are at T_ad 1602–1701 K (2 sessions, 0.125–1 mg/s, 1–3 atm). Every flame at ≥1785 K survived ignition. In train T_ad separates them cleanly, but the cut can sit anywhere between 1701 and 1785 K, and those runs are also exactly the "air plus x_f < 0.05" runs. The 5 s flames (2169–2197 K) are short normal lives, not flashes.
- **Inverse:** T_ad is not the separator.
  - Flashes were at 1835, 1868 and 1879 K.
  - Survivors include 1783–1787 K (07-15), 1873, 1885 and 1956 K.
  - All flashes were air injected into a lean 7–9 % ambient (Z_st 0.33–0.35): 3 of the 6 such runs flashed.
  - Of the 9 runs with ≤21 % O2 injected into ≥19.6 % fuel, none flashed.

### E. Repeats

| | Normal | Inverse |
|---|---|---|
| Repeat groups | 12 in 7 sessions | 7 in 4 sessions |
| Spread of ln t, max − min | median 0.15, range 0–0.38 | — |
| Pooled SD of ln t | 0.15 | 0.31 |
| TFP flag | no direction (0.68–1.19) | longer in 4 of 5 pairs (×1.0–2.0); the TFP run was always the later one |
| Order in the day | no direction | confounded with TFP |

- Normal: the only comment-backed repeat difference is 21106G6/G7 ("the igniter stayed in the flame longer") at 20 s against G9 at 14 s.
- Inverse TFP pairs: 21222B1/A1 1.70, 21228B1/A1 2.00, 21228J1/H2 1.20, 21256B1/A1 1.44, 21256J1/H1 1.00; and 21209B1 burned to 136 s while A3 stopped at 67 s.

### F. What the model cannot know

- **Script and operator stops:** 6–7 normal FTs and 16 inverse FTs.
- **Time-out, operator and automatic detection (inverse):** 21209K1, 21196F1, 21196A2.
- **Unknown stops:** 2 normal (ln rate −7.8 per flame-second); 6 inverse (7 unscripted stops in all, ln rate −5.8). D1 uses −7.6 for both.
- **Burner-limit setting and burner history:** they set the time of every limit stop.
- **Thermocouple-catalysed reignition on the final normal day:** 21291A1 and C1 are labelled FT instead of SE.
- **TFP fibre:** changes inverse SE times by up to ×2.
- **Igniter dwell:** ×1.4 (21106G6/G7).
- **Mis-recorded input:** 21256C2, "No nitrogen flowed", so its recorded 50 % O2 was really undiluted O2.

Share outside the inputs: about 12–13 of 81 normal runs and about 30 of 73 inverse runs.

### Proposals

**Normal:**
1. **Floor 6 → about 4 s.**
   - Support: 21130M3, N3, N4 (5 s) and M1, N1, N2 (6 s); 1 session, 2 pressures.
   - Would contradict it: a non-flash SE below 5 s.
   - Risk: low.
2. **Gate the low-flow hinge (unit 0) by O2 or T_ad.** The hinge should stay at about 1.5 mg/s in enriched O2 and move to about 0.6 mg/s in air: roughly +0.5 per unit c_o2, or +0.6 per unit c_tad, with b1 lowered to keep the enriched hinge where it is.
   - Support: 27 air pairs in 3 sessions; the enriched result is unchanged.
   - Would contradict it: an air flame at ≥1 atm whose life at 1.0 mg/s is ≥2× its life at 1.5–2.4 mg/s.
   - Risk: propane in enriched O2 is unchanged by construction; low-flow air lives get shorter.
3. **Burner clock flow coefficient 3 → ≥4**, keeping ethane at 0.5 mg/s at about 50 s and butane sooner.
   - Support: 06-02 (A4/C5 against A5/C4) and 21088F1; butane in 3 sessions.
   - Would contradict it: ethane at 0.75 mg/s reaching the limit within about 200 s.
   - Risk: the limit setting varies by session, so keep the clock wide (shape 2.5).
4. **Programmed stops as session-level facts:** 275.5 s atom p 0.70 → about 0.5 (1 of 2 sessions); keep the 98 s atom small.
   - Risk: small.
5. **(Weaker) Pressure −0.33 → about −0.45 per e-fold.**
   - Support: 3 sessions, 2 fuels.
   - Against it: 04-16 already runs +0.12 long.
   - Effect: about ±0.08 in ln t.

**Inverse:**
1. **Flash probability falls with ambient fuel fraction:** add about −5 to −10 per e-fold of c_xf to the weak-class logit, keeping the T_ad term.
   - Support: 3 flashes at 7–9 % in 3 sessions; 0 of 9 at ≥19.6 %.
   - Would contradict it: a flash in a fuel-rich ambient, or at ≥1950 K.
   - Effect: makes flashes more likely in leaner ambients such as 5.8 %.
2. **Background stop rate ln about −6 to −6.3**, or keep the small atoms at 26–63.5 s, but not both.
   - Support: 4 of 7 sessions.
3. **Keep the 66 s stop;** bring the 136 s atom (p 0.9 from 1 flame) to about 0.5. The 90 s atom comes from one session.
4. **(Weaker) Burner clock flow exponent ≥1.8.** Bounds only, from 2 sessions.

### Leave alone

- Normal flow law in enriched O2 above 1.0 mg/s.
- Normal O2 coefficient inside the enriched band.
- Normal butane coefficient −0.3 (the evidence says −0.3 to −0.43, within the repeat spread).
- Normal dilution coefficient +0.2 (sessions disagree on its sign).
- Normal T_ad flash law.
- The whole inverse self-extinction law: flow, 85/50 % O2, fuel, the fuel-fraction slope and the 13.5 % hinge.
- Inverse pressure.
- The spreads: repeat SD 0.15 sits well inside shape 4 (0.45) and shape 2.5 (0.73); train cannot choose between D1 and D2.
- Single or apparatus-specific runs: 21228C2, 21125A2, 21291A1/C1.

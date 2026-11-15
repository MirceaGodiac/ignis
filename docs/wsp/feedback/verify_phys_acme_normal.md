# Independent check of the normal-flame physics dossier on the train tests

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. An independent re-derivation with its own code; train tests only, no validation test was read.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# Independent check of `phys_acme_normal.md` (ACME CFI-G normal flames, train only)

Most of the dossier's numbers reproduce, but four things should change what the author types:

- **A seventh mis-tabulated row exists.** 21125D2 ran at a burner fuel fraction of about 0.037, not 0.045.
- **The clock fails on cells the dossier's table leaves out.** The ten 1 s flames are predicted at 7–13 s, and the three corrected butane-in-air rows at 0.5 atm at 39–98 s or "held".
- **Pressure is not p^-0.4 everywhere.** It is about p^-0.85 at 1.0 mg/s between 1 and 2 atm, for both ethane and butane.
- **A dilute burner gas does not burn longer in raw lives.** At equal fuel flow it burns the same or shorter.

Throughout, corner A is l1 = 7.35 mm with D1 = 21 mm²/s, and corner B is l1 = 11.0 mm with D1 = 42 mm²/s. Time zero is flame onset; my own radiometer onset agrees with the research5 onset within −0.34 to +0.03 s on all 80 tests.

## N1. Six rows with wrong inputs — CONFIRMED, plus one more row

I took the median readback of the fuel and nitrogen controllers over the fed window and the commanded volts from each test's setpoint file. I converted with textbook gas factors (ethane 0.50, propane 0.36, butane 0.26) and compared with the table.

| Test | Table | Readback / command | Verdict |
|---|---|---|---|
| 21125K1, K2, K3 | fuel fraction 0.031 | nitrogen command 0, readback −0.001 to −0.004 V; fuel fraction 0.30 | confirmed |
| 21125K1, K2, K3 | 1.5 / 3.0 / 0.75 mg/s | 1.50 / 3.01 / 0.74–0.75 mg/s; K1 reads 2.315 V, same as 21291A1 and D1 | flows are right |
| 21106G6, G7 | 0.8 mg/s | 0.486 (readback) – 0.497 (command); same volts as 21106F3 "0.5" | confirmed |
| 21106G9 | 0.8 mg/s | 0.651 – 0.662 mg/s | confirmed |
| **21125D2** | fuel fraction 0.045 | 0.0375 / 0.0370; nitrogen command 0.748 V against 0.595 V in its twin 21125D1 | **new: −17 %** |

- **Scan of the rest:** the other 73 tests with data agree with the table within 10 % by command, and their flows within 3 % by readback (ratio 0.97–1.02).
- **Readback-only disagreements above 10 %:** 21088H1, H2 and P1 give a fuel fraction of 0.556–0.572 by readback against 0.50 by command and table; 21130G7 gives 0.063 against 0.049.
- **Cause:** the nitrogen readback sits 0.024–0.061 V below its command in every diluted test (median 0.029 V, about 0.012 slpm). The fuel readback sits 0.006–0.047 V below. I cannot tell which is the true flow.
- **Not checkable:** 21130A9 has no data file.
- **Table consistency:** the table's Z_st reproduces from its own fuel and oxygen fractions within 1 % on all 81 rows. So Z_st and T_ad of K1–K3 were computed from the wrong fuel fraction.

## N2. Cycling flames 21291A1 and 21291C1 — CONFIRMED on times; plateau fraction and gap lengths WEAKENED

I scanned the photomultiplier sum (below 0.10 V for at least 0.3 s) inside the fed window of all 80 tests. Only these two relight.

| Test | First hot extinction | Dark gaps (s from onset) | Burner at each extinction | Radiometer in gap / before |
|---|---|---|---|---|
| 21291A1 | 40.6 s (40.4–40.7 across thresholds 0.03–0.5 V) | 40.6–52.7, 80.8–85.3, 104.5–109.4, 128.5–133.6, 152.3–157.0 | 269, 335, 348, 355, 358 °C | 29–36 % (+60 channel), 27–32 % (+30) |
| 21291C1 | 89.0 s (87.3–89.2) | 89.0–96.1, 125.8–132.5, 163.6–170.0, 197.1–203.4, 232.2–238.4 | 409, 422, 427, 430, 431 °C | 36–38 % (+60), 32–34 % (+30) |

- **Counts:** five gaps and six hot flames each, as claimed.
- **Gap lengths:** I get 12.1 s then 4.5–5.1 s for A1 and 6.2–7.1 s for C1. The dossier's 10.0, 2.7 and 5–5.6 s depend on its 0.03 V threshold, because the gap signal hovers at 0.01–0.04 V.
- **Plateau:** it is real but sits at 27–38 % of the hot signal, not 30–45 %. It falls only 5–12 % across a gap. In the 21 other tests still fed 10 s after extinction, the radiometer is at 6–21 % and falling.
- **Faint light in the gaps:** the photomultiplier reads 0.008–0.024 V there, against −0.001 V after a normal extinction.

| Test | Burner at hot extinction | After extinction (+60 radiometer) |
|---|---|---|
| 21291B1 (2 atm) | 189 °C | 0.10 V at +15 s, falling |
| 21291C2 (2 atm) | 177 °C | 0.11 V, falling |
| 21291H1 (2 atm) | 214 °C | 0.09 V, falling |
| 21291D1 (1 atm) | 256 °C | 0.105 V, falling |

The burner temperatures match the dossier (177–215, 257, 270, 408 °C). The "above about 515 K" rule holds only at 2 atm, where the threshold lies between 214 and 269 °C. At 1 atm, four butane flames went out with the burner at 256–413 °C and no plateau (21291D1, G2, J3, G1).

## N3. The clock — table REPRODUCED; the form is right only in a narrow regime

I implemented the equations of sections 1.2 and 4 as stated, with table T_ad and Z_st, and corrected inputs for the seven rows. For K1–K3 I read T_ad across from 21291D1, G1 and G2 (2652, 2645, 2639 K) rather than computing it.

The dossier's table reproduces within 3 % on every cell. Its example row gives 37.5 / 42.2 / 61.8 / 57.6 / 73.0 s at corner A and 46.3 / 70.9 / 258 / 208 / 638 s at corner B. One exception: for propane in air I get 22.9–33.8 s at corner B, not 23–65.

Lives in seconds (observed / corner A / corner B):

| Cell | Flow mg/s (test) | Observed | A | B |
|---|---|---|---|---|
| 30 % ethane, 1 atm, 37–40 % O2 | 2.36 (21162A1) | 43 | 37.5 | 46.3 |
| | 1.5 (A2) | 71 | 42.2 | 70.9 |
| | 1.0 (A3; B1) | 232; 158 | 61.8; 57.6 | 258; 208 |
| | 0.85 (A4) | 304 | 73.0 | 638 |
| 30 % ethane, 2 atm | 2.36 (21153C1) | 34 | 37.9 | 41.7 |
| | 1.5 (C2; C6) | 51; 44 | 37.7; 36.7 | 48.8; 46.5 |
| | 1.0 (C3; G1) | 123; 131 | 43.7; 43.2 | 78.9; 77.2 |
| | 0.75 (C4) | alive at 276 | 56.3 | 182 |
| 30 % ethane, 0.5 atm | 2.36 (21153A1) | 59 | 39.3 | 58.7 |
| | 1.5 (A2; F1 at 34 % O2) | 88; 44 | 52.1; 42.6 | 148; 90.6 |
| | 1.0 (A3, 36.4 % O2) | 258 | 112 | held |
| | 0.75 (A5, 33.2 % O2) | alive at 274 | 257 | held |
| Ethane 100 / 50 / 25 %, 0.93 atm | 2.0 (21088A1, B1; G1; J1) | 50, 43; 42; 37 | 48.4, 47.8; 41.8; 34.2 | 76.9, 75.0; 59.5; 43.5 |
| | 1.0 (D1; H1; H2; K1) | alive at 61; 62; 127; 98 | 115; 80.6; 72.8; 50.0 | 7421; 671; 439; 150 |
| 50 / 25 / 12.5 % butane, 0.5 atm | 1.6; 1.6; 1.3 (21106B1; A4; C1) | 62; 60; 63 | 77.5; 64.9; 52.4 | 436; 256; 194 |
| | 1.2 (D1) | 120 | 145 | held |
| 30 % butane, 1 atm | 3.0 (21125K2) | 17 | 40.1 | 47.0 |
| | 1.5 (21291D1; 21125K1) | 47; 35 | 51.1; 50.3 | 105; 101 |
| | 1.25 (G2); 1.0 (G1) | 60; 161 | 58.9; 91.9 | 171; 1336 |
| | 0.75 (21125K3) | alive at 125 | 250 | held |
| 15 % butane, 1 atm | 1.5 (J2); 1.0 (J3) | 27; 83 | 38.5; 54.6 | 61.3; 190 |
| 30 % butane, 2 atm | 1.75 (B1, C2) | 32, 27 | 40.4, 40.5 | 51.2, 51.3 |
| | 1.5 (A1); 1.0 (C1), first hot extinction | 40.6; 89 | 42.7; 53.0 | 60.3; 119 |
| 15 % butane, 2 atm | 1.25 (H1); 0.75 (H2) | 35; limit trip at 158 | 36.7; 52.1 | 51.6; 156 |
| 30 % ethane, air, 1 atm | 2.36 / 1.5 / 1.5 / 1.0 | 9 / 9 / 10 / 9 | 21.9 / 19.5 / 19.3 / 18.8 | 21.6 / 20.6 / 20.3 / 22.1 |
| | 0.75; 0.5, 0.5 | 11; 19, 16 | 19.5; 24.0, 23.1 | 26.3; 50.5, 46.6 |
| Ethane 100 %, air; 50 %, 18 % O2 | 2.0, 1.0; 2.0, 1.0 | 9, 13; 7, 8 | 24.9, 23.9; 19.3, 16.5 | 25.9, 30.9; 19.0, 18.3 |
| 50 % butane, air, 0.5 atm (corrected) | 0.49, 0.49; 0.655 | 20, 20; 14 | 98, 93; 38.8 | held, held; 191 |
| 30 % propane, air, 2 and 3 atm | 0.5–2.0 (seven tests) | 5–7 | 20.8–27.4 | 22.9–33.8 |
| 3 % butane, 1795 K (21125A2) | 0.5 | 8 | 10.9 | 15.8 |
| T_ad 1600–1700 K (ten tests) | 0.125–1.0 | 1 | 6.6–9.0 | 7.1–12.7 |

Where the form is right and wrong, as predicted / observed:

- **Far from the holding flow, enriched oxygen:** corner A is right. The ethane tests at 2.0–2.36 mg/s and 0.93–2 atm give 0.87–1.11; the one at 0.5 atm gives 0.67.
- **Near the holding flow:** corner A is 2–4 times too short (0.24–0.43 for ethane at 1.0 and 0.85 mg/s). Corner B is right at 1 atm (1.00–1.31) but gives 2.1 at 0.85 mg/s.
- **Pressure:** neither corner works.
  - Corner A is flat at high flow (39 / 38 / 38 s against 59 / 43 / 34 observed at 0.5 / 1 / 2 atm).
  - Corner B gives about 60 % of the observed drop at high flow (59 / 46 / 42 s).
  - At 2 atm near holding, corner B is too short (0.6 at 1.0 mg/s) and predicts extinction at 182 s for a flame alive at 276 s.
  - At 0.5 atm, corner B says "held" for flames that went out at 120 and 258 s.
- **Air:** too long everywhere.
  - Ethane at 1 atm: 1.3–2.8 (A) and 2.0–2.9 (B).
  - Propane at 2–3 atm: 3.5–6.8.
  - 50 % butane at 0.5 atm: 2.8–4.9 (A); B gives held or 13.7.
- **Butane against ethane:** corner A is right for butane at 1.25–1.5 mg/s (1.0–1.4), 2.4 times too long at 3.0 mg/s and 0.6 at 1.0 mg/s. Corner B is too long for every butane row (1.3–8.3).
- **Dilution:** no consistent error; see N8.
- **Weak flames:** 21125A2 is fair (1.4–2.0). The ten 1 s flames are predicted 6.6–12.7 times too long; the 3 s offset alone exceeds their life.
- **Flow-terminated rows:** corner A contradicts six of fifteen, corner B two (21153C4 and 21291H2).

## N4. Y_F·L/Θ — CONFIRMED, with a lower bound of 0.045

| Group | Range | Median | Rows |
|---|---|---|---|
| T_ad above 2000 K, all | 0.0447–0.0583 (±13 %) | 0.0552 | 70 |
| Enriched oxygen | 0.0520–0.0583 (±6 %) | 0.0559 | 49 |
| Air or 18 % O2 | 0.0447–0.0478 | 0.0476 | 21 |
| T_ad 1600–1800 K (outliers) | 0.068–0.083 | – | 11 |

All 21 air rows fall below the claimed 0.048. The quantity is not constant between air and enriched oxygen: air is 15 % lower, the opposite direction to what the short air lives need.

## N5. Burner slope thresholds — WEAKENED

I took a two-point slope between 3 and 6 s on the burner thermocouple (type K, cold junction 23.5 °C). My peaks match the table within −2.1 to +1.2 °C on all 80 tests.

| Slope 3–6 s (K/s) | Tests | Peak °C | How it ended |
|---|---|---|---|
| 23.0 | 21088F1 | 532 | limit abort |
| 20.3 | 21106F3 | 313 at 13.6 s | "extinction detected" abort, not a limit |
| 18.6 | 21153C5 | 512 | no stamp; crossed 500 °C, flow off 4.0 s later |
| 16.1 | 21153A4 | 509 | limit abort |
| 14.4 | 21106F1 | 506 | limit abort |
| 13.6 | 21153C4; 21125K3 | 436; 480 | scripted stop |
| 13.3 | 21088D1 | 458, still rising | scripted stop at 62 s |
| 12.6 | 21291H2; 21291J1 | 454; 452 | limit abort at the 450 °C limit |
| 12.4 | 21291C1 | 434 | cycling flame |
| 11.6 | 21088H1 | 415, still rising | scripted stop at 62 s |
| 10.5–10.8 | 21162A4; 21088H2; 21153A5 | 405; 421; 421 | – |
| 9.8 | 21291G1 | 426 (table 428) | extinction at 162 s |
| below 9.7 | 47 tests | below 405 | – |

- **The three bands reproduce numerically.** Four tests at 14.4–23.0 K/s peak at 506–532 °C; six at 12.4–13.6 K/s peak at 434–480 °C.
- **The middle band is censored.** Two of its six tests were cut by the 450 °C limit and one by the script while still rising.
- **The lower band has one exceedance.** 21291G1 peaks at 426–428 °C with 9.8 K/s.
- **The 500 °C threshold is thin.** It sits between 13.6 and 14.4 K/s, a 0.8 K/s gap. Moving the window one second earlier raises slopes by 0.5–2.6 K/s.
- **Under the 450 °C limit the slope does not separate.** 13.6 K/s gave 436 and 480 °C; 12.6 K/s tripped.
- **Flow says nearly the same.** Every 500 °C trip is at 0.8 mg/s or less with a fuel fraction of 0.3 or more.
- The other 17 tests died before 6 s, so they have no flame slope.

## N6. Limit trips — CONFIRMED, numbers depend on the definition of "off"

| Test | Crossed limit at | Flow off (readback) | Temperature at flow off | Peak |
|---|---|---|---|---|
| 21088F1 (March) | 500 °C at 24.8 s | 28.0 s | 531 °C | 532 |
| 21106F1 (April) | 66.3 s | 70.9 s | 505 °C | 506 |
| 21153A4 (June) | 52.1 s | 55.9 s | 509 °C | 509 |
| 21153C5 (June) | 44.4 s | 48.4 s | 511 °C | 512 |
| 21291H2 (October) | 450 °C at 156.4 s | 157.6 s | 452 °C | 454 |
| 21291J1 (October) | 79.0 s | 79.5 s | 450 °C | 452 |

The dossier's 502–522 °C is read at the one-second timeline stamp, which falls one to two seconds before the readback drops. By readback it is 505–531 °C, with a delay of 3.2–4.6 s after crossing 500 °C. October is 450–452 °C with a delay of 0.5–1.2 s.

Before October, 21125K3 reached 480 °C on 5 May and 21088D1 reached 458 °C in March without a trip. The 10 May and 11 June sessions have no test above 405 °C, so they give no direct evidence of their limit.

## N7. Calorimetry — CONFIRMED approximately; re-derivable as arithmetic

I evaluated V·ΔP/(γ−1) over fuel flow × heating value × 40 s, on channel PTA3, for the 25 tests alive from 20 to 60 s.

| Pressure | Dossier | Mine, raw | Minus injected moles | Tests |
|---|---|---|---|---|
| 0.5 atm | 0.04–0.09 | 0.056–0.094 | 0.051–0.082 | 7 |
| 1 atm | 0.08–0.14 | 0.102–0.153 | 0.095–0.138 | 12 |
| 2 atm | 0.13–0.19 | 0.150–0.193 | 0.136–0.178 | 6 |

What it assumes:

- **Calibration 0.4 V/bar:** verified. Pre-ignition PTA3 is 0.7145 / 0.9169 / 1.3265 V at the three table pressures, which gives 0.400–0.403 V/bar.
- **Volume 83 L:** not verifiable from the raw files. The table's oxygen before and after imply 87–90 L if water stays vapour and 67–70 L if it condenses.
- **γ = 1.4, complete combustion, lower heating value.**
- **No correction for injected moles.** They account for 4–21 % of the rise; the third column subtracts them.
- **PTA3, not PTA2.** PTA2 gives fractions 1.1–3.5 times larger and is not chamber pressure during flow.

Within each pressure the fraction rises as flow falls; the 0.75 mg/s tests are the highest.

## N8. The four trends

| Trend | Cells (lives in s) | Ratio | Verdict |
|---|---|---|---|
| Pressure, 2.36 mg/s ethane | 21153A1 59 (0.5 atm), 21162A1 43 (1), 21153C1 34 (2) | exponents −0.46, −0.34; −0.40 overall | CONFIRMED at high flow |
| Pressure, 1.5 mg/s | ethane 21162A2 71 → 21153C2 51; butane 21291D1 47 → A1 40.6 | −0.48; −0.21 | confirmed |
| Pressure, 1.0 mg/s, 1 → 2 atm | ethane 21162A3 232 → 21153C3 123, G1 131; butane 21291G1 161 → C1 89 | −0.92, −0.82; −0.86 | **WEAKENED: twice as steep near holding** |
| Pressure, propane in air, 2 → 3 atm | 21130M1/N1, M2/N2, M3/N3 | 0 to −0.38 (lives 5–7 s, 1 s resolution) | cannot tell |
| Air against the law | 30 % ethane, 1 atm, 21162C3, C4, C5, F2 | corner B, which fits 39 % O2 at 1 atm, is 2.0–2.5 times too long | CONFIRMED for ethane near 1 atm |
| | propane 2–3 atm; 50 % butane 0.5 atm | 3.5–6.8; 2.8–4.9 or held | understated there |
| Butane / ethane, same flow, pressure, O2 within 0.5 % | 21291D1/21162A2; G1/A3; A1/21153C2; C1/C3, C1/G1 | 0.66; 0.69; 0.80; 0.72, 0.68 | WEAKENED: 0.65–0.80, not 0.5–0.7 |
| Dilution, raw, same fuel flow | ethane 50 % and 25 % against 100 %; butane 25 / 50 %; butane 15 / 30 % at 1.5 and 1.0 mg/s | 0.98, 0.86; 0.97; 0.57–0.77, 0.52 | **REFUTED as a raw trend** |
| Dilution, against corner A | ethane 1.11 / 0.97 → 1.00 → 0.92; butane 0.5 atm 1.25 → 1.08 → 0.83; butane 1 atm 1.09–1.44 → 1.43 and 0.57 → 0.66 | outlives the law in two series, not in the October 1 atm series | WEAKENED |

- **Pressure:** the comparisons from 0.5 to 1 atm at 1.5 and 1.0 mg/s are confounded, because the 0.5 atm tests ran at lower oxygen.
- **Butane against ethane:** the 0.49 end of the dossier's range needs the corrected row 21125K1 (35 s), which itself differs from its repeat 21291D1 (47 s). I see no sign that the gap is larger at high flow, apart from 21125K2.
- **Raw air against enriched** at the same flow is 0.21 at 2.36 mg/s, 0.13 at 1.5 and 0.04–0.06 at 1.0.

## Numbers to hand the author

**Corrected inputs**

| Row | Value | Range, and what it rests on |
|---|---|---|
| 21125K1, K2, K3 | fuel fraction 0.30; Z_st 0.202 / 0.200 / 0.198; T_ad about 2640–2655 K | nitrogen readback and command both zero; T_ad read across from 21291D1, G1, G2 |
| 21106G6, G7 | 0.49 mg/s | 0.486–0.497; same volts as 21106F3 |
| 21106G9 | 0.655 mg/s | 0.651–0.662 |
| 21125D2 | fuel fraction 0.037 | 0.0370–0.0375; T_ad below the tabulated 1701 K, not computed |
| Lightly diluted rows, if the readback is the truth | 21088H1, H2, P1: 0.50–0.57; 21088G1, N1: 0.50–0.53; five 15 % butane rows: 0.150–0.163 | constant 0.03 V offset; undecided |

**First hot extinctions:** 21291A1 at 40.6 s (40.4–40.7), 21291C1 at 89.0 s (87.3–89.2).

**Holding flow as bracketed by train** (enriched oxygen):

| Pressure | Went out | Still alive at the stop |
|---|---|---|
| 0.5 atm, 30 % ethane | 1.0 mg/s at 258 s (36.4 % O2) | 0.75 mg/s at 274 s (33.2 % O2) |
| 0.5 atm, 50 % butane | 1.2 mg/s at 120 s | 0.8 mg/s at 72 s (limit trip) |
| 1 atm, 30 % ethane | 1.0 mg/s at 158–232 s; 0.85 mg/s at 304 s | nothing lower tested |
| 1 atm, 30 % butane | 1.0 mg/s at 161 s | 0.75 mg/s at 125 s (21125K3) |
| 2 atm, 30 % ethane | 1.0 mg/s at 123–131 s | 0.75 mg/s at 276 s |
| 2 atm, 30 % butane | 1.0 mg/s at 89 s (first hot extinction) | 15 % butane, 0.75 mg/s at 158 s (limit trip) |

The 1 atm flame at 0.85 mg/s outlived both "held" flames at 0.75 mg/s, which were only watched to 275 s. Train therefore cannot order the holding flow by pressure; it only says it is below 1.0 mg/s at all three.

**Burner:** a 3–6 s slope of 14.4 K/s or more reached 500 °C (four tests, 14.4–23.0); 13.6 K/s or less did not (maximum 480 °C). The limit is 500 °C with 3.2–4.6 s of delay through June, and 450 °C with 0.5–1.2 s in October.

## What the dossier did not say

- **21291A1's stop at 167 s was the script's extinction detector** (stamp at 162.9 s plus 3 s), not the 240 s script. 21291C1's 246 s is the scripted end.
- **21106F3's "FT at 13 s" is an "EXTINCTION DETECTED" abort** with the burner at 313 °C. It is neither a limit trip nor a scripted stop.
- **21153C5 has no abort stamp** but is a limit trip by temperature.
- **PTA2 is not chamber pressure during flow.** It jumps 0.6–3 kPa when the burner flow starts, keeps rising, and falls back to PTA3 when the flow stops.
- **The fuel command leads flame onset by 2.3 s** (median; 1.4–8.8 s).
- **21153G1 flickers for about 10 s before going out** (first dark at 126.3 s, final at 130.2 s, table 131 s).
- **Two other onset or end mismatches:** 21162B1 goes dark at 162.8 s against a table 158 s, and 21162F2 at 7.4 s from onset against a table 10 s.
- **The clock table in 2.2 omits 13 train rows** where the form fails worst: the ten 1 s flames and 21106G6, G7, G9.

## Tests opened and files written

- **Opened:** the 80 train tests of `acme_normal` that have a data file (every id from `lib.partition_ids("acme_normal", "train")` except 21130A9, which has none). For each I read `_data.csv`, `_mfcSetpoints.csv` and the "as run" file, and `_TP.txt` for the 79 that have one (21291J2 has none). No comments file was opened.
- **Reference files:** `rawlib.py`, `acme_raw_signals_train.md`, and `tests_burner.csv` (onset clock only, rows filtered to the 81 train names). `tp_timelines.csv` was opened only to print its header and first four rows, which are train tests; it was not used.
- **Nothing outside the train list was opened.** Raw paths were built from train names only and the raw folder was never listed. No validation record, test partition, `holdout` folder, `runs/dev/data_ext/`, `--final-test` or `tools/wsp/dev_eval.py` was touched. Nothing under `runs/dev/research6/acme_normal/` was run or read. No published per-test result was read, no git-tracked file was modified, nothing was fitted, and no helper was launched.

Files are in `D:\dev\SpaceApps-Ardealu\runs\dev\research7\verify_normal\`:

- `vlib.py`
- `n1_readbacks.py`, `n1_readbacks.csv` (first pass, in-flame window only; superseded for the 1 s flames)
- `n1b_cmd_vs_readback.py`, `n1b_cmd_vs_readback.csv`
- `n2_dark.py`, `n2_dark_all.csv`, `n2_after_extinction.csv`
- `n3_clock.py`, `n3_clock_table.csv` (all 81 rows, both corners), `n3_summary.py`
- `n5_burner.py`, `n5_burner.csv`
- `n7_calorimetry.py`, `n7_calorimetry.csv`
- `n8_trends.py`, `n8_trends.csv`
- `cache\` (80 `.npz` files, full-rate copies of the train data files)

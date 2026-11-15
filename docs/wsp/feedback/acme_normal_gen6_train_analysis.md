# ACME normal flames, sixth generation: train analysis for a new candidate

> Report of an analysis sub-agent (claude-fable-5-2, high effort), 15 November 2026, saved from its
> task transcript by script, unedited below this note. It read the TRAIN partition only, fitted
> nothing and wrote no weight. Its scripts and outputs are in `runs/dev/gen6/acme_normal/`
> (git-ignored). What the author rechecked, and what was typed from it, is recorded in
> `docs/wsp/PROTOCOL.md` section 12 and in the dossier; a number in this report is not a typed
> number until it appears there.

# Evidence for a new `acme_normal` candidate, from TRAIN only

Tags: **[M: file]** = measured by me with that script under `runs\dev\gen6\acme_normal\`; **[D]** = from the dossier, not rechecked. Nothing was fitted and no weight was written. Three single stated alternatives were scored once each on train and are marked.

## 0. Verdicts

| Item | Verdict | Why, in one line |
|---|---|---|
| A. Limit by era | **Type it**, expect no train gain | Two settings ordered by date in 9 sessions; with the present heating law train moves 0.0274 → 0.0281 |
| B. Thin filament | **Do not type** | 3 matched pairs: +0.04, −0.15, +0.15 in ln life |
| C. Corrected inputs | **Needs an input we do not have**, and a retype | The script file gives true flows before ignition; several typed numbers stand on the wrong rows, and P1 scores worse on corrected inputs |
| D. Pressure near the holding flow | **Do not type** | The hinge member already has it; bolted onto the growth member it worsens extinction 0.0518 → 0.0601 |
| D. Flash head start | **Do not type** | Real, but not separable from the 4 s floor; one matched pair |
| D. Cool flame | **Do not type** | Train loss is a label convention tied to one day's hardware; P1 already forecasts the first hot extinction |
| D. Heating law time constants | **Do not type a second constant** | Single 30 s law is within about ±10 % at 30, 60 and 120 s in six sessions |
| E. Planned-stop offset (new) | **Type it, after you recheck** | Normal-flame scripted stops fall after the stage sum, not 0.4 s before; three tests land on the wrong side of the 60 s horizon |
| E. Hinge burner clock ignores dilution (new) | Structural defect, n = 1 | 21125A2 alone is 16 % of the extinction loss and 11 % of the flow-termination loss |

## 1. Instrument check

I transcribed both members and the committee into Python (`p1.py`) and the evaluator's per-case contribution (`score.py`).

- Events at 120 s agree with the stored feedback on all 210 cases to 3·10⁻¹⁰ **[M: p1_check.py]**.
- Per-case contributions agree to 3·10⁻¹⁰; case means 0.05175 / 0.02740 **[M: e_losses.py]**.
- The script path gives 0.05364 / 0.02149, the card's 0.0536 / 0.0215 **[M: a_era_script.py]**.

## A. Burner limit by era

**Normal-flame train sessions** **[M: a_metrics.py, a_sessions.py]**

| Session | Tests | Limit trips | Crossed limit → flow off (s) | T at flow off / peak (°C) | Other evidence |
|---|---|---|---|---|---|
| 03-29 | 13 | 21088F1 | 500 at 24.9 → 27.9 (3.0 s) | 529 / 531 | 21088D1 above 450 °C for 3.6 s, peak 457, scripted stop, no trip |
| 04-16 | 9 | 21106F1 | 66.7 → 71.1 (4.4 s) | 505 / 506 | 26 s above 450 before the trip; 21106F3 is an "extinction detected" abort at 15 s, 313 °C |
| 05-05 | 6 | none | – | – | 21125K3 above 450 °C for 69 s, peak 480, no trip |
| 05-10 | 15 | none | – | – | highest peak 91 °C: silent |
| 06-02 | 14 | 21153A4, 21153C5 | 52.6 → 56.1 (3.5 s); 44.4 → 48.2 (3.8 s) | 508 / 509; 511 / 512 | C5 has no reason stamp |
| 06-11 | 12 | none | – | – | highest peak 405 °C: silent |
| 10-18 | 12 | 21291H2, 21291J1 | 450 at 156.5 → 157.5 (1.0 s); 79.2 → 79.5 (0.3 s) | 452 / 454; 450 / 452 | C1 433 and G1 426, no trip |

**Sessions between 30 August and 27 September** are inverse-flame train sessions **[M: a_inverse_limit.py]**:

| Session | Evidence |
|---|---|
| 07-15, 07-28 | 21196M2, M2_1, 21209A1, B2: off 2.8–3.5 s after crossing 500 °C, 6.7–14.5 s above 450 |
| 08-10, 08-16 | silent (peaks 178, 198 °C) |
| 08-30 | 21242A1: off 3.4 s after 500 °C, peak 650 |
| 09-13 | silent (peak 174 °C) |
| 09-27 | 21270A3, 21270H2: off 0.8 and 0.7 s after 450 °C |

- **By date or by session:** seven sessions show 500 °C, two show 450 °C, five are silent, none shows both. The change lies between 30 August and 27 September; the one session inside is silent.
- **Dwell:** 2.8–4.4 s after 500 °C (9 trips), 0.3–1.0 s after 450 °C (4 trips).
- **Knowable before ignition:** not from CFI-G's files. The script parameter file has 80 keys and none on burner temperature; the private root adds only `programmed_stop` **[M: root_keys.py]**. The s-Flame procedure (PSI-23 page cards, pp. 12–36) shows the ACME script family has `burnerTemperatureLimitC` and `burnerTemperatureLimitDurationSecs`, polled once a second. That matches three polls against one. Only the session date carries the era.

**Effect of era weights on train** (510 °C alone through June, 455 °C alone on 18 October; one alternative, scored once) **[M: a_era.py, a_era_script.py]**

- 48 of 210 cases (16 runs) move by more than 0.01 in P(flow termination).
- 36 cases (12 runs before September) go down, by at most 0.02 at 120 s.
- 12 cases (21291G1, C1, J1, H2) go up by 0.07–0.09.
- Case means: 0.05175 / 0.02740 → 0.05169 / 0.02812 on shared records; 0.05364 / 0.02149 → 0.05356 / 0.02168 with the planned stop.
- By run, flow termination summed over cutoffs: J1 −0.054 (tripped at 80 s); H2 +0.077 (tripped at 158 s, outside the window); G1 +0.059 and C1 +0.047 (never tripped).
- The March scripted stops 21088D1 and H1 get +0.023 and +0.030 worse on shared records only: the 455 °C component had been covering for stops the model could not see.

**Proposal**

- **Statement:** one limit per test, chosen by session date.
- **Reasoning:** an apparatus setting of the script, not a mixture within a session.
- **Input:** the session day, already in the shared records' `session_id` and run id; the forward does not read it today.
- **Parameters:** `bt.limit_c` = 500 and 450 °C (apparatus; my thermocouple reading is good to about ±2 K). `bt.limit_dwell_s` = 3–4.4 and 0.3–1.0 s (three polls and one poll of 1 s). An era boundary between 2021-08-30 and 2021-09-27 (uncertainty: four weeks, one silent session).
- **Direction on train:** flat to slightly worse.
- **Risk:** low for the two levels; the boundary date rests on train alone.
- Typing 500 °C plus dwell instead of 510 °C changes trip times by about 1 s.

## B. Thin filament

**[M: b_fiber.py, b_pairs.py]**

- `fiberTestPosition` is 17000 or 17200 in 18 of 80 train script files: all 13 of 03-29, 21153F1, 21153G1, 21162B1, 21162D1, 21291B1. It is 0 in the other 62.
- 21130A9 has neither a script file nor a data file.
- The table's TFP column says Yes for 15; the script disagrees on 21088A1, G1, L1.
- The header field `Fiber` is 1 on every test and is useless.
- The private root does not carry the field. The card counts 108 planned stops for 109 train and validation runs and 80 are train, so every validation run has a script file; I did not open them.

| Fibre test | Life (s) | Partner without | Life (s) | ΔX_O2 | ln ratio |
|---|---|---|---|---|---|
| 21153G1 (1.0 mg/s, 2 atm) | 128.3 | 21153C3 | 122.9 | −0.003 | +0.04 |
| 21162D1 (0.5 mg/s, air) | 16.7 | 21162C1 | 19.4 | −0.002 | −0.15 |
| 21291B1 (1.75 mg/s butane, 2 atm) | 31.2 | 21291C2 | 26.8 | −0.001 | +0.15 |
| 21153F1 | 44.0 | 21153A2 | 88.8 | −0.035 | confounded |
| 21162B1 | 163.2 | 21162A3 | 232.3 | −0.011 | confounded |

- **Physics:** three 14 µm SiC fibres radiate well under 1 % of the heat release (my estimate, about 0.1 W per fibre). The fibre cannot move a radiative extinction that is global. It can matter for a flame dying by local bursts, which is the inverse-flame finding.
- **Recorded end:** the photomultiplier end differs from the table by a median 0.8 s with fibre (n = 13, largest 5.2 s on 21162B1) and 0.5 s without (n = 41).
- **Verdict:** no parameter. Three clean pairs in three sessions, mixed sign, inside repeat scatter.

## C. Table errors

**Normal-flame train rows that disagree** **[M: c_table.py]**

I converted the script file's flows with N2-equivalent factors 0.50, 0.36, 0.26 and 24.15 L/mol. The other 73 rows with data then agree with the table within about 4 %. Script file, commanded volts and readbacks agree with each other on every row.

| Row | Table | Script file and command | Readback |
|---|---|---|---|
| 21106G6, G7 | 0.80 mg/s | 0.50 mg/s (same 0.06 slpm as 21106F3) | 0.48 |
| 21106G9 | 0.80 mg/s | 0.66 mg/s | 0.65 |
| 21125D2 | fuel fraction 0.045 | 0.037 (nitrogen 0.299 slpm against 0.238 in its twin D1) | 0.039 |
| 21125K1, K2, K3 | fuel fraction 0.031 | 0.300 (nitrogen command 0); flows 1.51, 3.02, 0.75 are right | 0.300 |

- 21130A9 cannot be checked.
- **The test labelled FT at 43 s that went out at 93 s is 21270H3, and it is an inverse-flame train row.** Flame end 93.8 s, flow command off 95.6 s **[M: a_inverse_limit.py, metrics_acme_inverse.json]**.
- Corrected T_ad and Z_st for K1–K3: 2652 / 2645 / 2639 K and 0.202 / 0.200 / 0.198, read across from the October rows; Z_st from its definition matches the table to 0.03 % on 81 rows. For D2, T_ad is about 1590 K by my constant-c_p scaling, which is crude **[M: c_effect.py]**.

**Typed numbers that stand on these rows**

| Typed number | What the comment says | What the row is | How it differs |
|---|---|---|---|
| `se.flash` (growth 1740 K, 45 K; hinge −18, −12) | "8–35 s and one held flame at 1785–1795 K" | Only 21125A2 is at 1795 K, and it lived 8.7 s. K1–K3 are 30 % butane near 2650 K | The upper anchor is one flame that died early, not normal lives. Lower side intact |
| Hinge unit 4, `b1 = −4.1`; growth `se.diff` unit 0, `b1 = −4.1` | "butane in air is on the ethane law (14–20 s at 0.8 mg/s)" | Flows were 0.49, 0.49, 0.66 mg/s | At true flows the laws give 27, 26, 17 s (hinge) and 29, 28, 18 s (growth) against 20.9, 21.1, 14.7. Butane in air is 0.72–0.86 of the law |
| Hinge unit 3, `b1 = −5.9` | "3 % butane flames are only 0.2–0.3 below" | K1–K3 are not dilute | No evidence left for levelling off below 12 % |
| Growth `se.diff.skip[3] = +0.2` | third leg "3 % butane +0.2 to +1.1" | same rows | Leg is false; the number was typed as half of the other two sessions, so it does not move |
| Hinge `ft.tau` butane offset | "72, 80, 125, 158 s at 0.75–0.8 mg/s" | K3 at 125 s is a scripted stop, peak 480 °C; 80 and 158 s are at the 450 °C limit | Under 500 °C the evidence is one trip (72 s) and one no-trip in 125 s |
| Growth card, "3 % butane heats the burner four times more" | outlier of the heating law | K3 with the right fuel fraction | Measured over law is 0.98 at 30 s, 0.94 at 60 s, 0.86 at 120 s **[M: d_heating.py]**. Nothing to retype |

**Corrected inputs under P1's present weights** (one evaluation) **[M: c_effect.py]**

| Row | Extinction, table → corrected | Flow termination |
|---|---|---|
| 21106G6 | 0.040 → 0.980 | 0.001 → 0.875 |
| 21106G7 | 0.041 → 0.938 | 0.000 → 0.838 |
| 21106G9 | 0.044 → 0.100 | 0.000 → 0.015 |
| 21125K1 | 0.149 → 0.343 | 0.001 → 0.007 |
| 21125K2 | 0.047 → 0.124 | 0.000 → 0.001 |
| 21125K3 | 0.550 → 0.002 | 0.122 → 0.630 |

- Train means would go 0.05175 → 0.05944 and 0.02740 → 0.03807.
- The laws were typed to the wrong rows, so corrected inputs without a retype do harm.
- K1 against its October repeat 21291D1 (same nominal condition): 34.9 against 47.5 s, ln −0.31. That is one direct measure of session-to-session spread.

**Proposal**

- **Statement:** read flow and fuel fraction from the script parameter file (`fuelTestFlow1Slpm`, `nitrogenTestFlow1Slpm`), as the planned stop is read.
- **Input:** before ignition, private root only; not in any record today.
- **Parameters:** none new in the forward. The gas factors and bottle fraction are handbook and apparatus values. T_ad needs recomputing.
- **Risk:** errors sit in two of seven train sessions; unseen sessions are unknown.
- **Order of work:** retype the five numbers above first.

## D. Other recorded findings

### Pressure near the holding flow

**[M: d_pressure.py]** Exponents of life against pressure, 1 → 2 atm, oxygen within 0.005:

| Fuel, flow | Pair | Measured | Hinge law | Growth law |
|---|---|---|---|---|
| ethane 1.0 mg/s | 21162A3 → 21153C3; → 21153G1 (two sessions) | −0.92; −0.86 | −0.86; −0.98 | −0.24; −0.36 |
| butane 1.0 mg/s | 21291G1 → 21291C1 (one session, first hot extinction) | −0.86 | −1.09 | −0.51 |
| ethane 1.5 mg/s | 21162A2 → 21153C2 | −0.49 | −0.54 | −0.39 |
| butane 1.5 mg/s | 21291D1 → 21291A1 | −0.20 | −0.55 | −0.39 |
| ethane 2.36 mg/s | 21162A1 → 21153C1 | −0.32 | −0.57 | −0.42 |

- It is a mechanism in transferable quantities: the holding flow rises with pressure and the life is steep near it.
- Support: three pairs, three sessions, two fuels. The 0.5 atm side is confounded by oxygen.
- The hinge member already carries it through its 0.24 shift of the hinges.
- I gave the growth member the same 0.24 on the critical flux at every oxygen level, once: 0.05175 → 0.06005 **[M: d_pressure_growth.py]**. It double-counts with `se.diff.skip[0] = +0.45` (21153C3 goes to 62 s against 123).
- It would need the diffusivity term retyped, and train cannot separate the two placements.

### Flash head start

**[M: d_flash.py]**

- The fuel command leads the flame by a median 2.0 s (1.4–3.9, n = 80). The lead follows the script parameter `fuelFlowIgnitionDeltaMs`: 0 → 1.8 s, 500 → 2.3 s, 2000 → 3.8 s.
- The chamber pressure step 1.5 s after onset equals 1.4 s of fuel (quartiles 1.2–1.6, n = 71), about the same in air (1.3) and enriched oxygen (1.5). This uses the dossier's 83 L and γ = 1.4 **[D]**.
- One matched pair: 21162F2 ignited with almost no step (0.1 s) and lived 12.8 s; 21162C4 at the same condition had a normal step and lived 9.1 s. That fits a 2–4 s head start.
- 21088A1 against B1 goes the other way (longer pre-flow, 50.5 against 43.9 s), inside repeat scatter.
- Every session used nearly the same ignition, so a head start cannot be told from the floor on train.

### Cool flame

**[M: d_cool.py]**

| Test | First hot extinction | Relights | Burner at extinction | Radiometer +10 s / −2 s |
|---|---|---|---|---|
| 21291A1 (2 atm) | 41.3 s | 5 | 269 °C | 0.30 |
| 21291C1 (2 atm) | 89.2 s | 5 | 409 °C | 1.06 (already relit) |
| 21291B1, C2, H1 (2 atm) | 31.2, 26.8, 34.5 s | 0 | 189, 177, 214 °C | 0.12–0.13 |
| 21291D1, G2, J3, G1 (1 atm) | 47.5–162 s | 0 | 256–413 °C | 0.14–0.17 |

- Only these two butane flames relight; 21153G1 (ethane) flickers once before going out.
- The data-file header `Thermocouple` is 0 on 18 October and 1 on the six other sessions. The CFI-G report (p. 14 card) says the thermocouple rake was arranged differently on the final test day.
- **P1 did not learn that these flames last.** Its clocks are 34 and 47 s for A1 and 81 and 390 s for C1 **[M: e_clocks.py]**. This contradicts the dossier's statement.
- Under the table label the two cost 1.299 and 0.258. If the endpoint were the first hot extinction they would cost 0.209 and 0.333 **[M: e_cycling.py]**.
- The rule rests on one session and five 2 atm flames.

### Heating law

**[M: d_heating.py]**

| Session | Measured / law at 30 s | at 60 s | at 120 s |
|---|---|---|---|
| 03-29 | 1.06 (n = 8) | 1.01 (4) | 0.93 (1) |
| 04-16 | 0.94 (5) | 0.98 (5) | 0.90 (1) |
| 06-02 | 1.06 (14) | 1.04 (7) | 0.95 (5) |
| 06-11 | 1.05 (5) | 1.04 (4) | 0.98 (3) |
| 10-18 | 1.00 (10) | 1.00 (7) | 0.97 (4) |

- Two-point time constants, session medians: 17–42 s from (10, 20 s), 26–36 s from (30, 60 s), 18–25 s from (60, 120 s).
- The trace is faster early and flattens sooner than one exponential. That is a decaying source, not a slow second constant.
- Peak rise over the law's asymptote is about 0.90–1.03 for the flames fed 120 s or more that I checked by hand from this table (0.85 for K3 at corrected inputs).
- Trip times at the central node: 21088F1 23 s against 24.9, 21153A4 52 against 52.6, 21153C5 46 against 44.4, 21106F1 53 against 66.7.
- This is what limits A: flames within ±10 % of the threshold go either way, which `bt.sigma = 0.12` already says.

## E. Where P1 loses on train

**By session** (shared records) **[M: e_losses.py]**

| Session | Cases | Extinction mean (share) | Flow termination mean (share) |
|---|---|---|---|
| 03-29 | 39 | 0.0320 (11 %) | 0.0527 (36 %) |
| 04-16 | 27 | 0.0389 (10 %) | 0.0169 (8 %) |
| 05-05 | 12 | 0.2043 (23 %) | 0.0635 (13 %) |
| 05-10 | 18 | 0.0088 (1 %) | 0.0013 (0 %) |
| 06-02 | 42 | 0.0460 (18 %) | 0.0393 (29 %) |
| 06-11 | 36 | 0.0241 (8 %) | 0.0048 (3 %) |
| 10-18 | 36 | 0.0878 (29 %) | 0.0176 (11 %) |

**By what happened** **[M: e_categories.py]**

| Group | Runs | Extinction, shared | Flow termination, shared | Flow termination, stop known |
|---|---|---|---|---|
| 21125A2 (3 % butane, 1795 K, out at 8.7 s) | 1 | 1.71 (16 %) | 0.64 (11 %) | 0.57 (13 %) |
| Cycling flames labelled FT (21291A1, C1) | 2 | 1.56 (14 %) | 0.05 | 0.01 |
| Rows with wrong inputs | 6 | 0.87 (8 %) | 0.12 | 0.07 |
| Scripted stops labelled FT | 7 | 0.76 (7 %) | 3.09 (54 %) | 1.14 (25 %) |
| Limit trips | 6 | 0.06 | 1.16 (20 %) | 1.28 (28 %) |
| Self-extinction in enriched oxygen, 27–304 s | 30 | 5.39 (50 %) | 0.50 (9 %) | 1.26 (28 %) |
| Self-extinction in air, 5–20 s | 18 | 0.51 (5 %) | 0.05 | 0.04 |

**The dominant cases**

- **21125A2.** The hinge law gives 1281 s (it sees 0.5 mg/s in 39.5 % oxygen, below the holding flow) and a burner trip at 17 s. The burner peaked at 61 °C because the total flow is 16 times the fuel flow. The growth member gives 60 s. None of A–D addresses it. The dossier's critical-temperature form does (7–11 s **[D]**), on one flame.
- **21291A1, C1.** Label convention, see D.
- **21125K3 (0.550).** The growth member reads Φ = 2.24 from the wrong fuel fraction and predicts 25 s; with 0.30 it holds (Φ = 0.89). Addressed by C.
- **The 30 enriched self-extinctions are half the loss and are not biased.** ln(observed / law) is within ±0.27 for the hinge member except 21106G6/G7 and within ±0.4 for the growth member except 21291G1 (−1.24) and 21153A3 (+0.67) **[M: e_clocks.py]**. Each costs 0.06–0.10 per case, which is the price of the clock width. A–D do not touch it.
- **Limit trips.** A changes them; net effect nil.

**The planned-stop offset** **[M: e_script_offset.py]**

Of 80 train tests, 11 ran to the scripted end.

| Test | Stage sum (s) | Flow command off, from my flame onset | Table burn minus sum |
|---|---|---|---|
| 21088D1, H1 (FT) | 60 | 61.9, 61.8 | +1, +2 |
| 21088H2, K1 (FT) | 125, 95 | 126.8, 96.8 | +2, +3 |
| 21153C7 (FT) | 60 | 60.2 | +1 |
| 21125K3, 21153A5, C4, 21291C1 (FT) | 125, 275, 275, 245 | +0.2, +0.1, +0.2, +0.5 past the sum | 0, −1, +1, +1 |
| 21106C1, D1 (SE) | 65, 125 | +0.1, +0.2 past the sum | – |

- The stop comes 0.1–0.5 s after the stage sum from April to October and 1.8 s after in March. On the table's clock the nine FT rows sit at a median +1 s.
- P1's `script.onset_s = 0.4` puts the stop at 59.6 s for the three 55 s scripts. The 60 s horizon then scores 21088D1, H1 and 21153C7 as stopped while the table says burning.
- With the stop at sum + 1.0 s (the measured median, scored once): 0.05364 / 0.02149 → 0.05336 / 0.01615.
- C-47 reports stops about 2 s after the sum on Flame Design **[D]**, the same sign.
- I could not reproduce P1's 29-test median for normal flames alone; I assume it pools inverse tests.
- **Parameter:** `script.onset_s` between −2 and 0 for normal flames (apparatus timing).
- **Risk:** the gain depends on a script end sitting within a second or two of a horizon.

## Files opened

- **Repository documents:** `AGENTS.md`; `tools/wsp/analysis/README.md`, `lib.py`; `docs/wsp/research/phys_acme_normal.md`, `astra_r2_acme_loss_analysis.md`; `docs/wsp/feedback/verify_phys_acme_normal.md`; cards `acme_committee.md`, `acme_growth.md`, `acme_clock.md`; `docs/wsp/dossier/05_claims.md` (burner and across-family rows), `08_in_progress.md` (8.4–8.7), `10_extension_map.md` (normal flames).
- **Models and code:** `edits.wsp` of wsp5 p1-fable, wsp2 d3-fable and baseline-clock, wsp3 d1-fable, d2-fable and baseline-theory; `flame.rs`, `flame3.rs`, `committee.rs` (grep), `ignis-lab/src/eval/scoring.rs` and `scores.rs` (parts); `tools/wsp/ext/acme_script.py`.
- **Feedback:** `docs/wsp/feedback/astra-r2-20261115/results/acme_normal.json`.
- **Records, through `lib`:** `experiments.jsonl`, `endpoints.jsonl`, `post.jsonl`, `extra_control_history.jsonl` of `acme_normal` (81 train ids) and the first three of `acme_inverse` (73 train ids).
- **Private root:** `runs/dev/data_ext/work/records/acme_normal/experiments.jsonl`, train lines only.
- **Raw files, paths built from train names only:** `_data.csv`, `_mfcSetpoints.csv`, `_TP.txt` and the "as run" file of 80 normal and 73 inverse train tests.
- **Earlier scratch:** `runs/dev/research7/verify_normal/vlib.py`; `runs/dev/research5/acme_raw/scripts/rawlib.py` (head), `summary.txt` (head), `asrun_keys_count.json`.
- **Page cards, by grep:** `open_pages.jsonl` snippets of the PSI-23 s-Flame procedure (pp. 12–36), the CFI-G final report (pp. 7, 10, 11, 14, 27, apparatus text), the ACME engineering report (pp. 17, 22, 102), and the first record of the file; two `documents.jsonl` records.

All my scripts and outputs are in `runs\dev\gen6\acme_normal\` (git-ignored). No tracked file was modified, staged or committed; no sub-agent was launched; no validation comparison was run.

## Protected data

**No validation or test outcome was opened or printed, and no row of a non-train run was printed.** Four things to know:

1. `lib.experiments` and `lib.jsonl_for` parse every line of the shared JSONL files before filtering by run id. Non-train rows, including endpoints, therefore passed through memory inside the sanctioned accessor. Nothing of them was kept, inspected or printed. For the private root I filtered by id with a regular expression before parsing.
2. The assigned reading contains aggregate validation scores (cards, 8.6, 8.7, C-43 to C-48, the Astra report) and, in section 5 of the physics dossier, the conditions and dates of the two validation sessions. I used none of it in a measurement.
3. `ls runs/dev` and `ls runs/dev/research6/acme_normal` printed file names that include validation comparison files and `validation_conditions_only.json`. None was opened.
4. The statement that every validation run has a script file is arithmetic on the card's count, not a file check.

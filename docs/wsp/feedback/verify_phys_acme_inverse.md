# Independent check of the inverse-flame physics dossier on the train tests

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. An independent re-derivation with its own code; train tests only, no validation test was read.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# Independent check of `phys_acme_inverse.md` on the 73 train inverse tests

The dossier's qualitative picture holds, but one train label is wrong (21270H3 went out by itself at 93.3 s), one flame died with no burst, and the closed form misses the pressure dependence between 1.1 and 2.2 bar entirely. Everything below is from my own code on the raw 125 Hz files; nothing was fitted.

**Definitions used throughout**
- **Onset of flame (t = 0):** first sample where the sum of the three LoGain radiometers exceeds its pre-test level by 0.15 V for 12 samples. It agrees with the reference onset within 0.03 s on all 73 tests.
- **Flame end:** first time the median of the three photomultipliers stays below 2.2 V (half of saturation) for 0.3 s. The "all dark below 0.1 V" end is a median 0.5 s later, and up to 2.9 s later for flashes.
- **Burst:** on each LoGain radiometer (0.1 s boxcar, baseline removed), a local maximum after 3 s that is at least 10 % above the lowest point in the 1 s before and in the 1 s after. It counts only if at least two of the three radiometers show it within 0.25 s.
- **Oscillation onset:** time of the first burst peak. I also ran 5 % and 3 % thresholds.

## I1. Bursts before death — WEAKENED (ethane, air and not-an-artefact parts CONFIRMED)

- **Propane and butane flames that go out after 5 s:** 33 of 34 show at least one burst. The exception is 21222C1 (50 % O2, 3.3 bar), which fades smoothly from 2 s and is out at 5.9 s with no burst even at the 3 % threshold.
- **Last burst peak to flame end:** 0.25–0.78 s, median 0.53 s (n = 34). The dossier's "0.3–0.5 s" is about right.
- **Amplitude:** peak over running level 1.06–2.4.
- **Period:** not 1.2–1.8 s in general; it scales with O2 flow.

| O2 flow (mg/s) | Median period (s) | Flames |
|---|---|---|
| 1.34 | 0.51 | 1 |
| 1.8 | 0.82–1.11 | 5 |
| 3.6 | 1.34–1.46 | 3 |
| 4.0–5.4 | 1.16–1.78 (2.84 at 3.3 bar) | 9 |
| 9 | 1.55–2.31 | 4 |

- **Ethane:** 18 of 18 show no oscillation. The single event in 21196F1 at 14.0 s coincides with a dip in the nitrogen readback (1.40 to 1.28 V), so it is a flow glitch.
- **Air-injected survivors:** no burst at the 10 % level in 21228F1, 21228L2, 21256F1. 21228F1 does show a damped ripple at 4–9 s (15 % falling to 3 %, period about 1.4 s).
- **Not artefacts:** every burst appears in all three radiometers. In all but three events at least two photomultipliers dip at the same moment. The burner heating rate is modulated in phase at 5.4 mg/s and below (±1 K/s averaged over 58 cycles in 21270H3) and not at 9 mg/s. Periods differ between tests and stretch within a test, which rules out a camera or lamp cycle.

Per-test table for propane and butane (f = fibre in the flame, from the as-run file; * = tabulated 50 % but ran at 85 %). Ethane: 18 tests, all flow-terminated, no bursts.

| Test | Cell | X_O2 | O2 mg/s | Z_st | Outcome, end (s) | First burst (s) | Bursts | Last burst to end (s) |
|---|---|---|---|---|---|---|---|---|
| 21222A3 | propane 8.0 %, 3.3 bar | 0.85 | 9.07 | 0.665 | SE 16.8 | 11.7 | 3 | 0.51 |
| 21222A1 | | 0.85 | 5.44 | 0.665 | SE 10.0 | 9.3 | 1 | 0.69 |
| 21222B1 f | | 0.85 | 5.44 | 0.665 | SE 16.1 | 12.5 | 2 | 0.78 |
| 21222A2 | | 0.85 | 3.63 | 0.665 | SE 15.8 | 12.4 | 3 | 0.48 |
| 21222A4 | | 0.85 | 1.81 | 0.665 | SE 12.0 | 9.5 | 3 | 0.33 |
| 21222A5 | | 0.85 | 1.36 | 0.665 | SE 2.5 | none | 0 | – |
| 21222C1 | | 0.50 | 5.44 | 0.551 | SE 5.9 | none | 0 | – |
| 21222C2 | | 0.50 | 3.63 | 0.551 | SE 7.4 | 6.9 | 1 | 0.47 |
| 21222F1 | | 0.21 | 5.44 | 0.349 | flash 1.8 | none | 0 | – |
| 21228A2 | propane 8.9 %, 2.2 bar | 0.85 | 9.07 | 0.641 | SE 10.1 | 9.3 | 1 | 0.78 |
| 21228A1 | | 0.85 | 5.44 | 0.641 | SE 7.7 | 7.2 | 1 | 0.40 |
| 21228B1 f | | 0.85 | 5.44 | 0.641 | SE 15.8 | 6.9 | 6 | 0.38 |
| 21228A3 | | 0.85 | 3.63 | 0.641 | SE 11.7 | 9.8 | 2 | 0.59 |
| 21228A4 | | 0.85 | 1.81 | 0.641 | SE 11.3 | 10.9 | 1 | 0.42 |
| 21228C4 * | | 0.85 | 1.81 | 0.644 | SE 8.5 | 8.2 | 1 | 0.27 |
| 21228C2 | | 0.50 | 9.07 | 0.524 | FT 65.6 | 7.1 | 19 | – |
| 21228C1 | | 0.50 | 5.44 | 0.524 | SE 7.2 | 6.7 | 1 | 0.56 |
| 21228C3 | | 0.50 | 3.63 | 0.527 | SE 6.4 | 5.9 | 1 | 0.45 |
| 21228F1 | | 0.21 | 5.44 | 0.327 | FT 65.2 | none | 0 | – |
| 21228H1 | propane 8.7 %, 1.1 bar | 0.85 | 5.44 | 0.646 | SE 7.3 | 6.6 | 1 | 0.65 |
| 21228H2 | | 0.85 | 1.81 | 0.646 | SE 15.0 | 12.6 | 3 | 0.49 |
| 21228J1 f | | 0.85 | 1.81 | 0.647 | SE 17.1 | 14.1 | 4 | 0.48 |
| 21228K2 * | | 0.85 | 1.81 | 0.647 | SE 12.6 | 12.2 | 1 | 0.41 |
| 21228K1 | | 0.50 | 5.44 | 0.530 | SE 9.2 | 7.1 | 2 | 0.57 |
| 21228L2 | | 0.21 | 5.44 | 0.331 | FT 14.8 (script) | none | 0 | – |
| 21228L1 | | 0.21 | 3.63 | 0.330 | flash 1.9 | none | 0 | – |
| 21242A1, P1 | propane 43 %, 0.55 bar | 0.85, 0.30 | 5.44 | 0.31, 0.14 | FT 12.4, 20.0 | none | 0 | – |
| 21256H5 | butane 7.0 %, 2.2 bar | 0.85 | 8.95 | 0.642 | SE 7.1 | 6.8 | 1 | 0.30 |
| 21256H1 | | 0.85 | 5.37 | 0.642 | SE 7.1 | 6.6 | 1 | 0.50 |
| 21256J1 f | | 0.85 | 5.37 | 0.642 | SE 6.8 | 6.3 | 1 | 0.50 |
| 21256H2 | | 0.85 | 3.58 | 0.642 | SE 6.6 | 6.1 | 1 | 0.48 |
| 21256H3 | | 0.85 | 1.79 | 0.642 | SE 4.3 | none | 0 | – |
| 21256H4 | | 0.85 | 1.34 | 0.642 | SE 1.6 | none | 0 | – |
| 21256K1 | | 0.50 | 5.37 | 0.525 | SE 3.5 | none | 0 | – |
| 21256M1 | | 0.21 | 5.37 | 0.326 | flash 1.0 | none | 0 | – |
| 21256A5 | butane 8.8 %, 2.2 bar | 0.85 | 8.95 | 0.592 | SE 9.1 | 8.5 | 1 | 0.59 |
| 21256A1 | | 0.85 | 5.37 | 0.592 | SE 9.2 | 8.6 | 1 | 0.61 |
| 21256B1 f | | 0.85 | 5.37 | 0.595 | SE 12.8 | 8.5 | 3 | 0.68 |
| 21256A2 | | 0.85 | 3.58 | 0.592 | SE 10.5 | 8.6 | 2 | 0.54 |
| 21256A3 | | 0.85 | 1.79 | 0.592 | SE 10.5 | 8.9 | 2 | 0.55 |
| 21256C2 * | | 0.85 | 1.79 | 0.595 | SE 9.0 | 7.4 | 2 | 0.61 |
| 21256A4 | | 0.85 | 1.34 | 0.592 | SE 4.8 | 3.0 | 4 | 0.25 |
| 21256C1 | | 0.50 | 5.37 | 0.472 | SE 8.3 | 7.7 | 1 | 0.59 |
| 21256F1 | | 0.21 | 5.37 | 0.281 | FT 65.3 | none | 0 | – |
| 21270H4 | butane 14 %, 1.1 bar | 0.85 | 8.95 | 0.492 | FT 82.5 | 11.3 | 24, last at 59.6 | – |
| 21270H1 | | 0.85 | 5.37 | 0.489 | SE 48.8 | 18.8 | 24 | 0.70 |
| 21270H5 | | 0.85 | 4.69 | 0.496 | SE 46.1 | 21.1 | 21 | 0.62 |
| 21270H3 | | 0.85 | 4.03 | 0.491 | **SE 93.3** (table: FT 43) | 27.7 | 58 | 0.59 |
| 21270H2 | | 0.85 | 2.68 | 0.491 | FT 55.5, burner 452 °C | none | 0 | – |
| 21270A2 | butane 17.5 %, 1.1 bar | 0.85 | 8.95 | 0.443 | FT 95.5 | 17.3 | 8, last at 31.0 | – |
| 21270A1 | | 0.85 | 5.37 | 0.442 | FT 65.5 | 38.9 | 21, still growing | – |
| 21270A4 | | 0.85 | 4.03 | 0.447 | FT 95.5 | none | 0 | – |
| 21270A3 | | 0.85 | 2.68 | 0.447 | FT 29.4, burner 457 °C | none | 0 | – |
| 21270C1 | | 0.50 | 5.37 | 0.334 | FT 69.5 | 18.7 | 5, last at 25.1 | – |

## I2. Outcomes after onset — WEAKENED (the 14 % butane row is wrong)

| Class | My count | Dossier row |
|---|---|---|
| Z_st ≥ 0.59 (85 % into fuel ≤ 8.9 %) | 29 of 29 died. 26 show bursts: 1–4 without fibre, 1–6 with. The 3 without bursts are at O2 ≤ 1.8 mg/s and died before 5 s. | confirmed; "1–3 cycles" should read 1–4, up to 6 with fibre |
| Z_st 0.47–0.55, true 50 % O2, O2 ≤ 5.4 mg/s, lean | 7 of 7 died; bursts 0, 0, 1, 1, 1, 1, 2 | confirmed as an outcome; two died with no cycle |
| 14 % butane, 85 % O2 | O2 4.0–5.4: 3 of 3 died (21, 24, 58 bursts). O2 8.95: held, bursts stopped at 59.6 s. O2 2.68: no oscillation, burner-limit stop. | **refuted for H3 and H4**: H3 died 2.1 s before the flow stop; H4 was not cycling at the stop |
| Z_st 0.47–0.53, O2 about 9 mg/s | 2 of 2 held (21228C2, 21270H4) | confirmed |
| Z_st 0.33–0.45, rich butane | A2 and C1 oscillate briefly and stop; A1 weak and growing at the stop; A4 and A3 none | confirmed |
| Z_st ≤ 0.33, air | 3 of 3 held, no bursts | confirmed |

## I3. Closed-form onset law — WEAKENED

Constants used:
- σ = 3.725 / 3.628 / 3.579 g O2 per g fuel (ethane / propane / butane).
- D_F = 0.160 / 0.125 / 0.106 cm²/s at 1 atm, divided by p in atm (bar / 1.01325), no temperature scaling.
- ρ_amb = p·W_mix / (R·300 K), with W_mix the fuel–nitrogen mixture molar mass.
- ρD_F = 0.0334 / Le_F g m⁻¹ s⁻¹, Le_F = 1.39 / 1.78 / 2.10. The blueprint's ×1.05 changes predictions by under 5 % for lean cells.
- r_ss = m_tot / (4π ρD_F ln(1/Z_st)), m_tot = m_O2 / Y_O,in.
- Set A: K = 2, critical flux 1.2. Set B: K = 0.8(1−Z_st)^−0.8, critical flux 1.0.
- Readback-corrected conditions for the three no-nitrogen tests and 21196M2_1.

| Cell | Predicted A (s) | Predicted B (s) | Observed onset (s) | A / observed | B / observed |
|---|---|---|---|---|---|
| Propane 8.0 %, 3.3 bar, 85 %, 3.6–9.1 mg/s | 10.8–12.8 | 14.6–18.0 | 9.3–12.5 | 0.93–1.25 | 1.26–1.72 |
| Propane 8.9 %, 2.2 bar, 85 %, 3.6–9.1 | 9.2–11.2 | 11.2–14.3 | 6.9–9.8 | 0.99–1.44 | 1.21–1.79 |
| Propane 8.9 %, 2.2 bar, 50 %, 3.6–9.1 | 8.9–10.2 | 6.9–8.2 | 5.9–7.1 | 1.26–1.71 | 0.97–1.38 |
| Propane 8.7 %, 1.1 bar, 85 %, 5.44 | 4.7 | 6.0 | 6.6 | 0.72 | 0.91 |
| Propane 8.7 %, 1.1 bar, 50 %, 5.44 | 4.5 | 3.6 | 7.1 | 0.64 | 0.51 |
| Propane 8.0 %, 3.3 bar, 50 %, 3.63 | 12.0 | 10.4 | 6.9 | 1.74 | 1.51 |
| Butane 7.0 %, 2.2 bar, 85 %, 3.6–9 | 7.8–9.0 | 9.5–11.3 | 6.1–6.8 | 1.15–1.48 | 1.39–1.85 |
| Butane 8.8 %, 2.2 bar, 85 %, 3.6–9 | 12.8–15.7 | 12.7–16.4 | 8.5–8.6 | 1.50–1.84 | 1.49–1.92 |
| Butane 8.8 %, 2.2 bar, 50 %, 5.37 | 13.3 | 8.8 | 7.7 | 1.73 | 1.14 |
| Butane 14 %: 8.95 / 5.37 / 4.69 / 4.03 / 2.68 | 17.7 / 21.7 / 21.5 / 24.9 / 36.3 | 12.7 / 16.1 / 16.5 / 19.4 / 31.8 | 11.3 / 18.8 / 21.1 / 27.7 / none in 55 s | 0.90–1.57 | 0.70–1.12 |
| Butane 17.5 %: 8.95 / 5.37 / 4.03 | 30.3 / 39.3 / 45.9 | 19.1 / 26.4 / 33.3 | 17.3 / 38.9 / none in 95 s | 1.74 / 1.01 | 1.10 / 0.68 |
| Lean 85 %, O2 1.8 mg/s | 7.6–23.3 | 11.0–27.1 | 7.4–14.1 | 0.54–3.0 | 0.78–3.5 |
| Butane 8.8 %, O2 1.34 mg/s | 32 | 41 | 3.0 | 10.6 | 13.8 |
| Ethane, all 18 | 9.6–79 | 3.5–137 | none in 25–137 s | – | – |
| Air into lean propane or butane, holds | 4.2–12.3 | 1.9–4.9 | none in 15–65 s | – | – |

- **Spread, lean cells with O2 ≥ 3.6 mg/s (n = 23):** A 0.64–1.84, median 1.31, sd of ln 0.27. B 0.51–1.92, median 1.46, sd of ln 0.29.
- **Pressure, wrong between 1.1 and 2.2 bar:** median A ratio is 0.68 at 1.1 bar (n = 2), 1.44 at 2.2 bar (n = 16), 1.03 at 3.3 bar (n = 5). The form's factor of 2 from doubling pressure is absent in the data.
- **Ambient fuel fraction:** too steep for A in the one clean pair (butane 7.0 to 8.8 %: A predicts ×1.69, B ×1.39, observed ×1.26–1.40). B's K(Z_st) gets it right.
- **Flow in a lean ambient:** right, in that there is no trend.
- **Flow in a rich ambient:** right direction, too weak. Observed onset rises ×2.45 from 8.95 to 4.03 mg/s; A gives ×1.41, B ×1.53. Both predict an onset at 2.68 mg/s (14 % butane) and at 4.03 mg/s (17.5 %) that never came.
- **Low flow, ethane, air flames:** the form fails, as the dossier says.
- **K(Z_st):** B is better than A for the 50 % flames (median 1.12 against 1.61).
- **Third variant, only because the dossier states it:** B with the blueprint's own p^0.15 on the critical flux gives medians 0.90 / 1.53 / 1.26 at 1.1 / 2.2 / 3.3 bar. It helps at 1.1 bar only.
- **Reproduction of the dossier's column:** my propane predictions match within 4 %. My butane predictions are 7–24 % longer than its table (for example 17.7 / 21.7 / 24.9 against 14 / 18 / 21). I could not find the cause without reading its script.

## I4. Z_st separates best — CONFIRMED

Of 65 flames alive at 5 s, the count inside the range where both outcomes occur:

| Variable | Table labels and table values (SE / FT) | Measured outcomes, readback-corrected (SE / FT) |
|---|---|---|
| Z_st | 7 / 9 | 6 / 8 |
| Ambient fuel mole fraction | 22 / 7 | 23 / 6 |
| Total flow | 28 / 17 | 26 / 17 |
| O2 flow | 25 / 32 | 26 / 31 |
| Injected O2 fraction | 33 / 16 | 34 / 15 |
| T_ad | 33 / 14 | 34 / 13 |

- The dossier's 5 / 9 for Z_st uses corrected Z_st with table labels.
- Inside the overlap, 5 ethane flames held. Propane and butane: 6 died (O2 3.6–5.4 mg/s), 3 held (9.07, 8.95, and 2.68 on the burner limit). That is not monotonic in O2 flow, and not the dossier's "5 died, 4 held".

## I5. Flash at ignition — T_ad part CONFIRMED, ordering WEAKENED

| Test | Fuel, X_F, p | Y_F | O2 mg/s | Z_st | T_ad (K) | Result |
|---|---|---|---|---|---|---|
| 21222F1 | propane 8.0 %, 3.3 bar | 0.120 | 5.44 | 0.349 | 1835 | flash, out at 1.8 s |
| 21256M1 | butane 7.0 %, 2.2 bar | 0.135 | 5.37 | 0.326 | 1879 | flash, out at 1.0 s |
| 21228L1 | propane 8.7 %, 1.1 bar | 0.130 | 3.63 | 0.330 | 1868 | flash, out at 1.9 s |
| 21228L2 | propane 8.7 %, 1.1 bar | 0.130 | 5.44 | 0.331 | 1867 | held (14.8 s script) |
| 21228F1 | propane 8.8 %, 2.2 bar | 0.132 | 5.44 | 0.327 | 1873 | held 65 s |
| 21256F1 | butane 8.8 %, 2.2 bar | 0.166 | 5.37 | 0.281 | 1956 | held 65 s |

- T_ad ranges overlap (flash 1835–1879, hold 1867–1956), as claimed.
- The ordering by ambient fuel holds only in mole fraction. In mass fraction the butane flash (0.135) sits above two propane holds (0.130, 0.132). Z_st does not order them either.
- It is six flames, with fuel and pressure not controlled. The nine air-injected ethane flames all held.

## I6. Flow readbacks — CONFIRMED

| Test | Nitrogen readback (slpm) | Expected (slpm) | X_O2 from readback | Table |
|---|---|---|---|---|
| 21228C4 | −0.002 | 0.069 | 0.85 | 0.50 |
| 21228K2 | 0.001 | 0.069 | 0.84 | 0.50 |
| 21256C2 | −0.004 | 0.067 | 0.85 | 0.50 (table comment already says "No nitrogen flowed") |

- 21196M2_1: O2 readback 1.520 V, identical to 21196M2 (1.519 V), so 2.79 mg/s in table units, not 5.59.
- O2 readback over table is 1.10 on every other test (1.08–1.13), which is a unit convention.

## I7. Session 2021-09-27 — CONFIRMED, plus a wrong outcome

| Test | Table | Flow stop (s) | Photomultiplier end (s) |
|---|---|---|---|
| A1 | FT 60 | 65.5 | 65.9 |
| A2 | FT 90 | 95.5 | 96.3 |
| A3 | FT 35 | 29.4 | 29.7 |
| A4 | FT 90 | 95.5 | 95.8 (not in the dossier's list) |
| C1 | FT 70 | 69.5 | 69.9 |
| H1 | SE 51 | 71.5 | 48.8 |
| H2 | FT 43 | 55.5 | 55.5 |
| H3 | FT 43 | 95.5 | **93.3, self-extinguished** |
| H4 | FT 83 | 82.5 | 83.4 |
| H5 | SE 46 | 77.5 | 46.1 |

In the other six sessions, measured end minus table burn time has medians of −0.6 to +0.1 s and never exceeds ±1.4 s.

## I8. Burner heating fraction — WEAKENED

Fraction = 0.31 J/K × dT/dt ÷ (O2 flow × 12.78 J/mg), lean propane and butane at 85 % O2 unless noted. The heating rate falls steeply in the first 2 s, so the answer depends on the window.

| Total flow (mg/s) | 0.5–1 s | 1–2 s | 2–3 s | Dossier |
|---|---|---|---|---|
| 2.1 (n = 9) | 31–37 % (23–27 K/s) | 16–21 % (12–15.5 K/s) | 12.5–18 % | 24 % |
| 4.2 (n = 4) | 14–17 % | 6–7 % | 3.4–4.7 % | 9 % |
| 6.2–6.3 (n = 9) | 7–11 % | 2.4–4.1 % | 1.3–2.2 % | 4.5 % |
| 10.1–10.5 (n = 7) | 0.4–3.6 % | −0.2–1.5 % | −0.1–1.3 % | 1.5 % |
| 17–23 (n = 7) | 0.1–0.3 % | 0–0.6 % | 0–0.8 % | 0.5 % |

- The dossier's figures sit at roughly the 1–1.5 s window; the fall with flow is confirmed.
- It is not a function of total flow alone. At 4.3 mg/s ethane gives 16–20 % (1–2 s) against 6–7 %. At 6.2 mg/s rich butane gives 3.5–5.7 % and 43 % propane 22 %. "Under 1 % above 17 mg/s" fails for ethane (1–4 %) and for 21242P1 (3.8 %).

## Numbers I would hand the author

**Onset time by cell** (first burst peak, 10 % threshold, no fibre unless marked)

| Cell | Onset (s) | Rests on |
|---|---|---|
| Propane 8.0 %, 3.3 bar, 85 %, 3.6–9 mg/s | 9.3–12.5 | 21222A1, A2, A3, B1(f) |
| Propane 8.9 %, 2.2 bar, 85 %, 3.6–9 | 6.9–9.8 | 21228A1, A2, A3, B1(f) |
| Propane 8.9 %, 2.2 bar, 50 %, 3.6–9 | 5.9–7.1 | 21228C1, C2, C3 |
| Propane 8.7 %, 1.1 bar, 5.44 | 6.6 (85 %), 7.1 (50 %) | 21228H1, K1 |
| Butane 7.0 %, 2.2 bar, 85 %, 3.6–9 | 6.1–6.8 | 21256H1, H2, H5, J1(f) |
| Butane 8.8 %, 2.2 bar, 85 %, 3.6–9 | 8.5–8.6 (50 %: 7.7) | 21256A1, A2, A5, B1(f), C1 |
| Lean, 85 %, 1.8 mg/s | 12.2–14.1 at 1.1 bar; 8.2–10.9 at 2.2 bar propane; 7.4–8.9 at 2.2 bar butane 8.8 %; 9.5 at 3.3 bar | 21228H2, J1(f), K2, A4, C4; 21256A3, C2; 21222A4 |
| Butane 14 %, 1.1 bar: 8.95 / 5.37 / 4.69 / 4.03 / 2.68 | 11.3 / 18.8 / 21.1 / 27.7 / none | 21270H4, H1, H5, H3, H2 |
| Butane 17.5 %, 1.1 bar: 8.95 / 5.37 / 4.03 | 17.3 / 38.9 / none | 21270A2, A1, A4 |

For the two rich cells the onset is threshold-defined, because the amplitude grows from about 3 %. At the 3 % threshold the 14 % butane onsets are 10.1 / 16.7 / 18.0 / 23.8 s.

**Scaling, as measured ratios**
- **Ambient fuel mass fraction, weaker than the square:** butane 7.0 to 8.8 % at 2.2 bar, same session and flows, Y_F ×1.237 gives onset ×1.26, ×1.31, ×1.40 at 8.95, 5.37, 3.58 mg/s. The square would give ×1.53; the measured exponent is 1.1–1.6. This is the only clean pair.
- **Rich ambient, steeper and flow-dependent:** butane 14 to 17.5 % gives ×1.53 at 8.95 mg/s and ×2.07 at 5.37 mg/s.
- **Pressure 1.1 to 2.2 bar:** ×1.10 (85 %, 21228H1 against A1) and ×0.94 (50 %, K1 against C1) at 5.44 mg/s. At 1.8 mg/s the trend reverses (12–14 s falling to 8–11 s).
- **Pressure 2.2 to 3.3 bar:** ×1.26–1.29 at matched flow, but with fuel 8.9 to 8.0 % and a different session.
- **Flow, lean:** within ±20 % for propane and ±5 % for butane, no trend.
- **Flow, rich:** 14 % butane onset × O2 flow = 101, 101, 99, 112 mg at 8.95, 5.37, 4.69, 4.03 mg/s, so onset goes as 1 / O2 flow there.

**Time from onset to death by class**

| Class | Count | Onset to death |
|---|---|---|
| Z_st ≥ 0.59, no fibre, with bursts | 21 | 0.3–5.1 s, median 0.6 s. 12 die on the first burst, 4 on the second, 4 on the third, 1 on the fourth: 36 bursts for 21 deaths, about 0.58 per burst |
| Same, fibre in the flame | 5 | 0.5–8.8 s; 1, 2, 3, 4, 6 bursts, about 0.31 per burst |
| True 50 % O2, lean, O2 ≤ 5.4 mg/s | 7 | 0.4–2.1 s after the first burst; two died with no burst at 3.5 and 5.9 s |
| 14 % butane, 85 %, O2 4.0–5.4 mg/s | 3 | 30.0, 25.0, 65.6 s (24, 21, 58 bursts) |

## What the dossier did not say

- **21270H3 is a self-extinction at 93.3 s.** All three photomultipliers go dark and the burner cools at −2.4 K/s while the O2 readback is still at set-point for another 2 s.
- **Two kinds of oscillation, and neither is a per-cycle lottery.**
  - In 14 % butane at 4.0–5.4 mg/s the burst amplitude grows steadily from 1.05 and the flame dies when it reaches ×2.0–2.2 (2.08, 2.24, 2.01).
  - At 9 mg/s the interval between bursts stretches (21228C2: 1.7 to 9.7 s) and the bursts stop altogether (21270H4 at 60 s, 21270A2 at 31 s, 21270C1 at 25 s). Those flames were recovering.
- **21270A1 was still growing** (1.21) at its 65.5 s stop, so its "held" is censored.
- **21270H2 and 21270A3 ended on the 450 °C burner limit**, so they are not evidence of indefinite holding.
- **A second death route:** diluted lean flames can fade without any burst (21222C1 at 5.9 s, 21256K1 at 3.5 s).
- **The fibre is knowable before ignition.** The as-run file has `fiberTestPosition` 17000 against 0. Train fibre tests: 21209B1, 21209B2, 21222B1, 21228B1, 21228J1, 21256B1, 21256J1. Against the matching no-fibre test the fibre adds +8.1, +6.1, +3.5, +2.1 and −0.3 s, not "5–8 s", and does not move the onset consistently.
- **Every diluted test reads 0.03–0.045 slpm less nitrogen than commanded.** If the readback is right, "50 %" is 0.52–0.56, "30 %" 0.31–0.33, "21 %" 0.215–0.229.
- **Flashes are out at 1.0–1.9 s.** The photomultipliers keep a tail to 3.5–4 s.
- **The as-run file named for 21196M2_1 carries the script name 21196M3 inside.**

## Tests opened, files written, statement

- **Data files:** all 73 train inverse tests, selected by exact name from `lib.partition_ids("acme_inverse", "train")`.
- **As-run files:** the same 73 names. The one named 21196M2_1 turned out to carry the inner name 21196M3; I did not use or print anything from it beyond that name.
- **Other raw files:** Comments of 21228A1 and 21228B1, TP of 21228B1.
- **Reference material:** the dossier, `rawlib.py`, `acme_raw_signals_train.md`, and the onset column of `tests_burner.csv`. I printed that file's header and first two rows, which are train normal-flame tests.
- **Not opened:** nothing outside the train list. No validation conditions, no test partition, no `holdout`, `data_ext`, `dev_eval.py`, no scripts or CSVs under `research6/acme_inverse/`, no literature. No fitting. No tracked file modified.

Files are in `D:\dev\SpaceApps-Ardealu\runs\dev\research7\verify_inverse\`:
- **Scripts:** `common.py`, `measure.py`, `crosscheck.py`, `flows.py`, `master.py`, `endhalf.py`, `fluxform.py`, `claims.py`, `summary.py`, `show.py`
- **Tables:** `master.csv` (one row per test, the full I1 table), `bursts_check.csv`, `burst_crosscheck.csv`, `flame_end_half.csv`, `flow_readbacks.csv`, `fluxform_check.csv`, `burner_heating.csv`, `asrun_train.csv`, `onset_ref.json`
- **Cache:** `cache\` holds 73 decoded train traces (65 MB) and can be deleted.

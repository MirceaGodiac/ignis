# Independent check of the n-heptane physics dossier on the train runs

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. An independent re-derivation with its own code; train runs only, no validation run was read.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# Independent check of `phys_flex_heptane.md` (FLEX n-heptane, TRAIN only)

Most of the dossier holds on train. Four things to fix before typing numbers: the H19H101 plateau, the post-cool-flame decay law, the 3.4 mm early deficit, and one helium statement about the clock. I launched no helper and did no fitting; every rate is a finite difference of box medians of D² (±0.1 to 0.25 s boxes).

## Flame-end re-read (all 49 runs, own definition)

My end is the last UV row that is neither zero nor a tracker jump above 1.25× the preceding 1 s median, or the last non-zero colour row. 47 of 49 agree with `flame_end_train.csv` within 0.1 s.

- **C01H302:** csv 8.27 s, mine 7.80 s. The last 0.47 s of the UV trace are artefact rows at 47 mm, and the droplet rate drops from 0.48 to 0.34 mm²/s at about 7.8 s.
- **H01H102:** csv 7.95 s, mine 7.53 s. The flame drifted out of view, so neither is a true flame end.

I used my own reading as t_L everywhere.

## H1 Cool-flame rate — CONFIRMED

Stage runs from t_L + 1 s to 1 s before the roll-off (first time the 1 s rate stays below 0.75·K_c for 2 s). K_c is the finite difference between the stage ends.

| Run | D0 mm | O2 / CO2 / p atm | Stage s | K_c mm²/s | First / second half |
|---|---|---|---|---|---|
| N03H101 | 2.55 | 0.15 / 0 / 1.0 | 5.5 | 0.321 | 0.325 / 0.317 |
| N02H101 | 3.34 | 0.16 / 0 / 1.0 | 16.9 | 0.338 | 0.329 / 0.348 |
| N02H203 | 2.52 | 0.16 / 0 / 1.0 | 4.9 | 0.333 | 0.349 / 0.318 |
| C04H101 | 3.76 | 0.17 / 0.20 / 1.0 | 24.4 | 0.349 | 0.325 / 0.373 |
| C04H201 | 2.90 | 0.17 / 0.20 / 1.0 | 11.7 | 0.353 | 0.356 / 0.350 |
| 301F001 | 3.43 | 0.18 / 0 / 1.0 | 18.9 (5 s gap) | 0.356 | — |
| C03H501 | 3.74 | 0.18 / 0.15 / 1.0 | 29.2 (no end in record) | 0.347 | 0.341 / 0.354 |
| C03H601 | 2.40 | 0.18 / 0.15 / 1.0 | 4.5 | 0.339 | 0.330 / 0.349 |
| C02H101 | 3.40 | 0.19 / 0.10 / 1.0 | 19.5 | 0.361 | 0.372 / 0.350 |
| C02H201 | 2.48 | 0.19 / 0.10 / 1.0 | 3.5 | 0.322 | 0.318 / 0.326 |
| C09H501 | 3.32 | 0.23 / 0.25 / 0.70 | 5.5 | 0.360 | 0.363 / 0.358 |
| C08H101 | 3.53 | 0.24 / 0.20 / 0.70 | 2.6 (record ends) | 0.342 | 0.327 / 0.357 |

- **Level:** 0.321–0.361, median 0.345, twelve runs.
- **Trends:** none beyond ±0.02 in O2, CO2 or pressure. Droplets of 3.3 mm and above sit about 0.015 higher than smaller ones (medians 0.349 and 0.333).
- **Helium, H19H101:** 0.43 over the stage, decaying from 0.48 to 0.40; the dossier's 0.48 is the early value.
- **2 atm, 248F001:** 0.49 (0.47–0.52), still burning at the end of the record.
- **14% O2, N04H101:** 0.18–0.22 and decaying, not a sustained cool flame.
- **Partial stages only:** 194F001 (0.38–0.40 for 2 s, then a 22 s gap) and C10H101 (0.28 for 1.4 s).

## H2 Cool-flame end diameter — CONFIRMED, with three corrections

My end is the first time at least 1.5 s after loss that the 1 s rate stays below f·K_c for 2 s. Primary f = 0.5; I also ran f = 0.75, f = 0.25, the dossier-like absolute 0.15, and a 0.5 s window.

| pO2 atm | Run | D0 | D_ce (f = 0.5) | D_ce/D0 at 0.75 / 0.5 / abs 0.15 | Dossier | Form β = 0.10 / 0.12 / 0.125 |
|---|---|---|---|---|---|---|
| 0.140 | N04H101 | 4.16 | 3.91 | 0.953 / 0.940 / 0.954 | 0.95 | 0.665 / 0.814 / 0.900 |
| 0.143 | 194F001 (0.7 atm) | 3.56 | bracket only | 0.67–0.82 | not listed | 0.64 / 0.76 / 0.82 |
| 0.151 | N03H101 | 2.55 | 1.64 | 0.680 / 0.645 / 0.645 | 0.66 | 0.590 / 0.656 / 0.686 |
| 0.160 | N02H101 | 3.34 | 1.93 | 0.584 / 0.579 / 0.579 | 0.57–0.59 | 0.543 / 0.576 / 0.589 |
| 0.160 | N02H203 | 2.52 | 1.48 | 0.596 / 0.588 / 0.571 | 0.57–0.59 | same |
| 0.162 | C09H501 (0.7 atm) | 3.32 | 2.01 | 0.612 / 0.606 / 0.594 | 0.57–0.59 | 0.536 / 0.565 / 0.577 |
| 0.168 | C04H101 | 3.76 | 2.16 | 0.577 / 0.573 / 0.573 | 0.57–0.59 | 0.509 / 0.524 / 0.530 |
| 0.168 | C04H201 | 2.90 | 1.71 | 0.601 / 0.591 / 0.589 | 0.57–0.59 | same |
| 0.180 | 301F001 | 3.43 | 1.62 | 0.479 / 0.474 / 0.474 | 0.47 | 0.47 |
| 0.180 | C03H601 | 2.40 | 1.24 | 0.535 / 0.518 / 0.518 | 0.51 | 0.47 |
| 0.180 | C03H501 | 3.74 | still burning at 1.78 | ≤ 0.475 | not listed | 0.47 |
| 0.190 | C02H101 | 3.40 | 1.44 | 0.430 / 0.423 / 0.419 | 0.42 | 0.444 / 0.436 / 0.434 |
| 0.190 | C02H201 | 2.48 | 0.97 | 0.406 / 0.392 / 0.389 | 0.39 | same |
| 0.211 | H19H101 | 3.54 | 1.61 | 0.469 / 0.455 / 0.454 | 0.42 | 0.398 / 0.381 / 0.375 (before the helium factor) |

- **Sensitivity:** the f = 0.75, f = 0.5 and absolute-0.15 definitions agree within 0.02 in D_ce/D0 (N03H101 0.035). The f = 0.25 threshold reads 0.01–0.11 lower because of long tails; do not use it.
- **Correction 1, H19H101:** the roll-off is at 1.61–1.62 mm (t ≈ 21.1 s), so D_ce/D0 is 0.455, not 0.42. The value 0.42 is the diameter after a 3.5 s gap in the series (1.48 mm). With the dossier's helium factor (c_He = 1.0, ×1.15) the form gives 0.43–0.46, which matches.
- **Correction 2, D0 exponent:** two-point values on the three pairs are 0.94 (N02), 0.88 (C04) and 1.24 (C02). Across definitions they span 0.84–1.24; the absolute-0.15 definition reproduces the dossier (1.05, 0.89, 1.23). A fourth pair the dossier did not use, C03H501/C03H601, gives at most 0.73–0.80. Against the diameter at flame loss instead of D0 the exponents are 0.63–0.83, so D0 is the better scale.
- **Correction 3, β:** 0.10 is too shallow. It under-predicts N03H101 by 9%, the 16% runs by 6–8% and N04H101 by 29%. β = 0.12–0.125 is within ±13% on eleven plateaus; N04H101 needs 0.125 or more. Two-point β values run from 0.085 to 0.13 depending on the pair.
- **Partial pressure, not mole fraction:** yes.
  - C09H501 (23% O2, 0.70 atm) gives 0.59–0.61, the same as 16% O2 at 1 atm (0.57–0.60); by mole fraction it would be below 0.40.
  - 194F001 (20% O2 in N2, 0.72 atm, no CO2) is 2.91 mm before its gap and flat at 2.385 mm after it, so D_ce/D0 lies in 0.67–0.82; by mole fraction it would be about 0.40.
  - At the same pO2 of about 0.14, N04H101 reads 0.94 and 194F001 at most 0.82, so the curve is not unique near the limit.
- **CO2 residual:** four runs at 15–25% CO2 sit 7–13% above the form, but C03H501 does not. No consistent CO2 term.

## H3 After the cool flame — WEAKENED

Rates over windows after my cool-flame end, ten runs, D_ce 0.97–2.16 mm; closed form evaluated with D = D_ce (t0 = 0.72·D² s). Measured values with the closed form in brackets:

| Run | D_ce | Cool flame lasted s | 0–2 s | 2–5 s | 5–10 s |
|---|---|---|---|---|---|
| C02H201 | 0.97 | 5.9 | 0.103 (0.079) | 0.073 (0.029) | 0.043 (0.015) |
| C03H601 | 1.24 | 7.1 | 0.117 (0.097) | 0.065 (0.043) | 0.045 (0.023) |
| N02H203 | 1.48 | 7.4 | 0.130 (0.110) | 0.072 (0.055) | 0.049 (0.030) |
| N03H101 | 1.64 | 9.5 | 0.088 (0.117) | 0.067 (0.062) | 0.039 (0.036) |
| C09H501 | 2.01 | 8.0 | 0.153 (0.129) | 0.070 (0.079) | 0.054 (0.049) |
| C02H101 | 1.44 | 21.7 | 0.082 (0.108) | 0.025 (0.052) | no data |
| 301F001 | 1.62 | 21.2 | 0.095 (0.116) | 0.020 (0.061) | no data |
| C04H201 | 1.71 | 14.2 | 0.120 (0.120) | 0.001 (0.066) | 0.055, contains a step (0.038) |
| N02H101 | 1.93 | 19.2 | 0.094 (0.127) | 0.055 (0.075) | 0.014 (0.046) |
| C04H101 | 2.16 | 26.7 | 0.060 (0.133) | −0.054, apparent growth (0.084) | no data |

- **0–2 s:** 0.06–0.15, median 0.10 (claimed 0.09–0.12). It moves with the origin: 0.09–0.16 with f = 0.75, 0.06–0.12 with the absolute-0.15 origin.
- **2–5 s:** −0.05 to 0.07, median 0.06 (claimed 0.02–0.07).
- **5–10 s:** 0.014–0.055, median 0.045, and only seven runs have data (claimed 0.045–0.055 on ten).
- **Closed form:** the 0–2 s amplitude is right (measured over form 0.45–1.3, median 0.9). The decay is not: it is 1.5–2.5 times too fast for droplets of 1.25 mm and below, and too slow for four of the five runs with a long cool flame.
- **t0 ∝ D² is not supported here.** The ratio of the 2–5 s rate to the 0–2 s rate should rise with D (0.37 to 0.63); measured it is 0.71 at 0.97 mm, 0.2–0.3 at 1.4–1.6 mm and 0.46–0.59 at 1.9–2.0 mm.
- **What does order the runs:** how long the cool flame lasted. After 10 s or less, rates are 0.09–0.15, then 0.065–0.073, then 0.04–0.054, then 0.01–0.04 over 10–15 s. After 14 s or more they are 0.06–0.12, then −0.05 to 0.055, then 0.015 or less. The second group is contaminated by the vapour cloud (see below).

## H4 After a hot flame in helium — CONFIRMED; N2/CO2 without cool flame — CANNOT TELL

Windows after t_L; the bracket is the range when t_L is moved ±0.15 s. The form uses K0 = 0.40 and D at loss.

| Run | D at loss | 0–1 s | 1–2 s | 3–5 s | 8–12 s | 12–20 s | Form / measured, 0–2 s |
|---|---|---|---|---|---|---|---|
| H12H101 | 2.33 | 0.51 (0.44–0.57) | 0.40 | 0.22 | 0.082 | 0.041 | 0.58 (0.57–0.61) |
| H13H201 | 2.43 | 0.58 (0.55–0.62) | 0.46 | 0.22 | 0.098 | 0.034 | 0.52 (0.50–0.55) |
| H13H101 | 3.31 | 0.59 (0.55–0.59) | 0.52 | 0.25 | 0.147 | 0.064 | 0.56 (0.55–0.59) |
| Claimed | | 0.45–0.56 | 0.40–0.50 | 0.20–0.24 | 0.08–0.14 | — | 0.55–0.70 |

- **Rates:** all within 0.03–0.06 of the claim, slightly higher in the first second with my t_L.
- **Form:** K0 = 0.40 gives 52–58% of the first two seconds (50–61% with the t_L shift), the low end of the claim. From 12 s on, form over measured is 0.94–1.23.
- **D² scaling at late times holds in helium:** large over small is 1.5–1.8 at 8–12 s and 1.6–1.9 at 12–20 s; the form says 1.6–1.8. That rests on one large and two small droplets. It does not hold at 3–5 s (ratio 1.0–1.1 against 1.35).
- **No step at loss:** the rate is 0.63–0.74 in the second before loss and 0.63–0.67 in the first half second after.
- **N2/CO2 without cool flame:** train has no clean case. Every tracked N2/CO2 run goes to a cool flame; C09H101, C10H101 and C01H302 have only 0.5–1.4 s of track.
- **N04H101 is the only candidate** (14% O2, 4.15 mm, weak cool flame possible). Measured 0.08 / 0.24 / 0.21 / 0.115 / 0.077 against the form's 0.37 / 0.31 / 0.23 / 0.14 / 0.10. The rate rises after loss, which a self-cooling law cannot do, so K0 = 0.40 does not fit its first second and is 10–30% high afterwards.

## H5 Early rate — direction CONFIRMED, 3.4 mm band WEAKENED

Deficit is K_ref minus the window rate, with K_ref over [max(2 s, 0.25·D0²), min(t_L − 0.5, track end − 0.2)]. I excluded runs with diameter artefacts.

| Class | n | 0.3–1 s, median (quartiles; range) | 1–2 s | 2–3 s |
|---|---|---|---|---|
| 3.2–3.7 mm | 10 | 0.12 (0.08–0.14; −0.08 to 0.28) | 0.066 | 0.050 |
| 2.2–2.75 mm | 10 | 0.025 (−0.055 to 0.06; −0.16 to 0.09) | 0.014 | 0.005 |
| 2.8–3.1 mm | 6 | 0.056 (0.03–0.07) | 0.040 | 0.033 |

- **3.4 mm:** the claimed 0.13–0.30 is the upper half. Runs in that band: 209F001 0.28, H19H101 0.17, 061F002 0.15, H12H101 0.13, 301F001 0.13. Below it: C09H501 0.11, C08H101 0.09, C02H101 0.08, 194F001 0.065, N01H102 −0.08.
- **2.5 mm:** consistent with 0–0.08, with three noisy negatives.
- **Timing ∝ D0²:** supported qualitatively. In five same-atmosphere families the rate reaches its late value at clock 0.2–0.4 for all sizes, while the times range from 1.5 to 4.5 s.
- **E-fold of 0.31 clock units:** cannot be pinned. On the seven long-flame 3.4 mm runs the deficit medians (0.129, 0.067, 0.058) give about 2.3 s, or 0.20 clock units, with single runs from 0.06 to 0.46. Some runs (209F001, 301F001) also show a fast part gone within 1–2 s.

## H6 Hot rate by class — CONFIRMED within about ±0.05

Window: [max(1 s, 0.15·D0²), min(t_L − 0.5, track end − 0.2)].

| Class | K mm²/s | Ratio to 0.544 | Claimed |
|---|---|---|---|
| N2 18% small / large (301F002 / 301F001) | 0.505 / 0.405 | 0.93 / 0.74 | 0.94 / 0.77 |
| N2 16% small / large (N02H203 / N02H101) | 0.424 / 0.350 | 0.78 / 0.64 | 0.77 / 0.62 |
| N2 15% (N03H101) | 0.373 | 0.68 | 0.71 |
| N2 20%, 0.7 atm (194F001) | 0.467 | 0.86 | 0.88 |
| CO2 20–25%, 0.7 atm (five runs, 2.1–3.5 mm) | 0.505–0.564 | 0.93–1.04 | 0.96–1.03 |
| CO2 5–10%, 1 atm (six runs) | 0.450–0.531 | 0.83–0.98 | 0.86–0.95 |
| CO2 70%, 2 atm (248F001) | 0.55–0.65 by window | 1.0–1.2 | 1.14 |
| He 30–35%, 0.7 atm (four runs) | 0.632–0.702 | 1.16–1.29 | 1.15–1.32 |
| He 10–15%, 1 atm (H02H101, H19H101, H19H201) | 0.541 / 0.563 / 0.648 | 0.99–1.19 | 1.06–1.22 |

- **Air reference:** by the same window, five clean air runs give 0.426, 0.497, 0.510, 0.554, 0.558; median 0.51. The stated 0.544 is the upper end, so ratios to air measured my way are about 7% higher than the table. Air records in train are poor: tracks end at 4–10 s of 11–24 s burns.
- **C09H101** (4.14 mm, same class as the 0.7 atm CO2 runs) burns at 0.36, a ratio of 0.66.

## H7 Clock at loss — CONFIRMED for N2/CO2 (17 runs); the helium statement is REFUTED for one run

| D0/D_crit (carried) | My clock values | Claimed band |
|---|---|---|
| 1.03–1.09 | C08H101 0.81, C02H201 1.14, 194F001 0.625, C01H302 0.77 (0.82 with the csv end) | 0.6–1.1 |
| 1.23–1.35 | C09H501 0.80, N02H203 0.70, C03H601 0.76, 301F001 0.405 | 0.41–0.80 |
| 1.45–1.68 | N03H101 0.52, C02H101 0.385, C09H101 0.355, N02H101 0.305 | 0.31–0.53 |
| 1.90 | C10H101 0.32 | between bands |
| 2.01–2.05 | C04H201 0.246, C03H501 0.20 | 0.20–0.25 |
| 2.61–2.68 | C04H101 0.123, N04H101 0.089 | 0.09–0.12 |
| Helium 0.98 / 1.08 / 1.30 | H12H101 0.72, H13H201 0.57, H13H101 0.316 | "low end, not outside" |

- **H13H101** is at D0/D_crit 1.30 with clock 0.32, below the 0.41 floor of its band. H13H201 (0.57) is just under its 0.6 floor. Helium loses its flame earlier than the N2/CO2 bands say.
- **N04H201** (1.18, clock 1.11) is above both neighbouring bands, but its droplet track ends at 1.2 s, so it is weak evidence.
- **C10H101** at 1.90 burns as long as runs at about 1.5, which suggests the carried D_crit for 30% CO2 at 0.7 atm (1.88 mm) is too small.
- **2 atm run:** 248F001 at 3.56 has clock 1.07, the known failure.

## H8 UV flame/droplet ratio at 3 s — camera values measured; CANNOT TELL against the dossier

The dossier publishes only law over measured, so there is no camera number to compare. My ratio is the median UV flame diameter over 2.85–3.15 s divided by droplet diameter at 3 s. It agrees with the `uv_ratio_3` column of the research5 csv within 0.1 on all 42 runs.

| Class | n | Median (range) |
|---|---|---|
| N2 1 atm | 14 | 8.96 (7.18–11.91) |
| N2 0.7 atm | 3 | 7.04 (6.58–8.77); 194F001 alone is 8.77 |
| CO2 0.7 atm | 8 | 8.76 (7.31–11.38) |
| CO2 1 atm | 8 | 10.21 (8.62–18.59) |
| CO2 2 atm | 1 | 8.48 |
| He 0.7 atm | 4 | 10.18 (9.68–10.64) |
| He 1 atm | 4 | 9.99 (8.53–11.21); H02H101 alone is 10.48 |

My counts are larger than the dossier's (42 against 32) because I kept every run with both traces at 3 s, including the 30% O2 and calibration runs.

## Numbers I would hand the author

- **Cool-flame rate:** 0.345 mm²/s (0.32–0.36) on the twelve runs of the H1 table, with no O2, CO2 or pressure term between 0.7 and 1 atm. Helium 15%: 0.43 stage mean (H19H101). 2 atm warm branch: 0.49 (248F001).
- **D_ce/D0 against pO2:** the H2 table, with H19H101 at 0.455 and 194F001 as a bracket of 0.67–0.82. If the form is kept, use g0 = 0.47 with β = 0.12–0.125, not 0.10. D0 exponent 0.95 (0.8–1.25, four pairs).
- **After a cool flame** (my end definition, ten runs):

| Window | All ten | Cool flame 10 s or less (five runs) | Cool flame 14 s or more (five runs) |
|---|---|---|---|
| 0–2 s | 0.10 (0.06–0.15) | 0.09–0.15 | 0.06–0.12 |
| 2–5 s | 0.06 (0.00–0.07) | 0.065–0.073 | −0.05 to 0.055 |
| 5–10 s | 0.045 (0.014–0.055), seven runs | 0.04–0.054 | 0.015 or less |

- **After a hot flame with no cool flame, helium 30–35% at 0.7 atm** (H12H101, H13H201, H13H101): 0–1 s 0.51–0.59; 1–2 s 0.40–0.52; 2–3 s 0.29–0.35; 3–5 s 0.22–0.25; 5–8 s 0.11–0.15; 8–12 s 0.08–0.15; 12–20 s 0.03–0.065.
- **Same, N2:** only N04H101, and it is ambiguous: 0.08–0.12, 0.24, 0.22, 0.21, 0.14, 0.115, 0.077 over the same windows.
- **Same, CO2:** no train run.
- **Closed form:**
  - K0 = 0.17 is a fair 0–2 s amplitude after a cool flame, but do not let t0 grow as D² there.
  - K0 = 0.40 gives 52–58% of the helium first two seconds; D² scaling of t0 is supported only from 8 s on in helium.
  - K0 = 0.40 is not supported for N2/CO2.

## What the dossier did not say

- **First second after loss depends on the clock at loss (N2/CO2).** It is 0.08–0.15 for clock below 0.21 (N04H101, C04H101, C03H501), 0.26–0.35 for 0.25–0.52, and 0.31–0.48 above 0.6. Late-loss runs overshoot (0.37–0.41, settling in about 2 s); the dossier describes only the dip.
- **Vapour-cloud artefact.** After a long cool flame the apparent diameter grows: 301F001, C02H101 and C04H101 show −0.03 to −0.16 mm²/s within 3–6 s of the end, and 301F001's last sample jumps to 2.76 mm. C04H201 has a 0.07 mm step at about 23 s.
- **C01H302.** After the flame end at 7.8 s it shows 0.9 s at 0.34 mm²/s, then D falls from 2.48 to 1.75 mm in 0.5 s as it leaves the view. That fall is an artefact, not burning.
- **Size dependence of the hot rate is steeper than −0.15.** Eleven same-atmosphere pairs give exponents from −0.11 to −0.68, median −0.39.
- **UV ratio at 3 s is strongly size-dependent,** so class medians mix sizes. In N2 at 1 atm it is 7.9–8.8 for 3.3–3.6 mm, 9.1–11.0 for 2.4–3.0 mm and 11.9 for 2.15 mm; in CO2 at 1 atm it reaches 14–19 for 1.35–1.64 mm.
- **Unreliable diameter records:** 20CAL13, 30CAL06, 182F002 and 195F002 (tracker D at t = 0 differs from the sheet D0 by 0.35–0.9 mm), N01H101 (0.3–2 s) and 248F001 (first second).
- **Sheet comments contradict the header atmosphere** in 061F002, C08H101 and H19H101. For H19H101 the cool-flame rate (0.43) and D_ce (0.455) fit 15% helium better than air.

## Runs opened, files written, partition statement

- **Runs opened:** all 49 train runs of `lib.partition_ids("flex_heptane", "train")` — DropletData and TestSheet for each, plus the LLUV and colour traces where they exist (184 files). The per-run list is in `opened_files.txt`.
- **Partition:** every path was built from the file-number alias of a run id asserted to be in the train list. No validation run (conditions included), no test run, no `holdout` folder, `runs/dev/data_ext/` or `tools/wsp/dev_eval.py` was touched. I did not run or copy `runs/dev/research6/flex_heptane/`, and I read no literature.
- **One disclosure:** an initial `head -5` of `runs/dev/research5/flex_flame/flame_end_train.csv` displayed its first four rows, which are methanol runs. They are not in the heptane split at all (they are in the all-fuels FLEX train list) and nothing from them was used; every later read of that csv was filtered to heptane train ids.
- **Tracked files:** none modified.

Files are in `D:\dev\SpaceApps-Ardealu\runs\dev\research7\verify_heptane\`:

- Code: `vload.py`, `vseries.py`, `v0_flame_end.py`, `v1_cool.py`, `v2_after_hot.py`, `v3_hot.py`, `v4_tables.py`
- Tables: `flame_end_reread.csv`, `cool_measure.csv`, `after_hot.csv`, `hot_measure.csv`, `hot_tables.csv`
- Script outputs: `v1_cool.txt`, `v1b_form.txt`, `v1c_post_cool.txt`, `v2_after_hot.txt`, `v3_hot.txt`, `v3b_hot_extra.txt`, `v4_tables.txt`
- Manifest: `opened_files.txt`

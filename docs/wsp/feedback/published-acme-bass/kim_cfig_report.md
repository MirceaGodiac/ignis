# Agent report (Opus 5.5 medium, 15 Nov 2026, extracted from transcript): Kim 2023, Santa 2007, Irace vs ACME CFI-G

I've done all five deliverables. The papers give no predictor of **when** a CFI-G flame goes out, and the only published predictor of **whether** it goes out has no skill on validation for normal flames. Only a whether-type comparison is honest; an extinction-time comparator cannot be rebuilt without inventing constants. I made no commits, switched no branches, read only train and validation records, and passed no `--final-test` or `reveal`.

## 1. Full text and equations

All four sources were read in full. PDFs are under `D:\Caches\Temp\claude\c2f4976cab650b4b\199e808b-35cd-44e2-918a-d476aca69199\scratchpad\papers\`; sha256 hashes are in the JSON file below.
- **Kim et al. 2023 (K23):** the open accepted manuscript.
- **Kim MS thesis 2022 (KT):** downloaded from the University of Maryland repository.
- **Santa et al. 2007 (S07):** the open PDF.
- **Irace's 2022 WashU dissertation (ID22):** its chapters 4–6 are Irace et al. CNF 2021, PCI 39 1905 (critical flux) and PCI 39 "On the existence of steady-state". I used the copy another agent had already saved in `scratchpad\papers\irace\irace_diss\diss2.pdf`, read as untrusted data.

**Equations:**
- **Radius, K23 Eqs 7–8 (normal) and Eq 13 (inverse):**
  - normal: r_hf = ṁ_total / (4π ρD ln(1/(1−Zst)))
  - inverse: r_hf = ṁ_total / (4π ρD ln(1/Zst))
  - ρD = 0.0334 g/m·s at 300 K, fitted assuming a 1400 K hot-flame peak.
  - **Gap I closed:** K23 and KT never say how ρD is carried from 300 K to the flame temperature.
    - Irace 2021 (ID22 Eq 4.10) uses ρD ∝ T_ch^½, with T_ch the mean of ambient and flame temperature.
    - That rule gives ρD_hot = 0.0334·√(850/300) = 0.05622; K23's own Fig 7 implies 0.05623.
    - For cool flames it gives 0.04418, which matches KT's Z_cf values.
- **Extinction radius, Irace PCI 2023a (ID22 Eqs 5.1, 5.2, 5.4):**
  - r_ext = √(ṁ_R / (4π J))
  - J = 0.2 g/m²·s fuel-based for normal flames, 0.68 g/m²·s O₂-based for inverse flames, T_ext = 1130 K.
  - Ethylene at 1 bar only.
- **Whether the flame extinguishes, Irace PCI 2023b (ID22 Eq 6.9):**
  - R = r_f / r_ext, with r_f from Eqs 6.5/6.7.
  - ρD = 1146 g/m³ × 20.4 mm²/s × √(T_ch/298), T_ch = (298 + 1130)/2.
  - R > 1: the flame cannot reach steady state and extinguishes. R < 1: it approaches steady state.
- **Santa 2007:** no closed-form predictor.
  - Q_R/Q_C ≈ 0.7 and T ≈ 1100 K are outputs of a detailed transient code.
  - Its extinction times (about 1–2 s) are drop-tower values.
  - ID22 ch. 5 reports ISS flames going out at loss fractions of 0.61–0.93, so 0.7 does not carry over to ISS flames.

**Per-flame data**, written to `docs\wsp\research\kim2023_flames.json` with provenance:
- **Table 2 (K23) and Table 4.2 (KT):** 15 cool-flame tests, A–G5. KT adds the hot-flame radii (12.4–17.7 mm) and Z_cf.
- **Table E.1 (KT):** maps the tests to our test points.
- **Fig 7 (K23):** the figure is vector, so I read the marker positions exactly from the PDF drawing commands, with axes calibrated from the tick marks. That gives 27 hot and 14 cool (measured, modeled) pairs, plus the droplet point.
- **No per-flame extinction times** beyond the test-matrix burn times. Those are the same as the public PSI tables already in our data.
- **Later CFI-G papers:** no later paper with per-flame extinction times turned up. Stocker's 2022 Central States meeting paper has none.

## 2. A when/whether predictor

- **When:** none published. Kim's radius is quasi-steady, Irace's criterion is when-free, and Santa needs a full numerical code. Any r(t) or t_ext would be our own derivation, so I did not build one.
- **Whether:** Irace's R > 1 criterion is published and complete, and it uses Kim's radius law. Its weak point is that J is published only for ethylene.
  - **Primary rule:** J = 0.2 for normal flames, 0.68 for inverse, taken as published.
  - **Sensitivity rule:** J_F = 0.68/ν_F per alkane.
  - **Third variant:** Kim's ρD in place of Irace's.

## 3. Identification and split

All 27 hot flames in Fig 7 and all 15 Table 2 tests are in train or validation. **None is a test run.**

| Flames | Runs | Split |
|---|---|---|
| Table 2 tests A–E | 21174C6, C4, D1, C3, F1 | validation (2021-06-23) |
| Table 2 tests F1–F5 and G1–G5 | 21291C1 and 21291A1 | train (2021-10-18) |
| The 12 other Fig 7 hot flames | 21291H2, J1, J3, G1, H1, G2, C2, D1, J2 | train |
| | 21174A4, A3, A2 | validation |

- Every match is within 0.03 % of the plotted model value.
- Two pairs were close and settled on the nearer value: J1/H2, and C2 against B1 (0.2 % off).
- K23's Fig 5a flame (n-butane, 2 mg/s, 0.5 bar, extinction at 30 s) is 21174A2 (validation).

## 4. Draft comparator

`tools\wsp\final\posthoc_kim_comparator.py`. It drops test runs by id before parsing them. Aggregates go to `runs/final/20261115/posthoc_kim_comparator_dryrun.json`, per-run rows to `runs/final/20261115/local/posthoc/kim_comparator/`.

**Radius reproduction** on the 15 Table 2 hot flames:

| Comparison | Mean error | Largest or mean absolute |
|---|---|---|
| Rebuilt vs their plotted model | +0.004 % | max 0.68 % (tests A and E) |
| Their model vs measured | +0.9 % | MAE 11.8 % |
| Rebuilt vs measured | +0.9 % | MAE 11.8 % |

**Whether-it-extinguishes dry run**, per run, against the observed self-extinguished (SE) or flow-terminated (FT) label:

| Family, split | Runs (SE / FT) | Accuracy, R > 1 rule | Always-SE base rate | AUC of R |
|---|---|---|---|---|
| Normal, train | 81 (64 / 17) | 0.877 | 0.79 | 0.92 |
| Normal, validation | 28 (22 / 6) | 0.786 | 0.786 | 0.47 |
| Inverse, train | 73 (41 / 32) | 0.658 | 0.56 | 0.62 |
| Inverse, validation | 24 (13 / 11) | 0.792 | 0.54 | 0.66 |

- On normal validation, R > 1 for 93 % of runs, so the rule is in effect "always SE" there and adds nothing over the base rate.
- The sensitivity rule changes almost nothing.
- The Kim-ρD variant shifts the threshold badly (normal accuracy falls to 0.55).

## 5. What can fairly be compared

- **Radius:** not against IGNIS. The ACME releases forecast no radius, and `observations.jsonl` is empty for ACME. Kim's law is only checked against Kim's own data, as above.
- **Whether:** a per-run comparison on the first terminal cause (SE vs FT).
  - IGNIS's score would be P(extinction first) = CIF_ext / (CIF_ext + CIF_term) at its last horizon (120 s), at the earliest cutoff.
  - The metric would be AUC, which needs no calibration, plus hard-label accuracy and Brier.
  - Mapping R to a probability would need fitting, which would be our tuning, not theirs.
- **When:** the IPCW Brier over horizons 6–120 s has no published counterpart.
- **Limits:**
  - An FT label does not mean a steady flame: operators stopped flames on a timeout or the 723 K burner limit.
  - J is an ethylene, 1 bar constant applied here to alkanes at 0.5–3 bar.
  - The normal-flame test partition is only 2 sessions.
  - I have not computed IGNIS F2's P(extinction first) on validation, so no paired number exists yet; the cached forecasts have per-case Brier contributions only. Running it on test would need the sealed test forecasts, which I did not open.

Sources:
- [Kim et al. 2023, NSF PAR](https://par.nsf.gov/servlets/purl/10426596)
- [Kim MS thesis 2022, Univ. of Maryland repository](https://drum.lib.umd.edu/handle/1903/29540)
- [Santa et al. 2007](https://www.grace.umd.edu/~pbs/2007%20Santa%20et%20al.%20CNF.pdf)
- [Irace dissertation 2022, WashU](https://repository.library.washu.edu/eng_etds/747)
- [Irace et al. PCI 2023 record](https://profiles.wustl.edu/en/publications/critical-temperature-and-reactant-mass-flux-for-radiative-extinct/)
- [Waddell thesis 2023](https://drum.lib.umd.edu/items/1d83d9b4-fe61-441b-ba38-df8b7a815e0f)
- [Stocker, Central States 2022 meeting paper](https://www1.grc.nasa.gov/wp-content/uploads/ACME-CSSCI-paper-20220402.pdf)
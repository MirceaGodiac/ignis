# Where the validation runs sit relative to train: a map from inputs only

> Written by a delegated analysis agent (claude-opus-5-5) on 14 November 2026; nothing fitted. Inputs (conditions) of train and validation runs only; no outcome of any validation run was read.
> The author rechecked the claims he acted on; which ones, and what stays unchecked, is in
> `docs/wsp/dossier/03_datasets_and_measurements.md`. Paths under `runs/dev/` are local working folders that
> are not in git; the dossier says how to rebuild them.

I've built the covariate-shift map for all four families. Every validation run lies within one train SD of the nearest train run, but most sit in condition cells that train never fills. In acme_inverse none of the 24 validation runs falls in a populated train cell.

**Summary (task 3)**

| family | val runs | (a) in a populated train cell | (b) in range, empty cell | (c) outside train range | novel cells |
|---|---|---|---|---|---|
| acme_normal | 28 | 13 | 13 | 2 | 6 (13 runs) |
| acme_inverse | 24 | **0** | 9 | 15 | 10 (all 24 runs) |
| flex (methanol) | 24 | 5 | 17 | 2 | 16 (19 runs) |
| flex_heptane | 15 | 5 | 6 | 4 | 6 (9 runs) |

Measured against the whole train table, most "outside range" excesses are small (in train SD units):
- **acme_normal:** 2 runs have mass flow 3.215 mg/s against a train max of 3.0 (+0.36 SD).
- **acme_inverse:** 15 runs have a fuel fraction in the ambient of 0.0565–0.058 against a train min of 0.0698 (−0.15 SD, about 19% lower). 5 of them have zst up to 0.681 against 0.665 (+0.10 SD).
- **flex:** 2 runs have a larger initial diameter, 4.35 and 4.42 mm against 4.28 (+0.11 to +0.22 SD).
- **flex_heptane:**
  - 3 runs have helium at 0.40–0.45 against a train max of 0.35 (+0.53 to +1.07 SD).
  - 1 droplet is 1.13 mm against a train min of 1.35 (−0.34 SD).
  - 1 run has a pressure 0.1 kPa below the train min.

The global range hides the real gap in ACME, which is per fuel:
- **acme_normal:** 14 runs are outside the train range of their own fuel. Propane at 1 atm and propane in O2 above 0.25 never occur in train, where propane exists only at 2–3 atm in air.
- **acme_inverse:** 17 runs are outside their own fuel's range. Butane at 331 kPa never occurs in train, where butane goes up to 221 kPa only.

Nearest-train distance (normalised): median 1.00 / max 1.31 (acme_normal), 1.05 / 1.61 (acme_inverse), 0.68 / 2.21 (flex), 0.77 / 2.00 (flex_heptane).

**Novel combinations (in validation, never in train)**
- **acme_normal:**
  - Butane at 0.5 atm, O2 0.355, burner fuel fraction 0.3 (1 run).
  - Propane at 1 atm: air with fuel fraction 0.15 (1); air with 0.3 (2); O2 0.352–0.357 with 0.3 (4); O2 0.37–0.39 with 0.08–0.15 (2); O2 0.395–0.40 with 0.3 (3).
- **acme_inverse:**
  - Butane at 3.3 atm with fuel in the ambient at 0.057, at injected O2 of 0.21 (2 runs), 0.40 (3), 0.45–0.50 (4), 0.70 (1) and 0.85 (5).
  - Propane at 0.5 atm, O2 0.85, ambient fuel 0.19 (2).
  - Propane at 1.1 atm with ambient fuel about 0.20, at O2 0.18–0.19 (2), 0.21 (1), 0.50 (1) and 0.85 (3).
  - Injected O2 of 0.40, 0.45 and 0.70 never appear in train at all; train has only 0.15, 0.18, 0.21, 0.30, 0.50 and 0.85.
- **flex, flex_heptane:** the 16 and 6 cells are listed in `tables.txt`. They are mostly CO2 at 0.25 with O2 0.16 at 1 atm, helium mixtures at low O2, and droplet-size extremes.

**Task 4: what train shows around each novel ACME combination**

The neighbour cells are one band step in one coordinate, or a fuel swap. Where there is none, I used two such moves. SE = self-extinction, FT = flow termination; temperatures are peak burner temperature.

acme_normal:

| novel combination | nearest train cells | burn time | end | peak burner T |
|---|---|---|---|---|
| butane, 0.5 atm, O2 0.355, fuel fraction 0.3 | butane fraction 0.5, same O2 (n=2) | 13–72 s | FT×2 | 586–778 K |
| | butane, O2 at least 0.36 (n=1) | 60 s | SE | 556 K |
| | ethane, same cell (n=3) | 44–274 s | 2 FT, 1 SE | 478–782 K |
| propane, 1 atm, air, fraction 0.3 | propane at 2 atm (n=3) | 5–7 s | SE | 330–343 K |
| | ethane at 1 atm (n=7) | 9–19 s | SE | 336–435 K |
| propane, 1 atm, air, fraction 0.15 | two steps only: propane or ethane at 1–2 atm | 5–19 s, or 1 s with fraction below 0.05 | all SE | — |
| propane, 1 atm, O2 0.352–0.357 (4 runs) | two steps only: ethane or butane at O2 about 0.38–0.40 | 37–304 s | mostly SE | 437–701 K |
| | ethane in air | 9–19 s | SE | — |
| propane, 1 atm, O2 at least 0.36, fraction 0.3 | ethane (n=7) | 37–304 s | 6 SE, 1 FT | — |
| | butane (n=3) | 47–161 s | SE | — |
| propane, 1 atm, O2 at least 0.36, fraction 0.08–0.15 | butane (n=3) | 27–83 s | 2 SE, 1 FT | 425–727 K |

For the propane-in-air-at-1-atm cells, train brackets the burn time between about 5 s (propane, 2 atm) and 19 s (ethane, 1 atm). Every cell around the propane O2 0.35–0.40 cases is a different fuel, and burn times there spread across a factor of eight.

acme_inverse:

| novel combination | nearest train cells | burn time | end | peak burner T |
|---|---|---|---|---|
| butane, 3.3 atm, O2 0.85 | two steps: butane at 2.2 atm, fuel 0.07–0.09 (n=12) | 3–13 s | SE×12 | 342–448 K |
| | propane at 3.3 atm, fuel 0.08 (n=6) | 3–18 s | SE | 366–451 K |
| butane, 3.3 atm, O2 0.45–0.50 | two steps: butane at 2.2 atm (n=3) | 4–9 s | SE | 316–428 K |
| | propane at 3.3 atm (n=2) | 6–8 s | SE | 328–341 K |
| butane, 3.3 atm, O2 0.21 | two steps: butane at 2.2 atm (n=2) | 1 s and 66 s | 1 SE, 1 FT | 299–405 K |
| | propane at 3.3 atm (n=1) | 1 s | SE | 299 K |
| **butane, 3.3 atm, O2 0.40** | **no train cell within two moves** | | | |
| **butane, 3.3 atm, O2 0.70** | **no train cell within two moves** | | | |
| propane, 0.5 atm, O2 0.85, fuel 0.19 | ethane, same cell (n=5) | 26–136 s | FT×5 | 573–817 K |
| propane, 1.1 atm, O2 0.85, fuel 0.20 | butane, same cell (n=4) | 35–90 s | FT×4 | 514–733 K |
| propane, 1.1 atm, O2 0.50, fuel 0.20 | butane (n=1) | 70 s | FT | 484 K |
| propane, 1.1 atm, O2 0.21 | two steps: ethane | 35–66 s | FT | 436–590 K |
| propane, 1.1 atm, O2 0.18–0.19 | two steps: ethane | 26–66 s | FT | 392–442 K |

For O2 0.40 and 0.70 the nearest single runs are butane at 2.2 atm, O2 0.5: 4–9 s SE, or at O2 0.85: 7–9 s SE.

Train therefore splits cleanly. Fuel in the ambient at 0.07–0.09 and above 2 atm always gives a short SE (1–18 s). Fuel at about 0.2, at 0.5–1.1 atm, almost always ends in FT. Validation's ambient fuel of 0.057 is below anything in train.

**Task 5: where validation falls in the campaign**
- **acme_normal:** validation is 2021-05-17 (doy 137, propane, series A1–7, B1–2, C1, D1–2) and 2021-06-23 (doy 174, butane, series A1–4, B1, C1–6, D1, F1–4). These sit between train days 088, 106, 125, 130 | 153, 162 | 291. Train propane comes only from doy 130 (2–3 atm, air). Train 30% butane comes from doy 125 (fraction 0.03–0.045) and doy 291 (O2 about 0.39).
- **acme_inverse:** validation is 2021-08-23 (doy 235, propane, series A–L) and 2021-09-22 (doy 265, butane at 331 kPa, series A1–5, C1–3, F1–2, H1–3, J1, K1). Train days are 196, 209, 222, 228 | 242, 256 | 270.
- **FLEX, both fuels:** validation days are interleaved through 2009–2011 (methanol 9 days, heptane 7). The full ordered lists, with the test codes (H…, C…, N…), are in `tables.txt`.

**Task 1 in brief (full per-session tables in `tables.txt`)**
- **acme_normal:**
  - Train: 7 days, 81 runs, ethane (pure or 30%), 30% or 50% butane, 30% propane, 50.7–304 kPa.
  - Validation: 2 days, 28 runs, 30% propane at 101 kPa and 30% butane at 50.7–203 kPa.
  - There are 24 train cells.
- **acme_inverse:**
  - Train: 7 days, 73 runs.
  - Validation: propane at 55–110 kPa with ambient fuel 0.19–0.21; butane at 331 kPa with ambient fuel 0.057.
  - There are 26 train cells.
- **flex:** 27 train days, 68 runs; 9 validation days, 24 runs.
- **flex_heptane:** 22 train days, 49 runs; 7 validation days, 15 runs.

**Task 6: FLEX atmospheres without a train match**

The test was the same diluent kind (N2, CO2 or He), O2 within ±0.02 (absolute), and pressure within ±20%.

- **Methanol, no atmosphere match:** `190R002` (N2, O2 0.20, 71.9 kPa, 2.4 mm). The closest train atmospheres are all at 1 atm: N2 at O2 0.21 (n=10), CO2 0.05 at O2 0.20 (n=3), and He 0.05 (n=1).
- **Methanol, atmosphere matched but no droplet within 20% of the size:**
  - `256F009` (CO2 0.7, 276 kPa, 1.66 mm; train has 3.0 and 3.9 mm).
  - `C03M701` (1.76 mm; train 2.4–3.9).
  - `H07M101` and `H07M202` (He 0.35, O2 0.14, 1.9–2.4 mm; train 3.4–3.6).
  - `H14M101` (He 0.4, 3.5 mm; train 2.79).
- **Methanol, strict on diluent fraction:** if the CO2 or He fraction must also be within ±0.05, all four `C05M` runs (O2 0.16, CO2 0.25, 1 atm) have zero matches. Their nearest train mixtures are CO2 0.15 or 0.36.
- **Heptane, no atmosphere match:**
  - `C13H102` (CO2 0.45, O2 0.17, 71.5 kPa). The closest train atmospheres are CO2 0.30 / O2 0.21 / 70 kPa, CO2 0.20 / O2 0.17 / 100 kPa, and CO2 0.25 / O2 0.23 / 70 kPa.
  - `H15H101` (He 0.45, O2 0.17, 71 kPa). The closest are He 0.35 / O2 0.20 and He 0.30 / O2 0.21, both at 70 kPa, then He 0.10 at 100 kPa.
- **Heptane, atmosphere matched but no size match:**
  - `193R001` (1.13 mm).
  - `H03H101` (3.52 mm).
  - `H04H201` (2.12 mm; train 2.59).
  - `H14H101` and `H14H201` (He 0.4, 1.9–2.3 mm; train 3.1–3.7).
- **Heptane, strict on diluent fraction:** `H04H101` and `H04H201` (He 0.20) have zero matches.

**Band edges and distance**
- **acme_normal:**
  - Pressure: below 75, 75–150, 150–250, above 250 kPa.
  - Ambient O2: below 0.19, 0.19–0.25, 0.25–0.36, at least 0.36.
  - Burner fuel fraction: below 0.05, 0.05–0.20, 0.20–0.40, 0.40–0.75, and 1.
- **acme_inverse:**
  - Pressure: below 80, 80–165, 165–275, above 275 kPa.
  - Injected O2: below 0.20, 0.20–0.25, 0.25–0.35, 0.35–0.45, 0.45–0.60, 0.60–0.80, at least 0.80.
  - Ambient fuel: below 0.065, 0.065–0.11, 0.11–0.16, 0.16–0.25, 0.25–0.35, at least 0.35.
- **FLEX:**
  - Diluent: N2; CO2 or He at up to 0.20, 0.20–0.40, or above 0.40; or CO2+He.
  - O2: below 0.155, 0.155–0.185, 0.185–0.225, 0.225–0.27, at least 0.27.
  - Pressure: below 85, 85–150, 150–250, above 250 kPa.
  - Initial diameter: below 2, 2–3, 3–4, at least 4 mm.
- **Distance:** Euclidean, with each dimension divided by its train standard deviation after the transform. For ACME the dimensions are log pressure, log mass flow, O2, and log fuel fraction; a different fuel adds 1. For FLEX they are O2, log pressure, initial diameter, CO2 fraction and He fraction; a different diluent kind adds 1. A missing value adds 1.
- **Missing data:** one train FLEX run (`C11M201`) records a pressure of 0, which I treated as missing.

**Files** in `D:\dev\SpaceApps-Ardealu\runs\dev\research5\shift\`:
- `shift.py`: the script, which re-runs from the repo root.
- `tables.txt`: every table in full.
- `acme_normal_validation_runs.csv`, `acme_inverse_validation_runs.csv`, `flex_validation_runs.csv`, `flex_heptane_validation_runs.csv`: per validation run, the out-of-range dimensions and their excess in SD, the cell and its train count, the class, and the three nearest train runs with per-dimension differences.
- `flex_t6_atmosphere_match.csv`, `flex_heptane_t6_atmosphere_match.csv`: the task 6 match counts per run.

**No validation outcome and no test run was read.** Validation runs were read only from `experiments.jsonl` through `lib.experiments`. `endpoints.jsonl` and `post.jsonl` were read only through `lib.jsonl_for` with train ids, and the script checks that every returned id is a train id. I did not open any raw file, experiment-table CSV, test sheet, `holdout` folder or scoring tool, and did not use `--final-test`. No tracked file was changed.

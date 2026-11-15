# Agent report (Opus 5.5 medium, 15 Nov 2026, extracted from transcript): Irace 2021/2023 vs Flame Design

I've rebuilt their extinction law from the full texts and it matches their own published numbers, but it can't be scored against IGNIS on burn time as it stands. Their paper gives an extinction radius and a burn / doesn't-burn criterion only. Burn time comes from their full numerical model, which needs each flame's measured burner temperature. Nothing was committed, no test-day or `holdout/` file was opened, no IGNIS weight was touched, and no validation outcome was read.

## 1. Full texts and what they publish

**Status.** The ScienceDirect manuscripts refused automated download (HTTP 403/400). The same papers are chapters of the authors' theses, read in full:
- **Irace PhD 2022, WashU** (doi:10.7936/kpc0-th51). Chapter 4 is the 2021 Combustion and Flame paper, chapter 5 is the PCI paper on critical temperature and mass flux (…043), chapter 6 is the PCI paper on steady-state flames (…049). The thesis says so on p. 16.
- **Waddell PhD 2023, University of Maryland** (DRUM). Read for its per-flame tables. No WashU thesis of Waddell exists.
- **Not read:** Waddell et al. PCI 2024 and CTM 2024.

The thesis chapters are the authors' manuscripts, so numbers in the printed papers could differ slightly. PDFs are in `D:\Caches\Temp\claude\c2f4976cab650b4b\199e808b-35cd-44e2-918a-d476aca69199\scratchpad\papers\irace\`.

**Equations (thesis numbering):**
- **Eq. 5.1:** J_F = ṁ_F / A_flame.
- **Eqs. 5.2 and 5.4:** r_ext = √(ṁ_R / (4π J_cr)). J_cr = 0.20 g m⁻² s⁻¹ with ṁ_R the ethylene flow (normal flames), or 0.68 with ṁ_R the O2 flow (inverse; 0.68 = 0.2 × 3.42).
- **Eqs. 4.9 and 6.5:** r_ss = ṁ_tot / (4π ρD ln(1/(1−Z_st))) for normal flames. **Eq. 6.7:** the same with ln(1/Z_st) for inverse flames.
- **Eq. 4.10:** ρD = ρ0·D0·(T_ch/298)^½, with ρ0 = 1146 g/m³ and D0 = 20.4 mm²/s. T_ch is (298+T_ad)/2 in chapter 4 and (298+1130)/2 in chapter 6.
- **Eq. 6.9:** R = r_ss / r_ext. R > 1 means the flame will go out radiatively; R < 1 means it approaches steady state.
- **Eq. 4.1:** Z_st = (1 + 3.42·Y_F/Y_O)⁻¹.
- **Time:** there is no analytical time. The model is a transient 1-D code (SphDiff, 57 species, narrow-band radiation) driven by each test's measured burner-temperature history, reported "on average within 20 %". Extinction is when the modelled radius reaches r_ext.

**Per-flame tables** (all transcribed into `docs\wsp\research\irace2023_flames.json` with page and PDF-index provenance):
- **The 2023 PCI extinction paper has no per-flame table.** Its 49 normal and 7 inverse flames appear only in raster plots (Figs. 5.5 and 5.7).
- **Irace 2021, Table 1 (4 flames):**
  - Measured burn time is given only for flame 3: extinction at 337 s (model 335 s).
  - Flames 1, 2 and 4 were stopped by the operator or the burner limit.
  - Their Table 2 gives steady-state radii for all four.
- **Steady-state paper, Table 1 (6 flames):**
  - IE1 and IE2 are ISS flames: IE1 was stopped at 867.6 s and IE2 at 126.4 s.
  - The four other flames (NS1, NS2, IS1, IS2) are simulations only.
- **Waddell thesis tables:**
  - Table 2.1: 17 normal flames that went out radiatively. I extracted their measured and modelled extinction radius (Fig. 2.13) and extinction temperature (Fig. 2.12) from the vector drawings.
  - Table 2.2: 8 flames used for the burner-temperature study.
  - Table 4.1: 3 inverse flames at 0.5, 1 and 3 atm.
  - None of these lists burn times.

## 2. Matching their flames to our tests

`identify` reads only train and validation rows of the NASA tables, filtering on the test id before any other column is parsed.

**Confident matches:**

| Their flame | Our test | Split | Agreement |
|---|---|---|---|
| IE1 (inverse) | `21328D1` | train | our termination at 867.3 s vs their 867.6 s |
| IE2 (inverse) | `21349M3` | train | 126.2 s vs 126.4 s |
| Waddell 2.2 A | `19115B1` | train | extinction |
| Waddell 2.2 C | `19171D4` | train | extinction |
| Waddell 2.2 D | `19189K1` | train | extinction, 1.31 atm |
| Waddell 2.2 G | `21349M2` | train | see below |
| Waddell 2.2 H | `21349M1` | train | see below |
| Waddell 2.2 E | `22018J2` | validation | |
| Waddell 4.1 C (0.5 atm) | `22035J1` | train | |
| Waddell 4.1 A (1 atm) | `22018G1` | validation | |

- **Waddell 4.1 B (3 atm)** is `22027B1` or `22027D5`, both train.
- **Waddell 2.1 M, N, O** are among `19171D4`, `19171D5`, `19156C2` and `19156C3` (train); the exact assignment is ambiguous.
- **Waddell 2.1 A and C** have one candidate, `19115L1` (train).
- **Discrepancy.** Waddell says all of Table 2.2 went out radiatively. Our endpoints for G and H (`21349M2`, `21349M1`) are operator terminations at about 30 s, and their own R gives 0.78 and 0.89 (below 1).

**Not identified:** all 4 flames of Irace 2021 and Waddell 2.1 B, D–L, P and Q.
- 47 train/validation normal rows are "RAMP" tests, including all of day 19137. They have no numeric conditions in the NASA table and are not in the scored set, which is likely where many of these flames are.
- The remaining ones may be on test days (19123, 19200, 19206 normal; 21361 inverse). **Checking that needs test-day condition rows, so I stopped there for your decision.**
- The 49 and 7 flames of the PCI extinction paper cannot be matched one by one, since they have no table.

## 3. The rebuild: `tools\wsp\final\posthoc_irace_comparator.py`

It has three commands: `literature`, `identify` and `dryrun`. Outputs go to `runs/final/20261115/posthoc_irace/`.

**Fidelity against their published numbers:**
- **Steady-state radii, Irace 2021 Table 2:** 9.06 / 7.83 / 14.24 / 14.99 mm against 9.0 / 7.7 / 14.3 / 14.9 mm. Largest difference 0.13 mm (1.7 %).
- **Extinction radius (Eq. 5.2) against Waddell's 17 measured radii:**
  - Measured critical flux 0.199 ± 0.010–0.013 g m⁻² s⁻¹; they report 0.2 ± 0.014 on their 49 flames.
  - R² 0.990–0.992; they report 0.95.
  - Mean error 2.2–2.6 %, worst 4.7–7.2 %.
  - The ranges come from trying every within-group label assignment (labels are not published within a fuel-flow group).
- **Extraction check:**
  - Their model's R² against measured radius re-derives as 0.978; they publish 0.98.
  - Extinction temperatures re-derive as 1130.0 ± 23.7 K measured and 1125.9 ± 6.0 K modelled; they publish 1130 ± 23 and 1126 ± 6.
- **R criterion:**
  - Steady-state paper, Table 1: 6 of 6 agree (NS1 1.05, NS2 0.85, IS1 1.05, IS2 0.85, IE1 0.85, IE2 0.79).
  - Waddell 2.1: all 17 flames have R > 1 (1.17–2.65).
  - Waddell 2.2: the 4 normal flames and inverse E agree; inverse F, G and H do not (R 0.77–0.91).

**Optional clock.** For a time comparison, the rebuild adds a clock that is not theirs: the derived growth law from `lit_spherical_flames.md` (r_f = r_ss·erfc(r_f/2√(Dt))), using their r_ext, their ρD and D0. On Irace 2021 flame 3 it gives 106 s against 337 s measured.

**Dry run on train days only; no IGNIS release was scored.** No F2 train forecasts exist on disk. The reference columns below were already stored in `train_days_cases.jsonl`. As a check of the scoring harness, F1 re-scored through this script reproduces its stored contributions exactly (difference 0.0).

Integrated Brier, case means (extinction / termination / sum):

| Family, path | Their law + clock | F1 | Ethylene as ethane | Incidence |
|---|---|---|---|---|
| normal, no stop | 0.174 / 0.356 / 0.529 | 0.206 / 0.291 / 0.497 | 0.127 / 0.276 / 0.404 | 0.241 / 0.290 / 0.531 |
| normal, stop | 0.128 / 0.069 / 0.198 | 0.143 / 0.079 / 0.222 | 0.089 / 0.065 / 0.154 | 0.241 / 0.290 / 0.531 |
| inverse, no stop | 0.179 / 0.614 / 0.793 | 0.117 / 0.287 / 0.404 | 0.161 / 0.298 / 0.459 | 0.247 / 0.353 / 0.600 |
| inverse, stop | 0.179 / 0.194 / 0.373 | 0.133 / 0.150 / 0.283 | 0.176 / 0.160 / 0.335 | 0.247 / 0.353 / 0.600 |

Sizes: normal 204 cases / 68 tests / 8 days; inverse 161 cases / 55 tests / 5 days.

- **R gives no separation on normal flames:** all 68 train tests have R > 1, the 28 that went out and the 40 that were stopped alike.
- **Inverse flames:** all 11 that went out have R > 1, as do 24 of the 44 that were stopped.
- **The clock runs late:** median ln(predicted/observed time) is +0.73 for normal flames (about 2.1×) and +2.0 for inverse flames (about 7×).
- **Out of range:** 29 of 68 normal and 42 of 55 inverse train tests are outside the published 1 bar ± 5 %.

## 4. What can be compared fairly, and the limits

**Fair:**
- The Brier score of a step-function forecast. Their R decides whether the flame goes out, the erfc clock decides when, and the known planned stop comes first if it is earlier. This must be labelled "their law plus a derived clock".
- The error in event time on flames that went out.
- R as a classifier of eventual extinction. Those labels are censored by planned stops, so they are weak.

All of these use only inputs IGNIS also has before ignition.

**Not fair, or not possible:**
- Their stated 20 % burn-time accuracy comes from the full numerical model with measured burner temperatures. It is not a forecast and was not rebuilt.
- Their published constants hold only for ethylene at about 1 bar.
- With no planned stop, their law has no termination part, so it is heavily penalised on termination.
- The stop path uses the new planned stop, while F1 used the old stage sum, so that comparison is slightly asymmetric.
- Comparing on their own flames is limited to a handful of train/validation tests and the radius side, since our records hold no flame radius.

## Decision for you

Scoring F2 against this comparator on the test days, and finishing the identification, both need test-day records. I have not opened any. The catch is that the time component of the comparator is ours, not theirs.
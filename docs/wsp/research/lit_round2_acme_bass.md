# Published comparators, round 2: ACME burner flames and BASS/Saffire fabric fronts

15 November 2026, post hoc. A literature search by a research agent (Opus 5.5) for published work that
predicts, for the ISS spherical burner flames (Flame Design, CFI-G, s-Flame) or for BASS/BASS-II/Saffire
thin-fabric spread, **(a)** whether and when a flame goes out, or **(b)** front position or spread rate with
quench. It extends `RESULTS_PUBLISHED_ACME_BASS.md` and the three reports in `feedback/published-acme-bass/`.

Items already covered there are not repeated: Irace 2021 and both 2023 PCI papers, Kim 2023, Santa 2007, the Waddell,
Irace and Kim theses, Zhao & T'ien 2015/2017 (via NASA/TM-20210011385), Ferkul 2012, Olson et al. 2022,
Li & Liao 2021, Budzinski & DesJardin 2023, Li et al. 2019/2021, and NASA PSI-62.

**Rules followed:**
- No file under `holdout/`, no evaluator vault and no split file was opened.
- No IGNIS weight was read or touched.
- Downloads are in
  `D:\Caches\Temp\claude\c2f4976cab650b4b\199e808b-35cd-44e2-918a-d476aca69199\scratchpad\papers\round2\`.
- Page text was read as data. MDPI and ScienceDirect pages were read in the browser pane.

**One caution before using item 1.** Frolov et al.'s published per-flame table names Flame Design flames from the days
19123, 19200 and 19206. Earlier reports flag those days as possible **test days**. The table states, for each flame,
whether radiative extinction was observed and at what radius. Use it only post hoc, as with the Irace test-day
comparison, which is already spent. Do not use it to tune anything.

Verdict scale:
- **rebuildable comparator**: a closed form or a small code with published constants that forecasts from inputs known
  before ignition.
- **per-test numbers matchable**: published numbers tied to identifiable NASA tests.
- **reference only**: neither.

---

## Ranking: the three most valuable new comparators

1. **Frolov, Ivanov, Frolov, Semenov, *Fire* 6 (2023) 285: per-test table and model clock for Flame Design.**
   - It gives our test ids for 24 normal and 13 inverse ethylene flames.
   - For each flame it gives the measured and modelled extinction radius.
   - From Figs. 4–7, read per flame: whether its transient 1-D detailed-chemistry model puts the flame out, and in which
     10 s bracket up to 50 s.
   - The model takes the test inputs and computes the burner heat transfer itself. It is not driven by the measured
     burner temperature, so it is a genuine forward forecast, unlike Irace's.
   - This is the first published per-test *when* for Flame Design that we can match by id.
   - Verdict: **per-test numbers matchable.**
2. **Snegirev, *Combust. Flame* 248 (2023) 112499: closed-form pre-extinction radius and steady-burning criterion.**
   - Gives the critical flame temperature by fuel: 1100 K for ethylene, 1200 K for propane, 1300 K for methane.
   - Every constant is published.
   - Checked against 19 self-extinguished Flame Design flames.
   - Offers a *whether* rule independent of Irace's critical-mass-flux J. Its R_ss/R_cr is built from an energy balance
     with a critical temperature, not from a critical flux, and the fuel-specific temperatures carry to CFI-G alkanes
     and to s-Flame methane without borrowing ethylene constants.
   - Verdict: **rebuildable comparator** (whether; a radius at extinction, not a time).
3. **Thomsen, Fernandez-Pello, Urban, Olson, Ferkul, *PCI* 42 (2026) 106538: analytical limiting-O₂ boundary for concurrent spread over thin SIBAL.**
   - An analytical LOC(U, P) quench boundary.
   - It is the sum of a heat-transfer branch (Eq. 2) and a chemical-kinetics branch (Eq. 6).
   - It is correlated with the available SIBAL LOC data, which is BASS-II.
   - This would be the first published closed-form *whether-quench* predictor for the BASS fabric.
   - **Only the abstract and outline could be read (paywalled)**, so the equations still have to be obtained.
   - Verdict: **rebuildable comparator once the full text is in hand**; reference only today.

Runners-up:
- **Xi, Torero, Guibaud, CNF 2022:** data-assimilation forecasting of concurrent spread, the flame-spread sibling of the
  Xi droplet comparator. It is validated on Saffire-IV PMMA and needs CFD, so it is not rebuildable as published.
- **Xia, Rutgers MS thesis 2015:** pre-flight s-Flame extinction times for H₂/CH₄/N₂ mixtures. The burner and flow
  units differ from the ISS hardware.
- **Han et al., PCI 42 (2026):** Saffire spread-rate power laws in P and X_O₂, exponents only.

---

## A. Spherical burner flames (ACME)

### A1. Frolov, Ivanov, Frolov, Semenov (2023): **rank 1**

**Citation and access**
- S.M. Frolov, V.S. Ivanov, F.S. Frolov, I.V. Semenov, "Spherical Diffusion Flames of Ethylene in Microgravity:
  Multidimensional Effects", *Fire* 6(8) (2023) 285, doi:10.3390/fire6080285.
- Gold open access: https://www.mdpi.com/2571-6255/6/8/285. MDPI refuses curl and WebFetch; the browser pane reads it.
- Figures are saved in `scratchpad/papers/round2/frolov_fire2023_figs/`.

**What it predicts**
- **Model:** transient 1-D and 2-D runs of a conjugate porous-burner model with detailed ethylene chemistry, a
  macrokinetic soot step and radiation. Inputs are the per-test flows and compositions. The supply tube is at 293 K,
  and the porous sphere is solved for, not prescribed.
- **Outputs:** flame radius and temperature vs time, extinction (radius and timing), and a critical temperature of about
  1200 K.

**Per-test numbers.** Tables 1 and 2, transcribed from the HTML:

| No. | Flame | X_O₂ (amb) | X_C₂H₄ (burner) | G_in mg/s | p atm | Z_st | R_fe exp mm | R_fe calc mm |
|---|---|---|---|---|---|---|---|---|
| 1 | 19115B1 | 0.203 | 1.000 | 0.660 | 1.020 | 0.062 | – | – |
| 2 | 19206L6 | 0.203 | 0.291 | 1.980 | 1.010 | 0.184 | – | – |
| 3 | 19171D4 | 0.376 | 1.000 | 2.520 | 1.010 | 0.106 | 32 | 28.0 |
| 4 | 19189K1 | 0.374 | 0.502 | 5.010 | 1.310 | 0.191 | 31 | 27.5 |
| 5–8 | F10, F02, F08, F05 | 0.38–0.40 | 1.0 / 0.288 | 1.22–4.51 | 1.19–1.26 | | – | – |
| 9 | 19156C2 | 0.366 | 1.000 | 2.529 | 1.040 | 0.104 | 32 | 28.0 |
| 10 | 19142J3 | 0.356 | 1.000 | 2.529 | 0.990 | 0.101 | 31 | 28.0 |
| 11 | 19150N1 | 0.296 | 0.168 | 4.885 | 1.010 | 0.360 | – | – |
| 12 | 19150G3 | 0.338 | 0.288 | 8.779 | 1.050 | 0.272 | 35.5 | 30.5 |
| 13 | 19175A3 | 0.391 | 1.000 | 1.960 | 1.270 | 0.110 | – | – |
| 14 | 19206A5 | 0.207 | 0.288 | 8.779 | 1.010 | 0.189 | 31 | 34 |
| 15 | 19206G1 | 0.205 | 1.000 | 2.529 | 1.010 | 0.062 | 32 | 32 |
| 16 | 19206G4 | 0.201 | 1.000 | 0.822 | 1.010 | 0.061 | – | – |
| 17 | 19206L4 | 0.195 | 0.288 | 2.835 | 1.010 | 0.180 | 19 | 19 |
| 18 | 19123F1 | 0.206 | 0.490 | 2.640 | 1.010 | 0.120 | 23 | 24 |
| 19 | 19123L1 | 0.202 | 1.000 | 2.510 | 1.010 | 0.061 | 33 | 32 |
| 20 | 19189J3 | 0.378 | 0.502 | 5.010 | 1.300 | 0.192 | 31 | 28 |
| 21 | 19200H3 | 0.285 | 0.131 | 4.430 | 1.020 | 0.411 | – | – |
| 22 | 19115F1 | 0.204 | 0.292 | 2.180 | 1.040 | 0.184 | – | – |
| 23 | 19123A2 | 0.209 | 1.000 | 1.620 | 1.000 | 0.063 | 25 | 26 |
| 24 | 19123C1 | 0.207 | 0.290 | 4.460 | 1.000 | 0.188 | 24 | 25 |

Inverse flames. Here X_C₂H₄ is ambient, X_O₂ is the burner stream, and G_in is the burner flow:

| No. | Flame | X_C₂H₄ (amb) | X_O₂ (burner) | G_in mg/s | p atm | Z_st | R_fe exp | R_fe calc |
|---|---|---|---|---|---|---|---|---|
| 25 | 21328D1 | 0.257 | 0.212 | 10.05 | 1.03 | 0.211 | – | – |
| 26 | 21349M3 | 0.270 | 0.212 | 9.11 | 1.00 | 0.203 | – | – |
| 27 | 22018H2 | 0.097 | 0.497 | 6.37 | 1.01 | 0.614 | 24 | 20 |
| 28 | 22018J1 | 0.096 | 0.318 | 9.73 | 1.01 | 0.514 | 25 | 21 |
| 29 | 22018G3 | 0.098 | 0.850 | 7.89 | 1.01 | 0.720 | 31 | 28 |
| 30 | 22018G2 | 0.099 | 0.850 | 5.90 | 1.01 | 0.718 | 28 | 24 |
| 31 | 22018G1 | 0.099 | 0.850 | 3.90 | 1.00 | 0.718 | – | – |
| 32 | 21328N5 | 0.080 | 0.850 | 2.27 | 0.96 | 0.759 | – | – |
| 33 | 22035J2 | 0.096 | 0.850 | 5.90 | 0.51 | 0.725 | – | – |
| 34 | 21340M1 | 0.121 | 0.850 | 9.22 | 1.01 | 0.676 | – | 29 |
| 35 | 21349N3 | 0.251 | 0.212 | 9.10 | 0.52 | 0.215 | – | – |
| 36 | 21349N4 | 0.246 | 0.212 | 10.03 | 0.52 | 0.218 | – | – |
| 37 | 22018B1 | 0.168 | 0.850 | 4.7 | 1.00 | 0.601 | – | – |

- The computed extinction temperature is 1200 K for every flame with a calc radius.
- A "–" means no extinction radius is reported for that flame. The paper says grey shading (lost in the HTML) "marks
  the flames for which radiative extinction was observed in experiments and/or in calculations".
- Flame 34 has a calc radius but no experimental one: the model puts it out, but the experiment's outcome is not stated.

**When: model extinction brackets**, read from Fig. 6. The figure plots R_f/G_in at 10, 20, 30, 40 and 50 s and shows only
flames still burning in the 1-D model. Fig. 7 shows the model-extinguished set.

| Model extinction by | Flames |
|---|---|
| 10–20 s | 19 (19123L1), 15 (19206G1), 14 (19206A5) |
| 20–30 s | 23 (19123A2); inverse 29 (22018G3; Fig. 5 gives about 28.5 s) |
| 30–40 s | 18 (19123F1), 24 (19123C1), 12 (19150G3), 4 (19189K1; Fig. 4 gives about 41 s, so the bracket edge is soft); inverse 28 (22018J1), 34 (21340M1), 30 (22018G2) |
| 40–50 s | 20 (19189J3) |
| still burning at 50 s in the model | normal 1, 2, 3, 5–11, 13, 16, 17, 21, 22; inverse 25–27, 31–33, 35–37 |

Flames 3, 9, 10, 17 and 27 have a calc extinction radius, so their model extinction falls after 50 s.

**Two full histories**, read from the figures:
- **19189K1, Fig. 4:** the model goes out at about 41 s, against a measured last point at about 40.5 s, with
  temperatures logged to about 43 s.
- **22018G3, Fig. 5:** the model goes out at about 28.5 s. The measured radius ends at about 17.7 s, so the model is late
  by about 60 %.

**Closed forms published in the figures.** Both are fits to their own 1-D runs, not to measurements. G_in is in mg/s and
R in mm.

| Quantity | Normal flames | Inverse flames |
|---|---|---|
| Radius at 10 s, R_f/G_in (Fig. 6a) | 2.31 + 24.8·exp(−15.15 Z_st) | 0.85 + 0.15·exp(6.99 Z_st) |
| Radius at extinction, R_fe/G_in (Fig. 7) | 1.51 + 22.3·exp(−9.35 Z_st) | 2.11 + 0.00015·exp(12.99 Z_st) |

**Coverage:** Flame Design, normal flames from 2019 and inverse flames from 2021–22. F02, F05, F08 and F10 are not ISS
ids; they are presumably drop-tower or ground runs.

**Caveats:**
- **Flows disagree with the companion paper.** The same authors' *Mathematics* 11 (2023) 261 table lists different
  G_in for some of the same ids: 19206L6 0.66 vs 1.98, 19171D4 1.372 vs 2.52, 19189K1 1.372 vs 5.01. One of the two
  tables uses different flows, perhaps an ignition or initial-stage flow. Check against our condition rows before
  matching.
- **Test-day flames.** Ids on days 19123, 19200 and 19206 are flagged above as possible test-day flames.

**How to use it.** As a step forecast on matched ids:
- **Whether:** the model's extinguish-or-not.
- **When:** the bracket midpoint, or the exact time for the two flames with a full history.

Its errors are visible on the two full histories: almost exact for 19189K1, about 60 % late for 22018G3.

### A2. Snegirev (2023): **rank 2**

**Citation and access**
- A.Yu. Snegirev, "On radiative extinction of microgravity diffusion flames", *Combust. Flame* 248 (2023) 112499,
  doi:10.1016/j.combustflame.2022.112499.
- Open: the publisher has a bronze copy, and the accepted manuscript is at
  https://biblio.ugent.be/publication/01GVDC1WNKVD592S75PZ2VRNPW. Read in full.

**Predictor.** Optically thin flame ball, with a critical flame temperature T_cr.

| Eq. | Name | Formula |
|---|---|---|
| 10 | Steady radius, no losses | R_f = (ṁ/(4πk/c_p)) / ln(1 + Y_O₂,∞/σ_O₂,dil) |
| 22 | Radiative fraction at extinction, no burner loss | f_r,cr,0 = 1 − c_p(T_cr − T_∞)(1 + σ_O₂,dil/Y_O₂,∞)/Δh_c,dil |
| 21 | Pre-extinction radius, no burner loss | R_f,cr,0 = [ (3/4π) · f_r,cr,0 · ṁ·Δh_c,dil / (4κ_f σ (T_cr⁴ − T_∞⁴)) ]^(1/3) |
| 20 | Pre-extinction radius, with burner loss | R_f,cr = R_f,cr,0 · [ (1 + √(1 − 4R_s³/(f_r,cr,0·R_f,cr,0³)))/2 ]^(1/3) |
| 31 | Critical heat release for steady burning | q_ch,cr ≈ [ 4π(k/c_p) · ( (3/4π)·f_r,cr,0/(4κ_fσ(T_cr⁴ − T_∞⁴)) )^(1/3) · (Δh_c/σ_O₂)·Y_O₂,∞ ]^(3/2) |

- **Whether:** a flame is steady if R_f < R_f,cr (Eq. 10 against Eq. 20) and self-extinguishes if R_f > R_f,cr.
- **Dilution:** Δh_c,dil = Y_fuel·Δh_c and σ_O₂,dil = Y_fuel·σ_O₂ (Eq. 3).
- **κ_f** (Eq. 35) = P·(X_CO₂,st·κ_P,CO₂(T) + X_H₂O,st·κ_P,H₂O(T)). Planck means from HITEMP2010, Table 2.

**Published constants:**

| Constant | Value |
|---|---|
| T_cr | 1100 K ethylene, 1200 K propane, 1300 K methane |
| c_p | 1700 J/(kg·K) |
| k | 0.07 W/(m·K) |
| κ_f | 4.1 m⁻¹ (BRE average) |
| T_∞ | 300 K |
| Δh_c (Table 1) | ethylene 47.17 MJ/kg, σ_O₂ = 3.42; propane 46.35 MJ/kg, σ_O₂ = 3.63 |

- **R_s for the spherical burner:** not stated in the part read. For BRE it is R_s = R_b/2. For the 6.4 mm sphere, the
  burner radius of 3.2 mm is the natural choice; flag it as ours.
- **Fit:** the fitted radiative fraction is about 0.48 for propane and 0.59–0.61 for ethylene.
- **Reduction:** without burner loss and gas heating, Eq. 20 becomes Nayagam et al.'s (2019) scaling,
  R_cr = C·[(3/4π)·ṁΔh/(4κσ(T⁴ − T∞⁴))]^(1/3), with C = f_r^(1/3) ≈ 0.75–0.78.

**Coverage:**
- 19 self-extinguished Flame Design flames: ethylene in N₂, X_fuel 0.29–1, X_O₂ 0.20–0.38, about 1 atm. Shown only in
  Fig. 2b against nominal heat release, with no ids.
- Santa 2007 propane drop-tower flames.
- BRE (ACME flat burner) ids are named only for illustration: 20261H1, 20352A2, 19094K1.

**Verdict:**
- **Rebuildable comparator for *whether*** on Flame Design, CFI-G (T_cr 1200 K for propane; ethane and n-butane would
  need a choice) and s-Flame methane (1300 K).
- It predicts no time. A time needs a growth clock, as with Irace.

### A3. Nayagam, Dietrich, Williams (2019)

- **Citation:** "Radiative extinction of burner-supported spherical diffusion flames: a scaling analysis", *Combust.
  Flame* 205 (2019) 368–370, doi:10.1016/j.combustflame.2019.04.027.
- **Access:** closed; full text not read.
- **Law:** R_ext ∝ (ṁ·Y / κ_P)^(1/3), stated in the abstract. Snegirev gives the calibrated form with C ≈ 0.75 (A2).
- **Fit:** Frolov's Fig. 8 shows it underpredicting Flame Design extinction radii by about 20–40 %.
- **Verdict:** rebuildable (radius, whether), but it is a special case of A2.

### A4. Waddell, Yablonsky, Constales, Sunderland, Axelbaum (2024)

- **Citation:** "The effects of elevated pressure on the kinetics and extinction of hot, warm, and cool gaseous ethylene
  spherical diffusion flames", *PCI* 40 (2024) 105400, doi:10.1016/j.proci.2024.105400.
- **Access:** OpenAlex lists a bronze accepted manuscript, but the ScienceDirect /am/ link returned 404 and "View Open
  Manuscript" would not open. Abstract and section snippets read.
- **Content:**
  - Three ISS inverse flames at 0.5, 1 and 3 bar, with burner X_O₂ 0.850, ambient X_C₂H₄ 0.100 and O₂ flow 3.41 mg/s.
  - Extinction temperature falls by about 50 K at 3 bar.
  - SphDiff simulations go to 50 bar.
- **Matching:** these are Waddell thesis Table 4.1 flames, already matched (22035J1, 22018G1, and 22027B1 or 22027D5).
- **Verdict:** **reference only.** No new per-test numbers beyond the thesis, and no time predictor.

### A5. Waddell, Irace, Yablonsky, Constales, Sunderland, Axelbaum (2024)

- **Citation:** "The kinetics and warm flame chemistry associated with radiative extinction of spherical diffusion
  flames", *Combust. Theory Model.* 28(4) (2024) 441–458, doi:10.1080/13647830.2024.2326830.
- **Access:** closed, with no repository copy (OpenAlex). Abstract only.
- **Content:** kinetics of the ethylene flame approaching the 1130 K limit. Warm-flame chemistry delays extinction but
  cannot prevent it.
- **Verdict:** **reference only.**

### A6. Waddell, Dietrich, Nayagam (2026) and the Central States 2024 paper

- **Citations:**
  - "The effect of pressure on radiative losses in hot, warm, and cool microgravity spherical diffusion flames",
    *Combust. Flame* (2026) 114884, doi:10.1016/j.combustflame.2026.114884. Closed.
  - Its precursor, Waddell, Dietrich, Nayagam, CSSCI Spring Meeting 2024: open,
    https://ntrs.nasa.gov/api/citations/20240003229/downloads/Radiation%20Paper%20CSSCI%202024.pdf. Read.
- **Content:** SphDiff burner runs at 0.9 mg/s heptane, 0.5–5 atm, to mimic FLEX/CFI dodecane droplets. Hot flames go
  out near 1200 K; flame growth and the extinction radius fall with pressure.
- **Verdict:** **reference only.** These are droplet-analogue simulations, not ACME tests.

### A7. Xia (2015), Rutgers MS thesis: s-Flame pre-flight

- **Citation:** T. Xia, "Computational simulation of spherical diffusion flame dynamics and extinction in
  microgravity", MS thesis, Rutgers University (2015), advisor S.D. Tse.
- **Access:** open, https://rucore.libraries.rutgers.edu/rutgers-lib/46458/. Read.
- **Content:** transient PREMIX-based code with SNB radiation and reabsorption, a 1.27 cm burner, ambient air, and
  flows in cc/s.

| Case | Fuel blend (H₂/CH₄/N₂ %) | Flow | Model extinction time | Temperature at extinction |
|---|---|---|---|---|
| 2a | 25/20/55 | 5 cc/s | 4.96 s | 1213 K |
| 2b | 20/25/55 | 5 cc/s | 3.73 s | 1224 K |
| 2c | 30/15/55 | 5 cc/s | 7.63 s | 1172 K |
| 2d | 0/45/55 | 5 cc/s | 1.96 s | 1285 K |

- **More cases:**
  - Flows of 5, 10 and 15 cc/s (Table 5.2): extinction is earlier at higher flow.
  - N₂ against He as the diluent (Table 5.3): He flames go out sooner.
  - Flow step-downs for kinetic extinction (Table 4.1): extinction at about 16–17 s.
- **Coverage:** s-Flame pre-flight matrix only; no ISS ids. The burner diameter and flows differ from the ACME ISS
  insert, which used a 6.4 mm sphere for Flame Design and CFI-G (s-Flame not confirmed here).
- **Verdict:** **reference only.** The direction is checkable: its "earlier at higher flow" runs against what
  `phys_sflame_f3.md` reports for the 44 % H₂ bottle.

### A8. Frolov et al., *Mathematics* 11 (2023) 261

- **Citation:** "Soot Formation in Spherical Diffusion Flames", doi:10.3390/math11020261. Gold open access.
- **Content:** a condition table for 31 normal and 12+ inverse Flame Design ids, and soot modelling. No extinction
  numbers.
- **Verdict:** **reference only.** Useful only to cross-check A1's flows; see the flow caveat there.

### A9. Other spherical-flame items checked, all reference only

| Item | Access | Why reference only |
|---|---|---|
| **s-Flame SRD** (Law, Tse, Sacksteder, NASA GRC 2010) | open: https://www1.grc.nasa.gov/wp-content/uploads/s-Flame_SRD_20100125.pdf | Fig. 1.12 gives pre-flight SNB extinction curves (H₂/CH₄ mixtures, 10–30 s scales). Raster plots, drop-tower burner |
| **Stocker, ACME CSSCI 2022** | NTRS 20220007191 | Program summary, no per-test predictions. BRE: "self-extinguish below ~25 % O₂" |
| **ACME publication list** (Stocker, 30 Aug 2023) | www1.grc.nasa.gov | Lists **no s-Flame paper with ISS results** |
| **Rodenhurst, Chao, Sunderland, Axelbaum, CNF 187 (2018)**; **Wang & Chao, CNF 158 (2011)** | | Steady attached-flame and kinetic/radiative extinction theory; no time, no ISS ids |
| **Snegirev et al., PCI 38 (2021) 4815** | UGent copy is HTML-gated, not read | Transient dynamics and lifetime of BRE flat-burner flames. BRE, not the spherical burner. Worth a follow-up if BRE ever gets a published-comparator pass |
| **Tang, Im, Atreya, CNF 157 (2010) parts I–II**; **Chao, Law, T'ien, PCI 23 (1991)** | | Parametric CFD and steady theory; no ISS tests. Not re-read |

### A10. CFI-G alkanes

No new published *when* or *whether* predictor for CFI-G hot-flame extinction was found beyond Kim 2023.
- Snegirev's T_cr gives a propane value (1200 K) only.
- No ethane or n-butane constant was found.

---

## B. Thin-fabric spread: BASS, BASS-II and Saffire

### B1. Thomsen, Fernandez-Pello, Urban, Olson, Ferkul (2026): **rank 3**

- **Citation:** "Extinction of concurrent flame spread over a thin solid", *PCI* 42 (2026) 106538,
  doi:10.1016/j.proci.2026.106538.
- **Access:** paywalled. Abstract, introduction and section snippets read.
- **Predictor:** an analytical limiting-O₂ fraction for concurrent spread over a thin fuel. It has two branches:
  - a heat-transfer limit for flame spread (HTL-FS, Eq. 2), the low-velocity or quench side;
  - a chemical-kinetics limit for fuel regression (CKL-FR, Eq. 6), the blow-off side.
  - Together they give LOC(U, P). It is correlated with LOC data for thin SIBAL and for paper.
- **Coverage:** the SIBAL concurrent LOC data are the BASS-II quench and blow-off points (Zhao et al. 2017 is in its
  reference list). Test ids are not visible in what was read.
- **Verdict:** **rebuildable comparator for whether-quench once the full text is obtained.** Ask the authors, or watch
  for an accepted manuscript. It would score directly against BASS-II quench and no-quench outcomes; our current
  boundary comes from Kimwipes model values (`lit_flame_spread.md` §2).

### B2. Xi, Torero, Guibaud (2022)

- **Citation:** "Data driven forecast of concurrent flame spread in micro-gravity", *Combust. Flame* 241 (2022) 112078,
  doi:10.1016/j.combustflame.2022.112078.
- **Access:** open accepted manuscript at UCL Discovery 10145279. Read.
- **Predictor:** a hybrid model.
  - Gas phase: Emmons/de Ris boundary-layer analysis.
  - Solid: thin-solid ignition-delay closure, t_ig = (π/4)·kρc·((T_ig − T∞)/q̇″)².
  - A 2-D CFD for the flow away from the fuel.
  - Two coefficients are assimilated from the observed pyrolysis length: C₁ on the ignition delay and C₂ on the surface
    heat-loss term in the Spalding B number. The forecast then runs ahead.
  - The assimilated values converge after about 100–200 s of data.
- **Coverage:** Saffire-IV only: one 0.4 × 0.18 m PMMA slab, 1 cm thick, at 20 cm/s and 22 % O₂. Saffire-IV is not in
  our data, and it is not SIBAL.
- **Why it matters:** it is the same "assimilate a prefix, then forecast" setting as IGNIS, by the group whose droplet
  method was our Xi comparator. Its own discussion flags that flame-length forecasts disagree with measurement.
- **Verdict:** **reference only as published** (CFD in the loop, thick PMMA). A reduced Emmons-only version on BASS
  SIBAL tracks would be our construction, not theirs.

### B3. Han, Cui, Thinnakornsutibutr, Liao (2026)

- **Citation:** "Numerical study of concurrent flow flame spread over thin solids in normoxic conditions in microgravity",
  *PCI* 42 (2026) 106327, doi:10.1016/j.proci.2026.106327.
- **Access:** open access. Read in the browser pane.
- **Predictor (2-D transient CFD on SIBAL):**
  - Heat-transfer ratio, Eq. 9: R = 727·P^2.05·X_O₂^4.31 when R > 1, and 1.1×10⁵·P^3.74·X_O₂^7.48 when R < 1
    (R² = 0.95).
  - Spread rate: Vf ∝ P^0.4·X_O₂^0.7 (convection-dominated, R < 1) and Vf ∝ P^1.1·X_O₂^2.8 (radiation-dominated, R > 1).
  - **The prefactors are not printed in the text read.** They are presumably anchored in Fig. 8 together with the
    Cui–Liao duct correction, 1/Vf linear in 1/H.
- **Per-test numbers (Table 2):**

| Test | Vf, model / exp. (mm/s) | Lp, model / exp. (mm) |
|---|---|---|
| Saffire IV | 3.36 / 3.60 | 80.6 / 85.0 |
| Saffire VI | 4.37 / 4.12 | 107 / 116 |

  Table 1 gives the conditions of Saffire I–IV and VI. Saffire I–III are compared only in Fig. 8 (raster).
- **Verdict:** **reference only.** These are exponent-only scalings for Saffire-scale samples. Saffire I–III are in
  IGNIS (PSI-98/99/100) but sit on two sheets.

### B4. Thomsen, Fernandez-Pello, Urban, Ruff, Olson (2018)

- **Citation:** "On simulating concurrent flame spread in reduced gravity by reducing ambient pressure", *PCI* 37(3)
  (2019) 3793–3800, doi:10.1016/j.proci.2018.05.004.
- **Access:** open, PMC6907022. Read via WebFetch.
- **Content:**
  - SIBAL samples (18.0 mg/cm², 150 × 50 mm) at 20 cm/s and 100–30 kPa in normal gravity.
  - Mixed-convection scaling: Vf ∝ P^½·l_p^½·U_f^½·(1 + g²l_p²/U_f⁴)^(1/8) and U_m = (P/P₀)·U_f·(1 + g²l_p²/U_f⁴)^(1/4).
  - **No fitted constants.**
  - Microgravity equivalence at about 30 kPa, from Saffire (2.03 mm/s at 20 cm/s).
- **Verdict:** **reference only.** It needs the pyrolysis length, and the constant is unpublished. The ICES 2018 companion
  (TTU hdl 2346/74186) was blocked by the repository's bot wall.

### B5. Olson, Ferkul, Fernandez-Pello, Miller et al. (2019): not retrieved

- **Citation:** "Flammability Limits from BASS-II Testing in Microgravity Compared to Normal Gravity Limits", ICES-2019,
  TTU hdl 2346/84505.
- **Access:** the item page is behind an AWS WAF challenge (HTTP 202/403), from curl and from the browser pane.
- **Content:** the published BASS-II flammability maps, including SIBAL concurrent, and the "oxygen margin" argument.
- **Overlap:** our `bass_published_burns.json` already holds a `quench_boundary_2p2cm` from TM-20210011385.
- **Verdict:** **per-test numbers likely, not retrieved.** Low marginal value given the TM.

### B6. Other fabric and solid items checked, all reference only

| Item | Content | Why reference only |
|---|---|---|
| **Ferkul et al., ASGSR 2014 poster** "Results from the BASS-II Experiment" (NTRS 20160000593, read) | SIBAL: 29 tests, 6 quenching cases and 1 blow-off case; quench example at 18.7 % O₂ (ignited at 5 cm/s, reduced to 2.2 cm/s) | No predictor |
| **Li, Liao, Ferkul et al., ICES 2020** (PAR 10205846) and **Li, Liao, Ferkul, IMECE 2019** (PAR 10205840) | Confined Combustion baffles on SIBAL; optimum spacing; quench at about 1 cm spacing | Different experiment |
| **Li, T'ien, Ferkul, Olson, Johnston, PCI 40 (2024) 105373** (NTRS 20240007628, read) | SoFIE-GEL PMMA spheres: pulsating quench and blow-off | Not BASS fabric |
| **Olson, PCI 40 (2024) 105495** | Smolder-spot wave behind SIBAL flames | Explains base-tracking noise; no predictor |
| **Gagnon, Carey, Fernandez-Pello, J. Energy Resour. Technol. 143 (2021)** | Artificial neural network for flame spread over electrical wires | The only ML spread forecaster found, and not for our hardware |

### B7. Machine-learning or data-driven forecasting of ISS flames, 2018–2026

Nothing applies ML to ACME or BASS ISS data. Searches covered OpenAlex, standard and extended web search, and
"data-driven", "data assimilation", "surrogate", "neural network" and "forecast" crossed with the experiment names.

The data-driven items are:
- Xi et al. 2022 (B2).
- Xi, Torero, Jahn, PCI 38 (2021) on droplets, already our droplet comparator.
- Gagnon et al. 2021 (wires).

---

## What to do with this

| Priority | Item | Next step | Score as | Separate from IGNIS |
|---|---|---|---|---|
| 1 | Frolov Fire 2023 | Match the 37 ids to our records by id. Confirm flows against our condition rows, given the flow caveat in A1 | Step forecast: model extinguishes or not, at the bracket midpoint, or exactly for 19189K1 and 22018G3 | Train, validation and test-day ids |
| 2 | Snegirev 2023 | Rebuild Eqs. 10, 20, 22 and 35 with the published constants | Whether, alongside Irace's R, on Flame Design and CFI-G train and validation | Different physics from Irace (critical temperature instead of critical flux), so it is a genuinely second published criterion |
| 3 | Thomsen et al. 2026 | Obtain the full text; rebuild LOC(U, P) | Against BASS-II quench and no-quench outcomes, on train and validation burns only | |

Sources (all read as data):
- [Frolov et al., Fire 2023](https://www.mdpi.com/2571-6255/6/8/285)
- [Frolov et al., Mathematics 2023](https://www.mdpi.com/2227-7390/11/2/261)
- [Snegirev, CNF 2023, UGent manuscript](https://biblio.ugent.be/publication/01GVDC1WNKVD592S75PZ2VRNPW)
- [Waddell et al., PCI 40 abstract](https://www.sciencedirect.com/science/article/pii/S1540748924002086)
- [Waddell, Dietrich, Nayagam, CSSCI 2024](https://ntrs.nasa.gov/api/citations/20240003229/downloads/Radiation%20Paper%20CSSCI%202024.pdf)
- [Xia, Rutgers MS 2015](https://rucore.libraries.rutgers.edu/rutgers-lib/46458/)
- [s-Flame SRD 2010](https://www1.grc.nasa.gov/wp-content/uploads/s-Flame_SRD_20100125.pdf)
- [ACME publications 2023](https://www1.grc.nasa.gov/wp-content/uploads/ACME-publications-20230830.pdf)
- [ACME CSSCI 2022](https://ntrs.nasa.gov/api/citations/20220007191/downloads/ACME%20CSSCI%2020220509.pdf)
- [Thomsen et al., PCI 42 abstract](https://doi.org/10.1016/j.proci.2026.106538)
- [Han et al., PCI 42](https://doi.org/10.1016/j.proci.2026.106327)
- [Xi, Torero, Guibaud, CNF 2022](https://discovery.ucl.ac.uk/10145279/)
- [Thomsen et al., PCI 37](https://pmc.ncbi.nlm.nih.gov/articles/PMC6907022/)
- [BASS-II poster 2014](https://ntrs.nasa.gov/api/citations/20160000593/downloads/20160000593.pdf)
- [GEL, PCI 40](https://ntrs.nasa.gov/api/citations/20240007628/downloads/GELsymposiumfinalRev1.pdf)
- [Olson et al., ICES 2019 record](https://ttu-ir.tdl.org/items/1922ad9f-ef4c-4f52-96a4-980c23debe5e)

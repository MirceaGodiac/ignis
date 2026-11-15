# IGNIS against published models and baselines

| | |
|---|---|
| **Scope** | Every head-to-head of the IGNIS hand-written releases against a published combustion model or a classical baseline, on NASA microgravity data: the comparator's equations, how it was rebuilt, cases, metric, scores with intervals, sources and reproduction commands |
| **Audience** | Reviewers, and anyone quoting a number from the pitch deck or the README |
| **Related** | [FORMULAS.md](FORMULAS.md) (the IGNIS models themselves) · [../wsp/RESULTS.md](../wsp/RESULTS.md) · [../wsp/PROTOCOL.md](../wsp/PROTOCOL.md) |

Lower is better for every score in this file. "Ratio" is the comparator's score divided by the IGNIS
score.

**Where the sources live.** Results, scripts and outputs are in the research repository
`SpaceApps-Ardealu`, branch `feat/handwritten-science`, commit `542dad8e` (15 November 2026).
- Paths such as `docs/wsp/…` and `tools/wsp/…` are in that repository.
- The last commit of each script is given beside it.
- The release folders under `configs/models/` and the forwards are identical in this repository.

## Papers we compared against

The strongest results come first. Each IGNIS release was frozen before the data it is scored on here
was read for that purpose. The published models were rebuilt from their papers and scored on exactly
the same cases.

| Paper or law | What it predicts | NASA test it was scored on | IGNIS | Theirs | Ratio |
|---|---|---|---|---|---|
| S.M. Frolov, V.S. Ivanov, F.S. Frolov, I.V. Semenov, "Spherical Diffusion Flames of Ethylene in Microgravity: Multidimensional Effects", *Fire* 6(8) (2023) 285, [doi:10.3390/fire6080285](https://doi.org/10.3390/fire6080285) | Transient 1-D detailed-chemistry model of ethylene spherical flames; when each flame goes out | Flame Design (PSI-10) test day, normal flames, planned stop known: 10 cases, 4 tests, 1 day | **0.027** | 0.330 | **12.2×** |
| P.H. Irace, K.A. Waddell, D. Constales, P.B. Sunderland, R.L. Axelbaum, "Critical temperature and reactant mass flux for radiative extinction of ethylene microgravity spherical diffusion flames at 1 bar", *Proc. Combust. Inst.* 39 (2023) 1905–1913, [doi:10.1016/j.proci.2022.07.043](https://doi.org/10.1016/j.proci.2022.07.043); with the steady-state criterion of the companion PCI 39 paper | Radiative extinction: $R=r_{ss}/r_{ext}>1$ means the flame goes out | Flame Design test days, normal flames, planned stop known: 83 cases, 29 tests, 3 days | **0.061** | 0.493 | **8.1×** |
| A.Yu. Snegirev, "On radiative extinction of microgravity diffusion flames", *Combust. Flame* 248 (2023) 112499, [doi:10.1016/j.combustflame.2022.112499](https://doi.org/10.1016/j.combustflame.2022.112499) | Closed-form pre-extinction radius and a steady-burning criterion | The same 83 cases | **0.061** | 0.493 | **8.1×** |
| Xi, Torero & Jahn, "Data driven forecast of droplet combustion", *Proc. Combust. Inst.* 38 (2021) 4785–4793, doi:10.1016/j.proci.2020.05.012 (DOI from a library catalogue record; author order differs between catalogues); deposited version UCL Discovery 10106831 | Droplet $d^2$ trajectory with the burning constant set by data assimilation from the observed prefix | FLEX (PSI-69) sealed test partition, methanol: 64 cases, 9 sessions | **22.4 µm** | 151.7 µm | **6.8×** |
| P.V. Ferkul, S.L. Olson, M.C. Johnston, J.S. T'ien, BASS-I concurrent spread-rate fits, ASGSR 2012 poster, NTRS 20130012832 | Steady flame-spread rate on the cotton-fibreglass fabric SIBAL: $V_f=0.20\,U+0.56$ mm/s (2 cm sample) | BASS-II (PSI-25) validation burns, measured blind: 3 burns, 11 cases | **0.121** | 0.449 | **3.7×** |
| K.-C. Lin, G.M. Faeth, "Shapes of nonbuoyant round luminous laminar-jet diffusion flames in coflowing air", *AIAA J.* 37 (1999) 759–765 (DOI not verified) | Flame length of a laminar jet in coflow | SPICE (PSI-107) test days: 6 cases | **1.18 mm** | 2.56 mm | **2.2×** |
| Thomson's space-charge-limited current with the saturation law of flame ionisation, as given in J. Lawton, F.J. Weinberg, *Electrical Aspects of Combustion* (1969) | Ion current drawn from a flame by an electric field | E-FIELD Flames (PSI-22) test days: 788 voltage levels | **0.319 µA** | 0.575 µA | **1.8×** |
| The classical $d^2$-law (Godsave; Spalding, 1953), as the reference baseline of the FLEX spec | Droplet diameter, $D^2=D_0^2-Kt$ | FLEX sealed test, methanol 65 cases / n-heptane 43 cases | **18.5 / 32.2 µm** | 31.6 / 54.0 µm | **1.7× / 1.7×** |
| S. Bhattacharjee, A. Simsek, S. Olson, P. Ferkul, critical flow velocity for radiative extinction in microgravity flame spread, *Combust. Flame* 163 (2016) 472–477, [doi:10.1016/j.combustflame.2015.10.023](https://doi.org/10.1016/j.combustflame.2015.10.023) | Flow speed below which a spreading flame quenches by radiation | BASS and BASS-II plastics (PSI-26, PSI-25) test days: 38 cases | **1.54 cm/s** | 2.58 cm/s | **1.7×** |
| F.G. Roper, flame-length correlation for circular ports, *Combust. Flame* 29 (1977) 219–226 (DOI not verified) | Laminar jet flame length | CLD Flame (PSI-21) test days, radiometer level: 63 cases | **0.115** | 0.144 | **1.3×** |

Notes on the scores:
- **Scores and metrics.** Every row is post-hoc or final-run scoring on data that no release was written
  or chosen from. Details are in the sections below.
- **Burner-flame rows** use the integrated Brier sum (extinction plus operator termination), as case
  means.
- **FLEX rows.** The FLEX/Xi row is the mean absolute error of the forecast median. The $d^2$-law row is
  CRPS.
- **Other rows.** Fabric uses tip CRPS in sample widths. SPICE uses the quantile score. E-FIELD and
  CLD use CRPS. Plastics uses interval CRPS.
- **Shared scores.** Irace's criterion and Snegirev's rule mark exactly the same normal test flames as
  "goes out", so their scores coincide (section 2).

## The pitch-deck rows, checked

Slide 5, "Better than the published science" (`docs/pitch/deck/index.html`), plus slide 4 (Saffire).
Each row was checked against the output file named.

| Slide row | Slide says | Source, checked | Status |
|---|---|---|---|
| Flame-out forecast, Frolov et al. 2023 | 0.027 vs 0.330 (12×) | `runs/final/20261115/posthoc_frolov_snegirev.json`; `docs/wsp/RESULTS_PUBLISHED_ACME_BASS.md` §5: test, normal, stop, 10 cases / 4 tests / 1 day, F2 0.027, Frolov 0.330 | matches |
| Will the flame go out?, Irace et al. 2023 | 0.061 vs 0.493 (8.1×) | `runs/final/20261115/posthoc_irace_testdays.json`, block `acme_normal/stop`: F2 sum 0.0612, `irace_R` sum 0.4933, 83 cases, 3 days | matches |
| Flame extinction limit, Snegirev 2023 | 0.061 vs 0.493 (8.1×) | `posthoc_frolov_snegirev.json`, block `test_days:acme_normal/stop`: `snegirev_R_only` sum 0.493 on the same 83 cases. His rule flags exactly the same flames as Irace's R (`feedback/published-acme-bass/frolov_snegirev_report.md`) | matches; equal by construction, not a copy error |
| Droplet burn-down, Xi, Torero & Jahn 2021 | 22.4 vs 151.7 µm (6.8×) | `runs/final/20261115/posthoc_xi_comparator.json`, `flex_methanol/ignis`: session-weighted MAE of the median, D3 22.44 µm, Xi rebuilt 151.68 µm, both on the same 64 cases and 9 sessions | matches. Metric: mean absolute error of the forecast median (not CRPS, not a median of errors), session-weighted |
| Fire spread on fabric, Ferkul et al. 2012 | 0.121 vs 0.449 (3.7×) | `RESULTS_PUBLISHED_ACME_BASS.md` §3: 3 validation burns, 11 cases, D1 tip CRPS 0.121, Ferkul fit as published 0.449 | matches the results file. The script's JSON output was written to a session scratchpad, not to `runs/` |
| Soot-flame length, Lin & Faeth | 1.18 vs 2.56 mm (2.2×) | `runs/final/gen7/final/spice.json`: P3 T2 1.1804, Lin and Faeth 2.5627, 6 of 6 cases | matches |
| Burning plastics, Bhattacharjee et al. 2016 | 1.54 vs 2.58 (1.7×) | `docs/wsp/RESULTS_FINAL_GEN7.md`, `bass`: quench case mean bass4 1.5438, Bhattacharjee 2.5790; by day 2.9103 vs 3.4074 | matches, and IGNIS is also lower on the day mean (1.17×) |
| Slide 4: Saffire, 4.3× | 0.058 vs 0.250 | `docs/wsp/feedback/saffire-transfer/results/README.md`: D1 0.058, persistence 0.250, 8 cases, 2 of 2 sheets lower | matches |

## More wins against classical laws and fitted baselines

The same frozen releases, the same final runs.

| Comparison | Cases | IGNIS | Comparator | Paired difference [95 %] | Source |
|---|---|---|---|---|---|
| FLEX methanol, D3 against the four optimizer-fitted FLEX models (`flex.v3`, `flex.d2_prefix`, `flex.v1`, `flex.d2_population`), diameter CRPS | 65 / 9 sessions | **18.5 µm** | 42.7 (best fitted, `flex.v3`), 55.7, 58.9, 75.4 | −24.2 [−32.7, −15.9] against `flex.v3`; every interval excludes zero | `docs/wsp/RESULTS_FINAL.md` step 1 |
| FLEX methanol, D3 against the $d^2$-law, CRPS | 65 / 9 | **18.5 µm** | 31.6 | −13.1 [−19.0, −7.7] | same |
| FLEX n-heptane, D2 against the $d^2$-law, CRPS | 43 / 7 | **32.2 µm** | 54.0 | −21.8 [−47.1, −4.9] | same |
| FLEX methanol, MAE of the median, against the fitted models | 64 / 9 | **22.4 µm** | 65.1 (`flex.v3`) to 89.7 | −42.7 [−58.9, −26.4] against `flex.v3` | `docs/wsp/RESULTS_POINT_METRICS.md` |
| CFI-G normal flames (PSI-159), F2 against the fitted linear, hazard, piecewise and constant baselines, extinction IBS | 60 / 2 sessions | **0.0414** | 0.0933, 0.1068, 0.2388, 0.2420 | −0.0498 [−0.0566, −0.0430] against the best (linear) | `RESULTS_FINAL.md` step 1 |
| CFI-G inverse flames, F2 against the fitted baselines, extinction IBS | 69 / 3 | **0.1045** | 0.1645 (hazard) to 0.2270 | −0.0553 [−0.1181, −0.0086] against hazard | same |
| SPICE flame length given the flow, P3 against per-fuel climatology | 6 | **1.18 mm** | 10.33 | | `RESULTS_FINAL_GEN7.md` |
| E-FIELD ion current against a constant | 788 levels | **0.319 µA** | 0.986 | | same |
| E-FIELD flame out in the window against the stratified incidence | 39 | **0.055** | 0.146 | | same |
| CLD Flame radiometer against the Roper law with fuel offsets, mean of day means | 63 | **0.113** | 0.175 | | same |
| SAME-R particle diameter against the per-material baseline | 33 | **0.167** | 0.256 | | same |
| BRE extinction (PSI-20) against the incidence baseline | 95 | **0.139** | 0.222 | | same |
| Saffire sheets, D1 against persistence | 8 cases, 2 burns | **0.058** | 0.250 | −0.192; lower on 2 of 2 burns | `feedback/saffire-transfer/results/README.md` |
| BASS-II validation, D1 against persistence | 12 burns | **0.117** | 0.234 | persistence − D1 = +0.117 [+0.051, +0.182] over 12 burns | `docs/wsp/README.md` |

---

## 1. Frolov et al. (2023): flame-out time on Flame Design

**What they publish.**
- **The model.** A transient one-dimensional spherical flame with detailed chemistry that computes the
  burner heating itself. That makes it a real forecast from the inputs of a test.
- **What is read from the paper.** For 37 Flame Design flames, the paper gives whether and when the
  model puts the flame out: 10 s windows in their Fig. 6, and exact times for two flames in Figs. 4 and 5.
- **The transcription** is in `docs/wsp/research/frolov2023_flames.json`, with provenance.
- **Their published comparisons were reproduced** for 22018G3.

**How the comparator is built** (`tools/wsp/final/posthoc_frolov_snegirev.py`, `09c22d3`):
- **Step forecast.** The flame goes out at their modelled time, taken as the window midpoint or the
  plotted time.
- **Stop path.** When the planned stop is known and comes first, the flame ends at the stop.
- **Which flames have a forecast.** It exists for the flames their model puts out within 50 s. A second
  block adds the flames their model never puts out, forecast as no extinction.

**IGNIS model.** `acme_normal.wsp12` F2 (`sha256:55fcf4cb…971d`), frozen before the final run.

**Cases.** Flame Design (PSI-10) test days, split `docs/wsp/atlas/splits/PSI-10.json`. The endpoints
and sealed forecasts are the final run's own (`runs/final/20261115/flame-design.json`).

**Metric.** Integrated Brier score from 6 to 120 s with inverse-probability-of-censoring weights, for
extinction and for operator termination. The sum is reported as a case mean
(`tools/wsp/ext/transfer_acme.py` `score`).

| Block | Cases / tests / days | **F2** | Frolov | F2 − Frolov |
|---|---|---|---|---|
| test, normal, planned stop known | 10 / 4 / 1 | **0.027** | 0.330 | −0.303 (1 day) |
| test, normal, no stop | 12 / 4 / 1 | **0.014** | 0.124 | −0.110 (1 day) |
| test, adding flames their model never puts out, no stop | 15 / 5 / 2 | **0.032** | 0.099 | −0.067 [−0.110, +0.103] |

**Extinction time.** On the test flames that went out, the median absolute error is **3.9 s** for F2
and 15.8 s for Frolov (4 flames). Their model goes out about 2.1 times late.

**Checks.** F2, F1 and the surrogate re-scored by the script reproduce the stored scores on train,
validation and test days, with a largest difference of 0.0.

```bash
python tools/wsp/final/posthoc_frolov_snegirev.py      # -> runs/final/20261115/posthoc_frolov_snegirev.json
```

## 2. Irace et al. (2023) and Snegirev (2023): will the flame go out?

**Irace et al., as published.** The sources are the thesis chapters: Irace PhD 2022, WashU,
doi:10.7936/kpc0-th51. Chapter 4 is *Combust. Flame* 229 (2021) 111373, and chapters 5 and 6 are the two
PCI 39 papers.

$$r_{ext}=\sqrt{\frac{\dot m_R}{4\pi J_{cr}}},\qquad J_{cr}=0.20\ \text{g m}^{-2}\text{s}^{-1}\ (\text{fuel, normal}),\quad 0.68\ \text{g m}^{-2}\text{s}^{-1}\ (\text{O}_2,\ \text{inverse})\qquad\text{(Eqs. 5.2, 5.4)}$$

$$r_{ss}=\frac{\dot m_{tot}}{4\pi\,\rho D\,\ln\!\big(1/(1-Z_{st})\big)}\ \text{(normal)},\qquad r_{ss}=\frac{\dot m_{tot}}{4\pi\,\rho D\,\ln(1/Z_{st})}\ \text{(inverse)}\qquad\text{(Eqs. 4.9, 6.5, 6.7)}$$

$$\rho D=\rho_0D_0\Big(\frac{T_{ch}}{298\ \text{K}}\Big)^{1/2},\ \rho_0=1146\ \text{g m}^{-3},\ D_0=20.4\ \text{mm}^2\text{s}^{-1};\qquad Z_{st}=\Big(1+\frac{sY_F}{Y_O}\Big)^{-1},\ s=3.42\qquad\text{(Eqs. 4.10, 4.1)}$$

$$R=\frac{r_{ss}}{r_{ext}}>1\ \Rightarrow\ \text{the flame extinguishes radiatively}\qquad\text{(Eq. 6.9)}$$

**Fidelity of the rebuild** (`posthoc_irace_comparator.py literature`, `740b3df`):
- The steady radii of their CNF 2021 Table 2 are reproduced within 0.13 mm (1.7 %).
- Fitting Eq. 5.2 to Waddell's 17 measured extinction radii gives a critical flux of 0.199, against
  their 0.20.
- The R rule agrees with their steady-state table in 6 of 6 cases.

**Snegirev, as published.**
- **The law.** A closed-form pre-extinction radius (Eq. 20, with Eqs. 3, 14/22, 17/21 and 35–38), the
  steady radius (Eq. 10), and the test $R_{ss}>R_{cr}$ ⇒ the flame self-extinguishes.
- **His constants:** $T_{cr}$ = 1100 K for ethylene, $c_p$ = 1700 J kg⁻¹ K⁻¹, $k$ = 0.07 W m⁻¹ K⁻¹,
  $T_\infty$ = 300 K, $\Delta h_c$ = 47.17 MJ kg⁻¹, $\sigma_{O_2}$ = 3.42, and HITEMP-2010 Planck means
  at $T_{cr}$.
- **Our assumptions**, flagged in the code: a sphere radius $R_s$ = 3.2 mm, and $\kappa_f$ computed per
  test from Eq. 35.

**Comparator** (`tools/wsp/final/posthoc_irace_testdays.py`, `740b3df`; Snegirev in
`posthoc_frolov_snegirev.py`):
- **"As published" (R only).** The criterion decides whether the flame goes out, and the forecast says
  it goes out at the first horizon. This is the published criterion with nothing of ours added.
- **"+ clock".** Neither paper publishes a closed-form burn time; Irace's times come from a transient
  CFD driven by each test's measured burner temperature, which is not a forecast. So we also pair each
  criterion with our erfc growth clock:

$$r_f=r_{ss}\,\mathrm{erfc}\!\Big(\frac{r_f}{2\sqrt{Dt}}\Big),\qquad t_{ext}=\frac{r_{ext}^2}{4D\,[\mathrm{erfc}^{-1}(r_{ext}/r_{ss})]^2}\quad (R>1)$$

- **Inputs.** It reads only the conditions in F2's prefix, and on the stop path the same planned stop.

**Cases.** Flame Design test days: normal flames on days 19123, 19200 and 19206 (29 tests); inverse
flames on day 21361 (20 tests). Every test is ethylene, inside their 1 bar domain.
- **Metric:** the integrated Brier sum, as case means.
- **Interval:** paired, F2 minus the comparator, 95 % bootstrap over days (10,000 draws).

| Path | Cases / days | **F2** | Irace R (as published) | Snegirev rule (as published) | Irace R + clock | Snegirev + clock |
|---|---|---|---|---|---|---|
| normal, planned stop known | 83 / 3 | **0.061** | 0.493 | 0.493 | 0.145 | 0.156 |
| normal, no stop | 87 / 3 | **0.213** | 0.540 | 0.540 | 0.337 | 0.301 |
| inverse, no stop | 60 / 1 | **0.275** | 1.565 | 1.765 | 0.815 | 0.837 |

Paired F2 minus the comparator, sum [95 % over days]:

| Path | vs Irace R | vs Snegirev rule | vs Irace R + clock | vs Snegirev + clock |
|---|---|---|---|---|
| normal, stop | −0.432 [−0.959, −0.027] | −0.432 [−0.959, −0.027] | −0.084 [−0.223, +0.062] | −0.095 [−0.130, −0.058] |
| normal, no stop | −0.327 [−0.744, −0.031] | −0.327 [−0.744, −0.031] | −0.124 [−0.147, −0.080] | −0.088 [−0.149, −0.041] |

**Reading.**
- **As published.** F2 beats both published laws on every normal-flame path, and every interval
  excludes zero.
- **Paired with our clock.** F2 still beats both, with intervals that exclude zero on three of the four
  paths.
- **Inverse flames.** On the inverse test day no flame went out. Both rules say most would: Irace 15 of
  20, Snegirev 19 of 20.

**CFI-G (PSI-159), whether the flame goes out, train runs.** Here Irace's R is carried to alkanes. On
inverse flames F2 has an AUC of **0.995** against 0.60 for R (71 runs; normal flames 0.94 against 0.92)
(`posthoc_irace_cfig_occurrence.py`, `740b3df`). F2 was typed after its author read train, so this row
describes the published law on these flames rather than a held-out win.

```bash
python tools/wsp/final/posthoc_irace_comparator.py literature   # fidelity against their own numbers
python tools/wsp/final/posthoc_irace_comparator.py identify     # their flames in our records (train/validation days only)
python tools/wsp/final/posthoc_irace_testdays.py                 # -> runs/final/20261115/posthoc_irace_testdays.json
python tools/wsp/final/posthoc_irace_cfig_occurrence.py          # -> runs/final/20261115/posthoc_irace_cfig_occurrence.json
python tools/wsp/final/posthoc_frolov_snegirev.py                # Snegirev blocks
```

## 3. Xi, Torero & Jahn (2021): droplet burn-down

**As published** (deposited version, UCL Discovery 10106831, sections 2.1, 3 and 5). Spalding's
solution for the liquid phase gives the droplet-size forecast

$$R(t)^2=R_0^2-\hat K\,t\qquad\text{(Eqs. 2 and 8)}$$

- $R_0$ is the initial radius.
- $\hat K$ is the one droplet-phase parameter. It is corrected from the observed radius by minimising
  the distance between prediction and observation.
- The observations during thermal swelling are left out.
- Their CFD gas phase and second parameter set only the flame radius, so they do not affect the droplet
  forecast.

**Rebuild** (`tools/wsp/final/posthoc_xi_comparator.py`, `d513ce0`):
- $\hat K$ is fitted by least squares on $R^2$ over the valid prefix (from the largest diameter seen up
  to the cutoff), then refined by Newton steps on the radius residuals, their objective.
- It reads only what every release got at the cutoff. A case with fewer than 3 valid prefix points gets
  no forecast; there were none.

**Fidelity.**
- **Their droplet.** Their droplet was identified as FLEX-094 (`PSI-69:C08H101:2010-06-15`,
  $d_0$ = 3.53 mm, a train run) by matching their traced Fig. 2
  (`docs/wsp/research/xi2021_fig2_heptane_trace.json`).
- **The trace.** The traced prediction against the traced experiment gives 2.42 %, where they report
  2.46 %.
- **The rebuild on FLEX-094.** On their task (cutoff 5 s, scored 5–10 s) the rebuild scores 2.58 %
  nRMSE, where they report 2.46 %.

**Cases.** The FLEX sealed test partition, scored once in the final run of 15 November
(`docs/wsp/RESULTS_FINAL.md`):
- methanol: 64 scored cases of 22 runs in 9 sessions;
- n-heptane: 43 cases of 16 runs in 7 sessions;
- cutoffs 1, 3 and 5 s, horizon 10 s.

Every model is scored on the same continuation observations, opened through
`ignis eval reveal --final-test` (108 reveal-log lines).

**Metrics** (`tools/wsp/final/posthoc_point_metrics.py`, `38ce9bc`), computed on the forecast median at
each observation time with cutoff < t ≤ cutoff + 10 s:
- **MAE** in µm;
- **nRMSE**, the RMSE divided by the mean measured diameter (their metric).

Values are session-weighted (the mean of session means, as the evaluator weights them). Intervals are
95 % bootstrap over sessions.

| Task | Cases | **IGNIS** | Xi rebuilt | $d^2$-law | IGNIS − Xi [95 %] |
|---|---|---|---|---|---|
| Methanol, evaluator window, MAE of the median | 64 | **22.4 µm** (D3) | 151.7 µm | 43.8 µm | −129.2 [−164.6, −98.6] µm |
| Methanol, evaluator window, nRMSE | 64 | **1.52 %** | 6.95 % | 2.52 % | −5.43 [−7.62, −3.77] |
| Methanol, their task, nRMSE | 20 | **1.40 %** | 3.27 % | 1.72 % | −1.87 [−2.84, −1.10] |
| n-Heptane, evaluator window, MAE of the median | 43 | **43.9 µm** (D2) | 122.1 µm | 72.4 µm | −78.2 [−128.1, −33.3] µm |
| n-Heptane, evaluator window, nRMSE | 43 | **5.41 %** | 17.16 % | 6.78 % | −11.75 [−29.76, −1.62] |
| n-Heptane, their task, nRMSE | 12 | **3.20 %** | 16.05 % | 3.22 % | −12.84 [−36.44, −0.58] |
| Their own droplet FLEX-094 (a train run), their task, nRMSE | 1 | **0.19 %** (D2); 0.65 % zero-shot W0 | 2.58 % | 0.92 % | |

**On the heptane rows.** One 1.49 mm heptane droplet (`H05H103`) is one for which their law, with
$R_0$ fixed, predicts burnout while the droplet is still measured. Without it, the paired differences
are −2.81 [−4.51, −1.14] (evaluator window, 40 cases) and −1.08 [−1.76, −0.35] (their task, 11 cases).
Every interval excludes zero either way.

```bash
python tools/wsp/final/posthoc_point_metrics.py   # opens the continuations through the evaluator; -> posthoc_point_metrics.json
python tools/wsp/final/posthoc_xi_comparator.py   # -> runs/final/20261115/posthoc_xi_comparator.json (no new reveal)
python tools/wsp/final/posthoc_xi_window.py       # their 5-10 s window for the releases
```

## 4. Ferkul et al. (2012): fire spread on fabric

**As published.** The BASS-I concurrent spread fits at about 21 % O₂ (ASGSR 2012 poster, NTRS
20130012832), with $U$ in cm/s:

$$V_f\,[\text{mm/s}]=0.20\,U+0.56\ \ (2\ \text{cm sample}),\qquad V_f=0.15\,U+0.07\ \ (1\ \text{cm})$$

The transcriptions are in `docs/wsp/research/bass_published_burns.json`. Its other sources are
NASA/TM-20210011385 §7.1 (Table 7.1, Fig. 7.4) and Olson et al. (UCL Discovery 10164340).

**Rebuild** (`tools/wsp/final/posthoc_bass_published.py`, `2524633`):
- **Anchor.** Every comparator starts from the same point at the cutoff, the value of the least-squares
  line of the last 10 s of tip stills. The comparators then differ only in spread rate.
- **The rates compared:**
  - Ferkul's fit at the track's flow history, as published (`pos12`);
  - the same fit scaled by $X_{O_2}/0.21$ (our scaling);
  - the steady rate fitted over the 10 s before the cutoff, which is the investigators' way of
    measuring $V_f$.
- **Clipping.** Positions are clipped at the sample end.
- **Fidelity.** Our tracks' steady slopes match their published $V_f$ within about 7 %, except one
  burn that the crew called not steady (+18.6 %).

**Cases.** BASS-II (PSI-25) validation burns, measured blind from NASA video: 3 burns, 11 cases (split
`data/splits/bass-4083156d5b18dab4.json`).
- **Cutoffs:** 5, 10, 20 and 40 s after the igniter goes off.
- **Targets:** the flame stills in (cutoff, cutoff + 60 s].
- **Metric:** tip CRPS in sample widths (`wsp-dev front-feedback`, `crates/ignis-wsp-dev/src/front.rs`).
  For a point forecast, the CRPS is its absolute error.

| Forecast | Tip CRPS, sample widths |
|---|---|
| **`bass_front.wsp` D1** (`sha256:ac15e49f…855c`) | **0.121** |
| steady rate over the last 10 s | 0.228 |
| Ferkul 2012 with an O₂ scaling (ours) | 0.352 |
| Ferkul 2012 as published | 0.449 |

D1 is lower than every comparator on all 3 burns. Its mean per-burn margins are −0.108, −0.232 and
−0.328.

**Fresh burns T13 and T20.** These were measured blind after freezing (`e036039`). D1 is again lower
than every published-constant forecast on both:

| Track | **D1** | Ferkul as published |
|---|---|---|
| T13, top view | **0.158** | 0.222 |
| T13, duct | **0.186** | 0.274 |
| T20, top view (1 cm sample) | **0.270** | 0.467 |

Source: `RESULTS_PUBLISHED_ACME_BASS.md` §6.

```bash
python -I tools/wsp/final/posthoc_bass_published.py --partition both
```

## 5. Saffire: from a 2 cm strip to a 40.6 cm sheet

**What was tested.** The adopted fabric model D1 was typed from three 2 cm BASS-II burns. It was run
unchanged, with **zero weights changed** (`docs/wsp/feedback/saffire-transfer/freeze.json`), on the
Saffire I and III sheets: SIBAL, 40.6 cm × 94 cm, flow with the flame at 20 and 30 cm/s, burned aboard
Cygnus. These sheets are about 20 times wider than the strips D1 was typed from.

**Registration and execution.**
- **Inputs:** the width ratio 0.406/0.022, the commanded flow and nominal cabin oxygen. All of them are
  known before ignition.
- **Registration.** The protocol (`docs/wsp/SAFFIRE_TRANSFER.md`, `ba5c44c`) was registered before any
  measurement.
- **Scorer.** The scorer was frozen in `a754fd8`.
- **Measurement.** A sub-agent measured the tracks blind from NASA's mapped frames, and a second model
  checked them (30 of 30 overlays accepted).
- **Chain check.** The scoring chain reproduced the committed train aggregate to 1e-12 before the run.

**Comparator.** Persistence: the tip speed the prefix shows, carried forward (`sha256:ce53fe1f…de46`).

**Metric.** Tip CRPS in sample widths, at cutoffs 5, 10, 20 and 40 s.

| Release | Tip CRPS | Median abs. error | Burns lower than persistence |
|---|---|---|---|
| **D1, unchanged** | **0.058** | 0.075 | 2 of 2 |
| joint Astra (`bass_front.wsp3`) | 0.061 | 0.081 | 2 of 2 |
| persistence | 0.250 | 0.386 | |

That is 8 cases on 2 sheets. By the registered rule, the result is **"transfers"**. D1's forecast is
also conservative: 93 % of targets fall inside its 50 % band.

```bash
python tools/wsp/saffire/score_transfer.py --execute   # one-time look; refuses if the protocol, reader or freeze changed
```

## 6. Lin & Faeth, the space-charge law, Roper, Bhattacharjee: the seventh-generation final run

**Run.** All four were scored in the one final run on the unread test days of the seventh generation
(PROTOCOL 13.7, 15 November). Each published law is built at its published constants by
`tools/wsp/ext/law_score.py` (`3e076f6`). Where a spread is needed, it is a single log-width set on
train days only.
- **Release list:** `tools/wsp/final/releases_final_gen7.json`, sha256
  `f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`.
- **Results:** `docs/wsp/RESULTS_FINAL_GEN7.md` (`cccdf3d`) and `runs/final/gen7/final/<slug>.json`.

### 6.1 SPICE (PSI-107): flame length given the flow, against Lin & Faeth

Lin and Faeth's coflow law, with unity coefficients (`tools/wsp/ext/spice_cases.py:186-197`):

$$L_{LF}=\frac{\dot m}{4\pi\,\mu_{air}\,Z_{st}},\qquad q_p=L_{LF}\,e^{z_p\hat\sigma},\quad \hat\sigma=0.1218\ \text{(train, 42 cases)}$$

Here $\dot m$, $\mu_{air}$ and $Z_{st}$ come from the investigators' Smoke Point Datasheet. The metric is
the quantile score ×2 over the levels 0.05–0.95, in mm.

| Release | Cases | Flame length given the flow |
|---|---|---|
| **`spice` P3** (`sha256:000b88b8…98a0`) | 6 | **1.18 mm** |
| Lin and Faeth coflow law | 6 | 2.56 mm |
| per-fuel climatology | 6 | 10.33 mm |

On the smoke-point length, P3 scores **4.18 mm** against 10.33 mm for the per-fuel climatology.

### 6.2 E-FIELD Flames (PSI-22): ion current, against the space-charge law

Thomson's planar space-charge-limited current, capped by the saturation current of flame ionisation
(`tools/wsp/ext/efield_cases.py:646-672`):

$$I=\min\!\Big(\eta\,\dot N_C\,e,\ \ \tfrac98\,\varepsilon_0\,\mu_{ion}\,\frac{V^2}{d^3}\,A\Big),\qquad \eta=5\times10^{-7},\ \mu_{ion}=1.4\ \text{or}\ 1.9\times10^{-4}\ \text{m}^2\text{V}^{-1}\text{s}^{-1}$$

Here $d$ is the mesh gap (35 or 40 mm) and $A$ the 8.4 cm mesh disc. The spread is log-normal, with
the width set on train.

| Release | Levels | CRPS, µA, case mean | mean of day means |
|---|---|---|---|
| **`efield3`** (`sha256:8575550b…6f04`) | 788 | **0.319** | **0.314** |
| space-charge law up to saturation | 788 | 0.575 | 0.563 |
| constant | 788 | 0.986 | 0.985 |

### 6.3 CLD Flame (PSI-21): radiometer level, against Roper's flame length

Roper's correlation for a circular port (`tools/wsp/ext/cld_cases.py:294-303`):

$$L_f=1330\ \text{s m}^{-2}\ \frac{Q_F}{\ln(1+1/S)}$$

Here $Q_F$ is the volumetric fuel flow and $S$ the moles of air per mole of nozzle fluid at
stoichiometry. The comparator adds one offset per fuel, fitted on train, to $\log_{10}L_f$
(`cld_cases.py:714-726`).

| Release | Cases | CRPS of log₁₀ V, case mean | mean of day means |
|---|---|---|---|
| **`cld3`**; equal to `cld` D1, the one served | 63 | **0.1145** | **0.1134** |
| Roper length plus fuel offset | 63 | 0.1439 | 0.1750 |

### 6.4 BASS and BASS-II plastics (PSI-26, PSI-25): quench speed, against Bhattacharjee et al. 2016

The critical flow velocity for radiative extinction, as restated in NASA/TM-20210011385 §2.2.7, Eq. 2.5
(`tools/wsp/ext/bass1_cases.py:693-718`):

$$V_{g,cr}=\frac{\varepsilon\,\sigma_{SB}\,(T_v^4-T_\infty^4)}{\rho_g\,c_g\,(T_f-T_v)},\qquad T_f=T_\infty+(2199-T_\infty)\frac{X_{O_2}}{0.21}$$

The constants are $T_v$ = 673 K, $T_\infty$ = 300 K, $c_g$ = 1070 J kg⁻¹ K⁻¹ and $\varepsilon$ = 1.
The quantiles are $V_{g,cr}e^{0.4z_p}$, with the width set on train. The metric is the interval CRPS in
cm/s.

| Release | Cases | Case mean | Mean of day means |
|---|---|---|---|
| **`bass4`** (`sha256:ceefc15d…ab86`) | 38 | **1.544** | **2.910** |
| Bhattacharjee et al. 2016 | 38 | 2.579 | 3.407 |

The same final run scored "kept burning after the igniter" at **0.048** (case mean, 42 cases) against
0.071 for the train incidence and 0.075 for "always lit".

```bash
python -I tools/wsp/final/run_final_gen7.py rehearse --slug spice,efield,cld,bass   # the same code path on validation days
python -I tools/wsp/final/run_final_gen7.py final --slug spice --releases tools/wsp/final/releases_final_gen7.json \
    --sha256 f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2 --execute
```

## 7. FLEX and the burner flames against the reference and fitted baselines

**FLEX** (`tools/wsp/final/run_final.py`, `5686866`, step `sealed`).
- **Reference baseline.** The spec's $d^2$-law has a constant population burning rate (0.46 mm²/s for
  methanol, 0.55 mm²/s for heptane), pulled toward the prefix's own rate
  ([FORMULAS.md](FORMULAS.md), Part 1, §3).
- **Fitted baselines.** The four FLEX baselines (`configs/models/flex/v1.toml`, `v3.toml`,
  `d2_population.toml`, `d2_prefix.toml`) are fitted by optimisation on the train partition.
- **Metric.** Diameter CRPS, cluster-equal over sessions, with the evaluator's cluster bootstrap.

| FLEX methanol, sealed test (65 cases, 9 sessions) | CRPS µm | MAE of median µm | D3 − comparator, CRPS [95 %] |
|---|---|---|---|
| **`flex_methanol.wsp2` D3** | **18.5** | **22.4** | |
| $d^2$-law of the spec | 31.6 | 43.8 | −13.1 [−19.0, −7.7] |
| `flex.wsp` W0 (zero-shot) | 37.2 | 48.4 | −18.7 [−35.2, −5.8] |
| `flex.v3` (MAP, fitted) | 42.7 | 65.1 | −24.2 [−32.7, −15.9] |
| `flex.d2_prefix` (fitted) | 55.7 | 89.7 | −37.2 [−45.4, −28.8] |
| `flex.v1` (MAP, fitted) | 58.9 | 75.7 | −40.4 [−56.2, −26.5] |
| `flex.d2_population` (fitted) | 75.4 | 82.9 | −56.9 [−86.5, −30.7] |

| FLEX n-heptane, sealed test (43 cases, 7 sessions) | CRPS µm | MAE of median µm | D2 − comparator [95 %] |
|---|---|---|---|
| **`flex_heptane.wsp2` D2** | **32.2** | **43.9** | |
| $d^2$-law of the spec | 54.0 | 72.4 | −21.8 [−47.1, −4.9] |
| `flex_heptane.wsp` W0 (zero-shot, 42 cases) | 80.4 | 100.1 | −47.0 [−124.9, −3.2] |

**Burner flames.** CFI-G sealed test: normal flames 60 cases in 2 sessions, inverse flames 93 cases in
3 sessions. Each fitted baseline is a statistical survival model fitted on train. The metric is the
integrated Brier score.

| CFI-G, extinction IBS | Normal | Inverse |
|---|---|---|
| **F2** | **0.0414** | **0.0998** (0.1045 on the 69 cases the fitted baselines answer) |
| baseline set of the spec | 0.1200 | 0.2222 |
| linear (fitted) | 0.0933 | 0.1744 |
| hazard (fitted) | 0.1068 | 0.1645 |
| piecewise (fitted) | 0.2388 | 0.2270 |
| constant (fitted) | 0.2420 | 0.2233 |

**Flame Design test days.** These are scored against the incidence baseline. Normal flames with the
planned stop score **0.061** against 0.432 (83 cases, 3 days). Inverse flames with the planned stop
score **0.002** against 0.677 (60 cases, 1 day). Source: `RESULTS_FINAL.md` step 2.

```bash
python tools/wsp/final/run_final.py preflight --releases tools/wsp/final/releases_final_20261115.json
python tools/wsp/final/run_final.py rehearse all    # reproduces every validation number exactly (README of tools/wsp/final)
```

The final mode ran once on 15 November. Its markers (`runs/final/20261115/markers/`) refuse a second
run by design. The post-hoc scripts of sections 1 to 4 re-score from the final run's local caches
without opening anything new.

## 8. Where a published model scores lower

These are listed for completeness, with the sources in which each appears.

- **Farouk & Dryer (2014).** Their detailed simulation on one FLEX validation droplet, FLEX-035 (their
  choice), has a whole-burn nRMSE of 5.18 % against 13.05 % for D2 (`RESULTS_POINT_METRICS.md`).
- **The SPICE investigators' published smoke-point fit.** It scores 3.16 against 4.18 mm for P3 on the
  test days. That release was built on the same fit, so it is not blind (`RESULTS_FINAL_GEN7.md`).
- **Frolov on the inverse validation flames, no planned stop.** Frolov scores 0.121 against F2's 0.180,
  +0.059 [+0.013, +0.200]. With the stop, F2 is lower (0.109 against 0.121).

## Reproducing everything

1. **Build the binaries.** Build `ignis` and `wsp-dev` from the research repository at `542dad8e`.
   The final run used the binaries with sha256 `b03a35c8…` and `bd6055e0…` (`RESULTS_FINAL.md`).
2. **Rebuild the releases.** Run `python tools/wsp/reassemble.py` and check the ids against
   `configs/models/**/release.json` ([FORMULAS.md](FORMULAS.md), Part 5).
3. **Build the data.** The NASA data come from the public archives through the data pipeline
   ([../DATA_PIPELINE.md](../DATA_PIPELINE.md)). The comparator transcriptions are committed:
   `docs/wsp/research/frolov2023_flames.json`, `irace2023_flames.json`, `kim2023_flames.json`,
   `xi2021_fig2_heptane_trace.json`, `bass_published_burns.json` and
   `farouk_dryer2014_fig5_model.json`.
4. **Run the scripts.** Run the scripts named in each section. Every script prints aggregates only and
   writes its JSON to `runs/final/20261115/`. Per-case rows stay in the local, git-ignored
   `runs/final/20261115/local/`.

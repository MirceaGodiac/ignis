# Point-error metrics of the FLEX releases on the sealed test, and the published droplet studies

15 November 2026, after the final run (PROTOCOL 12.11). A post-hoc look: the frozen releases of
`tools/wsp/final/releases_final_20261115.json` and the forecasts sealed by the final run, scored with
point metrics the evaluator does not report, so the results can be set beside published work.
**Nothing is adopted or refused.**

- Script: `tools/wsp/final/posthoc_point_metrics.py`. Aggregates: `runs/final/20261115/posthoc_point_metrics.json`.
  Per-case rows: `runs/final/20261115/local/posthoc/` (local, not in git).
- Every continuation was opened through `ignis eval reveal --final-test`: 108 reveal-log lines
  (one per test case and cutoff; purpose `reveal`). The other releases reuse those continuations.
- Check: the session-weighted MAE of the median reproduces RESULTS_FINAL to the decimal
  (methanol D3 22.4 µm, d-squared law 43.8; heptane D2 43.9, d-squared law 72.4).

**Metrics**, on the continuation observations inside the forecast window (cutoff < t ≤ the last forecast
time; horizon 10 s; cutoffs 1, 3 and 5 s), using the forecast median:
MAE and RMSE in µm; **nRMSE**: RMSE divided by the mean measured diameter, the metric of Xi, Torero and
Jahn (2021); MAPE. Values are session-weighted means (the mean of session means, as the evaluator's
estimate); case means in brackets.

## FLEX methanol: 64 cases, 9 sessions

| Release | MAE µm | RMSE µm | nRMSE | MAPE |
|---|---|---|---|---|
| **`flex_methanol.wsp2` D3 (adopted)** | **22.4** (27.9) | **41.9** | **1.52 %** (2.09) | **4.71 %** |
| d-squared law of the spec | 43.8 (50.0) | 67.4 | 2.52 % (3.08) | 5.60 % |
| `flex.v3` (MAP, fitted) | 65.1 (66.1) | 91.1 | 3.47 % (3.80) | 6.54 % |
| `flex.v1` (MAP, fitted) | 75.7 (71.2) | 101.7 | 4.04 % (4.13) | 7.12 % |
| `flex.d2_population` (fitted) | 82.9 (82.5) | 107.2 | 4.25 % (4.43) | 7.33 % |
| `flex.d2_prefix` (fitted) | 89.7 (89.0) | 118.4 | 4.41 % (4.68) | 7.43 % |

Paired, D3 minus comparator, 95 % bootstrap over sessions: MAE −21.4 µm [−29.4, −12.8] against the
d-squared law and −42.7 [−58.9, −26.4] against `flex.v3`; nRMSE −1.00 points [−1.50, −0.48] and −1.96
[−2.85, −1.09].

## FLEX n-heptane: 43 cases, 7 sessions

| Release | MAE µm | RMSE µm | nRMSE | MAPE |
|---|---|---|---|---|
| **`flex_heptane.wsp2` D2 (adopted)** | **43.9** (52.7) | **61.3** | **5.41 %** (6.24) | **5.32 %** |
| d-squared law of the spec | 72.4 (80.8) | 96.4 | 6.78 % (7.62) | 5.70 % |

Paired, D2 minus the d-squared law: MAE −28.5 µm [−60.1, −5.2]; nRMSE −1.37 points [−2.73, −0.33].

## Beside the published studies

| Study | What was predicted, and how | Reported | IGNIS on the same metric | Comparable? |
|---|---|---|---|---|
| Xi, Torero & Jahn, *Proc. Combust. Inst.* 38 (2021) 4785, "Data driven forecast of droplet combustion" | FLEX droplets; Spalding's solution plus a numerical gas phase; parameters corrected by data assimilation from the observed part of **one droplet per fuel** | nRMSE (RMSE / mean measurement) 2.46 % n-heptane, 2.2 % n-decane | 5.41 % n-heptane, 1.52 % methanol, over 43 and 64 sealed cases; no decane test | Same metric and data family; different regime: one droplet each, parameters set from that droplet's first 5 s, scored on 5–10 s only, against blind forecasts 10 s ahead from cutoffs 1, 3 and 5 s over 7 and 9 sessions. Not a head-to-head; see the next section for their window. |
| Xu et al., arXiv 2607.16587 (2026), SAM2 measurement | Measuring the flame diameter from images | MAPE 3.1 % against manual measurement, 19,537 images | Forecast MAPE 4.71 % methanol, 5.32 % heptane | Measurement, not forecasting. Useful as a scale only: a 10 s forecast lands within about two points of what image measurement itself achieves. Different quantity (flame vs liquid diameter). |
| Malekjani, Kharaghani & Tsotsas (physics-informed networks, n-heptane) | Droplet **evaporation** without flame | R² > 0.999 interpolation; extrapolation within ±20 % | — | Not comparable (no combustion). The journal given in our notes (CERD 2026) did not match the search result (*Process Safety and Environmental Protection*); check before citing. |
| "Fotia" (Space Apps 2026) | Ridge regression for the burning-rate constant, then a d-squared trajectory | CV R² 0.64; typical K error 0.05 mm²/s | not computed | Different target. The project was not found in a web search on 15 November; get the link before citing. |

## Under the protocol of Xi et al.

Their section 5 (deposited version, UCL Discovery 10106831): the first 5 s of droplet size are assimilated, the
model forecasts 5 to 10 s, and the 2.46 % (n-heptane, initial radius 1.74 mm) is RMSE over 5–10 s divided by
the mean measurement. They do not name the FLEX test. Our cutoff-5 forecasts see the same information (the
prefix up to 5 s, same NASA DropletData time base); scored on 5 < t ≤ 10 s only, with
`tools/wsp/final/posthoc_xi_window.py` on the cached continuations (no new reveal):

| Release | Runs (sessions) | nRMSE median | case mean | session-weighted | runs ≤ 2.46 % |
|---|---|---|---|---|---|
| heptane `flex_heptane.wsp2` D2 (adopted) | 12 (7) | 1.08 % | 2.40 % | 3.20 % | 11 / 12 |
| heptane d-squared law | 12 (7) | 1.11 % | 2.45 % | 3.22 % | 11 / 12 |
| methanol `flex_methanol.wsp2` D3 (adopted) | 20 (9) | 0.30 % | 2.15 % | 1.40 % | 19 / 20 |
| methanol d-squared law | 20 (9) | 0.63 % | 2.40 % | 1.72 % | 17 / 20 |

One heptane run (`PSI-69:H05H103:2011-10-05`, 17.5 %) carries the mean; the other eleven are 0.14–2.12 %.
**Reading.** On their window our typical heptane run is under their single-droplet figure, but so is the
d-squared law: 5–10 s after a 5 s prefix is the steady-shrinking part of the burn and separates models
little. It shows we match their error on their task across many droplets; it does not show we beat their
method. The like-for-like number still needs their droplet identified (d0 ≈ 3.48 mm; the FLEX table has
about a dozen heptane runs within 0.1 mm, so diameter alone does not identify it).

## Head-to-head with their method, rebuilt (15 November 2026, post-hoc)

**The rebuild.** `tools/wsp/final/posthoc_xi_comparator.py` implements their droplet forecast from their
Eqs. 2 and 8: R(t)² = R0² − K̂t, R0 the initial radius, K̂ the one droplet-phase parameter set by least squares
on the prefix after thermal swelling. Their CFD and second parameter set only the flame radius, so they do not
enter. It reads only what every release got at the cutoff, and scores on the cached continuations (no reveal).

**Their droplet is FLEX-094** (`PSI-69:C08H101:2010-06-15`, d0 3.53 mm, 71 kPa, 24 % O2, **train**). Their
Fig. 2 heptane curve was traced from the deposited PDF (`docs/wsp/research/xi2021_fig2_heptane_trace.json`);
the traced prediction against the traced experiment gives 2.42 % (they report 2.46 %), so the trace is sound.
Matched against every FLEX heptane run, FLEX-094 fits with no time shift (22.6 µm over 0–10 s; next
FLEX-238 at 28.6 µm needing a 0.56 s shift). Exposure: the match read the 0–10 s trajectories of all heptane
runs, including validation runs, for identification only; nothing was tuned on it.

**On their droplet**, cutoff 5 s, scored 5–10 s (nRMSE): rebuilt Xi **2.58 %** (they report 2.46 %, so the
rebuild is faithful); `flex_heptane.wsp2` D2 **0.19 %**; zero-shot W0 **0.65 %**; d-squared law 0.92 %.
From earlier cutoffs, scored to cutoff + 10 s: cutoff 3 s Xi 4.75 %, D2 0.71 %; cutoff 1 s Xi 9.32 %, D2 0.24 %.
D2 was typed after its author could read train, which holds FLEX-094; W0 was typed before any data.

**On the sealed test, same cases for every model** (nRMSE, session-weighted; paired D2 minus Xi, 95 %
bootstrap over sessions):

| Task | Cases | Xi rebuilt | D2 / D3 adopted | d-squared law | D2 − Xi |
|---|---|---|---|---|---|
| Heptane, their task (cut 5 s, 5–10 s) | 12 | 16.05 % (median 2.02) | 3.20 % (median 1.08) | 3.22 % | −12.84 [−36.44, −0.58] |
| same, without `H05H103` | 11 | 1.89 % | 0.82 % | 1.02 % | −1.08 [−1.76, −0.35] |
| Heptane, evaluator window (cuts 1, 3, 5 s; 10 s) | 43 | 17.16 % | 5.41 % | 6.78 % | −11.75 [−29.76, −1.62] |
| same, without `H05H103` | 40 | 6.25 % | 3.44 % | 5.10 % | −2.81 [−4.51, −1.14] |
| Methanol, their task | 20 | 3.27 % | 1.40 % | 1.72 % | −1.87 [−2.84, −1.10] |
| Methanol, evaluator window | 64 | 6.95 % | 1.52 % | 2.52 % | −5.43 [−7.62, −3.77] |

`H05H103` is a 1.49 mm droplet for which their law, R0 fixed, predicts burnout before 5 s while it is still
measured (Xi 101 %, D2 17.5 %); it is shown with and without. Every interval excludes zero either way.
**Reading.** Rebuilt from their equations and checked against their own published number on their own
droplet, the Xi et al. droplet forecaster has a higher error than the adopted IGNIS releases on their task
and more so from earlier cutoffs, on both fuels. Limits: a rebuild, not their code; droplet size only (their
flame-radius forecast is not compared); post-hoc on a spent test.

## Farouk & Dryer (2014), detailed simulation, on FLEX-035 (15 November 2026, post-hoc)

Their predicted droplet curve (Combust. Flame 161:565; sphero-symmetric transient model, detailed low- and
high-temperature chemistry, radiation) is shown over FLEX data in Dietrich et al. 2014, *Microgravity Sci.
Technol.*, Fig. 5 (n-heptane, D0 3.9 mm, air, 1 atm). Curve and FLEX points were extracted from the PDF's
vector paths (`docs/wsp/research/farouk_dryer2014_fig5_model.json`). The FLEX points match
**FLEX-035** (`PSI-69:181F001:2009-11-27`, d0 3.85 mm, 102.4 kPa, air, **validation**) with its own d0 and no
time shift: RMS 0.0025 in (D/D0)², next run 0.019. Exposure: the 0–38 s trajectory of this validation run
was read and studied in detail (diagnosis below). Scored with `tools/wsp/final/posthoc_published_curve.py`
and 40 s horizons for the releases (nRMSE):

| Window | Farouk & Dryer (initial conditions only) | D2 (adopted) | W0 | d-squared law | Xi rebuilt |
|---|---|---|---|---|---|
| 1–11 s (cutoff 1) | **0.62 %** | 0.86 % | 1.32 % | 2.08 % | 1.57 % |
| 3–13 s (cutoff 3) | 0.84 % | 1.33 % | 2.17 % | **0.36 %** | 1.26 % |
| 5–15 s (cutoff 5) | 1.05 % | 1.89 % | 0.49 % | **0.47 %** | 0.92 % |
| whole burn from cutoff 1 (to 38.8 s) | **5.18 %** | 13.05 % | 12.46 % | 28.65 % | 14.52 % |

**Farouk & Dryer beat the adopted heptane release on this run**, with less information. Diagnosis: the hot
flame ends radiatively near 7–9 s (their curve shows flame collapse at about 6.4 s) and the droplet burns on in
a cool flame at K ≈ 0.37–0.40 mm²/s until it stalls at 1.27 mm. D2 has these states, but spreads the
hot-flame end (median about 10.5 s, 20 % still hot at 18 s) and its median forecast runs at an effective
K ≈ 0.45 mm²/s from 10 s on; the error grows linearly from there. D2 gets the stall diameter (1.32 mm), which
their model misses (about 0.6 mm). One run, chosen by them; tuning in their paper unknown (paywalled).

**D3 on FLEX-035 (exposed run, after diagnosis; D3 not adopted under PROTOCOL 13.7).** nRMSE, Farouk & Dryer /
D3 / D2: 1–11 s 0.62 / 0.76 / 0.86 %; 3–13 s 0.84 / 1.12 / 1.33 %; 5–15 s 1.05 / 1.49 / 1.89 %; whole burn from
cutoff 1 s 5.18 / 11.48 / 13.05 %. D3's slower cool-flame rate closes part of the gap; the detailed simulation
still wins on every window. The remaining gap is the timing of the hot-flame end and a rate sag after an early
flame loss, which train supports in five sessions but needs a new forward (`RESULTS_HEPTANE_D3.md`).

**D4 on FLEX-035 (`flex_heptane.wsp10`, sha256:f3ccc122…b164;; exposed run).** Farouk & Dryer /
D4: 1–11 s 0.62 / 0.70 %; 3–13 s 0.84 / 0.99 %; 5–15 s 1.05 / 1.21 %; whole burn from 1 s 5.18 / 11.36 %. The
transition stage helps a little; the hot-flame end is unchanged (its radiative clock failed the train check), so
the simulation still wins every window. Its fair judge is the blind test of PROTOCOL 13.9.

Studies not comparable from open sources (15 November 2026): Tyurenkova 2019 (two-regime model; closed),
Cuoci et al. 2017 and 2024 (repository behind a Cloudflare check; OpenSMOKE++ solver needs academic
registration), Farouk & Dryer 2012 and Paczko et al. 2014 (closed; open NASA papers compare models only on
extinction diameter and burning rate; Paczko predicts no trajectory).

**What this supports.** On the sealed FLEX test, the adopted releases have the lowest MAE, RMSE, nRMSE
and MAPE of every comparator on both fuels, with session intervals that exclude zero against the
d-squared law and the fitted models. Against Xi et al. it now supports, with its limits: "on their own droplet and on every sealed test
droplet, under their protocol and from earlier cutoffs, lower error than their method rebuilt from their
paper" (section above). Comparing our 5.41 % with their 2.46 % is not like for like: the first is the harder
evaluator window, the second their 5–10 s task, where the same release scores 0.82–3.20 %.

**Not done.** Their n-decane droplet (radius 1.98 mm): IGNIS has no decane model. Their flame-radius
forecast: IGNIS releases forecast the liquid diameter only.

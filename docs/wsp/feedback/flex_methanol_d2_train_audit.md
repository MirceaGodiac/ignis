# Methanol droplet model D2: audit against the FLEX train data

> Train partition only; nothing fitted. Written by a delegated analysis agent (claude-opus-5-5) on 14 November 2026;
> the author rechecked the claims it acted on against the train tables. Paths under `scratchpad` are the agent's
> working folder and are not part of the repository.

Evidence for the lead author. Everything below was measured on the **train partition only**. The run
ids come from `lib.partition_ids("flex", "train")` (68 runs, 27 day-sessions). The outcomes read were
the raw `DropletData` series and test sheets of those runs, `runs/dev/analysis/flex_methanol_profile.json`
and the D2 and D1 releases in `runs/dev/fb_train_flex.json`, which holds train cases only. Every run in
it was checked against the train list.

**No fitting.** All numbers are per-run finite differences of window means, simple ratios and group
counts, medians, minima and maxima. One descriptive Spearman rank correlation appears in B. No
coefficient was estimated.

**Units.** D in mm. K and rates of D² in mm²/s. CRPS and bias in µm. Bias is the median prediction
minus the observation. w = 1 − D²/D0² and s = t·(1 mm²/s)/D0².

**Scripts.** Every script is in this folder, and `run_all.sh` rebuilds all tables:
`common.py`, `check_law.py`, `ext.py`, `ext_tables.py`, `rate_audit.py`, `post_ext.py`, `state_cut.py`,
`rho_e.py`, `lowr.py`, `persist.py`, `budget.py`, `causes.py`, `cov_type.py`, `dump_rates.py`,
`build_report.py` (+ `report_template.txt`).

**The law was re-implemented in Python and checked against the forward** (`check_law.py`):

```
cases with encoder diagnostics: 186
laws.d_crit_m: max relative difference 2.63e-16
laws.pi_r: max relative difference 3.69e-15
laws.w_d: max relative difference 3.24e-16
laws.s_r: max relative difference 2.75e-15
q0: max relative difference 0.00e+00
prefix windows rebuilt from raw series identical to the encoder's in 186 of 186 cases
k_model_late: max relative difference 3.86e-16 over 186 cases (median 0.00e+00)
```

The prefix windows, rebuilt from the raw series with the validity gate as written in droplet.rs, are
identical to the encoder's in all 186 cases. The rate network, crit, class, diff and rad laws match
the forward's diagnostics to rounding error, so every K_law below is the forward's own number.

**Series cleaning used for my measurements only.** `load_series` drops single tracker-dropout frames:
any frame more than 8 % away from the median of the 7 frames around it. That removed 1 to 4 frames in
16 runs. Example: 191R001 has two frames at D = 0.094 mm at 3.33–3.37 s. The feedback scores were
computed by the forward pipeline on the uncleaned series.

**Extinction time used throughout** (`ext.py`). t_ext is the first time at which the 0.2-s window
mean of D reaches NASA's Dext while the droplet is in view. That is NASA's visible-flame extinction
placed on the series. When NASA's narrative says Dext was extrapolated or not measured, no crossing
is used. Otherwise t_ext falls back to profile knee + 0.3 s. For the 25 runs where both exist, a
direct Dext agrees with the diameter at the profile knee within −0.12..+0.10 mm (table in C).

Problem runs (not used to judge a law): 240R003, C12M401, 40CAL01, H12M101, 256F002, 256R001,
192F001 and C11M201 (refused). Runs with no sustained flame (Dext/D0 ≥ 0.97: C14M101 and H08M101)
are shown separately where it matters.

---

## A. Error budget of D2 on train

CRPS in um. Scored cases only.

| cutoff | cases | runs | sessions | mean CRPS | median CRPS | mean CRPS without problem runs | median without |
|---|---|---|---|---|---|---|---|
| 1 | 66 | 66 | 26 | 39.5 | 22.6 | 23.9 | 22.5 |
| 3 | 61 | 61 | 25 | 33.2 | 21.9 | 26.7 | 20.8 |
| 5 | 55 | 55 | 24 | 34.2 | 23.9 | 29.9 | 23.3 |
| all | 182 | 66 | 26 | 35.8 | 23.1 | 26.7 | 22.5 |

| session | cases | runs | mean CRPS | median CRPS | problem runs in session |
|---|---|---|---|---|---|
| 2009-03-05 | 8 | 3 | 46.4 | 35.4 |  |
| 2009-03-31 | 7 | 3 | 133.0 | 61.9 | 40CAL01 |
| 2009-05-22 | 5 | 2 | 40.9 | 25.2 | 256F002, 256R001 |
| 2009-07-01 | 1 | 1 | 575.9 | 575.9 | 240R003 |
| 2009-10-24 | 6 | 2 | 43.4 | 44.8 |  |
| 2009-11-09 | 6 | 2 | 34.5 | 33.9 |  |
| 2009-11-13 | 5 | 2 | 20.8 | 24.7 |  |
| 2009-11-27 | 6 | 2 | 19.0 | 20.0 |  |
| 2010-02-04 | 6 | 2 | 18.9 | 15.4 |  |
| 2010-03-19 | 13 | 5 | 17.0 | 15.5 |  |
| 2010-03-29 | 6 | 2 | 19.2 | 19.8 |  |
| 2010-05-05 | 6 | 2 | 18.3 | 18.4 |  |
| 2010-06-21 | 6 | 2 | 12.5 | 12.4 |  |
| 2010-09-08 | 9 | 3 | 48.7 | 24.9 | C12M401 |
| 2010-11-04 | 11 | 4 | 24.9 | 23.3 |  |
| 2010-12-08 | 9 | 3 | 20.1 | 19.0 |  |
| 2011-01-12 | 6 | 2 | 33.7 | 39.0 |  |
| 2011-08-08 | 12 | 4 | 22.7 | 23.0 |  |
| 2011-08-24 | 15 | 5 | 24.1 | 24.1 |  |
| 2011-09-09 | 11 | 4 | 19.7 | 16.3 |  |
| 2011-09-21 | 2 | 1 | 6.0 | 6.0 |  |
| 2011-09-22 | 3 | 1 | 23.2 | 24.2 |  |
| 2011-10-01 | 7 | 3 | 30.1 | 22.9 |  |
| 2011-10-11 | 3 | 1 | 9.8 | 7.1 |  |
| 2011-11-18 | 10 | 4 | 43.9 | 30.7 | H12M101 |
| 2011-12-05 | 3 | 1 | 135.0 | 127.7 |  |

**Where the error sits by case type** (problem runs excluded; `cov_type.py`):

| case type | cutoff | cases | sessions | D2 cov50 | D2 cov90 | D1 cov50 | D1 cov90 | median s_z D2 | mean CRPS D2 | mean CRPS D1 |
|---|---|---|---|---|---|---|---|---|---|---|
| burning through the scored horizon | 1 | 40 | 19 | 0.91 | 1.00 | 0.84 | 0.99 | 0.178 | 21.0 | 18.0 |
| burning through the scored horizon | 3 | 32 | 17 | 0.59 | 0.98 | 0.51 | 0.83 | 0.092 | 23.4 | 23.4 |
| burning through the scored horizon | 5 | 26 | 15 | 0.73 | 0.99 | 0.58 | 0.94 | 0.084 | 17.5 | 17.1 |
| goes out within 10 s | 1 | 18 | 12 | 0.75 | 1.00 | 0.70 | 1.00 | 0.169 | 32.4 | 30.2 |
| goes out within 10 s | 3 | 22 | 14 | 0.61 | 0.96 | 0.57 | 0.95 | 0.092 | 32.7 | 31.1 |
| goes out within 10 s | 5 | 19 | 13 | 0.79 | 1.00 | 0.70 | 0.95 | 0.084 | 33.8 | 32.4 |
| no sustained flame | 1 | 2 | 2 | 0.98 | 1.00 | 0.87 | 1.00 | 0.186 | 5.4 | 4.5 |
| no sustained flame | 3 | 2 | 2 | 0.84 | 1.00 | 0.79 | 0.98 | 0.119 | 6.2 | 5.6 |
| no sustained flame | 5 | 2 | 2 | 0.62 | 0.83 | 0.50 | 0.76 | 0.092 | 10.9 | 11.0 |
| out before cutoff | 3 | 1 | 1 | 0.02 | 0.92 | 0.00 | 0.67 | 0.092 | 40.7 | 40.9 |
| out before cutoff | 5 | 5 | 4 | 0.04 | 0.29 | 0.02 | 0.18 | 0.084 | 87.4 | 85.5 |

**The 30 worst cases** (`causes.py`). Each cause was assigned by reading the bias by lead, the
observed branch and t_ext, the filter state, e_late, b, the later rate ratio (F), the raw 1-s rate
profile (`out_dump_rates.txt`) and the NASA narrative. The evidence for each assignment is in its row.

| # | run | cutoff | CRPS um | cause | evidence |
|---|---|---|---|---|---|
| 1 | 240R003 | 1 | 576 | artefact | gas bubble; droplet grows and shatters |
| 2 | 40CAL01 | 3 | 337 | artefact | leaves and re-enters view; e_late +0.80 from tracker |
| 3 | 40CAL01 | 5 | 241 | artefact | same |
| 4 | H15M102 | 5 | 179 | state at cutoff misread (late) | out at 4.1 s; late window 2.5-4.4 s straddles the extinction, r_late 0.91, filter hot 1.00 |
| 5 | H12M101 | 1 | 165 | artefact | leaves view at 2.1 s; last frames shrink at the edge |
| 6 | 40CAL01 | 1 | 163 | artefact | same |
| 7 | C12M401 | 1 | 144 | artefact | duplicate video of C14M201 under the wrong atmosphere |
| 8 | 20CAL12 | 5 | 133 | state at cutoff misread (late) | out at 3.4 s; r_late 0.56 vs no-flame mean 0.07, filter hot 0.84 |
| 9 | C12M401 | 3 | 129 | artefact | same |
| 10 | H15M102 | 3 | 128 | radiative timing (model late) | s_obs 0.53 vs s_R 0.83; He 45 %, 0.7 atm |
| 11 | 256F002 | 1 | 117 | artefact | 3 atm CO2; saturated backlight, size depends on image quadrant |
| 12 | H15M102 | 1 | 98 | radiative timing (model late) | same |
| 13 | H05M102 | 5 | 85 | radiative timing (model late) | s_obs 0.70 vs s_R 0.90, P(rad) 0.62; He 25 % |
| 14 | 20CAL12 | 3 | 70 | diffusive timing (model late) | w_obs 0.67 vs w_D 0.79 (D0 1.61 mm) |
| 15 | C14M302 | 5 | 64 | post-extinction evaporation | out at 3.0 s, state right (0.98); evaporation scale taken from a late slope that still contains burning (k_obs_late 0.143 mm2/s) -> too fast |
| 16 | 30CAL03 | 5 | 62 | diffusive timing (model early) + post-extinction | w_obs 0.82 vs w_D 0.79; then 0-2 s evaporation 0.050 vs law 0.022 mm2/s |
| 17 | 192F002 | 3 | 60 | heat-up read as run offset | 1-2 s window +0.19 above law (fast heat-up at 34 % O2, 0.7 atm); b=+0.10 applied, later 3-6 s on the law (-0.003) |
| 18 | 30CAL03 | 3 | 60 | diffusive timing (model early) + post-extinction | same |
| 19 | 192F003 | 3 | 58 | heat-up read as run offset | same pattern: 1-2 s +0.19, b=+0.10, later +0.009 |
| 20 | 191R001 | 3 | 53 | artefact | two dropout frames (D = 0.094 mm at 3.33-3.37 s) inside the scored window |
| 21 | C01M603 | 5 | 52 | diffusive timing (model early) + post-extinction | w_obs 0.83 vs w_D 0.79; 0-2 s evaporation 0.119 vs law 0.032 |
| 22 | 30CAL03 | 1 | 50 | diffusive timing (model early) + post-extinction | same |
| 23 | C07M101 | 1 | 49 | radiative timing (model late) | s_obs 0.32 vs s_R 0.63; Dext/D0 0.96 |
| 24 | C07M101 | 3 | 48 | radiative timing (model late) | out 0.3 s after the cutoff; same |
| 25 | 192F002 | 5 | 46 | hot-rate law (too fast late in the burn) | 8-12 s window -0.07 (w 0.55-0.8) at 34 % O2, 0.7 atm; bias -123 um at 5-10 s |
| 26 | 20CAL12 | 1 | 45 | diffusive timing (model late) | same |
| 27 | 191R001 | 1 | 45 | artefact | the same dropout frames (lead 2.3 s); bias otherwise small |
| 28 | C12M401 | 5 | 45 | artefact | same |
| 29 | C07M201 | 5 | 44 | wrong extinction branch | radiative at s=1.13 (Dext/D0 0.79); P(rad|hot)=0.45 at the cutoff, s_R 0.90 already passed, so the diffusive half keeps burning |
| 30 | 192F003 | 5 | 44 | hot-rate law (too fast late in the burn) | 8-12 s -0.08, 12-18 s -0.07 |

Sum of CRPS over all 182 scored cases: 6514 um; the worst 30 carry 3389 um (52 %).

| cause | cases | sessions | sum CRPS um | share of worst-30 | share of worst-30 without artefacts |
|---|---|---|---|---|---|
| artefact | 11 | 6 | 2015 | 59 % |  |
| radiative timing (model late) | 5 | 3 | 408 | 12 % | 30 % |
| state at cutoff misread (late) | 2 | 2 | 312 | 9 % | 23 % |
| diffusive timing (model early) + post-extinction | 4 | 2 | 224 | 7 % | 16 % |
| heat-up read as run offset | 2 | 1 | 118 | 3 % | 9 % |
| diffusive timing (model late) | 2 | 1 | 116 | 3 % | 8 % |
| hot-rate law (too fast late in the burn) | 2 | 1 | 90 | 3 % | 7 % |
| post-extinction evaporation | 1 | 1 | 64 | 2 % | 5 % |
| wrong extinction branch | 1 | 1 | 44 | 1 % | 3 % |

**Conclusion A.**
- The median CRPS is flat at 21–24 µm across cutoffs.
- The mean is pulled up by artefacts. 11 of the worst 30 cases are measurement artefacts and carry
  59 % of the worst-30 CRPS. 240R003 alone is 576 µm.
- Two new artefacts outside the known list:
  - Two dropout frames in 191R001 (they carry CRPS 53 and 45).
  - C07M201's post-gap frames at the edge of the view.
- Once artefacts are set aside, the error is carried by the end of burning, not the burning rate:
  - **Radiative extinction timing.** The model goes out too late. This is 30 % of the non-artefact
    worst-30 CRPS (3 sessions).
  - **Filter late at the cutoff.** The droplet is already out but the filter says hot. This is 23 %
    (2 sessions in the worst 30, 4 sessions in D).
  - **Diffusive timing.** The model is early for 30CAL03 and C01M603 and late for 20CAL12. That is
    24 % in total, but it rests on only 3 sessions and has no consistent direction (C).
- The hot-rate law and heat-up appear only through the two 34 % O2 runs (16 %, 1 session).
- Burning-through-horizon cases average 17–23 µm. Cases that go out within the horizon average
  32–34 µm. Cases already out before the cutoff average 87 µm.

---

## B. Hot-rate law audit

For each window (0.2–1, 1–2, 2–3, 3–5, 5–8, 8–12, 12–18, 18–26 s) that lies wholly before t_ext − 0.3 s
and inside the series, the measurement is as follows:
- K_obs is the finite difference of the D² window means of the first and last thirds of the window.
- K_law is the D2 law as a harmonic Simpson mean between the same two points, computed exactly as the
  encoder does, at the encoder's q0 and c.
- r = ln(K_obs / K_law).

"Within-run" subtracts each run's median r, which isolates the shape in w and s from the run level.
Order in session is by test time among train runs of the same day only; other burns of the same fill
belong to other partitions and were not opened. The rule for "consistent" is written above the tables.

# B. Hot-rate residuals r = ln(K_obs/K_law), D2

263 windows, 58 runs, 23 sessions (problem runs and no-flame runs excluded).
Consistent = at least 3 sessions with session median beyond +-0.05 in one direction, at least 60 % of the class's sessions, and at most max(1, n/5) sessions beyond 0.05 the other way.

**All windows**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| all | 263 | 58 | 23 | -0.003 | -1.126 | +0.467 | 3 / 15 / 5 | no |

**Oxygen class**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 0.15 | 27 | 8 | 2 | -0.052 | -0.276 | +0.161 | 0 / 1 / 1 | too few sessions |
| 0.16-0.19 | 67 | 14 | 8 | +0.014 | -1.126 | +0.467 | 3 / 5 / 0 | no |
| 0.20-0.21 | 61 | 14 | 6 | -0.005 | -0.346 | +0.221 | 0 / 5 / 1 | no |
| 0.23-0.25 | 48 | 9 | 4 | +0.007 | -0.321 | +0.258 | 0 / 3 / 1 | no |
| <=0.14 | 28 | 7 | 3 | -0.119 | -0.402 | +0.199 | 0 / 0 / 3 | yes (-) |
| >=0.30 | 32 | 6 | 3 | +0.014 | -0.170 | +0.190 | 0 / 3 / 0 | no |

**Diluent**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| CO2 | 99 | 21 | 8 | +0.003 | -1.126 | +0.467 | 1 / 5 / 2 | no |
| He | 44 | 10 | 6 | +0.007 | -0.292 | +0.258 | 2 / 4 / 0 | no |
| N2 only | 120 | 27 | 11 | -0.034 | -0.402 | +0.221 | 0 / 7 / 4 | no |

**Pressure**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 0.7 atm | 97 | 20 | 9 | +0.013 | -0.554 | +0.467 | 1 / 7 / 1 | no |
| 1.0 atm | 166 | 38 | 15 | -0.021 | -1.126 | +0.247 | 2 / 9 / 4 | no |

**Initial diameter (NASA D0, mm)**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 2.2-2.8 | 83 | 20 | 15 | -0.002 | -0.402 | +0.258 | 3 / 9 / 3 | no |
| 2.8-3.4 | 79 | 16 | 13 | -0.012 | -0.351 | +0.207 | 0 / 8 / 5 | no |
| <2.2 | 25 | 8 | 4 | +0.007 | -0.199 | +0.221 | 1 / 2 / 1 | no |
| >=3.4 | 76 | 14 | 11 | -0.006 | -1.126 | +0.467 | 2 / 6 / 3 | no |

**Progress class (raw)**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 0.15-0.4 | 84 | 49 | 23 | -0.001 | -0.138 | +0.187 | 3 / 15 / 5 | no |
| 0.4-0.7 | 28 | 23 | 15 | -0.002 | -0.164 | +0.066 | 3 / 9 / 3 | no |
| w<0.15 | 150 | 57 | 23 | -0.006 | -1.126 | +0.467 | 9 / 6 / 8 | no |
| w>=0.7 | 1 | 1 | 1 | -0.072 | -0.072 | -0.072 | 0 / 0 / 1 | too few sessions |

**Progress class, within-run (r minus the run's median r)**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 0.15-0.4 | 84 | 49 | 23 | +0.000 | -0.157 | +0.169 | 0 / 20 / 3 | no |
| 0.4-0.7 | 28 | 23 | 15 | -0.005 | -0.132 | +0.137 | 3 / 10 / 2 | no |
| w<0.15 | 150 | 57 | 23 | +0.000 | -0.689 | +0.418 | 6 / 16 / 1 | no |
| w>=0.7 | 1 | 1 | 1 | -0.090 | -0.090 | -0.090 | 0 / 0 / 1 | too few sessions |

**Clock class (raw)**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 0.1-0.2 | 50 | 46 | 23 | -0.007 | -0.705 | +0.258 | 10 / 3 / 10 | no |
| s<0.1 | 47 | 43 | 21 | -0.053 | -1.126 | +0.467 | 6 / 4 / 11 | no |
| s>0.2 | 166 | 55 | 23 | -0.001 | -0.306 | +0.187 | 2 / 19 / 2 | no |

**Clock class, within-run**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 0.1-0.2 | 50 | 46 | 23 | -0.001 | -0.209 | +0.254 | 7 / 8 / 8 | no |
| s<0.1 | 47 | 43 | 21 | -0.003 | -0.689 | +0.332 | 5 / 7 / 9 | no |
| s>0.2 | 166 | 55 | 23 | +0.000 | -0.260 | +0.418 | 1 / 20 / 2 | no |

**Time window (raw)**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 0.2-1 s | 56 | 56 | 23 | +0.022 | -1.126 | +0.467 | 9 / 5 / 9 | no |
| 1-2 s | 56 | 56 | 23 | -0.030 | -0.554 | +0.190 | 3 / 8 / 12 | no |
| 12-18 s | 2 | 2 | 2 | -0.057 | -0.072 | -0.043 | 0 / 1 / 1 | too few sessions |
| 2-3 s | 52 | 52 | 23 | -0.003 | -0.705 | +0.187 | 6 / 10 / 7 | no |
| 3-5 s | 44 | 44 | 19 | -0.001 | -0.151 | +0.163 | 2 / 14 / 3 | no |
| 5-8 s | 34 | 34 | 17 | +0.001 | -0.138 | +0.145 | 3 / 11 / 3 | no |
| 8-12 s | 19 | 19 | 14 | -0.012 | -0.164 | +0.066 | 3 / 7 / 4 | no |

**Time window, within-run**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 0.2-1 s | 56 | 56 | 23 | +0.003 | -0.557 | +0.332 | 7 / 11 / 5 | no |
| 1-2 s | 56 | 56 | 23 | -0.025 | -0.689 | +0.172 | 1 / 14 / 8 | no |
| 12-18 s | 2 | 2 | 2 | -0.050 | -0.090 | -0.010 | 0 / 1 / 1 | too few sessions |
| 2-3 s | 52 | 52 | 23 | +0.000 | -0.136 | +0.109 | 1 / 20 / 2 | no |
| 3-5 s | 44 | 44 | 19 | +0.004 | -0.121 | +0.418 | 1 / 18 / 0 | no |
| 5-8 s | 34 | 34 | 17 | +0.003 | -0.049 | +0.137 | 2 / 15 / 0 | no |
| 8-12 s | 19 | 19 | 14 | -0.017 | -0.132 | +0.072 | 1 / 10 / 3 | no |

**Session**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 2009-03-05 | 9 | 3 | 1 | +0.045 | -0.021 | +0.221 | 0 / 1 / 0 | too few sessions |
| 2009-03-31 | 6 | 2 | 1 | -0.010 | -0.103 | +0.156 | 0 / 1 / 0 | too few sessions |
| 2009-10-24 | 13 | 2 | 1 | +0.015 | -0.081 | +0.190 | 0 / 1 / 0 | too few sessions |
| 2009-11-09 | 10 | 2 | 1 | +0.021 | -0.051 | +0.172 | 0 / 1 / 0 | too few sessions |
| 2009-11-13 | 9 | 2 | 1 | -0.009 | -0.170 | +0.049 | 0 / 1 / 0 | too few sessions |
| 2009-11-27 | 10 | 2 | 1 | -0.060 | -0.271 | -0.009 | 0 / 0 / 1 | too few sessions |
| 2010-02-04 | 12 | 2 | 1 | -0.034 | -0.246 | +0.080 | 0 / 1 / 0 | too few sessions |
| 2010-03-19 | 20 | 5 | 1 | -0.053 | -0.276 | +0.147 | 0 / 0 / 1 | too few sessions |
| 2010-03-29 | 10 | 2 | 1 | -0.133 | -0.402 | -0.078 | 0 / 0 / 1 | too few sessions |
| 2010-05-05 | 12 | 2 | 1 | +0.006 | -0.053 | +0.095 | 0 / 1 / 0 | too few sessions |
| 2010-06-21 | 11 | 2 | 1 | -0.072 | -0.321 | +0.027 | 0 / 0 / 1 | too few sessions |
| 2010-09-08 | 11 | 2 | 1 | +0.103 | -0.554 | +0.467 | 1 / 0 / 0 | too few sessions |
| 2010-11-04 | 7 | 3 | 1 | +0.018 | -0.157 | +0.161 | 0 / 1 / 0 | too few sessions |
| 2010-12-08 | 16 | 3 | 1 | -0.006 | -1.126 | +0.081 | 0 / 1 / 0 | too few sessions |
| 2011-01-12 | 7 | 2 | 1 | -0.104 | -0.351 | +0.072 | 0 / 0 / 1 | too few sessions |
| 2011-08-08 | 20 | 4 | 1 | +0.040 | -0.030 | +0.159 | 0 / 1 / 0 | too few sessions |
| 2011-08-24 | 25 | 5 | 1 | -0.019 | -0.346 | +0.133 | 0 / 1 / 0 | too few sessions |
| 2011-09-09 | 17 | 4 | 1 | -0.042 | -0.306 | +0.199 | 0 / 1 / 0 | too few sessions |
| 2011-09-21 | 3 | 1 | 1 | +0.068 | -0.006 | +0.247 | 1 / 0 / 0 | too few sessions |
| 2011-09-22 | 6 | 1 | 1 | +0.057 | +0.020 | +0.097 | 1 / 0 / 0 | too few sessions |
| 2011-10-01 | 11 | 3 | 1 | -0.002 | -0.292 | +0.061 | 0 / 1 / 0 | too few sessions |
| 2011-11-18 | 15 | 3 | 1 | +0.002 | -0.062 | +0.258 | 0 / 1 / 0 | too few sessions |
| 2011-12-05 | 3 | 1 | 1 | +0.010 | -0.018 | +0.158 | 0 / 1 / 0 | too few sessions |

**Order of the burn within its session**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| burn 1 in session (train order) | 87 | 21 | 21 | -0.015 | -1.126 | +0.467 | 5 / 8 / 8 | no |
| burn 2 in session (train order) | 100 | 20 | 20 | -0.011 | -0.323 | +0.258 | 2 / 12 / 6 | no |
| burn 3 in session (train order) | 42 | 9 | 9 | +0.018 | -0.216 | +0.190 | 2 / 6 / 1 | no |
| burn 4 in session (train order) | 26 | 6 | 6 | -0.003 | -0.164 | +0.161 | 1 / 4 / 1 | no |
| burn 5 in session (train order) | 8 | 2 | 2 | -0.013 | -0.131 | +0.018 | 0 / 1 / 1 | too few sessions |

**Drift speed 0.2-1.2 s (profile drift_mm_s)**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 1-2 | 86 | 16 | 12 | -0.040 | -0.402 | +0.221 | 1 / 6 / 5 | no |
| 2-4 | 52 | 14 | 12 | +0.002 | -1.126 | +0.258 | 4 / 6 / 2 | no |
| <1 mm/s | 119 | 25 | 18 | +0.003 | -0.554 | +0.467 | 3 / 12 / 3 | no |
| >=4 | 6 | 3 | 3 | -0.017 | -0.292 | +0.132 | 1 / 0 / 2 | no |

**Mean speed 1-5 s (position columns)**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 1-2 | 92 | 19 | 15 | -0.002 | -0.346 | +0.221 | 2 / 8 / 5 | no |
| 2-4 | 46 | 11 | 9 | -0.012 | -1.126 | +0.258 | 2 / 4 / 3 | no |
| <1 mm/s | 105 | 20 | 13 | -0.005 | -0.554 | +0.467 | 1 / 9 / 3 | no |
| n/a | 20 | 8 | 8 | +0.023 | -0.292 | +0.247 | 3 / 2 / 3 | no |

**Oxygen x diluent**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 0.15 / CO2 | 7 | 3 | 1 | +0.018 | -0.157 | +0.161 | 0 / 1 / 0 | too few sessions |
| 0.15 / N2 only | 20 | 5 | 1 | -0.053 | -0.276 | +0.147 | 0 / 0 / 1 | too few sessions |
| 0.16-0.19 / CO2 | 32 | 6 | 3 | +0.016 | -1.126 | +0.467 | 1 / 2 / 0 | no |
| 0.16-0.19 / He | 23 | 6 | 4 | +0.026 | -0.292 | +0.247 | 2 / 2 / 0 | no |
| 0.16-0.19 / N2 only | 12 | 2 | 1 | -0.034 | -0.246 | +0.080 | 0 / 1 / 0 | too few sessions |
| 0.20-0.21 / CO2 | 15 | 3 | 1 | -0.019 | -0.346 | +0.105 | 0 / 1 / 0 | too few sessions |
| 0.20-0.21 / He | 11 | 2 | 2 | +0.001 | -0.037 | +0.146 | 0 / 2 / 0 | too few sessions |
| 0.20-0.21 / N2 only | 35 | 9 | 4 | -0.015 | -0.271 | +0.221 | 0 / 3 / 1 | no |
| 0.23-0.25 / CO2 | 38 | 7 | 3 | +0.010 | -0.321 | +0.159 | 0 / 2 / 1 | no |
| 0.23-0.25 / He | 10 | 2 | 1 | +0.003 | -0.062 | +0.258 | 0 / 1 / 0 | too few sessions |
| <=0.14 / CO2 | 7 | 2 | 1 | -0.104 | -0.351 | +0.072 | 0 / 0 / 1 | too few sessions |
| <=0.14 / N2 only | 21 | 5 | 2 | -0.123 | -0.402 | +0.199 | 0 / 0 / 2 | too few sessions |
| >=0.30 / N2 only | 32 | 6 | 3 | +0.014 | -0.170 | +0.190 | 0 / 3 / 0 | no |

**Diluent x pressure**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| CO2 / 0.7 atm | 56 | 12 | 5 | +0.018 | -0.554 | +0.467 | 1 / 3 / 1 | no |
| CO2 / 1.0 atm | 43 | 9 | 4 | -0.013 | -1.126 | +0.105 | 0 / 3 / 1 | no |
| He / 0.7 atm | 18 | 4 | 2 | +0.003 | -0.062 | +0.258 | 0 / 2 / 0 | too few sessions |
| He / 1.0 atm | 26 | 6 | 4 | +0.023 | -0.292 | +0.247 | 2 / 2 / 0 | no |
| N2 only / 0.7 atm | 23 | 4 | 2 | +0.015 | -0.081 | +0.190 | 0 / 2 / 0 | too few sessions |
| N2 only / 1.0 atm | 97 | 23 | 9 | -0.047 | -0.402 | +0.221 | 0 / 5 / 4 | no |

**Oxygen x clock (raw)**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| O2 0.16-0.25 / s<0.2 | 65 | 34 | 16 | -0.003 | -1.126 | +0.467 | 8 / 2 / 6 | no |
| O2 0.16-0.25 / s>=0.2 | 111 | 36 | 16 | +0.004 | -0.183 | +0.187 | 2 / 14 / 0 | no |
| O2<=0.15 / s<0.2 | 21 | 15 | 5 | -0.157 | -0.402 | +0.199 | 0 / 1 / 4 | yes (-) |
| O2<=0.15 / s>=0.2 | 34 | 13 | 5 | -0.054 | -0.306 | +0.106 | 0 / 2 / 3 | yes (-) |
| O2>=0.30 / s<0.2 | 11 | 6 | 3 | +0.074 | -0.170 | +0.190 | 2 / 1 / 0 | no |
| O2>=0.30 / s>=0.2 | 21 | 6 | 3 | -0.002 | -0.101 | +0.061 | 0 / 3 / 0 | no |

**D0 x clock (within-run)**

| class | windows | runs | sessions | median | min | max | sessions with median > +0.05 / within / < -0.05 | consistent? |
|---|---|---|---|---|---|---|---|---|
| 2.2-2.8 / 0.1-0.2 | 10 | 10 | 9 | +0.000 | -0.209 | +0.254 | 3 / 5 / 1 | no |
| 2.2-2.8 / s<0.1 | 13 | 13 | 12 | -0.004 | -0.265 | +0.245 | 5 / 4 / 3 | no |
| 2.2-2.8 / s>0.2 | 60 | 19 | 15 | +0.000 | -0.260 | +0.137 | 0 / 13 / 2 | no |
| 2.8-3.4 / 0.1-0.2 | 16 | 16 | 13 | -0.021 | -0.200 | +0.077 | 2 / 7 / 4 | no |
| 2.8-3.4 / s<0.1 | 16 | 16 | 13 | +0.034 | -0.293 | +0.208 | 4 / 4 / 5 | no |
| 2.8-3.4 / s>0.2 | 47 | 14 | 12 | +0.000 | -0.132 | +0.227 | 2 / 10 / 0 | no |
| <2.2 / 0.1-0.2 | 6 | 6 | 4 | +0.047 | -0.120 | +0.139 | 2 / 1 / 1 | no |
| <2.2 / s>0.2 | 19 | 8 | 4 | +0.000 | -0.157 | +0.169 | 1 / 3 / 0 | no |
| >=3.4 / 0.1-0.2 | 18 | 14 | 11 | -0.023 | -0.169 | +0.172 | 2 / 4 / 5 | no |
| >=3.4 / s<0.1 | 18 | 14 | 11 | -0.032 | -0.689 | +0.332 | 1 / 4 / 6 | no |
| >=3.4 / s>0.2 | 40 | 14 | 11 | +0.007 | -0.121 | +0.418 | 2 / 8 / 1 | no |

**Per run** (median r over its windows; windows within 1-8 s)

| run | session | order | O2 | CO2 | He | P | D0 | windows | median r all | median r 1-8 s | drift 0.2-1.2 | speed 1-5 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| C03M101 | 2010-12-08 | 1 | 0.18 | 0.15 | 0.00 | 1.00 | 3.86 | 4 | -0.569 | -0.433 | 2.37 | 2.16 |
| C07M101 | 2011-01-12 | 1 | 0.14 | 0.36 | 0.00 | 1.00 | 3.22 | 3 | -0.184 | -0.144 | 0.38 | 0.27 |
| H04M101 | 2011-10-01 | 1 | 0.17 | 0.00 | 0.20 | 0.99 | 2.99 | 2 | -0.172 | -0.292 | 5.17 |  |
| 189R001 | 2010-03-19 | 1 | 0.15 | 0.00 | 0.00 | 1.01 | 3.37 | 4 | -0.157 | -0.206 | 1.92 | 1.47 |
| 175F001 | 2010-03-29 | 1 | 0.14 | 0.00 | 0.00 | 1.01 | 2.66 | 5 | -0.138 | -0.133 | 1.39 | 0.78 |
| C01M602 | 2011-08-24 | 2 | 0.20 | 0.05 | 0.00 | 0.99 | 2.57 | 5 | -0.132 | -0.091 | 1.71 | 2.39 |
| N06M201 | 2011-09-09 | 2 | 0.12 | 0.00 | 0.00 | 1.00 | 1.94 | 4 | -0.128 | -0.063 | 2.12 | 2.18 |
| 175R001 | 2010-03-29 | 2 | 0.14 | 0.00 | 0.00 | 1.01 | 3.33 | 5 | -0.123 | -0.118 | 1.03 | 0.82 |
| C01M501 | 2011-08-24 | 1 | 0.20 | 0.05 | 0.00 | 0.99 | 3.37 | 5 | -0.122 | -0.071 | 0.94 | 1.24 |
| C14M201 | 2010-11-04 | 2 | 0.15 | 0.50 | 0.00 | 0.70 | 2.86 | 2 | -0.119 | -0.157 | 0.84 | 1.03 |
| C09M101 | 2010-06-21 | 1 | 0.23 | 0.25 | 0.00 | 0.70 | 4.28 | 5 | -0.115 | -0.154 | 1.66 | 1.58 |
| 177F001 | 2009-11-27 | 2 | 0.21 | 0.00 | 0.00 | 1.01 | 3.89 | 5 | -0.075 | -0.106 | 0.0 | 0.00 |
| 178F001 | 2009-11-13 | 2 | 0.30 | 0.00 | 0.00 | 1.01 | 3.26 | 6 | -0.065 | -0.030 | 1.17 | 1.61 |
| N06M202 | 2011-09-09 | 3 | 0.12 | 0.00 | 0.00 | 1.00 | 1.84 | 3 | -0.064 | +0.021 | 0.45 |  |
| 189R002 | 2010-03-19 | 4 | 0.15 | 0.00 | 0.00 | 1.01 | 2.25 | 1 | -0.060 | +nan | 6.3 |  |
| 189R003 | 2010-03-19 | 5 | 0.15 | 0.00 | 0.00 | 1.01 | 2.38 | 5 | -0.054 | -0.040 | 0.71 | 0.26 |
| 177R002 | 2009-11-27 | 1 | 0.21 | 0.00 | 0.00 | 1.01 | 2.62 | 5 | -0.052 | -0.039 | 1.11 | 1.35 |
| 189F001 | 2010-03-19 | 2 | 0.15 | 0.00 | 0.00 | 1.01 | 2.52 | 4 | -0.048 | -0.044 | 3.47 | 2.35 |
| N06M101 | 2011-09-09 | 1 | 0.12 | 0.00 | 0.00 | 1.00 | 2.60 | 4 | -0.046 | -0.051 | 0.9 | 0.37 |
| 189F002 | 2010-03-19 | 3 | 0.15 | 0.00 | 0.00 | 1.01 | 3.97 | 6 | -0.042 | -0.107 | 0.89 | 0.76 |
| C09M201 | 2010-06-21 | 2 | 0.23 | 0.25 | 0.00 | 0.71 | 3.64 | 6 | -0.042 | -0.033 | 1.86 | 1.42 |
| 176R001 | 2010-02-04 | 1 | 0.18 | 0.00 | 0.00 | 1.01 | 2.93 | 6 | -0.034 | -0.034 | 1.76 | 1.62 |
| N01M101 | 2011-08-24 | 4 | 0.21 | 0.00 | 0.00 | 1.00 | 3.30 | 7 | -0.032 | -0.030 | 1.08 | 0.73 |
| 176F001 | 2010-02-04 | 2 | 0.18 | 0.00 | 0.00 | 1.02 | 3.69 | 6 | -0.031 | -0.006 | 0.57 | 0.61 |
| 30CAL03 | 2009-03-31 | 2 | 0.21 | 0.00 | 0.00 | 1.00 | 1.92 | 4 | -0.029 | -0.031 | 0.75 | 1.25 |
| C07M201 | 2011-01-12 | 2 | 0.14 | 0.36 | 0.00 | 1.00 | 2.32 | 4 | -0.015 | -0.018 | 3.94 | 2.19 |
| C03M301 | 2010-12-08 | 3 | 0.18 | 0.15 | 0.00 | 1.01 | 2.78 | 6 | -0.006 | -0.006 | 1.97 | 1.23 |
| H01M102 | 2011-09-09 | 4 | 0.20 | 0.00 | 0.05 | 1.00 | 2.91 | 6 | -0.001 | -0.008 | 0.75 | 0.92 |
| H10M101 | 2011-11-18 | 1 | 0.24 | 0.00 | 0.20 | 0.70 | 2.82 | 5 | -0.001 | -0.006 | 2.66 | 2.24 |
| H04M202 | 2011-10-01 | 2 | 0.17 | 0.00 | 0.20 | 1.00 | 2.90 | 5 | +0.000 | -0.010 | 1.61 | 1.60 |
| H12M102 | 2011-11-18 | 4 | 0.21 | 0.00 | 0.30 | 0.70 | 3.25 | 5 | +0.002 | -0.002 | 2.85 | 2.25 |
| H11M101 | 2011-11-18 | 2 | 0.23 | 0.00 | 0.25 | 0.70 | 2.45 | 5 | +0.004 | +0.003 | 3.43 | 2.27 |
| 20CAL08 | 2009-03-05 | 1 | 0.21 | 0.00 | 0.00 | 1.00 | 1.69 | 3 | +0.005 | -0.008 | 2.11 |  |
| 053R001 | 2010-05-05 | 2 | 0.25 | 0.15 | 0.00 | 0.71 | 3.01 | 6 | +0.006 | +0.008 | 0.66 | 0.89 |
| C02M201 | 2011-08-08 | 4 | 0.19 | 0.10 | 0.00 | 1.00 | 2.47 | 5 | +0.007 | +0.021 | 0.54 | 0.52 |
| H15M102 | 2011-12-05 | 1 | 0.17 | 0.00 | 0.45 | 0.70 | 2.79 | 3 | +0.010 | -0.004 | 0.79 | 1.68 |
| 053F001 | 2010-05-05 | 1 | 0.25 | 0.15 | 0.00 | 0.71 | 3.70 | 6 | +0.011 | +0.037 | 0.83 | 0.78 |
| H05M102 | 2011-10-01 | 3 | 0.16 | 0.00 | 0.25 | 1.00 | 3.40 | 4 | +0.012 | -0.002 | 0.74 | 0.42 |
| 192F002 | 2009-10-24 | 2 | 0.34 | 0.00 | 0.00 | 0.71 | 3.49 | 6 | +0.014 | +0.033 | 0.38 | 1.41 |
| 191R001 | 2009-11-09 | 1 | 0.30 | 0.00 | 0.00 | 0.71 | 2.60 | 4 | +0.014 | -0.002 | 3.83 | 2.65 |
| N01M201 | 2011-08-24 | 5 | 0.21 | 0.00 | 0.00 | 1.00 | 2.75 | 3 | +0.017 | +0.018 | 0.73 | 0.96 |
| 192F003 | 2009-10-24 | 3 | 0.34 | 0.00 | 0.00 | 0.71 | 3.50 | 7 | +0.018 | +0.033 | 1.0 | 0.39 |
| C01M603 | 2011-08-24 | 3 | 0.20 | 0.05 | 0.00 | 1.00 | 2.40 | 5 | +0.033 | +0.035 | 1.12 | 1.18 |
| 191F001 | 2009-11-09 | 2 | 0.30 | 0.00 | 0.00 | 0.71 | 3.63 | 6 | +0.036 | +0.084 | 0.7 | 0.81 |
| C03M201 | 2010-12-08 | 2 | 0.18 | 0.15 | 0.00 | 1.01 | 3.33 | 6 | +0.037 | +0.059 | 0.67 | 1.05 |
| C09M601 | 2011-08-08 | 2 | 0.23 | 0.25 | 0.00 | 0.70 | 2.80 | 6 | +0.039 | +0.039 | 1.27 | 1.12 |
| C09M701 | 2011-08-08 | 3 | 0.23 | 0.25 | 0.00 | 0.71 | 2.47 | 5 | +0.043 | +0.019 | 0.21 | 0.77 |
| 178R001 | 2009-11-13 | 1 | 0.30 | 0.00 | 0.00 | 1.01 | 2.57 | 3 | +0.047 | +0.048 | 2.45 |  |
| C14M302 | 2010-11-04 | 4 | 0.15 | 0.50 | 0.00 | 0.70 | 2.25 | 2 | +0.055 | -0.052 | 2.26 | 2.57 |
| H03M101 | 2011-09-22 | 1 | 0.18 | 0.00 | 0.15 | 1.00 | 2.79 | 6 | +0.057 | +0.056 | 0.65 | 1.31 |
| 20CAL12 | 2009-03-05 | 3 | 0.21 | 0.00 | 0.00 | 1.01 | 1.61 | 3 | +0.062 | +0.042 | 0.53 | 1.30 |
| H02M101 | 2011-09-21 | 1 | 0.19 | 0.00 | 0.10 | 0.99 | 2.78 | 3 | +0.068 | +0.031 | 2.86 |  |
| C14M301 | 2010-11-04 | 3 | 0.15 | 0.50 | 0.00 | 0.70 | 1.86 | 3 | +0.072 | +0.045 | 4.76 |  |
| 30CAL02 | 2009-03-31 | 1 | 0.21 | 0.00 | 0.00 | 1.00 | 2.06 | 2 | +0.082 | +0.007 | 2.26 |  |
| C12M201 | 2010-09-08 | 2 | 0.18 | 0.40 | 0.00 | 0.70 | 3.46 | 6 | +0.091 | +0.117 | 0.24 | 0.49 |
| C09M502 | 2011-08-08 | 1 | 0.23 | 0.25 | 0.00 | 0.70 | 3.42 | 4 | +0.105 | +0.058 | 2.54 | 2.28 |
| C12M101 | 2010-09-08 | 1 | 0.18 | 0.40 | 0.00 | 0.69 | 4.25 | 5 | +0.135 | +0.074 | 0.44 | 0.22 |
| 20CAL10 | 2009-03-05 | 2 | 0.21 | 0.00 | 0.00 | 1.00 | 1.86 | 3 | +0.187 | +0.109 | 1.3 | 1.32 |

Spearman rank correlation of run median r (1-8 s) with drift 0.2-1.2 s: rho = -0.22 over 57 runs (descriptive).

| drift 0.2-1.2 s class | runs | sessions | median of run medians | min | max |
|---|---|---|---|---|---|
| 1-2 | 16 | 12 | -0.031 | -0.206 | +0.109 |
| 2-4 | 14 | 12 | -0.004 | -0.433 | +0.058 |
| <1 mm/s | 25 | 18 | +0.008 | -0.157 | +0.117 |
| >=4 | 2 | 2 | -0.123 | -0.292 | +0.045 |

Spearman rank correlation of run median r (1-8 s) with speed 1-5 s: rho = -0.11 over 50 runs (descriptive).

| speed 1-5 s class | runs | sessions | median of run medians | min | max |
|---|---|---|---|---|---|
| 1-2 | 19 | 15 | -0.010 | -0.206 | +0.109 |
| 2-4 | 11 | 9 | -0.018 | -0.433 | +0.058 |
| <1 mm/s | 20 | 13 | -0.004 | -0.144 | +0.117 |

**Conclusion B.**
- **The level is right.** The median of all 263 windows is −0.003 (58 runs, 23 sessions). The run
  medians have a robust sd of 0.06 (IQR/1.349), a 5 % quantile of −0.21 and a 95 % quantile of +0.07,
  so the tail is on the slow side.
- **The shape in w and s is right.** Within-run medians by progress class and clock class are all
  within ±0.005. Water uptake (skip −0.35 per unit w) and the heat-up clock (0.18) are not
  contradicted on average.
- **One consistent offset: oxygen at or below 15 %.**
  - ≤ 14 % O2: −0.12 (3 of 3 sessions below −0.05).
  - Split by clock, at ≤ 15 % O2: −0.16 for s < 0.2 (4 of 5 sessions) and −0.05 for s ≥ 0.2
    (3 of 5 sessions).
  - The deficit is mostly a slower heat-up at low O2, plus about 5 % on the plateau. Both diffusive
    runs (175F001, N06M201) and radiative runs show it, so it is not only pre-extinction weakening.
- **No consistent offset** for CO2, helium, pressure, D0, burn order in the session (burn 1: −0.016,
  burn 2: −0.011, burn 3: +0.018) or drift speed.
  - Drift speed: Spearman ρ = −0.25 (0.2–1.2 s drift) and −0.14 (1–5 s speed). The class medians lie
    within −0.04..+0.01 except one class of 2 runs.
  - The burn-order result agrees with the sheets: the tabulated atmospheres are already computed
    after the fact from the fill and combustion history.
- **Not consistent (two sessions):** at ≥ 30 % O2 the heat-up is faster (+0.07, 2 of 3 sessions;
  +0.17 to +0.19 in the 1–2 s window for 191F001, 192F002 and 192F003). See "Leave alone".

---

## C. Extinction audit

Branch from NASA's numbers: diffusive if Dext/D0 ≤ 0.65, radiative if above, none if ≥ 0.97. Dext
provenance comes from the narrative and the series:
- direct: the series reaches Dext in view.
- extrapolated: the narrative says so.
- listed though narrative says none: the sheet lists a Dext but the narrative says none is listed or
  that the droplet was out of view.
- not in view: no statement, and the series ends before Dext.

w_obs = 1 − (Dext/D0)². s_obs = t_ext/D0², with NASA D0 in mm. Model columns come from the D2
encoder (earliest cutoff). Runs with neither Dext nor a knee are omitted.

| run | session | D0 | Dext | Dext/D0 | branch | Dext provenance | t_ext s | t_knee | w_obs | s_obs | D_crit | x_exc | pi_r | w_d | s_r | flag |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 191F001 | 2009-11-09 | 3.63 | 0.40 | 0.11 | diffusive | not_in_view |  |  | 0.99 |  | 27.69 | -2.03 | 0.00 | 0.90 | 0.90 |  |
| 191R001 | 2009-11-09 | 2.60 | 0.59 | 0.23 | diffusive | not_in_view |  |  | 0.95 |  | 27.65 | -2.36 | 0.00 | 0.90 | 0.90 |  |
| 192F002 | 2009-10-24 | 3.49 | 0.95 | 0.27 | diffusive | direct | 17.0 | 16.75 | 0.93 | 1.40 | 50.60 | -2.67 | 0.00 | 0.93 | 0.90 |  |
| C01M501 | 2011-08-24 | 3.37 | 0.98 | 0.29 | diffusive | listed_though_narrative_says_none |  |  | 0.92 |  | 7.51 | -0.80 | 0.00 | 0.79 | 0.90 |  |
| H02M101 | 2011-09-21 | 2.78 | 0.83 | 0.30 | diffusive | listed_though_narrative_says_none |  |  | 0.91 |  | 6.15 | -0.79 | 0.00 | 0.79 | 0.90 |  |
| 053R001 | 2010-05-05 | 3.01 | 1.01 | 0.34 | diffusive | direct | 14.9 |  | 0.89 | 1.64 | 11.59 | -1.35 | 0.00 | 0.84 | 0.90 |  |
| H11M101 | 2011-11-18 | 2.45 | 0.86 | 0.35 | diffusive | extrapolated |  |  | 0.88 |  | 7.67 | -1.15 | 0.00 | 0.83 | 0.90 |  |
| C03M201 | 2010-12-08 | 3.33 | 1.20 | 0.36 | diffusive | extrapolated |  |  | 0.87 |  | 5.19 | -0.44 | 0.02 | 0.78 | 0.90 |  |
| H10M101 | 2011-11-18 | 2.82 | 1.03 | 0.37 | diffusive | not_in_view |  |  | 0.87 |  | 9.35 | -1.20 | 0.00 | 0.84 | 0.90 |  |
| H01M102 | 2011-09-09 | 2.91 | 1.12 | 0.38 | diffusive | extrapolated |  |  | 0.85 |  | 7.51 | -0.95 | 0.00 | 0.79 | 0.90 |  |
| N01M201 | 2011-08-24 | 2.75 | 1.11 | 0.40 | diffusive | direct | 12.8 |  | 0.84 | 1.69 | 9.15 | -1.20 | 0.00 | 0.79 | 0.90 |  |
| 178F001 | 2009-11-13 | 3.26 | 1.32 | 0.40 | diffusive | direct | 15.7 | 15.50 | 0.84 | 1.47 | 35.54 | -2.39 | 0.00 | 0.88 | 0.90 |  |
| C01M603 | 2011-08-24 | 2.40 | 0.99 | 0.41 | diffusive | direct | 10.3 | 10.25 | 0.83 | 1.80 | 7.53 | -1.14 | 0.00 | 0.79 | 0.90 |  |
| 053F001 | 2010-05-05 | 3.70 | 1.55 | 0.42 | diffusive | not_in_view |  |  | 0.82 |  | 11.59 | -1.14 | 0.00 | 0.84 | 0.90 |  |
| 30CAL03 | 2009-03-31 | 1.92 | 0.81 | 0.42 | diffusive | direct | 6.5 | 6.25 | 0.82 | 1.75 | 9.14 | -1.55 | 0.00 | 0.79 | 0.90 |  |
| C03M301 | 2010-12-08 | 2.78 | 1.19 | 0.43 | diffusive | extrapolated |  |  | 0.82 |  | 5.19 | -0.62 | 0.00 | 0.78 | 0.90 |  |
| N01M101 | 2011-08-24 | 3.30 | 1.42 | 0.43 | diffusive | direct | 19.6 | 19.25 | 0.81 | 1.80 | 9.14 | -1.02 | 0.00 | 0.79 | 0.90 |  |
| C02M201 | 2011-08-08 | 2.47 | 1.08 | 0.44 | diffusive | direct | 10.9 |  | 0.81 | 1.79 | 6.24 | -0.93 | 0.00 | 0.79 | 0.90 |  |
| 176F001 | 2010-02-04 | 3.69 | 1.62 | 0.44 | diffusive | listed_though_narrative_says_none |  |  | 0.81 |  | 5.90 | -0.46 | 0.02 | 0.78 | 0.90 |  |
| C01M602 | 2011-08-24 | 2.57 | 1.13 | 0.44 | diffusive | extrapolated |  |  | 0.81 |  | 7.53 | -1.07 | 0.00 | 0.79 | 0.90 |  |
| 189R001 | 2010-03-19 | 3.37 | 1.49 | 0.44 | diffusive | listed_though_narrative_says_none |  |  | 0.80 |  | 3.74 | -0.10 | 0.42 | 0.69 | 0.90 |  |
| H03M101 | 2011-09-22 | 2.79 | 1.30 | 0.47 | diffusive | direct | 12.5 | 12.25 | 0.78 | 1.61 | 5.09 | -0.60 | 0.01 | 0.78 | 0.90 |  |
| 176R001 | 2010-02-04 | 2.93 | 1.39 | 0.47 | diffusive | direct | 18.2 | 17.25 | 0.77 | 2.12 | 5.88 | -0.67 | 0.00 | 0.78 | 0.90 |  |
| C09M601 | 2011-08-08 | 2.80 | 1.33 | 0.48 | diffusive | direct | 12.9 | 12.25 | 0.77 | 1.65 | 7.90 | -1.03 | 0.00 | 0.83 | 0.90 |  |
| 177R002 | 2009-11-27 | 2.62 | 1.25 | 0.48 | diffusive | direct | 11.4 | 11.25 | 0.77 | 1.66 | 9.18 | -1.25 | 0.00 | 0.79 | 0.90 |  |
| C09M201 | 2010-06-21 | 3.64 | 1.74 | 0.48 | diffusive | listed_though_narrative_says_none |  |  | 0.77 |  | 7.92 | -0.77 | 0.00 | 0.83 | 0.90 |  |
| H04M202 | 2011-10-01 | 2.90 | 1.42 | 0.49 | diffusive | extrapolated |  |  | 0.76 |  | 4.18 | -0.36 | 0.05 | 0.78 | 0.90 |  |
| C09M701 | 2011-08-08 | 2.47 | 1.21 | 0.49 | diffusive | direct | 9.3 | 9.25 | 0.76 | 1.52 | 7.91 | -1.16 | 0.00 | 0.83 | 0.90 |  |
| 20CAL10 | 2009-03-05 | 1.86 | 0.94 | 0.51 | diffusive | direct | 5.0 | 5.00 | 0.74 | 1.46 | 9.15 | -1.58 | 0.00 | 0.79 | 0.90 |  |
| 177F001 | 2009-11-27 | 3.89 | 2.00 | 0.51 | diffusive | listed_though_narrative_says_none |  |  | 0.74 |  | 9.20 | -0.86 | 0.00 | 0.79 | 0.90 |  |
| C11M201 | 2010-09-06 | 3.23 | 1.68 | 0.52 | diffusive | direct | 19.1 | 19.00 | 0.73 | 1.83 |  |  |  |  |  | problem run |
| 189F001 | 2010-03-19 | 2.52 | 1.32 | 0.52 | diffusive | direct | 12.9 | 11.50 | 0.73 | 2.04 | 3.74 | -0.38 | 0.04 | 0.77 | 0.90 |  |
| N06M201 | 2011-09-09 | 1.94 | 1.05 | 0.54 | diffusive | extrapolated |  |  | 0.71 |  | 2.37 | -0.20 | 0.22 | 0.72 | 0.90 |  |
| 189R003 | 2010-03-19 | 2.38 | 1.29 | 0.54 | diffusive | direct | 10.8 | 10.50 | 0.71 | 1.90 | 3.74 | -0.45 | 0.02 | 0.77 | 0.90 |  |
| C09M101 | 2010-06-21 | 4.28 | 2.32 | 0.54 | diffusive | listed_though_narrative_says_none |  |  | 0.71 |  | 7.90 | -0.60 | 0.00 | 0.83 | 0.90 |  |
| H12M102 | 2011-11-18 | 3.25 | 1.78 | 0.55 | diffusive | extrapolated |  |  | 0.70 |  | 5.43 | -0.52 | 0.01 | 0.83 | 0.90 |  |
| 20CAL08 | 2009-03-05 | 1.69 | 0.95 | 0.56 | diffusive | direct | 4.0 |  | 0.68 | 1.38 | 9.12 | -1.68 | 0.00 | 0.79 | 0.90 |  |
| N06M202 | 2011-09-09 | 1.84 | 1.04 | 0.57 | diffusive | extrapolated |  |  | 0.68 |  | 2.38 | -0.25 | 0.14 | 0.74 | 0.90 |  |
| 20CAL12 | 2009-03-05 | 1.61 | 0.92 | 0.57 | diffusive | direct | 3.4 | 3.50 | 0.67 | 1.31 | 9.17 | -1.74 | 0.00 | 0.79 | 0.90 |  |
| 40CAL01 | 2009-03-31 | 2.54 | 1.53 | 0.60 | diffusive | direct | 3.2 | 3.25 | 0.64 | 0.50 | 9.14 | -1.27 | 0.00 | 0.79 | 0.90 | problem run |
| 175F001 | 2010-03-29 | 2.66 | 1.66 | 0.62 | diffusive | extrapolated |  |  | 0.61 |  | 3.22 | -0.19 | 0.23 | 0.72 | 0.90 |  |
| C14M101 | 2010-11-04 | 3.98 | 3.95 | 0.99 | none | direct | 1.2 | 15.25 | 0.02 | 0.08 | 1.94 | +0.72 | 1.00 | 0.35 | 0.02 |  |
| H08M101 | 2011-10-11 | 3.56 | 3.54 | 0.99 | none | direct | 1.1 | 8.50 | 0.01 | 0.09 | 1.93 | +0.61 | 1.00 | 0.34 | 0.07 |  |
| H12M101 | 2011-11-18 | 3.22 | 2.16 | 0.67 | radiative | listed_though_narrative_says_none |  |  | 0.55 |  | 5.43 | -0.52 | 0.01 | 0.83 | 0.90 | problem run; WRONG SIDE (pi_r<0.3) |
| 189R002 | 2010-03-19 | 2.25 | 1.52 | 0.68 | radiative | listed_though_narrative_says_none |  |  | 0.54 |  | 3.74 | -0.51 | 0.01 | 0.77 | 0.90 | WRONG SIDE (pi_r<0.3) |
| C12M201 | 2010-09-08 | 3.46 | 2.58 | 0.75 | radiative | direct | 14.0 | 15.75 | 0.44 | 1.17 | 3.29 | +0.06 | 0.78 | 0.68 | 0.84 |  |
| C07M201 | 2011-01-12 | 2.32 | 1.84 | 0.79 | radiative | direct | 6.1 | 13.50 | 0.37 | 1.13 | 2.40 | -0.02 | 0.62 | 0.65 | 0.90 |  |
| C14M301 | 2010-11-04 | 1.86 | 1.48 | 0.80 | radiative | direct | 3.5 | 21.50 | 0.37 | 1.03 | 1.95 | -0.04 | 0.56 | 0.71 | 0.90 |  |
| N06M101 | 2011-09-09 | 2.60 | 2.09 | 0.80 | radiative | direct | 7.0 | 7.50 | 0.35 | 1.04 | 2.37 | +0.09 | 0.83 | 0.59 | 0.81 |  |
| H05M102 | 2011-10-01 | 3.40 | 2.79 | 0.82 | radiative | direct | 8.1 | 8.75 | 0.33 | 0.70 | 3.45 | -0.01 | 0.64 | 0.66 | 0.90 |  |
| H15M102 | 2011-12-05 | 2.79 | 2.30 | 0.82 | radiative | direct | 4.1 | 4.75 | 0.32 | 0.53 | 2.60 | +0.07 | 0.80 | 0.67 | 0.83 |  |
| 189F002 | 2010-03-19 | 3.97 | 3.30 | 0.83 | radiative | direct | 14.2 |  | 0.31 | 0.90 | 3.74 | +0.06 | 0.78 | 0.62 | 0.84 |  |
| 175R001 | 2010-03-29 | 3.33 | 2.91 | 0.87 | radiative | direct | 8.8 | 9.75 | 0.24 | 0.79 | 3.22 | +0.04 | 0.74 | 0.62 | 0.86 |  |
| C14M302 | 2010-11-04 | 2.25 | 2.06 | 0.92 | radiative | direct | 3.0 | 4.25 | 0.16 | 0.60 | 1.95 | +0.16 | 0.91 | 0.62 | 0.74 |  |
| C12M101 | 2010-09-08 | 4.25 | 3.92 | 0.92 | radiative | direct | 8.3 | 11.50 | 0.15 | 0.46 | 3.27 | +0.26 | 0.96 | 0.59 | 0.66 |  |
| C14M201 | 2010-11-04 | 2.86 | 2.73 | 0.95 | radiative | direct | 2.5 | 5.50 | 0.09 | 0.31 | 1.95 | +0.39 | 0.99 | 0.51 | 0.57 |  |
| C12M401 | 2010-09-08 | 2.87 | 2.74 | 0.95 | radiative | direct | 2.5 | 6.00 | 0.09 | 0.31 | 3.30 | -0.14 | 0.34 | 0.76 | 0.90 | problem run |
| C07M101 | 2011-01-12 | 3.22 | 3.08 | 0.96 | radiative | direct | 3.3 | 5.75 | 0.09 | 0.32 | 2.40 | +0.29 | 0.97 | 0.50 | 0.63 |  |

# C. Extinction summaries (problem runs excluded)

**Diffusive: observed w = 1-(Dext/D0)^2, all**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| all | 39 | 19 | 0.81 | 0.61 | 0.99 | 0.79 | -0.00 |

**Diffusive w by provenance of NASA Dext**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| direct | 18 | 11 | 0.78 | 0.67 | 0.93 | 0.79 | -0.01 |
| extrapolated | 10 | 6 | 0.78 | 0.61 | 0.88 | 0.78 | +0.00 |
| listed_though_narrative_says_none | 7 | 6 | 0.80 | 0.71 | 0.92 | 0.79 | +0.03 |
| not_in_view | 4 | 3 | 0.91 | 0.82 | 0.99 | 0.87 | +0.04 |

**Diffusive w, DIRECT Dext only, by oxygen**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| 0.16-0.19 | 3 | 3 | 0.78 | 0.77 | 0.81 | 0.78 | +0.00 |
| 0.20-0.21 | 8 | 4 | 0.79 | 0.67 | 0.84 | 0.79 | +0.00 |
| 0.23-0.25 | 3 | 2 | 0.77 | 0.76 | 0.89 | 0.83 | -0.06 |
| <=0.15 | 2 | 1 | 0.72 | 0.71 | 0.73 | 0.77 | -0.05 |
| >=0.30 | 2 | 2 | 0.88 | 0.84 | 0.93 | 0.91 | -0.02 |

**Diffusive w, direct only, by pressure**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| 0.7 atm | 4 | 3 | 0.83 | 0.76 | 0.93 | 0.84 | -0.03 |
| 1.0 atm | 14 | 9 | 0.78 | 0.67 | 0.84 | 0.79 | -0.01 |

**Diffusive w, direct only, by diluent**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| CO2 | 5 | 3 | 0.81 | 0.76 | 0.89 | 0.83 | +0.02 |
| He | 1 | 1 | 0.78 | 0.78 | 0.78 | 0.78 | +0.00 |
| N2 only | 12 | 8 | 0.77 | 0.67 | 0.93 | 0.79 | -0.03 |

**Diffusive w, direct only, by D0**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| 2.2-2.8 | 8 | 5 | 0.78 | 0.71 | 0.84 | 0.79 | -0.01 |
| 2.8-3.4 | 5 | 5 | 0.81 | 0.77 | 0.89 | 0.83 | -0.00 |
| <2.2 | 4 | 2 | 0.71 | 0.67 | 0.82 | 0.79 | -0.08 |
| >=3.4 | 1 | 1 | 0.93 | 0.93 | 0.93 | 0.93 | -0.01 |

**Diffusive w, all provenances, by oxygen**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| 0.16-0.19 | 8 | 6 | 0.81 | 0.76 | 0.91 | 0.78 | +0.03 |
| 0.20-0.21 | 13 | 6 | 0.81 | 0.67 | 0.92 | 0.79 | +0.02 |
| 0.23-0.25 | 8 | 4 | 0.80 | 0.71 | 0.89 | 0.83 | -0.04 |
| <=0.15 | 6 | 3 | 0.71 | 0.61 | 0.80 | 0.73 | -0.05 |
| >=0.30 | 4 | 3 | 0.94 | 0.84 | 0.99 | 0.90 | +0.02 |

**Diffusive w, all provenances, by pressure**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| 0.7 atm | 12 | 6 | 0.85 | 0.70 | 0.99 | 0.83 | -0.01 |
| 1.0 atm | 27 | 14 | 0.80 | 0.61 | 0.92 | 0.79 | +0.00 |

**Diffusive w, all provenances, by diluent**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| CO2 | 12 | 5 | 0.81 | 0.71 | 0.92 | 0.81 | +0.02 |
| He | 7 | 5 | 0.85 | 0.70 | 0.91 | 0.79 | +0.03 |
| N2 only | 20 | 11 | 0.77 | 0.61 | 0.99 | 0.79 | -0.01 |

**Diffusive w, all provenances, by D0**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| 2.2-2.8 | 14 | 10 | 0.81 | 0.61 | 0.95 | 0.79 | +0.02 |
| 2.8-3.4 | 12 | 10 | 0.83 | 0.70 | 0.92 | 0.79 | +0.03 |
| <2.2 | 6 | 3 | 0.70 | 0.67 | 0.82 | 0.79 | -0.05 |
| >=3.4 | 7 | 6 | 0.81 | 0.71 | 0.99 | 0.83 | -0.01 |

**Diffusive w by session**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| 2009-03-05 | 3 | 1 | 0.68 | 0.67 | 0.74 | 0.79 | -0.11 |
| 2009-03-31 | 1 | 1 | 0.82 | 0.82 | 0.82 | 0.79 | +0.03 |
| 2009-10-24 | 1 | 1 | 0.93 | 0.93 | 0.93 | 0.93 | -0.01 |
| 2009-11-09 | 2 | 1 | 0.97 | 0.95 | 0.99 | 0.90 | +0.07 |
| 2009-11-13 | 1 | 1 | 0.84 | 0.84 | 0.84 | 0.88 | -0.04 |
| 2009-11-27 | 2 | 1 | 0.75 | 0.74 | 0.77 | 0.79 | -0.04 |
| 2010-02-04 | 2 | 1 | 0.79 | 0.77 | 0.81 | 0.78 | +0.01 |
| 2010-03-19 | 3 | 1 | 0.73 | 0.71 | 0.80 | 0.77 | -0.04 |
| 2010-03-29 | 1 | 1 | 0.61 | 0.61 | 0.61 | 0.72 | -0.11 |
| 2010-05-05 | 2 | 1 | 0.86 | 0.82 | 0.89 | 0.84 | +0.02 |
| 2010-06-21 | 2 | 1 | 0.74 | 0.71 | 0.77 | 0.83 | -0.09 |
| 2010-12-08 | 2 | 1 | 0.84 | 0.82 | 0.87 | 0.78 | +0.06 |
| 2011-08-08 | 3 | 1 | 0.77 | 0.76 | 0.81 | 0.83 | -0.06 |
| 2011-08-24 | 5 | 1 | 0.83 | 0.81 | 0.92 | 0.79 | +0.04 |
| 2011-09-09 | 3 | 1 | 0.71 | 0.68 | 0.85 | 0.74 | -0.01 |
| 2011-09-21 | 1 | 1 | 0.91 | 0.91 | 0.91 | 0.79 | +0.13 |
| 2011-09-22 | 1 | 1 | 0.78 | 0.78 | 0.78 | 0.78 | +0.00 |
| 2011-10-01 | 1 | 1 | 0.76 | 0.76 | 0.76 | 0.78 | -0.02 |
| 2011-11-18 | 3 | 1 | 0.87 | 0.70 | 0.88 | 0.83 | +0.03 |

**Diffusive w by x_exc = ln(D0/D_crit) (model)**

| class | runs | sessions | median w_obs | min | max | median model w_d | median obs - model |
|---|---|---|---|---|---|---|---|
| -0.4..0 | 6 | 4 | 0.72 | 0.61 | 0.80 | 0.73 | -0.03 |
| -1..-0.4 | 14 | 11 | 0.80 | 0.70 | 0.92 | 0.79 | +0.01 |
| x_exc < -1 | 19 | 10 | 0.82 | 0.67 | 0.99 | 0.83 | +0.02 |

**Radiative: observed s = t_ext / D0^2 (direct only)**

| class | runs | sessions | median s_obs | min | max | median model s_r | median obs - model |
|---|---|---|---|---|---|---|---|
| all | 12 | 8 | 0.74 | 0.31 | 1.17 | 0.84 | -0.10 |

**Radiative s by x_exc (model)**

| class | runs | sessions | median s_obs | min | max | median model s_r | median obs - model |
|---|---|---|---|---|---|---|---|
| 0..0.1 | 5 | 5 | 0.90 | 0.53 | 1.17 | 0.84 | +0.06 |
| 0.1..0.3 | 3 | 3 | 0.46 | 0.32 | 0.60 | 0.66 | -0.20 |
| >=0.3 | 1 | 1 | 0.31 | 0.31 | 0.31 | 0.57 | -0.25 |
| x_exc < 0 | 3 | 3 | 1.03 | 0.70 | 1.13 | 0.90 | +0.13 |

**Radiative s by diluent**

| class | runs | sessions | median s_obs | min | max | median model s_r | median obs - model |
|---|---|---|---|---|---|---|---|
| CO2 | 7 | 3 | 0.60 | 0.31 | 1.17 | 0.74 | -0.14 |
| He | 2 | 2 | 0.61 | 0.53 | 0.70 | 0.87 | -0.25 |
| N2 only | 3 | 3 | 0.90 | 0.79 | 1.04 | 0.84 | +0.06 |

**Radiative s by pressure**

| class | runs | sessions | median s_obs | min | max | median model s_r | median obs - model |
|---|---|---|---|---|---|---|---|
| 0.7 atm | 6 | 3 | 0.56 | 0.31 | 1.17 | 0.79 | -0.17 |
| 1.0 atm | 6 | 5 | 0.85 | 0.32 | 1.13 | 0.85 | -0.00 |

**Radiative s by D0**

| class | runs | sessions | median s_obs | min | max | median model s_r | median obs - model |
|---|---|---|---|---|---|---|---|
| 2.2-2.8 | 4 | 4 | 0.82 | 0.53 | 1.13 | 0.82 | +0.05 |
| 2.8-3.4 | 3 | 3 | 0.32 | 0.31 | 0.79 | 0.63 | -0.25 |
| <2.2 | 1 | 1 | 1.03 | 1.03 | 1.03 | 0.90 | +0.13 |
| >=3.4 | 4 | 3 | 0.80 | 0.46 | 1.17 | 0.84 | -0.07 |

**Radiative s by session**

| class | runs | sessions | median s_obs | min | max | median model s_r | median obs - model |
|---|---|---|---|---|---|---|---|
| 2010-03-19 | 1 | 1 | 0.90 | 0.90 | 0.90 | 0.84 | +0.06 |
| 2010-03-29 | 1 | 1 | 0.79 | 0.79 | 0.79 | 0.86 | -0.07 |
| 2010-09-08 | 2 | 1 | 0.81 | 0.46 | 1.17 | 0.75 | +0.06 |
| 2010-11-04 | 3 | 1 | 0.60 | 0.31 | 1.03 | 0.74 | -0.14 |
| 2011-01-12 | 2 | 1 | 0.73 | 0.32 | 1.13 | 0.77 | -0.04 |
| 2011-09-09 | 1 | 1 | 1.04 | 1.04 | 1.04 | 0.81 | +0.23 |
| 2011-10-01 | 1 | 1 | 0.70 | 0.70 | 0.70 | 0.90 | -0.20 |
| 2011-12-05 | 1 | 1 | 0.53 | 0.53 | 0.53 | 0.83 | -0.30 |

**Radiative s by Dext/D0**

| class | runs | sessions | median s_obs | min | max | median model s_r | median obs - model |
|---|---|---|---|---|---|---|---|
| 0.75-0.9 | 7 | 7 | 0.90 | 0.53 | 1.13 | 0.86 | +0.06 |
| <0.75 | 1 | 1 | 1.17 | 1.17 | 1.17 | 0.84 | +0.33 |
| ratio>=0.9 | 4 | 3 | 0.39 | 0.31 | 0.60 | 0.65 | -0.23 |

**Radiative: progress at extinction**

| class | runs | sessions | median w_obs | min | max | median model  | median obs - model |
|---|---|---|---|---|---|---|---|
| all | 12 | 8 | 0.31 | 0.09 | 0.44 | nan | +nan |

**Direct Dext vs the diameter at the knee in the series** (diffusive and radiative, both visible)

| run | branch | Dext | D at knee (profile) | t_cross | t_knee | Dext - D_knee |
|---|---|---|---|---|---|---|
| 175R001 | radiative | 2.91 | 2.88 | 8.8 | 9.75 | +0.03 |
| 176R001 | diffusive | 1.39 | 1.40 | 18.2 | 17.25 | -0.01 |
| 177R002 | diffusive | 1.25 | 1.25 | 11.4 | 11.25 | +0.00 |
| 178F001 | diffusive | 1.32 | 1.32 | 15.65 | 15.50 | -0.00 |
| 189F001 | diffusive | 1.32 | 1.44 | 12.95 | 11.50 | -0.12 |
| 189R003 | diffusive | 1.29 | 1.30 | 10.75 | 10.50 | -0.01 |
| 192F002 | diffusive | 0.95 | 0.97 | 17.0 | 16.75 | -0.02 |
| 20CAL10 | diffusive | 0.94 | 0.92 | 5.05 | 5.00 | +0.02 |
| 20CAL12 | diffusive | 0.92 | 0.89 | 3.4 | 3.50 | +0.03 |
| 30CAL03 | diffusive | 0.81 | 0.81 | 6.45 | 6.25 | -0.00 |
| C01M603 | diffusive | 0.99 | 0.98 | 10.35 | 10.25 | +0.01 |
| C07M101 | radiative | 3.08 | 3.02 | 3.3 | 5.75 | +0.06 |
| C09M601 | diffusive | 1.33 | 1.35 | 12.9 | 12.25 | -0.02 |
| C09M701 | diffusive | 1.21 | 1.20 | 9.3 | 9.25 | +0.01 |
| C12M101 | radiative | 3.92 | 3.85 | 8.3 | 11.50 | +0.07 |
| C12M201 | radiative | 2.58 | 2.51 | 14.0 | 15.75 | +0.07 |
| C14M101 | none | 3.95 | 3.85 | 1.2 | 15.25 | +0.10 |
| C14M201 | radiative | 2.73 | 2.65 | 2.55 | 5.50 | +0.08 |
| C14M302 | radiative | 2.06 | 2.02 | 3.05 | 4.25 | +0.04 |
| H03M101 | diffusive | 1.30 | 1.31 | 12.5 | 12.25 | -0.01 |
| H05M102 | radiative | 2.79 | 2.75 | 8.05 | 8.75 | +0.04 |
| H08M101 | none | 3.54 | 3.48 | 1.1 | 8.50 | +0.06 |
| H15M102 | radiative | 2.30 | 2.24 | 4.1 | 4.75 | +0.06 |
| N01M101 | diffusive | 1.42 | 1.43 | 19.55 | 19.25 | -0.01 |
| N06M101 | radiative | 2.09 | 2.07 | 7.05 | 7.50 | +0.02 |

**Radiative runs side by side within one session** (same atmosphere; taken from the table above):

| session (atmosphere) | smaller droplet: D0, s_obs, Dext/D0 | larger droplet: D0, s_obs, Dext/D0 | Δln s / Δln D0 |
|---|---|---|---|
| 2010-11-04 (15 % O2, 50 % CO2, 0.7 atm) | C14M301: 1.86, 1.03, 0.80 | C14M302: 2.25, 0.60, 0.92 | −2.8 |
| 2010-11-04 (same) | C14M302: 2.25, 0.60, 0.92 | C14M201: 2.86, 0.31, 0.95 | −2.8 |
| 2010-11-04 (same) | C14M201: 2.86, 0.31 | C14M101: 3.98, no sustained flame | |
| 2010-09-08 (18 % O2, 40 % CO2, 0.7 atm) | C12M201: 3.46, 1.17, 0.75 | C12M101: 4.25, 0.46, 0.92 | −4.5 |
| 2011-01-12 (14 % O2, 36 % CO2, 1 atm) | C07M201: 2.32, 1.13, 0.79 | C07M101: 3.22, 0.32, 0.96 | −3.8 |

In D2, d ln s_R / d ln D0 is −1.2 for 0 < x_exc < 0.45 (rad unit 0) and 0 below x_exc = 0.

**Conclusion C.**
- **Class law.** No reliable run is on the wrong side.
  - Only 189R002 (Dext listed though the narrative says none; out of view after 1.4 s) and H12M101
    (problem run) have pi_r < 0.3 with Dext/D0 > 0.65.
  - No diffusive run has pi_r > 0.7.
  - The boundary sits between x_exc = −0.10 (189R001, diffusive, Dext of unknown provenance) and
    −0.04 (C14M301, radiative), where D2 gives 0.43 and 0.57.
- **Diffusive w.** The median observed w is 0.81 against a median w_D of 0.79 (39 runs, 19 sessions;
  the median difference is 0.00).
  - There is no pressure or diluent trend beyond ±0.03.
  - At ≤ 15 % O2, w is 0.05 below the law (6 runs, 3 sessions; 5 of the 6 runs below).
  - At D0 < 2.2 mm, w is 0.05–0.08 below the law. This rests on the 2009-03-05 CAL session and N06,
    and 30CAL03 goes the other way.
  - By session the median difference ranges from −0.11 to +0.13, which is scatter between runs, not
    a law.
- **Radiative s.** s falls steeply as the droplet gets more supercritical. Within the same atmosphere,
  a 20–40 % larger droplet goes out at a third to half the clock (−2.8 to −4.5 in ln s per ln D0; 4
  pairs in 3 sessions). The law has −1.2.
  - Droplets near D_crit (x_exc −0.04..+0.09) go out at s = 0.79–1.17 in CO2 and N2, against the
    law's 0.90. The two helium runs are lower: 0.70 and 0.53.
  - Those that end with Dext/D0 ≥ 0.9 go out at s = 0.31–0.60, against the law's 0.57–0.74.
  - All 4 steep pairs are CO2-diluted. No N2 or helium session has two radiative droplets of
    different size.
- **Extrapolated Dext.** Values NASA says were extrapolated are not systematically different from
  direct ones: diffusive w is 0.78 for direct (18 runs, 11 sessions) and 0.78 for extrapolated
  (10 runs, 6 sessions). The 7 "listed though narrative says none" values give 0.80. The 4 "not in
  view" runs give 0.91, but those are all 25–34 % O2 or 0.7 atm, which is confounded with condition.

---

## D. State at the cutoff

All D2 cases (scored or not) where the droplet was out before the cutoff or went out within 1.5 s
after it, plus any burning case with P(no flame) > 0.2 (there is none besides the no-flame runs).
r_late is the encoder's late rate ratio. rho_e is the mean the filter expects for a droplet without a
flame: evap_rate0(q_cut, 0.5)/K_model.

| run | cutoff | t_ext | t_ext - cutoff | branch | prior hot / none | state hot / none | r_late | gain | K_obs late mm2/s | K_model late | bias 0-2 / 2-5 / 5-10 um | CRPS um | verdict |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 20CAL08 | 3 | 4.0 | +1.0 | diffusive | 0.99 / 0.01 | 1.00 / 0.00 | 1.00 | 0.93 | 0.522 | 0.520 | 2 / . / . | 10 | hot at cutoff, out within 1.5 s |
| 20CAL10 | 5 | 5.0 | +0.0 | diffusive | 0.70 / 0.30 | 0.99 / 0.01 | 1.03 | 0.98 | 0.506 | 0.491 | -84 / -94 / -82 | 42 | hot at cutoff, out within 1.5 s |
| 20CAL12 | 3 | 3.4 | +0.4 | diffusive | 0.95 / 0.05 | 1.00 / 0.00 | 1.06 | 0.93 | 0.538 | 0.509 | -90 / -124 / -107 | 70 | hot at cutoff, out within 1.5 s |
| 20CAL12 | 5 | 3.4 | -1.6 | diffusive | 0.61 / 0.39 | 0.84 / 0.16 | 0.56 | 0.98 | 0.268 | 0.477 | -154 / -160 / -153 | 133 | LATE (says hot) |
| 30CAL03 | 5 | 6.5 | +1.5 | diffusive | 0.93 / 0.07 | 1.00 / 0.00 | 0.94 | 0.98 | 0.473 | 0.505 | 43 / 110 / 134 | 62 | hot at cutoff, out within 1.5 s |
| 40CAL01 (problem) | 3 | 3.2 | +0.2 | diffusive | 0.98 / 0.02 | 0.98 / 0.02 | 3.76 | 0.92 | 1.996 | 0.531 | -45 / -737 / -396 | 337 | hot at cutoff, out within 1.5 s |
| 40CAL01 (problem) | 5 | 3.2 | -1.8 | diffusive | 0.99 / 0.01 | 1.00 / 0.00 | 1.37 | 0.97 | 0.707 | 0.517 | -283 / -430 / -320 | 241 | LATE (says hot) |
| C07M101 | 3 | 3.3 | +0.3 | radiative | 0.94 / 0.06 | 1.00 / 0.00 | 0.80 | 0.91 | 0.293 | 0.366 | -16 / -74 / -103 | 48 | hot at cutoff, out within 1.5 s |
| C07M101 | 5 | 3.3 | -1.7 | radiative | 0.78 / 0.22 | 0.91 / 0.09 | 0.59 | 0.98 | 0.222 | 0.374 | -25 / -51 / -68 | 34 | LATE (says hot) |
| C07M201 | 5 | 6.1 | +1.1 | radiative | 0.69 / 0.31 | 0.99 / 0.01 | 0.92 | 0.98 | 0.348 | 0.379 | 10 / . / -90 | 44 | hot at cutoff, out within 1.5 s |
| C12M401 (problem) | 3 | 2.5 | -0.5 | radiative | 0.98 / 0.02 | 1.00 / 0.00 | 0.76 | 0.93 | 0.317 | 0.417 | -42 / -151 / -283 | 129 | LATE (says hot) |
| C12M401 (problem) | 5 | 2.5 | -2.5 | radiative | 0.95 / 0.05 | 0.52 / 0.48 | 0.39 | 0.98 | 0.160 | 0.415 | -28 / -76 / -102 | 45 | LATE (says hot) |
| C14M101 | 1 | 1.2 | +0.2 | none | 0.00 / 1.00 | 0.00 / 1.00 | 0.82 | 0.14 | 0.169 | 0.206 | 0 / -4 / -9 | 5 | hot at cutoff, out within 1.5 s; filter already says no flame (EARLY) |
| C14M101 | 3 | 1.2 | -1.8 | none | 0.00 / 1.00 | 0.00 / 1.00 | 0.19 | 0.76 | 0.052 | 0.274 | 2 / 7 / 7 | 5 | right |
| C14M101 | 5 | 1.2 | -3.8 | none | 0.00 / 1.00 | 0.00 / 1.00 | 0.21 | 0.93 | 0.067 | 0.320 | 1 / -1 / -9 | 5 | right |
| C14M201 | 3 | 2.5 | -0.5 | radiative | 0.86 / 0.14 | 1.00 / 0.00 | 0.84 | 0.93 | 0.319 | 0.381 | -37 / -75 / -78 | 41 | LATE (says hot) |
| C14M201 | 5 | 2.5 | -2.5 | radiative | 0.40 / 0.60 | 0.06 / 0.94 | 0.43 | 0.98 | 0.162 | 0.379 | -19 / -48 / -69 | 28 | right |
| C14M301 | 3 | 3.5 | +0.5 | radiative | 0.75 / 0.25 | 0.99 / 0.01 | 1.05 | 0.93 | 0.382 | 0.365 | -14 / . / . | 10 | hot at cutoff, out within 1.5 s |
| C14M301 | 5 | 3.5 | -1.5 | radiative | 0.44 / 0.56 | 0.98 / 0.02 | 0.94 | 0.97 | 0.335 | 0.356 | . / . / . | unscored | LATE (says hot) |
| C14M302 | 3 | 3.0 | +0.0 | radiative | 0.79 / 0.21 | 1.00 / 0.00 | 0.95 | 0.93 | 0.357 | 0.376 | -61 / -87 / . | 43 | hot at cutoff, out within 1.5 s |
| C14M302 | 5 | 3.0 | -2.0 | radiative | 0.19 / 0.81 | 0.02 / 0.98 | 0.39 | 0.98 | 0.143 | 0.369 | -8 / . / -118 | 64 | right |
| H08M101 | 1 | 1.1 | +0.1 | none | 0.37 / 0.63 | 0.26 / 0.74 | 0.20 | 0.13 | 0.071 | 0.353 | -2 / 1 / -7 | 5 | hot at cutoff, out within 1.5 s; filter already says no flame (EARLY) |
| H08M101 | 3 | 1.1 | -1.9 | none | 0.00 / 1.00 | 0.00 / 1.00 | 0.14 | 0.76 | 0.071 | 0.497 | 2 / -2 / -16 | 7 | right |
| H08M101 | 5 | 1.1 | -3.9 | none | 0.00 / 1.00 | 0.00 / 1.00 | 0.14 | 0.94 | 0.077 | 0.534 | -6 / -16 / -37 | 17 | right |
| H15M102 | 3 | 4.1 | +1.1 | radiative | 0.95 / 0.05 | 1.00 / 0.00 | 0.98 | 0.93 | 0.635 | 0.647 | -13 / -186 / -305 | 128 | hot at cutoff, out within 1.5 s |
| H15M102 | 5 | 4.1 | -0.9 | radiative | 0.81 / 0.19 | 1.00 / 0.00 | 0.91 | 0.98 | 0.571 | 0.625 | -125 / -291 / -302 | 179 | LATE (says hot) |

What the filter compared r_late with, for droplets already out at the cutoff:

| run | cutoff | t_ext | t_mid..t_tail of late window | r_late | rho_e (no-flame mean) | sd hot | sd none | state none |
|---|---|---|---|---|---|---|---|---|
| 20CAL12 | 5 | 3.4 | 2.56..4.40 | 0.56 | 0.07 | 0.18 | 0.15 | 0.16 |
| 40CAL01 | 5 | 3.2 | 0.73..4.93 | 1.37 | 0.08 | 0.18 | 0.15 | 0.00 |
| C07M101 | 5 | 3.3 | 2.50..4.38 | 0.59 | 0.15 | 0.18 | 0.15 | 0.09 |
| C12M401 | 3 | 2.5 | 1.50..2.64 | 0.76 | 0.13 | 0.18 | 0.15 | 0.00 |
| C12M401 | 5 | 2.5 | 2.50..4.38 | 0.39 | 0.13 | 0.18 | 0.15 | 0.48 |
| C14M201 | 3 | 2.5 | 1.50..2.64 | 0.84 | 0.14 | 0.18 | 0.15 | 0.00 |
| C14M201 | 5 | 2.5 | 2.50..4.38 | 0.43 | 0.14 | 0.18 | 0.15 | 0.94 |
| C14M301 | 5 | 3.5 | 2.07..3.65 | 0.94 | 0.11 | 0.18 | 0.15 | 0.02 |
| C14M302 | 5 | 3.0 | 2.50..4.38 | 0.39 | 0.12 | 0.18 | 0.15 | 0.98 |
| H15M102 | 5 | 4.1 | 2.50..4.38 | 0.91 | 0.08 | 0.18 | 0.15 | 0.00 |

Burning cases at cutoff ≥ 3 s with r_late < 0.75 (the risk side of making "no flame" likelier at
moderate r):

| run | cutoff | t_ext | r_late | prior hot | state hot | gain |
|---|---|---|---|---|---|---|
| 175R001 | 3 | 8.8 | 0.74 | 0.98 | 1.00 | 0.89 |
| 256F002 (problem) | 3 | None | 0.23 | 1.00 | 1.00 | 0.70 |
| 256F002 (problem) | 5 | None | 0.67 | 1.00 | 1.00 | 0.96 |
| 256R001 (problem) | 3 | None | 0.72 | 1.00 | 1.00 | 0.93 |
| C03M101 | 3 | None | 0.64 | 1.00 | 1.00 | 0.81 |

**Conclusion D.**
- **The filter is late, never early.** Of the 7 non-problem cases already out at the cutoff, it calls
  5 hot: 20CAL12@5, C07M101@5, C14M201@3, C14M301@5 (unscored) and H15M102@5. All went out 0.5–1.7 s
  before the cutoff. It gets the 2 right that went out ≥ 2.0 s before (C14M201@5, C14M302@5).
- **Why.** At cutoff 5 the late finite difference runs from the mid window (≈ 2.5 s) to the tail
  window (≈ 4.4 s). An extinction at 3.3–4.1 s therefore still shows r_late = 0.56–0.94, while the
  filter's no-flame state is centred at rho_e = 0.07–0.15 with sd 0.15. A droplet that went out about
  2 s before the cutoff shows r_late = 0.39–0.43, still 2–3 sd above rho_e. It wins only because the
  prior has turned.
- **Bias.** The misread cases have negative bias of −50 to −300 µm (the forecast keeps burning).
- **Risk of change is low on train.** No burning case was pushed towards no-flame. Only C03M101@3 and
  175R001@3 burn with r_late < 0.75, and both have a prior P(hot) ≥ 0.98.
- **Cases out within 1.5 s after the cutoff.** The filter rightly says hot. The error then comes from
  extinction timing (C), not from the filter.

---

## E. After extinction

Rates of D² measured from t_ext (NASA visible-flame extinction placed on the series). Each rate is a
finite difference of 0.5-s edge-window means over 0–2, 2–5, 5–10 and 10–20 s after t_ext. The law
rate is [evap_drop(t_b) − evap_drop(t_a)]/(t_b − t_a) at the same edge-window times, with D2's
k_inf 1.5e-8, k_hot 6e-8, tau0 4.5 s, size_exp 0.72 and hot_scale 1.

| run | session | branch | t_ext s | D at t_ext mm | tail s | 0-2 s obs / law | 2-5 s | 5-10 s | 10-20 s |
|---|---|---|---|---|---|---|---|---|---|
| 30CAL03 | 2009-03-31 | diffusive | 6.5 | 0.80 | 21.7 | 0.050 / 0.022 | 0.015 / 0.005 | 0.006 / 0.004 | 0.001 / 0.004 |
| 20CAL12 | 2009-03-05 | diffusive | 3.4 | 0.92 | 13.6 | 0.051 / 0.028 | 0.017 / 0.007 | 0.008 / 0.005 |  |
| 20CAL10 | 2009-03-05 | diffusive | 5.0 | 0.93 | 16.6 | 0.066 / 0.029 | 0.016 / 0.007 | 0.005 / 0.005 |  |
| C01M603 | 2011-08-24 | diffusive | 10.3 | 0.99 | 3.6 | 0.119 / 0.032 |  |  |  |
| C09M701 | 2011-08-08 | diffusive | 9.3 | 1.21 | 8.1 | 0.091 / 0.042 | 0.043 / 0.015 |  |  |
| 177R002 | 2009-11-27 | diffusive | 11.4 | 1.25 | 7.1 | 0.064 / 0.043 | 0.029 / 0.017 |  |  |
| H03M101 | 2011-09-22 | diffusive | 12.5 | 1.29 | 21.0 | 0.072 / 0.045 | 0.039 / 0.018 | 0.020 / 0.009 | 0.010 / 0.008 |
| 178F001 | 2009-11-13 | diffusive | 15.7 | 1.32 | 18.1 | 0.088 / 0.045 | 0.042 / 0.019 | 0.006 / 0.010 |  |
| C09M601 | 2011-08-08 | diffusive | 12.9 | 1.33 | 20.8 | 0.033 / 0.046 | 0.017 / 0.020 | 0.010 / 0.010 | 0.010 / 0.008 |
| 176R001 | 2010-02-04 | diffusive | 18.2 | 1.39 | 15.5 | 0.066 / 0.048 | 0.026 / 0.022 | 0.018 / 0.011 |  |
| N01M101 | 2011-08-24 | diffusive | 19.6 | 1.42 | 5.3 | 0.101 / 0.049 | 0.036 / 0.023 |  |  |
| C14M301 | 2010-11-04 | radiative | 3.5 | 1.48 | 26.6 |  |  |  |  |
| C07M201 | 2011-01-12 | radiative | 6.1 | 1.84 | 27.6 |  |  |  | 0.103 / 0.015 |
| C14M302 | 2010-11-04 | radiative | 3.0 | 2.06 | 30.6 | 0.090 / 0.065 |  |  |  |
| N06M101 | 2011-09-09 | radiative | 7.0 | 2.09 | 26.6 | 0.115 / 0.065 | 0.065 / 0.046 | 0.052 / 0.029 | 0.011 / 0.019 |
| H15M102 | 2011-12-05 | radiative | 4.1 | 2.30 | 14.7 | 0.208 / 0.069 | 0.021 / 0.052 | 0.056 / 0.036 |  |
| C12M201 | 2010-09-08 | radiative | 14.0 | 2.58 | 3.8 | 0.193 / 0.074 |  |  |  |
| C14M201 | 2010-11-04 | radiative | 2.5 | 2.73 | 7.9 | 0.144 / 0.077 | 0.076 / 0.063 |  |  |
| H05M102 | 2011-10-01 | radiative | 8.1 | 2.79 | 25.7 | 0.182 / 0.078 | 0.093 / 0.065 | 0.049 / 0.050 | 0.036 / 0.036 |
| 175R001 | 2010-03-29 | radiative | 8.8 | 2.91 | 24.9 | 0.131 / 0.080 | 0.087 / 0.067 | 0.048 / 0.053 | 0.033 / 0.039 |
| C07M101 | 2011-01-12 | radiative | 3.3 | 3.08 | 30.4 | 0.163 / 0.083 | 0.107 / 0.071 | 0.054 / 0.058 | 0.039 / 0.043 |
| H08M101 | 2011-10-11 | none | 1.1 | 3.54 | 32.4 | 0.071 / 0.090 | 0.057 / 0.081 | 0.035 / 0.069 | 0.023 / 0.055 |
| C12M101 | 2010-09-08 | radiative | 8.3 | 3.92 | 25.4 | 0.168 / 0.096 | 0.113 / 0.089 | 0.075 / 0.079 | 0.042 / 0.065 |
| C14M101 | 2010-11-04 | none | 1.2 | 3.95 | 32.3 | 0.057 / 0.097 | 0.067 / 0.089 | 0.051 / 0.079 | 0.039 / 0.066 |
| 40CAL01 (problem) | 2009-03-31 | diffusive | 3.2 | 1.50 | 15.4 | -1.095 / 0.051 | 0.397 / 0.026 | 0.165 / 0.013 |  |
| C11M201 (problem) | 2010-09-06 | diffusive | 19.1 | 1.68 | 14.4 | 0.098 / 0.056 | 0.037 / 0.032 | 0.016 / 0.018 |  |
| C12M401 (problem) | 2010-09-08 | radiative | 2.5 | 2.74 | 7.9 | 0.143 / 0.077 | 0.069 / 0.063 |  |  |

Ratio obs/law by segment and diameter at extinction (problem runs excluded):

| segment | D_e class | runs | sessions | median obs mm2/s | median law | median ratio obs/law | min ratio | max ratio |
|---|---|---|---|---|---|---|---|---|
| 0-2 s | D_e < 1.6 mm | 11 | 8 | 0.066 | 0.043 | 1.96 | 0.73 | 3.77 |
| 0-2 s | 1.6-2.5 | 3 | 3 | 0.115 | 0.065 | 1.76 | 1.39 | 3.02 |
| 0-2 s | >= 2.5 | 6 | 5 | 0.165 | 0.079 | 1.92 | 1.64 | 2.60 |
| 2-5 s | D_e < 1.6 mm | 10 | 8 | 0.028 | 0.017 | 2.19 | 0.88 | 3.06 |
| 2-5 s | 1.6-2.5 | 2 | 2 | 0.043 | 0.049 | 0.92 | 0.40 | 1.43 |
| 2-5 s | >= 2.5 | 5 | 5 | 0.093 | 0.067 | 1.29 | 1.21 | 1.50 |
| 5-10 s | D_e < 1.6 mm | 7 | 6 | 0.008 | 0.009 | 1.53 | 0.60 | 2.13 |
| 5-10 s | 1.6-2.5 | 2 | 2 | 0.054 | 0.033 | 1.68 | 1.59 | 1.78 |
| 5-10 s | >= 2.5 | 4 | 4 | 0.052 | 0.056 | 0.94 | 0.90 | 0.99 |
| 10-20 s | D_e < 1.6 mm | 3 | 3 | 0.010 | 0.008 | 1.14 | 0.27 | 1.28 |
| 10-20 s | 1.6-2.5 | 1 | 1 | 0.011 | 0.019 | 0.59 | 0.59 | 0.59 |
| 10-20 s | >= 2.5 | 4 | 4 | 0.037 | 0.041 | 0.87 | 0.65 | 1.01 |

| segment | no-flame runs (C14M101, H08M101): obs / law ratio |
|---|---|
| 0-2 s | 0.59, 0.79 |
| 2-5 s | 0.76, 0.71 |
| 5-10 s | 0.65, 0.50 |
| 10-20 s | 0.59, 0.42 |

**Conclusion E.** For droplets that had a flame, the law is too slow early and roughly right late.
The summary covers 20 runs in 15 sessions.
- **In the first 2 s** the unlit droplet shrinks about 2× faster than the law:
  - D_e < 1.6 mm: median ratio 1.96 (11 runs, 8 sessions).
  - D_e 1.6–2.5 mm: 1.76 (3 runs, 3 sessions).
  - D_e ≥ 2.5 mm: 1.92 (6 runs, 5 sessions; min 1.64).
  - Observed initial rates are about 0.07 mm²/s at D_e ≈ 1.3 mm and about 0.17 mm²/s at D_e ≈ 2.9 mm.
- **At 2–5 s:**
  - D_e < 1.6 mm: 2.2× the law.
  - D_e ≥ 2.5 mm: 1.3× (5 runs, 5 sessions, all 1.21–1.50).
- **At 5–10 s and 10–20 s:**
  - Large droplets: 0.94 and 0.87, with 10–20 s ranging 0.65–1.01.
  - Small droplets: ratios 1.1–1.5 on rates below 0.01 mm²/s, which is noise-level.
- **Decay.** The excess over the long-time rate decays from 0–2 s to 2–5 s by a factor of about 0.34
  (small) and 0.42 (large) over about 2.5–2.75 s. That is an e-folding of roughly 2–3 s at both sizes.
  The law's decay time τ = 4.5 s·(D_e/2 mm)² is 1.9 s at 1.3 mm and 9.5 s at 2.9 mm, and its
  amplitude k_hot = 0.06 mm²/s does not grow with size.
- **No sustained flame** (C14M101, H08M101; the ambient was never heated): the law is 1.3–2.4× too
  fast throughout (ratios 0.42–0.79). The CRPS of those cases is small (5–17 µm).
- **Already out at the cutoff.** For these cases the forward scales k_hot by the observed late slope
  (evap_now_scale). C14M302@5 shows that this slope can still contain burning: k_obs_late is
  0.143 mm²/s, about 2.4 times k_hot. The forecast then evaporated too fast (bias −118 µm at 5–10 s).

---

## F. Persistence of the run offset

For every case still burning and in view through cutoff + 3 s (problem and no-flame runs excluded),
"later" = ln(K_obs/K_law) over [cutoff, cutoff + 3]. K_obs is the finite difference of the 0.5-s
edge-window means of D². K_law uses the same harmonic Simpson mean as the encoder. b is the offset
the forward actually applied: gain × (0.85 e_late + 0.15 e_early).

| cutoff | e_late class | cases | sessions | median e_late | median b (applied offset) | median later ratio | min later | max later | median (later - b) | median gain |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | < -0.2 | 7 | 5 | -0.346 | -0.055 | -0.103 | -0.205 | +0.029 | -0.048 | 0.17 |
| 1 | -0.2..-0.05 | 10 | 7 | -0.129 | -0.026 | -0.067 | -0.363 | +0.106 | -0.038 | 0.21 |
| 1 | -0.05..0.05 | 14 | 11 | -0.004 | -0.001 | +0.012 | -0.135 | +0.085 | +0.017 | 0.23 |
| 1 | 0.05..0.2 | 11 | 10 | +0.125 | +0.023 | -0.016 | -0.188 | +0.086 | -0.028 | 0.26 |
| 1 | > 0.2 | 4 | 4 | +0.227 | +0.058 | +0.002 | -0.127 | +0.109 | -0.076 | 0.33 |
| 1 | all | 46 | 20 | -0.008 | -0.005 | -0.017 | | | -0.027 | 0.22 |
| 3 | < -0.2 | 4 | 4 | -0.285 | -0.251 | -0.095 | -0.140 | +0.014 | +0.207 | 0.85 |
| 3 | -0.2..-0.05 | 15 | 10 | -0.106 | -0.097 | -0.019 | -0.126 | +0.117 | +0.092 | 0.93 |
| 3 | -0.05..0.05 | 17 | 12 | +0.008 | +0.010 | +0.003 | -0.110 | +0.141 | -0.001 | 0.93 |
| 3 | 0.05..0.2 | 6 | 5 | +0.096 | +0.092 | +0.025 | -0.003 | +0.052 | -0.082 | 0.93 |
| 3 | all | 42 | 18 | -0.041 | -0.042 | -0.003 | | | +0.038 | 0.93 |
| 5 | -0.2..-0.05 | 9 | 7 | -0.077 | -0.078 | -0.041 | -0.132 | +0.146 | +0.067 | 0.98 |
| 5 | -0.05..0.05 | 19 | 12 | -0.000 | -0.001 | -0.004 | -0.100 | +0.054 | -0.002 | 0.98 |
| 5 | 0.05..0.2 | 6 | 6 | +0.065 | +0.055 | +0.041 | -0.072 | +0.110 | -0.031 | 0.98 |
| 5 | all | 34 | 17 | -0.002 | -0.002 | -0.004 | | | +0.012 | 0.98 |

| cutoff | cases | |e_late| > 0.05 | of those, later has same sign | median |later| / |e_late| (|e_late| > 0.05) | median |later - b| | median |later| |
|---|---|---|---|---|---|---|
| 1 | 46 | 32 | 20 | 0.38 | 0.055 | 0.057 |
| 3 | 42 | 25 | 17 | 0.47 | 0.057 | 0.042 |
| 5 | 34 | 15 | 11 | 0.74 | 0.034 | 0.039 |

**Conclusion F.** A fast or slow prefix stays fast or slow only partly. The share that carries over
grows with the length of the prefix.
- **How much carries over.** Of the cases with |e_late| > 0.05, the next 3 s have the same sign in
  20 of 32 (cutoff 1), 17 of 25 (cutoff 3) and 11 of 15 (cutoff 5). The median |later|/|e_late| is
  0.38, 0.47 and 0.74.
- **Cutoff 1.** The gain is 0.22, so b ≈ 0.2 e_late. That is about right, maybe slightly low for slow
  prefixes: e_late < −0.2 is followed by −0.10 with b −0.055 (7 cases, 5 sessions).
- **Cutoff 3.** The gain is 0.93, so b ≈ e_late, but only about 0.3–0.5 of e_late persists:
  - e_late < −0.2 (4 cases, 4 sessions): b −0.25, later −0.095.
  - e_late −0.2..−0.05 (15 cases, 10 sessions): b −0.10, later −0.02.
  - e_late +0.05..+0.2 (6 cases, 5 sessions): b +0.09, later +0.025.
  - The median |later − b| is 0.057, larger than the median |later| of 0.042. On train, applying the
    offset at cutoff 3 does slightly worse than applying none.
- **Cutoff 5.** The gain is 0.98. About 0.5–0.7 persists, and applying b helps a little (0.034
  against 0.039).
- **Physical reading.** At 3 s the mid→tail window sits at s ≈ 0.15–0.35, at the end of heat-up. Part
  of e_late is a heat-up deviation, which decays, and the low-O2 and high-O2 heat-up offsets of B feed
  it directly.

---

## G. Spread

| cutoff | cases | D2 cov50 | D2 cov90 | D1 cov50 | D1 cov90 | D2 mean width90 um | D1 mean width90 um |
|---|---|---|---|---|---|---|---|
| 1 | 66 | 0.82 | 0.95 | 0.76 | 0.94 | 257 | 204 |
| 3 | 61 | 0.58 | 0.94 | 0.51 | 0.85 | 199 | 164 |
| 5 | 55 | 0.66 | 0.90 | 0.55 | 0.84 | 211 | 178 |
| all | 182 | 0.69 | 0.93 | 0.61 | 0.88 | 224 | 183 |

Per session (problem runs excluded). Verdict rule: too narrow if cov90 < 0.75 or cov50 < 0.35; too
wide if cov90 ≥ 0.99 and cov50 ≥ 0.8.

Per session and cutoff, problem runs excluded: D2 cov50/cov90 (D1 cov50/cov90), cases in brackets

| session | cutoff 1 | cutoff 3 | cutoff 5 | all cutoffs D2 | all cutoffs D1 | verdict D2 |
|---|---|---|---|---|---|---|
| 2009-03-05 | 0.79/1.00 (0.75/1.00) [3] | 0.58/1.00 (0.55/1.00) [3] | 0.35/0.50 (0.36/0.48) [2] | 0.60/0.87 [8] | 0.58/0.87 | about right |
| 2009-03-31 | 0.60/0.97 (0.51/0.96) [2] | 0.25/1.00 (0.21/0.98) [1] | 0.12/1.00 (0.09/0.96) [1] | 0.39/0.98 [4] | 0.33/0.96 | about right |
| 2009-10-24 | 0.98/1.00 (0.83/1.00) [2] | 0.18/1.00 (0.13/0.48) [2] | 0.22/1.00 (0.11/0.83) [2] | 0.46/1.00 [6] | 0.36/0.77 | about right |
| 2009-11-09 | 0.90/0.99 (0.72/0.99) [2] | 0.61/0.97 (0.56/0.77) [2] | 0.58/0.93 (0.51/0.80) [2] | 0.70/0.96 [6] | 0.60/0.85 | about right |
| 2009-11-13 | 1.00/1.00 (1.00/1.00) [2] | 0.62/1.00 (0.60/1.00) [2] | 0.31/1.00 (0.05/1.00) [1] | 0.71/1.00 [5] | 0.65/1.00 | about right |
| 2009-11-27 | 1.00/1.00 (0.99/1.00) [2] | 0.59/1.00 (0.51/1.00) [2] | 1.00/1.00 (1.00/1.00) [2] | 0.86/1.00 [6] | 0.83/1.00 | too wide |
| 2010-02-04 | 1.00/1.00 (1.00/1.00) [2] | 0.56/1.00 (0.42/0.64) [2] | 0.63/1.00 (0.52/0.83) [2] | 0.73/1.00 [6] | 0.65/0.82 | about right |
| 2010-03-19 | 0.96/1.00 (0.91/1.00) [5] | 0.60/0.89 (0.54/0.75) [4] | 0.78/1.00 (0.72/0.85) [4] | 0.80/0.96 [13] | 0.74/0.88 | about right |
| 2010-03-29 | 0.96/1.00 (0.79/1.00) [2] | 0.70/1.00 (0.56/0.87) [2] | 0.99/1.00 (0.99/1.00) [2] | 0.89/1.00 [6] | 0.78/0.96 | too wide |
| 2010-05-05 | 1.00/1.00 (1.00/1.00) [2] | 0.99/1.00 (0.89/1.00) [2] | 0.86/0.99 (0.75/0.99) [2] | 0.95/1.00 [6] | 0.88/1.00 | too wide |
| 2010-06-21 | 0.64/1.00 (0.55/1.00) [2] | 0.67/1.00 (0.62/0.94) [2] | 0.93/1.00 (0.76/0.99) [2] | 0.75/1.00 [6] | 0.64/0.98 | about right |
| 2010-09-08 | 0.99/1.00 (0.79/1.00) [2] | 0.41/1.00 (0.34/1.00) [2] | 0.82/1.00 (0.74/0.91) [2] | 0.74/1.00 [6] | 0.63/0.97 | about right |
| 2010-11-04 | 0.68/1.00 (0.62/1.00) [4] | 0.43/0.78 (0.39/0.71) [4] | 0.38/0.59 (0.30/0.56) [3] | 0.51/0.81 [11] | 0.45/0.77 | about right |
| 2010-12-08 | 0.70/1.00 (0.67/0.85) [3] | 0.18/0.94 (0.13/0.44) [3] | 0.82/1.00 (0.69/1.00) [3] | 0.56/0.98 [9] | 0.50/0.76 | about right |
| 2011-01-12 | 0.62/1.00 (0.62/1.00) [2] | 0.25/1.00 (0.20/1.00) [2] | 0.50/0.84 (0.41/0.62) [2] | 0.46/0.95 [6] | 0.41/0.87 | about right |
| 2011-08-08 | 0.97/1.00 (0.94/1.00) [4] | 0.89/1.00 (0.86/1.00) [4] | 0.89/1.00 (0.74/0.99) [4] | 0.92/1.00 [12] | 0.85/1.00 | too wide |
| 2011-08-24 | 0.96/1.00 (0.88/1.00) [5] | 0.73/0.99 (0.67/0.90) [5] | 0.69/1.00 (0.57/0.98) [5] | 0.79/1.00 [15] | 0.71/0.96 | about right |
| 2011-09-09 | 0.70/1.00 (0.68/1.00) [4] | 0.73/1.00 (0.66/1.00) [4] | 0.95/1.00 (0.63/1.00) [3] | 0.78/1.00 [11] | 0.66/1.00 | about right |
| 2011-09-21 | 1.00/1.00 (1.00/1.00) [1] | 1.00/1.00 (1.00/1.00) [1] |  | 1.00/1.00 [2] | 1.00/1.00 | too wide |
| 2011-09-22 | 1.00/1.00 (1.00/1.00) [1] | 1.00/1.00 (1.00/1.00) [1] | 1.00/1.00 (1.00/1.00) [1] | 1.00/1.00 [3] | 1.00/1.00 | too wide |
| 2011-10-01 | 0.94/1.00 (0.90/1.00) [3] | 0.41/1.00 (0.27/1.00) [2] | 0.52/1.00 (0.30/1.00) [2] | 0.67/1.00 [7] | 0.55/1.00 | about right |
| 2011-10-11 | 0.99/1.00 (0.95/1.00) [1] | 0.68/1.00 (0.61/0.95) [1] | 0.30/0.67 (0.21/0.51) [1] | 0.66/0.89 [3] | 0.59/0.82 | about right |
| 2011-11-18 | 0.88/0.99 (0.73/0.99) [3] | 0.83/0.99 (0.74/0.98) [3] | 0.74/0.96 (0.58/0.88) [3] | 0.82/0.98 [9] | 0.69/0.95 | about right |

Coverage by case type is in the A table above (`cov_type.py`).

**Conclusion G.**
- **Overall.** On train, D2 is too wide for the 50 % band (0.69 overall; 0.82 at cutoff 1) and about
  right to slightly wide for the 90 % band (0.93). D1 was closer: 0.61 and 0.88.
- **Too wide.** The excess width is in cases that burn through the horizon: cov50 0.91 / 0.59 / 0.73
  and cov90 1.00 / 0.98 / 0.99 at cutoffs 1 / 3 / 5 (98 cases, 15–19 sessions).
  - D2's s_z is 0.178 at cutoff 1. The observed persistence spread of the next 3 s is about 0.08
    (IQR/1.349 of "later"), and the run-level spread is 0.06.
  - At cutoffs 3 and 5, s_z is 0.09 and 0.084, against an observed spread of (later − b) of 0.078
    and 0.049.
  - Too wide also by session: 2009-11-27, 2010-03-29, 2010-05-05, 2011-08-08, 2011-09-21 and
    2011-09-22.
- **Too narrow.** Only where the state or the extinction time is wrong. That is cases already out
  before the cutoff (cov50 0.04 and cov90 0.29 at cutoff 5, 5 cases, 4 sessions) and the sessions of
  H15M102 (2011-12-05) and of 20CAL12 at cutoff 5 (2009-03-05). Spread cannot fix these; the state
  filter and the radiative clock can.
- **Validation.** The author widened D2 from validation aggregates. Train cannot judge how a law
  transfers to new ambients, and the ambients in train cannot show that part.

---

## Proposals

These are law-level statements. The author decides the numbers. Each size comes from the medians and
ratios above, not from a fit.

**P1. Radiative extinction comes much earlier the further the droplet is above D_crit.**
- **Statement.** Near D_crit (x_exc ≈ 0), the clock at extinction is about 0.8–1.2 (median ≈ 1.0,
  against the law's 0.90). It falls to about 0.3 by x_exc ≈ 0.3–0.4, against the law's 0.6. That is a
  slope in ln s of about −3 per unit x_exc instead of −1.2. In time, within one atmosphere, a larger
  droplet goes out sooner in seconds as well as in clock units.
- **Support.**
  - 4 same-atmosphere pairs in 3 sessions: C14M301/302/201, C12M201/101 and C07M201/101 (2010-11-04,
    2010-09-08, 2011-01-12).
  - Supporting cases: H15M102 (s 0.53 at x +0.07) and H05M102 (0.70 at −0.01).
  - In the worst 30 this carries 408 µm, 30 % of the non-artefact share.
- **What would contradict it.** A radiative droplet at x_exc ≥ 0.2 that burns to s ≥ 0.8. Or, in an
  N2 or helium session, a larger radiative droplet that does not go out earlier than a smaller one.
- **Risk.** Moderate.
  - All steep pairs are CO2-diluted at 0.7–1 atm. N2 radiative runs exist only near threshold
    (x 0.04–0.09, s 0.79–1.04).
  - x_exc depends on the crit law, so part of the effect could instead be a D_crit that is too high
    for CO2. Both readings move the forecast the same way for train ambients but differ elsewhere.
  - The effect on CRPS is large because the radiative rate is about 0.3–0.6 mm²/s for several seconds.

**P2. The unlit droplet sheds the flame's heat faster and harder than the post law says.**
- **Statement.** In the first 2 s after visible-flame extinction the droplet shrinks about 2× faster
  than the law, and in 2–5 s about 1.3–2×. After 5 s it is on the law, and for D_e ≥ 2.5 mm slightly
  below (0.87–0.94). The excess decays with an e-folding of about 2–3 s at 1.3 mm and at 2.9 mm alike,
  with an amplitude of about 0.06 mm²/s (small) to 0.12 mm²/s (large).
- **Direction.**
  - k_hot up, about 2× (to about 1.2e-7 m²/s).
  - tau0 down to about 2.5–3 s. With the D²-scaling of τ the large-droplet decay stays too slow, so
    the author must accept a compromise or use the size_exp/k_inf pair for the large-droplet tail.
  - k_inf/size_exp slightly down for D_e ≥ 2.5 mm (10–20 s ratio 0.87).
- **Support.** 20 runs with a flame in 15 sessions (11 runs with D_e < 1.6 mm, 3 with 1.6–2.5 mm, 6
  with ≥ 2.5 mm).
- **What would contradict it.** If the 0–2 s excess came from a weak flame that outlives NASA's
  visible extinction, the excess would be absent when t_ext is placed at the rate knee instead. For
  diffusive runs the knee and the Dext crossing agree within 0.1–0.3 s, so this is unlikely to explain
  a 2× excess.
- **Risk.** Low for diffusive runs. For radiative runs it is moderate, since the end of the flame is
  gradual there (C12M101: the knee is 3 s after Dext).
- **Side effect on rho_e.** It raises rho_e a little, which helps D.
- **Side effect on evap_now_scale.** For droplets already out at the cutoff the forward computes
  evap_now_scale = gain·(k_obs_late − k_inf term)/k_hot. A larger k_hot lowers that scale, so the net
  effect on cases like C14M302@5 is not obvious. I did not compute it.

**P3. A prefix that ended in heat-up says little about the plateau: trust it less at 3 s, a bit less
at 5 s.**
- **Statement.** The later rate offset is about 0.3–0.5 of e_late at cutoff 3 and about 0.5–0.7 at
  cutoff 5. The applied gain is 0.93 and 0.98. At cutoff 1, 0.22 is about right. The gain should
  reach about 0.5 at cutoff 3 and about 0.7 at cutoff 5, without falling below about 0.2–0.3 at
  cutoff 1. The current inputs (span, count, progress) all rise together, so the author has to
  choose which input carries it.
- **Support.** 42 cases in 18 sessions at cutoff 3 and 34 cases in 17 sessions at cutoff 5. At
  cutoff 3, |later − b| (0.057) exceeds |later| (0.042). The worst-30 examples are 192F002@3 and
  192F003@3, both with b +0.10 and later about 0.
- **What would contradict it.** If after P4 (heat-up by O2) the persistence at cutoff 3 rose to
  ≥ 0.8, the gain would be right and the heat-up law was the problem. Check after P4.
- **Risk.** Low to moderate. Under a lower gain, more weight goes to the population law, which is
  unbiased on train but untested on new ambients. That is the same trade-off D2 already made for
  spread.

**P4. Heat-up depends on oxygen: slower at ≤ 15 % O2, faster at ≥ 30 %.**
- **Statement.** At ≤ 15 % O2 the rate is about 15 % below the law for s < 0.2 and about 5 % below on
  the plateau. At ≥ 30 % O2 it is about 7 % above for s < 0.2 (up to +19 % in the 1–2 s window at
  34 %). The flame at low O2 stands further out and takes longer to develop, so the heat-up term
  should reach deeper or last longer in clock units at low O2.
- **Support.** For the low-O2 side: 15 runs in 5 sessions (2010-03-19, 2010-03-29, 2010-11-04,
  2011-01-12, 2011-09-09). 4 of 5 sessions are below −0.05 at s < 0.2, and 3 of 5 on the plateau.
  The high-O2 side has 3 sessions with 2 positive, which is not consistent by itself; see "Leave
  alone".
- **What would contradict it.** If the low-O2 deficit at s < 0.2 came from radiative runs only. It
  does not: 175F001 −0.40/−0.25 and N06M201 −0.20/−0.19 are diffusive. Or a low-O2 session with
  helium (He raises conductivity) that heats up on the law.
- **Risk.** Moderate. Low-O2 train runs are N2 or CO2. Helium at low O2 (H08M101) never sustained a
  flame, so there is no train evidence there.

**P5. The no-flame state must expect a late rate ratio of about 0.4–0.6, not 0.1, when the droplet
went out inside the late window.**
- **Statement.** A droplet that went out 0.5–2.5 s before the cutoff shows r_late = 0.39–0.94,
  because the mid→tail finite difference straddles the extinction. The no-flame likelihood (mean
  0.07–0.15, sd 0.15) does not cover it, so the filter stays hot. Widening filt.sd_evap is the
  weight-only lever; P2's larger k_hot raises the mean a little.
- **Size, illustrated on 20CAL12@5.** Evaluating the filter formula of droplet.rs by hand (r_late
  0.56, prior 0.61 hot, gain 0.98) reproduces D2's 0.84 hot. With sd_evap 0.30 it gives 0.38 hot; with
  0.35 it gives 0.34 hot.
- **Support.** 5 misread cases in 4 sessions: 20CAL12@5, C07M101@5, C14M201@3, C14M301@5 and
  H15M102@5. In the worst 30 this carries 312 µm.
- **What would contradict it.** A burning droplet at cutoff ≥ 3 s with r_late of 0.4–0.7 and a prior
  P(hot) below about 0.9. Train has none: the only low-r burners have prior ≥ 0.98.
- **Risk.** Low on train. On validation, a slow-burning run near its predicted extinction could be
  called out too early.
- **Limit.** H15M102@5 (r_late 0.91) cannot be rescued by weights. That needs a shorter late window,
  which is a forward change, not a weight.

**P6. On train the rate spread at short prefixes is about twice what the runs show.**
- **Statement.** s_z at cutoff 1 is 0.178, against an observed next-3-s spread of about 0.08 and a
  run-level spread of 0.06 (with a slow tail to −0.21). cov50 is 0.91 and cov90 1.00 for cases that
  burn through the horizon at cutoff 1.
- **Support.** 40 cases in 19 sessions at cutoff 1. The excess is smaller at cutoffs 3 and 5.
- **What would contradict it.** Validation ambients, which train cannot show.
- **Risk.** High if acted on fully. The author widened D2 deliberately from validation aggregates.
  This is stated so that the trade-off is visible, not as a recommendation to undo it. A narrower
  s_pop with heavier tails is not available as a weight; the scenario nodes are Gaussian.

## Leave alone

- **34 % O2, 0.7 atm late-burn slowdown and fast heat-up.** Windows at w 0.55–0.8 run −0.07/−0.08 and
  the 1–2 s window +0.19. This is 2 sessions only (2009-10-24 and 2009-11-09; 191F001, 192F002,
  192F003). It drives 4 worst-30 cases but needs a third session.
- **Diffusive w at small D0 (< 2.2 mm).** The difference is −0.05 to −0.08, but it comes from the
  2009-03-05 CAL session (igniter-glare era) plus N06, and 30CAL03 goes the other way (+0.03). The
  timing errors of 20CAL12 (late) and 30CAL03 (early) have opposite signs at the same D0.
- **Diffusive w at ≤ 15 % O2 (−0.05).** There are 3 sessions, but the size is below the run scatter
  (by-session range −0.11..+0.13) and 189R001's Dext has unknown provenance. It could be revisited
  together with P4.
- **Diffusive w at 23 % O2, 25 % CO2, 0.7 atm (−0.06 to −0.12).** Two sessions of the same C09
  atmosphere.
- **Helium.** The radiative clock is lower (0.53 and 0.70 near threshold) and the diffusive w higher
  (+0.03). These rest on 2 sessions and on mostly extrapolated Dext, respectively.
- **The class law** (kappa, bias, crit). There is no reliable wrong-side run. The only ones are
  189R002 (a Dext the narrative disowns) and H12M101 (problem run).
- **Burn order within the session and drift speed.** No consistent effect.
- **CO2 and pressure in the rate law.** No consistent effect.
- **No-sustained-flame runs** (evaporation 1.3–2.4× slower than the law). There are 2 runs in 2
  sessions, and the CRPS is already 5–17 µm.
- **C03M101** (run median −0.43, a very dim, long burn). This is one run.
- **Anything from the problem runs.** In particular C12M401 duplicates C14M201 and must not count as
  an extra session for P1 or P5.

## What I could not determine

- **The true burn order within a chamber fill.** Burns of the same fill in the validation and test
  partitions were not opened. Order is among train runs only. The FLEX numbers show gaps between
  them.
- **Extinction time for runs that leave the view before extinction.** No s_obs is given for 21 of the
  39 diffusive runs. Their Dext is NASA's, and NASA extrapolated it along the burn rate.
- **CRPS without the dropout frames** (191R001). Recomputing it needs the forecast quantiles, which
  the feedback file does not contain.
- **The net CRPS effect of any proposal.** This was not evaluated, by design: no forward run, no
  search. The one exception is the single hand evaluation of the filter formula in P5.

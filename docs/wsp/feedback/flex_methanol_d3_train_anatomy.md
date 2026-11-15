# Methanol droplet model D3: error anatomy on the FLEX train data

> Train partition only; nothing fitted. Written by a delegated analysis agent (claude-opus-5-5) on 14 November
> 2026; the author rechecked the claims it acted on against the train tables. Paths under `scratchpad` are the
> agent's working folder and are not part of the repository.

I re-ran D3 at 120 truncated forecast grids (fb/cdf_c{1,3,5}_h{1..40}.json). Truncating at 10 s reproduces the stored case CRPS exactly (max difference 0.0 µm). From those runs I got the CRPS per lead band and the forward's own extinction-time distribution, P(hot flame ended by cutoff + h). Everything below is train only. Nothing was fitted and no repository file was changed.

**Results in one paragraph.**
- Artefacts carry 39 % of train CRPS.
- 5-10 s leads carry 47 % of all CRPS; leaving artefacts out they are 62 % of what remains.
- Little of the remaining error can be reached from the inputs the model has:
  - The 0–2 s CRPS is the assumed 20 µm measurement noise, not real error.
  - Two artefact runs can be seen in the prefix.
  - Radiative timing comes late because of the class mixture, not the clock law.
  - At 5 s the end rate carries forward and D3 under-uses it.
  - The rate plateau at 1 s cannot be reached from the prefix.

Case classes come from the hand-read extinction times of the D2 audit. "Artefact" means the eight problem runs, plus 191R001 at 1 and 3 s.

## 1. Error anatomy

Total train CRPS is 5436 µm summed over 182 cases (case mean 29.9).

| class | cut 1: n / share / mean | cut 3 | cut 5 | all |
|---|---|---|---|---|
| burning through horizon | 39 / 11.4 % / 15.8 | 31 / 8.8 % / 15.4 | 26 / 7.4 % / 15.6 | 96 / 27.6 % / 15.6 |
| extinguishes in horizon | 18 / 7.2 % / 21.8 | 21 / 10.3 % / 26.7 | 18 / 9.8 % / 29.5 | 57 / 27.3 % / 26.0 |
| already out at cutoff | – | 2 / 1.0 % / 28.1 | 6 / 3.9 % / 35.2 | 8 / 4.9 % / 33.4 |
| no sustained flame (C14M101, H08M101) | 2 / 1.0 % | 2 / 0.2 % | 2 / 0.2 % | 6 / 1.4 % / 12.8 |
| artefact | 7 / 22.5 % / 175 | 5 / 10.8 % / 118 | 3 / 5.5 % / 99 | 15 / 38.8 % / 141 |

By lead band (181 cases; one case has no frames at 0–2 s):

| class | share of all CRPS, 0-2 / 2-5 / 5-10 s | CRPS per frame (µm) | cov50 | cov90 | width50 (µm) |
|---|---|---|---|---|---|
| burning | 4.4 / 7.7 / 15.4 % | 6.3 / 13.2 / 27.4 | 0.92 / 0.83 / 0.76 | 1.00 / 0.99 / 1.00 | 31 / 56 / 115 |
| extinguishes | 2.1 / 5.7 / 18.8 % | 7.5 / 18.8 / 42.9 | 0.93 / 0.84 / 0.71 | 1.00 / 1.00 / 1.00 | 35 / 77 / 159 |
| out at cutoff | 1.0 / 1.5 / 2.4 % | 21 / 37 / 52 | 0.47 / 0.35 / 0.23 | 0.76 / 0.75 / 0.69 | |
| artefact | 18.5 / 11.2 / 9.0 % | 84 / 195 / 204 | 0.50 / 0.29 / 0.22 | 0.65 / 0.36 / 0.58 | |
| without artefacts | 7.7 / 15.3 / 37.5 % | 7.4 / 16.2 / 34.0 | 0.90 / 0.80 / 0.70 | 0.99 / 0.97 / 0.97 | 33 / 64 / 129 |

The 20 worst cases carry 2617 µm (48 %). Δt is the implied median extinction time minus the hand-read one.

| # | case | CRPS | cause |
|---|---|---|---|
| 1 | 240R003@1 | 576 | artefact: bubble; 20 frames gated after 0.2 s in the prefix |
| 2–3 | 40CAL01@3, @5 | 347, 241 | artefact: leaves and re-enters the view; 34 and 99 frames gated; b = +0.26 comes from the tracker |
| 4, 7 | C12M401@1, @3 | 167, 159 | artefact: duplicate video under the wrong atmosphere; prefix clean |
| 5 | H12M101@1 | 165 | artefact: leaves the view at 2.1 s; prefix clean |
| 6 | 40CAL01@1 | 159 | artefact; prefix clean at 1 s |
| 8 | H15M102@5 | 122 | wrong state: out 0.9 s before the cutoff; prior P(hot) 0.43 was pushed to 0.92 because r_end = 0.74 straddles the extinction |
| 9 | 256F002@1 | 106 | artefact: saturated backlight; prefix frame scatter 5.3 µm against a clean median of 1.2 |
| 10–11 | 20CAL12@3, @1 | 91, 59 | diffusive extinction late by +0.65 / +0.69 s (w_obs 0.68 vs w_D 0.79) |
| 12 | 20CAL10@5 | 53 | out exactly at the cutoff (5.0 s); filter says hot 0.99; cannot be reached |
| 13 | 191R001@3 | 53 | artefact: dropout frames inside the scored window |
| 14 | H05M102@5 | 52 | radiative implied +2.3 s late; the s_R law is only −0.08 off in ln; 39 % of the mass sits on the diffusive branch |
| 15, 20 | C01M603@5, @3 | 47, 40 | diffusive early (−0.74 / −0.65 s): the forecast stops shrinking too soon (bias +151 µm at 5–10 s) |
| 16, 18 | 30CAL03@5, @3 | 46, 45 | diffusive early (−0.15 / −0.27 s), +90 to +107 µm at 5–10 s |
| 17 | 175R001@5 | 46 | radiative implied +3.7 s: s_R 1.0 vs observed 0.79, and 26 % diffusive branch |
| 19 | C09M701@5 | 42 | diffusive late +1.07 s (w_obs 0.76 vs 0.83) |

## 2. Extinction timing

**How I got the implied time.** It is the median lead at which P(hot flame ended) from the forward reaches 0.5. Disruption mass is left out of that probability (λ ≈ 0.008/s), which biases the median late by ≤ 0.1 s. Observed times are the audit's direct Dext crossings. Clock unit: Δs = Δt / D0² with NASA's D0.

| branch | cut | cases (runs / sessions) | Δt median (IQR), s | mean \|Δt\| | Δs median (IQR) | PIT median |
|---|---|---|---|---|---|---|
| diffusive | 1 | 8 (8 / 5) | +0.57 (−0.19..+0.67) | 0.64 | +0.157 | 0.28 |
| | 3 | 13 (13 / 7) | +0.22 (−0.23..+0.67) | 0.54 | +0.028 | 0.45 |
| | 5 | 11 (11 / 7) | +0.09 (−0.14..+0.65) | 0.51 | +0.012 | 0.50 |
| radiative | 1 | 10 (10 / 7) | +0.57 (−0.02..+0.80) | 0.73 | +0.083 | 0.28 |
| | 3 | 8 (8 / 7) | +0.92 (+0.36..+2.04) | 1.36 | +0.101 | 0.20 |
| | 5 | 7 (7 / 6) | +0.67 (−0.22..+2.01) | 1.42 | +0.103 | 0.28 |

- **Diffusive.** No overall bias at 3 and 5 s, and the forward's spread is right (no PIT below 0.1 or above 0.9). The error is session scatter in the progress at extinction (Δw sd about 0.055):
  - late in 2009-03-05 (three CAL runs, −0.10 to −0.12), 2011-08-08 (C09M601/701) and 189R003;
  - early in C01M603, N01M201, 30CAL03 and 053R001.
  - Neither prefix trend nor curvature orders it (section 3).
- **Radiative.** The s_R law itself is unbiased: the median of ln(s_obs/s_R) over 12 runs is −0.03, sd about 0.14. The late implied time comes from the soft class:
  - For droplets near D_crit (x_exc −0.02 to +0.09: H05M102, 175R001, C07M201, C12M201, N06M101) the probability of the radiative class given a hot flame is only 0.5–0.75.
  - The diffusive branch would burn for much longer. That pulls the median late and makes the band bimodal (90 % width 419 µm at 5–10 s, cov90 1.00).
- **How much is timing.** Cases that go out in the horizon carry 1484 µm (27 % of the total). 69 % of that sits in frames after the earlier of the observed and implied extinction. Their excess over burning cases at the same leads is 427 µm. That excess is 3 µm per case when |Δt| < 0.5 s and 11 when |Δt| is 0.5–1.5 s. So about 270 µm (≈ 5 % of train CRPS, about 1.5 µm of the case mean) is timing. The rest is rate drift that would have happened anyway.

## 3. Precursors in the prefix (terciles, cases that are not artefacts and are burning at the cutoff)

The table gives, per tercile of the prefix quantity, the law-relative rate over the next 3 s, ln(K_obs/K_law), from my re-implementation of D3's rate network. It reproduces the encoder's q0 exactly.

| cutoff | prefix quantity | low / mid / high tercile of the quantity | rate over the next 3 s |
|---|---|---|---|
| 5 | ln r_end | −0.042 / +0.011 / +0.056 | **−0.047 / +0.016 / +0.050** (9–10 sessions each) |
| 5 | e_late | −0.069 / −0.001 / +0.056 | −0.024 / +0.011 / +0.019 |
| 3 | ln r_end | −0.120 / −0.005 / +0.075 | −0.011 / −0.006 / +0.031 |
| 3 | e_late | −0.115 / −0.015 / +0.083 | −0.012 / −0.006 / +0.031 |
| 1 | either | | no ordering |

- **At 5 s the end rate carries almost one for one into the next 3 s.** The mid-to-tail innovation carries only about 0.35. D3 builds the offset from e_late, so the top r_end tercile is still under-applied by +0.028 after b.
- **At 3 s the persistence is 0.1–0.4. At 1 s it is nothing.**
- **The trend ln(r_end/r_late) and the curvature of D² say nothing beyond that.** Their terciles give no monotone ordering of the later bias, of the time left to extinction, or of Δt at any cutoff.

## 4. Artefacts seen from the prefix

I re-implemented the validity gate; it matches the forward's dropped-frame count in all 182 cases.

| rule | artefact cases where it fires (CRPS) | clean cases where it fires (of 167) |
|---|---|---|
| R1: frames dropped by the gate at t ≥ 0.2 s ≥ 3 | 240R003@1, 40CAL01@3, 40CAL01@5 (1164 µm) | 0 |
| R2: 0.2-s window mean of kept D² rises > 2 % above an earlier window after 0.2 s | 256F002@3, @5 (56 µm) | 0 |
| R1 or R2 | 5 cases, 1220 µm of the 2109 artefact total | 0 |

- **Not visible in the prefix:** C12M401 (wrong atmosphere), H12M101 (leaves the view later), 191R001 (future dropouts), 40CAL01@1, 256R001.
- **Frame-jump and frozen-value tests also fire on clean runs:** the CAL igniter glare before 0.2 s, and 191R001@5, whose prefix dropouts the gate already handles. That is why both rules start at 0.2 s and use the kept frames.
- **Coverage:** three sessions (2009-07-01, 2009-03-31, 2009-05-22), but only two runs carry the CRPS.

## 5. Calibration shape

- **Narrow core, heavy tail.** The case median is 17.6 µm, the 90th percentile 40, the maximum 576. The top 10 % of cases carry 47 % of CRPS. The 9 cases with cov90 < 0.5 carry 29 %: 6 artefact cases, plus H15M102@5, C14M302@3, C14M101@1 and C03M101@1.
- **Too wide at 0–2 s everywhere.**
  - CRPS per frame is 5.5 / 5.5 / 6.2 µm in the three terciles of prefix frame scatter, while |bias 0–2 s| has a median of 3–5 µm.
  - That 5.5 µm is the CRPS floor 0.234·σ with σ ≈ 23 µm, set by `unc.sigma_d` = 20 µm.
  - The measured frame scatter is about 1.5 µm (90th percentile 2.6). Prefix and continuation come from the same tracker, so the ±50 µm calibration error cancels out.
- **Too wide for burning cases at the 1-s cutoff** (cov50 0.88 / 0.84 / 0.90 by band). s_z is 0.11, but the plateau scatter after the cutoff has sd ≈ 0.05 (IQR −0.032..+0.037).
- **Too wide for cases that go out in the horizon** (cov90 1.00, from the bimodal class mixture).
- **About right** at 5–10 s for burning cases at 3 and 5 s (cov50 0.67 / 0.61).
- **Too narrow** only for artefacts, cases already out at the cutoff, and no-flame runs.

## 6. The 1-s cutoff

Rate residual r = ln(K_obs/K_law) − b over windows of 1–11 s, before t_ext − 0.3 s:

| phase / class | windows (runs, sessions) | median r | sessions above +0.05 / within / below −0.05 |
|---|---|---|---|
| plateau, s ≥ 0.2 | 163 (54, 23) | +0.006 | 2 / 21 / 0 |
| heat-up, s < 0.2 | 44 (35, 21) | −0.050 | 4 / 8 / 9 |
| heat-up, 1 atm, 16–21 % O2 | 18 (15, 8) | −0.092 | 0 / 3 / 5 |
| heat-up, 1 atm, ≤ 15 % O2 | 5 (4, 3) | −0.113 | 0 / 1 / 2 |
| heat-up, 0.7 atm, 16–25 % O2 | 15 (11, 7) | ≈0 | 3 / 3 / 1 |
| heat-up, 0.7 atm, ≥ 30 % O2 | 4 (3, 2) | +0.177 | 2 / 0 / 0 |

- **The plateau law is right by every class.** No O2, diluent, pressure or D0 class has a consistent session offset, and per session only 2010-09-08 (+0.074) and 2011-09-22 (+0.055) are beyond ±0.05.
- **At 1 s the error is run-level plateau scatter.** It is 57 % of the CRPS of burning cases at 1 s, at 5–10 s. The prefix cannot reach it: no e_late or r_end ordering.
- **Heat-up is a small level term,** about 5–9 µm for large droplets at 1 atm.
- **Extinction clock at 1 s:**
  - diffusive +0.57 s, mostly session 2009-03-05;
  - radiative +0.57 s: H05M102 +2.2, C07M101 +0.9, C14M301 +0.8, and N06M101 −0.8 the other way.

## 7. Structural problems a number cannot fix

- **H15M102@5: the end windows straddle the extinction.** The flame goes out at 4.1 s inside end_a (3.5–4.25 s); end_b (4.25–5.0 s) is fully without flame. r_end = 0.74 reads as hot, and the evidence overrides a prior that was right.
- **The class does not learn from the prefix, and its two branches are not separated.** Near D_crit the forecast is a mixture of a radiative branch that goes out in seconds and a diffusive one that goes out tens of seconds later. The median lands between them (H05M102, 175R001, C07M201).
- **The measurement band is one constant for every run.** It is not read from the tracker, and no input exists to widen it when the gate fires.
- **The run offset ignores the end rate,** which is the better predictor at 5 s.
- **Not reachable from any input:** droplets that go out exactly at the cutoff (20CAL10@5, C14M302@3) and artefacts whose prefix is clean.

## Proposals, ranked

The sizes are my rough hand estimates. They are not forward runs.

1. **Widen the band when the prefix shows an unstable tracker** (forward). Inflate the measurement spread when gated frames after 0.2 s ≥ 3, or kept D² grows by > 2 %.
   - Support: 240R003, 40CAL01, 256F002, in 3 sessions, but two runs carry it.
   - Contradicts: none (0 of 167 clean cases fire).
   - Effect: about −2 to −3 µm.
2. **Read the measurement noise from the prefix** (forward). Use the frame scatter from second differences, plus a level term of a few µm, instead of a constant 20 µm.
   - Support: all 26 sessions; the 0–2 s CRPS is the floor in every scatter tercile.
   - Contradicts: artefacts with a clean prefix get worse by about 9 µm per frame (C12M401, H12M101, 191R001, 40CAL01@1, 256R001).
   - Effect: about −0.5 to −1 µm net.
3. **Sharpen the class near D_crit, or update it from the elapsed clock** (law; partly a number, κ). Make the radiative branch dominate above the bracketed boundary (x ≈ −0.07).
   - Support: H05M102, 175R001, C07M201, C12M201, N06M101, in 5 sessions.
   - Contradicts: no wrong-side run on train, but D_crit between atmospheres is uncertain (189R001 at −0.10 and N06M201 at −0.20 must stay diffusive).
   - Effect: about −0.4 to −0.6 µm.
4. **Detect an extinction inside the end windows** (forward). Take the rate within the last end window, or its two halves, or add a state "went out inside the end window".
   - Support: H15M102@5 only, so 1 session, fewer than three.
   - Contradicts: none seen.
   - Effect: about −0.5 µm.
5. **At long prefixes, build the run offset from the end innovation** (forward/law). Use ln r_end with its own persistence (about 1 at 5 s, about 0.2–0.4 at 3 s).
   - Support: 37 cases in 10 sessions at 5 s.
   - Contradicts: at 3–8 s the persistence is weaker (bottom tercile −0.069 far) and it is noisy at 3 s.
   - Effect: about −0.3 µm.
6. **Heat-up in a diffusivity clock, slower at 1 atm** (law).
   - Support: 7 of 11 sessions at 1 atm below −0.05, none above; at 0.7 atm on the law or faster (2009-10-24 and 2009-11-09 at +0.18).
   - Contradicts: 178F001 (1 atm, 30 % O2, +0.016).
   - Effect: about −0.3 µm.

Not proposed:
- diffusive progress by session (no prefix signal; the audit already said leave it alone);
- narrowing s_pop at 1 s (a choice of margin for new sessions; about −0.3 µm on train).

## What I could not determine

- **The CRPS any proposal would actually save.** No forward change was run; the sizes above are hand estimates.
- **Timing for diffusive runs that leave the view** (21 of 39 have no observed time).
- **Whether the near-critical problem is the class sharpness or D_crit in CO2.**
- **How the measurement-noise change would transfer to new sessions** (only train was read).

Small inconsistencies: per-frame sums differ by ≤ 1 µm in 20CAL08. 37 grids written by a stale earlier batch were detected and regenerated before any table was built.

Files are in `D:\Caches\Temp\claude\23fcab7049aa850b\bd40257b-5cbd-4ff9-b44f-69568de1670b\scratchpad\analysis4\methanol\`:
- scripts: `anatomy.py`, `timing.py`, `precursor.py`, `persist.py`, `prefix_sig.py`, `gate_sig.py`, `rate1.py`, `worst.py`, `run_fb.sh`
- outputs: `out_*.txt`
- reruns: `fb\`

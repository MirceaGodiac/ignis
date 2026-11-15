# Model card — `flex_methanol.wsp` and `flex_heptane.wsp` (the multistate droplet forward)

**State: experimental.** Hand-written weights, not trained. Evaluated on development partitions
only; the test partition has not been read. Uncertainty is assumed, not calibrated. One forward
(`crates/ignis-wsp/src/droplet.rs`), two specs, separate weights: no methanol number was copied
into the heptane model.

## What they predict

For a free-floating droplet (FLEX, PSI-69), given the atmosphere and the liquid-diameter
measurements up to a cutoff: the measured liquid diameter at later times as a predictive
distribution over the droplet's state — burning in a hot flame, burning in a cool flame (heptane
only), or evaporating without a flame — and the cumulative incidence of the end of the hot-flame
phase. The diagnostics carry the probability of each state at every forecast time.

Not predicted: flame diameters, temperatures, species, fibre-supported droplets, other fuels.
A release of one fuel refuses the other. Conditions the records mark as conflicting are refused.

## Releases

| Model | Candidate | Release id | How written | Nonzero / authored |
|---|---|---|---|---|
| `flex_methanol.wsp` | D2 | `sha256:b2f6930fc58ddcc0565f25e3968ffa405a718379584890850794bf5ec34a4e97` | D1 with wider spread, after D1's aggregate validation coverage | 61 / 370 |
| | D1 | `sha256:f1f723892493030df0525e3410c7fca42e6709a384b224a69e20d188836b28fb` | data-informed, from the methanol train runs | 61 / 370 |
| | D²-law | `sha256:6c6fd99d756e4d2edb550547d1b5f0767a8878cf3a63d73866fc799dbca16962` | reference baseline and lineage root | 22 / 370 |
| `flex_heptane.wsp` | D1 | `sha256:74b3a0f195884a860d28ab41d9fb3250633a346d0b980138ee1c7d84881842a3` | data-informed revision of W0, from the heptane train runs | 84 / 567 |
| | W0 | `sha256:6ea680d16a04a17cb180bd1628467d7cc0f246c882feffebc25050d226646d8f` | one shot from the literature, before any heptane outcome was opened | 83 / 567 |
| | D²-law | `sha256:7b4ce460f76174b3e9f704fdf09321227c7df2d5ddc048e17744570b4ee9cd6e` | reference baseline | 27 / 567 |

Specs `flex_methanol-wsp-v2` (`sha256:23e20743…`) and `flex_heptane-wsp-v1` (`sha256:d3c57227…`);
forward `sha256:7bef403360a2eaf117128b5bca577cfae8887bb019237a002ab58f8bf65e9262`; author
`claude-fable-5-2` (Anthropic). Files: `configs/models/flex/wsp/{methanol,heptane}/<candidate>/`.

**What "zero-shot" means here.** Heptane W0's edits file (sha256 `3b49e69c…4f7c`) was written and
committed (commit `73a09da`) after the heptane cohort and split were frozen from metadata (commit
`c65b639`) and before any heptane series, test sheet or endpoint was opened. Its author had read
the methanol train runs and the published FLEX literature, including aggregate statistics of the
data report's summary table. The forward was corrected afterwards (event rows, quantiles, validity
gate); the release id changed with it, the numbers did not. Every other candidate in the table is
data-informed. A data-informed first candidate of a new spec is registered as a literal revision
of that spec's D²-law baseline, because the contract reserves "one shot" for releases written
without development exposure.

## Computation

```text
q = D²,  w = 1 − q/q0,  s = t·K_ref/q0  (K_ref = 1 mm²/s; t from igniter-on)
c = (c_o2, c_co2, c_he, c_logp, c_logd0),   x_exc = c_logd0 − N_crit(c) = ln(D0 / D_crit)

hot        dq/dt = −K_h,   ln(K_h/K_ref) = N_rate(c, w, s) + b + s_z·z
class      P(radiative) = sigmoid(κ·x_exc + bias)
diffusive  hot flame ends at progress W, median w_D = sigmoid(N_diff(c, x_exc)), Gompertz sharpness a_D
radiative  hot flame ends at clock S, median s_R = exp(N_rad(c, x_exc)), Gompertz sharpness a_R
cool       (heptane) with probability sigmoid(N_cool_p): dq/dt = −K_c until D = D_ce,
           ln(K_c/K_ref) = N_cool_k(c),  ln(D_ce/2 mm) = N_cool_d(c) + sd·u
no flame   D² falls by k_inf·(q_e/q_ref)^m·Δt + k_hot·τ_e·(1 − exp(−Δt/τ_e)),  τ_e = τ0·q_e/q_ref
N(x) = b0 + skip·x + Σ_j w2[j]·relu(b1[j] + W1[j,:]·x)
```

Prefix encoder (declared input transforms, no fitting):

1. *Validity gate.* A prefix value of D² is dropped when at least three later prefix values exceed
   it by more than 5 % (a droplet does not grow: igniter glare, frozen tracker) or lie below what
   2 mm²/s could have burned in the time between (soot shell counted as droplet).
2. *Level.* Tail-window mean of D² carried to the cutoff along the observed late finite difference,
   limited to [0, 2 mm²/s]. No weight enters it.
3. *Rate offset.* `b = gain·(mix·(e_late, e_early))`, `e` = clamped log-ratio of observed to
   modelled finite-difference rate.
4. *State at the cutoff.* Prior probabilities of (hot, cool, no flame) from the laws at the observed
   progress and clock, times a Gaussian likelihood of the late rate ratio under each state (means
   1, K_c/K_h, k_evap/K_h), raised to the gain and normalised.

Hot phase: RK4 on a uniform 0.05 s grid for 15 rate scenarios; extinction mass collected in 0.5 s
bins; everything after a transition is closed form. The predictive distribution at a time is the
mixture of Gaussians of sd `unc.sigma_d` centred on the mixture atoms; quantiles by bisection of
its CDF.

## What the numbers say

### Methanol D1 / D2 (typed from the train table)

| Law | Numbers | Train evidence | Can fail when |
|---|---|---|---|
| Rate, reference | 0.59 mm²/s at w = 0; plateau 0.55 at w = 0.2 | air, D0 ≤ 2.8 mm: 0.54–0.56 | — |
| Rate, O2 | +0.14 per 5 % above air; −0.33 per 5 % down to 18 %, −0.12 below | 0.69 at 30 %, 0.44 at 18 %, 0.37 at 12–14 % | above 34 % O2 |
| Rate, diluent, pressure | He +0.18 per 20 %, CO2 −0.04 per 20 %, +0.10 per e-fold of pressure | He series at 1 and 0.7 atm | 2–3 atm CO2 runs (too few, unreliable) |
| Rate, size and progress | −0.5 per e-fold of current D above 2.7 mm; −0.35 per unit progress | 3.9 mm in air: 0.47; late rates 17 % lower | droplets above 4.5 mm |
| Heat-up | starts at 55 %, plateau at s = 0.18 | 1.2 s at 2.6 mm, 2.7 s at 3.9 mm | time origin not at igniter-on |
| Critical diameter | 9 mm in air; ×e^0.75 per 5 % O2; CO2 −0.16, He −0.18 per 20 %; +0.7 per e-fold of pressure | brackets at 12, 14, 15 % O2; CO2 and He series | atmospheres far from the train table, pressure above 1 atm |
| Class | even at 7 % below D_crit; κ = 14 (D1), 10 (D2) | eight near-critical train droplets | — |
| Diffusive extinction | w_D = 0.79 (De/D0 = 0.46), later with O2 above 26 % and at 0.7 atm, earlier near D_crit; a_D = 14 | NASA's Dext of the train runs | disruptions (needle contamination) |
| Radiative extinction | s_R = 0.9 at D_crit, ×e^−1.2 per e-fold above, immediate beyond 55 % above | train knees | — |
| No flame | k_inf 0.015 mm²/s at 2 mm (∝ D^1.44), k_hot 0.06 mm²/s, τ0 4.5 s | post-extinction series | — |
| Spread | D1: s_pop 0.14, s_floor 0.05, σ_d 15 µm. D2: 0.18, 0.08, 20 µm | D1's 90 % band covered 0.88 on train, 0.80 on validation | — |

### Heptane W0 → D1

| Law | W0 (literature) | D1 (after train) |
|---|---|---|
| Hot rate, air, 2 mm | 0.60 mm²/s | 0.62; flame development: starts 26 % low, plateau at s = 0.6 |
| Fast heat-up | 50 % over s = 0.2 | 22 % over s = 0.15 |
| Critical diameter, air | 2.8 mm | 3.7 mm |
| … per 5 % O2 / 20 % CO2 / 20 % He | +0.55 / −0.15 / −0.05 | +0.62 / −0.45 / −0.08 |
| Radiative extinction clock | 0.55, −0.8 per e-fold above 10 % | 0.85, −2.0 per e-fold |
| Diffusive extinction | w_D 0.96, rising with D0 | 0.92, nearly flat in D0 |
| Cool flame follows | 0.85, falling linearly with He and with O2 below air | 0.95; off below 16 % O2 and above 15 % He |
| Cool rate | 0.40 mm²/s at reference | 0.38 (−5 %) |
| Cool extinction diameter | 1.13 mm at reference, D0^0.5, [O2]^-0.75, sd 0.25 | D0^0.8, steeper in O2, sd 0.12 |
| After the cool flame | nearly flat | 0.03 mm²/s (∝ D^0.6) plus 0.06 decaying |

The published cool-flame scalings were the part of W0 that needed least correction; the critical
diameter and its CO2 dependence, which the literature gives only as "about 3 mm in air", needed most.

## Results (development)

Cutoffs 1, 3, 5 s; horizon 10 s; CRPS in µm. Full tables with intervals: [RESULTS.md](../RESULTS.md).

**Methanol** — validation 56 common cases in 9 sessions; train 182 cases.

| Release | Validation CRPS | Coverage 50 / 90 % | Train CRPS mean / median |
|---|---|---|---|
| `flex_methanol.wsp` D2 | 45.3 | 0.63 / 0.86 | 35.8 / 23.1 |
| `flex_methanol.wsp` D1 | 46.3 | 0.53 / 0.80 | 34.6 / 20.9 |
| `flex.wsp` W2 (phase 1; written after reading validation per run) | 41.7 | 0.56 / 0.93 | 45.4 / 24.0 |
| `flex.wsp` W0 (zero-shot) | 50.4 | 0.58 / 0.85 | 62.4 / 31.6 |
| flex.v3, fitted (MAP) | 77.7 | 0.34 / 0.75 | 68.2 / 47.3 |
| flex.d2_prefix, fitted | 103.4 | 0.50 / 0.79 | 81.1 / 58.9 |
| D²-law, hand-written | 105.5 | 0.52 / 0.71 | 88.3 / 38.6 |

Paired on validation (other − D1): W2 −4.6 [−17.4, +5.4], W0 +4.1 [−9.8, +19.1], flex.v3 +31.4
[+11.1, +52.2], d2_prefix +57.1 [+32.6, +88.2]. D1 was written from train only and saw validation
once; it is level with the phase-1 W2, whose author had read validation run by run, and it is not
separable from the zero-shot W0 on 9 sessions. Most validation atmospheres are not in train.

**Heptane** — validation 39 common cases in 7 sessions; train 125 cases.

| Release | Validation CRPS | Coverage 50 / 90 % | Train CRPS mean / median |
|---|---|---|---|
| `flex_heptane.wsp` D1 | 51.9 | 0.69 / 0.97 | 49.3 / 28.2 |
| `flex_heptane.wsp` W0 (zero-shot on heptane) | 60.3 | 0.69 / 0.97 | 58.3 / 33.0 |
| D²-law, hand-written | 107.6 | 0.59 / 0.78 | 75.0 / 34.7 |

Paired on validation: W0 − D1 = +8.5 [−5.2, +25.1] (39 cases, 7 sessions). The literature-only W0
halves the error of the D²-law on runs nobody had looked at; the train data bought a further 14 %.
The fitted FLEX experts refuse heptane (their population is methanol), so there is no fitted
baseline. The heptane bands are too wide (0.97 at 90 %); they were left as they are.

## Checks

- Build checks passed for every release: spec, patch, tensors complete, strict-route audit, finite
  forward on the synthetic fixture.
- `tests/droplet_forward.rs` (8 tests, synthetic droplets): a constant rate reproduces the D²-law
  for both fuels; a diffusive extinction stops the droplet at the written progress; a radiative
  extinction comes at the written clock and a cool flame follows it to the written diameter; the
  gate drops glare-collapsed frames and soot-shell frames and keeps the frames before a late
  outlier; a droplet that stopped shrinking is read as extinguished; the band is the mixture of
  measurement Gaussians; a release of one fuel refuses the other.
- First-generation release ids are unchanged by this work.

## Limits

- Refused: runs with conflicting atmosphere records (H19H101, H19H201, C08H502 in heptane train),
  a stored pressure of 0 Pa (C11M201), unresolved conditions on validation.
- The first second of a sooty run (29–34 % O2 heptane) still misleads the encoder: at a 1 s cutoff
  there are not yet enough later frames to contradict the soot shell.
- The hot-to-cool transition dips the rate for a second or two; a prefix that ends there is read
  as a slow droplet within the clamp.
- One 2 atm heptane run in the cohort (cool flame at 0.55–0.60 mm²/s): the pressure laws are
  unsupported above 1 atm.
- Extinction and cool-flame plateau times are not adjudicated in the stored endpoints, so the event
  rows and the state probabilities have not been scored; only the diameter has.
- D2 and the choice not to recalibrate heptane were made after one aggregate validation look each.

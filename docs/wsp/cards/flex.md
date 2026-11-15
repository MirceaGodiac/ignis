# Model card — `flex.wsp` (spec `flex-wsp-v1`)

**State: experimental.** Hand-written weights, not trained. Evaluated on development partitions only;
the test partition has not been read. Uncertainty is assumed, not calibrated.

## What it predicts

For a free-floating methanol droplet (FLEX, PSI-69), given the conditions and the liquid-diameter
measurements up to a cutoff: the measured liquid diameter at later times as a predictive
distribution, and the cumulative incidence of the end of the hot-flame phase and of disruption.

It does not predict flame diameters, temperatures, species, cool-flame burning, other fuels or
fibre-supported droplets.

## Releases

| | Release id | How written | Nonzero / authored |
|---|---|---|---|
| W0 | `sha256:fa8eb18cbb2affa850846fc90b82475fbac13f44d2fbca0fd2321024f4a6e28f` | One shot, before any run | 89 / 316 |
| W1 | `sha256:a3fbfef98e5f02b7a3213dadbb7270e3ee87f0c9084725eb6f3735cc3a03f13b` | Revision from train feedback | 74 / 316 |
| W2 | `sha256:2ce097c5843f7a098e361f87299ca43ad16b4ef595acbcc6fb7a5b8927c4c1de` | Revision of W1's extinction network from validation feedback | 72 / 316 |

Spec `sha256:f67da795eb231b5eee48a1e2d86fc7e4121f86eddfe72514dd26f7d178ff79c9` · forward
`sha256:4c15baf832f950615b452a9c0ec66a09d4e036cd759eb0d457b55a24868ba33c` · author `claude-fable-5-2`
(Anthropic); the reasoning-effort setting is not visible from inside the authoring session and is not
recorded. Edits, provenance and release records: `configs/models/flex/wsp/{w0,w1,w2}-fable/`.

## Computation

```text
q = D²,  w = 1 − q/q0 (burn progress),  s = t·K_ref/q0 (diffusion clock)
x = (c_o2, c_co2, c_he, c_logp, c_logd0, w, s)

burning        dq/dt = −K,              ln(K/K_ref) = N_rate(x) + b + s_z·z
extinction     dΛ/dt = η·K/q0,          ln η = min(N_haz(x) + link_b·b, cap)
extinguished   dq/dt = −k_evap
N(x) = b0 + skip·x + Σ_j w2[j]·relu(b1[j] + W1[j,:]·x)       16 hidden units each
```

Fixed-step RK4 for 15 rate scenarios `z`; at each forecast time the predictive is the mixture of
"still burning" and "extinguished at an earlier step", widened by the measurement noise.

### Neurons, W0 (rate network)

| Unit | Reads | Computes | Numbers | Can fail when |
|---|---|---|---|---|
| skip | all | The d²-law in log form: B-number scaling with the oxygen **mass** fraction | O₂ +0.16 per 5 %, CO₂ −0.075 and He +0.11 per 20 %, pressure +0.08 per e-fold, size −0.08 per e-fold, progress +0.052 | Diluents act through kinetics rather than transport |
| 0–3 | c_o2 | Concavity of ln(1+B) in oxygen | knots at 25 %, 31 %, 18 %, 15 % O₂ | Near the flammability limit |
| 4–5 | c_logd0, w, c_o2, c_co2, c_logp | Radiative loss of a large flame, relaxing as the droplet shrinks | knots at current D = 2.4 mm and 3.6 mm | Droplets above about 5 mm |
| 6–7 | w | Water uptake: the rate sags late in the burn | knots at 25 % and 60 % burned | Other fuels |
| 8 | s | Heat-up and flame establishment | starts at e⁻¹, recovers over 0.15 of the clock | Time origin not at ignition |

### Neurons, W0 (extinction network)

`ln η = ln(a·ln 2) + a·(w − w_ext(c))`, so `w_ext` is the median progress at extinction and `a` its
sharpness (W0: a = 12, deliberately wide). Units 0–1 saturate the gain above air, 2–4 pull
extinction earlier towards the limiting oxygen index, 5 steepens CO₂ suppression, 6–7 are radiative
extinction of large droplets.

### Encoder, spread and the rest

| Parameter | W0 | Meaning |
|---|---|---|
| `enc.gain_w`, `enc.gain_b` | [2.0, 0.5, 2.0], −1.5 | Trust in the prefix's finite-difference rate: about 0.2 at a 1 s cutoff, 0.9 at 3 s, 0.98 at 5 s |
| `enc.mix` | [0.85, 0.15] | Late and early finite difference |
| `unc.s_pop`, `unc.s_floor`, `unc.sigma_d` | 0.22, 0.06, 20 µm | Assumed spread of ln K before and after evidence; measurement noise |
| `post.k_evap` | 2e-8 m²/s | Extinguished droplet |
| `haz.log_disruption`, `haz.link_b` | −4.83, 0 | Constant disruption hazard; no coupling |

### What the revisions changed, and why

- **W1** (train feedback, `docs/wsp/feedback/flex_w0_train.md`): the extinction network was
  rewritten (in air the flame goes out near 80 % burned, not 90–93 %; a sharp size threshold;
  W0 killed 12 % O₂ flames far too early); `link_b = −8` so a droplet that has already stopped
  shrinking is forecast as extinguished; small rate corrections where the train table showed a
  systematic ratio. **It scored worse than W0 on validation** — a pressure term read off three or
  four runs did not hold.
- **W2** (validation feedback, `docs/wsp/feedback/flex_w1_validation.md`): that pressure term
  removed, threshold softened, sharpness back to 12, `link_b = −6`. Rate law unchanged from W1.

## Results (development)

Cutoffs 1, 3, 5 s; horizon 10 s. Validation is 56 common cases in 9 clusters.

| | Validation CRPS | Validation MAE | Coverage 90 % | Train CRPS |
|---|---|---|---|---|
| **W0** (zero-shot) | 50.4 µm | 68.7 µm | 0.85 | 62.4 µm |
| W1 | 56.3 µm | 77.0 µm | 0.81 | 44.1 µm |
| **W2** | 41.7 µm | 59.8 µm | 0.93 | 45.4 µm |
| flex.v3, physics-neural (fitted, MAP) | 77.7 µm | 108.6 µm | 0.75 | see RESULTS.md |
| flex.v1 (fitted, MAP) | 93.8 µm | 124.7 µm | 0.82 | see RESULTS.md |
| flex.d2_prefix (fitted) | 103.4 µm | 145.0 µm | 0.79 | 81.1 µm |
| flex.d2_population (fitted) | 137.5 µm | 149.9 µm | 0.11 | 149.9 µm |

Paired CRPS difference to the population baseline on validation: W0 −87.1 µm [−153.3, −26.0],
W2 −95.8 µm [−149.2, −47.4]. W2's validation score is a development score: its author read
validation run by run. `flex.v1` and `flex.v3` were fitted with MAP (8 s and 52 s); their NUTS
fits (about nine hours) were not run. Part of the gap is structural: the fitted FLEX experts forecast
the diameter given that the flame is still burning, while the evaluator scores every post-cutoff
diameter, including those of an extinguished droplet, which `flex.wsp` models explicitly.

## Checks

- Build checks passed: spec, patch, tensors complete, strict-route audit, finite forward on the
  synthetic fixture.
- `tests/flex_forward.rs`: the d²-law is reproduced to 1e-9 with a constant rate; each condition
  reaches its own column; the encoder moves the rate towards the prefix and is clamped; the event
  rows sum to one, incidences never decrease, the median never grows; refusals name their gate.
- Latency: 2.1 ms per forecast (p50, one thread, artifact construction and hashing included, 91
  prefix observations); 2.6 kB of parameters.

## Limits

- 6 of 65 validation cases are refused (unresolved oxygen and CO₂ conditions); one train run is
  refused for a stored pressure of 0 Pa.
- Runs whose first half-second holds deployment artefacts (the 2009 "CAL" runs) mislead the
  encoder: the worst errors are there.
- The extinction boundary is noisy: the same nominal condition lands on either side of the size
  threshold, and a single-mode extinction distribution cannot say so.
- Extinction event times are not adjudicated in the stored endpoints, so the event rows have not
  been scored; only the diameter has.

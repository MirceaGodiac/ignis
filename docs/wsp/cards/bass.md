# Model card — `bass.wsp` (spec `bass-wsp-v1`)

**State: experimental, runnable, never evaluated.** No BASS run of the cohort has a metric
calibration, so there is no observation this model could be scored against. Nothing below is a
performance claim.

## What it would predict

Given single-valued flame-front contours in a **metrically calibrated** camera plane up to a
cutoff: the front contour f(s, t) at later times and its spatial mean with an assumed spread.

It refuses pixel-only contours and contours that name no calibration, instead of rescaling them.
It does not infer temperature, species, a pyrolysis or char front, or extinction: the camera sees
luminous emission, and a front leaving view is not an extinction. It claims no effect of oxygen,
flow, pressure or thickness, because no metric run exists to support one.

## Release

| | Release id | How written | Nonzero / authored |
|---|---|---|---|
| W0 | `sha256:9ed0451d6819839fe5c9a2825ba5ef08bcb01df025c6c20f37ccdff5bcf29e0a` | One shot, from front kinematics; no contour, calibration or video was looked at | 13 / 15 |

Spec `sha256:fe03eb0d05cce37d2a49c3c4012b0bd3365d92ac2c4576998d69ae5685934cd1` · forward
`sha256:79b2690f33d1d901e3efae99f78346b68b72e60ed640878775907714d4f12311` · author `claude-fable-5-2`.
Files: `configs/models/bass/wsp/w0-fable/`.

## Computation

```text
ξ = (s − s_first)/(s_last − s_first),   φ_0 = 1,  φ_m = √2·cos(mπξ),  m = 0..3
a_m = ∫ f·φ_m dξ   (trapezoid)          v_m = finite difference of window means of a_m
a_m(t_n + D) = a_m·exp(−damp_m·D) + gain_m·v_m·T_m·(1 − exp(−D/T_m))
f(ξ, t) = r(ξ) + Σ_m a_m(t)·φ_m(ξ)      r = what four modes do not resolve, carried unchanged
```

| Coefficient | W0 | Reading |
|---|---|---|
| `op.gain` | [1.0, 0.5, 0.25, 0.0] | The mean front keeps the velocity the prefix shows; shape drift is trusted less mode by mode |
| `op.persist_s` | [1000, 5, 3, 1] | Steady mean spread; shape drift fades in seconds |
| `op.damp_per_s` | [0, 0.099, 0.395, 0.888] | Curvature smoothing ν(mπ/W)² with ν/W² = 0.01 per second |
| `unc.sigma_m`, `unc.rel_v`, `unc.sigma_v_m_s` | 0.3 mm, 0.2, 0.1 mm/s | Assumed spread of the mean front |

This replaces the legacy cosine neural operator (5,825 weights per member, five members), whose
per-run locals are estimated at request time by L-BFGS with a Laplace approximation.

## Why there is no evaluation

- Calibrations on `main`: one, pixel-only, for a **test** run. The data and calibration branches
  add three pixel-only train runs. Metric calibrations: none, on any branch.
- The main PSI-25 camera views the fabric sheet edge-on, so the transverse coordinate of a contour
  is height above the sheet, not position across the specimen width.
- `wsp-dev feedback --partition train`: 4 runs, 0 forecast cases.

A metric calibration for at least the train runs is the precondition for any statement about this
model. A pixel-unit target would need a contract change (every front quantity is in metres) and
would still say nothing physical.

## Checks

- Build checks passed, including a finite forward on a synthetic metric fixture.
- `tests/bass_forward.rs`: a translating tilted front is carried exactly; a damped mode relaxes at
  the written rate; unresolved nodes are bridged and mostly-missing contours dropped; pixel-only and
  uncalibrated contours are refused; the artifact carries the contour field the front scorer reads.
- Latency 0.19 ms per forecast (p50, one thread, synthetic fixture of 21 contours); 152 bytes of
  parameters.

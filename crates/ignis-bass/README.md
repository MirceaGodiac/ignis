# ignis-bass

BASS visible-front experts (V3.1 §9) for solid-fuel flame spread from NASA PSI-25 (BASS-II). The
target is a **geometric observation**: the leading luminous front measured by `ignis-vision`, on one
qualified material/geometry/protocol.

| Model | Kind | What it is |
|---|---|---|
| `bass.operator` | expert | compact neural operator with cosine modes on a nonperiodic domain; 5-member group-bootstrap AdamW MAP ensemble, members trained in parallel |
| `bass.persistence` | baseline | the last prefix contour is carried forward |
| `bass.constant_velocity` | baseline | per-point linear extrapolation from the last (up to 3) prefix contours |

Does not predict pyrolysis, temperature, material certification, or other surfaces. It is a kinematic
model of the visible front, not a reacting-flow solver.

## Modules

`operator` (the cosine neural operator) · `dynamics` (front PDE: second-order ENO2 / local
Lax–Friedrichs, SSP-RK2) · `observation` (Student-t contour likelihood with one run-level scale error,
adaptive Gauss–Hermite) · `model` (member objective, priors, local state inference) · `data` (views →
run arrays, geometric support checks) · `expert` (the `Expert` implementations) · `synthetic`
(labelled synthetic fronts). Hand-written reverse mode runs through the operator, the PDE and the
likelihood. Config: `configs/models/bass/operator.toml`.

## Commands (through `ignis`)

```bash
ignis fit run bass.operator --cohort <cohort> --split <split>    # dry-run plan
ignis vision contours <clip> --calibration <spec> --execute      # produces the observations it needs
```

## Status

- Fit of 5 members: 5.3 s (previous JAX code 174 s), development laptop, synthetic data.
- The planar SIBAL candidate cohort has 25 runs in only 4 groups (split 2/1/1): **not qualified**.
- No contour observations from the cohort's videos exist yet (the dry-run reports 0 train
  observations); 112 `video_run` review items are open.

## Tests

```bash
cargo test --release -p ignis-bass
```

`parity.rs` (JAX fixture `tests/fixtures/bass_parity.json`), `convergence.rs` (spatial convergence
order, ENO2 reverse mode vs finite differences), `smoke.rs` (cosine normalisation, bootstrap
multiplicity, synthetic fit → save → load → forecast).

See also: [GUIDE](../../docs/GUIDE.md#10-vision-metrology) · [REPOMAP](../../docs/REPOMAP.md)

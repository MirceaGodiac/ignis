# ignis-acme

ACME competing-risk experts (V3.1 §8, §10.1) for gaseous flames from NASA PSI-159 (CFI-G). Normal and
inverse configurations are **two separate models**: in normal the injected stream is fuel, in inverse
it is oxidiser, so a numerically similar column can have a different physical role.

Model names: `acme_<normal|inverse>.<variant>` with variant `hazard` (expert) or
`constant_baseline`, `piecewise_baseline`, `linear_baseline`.

Predicts the time and cause (extinction vs operator/automatic termination) of the end of the
established flame under the observed experimental policy, given that it is still burning at the
cutoff. Does not predict a natural lifetime without intervention when the data do not identify it.

## Modules

| Module | Contents |
|---|---|
| `features` | role-aware admissible pre-event features and their train-only normalisation |
| `model` | cause-specific additive neural hazards with session frailties; marginal likelihood with analytic gradient; forecast |
| `expert` | fitting (L-BFGS MAP; NUTS FP64 when qualified), serving, refusals |
| `optim`, `sampler` | thin L-BFGS and NUTS (nuts-rs) wrappers, to be consolidated into `ignis-science` |
| `synthetic` | labelled synthetic event rows for tests, benchmarks and accelerator tasks |
| `common` | config merge, artifact/release construction |

Serving marginalises session frailties with 2-D Gauss–Hermite quadrature; no accelerator is required.
Configs: `configs/models/acme/{normal,inverse}.toml`.

## Commands (through `ignis`)

```bash
ignis fit run acme_normal.constant_baseline --cohort <cohort> --split <split> --execute
ignis fit run acme_normal.hazard --cohort <cohort> --split <split>     # dry-run plan
ignis accel benchmark --release <rel> --tasks <tasks.npz>              # optional (ignis-accel)
```

## Status (development laptop, synthetic data)

- MAP fit 0.08 s (previous JAX code 14.5 s); NUTS 4 chains 13 s, qualified, R-hat 1.006 (previously
  out of memory).
- Real cohorts: normal 129 runs, inverse 131 runs; not fitted on NASA data yet.
- 3 `endpoint_cause` review items (normal) are still unadjudicated and keep those runs out of the cohort.

## Tests

```bash
cargo test --release -p ignis-acme
```

`parity.rs` (Gauss–Hermite vs NumPy, likelihood/gradient/forecast parity with the JAX fixture in
`tests/fixtures/acme_parity.json`, joint normalisation), `smoke.rs` (parameter inventory, synthetic
fit → save → load → forecast).

See also: [GUIDE](../../docs/GUIDE.md#5-fitting-baselines-and-experts) · [REPOMAP](../../docs/REPOMAP.md)

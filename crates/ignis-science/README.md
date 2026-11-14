# ignis-science

Shared numerics for the IGNIS experts. No I/O, no CLI: the FLEX, ACME and BASS crates and the
accelerators build on it. It must never depend on `ignis-evidence` (checked by
`ignis evidence firewall`).

## Modules

| Module | Contents |
|---|---|
| `ad` | reverse-mode automatic differentiation for small FP64 models |
| `transforms` | parameter transforms with log-Jacobians; priors on natural coordinates |
| `studentt` | Student-t process observation model on irregular times (V3.1 ch. 7) |
| `integrate` | Gauss–Legendre integration, cumulative panel tables, monotone inversion with implicit gradients |
| `gauss_hermite` | Gauss–Hermite quadrature for correlated Gaussian local effects |
| `hazards` | cause-specific competing-risk accounting over piecewise cells |
| `events` | event and support accounting after a cutoff |
| `flows` | rational-quadratic spline transforms and coupling masks |
| `nuts` | NUTS through `nuts-rs` on an explicit log density with gradient |
| `optim` | Adam and L-BFGS for MAP fits |
| `diagnostics` | rank-normalised split R-hat, bulk/tail ESS, divergences, E-BFMI |
| `predictive` | weighted particle summaries and the candidate-measurement interface used by VOI |

## Measured (development laptop, synthetic data)

The Rust port reproduces the previous JAX log-likelihoods and gradients within ~1e-12 (parity tests
live in the expert crates). FLEX v3 joint value + gradient at 100 runs on 32 threads: 11.2 ms.

## Tests

```bash
cargo test --release -p ignis-science
```

Unit tests sit next to the code (`ad`, `studentt`, `hazards`, `gauss_hermite`, `integrate`, `nuts`,
`diagnostics`, `optim`, `flows`, `transforms`). A change here should also be checked against every
expert crate's parity tests.

See also: [GUIDE](../../docs/GUIDE.md) · [REPOMAP](../../docs/REPOMAP.md)

# ignis-flex

FLEX droplet-combustion experts (V3.1 chapters 6–7, §23): the physics-neural model `flex.v3` and its
baselines, for **free-floating methanol droplets** from NASA PSI-69.

| Model | Kind | What it is |
|---|---|---|
| `flex.v3` | expert | D² dynamics with curvature, bounded 6→16→1 residual network, competing end-of-phase hazards, Student-t observation model |
| `flex.v1` | baseline | same likelihood and events without the neural residual |
| `flex.d2_population` | baseline | population D² law: constant K from the conditions, D0 from the prefix |
| `flex.d2_prefix` | baseline | constant K fitted on the run's own prefix (broad fixed prior) |

Predicts the liquid diameter over time and the distribution of the end of the observed phase. Does
not predict pure-methanol mass, temperature or species fields, or anything outside the admitted
support (admission gates refuse instead).

## Modules

`structure` (features, variants, parameter inventory) · `dynamics` (dynamics, separable integration,
hazards, per-run likelihood) · `inference` (training joint density, prefix-conditioned local targets) ·
`support` (admission gates: semantic, measurement, condition support, numerical, horizon) · `data`
(views → masked per-run arrays) · `expert` (`FlexExpert`, the `Expert` implementation) · `synthetic`
(explicitly synthetic dataset from the model's generative process).

`experts()` is collected by the `ignis` binary; each expert exposes its candidate model to `ignis decide` and the acquisition API through `Expert::candidate_predictive`.
Configs: `configs/models/flex/{v3,v1,d2_population,d2_prefix}.toml`.

## Commands (through `ignis`)

```bash
ignis fit run flex.v3 --cohort <cohort> --split <split>               # dry-run plan
ignis fit run flex.v3 --fixture synthetic --set fit.method=map --execute
ignis decide rank --release <rel> --run <run-id>
cargo run --release -p ignis-flex --example bench [n_runs]            # value+gradient timing
cargo run --release -p ignis-flex --example geometry                  # NUTS geometry diagnosis (synthetic)
```

## Status (development laptop)

- Parity with the previous JAX code: run likelihood, gradient, trajectory and event CDFs within ~1e-12.
- Joint value + gradient at 100 synthetic runs on 32 threads: 11.2 ms.
- Real cohort: 114 runs, split 27/9/9 groups; the planner estimates ~9 h CPU for a flex.v3 NUTS fit
  (68 train runs, 33,137 observations). **Not run yet; no NASA-trained release exists.**
- Open: flex.v1 NUTS on synthetic data is `insufficient_inference` (R-hat 1.017, 1.5 % divergences).

## Tests

```bash
cargo test --release -p ignis-flex
```

`parity.rs` (JAX fixtures), `identities.rs` (constant-D² limit recovers the SI law, implicit trajectory
gradient vs finite differences, support-mass partition).

See also: [GUIDE](../../docs/GUIDE.md#5-fitting-baselines-and-experts) · [REPOMAP](../../docs/REPOMAP.md)

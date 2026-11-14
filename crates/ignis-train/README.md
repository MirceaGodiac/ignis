# ignis-train

Generic training and decision drivers. It depends on no model crate: the binary passes the collected
experts (`&[Box<dyn Expert>]`), a `&dyn ViewSource` (the real `DataStore`) and the synthetic fixtures.

## Modules

| Module | Contents |
|---|---|
| `fit` | `ignis fit list \| plan \| run`: config loading, resource guard, job ledger, release saving, resume |
| `decide` | `ignis decide rank \| replay`: observation-value ranking (V3.1 ch. 11) through `Expert::candidate_predictive`; reads stored run records (`ViewSource::run_archives`); the continuation is never an input. `score_candidates` is the scoring behind `POST /api/acquisition` |
| `config` | merging `configs/training/defaults.toml` under `configs/models/<family>/<model>.toml` and `--set` overrides |

## Commands

```bash
ignis fit list
ignis fit plan [--n-runs N] [--obs-per-run M]
ignis fit run <model> --cohort <id> --split <id> [--profile p] [--set k=v] [--resume <rel>] [--execute]
ignis fit run <model> --fixture synthetic [--fixture-runs 24] [--seed 0] [--execute]
ignis decide rank --release <rel> [--run <id>] [--cutoff 3] [--policy voi] [--partition validation]
ignis decide replay --release <rel> [--runs 3] [--budget 1] [--policies voi,random,uniform,max_variance]
```

Without `--execute`, `fit run` prints the plan (train/validation runs, observations, method, threads,
estimated hours and USD for the profile) and stops. With `--execute` it writes
`artifacts/models/<hex>/` and a ledger entry in `runs/jobs/`. `--resume` makes a MAP warm-started
`fine_tune` child; NUTS refuses to resume.

Planning constants come from `[estimate]` in `configs/training/defaults.toml` (declared rough values)
unless a model config overrides them; the FLEX configs carry constants measured on the Rust port
(`ignis-flex` example `bench`). The FLEX models have no GPU path: GPU profiles run the same CPU code.

## Tests

```bash
cargo test --release -p ignis-train
```

`fit_save_load_forecast_and_voi_isolation`: synthetic fit → save → load → forecast, and VOI never sees
the continuation.

See also: [GUIDE](../../docs/GUIDE.md#5-fitting-baselines-and-experts) · [REPOMAP](../../docs/REPOMAP.md)

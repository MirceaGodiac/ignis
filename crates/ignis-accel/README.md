# ignis-accel

Optional inference accelerators (V3.1 §10.3–§10.6): a prefix encoder plus rational-quadratic spline
coupling flows (candle autograd) used as **proposals**, with defensive-mixture importance reweighting.

Every expert works without this crate: ACME serves through 2-D Gauss–Hermite quadrature, BASS through
Laplace / defensive importance sampling. A proposal can only change efficiency, never the target
distribution. Accelerators are trained on model-generated statistical tasks, labelled as such, never on
invented NASA data; none has been trained for a NASA release.

## Modules

| Module | Contents |
|---|---|
| `tasks` | generate proposal tasks from a fitted release (labelled model-generated) |
| `nn` | encoder + spline flow networks (candle) |
| `train` | training loop for the encoder/flow |
| `proposal` | defensive-mixture importance reweighting |
| `benchmark` | ESS / error / latency of the proposal against the Gauss–Hermite reference (ACME) |

Configs: `configs/accel/{acme_normal,bass}.toml`.

## Commands

```bash
ignis accel tasks --release <rel> --config configs/accel/acme_normal.toml   # dry-run; --execute writes runs/accel/tasks.npz
ignis accel train --tasks runs/accel/tasks.npz --config configs/accel/acme_normal.toml   # dry-run; --execute trains
ignis accel benchmark --release <rel> --tasks runs/accel/tasks.npz [--accel <accel-release>]
```

`tasks` and `train` are guarded by the resource guard (`--profile`, `--execute`). Run them on a GPU
profile only when profiling shows a benefit end to end (V3.1 §12.6).

## Tests

```bash
cargo test --release -p ignis-accel
```

`parameter_inventory`, `tensor_spline_matches_fp64_reference`, `flow_proposal_density_is_exact`,
`tasks_train_benchmark_smoke`, `bass_tasks_smoke`; unit tests in `proposal`.

See also: [GUIDE](../../docs/GUIDE.md#12-running-on-runpod) · [REPOMAP](../../docs/REPOMAP.md)

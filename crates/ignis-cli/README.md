# ignis-cli

The `ignis` binary. Each command group lives in the crate that owns it (a clap `Subcommand` enum plus a
`run` function); `src/main.rs` only wires them together and injects shared services:

- `experts()`: every expert compiled into the binary (`ignis_acme::experts()`,
  `ignis_bass::experts()`, `ignis_flex::experts()`). A new expert is registered here.
- `fixtures()`: generated (never NASA) archives for `--fixture synthetic`.
- `ignis_data::views::DataStore::new()`: the real data as the `ViewSource` for fit, decide, eval, lab, infer.
- `acquisition_scorer()`: VOI scoring for `POST /api/acquisition`.

| Group | Owner crate | Purpose |
|---|---|---|
| `schemas` | ignis-core | export JSON Schemas |
| `sources`, `data` | ignis-data | sources, guarded fetch, ingest, cohorts, splits, views, review |
| `fit`, `decide` | ignis-train | fit experts and baselines; observation-value ranking |
| `accel` | ignis-accel | optional accelerators |
| `eval`, `infer`, `lab` | ignis-lab | sealed evaluation; forecasts from the CLI; checkpoint lab |
| `evidence` | ignis-evidence | retrieval and the firewall |
| `vision` | ignis-vision | metrology |
| `serve` | ignis-server | API + dashboard |

## Build

```bash
cargo build --release -p ignis-cli               # target/release/ignis(.exe)
cargo build --release -p ignis-cli --features pdf  # + PDF ingest (MuPDF; see GUIDE §1)
ignis --help
ignis <group> --help
```

Expensive commands are dry-runs unless `--execute`. Lab/eval print JSON and exit with code 2 on a
refusal.

## Tests

The binary has no tests of its own; `cargo test --release --workspace` runs every crate's suite.

See also: [GUIDE](../../docs/GUIDE.md) · [REPOMAP](../../docs/REPOMAP.md)

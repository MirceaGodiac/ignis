# ignis-lab

Sealed evaluation (`ignis eval`), forecasts from the CLI (`ignis infer`) and the checkpoint laboratory for
Codex / Astra (`ignis lab`). All
receive the experts and the view source from the binary; this crate depends only on `ignis-core` (`ignis-flex` in tests only).

## Modules

| Module | Contents |
|---|---|
| `eval::vault` | deposit / seal / reveal with a reveal log; the only reader of `holdout/` |
| `eval::deposit` | builds the hidden continuation of each (run, cutoff) of a cohort; holds nothing at or before the cutoff |
| `infer` | `ignis infer forecast` / `forecast-cohort`, the same forecast path as `POST /forecast` (`ignis_core::analysis`) |
| `eval::scores`, `eval::scoring` | CRPS, coverage, log density, MAE; censoring-corrected Brier / IBS / calibration for competing events; masked front metrics; refusal accounting |
| `eval::bootstrap` | group-cluster bootstrap (Rayon) |
| `eval::compare`, `eval::report` | same-case, same-metric comparison; JSON / Markdown summaries |
| `lab::inspect`, `lab::probe` | show / tensor / lineage / diff; probes through the expert's `objective` / `activations` hooks |
| `lab::edits` | reproducible edits into new releases (ops documented at the top of the file) |
| `lab::resume` | hand back to `ignis fit run --resume` |
| `lab::guard`, `access` | lab sessions cannot reach the test partition or the vault |

Vault layout: `holdout/<split-hex>/<partition>/…` (continuations), `runs/evaluation/{artifacts,seals,splits}/`
and `runs/evaluation/reveal_log.jsonl` (every reveal attempt, granted or refused).

## Commands

```bash
ignis eval deposit <split> <payload.json> --run <run>
ignis eval deposit-cohort --cohort <id> --split <id> [--cutoffs 1,3,5] [--partition validation|test|all] [--fixture synthetic]
ignis eval seal <artifact.json> --run <run> --split <split>
ignis eval seals [--release R] [--partition P]
ignis eval reveal <artifact-id> [--final-test]
ignis eval score <ids...> | --release R [--partition validation] [--out f]
ignis eval compare <rel...> [--intersect] [--out f]
ignis eval report <result.json> [--md f.md]
ignis infer forecast --release R --run <run> --cutoff <s> [--settings json] [--out artifact.json]
ignis infer forecast-cohort --release R --split <id> [--partition validation] [--cutoffs 1,3,5] [--execute] [--seal]
ignis lab show|tensor|lineage|diff|activations|grad|sensitivity|residuals|edit|apply|resume ...
```

`--split` for `deposit`/`seal` takes a plain `Split` JSON path or an id resolved under `IGNIS_SPLITS`
(default `data/work/splits/<hex>.json`); the committed `data/splits/*.json` files wrap the split with
its cohort and must be unwrapped first ([GUIDE §6](../../docs/GUIDE.md#6-sealed-evaluation)).

The rules for lab sessions (lineage kinds, posterior invalidation, no test outcomes) are in
[AGENTS.md](../../AGENTS.md).

## Tests

```bash
cargo test --release -p ignis-lab
```

`reveal_requires_prior_seal_and_final_test_flag_and_logs_every_attempt`,
`compare_refuses_mismatched_feature_order_and_scores_identical_cases`,
`edit_creates_child_and_leaves_parent_byte_identical`,
`edit_of_posterior_parameters_invalidates_draws_and_npy_edits_are_recorded`,
`lab_cannot_reach_test_partition_or_the_vault`.

See also: [GUIDE](../../docs/GUIDE.md#6-sealed-evaluation) · [REPOMAP](../../docs/REPOMAP.md)

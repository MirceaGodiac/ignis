# ignis-core

The single authority for shared scientific definitions. Every other crate builds on it; it depends on
no workspace crate.

## Modules

| Module | Owns |
|---|---|
| `contracts` | typed records and their JSON Schemas: `Source`, `ExperimentRecord`, `Observation`, `EndpointRecord`, `ContourObservation`, `Cohort`, `Split`, `PrefixBundle`, `ModelRelease`, `AnalysisArtifact`, `AcquisitionAction`, `DecisionTrace`, `SealedPrediction`, … |
| `contracts::{common,research,evidence,dataset,wsp,evaluation}` | C0 surface: ids, scope, artifacts, errors; episodes/events/jobs/tools/answers; evidence/search/read/wiki; dataset and visual bundles; model spec, literal patch, build manifest, forecast request; evaluation ([CONTRACTS](../../docs/implementation/CONTRACTS.md)) |
| `services` | synchronous domain service traits (`KnowledgeService`, `DatasetService`, `ModelService`, `ArtifactStore`, `EvaluationService`) |
| `schema` | export of one JSON Schema per contract (`schemas/`) |
| `prefix` | causal prefix construction and information-isolation checks |
| `views` | array views handed from curation to the experts; the `ViewSource` trait and `RunArchive` (stored records of a run) |
| `analysis` | the forecast path shared by `ignis serve` and `ignis infer`: settings, run prefix, artifact validation, write-once store, landmarks |
| `records` | run-record registry loaded from JSON lines |
| `expert` | the `Expert` trait: `fit`, `forecast`, `log_likelihood`, optional `objective` / `activations` hooks for the lab, optional `candidate_predictive` for VOI |
| `registry` | immutable content-addressed releases under `artifacts/models/<hex>/` (npz tensors verified on load) |
| `resources` | resource profiles (`configs/resources/`), the opt-in `guard` (dry-run unless `--execute`) and the `Job` ledger in `runs/jobs/` |
| `units`, `hashing`, `interpret`, `npz`, `kernels`, `batch` | SI units, content hashes, npz I/O, shared kernels (OU/Kalman), batch scoring |

## Commands

`ignis schemas export [--out schemas]` writes the schemas consumed by the web client
(`npm --prefix web run gen:types`) and the SAM2 sidecar.

## Tests

```bash
cargo test --release -p ignis-core
```

- `committed_schemas_match_the_contracts`: `schemas/` is in sync with the Rust types.
- `tests/contracts.rs`: fixtures vs committed schemas and wire form, cross-fixture hashes, WSP release
  classification, literal patch validation, scope/lockbox, tool/event/answer/evaluation invariants.
- `prefix_admits_only_the_available_past_and_detects_tampering`.
- `release_roundtrip_verifies_tensors_and_feature_order`.
- `kalman_equals_dense_on_irregular_times`, `duplicate_timestamps_are_rejected` (kernels).

After changing a contract: run `ignis schemas export`, regenerate web types, and update consumers.

See also: [GUIDE](../../docs/GUIDE.md) · [REPOMAP](../../docs/REPOMAP.md)

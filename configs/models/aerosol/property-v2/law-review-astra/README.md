# Aerosol v2 correction packet

This is a synthetic-review-informed revision, **not frozen** pending independent
re-review. It is distinct from the preserved zero-shot W0. No ModelSpec or model
release is claimed; the aerosol/detector shared-contract gap remains.

The kernel is `crates/ignis-wsp/src/property_aerosol_v2.rs`. Its first eleven
tests retain the predecessor invariant cases; ten new cases cover independently
reported failures and the newly derived, explicitly selectable absorbing-film
flux. `AbsorbingFace` now requires an explicit `film_flux_law`; choosing
`AdditivePredecessorApproximation` retains the predecessor numerical behavior for
valid supported inputs. Choosing `SteadyDriftDiffusionAbsorbingFilm` requires an
independent applicability sample and changes only that named physical closure.

Prediction and detector output preserve immutable-access input lineage through
`input_evidence()`. Observation rechecks that lineage against the selected
validation truth. The complete generating physical context, including phase,
composition, T/P, gravity/frame/regime and property provenance, is retained
privately and must match at observation. Only validation-truth expansion is
allowed; a legitimate independent alternate detector can still be supplied.
Explicit input dependency closure is still required from the
supplier; missing raw source identities cannot be discovered by the kernel.

`constants.ops`, `literal-ops.json`, `original-author.json` and
`original-parameter-ledger.md` are exact copies of the corresponding W0 artifacts.
They retain the original literal author and provenance. `author.json` describes
this correction author and synthetic review exposure. No literal changed.

```powershell
rustfmt --edition 2021 --check crates/ignis-wsp/src/property_aerosol_v2.rs
rustc --edition 2021 --test crates/ignis-wsp/src/property_aerosol_v2.rs -o runs/tooling/astra-aerosol-target/aerosol-v2-invariants.exe
& runs/tooling/astra-aerosol-target/aerosol-v2-invariants.exe
```

The research note `docs/wsp/research/astra_aerosol_v2.md` records the independent
reproduction, pre-implementation derivation and limitations. No NASA outcome,
held-out data, fit, parameter search or generated scientific weight was used.

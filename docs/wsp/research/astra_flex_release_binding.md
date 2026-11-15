# Source-bound FLEX diameter release binding

Author: GPT-6 Astra. This is an integration derivative of the frozen clean-room
W0 physics, not another independent zero-shot experiment. No NASA records,
measurements, continuation outcomes, protected test partition, old parameter
tensors or baseline predictions were read by this adapter author. The approved
existing `configs/models/flex/wsp2/{methanol,heptane}/binding.json` files were read
only for cohort, split and source-manifest identities.

The frozen kernel is unchanged: SHA256
`d683f755441a5c730777254421731bfc9e34fe7a99f6bc5a992ba62ce9a69199`.
Its literal operations are rebound, without numerical changes, to a distinct
adapter ModelSpec and PatchId. No missing property, initial state, distribution
width, gravity correction or material-dependent coefficient is filled in.

## Implemented interface and release identity

`property_droplet_release.rs` exports:

- `specification()` and `authored_patch()` for the distinct model
  `flex.property_droplet.source_bound.v1`.
- `forward_hash()` over exact adapter, frozen kernel, physical-core and artifact
  helper bytes.
- `assemble_checked(cohort_id, split_id, source_manifest_hash)`: actually runs
  spec validation, patch validation, materialization, stored-tensor validation,
  and a synthetic source-packet-to-AnalysisArtifact fixture before returning a
  release eligible for write-once registry save. The whole-route audit remains
  `not_run`; no unexecuted audit is marked passed.
- `required_input_items(input, initial_time_s)` defines the exact internal host
  input inventory. Each item's hash includes raw/SI values, units, phases,
  uncertainty, reference and property-domain metadata where applicable.
- `SourceBoundInputs::bind(...)` and `from_snapshot(prefix, Value)` validate a
  host-resolved packet. `snapshot()` exposes its exact reproducible internal
  artifact format. These are not new client/tool request contracts.
- `forecast(scope, request, release, prefix, input_source)` returns the existing
  `AnalysisArtifact` contract and uses the release's stored tensors through the
  frozen kernel's `Parameters::from_tensors`.

The host implements `InputSource::provenance` and `resolve`. Capability and
partition checks precede material resolution. `ExecutionScope` remains a
separate host argument. No scope, material table or partition grant is accepted
through `ForecastRequest` or `ForecastSettings`.

The new local registry is
`runs/dev/astra_flex_source_bound_registry`, outside the original immutable model
registry. Final origin-guard assembly receipts identify:

| Population binding | Release id |
|---|---|
| Methanol | `sha256:357112c4518283f576e655bd0de203e5690b7a61cc26dcada26e2f82976e7bb4` |
| Heptane | `sha256:a93aa81dd009a0bf1ebf596c6ffbf4e3f676286db3248b5ce5b37f3b0b9cbd32` |

Both carry `experimental`, `not_trained`, and forward hash
`sha256:a566a727615492a3807a287d0b882fd9929a084bb0ab3de6426818742ea2d1a2`.
Their immutable receipts are `methanol-receipt-origin-guard.json` and `heptane-receipt-origin-guard.json` in
that new registry root. These are executable candidate identities, not evidence
that real material inputs have been admitted or that a real run was forecast.

## What is required to obtain a defensible diameter prediction

The minimum is a complete physical initial state and the properties consumed by
the frozen law. A liquid diameter series by itself does not identify liquid
temperature, composition or interfacial state.

| Required input | Units / meaning |
|---|---|
| Initial time and time origin | Explicit seconds since the same prefix origin, at or before cutoff |
| Component inventory | Positive mass in kg for each component at that time; identity and CHONS/He/Ar atom counts |
| Initial liquid temperature | K, determined at that same initial-state time |
| Component molar mass | kg/mol |
| Liquid density | kg/m3; justified constant-property domain |
| Isobaric liquid heat capacity | J/(kg K), **not saturation-path heat capacity** |
| Latent heat at reference | J/kg, liquid-to-vapor, shared declared reference state |
| Saturation pressure at reference | Pa at the same declared reference temperature |
| Ambient temperature / pressure | K / Pa, fixed boundary evidence |
| Oxygen mass fraction | Dimensionless mass fraction, not mole fraction silently relabelled |
| Representative gas density / heat capacity | kg/m3 and J/(kg K) |
| Gas conductivity / common diffusivity / viscosity | W/(m K), m2/s, Pa s |
| Background and O2 molar masses | kg/mol; surface background is not inferred from oxygen fraction alone |
| Relative velocity and gravity | Three components in m/s and m/s2 with an explicit shared frame |
| Ideal-solution and constant-environment closures | Source-backed applicability of the frozen approximations |
| Burning branch only | Positive vapor complete-combustion heat J/kg and explicit established-sheet scientific admission |

A mass inventory may be independently derived from a sourced diameter and
density under a documented shape assumption, but that derivation must be its
own provenance-bearing input artifact. This adapter does not invent or infer
that inventory. It also does not estimate temperature, vapor pressure or heat
capacity from observed diameter slope.

Every scalar requires source and locator, source-byte identity, raw/SI units and
values, phase, uncertainty status, thermochemical reference identity and
temperature/pressure applicability domain. Initial masses and temperature need
state determinations at the explicit initial time. The supplied constant-domain
properties must remain valid throughout the trajectory. In the burning branch,
gas domains also cover the predicted reaction-sheet temperature.

The evaporation-only branch removes combustion enthalpy and sheet-temperature
requirements, but still needs all liquid and gas transport properties and the
initial state. It cannot be selected merely to bypass missing combustion inputs
for a known burning interval. A point heat-capacity measurement is not promoted
to a constant-property temperature interval. A saturation-path heat capacity
`C_sat` is not silently substituted for isobaric `Cp`.

At implementation time the property author reported only universal constants
and sourced heptane `C_sat` points as admitted. This does not constitute a
complete W0 input packet. No existing real prefix's sufficiency was asserted by
this author because the task explicitly prohibited reading NASA run data. The
coordinator/data custodian must establish that sufficiency independently.

## Causality, source admission and honest refusals

Prefix identity is rehashed. Observations and contours must belong to the same
run and be taken and available by the cutoff. Forbidden terminal/full-history
condition names are refused. The material packet is tied to that exact prefix;
all evidence and dependencies must be available by the cutoff, and each input
item must match its evidence-bound hash. Unexpected or missing ledger entries
are not ignored. Time-varying controls intersecting the simulated interval are
unsupported by this fixed-boundary kernel.

A generic `phase_established_at_s` marker is **not** evidence for an infinitely
fast reaction sheet. The core legacy helper can derive such a marker from
whole-run endpoint/activity information. The adapter therefore requires a
separate `ReviewedUse::EstablishedReactionSheet` admission artifact with physical
justification, causal known-at time and a validity interval covering the initial
state and requested forecast horizon. The property/phase authority is the
trusted host scientific admission workflow. Strings, hashes and a typed marker
alone do not prove that a source supports the declared physics; the adapter
checks identity and causality, not the truth of arbitrary source content.

The binding release rejects new numerical `ExplicitScenarioAssumption` values
and any spherical extrapolation setting. Additional assumed temperatures,
constant-property approximations or gravity corrections need a separately
authored candidate and explicit literal rationale. Synthetic evidence cannot
enter a real partition request. Model outputs cannot masquerade as supplied
source measurements.

Missing host packets return an unsupported, countable AnalysisArtifact with no
numerical forecast. Malformed/tampered packets are invalid requests. Numerical
or property-domain failure rejects the complete requested forecast; no partial
continuation is scored as if complete. No refusal is recast as extinction.

Forecast times must be explicit, strictly increasing and strictly after cutoff.
No default horizon/grid is introduced. Every requested endpoint integrates the
same admitted initial state through the unchanged kernel; there is no inferred
restart, post-cutoff assimilation or interpolation. `AnalysisArtifact.mean` is
used as the existing deterministic scalar-prediction slot, with that meaning
declared in diagnostics. Quantiles and calibrated intervals are absent, and an
explicit request for a distribution is refused.

The physical packet snapshot and its digest enter artifact diagnostics and
therefore the artifact content id. Generic caches keyed only by
release/prefix/settings must not be reused across mutable material snapshots;
the host cache needs the physical-input digest, or an immutable packet mapping.
The local consumer/comparator performs fresh calls and does not use that cache.

## Executable consumer

The new `flex_source_bound` example requires `--execute`. It performs no dataset
discovery, network access, parameter fitting or scoring. It verifies the frozen
split against the candidate population and checks the explicit host run id is
TRAIN before reading the prefix artifact. The local host creates a TRAIN-only
research scope; the request cannot widen it. Optional material bytes are read
only by `InputSource::resolve`, after capability and partition checks.

```powershell
$env:CARGO_TARGET_DIR = 'runs\tooling\astra-target'
cargo run --offline -p ignis-wsp --example flex_source_bound -- --execute assemble --binding configs/models/flex/wsp2/methanol/binding.json --registry-root runs/dev/astra_flex_source_bound_registry
cargo run --offline -p ignis-wsp --example flex_source_bound -- --execute assemble --binding configs/models/flex/wsp2/heptane/binding.json --registry-root runs/dev/astra_flex_source_bound_registry
```

For a prepared, already admitted host prefix and explicit ForecastRequest:

```powershell
cargo run --offline -p ignis-wsp --example flex_source_bound -- --execute forecast --registry-root runs/dev/astra_flex_source_bound_registry --release-id <candidate-id> --run-id <train-run-id> --prefix <admitted-prefix.json> --request <forecast-request.json> --split-file <frozen-split.json> --output <new-artifact.json>
```

Without `--packet`, the result is an explicit `missing_material_packet` refusal.
With `--packet <source-bound-snapshot.json>`, the same executable validates the
complete packet and runs the real stored release. Output paths are create-new:
existing artifacts are preserved. The snapshot is an internal host artifact,
not an additional frontend or MCP request schema.

## Executed checks and comparison boundary

- `cargo test --offline -p ignis-wsp --lib property_droplet_release --quiet`:
  **9 passed**. This covers stored-release/material-packet/artifact execution,
  no invented spread, strict packet roundtrip, future/missing/tampered evidence,
  missing-versus-invalid distinction, partition denial before material read,
  synthetic-versus-real refusal, stale prefix identity, release spec/forward/
  tensor mismatch, insufficiency of a generic phase marker, and rejection of
  synthetic/model-output prefix dependency taint before host material resolution.
- `cargo check --offline -p ignis-wsp --example flex_source_bound`: passed.
- Both explicit assembly commands above executed successfully; registry save
  reloaded and compared the stored tensors. The synthetic end-to-end fixture is
  executed by the constructor, not merely asserted from a test name.
- Formatting checks passed; frozen W0 kernel hash still matches the original.

These are software/causality checks and synthetic algebra evidence. They do not
establish NASA predictive accuracy or source-property sufficiency. The
coordinator's separately preregistered TRAIN comparator owns the population,
baseline binding, pair selection, refusal denominator and point-error metrics.
If no complete packets are admitted, the honest result is zero candidate
coverage with no paired MAE; a baseline-only score is not a successful candidate
comparison. This adapter author neither scored releases nor read comparison
outcomes.

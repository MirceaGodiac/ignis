# Targets of the law forward that the shared contract cannot name: proposals for the contract owners

A document, not a change: `ignis-core` is untouched. Context: [law_forward.md](law_forward.md),
section 5. Contract rules: `docs/implementation/CONTRACTS.md` (a contract change ships producer,
fixture, consumer and regenerated schemas together).

## How they are carried now (no aliasing)

A positive target of a law slug either *is* a member of the shared `Quantity` enum (its declaration
says `"shared_quantity": "<member>"`, the units must agree, and its artifacts carry that quantity) or
it is **lane-local**:

| Where | What a lane-local target looks like |
|---|---|
| `AnalysisArtifact.quantity`, `.unit` | `null`, `null` (the unit is derived from the quantity in `common::artifact`, so it stays empty) |
| `AnalysisArtifact.diagnostics.law` | `{"slug", "target", "kind", "label": "<slug>:<target>", "unit": "<SI unit>", "shared_quantity": null, "status", "description"}`, on forecasts and on refusals |
| `ModelRelease.applicability.targets`, `.observation_model.targets` | the same blocks, one per target, and the full declaration |
| `ModelSpec.target` | one required shared quantity: the first target that has one (an event target counts as `phase_end_time`, as the ACME specs do). A slug whose targets are all lane-local names `phase_end_time` there only because the field is mandatory; its first `excludes` line says so and the next lines list every lane-local target with its unit |
| `ModelSpec.notes` | the full declaration as JSON |

A consumer that keys on `quantity` therefore never mistakes a flame length for a diameter or a
position; one that needs the lane-local value reads `diagnostics.law.label` and the quantiles.
Event targets carry `quantity = null` as the ACME event models do; binary targets are
`event_probabilities` artifacts with categories `["yes", "no"]` at one time (the cutoff).

## Proposed additions to `Quantity` (each: SI unit, meaning, who forecasts it)

| Proposed member | SI unit | Meaning | Brief's investigations |
|---|---|---|---|
| `flame_length` | m | visible (or OH*/CH*) length of a jet, coflow or burner flame from the burner exit to its tip, steady state | coflow and jet flames, burner rig, electric-field flames |
| `smoke_point_length` | m | flame length at which a laminar jet flame first emits soot from its tip | smoke points |
| `flame_height` / `flame_standoff` | m | height or standoff of a flame over a solid or a burner (where it is not the length of a jet) | PMMA solids, burner rig |
| `ion_current` | A | current drawn through a flame between electrodes under an applied field | electric-field flames |
| `particle_diameter` | m | a stated statistic of the aerosol size distribution (count or mobility median); the statistic must be part of the member or of a qualifier, not left implicit | aerosols and detectors |
| `particle_number_concentration` | 1/m3 | number density of the aerosol | aerosols and detectors |
| `detector_signal` | instrument-defined (for example obscuration in %/m, ionisation ratio 1) | a smoke-detector reading; needs a per-detector unit qualifier, so it may be better as a typed `instrument_reading` with an instrument id than as one member | aerosols and detectors |
| `soot_volume_fraction` | 1 | soot volume fraction, a local or line-of-sight value (qualifier needed) | smoke points, coflow flames |
| `mass_burning_rate` | kg/s | mass loss rate of a burning solid or droplet (where `mass_flow` would mean a fed flow) | PMMA solids |

`spread_rate`, `flow_velocity`, `temperature`, `pressure`, `mole_fraction` and the diameters already
exist and are used directly when a target is one of them.

## Proposed changes to the shapes

1. `ModelSpec.target` (one required `TargetSpec`) cannot describe a model with several targets of
   different kinds. Proposal: `targets: Vec<TargetSpec>` (first = primary), with
   `TargetSpec.quantity: Option<Quantity>` and a required `label` and `unit` when the quantity is
   absent; or keep `target` and add an optional `additional_targets`. Either removes the
   `phase_end_time` placeholder of all-lane-local slugs.
2. `AnalysisArtifact` has no target label and derives nothing but `unit` from `quantity`. Proposal:
   an optional `target_label` (`<model>:<target>`) and an independent `unit` for artifacts whose
   quantity is `null`, validated as an SI unit string.
3. A yes/no outcome has no artifact kind of its own; the law forward uses `event_probabilities` with
   categories `yes`/`no` at the cutoff. Proposal: a kind `outcome_probability` (or a documented
   convention for `event_probabilities` with a single time), so that Brier scoring is not inferred
   from category names.
4. A positive target with no time dependence is returned at `times_s = [cutoff_s]`. Proposal: a
   documented convention (or an `at_time_s: Option<f64>`) for scalar endpoints.

Until the owners decide, the law forward keeps the representation above; changing it later is a
new forward version (a new file), never an edit of `law.rs`.

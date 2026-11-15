# Astra FLEX TRAIN comparison design and independent adapter review

Status: preregistration design, with no outcome scoring by this author. The executable is
`crates/ignis-wsp/examples/astra_flex_train_compare.rs`. Governing registration is
`docs/wsp/PROTOCOL.md` section 11.4; the integrator commits both files, the protocol,
and the exact host-input manifest before executing. This is a descriptive TRAIN
comparison and admission audit, not held-out qualification or a new numerical-authoring claim.

## Frozen population and release identities

The coordinator verified only split/binding metadata and reported 68 methanol TRAIN
runs in 27 frozen groups, plus 49 heptane TRAIN runs in 22 frozen groups. The three
registered cutoffs produce 204+147=351 cases before any model or observation exclusion.
The reviewer did not open experiment outcomes to choose this population.

| Fuel | Adopted baseline | New source-bound adapter release |
|---|---|---|
| Methanol | `sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654` | `sha256:357112c4518283f576e655bd0de203e5690b7a61cc26dcada26e2f82976e7bb4` |
| Heptane | `sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef` | `sha256:a93aa81dd009a0bf1ebf596c6ffbf4e3f676286db3248b5ce5b37f3b0b9cbd32` |

Baseline registry root is `artifacts`; models are `flex_methanol.wsp2` and
`flex_heptane.wsp2`. Candidate registry root is
`runs/dev/astra_flex_source_bound_registry`; model is
`flex.property_droplet.source_bound.v1`. Candidate forward identity is
`sha256:a566a727615492a3807a287d0b882fd9929a084bb0ab3de6426818742ea2d1a2`.
Earlier adapter release IDs produced before the origin-taint correction are superseded.

Bindings are exactly `configs/models/flex/wsp2/{methanol,heptane}/binding.json`:

- Methanol cohort `sha256:9e31b1aa6b56b2086aac85776d27778bb1680a83fcdfc543dad62e0385da637e`, split `sha256:660b7d9ede629b3009b4b761ea9e5875560719566e3dea21112f607cddd49894`.
- Heptane cohort `sha256:8ffa1875d0c963fe3b3ce6886ba7197a7b0c8675ffd837bbf5139049c1655797`, split `sha256:c43bbe3580ffce2a33c400d5aea73b3330cf721133f925c2963579dee6d32001`.
- Both source-manifest binding identities: `sha256:74dedf7ea617ed6da1c5555822137b8433cfbf59cf293ab9989fd5d3c31a6382`.

The runner verifies split-envelope content hashes and release family/cohort/split/source
identity equality. It records exact split-file, binding and TRAIN-allowlist hashes.
`Split.train` contains **group IDs**, not run IDs: enumerate `run_to_group` entries
whose group belongs to `train`, checking `partition_of(run)==Train`.

## Exact access boundary

Use the existing identity-first loader in
`crates/ignis-wsp-dev/src/train_source.rs`, included by path. It reads top-level
row identity while ignoring other fields, then deserializes domain payloads only
for TRAIN identities. `selected == allow` is enforced: a missing TRAIN experiment
record aborts execution with expected/found counts instead of quietly reducing351.
All three cutoffs are enumerated for every returned run; no endpoint-derived
at-risk filter decides which cases exist. Prefix refusals remain case records.

Do not use `DataStore::run_archive`/`run_archives`, `DataStore::views`, or automatic
view rebuilding: their current source paths load a whole family before filtering.
Do not invoke `ignis eval compare`: its current implementation accepts only
validation/test, not TRAIN. The dedicated runner never accesses holdout or evaluation
vault payloads and never grants Validation or Test in its host-derived scope.
The loader necessarily scans mixed JSONL bytes to locate identities; it does not
materialize or retain non-TRAIN observation/endpoint domain records. This is the
actual boundary, not a claim that no non-TRAIN byte is encountered by file I/O.

## Inputs, absence and causal routes

The initial manifest is
`configs/models/droplets/source-bound-train-astra/input_manifest.json`, schema
`astra.flex.host_inputs.v1`, with an explicit absence reason and empty `packets`.
The runner supports future exact packets keyed by strict prefix hash and routes
each through `SourceBoundInputs::from_snapshot`; it is not permanently hardwired
to fabricate a refusal. A later populated manifest requires a fresh registration.
An omitted manifest also means explicitly absent host inputs, never generated defaults.

Input availability is an outcome-free report from the property author, not a new
material measurement by this reviewer. Material catalog
`configs/models/properties/catalog-materials-astra/materials.json`, hash
`1e2e38b0f2ae0582d5899eb0fff95c6816d981ffcb4010780bcc4ec7fe86ea06`,
admits n-heptane **saturation-path heat capacity C_sat** at specified temperatures.
C_sat is neither solubility nor isobaric Cp and must not fill the kernel's Cp slot.
Universal definitions/R/sigma have a separate catalog hash
`982c31048f31e42d5657890aa9784aacd7a490c433f644b581b3f4af04b3c0b4`.
Neither catalog supplies a complete droplet packet.

Missing admitted inputs include explicit initial liquid temperature and component
inventory at a named initial time; liquid molar mass, density, isobaric Cp, latent
heat and saturation pressure sharing a reference; representative gas density,
common species/vapor Cp, conductivity, diffusivity, viscosity, nonfuel-background
and oxygen molar masses; and applicable temperature/pressure/composition domains.
Burning additionally requires vapor combustion heat and reviewed admission of an
already-established infinitely-fast sheet over the prediction interval. The generic
FLEX parser exposes ambient temperature, pressure, fuel, initial diameter, ambient
mole fractions and fiber context; ambient T is not liquid T, diameter alone is not
component mass, mole fraction is not oxygen mass fraction without molecular data,
and ambient composition need not equal the burning-film background composition.
Source derivations could supply such transformations with complete inputs/provenance;
this execution does not invent them. No all-material numerical fill or fitted
post-cutoff initial condition is permitted.

The four output routes are intentionally distinct:

1. `candidate_strict`: actual source-bound adapter call using a strict prefix.
2. `baseline_strict`: unchanged adopted baseline on that identical strict prefix.
3. `persistence_strict`: last valid source-backed liquid diameter admitted by the
   same cutoff, held constant; equal timestamp ties choose last ID in sorted order.
4. `baseline_inherited_activity`: separate historical-policy diagnostic using
   unchanged `build_run_prefix`, whose activity marker can depend on full-run
   endpoint/continuation evidence. It is never supplied to the new model as
   reaction-sheet evidence and never pooled into the strict paired comparison.

Strict prefixes use `build_prefix` and the adopted expert's prospective condition
policy. Observations/contours must already be available and observed by cutoff;
controls must start by cutoff. `phase_established_at_s` is absent. Direct source
inspection confirms the adopted baseline refuses a missing phase marker. Therefore
strict baseline refusal is expected and must not be presented as its usual historical
forecast performance. No credible combustion-phase determination can be manufactured
from liquid diameter alone. The separate inherited route preserves a real reference
calculation, while explicitly disclosing its different information policy.

The candidate has no authored probability distribution. `AnalysisArtifact.mean`
is only the shared deterministic scalar slot. No invented noise, cloned samples,
zero-width intervals, fabricated CRPS, predictive-interval coverage, log score,
calibration or uncertainty score is generated. Baseline means are compared as
means; baseline quantiles are not substituted when a mean is absent.

## Timing, target validity and arithmetic

Each run has cutoffs1,3,5s. Each case predicts at100 fixed times
`cutoff + j/10` for integer j=1..100: strictly future, ending10s after cutoff.
This grid is fixed before outcomes and is an evaluation choice, not a new WSP weight.
The candidate integrates independently to those endpoints under its frozen numeric
controls. No new restart state is inferred at cutoff.

The evaluator scores valid, finite, positive liquid-diameter measurements in metres
within the closed forecast window. It excludes synthetic/model-output measurements
and standard invalid quality flags through `measurement_valid`. Non-target quantities
are not diameter targets. Repeated otherwise-valid timestamps retain the first row
in the loader's deterministic `(time,id)` order; later duplicates are counted.
The report counts outside-window, invalid/origin and duplicate rejections; missing
values are not zero. A case with no valid observations is explicitly unscoreable.

Forecast values are linearly interpolated to the unchanged measurement times, as
in the existing `mae_mean` scoring path. No target interpolation or forecast
extrapolation occurs. A model must return exactly the fixed requested grid with a
finite metre-valued mean; otherwise that case is refused/invalid in accounting.
All models face the same target list in a case. This is intentionally narrower than
legacy scorer permissiveness about missing quantity labels and does not reuse old scores.
For j=1..n valid measurements, case MAE is sum(|prediction_j-observation_j|)/n.
Nonfinite accumulated error fails explicitly. Empty scores are null, never zero.

Primary aggregation: average scored cutoff-case MAEs within a run, then average
those run means within each experimental session, then give sessions equal weight.
Report equal-run macro MAE and measurement-occurrence micro MAE as secondary metrics.
An observation may occur in several registered cutoff windows; the micro denominator
counts each forecast-target occurrence, not unique experimental measurements.
If any scored run lacks session identity, the primary session metric is null with
an explicit reason; it must not silently exclude that run or invent a session.
The frozen split group is retained separately and is not substituted for a missing
experimental session. Report per-fuel and pooled summaries.

Every route reports total registered cases, predicted/scored cases, prediction/score
coverage, status and reason counts, scored runs/sessions/observations, and all individual
case statuses. Conditional MAEs use scored cases and must be read with full-denominator
coverage. Refusals are neither zero-error successes nor arbitrary numeric penalties.
Strict paired scores use only cases scored by both candidate and strict baseline on
identical targets; full population and unpaired cases remain separately visible.
For the registered empty manifest, zero candidate coverage and an empty paired set
are expected before any outcomes. Verdict is `not_comparable_no_candidate_predictions`,
with null paired scores, not a tie or a win. Future runs with candidate predictions
but no common scored cases need their own registration and explicit interpretation.
No uncertainty intervals or bootstrap are part of this protocol.

## Independent adapter review receipt

Final reviewed adapter source SHA-256:
`64990705c125da06d7dabceaf950f12d48400e16114ad9a32bb8e8b872b88bb5`.
It uses actual stored tensors, checks spec/release/forward/tensor identity, enforces
scope and partition before material resolution, binds item hashes and explicit
initial-state time, and validates future prediction times. Packet snapshot parsing
reconstructs the frozen internal types and rejects unknown/noncanonical items.
Missing supported evidence returns an actual countable unsupported artifact;
malformed/mismatched packets remain invalid requests. Numerical failure returns
no partial scored trajectory. Physical input snapshot identity is in diagnostics,
so distinct material packets cannot masquerade as the same scientific result.

Two review issues were raised to the adapter author before this final identity:

- Generic `phase_established_at_s` was not adequate evidence for an infinitely-fast
  sheet; it can be endpoint-derived. The final adapter requires typed
  `ReviewedUse::EstablishedReactionSheet` with admission-artifact identity, physical
  justification, known-at and validity interval covering simulation and requested horizon.
- A real input ledger could formerly depend on synthetic/model-output prefix evidence.
  The final adapter checks observation/contour/condition origins against host provenance
  before resolving inputs and checks dependency origins in binding. The author added
  a regression demonstrating rejection; the reviewer independently reread those guards.

No remaining blocking defect was demonstrated in the scoped final source review.
Source-content truth and validity of physical assumptions remain a host scientific
review trust boundary: nonempty locator/hash metadata does not prove a publication
supports a property. The adapter is a completed conditional software path, not proof
that a complete real packet exists or that the physical model has predictive skill.
The author reports9 synthetic adapter tests passed; this reviewer does not relabel
that report as an independent NASA validation.

## Local verification, read extent and execution

Executed by this reviewer before any scoring:

- `rustfmt --edition 2021 crates/ignis-wsp/examples/astra_flex_train_compare.rs`.
- `cargo test -p ignis-wsp --example astra_flex_train_compare`, using existing
  `runs/tooling/astra-target`:5 passed,0 failed. Includes independent point-scoring/
  denominator/session invariants and the3 TRAIN-loader isolation invariants.
- A final test rerun after the missing-session guard is recorded by the coordinator's
  local gates; no claim is made that a later source change was tested by the earlier run.

Source-only inspection covered scalar scorer interfaces, TRAIN-loader implementation,
split/binding and prefix contracts, data-store load paths, prospective FLEX field
schema, registry/host-scope interfaces, and the new adapter. Baseline binding metadata
was read; initial filename discovery also surfaced other candidate paths. No baseline
weight archive or outcome file was opened. A targeted source search for the baseline
phase requirement incidentally showed a source numerical-step literal and a Gaussian
normalization expression around its gate; no physical parameter or result was copied.
No prior evaluation outcome, final-test data, NASA report or material-value table was
read. Only newly requested PROTOCOL section11.4 was opened. The initial kernel review
and source-author status messages are not independent numeric authorship for this reviewer.

Execution requires the registration commit to contain the current protocol, this
design, executable and supplied host manifest. The runner compares committed bytes
with the working tree (normalizing checkout CRLF), then opens releases/data. It
writes a new output file only and emits its content hash. Root performs the run:

```powershell
$env:CARGO_TARGET_DIR = 'runs/tooling/astra-target'
cargo run -p ignis-wsp --example astra_flex_train_compare -- `
  --execute --registration-commit FULL_REGISTRATION_COMMIT_SHA `
  --protocol docs/wsp/PROTOCOL.md `
  --candidate-registry runs/dev/astra_flex_source_bound_registry `
  --candidate-methanol sha256:357112c4518283f576e655bd0de203e5690b7a61cc26dcada26e2f82976e7bb4 `
  --candidate-heptane sha256:a93aa81dd009a0bf1ebf596c6ffbf4e3f676286db3248b5ce5b37f3b0b9cbd32 `
  --input-manifest configs/models/droplets/source-bound-train-astra/input_manifest.json `
  --output runs/dev/astra_flex_source_bound_comparison_v1/report.json
```

No execution of this command or reading of its eventual scores was performed by
this design/review author. Later evidence may justify another registered experiment;
it must not retroactively alter this population, input policy or metric.

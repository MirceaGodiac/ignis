# Where the scientific models fit in current IGNIS

Inspected 2026-11-14 after fetching `https://github.com/MirceaGodiac/ignis.git`.
Default branch `main` and `dev` both point to
`e28f177fdc623bbe72138146759affd40929f099`. The science checkout remains
`feat/handwritten-science`; no checkout, merge, push, deployment or remote test
was performed. This is source inspection, not verification of the running VM.

## What is actually integrated

Main has the Rust `ModelService` contract (`releases`, `release`, `forecast`),
HTTP model/analysis endpoints, MCP `models_inspect`/`models_forecast`, queued
forecast execution, artifact storage, host scopes, and answer submission checks.
`answer_submit` requires pinned releases and prediction evidence from an
AnalysisArtifact. The model remains an experimental release, not an LLM claim.

Main's `ignis-cli::research_services` configures knowledge and datasets only.
It supplies no production ModelService. The `ignis-wsp` crate is absent from
that tree; it exists on the unmerged science branch. Turning on `--enable-models`
cannot instantiate a missing service. This is an integration gap in addition
to the new kernels' scientific input gaps.

The WSP request/manifest contract is unchanged between main and the science
branch. A material difference is `Quantity::PhaseEndTime`, present only in the
science branch. Burner event-model specifications cannot be transferred blindly
without resolving that shared-contract difference. Preserve main's newer
evidence/dataset/research contracts; never replace them with the older branch.

## Intended common route

Agent -> models_forecast / POST /api/v1/analyses -> host-authorized job ->
ModelService -> release-specific prefix and property/state preparation ->
strict forward -> AnalysisArtifact -> ArtifactStore -> answer_submit and UI.

Use the existing lane interfaces and job runner. Do not add a parallel agent
loop, public physics-packet API, scoring service or transport-specific forward.
The local WspModelService already supplies registry routing, release caching,
prefix provenance and scope checks for the registered legacy WSP forwards.
The new source-bound droplet adapter is a direct internal function with an
extra host InputSource; it is not yet registered in that production service.
Its local CLI example and TRAIN run do not establish HTTP/MCP integration.

## Six physical families and actual contract fit

| Family | Useful output / existing route | Remaining scientific and integration boundary |
|---|---|---|
| Droplets (FLEX, FLEX-2, CFI) | LiquidDiameter is already a supported quantity; legacy WSP forwards and source-bound adapter exist locally | Integrate service dispatch and host inputs. Source-bound candidate refused all 351 registered FLEX cases. Unknown liquid temperature must be explicitly modelled or measured, not required from a nonexistent per-test packet. |
| Spherical burners (CFI-G, s-Flame; apparatus-dependent Flame Design) | Existing normal/inverse event releases fit model-service architecture; Temperature can represent a thermal field at declared coordinates | Resolve PhaseEndTime contract delta; physical sheet geometry is not automatically visible/UV diameter. Finite-rate extinction and nonradial gravity remain physical gaps. |
| Burning-rate emulator | Temperature can represent actual apparatus response | Bind actual versus virtual apparatus quantities; autonomous feedback closure missing. Heat flux/heat-release outputs need explicit supported quantities. |
| Jet/coflow flames (CLD, SLICE, E-FIELD, SPICE where apparatus agrees) | Shared release/service/artifact mechanism; existing temperature/velocity quantities cover only appropriate scalar observations | Transport kernel is not lift-off, smoke-point or electric-force physics. No proxy labelling of missing observables. Source-bound geometry, properties and stabilization remain necessary. |
| Solids (BASS, BASS-II, Saffire I-III) | FrontPosition/SpreadRate and Temperature exist; legacy BASS release has a service route | New thermal/pyrolysis kernel is not a calibrated visible-front model. Couple gas/solid feedback and bind the measured front. Retain pixel-only versus metric distinction. |
| Aerosols (SAME, SAME-R, DAFT) | Same service/release lifecycle is reusable | Current Quantity lacks concentration, size moments and detector response. Needs a contract proposal plus real source/detector properties, not an alias to flame diameter or temperature. |

Physical model families are not investigation IDs: six reusable implementations
serve nineteen investigations through apparatus/material/condition mappings.
`Family::Other` exists but does not manufacture a missing observable or its
validation semantics.

## What the current data contracts supply

DatasetBundle carries partition/audience, cohort/split/source identities, views,
quality metadata and explicit time origin. ConditionsTable has typed keys for
oxygen fraction, pressure, flow velocity, gravity, ambient temperature, fuel,
diluent, geometry, hardware and diagnostics, with locators and missing/conflicting
states. These are the correct inputs to a host-side preparation component.

They do not currently provide a complete liquid-temperature history, a verified
material property library or all new-family physical quantities. Do not confuse
having a ConditionsTable implementation with having every physical input on each
run. ForecastInput currently accepts StoredRun or an admitted Prefix hash; it
does not expose a general off-test physical-scenario request.

The newer remote R6b/R7a branches concern acquisition/adoption and documentary
parse views. Their source diffs do not install WSP model dispatch or supply a
new physical-property request contract. They remain separate from merged main.
The newer M4 branch concerns deployment automation. No unmerged branch is
reported as running production behaviour here.

## Concrete integration sequence

1. Prepare a scoped integration of existing WSP assembler, registry adapter,
   service and adopted releases against main. Preserve main's newer contracts;
   submit PhaseEndTime and missing targets through the existing contract process.
2. Provide a host prefix/data adapter through DatasetService or agreed data-lane
   access. Check partition before bytes and bind property/state inputs to the
   artifact identity. Do not ask an end user or the conversation model to supply
   a large private physical packet.
3. Make reusable material properties part of a versioned scientific input/model
   catalogue. Treat unavailable per-test state as a separately authored physical
   scenario distribution when defensible, with literal Astra numbers and reasons.
   Preserve measured values versus assumptions. Revise unrealistic constant-property
   closures in new forward files rather than broadening source domains by fiat.
4. Inject ModelService in the composition root and enable model tools only for
   deployable releases. This is Mircea's shared integration point; no such edit
   or deployment is performed by this inspection.
5. Prove HTTP/MCP request -> real forward -> stored AnalysisArtifact -> pinned
   answer with an end-to-end test. Then preregister a real TRAIN prediction
   comparison and report accuracy together with coverage. Zero-output admission
   tests cannot qualify a scientific model.

The initial deployable set should include already usable adopted releases with
their actual qualification limits. The new six-family kernels are experimental
extensions; expose them only with accurate supported-domain descriptions and
actual required inputs. This is not authority to count inherited predictions
as new-model improvement or to replace the all-19 objective with legacy coverage.

## Source inspection record

Read remote main AGENTS.md fully; relevant portions of WORKBOARD/INTEGRATION;
CLI composition, ModelService/DatasetService contracts, MCP dispatch, answer
submission guards, Quantity/Family/ForecastInput, ConditionsTable and DatasetBundle.
Compared main/science WSP contracts and inspected local WspModelService and the
new adapter's already tested boundary. Inspected remote branch metadata and
R7a changed-path list only; no NASA outcome files, holdout, validation series,
gold benchmark questions or private evaluation outputs were opened. No new
physical numbers, candidates or accuracy results were produced by this review.

# Proposed boundaries for physical scenarios and additional observables

2026-11-14. Proposal only. No shared Rust contract, JSON Schema, generated
TypeScript, registry record, release or public endpoint is changed by this file.
The concrete consumer is a property-based family forecast, including smoke
transport, that cannot honestly be represented by every existing WSP target.
The proposal must go through the owning contract lane with producer, fixture,
consumer and regenerated schemas before it becomes an application capability.

## What the owning code supports now

`crates/ignis-core/src/contracts.rs` defines a closed `Quantity` enum and its SI
unit mapping. It includes liquid/visible/UV diameters, droplet position,
pressure, mole fraction, temperature, mass/actual-volume/standard-volume flow,
front geometry, spread/flow speed, gravity level and phase-end time.
`ModelSpec::validate` requires the target unit to equal that mapping.
`Family::Other` already exists; a new family enum entry alone is not the barrier.

`PrefixBundle` requires a string `run_id`. Its `known_conditions` is a map of
`Measured` values, with separately represented observations, control stages,
cutoff, time origin, transforms and input hash. This permits additional named
numeric conditions in a private forward adapter, but does not create a checked
property-bundle contract, a scenario identity or a new target quantity.
`AnalysisArtifact.run_id` is optional; the asymmetry does not authorize inventing
a NASA run to satisfy the input. The current `WspModelService` uses a prefix
source. None of this is a shipped conditions-only forecast interface.

## Quantities that need an explicit scientific meaning

| Proposed target | SI unit | Required observable definition |
|---|---|---|
| Reaction-sheet radius | m | Radius of a stated reaction surface in a particular chemical reduction; distinct from optical diameter or peak emission |
| Lift-off height | m | Distance from an explicitly located burner exit to a specified flame-base contour; projection and spectral band stated |
| Heat-release rate | W | Domain-integrated chemical source under a declared enthalpy convention; distinct from radiation or heat delivered to a solid |
| Heat flux | W/m2 | Signed transfer through a specified surface; conduction, convection and radiation distinguished, with an area convention |
| Aerosol number concentration | 1/m3 | Number per actual sampled gas volume, size/charge/efficiency range and transport correction specified |
| Aerosol mass concentration | kg/m3 | Material mass per actual gas volume; detector conversion and particle-density/shape assumptions retained |
| Particle diameter | m | A closed definition such as volume-equivalent, mobility-equivalent, aerodynamic or optical-equivalent; definitions are not interchangeable |
| Aerosol size moment | m^(q-3) | Integer order q and exact distribution/volume/diameter convention; an order-dependent unit cannot masquerade as a fixed-unit quantity |

For number distribution `n(d)` per gas volume per diameter, define
`M_q = integral d^q n(d) dd`. Thus `M_0` has unit `1/m3`, `M_1` has `1/m2`,
`M_2` has `1/m`, and `M_3` is dimensionless. These are dimensional identities,
not model coefficients. A compact monodensity sphere assumption would relate
mass concentration to `(pi/6) rho_particle M_3`; fractal or porous aggregates
need a different mass-size relation. A moment inferred from an instrument is a
derived observation with the instrument inversion and its assumptions, not a
direct counter reading. A cross-lane design should either enumerate the concrete
supported moment orders or introduce a checked moment descriptor with compatible
units. It must not accept an arbitrary dimension string without validation.

Raw PMT counts, voltage, camera intensity and instrument alarms belong to
channel-specific observations with a calibration/response reference. They do
not become heat release, temperature, particle concentration or physical
extinction by changing their field name. Existing `LiquidDiameter` must never
be used for smoke particles, and `MoleFraction` must never stand for particle
number fraction or an uncalibrated detector response.

## A conditions-only source with a separate identity

Propose a discriminated forecast source, not another agent loop or physics
implementation in HTTP/MCP:

- **Stored experiment:** a typed existing run reference and explicit prefix
  cutoff, resolved through the existing host-controlled data service.
- **Physical scenario:** a scenario identity plus content-addressed references
  to conditions, geometry, material/property bundle, initial state and controls.
  No experimental run is implied. Optional documentary links explain the
  scenario's provenance without pretending those documents contain its outcome.

Both resolve to the same scientific library's validated inputs. Execution scope,
workspace identity and partition grants remain separate host-derived arguments.
No scenario, request document or supplied property can grant access to a dataset.
The service must resolve and authorize every referenced artifact before reading
it. A scenario cannot smuggle an after-cutoff observation into a material property.

The source distinguishes observed initial values, independently sourced material
properties, prescribed controls and author/user assumptions. Each number keeps
raw value/unit, SI value/unit, source locator, uncertainty or explicit unknown,
and its role. Properties carry phase, temperature/pressure/composition domain,
reference state, correlation/version and shared dependency identity. A material
name joins a record; it does not silently select missing density, kinetics,
pyrolysis, optics or detector transfer coefficients.

Initial state is not the same as atmosphere. Temperature/composition fields,
liquid inventories, thermal history, reactive branch and particle distribution
may be needed even when pressure, oxygen percentage and flow are known. A source
must distinguish a commanded rate from an unknown rate being predicted. Unknown
initial temperatures and mixture fractions remain unknown, never room-temperature
defaults or zero. Gravity is a vector or a source-bound history in a stated frame;
the legacy dimensionless gravity-level scalar cannot encode direction by itself.

The input hash includes source kind/identity, all resolved input hashes, property
and transformation versions, model release and settings. Experiment and scenario
artifacts must not share a cache entry merely because some condition values
coincide. Derived artifacts retain `model_output` provenance and the literal
release's `not_trained` / `experimental` construction state.

## Applicability is a separate, evidenced result

The requested indication of whether a scenario is inside checked conditions
must identify the release, target, evaluation artifact, cohort and rule used.
It cannot be an unqualified Boolean inferred from a rectangular min/max box.
A nominally in-range pressure with an unseen material, optical definition,
gravity orientation or reaction branch can be an extrapolation.

A minimal result distinguishes: no checked domain available, inside the
declared evaluated condition domain, partial coverage with named missing axes,
outside that domain, and outside the forward's physical support. These are
proposed semantics, not new contract enum values. The comparison uses authorized
condition metadata and frozen evaluation declarations. It does not reveal
withheld per-test outcomes, fit parameters to an evaluation, or upgrade a release
because its numerical tests passed. Unknown uncertainty remains explicit.

Support and empirical qualification remain separate. An equation may execute
within a property library's validity range while no NASA experiment has checked
its coupled prediction there. A hypothetical lunar or suit atmosphere may be a
useful scenario and still be unqualified extrapolation. A conditional supplied
transport coefficient is not an autonomous prediction of gravity response.

## Optical/event evidence needs its own endpoint identity

The existing phase-end target is insufficient to equate all extinction-like
signals. A proposed event definition reference should bind the observable,
instrument/method version, threshold and confirmation rule, time-origin mapping,
supporting source interval and censoring semantics. First observed loss of light,
later relighting, last observed loss, recording termination and commanded flow
termination are distinct assertions. An operator abort is a completion state;
it neither proves nor excludes a chemical extinction cause.

Keep source timing brackets and processing support separate from calibrated
measurement uncertainty. An interval straddling a forecast cutoff does not prove
that the system was active at that cutoff. A short dark tail at EOF is not a
fully observed confirmation interval. A timestamp gap cannot be counted as
observed darkness. These semantics follow the new TRAIN diagnostic audit; they
do not retroactively change an old release, measurement or registered score.

## Proposed integration evidence

The contract owner should require producer/consumer fixtures covering a scenario
with no run, an identical numerical scenario with different property provenance,
an unsupported particle-equivalent diameter conversion, an interval-censored
optical event and a missing physical input. Fixture origin remains synthetic.
Boundary tests should reject a client-supplied scope, a false SI unit, a missing
property dependency, mismatched phase/reference conventions, a forged evaluated
domain flag and a protected measurement hidden behind a scenario reference.

Until this proposal is implemented through its owning lane, local aerosol and
heat-transfer kernels can be verified as scientific computations with honest
private artifacts. They must not be registered under a semantically false
existing target. For the nineteen-investigation ledger, this is an explicit
contract limitation where it prevents a release/result, not a claim of coverage.

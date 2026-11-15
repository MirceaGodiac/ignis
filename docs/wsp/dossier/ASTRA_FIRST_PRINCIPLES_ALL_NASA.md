# Astra: one physical subject, six reductions

Written 2026-11-14, before building the all-NASA forwards or selecting their
numbers. Author: GPT-6 Astra (`gpt-6-astra`). This is a prospective design,
not a coverage or validation claim. The ten releases in `ASTRA_RESULTS.md`
are an earlier **data-informed** experiment; they are not the candidates
proposed here. This author has development exposure. A new numerical W0
author must receive a screened, fresh context.

## 1. Questions and commitments before construction

The investigations differ in boundary conditions, material inventory and
instruments. Investigation identity is provenance, not a numerical feature.
The proposed decomposition is

`properties + initial state + boundaries -> conservation law -> observable -> instrument`.

The common core conserves species, elements, mass and energy. Momentum carries
gravity; charged flows also require charge transport. Condensed interfaces
exchange mass and enthalpy with the gas. Aerosols carry a population balance.
No single burn-time clock closes all those problems. The construction must
answer these questions explicitly:

1. Is a quantity an imposed control, an unknown state, or an observation?
2. Which state and boundary data identify the requested prediction?
3. Which omitted mechanism could reverse a predicted trend?
4. Which physical property is shared with another investigation, and which
   apparatus coefficient must remain local?
5. What would distinguish an inaccurate law from an inaccurate measurement?
6. Which source justifies every number, and what has its author already seen?

For example, a commanded burner feed is not an independently predicted burning
rate; a visible luminous edge is not automatically a reaction sheet; an
estimated smoke mass does not specify particle number or diameter.

## 2. Minimal laws and coordinates

Gas conservation starts with mass continuity and a balance for each species,
with mass-averaged diffusive fluxes summing to zero. The energy equation must
use one consistent enthalpy convention: formation enthalpies and an additional
complete-combustion heat source must not count chemical energy twice. Transfer
terms at a gas/liquid/solid interface cancel in the combined system.

The Cantera 3.1 governing-equation reference provides an inspectable example
of species/energy balances, corrected diffusive fluxes and explicit inlet
conditions. Its stagnation-flow reduction is not a universal geometry.
[Governing equations](https://cantera.org/3.1/reference/onedim/governing-equations.html).

Define a geometric length `L`, gas diffusivity `D`, kinematic viscosity `nu`,
thermal diffusivity `alpha`, and reference states. The diffusion time `L²/D`
remains meaningful when imposed velocity and gravity vanish. A useful basis is:

| Mechanism | Coordinates retained | Reason for the choice |
|---|---|---|
| Forced transport | `Pe=UL/D`, `Sc=nu/D`, `Le=alpha/D` | Advection relative to diffusion; species need not share a diffusivity |
| Chemistry | `Da=L²/(D tau_chem)`, state and collider composition | Hot and low-temperature chemistry need different pathways |
| Gravity | `G=|g| (Delta rho/rho) L³/D²`, acceleration direction/history | Finite at zero imposed flow; `Ri=G/Pe²` is secondary when `Pe` is nonzero |
| Heat loss | optical depth, surface/gas radiation-to-conduction ratios | Geometry, walls, soot and spectral properties determine loss |
| Liquid interface | `p_sat(Ts)/p`, Stefan mass-transfer number, latent/sensible energy ratio | Volatility couples composition, mass loss and temperature |
| Solid response | `Fo=alpha_s t/delta²`, `Bi=h delta/k_s`, pyrolysis/char state | Thickness alone does not decide thermal thinness |
| Apparatus | curvature, confinement, blockage and feed ratios | Similar fuel and atmosphere do not make apparatus equivalent |
| Charged gas | ion drift/diffusion ratio, charge density, field geometry | Voltage alone does not specify electric force or chemistry |
| Aerosol | particle size definition, slip, collision/residence and settling ratios | Number, condensed volume and detector response are different observables |

These coordinates are diagnostics, not independently adjustable weights.
Redundant ratios are not additional information. Equal oxygen partial pressure
does not fix dilution, diffusivity, density, heat capacity or reaction rates.
At zero gravity Brownian diffusion, forced flow and heat loss persist.

Species properties and mixture transport rules are separate objects; adopting
unity Lewis number is a declared approximation. Every mixture rule must state
its phase/state domain. [Cantera transport reference](https://cantera.org/3.1/reference/transport/index.html).

## 3. Six reductions, with different state inventories

| Family | State and boundary distinction | Candidate observables; principal unresolved closure |
|---|---|---|
| Droplets | Shrinking liquid inventory, liquid temperature and evolving mixture composition; surrounding reacting gas | Liquid diameter, flame geometry and phase-end events; gas chemistry, activity, support heat loss |
| Spherical burners | Fixed burner boundary and imposed stream; normal/inverse orientation uses actual inlet compositions | Flame radius and survival; radiative/kinetic extinction and burner heat leak |
| Burning-rate emulator | Heated or porous surface with controlled supply and heat feedback | Surface heat transfer or flame geometry; a rate only when feed is an unknown closed by a surface balance |
| Jets and coflows | Axial transport, entrainment, scalar mixing and strain; optional electric charge/field | Length, lift-off and extinction; soot and electrical response require additional justified state |
| Solids in flow | Conjugate gas/solid heat transfer, finite fuel, pyrolysis, moisture and char | Pyrolysis-front/spread and visible-flame observables separately; formulation and through-thickness response |
| Smoke and aerosol | Formation or prescribed initial distribution, transport, coagulation and losses | Defined size moments/concentration or detection response; formation yield and detector transfer are distinct |

The first executable member of each family may be a restricted limiting law.
Its card must state which requested targets remain unsupported. A transport
model conditioned on an observed initial aerosol is a prefix forecast, not
a prediction of smoke production from material identity. A flame-sheet model
with prescribed gas properties does not thereby predict soot or a cool flame.

Shared properties mean identical physical inputs return identical property
functions in every family. Common radiation or transport code is shared only
where its assumptions apply. Geometry-specific heat-transfer closures do not
become universal by putting their constants in one array.

## 4. What a property block owes its users

Every entry needs material/composition identity, phase, temperature/pressure
range, raw and SI units, reference state, source/locator, uncertainty and
provenance dependencies. Two handbooks copying one determination are one
source. Mixtures need composition and a justified mixing/activity law. A
trade name is not a polymer formulation, porosity, moisture or char law.

The authored interface accepts physical properties; names are only lookup and
provenance keys. Perturbing a material name while preserving its property
record must leave the result unchanged. Missing transport or chemistry is a
stated limitation, not a fallback to a convenient known fuel.

Literature correlations retain their external empirical origin. Transcribing
published fitted thermochemical coefficients would not make those coefficients
first-principles or newly inferred by Astra. No script will fit, optimize,
interpolate an outcome-dependent weight, or silently import a mechanism into
a handwritten patch. Under the owner's literal rule every retained numerical
assignment is typed and justified by its author. An external correlation that
cannot be reconciled with that rule remains a comparison source, not a hidden
parameter generator. Constants and discretization choices are inventoried too.

A disagreement between investigations about one shared property first tests
composition, units, reference state, observable definition, calibration and
missing mechanisms. We will not manufacture an investigation-specific
diffusivity to make both scores look good.

## 5. Uncertainty and gravity without fitted reassurance

Keep condition uncertainty, measurement uncertainty, property uncertainty,
unknown initial/boundary state, model discrepancy and numerical error separate.
Shared calibrations and shared sources correlate errors across observations.
First-order covariance propagation is conditional on the measurement function;
it is not a calibration of forecast coverage. [NIST TN 1297 Appendix A](https://www.nist.gov/pml/nist-technical-note-1297/nist-tn-1297-appendix-law-propagation-uncertainty).

Where probabilities are unsupported, return named scenarios or conditional
ranges, with unquantified discrepancy explicitly retained. A typed probability
law is an author's assumption, not a posterior. Near an ignition/extinction
branch, small continuous perturbations need not imply small output changes.
More frames do not average away a common length-calibration error.

Gravity is a vector/history input to momentum, settling, deformation and their
coupled heat/mass transfers. There is no first-principles theorem that the
microgravity and normal-gravity endpoints bound every intermediate response.
The owner's NASA attribution requires a screened source and its exact scope;
it is not adopted as a universal interpolation constraint. A lunar prediction
without relevant withheld evidence is an unvalidated scenario, never a safety
bound simply because its acceleration lies between two measured endpoints.

## 6. Conditions-only requests and the existing contract boundary

A scenario identifies atmosphere and fraction basis, pressure/temperature,
physical material state, geometry, flow/control history, gravity/field, initial
or ignition state, requested observable and horizon. Missing values remain
missing or explicit scenario assumptions. Store a scenario hash independently
of any experimental test ID.

Return separate applicability statements for mathematical input acceptance,
property range, physical reduction, instrument comparability, checked
experimental domain and gravity extrapolation. Marginal range membership does
not establish coverage of a new combination.

The current shared contract has `Family::Other`, but its `Quantity` list does
not contain aerosol moments, particle number concentration or radiative heat
flux. Its `TargetSpec` requires that closed quantity and its SI unit. We will
not mislabel a smoke moment as liquid diameter, or bypass registry checks to
claim a release. Scenario input and new observables need an explicit proposal
to the shared-contract owner; this task does not edit those contracts. Private
scientific prototypes are useful executable work, but remain prototypes until
an authorized, honest contract can carry their output.

## 7. Evidence production and what counts as coverage

Pictures can score a model only after a separately authored measurement method
has a fixed observable, calibration, failure rules, measured TRAIN-reference
error and independent repeat. Pixels are evidence; the forecast must not
choose the segmentation. Each image-derived value carries a method version,
source/frame locator, time origin and correlated uncertainty. A lack of an
existing numeric table does not mean the archive can never score the model.

For each investigation record these distinct states: catalogue located;
conditions resolved; frozen groups available; observable defined; source
acquired; measurement qualified; physical properties checked; forward accepts
eligible inputs; literal release built; prospective evaluation registered;
withheld-group score available; qualification decision. None implies the next.

Coverage uses the owner's fixed nineteen-accession denominator, with a row per
accession. Each row names its actual observable, accepted/refused test units,
missing conditions, measurement availability, release, registration and result
or exact reason a score is unavailable. A matrix row/day is not counted as a
completed test. Reusing one family is desirable; relabelling its output or
silently excluding inconvenient cases is not coverage.

Page cards, test labels, clusters, edges and findings use Fable's schema and
closed lists. Apparatus-derived flame type and title-derived research topic
are not observed test outcomes. Missing vocabulary goes in
`measure/VOCABULARY_PROPOSALS.md`. Label accuracy has its own blind audit;
agreement is reproducibility, not empirical model accuracy. Protected labels,
mixed pages and findings with protected dependencies remain closed to authors.

## 8. Prospective authoring and build order

1. Consume only the document lane's released conditions and frozen group
   identities; preserve the author projection before any new-investigation
   TRAIN outcome is read. Raw documentary work is no longer Astra's role.
2. Request source-bound properties, causal controls and independently qualified
   TRAIN measurements through `measure/REQUESTS.md`. The document lane owns
   acquisition, screening, labelling, discrepancy extraction and measurement.
3. Check physical-property entries against independent sources. Create the
   smallest reusable conservation/transport blocks with outcome-free unit,
   reference-state, symmetry, positivity and conservation checks.
4. Construct one clean-room numerical candidate per family from a screened
   specification and class A/B physics, then retain it unchanged. The original
   prompt explicitly requests fresh-agent W0 experiments; this specific role
   is distinct from delegates choosing this informed author's weights. Record
   that structure designed by informed authors is not itself zero-shot.
5. Commit exact releases and condition-only inclusion rules, endpoints,
   metrics, baseline and interpretation in PROTOCOL section 11 before opening
   corresponding outcomes. Do not spend validation on exploratory repairs.
6. Author subsequent **data-informed** candidates using TRAIN evidence only,
   with literal origin counts, all attempted candidates and negative results.

This sequence is a scientific dependency order, not a claim that all steps
already work. Astra proceeds on independently sourced physics while the other
lane produces admissible conditions and measurement releases.

## 9. What the first executable families change in the argument

This section is written **after** the new kernels and independent checks, not
retroactively presented as a preconstruction choice. Their numerical authors
are fresh `gpt-6-astra` contexts. The coordinating author remains data-informed.
The original zero-shot packets stay frozen; synthetic-review corrections have
new files and explicit data-informed status without NASA outcome feedback.

### What is a parameter when the forward is code?

A parameter is a numerical choice that can change a forecast while the declared
law, boundary problem and input evidence stay fixed. Being a tensor, a JSON
field, a code literal or an alleged property does not determine that role.
The current six-family inventory has 25 scalar literal assignments when the
shared normal/inverse spherical packet is counted once: droplet 9, spherical
3, BRE 1, jets 3, solids 2, aerosol 7. Counting both spherical specifications
as separate tensor inventories gives 28 positions, not three new independent
choices. These counts exclude declared exact algebraic factors and externally
supplied physical data; each author's inventory records those separately.

The physical core and catalogue do not remove the burden of evidence. A supplied
heat-return fraction, initial liquid temperature or particle-source yield can
become a free per-test parameter if chosen after viewing the outcome. A string
called provenance cannot prevent that. The eventual host adapter must bind each
input to its pre-cutoff evidence or an explicitly labelled scenario assumption.
An outcome-free internal library is therefore necessary but insufficient for a
qualified forecast service.

### Where does generalisation come from, and what destroys it?

Conservation, units, composition, symmetry and explicit boundaries let one
physical calculation apply to different material identities without a fitted
name switch. Its empirical generalisation still requires independent groups,
the same observable definition and an adequate physical regime. The new jets
branch demonstrates a meaningful distinction: signed axial gravity can enter a
derived momentum balance; a spherical scalar rescaling cannot represent the
shape change caused by a general gravity vector.

Independent checks also refuted narrower assumptions than full combustion
physics: a heat-capacity reference path can leave its property domain even when
the inlet is inside it; a solid temperature can leave the domain of its boundary
law; an aerosol observer can lose the initial source dependency. Correcting
those boundaries improves the meaning of a forecast. It is not evidence of an
unseen-group score improvement. Their exact counterexamples and successor checks
are in the two `astra_property_*_check.md` reports.

### How can the number of literals grow without train-directed freedom?

Expand an independently specified property grid, component inventory or exact
operator structure with fixed units, reference states and shared dependencies.
Tie repeated constants and retain source provenance. Do not count duplicated
entries as extra scientific information, or choose source ranges, grid density
and regime thresholds because a difficult TRAIN run improves. The current
catalogue's few admitted universal constants are an honest starting point; they
do not establish that this programme has scaled to thousands of defensible
material values. Unverified empirical entries remain unavailable to evaluation.

Published property/correlation coefficients retain their **external empirical
origin** when manually transcribed under the allowed class-A/B source rule.
For example the aerosol slip correlation is restricted to its stated particle
and gas regime; it is not a newly first-principles result or a NASA-fitted weight.
No script imports, fits, searches for or adjusts those coefficients here. Every
retained numerical choice, including a domain threshold, must remain inspectable.

### Where can honest uncertainty for an unseen condition come from?

Keep measurement uncertainty, property uncertainty, unresolved physical states,
model discrepancy, numerical error and unsupported regimes distinct. A point
law with a conserved energy ledger is not a calibrated distribution: the solid
review produced a large temperature error with an essentially exact ledger.
The successor's temporal refinement estimates discretisation discrepancy under
observed contraction; it explicitly does not certify a rigorous global bound,
spatial error or physical accuracy.

A committee can express separately justified scenarios with typed probabilities;
it cannot manufacture an unknown soot source, cool-flame mechanism or detector
calibration. Missing quantities remain unavailable. A future assumed spread must
be authored and registered before scoring, and retain its lack of calibration.
The present deterministic conditional kernels do not yet fulfil that deliverable.

### Where does typing stop and programmatic choice begin?

Evaluating a typed physical law, converting explicit units, conserving fluxes,
or refining a time grid against a declared numerical-error budget computes the
law's consequences. Selecting a diffusivity, reaction rate or mixture weight to
reduce outcome residuals chooses a model and is prohibited. Refinement must leave
the literal physical parameters unchanged and expose its work/error limits.
The review-informed solid solver does this; its error estimate is not a search
over material coefficients. Likewise a source-bound property conversion needs
its derivation and dependency identity; it is not an undocumented weight generator.

These distinctions determine the next integration work. Each model needs a real
observable, a truthful property/input binding, a literal release with executed
checks and a registered comparison. None follows merely from a successful kernel
test or from increasing the parameter count.

## Reading and derivation record

The companion `research/astra_all_nasa_dimensional_basis.md` records its own
primary sources, exact read extents and analyst exposure. This memo uses its
symbolic synthesis, not experimental outcomes or new numerical parameters.
The main author additionally opened the three primary web sources linked
above on 2026-11-14: returned Cantera balance/flux/boundary text and transport
page; NIST Appendix A through its returned propagation and Type A/B examples.
These are partial-page/tool-return reads, not full books. The six reductions,
coverage gates and build order are this author's proposals. Shared enum and
target limitations were checked directly in `ignis-core/src/contracts.rs`
and `contracts/wsp.rs`; no shared file changed.

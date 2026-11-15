# Model-lane requests to the document lane

## WSP-R01 — complete physical input inventory (all 19; conditions only)

What: inventory of fuel, diluent and solid formulations actually used, including
mixture basis/fractions, moisture/char or composite specification where known,
phase, initial temperature/pressure, geometry and source locators. Preserve raw
and SI fields, conflicting alternatives and missingness. Link each entry to exact
catalogue test identities and frozen group identity without outcome summaries.
Why: build one typed property block without material-name branches or invented
mixture definitions; certify which tests a family can admit.
Status: requested; no released artifact exists at the initial model-lane check.

## WSP-R02 — prospective apparatus and control inputs

What: exact test-to-script/source joins for commanded component flows, stages and
planned termination; flow calibration/standard-volume reference state, burner
geometry, coflow/solid sample orientation, gravity definition and boundary thermal
conditions. For FLEX-2, fuel/mixture-to-burn joins rather than folder-level guesses;
for BASS PSI-26, source-bound test rows. All 19 investigations where applicable.
Why: distinguish an imposed rate/stop from a predicted endpoint and avoid deriving
conditions from subsequent flame behaviour. No differencing consecutive chamber fills.
Status: requested; unresolved joins must remain unresolved.

## WSP-R03 — instrument-defined TRAIN targets and uncertainty

What: TRAIN-only released measurements, labels and discrepancies with method and
source identity, units, time origin, independent repeatability/calibration error,
view loss/censoring and apparatus limits. Droplet liquid/flame geometry; burner
optical versus physical endpoint; solid flame versus material front; BRE heat flux
versus imposed feed; jet geometry/lift-off; smoke/aerosol instrument moments.
Why: establish six-family error budgets, observables and registered metrics.
If a family has no TRAIN support, report that absence without held-run outcomes.
Status: requested. Do not convert missing calibration or error into zero.

## WSP-R04 — selected methods/properties pages and two-source support

What: released page cards and the few relevant pages for thermophysical property
reference states, valid T/p/composition ranges, formulation, calibration and
measurement operator. Only pages approved as not revealing per-test outcomes.
Include source DOI/URL/file, page/table/equation and provenance/dependence of the
second source; two mirrors of one table do not count as two independent sources.
Why: hand-type defensible physical values and domains, with two-source checks,
without a bulk document read or an opaque fitted mechanism download.
Status: requested; source-specific absence or disagreement is an acceptable answer.

## WSP-R05 — family-specific minimum evidence for the ranked changes

The six-family error budget supplies the following narrower requests. All are
requested, with no new release received at this checkpoint. Outcome information
is TRAIN only; conditions may cover all frozen groups. These are evidence requests,
not authority to change NASA values, group membership or target definitions.

| Family / ranking | Tests and minimum released evidence | Next decision |
|---|---|---|
| Droplets D1-D4 | TRAIN duplicate diameter/state checks, scale/time-zero uncertainty and prefix-only quality; source-bound FLEX-2 and other droplet fuel/mixture conditions | Distinguish observation failure, state uncertainty and property-domain errors |
| Spherical S1-S3 | CFI-G TRAIN source comparisons; immutable pre-ignition plans, MFC correction/reference states, thermocouple lag/cold junction and distinct TRAIN endpoint labels | Admit causal control inputs and separate extinction, scheduled stops and limit trips |
| BRE B1-B2 | Apparatus cards, geometry and command timeline; independently measured TRAIN response, calibration and sensor lag | Choose a target that is not merely the imposed feed rate |
| Jets J1-J3 | Nozzle/coflow geometry, mixture and temperature conditions; TRAIN optical/event targets with their measurement definitions | Distinguish reaction-sheet geometry, visible/UV geometry, lift-off and extinction |
| Solids F1-F3 | BASS PSI-26 conditions, loading/material formulation, calibrated flow and ignition history; TRAIN front/base traces and censoring/end labels | Forecast physical units and transient response without equating disappearance with quench |
| Aerosol A1-A3 | Source/blank/calibration and detector response cards; volume, sampling geometry/line losses and TRAIN signals | Establish whether concentration, a distribution moment or a detector event is identifiable |
| All groups Q-G | Release-safe split identifiers and independence unit for each investigation | Freeze metric, baseline, censoring/refusal treatment and grouping before scoring |

No request permits differencing consecutive tests in one chamber fill. Source
timestamps must establish availability before forecast cutoff; an archived
"as run" description alone does not prove prospective availability.

## WSP-R06 — first FLEX source-bound forecast: missing initial state

Priority: first preregistered TRAIN comparison of the new droplet kernel.
Requested tests: TRAIN identities in the existing FLEX methanol/heptane splits,
with prospective cutoffs 1/3/5 seconds. No continuation values are requested here.

Provide a release-safe availability inventory for liquid temperature and mass or
diameter at a declared initial time, liquid composition, ambient temperature and
pressure, oxygen composition, gravity and relative velocity. Distinguish measured,
commanded, assumed and unavailable. A diameter observation alone does not establish
liquid temperature; missing initial temperature must remain missing. For a burning
scenario, evidence of an established reaction sheet must be available before the
cutoff rather than inferred from its later outcome. Include exact locators and
availability timestamps; do not derive state by fitting diameter trajectories.

Model-lane property audit separately requires liquid density, true isobaric heat
capacity, latent heat and saturation pressure at compatible reference states;
gas density/heat capacity/conductivity/diffusivity/viscosity and composition-related
molar masses over the admitted thermal range. This request does not ask the
document lane to invent those values or certify constant properties from one
room-temperature datum. Saturation-path heat capacity cannot substitute for Cp.

Status: requested; no complete admitted packet or document-lane release exists
at this check. The comparator must report missing-input refusals transparently.

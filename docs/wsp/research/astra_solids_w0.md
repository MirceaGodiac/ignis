# Solid property W0 — independent Astra author

Author: `gpt-6-astra`. Outcome exposure: none. This is a conditional solid-phase
forecast, not a qualified autonomous flame-spread model. No material-name dispatch.

## Derivation fixed before implementation

Consider a rectangular, fixed-volume porous solid on a persistent inert matrix.
Let x run along the specimen and z through its thickness; side edges are insulated.
Each finite volume contains inert mass m0 and reactant masses ma. All constituents
share temperature T. Properties are explicitly supplied constant-domain values.
The inert matrix remains after reactants escape; conductivity is a supplied
effective matrix-plus-pore value, explicitly assumed invariant during conversion.
This excludes collapsing films, melting/dripping, changing porosity, char chemistry,
and shape change. Different constituents are composition, not material-name branches.

Choose a common sensible-energy reference Tr. Stored energy is
E = (m0 c0 + sum(ma ca)) (T-Tr). A face between cells carries
Qij = k Aij (Tj-Ti)/dij; the same number enters one cell and leaves the other.
At each exposed face Q = A [q_abs + h (Tgas-T) + epsilon sigma (Trad^4-T^4)].
Here q_abs is independently supplied absorbed incident power, not burning rate;
the other terms must not be counted in q_abs again. Front/back histories are
piecewise constant controls with declared flow vector, oxygen fraction, pressure,
gravity vector, orientation normal and provenance valid for that exact scenario.
These quantities cannot determine h or flame feedback by themselves.

For constituent a, supplied first-order kinetics give
ka(T) = Aa exp(-Ea/(R T)); dm_a = ma [1-exp(-ka dt)].
This is a first-order operator split with exact frozen-temperature reaction mass,
not a universal pyrolysis threshold. Each released unit mass exports ca(T-Tr)
sensible energy; supplied endothermic reaction enthalpy La also consumes energy.
After conductive heating, remove sum(dm_a [ca(T-Tr)+La]) and divide by the
remaining heat capacity to recover T. A step that crosses the supplied property
domain is rejected rather than clipped or silently made energy-limited.
The caller must reduce dt; no physical parameter is adjusted.
This closure assumes each escaped gas constituent has the same constant sensible
heat capacity as its parent solid, making the supplied constant reaction enthalpy
consistent with the common reference. Materials violating this restriction need
separate gas thermochemistry and temperature-dependent reaction enthalpy.

Mass accounting: initial reactant = remaining reactant + cumulative released gas.
Energy accounting: initial stored + cumulative boundary input = stored +
cumulative gas sensible export + cumulative reaction enthalpy consumption.
Internal face fluxes cancel exactly algebraically. The persistent inert matrix
keeps heat capacity positive even at complete fuel exhaustion.

Conversion is one minus remaining/initial reactant. A caller-declared conversion
isocontour defines the solid front; multiple disjoint regions are retained as
intervals. Finite sample bounds and a caller-declared residual-mass resolution
define exhaustion. Neither is a luminous flame front or quench event. Gas-phase
flame feedback, luminosity and extinction are unavailable; view loss is a separate
observation flag. Predicted release of volatiles must not be labeled burning.
First-order reactant decay has an asymptotic tail: an exhaustion report requires
a strictly positive declared mass resolution. Zero resolution disables that
predicate; floating-point underflow does not prove an exact finite exhaustion time.

## Primary-source reading ledger

- NIST FDS Technical Reference, sixth edition Vol 1, 2013 version 6.0.1:
  https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.1018e6.pdf
  Only sections 7.1–7.2 were intentionally read: PDF pages 90–95 (printed 67–72),
  equations 7.2–7.7, 7.13–7.15, 7.22–7.34. Search/open excerpts also exposed the
  section 7.2.3/7.2.4/7.2.5 headings and snippets on PDF 96. The PDF tool initially
  returned frontmatter pages 0–13 and the table of contents. The allowed section
  7.1.2 excerpt also included a generic statement of near-wall-model validation
  for low-Reynolds-number heated channel flow and use for an airplane cargo
  compartment. No per-experiment numerical results were exposed or used; that
  optional model is not adopted. No LES default coefficient is adopted. Generic conservation and
  first-order Arrhenius are class A/B structure, not evidence of qualified spread.
- NIST constants search results: https://physics.nist.gov/cuu/pdf/all.pdf
  (molar-gas-constant row only, search excerpt),
  https://pages.nist.gov/feasst/docs/source/physical_constants.html
  (MolarGasConstant value excerpt), and
  https://physics.nist.gov/cgi-bin/cuu/Value?sigma%7Csearch_for=stefan+boltzmann
  (Stefan–Boltzmann value excerpt). Malformed `Value?r` / `Value?sigma` requests
  returned only errors. Generic search snippets included other constants sources;
  only NIST values are used.
- Local interface reads: root AGENTS.md; `contracts/wsp.rs` lines 1–580;
  `contracts.rs` lines 1–260; `assemble.rs`; `physical_core.rs`; crate/root Cargo
  manifests. `physical_core.rs` includes its own synthetic algebra tests; these
  were incidentally in the allowed file read. No family candidates, test/holdout
  outcomes, NASA result papers/data, research documents or feedback were read.

## Literal number inventory

Physical universal constants, manually authored in ops: R = 8.31446261815324
J/(mol K), exact SI product expressed as decimal; sigma = 5.670374419e-8
W/(m2 K4), truncated representation of the exact SI-derived constant.
Neither is fitted. Material, reaction and boundary properties have no defaults;
all are supplied with raw/SI unit, source, locator, origin, uncertainty, phase,
temperature/pressure domain and sensible-energy reference.

Algebraic integers: 0 (absence, signs and array origins), 1 (complements and order),
2 (half-cell and central geometry), 4 (radiation power); exponent 3 is the
derivative of the fourth-power radiation term. Mathematical pi in the exact
semi-infinite heat-equation limit uses Rust's standard-library constant.
Numerical choices:
explicit finite-volume heat step; frozen-temperature exponential reaction split;
caller supplies mesh, maximum step, conservation tolerances and observation
conversion/exhaustion resolutions. A conductive positivity bound is checked per
cell, including boundary radiation derivative. No adaptive parameter search.
Synthetic tests use explicitly invented dimensioned properties; they are not
material determinations or physical release coefficients.

## Qualification boundary

Temperature, released mass, stored energy and conversion intervals are useful
conditional observables given known future thermal boundaries. Oxidative solid
reactions, gas chemistry, oxygen transport, buoyancy, flow-dependent transfer,
flame-to-solid radiation, extinction and luminous-front observation need separate
closures. Oxygen, gravity and orientation are boundary-condition context, not
magic scalar corrections. The current kernel therefore cannot establish an
unconditional spread safety limit, extinction threshold, or full spread qualification.
No physical uncertainty propagation is claimed; supplied uncertainty is retained.

## Executed validation

- `rustc --edition 2021 --test crates/ignis-wsp/src/property_solids_w0.rs -o configs/models/solids/property-w0/cleanroom-astra/solids-kernel-check.exe`, then
  `solids-kernel-check.exe --nocapture`: **10 passed, 0 failed**, 0.11 s reported by
  test runner. Synthetic only. Output executable/PDB removed after the run.
- `rustfmt --edition 2021 --check crates/ignis-wsp/src/property_solids_w0.rs configs/models/solids/property-w0/cleanroom-astra/literal_contract.rs`: passed after formatting.
- Core contract harness prepared in `literal_contract.rs`, with a standalone
  Cargo manifest. The author did not run Cargo because the coordinator reported
  disk pressure and reserved the existing build target. Coordinator must run this
  contract test before assembly. No CI, live API, evaluation or NASA data used.

The tests cover exact uniform/no-flux limits, reaction mass and energy including
volatile sensible export, full reactant exhaustion without quench inference,
disjoint finite conversion intervals, conductive positivity and transactional
failure, refinement toward the semi-infinite solution, reaction-step convergence,
composition/name invariance, heating-control transitions and boundary cooling,
and invalid units/reference/domain/environment rejection. They establish internal
numerical behavior only. The thin closed form and half-space surface closed form
are separate exact limits; the finite-volume kernel converges toward the thick
limit and does not claim an exact coarse-grid thick solution.

`freeze.json` records SHA-256 of the authored kernel, specification, literal ops,
metadata, this derivation and the test harness after these synthetic checks.

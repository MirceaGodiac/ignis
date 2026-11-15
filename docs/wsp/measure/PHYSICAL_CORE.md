# Outcome-free physical/property algebra

`crates/ignis-wsp/src/physical_core.rs` is an internal scientific prototype. It
implements algebra on supplied physical inputs. It is not a model release, a
prediction service, a new shared contract, or evidence of experimental accuracy.
There is no property lookup, fuel-name switch, mechanism import, coefficient
selection, fit, optimizer, or outcome-dependent interpolation.

## Inputs and applicability

`Finite` and `Positive` reject nonfinite inputs. Constructed properties and gas
mixtures keep their validated fields private. Dimensional fields name their SI
units; the input property unit enum distinguishes kg/mol, J/(mol K), and
J/(kg K). The supplied raw value/unit is retained. This prototype accepts SI
property entries only; converting a non-SI source while retaining its original
value/unit and locator belongs to a future property-ingestion boundary.

Each supplied property retains phase, a closed temperature/pressure domain,
reference temperature/pressure, thermochemical reference convention and identity,
source, locator, dependency identities, origin, and explicit uncertainty status.
No uncertainty is propagated or inferred. A scalar property with a domain wider
than a single state is the supplier's declared constant-property approximation;
the module neither interpolates nor extrapolates it. Supplying a domain is not
evidence that the underlying property has been checked experimentally.

Gas components require gas-phase molar mass and heat capacity. Every mixture
component must cover the requested state, including zero-fraction entries. This
conservative interface requires the same reference convention, reference state,
and reference identity for both properties and all components. It does not
convert between otherwise reconcilable reference conventions. The heat capacity
alone would not determine an enthalpy zero; the reference guard anticipates the
energy boundary and is stricter than the algebra for heat capacity requires.

Neutral CHONS species and explicitly represented helium and argon are supported.
Helium/argon have their own atom counts and caller-supplied atomic masses; neither
is substituted with nitrogen or represented as a zero-element species. They
contribute no oxygen demand, but retain their supplied mass and heat-capacity
properties and their separate elemental inventories. Other unsupported elements
and charged species must not be encoded as another species or dropped. Missing
material composition is not an inert species. Species identity joins records and orders
sums; it does not select a physical value. The optional atomic-mass check verifies
that a supplied molecular mass equals the sum of its supplied atomic masses.
Without that check the molar mass is a declared input, not a verified elemental
mass balance. The gas constant and atomic masses are named caller inputs; their
source records remain the caller's responsibility. No values for them are built in.

## Implemented laws

For mole fractions `x_i`, mass fractions `y_i`, and supplied molar masses `M_i`:

```text
M_mix = sum_i(x_i M_i)
y_i   = x_i M_i / M_mix
x_i   = (y_i / M_i) / sum_j(y_j / M_j)
rho   = p M_mix / (R T)
Cp_i  = cp_i M_i                  [when supplied on a mass basis]
Cp_mix = sum_i(x_i Cp_i)
cp_mix = Cp_mix / M_mix = sum_i(y_i cp_i)
```

Fractions must individually lie in `[0,1]` and sum to one within the explicitly
supplied numerical error budget. Only then are they normalized to remove that
roundoff residual. Duplicate identities are refused. Sorting before summation
makes input permutation produce the same ordered arithmetic. Ideal-gas property
evaluation additionally requires every component to have `Cp_i > R`, so its
ideal-gas `Cv_i` is positive. Density and heat capacity are conditional on the
ideal-gas assumption. These equations are consistent with the primary
[Cantera ideal-gas formulation](https://cantera.org/3.1/cxx/d7/dfa/classCantera_1_1IdealGasPhase.html).

Complete oxidation to `CO2`, `H2O`, `N2`, and `SO2`, with helium and argon
retained unchanged, gives the signed oxygen demand for the neutral formula
`C_c H_h O_o N_n S_s He_he Ar_ar`:

```text
q_i [mol O2 / mol species] = c + h/4 + s - o/2
q_stream [mol O2 / kg]     = sum_i(y_i q_i / M_i)
q_stream [kg O2 / kg]      = q_stream [mol O2 / kg] * (2 M_atomic_oxygen)
Z_st                      = -q_oxidizer / (q_fuel - q_oxidizer)
```

The factors two and four follow from atom counting. Nitrogen is assigned to
`N2`, so no NOx demand is included. Oxygen excess is negative, never clamped to
zero. The mass-demand method first checks every component against the supplied
atomic masses. The `Z_st` method requires strictly positive fuel-stream demand
and strictly negative oxidizer-stream demand, and compatible stream records.
`Z_st` is the mass fraction of the fuel stream, including its diluents. This
same definition supports mixed fuels and either apparatus orientation; apparatus
names do not enter the calculation. It is a conserved-composition coordinate,
not a statement that a flame exists, oxygen is kinetically available, or those
products are the equilibrium products.

Composition mixing explicitly computes `y_i = z y_i,a + (1-z) y_i,b` and checks
the output property domains at a caller-specified output state. A shared species
identity with conflicting properties or provenance is refused. Element inventory
is returned in C/H/O/N/S/He/Ar order as mol atoms/kg, allowing conservation to be checked
without a reaction solver. No mixed temperature or enthalpy is inferred.

With supplied positive length `L`, diffusivity `D`, kinematic viscosity `nu`,
thermal diffusivity `alpha`, vector velocity `u`, vector gravity `g`, and signed
contrast `delta = (rho_reference-rho_comparison)/rho_reference`:

```text
t_diff = L^2 / D
Pe = |u| L / D       Sc = nu / D
Pr = nu / alpha      Le = alpha / D
G = |g| delta L^3 / D^2
Ri = |g| delta L / |u|^2     [only when |u| > 0]
```

Velocity and gravity require the same named Cartesian frame. Positive comparison
density requires `delta < 1`; negative contrasts are allowed. Zero velocity gives
`Pe=0` and an explicitly absent Richardson number. Zero gravity or zero density
contrast gives `G=0`, with no epsilon denominator. Diffusion, `Sc`, `Pr`, and `Le`
remain defined. The full gravity vector is retained. Presence flags distinguish
imposed advection, gravity, and density contrast; they are not claims about
laminar/turbulent, flame, or extinction regimes. `G` and `Ri` are signed and
cannot encode geometry-relative orientation by themselves. The group definitions
are dimensional reductions proposed in the first-principles memo, not empirical
correlations. Transport coefficients are supplied independently: no mixture rule,
unity-Lewis approximation, or geometry-specific transfer coefficient is hidden.

At boundaries the module checks that every species has a diffusive mass-flux
vector in the same frame and that their sum is zero in each direction. It never
silently repairs a species flux. This is the mass-averaged conservation condition
illustrated by the [Cantera flux equations](https://cantera.org/3.1/reference/onedim/governing-equations.html#diffusive-fluxes).
A separate guard refuses complete-combustion heat added on top of enthalpies
including formation energy. It does not solve an energy or interface balance.

## Numerical scope and verification

Arithmetic uses `f64`. `NumericalTolerance` has caller-supplied absolute and
relative bounds and no default. Acceptance means
`abs(a-b) <= absolute + relative * max(abs(a), abs(b))`; absolute bounds carry
the comparison's units. These are engineering error budgets, not science
parameters, uncertainty estimates, or permission to normalize arbitrary input.
Callers must justify their chosen bounds. Nonfinite intermediate/final results
are refused when checked; conversion and normalization reject loss of a positive
fraction to zero. Composition mixing rejects an unrepresentable positive product
of stream weight and species mass fraction; exact zero contributions and endpoint
mixing remain valid. Nonzero signed Richardson numbers are checked in magnitude
before restoring their sign, so underflow cannot masquerade as absent gravity or
density contrast. Required positive diagnostics likewise reject zero/underflow. This
is not a general arbitrary-precision or extreme-dynamic-range implementation.
No physical value is clipped, and no denominator is replaced by an epsilon.

The inline fixtures are explicitly `origin = SyntheticFixture`, with source
`synthetic_fixture`; their invented masses, heat capacities, states, and atomic
masses describe no real fuel. The `1e-12` absolute/relative test budget covers
short floating-point sums and conversions in these small fixtures only. It is
not used as an operational default or a physical threshold.

The isolated standard-library test binary checks species permutation,
mass/mole round trips and units; mixed-fuel oxygen balance and element
conservation; nonfinite, unnormalized, negative and duplicate inputs; domain,
phase and reference incompatibility; zero velocity/gravity limits and vector
retention; diffusive mass conservation and atomic/molecular mass consistency;
material-name invariance, representational extremes, and nonpositive ideal-gas
heat capacity at constant volume. Dedicated helium/argon tests verify inert
oxygen demand, separate inventories, caller-supplied atomic-mass consistency,
mass/mole conversion, mixture density/heat capacity, and seven-element
conservation at a diluted-stream stoichiometric coordinate.

Executed during implementation:

```powershell
rustfmt --edition 2021 --config skip_children=true crates/ignis-wsp/src/physical_core.rs
$physicalCoreTest = Join-Path $env:TEMP 'ignis-physical-core-tests.exe'
rustc --edition 2021 --test crates/ignis-wsp/src/physical_core.rs -o $physicalCoreTest
& $physicalCoreTest
```

The post-review isolated run passed all ten tests. The integration owner has added
the module declaration in `lib.rs`; the next workspace gates check that integration.
Isolated tests alone do not establish workspace integration. No registered
release, shared schema, Cargo manifest, lockfile, or generated TypeScript changed
in this implementation.

The independent review in `docs/wsp/research/astra_physical_core_review.md`
identified the positive stream-contribution and Richardson-underflow cases.
Before fixing them, the two new regression tests failed and the original seven
tests passed. After the fixes, those regressions also check exact-zero and
endpoint behavior, a nearby representable trace contribution, both contrast
signs, and ordinary signed Richardson values. The report describes the earlier
CHONS-only source; helium/argon support was added in response to its coverage
finding. No experimental outcome informed these revisions.

## Deliberately unimplemented

Property acquisition/qualification, non-SI ingestion, temperature-dependent
property models, nonideal mixtures, phase change, enthalpy integration, kinetics,
transport mixture rules, evolving conservation equations, gravity histories,
charge transport, radiation, soot/aerosol formation, interface energy balances,
flame/extinction predictions, measurement operators, uncertainty propagation,
and experiment/model qualification remain outside this module. There is no
claim of coverage of a NASA investigation or of any model being trained or
validated.

## Reading and derivation record

Implementation date: 2026-11-14. Read the repository root `AGENTS.md` and the full
`docs/wsp/dossier/ASTRA_FIRST_PRINCIPLES_ALL_NASA.md`. Inspected only the package
manifest and static library module declarations for integration context. Did not
read linked result documents, model releases, experimental files, or data directories.
For the revision, also read the independent physical-core review named above.

Primary web reads were partial tool-return excerpts, not full-source reviews:

- [Cantera species thermodynamics](https://cantera.org/3.1/reference/thermo/species-thermo.html):
  returned lines 71-122, especially the reference-pressure/state distinctions and
  the heat-capacity/enthalpy reference relationship. No published coefficient
  values were copied; the polynomial mechanisms shown there are not implemented.
- [Cantera phase thermodynamics](https://cantera.org/3.1/reference/thermo/phase-thermo.html):
  returned lines 71-132, identifying the ideal-gas model and its separate domain.
- [Cantera IdealGasPhase](https://cantera.org/3.1/cxx/d7/dfa/classCantera_1_1IdealGasPhase.html):
  returned lines 17-440, including the standard/partial-molar relations, ideal-gas
  molar volume, and mass-fraction mixture-coordinate definition. No library or
  mechanism was imported into the implementation.
- [Cantera one-dimensional governing equations](https://cantera.org/3.1/reference/onedim/governing-equations.html):
  returned lines 71-175, including species/energy equations, mass-conserving
  diffusion, and inlet/symmetry conditions. Their stagnation-flow solver was not
  adopted as a general physical model.

The CHONS oxygen equation, unchanged by inert He/Ar atom bookkeeping, and the
two-stream root are direct atom-balance and linear mixing derivations. The
transport groups are definitions/dimensional analysis
from the memo. This implementation introduces no numerical author-model weights.

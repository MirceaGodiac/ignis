# Astra spherical property W0: conditional physical kernel

Author: **gpt-6-astra**, OpenAI. Numerical authorship is one-shot and outcome-blind.
This implements a useful conditional reference and a conservative transient extension. It
does **not** complete or qualify the ACME burner family. In particular, finite-rate
extinction, optical observables and nonzero-gravity momentum remain unsupported.
The coordinator's choice to request this architecture may be data-informed; that is
distinct from the author's numerical exposure.

## Exposure and sources actually read

No experiment outcome, NASA report/result, dataset, development evaluation, prior
candidate/forward, holdout, validation partition, model release or historical author note
was inspected. The only repository interface reads were root AGENTS.md, the WSP Cargo
manifest, contracts.rs, contracts/wsp.rs, contracts/common.rs, assemble.rs and physical_core.rs.
The outcome-free core supplied composition/property algebra, not empirical properties.
The generic PDF skill was read for extracting equations. No memory files were used.

* [Matalon, CEFRC 2013 Lecture 10](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-10.pdf):
  file downloaded; first 130 text characters per page were inspected across its 29 pages;
  a theory-keyword search displayed the final 320 characters of matching pages
  5, 9–12, 14–15, 18, 22, 24–25, 27–29. Pages 25–27 were then read in full text,
  and page 25 rendered and visually inspected. These establish the equal-diffusivity
  conserved scalar, stoichiometric sheet, unity-Lewis restriction and adiabatic
  enthalpy interpretation. They are not spherical-burner validation. The spherical
  flux solution below was independently derived. Download SHA256:
  `3f139912cd10d7671b701a8fe5e8c4513446f35c9c1c6c1473d5ae01118fa67a`.
  The downloaded PDF and rendering were temporary; they are not redistributed.
* [Cantera 3.1 species thermodynamics](https://cantera.org/3.1/reference/thermo/species-thermo.html):
  retrieved page including reference-state definitions, integrated enthalpy,
  polynomial descriptions and constant-cp subsection, especially rendered lines
  83–146. Used to keep formation/reference enthalpy distinct from sensible heat
  and to identify constant cp as a declared approximation. No polynomial
  coefficient table or experimental outcome was acquired.
* [Cantera 3.1 governing equations](https://cantera.org/3.1/reference/onedim/governing-equations.html):
  retrieved equations, diffusive-flux definitions and boundary conditions,
  rendered lines 71–200. This documents species/energy transport and the need for
  momentum and chemistry closures. Its stagnation geometry is not used as a
  spherical solution; its nonlinear solver is not used.
* [NIST CODATA 2022 Stefan–Boltzmann constant](https://physics.nist.gov/cgi-bin/cuu/Value?sigma%7Csearch_for=stefan+boltzmann):
  read the search-returned official value and uncertainty statement. The initial
  bare `?sigma` page request failed; the searchable value page supplied the number.
  The underlying SI constant is exact; the manually written finite decimal is
  truncated. Its omitted digits are numerical precision, not calibration or a
  physical uncertainty estimate. No other search result was used for a coefficient.

The PDF keyword snippets contain generic fast-chemistry/extinction theory, not modeled
test outcomes. No incidental outcome exposure occurred.

## Derived steady solution

Let `m` be the outward injected mass flow, `rb` the burner radius and
`Gamma = rho*D` a supplied constant mass-diffusion coefficient. Define `w` as the
mass fraction originating from the injected stream. Spherical continuity gives
`A(r)*rho*u = m`, with `A(r)=Omega*r²`, `Omega=4*pi`. A conserved scalar has flux

```
Jw = m*w - Omega*r²*Gamma*dw/dr = m.
w(infinity) = 0.
a = m/(Omega*Gamma).
w(r) = 1 - exp(-a/r).
dw/dr = -(a/r²)*exp(-a/r).
```

The inner condition is the injected **flux**, not `w(rb)=1`. The latter would
change the flux problem. `expm1` and `log1p` preserve the far-field/small-fraction
limits without an empirical epsilon. No root search is used.

Elemental demand per mole for the declared complete products is
`C + H/4 + S - O/2`. Mixture mass fractions and supplied molar masses give the
signed oxygen demand per kg of each stream. Their zero crossing gives `zst`, the
stoichiometric fuel-origin mass fraction. No fuel name enters a calculation.
Fuel/oxidizer streams must not contain an opposing-sign reactive component.

For normal injection `z=w`; for inverse injection `z=1-w`. Therefore

```
q = zst                 (normal fuel injection)
q = 1-zst               (inverse oxidizer injection)
rf = a / [-log(1-q)]
m_min = Omega*Gamma*rb*[-log(1-q)].
```

An exterior sheet requires `m > m_min`. Failure is a geometric/admission failure,
**not** a prediction of extinction. `rf` is a reaction-sheet radius, not visible
or UV diameter. The existing contract quantity is `Temperature`; no optical
quantity has been reused for this diagnostic.

The caller supplies fuel, oxidizer and stoichiometric product mixtures with
formation enthalpies per kg and species cp. Products must have the elemental
compositions of CO2/H2O/N2/SO2/He/Ar; each element per kg is balanced against the
stoichiometric reactants. This is complete chemistry without dissociation,
not equilibrium speciation. All references and gas phases must agree.

For `z <= zst`, product mass fraction is `z/zst`; the remainder is excess
oxidizer stream. For `z > zst`, product fraction is `(1-z)/(1-zst)`; the remainder
is excess fuel stream. With these fractions `Yi`,

```
h_in(z) = z*[hf0 + cpf*(Tf-Tref)] + (1-z)*[ho0 + cpo*(To-Tref)]
hf_mix = sum(Yi*hfi0)
cp_mix = sum(Yi*cpi)
T = Tref + [h_in(z)-hf_mix]/cp_mix.
```

Formation energy is counted once. For common species diffusivity and
`lambda = Gamma*cp_mix` (unity Lewis), including species enthalpy diffusion makes
the total enthalpy flux `m*h-A*Gamma*dh/dr`. This supports the same conserved
scalar construction even when the three constant mixture heat capacities differ.
The temperature is not an invented growth law. It is a conditional enthalpy
balance on an **already established** fast-chemistry burning branch.

## Transient and apparatus extension

`evolve_radial` advances fuel-origin fraction and total enthalpy in user-supplied
spherical shells. It uses explicit Euler, outward upwind advection and centred
diffusion with a shared face flux. Cell mass is supplied frozen density times
exact shell volume `Omega/3*(rout-rin)*(rout²+rout*rin+rin²)`. Density/momentum
feedback from heating is not solved. The apparatus has a finite outer ambient
boundary: diffusion uses ambient composition/enthalpy; outward advection uses
the last cell. Inner total scalar/enthalpy fluxes equal the injection fluxes.

The initial field includes a time origin, elapsed seconds, composition fractions,
total enthalpy and provenance. The method reconstructs complete-reaction
composition algebraically and obtains temperature from conserved enthalpy.
An initially unreacted cold mixing layer cannot be used to infer ignition.

For supplied Planck-mean absorption `kappa`, local thermal equilibrium and negligible
reabsorption/scattering, integration of blackbody intensity over solid angle gives
volumetric loss `4*kappa*sigma*(T^4-Tbath^4)`. This is an optically thin closure,
not a soot or line-by-line radiation model. Returned `kappa*(rout-rb)` is a
diagnostic; values at least unity are refused, while smaller values alone do not
establish negligible self-absorption. The caller must supply the applicability
source/assumption. A supplied conductance gives inner wall loss
`Hwall*(Tfirst-Twall)`; it may be negative when the wall heats the gas. There is
no inferred heat-transfer coefficient or fitted radiation multiplier.

For each cell, the explicit step must satisfy

```
dt * [m + Kleft + Kright + 16*kappa*sigma*max(T,Tbath)^3*V/cp
      + Hwall/cp] / cell_mass <= 1,
Kface = Aface*Gamma / centre_distance.
```

`Hwall` contributes only to the first cell. The radiative derivative uses the
hotter bath/cell temperature. This provides a monotonic loss bound for a frozen
composition; combined changes are additionally checked by reconstructing every
new thermochemical state. No failed composition, temperature or step is clipped.
The caller chooses timestep/count; the forward never searches a better step.
Every internal flux cancels in global scalar/energy budgets, which are checked
against explicit arithmetic tolerances. No resource limit is disguised as a
physical coefficient.

## Literal and input inventory

The three authored assignments, written directly into `literal-ops.json`, are:

| Tensor | Literal | Role and origin |
|---|---:|---|
| `solid_angle_sr` | 12.566370614359172 | Manually written binary64-scale value of mathematical `4*pi`; geometry, no data fit. |
| `lewis_number` | 1.0 | Explicit equal thermal/species diffusivity restriction, not an estimated property. |
| `stefan_boltzmann_w_m2_k4` | 0.00000005670374419 | Manually transcribed NIST CODATA 2022 displayed decimal, SI W/(m² K⁴). |

The kernel requires these literals; physical adaptation requires a new derivation
and release, not retuning them. Three separate scalar tensors preserve units.
Spec bounds have strictly increasing endpoints; they are envelope checks, not
claims that the derivation works for arbitrary values within them.

Exact algebraic integers are separate from predictive coefficients: 0/1 define
fractions and absence; 2 is midpoint/oxygen atom count; 3 integrates sphere
volume; 4 is solid-angle/radiation and hydrogen stoichiometry; 16 is the derivative
of `4*T^4`. SI prefix conversions 1000 and 0.001 are exact decimal unit factors.
The literal atom counts defining allowed complete products are chemical formulas.
Tests use additional artificial numbers solely as `synthetic_fixture` inputs.

Material and apparatus values are **inputs**, not hidden learned weights:
species elemental counts/molar masses/cp, stream fractions, three mixture formation
enthalpies, pressure and inlet temperatures; radius, mass flow, `rho*D`; gravity
and ambient-flow vectors; transient geometry, frozen density, absorption,
bath/wall temperatures, wall conductance and initial fields. No default fuel
property, fitted transport law, arbitrary chemical rate, noise or hazard exists.
There is no uncertainty interval: property uncertainty is retained but not
propagated or calibrated.

`ScalarInput` retains raw value/unit, converted SI, gas/scenario domain, common
formation reference and source/locator/dependency/uncertainty metadata. SI-only
absolute uncertainty bounds are exposed separately after converting the original
raw-unit bounds; no probabilistic meaning is added. SI-only
vectors and arrays keep their original SI values, explicit units and source;
they do not silently accept g-units, normal litres or standard volume flow.
`GasMixture` accepts only explicitly supplied composition/properties and preserves
species provenance. Its already-admitted normalization is arithmetic, not a fit.
Pressure must match the common reference. Species constant-property domains must
cover the full temperature path; extrapolation is refused. Formation samples
apply at their reference state. Broad domains assert a supplier approximation,
not accuracy established by the kernel. A provenance label cannot itself prove
that the supplied thermochemistry is scientifically credible.

## Gravity and remaining scientific gaps

Actual gravity is a required vector with frame and provenance. Any nonzero
value is refused. The same applies to imposed ambient crossflow. Uniform gravity
with density gradients generates a preferred direction; a scalar correction to
the radial solution cannot satisfy the missing momentum equation. This includes
residual microgravity and partial gravity. Such cases remain **unqualified and
unsupported**, rather than silently mapped to zero. A real extension needs
multidimensional continuity/momentum/species/enthalpy, time-resolved acceleration,
boundaries and source-bound viscosity/transport/thermochemistry. No general-gravity
coverage is claimed here.

Other missing closures/inputs: finite-rate chemistry and ignition history;
temperature-dependent/differential diffusion; thermal expansion momentum;
finite-rate extinction and radiative quenching criteria; dissociation/soot;
optical emission, camera response and contour definition; validated material
property domains; apparatus support losses and externally measured initial fields.
The thermal branch may cool while fast chemistry remains imposed. That does not
predict flame survival. NASA-qualified errors, intervals and survival statistics
require a separately authorized evaluation after these gaps are addressed.

## Verification and integration boundary

Executed locally, with no Cargo build or networked evaluation:

```
rustfmt --edition 2021 crates/ignis-wsp/src/property_spherical_w0.rs configs/models/spherical/property-w0/cleanroom-astra/check.rs
rustc --edition 2021 --test configs/models/spherical/property-w0/cleanroom-astra/check.rs -o configs/models/spherical/property-w0/cleanroom-astra/check.exe
configs/models/spherical/property-w0/cleanroom-astra/check.exe property_spherical_w0 --nocapture
```

Result: **6 passed, 0 failed**, 10 outcome-free physical-core tests filtered out.
Tests exercise both injection directions, analytic radius/flux/energy limits,
far-field cancellation, flow scaling, units/references/invalid products/domain,
gravity rejection, finite-volume scalar/energy budgets, cooling, zero-loss limit,
optical-depth and explicit-step rejection, and time-step halving consistency.
These verify implementation on synthetic algebraic cases, not real combustion accuracy.
The generated test executable/PDB is not a release artifact and can be regenerated.

`spec-normal.json` and `spec-inverse.json` are actual `ModelSpec` payloads;
`patch-meta.json` is a `PatchMeta` envelope and `literal-ops.json` contains actual
`LiteralOp` entries. The coordinator must deserialize/validate them using the
current contracts, bind each patch to the corresponding spec hash, materialize
the tensors and enforce mode/spec coherence. That contract validation is not
claimed as run by this author. `Parameters::from_literals` expects tensor order
solid angle, Lewis number, Stefan–Boltzmann constant. The author did not change
lib.rs, registries, shared records/contracts, runtime adapters or audit code.
No release was created or committed. Any eventual literal release must remain
`experimental`, `not_trained`, with this exposure and missing-closure inventory.

Freeze hashes of the source, handwritten JSON and this document are recorded in
`configs/models/spherical/property-w0/cleanroom-astra/freeze.json`. Hashing merely
records authored bytes; it does not generate weights or establish model quality.

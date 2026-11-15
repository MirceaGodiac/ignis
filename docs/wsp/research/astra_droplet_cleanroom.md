# Astra clean-room property droplet W0

Author: `gpt-6-astra`. Date: 2026-11-14. State: experimental, not trained.

This is a new model, not a revision of an earlier candidate. It predicts a liquid
volume-equivalent diameter from the component mass inventory, interfacial vapor
equilibrium, a quasi-steady spherical gas film, and a time-dependent liquid
enthalpy balance. An explicitly selected, already-established, infinitely-fast
reaction sheet adds conditional heat release, oxygen consumption and geometric
sheet radius. None of those quantities is a measured result.

## Clean room and evidence boundary

Read only root `AGENTS.md`, the allowed core contracts (`contracts.rs`,
`contracts/wsp.rs`, `contracts/common.rs`), WSP `assemble.rs`, the outcome-free
`physical_core.rs`, and Cargo manifests. The physical core supplied only its
element-count oxygen-demand algebra; no material coefficient was copied.
The synthetic algebra tests in that allowed interface were visible. No goals,
dossiers, previous author research, candidate forwards/weights, NASA reports,
NASA observations, development runs, holdout, validation/test results, or model
registry releases were read. No documentary processing or measurement ran.

Generic theory was retrieved through the web tool. A request at PDF line 700
returned wider neighboring Matalon pages 6–12, including a qualitative statement
about experimental agreement of the d² law on page 11 and example empirical
convection factors on page 12. This incidental exposure was immediately disclosed
to the coordinator. No experimental numerical outcomes were shown; none of those
empirical factors was used. The coordinator confirmed that this generic theory
exposure does not constitute modeled-result feedback. The patch has empty
datasets, development feedback and prior candidates. Generic URLs have stable
`author-packet:` aliases in `exposure.sources`, with URL/hash mappings in the
source inventory. These aliases describe this author's provenance packet and do
not claim registered application Source records or invent dataset identities.

A later scheduling-status query incidentally returned other lanes' completion
summaries (aerosol and solids feature descriptions, test counts and a file hash).
It showed no physical coefficients, forecasts, experimental results, files or
released candidate identifiers. This was immediately disclosed. No physics or
literal assignment was changed based on those structural status summaries.

The source byte identities and exact read extents are in
`configs/models/droplets/property-w0/cleanroom-astra/source_inventory.json`.
Where upstream downloading failed, the hash is explicitly absent: a web-extracted
page is not represented as a verified copy of the upstream bytes.

## Source contributions

- [Matalon, CEFRC 2013, Lecture 12](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-12.pdf),
  pages 3–9: spherical quasi-steady droplet transport, interface balances,
  vapor-pressure equilibrium and the infinitely-fast sheet limit. The present
  evolving mixture and sensible-enthalpy implementation is independently derived
  below; the lecture's prescribed boiling-surface temperature is not imposed.
- [Cantera 3.1 species thermo](https://cantera.org/3.1/reference/thermo/species-thermo.html),
  introductory standard/reference-state and heat-capacity integrals: retain the
  reference convention and integrate enthalpy consistently. No polynomial
  coefficient or mechanism was imported.
- [Cantera 3.1 phase thermo](https://cantera.org/3.1/reference/thermo/phase-thermo.html),
  introductory phase models: ideal-gas and nonideal-liquid assumptions are
  distinct closures, not interchangeable property labels.
- [Cantera 3.1 governing equations](https://cantera.org/3.1/reference/onedim/governing-equations.html),
  energy/species equations: species enthalpy transport and chemical heat must
  appear in the conservation statement. No Cantera solver or reaction library ran.
- [IDAES 2.8 activity-equilibrium model](https://idaes-pse.readthedocs.io/en/2.8.0/reference_guides/model_libraries/generic/property_models/activity_coefficient.html),
  mixture enthalpy and NRTL/Wilson sections: nonideal activity needs independently
  supplied interaction parameters. This implementation supports explicitly
  justified ideal solution only; it does not substitute guessed coefficients.
- [NIST CODATA 2022 wall chart](https://www.physics.nist.gov/cuu/pdf/wall_2022.pdf),
  molar gas constant search excerpt, checked against the fetched
  [NIST constants table](https://physics.nist.gov/cuu/Constants/Table/allascii.txt)
  entries for R, N_A and k_B: R = N_A k_B. The handwritten decimal
  8.31446261815324 J/(mol K) is the product of the exact SI defining values,
  not an experimental regression or a droplet-specific constant.

## Closure and inputs

The liquid is spherical and internally well mixed. Component densities and heat
capacities are supplied constant-domain approximations; volume and sensible
enthalpy are additive. The gas film has supplied representative constant density,
thermal conductivity, viscosity, common heat capacity and a common mass
diffusivity for every fuel and oxidizer. Common gas heat capacity also defines
the vapor heat capacity and the simplified product/reactant sensible convention.
This is an explicit transport/thermochemistry approximation, not a detailed
multicomponent reacting-flow solution. No fuel-name lookup exists.

Every supplied material property and initial condition has raw value/unit, SI
value/unit, phase, source and locator, origin, reference temperature/pressure and
identity, temperature/pressure validity domain, and an uncertainty status. The
kernel accepts raw SI only and rejects mismatched raw/SI values. It does not
silently convert a declared g/mol value into kg/mol. Unit conversion, if needed,
must occur in a separately auditable upstream artifact. Properties are checked
at the initial/liquid/ambient temperatures and, for gas-film properties, the
reaction-sheet temperature. No property interpolation, lookup, fitting or
extrapolation outside its supplied domain occurs.

For each component the inputs include mass, elemental atom counts, molar mass,
liquid density/cp, latent heat and saturation pressure at a common reference
state. A burning calculation additionally needs vapor complete-combustion heat
to CO2/H2O(v)/N2/SO2 with the same declared thermochemical reference. The atom
inventory supports CHONS/He/Ar. The supplied molar mass must be externally
justified; this kernel does not independently reconstruct it from atomic masses.

Initial temperature, far-field temperature, pressure, oxygen mass fraction,
relative velocity and gravity are supplied. The surface nonfuel-background molar
mass is also explicitly supplied: it cannot generally be recovered from a
single ambient oxygen fraction, especially inside a burning film. The validity
of holding that background property fixed as liquid composition changes belongs
to the supplied closure. Ambient fuel is zero, the reservoir is infinite and
pressure is fixed. Chamber oxygen depletion and recirculation are not modeled.

## Independently stated algebra

For positive component masses m_i, molar masses W_i, and constant densities rho_i:

    V = sum_i m_i/rho_i
    r = (3 V/(4 pi))^(1/3); d_liquid = 2 r
    x_i = (m_i/W_i) / sum_j(m_j/W_j)
    C_liquid = sum_i m_i cp_liquid_i
    H_liquid = C_liquid (T_s - T_ref)

All component sensible enthalpies share T_ref and reference identity. A latent
heat at T_ref is advanced consistently with the declared constant heat capacities:

    delta_cp_i = cp_gas - cp_liquid_i
    L_i(T) = L_i_ref + delta_cp_i (T - T_ref)
    ln[p_sat_i(T)/p_sat_i_ref] = (W_i/R) *
        [(L_i_ref - delta_cp_i T_ref)(1/T_ref - 1/T)
         + delta_cp_i ln(T/T_ref)]

This is the integrated ideal-vapor/incompressible-liquid Clausius–Clapeyron
approximation with Kirchhoff heat-capacity correction, not an Antoine fit.
The pressure-volume term of the liquid, Kelvin curvature correction, critical
behavior, gas nonideality and Poynting correction are omitted. The supplied
property domains must justify these omissions. Negative latent heat, nonfinite
pressure or the mixture bubble boundary are rejected.

Explicit ideal-solution activity gives y_i,s = x_i p_sat_i/P. With supplied
background molar mass W_b, convert to interface mass fractions:

    W_interface = sum_i y_i,s W_i + (1-sum_i y_i,s) W_b
    Y_i,s = y_i,s W_i/W_interface; Y_F,s = sum_i Y_i,s
    f_i = Y_i,s/Y_F,s

The calculation requires 0 < sum y_i,s < 1. It does not clip supersaturation,
create a boiling regime, invent a nonvolatile residue law, or continue after a
component reaches numerical depletion. For a common diffusivity and zero fuel
at the outer boundary, the fuel flux shares equal f_i. Preferential evaporation
then follows from the actual mole fractions and vapor pressures:

    dm_i/dt = -f_i mdot

Pure evaporation uses O = 0 below. For the established reaction sheet:

    nu_O2,i = C_i + H_i/4 + S_i - O_i/2
    s = sum_i f_i nu_O2,i W_O2/W_i
    q = sum_i f_i q_i_vapor
    O = Y_O2,infinity / s

All liquid components in this burning closure must have positive oxygen demand
and a positive supplied vapor combustion enthalpy; inert liquid components and
oxidizer-bearing liquids require another closure. The complete-combustion
formula does not imply a finite kinetic rate, emissions yield, ignition, or
extinction threshold.

Species conservation integrated in spherical radius gives:

    b = ln[(1+O)/(1-Y_F,s)]
    mdot = 4 pi r rho_g D b
    r_sheet = r b/ln(1+O)       [burning only]

The energy equation is integrated separately inside and outside the sheet. Set
a = mdot cp_g/(4 pi r k_g) and a_f = a r/r_sheet (zero when not burning).
Then heat conducted into the liquid is:

    Q_in = mdot * [cp_g (T_infinity-T_s) + q expm1(a_f)]/expm1(a)

This expression remains valid for nonunity thermal/species Lewis number under
the declared common constant-property closures. Define
e_inner = T_s - Q_in/(mdot cp_g). The inner temperature solution evaluated at
the sheet is:

    T_sheet = e_inner + (T_s-e_inner) exp[a(1-r/r_sheet)]

The outer conserved energy has e_outer = e_inner + q/cp_g, so the independent
outer solution T_sheet = e_outer + (T_infinity-e_outer) exp(-a_f) must agree.
An invariant test checks this nonunity-Lewis continuity. With unity Lewis,
Q_in reduces to mdot [cp_g(T_infinity-T_s)+q O]/B, where B=expm1(b), which
recovers the standard steady droplet energy limit when storage vanishes.

The time integration evolves conservative sensible enthalpy directly:

    h_vapor_i(T_s) = L_i_ref + cp_g (T_s-T_ref)
    dH_liquid/dt = Q_in - sum_i mdot_i h_vapor_i

Differentiating H_liquid recovers C_liquid dT_s/dt = Q_in - sum mdot_i L_i(T_s).
This avoids losing the sensible enthalpy carried by departing mass. Every RK
stage updates cumulative evaporated mass, heat into liquid and vapor enthalpy
with the same flux quadrature; the final mass/energy residuals are audited.

Conditional fire-related outputs are HRR = mdot q, oxygen consumption = mdot s,
and their integrals. They assume complete combustion in an established ideal
sheet and an infinite ambient reservoir. They are neither ignition probability
nor a safety qualification. Radiation to a person/material, smoke toxicity,
chamber oxygen history and ignition distance are unsupported.

## Gravity, motion and observable separation

The entire gravity vector, relative-velocity vector and their shared frame remain
in the input. Output diagnostics are Re = rho_g |u|d/mu, Pe = |u|d/D and
G_nu = |g|d^3/(mu/rho_g)^2. G_nu is explicitly NOT Grashof: no temperature/density
contrast was fabricated. Directional convection, deformation, droplet trajectory
and settling are not solved by a spherical film.

Default applicability rejects every nonzero imposed gravity or relative motion.
`ExplicitSphericalExtrapolation` permits the limiting spherical calculation but
labels it extrapolated. Partial gravity therefore does not acquire an
unsupported empirical coefficient or a false claim of applicability. Zero
gravity is also not sufficient by itself to prove all other assumptions valid.

Liquid volume-equivalent diameter, reaction-sheet diameter and radiation-based
diameters have separate fields. Visible and UV diameters are absent because no
emission mechanism, spectral response, radiative transfer or image threshold
exists. Finite-rate extinction time is absent because no kinetic mechanism
exists. No optical equality, cool-flame clock or fixed extinction diameter is
introduced. Reaching a depleted inventory, a domain boundary, or a numerical
failure is an error; it is never called physical extinction.

## Numeric origin inventory

Every noninteger constant in the predictive kernel comes from the literal patch
or a provenance-bearing supplied property. The nine literal assignments are:

`Parameters::from_tensors(spec, stored_tensors)` binds a release adapter to the
actual stored values after exact forward-spec identity, inventory, shape,
finiteness, bounds and numerical-invariant checks. It never rereads the literal
ops. `Parameters::authored()` is the convenience path for independently
materializing the committed patch. The release adapter must also check its
construction manifest against the supplied ModelSpec.

| Tensor | Literal | Origin and reason |
|---|---:|---|
| pi | 3.141592653589793 | Handwritten mathematical circle constant; spherical geometry |
| molar_gas_constant | 8.31446261815324 | Handwritten exact-SI product N_A k_B; vapor-pressure law |
| step_seconds | 0.001 | Zero-shot numerical design, before checking any fixture |
| max_fraction_mass_loss | 0.02 | Zero-shot numerical stage positivity/resolution guard |
| max_fraction_temperature_change | 0.01 | Zero-shot numerical temperature resolution guard |
| relative_local_error | 0.0001 | Zero-shot step-doubling acceptance bound |
| absolute_mass_error_kg | 0.0000000000000001 | Zero-shot numerical absolute error floor |
| absolute_temperature_error_kelvin | 0.0001 | Zero-shot numerical absolute error floor |
| conservation_relative_error | 0.0000000001 | Zero-shot accumulation roundoff allowance |

Bounds in ModelSpec are non-singleton admissibility bounds, not a fitting
interval. All tensors are `author_writes_all` and all ops are literal `set`.
The existing tensor-role enum has no numerical-control value, so the relevant
descriptions explicitly identify those controls despite `physical_parameter`.
No search, scan, fit, random seed, optimizer or generated numerical assignment
was used. Material properties are not embedded in the patch because they belong
to the particular externally documented material/initial-state bundle.

Exact algebraic numerals in code: 0 is absence/initial cumulative quantity; 1 is
normalization/identity; 2 is radius-to-diameter, O2 atom balance and midpoint
half-step; 3 and 4 are sphere volume/area geometry, with 4 also hydrogen-to-O2
atom balance; integer powers 2 and 3 implement norms and geometric scaling.
Array lengths/indices and boolean flags are structural. None is a tunable
empirical multiplier. Tests contain additional explicitly synthetic numbers;
those are fixture definitions and tolerances, never production properties.

## Numerical method and checks

Fixed-step explicit midpoint uses the authored 0.001 s macro step (shortened
only at the requested endpoint). One full step is compared with two half steps;
the two-half result is kept. Each component-mass and temperature difference
must satisfy its authored absolute-plus-relative bound. There is no automatic
retry, time-step tuning or parameter adjustment. Stage loss and temperature
guards, positive inventory, finite arithmetic, property domains, bubble boundary
and final conservative budgets fail closed. A future differently resolved
release requires explicit different numerics, not silent forward behavior.

The error check is a local deterministic numerical estimate, not predictive
uncertainty or a proof of global stability. The fixed step may reject small or
rapidly heated droplets. No finite-time depletion event interpolation is
implemented. Output trajectories currently retain every macro step in memory;
long integration should be driven through a bounded job/streaming adapter before
use at service scale. No arbitrary physical time or component-count cap exists.

Tests exercise a true property-to-trajectory path on synthetic fixtures:

1. Valid ModelSpec, literal patch/materialization, complete tensor inventory,
   no parent and no outcome exposure.
2. A deliberately constructed pure-fluid thermal equilibrium recovers the d²
   limit and conservative mass/enthalpy budgets.
3. A binary ideal mixture preferentially evaporates, heats and conserves;
   permutation and species renaming do not change the physical result.
4. A burning mixture obeys complete-combustion HRR/O2 balances, has a distinct
   geometric sheet, and supplies no invented optical/extinction result.
5. Independent inner/outer energy solutions agree at a nonunity-Lewis sheet.
6. Gravity/motion default rejection, explicit extrapolation, and diagnostics.
7. Wrong units, raw/SI mismatch, missing source, invalid domain/reference,
   unsupported activity, boiling, nonfinite fractions and duplicate inventory
   fail closed.
8. Depletion guard and a separate half-step convergence comparison. That
   numerical check changes only a private test copy of the controls; it does
   not revise the committed one-shot literals.

Reproduce after coordinator module registration:

```powershell
$env:CARGO_TARGET_DIR = 'runs/tooling/astra-target'
cargo test -p ignis-wsp property_droplet_cleanroom
rustfmt --edition 2021 --check crates/ignis-wsp/src/property_droplet_cleanroom.rs
```

The author ran rustfmt and the final formatting check successfully. The
coordinator ran `cargo test --offline -p ignis-wsp --lib property_ --quiet` with
`CARGO_TARGET_DIR=D:\dev\SpaceApps-Ardealu\runs\tooling\astra-target`: all 46
property tests passed, including this module's eight. The coordinator verified
matching pre/post kernel SHA256
`d683f755441a5c730777254421731bfc9e34fe7a99f6bc5a992ba62ce9a69199`.
An independent thermal reviewer reconciled that same source hash and found no
demonstrated thermal double count or nonunity-Lewis structural error within the
declared assumptions. This is a narrow structural review, not proof of all
physics or a comparison to experimental outcomes.

Cargo execution used the existing coordinator target because the drive had
approximately 1 GB free; this author created no new Cargo build cache. Exact
frozen file hashes and the limited check evidence are in
`configs/models/droplets/property-w0/cleanroom-astra/freeze.json`.
Synthetic checks establish software/algebra invariants only, not NASA validity,
predictive quality, full chemistry, trained uncertainty or operational safety.

# Astra jets/coflows W0: a conditional physical forward

Author: `openai / gpt-6-astra`. Date: 2026-11-14. Numeric authoring: W0,
outcome-blind. This is an **experimental, unqualified conditional calculation**,
not a NASA release or a claim that a flame can ignite or remain stable.

Owned deliverables are `crates/ignis-wsp/src/property_jets_w0.rs` and
`configs/models/jets/property-w0/cleanroom-astra/`. The coordinator owns structural
integration, shared contracts and release assembly. No release was written here.

## Scientific scope

The main forward couples laminar axial momentum and conserved-scalar transport
for unequal positive jet/coflow velocities, with signed axial Boussinesq buoyancy.
It uses a finite circular inlet, an explicitly supplied outer coflow streamline,
composition-based stoichiometry and a complete-reaction caloric energy balance.
Normal and inverse configurations use the same signed oxygen-demand algebra.

The family implementation is not merely an equal-speed reference formula. That
formula is retained separately to check the transport implementation in its exact
limiting case. Gravity support is partial and unqualified: transverse gravity is
rejected, density variation is restricted by an explicit caller budget, and an
adverse axial force cannot be extrapolated through flow reversal.

No visible/UV diameter, lift-off, extinction, ignition, hazard probability or event
time is predicted. Stoichiometric radial brackets and the analytical axial tip
are local reaction-sheet diagnostics. They are not assigned an existing `Quantity`
whose physical meaning differs. The `ModelSpec` target is `temperature`, family
`other`; it does not impersonate a normal/inverse ACME release.

## Reading extent and exposure

Only the following scientific sources informed the derivation:

1. Moshe Matalon, CEFRC Summer 2013, [Lecture 10](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-10.pdf),
   PDF pages **10–13 and 25–29 only**. These supply reaction-sheet conditions,
   the assumptions behind the Burke–Schumann coflow problem, the mixture-fraction
   construction, and the distinction between scalar transport and finite-rate
   extinction. The finite-duct Bessel formula on p13 is not used as an unbounded
   jet formula. No Bessel roots or empirical values were copied into the patch.
   Direct web extraction failed on file size. With coordinator authorization,
   the PDF was downloaded solely into
   `runs/tooling/astra-jets-target/research/matalon-lecture10.pdf`; `pypdf` extracted
   exactly the page ranges above. SHA-256:
   `3f139912cd10d7671b701a8fe5e8c4513446f35c9c1c6c1473d5ae01118fa67a`.
2. Matalon, [Lecture 11](https://cefrc.princeton.edu/document/251): the web tool
   returned the **complete parsed text, six PDF pages**. Incidental exposure
   included the first-page Chung & Lee propane/air image caption and theoretical
   edge-flame contour captions/parameter examples on later pages. Those numerical
   examples were not used. This source motivates treating stabilization and
   lifting as separate hydrodynamic/finite-rate problems, not a scalar threshold.
3. [Cantera 3.1 governing equations](https://cantera.org/3.1/reference/onedim/governing-equations.html):
   the complete web text, lines 0–230, was returned. Relevant reading covered
   conservation equations, diffusive mass-flux correction, inlet conditions and
   charged-species/Poisson closure. Cantera's displayed stagnation-flow geometry
   is **not** the jet geometry solved here; its equations are a consistency check
   on the conserved quantities, not a transplanted jet solution.

A first search also returned the Lecture 10 p25 excerpt and generic theory
search snippets. Two later over-specific searches returned unrelated search
results; none informed the work. No NASA paper, per-test result, dataset payload,
development outcome, prior candidate, validation file, holdout or model artifact
was opened. The authorized electric-paper link was not needed or opened because
no charged-flow branch is claimed. `DataExposure.sources` records the three
generic theory sources; datasets, development feedback and prior candidates are
empty. Root AGENTS, Cargo manifests, contracts/wsp.rs, contracts.rs, assemble.rs
and unchanged physical_core.rs were the interface reads. Historical handoffs,
dossiers and earlier forwards/research were not read.

After the jets physics, literals, implementation and tests had been authored, a
`list_agents` coordination call unexpectedly returned completed **cross-family**
aerosol and solids author summaries: high-level implemented scope, synthetic test
counts, contract limitations and an aerosol source hash. No coefficients, source
files, release contents or experimental outcomes were exposed. This incidental
status-summary exposure did not inform any jets scientific choice. It is recorded
as `incidental-cross-family-author-status` in exposure sources; there is no released
candidate id to place in `prior_candidates`. The author makes no stronger claim
of isolation than this ledger supports.

## Symbolic derivation, before literal authoring

### Composition and reaction progress

For a neutral molecule with CHONS/He/Ar atom counts, signed complete-oxidation
oxygen demand is

\[
d_k=C_k+H_k/4+S_k-O_k/2.
\]

For a stream of mass fractions \(Y_k\) and molar masses \(M_k\),
\(d=\sum_kY_kd_k/M_k\) has units mol O2/kg. Let \(Z\) mean material originating
in the **inner jet**, whether that jet is fuel or oxidizer. With opposing demands
\(d_j,d_c\), stoichiometry is

\[
Z_s=-d_c/(d_j-d_c),\qquad 0<Z_s<1.
\]

Every component's molar mass is checked against the supplied atomic masses. The
supplied product composition must conserve all seven elemental inventories per
kg of stoichiometric mixture and comprise gas CO2/H2O/N2/SO2/He/Ar. Species names
do not select coefficients. Opposing reactive constituents already premixed within
one inlet are rejected: that case needs an inlet reaction-progress closure.

Complete reaction produces mass fraction

\[
f=\min\{Z/Z_s,(1-Z)/(1-Z_s)\}.
\]

On the \(Z\leq Z_s\) side the remaining coflow is \(1-f\), and on the other side
the remaining jet is \(1-f\). All three weights sum to one. Interchanging the two
streams and replacing \(Z\) by \(1-Z\) leaves the thermochemistry invariant.

### Caloric energy and buoyancy

The caller supplies the positive reaction heat \(Q_s\), **J per kg stoichiometric
mixture at a named common reference state**. It must identify the thermochemical
reaction/composition dependency. This value is not inferred from a fuel name or
from an observed temperature. It remains a supplier assertion with provenance
and uncertainty; no formation-enthalpy database is embedded here.

With sensible enthalpies zeroed at \(T_r\), inlet heat capacities \(c_j,c_c\),
product capacity \(c_p\), and remaining fractions \(j_l,c_l\):

\[
h_{in}=Zc_j(T_j-T_r)+(1-Z)c_c(T_c-T_r),
\]
\[
c_{out}=fc_p+j_lc_j+c_lc_c,\qquad
T=T_r+(h_{in}+fQ_s)/c_{out}.
\]

This is an actual constant-caloric-property energy balance. It assumes complete
oxidation, adiabatic transport and common scalar/total-enthalpy diffusivity;
it does not include dissociation, soot, radiation or finite-rate chemistry.
Formation-inclusive enthalpy references are rejected to prevent adding chemical
heat twice. Every resulting temperature and pressure must remain in every supplied
gas property domain. A mismatch is an error, not clipping or extrapolation.

The product/leftover mole sum gives
\(M^{-1}=f/M_p+j_l/M_j+c_l/M_c\). At the common pressure, the ideal-gas density
ratio is \(\rho/\rho_c=(M/M_c)(T_c/T)\); the gas constant cancels. A supplied
coflow reference density sets the dimensional mass/momentum scale. Its provenance
is preserved; this code does not derive an independent density determination.

The Boussinesq approximation uses reference density in transport/inertia and
the computed ratio **only** in the body force. Each evaluated ratio is checked
against the caller's explicitly supplied density-change budget. Allowing a budget
is not proof that it is suitably small: the declared laminar/parabolic/Boussinesq
assumptions remain unqualified and must be justified for a real application.

### Unequal-speed jet/coflow momentum and gravity

For axial speed \(u>0\), radial speed \(v\), constant \(\nu,D\), and the signed
projection \(g_z=\boldsymbol g\cdot\boldsymbol e_z\), the independent boundary-layer
derivation is

\[
u_z+(rv)_r/r=0,
\]
\[
uu_z+vu_r=\nu(ru_r)_r/r+g_z(\rho/\rho_c-1),
\]
\[
uZ_z+vZ_r=D(rZ_r)_r/r.
\]

The ambient hydrostatic pressure has been subtracted. An upward-oriented axis
has negative \(g_z\); hot/light fluid then receives positive axial acceleration.
The complete gravity vector, axis and frame are retained. Transverse gravity
cannot be represented by this axisymmetric pressure field and is rejected.

Introduce the volume streamfunction \(\psi=\int_0^r u(s,z)s\,ds\), in m3/s.
Continuity makes surfaces of constant \(\psi\) streamlines, and

\[
r^2=2\int_0^\psi d\psi'/u,
\]
\[
u_z|_\psi=\nu\partial_\psi(r^2u\partial_\psi u)
              +g_z(\rho/\rho_c-1)/u,
\]
\[
Z_z|_\psi=D\partial_\psi(r^2u\partial_\psi Z).
\]

These equations, rather than an empirical gravity multiplier, are marched by the
kernel. The model excludes axial diffusion, radial pressure coupling, turbulence,
swirl and reversal. The nozzle rim discontinuity is idealized, so the immediate
near-rim field has no accuracy claim. Streamline slope is reported as an additional
parabolic-domain diagnostic; a large value is not silently treated as evidence
that the boundary-layer assumption holds.

### Analytical reference

For constant equal speeds \(U\), zero gravity, and an **unbounded** coflow, the
scalar is the two-dimensional heat kernel convolved with a disk of radius \(a\),
with transport time \(z/U\). At the centreline:

\[
Z(0,z)=1-\exp[-a^2U/(4Dz)],\qquad
L_s=-a^2U/[4D\log(1-Z_s)].
\]

`expm1` and `ln_1p` preserve accuracy in the small-argument limits. This reference
has a finite inlet value and is not a singular point-source approximation. It
does not use the finite outer radius of the numerical model: that distinction is
explicit, and numerical comparisons require an outer-domain convergence check.
\(L_s\) is a stoichiometric-sheet endpoint, not flame lifetime, lift-off or an
optical flame length.

The numerical station also reports the radial part of scalar dissipation,
\(\chi_r=2D(\partial_r Z)^2\), in 1/s at the cell faces. This omits the axial
gradient contribution and is not compared with an invented extinction threshold.
A chemistry-specific finite-rate extinction law is still missing.

## Numerical method and limits

The radial streamfunction grid aligns exactly with the inlet interface. Within
each inlet stream, cells are uniform in squared radius. Cell widths in \(\psi\)
therefore include the actual corresponding inlet speed. At each marching stage,
the radial geometry is reconstructed from \(2\int d\psi/u\). Interior face speeds
use linear interpolation on the nonuniform \(\psi\) grid. The axis has zero
radial diffusive flux; the outer face has prescribed coflow velocity and \(Z=0\).
It is an outer reservoir boundary, not an impermeable tube wall.

The finite-volume diffusive flux is \(D(r^2u)_f\Delta Z/\Delta\psi\), and similarly
with \(\nu\) for momentum. SSPRK2 advances in **axial distance**, using explicit
diffusive/source positivity bounds at both stages. The caller supplies grid counts,
maximum step, CFL fraction, arithmetic tolerances and a finite work budget. If
a second-stage CFL bound is tighter, the distance step is reduced deterministically.
No parameters are inferred. Invalid states, nonpositive axial velocity, arithmetic
overflow/underflow or budget exhaustion fail closed. No unfinished result is
returned as a completed station.

The absolute arithmetic tolerance applies only to dimensionless fraction/unit-axis
checks. Dimensional atomic-mass, elemental-inventory, gravity-component and flux
balance comparisons use relative tolerance, avoiding reuse of a dimensionless
absolute allowance as though it had several incompatible physical units.

The code checks discrete balances:

\[
\dot m_{j,origin}=2\pi\rho_c\sum_iZ_i\Delta\psi_i,
\qquad \Pi=2\pi\rho_c\sum_i u_i\Delta\psi_i.
\]

Their changes must equal integrated outer diffusive flux and, for momentum,
the integrated buoyancy source. `jet_origin_mass_flow` denotes the conserved
origin tracer, not unburned fuel mass. Caloric energy residuals are exposed by
the thermochemical function. A cell-centre crossing of \(Z_s\) produces a radial
bracket, without root-fitting or invented optical equivalence. Empty brackets do
not prove that a sheet is absent; a near-axis sheet may be unresolved.

The embedded Euler/SSPRK2 temperature difference is an error **diagnostic**, not
a certified global bound. Spatial, finite-domain and physical-model errors are
not estimated by the current forward. Source uncertainty is retained but not
propagated. Mesh/step/domain convergence and real-regime validation remain necessary
for each scientific use. Synthetic convergence tests are not that validation.

## Numbers and literal inventory

| Item | Literal or source | Role and reason |
|---|---|---|
| `analytic_constants[0]` | `3.141592653589793` | Author-typed nearest f64 pi, circle mass/momentum flux geometry. |
| `analytic_constants[1]` | `2.0` | Exact circular streamfunction integral, midpoint geometry and SSPRK2 averaging. |
| `analytic_constants[2]` | `4.0` | Exact heat-kernel variance normalization in the analytic reference. |
| `0`, `1`, signs | Algebraic identities | Zero boundary/identity weights and conservation; not fitted numbers. |
| Element count `2` and oxygen demand divisors `2`, `4` | Exact atom balance | Complete CO2/H2O/N2/SO2 products and unchanged physical_core stoichiometry. |
| `1000` | Exact SI prefix conversion | mm to m, kPa to Pa; uncertainty bounds use the same conversion. |
| Tensor bounds `1`, `5` | Structural nonempty envelope | All literal tensor values lie inside; the forward also verifies their exact identities. Not a fit range. |
| `nu`, `D`, density, heat release, cp, molar/atomic masses | Mandatory provenance-bearing inputs | No material constants or fuel factors are hidden in code or patch. |
| Numerical budgets/resolutions/tolerances | Explicit call settings | Engineering controls, not scientific coefficients; no defaults. |
| Test properties and test acceptance tolerances | Explicit synthetic fixture literals | Invented for conservation, dimensional and algorithmic checks; never offered as NASA/material data. |

The patch is three manually typed `set` assignments. There is no generated tensor,
randomness, fitting, regression, weight search, optimizer or data-dependent literal
authoring. Test changes may fix implementation errors; no experimental outcome
feedback is used. The constants are analytical identities and cannot be revised
arbitrarily without changing the physical specification.

## Verification and delivery state

`rustfmt --edition 2021 crates/ignis-wsp/src/property_jets_w0.rs` was executed
successfully during authoring. The module contains six synthetic tests covering
stoichiometry, element/mass/energy conservation, inverse streams, the analytical
transport reference and mesh refinement, unequal-speed momentum transport,
finite outer-boundary scalar loss, gravity sign/rotation, unit/uncertainty conversion,
domain/reference/missing-input refusal, budget exhaustion, and spec/patch assembly.

Cargo execution was delegated back to the coordinator because free disk space
fell to approximately 1 GB and the coordinator explicitly stopped new private
target builds. No author-side compiler was launched. Actual coordinator test
results must be recorded separately and must not be inferred from test presence.
The planned command is `cargo test -p ignis-wsp property_jets_w0`, after the
coordinator registers the module in `lib.rs`.

`freeze.json` binds the final owned source, specification, literal ops, metadata,
this derivation and the permitted PDF hash. This freezes an outcome-blind candidate;
it is not release publication, NASA qualification or permission to reveal tests.

# Astra aerosol W0: private scientific boundary

Authored by `gpt-6-astra`, clean-room, 2026-11-14. This is an experimental,
not-trained, conditional physical kernel, not a registered model/release. No NASA
investigation outcomes, project datasets, prior candidates, development feedback,
holdout, evaluation results, or existing family forwards were read. No fitting,
parameter search, random initialization or runtime optimization is used.

## Symbolic derivation recorded before literal coefficients

Each section stores number concentration `N_i` [m^-3] and condensed particle
volume concentration `U_i` [m^3_particle/m^3_chamber]. Its representative compact
sphere has volume `v_i=U_i/N_i`, diameter `d_i=(6v_i/pi)^(1/3)` and mass
`m_i=rho_p v_i`. The total mass concentration is `rho_p sum(U_i)`;
`sum(N_i)` is number concentration. `sum(N_i v_i^2)` is a sectional approximation
to the second *volume* moment [m^3], not diameter variance. A section represents
one mean volume; within-section width is unresolved. Number-, mobility-,
aerodynamic-, optical- and volume-equivalent diameters are not interchangeable.

For drag, `Kn=2 lambda_g/d`, `Cc=1+Kn(a+b exp(-c/Kn))`,
`D=k_B T Cc/(3 pi mu d)`. The published slip coefficients are a conditional
approximation established for spherical PSL in air, not a universal smoke law.
They require supplied composition/phase/property-domain support and are never
selected by a material-name lookup. The alternative no-slip continuum law is
admissible only below an explicit conservative Kn domain threshold.

For neutral hard spheres with unit sticking probability, use the Fuchs transition
kernel from Roldin et al., Appendix C: `c_i=sqrt(8 k_B T/(pi m_i))`,
`l_i=8D_i/(pi c_i)`,
`delta_i=((d_i+l_i)^3-(d_i^2+l_i^2)^(3/2))/(3d_i l_i)-d_i`,
`K_ij=2pi(d_i+d_j)(D_i+D_j) / [(d_i+d_j)/(d_i+d_j+2sqrt(delta_i^2+delta_j^2))
+8(D_i+D_j)/(sqrt(c_i^2+c_j^2)(d_i+d_j))]`.
The cited equation's collision-efficiency ambiguity is avoided by supporting
only the explicit unit-sticking hard-sphere closure. Charge, van der Waals
enhancement, fractal aggregates, condensation and fragmentation are excluded.
With `x=l/d`, `s=sqrt(1+x^2)`, the algebraically identical stable expression is
`delta/d=x[1+2s+x(4-1/(s+1))]/[3(1+x+s)]`; this avoids subtracting nearby cubes.

Each unordered pair makes `J_ij=K_ij N_i N_j dt` collisions, halved for
self-collisions. Remove one particle and its representative volume from each
reactant (twice for self-collisions), add one particle of volume `v_i+v_j` to the
destination section. Thus every event removes exactly one particle and preserves
condensed volume and, for fixed composition/density, mass. Section boundaries
are supplied numerical controls. The final section is open, stores both moments,
and continues participating; it is never an inert overflow sink. Explicit
substeps satisfy a caller-supplied fractional-loss bound below unity.

A source supplies an independent particle number rate and particle volume per
particle. Its mass production follows `rho_p S_N v`, not material identity.
Unknown source yield is an error, not zero. A plug-flow transfer line gives a
delay `V_line/Q_line` and transmission `exp(-k_line delay)` with independent
per-source-size loss rates and an explicit flow branch fraction. These losses
are tracked separately. The initially empty line is an explicit supplied support
input, not an assumed inventory. Emitted, in-transit, line-deposited and delivered
number/volume budgets are separately integrated; loss acts during transit.
History intervals are absolute seconds since one explicit
time origin and must be contiguous; they are never differenced as successive
experiment conditions.

The well-mixed, constant-volume chamber has balanced total carrier inflow/outflow
`Q`, including the source-line branch. That branch cannot exceed total throughflow;
the remaining inflow is particle-free. A disconnected line has explicit zero
branch fraction. Solid versus liquid particle phase is supplied and checked;
compact coalescence/sintering remains a separate required support assumption.
Dilution removes all sectional moments at rate `Q/V`. For a perfectly absorbing
wall with an independently specified stagnant film thickness `h`, Brownian
deposition speed is `D/h`. Gravitational drift is
`u_g=(rho_p-rho_g) d^2 Cc g_vector/(18mu)`; each face removes
`A/V max(u_g dot outward_normal,0)`. Geometry and frame are explicit.
Finite-Re drag, buoyancy-driven chamber mixing, thermophoresis, electric fields,
resuspension and inertial sampling losses are not modeled. Partial gravity is
always marked unqualified: inserting a gravity vector does not qualify mixing,
source production or detector transport there.

Optical observation is separate. For nonabsorbing compact spheres with supplied
real relative refractive index `m` and wavelength in the medium `lambda`,
`x=pi d/lambda`, `C_sca=(8pi/3)r^2 x^4[(m^2-1)/(m^2+2)]^2`.
`beta_sca=sum(N_i C_sca_i)` [m^-1]. A narrow collimated extinction detector
with no accepted scattered light has `P=P0 exp(-beta_sca L)` and
`voltage=dark_voltage+responsivity_V_per_W P`. An explicit Rayleigh-domain
check is applied. Absorbing soot, Mie-sized particles, angular scattering
photometers, broadband instruments, ionization detectors and detector events
are not predicted. No concentration is obtained by inverting the same detector
used as validation truth. Detector/source transitive evidence dependencies must
be complete and disjoint. Output has unquantified model uncertainty, never a
calibrated interval.

## Sources and actual read extent

* NIST CODATA Boltzmann constant, whole short value page:
  <https://physics.nist.gov/cgi-bin/cuu/Value?k%7Csearch_for=Boltzmann>.
* Kim et al. (2005), J. Res. NIST 110, 31-54,
  <https://nvlpubs.nist.gov/nistpubs/jres/110/1/j110-1kim.pdf>:
  abstract and introductory equations (1)-(3), printed pp.31-32; browser extraction
  also exposed opening theory text on p.33. No tables/fit execution/results section
  used. The abstract reports generic instrument determinations; its published
  slip coefficients are transcribed, not refitted or optimized here.
* Roldin et al. (2011), ADCHEM methods,
  <https://acp.copernicus.org/articles/11/5867/2011/>: abstract; and
  <https://acp.copernicus.org/articles/11/5867/2011/acp-11-5867-2011.pdf>:
  Appendix C equations C1-C2, printed pp.5889-5890, screenshot request p.5889;
  neighboring Appendix B/C3-C8/D1-D2 and opening pp.5867-5869 were returned by
  browser extraction. The abstract contains generic atmospheric-model outcomes;
  none supplies coefficients or qualification here. The separate 2010 preprint
  Appendix C equation snippet was encountered in search. Only the published
  Brownian equations are used; this implementation derives its own conservative
  moment bookkeeping and does not copy the atmospheric model configuration.
* NIST SCATMECH `RayleighScatterer`, whole short documentation and formula source:
  <https://pages.nist.gov/SCATMECH/docs/rayscat.htm> and
  <https://pages.nist.gov/SCATMECH/code/rayscat.cpp>. Only the nonabsorbing-sphere
  scattering expression is used; no software default properties are adopted.
* Norris (1999), abstract and first-page population-balance equation search
  excerpt, printed p.78, <https://doi.org/10.1214/aoap/1029962598>.
  Direct publisher PDF opening did not provide usable text beyond an iframe.

Search exposure: narrowly targeted generic theory searches also returned
non-NASA literature snippets (coagulation, size metrology, optics) and unrelated
NASA-hosted Rayleigh/radar-theory snippets. No NASA combustion investigation,
dataset or payload was opened. These incidental theory snippets did not supply
numbers. Publisher methods-page opening returned HTTP 403 for
<https://doi.org/10.1080/02786820701697804>; not used. Full bibliography/corpus
downloads and local outcome-bearing documents were not accessed.

## Contract gap and release boundary

`Quantity` currently has no aerosol number concentration [m^-3], particle mass
concentration [kg/m^3], condensed-volume concentration [1], particle-volume
moment with declared order, aerosol size definition [m], extinction/scattering
coefficient [m^-1], transmitted optical power [W], or detector voltage [V].
`MoleFraction`, `LiquidDiameter` and flame quantities are scientifically wrong
substitutes. `PhaseEndTime` also does not define a detector threshold event.
No ModelSpec, patch claiming a spec hash, or registered release is created.
The literal assignments and author envelope are preparatory private artifacts;
they are not an assembled `HandwrittenWeightPatch` until a correct shared
contract and producer/consumer fixtures exist. The private structs in the Rust
file are not wire contracts and are not exposed through API/MCP/TS.

Required shared work: typed aerosol moments and size-basis tags; detector signal,
spectral/angular response and event semantics; explicit time-origin/history and
source-independence lineage; property domain and uncertainty; source yield
missingness; per-bin numerical approximation state; a model-output provenance
envelope and gravity qualification state. Shared contract changes are left to
the coordinator/integrator.

## Literal ledger and numerical controls

The literal ledger is recorded separately beside `constants.ops`; exact sphere
geometry, binary-collision counting, pi and SI relations are algebraic constants.
Time step, fractional loss limit, section boundaries, arithmetic tolerance and
maximum substeps are caller-specified numerical/resource controls, not tuned
science parameters. No default property values exist. Synthetic tests check
mechanical invariants only and do not qualify smoke prediction.

## Verification and reproduction

Run only this private module (no NASA data or other family forward is needed):

```powershell
rustfmt --edition 2021 --check crates/ignis-wsp/src/property_aerosol_w0.rs
New-Item -ItemType Directory -Force runs/tooling/astra-aerosol-target | Out-Null
rustc --edition 2021 --test crates/ignis-wsp/src/property_aerosol_w0.rs -o runs/tooling/astra-aerosol-target/aerosol-invariants.exe
& runs/tooling/astra-aerosol-target/aerosol-invariants.exe
```

The eleven offline synthetic checks cover closed/empty/dilution limits; the
number/volume/mass distinction and conservation under coagulation; the Brownian
continuum limit and pair symmetry; delayed-source/line inventory conservation;
source-dilution and coagulation time-step convergence; gravity orientation and
explicit partial-gravity unqualification; optical moment versus mass/voltage;
unit, domain, provenance independence, missing yield and resource failures; and
strict literal syntax. A resolved first-coagulation birth recovers the independent
second-moment identity while a single broad section loses that moment. This is a
local section-resolution check, not a proof of global size-distribution convergence.

`literal-ops.json` is the same manually authored scalar inventory in the shared
`LiteralOp` shape, without a counterfeit ModelSpec/patch wrapper. `constants.ops`
is the strict standalone loader input. The author envelope has system origin and
no project/NASA-outcome exposure. Freeze hashes cover the kernel, both literal
representations, envelope, ledger and this note. No scientific coefficient was
revised from test outcomes; tests exposed only implementation/invariant behavior.

The primary limitations remain substantive: compact neutral coalescing spheres
with unit sticking; an independently supported drag law and mixing/film model;
no source chemistry/yield prediction, irregular soot morphology, thermophoresis,
charge, wall resuspension or coagulation in the transfer line; no absorbing-soot
or Mie optical response; no external detector threshold event; unquantified
model uncertainty; and no gravity/source/domain qualification from fixtures.

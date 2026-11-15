# Independent Astra transport review

Status: completed; outcome-blind physical review of the three new property transport prototypes, not model qualification. Four actionable admission/provenance findings, plus a stated deposition-closure limitation.

## Independent derivation recorded before implementation inspection

For a fixed spherical shell [r_w,R], outward injected mass rate m_dot > 0 implies rho u r^2 = m_dot/(4 pi). For species k the outward total mass flux is m_dot Y_k - 4 pi r^2 rho D_k dY_k/dr. Its wall value must equal m_dot Y_k,in when the injected composition is prescribed as a net injection: prescribing Y_wall alone instead mixes a reservoir Dirichlet condition with independent diffusion. The integrated shell balance is d integral(rho Y_k dV)/dt = F_k(wall)-F_k(outer)+integral(omega_k dV). A conservative finite-volume update therefore uses face flux differences and shell volumes 4 pi(r_+^3-r_-^3)/3, not area-free Cartesian differences. Net sensible-enthalpy injection is m_dot h_in; diffusive/conductive outward energy flux is -4 pi r^2 k dT/dr. With thermal sensible enthalpy h(T;T_ref), advective flux must use that same reference at both boundaries. If chemistry heat release is external to sensible h, volumetric chemical power is added exactly once. Fixed density, constant heat capacity are declared approximations; energy storage and source units must still match watts and joules. Radiative loss has nonnegative outward sign above the radiative bath, and can be heat gain below it if modeled as exchange.

For an axisymmetric incompressible jet define u_z=(1/r) dpsi/dr, u_r=-(1/r) dpsi/dz; this enforces div(u)=0 continuously. Curl convention omega_theta=du_r/dz-du_z/dr gives (d_zz+d_rr-(1/r)d_r)psi=-r omega_theta. Vorticity equation is d_t omega + u_r d_r omega + u_z d_z omega - (u_r/r)omega = nu(d_rr+(1/r)d_r+d_zz-1/r^2)omega + curl(f)_theta. If the z axis points upward and f_z=+g beta(T-T_ref), curl(f)_theta=-g beta d_r T: a hot center with negative radial gradient generates positive omega_theta. A supplied downward gravity vector changes that convention through f=-beta(T-T_ref)g_vector. Conservative scalar transport is d_t q + (1/r)d_r(r u_r q)+d_z(u_z q)=D[(1/r)d_r(r d_r q)+d_zz q]+S. A streamfunction alone does not make an arbitrary separate discrete derivative divergence-free; face velocities and cell volume fluxes must be consistent. At r=0 regularity requires u_r=0, d_r u_z=d_r q=0, omega_theta=0 and psi=O(r^2). Explicit stability must include combined axial and radial advection, cylindrical diffusion, and reaction/forcing limits. A prescribed velocity flow without momentum evolution cannot claim buoyancy-feedback or momentum-coupled coverage.

For aerosol number density N [m^-3] and dispersed material volume fraction V [m^3/m^3], particle transport uses the same particle velocity u_p=u_g+v_settle and conservative divergence for both moments. Monodisperse binary coagulation with kernel K [m^3/s] gives dN/dt=-K N^2/2 if K counts unordered collision pairs; dV/dt=0. Radius a=(3 V/(4 pi N))^(1/3), and particle mass per volume is rho_p V. A kernel convention absorbing 1/2 must be explicit. Volume destruction from a coagulation event is unphysical. Settling in a uniform gas without boundary losses translates both moments without changing total particle material; losses at absorbing boundaries must be accounted by outward flux. Dilute Stokes settling is v_s=(rho_p-rho_g)d_p^2 g/(18 mu), with sign set by the gravity vector and positive slip correction if used. Brownian equal-sphere continuum kernel is K=8 k_B T/(3 mu), before transition/slip corrections. Turbulent, inertial or Knudsen-range coverage needs additional assumptions; positive inputs do not establish those approximations. A fixed mean-radius coagulation closure is not a resolved size distribution. Positivity clipping must not silently create or destroy integral material. Temperature mixing or evaporation requires explicit gas/particle heat and material balances and causal dependence on current state.

## Review conclusion and reproducible findings

Review completed 2026-11-14 by an independent GPT-6 Astra physical reviewer, not the numerical author. Four actionable admission/provenance findings remain in the reviewed originals. No spherical flux/energy sign defect or jet scalar/momentum transformation defect was identified. This limited conclusion is not accuracy qualification. Original kernels/literals/specifications were not edited; corrections belong in separately identified follow-on artifacts.

### F1 — Aerosol admits a physically impossible initial material volume (P2)

At `property_aerosol_w0.rs:859-878`, initial moments are accumulated without checking that total condensed volume occupies less than the chamber. The only occupancy check is at lines 1006-1013, after coagulation, dilution/deposition and source addition. A state with `sum(U)=2` is physically impossible before even considering the stronger dilute-particle restriction, but one step with dilution rate 2/s and duration 1 s returns `sum(U)=2*exp(-2)=0.2706705664732254` successfully. The final budget balances exactly enough; it cannot establish validity of the initial condition.

Independent counterexample: `runs/tooling/astra-transport-review/aerosol_independent.rs`, marker `INVALID_INITIAL_OCCUPANCY`. Particle diameter is 2 micrometres, coagulation is disabled, no source or deposition exists, and the balanced chamber throughflow is 2 m3/s through a 1 m3 chamber. All are explicitly synthetic inputs. Check the aggregate initial material occupancy before the first evolution operator. The explicit dilute-support assertion must not override an impossible geometric occupancy.

### F2 — Aerosol diameter domains are bypassed for closure support and volume samples (P2)

The compact/coalescing-sphere support record is read only with `diameter=None` in `AerosolProperties::check` (lines 351-357); `particle()` rechecks the numerical particle properties at actual diameter but never the support record. A support domain ending at 1e-9 m accepts a 2e-6 m particle. Initial and source `particle_volume` records likewise receive only `None` at lines 832-837 and 860-865, despite their domain carrying a diameter interval. A 2e-6 m particle volume with its own declared diameter maximum 1e-9 m is accepted.

The independent probe isolates both cases, returning N=1e8/m3 unchanged. Markers are `SUPPORT_DIAMETER_BYPASS` and `PARTICLE_VOLUME_DIAMETER_BYPASS`. Material assumptions need the same actual-diameter admission as density, viscosity, mean free path, film thickness and optical properties; otherwise the domain metadata overstates the enforced scientific boundary. This is separate from whether the supplied support assertion is credible.

### F3 — Aerosol observation cannot recheck the prediction's source independence (P2)

`Prediction` (lines 684-696, constructed at 1074-1086) retains only condition/time strings, output moments, particle density and budgets; it does not retain the initial/source/property evidence closure or the admitted context. `observe` (lines 1146-1150) checks only matching condition/time strings before checking detector inputs against the caller's current truth-dependency set. The evidence of the prediction's initial/source inputs is unavailable to that check.

Fresh counterexample: propagate with initial-number source identity `truth-reused-as-initial`, then observe with the same condition/time identities and a detector-truth dependency set containing that exact source. Observation succeeds and returns beta=7.6734412542424942e-14 /m. Marker: `OBSERVATION_LINEAGE_BYPASS`. This uses no real detector trace: it is an identity-only synthetic test. The initial propagation correctly enforces its own supplied truth set, but the two-stage API does not preserve the information needed to enforce the same claim at observation. Bind complete predictive dependencies and physical context to the prediction, or refuse changed incompatible context; host-derived evaluation authorization remains a separate requirement.

### F4 — Jet sensible enthalpy integrates outside the supplied heat-capacity domain (P2)

`Prepared::new` checks component properties at inlet mixture states; products are built at the common reference state. `caloric` checks reconstructed states, but neither requires the jet/coflow heat-capacity domains to contain the reference state or the complete integration path. The formula near lines 563-566 computes `cp_j*(T_j-T_ref)` and `cp_c*(T_c-T_ref)` regardless.

Independent public-API counterexample: common sensible reference T_ref=300 K, fuel heat-capacity domain [310,2000] K, fuel inlet T=320 K, all reconstructed endpoint temperatures inside their admitted domains. `caloric_state(...,Z=1,...)` succeeds with 320 K despite using cp over [300,310] K outside its domain. Marker: `JET_ENTHALPY_REFERENCE_DOMAIN_BYPASS` in `physics_independent.rs`. The intentionally loose synthetic Boussinesq budget is irrelevant to this domain failure and establishes no physical regime. Require each integrated constant-cp property domain to cover its entire reference-to-state path. The spherical implementation already performs this kind of check in `cp_mass`.

## Completed independent physical checks

### Spherical injected scalar and total enthalpy

Writing `Gamma=rho D`, the steady injected-origin scalar solves

```
F = m w - 4 pi r^2 Gamma w_r = m,
a = m/(4 pi Gamma),
w = 1 - exp(-a/r),
w_r = -a exp(-a/r)/r^2.
```

The integration constant follows from a prescribed injected *net flux* and w(infinity)=0. In normal injection Z=w; in inverse injection Z=1-w. Therefore the total fuel-origin flux is m and 0, respectively, not m in both modes. The implementation's internal diagnostic is intentionally the injected-origin flux, so its common subtraction of m is correct. Sheet radius uses Z=Zst, with the inverse transformation accounted for.

With formation-inclusive h and common scalar/enthalpy diffusivity, the conserved enthalpy is `h=Z*h_f,in+(1-Z)*h_o,in`. Piecewise product/excess-stream mass fractions sum to one, and `T=T_ref+(h-sum(Y h_form))/sum(Y cp)`. No additional chemical source belongs in this conserved h. The transient shared face fluxes telescope; exact spherical cell volumes and the prescribed inner m*h_in flux make their reported global budgets dimensionally correct. The outer boundary combines ambient diffusion and last-cell outward advection, consistently with its declared reservoir boundary. Radiation and wall terms have correct units and signs, including bath/wall heating when colder gas receives heat.

A fresh synthetic H2/O2/H2O accounting case uses inlet T=320 K, cp=1000 J/(kg K), product formation enthalpy -800000 J/kg and zero reactant formation enthalpy. Independent energy accounting predicts T_sheet=1120 K, recovered for both directions. At r=0.01,0.02,0.1,1,1000 m the independently reconstructed fuel-origin flux residual is at most 6.7762635780344027e-21 kg/s. These are algebraic scenarios, not molecular-property determinations or real flames.

The explicit transport coefficient bound is correct for the scalar finite-volume stencil. The radiative derivative bound is a frozen-composition monotonicity bound; it does not prove a coupled reactive invariant region, extinction, global error control or mesh convergence. The code reconstructs/rejects invalid new thermochemical states rather than clipping them. No separate transient manufactured-solution or spatial-convergence study was run in this review.

### Jet streamfunction transformation, sign and discrete geometry

The implemented jet is a steady parabolic momentum calculation, not the full transient vorticity system derived initially above. Let x denote axial distance to avoid confusing it with mixture fraction Z. With `psi_r=r u`, `psi_x=-r v`, differentiation along fixed psi gives `u d_x|psi = u d_x|r + v d_r`. Thus

```
d_x u|psi = nu d_psi(r^2 u d_psi u) + g_parallel (rho/rho_coflow-1)/u,
d_x Z|psi = D d_psi(r^2 u d_psi Z),
r^2 = 2 integral_0^psi dpsi'/u.
```

The source has units 1/s in the u-versus-x equation. For upward positive x and downward gravity, a light fluid has two negative factors and accelerates positively. The code uses that sign. Momentum flux is `2 pi rho_ref integral u dpsi` [N]; origin-tracer mass flux is `2 pi rho_ref integral Z dpsi` [kg/s]. Shared face fluxes and stage-consistent buoyancy accumulation telescope into those budgets. Constant-psi faces make the radial motion compatible with continuity; this avoids the unrelated-velocity-stencil issue in a naive collocated Eulerian scheme.

For constant u=1 m/s and `Z=1-psi/psi_B = 1-r^2/R_B^2`, independent cylindrical differentiation gives `d_x Z=-2D/psi_B`, including the regular axis. A deliberately nonuniform streamfunction grid with 7 inner and 13 outer cells gives maximum RHS error 1.9810542095655137e-15 against -0.03000000000000001 /m. This checks the factor of two, geometric flux, axis and finite outer boundary together. It accesses the unchanged numerical operator through a local source snapshot plus a review-only function; it does not change the original module or authored literals.

The latest scalar-dissipation output uses `d_r Z=r*u*d_psi Z` and `chi_r=2D(d_r Z)^2`, with units 1/s. It is radial-only and cannot by itself predict extinction. The latest dimensional comparisons use relative tolerance; the previous report/source mismatch described below is resolved. SSPRK2's positive conductances and per-stage bounds support nonnegative transport/positive velocity within the declared regime. No proof of arbitrary nonuniform-grid convergence, pressure closure, large-density-change Boussinesq validity, or complete reversal handling is supplied or inferred.

### Aerosol coagulation, delay and wall-flux limits

For each unordered pair `(i,j)`, the code's event count is `K_ij N_i N_j dt`, with self-pairs halved. It removes the correct two reactants and adds one product of volume `v_i+v_j`. Consequently `delta(sum N)=-events` and `delta(sum U)=0`. Uniform proportional removal of both sectional moments preserves representative volume during linear loss. With positive source delay, the exact interval-overlap integration cannot inject a cohort before its arrival time.

A fresh closed-chamber coagulation probe with an open single section and N_initial=1e12/m3 gives material-volume absolute error 9.3173624197973037e-21 and number-plus-event absolute error 0.0010035037994384766 /m3, at a number scale 1e12/m3. A separate delay probe with source rate 1e5/s, 1 s line residence time and observation at 0.5 s gives emitted=50000, in-line=50000, admitted=0. These exercise the implemented public propagation API without running the author's tests.

The Stokes settling sign `(rho_p-rho_g)*g_vector` is correct, including buoyant particles. Absorbing-face outward projection is nonnegative and is applied consistently to number and material volume. The Fuchs self-pair counting is consistent with the standard continuum equal-sphere limit K=8 k_B T/(3 mu); using the same kernel with the explicit factor 1/2 is appropriate. No externally sourced coefficient or alternative model value was introduced by this review.

An additional **model limitation**, not a claimed conservation defect: adding `D/h + max(w,0)` for one absorbing face is an additive deposition approximation. If the intended interpretation is one steady film subject to both drift and diffusion, solving `J=w n-D n_x`, with `n(0)=N` and `n(h)=0`, gives

```
J/N = w/[1-exp(-w h/D)],
lim_(w->0) J/N = D/h.
```

At Peclet number w*h/D=-1 the code gives D/h whereas the coupled film gives 0.5819767069 D/h; at +1 the code gives 2D/h versus 1.5819767069 D/h. Gravity opposing deposition suppresses diffusive transport through the film. The report should retain this additive-closure limitation instead of implying a solved coupled drift-diffusion layer. These comparisons derive directly from an elementary boundary-value problem; they do not select a new model or calibrate a coefficient.

## Exposure, source drift and exact identities

No goals, dossiers, development feedback, old candidates/forwards, NASA reports/payloads/outcomes, runs/dev, holdout, model artifacts, validation or test partitions were read. No memory files or external web sources were used. The independent derivation at the beginning was written to this report before implementation inspection. Primary-physics links cited by the three author reports were read as link text only; their destinations were not opened.

Repository reads were root AGENTS.md in full; the three new source files' production paths; the three new author reports; physical_core.rs lines 1-737 plus declaration-name searches; the tiny spherical clean-room check.rs; and aerosol constants.ops through its compiler include. Initial commands also displayed portions of the three new files' inline synthetic tests (including aerosol fixture constructors); the independent probe constructors and assertions were written afresh and no author test was executed. File-name discovery found but did not read older physical review or droplet files. No historical handoff or common-contract file was needed.

Jets changed while this review ran. Earlier displayed code lacked chi output and separate dimensional tolerances while its report already described them. A later source snapshot and current reread contain both changes. The initial hash collection occurred after the earlier code display, so the hash must not be attached retroactively to those older displayed lines. This report's jet findings/checks apply to the following current source hash, independently reread/rehashed at the end. The report hash also changed from an earlier 2cbc... snapshot to 24e3...; no obsolete mismatch is retained as a current defect. The coordinator has identified the spherical and aerosol originals as frozen, with fixes to use separately named artifacts.

SHA-256 (repository-relative paths):

| Path | Hash |
|---|---|
| AGENTS.md | a0c2851531cc2da8207aff8245fad75bd6e80b75dc76ff5013b871951056c9ce |
| crates/ignis-wsp/src/property_spherical_w0.rs | 398c6fc0c9f11607cf840a9d11539396b06d905e3049c41060e2f095c5a54479 |
| crates/ignis-wsp/src/property_jets_w0.rs | 1fbde4e90c869dfe5b85278a2fd4254cfcd91c8844db0075fbe04f82ab8ba81b |
| crates/ignis-wsp/src/property_aerosol_w0.rs | aaaf4a97cb92a4d278f77246391768bbaaa3afa638e23e6711ccdd9108a7f0b5 |
| crates/ignis-wsp/src/physical_core.rs | c6d4ead14d4a85ccc20113fe94fc61529d5cacf7aad73e97b7569d48e0946644 |
| docs/wsp/research/astra_spherical_w0.md | 9524ddd898b415b689211b73a5f1398141e78b0dcca063825b4de336334949b8 |
| docs/wsp/research/astra_jets_w0.md | 24e3d6ad92d8d2cfe6d29a40e8a406b185890274e4917b6465ebff1985d14887 |
| docs/wsp/research/astra_aerosol_w0.md | b83232471184e42443fcdb5494c31d66473ef2cbaa04bc2db02a524e6414789b |
| configs/models/spherical/property-w0/cleanroom-astra/check.rs | 56ad8306110d96bc0a146354e2fc7044c645c95326fa188e10b5b8e874eeb17c |
| configs/models/aerosol/property-w0/cleanroom-astra/constants.ops | 6255cdf9c3e5a39da9db3e96a37ebd74a69f5b70c5f34b53f8dd20f9974b00dc |
| runs/tooling/astra-transport-review/aerosol_independent.rs | 815c78cfb0168b394f8faf32abf4c810102247ec8fed5cc70d6bc1b382b2d04a |
| runs/tooling/astra-transport-review/physics_independent.rs | 7d4dfc3762cb6660b371fe45f35d9e5cdd5c8f68a73dfe87416019ab91f2c978 |
| runs/tooling/astra-transport-review/jets_snapshot.rs | 8910ca1670bd2fc8c9e2219a09df7f0176baf810099f3a6e4a4d99d8c4c5d490 |

The jet snapshot contains the reviewed source plus a small review-only operator function, so its hash deliberately differs from the original module. No literals were changed. The standalone scripts contain absolute module paths to the authorized checkout; adjust only those paths when reproducing elsewhere.

## Commands actually executed and scope limits

From `.`:

```powershell
rustc --edition 2021 runs/tooling/astra-transport-review/aerosol_independent.rs -o runs/tooling/astra-transport-review/aerosol_independent.exe
& runs/tooling/astra-transport-review/aerosol_independent.exe
rustc --edition 2021 runs/tooling/astra-transport-review/physics_independent.rs -o runs/tooling/astra-transport-review/physics_independent.exe
& runs/tooling/astra-transport-review/physics_independent.exe
```

Both final compilations and executions exited 0. Their successful execution **confirms the negative counterexamples are currently admitted**, rather than declaring those cases scientifically valid. An initial physics-probe compile failed because jets changed the private Prepared::new signature from NumericalTolerance to MarchOptions while the review was in progress; updating the local review-only invocation fixed it. The original modules were unchanged by this reviewer.

No Cargo command, dependency download, runtime model evaluation, scientific parameter search, fitted coefficient generation, CI action, release operation or commit occurred. Only this report and the small authorized runs/tooling/astra-transport-review files were written. The six-family prototypes remain zero-shot numeric artifacts, not qualified releases. This review covers three transport kernels; it does not certify the other families, statistical error, detector validity, real-property support, NASA agreement, release construction or user-facing exposure controls.



## Targeted v2 re-review — first snapshot, 2026-11-14

The coordinator explicitly authorized the NEW `property_aerosol_v2.rs`, `astra_aerosol_v2.md`, `property_jets_v2.rs`, `astra_jets_v2.md` and their necessary packet dependencies. The original exposure exclusions remain unchanged. Only the new diff/production changes, jet wrapper, new reports, prior independent harnesses and aerosol literal include were read; no external sources or outcome-bearing files were accessed. These revisions are synthetic-review-informed implementations, not fresh zero-shot artifacts.

### Original counterexamples and valid predecessor behavior

The exact original inputs were rerun against actual v2 modules through standalone harnesses, with assertions changed from expected acceptance to expected rejection:

- Initial aerosol occupancy U=2 is rejected before dilution.
- Compact/coalescing support with maximum diameter 1e-9 m rejects actual 2e-6 m particles.
- The initial particle-volume record's own diameter interval is enforced. An additional source-volume check confirms the same gate for emission cohorts, even before their delayed arrival.
- Selecting the original initial-number evidence source as detector truth at observation now rejects the prediction. Inspection confirms the complete supplied input-evidence inventory is retained and rechecked in both `observe` and `moments`, with detector evidence appended to the observation result. Declared transitive dependencies still rely on truthful, complete caller inputs; the code cannot discover omitted lineage.
- Jet F4 is rejected by **all three** v2 public functions: `caloric_state`, `forward`, `analytic_centerline_reference`. The original public API still admits the original 300 K reference / 310 K lower cp domain / 320 K inlet case, verifying that the counterexample remains discriminating.

Jet v2 checks every component's cp and molar-mass domain at the common reference and at reference temperature with operating pressure. W0 checks endpoints, common reference and operating-pressure compatibility; rectangular closed domains make this sufficient for the complete path. No new numerical law is introduced. Fresh valid equal-speed, unequal-speed and signed axial-gravity scenarios give exactly identical complete `JetForecast` debug records, and five caloric fractions per scenario and the equal-speed analytical reference also match exactly. Reversing the intentionally light H2/O2 fixture exceeds the predecessor Boussinesq contrast domain; both versions reject identically. This was an initial probe assertion failure, corrected to document the actual limitation rather than claim a valid inverse case.

Aerosol positive coagulation and causal-delay outputs are unchanged from the earlier independent probe. A stronger direct comparison compiles the same fresh fixture twice, selecting the original or v2 module. The scenario combines three sections, coagulation, nonzero signed gravity, an explicitly selected predecessor additive film, dilution, delayed emission and transfer-line losses. **Every section, the entire Budget, Moments and the 100-step count match exactly** between original and v2. Final number is 818837111.0725634 /m3 and condensed volume is 3.4303844837039237e-9. This equality applies to the explicit predecessor branch and admitted inputs; the new exact-film branch intentionally differs.

### Independent new-film verification

For the new named `SteadyDriftDiffusionAbsorbingFilm` branch, the source implements

```
base = D/h
Pe = w/base
w == 0: speed = base
w > 0: speed = w / [-expm1(-Pe)]
w < 0: speed = (-w)*exp(Pe) / [-expm1(Pe)]
```

The negative-drift expression is algebraically equivalent to the derived BVP and avoids exp(-Pe) overflow. The exact zero branch and expm1 denominators preserve the limit; numerical underflow is explicitly rejected, not silently clipped. The branch requires independently supplied size/condition applicability and records its selected name in the output. It remains a steady planar constant-property film, not a chamber mixing or turbulent deposition solution.

The independent test reconstructs Brownian D and Stokes mobility from the supplied synthetic scenario and the unchanged literal include, chooses signed gravity for specified dimensionless Pe, and infers the implemented transfer rate from the public propagation's deposited-number budget using `-log1p(-removed/N)`. It does not call the private film function or copy its sign-dependent expression as the oracle. The separate BVP expression `base*Pe/[1-exp(-Pe)]` is used over the following numerically moderate verification range:

| Pe | Observed speed / (D/h) |
|---:|---:|
| -100 | 3.7200759760207834e-42 |
| -1 | 0.58197670686932623 |
| -1e-9 | 0.99999999950000007 |
| 0 | 1.0 |
| +1e-9 | 1.0000000005 |
| +1 | 1.5819767068693262 |
| +100 | 100.00000000000003 |

All agree within 1e-12 relative error. The near-zero signed difference also verifies the first-order coefficient 1/2. This is a law check, not coefficient selection or empirical calibration.

### Remaining failure R1 — observation can change the physical particle phase (P2)

**Do not treat the first aerosol v2 snapshot as having a complete physical-context binding.** The lineage correction retains Evidence but does not retain or compare the generating physical context. `observe` still compares only condition_id and time_origin strings before interpreting particle sections with the supplied optical context.

The actual-source independent probe propagates a liquid-particle scenario. It then creates a self-consistent solid-particle observation context by changing `particle_phase` to Solid and changing every particle-composition fraction's phase to Solid. The detector refractive-index and nonabsorbing-support sample phases are also Solid. All condition, time-origin and composition identity strings remain the same. `observe` accepts the liquid prediction with solid optical properties and emits beta=7.6734412542424942e-14 /m. No phase-transition or material transformation was computed.

The marker `V2_REMAINING_CONTEXT_PHASE_REBIND` reproduces this remaining boundary failure. Merely checking that the new context is internally valid cannot fix it; bind and compare the generating phase/composition/thermodynamic/frame context or provide an explicitly justified transformation. The detector-truth dependency set may legitimately expand and should still be rechecked separately. This finding was sent to the coordinator and aerosol author before any freeze. R1 is separate from F3's original truth-lineage case, which is now closed.

### Snapshot identities and executed commands

First v2 source/report snapshot hashes (before any R1 fix):

| Path | SHA-256 |
|---|---|
| crates/ignis-wsp/src/property_aerosol_v2.rs | fadc521d4b2153286298456cdb554d0b6ba6970fe76f4acfc37e6685d93a4aef |
| crates/ignis-wsp/src/property_jets_v2.rs | 7f2fa2fd7a93ccf9c15bd81678e93c4be3499b639c4140096f16ef938ad38ff7 |
| docs/wsp/research/astra_aerosol_v2.md | 2c7686bee5dcec72e90440f045fbe19cca5f2336cdd862546ef843cbe09c6e3a |
| docs/wsp/research/astra_jets_v2.md | 7b13060d1188378bef59ee6ed22bf8960fb27fa464e3245ca3b125f0babcb4f7 |
| configs/models/aerosol/property-v2/law-review-astra/constants.ops | 6255cdf9c3e5a39da9db3e96a37ebd74a69f5b70c5f34b53f8dd20f9974b00dc |

The aerosol literal hash equals its original W0 literal hash. The jet wrapper hash matches the coordinator's supplied identity and delegates unchanged constants to the original kernel. No spec, source, packet or literal was edited by this reviewer.

Executed successfully after the documented inverse-domain assertion adjustment:

```powershell
rustc --edition 2021 runs/tooling/astra-transport-review/aerosol_v2_independent.rs -o runs/tooling/astra-transport-review/aerosol_v2_independent.exe
& runs/tooling/astra-transport-review/aerosol_v2_independent.exe
rustc --edition 2021 runs/tooling/astra-transport-review/physics_v2_independent.rs -o runs/tooling/astra-transport-review/physics_v2_independent.exe
& runs/tooling/astra-transport-review/physics_v2_independent.exe
rustc --edition 2021 --cfg predecessor runs/tooling/astra-transport-review/aerosol_predecessor_compare.rs -o runs/tooling/astra-transport-review/aerosol_compare_w0.exe
rustc --edition 2021 runs/tooling/astra-transport-review/aerosol_predecessor_compare.rs -o runs/tooling/astra-transport-review/aerosol_compare_v2.exe
$a = & runs/tooling/astra-transport-review/aerosol_compare_w0.exe
$b = & runs/tooling/astra-transport-review/aerosol_compare_v2.exe
if (($a -join "`n") -cne ($b -join "`n")) { throw 'Predecessor numerical output mismatch' }
```

Final runs exited 0. Compilation emitted only unused-import/unused-variable warnings in the standalone harness context. The first v2 aerosol probe intentionally asserts that R1 still succeeds, so its exit 0 is confirmation of that residual failure, not full acceptance. No Cargo or author test suite was run by this reviewer. No qualification or freeze authority is inferred.

## Final aerosol-context correction receipt — 2026-11-14

**R1 is closed at the exact tested aerosol v2 source hash below. No remaining blocker was found in this targeted context/lineage re-review.** This supersedes the first-v2-snapshot R1 finding above; it does not rewrite or erase that earlier evidence. The original W0 findings remain findings against the frozen originals. This receipt is an independent synthetic/code review result, not a model qualification or authorization to expose protected data.

The reviewer read the current aerosol v2 context/provenance changes and its own author report only, reusing the authorized synthetic review scripts. No prohibited data, NASA outcomes, dossier feedback, external scientific results or other family revisions were inspected. The source hash was checked before compilation, after execution and again immediately before saving this durable receipt; all three reads matched. The resumed owner request required only saving this receipt, so no numerical file or probe was changed during that final checkpoint step.

### Physical/context admission now verified

`Prediction` retains the generating Context and original particle-density Sample privately. `check_context` first requires the original detector-truth set to be a subset of the supplied set; it then compares the entire physical/provenance Context after normalizing only that truth set to its original value. It revalidates the context and checks all retained predictive Evidence against the expanded truth set. `observe` and `moments` both call this gate; `moments` additionally requires full equality of the supplied density sample to the retained one.

The independently written actual-source harness `runs/tooling/astra-transport-review/aerosol_v2_context_independent.rs` confirmed:

- The exact strengthened same-ID Liquid-to-Solid substitution now fails with `observation physical context or provenance differs from generating context`. Both the particle-composition fraction phases and detector optical-property phases are changed consistently, so rejection is due to the generating-context binding, not merely an internally inconsistent replacement context.
- An alternate independently sourced detector with a different responsivity/calibration source works against the preserved generating context. Unrelated additional detector-truth identities are admitted. Scattering coefficient and transmitted power remain identical; detector voltage changes according to the independently supplied responsivity, to 0.08999999999999878 V in this synthetic case. `moments` also accepts the legitimate truth-set expansion.
- Adding either the initial-number source identity or its transitive raw dependency to the truth set rejects both observation and moments. Adding either the alternate detector's calibration source or its raw dependency rejects observation. Thus adding truth metadata cannot hide a retained dependency collision.
- Removing an originally selected truth identity is refused even when another truth identity remains in the set.
- Eleven individual context substitutions are refused by both observation and moments: temperature, pressure, gravity, coordinate frame, gravity-regime tag, gas-composition provenance, particle-composition provenance, temperature reference, temperature uncertainty, condition identity and time origin.
- Seven density substitutions are refused by moments: numerical density, provenance locator, dependency closure, origin tag, reference, diameter-domain bound and uncertainty. The stored density and context have no public mutable field or mutable getter.

Exact Context equality intentionally also binds its original provenance records. A newly independently measured physical context is not silently substituted even if some numbers happen to match: it needs a new propagation or a separately justified transformation. The legitimate alternate observation tested here changes independently sourced detector inputs while preserving the generating physical context. Complete transitive evidence remains a caller responsibility; this kernel cannot discover omitted dependencies or establish experimental validity from source labels.

### Numerical regression checks retained

The context harness reruns the earlier exact F1-F3 aerosol counterexamples and the extra source-volume diameter case; all reject correctly. Closed coagulation conservation and causal source delay still pass with their previously recorded values. The seven signed-film checks at Pe=-100,-1,-1e-9,0,+1e-9,+1,+100 still agree with the independent steady-film boundary-value solution, including the D/h limit and first-order signed coefficient 1/2.

The existing independent predecessor comparison was recompiled against this final source. Its combined emission, delayed line transport, line losses, coagulation, dilution and predecessor-additive-deposition scenario still produces exactly equal complete sections, Budget, Moments and step count to the frozen W0 executable. The new context gate changes admission and lineage retention, not the accepted scenario's numerical trajectory.

### Exact identities

| Artifact | SHA-256 |
|---|---|
| crates/ignis-wsp/src/property_aerosol_v2.rs | 9831d4ea5e2190dee23dee2da70a2a7aabb626172681e7cbabc1a31da1251302 |
| docs/wsp/research/astra_aerosol_v2.md | 54b81d2dcaedb63f4f5de2b6de1b238ae7fbc783a7269f9e80bb720bc6dc0718 |
| runs/tooling/astra-transport-review/aerosol_v2_context_independent.rs | 58b7b4e60cf270417be0732ba8f007b6111784c1e2690b935023f0c55b85e088 |
| configs/models/aerosol/property-v2/law-review-astra/constants.ops | 6255cdf9c3e5a39da9db3e96a37ebd74a69f5b70c5f34b53f8dd20f9974b00dc |

The last hash remains identical to the original aerosol literal assignment file. The existing predecessor-comparison script is unchanged from the preceding re-review; its original and v2 builds select their modules by `--cfg predecessor` and the default configuration respectively.

### Commands executed for this correction

From `.`, before the interruption and resumed receipt checkpoint:

```powershell
rustc --edition 2021 runs/tooling/astra-transport-review/aerosol_v2_context_independent.rs -o runs/tooling/astra-transport-review/aerosol_v2_context_independent.exe
& runs/tooling/astra-transport-review/aerosol_v2_context_independent.exe
rustc --edition 2021 runs/tooling/astra-transport-review/aerosol_predecessor_compare.rs -o runs/tooling/astra-transport-review/aerosol_compare_v2.exe
$a = & runs/tooling/astra-transport-review/aerosol_compare_w0.exe
$b = & runs/tooling/astra-transport-review/aerosol_compare_v2.exe
if (($a -join "`n") -cne ($b -join "`n")) { throw 'Predecessor numerical output mismatch' }
```

All commands completed successfully with exit code 0. The comparison harness emitted only its existing unused-variable warnings. Its equality check printed `CONTEXT_FIX_VALID_PREDECESSOR_NUMERICAL_EQUALITY_PASS`; the context harness printed the individual refusal/acceptance markers described above. These tests ran against the actual source at hash 9831d4ea..., not against an edited copy. No Cargo, dependency download, model evaluation, parameter search, source/spec/literal edit, release operation or commit was performed by this reviewer. The resumed checkpoint only reconfirmed hashes and appended this report section.

The remaining limits are the already declared scientific/model and shared-contract gaps: experimental and unqualified aerosol transport/optics, explicit applicability assumptions, unquantified uncertainty, unresolved within-section distributions, and reliance on complete input provenance. This bounded review does not certify real-material support, measured accuracy, NASA agreement, production integration or any protected evaluation outcome.

# Astra: smoke, aerosol and dust — distributions, conservation and instruments

2026-11-14. Fundamental theory/methods supplement for the nineteen-investigation programme. **S** = sourced fact; **D** = independent symbolic derivation; **P** = prospective requirement. No numerical aerosol coefficient, model weight, implementation, measurement, fit or score is supplied. Geometrical/algebraic factors below are derived, not selected candidate parameters.

## Exposure and scope

This analyst is exposed through historical aggregate handoff material and unintended publisher snippets recorded in the companion dimensional, spherical and BRE notes. The analyst is not a clean-room W0 author. No exposed NASA outcome value or trend is used here. No new atlas outcome, protected time series, model release, further handoff or result dossier was opened. All new-investigation outcomes remain closed under the assignment.

SAME, SAME-R and DAFT/DAFT-2 motivate an aerosol family only at the level of the owner's named scope. This note does not assert which classifier, particle material, charging history or measurement moment was used in any particular run. Actual apparatus methods need a safely screened source package. Primary methods read here concern generic modal aerosol descriptions, particle classifiers and deposition, without opening NASA investigation papers or cited-by pages.

## 1. The state is a distribution and the target is an instrument-defined functional

**D.** A particle's relevant attributes can include material volume/mass, composition, morphology, charge and water content, together with position and velocity. Reducing them to one diameter requires a declared equivalence. Compact spheres of known constant density admit a simple volume-to-mass relation; soot aggregates, porous dust and hygroscopic particles may not.

Let `n(v,x,t) dv` be the number of particles per gas volume whose material volume lies in `[v,v+dv]`. A schematic population balance is

\[
\partial_t n+\nabla_x\!\cdot[(u+v_{drift})n-D_p\nabla_x n]
+\partial_v(\dot v\,n)
=\mathcal C[n]+S_{nuc}+S_{external}-L.
\]

Coagulation is `C`; surface growth/evaporation contributes the volume-coordinate flux; source and loss definitions must avoid double counting. A composition- or charge-dependent system may require a joint distribution, not just `n(v)`. Gas-to-particle condensation removes the corresponding gas species and transfers enthalpy; it is not a source of matter for the combined system.

An aerosol transport calculation supplied with a measured source distribution is a conditional transport prediction. It does not predict smoke production from an unknown burning material. Coupling to flames requires formation, oxidation and gas/particle exchange or an explicitly imposed source; carbon content alone does not supply a soot yield.

## 2. Coagulation has exact moment identities before closure

**D.** Assume binary sticking/coalescence with additive material volume and no fragmentation. A symmetric collision kernel `K(v,w)` has units of gas volume per time. The gain-loss operator is

\[
\mathcal C[n](v)=\frac12\int_0^v K(w,v-w)n(w)n(v-w)\,dw
-n(v)\int_0^\infty K(v,w)n(w)\,dw.
\]

Define volume-coordinate moments `M_q=int v^q n(v) dv`. Interchanging integrals gives the exact coagulation contribution

\[
\dot M_q\big|_{coag}=\frac12\int_0^\infty\!\int_0^\infty
K(v,w)n(v)n(w)[(v+w)^q-v^q-w^q]\,dv\,dw.
\]

For `q=0`, number decreases by one per binary merger. For `q=1`, material volume is conserved. For constant particle material density, this also conserves particle mass. If chemical composition/density changes during collision, track additive component masses instead. Numerical bins must preserve the additive quantity even when the merged size lies between bins.

For fully coalesced compact spheres, surface area is proportional to `v^(2/3)` and decreases on merger by concavity. For fractal aggregation without full coalescence, accessible surface area need not follow that spherical rule. A moment called “surface area” can therefore be a geometrical model output rather than a direct measurement of reactive surface.

In a continuum diffusion-limited spherical collision model, independent particle diffusion gives a kernel proportional to the relative diffusivity and capture radius; the absorbing-sphere boundary problem yields

\[
K_B=4\pi(D_{p,v}+D_{p,w})(a_v+a_w).
\]

Finite sticking, electric interaction and transition/free-molecular transport change this model. Their parameters must have independent provenance. This expression is an analytical reference limit, not a numerical kernel selected for a NASA aerosol.

## 3. Moment methods need an explicit distribution assumption

**S.** [EPA's CMAQ Chemical Process Overview](https://www.epa.gov/cmaq/chemical-process-overview), “Airborne Particle Microphysics,” describes lognormal populations and tracked number, surface-area and volume moments, together with condensation, coagulation and formation. This is one modal approximation for atmospheric particles. Its mode structure is not assumed appropriate for an instrument-generated smoke/dust population.

**D.** General kernels and growth laws make a low-order moment equation depend on additional moments or the full distribution. Merely storing a few moments does not close their dynamics. A lognormal shape is one closure; a section/bin representation is another; a prescribed narrow distribution is a restrictive boundary condition. None follows from dimensional analysis alone.

To make the closure visible, define diameter moments `D_k=int d^k n_d(d) dd`; these are different from the volume moments `M_q`. For a single lognormal diameter population with number `N`, reference length `d_ref`, `mu=ln(d_g/d_ref)` and `s=ln(sigma_g)`, integration gives

\[
D_k=N d_{ref}^k\exp(k\mu+k^2s^2/2).
\]

Under that assumption only, the measured/modelled moments `N,D_2,D_3` determine

\[
s^2=\frac23\ln\frac{D_3}{Nd_{ref}^3}
-\ln\frac{D_2}{Nd_{ref}^2},\qquad
\mu=\frac12\ln\frac{D_2}{Nd_{ref}^2}-s^2.
\]

A negative inferred `s²` is incompatible with this assumed single-lognormal representation or the supplied moments. It is not repaired by silently clipping the width. The zero-width limit is a delta distribution and should be treated as a limit, not a division by zero. The reference length cancels from physical predictions.

Even physically consistent low-order moments do not uniquely specify a distribution. Different multimodal/tail populations can share them while producing different coagulation rates, detector responses or filter penetration. Thus a claimed forecast of a high-order optical signal from three moments must expose the closure uncertainty.

## 4. Mobility ties diffusion, settling and electrical drift together

**D.** Let mechanical mobility `B` mean velocity divided by force in the low-inertia drag regime. For an equilibrium Brownian particle, `D_p=k_B T B`. Gravitational settling uses its effective weight, while electrical drift uses its charge:

\[
v_g=B(m_p-\rho_g V_{disp})\,g,\qquad
v_E=Bq_p E.
\]

These are vector relations under their stated drag approximation. A single consistent drag/morphology description should feed all three effects; selecting unrelated “diffusion,” “settling” and “electric” multipliers can violate that consistency. The continuum sphere relation must be corrected when gas mean free path, particle shape or inertia demands it. Temperature and pressure changes affect those corrections as well as gas viscosity.

Gravity tending to zero removes settling driven by weight. Brownian coagulation/diffusion, wall losses, thermophoresis, imposed flow and electrical drift can remain. A zero-gravity aerosol is therefore not a static population. A chamber may have strong size selection through sampling even when gravitational selection is negligible.

**S.** [Yao et al., NIST-hosted 2020 manuscript](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=929243), §3, separates mechanical/electrical mobility, aerodynamic classification and mass-to-charge classification. Its transfer-function discussion treats finite selection widths in their native measurands. Its source-specific geometry, slip coefficients, example values and measured comparisons are not adopted.

Useful dimensionless coordinates include particle Knudsen number; particle relaxation time relative to flow time; particle Péclet number; coagulation time relative to residence time; settling/drift crossing time relative to residence; condensation time relative to sampling time; optical depth; and humidity/composition coordinates controlling water uptake. The relevant carrier/particle size definition must accompany every such group.

## 5. Deposition is transfer to another inventory

**D.** A wall flux such as `J_dep(v)=v_dep(v)n_wall(v)` removes particles from suspended number and mass inventories. Add the same amount to a wall inventory unless it leaves the declared system. Deposition, wall reaction, resuspension and outlet escape are distinct processes. A model that removes airborne soot without recording its destination may satisfy a concentration curve while violating total material conservation.

**S.** The pinned [FDS 6.0.1 mathematical-model manual](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.1018e6.pdf), §7.3, distinguishes gravitational, thermophoretic and diffusion/turbulence deposition and records deposited mass on surfaces. Its assumed additive empirical deposition velocities and compartment-fire omissions are not universal laws for the present family.

For a perfectly mixed chamber with no source, a constant collision kernel `K` and size-independent first-order removal `lambda`, the number balance is an exact benchmark of those assumptions:

\[
\dot N=-KN^2/2-\lambda N,
\qquad
N(t)=\frac{N_0e^{-\lambda t}}
{1+[KN_0/(2\lambda)](1-e^{-\lambda t})}.
\]

For zero removal its continuous limit is `N_0/[1+K N_0 t/2]`; for zero coagulation it is exponential removal. Coagulation preserves material volume, whereas actual removal decreases suspended volume. Therefore a number-decay curve alone cannot identify which process dominates. This benchmark supplies no numerical `K` or `lambda` and is not an adopted flight law.

With size-dependent removal, a single constant cannot generally close all moments. Finite chamber mixing and inlet/outlet short circuits can also invalidate the perfectly mixed assumption. Walls and sampling lines are part of the apparatus physics, not a harmless post-processing correction.

## 6. A particle-size instrument measures through a transfer function

**D.** A general response channel can be represented schematically by

\[
y_j(t)=b_j+\int_{-\infty}^t h_j(t-\tau)
\left[\int\mathcal K_j(\psi;\eta_j(\tau))n(\psi,\tau)\,d\psi\right]d\tau,
\]

where `psi` includes the particle attributes needed by the instrument, `eta_j` includes settings/environment, `K_j` includes sampling/selection/detection and `h_j` includes time response. This is linear only under the assumed dilute, nonsaturating response. Coincidence, dead time or loading can make it nonlinear.

| Measurement | Required interpretation |
|---|---|
| Mobility diameter | Equivalent sphere under a drag, gas-state and charge convention; not automatically geometric diameter |
| Aerodynamic diameter | Equivalent response to inertia/drag under a stated reference density; shape/density remain relevant |
| Particle mass analyzer | Classified mass-to-charge unless charge is independently known |
| Optical particle counter | Signal-to-size map depends on optical properties, shape, wavelength and calibration material |
| Condensation particle counter | Detection efficiency, activation, losses and counting response are part of the number estimate |
| Extinction/scattering | Weighted cross-section integral; not a direct mass concentration without an additional relation |

For a DMA-like classifier, multiple charge states can give the same mobility for different physical particles. Charge conditioning does not mean every particle is uncharged or singly charged. A scan acquired while the population evolves is not an instantaneous spectrum. Its reported distribution can depend on scan direction and time assignment unless the temporal evolution is accounted for.

Optical extinction has the generic form `beta_lambda=int C_ext(psi,lambda)n(psi)dpsi`. It is not necessarily one of the retained low-order moments. Particle composition, aggregate morphology and wet/dry state can change the cross-section at fixed particle mass. A visibility or detector-alarm forecast therefore requires both population dynamics and an observation/decision model.

Instrument inversion/calibration belongs to the independent measurement lane. A forecaster should not revise transfer-function widths or charge fractions to improve agreement with protected outcomes. Comparisons may be made either in a well-defined raw-signal space or to independently processed measurands with propagated covariance, but the two routes must not be mixed silently.

## 7. Conditions-only predictions, uncertainty and prospective checks

Required initial/forcing inputs include the particle population or an explicit prior family, material/shape/charge information, source history, gas state, humidity, geometry/flow/gravity, wall/sampling properties and instrument settings. A future measured concentration is an observation, not a condition available to an autonomous forecast. If source chemistry is unknown, the honest product may be conditional aerosol transport rather than a prediction of smoke generation.

Uncertainty separates source, shape/composition, kernel/closure, deposition, mixing and instrument effects. Low-order moment uncertainty is correlated because the moments often come from the same inversion. Missing tails can dominate mass or optical response while leaving number apparently precise. A distributional closure error is not equivalent to uncertainty in one scalar width.

Proposed outcome-free checks, not run and not evidence of NASA validity:

- Preserve nonnegativity and additive component mass through coagulation; recover the exact number/volume moment identities.
- Recover the constant-kernel chamber limits and track every removed particle's material destination.
- Check moment realizability and lognormal inversion only when that shape assumption is explicitly selected.
- Preserve consistent mechanical mobility across Brownian diffusion, settling and electric drift in their shared limit.
- At zero gravity remove only gravity-driven terms, retaining other processes.
- Verify volume/diameter coordinate changes, Jacobians and moment units; do not equate the two moment index conventions.
- Pass synthetic distributions through finite-width, multiple-charge and temporal response operators; distinguish this from calibration on real blind measurements.
- Demonstrate that changing arbitrary bin edges or time-grid resolution does not create or destroy mass under a future implementation's declared convergence regime.

## Source/read ledger

Access: 2026-11-14. Retrieved bytes were handled in memory. No external source coefficient or numerical mode setting was selected.

| Source | Actual reading extent | Pin |
|---|---|---|
| EPA CMAQ Chemical Process Overview | Web text through “Airborne Particle Microphysics,” especially its Scientific Approach; additional adjacent photolysis/multiphase/cloud excerpts returned. Only modal/phase-transfer methodology used. | HTML SHA-256 `6d04100205a0ccb01a18a634fadd02c89fb363556047c01d40a94623ceaea7ae` |
| Yao, Asa-Awuku, Zangmeister and Radney, 2020 | NIST-hosted submitted manuscript AST-MS-2020-015.R1, submitted 7 April 2020. Physical pp. 1–2 metadata/abstract; pp. 4–10, including §2 methods and §3.1–3.5 classifier/transfer/inversion theory. §3.6 heading only; later body/results not read. Generic metrology outcomes appearing in the abstract/method examples were seen but no values adopted. | PDF SHA-256 `01506a989ba833346c1eb20abecfcdac2b0d84ea50d68b33ab9ca3a51181e446`; published DOI `10.1080/02786826.2020.1763248` |
| FDS Technical Reference Guide, Volume 1 | FDS 6.0.1, SVN 17529, 26 November 2013. Full physical pp. 98–99 (printed 74–75), §7.3 Aerosol Deposition. Adjacent solid methods read for companion note. | PDF SHA-256 `ae8152d2cfdbf547e49abfd3ee8cf803db9e44cb7f2919d256d96149a8925bb6` |

Generic searches also returned atmospheric hygroscopicity and classifier-method abstracts. Their fitted/example values were not adopted, and those papers were not opened as substantive sources. No NASA investigation-result route was followed. These notes establish neither the actual flight instrument nor a qualified smoke/dust forecast. No forward, literal patch, experiment, score, commit or push was produced.

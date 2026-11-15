# Astra: multicomponent droplets — interfaces, composition and chemical branches

2026-11-14. Theory/methods supplement for the nineteen-investigation programme. This document supplies symbolic reasoning, input requirements and proposed checks, not numerical weights, property values, a forward implementation or experimental qualification. **S** denotes a sourced statement; **D** an independent derivation with explicit assumptions; **P** a proposed requirement.

## Exposure and scope

This is an exposed theory analyst, not a clean-room numerical author. Historical handoff aggregate exposure and later unintended publisher snippets are recorded in `astra_all_nasa_dimensional_basis.md`, `astra_spherical_all_nasa.md` and `astra_bre_all_nasa.md`. No exposed outcome values or trends are used here. New-investigation outcomes remain closed under the assignment. No atlas outcome, protected series, model release, further handoff or result dossier was opened for this note. Fresh W0 authors need a separately screened symbolic source/specification package.

The earlier Matalon spherical-interface reading is reused explicitly; it was not re-read in full. New substantive reading concerns multicomponent phase-equilibrium thermodynamics. An LLNL kinetic report was inspected only at its title/disclaimer pages; its result-bearing body is pending custodian screening. Nothing here establishes the actual apparatus or fuel history of a NASA run. FLEX, FLEX-2 and CFI are prospective applications of the same interface family, conditional on safe metadata and later independent measurement.

## 1. A droplet requires more state than a diameter

**D.** A minimal candidate state depends on the assumptions being defended. A well-mixed single-phase liquid can be represented by total mass, independent component inventories, temperature and position/velocity. If internal diffusion or thermal relaxation is slow, replace the lumped composition/temperature by fields. If liquid phase separation, precipitation, bubble nucleation or strong shape deformation occurs, a single homogeneous sphere is no longer an adequate state.

The gas state must represent the transport and chemistry necessary for the target. A hot reaction sheet may suffice for an ideal mixing benchmark; it cannot represent a low-temperature radical pool. Soot-shell position requires particle transport and formation/oxidation, not only liquid mass conservation. Support fibers, ignition and chamber walls create additional thermal and geometrical boundaries.

**S.** [Matalon, CEFRC Lecture 12, Summer 2013](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-12.pdf), selected pp. 3–9, derives spherical droplet conservation and moving-interface jump conditions under an explicitly reduced problem. Its treatment supports keeping interface mass and energy balances coupled. It does not establish multicomponent liquid uniformity or a particular low-temperature mechanism.

## 2. Thermodynamic equilibrium is a boundary relation, not an evaporation rate

**D.** If local phase equilibrium is justified, each volatile component satisfies equality of chemical potentials across the interface. For a planar interface this can be expressed as equality of fugacities. A useful symbolic representation is

\[
y_i\phi_i p=x_i\gamma_i f_i^{sat}(T_s)\,
\exp\!\left[\int_{p_i^{sat}}^{p_l}\frac{v_i^{l,*}}{RT_s}\,dp\right].
\]

Here `x_i` and `y_i` are interfacial liquid and gas mole fractions, `gamma_i` is a liquid activity coefficient at the surface state with a pure-liquid standard state, `phi_i` is a gas fugacity coefficient, and `v_i^(l,*)` is that pure-liquid standard state's molar volume. The exponential is its pressure correction, not an extra correction using a mixture partial molar volume after pressure dependence has already entered the activity coefficient. In a curved droplet, liquid/gas pressures differ through interfacial stress; a consistent curvature correction may matter at sufficiently small radius. Do not apply both a curvature correction and a pressure correction that already accounts for the same chemical-potential shift.

For ideal vapor, an ideal liquid, a nearly planar interface and negligible pressure correction, this reduces to `y_i p=x_i p_i^sat(T_s)`. That is a limiting assumption. A mixture's vapor composition generally differs from its liquid composition. The vapor-pressure relation supplies a boundary state, while diffusion, Stefan flow, surface kinetics and energy supply determine the net flux.

**S.** The [IDAES 2.8.0 VLE documentation](https://idaes-pse.readthedocs.io/en/2.8.0/reference_guides/model_libraries/generic/property_models/activity_coefficient.html) explicitly distinguishes ideal vapor, ideal liquid and NRTL/Wilson liquid activity models. Its activity models require binary interaction information. Its smooth flash construction is for an equation-oriented phase-equilibrium package; neither its solver nor its smoothing parameters are adopted here.

**D/P.** A finite evaporating interface is not a closed, globally equilibrated flash vessel. Ambient gas can contain an inert diluent and unsaturated fuel vapor, while the surface alone approaches equilibrium. If accommodation or Knudsen effects make local equilibrium inadequate, specify a kinetic interface model and its independently sourced parameters. A liquid correlation used above its critical or phase-stability domain is invalid even if it returns a finite number.

Thermodynamic properties should come from a mutually consistent state description. For example, with `mu_i=mu_i^0+RT ln(x_i gamma_i)`, the excess partial molar enthalpy follows from the temperature derivative of the excess chemical potential:

\[
\bar h_i^{E}=-RT^2\left(\partial_T\ln\gamma_i\right)_{p,x}.
\]

Thus a temperature-dependent activity model and an independently imposed ideal mixing enthalpy may be inconsistent. Pure-component heat capacities alone do not identify excess mixing enthalpy. The constant-temperature Gibbs–Duhem constraint also couples activity coefficients; they cannot be supplied as unrelated curves. These are consistency requirements, not numerical parameter choices.

## 3. Component conservation reveals differential volatility

**D.** Let `n` point from liquid to gas and let the surface move with velocity `v_s`. With no interfacial storage or surface reaction, the mass flux of each component is continuous:

\[
\rho_lY_{i,l}(u_l-v_s)\!\cdot n+j_{i,l}\!\cdot n
=\rho_gY_{i,g}(u_g-v_s)\!\cdot n+j_{i,g}\!\cdot n
=\dot m_i''.
\]

The barycentric diffusive fluxes sum to zero within each phase. Summing the component conditions gives total mass conservation. Imposing both a prescribed surface concentration and an unrelated prescribed component flux can overdetermine the interface.

For a well-mixed liquid of mass `m` with liquid mass fractions `w_i`, define outward component evaporation rates `m_dot_i` and total `m_dot=sum m_dot_i`. Then

\[
\frac{dm}{dt}=-\dot m,\qquad
m\frac{dw_i}{dt}=w_i\dot m-\dot m_i.
\]

This immediately distinguishes bulk depletion from surface equilibrium. Even if the initial composition is known, it remains constant only if the evaporative flux composition equals the bulk mass composition throughout the trajectory.

Under the ideal-vapor equilibrium assumptions above, a relative-volatility ratio is

\[
\frac{y_i}{y_j}=\frac{x_i}{x_j}
\frac{\gamma_i p_i^{sat}(T_s)}{\gamma_j p_j^{sat}(T_s)}.
\]

The second factor can depend on temperature and composition. It does not by itself determine `m_dot_i/m_dot_j`, because species diffusivities and counterdiffusion also matter. A slowly diffusing liquid can develop a depleted surface layer even while the bulk contains a volatile component. A single averaged boiling temperature cannot encode this memory.

**P.** A fuel property block should represent an actual mixture or a declared surrogate composition. Matching density and total heat of combustion does not uniquely match volatility, liquid mixing or chemical pathways. An unknown commercial-fuel composition is a scenario uncertainty; a fuel-name token cannot supply it invisibly.

## 4. Energy and the precise limits of the diameter-squared law

**D.** For a well-mixed drop at approximately constant pressure, an enthalpy inventory can be written schematically as

\[
\frac{dH_l}{dt}=Q_{gas}+Q_{rad}+Q_{support}
-\sum_i\dot m_i\bar h_{i,g},
\]

with all transfer terms signed into the liquid and a consistent common enthalpy reference. In this mass-inventory equation, `h_bar_i,g` is a mass-specific partial component enthalpy (the molar quantity divided by the component molar mass). `Q_gas` excludes the separately listed radiative exchange. When differentiating `H_l(T,m_i)`, component-removal enthalpies emerge on the left. The resulting thermal equation contains partial vaporization enthalpies; adding another full latent-heat loss after that expansion would double count. A resolved liquid instead requires an energy field and moving-boundary jump conditions. Radiation may be absorbed in depth rather than only at the surface.

As an analytically useful limit, take one volatile species, constant liquid density, a spherical quasi-steady gas, constant `rho_g D_g`, a nonreacting diffusion/Stefan problem and known surface and ambient vapor mass fractions. Define

\[
B_M=\frac{Y_{F,s}-Y_{F,\infty}}{1-Y_{F,s}}.
\]

The steady spherical mass flux and drop inventory imply

\[
\dot m=4\pi a\rho_gD_g\ln(1+B_M),\qquad
\frac{d(d_l^2)}{dt}=-\frac{8\rho_gD_g}{\rho_l}\ln(1+B_M),
\quad d_l=2a.
\]

This is a reference limit, not an asserted law for every burning drop. A reactive problem changes the coupled thermal/species boundary problem. For mixtures, independently adding single-species logarithmic fluxes usually violates the shared Stefan-flow constraint. If `rho_l` changes with temperature or composition, the spherical inventory alone gives the extra term

\[
\frac{d(d_l^2)}{dt}=-\frac{4\dot m}{\pi\rho_l d_l}
-\frac{2d_l^2}{3}\frac{d\ln\rho_l}{dt}.
\]

The first expression follows on substituting the special mass-flux law into this identity when density is constant. Thermal expansion can change diameter without creating liquid mass. Consequently, an observed slope in diameter-squared is not automatically a chemical burning constant or a mass-loss rate.

## 5. Low-temperature chemistry cannot be recovered from bulk fuel properties

**S.** [LLNL's combustion-methods overview](https://combustion.llnl.gov/) describes assembling kinetic models from reaction pathways, thermochemical information and rate rules informed by fundamental calculations and measurements. A handbook table of liquid properties is a different information source. The generic low-temperature section returned in the search excerpt for [LLNL-JRNL-401196](https://www.osti.gov/servlets/purl/946943) identifies competition between alkyl-radical oxygen addition and its reverse process, and pressure dependence of the regime. Only that excerpt and report metadata were read; no full mechanism or parameter set is claimed audited.

**D.** Consider, schematically, a reactive intermediate with competing conversion, reversal and termination routes. Its branching fraction depends on ratios of route rates, not merely total fuel heating value or stoichiometric oxygen demand. Distinct mechanisms can have identical element balance and net reaction enthalpy while having different intermediate lifetimes and ignition/extinction branches. A mixture can also share radicals between components, so summing pure-fuel burn times is not a chemical mixing rule.

Radius histories constrain an integrated evaporation/heat-feedback effect. Without independent temperature/species diagnostics, they generally cannot separate a changing liquid composition, radiative loss, support heat transfer, weak reaction and detector sensitivity. A disappearing visible flame is not proof that all chemical heat release ended; continued liquid shrinkage is not proof of a particular low-temperature mechanism. These observations must remain separate endpoints.

**P.** A cool-flame-capable release needs a named, versioned chemical reduction and explicit state/initial-branch assumptions. This note supplies none of its numerical rates. Until the required fundamental mechanism has passed source screening and literal-origin review, report that target as unavailable or as a clearly assumed scenario, rather than assign a universal late-burning multiplier.

## 6. Similarity, gravity and measurable quantities

**D/P.** Relevant independent comparisons include gas thermal/species Lewis numbers; `Re`, `Pe` and a gravity/diffusion comparison; liquid diffusion time `a²/D_l` versus evaporation time; liquid thermal time `a²/alpha_l`; thermal and solutal interfacial-stress effects if gradients drive internal motion; radiative optical thickness; latent/sensible enthalpy ratios; and the full relative-volatility/activity relations. A single diffusion time leaves these independent mechanisms unspecified.

Gravity can change gas convection, drop drift, shape, support interaction and liquid circulation. Setting gravity to zero removes its body force; it does not remove forced flow, residual acceleration, thermocapillary motion or ignition transients. Intermediate-gravity behavior need not lie between observed endpoint targets because the controlling branch can change. A partial-gravity scenario must retain orientation and be labelled unqualified where independent evidence is absent.

Liquid silhouette diameter, volume-equivalent diameter, flame-emission diameter and soot-shell diameter are different observables. A projected non-spherical silhouette does not uniquely determine volume. Refraction, blur, fiber occlusion, saturation, camera motion and calibration affect tracking. Flame emission additionally requires spectral/line-of-sight radiation and a declared segmentation rule, as discussed in `astra_spherical_all_nasa.md`.

Initial diameter and composition may be legitimate conditions if obtained independently before the forecast cutoff. A later measured diameter, flame radius or temperature cannot be inserted as a “property” into an autonomous conditions-only forecast. Optical disappearance and last visible frame require detection/censoring semantics.

## 7. Uncertainty and prospective checks

The uncertainty ledger should separate initial composition/temperature, thermophysical and activity properties, transport, chemical mechanism, support/chamber conditions and measurement operator. Preserve correlations among derived enthalpy, vapor pressure and density. Where multiple chemical branches are plausible, report branch-specific scenarios or a set of predictions rather than an unjustified narrow symmetric interval. Choosing the branch after seeing the outcome is model revision.

Proposed checks, not executed and not experimental validation:

- Preserve total mass and each element across the moving interface; verify that component fractions remain realizable.
- Recover the single-volatile Stefan limit only under its exact assumptions, and recover the zero-net-flux equilibrium limit when appropriate.
- Permute component labels with their full properties and obtain identical physical results.
- Check that equal-volatility/equal-transport components collapse to the corresponding indistinguishable-mixture limit; do not impose that limit on unequal components.
- Close energy with changing component inventories without counting vaporization twice.
- Separate changing density from mass loss when converting a synthetic diameter trajectory.
- Preserve the distinction between a prescribed boundary history and an autonomous prediction.
- Check optics on synthetic geometries; keep those checks separate from blind measurement error and withheld-run evidence.

## Source ledger and limits

Access date: 2026-11-14. Sources fetched in memory; no property tables, mechanisms or data were added. Hashes pin retrieved bytes, not experimental validity.

| Source | Actual reading extent / use | Pin |
|---|---|---|
| Matalon, Lecture 12 | Reuse of earlier full extracted pp. 3–9; earlier web excerpt pp. 1–2. No claim of a new complete lecture read. Interface assumptions and conservation only. | SHA-256 `2fb1d345f189ecc8b061f9ce3b01ac7fa0a5f122f0b5679b101deb96d438389e` |
| IDAES VLE/activity documentation | Version 2.8.0 introduction, inputs/degrees of freedom via web; full extracted section “VLE Model with Smooth Phase Transition” through NRTL, Wilson, variable lists and Initialization, stopping before Config Block Documentation. No parameter fitting, smoothing or solver imported. | HTML SHA-256 `866aa831803d0ead4821719386fa84454c6711dd7a5032c97ae500f85472fac5` |
| LLNL combustion overview | Full main-text overview through references read; only general mechanism-construction facts used. Fuel-surrogate claims are not treated as droplet evidence. | HTML SHA-256 `cbe1d4296d65eb75e5976fee82a552631201f3de5d7f049fb85b6a73530742d2` |
| Westbrook et al., LLNL-JRNL-401196, 11 February 2008 | PDF physical pp. 1–2 title/disclaimer only; separate search-returned low-temperature section excerpt. Remaining body pending custodian screening; not represented as a full primary kinetic-method read. | PDF SHA-256 `45b1327270683236567a314ff70e40099bdaa49929b9f630c214f1e1ac5d7a00` |

The initially attempted IDAES 2.9 path was unavailable; the actually read 2.8 edition is identified above. Cantera phase-model index/API metadata was also consulted but supplies no fuel-specific model here. No NASA apparatus-specific mixed result paper was opened. No forward, weight patch, experiment, score, fit, commit or push was produced.

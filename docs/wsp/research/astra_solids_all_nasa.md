# Astra: solids in flow — conjugate heating, conversion and moving flames

2026-11-14. Fundamental theory/methods research for the nineteen-investigation programme. **S** = source fact; **D** = independent symbolic deduction; **P** = proposed requirement. No numerical material coefficient, weight, forward implementation, measurement, fit or score is authored.

## Exposure, source boundary and applicability

This is an exposed theory analyst, not a clean-room W0 author. The historical aggregate handoff and unintended publisher-snippet exposures recorded in the companion dimensional, spherical and BRE notes still apply. Their outcomes are not reproduced or used. No new atlas outcome, protected series, further handoff, model release or result dossier was opened. All new-investigation outcomes remain closed under the instruction supplied to this analyst.

This note addresses the physical family that may serve BASS/BASS-II and Saffire, subject to safe geometry/material metadata. It does not claim the fabric, rods, foams, sheets or composites of any run are interchangeable. No investigation apparatus/result paper was opened. The new full selected primary-method read is the solid-phase chapter of a pinned historical FDS mathematical-model manual; generic radiometer methods read earlier are reused. Historical FDS options are methodological examples, not a proposed current default or an imported model.

## 1. The gas flame and the condensed material form a coupled problem

**D.** Gas-phase reaction produces heat; part returns to the solid; solid heating and conversion supply gases; those gases alter the flame. Geometry, oxygen delivery and losses close the loop. If surface mass flux or heat release is supplied from a measured future trace, the calculation becomes a conditional gas/flame forecast, not a prediction of the solid's autonomous burning rate.

A useful state hierarchy is:

- nonreacting solid: temperature field and independently known thermophysical properties;
- gasifying material: temperature plus component inventories and conversion-dependent properties;
- charring/porous material: virgin solid, char, moisture and pore-gas state, with evolving geometry/permeability where necessary;
- melting/dripping or deforming material: additional phase and mechanical/transport states;
- gas flame: flow, composition, chemistry and radiation at the fidelity required by the endpoint.

Choosing a simpler state is defensible only with a stated separation of scales and target. It is not made defensible by giving each investigation a different coefficient in the same scalar clock.

**S.** [FDS Technical Reference Guide, sixth edition, Volume 1](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.1018e6.pdf), FDS 6.0.1, §§7.1–7.2, separates heat conduction, radiation and solid reactions. Its reduced pyrolysis treatment assumes instantaneous product-gas release and does not solve pore-gas species transport. It distinguishes specified heat release from predicted gasification and describes density/thickness changes. These limitations, rather than its numerical defaults, are the facts used here.

## 2. Conservation before a pyrolysis-rate expression

**D.** Let `c_alpha` be mass of condensed component `alpha` per bulk volume. In a fixed material frame without swelling, its balance has the form

\[
\partial_t c_\alpha=\sum_r\nu_{\alpha r}\dot\omega_r,
\]

where the convention and units of each reaction extent are explicit. If the skeleton moves or deforms, add its advective/volume-change terms. Gas production, char production, oxygen uptake, water release and any omitted products must close mass and elements together. A char yield is not a free mass sink; untracked residual mass must remain an explicit inventory.

For a porous region, the gas-phase component balance requires storage and transport if they are not negligible:

\[
\partial_t(\epsilon\rho_gY_i)+\nabla\!\cdot
(\epsilon\rho_gu_pY_i+j_{i,p})=S_i.
\]

The pore velocity may need a permeability/pressure relation, and the transport tensor may be anisotropic. The surface flux is obtained from this balance. Replacing it by the depth integral of production assumes negligible pore storage and a rapid escape path; it is not exact for every char layer or thick sample.

Use one consistent energy convention. With sensible enthalpy excluding chemical formation terms, a schematic stationary-skeleton balance is

\[
\partial_t H_s+\nabla\!\cdot F_{p,h}
=\nabla\!\cdot(k_s\nabla T)-\nabla\!\cdot q_{rad}
-\sum_r\Delta h_r\dot\omega_r.
\]

`H_s` here is bulk sensible enthalpy per unit volume, including pore-gas storage when that phase is retained; `F_p,h` is pore-gas sensible enthalpy transport. If total component enthalpies already include formation energy, the chemical accounting must change accordingly; adding the same reaction enthalpy again would be wrong. Local gas/solid thermal equilibrium is another approximation; without it, separate phase energy balances and interphase exchange are required. Melting and water vaporization require latent/enthalpy bookkeeping rather than a temperature threshold that discards energy.

At the exposed boundary, use a signed heat balance with conduction into the solid, nonradiative gas transfer, net radiation and any enthalpy carried by mass crossing the interface. Radiation absorbed in depth belongs in a volume term; the same absorbed energy must not also be applied as a surface heat flux. The rear face, edges and support contacts need their own conditions.

## 3. Two exact heating limits show why thickness and history matter

**D.** Consider an inert, initially uniform semi-infinite solid with constant `k`, `rho c`, diffusivity `alpha=k/(rho c)`, no reaction and imposed net surface heat flux `q''(t)`. The linear heat equation gives a surface-temperature convolution,

\[
T_s(t)-T_0=\frac{1}{\sqrt{\pi k\rho c}}
\int_0^t\frac{q''(\tau)}{\sqrt{t-\tau}}\,d\tau.
\]

For a constant positive flux, this becomes `2 q'' sqrt(t)/(sqrt(pi k rho c))`. If a chosen thermal threshold `T_p` is reached while these assumptions hold,

\[
t_p=\frac{\pi k\rho c\,(T_p-T_0)^2}{4(q'')^2}.
\]

This is a heating benchmark. `T_p` is not automatically ignition temperature, and reaching it does not establish a flammable gas mixture or stable flame. The formula also demonstrates memory: equal total absorbed energies delivered with different temporal profiles need not produce equal surface temperatures.

For a thermally thin inert sheet whose thickness is nearly isothermal, with areal heat capacity `C_A` and negligible back/edge loss, the corresponding equation is

\[
C_A\frac{dT}{dt}=q'',\qquad t_p=C_A(T_p-T_0)/q''.
\]

Different flux dependence follows from different conduction regimes. The thermal penetration scale is `sqrt(alpha t_preheat)`. Compare it with material thickness; include back-face conditions and a Biot-type measure before choosing a lumped temperature. A specimen can move between regimes during one transient. A constant “thin/thick” label inferred from investigation name is inadequate.

## 4. Conversion, char and moisture create additional memory

**D.** A temperature-only rate cannot generally represent competing reaction paths, conversion dependence, oxygen access and product inhibition. An Arrhenius-like factor may appear in a particular kinetic reduction, but the reaction network, pre-exponential factors, activation energies and yields require independent provenance. Similar net mass-loss histories can arise from different parameter combinations and therefore do not identify a unique mechanism.

Char changes thermal conductivity, radiation absorption, permeability, effective area and oxygen accessibility. It can insulate underlying material while also participating in heterogeneous oxidation. A char front, a virgin-material conversion front and a visible gas flame can move differently. A glowing surface is not automatically a gas flame, and loss of gas luminosity does not establish that the solid ceased reacting.

Moisture needs a basis: dry-basis water content is water mass divided by dry-solid mass; wet-basis content uses total wet mass. Confusing the bases changes both available dry fuel and energy demand. Bound/desorbing water and free liquid water need not share one release law. Evaporation consumes energy, released vapor changes the adjacent gas, and condensation elsewhere can return energy. An initial moisture percentage alone does not specify its distribution or release kinetics.

**P.** Keep water, virgin solid and residual char inventories separate. Use temperature/conversion-dependent properties where the evidence requires them. If moisture or char properties are absent, expose scenario assumptions instead of treating the sample as a verified dry homogeneous solid. A literature coefficient for a different heating rate, morphology or oxygen history is not automatically transferable.

## 5. Upstream and downstream spread are different boundary-value problems

**D.** Let the material lie along `x`, and let a nominal conversion/spread front travel at `V_s` in the positive direction. In the moving coordinate `xi=x-V_s t`, an approximately translating solid thermal field has `partial_t T=-V_s partial_xi T`. The gas velocity relative to that frame is `u_lab-V_s e_x`. Its sign and profile determine whether fresh oxidizer approaches from ahead of or behind the advancing region.

Downstream/concurrent spread can put hot products and flame radiation over unheated material. Upstream/opposed spread has different advective preheating and flame-base attachment. These descriptions require the actual geometry and flow direction; “upstream” is not equivalent to a positive or negative gravity magnitude. Flame tip and flame base can have different trajectories, especially during growth, detachment or intermittent extinction.

An energy scale for advance is

\[
\rho_s H\,\mathcal H_{preheat}\,V_s
\sim\int_{\text{preheat region}}q_{net}''(x)\,dx
\]

for a uniform thin sheet with an appropriately defined enthalpy demand per mass. Both sides are power per unit width. This is a conditional scaling relation: the preheat length, flux profile, gasification/attachment state and losses are still unknown. Replacing the integral by a freely adjusted constant would hide the actual spread problem.

Thermally thick or finite-width samples require different heating volumes and lateral conduction. Curvature, gaps, fabric weave, edge exposure and melting can invalidate planar similarity. Large-scale spread can introduce flame turbulence, radiative exchange over longer distances, wall feedback and oxygen depletion. Increasing length at unchanged material properties does not preserve all relevant nondimensional groups.

## 6. Gravity, oxygen and convection cannot be collapsed into one monotone multiplier

**D.** Gas momentum contains the gravity vector acting on density differences. A buoyancy velocity scale is `sqrt(g' L)` when an appropriate reduced gravity `g'` can be defined. A diffusion-based comparison such as `g' L^3/alpha_g^2` remains meaningful when the imposed-flow speed vanishes; an inlet Richardson number may not.

Gravity changes heat transfer, flame orientation, residence, oxygen entrainment and possibly material drainage. Enhanced flow can increase oxygen/heat supply or increase cooling/stretch, depending on the state. The competing terms provide no general proof that spread, quenching or burning rate varies monotonically between normal and microgravity. A terrestrial empirical natural-convection correlation with an embedded reference gravity cannot be reused at partial gravity by changing an unrelated feature.

Oxygen mole fraction and total pressure are distinct inputs. Diluent identity affects density, heat capacity, transport and radiation. Equal oxygen partial pressure does not guarantee equivalent combustion or material heating. Radiation and the solid's thermal memory can remain important when buoyancy is weak.

**P.** Conditions-only scenarios require geometry/orientation, atmosphere, vector flow/gravity, ignition and initial solid thermal/moisture state. If an idealized boundary is assumed, identify it. A lunar-gravity result with no independent evidence is an extrapolation scenario, not an interpolated validated fire-safety limit.

## 7. Observation definitions and structural nonidentifiability

| Observable | What it measures / what it cannot establish alone |
|---|---|
| Visible tip/base position | Optical contour under a stated bandpass and threshold; not necessarily material conversion |
| Spread rate | Derivative of a specified front position; differentiation amplifies spatial/time calibration error |
| Burn length / damaged length | Material transformation under an explicit post-event rule; not instantaneous luminous length |
| Sample mass loss | Net escaping/entering material inventory; can include moisture and noncombusting products |
| Heat release | Chemical energy conversion inferred by a declared calorimetric method; not equal to imposed heat flux |
| Surface temperature | Instrument-dependent radiometric/contact estimate; emissivity and flame contamination matter |
| Blow-off / quench | Different physical events requiring separate criteria; loss of visibility alone is ambiguous |

**S.** The previously read [NIST SP 250-65](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication250-65.pdf) distinguishes spectral/angular sensor response and steady calibration. Its methods do not justify equating a cooled gauge signal with net heat absorbed by a hot, reacting sample.

**D/P.** A single front trajectory often constrains a combination of thermal inertia, net feedback, ignition criterion and gas chemistry. It cannot independently identify all those quantities. Many frames of one calibrated movie do not create independent experiments. A corrected optical tracker can improve measurement without changing the physics; conversely, changing a threshold to improve model agreement contaminates measurement independence.

Uncertainty must include correlated material parameters, sample heterogeneity, initial-state uncertainty, geometry, boundary heat transfer and observation response. Near extinction/reattachment, a continuous error bar around one branch can conceal a discrete model ambiguity. Missing events are censored at a documented observation horizon, not assigned an arbitrary physical duration.

## 8. Prospective checks and missing prerequisites

These checks are proposed only; no model or experiment was run:

- Recover both inert heating limits and the heat-flux convolution under their exact assumptions.
- Close condensed, pore-gas and external-gas mass/element/energy inventories, including moisture and residues.
- Test the rapid-pore-escape reduction against its balance assumptions before replacing transport by an integrated source.
- Change the dry/wet moisture reporting convention while preserving actual component masses and recover the same physical state.
- Rotate the full geometry, inlet and gravity together; preserve scalar predictions while transforming positions/velocities consistently.
- Reverse imposed flow as a new boundary problem; do not enforce an invented equality between opposed and concurrent spread.
- Separate a prescribed mass-flux boundary from predicted pyrolysis; do not score the prescribed quantity as a forecast.
- Preserve separate optical, conversion, damaged-length and mass-loss endpoints on synthetic states.

Necessary inputs remain actual material composition/morphology, reaction and char/moisture properties, temperature-dependent transport, sample/holder geometry, ignition, gas conditions and instrument definitions. Missing kinetic information can still permit a conditional inert-heating calculation. It must prevent that calculation being advertised as a complete spread/extinction prediction.

## Source/read ledger

Access date: 2026-11-14. Source bytes were fetched in memory only. No material parameter or source default was selected.

| Source | Actual extent / version | Pin |
|---|---|---|
| FDS Technical Reference Guide Volume 1 | Sixth edition, FDS 6.0.1, SVN 17529, 26 November 2013. Metadata/front matter and physical pp. 10–18; full selected solid methods physical pp. 91–97 (printed 67–73), §§7.1–7.2; pp. 98–99 (§7.3) read for companion aerosol note. This is not the current FDS manual and no current implementation equivalence is claimed. | SHA-256 `ae8152d2cfdbf547e49abfd3ee8cf803db9e44cb7f2919d256d96149a8925bb6` |
| NIST SP 250-65, May 2004 | Reuse of earlier full selected physical pp. 9–12 and 16–17: measurement principle, traceability, calibration apparatus/procedure and start of uncertainty. Not a new complete report read. | SHA-256 `418424f31476d2bb5cbd61ea67cff2d533d2137c4ca936b5c672029c265669eb` |

An attempted USDA Wood Handbook chapter-4 PDF returned HTTP 403 to direct retrieval and timed out through the web reader; no content was read or used. Moisture statements above are explicit balance/model requirements, not extracted USDA property recommendations. Mixed investigation papers remain for custodian screening. No NASA outcome evidence, model qualification, implementation, fit, commit or push is supplied.

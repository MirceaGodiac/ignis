# Astra: burning-rate emulation — distinguish physical feedback from prescribed fire

2026-11-14. Generic theory, control and metrology research for the BRE family in the nineteen-investigation programme. No numerical property/weight selection, candidate, fitted coefficient, forward implementation, measurement or score. **S** = source fact; **D** = symbolic derivation under explicit assumptions; **P** = prospective requirement/check.

## Exposure and scope

This analyst has historical aggregate exposure recorded in `astra_all_nasa_dimensional_basis.md` and is not clean-room. A generic query about porous-burner heat-flux feedback, with NASA/ACME excluded, returned snippets for a BRE apparatus paper and an ellipsoidal-model paper. They included qualitative agreement claims and mention of ground microgravity experiments. Neither paper was opened. A separate spherical-theory query returned an unsolicited ISS cohort-summary expansion; its numeric contents are not reproduced or used here. Broad flame-literature searches were stopped. New theory sources below are generic heat-flux calibration, feedback theory and conservation, not investigation outcome papers.

No further handoff, atlas data, model release, protected measurement series, validation/test outcomes or result dossiers were read. All new-investigation outcomes remain closed under the assignment. This report does not establish the actual flight BRE control algorithm, hardware constants or delivered conditions. Where it describes possible control modes, those are alternatives to resolve from screened apparatus metadata, not claims that a particular mode was implemented.

## 1. Three different experiments can share the same-looking burner

**D/P.** A porous gas burner may support at least three distinct questions:

1. **Prescribed feed:** gas mass flux is set externally; predict flame shape, thermal feedback or another independent output. Returning that input as a burning-rate forecast proves nothing.
2. **Apparent-material emulation:** measured heat feedback and known gas flow are combined into an effective material parameter. This can be a derived experimental quantity, but it is not an independent prediction of that same rate.
3. **Closed feedback:** a material energy-balance emulator uses measured/predicted heat flux to adjust gas feed. Predicting its dynamics requires the physical flame and the apparatus controller, sensor and actuator states.

Do not assume mode 3 merely from the word “emulator.” A claim about condensed-fuel burning requires an explicit mapping between the emulated fuel's mass loss and the burner gas mixture's delivered mass flux. Flame geometry and interface feedback can be modelled even if an effective material parameter cannot be identified, but the endpoint must say so.

## 2. Material and hardware energy balances

**D.** For a fixed-area, approximately isothermal condensed surface, a schematic energy budget per unit exposed area is

\[
C_A\dot T_s=q_{conv}''+q_{rad,in}''-q_{rad,out}''-q_{back}''-\dot m_s''H_g.
\]

`C_A` is areal heat capacity, and `H_g` is an enthalpy demand per mass transferred into gas under a specified reference convention. Define `H_g` so that sensible heating already represented by `C_A dot(T_s)` is not counted again. A shrinking/decomposing material may require an enthalpy formulation and multiple inventory states instead of this lumped equation. Heat of gasification, heat of combustion and thermal inertia have different meanings and units.

In a restrictive steady limit with known surface temperature, negligible storage and known back loss,

\[
\dot m_s''=q_{net}''/H_g.
\]

This is an interface balance, not a complete flame-spread or fire-growth model. The gas flame determines some of `q_net`; the material and geometry influence the gas flame. Negative `q_net` does not imply a negative physical burning rate: it means the assumed steady gasifying branch cannot satisfy that balance. The surface may cool, cease gasification or undergo a different process, requiring state evolution.

For a gas-fed porous burner, the analogous hardware balance includes feed heating and possible cooling/control power:

\[
C_{A,b}\dot T_b=q_{gas\to b}''+q_{rad\to b}''-q_{support}''-q_{cool}''
-\dot m_b''[h_{exit}(T_b)-h_{feed}]+p_{heater}''.
\]

Here `q_gas->b` denotes nonradiative gas-to-surface heat transfer and `q_rad->b` denotes net radiative exchange. There is no literal latent-heat sink merely because gas is supplied. An effective gasification demand must arise from the defined emulation mapping, an actual heat sink, or a controller that enforces the target relation. It must not be subtracted from the gas energy budget without a corresponding physical/virtual accounting statement.

**P.** Keep material state, burner hardware state and emulation mapping separate. A virtual solid should be identified as virtual; predicting the hardware's heat balance does not establish the real material's pyrolysis chemistry, melting, char growth, porosity or toxic-product yields.

## 3. Similarity conditions are stronger than matching heat release

**D.** Let the target condensed fuel have mass-loss flux `m_s''` and heat of combustion `Delta h_c,s`. Let the burner mixture contain fuel mass fraction `Y_F,b` with heat of combustion `Delta h_c,b` per fuel mass. In a complete-consumption approximation, equal heat release per surface area would require

\[
\dot m_b''Y_{F,b}\Delta h_{c,b}=\dot m_s''\Delta h_{c,s}.
\]

If the target and burner have different incomplete-consumption fractions, those must appear explicitly. Equal heat release alone does not match oxygen demand. With oxygen/fuel mass ratios `s_s` and `s_b`, simultaneous matching requires consistency of `s/Delta h_c` for the chosen fuels, or an explicitly different atmosphere/flow arrangement. That tradeoff cannot be removed by renaming the gas fuel as the target material.

Equal heat release also does not enforce equal:

- product composition or spectral radiation;
- fuel and oxidizer diffusivities and density ratios;
- momentum injection and local strain;
- surface temperature and conductive/radiative feedback;
- soot formation/oxidation or toxic-product pathways;
- solid thermal memory and geometry evolution.

For a chosen target, identify which similarity conditions matter and which remain unmatched. “Material-independent emulator” is too broad unless the observable is deliberately limited and the unmatched mechanisms are shown irrelevant within its domain.

**D.** A generalized transfer number can compare available thermal/chemical enthalpy with gasification demand, but specifying that number alone does not determine the transport geometry or boundary temperature. Two cases with the same transfer number and different radiative optical depth, confinement or surface size can have different feedback. A property-based approach should expose these missing groups rather than introduce an investigation indicator.

## 4. A feedback-emulated boundary can oscillate because of the apparatus

**S.** Åström and Murray's [Feedback Systems, second edition, Chapter 2](https://www.cds.caltech.edu/~murray/books/AM08/pdf/fbs-principles_24Jul2020.pdf) develops static and dynamic feedback separately, including transfer functions, sensitivity and positive-feedback saturation. A static input/output relation does not establish closed-loop dynamic stability. The following burner-specific reduction is derived here, not copied from an experimental controller.

**D.** Linearize about a specified steady branch. Write the flame heat-feedback perturbation as

\[
\delta q(s)=G_f(s)\delta m_b(s)+G_d(s)\delta d(s),
\]

where `d` denotes an external disturbance such as irradiation or ambient change. Let `H_s(s)` be sensor response, `H_a(s)` actuator/feed response, and let the positive material-feedback mapping be locally `delta m_b = K H_a H_s delta q`, including any delay in the blocks. Then

\[
[1-KH_a(s)H_s(s)G_f(s)]\delta q(s)=G_d(s)\delta d(s).
\]

Thus sensor/actuator dynamics participate in the characteristic equation. A separate temperature-regulation loop may have opposite sign and additional poles; it cannot be omitted simply because its steady temperature is near a setpoint. No controller gain or time constant is chosen here.

`K` contains material mapping and units; it is not a free model accuracy knob. If the mapping is `m_s''=q_net''/H_g` and burner-to-target mass conversion is known, its local derivative follows from that mapping. If material demand changes with temperature or conversion, the derivative has extra state dependence.

The physical flame has its own delay and storage through diffusion, buoyancy, residence, radiation and burner heating. An observed oscillation could be hydrodynamic, chemical, thermal, instrumental or closed-loop. A camera trace alone cannot identify the cause. A sensor that averages over time can attenuate an actual oscillation; compensating it by changing chemistry would be a structural error.

**P.** Safe apparatus metadata must state sample time, filtering, commanded and delivered feed definitions, actuator limits, sensor placement, feed-line volume, initial controller state and any shutdown logic. These are conditions or measurement-system properties. Determining them from protected output traces would be outcome access, not metadata reading.

## 5. Heat-flux sensing is an observation operator, not a surface-energy oracle

**S.** [NIST SP 250-65, May 2004](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication250-65.pdf), §2, formulates radiometer output as integration over area, direction and wavelength, and explicitly excludes temporal dependence from its steady calibration equation. Its calibration procedure includes dark signal, responsivity, coating/lead checks and maintained settings. Numerical calibration values in that report are specific to its instruments and are not imported.

**D.** A compact representation is

\[
V(t)-V_0=\mathcal H_t\left[\int_A\int_\Omega\int_\lambda
R(x,\Omega,\lambda)I_\lambda(x,\Omega,t)\cos\theta\,d\lambda\,d\Omega\,dA
\right]+V_{nonrad}(t),
\]

where `H_t` is the temporal response and `V_nonrad` represents other contributions if the instrument is sensitive to them. A total heat-flux sensor may have both convective and radiative contributions. A radiation calibration alone does not identify the convective correction at a hot burner surface.

The sensor surface and the emulated material can be at different temperatures and have different emissivity, absorptivity, roughness and local flow. Even with identical incident radiance, their net absorbed radiant fluxes differ by their surface response and emission. Convective exchange additionally depends on the local temperature difference and flow. A cooled gauge's signal therefore cannot simply be labelled the hot material's net thermal feedback.

If a documented scalar calibration is appropriate, `q_meas=(V-V_dark)/S` is a measurement conversion. Using calibration obtained independently by the measurement lane is not fitting a WSP forecast. Re-estimating `S` to make a model agree with a NASA curve would violate the separation. Unknown calibration produces an explicit missing/uncertain measurement, not a new handwritten physics factor.

**P.** Do not calibrate a prediction against heat flux that was itself computed from the prediction. If heat flux is an input, score an independently measured downstream target. If burning rate is the target, its independently measured mass/flow definition must not be reconstructed from the same assumed `H_g` used by the forecaster.

## 6. Gas-flow metrology and causal inputs

**S.** NIST's [Gas Flow Standards](https://www.nist.gov/laboratories/tools-instruments/gas-flow-standards) describes mass accumulation over time using measured pressure, temperature, volume and an equation of state. Its method is distinct from a flow-controller command.

**D.** For a documented reference volume flow,

\[
\dot m=\rho(p_{ref},T_{ref},Y)\dot V_{ref},\qquad
\dot V_{actual}=\dot m/\rho(p_{actual},T_{actual},Y).
\]

Mixing separately metered gases requires conversion on each meter's actual calibration and reference basis before summing species mass flows. A thermal meter calibrated for one gas may require a documented mixture response; the ideal-gas law alone does not supply its instrumental correction. A plenum can store gas so delivered porous-surface flux differs transiently from upstream metered flux.

Conditions-only forecasting must distinguish prescribed future control signals from future observations. Supplying a measured future feed trace can be legitimate for a conditional flame forecast, but cannot support a claim to have predicted autonomous fire growth. The cutoff and conditioning information are part of the scientific question.

## 7. Required data and honest failure modes

| Requirement | Consequence if absent |
|---|---|
| Declared emulation mode and independent target | Cannot tell prediction from replay or a derived apparent parameter |
| Burner gas composition, flow basis and delivery history | No defensible mass, momentum or heat-release boundary |
| Burner geometry and injection distribution | No justified spherical, disk, planar or axisymmetric reduction |
| Surface/support/cooling thermal properties | Heat storage and losses are confounded with gasification demand |
| Target material `H_g`, thermal state and composition with provenance | Only a virtual assumed material scenario, not a specific real material prediction |
| Gauge spectral/angular/temporal response and surface conditions | Measured signal cannot be equated to material net feedback |
| Actual controller and actuator description, if present | Hardware oscillation and branch behavior remain unresolved |
| Independent rate/heat/geometry observations and uncertainties | No experimental qualification even if algebraic balances close |

The uncertainty in a ratio `q_net/m_s''` is especially unstable near small mass-loss flux and can be strongly correlated if the same instrument/control logic constructs numerator and denominator. A narrow uncertainty on a derived apparent `H_g` need not identify material physics. Preserve covariance and censoring; never replace an unobserved ignition/extinction event by an arbitrary timeout.

## 8. Prospective outcome-free checks

None were run here; none would validate an investigation:

1. Close gas, virtual material and physical burner energy ledgers separately and jointly; every heat sink must have an identified destination.
2. Check that an imposed feed cannot be reported as an independently predicted mass-loss endpoint.
3. Use unit-consistent mass and heat-release mappings; test that changing the reference volume-flow convention preserves mass flow.
4. Remove feedback and recover the declared open-loop problem; remove sensor delay and recover only the corresponding ideal-instrument limit, not a different flame law.
5. Perturb symbolic sensor/actuator time scales in an analysis of the characteristic equation; classify this as apparatus sensitivity, not a fitted chemistry correction. Any numerical implementation requires separate authorization/specification.
6. Require a causal input cutoff and explicit initial state for every dynamic forecast.
7. Verify the sensor observation operator on synthetic radiance/heat-flux fields with stated calibration; keep such checks separate from independent experimental measurement error.
8. Treat a lost steady gasification branch as a model-state issue, not an invented negative rate or automatic detector-defined extinction time.

## Read ledger and pins

Access: 2026-11-14. Only the new research documentation is written. PDFs were fetched and text-extracted in memory; no calibration data were fitted and no numerical setting selected. The primary methods source versions below are identified by edition/date and, where obtained, SHA-256.

| Source | Actually read / fact used | Pin |
|---|---|---|
| [NIST SP 250-65](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication250-65.pdf), May 2004 | Front matter/TOC; full physical PDF pp. 9–12 (printed pp. 1–4), including §2 Measurement Principle, §3 Traceability and §4.1–4.2; full PDF pp. 16–17 (printed pp. 8–9), §5.1–5.6 and opening §6. Page-head screening pp. 1–24 also exposed generic calibration-summary text. No NASA outcomes. | `418424f31476d2bb5cbd61ea67cff2d533d2137c4ca936b5c672029c265669eb` |
| Åström & Murray, [Feedback Systems 2e, Chapter 2](https://www.cds.caltech.edu/~murray/books/AM08/pdf/fbs-principles_24Jul2020.pdf), 24 July 2020 | Full extracted physical pp. 5–8: static positive feedback and linear transfer-function derivation; web find also returned §2.1 and §2.6 portions through the saturation example. No tuning rules or numerical example values adopted. | `6124a3db6fd6a71561c98e56c792559c7794f074a5f0f85545ad6d72d076f708` |
| [NIST Gas Flow Standards](https://www.nist.gov/laboratories/tools-instruments/gas-flow-standards) | Search-returned full Summary paragraphs about accumulated mass, PVTt/RoR and equation-of-state use. Not a full hardware-report read. | Access-date locator only; no byte hash. No numerical coefficient sourced from it. |

NIST calibration results and illustrative control examples concern generic instruments/theory and are not modelled-investigation outcomes; they were not imported as forecast evidence or parameters. The result-bearing BRE and ellipsoidal-model papers remain unopened. Their snippets did not establish the flight apparatus design. No experiments, models, regressions, searches for weights, commits or pushes were performed.

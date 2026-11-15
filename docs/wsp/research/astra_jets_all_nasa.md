# Astra: gas jets, coflows and electric fields — transport, stabilization and observables

2026-11-14. Theory/methods supplement for the nineteen-investigation programme. No literal weights, property values, fitted coefficients, model implementation, experimental measurements or scores. **S** = sourced fact; **D** = independent symbolic derivation; **P** = proposed requirement/check. Algebraic factors below are derived geometry, not selected model parameters.

## Exposure and applicability

This analyst is not clean-room: historical aggregate handoff exposure and unintended publisher snippets are recorded in `astra_all_nasa_dimensional_basis.md`, `astra_spherical_all_nasa.md` and `astra_bre_all_nasa.md`. The later spherical search snippet included an ISS cohort summary even though the query excluded NASA/ISS terms. No linked outcome paper was opened; no exposed outcome numbers or trends are used here. Further reading was restricted to screened archived theory/methods sources and a versioned generic electric-field theoretical formulation. Fresh W0 authors require a separate screened package and exposure record.

No investigation-result search, atlas-data read, protected time series, per-run validation or test file was deliberately opened. All new-investigation outcomes remain closed under the instruction supplied to this analyst. Assigning Flame Design, CLD Flame, E-FIELD, SPICE or SLICE to a particular jet/coflow reduction requires safe inlet/geometry metadata; the names alone do not establish it. This document is not experimental coverage or qualification.

## 1. Specify the inlet problem before choosing a flame-length law

**D/P.** The input comprises both streams: composition with fraction basis, temperatures, pressure, mass/velocity profiles, nozzle radius/thickness, coflow extent, wall boundary, ignition and temporal forcing. A jet into quiescent ambient, equal-velocity coflow, opposed jets and a buoyant plume have different momentum and residence fields. Replacing them all by fuel volume flow discards part of the problem.

The gas core remains mass, momentum, species and energy conservation with state-dependent properties. Mixture fraction removes the chemical source only under an appropriate elemental/stream definition; representing its flux by one scalar diffusivity is an additional approximation. Preferential diffusion, extra streams, wall gasification and composition-dependent diffusivity can invalidate a one-coordinate reduction.

**S.** [Matalon Lecture 10, Summer 2013](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-10.pdf) gives the Burke–Schumann duct problem with explicit axial-flow, density, diffusion and wall assumptions. It also treats counterflow and scalar dissipation. These are distinct reductions; its numerical examples and example extinction thresholds are not adopted.

## 2. An analytical coflow benchmark with no empirical constants

**D.** Consider a deliberately restrictive benchmark: incompressible constant-density gas, constant scalar diffusivity `D`, uniform common axial velocity `U>0`, no radial velocity, negligible axial diffusion, and an unbounded transverse plane. At `z=0`, the normalized fuel-stream fraction is one inside a disk of radius `a` and zero outside. The scalar equation is

\[
U\partial_z Z=D\left[\frac1r\partial_r(r\partial_r Z)\right].
\]

With `tau=z/U`, this is transverse diffusion from a disk. Its solution can be expressed without fitted coefficients as a heat-kernel convolution,

\[
Z(\mathbf r,z)=\int_{|\mathbf r_0|<a}
\frac{\exp[-|\mathbf r-\mathbf r_0|^2/(4D\tau)]}{4\pi D\tau}\,d^2\mathbf r_0.
\]

On the axis, radial integration gives

\[
Z(0,z)=1-\exp\left[-\frac{Ua^2}{4Dz}\right].
\]

In a fast-reaction-sheet approximation, its axis intersection is

\[
L_f=\frac{Ua^2}{4D[-\ln(1-Z_{st})]}
=\frac{Q}{4\pi D[-\ln(1-Z_{st})]},\qquad Q=\pi a^2U.
\]

`Q` is the actual volumetric flow of the inner disk in this benchmark. The equation is an exact scalar/geometric consequence of the specified limit, not a universal measured flame-length correlation. It excludes unequal jet/coflow velocity, density-driven acceleration, momentum entrainment, finite kinetics, radiation-induced changes, inlet development and confinement. It cannot be transferred to a free jet by merely replacing `U` with its nozzle average.

The integral of `Z` over the transverse plane is the original disk area, so `rho U integral(Z dA)` is conserved. This provides a useful independent check of a future numerical scalar solution. In a finite duct of radius `b` with impermeable scalar boundary and the same uniform velocity, the downstream scalar tends to its cross-sectional mean `a²/b²`. Depending on that mean relative to `Z_st`, the reaction sheet may close on the axis or reach a wall. An axis-closure formula is not valid for every inlet mixture/duct ratio.

No root finder, integration code or coefficient table is implemented by this note. These expressions are mathematical reference cases for a future reviewed forward specification.

## 3. Mixing, finite chemistry and stabilization are separate problems

**D.** For a prescribed planar stagnation field `u_x=-kappa x`, with constant `D`, a passive mixture fraction satisfies

\[
D Z''+\kappa x Z'=0,\qquad
Z(-\infty)=1,\quad Z(+\infty)=0.
\]

It follows that

\[
Z(x)=\tfrac12\operatorname{erfc}\left(x\sqrt{\kappa/(2D)}\right),
\qquad
\chi(x)=2D(Z')^2=\frac{\kappa}{\pi}\exp(-\kappa x^2/D).
\]

The expression depends on this strain convention; some texts define a strain with an extra geometrical factor. Exporting a quoted critical scalar dissipation without its mixture definition, strain convention and chemistry is therefore meaningless. A local `chi_st` helps compare mixing with reaction, but its threshold is not determined by dimensional analysis.

A flame base is an edge joining premixed and diffusion structures in an inhomogeneous mixture. Its stationary position involves flow relative to the edge, propagation in the local mixture/enthalpy field, curvature/strain and heat loss. Symbolically, a local normal interface speed can be written `V_n,lab=u dot n+S_n`, with the sign convention stated; stabilization requires that speed to vanish. A downstream scalar contour does not by itself determine `S_n`.

**S.** [Matalon Lecture 11, Summer 2013](https://cefrc.princeton.edu/document/251) distinguishes an inner wake from a near-tip region requiring fuller flow equations and discusses thermal-expansion effects. Its full six-page text was returned; the first page includes an illustrative terrestrial flame photograph/reference, not a modelled NASA series. No empirical condition or numerical plot value is used here.

**P.** Keep endpoints separate: reaction-sheet geometry, visible length, lift-off, blow-off, wall contact, chemical extinction and smoke point. A steady analytical scalar field can support a geometry benchmark while remaining incapable of predicting several of these events. Extinction is a kinetic/thermal branch problem; blow-off also involves flame stabilization and advection. Do not invent an event from failure of a geometry formula.

## 4. Gravity, confinement and time scales

**D.** The inlet Reynolds and Péclet numbers do not determine downstream buoyancy everywhere. Flame heating changes density and velocity, and the relevant length grows along the jet. Use a local or explicitly defined global buoyancy comparison, retaining gravity direction. At zero/weak imposed flow, an inlet-based Richardson number can be singular or misleading; a diffusion-based scale remains available.

Fuel and coflow velocities are separately needed: changing both while holding their difference fixed does not preserve residence time. Changing nozzle size at fixed fuel flow changes injection velocity, surface loss and radial mixing even when one ideal length scale cancels size. Changing chamber walls changes heat exchange and scalar boundary conditions, so geometrical similarity must include confinement ratios.

For transient forcing, compare input time scales with convection, radial diffusion, chemistry, wall/burner response and detector response. A time-averaged inlet does not in general predict time-averaged nonlinear extinction/soot behaviour. If future measured flow is supplied, the forecast is conditional on that flow, not a prediction of the actuator or a spontaneous fire trajectory.

## 5. Electric fields require charge and energy accounting

**S.** Sánchez-Sanz, Murphy and Fernandez-Pello's [versioned theoretical formulation, arXiv:2202.03736v1](https://arxiv.org/pdf/2202.03736v1), §2, uses weakly ionized transport with electromigration and an internally generated electrostatic field. Its problem is a constant-property, lean **premixed** planar flame, not a general nonpremixed jet. Its reaction constants, simplifying mobility relations and numerical examples are not transferred.

**D.** In a dilute charged gas, a schematic charged-species number flux is

\[
\mathbf\Gamma_i=n_i\mathbf u-D_i\nabla n_i+z_i b_i n_i\mathbf E,
\qquad
\partial_t n_i+\nabla\cdot\mathbf\Gamma_i=S_i,
\]

where `b_i>0` is defined per unit charge number under this convention. Multicomponent, thermal-diffusion and electron-energy effects may require additional terms. Close the electrostatic field with

\[
\mathbf E=-\nabla\phi,\qquad
\nabla\cdot(\epsilon\mathbf E)=\rho_e=e\sum_i z_i n_i.
\]

The source mechanism must conserve charge: `sum(z_i S_i)=0` unless an explicitly modelled external source/boundary supplies it. Electrode boundaries determine injection, collection and possible surface loss. Electrons carry little mass but can carry substantial current; a neutral mass-fraction normalization does not make their electrical role negligible.

A voltage difference divided by electrode spacing is a field only in an appropriate nearly uniform, negligible-space-charge geometry. Actual flame-space charge and electrode shapes can change the field. Imposing uniform `E` and separately solving for significant space charge can be inconsistent. Quasineutrality in a bulk region does not automatically apply to electrode boundary regions.

The gas momentum equation includes an electrical force density, often `rho_e E` in a dilute constant-permittivity approximation. Total electrical work uses the laboratory-frame current `J_lab dot E`; bulk mechanical work is `rho_e u dot E`. If a thermal energy equation separately includes mechanical energy, the remaining conductive/diffusive electrical work must be defined consistently. Adding the full electrical work twice, once as gas heating and again as mechanical energy subsequently dissipated, violates conservation. Variable permittivity or magnetically significant operation requires a more complete electromagnetic stress/energy treatment.

**D/P.** An “ionic wind” model that only advects heat with an empirical voltage-scaled speed is not a closed charge-transport model. Conversely, a premixed constant-density charged-chemistry model that omits momentum response cannot establish jet deflection. Identify which mechanisms are retained: charged-species redistribution, mechanical forcing, electrical heating, electron-driven chemistry and boundary collection. The distinction must remain visible in conditions-only forecasts.

Useful dimensionless coordinates are

| Quantity | Meaning / limit |
|---|---|
| `Pe_E,i=b_i |z_i E| L/D_i` | Drift relative to diffusion for each carrier; sign/direction remain explicit |
| `N_E=|rho_e E|L/(rho U_*²)` | Electrical force versus inertial scale; unknown charge cannot be replaced by voltage |
| `t_drift=L/(b_i |z_i E|)` | Carrier crossing scale; undefined at zero field, where the drift term vanishes |
| `t_eps=epsilon/sigma_el` | Charge-relaxation scale under an appropriate conductive closure |
| `omega t_drift`, `omega t_eps`, `omega t_flow` | AC response can depend on different carrier and flow memories |
| `E/N_gas` | Reduced field relevant to nonequilibrium electron energy; it is not specified by voltage alone |
| `lambda_D/L` | Screening-scale diagnostic under stated carrier-temperature assumptions; not a license to ignore boundaries |

The Einstein relation between mobility and diffusion assumes an appropriate near-equilibrium carrier population and conventions. It should not be imposed automatically on nonthermal electrons. Reversing voltage need not preserve a flame when electrodes, carrier mobilities, surfaces or ionization zones are asymmetric. Zero applied voltage can still permit an internal ambipolar field; the neutral-model limit requires more than setting an external scalar to zero.

## 6. Observation operators and limits of inference

**D.** A camera integrates line-of-sight emission/attenuation, optical blur, spectral response and finite exposure. A spectrally selected radical-emission boundary is not generally the soot-continuum boundary. Reaction-sheet height derived from `Z_st` cannot be scored against a luminous endpoint without an explicitly justified mapping. Finite exposure can merge moving fronts and change the apparent lift-off position.

**S.** [Hanson's quantitative emission/absorption lecture](https://cefrc.princeton.edu/document/162), slides 1–6, supplies the distinction between emission plus absorption and the high-background approximation used for Beer–Lambert attenuation. A transparent-gas optical operator is a limiting assumption, not an inherent property of every flame.

**D/P.** A photomultiplier integrates over its field of view and bandpass; a radiometer has angular/spectral/temporal response; electrical current includes charge collection and potentially displacement current under changing voltage. To compare measured and predicted electrode current, specify the electrode, sign, integration surface and whether displacement current is included. A current signal alone cannot uniquely identify spatial charge density or heat-release changes.

Soot mass, optical extinction and visible length are different targets. Predicting smoke requires formation, growth, oxidation and transport (or clearly conditioned soot inputs), plus optical properties if the target is a detector signal. Neither fuel carbon content nor reaction-sheet length alone supplies a soot yield. A declared smoke-point observation also requires its instrument and decision rule; a generic threshold selected by an author is not a measurement definition.

## 7. Physical-property and apparatus requirements

| Block | Required information |
|---|---|
| Neutral gas | Full compositions, equation of state, heat capacity/enthalpy, viscosity, conductivity, binary/multicomponent diffusion and applicable reaction mechanism |
| Feed / flow | Both inlet profiles or a justified reduction, standard/actual flow basis, calibration gas, temperature, pressure, start-up history and wall conditions |
| Radiation / particles | Spectral gas properties; soot model/domain or conditioned particle state; wall temperatures and view geometry |
| Electrical | Electrode geometry/material/potentials, waveform, circuit/current limits, charge species and reaction source, carrier mobilities/diffusivities, permittivity, electron-energy assumptions |
| Initial state | Ignition, established branch, burner/wall temperature and any accumulated charge/material history |
| Measurement | Contour/band definition, spatial/time calibration, PSF/exposure, detector response, current convention and censoring |

A missing electrical block cannot be repaired by assigning an electric-field response weight. A missing inlet profile can sometimes support an explicitly assumed ideal coflow scenario, but not an assertion that an archived test has been reproduced. Unknown soot chemistry should not prevent a neutral mixing calculation, but must prevent labelling that calculation a smoke forecast.

## 8. Prospective outcome-free checks

Not executed; these are verification requirements only:

- Recover the disk-source coflow scalar solution and its cross-sectional conservation in its exact assumptions.
- Recover the stagnation scalar solution with the declared strain convention; distinguish it from an empirically chosen extinction criterion.
- Preserve species/element conservation and common thermophysical outputs across jet, sphere and droplet reductions at identical state.
- Rotate the complete geometry, inlet velocities, gravity and electric field together and preserve scalar predictions; changing only one vector is a new physical problem.
- Check charge continuity and electrode-current balance, including field storage for AC operation.
- Check that zero applied voltage does not incorrectly erase internal charge-separation physics; separately test the consistently neutral limit.
- Close electrical, mechanical, chemical, radiative and sensible-energy bookkeeping without double counting.
- Forward-project synthetic emitting/absorbing fields through instruments; keep optical and reaction-defined endpoints separate.
- Refuse unavailable branches/targets with specific missing physics instead of returning a generic duration for every investigation.

Independent withheld-group evidence remains required for every scientific qualification claim. No outcome-free analytical or conservation check supplies that evidence.

## Source/read ledger and version pins

Access date: 2026-11-14. Selected PDFs were fetched into memory; no literature data files were written. Mathematical text extraction contained glyph/sign errors, so no numerical expression was imported into a candidate. Symbolic benchmark equations above were derived from explicitly stated equations and boundaries.

| Source | Actually read / scope | Pin |
|---|---|---|
| [Matalon Lecture 10](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-10.pdf), Summer 2013 | Page-head screening all 29 pages; full extracted pp. 3–14, 17–20, 24–29. Pp. 15–16 partially returned. Jet theory pp. 10–13 and mixture-fraction pp. 25–29 support the distinctions; no example coefficients selected. | SHA-256 `3f139912cd10d7671b701a8fe5e8c4513446f35c9c1c6c1473d5ae01118fa67a` |
| [Matalon Lecture 11](https://cefrc.princeton.edu/document/251), Summer 2013 | Full six-page extracted text returned by web; generic near-tip/inner-wake theory and illustrative plots. No plot calibration or NASA series read. | Archived edition/URI; byte hash not obtained. No numerical input taken. |
| [Sánchez-Sanz et al., arXiv:2202.03736v1](https://arxiv.org/pdf/2202.03736v1) | Abstract and introduction from HTML; full §2 Formulation, physical PDF pp. 5–10, including its asymptotic boundary discussion and opening line of §3. Initial page-head screening through p. 13 also exposed generic theoretical-result sentences; no NASA outcomes. Result figures and parameter sweeps were not opened or used. | SHA-256 `647b5fecaf1c8ea52577a15e02a35c20621395ef3e40aebb4f36bda5696db0f7`; arXiv v1, 2022-02-08 |
| [Hanson Lecture 5](https://cefrc.princeton.edu/document/162), archived CEFRC 2013 | Full text slides 1–6; additional returned excerpts of later spectroscopy slides not used for this note. | Archived URI/date; no byte hash. No spectral coefficient selected. |

The originally requested CEFRC lecture-10 URL exceeded the web reader's size limit and its document route returned 403 to direct requests; the publicly indexed direct PDF path succeeded and supplied the hash above. A separate larger Pitsch lecture PDF also exceeded the web reader's size limit and was not read. These retrieval limitations are not filled by claiming full-paper coverage. No weight patch, forward, fit, regression, experiment, score, commit or push was produced.

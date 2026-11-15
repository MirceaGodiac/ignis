# Astra: dimensional basis for the nineteen-investigation programme

Date: 2026-11-14. Role: independent theory/methods analyst. Scope: preparatory analysis for `ASTRA_FIRST_PRINCIPLES_ALL_NASA.md`; no implementation, property-value selection, literal weight patch, fitting, numerical search, measurements or evaluation. This is an architectural physics argument, not a qualified model or evidence that any new investigation is covered.

## Exposure and authority record

No linked result dossiers, experimental-results papers for the nineteen investigations, per-run outcomes, measurement series, images, old model releases, `holdout/`, sealed validation directories, or final-test artifacts were opened. The assignment states that atlas split files do not yet exist; this report does not independently inspect that directory and treats every new-investigation outcome as closed. The owner prompt itself contains archive counts and some prior feasibility statements; those are instructions/context, not independent scientific evidence used to choose a law here. Broad methods searches returned unrelated aerosol-study abstract snippets; no such empirical result is used as a model premise. The DeCarlo theoretical-paper landing page was read only as an abstract, including its mention of literature applications; the paper and applications were not opened.

The existing five-model checkpoint is not the nineteen-investigation programme. Historical development exposure does not qualify a new family, new fuel, new apparatus or new gravity. This file changes only its own path, makes no claim about code currently implementing its proposals, and records no measured performance.

Labels below: **S** = directly supported by a named source and the read extent in the ledger; **D** = derivation or physical reasoning made here; **P** = proposed modelling/qualification rule. Equations are symbolic laws and limiting arguments, not authored model weights. Algebraic integers and geometric factors are not fitted coefficients.

## 1. Smallest defensible common core

**D/P.** The common object is a reacting, multicomponent, multiphase transport problem with specified boundaries and an observation operator. It is not a function from investigation name to burn time. A useful decomposition is:

`conditions + material state + apparatus boundaries -> physical state evolution -> observable -> instrument response`.

The first arrow must conserve material and energy. The last arrow must describe what a camera, radiometer, mass-flow instrument or aerosol counter actually observes. Geometry changes the differential operators and boundaries; it does not license a new gas heat capacity or diffusion coefficient.

**S.** Species balances, enthalpy transport, species diffusion and boundary conditions appear explicitly in the [Cantera governing-equation reference](https://cantera.org/3.1/reference/onedim/governing-equations.html). Its one-dimensional similarity geometry is a particular reduction, not a universal combustion model. The [FDS governing-equation chapter](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Equation_Chapter.tex) supplies another primary implementation reference, including positivity, species normalization and corrected diffusive fluxes. Neither implementation's default closures are adopted here.

**D.** In a resolved gas domain, excluding phase interfaces, a minimal symbolic skeleton is

\[
\partial_t\rho+\nabla\cdot(\rho\mathbf u)=0,
\qquad
\partial_t(\rho Y_k)+\nabla\cdot(\rho\mathbf uY_k+\mathbf j_k)=W_k\dot\omega_k,
\]

\[
\rho D_t\mathbf u=-\nabla p+\nabla\cdot\boldsymbol\tau+\rho\mathbf g+\rho_e\mathbf E,
\]

with the electric term present only in an appropriate charged-gas model. Close this with an equation of state, an energy balance, chemical production rates, transport constitutive laws and boundary/interface balances. For low-Mach gases the density can still vary strongly through temperature and composition: low Mach is not constant density. The equation of state can begin with an ideal-gas approximation only inside its declared pressure/temperature domain.

For energy accounting, choose either total species enthalpies, including formation enthalpies, or sensible enthalpy plus an explicit chemical source. Do not add a heat-of-combustion source to an enthalpy formulation already accounting for chemical formation energy. Gas/liquid/solid transfer terms must cancel in the combined system. Radiation removes energy from one location and deposits it elsewhere or escapes the boundary; it is not automatically a net loss of the whole enclosure.

Required constraints are `sum(Y)=1`, nonnegative concentrations and particle populations, `sum(j_k)=0` in the mass-averaged frame, elemental conservation in reactions, consistent energy reference states and physically valid phase states. Charge conservation is an additional constraint when charged species are retained. These requirements define the reusable core more sharply than a universal empirical oxygen exponent.

**S.** [Cantera transport documentation](https://cantera.org/3.1/reference/transport/index.html) distinguishes species transport parameters from the mixture transport model. Unity Lewis number is an explicit approximation, not a material identity. [Reaction-rate documentation](https://cantera.org/3.1/reference/kinetics/reaction-rates.html) includes collider efficiencies and pressure-dependent reactions. Thus changing a diluent can change reaction pathways as well as density, heat capacity and diffusion.

**P.** Reuse five constitutive blocks: thermochemistry; multicomponent transport; radiation; condensed-phase/interface transfer; particle population/transport. Add charge transport when the apparatus requires it. Build a few geometrical reductions around these blocks. Do not build a whole CFD platform before a concrete target needs it; use each reduction only within assumptions that can be inspected from conditions.

## 2. Dimensionless coordinates without singular zero-gravity or zero-flow limits

**D.** Choose an explicit characteristic length `L`, reference diffusivity `D`, density `rho`, temperature `T*`, and temperature difference `DeltaT`. Their definitions are part of the model: nozzle radius, initial droplet radius, material thickness and enclosure length are not interchangeable. Use diffusion scales `t_D=L²/D` and `U_D=D/L` when imposed flow can vanish. Define `nu=mu/rho` and `alpha=k/(rho cp)` consistently at a declared state. The following are coordinates and regime diagnostics, not an assertion that a scalar function of them is known.

| Mechanism | Dimensionless quantity | Why it matters / caveat |
|---|---|---|
| Imposed flow | `Pe=U L/D`; `Re=U L/nu` | Signed velocity vectors/orientation are retained in addition to magnitudes; `Pe=Re Sc` |
| Transport ratios | `Sc=nu/D`; `Pr=nu/alpha`; `Le=alpha/D=Sc/Pr` | A multicomponent gas has multiple relevant diffusivities; one effective `Le` requires a reduction argument |
| Reaction versus diffusion | `Da_D=L²/(D tau_chem)` | A chemical timescale is state- and pathway-dependent; cool and hot chemistry need not share one clock |
| Local scalar mixing | `chi=2 D |grad Z|²`; `chi tau_chem` | Defined for a chosen mixture fraction and scalar diffusivity; extinction thresholds are not universal constants |
| Buoyancy versus diffusive inertia | `G_D=|g| (Delta rho/rho) L³/D²` | Includes composition-driven density differences; retain gravity direction and time history |
| Conventional buoyancy groups | `Gr=G_D/Sc²`; `Ri=G_D/Pe²` when `Pe != 0` | `Ri` is inappropriate as the sole feature at zero imposed flow; for weak thermal density change, `Delta rho/rho ~ beta DeltaT` |
| Surface radiation/conduction | `R_s=epsilon sigma (T*⁴-T_wall⁴) L/(k DeltaT)` | State the surface, view geometry and radiation-temperature convention; this is a flux ratio |
| Participating-gas radiation | `tau_opt=kappa L`; `R_v=4 kappa_P sigma (T*⁴-T_rad⁴) L²/(k DeltaT)` | `R_v` is an optically-thin gray scaling only; spectral transport is needed outside that approximation |
| Volatility | `p_sat(T_s)/p`; `B_M=(Y_F,s-Y_F,infty)/(1-Y_F,s)` | A mixture needs component activities/composition evolution; boiling point alone is insufficient |
| Thermal phase-change demand | `Ste=cp DeltaT/L_v` | `cp` and latent heat must refer to a stated phase/path; define separately for pyrolysis |
| Condensed thermal response | `Fo_s=alpha_s t/delta_s²`; `Bi_s=h delta_s/k_s` | Thermal thinness/thickness depends on time and both boundary conditions, not only material name |
| Interface injection | `Pe_b=m_dot/(A rho) * L/D` | Burner feed, droplet Stefan flow and pyrolysis flow share mass transfer but not boundary conditions |
| Geometry/confinement | `wall_distance/L`, aspect ratios, blockage, curvature `L/R_c` | Large and small apparatus are not similar if these change |
| Surface deformation | `Bo=Delta rho |g| L²/gamma`; `We=rho U² L/gamma` | Required if droplets or liquid interfaces cease to be spherical; independent from thermal buoyancy |
| Electric drift | `Pe_E=b_i |E| L/D_i` | `b_i` is ion mobility; sign of charge, electrode geometry and field polarity remain explicit |
| Electric body force | `N_E=rho_e E* L/(rho U_D²)` | Requires charge density, not voltage alone; this ratio cannot close charge production |
| Particle transport | `Kn=lambda/r_p`; `St=tau_p U/L`; `Pe_p=U L/D_B` | Particle shape, slip correction and definition of radius are part of the model |
| Particle evolution | `Co=K_coll N t_res`; `S_set=v_set t_res/H` | Collision, residence and settling compete; `N` is number concentration, not smoke mass |
| Unsteadiness | `t/t_D`, forcing frequency `omega t_D`, ignition pulse duration `/t_D` | Initial heat/radical states and apparatus thermal memory survive after nominal ignition |

`R_v` is written only to expose dimensions: a gray optically-thin cooling density scales as absorption coefficient times radiative flux, whereas conduction density scales as `k DeltaT/L²`. It does not justify using one absorption coefficient for every atmosphere. The [FDS radiation chapter](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Radiation_Chapter.tex) explicitly distinguishes spectral absorption/emission/scattering from band and gray reductions (**S**).

**D.** These are not all independent. For example, carrying `Pe`, `Re`, `Sc`, `Pr` and `Le` as independent learned inputs repeats the same information. A minimal basis is selected after specifying geometry and retained terms. Conversely, dropping pressure because it appears in density loses independent kinetic, diffusion and phase-equilibrium effects. Equal oxygen partial pressure does not imply equal oxygen mass fraction, density, diffusivity or third-body chemistry.

**D.** Stoichiometry supplies a useful genuinely shared coordinate. For a fuel molecule `C_a H_b O_c`, complete oxidation requires `a+b/4-c/2` molecules of oxygen per fuel molecule, provided that oxidation pathway is meaningful. The oxygen/fuel mass ratio follows from molecular masses. This identity does not supply a reaction rate, incomplete-combustion yield, soot yield or a pyrolysis product distribution. For two specified inlet streams a mixture-fraction stoichiometric surface can be constructed from that balance; fuel and oxidizer stream definitions remain part of the conditions.

## 3. What each family adds

### 3.1 Droplets: a moving inventory coupled to a reacting gas

**D.** Consider an isolated spherical pure-liquid droplet with uniform liquid density, quasi-steady constant-property gas transport, negligible ambient flow, and equilibrium vapor mass fraction at the surface. Integrating radial diffusion with Stefan outflow gives the *nonreacting vapor-transfer limit*

\[
\dot m_{evap}=2\pi d\rho_g D\ln(1+B_M),\qquad
m_l=\pi\rho_l d^3/6,
\]

and hence

\[
-\frac{d(d^2)}{dt}=\frac{8\rho_gD}{\rho_l}\ln(1+B_M).
\]

The constant `d²` slope follows only if the right-hand side is effectively constant. It is a derived limiting check, not a burning-law calibration. A burning droplet has a reactive boundary-value problem for heat and species; substituting an arbitrary ambient-temperature vapor-pressure term for that problem is unjustified. Liquid heating, transient diffusion, changing mixture composition, radiation, support conduction and motion can all destroy the constant slope without violating conservation.

A schematic lumped liquid budget is `m_l cp_l dT_l/dt = Q_gas + Q_rad + Q_support - m_dot_evap L_v`, with enthalpy conventions and changing composition treated consistently. Spatial liquid temperature/composition must replace the lumped assumption when internal transport is slow. Flame radius, soot-shell radius and liquid radius are separate state/observation quantities. A dark image is not proof of extinguished chemistry.

**S.** [FDS liquid-droplet heating and evaporation](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Particle_Chapter.tex) separates mass transfer, temperature, vapor fraction, latent heat and gas/liquid exchange, under an explicit pure-liquid/lumped-temperature assumption. Its sprinkler/spray closures are not transferred to isolated burning droplets here.

**P.** Share the gas and phase-property blocks with other families; retain droplet inventory, liquid temperature, liquid composition and drift as family state. A cool-flame-capable claim additionally requires low-temperature chemistry/radical state or an independently justified reduction. Matching volatility and heat capacity across fuels does not establish such chemistry. Unknown support geometry, gas history or composition should yield conditional scenarios or an identifiability refusal for the affected target.

### 3.2 Spherical burner flames: fixed geometry with imposed or controlled supply

**D.** A porous spherical burner can share spherical gas transport with a droplet but has a different boundary: fixed radius and prescribed/controlled feed instead of a shrinking liquid inventory. In the zero-advection, constant-diffusivity, steady scalar limit, a normalized conserved scalar with `Z(a)=1`, `Z(infinity)=0` has `Z=a/r`. An infinitely thin reaction sheet then gives `r_f/a=1/Z_st`. This is a limiting geometry/stoichiometry check, not a forecast of a real burner: finite injection introduces Stefan advection, finite-rate chemistry changes the sheet, and confinement replaces the infinity boundary.

Normal and inverse flames require swapping complete inlet compositions and preserving transport/thermochemistry, not switching an investigation indicator. Flow-meter standard conditions must be converted to mass flow or actual volumetric flow with their reference state. Burner heat capacity, conductive support, warm-up and ignition history can alter the boundary independently of chemistry. A flame-radius trajectory cannot uniquely reveal burner temperature, radiation loss and kinetic rate simultaneously.

**P.** Use the same molecular diffusion/enthalpy/radiation functions as for droplets and jets. Apparatus surface temperature and feed control are separate inputs or explicitly uncertain states. Do not force a droplet `d²` law onto a growing spherical flame; the conserved inventories are different.

### 3.3 Jet and coflow flames: entrainment, residence and chemistry

**D.** For a laminar diffusion-dominated jet of radius `a`, radial mixing time scales as `a²/D`. Advecting that time at a representative axial speed gives a length scale `U a²/D`, equivalently volumetric feed divided by diffusivity up to geometry and stoichiometric dependence. This identifies a possible asymptotic scaling, not a universal coefficient or a formula valid for buoyant, turbulent, lifted or strongly radiating flames.

The necessary inlet description includes fuel and coflow composition, their velocities or mass fluxes, velocity profiles, temperatures, nozzle geometry, downstream confinement and ignition. Lift-off depends on stabilization and propagation as well as mixing. Soot formation and oxidation depend on residence and composition history; equal flame length does not imply equal smoke production. Strain, curvature, finite-rate chemistry and preferential diffusion must be checked before carrying a spherical-burner closure into a jet.

**P.** CLD-type and other burner investigations may share this geometric reduction only after condition metadata establishes the relevant flow class. E-FIELD requires the same neutral core plus charge production, charged-species mobility, electrode boundary conditions and electric forces. Voltage cannot be an empirical multiplier on length.

**S.** The primary [Cantera IonFlow implementation, v3.1.0](https://raw.githubusercontent.com/Cantera/cantera/v3.1.0/src/oneD/IonFlow.cpp), explicitly stores species charges, obtains mobilities and adds electric drift to charged-species fluxes. This confirms which physical inputs are distinct; it is not a recommendation to use that solver or its numerical settings as WSP.

**D.** Electrostatic closure normally requires `div(epsilon E)=rho_e` and `E=-grad(phi)` with electrode potentials. Charge creation/recombination and current continuity are not implied by neutral thermophysical properties. A zero-field limit should recover the neutral model when charged-chemistry effects are consistently negligible; reversing field polarity need not leave a flame unchanged because electrode geometry and mobilities can be asymmetric.

### 3.4 Burning-rate emulation: identify the controlled boundary before the target

**D/P.** A burning-rate emulator is not a separate universal reaction law. Distinguish prescribed fuel flux, prescribed thermal feedback, and a feedback-controlled boundary whose flux responds to heating. If measured mass flow is the experimental input, returning that same input is not prediction of burning rate. If feedback closes the boundary, the control law, sensor delay, thermal mass and saturation limits belong to the apparatus model. For a condensed fuel, a symbolic surface budget can relate net feedback to gasification demand, but its effective heat of gasification is material- and state-dependent.

**S.** [FDS Solid Phase, “Specified Heat Release Rate”](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Solid_Chapter.tex) explicitly distinguishes prescribing a fire's heat release from predicting it. This distinction is essential for preventing an input-fed emulator from masquerading as an independent fire-growth forecast.

### 3.5 Solids: moving fronts generated by conjugate heating and decomposition

**D.** The material needs a heat equation coupled to mass conversion, not a fuel-name multiplier:

\[
\rho_s c_s\partial_tT_s=\nabla\cdot(k_s\nabla T_s)-\sum_r\dot m_r'''\Delta h_r+q_{abs}'''.
\]

Here positive `Delta h_r` denotes endothermic conversion; the sign convention is explicit. At the interface, gas feedback, surface emission, conduction, sensible storage and gasification energy must balance. Volatile products become boundary species fluxes for the shared gas core. Char, moisture, melting, shrinkage, porosity and anisotropy are additional physical states when present, not nuisance offsets.

For a thermally thin sheet heated over a preheat length `ell`, energy per newly heated area scales as `rho_s delta_s H_req`. A moving front then has the scaling `V_sp ~ q_net'' ell/(rho_s delta_s H_req)`. Both `q_net''` and `ell` must come from a justified coupled flame/preheat model; choosing them from an observed spread rate would be circular. For a thermally thick solid under an idealized constant heat flux, a conduction time scale instead behaves as `t_heat ~ k_s rho_s cp_s (DeltaT/q'')²`. These different balances explain why thickness cannot be absorbed into one global spread coefficient.

**S.** The [FDS solid-conduction and radiation boundary sections](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Solid_Chapter.tex) treat layered materials, reaction sources, front/back boundaries and possible radiation penetration. The one-dimensional approximation in that implementation is a modelling choice, not proof that lateral conduction is negligible in a spreading front.

**P.** BASS/BASS-II and Saffire may use the same interface/gas physics while differing in material state, size, confinement and heat recirculation. A visible flame tip, pyrolysis front, char front and mass-regression surface are distinct targets. Without calibrated lengths, one may forecast dimensionless position in a stated specimen scale; one may not label it metres or a metric spread speed.

### 3.6 Smoke, aerosols and dust: a population, transport and instrument problem

**D.** Define a number distribution `n(v,x,t)` per gas volume and particle-volume interval. It evolves through advection/drift, Brownian diffusion, coagulation gain/loss, condensational size growth, formation and removal. The moments `M_j=integral(v^j n dv)` have different meanings: `M_0` is number concentration and `M_1` is condensed volume fraction. Pure coagulation reduces number while conserving condensed volume; condensation changes condensed volume as well. Thus smoke mass alone does not identify number, size distribution or optical response.

For isolated spherical particles in the creeping-flow regime, balance buoyant weight against drag to obtain `v_set ~ (rho_p-rho_g) g d_p² C_c/(18 mu)`, with slip correction `C_c`. This is a limit, not a rule for arbitrary aggregates or high particle Reynolds numbers. Brownian motion and thermophoresis persist at zero gravity. Microgravity removes gravitational settling in the ideal limit; it does not make every aerosol stationary or remove wall losses.

**S.** The [FDS aerosol chapter](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Aerosol_Chapter.tex) separates deposition mechanisms, gravitational drift, agglomeration kernels and condensation. Its empirical coefficients are not imported here. The [DeCarlo et al. theoretical-paper abstract](https://espo.nasa.gov/attrex/content/Particle_Morphology_and_Density_Characterization_by_Combined_Mobility_and_Aerodynamic_0) distinguishes mobility and aerodynamic equivalent diameters and their dependence on density and shape; the full paper was not read.

**P.** Separate formation from downstream transport. A transport-only model supplied with an initial particle distribution can be meaningful without predicting combustion aerosol yield, but must say that yield/distribution are inputs. SAME/SAME-R/DAFT coverage cannot be claimed by returning a generic combustion duration. The relevant observable requires particle-size definition, instrument transfer function, sampling line losses, dilution, residence time, humidity, composition and, when relevant, charge distribution. Dust resuspension requires an initial deposited inventory and adhesion/forcing information; a smoke-nucleation law cannot silently substitute for it.

## 4. Invariances and deliberately non-invariant cases

**D/P.** These are outcome-free consistency requirements. Passing them establishes only internal consistency.

1. **Units and labels:** changing SI display units, investigation name, test ID or species ordering while preserving the physical state cannot change the physical prediction. Species permutation must permute every coupled property and reaction entry consistently.
2. **Coordinates:** translating the complete apparatus, or rotating all vectors, gravity, fields and boundaries together, preserves scalar observables and transforms vector observables covariantly. Rotating the specimen while leaving gravity fixed changes the problem.
3. **Reference frame:** translating gas and apparatus velocities together preserves their relative dynamics in a compatible inertial description. Translating gas alone relative to a fixed wall changes transport.
4. **Time origin:** shifting an explicitly recorded origin must shift output timestamps without changing elapsed dynamics. Dropping the ignition/heating history does change state.
5. **Energy references:** changing a consistent species enthalpy reference cannot change a physical heat balance. Mixing reference conventions between liquid, gas and reaction tables can.
6. **Geometry and scale:** geometrically similar cases have equal normalized responses only if all retained dimensionless groups, boundaries, material states and observation definitions match. Radiation, kinetics, wall distance and finite liquid/solid response can break a naive size law.
7. **Zero mechanisms:** removing reaction removes chemical heat release; removing gravity removes the buoyancy/settling terms; an adiabatic interface has no imposed heat leak. A reaction disabled in the model is not a physical extinction prediction.
8. **Conservation:** gas/condensed exchange balances and coagulation volume conservation must survive discretization within declared numerical error. Clipping a negative inventory must not create missing mass silently.

Useful cross-family metamorphic cases include identical gas mixtures at identical state returning identical transport properties, and identical optical path/composition/temperature returning identical spectral absorption. Equal total fuel feed through different geometries need not yield equal flame length, radius, heat feedback or smoke.

## 5. Which numbers are shared, and what disagreement means

**P.** Share physical constants, molecular identities, reaction stoichiometry, state-dependent thermochemical functions, compatible transport models and applicable optical properties. Share a kinetic mechanism only over its justified chemistry/state domain. Preserve the source and validity interval of every imported parameter. Do not share nozzle discharge, burner heat leak, camera threshold, fabric porosity or particle sticking efficiency merely because they are dimensionless.

**S.** [Cantera thermodynamic models](https://cantera.org/3.1/reference/thermo/species-thermo.html) distinguish standard/reference states and temperature parameterizations. [NIST WebBook's guide](https://webbook.nist.gov/chemistry/guide/) identifies separate gas, condensed-phase, phase-change and reaction data and their conditions. [NASA's thermodynamic-library record](https://ntrs.nasa.gov/citations/20020085330) explicitly describes coefficient generation by least squares. Transcribing those coefficients is use of an externally constructed property representation, not a claim that an LLM derived or invented the property values.

**P.** The property block needs: chemical/material identity and composition; phase; temperature/pressure/composition domain; unit and reference convention; value or function and uncertainty; original source locator; correlation family; provenance and shared-source dependencies; extrapolation policy. A trade name alone cannot specify a fabric, foam, coating or polymer formulation. A mixture needs component fractions and a mixing/phase-equilibrium model, not averages of pure-fuel boiling points. A source's validity range is not automatically the checked range of the IGNIS release.

Two handbooks repeating the same original determination are not independent cross-checks. [NIST-JANAF's background page](https://janaf.nist.gov/background.html) describes its website as a historical archive; it should not be described as the latest determination by default. This report checks source structure only: it does not verify any existing project property table or certify any numerical entry against a second source.

**D/P.** If two apparatus require incompatible apparent diffusion constants, first inspect composition, reference state, length convention, heat loss, support effects and measurement definitions. The discrepancy can expose missing physics, calibration error or an invalid reduction. It is not evidence that one molecule has an investigation-dependent transport constant. If apparently shared chemistry fails under a new pressure or diluent, investigate pathway/collider changes before adjusting a global oxygen exponent.

## 6. Gravity and conditions-only scenarios

**D.** Gravity enters momentum, hydrostatic boundaries, interface deformation, particle settling and residence histories. It can indirectly alter heat feedback, reactant supply, radiative exposure and losses. These effects compete. No conservation law makes flame spread, extinction time, smoke yield or hazard monotone in gravity. Therefore endpoints at microgravity and normal gravity do **not**, by first principles alone, bound all intermediate-gravity outcomes. The owner prompt's endpoint-bounding statement is treated as an unverified attribution, not adopted as a theorem or calibration fact.

At imposed-flow domination and small `Ri`, gravity may be a perturbation, but the smallness of `Ri` requires a meaningful nonzero flow scale. Near zero flow, use the diffusion-based buoyancy group and actual enclosure scales. Near extinction, even a small perturbation can change branch or event existence. Constant lunar gravity and a time-varying residual-acceleration history are different forcings. Orientation relative to the flame, substrate, inlet and electrodes must be retained.

**P.** A gravity scenario must preserve all other specified conditions, name the assumed acceleration vector/history, and identify the geometrical and constitutive validity envelope. A partial-gravity forecast with no relevant withheld measurements is an unvalidated prediction, even if gravity lies numerically between tested endpoints. Do not market it as a safety bound. A sealed prediction should freeze the conditions, output observable, horizon, release, uncertainty semantics, exclusion rule and evaluation plan before relevant outcomes become available.

**P.** A conditions-only request conceptually contains atmosphere composition with fraction basis, pressure, temperature, humidity; fuel/material properties and state; geometry and boundaries; flow and control history; gravity/field vectors; ignition/initial state; requested observable and horizon. Values that cannot be obtained are represented as missing or as explicitly varied scenario assumptions. This is a design proposal only; no shared contract is changed.

Return separate statuses for: mathematically accepted; physical assumptions applicable; property domain covered; experimentally checked domain; apparatus/measurement comparability; and gravity extrapolation. A single in-range flag cannot distinguish them. Being inside every marginal property interval is insufficient if the combination, geometry, pressure-dependent chemistry or thermal boundary has never been checked.

## 7. Honest uncertainty without fitting

**D/P.** Track at least five distinct sources: condition/instrument uncertainty; material-property uncertainty and correlated source errors; unknown initial or boundary state; reduction/model discrepancy; numerical approximation. Repeatability variation is another component when evidenced. An author's invented probability distribution is not empirical uncertainty calibration.

**S.** [NIST's uncertainty propagation guide](https://physics.nist.gov/cuu/Uncertainty/combination.html) describes first-order propagation through a measurement function including covariance. It is a local approximation, not a validation procedure.

**D.** For a differentiable map, `Cov(y) approximately J Cov(x) J^T` propagates input uncertainty conditional on the map. Shared thermochemical sources can correlate errors across species and experiments; treating them as independent artificially shrinks uncertainty. Near ignition, extinction, melting, detector threshold or branch switching, local linear propagation may be misleading. Distinct admissible closures and initial-state possibilities should remain visibly distinct scenarios if no evidence supplies probabilities.

An unseen fuel can have small measured `cp` uncertainty but large uncertain kinetic reduction or soot yield. An unseen solid can have bounded density and thermal conductivity while its decomposition/char behaviour remains unspecified. An unseen gravity can be precisely known yet have large model discrepancy. There is no honest universal percentage to attach to these cases. Report conditional ranges or explicitly unquantified discrepancy; avoid calibrated-coverage language.

**P.** Image measurement uncertainty belongs to the blind measuring pipeline: dimensional calibration, projection, pixel/edge ambiguity, exposure, saturation, occlusion, timing and observable definition. The modelling analyst must not tune a tracker to predicted curves. Correlated calibration error across all frames does not vanish when many frames are sampled. Finite differencing a noisy front amplifies error; uncertainty in spread velocity cannot be inherited unchanged from position. Repeated blind measurements assess measurement agreement, not forecast validity.

An event not observed before recording ends is censored, not an event at the final frame. A flame becoming invisible is a radiometric/optical event unless independent evidence supports a chemical-extinction definition. A conditions-only request lacking ignition state may support a conditional burning branch without supporting ignition probability.

## 8. Coverage cannot be acquired by relabelling

**P.** The following is a *provisional target/interface allocation based on the owner prompt*, not an apparatus audit or a statement of achieved coverage. Each named investigation remains a separate denominator entry even when it shares a release. All new-investigation outcome access stays closed until its authorized group freeze; this analyst performs no scoring.

| Investigation | Candidate physical family | Nontrivial target and decisive additional information |
|---|---|---|
| FLEX, PSI-69 | Droplet | Liquid diameter trajectory / stated extinction observable; calibration, liquid/gas/ignition state |
| FLEX-2, PSI-68 | Droplet | Same target class; composition and dimensional image measurement must exist |
| CFI, PSI-39 | Droplet with appropriate chemistry | Liquid/flame trajectories or combustion-stage event; mixture and chemical-state assumptions |
| ACME CFI-G, PSI-159 | Spherical burner, subject to metadata | Flame geometry / stated event; feed, burner temperature and normal/inverse streams |
| Flame Design, PSI-10 | Jet/coflow, subject to metadata | Shape or smoke-related observable; geometry, inlet profiles and soot closure if claimed |
| s-Flame, PSI-23 | Spherical burner, subject to metadata | Shape / radiometric observable; burner boundary and optical definition |
| BRE, PSI-20 | Controlled burning boundary | Feedback-dependent rate only if independent of prescribed feed; control law and thermal budget |
| CLD Flame, PSI-21 | Gas-flame reduction to be established | A declared geometry/stability observable; inlet configuration must decide the reduction |
| E-FIELD Flames, PSI-22 | Gas flame plus charge transport | Field response; electrode geometry, polarity, mobility and charge mechanism |
| SPICE, PSI-107 | Duct gas flame | Declared shape/stability/smoke endpoint; duct flow and instrument definition |
| SLICE, PSI-106 | Duct gas flame | Declared lift-off/shape/stability endpoint; actual inlet and coflow configuration |
| BASS, PSI-26 | Solid/gas interface | Calibrated front trajectory or specimen-normalized observable; material and frame scale |
| BASS-II, PSI-25 | Solid/gas interface | Same observable class across actual specimen properties; thickness/porosity/char status |
| Saffire-I, PSI-98 | Solid/conjugate enclosure | Front/heat-feedback related target; enclosure flow, geometry and material state |
| Saffire-II, PSI-99 | Solid/conjugate enclosure | Declared spread/flammability observable; gravity/orientation and boundary conditions |
| Saffire-III, PSI-100 | Solid/conjugate enclosure | Declared spread/growth observable; scale and recirculating heat/flow effects |
| SAME, PSI-102 | Particle population/instrument | Actual reported size moments or detector response; size definition and transfer function |
| SAME-R, PSI-101 | Particle population/instrument | Same observable class only after method comparison; sampling and aging history |
| DAFT / DAFT-2, PSI-47 | Dust/aerosol transport/instrument | Actual particle/detector observable; initial inventory, charge and instrument calibration |

The property route is necessary but not sufficient for acceptance. Count separately: listed; metadata usable; target observable defined; blind measurement method characterized; properties sufficiently sourced; release executable; protocol frozen; independent groups scored; qualified within a stated domain. Publish exclusions and failure counts. Missing measurable targets should be labelled unscorable with the specific missing measurement/calibration; returning a number does not complete coverage.

Transfer evidence should distinguish a new run of the same apparatus, a withheld session, a new fuel, a different apparatus and a new gravity. Frames from one run are not independent test cases. Leave-fuel/apparatus-out claims require a split that actually withholds that factor and an author exposure history consistent with the claim. Historical validation used repeatedly for choice is development evidence, not a new untouched final test.

## 9. What can still honestly be called hand-written

**P.** A large property table can be manually transcribed and source-audited, but its scientific origin remains published measurement/theory/correlation. It is not thousands of independent discoveries by the author. A measured dataset belongs to measurement provenance, not to literal weights. Thousands of hand-typed corrections chosen after seeing outcomes remain data-informed W1 construction.

A deterministic forward integrating fixed physical equations can be distinguished from optimizing weights or inferring latent parameters against an observed trace. But the governing IGNIS literal-patch rules and `ModelSpec` decide admissibility: symbolic derivations in this memo do not authorize new solvers, generated coefficients, hidden regressions, external fitted mechanism files or an unreviewed construction path. Numerical accuracy settings also affect predictions and require identity/provenance. This report authors none of them.

The strongest publishable aspiration is a preregistered, property-driven family model that transfers across a genuinely withheld apparatus/fuel under a shared constitutive core, with separate blind measurements and explicit negative results. The present report supports only the choice of physical structure and the identification of missing information. It is not a claim of NASA validation, all-NASA coverage, WSP superiority or a habitat fire-safety assessment.

## 10. Source/read ledger

All external sources were accessed on 2026-11-14. Class A here means fundamental equations or primary scientific/software equation documentation; class B means property/method documentation. It does not mean an experimental qualification grade. Read extents are sections/excerpts actually returned, not a claim of full-paper reading. Mutable FDS `master` pages are source locators, not frozen release inputs; freeze a commit or archived source before numerical construction. No named-investigation outcome paper was deliberately searched or opened.

| ID / class | Source and exact extent read | Fact used and limit |
|---|---|---|
| S1 / A | [Cantera 3.1, Governing Equations](https://cantera.org/3.1/reference/onedim/governing-equations.html): Axisymmetric Stagnation Flow, equations/definitions; Diffusive Fluxes; Inlet, Outlet and Symmetry boundary excerpts (web lines 71-175, in separate returned excerpts) | Coupled balances; conservative flux correction; Soret as a separate term; boundary dependence. No numerical examples or validation read. |
| S2 / A-B | [Cantera 3.1, Species Thermodynamic Models](https://cantera.org/3.1/reference/thermo/species-thermo.html): introduction, Models for Species Standard State Enthalpy/Entropy/Heat Capacity, NASA7 and opening NASA9 discussion (lines 71-122) | Reference states, integrated thermochemical consistency and distinct parameterization forms. No species coefficient table read. |
| S3 / B | [NIST Chemistry WebBook Guide](https://webbook.nist.gov/chemistry/guide/): introduction/search/identity excerpts; Function Plots; General Information; Gas/Condensed Thermochemistry; Phase Change; Reaction Thermochemistry (lines 4-122 and 269-349 in returned excerpts); adjacent ion-data headings also visible | Property identity, conditions, domain/source metadata and separation of phase/reaction data. No project fuel values selected. |
| S4 / A | [FDS Equation Chapter](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Equation_Chapter.tex): opening model overview; parts of grid discussion; Mass and Species Transport and realizability (lines 106-146); returned energy paragraph around lines 152-168 | A primary implemented conservation/positivity reference; no adoption of LES/eddy-dissipation defaults. Not full chapter. |
| S5 / A | [FDS Radiation Chapter](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Radiation_Chapter.tex): Radiation Transport Equation and Radiation Source Term, lines 4-49; beginning of spectral-band table visible | Spectral absorption/emission/scattering and limits of gray reduction. No NASA outcomes. No coefficient selected. |
| S6 / B | [NIST Combining Uncertainty Components](https://physics.nist.gov/cuu/Uncertainty/combination.html): complete page body, lines 19-48 | First-order sensitivity/covariance propagation and distributional qualifications. Formula image itself was not inspected; displayed text/special cases were. |
| S7 / A | [Cantera 3.1 Reaction Rates](https://cantera.org/3.1/reference/kinetics/reaction-rates.html): Elementary Reactions, Three-Body Reactions, Collider-specific parameterizations, opening Pressure-dependent Reactions (lines 71-126) | Kinetics requires more than a thermal-property table; collider effects. No mechanism coefficients or experimental comparisons. |
| S8 / A-B | [FDS Particle Chapter](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Particle_Chapter.tex): Particle Transport opening (lines 4-47); Heating and Evaporation equations/definitions (160-173); adjacent spray-distribution lines visible | Explicit liquid mass/thermal/gas coupling and assumptions. Spray constants/distributions not used. |
| S9 / A | [FDS Solid Chapter](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Solid_Chapter.tex): conduction and front/back radiation boundaries (4-72); material-mixture excerpt (185-203); Pyrolysis opening, Specified Heat Release Rate and opening of Scaling Burning Rate by Heat Flux (206-223) | Conjugate solid budget; prescribed fire differs from predicted fire. No cone-calorimeter data or fitted scaling imported. |
| S10 / A-B | [Cantera 3.1 Transport](https://cantera.org/3.1/reference/transport/index.html): complete substantive body, lines 71-103 | Species/phase model separation and explicit transport approximations. No transport coefficients selected. |
| S11 / B, abstract only | [DeCarlo et al., 2004, Part 1: Theory, NASA-hosted landing page](https://espo.nasa.gov/attrex/content/Particle_Morphology_and_Density_Characterization_by_Combined_Mobility_and_Aerodynamic_0): citation and abstract, lines 46-54 | Equivalent particle diameters depend on morphology/density. Full paper and literature applications not read or relied on. |
| S12 / A, code | [Cantera v3.1.0 IonFlow.cpp](https://raw.githubusercontent.com/Cantera/cantera/v3.1.0/src/oneD/IonFlow.cpp): returned lines 0-156, including constructor, transport update, frozen-ion stage and electricFieldMethod beginning | Charges/mobilities/electric drift are explicit physical inputs. Numerical bounds, frozen-ion staging and polynomial routines not adopted. No claim of a full solver audit. |
| S13 / A-B | [FDS Aerosol Chapter](https://raw.githubusercontent.com/firemodels/fds/master/Manuals/FDS_Technical_Reference_Guide/Aerosol_Chapter.tex): full text returned by find, lines 4-224; especially Deposition, Gravitational Settling, Thermophoresis, Agglomeration and Condensation | Separate particle mechanisms and population balance. Empirical coefficients and default nucleation assumptions not imported. |
| S14 / B, metadata only | [McBride, Zehe and Gordon, NASA/TP-2002-211556](https://ntrs.nasa.gov/citations/20020085330): NTRS abstract/metadata, lines 15-76 | Thermodynamic library has externally least-squares-generated coefficients; source provenance must say so. PDF and property tables not read. |
| S15 / B | [NIST-JANAF Background](https://janaf.nist.gov/background.html): complete eight-line page | Historical archive status, not automatic current-property authority. No tabulated values read. |

Search-only/unused items: NIST FDS version-2 publication abstract; transport API summaries; general aerosol-metrology search snippets; NIST SI-temperature navigation page; JANAF search metadata. A guessed FDS `Soot_Chapter.tex` URL returned 404 and supplied no evidence. No inference depends on that failed retrieval. This ledger does not claim to have reviewed apparatus manuals for all nineteen investigations; the table above is explicitly provisional pending safe methods/conditions metadata.

## 11. Immediate handoff to the main author

1. Freeze the few-family reduction assumptions, symbolic balances, observable definitions and source-screening rules before fresh agents write numbers.
2. Ask the atlas/measurement lanes for condition-only apparatus and property requirements, followed by blind measurement-method cards. Keep their protected outputs inaccessible to numerical authors.
3. Put shared thermochemistry/transport/radiation identity ahead of fuel-name expansion; require separate evidence for kinetics, soot, pyrolysis and particle formation.
4. Treat the invariance and limiting-law checks above as verification only. They do not close the missing independent experimental evidence for any new investigation.
5. Register transferable scientific claims narrowly enough to fail: same physical coefficients, stated withheld factor, independently measured target and explicit exclusions. Preserve failures rather than replacing the denominator with accepted runs.

Only this documentation file was written. No Rust/web code, contracts, existing forwards, releases, datasets, shared records or protocol was modified; no model or evaluation was run, and no commit/push was performed.

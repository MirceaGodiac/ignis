# Astra property droplet W0: scientific specification and origin ledger

Status: fresh numerical author, zero-shot W0, unexposed to experimental outcomes,
other candidates or development feedback. This is an experimental scientific
kernel, not a qualified NASA model. The coordinator disclosed prior exposure;
that structural coordination is separate from this author's numerical exposure.

## Domain and observable contract (written before implementation)

The primary output is **liquid equivalent-volume diameter**, from component
inventories and additive incompressible liquid volumes. It is neither a luminous
diameter nor a flame diameter. Chemical state, ignition, cool flames, extinction,
soot, visibility, support effects and measurement calibration are unavailable.
Names identify components only; no name selects a property or coefficient.

The transient branch describes an ideal, well-mixed liquid solution with uniform
temperature, additive volumes and no heat of mixing. It exchanges components
with an ideal, nonreacting gas containing one insoluble carrier. All volatile
species share a constant mass-based diffusivity; rho*D is constant in the gas
film. Each component has constant liquid/vapor heat capacities, liquid density,
and a supplied saturation-pressure/latent-heat reference. All approximations
have explicit temperature/pressure domains. No absent property is imputed.

There are two transport choices. The spherical reference requires zero gravity
and zero imposed velocity. Supplied conductances permit nonzero gravity and
velocity only inside the supplier's explicit radius, composition, thermodynamic
and environment envelope. This second choice is a conditional film closure,
**not a prediction of natural convection or shape under gravity**. Sphericity
and a well-mixed liquid remain assumptions. No empirical Nusselt/Sherwood or
deformation threshold is invented. A finite nonzero gravity is never silently
treated as zero.

Two additional analytic outputs are reference limits: an externally thermostatted
pure-liquid d-squared law, and an existing pure-fuel Burke-Schumann reaction
sheet with unity Lewis number, infinite chemistry rate, constant properties,
zero gravity/velocity, no radiation and prescribed surface temperature. The
reaction-sheet radius is a chemical interface, not an optical signal. Neither
reference establishes that its assumed state actually occurs.

## Independently derived balances

For component i, liquid mass m_i, molar mass M_i and density rho_li,

    x_i = (m_i/M_i) / sum_j(m_j/M_j)
    V = sum_i(m_i/rho_li), r = (3 V/(4 pi))^(1/3), d = 2 r.

Use one declared enthalpy reference temperature T0:

    h_li(T) = cp_li (T-T0)
    h_vi(T) = L_i0 + cp_vi (T-T0)
    L_i(T) = L_i0 + (cp_vi-cp_li)(T-T0).

Thus the latent heat and heat capacities cannot contradict each other. Positive
latent heat is required throughout the component's declared domain. Integrating
the ideal-vapor Clausius-Clapeyron equation, neglecting liquid specific volume,
gives the fixed law

    log(p_sat_i(T)/p_sat_i(T0)) = M_i/R *
      [(L_i0-dcp_i*T0)*(1/T0-1/T) + dcp_i*log(T/T0)].

For ideal liquid equilibrium, X_si = x_i p_sat_i/P. The remaining surface gas
mole fraction belongs to the insoluble carrier. Convert these mole fractions
to mass fractions using the supplied molar masses; do not confuse liquid mole,
gas mole and gas mass fractions. Sum(X_si)>=1 is outside the sub-boiling carrier
film model and is refused, not clipped. Nonideal liquid activity coefficients,
gas fugacity, Kelvin/Poynting corrections and critical behavior are absent.

Let S_s and S_inf be summed volatile gas mass fractions. Zero net carrier flux
and equal species diffusivities yield

    z = log((1-S_inf)/(1-S_s))
    C_m = 4 pi r (rho D)                 [kg/s, spherical branch]
    mdot_i = C_m [b(z)(Y_si-Y_inf_i) + z Y_si]
    b(z) = z/expm1(z), b(0)=1.

This stable form follows by integrating the steady species conservation law.
It supports evaporation, condensation and finite counterdiffusion with zero
total mass flux. The sum is C_m z; no balancing species is silently modified.
The actual sum of component rates is used for subsequent bookkeeping, and its
residual against C_m z is checked against the caller's arithmetic tolerance.

The sensible gas-energy equation contains the constant outward heat-capacity
flow C_adv = sum_i(mdot_i cp_vi); carrier has zero net mass flow. Integrating it
with constant gas conductivity gives

    C_h = 4 pi r k                       [W/K, spherical branch]
    Pe_h = C_adv/C_h
    Q_gas = C_h (T_inf-T) b(Pe_h).

This retains species enthalpy transport under the declared constant-cp,
equal-diffusivity closure. The same formula with supplied positive C_m and C_h
defines the conditional effective-film branch, not an exact 3D solution.
Separately supplied external heat Q_ext is a signed, provenance-bearing input.
It is not inferred from flame labels and must not duplicate Q_gas.

Conservation, with outward mdot positive, is

    dm_i/dt = -mdot_i
    H_l = sum_i m_i cp_li (T-T0)
    dH_l/dt = Q_gas + Q_ext - sum_i mdot_i h_vi(T).

The explicit step updates m_i and H_l, then reconstructs T from the updated
heat capacity. This conserves the stated discrete flux budget rather than
updating temperature with an inconsistent old mass. Negative masses, depletion
of the whole liquid, nonpositive temperature and property-domain crossings
reject the step. The caller must provide a smaller timestep; there is no hidden
clipping, automatic normalization, optimizer or timestep search. Requested time
increments and arithmetic tolerances are numerical inputs, never material
parameters. Time is elapsed seconds since the explicit initial time origin.

For a single thermostatted component at constant conditions, z is constant:

    d(d^2)/dt = -8 rhoD z/rho_l.

The exact endpoint returns the evaporation time when the liquid is exhausted;
this is liquid depletion, not chemical extinction. Thermostat heat required to
maintain T is mdot L(T)-Q_gas-Q_ext; it is reported explicitly. A condensing
reference increases d^2 and has no evaporation time.

For the distinct flame-sheet reference, with q the positive complete-reaction
heat per kg fuel, s the positive kg oxidizer/kg fuel, Y_O the far-field oxidizer
mass fraction and L the positive latent heat at prescribed T_s:

    B_c = [cp_g(T_inf-T_s) + q Y_O/s]/L
    K = 8 k/(rho_l cp_g) log(1+B_c)
    mdot = 4 pi r k/cp_g log(1+B_c)
    r_f/r = log(1+B_c)/log(1+Y_O/s)
    T_f = T_inf + [q-L-cp_g(T_inf-T_s)]/[cp_g(1+s/Y_O)].

Require B_c>0, Y_O>0, r_f>r, T_f>T_s, T_f>T_inf and domain-valid temperatures.
Failure means no admissible sheet in this reference, not a universal extinction
decision. Pure fuel, no ambient fuel, single-step stoichiometry and a preexisting
sheet are explicit assertions. Chemical energy is entered once as q; formation
enthalpies are not another input to this branch.

## Numerical-origin ledger

| Item | Origin | Role |
|---|---|---|
| pi = 3.141592653589793 | mathematical identity, nearest binary64 | handwritten scalar tensor; sphere geometry only |
| 0, 1 | algebraic identities / exact boundary definitions | zero flux, conservation and exact domains |
| 2 | exact diameter/radius relation | geometric conversion |
| 3, 4 | sphere volume/area integration | V=4*pi*r^3/3 |
| 8 = 2*4 | derivative of diameter squared with mass conservation | analytic d-squared reference |
| exponent 3, cube root | geometric dimension | radius-volume conversion |
| exponent/log/exp/expm1 | fixed-law evaluation | no fitted series or polynomial coefficients |
| explicit Euler coefficient 1 | first-order finite-volume time rule | caller-specified dt; no learned integration parameters |
| every dimensional property, R, reference T and saturation pressure | explicit source-bearing physical input | no embedded fuel tables or default numbers |
| heat input, initial masses/T, ambient composition/T/P, g and velocity | explicit source-bearing state/scenario input | no inference from unavailable measurements |
| supplied conductance and its envelope | external determination or declared scenario assumption | conditional nonzero-gravity/flow branch only |
| comparison tolerances and timestep | caller-declared numerical budget | checks/reproducibility, not physical qualification |
| numbers inside unit tests | synthetic_fixture | analytic invariants only, never property estimates |

No fitted/generated coefficients, seeds, random initialization, optimizers,
parameter searches, data-driven interpolation or previous releases are used.
The patch contains a literal assignment only. Pi must equal the language's
binary64 mathematical constant; changing it is not a physical model revision.

## Actual reading and limitations

Allowed local reads: CLEANROOM_DROPLET_W0.md in full; physical_core.rs (algebra,
types and transport helpers; initial full read was output-truncated); wsp.rs
spec/patch/exposure semantics; ignis-wsp/Cargo.toml. The coordinator supplied
only outcome-free Family/Quantity/Origin wire tags and Digest canonicalization.
No other local scientific/model/data file was opened. No experimental,
development, validation, holdout or test outcome was accessed.

Primary sources actually inspected on 2026-11-14:

- [Matalon CEFRC 2013 Lecture 12](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-12.pdf),
  extracted pp.3-9: assumptions, interface mass/energy, heating, ideal equilibrium,
  pure vaporization and Burke-Schumann sheet formulas. No claim to have read the
  entire lecture. These motivate the conditional reference domain; the mixture
  and conservative-step derivations above are this author's extensions.
- [Cantera 3.1 species thermodynamics](https://cantera.org/3.1/reference/thermo/species-thermo.html),
  standard/reference-state conventions and cp-integral enthalpy relations.
  No species coefficients copied.
- [Cantera 3.1 phase thermodynamics](https://cantera.org/3.1/reference/thermo/phase-thermo.html),
  introductory ideal-gas and phase-model descriptions only.
- [Cantera 3.1 governing equations](https://cantera.org/3.1/reference/onedim/governing-equations.html),
  energy/species equations and diffusive-flux conservation, lines 73-144.
- [IDAES 2.8 activity conventions](https://idaes-pse.readthedocs.io/en/2.8.0/reference_guides/model_libraries/generic/property_models/activity_coefficient.html),
  VLE fugacity relation and ideal-liquid activity convention, lines 225-259.
  Its empirical parameters and phase-transition smoothing were not adopted.

This kernel does not claim combustion-mixture prediction, differential gas
diffusion, internal liquid gradients, preferential surface segregation,
nonideal thermodynamics, finite-rate chemistry, soot, radiation, gravity-driven
flow, moving droplet dynamics or uncertainty calibration. Its tests establish
only synthetic algebra/conservation and domain behavior. Registration,
contract-adapter integration and release construction belong to the coordinator.

## Verification and freeze checkpoint

Standalone `rustc 1.94.1` compilation and all ten synthetic tests passed on
2026-11-14. `rustfmt --edition 2021 --check` passed; the three authored JSON
inputs parsed successfully. These checks did not run a dataset, model score,
workspace test suite, contract assembler or compatibility adapter.

The first synthetic test execution passed nine tests and rejected one fixture
whose deliberately fictitious saturation law exceeded the carrier-film boiling
boundary. Raising only that test's explicitly supplied ambient pressure into
its declared domain made it a valid enthalpy-bookkeeping case. The scientific
kernel and literal coefficient were not changed in response to that fixture
failure. The corrected full run passed ten tests. Synthetic verification is
permitted W0 work, not experimental feedback or qualification.

The passing tests cover exact equilibrium, condensation, zero-net-mass
counterdiffusion, component and enthalpy conservation, input-order/name
independence, first-order convergence to an independently integrated pure-liquid
d-squared limit, cp/latent consistency, matching/nonmatching supplied gravity
closure, unit/provenance/finite/phase/depletion failures, absent-species
condensation, and the flame-sheet reference's radius scaling and domain limits.

`configs/models/properties/droplet-w0-astra/freeze.json` records byte hashes of
the source, spec, manual ops, author envelope, research ledger and verification
receipt after these checks. No TRAIN outcome or feedback existed for this
candidate at freeze. Numerical status remains **zero-shot W0**. Any subsequent
integration must preserve this checkpoint and distinguish new adapters or
outcome-informed changes from this frozen author output.

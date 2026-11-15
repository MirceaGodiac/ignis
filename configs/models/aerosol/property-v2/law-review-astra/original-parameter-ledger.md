# Literal parameter ledger

All assignments are scalars, manually typed by GPT-6 Astra. `constants.ops` is a
private strict literal-only input; no spec hash or registry claim accompanies it.

| Scalar | Literal | Unit | Origin and interpretation |
|---|---:|---|---|
| boltzmann_j_per_k | 1.380649e-23 | J/K | Exact SI defining constant, NIST CODATA value page. |
| slip_alpha | 1.165 | 1 | Kim et al. 2005 abstract, pp.31-32, Cc law. Published empirical property approximation for PSL spheres in air. |
| slip_beta | 0.483 | 1 | Same original source and restriction. |
| slip_gamma | 0.997 | 1 | Same original source and restriction. |
| maximum_continuum_kn | 0.01 | 1 | Author-chosen conservative domain rejection threshold for no-slip approximation; not an error bound or calibration. |
| maximum_settling_reynolds | 0.1 | 1 | Author-chosen conservative domain rejection threshold for creeping drag; not an uncertainty statement. |
| maximum_rayleigh_size | 0.1 | 1 | Author-chosen conservative domain rejection threshold for both x and abs(m)*x; no calibrated optical-error guarantee. |

The published slip approximation requires an explicit independent applicability
statement bound to the supplied composition, phase, temperature, pressure and
diameter domain. Its use on smoke is an unqualified extrapolation unless that
statement supplies actual support. The kernel never manufactures this statement.

Exact algebraic coefficients in code: 0/1 for presence and identities; 2 for
diameter/radius, distinct-pair counting and two reactants; 3 for sphere volume,
Stokes drag and geometric mean-distance algebra; 4/6/8/18 and pi from sphere
geometry, Maxwell thermal speed, Stokes balance and Rayleigh theory. The binary
self-collision factor is 1/2. No empirical coefficient is hidden in code.

Numerical controls are explicit arguments: section upper edges in particle
volume, max time step, max fractional coagulation loss, floating-point comparison
tolerance, and max substeps as a real CPU budget. The last section is unbounded;
its mean may grow and remains dynamically active. Coarseness is disclosed.

Source yield, rate, density, gas properties, composition, wall film thickness,
line loss, flow, geometry, gravity, refractive index, wavelength, path length,
optical power, responsivity and dark voltage are supplied physical inputs with
provenance/domain/reference/unit/uncertainty. They are not estimated from the
detector validation trace. There is no material-to-soot conversion coefficient.

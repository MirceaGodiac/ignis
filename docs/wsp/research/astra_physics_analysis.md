# Astra: independent physics and uncertainty analysis

Author model: **gpt-6-astra**, explicitly selected for this analysis; date: 2026-11-14.
Scope: scientific design and algebra only, across the five families. No model weights, release,
fit, parameter search, simulation sweep, train feedback run, validation look, commit or push was
performed by this author. This is a data-informed analysis: the required dossier exposes train
summaries, aggregate development scores and its historical contamination ledger. It is not W0.

## Decision

Build a new, opt-in mass–energy cooling member for unlit droplets and a coherent persistent-condition
mixture for each burner family. Build the fabric oxygen budget as an alternative **steady** speed
closure with an explicit transient limitation. These are defensible architectural additions before
any claim that they improve forecasts. Preserve all predecessors exactly through a neutral switch.

The ambitious destination is a shared, checked property library feeding small conservative physical
state models, with distinct observation models and persistent uncertainty. A larger transport solver
is defensible only after its conserved quantities, boundary conditions and approximations are checked.
Adding equations does not identify unobserved initial conditions or establish a probability law.

| Family | First defensible addition | What it could improve | Main unresolved freedom |
|---|---|---|---|
| FLEX methanol | Coupled unlit fuel/water mass and liquid temperature; begin with explicitly declared pure-fuel approximation if water state is unavailable | Transfer across diameter and ambient conditions; separation of shrinking from cooling contraction | Temperature and water fraction at flame loss; remaining hot-gas heat input |
| FLEX n-heptane | Same conservative unlit kernel with separate hot-flame/cool-flame initial-state scenarios | A physically named alternative to empirical after-flame exponents | Cool-flame thermal history; proof that the flame is actually absent; transport-property validity |
| ACME normal | Persistent condition/apparatus scenarios inside each existing law member | Honest uncertainty from composition, flow and stopping conditions; coherent cause probabilities | Prospectively known stop/limit metadata; correlated controller errors; structural radiation error |
| ACME inverse | Same mixture machinery, retaining qualitatively different held/oscillatory/extinguishing hypotheses | Avoid false certainty near a regime boundary | Oscillation onset, re-ignition and filament influence are not identified by nominal conditions alone |
| BASS | Oxygen-budget steady closure using physical width and fuel loading; independent flame-length response | Dependence on oxygen, flow and areal loading with interpretable units | Capture-layer closure, pyrolysis state, geometry, fan calibration and visible-front observation mapping |

Expected gain here means a reason a mechanism may transfer, **not** a numerical score prediction.
The information available in this analysis does not support an honest expected CRPS/Brier reduction.

## 1. Droplet mass and energy: what is actually derived

Let `r` be liquid radius, `d=2r`, `q=d²`, `m` liquid mass, and `mdot>0` outward evaporation rate.
For a single liquid with constant density,

```text
m = π ρ_l d³ / 6
dq/dt = -4 mdot / (π ρ_l d) = -K(T_l, composition, atmosphere)
m c_l dT_l/dt = Q_g + Q_rad - mdot L
```

This energy equation already accounts for vapor leaving with its enthalpy relative to the liquid.
Differentiating `m c_l T` and subtracting `mdot L` again without the outgoing sensible-enthalpy term
double-counts part of the variable-mass energy balance. With water or temperature-dependent
properties, use species masses and consistent phase enthalpies; do not merely substitute one
effective latent heat into an otherwise inconsistent energy equation.

For constant density and evaporative cooling alone:

```text
dT_l/dt = -(3 L / (2 c_l)) K(T_l) / q.
```

This verifies the dossier's factor `1.5` and its diameter-squared thermal clock. It does **not** verify
an initial temperature, a vapor-pressure approximation, or the absence of heat from the gas.

### The reciprocal-time law is local, not universal

Assume `K(T)=K0 exp((T-T0)/Theta)`, constant `L,c_l`, and freeze `q=q0` in the temperature equation.
Then direct differentiation gives

```text
K(t) = K0 / (1 + t/t0)
t0 = 2 c_l Theta q0 / (3 L K0).
```

Thus its temperature e-fold `Theta` is the local logarithmic slope of an evaporation law, not a
universal material constant. Freezing diameter is another assumption. Keeping shrinking diameter
but retaining the same exponential law gives the independent exact check

```text
n = 3 L / (2 c_l Theta)
T-T0 = (3 L / (2 c_l)) ln(q/q0)
K = K0 (q/q0)^n
q/q0 = [1 + (n-1) K0 t/q0]^[-1/(n-1)]            (n != 1)
q/q0 = exp(-K0 t/q0)                             (n = 1).
```

The first expression is used only while its bracket and the assumed liquid/property regime remain
valid. The reciprocal-time result approximates this system when the temperature-sensitive cooling
changes the rate much faster than it changes diameter. Neither construction supplies a physical
long-time evaporation floor: ambient heat transport must enter to produce the appropriate coupled
wet-bulb state. Likewise, a fixed positive floor added to reciprocal decay is an empirical closure,
not a solution of the displayed energy balance.

### Do not mix molar and mass transport conventions

For an isothermal ideal binary gas with constant molar concentration `c=p/(R T_g)`, diffusivity `D`,
stationary inert carrier in the molar frame, and fuel mole fractions `x_s,x_inf`, integration gives

```text
mdot = 4π r M_F c D ln[(1-x_inf)/(1-x_s)]
K = 8 M_F c D / ρ_l · ln[(1-x_inf)/(1-x_s)].
```

For the distinct constant-`rho D` mass-fraction approximation,

```text
mdot = 4π r (rho D) ln[(1-Y_inf)/(1-Y_s)].
```

Both can be valid reduced descriptions under their stated approximations. Substituting ambient
density into the second while claiming the first's constant molar-density derivation is inconsistent
for unequal fuel/carrier molecular weights. Methanol is close to air in molecular weight; heptane
is not. Use one derivation and document the film convention. For nonisothermal, multicomponent gas,
neither is an exact universal transport law. The primary evaporation paper explicitly distinguishes
the constant-`rho D` assumption from isothermal gas and discusses coupled surface-temperature
determination. [Finneran, Garner and Nadal (2021)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8292777/)

### Heat-transfer correction: the hidden Lewis-number assumption

Independent integration of steady spherical constant-property advection–conduction with outward
mass flow yields

```text
Q_g = 4π r k_g (T_g-T_l) · beta/expm1(beta)
beta = mdot c_pg / (4π r k_g).
```

The zero-flow limit is ordinary spherical conduction because `beta/expm1(beta) -> 1`.
Under the mass-transfer convention above, `beta=ln(1+B_M)/Le`, where
`Le=k_g/(rho c_pg D)`. The dossier's multiplier `ln(1+B_M)/B_M` follows when `Le=1`.
It should not silently be reused in a helium-rich gas with nonunit Lewis number. A multicomponent
enthalpy-diffusion treatment needs further terms; the expression above is a declared reduced model,
not an assertion that those terms vanish in the real experiment.

### Sourced properties and their actual scope

NIST's methanol Antoine entry uses `log10(P/bar)=A-B/(T+C)` and gives one coefficient set over
288.1–356.83 K; its displayed latent-heat correlation covers 298–477 K. NIST's heptane Antoine sets
cover 185.29–295.60 K and 299.07–372.43 K. The intervening gap and excursions beyond a range are
not covered just because an equation returns a finite value. A continuous extrapolation may be
declared and bounded as an approximation, or a better source supplied; silently clamping temperature
changes the physical law. [Methanol](https://webbook.nist.gov/cgi/cbook.cgi?ID=C67561&Mask=4),
[heptane](https://webbook.nist.gov/cgi/cbook.cgi?ID=C142825&Mask=4)

Using those externally published correlation coefficients verbatim is allowed source transcription
under this lane's rules. The correlations were fitted by their source authors; therefore the precise
claim is **no fitting to IGNIS outcomes**, not that the property correlation has never been fitted.
Choosing among correlations or changing a film-temperature recipe after observing model error adds
author freedom even if every individual coefficient has a citation.

### Identifiability and initialization

A diameter prefix identifies a current apparent size and, if its measurement quality permits, a
local size trend. It does not uniquely identify liquid temperature, water concentration, gas-cloud
temperature and flame state. Different combinations can produce the same shrinking rate.
The physical mass equation must distinguish true diameter from tracker output; variable density
alone gives `qdot/q=(2/3)(mdot_liquid/m-rhodot/rho)`, where `mdot_liquid` is signed.
Cooling contraction can shrink the image without the same fuel consumption as evaporation.

Start the unlit member at a causal flame-loss state produced by that member or the admitted prefix.
Never reset its temperature at every requested forecast horizon. If the state is not observed,
carry alternatives whose spread remains visible in output. A cool flame is still reactive; its
evaporation must not be treated as inert merely because the visible hot flame disappeared.

The simplest useful extension can neglect water explicitly and say so. It cannot then claim to have
explained methanol–water extinction. The classical flame-sheet construction deliberately excludes
liquid heat-up, radiation and several transport effects; relaxing its assumptions is scientifically
useful only when the extra closures are exposed. [Dryer's Princeton research background](https://www.princeton.edu/~fldryer/nasa.dir/backgrnd.htm)

## 2. Fabric: conservation constrains a closure but does not supply it

Write the steady per-face oxygen budget in physical units:

```text
v m_F sigma = rho_inf Y_O U delta
delta² = nu L/U + (alpha/U)².
```

Here `m_F` is combustible areal loading per face, `sigma` the oxygen/fuel stoichiometric mass ratio,
and `delta` a hypothesized capture thickness. The integral oxygen flux across a specified control
surface is conserved; replacing that integral by `rho_inf Y_O U delta` and the displayed layer law
is the closure. Using hot-layer diffusivities and ambient density requires an explicit reference-state
argument. The profile shape and factor multiplying layer thickness are not proven to equal one by
dimensional analysis. Cancellation of width in a two-dimensional budget does not establish behavior
when side entrainment and holder losses become comparable to the width.

If `L=v tau`, let `C=rho_inf/(sigma m_F)`. Squaring gives

```text
v² = (C² Y_O² nu U tau) v + C² Y_O² alpha²
a = C² Y_O² nu U tau; b = C² Y_O² alpha²
v = (a + sqrt(a²+4b))/2.
```

This algebra is correct. The units are `a: m/s`, `b: m²/s²`.
At zero flow, the formula's limit is `sqrt(b)`; its derivative with respect to `U` is
`(da/dU)/2` there. At high flow it approaches `a+b/a`. Therefore the dossier's coefficient of `a`
is an asymptotic slope, not the exact small-flow slope of one universal straight line.
The positive zero-flow limit is a transport branch, not proof that a flame survives at zero flow.
Energy loss and finite-rate chemistry remain separate existence conditions. Older, independent
thin-cellulose microgravity work explicitly reports near-limit departures from simple thickness
scaling and a role for heat loss and finite-rate kinetics; it does not establish a SIBAL threshold.
[Olson, NASA TM-100195 (1987), abstract](https://ntrs.nasa.gov/citations/19880006471)

### The mean of tip/base velocities hides a state assumption

Let `f` be mean consumed fraction of fuel inside the visible zone of length `L=x_tip-x_base`.
Under the additional assumption that the region behind the base is fully consumed, consumed mass
per unit width is `m_F [x_base + f L]`, up to a fixed origin. Its derivative is

```text
m_F [(1-f) v_base + f v_tip + L df/dt].
```

This becomes `m_F(v_base+v_tip)/2` only for `f=1/2` and `df/dt=0`. A growing or shrinking flame need
not satisfy either. Moreover, the visible base need not coincide with complete burnout and the tip
need not be the solid pyrolysis front. The dossier's arithmetic can be right while its transient
interpretation is wrong. Consequently, reuse an admitted flame-length response as a declared closure,
or add a solid heating/burnout state with an explicit observation mapping; do not present the mean
velocity formula as exact transient conservation.

A physical width supplied by engineering metadata can convert `m/s` to sample-widths/s. A width
inferred only from an uncalibrated image cannot establish metres. Future fan history is a scenario
or a known schedule only when prospectively available; conditioning on what the operator actually
did later answers a different forecasting question.

## 3. Burner flames: mechanism uncertainty and valid competing risks

Normal and inverse flames can share properties and numerical probability machinery without sharing
one extinction law. Which reactant diffuses against the outflow changes the relevant transport and
Lewis-number effects. Ignition geometry, burner thermal loading and apparatus stops remain separate.

The older Santa et al. study supports distinguishing transient radiation loss from kinetic extinction
and shows why falling heat release can matter alongside growing radiation. Its large-flame radiation
scaling uses a shell whose volume grows approximately with area when thickness stops growing.
Therefore an isothermal hot ball radiating as `r³` is not a universal closure, and optically thin
emission is not automatically net radiative loss. These are mechanisms from a different, short-duration
experiment, not transferred CFI-G coefficients. [Santa et al. (2007)](https://www.grace.umd.edu/~pbs/2007%20Santa%20et%20al.%20CNF.pdf)

In particular, an `erfc` concentration solution for a fixed source in a constant-property diffusion
problem does not by itself make `r_f=r_ss erfc(r_f/(2 sqrt(Dt)))` an exact moving reacting-flame
solution. A moving boundary, Stefan advection, variable density/properties and ignition state require
additional assumptions. Treat the existing relation as a physically motivated closure whose limits
can be checked. Do not relabel its empirical correction as a measured diffusivity.

### Persistent scenarios, not new random conditions every step

Let `z` denote one persistent physical condition/apparatus node and `l` a structural law member.
Within each pair, calculate cause-specific hazards `lambda_k(t|z,l)` with survival

```text
S(t|z,l) = exp[-integral(sum_k lambda_k)]
F_k(t|z,l) = integral S(u-|z,l) lambda_k(u|z,l) du.
```

For weights already conditional on the admitted prefix:

```text
S_mix(t) = sum_(z,l) w_(z,l) S(t|z,l)
F_mix,k(t) = sum_(z,l) w_(z,l) F_k(t|z,l).
```

These preserve `S + sum_k F_k = 1`. Averaging hazards with fixed weights before integrating
generally does not produce this mixture. If an aggregate hazard is needed, its weights are
proportional to `w_(z,l) S(t|z,l)` among survivors and change with time.

If weights are instead priors at ignition and the prefix says the relevant state survived to cutoff
`c`, first condition them: `w(z,l|survived c) proportional to w(z,l) S(c|z,l)`.
Do not apply this twice if an existing prefix updater already conditions them. More informative
observations require their stated likelihood; survival alone cannot identify true oxygen or true flow.

For a finite step with hazards held constant, `h=sum_k lambda_k`, distribute the lost survival mass
as `Delta F_k=S_before[-expm1(-h dt)]lambda_k/h` and set `S_after=S_before exp(-h dt)`.
The `h=0` limit is zero incidence. This is a conservation-preserving time substep, not a fit.
A known scheduled stop is an atom: assign the mass surviving immediately before it to the stop cause,
then set survival to zero. It is not an independent probability added on top of extinction.

Use one node through the full trajectory and reuse it across requested horizons. A high-bias flow
controller node must affect all terms that depend on that flow, not only the desired extinction clock.
Correlated composition/flow errors need joint nodes. Independent Cartesian perturbations may create
physically impossible atmospheres, violate mixture normalization, or double-count existing spreads.

Law weights and condition weights mean different things. Two equally weighted laws are an explicit
judgment committee, not a calibrated probability that either is true. Node spreads derived from
apparatus precision represent measurement uncertainty; larger session heterogeneity and missing
mechanisms must not be hidden under the same label. This analysis selects no node widths or weights.

For inverse flames, an oscillator/re-ignition state could eventually be a substantial advance. It
requires admissible causal flame or photometric information plus an independently checked mechanism;
a critical flux measured from a transport solver with wrong temperatures is not established by
dimensional plausibility. Preserve the held scenario while that ambiguity remains. For normal
flames, a burner heat balance is preferable to an unexplained time constant only if sphere, tube,
incoming gas and radiative input are all accounted for or explicitly omitted with uncertainty.

## 4. Error budgets and freedom that parameter counts miss

The appropriate forecast target is the observable used by the evaluator. A latent physical diameter
or pyrolysis front is not interchangeable with an optical tracker or visible flame tip. Measurement
error, bias, censoring, transient process error, session variation and structural uncertainty need
separate mechanisms where the evidence permits. Random observation scatter cannot justify shrinking
session or model-discrepancy bands. Out-of-view and physically extinguished are different events.

No numerical irreducible-error floor was estimated here. It would require independent repeat
measurements or an identified observation process. Residual error against one model is not such a
floor, and a retrospective best physical trajectory is not an attainable causal forecast.

The extension map's claim that every nature/apparatus number removes a free law/uncertainty parameter
is too strong. It only does so if an earlier free coefficient is actually removed and no compensating
closure is introduced. A table can be large and scientifically fixed, but choosing its source,
temperature range, averaging rule, interpolation, mixture rule and placement after seeing outcomes
adds freedom. Counting only train-read scalar values misses all those choices and the number of
attempts. Report the source of every block **and** the author choices, exposures and rejected designs.

Allowed source transcription includes published externally determined coefficients, units, validity
ranges and reference states. A new constant solved from target train points, a chosen member of a
numerical sweep, or an optimized material coefficient is a new fit regardless of whether the author
types its decimal representation. Evaluation of an already fixed physical law in the forward is
computation; generating a weight table from that evaluation is not literal authoring under this
protocol. This report creates neither kind of table.

## 5. Implementation and independent checks before any outcome comparison

Use new modules/model names. Preserve the predecessor call path when all new switches are neutral,
including output distributions, event kinds and missing-input behavior. Additive registry/source-hash
registration is necessary, but this analysis does not edit those files. Architecture details below
were supplied by the `forward_design` collaborator and were not independently audited in source.

1. **Droplets:** explicit positive species masses and energy state; documented property domains;
   well-defined exhaustion and saturation/boiling boundaries; causal phase transitions. Do not let
   the `x_s -> 1` Stefan logarithm trigger an arbitrary rate cap disguised as physics. Check energy
   and mass residuals on synthetic cases and compare the evaporative-only branch with the analytical
   formula above. Check temporal convergence at fixed typed coefficients; timestep refinement is
   a numerical test, not a physical-parameter scan.
2. **Fabric:** unit check in metres before width conversion; exact steady quadratic identity;
   low-/high-flow limits; zero oxygen gives zero transport speed; independent declaration of survival
   and flame length. Synthetic step changes should expose rather than conceal the fixed-progress
   assumption. Do not derive a calibrated extinction boundary from the positive transport branch.
3. **Burners:** per-node and mixture probability conservation, monotone CIFs, scheduled-stop atom,
   node identity across horizons, zero-spread reduction, and neutral-switch equality. A duplicated
   identical law member must leave the forecast unchanged when its weight is split.
4. **Observation layer:** distinguish no detection from no flame; preserve missing and conflicting
   fields. Never infer another run's outcome from successive chamber-fill condition differences.

These checks test correctness of constructions. They do not qualify a model, establish unseen-session
performance or justify another validation look. Train checks should be predeclared falsification of
mechanisms; protected outcomes stay with the evaluator.

## 6. Read/exposure record and attribution

Local reads used for this analysis:

- `phys_flex_methanol.md`: targeted lines 1–285, plus the displayed later audit, blueprint,
  validation-input-only scenarios and source/exposure sections from the initial reads. Initial
  large combined outputs were truncated, so this is not claimed as a certified full line-by-line
  read of all 469 lines. The actual derivation relied on §1, especially §1.6, not numerical raw output.
- `phys_bass.md`: lines 1–155; `phys_flex_heptane.md`: lines 1–100;
  `phys_acme_inverse.md`: lines 1–115; `phys_acme_normal.md`: lines 1–105.
  These reports themselves disclose earlier source contamination. Their equations are claims to
  assess, not independent corroboration of their train measurements or inherited thresholds.
- No original `exp_*` cohort dossier, raw series, test/holdout data, sealed validation track,
  per-run validation outcome, or evaluation continuation vault was opened by this author.

Primary web sources actually used:

| Source | Read extent | What it supports; what it does not |
|---|---|---|
| NIST methanol / heptane pages linked in §1 | Returned phase-change sections, particularly Antoine equations/units/ranges and latent-heat correlation entries; not all cited original papers | External property coefficients and domains; not complete mixture transport or droplet thermal history |
| Finneran et al. 2021 | Abstract/introduction and returned selected equation/assumption passages around HTML lines 99–102 and 190–198; not the complete paper or supplement | Need for a coupled energy problem and explicit transport assumptions; no numerical coefficient imported |
| Dryer Princeton background | Returned theory/background page, including assumption list and displayed continuation; no linked experimental archive | Scope of classical spherical-droplet theory |
| Olson 1987 NTRS record | Metadata and complete abstract; search also displayed excerpts of the same old report | Qualitative mechanism caution for thin fuels; no SIBAL threshold imported |
| Santa et al. 2007 CNF | Publisher abstract; author-hosted PDF extracted text displayed for abstract, introductory scaling and selected later passages; not a complete page-by-page read | Radiation/heat-release mechanism and geometry caution in an independent older experiment |

Search results also displayed short excerpts for other old flame-spread sources, a 2023 radiative
extinction article, and statistical competing-risk articles. No modelled-series per-test result was
used, no coefficient was taken from those snippets, and those sources are not treated as read papers.
The probability identities in §3 and the closed-form corrections in §§1–2 are this author's algebra,
not claims that those searches verified an implemented algorithm.

Collaboration: `physics_research` supplied clean theory/property observations; I independently opened
the NIST and selected primary sources used above. `forward_design` supplied architecture constraints.
`audit_methanol` and the parent disclosed that an exploratory alias-based raw-file read could have
matched a non-train FLEX file. **All raw measurements/output from that attempt are excluded from
this analysis.** I requested only analytic findings, did not receive/use that raw output, and do not
certify the other agent's raw access as train-only. This boundary needs to remain in the shared
exposure ledger; it cannot be undone by excluding a number from a report.

No new author-authored model weight exists in this file. Any later candidate using this report must
record its actual dossier exposure and the property/closure choices it adopts, and remain experimental.

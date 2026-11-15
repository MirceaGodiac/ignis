# Astra review of the new thermal, droplet7 and front3 forwards

Reviewer: **gpt-6-astra**, explicitly selected. Date: 2026-11-14.
Scope: read-only inspection of `crates/ignis-wsp/src/thermal.rs`, `droplet7.rs` and `front3.rs`;
no model weights, configuration values, outcomes, raw experimental files, tests or predecessor
source files were read or authored in this review. This note records the source snapshot inspected;
other workers were actively editing, so findings are not automatically verdicts on later changes.

## Decision at the inspected snapshot

The new mass/energy equations and the steady oxygen-speed algebra have the correct units and
geometrical factors. The assumptions are generally unusually explicit. Before authoring active
candidates, repair the fabric first-event probabilities and enforce physical input/time coverage.
The thermal terminal-event shortcut also needs a numerical guard or an explicit refusal rather than
silently making an irreversible decision from a first-order predictor.

### 1. High priority: fabric reached-end incidence can decrease

At inspected `front3.rs` lines 933–951, `p_q` is the quench CDF and `p_end` is a monotone envelope
of a front-location probability. At lines 1402–1405, the artifact writes

```text
F_extinction = q
F_reached_end = (1-q) e
S = (1-q)(1-e).
```

Although these sum to one, they are not generally competing first-event cumulative incidences.
Once `e=1`, `F_reached_end=1-q` falls whenever subsequent quenching rises. Previously ended samples
are effectively reassigned to extinction. Enforcing monotonicity of `e` alone cannot prevent this.
The flow-path mixture preserves the same issue because it pools those path-level quantities.

**Required property:** once a path's mass has entered either terminal cause, it stays there. Build
cause-specific increments against joint survival on a fixed time grid, then mix the resulting CIFs.
The existing `p_reached_end` DTO can carry `F_reached_end/(1-F_extinction)` if necessary, provided it
is explicitly a coordinate used to reconstruct the CIF, not itself described as that CIF.

The `implement_front` collaborator proposed this repair after being notified. Their proposal converts
successive changes in a monotone end marginal into a conditional end hazard and combines it with
the quench hazard. That is probabilistically coherent as an **assumed first-passage model**; the
Gaussian endpoint probabilities are not a derivation of a physical first-passage distribution.
An endpoint mass taking the end CDF to one must be handled as an atom. Treating it as an infinite
hazard across the whole interval would erase the quench probability that accrues before that atom.
This note has not yet reviewed the repair.

### 2. Medium priority: reject atmospheres outside the composition simplex

At inspected `droplet7.rs` lines 1567–1632, the adapter reads oxygen, carbon dioxide and helium,
then checks separate feature bounds. Those bounds permit combinations whose total exceeds one.
`Gas::thermal` drops oxygen and `thermal::flux` checks only `x_CO2+x_He<=1` (line 398).
Consequently the thermal law can manufacture a residual N2/O2 carrier for an impossible supplied
atmosphere. This is a support-boundary concern independently of whether the data currently exercise it.

Reject nonfinite/negative fractions and `x_O2+x_CO2+x_He>1` at the new forward boundary, with only a
documented numerical roundoff tolerance if needed. The unspecified remainder is then an explicit
N2 assumption. This review did not inspect upstream contracts and does not assume they supply this
missing invariant. A nitrogen-only mechanism switch is not a substitute for composition validation.

### 3. Medium priority: Euler exhaustion predictor can terminate too early

At inspected `thermal.rs` lines 504–508, the integrator sets `q=q_floor` and breaks whenever
`q-h*K_start<=q_floor`, before evaluating the midpoint or cooling state. During strong cooling,
`K_start` can substantially exceed the rate later in the step. The Euler predictor crossing is then
not evidence that the coupled trajectory reaches the floor. The code can enter its absorbing state
even when a smaller-step or correctly resolved trajectory stays above it. The opposite, a rising
rate reaching the floor during a step, is also handled only by endpoint clipping later.

A safe minimum change is a scientific/numerical refusal requesting a smaller authored step when
the crossing is not resolved. Alternatively use a declared event-resolution/substep rule at fixed
physical coefficients. Numerical substepping is not parameter fitting. Check the event with a
synthetic rapidly cooling case and timestep refinement. Keeping the last resolved temperature after
an actual terminal event is acceptable if explicitly marked unresolved; it does not make an
unverified event time correct.

### 4. Medium priority: flow schedule must cover the integrated prehistory

At inspected `front3.rs` lines 635–637, a query earlier than the first flow stage returns the first
stage's value. `check` validates nonempty, nonnegative sorted entries but not coverage back to
`igniter_on_s`. The path integrates from ignition. A schedule first known later is therefore
silently extended backward into an unknown interval, and that affects both the state and prefix
innovation. Require coverage, or expose a deliberate prehistory assumption. Future controls can be
valid prospective inputs when genuinely scheduled; their values must not be applied before their
own start times merely because there is no earlier record.

## What checked correctly

- **Thermal energy factors.** For `m=π rho d³/6`, `K=-d(d²)/dt`, and
  `Q_g=4πr k(Tg-Tl)f`, division by `m c_l` gives
  `Tdot=[12 k(Tg-Tl)f/rho - 1.5 L K]/(c_l d²)`.
  This is exactly the inspected `flux` expression. Neither a missing factor of two nor a latent-heat
  double count was found in that expression.
- **Mass-transfer convention.** `K=8 M p D ln[(1-x_inf)/(1-x_s)]/(rho_l R Tfilm)` is the
  declared molar Stefan approximation. The dimensions are `m²/s`. Pressure-scaled diffusivity
  cancels the ideal-gas density pressure dependence as expected before surface equilibrium enters.
- **Stefan heat correction.** `beta=rho_l K c_pg/(8 k_g)` and `beta/expm1(beta)` follow from
  the same outward mass rate. They do not force Lewis number to one. The zero-rate limit is handled.
- **Property units.** Antoine pressure is converted from bar to Pa using `100000`; the latent
  correlation converts kJ/mol to J/kg with `1000/M`. The boundary Clausius–Clapeyron extension
  uses molar latent heat and molar `R`, with the correct sign. The inverse has the same algebra.
- **Physical root.** `front3` uses `v=a/2+hypot(a/2,A alpha)` for the positive root of
  `v²-a v-(A alpha)²=0`, with `a=A² nu tau U`. `A=faces rho Y/(fuel_areal sigma)` has units
  `1/m`; the result is `m/s`, then division by physical width yields sample-widths/s. The loading
  is explicitly total combustible loading across the exposed faces, avoiding a hidden factor two.
- **Thermal trajectory causality.** The cache keys the initial squared diameter and last-flame
  kind; cloud age runs from that transition on its fixed step grid. Repeated horizons query the
  same trajectory instead of resetting temperature. The new law overrides the separate dying-flame
  mass-loss term when selected, avoiding that double addition.
- **Unsupported carrier fallback.** With `nitrogen_only=1`, CO2/He runs retain predecessor
  thermal-related branches and diagnostics disclose the fallback. With the switch off, reciprocal
  diffusion and linear conductivity mixing are declared approximations. Oxygen is deliberately
  lumped with nitrogen; this is not a verified full multicomponent transport calculation.

## Limitations that remain choices, not algebra bugs

1. `cutoff_on=1` explicitly restarts an authored hot/unknown temperature and cloud at cutoff.
   This is causal but not thermal-state inference and need not be consistent across advancing
   cutoffs. Leaving it off is the clean isolation experiment for future flame transitions.
2. A pressure cap modifies the assigned initial temperature. It is disclosed, but the cap is a
   phenomenological initial-state choice rather than an energy-conserving transition from the lit
   predecessor. Heating at the cap can immediately hit the refusal boundary; that is preferable
   to silently clipping every subsequent state, but configuration authors must check it.
3. Constant density omits thermal contraction. Constant activity is not a methanol–water species
   balance. The cloud term prescribes gas temperature without tracking cloud energy; the coupled
   balance conserves the modeled liquid energy, not a closed liquid-plus-gas system.
4. The source-range switches make extrapolation explicit. The broader accepted `0<T<Tc` interval
   is not proof of a valid liquid phase below its melting line or of accurate near-critical
   transport. None of the declared simple mixing laws has a universal high-temperature guarantee.
5. Fabric density and transport properties are fixed at authored reference states. No pressure,
   temperature, alternate carrier or finite-width edge correction is inferred. The allowed input
   surface should not be advertised as a physically verified domain for all of those conditions.
6. The oxygen residence time closes a steady equation; the inherited overhang/burnout, ignition
   multiplier, prefix offset and quench hazard are independent closures. A physical speed member
   does not turn the whole transient front forecast into a conservative solid-fuel solver.

## Verification extent

The initial three-file read was truncated. Targeted source reads then covered the thermal block's
tensor descriptions, validation, correlations, initialization, flux, integration and cache; droplet7's
new binding/adapters, support inputs, cutoff state/filter, thermal diagnostics and transition-routing
sections; and front3's oxygen closure, width binding, path integration, support checking, run/path
mixtures, event construction and artifact adapter. Unrelated copied networks and their entire
parameter specifications were not re-audited. No source coefficient was newly looked up for this
code review; the earlier analysis contains its primary-source record. No compilation or dynamic
test was run by this reviewer. Fixes require their own observed checks before claiming resolution.

Follow-up scope extension authorized by the parent: matching `front2.rs` event excerpts were then
read to establish inheritance. The formulas match; details and the new/compatibility/joint semantics
decision are recorded in [F-007](../dossier/review/F-007-front-event-cdf-semantics.md). No predecessor
was edited. This follow-up supersedes the initial scope sentence only for that narrow source read.

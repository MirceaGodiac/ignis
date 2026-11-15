# Astra R2: droplet TRAIN losses and one shared physical correction

2026-11-15. Author: GPT-6 Astra. **DATA-INFORMED, not zero-shot.** These are proposed
literal revisions, not assembled or qualified releases. No new candidate score has
been computed or read. One candidate per fuel; all new literals were chosen once,
before the synthetic checks below. The correction concerns a physical species
assignment, not an estimated residual correction. It is not a baseline rollback.

**Author freeze: 2026-11-14T22:09:11Z (15 November in Europe/Bucharest).** The two
literal-file hashes recorded below are final at this timestamp, before any R2
outcome or score is available to this author. No further literal revision is
authorized under these R2 identities after scoring.

## Exact parent and literal decisions

| Fuel | Exact parent file | Parent release id | Changed scalar |
|---|---|---|---|
| Methanol | `configs/models/flex/wsp6/methanol/r1-astra/release.json` | `sha256:ef2339fe8a8e0fa377adf9ff3aa9e44308f1161dc2792675698e2dcfe3ac2311` | `thermal.cp_gas_j_kg_k`: 1040 -> **1380** J/(kg K) |
| Heptane | `configs/models/flex/wsp6/heptane/r1-astra/release.json` | `sha256:51ce4c7c3622f2563eb5472df7c751b6132a01eb50747e4b2406a3d64e14fac0` | `thermal.cp_gas_j_kg_k`: 1040 -> **1660** J/(kg K) |

Exact edit parents are respectively `configs/models/flex/wsp6/methanol/r1-astra/edits.wsp`
(SHA256 `3d549befbfead7d37701f36ba24483c2e19b15ffdb36b374c3ec08d780687856`) and
`configs/models/flex/wsp6/heptane/r1-astra/edits.wsp`
(SHA256 `b0cd59aef2fc1ced2beef12b4477d1fe24f9c51efb65f567b0ca486e83d0c03e`).
Apply the corresponding `r2-astra/changes.wsp` as a literal revision to that parent.
Do not substitute the joint candidate or the older baseline as parent.

Each patch contains **one new scalar, zero new arrays, zero TRAIN-read numeric
coefficients**. Classification: N (external material property), with L (frozen
300 K property approximation and correct species interpretation). The author typed
1380 and 1660 by rounded SI conversion of the cited molar values, not from a program
output or TRAIN inversion. Existing inherited tensors retain all prior exposure.
`thermal.member_weight=1`, applicability `[1,0,0]`, nitrogen-only carrier restriction,
cutoff restart off, and all other parameters remain inherited. The existing spec
and `droplet7.rs`/`thermal.rs` implement this correction without a code change.

## What the available TRAIN evidence establishes

Inputs read: the two `docs/wsp/feedback/astra-r2-20261115/flex_*-diagnostic.json`
files and the previously frozen TRAIN `docs/wsp/feedback/astra-frozen-train-20261114/results/summary.json`.
No raw experimental continuation was opened. Numbers below describe those prior
releases, not R2. CRPS uses the existing diameter score scale in micrometres.
The orchestrator verified the diagnostic bias convention against `wsp-dev`:
predicted median minus observed diameter. Thus positive is too large and negative
is too small.

| Fuel | Scored cases / clusters | R1 minus baseline cluster-balanced CRPS | Existing 95% cluster-bootstrap interval | Joint minus baseline |
|---|---|---|---|---|
| Methanol | 182 / 26 | +0.0301254 | [-0.0433032, +0.131868] | +0.0155749 |
| Heptane | 125 / 20 | +0.127427 | [-0.0064790, +0.387381] | +0.0559483 |

These intervals overlap zero. They do not identify a population-wide regression,
and do not establish equivalence. The diagnostics count 59 changed methanol cases
and 50 changed heptane cases; the exposure of the thermal branch is sparse.
Case-weighted cutoff summaries are a different aggregation from the balanced
headline: methanol changes at 1/3/5 seconds are +0.03744/+0.06649/+0.07528;
heptane +0.40394/-0.00204/+0.00449. Do not mix those estimands.

**Methanol.** The 2011-09-09 cluster contributes +1.12624 CRPS, offset partly by
-0.391915 on 2010-03-29. N06M101 at cutoff 5 s changes CRPS from 21.4420 to
27.3356; its reported 5-10 s median bias grows from +23 to +56 micrometres.
At 3 s the same run changes from 22.4354 to 26.6618 and +35 to +60 micrometres.
The affected forecast is already almost entirely hot at cutoff (about 0.9954 at
5 s), and the thermal patch acts after future transitions. This localizes the
model difference to prospective unlit paths, rather than proving a bad cutoff
state. The counterexample 175R001 at 5 s improves CRPS by 2.24149, while its large
negative long-lead bias changes only from -137 to -133 micrometres. Most of that
error remains outside the reach of this patch.

**Heptane.** The 2011-09-01 cluster contributes +2.56070 CRPS. One especially
influential case, N04H101 at cutoff 1 s, changes CRPS from 25.8087 to 44.3884;
the reported 5-10 s bias changes from -50 to -90 micrometres. Its cutoff hot/cool/
unlit state probabilities are approximately 0.8815/0.0603/0.0582, hot radiative
class probability is nearly one, and its future cool-flame probability is 0.5.
Those branch probabilities and the thermal state initialization jointly affect
the forecast. They do not identify the actual liquid temperature or missing gas
heat. In the same session N03H101 at cutoff 1 s improves CRPS by 0.38465 while
median absolute error worsens by 1.45473: distribution-score gain need not be
median-trajectory gain.

**Causal boundary.** R1 differs from its predecessor through an explicitly gated
thermal law, so the changed predictions can be attributed to that implementation
path. The diagnostics do not reveal which physical approximation is responsible
for empirical error. Methanol's dominant affected bias is positive and heptane's
negative; a shared increase or decrease of evaporation cannot be justified as a
universal repair. Neither the diagnostics nor the aggregate summary can estimate
an irreducible measurement-error floor or the score of a perfect physical law.
No such floor is claimed here.

## Ranked options and decision before scoring

1. **Correct the species heat capacity in Stefan enthalpy transport.** Strong
   physical justification, minimal parameter change, uncertain and probably small
   forecast benefit. Selected for both fuels as one shared hypothesis.
2. **Resolve post-flame liquid temperature, internal gradients, and gas thermal
   memory.** Potentially much larger relevance to the identified errors, but the
   available TRAIN diagnostics cannot separately identify them. A warm-cloud
   amplitude/time or new initial temperature chosen to cancel these biases would
   be an unsupported tuned closure. Not authored.
3. **Change radiation/cool-flame probabilities or blend back toward the baseline.**
   These may reduce a retrospective loss, but the same session contains opposing
   metric movements. No independent probability evidence justifies new weights.
   No mixture weight or gate is changed.
4. **No change.** Prefer this for empirical deployment unless R2 meets the
   preregistered evaluation rule. Keeping R1 is preferable to inventing a large
   TRAIN correction. Physical consistency alone does not authorize a superiority
   claim or deployment.

## Physical derivation and source boundary

For a quasi-steady spherical film with stagnant inert carrier, the carrier's net
mass flux is zero and the outward mass flow is fuel vapor. Including species
diffusive enthalpy in the energy balance gives, for constant conductivity and
vapor heat capacity,

`d/dr [ mdot * h_v(T) - 4*pi*r^2*k*dT/dr ] = 0`.

Consequently the steady radial temperature equation contains `mdot * cp_v`, not
`mdot * cp_N2`. With radius `a`, its inward surface heat is

`Q_in = 4*pi*a*k*(T_inf - T_s) * beta / (exp(beta)-1)`,

`beta = mdot*cp_v/(4*pi*a*k) = rho_liquid*K*cp_v/(8*k)`.

The last equality uses `K=-d(D^2)/dt` and constant liquid density. This is the beta
already evaluated by `Thermal::flux`; its `cp_gas_j_kg_k` scalar appears there.
The existing N2 value silently equates carrier and vapor heat capacity. Changing
the scalar corrects that approximation. It does not supply missing multicomponent
transport, vapor-dependent conductivity, liquid gradients, or unsteady gas storage.

[Abramzon and Sirignano (1989), DOI 10.1016/0017-9310(89)90043-4](https://doi.org/10.1016/0017-9310(89)90043-4)
identifies the vapor heat capacity in Eq. (9), separately from film-mixture
properties. Read the publisher abstract and the original-paper transcription on
[the author's hosted copy](https://www.academia.edu/49657986/Droplet_vaporization_model_for_spray_combustion_calculations),
especially pp. 1605-1609, nomenclature and gas-phase analysis. OCR damages some
equations, so the derivation above is explicit; this is not a claim that the full
Abramzon-Sirignano model is implemented. The paper predates the modeled ISS series
and supplies class A physical theory. Search previews of later academic theses
helped locate the primary equation; no constants were taken from those theses.

The NIST WebBook gas tables supply class A property data:

| Source | Exact source quantity | Typed constant and approximation |
|---|---|---|
| [Methanol, gas thermochemistry](https://webbook.nist.gov/cgi/cbook.cgi?ID=C67561&Mask=1), TRC 1997 recommended table | 44.17 J/(mol K) at 300 K; molecular mass 32.0419 g/mol | 1380 J/(kg K), about 0.1% upward rounding; held constant |
| [Heptane, gas thermochemistry](https://webbook.nist.gov/cgi/cbook.cgi?ID=C142825&Mask=1), Scott 1974 recommended table | 165.98 J/(mol K) at 300 K; molecular mass 100.2019 g/mol | 1660 J/(kg K), about 0.2% upward rounding; held constant |

Read identity, gas-Cp tables and their reference notes, not modeled-series
results. These are evaluated ideal-gas properties; the original TRC/Scott
compilations and underlying measurements were not independently reanalyzed.
The tables themselves show temperature dependence: a constant 300 K property is
still an approximation away from 300 K. Choosing the inherited reference avoids
inventing a TRAIN-specific representative film temperature. NIST URLs with an
additional `Units=SI` query initially failed in the web tool; the cited URLs
without it opened successfully and display SI molar units.

## Preregistered expectations and falsifiers

**Mechanistic hypothesis:** replacing carrier Cp with vapor Cp restores the
constant-property stagnant-carrier Stefan energy balance. At an identical state,
evaporation K is unchanged immediately; only the energy derivative changes. Since
`beta/(exp(beta)-1)` decreases for positive beta, a warm liquid (`T_s>T_inf`)
loses less sensible heat to the gas. Below ambient, it receives less sensible
heat. At equal temperatures the conductive term stays zero. Later evaporation
therefore has competing transient effects; there is no predeclared global sign
for long-horizon diameter or CRPS.

The warm-phase tendency may shrink diameter slightly more and thus **worsen** the
dominant negative heptane bias. That risk is accepted before a score. This is a
physically justified correction with unproven predictive utility, not a claim to
have fixed the dominant TRAIN losses. Unsupported carriers and unaffected flame
paths should retain the parent's behavior; no state-filter or event-clock claim
is made.

Recommended single frozen TRAIN comparison, owned by the orchestrator: compare
each R2 with its exact R1 parent and the retained baseline on the same case sets;
report refusals/lost cases first, then the existing cluster-balanced CRPS and its
paired cluster uncertainty, median error and the same 1/3/5 s cutoff summaries.
Keep every result, including harm. A TRAIN gain is exploratory and does not
establish generalization. No validation/test scoring is authorized here. Do not
adjust these two literals after inspecting R2 outcomes under the same R2 label.

## Checks and exposure accounting

Executed `cargo test -p ignis-wsp --test droplet7_forward --locked --offline`:
**15 passed, 0 failed**. This existing synthetic suite covers spherical energy and
mass balances, convergence, source-domain refusal, carrier fallbacks, the pressure
cap, preserved cutoff history, distribution mixtures, and disabled equivalence.
It does not score R2 or establish its empirical accuracy.

Executed an independent Python fixed-state calculation via standard input. It
read only each R1 `addition.wsp` and R2 `changes.wsp`, asserted exactly one scalar
assignment with the declared value, and evaluated `Thermal::flux`'s published
formulas for a synthetic 2 mm droplet at 101325 Pa in a nitrogen/oxygen carrier,
300 K ambient and no cloud. Liquid temperatures were the unchanged authored
initial temperature, 280 K, and 300 K. The source extension at 280 K is the
inherited boundary Clausius-Clapeyron rule. All six finite-state sign checks
passed; K is identical before/after at each fixed state.

| Fuel | Liquid K | Old Stefan factor | R2 Stefan factor | Old dT/dt K/s | R2 dT/dt K/s |
|---|---:|---:|---:|---:|---:|
| Methanol | 330 | 0.517645 | 0.407182 | -50.692741 | -50.563476 |
| Methanol | 280 | 0.973212 | 0.964560 | -1.669553 | -1.676303 |
| Methanol | 300 | 0.913671 | 0.886590 | -7.788252 | -7.788252 |
| Heptane | 360 | 0.416217 | 0.226249 | -26.051769 | -25.470683 |
| Heptane | 280 | 0.986421 | 0.978385 | +0.502877 | +0.494683 |
| Heptane | 300 | 0.959838 | 0.936421 | -1.452119 | -1.452119 |

These are synthetic instantaneous derivatives, not NASA measurements, trajectories,
scores or new weight values. The checks use the formulas above together with
`dT/dt = [12*k*(T_inf-T_s)*factor/rho - 1.5*L*K]/(cp_liquid*D^2)`.
They verify direction and scalar identity independently of the Rust execution;
they do not substitute for candidate parsing/assembly. The literal file SHA256s
after writing are methanol
`f03cb101900f64575d372e802eba129e30fd4ceb986aa43b50909e67bd2c74f5`
and heptane `75fadf6481e6d8828060ace61cdb65c4754a364207f1d17f768f0839d49ceb89`.
Release parsing/assembly and any scoring remain the orchestrator's responsibility.
No prior release, existing forward, shared contract or generated schema is changed.

The author also read root AGENTS, relevant implementation ownership, existing
thermal code/spec/test descriptions, R1 additions and metadata, and
`ASTRA_PROPERTIES.md`. Opening the older `ASTRA_PROMPT.md` to inspect author rules
also exposed qualitative historical validation decisions and a stated historical
precision range; no validation table, per-run outcome, sealed data or test
partition was opened. This incidental historical prose exposure must be retained
alongside the parent's full exposure record; it is not a clean-room candidate.
No value here was selected from that prose. No optimizer, regression, parameter
scan, residual-derived numeric correction, coefficient inversion, or new-candidate
scoring was used.

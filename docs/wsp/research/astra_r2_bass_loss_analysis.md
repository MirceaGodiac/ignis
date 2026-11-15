# Astra R2 BASS: TRAIN loss analysis and no-change decision

Author: GPT-6 Astra. Date: 2026-11-15. **DATA-INFORMED, not zero-shot.**

**Decision: no new BASS candidate.** The current evidence identifies limitations of
the steady supply and observation closures, but does not identify a defensible new
literal within `bass_front-wsp-v3`. The comments-only `r2-astra/changes.wsp` is a
decision record, not an executable patch. Do not assemble a duplicate R2 release.
The exact retained reference is `configs/models/bass/front3/joint-astra`, release
`sha256:ffbb724a068338c789a7276254d1b9e81ee18a435c760d215b21ac8336a3db37`.
Retention is continuity of the existing candidate, not a new adoption or qualification.

No literals, old forwards, immutable releases, splits, metrics or shared contracts
were changed. No candidate was scored. No fitting, inversion from track values,
coefficient sweep or parameter search was performed.

## Evidence actually supplied

The authorized frozen TRAIN summary contains three scored tracks, T16, T5 and T6,
at four cutoffs each: **12 correlated cases from one independent test day**, group
`2014-100`. Here “test day” means the experimental grouping, not the protected test
partition. These are TRAIN outcomes. A confidence interval across independent days
cannot be supported by this single group; the summary correctly has `ci95: null`.

| Existing candidate | Mean CRPS, sample widths | Mean absolute error of median | 50% coverage | 90% coverage |
|---|---:|---:|---:|---:|
| Frozen baseline | 0.076101 | 0.088031 | 0.82553 | 0.97365 |
| R1 Astra | 0.080217 | 0.089997 | 0.84029 | 0.98908 |
| Joint Astra | 0.073439 | 0.069699 | 0.86265 | 0.98908 |

These are copied from the frozen summary, not newly computed candidate scores.
The joint mean is modestly lower than baseline, with much more than nominal central
coverage on these dependent cases. This does not establish calibrated uncertainty,
improved physics or generalization. CRPS assesses both location and spread, so a
median improvement is not alone an explanation of its change.

| Run / cutoff, s | Baseline CRPS | R1 CRPS | Joint CRPS |
|---|---:|---:|---:|
| T16 / 5 | 0.076199 | 0.102204 | 0.079167 |
| T16 / 10 | 0.086984 | 0.063241 | 0.074996 |
| T16 / 20 | 0.055061 | 0.065158 | 0.051599 |
| T16 / 40 | 0.031280 | 0.041649 | 0.035909 |
| T5 / 5 | 0.110244 | 0.141190 | 0.119320 |
| T5 / 10 | 0.103999 | 0.144236 | 0.120561 |
| T5 / 20 | 0.140138 | 0.078653 | 0.102607 |
| T5 / 40 | 0.038477 | 0.074878 | 0.053136 |
| T6 / 5 | 0.079314 | 0.105919 | 0.079563 |
| T6 / 10 | 0.095910 | 0.068197 | 0.081093 |
| T6 / 20 | 0.066567 | 0.039307 | 0.051070 |
| T6 / 40 | 0.029039 | 0.037971 | 0.032249 |

R1 is worse at all three 5 s cutoffs and all three 40 s cutoffs; its benefit at
intermediate cutoffs is mixed. At 5 s the encoder has zero span and zero gain: the
windowed image anchors position, but it supplies no run speed correction. Thus an
early discrepancy can reflect the unadjusted law, optical overhang or measurement
timing; it does not uniquely identify any one. At 40 s the gain is about 0.986, yet
the later 30–60 s lead CRPS remains worse for R1 in all three tracks. A nearly fully
trusted prefix does not remove a wrong flow dependence or transient observation law.
Absolute errors and CRPS do not reveal the sign of position bias; no signed claim
about observed overprediction is inferred from them.

**Diagnostic caveat:** `joint.encoded` is the reference path of the first scenario,
which here is the predecessor-speed member at nominal flow. `run_raw` creates that
member first and selects the first zero-sign path. It is neither a mean encoder nor
the oxygen member's speed. Full position CDFs are mixed, with separately encoded
members. The joint is not equivalent to a single law with an averaged velocity.
Its event rows also use the new first-event interpretation for both members; an
event-score change cannot be assigned solely to a speed-law improvement.

## Oxygen supply: units are correct; source domains do not close the physics

The implemented per-face budget is

`v (m_total / faces) stoich = rho Y U delta`,

with `delta^2 = nu L/U + (alpha/U)^2` and the steady closure `L = v tau`.
Writing `A = faces rho Y / (m_total stoich)` gives

`v^2 = A^2 nu tau U v + (A alpha)^2`.

`A` has units 1/m; `A^2 nu tau U` and `A alpha` both have units m/s.
The positive quadratic root in `speed_m_s` therefore has units m/s, and dividing
by the explicit physical width produces widths/s. `18.05 mg/cm2` is `0.1805 kg/m2`
total material; the inherited 75% combustible share rounds to `0.1354 kg/m2`.
Dividing that total loading by two faces exactly once is dimensionally consistent.
This is a unit audit, not verification of the specimen's actual loading or exposed
area. No factor-of-two or centimetre conversion correction is justified here.

The inherited provenance has distinct domains:

- `rho = 1.15 kg/m3` is an ambient nominal environment assumption.
- `nu = 0.000178 m2/s` and `alpha = 0.000241 m2/s` are rounded dry-air transport
  quantities at a 1250 K reference, not measured BASS boundary-layer properties.
  The repository dossier records a prior CoolProp cross-check; it was not rerun
  here. The directly reopened [CoolProp Air documentation](https://coolprop.org/fluid_properties/fluids/Air.html)
  identifies its air equation of state and viscosity/conductivity references. That
  is implementation/property provenance, not a derivation of this capture layer.
- `Y` is converted from oxygen mole fraction using a binary O2/N2 approximation;
  fixed dry-air transport is not recomputed for the actual composition.
- The SIBAL loading's repository citation is a Saffire VI material description.
  Transfer to these BASS specimens remains an apparatus assumption, not a measured
  value for these tracks. The Saffire report was not opened in this review.
- `tau = 10 s` was explicitly derived by the earlier author from TRAIN
  length/speed ratios. It is a data-informed closure, not a universal material
  property, and is distinct from the inherited visible-base delay near 6.30 s.
- The reference width `0.022 m` is inherited nominal calibration; no new image
  calibration was obtained here. The supplied scores remain in sample widths.

Using ambient density with hot-layer transport can be an explicitly approximated
capture-layer scaling. It is not a uniform-temperature gas calculation. Replacing
only density with a hot-state value is not a demonstrated repair: the imposed
velocity, thermal expansion and capture-area definitions must change consistently
with the control volume. The steady law is monotone in A, so such an isolated density
reduction would lower its speed; that directional fact is not evidence it improves
the observed tracks. No replacement value is authored.

The alpha term represents thermal preheat length, not a measured oxygen diffusion
coefficient. Equating thermal and species transport would require an additional
Lewis-number approximation. The nonzero `v = A alpha` limit at U=0 is the limit of
this scaling, not proof of a sustained zero-flow flame: heat losses and extinction
are outside its budget. Finite width, lateral oxygen entrainment and fiberglass heat
storage are also absent. These are established omissions of the equations; which
omission dominates the TRAIN error is unproven.

## Transient prefix closure: identifiable assumptions, unidentified corrections

The code supplies a steady pyrolysis advance and adds an independently relaxing
visible overhang. It estimates a multiplicative advance correction from two windows.
The tip channel subtracts modeled overhang; the base channel compares base advance
with a path shifted back by a fixed burn-through delay. Both are proxies for the
same latent front only under those observation assumptions.

For R1 at T16/10 s, tip/base log offsets are approximately -0.306/+0.199; at T5/10 s,
they are +0.046/-0.460. Their opposite signs demonstrate disagreement of the two
proxy estimates at these prefixes. They do not identify which channel is correct.
The inherited 0.8/0.2 mix and flow/length-based gain can carry transient overhang,
base delay or optical error into a persistent multiplicative spread correction.
At later cutoffs the windows still span earlier flow changes: “settled” at the last
window is not a guarantee that the entire offset interval is stationary.

A concrete mathematical approximation also exists: the code subtracts `h(mean t)`
from the mean observed tip, and uses `x(mean t)` for each window. For nonlinear
transient paths these are not the means of h and x at the observation times. With
`h(t) = h_ss + C exp(-t/tau_h)` over a constant-flow interval, the discrepancy is
`C [mean(exp(-t/tau_h)) - exp(-mean(t)/tau_h)]`. It is zero for a constant path or
zero-width window, but generally nonzero during relaxation, with sign set by C.
This follows from the implemented window operation and does not need measured
track fitting. Its magnitude in the actual observations has not been evaluated.
A future matched-window observation operator would remove this approximation for
the modeled path; it would not itself establish that the modeled overhang is real.
Changing a global relaxation time cannot generally correct this operator error.

`L = v tau` responds immediately to changed flow; the separate h relaxation and base
delay do not turn it into transient fuel/oxygen conservation. Conversely, making
the residence coefficient equal to the visible-base delay would conflate two
different definitions without a supporting measurement. Changing the gain, removing
the base proxy or reducing uncertainty would be a new judgment with an unidentified
effect, not an evidence-backed physical correction.

## Why no R2 literals, and what a later mechanism would predict

No mixture weight adjustment is proposed: moving toward whichever member scored
better is score-guided weighting, not new physics. No uncertainty tightening is
proposed despite excess TRAIN coverage: one day cannot distinguish conservative
noise from dependent sampling and unmodeled run effects. No flow-deceleration,
residence or overhang literal is inferred from the track values.

A later, separately scoped architecture experiment could compare window-matched
observation predictions and a physically defined transient mass/energy inventory,
with typed thermal/ambient states and independently specified transport domains.
Window matching should reduce modeled averaging error specifically near transients
and have no effect on a linear/constant modeled window; transient inventory should
retain history rather than instantly impose `L = v tau`. Neither implies a lower
CRPS, a known optimal literal or a correct physical flame. Such work needs a new
forward/spec and a predeclared protocol, not edits to the frozen forward here.

For the present no-change decision, the predicted effect is exactly none: the
retained reference and all its limitations stay unchanged. Zero new authored scalar
positions, zero new candidate evaluations, no new release identity.

## Exposure and executed checks

Read: root AGENTS/ownership and Iancu handoff, the two explicitly authorized TRAIN
JSON files, front3 source/spec and R1/joint patches/cards, the property dossier's
fabric/property context, author-boundary guidance, and property-review documents
found while locating that context. No raw TRAIN track file was needed or opened;
only names in the allowed TRAIN directory were listed. The mandatory handoff
incidentally exposed historical aggregate validation summaries, including old BASS
generations. This exposure is disclosed and was reported to the coordinator; those
numbers were not used to choose any change. This review must not be described as
validation-blind. No protected partition directory, per-run validation table,
published per-test outcome report or final-test payload was opened. The initial
authorized frozen-summary read also exposed other families' TRAIN summaries.

Executed read-only checks: PowerShell JSON parsing and extraction of existing BASS
rows; manual algebra/unit and source-domain audit; source inspection of encoder,
mixture and reference-path semantics; SHA-256 identification of inputs; final
`git diff --check` and explicit verification that the decision file has no active
assignments. No Cargo test, inference, assembly or scoring command was run: only
documentation and a non-executable decision record were written.

| Input | SHA-256 |
|---|---|
| `docs/wsp/feedback/astra-r2-20261115/bass_front-diagnostic.json` | `ff3cb40d3cc92ae4b15b8a55296964144352ca0898cea6ad56c353531cab0511` |
| `docs/wsp/feedback/astra-frozen-train-20261114/results/summary.json` | `53ab5386824d1f5930ecc5a7753c0c454b39e4c54718982e9d29b642eca61840` |
| `configs/models/bass/front3/spec.json` | `6c82c4eb7052725b163dcb5fd03b5be8b431c1298f8ae9784e793974f9c1ca3d` |
| `configs/models/bass/front3/joint-astra/edits.wsp` | `32dc1e162e31ba068f8a5a2545b7e530fac82a95bba2a5065cb8fd4fa43ccb99` |
| `crates/ignis-wsp/src/front3.rs` | `129ccee02e21eaa1b5c7fcf9f975d7822e5671f5efa600a79c10d83bcaf4a093` |

These hashes identify the inspected snapshot; they are not qualification evidence.

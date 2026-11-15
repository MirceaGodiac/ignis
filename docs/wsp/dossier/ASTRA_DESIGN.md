# Astra: more mechanism, with an account of every choice

14 November 2026. First-principles design decision before authoring the new candidate
weights. **All Astra candidates in this work are data-informed, not zero-shot.** I have
read this dossier's train measurements and historical aggregate validation results.
The user requested GPT-6 Astra for research analysis; the independent analysis tasks
`astra_research_analysis` and `astra_physics_analysis` were explicitly launched as
`gpt-6-astra`, high effort. Their source-backed reports accompany this memo.

## What is being programmed?

A scientific forecast is a composition: input semantics, observation model, state
evolution, unresolved alternatives, and output distribution. Its adjustable content
includes code, feature choices, thresholds, source selection and parameter values.
Counting only tensor elements misses most of that content. A hand-written forward is
not automatically a neural-network construction in the sense of Tracr or the recent
WSP experiments; it is a mechanistic program whose scientific coefficients are carried
as literal tensors. That distinction is useful and should remain visible.

Put identities, conservation relations, unit conversions and integration algorithms in
code. Put uncertain physical levels, source property coefficients, apparatus settings,
closure choices and scenario masses in typed tensors. Do not put a fitted answer into
preprocessing and call its downstream tensor a handwritten model. A property equation
evaluated at runtime is computation; a program evaluating it on a grid and saving the
outputs as candidate weights is weight generation, which this protocol forbids.

## Where can generalisation come from?

It can come from a mechanism that remains applicable, from inputs available under the
same timing policy, and from uncertainty that admits mechanisms the observations cannot
distinguish. None is a theorem that a physically named coefficient transfers. A wrong
radiative closure can transfer a wrong answer very consistently. Fable's nine outcomes
are useful warnings, not an independently tested law of generalisation.

The N/A/L/S/U taxonomy describes a number's role. Add a second, independent axis for
its evidence: external source, apparatus document, direct train measurement, inherited
train-informed value, author judgment, or exact structural identity. Record source
selection, validity range and whether the choice was made after feedback. An externally
tabulated value can be precisely transcribed yet selected adaptively to explain train.
There is no defensible equality between number of train-read coefficients and effective
degrees of freedom. Nor does adding one nature constant necessarily remove one fitted
degree of freedom. Report counts as an audit inventory, not a complexity penalty.

A large model is defensible when its added states have observable consequences or
testable limiting behaviour, and its constants are tied across uses. Repeating a handbook
constant a thousand times adds storage, not evidence. Tying it once and exposing both
consumers to the same error is stronger. A full resolved transport solver would need
boundary conditions, chemistry and property domains this workspace does not yet identify.
This generation will add limited transport mechanisms and physically named uncertainty,
not claim that resolution alone solves those missing conditions.

## Chosen extensions

The quantified [train error budget and ranking](ASTRA_ERROR_BUDGET.md) supplements this
memo. Its chronology is explicit: first module prototypes preceded the numerical budget;
candidate weights followed it. Every expected unseen gain remains unmeasured and may
be negative. The budget also explains why large tracker/label errors are left unresolved.

1. **Unlit droplets: coupled mass and energy.** Retain the predecessor's hot/cool-flame
   regime and prefix model. Add a new optional post-flame law using molar Stefan diffusion,
   a liquid energy balance, source vapor pressure and latent heat, and explicit numerical
   integration. Derive the mass flux consistently in mole fractions; using ambient gas
   density in a mass-fraction formula is particularly suspect for heavy heptane vapor.
   Initial liquid temperature, residual gas heating and composition remain assumptions,
   not recovered measurements. The new mechanism must say where it applies and preserve
   the earlier law with its switch off. The frozen-diameter hyperbolic cooling law is a
   local approximation, not an all-time energy identity. Methanol's absorbed water and
   heptane's cool-flame history remain distinct limitations.
2. **Fabric: a steady oxygen-supply closure.** Add an alternative spread law from the
   oxygen budget, gas transport and fuel loading. Keep the measured-prefix anchoring,
   delay and finite flame-length response explicit. The positive quadratic root follows
   from the steady closure `L = v tau`; it is not a measured transient balance. In
   particular, the mean of tip and base velocities requires a stationary burn-progress
   profile. Do not derive extinction probabilities from a speed balance. Retain those
   as a separately disclosed uncertain mechanism.
3. **Burner flames: persistent condition uncertainty.** Integrate complete forecasts at
   literal condition nodes representing uncertain flow and pressure. Composition stays
   fixed because the supplied Tad/Zst are not recomputed. One
   node describes an entire run; it is not reselected at each horizon. Average cumulative
   incidences and survival with the same node weights. Do not average hazards or pretend
   separate event clocks prove independent physical causes. Keep disagreement between
   laws separate from uncertainty in an instrument's input.

These are new forwards and new model names. New source modules enter only the new
forward hashes. Existing forward sources, the four shared numerical sources, releases,
contracts, shared records, endpoint labels and partitions remain frozen.

## What uncertainty means here

When theory supplies a dependence and train supplies a level, the uncertainty in a new
condition does not follow from the train residual alone. Use explicitly assumed scenarios
for the disputed level, input or mechanism, and report sensitivity. Without exchangeable
calibration data or a defensible generative model there is no distribution-free coverage
claim. Assumed scenario masses are not posterior probabilities.

An equal mixture of two different authors' distributions is a reasonable preregistered
joint statement of unresolved disagreement. It improves a quadratic score relative to
the average member score by the disagreement term; it need not beat the better member.
CRPS concerns the CDF mixture, not averaged quantiles. Shared priors and sources make the
authors correlated, so two names do not create two independent experiments. A joint
candidate must preserve complete member distributions and exposure ancestry.

## Where handwriting ends

Allowed: typing a source coefficient with its reference; transcribing an immutable
member's literal assignments without changing values; choosing an explicit round-number
scenario by scientific judgment; evaluating a fixed law and measuring train residuals.
All must retain their actual origin. A handbook correlation may itself have been fitted
by its original investigators: that is external property evidence, not a new fit to
IGNIS runs, and its empirical nature must be stated.

Forbidden: solving two train points for a coefficient, scanning several numbers and
choosing the best, optimizing latent parameters in the forward, selecting a residual
correction table by run identity, or asking a delegate to author weights. Calling a
two-point solution algebra does not make it independent of the data. A copied value
keeps the first author's exposure; copying does not make it zero-shot.

## Acceptance and stopping rule, before feedback

The purpose of the next train runs is to expose failures, check support and compare
mechanisms, not select numerical weights. Type each initial candidate once after the
source and dimensional checks. Record all results, including refusals and worse scores.
Correct implementation errors with a stated invariant; a scientific change after feedback
is a separately named revision. Do not silently revise the original candidate.

Require neutral-switch identity, valid probability mass, monotone time evolution where
the law requires it, mass/energy and unit checks, source-domain disclosure, fixed-step
convergence checks, complete literal assembly and immutable predecessor hashes. These
checks establish implementation properties on their stated domains, not NASA validation.

No new validation look is authorized by this design. The worn partitions cannot settle
an argument about mechanisms. Existing carried models stay carried. The new releases
are experimental, data-informed alternatives and, where implemented, explicit joint
committees; train results do not promote them to validated models.

## Honest ancestry across architectures

The existing assembler rightly requires same-spec tensor inheritance. A new architecture
with an exposed author cannot truthfully be emitted as a root `LiteralOneShot`, nor can
prior-candidate exposure be erased to make that pass. The new explicit `rewrite` route
requires a complete destination patch, fresh spec initialization, a real literal parent
of the same family and population binding, a new model/spec, and full parent exposure.
It serializes `LiteralRevision`/`WeightEdit`: a new experiment with ancestry, not W0 and
not implicit cross-shape inheritance. Ordinary assembly is unchanged. A parentless copy
checks coverage only; no parentless release is serialized. This is a deliberate new
owned-lane authoring operation, reviewed by GPT-6 Astra and tested without shared-contract
changes. Exact inherited values retain Fable's credit; the new complete patch belongs to
Astra, including all new neutral/physical literals.

The independent forward review found an inherited BASS event-semantics defect (F-007).
Only the active new route gets a coherent first-event mapping. In the joint experiment,
both speed members receive that mapping: position distributions preserve equal-author
disagreement, while the event model is explicitly a new interpretation. The neutral
compatibility path remains exact, and no historical release is changed.

## Audit boundary

The review of programmed stops confirms a timing relationship on train, not file creation
before ignition (F-001). Scores using that input remain conditional on acceptance of its
provenance. A collector also reported a possible raw-file alias collision during a
methanol header check. Raw access was stopped; uncertain material is excluded from this
design and the access uncertainty is recorded in F-004. No protected values are reproduced
here. Property theory and independent analytic derivations remain separately attributable.

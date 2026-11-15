# Astra: what weight-space research justifies for IGNIS

Author: **gpt-6-astra**, explicitly selected by the owner as a second author alongside Fable.
Date: 2026-11-14. Responsibility here: research and architectural judgment only; no weights,
parameter fitting, candidate selection, model execution, or validation look was performed.

This is a source-backed design analysis, not a claim to have reproduced the cited experiments.
The repository dossier has exposed this author to train-derived statements and aggregate
development results. Any subsequent candidate using this analysis is **data-informed**, not W0.
No test/holdout files, sealed validation continuations, per-run validation outcomes, or published
per-test outcomes of the protected NASA series were opened. The exposure ledger itself was read;
reading a ledger does not erase the historical exposure it describes.

## Judgment

The ambitious direction is a **composable scientific program with literal property, apparatus,
mechanism and uncertainty blocks**, accompanied by a checkable account of what every block means.
It need not be a dense neural network. Its size should grow when another independently justified
physical state, property curve, instrument effect or alternative mechanism becomes representable.
Its freedom to accommodate the observed runs must be accounted for separately.

I agree with Fable that the present compressed empirical laws can hide the quantities needed for
transfer. I disagree with three stronger interpretations of the extension map:

1. Hundreds of external constants plus twenty train-informed constants do **not** establish twenty
   effective degrees of freedom. Choosing the source, interpolation, closure, state variables,
   regime partition, measurement interpretation and candidate after seeing outcomes also adapts
   the model. The number of decimals typed is not a statistical complexity measure.
2. Adding a nature/apparatus constant does **not automatically remove** a law/structure/uncertainty
   degree of freedom. It does so only if an independently fixed relationship actually replaces a
   discretionary choice. Adding a property table while retaining a free correction can increase
   complexity and preserve the same freedom.
3. Mechanistic agreement in a few development cases is evidence of plausibility, not a transfer
   law. A physically named coefficient can compensate for the wrong mechanism. Correct ratios
   with a factor-of-two level error can reflect a missing multiplicative factor, but can also be
   coincidental cancellation; the two explanations make different extrapolation predictions.

These are methodological objections, not reasons to retreat to small models. A much larger model
can be more defensible when its added quantities have external provenance, explicit validity
domains and cross-family consequences that were specified before seeing the outcome.

## 1. What the WSP literature establishes—and what IGNIS adds

Takehi, Teoh, Zhang and Khattab present WSP as a capability/concept, explicitly not a single
optimization method. Their November 12 response says the broader concept can include gradients;
their reported experiments prohibit them. Tool access and feedback differ across experiments.
Their Qwen edit addresses looping; the separate Gemma experiment did not establish a general
reasoning improvement. Those distinctions prevent treating the headline as proof that arbitrary
missing scientific knowledge can be added through a few edits. I verified the primary post's
wording and selected setups, not the released executions.
([Primary post](https://rikiyat.github.io/blog/weight-space-programming/), experiments 8/additional
experiment and response to comments.)

Lehmann's November 12 study supplies a useful counterweight: known, source-visible mechanics can
be expressed in compact networks, but ordinary programs did better in the tested setting. The
paper explicitly distinguishes within-family generalization from discovering unknown rules and
acknowledges unequal information access between writers and learners. Its free-architecture
condition and fragile written weights also make architecture and numerical robustness part of the
question. These are author-reported results; I read the abstract, main findings, limitations and
selected methods, not all appendices or code. For IGNIS the inference is to preserve explicit
physical computation where it is clearer, and test numerical margins rather than imitate a game
network's parameter count.
([arXiv:2611.16981v1](https://arxiv.org/html/2611.16981v1), sections 1, 5 and selected appendix A.)

Gross combines number-valued network outputs with model checking over specified PRISM
environments. Encoding self-checks and property checking are separate; the paper warns that a
verified returned policy need not satisfy the target within the allotted rounds. This supports
separating transcription correctness from the intended behavior. It does not verify the physical
environment model used by IGNIS. A proof of a finite state machine's legal transitions would not
prove that its extinction law describes a flame.
([arXiv:2611.11604v1](https://arxiv.org/html/2611.11604v1), sections 4.1–4.3.)

IGNIS's requirement is stricter than the name WSP: the released parameter values must originate
as author-written literals, not a coefficient-generating program, optimizer or compiled formula.
Consequently the transferable idea is **reasoning about a structured executable representation**,
not importing every published weight-construction procedure. “No gradients,” “no training loop,”
“no program chose the coefficients,” and “no target-data feedback” are different assertions.

## 2. Constructive networks, interpretation and symmetry

RASP provides sequence-processing primitives for reasoning about transformer computations;
Tracr compiles programs into transformers with known structure, useful as ground truth for
interpretability. Their direct value here is the pairing of a high-level specification with a
lower-level representation whose behavior can be checked. Running a compiler to produce IGNIS's
literal scientific weights would violate this project's rule. A verifier that consumes already
typed weights and compares their behavior with the claimed law is a different, useful tool.
([RASP](https://arxiv.org/abs/2106.06981),
[Tracr](https://arxiv.org/abs/2301.05062), abstracts read; compiler not executed.)

Yarotsky gives constructive ReLU approximations of squaring and multiplication on bounded
domains, including accuracy-dependent depth/size. This is stronger than merely invoking universal
approximation: one can discuss an explicit error budget. It still does not identify the correct
physical function. For IGNIS, use a typed piecewise closure only when there is a reason to need
that representation; direct multiplication or an explicit physical formula is simpler when
allowed. Separate approximation error from uncertain physics, and never silently extrapolate a
bounded construction beyond its domain.
([Yarotsky](https://arxiv.org/html/1610.01145v3), proposition 1 and section 3.1 read.)

Transformer-circuit work shows why recognizable individual weights are not the whole story:
end-to-end products and computational paths can matter, and not every internal space has a
privileged coordinate basis. Permutation-symmetry work likewise makes raw coordinate comparison
between independently authored networks unreliable without alignment. These results suggest
semantic block names and fixed coordinate conventions, rather than a claim that each scalar has
one unique interpretation. They do not authorize using an alignment optimizer to generate a
literal release.
([Transformer Circuits](https://www.transformer-circuits.pub/2021/framework/index.html), summary
and basis discussion; [Git Re-Basin](https://arxiv.org/abs/2209.04836), abstract.)

My proposed implementation consequence is to give each block a short *computational claim*:
inputs with units, admissible domain, output meaning, limiting behavior and a neutral setting.
Verify intermediate quantities as well as final forecasts. Two wrong submodules can otherwise
cancel on the output being scored. A future neural circuit should use named channels and fixed
ordering from its first version, avoiding a post hoc canonicalization problem.

## 3. Where the generalization claim should come from

The N/A/L/S/U taxonomy should remain as an origin ledger, supplemented by independent columns:

| Record | Why it matters |
|---|---|
| Scientific quantity, units and admissible domain | Prevents a fitted-looking constant from receiving authority merely through a physical name |
| Exact source/version/locator and reading extent | Separates verified transcription from inherited claims |
| Direct transcription, author derivation, judgment, or train-informed value | Distinguishes acquisition routes without hiding adaptation |
| Information available when value **and structure** were chosen | Captures source selection and regime choices after outcomes |
| Ties and dependencies on other blocks | Makes a shared constant one scientific assertion, even when repeated in storage |
| Alternatives considered/rejected and feedback available | Records adaptive freedom that scalar counts miss |
| Physical uncertainty, approximation error and measurement uncertainty | Prevents an anonymous band from concealing several unrelated issues |

An external handbook coefficient may itself have been empirically fitted by its publisher.
Transcribing that externally fixed property law is different from fitting IGNIS's campaign. It
must be disclosed as an external correlation with its original domain, not described as a
fundamental constant or independently remeasured value. Cantera's documentation illustrates the
distinction between a temperature-dependent property model and its tabulated coefficients, and
also why polynomial convention and temperature intervals must travel with them.
([Cantera species thermodynamics](https://cantera.org/stable/reference/thermo/species-thermo.html),
overview and NASA polynomial description read; no coefficients imported.)

MDL is useful as discipline, not as “shorter explanation wins.” Its formal versions need a coding
scheme and a predictive/data model; prose length and count of train-sourced scalars are neither.
The tutorial explicitly treats model coding choices and the limits of simple formulations. A
fixed, preregistered catalogue of mechanisms could support an honest description-length analysis
later. No MDL bound, effective dimension or ranking has been computed here.
([Grünwald](https://arxiv.org/pdf/math/0406077), introduction and selected two-part-code passages.)

My practical proposal is a **decision ledger**, not a synthetic scalar freedom score: record
which decisions could have changed after each information exposure. This cannot reconstruct
unrecorded historical selection, but it makes the next generation auditable. “Mechanisms that
mean the same thing in another session transferred” remains an encouraging repository observation,
not a theorem established by the current small and repeatedly consulted development partitions.

## 4. Expert uncertainty and committees

SHELF treats elicitation as a documented synthesis of expert uncertainty. AutoElicit investigates
LLMs as sources of informative priors for predictive models. Neither makes an LLM's confident
number a measured property, nor establishes that a prior is well calibrated for these experiments.
The useful adaptation is to ask independent authors for assumptions, plausible ranges and
observable consequences before reconciling them. Preserve disagreements that describe different
mechanisms; agreeing after reading one another is not independent confirmation.
([SHELF](https://shelf.sites.sheffield.ac.uk/), official overview;
[AutoElicit](https://arxiv.org/abs/2411.17284), abstract only.)

Use a distribution over explicit scenarios when the uncertainty is “which physical explanation
applies.” Use within-scenario spread for unresolved variability or measurement uncertainty. A
mixture of laws is not equivalent to averaging their parameter vectors or their predicted
quantiles. In particular, preserve path identity: a member should carry one coherent history,
not choose a new unrelated physical world at every forecast time.

For scalar events, equal-weight Brier averaging satisfies, on the **same observation**,

`BS((p + q)/2, y) = (BS(p,y) + BS(q,y))/2 - (p-q)^2/4`.

For predictive CDFs with finite CRPS, linear pooling similarly gives

`CRPS((F+G)/2,y) = (CRPS(F,y)+CRPS(G,y))/2 - integral (F-G)^2 dz / 4`.

The second identity follows by expanding the squared-CDF definition, exactly as the first does
for squared scalar error. These are mathematical derivations, not experimental results. The gain
is relative to the **average** member loss, not necessarily the better member. “Different in
kind” is a scientific reason to retain scenarios, not an algebraic necessity or a performance
guarantee. Equal author weights avoid fitted mixing weights; they do not establish posterior
model probabilities. Cross-review with the inference-audit agent corrected an initial draft that
had mistakenly excluded this CRPS identity.

Proper scoring rules reward honest probabilistic forecasts in expectation under their conditions;
they do not turn a hand-picked spread into calibrated coverage. The primary Gneiting–Raftery
paper's metadata and abstract were retrieved, but the PDF failed to load here, so I do not claim
a full-paper reading. The identities above are independently derived from their definitions.
([Gneiting and Raftery](https://doi.org/10.1198/016214506000001437).)

Conformal prediction is also not a no-data calibration shortcut: its standard construction uses
calibration examples and exchangeability, with additional machinery/assumptions under shift.
Calling typed scenario coverage “distribution-free” would therefore be unjustified.
([Angelopoulos and Bates](https://arxiv.org/html/2107.07511v6), abstract and introductory material;
extensions not independently reviewed.)

A planned stop, loss of visibility and physical extinction must remain different events. A
forecast conditional on the actual future control history answers a different question from an
unconditional forecast at the cutoff. Unknown future controls should be scenarios, not silently
read observations. An observation model should allow “not visible” without forcing “extinct.”
These are semantic requirements for any family; this memo has not audited the raw instrument
files or established that the ACME as-run script is a genuinely prospective input.

## 5. The largest defensible next architecture

The following is my design proposal, informed by Fable's extension map rather than a new
experimental verification of its combustion claims.

**Shared property and apparatus foundation.** Introduce explicit, source-located fuel/diluent
property blocks with validity intervals, a common unit convention and tests of identity and
limits. Share the scientific definition and provenance across families. Do not silently retrofit
old releases to a new property table: each release must freeze the values/code it used. A
cross-family discrepancy then tests the closure as well as the constant; it does not automatically
prove the common constant wrong.

**FLEX methanol.** Separate liquid energy/storage, evaporation after flame loss, composition
(including the proposed water mechanism), radiative loss and observation state. A self-cooling
branch is a concrete independent mechanism; a composition-triggered extinction branch remains a
hypothesis until independently checked. Do not promote the inherited unverified methanol dossier's
absolute constants. Check water/mass/energy bounds and limiting behavior before train comparison.

**FLEX n-heptane.** Make hot, cool and unlit states carry their own energy and composition history.
The extension map's challenge to a universal post-cool-flame size clock is a reason to preserve
memory, not add a sharper size exponent. Treat alternative cool-flame termination explanations as
distinct scenarios until mechanism-specific evidence resolves them. Share transferable transport
properties with methanol while keeping fuel chemistry and transition assumptions explicit.

**ACME normal.** Separate the gas/flame state, burner thermal state, instrument observation and
administrative control termination. The proposed pressure placement and ignition head start are
competing physical explanations, not corrections established by this memo. A burner heat balance
can add meaningful state and apparatus coefficients; it should predict an intermediate thermal
signal before its termination forecast is credited with physical explanation.

**ACME inverse.** Retain mixture composition and transient transport explicitly. A flux/oscillation
clock whose supporting simulation has wrong temperatures should not be given a precise single
threshold. A held-flame and an extinction scenario can coexist without claiming the typed weights
are estimated probabilities. Add a mechanism only with a checkable intermediate implication, such
as the ordering of transport and heating times, not merely a better end-time match.

**BASS.** Separate latent front motion, available fuel, thermal history, oxygen delivery, imposed
flow, visibility and tracking termination. Use the oxygen-balance proposal as a steady-limit
check, not as a demonstrated transient closure. Uncertain fan response belongs in physically
named flow paths; unknown future operator changes are conditional scenarios. Do not strengthen
the contaminated quench boundary or turn one visible flame into certainty of future survival.

**Resolved transport as a later stage.** A conservative discretization with explicit integration
and typed coefficients can be legitimate WSP even if large. Before adopting it, require analytic
limit cases, conservation budgets, time/space convergence, and independent intermediate observables.
These distinguish solving the chosen equations correctly from choosing physically adequate
equations. NASA's V&V overview makes precisely that distinction; neither a finer grid nor passing
code tests constitutes physical validation.
([NASA CFD V&V](https://www.grc.nasa.gov/www/wind/valid/tutorial/overview.html), complete short overview.)

Do not let an adaptive numerical tolerance become an undisclosed parameter chosen to improve a
forecast score. Numerical refinement may measure discretization error at fixed scientific
parameters; it must not search for physical constants. No solver or coefficient-generating
construction is recommended for insertion without the separate strict-forward audit.

## 6. Implementation order and rejection criteria

1. Write down the observable, cutoff and future-control assumptions; resolve source admissibility
   before treating an input-driven improvement as scientific progress.
2. Add independently checked property/apparatus blocks and source/units/domain metadata. Require a
   concrete consumer; avoid a speculative universal library.
3. Add one named physical state or mechanism per new forward, preserving a neutral reduction to
   its predecessor when that reduction is mathematically meaningful.
4. Verify encoding, conservation/limits and numerical behavior on synthetic/analytic cases.
   Those checks establish implementation facts only.
5. Author literal candidates independently of diagnostic scripts; state every exposure and
   distinguish authored judgment from external transcription. Evaluation may reveal a problem;
   only the authorized author decides and writes any subsequent parameter change.
6. Compare specified intermediate implications and forecasts on train. Preserve failures. A
   contradictory intermediate observable can reject an explanation even if its aggregate forecast
   score is acceptable.
7. Freeze a scientifically justified candidate or committee and its adoption rule before any
   permitted aggregate validation look. A confidence interval crossing zero does not establish
   practical equivalence; equivalence requires its own margin and procedure.

There is no source-backed reason to target a parameter count. The strongest joint Fable/Astra
release would preserve distinct defensible mechanisms, shared independently checked properties
and explicit uncertainty, rather than hide disagreement inside averaged coefficients. Its
qualification remains experimental until the authorized evaluation establishes something more.

## Source-reading and verification ledger

All links above are primary author/institution/publisher sources. “Read” describes inspected
content, not a completed reproduction. Search snippets were used for discovery, not substituted
for an unread full paper. No literature search was directed at protected NASA per-test outcomes.

| Source | Reading extent here | Verification boundary |
|---|---|---|
| Takehi et al., primary WSP post | Selected experiment setups, Qwen/Gemma discussion, November 12 response | Source wording checked; logs/metrics not reproduced |
| Lehmann, 2611.16981v1 | Abstract, main findings, limitations, selected methods/redaction appendix | Private-game outcomes and released code on author report |
| Gross, 2611.11604v1 | Abstract, pipeline/self-check/model-checking methods and limitations | No model checker execution |
| RASP / Tracr | Primary arXiv abstracts | Compiler behavior beyond abstract on author report |
| Yarotsky, 1610.01145v3 | Proposition 1 and section 3.1 constructions, including bounded multiplication | No implementation or reproduction of theorem proofs |
| Transformer Circuits | Summary and privileged-basis discussion | No trained network inspected |
| Git Re-Basin | Primary arXiv abstract | Alignment algorithms and empirical results not reproduced |
| SHELF | Complete official overview | Detailed elicitation manual/software not used |
| AutoElicit | Primary arXiv abstract | Empirical claims not independently checked |
| Grünwald MDL tutorial | Introduction and selected two-part-coding passages | Not the complete 80-page tutorial; no numerical MDL analysis |
| Gneiting–Raftery | Primary bibliographic/abstract material; PDF retrieval failed | CRPS/Brier algebra independently derived; no full-paper claim |
| Conformal introduction | Abstract and introductory material/structure | No implementation or review of all shift extensions |
| Cantera species thermo | Overview and NASA polynomial description | No handbook/property coefficient verified or imported |
| NASA V&V overview | Complete short page | Supports methodology, not this model's physics |
| Local dossier README, HANDOFF_ASTRA, extension map, limits ledger; Iancu handoff | Direct reads; large combined tool output was partly truncated, followed by focused reads | No raw-data audit; numerical family findings remain Fable/delegates' reports |
| [Inference-audit collaborator](astra_uncertainty_and_freedom.md) | Complete draft read after direct messages and cross-review | CRPS correction jointly resolved; raw-data/implementation claims not independently verified |
| [Broader WSP source ledger](astra_weight_space_research.md) | Central source tables and concluding synthesis read; combined output partly truncated | Supports a wider reading queue; its source-reading/reproduction status remains that collaborator's report |

The existing `mit_wsp.md`, `related_work.md` and `legora.md` were requested in a combined read whose
output exceeded the tool budget. Their source pointers helped discovery; their full contents and
reproduction claims are **not** represented as independently read or verified in this memo.

## Addendum: truthful ancestry across a new specification

During implementation review, direct inspection of `assemble.rs` and
`ignis-core/src/contracts/wsp.rs` exposed a provenance gap in the existing workflow. Ordinary
materialization requires a revision's parent to share its specification. A parentless build is
classified as `LiteralOneShot`, whose manifest rejects evaluation-feedback/prior-candidate
exposure. An already exposed author cannot solve this by removing those exposures or renaming
them as sources. Calling such a root a reference baseline in prose does not change its manifest.

An explicit, separately named **new-spec full literal rewrite** route is a defensible narrow
extension of the lane's implementation. The existing serialized `LiteralRevision` contract
requires actual parent ancestry but does not itself require equal parent/child spec hashes.
Preserve the ordinary assembler's equal-spec rule. The new route should require a real literal
parent, the same scientific family, a new spec/model identity, full current exposure and explicit
destination assignments. It should construct destination tensors freshly from the destination
spec and complete literal patch; the parent supplies provenance, never silent tensor inheritance.

A temporary patch copy with only `parent_release_id` removed can invoke existing completeness
validation. It must retain all exposure fields and must never be built or saved as a root release.
The actual release uses the original patch and actual parent, yielding `LiteralRevision` /
`WeightEdit`, with the new spec and **data-informed** authorship. Thus experiment identity is new
while the coarse lineage category truthfully records revision/copying of prior work. The patch's
author is the current author; unchanged inherited blocks separately credit their original author.

This is a reviewed design recommendation, not a claim that this memo implemented or tested the
route. Required checks include complete destination coverage, real/matching parent identity,
literal and same-family ancestry, explicit parent exposure, rejection of same-spec misuse, fresh
materialization without parent-array inheritance, registry roundtrip and unchanged old releases.
Exact-copy claims and neutral forward equivalence require separate checks. No new model weights
were written during this review.

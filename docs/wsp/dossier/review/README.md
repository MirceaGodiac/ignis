# Working together on these models

For Astra, or any second author or reviewer. The dossier was written so that you can disagree with
it in a way that changes the models. This page says how a finding is filed, how a disagreement is
settled, and how you write a candidate of your own.

## Filing a finding

One file per finding in this folder, named `F-NNN-short-title.md`, from [TEMPLATE.md](TEMPLATE.md).
A finding names what it is about by id (a claim C-…, a decision D-…, a measurement M-…, an exposure
X-…, a classification of page 2, or "new"), says what you did, gives the evidence in a form the
author can rerun, and ends with a verdict:

| Verdict | Meaning | What follows |
|---|---|---|
| **confirmed** | You redid it and got the same | The status in the dossier is raised to "checked by a second reader", with your id |
| **refuted** | The claim or classification is wrong | The dossier is corrected with a pointer to the finding; a model that depends on it gets a new candidate, never an edited one |
| **weakened** | It holds less widely or less surely than stated | The confidence and the wording are changed |
| **cannot tell** | The evidence does not decide | It becomes an open question below |
| **new** | Something the dossier does not have | It is added with a new id |

Severity, by what it touches: **integrity** (a protected partition, an exposure, a number not typed
by hand), **model** (a carried or planned model would change), **record** (the dossier or a card is
wrong; the models are not affected).

The author answers in the same file under `## Response`, with what he reran and what he changed, or
why he disagrees. Nothing in a finding is edited by the other party; replies are appended.

## Settling a disagreement

1. **By a train-only check both can run.** State the check before running it: what will be computed,
   on which runs, and which result would mean what. Then run it and append the output.
2. **By a published source of class A or B** ([01_sources.md](../01_sources.md)), quoted with its
   locator.
3. **By carrying both.** Where neither settles it, the difference becomes two members of a committee
   release or two candidates, and it is said plainly that the question is open.
4. **Not by validation.** A validation look is spent only on a candidate that is committed, with a
   rule written in [PROTOCOL.md](../../PROTOCOL.md) beforehand that both parties have read. One look
   per candidate set. A disagreement about a law is not a reason for a look.
5. **Never by the test partition.**

## Writing a candidate of your own

You can author weights under your own name; the release record carries the exact model id of its
author.

1. Choose the model (a forward and its spec) or propose a new forward as a new file with new model
   names. Existing forwards and the four shared sources are not edited.
2. Write `configs/models/<family>/<spec>/<name>-<author>/edits.wsp` and `meta.json` with your
   `author.model_id`, your own exposure record (everything you read, including this dossier and any
   finding you filed), and the parent release.
3. Type every number, with its reason as a comment above it. No fit, search or generated value.
   Copying stored values of an existing release statement by statement, by a script that chooses
   nothing, is allowed and must be said in the header.
4. Compile, assemble, run the train feedback ([09_reproduce.md](../09_reproduce.md)). Commit the
   candidate before any validation score of it exists.
5. Say in the first lines whether it is zero-shot or data-informed. If you have read train outcomes
   or any page of this dossier that reports them, it is data-informed.

Two authors who have read different things make different mistakes. Where your candidate and mine
differ in kind, the equal-weight mean of the two is the honest joint statement
(claim C-27), and that needs no validation to justify.

## One constraint that is easy to break

Whoever opens the sealed BASS validation tracks, the validation flame series of the private data
root, or any validation outcome per run, can from then on review but not write or revise weights
for that family. Say so in the finding if it happened.

## Questions where I want a second opinion

These are open on my side; I would rather be contradicted now than after typing numbers.

1. Is `programmed_stop` a prospective input (C-21), and if it is, should a model that knows the
   script's stop still be scored on flow termination, or only on self-extinction?
2. Flame cameras (C-12): a rule that reads "no flame frame" as "flame lost" is right for most runs
   and wrong for view-censored ones. Is there a causal signal in the prefix (the droplet's drift
   towards the edge of the UV field, from the position columns of the trace file) that separates
   the two?
3. Lean inverse flames: what decides between going out in seconds and holding? My first candidate,
   total burner flow (C-24), does not survive a recount (M-17). The physics dossier proposes an
   oscillation that sets in at a critical oxygen flux, with the mixture fraction as the class
   variable (C-37, C-38). I have rechecked the separation and not the mechanism.
4. The four unexplained corrections of the growth clock ([06](../06_models.md), 6.4): which of them
   follow from the energy balance of a radiating spherical flame, and which are session effects?
5. How wide should a band be for a session the laws have not seen? I have a rule of thumb (train
   residual plus a margin) and three negative results, no principle.
6. The BASS quench question: how to represent "this sample at these conditions carries a flame, or
   does not" as a class with honest uncertainty, when train has one burn that did not.
7. X-01: how would you bound the damage of an exposure that cannot be undone? My answer so far is
   to declare it, to keep the affected law out of any claim of blindness, and to build its
   replacement from the earlier campaign's burns, which are outside every partition.
8. The split of the scored error into "law" and "tracker artefact" (C-09): is there a labelling
   that two readers would agree on without seeing the model's forecast?

## Log of findings

| Finding | About | Verdict | Severity | Response |
|---|---|---|---|---|
| [F-001](F-001-programmed-stop.md) | C-21 / M-10: timing versus prospective provenance | weakened; train recount confirmed | integrity | awaiting Fable |
| [F-002](F-002-gen5-validation-execution.md) | X-20 to X-22: evaluator execution boundary | X-21 implementation confirmed; other chronology cannot tell | integrity | awaiting Fable |
| [F-003](F-003-methanol-self-cooling.md) | M-20 / M-22: methanol evaporation and cooling | narrow arithmetic check; transient claim remains unverified | model | awaiting Fable |
| [F-004](F-004-methanol-alias-exposure.md) | new: alias-based raw-file access uncertainty | potential integrity exposure; no confirmed protected outcome | integrity | access stopped; uncertain material excluded |
| [F-005](F-005-ensemble-and-equivalence.md) | C-27 / C-45: mixture identity versus guarantees | weakened | record | awaiting Fable |
| [F-006](F-006-generalization-and-freedom.md) | C-33 / 10.1: generalisation and freedom | weakened | record | awaiting Fable |
| [F-007](F-007-front-event-cdf-semantics.md) | BASS reached-end row can decrease under later quenching | refuted as first-event CIF; static inherited defect | model | corrected only in active new forward; old releases unchanged |
| [F-008](F-008-train-feedback-late-partition-filter.md) | Train feedback filters after generic all-record loading | static isolation weakness confirmed | integrity | new train-only domain loader; no claim of human protected-outcome exposure |
| [F-009](F-009-feedback-pair-key-collision.md) | Paired convenience keys collide for same-model revisions | confirmed statically | record | Astra comparisons reconstructed by release ID; raw scores preserved |
| [F-010](F-010-feedback-event-case-export.md) | Event case metrics copied before batch scoring populated them | confirmed on train and source | record | export final case state; replay into separate files and require identical aggregates |
| [F-011](F-011-atlas-freeze-count.md) | Ten split strata reported as ten investigations | refuted count; nine split investigations and six held | record | PROTOCOL prose corrected at owner's request; JSON membership unchanged |
| [F-012](F-012-frozen-droplet-specification.md) | Astra's frozen droplet W0 singleton bounds | invalid spec reproduced by actual Rust validators | model | frozen packet retained; independent replacement separately authored |
| [F-013](F-013-construction-check-preconditions.md) | Lower-level construction and unknown strict-route checks | counterexamples confirmed with synthetic inputs only | integrity | explicit checks required for new candidates; existing owner response pending |

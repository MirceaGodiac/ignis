---
name: source-grounding
description: Answer documentary questions from the IGNIS corpus with claims bound to retrieved evidence; use whenever an answer states what a paper, report or dataset description says.
---

# Source grounding

## 1. Retrieve before you state

- Start with `sources_search` using the researcher's own technical terms (fuel, experiment, quantity,
  condition), then a reformulation with synonyms if the first page is thin. Use filters (experiment,
  source type) rather than longer queries.
- Search hits are pointers, not proof. When a hit is decisive, open it with `sources_read` at its
  locator to see the surrounding passage, table or figure caption; follow `next` when the passage
  continues.
- An empty or short page does not prove the corpus lacks the information (`total_count` may be
  absent). Say what you searched for before concluding that nothing was found.
- A backend reported as degraded in `backends` means the search was incomplete; say so.

## 2. Bind every statement

- One claim per checkable statement. `source_reports` for "source X reports ...", `observed` for a
  measured value with its evidence. Cite the `evidence_id` of the hit or segment that contains it.
- Keep numbers exactly as the source states them, with the source's unit; if you convert, state the
  conversion as a separate `computed` or `interpretation` claim.
- When sources disagree, report both with their evidence and say they conflict; do not average or
  pick one silently. Note differences in conditions (pressure, oxygen fraction, fuel, gravity level)
  that might explain the disagreement.

## 3. Know when to stop

- If the evidence answers only part of the question, submit `partial` and list the unanswered part in
  `limitations`.
- If the sources you found answer the question with "no effect reported" or "not observed", that is
  a null result: submit it as `complete` with claims bound to those sources. It is not an abstention.
- Reserve `insufficient_evidence` for missing, conflicting or incoherent evidence: say which searches
  you ran and name the specific source or quantity that would settle it. Never fill the gap from
  memory.

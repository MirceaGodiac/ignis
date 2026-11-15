---
name: source-grounding
description: Answer documentary questions from the IGNIS corpus with claims bound to retrieved evidence; use whenever an answer states what a paper, report or dataset description says.
---

# Source grounding

## 1. Retrieve before you state

- Start with `sources_search` using the researcher's own technical terms (fuel, experiment, quantity,
  condition). Use filters (`investigation_ids`, `source_ids`, `source_kinds`) rather than longer
  queries. Run as many searches as the question needs: reformulate with synonyms, abbreviations and
  the quantity's other names, search each sub-question separately, page on with `next_cursor` when
  the first page is relevant but not decisive.
- Every documentary search carries `"schema_version": "evidence.v2"` and `expected_terms`: before
  calling, write down 3 to 8 terms a relevant passage would contain that the query itself does not
  (equivalent phrasings, abbreviations, instruments, materials, the quantity's other names), each as
  an object with `term` and `importance`: `high` for the terms that discriminate, `medium`,
  or `low` for broad ones. Shape: `"expected_terms": [{"term": "extinction diameter", "importance": "high"},
  {"term": "radiative loss", "importance": "medium"}]`. Never put the answer you expect in a term.
  Terms the corpus does not hold are ignored silently (the page does not say which were kept): when
  hits stay off-topic, change the terms and the query.
- Cite the search hit directly when its snippet or quote holds the statement or the value: a hit
  is the citable evidence, and `sources_read` adds no evidence id. Read (`{"kind": "passage",
  "evidence_id": ...}`) only when the passage is cut off before what you need, or to check a
  decisive number in context. For a long report, `sources_outline` shows which pages hold results;
  search their wording with `filters.source_ids` to get the citable hit.
- Show the decisive passage with a `source_excerpt` view (`evidence_id` of the hit).
- An empty or short page does not prove the corpus lacks the information (`total_count` may be
  absent). Try other phrasings and filters, and say what you searched for before concluding that
  nothing was found.
- If a read passage is unreadable (letters shifted or substituted, e.g. `3FXEMRIH WMRKPI JYIP`,
  or runs of replacement characters), its parse view is damaged: do not quote, paraphrase or cite
  the damaged part. Cite only the readable lines, look for the same fact in another hit or source,
  and name the damaged page in `limitations`.
- A lockbox note in the lexical backend's detail is the session's standing rule (see "Lockbox" in
  your instructions), not a lost hit.
- A backend reported `unavailable` in `backends` means the search was degraded; say so.

## 2. Bind every statement

- One claim per checkable statement. `source_reports` for "source X reports ...", `observed` for a
  measured value with its evidence. What a hit carries is grounded, even when you also knew it:
  prefer it over a `background` claim. Cite the `evidence_id` of the sources_search hit whose passage
  contains it.
- Keep numbers exactly as the source states them, with the source's unit, and bind each in the
  claim's `values` as `{"value": 3.05, "unit": "mm", "source": {"kind": "evidence_quote",
  "evidence_id": ...}}`: the host checks the number and unit are written in that hit's own passage,
  so a number seen only in a neighbouring segment needs its own hit. If you convert, state the
  conversion as a separate `computed` claim from an analysis.
- For each source or dataset you rely on, name its PSI investigation id (PSI-nn), title and version
  as the tool returned them, and its DOI or NASA landing URL only when a tool returned one; never
  type a DOI from memory.
- When sources disagree, report both with their evidence and say they conflict; do not average or
  pick one silently. Note differences in conditions (pressure, oxygen fraction, fuel, gravity level)
  that might explain the disagreement.
- Before submitting, re-read the passage behind each decisive statement and check the claim says no
  more than it does.

## 3. When to stop

Stop on evidence, not on a count of calls.

- `complete` when every part of the question is answered, including a null result ("no effect
  reported", "not observed" in the sources found): bind it to those sources. It is not an abstention.
- `partial` only after the reasonable avenues for the missing part were tried (reformulations,
  filters, the key report's outline and pages, conditions, the graph); `limitations` names the
  unanswered part and what you tried.
- When the corpus is silent on a part you know reliably (what an experiment studied, which
  instrument it flew, a definition, a published conclusion in general), answer it as a `background`
  claim: no support, no values, never worded as "source X reports" and never with an invented
  citation, DOI or locator. Never state a specific flight run's outcome from memory.
- `insufficient_evidence` only when neither the evidence nor reliable background knowledge answers
  it: say which searches you ran and name the specific source or quantity that would settle it.
  Never present background as a retrieved source.

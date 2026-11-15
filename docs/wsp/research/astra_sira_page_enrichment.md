# SIRA-inspired page enrichment for the protected NASA corpus

2026-11-14. This is retrieval preparation, not a scientific model or a claim of
retrieval accuracy. The owner explicitly selected **GPT 6.1 Terra, xHigh** for
both page cards and enrichment in one pass. The consolidated instruction at
`3577073e7112a5f2ecebb3d0e5951d2ea9e4a5fa` governs the workflow.

## Primary-source reading

[Meta's SIRA paper, version 3](https://arxiv.org/html/2605.06647v3), sections
3.1–3.2, describes offline vocabulary enrichment and online query expansion.
Offline proposals fill lexical gaps with useful aliases and search terms;
corpus frequency rejects overly common proposals. Zero-frequency terms are
allowed offline because this step introduces new vocabulary. Online expansion
instead requires presence in the enriched index. The original query and filtered
expansion enter a weighted BM25 retrieval. This task prepares the offline side;
it does not implement a query agent or infer NASA performance from paper results.
The web return also included introduction and benchmark passages; no benchmark
number is used here or presented as an IGNIS result.

The [official repository](https://github.com/facebookresearch/sira/tree/57f9cbbe5d0a00c91a60f14f3532c8268edd6332)
was checked at commit `57f9cbbe5d0a00c91a60f14f3532c8268edd6332`:

- `scripts/add_doc_index_adapter.py`: proposal, filtering, trace/export and main
  enrichment path read; shard-merging tail only partly read. It keeps raw LLM
  responses and proposed/kept terms, and applies enrichment to separate postings.
- `scripts/configs/enrich/default.yaml`: read in full. Its reference n-gram
  maximum is four and its common-term threshold is five percent of corpus units.
  These are retrieval configuration, not physical model coefficients or a claim
  of optimal settings for NASA.
- `scripts/configs/enrich/prompts/doc_v06.txt` and `doc_citation_v01.txt`: read in
  full. They target absent, diverse retrieval vocabulary and adapt the prompt to
  the retrieval task. Academic citation-network expansion is not automatically
  appropriate to fire-safety document search.
- README: read. Its dataset-specific prompt/configuration caveat prevents treating
  its reported comparisons as a single universal configuration.

We do not adopt the upstream character truncation, fixed phrase-count cap or
resume rule that considers failed records done. The owner's full-page coverage,
explicit failures and auditable resumption take precedence. Original text is
preserved; generated indexing vocabulary never becomes documentary evidence.

## Our page-level adaptation

One no-tools Terra request receives consecutive extracted pages of one document
in an explicitly bounded batch. Figure/table-dominant pages also receive rendered
images supplied by the host. The model opens no file. Every page gets its own
stable source-bound row, including empty/unreadable pages and failures. Requests
pin model, xHigh effort, schema, prompt, source text/render identities and API
status/usage. A different model or reasoning setting is not a silent fallback.

In that same response, keep two meanings separate:

1. Source-grounded concepts, exact entities, instruments, materials, test mentions,
   quantities, units and abbreviations, with page evidence.
2. Generated retrieval cues: useful synonyms, alternate terminology and plausible
   search phrasings absent from the text, anchored to the underlying page concept.
   These are indexing suggestions, not added facts or predicted experimental results.

The offline exporter uses versioned normalization, phrase/ngram semantics and
corpus-unit definitions. Deduplicate identical physical page identities while
retaining their source-listing joins. Keep proposed, retained and rejected cues
with explicit reasons, and preserve original versus enriched postings separately.
Portable manifests/checksums and JSONL tables are the handoff for future VM
integration. No VM deployment or application search change is part of this pass.

## Qualification and access precede publication

The exact 200-page comparison sample was frozen before Terra output at
`62cd3a80dead2f1486e569b491acd232d7712d934b9e45acc84cc446ca813639`.
It contains the existing strong-labelled pages and additions selected by page
layout metadata. It is a convenience sample, not a random representative sample
of the complete archive. Strong and Terra labels remain mutually blind until
both sides are sealed. The strong reader stops at 200; it subsequently handles
only the expressly required risk/low-confidence/disagreement cases.

Report agreement per closed label kind with coverage, missing/failed rows,
confusion counts and positive/negative support. The owner's acceptance threshold
is at least nine pages in ten for that kind. Rare classes, correlated pages from
one document and a finite convenience sample limit the conclusion. A failed kind
requires an explicit definition revision and another recorded comparison; reusing
the same pages is disclosed. No historical strong labels are silently migrated
to fabricate newly requested enrichment. Free-form synonyms cannot be assessed
by exact equality to an absent reference string; their evidence grounding and
eventual retrieval usefulness are separate checks.

Terra never grants outcome access. A strong reviewer or the conservative rule
that uncertainty means outcome-bearing decides disclosure. That decision applies
to the full card, generated cues, postings and derived frequency statistics.
An index for an author must be constructed from an already authorized,
dependency-closed page projection **before** counting frequencies. Filtering
protected hits after retrieval would still expose protected indexing information.
The complete corpus products therefore remain private to custodians.

The nineteen narratives will consume distilled cards and selected summary or
conclusion pages. Their strong interpreters do not read whole reports in bulk.
Narratives state investigation-level findings with locators, never an individual
test outcome; source statements and interpretation remain distinct.

## Current execution status

An authenticated model-list request to the official OpenAI API returned the exact
`gpt-6.1-terra` identifier. The configured secret was used only in the request
header and never printed or persisted in a receipt. This establishes availability,
not a completed Terra inference or qualification result. The API batch runner,
versioned label-envelope migration and portable enrichment exporter are in
implementation; no agreement rate or full-corpus enrichment is claimed yet.

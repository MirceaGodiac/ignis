# The model passes over the corpus: what they do, what they cost, how to approve them

Status (2026-11-15): **built and tested on fixtures with a fake model; never run.** No model has been
called. Every command below is a dry run unless it gets `--execute`, and `--execute` needs the
user's approval of the spend.

Three Lane B tasks need a model to read the documents:

| Task | Output | Consumer |
|---|---|---|
| R7b section summaries | 1 to 3 sentences per leaf section (`GeneratedText`: model id + prompt hash) | `sources_outline` (navigation, never evidence) |
| R8 document labels | `SourceLabel` per document: type, families, investigations, run mentions, conditions | `KnowledgeService::labels`; later the graph (mentions, never identities) |
| R12b expansion vocabulary | up to 12 search terms per page | `expansion_fts` via `ignis evidence enrich` (never quoted or cited) |

## The combined pass (recommended)

`ignis evidence llm-pass --all [--model haiku] [--usd-per-mtok-in X --usd-per-mtok-out Y]`

- **One call per leaf section** of every document (outline from R7b). The call returns the section
  summary, the expansion terms of each of its pages, and, in the document's first section only,
  label pass `a`. A chunk is a paragraph or a table row, so one call per chunk would be ~10× the
  calls. A section gives the model enough context, and the static prefix (instructions + schema)
  is paid once per section.
- Without `--execute` it writes `combined_requests.jsonl` next to the catalog (the exact prompts,
  for review) and prints the **estimate**:
  - tokens = characters / 4;
  - output at each request's cap (an upper bound);
  - the prefix priced in full (prompt caching would lower it);
  - at the per-million-token prices you pass.
- With `--execute`:
  - it calls the model through `claude -p`, isolated: no tools, settings or MCP, and no inherited
    `CLAUDE*`/`ANTHROPIC*` variables, so it runs on the logged-in account;
  - it appends the answers to `combined_answers.jsonl`, and a rerun resumes, paying nothing twice;
  - it stops after 5 consecutive failures;
  - it splits the answers into `outline_summaries.jsonl` (read by `sources_outline`),
    `label_answers.jsonl` (label pass `a`) and an expansion pack `expansion-llm/`.
- Then:
  - `ignis evidence labels --all --execute` runs only the missing label pass `b` (independent: the
    outline plus the middle and last pages) and writes `source_labels.jsonl`;
  - `ignis evidence enrich <catalog dir>/expansion-llm` loads the vocabulary.

## Separate passes (if the combined one is not wanted)

- `ignis evidence outline-summaries --all [--parents]`: summaries only.
- `ignis evidence labels --all`: both label passes (`a`: first pages + outline; `b`: outline +
  middle + last pages).
- Expansion vocabulary has no separate pass. The existing pack (`/srv/ignis/data/enrichment/docs-v1`,
  PR #45) stays loadable.

## Rules the passes keep

- **Generated text is never evidence.** Summaries are attached as `GeneratedText` and only for
  documents where the scope's lockbox withholds nothing. Expansion terms are never returned as
  snippets.
- **Labels:**
  - A label is `resolved` only when both passes agree on the type and the investigations;
    otherwise it is `unresolved_conflict`, with the disagreement kept.
  - When pass `a` came from the combined prompt and pass `b` from the label prompt, the label's
    prompt hash names both.
  - Run names in a label are mentions, never resolved identities.
- **Labels are not used for claims until measured.** The gold set (below) gives their accuracy.

## Gold set (R8): what needs a person

1. `ignis evidence gold-select --n 100 [--seed 2026] [--out data/labels/gold/gold_template.jsonl]`
   writes ~100 documents. They are stratified across investigations and document kinds (kinds
   guessed from file names, only for stratifying), in a seeded, reproducible order. **Every label
   field is empty.** An existing file is never overwritten.
2. **A person labels the rows:** `doc_type`, `families`, `investigation_ids`, `runs`,
   `conditions_mentioned`, `labelled_by`, `labelled_at`. Code never fills them. Rows without
   `labelled_by` are not scored. The selected set and a labelling page
   (`data/labels/gold/labelling_sheet.html`: outline, PDF link, closed values, rules, export) are in
   `data/labels/gold/` (see its README).
3. `ignis evidence gold-eval --gold <file> [--labels <catalog dir>/source_labels.jsonl]` reports:
   - document-type accuracy, investigation-set exact-match accuracy, and F1 for families, run
     mentions and conditions;
   - the same over resolved labels only;
   - a confusion table.

   It refuses an unlabelled gold set.
4. Choose the model by these numbers: run `labels` with each candidate `--model` on the gold
   documents only (name them as sources), then compare.

## What to approve

1. Run the dry runs on the VM after the Docling import, and read their estimates:
   - `ignis evidence llm-pass --all`
   - `ignis evidence labels --all`
2. Approve a model and a budget.
3. Then run them with `--execute`. Start with the gold documents only, to measure before the full
   corpus.

Not done and needing a person: the gold labels, the model choice, and the approval of the spend.

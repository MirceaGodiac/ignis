# Terra page qualification and same-pass enrichment

`tools/wsp/astra/terra_pages.py` is a protected document-custody worker. It does
not author model weights, implement an agent loop, grant author access, or publish
bulk labels. The only scientific sources it may submit in this revision are the
exact frozen 200-page qualification sample. A synthetic smoke must pass with the
same model, effort, code, schema, authority, prompt and renderer identities first.

## Execution

```powershell
python tools/wsp/astra/test_terra_pages.py
python tools/wsp/astra/terra_pages.py --smoke
python tools/wsp/astra/terra_pages.py --smoke --execute --retry-failed
python tools/wsp/astra/terra_pages.py --execute --retry-failed --sample runs/dev/data_astra/work/terra_pages/qualification_200_62cd3a80dead2f1486e569b491acd232d7712d934b9e45acc84cc446ca813639.json
```

Without `--execute`, no credential or payload is opened and no network request
is made. The credential is `CODEX_API_KEY`, from the process environment or the
repository `.env`; it is never printed, copied into a request artifact, or stored
in a receipt. The official Responses endpoint is fixed. Every request explicitly
sets `model=gpt-6.1-terra`, `reasoning.effort=xhigh`, `tools=[]`,
`tool_choice=none`, `store=false`, and strict structured output. There is no model,
effort or provider fallback. There is no previous-response conversation chain.

The worker rejects incomplete provider responses, refusals, non-text output,
unexpected tool calls, and mismatched execution settings. It records provider
status, response/request IDs, actual model/effort and full usage objects. The
[Responses API](https://developers.openai.com/api/reference/resources/responses/methods/create)
defines these request controls; they are checked again in the actual response.

## Frozen blind sample

The sample preserves the existing 171 reviewed page IDs and adds 29 pages as
three consecutive runs of 10, 10 and 9 pages from previously unreviewed documents.
Selection used only page layout/status metadata. The strong labeller received
only the added IDs. The Terra worker has not read the strong labels, producer
receipts, comparison results or outcome classifications.

This is a convenience qualification sample, not a random representative corpus
sample. Both sample bytes and the source census are hash-pinned. All 200 pages
must remain in the qualification denominator. No failed or difficult page may
be replaced. Strong review stops at that sample except for the owner's explicit
low-confidence figure/table, disagreement and outcome-authority cases.

## Source flow and resource use

The host streams the selected census rows, verifies original document bytes and
any converted PDF, then verifies each complete extracted-text hash. Source text
is passed unchanged, without character or token truncation. The host passes only
catalogue identities and aliases: no group, partition, outcomes or prior labels.
The catalogue is supplied once per document batch, not repeated on every page.
Its complete source-byte hash and the extracted identity fields are retained.

Default batches hold up to 24 consecutive pages of one document. A gap or a new
document ends a batch. Batches split further to respect the documented request
payload limit; a single oversized page fails explicitly instead of truncating.
The image-input guide currently documents a
[512 MB request payload limit](https://developers.openai.com/api/docs/guides/images-vision).
Three independent API requests may be active; source documents are opened and
rendered sequentially by the host. These are resource controls, not corpus limits.

Images are added automatically using conservative source-layout metadata:
low/no extracted text, dominant raster area, narrow-column/table geometry, or
substantial vector graphics. Blank/unreadable candidates receive images so that
failure/blankness is not invented from missing text. Detected tables and failed
geometric detection are treated conservatively. Images are rendered at 150 DPI;
the exact PNG bytes and hashes are retained. Text-only pages receive no image.
The remote model can open no file and invoke no tool.

## Page cards and enrichment

Each response row covers one requested page and every requested closed label
kind. Host validation rejects missing/duplicate/extra pages, missing/duplicate
label kinds, vocabulary errors, unknown catalogue IDs, invalid locators and
inconsistent review decisions. Test mentions need their own exact evidence.

Text evidence uses a verbatim quote and occurrence number; the host computes and
checks character offsets against the already-supplied text. Image evidence must
refer to an image actually supplied for that page and use a normalized region.
Whole-page failure locators cannot support assigned or absent labels.

The same Terra call produces both the card and `search_enrichment`. Grounded
terms carry exact source evidence. Generated retrieval cues carry grounding IDs
and explicitly identified new vocabulary absent from the supplied page text.
Each novel term must occur in its generated cue; empty enrichment is preferable
to fabrication. Generated cues are retrieval hints, never measurements or facts
asserted by NASA. Original source text stays unchanged.

This follows the owner-requested offline SIRA principle: add useful query
vocabulary absent from the source while retaining the original document and
separating generated text from evidence. References are the
[SIRA paper](https://arxiv.org/html/2605.06647v3) and
[official implementation](https://github.com/facebookresearch/sira/tree/57f9cbbe5d0a00c91a60f14f3532c8268edd6332).
This worker does not copy an upstream text truncation, phrase quota, or
failed-as-complete resume policy. The separate artifact builder owns corpus-level
document-frequency filtering and portable search exports.

Enrichment and page labels share the actual assignment ID and prompt/tool/config
hashes. Free-form enrichment quality is not established by exact phrase agreement;
it requires a grounding/error review and later retrieval evaluation.

## Protection and failures

Every Terra page is conservatively protected with `reveals_outcome=true` and
`outcome_decision={method:conservative, assignment_id:null,
rule:terra_non_authorizing/1}`. This is a protection decision, not a claim about
the scientific content. Terra's own outcome suggestion is retained only in the
private provider output and cannot authorize release. Low/unresolved-confidence
figure/table pages require strong review. Nothing bypasses the dependency-closed
author gate.

`--retry-failed` permits one automatic repair round containing only failed rows;
accepted pages are never sent again in that round. Remaining failures stay
incomplete. Repeating the identical qualification command resumes by validating
accepted checkpoints and requesting only missing/failed pages. A changed code,
prompt, schema or authority identity is a new run, not a silent continuation.

All artifacts are under `runs/dev/data_astra/work/terra_pages/`. Each run retains
`config.json`, `schema.json`, `prompt.txt`, original request bytes, full provider
response bytes, receipts, row-validation results, supplied PNGs, accepted sealed
v3 page records, `runtime_receipts.json`, `failed_rows.json` and `summary.json`.
Hashes bind inputs, responses, configuration and source locators. Credentials are
excluded even if unexpectedly echoed in a provider response; such redaction is
explicitly recorded alongside the original response hash.

The summary never declares bulk qualification or author release. A separate
independent comparison must qualify each closed label kind at the owner's 0.90
resolved-agreement threshold on the full frozen 200-page denominator. Unresolved
agreement does not qualify a label. Any subsequent export must bind that host
qualification receipt to exact card bytes and accepted kinds.

## Verification status

The offline tests exercise exact model/effort/tool/storage settings, untruncated
text, evidence integrity, conservative authority, failed-row selection, page and
test identity checks, same-document batches, payload splitting, novel-term
grounding, and rejection of incomplete or incompatible provider responses.
Live smoke and qualification outcomes are recorded in private run summaries;
the existence of this worker or its passing tests is not a corpus-quality claim.

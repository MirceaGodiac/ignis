# Private SIRA page artifacts

`tools/wsp/astra/sira_artifacts.py` builds offline, portable artifacts from qualified
Terra v3 page cards. It does not call a model, implement a search runtime, choose
scientific parameters, grant `ExecutionScope`, deploy anything, or publish an index.
The implementation has been exercised on synthetic fixtures only. No live NASA
export or retrieval-quality result is claimed.

The owner authority is the whole-goal instruction at commit
, particularly steps 2 and 9. Page cards and
their enrichment come from the same Terra assignment. This artifact is the page
component of step 9; it does not invent the narratives, findings, discrepancy
verdicts, or release scorecards that other producers must supply.

## Reference and fixed configuration

The reference is [Meta SIRA, pinned commit
57f9cbbe5d0a00c91a60f14f3532c8268edd6332](https://github.com/facebookresearch/sira/tree/57f9cbbe5d0a00c91a60f14f3532c8268edd6332).
[Paper version 3, section 3](https://arxiv.org/html/2605.06647v3#S3) distinguishes
offline corpus enrichment, which may introduce vocabulary absent from the
original corpus, from later query enrichment, which requires positive frequency
in the enriched index. This implementation covers only the offline artifacts.

The [pinned enrichment configuration](https://github.com/facebookresearch/sira/blob/57f9cbbe5d0a00c91a60f14f3532c8268edd6332/scripts/configs/enrich/default.yaml)
provides the explicit defaults `max_n=4` and `max_df_ratio="0.05"`. They are
retrieval configuration, not scientific model weights. They are not tuned against
NASA outcomes. This implementation does not adopt the upstream input truncation,
phrase-count cap, or failed-request completion behavior. It uses a documented
tokenizer of its own and does not claim byte-equivalence with upstream BM25.

## Qualification and custody

`build_terra_private` is a Python API called by trusted custodian code. It requires:

- immutable cards, physical-page census, and receipt-verified document registry;
- the independently captured runtime receipts and current `label_records.Definitions`;
- independent expected hashes for each input and the qualification review;
- a trusted `host_check(VerifiedPage, pins)` callable returning exactly `True`
  only after it has checked that page's producer, source/dependency closure,
  applicable label qualification, and independent free-form grounding/error review.

The callback is supplied by host code; it cannot be loaded from JSON, a prompt,
or a CLI argument. A report saying `qualified=true` is not sufficient. Agreement
on closed labels does not qualify free-form cues. A refusal aborts the build;
it does not silently drop a page. Source-evidence pages cannot enter the explicit
synthetic-fixture mode. Ordinary use has no fixture fallback.

The adapter repeats v3 record validation against actual runtime receipts, matches
the source/page identity to the census, checks every full UTF-8 text hash, and
checks input snapshot hashes again before completing. It keeps file-name aliases
from the document registry, so the application can join by NASA filename and page
as well as by source hash and canonical page ID. Exact document aliases remain
available without multiplying physical pages in the frequency denominator.

Everything stays under `runs/dev/data_astra/work/sira_artifacts/`, including SQLite
state, page text, labels, generated terms, frequencies, postings, and exports.
Resolved paths must remain under their private roots; forbidden sealed path names
are rejected. A directory convention is not an OS access-control grant: the
custodian/app host must maintain its existing protected filesystem boundary.

**There is no author-view export.** Protected source dependencies make all derived
vocabulary and corpus statistics protected too. A future author index must start
from an already host-authorized, dependency-closed page projection and compute
its statistics on that projection. Filtering global search results afterwards
does not remove the leakage through vocabulary, DF, or rankings. Neither an input
file nor this artifact can grant a workspace, partition, or scope.

## Tokenization, frequency, and provenance

The versioned tokenizer applies Unicode NFKC, then case folding. Tokens are runs
of Unicode letters/numbers with following combining marks; all other characters
are separators. It does not stem, strip diacritics, remove stopwords, or reset at
sentence boundaries. The Unicode database version is in the configuration. Exact
UTF-8 source bytes are stored unchanged; normalized text is never substituted for
the original.

DF counts unique canonical physical page IDs containing an exact contiguous
normalized phrase. It is not an intersection of word bags. Long phrases are
matched against candidate pages using their indexed prefix and then their full
token sequence. Identical text on two different physical pages counts twice;
identical repeated rows for one physical page count once. A repeated page ID with
different content, source aliases, or text is an error. Alias page IDs cannot
replace the canonical source-hash/page-ordinal identity.

Each `retrieval_cues[].new_terms[]` is a proposal. Its full normalized phrase is
retained when it is nonempty, new to its page, and `DF <= max_df_ratio * N` on the
original corpus. The ratio comparison is exact rational arithmetic; zero means
only absent phrases pass. `DF=0` is eligible. Empty, duplicate, already-present,
and overly common proposals retain explicit rejection reasons. There is no
minimum corpus-frequency requirement at this offline stage.

A retained phrase contributes distinct contiguous atomic n-grams of lengths
1 through `max_n`. Frequency filtering applies to the proposed full phrase;
shorter constituent n-grams are not separately filtered. Generated postings are
binary per page/atom; source postings retain source term frequency. The two
posting sets remain separate. Enriched DF is their union by page, not their sum.
Any future query adapter must use the enriched DF with `DF>0`.

Grounded concepts/entities remain in `record.body.search_enrichment.grounded_terms`
with their exact locators. They are never silently promoted from generated cues.
Every proposal carries page ID, cue/term ordinal, page-record hash, source hash,
producer hash, original DF, and decision. Injection links connect each atom back
to those proposals. The enclosing v3 card retains assignment, prompt/tool/config
hashes, cue grounding IDs, locators, and dependencies. Generated cues are marked
as retrieval cues, never evidence or measurements.

## Files and deterministic handoff

SQLite bounds corpus memory; ingestion retains one page at a time, with an 8 MiB
SQLite cache target and disk-backed temporary tables. Runtime receipt metadata is
loaded as a mapping; original corpus text is not loaded wholesale. The host must
reserve sufficient storage within the shared acquisition budget. There is no
document/token/phrase cap or silent truncation.

An OS-held single-writer lock releases after a crash. Page ingestion commits
transactionally; retrying the same stream resumes verified pages. Unresolved
enrichment is not marked complete. Finalization freezes the population and
recomputes decisions in one transaction. A completed build cannot gain pages.
Changed code, definition code, Unicode version, settings, or input pins require
a new build directory.

`exports/<build_id>/` contains sorted, canonical UTF-8 JSONL files:

| File | Contents |
|---|---|
| `pages.jsonl` | Full v3 card, exact original text/hash, filename aliases, producer hash |
| `proposals.jsonl` | Every proposed term and retained/rejected reason |
| `original_postings.jsonl` | Original phrase postings with term frequency |
| `enriched_postings.jsonl` | Generated atomic phrase postings |
| `injection_links.jsonl` | Atom-to-proposal provenance |
| `document_frequencies.jsonl` | Separate original and enriched DF |
| `manifest.json` | Input/configuration pins, visibility, file hashes/bytes/counts |

The manifest is written last. Each file is streamed through a temporary file,
flushed, and renamed; matching existing files are preserved, conflicts fail.
Interrupted exports resume deterministically. The manifest says completion means
the entire supplied card stream, not a complete NASA corpus. No timestamp or local
absolute path enters the export identity. Export checksums, not SQLite bytes,
are the portable identity.

The application's lane may place this verified component under the protected
step-9 `runs/dev/data_astra/export/` handoff and include its manifest hash in the
whole-goal manifest. It must retain custody for labels outside train. This script
does not copy it there, transfer it to a VM, expose it to an author, or claim that
the other step-9 products exist. Before copying, the host also retains independently
pinned qualification/receipt/source manifests needed to audit the export.

Verification after an authorized private transfer uses the same repository layout:

```powershell
python tools/wsp/astra/sira_artifacts.py --execute --verify-private-export <private-export-directory>
```

Compare the returned manifest hash to the independently captured source-side
manifest hash. The command prints counts, hashes, and total bytes only. The CLI
has no build-with-untrusted-qualification shortcut; without `--execute` it reads
no artifact payload. Library exceptions belong to custodian logs, not author logs.

## Reproduction and limits

```powershell
python -m unittest discover -s tools/wsp/astra -p test_sira_artifacts.py -v
```

The 14 synthetic tests cover absent/common vocabulary, exact long-phrase matching,
Unicode duplicates, unchanged original text, physical-page duplicates/conflicts,
frequency boundaries, no truncation/cap, protected dependency preservation,
qualification refusal, exact grounding quotes, pinned Terra adapter inputs, filename joining, transactional
retry, interrupted/reordered export, tamper detection, and concurrent-writer refusal.
They do not establish useful retrieval, scientific correctness, or actual Terra
qualification. No test or validation outcome payload was opened to implement them.

This component files no scientific finding. The owner's per-finding independent
refutation requirement still applies to findings supplied to the later application
handoff; retaining a generated search cue does not create or confirm a finding.

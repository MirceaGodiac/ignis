# Private document page census

`tools/wsp/astra/document_pages.py` is a non-modelling custody and extraction tool.
All source identities, text, page labels, geometry and renderings remain in
`runs/dev/data_astra/work/document_pages/`. Model authors must not open that
directory's semantic payloads. It assigns no scientific label, test outcome or
model weight. `summary.json` contains aggregate operational counts and software
identity suitable for an author-facing status report.

Run from the repository root:

```powershell
python tools/wsp/astra/document_pages.py
python tools/wsp/astra/test_document_pages.py
python tools/wsp/astra/document_pages.py --execute --convert-office
python tools/wsp/astra/document_pages.py --execute
python tools/wsp/astra/document_pages.py --execute --render-page '<private page_id>' --dpi 150
```

The first command is a dry run and opens no source payload. The PDF extractor and
renderer are PyMuPDF 1.26.7 (MuPDF version is recorded at execution), installed
locally under the private work directory's `dependencies/`. A missing dependency
can be installed with:

```powershell
python -m pip install --target runs/dev/data_astra/work/document_pages/dependencies PyMuPDF==1.26.7
```

## Custody and joins

The tool snapshots selected metadata and complete acquisition receipt lines;
an unfinished final receipt line is counted and deferred. Malformed complete
receipt lines fail closed. Only DOC categories and the latest
`verified_download` or `verified_existing` receipt are accepted. Each selected
metadata identity must match the receipt, and the final incoming file's SHA-256
and byte count are recomputed before extraction. Raw Data, `.part` files,
paths outside `corpus/incoming`, and sealed directories are never consumed.
The downloader's files and receipts are never changed.

`documents.jsonl` contains one source row per selected document-category file,
including metadata, verification status, content hash and extraction receipt.
`pages.jsonl` contains one row per unique PDF content hash and physical page
ordinal. Equal bytes listed in multiple places share page rows; join their
`document_sha256` against all matching document rows to recover every source.
The page ID is `sha256:<original file hash>:page:<one-based ordinal, padded to six digits>`.
`source_locator.physical_page` is never an inferred printed page number. Office
conversions instead use `conversion_page` plus the converted PDF's SHA-256;
PowerPoint adds `slide_ordinal`. The converted artifact and converter receipt
are retained. Converted pagination is never represented as original PDF pagination.

Text lives in `documents/<sha256>/<ordinal>.txt`; each page records its text hash,
extractor/version, pipeline code hash, text status, image/vector-object counts,
dimensions and printed page label. It also records `semantic_labels_status =
not_assigned` and `requires_visual_review = true`. No page card is silently
treated as a completed semantic label. Downstream labels must use the atlas's
closed lists and evidence locators, and must keep outcome-containing labels
away from authors. The independent labeller reads this manifest and source join.

## Completeness and quarantine

Every physical PDF page is retained, including blank candidates, image-only
pages, vector-only pages and extraction errors. `low_text` means fewer than 40
non-whitespace characters: a conservative review signal, not a content limit or
a claim that the page lacks information. `blank_candidate` requires visual
confirmation. Text extraction alone is insufficient for figures or tables.
Render these pages before assigning labels; all pages remain flagged for visual
review because even dense text can coexist with outcome-bearing plots.

Unreadable PDFs and zero-page PDF containers have a document-level error receipt.
Zero pages is a coverage defect, never a successfully completed page census.
Per-page failures retain
the ordinal with an error status and empty text artifact. `quarantine.jsonl`
accounts for missing verified receipts, corrupt sources, unsupported paginated
formats, and nonpaginated formats. Quarantine is an index; originals are not
moved or deleted. Word and PowerPoint need the separately invoked Office
conversion described below. Formats without a successful conversion receipt
remain unsupported. Spreadsheets, CSV, plain text and
Igor files are nonpaginated; sheets are never claimed as PDF pages.

Global manifests are replaced after a completed census. A rerun incorporates
newly verified downloads and reuses per-document extraction only when the
source, software and code identities and cached text/manifest hashes match.
An acquisition in progress yields a snapshot census, not a complete-corpus claim.
Read `document_states`, both completeness flags and `page_extraction_errors`
before claiming coverage. `semantic_labelling_complete` remains false.

## Office conversion

`--convert-office --execute` uses installed Microsoft Office through a hidden
read-only worker, one source at a time. It refuses to start when that application's
process already exists. It closes only its own document and quits only its own
empty application, never killing processes. Word suppresses recent-file updates;
PowerPoint opens an untitled copy. Macros are force-disabled. Word link-at-open
and print-time field/link refresh are disabled and the prior option values are
restored before exit. PPTX preflight refuses external payload relationships
(ordinary hyperlink relationships are not loaded as payloads); no link-update
operation is called. Legacy binary PPT stays quarantined because this worker
cannot establish the external-link precondition safely.

PowerPoint exports every slide, including hidden slides, one per PDF page.
Word exports the full document. Converter-reported page/slide count must match
the resulting PDF page count. Original hash and size are checked again after
conversion. A receipt records application/version/build, script hash, original
hash, converted hash and bytes. Source files are never saved or modified.
Conversion fonts and pagination depend on the installed Office environment;
font substitution is explicitly `not_independently_verified`. Every converted
page still requires visual review. Original PDF pages and converted Office pages
are counted separately in the summary. Failed conversions are retained in
`conversion_receipts.jsonl`, with worker diagnostics kept private.

The conversion uses Microsoft's documented
[PowerPoint fixed-format export](https://learn.microsoft.com/en-us/office/vba/api/powerpoint.presentation.exportasfixedformat),
[Word fixed-format export](https://learn.microsoft.com/en-us/office/vba/api/word.document.exportasfixedformat),
and [Word link-update setting](https://learn.microsoft.com/en-us/office/vba/api/word.options.updatelinksatopen).
The PDF skill's operation marker is required once before a conversion authoring
operation, despite the originals remaining read-only.

## Resources and evidence

Extraction holds one document and one page's text at a time; manifest writes
stream. Rendering is requested for one page at a time and records a PNG hash,
source hash, page ID, DPI and renderer version. The storage budget reserves the
remaining bytes of the entire selected acquisition as well as current files
under the data root, then charges every output against the shared 40,000,000,000
byte ceiling. At least 1 GiB free space is preserved. Sources and media archives
are never removed. Cache checks retain only one document's page metadata in RAM.

Synthetic tests cover blank/vector pages, reproducible identities, source hash
and receipt mismatches, path confinement, partial receipt tails, corrupt and
unsupported sources, raw-data exclusion, duplicate-content joins, and rendering
that fails if source bytes change. Additional checks cover converted-page
provenance, external presentation payload links, and zero-page containers.
Fixtures are `synthetic_fixture`; they prove
custody/extraction invariants and are not NASA measurements or model evidence.

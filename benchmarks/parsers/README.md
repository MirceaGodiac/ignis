# Parser benchmark (R7a): which parser gives citable, box-accurate parse views of PSI PDFs?

Decision record for R7a, following the research method of `docs/implementation/LANES_V2.md` §3.
Measured 2026-11-14 on an Apple M5 (10 cores, 24 GB), same inputs for every parser.

## Evaluation

10 real NASA PDFs (702 pages) chosen for the hard cases, 19 gold items marked by hand
(`build_gold.py` → `gold.json`; every box checked by eye on the rendered page):

| Case | Document |
|---|---|
| raster scan | FLEX-2 Package Assembly drawing (1 p) |
| partly scanned slides | ACME s-Flame presentation (44 p, 8 pages without text) |
| table-heavy, long | BASS-II Summary Report (144 p) |
| table-heavy | FLEX-2 Radiometer Calibration Report (56 p) |
| long | FLEX TP, NTRS 20150023456 (314 p); ACME CFI-G Final Report (93 p) |
| slides | CFI Cool-Flame presentation (20 p); SPICE crew training (27 p) |
| engineering, vector text | SAME Aging Chamber drawing (1 p); SPICE Experiment Assembly (2 p, rotated 90°) |

Gold boxes come from the PDF itself (glyph boxes, image placements), so no parser's segmentation can
bias them; pages without a text layer have boxes marked on the rendering (`basis: visual`).
Scoring (`run_eval.py`): a passage is found when a unit contains the quote (box IoU, coverage,
spill = unit area / gold area); a cell when a table cell holds the value **in the right row and
column**; a figure when a picture unit overlaps the gold box with IoU ≥ 0.5.

## Results

| | MuPDF (current pipeline + PyMuPDF tables/images) | Docling 2.150.1 |
|---|---|---|
| gold items found | **11 / 19** | **14 / 18** (FLEX TP not scored: see below) |
| passages (text layer) | all found; boxes are paragraphs (spill 2–35×) | all found; tighter on slides (IoU 0.96 vs 0.42) |
| table cells, right row and column | 2 / 5 | **4 / 4** with a text layer (cell IoU 0.64–1.0) |
| figures | 2 / 3 | **3 / 3** |
| OCR (no text layer) | 0 / 6 (no OCR) | 2 / 6 (raster title block, rotated drawing stamp) |
| speed | 0.04 s/page | 1.2 s/page wall, 2.4 CPU s/page |
| peak memory | 0.9 GB | 6.9 GB |
| licence | **AGPL-3.0** (or commercial) | MIT (code), Apache-2.0 (layout model, RapidOCR) |

Note that the current Rust pipeline emits no tables, no figures and no OCR; the MuPDF column gives it
PyMuPDF's table finder and image placements as its best case.

Failure modes found (each handled in the sidecar or documented):
- **Native crash** (SIGSEGV) on the 314-page FLEX TP, not deterministic (the same 40-page batch
  crashed once and passed twice). Several conversions in one process make it likelier.
  → every batch runs in a fresh worker process, a crashed batch is retried twice, and a view with
  pages that still fail is refused at import (the document keeps its MuPDF view).
- **Large tables lose their structure** ("1024-step decode limit"; BASS-II test-by-test tables):
  the text is kept as one cell. → marked `structure: text_only`, imported as a `table` chunk, never
  as rows.
- **Huge drawings** exceed PIL's generic 179 Mpx guard. → a deliberate 400 Mpx bound for our own
  hashed NASA inputs.
- **OCR of vector-drawn drawing text and white-on-dark slides fails** (RapidOCR returns garbage or
  nothing; forcing full-page OCR or the English recognizer did not help). Rare in the corpus
  (engineering drawings); left to R10 if the T10 benchmark shows it matters.
- Neither parser gives **quote-level boxes**: both box whole paragraphs. A precise highlight of a
  quote needs character geometry (a follow-up on top of either view).

## Decision

**Docling becomes the parser of new parse views, as an offline sidecar** (`sidecar/docling/`);
the MuPDF views stay stored and readable.

- It wins where citations need structure: table cells in the right row and column, figures, raster OCR,
  and its licence (MIT/Apache) avoids AGPL.
- It runs offline, where documents are parsed, and writes a JSON view that
  `ignis evidence import-view` imports. **The server stays Rust-only**: no Python, PyTorch or 7 GB
  parser on the request path.
- Cost of a full corpus pass is a one-time batch: about 1.2 s/page on this machine.
- MuPDF remains the fallback for a document whose Docling view fails, and the existing citations
  against MuPDF views keep resolving (catalog v3: several views per publication, search on the
  active one).

GROBID (headers/references) was not run: Docker was not available; it is not needed for passage,
cell or figure locators and can be measured separately for bibliographic metadata.

## Reproduce

```bash
uv venv --python 3.12 .venv-parsers && uv pip install --python .venv-parsers/bin/python \
  docling==2.150.1 pymupdf==1.26.5
.venv-parsers/bin/python benchmarks/parsers/build_gold.py          # gold.json from the PDFs
.venv-parsers/bin/python benchmarks/parsers/run_eval.py mupdf
HF_HUB_OFFLINE=1 .venv-parsers/bin/python benchmarks/parsers/run_eval.py docling \
  --docs <key> --out docling-<key> --models <models dir>            # one document per run
.venv-parsers/bin/python benchmarks/parsers/merge_results.py docling
```

The PDFs come from `ignis sources fetch` (PSI) and NTRS; results are in `results/`.

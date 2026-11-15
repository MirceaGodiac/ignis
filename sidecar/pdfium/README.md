# PDFium sidecar

Produces **text-layer parse views** (`ignis.pdfium_view.v1`) for every PDF of the corpus: the PDF's
own text with paragraph and **line** boxes (normalized, top-left, page as displayed), no OCR, tables
or figures. About 3 ms per page; the 1,353 openable corpus PDFs (15,535 pages) take under a minute on
an M5. PDFium is BSD-3/Apache-2.0 (no AGPL, unlike MuPDF).

Documents that need structure (tables in the right row and column, figures, OCR of scans) also get a
Docling view (`sidecar/docling`). `ignis evidence import-view` imports both; **a PDFium view never
displaces a Docling view** (it is stored inactive: readable and resolvable, not searched).

```bash
uv run --project sidecar/pdfium python sidecar/pdfium/ignis_pdfium.py <out_dir> <pdf>...
uv run --project sidecar/pdfium python sidecar/pdfium/ignis_pdfium.py <out_dir> --from0 <NUL-separated list>
ignis evidence import-view <out_dir>/<inv>/<stem>.pdfium.json --pdf <the same pdf>
```

A page PDFium cannot load is listed in `failed_pages` and the import is refused (in the corpus:
two ACME papers of PSI-21). A file PDFium cannot open gets no view (three corpus files are not
readable PDFs as served by NASA: PSI-39 `FLEX-1248_Summary.pdf`, PSI-21 `ACME-DOC-010_D.pdf`, PSI-22
`Chien et al 2018 Electric-Field Effects on Flames in Microgravity.pdf`).

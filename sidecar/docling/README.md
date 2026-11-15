# Docling sidecar

Produces **Docling parse views** of PDFs as JSON (`ignis.docling_view.v1`). It is an offline batch
tool: it runs where documents are parsed, never on the request path. The Rust side imports the JSON:

```bash
ignis evidence import-view <view.json> --pdf <the same pdf>
```

The imported view becomes the PDF's active view; its MuPDF view (if any) stays stored, so citations
made against it still resolve. Why Docling, with numbers: [`benchmarks/parsers/README.md`](../../benchmarks/parsers/README.md).

## Setup

Requires Python 3.11–3.13 and [uv](https://docs.astral.sh/uv/). Versions are pinned in
`pyproject.toml`; changing one is a new parser version, hence a new parse view.

```bash
uv sync --project sidecar/docling
uv run --project sidecar/docling docling-tools models download layout tableformer rapidocr -o data/models/docling
```

If the Hugging Face download hangs (its Xet backend), set `HF_HUB_DISABLE_XET=1`, or fetch the files
of `docling-project/docling-layout-heron` and `docling-project/docling-models` (TableFormer) directly
into `data/models/docling/<repo with / replaced by -->`. Do not commit the models.

## Use

```bash
HF_HUB_OFFLINE=1 uv run --project sidecar/docling python sidecar/docling/ignis_docling.py \
    <pdf> <out.json> --models data/models/docling
```

Output: pages (size), items in reading order (text, list items, captions, section headers, footnotes,
table cells with row/column/spans), tables (with `structure: cells | text_only`), figures (with
captions), and `failed_pages`. Boxes are normalized `[x0, y0, x1, y1]` in `[0, 1]`, top-left origin,
on the 1-based physical page as displayed.

Robustness (from the benchmark): each 40-page batch runs in a fresh worker process and is retried
twice on a crash; pages that still fail are listed and the importer refuses the view. Large drawings
use a deliberate 400 Mpx image bound. Memory peaks near 7 GB.

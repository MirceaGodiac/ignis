"""IGNIS Docling sidecar: one PDF -> one parse-view JSON (`ignis.docling_view.v1`), R7a.

Offline batch tool: it runs where documents are parsed, never on the request path. The Rust side
(`ignis evidence import-view`) turns the JSON into a parse view of the catalog next to any existing
view of the same PDF (old citations keep resolving).

Measured choices (benchmarks/parsers, 10 PSI/NTRS PDFs, 19 gold items):
* Every batch of BATCH pages runs in a fresh worker process. Several conversions in one process end
  in a native crash (SIGSEGV) on the 314-page FLEX TP; the same batch alone succeeds, and the crash is
  not deterministic. A crashed batch is retried RETRIES times; a batch that still fails is listed in
  `failed_pages` and the importer refuses the view (the document keeps its MuPDF view).
* Large engineering drawings exceed PIL's generic 179 Mpx guard; inputs are NASA files we fetched and
  hashed, so the bound is a deliberate 400 Mpx.
* Boxes are converted at the producer: normalized to [0, 1], top-left origin, 1-based physical page
  as displayed. Docling's own boxes are bottom-left in points.
* A table whose structure Docling discarded ("decode limit"; it keeps the text as one cell) is marked
  `structure: text_only`, never presented as rows and columns.

Usage:
  uv run --project sidecar/docling python sidecar/docling/ignis_docling.py <pdf> <out.json> \
      [--models <dir>]
Models: `docling-tools models download layout tableformer rapidocr` (or the files fetched directly
from the same Hugging Face repositories); pass the directory with --models (HF_HUB_OFFLINE=1).
"""
import argparse, hashlib, json, subprocess, sys, tempfile
from importlib import metadata
from pathlib import Path

FORMAT = "ignis.docling_view.v1"
BATCH = 40
RETRIES = 2
MAX_PIXELS = 400_000_000


def converter(models, ocr=True):
    from PIL import Image
    Image.MAX_IMAGE_PIXELS = MAX_PIXELS
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    from docling.document_converter import DocumentConverter, PdfFormatOption
    opts = PdfPipelineOptions(do_ocr=ocr, do_table_structure=True)
    if models:
        opts.artifacts_path = models
    opts.accelerator_options = accelerator()
    return DocumentConverter(format_options={InputFormat.PDF: PdfFormatOption(pipeline_options=opts)})


def accelerator():
    """Device from IGNIS_DOCLING_DEVICE (auto | cpu | mps | cuda) and threads from
    IGNIS_DOCLING_THREADS. Several workers on one Apple GPU queue behind each other, so a mixed pool
    (some on mps, some on cpu) uses the machine better than all on the GPU."""
    import os
    from docling.datamodel.accelerator_options import AcceleratorOptions
    return AcceleratorOptions(device=os.environ.get("IGNIS_DOCLING_DEVICE", "auto"),
                              num_threads=int(os.environ.get("IGNIS_DOCLING_THREADS", "4")))


def round5(x):
    return round(float(x), 5)


def to_box(bbox, size):
    W, H = size
    b = bbox.to_top_left_origin(page_height=H)
    x0, x1 = sorted((b.l / W, b.r / W))
    y0, y1 = sorted((b.t / H, b.b / H))
    clamp = lambda v: round5(min(1.0, max(0.0, v)))
    return [clamp(x0), clamp(y0), clamp(x1), clamp(y1)]


def needs_ocr(path, a, b):
    """OCR only for a batch with a page without a text layer: 95% of document pages have text, and
    OCR on every page is the main cost."""
    import pypdfium2
    pdf = pypdfium2.PdfDocument(path)
    for i in range(a - 1, b):
        tp = pdf[i].get_textpage()
        try:
            if len(tp.get_text_range().strip()) < 50:
                return True
        finally:
            tp.close()
    return False


def batch_view(path, a, b, models):
    """Pages a..b of one PDF as view fragments (pages, items, tables, figures)."""
    from docling_core.types.doc import PictureItem, SectionHeaderItem, TableItem
    ocr = needs_ocr(path, a, b)
    doc = converter(models, ocr).convert(path, page_range=(a, b)).document
    sizes = {n: (p.size.width, p.size.height) for n, p in doc.pages.items()}
    for n in sizes:
        if not a <= n <= b:
            raise RuntimeError(f"page {n} outside the requested range {a}-{b}")
    pages = [{"page": n, "width": round5(w), "height": round5(h)} for n, (w, h) in sorted(sizes.items())]
    items, tables, figures = [], [], []
    for item, _ in doc.iterate_items():
        prov = (getattr(item, "prov", None) or [None])[0]
        if prov is None:
            continue
        page = prov.page_no
        box = to_box(prov.bbox, sizes[page])
        if isinstance(item, TableItem):
            t = len(tables)
            cells = [c for c in item.data.table_cells if c.bbox is not None]
            text_only = item.data.num_rows <= 1 or item.data.num_cols <= 1
            tables.append({"table": t, "page": page, "bbox": box, "n_rows": item.data.num_rows,
                           "n_cols": item.data.num_cols, "structure": "text_only" if text_only else "cells",
                           "caption": item.caption_text(doc) or None})
            for c in cells:
                items.append({"kind": "table_cell", "page": page, "bbox": to_box(c.bbox, sizes[page]),
                              "text": c.text, "table": t, "row": c.start_row_offset_idx,
                              "col": c.start_col_offset_idx,
                              "row_span": c.end_row_offset_idx - c.start_row_offset_idx,
                              "col_span": c.end_col_offset_idx - c.start_col_offset_idx,
                              "header": bool(c.column_header or c.row_header)})
        elif isinstance(item, PictureItem):
            figures.append({"figure": len(figures), "page": page, "bbox": box,
                            "caption": item.caption_text(doc) or None})
        elif getattr(item, "text", None):
            kind = "section_header" if isinstance(item, SectionHeaderItem) else str(item.label.value)
            items.append({"kind": kind, "page": page, "bbox": box, "text": item.text})
    return {"pages": pages, "items": items, "tables": tables, "figures": figures,
            "ocr_batches": [[a, b]] if ocr else []}


def run(pdf: Path, out: Path, models):
    import pypdfium2  # ships with Docling (Apache-2.0/BSD); only for the page count
    data = pdf.read_bytes()
    n = len(pypdfium2.PdfDocument(data))
    view = {"format": FORMAT,
            "parser": {"name": "docling", "version": metadata.version("docling"),
                       "docling_core": metadata.version("docling-core"),
                       "docling_ibm_models": metadata.version("docling-ibm-models"),
                       "layout_model": "docling-project/docling-layout-heron",
                       "table_model": "docling-project/docling-models tableformer accurate",
                       "ocr": {"engine": "rapidocr", "version": metadata.version("rapidocr"), "backend": "torch"},
                       "batch_pages": BATCH, "max_image_pixels": MAX_PIXELS, "ocr": "batches with a page without text layer",
                       "device": __import__("os").environ.get("IGNIS_DOCLING_DEVICE", "auto")},
            "source": {"file_name": pdf.name, "sha256": hashlib.sha256(data).hexdigest(), "n_pages": n},
            "pages": [], "items": [], "tables": [], "figures": [], "failed_pages": [], "ocr_batches": []}
    for a in range(1, n + 1, BATCH):
        b = min(n, a + BATCH - 1)
        frag = None
        for attempt in range(1 + RETRIES):
            with tempfile.NamedTemporaryFile(suffix=".json") as tmp:
                cmd = [sys.executable, __file__, "_batch", str(pdf), str(a), str(b), tmp.name, models or ""]
                done = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
                if done.returncode == 0:
                    frag = json.loads(Path(tmp.name).read_text())
                    break
                print(f"{pdf.name} pages {a}-{b}: worker exit {done.returncode} (attempt {attempt + 1})",
                      file=sys.stderr)
        if frag is None:
            view["failed_pages"] += list(range(a, b + 1))
            continue
        t0, f0 = len(view["tables"]), len(view["figures"])
        for t in frag["tables"]:
            t["table"] += t0
        for f in frag["figures"]:
            f["figure"] += f0
        for it in frag["items"]:
            if "table" in it:
                it["table"] += t0
        for k in ("pages", "items", "tables", "figures", "ocr_batches"):
            view[k] += frag[k]
    out.write_text(json.dumps(view, ensure_ascii=False) + "\n")
    print(json.dumps({"out": str(out), "pages": n, "items": len(view["items"]), "tables": len(view["tables"]),
                      "figures": len(view["figures"]), "failed_pages": len(view["failed_pages"])}))


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "_batch":
        _, _, pdf, a, b, out, models = sys.argv
        Path(out).write_text(json.dumps(batch_view(pdf, int(a), int(b), models or None)))
        return
    ap = argparse.ArgumentParser()
    ap.add_argument("pdf", type=Path)
    ap.add_argument("out", type=Path)
    ap.add_argument("--models", default=None)
    a = ap.parse_args()
    run(a.pdf, a.out, a.models)


if __name__ == "__main__":
    main()

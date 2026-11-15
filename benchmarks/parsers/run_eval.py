"""R7a parser benchmark: score a PDF parser on gold.json (LANES_V2 §3).

Every parser's output is converted to the IGNIS locator convention before scoring: boxes normalized
to [0, 1], top-left origin, on the 1-based physical page as displayed (after /Rotate).

Per gold item:
  passage  found = a unit on the page whose text contains the quote (case and whitespace folded);
           box: coverage = |gold ∩ unit| / |gold|, iou, spill = |unit| / |gold| (1 = tight)
  cell     found = a table cell with the value whose row holds the row header and whose column
           holds the column header above it (structure, not only text); box iou of that cell
  figure   found = a picture unit on the page with iou >= 0.5
Performance per document: wall seconds, process CPU seconds (all threads), pages; peak RSS.

Usage: python run_eval.py mupdf|docling [--docs key,key]   -> results/<parser>.json
"""
import argparse, json, re, resource, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import build_gold as bg  # noqa: E402

def fold(s):
    s = s.replace("­", "").replace("ﬁ", "fi").replace("ﬂ", "fl")
    return re.sub(r"[^0-9a-z]+", " ", s.lower()).strip()

def area(b):
    return max(0.0, b[2] - b[0]) * max(0.0, b[3] - b[1])

def inter(a, b):
    return area([max(a[0], b[0]), max(a[1], b[1]), min(a[2], b[2]), min(a[3], b[3])])

def iou(a, b):
    i = inter(a, b)
    u = area(a) + area(b) - i
    return i / u if u else 0.0

# ------------------------------------------------------------------ parsers -> units
# unit: {"kind": "text"|"cell"|"figure", "page": int, "bbox": [..], "text": str,
#        "table": int, "row": int, "col": int}

def run_mupdf(path):
    import pymupdf
    doc = pymupdf.open(path)
    units = []
    for page in doc:
        pno = page.number + 1
        W, H = page.rect.width, page.rect.height
        m = page.rotation_matrix
        def n(r):
            r = pymupdf.Rect(r) * m
            return [r.x0 / W, r.y0 / H, r.x1 / W, r.y1 / H]
        for b in page.get_text("dict")["blocks"]:
            if b.get("type") == 0:
                text = " ".join(s["text"] for l in b["lines"] for s in l["spans"])
                units.append({"kind": "text", "page": pno, "bbox": n(b["bbox"]), "text": text})
        try:
            tables = page.find_tables().tables
        except Exception:
            tables = []
        for ti, t in enumerate(tables):
            data = t.extract()
            for ri, row in enumerate(t.rows):
                for ci, cell in enumerate(row.cells):
                    if cell is None:
                        continue
                    txt = (data[ri][ci] or "") if ri < len(data) and ci < len(data[ri]) else ""
                    units.append({"kind": "cell", "page": pno, "bbox": n(cell), "text": txt,
                                  "table": ti, "row": ri, "col": ci})
        for x in page.get_images(full=True):
            for r in page.get_image_rects(x[0]):
                if r.width * r.height > 0.02 * W * H:
                    units.append({"kind": "figure", "page": pno, "bbox": n(r), "text": ""})
    return units, doc.page_count

def run_docling(path, models=None):
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    from docling.document_converter import DocumentConverter, PdfFormatOption
    from docling_core.types.doc import TableItem, PictureItem
    opts = PdfPipelineOptions(do_ocr=True, do_table_structure=True)
    if models:
        opts.artifacts_path = models
    # Large engineering drawings render past PIL's generic 179 Mpx "decompression bomb" guard. Our
    # inputs are NASA files we fetched and hashed, so the bound is raised to a deliberate 400 Mpx.
    from PIL import Image
    Image.MAX_IMAGE_PIXELS = 400_000_000
    import pymupdf
    total = pymupdf.open(path).page_count
    # Long documents are converted in page batches: bounded memory, and the 314-page FLEX TP crashes
    # natively when converted whole.
    # Each batch runs in a fresh worker process: several conversions in one long-lived process end
    # in a native crash (segfault) on the 314-page FLEX TP, while every batch alone succeeds.
    import subprocess, tempfile
    units = []
    for a in range(1, total + 1, BATCH):
        b = min(total, a + BATCH - 1)
        with tempfile.NamedTemporaryFile(suffix=".json") as out:
            cmd = [sys.executable, __file__, "_batch", str(path), str(a), str(b), out.name, models or ""]
            done = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
            if done.returncode != 0:
                tail = "\n".join(done.stderr.strip().splitlines()[-6:])
                raise RuntimeError(f"{path.name} pages {a}-{b}: worker exit {done.returncode}\n{tail}")
            units += json.loads(Path(out.name).read_text())
    return units, total


def _batch_worker(path, a, b, out, models):
    from PIL import Image
    Image.MAX_IMAGE_PIXELS = 400_000_000
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    from docling.document_converter import DocumentConverter, PdfFormatOption
    opts = PdfPipelineOptions(do_ocr=True, do_table_structure=True)
    if models:
        opts.artifacts_path = models
    conv = DocumentConverter(format_options={InputFormat.PDF: PdfFormatOption(pipeline_options=opts)})
    doc = conv.convert(path, page_range=(int(a), int(b))).document
    Path(out).write_text(json.dumps(_docling_units(doc)))

BATCH = 40

def _docling_units(doc):
    from docling_core.types.doc import TableItem, PictureItem
    sizes = {n: (p.size.width, p.size.height) for n, p in doc.pages.items()}
    def n(prov):
        W, H = sizes[prov.page_no]
        b = prov.bbox.to_top_left_origin(page_height=H)
        return [b.l / W, b.t / H, b.r / W, b.b / H]
    units = []
    for ti, item in enumerate(doc.tables):
        for c in item.data.table_cells:
            if c.bbox is None:
                continue
            W, H = sizes[item.prov[0].page_no]
            b = c.bbox.to_top_left_origin(page_height=H)
            units.append({"kind": "cell", "page": item.prov[0].page_no, "bbox": [b.l / W, b.t / H, b.r / W, b.b / H],
                          "text": c.text, "table": ti, "row": c.start_row_offset_idx, "col": c.start_col_offset_idx})
    for item, _ in doc.iterate_items():
        for prov in getattr(item, "prov", []) or []:
            if isinstance(item, PictureItem):
                units.append({"kind": "figure", "page": prov.page_no, "bbox": n(prov), "text": ""})
            elif isinstance(item, TableItem):
                continue
            elif getattr(item, "text", None):
                units.append({"kind": "text", "page": prov.page_no, "bbox": n(prov), "text": item.text})
    return units

# ------------------------------------------------------------------ scoring

def score(item, units):
    page = [u for u in units if u["page"] == item["page"]]
    g = item["bbox"]
    if item["kind"] == "figure":
        best = max((iou(g, u["bbox"]) for u in page if u["kind"] == "figure"), default=0.0)
        return {"found": best >= 0.5, "iou": round(best, 3)}
    if item["kind"] == "passage":
        q = fold(item["quote"])
        hits = [u for u in page if u["kind"] in ("text", "cell") and q in fold(u["text"])]
        if not hits:
            return {"found": False}
        u = max(hits, key=lambda u: iou(g, u["bbox"]))
        return {"found": True, "iou": round(iou(g, u["bbox"]), 3),
                "coverage": round(inter(g, u["bbox"]) / area(g), 3),
                "spill": round(area(u["bbox"]) / area(g), 2)}
    # cell: value, its row's header and its column's header, in the same table
    cells = [u for u in page if u["kind"] == "cell"]
    v, rh, ch = fold(item["text"]), fold(item["row_header"]), fold(item["col_header"])
    best = {"found": False, "text_found": any(fold(u["text"]) == v for u in cells)}
    for u in cells:
        if fold(u["text"]) != v:
            continue
        same = [c for c in cells if c["table"] == u["table"]]
        row_ok = any(rh in fold(c["text"]) for c in same if c["row"] == u["row"])
        col_ok = any(ch in fold(c["text"]) or fold(c["text"]) in ch and len(fold(c["text"])) > 3
                     for c in same if c["col"] == u["col"] and c["row"] < u["row"])
        if row_ok and col_ok:
            return {"found": True, "text_found": True, "iou": round(iou(g, u["bbox"]), 3)}
    return best

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("parser", choices=["mupdf", "docling"])
    ap.add_argument("--docs", default="")
    ap.add_argument("--models", default=None)
    ap.add_argument("--out", default=None, help="results file stem (default: the parser name)")
    a = ap.parse_args()
    gold = json.loads((HERE / "gold.json").read_text())
    keys = [k for k in gold["documents"] if not a.docs or k in a.docs.split(",")]
    run = {"mupdf": run_mupdf, "docling": lambda p: run_docling(p, a.models)}[a.parser]
    perf, items = {}, []
    for k in keys:
        t0, c0 = time.perf_counter(), time.process_time()
        units, pages = run(bg.path(k))
        perf[k] = {"pages": pages, "wall_s": round(time.perf_counter() - t0, 2),
                   "cpu_s": round(time.process_time() - c0, 2), "units": len(units)}
        for it in gold["items"]:
            if it["doc"] == k:
                items.append({"id": it["id"], "kind": it["kind"], "basis": it["basis"], **score(it, units)})
        print(k, perf[k], flush=True)
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 2**20  # macOS: bytes
    out = {"parser": a.parser, "items": items, "perf": perf, "peak_rss_mb": round(peak)}
    (HERE / "results").mkdir(exist_ok=True)
    (HERE / "results" / f"{a.out or a.parser}.json").write_text(json.dumps(out, indent=1) + "\n")
    found = sum(i["found"] for i in items)
    print(f"{a.parser}: {found}/{len(items)} gold items found; peak RSS {out['peak_rss_mb']} MB")

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "_batch":
        _batch_worker(*sys.argv[2:7])
    else:
        main()

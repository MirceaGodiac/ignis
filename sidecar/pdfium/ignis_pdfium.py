"""IGNIS PDFium producer: PDFs -> text-layer parse views (`ignis.pdfium_view.v1`), R7a.

The fast view for every PDF of the corpus (about 3 ms per page): the PDF's own text layer with
geometry, no OCR, no tables, no figures. Documents that need structure get a Docling view as well
(`sidecar/docling`); `ignis evidence import-view` keeps a Docling view active over a PDFium one.

Output, same shape as a Docling view so one importer reads both:
* `items`: one `text` item per paragraph in the PDF's character order, each with its `lines`
  (text, box, and [start, end) of the line in the item text, in Unicode scalars), so a quote can be highlighted at line precision instead of a whole paragraph.
* Boxes are normalized `[x0, y0, x1, y1]` in [0, 1], top-left origin, on the 1-based physical page
  **as displayed** (the page's /Rotate applied), like Docling views.
* `pages_without_text`: pages whose text layer is empty (scans): they have no items here; OCR is a
  Docling view's job.
* Whitespace: PDFium's tabs, carriage returns and no-break spaces become single spaces.
* A PDF that PDFium cannot open is reported on stderr and gets no view (exit status 2); a page it
  cannot load is listed in `failed_pages` and the importer refuses the view.

Usage:
  python sidecar/pdfium/ignis_pdfium.py <out_dir> <pdf>...     # writes <out_dir>/<stem>.pdfium.json
  python sidecar/pdfium/ignis_pdfium.py <out_dir> --from0 <list>  # NUL-separated list of PDFs
"""
import hashlib, json, sys
from pathlib import Path

import pypdfium2 as pdfium

FORMAT = "ignis.pdfium_view.v1"
# A new line starts when the next glyph's vertical centre leaves the current line by more than this
# fraction of the line height (PDFium usually marks line ends itself, with "\r\n").
LINE_JUMP = 0.6
# A new paragraph starts when the gap between two lines exceeds this many line heights.
PARA_GAP = 0.8


def r5(x):
    return round(min(1.0, max(0.0, float(x))), 5)


def displayed(page):
    """Map an unrotated PDF point (x, y) to normalized displayed coordinates (top-left origin)."""
    l, b, r, t = page.get_cropbox()
    w, h = (r - l) or 1.0, (t - b) or 1.0
    rot = page.get_rotation() % 360

    def f(x, y):
        u, v = (x - l) / w, (t - y) / h
        if rot == 90:
            return 1 - v, u
        if rot == 180:
            return 1 - u, 1 - v
        if rot == 270:
            return v, 1 - u
        return u, v

    def box(cb):
        x0, y0, x1, y1 = cb
        (a, c), (e, g) = f(x0, y0), f(x1, y1)
        return [r5(min(a, e)), r5(min(c, g)), r5(max(a, e)), r5(max(c, g))]

    size = (h, w) if rot in (90, 270) else (w, h)
    return box, size


def union(bs):
    return [min(b[0] for b in bs), min(b[1] for b in bs), max(b[2] for b in bs), max(b[3] for b in bs)]


def page_lines(page):
    """Lines of the page: (text, box) in PDFium's character order."""
    box, _ = displayed(page)
    tp = page.get_textpage()
    try:
        n = tp.count_chars()
        text = tp.get_text_range(0, n) if n else ""
        lines, cur, boxes = [], [], []

        def flush():
            s = " ".join("".join(cur).split())
            if s and boxes:
                lines.append((s, union(boxes)))
            cur.clear()
            boxes.clear()

        for i, ch in enumerate(text[:n]):
            if ch == "\n":
                flush()
                continue
            if ch in "\r\t\xa0" or ch.isspace():
                cur.append(" ")
                continue
            b = box(tp.get_charbox(i))
            if b[2] <= b[0] and b[3] <= b[1]:
                cur.append(ch)
                continue
            if boxes:
                lb = union(boxes)
                lh = max(lb[3] - lb[1], 1e-4)
                cy = (b[1] + b[3]) / 2
                if cy < lb[1] - LINE_JUMP * lh or cy > lb[3] + LINE_JUMP * lh:
                    flush()
            cur.append(ch)
            boxes.append(b)
        flush()
        return lines
    finally:
        tp.close()


def paragraphs(lines):
    paras, cur = [], []
    for s, b in lines:
        if cur:
            pb = cur[-1][1]
            lh = max(pb[3] - pb[1], 1e-4)
            gap = b[1] - pb[3]
            if gap > PARA_GAP * lh or gap < -3 * lh:  # a gap, or a jump back up (next column)
                paras.append(cur)
                cur = []
        cur.append((s, b))
    if cur:
        paras.append(cur)
    return paras


HYPHEN = ("\ufffe", "\x02")  # PDFium's marks for a word broken at the line end


def join(para):
    """Paragraph text and each line's [start, end) in it (Unicode scalars). A word hyphenated at a
    line end is joined without the mark; other lines are joined with one space."""
    text, lines = "", []
    for k, (s, b) in enumerate(para):
        broken = s.endswith(HYPHEN)
        s = s.rstrip("".join(HYPHEN)) if broken else s
        start = len(text)
        text += s
        lines.append({"text": s, "start": start, "end": len(text), "bbox": b})
        if k + 1 < len(para) and not broken:
            text += " "
    return text, lines


def view(path):
    data = Path(path).read_bytes()
    pdf = pdfium.PdfDocument(data)
    try:
        pages, items, empty, failed = [], [], [], []
        for i in range(len(pdf)):
            try:
                page = pdf[i]
            except pdfium.PdfiumError:  # listed; the importer then refuses the view
                failed.append(i + 1)
                pages.append({"page": i + 1, "width": 0.0, "height": 0.0})
                continue
            try:
                _, (w, h) = displayed(page)
                pages.append({"page": i + 1, "width": round(w, 3), "height": round(h, 3)})
                lines = page_lines(page)
                if not lines:
                    empty.append(i + 1)
                for para in paragraphs(lines):
                    text, ls = join(para)
                    if text.strip():
                        items.append({"kind": "text", "page": i + 1, "bbox": union([b for _, b in para]),
                                      "text": text, "lines": ls})
            finally:
                page.close()
        return {
            "format": FORMAT,
            "parser": {
                "name": "pdfium",
                "version": str(pdfium.version.PYPDFIUM_INFO),
                "pdfium": str(pdfium.version.PDFIUM_INFO),
                "line_jump": LINE_JUMP,
                "paragraph_gap": PARA_GAP,
            },
            "source": {"file_name": Path(path).name, "sha256": hashlib.sha256(data).hexdigest(), "n_pages": len(pdf)},
            "pages": pages,
            "items": items,
            "tables": [],
            "figures": [],
            "failed_pages": failed,
            "pages_without_text": empty,
        }
    finally:
        pdf.close()


def main(argv):
    out = Path(argv[0])
    if argv[1] == "--from0":
        paths = [p for p in Path(argv[2]).read_bytes().decode().split("\0") if p]
    else:
        paths = argv[1:]
    bad = 0
    for p in paths:
        p = Path(p)
        dst = out / p.parent.name / (p.stem + ".pdfium.json")
        try:
            v = view(p)
        except Exception as e:  # unopenable PDF: no view, reported
            print(f"fail\t{p}\t{type(e).__name__}: {e}", file=sys.stderr)
            bad += 1
            continue
        dst.parent.mkdir(parents=True, exist_ok=True)
        tmp = dst.with_suffix(".tmp")
        tmp.write_text(json.dumps(v, ensure_ascii=False))
        tmp.replace(dst)
    return 2 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

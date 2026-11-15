"""R7a parser benchmark: gold items for 10 NASA PSI/NTRS PDFs (LANES_V2 §3 research method).

Boxes are normalized [x0, y0, x1, y1] in [0, 1], top-left origin, on the 1-based *physical* page as
displayed (after /Rotate). `basis` says where a gold box comes from:
  glyphs        union of the PDF's own glyph boxes for the exact text (no parser segmentation)
  image_object  the placement rectangle of a raster image object in the PDF
  visual        marked by eye on a 60 dpi rendering (pages without a text layer); approximate
Run: python build_gold.py  (needs PyMuPDF and the files under data/raw, fetched with `ignis sources fetch`).
"""
import json, re, sys
from pathlib import Path
import pymupdf

ROOT = Path(__file__).resolve().parents[2]
R = ROOT / "data/raw/samples"
DOCS = {
    "cfig": ("PSI-159", "PSI-159_Reports_FinalRpt_ACME.CFI-G.pdf", "long"),
    "bass2": ("PSI-25", "PSI-25_Reports_BASS-II Summary Report.pdf", "table-heavy,long"),
    "radiometer": ("PSI-68", "PSI-68_Calibration_Radiometer Calibration Test Report, Fluids.pdf", "table-heavy"),
    "flextp": ("NTRS-20150023456", "NTRS-20150023456.pdf", "long"),
    "cool": ("PSI-39", "PSI-39_Presentations_Aspects of Cool-Flame Supported Droplet Combustion in Microgravity_CFI.pdf", "slides"),
    "ipc": ("PSI-107", "PSI-107_Presentations_IP-C Crew training 2010.pdf", "slides"),
    "sflame": ("PSI-23", "PSI-23_Presentations_20120015928.pdf", "scanned-pages"),
    "same_dwg": ("PSI-102", "PSI-102_Engineering Documents_60089MFHA150-B Aging Chamber Assembly.pdf", "engineering,vector-text"),
    "spice_dwg": ("PSI-107", "PSI-107_Engineering Documents_60054MAFH000-C SPICE Experiment Assembly.pdf", "engineering,rotated"),
    "flex2_dwg": ("PSI-68", "PSI-68_Engineering Documents_67212MFAN11000-4 FLEX-2 Package Assembly.pdf", "scanned,engineering"),
}

def path(key):
    inv, name, _ = DOCS[key]
    if inv.startswith("NTRS"):
        return ROOT / "data/raw/evidence/ntrs" / name
    hits = sorted((R / inv).glob(name.split(".pdf")[0][:60] + "*"))
    if not hits:
        sys.exit(f"missing {name}: fetch it first")
    return hits[0]

def norm(r, page):
    W, H = page.rect.width, page.rect.height
    return [round(r.x0 / W, 5), round(r.y0 / H, 5), round(r.x1 / W, 5), round(r.y1 / H, 5)]

def union(rs):
    r = pymupdf.Rect(rs[0])
    for x in rs[1:]:
        r |= x
    return r

def glyphs(page, quote, near=None):
    """Union of the glyph boxes of `quote` (one occurrence; the one nearest `near` if several)."""
    hits = page.search_for(quote, quads=False)
    if not hits:
        sys.exit(f"{quote!r} not found on page {page.number + 1}")
    # search_for returns one rect per line of a multi-line hit; group consecutive lines
    groups, cur = [], [hits[0]]
    for h in hits[1:]:
        if h.y0 - cur[-1].y1 < 2 * cur[-1].height and len(quote) > 40:
            cur.append(h)
        else:
            groups.append(cur); cur = [h]
    groups.append(cur)
    if near is not None:
        groups.sort(key=lambda g: abs(union(g).y0 - near.y0) + abs(union(g).x0 - near.x0) * 0.0)
    return union(groups[0])

def cell_box(page, value, row_header, col_header):
    row = page.search_for(row_header)[0]
    col = page.search_for(col_header)[0]
    cands = [h for h in page.search_for(value) if abs(h.y0 - row.y0) < row.height * 1.5]
    cands.sort(key=lambda h: abs((h.x0 + h.x1) / 2 - (col.x0 + col.x1) / 2))
    if not cands:
        sys.exit(f"cell {value!r} not found in row {row_header!r}")
    return cands[0]

def image_box(page, i=0, min_frac=0.05):
    rs = [r for x in page.get_images(full=True) for r in page.get_image_rects(x[0])]
    rs = [r for r in rs if r.width * r.height > min_frac * page.rect.width * page.rect.height]
    rs.sort(key=lambda r: (r.y0, r.x0))
    return rs[i]

def main():
    items = []
    def add(id_, key, pno, kind, box, basis, **kw):
        inv, name, cases = DOCS[key]
        items.append(dict(id=id_, doc=key, investigation=inv, file_name=name, cases=cases, page=pno,
                          kind=kind, bbox=box, basis=basis, **kw))
    d = {k: pymupdf.open(path(k)) for k in DOCS}
    def P(k, n): return d[k][n - 1]

    # --- born-digital: glyph and image-object geometry
    p = P("cfig", 21)
    add("cfig-passage", "cfig", 21, "passage", norm(glyphs(p, "is only nominal where the pressure increases"), p), "glyphs",
        quote="is only nominal where the pressure increases")
    add("cfig-cell", "cfig", 21, "cell", norm(cell_box(p, "12", "21137", "ignited"), p), "glyphs",
        text="12", row_header="21137", col_header="Hot flames ignited")
    p = P("cfig", 7)
    add("cfig-figure", "cfig", 7, "figure", norm(image_box(p), p), "image_object", caption_start="Figure 2: Design of the Advanced Combustion")
    add("cfig-passage2", "cfig", 7, "passage", norm(glyphs(p, "provides common mounting for a variety of"), p), "glyphs",
        quote="provides common mounting for a variety of")
    p = P("bass2", 55)
    add("bass2-passage", "bass2", 55, "passage", norm(glyphs(p, "with diameters of 6.35, 9.525,"), p), "glyphs",
        quote="with diameters of 6.35, 9.525,", page_label="47")
    add("bass2-cell", "bass2", 55, "cell", norm(cell_box(p, "4", "Samples burnt", "0.009525"), p), "glyphs",
        text="4", row_header="Samples burnt", col_header="0.009525")
    p = P("radiometer", 11)
    add("radiometer-cell", "radiometer", 11, "cell", norm(union([cell_box(p, "TDS 5054, NASA", "Oscilloscope", "Model Number"),
        cell_box(p, "tag 13008", "Oscilloscope", "Model Number")]), p), "glyphs",
        text="TDS 5054, NASA tag 13008", row_header="Oscilloscope", col_header="Model Number")
    add("radiometer-passage", "radiometer", 11, "passage", norm(glyphs(p, "took place from June 28, 2011 through August 4, 2011"), p), "glyphs",
        quote="took place from June 28, 2011 through August 4, 2011", page_label="4")
    p = P("flextp", 40)
    add("flextp-passage", "flextp", 40, "passage", norm(glyphs(p, "The droplet was misshapen on the needles before it stretched and deployed."), p), "glyphs",
        quote="The droplet was misshapen on the needles before it stretched and deployed.", page_label="34")
    p = P("cool", 6)
    add("cool-passage", "cool", 6, "passage", norm(glyphs(p, "Alcohol slows the CF burning rate and increases the extinction diameter"), p), "glyphs",
        quote="Alcohol slows the CF burning rate and increases the extinction diameter")
    p = P("ipc", 8)
    add("ipc-passage", "ipc", 8, "passage", norm(glyphs(p, "Zero-g facility test burning 2-cm-diameter PMMA"), p), "glyphs",
        quote="Zero-g facility test burning 2-cm-diameter PMMA")
    add("ipc-figure", "ipc", 8, "figure", norm(image_box(p, 0), p), "image_object")

    # --- no text layer: visual boxes on the 60 dpi rendering (normalized), approximate
    add("sflame-figure", "sflame", 25, "figure", [0.22, 0.245, 0.917, 0.814], "visual")
    add("sflame-ocr", "sflame", 25, "passage", [0.104, 0.455, 0.184, 0.492], "visual", quote="ICE-GA")
    add("same-ocr", "same_dwg", 1, "passage", [0.626, 0.282, 0.734, 0.314], "visual", quote="THIS VIEW SHOWN WITH TOP PLATE REMOVED")
    add("same-cell", "same_dwg", 1, "cell", [0.055, 0.164, 0.113, 0.180], "visual",
        text="NAS1291C3M", row_header="C5", col_header="PART NUMBER")
    add("flex2-ocr", "flex2_dwg", 1, "passage", [0.881, 0.919, 0.972, 0.931], "visual", quote="HIBMS PACKAGE ASSEMBLY")
    add("flex2-cell", "flex2_dwg", 1, "cell", [0.041, 0.088, 0.081, 0.097], "visual",
        text="67212MFAN31000", row_header="FIXED MIRROR MODULE ASSEMBLY", col_header="PART NUMBER")
    add("spice-ocr", "spice_dwg", 1, "passage", [0.953, 0.720, 0.988, 0.755], "visual", quote="NASA DR. NO.")

    for it in items:
        b = it["bbox"]; assert 0 <= b[0] < b[2] <= 1 and 0 <= b[1] < b[3] <= 1, it
    out = {"note": __doc__.strip().splitlines()[0], "documents": {k: {"investigation": v[0], "file_name": v[1], "cases": v[2],
           "pages": d[k].page_count} for k, v in DOCS.items()}, "items": items}
    Path(__file__).with_name("gold.json").write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n")
    print(len(items), "gold items over", len(DOCS), "documents,", sum(d[k].page_count for k in DOCS), "pages")

if __name__ == "__main__":
    main()

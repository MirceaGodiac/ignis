"""Document lane: the NASA document-category files the first extraction left out (old PowerPoint,
unverified or failed PDFs, plain text, spreadsheets, CSV, Igor), made into inputs of the same
whole-document page pass.

- .ppt  -> PDF by PowerPoint (COM, local), then one page per slide with PyMuPDF.
- .pdf  -> one page per PDF page with PyMuPDF.
- .txt  -> pages of at most 120 lines; a file over 200 kB is profiled like a table instead.
- .xls / .xlsx / .csv -> one page per sheet: the header, a profile of every column (type, count,
  range, a few distinct values) and the first 5 rows; the whole sheet is also written as a CSV
  under runs/dev/data_docs/rest/tables/ for scripts. The model never reads a table cell by cell.
- .pxp  -> listed, not extracted here (tools/wsp/ext/igor_pxp.py reads it).

Appends the new documents and batches to runs/dev/data_docs/index.json (batch ids r000...), so that
brief.py, validate.py and the protection pass work on them unchanged. Re-runnable.

    python tools/wsp/docs/extract_rest.py
"""
import csv
import hashlib
import io
import json
import os
import subprocess
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
ASTRA = Path("runs/dev/data_astra")
sys.path.insert(0, str(ASTRA / "work/document_pages/dependencies"))
import fitz  # noqa: E402  PyMuPDF 1.26.7, Astra's pinned copy

TESTS = REPO / "runs/dev/data_all/work/atlas/tests.jsonl"
BATCH_CHARS, BATCH_PAGES = 220_000, 150
TXT_PAGE_LINES, TXT_TABLE_BYTES = 120, 200_000


def sha_file(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def ppt_to_pdf(src, dst):
    ps = (f"$p = New-Object -ComObject PowerPoint.Application; "
          f"$d = $p.Presentations.Open('{src}', $true, $false, $false); "
          f"$d.SaveAs('{dst}', 32); $d.Close(); $p.Quit()")
    subprocess.run(["powershell", "-NoProfile", "-Command", ps], check=True, timeout=300)


def pdf_pages(p):
    doc = fitz.open(p)
    return [(pg.get_text("text"), "text" if len(pg.get_text("text").strip()) >= 40 else "low_text") for pg in doc]


def profile(rows, name):
    """A sheet as text: header, one line per column, first 5 rows."""
    rows = [r for r in rows if any(str(c).strip() for c in r)]
    if not rows:
        return f"SHEET {name}: empty", 0
    width = max(len(r) for r in rows)
    rows = [list(r) + [""] * (width - len(r)) for r in rows]
    head, body = rows[0], rows[1:]
    out = [f"SHEET {name}: {len(body)} data rows, {width} columns", "HEADER: " + " | ".join(map(str, head))]
    for j in range(width):
        col = [str(r[j]).strip() for r in body if str(r[j]).strip()]
        nums = []
        for v in col:
            try:
                nums.append(float(v))
            except ValueError:
                pass
        kind = "number" if col and len(nums) >= 0.8 * len(col) else "text"
        distinct = list(dict.fromkeys(col))[:5]
        rng = f", range {min(nums):g} to {max(nums):g}" if kind == "number" and nums else ""
        out.append(f"COLUMN {j + 1} '{head[j]}': {kind}, {len(col)} values{rng}, e.g. {distinct}")
    out.append("FIRST ROWS:")
    out += [" | ".join(map(str, r)) for r in body[:5]]
    return "\n".join(out), len(body)


def sheets(path, ext):
    if ext == "csv":
        with io.open(path, encoding="utf-8", errors="replace", newline="") as f:
            return [("csv", list(csv.reader(f)))]
    if ext == "xlsx":
        import openpyxl
        wb = openpyxl.load_workbook(path, read_only=True, data_only=True)
        return [(ws.title, [["" if c is None else c for c in r] for r in ws.iter_rows(values_only=True)]) for ws in wb.worksheets]
    import xlrd
    wb = xlrd.open_workbook(path)
    return [(s.name, [s.row_values(i) for i in range(s.nrows)]) for s in wb.sheets()]


def main():
    up = {}
    for line in io.open(ASTRA / "logs/upload_verified.jsonl", encoding="utf-8"):
        r = json.loads(line)
        up[(r["investigation"], r["file_name"])] = r["source_path"]
    done = set()
    for line in io.open(ASTRA / "work/document_pages/documents.jsonl", encoding="utf-8"):
        r = json.loads(line)
        if r.get("extraction", {}).get("status") == "complete":
            done.add((r["source"]["investigation"], r["source"]["file_name"]))
    rows = list(csv.DictReader(io.open(REPO / "docs/wsp/atlas/server_documents.csv", encoding="utf-8")))
    rest = [r for r in rows if (r["investigation"], r["file_name"]) not in done]
    test_ids = {}
    for line in io.open(TESTS, encoding="utf-8"):
        r = json.loads(line)
        test_ids.setdefault(r["investigation_id"], set()).add(r["test_id"])

    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    known = {d["sha"] for d in idx["documents"]} | {s for d in idx["documents"] for s in d.get("same_text_as", [])}
    (OUT / "inputs").mkdir(exist_ok=True); (OUT / "rest/tables").mkdir(parents=True, exist_ok=True)
    (OUT / "rest/pdf").mkdir(parents=True, exist_ok=True)
    new, status = [], Counter()
    by_sha = {}
    for r in rest:
        inv, name = r["investigation"], r["file_name"]
        ext = name.rsplit(".", 1)[-1].lower()
        src = up.get((inv, name))
        if not src or not os.path.exists(src):
            status[f"{ext}: not on disk"] += 1; continue
        sha = sha_file(src)
        source = {"investigation": inv, "file_name": name, "category": r["category"], "version": int(r["file_version"])}
        if sha in known:
            status[f"{ext}: same bytes already read"] += 1; continue
        if sha in by_sha:
            by_sha[sha]["sources"].append(source); status[f"{ext}: same bytes, other name"] += 1; continue
        try:
            if ext == "pxp":
                status["pxp: left to igor_pxp.py"] += 1; continue
            if ext in ("pdf", "ppt"):
                pdf = src
                if ext == "ppt":
                    pdf = str(OUT / "rest/pdf" / f"{sha}.pdf")
                    if not os.path.exists(pdf):
                        ppt_to_pdf(os.path.abspath(src), os.path.abspath(pdf))
                pages = pdf_pages(pdf)
            elif ext == "txt" and os.path.getsize(src) <= TXT_TABLE_BYTES:
                lines = io.open(src, encoding="utf-8", errors="replace").read().splitlines()
                pages = [("\n".join(lines[i:i + TXT_PAGE_LINES]), "text") for i in range(0, max(len(lines), 1), TXT_PAGE_LINES)]
            else:
                if ext == "txt":
                    with io.open(src, encoding="utf-8", errors="replace") as f:
                        first = f.readline()
                        delim = "\t" if "\t" in first else ("," if "," in first else None)
                        f.seek(0)
                        sh = [("text", [l.split(delim) if delim else l.split() for l in f.read().splitlines()])]
                else:
                    sh = sheets(src, ext)
                pages = []
                for k, (sname, srows) in enumerate(sh):
                    text, n = profile(srows, sname)
                    pages.append((text, "table_profile"))
                    with io.open(OUT / "rest/tables" / f"{sha}.{k:02d}.csv", "w", encoding="utf-8", newline="") as f:
                        csv.writer(f).writerows(srows)
        except Exception as e:  # recorded, never silent
            status[f"{ext}: failed {type(e).__name__}"] += 1; continue
        d = {"sha": sha, "investigations": [inv], "pages": len(pages), "chars": sum(len(t) for t, _ in pages),
             "page_ids": [f"sha256:{sha}:page:{i + 1:06d}" for i in range(len(pages))],
             "input": f"runs/dev/data_docs/inputs/{sha}.txt", "sources": [source], "same_text_as": [],
             "origin": "extract_rest", "kind": ext, "_pages": pages}
        by_sha[sha] = d; new.append(d)
        status[f"{ext}: extracted"] += 1

    for d in new:
        pages = d.pop("_pages")
        invs = sorted({s["investigation"] for s in d["sources"]}); d["investigations"] = invs
        lines = [f"DOCUMENT {d['sha']}", f"INVESTIGATIONS: {', '.join(invs)}",
                 "NASA FILE NAMES (same bytes): " + " | ".join(f"{s['investigation']} / {s['category']} / {s['file_name']} (v{s['version']})" for s in d["sources"]),
                 f"PAGES: {len(pages)}"
                 + ("  (each page is one sheet of a table: its header, a profile of each column and the first rows; write one card per sheet)"
                    if d["kind"] in ("xls", "xlsx", "csv") or pages and pages[0][1] == "table_profile" else "")]
        for inv in invs:
            ids = sorted(test_ids.get(inv, ()))
            lines.append(f"TEST IDS OF {inv} IN THE CATALOGUE ({len(ids)}): " + (", ".join(ids) if ids else "none"))
        lines.append("")
        for i, (t, st) in enumerate(pages, 1):
            lines.append(f"===== PAGE {i} of {len(pages)} | text_status={st} | printed_label=- | images=0 =====")
            lines.append(t.rstrip()); lines.append("")
        io.open(OUT / "inputs" / f"{d['sha']}.txt", "w", encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")

    idx["documents"] = [x for x in idx["documents"] if x.get("origin") != "extract_rest"] + new
    idx["batches"] = [b for b in idx["batches"] if not b["id"].startswith("r")]
    new.sort(key=lambda d: (d["investigations"][0], -d["chars"]))
    cur, size, pin, k = [], 0, 0, 0
    def flush():
        nonlocal cur, size, pin, k
        if cur:
            idx["batches"].append({"id": f"r{k:03d}", "documents": [x["sha"] for x in cur],
                                   "pages": pin, "chars": size}); k += 1
        cur, size, pin = [], 0, 0
    for d in new:
        if cur and (size + d["chars"] > BATCH_CHARS or pin + d["pages"] > BATCH_PAGES):
            flush()
        cur.append(d); size += d["chars"]; pin += d["pages"]
    flush()
    io.open(OUT / "index.json", "w", encoding="utf-8", newline="\n").write(json.dumps(idx, indent=1))
    for s, n in sorted(status.items()):
        print(f"{n:4d}  {s}")
    print(f"{len(new)} new documents, {sum(d['pages'] for d in new)} pages, {k} batches r000..r{k - 1:03d}")


if __name__ == "__main__":
    main()

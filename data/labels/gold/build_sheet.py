"""Builds the R8 gold labelling sheet (labelling_sheet.html) from the gold template and a catalog.

The sheet shows, per template row, the facts a person needs to label it: file name, title, where the
file is stored, page count, a link to the PDF and the section outline (`ignis evidence outline`). It
never shows or suggests a label: gold labels are the person's own judgement, and a model-suggested
label would bias the set that measures the model.

    python3 -I data/labels/gold/build_sheet.py --ignis target/release/ignis --db <catalog.sqlite>

The PDF links are relative to this folder (`../../raw/samples/<inv>/<file>`), so they open from the
main checkout, where `data/raw/` holds the PDFs.
"""

import argparse
import json
import sqlite3
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent


def flat(nodes, depth=0, acc=None):
    acc = [] if acc is None else acc
    for n in nodes:
        acc.append({"d": depth, "t": n["title"], "p0": n["page_start"], "p1": n["page_end"]})
        flat(n.get("children", []), depth + 1, acc)
    return acc


def gather(ignis, db, template):
    con = sqlite3.connect(db)
    docs = []
    for line in Path(template).read_text().splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        sha = r["source_id"].split(":", 1)[1]
        row = con.execute(
            "select title, investigation_ids_json, parser_version from publications where source_sha256=?",
            (sha,),
        ).fetchone()
        if row is None:
            raise SystemExit(f"{r['source_id']} is not in {db}")
        title, invs, view = row
        loc = con.execute("select rel_path from asset_locations where sha256=?", (sha,)).fetchone()
        path = loc[0] if loc else ""
        out = subprocess.run(
            [ignis, "evidence", "outline", r["source_id"], "--depth", "2", "--db", db],
            check=True, capture_output=True, text=True,
        ).stdout
        o = json.loads(out)
        rel = "../../" + path.split("/data/", 1)[1] if "/data/" in path else ""
        docs.append({
            "row": r,
            "title": title,
            "stored_under": json.loads(invs),
            "view": view,
            "pdf_rel": rel,
            "outline_method": o.get("method"),
            "outline": flat(o.get("nodes", [])),
        })
    return docs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", required=True, help="the ignis binary")
    ap.add_argument("--db", required=True, help="the catalog the template was selected from")
    ap.add_argument("--template", default=str(HERE / "gold_template.jsonl"))
    ap.add_argument("--out", default=str(HERE / "labelling_sheet.html"))
    a = ap.parse_args()
    docs = gather(a.ignis, a.db, a.template)
    page = (HERE / "sheet_template.html").read_text()
    data = json.dumps(docs, ensure_ascii=False).replace("</", "<\\/")
    Path(a.out).write_text(page.replace("/*DOCS*/[]", data))
    print(f"{len(docs)} documents -> {a.out}")


if __name__ == "__main__":
    main()

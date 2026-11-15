"""Document lane: merges the readings into the final page cards.

- Text pass (Sonnet, ignis.docs.page_card/1): the base card of every page.
- Visual pass (Muse, ignis.docs.page_card/2), where it exists: replaces summary, quantities, tests
  and enrichment (it saw the page image); the labels (kind, flame_type, studied, fire_safety_need)
  stay from the text pass, which agreed better with the strong check labels; Muse's label values
  are kept beside them under "visual_labels".
- Table sheets: needs_image is false (a table has no image).
- Flame type: "none" and an empty list mean the same; stored as an empty list.
- Protection: a page is protected when ANY reading (text pass, protection pass, visual pass) says
  "yes" or "unsure", or when a reading is missing.

Writes runs/dev/data_docs/final/<sha>.jsonl (document row + page rows) and prints aggregates.

    python tools/wsp/docs/merge.py
"""
import glob
import io
import json
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
PROT = ("yes", "unsure")
TABLE_KINDS = {"xls", "xlsx", "csv"}


def read(pattern):
    rows = {}
    for f in sorted(glob.glob(str(pattern))):
        for line in io.open(f, encoding="utf-8"):
            if line.strip():
                try:
                    r = json.loads(line)
                except json.JSONDecodeError:
                    continue
                rows.setdefault(r.get("document_sha256"), []).append(r)
    return rows


def main():
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    text, prot, vis = read(OUT / "cards" / "*.jsonl"), read(OUT / "protect" / "*.jsonl"), read(OUT / "visual" / "*.jsonl")
    (OUT / "final").mkdir(exist_ok=True)
    c = Counter()
    for d in idx["documents"]:
        sha = d["sha"]
        tr = text.get(sha, [])
        head = next((r for r in tr if r.get("schema") == "ignis.docs.document_card/1"), None)
        pages = {r["page"]: r for r in tr if r.get("schema") == "ignis.docs.page_card/1"}
        pp = {r["page"]: r.get("reveals_test_outcome") for r in prot.get(sha, []) if r.get("schema") == "ignis.docs.protect/1"}
        vv = {r["page"]: r for r in vis.get(sha, []) if r.get("schema") == "ignis.docs.page_card/2"}
        out = []
        if head:
            head = dict(head, schema="ignis.docs.document_card/final-1",
                        sources=d["sources"], same_text_as=d.get("same_text_as", []))
            out.append(head)
        for n in range(1, d["pages"] + 1):
            base = dict(pages.get(n) or {"page": n, "document_sha256": sha, "missing_text_card": True})
            v = vv.get(n)
            if v:
                for k in ("summary", "quantities", "tests", "enrichment"):
                    if v.get(k):
                        base[k] = v[k]
                base["visual_labels"] = {k: v.get(k) for k in ("kind", "flame_type", "studied", "fire_safety_need", "is_summary_or_conclusion")}
                base["read_visually_by"] = v.get("read_by")
                c["visual"] += 1
            if d.get("kind") in TABLE_KINDS:
                base["needs_image"] = False
            base["flame_type"] = [x for x in (base.get("flame_type") or []) if x != "none"]
            readings = [base.get("reveals_test_outcome"), pp.get(n), (v or {}).get("reveals_test_outcome") if v else "absent"]
            protected = any(r in PROT or r is None for r in readings[:2]) or (v is not None and readings[2] in PROT)
            base["protected"] = protected
            base["protection_readings"] = {"text_pass": readings[0], "protection_pass": readings[1],
                                           "visual_pass": None if v is None else readings[2]}
            base["schema"] = "ignis.docs.page_card/final-1"
            out.append(base)
            c["pages"] += 1; c["protected"] += protected
            c["still_needs_image"] += bool(base.get("needs_image")) and not v
        with io.open(OUT / "final" / f"{sha}.jsonl", "w", encoding="utf-8", newline="\n") as f:
            for r in out:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
        c["documents"] += 1
    print(json.dumps(dict(c)))
    print(f"protected {c['protected']} of {c['pages']} ({c['protected'] / max(c['pages'], 1):.1%})")


if __name__ == "__main__":
    main()

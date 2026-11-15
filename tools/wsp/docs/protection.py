"""Document lane: combines the two protection readings. A page is protected when the page pass
or the protection pass says "yes" or "unsure", or when either reading is missing. Writes
runs/dev/data_docs/protected_pages.json (page ids only) and prints aggregates, including the
check against the strong labels of the 200-page sample.

    python tools/wsp/docs/protection.py
"""
import glob
import io
import json
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
PROT = ("yes", "unsure")


def rows(pattern, schema):
    out = {}
    for f in glob.glob(str(pattern)):
        for line in io.open(f, encoding="utf-8"):
            if line.strip():
                try:
                    r = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if r.get("schema") == schema:
                    out[(r["document_sha256"], r["page"])] = r.get("reveals_test_outcome")
    return out


def main():
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    a = rows(OUT / "cards" / "*.jsonl", "ignis.docs.page_card/1")
    b = rows(OUT / "protect" / "*.jsonl", "ignis.docs.protect/1")
    c, protected, missing_b = Counter(), [], 0
    for d in idx["documents"]:
        for p in range(1, d["pages"] + 1):
            k = (d["sha"], p)
            ra, rb = a.get(k), b.get(k)
            if ra is None and rb is None:
                c["no reading yet"] += 1; continue
            if rb is None:
                missing_b += 1
            pa, pb = ra in PROT or ra is None, rb in PROT or rb is None
            c[("both protect" if pa and pb else "only page pass" if pa else "only protection pass" if pb else "both open")] += 1
            if pa or pb:
                protected.append(f"{d['sha']}:{p}")
    io.open(OUT / "protected_pages.json", "w", encoding="utf-8").write(json.dumps(sorted(protected)))
    total = sum(v for k, v in c.items() if k != "no reading yet")
    print(f"pages with a reading: {total}; protection pass missing on {missing_b}")
    for k, v in c.most_common():
        print(f"  {k}: {v}")
    print(f"protected: {len(protected)} ({len(protected) / max(total, 1):.1%}); open: {total - len(protected)}")
    gold = rows(OUT / "gold" / "g*.jsonl", "ignis.docs.page_card/1")
    leak_a = sum(1 for k, v in gold.items() if v in PROT and a.get(k) not in PROT)
    leak_ab = sum(1 for k, v in gold.items() if v in PROT and a.get(k) not in PROT and b.get(k) not in PROT)
    print(f"sample: {len(gold)} pages, {sum(v in PROT for v in gold.values())} protected by the strong labeller; "
          f"left open by the page pass alone: {leak_a}; left open by both passes: {leak_ab}")


if __name__ == "__main__":
    main()

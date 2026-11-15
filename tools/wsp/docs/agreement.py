"""Document lane: agreement between the Sonnet page pass and the strong check labels on the
200-page sample. Prints per-field agreement only; never prints a card.

    python tools/wsp/docs/agreement.py

Fields:
- kind: exact match of the page kind.
- reveals (protection): the two collapsed to "protected" (yes or unsure) and "open" (no). Also the
  number of pages the check labeller protects and Sonnet leaves open: the error that matters.
- needs_image, is_summary_or_conclusion: exact.
- flame_type, studied, fire_safety_need: exact set match, and mean Jaccard overlap.
- tests: mean Jaccard of the test id sets, on pages where either names a test.
A field is kept from the Sonnet pass where exact agreement is at least 0.90 (the rule of PAGE_PASS.md).
"""
import glob
import io
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"


def load_sonnet(sha):
    rows = {}
    for f in sorted(glob.glob(str(OUT / "cards" / f"{sha}*.jsonl"))):
        for line in io.open(f, encoding="utf-8"):
            if line.strip():
                r = json.loads(line)
                if r.get("schema") == "ignis.docs.page_card/1":
                    rows[r["page"]] = r
    return rows


def jac(a, b):
    a, b = set(a or []), set(b or [])
    return 1.0 if not a and not b else len(a & b) / len(a | b)


def main():
    gold = {}
    for f in sorted(glob.glob(str(OUT / "gold" / "g*.jsonl"))):
        for line in io.open(f, encoding="utf-8"):
            if line.strip():
                r = json.loads(line)
                gold[(r["document_sha256"], r["page"])] = r
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    sample = [(p["sha"], p["page"]) for p in idx["check_sample"]["pages"]]
    cache, pairs, missing_gold, missing_sonnet = {}, [], 0, 0
    for sha, page in sample:
        g = gold.get((sha, page))
        if g is None:
            missing_gold += 1; continue
        if sha not in cache:
            cache[sha] = load_sonnet(sha)
        s = cache[sha].get(page)
        if s is None:
            missing_sonnet += 1; continue
        pairs.append((s, g))
    n = len(pairs)
    print(f"sample {len(sample)}, compared {n}, no check label {missing_gold}, no Sonnet card {missing_sonnet}")
    if not n:
        return

    def prot(r):
        return r.get("reveals_test_outcome") in ("yes", "unsure")

    res = {
        "kind": sum(s.get("kind") == g.get("kind") for s, g in pairs) / n,
        "reveals_protected_vs_open": sum(prot(s) == prot(g) for s, g in pairs) / n,
        "needs_image": sum(bool(s.get("needs_image")) == bool(g.get("needs_image")) for s, g in pairs) / n,
        "is_summary_or_conclusion": sum(bool(s.get("is_summary_or_conclusion")) == bool(g.get("is_summary_or_conclusion")) for s, g in pairs) / n,
    }
    for f in ("flame_type", "studied", "fire_safety_need"):
        res[f + "_exact"] = sum(set(s.get(f) or []) == set(g.get(f) or []) for s, g in pairs) / n
        res[f + "_jaccard"] = sum(jac(s.get(f), g.get(f)) for s, g in pairs) / n
    tp = [(s, g) for s, g in pairs if (s.get("tests") or g.get("tests"))]
    res["tests_jaccard_where_named"] = (sum(jac(s.get("tests"), g.get("tests")) for s, g in tp) / len(tp)) if tp else None
    leak = sum(prot(g) and not prot(s) for s, g in pairs)
    over = sum(prot(s) and not prot(g) for s, g in pairs)
    for k, v in res.items():
        keep = "" if v is None or k.endswith("_jaccard") or k.startswith("tests") else ("  keep" if v >= 0.9 else "  repair")
        print(f"{k:32s} {'-' if v is None else f'{v:.3f}'}{keep}")
    print(f"pages the check protects and Sonnet leaves open: {leak} of {n}")
    print(f"pages Sonnet protects and the check leaves open: {over} of {n}")
    print(f"pages naming tests (either side): {len(tp)}")


if __name__ == "__main__":
    main()

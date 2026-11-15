"""Document lane: validates the cards written by the page pass and prints aggregates only.

    python tools/wsp/docs/validate.py            # all batches
    python tools/wsp/docs/validate.py b003 b007  # some batches

Prints, per batch, whether every document and page came back exactly once and valid, and the
list of documents to send back. Never prints a card's content.
"""
import io
import json
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
DOC_TYPES = {"requirements", "design_or_engineering", "operations_or_procedures", "test_matrix",
             "data_description", "results_report", "paper_or_presentation", "calibration", "other"}
PAGE_KINDS = {"front_matter", "contents_or_lists", "requirements", "methods", "apparatus",
              "procedure_or_script", "test_matrix", "results_table", "results_figure", "discussion",
              "conclusions_or_summary", "drawing_or_schematic", "data_listing", "references",
              "administrative", "blank_or_unreadable"}
FLAME = {"spherical_diffusion_flame", "flat_burner_diffusion_flame", "jet_diffusion_flame",
         "coflow_jet_diffusion_flame", "droplet_flame", "solid_surface_flame", "overheat_smoke", "none"}
STUDIED = {"cool_flame", "smoke_point", "extinction", "electric_field", "liftoff", "burning_rate", "aerosol"}
NEED = {"prevention", "detection", "suppression"}


def check_doc(d):
    """Returns (problems, counters) for one document."""
    files = sorted((OUT / "cards").glob(f"{d['sha']}*.jsonl"))
    if not files:
        return ["missing"], Counter()
    probs, seen, c, head = [], Counter(), Counter(), 0
    lines = [l for f in files for l in io.open(f, encoding="utf-8")]
    for n, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            r = json.loads(line)
        except json.JSONDecodeError:
            probs.append(f"line {n}: not JSON"); continue
        if r.get("document_sha256") != d["sha"]:
            probs.append(f"line {n}: wrong document"); continue
        if r.get("schema") == "ignis.docs.document_card/1":
            head += 1
            if r.get("doc_type") not in DOC_TYPES: probs.append("doc_type")
            if len(str(r.get("abstract", "")).split()) > 100: probs.append("abstract too long")
            continue
        if r.get("schema") != "ignis.docs.page_card/1":
            probs.append(f"line {n}: schema"); continue
        p = r.get("page")
        if not isinstance(p, int) or not 1 <= p <= d["pages"]:
            probs.append(f"line {n}: page"); continue
        seen[p] += 1
        bad = []
        if r.get("kind") not in PAGE_KINDS: bad.append("kind")
        if not set(r.get("flame_type", [])) <= FLAME: bad.append("flame_type")
        if not set(r.get("studied", [])) <= STUDIED: bad.append("studied")
        if not set(r.get("fire_safety_need", [])) <= NEED: bad.append("need")
        if r.get("reveals_test_outcome") not in ("yes", "no", "unsure"): bad.append("reveals")
        if r.get("confidence") not in ("high", "medium", "low"): bad.append("confidence")
        e = r.get("enrichment") or {}
        if not isinstance(e.get("queries"), list) or not isinstance(e.get("concepts"), list): bad.append("enrichment")
        if len(str(r.get("summary", "")).split()) > 55: c["summary_over_55_words"] += 1  # a warning, not a failure
        if bad:
            probs.append(f"page {p}: " + ",".join(bad))
        c["pages"] += 1
        c["reveals_" + str(r.get("reveals_test_outcome"))] += 1
        c["kind_" + str(r.get("kind"))] += 1
        c["needs_image"] += bool(r.get("needs_image"))
        c["low_confidence"] += r.get("confidence") == "low"
    if head != 1: probs.append(f"{head} document rows")
    missing = [p for p in range(1, d["pages"] + 1) if seen[p] == 0]
    dup = [p for p, k in seen.items() if k > 1]
    if missing: probs.append(f"{len(missing)} pages missing")
    if dup: probs.append(f"{len(dup)} pages repeated")
    return probs, c


def main():
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    docs = {d["sha"]: d for d in idx["documents"]}
    want = set(sys.argv[1:])
    total, resend = Counter(), []
    for b in idx["batches"]:
        if want and b["id"] not in want:
            continue
        bad = []
        for sha in b["documents"]:
            probs, c = check_doc(docs[sha])
            total += c
            if probs:
                bad.append((sha[:12], probs[:4]))
                resend.append(sha)
        print(f"{b['id']}: {len(b['documents'])} documents, {b['pages']} pages, "
              f"{'ok' if not bad else str(len(bad)) + ' to resend: ' + str(bad)[:300]}")
    print("aggregate:", json.dumps(dict(sorted(total.items()))))
    io.open(OUT / "resend.json", "w", encoding="utf-8").write(json.dumps(resend))
    print(f"{len(resend)} documents to resend (runs/dev/data_docs/resend.json)")


if __name__ == "__main__":
    main()

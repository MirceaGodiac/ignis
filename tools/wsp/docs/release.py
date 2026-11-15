"""Document lane, step 9 for authors: what a model author (Astra) may read from the document pass.

Writes runs/dev/data_docs/release/:

- open_pages.jsonl  every page no reader protects (three independent readings, any "yes" or
                    "unsure" protects): NASA file names, investigation, page, kind, generated summary
                    (marked generated), quantities with units, instruments, materials, test ids named,
                    and the extracted page text (NASA's own words) for methods, apparatus,
                    requirements, procedures, test matrices, drawings and discussion pages.
- documents.jsonl   document cards (title, type, generated abstract, glossary) of documents with at
                    least one open page.
- MANIFEST.json     counts, hashes, the rule, and what the release does not hold.

Never in the release: a protected page in any form; per-test outcomes; test-day membership;
narratives (they summarise results at investigation level and stay with the application).

    python tools/wsp/docs/release.py
"""
import datetime
import glob
import hashlib
import io
import json
import re
import shutil
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
REL = OUT / "release"
# kinds left out even when every reader says the page reveals no outcome: an extra margin for authors
EXCLUDED_KINDS = {"results_figure", "results_table", "data_listing", "conclusions_or_summary"}
TEXT_KINDS = {"methods", "apparatus", "requirements", "procedure_or_script", "test_matrix",
              "drawing_or_schematic", "discussion", "front_matter", "contents_or_lists"}


def page_texts(input_path):
    """Page n -> text, from the whole-document input file (the extraction the readers saw)."""
    text = io.open(input_path, encoding="utf-8").read()
    parts = re.split(r"^===== PAGE (\d+) of \d+ [^\n]*=====\n", text, flags=re.M)
    return {int(parts[i]): parts[i + 1].strip() for i in range(1, len(parts) - 1, 2)}


def main():
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    if REL.exists():
        shutil.rmtree(REL)
    REL.mkdir(parents=True)
    c = Counter()
    with io.open(REL / "open_pages.jsonl", "w", encoding="utf-8", newline="\n") as fp, \
         io.open(REL / "documents.jsonl", "w", encoding="utf-8", newline="\n") as fd:
        for d in idx["documents"]:
            p = OUT / "final" / f"{d['sha']}.jsonl"
            if not p.exists():
                continue
            rows = [json.loads(l) for l in io.open(p, encoding="utf-8") if l.strip()]
            pages = [r for r in rows if r.get("schema") == "ignis.docs.page_card/final-1"]
            open_pages = [r for r in pages if r.get("protected") is False
                          and all(v == "no" for v in (r.get("protection_readings") or {}).values() if v is not None)
                          and r.get("kind") not in EXCLUDED_KINDS]
            c["pages"] += len(pages); c["open"] += len(open_pages)
            if not open_pages:
                continue
            texts = page_texts(REPO / d["input"]) if (REPO / d["input"]).exists() else {}
            head = next((r for r in rows if r.get("schema", "").startswith("ignis.docs.document_card")), {})
            fd.write(json.dumps({"schema": "ignis.docs.release.document/1", "nasa_files": d["sources"],
                                 "source_sha256": [d["sha"]] + list(d.get("same_text_as") or []),
                                 "investigations": d["investigations"], "title": head.get("title"),
                                 "doc_type": head.get("doc_type"), "abstract_generated": head.get("abstract"),
                                 "glossary": head.get("glossary") or [], "open_pages": len(open_pages),
                                 "pages": d["pages"]}, ensure_ascii=False) + "\n")
            c["documents"] += 1
            for r in open_pages:
                e = r.get("enrichment") or {}
                row = {"schema": "ignis.docs.release.page/1", "source_sha256": [d["sha"]] + list(d.get("same_text_as") or []),
                       "nasa_files": [s["file_name"] for s in d["sources"]], "investigations": d["investigations"],
                       "page": r["page"], "kind": r.get("kind"), "summary_generated": r.get("summary"),
                       "quantities": r.get("quantities") or [], "tests_named": r.get("tests") or [],
                       "instruments": e.get("instruments") or [], "materials": e.get("materials") or []}
                if r.get("kind") in TEXT_KINDS and texts.get(r["page"]):
                    row["nasa_text"] = texts[r["page"]]
                    c["with_text"] += 1
                fp.write(json.dumps(row, ensure_ascii=False) + "\n")
                c["kind:" + str(r.get("kind"))] += 1
    files = {f.name: hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(REL.glob("*.jsonl"))}
    manifest = {"schema": "ignis.docs.release.manifest/1",
                "created": datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat(),
                "counts": dict(c), "files": files,
                "rule": "a page is released only when all three readings (text pass, independent protection pass, "
                        "visual pass where run) say it reveals no outcome of an individual test; 'unsure' protects",
                "check": "on a blind 200-page sample, 0 of 53 pages a strong labeller protects were left open by the union",
                "not_in_release": ["any protected page, in any form", "per-test outcomes, of any group",
                                   "test-day membership of the frozen splits", "investigation narratives",
                                   "spreadsheet sheets of results (their pages are protected)",
                                   "pages of kind " + ", ".join(sorted(EXCLUDED_KINDS)) + " (excluded as a margin even when no reader protects them)"],
                "generated_fields": ["summary_generated", "abstract_generated"],
                "use": "summary and abstract are generated by Claude Sonnet 5.5 / Muse Spark 1.3 Contributor: "
                       "orientation, never a source of a typed number; type numbers from nasa_text or the NASA file itself"}
    io.open(REL / "MANIFEST.json", "w", encoding="utf-8", newline="\n").write(json.dumps(manifest, indent=1))
    print(json.dumps({k: v for k, v in c.items() if not k.startswith("kind:")}))
    print({k[5:]: v for k, v in sorted(c.items()) if k.startswith("kind:")})


if __name__ == "__main__":
    main()

"""Document lane, step 9: the export the application reads (ignis-evidence, the investigation cards).

Writes runs/dev/data_docs/export/ (then copied to the VM, /srv/ignis/data/enrichment/docs-v1/):

- pages.jsonl      one row per page (or sheet) of every NASA document read, keyed by the sha256 of the
                   NASA file's bytes (the key of publications.source_sha256 in the evidence catalog)
                   and the 1-based page: kind, summary (generated, never evidence), expansion terms
                   for retrieval (R12b: never shown as a snippet), tests, quantities, labels, and
                   outcome_bearing (the union of three readings; it may only raise the catalog's flag).
- documents.jsonl  one row per document: NASA file names and versions, investigations, title, type,
                   abstract (generated), glossary.
- narratives/      the 19 investigation narratives (ignis.atlas.narrative/1).
- MANIFEST.json    file hashes, counts, models, schema versions, and the rules above.

    python tools/wsp/docs/export.py
"""
import datetime
import glob
import hashlib
import io
import json
import shutil
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
EXP = OUT / "export"


def terms(e):
    out = []
    for k in ("concepts", "synonyms", "queries", "instruments", "materials"):
        for t in (e or {}).get(k) or []:
            if isinstance(t, str) and t.strip():
                out.append(t.strip())
    seen, uniq = set(), []
    for t in out:
        if t.lower() not in seen:
            seen.add(t.lower()); uniq.append(t)
    return uniq


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    if EXP.exists():
        shutil.rmtree(EXP)
    (EXP / "narratives").mkdir(parents=True)
    c = Counter()
    with io.open(EXP / "pages.jsonl", "w", encoding="utf-8", newline="\n") as fp, \
         io.open(EXP / "documents.jsonl", "w", encoding="utf-8", newline="\n") as fd:
        for d in idx["documents"]:
            path = OUT / "final" / f"{d['sha']}.jsonl"
            if not path.exists():
                c["documents_without_cards"] += 1; continue
            rows = [json.loads(l) for l in io.open(path, encoding="utf-8") if l.strip()]
            head = next((r for r in rows if r.get("schema", "").startswith("ignis.docs.document_card")), {})
            keys = [d["sha"]] + list(d.get("same_text_as") or [])   # every NASA byte identity with this text
            doc = {"schema": "ignis.docs.export.document/1", "source_sha256": keys, "investigations": d["investigations"],
                   "nasa_files": d["sources"], "kind": d.get("kind", "pdf"), "pages": d["pages"],
                   "title": head.get("title"), "doc_type": head.get("doc_type"), "abstract": head.get("abstract"),
                   "glossary": head.get("glossary") or [], "test_id_scheme": head.get("test_id_scheme"),
                   "generated": True}
            fd.write(json.dumps(doc, ensure_ascii=False) + "\n"); c["documents"] += 1
            for r in rows:
                if r.get("schema") != "ignis.docs.page_card/final-1":
                    continue
                vis = bool(r.get("read_visually_by"))
                row = {"schema": "ignis.docs.export.page/1", "source_sha256": keys, "page": r["page"],
                       "investigations": d["investigations"], "kind": r.get("kind"), "summary": r.get("summary"),
                       "expansion": terms(r.get("enrichment")), "tests": r.get("tests") or [],
                       "quantities": r.get("quantities") or [], "flame_type": r.get("flame_type") or [],
                       "studied": r.get("studied") or [], "fire_safety_need": r.get("fire_safety_need") or [],
                       "is_summary_or_conclusion": bool(r.get("is_summary_or_conclusion")),
                       "outcome_bearing": bool(r.get("protected", True)),
                       "read_by": ["claude-sonnet-5-5"] + (["muse-spark-1.3-contributor"] if vis else []),
                       "generated": True}
                fp.write(json.dumps(row, ensure_ascii=False) + "\n")
                c["pages"] += 1; c["outcome_bearing"] += row["outcome_bearing"]; c["expansion_terms"] += len(row["expansion"])
    for p in sorted((REPO / "docs/wsp/atlas/narratives").glob("PSI-*.json")):
        shutil.copy(p, EXP / "narratives" / p.name); c["narratives"] += 1
    files = {str(p.relative_to(EXP)).replace("\\", "/"): sha(p) for p in sorted(EXP.rglob("*")) if p.is_file()}
    manifest = {
        "schema": "ignis.docs.export.manifest/1", "created": datetime.datetime.utcnow().replace(microsecond=0).isoformat() + "Z",
        "counts": dict(c), "files": files,
        "readers": {"text_pass": "claude-sonnet-5-5 (high effort), whole document per reader",
                    "protection_pass": "claude-sonnet-5-5 (high effort), independent of the text pass",
                    "visual_pass": "muse-spark-1.3-contributor (page image + whole document text)",
                    "narratives": "muse-spark-1.3-contributor, from the page and document cards"},
        "check": "200 random pages labelled blind by claude-opus-5-5: outcome protection 94% agreement for the text pass alone, "
                 "0 of 53 protected pages left open by the union; kind 73%, flame type 86.5%, studied 86.5%, need 86% (filters only)",
        "rules": ["key: sha256 of the NASA file bytes and the 1-based page; one text may stand for several NASA files (source_sha256 is a list)",
                  "summary, abstract, expansion and narratives are generated: never shown as evidence, a quote or a citation",
                  "expansion goes into a separate retrieval column with its own weight (LANES_V2 R12b)",
                  "outcome_bearing may only raise the catalog's flag, never lower it",
                  "narrative findings marked 'NASA states' were labelled so by the generating model and are not verified page by page"],
    }
    io.open(EXP / "MANIFEST.json", "w", encoding="utf-8", newline="\n").write(json.dumps(manifest, indent=1))
    print(json.dumps(dict(c)))


if __name__ == "__main__":
    main()

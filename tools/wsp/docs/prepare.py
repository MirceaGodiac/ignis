"""Document lane, step 2: builds the inputs of the whole-document page pass.

For each of the unique NASA documents whose text was extracted (Astra's custody tool,
runs/dev/data_astra/work/document_pages), writes one text file with every page marked, a header
naming the investigation and the NASA file names, and the list of test ids of that investigation
from the test catalogue (ids only: no group, no partition, no outcome). Groups the documents into
batches for sub-agents without ever splitting a document. Draws the 200-page check sample.

Writes under runs/dev/data_docs/ (private, gitignored). Model authors do not read that folder.

    python tools/wsp/docs/prepare.py
"""
import collections
import hashlib
import io
import json
import random
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SRC = Path("runs/dev/data_astra/work/document_pages")
if not SRC.exists():
    SRC = REPO / "runs/dev/data_astra/work/document_pages"
TESTS = REPO / "runs/dev/data_all/work/atlas/tests.jsonl"
OUT = REPO / "runs/dev/data_docs"
BATCH_CHARS = 220_000   # text per sub-agent; a larger document gets a batch of its own
BATCH_PAGES = 150       # pages per sub-agent, so that the rows it writes stay a manageable size
SAMPLE_PAGES = 200
SEED = 20261114


def main():
    sources = collections.defaultdict(list)
    for line in io.open(SRC / "documents.jsonl", encoding="utf-8"):
        r = json.loads(line)
        if "document_sha256" in r and r.get("extraction", {}).get("status") == "complete":
            s = r["source"]
            sources[r["document_sha256"]].append({"investigation": s["investigation"], "file_name": s["file_name"],
                                                  "category": s["category"], "version": s["file_version"]})
    pages = collections.defaultdict(list)
    for line in io.open(SRC / "pages.jsonl", encoding="utf-8"):
        r = json.loads(line)
        pages[r["document_sha256"]].append(r)
    test_ids = collections.defaultdict(set)
    for line in io.open(TESTS, encoding="utf-8"):
        r = json.loads(line)
        test_ids[r["investigation_id"]].add(r["test_id"])

    (OUT / "inputs").mkdir(parents=True, exist_ok=True)
    docs, by_text = [], {}
    for sha in sorted(pages):
        if sha not in sources:
            continue
        ps = sorted(pages[sha], key=lambda p: p["page_ordinal"])
        # two NASA files with different bytes but the same text are read once
        text_key = hashlib.sha256("|".join(p["text_sha256"] for p in ps).encode()).hexdigest()
        if text_key in by_text:
            by_text[text_key]["same_text_as"].append(sha)
            continue
        srcs = sorted(sources[sha], key=lambda s: (s["investigation"], s["file_name"]))
        invs = sorted({s["investigation"] for s in srcs})
        lines = [f"DOCUMENT {sha}",
                 f"INVESTIGATIONS: {', '.join(invs)}",
                 "NASA FILE NAMES (same bytes): " + " | ".join(f"{s['investigation']} / {s['category']} / {s['file_name']} (v{s['version']})" for s in srcs),
                 f"PAGES: {len(ps)}"]
        for inv in invs:
            ids = sorted(test_ids.get(inv, ()))
            lines.append(f"TEST IDS OF {inv} IN THE CATALOGUE ({len(ids)}): " + (", ".join(ids) if ids else "none"))
        lines.append("")
        chars = 0
        for p in ps:
            text = io.open(SRC / p["text_path"], encoding="utf-8").read()
            chars += len(text)
            lines.append(f"===== PAGE {p['page_ordinal']} of {len(ps)} | text_status={p['text_status']} | printed_label={p.get('printed_page_label') or '-'} | images={p.get('image_objects', 0)} =====")
            lines.append(text.rstrip())
            lines.append("")
        path = OUT / "inputs" / f"{sha}.txt"
        io.open(path, "w", encoding="utf-8", newline="\n").write("\n".join(lines) + "\n")
        docs.append({"sha": sha, "investigations": invs, "pages": len(ps), "chars": chars,
                     "page_ids": [p["page_id"] for p in ps], "input": str(path.relative_to(REPO)).replace("\\", "/"),
                     "sources": srcs, "same_text_as": []})
        by_text[text_key] = docs[-1]

    # batches: documents in order of investigation, then size; never split one
    docs.sort(key=lambda d: (d["investigations"][0], -d["chars"]))
    batches, cur, size, pages_in = [], [], 0, 0
    for d in docs:
        if cur and (size + d["chars"] > BATCH_CHARS or pages_in + d["pages"] > BATCH_PAGES):
            batches.append(cur); cur, size, pages_in = [], 0, 0
        cur.append(d["sha"]); size += d["chars"]; pages_in += d["pages"]
    if cur:
        batches.append(cur)

    rng = random.Random(SEED)
    all_pages = [(d["sha"], i + 1) for d in docs for i in range(d["pages"])]
    sample = sorted(rng.sample(all_pages, SAMPLE_PAGES))

    index = {"schema": "ignis.docs.index/1", "source": str(SRC), "documents": docs,
             "batches": [{"id": f"b{i:03d}", "documents": b,
                          "pages": sum(next(d["pages"] for d in docs if d["sha"] == s) for s in b),
                          "chars": sum(next(d["chars"] for d in docs if d["sha"] == s) for s in b)}
                         for i, b in enumerate(batches)],
             "check_sample": {"seed": SEED, "pages": [{"sha": s, "page": n} for s, n in sample]}}
    io.open(OUT / "index.json", "w", encoding="utf-8", newline="\n").write(json.dumps(index, indent=1))
    print(f"{len(docs)} documents, {sum(d['pages'] for d in docs)} pages, {sum(d['chars'] for d in docs)} characters, "
          f"{len(batches)} batches; sample of {len(sample)} pages from {len({s for s, _ in sample})} documents")
    print("index sha256", hashlib.sha256((OUT / "index.json").read_bytes()).hexdigest())


if __name__ == "__main__":
    main()

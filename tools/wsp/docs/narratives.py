"""Document lane, step 3: the 19 investigation narratives, on Muse Spark 1.3 Contributor.

For each investigation Muse gets: the atlas card (NASA's metadata text, the fire-safety need typed in
the card), the document card of every document of that investigation (title, type, abstract,
glossary, file names), and the page summaries of its summary, conclusion, discussion, methods,
apparatus and test-matrix pages, each with its locator (NASA file name, page). It writes
docs/wsp/atlas/narratives/PSI-<id>.json in schema ignis.atlas.narrative/1, merged into the card by
tools/wsp/atlas/build_cards.py.

Findings are about the investigation as a whole; no outcome of an individual test is stated.

    python tools/wsp/docs/narratives.py            # all 19
    python tools/wsp/docs/narratives.py PSI-69     # one
"""
import datetime
import glob
import io
import json
import re
import sys
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import muse  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
ATLAS = REPO / "docs/wsp/atlas"
KINDS = {"conclusions_or_summary", "discussion", "methods", "apparatus", "test_matrix", "front_matter",
         "results_figure", "results_table", "procedure_or_script", "requirements"}

PROMPT = """You write the document part of an investigation card for a NASA microgravity combustion dashboard (NASA Space Apps 2026, "Flame in Freefall": fire-safety insight for crewed spacecraft, stations, surface bases and rovers).

You receive: the investigation's card (NASA's own metadata text and the fire-safety need typed by the atlas), the document card of each of its documents, and the summaries of its relevant pages, each with a locator "FILE | p. N".

Return ONLY this JSON object:
{"how_tests_differ": {"text": "<at most 90 words: what was varied across tests (fuels, materials, oxygen, pressure, flow, burner, sample, gravity)>", "sources": [LOC]},
 "sensors_and_measurements": [{"instrument": "...", "measures": "...", "source": LOC}],
 "findings": [{"statement": "<at most 45 words>", "kind": "NASA states" | "interpretation", "source": LOC, "quote": "<at most 20 words copied from the page summary or document text, or null>"}],
 "fire_safety_relevance": {"text": "<at most 60 words>", "kind": "NASA states" | "interpretation", "sources": [LOC]},
 "need_check": {"agrees_with_card": true | false, "comment": "<one sentence: does the need typed in the card (prevention / detection / suppression) match what the documents say>"},
 "not_found": ["<what a fire-safety reader would want that the documents given do not say>"],
 "documents_read": [{"file": "<NASA file name>", "pages_read": "<page list or 'document card only'>"}]}
LOC = {"file": "<NASA file name exactly as given, or 'PSI metadata record'>", "page": <number or null>, "field": "<null or a metadata field name>"}

Rules:
- 3 to 6 findings, about the investigation as a whole (mechanisms, regimes, trends, design lessons). NEVER the outcome of an individual test or run (no burn time, extinction, diameter or spread rate of a named test).
- "NASA states" only when a page summary or the metadata says it; otherwise "interpretation".
- Every statement has a source from the material given. Do not invent files, pages or numbers.
- English, plain, factual."""


def main():
    want = set(sys.argv[1:])
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    docs_by_inv = defaultdict(list)
    for d in idx["documents"]:
        for inv in d["investigations"]:
            docs_by_inv[inv].append(d)
    final = {}
    for f in glob.glob(str(OUT / "final" / "*.jsonl")):
        rows = [json.loads(l) for l in io.open(f, encoding="utf-8") if l.strip()]
        if rows:
            final[rows[0].get("document_sha256")] = rows
    cards = {Path(p).stem: json.load(io.open(p, encoding="utf-8")) for p in glob.glob(str(ATLAS / "cards" / "PSI-*.json"))}
    (ATLAS / "narratives").mkdir(exist_ok=True)
    jobs = [inv for inv in sorted(cards, key=lambda s: int(s[4:])) if not want or inv in want]

    def one(inv):
        card = cards[inv]
        lines = ["INVESTIGATION CARD:", json.dumps({k: card.get(k) for k in ("investigation_id", "acronym", "title", "nasa_text", "hardware", "fire_safety", "crew_atmospheres", "imposed_flow")}, ensure_ascii=False), ""]
        n_pages = 0
        for d in docs_by_inv.get(inv, []):
            rows = final.get(d["sha"], [])
            names = [s["file_name"] for s in d["sources"] if s["investigation"] == inv] or [d["sources"][0]["file_name"]]
            head = next((r for r in rows if r.get("schema", "").startswith("ignis.docs.document_card")), {})
            lines.append(f"DOCUMENT {names[0]} (also: {', '.join(names[1:]) or 'none'}) type={head.get('doc_type')} title={head.get('title')}")
            lines.append(f"  abstract: {head.get('abstract')}")
            if head.get("glossary"):
                lines.append("  glossary: " + "; ".join(f"{g.get('short')}={g.get('long')}" for g in head["glossary"][:40]))
            for r in rows:
                if r.get("schema") != "ignis.docs.page_card/final-1":
                    continue
                if r.get("is_summary_or_conclusion") or r.get("kind") in KINDS:
                    lines.append(f"  {names[0]} | p. {r['page']} | {r.get('kind')} | {r.get('summary')}")
                    n_pages += 1
            lines.append("")
        msgs = [{"role": "system", "content": PROMPT}, {"role": "user", "content": "\n".join(lines)}]
        r = muse.call(msgs, response_format={"type": "json_object"}, effort="high")
        raw = re.sub(r"^```(json)?|```$", "", r["choices"][0]["message"]["content"].strip(), flags=re.M)
        n = json.loads(raw)
        n = {"schema": "ignis.atlas.narrative/1", "investigation_id": inv, **n,
             "written_by": muse.MODEL, "date": datetime.date.today().isoformat(),
             "input": {"documents": len(docs_by_inv.get(inv, [])), "page_summaries": n_pages,
                       "made_by": "tools/wsp/docs/narratives.py"}}
        io.open(ATLAS / "narratives" / f"{inv}.json", "w", encoding="utf-8", newline="\n").write(json.dumps(n, indent=1, ensure_ascii=False) + "\n")
        u = r.get("usage", {})
        return inv, len(docs_by_inv.get(inv, [])), n_pages, len(n.get("findings", [])), u.get("prompt_tokens"), u.get("completion_tokens")

    with ThreadPoolExecutor(19) as ex:
        for res in ex.map(one, jobs):
            print("%s: %d documents, %d page summaries, %d findings, tokens in/out %s/%s" % res, flush=True)


if __name__ == "__main__":
    main()

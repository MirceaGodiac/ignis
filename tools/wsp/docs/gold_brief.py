"""Document lane: briefs of the check labellers. A strong model labels the 200 sample pages drawn
by prepare.py, blind to the Sonnet cards, with the same schema and the whole document as context.
Writes runs/dev/data_docs/gold_briefs/g<k>.txt, documents split over N labellers by size.

    python tools/wsp/docs/gold_brief.py 5
"""
import collections
import io
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
WIN = str(REPO).replace("/", "\\")

HEAD = """You are a check labeller for a NASA microgravity combustion archive (the PSI repository). Another reader has labelled every page of these documents; you label a sample of their pages independently, so that the two can be compared. You must not look at the other reader's work: never open anything under runs\\dev\\data_docs\\cards.

FIRST read the schema and rules, in full: {win}\\tools\\wsp\\docs\\PAGE_PASS.md

THEN, for each document below, read its input file IN FULL (use Read with offset/limit until the end), so that you have the whole document as context, and write page rows ONLY for the sample pages listed. Do not write a document row.

Documents and sample pages:
{docs}

Output: one file {win}\\runs\\dev\\data_docs\\gold\\{gid}.jsonl, one JSON object per line, schema ignis.docs.page_card/1, exactly one row per listed sample page, with "document_sha256" and "page" set. Use only the closed lists in PAGE_PASS.md. reveals_test_outcome: "yes" if the page states or shows how an individual test ended or what it measured; "unsure" whenever in doubt.

Tools: only Read (PAGE_PASS.md and the listed input files) and Write (your output file). No commands, no web, no other file.

Reply with one line: rows written, and how many marked reveals_test_outcome yes/unsure. No card content, no test outcome."""


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 5
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    docs = {d["sha"]: d for d in idx["documents"]}
    pages = collections.defaultdict(list)
    for p in idx["check_sample"]["pages"]:
        pages[p["sha"]].append(p["page"])
    order = sorted(pages, key=lambda s: -docs[s]["chars"])
    groups, loads = [[] for _ in range(n)], [0] * n
    for s in order:
        k = loads.index(min(loads)); groups[k].append(s); loads[k] += docs[s]["chars"]
    (OUT / "gold_briefs").mkdir(exist_ok=True); (OUT / "gold").mkdir(exist_ok=True)
    for k, g in enumerate(groups):
        lines = [f"- {WIN}\\runs\\dev\\data_docs\\inputs\\{s}.txt ({docs[s]['pages']} pages): sample pages "
                 + ", ".join(map(str, sorted(pages[s]))) for s in g]
        gid = f"g{k}"
        io.open(OUT / "gold_briefs" / f"{gid}.txt", "w", encoding="utf-8", newline="\n").write(
            HEAD.format(win=WIN, docs="\n".join(lines), gid=gid))
        print(gid, len(g), "documents", sum(len(pages[s]) for s in g), "pages", loads[k], "characters")


if __name__ == "__main__":
    main()

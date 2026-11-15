"""Document lane: the second, independent protection pass. A reader that has not seen the page
cards reads each document whole and says, for every page, whether it reveals how an individual
test ended or what it measured. A page is protected when either pass protects it.

Batches are the page-pass batches, one reader each. Writes
runs/dev/data_docs/protect_briefs/p<k>.txt; output goes to runs/dev/data_docs/protect/<sha>.jsonl.

    python tools/wsp/docs/protect_brief.py
"""
import io
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
WIN = str(REPO).replace("/", "\\")

HEAD = """You are a protection reader for a NASA microgravity combustion archive. Model authors must never see how an individual test ended or what it measured, so every page that reveals that is locked away. Another reader has already judged these pages; you judge them again independently. Never open anything under runs\\dev\\data_docs\\cards or runs\\dev\\data_docs\\gold.

For each document below, read its input file IN FULL (Read with offset/limit until the end); pages are marked "===== PAGE n of N ... =====". Then write, for EVERY page 1..N, one JSON line:
{{"schema": "ignis.docs.protect/1", "document_sha256": "<sha>", "page": n, "reveals_test_outcome": "yes" | "no" | "unsure", "why": "<at most 12 words, no values>"}}

"yes" when the page states or shows, for one or more individual tests or runs: how it ended (extinction, blow-off, quench, burn-out, flow stopped, no ignition), a burn time or lifetime, a measured value or series (diameters, flame size, standoff, spread rate, temperature, radiation, soot, smoke, pressure rise), a plot or table of measured results, an observation of what a test did, or a comparison of a model against a test. Also "yes" for a test log or summary that records results per test.
"no" only when the page holds nothing of that: requirements, design, apparatus, planned conditions and test matrices without results, procedures, crew operations, administration, references, theory, or results only of ground tests in normal gravity that are clearly not flight tests of this investigation.
"unsure" whenever in doubt, including image-only pages and garbled text. A missed page is the costly error; an extra "unsure" costs nothing.

Documents (input file -> output file):
{docs}

Tools: only Read (the listed input files) and Write (the listed output files). No commands, no web, no other file.

Reply with one line per document: sha256 prefix (12 chars), pages written, count of yes / unsure / no. Nothing else: no content, no outcome."""


def main():
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    docs = {d["sha"]: d for d in idx["documents"]}
    b = idx["batches"]
    (OUT / "protect_briefs").mkdir(exist_ok=True); (OUT / "protect").mkdir(exist_ok=True)
    k = 0
    for i in range(0, len(b), 1):
        shas = [s for x in b[i:i + 1] for s in x["documents"]]
        lines = [f"- {WIN}\\runs\\dev\\data_docs\\inputs\\{s}.txt ({docs[s]['pages']} pages) -> "
                 f"{WIN}\\runs\\dev\\data_docs\\protect\\{s}.jsonl" for s in shas]
        io.open(OUT / "protect_briefs" / f"p{k:02d}.txt", "w", encoding="utf-8", newline="\n").write(
            HEAD.format(docs="\n".join(lines)))
        print(f"p{k:02d}", len(shas), "documents", sum(docs[s]["pages"] for s in shas), "pages",
              sum(docs[s]["chars"] for s in shas), "characters")
        k += 1


if __name__ == "__main__":
    main()

"""Document lane: prints the brief of one sub-agent of the page pass, for a batch id.

    python tools/wsp/docs/brief.py b012

The brief is the same for every batch; only the list of documents and output files changes.
"""
import io
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
WIN = str(REPO).replace("/", "\\")
PART = 40

HEAD = """You are a page reader for a NASA microgravity combustion archive (the PSI repository). You read whole NASA documents and write one card per page plus search enrichment. You are not a modeller; nobody needs your opinion about any model.

FIRST read the schema and rules, in full: {win}\\tools\\wsp\\docs\\PAGE_PASS.md

THEN, for each document below, read its input file IN FULL before writing anything for it (use Read with offset/limit until you reach the end; never write cards from a partial reading). The file header names the investigation, the NASA file names and the test ids the catalogue knows for that investigation. Pages are marked "===== PAGE n of N ... =====".

Documents of batch {bid} (input file, then the output file or files):
{docs}

How to write:
- One JSON object per line, no prose, UTF-8. First line of the first output file of a document: the document row (schema ignis.docs.document_card/1). Then one page row (schema ignis.docs.page_card/1) for EVERY page 1..N, in order, exactly once, even blank or unreadable pages (kind blank_or_unreadable, needs_image true).
- A document longer than {part} pages is written in the parts listed above, after it has been read whole: .p01 = document row + pages 1-{part}, .p02 = the next {part} pages, and so on.
- Use only the closed lists in PAGE_PASS.md. Test ids: match to the catalogue list in the header when the page clearly means that test; otherwise write the id as the page writes it.
- Use what the whole document says: an abbreviation defined on page 2 explains page 30; a test matrix on page 5 tells which tests a later table is about.
- reveals_test_outcome: "yes" if the page states or shows how an individual test ended or what it measured; "unsure" whenever in doubt.
- needs_image: true when the page is mostly a figure, a drawing or a table whose meaning the text extraction loses, or when text_status is not "text".
- Summaries and enrichment in English, factual, from the page only (plus context of the same document). Do not invent values. No quote longer than 15 words.
- Enrichment queries: 3 to 6 phrasings a researcher would type to find this page.

Tools: use only Read (on PAGE_PASS.md and the listed input files) and Write (on the listed output files). Do not run commands, search the web, or open any other file.

When done, reply with ONE line per document: sha256 prefix (12 chars), pages written, number of pages marked reveals_test_outcome yes/unsure, number marked needs_image. Do not include any card content or any test outcome in your reply."""


def brief(bid, only=None):
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    docs = {d["sha"]: d for d in idx["documents"]}
    b = next(x for x in idx["batches"] if x["id"] == bid)
    lines = []
    for sha in b["documents"]:
        if only and sha not in only:
            continue
        n = docs[sha]["pages"]
        inp = f"{WIN}\\runs\\dev\\data_docs\\inputs\\{sha}.txt"
        if n <= PART:
            outs = f"{WIN}\\runs\\dev\\data_docs\\cards\\{sha}.jsonl"
        else:
            k = (n + PART - 1) // PART
            outs = ", ".join(f"{WIN}\\runs\\dev\\data_docs\\cards\\{sha}.p{i:02d}.jsonl" for i in range(1, k + 1))
        lines.append(f"- {inp} ({n} pages) -> {outs}")
    return HEAD.format(win=WIN, bid=bid, docs="\n".join(lines), part=PART)


if __name__ == "__main__":
    print(brief(sys.argv[1], set(sys.argv[2:]) or None))

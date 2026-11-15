"""Document lane, the visual pass, on Muse Spark 1.3 Contributor (public NASA data only).

For every page the text pass marked needs_image (or low confidence), Muse sees the page image and
the whole document's text as context, and rewrites the page card with the sharpened label rules
below. Sheets of tables are not images: their needs_image flag is cleared, not sent.

The document text is the same leading part of every request for that document, so the provider
caches it. Pages go in runs of up to RUN images. Output: runs/dev/data_docs/visual/<sha>.jsonl, one
row per page sent, schema ignis.docs.page_card/2. Re-runnable: pages already written are skipped.

    python tools/wsp/docs/visual_pass.py --pilot 3     # three documents
    python tools/wsp/docs/visual_pass.py --sample      # the 200-page check sample, all of it
    python tools/wsp/docs/visual_pass.py               # everything flagged
"""
import argparse
import glob
import io
import json
import re
import sys
import threading
import time
from collections import Counter
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import muse  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
OUT = REPO / "runs/dev/data_docs"
ASTRA = Path("runs/dev/data_astra")
sys.path.insert(0, str(ASTRA / "work/document_pages/dependencies"))
import fitz  # noqa: E402

RUN, DPI, WORKERS = 8, 110, 24
TABLE_KINDS = {"xls", "xlsx", "csv"}

RULES = """You label pages of NASA microgravity combustion documents (the PSI repository). You receive the WHOLE document's extracted text as context, then images of some of its pages. For EACH page image, return one card.

Return ONLY a JSON object {"cards": [ ... ]}, one element per page image, in the order given:
{"page": <n>, "kind": "<KIND>", "summary": "<at most 40 words>",
 "tests": ["<test ids the page speaks about, matched to the catalogue list in the header when clear, else as written>"],
 "quantities": [{"name": "...", "unit": "..."}],
 "flame_type": [<FLAME>], "studied": [<STUDIED>], "fire_safety_need": [<NEED>],
 "reveals_test_outcome": "yes" | "no" | "unsure", "is_summary_or_conclusion": true | false,
 "confidence": "high" | "medium" | "low",
 "enrichment": {"concepts": [...], "synonyms": ["ABBR = expansion"], "queries": ["3 to 6 phrasings a researcher would search with"], "instruments": [...], "materials": [...]}}

KIND, decided in this order (take the first that fits):
1. blank_or_unreadable: nothing legible.
2. drawing_or_schematic: the page is mostly a drawing, diagram, photo of hardware or schematic (even if it shows apparatus).
3. results_figure: the page is mostly a plot, chart or image of measured or computed results.
4. results_table: the page is mostly a table of measured or computed results.
5. test_matrix: the page is mostly a table of planned test conditions, without results.
6. data_listing: raw numbers, logs or listings, without explanation.
7. otherwise by what the text is about: front_matter (title, cover, signatures), contents_or_lists (contents, lists of figures, acronyms), requirements (shall-statements, specifications), apparatus (description of hardware), methods (how quantities are measured or computed), procedure_or_script (operating steps, crew procedures, timelines), discussion (results described in prose), conclusions_or_summary (abstract, summary, conclusions), references, administrative (revision history, distribution, approvals).

FLAME (the phenomenon the page is about; empty list when it is about no combustion phenomenon, never the word "none" unless the page is explicitly about a no-combustion case): spherical_diffusion_flame, flat_burner_diffusion_flame, jet_diffusion_flame, coflow_jet_diffusion_flame, droplet_flame, solid_surface_flame, overheat_smoke, none.
STUDIED (only what the page itself studies or reports): cool_flame, smoke_point, extinction, electric_field, liftoff, burning_rate, aerosol.
NEED (only when the page itself bears on it, not because it is about combustion in general): prevention (flammability limits, ignition, flame spread, material selection), detection (smoke, aerosol, detectors), suppression (extinction by flow, diluent, reduced oxygen, extinguishers).

reveals_test_outcome: "yes" when the page states or shows, for individual tests or runs, how they ended or what they measured (extinction, burn time, diameters, flame size, spread rate, temperatures, smoke, a plot or table of measured results, a model compared with a test); "unsure" whenever in doubt; "no" only for requirements, design, apparatus, planned conditions, procedures, administration, references, theory.

Write in English, factual, from the page and the document only. Do not invent values. No quote longer than 15 words."""


def pdf_for(doc, src_map):
    sha = doc["sha"]
    conv = ASTRA / "work/document_pages/conversions" / sha / "converted.pdf"
    if conv.exists():
        return conv
    ppt = OUT / "rest/pdf" / f"{sha}.pdf"
    if ppt.exists():
        return ppt
    p = src_map.get(sha)
    return p if p and p.exists() and p.suffix.lower() == ".pdf" else None


def load_cards():
    cards = {}
    for f in glob.glob(str(OUT / "cards" / "*.jsonl")):
        for line in io.open(f, encoding="utf-8"):
            if line.strip():
                r = json.loads(line)
                if r.get("schema") == "ignis.docs.page_card/1":
                    cards[(r["document_sha256"], r["page"])] = r
    return cards


def done_pages(sha):
    p = OUT / "visual" / f"{sha}.jsonl"
    if not p.exists():
        return set()
    return {json.loads(l)["page"] for l in io.open(p, encoding="utf-8") if l.strip()}


lock = threading.Lock()


def run_doc(doc, pages, pdf):
    text = io.open(REPO / doc["input"], encoding="utf-8").read()
    pdfdoc = fitz.open(pdf)
    out_rows, stats = [], Counter()
    for i in range(0, len(pages), RUN):
        chunk = pages[i:i + RUN]
        content = [{"type": "text", "text": "WHOLE DOCUMENT TEXT (context):\n\n" + text}]
        content.append({"type": "text", "text": f"Now the images of pages {', '.join(map(str, chunk))}, in this order. Return one card per page."})
        for n in chunk:
            png = pdfdoc[n - 1].get_pixmap(dpi=DPI).tobytes("png")
            content.append({"type": "text", "text": f"PAGE {n}:"})
            content.append(muse.image_part(png))
        msgs = [{"role": "system", "content": RULES}, {"role": "user", "content": content}]
        try:
            r = muse.call(msgs, response_format={"type": "json_object"}, effort="high")
            raw = r["choices"][0]["message"]["content"]
            raw = re.sub(r"^```(json)?|```$", "", raw.strip(), flags=re.M)
            got = {c["page"]: c for c in json.loads(raw)["cards"] if isinstance(c, dict) and "page" in c}
            u = r.get("usage", {})
            stats["prompt_tokens"] += u.get("prompt_tokens", 0); stats["completion_tokens"] += u.get("completion_tokens", 0)
            stats["cached_tokens"] += (u.get("prompt_tokens_details") or {}).get("cached_tokens", 0)
        except Exception as e:  # recorded, the pages stay unsent and are retried next run
            stats["failed_runs"] += 1; stats["error:" + type(e).__name__] += 1
            continue
        for n in chunk:
            c = got.get(n)
            if not c:
                stats["missing_page"] += 1; continue
            c.update({"schema": "ignis.docs.page_card/2", "document_sha256": doc["sha"], "page": n,
                      "read_by": muse.MODEL, "image_dpi": DPI})
            out_rows.append(c); stats["pages"] += 1
    if out_rows:
        with lock, io.open(OUT / "visual" / f"{doc['sha']}.jsonl", "a", encoding="utf-8", newline="\n") as f:
            for c in out_rows:
                f.write(json.dumps(c, ensure_ascii=False) + "\n")
    return stats


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pilot", type=int, default=0)
    ap.add_argument("--sample", action="store_true")
    a = ap.parse_args()
    idx = json.load(io.open(OUT / "index.json", encoding="utf-8"))
    src_map = {}
    for line in io.open(ASTRA / "work/document_pages/documents.jsonl", encoding="utf-8"):
        r = json.loads(line)
        if "document_sha256" in r and r["source"].get("local_path"):
            src_map[r["document_sha256"]] = ASTRA / r["source"]["local_path"]
    up = {}
    for line in io.open(ASTRA / "logs/upload_verified.jsonl", encoding="utf-8"):
        r = json.loads(line); up[(r["investigation"], r["file_name"])] = Path(r["source_path"])
    for d in idx["documents"]:
        if d.get("origin") == "extract_rest" and d["sha"] not in src_map:
            s = d["sources"][0]; p = up.get((s["investigation"], s["file_name"]))
            if p: src_map[d["sha"]] = p
    cards = load_cards()
    (OUT / "visual").mkdir(exist_ok=True)
    sample = {(p["sha"], p["page"]) for p in idx["check_sample"]["pages"]}
    jobs, cleared, no_pdf = [], 0, Counter()
    for d in idx["documents"]:
        if d.get("kind") in TABLE_KINDS or (d.get("origin") == "extract_rest" and d.get("kind") == "txt"):
            cleared += sum(1 for n in range(1, d["pages"] + 1) if (cards.get((d["sha"], n)) or {}).get("needs_image"))
            continue
        if a.sample:
            pages = [n for n in range(1, d["pages"] + 1) if (d["sha"], n) in sample]
        else:
            pages = [n for n in range(1, d["pages"] + 1)
                     if (cards.get((d["sha"], n)) or {}).get("needs_image") or (cards.get((d["sha"], n)) or {}).get("confidence") == "low"]
        pages = [n for n in pages if n not in done_pages(d["sha"])]
        if not pages:
            continue
        pdf = pdf_for(d, src_map)
        if pdf is None:
            no_pdf[d.get("kind", "pdf")] += len(pages); continue
        jobs.append((d, pages, pdf))
    if a.pilot:
        jobs = sorted(jobs, key=lambda j: len(j[1]))[:a.pilot]
    print(f"{len(jobs)} documents, {sum(len(j[1]) for j in jobs)} pages to send; table sheets cleared: {cleared}; "
          f"pages without a PDF: {dict(no_pdf)}", flush=True)
    total, t0 = Counter(), time.time()
    with ThreadPoolExecutor(WORKERS) as ex:
        futs = {ex.submit(run_doc, *j): j[0]["sha"] for j in jobs}
        for k, f in enumerate(as_completed(futs), 1):
            total += f.result()
            if k % 10 == 0 or k == len(jobs):
                print(f"{k}/{len(jobs)} documents, {total['pages']} pages, {time.time() - t0:.0f} s, {dict(total)}", flush=True)
    print("done", dict(total))


if __name__ == "__main__":
    main()

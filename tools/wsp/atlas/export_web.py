"""Investigation cards for the IGNIS web app: the 19 atlas cards and their narratives, reduced to what
a researcher reads on the Conditions and Investigations views.

Kept: acronym, title, NASA sub-area, hardware family, period, fuels, typed fire-safety needs with
NASA's own words, the narrative (how tests differ, fire-safety relevance, findings with their source
file and page, what the documents do not hold), sensors, related investigations.
Left out: model scores (development numbers), file sizes, licence, the inputs the narratives were
written from. Numbers about conditions come from the server's conditions table, not from here.

    python tools/wsp/atlas/export_web.py <ignis checkout>/web/src/data/investigations.json
"""
import io
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
CARDS = REPO / "docs/wsp/atlas/cards"
NARR = REPO / "docs/wsp/atlas/narratives"


def src(s):
    if not s:
        return None
    return {k: s.get(k) for k in ("file", "page") if s.get(k) is not None} or None


def card(path):
    c = json.load(io.open(path, encoding="utf-8"))
    pid = c["investigation_id"]
    n = json.load(io.open(NARR / f"{pid}.json", encoding="utf-8")) if (NARR / f"{pid}.json").exists() else {}
    fuels = (c.get("tests") or {}).get("fuels_or_materials") or {}
    return {
        "id": pid,
        "acronym": c.get("acronym"),
        "title": c.get("title"),
        "sub_area": c.get("nasa_sub_area"),
        "hardware": (c.get("hardware") or {}).get("family"),
        "period": c.get("project_period_nasa_record"),
        "objective": (c.get("nasa_text") or {}).get("objective"),
        "fuels": sorted(fuels, key=lambda k: -fuels[k])[:6],
        "needs": [
            {"need": f["need"], "tier": f.get("tier"), "basis": f.get("basis"), "inference": f.get("inference")}
            for f in c.get("fire_safety") or []
        ],
        "gravity": (c.get("gravity") or {}).get("level"),
        "how_tests_differ": (n.get("how_tests_differ") or {}).get("text"),
        "relevance": (n.get("fire_safety_relevance") or {}).get("text"),
        "findings": [
            {"statement": f["statement"], "kind": f.get("kind"), "source": src(f.get("source"))}
            for f in (n.get("findings") or [])
        ],
        "sensors": [
            {"instrument": s.get("instrument"), "measures": s.get("measures"), "source": src(s.get("source"))}
            for s in (n.get("sensors_and_measurements") or [])
        ],
        "not_found": n.get("not_found") or [],
        "related": [{"id": r["investigation_id"], "acronym": r.get("acronym"), "shared": r.get("shared") or []} for r in c.get("related") or []],
        "written_by": n.get("written_by"),
    }


def main():
    out = Path(sys.argv[1])
    cards = [card(p) for p in sorted(CARDS.glob("PSI-*.json"), key=lambda p: int(p.stem.split("-")[1]))]
    doc = {"schema": "ignis.web.investigations/1", "source": "docs/wsp/atlas (cards + narratives)",
           "generated_text": "narrative fields are written by Claude Opus 5.5 from NASA's documents; each finding names its source file and page",
           "investigations": cards}
    out.parent.mkdir(parents=True, exist_ok=True)
    io.open(out, "w", encoding="utf-8", newline="\n").write(json.dumps(doc, ensure_ascii=False, indent=1) + "\n")
    print(len(cards), "investigations,", out.stat().st_size, "bytes")


if __name__ == "__main__":
    main()

"""Freezes the evaluation groups of BASS (PSI-26) by test day, and writes the per-track partition map.

PSI-26 was declared outside every partition before the split script existed and has no row in the test
catalogue. Its burn videos are named by date and time. This script takes the dates from the NAMES of the
videos in the measurer's download manifest (names only: no video, no track and no outcome is opened), applies
the project's split rule and seed, unchanged, and writes a new split file. It never rewrites a frozen one.

    python tools/wsp/atlas/freeze_psi26_days.py            # writes docs/wsp/atlas/splits/PSI-26-days.json
    python tools/wsp/atlas/freeze_psi26_days.py --check    # recomputes and compares, writes nothing
    python tools/wsp/atlas/freeze_psi26_days.py --map OUT  # partition map for the tracks that exist now

The map gives each track file stem (which begins with the date of its burn) the partition of its day. It is a
derived file: the split is the frozen object.
"""
import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from freeze_groups import RULE, SEED, split_groups  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
MANIFEST = REPO / "runs/dev/media/fabric/manifest_raw.jsonl"
TRACKS = REPO / "runs/dev/media/fabric/tracks/psi26"
OUT = REPO / "docs/wsp/atlas/splits/PSI-26-days.json"
DATE = re.compile(r"(20\d{2})[ _.-]?(\d{2})[ _.-]?(\d{2})")


def day_of(name):
    m = DATE.search(name)
    return "%s-%s-%s" % m.groups() if m else None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--map", default=None)
    args = ap.parse_args()
    names = []
    for line in MANIFEST.open(encoding="utf-8"):
        r = json.loads(line)
        key = str(r.get("key") or r.get("file_name") or "")
        if "PSI-26" in key or r.get("investigation") == "PSI-26":
            names.append(key.split("/")[-1])
    days = Counter(d for d in (day_of(n) for n in names) if d)
    undated = sorted(n for n in names if not day_of(n))
    parts = dict(zip(("train", "validation", "test"), split_groups(list(days))))
    of = {g: p for p, gs in parts.items() for g in gs}
    doc = {
        "schema": "ignis.atlas.split/1", "investigation_id": "PSI-26", "acronym": "BASS",
        "rule": RULE, "seed": SEED, "group_key": "the date in the name of the burn video",
        "frozen_on": "2026-11-15", "source": "names of the PSI-26 videos in runs/dev/media/fabric/manifest_raw.jsonl (no video, track or outcome opened)",
        "groups": parts, "videos": {p: sum(days[g] for g in gs) for p, gs in parts.items()},
        "undated_videos": undated,
        "note": "A burn whose video has no date in its name stays held. Tracks of test days are scored only in the final run.",
    }
    text = json.dumps(doc, indent=1) + "\n"
    if args.map:
        frozen = json.loads(OUT.read_text(encoding="utf-8"))
        of = {g: p for p, gs in frozen["groups"].items() for g in gs}
        m = {}
        for f in sorted(TRACKS.glob("*.json")):
            d = day_of(f.stem)
            if d in of:
                m[f.stem] = of[d]
        Path(args.map).write_text(json.dumps(m, indent=1) + "\n", encoding="utf-8", newline="\n")
        print("map:", dict(Counter(m.values())), "tracks; without a frozen day:", len(list(TRACKS.glob("*.json"))) - len(m))
        return 0
    if args.check:
        same = OUT.read_text(encoding="utf-8") == text
        print("identical to the frozen file" if same else "DIFFERS from the frozen file")
        return 0 if same else 1
    if OUT.exists():
        sys.exit(f"{OUT} is frozen and is not rewritten")
    OUT.write_text(text, encoding="utf-8", newline="\n")
    print("videos:", len(names), "days:", {p: len(gs) for p, gs in parts.items()}, "videos per partition:", doc["videos"], "undated:", len(undated))
    return 0


if __name__ == "__main__":
    sys.exit(main())

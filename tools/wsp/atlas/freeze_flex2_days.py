"""Freezes the evaluation groups of FLEX-2 (PSI-68) by test day, and selects the tests of the first image batch.

FLEX-2 was held whole on 14 November because NASA's table is a test matrix (docs/wsp/atlas/splits/PSI-68.json:
"a group id that is not listed here is held until this file's rule has been applied to it in a new file").
The per-test catalogue of 15 November (tools/wsp/ext/flex2_catalogue.py, conditions only) gives every rack test
point a date. This script applies the project's split rule and seed, unchanged, to those dates and writes a
NEW file; it never rewrites a frozen one. It reads conditions only: no frame, no outcome.

    python tools/wsp/atlas/freeze_flex2_days.py            # writes splits/PSI-68-days.json and the batch list
    python tools/wsp/atlas/freeze_flex2_days.py --check    # recomputes and compares, writes nothing

The batch (second part): the tests whose frames are downloaded and measured first. The rule is fixed here,
before any frame is fetched, and uses conditions and file sizes only:
  - a single free droplet of a pure fuel that a droplet candidate covers, inside the declared domain of the
    forward by its measured conditions, with a backlit archive, on a TRAIN or VALIDATION day;
  - every such n-heptane and iso-octane test;
  - n-octane and n-decane: per day in date order, the first two test points by name, days taken in date order
    alternating between the two fuels, until the batch holds CAP_BYTES.
Test days are not in the batch: their frames stay on NASA's server until the final run.
"""
import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from freeze_groups import RULE, SEED, split_groups  # noqa: E402

REPO = Path(__file__).resolve().parents[3]
CAT = REPO / "runs/dev/data_all/work/atlas/flex2_tests.jsonl"
OUT = REPO / "docs/wsp/atlas/splits/PSI-68-days.json"
BATCH = REPO / "docs/wsp/atlas/splits/PSI-68-batch1.json"
CAP_BYTES = 95 * 1024 ** 3


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    rows = [json.loads(line) for line in CAT.open(encoding="utf-8")]
    dated = [r for r in rows if r.get("group")]
    parts = dict(zip(("train", "validation", "test"), split_groups([r["group"] for r in dated])))
    of = {g: p for p, gs in parts.items() for g in gs}
    counts = Counter(of[r["group"]] for r in dated)
    doc = {
        "schema": "ignis.atlas.split/1", "investigation_id": "PSI-68", "acronym": "FLEX-2",
        "rule": RULE, "seed": SEED, "group_key": "the test day of the per-test catalogue (GMT date of igniter on)",
        "frozen_on": "2026-11-15", "supersedes_for_dated_tests": "PSI-68.json (held whole: no per-test list then)",
        "catalogue": "runs/dev/data_all/work/atlas/flex2_tests.jsonl (conditions only; tools/wsp/ext/flex2_catalogue.py)",
        "groups": {p: sorted(gs) for p, gs in parts.items()},
        "test_points": {p: counts.get(p, 0) for p in ("train", "validation", "test")},
        "undated_test_points": sorted(r["test_id"] for r in rows if not r.get("group")),
        "note": "Undated test points stay held. Frames of test days are not downloaded before the final run.",
    }
    # ---- the first image batch
    ok = [r for r in dated if r.get("single_pure_candidate") and r.get("inside_domain")
          and r.get("backlit_archive_bytes") and of[r["group"]] in ("train", "validation")]
    chosen, total = [], 0
    for r in sorted(ok, key=lambda r: (r["group"], r["test_id"])):
        if r["fuel"] in ("n-heptane", "iso-octane"):
            chosen.append(r); total += r["backlit_archive_bytes"]
    by = {f: defaultdict(list) for f in ("n-octane", "n-decane")}
    for r in sorted(ok, key=lambda r: (r["group"], r["test_id"])):
        if r["fuel"] in by:
            by[r["fuel"]][r["group"]].append(r)
    queues = {f: [d[g][:2] for g in sorted(d)] for f, d in by.items()}
    full = False
    while not full and any(queues.values()):
        for f in ("n-octane", "n-decane"):
            if not queues[f]:
                continue
            for r in queues[f].pop(0):
                if total + r["backlit_archive_bytes"] > CAP_BYTES:
                    full = True
                    break
                chosen.append(r); total += r["backlit_archive_bytes"]
            if full:
                break
    batch = {
        "schema": "ignis.atlas.batch/1", "investigation_id": "PSI-68", "frozen_on": "2026-11-15",
        "rule": __doc__.split("The batch (second part):")[1].strip(), "cap_bytes": CAP_BYTES, "bytes": total,
        "tests": [{"test_id": r["test_id"], "date": r["group"], "partition": of[r["group"]], "fuel": r["fuel"],
                   "candidate": r.get("candidate"), "archive": r["test_id"] + "A.zip",
                   "bytes": r["backlit_archive_bytes"]} for r in chosen],
    }
    text, btext = json.dumps(doc, indent=1) + "\n", json.dumps(batch, indent=1) + "\n"
    if args.check:
        same = OUT.read_text(encoding="utf-8") == text and BATCH.read_text(encoding="utf-8") == btext
        print("identical to the frozen files" if same else "DIFFERS from the frozen files")
        return 0 if same else 1
    for p in (OUT, BATCH):
        if p.exists():
            sys.exit(f"{p} is frozen and is not rewritten")
    OUT.write_text(text, encoding="utf-8", newline="\n")
    BATCH.write_text(btext, encoding="utf-8", newline="\n")
    print("days:", {p: len(gs) for p, gs in parts.items()}, "test points:", dict(counts))
    c = Counter((t["fuel"], t["partition"]) for t in batch["tests"])
    print("batch: %d tests, %.1f GB" % (len(chosen), total / 1024 ** 3))
    for k in sorted(c):
        print("  ", k, c[k])
    elig = Counter((r["fuel"], of[r["group"]]) for r in ok)
    print("eligible on train and validation days:", dict(elig))
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Counts over the measured tracks and the manifest of raw files (sha256, source key, url).

Prints per-burn outcomes and class counts for TRAIN only; for held partitions, unpartitioned PSI-25
burns and PSI-26 it prints counts of usable tracks only.

    python -I tools/wsp/media/fabric/summarize.py --root runs/dev/media/fabric
"""
import argparse
import json
from collections import Counter
from pathlib import Path


def usable(d):
    if d.get("quality") == "failed":
        return False
    k = d["outcome"]["kind"]
    return k == "never_lit" or (k != "cannot_tell" and bool(d.get("duct"))) or bool(d.get("duct"))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    args = ap.parse_args()
    root = Path(args.root)
    rows = []
    for f in sorted((root / "tracks").rglob("*.json")):
        d = json.loads(f.read_text(encoding="utf-8"))
        inv = "PSI-25" if "psi25" in f.parts else "PSI-26"
        part = f.parent.name if inv == "PSI-25" else "none"
        rows.append(dict(file=str(f.relative_to(root)).replace("\\", "/"), inv=inv, part=part, role=d.get("file_role", "primary"),
                         run=d["run_id"], kind=d["outcome"]["kind"], conf=d["outcome"]["confidence"],
                         quality=d.get("quality"), usable=usable(d), n=len(d.get("duct") or []),
                         sibal="sibal" in (d.get("material") or "").lower()))
    summary = {}
    for inv in ("PSI-25", "PSI-26"):
        for part in sorted({r["part"] for r in rows if r["inv"] == inv}):
            sel = [r for r in rows if r["inv"] == inv and r["part"] == part]
            prim = [r for r in sel if r["role"] == "primary"]
            e = {"files": len(sel), "burns (primary files)": len(prim),
                 "usable": sum(r["usable"] for r in prim),
                 "quality": dict(Counter(r["quality"] for r in prim))}
            if part == "train":
                e["per_burn"] = {r["run"]: f"{r['kind']} ({r['conf']}), {r['n']} track points" for r in prim}
                e["outcome_classes"] = dict(Counter(r["kind"] for r in prim))
            summary[f"{inv} {part}"] = e
    (root / "summary_counts.json").write_text(json.dumps(summary, indent=1), encoding="utf-8")
    # manifest of raw files
    raw = [json.loads(l) for l in (root / "manifest_raw.jsonl").read_text(encoding="utf-8").splitlines()]
    uniq = {}
    for r in raw:
        uniq[r["file_name"]] = r
    man = {"note": "every raw burn video downloaded for this measurement; fetch again with the stable_url and check sha256",
           "files": sorted(uniq.values(), key=lambda r: (r["investigation"], r["file_name"])),
           "total_bytes": sum(r["size_bytes"] for r in uniq.values())}
    (root / "manifest.json").write_text(json.dumps(man, indent=1), encoding="utf-8")
    print(json.dumps(summary, indent=1))
    print("raw files:", len(uniq), "bytes:", man["total_bytes"])


if __name__ == "__main__":
    main()

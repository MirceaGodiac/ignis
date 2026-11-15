#!/usr/bin/env python3
"""Rebuild every served hand-written release from its committed patch and compare the ids.

For each release in configs/models/served/served.json: find its candidate directory under
configs/models (the release.json whose id it is), copy its parent release into a temporary
registry, run `ignis wsp assemble` on the committed patch.json and binding.json, and check that
the rebuilt release has the served id and that its record equals the served record (all fields but
created_at). Nothing is written outside the temporary directory; no number is touched.

    python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry with parents>
"""
import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

KEYS = {
    "flex_methanol.wsp2": "flex_methanol2",
    "flex_heptane.wsp2": "flex_heptane2",
    "acme_normal.wsp12": "acme_normal12",
    "acme_normal.wsp13": "acme_normal13",
    "acme_inverse.wsp12": "acme_inverse12",
    "bass_front.wsp": "bass_front",
}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", required=True)
    ap.add_argument("--parents", required=True, help="registry root holding models/<parent hex>/")
    a = ap.parse_args()
    served = Path("configs/models/served")
    entries = json.loads((served / "served.json").read_text(encoding="utf-8"))["releases"]
    candidates = {}
    for p in Path("configs/models").rglob("release.json"):
        if served in p.parents:
            continue
        try:
            candidates[json.loads(p.read_text(encoding="utf-8"))["id"]] = p.parent
        except (ValueError, KeyError):
            pass
    ok = True
    for e in entries:
        rid, name = e["release_id"], e["model_name"]
        cand = candidates.get(rid)
        record = json.loads((served / "models" / rid[7:] / "release.json").read_text(encoding="utf-8"))
        if cand is None:
            print(f"FAIL {name} {rid}: no candidate directory under configs/models")
            ok = False
            continue
        parent = record["parent_id"]
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "registry"
            if parent:
                src = Path(a.parents) / "models" / parent[7:]
                shutil.copytree(src, root / "models" / parent[7:])
            out = Path(tmp) / "rebuilt.json"
            r = subprocess.run(
                [a.ignis, "wsp", "assemble", "--family", KEYS.get(name, name.removesuffix(".wsp")), "--patch", str(cand / "patch.json"),
                 "--binding", str(cand.parent / "binding.json"), "--root", str(root), "--out", str(out)],
                capture_output=True, text=True)
            if r.returncode != 0:
                print(f"FAIL {name}: assemble: {r.stderr.strip()[:400]}")
                ok = False
                continue
            rebuilt = json.loads(out.read_text(encoding="utf-8"))
            same_id = rebuilt["id"] == rid
            strip = lambda d: {k: v for k, v in d.items() if k != "created_at"}
            same_record = strip(rebuilt) == strip(record)
            ok &= same_id and same_record
            print(f"{'OK  ' if same_id and same_record else 'FAIL'} {name} {cand.as_posix()}: rebuilt {rebuilt['id']} "
                  f"(served {rid}); record identical: {same_record}; forward_hash {rebuilt['construction']['forward_hash']}")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()

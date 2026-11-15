"""Write a failed track for every planned PSI-25 file whose video cannot be decoded (no track written).
    python -I tools/wsp/media/fabric/stub_undecodable.py --root runs/dev/media/fabric --only PSI-25
"""
import argparse
import importlib.util
import json
from pathlib import Path

import cv2

ap = argparse.ArgumentParser()
ap.add_argument("--root", required=True)
ap.add_argument("--only", required=True)
args = ap.parse_args()
root = Path(args.root)
spec = importlib.util.spec_from_file_location("ra", Path(__file__).with_name("run_all.py"))
ra = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ra)
for j in ra.plan(root):
    if j["inv"] != args.only or j["out"].exists():
        continue
    v = root / j["file"]["local"]
    cap = cv2.VideoCapture(str(v))
    ok = cap.isOpened() and cap.read()[0]
    reason = "the archive file cannot be decoded (mp4 without a moov atom: the file is truncated at the source, its size equals the listing)" if not ok else "no track written (record too short or no dark record)"
    d = {"schema": "wsp-bass-front-track/1", "run_id": j["run_id"], "partition": j["partition"], "material": j["material"],
         "file_role": j["role"], "source": {"video": j["file"]["file_name"], "key": j["file"]["key"], "sha256": j["file"]["sha256"]},
         "quality": "failed", "qc_flags": ["R0 " + reason], "outcome": {"kind": "cannot_tell", "confidence": "low", "reason": reason},
         "duct": [], "igniter": None, "notes": [reason]}
    j["out"].parent.mkdir(parents=True, exist_ok=True)
    j["out"].write_text(json.dumps(d, indent=1), encoding="utf-8")
    print("stub", j["out"].name)

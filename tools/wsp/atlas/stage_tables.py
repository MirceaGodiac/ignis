"""Copy the experiment tables that normalize_tests.py reads into runs/dev/data_all/raw/samples/PSI-<id>/.

normalize_tests.py looks for its tables flat under raw/samples/PSI-<id>/ (the layout of the earlier
sample download). This copies, byte for byte and without opening them, the downloaded files of the
category "Experimental table" of the 15 new investigations, plus the CFI data summary workbook, from
runs/dev/data_all/raw/<PSI-id>/ (fetch_text.py) to that place. The sha256 of each copy is checked
against MANIFEST.jsonl.

Usage (from the repository root):  python -I tools/wsp/atlas/stage_tables.py
"""
import hashlib
import json
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RAW = ROOT / "runs/dev/data_all/raw"
NEW_IDS = [10, 20, 21, 22, 23, 39, 47, 68, 98, 99, 100, 101, 102, 106, 107]
EXTRA = {"PSI-39": ("CFI_DataSummary_Spreadsheet_Jan2021.xlsx",)}


def main():
    manifest = {}
    for line in (RAW / "MANIFEST.jsonl").open(encoding="utf-8"):
        r = json.loads(line)
        manifest[r["key"]] = r
    n = 0
    missing = []
    for i in NEW_IDS:
        inv = f"PSI-{i}"
        for line in (ROOT / "runs/dev/data_all/manifests" / f"{inv}.listing.jsonl").open(encoding="utf-8"):
            r = json.loads(line)
            want = r["category"] == "Experimental table" or r["file_name"].endswith(EXTRA.get(inv, ("\0",)))
            m = manifest.get(r["key"])
            if not want or m is None:
                continue
            if m["status"] not in ("ok", "reused"):
                missing.append(r["key"])
                continue
            src = RAW / m["local_path"]
            dst = RAW / "samples" / inv / r["file_name"]
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, dst)
            if hashlib.sha256(dst.read_bytes()).hexdigest() != m["sha256"]:
                raise SystemExit(f"sha256 differs after copy: {dst}")
            n += 1
    print(json.dumps(dict(staged=n, missing=missing)))
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())

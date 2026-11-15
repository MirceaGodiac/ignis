#!/usr/bin/env python3
"""Adds the script's programmed stop to the ACME records of a PRIVATE copy of the data root.

Every CFI-G test is run by a script whose stage durations are set before ignition and written to
NASA's "as run" parameter file of the test (`PSI-159_Raw Data_as run<test>.txt`: key:value lines,
script and camera settings only, no measurement). This tool reads four kinds of keys from it,

    stabilizationTime1Sec + stabilizationTime2Sec + ignitionToTestRampTimeSec
    + constantFlowTime1Sec + testFlowRamp1To2TimeSec + constantFlowTime2Sec
    + testFlowRamp2To3TimeSec + constantFlowTime3Sec

and writes the sum, in seconds from the script's ignition stamp to the end of its last constant-flow
stage, as the condition `programmed_stop` of the run in `<root>/work/records/<family>/experiments.jsonl`.
It is a prospective input: the value exists before the test starts and says nothing about how the
test ended.

Rules:
  * the root must be a private copy (it refuses the shared `data` folder);
  * only train and validation runs of the frozen splits are touched (the test partition is not
    readable through tools/wsp/analysis/lib.py and is not touched here);
  * "as run" files are fetched only when missing, from the URL in data/manifests/PSI-159.listing.jsonl,
    and nothing of their content is printed: the report is counts only.

    python tools/wsp/ext/acme_script.py --root runs/dev/data_ext [--fetch]
"""
import argparse
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "tools/wsp/analysis"))
import lib  # noqa: E402

KEYS = ["stabilizationTime1Sec", "stabilizationTime2Sec", "ignitionToTestRampTimeSec", "constantFlowTime1Sec",
        "testFlowRamp1To2TimeSec", "constantFlowTime2Sec", "testFlowRamp2To3TimeSec", "constantFlowTime3Sec"]
CONDITION = "programmed_stop"


def parse(path: Path):
    vals = {}
    for line in path.read_text(encoding="utf-8", errors="replace").replace("\r", "\n").split("\n"):
        key, sep, value = line.strip().rstrip(",").partition(":")
        if sep and key in KEYS:
            try:
                vals[key] = float(value)
            except ValueError:
                return None
    if "constantFlowTime1Sec" not in vals:
        return None
    return sum(vals.get(k, 0.0) for k in KEYS)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True, help="private copy of the data root")
    ap.add_argument("--files", default="runs/dev/nasa/PSI-159", help="folder holding the 'as run' files")
    ap.add_argument("--fetch", action="store_true", help="download missing 'as run' files of train and validation runs")
    args = ap.parse_args()
    root = (REPO / args.root).resolve()
    if root == (REPO / "data").resolve():
        sys.exit("refusing to modify the shared data root: give a private copy")
    files = REPO / args.files
    files.mkdir(parents=True, exist_ok=True)
    listing = {}
    for line in (REPO / "data/manifests/PSI-159.listing.jsonl").read_text(encoding="utf-8").splitlines():
        f = json.loads(line)
        if f["file_name"].startswith("PSI-159_Raw Data_as run"):
            listing[f["file_name"]] = f["stable_url"]
    report = {}
    for family in ("acme_normal", "acme_inverse"):
        ids = set(lib.partition_ids(family, "train")) | set(lib.partition_ids(family, "validation"))
        path = root / "work/records" / family / "experiments.jsonl"
        out, n = [], dict(runs=0, written=0, fetched=0, no_file=0, unreadable=0)
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            rec = json.loads(line)
            if rec["id"] in ids:
                n["runs"] += 1
                test = rec["id"].split(":")[-1]
                name = f"PSI-159_Raw Data_as run{test}.txt"
                local = files / name
                if not local.exists() and args.fetch and name in listing:
                    r = subprocess.run(["curl", "-sS", "-L", "-A", "research-client", "-o", str(local), listing[name]],
                                       capture_output=True, text=True)
                    if r.returncode == 0 and local.exists() and local.stat().st_size > 200:
                        n["fetched"] += 1
                    elif local.exists():
                        local.unlink()
                    time.sleep(0.3)
                value = parse(local) if local.exists() else None
                if not local.exists():
                    n["no_file"] += 1
                elif value is None:
                    n["unreadable"] += 1
                if value is not None and value > 0:
                    rec["conditions"][CONDITION] = {
                        "derivation": {"agent": None, "inputs": [], "method": "deterministic",
                                       "method_id": "wsp.ext.acme.cond.programmed_stop", "method_version": "1",
                                       "notes": "sum of the script's stage durations up to the end of its last constant-flow stage, "
                                                "from the test's 'as run' parameter file; set before ignition"},
                        "evidence_class": "source_derived", "flags": [],
                        "locator": {"asset_sha256": hashlib.sha256(local.read_bytes()).hexdigest(), "char_end": None,
                                    "char_start": None, "column": "stage durations", "frame": None, "page": None,
                                    "path": name, "pts_s": None, "row": None},
                        "raw_unit": "s", "raw_value": value, "resolution": "resolved",
                        "uncertainty_reason": "not reported by source", "uncertainty_si": None, "unit_si": "s", "value_si": value,
                    }
                    n["written"] += 1
                else:
                    rec["conditions"].pop(CONDITION, None)
            out.append(json.dumps(rec, ensure_ascii=False))
        path.write_text("\n".join(out) + "\n", encoding="utf-8", newline="\n")
        report[family] = n
    print(json.dumps(report, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())

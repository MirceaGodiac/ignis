#!/usr/bin/env python3
"""Run the frozen measurement (measure.py) on every downloaded burn video, in parallel.

Run with `python -I`. Reads runs/dev/media/fabric/manifest_raw.jsonl (written by fetch_videos.py),
maps each PSI-25 file to its test id and partition (project split bass-4083156d5b18dab4; the
test-partition ids are only used to place files, no outcome is read), and writes:
  tracks/psi25/{train,held_validation,held_test,unpartitioned}/<test>[__<variant>].json
  tracks/psi26/<video stem>.json
  check/<investigation>_<name>.jpg   (six-panel contact sheets)
  ovl/<investigation>_<name>.npz     (overlay strips, 1 per second)
It prints counts only, never an outcome of a held or PSI-26 burn.

    python -I tools/wsp/media/fabric/run_all.py --root runs/dev/media/fabric --workers 12 [--only PSI-25]
"""
import argparse
import csv
import hashlib
import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
MEASURE = Path(__file__).with_name("measure.py")
SPLIT = REPO / "data/splits/bass-4083156d5b18dab4.json"
TABLE = REPO / "data/raw/samples/PSI-25/PSI-25_Experimental table_BASS-II.csv"

ID_RE = re.compile(r"test[ _\-]?([a-z]\d+[ab]?)(?![0-9])", re.I)


def partitions():
    d = json.loads(SPLIT.read_text(encoding="utf-8"))["split"]
    part = {}
    for p in ("train", "validation"):
        for g in d[p]:
            part[g] = p
    return {r.split(":", 1)[1]: {"train": "train", "validation": "held_validation"}.get(part.get(g), "held_test")
            for r, g in d["run_to_group"].items()}


def materials():
    """Test id -> fuel sample material (only these two columns are read)."""
    out = {}
    with open(TABLE, encoding="utf-8-sig") as f:
        for r in csv.DictReader(f):
            out[r["Test #"].strip()] = " ".join(r["Fuel Sample Material"].split())
    return out


def psi25_id(name):
    stem = name.replace("PSI-25_Raw Data_", "")
    m = ID_RE.search(stem)
    if m:
        tid = m.group(1)
        tid = tid[0].upper() + tid[1:]
        return tid, stem
    m = re.match(r"([TOMPBF]\d+[ab]?)[-_]", stem)
    if m:
        return m.group(1), stem
    return None, stem


def is_exact(tid, stem):
    s = stem.lower()
    return bool(re.fullmatch(r"bass_gmt_?\d+_sample_?\d+_test[ _]?" + tid.lower() + r"\.mp4", s))


def plan(root):
    rows = [json.loads(l) for l in (root / "manifest_raw.jsonl").read_text(encoding="utf-8").splitlines()]
    seen = {}
    for r in rows:
        seen[r["file_name"]] = r
    rows = list(seen.values())
    part = partitions()
    mat = materials()
    jobs = []
    by_id = {}
    for r in rows:
        if r["investigation"] != "PSI-25":
            continue
        tid, stem = psi25_id(r["file_name"])
        by_id.setdefault(tid, []).append((r, stem))
    for tid, files in by_id.items():
        exact = [f for f in files if tid and is_exact(tid, f[1])]
        if exact:
            primary = exact[0]
        else:
            pref = [f for f in files if "-all" in f[1].lower()] or [f for f in files if "second" in f[1].lower()]
            primary = pref[0] if pref else max(files, key=lambda f: f[0]["size_bytes"])
        folder = part.get(tid, "unpartitioned") if tid else "unpartitioned"
        for f in files:
            r, stem = f
            base = tid if tid else re.sub(r"[^A-Za-z0-9]+", "_", stem[:-4])
            if f is not primary:
                var = re.sub(r"[^A-Za-z0-9]+", "_", stem[:-4])
                base = f"{base}__{var[-40:]}"
            jobs.append(dict(inv="PSI-25", file=r, run_id=f"PSI-25:{tid}" if tid else f"PSI-25:{stem[:-4]}",
                             partition=folder, material=mat.get(tid), role="primary" if f is primary else "secondary",
                             out=root / "tracks/psi25" / folder / f"{base}.json",
                             sheet=root / "check" / f"psi25_{base}.jpg", ovl=root / "ovl" / f"psi25_{base}.npz"))
    for r in rows:
        if r["investigation"] != "PSI-26":
            continue
        stem = re.sub(r"[^A-Za-z0-9]+", "_", r["file_name"].replace("PSI-26_Raw Data_", "")[:-4])
        jobs.append(dict(inv="PSI-26", file=r, run_id=f"PSI-26:{stem}", partition=None, material=None, role="primary",
                         out=root / "tracks/psi26" / f"{stem}.json", sheet=root / "check" / f"psi26_{stem}.jpg",
                         ovl=root / "ovl" / f"psi26_{stem}.npz"))
    return jobs


def run(job, root):
    r = job["file"]
    cmd = [sys.executable, "-I", "-W", "ignore", str(MEASURE), "--video", str(root / r["local"]), "--run-id", job["run_id"],
           "--out", str(job["out"]), "--sheet", str(job["sheet"]), "--ovl", str(job["ovl"]),
           "--source-key", r["key"], "--source-sha256", r["sha256"]]
    if job["partition"]:
        cmd += ["--partition", job["partition"]]
    if job["material"]:
        cmd += ["--material", job["material"]]
    env = dict(os.environ, OMP_NUM_THREADS="1", OPENBLAS_NUM_THREADS="1", MKL_NUM_THREADS="1", OPENCV_FOR_THREADS_NUM="1")
    p = subprocess.run(cmd, capture_output=True, text=True, env=env)
    if p.returncode == 0 and job["out"].exists():
        d = json.loads(job["out"].read_text(encoding="utf-8"))
        d["file_role"] = job["role"]
        job["out"].write_text(json.dumps(d, indent=1), encoding="utf-8")
        return True, ""
    return False, (p.stderr or "")[-600:]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--workers", type=int, default=10)
    ap.add_argument("--only", default=None)
    ap.add_argument("--skip-existing", action="store_true")
    ap.add_argument("--names-file", default=None, help="only these raw file names (one per line)")
    args = ap.parse_args()
    root = Path(args.root)
    jobs = [j for j in plan(root) if not args.only or j["inv"] == args.only]
    if args.names_file:
        keep = set(Path(args.names_file).read_text(encoding="utf-8").splitlines())
        jobs = [j for j in jobs if j["file"]["file_name"] in keep]
    if args.skip_existing:
        jobs = [j for j in jobs if not j["out"].exists()]
    jobs.sort(key=lambda j: -j["file"]["size_bytes"])
    print(f"{len(jobs)} jobs", flush=True)
    ok = fail = 0
    errs = []
    with ThreadPoolExecutor(args.workers) as ex:
        futs = {ex.submit(run, j, root): j for j in jobs}
        for f in as_completed(futs):
            good, err = f.result()
            if good:
                ok += 1
            else:
                fail += 1
                errs.append({"file": futs[f]["file"]["file_name"], "error": err})
            if (ok + fail) % 10 == 0:
                print(f"{ok + fail}/{len(jobs)} done, {fail} failed", flush=True)
    (root / "logs").mkdir(exist_ok=True)
    (root / "logs" / f"run_all_errors_{args.only or 'all'}.json").write_text(json.dumps(errs, indent=1), encoding="utf-8")
    print(f"finished: {ok} ok, {fail} failed", flush=True)


if __name__ == "__main__":
    main()

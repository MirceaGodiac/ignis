#!/usr/bin/env python3
"""One scored run of the Saffire transfer test (docs/wsp/SAFFIRE_TRANSFER.md, section 8).

Without --execute it only checks what it can without reading a held track (git state, freeze, binary)
and says what it would do. With --execute, in this order:

  1. the registration, the reader, this script and the freeze are committed and unchanged;
  2. the scorer snapshot's sha256 is the frozen one;
  3. the committed TRAIN aggregate of the matched-window comparison is reproduced for its three
     releases to 1e-12 (otherwise nothing is scored on Saffire);
  4. the held tracks are the registered run ids, partition `validation`, group status `held`, with
     the sha256 of the measurer's receipt; nothing of their content is printed;
  5. the wide and strip folders are scored once each with --quiet-cases and no --detail into
     docs/wsp/feedback/saffire-transfer/results/, which must not exist;
  6. aggregates are printed and the sentence of section 7 is applied mechanically.

It prints and writes aggregates only. It never runs twice: the results folder is its lock.
"""
import argparse
import hashlib
import json
import math
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PROTOCOL = "docs/wsp/SAFFIRE_TRANSFER.md"
FREEZE = "docs/wsp/feedback/saffire-transfer/freeze.json"
COMMITTED = [
    PROTOCOL,
    FREEZE,
    "tools/wsp/saffire/score_transfer.py",
    "crates/ignis-wsp/src/front_tools.rs",
    "crates/ignis-wsp-dev/src/front.rs",
]
RESULTS = ROOT / "docs/wsp/feedback/saffire-transfer/results"
HELD = ROOT / "runs/dev/saffire/tracks_held"
TRAIN = ROOT / "tools/wsp/bass_optical/tracks/train"
CHAIN_REF = ROOT / "docs/wsp/feedback/astra-bass-operator-20261115/results/aggregate.json"

PERSISTENCE = "sha256:ce53fe1fb3e6300c5e818dd7c9c5ec8e8f4c2cfd19673449d3936de05527de46"
D1 = "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c"
JOINT = "sha256:79d27b0203a7935f1bdaa275e2f508f24f0d2a52edd67ebe38fd327647c5a1e0"
WSP4 = "sha256:01a4edf415434f4f041a91a7b44cde9999ea9afa8f921c28f6200a7d3837c87a"
RELEASES = [PERSISTENCE, D1, JOINT, WSP4]
NAMES = {PERSISTENCE: "persistence", D1: "D1 (adopted)", JOINT: "joint Astra", WSP4: "wsp4 operator"}
STRATA = {
    "wide": {"PSI-98:S1", "PSI-98:S2", "PSI-100:S1", "PSI-100:S2"},
    "strip": {"PSI-99:2-5", "PSI-99:2-6"},
}
GROUPS = {"PSI-98": "PSI-98:flight", "PSI-99": "PSI-99:flight", "PSI-100": "PSI-100:flight"}


def die(msg: str) -> None:
    print(f"REFUSED: {msg}", file=sys.stderr)
    sys.exit(2)


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def check_git() -> str:
    for p in COMMITTED:
        if not (ROOT / p).exists():
            die(f"{p} does not exist")
        if git("ls-files", "--", p).strip() != p:
            die(f"{p} is not committed")
        if git("status", "--porcelain", "--", p).strip():
            die(f"{p} differs from the commit")
    return git("rev-parse", "HEAD").strip()


def run_scorer(binary: Path, tracks: Path, label: str, out: Path) -> str:
    cmd = [str(binary), "front-feedback"]
    for r in RELEASES if label != "chain" else [D1, JOINT, WSP4]:
        cmd += ["--release", r]
    cmd += ["--tracks", str(tracks), "--label", label, "--out", str(out), "--quiet-cases"]
    p = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if p.returncode != 0:
        die(f"scorer exit {p.returncode} on {tracks.name}: {p.stderr.strip()[-400:]}")
    return p.stdout


def close(a, b, tol=1e-12) -> bool:
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(close(a[k], b[k], tol) for k in a)
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(close(x, y, tol) for x, y in zip(a, b))
    if isinstance(a, (int, float)) and isinstance(b, (int, float)) and not isinstance(a, bool):
        return math.isclose(a, b, rel_tol=0, abs_tol=tol) or (math.isnan(a) and math.isnan(b))
    return a == b


def chain_check(binary: Path) -> None:
    ref = json.loads(CHAIN_REF.read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "chain.json"
        run_scorer(binary, TRAIN, "chain", out)
        got = json.loads(out.read_text(encoding="utf-8"))
    for key in ("counts",):
        if not close(got[key], ref[key]):
            die(f"chain check: {key} differs from the committed TRAIN aggregate")
    gr = {r["release_id"]: r["aggregate"] for r in got["releases"]}
    for r in ref["releases"]:
        if not close(gr.get(r["release_id"]), r["aggregate"]):
            die(f"chain check: aggregate of {r['release_id'][:15]} differs from the committed one")
    gp = {p["release_id"]: p["scores"] for p in got["paired_vs_first"]}
    for p in ref["paired_vs_first"]:
        if not close(gp.get(p["release_id"]), p["scores"]):
            die(f"chain check: paired scores of {p['release_id'][:15]} differ")
    print("chain check: the committed TRAIN aggregate is reproduced for all three releases (1e-12)")


def check_tracks() -> dict:
    receipt_path = HELD / "RECEIPT.json"
    if not receipt_path.exists():
        die("no measurement receipt")
    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    hashes = receipt.get("files_sha256", {})
    seen = {}
    for stratum, expected in STRATA.items():
        folder = HELD / stratum
        files = sorted(folder.glob("T*.json"))
        ids = {}
        for f in files:
            rel = f"{stratum}/{f.name}"
            if hashes.get(rel) != sha256(f):
                die(f"{rel}: sha256 differs from the receipt")
            v = json.loads(f.read_text(encoding="utf-8"))
            rid = v.get("run_id")
            inv = str(rid).split(":")[0]
            if v.get("schema") != "wsp-bass-front-track/1" or v.get("partition") != "validation":
                die(f"{rel}: schema or partition is not the registered one")
            if v.get("group_status") != "held" or v.get("evaluation_group") != GROUPS.get(inv):
                die(f"{rel}: evaluation group is not the registered one")
            ids[rid] = rel
        if set(ids) != expected:
            die(f"{stratum}: run ids are not the registered ones (counts {len(ids)} against {len(expected)})")
        seen[stratum] = {"tracks": len(files), "files_sha256": {ids[k]: hashes[ids[k]] for k in sorted(ids)}}
    return seen


def table(rep: dict) -> list:
    rows = []
    paired = {p["release_id"]: p["scores"] for p in rep.get("paired_vs_first", [])}
    for r in rep["releases"]:
        a = r["aggregate"]
        o = a["overall"]
        pc = paired.get(r["release_id"], {}).get("crps")
        ev = a.get("events", {})
        rows.append({
            "release": NAMES[r["release_id"]],
            "release_id": r["release_id"],
            "runs": o.get("runs"),
            "cases": o.get("cases"),
            "crps_mean": o["crps"]["mean_over_runs"] if o.get("crps") else None,
            "ae_median_mean": o["ae_median"]["mean_over_runs"] if o.get("ae_median") else None,
            "coverage_50": o["coverage_50"]["mean_over_runs"] if o.get("coverage_50") else None,
            "coverage_90": o["coverage_90"]["mean_over_runs"] if o.get("coverage_90") else None,
            "cases_refused": a.get("cases_refused"),
            "events": ev,
            "ignition_outcome": a.get("ignition_outcome"),
            "vs_persistence": pc,
        })
    return rows


def verdict(rows: list, release: str) -> dict:
    row = next(r for r in rows if r["release_id"] == release)
    pc = row["vs_persistence"] or {}
    n = pc.get("runs") or 0
    lower = pc.get("runs_where_release_lower") or 0
    higher = pc.get("runs_where_first_lower") or 0
    if n < 2:
        word = "not decided"
    elif lower == n:
        word = "transfers"
    elif higher == n:
        word = "does not transfer"
    else:
        word = "undecided"
    return {"release": row["release"], "burns_scored": n, "lower_than_persistence": lower,
            "higher_than_persistence": higher, "cases": row["cases"], "coverage_90": row["coverage_90"],
            "outcome": word}


def fmt(x):
    return "-" if x is None else (f"{x:.4f}" if isinstance(x, float) else str(x))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--execute", action="store_true")
    args = ap.parse_args()
    head = check_git()
    freeze = json.loads((ROOT / FREEZE).read_text(encoding="utf-8"))
    binary = Path(freeze["scorer_snapshot"]["path"])
    if not binary.is_absolute():
        binary = ROOT / binary
    if sha256(binary) != freeze["scorer_snapshot"]["sha256"]:
        die("scorer snapshot sha256 differs from the freeze")
    if freeze.get("release_ids") != RELEASES:
        die("release ids of the freeze differ from this script")
    if RESULTS.exists():
        die(f"{RESULTS.relative_to(ROOT)} exists: the test has been scored; it is not scored twice")
    if not args.execute:
        print(f"checks passed at {head[:12]}; nothing scored (pass --execute for the one run)")
        return 0
    started = datetime.now(timezone.utc).isoformat()
    chain_check(binary)
    tracks = check_tracks()
    RESULTS.mkdir(parents=True)
    summary = {"schema": "ignis.wsp.saffire-transfer-result/1", "protocol": PROTOCOL, "protocol_commit": head,
               "started_at": started, "scorer_sha256": freeze["scorer_snapshot"]["sha256"],
               "release_ids": RELEASES, "tracks": tracks, "strata": {}}
    for stratum in STRATA:
        out = RESULTS / f"{stratum}.json"
        stdout = run_scorer(binary, HELD / stratum, f"saffire-transfer-{stratum}", out)
        (RESULTS / f"{stratum}.log").write_text(stdout, encoding="utf-8")
        rep = json.loads(out.read_text(encoding="utf-8"))
        rows = table(rep)
        summary["strata"][stratum] = {
            "counts": rep.get("counts"),
            "rows": rows,
            "verdicts": [verdict(rows, r) for r in (D1, JOINT, WSP4)],
        }
    summary["finished_at"] = datetime.now(timezone.utc).isoformat()
    (RESULTS / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    for stratum, s in summary["strata"].items():
        c = s["counts"] or {}
        print(f"\n== {stratum}: tracks {c.get('tracks')}, scoreable {c.get('tracks_scoreable')}, "
              f"runs with cases {c.get('runs_with_cases')}, cases {c.get('cases')}, "
              f"unscoreable {c.get('tracks_unscoreable_by_reason')}")
        print(f"{'release':16} {'runs':>4} {'cases':>5} {'CRPS':>8} {'AE med':>8} {'cov50':>6} {'cov90':>6} "
              f"{'d CRPS':>8} {'lower/n':>7}")
        for r in s["rows"]:
            pc = r["vs_persistence"] or {}
            ln = f"{pc.get('runs_where_release_lower')}/{pc.get('runs')}" if pc else "ref"
            print(f"{r['release']:16} {fmt(r['runs']):>4} {fmt(r['cases']):>5} {fmt(r['crps_mean']):>8} "
                  f"{fmt(r['ae_median_mean']):>8} {fmt(r['coverage_50']):>6} {fmt(r['coverage_90']):>6} "
                  f"{fmt(pc.get('mean_difference_over_runs')):>8} {ln:>7}")
        for v in s["verdicts"]:
            print(f"  {v['release']}: {v['outcome']} ({v['lower_than_persistence']} of {v['burns_scored']} "
                  f"burns lower than persistence, {v['cases']} cases)")
    d1 = summary["strata"]["wide"]["verdicts"][0]
    print(f"\nDECISION (section 7, D1, wide sheet): {d1['outcome']}, on {d1['burns_scored']} burns")
    return 0


if __name__ == "__main__":
    sys.exit(main())

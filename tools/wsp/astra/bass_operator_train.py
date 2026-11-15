"""Execute the committed frozen-weight BASS operator comparison, on TRAIN only."""
import argparse
from collections import defaultdict
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import statistics
import subprocess

from binary_snapshot import load_snapshot

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "docs/wsp/feedback/astra-bass-operator-20261115"
PROTOCOL = ROOT / "docs/wsp/dossier/ASTRA_BASS_OPERATOR_20261115.md"
TRACKS = ROOT / "tools/wsp/bass_optical/tracks/train"
METRICS = ["crps", "ae_median", "coverage_50", "coverage_90", "ended_brier"]


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write("\n")


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def verify(frozen):
    for name, expected in frozen["file_sha256"].items():
        require(digest(ROOT / name) == expected, "Frozen file changed: " + name)
    # The scorer enumerates this folder; refuse even an extra track rather than expand the cohort.
    paths = sorted(p for p in TRACKS.iterdir() if p.is_file()
                   and p.name.startswith("T") and p.suffix.lower() == ".json")
    require([p.name for p in paths] == frozen["track_names"], "TRAIN folder membership changed")
    days = {}
    for path in paths:
        value = read(path)
        require(value["partition"] == "train", "Non-TRAIN track refused")
        require(value["run_id"] not in days, "Duplicate run id")
        days[value["run_id"]] = value["conditions"]["gmt_day"]
    require(days == frozen["days"], "Grouping changed")


def rows_of(detail, rid):
    rows = {}
    for row in detail["rows"]:
        if row["release_id"] != rid or "cutoff_s" not in row or "refusal" in row:
            continue
        key = (row["run_id"], row["cutoff_s"])
        require(key not in rows, "Duplicate paired case")
        metrics = {m: row[m] for m in METRICS[:-1]}
        ev = row.get("events", [])
        metrics["ended_brier"] = statistics.mean((e["p_ended"] - e["observed"])**2 for e in ev) if ev else None
        rows[key] = metrics
    return rows


def compare(candidate, baseline, days):
    result = {}
    for metric in METRICS:
        keys = sorted(k for k in candidate.keys() & baseline.keys()
                      if all(isinstance(r[k][metric], (int, float)) and math.isfinite(r[k][metric])
                             for r in (candidate, baseline)))
        by_run = defaultdict(list)
        for key in keys:
            by_run[key[0]].append(key)
        by_day = defaultdict(list)
        runs = {}
        for run, ks in by_run.items():
            a = statistics.mean(candidate[k][metric] for k in ks)
            b = statistics.mean(baseline[k][metric] for k in ks)
            runs[run] = {"candidate": a, "comparator": b, "difference": a - b, "cases": len(ks)}
            by_day[days[run]].append((a, b))
        day_means = [(statistics.mean(x[0] for x in rs), statistics.mean(x[1] for x in rs))
                     for rs in by_day.values()]
        result[metric] = {
            "candidate": statistics.mean(x[0] for x in day_means) if keys else None,
            "comparator": statistics.mean(x[1] for x in day_means) if keys else None,
            "difference": statistics.mean(x[0]-x[1] for x in day_means) if keys else None,
            "common_cases": len(keys), "runs": len(by_run), "independent_days": len(by_day),
            "ci95": None, "ci_reason": "one independent TRAIN day; no independent-day interval",
            "by_run": runs,
            "by_case": [{"run_id": k[0], "cutoff_s": k[1], "candidate": candidate[k][metric],
                         "comparator": baseline[k][metric], "difference": candidate[k][metric]-baseline[k][metric]} for k in keys],
        }
    lost = sorted(baseline.keys() - candidate.keys())
    gained = sorted(candidate.keys() - baseline.keys())
    primary = result["crps"]["difference"]
    guard = result["ended_brier"]["difference"]
    if primary is None:
        verdict = "not comparable"
    elif lost:
        verdict = "coverage loss; no unqualified win"
    elif primary > 1e-12:
        verdict = "worse TRAIN CRPS"
    elif primary >= -1e-12:
        verdict = "level TRAIN CRPS"
    elif guard is not None and guard > 1e-12:
        verdict = "lower TRAIN CRPS; track-ended Brier trade-off"
    else:
        verdict = "lower TRAIN CRPS; generalization unestablished"
    return {"metrics": result, "lost_cases": lost, "gained_cases": gained, "verdict": verdict}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    require(args.execute, "Scoring requires --execute")
    frozen = read(BASE / "freeze.json")
    required = frozen["committed_paths"] + [str((BASE / "freeze.json").relative_to(ROOT)).replace("\\", "/")]
    for name in required:
        subprocess.run(["git", "ls-files", "--error-unmatch", name], cwd=ROOT, check=True, capture_output=True)
    require(not subprocess.check_output(["git", "status", "--porcelain", "--", *required], cwd=ROOT, text=True).strip(),
            "Commit protocol, candidate and code before scoring")
    commit = subprocess.check_output(["git", "log", "-1", "--format=%H", "--", *required], cwd=ROOT, text=True).strip()
    verify(frozen)
    snap = load_snapshot(Path(frozen["scorer_snapshot"]["manifest_path"]).parent)
    require(snap.receipt() == frozen["scorer_snapshot"], "Scorer snapshot changed")
    snap.verify()
    output = ROOT / frozen["output_dir"]
    output.mkdir(exist_ok=False)
    # Preserve hashes of exact scorer bytes when Git commits Windows output.
    (output / ".gitattributes").write_text("*.json -text whitespace=cr-at-eol\n*.log -text whitespace=cr-at-eol\n", encoding="utf-8", newline="\n")
    command = ["front-feedback"]
    for rid in frozen["release_ids"]:
        command += ["--release", rid]
    command += ["--tracks", str(TRACKS), "--label", "frozen-astra-bass-operator-train-20261115",
                "--out", str(output / "aggregate.json"), "--detail", str(output / "cases.json")]
    env = os.environ.copy()
    env["IGNIS_ARTIFACTS"] = str(ROOT / "artifacts")
    started = datetime.now(timezone.utc).isoformat()
    response = snap.run(command, cwd=ROOT, env=env, capture_output=True, text=True)
    (output / "execution.log").write_text(response.stdout + response.stderr, encoding="utf-8", newline="\n")
    write(output / "execution.json", {"protocol_commit": commit, "started_at": started,
          "finished_at": datetime.now(timezone.utc).isoformat(), "command": command,
          "binary": snap.receipt(), "exit_code": response.returncode})
    require(response.returncode == 0, "Scorer failed; retain execution receipt, do not change weights")
    verify(frozen)
    snap.verify()
    report = read(output / "aggregate.json")
    detail = read(output / "cases.json")
    require(set(report["counts"]["partitions"]) == {"train"}, "Unexpected partition")
    require(report["release_ids"] == frozen["release_ids"], "Unexpected release order")
    require(report["counts"] == frozen["expected_counts"], "Eligibility changed")
    historical = read(ROOT / frozen["historical_cases"])
    historical_report = read(ROOT / frozen["historical_report"])
    checks = []
    for rid in frozen["release_ids"][:2]:
        old = [r for r in historical["rows"] if r["release_id"] == rid]
        now = [r for r in detail["rows"] if r["release_id"] == rid]
        require(old == now, "Historical case records did not reproduce: " + rid)
        before = next(r for r in historical_report["releases"] if r["release_id"] == rid)
        after = next(r for r in report["releases"] if r["release_id"] == rid)
        require(before == after, "Historical aggregate did not reproduce: " + rid)
        checks.append({"release_id": rid, "exact_case_records": len(now), "aggregate_exact": True})
    ignition = []
    for rid in frozen["release_ids"][1:]:
        ignition.append([{k: v for k, v in r.items() if k != "release_id"}
                         for r in detail["rows"] if r["release_id"] == rid and "ignition_outcome" in r])
    require(ignition[0] == ignition[1], "No-prefix ignition changed")
    rows = [rows_of(detail, rid) for rid in frozen["release_ids"]]
    comparisons = []
    for index, label in enumerate(["Fable", "joint Astra"]):
        comparison = compare(rows[2], rows[index], frozen["days"])
        comparison.update({"comparator": label, "candidate_release_id": frozen["release_ids"][2],
                           "comparator_release_id": frozen["release_ids"][index]})
        comparisons.append(comparison)
    summary = {"status": "data-informed; experimental; not trained; no automatic adoption",
               "partition": "train", "protocol_commit": commit, "counts": report["counts"],
               "aggregation": "case means, equal run means, equal independent-day means",
               "comparisons": comparisons,
               "refusals": {rid: sum(r["release_id"] == rid and "refusal" in r for r in detail["rows"]) for rid in frozen["release_ids"]},
               "verification": checks,
               "joint_candidate_ignition_exact": True,
               "limitations": ["One independent day; scorer's run bootstrap is not independent-day evidence.",
                               "Track-ended includes leaving view, not solely physical quenching.",
                               "Forecasts remain conditional on supplied future flow history.",
                               "This corrects an observation approximation, not the physical quench law."]}
    write(output / "summary.json", summary)
    write(output / "receipt.json", {"protocol_commit": commit, "frozen_files_verified_before_and_after": True,
          "files_sha256": {p.name: digest(p) for p in sorted(output.iterdir()) if p.is_file()}})
    for c in comparisons:
        print(c["comparator"], c["verdict"], c["metrics"]["crps"], flush=True)


if __name__ == "__main__":
    main()

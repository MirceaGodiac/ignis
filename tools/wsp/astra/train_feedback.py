"""Run the preselected Astra/Fable comparisons on TRAIN only; never changes a model.

All five comparisons were fixed before scores. The shared data root is explicit; private
extended data and sealed evaluation workflows are not used. Existing feedback is immutable.
"""
from concurrent.futures import ThreadPoolExecutor, as_completed
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path

from candidates import ROOT, CASES, read, write_same
from binary_snapshot import add_snapshot_arguments, select_snapshot

DEV = ROOT / "runs/tooling/astra-target/release/wsp-dev.exe"
OUT = ROOT / "runs/dev/astra"


def resolve_receipt_directory(directory):
    resolved = Path(directory).resolve()
    if not resolved.is_relative_to((ROOT / "runs/dev/astra").resolve()):
        raise ValueError("receipt directory must remain inside runs/dev/astra")
    return resolved


def run_case(case, execution_binary):
    key, old, new = case
    paths = [ROOT / "configs/models" / old / "release.json"] + [
        ROOT / "configs/models" / new / f"{label}-astra/release.json"
        for label in ("r1", "joint")]
    ids = [read(path)["id"] for path in paths]
    out = OUT / f"train_{key}.json"
    receipt_path = OUT / f"train_{key}_execution.json"
    if out.exists() or receipt_path.exists():
        raise RuntimeError(f"refuse to overwrite existing feedback: {out}")
    command = [str(execution_binary.path), "front-feedback" if key == "bass_front3" else "feedback"]
    for rid in ids:
        command += ["--release", rid]
    if key == "bass_front3":
        detail = OUT / "train_bass_front3_cases.json"
        if detail.exists():
            raise RuntimeError(f"refuse to overwrite existing detail: {detail}")
        command += ["--tracks", str(ROOT / "tools/wsp/bass_optical/tracks/train"),
                    "--label", "astra-data-informed-train-only", "--detail", str(detail)]
    else:
        command += ["--partition", "train", "--cutoffs", "1,3,5", "--horizon-s", "10"]
        if key.startswith("acme"):
            command += ["--times", "6,8,10,12,15,20,25,30,40,50,60,80,100,120"]
    command += ["--out", str(out)]
    env = os.environ.copy()
    env["IGNIS_DATA"] = str(ROOT / "data")
    env["IGNIS_ARTIFACTS"] = str(ROOT / "artifacts")
    start = datetime.now(timezone.utc).isoformat()
    result = execution_binary.run(command[1:], cwd=ROOT, env=env, capture_output=True, text=True)
    receipt = {"model": key, "candidate_status": "data-informed; not zero-shot",
               "started_at": start, "finished_at": datetime.now(timezone.utc).isoformat(),
               "command": command, "data_root": env["IGNIS_DATA"],
               "release_ids": ids, "exit_code": result.returncode,
               "execution_binary": execution_binary.receipt()}
    write_same(OUT / f"train_{key}.log", result.stdout + result.stderr)
    if result.returncode:
        execution_binary.verify()
        write_same(receipt_path, receipt)
        raise RuntimeError(f"{key} feedback failed ({result.returncode}): {result.stderr}")
    report = read(out)
    if key == "bass_front3":
        if set(report["counts"]["partitions"]) != {"train"}:
            raise RuntimeError("front feedback did not contain train tracks exclusively")
    elif report["partition"] != "train":
        raise RuntimeError("feedback partition escaped train")
    execution_binary.verify()
    write_same(receipt_path, receipt)
    return receipt


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt-directory", default=str(OUT))
    add_snapshot_arguments(parser)
    args = parser.parse_args()
    try:
        OUT = resolve_receipt_directory(args.receipt_directory)
    except ValueError as error:
        raise SystemExit(str(error)) from None
    execution_binary = select_snapshot(args, DEV)
    print(json.dumps({"execution_binary": execution_binary.receipt()}), flush=True)
    OUT.mkdir(parents=True, exist_ok=True)
    failures = []
    with ThreadPoolExecutor(max_workers=len(CASES)) as pool:
        futures = {pool.submit(run_case, case, execution_binary): case[0] for case in CASES}
        for future in as_completed(futures):
            try:
                receipt = future.result()
                print(json.dumps({name: receipt[name] for name in
                                  ("model", "candidate_status", "exit_code", "finished_at")}), flush=True)
            except Exception as exc:
                failures.append(str(exc))
                print(json.dumps({"model": futures[future], "failure": str(exc)}), flush=True)
    if failures:
        raise SystemExit("; ".join(failures))
    execution_binary.verify()

"""Summarize fixed train receipts by release ID; no fitting or model mutation.

Require identical aggregates before/after the diagnostic-export correction. Case and
session comparisons come from final scorer contributions, not reimplemented event scores.
"""
from collections import defaultdict
import hashlib
import json
import math
from statistics import fmean, median

from candidates import ROOT, CASES, read, write_same

RAW = ROOT / "runs/dev/astra"
REPLAY = RAW / "session_receipts"
LABELS = ("Fable", "R1", "joint")


def receipt(path):
    return {"path": str(path.relative_to(ROOT)).replace("\\", "/"),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def valid_cases(release, metric):
    result = {}
    for case in release["cases"]:
        value = case["metrics"].get(metric)
        if not case["scored"] or value is None or not math.isfinite(value):
            continue
        key = (case["run_id"], case["cutoff_s"])
        assert key not in result
        result[key] = case
    return result


def compare(releases, metric, scale):
    maps = [valid_cases(r, metric) for r in releases]
    common = set.intersection(*(set(m) for m in maps))
    result = []
    for index, label in enumerate(LABELS[1:], 1):
        sessions = defaultdict(list)
        rows = []
        for key in sorted(common):
            reference, candidate = maps[0][key], maps[index][key]
            assert reference["cluster"] == candidate["cluster"]
            a, b = reference["metrics"][metric] * scale, candidate["metrics"][metric] * scale
            row = {"run_id": key[0], "cutoff_s": key[1], "reference": a,
                   "candidate": b, "difference": b - a}
            rows.append(row)
            sessions[reference["cluster"]].append(row)
        by_session = [{"cluster": cluster, "cases": len(group),
                       "reference": fmean(r["reference"] for r in group),
                       "candidate": fmean(r["candidate"] for r in group),
                       "difference": fmean(r["difference"] for r in group)}
                      for cluster, group in sorted(sessions.items())]
        deltas = [r["difference"] for r in rows]
        item = {"label": label, "release_id": releases[index]["release_id"],
                "metric": metric, "scale": scale, "common_cases": len(rows),
                "case_mean_difference": fmean(deltas), "case_median_difference": median(deltas),
                "equal_session_mean_difference": fmean(r["difference"] for r in by_session),
                "by_session": by_session,
                "largest_increases": sorted(rows, key=lambda r: r["difference"], reverse=True)[:5],
                "largest_decreases": sorted(rows, key=lambda r: r["difference"])[:5]}
        if metric == "crps":
            # This is the old D-40 descriptive tolerance, not a new selection rule.
            item["sessions_no_worse_within_historical_1um_tolerance"] = sum(
                r["difference"] <= 1.0 for r in by_session)
            eligible = [r for r in rows if all(
                maps[0][(r["run_id"], r["cutoff_s"])]["conditions"].get(k) == 0.0
                for k in ("x_co2", "x_he"))]
            item["nitrogen_carrier_eligible_inputs"] = {
                "cases": len(eligible),
                "case_mean_difference": fmean(r["difference"] for r in eligible) if eligible else None,
                "interpretation": "Input eligibility only; not proof that a hot-radiative transition occurred or that the thermal path dominated."}
        result.append(item)
    return result


def summarize_generic(key, original, replay):
    assert original["partition"] == replay["partition"] == "train"
    rows = []
    for label, old, new in zip(LABELS, original["releases"], replay["releases"]):
        assert old["release_id"] == new["release_id"]
        assert old["aggregate"] == new["aggregate"], f"aggregate changed: {key}/{label}"
        assert old["refused"] == new["refused"]
        metrics = {}
        for name, value in new["aggregate"]["metrics"].items():
            scale = 1e6 if key.startswith("flex") and name in (
                "crps", "mae_mean", "mae_median", "width@0.5", "width@0.9") else 1.0
            metrics[name] = {"case_mean": value["case_mean"] * scale,
                             "equal_cluster_mean": value["estimate"] * scale,
                             "n_cases": value["n_cases"], "n_clusters": value["n_clusters"], "scale": scale}
            cases = valid_cases(new, name)
            assert len(cases) == value["n_cases"]
            assert math.isclose(fmean(c["metrics"][name] for c in cases.values()),
                                value["case_mean"], rel_tol=1e-12, abs_tol=1e-14)
            groups = defaultdict(list)
            for c in cases.values():
                groups[c["cluster"]].append(c["metrics"][name])
            assert len(groups) == value["n_clusters"]
            assert math.isclose(fmean(fmean(v) for v in groups.values()),
                                value["estimate"], rel_tol=1e-12, abs_tol=1e-14)
        rows.append({"label": label, "release_id": new["release_id"], "metrics": metrics,
                     "domain_refusals": new["refused"],
                     "admitted_accounting": new["aggregate"]["accounting"]})
    chosen = [("crps", 1e6)] if key.startswith("flex") else [
        ("ibs_contrib[extinction]", 1.0), ("ibs_contrib[operator_termination]", 1.0)]
    comparisons = [item for metric, scale in chosen for item in compare(replay["releases"], metric, scale)]
    return {"runs_in_train_split": replay["runs"], "not_at_risk_cases": replay["not_at_risk_cases"],
            "cutoffs_s": replay["cutoffs_s"], "scoring_spec": replay["spec"],
            "releases": rows, "paired_by_release_id": comparisons}


def summarize_front(original, replay):
    assert set(original["counts"]["partitions"]) == set(replay["counts"]["partitions"]) == {"train"}
    assert original == replay
    detail = read(REPLAY / "train_bass_front3_cases.json")
    rows = []
    for label, release in zip(LABELS, replay["releases"]):
        a = release["aggregate"]
        by_run = defaultdict(list)
        for row in detail["rows"]:
            if row["release_id"] == release["release_id"] and "crps" in row:
                by_run[row["run_id"]].append(row["crps"])
        means = {run: fmean(values) for run, values in sorted(by_run.items())}
        assert math.isclose(fmean(means.values()), a["overall"]["crps"]["mean_over_runs"], rel_tol=1e-12)
        rows.append({"label": label, "release_id": release["release_id"],
                     "cases_scored": a["cases_scored"], "cases_refused": a["cases_refused"],
                     "tip_crps_mean_over_runs": a["overall"]["crps"]["mean_over_runs"],
                     "tip_crps_by_run": means, "track_end_brier": a["events"]["brier_mean_over_runs"],
                     "event_pairs": a["events"]["pairs"],
                     "observed_track_end_pairs": a["events"]["pairs_observed_ended"],
                     "ignition_brier": a["ignition_outcome"]["brier_mean_over_tracks"]})
    return {"counts": replay["counts"], "protocol": replay["protocol"], "releases": rows,
            "paired_by_release_id": replay["paired_vs_first"],
            "independent_session_limit": "Three spreading tracks share one day; per-run comparisons are not three independent session transfers."}


def main():
    models = {}
    receipts = []
    executions = []
    for key, old, new in CASES:
        original_path = RAW / f"train_{key}.json"
        replay_path = REPLAY / f"train_{key}.json"
        original, replay = read(original_path), read(replay_path)
        expected = [read(ROOT / "configs/models" / old / "release.json")["id"]] + [
            read(ROOT / "configs/models" / new / f"{label}-astra/release.json")["id"]
            for label in ("r1", "joint")]
        assert [r["release_id"] for r in replay["releases"]] == expected
        models[key] = summarize_front(original, replay) if key == "bass_front3" else summarize_generic(key, original, replay)
        receipts.extend([receipt(original_path), receipt(replay_path)])
        executions.extend([read(RAW / f"train_{key}_execution.json"),
                           read(REPLAY / f"train_{key}_execution.json")])
    result = {"candidate_status": "All Astra candidates are data-informed, not zero-shot; no new validation look.",
              "purpose": "Train diagnostics, not a generalization or adoption claim",
              "data_root": "data (shared records only)",
              "initial_and_replay_aggregates_identical": True,
              "pairing": "Full release IDs and matched cases; generic paired_vs_first intentionally ignored (F-009).",
              "event_metric": "Existing scorer's IPCW integrated Brier contributions, exported after batch computation (F-010).",
              "receipts": receipts, "executions": executions, "models": models}
    write_same(ROOT / "docs/wsp/feedback/astra_train_summary.json", result)
    for key, model in models.items():
        for release in model["releases"]:
            if key == "bass_front3":
                scores = {k: release[k] for k in ("tip_crps_mean_over_runs", "track_end_brier")}
            else:
                scores = {k: v["case_mean"] for k, v in release["metrics"].items()
                          if k in ("crps", "ibs_contrib[extinction]", "ibs_contrib[operator_termination]")}
            print(json.dumps({"family": key, "candidate": release["label"], "scores": scores}))


if __name__ == "__main__":
    main()

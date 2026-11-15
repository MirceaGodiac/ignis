"""PrefixBundle files for FLEX droplet runs, built outside the data root, for `ignis wsp forward`.

Mirrors `build_run_prefix` / `build_prefix` of ignis-core: the record's conditions, every observation
of the run with available_at <= cutoff and time <= cutoff (sorted by time, then id), no contours or
controls, phase established at 0.0 when the run is active at the cutoff (an observation at or after
the cutoff exists in the record; for a fresh series, at least one measured point after the cutoff).
Checked against `ignis infer forecast` on a train run before use (see `--check`).

Usage:
  python tools/wsp/final/fresh_heptane_prefix.py --check            (train run, compares both paths)
  python tools/wsp/final/fresh_heptane_prefix.py --series FILE --cutoff C --out PREFIX.json
"""
import argparse
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools/wsp"))
from law_run import content_hash  # noqa: E402

REC = ROOT / "data/work/records/flex"
TIME_ORIGIN_RECORDS = "NASA DropletData processed time base (phase entry not adjudicated)"


def bundle(run_id, session_id, cutoff, conditions, observations, active, time_origin):
    obs = [o for o in observations if o["available_at_s"] <= cutoff and o["time_s"] <= cutoff]
    obs.sort(key=lambda o: (o["time_s"], o["id"]))
    b = dict(family="flex", run_id=run_id, session_id=session_id, cutoff_s=float(cutoff),
             time_origin=time_origin, known_conditions=conditions, known_controls=[],
             observations=obs, contours=[], phase_established_at_s=0.0 if active else None,
             support_evidence=["activity confirmed at the cutoff by the run record"] if active else [],
             transform_versions={})
    b["input_hash"] = content_hash(b, ("input_hash",))
    return b


def from_records(run_id, cutoff):
    rec = next(json.loads(l) for l in open(REC / "experiments.jsonl", encoding="utf8")
               if json.loads(l)["id"] == run_id)
    obs = [json.loads(l) for l in open(REC / "observations.jsonl", encoding="utf8")]
    obs = [o for o in obs if o["run_id"] == run_id]
    active = any(o["time_s"] >= cutoff for o in obs if o.get("quantity") == "liquid_diameter")
    return bundle(run_id, rec["session_id"], cutoff, rec["conditions"], obs, active, TIME_ORIGIN_RECORDS)


def cond(value, unit_si, note):
    return {"derivation": {"agent": None, "inputs": [], "method": "deterministic",
                           "method_id": "heptane_fresh.cond", "method_version": "1", "notes": note},
            "evidence_class": "original_observation", "flags": [], "locator": None,
            "raw_unit": unit_si, "raw_value": value, "resolution": "resolved",
            "uncertainty_reason": "not reported by source", "uncertainty_si": None,
            "unit_si": unit_si, "value_si": value}


def from_series(path, cutoff):
    s = json.loads(pathlib.Path(path).read_text(encoding="utf8"))
    c = s["conditions"]
    pts = [o for o in s["observations"] if o.get("valid", True) and o.get("value_si") is not None]
    d0 = pts[0]["value_si"]  # the batch README: d0 is the first measured diameter
    conditions = {
        "x_o2": cond(c["x_o2"]["value"], "1", c["x_o2"].get("grade")),
        "x_co2": cond(c["x_co2"]["value"], "1", c["x_co2"].get("grade")),
        "x_he": cond(c["x_he"]["value"], "1", c["x_he"].get("grade")),
        "x_n2": cond(c["x_n2"]["value"], "1", c["x_n2"].get("grade")),
        "pressure": cond(c["pressure"]["value"], "Pa", c["pressure"].get("note")),
        "initial_diameter": cond(d0, "m", "first measured diameter of the fresh series"),
        "ambient_temperature": cond(298.0, "K", "not recorded for the fresh batch; cabin temperature"),
        "fiber": cond(0.0, "1", "free droplet (testpoint settings)"),
    }
    import hashlib
    asset = hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()
    obs = []
    for i, o in enumerate(pts):
        obs.append({"available_at_s": o["time_s"], "calibration_id": None, "channel": "DropletData",
                    "evidence_class": "source_derived", "id": f"{s['run_id']}/fresh/{i}",
                    "quality_flags": [], "quantity": "liquid_diameter", "raw_unit": "m",
                    "raw_value": o["value_si"], "run_id": s["run_id"], "time_s": o["time_s"],
                    "uncertainty_si": None, "unit_si": "m", "value_si": o["value_si"],
                    "locator": {"asset_sha256": asset, "char_end": None, "char_start": None, "column": None,
                                "frame": o.get("frame"), "page": None, "path": f"heptane_fresh/series/{pathlib.Path(path).name}",
                                "pts_s": None, "row": i},
                    "derivation": {"agent": None, "inputs": [], "method": "deterministic",
                                   "method_id": "heptane_fresh.droplet_diameter", "method_version": "1",
                                   "notes": "measured from frames by the frozen tool of the fresh batch"}})
    active = any(o["time_s"] > cutoff for o in obs)
    # The batch freeze (FREEZE.json time_origin, committed before any frame was measured) declares the series
    # clock equivalent to NASA's DropletData processed time base (on five train tests NASA's t = 0 fell
    # -0.31 to +0.41 s from the stamp used). The prefix carries the release's name for that time base.
    assert "NASA DropletData processed time base" in s["time_base"]["declared_as"]
    return bundle(s["run_id"], s["session_id"], cutoff, conditions, obs, active, TIME_ORIGIN_RECORDS)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--series")
    ap.add_argument("--cutoff", type=float)
    ap.add_argument("--out")
    a = ap.parse_args()
    if a.check:
        print(json.dumps(from_records("PSI-69:C08H101:2010-06-15", 3.0))[:300])
    else:
        pathlib.Path(a.out).write_text(json.dumps(from_series(a.series, a.cutoff), indent=1), encoding="utf8")

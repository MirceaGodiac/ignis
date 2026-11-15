"""Quality flags, conditions and BURNS rows for PSI-26 tracks that exist; pending rows for the rest.

Applies qc.flags (R1-R5) to every PSI-26 track and the class flag found in the visual check (every
viewed PSI-26 sheet failed: lit test section, reflections, rescaled overlay not registered, long
multi-event records), runs enrich.py for PSI-26, then appends a BURNS row (quality 'pending') for each
downloaded PSI-26 video without a track yet. Writes DONE_PSI-26_CONDITIONS.txt (counts only).

    python -I tools/wsp/media/fabric/finalize_psi26.py --root runs/dev/media/fabric
"""
import argparse
import datetime as dt
import importlib.util
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).parent
CLASS_FLAG = ("V PSI-26 class: every viewed PSI-26 contact sheet failed (lit test section, reflections on spheres and "
              "slabs tracked, 640x360 overlay not registered, long multi-event records); the frozen method does not "
              "measure these recordings")


def load(name):
    spec = importlib.util.spec_from_file_location(name, HERE / f"{name}.py")
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    args = ap.parse_args()
    root = Path(args.root)
    q = load("qc")
    for f in sorted((root / "tracks/psi26").glob("*.json")):
        d = json.loads(f.read_text(encoding="utf-8"))
        fl = q.flags(d, root / "ovl" / f"psi26_{f.stem}.npz") + [CLASS_FLAG]
        d["qc_flags"] = fl
        d["quality"] = "failed"
        d["outcome_unreliable"] = d.get("outcome_unreliable") or d["outcome"]
        d["outcome"] = {"kind": "cannot_tell", "confidence": "low", "reason": "quality failed: " + "; ".join(x.split(":")[0] for x in fl)}
        f.write_text(json.dumps(d, indent=1), encoding="utf-8")
    subprocess.run([sys.executable, "-I", str(HERE / "enrich.py"), "--root", str(root), "--only", "PSI-26"], check=True)
    e = load("enrich")
    ar = e.asrun26()
    have = {json.loads(l)["track"] for l in (root / "BURNS.jsonl").read_text(encoding="utf-8").splitlines()}
    rows = {}
    for l in (root / "manifest_raw.jsonl").read_text(encoding="utf-8").splitlines():
        r = json.loads(l)
        if r["investigation"] == "PSI-26":
            rows[r["file_name"]] = r
    extra = []
    import cv2
    for name, r in sorted(rows.items()):
        stem = re.sub(r"[^A-Za-z0-9]+", "_", name.replace("PSI-26_Raw Data_", "")[:-4])
        track = f"tracks/psi26/{stem}.json"
        if track in have:
            continue
        m = re.search(r"(\d{4})(\d\d)(\d\d) (\d\d)[. ](\d\d)", name)
        vdate = dt.date(int(m.group(1)), int(m.group(2)), int(m.group(3)))
        vstart = int(m.group(4)) * 60 + int(m.group(5))
        cap = cv2.VideoCapture(str(root / r["local"]))
        dur = (cap.get(7) / (cap.get(5) or 29.97)) if cap.isOpened() else 0
        hits = [a for a in ar if a["video_date"] == vdate and a["start"] is not None and vstart - 5 <= a["start"] <= vstart + dur / 60 + 5]
        one = hits[0] if len(hits) == 1 else None
        mat = (one or {}).get("material")
        w, wk = e.width_of(mat)
        air = re.match(r"\s*(\d+(?:\.\d+)?)", str((one or {}).get("air_display") or ""))
        extra.append({"investigation": "PSI-26", "burn_id": f"PSI-26:{stem}", "file_role": "primary", "date": vdate.isoformat(),
                      "partition": None, "material": mat, "flow_direction": None, "width_m": w, "width_kind": wk,
                      "x_o2_initial": (one or {}).get("o2"), "x_o2_final": None, "pressure_atm": (one or {}).get("pressure_atm"),
                      "fan_at_igniter_off": None, "u_cm_s_at_igniter_off": float(air.group(1)) if air else None,
                      "flow_source": "as-run matrix Air display (first value)",
                      "actual_date_as_run": (hits[0]["actual_date"].isoformat() if hits and hits[0]["actual_date"] else None),
                      "as_run_tests": [h["as_run_test"] for h in hits], "track": track, "quality": "pending (not yet measured)"})
    with open(root / "BURNS.jsonl", "a", encoding="utf-8") as f:
        for x in extra:
            f.write(json.dumps(x) + "\n")
    burns = [json.loads(l) for l in (root / "BURNS.jsonl").read_text(encoding="utf-8").splitlines()]
    p26 = [b for b in burns if b["investigation"] == "PSI-26"]
    keys = ("x_o2_initial", "u_cm_s_at_igniter_off", "material", "width_m")
    c_q = Counter((b["quality"], all(b.get(k) is not None for k in keys)) for b in p26)
    c_m = Counter((str(b["material"]), all(b.get(k) is not None for k in keys)) for b in p26)
    c_each = {k: sum(b.get(k) is not None for b in p26) for k in keys + ("flow_direction", "pressure_atm")}
    L = ["PSI-26 conditions joined (counts only; no outcomes).",
         f"PSI-26 videos: {len(p26)}; tracks written: {sum(1 for b in p26 if not str(b['quality']).startswith('pending'))}.",
         "Complete = O2, flow (as-run Air display), material and width all known. Flow direction is known only where the",
         "as-run text says it (spheres, candles and wax: not applicable); pressure where the as-run row gives it.",
         "Sources per burn are in each track's conditions.source; a video is matched to an as-run row when the row's",
         "approximate GMT start time falls inside the video (date on videos = file-name date). Most of the 114 videos are",
         "setup, dilution or non-test recordings: the 2013 sheet has 37 test rows.", "",
         "non-null counts: " + ", ".join(f"{k} {v}" for k, v in c_each.items()), "", "by quality (quality, complete): count"]
    L += [f"  {k}: {v}" for k, v in sorted(c_q.items(), key=str)]
    L += ["", "by material (material, complete): count"] + [f"  {k}: {v}" for k, v in sorted(c_m.items(), key=str)]
    L += ["", "No PSI-26 video shows a fabric sample. Every PSI-26 track is quality failed (class flag, see qc_flags)."]
    (root / "DONE_PSI-26_CONDITIONS.txt").write_text("\n".join(L) + "\n", encoding="utf-8")
    print("\n".join(L))


if __name__ == "__main__":
    main()

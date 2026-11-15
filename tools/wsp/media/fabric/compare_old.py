#!/usr/bin/env python3
"""Agreement of the new duct-video tracks with the earlier tracks, statistics only.

Earlier tracks: train in git (tools/wsp/bass_optical/tracks/train/), validation in
runs/dev/bass_front/val_sealed/ (local). For validation burns NO per-burn value is printed or written:
only pooled statistics over the partition. Run after the frozen measurement.

    python -I tools/wsp/media/fabric/compare_old.py --root runs/dev/media/fabric
"""
import argparse
import json
from collections import Counter
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[4]
OLD = {"train": REPO / "tools/wsp/bass_optical/tracks/train", "held_validation": REPO / "runs/dev/bass_front/val_sealed"}


def match(new_pts, old_pts, key_new, key_old, tol=0.6):
    if not new_pts or not old_pts:
        return []
    nt = np.array([p["t_s"] for p in new_pts])
    out = []
    for p in old_pts:
        if p.get(key_old) is None or p["t_s"] is None or p["t_s"] < 0:
            continue
        j = int(np.argmin(np.abs(nt - p["t_s"])))
        if abs(nt[j] - p["t_s"]) <= tol and new_pts[j].get(key_new) is not None:
            out.append(new_pts[j][key_new] - p[key_old])
    return out


def stats(v):
    v = np.asarray(v, float)
    if len(v) == 0:
        return None
    return {"n": int(len(v)), "median": round(float(np.median(v)), 4), "median_abs": round(float(np.median(np.abs(v))), 4),
            "p90_abs": round(float(np.percentile(np.abs(v), 90)), 4)}


def old_lit(o):
    pts = [s for s in (o.get("stills") or []) if s.get("tip_w") is not None and s["t_s"] is not None and s["t_s"] > 2.0
           and s.get("state") in (None, "flame")]
    pts += [d for d in (o.get("duct") or []) if d.get("tip_fw") is not None and d["t_s"] is not None and d["t_s"] > 2.0]
    return len(pts) >= 3


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    args = ap.parse_args()
    root = Path(args.root)
    report = {}
    for (part, odir), subset in [(kv, sub) for kv in OLD.items() for sub in ("all", "quality_ok", "ok_2cm", "ok_1cm")]:
        tip_fw, base_fw, tip_w, base_w, ign_on, last_t = [], [], [], [], [], []
        lit_agree = Counter()
        end_agree = Counter()
        n_pairs = 0
        missing = 0
        for of in sorted(odir.glob("*.json")):
            o = json.loads(of.read_text(encoding="utf-8"))
            nf = root / "tracks/psi25" / part / of.name
            if not nf.exists():
                missing += 1
                continue
            n = json.loads(nf.read_text(encoding="utf-8"))
            if subset != "all" and n.get("quality") not in ("ok", "suspect_geometry"):
                continue
            w1 = "1 cm" in (n.get("material") or "")
            if (subset == "ok_2cm" and w1) or (subset == "ok_1cm" and not w1):
                continue
            n_pairs += 1
            duct = n.get("duct") or []
            tip_fw += match(duct, o.get("duct") or [], "tip_fw", "tip_fw")
            base_fw += match(duct, o.get("duct") or [], "base_fw", "base_fw")
            st = [s for s in (o.get("stills") or []) if s.get("state") in (None, "flame")]
            tip_w += match(duct, st, "tip_w", "tip_w")
            base_w += match(duct, st, "base_w", "base_w")
            if n.get("igniter") and o.get("igniter") and o["igniter"].get("on_t_s") is not None:
                ign_on.append(n["igniter"]["on_t_s"] - o["igniter"]["on_t_s"])
            ol = old_lit(o)
            nl = n["outcome"]["kind"] not in ("never_lit",) and bool(duct)
            if n["outcome"]["kind"] == "cannot_tell" and not duct:
                lit_agree["new_cannot_tell_no_track"] += 1
            else:
                lit_agree["agree" if ol == nl else "disagree"] += 1
            oe = (o.get("end") or {})
            if "at_downstream_window_end" in oe and oe["at_downstream_window_end"] is not None and n["outcome"]["kind"] in ("spread_to_end", "went_out"):
                end_agree["agree" if bool(oe["at_downstream_window_end"]) == (n["outcome"]["kind"] == "spread_to_end") else "disagree"] += 1
            if oe.get("last_flame_t_s") is not None and (n.get("end") or {}).get("t_last_flame_s") is not None:
                last_t.append(n["end"]["t_last_flame_s"] - oe["last_flame_t_s"])
        report[f"{part} / {subset}"] = {
            "burns_with_old_and_new_track": n_pairs, "old_tracks_without_new_file": missing,
            "duct_tip_fw_new_minus_old": stats(tip_fw), "duct_base_fw_new_minus_old": stats(base_fw),
            "tip_w_new_duct_minus_old_still": stats(tip_w), "base_w_new_duct_minus_old_still": stats(base_w),
            "igniter_on_t_new_minus_old_s": stats(ign_on), "last_flame_t_new_minus_old_s": stats(last_t),
            "lit_or_not_agreement": dict(lit_agree), "reached_end_agreement": dict(end_agree),
        }
    out = root / "agreement_with_earlier_tracks.json"
    out.write_text(json.dumps(report, indent=1), encoding="utf-8")
    print(json.dumps(report, indent=1))


if __name__ == "__main__":
    main()

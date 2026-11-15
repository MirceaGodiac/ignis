#!/usr/bin/env python3
"""Convert the blind measurer's fabric tracks (fabric-duct/1) into the track format the fabric-flame
scorer reads (`wsp-bass-front-track/1` with positions in sample widths, the layout of
tools/wsp/bass_optical/tracks/train/*.json, read by crates/ignis-wsp/src/front_tools.rs and
front5_tools.rs).

What is measured is not changed: no point is smoothed, moved, selected or dropped. Each duct point
becomes one still at the same time; its tip/base become sample widths along the spread direction from
the measurer's own origin (position_origin_fw) with the measurer's own scale (width_scale_w_per_fw):

    w = (position_origin_fw - x_fw) * width_scale_w_per_fw   (spread toward smaller x)
    w = (x_fw - position_origin_fw) * width_scale_w_per_fw   (spread toward larger x)

The time origin is the measurer's (igniter off, t_s in video seconds after it), unchanged. The
measurer's `outcome` and `end` are read only to be carried across: verbatim as `measured_outcome` /
`measured_end`, and as the scorer's `end` by the fixed table END_KIND below.

A track rated `quality: failed`, or one lacking a field the conversion needs, is not converted; it is
listed with the reason category only (no measured value, no outcome).

    python -I tools/wsp/media/fabric/to_scorer_format.py --src runs/dev/media/fabric/tracks/psi25 \
        --out runs/dev/media/fabric/tracks_scorer/psi25 [--folders train held_validation held_test]

Prints counts per folder only.
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

SCHEMA = "wsp-bass-front-track/1"
PARTITION = {"train": "train", "held_validation": "validation", "held_test": "test"}
TIP_W_TOL = 0.002  # agreement required between the recomputed width and the measurer's own tip_w/base_w (rounding)


def sha256_file(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


class Missing(Exception):
    pass


def need(d, *path):
    cur = d
    for k in path:
        if not isinstance(cur, dict) or cur.get(k) is None:
            raise Missing(".".join(path))
        cur = cur[k]
    return cur


def direction_of(text):
    t = (text or "").lower()
    if "smaller x" in t:
        return -1
    if "larger x" in t:
        return 1
    raise Missing("spread_direction")


def end_of(outcome, end):
    """The scorer's `end` ({kind, t_s}) from the measurer's outcome and end, by a fixed table.

    never_lit                         -> no_spread, t_s null
    went_out                          -> extinction at end.t_last_flame_s
    spread_to_end, tip left picture   -> left_view at end.t_last_flame_s
    spread_to_end, otherwise          -> extinction at end.t_last_flame_s (flame gone at the sample end;
                                         the earlier T5 track calls its burn-out at the end `extinction`)
    cannot_tell, flame_at_record_end  -> record_end at end.t_last_flame_s (censored after it)
    cannot_tell, other, with an end   -> lost at end.t_last_flame_s (censored after it)
    cannot_tell without an end        -> not convertible (the scorer's ignition score would read it as spread)
    """
    kind = need(outcome, "kind")
    if kind == "never_lit":
        return {"kind": "no_spread", "t_s": None}
    if not isinstance(end, dict) or end.get("t_last_flame_s") is None:
        raise Missing("end.t_last_flame_s")
    t = end["t_last_flame_s"]
    if kind == "went_out":
        return {"kind": "extinction", "t_s": t}
    if kind == "spread_to_end":
        if "leaves the captured picture" in (outcome.get("reason") or ""):
            return {"kind": "left_view", "t_s": t}
        return {"kind": "extinction", "t_s": t}
    if kind == "cannot_tell":
        if outcome.get("sub") == "flame_at_record_end":
            return {"kind": "record_end", "t_s": t}
        return {"kind": "lost", "t_s": t}
    raise Missing("outcome.kind (unknown value)")


def material_index(material):
    m = (material or "").lower()
    if "sibal" in m:
        return 0
    if "pmma" in m and ("1 mm" in m or "1mm" in m):
        return 1
    return None


def convert(d, folder, src_path):
    if d.get("schema") != SCHEMA:
        raise Missing("schema")
    run_id = need(d, "run_id")
    part = PARTITION[folder]
    fp = d.get("partition")
    if fp is not None and PARTITION.get(fp, fp) != part:
        raise Missing("partition (file disagrees with folder)")
    origin = need(d, "position_origin_fw")
    end_fw = need(d, "sample_end_fw")
    scale = need(d, "width_scale_w_per_fw")
    direction = direction_of(need(d, "spread_direction"))
    c = need(d, "conditions")
    x_o2 = need(c, "x_o2_initial")
    material = need(c, "material")
    flow_dir = need(c, "flow_direction")
    if str(flow_dir).lower() not in ("concurrent", "opposed"):
        raise Missing("conditions.flow_direction (not concurrent/opposed)")
    on = need(d, "igniter", "on_t_s")
    off = need(d, "igniter", "off_t_s")
    flow = need(c, "flow")
    cal = c.get("fan_calibration") or []
    duct = d.get("duct")
    if duct is None:
        raise Missing("duct")
    outcome = need(d, "outcome")
    end = end_of(outcome, d.get("end"))

    def w(x):
        return -direction * (origin - x) * scale

    stills = []
    for p in duct:
        tip = None if p.get("tip_fw") is None else w(p["tip_fw"])
        base = None if p.get("base_fw") is None else w(p["base_fw"])
        for name, v in (("tip_w", tip), ("base_w", base)):
            if p.get(name) is not None and v is not None and abs(p[name] - v) > TIP_W_TOL:
                raise Missing(f"duct.{name} (measurer's value disagrees with its own origin and scale)")
        stills.append({
            "t_s": p["t_s"],
            "tip_w": None if tip is None else round(tip, 4),
            "base_w": None if base is None else round(base, 4),
            "state": "flame",
            "quality": None,
            "tip_fw": p.get("tip_fw"),
            "base_fw": p.get("base_fw"),
            "area_px": p.get("area_px"),
            "score": p.get("score"),
        })
    fan = []
    unread = 0
    for e in flow:
        if e.get("fan") is None:
            unread += 1  # FAN cell not read in that second: no value to carry; the step function holds the previous reading
            continue
        fan.append({"t_s": e["t_s"], "fan": e["fan"], "u_cm_s": e.get("u_cm_s")})
    if not fan:
        raise Missing("conditions.flow (no FAN reading)")
    if any(e["u_cm_s"] is None for e in fan) and not cal:
        raise Missing("conditions.fan_calibration")

    conditions = {
        "x_o2_initial": x_o2,
        "x_o2_final": c.get("x_o2_final"),
        "sample_label": material,
        "flow": str(flow_dir).lower(),
        "pressure_atm": c.get("pressure_atm"),
        "width_m_nominal": c.get("width_m"),
        "restrictor": c.get("restrictor"),
        "gmt_day": c.get("gmt_day"),
        "date": c.get("date"),
        "sample": c.get("sample"),
        "source": c.get("source"),
    }
    mi = material_index(material)
    if mi is not None:
        conditions["material_index"] = mi
    return {
        "schema": SCHEMA,
        "run_id": run_id,
        "partition": part,
        "time_origin": d.get("time_origin"),
        "position_unit": "sample_width",
        "position_origin": (f"the measurer's origin position_origin_fw = {origin} fw (igniter box upstream edge + 0.006 fw); "
                            f"positive downstream along the spread direction; {scale} sample widths per frame width"),
        "conditions": conditions,
        "geometry": {
            "window_length_w": round(abs(origin - end_fw) * scale, 4),
            "source_view": "duct (main) video",
        },
        "igniter": {"on_t_s": on, "off_t_s": off},
        "fan": fan,
        "fan_calibration": cal,
        "stills": stills,
        "end": end,
        "ignition_t_s": d.get("ignition_t_s"),
        "measured_outcome": outcome,
        "measured_end": d.get("end"),
        "quality": d.get("quality"),
        "qc_flags": d.get("qc_flags"),
        "file_role": d.get("file_role"),
        "source": d.get("source"),
        "measurement": d.get("measurement"),
        "conversion": {
            "script": "tools/wsp/media/fabric/to_scorer_format.py",
            "script_sha256": sha256_file(__file__),
            "from_file": src_path.as_posix(),
            "from_sha256": sha256_file(src_path),
            "stills_from": "duct points, one still per point, none dropped; state = flame, quality = null (no per-point quality in the source)",
            "fan_entries_unread_omitted": unread,
            "sample_end_fw": end_fw,
        },
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--folders", nargs="+", default=["train", "held_validation", "held_test"])
    args = ap.parse_args()
    src, out = Path(args.src), Path(args.out)
    report = {}
    left_out = {}
    for folder in args.folders:
        if folder not in PARTITION:
            sys.exit(f"folder {folder!r} has no scorer partition")
        dst = out / PARTITION[folder]
        dst.mkdir(parents=True, exist_ok=True)
        n = {"converted": 0, "left_out_quality_failed": 0, "left_out_missing_field": 0}
        lst = []
        for f in sorted((src / folder).glob("*.json")):
            d = json.loads(f.read_text(encoding="utf-8"))
            if d.get("quality") == "failed":
                n["left_out_quality_failed"] += 1
                lst.append({"file": f.name, "reason": "quality failed"})
                continue
            try:
                t = convert(d, folder, f)
            except Missing as e:
                n["left_out_missing_field"] += 1
                lst.append({"file": f.name, "reason": f"missing or unusable field: {e}"})
                continue
            (dst / f.name).write_text(json.dumps(t, indent=1), encoding="utf-8")
            n["converted"] += 1
        report[PARTITION[folder]] = n
        left_out[PARTITION[folder]] = lst
    # kept outside the partition folders; names and reason categories only
    (out / "left_out.json").write_text(json.dumps(left_out, indent=1), encoding="utf-8")
    print(json.dumps(report))


if __name__ == "__main__":
    main()

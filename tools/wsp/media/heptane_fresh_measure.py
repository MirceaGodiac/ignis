"""Fetch and measure the frozen fresh blind n-heptane batch (docs/wsp/atlas/splits/heptane-fresh-blind.json).

Blind measurement: writes diameter series, tool outputs and contact sheets; computes no burn time, burning
rate or extinction, reads no model output, and prints only counts and quality.

    python -I tools/wsp/media/heptane_fresh_measure.py fetch [--connections 128 --files 4]
    python -I tools/wsp/media/heptane_fresh_measure.py measure [--workers 6]   # polls the manifest
    python -I tools/wsp/media/heptane_fresh_measure.py relabel --visual runs/dev/heptane_fresh/VISUAL_CHECK.tsv

Everything the measurement depends on is read from the freeze file and checked: the tool's sha256, the
field of view (30.25 mm over the image width), the time origin stamp and the acceptance rule of
runs/dev/media/droplet/FREEZE.json. The download reuses fetch_flex2_batch.fetch_file (ranged parallel
requests, zip CRC verification) without changing it. An archive is deleted once its series and sheets
are written and read back.
"""
import argparse
import concurrent.futures as cf
import csv
import hashlib
import json
import os
import shutil
import sys
import time
import traceback
from datetime import datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
FREEZE_BATCH = REPO / "docs/wsp/atlas/splits/heptane-fresh-blind.json"
TOOL_FREEZE = REPO / "runs/dev/media/droplet/FREEZE.json"
TOOL = HERE / "droplet_diameter.py"
OUT = REPO / "runs/dev/heptane_fresh"
RAW = OUT / "raw"
FOV_MM = 30.25
SEEN = {"ok", "soot", "glare", "outlier", "edge", "static"}


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 22), b""):
            h.update(chunk)
    return h.hexdigest()


def load_batch():
    b = json.loads(FREEZE_BATCH.read_text(encoding="utf-8"))
    tf = json.loads(TOOL_FREEZE.read_text(encoding="utf-8"))
    if sha256(TOOL) != b["method"]["tool_sha256"] or tf["tool_sha256"] != b["method"]["tool_sha256"]:
        sys.exit("droplet_diameter.py differs from the freeze: refusing")
    return b, tf


def text_src(t, suffix):
    if t["investigation"] == "PSI-68":
        return REPO / "runs/dev/data_all/raw/PSI-68/Raw Data" / f"{t['test_id']}{suffix}"
    return REPO / "runs/dev/data_all/raw/PSI-69/Raw Data" / f"PSI-69_Raw Data_{t['test_id']}{suffix}"


# ------------------------------------------------------------------------------------------ fetch
def cmd_fetch(a):
    import fetch_flex2_batch as fb  # used unchanged
    b, _ = load_batch()
    want = [t for t in b["tests"] if t["in_measured_batch"]]
    listing = {}
    for inv in ("PSI-68", "PSI-69"):
        for line in (REPO / f"runs/dev/data_all/manifests/{inv}.listing.jsonl").open(encoding="utf-8"):
            r = json.loads(line)
            if r["file_name"] not in listing or r["file_version"] > listing[r["file_name"]]["file_version"]:
                listing[r["file_name"]] = r
    RAW.mkdir(parents=True, exist_ok=True)
    for stale in RAW.glob("*.part"):
        stale.unlink()
    man = RAW / "MANIFEST.jsonl"
    known = set()
    if man.exists():
        known = {json.loads(l)["archive"] for l in man.open(encoding="utf-8") if '"verified"' in l}
    pool = cf.ThreadPoolExecutor(a.connections)
    t0, done, lock = time.time(), [], __import__("threading").Lock()
    measured = set()
    log = OUT / "measure_log.jsonl"
    if log.exists():
        measured = {json.loads(l)["test_id"] for l in log.open(encoding="utf-8")}

    def one(t):
        if t["archive"] in known or t["test_id"] in measured:
            return
        row = listing[t["listing_file"]]
        assert row["size_bytes"] == t["archive_bytes"], t["test_id"]
        dest = RAW / t["archive"]
        status, verified = fb.fetch_file(row, dest, pool, a.min_free_gb)
        rec = dict(archive=t["archive"], test_id=t["test_id"], key=row["key"], size_bytes=row["size_bytes"],
                   status=status)
        if status in ("ok", "present"):
            rec.update(verified=verified, sha256=sha256(dest))
        with lock:
            done.append(rec)
            with man.open("a", encoding="utf-8") as f:
                f.write(json.dumps(rec) + "\n")
            gb = sum(d["size_bytes"] for d in done if d["status"] == "ok") / 1e9
            dt = time.time() - t0
            print(f"{len(done)} {t['test_id']} {status}; {gb:.1f} GB in {dt:.0f} s ({gb * 1e3 / max(dt, 1):.1f} MB/s)",
                  flush=True)

    with cf.ThreadPoolExecutor(a.files) as ex:
        list(ex.map(one, want))  # in the frozen order
    pool.shutdown()
    print("fetch done", flush=True)


# ------------------------------------------------------------------------------------------ measure
def accept(meta, rows, rule):
    """FREEZE.json acceptance rule (as flex2_measure.accept). Returns (quality, reasons, stats)."""
    usable = set(rule["usable_flags"])
    reasons = []
    cc = meta.get("clock_check")
    if not cc or abs(cc["seen_minus_stamp_s"]) > rule["max_clock_check_s"]:
        reasons.append(f"clock check failed: {cc and cc['seen_minus_stamp_s']}")
    t_dep = meta.get("t_deploy_used_s")
    post = [r for r in rows if r.get("t_s") is not None and t_dep is not None and r["t_s"] >= t_dep + 0.1]
    good = [r for r in post if r["flag"] in usable]
    stats = dict(frames=len(rows), post_deploy_frames=len(post), usable_frames=len(good))
    if len(good) < rule["min_usable_frames"]:
        reasons.append(f"{len(good)} usable frames < {rule['min_usable_frames']}")
        return "not_usable", reasons, stats
    t_last = max(r["t_s"] for r in good)
    span = [r for r in post if r["t_s"] <= t_last]
    cov = len(good) / max(1, len(span))
    stats["coverage_of_measured_span"] = round(cov, 3)
    if cov < rule["min_coverage"]:
        reasons.append(f"coverage {cov:.2f} < {rule['min_coverage']}")
    return ("usable" if not reasons else "not_usable"), reasons, stats


def conditions(t):
    a = t["ambient"]
    src = "docs/wsp/atlas/splits/heptane-fresh-blind.json"
    ev = a["evidence"]
    c = {}
    for key in ("x_o2", "x_co2", "x_he", "x_n2"):
        v = a.get(key)
        if v is None:
            c[key] = dict(value=None, unit="1", source=src, resolution="conflicting",
                          alternatives=a.get(key + "_alternatives"), note=a["how"])
        else:
            c[key] = dict(value=float(v), unit="1", source=src, evidence=ev, grade=a["grade"])
    c["x_o2"]["note"] = a["how"]
    p = t["pressure"]
    c["pressure"] = dict(value=p["value_pa"], unit="Pa", source=src, note=p["how"], evidence=p["evidence"])
    c["fuel"] = dict(value="Heptane", unit=None, source=src, note=t["fuel_how"], evidence="original_observation")
    c["configuration"] = dict(value=t["configuration"], source=src, note=t["configuration_how"])
    return c


def measure(t, zip_path, zip_sha, rule, stamp):
    import droplet_diameter as dd  # the frozen tool, unchanged
    from flex2_truncation_screen import screen  # unchanged
    tp = t["test_id"]
    t0 = time.time()
    work = OUT / "work" / tp
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    staged = work / zip_path.name
    os.link(zip_path, staged)
    tl, st = text_src(t, ".txt"), text_src(t, "_HIBMS_Camera_Settings.txt")
    shutil.copyfile(tl, work / f"{t['investigation']}_{tp}.txt")
    if st.exists():
        shutil.copyfile(st, work / f"{tp}_HIBMS_Camera_Settings.txt")
    sheet = OUT / "check" / tp
    series_p = OUT / "series" / f"{tp}.json"
    series_p.parent.mkdir(parents=True, exist_ok=True)
    result = dict(test_id=tp, archive=zip_path.name, archive_sha256=zip_sha, archive_bytes=zip_path.stat().st_size)
    try:
        meta, rows = dd.run(work, investigation="FLEX", fov_mm=FOV_MM, sheet_dir=sheet)
        dd.write(meta, rows, OUT / "tool" / tp)
        quality, reasons, stats = accept(meta, rows, rule)
        fc = meta.get("flag_counts") or {}
        post = sum(v for k, v in fc.items() if k != "pre_deploy")
        seen = sum(v for k, v in fc.items() if k in SEEN)
        if quality == "not_usable" and post and seen / post < 0.05:
            quality, reasons = "failed", ["droplet not detected by the tool in > 95 % of post-deployment frames"] + reasons
        sc = screen(str(OUT / "tool" / f"{tp}.csv"))
        if quality == "usable" and sc.get("hijacked"):
            quality, reasons = "failed", [f"track jumped to another object ({sc['jumps']} diameter jumps > 30 % "
                                          "between usable frames)"]
        elif quality == "usable" and sc.get("truncated"):
            quality, reasons = "failed", [f"track truncated while the droplet was still in view ({sc['n']} "
                                          "droplet-sized lost frames)"]
        inp = dd.Inputs(work)
        ign = inp.events.get("AS Burn to Completion/Igniter On")
        off = inp.events.get(stamp)
        inp.z.close()
        if not (isinstance(ign, datetime) and isinstance(off, datetime)):
            quality, reasons, shift = "failed", reasons + ["timeline without igniter stamps"], None
        else:
            shift = (off - ign).total_seconds()
        usable = set(rule["usable_flags"])
        obs = []
        if shift is not None:
            for r in rows:
                if r["flag"] in usable and r.get("t_s") is not None and r["t_s"] >= meta["t_deploy_used_s"] + 0.1:
                    obs.append(dict(quantity="liquid_diameter", time_s=round(r["t_s"] - shift, 6),
                                    value_si=float(r["d_m"]), quality_flags=[], note=f"tool flag {r['flag']}",
                                    source=zip_path.name, sha256=zip_sha, row=int(r["frame"]),
                                    channel="hibms_backlit"))
        series = dict(
            run_id=f"{t['investigation']}:{tp}", session_id=t["date"], investigation=t["investigation"],
            partition="fresh_blind", batch="heptane-fresh-blind", conditions=conditions(t), observations=obs,
            endpoint=None, quality=quality, quality_reasons=reasons, quality_stats=stats, screens=sc,
            time_base=dict(
                declared_as="FLEX time origin (NASA DropletData processed time base), by declared equivalence",
                placed_at=f"timeline '{stamp}' stamp = 0 s", unit="s",
                rack_igniter_on_s=-shift if shift is not None else None,
                why="FREEZE.json time_origin: on five FLEX train tests NASA's t = 0 fell -0.31 to +0.41 s from "
                    "this stamp", camera_clock_check=meta.get("clock_check")),
            measurement=dict(tool="tools/wsp/media/droplet_diameter.py", tool_sha256=rule["tool_sha256"],
                             method_version=meta["method_version"], px_per_mm=meta["px_per_mm"],
                             m_per_px=meta.get("m_per_px"), scale_source=meta["scale_source"],
                             scale_basis="freeze heptane-fresh-blind.json method.pixel_scale",
                             ruler_check=meta.get("ruler_check"), image_px=meta["image_px"],
                             settings=meta.get("settings"), flag_counts=fc,
                             freeze="docs/wsp/atlas/splits/heptane-fresh-blind.json"))
        series_p.write_text(json.dumps(series, indent=1) + "\n", encoding="utf-8")
        back = json.loads(series_p.read_text(encoding="utf-8"))
        sheets = sorted(sheet.glob("*.png"))
        ok = back["run_id"] == series["run_id"] and len(back["observations"]) == len(obs) and (
            sheets or quality != "usable")
        result.update(quality=quality, reasons=reasons, observations=len(obs), sheets=len(sheets), verified=bool(ok),
                      compute_s=round(time.time() - t0, 1), px_per_mm=meta["px_per_mm"], image_px=meta["image_px"],
                      ruler=meta.get("ruler_check"), clock_check=meta.get("clock_check"), flag_counts=fc, screens=sc)
    except Exception as e:  # the tool failed: reported, not retuned
        result.update(quality="failed", reasons=[f"tool error: {type(e).__name__}: {e}"],
                      trace=traceback.format_exc()[-2000:], verified=False, compute_s=round(time.time() - t0, 1))
        series_p.write_text(json.dumps(dict(run_id=f"{t['investigation']}:{tp}", session_id=t["date"],
                                            partition="fresh_blind", observations=[], quality="failed",
                                            quality_reasons=result["reasons"]), indent=1) + "\n", encoding="utf-8")
    freed = 0
    if result.get("verified"):
        import gc
        gc.collect()
        for p in (staged, zip_path):
            for _ in range(3):
                try:
                    sz = p.stat().st_size
                    p.unlink()
                    freed += sz if p == zip_path else 0
                    break
                except FileNotFoundError:
                    break
                except PermissionError:
                    time.sleep(2)
        shutil.rmtree(work, ignore_errors=True)
    result["bytes_freed"] = freed
    return result


def cmd_measure(a):
    b, tf = load_batch()
    rule = dict(tf["acceptance"], tool_sha256=tf["tool_sha256"])
    stamp = tf["time_origin"]["timeline_stamp"]
    assert stamp == b["method"]["time_origin"]["stamp"]
    tests = {t["archive"]: t for t in b["tests"] if t["in_measured_batch"]}
    log = OUT / "measure_log.jsonl"
    done = set()
    if log.exists():
        done = {json.loads(l)["archive"] for l in log.open(encoding="utf-8")}
    pending = {}
    with cf.ProcessPoolExecutor(a.workers) as ex:
        while True:
            man = RAW / "MANIFEST.jsonl"
            recs = {}
            if man.exists():
                for line in man.open(encoding="utf-8"):
                    r = json.loads(line)
                    recs[r["archive"]] = r
            for name, r in recs.items():
                if name in done or name in pending or name not in tests:
                    continue
                if r["status"] in ("ok", "present") and (RAW / name).exists():
                    zp = RAW / name
                    if sha256(zp) != r["sha256"]:
                        res = dict(archive=name, test_id=tests[name]["test_id"], quality="failed",
                                   reasons=["sha256 differs from MANIFEST"], verified=False, bytes_freed=0)
                        with log.open("a", encoding="utf-8") as f:
                            f.write(json.dumps(res) + "\n")
                        done.add(name)
                        continue
                    pending[name] = ex.submit(measure, tests[name], zp, r["sha256"], rule, stamp)
                elif r["status"] not in ("ok", "present"):
                    print(f"download not ok: {name} {r['status']}", flush=True)
            for name, fut in list(pending.items()):
                if fut.done():
                    try:
                        res = fut.result()
                    except Exception as e:
                        res = dict(archive=name, test_id=tests[name]["test_id"], quality="failed",
                                   reasons=[f"worker: {e!r}"], verified=False, bytes_freed=0)
                    with log.open("a", encoding="utf-8") as f:
                        f.write(json.dumps(res, default=str) + "\n")
                    done.add(name)
                    del pending[name]
                    print(f"{len(done)}/{len(tests)} {res['test_id']} {res.get('quality')} "
                          f"freed {res.get('bytes_freed', 0) / 1e9:.2f} GB", flush=True)
            if len(done) == len(tests) or (a.once and not pending):
                break
            time.sleep(a.poll_s)
    print("measure done", flush=True)


def cmd_relabel(a):
    """Apply the measurer's visual check (tab-separated: test_id, PASS|FAIL, end of series, note). Fails only.

    'end of series' says why the written series stops (a measurement fact, so that a scorer does not read
    the last frame as the end of the burn): left_view (the droplet drifted out of the field of view),
    vanished_in_view (the droplet shrank or went out while in view), track_lost, planned_stop (frames end)."""
    rows = [l.split("\t") for l in Path(a.visual).read_text(encoding="utf-8").splitlines()
            if l.strip() and not l.startswith("#")]
    for p in rows:
        tp, verdict = p[0], p[1]
        end, why = (p[2] if len(p) > 2 else ""), (p[3] if len(p) > 3 else "")
        sp = OUT / "series" / f"{tp}.json"
        s = json.loads(sp.read_text(encoding="utf-8"))
        s["visual_check"] = dict(verdict=verdict, end_of_series=end, note=why, by="measurer, contact sheets")
        s["endpoint_note"] = dict(end_of_series=end, how="measurer's visual check of the contact sheets")
        if verdict == "FAIL" and s["quality"] == "usable":
            s["quality"], s["quality_reasons"] = "failed", [f"visual check: {why}"]
            s["quality_rule_note"] = "relabelled by the visual check; no value changed"
        sp.write_text(json.dumps(s, indent=1) + "\n", encoding="utf-8")
        print(tp, verdict, s["quality"])


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    f = sub.add_parser("fetch")
    f.add_argument("--connections", type=int, default=128)
    f.add_argument("--files", type=int, default=4)
    f.add_argument("--min-free-gb", type=float, default=150.0)
    m = sub.add_parser("measure")
    m.add_argument("--workers", type=int, default=6)
    m.add_argument("--poll-s", type=float, default=20.0)
    m.add_argument("--once", action="store_true", help="stop when nothing is pending instead of waiting")
    r = sub.add_parser("relabel")
    r.add_argument("--visual", required=True)
    a = ap.parse_args()
    dict(fetch=cmd_fetch, measure=cmd_measure, relabel=cmd_relabel)[a.cmd](a)


if __name__ == "__main__":
    main()

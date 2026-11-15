"""Measure a frozen FLEX-2 image batch with the frozen droplet tool, as archives arrive.

Blind measurement: this script writes diameter series and contact sheets; it computes no burn time,
burning rate or extinction and prints only counts and quality.

    python -I tools/wsp/media/flex2_measure.py --batch docs/wsp/atlas/splits/PSI-68-batch1.json \
        --raw runs/dev/media/droplet/flex2_raw --workers 12

For each archive listed with status ok in <raw>/MANIFEST.jsonl:
  1. check its sha256 against the manifest;
  2. stage it (hard link) with its text files (timeline, HiBMs settings) under the names the tool reads;
  3. run tools/wsp/media/droplet_diameter.py (sha256 checked against FREEZE.json, never edited) with
     --investigation FLEX-2 and the contact sheets;
  4. apply the FREEZE.json acceptance rule and write the series in the external-test format of
     tools/wsp/ext/transfer_droplet.py: flex2_series/<partition>/<test>.json;
  5. re-read and verify the series and the sheet, then delete the archive; log the bytes freed.
"""
import argparse
import concurrent.futures as cf
import hashlib
import json
import os
import shutil
import sys
import time
import traceback
from datetime import datetime
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
TOOL = REPO / "tools/wsp/media/droplet_diameter.py"
OUT = REPO / "runs/dev/media/droplet"
TEXT = REPO / "runs/dev/data_all/raw/PSI-68/Raw Data"
TESTS = REPO / "runs/dev/data_all/work/atlas/flex2_tests.jsonl"
FREEZE = OUT / "FREEZE.json"
FUEL_NAME = {"n-heptane": "Heptane"}
sys.path.insert(0, str(TOOL.parent))  # also in the spawned workers  # the FLEX records' spelling; other fuels keep the catalogue name


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 22), b""):
            h.update(chunk)
    return h.hexdigest()


def text_file(tp, suffix, text_dir=None):
    d = Path(text_dir) if text_dir else TEXT
    for name in (f"{tp}{suffix}", f"PSI-68_Raw Data_{tp}{suffix}", f"PSI-68_{tp}{suffix}",
                 f"PSI-39_Raw Data_{tp}{suffix}", f"PSI-39_{tp}{suffix}"):
        p = d / name
        if p.exists():
            return p
    return None


def accept(meta, rows, rule):
    """The FREEZE.json acceptance rule. Returns (quality, reasons, stats)."""
    usable = set(rule["usable_flags"])
    reasons = []
    cc = meta.get("clock_check")
    if not cc or abs(cc["seen_minus_stamp_s"]) > rule["max_clock_check_s"]:
        reasons.append(f"clock check failed: {cc and cc['seen_minus_stamp_s']}")
    t_dep = meta.get("t_deploy_used_s")
    post = [r for r in rows if r.get("t_s") is not None and t_dep is not None and r["t_s"] >= t_dep + 0.1]
    good = [r for r in post if r["flag"] in usable]
    stats = dict(frames=len(rows), usable_frames=len(good))
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


def conditions(rec):
    amb = rec.get("ambient") or {}
    src = "runs/dev/data_all/work/atlas/flex2_tests.jsonl"
    c = {}
    for key, gas in (("x_o2", "O2"), ("x_co2", "CO2"), ("x_he", "He")):
        if gas in amb:
            c[key] = dict(value=float(amb[gas]), unit="1", source=src, note=rec.get("ambient_how"),
                          evidence="original_observation")
        else:
            c[key] = dict(value=None, unit="1", source=src, resolution="unknown")
    if rec.get("pressure_pa") is not None:
        c["pressure"] = dict(value=float(rec["pressure_pa"]), unit="Pa", source=src, note=rec.get("pressure_how"),
                             evidence="original_observation")
    else:
        c["pressure"] = dict(value=None, unit="Pa", source=src, resolution="unknown")
    c["fuel"] = dict(value=FUEL_NAME.get(rec["fuel"], rec["fuel"]), unit=None, source=src, note=rec.get("fuel_how"),
                     evidence="original_observation")
    return c


def measure(test, zip_path, zip_sha, rule, ign_off_key, dirs=None):
    import droplet_diameter as dd  # the frozen tool
    dirs = dirs or dict(work="flex2_work", check="flex2_check", tool="flex2_tool", series="flex2_series/{part}")
    tp, part = test["test_id"], test.get("partition", "test")
    t0 = time.time()
    work = OUT / dirs["work"] / tp
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    staged = work / zip_path.name
    os.link(zip_path, staged)
    tl = text_file(tp, ".txt", dirs.get("text"))
    st = text_file(tp, "_HIBMS_Camera_Settings.txt", dirs.get("text"))
    if tl:
        shutil.copyfile(tl, work / f"PSI-68_{tp}.txt")
    if st:
        shutil.copyfile(st, work / f"{tp}_HIBMS_Camera_Settings.txt")
    rec = {}
    for line in (Path(dirs["catalogue"]) if dirs.get("catalogue") else TESTS).open(encoding="utf-8"):
        r = json.loads(line)
        if r.get("test_id", r.get("cir")) == tp:
            rec = r
            break
    sheet = OUT / dirs["check"] / tp
    result = dict(test_id=tp, partition=part, fuel=test["fuel"], archive=zip_path.name, archive_sha256=zip_sha,
                  archive_bytes=zip_path.stat().st_size)
    try:
        meta, rows = dd.run(work, investigation="FLEX-2", sheet_dir=sheet)
        dd.write(meta, rows, OUT / dirs["tool"] / tp)
        quality, reasons, stats = accept(meta, rows, rule)
        inp = dd.Inputs(work)
        ign = inp.events.get("AS Burn to Completion/Igniter On")
        off = inp.events.get(ign_off_key)
        inp.z.close()
        if not (isinstance(ign, datetime) and isinstance(off, datetime)):
            quality, reasons = "failed", reasons + ["timeline without igniter stamps"]
            shift = None
        else:
            shift = (off - ign).total_seconds()
        usable = set(rule["usable_flags"])
        obs = []
        if shift is not None:
            for r in rows:
                if r["flag"] in usable and r.get("t_s") is not None and r["t_s"] >= meta["t_deploy_used_s"] + 0.1:
                    obs.append(dict(quantity="liquid_diameter", time_s=round(r["t_s"] - shift, 6),
                                    value_si=float(r["d_m"]), quality_flags=[],
                                    note=f"tool flag {r['flag']}", source=zip_path.name, sha256=zip_sha,
                                    row=int(r["frame"]), channel="hibms_backlit"))
        series = dict(
            run_id=f"PSI-68:{tp}", session_id=test["date"], investigation="PSI-68", partition=part,
            conditions=conditions(rec), observations=obs, endpoint=None,
            quality=quality, quality_reasons=reasons, quality_stats=stats,
            time_base=dict(
                declared_as="FLEX time origin (NASA DropletData processed time base), by declared equivalence",
                placed_at=f"timeline '{ign_off_key}' stamp = 0 s",
                rack_igniter_on_s=-shift if shift is not None else None,
                why="FREEZE.json time_origin: on six FLEX train tests NASA's t = 0 fell -0.30 to +0.41 s from this "
                    "stamp (mean ~0) and +0.58 to +1.72 s after igniter-on",
                camera_clock_check=meta.get("clock_check")),
            measurement=dict(tool="tools/wsp/media/droplet_diameter.py", tool_sha256=rule["tool_sha256"],
                             method_version=meta["method_version"], px_per_mm=meta["px_per_mm"],
                             scale_source=meta["scale_source"], ruler_check=meta.get("ruler_check"),
                             image_px=meta["image_px"], settings=meta.get("settings"),
                             flag_counts=meta.get("flag_counts"), freeze="runs/dev/media/droplet/FREEZE.json"))
        dest = OUT / dirs["series"].format(part=part) / f"{tp}.json"
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(json.dumps(series, indent=1) + "\n", encoding="utf-8")
        # verify before deleting the archive
        back = json.loads(dest.read_text(encoding="utf-8"))
        sheets = sorted(sheet.glob("*.png"))
        ok = back["run_id"] == series["run_id"] and len(back["observations"]) == len(obs) and (
            sheets or quality != "usable")
        result.update(quality=quality, reasons=reasons, observations=len(obs), sheets=len(sheets), verified=bool(ok),
                      compute_s=round(time.time() - t0, 1), scale_source=meta["scale_source"],
                      px_per_mm=meta["px_per_mm"], image_px=meta["image_px"], ruler=meta.get("ruler_check"),
                      clock_check=meta.get("clock_check"), flag_counts=meta.get("flag_counts"))
    except Exception as e:  # the tool failed: reported, not retuned
        result.update(quality="failed", reasons=[f"tool error: {type(e).__name__}: {e}"],
                      trace=traceback.format_exc()[-2000:], verified=False, compute_s=round(time.time() - t0, 1))
        dest = OUT / dirs["series"].format(part=part) / f"{tp}.json"
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(json.dumps(dict(run_id=f"PSI-68:{tp}", session_id=test["date"], investigation="PSI-68",
                                        partition=part, observations=[], endpoint=None, quality="failed",
                                        quality_reasons=result["reasons"]), indent=1) + "\n", encoding="utf-8")
    freed = 0
    if result.get("verified"):
        import gc
        gc.collect()
        for p in (staged, zip_path):
            try:
                freed += p.stat().st_size if p == zip_path else 0
                p.unlink()
            except FileNotFoundError:
                pass
            except PermissionError:
                time.sleep(2)
                p.unlink()
        shutil.rmtree(work, ignore_errors=True)
    result["bytes_freed"] = freed
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--batch", required=True)
    ap.add_argument("--raw", required=True)
    ap.add_argument("--workers", type=int, default=12)
    ap.add_argument("--poll-s", type=float, default=30.0)
    ap.add_argument("--tag", default="", help="suffix of the output folders and log, e.g. _test")
    ap.add_argument("--flat-series", action="store_true", help="series directly in flex2_series<tag>/")
    ap.add_argument("--text-dir", default=None, help="folder of the timelines and camera settings (default PSI-68)")
    ap.add_argument("--catalogue", default=None, help="conditions catalogue jsonl (default the FLEX-2 one)")
    ap.add_argument("--investigation-id", default="PSI-68")
    a = ap.parse_args()
    freeze = json.loads(FREEZE.read_text(encoding="utf-8"))
    if sha256(TOOL) != freeze["tool_sha256"]:
        sys.exit("droplet_diameter.py differs from FREEZE.json: refusing to measure")
    rule = dict(freeze["acceptance"], tool_sha256=freeze["tool_sha256"])
    ign_off_key = freeze["time_origin"]["timeline_stamp"]
    batch = json.loads(Path(a.batch).read_text(encoding="utf-8"))
    tests = {t["archive"]: t for t in batch["tests"]}
    raw = Path(a.raw)
    tag = a.tag
    dirs = dict(work=f"flex2_work{tag}", check=f"flex2_check{tag}", tool=f"flex2_tool{tag}",
                series=f"flex2_series{tag}" if a.flat_series else f"flex2_series{tag}/{{part}}",
                text=a.text_dir, catalogue=a.catalogue, investigation=a.investigation_id)
    log = OUT / f"flex2_measure_log{tag}.jsonl"
    done = set()
    if log.exists():
        for line in log.open(encoding="utf-8"):
            done.add(json.loads(line)["archive"])
    pending, seen_records = {}, {}
    with cf.ProcessPoolExecutor(a.workers) as ex:
        while True:
            man = raw / "MANIFEST.jsonl"
            if man.exists():
                for line in man.open(encoding="utf-8"):
                    r = json.loads(line)
                    seen_records[r["archive"]] = r
            for name, r in seen_records.items():
                if name in done or name in pending or name not in tests:
                    continue
                if r["status"] in ("ok", "present") and (raw / name).exists():
                    zp = raw / name
                    if sha256(zp) != r["sha256"]:
                        res = dict(archive=name, test_id=tests[name]["test_id"], quality="failed",
                                   reasons=["sha256 differs from MANIFEST"], verified=False, bytes_freed=0)
                        log.open("a", encoding="utf-8").write(json.dumps(res) + "\n")
                        done.add(name)
                        continue
                    pending[name] = ex.submit(measure, tests[name], zp, r["sha256"], rule, ign_off_key, dirs)
                elif r["status"] not in ("ok", "present"):
                    res = dict(archive=name, test_id=tests[name]["test_id"], partition=tests[name]["partition"],
                               fuel=tests[name]["fuel"], quality="failed", reasons=[f"download: {r['status']}"],
                               verified=False, bytes_freed=0)
                    log.open("a", encoding="utf-8").write(json.dumps(res) + "\n")
                    done.add(name)
            for name, fut in list(pending.items()):
                if fut.done():
                    try:
                        res = fut.result()
                    except Exception as e:
                        res = dict(archive=name, test_id=tests[name]["test_id"], quality="failed",
                                   reasons=[f"worker: {e!r}"], verified=False, bytes_freed=0)
                    log.open("a", encoding="utf-8").write(json.dumps(res, default=str) + "\n")
                    done.add(name)
                    del pending[name]
                    free = shutil.disk_usage(OUT).free / 1e9
                    held = sum(p.stat().st_size for p in raw.glob("*.zip*")) / 1e9
                    print(f"{len(done)}/{len(tests)} {res['test_id']} {res.get('quality')} "
                          f"freed {res.get('bytes_freed', 0) / 1e9:.2f} GB; raw holds {held:.1f} GB; D: free {free:.0f} GB",
                          flush=True)
            if len(done) == len(tests):
                break
            if shutil.disk_usage(OUT).free < 150e9:
                print("STOP: drive D: below 150 GB free", flush=True)
            time.sleep(a.poll_s)
    print("all batch archives handled", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())

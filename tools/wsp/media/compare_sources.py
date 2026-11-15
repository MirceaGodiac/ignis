#!/usr/bin/env python3
"""Compare the image sources of `flame_radius.py` with each other and with the sensor endpoints of
`tools/wsp/ext/flame_design.py`, for the tests whose series are in <out>/series/.

For every test:
  * colour camera: radius noise (scatter of r about a 1 s running median), the exposure bias where the
    camera brackets exposures (r at each exposure against the running median of all), scale error;
  * video and intensified camera: the scale transferred from the colour camera (median of
    r_colour_m / r_px over frames where both show a full rim within 0.15 s of each other), and the
    spread of that ratio, which is the radius agreement between the sources;
  * end times of every source against the sensor flame end (photomultiplier end, its tight variant,
    the steepest radiometer fall) and the flow change.

Writes <out>/compare.csv (one row per test and source), <out>/compare.json and an r(t) chart per
test, <out>/check/<test>_rt.png (all sources on the test clock, sensor end marked).

    python -I tools/wsp/media/compare_sources.py --out runs/dev/media/acme --raw runs/dev/media/acme/raw
"""
import argparse
import csv
import json
import math
import sys
from pathlib import Path

import cv2
import numpy as np

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "tools/wsp/ext"))
import flame_design as fd  # noqa: E402

NAN = float("nan")
FULL_RIM = ("few_rays", "partial_arc", "saturated", "no_rim", "lit_view", "low_snr")


def load(out, test, source):
    p = Path(out, "series", f"{test}_{source}.csv")
    if not p.exists():
        return None, None
    rows = list(csv.DictReader(p.open(encoding="utf-8")))
    meta = json.loads(Path(out, "series", f"{test}_{source}.json").read_text(encoding="utf-8"))

    def f(x):
        try:
            return float(x)
        except (TypeError, ValueError):
            return NAN
    for r in rows:
        for k in ("t_s", "r_m", "r_px", "asym", "exposure_s", "coverage", "n_rays", "signal_per_s"):
            r[k] = f(r.get(k))
    return rows, meta


def full(r):
    q = r.get("quality", "")
    return np.isfinite(r["r_px"]) and not any(b in q for b in FULL_RIM)


def running_median(t, y, w=1.0):
    """Local straight line through the other points within w/2 (the point itself left out, so a
    smooth monotonic series does not give zero scatter), robust: two passes, outliers of the first
    pass left out of the second."""
    out = np.full(len(t), NAN)
    for i in range(len(t)):
        m = (np.abs(t - t[i]) <= w / 2)
        m[i] = False
        if m.sum() < 3:
            m = np.abs(t - t[i]) <= w
            m[i] = False
        if m.sum() < 3:
            continue
        tt, yy = t[m], y[m]
        c = np.polyfit(tt - t[i], yy, 1)
        res = yy - np.polyval(c, tt - t[i])
        keep = np.abs(res) <= 3 * 1.4826 * np.median(np.abs(res)) + 1e-9
        if keep.sum() >= 3:
            c = np.polyfit(tt[keep] - t[i], yy[keep], 1)
        out[i] = c[1]
    return out


def sensor(test, raw):
    fd.FILES = Path(raw)
    fn = dict(data=f"{test}_data.csv", setpoints=f"{test}_mfcSetpoints.csv", tp=f"{test}_TP.txt", script=f"as run{test}.txt")
    fn = {k: (v if (Path(raw) / v).exists() else None) for k, v in fn.items()}
    return fd.measure(test, fn)


def chart(path, test, series, sens):
    W, H, L, B = 900, 420, 70, 40
    img = np.full((H, W, 3), 255, np.uint8)
    ts = [r["t_s"] for s in series.values() for r in s if np.isfinite(r.get("r_mm", NAN))]
    rs = [r["r_mm"] for s in series.values() for r in s if np.isfinite(r.get("r_mm", NAN))]
    if not ts:
        return
    t_hi = max(max(ts), sens.get("pmt_end_s", 0) or 0) + 2
    r_hi = float(np.percentile(rs, 99)) * 1.15 + 1
    X = lambda t: int(L + (t + 1) / (t_hi + 1) * (W - L - 20))  # noqa: E731
    Y = lambda r: int(H - B - r / r_hi * (H - B - 30))  # noqa: E731
    cv2.line(img, (L, H - B), (W - 20, H - B), (0, 0, 0), 1)
    cv2.line(img, (L, 30), (L, H - B), (0, 0, 0), 1)
    for k in range(0, int(r_hi) + 1, max(1, int(r_hi // 6))):
        cv2.putText(img, f"{k}", (L - 30, Y(k) + 4), 0, 0.4, (0, 0, 0), 1)
    step = max(1, int(t_hi // 10))
    for k in range(0, int(t_hi) + 1, step):
        cv2.putText(img, f"{k}", (X(k) - 6, H - B + 16), 0, 0.4, (0, 0, 0), 1)
    cols = {"colour": (0, 0, 220), "video": (0, 150, 0), "intns": (200, 0, 0)}
    for src, s in series.items():
        for r in s:
            if np.isfinite(r.get("r_mm", NAN)):
                cv2.circle(img, (X(r["t_s"]), Y(r["r_mm"])), 1 if src != "colour" else 2, cols[src], -1)
    for name, key, col in (("PMT end", "pmt_end_s", (0, 0, 0)), ("radiometer fall", "radiometer_steepest_fall_s", (120, 120, 120))):
        v = sens.get(key, NAN)
        if v == v:
            cv2.line(img, (X(v), 30), (X(v), H - B), col, 1)
            cv2.putText(img, name, (X(v) + 3, 45 if key == "pmt_end_s" else 60), 0, 0.4, col, 1)
    y = 18
    cv2.putText(img, f"{test}  flame radius (mm) vs time since sensor onset (s)", (L, y), 0, 0.5, (0, 0, 0), 1)
    for i, (src, col) in enumerate(cols.items()):
        if src in series:
            cv2.putText(img, src, (W - 220 + 70 * i, y), 0, 0.45, col, 1)
    cv2.imwrite(str(path), img)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--raw", required=True)
    a = ap.parse_args()
    tests = sorted({p.stem.rsplit("_", 1)[0] for p in Path(a.out, "series").glob("*_colour.csv")})
    table, report = [], {}
    for test in tests:
        sens = sensor(test, a.raw)
        col, cmeta = load(a.out, test, "colour")
        rep = dict(sensor=dict((k, sens.get(k)) for k in ("pmt_end_s", "pmt_end_tight_s", "radiometer_steepest_fall_s",
                                                         "flow_change_s", "label_sensor", "tp_class", "end_s")))
        series = {}
        # ---- colour ---------------------------------------------------------------------------------
        ok = [r for r in col if full(r) and r["t_s"] > 0.5]
        t = np.array([r["t_s"] for r in ok])
        rm = np.array([r["r_m"] for r in ok]) * 1e3
        c = dict(n_frames=cmeta["n_frames"], n_full_rim=len(ok), mm_per_px=cmeta.get("mm_per_px_full"),
                 scale_source=cmeta.get("scale_source"))
        if len(ok) > 5:
            med = running_median(t, rm, 1.0)
            resid = rm - med
            ok_r = np.isfinite(resid)
            resid, med = np.where(ok_r, resid, 0.0), np.where(ok_r, med, np.nanmedian(med))
            c["r_noise_mm"] = float(1.4826 * np.median(np.abs(resid)))
            c["r_noise_rel"] = float(np.median(np.abs(resid) / med) * 1.4826)
            exps = sorted({r["exposure_s"] for r in ok})
            if len(exps) > 1:
                c["exposure_bias_mm"] = {f"{e:g}s": float(np.median(resid[np.array([r["exposure_s"] for r in ok]) == e])) for e in exps}
            c["asym_median"] = float(np.median([r["asym"] for r in ok]))
            c["r_first_mm"], c["r_last_mm"], c["r_max_mm"] = float(med[0]), float(med[-1]), float(np.max(med))
        for k in ("image_end_s", "light_end_s", "end_s", "end_err_s", "end_from", "image_onset_s", "light_onset_s", "frame_interval_median_s",
                  "compute_s", "file_bytes", "image_end_note", "light_end_note", "image_last_frame_s"):
            if k in cmeta:
                c[k] = cmeta[k]
        # clock check: the brightest frame within 2 s of the sensor onset is the ignition flash
        win = [r for r in col if -1.0 < r["t_s"] < 2.0 and np.isfinite(r["signal_per_s"])]
        if win:
            fl = max(win, key=lambda r: r["signal_per_s"])
            c["flash_frame_t_s"], c["flash_frame_exposure_s"] = fl["t_s"], fl["exposure_s"]
        rep["colour"] = c
        series["colour"] = [dict(t_s=r["t_s"], r_mm=r["r_m"] * 1e3) for r in ok]
        # ---- video, intensified ------------------------------------------------------------------------
        for src in ("video", "intns"):
            rows, meta = load(a.out, test, src)
            if rows is None:
                continue
            okv = [r for r in rows if full(r) and r["t_s"] > 0.5]
            d = dict(n_frames=meta["n_frames"], n_full_rim=len(okv))
            for k in ("align_lag_s", "align_corr", "align_second_corr", "align_peak_width_s", "image_end_s", "light_end_s",
                      "end_s", "end_err_s", "end_from", "image_onset_s", "frame_interval_median_s", "compute_s", "file_bytes",
                      "time_alignment"):
                if k in meta:
                    d[k] = meta[k]
            # transferred scale: pairs within 0.15 s
            ratios, pair_t = [], []
            if len(ok) and okv:
                for r in okv:
                    j = int(np.argmin(np.abs(t - r["t_s"])))
                    if abs(t[j] - r["t_s"]) <= 0.15:
                        ratios.append(rm[j] / r["r_px"])
                        pair_t.append(r["t_s"])
            if len(ratios) >= 5:
                k_mm = float(np.median(ratios))
                d.update(scale_mm_per_px=k_mm, scale_pairs=len(ratios),
                         ratio_spread_rel=float(1.4826 * np.median(np.abs(np.array(ratios) / k_mm - 1))),
                         ratio_trend_rel=float(np.polyfit(pair_t, np.array(ratios) / k_mm, 1)[0] * (max(pair_t) - min(pair_t)))
                         if max(pair_t) > min(pair_t) else NAN)
                tv = np.array([r["t_s"] for r in okv])
                rv = np.array([r["r_px"] for r in okv]) * k_mm
                if len(okv) > 5:
                    medv = running_median(tv, rv, 1.0)
                    d["r_noise_mm"] = float(1.4826 * np.nanmedian(np.abs(rv - medv)))
                series[src] = [dict(t_s=x, r_mm=y) for x, y in zip(tv, rv)]
            else:
                d["scale_note"] = f"{len(ratios)} matched full-rim pairs with the colour camera: no transferred scale"
            rep[src] = d
        report[test] = rep
        chart(Path(a.out, "check", f"{test}_rt.png"), test, series, sens)
        for src in ("colour", "video", "intns"):
            if src not in rep:
                continue
            d = rep[src]
            table.append(dict(test=test, source=src, n_frames=d.get("n_frames"), n_full_rim=d.get("n_full_rim"),
                              mm_per_px=d.get("mm_per_px", d.get("scale_mm_per_px")), r_noise_mm=d.get("r_noise_mm"),
                              ratio_spread_rel=d.get("ratio_spread_rel"), end_s=d.get("end_s"), end_err_s=d.get("end_err_s"),
                              end_from=d.get("end_from"), rim_end_s=d.get("image_end_s"), light_end_s=d.get("light_end_s"),
                              sensor_pmt_end_s=sens.get("pmt_end_s"), sensor_rad_fall_s=sens.get("radiometer_steepest_fall_s"),
                              end_minus_pmt_s=(d.get("end_s", NAN) or NAN) - sens.get("pmt_end_s", NAN),
                              end_minus_radfall_s=(d.get("end_s", NAN) or NAN) - sens.get("radiometer_steepest_fall_s", NAN),
                              flow_change_s=sens.get("flow_change_s"), sensor_label=sens.get("label_sensor"),
                              frame_interval_s=d.get("frame_interval_median_s"), file_gb=(d.get("file_bytes") or NAN) / 1e9,
                              compute_s=d.get("compute_s")))
    keys = list(table[0].keys())
    with open(Path(a.out, "compare.csv"), "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=keys)
        w.writeheader()
        for r in table:
            w.writerow({k: (f"{v:.4g}" if isinstance(v, float) else v) for k, v in r.items()})
    Path(a.out, "compare.json").write_text(json.dumps(report, indent=1, default=lambda x: None if x != x else float(x)), encoding="utf-8")
    for r in table:
        print({k: (round(v, 3) if isinstance(v, float) else v) for k, v in r.items()})


if __name__ == "__main__":
    sys.exit(main())

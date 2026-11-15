"""Compare droplet_diameter.py output with NASA's processed FLEX DropletData (TRAIN tests only).

    python -I tools/wsp/media/compare_nasa.py <tool.csv> <FLEX-nnn_DropletData.txt> [--plot out.png] [--json out.json]

Reports: time offset (tool clock minus NASA clock, best match of D(t)), bias and RMS of D in micrometres
on matched frames, ratio of burning-rate constants (slope of D^2 against t, same window for both), share of
frames flagged bad inside NASA's measured span, and the diameter ratio tool/NASA.
"""
import argparse
import json
import sys

import numpy as np

GOOD = {"ok", "soot"}


def load_tool(path):
    t, d, flag = [], [], []
    with open(path, encoding="utf-8") as f:
        next(f)
        for line in f:
            p = line.rstrip("\n").split(",")
            t.append(float(p[1]) if p[1] else np.nan)
            d.append(float(p[2]) * 1e3 if p[2] else np.nan)  # mm
            flag.append(p[5])
    return np.array(t), np.array(d), np.array(flag)


def load_nasa(path):
    txt = open(path, encoding="utf-8", errors="replace").read().replace("\r\n", "\n").replace("\r", "\n")
    rows = []
    for line in txt.split("\n")[1:]:
        p = line.split("\t")
        try:
            rows.append((float(p[0]), float(p[1])))
        except (ValueError, IndexError):
            pass
    a = np.array(rows)
    return a[:, 0], a[:, 1]


def match(tt, td, good, nt, shift, tol=0.51 / 30):
    """For each NASA time, the tool frame at nt + shift (nearest, within tol). Returns index or -1."""
    order = np.argsort(tt)
    ts = tt[order]
    q = nt + shift
    j = np.clip(np.searchsorted(ts, q), 1, len(ts) - 1)
    left = ts[j - 1]
    right = ts[j]
    jj = np.where(np.abs(q - left) <= np.abs(right - q), j - 1, j)
    ok = np.abs(ts[jj] - q) <= tol
    idx = np.where(ok, order[jj], -1)
    idx = np.where((idx >= 0) & good[np.maximum(idx, 0)], idx, -1)
    return idx


def kfit(t, d2):
    if len(t) < 5:
        return np.nan
    A = np.vstack([t, np.ones_like(t)]).T
    s, _ = np.linalg.lstsq(A, d2, rcond=None)[0]
    return -s


def compare(tool_csv, nasa_txt):
    tt, td, fl = load_tool(tool_csv)
    nt, nd = load_nasa(nasa_txt)
    good = np.isin(fl, list(GOOD)) & np.isfinite(td) & np.isfinite(tt)
    # offset search: tool time = NASA time + shift; robust (median abs) on the burning part
    # Search +-3 s (the camera clock agrees with the rack stamps to ~0.1 s, see clock_check); only
    # shifts that keep >= 90 % of the best overlap compete, so a shift cannot win by dropping frames.
    scan = []
    for shift in np.arange(-3.0, 3.0001, 0.001):
        idx = match(tt, td, good, nt, shift)
        m = idx >= 0
        if m.sum() < 10:
            continue
        scan.append((float(np.median(np.abs(td[idx[m]] - nd[m]))), float(shift), int(m.sum())))
    best = None
    if scan:
        nmax = max(s[2] for s in scan)
        ok = [s for s in scan if s[2] >= 0.9 * nmax]
        best = min(ok)
        near = [s[1] for s in ok if s[0] <= 1.1 * best[0]]
        offset_range = [min(near), max(near)]
    shift = best[1] if best else 0.0
    # a nearly flat D(t) (no ignition, slow evaporation) does not fix the offset: then use the rack clock
    offset_determined = bool(best) and (offset_range[1] - offset_range[0]) <= 0.5 and abs(best[1]) < 2.95
    if not offset_determined:
        shift = 0.0
    idx = match(tt, td, good, nt, shift)
    m = idx >= 0
    diff_um = (td[idx[m]] - nd[m]) * 1e3
    idx0 = match(tt, td, good, nt, 0.0)
    m0 = idx0 >= 0
    diff0 = (td[idx0[m0]] - nd[m0]) * 1e3
    # burning-rate constants over the same window: NASA span from t=1.0 s to its last sample
    w0, w1 = 1.0, nt.max()
    wn = (nt >= w0) & (nt <= w1)
    kn = kfit(nt[wn], nd[wn] ** 2)
    wt = good & (tt - shift >= w0) & (tt - shift <= w1)
    kt = kfit(tt[wt] - shift, td[wt] ** 2)
    span = np.isfinite(tt) & (tt - shift >= nt.min()) & (tt - shift <= nt.max())
    bad_share = float(np.mean(~good[span])) if span.any() else np.nan
    flags = {}
    for f in fl[span]:
        flags[f] = flags.get(f, 0) + 1
    late = nt[m] >= 1.0
    dl = diff_um[late]
    mad = float(1.4826 * np.median(np.abs(diff_um - np.median(diff_um))))
    res = dict(n_nasa=int(len(nt)), n_matched=int(m.sum()), coverage=float(m.mean()),
               time_offset_s=float(shift), time_offset_range_s=offset_range if best else None,
               time_offset_determined=offset_determined,
               best_offset_scan_s=float(best[1]) if best else None, offset_search_median_abs_um=float(best[0] * 1e3) if best else None,
               bias_um=float(np.mean(diff_um)), median_diff_um=float(np.median(diff_um)),
               rms_um=float(np.sqrt(np.mean(diff_um ** 2))),
               sd_um=float(np.std(diff_um)), robust_sd_um=mad,
               n_matched_after_1s=int(late.sum()), bias_um_after_1s=float(np.mean(dl)) if dl.size else None,
               rms_um_after_1s=float(np.sqrt(np.mean(dl ** 2))) if dl.size else None,
               median_diff_um_after_1s=float(np.median(dl)) if dl.size else None, p95_abs_um=float(np.percentile(np.abs(diff_um), 95)),
               bias_um_no_shift=float(np.mean(diff0)) if m0.any() else None,
               rms_um_no_shift=float(np.sqrt(np.mean(diff0 ** 2))) if m0.any() else None,
               ratio_d_median=float(np.median(td[idx[m]] / nd[m])),
               k_window_s=[w0, float(w1)], k_nasa_mm2_s=float(kn), k_tool_mm2_s=float(kt),
               k_ratio=float(kt / kn) if kn else None, bad_share_in_nasa_span=bad_share, flags_in_span=flags,
               nasa_t_range=[float(nt.min()), float(nt.max())], nasa_d0_mm=float(nd[0]))
    return res, (tt, td, fl, good, nt, nd, shift, idx)


def plot(data, out, title):
    import cv2
    tt, td, fl, good, nt, nd, shift, idx = data
    W, H, L, B = 900, 520, 60, 40
    img = np.full((H, W, 3), 255, np.uint8)
    t0, t1 = min(nt.min(), np.nanmin(tt - shift)) - 0.2, max(nt.max(), np.nanmax(tt - shift)) + 0.2
    t0 = max(t0, nt.min() - 2)
    t1 = min(t1, nt.max() + 2)
    dmax = max(np.nanmax(nd), np.nanmax(np.where(good, td, np.nan))) * 1.1

    def px(t, d):
        return int(L + (t - t0) / (t1 - t0) * (W - L - 10)), int(H - B - d / dmax * (H - B - 20))
    cv2.rectangle(img, (L, 20), (W - 10, H - B), (0, 0, 0), 1)
    for t in np.arange(np.ceil(t0), t1, 1 if t1 - t0 < 15 else 5):
        x, _ = px(t, 0)
        cv2.line(img, (x, H - B), (x, H - B + 4), (0, 0, 0), 1)
        cv2.putText(img, f"{t:.0f}", (x - 6, H - B + 18), cv2.FONT_HERSHEY_SIMPLEX, 0.4, (0, 0, 0), 1)
    for d in np.arange(0, dmax, 0.5):
        _, y = px(t0, d)
        cv2.putText(img, f"{d:.1f}", (10, y + 4), cv2.FONT_HERSHEY_SIMPLEX, 0.4, (0, 0, 0), 1)
    for t, d in zip(nt, nd):
        cv2.circle(img, px(t, d), 2, (0, 0, 0), -1)
    for t, d, g in zip(tt, td, good):
        if np.isfinite(t) and np.isfinite(d) and t0 <= t - shift <= t1:
            cv2.circle(img, px(t - shift, d), 1, (0, 160, 0) if g else (0, 0, 255), -1)
    cv2.putText(img, title + "  black NASA, green tool ok, red tool flagged; D (mm) vs t (s)", (L, 14),
                cv2.FONT_HERSHEY_SIMPLEX, 0.42, (0, 0, 0), 1)
    cv2.imwrite(out, img)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("tool_csv")
    ap.add_argument("nasa_txt")
    ap.add_argument("--plot")
    ap.add_argument("--json")
    ap.add_argument("--title", default="")
    a = ap.parse_args()
    res, data = compare(a.tool_csv, a.nasa_txt)
    if a.plot:
        plot(data, a.plot, a.title)
    if a.json:
        open(a.json, "w", encoding="utf-8").write(json.dumps(res, indent=2) + "\n")
    print(json.dumps(res, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())

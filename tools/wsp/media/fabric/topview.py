#!/usr/bin/env python3
"""Top-view (still camera) front track of a BASS-II SIBAL burn, in sample widths, from NASA's proof sheet.

Written 15 Nov 2026 for the frozen batch docs/wsp/atlas/splits/BASS-batch-fresh.json (PSI-25 T13, T20).
It is the smallest addition that gives a per-burn band width: the fabric measurer (measure.py) works in
the duct view only, where the 1-cm samples fail its QC rule R6 (its width map was set on 2-cm holders).
Nothing here changes measure.py, qc.py or enrich.py; this script reads the measurer's own track of the
same burn for the time origin (igniter off), the igniter-on time, the overlay clock and the flow history.

The image method is the frozen train method of docs/wsp/feedback/bass_train_measurement.md section 4 and
the notes of tools/wsp/bass_optical/tracks/train/*.json, re-implemented from that text (the train scripts
were never committed). Every constant below is copied from that text; none was chosen on T13 or T20.

  thumbnails  each proof-sheet page is rendered at 300 dpi; each still is the union of its image strips,
              cropped and resampled to the native 324 x 216 px (as the train thumbnails). The camera stamp
              (whole seconds, "Date/Time set to BASS Video Display") is the hh:mm:ss under the thumbnail.
  sequence    stills more than GAP_S after the previous one are post-test stills (state posttest, not measured).
  lit         'igniter onset': first still whose hottest 8x8 block of red (right quarter of the frame)
              exceeds the pre-onset level by GLOW; 'window lit': frame-mean red above the sequence minimum
              by more than LIT x the range of frame-mean red over the sequence. Lit stills are state igniter.
  times       local cadence reconstruction of the whole-second stamps (+-8 neighbouring stamps, their
              first-to-last cadence; a still is placed at the middle of the intersection of its neighbours'
              whole-second intervals, else at stamp + 0.5 s). Camera minus overlay clock from the brackets:
              onset between the last unlit and first glowing still (overlay: igniter on), light gone between
              the last lit and first unlit still (overlay: igniter off); intersection of the two brackets, or
              the off bracket alone when the video starts with the igniter already on. t_s = overlay seconds
              after igniter off (the measurer's time origin).
  band        band centre row = row of the largest red mean over the middle of the frame in the lit stills;
              band width = full width at half maximum of the green channel at 9 columns of every lit still,
              median (pixels of the 324-px thumbnail); x_up = right (igniter-side) half-maximum edge of the
              band and coil in the green channel of the dimmest lit still; window end = left half-rise (steepest
              rise, half-way between the levels 8 px either side) of the red channel along the band, median over the 3 brightest lit stills (train: "median of 3 igniter-lit
              stills"); the 9 columns of each lit still lie between x_up and its own lit end (where its red falls below half
              the mid-band level: the igniter light spreads from the coil).
  flame       per-pixel median of the unlit stills of the sequence as background; difference, 3 x 3 median
              filter, blue above background by BLUE_T; largest 8-connected region of at least MIN_PX pixels,
              3-px border masked. tip_w = (x_up - x_left)/band, base_w = (x_up - x_right)/band.
  quality     edge = region touches the masked border, base within 1 px of x_up, or tip within 1 px of the
              window end; weak = area < 200 px or blue sum < 12000; else ok.
  end         last flame still followed by a still without flame: extinction at the midpoint; flame in the
              last still of the sequence: record_end at that still.

    python -I tools/wsp/media/fabric/topview.py --proof SHEET.pdf --duct-track TRACK.json --run-id PSI-25:T20 \
        --partition validation --out OUT.json [--check OUT.png]
"""
import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

import cv2
import fitz
import numpy as np

METHOD_VERSION = "fabric-topview/1"
W, H = 324, 216
DPI = 300
GAP_S = 20.0          # a later still after a gap this long is post-test
GLOW = 60.0           # red, 8x8 block, over the pre-onset level (the measurer's IGN_ON is 60 b-units per 16-px block)
LIT = 0.2             # frame-mean red over the sequence minimum, as a fraction of the brightest still's: the whole
                      # window lit by the igniter (a large flame lights the window too: T5 at FAN 61 reaches 0.14)
BLUE_T = 40.0         # train: blue 40 above background
MIN_PX = 12           # train: largest region >= 12 px
BORDER = 3            # train: 3 px border mask
WEAK_AREA, WEAK_SUM = 200, 12000   # train quality rule
NEIGH = 8             # train: +-8 neighbouring stamps
N_COLS = 9            # train: 9 columns


def sha256(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def thumbnails(pdf):
    out = []
    doc = fitz.open(pdf)
    for page in doc:
        M = page.rotation_matrix
        pix = page.get_pixmap(matrix=fitz.Matrix(DPI / 72, DPI / 72))
        img = np.frombuffer(pix.samples, np.uint8).reshape(pix.h, pix.w, pix.n)[:, :, :3][:, :, ::-1].copy()
        groups = {}
        for i in page.get_image_info():
            r = fitz.Rect(i["bbox"]) * M
            groups.setdefault((round(r.x0), round(r.x1)), []).append(r)
        cells = []
        for rs in groups.values():
            rs.sort(key=lambda r: r.y0)
            cur = None
            for r in rs:
                if cur is not None and abs(r.y0 - cur.y1) < 2:
                    cur = cur | r
                else:
                    if cur is not None:
                        cells.append(cur)
                    cur = fitz.Rect(r)
            cells.append(cur)
        words = [(fitz.Rect(w[:4]) * M, w[4]) for w in page.get_text("words")]
        s = DPI / 72
        for c in cells:
            if c.width < 100 or c.height < 60:
                continue
            lab = [(r, t) for r, t in words if c.x0 - 2 <= r.x0 <= c.x1 and 0 <= r.y0 - c.y1 <= 8]
            sid = next((t for _, t in lab if re.fullmatch(r"iss\d{3}e\d{6}", t)), None)
            clk = next((t for _, t in lab if re.fullmatch(r"\d\d:\d\d:\d\d", t)), None)
            if sid is None or clk is None:
                continue
            crop = img[int(round(c.y0 * s)):int(round(c.y1 * s)), int(round(c.x0 * s)):int(round(c.x1 * s))]
            hh, mm, ss = (int(v) for v in clk.split(":"))
            out.append(dict(id=sid, stamp=hh * 3600 + mm * 60 + ss, stamp_text=clk,
                            img=cv2.resize(crop, (W, H), interpolation=cv2.INTER_AREA).astype(np.float32)))
    out.sort(key=lambda t: t["id"])
    return out


def cadence_times(stamps):
    """Local cadence reconstruction of whole-second stamps."""
    n = len(stamps)
    out, placed, widths = [], 0, []
    for i in range(n):
        js = list(range(max(0, i - NEIGH), min(n, i + NEIGH + 1)))
        if len(js) < 2 or stamps[js[-1]] == stamps[js[0]]:
            out.append(stamps[i] + 0.5)
            continue
        k = (stamps[js[-1]] - stamps[js[0]]) / (js[-1] - js[0])
        lo, hi = -1e18, 1e18
        for j in js:
            lo = max(lo, stamps[j] - (j - i) * k)
            hi = min(hi, stamps[j] + 1 - (j - i) * k)
        if hi > lo:
            out.append((lo + hi) / 2)
            placed += 1
            widths.append(hi - lo)
        else:
            out.append(stamps[i] + 0.5)
    return out, placed, (float(np.median(widths)) if widths else None)


def half_cross(prof, start, step, half):
    """First crossing below `half` walking from index start by step; linear interpolation (index units)."""
    i = start
    while 0 <= i + step < len(prof):
        if prof[i + step] < half <= prof[i]:
            return float(i + step * (prof[i] - half) / (prof[i] - prof[i + step]))
        i += step
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--proof", required=True)
    ap.add_argument("--duct-track", required=True, help="the measurer's track of the same burn (after qc.py and enrich.py)")
    ap.add_argument("--run-id", required=True)
    ap.add_argument("--partition", required=True, choices=["train", "validation"])
    ap.add_argument("--out", required=True)
    ap.add_argument("--check", default=None)
    a = ap.parse_args()
    D = json.loads(Path(a.duct_track).read_text(encoding="utf-8"))
    th = thumbnails(a.proof)
    if not th:
        sys.exit("no thumbnails found")
    # sequence and post-test
    seq = [th[0]]
    post = []
    for t in th[1:]:
        if post or t["stamp"] - seq[-1]["stamp"] > GAP_S:
            post.append(t)
        else:
            seq.append(t)
    imgs = np.stack([t["img"] for t in seq])           # BGR float
    red = imgs[..., 2]
    mean_r = red.reshape(len(seq), -1).mean(1)
    lit = (mean_r - mean_r.min()) > LIT * (mean_r.max() - mean_r.min())
    # coil glow: hottest 8x8 block of red in the right quarter, over its pre-onset level
    blk = np.stack([cv2.resize(r[:, 3 * W // 4:], (W // 4 // 8, H // 8), interpolation=cv2.INTER_AREA) for r in red])
    first_lit = int(np.argmax(lit)) if lit.any() else None
    if first_lit is None:
        sys.exit("no igniter-lit still: no band, no origin")
    pre_level = np.median(blk[:max(1, first_lit)], axis=0) if first_lit > 0 else np.zeros_like(blk[0])
    glow = (blk - pre_level[None]).reshape(len(seq), -1).max(1) > GLOW
    onset = int(np.argmax(glow[:first_lit + 1]))
    lit_idx = np.nonzero(lit)[0]
    last_lit = int(lit_idx[-1])
    if not (lit[first_lit:last_lit + 1].all()):
        sys.exit("igniter-lit stills are not one block")
    times, placed, med_w = cadence_times([t["stamp"] for t in seq])
    # clock offset (camera minus overlay) from the brackets
    ov = D.get("overlay") or {}
    c0 = ov.get("clock_at_t0")
    if c0 is None:
        sys.exit("the measurer's track has no overlay clock: no time origin for the stills")
    hh, mm, ss = c0.split(":")
    off_ovl = int(hh) * 3600 + int(mm) * 60 + float(ss)
    on_ovl = off_ovl + D["igniter"]["on_t_s"]
    on_video = D["igniter"]["off_video_s"] + D["igniter"]["on_t_s"]
    brackets = {}
    if last_lit + 1 < len(seq):
        brackets["off"] = [times[last_lit] - off_ovl, times[last_lit + 1] - off_ovl]
    if onset > 0 and on_video > 0.5:
        brackets["onset"] = [times[onset - 1] - on_ovl, times[onset] - on_ovl]
    if "off" not in brackets:
        sys.exit("the igniter light does not go out within the still sequence")
    lo = max(b[0] for b in brackets.values())
    hi = min(b[1] for b in brackets.values())
    if hi >= lo:
        offset, unc, how = (lo + hi) / 2, (hi - lo) / 2, "intersection of the brackets"
    else:
        mids = [(b[0] + b[1]) / 2 for b in brackets.values()]
        offset, unc, how = float(np.mean(mids)), (max(mids) - min(mids)) / 2 + 0.5, "brackets disjoint: mean of their midpoints"
    t_s = [tt - offset - off_ovl for tt in times]
    # band geometry from the lit stills
    L = imgs[lit_idx]
    mid = slice(int(0.3 * W), int(0.7 * W))
    rowprof = L[..., 2][:, :, mid].mean(axis=(0, 2))
    r0, r1 = int(0.3 * H), int(0.8 * H)
    yc = r0 + int(np.argmax(rowprof[r0:r1]))
    dim = int(lit_idx[int(np.argmin(mean_r[lit_idx]))])
    g = imgs[dim][..., 1][yc - 3:yc + 4].mean(0)
    pk = int(np.argmax(g[W // 2:])) + W // 2
    base = float(g[pk:].min())
    x_up = half_cross(g, pk, +1, (g[pk] + base) / 2)
    ends, lit_end = {}, {}
    for k in lit_idx:
        rr = imgs[k][..., 2][yc - 3:yc + 4].mean(0)
        # left (downstream) end: half-rise of the steepest rise in the left half of the band (the igniter
        # light falls off along the band, so a level taken mid-band would put the end inside it)
        grad = np.array([rr[i + 2] - rr[i - 2] for i in range(10, W // 2)])
        i0 = 10 + int(np.argmax(grad))
        lo_, hi_ = float(rr[max(0, i0 - 8):i0].min()), float(rr[i0:i0 + 8].max())
        if hi_ - lo_ >= 10:
            e = half_cross(rr, min(W - 1, i0 + 8), -1, (lo_ + hi_) / 2)
            if e is not None:
                ends[int(k)] = e
        # lit extent of this still (where its band is lit at least half the mid-band level): the columns
        # of the band-width profiles stay inside it
        plateau = float(np.median(rr[mid]))
        b0 = float(rr[:int(0.3 * W)].min())
        e2 = half_cross(rr, int(0.5 * W), -1, (plateau + b0) / 2)
        if e2 is not None:
            lit_end[int(k)] = e2
    bright3 = sorted(ends, key=lambda k: -mean_r[k])[:3]
    x_end = float(np.median([ends[k] for k in bright3])) if bright3 else None
    if x_up is None or x_end is None:
        sys.exit("band ends not found")
    widths = []
    for k in lit_idx:
        if int(k) not in lit_end:
            continue
        e_k = lit_end[int(k)]   # columns only where this still's band is lit (the igniter light spreads from the coil)
        gg = imgs[k][..., 1]
        for x in np.linspace(e_k + 0.1 * (x_up - e_k), x_up - 0.25 * (x_up - e_k), N_COLS):
            xi = int(round(x))
            v = gg[:, xi - 1:xi + 2].mean(1)
            lo_y, hi_y = max(0, yc - 40), min(H, yc + 41)
            p = max(0, yc - 15) + int(np.argmax(v[max(0, yc - 15):min(H, yc + 16)]))
            b = float(np.percentile(v[lo_y:hi_y], 10))
            half = (v[p] + b) / 2
            y_lo = half_cross(v, p, -1, half)
            y_hi = half_cross(v, p, +1, half)
            if y_lo is not None and y_hi is not None:
                widths.append(y_hi - y_lo)
    band = float(np.median(widths))
    # flame regions
    unlit = [i for i in range(len(seq)) if not lit[i]]
    bg = np.median(imgs[unlit], axis=0)
    mask = np.zeros((H, W), bool)
    mask[BORDER:H - BORDER, BORDER:W - BORDER] = True
    stills = []
    for i, t in enumerate(seq):
        diff = imgs[i] - bg
        blue = cv2.medianBlur(np.clip(diff[..., 0], -255, 255).astype(np.float32), 3)
        lum = ((blue > BLUE_T) & mask).astype(np.uint8)
        n, lab_, stats, _ = cv2.connectedComponentsWithStats(lum, connectivity=8)
        e = {"t_s": round(t_s[i], 2), "tip_w": None, "base_w": None, "state": None, "quality": None,
             "still": t["id"], "stamp": t["stamp_text"]}
        reg = None
        if n > 1:
            k = 1 + int(np.argmax(stats[1:, cv2.CC_STAT_AREA]))
            if stats[k, cv2.CC_STAT_AREA] >= MIN_PX:
                reg = k
        if reg is not None:
            xl = stats[reg, cv2.CC_STAT_LEFT]
            xr = xl + stats[reg, cv2.CC_STAT_WIDTH] - 1
            yt = stats[reg, cv2.CC_STAT_TOP]
            yb = yt + stats[reg, cv2.CC_STAT_HEIGHT] - 1
            area = int(stats[reg, cv2.CC_STAT_AREA])
            bsum = float(blue[lab_ == reg].sum())
            touches = xl <= BORDER or yt <= BORDER or xr >= W - 1 - BORDER or yb >= H - 1 - BORDER
            q = ("edge" if (touches or abs(xr - x_up) <= 1 or abs(xl - x_end) <= 1) else
                 "weak" if (area < WEAK_AREA or bsum < WEAK_SUM) else "ok")
            e.update(tip_w=round((x_up - xl) / band, 3), base_w=round((x_up - xr) / band, 3), quality=q,
                     area_px=area, blue_sum=round(bsum), x_left_px=int(xl), x_right_px=int(xr))
        if i < onset:
            e["state"] = "pre"
        elif lit[i] or i < first_lit:
            e["state"] = "igniter"
        elif reg is not None:
            e["state"] = "flame"
        else:
            e["state"] = "none"
        if e["state"] not in ("flame", "igniter"):
            for k_ in ("tip_w", "base_w", "quality"):
                e[k_] = None
        stills.append(e)
    for t in post:
        stills.append({"t_s": round(t["stamp"] + 0.5 - offset - off_ovl, 2), "tip_w": None, "base_w": None,
                       "state": "posttest", "quality": None, "still": t["id"], "stamp": t["stamp_text"]})
    fl = [s for s in stills if s["state"] == "flame"]
    insq = [s for s in stills if s["state"] in ("flame", "none")]
    if not fl:
        end = {"kind": "no_spread", "t_s": None}
    else:
        last = fl[-1]
        after = [s for s in insq if s["t_s"] > last["t_s"]]
        end = ({"kind": "extinction", "t_s": round((last["t_s"] + after[0]["t_s"]) / 2, 2)} if after else
               {"kind": "record_end", "t_s": last["t_s"]})
    c = D.get("conditions") or {}
    fan = [{"t_s": f["t_s"], "fan": f["fan"], "u_cm_s": f.get("u_cm_s")} for f in c.get("flow", []) if f.get("fan") is not None]
    sd = D.get("spread_direction") or ""
    flow_dir = "concurrent" if "smaller x" in sd else ("opposed" if "larger x" in sd else None)
    material = c.get("material") or D.get("material")
    mi = 0 if "sibal" in (material or "").lower() else None
    track = {
        "schema": "wsp-bass-front-track/1",
        "run_id": a.run_id,
        "partition": a.partition,
        "time_origin": "igniter off of the measurer's duct track (video time); t_s = overlay-clock seconds after it",
        "position_unit": "sample_width",
        "position_origin": "upstream (igniter-side) end of the lit sample band in the top view; positive downstream",
        "conditions": {"x_o2_initial": c.get("x_o2_initial"), "x_o2_final": c.get("x_o2_final"), "sample_label": material,
                       "flow": flow_dir, "pressure_atm": c.get("pressure_atm"), "restrictor": c.get("restrictor"),
                       "gmt_day": c.get("gmt_day"), "date": c.get("date"), "sample": c.get("sample"),
                       **({"material_index": mi} if mi is not None else {}),
                       "source": {"table": "enrich.py conditions of the measurer's track (NASA experiment table)",
                                  "flow": "the measurer's igniter position: igniter at the right = concurrent (enrich.py rule, without its quality gate: the top view replaces the duct widths that QC refused)"}},
        "geometry": {"band_width_px": round(band, 2), "window_length_w": round((x_up - x_end) / band, 3),
                     "source_view": "top_proof_sheet", "x_up_px": round(x_up, 1), "window_end_px": round(x_end, 1),
                     "band_centre_row_px": yc, "band_profiles": len(widths),
                     "band_width_p10_p90_px": [round(float(np.percentile(widths, 10)), 2), round(float(np.percentile(widths, 90)), 2)]},
        "igniter": {"on_t_s": D["igniter"]["on_t_s"], "off_t_s": 0.0},
        "fan": fan,
        "fan_calibration": c.get("fan_calibration") or [],
        "stills": stills,
        "end": end,
        "clock": {"still_minus_overlay_s": round(offset, 2), "uncertainty_s": round(unc, 2), "how": how,
                  "brackets": {k: [round(v, 2) for v in b] for k, b in brackets.items()},
                  "stills_placed_by_cadence": placed, "stills_in_sequence": len(seq), "median_interval_s": med_w},
        "measurement": {"method_version": METHOD_VERSION, "code_sha256": sha256(__file__), "view": "top view, NASA proof sheet",
                        "measured_blind": True},
        "source": {"proof_sheet": Path(a.proof).name, "proof_sha256": sha256(a.proof),
                   "duct_track": Path(a.duct_track).as_posix(), "duct_track_sha256": sha256(a.duct_track),
                   "duct_track_quality": D.get("quality"), "duct_track_qc_flags": D.get("qc_flags")},
        "notes": ["fan: the measurer's overlay FAN readings (1 per second) and u_cm_s from enrich.py; the scorer holds the last reading after the video ends",
                  "duct track used for: igniter on/off (time origin), overlay clock, flow history, conditions; not for positions"],
    }
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(track, indent=1), encoding="utf-8")
    if a.check:
        # first lit still and the first six flame stills with the geometry drawn (no late still is drawn)
        pick = [dim] + [i for i, s in enumerate(stills[:len(seq)]) if s["state"] == "flame"][:5]
        pan = []
        for i in pick:
            im = np.clip(imgs[i] * 3, 0, 255).astype(np.uint8)
            for x, col in ((x_up, (255, 255, 0)), (x_end, (0, 255, 255))):
                cv2.line(im, (int(round(x)), 0), (int(round(x)), 12), col, 1)
            cv2.line(im, (0, int(yc - band / 2)), (8, int(yc - band / 2)), (0, 255, 0), 1)
            cv2.line(im, (0, int(yc + band / 2)), (8, int(yc + band / 2)), (0, 255, 0), 1)
            s = stills[i]
            if s.get("x_left_px") is not None:
                cv2.line(im, (s["x_left_px"], yc - 30), (s["x_left_px"], yc + 30), (0, 255, 0), 1)
            cv2.putText(im, f"t={s['t_s']:+.1f} {s['state']}", (4, H - 6), cv2.FONT_HERSHEY_SIMPLEX, 0.4, (255, 255, 255), 1)
            pan.append(im)
        while len(pan) % 3:
            pan.append(np.zeros_like(pan[0]))
        grid = np.vstack([np.hstack(pan[r:r + 3]) for r in range(0, len(pan), 3)])
        cv2.imwrite(a.check, cv2.resize(grid, None, fx=2, fy=2, interpolation=cv2.INTER_NEAREST))
    print(json.dumps({"run": a.run_id, "stills": len(seq), "band_px": round(band, 2), "done": True}))


if __name__ == "__main__":
    main()
